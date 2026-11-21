//! 一条流的报告：各批、各段的耗时与文件，不适用的来源核对与 O1-RefFS 的核对，连同流的合计的单测。

use super::*;

fn recorded_call_for_the_test(
    label: &'static str,
    outcome: CallOutcome,
    model: ModelVerdictOfCall,
) -> RecordedCall {
    RecordedCall {
        label,
        first_operation: 0,
        return_point: 0,
        outcome,
        model,
        return_value: None,
        memory_objects: Vec::new(),
        published_content: None,
        observation_sha256: None,
    }
}

fn disagreement_for_the_test() -> ModelVerdictOfCall {
    ModelVerdictOfCall::Disagrees {
        aspect: "模型说该成、实现拒了",
        model_answer: "往前走".to_string(),
        implementation_answer: "拒了:X".to_string(),
    }
}

fn stop_check_values(checks: &[StopCheck]) -> Vec<(&str, &str, bool)> {
    checks
        .iter()
        .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
        .collect()
}

/// 逐行那一趟的 S5：交回了错误而 O1-对拍没问模型（`not_modelled`）的调用、与不适用来自读法 6 那三处之外的来源，各自回比 0。
#[test]
fn the_row_not_applicable_source_checks_count_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source(
) {
    let errored = |text: &str| CallOutcome::ReturnedError(text.to_string());
    // 取号出错、O1-对拍没记红：第一样 1 处不合；模型停在出错的那次调用（读法 6 的来源之一），第二样 0。
    let silent = [recorded_call_for_the_test(
        "acquire",
        errored("X"),
        ModelVerdictOfCall::NotModelled,
    )];
    let silent_checks =
        not_applicable_source_checks_of_a_bad_implementation_row("s", &silent, Some(0), 3);
    assert_eq!(
        silent_checks
            .iter()
            .map(|check| check.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "s_o1_every_errored_call_is_judged_by_the_comparison",
            "s_o1_not_applicable_only_after_a_disagreement_an_error_or_a_panic"
        ]
    );
    assert_eq!(
        stop_check_values(&silent_checks),
        vec![("0", "1", false), ("0", "0", true)]
    );
    // 取号出错、记成对拍的红：两样都 0。
    let red = [recorded_call_for_the_test(
        "acquire",
        errored("X"),
        disagreement_for_the_test(),
    )];
    assert!(
        not_applicable_source_checks_of_a_bad_implementation_row("s", &red, Some(0), 9)
            .iter()
            .all(StopCheck::holds)
    );
    // 对拍在发布上判出不同、模型停在那里，之后的可写挂载出错（`skipped_after_the_model_stopped`）：两样都 0（原登记修订 4 那一形）。
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
    assert_eq!(
        stop_check_values(&not_applicable_source_checks_of_a_bad_implementation_row(
            "s",
            &error_after_the_stop,
            Some(0),
            6
        )),
        vec![("0", "0", true), ("0", "0", true)]
    );
    // 发布出错而模型说拒得对（`agrees`）、之后有 5 个不适用的态：读法 6「调用交回错误之后（含那一次）」是合规来源，两样都 0。
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
    assert_eq!(
        stop_check_values(&not_applicable_source_checks_of_a_bad_implementation_row(
            "s",
            &allowed_refusal,
            Some(2),
            5
        )),
        vec![("0", "0", true), ("0", "0", true)]
    );
    // 卸载由模型答之后，模型停在一次对拍判过、成功的卸载上不再是合规来源：5 个不适用的态记 5 处不合。
    let stopped_on_an_agreeing_unmount = [
        recorded_call_for_the_test(
            "publish",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::Agrees,
        ),
        recorded_call_for_the_test(
            UNMOUNT_CALL_LABEL,
            CallOutcome::Succeeded,
            ModelVerdictOfCall::Agrees,
        ),
    ];
    assert_eq!(
        stop_check_values(&not_applicable_source_checks_of_a_bad_implementation_row(
            "s",
            &stopped_on_an_agreeing_unmount,
            Some(1),
            5
        )),
        vec![("0", "0", true), ("0", "5", false)]
    );
    // panic 截断：模型停在 panic 的那次调用，合规；panic 那次不算「出了错没问模型」。
    let panicked = [
        recorded_call_for_the_test(
            "publish",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::Agrees,
        ),
        recorded_call_for_the_test(
            "publish",
            CallOutcome::Panicked("炸".to_string()),
            ModelVerdictOfCall::NotComparedBecauseTheImplementationCrashed,
        ),
    ];
    assert!(
        not_applicable_source_checks_of_a_bad_implementation_row("s", &panicked, Some(1), 4)
            .iter()
            .all(StopCheck::holds)
    );
    // 模型没停：没有不适用的态，合规。
    let no_stop = [recorded_call_for_the_test(
        "publish",
        CallOutcome::Succeeded,
        ModelVerdictOfCall::Agrees,
    )];
    assert!(
        not_applicable_source_checks_of_a_bad_implementation_row("s", &no_stop, None, 0)
            .iter()
            .all(StopCheck::holds)
    );
}

/// 哪一趟判哪一份 S5：逐行那一趟判「交回了错误的调用都记了对拍的红」那一份，基线那一趟判「模型一次都不停」那一份；
/// 关掉 O1、流水线 panic 的流不判；基线带阳性对照、流没导出也不判。
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
            ("new_pool_file_creation_o1_every_errored_call_is_judged_by_the_comparison", false),
            ("new_pool_file_creation_o1_not_applicable_only_after_a_disagreement_an_error_or_a_panic", true)
        ]
    );
    let with_the_model_off = RunSwitches {
        positive_control: None,
        disabled_oracle: Some(DisabledOracle::ComparisonAndCrashJudge),
        writable_mount_probe: false,
    };
    assert!(not_applicable_source_checks_to_judge(
        &export,
        RunKind::BadImplementationRow,
        with_the_model_off
    )
    .is_empty());
    let mut pipeline_panicked = silent_acquire_stopped_by_the_error();
    pipeline_panicked.ending = None;
    assert!(not_applicable_source_checks_to_judge(
        &pipeline_panicked,
        RunKind::BadImplementationRow,
        RunSwitches::default()
    )
    .is_empty());
    // panic 截断的流照判（读法 5：panic 与报错同样截断照判）。
    let mut panicked = silent_acquire_stopped_by_the_error();
    panicked.ending = Some(StreamEnding::StoppedByAPanic {
        call_index: 0,
        message: "炸".to_string(),
    });
    assert_eq!(
        not_applicable_source_checks_to_judge(
            &panicked,
            RunKind::BadImplementationRow,
            RunSwitches::default()
        )
        .len(),
        2
    );
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
        recorded_call_for_the_test(
            UNMOUNT_CALL_LABEL,
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
            ("new_pool_file_creation_o1_model_never_stops", true),
            (
                "new_pool_file_creation_every_call_is_judged_or_is_the_acquire_or_the_warm_up",
                true
            )
        ]
    );
    // 卸载留在 not_modelled（没改调模型的卸载答）在基线上不过。
    clean.calls[3].model = ModelVerdictOfCall::NotModelled;
    assert!(!not_applicable_source_checks_to_judge(
        &clean,
        RunKind::Baseline,
        RunSwitches::default()
    )[1]
    .holds());
    clean.not_exported = Some("x".to_string());
    assert!(not_applicable_source_checks_to_judge(
        &clean,
        RunKind::Baseline,
        RunSwitches::default()
    )
    .is_empty());
}
