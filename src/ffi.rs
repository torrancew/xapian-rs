//! This module contains FFI bindings for the Xapian API, along
//! with traits to make the bindings more accessible to Rust wrappers.
//!
//! Most of the bindings are generated via `autocxx`. Where
//! necessary, C++ shims or manual bindings (via `cxx`) are written
//!
//! Additionally, this module implements a few basic traits to make comparisons
//! and C++ object cloning somewhat more accessible to Rust wrappers, a

// These are disabled to silence warnings about generated code we don't directly control
#![allow(unused_imports)]
#![allow(clippy::boxed_local)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::new_ret_no_self)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::upper_case_acronyms)]

use std::{any::TypeId, cell::RefCell, fmt::Debug, path::Path, pin::Pin, rc::Rc};

use autocxx::{
    cxx::{CxxString, UniquePtr},
    prelude::*,
    subclass::prelude::*,
};
use bytes::Bytes;

pub(crate) use self::ffi::Xapian::*;

pub mod shim {
    pub use super::ffi::shim::*;
}

include_cpp! {
    #include "shim.h"
    safety!(unsafe)

    block!("Xapian::DateValueRangeProcessor")
    block!("Xapian::ErrorHandler")
    block!("Xapian::ExpandDecider")
    block!("Xapian::FieldProcessor")
    block!("Xapian::KeyMaker")
    block!("Xapian::MatchDecider")
    block!("Xapian::MatchSpy")
    block!("Xapian::NumberValueRangeProcessor")
    block!("Xapian::Stopper")
    block!("Xapian::ValueRangeProcessor")

    subclass!("shim::FfiExpandDecider", RustExpandDecider)
    subclass!("shim::FfiFieldProcessor", RustFieldProcessor)
    subclass!("shim::FfiMatchDecider", RustMatchDecider)
    subclass!("shim::FfiMatchSpy", RustMatchSpy)
    subclass!("shim::FfiStopper", RustStopper)

    generate!("Xapian::doccount")
    generate!("Xapian::doccount_diff")
    generate!("Xapian::sortable_serialise")
    generate!("Xapian::sortable_unserialise")
    generate!("Xapian::Database")
    generate!("Xapian::DateRangeProcessor")
    generate!("Xapian::Document")
    generate!("Xapian::Enquire")
    generate!("Xapian::ExpandDeciderAnd")
    generate!("Xapian::ExpandDeciderFilterPrefix")
    generate!("Xapian::ExpandDeciderFilterTerms")
    generate!("Xapian::ESet")
    generate!("Xapian::ESetIterator")
    generate!("Xapian::MSet")
    generate!("Xapian::MSetIterator")
    generate!("Xapian::NumberRangeProcessor")
    generate!("Xapian::Query")
    generate!("Xapian::QueryParser")
    generate!("Xapian::QueryParser_feature_flag")
    generate!("Xapian::RSet")
    generate!("Xapian::RangeProcessor")
    generate!("Xapian::SimpleStopper")
    generate!("Xapian::Stem")
    generate!("Xapian::TermGenerator")
    generate!("Xapian::TermGenerator_stem_strategy")
    generate!("Xapian::TermGenerator_stop_strategy")
    generate!("Xapian::QueryParser_stem_strategy")
    generate!("Xapian::TermIterator")
    generate!("Xapian::ValueCountMatchSpy")
    generate!("Xapian::WritableDatabase")

    generate!("Xapian::InMemory::open")

    generate_ns!("shim")
}

#[subclass]
pub struct RustExpandDecider {
    inner: Pin<Box<dyn crate::ExpandDecider + 'static>>,
}

impl RustExpandDecider {
    pub fn from_trait(decider: impl crate::ExpandDecider + 'static) -> Rc<RefCell<Self>> {
        let me = Self {
            inner: Box::pin(decider),
            cpp_peer: Default::default(),
        };
        Self::new_rust_owned(me)
    }
}

