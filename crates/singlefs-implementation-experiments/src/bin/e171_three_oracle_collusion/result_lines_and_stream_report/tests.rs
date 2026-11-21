//! 结果行（`E7RESULT`、`E7TIMING`）、与流无关的核对和一条流的报告，连同不适用的来源核对的单测。

use super::*;

/// 场景：S1 里与流无关的那几条（盘宽、物理块与 io_min、扇区宽、固定 fsid、固定写时刻、第一个文件的字节数与内容），本地常量对验证包库的值。
/// 预期：每一条都成立，至少比了一条。
/// 判别力（`crates/mutations.tsv`）：本地的盘字节数多 512，这条红。
#[test]
fn stream_independent_stop_checks_hold_against_the_verification_library() {
    let checks = stream_independent_checks();
    assert!(!checks.is_empty(), "S1 里与流无关的回比至少一条");
    let broken: Vec<&StopCheck> = checks.iter().filter(|check| !check.holds()).collect();
    assert!(broken.is_empty(), "S1 里与流无关的这几条没过：{broken:?}");
}

#[test]
fn timing_summary_takes_the_lower_middle_and_the_ninety_fifth_percentile() {
    assert_eq!(timing_summary(vec![]), (0, 0, 0, 0));
    assert_eq!(timing_summary(vec![4, 1, 3, 2]), (10, 2, 4, 4));
    let hundred: Vec<u128> = (1..=100).collect();
    assert_eq!(timing_summary(hundred), (5050, 50, 95, 100));
}

fn recorded_call_for_the_test(
    label: &'static str,
    outcome: CallOutcome,
    model: ModelVerdictOfCall,
) -> RecordedCall {
    RecordedCall {
        label: label.to_string(),
        first_operation: 0,
        return_point: 0,
        outcome,
        model,
    }
}

fn disagreement_for_the_test() -> ModelVerdictOfCall {
    ModelVerdictOfCall::Disagrees {
        aspect: "模型说该成、实现拒了".to_string(),
        model_answer: "往前走".to_string(),
        implementation_answer: "拒了:X".to_string(),
    }
}

/// 逐行那一趟的 S5：返回了错误而 O1-对拍没问模型（`not_modelled`）的调用、与不适用来自卸载和对拍不同之外的来源，各自回比 0，
/// 不是 0 就是停机。绝对值：期望写 `0`，实际写不合的个数。
#[test]
fn the_row_not_applicable_source_checks_count_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source(
) {
    let errored = |text: &str| CallOutcome::ReturnedError(text.to_string());
    // 取号出错、O1-对拍没记红（修之前的样子）：第一样 1 处不合；模型停在这次调用、来源不是卸载也不是对拍不同，但没有不适用的态，第二样 0。
    let silent = [recorded_call_for_the_test(
        "acquire",
        errored("X"),
        ModelVerdictOfCall::NotModelled,
    )];
    let silent_acquire_checks =
        not_applicable_source_checks_of_a_bad_implementation_row("s", &silent, Some(0), 0);
    assert_eq!(
        silent_acquire_checks
            .iter()
            .map(|check| check.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "s_o1_every_errored_call_but_the_unmount_is_judged_by_the_comparison",
            "s_o1_not_applicable_only_after_the_unmount_or_a_disagreement"
        ]
    );
    assert_eq!(
        silent_acquire_checks
            .iter()
            .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
            .collect::<Vec<_>>(),
        vec![("0", "1", false), ("0", "0", true)]
    );
    // 取号出错、记成对拍的红：两样都 0，不适用的态来自对拍不同，合规。
    let red = [recorded_call_for_the_test(
        "acquire",
        errored("X"),
        disagreement_for_the_test(),
    )];
    let recorded_red_checks =
        not_applicable_source_checks_of_a_bad_implementation_row("s", &red, Some(0), 9);
    assert!(recorded_red_checks.iter().all(StopCheck::holds));
    // 对拍在发布上判出不同、模型停在那里，之后的可写挂载出错（`skipped_after_the_model_stopped`，第 108 行那一形）：登记读法 6 的合规来源，
    // 两样都 0（登记修订 4）。
    let error_after_the_stop = [
        recorded_call_for_the_test(
            "publish",
            CallOutcome::Succeeded,
            disagreement_for_the_test(),
        ),
        recorded_call_for_the_test(
            "mount_writable",
            errored("Recovery"),
            ModelVerdictOfCall::SkippedAfterTheModelStopped,
        ),
    ];
    let error_after_the_stop_checks = not_applicable_source_checks_of_a_bad_implementation_row(
        "s",
        &error_after_the_stop,
        Some(0),
        6,
    );
    assert_eq!(
        error_after_the_stop_checks
            .iter()
            .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
            .collect::<Vec<_>>(),
        vec![("0", "0", true), ("0", "0", true)]
    );
    // 发布出错而模型说拒得对（`agrees`）、之后有 5 个不适用的态：不是 O1-对拍的红，来源不在登记的两种里，第二样 5 处不合。
    let allowed_refusal = [
        recorded_call_for_the_test(
            "acquire",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::NotModelled,
        ),
        recorded_call_for_the_test(
            "warm_up",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::NotModelled,
        ),
        recorded_call_for_the_test("publish", errored("Y"), ModelVerdictOfCall::Agrees),
    ];
    let allowed_refusal_checks =
        not_applicable_source_checks_of_a_bad_implementation_row("s", &allowed_refusal, Some(2), 5);
    assert_eq!(
        allowed_refusal_checks
            .iter()
            .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
            .collect::<Vec<_>>(),
        vec![("0", "0", true), ("0", "5", false)]
    );
    // 模型停在卸载：不适用的态合规；卸载出错也不算（模型没有卸载的答）。
    let unmount_stop = [
        recorded_call_for_the_test(
            "publish",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::Agrees,
        ),
        recorded_call_for_the_test(
            UNMOUNT_CALL_LABEL,
            errored("Z"),
            ModelVerdictOfCall::NotModelled,
        ),
    ];
    let unmount_stop_checks =
        not_applicable_source_checks_of_a_bad_implementation_row("s", &unmount_stop, Some(1), 7);
    assert!(unmount_stop_checks.iter().all(StopCheck::holds));
    // 模型没停：没有不适用的态，合规。
    let no_stop = [recorded_call_for_the_test(
        "publish",
        CallOutcome::Succeeded,
        ModelVerdictOfCall::Agrees,
    )];
    let no_stop_checks =
        not_applicable_source_checks_of_a_bad_implementation_row("s", &no_stop, None, 0);
    assert!(no_stop_checks.iter().all(StopCheck::holds));
}

