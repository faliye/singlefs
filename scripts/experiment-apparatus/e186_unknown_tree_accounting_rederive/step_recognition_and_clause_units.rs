//! 台阶行的认法、条款单位与它们在一版上成不成立、从手算账本取条款单位、轨迹。

use super::*;

/// 认台阶行的两种认法：D5 意义（按 (统计量, 树) 分、水位变了的那次发布才留），与池级 checker 今天判 I-3.13 时的语法认法
/// （`crates/singlefs-checker/src/walk.rs` 判 I-3.13 的那一段：水位那一统计量、不带设备维、代段早于这一版 txg 的行就跳过）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StepRecognition {
    ByWatermarkChangeHistory,
    ByCheckerSyntax,
}

/// U7（I-3.13）认台阶行用哪一种：登记第六节「台阶行按 D5 意义认，不按语法认」。
const STEP_RECOGNITION_OF_THE_ROW_GENERATION_INVARIANT: StepRecognition = StepRecognition::ByWatermarkChangeHistory;

fn row_counts_as_a_step(row: &AccountingRow, version_txg: u64, history: &WatermarkChangeHistory, recognition: StepRecognition) -> bool {
    match recognition {
        StepRecognition::ByWatermarkChangeHistory => history.is_a_step(row),
        StepRecognition::ByCheckerSyntax => row.statistic == Statistic::InodeNumberWatermark && row.generation_txg < version_txg,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RowsJudged {
    AllRows,
    WatermarkRowsOnly,
}

/// P-gen：这一版里每一行的代段等于这一版的 txg，或者它是一级台阶行。
fn every_row_is_this_generation_or_a_step(version: &AccountingVersion, history: &WatermarkChangeHistory, recognition: StepRecognition, judged: RowsJudged) -> bool {
    version
        .rows
        .iter()
        .filter(|row| match judged {
            RowsJudged::AllRows => true,
            RowsJudged::WatermarkRowsOnly => row.statistic == Statistic::InodeNumberWatermark,
        })
        .all(|row| row.generation_txg == version.txg || row_counts_as_a_step(row, version.txg, history, recognition))
}

/// P-step：上一版的每一级台阶行原样在这一版里。
fn every_step_of_the_previous_version_is_kept(previous: &AccountingVersion, version: &AccountingVersion, history: &WatermarkChangeHistory) -> bool {
    previous.rows.iter().filter(|row| history.is_a_step(row)).all(|step| version.rows.contains(step))
}

/// 登记第六节条款清单 U1–U13（一条 = 一个已定项或不变量清单的一行）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum ClauseUnit {
    /// U1：D5 已定项 2（条文与射程「inode 号水位的台阶行」）。
    DecisionFiveItemTwoGenerationAndSteps,
    /// U2：D5 已定项 4 射程第 12 项那一条。
    DecisionFiveItemFourWatermarkRows,
    /// U3：D8 已定项 1。
    DecisionEightItemOneWholeBatch,
    /// U4：D8 已定项 6「水位行……照台阶行的规矩写」。
    DecisionEightItemSixWatermarkRowRule,
    /// U5：D8 已定项 12 条文「含已定项 8 ㊀ 那类旧写者搬运的记账行」。
    DecisionEightItemTwelveCarriedRows,
    /// U6：D16 已定项 9「每次发布把这一代的行全写一遍」。
    DecisionSixteenItemNineRewriteEachPublish,
    /// U7：I-3.13。
    InvariantGenerationEqualsTheVersionTxg,
    /// U8：I-9.11。
    InvariantWritableHeadHasAWatermarkRow,
    /// U9：I-9.30 ①。
    InvariantWatermarkNotBelowItsSteps,
    /// U10：D18 已定项 24 第 5 条。
    DecisionEighteenItemTwentyFourLineageSteps,
    /// U11：D23 已定项 14「不点删旧代，跳号不留要删的行」。
    DecisionTwentyThreeItemFourteenNoPointDeletion,
    /// U12：D8 已定项 8 ㊀（只在读法 B2 下计）。
    DecisionEightItemEightCarryUnknownRows,
    /// U13：D8 已定项 8 ㊁（只在读法 B2 下计）。
    DecisionEightItemEightRejectUnknownBits,
}

pub(super) const ALL_CLAUSE_UNITS: [ClauseUnit; 13] = [
    ClauseUnit::DecisionFiveItemTwoGenerationAndSteps,
    ClauseUnit::DecisionFiveItemFourWatermarkRows,
    ClauseUnit::DecisionEightItemOneWholeBatch,
    ClauseUnit::DecisionEightItemSixWatermarkRowRule,
    ClauseUnit::DecisionEightItemTwelveCarriedRows,
    ClauseUnit::DecisionSixteenItemNineRewriteEachPublish,
    ClauseUnit::InvariantGenerationEqualsTheVersionTxg,
    ClauseUnit::InvariantWritableHeadHasAWatermarkRow,
    ClauseUnit::InvariantWatermarkNotBelowItsSteps,
    ClauseUnit::DecisionEighteenItemTwentyFourLineageSteps,
    ClauseUnit::DecisionTwentyThreeItemFourteenNoPointDeletion,
    ClauseUnit::DecisionEightItemEightCarryUnknownRows,
    ClauseUnit::DecisionEightItemEightRejectUnknownBits,
];

impl ClauseUnit {
    pub(super) fn registered_label(self) -> &'static str {
        match self {
            ClauseUnit::DecisionFiveItemTwoGenerationAndSteps => "U1",
            ClauseUnit::DecisionFiveItemFourWatermarkRows => "U2",
            ClauseUnit::DecisionEightItemOneWholeBatch => "U3",
            ClauseUnit::DecisionEightItemSixWatermarkRowRule => "U4",
            ClauseUnit::DecisionEightItemTwelveCarriedRows => "U5",
            ClauseUnit::DecisionSixteenItemNineRewriteEachPublish => "U6",
            ClauseUnit::InvariantGenerationEqualsTheVersionTxg => "U7",
            ClauseUnit::InvariantWritableHeadHasAWatermarkRow => "U8",
            ClauseUnit::InvariantWatermarkNotBelowItsSteps => "U9",
            ClauseUnit::DecisionEighteenItemTwentyFourLineageSteps => "U10",
            ClauseUnit::DecisionTwentyThreeItemFourteenNoPointDeletion => "U11",
            ClauseUnit::DecisionEightItemEightCarryUnknownRows => "U12",
            ClauseUnit::DecisionEightItemEightRejectUnknownBits => "U13",
        }
    }

    /// 读法 B1 计不计它：㊀ ㊁ 两条不计（每条非现任臂都要改它，计进去等于先给现任臂一票）。
    pub(super) fn counts_when_the_judged_items_are_excluded(self) -> bool {
        !matches!(self, ClauseUnit::DecisionEightItemEightCarryUnknownRows | ClauseUnit::DecisionEightItemEightRejectUnknownBits)
    }
}

