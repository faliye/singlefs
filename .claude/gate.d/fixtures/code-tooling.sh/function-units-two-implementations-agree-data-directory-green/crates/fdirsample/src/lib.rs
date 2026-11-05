use std::path::Path;

pub fn count_rules() -> usize {
    std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("rules")).expect("listable").count()
}

pub fn table() -> usize {
    include_str!("table.txt").len()
}
