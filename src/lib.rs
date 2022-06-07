/*
 * Description: A grammar specification language.
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

//! A grammar specification language.

#![no_std]
/* #![warn(missing_docs)] */
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

pub mod utils;

pub mod messaging {
  pub trait Readable {
    type ReadChunk: Send;
  }

  pub trait Writable {
    type WriteChunk: Send;
  }
}

pub mod collection {
  use crate::components::{direct, indirect, synthesis};

  use grammar_type_info::{Type, TypeInfo};

  pub trait GrammarCase {
    type Tok: direct::Token;
    type Ref: indirect::Reference;
    type Elements: Iterator<Item = synthesis::CaseElement<Self::Tok, Self::Ref>>;
    fn elements(&self) -> Self::Elements;
  }

  pub trait Collector {
    type Args;
    type Result: TypeInfo;
    fn collect(&self, args: Self::Args) -> Self::Result;
    fn result_type() -> Type {
      Self::Result::TYPE
    }
  }
}

pub mod components {
  pub mod direct {
    use crate::pipeline::{CoversExtent, InputElement};

    pub trait Token: CoversExtent {
      type Source: InputElement;
    }
  }

  pub mod indirect {
    pub trait Reference {}
  }

  /* pub mod parallel {} */

  pub mod synthesis {
    use crate::collection::{Collector, GrammarCase};

    use displaydoc::Display;

    use core::iter::IntoIterator;

    #[derive(Debug, Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum CaseElement<Tok, Ref> {
      /// <tok {0}>
      Tok(Tok),
      /// <ref {0}>
      Ref(Ref),
    }

    pub trait Case: GrammarCase + Collector {}

    pub trait Production: IntoIterator {
      type Result;
      type C: Case<Result = Self::Result>;
      type Item: Into<Self::C>;
    }

    pub trait Grammar: IntoIterator {
      type P: Production;
      type Item: Into<(<<Self::P as Production>::C as GrammarCase>::Ref, Self::P)>;
    }
  }
}

pub mod pipeline {
  use super::{
    components::{direct::Token, indirect::Reference},
    messaging::{Readable, Writable},
  };

  use core::ops::Range;

  pub trait CoversExtent: AsRef<[Self::Source]> {
    type Source;
    fn covers_extent(&self) -> Range<usize>;
  }

  pub trait InputElement {}

  pub trait Input: Readable {
    type ReadChunk: InputElement;
  }

  pub trait Tokenizer: Readable + Writable {
    type WriteChunk: InputElement;
    type ReadChunk: Token;
  }

  pub trait Match: AsRef<Self::Ref> + CoversExtent {
    type Source: Token;
    type Ref: Reference;
  }

  pub trait Parser: Readable + Writable {
    type WriteChunk: Token;
    type ReadChunk: Match;
  }

  pub trait Collector: Writable {
    type WriteChunk: Match;
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
