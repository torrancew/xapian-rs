use crate::ffi;

use std::{
    fmt::{self, Debug, Display},
    num::NonZeroU32,
    pin::Pin,
};

use autocxx::{cxx, prelude::*};
use bytes::Bytes;

pub struct Document(Pin<Box<ffi::Document>>);

impl Document {
    pub(crate) fn new(ptr: Pin<Box<ffi::Document>>) -> Self {
        Self(ptr)
    }

    pub fn add_boolean_term(&mut self, term: impl AsRef<str>) {
        cxx::let_cxx_string!(term = term.as_ref());
        self.0.as_mut().add_boolean_term(&term)
    }

    pub fn add_posting(
        &mut self,
        term: impl AsRef<str>,
        pos: ffi::termpos,
        increment: impl Into<Option<ffi::termcount>>,
    ) {
        cxx::let_cxx_string!(term = term.as_ref());
        self.0
            .as_mut()
            .add_posting(&term, pos, increment.into().unwrap_or(1.into()))
    }

    pub fn add_term(
        &mut self,
        term: impl AsRef<str>,
        increment: impl Into<Option<ffi::termcount>>,
    ) {
        cxx::let_cxx_string!(term = term.as_ref());
        self.0
            .as_mut()
            .add_term(&term, increment.into().unwrap_or(1.into()))
    }

    pub fn clear_terms(&mut self) {
        self.0.as_mut().clear_terms()
    }

    pub fn data(&self) -> Bytes {
        ffi::cxx_bytes(&self.0.get_data())
    }

    pub fn id(&self) -> Option<NonZeroU32> {
        NonZeroU32::new(self.0.get_docid().into())
    }

    pub fn remove_term(&mut self, term: impl AsRef<str>) {
        cxx::let_cxx_string!(term = term.as_ref());
        self.0.as_mut().remove_term(&term)
    }

    pub fn set_data(&mut self, data: impl AsRef<[u8]>) {
        cxx::let_cxx_string!(data = data);
        self.0.as_mut().set_data(&data);
    }

    pub fn set_value(&mut self, slot: impl Into<ffi::valueno>, value: impl crate::ToValue) {
        cxx::let_cxx_string!(value = value.serialize());
        self.0.as_mut().add_value(slot.into(), &value)
    }

    pub fn terms(&self) -> crate::iter::TermIter {
        crate::iter::TermIter::new(
            self.0.termlist_begin().within_box(),
            self.0.termlist_end().within_box(),
        )
    }

    pub fn value<T: crate::FromValue>(
        &self,
        slot: impl Into<ffi::valueno>,
    ) -> Option<Result<T, T::Error>> {
        let s = self.0.get_value(slot.into());
        match s.is_empty() {
            true => None,
            false => Some(T::deserialize(ffi::cxx_bytes(&s))),
        }
    }
}

impl AsRef<ffi::Document> for Document {
    fn as_ref(&self) -> &ffi::Document {
        &self.0
    }
}

impl Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Document").field(&self.id()).finish()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self(ffi::Document::new3().within_box())
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0.get_data()))
    }
}

impl From<crate::Match> for Document {
    fn from(value: crate::Match) -> Self {
        value.document()
    }
}
