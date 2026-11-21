use super::host_cpu_per_submission_segment::{
    cell_calibration_ratio, check_string_holds, comparison_control_failures,
    cpu_crc_of_the_same_bytes, finish_the_calibration, kernel_one_findings, segment_units,
    steps_of, thread_clock_control_failures, threads_of_this_process, thresholds_of_the_segment,
    wait_until_the_card_has_no_other_compute_process_reading_the_cards, SegmentThresholds,
    SegmentUnits,
};
use super::resident_gpu_kernels::ResidentKernels;
use super::*;
use singlefs_verification::gpu_unit_checks::{gpu_cards_by_priority, GpuCard};

/// 真卡上的一格一轮：第 3 次重跑的提交步骤与真钟，16 MiB 那份 CRC 的校准。
struct GpuCellEnvironment<'segment> {
    card_index: u32,
    card: &'segment GpuCard,
    kernels: &'segment ResidentKernels,
    segment_units: &'segment SegmentUnits,
    calibration_buffer: &'segment [u8],
    baseline_nanoseconds: f64,
    link_peak_bytes_per_second: f64,
    steps: Option<Box<dyn SubmissionSteps + 'segment>>,
}

impl CellRoundEnvironment for GpuCellEnvironment<'_> {
    fn card_index(&self) -> u32 {
        self.card_index
    }

    fn link_peak_bytes_per_second(&self) -> f64 {
        self.link_peak_bytes_per_second
    }

    fn host_core_reading(&mut self) -> HostCoreReading {
        host_core_reading_now()
    }

    fn calibration_ratio(&mut self) -> Option<f64> {
        cell_calibration_ratio(self.calibration_buffer, self.baseline_nanoseconds)
    }

    fn threads(&mut self) -> Option<usize> {
        threads_of_this_process()
    }

    fn open_the_cell(&mut self, cell: SubmissionCell) {
        self.steps = Some(steps_of(
            cell.form,
            cell.wait,
            cell.units,
            self.kernels,
            self.card,
            self.segment_units,
            None,
        ));
    }

    fn time_the_submissions(
        &mut self,
        count: usize,
        gap: Duration,
        insertion: &mut dyn FnMut(),
    ) -> Vec<Option<SubmissionTiming>> {
        let steps = self
            .steps
            .as_mut()
            .expect("open_the_cell 之后才计时提交：measure_one_cell_round 与暖到稳都先开格");
        let mut sleep_between_submissions = |duration: Duration| {
            if !duration.is_zero() {
                std::thread::sleep(duration);
            }
        };
        time_the_submissions_of_a_cell(
            steps.as_mut(),
            &mut ClocksOfThisThread,
            count,
            gap,
            &mut sleep_between_submissions,
            insertion,
        )
    }

    fn close_the_cell(&mut self) {
        self.steps = None;
    }
}

/// 真卡的读卡：第 3 次重跑开格前那一次 `nvidia-smi --query-gpu` 多查六栏（[`CARD_QUERY_FIELDS`]），等法照旧。
struct GpuCardReader;

impl CardReader for GpuCardReader {
    fn wait_for_the_card_and_read_the_cards(
        &mut self,
        card_index: u32,
        emitter: &mut Emitter,
    ) -> BTreeMap<u32, CardClockReading> {
        let query_text = wait_until_the_card_has_no_other_compute_process_reading_the_cards(
            emitter,
            card_index,
            CARD_QUERY_FIELDS,
            "csv,noheader,nounits",
            |text| {
                card_clock_readings_of(text)
                    .into_iter()
                    .map(|(index, reading)| (index, reading.bus_identifier))
                    .collect()
            },
        );
        query_text
            .as_deref()
            .map(card_clock_readings_of)
            .unwrap_or_default()
    }
}

/// 控漂移段里起好的一张卡：两份核、链路峰值、P2 过没过、P3 没过的输入形态。
struct CardOfTheSegment {
    index: u32,
    card: GpuCard,
    kernels: ResidentKernels,
    link_peak_bytes_per_second: f64,
    check_string_holds: bool,
    comparison_control_failures: Vec<InputForm>,
}

/// 起一张卡（第 3 次重跑第一段原样）：编核、开常驻缓冲、P2、停机 J-S1、P3。卡起不来交 `Ok(None)`（`name=gpu_card status=not_run`）；
/// 链路读不到交 `Ok(None)` 并记进 `card_failures`；J-S1 对不上交 `Err(那几条)`。
fn start_the_card(
    card_index: u32,
    links: &BTreeMap<u32, (u32, u32)>,
    segment_units: &SegmentUnits,
    card_failures: &mut Vec<String>,
    emitter: &mut Emitter,
) -> Result<Option<CardOfTheSegment>, Vec<String>> {
    let Some(card) = gpu_cards_by_priority(&[(card_index, CARD_QUOTA_MEBIBYTES)])
        .into_iter()
        .next()
    else {
        emitter.line(&format!(
            "name=gpu_card index={card_index} status=not_run reason=card_not_usable"
        ));
        return Ok(None);
    };
    let Some(link_peak) = links
        .get(&card_index)
        .and_then(|(generation, width)| pcie_link_peak_bytes_per_second(*generation, *width))
    else {
        emitter.line(&format!(
            "name=gpu_card index={card_index} status=invalid reason=pcie_link_unreadable"
        ));
        card_failures.push(format!("card_{card_index}_link_unreadable"));
        return Ok(None);
    };
    let kernels = ResidentKernels::start(&card, LARGEST_SUBMISSION_UNITS);
    emitter.line(&format!(
        "name=gpu_card index={card_index} card_name={} driver={} free_memory_mebibytes={} link_peak_bytes_per_second={link_peak:.0} resident_capacity_units={} threads_after_the_card_started={}",
        field_token(&card.name),
        field_token(&card.driver_version),
        card.free_memory_mebibytes,
        kernels.capacity_units,
        threads_of_this_process().map_or_else(|| "none".to_string(), |count| count.to_string()),
    ));
    let check_string_holds = check_string_holds(&kernels, emitter, card_index);
    let findings = kernel_one_findings(&kernels, &card, segment_units, emitter);
    if !findings.is_empty() {
        return Err(findings);
    }
    let comparison_control_failures =
        comparison_control_failures(&kernels, &card, segment_units, emitter);
    Ok(Some(CardOfTheSegment {
        index: card_index,
        card,
        kernels,
        link_peak_bytes_per_second: link_peak,
        check_string_holds,
        comparison_control_failures,
    }))
}

