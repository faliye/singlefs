use super::*;
use crate::helpers_shared_by_the_tests::{row_of, scratch_directory};

#[test]
fn the_family_summary_reports_spread_and_the_count_of_flows_with_a_thousand_writes() {
    let rows = vec![
        row_of(
            "random_history-broad-FourGibibytes-steps0100-seed00",
            "random_history",
            "candidate",
            900,
            Some(1 << 10),
            12,
        ),
        row_of(
            "random_history-broad-FourGibibytes-steps0100-seed01",
            "random_history",
            "candidate",
            1000,
            Some(1 << 20),
            28,
        ),
        row_of(
            "random_history-broad-FourGibibytes-steps0100-seed02",
            "random_history",
            "candidate",
            2000,
            None,
            40,
        ),
        row_of(
            "overwrite_chain-04_units_per_overwrite",
            "overwrite_chain",
            "candidate",
            1100,
            Some(1 << 30),
            28,
        ),
    ];
    let bodies = family_summary_bodies(&rows);
    assert_eq!(bodies.len(), 2, "两个（族，组）");
    let histories = bodies
        .iter()
        .find(|body| body.contains("family=random_history "))
        .expect("随机历史那一行");
    assert!(histories.contains("group=random_history-broad-FourGibibytes-steps0100 flows=3 flows_with_writes_at_least_1000=2 flows_beyond_u64=1"), "{histories}");
    assert!(
        histories.contains("writes=[minimum=900 median=1000 maximum=2000]")
            && histories.contains("longest_segment=[minimum=12 median=28 maximum=40]"),
        "{histories}"
    );
    assert!(
        histories.contains("log2_two_state=[minimum=10 median=15 maximum=20]"),
        "装不进 u64 的那条不进 log2 的统计：{histories}"
    );
}

#[test]
fn the_keys_command_writes_key_files_and_controls_for_a_dump_and_a_second_run_reuses_them() {
    let (dumps, keys) = (
        scratch_directory("keys-dumps"),
        scratch_directory("keys-out"),
    );
    let prepared = prepared_snapshot_creation_stream();
    let dump_path = dumps.join("smoke-snapshot_creation_stream.dump");
    prepared
        .dump()
        .write_atomically(&dump_path)
        .expect("写转储");
    let arguments: Vec<String> = [
        "--dump",
        dumps.to_str().expect("路径"),
        "--out",
        keys.to_str().expect("路径"),
        "--controls",
    ]
    .iter()
    .map(|word| (*word).to_string())
    .collect();
    assert_eq!(run_keys_command(&arguments), EXIT_CODE_OK);
    let full = KeysFile::from_text(
        &std::fs::read_to_string(keys.join("smoke-snapshot_creation_stream.keys"))
            .expect("全量块键文件"),
    )
    .expect("读得回");
    assert_eq!(
        (full.expansion, full.row.full, full.state_total()),
        (Expansion::Full, Some(16_842_815), 16_842_815)
    );
    let in_place_subsets = KeysFile::from_text(
        &std::fs::read_to_string(keys.join("smoke-snapshot_creation_stream.in_place_subsets.keys"))
            .expect("甲二块键文件"),
    )
    .expect("读得回");
    assert_eq!(
        (
            in_place_subsets.expansion,
            Some(in_place_subsets.state_total())
        ),
        (Expansion::InPlaceSubsets, full.row.in_place_subsets)
    );
    let controls_text =
        std::fs::read_to_string(keys.join("smoke-snapshot_creation_stream.controls"))
            .expect("对照文件");
    assert_eq!(
        control_lines_of(&controls_text).map(|lines| lines.len()),
        Some(4)
    );
    match keys_of_one_dump(&dump_path, &keys, true) {
        KeysOutcome::Done {
            reused,
            controls,
            row,
            ..
        } => {
            assert!(reused, "写完的块键文件与对照整份跳过");
            assert_eq!(controls.len(), 4);
            assert!(controls.iter().all(|(_, passed)| *passed));
            assert_eq!(row.label, "smoke-snapshot_creation_stream");
        }
        KeysOutcome::Failed { message, .. } => panic!("第二趟不该失败：{message}"),
    }
}
