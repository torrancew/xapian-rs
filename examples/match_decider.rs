use std::path::PathBuf;

use clap::Parser;
use xapian_rs::{Database, Enquire, MatchDecider, QueryParser, Slot, Stem};

#[derive(Parser)]
struct Args {
    db: PathBuf,
    reject: f64,
    queries: Vec<String>,
}

pub struct RejectDecider<T: xapian_rs::FromValue> {
    slot: Slot,
    term: T,
}

impl<T: xapian_rs::FromValue> MatchDecider for RejectDecider<T> {
    fn is_match(&self, doc: &xapian_rs::Document) -> bool {
        let value = doc.value::<T>(self.slot);
        matches!(value, Some(Ok(t)) if t == self.term)
    }
}

fn main() -> anyhow::Result<()> {
    let stemmer = Stem::for_language("english");

    let args = Args::parse();
    let db = Database::open(args.db, None);
    let qstr = args.queries.join(" ");
    let decider = RejectDecider {
        slot: 0.into(),
        term: args.reject,
    }
    .into_ffi();

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

    let enquire = Enquire::new(db, &query, None);
    for m in enquire.mset(0, 100, 100, None, decider).matches() {
        println!("{}", m.document());
    }

    Ok(())
}
