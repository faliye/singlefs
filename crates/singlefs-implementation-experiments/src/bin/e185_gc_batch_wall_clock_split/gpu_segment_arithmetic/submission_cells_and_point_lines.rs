use super::*;

/// 一种提交形态（登记第五节阳性对照 P1「全部 7 种提交形态」：常驻 × 三种等法 × 两种输入，加今天的写法）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SubmissionForm {
    Resident(InputForm),
    TodaySingleKernel,
}

impl SubmissionForm {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            SubmissionForm::Resident(form) => form.label(),
            SubmissionForm::TodaySingleKernel => "today_single_kernel",
        }
    }

    /// 这一种形态量哪几种等法（今天的写法只有它自己的阻塞等）。
    pub(crate) fn wait_methods(self) -> Vec<WaitMethod> {
        match self {
            SubmissionForm::Resident(_) => WaitMethod::ALL.to_vec(),
            SubmissionForm::TodaySingleKernel => vec![WaitMethod::Blocking],
        }
    }

    /// 这一种形态一次提交写进卡的单元字节（今天的写法拷一份）。
    pub(crate) const fn input_bytes(self, units: u64) -> u64 {
        match self {
            SubmissionForm::Resident(form) => expected_input_bytes(form, units),
            SubmissionForm::TodaySingleKernel => units * LOCAL_GPU_SLOT_BYTES,
        }
    }
}

/// 两次提交之间计时窗口外睡多久（毫秒；登记第一节「间隔」）：0 与 50，50 只量共用输入的三种等法。
pub(super) const GAP_BETWEEN_TWO_BATCHES_MILLISECONDS: u64 = 50;
pub(crate) const GAP_MILLISECONDS: [u64; 2] = [0, GAP_BETWEEN_TWO_BATCHES_MILLISECONDS];

/// 第一段一张卡上的一格（登记第五节「每轮」：格的次序间隔、输入、等法、n 由外到里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SubmissionCell {
    pub(crate) gap_milliseconds: u64,
    pub(crate) form: SubmissionForm,
    pub(crate) wait: WaitMethod,
    pub(crate) units: u64,
}

impl SubmissionCell {
    /// 同一格（间隔、形态、等法）换一档 n。
    pub(crate) const fn with_units(self, units: u64) -> SubmissionCell {
        SubmissionCell {
            gap_milliseconds: self.gap_milliseconds,
            form: self.form,
            wait: self.wait,
            units,
        }
    }

    pub(crate) fn label(self) -> String {
        format!(
            "form={} wait={} gap_milliseconds={} units={}",
            self.form.label(),
            self.wait.label(),
            self.gap_milliseconds,
            self.units
        )
    }
}

/// 一张卡上的 80 格（常驻 48 格 + 间隔 50 ms 24 格 + 今天的写法 8 格），按间隔、输入、等法、n 由外到里。
pub(crate) fn submission_cells_of_a_card() -> Vec<SubmissionCell> {
    let mut cells = Vec::new();
    for gap_milliseconds in GAP_MILLISECONDS {
        let forms: Vec<SubmissionForm> = if gap_milliseconds == 0 {
            vec![
                SubmissionForm::Resident(InputForm::Shared),
                SubmissionForm::Resident(InputForm::Split),
                SubmissionForm::TodaySingleKernel,
            ]
        } else {
            vec![SubmissionForm::Resident(InputForm::Shared)]
        };
        for form in forms {
            for wait in form.wait_methods() {
                for units in SUBMISSION_UNIT_COUNTS {
                    cells.push(SubmissionCell {
                        gap_milliseconds,
                        form,
                        wait,
                        units,
                    });
                }
            }
        }
    }
    cells
}

/// 每格热身几次（不计）、计时几次（间隔 0 的格 100 次、间隔 50 ms 的格 50 次）：第 3 次重跑第一段与第 4 次重跑控漂移段共用。
pub(crate) const WARM_UP_SUBMISSIONS: usize = 10;
pub(crate) const TIMED_SUBMISSIONS_WITHOUT_A_GAP: usize = 100;
pub(crate) const TIMED_SUBMISSIONS_WITH_A_GAP: usize = 50;

/// 第 `round` 轮（从 1 起）的格的次序：从第 `round` 格起轮换（登记第五节「每轮」）。
pub(crate) fn submission_cells_in_the_order_of_the_round(round: usize) -> Vec<SubmissionCell> {
    let mut cells = submission_cells_of_a_card();
    let start = (round - 1) % cells.len();
    cells.rotate_left(start);
    cells
}

/// 一格一轮算点要的三个中位（纳秒）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CellRoundMedians {
    pub(crate) thread_cpu_nanoseconds: f64,
    pub(crate) floor_thread_cpu_nanoseconds: Option<f64>,
    pub(crate) process_cpu_nanoseconds: f64,
}

/// 第一段全部格的各轮中位：键（卡序号、格），值 5 轮（作废的那一轮是 `None`）。
pub(crate) type CellMediansOfTheSegment =
    BTreeMap<(u32, SubmissionCell), Vec<Option<CellRoundMedians>>>;

/// 门槛是哪一台的 CPU（登记第一节「机器换算」，第一节主 agent 定第 2 条：第 1 行读 `machine_scaled`，`attack_ledger` 并列报）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ThresholdKind {
    AttackLedger,
    MachineScaled,
}

