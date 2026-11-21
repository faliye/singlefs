use super::*;

// ───────────────────────────── 校准（第 2 次重跑登记第六节 N1、N2、N15，第十一节 R2-1、R2-3、R2-4） ─────────────────────────────

/// 校准那块缓冲的字节（N1：16 MiB）。
const CALIBRATION_BUFFER_BYTES: usize = 16 << 20;
/// 校准缓冲第 i 字节 = i mod 251（第七节乙类锚点的算法）。
pub(super) const CALIBRATION_BYTE_MODULUS: usize = 251;
/// 16 MiB 校准缓冲的 CRC-32C（第七节乙类，登记第十三节 python 命令的输出）。
pub(super) const CALIBRATION_CRC_ANCHOR: u32 = 0x3430_BE3B;
/// 加倍那一遍（32 MiB，同法）的 CRC-32C（R2-3）。
const DOUBLED_CALIBRATION_CRC_ANCHOR: u32 = 0x84B6_5AA5;
/// 一次校准 CRC 那份活连做几遍取中位（N1）。
const CALIBRATION_RUNS_PER_READING: usize = 3;
/// 段开头、段末的基线各做几遍取中位（N2）。
const CALIBRATION_RUNS_OF_THE_BASELINE: usize = 21;
/// 一条意图前后两个校准比有一个超过它，那一条意图作废（R2-1；等于不作废）。
const CALIBRATION_RATIO_ABOVE_FOR_VOIDING: f64 = 1.10;
/// 段末基线 ÷ 段开头基线不在这个区间（含两端）就记漂移（R2-4）。
const CALIBRATION_DRIFT_LOWER: f64 = 0.90;
const CALIBRATION_DRIFT_UPPER: f64 = 1.10;
/// 校准自己的阳性对照：加倍的活 ÷ 段开头基线要落在这个区间（含两端，R2-3）。
const CALIBRATION_DOUBLING_RATIO_LOWER: f64 = 1.8;
const CALIBRATION_DOUBLING_RATIO_UPPER: f64 = 2.2;
/// 第八节「新加的轨迹」另报的两条门槛（门槛挪一档会多作废、少作废几条，只报）。
const CALIBRATION_TRAJECTORY_LOWER_EDGE: f64 = 1.05;
const CALIBRATION_TRAJECTORY_UPPER_EDGE: f64 = 1.20;
/// 追指针表的项数（N15：2^24 项 `u32`，64 MiB）。
const CHASE_TABLE_ENTRIES: u64 = 1 << 24;
/// 追指针表第 x 项 = (2654435761 x + 1) mod 2^24。
const CHASE_MULTIPLIER: u64 = 2_654_435_761;
/// 一遍追指针走几步。
const CHASE_STEPS: u64 = 1 << 20;
/// 从 0 走 2^20 步的终点下标（第七节乙类）。
const CHASE_END_INDEX_ANCHOR: u32 = 9_437_184;
/// 一段里校准锚点不符（每一次校准读数算一次）累计超过几次就停、退 3（停机 10）。
const CALIBRATION_ANCHOR_MISMATCHES_PER_SEGMENT_AT_MOST: usize = 3;

/// 校准的缓冲：第 i 字节 = i mod 251。N1 用 16 MiB，R2-3 加倍那一遍用 32 MiB。
fn calibration_buffer(bytes: usize) -> Vec<u8> {
    (0..bytes)
        .map(|position| {
            u8::try_from(position % CALIBRATION_BYTE_MODULUS).expect("余数小于模，装得进 u8")
        })
        .collect()
}

/// 追指针表（N15）。
fn chase_table() -> Vec<u32> {
    (0..CHASE_TABLE_ENTRIES)
        .map(|index| {
            u32::try_from((CHASE_MULTIPLIER * index + 1) % CHASE_TABLE_ENTRIES)
                .expect("模 2^24 的余数装得进 u32")
        })
        .collect()
}

/// 一遍 CRC 那份活：（线程 CPU 纳秒、算出的 CRC）；线程 CPU 时钟读不到交回 `None`。
pub(super) fn one_calibration_run(buffer: &[u8]) -> Option<(u64, u32)> {
    let started = thread_cpu_nanoseconds()?;
    let crc = std::hint::black_box(crc32_castagnoli(std::hint::black_box(buffer)));
    let finished = thread_cpu_nanoseconds()?;
    Some((finished.checked_sub(started)?, crc))
}

