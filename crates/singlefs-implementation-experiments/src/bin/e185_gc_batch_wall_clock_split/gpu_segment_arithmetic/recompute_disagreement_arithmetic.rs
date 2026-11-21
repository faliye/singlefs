use super::*;

// ───────────────────────────── recompute-disagreement 的算术（第 3 次重跑登记第五节「第二段怎么量」、第六节 R6–R8；纯函数） ─────────────────────────────

/// 第二段每张卡每种输入的批数与每批的单元数。
pub(crate) const DISAGREEMENT_BATCHES: u64 = 64;
pub(crate) const UNITS_PER_DISAGREEMENT_BATCH: u64 = 16_384;
/// 一个单元不一致之后至多单独重算几次。
pub(crate) const RECOMPUTES_AT_MOST: usize = 3;
/// 注入只在第 1–8 批；P5（确定性）注第 7 个单元（下标 7），P6（暂态）注第 11 个单元（下标 11）；P5 在分开输入时改核二那一份输入的第 100 字节。
pub(crate) const INJECTED_BATCHES: std::ops::RangeInclusive<u64> = 1..=8;
pub(crate) const DETERMINISTIC_INJECTION_UNIT: u64 = 7;
pub(crate) const TRANSIENT_INJECTION_UNIT: u64 = 11;
pub(super) const DETERMINISTIC_INJECTION_BYTE: usize = 100;
/// 两张卡同一输入那一批：卡序号一栏填 0xFFFF、批号 0（共用输入）。
pub(crate) const BOTH_CARDS_SEED_CARD: u64 = 0xFFFF;
pub(crate) const BOTH_CARDS_SEED_BATCH: u64 = 0;
/// 内容的种子起点与 xorshift64* 的乘数（登记第五节「第二段怎么量」）。
pub(super) const CONTENT_SEED_START: u64 = 0x9E37_79B9_7F4A_7C15;
pub(super) const CONTENT_MULTIPLIER: u64 = 0x2545_F491_4F6C_DD1D;
/// 第二段单元的两种长度：偶数号单元 32768、奇数号 16384。
pub(super) const EVEN_UNIT_BYTES: usize = 32_768;
pub(super) const ODD_UNIT_BYTES: usize = 16_384;

/// 一个单元的长度（偶数号整单元，奇数号半个）。
pub(crate) const fn disagreement_unit_length(unit: u64) -> usize {
    if unit.is_multiple_of(2) {
        EVEN_UNIT_BYTES
    } else {
        ODD_UNIT_BYTES
    }
}

/// 第二段一个单元的字节：起点 x = 种子起点 ^ ((卡 << 48) | (输入形态 << 40) | (批号 << 20) | 单元号)，xorshift64* 每步出 8 字节、小端，截到长度。
pub(crate) fn disagreement_unit_content(
    card_seed: u64,
    form: InputForm,
    batch: u64,
    unit: u64,
    into: &mut [u8],
) {
    let mut state =
        CONTENT_SEED_START ^ ((card_seed << 48) | (form.seed_bits() << 40) | (batch << 20) | unit);
    for chunk in into.chunks_mut(8) {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let output = state.wrapping_mul(CONTENT_MULTIPLIER).to_le_bytes();
        chunk.copy_from_slice(&output[..chunk.len()]);
    }
}

/// 第七节乙类「第二段的内容」锚点：(卡种子、形态、批、单元、前 8 字节、CRC-32C)。
pub(crate) const CONTENT_ANCHORS: [(u64, InputForm, u64, u64, u64, u32); 4] = [
    (
        0,
        InputForm::Shared,
        1,
        0,
        0x3ae4_b0f4_eaea_c4c4,
        0xaac1_beb6,
    ),
    (
        0,
        InputForm::Shared,
        1,
        1,
        0x57c1_1d0a_c205_6cc7,
        0x0dff_e7b3,
    ),
    (
        1,
        InputForm::Split,
        64,
        16_383,
        0x972b_7b78_30e1_c500,
        0x261c_3840,
    ),
    (
        BOTH_CARDS_SEED_CARD,
        InputForm::Shared,
        BOTH_CARDS_SEED_BATCH,
        0,
        0x7aba_e00a_7fb3_6a3a,
        0x05f3_bd54,
    ),
];

/// 停机 J-S8：内容锚点有一条不符交回一句；都符交 `None`。
pub(crate) fn content_anchor_finding() -> Option<String> {
    CONTENT_ANCHORS.iter().find_map(|(card_seed, form, batch, unit, first_eight, crc)| {
    let mut bytes = vec![0_u8; disagreement_unit_length(*unit)];
    disagreement_unit_content(*card_seed, *form, *batch, *unit, &mut bytes);
    let head = u64::from_be_bytes(bytes[..8].try_into().expect("8 字节"));
    let computed_crc = crc32_castagnoli(&bytes);
    (head != *first_eight || computed_crc != *crc).then(|| {
        format!(
            "clause_j_s8_content_card_{card_seed}_form_{}_batch_{batch}_unit_{unit}_head_{head:016x}_crc_{computed_crc:08x}",
            form.label()
        )
    })
})
}

