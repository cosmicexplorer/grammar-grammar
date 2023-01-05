/*
 * Description: Protocols for parser outputs.
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

//! Protocols for parser outputs.

/* #![warn(missing_docs)] */
/* #![deny(rustdoc::missing_crate_level_docs)] */
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

#[derive(Clone, Debug, displaydoc::Display, thiserror::Error)]
pub enum InternalError {
  /// token error: {0}
  Token(#[from] tokens::TokenError),
  /// parameter resolution error: {0}
  Parameter(#[from] parameters::ParameterResolutionError),
}

pub mod tokens {
  use super::parameters::{ArgsInstance, ArgsSpec};

  use displaydoc::Display;
  use indexmap::IndexMap;
  use thiserror::Error;

  #[derive(Clone, Debug, Display, Error)]
  pub enum TokenError {
    /// token @ {0} registered twice
    TokenRegistration(UniqueDescriptor),
  }

  /// unique descriptor: '{0}'
  ///
  /// A string used to uniquely identify a token class, to avoid duplicate registrations. This
  /// string is also used in error messages, so it should briefly describe the token itself as well.
  #[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
  #[ignore_extra_doc_attributes]
  pub struct UniqueDescriptor(pub &'static str);

  /// a class of tokens which may be emitted by some stream
  #[derive(Clone, Debug)]
  pub struct TokenSpec {
    pub unique_descriptor: UniqueDescriptor,
    pub args_spec: ArgsSpec,
  }

  #[derive(Clone, Debug)]
  pub struct Registry {
    token_specs: IndexMap<UniqueDescriptor, TokenSpec>,
  }

  impl Registry {
    pub fn new() -> Self {
      Self {
        token_specs: IndexMap::new(),
      }
    }

    pub fn register_token_spec(&mut self, token_spec: TokenSpec) -> Result<(), TokenError> {
      let id = token_spec.unique_descriptor;
      if self.token_specs.insert(id, token_spec).is_some() {
        Err(TokenError::TokenRegistration(id))
      } else {
        Ok(())
      }
    }
  }

  pub struct TokenInstance<I> {
    pub unique_descriptor: UniqueDescriptor,
    pub args: ArgsInstance<I>,
  }
}

pub mod parameters {
  use displaydoc::Display;
  use thiserror::Error;

  #[derive(Clone, Debug, Display, Error)]
  pub enum ParameterResolutionError {
    /// args provided to token failed to validate against their specification: {0}
    ArgsValidation(String),
  }

  /* TODO: displaydoc! */
  /// "argument specification" for a token
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum ArgsSpec {
    /// no arguments means each instance of this token is indistinguishable from the others
    NoArguments,
    /// may take some arguments sometimes
    SomeArguments,
  }

  impl ArgsSpec {
    pub fn validate<I>(instance: ArgsInstance<I>) -> Result<(), ParameterResolutionError> {
      Err(ParameterResolutionError::ArgsValidation(format!(
        "TODO: implement this method"
      )))
    }
  }

  pub enum ArgsInstance<I> {
    None,
    Args(Vec<I>),
  }
}

pub mod control_flow {
  use super::tokens::Registry;

  pub trait Stream {
    fn registry(&self) -> &Registry;
  }

  pub trait Parser {
    type Input: Stream;
    type Output: Stream;
  }
}
