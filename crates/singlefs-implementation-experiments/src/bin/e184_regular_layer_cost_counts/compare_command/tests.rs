use super::*;
use crate::helpers_shared_by_the_tests::{
    block_of, keys_file_of, row_of, scratch_directory, tiny_dump, tiny_write,
};

#[test]
fn the_invalidated_blocks_are_the_after_blocks_missing_in_the_before_flow() {
    let before = vec![
        block_of(1, 1, 0, 10, 0),
        block_of(2, 2, 0, 20, 1),
        block_of(3, 3, 0, 30, 2),
        block_of(4, 4, 0, 40, 3),
    ];
    let after = vec![
        block_of(1, 1, 0, 10, 0),
        block_of(2, 2, 0, 20, 1),
        block_of(3, 3, 0, 30, 2),
        block_of(5, 5, 0, 50, 3),
        block_of(6, 6, 0, 60, 4),
        block_of(7, 7, 0, 70, 5),
    ];
    let invalidated = invalidated_blocks(&before, &after);
    assert_eq!(invalidated.len(), 3, "after 的 6 块里有 3 块 before 没有");
    assert_eq!(
        invalidated.iter().map(|block| block.states).sum::<u64>(),
        180
    );
    assert_eq!(
        invalidated_blocks(&after, &before).len(),
        1,
        "反过来数只有 1 块（before 里的第四块）：方向不能弄反"
    );
}

#[test]
fn the_first_difference_is_the_base_or_the_first_write_that_differs_or_the_shorter_length() {
    let before = tiny_dump("flow", 2);
    assert_eq!(
        first_difference(&before, &tiny_dump("flow", 2)),
        FirstDifference::Identical
    );
    assert_eq!(
        first_difference(&before, &tiny_dump("flow", 9)),
        FirstDifference::Write(1)
    );
    let mut longer = tiny_dump("flow", 2);
    longer
        .writes
        .push(tiny_write(0, StepKind::UnitWrite, 12_288, 4));
    assert_eq!(
        first_difference(&before, &longer),
        FirstDifference::Write(3),
        "后一份多写一次：第一处不同是较短那份的写数"
    );
    let mut other_base = tiny_dump("flow", 9);
    other_base.base_writes[0] = tiny_write(0, StepKind::UnitWrite, 0, 0x22);
    assert_eq!(
        first_difference(&before, &other_base),
        FirstDifference::Base,
        "基镜像不同时先报 base，哪怕写表也不同"
    );
    assert_eq!(FirstDifference::Write(7).text(), "write:7");
    assert_eq!(FirstDifference::Base.text(), "base");
    assert_eq!(FirstDifference::Identical.text(), "none");
}

#[test]
fn write_index_is_placed_in_its_segment_and_in_the_crash_point_of_the_enumeration_cases() {
    let dump = tiny_dump("flow", 2);
    assert_eq!(segment_of_write(&dump.segments, 0), Some(0));
    assert_eq!(segment_of_write(&dump.segments, 1), Some(0));
    assert_eq!(segment_of_write(&dump.segments, 2), Some(1));
    assert_eq!(segment_of_write(&dump.segments, 3), None);
    assert_eq!(
        point_of_write(&dump, 1),
        Some(0),
        "写 0、1 是第一步，写 2 是第二步"
    );
    assert_eq!(point_of_write(&dump, 2), Some(1));
    assert_eq!(point_of_write(&dump, 3), None);
}

