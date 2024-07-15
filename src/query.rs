use crate::ffi::{self, ToCxxString};

use std::{
    fmt::{self, Debug, Display},
    ops::{BitAnd, BitOr, BitXor, Deref},
    pin::Pin,
};

use autocxx::{cxx, prelude::*};

#[repr(u32)]
#[non_exhaustive]
#[derive(PartialEq)]
pub enum Operator {
    And = 0,
    Or,
    AndNot,
    XOr,
    AndMaybe,
    Filter,
    Near,
    Phrase,
    ValueRange,
    ScaleWeight,
    EliteSet,
    ValueGe,
    ValueLe,
    Synonym,
    Max,
    Wildcard,
    Invalid = 99,
    LeafTerm = 100,
    LeafPostingSource,
    LeafMatchAll,
    LeafMatchNothing,
}

impl From<Operator> for ffi::Query_op {
    fn from(value: Operator) -> Self {
        use ffi::Query_op::*;
        use Operator as Op;
        match value {
            Op::And => OP_AND,
            Op::Or => OP_OR,
            Op::AndNot => OP_AND_NOT,
            Op::XOr => OP_XOR,
            Op::AndMaybe => OP_AND_MAYBE,
            Op::Filter => OP_FILTER,
            Op::Near => OP_NEAR,
            Op::Phrase => OP_PHRASE,
            Op::ValueRange => OP_VALUE_RANGE,
            Op::ScaleWeight => OP_SCALE_WEIGHT,
            Op::EliteSet => OP_ELITE_SET,
            Op::ValueGe => OP_VALUE_GE,
            Op::ValueLe => OP_VALUE_LE,
            Op::Synonym => OP_SYNONYM,
            Op::Max => OP_MAX,
            Op::Wildcard => OP_WILDCARD,
            Op::Invalid => OP_INVALID,
            Op::LeafTerm => LEAF_TERM,
            Op::LeafPostingSource => LEAF_POSTING_SOURCE,
            Op::LeafMatchAll => LEAF_MATCH_ALL,
            Op::LeafMatchNothing => LEAF_MATCH_NOTHING,
        }
    }
}

impl From<ffi::Query_op> for Operator {
    fn from(value: ffi::Query_op) -> Self {
        use ffi::Query_op::*;
        use Operator as Op;
        match value {
            OP_AND => Op::And,
            OP_OR => Op::Or,
            OP_AND_NOT => Op::AndNot,
            OP_XOR => Op::XOr,
            OP_AND_MAYBE => Op::AndMaybe,
            OP_FILTER => Op::Filter,
            OP_NEAR => Op::Near,
            OP_PHRASE => Op::Phrase,
            OP_VALUE_RANGE => Op::ValueRange,
            OP_SCALE_WEIGHT => Op::ScaleWeight,
            OP_ELITE_SET => Op::EliteSet,
            OP_VALUE_GE => Op::ValueGe,
            OP_VALUE_LE => Op::ValueLe,
            OP_SYNONYM => Op::Synonym,
            OP_MAX => Op::Max,
            OP_WILDCARD => Op::Wildcard,
            OP_INVALID => Op::Invalid,
            LEAF_TERM => Op::LeafTerm,
            LEAF_POSTING_SOURCE => Op::LeafPostingSource,
            LEAF_MATCH_ALL => Op::LeafMatchAll,
            LEAF_MATCH_NOTHING => Op::LeafMatchNothing,
        }
    }
}

/// A parsed query, ready for use in a search
#[derive(Clone)]
pub struct Query(Pin<Box<ffi::Query>>);

impl Query {
    pub(crate) fn from_ffi(ptr: Pin<Box<ffi::Query>>) -> Self {
        Self(ptr)
    }

    pub fn combine(op: Operator, a: impl AsRef<ffi::Query>, b: impl AsRef<ffi::Query>) -> Self {
        Self(ffi::Query::new7(op.into(), a.as_ref(), b.as_ref()).within_box())
    }

    pub fn value_ge(slot: impl Into<crate::Slot>, lower: impl crate::ffi::ToCxxString) -> Self {
        Self(
            ffi::Query::new9(
                Operator::ValueGe.into(),
                ffi::valueno::from(slot.into()),
                &lower.to_cxx_string(),
            )
            .within_box(),
        )
    }

    pub fn value_le(slot: impl Into<crate::Slot>, upper: impl crate::ffi::ToCxxString) -> Self {
        Self(
            ffi::Query::new9(
                Operator::ValueLe.into(),
                ffi::valueno::from(slot.into()),
                &upper.to_cxx_string(),
            )
            .within_box(),
        )
    }

    pub fn value_range(
        slot: impl Into<crate::Slot>,
        lower: impl crate::ffi::ToCxxString,
        upper: impl crate::ffi::ToCxxString,
    ) -> Self {
        Self(
            ffi::Query::new10(
                Operator::ValueRange.into(),
                ffi::valueno::from(slot.into()),
                &lower.to_cxx_string(),
                &upper.to_cxx_string(),
            )
            .within_box(),
        )
    }

    pub fn is_invalid(&self) -> bool {
        self.operator() == Operator::Invalid
    }

    pub fn operator(&self) -> Operator {
        self.0.get_type().into()
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self(ffi::Query::new5(factor, self.as_ref()).within_box())
    }

    pub fn subqueries(&self) -> crate::iter::SubqueryIter {
        crate::iter::SubqueryIter::new(self.as_ref())
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

impl BitAnd for Query {
    type Output = Query;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::combine(Operator::And, &self, &rhs)
    }
}

impl BitOr for Query {
    type Output = Query;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::combine(Operator::Or, &self, &rhs)
    }
}

impl BitXor for Query {
    type Output = Query;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::combine(Operator::XOr, &self, &rhs)
    }
}

impl Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Query")
            .field(&self.0.get_description())
            .finish()
    }
}

impl Default for Query {
    fn default() -> Self {
        Self(ffi::Query::new().within_box())
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0.get_description()))
    }
}

/// A type for building [`Query`] objects from strings
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

    pub fn add_rangeprocessor<'g>(
        &mut self,
        range_proc: Pin<&mut ffi::RangeProcessor>,
        grouping: impl Into<Option<&'g str>>,
    ) {
        use crate::ffi::ToCxxString;

        let grouping = grouping
            .into()
            .map(|g| g.to_cxx_string().into_raw())
            .unwrap_or(std::ptr::null_mut());

        unsafe {
            self.0
                .as_mut()
                .add_rangeprocessor(range_proc.release(), grouping)
        }
    }

    pub fn set_stemmer(&mut self, stemmer: impl AsRef<ffi::Stem>) {
        self.0.as_mut().set_stemmer(stemmer.as_ref())
    }

    pub fn set_stemming_strategy(&mut self, strategy: impl Into<ffi::QueryParser_stem_strategy>) {
        self.0.as_mut().set_stemming_strategy(strategy.into())
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
        Query::from_ffi(
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
