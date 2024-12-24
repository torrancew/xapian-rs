#[path = "../tests/common.rs"]
mod common;

use std::{collections::HashSet, path::PathBuf};

use clap::Parser;
use xapian_rs::{Database, Enquire, ExpandDecider, QueryParser, RSet, Stem};

const STOPWORDS: &str = include_str!("../tests/data/stopwords.txt");

#[derive(Parser)]
struct Args {
    db: PathBuf,
    queries: Vec<String>,
}

pub struct DescriptionDecider<'a>(HashSet<&'a str>);

impl<'a> FromIterator<&'a str> for DescriptionDecider<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl ExpandDecider for DescriptionDecider<'_> {
    fn should_keep(&self, term: &str) -> bool {
        term.starts_with("XD:") && !self.0.contains(term.trim_start_matches("XD:"))
    }
}

fn main() -> anyhow::Result<()> {
    let stemmer = Stem::for_language("english");

    let args = Args::parse();
    let db = Database::open(args.db.join("museum"), None);
    let qstr = args.queries.join(" ");

    let mut qp = QueryParser::default();
    qp.add_prefix("description", "XD:");

    let decider = DescriptionDecider::from_iter(STOPWORDS.lines());

    qp.set_stemmer(stemmer);
    let query = qp.parse_query(qstr, None, "S:");

    let mut enquire = Enquire::new(db);
    enquire.set_query(&query, None);
    let mset = enquire.mset(0, 100, 100, None);
    let matches = mset.matches();
    let rset = RSet::from_iter(matches.clone().take(2));
    for m in matches {
        println!("{}", m.document());
    }

    println!("Consider adding:");
    for term in enquire.eset(100, rset, 0, decider, 0.0).terms() {
        println!("\t{term}")
    }

    Ok(())
}
