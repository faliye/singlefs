#![cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "控漂移段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]

use super::*;

/// 控漂移段的两条臂（登记第一节「复现臂」「受控臂」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DriftArm {
    /// 照第 3 次重跑第一段原样量卡 0：不另暖机、每格 10 次热身、不绑核。
    Reproduction,
    /// 暖到稳、每格暖到稳、绑核、读频率、两张卡交替。
    Controlled,
}

impl DriftArm {
    pub(super) const ALL: [DriftArm; 2] = [DriftArm::Reproduction, DriftArm::Controlled];

    pub(super) const fn label(self) -> &'static str {
        match self {
            DriftArm::Reproduction => "reproduction",
            DriftArm::Controlled => "controlled",
        }
    }

    /// 这条臂每格的热身怎么做（第一节受控臂 ②；复现臂照第 3 次重跑每格 10 次）。
    pub(super) const fn cell_warm_up(self) -> CellWarmUp {
        match self {
            DriftArm::Reproduction => CellWarmUp::FixedTenSubmissions,
            DriftArm::Controlled => CellWarmUp::UntilTwoGroupsAgree,
        }
    }
}

// ───── 受控臂 ① 暖到稳：探针格每 20 次一窗，至少 60 秒，连着三窗的中位离三窗均值都不超过 3%，至多 600 秒 ─────

pub(super) const WARM_UP_WINDOW_SUBMISSIONS: usize = 20;
pub(super) const WARM_UP_SECONDS_AT_LEAST: f64 = 60.0;
pub(super) const WARM_UP_SECONDS_AT_MOST: f64 = 600.0;
pub(super) const WARM_UP_WINDOWS_COMPARED: usize = 3;
pub(super) const WARM_UP_WINDOW_TOLERANCE: f64 = 0.03;
/// 暖到稳用的探针格的 n（登记第一节受控臂 ①：n = 512、常驻、共用输入、阻塞等、间隔 0）。
pub(super) const WARM_UP_PROBE_UNITS: u64 = 512;

/// 最近三窗稳了没有：三窗都读得到，三窗的中位各自离三窗均值都不超过 3%。不满三窗不算稳。`window_medians_nanoseconds` 是开暖以来全部的窗。
pub(super) fn are_the_last_windows_stable(window_medians_nanoseconds: &[Option<f64>]) -> bool {
    let Some(start) = window_medians_nanoseconds
        .len()
        .checked_sub(WARM_UP_WINDOWS_COMPARED)
    else {
        return false;
    };
    let Some(last_windows) = window_medians_nanoseconds[start..]
        .iter()
        .copied()
        .collect::<Option<Vec<f64>>>()
    else {
        return false;
    };
    let mean = last_windows.iter().sum::<f64>() / last_windows.len() as f64;
    mean > 0.0
        && last_windows
            .iter()
            .all(|median| ((median - mean) / mean).abs() <= WARM_UP_WINDOW_TOLERANCE)
}

/// 暖到稳的经过（`name=warm_up_to_stable` 那一行与第八节轨迹）。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct WarmUpToStable {
    pub(super) window_medians_nanoseconds: Vec<Option<f64>>,
    pub(super) seconds: f64,
    pub(super) reached_the_cap: bool,
}

impl WarmUpToStable {
    /// 轨迹三样：窗中位的峰值、与前一窗相差超过 3% 的窗数、最后一窗的中位。读不到的窗不算进来。
    pub(super) fn trajectory(&self) -> (Option<f64>, usize, Option<f64>) {
        let read: Vec<f64> = self
            .window_medians_nanoseconds
            .iter()
            .flatten()
            .copied()
            .collect();
        let peak = read.iter().copied().reduce(f64::max);
        let changed = read
            .windows(2)
            .filter(|pair| ((pair[1] - pair[0]) / pair[0]).abs() > WARM_UP_WINDOW_TOLERANCE)
            .count();
        (peak, changed, read.last().copied())
    }
}

/// 暖到稳（登记第一节受控臂 ①）：`run_window` 连发一窗、交回这一窗的线程钟中位（读不到交 `None`，这一窗不算稳），
/// `elapsed_seconds` 交回从开暖起过了几秒。至少 60 秒且最近三窗稳了就停；到 600 秒没稳照停，记 `reached_the_cap`。
/// 钟与提交从参数进来，单测换成假的。
pub(super) fn warm_up_to_stable(
    run_window: &mut dyn FnMut() -> Option<f64>,
    elapsed_seconds: &mut dyn FnMut() -> f64,
) -> WarmUpToStable {
    let mut window_medians_nanoseconds: Vec<Option<f64>> = Vec::new();
    // 迭代上界：elapsed_seconds 单调前进（每窗至少发 20 次提交），到 600 秒必出去；跨圈的状态只有各窗的中位，只增不改
    loop {
        window_medians_nanoseconds.push(run_window());
        let seconds = elapsed_seconds();
        let is_stable = are_the_last_windows_stable(&window_medians_nanoseconds);
        if seconds >= WARM_UP_SECONDS_AT_LEAST && is_stable {
            return WarmUpToStable {
                window_medians_nanoseconds,
                seconds,
                reached_the_cap: false,
            };
        }
        if seconds >= WARM_UP_SECONDS_AT_MOST {
            return WarmUpToStable {
                window_medians_nanoseconds,
                seconds,
                reached_the_cap: true,
            };
        }
    }
}

// ───── 受控臂 ② 每格暖到稳：先 10 次，之后每 10 次一组，相邻两组线程钟中位相差不超过 5% 停，至多 100 次 ─────

pub(super) const CELL_WARM_UP_GROUP_SUBMISSIONS: usize = 10;
pub(super) const CELL_WARM_UP_SUBMISSIONS_AT_MOST: usize = 100;
pub(super) const CELL_WARM_UP_GROUP_TOLERANCE: f64 = 0.05;

/// 每格的热身怎么做。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CellWarmUp {
    /// 第 3 次重跑原样：[`WARM_UP_SUBMISSIONS`] 次。
    FixedTenSubmissions,
    /// 受控臂 ②：相邻两组的线程钟中位相差不超过 5% 停，至多 100 次。
    UntilTwoGroupsAgree,
}

