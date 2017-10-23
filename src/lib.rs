//! Wrapper type for by-address hashing and comparison.
//!
//! [ByAddress] can be used to wrap any pointer type (i.e. any type that implements the Deref
//! trait).  This includes references, raw pointers, smart pointers like `Rc<T>` and `Box<T>`, and
//! specialized pointer-like type like `Vec<T>` and `String`.
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
//! fn call_each_once(callbacks: &[&Fn()]) {
//!     let mut seen: HashSet<ByAddress<&Fn()>> = HashSet::new();
//!     for &f in callbacks {
//!         if !seen.insert(ByAddress(f)) {
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
//! However, due to limitations of safe Rust, hashing takes only the data address into account.
//! **This may cause performance problems if you use slices as keys in a by-address HashMap or
//! HashSet.**  It won't cause correctness bugs, but it may cause a high rate of hash collisions.
//!
//! ```
//! # use by_address::ByAddress;
//! # use std::collections::hash_map::DefaultHasher;
//! # use std::hash::{Hash, Hasher};
//! #
//! fn hash<T: Hash>(t: T) -> u64 {
//!     let mut s = DefaultHasher::new();
//!     t.hash(&mut s);
//!     s.finish()
//! }
//!
//! let v = [1, 2, 3, 4];
//! assert_eq!(hash(ByAddress(&v[0..4])),
//!            hash(ByAddress(&v[0..2]))); // Uh-oh!
//! ```
//!
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
pub struct ByAddress<T: ?Sized + Deref>(pub T);

impl<T: ?Sized + Deref> ByAddress<T> {
    /// Convenience method for pointer casts.
    fn addr(&self) -> *const T::Target { &*self.0 }
}

impl<T: ?Sized + Deref> Deref for ByAddress<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T: ?Sized + Deref> DerefMut for ByAddress<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<T: ?Sized + Deref + AsRef<U>, U> AsRef<U> for ByAddress<T> {
    fn as_ref(&self) -> &U { self.0.as_ref() }
}

impl<T: ?Sized + Deref + AsMut<U>, U> AsMut<U> for ByAddress<T> {
    fn as_mut(&mut self) -> &mut U { self.0.as_mut() }
}

impl<T: ?Sized + Deref> Hash for ByAddress<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // FIXME: For fat pointers to dynamically-sized types, this discards the extra data (vtable
        // pointer or length), so it may have a high collision rate in certain cases.
        (self.addr() as *const ()).hash(state)
    }
}

impl<T: ?Sized + Deref> PartialEq for ByAddress<T> {
    fn eq(&self, other: &Self) -> bool {
        self.addr() == other.addr()
    }
}

impl<T: ?Sized + Deref> Eq for ByAddress<T> {}

impl<T: ?Sized + Deref> Ord for ByAddress<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr().cmp(&other.addr())
    }
}

impl<T: ?Sized + Deref> PartialOrd for ByAddress<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.addr().cmp(&other.addr()))
    }
}
