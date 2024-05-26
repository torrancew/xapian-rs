use crate::ffi;

use std::{
    collections::HashSet,
    fmt::{self, Debug, Display},
    pin::Pin,
};

use autocxx::{cxx::CxxString, prelude::*};

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

pub struct SimpleStopper(Pin<Box<ffi::SimpleStopper>>);

impl SimpleStopper {
    pub fn add(&mut self, word: impl AsRef<str>) {
        cxx::let_cxx_string!(word = word.as_ref());
        self.0.as_mut().add(&word)
    }

    pub fn stop_at(&self, word: impl AsRef<str>) -> bool {
        cxx::let_cxx_string!(word = word.as_ref());
        ffi::shim::simple_stopper_stop_at(&self.0, &word)
    }
}

impl AsRef<ffi::Stopper> for SimpleStopper {
    fn as_ref(&self) -> &ffi::Stopper {
        ffi::shim::simple_stopper_downcast(&self.0)
    }
}

impl Default for SimpleStopper {
    fn default() -> Self {
        Self(ffi::SimpleStopper::new().within_box())
    }
}

impl<S> FromIterator<S> for SimpleStopper
where
    S: AsRef<str>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut stopper = Self::default();
        for word in iter {
            stopper.add(word);
        }

        stopper
    }
}

pub struct Stopper<'s>(Pin<&'s ffi::Stopper>);

impl Stopper<'_> {
    pub fn stop_at(&self, word: impl AsRef<str>) -> bool {
        cxx::let_cxx_string!(word = word.as_ref());
        ffi::shim::stopper_stop_at(&self.0, &word)
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
    pub fn index_text<T>(
        &mut self,
        text: impl AsRef<str>,
        increment: impl Into<Option<ffi::termcount>>,
        prefix: impl Into<Option<T>>,
    ) where
        T: AsRef<str> + Default,
    {
        cxx::let_cxx_string!(text = text.as_ref());
        cxx::let_cxx_string!(prefix = prefix.into().unwrap_or_default().as_ref());
        self.0
            .as_mut()
            .index_text1(&text, increment.into().unwrap_or(1.into()), &prefix)
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
