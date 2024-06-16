#[path = "../tests/common.rs"]
mod common;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct Args {
    db: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    common::seed_objects(args.db.join("museum"));
    common::seed_states(args.db.join("states"));

    Ok(())
}