/// 开格前读到的主机核与这张卡（读不到这张卡是 `None`）。
type ReadingsBeforeTheCell = (HostCoreReading, Option<CardClockReading>);

/// 一条臂量下来的东西：各格各轮的中位、作废、触发 J5 的格、各卡的累计作废读数、两格探针各轮的中位、频率读数、暖到稳。
struct ArmRecord {
    arm: DriftArm,
    cards: Vec<u32>,
    medians: CellMediansOfTheSegment,
    voiding: BTreeMap<(u32, SubmissionForm, WaitMethod), &'static str>,
    process_clock_cells: BTreeSet<(u32, SubmissionCell)>,
    tallies: BTreeMap<u32, CardTallies>,
    probe_medians: BTreeMap<(u32, DriftProbe), Vec<Option<f64>>>,
    /// 每张卡每轮各格开格前的读数（`name=clock_round_summary`）。
    before_readings: BTreeMap<(u32, usize), Vec<ReadingsBeforeTheCell>>,
    /// 每张卡（格一级 c_rt、开格前的 SM 时钟）的对子（`name=clock_correlation`）。
    clock_pairs: BTreeMap<u32, Vec<(f64, f64)>>,
    warm_ups: BTreeMap<u32, WarmUpToStable>,
    cell_warm_up_submissions: BTreeMap<u32, Vec<usize>>,
}

impl ArmRecord {
    fn new(arm: DriftArm, cards: &[u32]) -> ArmRecord {
        ArmRecord {
            arm,
            cards: cards.to_vec(),
            medians: cards
                .iter()
                .flat_map(|card| {
                    submission_cells_of_a_card()
                        .into_iter()
                        .map(move |cell| ((*card, cell), vec![None; ROUNDS]))
                })
                .collect(),
            voiding: BTreeMap::new(),
            process_clock_cells: BTreeSet::new(),
            tallies: cards
                .iter()
                .map(|card| (*card, CardTallies::default()))
                .collect(),
            probe_medians: cards
                .iter()
                .flat_map(|card| DriftProbe::ALL.map(|probe| ((*card, probe), vec![None; ROUNDS])))
                .collect(),
            before_readings: BTreeMap::new(),
            clock_pairs: BTreeMap::new(),
            warm_ups: BTreeMap::new(),
            cell_warm_up_submissions: BTreeMap::new(),
        }
    }
}

/// 开格前读到的卡读数要配给上一格当「后」：GPU 那几栏取下一格开格前那一次 `nvidia-smi`（并进本来就起的那一次，不多起进程），
/// 主机核那两栏取上一格量完当场读的。
struct PendingAfterReading {
    key_text: String,
    card_index: u32,
    round: usize,
    host_after: HostCoreReading,
}

fn clock_reading_line(
    key_text: &str,
    round: usize,
    position: &str,
    host: HostCoreReading,
    card: Option<&CardClockReading>,
) -> String {
    let count_of =
        |value: Option<u64>| value.map_or_else(|| "none".to_string(), |number| number.to_string());
    let word_of = |value: Option<&String>| value.cloned().unwrap_or_else(|| "none".to_string());
    format!(
        "name=clock_reading {key_text} round={round} position={position} host_core={} scaling_current_frequency_kilohertz={} average_frequency_kilohertz={} sm_clock_megahertz={} memory_clock_megahertz={} performance_state={} temperature_celsius={} power_watts={} clocks_event_reasons={}",
        host.core.map_or_else(|| "none".to_string(), |core| core.to_string()),
        count_of(host.scaling_current_frequency_kilohertz),
        count_of(host.average_frequency_kilohertz),
        optional_number(card.and_then(|reading| reading.sm_clock_megahertz)),
        optional_number(card.and_then(|reading| reading.memory_clock_megahertz)),
        word_of(card.and_then(|reading| reading.performance_state.as_ref())),
        optional_number(card.and_then(|reading| reading.temperature_celsius)),
        optional_number(card.and_then(|reading| reading.power_watts)),
        word_of(card.and_then(|reading| reading.clocks_event_reasons.as_ref())),
    )
}

/// 上一格还欠的「后」读数，用这一次读到的各张卡补上。
fn emit_the_pending_after_reading(
    pending: &mut Option<PendingAfterReading>,
    cards_now: &BTreeMap<u32, CardClockReading>,
    emitter: &mut Emitter,
) {
    if let Some(waiting) = pending.take() {
        emitter.line(&clock_reading_line(
            &waiting.key_text,
            waiting.round,
            "after",
            waiting.host_after,
            cards_now.get(&waiting.card_index),
        ));
    }
}

/// 段里共用的东西：门槛的边、校准、读卡、欠着的「后」读数。
struct SegmentContext<'segment> {
    threshold_edges: (f64, f64),
    calibration: &'segment mut SegmentCalibration,
    card_reader: GpuCardReader,
    pending_after: Option<PendingAfterReading>,
}