impl ThresholdKind {
    pub(crate) const ALL: [ThresholdKind; 2] =
        [ThresholdKind::AttackLedger, ThresholdKind::MachineScaled];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            ThresholdKind::AttackLedger => "attack_ledger",
            ThresholdKind::MachineScaled => "machine_scaled",
        }
    }

    /// 这一种门槛的值：原样，或 × ρ_机器（ρ 读不到交 `None`，那几行写 not_run）。
    pub(crate) fn scaled(
        self,
        attack_ledger_value: f64,
        machine_ratio: Option<f64>,
    ) -> Option<f64> {
        match self {
            ThresholdKind::AttackLedger => Some(attack_ledger_value),
            ThresholdKind::MachineScaled => machine_ratio.map(|ratio| attack_ledger_value * ratio),
        }
    }
}

/// 一个点的门槛（`out/g1.out` 那一点的 T 与省下的上界；A、B、D 是装置从 r2 第二段产物现算的）。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PointThresholds {
    pub(crate) by_candidate: BTreeMap<GpuCandidate, f64>,
    pub(crate) saved_upper_milliseconds: f64,
}

/// 用哪个 CPU 钟算 c_rt（第十条 J5：线程钟是读法，进程钟只在那一格触发 J5 时并列报）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CpuClockKind {
    Thread,
    Process,
}

impl CpuClockKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            CpuClockKind::Thread => "thread",
            CpuClockKind::Process => "process",
        }
    }
}

/// R1、R2、R3 一行的键（旋钮翻面按它配对）：点、候选、卡、形态、等法、间隔、门槛的机器、钟。
pub(crate) type PointJudgementKey = (
    SubmissionPoint,
    GpuCandidate,
    u32,
    SubmissionForm,
    WaitMethod,
    u64,
    ThresholdKind,
    CpuClockKind,
);

// ───────────────────────────── 两种 GPU 子命令与附带段共用的旋钮翻面（第 3 次重跑登记第八节「几何敏感性」、第十条 J2–J4） ─────────────────────────────

/// 一格写下的结局算不算「判定」（第 3 次重跑登记第十二节 R3-7「两端都有判定的对数」）：第十条 J9 的轮数不足「不下判定、不当阴性」；
/// 第十一节作废的（阳性对照红、GPU 结果错与计时没等到记成的 `void_` 那几种）与读不到的（门槛读不到写的 `not_run`）不当结果；
/// 那一端没有这个量的（`not_applicable`：进程钟与今天的写法没有 c_floor）没有可判的。第六节「不全相同写 `unstable`」与第五节
/// 分段表「都有判定（含「不稳定」）」：`unstable` 算判定。
pub(super) fn is_a_judgement(outcome: &str) -> bool {
    match outcome {
        "rounds_insufficient" | "not_run" | "not_applicable" => false,
        voided if voided.starts_with("void_") => false,
        _ => true,
    }
}

/// 一个键在这个旋钮另一端的键，两端的结局都是判定（[`is_a_judgement`]）时交回（另一端的键、这一端的判定、另一端的判定）；
/// 一对只出一次：换过去的键比原键大才收。
pub(super) fn both_ends_with_a_judgement<'judgements, Key: Ord>(
    judgements: &'judgements BTreeMap<Key, String>,
    key: &Key,
    other_end: &impl Fn(&Key) -> Option<Key>,
) -> Option<(Key, &'judgements str, &'judgements str)> {
    let other = other_end(key).filter(|other| other > key)?;
    let judgement = judgements.get(key)?;
    let other_judgement = judgements.get(&other)?;
    (is_a_judgement(judgement) && is_a_judgement(other_judgement)).then_some((
        other,
        judgement.as_str(),
        other_judgement.as_str(),
    ))
}

/// 一个量在一个旋钮两端的翻面：`judgements` 里每个键有一个结局，`other_end` 把一个键换成那个旋钮另一端的键（这个键本来就在另一端、
/// 或那一端没有对应的格交回 `None`）；交回两端都有判定而判定不同的每一对（[`both_ends_with_a_judgement`]）。
pub(super) fn knob_flips<Key: Ord + Clone>(
    judgements: &BTreeMap<Key, String>,
    other_end: impl Fn(&Key) -> Option<Key>,
) -> Vec<(Key, Key)> {
    judgements
        .keys()
        .filter_map(|key| {
            let (other, judgement, other_judgement) =
                both_ends_with_a_judgement(judgements, key, &other_end)?;
            (other_judgement != judgement).then(|| (key.clone(), other))
        })
        .collect()
}

/// 同一组键里，在这个旋钮两端都有判定的对数（「这几个点上不翻」那一句的分母）。
pub(super) fn knob_pairs<Key: Ord + Clone>(
    judgements: &BTreeMap<Key, String>,
    other_end: impl Fn(&Key) -> Option<Key>,
) -> usize {
    judgements
        .keys()
        .filter(|key| both_ends_with_a_judgement(judgements, key, &other_end).is_some())
        .count()
}

// ───────────────────────────── 第一段的点、拟合与攒批（第 3 次重跑登记第六节 R1–R5；纯函数，单测喂合成的各格中位） ─────────────────────────────

/// 一个点在一张卡、一种形态、一种等法、一种间隔下各轮的毫秒：序列里每档 n 取那一轮那一格的中位（`value_of` 取哪个钟），取平均
/// （[`point_milliseconds_of_the_sequence`]）。有一档那一轮没有读数，那一轮是 `None`。
pub(super) fn point_milliseconds_per_round(
    medians: &CellMediansOfTheSegment,
    card: u32,
    cell_without_the_units: SubmissionCell,
    submission_units: &[u64],
    value_of: fn(&CellRoundMedians) -> Option<f64>,
) -> Vec<Option<f64>> {
    (0..ROUNDS)
        .map(|round_index| {
            let by_units: BTreeMap<u64, f64> = submission_units
                .iter()
                .map(|units| {
                    let cell = cell_without_the_units.with_units(*units);
                    let value = medians
                        .get(&(card, cell))?
                        .get(round_index)?
                        .as_ref()
                        .and_then(value_of)?;
                    Some((*units, value / 1e6))
                })
                .collect::<Option<BTreeMap<u64, f64>>>()?;
            point_milliseconds_of_the_sequence(submission_units, &by_units)
        })
        .collect()
}

