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

/* use std::iter::Iterator; */

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

  use regex::{Regex, RegexSet};

  #[test]
  fn it_works() {
    #[derive(Debug, PartialEq, Eq)]
    enum Tokens {
      Sym(String),
      Dot,
      OpenParen,
      CloseParen,
    }

    impl Tokens {
      pub fn patterns() -> RegexSet {
        RegexSet::new(&["[a-zA-Z_\\-][a-zA-Z_\\-0-9]*", "\\.", "\\(", "\\)"]).unwrap()
      }

      pub fn match_index(text: &str, match_idx: usize) -> Self {
        match match_idx {
          0 => Self::Sym(text.to_string()),
          1 => Self::Dot,
          2 => Self::OpenParen,
          3 => Self::CloseParen,
          _ => unreachable!("only 4 patterns defined!"),
        }
      }
    }

    let text = "a.b()";

    // Compile a set matching any of our patterns.
    let set = Tokens::patterns();
    // Compile each pattern independently.
    let regexes: Vec<_> = set
      .patterns()
      .iter()
      .map(|pat| Regex::new(pat).unwrap())
      .collect();

    // Match against the whole set first and identify the individual
    // matching patterns.
    let matches: Vec<Tokens> = set
      .matches(text)
      .into_iter()
      // Dereference the match index to get the corresponding
      // compiled pattern.
      .map(|match_idx| (&regexes[match_idx], match_idx))
      // To get match locations or any other info, we then have to search
      // the exact same text again, using our separately-compiled pattern.
      .map(|(pat, match_idx)| (pat.find(text).unwrap().as_str(), match_idx))
      .map(|(text, match_idx)| Tokens::match_index(text, match_idx))
      .collect();

    assert_eq!(
      vec![
        Tokens::Sym("a".to_string()),
        Tokens::Dot,
        /* FIXME: this doesn't work because we only search for matches once with .find() above! */
        Tokens::Sym("b".to_string()),
        Tokens::OpenParen,
        Tokens::CloseParen,
      ],
      matches
    );
  }
}