/// 一张卡在一条臂上的一格一轮（80 格之一或两格探针之一）：量、记进 `record`、打频率读数。停机 J-S5 交回 `Err`。
fn measure_and_record(
    card: &CardOfTheSegment,
    segment_units: &SegmentUnits,
    context: &mut SegmentContext<'_>,
    record: &mut ArmRecord,
    request: CellRoundRequest,
    emitter: &mut Emitter,
) -> Result<(), String> {
    let SegmentCalibration {
        buffer,
        baseline,
        waiting,
        larger_ratio_per_intent,
        ..
    } = &mut *context.calibration;
    let mut environment = GpuCellEnvironment {
        card_index: card.index,
        card: &card.card,
        kernels: &card.kernels,
        segment_units,
        calibration_buffer: buffer.as_slice(),
        baseline_nanoseconds: baseline.crc_median_nanoseconds,
        link_peak_bytes_per_second: card.link_peak_bytes_per_second,
        steps: None,
    };
    // 探针格的 n = 8 与 80 格里的一格同键：它触发 J5 不往 80 格的进程钟那几行里记，别的作废读数照算进这张卡
    let mut probe_tallies = CardTallies::default();
    let tallies = match request.probe {
        None => record.tallies.entry(card.index).or_default(),
        Some(_) => &mut probe_tallies,
    };
    let mut reader_noting_the_pending = PendingNotingReader {
        reader: &mut context.card_reader,
        pending: &mut context.pending_after,
    };
    let outcome = measure_one_cell_round(
        &mut environment,
        &mut reader_noting_the_pending,
        request,
        tallies,
        waiting,
        larger_ratio_per_intent,
        emitter,
    )?;
    let key_text = request.key_text(card.index);
    let card_before = outcome.cards_before.get(&card.index);
    emitter.line(&clock_reading_line(
        &key_text,
        request.round,
        "before",
        outcome.host_before,
        card_before,
    ));
    context.pending_after = Some(PendingAfterReading {
        key_text,
        card_index: card.index,
        round: request.round,
        host_after: outcome.host_after,
    });
    record
        .before_readings
        .entry((card.index, request.round))
        .or_default()
        .push((outcome.host_before, card_before.cloned()));
    let round_index = request.round - 1;
    match request.probe {
        None => {
            if let Some(slot) = record
                .medians
                .get_mut(&(card.index, request.cell))
                .and_then(|rounds| rounds.get_mut(round_index))
            {
                *slot = outcome.medians;
            }
            if let (Some(medians), Some(sm_clock)) = (
                outcome.medians,
                card_before.and_then(|reading| reading.sm_clock_megahertz),
            ) {
                record
                    .clock_pairs
                    .entry(card.index)
                    .or_default()
                    .push((medians.thread_cpu_nanoseconds, sm_clock));
            }
            if record.arm == DriftArm::Controlled {
                record
                    .cell_warm_up_submissions
                    .entry(card.index)
                    .or_default()
                    .push(outcome.warm_up_submissions);
            }
        }
        Some(probe) => {
            if let Some(slot) = record
                .probe_medians
                .get_mut(&(card.index, probe))
                .and_then(|rounds| rounds.get_mut(round_index))
            {
                *slot = outcome
                    .medians
                    .map(|medians| medians.thread_cpu_nanoseconds);
            }
        }
    }
    if request.probe.is_some() {
        let card_tallies = record.tallies.entry(card.index).or_default();
        card_tallies.result_mismatches += probe_tallies.result_mismatches;
        card_tallies.floor_violations += probe_tallies.floor_violations;
        card_tallies.input_byte_mismatches += probe_tallies.input_byte_mismatches;
        card_tallies
            .coverage_failures
            .extend(probe_tallies.coverage_failures);
    }
    Ok(())
}

/// 读卡时顺手补上一格欠的「后」读数（同一次 `nvidia-smi`）。
struct PendingNotingReader<'reader> {
    reader: &'reader mut GpuCardReader,
    pending: &'reader mut Option<PendingAfterReading>,
}

impl CardReader for PendingNotingReader<'_> {
    fn wait_for_the_card_and_read_the_cards(
        &mut self,
        card_index: u32,
        emitter: &mut Emitter,
    ) -> BTreeMap<u32, CardClockReading> {
        let cards_now = self
            .reader
            .wait_for_the_card_and_read_the_cards(card_index, emitter);
        emit_the_pending_after_reading(self.pending, &cards_now, emitter);
        cards_now
    }
}

/// 受控臂 ① 暖到稳一张卡：探针格（n = 512、常驻共用输入、阻塞等、间隔 0）每 20 次一窗，打 `name=warm_up_to_stable`。
fn warm_up_the_card_to_stable(
    card: &CardOfTheSegment,
    segment_units: &SegmentUnits,
    context: &mut SegmentContext<'_>,
    emitter: &mut Emitter,
) -> WarmUpToStable {
    let mut cards_now = context
        .card_reader
        .wait_for_the_card_and_read_the_cards(card.index, emitter);
    emit_the_pending_after_reading(&mut context.pending_after, &cards_now, emitter);
    cards_now.clear();
    let mut environment = GpuCellEnvironment {
        card_index: card.index,
        card: &card.card,
        kernels: &card.kernels,
        segment_units,
        calibration_buffer: context.calibration.buffer.as_slice(),
        baseline_nanoseconds: context.calibration.baseline.crc_median_nanoseconds,
        link_peak_bytes_per_second: card.link_peak_bytes_per_second,
        steps: None,
    };
    let cell = warm_up_probe_cell();
    environment.open_the_cell(cell);
    let started = Instant::now();
    let warm_up = warm_up_to_stable(
        &mut || {
            let timings = environment.time_the_submissions(
                WARM_UP_WINDOW_SUBMISSIONS,
                Duration::ZERO,
                &mut || {},
            );
            cell_round_reading(&timings, cell.units)
                .map(|reading| reading.thread_cpu_median_nanoseconds)
        },
        &mut || started.elapsed().as_secs_f64(),
    );
    environment.close_the_cell();
    let (peak, changed, last) = warm_up.trajectory();
    emitter.line(&format!(
        "name=warm_up_to_stable arm=controlled index={} {} windows={} window_submissions={WARM_UP_WINDOW_SUBMISSIONS} seconds={:.3} window_medians_nanoseconds={} reached_the_cap={} window_median_peak_nanoseconds={} windows_off_the_previous_by_more_than_three_percent={changed} last_window_median_nanoseconds={}",
        card.index,
        cell.label(),
        warm_up.window_medians_nanoseconds.len(),
        warm_up.seconds,
        warm_up.window_medians_nanoseconds.iter().map(|median| optional_number(*median)).collect::<Vec<_>>().join(","),
        warm_up.reached_the_cap,
        optional_number(peak),
        optional_number(last),
    ));
    warm_up
}