pub(super) fn thread_milliseconds_source(medians: &CellRoundMedians) -> Option<f64> {
    Some(medians.thread_cpu_nanoseconds)
}

pub(super) fn floor_milliseconds_source(medians: &CellRoundMedians) -> Option<f64> {
    medians.floor_thread_cpu_nanoseconds
}

pub(super) fn process_milliseconds_source(medians: &CellRoundMedians) -> Option<f64> {
    Some(medians.process_cpu_nanoseconds)
}

/// 一格（不带 n）在第一段里有没有：间隔 50 只有共用输入的常驻写法，今天的写法只有阻塞等。
pub(super) fn is_a_measured_cell(
    gap_milliseconds: u64,
    form: SubmissionForm,
    wait: WaitMethod,
) -> bool {
    form.wait_methods().contains(&wait)
        && (gap_milliseconds == 0 || form == SubmissionForm::Resident(InputForm::Shared))
}

/// 各轮的数与判定写成逗号接起来的两串；读不到的轮写 `not_run`。
pub(super) fn per_round_texts(
    values: &[Option<f64>],
    judgements: &[Option<&'static str>],
) -> (String, String) {
    (
        values
            .iter()
            .map(|value| optional_number(*value))
            .collect::<Vec<_>>()
            .join(","),
        judgements
            .iter()
            .map(|judgement| judgement.unwrap_or("not_run"))
            .collect::<Vec<_>>()
            .join(","),
    )
}

/// R1、R2、R3 各轮判定与 5 轮之后的判定：形态或卡作废了（`voiding` 给了原因）写那个原因；门槛读不到写 not_run；否则照 [`judgement_of_the_valid_rounds`]。
pub(super) fn judgements_of_the_point(
    milliseconds_per_round: &[Option<f64>],
    threshold: Option<f64>,
    candidate: GpuCandidate,
    voiding: Option<&'static str>,
) -> (Vec<Option<&'static str>>, &'static str) {
    let per_round: Vec<Option<&'static str>> = milliseconds_per_round
        .iter()
        .map(|milliseconds| {
            Some(judgement_against_the_threshold(
                (*milliseconds)?,
                threshold?,
                candidate,
            ))
        })
        .collect();
    let overall = match (voiding, threshold) {
        (Some(reason), _) => reason,
        (None, None) => "not_run",
        (None, Some(_)) => judgement_of_the_valid_rounds(&per_round),
    };
    (per_round, overall)
}

/// 第一段算点的输入：门槛、ρ、卡、各格中位、作废的（卡、形态）与原因、触发 J5 的格。
pub(crate) struct HostCpuPointInputs<'inputs> {
    pub(crate) thresholds: &'inputs BTreeMap<SubmissionPoint, PointThresholds>,
    pub(crate) machine_ratio: Option<f64>,
    pub(crate) cards: &'inputs [u32],
    pub(crate) medians: &'inputs CellMediansOfTheSegment,
    pub(crate) voiding: &'inputs BTreeMap<(u32, SubmissionForm, WaitMethod), &'static str>,
    pub(crate) process_clock_cells: &'inputs BTreeSet<(u32, SubmissionCell)>,
}