/// 一遍追指针：从下标 0 起每步取表里那一项当下一个下标，走 2^20 步；交回（线程 CPU 纳秒、终点下标）。
fn one_chase_run(table: &[u32]) -> Option<(u64, u32)> {
    let table = std::hint::black_box(table);
    let started = thread_cpu_nanoseconds()?;
    let mut index = 0_u32;
    // 迭代上界：CHASE_STEPS 步；跨步的状态只有下标，每一步都依赖上一步取到的那一项，编译器省不掉
    for _step in 0..CHASE_STEPS {
        index = table[usize::try_from(index).expect("u32 装得进 usize")];
    }
    let finished = thread_cpu_nanoseconds()?;
    Some((finished.checked_sub(started)?, std::hint::black_box(index)))
}

/// 一次校准的读数（N1、N15）：CRC 那份活连做 3 遍的中位与追指针一遍；读不到的是 `None`（不当 0）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CalibrationReading {
    pub(super) crc_median_nanoseconds: Option<f64>,
    /// 算出的 CRC 不是 `0x3430be3b` 的遍数（校准的活确实做了、没被优化掉时是 0）。
    crc_runs_off_the_anchor: usize,
    chase_nanoseconds: Option<f64>,
    /// 追指针的终点是 9437184（读不到时不算不符）。
    is_the_chase_end_the_anchor: bool,
}

impl CalibrationReading {
    /// 这一次校准里锚点不符的遍数（停机 10 按遍累计）：CRC 不符的遍数，追指针终点不符再加一遍。
    fn anchor_mismatching_runs(&self) -> usize {
        self.crc_runs_off_the_anchor + usize::from(!self.is_the_chase_end_the_anchor)
    }

    fn are_the_anchors_holding(&self) -> bool {
        self.anchor_mismatching_runs() == 0
    }
}

fn calibration_reading(buffer: &[u8], table: &[u32]) -> CalibrationReading {
    let mut run_nanoseconds: Vec<f64> = Vec::new();
    let mut crc_runs_off_the_anchor = 0_usize;
    for _run in 0..CALIBRATION_RUNS_PER_READING {
        if let Some((nanoseconds, crc)) = one_calibration_run(buffer) {
            run_nanoseconds.push(nanoseconds as f64);
            crc_runs_off_the_anchor += usize::from(crc != CALIBRATION_CRC_ANCHOR);
        }
    }
    let chase = one_chase_run(table);
    CalibrationReading {
        crc_median_nanoseconds: (run_nanoseconds.len() == CALIBRATION_RUNS_PER_READING)
            .then(|| median_of(&run_nanoseconds))
            .flatten(),
        crc_runs_off_the_anchor,
        chase_nanoseconds: chase.map(|(nanoseconds, _end_index)| nanoseconds as f64),
        is_the_chase_end_the_anchor: chase
            .is_none_or(|(_nanoseconds, end_index)| end_index == CHASE_END_INDEX_ANCHOR),
    }
}

/// 校准比：这一次 ÷ 段开头基线（N1、N15）。
pub(super) fn calibration_ratio(this_nanoseconds: f64, baseline_nanoseconds: f64) -> f64 {
    this_nanoseconds / baseline_nanoseconds
}

/// 一个校准比超过 1.10（等于 1.10 不算）。
pub(super) fn is_the_calibration_ratio_above_the_edge(ratio: f64) -> bool {
    ratio > CALIBRATION_RATIO_ABOVE_FOR_VOIDING
}

/// R2-1：一条意图前后两个校准比有一个超过 1.10 就作废。
pub(super) fn is_voided_by_the_calibration_ratios(before_ratio: f64, after_ratio: f64) -> bool {
    is_the_calibration_ratio_above_the_edge(before_ratio)
        || is_the_calibration_ratio_above_the_edge(after_ratio)
}

/// R2-4：段末基线 ÷ 段开头基线（[`calibration_ratio`]）不在 [0.90, 1.10] 里就是漂移了。
fn calibration_drifted(end_to_start_ratio: f64) -> bool {
    !(CALIBRATION_DRIFT_LOWER..=CALIBRATION_DRIFT_UPPER).contains(&end_to_start_ratio)
}

