use super::*;
use crate::helpers_shared_by_the_tests::{block_of, keys_file_of, line_with, row_of};

#[test]
fn the_wall_clock_band_puts_the_two_boundaries_in_the_lower_band() {
    assert_eq!(
        band_of_seconds(3600.0),
        WallClockBand::WithinTwoHours,
        "一小时落在 2 小时以内"
    );
    assert_eq!(band_of_seconds(7200.0), WallClockBand::WithinTwoHours);
    assert_eq!(
        band_of_seconds(7200.5),
        WallClockBand::BetweenTwoHoursAndADay
    );
    assert_eq!(
        band_of_seconds(86_400.0),
        WallClockBand::BetweenTwoHoursAndADay
    );
    assert_eq!(band_of_seconds(86_400.5), WallClockBand::OverADay);
}

#[test]
fn the_commit_pairs_verdict_needs_ten_pairs_and_then_reads_the_median() {
    assert_eq!(
        commit_pairs_verdict(&[0.9; 9]),
        CommitPairsVerdict::StillMissing,
        "9 对不到 10 对"
    );
    assert_eq!(
        commit_pairs_verdict(&[0.5; 10]),
        CommitPairsVerdict::MoreThanHalfInvalidated
    );
    assert_eq!(
        commit_pairs_verdict(&[0.1; 10]),
        CommitPairsVerdict::MostlyReused
    );
    assert_eq!(
        commit_pairs_verdict(&[0.3; 10]),
        CommitPairsVerdict::Partial
    );
    assert_eq!(commit_pairs_verdict(&[]), CommitPairsVerdict::StillMissing);
}

#[test]
fn the_commit_pair_trajectory_counts_the_positive_pairs_and_keeps_peak_median_and_final() {
    let trajectory = commit_pair_trajectory(&[0.0, 0.2, 0.5], Some(2)).expect("三对");
    assert_eq!(trajectory.computable, 3);
    assert_eq!(trajectory.positive, 2, "作废 0 的那一对不算为正");
    assert!(
        (trajectory.peak - 0.5).abs() < f64::EPSILON
            && (trajectory.median - 0.2).abs() < f64::EPSILON
    );
    assert_eq!(trajectory.final_value, Some(0.5));
    assert_eq!(
        commit_pair_trajectory(&[0.0, 0.2], None)
            .expect("两对")
            .final_value,
        None
    );
    assert_eq!(commit_pair_trajectory(&[], None), None);
}

#[test]
fn the_budget_opens_points_in_ascending_state_order_until_the_next_one_no_longer_fits() {
    let points = [1000, 10, 30, 20];
    assert_eq!(
        open_under_budget(&points, 60.0, 1.0),
        OpenUnderBudget {
            points: 3,
            states: 60
        },
        "10 + 20 + 30 = 60 秒恰好开得下"
    );
    assert_eq!(
        open_under_budget(&points, 59.0, 1.0),
        OpenUnderBudget {
            points: 2,
            states: 30
        }
    );
    assert_eq!(
        open_under_budget(&points, 5.0, 1.0),
        OpenUnderBudget {
            points: 0,
            states: 0
        }
    );
    assert_eq!(
        open_under_budget(&points, 2000.0, 2.0),
        OpenUnderBudget {
            points: 4,
            states: 1060
        }
    );
}

#[test]
fn the_cumulative_curve_has_ten_ticks_over_the_points_in_ascending_order() {
    let points: Vec<u64> = (1..=10).rev().collect();
    let curve = cumulative_curve(&points, 1.0);
    let seconds: Vec<f64> = curve.iter().map(|(_, seconds)| *seconds).collect();
    assert_eq!(
        seconds,
        vec![1.0, 3.0, 6.0, 10.0, 15.0, 21.0, 28.0, 36.0, 45.0, 55.0]
    );
    assert!((curve[0].0 - 0.1).abs() < f64::EPSILON && (curve[9].0 - 1.0).abs() < f64::EPSILON);
}