/// 一条臂的 5 轮（登记第五节「复现臂」「受控臂」）：每轮开头等别的性能测量；按 [`cards_in_the_order_of_the_round`] 量卡，每张卡 80 格照第 3 次重跑
/// 轮转、再两格探针。受控臂开轮之前先暖卡（后暖的先量：先暖次序表里后量的那张）。停机 J-S5 交回 `Err`。
fn measure_the_arm(
    cards: &[&CardOfTheSegment],
    segment_units: &SegmentUnits,
    context: &mut SegmentContext<'_>,
    record: &mut ArmRecord,
    emitter: &mut Emitter,
) -> Result<(), String> {
    let indices: Vec<u32> = cards.iter().map(|card| card.index).collect();
    let card_of = |index: u32| cards.iter().copied().find(|card| card.index == index);
    if record.arm == DriftArm::Controlled {
        for index in cards_in_the_order_of_the_round(record.arm, 1, &indices)
            .into_iter()
            .rev()
        {
            let card = card_of(index).expect("次序里的卡都在起好的卡里");
            let warm_up = warm_up_the_card_to_stable(card, segment_units, context, emitter);
            record.warm_ups.insert(index, warm_up);
        }
    }
    for round in 1..=ROUNDS {
        wait_until_no_other_measurement_runs(emitter);
        for index in cards_in_the_order_of_the_round(record.arm, round, &indices) {
            let card = card_of(index).expect("次序里的卡都在起好的卡里");
            let requests = submission_cells_in_the_order_of_the_round(round)
                .into_iter()
                .map(|cell| (cell, None))
                .chain(DriftProbe::ALL.map(|probe| (probe_cell(), Some(probe))));
            for (cell, probe) in requests {
                let request = CellRoundRequest {
                    arm: record.arm,
                    cell,
                    round,
                    probe,
                    threshold_edges: context.threshold_edges,
                };
                measure_and_record(card, segment_units, context, record, request, emitter)?;
            }
        }
    }
    let cards_now = context
        .card_reader
        .wait_for_the_card_and_read_the_cards(indices[0], emitter);
    emit_the_pending_after_reading(&mut context.pending_after, &cards_now, emitter);
    for (card, tallies) in &record.tallies {
        for cell in &tallies.process_clock_cells {
            record.process_clock_cells.insert((*card, *cell));
        }
    }
    Ok(())
}

/// 一条臂量完之后的作废：P3（起卡时）、P1（这条臂里）、第七节甲类的区间与计数（这条臂里）、卡作废（P2、结果、物理下限、写入字节锚点）。
/// 交回这条臂没过的卡（`card_failures` 那几条）。
fn void_the_arm(
    cards: &[&CardOfTheSegment],
    thread_clock_failures: &BTreeMap<u32, Vec<(SubmissionForm, WaitMethod)>>,
    record: &mut ArmRecord,
    emitter: &mut Emitter,
) -> Vec<String> {
    let mut card_failures = Vec::new();
    for card in cards {
        for form in &card.comparison_control_failures {
            for wait in WaitMethod::ALL {
                record.voiding.insert(
                    (card.index, SubmissionForm::Resident(*form), wait),
                    "void_positive_control",
                );
            }
        }
        for (form, wait) in thread_clock_failures.get(&card.index).into_iter().flatten() {
            record
                .voiding
                .insert((card.index, *form, *wait), "void_positive_control");
        }
        let tallies = record.tallies.get(&card.index).cloned().unwrap_or_default();
        for (form, wait) in &tallies.coverage_failures {
            emitter.line(&format!(
                "name=positive_control row=interval_coverage arm={} index={} form={} wait={} holds=false",
                record.arm.label(),
                card.index,
                form.label(),
                wait.label()
            ));
            record
                .voiding
                .insert((card.index, *form, *wait), "void_positive_control");
        }
        let is_the_card_valid = card.check_string_holds
            && tallies.result_mismatches == 0
            && tallies.floor_violations == 0
            && tallies.input_byte_mismatches == 0;
        emitter.line(&format!(
            "name=gpu_card_verdict arm={} index={} check_string_holds={} result_mismatching_cell_rounds={} floor_violations={} input_byte_mismatching_cell_rounds={} card_valid={is_the_card_valid}",
            record.arm.label(),
            card.index,
            card.check_string_holds,
            tallies.result_mismatches,
            tallies.floor_violations,
            tallies.input_byte_mismatches,
        ));
        if !is_the_card_valid {
            card_failures.push(format!(
                "{}_card_{}_result_or_floor",
                record.arm.label(),
                card.index
            ));
            for form in [
                SubmissionForm::Resident(InputForm::Shared),
                SubmissionForm::Resident(InputForm::Split),
                SubmissionForm::TodaySingleKernel,
            ] {
                for wait in WaitMethod::ALL {
                    record.voiding.insert((card.index, form, wait), "void_card");
                }
            }
        }
    }
    card_failures
}

