use super::resident_gpu_kernels::{
    untimed_results, ResidentKernels, ResidentSubmission, TodaySubmission,
};
use super::*;
use singlefs_verification::gpu_unit_checks::{gpu_cards_by_priority, GpuCard};

/// P1 每轮基线、甲、乙各量几次；P1、P3 用的 n。
const POSITIVE_CONTROL_SUBMISSIONS: usize = 30;
const POSITIVE_CONTROL_UNITS: u64 = 8;
/// 每格前后的校准：16 MiB 那份 CRC 连做几遍取中位。
const CELL_CALIBRATION_RUNS: usize = 3;
/// 本机同一线程上 CPU 算同一份字节（只报那一行）每档 n 做几遍取中位。
const CPU_CRC_RUNS: usize = 21;
/// 停机 J-S1 对拍用的两档 n。
const KERNEL_ONE_COMPARISON_UNITS: [u64; 2] = [8, 512];

/// 计时区间之外报的本进程线程数（`/proc/self/task` 的条数）；读不到交 `None`。
pub(super) fn threads_of_this_process() -> Option<usize> {
    std::fs::read_dir("/proc/self/task")
        .ok()
        .map(Iterator::count)
}

fn nvidia_smi_text(arguments: &[&str]) -> Option<String> {
    let output = std::process::Command::new("nvidia-smi")
        .args(arguments)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// 开格之前等这张卡上别的计算进程走（登记第十一节「机器被别的活挤」：这时等它，不开格）；等满一小时仍在就停、退 5。
fn wait_until_the_card_has_no_other_compute_process(emitter: &mut Emitter, card_index: u32) {
    let _query_text = wait_until_the_card_has_no_other_compute_process_reading_the_cards(
        emitter,
        card_index,
        "index,pci.bus_id",
        "csv,noheader",
        bus_identifiers_by_index_of,
    );
}

/// 同上，查卡那一次 `nvidia-smi --query-gpu=<query_fields> --format=<format>` 由调用方点栏（第 4 次重跑控漂移段多查六栏频率读数，
/// 并进这一次，不多起进程）；`bus_identifiers_of` 从它的输出取各张卡的总线号。交回最后那一次（没有别的计算进程时）的输出，`nvidia-smi` 读不到交 `None`。
pub(super) fn wait_until_the_card_has_no_other_compute_process_reading_the_cards(
    emitter: &mut Emitter,
    card_index: u32,
    query_fields: &str,
    format: &str,
    bus_identifiers_of: impl Fn(&str) -> BTreeMap<u32, String>,
) -> Option<String> {
    let own_process = std::process::id();
    let query_argument = format!("--query-gpu={query_fields}");
    let format_argument = format!("--format={format}");
    for look in 0..LOOKS_AT_OTHER_MEASUREMENTS_AT_MOST {
        let query_text = nvidia_smi_text(&[query_argument.as_str(), format_argument.as_str()]);
        let bus_identifiers = query_text
            .as_deref()
            .map(&bus_identifiers_of)
            .unwrap_or_default();
        let others = match (
            bus_identifiers.get(&card_index),
            nvidia_smi_text(&[
                "--query-compute-apps=pid,gpu_bus_id",
                "--format=csv,noheader",
            ]),
        ) {
            (Some(bus), Some(apps)) => other_compute_processes_on_the_card(&apps, bus, own_process),
            (None, _) | (_, None) => Vec::new(),
        };
        if others.is_empty() {
            return query_text;
        }
        eprintln!("卡 {card_index} 上有别的计算进程 {others:?}，等它（第 {look} 次看）");
        std::thread::sleep(Duration::from_secs(
            SECONDS_BETWEEN_LOOKS_AT_OTHER_MEASUREMENTS,
        ));
    }
    emitter.line(&format!(
        "name=stopped reason=other_compute_processes_still_on_card_{card_index}_after_an_hour"
    ));
    emitter.finish();
    std::process::exit(EXIT_CODE_OTHER_MEASUREMENT_STILL_RUNNING);
}

/// 每格前后的校准比：16 MiB 那份 CRC 连做 3 遍取中位 ÷ 段开头基线；有一遍读不到或 CRC 不是锚点交回 `None`（读不到，作废）。
pub(super) fn cell_calibration_ratio(buffer: &[u8], baseline_nanoseconds: f64) -> Option<f64> {
    let runs: Vec<f64> = (0..CELL_CALIBRATION_RUNS)
        .map(|_run| {
            one_calibration_run(buffer)
                .filter(|(_nanoseconds, crc)| *crc == CALIBRATION_CRC_ANCHOR)
                .map(|(nanoseconds, _crc)| nanoseconds as f64)
        })
        .collect::<Option<Vec<f64>>>()?;
    Some(calibration_ratio(median_of(&runs)?, baseline_nanoseconds))
}

/// 这一段的单元：真数据单元（造池时写的一个，同区域扫描读盘之后的样子）连着放 512 份、长度表、CPU 算好的期望值。
pub(super) struct SegmentUnits {
    unit_bytes: Vec<u8>,
    lengths: Vec<u32>,
    expected: Vec<u32>,
    unit: Vec<u8>,
}

pub(super) fn segment_units() -> SegmentUnits {
    let unit = super::gpu_submission::one_data_unit_of_the_workload();
    let count = usize::try_from(LARGEST_SUBMISSION_UNITS).expect("512");
    let unit_crc = crc32_castagnoli(&unit);
    SegmentUnits {
        unit_bytes: unit.repeat(count),
        lengths: vec![u32::try_from(unit.len()).expect("32768"); count],
        expected: vec![unit_crc; count],
        unit,
    }
}

impl SegmentUnits {
    fn bytes_of(&self, units: u64) -> &[u8] {
        &self.unit_bytes[..usize::try_from(units * LOCAL_GPU_SLOT_BYTES).expect("字节")]
    }

    fn lengths_of(&self, units: u64) -> &[u32] {
        &self.lengths[..usize::try_from(units).expect("单元数")]
    }

    fn expected_of(&self, units: u64) -> &[u32] {
        &self.expected[..usize::try_from(units).expect("单元数")]
    }
}

/// 本机同一线程上 `crc32_castagnoli` 算 n 个单元的线程钟中位（只报；A 在本机能省下的参照，第十条 J10 的旁证）。
pub(super) fn cpu_crc_of_the_same_bytes(
    units: &SegmentUnits,
    emitter: &mut Emitter,
) -> BTreeMap<u64, f64> {
    let mut milliseconds_by_units = BTreeMap::new();
    for submission_units in SUBMISSION_UNIT_COUNTS {
        let bytes = units.bytes_of(submission_units);
        let runs: Option<Vec<f64>> = (0..CPU_CRC_RUNS)
            .map(|_run| {
                let started = thread_cpu_nanoseconds()?;
                for unit in bytes.chunks(units.unit.len()) {
                    std::hint::black_box(crc32_castagnoli(std::hint::black_box(unit)));
                }
                Some(thread_cpu_nanoseconds()?.saturating_sub(started) as f64)
            })
            .collect();
        let median = runs.as_deref().and_then(median_of);
        emitter.line(&format!(
            "name=cpu_crc_of_the_same_bytes units={submission_units} runs={CPU_CRC_RUNS} thread_cpu_median_nanoseconds={}",
            optional_number(median)
        ));
        if let Some(nanoseconds) = median {
            milliseconds_by_units.insert(submission_units, nanoseconds / 1e6);
        }
    }
    milliseconds_by_units
}

/// 一格的提交步骤：常驻写法的两种输入，或今天的写法。`kernel_two_flip` 只给 P3。
pub(super) fn steps_of<'segment>(
    form: SubmissionForm,
    wait: WaitMethod,
    units: u64,
    kernels: &'segment ResidentKernels,
    card: &'segment GpuCard,
    segment_units: &'segment SegmentUnits,
    kernel_two_flip: Option<usize>,
) -> Box<dyn SubmissionSteps + 'segment> {
    match form {
        SubmissionForm::Resident(input_form) => Box::new(ResidentSubmission {
            kernels,
            form: input_form,
            wait_method: wait,
            unit_bytes: segment_units.bytes_of(units),
            lengths: segment_units.lengths_of(units),
            expected: segment_units.expected_of(units),
            kernel_two_flip,
            submission: None,
            pending: None,
        }),
        SubmissionForm::TodaySingleKernel => Box::new(TodaySubmission {
            card,
            ranges: vec![segment_units.unit.as_slice(); usize::try_from(units).expect("单元数")],
            expected: segment_units.expected_of(units),
            results: Vec::new(),
        }),
    }
}

