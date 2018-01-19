//! Wrapper type for by-address hashing and comparison.
//!
//! [ByAddress] can be used to wrap any pointer type (i.e. any type that implements the Deref
//! trait).  This includes references, raw pointers, smart pointers like `Rc<T>` and `Box<T>`, and
//! specialized pointer-like types such as `Vec<T>` and `String`.
//!
//! Comparison, ordering, and hashing of the wrapped pointer will be based on the address of its
//! contents, rather than their value.
//!
//! ```
//! use by_address::ByAddress;
//! use std::rc::Rc;
//!
//! let rc = Rc::new(5);
//! let x = ByAddress(rc.clone());
//! let y = ByAddress(rc.clone());
//!
//! // x and y are two pointers to the same address:
//! assert_eq!(x, y);
//!
//! let z = ByAddress(Rc::new(5));
//!
//! // *x and *z have the same value, but not the same address:
//! assert_ne!(x, z);
//! ```
//!
//! You can use wrapped pointers as keys in hashed or ordered collections, like BTreeMap/BTreeSet
//! or HashMap/HashSet, even if the target of the pointer doesn't implement hashing or ordering.
//! This even includes pointers to trait objects, which usually don't implement the Eq trait
//! because it is not object-safe.
//!
//! ```
//! # use by_address::ByAddress;
//! # use std::collections::HashSet;
//! #
//! /// Call each item in `callbacks`, skipping any duplicate references.
//! fn call_each_once(callbacks: &[&Fn()]) {
//!     let mut seen: HashSet<ByAddress<&Fn()>> = HashSet::new();
//!     for &f in callbacks {
//!         if seen.insert(ByAddress(f)) {
//!             f();
//!         }
//!     }
//! }
//! ```
//!
//! If `T` is a pointer to an unsized type, then comparison and ordering of `ByAddress<T>` compare
//! the entire fat pointer, not just the "thin" data address.  This means that two slice pointers
//! are consider equal only if they have the same starting address *and* length.
//!
//! ```
//! # use by_address::ByAddress;
//! #
//! let v = [1, 2, 3, 4];
//!
//! assert_eq!(ByAddress(&v[0..4]), ByAddress(&v[0..4])); // Same address and length.
//! assert_ne!(ByAddress(&v[0..4]), ByAddress(&v[0..2])); // Same address, different length.
//! ```
//!
//! This crate does not depend on libstd, so it can be used in [`no_std`] projects.
//!
//! [`no_std`]: https://doc.rust-lang.org/book/first-edition/using-rust-without-the-standard-library.html
//! [ByAddress]: struct.ByAddress.html

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![no_std]

use core::cmp::Ordering;
use core::convert::AsRef;
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};

/// Wrapper for pointer types that implements by-address comparison.
///
/// See the [crate-level documentation](index.html) for details.
#[derive(Copy, Clone, Debug, Default)]
pub struct ByAddress<T>(pub T) where T: ?Sized + Deref;

impl<T> ByAddress<T> where T: ?Sized + Deref {
    /// Convenience method for pointer casts.
    fn addr(&self) -> *const T::Target { &*self.0 }
}

/// Raw pointer equality
impl<T> PartialEq for ByAddress<T> where T: ?Sized + Deref {
    fn eq(&self, other: &Self) -> bool {
        self.addr() == other.addr()
    }
}
impl<T> Eq for ByAddress<T> where T: ?Sized + Deref {}

/// Raw pointer ordering
impl<T> Ord for ByAddress<T> where T: ?Sized + Deref {
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr().cmp(&other.addr())
    }
}

/// Raw pointer comparison
impl<T> PartialOrd for ByAddress<T> where T: ?Sized + Deref {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.addr().cmp(&other.addr()))
    }
}

/// Raw pointer hashing
impl<T> Hash for ByAddress<T> where T: ?Sized + Deref {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr().hash(state)
    }
}

// Generic conversion traits:

impl<T> Deref for ByAddress<T> where T: ?Sized + Deref {
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> DerefMut for ByAddress<T> where T: ?Sized + Deref {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<T, U> AsRef<U> for ByAddress<T> where T: ?Sized + Deref + AsRef<U> {
    fn as_ref(&self) -> &U { self.0.as_ref() }
}

impl<T, U> AsMut<U> for ByAddress<T> where T: ?Sized + Deref + AsMut<U> {
    fn as_mut(&mut self) -> &mut U { self.0.as_mut() }
}

impl<T> From<T> for ByAddress<T> where T: Deref {
    fn from(t: T) -> ByAddress<T> { ByAddress(t) }
}