/// 相邻两组近不近：后一组 ÷ 前一组离 1 不超过 5%；有一组读不到不算近。
pub(super) fn are_two_warm_up_groups_close(earlier: Option<f64>, later: Option<f64>) -> bool {
    match (earlier, later) {
        (Some(earlier_median), Some(later_median)) if earlier_median > 0.0 => {
            (later_median / earlier_median - 1.0).abs() <= CELL_WARM_UP_GROUP_TOLERANCE
        }
        (Some(_), Some(_)) | (None, _) | (_, None) => false,
    }
}

/// 一格的热身：`run_group(次数)` 连发这么多次、交回这一组的线程钟中位（读不到交 `None`）。交回一共热身了几次。
/// 两种都先发 [`WARM_UP_SUBMISSIONS`] 次；受控臂之后每 10 次一组接着发。
pub(super) fn warm_up_the_cell(
    policy: CellWarmUp,
    run_group: &mut dyn FnMut(usize) -> Option<f64>,
) -> usize {
    let mut submissions = WARM_UP_SUBMISSIONS;
    let mut earlier = run_group(WARM_UP_SUBMISSIONS);
    match policy {
        CellWarmUp::FixedTenSubmissions => submissions,
        CellWarmUp::UntilTwoGroupsAgree => {
            // 迭代上界：每圈加 10 次，到 100 次必出去；跨圈的状态是上一组的中位
            while submissions < CELL_WARM_UP_SUBMISSIONS_AT_MOST {
                let later = run_group(CELL_WARM_UP_GROUP_SUBMISSIONS);
                submissions += CELL_WARM_UP_GROUP_SUBMISSIONS;
                if are_two_warm_up_groups_close(earlier, later) {
                    break;
                }
                earlier = later;
            }
            submissions
        }
    }
}

// ───── 受控臂 ⑤ 卡交替：第奇数轮先量 cards 里头一张，第偶数轮倒过来 ─────

/// 第 `round` 轮（从 1 起）这条臂按什么次序量卡。复现臂只有一张卡、照给的次序。
pub(super) fn cards_in_the_order_of_the_round(
    arm: DriftArm,
    round: usize,
    cards: &[u32],
) -> Vec<u32> {
    let mut ordered = cards.to_vec();
    match arm {
        DriftArm::Reproduction => {}
        DriftArm::Controlled => {
            if round.is_multiple_of(2) {
                ordered.reverse();
            }
        }
    }
    ordered
}

/// 这张卡在第 `round` 轮是不是先量的（`card_order_effect` 按它分两组）。
pub(super) fn is_the_card_measured_first_in_the_round(
    arm: DriftArm,
    round: usize,
    cards: &[u32],
    card: u32,
) -> bool {
    cards_in_the_order_of_the_round(arm, round, cards).first() == Some(&card)
}

// ───── 漂移探针格（阳性对照 P-drift）：每轮 80 格之后两格，n = 8、常驻、共用输入、阻塞等、间隔 0，计时区间 ① 之前按线程钟自旋 ─────

/// 两格探针。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DriftProbe {
    /// 第 r 轮自旋 8.0 − 1.9 × (r − 1) 毫秒。
    Decreasing,
    /// 每轮 2 毫秒（只报）。
    Constant,
}

impl DriftProbe {
    pub(super) const ALL: [DriftProbe; 2] = [DriftProbe::Decreasing, DriftProbe::Constant];

    pub(super) const fn label(self) -> &'static str {
        match self {
            DriftProbe::Decreasing => "decreasing",
            DriftProbe::Constant => "constant",
        }
    }
}

pub(super) const DECREASING_PROBE_FIRST_ROUND_NANOSECONDS: u64 = 8_000_000;
pub(super) const DECREASING_PROBE_STEP_NANOSECONDS: u64 = 1_900_000;
pub(super) const CONSTANT_PROBE_NANOSECONDS: u64 = 2_000_000;
pub(super) const PROBE_UNITS: u64 = 8;
/// 每条臂每张卡每轮量几格（登记第七节乙类：80 格 + 2 格探针）。
pub(super) const CELLS_AND_PROBES_PER_ROUND: usize = 80 + 2;
/// 轮次比那几行（候选 A、换到本机的门槛、线程钟）每张卡几行（登记第七节乙类：12 个点 × 9 种形态等法间隔）。
pub(super) const POINT_LINES_OF_THE_ROUND_RATIO_PER_CARD: usize = 108;
/// P-drift：递减格的第 5 轮 ÷ 第 1 轮 ≤ 0.80 才算看得出漂移。
pub(super) const PROBE_ROUND_RATIO_AT_MOST: f64 = 0.80;

/// 一格探针在第 `round` 轮（从 1 起）计时区间里自旋多少纳秒。
pub(super) fn probe_spin_nanoseconds(probe: DriftProbe, round: usize) -> u64 {
    match probe {
        DriftProbe::Decreasing => {
            let steps = u64::try_from(round.saturating_sub(1)).expect("轮号装得进 u64");
            DECREASING_PROBE_FIRST_ROUND_NANOSECONDS - DECREASING_PROBE_STEP_NANOSECONDS * steps
        }
        DriftProbe::Constant => CONSTANT_PROBE_NANOSECONDS,
    }
}

/// 探针格的那一格（间隔 0、常驻共用输入、阻塞等、n = 8）。
pub(super) const fn probe_cell() -> SubmissionCell {
    SubmissionCell {
        gap_milliseconds: 0,
        form: SubmissionForm::Resident(InputForm::Shared),
        wait: WaitMethod::Blocking,
        units: PROBE_UNITS,
    }
}

/// 暖到稳用的探针格（n = 512，别的同探针格）。
pub(super) const fn warm_up_probe_cell() -> SubmissionCell {
    probe_cell().with_units(WARM_UP_PROBE_UNITS)
}

/// 第 5 轮 ÷ 第 1 轮；有一轮读不到或第 1 轮是 0 交 `None`。
pub(super) fn round_five_over_round_one(values: &[Option<f64>]) -> Option<f64> {
    let first = (*values.first()?)?;
    let fifth = (*values.get(ROUNDS - 1)?)?;
    (first > 0.0).then(|| fifth / first)
}

