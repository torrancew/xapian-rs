use crate::ffi;

use std::{
    cell::{Ref, RefCell},
    collections::HashSet,
    fmt::{self, Debug, Display},
    ops::Deref,
    pin::Pin,
    rc::Rc,
};

use autocxx::{
    cxx::{CxxString, UniquePtr},
    prelude::*,
};

/// An individual expansion term from an `ESet`, with access to position and frequency information
pub struct Expansion {
    value: UniquePtr<CxxString>,
    ptr: Pin<Box<ffi::ESetIterator>>,
}

impl Expansion {
    pub(crate) fn new(ptr: Pin<Box<ffi::ESetIterator>>) -> Self {
        let value = ffi::shim::eset_iterator_term(&ptr);
        Self { value, ptr }
    }

    /// Get the weight of this term
    pub fn weight(&self) -> f64 {
        self.ptr.get_weight()
    }
}

impl AsRef<str> for Expansion {
    fn as_ref(&self) -> &str {
        self.value.to_str().unwrap()
    }
}

impl AsRef<CxxString> for Expansion {
    fn as_ref(&self) -> &CxxString {
        &self.value
    }
}

impl AsRef<ffi::ESetIterator> for Expansion {
    fn as_ref(&self) -> &ffi::ESetIterator {
        &self.ptr
    }
}

impl Debug for Expansion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Expansion")
            .field("value", &self.value)
            .finish()
    }
}

impl Display for Expansion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl PartialEq for Expansion {
    fn eq(&self, other: &Self) -> bool {
        self.value.as_bytes() == other.value.as_bytes()
    }
}

impl PartialOrd for Expansion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(self.value.as_bytes(), other.value.as_bytes())
    }
}
/// A strategy to apply to a `Stem` instance
pub enum StemStrategy {
    /// Generate only unstemmed terms
    None,
    /// Generate both stemmed () and unstemmed terms.
    /// Stemmed terms are prefixed with `Z`.
    /// Unstemmed terms contain no positional information.
    Some,
    /// Generate only stemmed terms, with no `Z` prefix.
    All,
    /// Generate only stemmed terms, prefixed with a `Z`.
    AllZ,
    /// Generate both stemmed () and unstemmed terms.
    /// Stemmed terms are prefixed with `Z`.
    /// All terms contain positional information.
    SomeFullPos,
}

impl From<ffi::QueryParser_stem_strategy> for StemStrategy {
    fn from(value: ffi::QueryParser_stem_strategy) -> Self {
        use ffi::QueryParser_stem_strategy::*;
        use StemStrategy::*;

        match value {
            STEM_NONE => None,
            STEM_SOME => Some,
            STEM_ALL => All,
            STEM_ALL_Z => AllZ,
            STEM_SOME_FULL_POS => SomeFullPos,
        }
    }
}

impl From<ffi::TermGenerator_stem_strategy> for StemStrategy {
    fn from(value: ffi::TermGenerator_stem_strategy) -> Self {
        use ffi::TermGenerator_stem_strategy::*;
        use StemStrategy::*;

        match value {
            STEM_NONE => None,
            STEM_SOME => Some,
            STEM_ALL => All,
            STEM_ALL_Z => AllZ,
            STEM_SOME_FULL_POS => SomeFullPos,
        }
    }
}

impl From<StemStrategy> for ffi::QueryParser_stem_strategy {
    fn from(value: StemStrategy) -> Self {
        use ffi::QueryParser_stem_strategy::*;
        use StemStrategy::*;

        match value {
            None => STEM_NONE,
            Some => STEM_SOME,
            All => STEM_ALL,
            AllZ => STEM_ALL_Z,
            SomeFullPos => STEM_SOME_FULL_POS,
        }
    }
}

impl From<StemStrategy> for ffi::TermGenerator_stem_strategy {
    fn from(value: StemStrategy) -> Self {
        use ffi::TermGenerator_stem_strategy::*;
        use StemStrategy::*;

        match value {
            None => STEM_NONE,
            Some => STEM_SOME,
            All => STEM_ALL,
            AllZ => STEM_ALL_Z,
            SomeFullPos => STEM_SOME_FULL_POS,
        }
    }
}