#[test]
fn two_points_fit_a_fixed_cost_and_a_slope() {
    let (fixed, slope) = fit_through_two_points(100, 10.0, 300, 20.0).expect("两点上升");
    assert!(
        (slope - 20.0).abs() < 1e-9 && (fixed - 5.0).abs() < 1e-9,
        "斜率 (300 − 100) ÷ (20 − 10) = 20 态每秒，固定 10 − 100 ÷ 20 = 5：{fixed} {slope}"
    );
    assert_eq!(
        fit_through_two_points(100, 10.0, 300, 10.0),
        None,
        "秒数不升就拟合不出"
    );
    assert_eq!(
        fit_through_two_points(300, 10.0, 100, 20.0),
        None,
        "态数不升就拟合不出"
    );
}

fn cold_log(
    item: &str,
    states: u64,
    reused: u64,
    seconds: f64,
    compile: f64,
    judge: f64,
    storing: f64,
) -> String {
    format!(
        "some other line\nCRASH_AMPLIFICATION_GPU_TOTAL cards_used=2 blocks=1 states={states} compile_seconds={compile} judge_seconds={judge} states_per_second=1 seconds_storing={storing} expand_up_to=6 exhaustive=true\nCRASH_AMPLIFICATION_ITEM item={item} exit=0 states={states} reused={reused} red=0 lines=1 expand_up_to=6 exhaustive=true verdict=ok seconds={seconds} ignored_points=0\n"
    )
}

#[test]
fn cold_run_log_gives_the_state_count_the_seconds_and_the_two_throughputs() {
    let run = ColdRun::from_log(&cold_log(
        "snapshot-creation",
        16_842_807,
        0,
        2000.0,
        100.0,
        400.0,
        100.0,
    ))
    .expect("日志读得出");
    assert_eq!(
        (run.item.as_str(), run.states, run.reused, run.exit_code),
        ("snapshot-creation", 16_842_807, 0, 0)
    );
    assert!(
        (run.recording_seconds() - 1400.0).abs() < 1e-9,
        "2000 − 100 − 400 − 100"
    );
    assert!(run.outer_seconds_cover_the_inner_ones());
    assert!(
        (run.rate_total() - 16_842_807.0 / 2000.0).abs() < 1e-6
            && (run.rate_judge() - 16_842_807.0 / 400.0).abs() < 1e-6
    );
    assert!(ColdRun::from_log(
        "CRASH_AMPLIFICATION_ITEM item=x exit=0 states=1 reused=0 seconds=1"
    )
    .expect_err("没有判段行")
    .contains("GPU_TOTAL"));
    assert!(ColdRun::from_log("nothing")
        .expect_err("没有项行")
        .contains("恰好一行"));
    let two_hosts = format!("{}{}", cold_log("x", 10, 0, 100.0, 10.0, 40.0, 5.0), "CRASH_AMPLIFICATION_GPU_TOTAL cards_used=1 blocks=1 states=10 compile_seconds=12.0 judge_seconds=30.0 states_per_second=1 seconds_storing=7.0 expand_up_to=6 exhaustive=true\n");
    let merged = ColdRun::from_log(&two_hosts).expect("两台");
    assert_eq!(
        (
            merged.gpu_total_lines,
            merged.compile_seconds,
            merged.judge_seconds,
            merged.seconds_storing
        ),
        (2, 12.0, 40.0, 7.0),
        "两台同时判，各段取最大"
    );
}