/// P-drift 的判定：递减格的比 ≤ 0.80 才过。
pub(super) fn probe_drift_holds(round_five_over_round_one: f64) -> bool {
    round_five_over_round_one <= PROBE_ROUND_RATIO_AT_MOST
}

// ───── D1–D4 的三条线（登记第一节问题单第 1 行、主 agent 定第 1 条：读过第 3 次重跑的 0.528 与 0.958 之后定的） ─────

pub(super) const DRIFT_GONE_LOWER: f64 = 0.90;
pub(super) const DRIFT_GONE_UPPER: f64 = 1.0 / 0.90;
pub(super) const DRIFT_REMAINS_BELOW: f64 = 0.80;
pub(super) const DRIFT_REMAINS_ABOVE: f64 = 1.25;
/// D4：卡 0 ÷ 卡 1 的 `unstable` 数 ≤ 1.5 判卡 0 不比卡 1 更不稳，≥ 2 判更不稳。
pub(super) const UNSTABLE_RATIO_AT_MOST_FOR_NOT_LESS_STABLE: f64 = 1.5;
pub(super) const UNSTABLE_RATIO_AT_LEAST_FOR_LESS_STABLE: f64 = 2.0;

/// D1、D3（受控臂）：在 [0.90, 1 ÷ 0.90] 判漂移消失；< 0.80 或 > 1.25 判仍在；之间不判。
pub(super) fn controlled_round_ratio_judgement(ratio: f64) -> &'static str {
    if (DRIFT_GONE_LOWER..=DRIFT_GONE_UPPER).contains(&ratio) {
        "drift_gone"
    } else if !(DRIFT_REMAINS_BELOW..=DRIFT_REMAINS_ABOVE).contains(&ratio) {
        "drift_remains"
    } else {
        "between"
    }
}

/// D2（复现臂）：≤ 0.80 判复现了；在 [0.90, 1 ÷ 0.90] 判没复现；0.80 与 0.90 之间不判。登记三档之外（> 1 ÷ 0.90）另记一档，不当三档里的哪一档。
pub(super) fn reproduction_round_ratio_judgement(ratio: f64) -> &'static str {
    if ratio <= DRIFT_REMAINS_BELOW {
        "drift_reproduced"
    } else if (DRIFT_GONE_LOWER..=DRIFT_GONE_UPPER).contains(&ratio) {
        "drift_not_reproduced"
    } else if ratio < DRIFT_GONE_LOWER {
        "between"
    } else {
        "above_the_registered_bands"
    }
}

/// D4：两张卡都没有 `unstable` 判 `both_zero`；否则卡 0 ÷ 卡 1（卡 1 是 0 时算无穷大）照两条线判。
pub(super) fn unstable_ratio_judgement(
    card_zero: usize,
    card_one: usize,
) -> (Option<f64>, &'static str) {
    if card_zero == 0 && card_one == 0 {
        return (None, "both_zero");
    }
    let ratio = card_zero as f64 / card_one as f64;
    let judgement = if ratio <= UNSTABLE_RATIO_AT_MOST_FOR_NOT_LESS_STABLE {
        "card_zero_not_less_stable"
    } else if ratio >= UNSTABLE_RATIO_AT_LEAST_FOR_LESS_STABLE {
        "card_zero_less_stable"
    } else {
        "between"
    };
    (Some(ratio), judgement)
}

/// D1–D3 的一个值的判定：那条臂那张卡的 P-drift 没过写 `void_positive_control`（第十一节「阳性对照红」），读不到写 `not_run`，
/// 否则照那一档（`judge`）判。
pub(super) fn drift_round_ratio_judgement(
    value: Option<f64>,
    is_the_probe_failing: bool,
    judge: fn(f64) -> &'static str,
) -> (Option<f64>, &'static str) {
    if is_the_probe_failing {
        return (value, "void_positive_control");
    }
    value.map_or((None, "not_run"), |ratio| (Some(ratio), judge(ratio)))
}

/// 第十条 L4（控住的跑法本身造出漂移）：受控臂卡 1 判 `drift_remains`，或受控臂两张卡暖到稳都到了顶。
pub(super) fn is_the_controlled_run_drifting_failure_clause_triggered(
    card_one_judgement: Option<&str>,
    warm_ups_reaching_the_cap: &[bool],
) -> bool {
    let both_reached_the_cap = warm_ups_reaching_the_cap.len() == 2
        && warm_ups_reaching_the_cap.iter().all(|reached| *reached);
    card_one_judgement == Some("drift_remains") || both_reached_the_cap
}

/// 第十条 L5（这一天复现不出）：复现臂卡 0 判 `drift_not_reproduced`。
pub(super) fn is_the_drift_not_reproduced_failure_clause_triggered(
    reproduction_judgement: Option<&str>,
) -> bool {
    reproduction_judgement == Some("drift_not_reproduced")
}

// ───── 从点行算轮次比与 `unstable` 数（第一节「轮次比」：候选 A、换到本机的门槛、线程钟，每张卡 108 行） ─────

