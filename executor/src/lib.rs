/*
 * Description: An execution framework for parser generators.
 *
 * Copyright (C) 2022 Danny McClanahan <dmcC2@hypnicjerk.ai>
 * SPDX-License-Identifier: LGPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! An execution framework for parser generators.

#![warn(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

use grammar_grammar::messaging::{Readable, Writable};

use async_trait::async_trait;

mod utils;

/// Variants of stream-like objects.
pub mod streams {
  use super::*;

  /// Stream interfaces.
  pub mod traits {
    use super::*;

    /// A synchronous readable stream-like interface for efficient piping logic.
    pub trait Peekable: Readable+Send+Sync {
      /// Pick off the top element, if possible without contention.
      fn peek(&self) -> Option<Self::ReadChunk>;
    }

    /// Asynchronously generate values.
    #[async_trait]
    pub trait ReadableStream: Readable+Send+Sync {
      /// Wait to pick off the top element.
      async fn read_one(&self) -> Self::ReadChunk;
    }

    /// Umbrella trait for readable streams.
    pub trait Read: Peekable+ReadableStream {}

    /// A synchronous writable stream-like interface for efficient piping logic.
    pub trait Pushable: Writable+Send+Sync {
      /// Push a top element, if possible without contention.
      fn push(&self, chunk: Self::WriteChunk) -> Result<(), Self::WriteChunk>;
    }

    /// Asynchronously receive values.
    #[async_trait]
    pub trait WritableStream: Writable+Send+Sync {
      /// Wait to push the top element.
      async fn write_one(&self, chunk: Self::WriteChunk);
    }

    /// Umbrella trait for writable streams.
    pub trait Write: Pushable+WritableStream {}

    /// Umbrella trait for readable/writable streams.
    pub trait Duplex: Read+Write {}
  }

  /// Implementations of stream traits using [`async_channel`].
  pub mod channels {
    use super::{traits::*, *};

    use async_channel;

    mod readable {
      use super::*;

      /// An async channel implementing [`Peekable`] and [`ReadableStream`].
      #[derive(Debug, Clone)]
      pub struct ReadableChannel<T> {
        receiver: async_channel::Receiver<T>,
      }

      impl<T> ReadableChannel<T> {
        /// Create a new readable channel.
        pub fn new(receiver: async_channel::Receiver<T>) -> Self { Self { receiver } }
      }

      impl<T> Readable for ReadableChannel<T>
      where T: Send
      {
        type ReadChunk = T;
      }

      impl<T> Peekable for ReadableChannel<T>
      where T: Send
      {
        fn peek(&self) -> Option<Self::ReadChunk> {
          match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(async_channel::TryRecvError::Empty) => None,
            Err(e) => unreachable!("should never get this error peeking: {}", e),
          }
        }
      }

      #[async_trait]
      impl<T> ReadableStream for ReadableChannel<T>
      where T: Send
      {
        async fn read_one(&self) -> Self::ReadChunk {
          self
            .receiver
            .recv()
            .await
            .expect(".recv() should never error")
        }
      }

      impl<T> Read for ReadableChannel<T> where T: Send {}
    }
    pub use readable::ReadableChannel;

    mod writable {
      use super::*;

      /// An async channel implementing [`WritableStream`].
      #[derive(Debug, Clone)]
      pub struct WritableChannel<T> {
        sender: async_channel::Sender<T>,
      }

      impl<T> WritableChannel<T> {
        /// Create a new writable channel.
        pub fn new(sender: async_channel::Sender<T>) -> Self { Self { sender } }
      }

      impl<T> Writable for WritableChannel<T>
      where T: Send
      {
        type WriteChunk = T;
      }

      impl<T> Pushable for WritableChannel<T>
      where T: Send
      {
        fn push(&self, chunk: Self::WriteChunk) -> Result<(), Self::WriteChunk> {
          match self.sender.try_send(chunk) {
            Ok(()) => Ok(()),
            Err(async_channel::TrySendError::Full(val)) => Err(val),
            Err(e) => unreachable!("should never get this error pushing: {}", e),
          }
        }
      }

      #[async_trait]
      impl<T> WritableStream for WritableChannel<T>
      where T: Send
      {
        async fn write_one(&self, chunk: Self::WriteChunk) {
          self
            .sender
            .send(chunk)
            .await
            .expect(".send() should never error")
        }
      }

      impl<T> Write for WritableChannel<T> where T: Send {}
    }
    pub use writable::WritableChannel;

    mod duplex {
      use super::*;

      use displaydoc::Display;

      /// Types of channel inside a [`DuplexChannel`].
      #[derive(Copy, Clone, Debug, Display)]
      #[ignore_extra_doc_attributes]
      pub enum BufferConfig {
        /// <finite buffer: {0}>
        ///
        /// Choose a [bounded](async_channel::bounded) channel.
        Finite(usize),
        /// <infinite buffer>
        ///
        /// Choose an [unbounded](async_channel::unbounded) channel.
        Infinite,
      }

      /// An async channel implementing [`Duplex`].
      ///
      ///```
      /// # fn main() {
      /// # futures::executor::block_on(async {
      /// use grammar_executor::streams::{traits::*, channels::*};
      ///
      /// // Infinite buffers.
      /// let unbounded = DuplexChannel::<u8>::buffered(BufferConfig::Infinite);
      /// unbounded.write_one(5).await;
      /// assert!(5 == unbounded.read_one().await);
      /// unbounded.write_one(6).await;
      /// assert!(6 == unbounded.peek().unwrap());
      ///
      /// // Bounded buffers.
      /// let bounded = DuplexChannel::<u8>::buffered(BufferConfig::Finite(3));
      /// bounded.write_one(7).await;
      /// assert!(7 == bounded.read_one().await);
      /// # })
      /// # }
      ///```
      #[derive(Debug, Clone)]
      pub struct DuplexChannel<T> {
        sender: WritableChannel<T>,
        receiver: ReadableChannel<T>,
      }

      impl<T> DuplexChannel<T> {
        /// Generate a duplex channel with the given buffering specification.
        pub fn buffered(config: BufferConfig) -> Self {
          let (sender, receiver) = match config {
            BufferConfig::Finite(size) => async_channel::bounded(size),
            BufferConfig::Infinite => async_channel::unbounded(),
          };
          Self {
            sender: WritableChannel::new(sender),
            receiver: ReadableChannel::new(receiver),
          }
        }

        /// Extract the read and write ends.
        pub fn split_ends(self) -> (WritableChannel<T>, ReadableChannel<T>) {
          let Self { sender, receiver } = self;
          (sender, receiver)
        }
      }

      impl<T> Readable for DuplexChannel<T>
      where T: Send
      {
        type ReadChunk = T;
      }

      impl<T> Peekable for DuplexChannel<T>
      where T: Send
      {
        fn peek(&self) -> Option<Self::ReadChunk> {
          let Self { receiver, .. } = self;
          receiver.peek()
        }
      }

      #[async_trait]
      impl<T> ReadableStream for DuplexChannel<T>
      where T: Send
      {
        async fn read_one(&self) -> Self::ReadChunk {
          let Self { receiver, .. } = self;
          receiver.read_one().await
        }
      }

      impl<T> Read for DuplexChannel<T> where T: Send {}

      impl<T> Writable for DuplexChannel<T>
      where T: Send
      {
        type WriteChunk = T;
      }

      impl<T> Pushable for DuplexChannel<T>
      where T: Send
      {
        fn push(&self, chunk: Self::WriteChunk) -> Result<(), Self::WriteChunk> {
          let Self { sender, .. } = self;
          sender.push(chunk)
        }
      }

      #[async_trait]
      impl<T> WritableStream for DuplexChannel<T>
      where T: Send
      {
        async fn write_one(&self, chunk: Self::WriteChunk) {
          let Self { sender, .. } = self;
          sender.write_one(chunk).await;
        }
      }

      impl<T> Write for DuplexChannel<T> where T: Send {}

      impl<T> Duplex for DuplexChannel<T> where T: Send {}
    }
    pub use duplex::{BufferConfig, DuplexChannel};
  }

  /// Implementations of stream traits that transform or combine other streams.
  pub mod combinators {
    use super::{traits::*, *};

    mod pipe {
      use crate::control_flow::State;

      use super::*;

      use async_mutex::Mutex;

      /// A wrapper over two streams, one of which receives the other's output as input.
      ///
      ///```
      /// # fn main() {
      /// # futures::executor::block_on(async {
      /// use grammar_executor::streams::{traits::*, channels::*, combinators::Pipe};
      ///
      /// let left = DuplexChannel::<u8>::buffered(BufferConfig::Infinite);
      /// let right = DuplexChannel::<u8>::buffered(BufferConfig::Infinite);
      /// let pipe = Pipe::pipe(left, right);
      ///
      /// pipe.write_one(5).await;
      /// assert!(5 == pipe.read_one().await);
      /// # })
      /// # }
      ///```
      pub struct Pipe<I, O> {
        input: I,
        bottleneck: Mutex<()>,
        output: O,
      }

      impl<I, O> Pipe<I, O> {
        /// Connect the input to the output stream.
        pub fn pipe(input: I, output: O) -> Self {
          Self {
            input,
            bottleneck: Mutex::new(()),
            output,
          }
        }
      }

      impl<T, I, O> Pipe<I, O>
      where
        I: ReadableStream<ReadChunk=State<T>>,
        O: WritableStream<WriteChunk=State<T>>,
      {
        /// Extract all elements from the input stream and ferry them to the output stream.
        ///
        /// In cases where *neither* end of the pipe is explicitly pulling or pushing, the pipe will
        /// have no way to advance (this can occur e.g. if `I` is
        /// [`Source`](crate::control_flow::adapters::Source) and `O` is
        /// [`Sink`](crate::control_flow::adapters::Sink)). In that case, this method
        /// is necessary.
        ///
        /// This method's containing impl specializes for the case where the ferried chunk is
        /// a `State<T>`, because otherwise there is no concept of "completion", and the method
        /// would never return.
        ///
        ///```
        /// # fn main() {
        /// # futures::executor::block_on(async {
        /// use grammar_executor::{
        ///   streams::combinators::Pipe,
        ///   control_flow::{Collector, adapters::{Source, Sink}},
        /// };
        ///
        /// let source = Source::new([3, 4].into_iter());
        /// let (stream, sink) = Sink::<u8>::new();
        /// let pipe = Pipe::pipe(source, stream);
        /// let sum = sink.fold(0, |acc, cur| acc + cur);
        /// pipe.iterate().await;
        /// assert!(sum.await == 7);
        /// # })
        /// # }
        ///```
        pub async fn iterate(self) {
          let Self {
            input,
            bottleneck,
            output,
          } = self;
          let _ = bottleneck.lock().await;
          loop {
            match input.read_one().await {
              State::Yielded(chunk) => {
                output.write_one(State::Yielded(chunk)).await;
              },
              State::Completed => {
                output.write_one(State::Completed).await;
                return;
              },
            }
          }
        }
      }

      impl<I, O> Readable for Pipe<I, O>
      where O: Readable
      {
        type ReadChunk = O::ReadChunk;
      }

      impl<I, O> Peekable for Pipe<I, O>
      where
        I: Send+Sync,
        O: Peekable,
      {
        fn peek(&self) -> Option<Self::ReadChunk> {
          let Self { output, .. } = self;
          /* Try getting a value out of the output stream, but don't attempt to ferry anything from
           * the input stream. */
          output.peek()
        }
      }

      #[async_trait]
      impl<I, O> ReadableStream for Pipe<I, O>
      where
        I: Read,
        O: Duplex<WriteChunk=I::ReadChunk>,
      {
        async fn read_one(&self) -> Self::ReadChunk {
          let Self {
            input,
            bottleneck,
            output,
          } = self;
          /* (1) First, optimistically try getting a value out of the output stream. */
          if let Some(optimistic_output_chunk) = output.peek() {
            return optimistic_output_chunk;
          }
          /* (2) If not available, try to enter the critical section if uncontended. */
          if let Some(_) = bottleneck.try_lock() {
            /* (2.1) Ferry over any queued inner chunks. */
            while let Some(inner_chunk) = input.peek() {
              /* (2.1.1) Wait to write that value into the output stream. */
              output.write_one(inner_chunk).await;
            }
          }
          /* (3) Wait to get the result of transforming those queued values from the output
           * stream. */
          output.read_one().await
        }
      }

      impl<I, O> Read for Pipe<I, O>
      where
        I: Read,
        O: Duplex<WriteChunk=I::ReadChunk>,
      {
      }

      impl<I, O> Writable for Pipe<I, O>
      where I: Writable
      {
        type WriteChunk = I::WriteChunk;
      }

      impl<I, O> Pushable for Pipe<I, O>
      where
        I: Pushable,
        O: Send+Sync,
      {
        fn push(&self, chunk: Self::WriteChunk) -> Result<(), Self::WriteChunk> {
          let Self { input, .. } = self;
          input.push(chunk)
        }
      }

      #[async_trait]
      impl<I, O> WritableStream for Pipe<I, O>
      where
        I: Duplex<ReadChunk=O::WriteChunk>,
        O: WritableStream,
      {
        async fn write_one(&self, chunk: Self::WriteChunk) {
          let Self {
            input,
            bottleneck,
            output,
          } = self;
          /* (1) First, wait to write the chunk to input. */
          input.write_one(chunk).await;
          /* (2) Enter the critical section and ferry over any queued inner chunks. */
          let _ = bottleneck.lock().await;
          /* (2.1) Ferry over any queued input chunks. */
          while let Some(inner_chunk) = input.peek() {
            /* (2.1.1) If we could get an input chunk, then wait to write that chunk to output. */
            output.write_one(inner_chunk).await;
          }
        }
      }

      impl<I, O> Write for Pipe<I, O>
      where
        I: Duplex<ReadChunk=O::WriteChunk>,
        O: WritableStream,
      {
      }

      impl<I, O> Duplex for Pipe<I, O>
      where
        I: Duplex,
        O: Duplex<WriteChunk=I::ReadChunk>,
      {
      }
    }
    pub use pipe::Pipe;

    mod map {
      use super::*;

      mod read {
        use super::*;

        /// Map a function over the outputs of a readable stream.
        ///
        ///```
        /// # fn main() {
        /// # futures::executor::block_on(async {
        /// use grammar_executor::{
        ///   streams::combinators::{Pipe, ReadMap},
        ///   control_flow::{State, Collector, adapters::{Source, Sink}},
        /// };
        ///
        /// let source = Source::new([3, 4].into_iter());
        /// let shifted_source = ReadMap::new(source, |x: State<u8>| x.map_state(|x| x + 1));
        /// let (stream, sink) = Sink::<u8>::new();
        /// let pipe = Pipe::pipe(shifted_source, stream);
        /// let sum = sink.fold(0, |acc, cur| acc + cur);
        /// pipe.iterate().await;
        /// assert!(sum.await == 9);
        /// # })
        /// # }
        ///```
        pub struct ReadMap<S, R, F>
        where
          R: Readable,
          F: Fn(R::ReadChunk) -> S,
        {
          inner: R,
          transformer: F,
        }

        impl<S, R, F> ReadMap<S, R, F>
        where
          R: Readable,
          F: Fn(R::ReadChunk) -> S,
        {
          /// Construct a new instance mapping elements of readable stream `inner` with
          /// `transformer`.
          pub fn new(inner: R, transformer: F) -> Self { Self { inner, transformer } }
        }

        impl<S, R, F> Readable for ReadMap<S, R, F>
        where
          S: Send,
          R: Readable,
          F: Fn(R::ReadChunk) -> S,
        {
          type ReadChunk = S;
        }

        impl<S, R, F> Peekable for ReadMap<S, R, F>
        where
          S: Send,
          R: Peekable,
          F: Fn(R::ReadChunk) -> S+Send+Sync,
        {
          fn peek(&self) -> Option<Self::ReadChunk> {
            let Self { inner, transformer } = self;
            inner.peek().map(transformer)
          }
        }

        #[async_trait]
        impl<S, R, F> ReadableStream for ReadMap<S, R, F>
        where
          S: Send,
          R: ReadableStream,
          F: Fn(R::ReadChunk) -> S+Send+Sync,
        {
          async fn read_one(&self) -> Self::ReadChunk {
            (self.transformer)(self.inner.read_one().await)
          }
        }

        impl<S, R, F> Read for ReadMap<S, R, F>
        where
          S: Send,
          R: Read,
          F: Fn(R::ReadChunk) -> S+Send+Sync,
        {
        }
      }
      pub use read::ReadMap;

      mod write {
        use super::*;

        use crate::utils::_Phantom;

        /// Map a function over the inputs to a writable stream.
        ///
        ///```
        /// # fn main() {
        /// # futures::executor::block_on(async {
        /// use grammar_executor::{
        ///   streams::combinators::{Pipe, WriteMap},
        ///   control_flow::{State, Collector, adapters::{Source, Sink}},
        /// };
        ///
        /// let source = Source::new([3, 4].into_iter());
        /// let (stream, sink) = Sink::<u8>::new();
        /// let shifted_stream = WriteMap::new(stream, |x: State<u8>| x.map_state(|x| x + 1));
        /// let pipe = Pipe::pipe(source, shifted_stream);
        /// let sum = sink.fold(0, |acc, cur| acc + cur);
        /// pipe.iterate().await;
        /// assert!(sum.await == 9);
        /// # })
        /// # }
        ///```
        pub struct WriteMap<S, W, F>
        where
          W: Writable,
          F: Fn(S) -> W::WriteChunk,
        {
          /* NB: This appears to be a compiler bug that doesn't occur with ReadMap (where S is in
           * the return position). */
          _ph: _Phantom<S>,
          inner: W,
          transformer: F,
        }

        impl<S, W, F> WriteMap<S, W, F>
        where
          W: Writable,
          F: Fn(S) -> W::WriteChunk,
        {
          /// Construct a new instance mapping elements of writable stream `inner` with
          /// `transformer`.
          pub fn new(inner: W, transformer: F) -> Self {
            Self {
              inner,
              transformer,
              _ph: _Phantom::new(),
            }
          }
        }

        impl<S, W, F> Writable for WriteMap<S, W, F>
        where
          S: Send,
          W: Writable,
          F: Fn(S) -> W::WriteChunk,
        {
          type WriteChunk = S;
        }

        #[async_trait]
        impl<S, W, F> WritableStream for WriteMap<S, W, F>
        where
          S: Send,
          W: WritableStream,
          F: Fn(S) -> W::WriteChunk+Send+Sync,
        {
          async fn write_one(&self, chunk: Self::WriteChunk) {
            let Self {
              inner, transformer, ..
            } = self;
            inner.write_one(transformer(chunk)).await;
          }
        }
      }
      pub use write::WriteMap;
    }
    pub use map::{ReadMap, WriteMap};
  }
}

