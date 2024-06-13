use std::{cell::RefCell, collections::BTreeMap, path::PathBuf, rc::Rc};

use clap::Parser;
use xapian_rs::{Database, Enquire, FromValue, MatchSpy, QueryParser, Stem};

#[derive(Parser)]
struct Args {
    db: PathBuf,
    queries: Vec<String>,
}

#[derive(Clone)]
pub struct StringValueSpy<T: PartialEq> {
    slot: xapian_rs::Slot,
    stats: Rc<RefCell<BTreeMap<T, usize>>>,
}

impl<T: FromValue> StringValueSpy<T> {
    pub fn new(slot: impl Into<xapian_rs::Slot>) -> Self {
        Self {
            slot: slot.into(),
            stats: Default::default(),
        }
    }
}

impl<T: FromValue + Ord> MatchSpy for StringValueSpy<T> {
    fn observe(&self, doc: &xapian_rs::Document, _: f64) {
        if let Some(Ok(key)) = doc.value::<T>(self.slot) {
            let mut stats = self.stats.borrow_mut();
            let count = stats.entry(key).or_insert(0);
            *count += 1;
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stemmer = Stem::for_language("english");

    let args = Args::parse();
    let db = Database::open(args.db, None);
    let qstr = args.queries.join(" ");

    let spy = StringValueSpy::<String>::new(0);
    let mut qp = QueryParser::default();
    qp.add_prefix("keyword", "K:");
    qp.add_prefix("name", "N:");
    qp.add_prefix("id", "I:");

    qp.set_stemmer(stemmer);
    let query = qp.parse_query(qstr, None, "S:");
    eprintln!("query:{query}");

    for term in qp.stoplist() {
        eprintln!("stopword:{term}");
    }

    for term in query.terms() {
        for t in qp.unstem(&term) {
            eprintln!("unstem:{term}={t}");
        }
    }

    let mut enquire = Enquire::new(db, &query, None);
    enquire.add_matchspy(&spy);
    let results = enquire.mset(0, 100, 100, None, None);
    for (key, count) in spy.stats.borrow().iter() {
        eprintln!("spy:{key}={count}")
    }

    for m in results.matches() {
        println!("{}", m.document());
    }

    Ok(())
}
