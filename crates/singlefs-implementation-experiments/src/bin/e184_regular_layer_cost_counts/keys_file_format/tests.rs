use super::*;
use crate::helpers_shared_by_the_tests::{block_of, row_of};

#[test]
fn the_flow_row_and_the_keys_file_read_back_what_was_written() {
    let mut row = row_of(
        "random_history-broad-FourGibibytes-steps0100-seed03",
        "random_history",
        "candidate",
        2262,
        Some(1 << 30),
        30,
    );
    row.seed = Some(SEED_BASE_OF_THE_RANDOM_HISTORIES + 3);
    row.ending = "KnownRed:form=7".to_string();
    assert_eq!(FlowRow::from_body(&row.body()), Ok(row.clone()));
    let beyond = FlowRow {
        two_state: None,
        full: None,
        in_place_subsets: None,
        blocks_full: None,
        blocks_in_place_subsets: None,
        points: None,
        two_state_agrees_with_crates: None,
        ..row.clone()
    };
    assert_eq!(FlowRow::from_body(&beyond.body()), Ok(beyond.clone()));
    assert!(beyond
        .body()
        .contains("two_state=beyond_u64 log2_two_state=beyond_u64"));
    assert_eq!(
        beyond.group(),
        "random_history-broad-FourGibibytes-steps0100"
    );
    let blocks = vec![
        block_of(1, 2, 0, 65_536, 0),
        block_of(1, 2, 65_536, 7, 0),
        block_of(9, 8, 0, 1, 1),
    ];
    let file = KeysFile {
        expansion: Expansion::InPlaceSubsets,
        row,
        blocks,
    };
    let text = keys_file_text(&file.row, file.expansion, &file.blocks);
    assert_eq!(KeysFile::from_text(&text), Ok(file.clone()));
    assert_eq!(file.state_total(), 65_544);
    let without_trailer = text
        .lines()
        .take(text.lines().count() - 1)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(KeysFile::from_text(&without_trailer)
        .expect_err("没写完")
        .contains("keys_done"));
    let wrong_count = text.replace("# keys_done blocks=3", "# keys_done blocks=4");
    assert!(KeysFile::from_text(&wrong_count)
        .expect_err("块数对不上")
        .contains("实际读到 3"));
}