fn latest_watermark_row_of(version: &AccountingVersion, head: TreeOwner) -> Option<AccountingRow> {
    version.rows.iter().filter(|row| row.statistic == Statistic::InodeNumberWatermark && row.tree == head).max_by_key(|row| row.generation_txg).copied()
}

/// 一个头代段不晚于 `generation_limit` 的台阶里最大的水位（D18 已定项 24 第 5 条「代段 ≤ T_rb − 1 的台阶里最大的 H」）。
fn largest_step_watermark_up_to(version: &AccountingVersion, head: TreeOwner, history: &WatermarkChangeHistory, generation_limit: u64) -> Option<u64> {
    version.rows.iter().filter(|row| row.tree == head && history.is_a_step(row) && row.generation_txg <= generation_limit).map(|row| row.value).max()
}

fn all_writable_heads(point: &SamplePoint) -> Vec<TreeOwner> {
    let mut heads = known_writable_heads(point);
    heads.extend(unknown_writable_heads(point));
    heads
}

/// R 里每个不认识的树的维度元组代段最大那一行的值。
pub(super) fn latest_value_of_each_unknown_tuple(initial: &AccountingVersion) -> BTreeMap<(Statistic, TreeOwner), u64> {
    let mut latest: BTreeMap<(Statistic, TreeOwner), (u64, u64)> = BTreeMap::new();
    for row in initial.rows.iter().filter(|row| row.belongs_to_an_unknown_tree()) {
        let entry = latest.entry(row.tuple()).or_insert((row.generation_txg, row.value));
        if row.generation_txg >= entry.0 {
            *entry = (row.generation_txg, row.value);
        }
    }
    latest.into_iter().map(|(tuple, (_generation_txg, value))| (tuple, value)).collect()
}

