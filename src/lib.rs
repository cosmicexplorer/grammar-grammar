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

  use grammar_type_info::StaticTypeInfo;

  pub trait GrammarCase {
    type Tok: direct::Token;
    type Ref: indirect::Reference;
    type Elements: Iterator<Item = synthesis::CaseElement<Self::Tok, Self::Ref>>;
    fn elements(&self) -> Self::Elements;
  }

  pub trait Collector {
    type Args;
    type Result: StaticTypeInfo;
    fn collect(&self, args: Self::Args) -> Self::Result;
  }
}

pub mod components {
  pub mod direct {
    use crate::pipeline::{InputElement, Sourced};

    pub trait Token: Sourced {
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

  pub trait Sourced {
    type Source;
  }

  pub trait CollectiveSource<S> {}

  pub trait SourceOrigin: Sourced {
    type CS: CollectiveSource<Self::Source>;
    fn original_source(&self) -> Self::CS;
  }

  #[derive(Debug, Clone)]
  pub struct Extent(pub Range<usize>);

  pub trait CoversExtent: Sourced + SourceOrigin {
    fn covers_extent(&self) -> Extent;
    /* fn source_range(&self) -> &'a [Self::Source] { */
    /*   &self.original_source()[self.covers_extent().0] */
    /* } */
  }

  pub trait InputElement {}

  pub trait Input: Readable {
    type ReadChunk: InputElement;
  }

  pub trait SourcedToken: AsRef<Self::Tok> + CoversExtent {
    type Tok: Token;
  }

  pub trait Tokenizer: Readable + Writable {
    type WriteChunk: InputElement;
    type ReadChunk: SourcedToken;
  }

  pub trait Match: AsRef<Self::Ref> + CoversExtent {
    type Source: SourcedToken;
    type Ref: Reference;
  }

  pub trait Parser: Readable + Writable {
    type WriteChunk: SourcedToken;
    type ReadChunk: Match;
  }

  pub trait Collector: Writable {
    type WriteChunk: Match;
  }
}

#[cfg(test)]
mod test_framework {
  use super::{
    components::{direct, indirect, synthesis},
    pipeline,
  };

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
  pub struct InputElement(pub u8);
  impl pipeline::InputElement for InputElement {}

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
  pub struct Token(pub usize);
  impl pipeline::Sourced for Token {
    type Source = InputElement;
  }
  impl direct::Token for Token {
    type Source = InputElement;
  }

  #[derive(Debug, Clone)]
  pub struct InputSource;
  impl pipeline::CollectiveSource<InputElement> for InputSource {}

  #[derive(Debug, Clone)]
  pub struct SourcedToken {
    pub source: InputSource,
    pub token: Token,
    pub extent: pipeline::Extent,
  }

  impl pipeline::Sourced for SourcedToken {
    type Source = InputElement;
  }
  impl pipeline::SourceOrigin for SourcedToken {
    type CS = InputSource;
    fn original_source(&self) -> Self::CS {
      self.source.clone()
    }
  }
  impl pipeline::CoversExtent for SourcedToken {
    fn covers_extent(&self) -> pipeline::Extent {
      self.extent.clone()
    }
  }
  impl AsRef<Token> for SourcedToken {
    fn as_ref(&self) -> &Token {
      &self.token
    }
  }
  impl pipeline::SourcedToken for SourcedToken {
    type Tok = Token;
  }
}

#[cfg(test)]
mod tests {
  use super::test_framework::*;
  use crate::pipeline;

  #[test]
  fn it_works() {
    let el = InputElement(0);
    let tok = Token(0);
    let st = SourcedToken {
      source: InputSource,
      token: tok,
      extent: pipeline::Extent(0..1),
    };
  }
}