#[test]
fn cold_run_that_is_not_cold_or_not_monotone_or_not_the_pinned_item_is_void() {
    let small = ColdRun::from_log(&cold_log(
        "snapshot-creation",
        16_842_807,
        0,
        2000.0,
        100.0,
        400.0,
        100.0,
    ))
    .expect("小项");
    let medium = ColdRun::from_log(&cold_log(
        "overwrite-and-unlink-under-a-snapshot",
        100_000_000,
        0,
        10_000.0,
        100.0,
        5000.0,
        400.0,
    ))
    .expect("中项");
    assert_eq!(voided_cold_runs(&small, &medium), Vec::<String>::new());
    let warm = ColdRun {
        reused: 5,
        ..medium.clone()
    };
    assert!(voided_cold_runs(&small, &warm)
        .iter()
        .any(|reason| reason.contains("库不冷")));
    let not_rising = ColdRun {
        seconds: 1999.0,
        ..medium.clone()
    };
    assert!(voided_cold_runs(&small, &not_rising)
        .iter()
        .any(|reason| reason.contains("分不出规模")));
    let not_covering = ColdRun {
        compile_seconds: 6000.0,
        ..medium.clone()
    };
    assert!(voided_cold_runs(&small, &not_covering)
        .iter()
        .any(|reason| reason.contains("包不住")));
    let another_item = ColdRun {
        states: 16_842_808,
        ..small.clone()
    };
    assert!(voided_cold_runs(&another_item, &medium)
        .iter()
        .any(|reason| reason.contains("不是 16842807")));
}

#[test]
fn the_library_bytes_and_recorded_blocks_are_read_by_the_end_of_the_path() {
    let du = "168428070\t/work/e184/library-small\n1000000000\t/work/e184/library-medium\n";
    assert_eq!(library_bytes_of(du, "library-small"), Ok(168_428_070));
    assert_eq!(library_bytes_of(du, "library-medium"), Ok(1_000_000_000));
    assert!(library_bytes_of(du, "library-large").is_err());
    let reader = "CRASH_LIBRARY_KEYS\tlibrary=/work/e184/library-small\trecorded_blocks=257\tjudged_under_the_current_key=0\n\
                      CRASH_LIBRARY_KEYS\tlibrary=/work/e184/library-medium\trecorded_blocks=2561\tjudged_under_the_current_key=0\n";
    assert_eq!(recorded_blocks_of(reader, "library-small"), Ok(257));
    assert_eq!(recorded_blocks_of(reader, "library-medium"), Ok(2561));
    assert!(
        recorded_blocks_of(
            "CRASH_LIBRARY_KEYS\tlibrary=/x/library-small\tmissing\n",
            "library-small"
        )
        .is_err(),
        "读不到不当 0"
    );
}

fn cell_seconds(bodies: &[String], candidate: &str, commit_class: &str) -> f64 {
    let prefix = format!("name=compose_cell candidate={candidate} commit_class={commit_class} ");
    let body = bodies
        .iter()
        .find(|body| body.starts_with(&prefix))
        .unwrap_or_else(|| panic!("没有 {prefix:?} 的格子：{bodies:?}"));
    fields_of(body)["seconds"]
        .parse::<f64>()
        .expect("seconds 是数")
}