/// Use [`streams`] for control flow.
pub mod control_flow {
  use super::*;

  use displaydoc::Display;

  /// Modelled off of the [rust generator nightly feature][generator].
  ///
  /// Note that `Y` may be a [`Result`]!
  ///
  /// [generator]: https://doc.rust-lang.org/stable/unstable-book/language-features/generators.html
  #[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
  pub enum State<Y> {
    /// <yielded {0}>
    Yielded(Y),
    /// <completed>
    Completed,
  }

  impl<Y> State<Y> {
    /// Map a function over any yielded value, or propagate completeness.
    ///
    ///```
    /// use grammar_executor::control_flow::State;
    ///
    /// let f = |x| x + 1;
    /// assert!(State::Yielded(0).map_state(f) == State::Yielded(1));
    /// assert!(State::Yielded(1).map_state(f) == State::Yielded(2));
    /// assert!(State::Completed.map_state(f) == State::Completed);
    ///```
    pub fn map_state<T, F>(self, f: F) -> State<T>
    where F: Fn(Y) -> T {
      match self {
        Self::Yielded(val) => State::Yielded(f(val)),
        Self::Completed => State::Completed,
      }
    }
  }

  /// Interface to apply folds over streams.
  #[async_trait]
  pub trait Collector: Writable {
    /// Fold over the values of the stream.
    async fn fold<Acc, F>(self, init: Acc, f: F) -> Acc
    where
      Acc: Send,
      F: Fn(Acc, Self::WriteChunk) -> Acc+Send+Sync;
  }