/// 一行产物（没有 `E7RESULT ` 前缀）拆成「键 → 值」。
pub(super) fn fields_of_a_line(line: &str) -> BTreeMap<String, String> {
    line.split(' ')
        .filter_map(|field| field.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// 逗号接起来的各轮读数；`none` 是那一轮读不到。
pub(super) fn per_round_values_of(text: &str) -> Vec<Option<f64>> {
    text.split(',')
        .map(|value| value.parse::<f64>().ok())
        .collect()
}

/// 一条臂一张卡算轮次比用的那几行：`name=drift_control_point`、点名的臂与卡、候选 A、`machine_scaled`、线程钟。
pub(super) fn round_ratio_lines(
    lines: &[BTreeMap<String, String>],
    arm: DriftArm,
    card: u32,
) -> Vec<&BTreeMap<String, String>> {
    let card_text = card.to_string();
    lines
        .iter()
        .filter(|fields| {
            let is = |key: &str, value: &str| fields.get(key).map(String::as_str) == Some(value);
            is("name", "drift_control_point")
                && is("arm", arm.label())
                && is("index", &card_text)
                && is("candidate", GpuCandidate::ScanChecksumOnly.label())
                && is("threshold_kind", ThresholdKind::MachineScaled.label())
                && is("clock", CpuClockKind::Thread.label())
        })
        .collect()
}

/// 轮次比（`name=round_ratio`）：每行第 k 轮 ÷ 第 1 轮（k = 2–5）取中位；两轮有一轮读不到的行那一个比不收。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct RoundRatios {
    pub(super) lines: usize,
    /// 第 2、3、4、5 轮 ÷ 第 1 轮的中位，次序就是轮号。
    pub(super) median_over_round_one: [Option<f64>; 4],
}

impl RoundRatios {
    pub(super) fn round_five(&self) -> Option<f64> {
        self.median_over_round_one[3]
    }

    /// 第八节轨迹：五轮里最大的比（第 1 轮算 1）、比落在 [0.90, 1 ÷ 0.90] 之外的轮数、期末值（第 5 轮）。
    pub(super) fn trajectory(&self) -> (Option<f64>, usize, Option<f64>) {
        let read: Vec<f64> = self
            .median_over_round_one
            .iter()
            .flatten()
            .copied()
            .collect();
        let peak = read.iter().copied().fold(1.0_f64, f64::max);
        let off = read
            .iter()
            .filter(|ratio| !(DRIFT_GONE_LOWER..=DRIFT_GONE_UPPER).contains(*ratio))
            .count();
        ((!read.is_empty()).then_some(peak), off, self.round_five())
    }
}

/// 一串各行的各轮读数算轮次比。
pub(super) fn round_ratios_of(per_round_of_each_line: &[Vec<Option<f64>>]) -> RoundRatios {
    let median_of_round = |round_index: usize| {
        let ratios: Vec<f64> = per_round_of_each_line
            .iter()
            .filter_map(|values| {
                let first = (*values.first()?)?;
                let later = (*values.get(round_index)?)?;
                (first > 0.0).then(|| later / first)
            })
            .collect();
        median_of(&ratios)
    };
    RoundRatios {
        lines: per_round_of_each_line.len(),
        median_over_round_one: [1, 2, 3, 4].map(median_of_round),
    }
}

/// 点行一级的轮次比（D1–D3 的值）：那几行的 `host_cpu_per_submission_milliseconds_per_round`。
pub(super) fn round_ratios_of_the_point_lines(
    lines: &[BTreeMap<String, String>],
    arm: DriftArm,
    card: u32,
) -> RoundRatios {
    let per_round: Vec<Vec<Option<f64>>> = round_ratio_lines(lines, arm, card)
        .into_iter()
        .map(|fields| {
            fields
                .get("host_cpu_per_submission_milliseconds_per_round")
                .map_or_else(Vec::new, |text| per_round_values_of(text))
        })
        .collect();
    round_ratios_of(&per_round)
}

/// D4：一条臂一张卡全部 `name=drift_control_point` 行的 `judgement` 列里 `unstable` 几行（口径同数 1054 的那一列）；交回（行数、`unstable` 数）。
pub(super) fn unstable_count(
    lines: &[BTreeMap<String, String>],
    arm: DriftArm,
    card: u32,
) -> (usize, usize) {
    let card_text = card.to_string();
    let of_the_card: Vec<&BTreeMap<String, String>> = lines
        .iter()
        .filter(|fields| {
            fields.get("name").map(String::as_str) == Some("drift_control_point")
                && fields.get("arm").map(String::as_str) == Some(arm.label())
                && fields.get("index") == Some(&card_text)
        })
        .collect();
    let unstable = of_the_card
        .iter()
        .filter(|fields| fields.get("judgement").map(String::as_str) == Some("unstable"))
        .count();
    (of_the_card.len(), unstable)
}

/// 格一级的轮次比（`name=submission_cell` 的线程钟中位，每张卡 80 格）。
pub(super) fn round_ratios_of_the_cells(
    medians: &CellMediansOfTheSegment,
    card: u32,
) -> RoundRatios {
    let per_round: Vec<Vec<Option<f64>>> = medians
        .iter()
        .filter(|((card_index, _cell), _rounds)| *card_index == card)
        .map(|(_key, rounds)| {
            rounds
                .iter()
                .map(|round| {
                    round.map(|medians_of_the_round| medians_of_the_round.thread_cpu_nanoseconds)
                })
                .collect()
        })
        .collect();
    round_ratios_of(&per_round)
}

/// 受控臂一张卡先量的轮与后量的轮（第八节「几何敏感性」最后一行，只报）：轮次比那几行里每行先量那几轮的平均、后量那几轮的平均，
/// 各取中位；交回（先量的中位、后量的中位、后 ÷ 先）。
pub(super) fn card_order_effect(
    lines: &[BTreeMap<String, String>],
    cards: &[u32],
    card: u32,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let mean_of = |values: &[Option<f64>], first: bool| {
        let chosen: Vec<f64> = values
            .iter()
            .enumerate()
            .filter(|(round_index, _value)| {
                is_the_card_measured_first_in_the_round(
                    DriftArm::Controlled,
                    round_index + 1,
                    cards,
                    card,
                ) == first
            })
            .filter_map(|(_round_index, value)| *value)
            .collect();
        (!chosen.is_empty()).then(|| chosen.iter().sum::<f64>() / chosen.len() as f64)
    };
    let mut first_rounds = Vec::new();
    let mut second_rounds = Vec::new();
    for fields in round_ratio_lines(lines, DriftArm::Controlled, card) {
        let values = fields
            .get("host_cpu_per_submission_milliseconds_per_round")
            .map_or_else(Vec::new, |text| per_round_values_of(text));
        first_rounds.extend(mean_of(&values, true));
        second_rounds.extend(mean_of(&values, false));
    }
    let first_median = median_of(&first_rounds);
    let second_median = median_of(&second_rounds);
    let ratio = first_median
        .zip(second_median)
        .filter(|(first, _second)| *first > 0.0)
        .map(|(first, second)| second / first);
    (first_median, second_median, ratio)
}

/// 秩相关（Spearman）：两列各自排秩（并列取平均秩），秩的皮尔逊相关；不到 3 对或有一列没有变化交 `None`。
pub(super) fn rank_correlation(pairs: &[(f64, f64)]) -> Option<f64> {
    if pairs.len() < 3 {
        return None;
    }
    let ranks_of = |values: Vec<f64>| {
        let mut order: Vec<usize> = (0..values.len()).collect();
        order.sort_by(|left, right| values[*left].total_cmp(&values[*right]));
        let mut ranks = vec![0.0_f64; values.len()];
        let mut start = 0;
        // 迭代上界：每圈 start 至少前进 1（一组并列至少一个）
        while start < order.len() {
            let mut end = start + 1;
            while end < order.len() && values[order[end]] == values[order[start]] {
                end += 1;
            }
            let shared_rank = (start + end + 1) as f64 / 2.0;
            for position in &order[start..end] {
                ranks[*position] = shared_rank;
            }
            start = end;
        }
        ranks
    };
    let first = ranks_of(pairs.iter().map(|(value, _other)| *value).collect());
    let second = ranks_of(pairs.iter().map(|(_value, other)| *other).collect());
    let count = pairs.len() as f64;
    let mean_first = first.iter().sum::<f64>() / count;
    let mean_second = second.iter().sum::<f64>() / count;
    let covariance: f64 = first
        .iter()
        .zip(&second)
        .map(|(left, right)| (left - mean_first) * (right - mean_second))
        .sum();
    let spread_first: f64 = first.iter().map(|left| (left - mean_first).powi(2)).sum();
    let spread_second: f64 = second
        .iter()
        .map(|right| (right - mean_second).powi(2))
        .sum();
    (spread_first > 0.0 && spread_second > 0.0)
        .then(|| covariance / (spread_first * spread_second).sqrt())
}

// ───── 受控臂 ④ 频率读数：每格前后读那个核的两项与这张卡的六项（并进本来就起的那一次 nvidia-smi），计时窗口外 ─────

/// 开格前那一次 `nvidia-smi` 查的栏（头两栏同第 3 次重跑的 `index,pci.bus_id`，后面六栏是这一趟加的读数）。
pub(super) const CARD_QUERY_FIELDS: &str =
    "index,pci.bus_id,clocks.sm,clocks.mem,pstate,temperature.gpu,power.draw,clocks_event_reasons.active";

/// 一张卡此刻的读数；读不到的那一栏是 `None`（产物里写 `none`，不作废那一格，登记第十一节「读不到」）。
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct CardClockReading {
    pub(super) bus_identifier: String,
    pub(super) sm_clock_megahertz: Option<f64>,
    pub(super) memory_clock_megahertz: Option<f64>,
    pub(super) performance_state: Option<String>,
    pub(super) temperature_celsius: Option<f64>,
    pub(super) power_watts: Option<f64>,
    pub(super) clocks_event_reasons: Option<String>,
}

/// `nvidia-smi --query-gpu=<CARD_QUERY_FIELDS> --format=csv,noheader,nounits` 的输出，按卡的序号；栏数不对的行不收。
pub(super) fn card_clock_readings_of(query_text: &str) -> BTreeMap<u32, CardClockReading> {
    let number = |text: &str| text.trim().parse::<f64>().ok();
    let word = |text: &str| {
        let trimmed = text.trim();
        (!trimmed.is_empty() && !trimmed.starts_with('[') && trimmed != "N/A")
            .then(|| field_token(trimmed))
    };
    query_text
        .lines()
        .filter_map(|line| {
            let columns: Vec<&str> = line.split(',').collect();
            let [index, bus, sm, memory, state, temperature, power, reasons] = columns.as_slice()
            else {
                return None;
            };
            Some((
                index.trim().parse().ok()?,
                CardClockReading {
                    bus_identifier: bus.trim().to_ascii_lowercase(),
                    sm_clock_megahertz: number(sm),
                    memory_clock_megahertz: number(memory),
                    performance_state: word(state),
                    temperature_celsius: number(temperature),
                    power_watts: number(power),
                    clocks_event_reasons: word(reasons),
                },
            ))
        })
        .collect()
}

/// 发起提交那条线程此刻所在的核与它的两项频率（`sysfs` 读两个文件，不起进程）；读不到的是 `None`。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HostCoreReading {
    pub(super) core: Option<usize>,
    pub(super) scaling_current_frequency_kilohertz: Option<u64>,
    pub(super) average_frequency_kilohertz: Option<u64>,
}