#[test]
fn the_cells_judge_the_invalidated_states_at_the_judge_throughput_not_at_the_total_one() {
    let inputs = ComposeInputs {
        keys_seconds_total: 30.0,
        full_total: 4_000_000,
        in_place_subsets_total: 400_000,
        point_states: vec![600_000, 400_000, 1_000_000, 2_000_000],
        write_path_median_fraction: Some(0.5),
        write_path_upper_bound_fraction: None,
    };
    let bodies = cell_bodies(
        &inputs,
        "medium",
        &Rates {
            total: 1000.0,
            judge: 4000.0,
        },
    );
    assert!(
        (cell_seconds(
            &bodies,
            "every_commit_judges_the_full_domain_by_block_keys",
            "write_path_median"
        ) - (30.0 + 0.5 * 4_000_000.0 / 4000.0))
            .abs()
            < 1e-9,
        "30 + 500 = 530 秒；按总吞吐算会是 2030"
    );
    assert!(
        (cell_seconds(
            &bodies,
            "commit_judges_the_block_reuse_domain_only",
            "write_path_median"
        ) - (30.0 + 0.5 * 400_000.0 / 4000.0))
            .abs()
            < 1e-9
    );
    assert!(
        (cell_seconds(
            &bodies,
            "commit_queries_the_library_only",
            "write_path_median"
        ) - 30.0)
            .abs()
            < 1e-9,
        "丙只查库"
    );
    assert!(
        (cell_seconds(
            &bodies,
            "every_commit_judges_the_full_domain_by_block_keys",
            "judge_only"
        ) - (30.0 + 4_000_000.0 / 4000.0))
            .abs()
            < 1e-9,
        "只动判器的提交作废占比取 1"
    );
    assert!(
        bodies
            .iter()
            .any(|body| body.contains("commit_class=write_path_upper_bound")
                && body.contains("unavailable=no_invalidated_fraction")),
        "补丁那一格没有占比时写取不出，不当 0"
    );
    let full_by_the_main_agent = bodies
        .iter()
        .find(|body| body.contains("commit_queries_the_library_only_full_by_the_main_agent"))
        .expect("丙另列一格全量");
    assert!(
        full_by_the_main_agent.contains("seconds=4000.0"),
        "4,000,000 ÷ 1000 = 4000 秒：{full_by_the_main_agent}"
    );
    assert!(
        full_by_the_main_agent.contains("band=within_2h"),
        "{full_by_the_main_agent}"
    );
    assert_eq!(
        bodies
            .iter()
            .filter(|body| body.starts_with("name=compose_curve "))
            .count(),
        10
    );
    let two_hours = bodies
        .iter()
        .find(|body| body.contains("budget_hours=2 "))
        .expect("2 小时预算");
    assert!(
        two_hours.contains("open_points=4") && two_hours.contains("open_point_fraction=1.000000"),
        "4000 态每秒 × 7200 秒 = 28,800,000 ≥ 4,000,000：{two_hours}"
    );
}

fn compose_fixture() -> (ComposeTexts, Vec<KeysFile>) {
    let history = FlowRow {
        keys_seconds: 10.0,
        full: Some(1_000_000),
        in_place_subsets: Some(100_000),
        ..row_of(
            "random_history-broad-FourGibibytes-steps0100-seed00",
            "random_history",
            "candidate",
            2262,
            Some(900_000),
            30,
        )
    };
    let chain = FlowRow {
        keys_seconds: 20.0,
        full: Some(3_000_000),
        in_place_subsets: Some(300_000),
        ..row_of(
            "overwrite_chain-04_units_per_overwrite",
            "overwrite_chain",
            "candidate",
            1200,
            Some(2_900_000),
            28,
        )
    };
    let snapshot = FlowRow {
        full: Some(16_842_807),
        ..row_of(
            "smoke-snapshot_creation_stream",
            "smoke",
            "baseline",
            62,
            Some(16_842_782),
            24,
        )
    };
    let keys_output: String = [&history, &chain, &snapshot]
        .iter()
        .map(|row| format!("E7RESULT {} reused=false\n", row.body()))
        .collect();
    let keys_files = vec![
        keys_file_of(
            history,
            vec![block_of(1, 1, 0, 600_000, 0), block_of(2, 2, 0, 400_000, 1)],
        ),
        keys_file_of(
            chain,
            vec![
                block_of(3, 3, 0, 1_000_000, 0),
                block_of(4, 4, 0, 2_000_000, 1),
            ],
        ),
    ];
    let mut compare_output = String::new();
    for pair_number in 1..=10u64 {
        let pair = ComparedFlow {
            label: format!(
                "random_history-broad-FourGibibytes-steps0100-seed00-of-pair{pair_number}"
            ),
            group: "random_history-broad-FourGibibytes-steps0100".to_string(),
            first_difference: FirstDifference::Write(3),
            segment: Some(1),
            point: Some(1),
            first_journal_record_write: Some(3),
            blocks_before: 100,
            blocks_after: 100,
            invalidated_blocks: 5,
            states_after: 1000,
            invalidated_states: pair_number * 100,
        };
        compare_output.push_str(&format!(
            "E7RESULT {}\n",
            compare_summary_body(
                "commit-pair",
                &format!("pair{pair_number:02}"),
                &pair.group,
                &[&pair],
                0,
                "no"
            )
        ));
    }
    let patch = |group: &str, invalidated_states: u64| ComparedFlow {
        label: format!("{group}-seed00"),
        group: group.to_string(),
        first_difference: FirstDifference::Base,
        segment: None,
        point: None,
        first_journal_record_write: None,
        blocks_before: 10,
        blocks_after: 10,
        invalidated_blocks: 10,
        states_after: 1000,
        invalidated_states,
    };
    for (group, invalidated_states) in [
        ("random_history-broad-FourGibibytes-steps0100", 800),
        ("overwrite_chain-04_units_per_overwrite", 400),
        (
            "random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100",
            0,
        ),
    ] {
        compare_output.push_str(&format!(
            "E7RESULT {}\n",
            compare_summary_body(
                "patch",
                "journal-mac",
                group,
                &[&patch(group, invalidated_states)],
                0,
                "unavailable"
            )
        ));
    }
    let texts = ComposeTexts {
        keys_output,
        compare_output,
        small_log: cold_log("snapshot-creation", 16_842_807, 0, 2000.0, 100.0, 400.0, 100.0),
        medium_log: cold_log("overwrite-and-unlink-under-a-snapshot", 100_000_000, 0, 10_000.0, 100.0, 5000.0, 400.0),
        library_bytes: "168428070\t/work/e184/library-small\n1000000000\t/work/e184/library-medium\n".to_string(),
        recorded_blocks: "CRASH_LIBRARY_KEYS\tlibrary=/work/e184/library-small\trecorded_blocks=257\nCRASH_LIBRARY_KEYS\tlibrary=/work/e184/library-medium\trecorded_blocks=2561\n".to_string(),
        final_pair: Some("pair10".to_string()),
    };
    (texts, keys_files)
}

