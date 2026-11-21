use super::*;

// ───────────────────────────── 第 4 次重跑写法与拆分段 cpu-writings（登记 research/prompts/e185-r4-prereg.md 第五节「写法与拆分段怎么量」；不带 gpu 特性） ─────────────────────────────

/// 一条意图收下之后逐次调用的（种类、九样计时件），喂系数 φ（B2）。
type CallPiecesOfTheIntent = Vec<(CallKind, CpuWritingPieces)>;

/// 写法与拆分段一路记下的：每个点（f、P、K）每轮收下的那一条意图的逐次调用读数（作废的轮是 `None`）、PW1 或 PW2 没过的计时件。
struct CpuWritingsSegmentRecord {
    pieces_by_point: BTreeMap<SubmissionPoint, Vec<Option<CallPiecesOfTheIntent>>>,
    failed_pieces: BTreeSet<TimedPiece>,
    /// 打了几行 `name=stage`、几行 `name=cpu_writing_stage`（登记第七节乙类：两者相等，段收尾核）。
    stage_lines: usize,
    cpu_writing_stage_lines: usize,
}

/// 段收尾那一行锚点（登记第七节乙类「`cpu_writing_stage` 行数等于同一段 `name=stage` 行数」）。
fn cpu_writing_stage_line_count_anchor(record: &CpuWritingsSegmentRecord) -> (String, bool) {
    let holds = record.cpu_writing_stage_lines == record.stage_lines;
    (
        format!(
            "name=anchor item=cpu_writing_stage_lines value={} expected={} holds={holds}",
            record.cpu_writing_stage_lines, record.stage_lines
        ),
        holds,
    )
}

/// `name=cpu_writing_stage` 一行（B1）：每次重放的调用一行，键与同一次调用的 `name=stage` 行相同（`prefix`、`call_index`、`kind`）。
fn cpu_writing_stage_line(
    prefix: &str,
    call_index: usize,
    kind: CallKind,
    writings: &CpuWritingsOfTheCall,
) -> String {
    let pieces = &writings.pieces;
    format!(
        "name=cpu_writing_stage {prefix} call_index={call_index} kind={} crc_today_cpu_nanoseconds={} crc_hardware_single_cpu_nanoseconds={} crc_hardware_three_way_cpu_nanoseconds={} release_check_crc_today_cpu_nanoseconds={} release_check_crc_hardware_single_cpu_nanoseconds={} release_check_crc_hardware_three_way_cpu_nanoseconds={} scan_today_cpu_nanoseconds={} scan_identity_index_lookup_cpu_nanoseconds={} identity_index_build_cpu_nanoseconds={} identity_index_entries={} identity_index_location_keys={} choose_cpu_nanoseconds={} crc_bytes={} release_check_crc_bytes={} hardware_crc_bytes={} hardware_release_check_crc_bytes={} hardware_crc_matches={} identity_index_scan_matches={} readable={}",
        kind.label(),
        optional_number(pieces.crc_today),
        optional_number(pieces.crc_hardware_single),
        optional_number(pieces.crc_hardware_three_way),
        optional_number(pieces.release_check_crc_today),
        optional_number(pieces.release_check_crc_hardware_single),
        optional_number(pieces.release_check_crc_hardware_three_way),
        optional_number(pieces.scan_today),
        optional_number(pieces.scan_identity_index_lookup),
        optional_number(pieces.identity_index_build),
        writings.identity_index_entries,
        writings.identity_index_location_keys,
        optional_number(writings.choose_cpu_nanoseconds),
        writings.crc_bytes,
        writings.release_check_crc_bytes,
        writings.hardware_crc_bytes.map(|bytes| bytes.to_string()).join(","),
        writings
            .hardware_release_check_crc_bytes
            .map(|bytes| bytes.to_string())
            .join(","),
        writings.hardware_crc_mismatches == 0 && writings.do_the_hardware_crc_bytes_hold(),
        writings
            .identity_index_scan_matches
            .map_or_else(|| "none".to_string(), |matches| matches.to_string()),
        !writings.is_unreadable(),
    )
}

