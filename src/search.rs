use crate::ffi;

use std::{pin::Pin, string::FromUtf8Error};

use autocxx::{cxx, prelude::*};
use bitflags::bitflags;

pub struct DateRangeProcessor(Pin<Box<ffi::DateRangeProcessor>>);

impl DateRangeProcessor {
    pub fn new<T>(
        slot: impl Into<ffi::valueno>,
        marker_string: impl Into<Option<T>>,
        flags: impl Into<Option<RangeProcessorFlags>>,
        epoch_year: impl Into<Option<i32>>,
    ) -> Self
    where
        T: AsRef<str> + Default,
    {
        let flags = flags.into().unwrap_or_default();
        let epoch_year = epoch_year.into().unwrap_or(1970);
        cxx::let_cxx_string!(marker_string = marker_string.into().unwrap_or_default().as_ref());
        Self(
            ffi::DateRangeProcessor::new1(
                slot.into(),
                &marker_string,
                flags.bits().into(),
                epoch_year.into(),
            )
            .within_box(),
        )
    }

    pub fn check_range(&mut self, start: impl AsRef<str>, end: impl AsRef<str>) -> crate::Query {
        cxx::let_cxx_string!(start = start.as_ref());
        cxx::let_cxx_string!(end = end.as_ref());
        crate::Query::new(
            unsafe { ffi::upcast::<ffi::RangeProcessor, _>(self.0.as_mut()) }
                .check_range(&start, &end)
                .within_box(),
        )
    }
}

pub struct Enquire(Pin<Box<ffi::Enquire>>);

impl Enquire {
    pub fn new(
        db: impl AsRef<ffi::Database>,
        query: impl AsRef<ffi::Query>,
        qlen: impl Into<Option<u32>>,
    ) -> Self {
        let mut enquire = ffi::Enquire::new2(db.as_ref()).within_box();
        enquire
            .as_mut()
            .set_query(query.as_ref(), qlen.into().unwrap_or(0).into());
        Self(enquire)
    }

    pub fn mset(&self, first: u32, maxitems: u32, atleast: impl Into<Option<u32>>) -> MSet {
        MSet::new(
            ffi::shim::enquire_get_mset(
                &self.0,
                first.into(),
                maxitems.into(),
                atleast.into().unwrap_or(0).into(),
            )
            .within_box(),
        )
    }
}

impl AsRef<ffi::Enquire> for Enquire {
    fn as_ref(&self) -> &ffi::Enquire {
        &self.0
    }
}

#[derive(Clone)]
pub struct Match {
    value: ffi::docid,
    ptr: Pin<Box<ffi::MSetIterator>>,
}

impl Match {
    pub(crate) fn new(ptr: Pin<Box<ffi::MSetIterator>>) -> Self {
        let value = ffi::shim::mset_iterator_docid(&ptr);
        Self { value, ptr }
    }

    pub fn docid(&self) -> u32 {
        self.value.into()
    }

    pub fn document(&self) -> crate::Document {
        crate::Document::new(self.ptr.get_document().within_box())
    }

    pub fn percent(&self) -> i32 {
        self.ptr.get_percent().into()
    }

    pub fn rank(&self) -> u32 {
        self.ptr.get_rank().into()
    }

    pub fn weight(&self) -> f64 {
        self.ptr.get_weight()
    }
}

impl AsRef<ffi::MSetIterator> for Match {
    fn as_ref(&self) -> &ffi::MSetIterator {
        &self.ptr
    }
}

pub struct MSet(Pin<Box<ffi::MSet>>);

impl MSet {
    pub(crate) fn new(ptr: Pin<Box<ffi::MSet>>) -> Self {
        Self(ptr)
    }

    pub(crate) fn begin(&self) -> Pin<Box<ffi::MSetIterator>> {
        self.0.begin().within_box()
    }

    pub(crate) fn end(&self) -> Pin<Box<ffi::MSetIterator>> {
        self.0.end().within_box()
    }

    pub fn convert_to_percent(&self, weight: f64) -> i32 {
        self.0.convert_to_percent(weight).into()
    }

