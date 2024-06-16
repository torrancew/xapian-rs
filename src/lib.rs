mod db;

use std::num::NonZeroU32;

use bytes::Bytes;
pub use db::{Database, WritableDatabase};

mod doc;
pub use doc::Document;

pub(crate) mod ffi;

mod iter;

mod query;
pub use query::{Query, QueryParser};

mod search;
pub use search::{
    DateRangeProcessor, Enquire, MSet, Match, MatchDecider, MatchSpy, NumberRangeProcessor, RSet,
    RangeProcessor, RangeProcessorFlags,
};

mod term;
pub use term::{Stem, StemStrategy, Stopper, Term, TermGenerator};

#[derive(Debug, Clone, Copy)]
pub struct DocCount(ffi::doccount);

impl From<DocCount> for u32 {
    fn from(value: DocCount) -> Self {
        value.0.into()
    }
}

impl From<DocCount> for ffi::doccount {
    fn from(value: DocCount) -> Self {
        value.0
    }
}

impl From<u32> for DocCount {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl From<ffi::doccount> for DocCount {
    fn from(value: ffi::doccount) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DocCountDiff(ffi::doccount_diff);

impl From<i32> for DocCountDiff {
    fn from(value: i32) -> Self {
        Self(value.into())
    }
}

impl From<ffi::doccount_diff> for DocCountDiff {
    fn from(value: ffi::doccount_diff) -> Self {
        Self(value)
    }
}

impl From<DocCountDiff> for i32 {
    fn from(value: DocCountDiff) -> Self {
        value.0.into()
    }
}

impl From<DocCountDiff> for ffi::doccount_diff {
    fn from(value: DocCountDiff) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DocId(NonZeroU32);

impl DocId {
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

pub trait ToValue: Clone {
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

pub trait FromValue: Clone + PartialEq + PartialOrd + Sized {
    type Error: std::error::Error;

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

#[derive(Debug, Clone, Copy)]
pub struct Slot(ffi::valueno);

impl From<u32> for Slot {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl From<Slot> for u32 {
    fn from(s: Slot) -> Self {
        s.into()
    }
}

impl From<Slot> for ffi::valueno {
    fn from(s: Slot) -> Self {
        s.0
    }
}