  /// "Primitive" implementations of stream traits.
  ///
  ///```
  /// # fn main() {
  /// # futures::executor::block_on(async {
  /// use grammar_executor::{
  ///   streams::combinators::Pipe,
  ///   control_flow::{Collector, adapters::{Source, Sink}},
  /// };
  ///
  /// let source = Source::new([3, 4].into_iter());
  /// let (stream, sink) = Sink::<u8>::new();
  /// let pipe = Pipe::pipe(source, stream);
  /// let sum = sink.fold(0, |acc, cur| acc + cur);
  /// pipe.iterate().await;
  /// assert!(sum.await == 7);
  /// # })
  /// # }
  ///```
  pub mod adapters {
    use super::{streams::traits::*, *};

    use async_mutex::Mutex;

    mod source {
      use super::*;

      use std::iter::Iterator;

      /// Provide values from a non-async iterable.
      ///
      ///```
      /// use grammar_executor::{streams::traits::*, control_flow::{State, adapters::Source}};
      ///
      /// let source = Source::new([3, 4].into_iter());
      /// assert!(source.peek().unwrap() == State::Yielded(3));
      /// assert!(source.peek().unwrap() == State::Yielded(4));
      /// assert!(source.peek().unwrap() == State::Completed);
      ///```
      #[derive(Debug)]
      pub struct Source<I> {
        static_elements: Mutex<I>,
      }

