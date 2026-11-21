//! 结果行（`E7RESULT`、`E7TIMING`）、与流无关的核对和一条流的报告，连同不适用的来源核对。

use super::*;

// ───────────────────────────── 结果行、V2 与 main ─────────────────────────────

/// `E7RESULT` 行计数（收尾 `name=done emitted=N`，N 把 done 自己也算进去）；`E7TIMING` 行不计数、不进入库产物。
pub(super) struct ResultLines<'output> {
    pub(super) emitted: u64,
    pub(super) output: &'output mut dyn std::io::Write,
}

impl ResultLines<'_> {
    pub(super) fn emit_result_line(&mut self, body: &str) {
        self.emitted += 1;
        writeln!(self.output, "E7RESULT {body}").expect("写得了结果行");
    }
    fn timing(body: &str) {
        println!("E7TIMING {body}");
    }
    pub(super) fn finish(&mut self) {
        self.emitted += 1;
        writeln!(self.output, "E7RESULT name=done emitted={}", self.emitted).expect("写得了结果行");
    }
}

/// 结果行里的值不许带空白（按空格切字段）。
pub(super) fn field_text(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

/// S1 里与流无关的几条：本地常量对验证包库的公开常量或现算值（实验包的造池取的也是这几样，回比不经它）。盘宽是库给新池新建文件那一类池定的
/// 那一档（`history::HistoryDeviceWidth::FourGibibytes`，库里写明与 `tests/common/mod.rs` 的文件镜像同宽）；mkfs 参数是
/// `known_bad_images::parameters_of_the_common_test_pool`；固定写时刻、第一个文件的字节数与内容是 `scenario` 那一份。
pub(super) fn stream_independent_checks() -> Vec<StopCheck> {
    let parameters = singlefs_verification::known_bad_images::parameters_of_the_common_test_pool();
    vec![
        StopCheck::new(
            "device_bytes",
            LOCAL_DEVICE_BYTES,
            singlefs_verification::history::HistoryDeviceWidth::FourGibibytes.device_bytes(),
        ),
        StopCheck::new(
            "physical_block_bytes",
            LOCAL_PHYSICAL_BLOCK_BYTES,
            parameters.geometry.physical_block_size,
        ),
        StopCheck::new(
            "minimum_input_output_bytes",
            LOCAL_MINIMUM_INPUT_OUTPUT_BYTES,
            parameters.geometry.minimum_input_output_bytes,
        ),
        StopCheck::new(
            "sector_bytes",
            LOCAL_SECTOR_BYTES,
            singlefs_verification::memory_pool::SECTOR_BYTES,
        ),
        StopCheck::new(
            "filesystem_identifier",
            LOCAL_FILESYSTEM_IDENTIFIER,
            singlefs_verification::known_bad_images::E142_FILESYSTEM_IDENTIFIER,
        ),
        StopCheck::new(
            "fixed_write_time_seconds",
            LOCAL_FIXED_WRITE_TIME_SECONDS,
            singlefs_verification::scenario::FIXED_WRITE_TIME_SECONDS,
        ),
        StopCheck::new(
            "first_file_bytes",
            LOCAL_FIRST_FILE_BYTES,
            singlefs_verification::scenario::FIRST_FILE_BYTES,
        ),
        StopCheck::new(
            "first_file_content",
            true,
            local_first_file_content() == singlefs_verification::scenario::first_file_content(),
        ),
    ]
}

/// 中位取排好序之后下标 ⌊(n − 1)/2⌋ 那一个，p95 取下标 ⌈0.95·n⌉ − 1 那一个。
fn timing_summary(mut nanoseconds: Vec<u128>) -> (u128, u128, u128, u128) {
    if nanoseconds.is_empty() {
        return (0, 0, 0, 0);
    }
    nanoseconds.sort_unstable();
    let count = nanoseconds.len();
    let total: u128 = nanoseconds.iter().sum();
    let ninety_fifth_percentile_index = (count * 95).div_ceil(100) - 1;
    (
        total,
        nanoseconds[(count - 1) / 2],
        nanoseconds[ninety_fifth_percentile_index],
        nanoseconds[count - 1],
    )
}

/// 一条流的结果行（`E7RESULT` 与 `E7TIMING`）；交回 (S1 没过的条数, S4 对不上的态数, 掩码对不上的态数, S3 对不上的态数,
/// O1-对拍判红的调用数, O1-崩溃判官判红的态数, O2 判红的态数)。
pub(super) fn report_stream(lines: &mut ResultLines<'_>, export: &StreamExport) -> [usize; 7] {
    let label = export.name.directory_name();
    let mut failed_checks = 0usize;
    for check in &export.checks {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream={label} check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let ending_text = match &export.ending {
        Some(StreamEnding::Completed) => "completed".to_string(),
        Some(StreamEnding::StoppedByAnError { call_index, member }) => {
            format!(
                "stopped_by_an_error:call_{call_index}:{}",
                field_text(member)
            )
        }
        Some(StreamEnding::ImplementationCrashed { what }) => {
            format!("implementation_crashed:{}", field_text(what))
        }
        None => "pipeline_panicked".to_string(),
    };
    lines.emit_result_line(&format!(
        "name=stream stream={label} ending={ending_text} exported={} not_exported_why={} operations={} writes={} segments={} tearable_writes={} calls={} full_states={} in_place_subsets_states={} sampled_states={} model_stopped_at_call={}",
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
            .map_or_else(|| "none".to_string(), |call| call.to_string())
    ));
    let mut red_calls = 0usize;
    for (call_index, call) in export.calls.iter().enumerate() {
        red_calls += usize::from(matches!(call.model, ModelVerdictOfCall::Disagrees { .. }));
        lines.emit_result_line(&format!(
            "name=call stream={label} call={call_index} label={} first_operation={} return_point={} status={} o1_comparison={}",
            call.label,
            call.first_operation,
            call.return_point,
            field_text(&call.outcome.outcome_word()),
            field_text(&call.model.word()),
        ));
    }
    for scan in &export.torn_scans {
        lines.emit_result_line(&format!(
            "name=torn_scan stream={label} segment={} scanned={} found={} wanted={TORN_STATES_PER_SEGMENT}",
            scan.segment_index, scan.scanned, scan.found
        ));
    }
    let mut crash_judge_red = 0usize;
    let mut pool_checker_red = 0usize;
    for batch in 1..=3u8 {
        let in_batch: Vec<&StateObservation> = export
            .observations
            .iter()
            .filter(|observation| observation.batch == batch)
            .collect();
        let count_where = |predicate: &dyn Fn(&StateObservation) -> bool| {
            in_batch
                .iter()
                .filter(|observation| predicate(observation))
                .count()
        };
        let crash_judge_is =
            |word: &'static str| count_where(&|observation| observation.crash_judge.word() == word);
        let red_by_the_pool_checker = count_where(&|observation| {
            observation
                .pool_checker_red()
                .is_some_and(|items| !items.is_empty())
        });
        crash_judge_red += crash_judge_is("red");
        pool_checker_red += red_by_the_pool_checker;
        let segments: BTreeSet<usize> = in_batch
            .iter()
            .map(|observation| observation.segment_index)
            .collect();
        lines.emit_result_line(&format!(
            "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} o1_crash_judge_red={} o1_crash_judge_green={} o1_crash_judge_not_applicable={} o1_crash_judge_off={} o2_red={red_by_the_pool_checker} o2_matches_judgement_false={} judge_crash_image_pool_checker_red={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={}",
            in_batch.len(),
            segments.len(),
            count_where(&|observation| observation.landing_vector.contains('1')),
            crash_judge_is("red"),
            crash_judge_is("green"),
            crash_judge_is("na"),
            crash_judge_is("off"),
            count_where(&|observation| !observation.pool_checker_matches_the_judgement()),
            count_where(&|observation| !observation.verdict.pool_checker_red.is_empty()),
            count_where(&|observation| observation
                .verdict
                .red_items
                .iter()
                .any(|item| item.starts_with("record_checker:"))),
            count_where(&|observation| observation
                .verdict
                .red_items
                .iter()
                .any(|item| item.starts_with("journal_consulted_oracle:"))),
            count_where(&|observation| observation
                .verdict
                .red_items
                .iter()
                .any(|item| item.starts_with("journal_ignored_oracle:"))),
            count_where(&|observation| observation.verdict.consulted_root == "none"),
            count_where(&|observation| observation.verdict.consulted_outcome.starts_with("failed:")),
        ));
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
    let mask_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.mask_matches_the_facts)
        .count();
    let pool_checker_direct_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.pool_checker_matches_the_judgement())
        .count();
    lines.emit_result_line(&format!(
        "name=s4 stream={label} rebuilt_states={} digest_mismatches={}",
        export.rebuilt_states, export.rebuilt_digest_mismatches
    ));
    lines.emit_result_line(&format!(
        "name=crash_facts_mask stream={label} states={} mismatches={mask_mismatches}",
        export.observations.len()
    ));
    lines.emit_result_line(&format!(
        "name=s3 stream={label} states={} o2_direct_differs_from_judge_crash_image={pool_checker_direct_mismatches}",
        export.observations.len()
    ));
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
    [
        failed_checks,
        export.rebuilt_digest_mismatches,
        mask_mismatches,
        pool_checker_direct_mismatches,
        red_calls,
        crash_judge_red,
        pool_checker_red,
    ]
}

/// 基线上 O1 不适用的来源只有两种（登记第十一节 S5）：模型没有答的 `unmount`，与对拍判出不同之后。基线上不许有对拍判出不同，
/// 所以模型只许停在流二的 `unmount` 那一次调用（流一一次都不停）。这两条按 `StopCheck` 的形式回比：期望（本地）与实际（这一趟）。
fn not_applicable_source_checks_of_the_baseline(export: &StreamExport) -> Vec<StopCheck> {
    let label = export.name.directory_name();
    let expected_stop = match export.name {
        StreamName::NewPoolFileCreation => None,
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => export
            .calls
            .iter()
            .rposition(|call| call.label == UNMOUNT_CALL_LABEL),
    };
    vec![
        StopCheck::new(
            &format!("{label}_o1_model_stops_only_at_the_unmount"),
            expected_stop,
            export.model_stopped_at_call,
        ),
        StopCheck::new(
            &format!("{label}_every_call_is_judged_or_is_the_unmount"),
            true,
            export.calls.iter().all(|call| {
                matches!(
                    call.model,
                    ModelVerdictOfCall::Agrees | ModelVerdictOfCall::NotModelled
                )
            }),
        ),
    ]
}

/// 逐行那一趟（坏实现、阳性对照；不关 oracle）的 S5：与基线那一趟（[`not_applicable_source_checks_of_the_baseline`]）同一条停机条款（登记第十一节 S5：
/// 「不适用只许来自 `unmount` 与对拍判出不同之后」）。坏实现可以让对拍判出不同、让模型停在那里（那是 O1-对拍的红），所以这一趟不要求模型
/// 只停在 `unmount`；要求两样（登记第一节读法 5、6），都按「不合的个数」回比 0：
/// ① 返回了错误的调用（卸载除外：模型没有卸载的答）没有一次留在「没问模型」（`not_modelled`）——取号、暖机出错那一形；
/// 模型因对拍判出不同停下之后的调用（`skipped_after_the_model_stopped`）不数：读法 6 原文「一次对拍判出不同之后模型不再往前走」，
/// 那是合规来源（登记修订 4）；② O1-崩溃判官的不适用只来自模型停在卸载、或停在对拍判出不同的那次调用。
fn not_applicable_source_checks_of_a_bad_implementation_row(
    label: &str,
    calls: &[RecordedCall],
    model_stopped_at_call: Option<usize>,
    not_applicable_states: usize,
) -> Vec<StopCheck> {
    let errored_calls_the_comparison_did_not_judge = calls
        .iter()
        .filter(|call| call.label != UNMOUNT_CALL_LABEL)
        .filter(|call| matches!(call.outcome, CallOutcome::ReturnedError(_)))
        .filter(|call| matches!(call.model, ModelVerdictOfCall::NotModelled))
        .count();
    let model_stopped_at_the_unmount_or_at_a_disagreement =
        model_stopped_at_call.is_none_or(|stopped| {
            calls.get(stopped).is_some_and(|call| {
                call.label == UNMOUNT_CALL_LABEL
                    || matches!(call.model, ModelVerdictOfCall::Disagrees { .. })
            })
        });
    let not_applicable_states_with_another_source =
        if model_stopped_at_the_unmount_or_at_a_disagreement {
            0
        } else {
            not_applicable_states
        };
    vec![
        StopCheck::new(
            &format!("{label}_o1_every_errored_call_but_the_unmount_is_judged_by_the_comparison"),
            0usize,
            errored_calls_the_comparison_did_not_judge,
        ),
        StopCheck::new(
            &format!("{label}_o1_not_applicable_only_after_the_unmount_or_a_disagreement"),
            0usize,
            not_applicable_states_with_another_source,
        ),
    ]
}

/// 这一趟、这一条流要判的 S5（登记第十一节 S5）。基线那一趟判 [`not_applicable_source_checks_of_the_baseline`]：只判不带对照、不关 oracle、流导出了的基线
/// （阳性对照会让模型在翻过的那次调用停下，关掉 O1 时每次调用都不比）。逐行那一趟判 [`not_applicable_source_checks_of_a_bad_implementation_row`]：阳性对照照判
/// （它让模型停在翻过的那次调用，是对拍的红）；关掉 O1 不判；驱动没走完整（实现 panic、流水线 panic）的流走自崩那一类，不判；
/// 状态上实现 panic 的流驱动是走完了的，照判（那时没有观测，不适用的态数是 0）。
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
            let drive_ran_to_its_end = matches!(
                export.ending,
                Some(StreamEnding::Completed | StreamEnding::StoppedByAnError { .. })
            );
            if drive_ran_to_its_end && switches.disabled_oracle.is_none() {
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

#[cfg(test)]
mod tests;