#[test]
fn the_compose_lines_give_the_pinned_cells_for_a_hand_made_load() {
    let (texts, keys_files) = compose_fixture();
    let mut output = Output::new();
    assert_eq!(
        compose_lines(&texts, &keys_files, &mut output),
        Ok(EXIT_CODE_OK)
    );
    assert!(line_with(
        &output,
        "E7RESULT name=stop_check item=small_item_states_match_the_recorded_snapshot_full "
    )
    .contains("result=pass small_item_states=16842807"));
    let small = line_with(&output, "E7RESULT name=compose_rate rate_source=small ");
    assert!(
        small.contains("states_per_second_of_judging=42107.0")
            && small.contains("library_bytes_per_state=10.0000"),
        "{small}"
    );
    let medium = line_with(&output, "E7RESULT name=compose_rate rate_source=medium ");
    assert!(
        medium.contains("states_per_second_of_recording_and_judging=10000.0 states_per_second_of_judging=20000.0")
            && medium.contains("recording_seconds=4500.0"),
        "{medium}"
    );
    assert!(line_with(
        &output,
        "E7RESULT name=anchor identifier=small_item_states "
    )
    .ends_with("expected=16842807 actual=16842807 match=true"));
    assert!(
        line_with(
            &output,
            "E7RESULT name=anchor identifier=medium_item_states "
        )
        .ends_with("expected=167837792 actual=100000000 match=false"),
        "中项的态数与测试文件钉的不符只报不符，不作废、不改退出码"
    );
    let input = line_with(&output, "E7RESULT name=compose_input ");
    assert!(input.contains("flows_in_the_nominal_load=2 keys_seconds_total=30.0 full_total=4000000 in_place_subsets_total=400000 points_total=4"), "{input}");
    let by_states = line_with(
        &output,
        "E7RESULT name=commit_pairs_trajectory basis=states ",
    );
    assert!(by_states.contains("computable=10 positive=10 peak=1.000000 median=0.550000 final=1.000000 verdict=more_than_half_invalidated"), "{by_states}");
    let by_blocks = line_with(
        &output,
        "E7RESULT name=commit_pairs_trajectory basis=blocks ",
    );
    assert!(
        by_blocks.contains("median=0.050000 final=0.050000 verdict=mostly_reused"),
        "按块与按态数各判一次门槛：{by_blocks}"
    );
    let medium_cells: Vec<String> = output
        .lines
        .iter()
        .filter(|line| line.contains("rate_source=medium"))
        .map(|line| line.trim_start_matches("E7RESULT ").to_string())
        .collect();
    for (candidate, class, seconds) in [
        (
            "every_commit_judges_the_full_domain_by_block_keys",
            "write_path_median",
            140.0,
        ),
        (
            "every_commit_judges_the_full_domain_by_block_keys",
            "write_path_upper_bound",
            150.0,
        ),
        (
            "every_commit_judges_the_full_domain_by_block_keys",
            "judge_only",
            230.0,
        ),
        (
            "commit_judges_the_block_reuse_domain_only",
            "write_path_median",
            41.0,
        ),
        ("commit_queries_the_library_only", "write_path_median", 30.0),
    ] {
        assert!(
            (cell_seconds(&medium_cells, candidate, class) - seconds).abs() < 1e-6,
            "{candidate} {class} 应是 {seconds} 秒"
        );
    }
    assert!(medium_cells.iter().any(|body| body
        .contains("commit_queries_the_library_only_full_by_the_main_agent")
        && body.contains("seconds=400.0")));
    assert!(output
        .lines
        .last()
        .expect("有收尾行")
        .contains("name=compose_done commit_pairs_computable=10"));
}

