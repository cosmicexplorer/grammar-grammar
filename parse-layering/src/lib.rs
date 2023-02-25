/*
 * Description: A framework to combine parsers in layers.
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

//! A framework to combine parsers in layers.

#![warn(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

use async_trait::async_trait;
use regex::{Regex, RegexSet};

use std::iter::Iterator;

/* TODO: the below, via regex! */
/* ["a.", "b()"] -> ["a", ".b", "(", ")"] */
#[async_trait]
trait ParseStream {
  type Input;
  type Output;
  async fn read(&mut self);
  async fn compute(&mut self);
  async fn write(&mut self);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let patterns = ["a\\.", "b\\(\\)"];
    let text = "a.b()";

    // Compile a set matching any of our patterns.
    let set = RegexSet::new(&patterns).unwrap();
    // Compile each pattern independently.
    let regexes: Vec<_> = set
      .patterns()
      .iter()
      .map(|pat| Regex::new(pat).unwrap())
      .collect();

    // Match against the whole set first and identify the individual
    // matching patterns.
    let matches: Vec<&str> = set
      .matches(text)
      .into_iter()
      // Dereference the match index to get the corresponding
      // compiled pattern.
      .map(|match_idx| &regexes[match_idx])
      // To get match locations or any other info, we then have to search
      // the exact same text again, using our separately-compiled pattern.
      .map(|pat| pat.find(text).unwrap().as_str())
      .collect();

    assert_eq!(vec!["a.", "b()"], matches);
  }
}
