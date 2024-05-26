use std::path::PathBuf;

use clap::Parser;
use xapian_rs::{Database, Enquire, QueryParser, SimpleStopper, Stem};

#[derive(Parser)]
struct Args {
    db: PathBuf,
    queries: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let stemmer = Stem::for_language("english");

    let stopwords = ["a", "an", "the"];
    let stopper = SimpleStopper::from_iter(stopwords);

    let args = Args::parse();
    let db = Database::open(args.db, None);
    let qstr = args.queries.join(" ");

    let mut qp = QueryParser::default();
    qp.add_prefix("keyword", "K:");
    qp.add_prefix("name", "N:");
    qp.add_prefix("id", "I:");

    qp.set_stemmer(stemmer);
    qp.set_stopper(&stopper);
    let query = qp.parse_query(qstr, None, "S:");
    eprintln!("query:{query}");
    for term in qp.stoplist() {
        eprintln!("stopword:{term}");
    }

    for term in qp.unstem("ZS:licens") {
        eprintln!("unstem:{term}");
    }

    let enquire = Enquire::new(db, &query, None);
    for m in enquire.mset(0, 100, 100).matches() {
        println!("{}", m.document());
    }

    Ok(())
}
