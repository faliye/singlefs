//! 一条流的报告：各批、各段的耗时与文件，不适用的来源核对与 O1-RefFS 的核对，连同流的合计。

use super::*;

/// 一条流各项计数（[`report_stream`] 交回、`run_dump` 合起来写判决行）。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct StreamTotals {
    pub(super) failed_checks: usize,
    pub(super) rebuilt_digest_mismatches: usize,
    pub(super) mask_mismatches: usize,
    pub(super) pool_checker_direct_mismatches: usize,
    pub(super) comparison_red_calls: usize,
    pub(super) crash_judge_red_states: usize,
    pub(super) reference_filesystem_red_states: usize,
    pub(super) pool_checker_red_states: usize,
    pub(super) recovery_panic_states: usize,
    /// 各件 panic 的态数（修订 5 第 5.1 节第 5 条，按 [`JudgingPiece::ALL`] 的次序）。
    pub(super) piece_panic_states: [usize; JUDGING_PIECE_COUNT],
    /// 正常那一路 panic、逐件一件都没 panic 的态数（[`unattributed_panic_states_of`]）。
    pub(super) unattributed_panic_states: usize,
    pub(super) probed_states: usize,
    /// (ii) 为真的态、其中第二截判合法的、(i) 为真的态（Q4）。
    pub(super) counter_branch_states: usize,
    pub(super) counter_branch_states_judged_legal: usize,
    pub(super) undecidable_branch_states: usize,
    /// Q4-对账（停机 S11）：拒因是系统配置见证而 (i)(ii) 都不为真的态；(ii) 为真而挂上了的态。
    pub(super) witness_refusals_without_either_branch: usize,
    pub(super) counter_branch_states_that_mounted: usize,
}

/// 一条流的结果行（`E7RESULT` 与 `E7TIMING`）；交回各项计数。
pub(super) fn report_stream(lines: &mut ResultLines, export: &StreamExport) -> StreamTotals {
    let label = export.name.directory_name();
    let mut totals = StreamTotals::default();
    for check in &export.checks {
        totals.failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream={label} check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let ending = export
        .ending
        .as_ref()
        .map_or_else(|| "pipeline_panicked".to_string(), ending_text);
    lines.emit_result_line(&format!(
        "name=stream stream={label} ending={ending} exported={} not_exported_why={} operations={} writes={} segments={} tearable_writes={} calls={} full_states={} in_place_subsets_states={} sampled_states={} model_stopped_at_call={} reference_filesystem_segments={} reference_filesystem_stopped={} comparison_judged_calls_before_the_first_panic={}",
        export.not_exported.is_none(),
        export
            .not_exported
            .as_deref()
            .map_or_else(|| "-".to_string(), field_text),
        export.operation_count,
        export.write_count,
        export.segment_count,
        export.tearable_write_count,
        export.calls.len(),
        export.full_states,
        export.in_place_subsets_states,
        export.observations.len(),
        export
            .model_stopped_at_call
            .map_or_else(|| "none".to_string(), |call| call.to_string()),
        export.reference_filesystem_segments,
        export.reference_filesystem_stopped,
        export
            .calls
            .iter()
            .take_while(|call| !matches!(call.outcome, CallOutcome::Panicked(_)))
            .filter(|call| call.model.was_judged_by_the_comparison())
            .count(),
    ));
    for (call_index, call) in export.calls.iter().enumerate() {
        totals.comparison_red_calls +=
            usize::from(matches!(call.model, ModelVerdictOfCall::Disagrees { .. }));
        lines.emit_result_line(&format!(
            "name=call stream={label} call={call_index} label={} first_operation={} return_point={} status={} o1_comparison={} return_value_sha256={} memory_objects_sha256={}",
            call.label,
            call.first_operation,
            call.return_point,
            field_text(&call.outcome.outcome_word()),
            field_text(&call.model.word()),
            call.return_value
                .as_ref()
                .map_or_else(|| "-".to_string(), |value| value.whole.clone()),
            memory_objects_digest(&call.memory_objects),
        ));
    }
    for scan in &export.torn_scans {
        lines.emit_result_line(&format!(
            "name=torn_scan stream={label} segment={} scanned={} found={} wanted={TORN_STATES_PER_SEGMENT}",
            scan.segment_index, scan.scanned, scan.found
        ));
    }
    for batch in 1..=3u8 {
        let in_batch: Vec<&StateObservation> = export
            .observations
            .iter()
            .filter(|observation| observation.batch == batch)
            .collect();
        report_batch(lines, label, batch, &in_batch, &mut totals);
    }
    totals.mask_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.mask_matches_the_facts)
        .count();
    totals.pool_checker_direct_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.pool_checker_matches_the_judgement)
        .count();
    totals.rebuilt_digest_mismatches = export.rebuilt_digest_mismatches;
    lines.emit_result_line(&format!(
        "name=s4 stream={label} rebuilt_states={} digest_mismatches={}",
        export.rebuilt_states, export.rebuilt_digest_mismatches
    ));
    lines.emit_result_line(&format!(
        "name=crash_facts_mask stream={label} states={} mismatches={}",
        export.observations.len(),
        totals.mask_mismatches
    ));
    lines.emit_result_line(&format!(
        "name=s3 stream={label} states={} o2_direct_differs_from_judge_crash_image={}",
        export.observations.len(),
        totals.pool_checker_direct_mismatches
    ));
    report_files_and_timing(lines, label, export);
    totals
}

