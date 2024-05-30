use crate::ffi::{self, ToCxxString};

use std::{
    fmt::{self, Debug, Display},
    ops::Deref,
    pin::Pin,
};

use autocxx::{cxx, prelude::*};

pub struct Query(Pin<Box<ffi::Query>>);

impl Query {
    pub(crate) fn new(ptr: Pin<Box<ffi::Query>>) -> Self {
        Self(ptr)
    }

    pub fn terms(&self) -> crate::iter::TermIter {
        crate::iter::TermIter::new(
            self.0.get_terms_begin().within_box(),
            self.0.get_terms_end().within_box(),
        )
    }

    pub fn unique_terms(&self) -> crate::iter::TermIter {
        crate::iter::TermIter::new(
            self.0.get_unique_terms_begin().within_box(),
            self.0.get_unique_terms_end().within_box(),
        )
    }
}

impl AsRef<ffi::Query> for Query {
    fn as_ref(&self) -> &ffi::Query {
        &self.0
    }
}

impl Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Query")
            .field(&self.0.get_description())
            .finish()
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0.get_description()))
    }
}

pub struct QueryParser(Pin<Box<ffi::QueryParser>>);

impl QueryParser {
    pub fn add_prefix<T>(&mut self, field: impl AsRef<str>, prefix: impl Into<Option<T>>)
    where
        T: AsRef<str> + Default,
    {
        cxx::let_cxx_string!(field = field.as_ref());
        cxx::let_cxx_string!(prefix = prefix.into().unwrap_or_default().as_ref());
        self.0.as_mut().add_prefix(&field, &prefix)
    }

    pub fn add_boolean_prefix<T, U>(
        &mut self,
        field: impl AsRef<str>,
        prefix: impl Into<Option<T>>,
        grouping: impl Into<Option<U>>,
    ) where
        T: AsRef<str> + Default,
        U: AsRef<str> + Default,
    {
        cxx::let_cxx_string!(field = field.as_ref());
        cxx::let_cxx_string!(prefix = prefix.into().unwrap_or_default().as_ref());
        let grouping = grouping
            .into()
            .map_or(std::ptr::null(), |g| g.as_ref().to_cxx_string().into_raw());
        unsafe {
            self.0
                .as_mut()
                .add_boolean_prefix(&field, &prefix, grouping)
        }
    }

    pub fn set_stemmer(&mut self, stemmer: impl AsRef<ffi::Stem>) {
        self.0.as_mut().set_stemmer(stemmer.as_ref())
    }

    pub fn set_stopper<T: crate::Stopper + 'static>(&mut self, stopper: impl Into<Option<T>>) {
        let stopper = stopper.into().map_or(std::ptr::null(), |s| {
            Deref::deref(&s.into_ffi().upcast()) as *const _
        });
        unsafe { ffi::shim::query_parser_set_stopper(self.0.as_mut(), stopper) }
    }

    pub fn parse_query<T>(
        &mut self,
        query: impl AsRef<str>,
        flags: impl Into<Option<ffi::QueryParser_feature_flag>>,
        default_prefix: impl Into<Option<T>>,
    ) -> Query
    where
        T: AsRef<str> + Default,
    {
        cxx::let_cxx_string!(query = query.as_ref());
        cxx::let_cxx_string!(default_prefix = default_prefix.into().unwrap_or_default().as_ref());
        let flags = flags
            .into()
            .unwrap_or(ffi::QueryParser_feature_flag::FLAG_DEFAULT) as u32;
        Query::new(
            self.0
                .as_mut()
                .parse_query(&query, flags.into(), &default_prefix)
                .within_box(),
        )
    }

    pub fn stoplist(&self) -> crate::iter::TermIter {
        crate::iter::TermIter::new(
            self.0.stoplist_begin().within_box(),
            self.0.stoplist_end().within_box(),
        )
    }

    pub fn unstem(&self, term: impl AsRef<str>) -> crate::iter::TermIter {
        cxx::let_cxx_string!(term = term.as_ref());
        crate::iter::TermIter::new(
            self.0.unstem_begin(&term).within_box(),
            self.0.unstem_end(&term).within_box(),
        )
    }
}

impl AsRef<ffi::QueryParser> for QueryParser {
    fn as_ref(&self) -> &ffi::QueryParser {
        &self.0
    }
}

impl Default for QueryParser {
    fn default() -> Self {
        Self(ffi::QueryParser::new2().within_box())
    }
}
