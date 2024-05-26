//! This module contains FFI bindings for the Xapian API, along
//! with traits to make the bindings more accessible to Rust wrappers.
//!
//! Most of the bindings are generated via `autocxx`. Where
//! necessary, C++ shims or manual bindings (via `cxx`) are written
//!
//! Additionally, this module implements a few basic traits to make comparisons
//! and C++ object cloning somewhat more accessible to Rust wrappers, a

use std::{path::Path, pin::Pin};

use autocxx::{
    cxx::{self, CxxString, UniquePtr},
    prelude::*,
};
use bytes::Bytes;

pub use self::ffi::Xapian::*;
pub use manual::Stopper;

pub mod shim {
    pub use super::ffi::shim::*;
    pub use super::manual::{query_parser_set_stopper, simple_stopper_downcast, stopper_stop_at};
}

include_cpp! {
    #include "shim.h"
    safety!(unsafe)

    block!("shim::stopper_stop_at")
    block!("shim::simple_stopper_downcast")
    block!("Xapian::DateValueRangeProcessor")
    block!("Xapian::ErrorHandler")
    block!("Xapian::NumberValueRangeProcessor")
    block!("Xapian::ValueRangeProcessor")

    extern_cpp_opaque_type!("ExpandDecider", manual::ExpandDecider)
    extern_cpp_opaque_type!("FieldProcessor", manual::FieldProcessor)
    extern_cpp_opaque_type!("KeyMaker", manual::KeyMaker)
    extern_cpp_opaque_type!("MatchDecider", manual::MatchDecider)
    extern_cpp_opaque_type!("MatchSpy", manual::MatchSpy)
    extern_cpp_opaque_type!("Stopper", manual::Stopper)

    generate!("Xapian::Database")
    generate!("Xapian::DateRangeProcessor")
    generate!("Xapian::Document")
    generate!("Xapian::Enquire")
    generate!("Xapian::ExpandDeciderAnd")
    generate!("Xapian::ExpandDeciderFilterPrefix")
    generate!("Xapian::ExpandDeciderFilterTerms")
    generate!("Xapian::MSet")
    generate!("Xapian::MSetIterator")
    generate!("Xapian::NumberRangeProcessor")
    generate!("Xapian::Query")
    generate!("Xapian::QueryParser")
    generate!("Xapian::QueryParser_feature_flag")
    generate!("Xapian::RangeProcessor")
    generate!("Xapian::SimpleStopper")
    generate!("Xapian::Stem")
    generate!("Xapian::TermGenerator")
    generate!("Xapian::QueryParser_stem_strategy")
    generate!("Xapian::TermIterator")
    generate!("Xapian::ValueCountMatchSpy")
    generate!("Xapian::WritableDatabase")

    generate_ns!("shim")
}

#[cxx::bridge(namespace = "Xapian")]
mod manual {
    unsafe extern "C++" {
        include!("shim.h");

        type QueryParser = super::ffi::Xapian::QueryParser;
        type SimpleStopper = super::ffi::Xapian::SimpleStopper;

        type ExpandDecider;
        type FieldProcessor;
        type KeyMaker;
        type MatchSpy;
        type Stopper;
    }

    #[namespace = "shim"]
    unsafe extern "C++" {
        include!("shim.h");

        /// # Safety
        /// This method uses null as a sigil and is safe to call
        unsafe fn query_parser_set_stopper(
            query_parser: Pin<&mut QueryParser>,
            stopper: *const Stopper,
        );

        fn simple_stopper_downcast<'s>(stopper: &'s SimpleStopper) -> &'s Stopper;

        fn stopper_stop_at(stopper: &Stopper, word: &CxxString) -> bool;
    }
}

/// Cast a Pinned mutable reference of type D to one of type S
///
/// # Safety
/// The caller must ensure that the cast from *mut D to *mut S is valid
pub(crate) unsafe fn upcast<S, D>(derived: Pin<&mut D>) -> Pin<&mut S> {
    let ptr = Pin::into_inner_unchecked(derived) as *mut D;
    Pin::new_unchecked(&mut *ptr.cast::<S>())
}

/// Create a new pinned `Box` containing a copy of this `MSetIterator`
impl Clone for Pin<Box<MSetIterator>> {
    fn clone(&self) -> Self {
        shim::mset_iterator_copy(self).within_box()
    }
}

/// Compare two instances of `MSetIterator`
impl PartialEq for MSetIterator {
    fn eq(&self, other: &Self) -> bool {
        shim::mset_iterator_eq(self, other)
    }
}

/// Create a new pinned `Box` containing a copy of this `PositionIterator`
impl Clone for Pin<Box<PositionIterator>> {
    fn clone(&self) -> Self {
        shim::position_iterator_copy(self).within_box()
    }
}

/// Compare two instances of `PositionIterator`
impl PartialEq for PositionIterator {
    fn eq(&self, other: &Self) -> bool {
        shim::position_iterator_eq(self, other)
    }
}

/// Create a new pinned `Box` containing a copy of this `TermIterator`
impl Clone for Pin<Box<TermIterator>> {
    fn clone(&self) -> Self {
        shim::term_iterator_copy(self).within_box()
    }
}

/// Compare two instances of `TermIterator`
impl PartialEq for TermIterator {
    fn eq(&self, other: &Self) -> bool {
        shim::term_iterator_eq(self, other)
    }
}

mod private {
    pub trait Sealed {}

    impl<T: AsRef<[u8]>> Sealed for T {}
    impl Sealed for std::path::Path {}
}

/// Create a reference-counted byte buffer from the given `CxxString`
pub fn cxx_bytes(s: &CxxString) -> Bytes {
    Bytes::from_iter(s.as_bytes().iter().copied())
}

/// Create a new, C++ heap-backed `CxxString`
pub trait ToCxxString: private::Sealed {
    fn to_cxx_string(&self) -> UniquePtr<CxxString>;
}

impl<T: AsRef<[u8]>> ToCxxString for T {
    fn to_cxx_string(&self) -> UniquePtr<CxxString> {
        let mut s = ffi::make_string("");
        s.pin_mut().push_bytes(self.as_ref());
        s
    }
}

#[cfg(target_family = "unix")]
impl ToCxxString for Path {
    fn to_cxx_string(&self) -> UniquePtr<CxxString> {
        use std::os::unix::ffi::OsStrExt;
        ToCxxString::to_cxx_string(&self.as_os_str().as_bytes())
    }
}
