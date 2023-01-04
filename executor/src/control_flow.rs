/*
 * Description: Use [`streams`] for control flow.
 *
 * Copyright (C) 2022-2023 Danny McClanahan <dmcC2@hypnicjerk.ai>
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

//! Use [`streams`] for control flow.

use crate::messaging::{Readable, Writable};

use async_trait::async_trait;
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
  where
    F: Fn(Y) -> T,
  {
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
    F: Fn(Acc, Self::WriteChunk) -> Acc + Send + Sync;
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
  use super::*;

  use crate::streams::traits::*;

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
      I: Iterator<Item = T>,
    {
      type ReadChunk = State<I::Item>;
    }

    impl<T, I> Peekable for Source<I>
    where
      T: Send,
      I: Iterator<Item = T> + Send,
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
      I: Iterator<Item = T> + Send,
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
      I: Iterator<Item = T> + Send,
    {
    }
  }
  pub use source::Source;

  mod sink {
    use super::*;

    use crate::streams::channels::*;

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
    /// // Coalesce Ok results.
    /// let (stream, sink) = Sink::<Result<u8, String>>::new();
    /// let sum = sink.fold::<Result<u8, String>, _>(Ok(0), |acc, cur| Ok(acc? + cur?));
    /// stream.push(State::Yielded(Ok(3))).unwrap();
    /// stream.push(State::Yielded(Ok(4))).unwrap();
    /// stream.push(State::Completed).unwrap();
    /// assert!(sum.await == Ok(7));
    ///
    /// // Exit after err.
    /// let (stream, sink) = Sink::<Result<u8, String>>::new();
    /// let sum = sink.fold::<Result<u8, String>, _>(Ok(0), |acc, cur| Ok(acc? + cur?));
    /// stream.push(State::Yielded(Ok(3))).unwrap();
    /// stream.push(State::Yielded(Err("err".to_string()))).unwrap();
    /// stream.push(State::Yielded(Ok(4))).unwrap();
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
        let (sender, receiver) = DuplexChannel::default().split_ends();
        (sender, Self { receiver })
      }
    }

    impl<T> Writable for Sink<T>
    where
      T: Send,
    {
      type WriteChunk = T;
    }

    #[async_trait]
    impl<T> Collector for Sink<T>
    where
      T: Send,
    {
      async fn fold<Acc, F>(self, init: Acc, f: F) -> Acc
      where
        Acc: Send,
        F: Fn(Acc, Self::WriteChunk) -> Acc + Send + Sync,
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
