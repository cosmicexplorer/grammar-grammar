/*
 * Description: Type information for grammars.
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

//! Type information for grammars.
//!
//!```
//! use grammar_type_info::TypeInfo;
//! use grammar_type_info_derive::GrammarTypeInfo;
//!
//! #[derive(GrammarTypeInfo)]
//! pub struct S;
//!
//! #[derive(GrammarTypeInfo)]
//! pub struct T;
//!
//! assert!(S::TYPE.type_id.id.get_version().unwrap() == uuid::Version::Random);
//! assert!(T::TYPE.type_id.id.get_version().unwrap() == uuid::Version::Random);
//! assert!(S::TYPE.type_id.id != T::TYPE.type_id.id);
//!```

#![warn(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

use uuid::Uuid;

/// Type information.
#[derive(Clone, Debug)]
pub struct Type {
  /// A key uniquely identifying this concrete type at compile time.
  pub type_id: TypeId,
}

/// Unique identifier.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeId {
  /// A uuid generated at derive macro expansion time.
  pub id: Uuid,
}

/// Type information container.
pub trait TypeInfo {
  /// Single type information instance, constructed as a const literal via the derive macro.
  const TYPE: Type;
}
