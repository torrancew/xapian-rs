#[path = "../tests/common.rs"]
mod common;

use std::{collections::HashSet, path::PathBuf};

use clap::Parser;
use xapian_rs::{Database, Enquire, QueryParser, Stem, Stopper};

#[derive(Parser)]
struct Args {
    db: PathBuf,
    queries: Vec<String>,
}

pub struct MyStopper(HashSet<String>);

impl<S: ToString> FromIterator<S> for MyStopper {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self(iter.into_iter().map(|item| item.to_string()).collect())
    }
}

impl Stopper for MyStopper {
    fn is_stopword(&self, word: &str) -> bool {
        self.0.contains(word)
    }
}

fn main() -> anyhow::Result<()> {
    let stemmer = Stem::for_language("english");

    let stopwords = ["a", "an", "the"];
    let stopper = MyStopper::from_iter(&stopwords);

    let args = Args::parse();
    let db = Database::open(args.db.join("museum"), None);
    let qstr = args.queries.join(" ");

    let mut qp = QueryParser::default();
    qp.add_prefix("description", "XD:");

    qp.set_stemmer(stemmer);
    qp.set_stopper(stopper);
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
    for m in enquire.mset(0, 100, 100, None).matches() {
        println!("{}", m.document());
    }

    Ok(())
}
