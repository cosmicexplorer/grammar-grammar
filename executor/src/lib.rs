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

/// Variants of stream-like objects.
pub mod streams {
  use grammar_grammar::messaging::{Readable, Writable};

  use async_channel;
  use async_mutex::Mutex;
  use async_trait::async_trait;

  /// A synchronous readable stream-like interface for efficient piping logic.
  pub trait Peekable: Readable+Send {
    /// Pick off the top element, if possible without contention.
    fn peek(&mut self) -> Option<Self::ReadChunk>;
  }

  /// Asynchronously generate values.
  #[async_trait]
  pub trait ReadableStream: Readable+Send {
    /// Wait to pick off the top element.
    async fn read_one(&mut self) -> Self::ReadChunk;
  }

  /// Umbrella trait for readable streams.
  pub trait Read: Peekable+ReadableStream {}

  /// A synchronous writable stream-like interface for efficient piping logic.
  pub trait Pushable: Writable+Send {
    /// Push a top element, if possible without contention.
    fn push(&mut self, chunk: Self::WriteChunk) -> Result<(), Self::WriteChunk>;
  }

  /// Asynchronously receive values.
  #[async_trait]
  pub trait WritableStream: Writable+Send {
    /// Wait to push the top element.
    async fn write_one(&mut self, chunk: Self::WriteChunk);
  }

  /// Umbrella trait for writable streams.
  pub trait Write: Pushable+WritableStream {}

  /// Umbrella trait for readable/writable streams.
  pub trait Duplex: Read+Write {}

  /// An async channel implementing [Peekable] and [ReadableStream].
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
    fn peek(&mut self) -> Option<Self::ReadChunk> {
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
    async fn read_one(&mut self) -> Self::ReadChunk {
      self
        .receiver
        .recv()
        .await
        .expect(".recv() should never error")
    }
  }

  impl<T> Read for ReadableChannel<T> where T: Send {}

  /// An async channel implementing [WritableStream].
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
    fn push(&mut self, chunk: Self::WriteChunk) -> Result<(), Self::WriteChunk> {
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
    async fn write_one(&mut self, chunk: Self::WriteChunk) {
      self
        .sender
        .send(chunk)
        .await
        .expect(".send() should never error")
    }
  }

  impl<T> Write for WritableChannel<T> where T: Send {}

  /// An async channel implementing [Duplex].
  ///
  ///```
  /// # fn main() {
  /// # futures::executor::block_on(async {
  /// use grammar_executor::streams::*;
  ///
  /// // Infinite buffers.
  /// let mut unbounded = DuplexChannel::<u8>::buffered(BufferConfig::Infinite);
  /// unbounded.write_one(5).await;
  /// assert!(5 == unbounded.read_one().await);
  /// unbounded.write_one(6).await;
  /// assert!(6 == unbounded.peek().unwrap());
  ///
  /// // Bounded buffers.
  /// let mut bounded = DuplexChannel::<u8>::buffered(BufferConfig::Finite(3));
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

  /// Types of channel inside a [`DuplexChannel`].
  #[derive(Copy, Clone, Debug)]
  pub enum BufferConfig {
    /// Choose a [bounded](async_channel::bounded) channel.
    Finite(usize),
    /// Choose an [unbounded](async_channel::unbounded) channel.
    Infinite,
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
  }

  impl<T> Readable for DuplexChannel<T>
  where T: Send
  {
    type ReadChunk = T;
  }

  impl<T> Peekable for DuplexChannel<T>
  where T: Send
  {
    fn peek(&mut self) -> Option<Self::ReadChunk> {
      let Self { receiver, .. } = self;
      receiver.peek()
    }
  }

  #[async_trait]
  impl<T> ReadableStream for DuplexChannel<T>
  where T: Send
  {
    async fn read_one(&mut self) -> Self::ReadChunk {
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
    fn push(&mut self, chunk: Self::WriteChunk) -> Result<(), Self::WriteChunk> {
      let Self { sender, .. } = self;
      sender.push(chunk)
    }
  }

  #[async_trait]
  impl<T> WritableStream for DuplexChannel<T>
  where T: Send
  {
    async fn write_one(&mut self, chunk: Self::WriteChunk) {
      let Self { sender, .. } = self;
      sender.write_one(chunk).await;
    }
  }

  impl<T> Write for DuplexChannel<T> where T: Send {}

  impl<T> Duplex for DuplexChannel<T> where T: Send {}

  /// A wrapper over two streams, one of which receives the other's output as input.
  ///
  ///```
  /// # fn main() {
  /// # futures::executor::block_on(async {
  /// use grammar_executor::streams::*;
  ///
  /// let left = DuplexChannel::<u8>::buffered(BufferConfig::Infinite);
  /// let right = DuplexChannel::<u8>::buffered(BufferConfig::Infinite);
  /// let mut pipe = Pipe::pipe(left, right);
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

  impl<I, O> Pipe<I, O>
  where
    I: ReadableStream,
    O: WritableStream,
  {
    /// Connect the input to the output stream.
    pub fn pipe(input: I, output: O) -> Self {
      Self {
        input,
        bottleneck: Mutex::new(()),
        output,
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
    I: Send,
    O: Peekable,
  {
    fn peek(&mut self) -> Option<Self::ReadChunk> {
      let Self { output, .. } = self;
      /* Try getting a value out of the output stream, but don't attempt to ferry anything from the
       * input stream. */
      output.peek()
    }
  }

  #[async_trait]
  impl<I, O> ReadableStream for Pipe<I, O>
  where
    I: Read,
    O: Duplex<WriteChunk=I::ReadChunk>,
  {
    async fn read_one(&mut self) -> Self::ReadChunk {
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
      /* (3) Wait to get the result of transforming those queued values from the output stream. */
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
    O: Send,
  {
    fn push(&mut self, chunk: Self::WriteChunk) -> Result<(), Self::WriteChunk> {
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
    async fn write_one(&mut self, chunk: Self::WriteChunk) {
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

/// Use [streams] for control flow.
pub mod control_flow {
  use super::streams;

  /// Modelled off of the [rust generator nightly feature].
  ///
  /// Note that `Y` may be a [`Result`]!
  ///
  /// [rust generator nightly feature]: https://doc.rust-lang.org/stable/unstable-book/language-features/generators.html
  /* pub trait Generator<R=()> { */
  /*   type Yield; */
  /*   type Return; */
  /*   fn resume(&mut self, resume: R) -> State<Self::Yield, Self::Return>; */
  /* } */
  pub enum State<Y> {
    Yielded(Y),
    Complete,
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
  }
}