/// 一批的计数行（O1 三个子列、O2、恢复 panic、记录核对器与手写版本表的附带计数）、探针那几样，与阶段耗时。
fn report_batch(
    lines: &mut ResultLines,
    label: &str,
    batch: u8,
    in_batch: &[&StateObservation],
    totals: &mut StreamTotals,
) {
    let count_where = |predicate: &dyn Fn(&StateObservation) -> bool| {
        in_batch
            .iter()
            .filter(|observation| predicate(observation))
            .count()
    };
    let crash_judge_is =
        |word: &'static str| count_where(&|observation| observation.crash_judge.word() == word);
    let reference_filesystem_is = |word: &'static str| {
        count_where(&|observation| observation.reference_filesystem.word() == word)
    };
    let red_by_the_pool_checker = count_where(&|observation| {
        observation
            .pool_checker_red
            .as_ref()
            .is_some_and(|items| !items.is_empty())
    });
    totals.crash_judge_red_states += crash_judge_is("red");
    totals.reference_filesystem_red_states += reference_filesystem_is("red");
    totals.pool_checker_red_states += red_by_the_pool_checker;
    totals.recovery_panic_states +=
        count_where(&|observation| observation.recovery_panic.is_some());
    let piece_panic_states = piece_panic_states_of(in_batch.iter().copied());
    for (total, states) in totals.piece_panic_states.iter_mut().zip(piece_panic_states) {
        *total += states;
    }
    totals.unattributed_panic_states += unattributed_panic_states_of(in_batch.iter().copied());
    let segments: BTreeSet<usize> = in_batch
        .iter()
        .map(|observation| observation.segment_index)
        .collect();
    lines.emit_result_line(&format!(
        "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} o1_crash_judge_red={} o1_crash_judge_green={} o1_crash_judge_not_applicable={} o1_crash_judge_off={} o1_reference_filesystem_red={} o1_reference_filesystem_green={} o1_reference_filesystem_abstained={} o1_reference_filesystem_off={} recovery_panic_states={} o2_red={red_by_the_pool_checker} o2_matches_judgement_false={} judge_crash_image_pool_checker_red={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={} piece_panic_states={}",
        in_batch.len(),
        segments.len(),
        count_where(&|observation| observation.landing_vector.contains('1')),
        crash_judge_is("red"),
        crash_judge_is("green"),
        crash_judge_is("na"),
        crash_judge_is("off"),
        reference_filesystem_is("red"),
        reference_filesystem_is("green"),
        reference_filesystem_is("abstained"),
        reference_filesystem_is("off"),
        count_where(&|observation| observation.recovery_panic.is_some()),
        count_where(&|observation| !observation.pool_checker_matches_the_judgement),
        count_where(&|observation| !observation.verdict.pool_checker_red.is_empty()),
        count_where(&|observation| observation.verdict.red_items.iter().any(|item| item.starts_with("record_checker:"))),
        count_where(&|observation| observation.verdict.red_items.iter().any(|item| item.starts_with("journal_consulted_oracle:"))),
        count_where(&|observation| observation.verdict.red_items.iter().any(|item| item.starts_with("journal_ignored_oracle:"))),
        count_where(&|observation| observation.verdict.consulted_root == "none"),
        count_where(&|observation| observation.verdict.consulted_outcome.starts_with("failed:")),
        piece_panic_states_text(&piece_panic_states),
    ));
    let abstention_kinds: BTreeMap<&'static str, usize> =
        in_batch
            .iter()
            .fold(BTreeMap::new(), |mut kinds, observation| {
                if let ReferenceFileSystemVoteOfAState::Abstained { kind } =
                    &observation.reference_filesystem
                {
                    *kinds.entry(*kind).or_insert(0) += 1;
                }
                kinds
            });
    for (kind, states) in abstention_kinds {
        lines.emit_result_line(&format!(
            "name=reference_filesystem_abstentions stream={label} batch={batch} kind={} states={states}",
            field_text(kind)
        ));
    }
    let probes: Vec<(&StateObservation, &WritableMountProbe)> = in_batch
        .iter()
        .filter_map(|observation| {
            observation
                .probe
                .as_ref()
                .map(|probe| (*observation, probe))
        })
        .collect();
    if !probes.is_empty() {
        let probe_count_where = |predicate: &dyn Fn(&WritableMountProbe) -> bool| {
            probes.iter().filter(|(_, probe)| predicate(probe)).count()
        };
        let counter_branch =
            |probe: &WritableMountProbe| probe.criterion.counter_sentence.is_some();
        let undecidable = |probe: &WritableMountProbe| {
            probe
                .criterion
                .some_device_has_fewer_than_two_verified_slots
        };
        let counter_branch_legal =
            probe_count_where(&|probe| counter_branch(probe) && probe.second_crop == "legal");
        let without_either = probe_count_where(&|probe| {
            probe.refused_for_the_system_configuration_witness
                && !counter_branch(probe)
                && !undecidable(probe)
        });
        let counter_branch_mounted =
            probe_count_where(&|probe| counter_branch(probe) && probe.mount == "mounted");
        totals.probed_states += probes.len();
        totals.counter_branch_states += probe_count_where(&counter_branch);
        totals.counter_branch_states_judged_legal += counter_branch_legal;
        totals.undecidable_branch_states += probe_count_where(&undecidable);
        totals.witness_refusals_without_either_branch += without_either;
        totals.counter_branch_states_that_mounted += counter_branch_mounted;
        lines.emit_result_line(&format!(
            "name=probe_batch stream={label} batch={batch} probed_states={} mounted={} refused={} refused_for_the_witness={} undecidable_branch={} undecidable_branch_judged_legal={} counter_branch={} counter_branch_judged_legal={counter_branch_legal} counter_branch_judged_illegal={} witness_refusals_without_either_branch={without_either} counter_branch_mounted={counter_branch_mounted}",
            probes.len(),
            probe_count_where(&|probe| probe.mount == "mounted"),
            probe_count_where(&|probe| probe.mount == "refused"),
            probe_count_where(&|probe| probe.refused_for_the_system_configuration_witness),
            probe_count_where(&undecidable),
            probe_count_where(&|probe| undecidable(probe) && probe.second_crop == "legal"),
            probe_count_where(&counter_branch),
            probe_count_where(&|probe| counter_branch(probe) && probe.second_crop.starts_with("illegal")),
        ));
        for (observation, probe) in probes
            .iter()
            .filter(|(_, probe)| counter_branch(probe) || undecidable(probe))
        {
            lines.emit_result_line(&format!(
                "name=probe_state stream={label} batch={batch} ordinal={} segment={} landing_sha256={} undecidable_branch={} counter_branch={} witnessed_counter={} selected_version={} mount={} refusal={} on_the_d23_list={} second_crop={}",
                observation.ordinal,
                observation.segment_index,
                sha256_hexadecimal(observation.landing_vector.as_bytes()),
                undecidable(probe),
                probe.criterion.counter_sentence.map_or_else(|| "false".to_string(), |sentence| format!("{sentence:?}")),
                probe.criterion.witnessed_counter,
                probe.selected_version,
                probe.mount,
                field_text(&probe.refusal_member),
                field_text(&probe.refusal_on_the_legal_refusal_list),
                field_text(&probe.second_crop),
            ));
        }
    }
    report_stage_times(label, batch, in_batch);
}

