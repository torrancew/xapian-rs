use crate::{ffi, DocId};

use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    ops::Deref,
    pin::Pin,
    rc::Rc,
};

use autocxx::{cxx, prelude::*};
use bitflags::bitflags;

/// The primary interface to retrieve information from Xapian.
///
/// Used to perform searches, faceting, term iteration, expansion, sorting, relevancy and more.
pub struct Enquire(Pin<Box<ffi::Enquire>>);

impl Enquire {
    /// Create a new `Enquire` instance associated with the given `db`
    pub fn new(db: impl AsRef<ffi::Database>) -> Self {
        Self(ffi::Enquire::new2(db.as_ref()).within_box())
    }

    /// Attach a [`MatchSpy`] implementation to this `Enquire`
    ///
    /// Instances of `MatchSpy` can be used to implement faceting
    pub fn add_matchspy<T: crate::MatchSpy + Clone + 'static>(&mut self, spy: &T) {
        let spy = spy.clone().into_ffi();
        unsafe { ffi::shim::enquire_add_matchspy(self.0.as_mut(), spy.upcast()) }
    }

    /// Retrieve the term expansion set for this Enquire
    ///
    /// An ESet provides terms which may be relevant to the current query
    pub fn eset(
        &self,
        maxitems: u32,
        rset: impl AsRef<ffi::RSet>,
        flags: i32,
        decider: impl Into<Option<&'static ExpandDeciderWrapper>>,
        min_wt: f64,
    ) -> ESet {
        let decider = decider
            .into()
            .map_or(std::ptr::null(), |d| Deref::deref(&d.upcast()) as *const _);

        ESet(
            unsafe {
                ffi::shim::enquire_get_eset(
                    &self.0,
                    maxitems.into(),
                    rset.as_ref(),
                    flags.into(),
                    decider,
                    min_wt,
                )
            }
            .within_box(),
        )
    }

    /// Retrieve the [`MSet`] for the current [`Query`][crate::Query]
    pub fn mset(
        &self,
        first: u32,
        maxitems: u32,
        atleast: impl Into<Option<u32>>,
        rset: impl Into<Option<RSet>>,
        decider: impl Into<Option<&'static MatchDeciderWrapper>>,
    ) -> MSet {
        let rset = rset
            .into()
            .map_or(std::ptr::null(), |r| r.as_ref() as *const _);
        let decider = decider
            .into()
            .map_or(std::ptr::null(), |d| Deref::deref(&d.upcast()) as *const _);
        MSet::new(
            unsafe {
                ffi::shim::enquire_get_mset(
                    &self.0,
                    first.into(),
                    maxitems.into(),
                    atleast.into().unwrap_or(0).into(),
                    rset,
                    decider,
                )
            }
            .within_box(),
        )
    }

    /// Retrieve the query currently associated with this Enquire instance
    pub fn query(&self) -> crate::Query {
        crate::Query::from_ffi(ffi::shim::query_clone(self.0.get_query()).within_box())
    }

    /// Set the query currently associated with this Enquire instance
    pub fn set_query(&mut self, query: impl AsRef<ffi::Query>, qlen: impl Into<Option<u32>>) {
        self.0
            .as_mut()
            .set_query(query.as_ref(), qlen.into().unwrap_or(0).into());
    }
}

impl AsRef<ffi::Enquire> for Enquire {
    fn as_ref(&self) -> &ffi::Enquire {
        &self.0
    }
}

/// An [`ExpandDecider`] can be used to reject terms from an [`ESet`]
pub trait ExpandDecider {
    /// Decide whether this term should be included in the `ESet`
    fn should_keep(&self, term: &str) -> bool;

    #[doc(hidden)]
    fn into_ffi(self) -> &'static ExpandDeciderWrapper
    where
        Self: Sized + 'static,
    {
        Box::leak(Box::new(ExpandDeciderWrapper::from(self)))
    }
}

#[doc(hidden)]
pub struct ExpandDeciderWrapper(Rc<RefCell<ffi::RustExpandDecider>>);