/// 一条臂照第 3 次重跑第一段的算点与判定出行（R1–R5、J1–J11、R3-11 的翻面口径）：点行换名 `name=drift_control_point`，别的行加 `arm=`。
/// 交回这一条臂的全部行（拆成键值，D1–D5 从它们算）。
fn emit_the_analysis_of_the_arm(
    record: &ArmRecord,
    thresholds: &BTreeMap<SubmissionPoint, PointThresholds>,
    machine_ratio: Option<f64>,
    cpu_crc_milliseconds: &BTreeMap<u64, f64>,
    emitter: &mut Emitter,
) -> Vec<BTreeMap<String, String>> {
    let inputs = HostCpuPointInputs {
        thresholds,
        machine_ratio,
        cards: &record.cards,
        medians: &record.medians,
        voiding: &record.voiding,
        process_clock_cells: &record.process_clock_cells,
    };
    let arm_field = format!("arm={}", record.arm.label());
    analysis_lines(&inputs, cpu_crc_milliseconds)
        .into_iter()
        .map(|line| {
            let renamed = match line.strip_prefix("name=host_cpu_per_submission_point ") {
                Some(rest) => format!("name=drift_control_point {arm_field} {rest}"),
                None => match line.split_once(' ') {
                    Some((name, rest)) => format!("{name} {arm_field} {rest}"),
                    None => format!("{line} {arm_field}"),
                },
            };
            emitter.line(&renamed);
            fields_of_a_line(&renamed)
        })
        .collect()
}

/// P-drift（每条臂每张卡）：递减格的第 5 轮 ÷ 第 1 轮 ≤ 0.80；恒定格只报。交回没过的卡。
fn emit_the_drift_probes(record: &ArmRecord, emitter: &mut Emitter) -> BTreeSet<u32> {
    let mut failed = BTreeSet::new();
    for card in &record.cards {
        for probe in DriftProbe::ALL {
            let medians = record
                .probe_medians
                .get(&(*card, probe))
                .cloned()
                .unwrap_or_default();
            let ratio = round_five_over_round_one(&medians);
            let holds = match probe {
                DriftProbe::Decreasing => ratio.is_some_and(probe_drift_holds),
                DriftProbe::Constant => true,
            };
            if !holds {
                failed.insert(*card);
            }
            emitter.line(&format!(
                "name=drift_probe arm={} index={card} probe={} spin_nanoseconds_per_round={} thread_cpu_median_nanoseconds_per_round={} round_five_over_round_one={} judged={} holds={holds}",
                record.arm.label(),
                probe.label(),
                (1..=ROUNDS).map(|round| probe_spin_nanoseconds(probe, round).to_string()).collect::<Vec<_>>().join(","),
                medians.iter().map(|median| optional_number(*median)).collect::<Vec<_>>().join(","),
                optional_number(ratio),
                probe == DriftProbe::Decreasing,
            ));
        }
    }
    failed
}

/// 一条臂一张卡的轮次比两行（点行一级、格一级），交回点行一级的（D1–D3 的值）。
fn emit_the_round_ratios(
    record: &ArmRecord,
    card: u32,
    lines: &[BTreeMap<String, String>],
    emitter: &mut Emitter,
) -> RoundRatios {
    let point_ratios = round_ratios_of_the_point_lines(lines, record.arm, card);
    let cell_ratios = round_ratios_of_the_cells(&record.medians, card);
    for (scope, ratios) in [("point_lines", &point_ratios), ("cells", &cell_ratios)] {
        let (peak, off, last) = ratios.trajectory();
        emitter.line(&format!(
            "name=round_ratio arm={} index={card} scope={scope} lines={} lines_expected={} median_round_two_over_round_one={} median_round_three_over_round_one={} median_round_four_over_round_one={} median_round_five_over_round_one={} round_ratio_peak={} rounds_off_by_more_than_ten_percent={off} final_round_ratio={}",
            record.arm.label(),
            ratios.lines,
            match scope {
                "point_lines" => POINT_LINES_OF_THE_ROUND_RATIO_PER_CARD,
                _ => submission_cells_of_a_card().len(),
            },
            optional_number(ratios.median_over_round_one[0]),
            optional_number(ratios.median_over_round_one[1]),
            optional_number(ratios.median_over_round_one[2]),
            optional_number(ratios.median_over_round_one[3]),
            optional_number(peak),
            optional_number(last),
        ));
    }
    point_ratios
}

/// 一条臂一张卡的频率读数汇总（D6）：每轮开格前读数的中位，格一级 c_rt 与 SM 时钟的秩相关；受控臂另报每格热身次数的轨迹（D7、第八节）。
fn emit_the_clock_summaries(record: &ArmRecord, card: u32, emitter: &mut Emitter) {
    for round in 1..=ROUNDS {
        let readings = record
            .before_readings
            .get(&(card, round))
            .cloned()
            .unwrap_or_default();
        let median_of_the = |value_of: &dyn Fn(&ReadingsBeforeTheCell) -> Option<f64>| {
            let values: Vec<f64> = readings.iter().filter_map(value_of).collect();
            optional_number(median_of(&values))
        };
        emitter.line(&format!(
            "name=clock_round_summary arm={} index={card} round={round} cells={} cells_expected={CELLS_AND_PROBES_PER_ROUND} cells_hold={} scaling_current_frequency_kilohertz_median={} average_frequency_kilohertz_median={} sm_clock_megahertz_median={} memory_clock_megahertz_median={} temperature_celsius_median={} power_watts_median={}",
            record.arm.label(),
            readings.len(),
            readings.len() == CELLS_AND_PROBES_PER_ROUND,
            median_of_the(&|(host, _card)| host.scaling_current_frequency_kilohertz.map(|value| value as f64)),
            median_of_the(&|(host, _card)| host.average_frequency_kilohertz.map(|value| value as f64)),
            median_of_the(&|(_host, card_reading)| card_reading.as_ref()?.sm_clock_megahertz),
            median_of_the(&|(_host, card_reading)| card_reading.as_ref()?.memory_clock_megahertz),
            median_of_the(&|(_host, card_reading)| card_reading.as_ref()?.temperature_celsius),
            median_of_the(&|(_host, card_reading)| card_reading.as_ref()?.power_watts),
        ));
    }
    let pairs = record.clock_pairs.get(&card).cloned().unwrap_or_default();
    emitter.line(&format!(
        "name=clock_correlation arm={} index={card} pairs={} rank_correlation_of_the_thread_cpu_and_the_sm_clock={}",
        record.arm.label(),
        pairs.len(),
        optional_number(rank_correlation(&pairs)),
    ));
    if record.arm == DriftArm::Controlled {
        let counts = record
            .cell_warm_up_submissions
            .get(&card)
            .cloned()
            .unwrap_or_default();
        emitter.line(&format!(
            "name=cell_warm_up_trajectory arm=controlled index={card} cell_rounds={} warm_up_submissions_peak={} cell_rounds_at_the_cap={} last_cell_warm_up_submissions={}",
            counts.len(),
            counts.iter().max().map_or_else(|| "none".to_string(), usize::to_string),
            counts.iter().filter(|count| **count >= CELL_WARM_UP_SUBMISSIONS_AT_MOST).count(),
            counts.last().map_or_else(|| "none".to_string(), usize::to_string),
        ));
    }
}