/// R2-3：加倍的活 ÷ 段开头基线（[`calibration_ratio`]）落在 [1.8, 2.2] 里，校准才分得出两倍的活。
fn calibration_doubling_holds(twice_to_once_ratio: f64) -> bool {
    (CALIBRATION_DOUBLING_RATIO_LOWER..=CALIBRATION_DOUBLING_RATIO_UPPER)
        .contains(&twice_to_once_ratio)
}

/// 段开头或段末的基线（N2）：CRC 那份活与追指针各做 21 遍取中位。CRC 那份读不到的交回 `None`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CalibrationBaseline {
    pub(super) crc_median_nanoseconds: f64,
    chase_median_nanoseconds: Option<f64>,
    /// 21 遍里 CRC 与追指针终点都是锚点。
    are_the_anchors_holding: bool,
}

pub(super) fn calibration_baseline(buffer: &[u8], table: &[u32]) -> Option<CalibrationBaseline> {
    let mut crc_nanoseconds: Vec<f64> = Vec::new();
    let mut chase_nanoseconds: Vec<f64> = Vec::new();
    let mut are_the_anchors_holding = true;
    for _run in 0..CALIBRATION_RUNS_OF_THE_BASELINE {
        let (nanoseconds, crc) = one_calibration_run(buffer)?;
        crc_nanoseconds.push(nanoseconds as f64);
        are_the_anchors_holding &= crc == CALIBRATION_CRC_ANCHOR;
        if let Some((chase, end_index)) = one_chase_run(table) {
            chase_nanoseconds.push(chase as f64);
            are_the_anchors_holding &= end_index == CHASE_END_INDEX_ANCHOR;
        }
    }
    Some(CalibrationBaseline {
        crc_median_nanoseconds: median_of(&crc_nanoseconds)?,
        chase_median_nanoseconds: median_of(&chase_nanoseconds),
        are_the_anchors_holding,
    })
}

/// 一条意图前后两次校准的判定（R2-1、停机 10）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CalibrationVerdict {
    Holds,
    /// 前后两个比有一个超过 1.10（R2-1）：作废，等常态再重量（R2-2）。
    RatioAboveTheEdge,
    /// CRC 那份活的钟读不到（原登记第十一节「读不到」）：作废，当场重量。
    Unreadable,
    /// 某一遍的 CRC 或追指针终点不是锚点（停机 10）：作废，当场重量，一段里累计超过 3 次就停。
    AnchorMismatch,
}

/// R2-1 作废的那一条的 `invalid_reason`。
pub(super) const CALIBRATION_RATIO_INVALID_REASON: &str =
    "calibration_ratio_above_the_registered_edge";

impl CalibrationVerdict {
    pub(super) const fn label(self) -> &'static str {
        match self {
            CalibrationVerdict::Holds => "holds",
            CalibrationVerdict::RatioAboveTheEdge => "ratio_above_the_edge",
            CalibrationVerdict::Unreadable => "unreadable",
            CalibrationVerdict::AnchorMismatch => "anchor_mismatch",
        }
    }

    /// 作废的原因（写进 `name=intent` 的 `invalid_reason`）；不作废是 `None`。
    pub(super) const fn invalid_reason(self) -> Option<&'static str> {
        match self {
            CalibrationVerdict::Holds => None,
            CalibrationVerdict::RatioAboveTheEdge => Some(CALIBRATION_RATIO_INVALID_REASON),
            CalibrationVerdict::Unreadable => Some("calibration_unreadable"),
            CalibrationVerdict::AnchorMismatch => Some("calibration_anchor_mismatch"),
        }
    }
}

fn calibration_verdict(
    before: &CalibrationReading,
    after: &CalibrationReading,
    baseline: &CalibrationBaseline,
) -> CalibrationVerdict {
    if !before.are_the_anchors_holding() || !after.are_the_anchors_holding() {
        return CalibrationVerdict::AnchorMismatch;
    }
    match (before.crc_median_nanoseconds, after.crc_median_nanoseconds) {
        (Some(before_nanoseconds), Some(after_nanoseconds)) => {
            if is_voided_by_the_calibration_ratios(
                calibration_ratio(before_nanoseconds, baseline.crc_median_nanoseconds),
                calibration_ratio(after_nanoseconds, baseline.crc_median_nanoseconds),
            ) {
                CalibrationVerdict::RatioAboveTheEdge
            } else {
                CalibrationVerdict::Holds
            }
        }
        (None, _) | (_, None) => CalibrationVerdict::Unreadable,
    }
}