// `libc::sched_setaffinity` 照 glibc：`int sched_setaffinity(pid_t, size_t, const cpu_set_t *)`；
// 掩码在这里当 16 个 u64 存，传进去时把指针转成 `libc::cpu_set_t`。

/// `cpu_set_t` 的大小：1024 位，16 个 u64。
const AFFINITY_MASK_WORDS: usize = 16;
const BITS_PER_AFFINITY_WORD: usize = 64;
// 把 16 个 u64 的掩码当 `libc::cpu_set_t` 传，靠的是两边一样大、`cpu_set_t` 的对齐不比 u64 严；对不上编不过。
const _: () = assert!(
    std::mem::size_of::<libc::cpu_set_t>() == AFFINITY_MASK_WORDS * std::mem::size_of::<u64>()
        && std::mem::align_of::<libc::cpu_set_t>() <= std::mem::align_of::<u64>()
);

fn cpufreq_file_of(core: usize, file: &str) -> Option<u64> {
    std::fs::read_to_string(format!("/sys/devices/system/cpu/cpu{core}/cpufreq/{file}"))
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// 此刻这条线程在哪个核上、那个核的 `scaling_cur_freq` 与 `cpuinfo_avg_freq`。
pub(super) fn host_core_reading_now() -> HostCoreReading {
    // 核号照快照的读法取：读一次本线程的 `stat`、取第 39 栏。
    let core = parsed_value_of_the_file(
        Path::new("/proc/thread-self/stat"),
        cpu_number_of_the_counter_record,
    )
    .and_then(|number| usize::try_from(number).ok());
    HostCoreReading {
        core,
        scaling_current_frequency_kilohertz: core
            .and_then(|index| cpufreq_file_of(index, "scaling_cur_freq")),
        average_frequency_kilohertz: core
            .and_then(|index| cpufreq_file_of(index, "cpuinfo_avg_freq")),
    }
}

/// 内核写核集合的区间列表（`Cpus_allowed_list` 与 `/sys/devices/system/cpu/online` 都是这种写法：`0-3,8,10-11`）拆成闭区间，
/// 次序照原文；有一段认不得（不是数、起点比终点大、空串）交回 `None`。
pub(super) fn core_ranges_of_the_range_list(
    range_list: &str,
) -> Option<Vec<std::ops::RangeInclusive<usize>>> {
    range_list
        .trim()
        .split(',')
        .map(|range_text| match range_text.trim().split_once('-') {
            Some((first_core_text, last_core_text)) => {
                let first_core: usize = first_core_text.parse().ok()?;
                let last_core: usize = last_core_text.parse().ok()?;
                (first_core <= last_core).then_some(first_core..=last_core)
            }
            None => {
                let single_core: usize = range_text.trim().parse().ok()?;
                Some(single_core..=single_core)
            }
        })
        .collect()
}

/// `/proc/<…>/status` 里 `Cpus_allowed_list:` 那一行冒号之后的区间列表（不是十六进制掩码 `Cpus_allowed:` 那一行）。
pub(super) fn allowed_core_list_of_the_status(status_text: &str) -> Option<&str> {
    status_text
        .lines()
        .find_map(|line| line.strip_prefix("Cpus_allowed_list:"))
}

/// 两份区间列表都有的核，从小到大、不重复；有一份认不得交回 `None`。
pub(super) fn cores_in_both_range_lists(
    allowed_range_list: &str,
    online_range_list: &str,
) -> Option<Vec<usize>> {
    let allowed_ranges = core_ranges_of_the_range_list(allowed_range_list)?;
    let online_ranges = core_ranges_of_the_range_list(online_range_list)?;
    let mut cores: Vec<usize> = allowed_ranges
        .into_iter()
        .flatten()
        .filter(|core| {
            online_ranges
                .iter()
                .any(|online_range| online_range.contains(core))
        })
        .collect();
    cores.sort_unstable();
    cores.dedup();
    Some(cores)
}

/// 本线程此刻允许跑的核：`/proc/thread-self/status` 的 `Cpus_allowed_list` 与 `/sys/devices/system/cpu/online` 求交。
/// `Cpus_allowed_list` 是亲和掩码原样，有核下线时也列着它；`sched_getaffinity` 交回的是掩码与活动核的交（不在热插拔途中时活动核就是在线核），
/// 这里照那个交算。
/// 两份有一份读不到或认不得交 `None`。
pub(super) fn allowed_cores_of_this_thread() -> Option<Vec<usize>> {
    let status_text = std::fs::read_to_string("/proc/thread-self/status").ok()?;
    let online_range_list = std::fs::read_to_string("/sys/devices/system/cpu/online").ok()?;
    cores_in_both_range_lists(
        allowed_core_list_of_the_status(&status_text)?,
        &online_range_list,
    )
}

/// 把本线程绑到一个核上；交回绑成了没有。
#[allow(
    unsafe_code,
    reason = "std 没有把线程绑到核上的接口，只能经 libc 调 sched_setaffinity(2)；已有依赖没有安全封装，起 taskset 子进程要先拿本线程的线程号，std 也不给"
)]
pub(super) fn pin_this_thread_to(core: usize) -> bool {
    if core >= AFFINITY_MASK_WORDS * BITS_PER_AFFINITY_WORD {
        return false;
    }
    let mut mask = [0_u64; AFFINITY_MASK_WORDS];
    mask[core / BITS_PER_AFFINITY_WORD] |= 1_u64 << (core % BITS_PER_AFFINITY_WORD);
    // SAFETY: mask 是这个栈帧里活着的 128 字节，内核只读；进程号 0 是调用的这条线程；
    // 指针转成 `cpu_set_t` 靠上面那条编译期断言：两边大小相同、`cpu_set_t` 的对齐不比 u64 严。
    let status = unsafe {
        libc::sched_setaffinity(
            0,
            std::mem::size_of_val(&mask),
            mask.as_ptr().cast::<libc::cpu_set_t>(),
        )
    };
    status == 0
}