/// 这一个单元是不是注入阳性对照的那 16 个之一（不进天然计数）。
pub(super) fn is_an_injected_unit(batch: u64, unit: u64) -> bool {
    INJECTED_BATCHES.contains(&batch)
        && (unit == DETERMINISTIC_INJECTION_UNIT || unit == TRANSIENT_INJECTION_UNIT)
}

/// 每张卡每种输入的天然单元次：批数 × 每批单元数 − 注入的单元数（第七节乙类锚点 1048560）。
pub(crate) fn natural_unit_times(batches: u64, units_per_batch: u64, injected_units: u64) -> u64 {
    batches * units_per_batch - injected_units
}

/// 注入的批数（第 1–8 批）。
pub(super) fn injected_batches() -> u64 {
    INJECTED_BATCHES.end() - INJECTED_BATCHES.start() + 1
}

/// 每张卡每种输入注入的单元数（第 1–8 批各 2 个：P5 一个、P6 一个）。
pub(crate) fn injected_units_per_combination() -> u64 {
    2 * injected_batches()
}

/// 一个单元第几次计算（P6 只在头一次注入）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ComputationOfTheUnit {
    FirstInTheBatch,
    Recompute,
}

/// P5 在分开输入时改核二那一份输入的哪一个字节（每次计算都改，含重算）；别的单元、别的形态交 `None`。
pub(crate) fn kernel_two_input_flip(form: InputForm, batch: u64, unit: u64) -> Option<usize> {
    (form == InputForm::Split
        && INJECTED_BATCHES.contains(&batch)
        && unit == DETERMINISTIC_INJECTION_UNIT)
        .then_some(DETERMINISTIC_INJECTION_BYTE)
}

/// 读回之后把核二那一条的 CRC 异或 1 不：P5 在共用输入时每次（含重算）都注，P6 两种输入都只在头一次计算时注。
pub(super) fn is_the_kernel_two_readback_flipped(
    form: InputForm,
    batch: u64,
    unit: u64,
    computation: ComputationOfTheUnit,
) -> bool {
    if !INJECTED_BATCHES.contains(&batch) {
        return false;
    }
    let deterministic = form == InputForm::Shared && unit == DETERMINISTIC_INJECTION_UNIT;
    let transient =
        unit == TRANSIENT_INJECTION_UNIT && computation == ComputationOfTheUnit::FirstInTheBatch;
    deterministic || transient
}

/// 一个单元一次计算的三样比（登记第五节「每个单元比三样」）。核的结果读不到（没派那一核）交回 `None`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UnitComparison {
    pub(crate) kernels_differ: bool,
    pub(crate) kernel_one_differs_from_the_cpu: bool,
    pub(crate) kernel_two_differs_from_the_cpu: bool,
}

impl UnitComparison {
    pub(crate) fn of(
        kernel_one: Option<u32>,
        kernel_two: Option<u32>,
        cpu: u32,
    ) -> Option<UnitComparison> {
        let (kernel_one, kernel_two) = (kernel_one?, kernel_two?);
        Some(UnitComparison {
            kernels_differ: kernel_one != kernel_two,
            kernel_one_differs_from_the_cpu: kernel_one != cpu,
            kernel_two_differs_from_the_cpu: kernel_two != cpu,
        })
    }

    pub(crate) fn is_any_different(self) -> bool {
        self.kernels_differ
            || self.kernel_one_differs_from_the_cpu
            || self.kernel_two_differs_from_the_cpu
    }

    /// R8：两核一致而与 CPU 不同（第 8 条射程那句「抓不到两份一起错」的实例）。
    pub(crate) fn do_both_kernels_agree_and_differ_from_the_cpu(self) -> bool {
        !self.kernels_differ && self.kernel_one_differs_from_the_cpu
    }
}

/// 派哪几核（重算时两核都派，第九节 V118 盯的那一处）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KernelsToDispatch {
    Both,
    KernelOneOnly,
}

/// 一个单元的重算结局（登记第六节 R7）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecomputeOutcome {
    /// 第 1 次重算就一致。
    Transient,
    /// 第 2 或第 3 次重算才一致。
    AgreedLater,
    /// 3 次重算都不一致。
    Deterministic,
    /// 重算里有一次读不到核的结果。
    Unreadable,
}

impl RecomputeOutcome {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            RecomputeOutcome::Transient => "transient",
            RecomputeOutcome::AgreedLater => "agreed_later",
            RecomputeOutcome::Deterministic => "deterministic",
            RecomputeOutcome::Unreadable => "unreadable",
        }
    }
}