impl ExpandDeciderWrapper {
    pub fn upcast(&self) -> impl Deref<Target = ffi::shim::FfiExpandDecider> + '_ {
        Ref::map(self.0.borrow(), |s| s.as_ref())
    }
}

impl<T: ExpandDecider + 'static> From<T> for ExpandDeciderWrapper {
    fn from(value: T) -> Self {
        Self(ffi::RustExpandDecider::from_trait(value))
    }
}

/// An [`ESet`] represents a set of terms that may be useful for expanding the current query
pub struct ESet(Pin<Box<ffi::ESet>>);

impl ESet {
    pub(crate) fn begin(&self) -> Pin<Box<ffi::ESetIterator>> {
        self.0.begin().within_box()
    }

    /// Returns true if there are no terms in this `ESet`
    pub fn empty(&self) -> bool {
        self.0.empty()
    }

    pub(crate) fn end(&self) -> Pin<Box<ffi::ESetIterator>> {
        self.0.end().within_box()
    }

    /// Returns the size of this `ESet`
    pub fn size(&self) -> u32 {
        u32::from(self.0.size())
    }

    /// Retrieve the iterator of [`Match`] objects for this `MSet`
    pub fn terms(&self) -> crate::iter::ESetIter {
        crate::iter::ESetIter::new(self)
    }
}

/// An individual match item from the iterator yielded by [`MSet::matches`]
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

    /// Retrieve the [`DocId`][crate::DocId] associated with this Match
    pub fn docid(&self) -> crate::DocId {
        unsafe { crate::DocId::new_unchecked(self.value) }
    }

    /// Retrieve the [`Document`][crate::Document] associated with this Match
    pub fn document(&self) -> crate::Document {
        crate::Document::new(self.ptr.get_document().within_box())
    }

    /// Retrieve the weight of this Match, represented as a percentage
    pub fn percent(&self) -> i32 {
        self.ptr.get_percent().into()
    }

    /// Retrieve the [`MSet`] rank of this Match
    pub fn rank(&self) -> u32 {
        self.ptr.get_rank().into()
    }

    /// Retrieve the weight of this Match
    pub fn weight(&self) -> f64 {
        self.ptr.get_weight()
    }
}

impl AsRef<ffi::MSetIterator> for Match {
    fn as_ref(&self) -> &ffi::MSetIterator {
        &self.ptr
    }
}

impl Debug for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Match")
            .field(&self.ptr.get_description())
            .finish()
    }
}

impl PartialEq for Match {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

/// A [`MatchDecider`] can be used to reject documents from an [`MSet`]
pub trait MatchDecider {
    /// Decide whether this document should be included in the `MSet`
    fn is_match(&self, doc: &crate::Document) -> bool;

    #[doc(hidden)]
    fn into_ffi(self) -> &'static MatchDeciderWrapper
    where
        Self: Sized + 'static,
    {
        Box::leak(Box::new(MatchDeciderWrapper::from(self)))
    }
}

#[doc(hidden)]
pub struct MatchDeciderWrapper(Rc<RefCell<ffi::RustMatchDecider>>);

impl MatchDeciderWrapper {
    pub fn upcast(&self) -> impl Deref<Target = ffi::shim::FfiMatchDecider> + '_ {
        Ref::map(self.0.borrow(), |s| s.as_ref())
    }
}

impl<T: MatchDecider + 'static> From<T> for MatchDeciderWrapper {
    fn from(value: T) -> Self {
        Self(ffi::RustMatchDecider::from_trait(value))
    }
}

/// A [`MatchSpy`] can be used to accumulate information seen during the match.
///
/// Useful for faceting and generally profiling matching documents
pub trait MatchSpy {
    /// Process this [`Document`][crate::Document]
    ///
    /// Used to collect any desired data/metadata from the document
    fn observe(&self, doc: &crate::Document, weight: f64);

    #[doc(hidden)]
    fn into_ffi(self) -> &'static mut MatchSpyWrapper
    where
        Self: Sized + 'static,
    {
        Box::leak(Box::new(MatchSpyWrapper::from(self)))
    }

    /// An optional, human-friendly name for the MatchSpy
    fn name(&self) -> Option<String> {
        None
    }
}