/// 绑哪个核（第一节受控臂 ③）：允许集合里 `amd_pstate_prefcore_ranking` 最高的，并列取号小的；一个都读不到取允许集合里号最小的非 0 核
/// （只有 0 号核时取 0）。`rankings` 是（核号、排名，读不到是 `None`）。
pub(super) fn preferred_core(
    rankings: &[(usize, Option<u64>)],
    allowed: &[usize],
) -> Option<usize> {
    let ranked = rankings
        .iter()
        .filter(|(core, _ranking)| allowed.contains(core))
        .filter_map(|(core, ranking)| ranking.map(|value| (*core, value)))
        .max_by(|left, right| left.1.cmp(&right.1).then(right.0.cmp(&left.0)));
    ranked.map(|(core, _ranking)| core).or_else(|| {
        allowed
            .iter()
            .copied()
            .filter(|core| *core != 0)
            .min()
            .or_else(|| allowed.first().copied())
    })
}

/// 一条臂的线程放在哪（`name=arm_placement`）：绑的核（复现臂不绑）、那个核的 `scaling_governor`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ArmThreadPlacement {
    pub(super) pinned_core: Option<usize>,
    pub(super) allowed_cores: Option<Vec<usize>>,
    pub(super) scaling_governor: Option<String>,
}

/// 开一条臂之前放好这条线程：受控臂绑到 [`preferred_core`]，复现臂照旧（不绑，允许集合原样）。
pub(super) fn place_the_thread_of_the_arm(arm: DriftArm) -> ArmThreadPlacement {
    let allowed = allowed_cores_of_this_thread();
    let pinned_core = match arm {
        DriftArm::Reproduction => None,
        DriftArm::Controlled => allowed.as_deref().and_then(|cores| {
            let rankings: Vec<(usize, Option<u64>)> = cores
                .iter()
                .map(|core| (*core, cpufreq_file_of(*core, "amd_pstate_prefcore_ranking")))
                .collect();
            preferred_core(&rankings, cores).filter(|core| pin_this_thread_to(*core))
        }),
    };
    let governor_core = pinned_core.or_else(|| host_core_reading_now().core);
    ArmThreadPlacement {
        pinned_core,
        allowed_cores: allowed_cores_of_this_thread(),
        scaling_governor: governor_core.and_then(|core| {
            std::fs::read_to_string(format!(
                "/sys/devices/system/cpu/cpu{core}/cpufreq/scaling_governor"
            ))
            .ok()
            .map(|text| field_token(text.trim()))
        }),
    }
}