/// 一条意图前后的校准与读数，及其判定。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct IntentCalibration {
    pub(super) before: CalibrationReading,
    after: CalibrationReading,
    snapshot_before: ThreadAndMachineSnapshot,
    snapshot_after: ThreadAndMachineSnapshot,
    pub(super) verdict: CalibrationVerdict,
}

/// 一段的校准状态：缓冲与追指针表（段开头建一次、之后不改）、段开头的基线、锚点不符的累计（停机 10）、等常态的账（R2-2）与 N1 的轨迹。
pub(super) struct SegmentCalibration {
    pub(super) segment: &'static str,
    pub(super) started: Instant,
    pub(super) buffer: Vec<u8>,
    pub(super) chase_table: Vec<u32>,
    pub(super) baseline: CalibrationBaseline,
    clock_ticks_per_second: Option<u64>,
    anchor_mismatches: usize,
    pub(super) waiting: WaitingLedger,
    /// 每条意图前后两个 CRC 校准比里大的那个，按意图的次序（第八节轨迹）；读不到的是 `None`。
    pub(super) larger_ratio_per_intent: Vec<Option<f64>>,
}

/// `name=calibration_baseline` 的一行；段末那一行另带末 ÷ 头与漂移判定（R2-4）。
pub(super) fn calibration_baseline_line(
    segment: &str,
    position: &str,
    baseline: &CalibrationBaseline,
    end_to_start_ratio: Option<f64>,
) -> String {
    let drift_text = end_to_start_ratio.map_or_else(String::new, |ratio| {
        format!(
            " ratio_of_the_end_to_the_start={ratio:.6} drifted={}",
            calibration_drifted(ratio)
        )
    });
    format!(
        "name=calibration_baseline segment={segment} position={position} runs={CALIBRATION_RUNS_OF_THE_BASELINE} median_nanoseconds={:.1} chase_median_nanoseconds={} anchors_hold={}{drift_text}",
        baseline.crc_median_nanoseconds,
        optional_number(baseline.chase_median_nanoseconds),
        baseline.are_the_anchors_holding,
    )
}

impl SegmentCalibration {
    /// 段开头（时钟分辨率自检之后、准备之前）：建缓冲与表，做 21 遍基线（N2），再做一遍加倍的活当校准自己的阳性对照（R2-3）。
    /// 基线读不到或锚点不符（停机 10 收严到基线）时打 `name=stopped`、收尾、交回 `Err(3)`；加倍对照没过时交回 `Err(4)`：那一段一个点都不量。
    pub(super) fn start(
        segment: &'static str,
        started: Instant,
        emitter: &mut Emitter,
    ) -> Result<SegmentCalibration, i32> {
        enter_the_timing_window(TimingWindow::Calibration);
        let buffer = calibration_buffer(CALIBRATION_BUFFER_BYTES);
        let chase_table = chase_table();
        let Some(baseline) = calibration_baseline(&buffer, &chase_table) else {
            emitter.line(&format!(
                "name=stopped segment={segment} reason=calibration_baseline_unreadable"
            ));
            emitter.finish();
            return Err(EXIT_CODE_STOP_CLAUSE);
        };
        emitter.line(&calibration_baseline_line(
            segment, "start", &baseline, None,
        ));
        if !baseline.are_the_anchors_holding {
            emitter.line(&format!(
                "name=stopped segment={segment} reason=calibration_anchor_mismatch_in_the_baseline"
            ));
            emitter.finish();
            return Err(EXIT_CODE_STOP_CLAUSE);
        }
        let doubled_buffer = calibration_buffer(2 * CALIBRATION_BUFFER_BYTES);
        let twice = one_calibration_run(&doubled_buffer);
        drop(doubled_buffer);
        let twice_to_once_ratio = twice.map(|(nanoseconds, _crc)| {
            calibration_ratio(nanoseconds as f64, baseline.crc_median_nanoseconds)
        });
        let is_the_doubled_crc_the_anchor =
            twice.is_some_and(|(_nanoseconds, crc)| crc == DOUBLED_CALIBRATION_CRC_ANCHOR);
        let holds = is_the_doubled_crc_the_anchor
            && twice_to_once_ratio.is_some_and(calibration_doubling_holds);
        emitter.line(&format!(
            "name=positive_control row=calibration segment={segment} twice_nanoseconds={} once_median_nanoseconds={:.1} ratio={} crc_holds={is_the_doubled_crc_the_anchor} holds={holds}",
            optional_count(twice.map(|(nanoseconds, _crc)| nanoseconds)),
            baseline.crc_median_nanoseconds,
            optional_number(twice_to_once_ratio),
        ));
        if !holds {
            emitter.line(&format!(
                "name=stopped segment={segment} reason=calibration_cannot_tell_twice_the_work"
            ));
            emitter.finish();
            return Err(EXIT_CODE_POSITIVE_CONTROL_FAILED);
        }
        Ok(SegmentCalibration {
            segment,
            started,
            buffer,
            chase_table,
            baseline,
            clock_ticks_per_second: clock_ticks_per_second(),
            anchor_mismatches: 0,
            waiting: WaitingLedger {
                total_seconds: 0,
                longest_seconds: 0,
                waits: 0,
            },
            larger_ratio_per_intent: Vec::new(),
        })
    }

