#[path = "../tests/common.rs"]
mod common;

use std::path::PathBuf;

use clap::Parser;
use xapian_rs::{Database, Enquire, FieldProcessor, Query, QueryParser, Stem};

#[derive(Parser)]
struct Args {
    db: PathBuf,
    queries: Vec<String>,
}

#[derive(Clone)]
struct UpperCaseField;

impl FieldProcessor for UpperCaseField {
    fn process(&self, term: &str) -> Option<Query> {
        let term = format!("XC:{}", term.to_uppercase());
        Some(Query::term(term, None, None))
    }
}

fn main() -> anyhow::Result<()> {
    let stemmer = Stem::for_language("english");

    let args = Args::parse();
    let db = Database::open(args.db.join("museum"), None);
    let qstr = args.queries.join(" ");
    eprintln!("qstr={qstr}");

    let mut qp = QueryParser::default();
    qp.add_custom_prefix("description", UpperCaseField);

    qp.set_stemmer(stemmer);
    let query = qp.parse_query(qstr, None, "");
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
    for m in enquire.mset(0, 100, 100, None).matches() {
        println!("{}", m.document());
    }

    Ok(())
}