/// R 里属于不认识的树的行数（第二次跑新 U5 的 N）。
pub(super) fn unknown_tree_row_count_of(initial: &AccountingVersion) -> u64 {
    initial.rows.iter().filter(|row| row.belongs_to_an_unknown_tree()).count().try_into().expect("计数装得进 u64")
}

/// 新 U5 的 W_g：写者在 R_g 这一次为不认识的树换了代号写进去的行（读法第 10 条；照抄的台阶行、原样留着的行都不算）。
fn rows_the_writer_wrote_for_unknown_trees(version: &AccountingVersion) -> Vec<AccountingRow> {
    version.rows_written_for_unknown_trees.clone()
}

/// 第二次跑的新 U5（登记第二次跑第六节）：N = 0 ⇒ 真（没有对象）；N > 0 且 W_g 为空 ⇒ 假（条款成了空指称）；N > 0 且 W_g 非空 ⇒ W_g 每一行代段 = R_g 的 txg 才真。
fn dangling_carried_rows_clause_holds(initial: &AccountingVersion, version: &AccountingVersion) -> bool {
    let unknown_tree_row_count = unknown_tree_row_count_of(initial);
    let written = rows_the_writer_wrote_for_unknown_trees(version);
    if unknown_tree_row_count == 0 { return true; }
    if written.is_empty() { return false; }
    written.iter().all(|row| row.generation_txg == version.txg)
}