/// 写法与拆分段收下的一条意图（登记第五节「每次调用之后、计时窗口外」①–④）：逐次调用照第二段重放三个阶段、打 `name=stage` 行、核停机条款第 4 条；
/// 再量九种写法的计时件、打 `name=cpu_writing_stage`、核 K-S1 与 K-S2；这一点这一轮还欠 PW1、PW2 时，在头一批上做。交回逐次调用的份额行。
#[allow(
    clippy::too_many_arguments,
    reason = "一条意图的重放要的各样：产物、读数、标签、池、硬件 CRC、对照欠不欠、两份记账"
)]
fn replay_and_emit_the_cpu_writings(
    emitter: &mut Emitter,
    reading: &IntentReading,
    label: &MeasurementLabel,
    pool: &ConstructedPool,
    hardware: &HardwareCrc,
    are_the_controls_due: &mut bool,
    outcome: &mut StageSplitIntentOutcome,
    record: &mut CpuWritingsSegmentRecord,
) -> Vec<StageShareRow> {
    let prefix = format!(
        "{} batch_units={} round={}",
        label.point.label(),
        label.batch_units,
        label.round
    );
    let (batches_of_the_intent, median_batch_cpu) = batches_and_the_median_batch_cpu_of(reading);
    let mut rows = Vec::new();
    let mut pieces_of_the_intent: CallPiecesOfTheIntent = Vec::new();
    for material in &reading.replay_material {
        let call = &reading.calls[material.call_index];
        enter_the_timing_window(TimingWindow::StageReplay);
        let replay = replay_the_stages_of_a_call(
            material,
            reading.region,
            label.choice,
            label.batch_units,
            &pool.devices,
        );
        if let Some(finding) = &replay.stop_finding {
            emitter.line(&format!(
                "name=stop_clause clause=4 {prefix} finding={finding}"
            ));
            outcome.stop_findings.push(finding.clone());
        }
        rows.push(emit_the_stage_shares(
            emitter,
            &prefix,
            call,
            material,
            &replay,
            batches_of_the_intent,
            median_batch_cpu,
        ));
        record.stage_lines += 1;
        enter_the_timing_window(TimingWindow::CpuWritingPieces);
        let writings =
            cpu_writings_of_the_call(material, reading.region, &replay, hardware, &pool.devices);
        emitter.line(&cpu_writing_stage_line(
            &prefix,
            material.call_index,
            material.kind,
            &writings,
        ));
        record.cpu_writing_stage_lines += 1;
        if let Some((clause, finding)) = writings.stop_finding(material.call_index) {
            emitter.line(&format!(
                "name=stop_clause clause={clause} {prefix} finding={finding}"
            ));
            outcome.stop_findings.push(finding);
        }
        // 读不到（索引里一条都没有）的那一次不收（登记第十一节「读不到」），那一轮的系数随之读不到，不当 0
        pieces_of_the_intent.push((
            material.kind,
            if writings.is_unreadable() {
                CpuWritingPieces::default()
            } else {
                writings.pieces
            },
        ));
        if *are_the_controls_due && counts_as_a_batch(material.kind) {
            if let Some(region) = reading.region {
                *are_the_controls_due = false;
                record.failed_pieces.extend(emit_the_cpu_writing_controls(
                    emitter,
                    &prefix,
                    material,
                    region,
                    hardware,
                    &pool.devices,
                ));
            }
        }
    }
    let point = SubmissionPoint {
        fraction: label.point.fraction,
        pool_data_units: label.point.pool_data_units,
        batch_units: label.batch_units,
    };
    if let Some(slot) = record
        .pieces_by_point
        .entry(point)
        .or_insert_with(|| vec![None; ROUNDS])
        .get_mut(label.round - 1)
    {
        *slot = Some(pieces_of_the_intent);
    }
    rows
}

