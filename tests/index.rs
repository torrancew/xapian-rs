mod common;

#[test]
fn index_smoketest() {
    let museum_db = common::seed_objects(None);
    assert_eq!(museum_db.doc_count(), 100);

    let state_db = common::seed_states(None);
    assert_eq!(state_db.doc_count(), 50);
}
