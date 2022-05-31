/*
 * Description: Generate a token stream from text using regex.
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

//! Generate a token stream from text using [`regex`].

/* #![warn(missing_docs)] */
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

use grammar_grammar::{messaging, pipeline};

use regex;

use std::iter::Iterator;

pub struct StringInput(pub String);

/* pub enum ReadState<R> { */
/*   Ready(R), */
/*   Done, */
/* } */

/* impl messaging::Readable for StringInput { */
/*   type ReadChunk = ReadState<char>; */

/*   fn read(&mut self) -> Self::ReadChunk {} */
/* } */

/* impl Iterator for StringInput { */
/*   type Item = char; */
/* } */

/* pub enum Example { */
/*   LessThan, */
/*   GreaterThan, */
/* } */

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
  }
}
