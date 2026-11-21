use super::*;
use crate::helpers_shared_by_the_tests::{block_of, keys_file_of, line_with, row_of};

#[test]
fn the_share_verdict_has_inclusive_thresholds_and_a_partial_band() {
    assert_eq!(share_verdict(0.5, 1.0), ShareVerdict::Renamed);
    assert_eq!(share_verdict(0.49, 0.9), ShareVerdict::Partial);
    assert_eq!(share_verdict(0.0, 0.05), ShareVerdict::Independent);
    assert_eq!(share_verdict(0.0, 0.06), ShareVerdict::Partial);
    assert_eq!(share_verdict(0.2, 0.03), ShareVerdict::Independent);
}

#[test]
fn the_overlap_verdict_has_inclusive_thresholds_and_a_partial_band() {
    assert_eq!(overlap_verdict(0.5), OverlapVerdict::Overlap);
    assert_eq!(overlap_verdict(0.1), OverlapVerdict::Separate);
    assert_eq!(overlap_verdict(0.3), OverlapVerdict::Partial);
    assert_eq!(overlap_verdict(0.0), OverlapVerdict::Separate);
    assert_eq!(overlap_verdict(1.0), OverlapVerdict::Overlap);
}

#[test]
fn the_interval_of_the_smoke_layer_counts_both_ends_as_inside() {
    assert_eq!(
        place_against(Some(100), 100, 200),
        PlaceAgainstInterval::Inside,
        "恰好等于最小值算在里面"
    );
    assert_eq!(
        place_against(Some(200), 100, 200),
        PlaceAgainstInterval::Inside,
        "恰好等于最大值算在里面"
    );
    assert_eq!(
        place_against(Some(99), 100, 200),
        PlaceAgainstInterval::Below
    );
    assert_eq!(
        place_against(Some(201), 100, 200),
        PlaceAgainstInterval::Above
    );
    assert_eq!(
        place_against(None, 100, 200),
        PlaceAgainstInterval::Above,
        "装不进 u64 的比区间上端大"
    );
}

#[test]
fn candidate_blocks_all_inside_a_larger_comparison_set_share_all_of_them() {
    let candidate = vec![
        block_of(1, 1, 0, 10, 0),
        block_of(2, 2, 0, 20, 1),
        block_of(3, 3, 0, 30, 2),
    ];
    let mut against_blocks = candidate.clone();
    against_blocks.push(block_of(4, 4, 0, 5, 3));
    against_blocks.push(block_of(5, 5, 0, 5, 4));
    let against = block_keys_of(&against_blocks);
    assert_eq!(against.len(), 5);
    assert!(
        (block_share_fraction(&candidate, &against) - 1.0).abs() < f64::EPSILON,
        "分母是候选自己的块数 3，不是比较集的 5"
    );
    let partly = vec![
        block_of(1, 1, 0, 10, 0),
        block_of(9, 9, 0, 30, 1),
        block_of(8, 8, 0, 30, 2),
        block_of(7, 7, 0, 30, 3),
    ];
    assert!((block_share_fraction(&partly, &against) - 0.25).abs() < f64::EPSILON);
    assert!(
        (weighted_share_fraction(&partly, &against) - 10.0 / 100.0).abs() < f64::EPSILON,
        "共用块里的态数 10 ÷ 候选的态数 100"
    );
}

