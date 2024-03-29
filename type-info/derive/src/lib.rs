/*
 * Description: A derive macro for runtime type information.
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

//! A derive macro for runtime type information.

#![warn(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use uuid::Uuid;

/// Implements `TypeInfo` for a specific concrete type.
#[proc_macro_derive(GrammarTypeInfo)]
pub fn derive(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, .. } = parse_macro_input!(input);
  let id = Uuid::new_v4().as_u128();
  let output = quote! {
    impl ::grammar_type_info::StaticTypeInfo for #ident {
      const TYPE: ::grammar_type_info::Type =
        ::grammar_type_info::Type {
          type_id: ::grammar_type_info::TypeId {
             id: ::uuid::Uuid::from_u128(#id),
          },
        };
    }

    impl ::grammar_type_info::DynamicTypeInfo for #ident {
      fn get_type(&self) -> ::grammar_type_info::Type {
        ::grammar_type_info::Type {
          type_id: ::grammar_type_info::TypeId {
             id: ::uuid::Uuid::from_u128(#id),
          },
        }
      }
    }
  };
  output.into()
}