/// An instance of a Stemming algorithm
pub struct Stem(Pin<Box<ffi::Stem>>);

impl Stem {
    /// List all languages with an available Stem instance in the underlying Xapian library
    pub fn languages() -> HashSet<String> {
        ffi::Stem::get_available_languages()
            .to_string()
            .split(' ')
            .map(String::from)
            .collect()
    }

    /// Returns true if this Stem instance is a no-op
    pub fn is_noop(&self) -> bool {
        self.0.is_none()
    }

    /// Returns a stemmer instance for the given language, if one exists
    pub fn for_language(lang: impl AsRef<str>) -> Self {
        cxx::let_cxx_string!(lang = lang.as_ref());
        Self(ffi::Stem::new3(&lang).within_box())
    }

    /// Run the underlying stem algorithm against `word`, returning its stemmed form
    pub fn stem(&self, word: impl AsRef<str>) -> String {
        cxx::let_cxx_string!(word = word.as_ref());
        ffi::shim::stemmer_stem(&self.0, &word).to_string()
    }
}

impl AsRef<ffi::Stem> for Stem {
    fn as_ref(&self) -> &ffi::Stem {
        &self.0
    }
}

pub enum StopStrategy {
    None,
    All,
    Stemmed,
}

impl From<StopStrategy> for ffi::TermGenerator_stop_strategy {
    fn from(value: StopStrategy) -> Self {
        use ffi::TermGenerator_stop_strategy::*;
        use StopStrategy::*;

        match value {
            None => STOP_NONE,
            All => STOP_ALL,
            Stemmed => STOP_STEMMED,
        }
    }
}

impl From<ffi::TermGenerator_stop_strategy> for StopStrategy {
    fn from(value: ffi::TermGenerator_stop_strategy) -> Self {
        use ffi::TermGenerator_stop_strategy::*;
        use StopStrategy::*;

        match value {
            STOP_NONE => None,
            STOP_ALL => All,
            STOP_STEMMED => Stemmed,
        }
    }
}

/// Determines whether a given term matches a `stopword`.
/// Stopwords are not typically indexed or included in parsed queries.
pub trait Stopper {
    /// Evaluate whether a given word is a stopword.
    fn is_stopword(&self, word: &str) -> bool;
}

pub(crate) trait FfiStopper: Stopper + Sized + 'static {
    fn into_ffi(self) -> &'static StopperObj {
        Box::leak(Box::new(StopperObj::from(self)))
    }
}

impl<S: Stopper + Sized + 'static> FfiStopper for S {}

impl<F> Stopper for F
where
    F: Fn(&str) -> bool,
{
    fn is_stopword(&self, word: &str) -> bool {
        self(word)
    }
}

impl Stopper for HashSet<String> {
    fn is_stopword(&self, word: &str) -> bool {
        self.contains(word)
    }
}

impl Stopper for HashSet<&str> {
    fn is_stopword(&self, word: &str) -> bool {
        self.contains(word)
    }
}

pub(crate) struct StopperObj(Rc<RefCell<ffi::RustStopper>>);

impl StopperObj {
    pub(crate) fn upcast(&self) -> impl Deref<Target = ffi::shim::FfiStopper> + '_ {
        Ref::map(self.0.borrow(), |s| s.as_ref())
    }
}

impl<T: Stopper + 'static> From<T> for StopperObj {
    fn from(value: T) -> Self {
        Self(ffi::RustStopper::from_trait(value))
    }
}

/// An individual `term`, with access to position and frequency information
pub struct Term {
    value: UniquePtr<CxxString>,
    ptr: Pin<Box<ffi::TermIterator>>,
}

impl Term {
    pub(crate) fn new(ptr: Pin<Box<ffi::TermIterator>>) -> Self {
        let value = ffi::shim::term_iterator_term(&ptr);
        Self { value, ptr }
    }

