use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Args {
    db: PathBuf,
}

#[derive(Deserialize, Serialize)]
pub struct LicenseEntry {
    id: String,
    name: String,
    keywords: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut db = xapian_rs::WritableDatabase::open(args.db, None, None);
    let mut indexer = xapian_rs::TermGenerator::default();
    indexer.set_stemmer(xapian_rs::Stem::for_language("english"));

    let osl_data: Vec<LicenseEntry> =
        reqwest::blocking::get("https://api.opensource.org/licenses/")?.json()?;

    for license in osl_data {
        let mut doc = xapian_rs::Document::default();
        doc.set_value(0.into(), &license.id);
        doc.set_data(serde_json::to_string(&license).unwrap());

        indexer.set_document(&doc);
        indexer.index_text(&license.id, None, "I:");
        indexer.index_text(&license.name, None, "N:");

        indexer.index_text(&license.name, None, "S:");
        indexer.index_text(&license.id, None, "S:");
        indexer.index_text::<&str>(&license.id, None, None);

        for kw in license.keywords {
            doc.add_term(format!("K:{kw}"), None)
        }

        println!("{doc}");
        db.add_document(doc);
    }

    Ok(())
}
