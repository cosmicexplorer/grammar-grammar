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

/* #![warn(missing_docs)] */
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

/// Modelled off of the [rust generator nightly feature].
///
/// [rust generator nightly feature]: https://doc.rust-lang.org/stable/unstable-book/language-features/generators.html
pub mod streams {
  use grammar_grammar::messaging::{Readable, Writable};

  use async_channel;
  use async_mutex::Mutex;
  use async_trait::async_trait;
  use thiserror::Error;

  #[derive(Error, Debug, Clone)]
  pub enum StreamError {}

  /* pub trait Generator<R=()> { */
  /*   type Yield; */
  /*   type Return; */
  /*   fn resume(&mut self, resume: R) -> State<Self::Yield, Self::Return>; */
  /* } */
  pub enum State<Y> {
    Yielded(Y),
    Complete,
  }

  pub trait TryReadable: Readable+Send {
    fn try_read(&mut self) -> Option<Self::ReadChunk>;
  }

  #[async_trait]
  pub trait ReadableStream: Readable+Send {
    async fn read_one(&mut self) -> Self::ReadChunk;
  }

  #[async_trait]
  pub trait WritableStream: Writable+Send {
    async fn write_one(&mut self, chunk: Self::WriteChunk);
  }

  pub trait Duplex: ReadableStream+WritableStream {}

  /* #[derive(Debug, Clone)] */
  /* pub struct WrapperReader<T, S> */
  /* where S: Readable<ReadChunk=T> */
  /* { */
  /*   pub inner: Arc<S>, */
  /* } */

  /* impl<T, S> Readable for WrapperReader<T, S> */
  /* where S: Readable */
  /* { */
  /*   type ReadChunk = T; */
  /* } */

  /* #[async_trait] */
  /* impl<T, S> ReadableStream for WrapperReader<T, S> */
  /* where S: ReadableStream<ReadChunk=T> */
  /* { */
  /*   async fn read_one(&mut self) -> Self::ReadChunk { self.inner.read_one().await } */
  /* } */

  /* impl<T, S> TryReadable for WrapperReader<T, S> */
  /* where S: TryReadable<ReadChunk=T> */
  /* { */
  /*   fn try_read(&mut self) -> Option<Self::ReadChunk> { self.inner.try_read() } */
  /* } */

  #[derive(Debug, Clone)]
  pub struct ReadableChannel<T> {
    pub receiver: async_channel::Receiver<T>,
  }

  impl<T> Readable for ReadableChannel<T>
  where T: Send
  {
    type ReadChunk = T;
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

  impl<T> TryReadable for ReadableChannel<T>
  where T: Send
  {
    fn try_read(&mut self) -> Option<Self::ReadChunk> {
      match self.receiver.try_recv() {
        Ok(result) => Some(result),
        Err(async_channel::TryRecvError::Empty) => None,
        Err(e) => unreachable!("should never get this error: {}", e),
      }
    }
  }

  #[derive(Debug, Clone)]
  pub struct WritableChannel<T> {
    pub sender: async_channel::Sender<T>,
  }

  impl<T> Writable for WritableChannel<T>
  where T: Send
  {
    type WriteChunk = T;
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

  /* #[derive(Debug, Clone)] */
  /* pub struct WrapperWriter<T, S> */
  /* where S: Writable<WriteChunk=T> */
  /* { */
  /*   pub inner: S, */
  /* } */

  /* impl<T, S> Writable for WrapperWriter<T, S> { */
  /*   type WriteChunk = T; */
  /* } */

  /* #[async_trait] */
  /* impl<T, S> WritableStream for WrapperWriter<T, S> */
  /* where S: WritableStream<WriteChunk=T> */
  /* { */
  /*   async fn write_one(&mut self) -> Self::WriteChunk { self.inner.write_one().await } */
  /* } */

  /* #[derive(Debug, Clone)] */
  /* pub struct DuplexChannel<T> { */
  /*   pub sender: WritableChannel<T>, */
  /*   pub receiver: ReadableChannel<T>, */
  /* } */

  /* #[derive(Copy, Clone, Debug)] */
  /* pub enum BufferConfig { */
  /*   Finite(usize), */
  /*   Infinite, */
  /* } */

  /* impl<T> DuplexChannel<T> { */
  /*   pub fn buffered(config: BufferConfig) -> Self { */
  /*     let (sender, receiver) = match config { */
  /*       BufferConfig::Finite(size) => channel::bounded(size), */
  /*       BufferConfig::Infinite => channel::unbounded(), */
  /*     }; */
  /*     Self { */
  /*       sender: WritableChannel { sender }, */
  /*       receiver: ReadableChannel { receiver }, */
  /*     } */
  /*   } */
  /* } */

  /* impl<T> DuplexChannel<T> { */
  /*   pub fn unbounded() -> Self { */
  /*     let (sender, receiver) = channel::unbounded(); */
  /*     Self { */
  /*       sender: WritableChannel { sender }, */
  /*       receiver: ReadableChannel { receiver }, */
  /*     } */
  /*   } */
  /* } */

  pub struct Pipe<I, O> {
    input: Mutex<I>,
    output: Mutex<O>,
  }

  impl<I, O> Pipe<I, O>
  where
    I: ReadableStream,
    O: WritableStream,
  {
    pub fn pipe(input: I, output: O) -> Self {
      Self {
        input: Mutex::new(input),
        output: Mutex::new(output),
      }
    }
  }

  impl<I, O> Readable for Pipe<I, O>
  where O: Readable
  {
    type ReadChunk = O::ReadChunk;
  }

  #[async_trait]
  impl<I, O> ReadableStream for Pipe<I, O>
  where
    I: ReadableStream,
    O: Duplex<WriteChunk=I::ReadChunk>+TryReadable,
  {
    async fn read_one(&mut self) -> Self::ReadChunk {
      let Self { input, output } = self;
      /* (1) First, optimistically try getting a value out of the output stream. */
      let mut output = output.lock().await;
      if let Some(optimistic_output_chunk) = output.try_read() {
        return optimistic_output_chunk;
      }
      /* (2) If that fails, wait to get a value out of the input, then wait to write it to the
       * output! */
      async {
        let mut input = input.lock().await;
        let inner_chunk = input.read_one().await;
        output.write_one(inner_chunk).await;
      }
      .await;
      /* (3) Now, with the output lock still held, wait for any output from the output stream. */
      output.read_one().await
    }
  }

  impl<I, O> Writable for Pipe<I, O>
  where I: Writable
  {
    type WriteChunk = I::WriteChunk;
  }

  #[async_trait]
  impl<I, O> WritableStream for Pipe<I, O>
  where
    I: WritableStream+TryReadable,
    O: WritableStream<WriteChunk=I::ReadChunk>,
  {
    async fn write_one(&mut self, chunk: Self::WriteChunk) {
      let Self { input, output } = self;
      /* (1) First, wait to write the chunk to input. */
      let mut input = input.lock().await;
      input.write_one(chunk).await;
      /* (2) With the input lock still held, lock the output and optimistically check for any
       * output from the input stream. */
      let mut output = output.lock().await;
      if let Some(optimistic_inner_chunk) = input.try_read() {
        /* (2.1) If so, wait to write that into the output stream. */
        output.write_one(optimistic_inner_chunk).await;
      }
    }
  }

  impl<I, O> Duplex for Pipe<I, O>
  where
    I: Duplex+TryReadable,
    O: Duplex<WriteChunk=I::ReadChunk>+TryReadable,
  {
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