/// 比较集（冒烟层一条流，三块）和五条候选流：三条随机历史（共用 3 / 4 块、1 / 4 块、0 / 5 块）、一条随机历史写数不到 1000、一条覆盖写链。
fn share_fixture() -> (Vec<KeysFile>, Vec<KeysFile>) {
    let smoke = keys_file_of(
        row_of(
            "smoke-snapshot_creation_stream",
            "smoke",
            "baseline",
            62,
            Some(100),
            10,
        ),
        vec![
            block_of(1, 1, 0, 10, 0),
            block_of(2, 2, 0, 20, 1),
            block_of(3, 3, 0, 30, 2),
        ],
    );
    let history = |seed: u8, writes: usize, two_state: u64, blocks: Vec<BlockOfAPlan>| {
        keys_file_of(
            row_of(
                &format!("random_history-broad-FourGibibytes-steps0100-seed{seed:02}"),
                "random_history",
                "candidate",
                writes,
                Some(two_state),
                20,
            ),
            blocks,
        )
    };
    let candidates = vec![
        history(
            0,
            1500,
            50,
            vec![
                block_of(1, 1, 0, 10, 0),
                block_of(2, 2, 0, 20, 1),
                block_of(3, 3, 0, 30, 2),
            ],
        ),
        history(
            1,
            1500,
            100,
            vec![
                block_of(1, 1, 0, 10, 0),
                block_of(11, 11, 0, 10, 1),
                block_of(12, 12, 0, 10, 2),
                block_of(13, 13, 0, 10, 3),
            ],
        ),
        history(
            2,
            1500,
            5000,
            (0..5)
                .map(|index| block_of(20 + index, 20 + index, 0, 10, usize::from(index)))
                .collect(),
        ),
        history(3, 10, 5000, vec![block_of(30, 30, 0, 10, 0)]),
        keys_file_of(
            row_of(
                "overwrite_chain-04_units_per_overwrite",
                "overwrite_chain",
                "candidate",
                1200,
                Some(70),
                28,
            ),
            vec![block_of(40, 40, 0, 10, 0), block_of(41, 41, 0, 10, 1)],
        ),
    ];
    (candidates, vec![smoke])
}

#[test]
fn the_share_lines_report_each_flow_each_group_and_the_overlap_with_the_smoke_interval() {
    let (candidates, against) = share_fixture();
    let mut output = Output::new();
    assert_eq!(
        share_lines(
            candidates.iter().cloned().map(Ok),
            &against,
            &[],
            &mut output
        ),
        Ok(EXIT_CODE_OK)
    );
    let group = line_with(
        &output,
        "E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0100 ",
    );
    assert!(group.contains("flows=4 planned_flows=4"), "{group}");
    assert!(
        group.contains("median_share_fraction=0.125000 maximum_share_fraction=1.000000"),
        "四条流的占比 1、0.25、0、0：中位 0.125，最大 1：{group}"
    );
    assert!(group.ends_with("verdict=partial"), "{group}");
    let chain = line_with(
        &output,
        "E7RESULT name=share_group scope=family:overwrite_chain_main ",
    );
    assert!(
        chain.contains("median_share_fraction=0.000000 maximum_share_fraction=0.000000")
            && chain.ends_with("verdict=independent"),
        "{chain}"
    );
    let second = line_with(
        &output,
        "E7RESULT name=share flow=random_history-broad-FourGibibytes-steps0100-seed01 ",
    );
    assert!(second.contains("blocks=4 shared_blocks=1 share_fraction=0.250000 states=40 shared_states=10 weighted_fraction=0.250000 reuse_fraction=0.250000"), "{second}");
    let overlap = line_with(
        &output,
        "E7RESULT name=overlap scope=family:random_history_main ",
    );
    assert!(overlap.contains("smoke_interval_two_state_minimum=100 smoke_interval_two_state_maximum=100 smoke_interval_longest_segment_minimum=10 smoke_interval_longest_segment_maximum=10"), "{overlap}");
    assert!(
        overlap.contains("flows=4 flows_with_writes_at_least_1000=3 inside=1 above=1 below=1"),
        "{overlap}"
    );
    assert!(
        overlap.contains("inside_fraction=0.333333") && overlap.ends_with("verdict=partial"),
        "{overlap}"
    );
    assert!(
        !output
            .lines
            .iter()
            .any(|line| line.contains("passed=false")),
        "自比对照每条都过"
    );
    let closing = output.lines.last().expect("有收尾行");
    assert!(
        closing.contains("name=share_done candidates=5 against=1 controls_failed=0"),
        "{closing}"
    );
}