#[test]
fn warm_library_voids_the_cold_runs_and_a_small_item_other_than_the_recorded_snapshot_stops_the_compose(
) {
    let (mut texts, keys_files) = compose_fixture();
    texts.medium_log = cold_log(
        "overwrite-and-unlink-under-a-snapshot",
        100_000_000,
        3,
        10_000.0,
        100.0,
        5000.0,
        400.0,
    );
    let mut output = Output::new();
    assert_eq!(
        compose_lines(&texts, &keys_files, &mut output),
        Ok(EXIT_CODE_POSITIVE_CONTROL_FAILED)
    );
    assert!(line_with(&output, "E7RESULT name=void item=cold_library_runs ").contains("库不冷"));
    assert!(
        !output
            .lines
            .iter()
            .any(|line| line.contains("name=compose_cell")),
        "作废之后不出格子"
    );
    let (mut other_texts, other_keys) = compose_fixture();
    other_texts.small_log = cold_log(
        "snapshot-creation",
        16_842_808,
        0,
        2000.0,
        100.0,
        400.0,
        100.0,
    );
    let mut stopped = Output::new();
    assert_eq!(
        compose_lines(&other_texts, &other_keys, &mut stopped),
        Ok(EXIT_CODE_STOP_CLAUSE_FAILED)
    );
    assert!(line_with(
        &stopped,
        "E7RESULT name=stop_check item=small_item_states_match_the_recorded_snapshot_full "
    )
    .contains("result=fail"));
}

#[test]
fn the_nominal_load_is_the_hundred_step_histories_and_the_overwrite_chains_only() {
    let in_the_load = |label: &str| {
        is_in_the_nominal_regular_layer_load(&row_of(label, "x", "candidate", 1, Some(1), 1))
    };
    assert!(in_the_load(
        "random_history-broad-FourGibibytes-steps0100-seed19"
    ));
    assert!(in_the_load("overwrite_chain-16_units_per_overwrite"));
    assert!(!in_the_load(
        "random_history-broad-FourGibibytes-steps0050-seed00"
    ));
    assert!(!in_the_load(
        "random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100-seed00"
    ));
    assert!(!in_the_load("smoke-snapshot_creation_stream"));
}
