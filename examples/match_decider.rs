#[path = "../tests/common.rs"]
mod common;

use std::path::PathBuf;

use clap::Parser;
use xapian_rs::{Database, Enquire, MatchDecider, QueryParser, Slot, Stem};

#[derive(Parser)]
struct Args {
    db: PathBuf,
    reject: u32,
    queries: Vec<String>,
}

pub struct RejectDecider<T>
where
    T: xapian_rs::FromValue + xapian_rs::ToValue + Eq,
{
    slot: Slot,
    value: T,
}

impl<T> RejectDecider<T>
where
    T: xapian_rs::FromValue + xapian_rs::ToValue + Eq,
{
    pub fn new(slot: impl Into<xapian_rs::Slot>, value: T) -> Self {
        let slot = slot.into();
        Self { slot, value }
    }
}

impl<T> MatchDecider for RejectDecider<T>
where
    T: xapian_rs::FromValue + xapian_rs::ToValue + Eq,
{
    fn is_match(&self, doc: &xapian_rs::Document) -> bool {
        let value = doc.value::<T>(self.slot);
        matches!(value, Some(Ok(t)) if t != self.value)
    }
}

fn main() -> anyhow::Result<()> {
    let stemmer = Stem::for_language("english");

    let args = Args::parse();
    let db = Database::open(args.db.join("museum"), None);
    let qstr = args.queries.join(" ");
    let decider = RejectDecider::new(1, args.reject).into_ffi();

    let mut qp = QueryParser::default();
    qp.add_prefix("description", "XD:");

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

    let mut enquire = Enquire::new(db);
    enquire.set_query(&query, None);
    for m in enquire.mset(0, 100, 100, None, decider).matches() {
        println!("{}", m.document());
    }

    Ok(())
}