    fn crc_ratio_of(&self, reading: &CalibrationReading) -> Option<f64> {
        reading
            .crc_median_nanoseconds
            .map(|nanoseconds| calibration_ratio(nanoseconds, self.baseline.crc_median_nanoseconds))
    }

    fn chase_ratio_of(&self, reading: &CalibrationReading) -> Option<f64> {
        match (
            reading.chase_nanoseconds,
            self.baseline.chase_median_nanoseconds,
        ) {
            (Some(nanoseconds), Some(baseline)) => Some(calibration_ratio(nanoseconds, baseline)),
            (None, _) | (_, None) => None,
        }
    }

    /// 一条意图前后（N1、N3–N9、N15）：校准 → 读数 → 量 → 读数 → 校准，都在调用方进的计时窗口里、在 `measure` 之外。
    pub(super) fn around_the_intent<Measured>(
        &self,
        measure: impl FnOnce() -> Measured,
    ) -> (Measured, IntentCalibration) {
        let before = calibration_reading(&self.buffer, &self.chase_table);
        let snapshot_before = thread_and_machine_snapshot(self.started);
        let measured = measure();
        let snapshot_after = thread_and_machine_snapshot(self.started);
        let after = calibration_reading(&self.buffer, &self.chase_table);
        let verdict = calibration_verdict(&before, &after, &self.baseline);
        (
            measured,
            IntentCalibration {
                before,
                after,
                snapshot_before,
                snapshot_after,
                verdict,
            },
        )
    }