/// 哪一趟判哪一份 S5：逐行那一趟判「返回了错误的调用都记了对拍的红」那一份，基线那一趟判原来那一份；
/// 关掉 O1、驱动没走完整的流不判；基线带阳性对照、流没导出也不判。
#[test]
fn the_row_run_judges_the_row_not_applicable_source_checks_and_the_baseline_run_judges_the_baseline_ones(
) {
    let started = Instant::now();
    let silent_acquire_stopped_by_the_error = || {
        let mut export =
            StreamExport::not_exported(StreamName::NewPoolFileCreation, String::new(), started);
        export.not_exported = None;
        export.ending = Some(StreamEnding::StoppedByAnError {
            call_index: 0,
            member: "err:X".to_string(),
        });
        export.calls = vec![recorded_call_for_the_test(
            "acquire",
            CallOutcome::ReturnedError("X".to_string()),
            ModelVerdictOfCall::NotModelled,
        )];
        export.model_stopped_at_call = Some(0);
        export
    };
    let export = silent_acquire_stopped_by_the_error();
    let row_checks = not_applicable_source_checks_to_judge(
        &export,
        RunKind::BadImplementationRow,
        RunSwitches::default(),
    );
    assert_eq!(
        row_checks
            .iter()
            .map(|check| (check.name.as_str(), check.holds()))
            .collect::<Vec<_>>(),
        vec![
            (
                "new_pool_file_creation_o1_every_errored_call_but_the_unmount_is_judged_by_the_comparison",
                false
            ),
            (
                "new_pool_file_creation_o1_not_applicable_only_after_the_unmount_or_a_disagreement",
                true
            )
        ]
    );
    // 阳性对照不挡它（对拍的红正是它要看的）；关掉 O1 不判。
    let with_the_comparison_control = RunSwitches {
        positive_control: Some(PositiveControl::ComparisonOfTheCalls),
        disabled_oracle: None,
        sampling_threads: None,
    };
    assert_eq!(
        not_applicable_source_checks_to_judge(
            &export,
            RunKind::BadImplementationRow,
            with_the_comparison_control
        )
        .len(),
        2
    );
    let with_the_model_off = RunSwitches {
        positive_control: None,
        disabled_oracle: Some(DisabledOracle::ComparisonAndCrashJudge),
        sampling_threads: None,
    };
    assert!(not_applicable_source_checks_to_judge(
        &export,
        RunKind::BadImplementationRow,
        with_the_model_off
    )
    .is_empty());
    // 驱动没走完整（实现 panic）与流水线 panic（`ending` 是 `None`）的流不判，走自崩那一类。
    let mut crashed = silent_acquire_stopped_by_the_error();
    crashed.ending = Some(StreamEnding::ImplementationCrashed {
        what: "x".to_string(),
    });
    assert!(not_applicable_source_checks_to_judge(
        &crashed,
        RunKind::BadImplementationRow,
        RunSwitches::default()
    )
    .is_empty());
    crashed.ending = None;
    assert!(not_applicable_source_checks_to_judge(
        &crashed,
        RunKind::BadImplementationRow,
        RunSwitches::default()
    )
    .is_empty());
    // 基线那一趟判原来那一份：干净的流一两样都过；带阳性对照、流没导出的不判。
    let mut clean =
        StreamExport::not_exported(StreamName::NewPoolFileCreation, String::new(), started);
    clean.not_exported = None;
    clean.ending = Some(StreamEnding::Completed);
    clean.calls = vec![
        recorded_call_for_the_test(
            "acquire",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::NotModelled,
        ),
        recorded_call_for_the_test(
            "warm_up",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::NotModelled,
        ),
        recorded_call_for_the_test(
            "publish",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::Agrees,
        ),
    ];
    let baseline_checks =
        not_applicable_source_checks_to_judge(&clean, RunKind::Baseline, RunSwitches::default());
    assert_eq!(
        baseline_checks
            .iter()
            .map(|check| (check.name.as_str(), check.holds()))
            .collect::<Vec<_>>(),
        vec![
            (
                "new_pool_file_creation_o1_model_stops_only_at_the_unmount",
                true
            ),
            (
                "new_pool_file_creation_every_call_is_judged_or_is_the_unmount",
                true
            )
        ]
    );
    let with_the_pool_checker_control = RunSwitches {
        positive_control: Some(PositiveControl::PoolChecker),
        disabled_oracle: None,
        sampling_threads: None,
    };
    assert!(not_applicable_source_checks_to_judge(
        &clean,
        RunKind::Baseline,
        with_the_pool_checker_control
    )
    .is_empty());
    clean.not_exported = Some("x".to_string());
    assert!(not_applicable_source_checks_to_judge(
        &clean,
        RunKind::Baseline,
        RunSwitches::default()
    )
    .is_empty());
}
