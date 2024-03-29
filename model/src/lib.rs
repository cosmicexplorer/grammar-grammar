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

  /* pub struct TokenInstance<I> { */
  /*   pub unique_descriptor: UniqueDescriptor, */
  /*   pub args: ArgsInstance, */
  /* } */
}

pub mod parameters {
  use displaydoc::Display;
  use thiserror::Error;

  use std::any::Any;

  #[derive(Clone, Debug, Display, Error)]
  pub enum ParameterResolutionError {
    /// args provided to token failed to validate against their specification: {0}
    ArgsValidation(String),
    /// no args should have been provided to token, and yet these were: {0}
    ArgsWereUnexpected(String),
    /// arg requested at an out of bounds index: {0}
    ArgOutOfBounds(String),
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
    pub fn validate(instance: ArgsInstance) -> Result<(), ParameterResolutionError> {
      Err(ParameterResolutionError::ArgsValidation(format!(
        "TODO: implement this method"
      )))
    }
  }

  pub enum ArgsInstance {
    None,
    Args(Vec<Box<dyn Any>>),
  }

  impl ArgsInstance {
    /// Return a slice to the boxed arguments, if any.
    ///
    ///```
    /// use grammar_model::parameters::*;
    ///
    /// struct S;
    ///
    /// match ArgsInstance::None.get_args() {
    ///   Err(ParameterResolutionError::ArgsWereUnexpected(_)) => (),
    ///   _ => unreachable!(),
    /// }
    /// match ArgsInstance::Args(vec![Box::new(S)]).get_args() {
    ///   Ok(args) if args.len() == 1 => (),
    ///   _ => unreachable!(),
    /// }
    ///```
    pub fn get_args(&self) -> Result<&[Box<dyn Any>], ParameterResolutionError> {
      match self {
        Self::None => Err(ParameterResolutionError::ArgsWereUnexpected(
          "TODO".to_string(),
        )),
        Self::Args(args) => Ok(args.as_ref()),
      }
    }

    /// Return a reference to a particular boxed argument.
    ///
    ///```
    /// use grammar_model::parameters::*;
    ///
    /// struct S;
    ///
    /// let args = ArgsInstance::Args(vec![Box::new(S)]);
    /// assert!(args.get_arg_at(0).is_ok());
    /// match args.get_arg_at(1) {
    ///   Err(ParameterResolutionError::ArgOutOfBounds(_)) => (),
    ///   _ => unreachable!(),
    /// }
    ///```
    pub fn get_arg_at(&self, index: usize) -> Result<&Box<dyn Any>, ParameterResolutionError> {
      match self.get_args()?.get(index) {
        Some(arg) => Ok(arg),
        None => Err(ParameterResolutionError::ArgOutOfBounds("TODO".to_string())),
      }
    }
  }

  /// A callback to execute once a sub-parse/production has succeeded.
  ///
  ///```
  /// # fn main() -> Result<(), grammar_model::parameters::ParameterResolutionError> {
  /// use grammar_model::parameters::*;
  ///
  /// struct S(pub usize);
  /// struct Add;
  /// impl Resolver for Add {
  ///   type Product = S;
  ///   type Error = ParameterResolutionError;
  ///   fn resolve(&self, args: ArgsInstance) -> Result<S, Self::Error> {
  ///     let (a, b) = match args.get_args()? {
  ///       [a, b] => (a, b),
  ///       _ => unreachable!(),
  ///     };
  ///     let S(a) = *a.as_ref().downcast_ref::<S>().unwrap();
  ///     let S(b) = *b.as_ref().downcast_ref::<S>().unwrap();
  ///     Ok(S(a + b))
  ///   }
  /// }
  ///
  /// let args = ArgsInstance::Args(vec![Box::new(S(1)), Box::new(S(2))]);
  /// let S(c) = Add.resolve(args)?;
  /// assert!(c == 3);
  /// # Ok(())
  /// # }
  ///```
  pub trait Resolver {
    /// e.g. an AST node
    type Product;
    /// this will be used to generate a parse error
    type Error;

    /// Execute this node's logic to transform its `args` into a single product.
    fn resolve(&self, args: ArgsInstance) -> Result<Self::Product, Self::Error>;
  }
}

/* TODO: example of a parser as a translator by using std::str functions to parse utf8 bytes! */

///```
/// use grammar_model::control_flow::*;
///
/// struct Utf8Parser;
/// impl Parser for Utf8Parser {
///   type Input = u8;
///   type Output = char;
/// }
///
/// ???
///```
pub mod control_flow {
  pub trait Parser {
    type Input;
    type Output;
  }
}