impl shim::FfiExpandDecider_methods for RustExpandDecider {
    fn should_keep(&self, term: &CxxString) -> bool {
        self.inner.should_keep(&term.to_string())
    }
}

#[subclass]
pub struct RustFieldProcessor {
    inner: Pin<Box<dyn crate::FieldProcessor + 'static>>,
}

impl RustFieldProcessor {
    pub fn from_trait(proc: impl crate::FieldProcessor + 'static) -> Rc<RefCell<Self>> {
        let me = Self {
            inner: Box::pin(proc),
            cpp_peer: Default::default(),
        };
        Self::new_rust_owned(me)
    }
}

impl shim::FfiFieldProcessor_methods for RustFieldProcessor {
    fn process(&self, field: &CxxString) -> UniquePtr<Query> {
        let field = field.to_string();
        if let Some(query) = self.inner.process(&field) {
            shim::query_clone(query.as_ref()).within_unique_ptr()
        } else {
            Query::new13(Query_op::OP_INVALID).within_unique_ptr()
        }
    }
}

#[subclass]
pub struct RustMatchDecider {
    inner: Pin<Box<dyn crate::MatchDecider + 'static>>,
}

impl RustMatchDecider {
    pub fn from_trait(decider: impl crate::MatchDecider + 'static) -> Rc<RefCell<Self>> {
        let me = Self {
            inner: Box::pin(decider),
            cpp_peer: Default::default(),
        };
        Self::new_rust_owned(me)
    }
}

impl shim::FfiMatchDecider_methods for RustMatchDecider {
    fn is_match(&self, doc: &Document) -> bool {
        let doc = crate::Document::new(shim::document_copy(doc).within_box());
        self.inner.is_match(&doc)
    }
}

#[subclass]
pub struct RustMatchSpy {
    inner: Box<dyn crate::MatchSpy + 'static>,
}

impl RustMatchSpy {
    pub fn from_trait(stopper: impl crate::MatchSpy + 'static) -> Rc<RefCell<Self>> {
        let me = Self {
            inner: Box::new(stopper),
            cpp_peer: Default::default(),
        };
        Self::new_rust_owned(me)
    }
}

impl shim::FfiMatchSpy_methods for RustMatchSpy {
    fn name(&self) -> UniquePtr<CxxString> {
        self.inner
            .name()
            .unwrap_or(format!("{:?}", TypeId::of::<Self>()))
            .to_cxx_string()
    }

    fn observe(&mut self, doc: &Document, weight: f64) {
        let doc = crate::Document::new(shim::document_copy(doc).within_box());
        self.inner.observe(&doc, weight)
    }
}

#[subclass]
pub struct RustStopper {
    inner: Pin<Box<dyn crate::Stopper + 'static>>,
}

impl RustStopper {
    pub fn from_trait(stopper: impl crate::Stopper + 'static) -> Rc<RefCell<Self>> {
        let me = Self {
            inner: Box::pin(stopper),
            cpp_peer: Default::default(),
        };
        Self::new_rust_owned(me)
    }
}

impl shim::FfiStopper_methods for RustStopper {
    fn is_stopword(&self, word: &CxxString) -> bool {
        self.inner.is_stopword(&word.to_string())
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

/// Create a new pinned `Box` containing a copy of this `ESetIterator`
impl Clone for Pin<Box<ESetIterator>> {
    fn clone(&self) -> Self {
        shim::eset_iterator_copy(self).within_box()
    }
}

/// Compare two instances of `ESetIterator`
impl PartialEq for ESetIterator {
    fn eq(&self, other: &Self) -> bool {
        shim::eset_iterator_eq(self, other)
    }
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

/// Create a new pinned `Box` containing a copy of this `Query`
impl Clone for Pin<Box<Query>> {
    fn clone(&self) -> Self {
        shim::query_clone(self).within_box()
    }
}

impl Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Query{}{}{}",
            "{",
            self.get_description(),
            "}"
        ))
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