/// 一个不一致的单元单独重算（n = 1，同一种输入），至多 3 次，一致了就停；`recompute` 交回那一次的三样比。交回每次的比与结局。
pub(super) fn recompute_until_agreement(
    mut recompute: impl FnMut(KernelsToDispatch) -> Option<UnitComparison>,
) -> (Vec<Option<UnitComparison>>, RecomputeOutcome) {
    let mut comparisons = Vec::new();
    for recompute_number in 1..=RECOMPUTES_AT_MOST {
        let comparison = recompute(KernelsToDispatch::Both);
        comparisons.push(comparison);
        match comparison {
            None => return (comparisons, RecomputeOutcome::Unreadable),
            Some(agreeing) if !agreeing.is_any_different() => {
                let outcome = if recompute_number == 1 {
                    RecomputeOutcome::Transient
                } else {
                    RecomputeOutcome::AgreedLater
                };
                return (comparisons, outcome);
            }
            Some(_) => {}
        }
    }
    (comparisons, RecomputeOutcome::Deterministic)
}

/// R7 的类别：首轮 0 次 `zero`；有一个 3 次重算都不一致 `deterministic_present`；全部第 1 次重算就一致 `transient_only`；别的 `mixed`。
pub(crate) fn recompute_category(
    first_round_disagreements: u64,
    outcomes: &[RecomputeOutcome],
) -> &'static str {
    if first_round_disagreements == 0 {
        return "zero";
    }
    if outcomes.contains(&RecomputeOutcome::Deterministic) {
        return "deterministic_present";
    }
    if !outcomes.is_empty()
        && outcomes
            .iter()
            .all(|outcome| *outcome == RecomputeOutcome::Transient)
    {
        return "transient_only";
    }
    "mixed"
}

/// R6：零次时单侧 95% 上界 1 − 0.05^(1/n)（n = 1048560 → 2.856992e-06），按 −expm1(ln 0.05 ÷ n) 算免得相减丢位。
pub(crate) fn upper_bound_at_95_percent_when_zero(unit_times: u64) -> f64 {
    -(0.05_f64.ln() / unit_times as f64).exp_m1()
}

/// P5、P6 收尾：注入的 8 个单元全记确定性（或全记暂态）；首轮就没报不一致的那一个记 `None`，判不过。
pub(crate) fn injected_units_hold(
    outcomes: &[Option<RecomputeOutcome>],
    expected: RecomputeOutcome,
) -> bool {
    u64::try_from(outcomes.len()).is_ok_and(|count| count == injected_batches())
        && outcomes.iter().all(|outcome| *outcome == Some(expected))
}

/// 一次计算交回的两核结果（没派的那一核是 `None`），读回之后、注入之前的原样。
pub(crate) type KernelResultsOfAUnit = (Option<u32>, Option<u32>);

/// 一个重算过的单元：单元号、首轮那一次的比、每次重算的比、结局（`name=recompute` 一行）。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RecomputedUnit {
    pub(crate) unit: u64,
    pub(crate) first_round: UnitComparison,
    pub(crate) recomputes: Vec<Option<UnitComparison>>,
    pub(crate) outcome: RecomputeOutcome,
}

/// 第二段一批的结局（登记第六节 R6–R8 的计数，按天然单元算；注入的那几个另记 P5、P6 的结局）。
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct DisagreementBatchOutcome {
    pub(crate) natural_units: u64,
    pub(crate) first_round_unreadable: u64,
    pub(crate) first_round_kernel_disagreements: u64,
    pub(crate) first_round_any_difference: u64,
    pub(crate) both_kernels_agree_and_differ_from_the_cpu: u64,
    pub(crate) kernel_one_differs_from_the_cpu: u64,
    pub(crate) kernel_two_differs_from_the_cpu: u64,
    pub(crate) natural_recomputes: Vec<RecomputedUnit>,
    pub(crate) deterministic_injection: Option<Option<RecomputeOutcome>>,
    pub(crate) transient_injection: Option<Option<RecomputeOutcome>>,
}