/// 一批的阶段耗时（`E7TIMING`，不进入库产物）。
fn report_stage_times(label: &str, batch: u8, in_batch: &[&StateObservation]) {
    for (stage, pick) in [
        (
            "o1",
            (|observation: &StateObservation| observation.crash_judge_nanoseconds)
                as fn(&StateObservation) -> u128,
        ),
        ("o2", |observation| observation.pool_checker_nanoseconds),
        ("recovery", |observation| observation.recovery_nanoseconds),
    ] {
        let (total, median, ninety_fifth_percentile, maximum) = timing_summary(
            in_batch
                .iter()
                .map(|observation| pick(observation))
                .collect(),
        );
        ResultLines::timing(&format!(
            "name=stage_time stream={label} batch={batch} stage={stage} states={} total_ns={total} median_ns={median} p95_ns={ninety_fifth_percentile} max_ns={maximum}",
            in_batch.len()
        ));
    }
}

/// 写出的每个文件一行（带耗时的两个文件只在结果行里给去掉耗时之后的摘要）与这条流的分段耗时。
fn report_files_and_timing(lines: &mut ResultLines, label: &str, export: &StreamExport) {
    for file in &export.files {
        match &file.sha256_without_timing {
            None => lines.emit_result_line(&format!(
                "name=file stream={label} file={} bytes={} sha256={}",
                file.name, file.bytes, file.sha256
            )),
            Some(without_timing) => {
                lines.emit_result_line(&format!(
                    "name=file stream={label} file={} sha256_without_timing={without_timing}",
                    file.name
                ));
                ResultLines::timing(&format!(
                    "name=file_with_timing stream={label} file={} bytes={} sha256={}",
                    file.name, file.bytes, file.sha256
                ));
            }
        }
    }
    ResultLines::timing(&format!(
        "name=stream_time stream={label} elapsed_ns={} drive_ns={} observe_ns={} write_ns={}",
        export.elapsed_nanoseconds,
        export.drive_nanoseconds,
        export.observe_nanoseconds,
        export.write_nanoseconds
    ));
}

