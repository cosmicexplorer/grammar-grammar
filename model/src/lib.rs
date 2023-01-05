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

use indexmap::IndexMap;

pub mod error {
  use displaydoc::Display;
  use thiserror::Error;

  #[derive(Debug, Display, Error)]
  pub enum Error {
    /// args provided to token failed to validate against their specification: {0}
    ArgsValidationError(String),
  }
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
  pub fn validate<I>(instance: ArgsInstance<I>) -> Result<(), error::Error> {
    todo!()
  }
}

/* TODO: make these use UUIDs or something! */
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

  pub fn register_token_spec(&mut self, token_spec: TokenSpec) {
    self
      .token_specs
      .insert(token_spec.unique_descriptor, token_spec)
      /* TODO: make this return a Result! */
      .expect("key must be unique");
  }
}

pub enum ArgsInstance<I> {
  None,
  Args(Vec<I>),
}

pub struct TokenInstance<I> {
  pub unique_descriptor: UniqueDescriptor,
  pub args: ArgsInstance<I>,
}

pub trait Stream {
  fn registry(&self) -> &Registry;
}

pub trait Parser {
  type Input: Stream;
  type Output: Stream;
}