    /// Get the frequency of this term
    pub fn frequency(&self) -> u32 {
        self.ptr.get_termfreq().into()
    }

    /// Get the number of occurrences of this term
    pub fn positions_len(&self) -> u32 {
        self.ptr.positionlist_count().into()
    }

    /// Get an iterator over the specific occurrences of this term
    pub fn positions(&self) -> crate::iter::PositionIter {
        crate::iter::PositionIter::new(self)
    }

    /// Get the within-document-frequency for this term
    pub fn wdf(&self) -> u32 {
        self.ptr.get_wdf().into()
    }
}

impl AsRef<str> for Term {
    fn as_ref(&self) -> &str {
        self.value.to_str().unwrap()
    }
}

impl AsRef<CxxString> for Term {
    fn as_ref(&self) -> &CxxString {
        &self.value
    }
}

impl AsRef<ffi::TermIterator> for Term {
    fn as_ref(&self) -> &ffi::TermIterator {
        &self.ptr
    }
}

impl Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Term").field("value", &self.value).finish()
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.value.as_bytes() == other.value.as_bytes()
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(self.value.as_bytes(), other.value.as_bytes())
    }
}

/// An instance of a Xapian TermGenerator, which can be used to index text with optional stemming
pub struct TermGenerator(Pin<Box<ffi::TermGenerator>>);

impl TermGenerator {
    /// Increase the term position used when indexing index_text
    ///
    /// Useful to keep phrases from spanning logically separate regions of text
    pub fn increase_termpos(&mut self, delta: impl Into<Option<u32>>) {
        self.0
            .as_mut()
            .increase_termpos(delta.into().unwrap_or(100).into());
    }

    /// Index some text
    ///
    /// The within-document-frequency for the text will be increased by `increment` (or 1 if `None`)
    /// If provided, the text will be prefixed with `prefix`
    pub fn index_text<T>(
        &mut self,
        text: impl AsRef<str>,
        increment: impl Into<Option<u32>>,
        prefix: impl Into<Option<T>>,
    ) where
        T: AsRef<str> + Default,
    {
        cxx::let_cxx_string!(text = text.as_ref());
        cxx::let_cxx_string!(prefix = prefix.into().unwrap_or_default().as_ref());
        self.0
            .as_mut()
            .index_text1(&text, increment.into().unwrap_or(1).into(), &prefix)
    }

    /// Set the currently active database
    pub fn set_database(&mut self, db: impl AsRef<ffi::WritableDatabase>) {
        self.0.as_mut().set_database(db.as_ref())
    }

    /// Set the currently active document
    pub fn set_document(&mut self, doc: impl AsRef<ffi::Document>) {
        self.0.as_mut().set_document(doc.as_ref())
    }

    /// Set the stemmer to be used when indexing text
    pub fn set_stemmer(&mut self, stem: impl AsRef<ffi::Stem>) {
        self.0.as_mut().set_stemmer(stem.as_ref())
    }

    /// Set the stemming strategy to be used when indexing text
    pub fn set_stemming_strategy(&mut self, strategy: impl Into<ffi::TermGenerator_stem_strategy>) {
        self.0.as_mut().set_stemming_strategy(strategy.into())
    }

    /// Set the stopper to be used when indexing text
    pub fn set_stopper<T: crate::Stopper + 'static>(&mut self, stopper: impl Into<Option<T>>) {
        let stopper = stopper.into().map_or(std::ptr::null(), |s| {
            Deref::deref(&s.into_ffi().upcast()) as *const _
        });
        unsafe { ffi::shim::term_generator_set_stopper(self.0.as_mut(), stopper) }
    }

    /// Set the stopper strategy to be used when indexing text
    pub fn set_stopper_strategy(&mut self, strategy: impl Into<ffi::TermGenerator_stop_strategy>) {
        self.0.as_mut().set_stopper_strategy(strategy.into())
    }
}

impl Default for TermGenerator {
    fn default() -> Self {
        Self(ffi::TermGenerator::new2().within_box())
    }
}