/// D1–D5 与第十条 L4、L5 的触发（`name=drift_judgement`、`name=unstable_count`、`name=card_order_effect`、`name=failure_clause`）。
/// `probe_failures` 是 P-drift 没过的（臂、卡）：那几个 D 值写 `void_positive_control`。
fn emit_the_drift_judgements(
    reproduction: Option<(&ArmRecord, &[BTreeMap<String, String>])>,
    controlled: Option<(&ArmRecord, &[BTreeMap<String, String>])>,
    round_ratios: &BTreeMap<(DriftArm, u32), RoundRatios>,
    probe_failures: &BTreeSet<(DriftArm, u32)>,
    emitter: &mut Emitter,
) {
    let judged =
        |arm: DriftArm, card: u32, judge: fn(f64) -> &'static str| -> (Option<f64>, &'static str) {
            drift_round_ratio_judgement(
                round_ratios
                    .get(&(arm, card))
                    .and_then(RoundRatios::round_five),
                probe_failures.contains(&(arm, card)),
                judge,
            )
        };
    let controlled_cards = controlled
        .map(|(record, _lines)| record.cards.clone())
        .unwrap_or_default();
    let reproduction_card = reproduction.and_then(|(record, _lines)| record.cards.first().copied());
    let card_zero = controlled_cards.first().copied().or(reproduction_card);
    let card_one = controlled_cards.get(1).copied();
    let mut judgements: BTreeMap<&str, &'static str> = BTreeMap::new();
    for (quantity, arm, card, judge) in [
        (
            "controlled_card_zero_round_ratio",
            DriftArm::Controlled,
            card_zero,
            controlled_round_ratio_judgement as fn(f64) -> &'static str,
        ),
        (
            "reproduction_card_zero_round_ratio",
            DriftArm::Reproduction,
            reproduction_card,
            reproduction_round_ratio_judgement,
        ),
        (
            "controlled_card_one_round_ratio",
            DriftArm::Controlled,
            card_one,
            controlled_round_ratio_judgement,
        ),
    ] {
        let (value, judgement) = card.map_or((None, "not_run"), |index| judged(arm, index, judge));
        judgements.insert(quantity, judgement);
        emitter.line(&format!(
            "name=drift_judgement quantity={quantity} arm={} index={} value={} judgement={judgement}",
            arm.label(),
            card.map_or_else(|| "none".to_string(), |index| index.to_string()),
            optional_number(value),
        ));
    }
    let mut unstable_by_card: BTreeMap<(DriftArm, u32), usize> = BTreeMap::new();
    for (record, lines) in [reproduction, controlled].into_iter().flatten() {
        for card in &record.cards {
            let (count, unstable) = unstable_count(lines, record.arm, *card);
            unstable_by_card.insert((record.arm, *card), unstable);
            emitter.line(&format!(
                "name=unstable_count arm={} index={card} lines={count} unstable={unstable}",
                record.arm.label()
            ));
        }
    }
    let (value, judgement) = match (card_zero, card_one) {
        (Some(zero), Some(one)) => match (
            unstable_by_card.get(&(DriftArm::Controlled, zero)),
            unstable_by_card.get(&(DriftArm::Controlled, one)),
        ) {
            (Some(zero_count), Some(one_count)) => {
                unstable_ratio_judgement(*zero_count, *one_count)
            }
            (None, _) | (_, None) => (None, "not_run"),
        },
        (None, _) | (_, None) => (None, "not_run"),
    };
    emitter.line(&format!(
        "name=drift_judgement quantity=controlled_unstable_card_zero_over_card_one arm=controlled index=none value={} judgement={judgement}",
        optional_number(value),
    ));
    if let Some((record, lines)) = controlled {
        for card in &record.cards {
            let (first, second, ratio) = card_order_effect(lines, &record.cards, *card);
            emitter.line(&format!(
                "name=card_order_effect arm=controlled index={card} first_rounds_median={} second_rounds_median={} ratio={}",
                optional_number(first),
                optional_number(second),
                optional_number(ratio),
            ));
        }
        let warm_ups_reaching_the_cap: Vec<bool> = record
            .cards
            .iter()
            .map(|card| {
                record
                    .warm_ups
                    .get(card)
                    .is_some_and(|warm_up| warm_up.reached_the_cap)
            })
            .collect();
        let card_one_judgement = judgements.get("controlled_card_one_round_ratio").copied();
        emitter.line(&format!(
            "name=failure_clause clause=L4 card_one_judgement={} warm_ups_reaching_the_cap={} triggered={}",
            card_one_judgement.unwrap_or("not_run"),
            warm_ups_reaching_the_cap
                .iter()
                .map(bool::to_string)
                .collect::<Vec<_>>()
                .join(","),
            is_the_controlled_run_drifting_failure_clause_triggered(card_one_judgement, &warm_ups_reaching_the_cap),
        ));
    }
    emitter.line(&format!(
        "name=failure_clause clause=L5 triggered={}",
        is_the_drift_not_reproduced_failure_clause_triggered(
            judgements
                .get("reproduction_card_zero_round_ratio")
                .copied()
        )
    ));
}