/// 基线上 O1 不适用与不比的来源（登记第十一节 S5、第一节读法 6）：卸载由模型答之后，基线上模型一次都不许停（三条流都是），
/// 每次调用要么对拍过（`agrees`）、要么是取号暖机那两次（`not_modelled`）。按 `StopCheck` 的形式回比：期望（本地）与实际（这一趟）。
fn not_applicable_source_checks_of_the_baseline(export: &StreamExport) -> Vec<StopCheck> {
    let label = export.name.directory_name();
    vec![
        StopCheck::new(
            &format!("{label}_o1_model_never_stops"),
            None::<usize>,
            export.model_stopped_at_call,
        ),
        StopCheck::new(
            &format!("{label}_every_call_is_judged_or_is_the_acquire_or_the_warm_up"),
            true,
            export.calls.iter().all(|call| match call.model {
                ModelVerdictOfCall::Agrees => true,
                ModelVerdictOfCall::NotModelled => {
                    call.label == "acquire" || call.label == "warm_up"
                }
                ModelVerdictOfCall::Disagrees { .. }
                | ModelVerdictOfCall::SkippedAfterTheModelStopped
                | ModelVerdictOfCall::NotComparedBecauseTheImplementationCrashed
                | ModelVerdictOfCall::ModelCannotTakeTheObservation { .. } => false,
            }),
        ),
    ]
}

/// 逐行那一趟（坏实现、阳性对照；不关 oracle）的 S5：O1 不适用只许来自读法 6 那三处（对拍判出不同之后、调用交回错误之后、调用 panic 之后），
/// 都按「不合的个数」回比 0：① 交回了错误的调用没有一次留在「没问模型」（`not_modelled`）——取号、暖机出错那一形；
/// 模型停下之后的调用（`skipped_after_the_model_stopped`）不数；② O1-模型崩溃判官的不适用只来自模型停在那三处之一的那次调用。
fn not_applicable_source_checks_of_a_bad_implementation_row(
    label: &str,
    calls: &[RecordedCall],
    model_stopped_at_call: Option<usize>,
    not_applicable_states: usize,
) -> Vec<StopCheck> {
    let errored_calls_the_comparison_did_not_judge = calls
        .iter()
        .filter(|call| matches!(call.outcome, CallOutcome::ReturnedError(_)))
        .filter(|call| matches!(call.model, ModelVerdictOfCall::NotModelled))
        .count();
    let model_stopped_at_one_of_the_three_sources = model_stopped_at_call.is_none_or(|stopped| {
        calls.get(stopped).is_some_and(|call| {
            matches!(call.model, ModelVerdictOfCall::Disagrees { .. })
                || matches!(
                    call.outcome,
                    CallOutcome::ReturnedError(_) | CallOutcome::Panicked(_)
                )
        })
    });
    let not_applicable_states_with_another_source = if model_stopped_at_one_of_the_three_sources {
        0
    } else {
        not_applicable_states
    };
    vec![
        StopCheck::new(
            &format!("{label}_o1_every_errored_call_is_judged_by_the_comparison"),
            0usize,
            errored_calls_the_comparison_did_not_judge,
        ),
        StopCheck::new(
            &format!("{label}_o1_not_applicable_only_after_a_disagreement_an_error_or_a_panic"),
            0usize,
            not_applicable_states_with_another_source,
        ),
    ]
}

