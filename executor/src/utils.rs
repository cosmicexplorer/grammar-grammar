/*
 * Description: A version of [`PhantomData`] that (`unsafe`ly) impls [`Send`] and [`Sync`].
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

//! A version of [`PhantomData`] that (`unsafe`ly) impls [`Send`] and [`Sync`].

use core::marker::PhantomData;

/// A version of [`PhantomData`] that (`unsafe`ly) impls [`Send`] and [`Sync`].
///
/// The following fails to compile as of 1.61.0, complaining that the generic parameter `I` is
/// unused:
///```compile_fail
/// pub struct S<I, O, F>
/// where
///   F: Fn(I) -> O,
/// {
///   f: F,
/// }
///```
///
/// We can work around the failure for `I` using this struct. When we do so, the output parameter
/// `O` is correctly tracked without any extra effort:
///```
/// use grammar_executor::utils::PhantomSyncHack;
///
/// pub struct S<I, O, F>
/// where
///   F: Fn(I) -> O,
/// {
///   _ph: PhantomSyncHack<I>,
///   f: F,
/// }
///
/// impl<I, O, F> S<I, O, F>
/// where
///   F: Fn(I) -> O,
/// {
///   pub fn new(f: F) -> Self {
///     Self { _ph: PhantomSyncHack::default(), f }
///   }
///
///   pub fn g(&self, x: I) -> O {
///     (self.f)(x)
///   }
/// }
///
/// let s = S::new(|x| x + 1);
/// assert!(s.g(3) == 4);
///```
pub struct PhantomSyncHack<S>(PhantomData<S>);

impl<S> Default for PhantomSyncHack<S> {
  fn default() -> Self {
    Self(PhantomData)
  }
}

unsafe impl<S> Send for PhantomSyncHack<S> {}
unsafe impl<S> Sync for PhantomSyncHack<S> {}