#[test]
fn the_overlap_counts_use_only_flows_with_a_thousand_writes_and_put_the_boundary_inside() {
    let interval = SmokeInterval {
        two_state_minimum: 100,
        two_state_maximum: 200,
        longest_segment_minimum: 10,
        longest_segment_maximum: 20,
    };
    let rows = [
        row_of("a", "random_history", "candidate", 1000, Some(100), 10),
        row_of("b", "random_history", "candidate", 999, Some(100), 10),
        row_of("c", "random_history", "candidate", 5000, None, 64),
        row_of("d", "random_history", "candidate", 5000, Some(200), 21),
        row_of("e", "random_history", "candidate", 5000, Some(99), 9),
    ];
    let members: Vec<&FlowRow> = rows.iter().collect();
    let counts = OverlapCounts::of(&members, &interval);
    assert_eq!(
        counts,
        OverlapCounts {
            flows: 5,
            flows_with_writes_at_least_1000: 4,
            inside: 2,
            above: 1,
            below: 1,
            longest_segment_inside: 1
        },
        "a 恰好 1000 写算进去，b 差 1 写不算；c 的态数装不进 u64 算在上面"
    );
    assert_eq!(counts.fraction_of(counts.inside), Some(0.5));
    assert_eq!(
        OverlapCounts::of(&[&rows[1]], &interval).fraction_of(0),
        None,
        "没有写数不少于 1000 的流时占比读不出，不当 0"
    );
    assert!(OverlapCounts::of(&[&rows[1]], &interval)
        .body("scope", &interval)
        .ends_with("verdict=no_flow_with_writes_at_least_1000"));
}

#[test]
fn the_smoke_interval_takes_the_recorded_flows_and_the_pinned_ones() {
    let recorded = [
        row_of("s1", "smoke", "baseline", 62, Some(16_842_782), 24),
        row_of("s2", "smoke", "baseline", 41, Some(16_777_240), 24),
    ];
    let pinned = parse_pinned_smoke(
        "# 注释\nlifecycle_closure 288751838 26\n\ncompaction_relocation 1377370253 30\n",
    )
    .expect("合法");
    assert_eq!(pinned.len(), 2);
    assert_eq!(
        smoke_interval(&recorded, &pinned),
        Ok(SmokeInterval {
            two_state_minimum: 16_777_240,
            two_state_maximum: 1_377_370_253,
            longest_segment_minimum: 24,
            longest_segment_maximum: 30
        })
    );
    assert!(
        smoke_interval(&[], &[]).is_err(),
        "一条冒烟流都没有，区间定不出"
    );
    assert!(parse_pinned_smoke("only_two_words 12").is_err());
}

#[test]
fn baseline_flow_among_the_candidates_is_ignored_and_a_candidate_among_the_comparison_set_is_dropped(
) {
    let (mut candidates, mut against) = share_fixture();
    let smoke_among_the_candidates = against[0].clone();
    candidates.push(smoke_among_the_candidates);
    let history_among_the_comparison_set = candidates[0].clone();
    against.push(history_among_the_comparison_set);
    let mut output = Output::new();
    assert_eq!(
        share_lines(
            candidates.iter().cloned().map(Ok),
            &against,
            &[],
            &mut output
        ),
        Ok(EXIT_CODE_OK)
    );
    assert!(line_with(
        &output,
        "E7RESULT name=share_ignored flow=smoke-snapshot_creation_stream "
    )
    .ends_with("reason=tier_is_baseline"));
    let closing = output.lines.last().expect("有收尾行");
    assert!(
        closing.contains("name=share_done candidates=5 against=1 controls_failed=0"),
        "候选还是 5 条、比较集还是 1 条：{closing}"
    );
    let first = line_with(
        &output,
        "E7RESULT name=share flow=random_history-broad-FourGibibytes-steps0100-seed00 ",
    );
    assert!(
        first.contains("shared_blocks=3 share_fraction=1.000000"),
        "{first}"
    );
}
