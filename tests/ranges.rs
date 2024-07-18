mod common;

use xapian_rs::{
    Enquire, NativeRangeProcessor, QueryParser, RangeProcessorFlags, Stem, StemStrategy,
};

#[test]
fn search_range() {
    let museum_db = common::seed_objects(None);

    let mut qp = QueryParser::default();
    qp.set_stemmer(Stem::for_language("en"));
    qp.set_stemming_strategy(StemStrategy::Some);
    qp.add_prefix("title", "S:");
    qp.add_prefix("description", "XD:");

    let size_proc = Box::leak(Box::new(NativeRangeProcessor::number(
        0,
        "mm",
        RangeProcessorFlags::SUFFIX | RangeProcessorFlags::REPEATED,
    )));
    let year_proc = Box::leak(Box::new(NativeRangeProcessor::number(
        1,
        "year:",
        RangeProcessorFlags::default(),
    )));

    qp.add_rangeprocessor(size_proc.upcast(), None);
    qp.add_rangeprocessor(year_proc.upcast(), None);

    let query = qp.parse_query::<&str>("clock", None, None);

    let mut enquire = Enquire::new(museum_db);
    enquire.set_query(&query, None);

    let mset = enquire.mset(0, 10, 10, None, None);
    let matches = mset.matches();
    assert_eq!(matches.count(), 10);
}