#[test]
fn compared_flow_reports_where_it_first_differs_and_how_many_blocks_and_states_it_invalidates() {
    let before_dump = tiny_dump("flow", 2);
    let after_dump = tiny_dump("flow", 9);
    let row = row_of("flow", "random_history", "candidate", 3, Some(12), 2);
    let before_keys = keys_file_of(
        row.clone(),
        vec![block_of(1, 1, 0, 10, 0), block_of(2, 2, 0, 20, 1)],
    );
    let after_keys = keys_file_of(
        row,
        vec![
            block_of(1, 1, 0, 10, 0),
            block_of(5, 5, 0, 20, 1),
            block_of(6, 6, 0, 30, 1),
        ],
    );
    let compared = compare_one_flow(&before_dump, &after_dump, &before_keys, &after_keys);
    assert_eq!(
        (
            compared.first_difference,
            compared.segment,
            compared.point,
            compared.first_journal_record_write
        ),
        (FirstDifference::Write(1), Some(0), Some(0), Some(1))
    );
    assert_eq!(
        (
            compared.blocks_before,
            compared.blocks_after,
            compared.invalidated_blocks
        ),
        (2, 3, 2)
    );
    assert_eq!(
        (compared.states_after, compared.invalidated_states),
        (60, 50)
    );
    assert!(
        (compared.block_fraction() - 2.0 / 3.0).abs() < 1e-12
            && (compared.state_fraction() - 50.0 / 60.0).abs() < 1e-12
    );
    assert!(
        ExpectedFirstDifference::JournalRecord.holds_for(&compared),
        "第一处不同的写就是第一条 journal 记录写"
    );
    assert!(!ExpectedFirstDifference::Identical.holds_for(&compared));
    let unchanged = compare_one_flow(
        &before_dump,
        &tiny_dump("flow", 2),
        &before_keys,
        &before_keys,
    );
    assert!(
        ExpectedFirstDifference::Identical.holds_for(&unchanged)
            && !ExpectedFirstDifference::JournalRecord.holds_for(&unchanged)
    );
    let summary = compare_summary_body(
        "patch",
        "journal-mac",
        "g",
        &[&compared, &unchanged],
        1,
        "unavailable",
    );
    assert!(
        summary.contains("flows_compared=2 flows_unavailable=1 flows_with_a_difference=1 blocks_after=5 invalidated_blocks=2 invalidated_block_fraction=0.400000"),
        "{summary}"
    );
}

fn write_flow_files(directory: &Path, dump: &FlowDump, blocks: Vec<BlockOfAPlan>) {
    dump.write_atomically(&directory.join(format!("{}{DUMP_FILE_SUFFIX}", dump.label)))
        .expect("写转储");
    let row = row_of(
        &dump.label,
        &dump.family,
        &dump.tier,
        dump.writes.len(),
        Some(12),
        2,
    );
    let text = keys_file_text(&row, Expansion::Full, &blocks);
    std::fs::write(
        directory.join(format!("{}{FULL_KEYS_FILE_SUFFIX}", dump.label)),
        text,
    )
    .expect("写块键文件");
}

#[test]
fn the_compare_command_pairs_flows_by_label_writes_the_lines_and_checks_the_expected_first_difference(
) {
    let (before_directory, after_directory) = (
        scratch_directory("compare-before"),
        scratch_directory("compare-after"),
    );
    write_flow_files(
        &before_directory,
        &tiny_dump("flow-a", 2),
        vec![block_of(1, 1, 0, 10, 0), block_of(2, 2, 0, 20, 1)],
    );
    write_flow_files(
        &after_directory,
        &tiny_dump("flow-a", 9),
        vec![block_of(1, 1, 0, 10, 0), block_of(5, 5, 0, 20, 1)],
    );
    write_flow_files(
        &after_directory,
        &tiny_dump("flow-only-after", 2),
        vec![block_of(7, 7, 0, 5, 0)],
    );
    let out_file = after_directory.join("compare.out");
    let arguments: Vec<String> = [
        "--before",
        before_directory.to_str().expect("路径"),
        "--after",
        after_directory.to_str().expect("路径"),
        "--kind",
        "patch",
        "--pair",
        "journal-mac",
        "--expect-first-difference",
        "journal-record",
        "--out",
        out_file.to_str().expect("路径"),
    ]
    .iter()
    .map(|word| (*word).to_string())
    .collect();
    assert_eq!(run_compare_command(&arguments), EXIT_CODE_OK);
    let text = std::fs::read_to_string(&out_file).expect("读得到产物");
    assert!(
        text.contains(
            "name=compare flow=flow-a group=flow-a first_difference=write:1 segment=0 point=0"
        ),
        "{text}"
    );
    assert!(
        text.contains("invalidated_blocks=1 states_after=30 invalidated_states=20"),
        "{text}"
    );
    assert!(
        text.contains("name=compare_unavailable flow=flow-only-after reason=missing_on_one_side"),
        "{text}"
    );
    assert!(
        text.contains("name=control kind=expected_first_difference flow=flow-a passed=true"),
        "{text}"
    );
    assert!(text.contains("name=compare_summary kind=patch pair=journal-mac group=flow-a flows_compared=1 flows_unavailable=1"), "{text}");
    assert!(text.contains("name=compare_done kind=patch pair=journal-mac flows_compared=1 flows_unavailable=1 controls_failed=0"), "{text}");
    let wrong_expectation: Vec<String> = arguments
        .iter()
        .map(|word| {
            if word == "journal-record" {
                "none".to_string()
            } else {
                word.clone()
            }
        })
        .collect();
    assert_eq!(
        run_compare_command(&wrong_expectation),
        EXIT_CODE_POSITIVE_CONTROL_FAILED,
        "期望两份逐字相同而实际不同：阳性对照没过"
    );
}
