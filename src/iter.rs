use crate::ffi;

use std::pin::Pin;

use autocxx::prelude::*;

pub struct MSetIter<'mset> {
    size: (u32, u32),
    cursor_fwd: Pin<Box<ffi::MSetIterator>>,
    cursor_rev: Pin<Box<ffi::MSetIterator>>,
    mset: &'mset crate::MSet,
}

impl<'mset> MSetIter<'mset> {
    pub(crate) fn new(mset: &'mset crate::MSet) -> Self {
        let size = (mset.size(), 0);
        Self {
            size,
            cursor_fwd: mset.begin(),
            cursor_rev: mset.end(),
            mset,
        }
    }
}

impl<'mset> Iterator for MSetIter<'mset> {
    type Item = crate::Match;

    fn next(&mut self) -> Option<Self::Item> {
        // cursor_fwd starts out pointing to the first element
        // and therefore it yields an element before incrementing
        match &mut self.cursor_fwd {
            x if x == &self.cursor_rev || x == &self.mset.end() => None,
            c => {
                let item = crate::Match::new(c.clone());
                ffi::shim::mset_iterator_increment(c.as_mut());
                self.size.1 += 1;
                Some(item)
            }
        }
    }
}

impl<'mset> DoubleEndedIterator for MSetIter<'mset> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // cursor_rev starts out pointing after the last element
        // and therefore it decrements before it yields
        match &mut self.cursor_rev {
            x if x == &self.cursor_fwd || x == &self.mset.begin() => None,
            c => {
                ffi::shim::mset_iterator_decrement(c.as_mut());
                self.size.1 += 1;
                Some(crate::Match::new(c.clone()))
            }
        }
    }
}

pub struct PositionIter {
    size: (u32, u32),
    cursor: Pin<Box<ffi::PositionIterator>>,
    end: Pin<Box<ffi::PositionIterator>>,
}

impl PositionIter {
    pub(crate) fn new(term: &crate::Term) -> Self {
        let size = (term.positions_len(), 0);
        let term_iter = AsRef::<ffi::TermIterator>::as_ref(term);
        Self {
            size,
            cursor: term_iter.positionlist_begin().within_box(),
            end: term_iter.positionlist_end().within_box(),
        }
    }
}

impl Iterator for PositionIter {
    type Item = crate::Position;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cursor == self.end {
            false => {
                let item = ffi::shim::position_iterator_position(&self.cursor);
                ffi::shim::position_iterator_increment(self.cursor.as_mut());
                self.size.1 += 1;
                Some(item.into())
            }
            true => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (total, offset) = self.size;
        let remaining = (total - offset).try_into().unwrap();
        (remaining, Some(remaining))
    }
}

pub struct TermIter {
    cursor: Pin<Box<ffi::TermIterator>>,
    end: Pin<Box<ffi::TermIterator>>,
}

impl TermIter {
    pub(crate) fn new(
        start: Pin<Box<ffi::TermIterator>>,
        end: Pin<Box<ffi::TermIterator>>,
    ) -> Self {
        Self { cursor: start, end }
    }
}

impl AsRef<ffi::TermIterator> for TermIter {
    fn as_ref(&self) -> &ffi::TermIterator {
        &self.cursor
    }
}

impl Iterator for TermIter {
    type Item = crate::Term;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cursor == self.end {
            false => {
                let term = crate::Term::new(self.cursor.clone());
                ffi::shim::term_iterator_increment(self.cursor.as_mut());
                Some(term)
            }
            true => None,
        }
    }
}