/// B2 的全部行（`name=writing_factor`）：每个点、每种写法、每样阶段一行，各轮的系数与它们的中位；PW1 或 PW2 没过的计时件
/// 让用到它的写法写 `status=void_positive_control`（登记第十一节「阳性对照红」：那一种写法整段作废）。
fn writing_factor_lines(record: &CpuWritingsSegmentRecord) -> Vec<String> {
    let voided: BTreeSet<CpuWriting> = record
        .failed_pieces
        .iter()
        .flat_map(|piece| piece.writings_voided())
        .collect();
    let mut lines = Vec::new();
    for (point, rounds) in &record.pieces_by_point {
        for writing in CpuWriting::ALL {
            for stage in WritingStage::ALL {
                let factor_per_round: Vec<Option<f64>> = rounds
                    .iter()
                    .map(|intent| {
                        intent
                            .as_deref()
                            .and_then(|calls| writing_factor_of_the_intent(calls, writing, stage))
                    })
                    .collect();
                let factor = writing_factor_of_the_rounds(&factor_per_round);
                lines.push(format!(
                    "name=writing_factor {} writing={} stage={} factor_per_round={} factor={} status={}",
                    point.label(),
                    writing.label(),
                    stage.label(),
                    factor_per_round
                        .iter()
                        .map(|value| optional_number(*value))
                        .collect::<Vec<_>>()
                        .join(","),
                    optional_number(factor),
                    if voided.contains(&writing) {
                        "void_positive_control"
                    } else if factor.is_some() {
                        "measured"
                    } else {
                        "not_run"
                    },
                ));
            }
        }
    }
    lines
}

/// 写法与拆分段（登记第五节「写法与拆分段怎么量」，第一节主 agent 定第 6 条撤了拆法乙之后只剩写法那一半）。
pub(super) fn run_the_cpu_writings_segment(points: &[SamplingPoint]) -> i32 {
    let mut emitter = Emitter::new();
    cpu_writings_segment(points, &mut emitter)
}

/// 开跑核 `sse4.2`（不在退 78）与 PW3 的三项（对不上是停机 K-S1）；交回造好的硬件 CRC，或退出码（已打行、已收尾）。
fn hardware_crc_of_the_segment(emitter: &mut Emitter) -> Result<HardwareCrc, i32> {
    let Some(hardware) =
        HardwareCrc::when_the_instructions_are_present(&unit_lengths_of_the_segment())
    else {
        emitter.line("name=verdict segment=cpu_writings status=not_run reason=sse4_2_not_present");
        emitter.finish();
        return Err(EXIT_CODE_NOT_RUN);
    };
    let failures = hardware_crc_check_failures(&hardware);
    for lanes in HardwareCrcLanes::ALL {
        let mismatches: usize = failures
            .iter()
            .filter(|(_item, writing, _count)| *writing == lanes.label())
            .map(|(_item, _writing, count)| *count)
            .sum();
        emitter.line(&format!(
            "name=positive_control row=hardware_crc_check writing={} items=check_string,modulus_bytes_32768,modulus_bytes_16384,random_lengths random_length_strings={RANDOM_LENGTH_CHECKS} random_length_seed={RANDOM_LENGTH_SEED:#018x} mismatches={mismatches} holds={}",
            lanes.label(),
            mismatches == 0
        ));
    }
    if failures.is_empty() {
        return Ok(hardware);
    }
    for (item, writing, count) in &failures {
        emitter.line(&format!(
            "name=stop_clause clause=K-S1 segment=cpu_writings finding=clause_k_s1_{item}_{writing}_mismatches_{count}"
        ));
    }
    emitter.line(&format!(
        "name=verdict segment=cpu_writings stop_clause_findings={}",
        failures.len()
    ));
    emitter.finish();
    Err(EXIT_CODE_STOP_CLAUSE)
}

