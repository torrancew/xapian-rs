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

pub enum StemStrategy {
    None,
    Some,
    All,
    AllZ,
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

pub struct Stem(Pin<Box<ffi::Stem>>);

impl Stem {
    pub fn languages() -> HashSet<String> {
        ffi::Stem::get_available_languages()
            .to_string()
            .split(' ')
            .map(String::from)
            .collect()
    }

    pub fn is_noop(&self) -> bool {
        self.0.is_none()
    }

    pub fn for_language(lang: impl AsRef<str>) -> Self {
        cxx::let_cxx_string!(lang = lang.as_ref());
        Self(ffi::Stem::new3(&lang).within_box())
    }

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

pub trait Stopper {
    fn is_stopword(&self, word: &str) -> bool;

    fn into_ffi(self) -> &'static StopperWrapper
    where
        Self: Sized + 'static,
    {
        Box::leak(Box::new(StopperWrapper::from(self)))
    }
}

pub struct StopperWrapper(Rc<RefCell<ffi::RustStopper>>);

impl StopperWrapper {
    pub fn upcast(&self) -> impl Deref<Target = ffi::shim::FfiStopper> + '_ {
        Ref::map(self.0.borrow(), |s| s.as_ref())
    }
}

impl<T: Stopper + 'static> From<T> for StopperWrapper {
    fn from(value: T) -> Self {
        Self(ffi::RustStopper::from_trait(value))
    }
}

pub struct Term {
    value: UniquePtr<CxxString>,
    ptr: Pin<Box<ffi::TermIterator>>,
}

impl Term {
    pub(crate) fn new(ptr: Pin<Box<ffi::TermIterator>>) -> Self {
        let value = ffi::shim::term_iterator_term(&ptr);
        Self { value, ptr }
    }

    pub fn positions_len(&self) -> u32 {
        self.ptr.positionlist_count().into()
    }

    pub fn positions(&self) -> crate::iter::PositionIter {
        crate::iter::PositionIter::new(self)
    }

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

pub struct TermGenerator(Pin<Box<ffi::TermGenerator>>);

impl TermGenerator {
    pub fn increase_termpos(&mut self, delta: impl Into<Option<u32>>) {
        self.0
            .as_mut()
            .increase_termpos(delta.into().unwrap_or(100).into());
    }

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

    pub fn set_document(&mut self, doc: impl AsRef<ffi::Document>) {
        self.0.as_mut().set_document(doc.as_ref())
    }

    pub fn set_stemmer(&mut self, stem: impl AsRef<ffi::Stem>) {
        self.0.as_mut().set_stemmer(stem.as_ref())
    }

    pub fn set_stemming_strategy(
        &mut self,
        strategy: impl Into<Option<ffi::TermGenerator_stem_strategy>>,
    ) {
        self.0.as_mut().set_stemming_strategy(
            strategy
                .into()
                .unwrap_or(ffi::TermGenerator_stem_strategy::STEM_SOME),
        )
    }
}

impl Default for TermGenerator {
    fn default() -> Self {
        Self(ffi::TermGenerator::new2().within_box())
    }
}
