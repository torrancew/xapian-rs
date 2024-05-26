use crate::ffi::{self, cxx_bytes, ToCxxString};

use std::{path::Path, pin::Pin};

use autocxx::{cxx, prelude::*};
use bytes::Bytes;

pub struct Database(Pin<Box<ffi::Database>>);

impl Database {
    /// Open a read-only Database at the provided path
    pub fn open(path: impl AsRef<Path>, flags: impl Into<Option<i32>>) -> Self {
        let flags = flags.into().unwrap_or(0);
        Self(ffi::Database::new1(&path.as_ref().to_cxx_string(), flags.into()).within_box())
    }

    // Detect whether a given term exists in the database
    pub fn term_exists(&self, term: impl AsRef<[u8]>) -> bool {
        cxx::let_cxx_string!(term = term);
        self.0.term_exists(&term)
    }
}

impl AsRef<ffi::Database> for Database {
    fn as_ref(&self) -> &ffi::Database {
        &self.0
    }
}

pub struct WritableDatabase(Pin<Box<ffi::WritableDatabase>>);

impl WritableDatabase {
    pub fn open(
        path: impl AsRef<Path>,
        flags: impl Into<Option<i32>>,
        block_size: impl Into<Option<i32>>,
    ) -> Self {
        let flags = flags.into().unwrap_or(0);
        let block_size = block_size.into().unwrap_or(0);
        Self(
            ffi::WritableDatabase::new1(
                &path.as_ref().to_cxx_string(),
                flags.into(),
                block_size.into(),
            )
            .within_box(),
        )
    }

    pub fn add_database(&mut self, db: impl AsRef<ffi::WritableDatabase>) {
        self.0.as_mut().add_database(db.as_ref())
    }

    pub fn add_document(&mut self, doc: impl AsRef<ffi::Document>) -> u32 {
        self.0.as_mut().add_document(doc.as_ref()).into()
    }

    pub fn add_spelling(&self, word: impl AsRef<str>, increment: impl Into<Option<u32>>) {
        let increment = increment.into().unwrap_or(1);
        cxx::let_cxx_string!(word = word.as_ref());
        self.0.add_spelling(&word, increment.into())
    }

    pub fn commit(&mut self) {
        self.0.as_mut().commit()
    }

    pub fn metadata(&self, key: impl AsRef<str>) -> Bytes {
        let db: &ffi::Database = self.as_ref();
        cxx::let_cxx_string!(key = key.as_ref());
        cxx_bytes(&db.get_metadata(&key))
    }
}

impl AsRef<ffi::Database> for WritableDatabase {
    fn as_ref(&self) -> &ffi::Database {
        ffi::shim::writable_database_downcast(&self.0)
    }
}

impl AsRef<ffi::WritableDatabase> for WritableDatabase {
    fn as_ref(&self) -> &ffi::WritableDatabase {
        &self.0
    }
}