fn cpu_writings_segment(points: &[SamplingPoint], emitter: &mut Emitter) -> i32 {
    let segment_started = Instant::now();
    let settings = PreparationSettings::from_the_environment();
    emitter.line(&format!(
        "name=configuration segment=cpu_writings points={} batch_sizes={} rounds={ROUNDS} stage_replays={STAGE_REPLAYS} writings={} writing_stages={} timed_pieces={} worker_threads={} threads_of_the_measurement=1 {}",
        points.iter().map(|point| format!("{}:{}", point.fraction.label(), point.pool_data_units)).collect::<Vec<_>>().join(","),
        BATCH_SIZES.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        CpuWriting::ALL.map(CpuWriting::label).join(","),
        WritingStage::ALL.map(WritingStage::label).join(","),
        TimedPiece::ALL.map(TimedPiece::label).join(","),
        worker_threads_from_the_environment(),
        preparation_configuration_fields(points, None, &settings),
    ));
    assert_this_process_runs_one_thread("segment_start");
    if !check_the_apparatus_sha256(emitter) {
        return EXIT_CODE_STOP_CLAUSE;
    }
    if !check_the_thread_cpu_clock_resolution(emitter) {
        return stop_for_the_clock_resolution(emitter, "cpu_writings");
    }
    let hardware = match hardware_crc_of_the_segment(emitter) {
        Ok(hardware) => hardware,
        Err(exit_code) => return exit_code,
    };
    emit_the_machine_state(
        emitter,
        "cpu_writings",
        MachineStatePosition::SegmentStart,
        segment_started,
    );
    let mut calibration = match SegmentCalibration::start("cpu_writings", segment_started, emitter)
    {
        Ok(calibration) => calibration,
        Err(exit_code) => return exit_code,
    };
    let mismatches = check_the_constants_and_the_anchors(emitter);
    let mut stop_findings: Vec<String> = Vec::new();
    let mut record = CpuWritingsSegmentRecord {
        pieces_by_point: BTreeMap::new(),
        failed_pieces: BTreeSet::new(),
        stage_lines: 0,
        cpu_writing_stage_lines: 0,
    };
    if mismatches == 0 {
        let mut queue = PreparationQueue::new(
            "cpu_writings",
            points,
            None,
            &settings,
            &run_the_preparation_work,
        );
        'points: for point_index in 0..points.len() {
            let prepared = queue.prepared_point(point_index, emitter);
            let point = &prepared.recipe.point;
            for line in &prepared.lines {
                emitter.line(line);
            }
            emit_the_machine_state(
                emitter,
                "cpu_writings",
                MachineStatePosition::PointStart(*point),
                segment_started,
            );
            let mut rebuilt_pool_of_the_round: Option<ConstructedPool> = None;
            for round in 1..=ROUNDS {
                wait_until_no_other_measurement_runs(emitter);
                if round > 1 {
                    let rebuilt = queue.take_the_rebuilt_pool(point_index, round, emitter);
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
                    emitter.line(&format!("name=round_invalid arm=memory {} round={round} reason=start_image_digest_differs", point.label()));
                    continue;
                };
                // PW1、PW2 每个点每轮做一次（登记第五节阳性对照表），在这一轮头一条收下的意图的头一批上
                let mut are_the_controls_due = true;
                for batch_units in BATCH_SIZES {
                    let label = MeasurementLabel {
                        arm: DeviceArm::Memory,
                        point: *point,
                        batch_units,
                        round,
                        attempt: 0,
                        injection: WriteInjection::Nothing,
                        choice: prepared.choice,
                    };
                    let outcome = stage_split_one_intent(
                        emitter,
                        label,
                        pool,
                        &mut calibration,
                        &mut |replay_emitter, kept_reading, kept_label, kept_pool, kept_outcome| {
                            replay_and_emit_the_cpu_writings(
                                replay_emitter,
                                kept_reading,
                                kept_label,
                                kept_pool,
                                &hardware,
                                &mut are_the_controls_due,
                                kept_outcome,
                                &mut record,
                            )
                        },
                    );
                    if !outcome.stop_findings.is_empty() {
                        stop_findings.extend(outcome.stop_findings);
                        break 'points;
                    }
                }
            }
            queue.finish_the_point(point_index);
        }
    }
    for line in writing_factor_lines(&record) {
        emitter.line(&line);
    }
    let (anchor_line, does_the_anchor_hold) = cpu_writing_stage_line_count_anchor(&record);
    emitter.line(&anchor_line);
    if !does_the_anchor_hold {
        stop_findings.push("cpu_writing_stage_lines_differ_from_the_stage_lines".to_string());
    }
    calibration.finish(emitter);
    emit_the_machine_state(
        emitter,
        "cpu_writings",
        MachineStatePosition::SegmentEnd,
        segment_started,
    );
    emitter.line(&format!(
        "name=verdict segment=cpu_writings constant_or_anchor_mismatches={mismatches} stop_clause_findings={} positive_control_failures={} positive_control_failure_items={}",
        stop_findings.len(),
        record.failed_pieces.len(),
        if record.failed_pieces.is_empty() {
            "none".to_string()
        } else {
            record.failed_pieces.iter().map(|piece| piece.label()).collect::<Vec<_>>().join(",")
        },
    ));
    emitter.finish();
    if mismatches > 0 || !stop_findings.is_empty() {
        EXIT_CODE_STOP_CLAUSE
    } else if !record.failed_pieces.is_empty() {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

#[cfg(test)]
mod tests;