    /// `name=intent_machine_readings` 那一行（键同那条意图的 `name=intent`；N1、N3–N9、N15）。
    fn intent_machine_readings_text(
        &self,
        label: &MeasurementLabel,
        calibration: &IntentCalibration,
    ) -> String {
        let before_ratio = self.crc_ratio_of(&calibration.before);
        let after_ratio = self.crc_ratio_of(&calibration.after);
        let (before, after) = (&calibration.snapshot_before, &calibration.snapshot_after);
        let counter_increase = |pick: fn(&ThreadCounters) -> u64| {
            increase_of(
                before.thread_counters.as_ref().map(pick),
                after.thread_counters.as_ref().map(pick),
            )
        };
        format!(
            "name=intent_machine_readings {} calibration_before_nanoseconds={} calibration_after_nanoseconds={} calibration_before_ratio={} calibration_after_ratio={} chase_before_nanoseconds={} chase_after_nanoseconds={} chase_before_ratio={} chase_after_ratio={} calibration_anchors_hold={} calibration_verdict={} run_queue_wait_nanoseconds={} involuntary_switches={} user_ticks={} system_ticks={} clock_ticks_per_second={} minor_faults={} major_faults={} cpu_pressure_some_ten_seconds_before={} cpu_pressure_some_ten_seconds_after={} memory_pressure_some_ten_seconds_before={} memory_pressure_some_ten_seconds_after={} load_average_one_minute_before={} load_average_one_minute_after={} cpu_number_before={} cpu_number_after={} seconds_since_the_segment_start_before={:.3} seconds_since_the_segment_start_after={:.3}",
            label.text(),
            optional_number(calibration.before.crc_median_nanoseconds),
            optional_number(calibration.after.crc_median_nanoseconds),
            optional_number(before_ratio),
            optional_number(after_ratio),
            optional_number(calibration.before.chase_nanoseconds),
            optional_number(calibration.after.chase_nanoseconds),
            optional_number(self.chase_ratio_of(&calibration.before)),
            optional_number(self.chase_ratio_of(&calibration.after)),
            calibration.before.are_the_anchors_holding() && calibration.after.are_the_anchors_holding(),
            calibration.verdict.label(),
            optional_count(increase_of(before.run_queue_wait_nanoseconds, after.run_queue_wait_nanoseconds)),
            optional_count(increase_of(before.involuntary_switches, after.involuntary_switches)),
            optional_count(counter_increase(|counters| counters.user_ticks)),
            optional_count(counter_increase(|counters| counters.system_ticks)),
            optional_count(self.clock_ticks_per_second),
            optional_count(counter_increase(|counters| counters.minor_faults)),
            optional_count(counter_increase(|counters| counters.major_faults)),
            optional_with_two_decimals(before.cpu_pressure_some_ten_seconds),
            optional_with_two_decimals(after.cpu_pressure_some_ten_seconds),
            optional_with_two_decimals(before.memory_pressure_some_ten_seconds),
            optional_with_two_decimals(after.memory_pressure_some_ten_seconds),
            optional_with_two_decimals(before.load_average_one_minute),
            optional_with_two_decimals(after.load_average_one_minute),
            optional_count(before.cpu_number.map(u64::from)),
            optional_count(after.cpu_number.map(u64::from)),
            before.seconds_since_the_segment_start,
            after.seconds_since_the_segment_start,
        )
    }

    /// 打 `name=intent_machine_readings`（键同那条意图的 `name=intent`）、记 N1 轨迹与锚点不符的次数；一段里锚点不符累计超过 3 次就打
    /// `name=stopped`、收尾、退 3（停机 10：校准那段代码坏了，不是机器忙）。
    pub(super) fn note_the_intent(
        &mut self,
        emitter: &mut Emitter,
        label: &MeasurementLabel,
        calibration: &IntentCalibration,
    ) {
        let before_ratio = self.crc_ratio_of(&calibration.before);
        let after_ratio = self.crc_ratio_of(&calibration.after);
        emitter.line(&self.intent_machine_readings_text(label, calibration));
        self.larger_ratio_per_intent
            .push(match (before_ratio, after_ratio) {
                (Some(before_value), Some(after_value)) => Some(before_value.max(after_value)),
                (None, _) | (_, None) => None,
            });
        self.anchor_mismatches += calibration.before.anchor_mismatching_runs()
            + calibration.after.anchor_mismatching_runs();
        if self.anchor_mismatches > CALIBRATION_ANCHOR_MISMATCHES_PER_SEGMENT_AT_MOST {
            emitter.line(&format!(
                "name=stopped segment={} reason=calibration_anchor_mismatches_above_three_in_the_segment mismatches={}",
                self.segment, self.anchor_mismatches
            ));
            emitter.finish();
            std::process::exit(EXIT_CODE_STOP_CLAUSE);
        }
    }