// ───── 一格一轮（照第 3 次重跑的格：前校准 → 热身 → 计时提交 → 后校准，作废与重试照 R2-2）；卡与钟从参数进来，单测换成假的 ─────

/// 一格一轮在一张卡上要的东西：真卡（gpu 特性）与单测的假卡各一份。
pub(super) trait CellRoundEnvironment {
    fn card_index(&self) -> u32;
    fn link_peak_bytes_per_second(&self) -> f64;
    fn host_core_reading(&mut self) -> HostCoreReading;
    /// 16 MiB 那份 CRC 的校准比（读不到交 `None`，那一格那一轮作废）。
    fn calibration_ratio(&mut self) -> Option<f64>;
    fn threads(&mut self) -> Option<usize>;
    /// 开这一格的提交步骤（第 3 次重跑的 `steps_of`），热身与计时提交用同一份。
    fn open_the_cell(&mut self, cell: SubmissionCell);
    /// 用开着的那一格连发 `count` 次（第 3 次重跑的 `time_the_submissions_of_a_cell`，间隔睡在区间外）。
    fn time_the_submissions(
        &mut self,
        count: usize,
        gap: Duration,
        insertion: &mut dyn FnMut(),
    ) -> Vec<Option<SubmissionTiming>>;
    fn close_the_cell(&mut self);
}

/// 开格前读卡（计时窗口外）：等这张卡上别的计算进程走，交回这一次读到的各张卡的读数（第一节受控臂 ④，并进本来就起的那一次 `nvidia-smi`）。
pub(super) trait CardReader {
    fn wait_for_the_card_and_read_the_cards(
        &mut self,
        card_index: u32,
        emitter: &mut Emitter,
    ) -> BTreeMap<u32, CardClockReading>;
}

/// 一格一轮的要求。
#[derive(Clone, Copy, Debug)]
pub(super) struct CellRoundRequest {
    pub(super) arm: DriftArm,
    pub(super) cell: SubmissionCell,
    pub(super) round: usize,
    pub(super) probe: Option<DriftProbe>,
    pub(super) threshold_edges: (f64, f64),
}

impl CellRoundRequest {
    /// 产物行里这一格的头：`name=<submission_cell 或 drift_probe_cell> arm=<臂>[ probe=<探针>]`。
    pub(super) fn line_head(&self) -> String {
        match self.probe {
            None => format!("submission_cell arm={}", self.arm.label()),
            Some(probe) => format!(
                "drift_probe_cell arm={} probe={}",
                self.arm.label(),
                probe.label()
            ),
        }
    }

    /// 别的行里这一格的键：`arm=… [probe=…] index=… <格>`。
    pub(super) fn key_text(&self, card_index: u32) -> String {
        let probe_text = self
            .probe
            .map_or_else(String::new, |probe| format!(" probe={}", probe.label()));
        format!(
            "arm={}{probe_text} index={card_index} {}",
            self.arm.label(),
            self.cell.label()
        )
    }
}

/// 一张卡在一条臂上累计的作废读数（同第 3 次重跑 `CellsOfTheCard` 的几栏）。
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct CardTallies {
    pub(super) result_mismatches: usize,
    pub(super) floor_violations: usize,
    pub(super) input_byte_mismatches: usize,
    pub(super) process_clock_cells: BTreeSet<SubmissionCell>,
    pub(super) coverage_failures: BTreeSet<(SubmissionForm, WaitMethod)>,
}

/// 一格一轮量完交回的东西（收下的那一次；三次都作废时 `medians` 是 `None`）。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CellRoundOutcome {
    pub(super) medians: Option<CellRoundMedians>,
    pub(super) warm_up_submissions: usize,
    pub(super) cards_before: BTreeMap<u32, CardClockReading>,
    pub(super) host_before: HostCoreReading,
    pub(super) host_after: HostCoreReading,
    pub(super) wall_median_nanoseconds: Option<f64>,
}

/// 把第 3 次重跑那一行的头 `name=submission_cell ` 换成这一格的头。
pub(super) fn with_the_line_head(line: &str, head: &str) -> String {
    line.replacen("name=submission_cell ", &format!("name={head} "), 1)
}

/// 开着的那一格热身、再计时提交（间隔 0 的格 100 次、50 ms 的格 50 次）：计时区间 ① 之前插探针的自旋与 `extra`（平常什么都不做）。
/// 交回（热身了几次、计时提交的读数）。
pub(super) fn warm_up_and_time_the_cell(
    environment: &mut dyn CellRoundEnvironment,
    request: &CellRoundRequest,
    extra: &mut dyn FnMut(),
) -> (usize, Vec<Option<SubmissionTiming>>) {
    let gap = Duration::from_millis(request.cell.gap_milliseconds);
    environment.open_the_cell(request.cell);
    let warm_up_submissions = warm_up_the_cell(request.arm.cell_warm_up(), &mut |count| {
        let timings = environment.time_the_submissions(count, gap, &mut || {});
        cell_round_reading(&timings, request.cell.units)
            .map(|reading| reading.thread_cpu_median_nanoseconds)
    });
    let timed_submissions = if request.cell.gap_milliseconds == 0 {
        TIMED_SUBMISSIONS_WITHOUT_A_GAP
    } else {
        TIMED_SUBMISSIONS_WITH_A_GAP
    };
    let spin = request
        .probe
        .map(|probe| probe_spin_nanoseconds(probe, request.round));
    let mut insertion = || {
        if let Some(nanoseconds) = spin {
            let _spun = spin_until_the_thread_cpu_advances(nanoseconds);
        }
        extra();
    };
    let timings = environment.time_the_submissions(timed_submissions, gap, &mut insertion);
    environment.close_the_cell();
    (warm_up_submissions, timings)
}

