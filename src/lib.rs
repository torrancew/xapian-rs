mod db;

pub use db::{Database, WritableDatabase};

mod doc;
pub use doc::Document;

pub mod ffi;

mod iter;

mod query;
pub use query::{Query, QueryParser};

mod search;
pub use search::{DateRangeProcessor, Enquire, MSet, Match, NumberRangeProcessor, RangeProcessor};

mod term;
pub use term::{Stem, Stopper, Term, TermGenerator};

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
