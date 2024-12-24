mod common;

use xapian_rs::{Enquire, NumberRangeProcessor, QueryParser, Stem, StemStrategy};

#[test]
fn search_range() {
    let museum_db = common::seed_objects(None);

    let mut qp = QueryParser::default();
    qp.set_stemmer(Stem::for_language("en"));
    qp.set_stemming_strategy(StemStrategy::Some);
    qp.add_prefix("title", "S:");
    qp.add_prefix("description", "XD:");

    qp.add_rangeprocessor("mm", 0, NumberRangeProcessor, true, true, None);
    qp.add_rangeprocessor("year:", 1, NumberRangeProcessor, false, false, None);

    let query = qp.parse_query::<&str>("clock AND year:1900..2000", None, None);
    eprintln!("{query:?}");

    let mut enquire = Enquire::new(&museum_db);
    enquire.set_query(&query, None);

    let mset = enquire.mset(0, 10, 10, None);
    let matches = mset.matches();
    assert_eq!(matches.count(), 10);

    let query = qp.parse_query::<&str>("clock AND year:2000..", None, None);
    eprintln!("{query:?}");

    let mut enquire = Enquire::new(museum_db);
    enquire.set_query(&query, None);

    let mset = enquire.mset(0, 10, 10, None);
    let matches = mset.matches();
    assert_eq!(matches.count(), 0);
}