/// 停机：打每一条、打判决行、收尾。
fn stop_the_segment(findings: &[String], emitter: &mut Emitter) -> i32 {
    for finding in findings {
        emitter.line(&format!(
            "name=stop_clause segment=host_cpu_drift_control finding={finding}"
        ));
    }
    emitter.line(&format!(
        "name=verdict segment=host_cpu_drift_control stop_clause_findings={}",
        findings.len()
    ));
    emitter.finish();
    EXIT_CODE_STOP_CLAUSE
}

/// 控漂移段（登记第五节「控漂移段怎么量」）：段开头照第 3 次重跑第一段；复现臂只量 `card_indices` 的头一张卡；受控臂绑核、起第二张卡、
/// 两张卡各一遍 P1、暖到稳、5 轮卡交替；两条臂各自算点与判定，再算 D1–D7。
/// 取卡在两条臂起头一张卡之前一刻（`cards_and_pairing_of_the_gpu_segments::card_indices_handed_over_to_the_segment`），之前的自检、校准基线、造单元、
/// 本机 CPU 参照都不持卡。
// 不走先录后判：被测的是同一个进程、同一台机器上主机 CPU 时间与它的漂移（绑核、读频率、暖到稳、每格前后读校准比、两张卡逐轮交替、漂移探针），
// CPU 一侧的读数存进库再由另一次、可能另一台读回就不是被测的那个量
pub(super) fn measure_host_cpu_drift_control_segment(
    stage_split_product: &Path,
    attack_ledger: &Path,
    card_indices: &[u32],
) -> i32 {
    let mut emitter = Emitter::new();
    run_the_segment(
        stage_split_product,
        attack_ledger,
        card_indices,
        &mut emitter,
    )
}