      impl<I> Source<I> {
        /// Construct a new instance with the iterator `static_elements`.
        pub fn new(static_elements: I) -> Self {
          Self {
            static_elements: Mutex::new(static_elements),
          }
        }
      }

      impl<T, I> Readable for Source<I>
      where
        T: Send,
        I: Iterator<Item=T>,
      {
        type ReadChunk = State<I::Item>;
      }

      impl<T, I> Peekable for Source<I>
      where
        T: Send,
        I: Iterator<Item=T>+Send,
      {
        fn peek(&self) -> Option<Self::ReadChunk> {
          self
            .static_elements
            .try_lock()
            .map(|mut els| match els.next() {
              Some(val) => State::Yielded(val),
              None => State::Completed,
            })
        }
      }

      #[async_trait]
      impl<T, I> ReadableStream for Source<I>
      where
        T: Send,
        I: Iterator<Item=T>+Send,
      {
        async fn read_one(&self) -> Self::ReadChunk {
          let mut els = self.static_elements.lock().await;
          match els.next() {
            Some(val) => State::Yielded(val),
            None => State::Completed,
          }
        }
      }

      impl<T, I> Read for Source<I>
      where
        T: Send,
        I: Iterator<Item=T>+Send,
      {
      }
    }
    pub use source::Source;