/// 一格一轮（登记第五节「控漂移段怎么量」：每格照第 3 次重跑的格，受控臂只把 10 次热身换成暖到稳）：每次先读卡（等别的计算进程、读各张卡，
/// 计时窗口外）→ 前校准 → 热身 → 计时提交 → 后校准；校准比有一个 > 1.10 或有一次钟读不到，那一次作废、等常态（R2-2）再重量，
/// 至多 3 次，三次都作废打 `name=round_invalid`。每次打一行这一格的头的 `submission_cell` 行（[`CellRoundRequest::line_head`]），受控臂另打一行
/// `name=cell_warm_up`。有一次提交的线程钟大过挂钟（停机 J-S5）交回 `Err`。
pub(super) fn measure_one_cell_round(
    environment: &mut dyn CellRoundEnvironment,
    card_reader: &mut dyn CardReader,
    request: CellRoundRequest,
    tallies: &mut CardTallies,
    waiting: &mut WaitingLedger,
    larger_ratio_per_cell: &mut Vec<Option<f64>>,
    emitter: &mut Emitter,
) -> Result<CellRoundOutcome, String> {
    let card_index = environment.card_index();
    let cell = request.cell;
    let round = request.round;
    let mut outcome = CellRoundOutcome {
        medians: None,
        warm_up_submissions: 0,
        cards_before: BTreeMap::new(),
        host_before: HostCoreReading::default(),
        host_after: HostCoreReading::default(),
        wall_median_nanoseconds: None,
    };
    for attempt in 1..=ATTEMPTS_PER_ROUND_AT_MOST {
        let cards_before = card_reader.wait_for_the_card_and_read_the_cards(card_index, emitter);
        let (host_before, ratio_before, threads_before) = (
            environment.host_core_reading(),
            environment.calibration_ratio(),
            environment.threads(),
        );
        let (warm_up_submissions, timings) =
            warm_up_and_time_the_cell(environment, &request, &mut || {});
        let threads_after = environment.threads();
        let ratio_after = environment.calibration_ratio();
        let host_after = environment.host_core_reading();
        let reading = cell_round_reading(&timings, cell.units);
        if let Some(outside) = reading
            .as_ref()
            .filter(|cell_reading| !cell_reading.is_every_thread_cpu_inside_the_wall)
        {
            return Err(format!(
                "clause_j_s5_thread_cpu_above_the_wall_at_{}_round_{round}_median_{:.0}",
                request.key_text(card_index).replace([' ', '='], "_"),
                outside.thread_cpu_median_nanoseconds
            ));
        }
        let ratios = (ratio_before, ratio_after);
        let verdict = cell_round_verdict(
            cell,
            &timings,
            reading.as_ref(),
            ratios,
            environment.link_peak_bytes_per_second(),
        );
        tallies.floor_violations += verdict.floor_violations;
        tallies.result_mismatches += usize::from(reading.as_ref().is_some_and(|cell_reading| {
            !cell_reading.are_the_results_holding || !cell_reading.are_the_sequence_numbers_holding
        }));
        tallies.input_byte_mismatches +=
            usize::from(reading.is_some() && !verdict.input_bytes_hold);
        if reading.as_ref().is_some_and(|cell_reading| {
            !cell_reading.is_every_wait_inside_the_submission
                || !cell_reading.are_the_comparison_counts_holding
        }) {
            tallies.coverage_failures.insert((cell.form, cell.wait));
        }
        larger_ratio_per_cell.push(match ratios {
            (Some(before), Some(after)) => Some(before.max(after)),
            (None, _) | (_, None) => None,
        });
        emitter.line(&with_the_line_head(
            &submission_cell_line(
                card_index,
                cell,
                round,
                attempt,
                reading.as_ref(),
                &verdict,
                ratios,
                (threads_before, threads_after),
                request.threshold_edges,
            ),
            &request.line_head(),
        ));
        if request.arm == DriftArm::Controlled {
            emitter.line(&format!(
                "name=cell_warm_up {} round={round} attempt={attempt} warm_up_submissions={warm_up_submissions}",
                request.key_text(card_index)
            ));
        }
        outcome = CellRoundOutcome {
            medians: None,
            warm_up_submissions,
            cards_before,
            host_before,
            host_after,
            wall_median_nanoseconds: reading
                .as_ref()
                .map(|cell_reading| cell_reading.wall_median_nanoseconds),
        };
        match (verdict.invalid_reason, reading) {
            (None, Some(round_reading)) => {
                if is_the_thread_clock_underestimating(
                    round_reading.process_cpu_median_nanoseconds,
                    round_reading.thread_cpu_median_nanoseconds,
                ) {
                    tallies.process_clock_cells.insert(cell);
                }
                outcome.medians = Some(CellRoundMedians {
                    thread_cpu_nanoseconds: round_reading.thread_cpu_median_nanoseconds,
                    floor_thread_cpu_nanoseconds: round_reading.floor_thread_cpu_median_nanoseconds,
                    process_cpu_nanoseconds: round_reading.process_cpu_median_nanoseconds,
                });
                return Ok(outcome);
            }
            (Some(reason), _) if attempt < ATTEMPTS_PER_ROUND_AT_MOST => {
                let mut calibrate = || environment.calibration_ratio();
                let mut sleep_before_the_retry = |duration: Duration| std::thread::sleep(duration);
                if let Some(wait) = wait_before_the_retry(
                    reason,
                    waiting,
                    &mut calibrate,
                    &mut sleep_before_the_retry,
                ) {
                    emitter.line(&format!(
                        "name=normal_wait {} round={round} attempt={attempt} reason={reason} checks={} waited_seconds={} ratio_per_check={} outcome={} segment_waited_seconds={}",
                        request.key_text(card_index),
                        wait.checks,
                        wait.waited_seconds,
                        wait.ratio_per_check.iter().map(|ratio| optional_number(*ratio)).collect::<Vec<_>>().join(","),
                        wait.outcome.label(),
                        waiting.total_seconds,
                    ));
                    if let Some(exit_code) = wait.outcome.exit_code_stopping_the_segment() {
                        emitter.line("name=stopped segment=host_cpu_drift_control reason=machine_not_back_to_normal_within_the_waiting_cap");
                        emitter.finish();
                        std::process::exit(exit_code);
                    }
                }
            }
            (Some(_) | None, _) => {
                if attempt == ATTEMPTS_PER_ROUND_AT_MOST {
                    emitter.line(&format!(
                        "name=round_invalid segment=host_cpu_drift_control {} round={round} attempts={ATTEMPTS_PER_ROUND_AT_MOST}",
                        request.key_text(card_index)
                    ));
                }
            }
        }
    }
    Ok(outcome)
}

#[cfg(test)]
mod tests;
