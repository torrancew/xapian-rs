use crate::ffi;

use std::{
    fmt::{self, Debug, Display},
    pin::Pin,
};

use autocxx::{cxx, prelude::*};
use bytes::Bytes;

/// A document in a Xapian database
pub struct Document(Pin<Box<ffi::Document>>);

impl Document {
    pub(crate) fn new(ptr: Pin<Box<ffi::Document>>) -> Self {
        Self(ptr)
    }

    /// Add a boolean term to the document
    pub fn add_boolean_term(&mut self, term: impl AsRef<str>) {
        cxx::let_cxx_string!(term = term.as_ref());
        self.0.as_mut().add_boolean_term(&term)
    }

    /// Add an occurrence of `term` at the position given by `pos`
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

    /// Add a term to the document, without positional information
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

    /// Remove all terms and postings from the document
    pub fn clear_terms(&mut self) {
        self.0.as_mut().clear_terms()
    }

    /// Get the data blob stored in this document
    pub fn data(&self) -> Bytes {
        ffi::cxx_bytes(&self.0.get_data())
    }

    /// Get the document ID (if any) associated with this document
    pub fn id(&self) -> Option<crate::DocId> {
        crate::DocId::new(self.0.get_docid())
    }

    /// Remove `term` and all postings associated with it from this document
    pub fn remove_term(&mut self, term: impl AsRef<str>) {
        cxx::let_cxx_string!(term = term.as_ref());
        self.0.as_mut().remove_term(&term)
    }

    /// Set the data blob stored alongside this document
    pub fn set_data(&mut self, data: impl AsRef<[u8]>) {
        cxx::let_cxx_string!(data = data);
        self.0.as_mut().set_data(&data);
    }

    /// Set the value stored in the given slot number
    ///
    /// Xapian values are stored as strings, but are often more useful in some other form.
    /// To accomodate this, [`ToValue`][crate::ToValue] is used to serialize data in a
    /// Xapian-friendly fashion. This trait is already implemented for most numeric primitives,
    /// string types and byte collections.
    pub fn set_value(&mut self, slot: impl Into<crate::Slot>, value: impl crate::ToValue) {
        cxx::let_cxx_string!(value = value.serialize());
        self.0
            .as_mut()
            .add_value(ffi::valueno::from(slot.into()), &value)
    }

    /// Retrieve an iterator over the terms in this document
    pub fn terms(&self) -> crate::iter::TermIter {
        crate::iter::TermIter::new(
            self.0.termlist_begin().within_box(),
            self.0.termlist_end().within_box(),
        )
    }

    /// Retrieve the value (if any) stored in the given slot number
    ///
    /// Xapian values are stored as strings, but are often more useful in some other form.
    /// To accomodate this, [`FromValue`][crate::FromValue] is used to deserialize data
    /// from its Xapian representation. This trait is already implemented for most numeric
    /// primitives, string types and byte collections.
    ///
    /// Returns `None` when there is no value stored in `slot`
    /// Returns `Some(Err(T::Error)` when there is a value but deserialization fails
    /// Returns `Some(Ok(T))` otherwise
    pub fn value<T: crate::FromValue>(
        &self,
        slot: impl Into<crate::Slot>,
    ) -> Option<Result<T, T::Error>> {
        let s = self.0.get_value(ffi::valueno::from(slot.into()));
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