/// 这一趟、这一条流要判的 S5（登记第十一节 S5）。基线那一趟判 [`not_applicable_source_checks_of_the_baseline`]：只判不带对照、不关 oracle、流导出了的基线。
/// 逐行那一趟判 [`not_applicable_source_checks_of_a_bad_implementation_row`]：阳性对照照判（它让模型停在翻过的那次调用，是对拍的红）；关掉 O1 不判；
/// 流水线 panic（`ending` 是 `None`）的流不判。
pub(super) fn not_applicable_source_checks_to_judge(
    export: &StreamExport,
    run_kind: RunKind,
    switches: RunSwitches,
) -> Vec<StopCheck> {
    match run_kind {
        RunKind::Baseline => {
            if export.not_exported.is_none()
                && switches.positive_control.is_none()
                && switches.disabled_oracle.is_none()
            {
                not_applicable_source_checks_of_the_baseline(export)
            } else {
                Vec::new()
            }
        }
        RunKind::BadImplementationRow => {
            if export.ending.is_some() && switches.disabled_oracle.is_none() {
                let not_applicable_states = export
                    .observations
                    .iter()
                    .filter(|observation| {
                        observation.crash_judge == CrashJudgeVerdict::NotApplicable
                    })
                    .count();
                not_applicable_source_checks_of_a_bad_implementation_row(
                    export.name.directory_name(),
                    &export.calls,
                    export.model_stopped_at_call,
                    not_applicable_states,
                )
            } else {
                Vec::new()
            }
        }
    }
}

/// 停机 S9 的两样（每一趟都判，导出了的流才判）：RefFS 一侧切的段数与装置切的段数相等（第七节 A9）；翻译停下的理由只许是
/// 本装置走得到的那几种之一（[`ReferenceFileSystemStop`]，停在回退上说明映射或 m 表漏了东西）。
pub(super) fn reference_filesystem_checks_of_a_stream(export: &StreamExport) -> Vec<StopCheck> {
    if export.not_exported.is_some() {
        return Vec::new();
    }
    let label = export.name.directory_name();
    let stopped_on_a_rollback = export.reference_filesystem_stopped.contains(
        field_text(ReferenceFileSystemStop::TheRollbackTargetRootHasNoVersion.name()).as_str(),
    ) || export.reference_filesystem_stopped.contains(
        field_text(ReferenceFileSystemStop::TheReferenceFileSystemDoesNotTakeTheOperation.name())
            .as_str(),
    );
    vec![
        StopCheck::new(
            &format!("{label}_reference_filesystem_segments_equal_the_device_segments"),
            export.segment_count,
            export.reference_filesystem_segments,
        ),
        StopCheck::new(
            &format!("{label}_reference_filesystem_translation_does_not_stop_on_a_missing_root_or_a_refused_operation"),
            false,
            stopped_on_a_rollback,
        ),
    ]
}

pub(super) fn add_stream_totals(totals: &mut StreamTotals, stream: &StreamTotals) {
    totals.failed_checks += stream.failed_checks;
    totals.rebuilt_digest_mismatches += stream.rebuilt_digest_mismatches;
    totals.mask_mismatches += stream.mask_mismatches;
    totals.pool_checker_direct_mismatches += stream.pool_checker_direct_mismatches;
    totals.comparison_red_calls += stream.comparison_red_calls;
    totals.crash_judge_red_states += stream.crash_judge_red_states;
    totals.reference_filesystem_red_states += stream.reference_filesystem_red_states;
    totals.pool_checker_red_states += stream.pool_checker_red_states;
    totals.recovery_panic_states += stream.recovery_panic_states;
    for (total, states) in totals
        .piece_panic_states
        .iter_mut()
        .zip(stream.piece_panic_states)
    {
        *total += states;
    }
    totals.unattributed_panic_states += stream.unattributed_panic_states;
    totals.probed_states += stream.probed_states;
    totals.counter_branch_states += stream.counter_branch_states;
    totals.counter_branch_states_judged_legal += stream.counter_branch_states_judged_legal;
    totals.undecidable_branch_states += stream.undecidable_branch_states;
    totals.witness_refusals_without_either_branch += stream.witness_refusals_without_either_branch;
    totals.counter_branch_states_that_mounted += stream.counter_branch_states_that_mounted;
}

#[cfg(test)]
mod tests;
