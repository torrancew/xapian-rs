#![allow(dead_code)]

use std::path::PathBuf;

use csv::Reader;
use float_ord::FloatOrd;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use xapian_rs::{Document, Stem, TermGenerator, WritableDatabase};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct MuseumRecord {
    #[serde(rename = "id_NUMBER")]
    id_number: String,
    item_name: String,
    title: String,
    maker: String,
    date_made: String,
    place_made: String,
    materials: String,
    measurements: String,
    description: String,
    whole_part: String,
    collection: String,
}

#[derive(Deserialize, Serialize)]
pub struct StateRecord {
    name: String,
    capital: String,
    admitted: String,
    order: u8,
    population: u32,
    latitude: String,
    longitude: String,
    motto: String,
    description: String,
    midlat: f64,
    midlon: f64,
}

fn parse<T: DeserializeOwned>(data: &[u8]) -> Vec<T> {
    Reader::from_reader(data)
        .deserialize()
        .collect::<Result<Vec<_>, _>>()
        .expect("Malformed CSV data")
}

pub fn museum_objects() -> Vec<MuseumRecord> {
    parse(include_bytes!("data/100-objects.csv"))
}

pub fn us_states() -> Vec<StateRecord> {
    parse(include_bytes!("data/states.csv"))
}

pub fn seed_objects(path: impl Into<Option<PathBuf>>) -> WritableDatabase {
    let mut db = path.into().map_or_else(WritableDatabase::inmemory, |path| {
        WritableDatabase::open(path, None, None)
    });

    let mut indexer = TermGenerator::default();
    indexer.set_stemmer(Stem::for_language("english"));

    for item in museum_objects() {
        let mut doc = Document::default();
        indexer.set_document(&doc);

        indexer.index_text(&item.title, None, "S:");
        indexer.index_text(&item.description, None, "XD:");

        indexer.index_text::<&str>(&item.title, None, None);
        indexer.increase_termpos(None);
        indexer.index_text::<&str>(&item.description, None, None);

        let data = serde_json::to_string(&item).unwrap();
        doc.set_data(&data);

        if let Some(largest_measurement) = item
            .measurements
            .is_empty()
            .then(|| {
                item.measurements
                    .split(&[' ', '\t', '\n', '"'])
                    .filter_map(|s| s.parse::<f64>().ok().map(FloatOrd))
                    .max()
                    .map(|f| f.0)
            })
            .flatten()
        {
            doc.set_value(0, largest_measurement);
        }

        if let Some(year) = item
            .date_made
            .split(&[' ', '\t', '\n', '"', '-'])
            .filter_map(|s| s.parse::<i32>().ok())
            .next()
        {
            doc.set_value(1, year)
        }

        let idterm = format!("Q:{}", &item.id_number);
        doc.add_boolean_term(&idterm);

        db.replace_document_by_term(&idterm, &doc);
    }

    db
}

pub fn seed_states(path: impl Into<Option<PathBuf>>) -> WritableDatabase {
    let mut db = path.into().map_or_else(WritableDatabase::inmemory, |path| {
        WritableDatabase::open(path, None, None)
    });

    let mut indexer = TermGenerator::default();
    indexer.set_stemmer(Stem::for_language("english"));

    for item in us_states() {
        let mut doc = Document::default();
        indexer.set_document(&doc);

        indexer.index_text(&item.name, None, "S:");
        indexer.index_text(&item.description, None, "XD:");
        indexer.index_text(&item.motto, None, "XM:");

        indexer.index_text::<&str>(&item.name, None, None);
        indexer.increase_termpos(None);
        indexer.index_text::<&str>(&item.description, None, None);
        indexer.increase_termpos(None);
        indexer.index_text::<&str>(&item.motto, None, None);

        let admission_year = item.admitted[0..4].parse::<u16>().unwrap();
        doc.set_value(1, xapian_rs::ToValue::serialize(&admission_year));
        doc.set_value(2, &item.admitted);
        doc.set_value(3, xapian_rs::ToValue::serialize(&item.population));

        let idterm = format!("Q:{}", &item.order);
        doc.add_boolean_term(&idterm);

        db.replace_document_by_term(&idterm, &doc);
    }

    db
}
