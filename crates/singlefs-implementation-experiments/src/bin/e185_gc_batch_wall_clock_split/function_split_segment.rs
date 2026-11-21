use super::*;

// ───────────────────────────── function-split 的算术（第 3 次重跑登记第五节「第三段怎么量」、第六节 A1；纯函数） ─────────────────────────────

/// 附带段一次调用按函数拆开的读数（每项是 5 次计时重放的中位，纳秒；不在这一次调用里的项是 `None`：选段只在签发那一次）。
#[derive(Clone, Copy, Debug, PartialEq)]
struct FunctionSplitReading {
    call_cpu_nanoseconds: f64,
    clone_before_cpu_nanoseconds: f64,
    clone_after_cpu_nanoseconds: f64,
    read_the_version_mapping_cpu_nanoseconds: f64,
    read_the_version_pinned_cpu_nanoseconds: f64,
    region_scan_cpu_nanoseconds: Option<f64>,
    choose_cpu_nanoseconds: Option<f64>,
}

impl FunctionSplitReading {
    /// 两次克隆（调用之前那一版、调用之后发成的那一版）。
    fn clone_cpu_nanoseconds(&self) -> f64 {
        self.clone_before_cpu_nanoseconds + self.clone_after_cpu_nanoseconds
    }

    /// 读这一版（映射条目 + 钉住槽那一圈）。
    fn read_the_version_cpu_nanoseconds(&self) -> f64 {
        self.read_the_version_mapping_cpu_nanoseconds + self.read_the_version_pinned_cpu_nanoseconds
    }

    /// 「发布与其余」= 这一次调用的线程钟 CPU − 上面各项之和（含计划、发布、discard、记账；取余数，不单独重放）。
    fn remainder_cpu_nanoseconds(&self) -> f64 {
        self.call_cpu_nanoseconds
            - self.clone_cpu_nanoseconds()
            - self.read_the_version_cpu_nanoseconds()
            - self.region_scan_cpu_nanoseconds.unwrap_or(0.0)
            - self.choose_cpu_nanoseconds.unwrap_or(0.0)
    }

    /// `name=function_split` 那一行里各项与份额（份额 = 那一项 ÷ 这一次调用的 CPU）。
    fn fields_text(&self) -> String {
        let share = |value: f64| value / self.call_cpu_nanoseconds;
        let optional_share = |value: Option<f64>| optional_number(value.map(share));
        format!(
            "call_cpu_nanoseconds={:.1} clone_before_cpu_nanoseconds={:.1} clone_after_cpu_nanoseconds={:.1} read_the_version_mapping_cpu_nanoseconds={:.1} read_the_version_pinned_cpu_nanoseconds={:.1} region_scan_cpu_nanoseconds={} choose_cpu_nanoseconds={} remainder_cpu_nanoseconds={:.1} clone_share={:.6} read_the_version_share={:.6} region_scan_share={} choose_share={} remainder_share={:.6}",
            self.call_cpu_nanoseconds,
            self.clone_before_cpu_nanoseconds,
            self.clone_after_cpu_nanoseconds,
            self.read_the_version_mapping_cpu_nanoseconds,
            self.read_the_version_pinned_cpu_nanoseconds,
            optional_number(self.region_scan_cpu_nanoseconds),
            optional_number(self.choose_cpu_nanoseconds),
            self.remainder_cpu_nanoseconds(),
            share(self.clone_cpu_nanoseconds()),
            share(self.read_the_version_cpu_nanoseconds()),
            optional_share(self.region_scan_cpu_nanoseconds),
            optional_share(self.choose_cpu_nanoseconds),
            share(self.remainder_cpu_nanoseconds()),
        )
    }
}

/// 一轮里几次调用合起来的各项（批那几次相加；签发那一次单独一份），每项 ÷ 调用 CPU 之和得份额。交回（各项之和、份额）的文字。
fn function_split_round_totals(readings: &[FunctionSplitReading]) -> Option<[f64; 6]> {
    if readings.is_empty() {
        return None;
    }
    let total = |value_of: &dyn Fn(&FunctionSplitReading) -> f64| {
        readings.iter().map(value_of).sum::<f64>()
    };
    Some([
        total(&|reading| reading.call_cpu_nanoseconds),
        total(&FunctionSplitReading::clone_cpu_nanoseconds),
        total(&FunctionSplitReading::read_the_version_cpu_nanoseconds),
        total(&|reading| reading.region_scan_cpu_nanoseconds.unwrap_or(0.0)),
        total(&|reading| reading.choose_cpu_nanoseconds.unwrap_or(0.0)),
        total(&FunctionSplitReading::remainder_cpu_nanoseconds),
    ])
}

