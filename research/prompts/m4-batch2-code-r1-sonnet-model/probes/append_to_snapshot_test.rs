
#[test]
fn probe_five_readers_on_a_valid_two_pair_table() {
    let mut version = version_with_snapshot_pairs(2);
    let record = version.first.record.clone();
    let snapshot_entries = version.entries_of_every_pair();
    let root = version.root_carrying(&snapshot_entries);
    let outcomes = outcomes_of_every_tree_table_reader(&version.image, &root, &record);
    println!("PROBE_P1 valid_two_pairs: {:?}", outcomes);
}

#[test]
fn probe_five_readers_when_there_is_one_more_snapshot_inode_root_than_extent_roots() {
    let mut version = version_with_snapshot_pairs(3);
    let record = version.first.record.clone();
    let mut snapshot_entries = version.entries_of_every_pair();
    snapshot_entries.pop();
    let root = version.root_carrying(&snapshot_entries);
    let outcomes = outcomes_of_every_tree_table_reader(&version.image, &root, &record);
    println!("PROBE_P2 more_inode_roots: {:?}", outcomes);
}
