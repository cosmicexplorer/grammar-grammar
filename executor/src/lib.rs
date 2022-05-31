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

  /* pub trait Generator<R=()> { */
  /*   type Yield; */
  /*   type Return; */
  /*   fn resume(self: Pin<&mut Self>, resume: R) -> State<Self::Yield, Self::Return>; */
  /* } */
  pub enum State<Y> {
    Yielded(Y),
    Complete,
  }

  pub trait ReadableStream: Readable {
    fn read(&mut self) -> State<Self::ReadChunk>;
  }

  pub trait WritableStream: Writable {
    fn write(&mut self, chunk: State<Self::WriteChunk>);
  }

  pub trait Duplex: ReadableStream+WritableStream {}

  pub struct Pipe<I, O> {
    input: I,
    output: O,
  }

  impl<I, O> Readable for Pipe<I, O>
  where O: Readable
  {
    type ReadChunk = O::ReadChunk;
  }

  /* impl<I, O> ReadableStream for Pipe<I, O> */
  /* where */
  /*   I: ReadableStream, */
  /*   O: Duplex, */
  /* { */
  /*   fn read(&mut self) -> State<Self::ReadChunk> {} */
  /* } */

  impl<I, O> Writable for Pipe<I, O>
  where I: Writable
  {
    type WriteChunk = I::WriteChunk;
  }

  /* impl<I, O> WritableStream for Pipe<I, O> */
  /* where I: WritableStream */
  /* { */
  /*   fn write(&mut self, chunk: State<Self::WriteChunk>) { self.input.write(chunk); } */
  /* } */

  /* impl<I, O> Duplex for Pipe<I, O> {} */

  /* impl<I, O> Pipe<I, O> { */
  /*   pub fn pipe<R>(self, from: R) -> Self */
  /*   where R: ReadableStream; */
  /* } */
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
  }
}