#[doc(hidden)]
pub struct MatchSpyWrapper(Rc<RefCell<ffi::RustMatchSpy>>);

impl MatchSpyWrapper {
    #[doc(hidden)]
    pub fn upcast(&mut self) -> *mut ffi::shim::FfiMatchSpy {
        use ffi::shim::FfiMatchSpy_methods;
        self.0.borrow_mut().upcast()
    }
}

impl<T: MatchSpy + 'static> From<T> for MatchSpyWrapper {
    fn from(value: T) -> Self {
        Self(ffi::RustMatchSpy::from_trait(value))
    }
}

/// A list of search results with associated metadata
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

    /// Convert a weight to a percentage, taking into account weighted query terms
    pub fn convert_to_percent(&self, weight: f64) -> i32 {
        self.0.convert_to_percent(weight).into()
    }

    /// Detects whether this `MSet` is empty
    pub fn empty(&self) -> bool {
        self.0.empty()
    }

    /// Retrieve the iterator of [`Match`] objects for this `MSet`
    pub fn matches(&self) -> crate::iter::MSetIter {
        crate::iter::MSetIter::new(self)
    }

    /// The number of matches in this `MSet`
    pub fn size(&self) -> u32 {
        self.0.size().into()
    }

    /// Generate a snippet from the provided `text`
    ///
    /// `length` controls the size of the snippet
    /// `stemmer` should be an instance of the same stemming algorithm used to build the query
    /// `flags` are used to control specific bits of functionality
    /// `hl` is an optional pair of string-likes used to highlight matches within the snippet, for use in markup
    /// `omit` is used to indicate any truncated prefix or suffix
    /// mid-sen
    pub fn snippet<T, U, V>(
        &self,
        text: impl AsRef<str>,
        length: usize,
        stemmer: impl AsRef<ffi::Stem>,
        flags: u32,
        hl: impl Into<Option<(T, U)>>,
        omit: impl Into<Option<V>>,
    ) -> String
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

        text.to_string()
    }

    /// Get the number of documents which `term` occurs in
    pub fn termfreq(&self, term: impl AsRef<str>) -> u32 {
        cxx::let_cxx_string!(term = term.as_ref());
        self.0.get_termfreq(&term).into()
    }
}

bitflags! {
    /// A bitflag representation of flags supported by a RangeProcessor
    pub struct RangeProcessorFlags: u32 {
        /// Treat the given marker string as a suffix instead of a prefix
        const SUFFIX = 1;
        /// Optionally allow the given marker string on both ends of the range
        const REPEATED = 2;
        /// Interpret ambiguous dates as Month/Date/Year instead of Date/Month/Year
        const DATE_PREFER_MDY = 4;
    }
}

impl Default for RangeProcessorFlags {
    fn default() -> Self {
        Self::empty()
    }
}

/// A newtype wrapper for the three primary built-in RangeProcessors.
pub enum NativeRangeProcessor {
    /// A newtype wrapper for a `DateRangeProcessor`
    Date(Pin<Box<ffi::DateRangeProcessor>>),
    /// A newtype wrapper for a `NumberRangeProcessor`
    Number(Pin<Box<ffi::NumberRangeProcessor>>),
    /// A newtype wrapper for a `RangeProcessor`
    String(Pin<Box<ffi::RangeProcessor>>),
}

impl NativeRangeProcessor {
    /// Create a new `RangeProcessor` that works on date strings
    pub fn date<U>(
        slot: impl Into<ffi::valueno>,
        marker_string: impl Into<Option<U>>,
        flags: impl Into<Option<RangeProcessorFlags>>,
        epoch_year: impl Into<Option<i32>>,
    ) -> Self
    where
        U: AsRef<str> + Default,
    {
        let flags = flags.into().unwrap_or_default();
        let epoch_year = epoch_year.into().unwrap_or(1970);
        cxx::let_cxx_string!(marker_string = marker_string.into().unwrap_or_default().as_ref());
        Self::Date(
            ffi::DateRangeProcessor::new1(
                slot.into(),
                &marker_string,
                flags.bits().into(),
                epoch_year.into(),
            )
            .within_box(),
        )
    }