    /// 一条意图作废之后、重量之前（R2-2）：要等常态的原因就在测量线程上每 10 秒校准一次地等，打一行 `name=normal_wait`；一段里累计等满
    /// 600 秒就打 `name=stopped`、收尾、退 5。别的原因当场重量，不打行。
    pub(super) fn wait_for_the_machine_before_the_retry(
        &mut self,
        emitter: &mut Emitter,
        label: &MeasurementLabel,
        invalid_reason: &str,
    ) {
        if !is_retried_only_after_the_machine_is_back_to_normal(invalid_reason) {
            return;
        }
        enter_the_timing_window(TimingWindow::Calibration);
        let buffer = &self.buffer;
        let chase_table = &self.chase_table;
        let baseline_nanoseconds = self.baseline.crc_median_nanoseconds;
        let mut calibrate = || {
            let reading = calibration_reading(buffer, chase_table);
            if reading.are_the_anchors_holding() {
                reading
                    .crc_median_nanoseconds
                    .map(|nanoseconds| calibration_ratio(nanoseconds, baseline_nanoseconds))
            } else {
                None
            }
        };
        let mut sleep_on_this_thread = std::thread::sleep;
        let Some(wait) = wait_before_the_retry(
            invalid_reason,
            &mut self.waiting,
            &mut calibrate,
            &mut sleep_on_this_thread,
        ) else {
            return;
        };
        emitter.line(&format!(
            "name=normal_wait {} reason={invalid_reason} checks={} waited_seconds={} ratio_per_check={} outcome={} segment_waited_seconds={}",
            label.text(),
            wait.checks,
            wait.waited_seconds,
            wait.ratio_per_check.iter().map(|ratio| optional_number(*ratio)).collect::<Vec<_>>().join(","),
            wait.outcome.label(),
            self.waiting.total_seconds,
        ));
        if let Some(exit_code) = wait.outcome.exit_code_stopping_the_segment() {
            emitter.line(&format!(
                "name=stopped segment={} reason=machine_not_back_to_normal_within_the_waiting_cap segment_waited_seconds={}",
                self.segment, self.waiting.total_seconds
            ));
            emitter.finish();
            std::process::exit(exit_code);
        }
    }

    /// 段末：21 遍基线（N2 段末那一块，带末 ÷ 头与漂移，R2-4）与第八节「新加的轨迹」一行 `name=calibration_trajectory`。
    pub(super) fn finish(&self, emitter: &mut Emitter) {
        enter_the_timing_window(TimingWindow::Calibration);
        match calibration_baseline(&self.buffer, &self.chase_table) {
            Some(end) => emitter.line(&calibration_baseline_line(
                self.segment,
                "end",
                &end,
                Some(calibration_ratio(
                    end.crc_median_nanoseconds,
                    self.baseline.crc_median_nanoseconds,
                )),
            )),
            None => emitter.line(&format!(
                "name=calibration_baseline segment={} position=end status=unreadable",
                self.segment
            )),
        }
        emitter.line(&calibration_trajectory_line(
            self.segment,
            &self.larger_ratio_per_intent,
            &self.waiting,
        ));
    }
}

/// 第八节「新加的轨迹」：N1 前后两个比里大的那个（峰值、超过 1.10 / 1.05 / 1.20 的意图条数、期末值）与等常态的秒数（单次最长、等过的次数、累计）。
pub(super) fn calibration_trajectory_line(
    segment: &str,
    larger_ratio_per_intent: &[Option<f64>],
    waiting: &WaitingLedger,
) -> String {
    let readable: Vec<f64> = larger_ratio_per_intent.iter().flatten().copied().collect();
    let count_above = |edge: f64| readable.iter().filter(|ratio| **ratio > edge).count();
    format!(
        "name=calibration_trajectory segment={segment} intents={} intents_with_a_readable_ratio={} larger_ratio_peak={} intents_above_the_voiding_edge={} voiding_edge={CALIBRATION_RATIO_ABOVE_FOR_VOIDING:.2} intents_above_the_lower_reported_edge={} lower_reported_edge={CALIBRATION_TRAJECTORY_LOWER_EDGE:.2} intents_above_the_upper_reported_edge={} upper_reported_edge={CALIBRATION_TRAJECTORY_UPPER_EDGE:.2} larger_ratio_final={} wait_longest_seconds={} waits={} wait_total_seconds={}",
        larger_ratio_per_intent.len(),
        readable.len(),
        optional_number(readable.iter().copied().reduce(f64::max)),
        readable.iter().filter(|ratio| is_the_calibration_ratio_above_the_edge(**ratio)).count(),
        count_above(CALIBRATION_TRAJECTORY_LOWER_EDGE),
        count_above(CALIBRATION_TRAJECTORY_UPPER_EDGE),
        optional_number(larger_ratio_per_intent.last().copied().flatten()),
        waiting.longest_seconds,
        waiting.waits,
        waiting.total_seconds,
    )
}

#[cfg(test)]
mod tests;