/// `name=function_split_summary` 的各项名（与 [`function_split_round_totals`] 的次序相同）。
const FUNCTION_SPLIT_ITEMS: [&str; 6] = [
    "call",
    "clone",
    "read_the_version",
    "region_scan",
    "choose",
    "remainder",
];

/// 五轮中位：每项各轮之和的中位与各轮份额（那一项 ÷ 调用之和）的中位。有一轮读不到的那一项不收那一轮。
fn function_split_summary_text(round_totals: &[[f64; 6]]) -> String {
    FUNCTION_SPLIT_ITEMS
        .iter()
        .enumerate()
        .map(|(item_index, item)| {
            let values: Vec<f64> = round_totals
                .iter()
                .map(|totals| totals[item_index])
                .collect();
            let shares: Vec<f64> = round_totals
                .iter()
                .map(|totals| totals[item_index] / totals[0])
                .collect();
            format!(
                "{item}_cpu_nanoseconds_median={} {item}_share_median={}",
                optional_number(median_of(&values)),
                optional_number(median_of(&shares))
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 停机 J-S6：crates 的 `central_mapping_as_the_compaction_reads_it` 与装置的 `version_as_the_replay_reads_it` 取出的映射条目集合不同。
/// 两边按（key、位置条目的写法）排好再逐条比；交回一句或 `None`。
fn mapping_entries_finding(
    call_index: usize,
    crates_entries: &[CentralMappingEntry],
    apparatus_entries: &[CentralMappingEntry],
) -> Option<String> {
    let sorted = |entries: &[CentralMappingEntry]| {
        let mut texts: Vec<(Vec<u8>, String)> = entries
            .iter()
            .map(|entry| (entry.key.clone(), format!("{:?}", entry.locations)))
            .collect();
        texts.sort();
        texts
    };
    (sorted(crates_entries) != sorted(apparatus_entries)).then(|| {
        format!(
            "clause_j_s6_mapping_entries_differ_at_call_{call_index}_crates_{}_apparatus_{}",
            crates_entries.len(),
            apparatus_entries.len()
        )
    })
}

// ───────────────────────────── 第 3 次重跑第三段 function-split（附带；登记第五节「第三段怎么量」，不带 gpu 特性） ─────────────────────────────

/// 第三段只量 K = 8（登记第五节「第三段怎么量」）。
const FUNCTION_SPLIT_BATCH_UNITS: u64 = 8;

/// 装置那一份钉住槽那一圈（`version_as_the_replay_reads_it` 里与 crates `version_as_the_compaction_reads_it` 那一圈同法的部分）单独重放用。
fn pinned_slots_of_the_version(version: &TransactionOutput) -> BTreeSet<SlotNumber> {
    version_as_the_replay_reads_it(version).pinned_slots
}

/// 一次调用按函数拆（计时窗口外重放；各项先不计时跑一遍、再计时一遍，5 次取中位，[`median_stage_timing`]）：两次克隆、crates 的
/// `central_mapping_as_the_compaction_reads_it`、钉住槽那一圈、`scan_region` 与签发那一次的选段（[`replay_the_stages_of_a_call`]）。
/// 另核停机 J-S6（映射条目两份相同）与原第二段的停机条款第 4 条。没收两版、钟读不到的交回 `Err`（那一次的说明）。
fn function_split_of_the_call(
    material: &CallReplayMaterial,
    call: &CallReading,
    region: Option<CompactionRegion>,
    choice: ChoiceOfTheRegion,
    pool_devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Result<(FunctionSplitReading, Vec<String>), String> {
    let versions = material
        .versions_around_the_call
        .as_ref()
        .ok_or_else(|| format!("versions_not_captured_at_call_{}", material.call_index))?;
    let unreadable = || {
        format!(
            "thread_cpu_clock_unreadable_at_call_{}",
            material.call_index
        )
    };
    let cpu_of =
        |timing: Option<StageTiming>| timing.map(|read| read.cpu_nanoseconds_per_repetition);
    let clone_before = cpu_of(median_stage_timing(|| {
        std::hint::black_box(versions.before.clone());
    }))
    .ok_or_else(unreadable)?;
    let clone_after = match &versions.published {
        Some(published) => cpu_of(median_stage_timing(|| {
            std::hint::black_box(published.clone());
        }))
        .ok_or_else(unreadable)?,
        None => 0.0,
    };
    let mapping = cpu_of(median_stage_timing(|| {
        std::hint::black_box(central_mapping_as_the_compaction_reads_it(&versions.before));
    }))
    .ok_or_else(unreadable)?;
    let pinned = cpu_of(median_stage_timing(|| {
        std::hint::black_box(pinned_slots_of_the_version(&versions.before));
    }))
    .ok_or_else(unreadable)?;
    let mut findings: Vec<String> = mapping_entries_finding(
        material.call_index,
        &central_mapping_as_the_compaction_reads_it(&versions.before),
        &version_as_the_replay_reads_it(&versions.before).mapping_entries,
    )
    .into_iter()
    .collect();
    let stages = replay_the_stages_of_a_call(
        material,
        region,
        choice,
        FUNCTION_SPLIT_BATCH_UNITS,
        pool_devices,
    );
    findings.extend(stages.stop_finding.clone());
    Ok((
        FunctionSplitReading {
            call_cpu_nanoseconds: call.cpu_nanoseconds as f64,
            clone_before_cpu_nanoseconds: clone_before,
            clone_after_cpu_nanoseconds: clone_after,
            read_the_version_mapping_cpu_nanoseconds: mapping,
            read_the_version_pinned_cpu_nanoseconds: pinned,
            region_scan_cpu_nanoseconds: cpu_of(stages.scan),
            choose_cpu_nanoseconds: cpu_of(stages.choose),
        },
        findings,
    ))
}

/// 一个函数的重放（附带段的 2 ms 对照逐个跑）。
type ReplayOfAFunction<'version> = Box<dyn Fn() + 'version>;

/// 附带段新加的两个重放的 2 ms 对照（登记第五节「附带段的阳性对照」）：克隆与读这一版各在计时区间里加 2 ms（按线程钟自旋），
/// 增量在 2 ms 的 [0.9, 1.1] 倍。交回没过的项。
fn function_split_controls(emitter: &mut Emitter, versions: &VersionsAroundTheCall) -> Vec<String> {
    let mut failures = Vec::new();
    let spin = || {
        let _spun = spin_until_the_thread_cpu_advances(INSERTED_NANOSECONDS);
    };
    let cpu_of =
        |timing: Option<StageTiming>| timing.map(|read| read.cpu_nanoseconds_per_repetition);
    let items: [(&str, ReplayOfAFunction<'_>); 2] = [
        (
            "clone",
            Box::new(|| {
                std::hint::black_box(versions.before.clone());
            }),
        ),
        (
            "read_the_version",
            Box::new(|| {
                std::hint::black_box(central_mapping_as_the_compaction_reads_it(&versions.before));
                std::hint::black_box(pinned_slots_of_the_version(&versions.before));
            }),
        ),
    ];
    for (item, replay) in &items {
        let baseline = cpu_of(median_stage_timing(replay));
        let inserted = cpu_of(median_stage_timing(|| {
            replay();
            spin();
        }));
        let increment = baseline.zip(inserted).map(|(without, with)| with - without);
        let holds = increment.is_some_and(spin_control_holds);
        emitter.line(&format!(
            "name=positive_control row=function_split item={item} inserted_nanoseconds={INSERTED_NANOSECONDS} increment_nanoseconds={} holds={holds}",
            optional_number(increment)
        ));
        if !holds {
            failures.push(format!("function_split_{item}"));
        }
    }
    failures
}

/// 第三段一个点、一轮的结局：签发那一次与各批的拆分（收下的那一次；三次都作废是空的）、停机与对照失败。
#[derive(Default)]
struct FunctionSplitIntentOutcome {
    issue: Vec<FunctionSplitReading>,
    batches: Vec<FunctionSplitReading>,
    stop_findings: Vec<String>,
    control_failures: Vec<String>,
}

/// 第三段一条意图（`stage_split_one_intent` 同一套作废与等常态：前后校准、R2-1 作废、R2-2 等常态，至多 3 次）：收下的那一次逐次调用按函数拆，
/// 每次调用一行 `name=function_split`；`with_the_controls` 时另跑原第二段甲、乙、丙的 2 ms 对照与新加两个重放的对照。
fn function_split_one_intent(
    emitter: &mut Emitter,
    label: MeasurementLabel,
    pool: &ConstructedPool,
    with_the_controls: bool,
    calibration: &mut SegmentCalibration,
) -> FunctionSplitIntentOutcome {
    let switches = measured_switches(label.batch_units, label.choice.forced_region());
    let mut outcome = FunctionSplitIntentOutcome::default();
    for attempt in 1..=ATTEMPTS_PER_ROUND_AT_MOST {
        enter_the_timing_window(TimingWindow::StageSplitCompactionRun);
        let mut attempt_label = label;
        attempt_label.attempt = attempt;
        let (reading, intent_calibration) = calibration.around_the_intent(|| {
            measure_on_the_memory_device(
                pool,
                switches,
                WriteInjection::Nothing,
                ReplayCapture::OnWithTheVersions,
            )
        });
        calibration.note_the_intent(emitter, &attempt_label, &intent_calibration);
        let invalid_reason =
            invalid_reason_of(&reading).or(intent_calibration.verdict.invalid_reason());
        emit_the_intent(emitter, &attempt_label, &reading, invalid_reason);
        let Some(reason) = invalid_reason else {
            let prefix = format!(
                "{} batch_units={} round={}",
                label.point.label(),
                label.batch_units,
                label.round
            );
            for material in &reading.replay_material {
                enter_the_timing_window(TimingWindow::StageReplay);
                let call = &reading.calls[material.call_index];
                match function_split_of_the_call(material, call, reading.region, label.choice, &pool.devices) {
                    Ok((split, findings)) => {
                        emitter.line(&format!(
                            "name=function_split {prefix} call_index={} kind={} {}",
                            material.call_index,
                            material.kind.label(),
                            split.fields_text()
                        ));
                        for finding in &findings {
                            emitter.line(&format!("name=stop_clause segment=function_split {prefix} finding={finding}"));
                        }
                        outcome.stop_findings.extend(findings);
                        match material.kind {
                            CallKind::Issue => outcome.issue.push(split),
                            CallKind::Batch | CallKind::FinalBatch => outcome.batches.push(split),
                            CallKind::IntentDeletedWithoutABatch | CallKind::NoPublish => {}
                        }
                    }
                    Err(reason_of_the_call) => emitter.line(&format!(
                        "name=function_split {prefix} call_index={} kind={} status=not_run reason={reason_of_the_call}",
                        material.call_index,
                        material.kind.label()
                    )),
                }
            }
            if with_the_controls {
                enter_the_timing_window(TimingWindow::StageControls);
                let first_batch = reading
                    .replay_material
                    .iter()
                    .find(|material| counts_as_a_batch(material.kind));
                if let (Some(material), Some(region)) = (first_batch, reading.region) {
                    outcome.control_failures.extend(emit_the_stage_controls(
                        emitter,
                        material,
                        region,
                        &pool.devices,
                    ));
                    if let Some(versions) = &material.versions_around_the_call {
                        outcome
                            .control_failures
                            .extend(function_split_controls(emitter, versions));
                    }
                }
            }
            return outcome;
        };
        if attempt < ATTEMPTS_PER_ROUND_AT_MOST {
            calibration.wait_for_the_machine_before_the_retry(emitter, &attempt_label, reason);
        }
    }
    emitter.line(&format!(
        "name=round_invalid segment=function_split {} batch_units={} round={} attempts={ATTEMPTS_PER_ROUND_AT_MOST}",
        label.point.label(),
        label.batch_units,
        label.round,
    ));
    outcome
}

/// 第三段（附带，在第二台跑）：r2 第二段同一套造池、几何核、全局准备队列与校准，只量 K = 8、给的点、5 轮；每轮一条意图按函数拆，
/// 每个点五轮中位两行 `name=function_split_summary`（各批合起来一行、签发一行）。
pub(super) fn run_the_function_split_segment(points: &[SamplingPoint]) -> i32 {
    let segment_started = Instant::now();
    let settings = PreparationSettings::from_the_environment();
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=configuration segment=function_split points={} batch_units={FUNCTION_SPLIT_BATCH_UNITS} rounds={ROUNDS} replays_per_function={STAGE_REPLAYS} worker_threads={} {}",
        points.iter().map(|point| format!("{}:{}", point.fraction.label(), point.pool_data_units)).collect::<Vec<_>>().join(","),
        worker_threads_from_the_environment(),
        preparation_configuration_fields(points, None, &settings),
    ));
    assert_this_process_runs_one_thread("segment_start");
    if !check_the_apparatus_sha256(&mut emitter) {
        return EXIT_CODE_STOP_CLAUSE;
    }
    if !check_the_thread_cpu_clock_resolution(&mut emitter) {
        return stop_for_the_clock_resolution(&mut emitter, "function_split");
    }
    let mut calibration =
        match SegmentCalibration::start("function_split", segment_started, &mut emitter) {
            Ok(calibration) => calibration,
            Err(exit_code) => return exit_code,
        };
    let mismatches = check_the_constants_and_the_anchors(&mut emitter);
    let mut stop_findings: Vec<String> = Vec::new();
    let mut control_failures: Vec<String> = Vec::new();
    if mismatches == 0 {
        let mut queue = PreparationQueue::new(
            "function_split",
            points,
            None,
            &settings,
            &run_the_preparation_work,
        );
        let mut is_the_first_intent = true;
        'points: for point_index in 0..points.len() {
            let prepared = queue.prepared_point(point_index, &mut emitter);
            let point = prepared.recipe.point;
            for line in &prepared.lines {
                emitter.line(line);
            }
            let mut batch_totals: Vec<[f64; 6]> = Vec::new();
            let mut issue_totals: Vec<[f64; 6]> = Vec::new();
            let mut rebuilt_pool_of_the_round: Option<ConstructedPool> = None;
            for round in 1..=ROUNDS {
                wait_until_no_other_measurement_runs(&mut emitter);
                if round > 1 {
                    let rebuilt = queue.take_the_rebuilt_pool(point_index, round, &mut emitter);
                    let is_identical = rebuilt.digests == prepared.reference_digests;
                    emitter.line(&format!(
                        "name=memory_rebuild {} round={round} construction_seconds={:.3} digests_identical={is_identical}",
                        point.label(),
                        rebuilt.pool.construction_seconds
                    ));
                    rebuilt_pool_of_the_round = is_identical.then_some(rebuilt.pool);
                }
                let pool_of_the_round = if round == 1 {
                    Some(&prepared.pool)
                } else {
                    rebuilt_pool_of_the_round.as_ref()
                };
                let Some(pool) = pool_of_the_round else {
                    emitter.line(&format!(
                        "name=round_invalid segment=function_split {} round={round} reason=start_image_digest_differs",
                        point.label()
                    ));
                    continue;
                };
                let label = MeasurementLabel {
                    arm: DeviceArm::Memory,
                    point,
                    batch_units: FUNCTION_SPLIT_BATCH_UNITS,
                    round,
                    attempt: 0,
                    injection: WriteInjection::Nothing,
                    choice: prepared.choice,
                };
                let outcome = function_split_one_intent(
                    &mut emitter,
                    label,
                    pool,
                    is_the_first_intent,
                    &mut calibration,
                );
                is_the_first_intent = false;
                control_failures.extend(outcome.control_failures);
                batch_totals.extend(function_split_round_totals(&outcome.batches));
                issue_totals.extend(function_split_round_totals(&outcome.issue));
                if !outcome.stop_findings.is_empty() {
                    stop_findings.extend(outcome.stop_findings);
                    break 'points;
                }
            }
            for (scope, totals) in [("batches", &batch_totals), ("issue", &issue_totals)] {
                emitter.line(&format!(
                    "name=function_split_summary {} batch_units={FUNCTION_SPLIT_BATCH_UNITS} scope={scope} rounds={} {}",
                    point.label(),
                    totals.len(),
                    function_split_summary_text(totals)
                ));
            }
            queue.finish_the_point(point_index);
        }
    }
    calibration.finish(&mut emitter);
    emitter.line(&format!(
        "name=verdict segment=function_split constant_or_anchor_mismatches={mismatches} stop_clause_findings={} positive_control_failures={} positive_control_failure_items={}",
        stop_findings.len(),
        control_failures.len(),
        if control_failures.is_empty() { "none".to_string() } else { control_failures.join(",") },
    ));
    emitter.finish();
    if mismatches > 0 || !stop_findings.is_empty() {
        EXIT_CODE_STOP_CLAUSE
    } else if !control_failures.is_empty() {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

#[cfg(test)]
mod tests;
