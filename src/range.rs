use crate::{ffi, ToValue};

use std::{cell::RefCell, rc::Rc};

use bitflags::bitflags;

#[cfg(feature = "chrono")]
mod date {
    use crate::ToValue;

    /// Handle a range of dates
    pub struct DateRangeProcessor;

    impl super::RangeProcessor for DateRangeProcessor {
        fn process_range(
            &self,
            start: &str,
            end: &str,
        ) -> (Option<bytes::Bytes>, Option<bytes::Bytes>) {
            (
                start
                    .parse::<chrono::NaiveDate>()
                    .ok()
                    .map(|d| d.serialize()),
                end.parse::<chrono::NaiveDate>().ok().map(|d| d.serialize()),
            )
        }
    }

    /// Handle a range of timestamps
    pub struct DateTimeRangeProcessor;

    impl super::RangeProcessor for DateTimeRangeProcessor {
        fn process_range(
            &self,
            start: &str,
            end: &str,
        ) -> (Option<bytes::Bytes>, Option<bytes::Bytes>) {
            (
                start
                    .parse::<chrono::NaiveDateTime>()
                    .ok()
                    .map(|dt| dt.serialize()),
                end.parse::<chrono::NaiveDateTime>()
                    .ok()
                    .map(|dt| dt.serialize()),
            )
        }
    }
}

#[cfg(feature = "chrono")]
pub use date::*;

/// Handle a numeric range
pub struct NumberRangeProcessor;

impl RangeProcessor for NumberRangeProcessor {
    fn process_range(
        &self,
        start: &str,
        end: &str,
    ) -> (Option<bytes::Bytes>, Option<bytes::Bytes>) {
        use crate::ToValue;
        (
            start.parse::<f32>().ok().map(|x| x.serialize()),
            end.parse::<f32>().ok().map(|x| x.serialize()),
        )
    }
}

/// A [`RangeProcessor`] can be used to customize how a [`crate::QueryParser`] expands range-based
/// queries
pub trait RangeProcessor {
    /// Convert the start and end of the range from a query string into a Query, if possible
    fn process_range(&self, start: &str, end: &str)
        -> (Option<bytes::Bytes>, Option<bytes::Bytes>);

    #[doc(hidden)]
    fn to_ffi(
        self,
        slot: impl Into<crate::Slot>,
        marker: impl Into<String>,
        is_suffix: bool,
        can_repeat: bool,
    ) -> &'static mut RangeProcessorWrapper
    where
        Self: Sized + 'static,
    {
        Box::leak(Box::new(RangeProcessorWrapper::new(
            slot, marker, is_suffix, can_repeat, self,
        )))
    }
}

impl<F, T> RangeProcessor for F
where
    F: Fn(&str, &str) -> (Option<T>, Option<T>),
    T: crate::ToValue,
{
    fn process_range(
        &self,
        start: &str,
        end: &str,
    ) -> (Option<bytes::Bytes>, Option<bytes::Bytes>) {
        let (start, end) = self(start, end);
        (start.map(|s| s.serialize()), end.map(|e| e.serialize()))
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

#[doc(hidden)]
pub struct RangeProcessorWrapper(Rc<RefCell<ffi::RustRangeProcessor>>);

impl RangeProcessorWrapper {
    pub fn upcast(&mut self) -> *mut ffi::shim::FfiRangeProcessor {
        use ffi::shim::FfiRangeProcessor_methods;
        self.0.borrow_mut().upcast()
    }

    pub fn new<R: RangeProcessor + 'static>(
        slot: impl Into<crate::Slot>,
        marker: impl Into<String>,
        is_suffix: bool,
        can_repeat: bool,
        range_processor: R,
    ) -> Self {
        Self(ffi::RustRangeProcessor::new(
            slot,
            marker,
            is_suffix,
            can_repeat,
            range_processor,
        ))
    }
}

/// Handle a string range (effectively a no-op, relying on lexical sorting)
pub struct StringRangeProcessor;

impl RangeProcessor for StringRangeProcessor {
    fn process_range(
        &self,
        start: &str,
        end: &str,
    ) -> (Option<bytes::Bytes>, Option<bytes::Bytes>) {
        (
            (!start.is_empty()).then(|| start.serialize()),
            (!end.is_empty()).then(|| end.serialize()),
        )
    }
}
