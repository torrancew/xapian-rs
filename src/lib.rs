#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod db;

use std::num::NonZeroU32;

use bytes::Bytes;
pub use db::{Database, DbAction, DbBackend, DbFlags, WritableDatabase};

mod doc;
pub use doc::Document;

pub(crate) mod ffi;

mod iter;

mod query;
pub use query::{FieldProcessor, Operator, Query, QueryParser};

mod search;
pub use search::{
    ESet, Enquire, ExpandDecider, MSet, Match, MatchDecider, MatchSpy, NativeRangeProcessor, RSet,
    RangeProcessorFlags,
};

mod term;
pub use term::{Expansion, Stem, StemStrategy, Stopper, Term, TermGenerator};

/// A newtype wrapper representing a valid (non-zero) Xapian document ID
#[derive(Debug, Clone, Copy)]
pub struct DocId(NonZeroU32);

impl DocId {
    /// Attempt to create a `DocId` from the provided `u32`
    /// Returns `None` if the `u32` is `0`, to match Xapian's
    /// document ID semantics
    pub fn new(value: impl Into<u32>) -> Option<Self> {
        NonZeroU32::new(value.into()).map(Self)
    }

    pub(crate) unsafe fn new_unchecked(value: impl Into<u32>) -> Self {
        Self(NonZeroU32::new_unchecked(value.into()))
    }
}

impl From<DocId> for ffi::docid {
    fn from(value: DocId) -> Self {
        u32::from(value.0).into()
    }
}

impl From<DocId> for NonZeroU32 {
    fn from(value: DocId) -> Self {
        value.0
    }
}

impl From<DocId> for u32 {
    fn from(value: DocId) -> Self {
        u32::from(value.0)
    }
}

/// A newtype wrapper representing a valid document position
#[derive(Debug)]
pub struct Position(ffi::termpos);

impl From<ffi::termpos> for Position {
    fn from(value: ffi::termpos) -> Self {
        Self(value)
    }
}

impl From<u32> for Position {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl From<Position> for u32 {
    fn from(value: Position) -> u32 {
        value.0.into()
    }
}

impl From<Position> for ffi::termpos {
    fn from(value: Position) -> Self {
        value.0
    }
}

/// A trait representing the ability to be stored as a Xapian document value. Useful for features
/// such as faceting and other forms of advanced field-level filtering.
pub trait ToValue: Clone {
    /// Serialize an instance of this type into a byte buffer
    fn serialize(&self) -> Bytes;
}

impl ToValue for f64 {
    fn serialize(&self) -> Bytes {
        let value = ffi::sortable_serialise(*self);
        ffi::cxx_bytes(&value)
    }
}

macro_rules! primitive_to_value {
    ($t:ty) => {
        impl ToValue for $t {
            fn serialize(&self) -> bytes::Bytes {
                f64::serialize(&(*self as f64))
            }
        }
    };
}

primitive_to_value!(f32);
primitive_to_value!(i8);
primitive_to_value!(i16);
primitive_to_value!(i32);
primitive_to_value!(i64);
primitive_to_value!(isize);
primitive_to_value!(u8);
primitive_to_value!(u16);
primitive_to_value!(u32);
primitive_to_value!(u64);
primitive_to_value!(usize);

impl ToValue for Bytes {
    fn serialize(&self) -> Bytes {
        self.clone()
    }
}

impl ToValue for &[u8] {
    fn serialize(&self) -> Bytes {
        Bytes::copy_from_slice(self)
    }
}

impl ToValue for &str {
    fn serialize(&self) -> Bytes {
        Bytes::copy_from_slice(self.as_bytes())
    }
}

impl ToValue for String {
    fn serialize(&self) -> Bytes {
        Bytes::copy_from_slice(self.as_bytes())
    }
}

impl ToValue for &String {
    fn serialize(&self) -> Bytes {
        Bytes::copy_from_slice(self.as_bytes())
    }
}

/// A trait representing the ability to be loaded from a Xapian document value.
pub trait FromValue: Clone + PartialEq + PartialOrd + Sized {
    /// The error type returned if deserialization fails
    type Error: std::error::Error;

    /// Attempt to deserialize the provided bytes into an instance of this type
    fn deserialize(value: Bytes) -> Result<Self, Self::Error>;
}

macro_rules! primitive_from_value {
    ($t:ty) => {
        impl FromValue for $t {
            type Error = <f64 as FromValue>::Error;

            fn deserialize(value: bytes::Bytes) -> Result<Self, Self::Error> {
                Ok(f64::deserialize(value)? as Self)
            }
        }
    };
}

impl FromValue for f64 {
    type Error = std::convert::Infallible;

    fn deserialize(value: Bytes) -> Result<Self, Self::Error> {
        let value = ffi::ToCxxString::to_cxx_string(&value);
        Ok(ffi::sortable_unserialise(&value))
    }
}

primitive_from_value!(f32);
primitive_from_value!(i8);
primitive_from_value!(i16);
primitive_from_value!(i32);
primitive_from_value!(i64);
primitive_from_value!(isize);
primitive_from_value!(u8);
primitive_from_value!(u16);
primitive_from_value!(u32);
primitive_from_value!(u64);
primitive_from_value!(usize);

impl FromValue for Bytes {
    type Error = std::convert::Infallible;

    fn deserialize(value: Bytes) -> Result<Self, Self::Error> {
        Ok(value)
    }
}

impl FromValue for String {
    type Error = std::string::FromUtf8Error;

    fn deserialize(value: Bytes) -> Result<Self, Self::Error> {
        String::from_utf8(value.to_vec())
    }
}

/// A newtype wrapper representing a valid Xapian slot number (aka `valueno`)
#[derive(Debug, Clone, Copy)]
pub struct Slot(ffi::valueno);

impl From<u32> for Slot {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl From<Slot> for u32 {
    fn from(slot: Slot) -> Self {
        u32::from(slot.0)
    }
}

impl From<Slot> for ffi::valueno {
    fn from(slot: Slot) -> Self {
        slot.0
    }
}