    /// Create a numeric RangeProcessor
    pub fn number<T>(
        slot: impl Into<ffi::valueno>,
        marker_string: impl Into<Option<T>>,
        flags: impl Into<Option<RangeProcessorFlags>>,
    ) -> Self
    where
        T: AsRef<str> + Default,
    {
        let flags = flags.into().unwrap_or_default();
        cxx::let_cxx_string!(marker_string = marker_string.into().unwrap_or_default().as_ref());
        Self::Number(
            ffi::NumberRangeProcessor::new(slot.into(), &marker_string, flags.bits().into())
                .within_box(),
        )
    }

    /// Create a string-based RangeProcessor
    pub fn string<T>(
        slot: impl Into<ffi::valueno>,
        marker_string: impl Into<Option<T>>,
        flags: impl Into<Option<RangeProcessorFlags>>,
    ) -> Self
    where
        T: AsRef<str> + Default,
    {
        let flags = flags.into().unwrap_or_default();
        cxx::let_cxx_string!(marker_string = marker_string.into().unwrap_or_default().as_ref());
        Self::String(
            ffi::RangeProcessor::new2(slot.into(), &marker_string, flags.bits().into())
                .within_box(),
        )
    }

    #[doc(hidden)]
    pub fn upcast(&mut self) -> Pin<&mut ffi::RangeProcessor> {
        match self {
            Self::Date(rp) => unsafe { ffi::upcast(rp.as_mut()) },
            Self::Number(rp) => unsafe { ffi::upcast(rp.as_mut()) },
            Self::String(rp) => rp.as_mut(),
        }
    }
}

/// An `RSet` is used to hold documents marked as explicitly relevant to the current search
///
/// Useful for generating `MSet` and `ESet` instances
pub struct RSet(Pin<Box<ffi::RSet>>);

impl RSet {
    /// Add a document to this reference set by way of a `Match`
    pub fn add_document(&mut self, it: impl AsRef<ffi::MSetIterator>) {
        self.0.as_mut().add_document1(it.as_ref())
    }

    /// Add a document to this reference set by way of a [`DocId`][crate::DocId]
    pub fn add_document_by_id(&mut self, id: impl Into<ffi::docid>) {
        self.0.as_mut().add_document(id.into())
    }

    /// Returns `true` if this `RSet` contains the document specified by the given `Match`
    pub fn contains(&self, it: impl AsRef<ffi::MSetIterator>) -> bool {
        self.0.contains1(it.as_ref())
    }

    /// Returns `true` if this `RSet` contains the document specified by the given `id`
    pub fn contains_id(&self, id: impl Into<ffi::docid>) -> bool {
        self.0.contains(id.into())
    }

    /// Returns `true` if this `RSet` is empty
    pub fn empty(&self) -> bool {
        self.0.empty()
    }

    /// Remove the document specified by the given `Match` from this `RSet`
    pub fn remove_document(&mut self, it: impl AsRef<ffi::MSetIterator>) {
        self.0.as_mut().remove_document1(it.as_ref())
    }

    /// Remove the document specified by the given `DocId` from this `RSet`
    pub fn remove_document_by_id(&mut self, id: impl Into<ffi::docid>) {
        self.0.as_mut().remove_document(id.into())
    }

    /// Return the size of this `RSet`
    pub fn size(&self) -> u32 {
        self.0.size().into()
    }
}

impl AsRef<ffi::RSet> for RSet {
    fn as_ref(&self) -> &ffi::RSet {
        &self.0
    }
}

impl Default for RSet {
    fn default() -> Self {
        Self(ffi::RSet::new2().within_box())
    }
}

impl FromIterator<DocId> for RSet {
    fn from_iter<T: IntoIterator<Item = DocId>>(iter: T) -> Self {
        let mut rset = RSet::default();
        for id in iter {
            rset.add_document_by_id(id);
        }
        rset
    }
}

impl FromIterator<Match> for RSet {
    fn from_iter<T: IntoIterator<Item = Match>>(iter: T) -> Self {
        let mut rset = RSet::default();
        for m in iter {
            rset.add_document(m);
        }
        rset
    }
}