    pub fn empty(&self) -> bool {
        self.0.empty()
    }

    pub fn matches(&self) -> crate::iter::MSetIter {
        crate::iter::MSetIter::new(self)
    }

    pub fn size(&self) -> u32 {
        self.0.size().into()
    }

    pub fn snippet<T, U, V>(
        &self,
        text: impl AsRef<str>,
        length: usize,
        stemmer: impl AsRef<ffi::Stem>,
        flags: u32,
        hl: impl Into<Option<(T, U)>>,
        omit: impl Into<Option<V>>,
    ) -> Result<String, FromUtf8Error>
    where
        T: AsRef<str> + Default,
        U: AsRef<str> + Default,
        V: AsRef<str> + Default,
    {
        let (hl_start, hl_end) = hl.into().unwrap_or_default();
        cxx::let_cxx_string!(text = text.as_ref());
        cxx::let_cxx_string!(hl_start = hl_start.as_ref());
        cxx::let_cxx_string!(hl_end = hl_end.as_ref());
        cxx::let_cxx_string!(omit = omit.into().unwrap_or_default().as_ref());
        let text = self.0.snippet(
            &text,
            length,
            stemmer.as_ref(),
            flags.into(),
            &hl_start,
            &hl_end,
            &omit,
        );

        String::from_utf8(Vec::from(text.as_bytes()))
    }

    pub fn termfreq(&self, term: impl AsRef<str>) -> u32 {
        cxx::let_cxx_string!(term = term.as_ref());
        self.0.get_termfreq(&term).into()
    }
}

bitflags! {
    pub struct RangeProcessorFlags: u32 {
        const SUFFIX = 1;
        const REPEATED = 2;
        const DATE_PREFER_MDY = 4;
    }
}

impl Default for RangeProcessorFlags {
    fn default() -> Self {
        Self::empty()
    }
}

pub struct NumberRangeProcessor(Pin<Box<ffi::NumberRangeProcessor>>);

impl NumberRangeProcessor {
    pub fn new<T>(
        slot: impl Into<ffi::valueno>,
        marker_string: impl Into<Option<T>>,
        flags: impl Into<Option<RangeProcessorFlags>>,
    ) -> Self
    where
        T: AsRef<str> + Default,
    {
        let flags = flags.into().unwrap_or_default();
        cxx::let_cxx_string!(marker_string = marker_string.into().unwrap_or_default().as_ref());
        Self(
            ffi::NumberRangeProcessor::new(slot.into(), &marker_string, flags.bits().into())
                .within_box(),
        )
    }

    pub fn check_range(&mut self, start: impl AsRef<str>, end: impl AsRef<str>) -> crate::Query {
        cxx::let_cxx_string!(start = start.as_ref());
        cxx::let_cxx_string!(end = end.as_ref());
        crate::Query::new(
            unsafe { ffi::upcast::<ffi::RangeProcessor, _>(self.0.as_mut()) }
                .check_range(&start, &end)
                .within_box(),
        )
    }
}

pub struct RangeProcessor(Pin<Box<ffi::RangeProcessor>>);

impl RangeProcessor {
    pub fn new<T>(
        slot: impl Into<ffi::valueno>,
        marker_string: impl Into<Option<T>>,
        flags: impl Into<Option<RangeProcessorFlags>>,
    ) -> Self
    where
        T: AsRef<str> + Default,
    {
        let flags = flags.into().unwrap_or_default();
        cxx::let_cxx_string!(marker_string = marker_string.into().unwrap_or_default().as_ref());
        Self(
            ffi::RangeProcessor::new2(slot.into(), &marker_string, flags.bits().into())
                .within_box(),
        )
    }

    pub fn check_range(&mut self, start: impl AsRef<str>, end: impl AsRef<str>) -> crate::Query {
        cxx::let_cxx_string!(start = start.as_ref());
        cxx::let_cxx_string!(end = end.as_ref());
        crate::Query::new(self.0.as_mut().check_range(&start, &end).within_box())
    }
}