/// R1、R2、R3 的产物行与判定表（旋钮翻面用）：每个点 × 候选 × 卡 × 形态 × 等法 × 间隔 × 门槛的机器各一行线程钟的；序列里有一格触发 J5 的，
/// 另报一行进程钟的（`clock=process`，没有 c_floor）。今天的写法只按 C 的门槛判一行（只报，不当第 1 行的读数）。
pub(super) fn host_cpu_point_lines(
    inputs: &HostCpuPointInputs<'_>,
) -> (
    Vec<String>,
    BTreeMap<PointJudgementKey, String>,
    BTreeMap<PointJudgementKey, String>,
) {
    let mut lines = Vec::new();
    let mut point_judgements = BTreeMap::new();
    let mut floor_judgements = BTreeMap::new();
    let forms = [
        SubmissionForm::Resident(InputForm::Shared),
        SubmissionForm::Resident(InputForm::Split),
        SubmissionForm::TodaySingleKernel,
    ];
    for (point, point_thresholds) in inputs.thresholds {
        for (candidate, attack_threshold) in &point_thresholds.by_candidate {
            let submission_units =
                submission_units_of_the_intent(*candidate, point.fraction, point.batch_units);
            let units_text = submission_units
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(",");
            for card in inputs.cards {
                for form in forms {
                    if form == SubmissionForm::TodaySingleKernel
                        && *candidate != GpuCandidate::EveryStage
                    {
                        continue;
                    }
                    for wait in WaitMethod::ALL {
                        for gap_milliseconds in GAP_MILLISECONDS {
                            if !is_a_measured_cell(gap_milliseconds, form, wait) {
                                continue;
                            }
                            let cell = SubmissionCell {
                                gap_milliseconds,
                                form,
                                wait,
                                units: 0,
                            };
                            let has_a_process_clock_cell = submission_units.iter().any(|units| {
                                inputs
                                    .process_clock_cells
                                    .contains(&(*card, cell.with_units(*units)))
                            });
                            let clocks: &[CpuClockKind] = if has_a_process_clock_cell {
                                &[CpuClockKind::Thread, CpuClockKind::Process]
                            } else {
                                &[CpuClockKind::Thread]
                            };
                            for clock in clocks {
                                let (source, floor) = match clock {
                                    CpuClockKind::Thread => (
                                        thread_milliseconds_source
                                            as fn(&CellRoundMedians) -> Option<f64>,
                                        point_milliseconds_per_round(
                                            inputs.medians,
                                            *card,
                                            cell,
                                            &submission_units,
                                            floor_milliseconds_source,
                                        ),
                                    ),
                                    CpuClockKind::Process => (
                                        process_milliseconds_source
                                            as fn(&CellRoundMedians) -> Option<f64>,
                                        vec![None; ROUNDS],
                                    ),
                                };
                                let milliseconds = point_milliseconds_per_round(
                                    inputs.medians,
                                    *card,
                                    cell,
                                    &submission_units,
                                    source,
                                );
                                for kind in ThresholdKind::ALL {
                                    let threshold =
                                        kind.scaled(*attack_threshold, inputs.machine_ratio);
                                    let voiding = inputs.voiding.get(&(*card, form, wait)).copied();
                                    let (per_round, judgement) = judgements_of_the_point(
                                        &milliseconds,
                                        threshold,
                                        *candidate,
                                        voiding,
                                    );
                                    let (floor_per_round, floor_judgement) = match (clock, form) {
                                        (CpuClockKind::Thread, SubmissionForm::Resident(_)) => {
                                            judgements_of_the_point(
                                                &floor, threshold, *candidate, voiding,
                                            )
                                        }
                                        (CpuClockKind::Process, _)
                                        | (_, SubmissionForm::TodaySingleKernel) => {
                                            (vec![None; ROUNDS], "not_applicable")
                                        }
                                    };
                                    let (values_text, judgements_text) =
                                        per_round_texts(&milliseconds, &per_round);
                                    let (floor_values_text, floor_judgements_text) =
                                        per_round_texts(&floor, &floor_per_round);
                                    lines.push(format!(
                                    "name=host_cpu_per_submission_point {} candidate={} index={card} form={} wait={} gap_milliseconds={gap_milliseconds} clock={} threshold_kind={} threshold_milliseconds={} submission_units={units_text} host_cpu_per_submission_milliseconds_per_round={values_text} judgement_per_round={judgements_text} judgement={judgement} floor_host_cpu_per_submission_milliseconds_per_round={floor_values_text} floor_judgement_per_round={floor_judgements_text} floor_judgement={floor_judgement}",
                                    point.label(),
                                    candidate.label(),
                                    form.label(),
                                    wait.label(),
                                    clock.label(),
                                    kind.label(),
                                    optional_number(threshold),
                                ));
                                    let key = (
                                        *point,
                                        *candidate,
                                        *card,
                                        form,
                                        wait,
                                        gap_milliseconds,
                                        kind,
                                        *clock,
                                    );
                                    point_judgements.insert(key, judgement.to_string());
                                    floor_judgements.insert(key, floor_judgement.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    (lines, point_judgements, floor_judgements)
}

/// 一格（不带 n）的拟合：总的（8 档各取 5 轮中位的中位）与每轮各一次。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct FitsOfACell {
    pub(crate) overall: Option<FixedAndPerByteFit>,
    pub(crate) per_round: Vec<Option<FixedAndPerByteFit>>,
}

/// R4：一张卡、一种常驻形态、一种等法、一种间隔的拟合（线程钟，毫秒对字节）。
pub(super) fn fits_of_the_cell(
    medians: &CellMediansOfTheSegment,
    card: u32,
    cell_without_the_units: SubmissionCell,
) -> FitsOfACell {
    let milliseconds_of = |units: u64, round_index: usize| -> Option<f64> {
        medians
            .get(&(card, cell_without_the_units.with_units(units)))?
            .get(round_index)?
            .map(|round| round.thread_cpu_nanoseconds / 1e6)
    };
    let overall_points: Option<BTreeMap<u64, f64>> = SUBMISSION_UNIT_COUNTS
        .iter()
        .map(|units| {
            let rounds: Vec<f64> = (0..ROUNDS)
                .filter_map(|round_index| milliseconds_of(*units, round_index))
                .collect();
            Some((*units, median_of(&rounds)?))
        })
        .collect();
    let per_round = (0..ROUNDS)
        .map(|round_index| {
            let points: BTreeMap<u64, f64> = SUBMISSION_UNIT_COUNTS
                .iter()
                .map(|units| Some((*units, milliseconds_of(*units, round_index)?)))
                .collect::<Option<_>>()?;
            fixed_and_per_byte_fit(&points)
        })
        .collect();
    FitsOfACell {
        overall: overall_points.as_ref().and_then(fixed_and_per_byte_fit),
        per_round,
    }
}

/// R4 一行（`name=fixed_and_per_byte_fit`）。
pub(super) fn fit_line(
    card: u32,
    cell_without_the_units: SubmissionCell,
    fits: &FitsOfACell,
) -> String {
    let per_round = |value_of: fn(&FixedAndPerByteFit) -> f64| {
        fits.per_round
            .iter()
            .map(|fit| optional_number(fit.as_ref().map(value_of)))
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
    "name=fixed_and_per_byte_fit index={card} form={} wait={} gap_milliseconds={} fixed_milliseconds={} per_mebibyte_milliseconds={} maximum_relative_residual={} points={} fit_not_linear={} fixed_milliseconds_per_round={} per_mebibyte_milliseconds_per_round={}",
    cell_without_the_units.form.label(),
    cell_without_the_units.wait.label(),
    cell_without_the_units.gap_milliseconds,
    optional_number(fits.overall.map(|fit| fit.fixed_milliseconds)),
    optional_number(fits.overall.map(|fit| fit.per_mebibyte_milliseconds)),
    optional_number(fits.overall.map(|fit| fit.maximum_relative_residual)),
    SUBMISSION_UNIT_COUNTS.len(),
    fits.overall.is_none_or(|fit| !fit.is_linear()),
    per_round(|fit| fit.fixed_milliseconds),
    per_round(|fit| fit.per_mebibyte_milliseconds),
)
}

/// R5：每个点 × 卡 × 常驻形态 × 等法 × 间隔 × 门槛的机器一行（`name=merging_shape`），按 C 的提交序列与门槛、`saved_upper_ms`、
/// n = m 那一档的 c_rt(m) 与那一轮的拟合判；后两类要那一格的总拟合不是 `fit_not_linear`（第十条 J6）。交回行与判定表。
pub(super) fn merging_shape_lines(
    inputs: &HostCpuPointInputs<'_>,
) -> (Vec<String>, BTreeMap<PointJudgementKey, String>) {
    let mut lines = Vec::new();
    let mut categories = BTreeMap::new();
    for (point, point_thresholds) in inputs.thresholds {
        let Some(every_stage_threshold) =
            point_thresholds.by_candidate.get(&GpuCandidate::EveryStage)
        else {
            continue;
        };
        let submission_units = submission_units_of_the_intent(
            GpuCandidate::EveryStage,
            point.fraction,
            point.batch_units,
        );
        let units_per_intent = units_of_one_submission_per_intent(&submission_units);
        for card in inputs.cards {
            for form in InputForm::ALL.map(SubmissionForm::Resident) {
                for wait in WaitMethod::ALL {
                    for gap_milliseconds in GAP_MILLISECONDS {
                        if !is_a_measured_cell(gap_milliseconds, form, wait) {
                            continue;
                        }
                        let cell = SubmissionCell {
                            gap_milliseconds,
                            form,
                            wait,
                            units: units_per_intent,
                        };
                        let fits = fits_of_the_cell(inputs.medians, *card, cell);
                        let is_the_fit_linear = fits.overall.is_some_and(|fit| fit.is_linear());
                        let point_milliseconds = point_milliseconds_per_round(
                            inputs.medians,
                            *card,
                            cell,
                            &submission_units,
                            thread_milliseconds_source,
                        );
                        let per_intent_milliseconds = point_milliseconds_per_round(
                            inputs.medians,
                            *card,
                            cell,
                            &[units_per_intent],
                            thread_milliseconds_source,
                        );
                        for kind in ThresholdKind::ALL {
                            let threshold =
                                kind.scaled(*every_stage_threshold, inputs.machine_ratio);
                            let saved = kind.scaled(
                                point_thresholds.saved_upper_milliseconds,
                                inputs.machine_ratio,
                            );
                            let shapes: Vec<Option<MergingShape>> = (0..ROUNDS)
                                .map(|round_index| {
                                    merging_shape(
                                        point_milliseconds[round_index]?,
                                        threshold?,
                                        per_intent_milliseconds[round_index]?,
                                        saved?,
                                        units_per_intent,
                                        fits.per_round[round_index],
                                        is_the_fit_linear,
                                    )
                                })
                                .collect();
                            let labels: Vec<Option<&'static str>> = shapes
                                .iter()
                                .map(|shape| shape.map(MergingShape::label))
                                .collect();
                            let category =
                                match (inputs.voiding.get(&(*card, form, wait)), threshold) {
                                    (Some(reason), _) => reason,
                                    (None, None) => "not_run",
                                    (None, Some(_)) => judgement_of_the_valid_rounds(&labels),
                                };
                            let intents_text = shapes
                                .iter()
                                .map(|shape| match shape {
                                    Some(MergingShape::MergeAcrossIntents {
                                        intents_to_merge_at_least,
                                    }) => {
                                        format!("{intents_to_merge_at_least:.0}")
                                    }
                                    Some(
                                        MergingShape::OneSubmissionPerBatch
                                        | MergingShape::OneSubmissionPerIntent
                                        | MergingShape::LosesHoweverMerged
                                        | MergingShape::FitNotLinear,
                                    )
                                    | None => "none".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(",");
                            let per_byte_text = fits
                                .per_round
                                .iter()
                                .map(|fit| {
                                    optional_number(fit.map(|round_fit| {
                                        round_fit.per_byte_milliseconds
                                            * (units_per_intent * LOCAL_GPU_SLOT_BYTES) as f64
                                    }))
                                })
                                .collect::<Vec<_>>()
                                .join(",");
                            let (per_intent_text, category_text) =
                                per_round_texts(&per_intent_milliseconds, &labels);
                            lines.push(format!(
                            "name=merging_shape {} index={card} form={} wait={} gap_milliseconds={gap_milliseconds} threshold_kind={} threshold_milliseconds={} saved_per_intent_milliseconds={} one_submission_per_intent_units={units_per_intent} one_submission_per_intent_host_cpu_milliseconds_per_round={per_intent_text} per_byte_part_milliseconds_per_round={per_byte_text} intents_to_merge_at_least_per_round={intents_text} fit_not_linear={} category_per_round={category_text} category={category}",
                            point.label(),
                            form.label(),
                            wait.label(),
                            kind.label(),
                            optional_number(threshold),
                            optional_number(saved),
                            !is_the_fit_linear,
                        ));
                            categories.insert(
                                (
                                    *point,
                                    GpuCandidate::EveryStage,
                                    *card,
                                    form,
                                    wait,
                                    gap_milliseconds,
                                    kind,
                                    CpuClockKind::Thread,
                                ),
                                category.to_string(),
                            );
                        }
                    }
                }
            }
        }
    }
    (lines, categories)
}

/// 一个旋钮（登记第八节）：哪一条失败条款管它的翻面、怎么把一个键换到另一端。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JudgementKnob {
    /// J3：两张卡。
    Card,
    /// J4：共用 ／ 分开。
    InputForm,
    /// J2：阻塞等 ／ 睡 100 µs。
    BlockingAndShortSleep,
    /// J2：阻塞等 ／ 睡 1 ms。
    BlockingAndLongerSleep,
    /// J4：睡 100 µs ／ 睡 1 ms。
    ShortAndLongerSleep,
    /// J4：间隔 0 ／ 50 ms。
    Gap,
    /// J4、J10：门槛原样 ／ × ρ_机器。
    ThresholdMachine,
    /// J5：线程钟 ／ 进程钟（只在那一格触发 J5、报了进程钟那一行的点上有两端）。
    Clock,
}

impl JudgementKnob {
    pub(crate) const ALL: [JudgementKnob; 8] = [
        JudgementKnob::Card,
        JudgementKnob::InputForm,
        JudgementKnob::BlockingAndShortSleep,
        JudgementKnob::BlockingAndLongerSleep,
        JudgementKnob::ShortAndLongerSleep,
        JudgementKnob::Gap,
        JudgementKnob::ThresholdMachine,
        JudgementKnob::Clock,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            JudgementKnob::Card => "card",
            JudgementKnob::InputForm => "input_form",
            JudgementKnob::BlockingAndShortSleep => "wait_blocking_and_sleep_100_microseconds",
            JudgementKnob::BlockingAndLongerSleep => "wait_blocking_and_sleep_1_millisecond",
            JudgementKnob::ShortAndLongerSleep => {
                "wait_sleep_100_microseconds_and_sleep_1_millisecond"
            }
            JudgementKnob::Gap => "gap",
            JudgementKnob::ThresholdMachine => "threshold_machine",
            JudgementKnob::Clock => "clock",
        }
    }

    pub(crate) const fn clause(self) -> &'static str {
        match self {
            JudgementKnob::Card => "two_cards_judge_differently",
            JudgementKnob::BlockingAndShortSleep | JudgementKnob::BlockingAndLongerSleep => {
                "the_wait_method_decides"
            }
            JudgementKnob::InputForm | JudgementKnob::ShortAndLongerSleep | JudgementKnob::Gap => {
                "flips_with_a_knob"
            }
            JudgementKnob::ThresholdMachine => "machine_ratio_decides",
            JudgementKnob::Clock => "thread_clock_underestimates",
        }
    }

    /// 把键换到这个旋钮的另一端（`cards` 是第一段量的两张卡）。
    pub(crate) fn other_end(
        self,
        key: &PointJudgementKey,
        cards: &[u32],
    ) -> Option<PointJudgementKey> {
        let (point, candidate, card, form, wait, gap, kind, clock) = *key;
        let swap_wait = |one: WaitMethod, other: WaitMethod| {
            (wait == one).then_some((point, candidate, card, form, other, gap, kind, clock))
        };
        match self {
            JudgementKnob::Card => match cards {
                [one, other] if card == *one => {
                    Some((point, candidate, *other, form, wait, gap, kind, clock))
                }
                _ => None,
            },
            JudgementKnob::InputForm => (form == SubmissionForm::Resident(InputForm::Shared))
                .then_some((
                    point,
                    candidate,
                    card,
                    SubmissionForm::Resident(InputForm::Split),
                    wait,
                    gap,
                    kind,
                    clock,
                )),
            JudgementKnob::BlockingAndShortSleep => {
                swap_wait(WaitMethod::Blocking, WaitMethod::SleepingShortly)
            }
            JudgementKnob::BlockingAndLongerSleep => {
                swap_wait(WaitMethod::Blocking, WaitMethod::SleepingLonger)
            }
            JudgementKnob::ShortAndLongerSleep => {
                swap_wait(WaitMethod::SleepingShortly, WaitMethod::SleepingLonger)
            }
            JudgementKnob::Gap => (gap == 0).then_some((
                point,
                candidate,
                card,
                form,
                wait,
                GAP_BETWEEN_TWO_BATCHES_MILLISECONDS,
                kind,
                clock,
            )),
            JudgementKnob::ThresholdMachine => (kind == ThresholdKind::AttackLedger).then_some((
                point,
                candidate,
                card,
                form,
                wait,
                gap,
                ThresholdKind::MachineScaled,
                clock,
            )),
            JudgementKnob::Clock => (clock == CpuClockKind::Thread).then_some((
                point,
                candidate,
                card,
                form,
                wait,
                gap,
                kind,
                CpuClockKind::Process,
            )),
        }
    }
}

/// 一个量（R1、R2、R5）在每个旋钮两端的翻面：翻了的每一对一行 `name=knob_flip`，每个旋钮一行 `name=knob_flip_summary`（两端都有判定的对数、翻了几对；
/// 一端是轮数不足、作废、读不到或没有这个量的对不算，[`is_a_judgement`]）。
pub(super) fn knob_flip_lines(
    quantity: &str,
    judgements: &BTreeMap<PointJudgementKey, String>,
    cards: &[u32],
) -> Vec<String> {
    let mut lines = Vec::new();
    for knob in JudgementKnob::ALL {
        let other_end = |key: &PointJudgementKey| knob.other_end(key, cards);
        let flips = knob_flips(judgements, other_end);
        for (one, other) in &flips {
            let (point, candidate, card, form, wait, gap, kind, clock) = *one;
            lines.push(format!(
            "name=knob_flip quantity={quantity} knob={} clause={} {} candidate={} index={card} form={} wait={} gap_milliseconds={gap} threshold_kind={} clock={} other_end_index={} other_end_form={} other_end_wait={} other_end_gap_milliseconds={} other_end_threshold_kind={} judgement_at_one_end={} judgement_at_the_other_end={}",
            knob.label(),
            knob.clause(),
            point.label(),
            candidate.label(),
            form.label(),
            wait.label(),
            kind.label(),
            clock.label(),
            other.2,
            other.3.label(),
            other.4.label(),
            other.5,
            other.6.label(),
            judgements[one],
            judgements[other],
        ));
        }
        lines.push(format!(
            "name=knob_flip_summary quantity={quantity} knob={} clause={} pairs={} flipped={}",
            knob.label(),
            knob.clause(),
            knob_pairs(judgements, other_end),
            flips.len()
        ));
    }
    lines
}

/// 第十条 J10 的旁证：本机同一线程算 A 那一点的提交序列的 CPU（每次提交取它那一档 n 的中位，取平均）÷ T_A，与 ρ_机器 差一倍以上就记。
pub(super) fn machine_ratio_cross_check_lines(
    thresholds: &BTreeMap<SubmissionPoint, PointThresholds>,
    cpu_crc_milliseconds: &BTreeMap<u64, f64>,
    machine_ratio: Option<f64>,
) -> Vec<String> {
    thresholds
    .iter()
    .map(|(point, point_thresholds)| {
        let submission_units =
            submission_units_of_the_intent(GpuCandidate::ScanChecksumOnly, point.fraction, point.batch_units);
        let local = point_milliseconds_of_the_sequence(&submission_units, cpu_crc_milliseconds);
        let threshold = point_thresholds.by_candidate.get(&GpuCandidate::ScanChecksumOnly).copied();
        let ratio = local.zip(threshold).map(|(local_value, threshold_value)| local_value / threshold_value);
        format!(
            "name=machine_ratio_cross_check {} local_crc_per_submission_milliseconds={} scan_checksum_threshold_milliseconds={} ratio={} machine_ratio={} off_by_more_than_double={}",
            point.label(),
            optional_number(local),
            optional_number(threshold),
            optional_number(ratio),
            optional_number(machine_ratio),
            ratio
                .zip(machine_ratio)
                .is_some_and(|(value, ratio_of_the_machines)| is_the_cross_check_off_by_more_than_double(value, ratio_of_the_machines)),
        )
    })
    .collect()
}

/// 一格一轮量完之后（计时窗口外）的读数与判定：作废原因（读不到、校准比超过 1.10）、物理下限、结果与锚点。
pub(crate) struct CellRoundVerdict {
    pub(crate) invalid_reason: Option<&'static str>,
    pub(crate) floor_violations: usize,
    pub(crate) respects_the_floor: bool,
    pub(crate) input_bytes_hold: bool,
}

pub(crate) fn cell_round_verdict(
    cell: SubmissionCell,
    timings: &[Option<SubmissionTiming>],
    reading: Option<&CellRoundReading>,
    ratios: (Option<f64>, Option<f64>),
    link_peak_bytes_per_second: f64,
) -> CellRoundVerdict {
    let floor_violations = timings
        .iter()
        .flatten()
        .filter(|timing| {
            let seconds = timing.wall_nanoseconds as f64 / NANOSECONDS_PER_SECOND as f64;
            let respects = match cell.form {
                SubmissionForm::Resident(input_form) => respects_the_floor_of_the_form(
                    seconds,
                    input_form,
                    cell.units,
                    link_peak_bytes_per_second,
                ),
                SubmissionForm::TodaySingleKernel => respects_the_physical_floor(
                    seconds,
                    cell.form.input_bytes(cell.units),
                    link_peak_bytes_per_second,
                ),
            };
            !respects
        })
        .count();
    let invalid_reason = match (reading, ratios) {
        (None, _) => Some("thread_cpu_clock_unreadable"),
        (Some(_), (None, _) | (_, None)) => Some("calibration_unreadable"),
        (Some(_), (Some(before), Some(after))) => {
            is_voided_by_the_calibration_ratios(before, after)
                .then_some(CALIBRATION_RATIO_INVALID_REASON)
        }
    };
    CellRoundVerdict {
        invalid_reason,
        floor_violations,
        respects_the_floor: floor_violations == 0,
        input_bytes_hold: reading.is_some_and(|cell_reading| {
            cell_reading
                .input_bytes_written
                .iter()
                .all(|bytes| *bytes == cell.form.input_bytes(cell.units))
        }),
    }
}

/// `name=submission_cell` 一行（第六节「只报」那一格与第八节轨迹）。
#[allow(
    clippy::too_many_arguments,
    reason = "一行产物的各栏：格、轮、读数、判定、校准与线程数各是一样"
)]
pub(crate) fn submission_cell_line(
    card_index: u32,
    cell: SubmissionCell,
    round: usize,
    attempt: usize,
    reading: Option<&CellRoundReading>,
    verdict: &CellRoundVerdict,
    ratios: (Option<f64>, Option<f64>),
    threads: (Option<usize>, Option<usize>),
    thresholds_edges: (f64, f64),
) -> String {
    let trajectory = reading.and_then(|cell_reading| {
        submission_trajectory(
            &cell_reading.thread_cpu_in_order,
            thresholds_edges.0,
            thresholds_edges.1,
        )
    });
    let number_of = |value: Option<f64>| optional_number(value);
    let count_of =
        |value: Option<usize>| value.map_or_else(|| "none".to_string(), |count| count.to_string());
    format!(
        "name=submission_cell index={card_index} {} round={round} attempt={attempt} thread_cpu_median_nanoseconds={} floor_thread_cpu_median_nanoseconds={} process_cpu_median_nanoseconds={} wait_thread_cpu_median_nanoseconds={} wait_wall_median_nanoseconds={} wall_median_nanoseconds={} polls_median={} input_bytes_written={} input_bytes_written_holds={} results_agree_with_the_cpu={} sequence_numbers_hold={} uncertain_entries={} thread_cpu_maximum_nanoseconds={} thread_cpu_percentile_90_nanoseconds={} submissions_at_or_above_the_smallest_threshold={} submissions_at_or_above_the_largest_threshold={} thread_cpu_first_nanoseconds={} thread_cpu_last_nanoseconds={} respects_the_physical_floor={} floor_violations={} wait_inside_the_submission={} comparison_counts_hold={} thread_clock_underestimated={} calibration_ratio_before={} calibration_ratio_after={} threads_before={} threads_after={} invalid_reason={}",
        cell.label(),
        number_of(reading.map(|cell_reading| cell_reading.thread_cpu_median_nanoseconds)),
        number_of(reading.and_then(|cell_reading| cell_reading.floor_thread_cpu_median_nanoseconds)),
        number_of(reading.map(|cell_reading| cell_reading.process_cpu_median_nanoseconds)),
        number_of(reading.and_then(|cell_reading| cell_reading.wait_thread_cpu_median_nanoseconds)),
        number_of(reading.and_then(|cell_reading| cell_reading.wait_wall_median_nanoseconds)),
        number_of(reading.map(|cell_reading| cell_reading.wall_median_nanoseconds)),
        number_of(reading.and_then(|cell_reading| cell_reading.polls_median)),
        cell.form.input_bytes(cell.units),
        verdict.input_bytes_hold,
        reading.is_some_and(|cell_reading| cell_reading.are_the_results_holding),
        reading.is_some_and(|cell_reading| cell_reading.are_the_sequence_numbers_holding),
        count_of(reading.map(|cell_reading| cell_reading.uncertain_total)),
        number_of(trajectory.map(|path| path.maximum_nanoseconds)),
        number_of(trajectory.map(|path| path.percentile_90_nanoseconds)),
        count_of(trajectory.map(|path| path.at_or_above_the_smallest_threshold)),
        count_of(trajectory.map(|path| path.at_or_above_the_largest_threshold)),
        number_of(trajectory.map(|path| path.first_nanoseconds)),
        number_of(trajectory.map(|path| path.last_nanoseconds)),
        verdict.respects_the_floor,
        verdict.floor_violations,
        reading.is_some_and(|cell_reading| cell_reading.is_every_wait_inside_the_submission),
        reading.is_some_and(|cell_reading| cell_reading.are_the_comparison_counts_holding),
        reading.is_some_and(|cell_reading| is_the_thread_clock_underestimating(
            cell_reading.process_cpu_median_nanoseconds,
            cell_reading.thread_cpu_median_nanoseconds
        )),
        number_of(ratios.0),
        number_of(ratios.1),
        count_of(threads.0),
        count_of(threads.1),
        verdict.invalid_reason.unwrap_or("none"),
    )
}

/// 第一段量完之后的全部判定行：R1–R3（`name=host_cpu_per_submission_point`）、R4（`name=fixed_and_per_byte_fit`）、R5（`name=merging_shape`）、
/// 三个量在各旋钮两端的翻面（J2–J4、J10）与 J10 的旁证。
pub(crate) fn emit_the_analysis(
    inputs: &HostCpuPointInputs<'_>,
    cpu_crc_milliseconds: &BTreeMap<u64, f64>,
    emitter: &mut Emitter,
) {
    for line in analysis_lines(inputs, cpu_crc_milliseconds) {
        emitter.line(&line);
    }
}

/// [`emit_the_analysis`] 打的那几行，按打的次序（第 4 次重跑控漂移段每条臂各算一份，点行改名、各行加 `arm=` 之后再打）。
pub(crate) fn analysis_lines(
    inputs: &HostCpuPointInputs<'_>,
    cpu_crc_milliseconds: &BTreeMap<u64, f64>,
) -> Vec<String> {
    let mut lines = Vec::new();
    let (point_lines, point_judgements, floor_judgements) = host_cpu_point_lines(inputs);
    lines.extend(point_lines);
    for card in inputs.cards {
        for form in InputForm::ALL.map(SubmissionForm::Resident) {
            for wait in WaitMethod::ALL {
                for gap_milliseconds in GAP_MILLISECONDS {
                    if !is_a_measured_cell(gap_milliseconds, form, wait) {
                        continue;
                    }
                    let cell = SubmissionCell {
                        gap_milliseconds,
                        form,
                        wait,
                        units: 0,
                    };
                    lines.push(fit_line(
                        *card,
                        cell,
                        &fits_of_the_cell(inputs.medians, *card, cell),
                    ));
                }
            }
        }
    }
    let (merging_lines, categories) = merging_shape_lines(inputs);
    lines.extend(merging_lines);
    for (quantity, judgements) in [
        ("host_cpu_per_submission", &point_judgements),
        ("floor_host_cpu_per_submission", &floor_judgements),
        ("merging_shape", &categories),
    ] {
        lines.extend(knob_flip_lines(quantity, judgements, inputs.cards));
    }
    lines.extend(machine_ratio_cross_check_lines(
        inputs.thresholds,
        cpu_crc_milliseconds,
        inputs.machine_ratio,
    ));
    lines
}