    mod sink {
      use super::{streams::channels::*, *};

      /// Collect values from another stream.
      ///
      ///```
      /// # fn main() {
      /// # futures::executor::block_on(async {
      /// use grammar_executor::{streams::traits::*, control_flow::{*, adapters::Sink}};
      ///
      /// let (stream, sink) = Sink::<u8>::new();
      /// let sum = sink.fold(0, |acc, cur| acc + cur);
      /// stream.push(State::Yielded(3)).unwrap();
      /// stream.push(State::Yielded(4)).unwrap();
      /// stream.push(State::Completed).unwrap();
      /// assert!(sum.await == 7);
      /// # })
      /// # }
      ///```
      ///
      /// This can also be used to coalesce [`Result`]s:
      ///```
      /// # fn main() {
      /// # futures::executor::block_on(async {
      /// use grammar_executor::{streams::traits::*, control_flow::{*, adapters::Sink}};
      ///
      /// // ???
      /// let (stream, sink) = Sink::<Result<u8, String>>::new();
      /// let sum = sink.fold::<Result<u8, String>, _>(Ok(0), |acc, cur| Ok(acc? + cur?));
      /// stream.push(State::Yielded(Ok(3))).unwrap();
      /// stream.push(State::Yielded(Ok(4))).unwrap();
      /// stream.push(State::Completed).unwrap();
      /// assert!(sum.await == Ok(7));
      ///
      /// // ???
      /// let (stream, sink) = Sink::<Result<u8, String>>::new();
      /// let sum = sink.fold::<Result<u8, String>, _>(Ok(0), |acc, cur| Ok(acc? + cur?));
      /// stream.push(State::Yielded(Ok(3))).unwrap();
      /// stream.push(State::Yielded(Err("err".to_string()))).unwrap();
      /// stream.push(State::Completed).unwrap();
      /// assert!(sum.await == Err("err".to_string()));
      /// # })
      /// # }
      ///```
      #[derive(Debug)]
      pub struct Sink<T> {
        receiver: ReadableChannel<State<T>>,
      }

      impl<T> Sink<T> {
        /// Create a new (`writable_stream`, `sink`) pair.
        pub fn new() -> (WritableChannel<State<T>>, Self) {
          let (sender, receiver) = DuplexChannel::buffered(BufferConfig::Infinite).split_ends();
          (sender, Self { receiver })
        }
      }

      impl<T> Writable for Sink<T>
      where T: Send
      {
        type WriteChunk = T;
      }

      #[async_trait]
      impl<T> Collector for Sink<T>
      where T: Send
      {
        async fn fold<Acc, F>(self, init: Acc, f: F) -> Acc
        where
          Acc: Send,
          F: Fn(Acc, Self::WriteChunk) -> Acc+Send+Sync,
        {
          let mut cur = init;
          loop {
            match self.receiver.read_one().await {
              State::Yielded(chunk) => {
                cur = f(cur, chunk);
              },
              State::Completed => {
                return cur;
              },
            }
          }
        }
      }
    }
    pub use sink::Sink;
  }
}