/// P2 的 9 字节（两种输入各交两核一次）：两核都回 `0xe3069283`。9 字节放进一个 32 KiB 槽（写入要按 4 字节对齐），长度表写 9。
pub(super) fn check_string_holds(
    kernels: &ResidentKernels,
    emitter: &mut Emitter,
    card_index: u32,
) -> bool {
    let mut slot = vec![0_u8; usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768")];
    slot[..CHECK_STRING.len()].copy_from_slice(CHECK_STRING);
    let lengths = [u32::try_from(CHECK_STRING.len()).expect("9")];
    let mut holds = true;
    for form in InputForm::ALL {
        let (results, are_the_sequence_numbers_holding) =
            untimed_results(kernels, form, &slot, &lengths, &[], KernelsToDispatch::Both);
        let both_hold = results
            == [(
                Some(CHECK_STRING_CRC32_CASTAGNOLI),
                Some(CHECK_STRING_CRC32_CASTAGNOLI),
            )];
        holds &= both_hold && are_the_sequence_numbers_holding;
        let crc_text = |crc: Option<u32>| {
            crc.map_or_else(|| "none".to_string(), |value| format!("{value:#010x}"))
        };
        let first = results.first().copied().unwrap_or((None, None));
        emitter.line(&format!(
            "name=positive_control row=check_string index={card_index} form={} kernel_one={} kernel_two={} expected={CHECK_STRING_CRC32_CASTAGNOLI:#010x} sequence_numbers_hold={are_the_sequence_numbers_holding} holds={}",
            form.label(),
            crc_text(first.0),
            crc_text(first.1),
            both_hold && are_the_sequence_numbers_holding
        ));
    }
    holds
}

/// 停机 J-S1：同一份输入（n = 8 与 512；真数据单元连放与按种子造的各不相同的单元两种）交 crates 的 `crc32c_of_ranges` 与核一，逐条比。
pub(super) fn kernel_one_findings(
    kernels: &ResidentKernels,
    card: &GpuCard,
    segment_units: &SegmentUnits,
    emitter: &mut Emitter,
) -> Vec<String> {
    let slot_bytes = usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768");
    let mut distinct =
        vec![0_u8; usize::try_from(LARGEST_SUBMISSION_UNITS).expect("512") * slot_bytes];
    for (unit, slot) in (0_u64..).zip(distinct.chunks_mut(slot_bytes)) {
        disagreement_unit_content(
            u64::from(card.index),
            InputForm::Shared,
            BOTH_CARDS_SEED_BATCH,
            unit,
            slot,
        );
    }
    let mut findings = Vec::new();
    for (content, bytes) in [
        ("workload_unit", segment_units.unit_bytes.as_slice()),
        ("seeded_units", distinct.as_slice()),
    ] {
        for units in KERNEL_ONE_COMPARISON_UNITS {
            let count = usize::try_from(units).expect("单元数");
            let unit_bytes = &bytes[..count * slot_bytes];
            let lengths = vec![u32::try_from(slot_bytes).expect("32768"); count];
            let (results, _sequence) = untimed_results(
                kernels,
                InputForm::Shared,
                unit_bytes,
                &lengths,
                &[],
                KernelsToDispatch::KernelOneOnly,
            );
            let kernel_one: Vec<u32> = results
                .iter()
                .filter_map(|(kernel_one, _)| *kernel_one)
                .collect();
            let ranges: Vec<&[u8]> = unit_bytes.chunks(slot_bytes).collect();
            let crates_results = card.crc32c_of_ranges(&ranges);
            let finding =
                kernel_one_finding_against_the_crates(units, &kernel_one, &crates_results);
            emitter.line(&format!(
                "name=kernel_one_against_the_crates index={} content={content} units={units} holds={}",
                card.index,
                finding.is_none()
            ));
            findings.extend(finding);
        }
    }
    findings
}

/// P3：常驻两种输入、n = 8、阻塞等，一次提交里把核二回读的第 3 条异或 1：比对报恰好 1 条不确定、是第 3 条。交回没过的形态。
pub(super) fn comparison_control_failures(
    kernels: &ResidentKernels,
    card: &GpuCard,
    segment_units: &SegmentUnits,
    emitter: &mut Emitter,
) -> Vec<InputForm> {
    let mut failures = Vec::new();
    for form in InputForm::ALL {
        let mut steps = steps_of(
            SubmissionForm::Resident(form),
            WaitMethod::Blocking,
            POSITIVE_CONTROL_UNITS,
            kernels,
            card,
            segment_units,
            Some(READBACK_ENTRY_FLIPPED_BY_THE_COMPARISON_CONTROL),
        );
        let timing = time_one_submission(steps.as_mut(), &mut ClocksOfThisThread, &mut || {});
        let uncertain = timing
            .as_ref()
            .map(|reading| reading.comparison.uncertain_entries.clone());
        let holds = uncertain.as_deref().is_some_and(comparison_control_holds);
        emitter.line(&format!(
            "name=positive_control row=comparison index={} form={} units={POSITIVE_CONTROL_UNITS} flipped_entry={READBACK_ENTRY_FLIPPED_BY_THE_COMPARISON_CONTROL} uncertain_entries={} holds={holds}",
            card.index,
            form.label(),
            uncertain.map_or_else(|| "unreadable".to_string(), |entries| format!("{entries:?}").replace(' ', "")),
        ));
        if !holds {
            failures.push(form);
        }
    }
    failures
}

/// P1（第 1 行全部 7 种提交形态，每张卡，n = 8、间隔 0）：每轮里基线、甲（按线程钟自旋 2 ms）、乙（睡 2 ms）各 30 次挨着量，
/// 次序按轮号轮换，5 轮；甲的 c_rt 增量中位在 2 ms 的 [0.9, 1.1] 倍，乙的 c_rt 增量中位 ≤ 0.2 ms、挂钟增量中位 ≥ 1.8 ms。交回没过的（形态、等法）。
pub(super) fn thread_clock_control_failures(
    kernels: &ResidentKernels,
    card: &GpuCard,
    segment_units: &SegmentUnits,
    emitter: &mut Emitter,
) -> Vec<(SubmissionForm, WaitMethod)> {
    let insertions = [
        IntervalInsertion::Nothing,
        IntervalInsertion::SpinTwoMilliseconds,
        IntervalInsertion::SleepTwoMilliseconds,
    ];
    let forms = [
        SubmissionForm::Resident(InputForm::Shared),
        SubmissionForm::Resident(InputForm::Split),
        SubmissionForm::TodaySingleKernel,
    ];
    let mut failures = Vec::new();
    for form in forms {
        for wait in form.wait_methods() {
            let mut thread_medians: BTreeMap<IntervalInsertion, Vec<Option<f64>>> = BTreeMap::new();
            let mut wall_medians: BTreeMap<IntervalInsertion, Vec<Option<f64>>> = BTreeMap::new();
            for round in 1..=ROUNDS {
                let mut order = insertions;
                order.rotate_left((round - 1) % insertions.len());
                for insertion in order {
                    let mut steps = steps_of(
                        form,
                        wait,
                        POSITIVE_CONTROL_UNITS,
                        kernels,
                        card,
                        segment_units,
                        None,
                    );
                    let mut insert = || match insertion {
                        IntervalInsertion::Nothing => {}
                        IntervalInsertion::SpinTwoMilliseconds => {
                            let _spun = spin_until_the_thread_cpu_advances(INSERTED_NANOSECONDS);
                        }
                        IntervalInsertion::SleepTwoMilliseconds => {
                            std::thread::sleep(Duration::from_nanos(INSERTED_NANOSECONDS));
                        }
                    };
                    let mut no_sleep = |_duration: Duration| {};
                    let _warm_up = time_the_submissions_of_a_cell(
                        steps.as_mut(),
                        &mut ClocksOfThisThread,
                        WARM_UP_SUBMISSIONS,
                        Duration::ZERO,
                        &mut no_sleep,
                        &mut || {},
                    );
                    let timings = time_the_submissions_of_a_cell(
                        steps.as_mut(),
                        &mut ClocksOfThisThread,
                        POSITIVE_CONTROL_SUBMISSIONS,
                        Duration::ZERO,
                        &mut no_sleep,
                        &mut insert,
                    );
                    let reading = cell_round_reading(&timings, POSITIVE_CONTROL_UNITS);
                    thread_medians.entry(insertion).or_default().push(
                        reading
                            .as_ref()
                            .map(|cell_reading| cell_reading.thread_cpu_median_nanoseconds),
                    );
                    wall_medians.entry(insertion).or_default().push(
                        reading
                            .as_ref()
                            .map(|cell_reading| cell_reading.wall_median_nanoseconds),
                    );
                }
            }
            let increment = |medians: &BTreeMap<IntervalInsertion, Vec<Option<f64>>>, insertion| {
                median_increment_of_the_rounds(
                    &medians[&IntervalInsertion::Nothing],
                    &medians[&insertion],
                )
            };
            let spin_increment = increment(&thread_medians, IntervalInsertion::SpinTwoMilliseconds);
            let sleep_cpu_increment =
                increment(&thread_medians, IntervalInsertion::SleepTwoMilliseconds);
            let sleep_wall_increment =
                increment(&wall_medians, IntervalInsertion::SleepTwoMilliseconds);
            let spin_holds = spin_increment.is_some_and(spin_control_holds);
            let sleep_holds = match (sleep_cpu_increment, sleep_wall_increment) {
                (Some(cpu), Some(wall)) => sleep_control_holds(cpu, wall),
                (None, _) | (_, None) => false,
            };
            emitter.line(&format!(
                "name=positive_control row=thread_clock index={} form={} wait={} units={POSITIVE_CONTROL_UNITS} rounds={ROUNDS} submissions_per_insertion={POSITIVE_CONTROL_SUBMISSIONS} inserted_nanoseconds={INSERTED_NANOSECONDS} spin_increment_median_nanoseconds={} sleep_cpu_increment_median_nanoseconds={} sleep_wall_increment_median_nanoseconds={} spin_holds={spin_holds} sleep_holds={sleep_holds} holds={}",
                card.index,
                form.label(),
                wait.label(),
                optional_number(spin_increment),
                optional_number(sleep_cpu_increment),
                optional_number(sleep_wall_increment),
                spin_holds && sleep_holds,
            ));
            if !(spin_holds && sleep_holds) {
                failures.push((form, wait));
            }
        }
    }
    failures
}

/// 一张卡 80 格 5 轮的读数与这张卡上的作废：P2（每格每条结果与 CPU 相同）、P4（没快过物理下限）、`input_bytes_written` 锚点不符的格轮数，
/// 触发 J5 的格。
struct CellsOfTheCard {
    medians: BTreeMap<SubmissionCell, Vec<Option<CellRoundMedians>>>,
    result_mismatches: usize,
    floor_violations: usize,
    input_byte_mismatches: usize,
    process_clock_cells: BTreeSet<SubmissionCell>,
    /// 第七节甲类不符（⑤ 那一段大过整次，或「确认」+「不确定」≠ n）的（形态、等法）：装置没罩全，照「阳性对照红」作废那一种形态。
    coverage_failures: BTreeSet<(SubmissionForm, WaitMethod)>,
}

/// 一张卡 80 格 5 轮（登记第五节「每轮」）：每轮开头等别的性能测量；每格开格前等这张卡上别的计算进程；每格前校准 → 10 次热身 → 计时提交 →
/// 后校准；校准比有一个 > 1.10 或有一次钟读不到，那一格那一轮作废、等常态（R2-2 同法）再重量，至多 3 次，三次都作废打 `name=round_invalid`。
/// 有一次提交的线程钟大过挂钟（停机 J-S5）交回 `Err`。
#[allow(
    clippy::too_many_arguments,
    reason = "一张卡的量要的各样：卡、两核、单元、校准、链路峰值、门槛两端、产物"
)]
fn measure_the_cells_of_the_card(
    card_index: u32,
    card: &GpuCard,
    kernels: &ResidentKernels,
    segment_units: &SegmentUnits,
    calibration: &mut SegmentCalibration,
    link_peak_bytes_per_second: f64,
    threshold_edges: (f64, f64),
    emitter: &mut Emitter,
) -> Result<CellsOfTheCard, String> {
    let mut cells = CellsOfTheCard {
        medians: submission_cells_of_a_card()
            .into_iter()
            .map(|cell| (cell, vec![None; ROUNDS]))
            .collect(),
        result_mismatches: 0,
        floor_violations: 0,
        input_byte_mismatches: 0,
        process_clock_cells: BTreeSet::new(),
        coverage_failures: BTreeSet::new(),
    };
    let baseline_nanoseconds = calibration.baseline.crc_median_nanoseconds;
    for round in 1..=ROUNDS {
        wait_until_no_other_measurement_runs(emitter);
        for cell in submission_cells_in_the_order_of_the_round(round) {
            wait_until_the_card_has_no_other_compute_process(emitter, card_index);
            for attempt in 1..=ATTEMPTS_PER_ROUND_AT_MOST {
                let ratio_before =
                    cell_calibration_ratio(&calibration.buffer, baseline_nanoseconds);
                let threads_before = threads_of_this_process();
                let mut steps = steps_of(
                    cell.form,
                    cell.wait,
                    cell.units,
                    kernels,
                    card,
                    segment_units,
                    None,
                );
                let gap = Duration::from_millis(cell.gap_milliseconds);
                let mut sleep_between_submissions = |duration: Duration| {
                    if !duration.is_zero() {
                        std::thread::sleep(duration);
                    }
                };
                let _warm_up = time_the_submissions_of_a_cell(
                    steps.as_mut(),
                    &mut ClocksOfThisThread,
                    WARM_UP_SUBMISSIONS,
                    gap,
                    &mut sleep_between_submissions,
                    &mut || {},
                );
                let timed_submissions = if cell.gap_milliseconds == 0 {
                    TIMED_SUBMISSIONS_WITHOUT_A_GAP
                } else {
                    TIMED_SUBMISSIONS_WITH_A_GAP
                };
                let timings = time_the_submissions_of_a_cell(
                    steps.as_mut(),
                    &mut ClocksOfThisThread,
                    timed_submissions,
                    gap,
                    &mut sleep_between_submissions,
                    &mut || {},
                );
                drop(steps);
                let threads_after = threads_of_this_process();
                let ratio_after = cell_calibration_ratio(&calibration.buffer, baseline_nanoseconds);
                let reading = cell_round_reading(&timings, cell.units);
                if let Some(outside) = reading
                    .as_ref()
                    .filter(|cell_reading| !cell_reading.is_every_thread_cpu_inside_the_wall)
                {
                    return Err(format!(
                        "clause_j_s5_thread_cpu_above_the_wall_at_card_{card_index}_{}_round_{round}_median_{:.0}",
                        cell.label().replace([' ', '='], "_"),
                        outside.thread_cpu_median_nanoseconds
                    ));
                }
                let ratios = (ratio_before, ratio_after);
                let verdict = cell_round_verdict(
                    cell,
                    &timings,
                    reading.as_ref(),
                    ratios,
                    link_peak_bytes_per_second,
                );
                cells.floor_violations += verdict.floor_violations;
                cells.result_mismatches +=
                    usize::from(reading.as_ref().is_some_and(|cell_reading| {
                        !cell_reading.are_the_results_holding
                            || !cell_reading.are_the_sequence_numbers_holding
                    }));
                cells.input_byte_mismatches +=
                    usize::from(reading.is_some() && !verdict.input_bytes_hold);
                if reading.as_ref().is_some_and(|cell_reading| {
                    !cell_reading.is_every_wait_inside_the_submission
                        || !cell_reading.are_the_comparison_counts_holding
                }) {
                    cells.coverage_failures.insert((cell.form, cell.wait));
                }
                calibration.larger_ratio_per_intent.push(match ratios {
                    (Some(before), Some(after)) => Some(before.max(after)),
                    (None, _) | (_, None) => None,
                });
                emitter.line(&submission_cell_line(
                    card_index,
                    cell,
                    round,
                    attempt,
                    reading.as_ref(),
                    &verdict,
                    ratios,
                    (threads_before, threads_after),
                    threshold_edges,
                ));
                match (verdict.invalid_reason, reading) {
                    (None, Some(round_reading)) => {
                        if is_the_thread_clock_underestimating(
                            round_reading.process_cpu_median_nanoseconds,
                            round_reading.thread_cpu_median_nanoseconds,
                        ) {
                            cells.process_clock_cells.insert(cell);
                        }
                        if let Some(slot) = cells
                            .medians
                            .get_mut(&cell)
                            .and_then(|rounds| rounds.get_mut(round - 1))
                        {
                            *slot = Some(CellRoundMedians {
                                thread_cpu_nanoseconds: round_reading.thread_cpu_median_nanoseconds,
                                floor_thread_cpu_nanoseconds: round_reading
                                    .floor_thread_cpu_median_nanoseconds,
                                process_cpu_nanoseconds: round_reading
                                    .process_cpu_median_nanoseconds,
                            });
                        }
                        break;
                    }
                    (Some(reason), _) if attempt < ATTEMPTS_PER_ROUND_AT_MOST => {
                        let buffer = &calibration.buffer;
                        let mut calibrate = || cell_calibration_ratio(buffer, baseline_nanoseconds);
                        let mut sleep_on_this_thread = std::thread::sleep;
                        if let Some(wait) = wait_before_the_retry(
                            reason,
                            &mut calibration.waiting,
                            &mut calibrate,
                            &mut sleep_on_this_thread,
                        ) {
                            emitter.line(&format!(
                                "name=normal_wait index={card_index} {} round={round} attempt={attempt} reason={reason} checks={} waited_seconds={} ratio_per_check={} outcome={} segment_waited_seconds={}",
                                cell.label(),
                                wait.checks,
                                wait.waited_seconds,
                                wait.ratio_per_check.iter().map(|ratio| optional_number(*ratio)).collect::<Vec<_>>().join(","),
                                wait.outcome.label(),
                                calibration.waiting.total_seconds,
                            ));
                            if let Some(exit_code) = wait.outcome.exit_code_stopping_the_segment() {
                                emitter.line("name=stopped segment=host_cpu_per_submission reason=machine_not_back_to_normal_within_the_waiting_cap");
                                emitter.finish();
                                std::process::exit(exit_code);
                            }
                        }
                    }
                    (Some(_) | None, _) => {
                        if attempt == ATTEMPTS_PER_ROUND_AT_MOST {
                            emitter.line(&format!(
                                "name=round_invalid segment=host_cpu_per_submission index={card_index} {} round={round} attempts={ATTEMPTS_PER_ROUND_AT_MOST}",
                                cell.label()
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(cells)
}

/// 段开头读到的门槛（每个点）、第二台的校准基线（ρ_机器 的分母）、提交序列锚点对不上的条数。
pub(super) struct SegmentThresholds {
    pub(super) thresholds: BTreeMap<SubmissionPoint, PointThresholds>,
    pub(super) peer_baseline: Option<f64>,
    pub(super) anchor_mismatches: usize,
}

/// 段开头读两份输入（停机 J-S4 核 sha256）、算四个候选的门槛（P7、停机 J-S3）、提交序列的锚点（第七节乙类）与 ρ_机器的分母。
/// 交回（每个点的门槛、第二台的校准基线、对不上的条数）；读不成交回 `Err(退出码)`（已打行、已收尾）。
pub(super) fn thresholds_of_the_segment(
    segment: &str,
    stage_split_product: &Path,
    attack_ledger: &Path,
    emitter: &mut Emitter,
    stop_findings: &mut Vec<String>,
) -> Result<SegmentThresholds, i32> {
    let (Ok(stage_split_bytes), Ok(attack_ledger_bytes)) = (
        std::fs::read(stage_split_product),
        std::fs::read(attack_ledger),
    ) else {
        emitter.line(&format!(
            "name=verdict segment={segment} status=not_run reason=inputs_unreadable"
        ));
        emitter.finish();
        return Err(EXIT_CODE_USAGE);
    };
    for (item, digest, registered, holds) in
        input_file_checks(&attack_ledger_bytes, &stage_split_bytes)
    {
        emitter.line(&format!(
            "name=input_file item={item} sha256={digest} registered_sha256={registered} holds={holds}"
        ));
        if !holds {
            stop_findings.push(format!("clause_j_s4_{item}_sha256_{digest}"));
        }
    }
    let stage_lines = result_lines_of(&String::from_utf8_lossy(&stage_split_bytes));
    let peer_baseline = peer_calibration_baseline_of(&stage_lines);
    let computed = stage_ledger_rows_of(&stage_lines)
        .and_then(|rows| candidate_thresholds_of_the_ledger(&rows));
    let attack_points = attack_ledger_points_of(&String::from_utf8_lossy(&attack_ledger_bytes));
    let (Some(computed), Some(attack_points)) = (computed, attack_points) else {
        emitter.line(&format!(
            "name=verdict segment={segment} status=not_run reason=ledger_unreadable"
        ));
        emitter.finish();
        return Err(EXIT_CODE_USAGE);
    };
    let mut thresholds = BTreeMap::new();
    for point in every_submission_point() {
        let key = (
            point.fraction.label().to_string(),
            point.pool_data_units.to_string(),
            point.batch_units.to_string(),
        );
        let (Some(by_candidate), Some(attack_point)) =
            (computed.get(&key), attack_points.get(&key))
        else {
            stop_findings.push(format!(
                "clause_j_s3_point_missing_{}",
                point.label().replace([' ', '='], "_")
            ));
            continue;
        };
        for (candidate, threshold) in by_candidate {
            emitter.line(&format!(
                "name=candidate_threshold {} candidate={} saved_per_intent_milliseconds={:.6} submissions_per_intent={} threshold_milliseconds={:.6}",
                point.label(),
                candidate.label(),
                threshold.saved_per_intent_milliseconds,
                threshold.submissions_per_intent,
                threshold.threshold_milliseconds,
            ));
        }
        let finding = by_candidate
            .get(&GpuCandidate::EveryStage)
            .and_then(|every_stage| ledger_finding_of_the_point(point, every_stage, attack_point));
        emitter.line(&format!(
            "name=attack_ledger_check {} attack_ledger_threshold_milliseconds={} attack_ledger_submissions_per_intent={} holds={}",
            point.label(),
            attack_point.break_even_text,
            attack_point.submissions_per_intent,
            finding.is_none()
        ));
        stop_findings.extend(finding);
        let mut point_thresholds = PointThresholds {
            by_candidate: by_candidate
                .iter()
                .map(|(candidate, threshold)| (*candidate, threshold.threshold_milliseconds))
                .collect(),
            saved_upper_milliseconds: attack_point.saved_upper_milliseconds,
        };
        point_thresholds.by_candidate.insert(
            GpuCandidate::EveryStage,
            attack_point.break_even_milliseconds,
        );
        thresholds.insert(point, point_thresholds);
    }
    let mut mismatches = 0_usize;
    for (fraction, batch_units, submissions, mean_tenths, release_submissions) in
        SUBMISSION_SEQUENCE_ANCHORS
    {
        let every_stage =
            submission_units_of_the_intent(GpuCandidate::EveryStage, fraction, batch_units);
        let release =
            submission_units_of_the_intent(GpuCandidate::ReleaseCheckOnly, fraction, batch_units);
        let values = [
            (
                "submissions",
                u64::try_from(every_stage.len()).expect("次数"),
                submissions,
            ),
            (
                "mean_submission_tenths_of_bytes",
                mean_submission_tenths_of_bytes(&every_stage),
                mean_tenths,
            ),
            (
                "release_check_submissions",
                u64::try_from(release.len()).expect("次数"),
                release_submissions,
            ),
        ];
        for (item, value, expected) in values {
            let holds = value == expected;
            mismatches += usize::from(!holds);
            emitter.line(&format!(
                "name=anchor item={item}_fraction_{}_batch_units_{batch_units} value={value} expected={expected} holds={holds}",
                fraction.label()
            ));
        }
    }
    Ok(SegmentThresholds {
        thresholds,
        peer_baseline,
        anchor_mismatches: mismatches,
    })
}

/// 段末（没有线程核：起卡之后驱动的线程还在）：21 遍基线（N2 段末）与校准比的轨迹（第八节）。
pub(super) fn finish_the_calibration(calibration: &SegmentCalibration, emitter: &mut Emitter) {
    match calibration_baseline(&calibration.buffer, &calibration.chase_table) {
        Some(end) => emitter.line(&calibration_baseline_line(
            calibration.segment,
            "end",
            &end,
            Some(calibration_ratio(
                end.crc_median_nanoseconds,
                calibration.baseline.crc_median_nanoseconds,
            )),
        )),
        None => emitter.line(&format!(
            "name=calibration_baseline segment={} position=end status=unreadable",
            calibration.segment
        )),
    }
    emitter.line(&calibration_trajectory_line(
        calibration.segment,
        &calibration.larger_ratio_per_intent,
        &calibration.waiting,
    ));
}

/// 第一段（登记第五节「第一段怎么量」）：段开头自检、两份输入与门槛、校准与 P8、ρ_机器、本机 CPU 参照；每张卡起卡、P2、停机 J-S1、P3、
/// P1、80 格 5 轮；两张卡量完算 R1–R5、旋钮翻面与 J10，收尾。
/// 取卡在开设备前一刻（`cards_and_pairing_of_the_gpu_segments::card_indices_handed_over_to_the_segment`），之前的自检、校准基线、造单元、本机 CPU 参照都不持卡。
// 不走先录后判：被测的是同一个进程、同一台机器上 CPU 校准与 GPU 提交交替时主机线程花的 CPU 与挂钟（每格前后读校准比、
// 段开头与段末的校准基线、本机 CPU 参照 `cpu_crc_of_the_same_bytes` 都是这一台这一次的读数），存进库再由另一次、可能另一台读回就不是被测的那个量
pub(super) fn measure_host_cpu_per_submission_segment(
    stage_split_product: &Path,
    attack_ledger: &Path,
    card_indices: &[u32],
) -> i32 {
    let segment_started = Instant::now();
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=configuration segment=host_cpu_per_submission cards={} submission_units={} rounds={ROUNDS} warm_up_submissions={WARM_UP_SUBMISSIONS} timed_submissions_without_a_gap={TIMED_SUBMISSIONS_WITHOUT_A_GAP} timed_submissions_with_a_gap={TIMED_SUBMISSIONS_WITH_A_GAP} gap_milliseconds={} {} cells_per_card={} card_quota_mebibytes={CARD_QUOTA_MEBIBYTES} cell_calibration_runs={CELL_CALIBRATION_RUNS} attempts_per_round_at_most={ATTEMPTS_PER_ROUND_AT_MOST} threads_of_the_apparatus=1",
        card_indices.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
        SUBMISSION_UNIT_COUNTS.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        GAP_MILLISECONDS.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        sleep_interval_configuration_text(),
        submission_cells_of_a_card().len(),
    ));
    assert_this_process_runs_one_thread("segment_start");
    if !check_the_apparatus_sha256(&mut emitter) {
        return EXIT_CODE_STOP_CLAUSE;
    }
    if !check_the_thread_cpu_clock_resolution(&mut emitter) {
        return stop_for_the_clock_resolution(&mut emitter, "host_cpu_per_submission");
    }
    let mut stop_findings: Vec<String> = Vec::new();
    let constant_mismatches = check_the_constants_and_the_anchors(&mut emitter);
    let SegmentThresholds {
        thresholds,
        peer_baseline,
        anchor_mismatches,
    } = match thresholds_of_the_segment(
        "host_cpu_per_submission",
        stage_split_product,
        attack_ledger,
        &mut emitter,
        &mut stop_findings,
    ) {
        Ok(read) => read,
        Err(exit_code) => return exit_code,
    };
    let mismatches = constant_mismatches + anchor_mismatches;
    if mismatches > 0 || !stop_findings.is_empty() {
        for finding in &stop_findings {
            emitter.line(&format!(
                "name=stop_clause segment=host_cpu_per_submission finding={finding}"
            ));
        }
        emitter.line(&format!(
            "name=verdict segment=host_cpu_per_submission constant_or_anchor_mismatches={mismatches} stop_clause_findings={}",
            stop_findings.len()
        ));
        emitter.finish();
        return EXIT_CODE_STOP_CLAUSE;
    }
    let mut calibration =
        match SegmentCalibration::start("host_cpu_per_submission", segment_started, &mut emitter) {
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
    let cpu_crc_milliseconds = cpu_crc_of_the_same_bytes(&segment_units, &mut emitter);
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
    let mut medians: CellMediansOfTheSegment = BTreeMap::new();
    let mut voiding: BTreeMap<(u32, SubmissionForm, WaitMethod), &'static str> = BTreeMap::new();
    let mut process_clock_cells: BTreeSet<(u32, SubmissionCell)> = BTreeSet::new();
    let mut cards_measured = Vec::new();
    let mut card_failures: Vec<String> = Vec::new();
    let picked_card_indices =
        match cards_and_pairing_of_the_gpu_segments::card_indices_handed_over_to_the_segment(
            card_indices,
        ) {
            Ok(picked) => picked,
            Err(refusal) => {
                return cards_and_pairing_of_the_gpu_segments::stop_without_the_cards_handed_over(
                    "host_cpu_per_submission",
                    &refusal,
                    &mut emitter,
                )
            }
        };
    for card_index in &picked_card_indices {
        let Some(card) = gpu_cards_by_priority(&[(*card_index, CARD_QUOTA_MEBIBYTES)])
            .into_iter()
            .next()
        else {
            emitter.line(&format!(
                "name=gpu_card index={card_index} status=not_run reason=card_not_usable"
            ));
            continue;
        };
        let Some(link_peak) = links
            .get(card_index)
            .and_then(|(generation, width)| pcie_link_peak_bytes_per_second(*generation, *width))
        else {
            emitter.line(&format!(
                "name=gpu_card index={card_index} status=invalid reason=pcie_link_unreadable"
            ));
            card_failures.push(format!("card_{card_index}_link_unreadable"));
            continue;
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
        let check_string_holds = check_string_holds(&kernels, &mut emitter, *card_index);
        let kernel_one_findings =
            kernel_one_findings(&kernels, &card, &segment_units, &mut emitter);
        if !kernel_one_findings.is_empty() {
            stop_findings.extend(kernel_one_findings);
            break;
        }
        for form in comparison_control_failures(&kernels, &card, &segment_units, &mut emitter) {
            for wait in WaitMethod::ALL {
                voiding.insert(
                    (*card_index, SubmissionForm::Resident(form), wait),
                    "void_positive_control",
                );
            }
        }
        for (form, wait) in
            thread_clock_control_failures(&kernels, &card, &segment_units, &mut emitter)
        {
            voiding.insert((*card_index, form, wait), "void_positive_control");
        }
        let cells = match measure_the_cells_of_the_card(
            *card_index,
            &card,
            &kernels,
            &segment_units,
            &mut calibration,
            link_peak,
            threshold_edges,
            &mut emitter,
        ) {
            Ok(cells) => cells,
            Err(finding) => {
                stop_findings.push(finding);
                break;
            }
        };
        let is_the_card_valid = check_string_holds
            && cells.result_mismatches == 0
            && cells.floor_violations == 0
            && cells.input_byte_mismatches == 0;
        emitter.line(&format!(
            "name=gpu_card_verdict index={card_index} check_string_holds={check_string_holds} result_mismatching_cell_rounds={} floor_violations={} input_byte_mismatching_cell_rounds={} card_valid={is_the_card_valid}",
            cells.result_mismatches, cells.floor_violations, cells.input_byte_mismatches,
        ));
        if !is_the_card_valid {
            card_failures.push(format!("card_{card_index}_result_or_floor"));
            for form in [
                SubmissionForm::Resident(InputForm::Shared),
                SubmissionForm::Resident(InputForm::Split),
                SubmissionForm::TodaySingleKernel,
            ] {
                for wait in WaitMethod::ALL {
                    voiding.insert((*card_index, form, wait), "void_card");
                }
            }
        }
        medians.extend(
            cells
                .medians
                .into_iter()
                .map(|(cell, rounds)| ((*card_index, cell), rounds)),
        );
        process_clock_cells.extend(
            cells
                .process_clock_cells
                .into_iter()
                .map(|cell| (*card_index, cell)),
        );
        for (form, wait) in &cells.coverage_failures {
            emitter.line(&format!(
                "name=positive_control row=interval_coverage index={card_index} form={} wait={} holds=false",
                form.label(),
                wait.label()
            ));
            voiding.insert((*card_index, *form, *wait), "void_positive_control");
        }
        cards_measured.push(*card_index);
    }
    if !stop_findings.is_empty() {
        for finding in &stop_findings {
            emitter.line(&format!(
                "name=stop_clause segment=host_cpu_per_submission finding={finding}"
            ));
        }
        emitter.line(&format!(
            "name=verdict segment=host_cpu_per_submission constant_or_anchor_mismatches=0 stop_clause_findings={}",
            stop_findings.len()
        ));
        emitter.finish();
        return EXIT_CODE_STOP_CLAUSE;
    }
    let inputs = HostCpuPointInputs {
        thresholds: &thresholds,
        machine_ratio,
        cards: &cards_measured,
        medians: &medians,
        voiding: &voiding,
        process_clock_cells: &process_clock_cells,
    };
    emit_the_analysis(&inputs, &cpu_crc_milliseconds, &mut emitter);
    finish_the_calibration(&calibration, &mut emitter);
    let positive_control_failures = voiding
        .values()
        .filter(|reason| **reason == "void_positive_control")
        .count();
    emitter.line(&format!(
        "name=verdict segment=host_cpu_per_submission constant_or_anchor_mismatches=0 stop_clause_findings=0 cards_measured={} card_failures={} card_failure_items={} voided_forms_by_the_positive_controls={positive_control_failures}",
        cards_measured.len(),
        card_failures.len(),
        if card_failures.is_empty() { "none".to_string() } else { card_failures.join(",") },
    ));
    emitter.finish();
    if cards_measured.is_empty() {
        EXIT_CODE_NOT_RUN
    } else if !card_failures.is_empty() || positive_control_failures > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}
