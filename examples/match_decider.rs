use std::path::PathBuf;

use clap::Parser;
use xapian_rs::{Database, Enquire, MatchDecider, QueryParser, Stem};

#[derive(Parser)]
struct Args {
    db: PathBuf,
    reject: String,
    queries: Vec<String>,
}

pub struct RejectDecider {
    slot: xapian_rs::ffi::valueno,
    term: String,
}

impl MatchDecider for RejectDecider {
    fn is_match(&self, doc: &xapian_rs::Document) -> bool {
        !matches!(doc.value(self.slot), Some(x) if x.as_ref() == self.term.as_bytes())
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