/// 第六节表 U 列每一条的谓词，在 R_g（`version`）上判，`previous` 是 R_{g−1}，`initial` 是 R。
#[allow(clippy::too_many_arguments, reason = "这些参数是这一次计算彼此独立的输入，合成只用一次的结构体只多一层包装")]
pub(super) fn clause_holds_on_version(
    clause: ClauseUnit,
    initial: &AccountingVersion,
    previous: &AccountingVersion,
    version: &AccountingVersion,
    point: &SamplePoint,
    history: &WatermarkChangeHistory,
    flags_policy: UnknownFlagsPolicy,
    judgement: Judgement,
) -> bool {
    let generation_rule = |recognition: StepRecognition, judged: RowsJudged| every_row_is_this_generation_or_a_step(version, history, recognition, judged);
    match clause {
        ClauseUnit::DecisionFiveItemTwoGenerationAndSteps => {
            generation_rule(StepRecognition::ByWatermarkChangeHistory, RowsJudged::AllRows) && every_step_of_the_previous_version_is_kept(previous, version, history)
        }
        ClauseUnit::DecisionFiveItemFourWatermarkRows | ClauseUnit::DecisionEightItemSixWatermarkRowRule => {
            generation_rule(StepRecognition::ByWatermarkChangeHistory, RowsJudged::WatermarkRowsOnly)
        }
        ClauseUnit::DecisionEightItemOneWholeBatch | ClauseUnit::DecisionSixteenItemNineRewriteEachPublish => {
            generation_rule(StepRecognition::ByWatermarkChangeHistory, RowsJudged::AllRows)
        }
        ClauseUnit::DecisionEightItemTwelveCarriedRows => match judgement {
            Judgement::VacuousCarriedRowsClause => version.rows_written_for_unknown_trees.iter().all(|row| row.generation_txg == version.txg),
            Judgement::DanglingCarriedRowsClause => dangling_carried_rows_clause_holds(initial, version),
        },
        ClauseUnit::InvariantGenerationEqualsTheVersionTxg => generation_rule(STEP_RECOGNITION_OF_THE_ROW_GENERATION_INVARIANT, RowsJudged::AllRows),
        ClauseUnit::InvariantWritableHeadHasAWatermarkRow => {
            let heads_that_must_have_a_watermark_row: Vec<TreeOwner> = known_writable_heads(point).into_iter().chain(unknown_writable_heads(point)).collect();
            heads_that_must_have_a_watermark_row
                .iter()
                .all(|head| version.rows.iter().any(|row| row.statistic == Statistic::InodeNumberWatermark && row.tree == *head))
        }
        ClauseUnit::InvariantWatermarkNotBelowItsSteps => all_writable_heads(point).into_iter().all(|head| match latest_watermark_row_of(version, head) {
            None => true,
            Some(latest_row) => version.rows.iter().filter(|row| row.tree == head && history.is_a_step(row)).all(|step| step.value <= latest_row.value),
        }),
        ClauseUnit::DecisionEighteenItemTwentyFourLineageSteps => all_writable_heads(point).into_iter().all(|head| {
            largest_step_watermark_up_to(previous, head, history, previous.txg) == largest_step_watermark_up_to(version, head, history, version.txg - 1)
        }),
        // 模型每一版整棵重建，没有点删这一步：空真。
        ClauseUnit::DecisionTwentyThreeItemFourteenNoPointDeletion => true,
        ClauseUnit::DecisionEightItemEightCarryUnknownRows => latest_value_of_each_unknown_tuple(initial)
            .into_iter()
            .all(|(tuple, value)| version.rows.iter().any(|row| row.tuple() == tuple && row.generation_txg == version.txg && row.value == value)),
        ClauseUnit::DecisionEightItemEightRejectUnknownBits => flags_policy == UnknownFlagsPolicy::RejectEntry,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ClauseReading {
    /// 判的那一种：D8 已定项 8 的 ㊀ ㊁ 不计。
    ExcludesTheJudgedItems,
    /// 附带量：㊀ ㊁ 各算一条。
    CountsTheJudgedItems,
}

/// 第七节「条款账本（B1）」与「B2 增项」逐格照抄，与装置的谓词是两条路（第十节「账本两条路」）。
/// `dangling-u5` 下的账本是第二次跑登记第七节那张新表：`NoCarry`、随根留着在 `h_new > 0` 的各列各比第一次跑的表多 1（新 U5）。
pub(super) fn clause_units_from_the_hand_ledger(
    point: &SamplePoint,
    flags_policy: UnknownFlagsPolicy,
    accounting_policy: AccountingPolicy,
    reading: ClauseReading,
    judgement: Judgement,
) -> u64 {
    let accounting_arm_acts = matches!(effective_flags_policy_of(flags_policy, point), UnknownFlagsPolicy::Skip | UnknownFlagsPolicy::RejectEntry);
    let has_unknown_heads = point.unknown_writable_head_count > 0;
    let no_step_at_all = point.earlier_step_count == 0 && !point.initial_publish_changed_the_watermark;
    let ledger_excluding_the_judged_items = if !accounting_arm_acts || !has_unknown_heads {
        0
    } else {
        match (judgement, accounting_policy) {
            (_, AccountingPolicy::Carry) => 0,
            (Judgement::VacuousCarriedRowsClause, AccountingPolicy::NoCarry) => u64::from(no_step_at_all),
            (Judgement::DanglingCarriedRowsClause, AccountingPolicy::NoCarry) => {
                if no_step_at_all {
                    2
                } else {
                    1
                }
            }
            (Judgement::VacuousCarriedRowsClause, AccountingPolicy::KeepWithRoot) => match (point.initial_publish_changed_the_watermark, point.tree_dimension_statistic_count) {
                (false, _) => 6,
                (true, 1) => 0,
                (true, _) => 4,
            },
            (Judgement::DanglingCarriedRowsClause, AccountingPolicy::KeepWithRoot) => match (point.initial_publish_changed_the_watermark, point.tree_dimension_statistic_count) {
                (false, _) => 7,
                (true, 1) => 1,
                (true, _) => 5,
            },
        }
    };
    match reading {
        ClauseReading::ExcludesTheJudgedItems => ledger_excluding_the_judged_items,
        ClauseReading::CountsTheJudgedItems => {
            let carry_item_extra = u64::from(accounting_arm_acts && has_unknown_heads && accounting_policy != AccountingPolicy::Carry);
            let reject_item_extra = u64::from(flags_policy != UnknownFlagsPolicy::RejectEntry);
            ledger_excluding_the_judged_items + carry_item_extra + reject_item_extra
        }
    }
}

/// 一个量在 R_1 … R_G 上的三个数：峰值、为正的版数、期末值（R 那一版是初始供给、不算）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Trajectory {
    pub(super) peak: u64,
    pub(super) positive_versions: u64,
    pub(super) final_value: u64,
}

/// 没有新版本（只读挂载）时没有轨迹，交回 `None`，产物里写 `no_new_version`，不当 0。
pub(super) fn trajectory_of(values_by_version: &[u64]) -> Option<Trajectory> {
    let final_value = *values_by_version.last()?;
    Some(Trajectory {
        peak: values_by_version.iter().copied().max().unwrap_or(0),
        positive_versions: values_by_version.iter().filter(|value| **value > 0).count().try_into().expect("计数装得进 u64"),
        final_value,
    })
}

pub(super) fn render_trajectory(trajectory: Option<Trajectory>) -> String {
    match trajectory {
        Some(trajectory) => format!("{}/{}/{}", trajectory.peak, trajectory.positive_versions, trajectory.final_value),
        None => "no_new_version".to_string(),
    }
}

/// `crates/` 今天的台阶认法（`TransactionOutput::inode_number_watermark_steps` 与 checker 的 `inode_number_watermark_steps_of_a_version`
/// 同一条，按字段表另写一份）：水位那一统计量代段早于这一版 txg 的行都是台阶，按代段升序；代段等于这一版 txg 的第一行（key 次序），
/// 值与代段最大的台阶不同（或没有更早的台阶）也是台阶。不按树分。
pub(super) fn steps_by_the_crates_reading(version: &AccountingVersion) -> Vec<AccountingRow> {
    let mut steps: Vec<AccountingRow> =
        version.rows.iter().filter(|row| row.statistic == Statistic::InodeNumberWatermark && row.generation_txg < version.txg).copied().collect();
    steps.sort_by_key(|row| row.generation_txg);
    let row_of_this_version = version.rows.iter().find(|row| row.statistic == Statistic::InodeNumberWatermark && row.generation_txg == version.txg);
    if let Some(row) = row_of_this_version {
        if steps.last().is_none_or(|latest_step| latest_step.value != row.value) {
            steps.push(*row);
        }
    }
    steps
}

pub(super) fn steps_by_the_watermark_change_history(version: &AccountingVersion, history: &WatermarkChangeHistory) -> Vec<AccountingRow> {
    let mut steps: Vec<AccountingRow> = version.rows.iter().filter(|row| history.is_a_step(row)).copied().collect();
    steps.sort_by_key(|row| row.generation_txg);
    steps
}

/// I-3.13 按池级 checker 今天的语法认法判出的违例行数（附带量，停机条款第 3 项照它写）。
pub(super) fn row_generation_invariant_violations_under_the_checker_syntax(version: &AccountingVersion, history: &WatermarkChangeHistory) -> u64 {
    version
        .rows
        .iter()
        .filter(|row| !row_counts_as_a_step(row, version.txg, history, StepRecognition::ByCheckerSyntax) && row.generation_txg != version.txg)
        .count().try_into().expect("计数装得进 u64")
}

pub(super) fn rows_of_tuple(version: &AccountingVersion, tuple: (Statistic, TreeOwner)) -> u64 {
    version.rows.iter().filter(|row| row.tuple() == tuple).count().try_into().expect("计数装得进 u64")
}