fn run_the_segment(
    stage_split_product: &Path,
    attack_ledger: &Path,
    card_indices: &[u32],
    emitter: &mut Emitter,
) -> i32 {
    let segment_started = Instant::now();
    emitter.line(&format!(
        "name=configuration segment=host_cpu_drift_control cards={} arms={} reproduction_cards={} submission_units={} rounds={ROUNDS} warm_up_submissions={WARM_UP_SUBMISSIONS} cell_warm_up_group_submissions={CELL_WARM_UP_GROUP_SUBMISSIONS} cell_warm_up_submissions_at_most={CELL_WARM_UP_SUBMISSIONS_AT_MOST} warm_up_window_submissions={WARM_UP_WINDOW_SUBMISSIONS} warm_up_seconds_at_least={WARM_UP_SECONDS_AT_LEAST} warm_up_seconds_at_most={WARM_UP_SECONDS_AT_MOST} timed_submissions_without_a_gap={TIMED_SUBMISSIONS_WITHOUT_A_GAP} timed_submissions_with_a_gap={TIMED_SUBMISSIONS_WITH_A_GAP} gap_milliseconds={} {} cells_per_card={} probe_cells_per_round=2 card_quota_mebibytes={CARD_QUOTA_MEBIBYTES} attempts_per_round_at_most={ATTEMPTS_PER_ROUND_AT_MOST} threads_of_the_apparatus=1 threads_of_the_measurement=1",
        card_indices.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
        DriftArm::ALL.map(DriftArm::label).join(","),
        card_indices.first().map_or_else(|| "none".to_string(), u32::to_string),
        SUBMISSION_UNIT_COUNTS.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        GAP_MILLISECONDS.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        sleep_interval_configuration_text(),
        submission_cells_of_a_card().len(),
    ));
    assert_this_process_runs_one_thread("segment_start");
    if !check_the_apparatus_sha256(emitter) {
        return EXIT_CODE_STOP_CLAUSE;
    }
    if !check_the_thread_cpu_clock_resolution(emitter) {
        return stop_for_the_clock_resolution(emitter, "host_cpu_drift_control");
    }
    let mut stop_findings: Vec<String> = Vec::new();
    let constant_mismatches = check_the_constants_and_the_anchors(emitter);
    let SegmentThresholds {
        thresholds,
        peer_baseline,
        anchor_mismatches,
    } = match thresholds_of_the_segment(
        "host_cpu_drift_control",
        stage_split_product,
        attack_ledger,
        emitter,
        &mut stop_findings,
    ) {
        Ok(read) => read,
        Err(exit_code) => return exit_code,
    };
    if constant_mismatches + anchor_mismatches > 0 || !stop_findings.is_empty() {
        stop_findings.push(format!(
            "constant_or_anchor_mismatches_{}",
            constant_mismatches + anchor_mismatches
        ));
        return stop_the_segment(&stop_findings, emitter);
    }
    let mut calibration =
        match SegmentCalibration::start("host_cpu_drift_control", segment_started, emitter) {
            Ok(calibration) => calibration,
            Err(exit_code) => return exit_code,
        };
    let machine_ratio =
        peer_baseline.map(|peer| machine_ratio(calibration.baseline.crc_median_nanoseconds, peer));
    emitter.line(&format!(
        "name=machine_ratio local_baseline_nanoseconds={:.1} peer_baseline_nanoseconds={} ratio={} far_from_one={} status={}",
        calibration.baseline.crc_median_nanoseconds,
        optional_number(peer_baseline),
        optional_number(machine_ratio),
        machine_ratio.is_some_and(is_the_machine_ratio_far_from_one),
        if machine_ratio.is_some() { "measured" } else { "not_run" },
    ));
    let segment_units = segment_units();
    let cpu_crc_milliseconds = cpu_crc_of_the_same_bytes(&segment_units, emitter);
    let threshold_values: Vec<f64> = thresholds
        .values()
        .filter_map(|point| point.by_candidate.get(&GpuCandidate::EveryStage).copied())
        .collect();
    let threshold_edges = (
        threshold_values
            .iter()
            .copied()
            .reduce(f64::min)
            .unwrap_or(0.0),
        threshold_values
            .iter()
            .copied()
            .reduce(f64::max)
            .unwrap_or(0.0),
    );
    let links = super::gpu_submission::pcie_links_reported_by_nvidia_smi();
    let picked_card_indices =
        match cards_and_pairing_of_the_gpu_segments::card_indices_handed_over_to_the_segment(
            card_indices,
        ) {
            Ok(picked) => picked,
            Err(refusal) => {
                return cards_and_pairing_of_the_gpu_segments::stop_without_the_cards_handed_over(
                    "host_cpu_drift_control",
                    &refusal,
                    emitter,
                )
            }
        };
    let mut context = SegmentContext {
        threshold_edges,
        calibration: &mut calibration,
        card_reader: GpuCardReader,
        pending_after: None,
    };
    let mut card_failures: Vec<String> = Vec::new();
    let mut started_cards: Vec<CardOfTheSegment> = Vec::new();
    let mut records: Vec<(ArmRecord, Vec<BTreeMap<String, String>>)> = Vec::new();
    let mut round_ratios: BTreeMap<(DriftArm, u32), RoundRatios> = BTreeMap::new();
    let mut probe_failures: BTreeSet<(DriftArm, u32)> = BTreeSet::new();
    for arm in DriftArm::ALL {
        let wanted: Vec<u32> = match arm {
            DriftArm::Reproduction => picked_card_indices.iter().copied().take(1).collect(),
            DriftArm::Controlled => picked_card_indices.clone(),
        };
        let placement = place_the_thread_of_the_arm(arm);
        emitter.line(&format!(
            "name=arm_placement arm={} pinned_core={} allowed_cores={} scaling_governor={}",
            arm.label(),
            placement
                .pinned_core
                .map_or_else(|| "none".to_string(), |core| core.to_string()),
            placement
                .allowed_cores
                .map_or_else(|| "none".to_string(), |cores| cores.len().to_string()),
            placement
                .scaling_governor
                .unwrap_or_else(|| "none".to_string()),
        ));
        for index in &wanted {
            if started_cards.iter().any(|card| card.index == *index) {
                continue;
            }
            match start_the_card(*index, &links, &segment_units, &mut card_failures, emitter) {
                Ok(Some(card)) => started_cards.push(card),
                Ok(None) => {}
                Err(findings) => return stop_the_segment(&findings, emitter),
            }
        }
        let cards: Vec<&CardOfTheSegment> = wanted
            .iter()
            .filter_map(|index| started_cards.iter().find(|card| card.index == *index))
            .collect();
        if cards.is_empty() {
            emitter.line(&format!(
                "name=arm_verdict arm={} status=not_run reason=no_card_usable",
                arm.label()
            ));
            continue;
        }
        let mut thread_clock_failures = BTreeMap::new();
        for card in &cards {
            thread_clock_failures.insert(
                card.index,
                thread_clock_control_failures(&card.kernels, &card.card, &segment_units, emitter),
            );
        }
        let indices: Vec<u32> = cards.iter().map(|card| card.index).collect();
        let mut record = ArmRecord::new(arm, &indices);
        if let Err(finding) =
            measure_the_arm(&cards, &segment_units, &mut context, &mut record, emitter)
        {
            return stop_the_segment(&[finding], emitter);
        }
        card_failures.extend(void_the_arm(
            &cards,
            &thread_clock_failures,
            &mut record,
            emitter,
        ));
        let lines = emit_the_analysis_of_the_arm(
            &record,
            &thresholds,
            machine_ratio,
            &cpu_crc_milliseconds,
            emitter,
        );
        for card in emit_the_drift_probes(&record, emitter) {
            probe_failures.insert((arm, card));
        }
        for card in &record.cards {
            round_ratios.insert(
                (arm, *card),
                emit_the_round_ratios(&record, *card, &lines, emitter),
            );
            emit_the_clock_summaries(&record, *card, emitter);
        }
        records.push((record, lines));
    }
    let of_the_arm = |arm: DriftArm| {
        records
            .iter()
            .find(|(record, _lines)| record.arm == arm)
            .map(|(record, lines)| (record, lines.as_slice()))
    };
    emit_the_drift_judgements(
        of_the_arm(DriftArm::Reproduction),
        of_the_arm(DriftArm::Controlled),
        &round_ratios,
        &probe_failures,
        emitter,
    );
    finish_the_calibration(context.calibration, emitter);
    let positive_control_failures: usize = records
        .iter()
        .map(|(record, _lines)| {
            record
                .voiding
                .values()
                .filter(|reason| **reason == "void_positive_control")
                .count()
        })
        .sum::<usize>()
        + probe_failures.len();
    emitter.line(&format!(
        "name=verdict segment=host_cpu_drift_control constant_or_anchor_mismatches=0 stop_clause_findings=0 arms_measured={} cards_started={} card_failures={} card_failure_items={} voided_forms_by_the_positive_controls_or_probes={positive_control_failures}",
        records.len(),
        started_cards.len(),
        card_failures.len(),
        if card_failures.is_empty() { "none".to_string() } else { card_failures.join(",") },
    ));
    emitter.finish();
    if records.is_empty() {
        EXIT_CODE_NOT_RUN
    } else if !card_failures.is_empty() || positive_control_failures > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}