/// 第二段一批（登记第五节「每批」）：`first_results` 是这一批写入、两核各派一次、读回的原样结果（按单元次序）；对每个单元先照 P5、P6 注入，
/// 再比三样；三样有一样不同的单元单独重算（`recompute` 交回那一个单元 n = 1 再算一次的原样结果，照样注入），至多 3 次。
/// 注入的单元不进天然计数，它们的结局另记（P5、P6）。
pub(crate) fn process_the_disagreement_batch(
    form: InputForm,
    batch: u64,
    cpu_crcs: &[u32],
    first_results: &[KernelResultsOfAUnit],
    mut recompute: impl FnMut(u64, KernelsToDispatch) -> KernelResultsOfAUnit,
) -> DisagreementBatchOutcome {
    let mut outcome = DisagreementBatchOutcome::default();
    for ((unit_index, cpu_crc), (kernel_one, kernel_two)) in
        cpu_crcs.iter().enumerate().zip(first_results)
    {
        let unit = u64::try_from(unit_index).expect("单元号装得进 u64");
        let flip_of = |computation| {
            u32::from(is_the_kernel_two_readback_flipped(
                form,
                batch,
                unit,
                computation,
            ))
        };
        let kernel_two_seen =
            kernel_two.map(|crc| crc ^ flip_of(ComputationOfTheUnit::FirstInTheBatch));
        let is_injected = is_an_injected_unit(batch, unit);
        let Some(first_round) = UnitComparison::of(*kernel_one, kernel_two_seen, *cpu_crc) else {
            if !is_injected {
                outcome.natural_units += 1;
                outcome.first_round_unreadable += 1;
            }
            continue;
        };
        if !is_injected {
            outcome.natural_units += 1;
            outcome.first_round_kernel_disagreements += u64::from(first_round.kernels_differ);
            outcome.first_round_any_difference += u64::from(first_round.is_any_different());
            outcome.both_kernels_agree_and_differ_from_the_cpu +=
                u64::from(first_round.do_both_kernels_agree_and_differ_from_the_cpu());
            outcome.kernel_one_differs_from_the_cpu +=
                u64::from(first_round.kernel_one_differs_from_the_cpu);
            outcome.kernel_two_differs_from_the_cpu +=
                u64::from(first_round.kernel_two_differs_from_the_cpu);
        }
        let recomputed = first_round.is_any_different().then(|| {
            let (recomputes, recompute_outcome) = recompute_until_agreement(|kernels| {
                let (again_one, again_two) = recompute(unit, kernels);
                let again_two_seen =
                    again_two.map(|crc| crc ^ flip_of(ComputationOfTheUnit::Recompute));
                UnitComparison::of(again_one, again_two_seen, *cpu_crc)
            });
            RecomputedUnit {
                unit,
                first_round,
                recomputes,
                outcome: recompute_outcome,
            }
        });
        let injected_outcome = recomputed
            .as_ref()
            .map(|recomputed_unit| recomputed_unit.outcome);
        match (is_injected, unit) {
            (true, DETERMINISTIC_INJECTION_UNIT) => {
                outcome.deterministic_injection = Some(injected_outcome)
            }
            (true, _) => outcome.transient_injection = Some(injected_outcome),
            (false, _) => outcome.natural_recomputes.extend(recomputed),
        }
    }
    outcome
}

/// R7：重算过的天然单元里，头 `recomputes` 次重算都还不一致的个数（重算里读不到的不算进来，另由结局 `unreadable` 报）。
pub(crate) fn still_disagreeing_after(recomputed: &[RecomputedUnit], recomputes: usize) -> u64 {
    let still = recomputed
        .iter()
        .filter(|unit| {
            unit.recomputes.len() >= recomputes
                && unit.recomputes[..recomputes]
                    .iter()
                    .all(|comparison| comparison.is_some_and(UnitComparison::is_any_different))
        })
        .count();
    u64::try_from(still).expect("个数")
}

/// 第八节轨迹（第二段每批的首轮不一致）：64 批里最多的一批、不一致 > 0 的批数、最后一批的。
pub(crate) fn disagreement_trajectory_text(first_round_disagreements_per_batch: &[u64]) -> String {
    format!(
        "first_round_disagreements_peak_of_a_batch={} batches_with_first_round_disagreements={} first_round_disagreements_of_the_last_batch={}",
        first_round_disagreements_per_batch.iter().copied().max().map_or_else(|| "none".to_string(), |peak| peak.to_string()),
        first_round_disagreements_per_batch.iter().filter(|count| **count > 0).count(),
        first_round_disagreements_per_batch.last().map_or_else(|| "none".to_string(), u64::to_string),
    )
}

/// 第十条 J8：天然单元次不足（有批读不到、卡掉了没算完）。
pub(crate) fn are_the_natural_unit_times_sufficient(natural_units: u64) -> bool {
    natural_units
        >= natural_unit_times(
            DISAGREEMENT_BATCHES,
            UNITS_PER_DISAGREEMENT_BATCH,
            injected_units_per_combination(),
        )
}

/// R8「同一输入同一结果」：两份原样结果逐条相同的条数（两核都比）。
pub(crate) fn identical_entries(
    first: &[KernelResultsOfAUnit],
    again: &[KernelResultsOfAUnit],
) -> u64 {
    let identical = first
        .iter()
        .zip(again)
        .filter(|(earlier, later)| earlier == later)
        .count();
    u64::try_from(identical).expect("条数")
}
