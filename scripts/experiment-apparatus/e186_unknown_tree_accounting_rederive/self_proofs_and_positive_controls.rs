//! 各问法的自证、阳性对照与阴性对照的失败条目。

use super::*;

/// 第八节判别力自证·Q2：钉住选定读法（Q2「计」）下同一记账臂 `RejectEntry` 格与 `Skip` 格第 6 位的比较，翻 Q2 那一位之后比较变了才算由绿转红。
pub(super) fn flags_question_self_proof_turns_red(evaluated: &EvaluatedPoint, accounting_policy: AccountingPolicy) -> bool {
    let pinned_reading = JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED;
    let flipped_reading = pinned_reading.with_reject_entry_semantics(RejectEntrySemanticsReading::NotCounted);
    let pinned = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, pinned_reading)[5].cmp(&evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, pinned_reading)[5]);
    let after_flip =
        evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, flipped_reading)[5].cmp(&evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, flipped_reading)[5]);
    pinned != after_flip
}

/// 第八节判别力自证·Q1 第 6 位：钉住 `Carry` 与随根留着第 6 位的比较，把随根留着的第 6 位改成等于 `Carry` 的值之后比较变了才算由绿转红。
pub(super) fn accounting_question_position_six_self_proof_turns_red(evaluated: &EvaluatedPoint, flags_policy: UnknownFlagsPolicy) -> bool {
    let carry = evaluated.vector(flags_policy, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let keep_with_root = evaluated.vector(flags_policy, AccountingPolicy::KeepWithRoot, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let pinned = carry[5].cmp(&keep_with_root[5]);
    let corrupted_keep_with_root_position_six = carry[5];
    pinned != carry[5].cmp(&corrupted_keep_with_root_position_six)
}

/// 第八节判别力自证·Q1 第 7 位：钉住两格第 7 位的比较，把 `Carry` 的第 7 位减去 `h_new` × `r_tree_dim` 之后比较变了才算由绿转红。
pub(super) fn accounting_question_position_seven_self_proof_turns_red(evaluated: &EvaluatedPoint, flags_policy: UnknownFlagsPolicy) -> bool {
    let carry = evaluated.vector(flags_policy, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let keep_with_root = evaluated.vector(flags_policy, AccountingPolicy::KeepWithRoot, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let pinned = carry[6].cmp(&keep_with_root[6]);
    let corrupted_carry_position_seven = carry[6] - evaluated.point.unknown_writable_head_count * evaluated.point.tree_dimension_statistic_count;
    pinned != corrupted_carry_position_seven.cmp(&keep_with_root[6])
}

/// 第二次跑第八节新自证（`dangling-u5`）：钉住 `Carry` 与 `NoCarry` 第 6 位的比较，把 `NoCarry` 的 U5 换回空真读法（第 6 位减 1）之后比较变了才算由绿转红。
/// 跑在 `h_new = 2`、`lpc = 1`、`r_tree_dim = 1` 的点上：第一次跑的判据下 `Carry` 与 `NoCarry` 在这里第 6 位相等。
pub(super) fn accounting_question_dangling_carried_rows_self_proof_turns_red(evaluated: &EvaluatedPoint, flags_policy: UnknownFlagsPolicy) -> bool {
    let carry = evaluated.vector(flags_policy, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let no_carry = evaluated.vector(flags_policy, AccountingPolicy::NoCarry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let pinned = carry[5].cmp(&no_carry[5]);
    let no_carry_position_six_under_the_vacuous_reading = no_carry[5].saturating_sub(1);
    pinned != carry[5].cmp(&no_carry_position_six_under_the_vacuous_reading)
}

/// 第二次跑第八节新自证的取样点：`h_new = 2`、`lpc = 1`、`r_tree_dim = 1`。
pub(super) fn is_a_point_of_the_dangling_carried_rows_self_proof(point: &SamplePoint) -> bool {
    point.unknown_writable_head_count == 2 && point.initial_publish_changed_the_watermark && point.tree_dimension_statistic_count == 1
}

/// 第五节阳性对照表的每一行（`RejectEntry` 那一行有两条）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PositiveControl {
    IgnoreViolatesSemantics,
    SkipFreesReferencedBlocks,
    RejectEntryCountedSemanticsAddsOne,
    RejectEntryNoCarryLosesTuples,
    ReadOnlyNeverPublishes,
    PerBitEqualsTheArmItReducesTo,
    CarryWritesTheUnknownRows,
    NoCarryLosesTuples,
    KeepWithRootKeepsTheInitialRow,
    /// 第二次跑第五节补的三条（新 U5，三条记账臂各一条），只在 `dangling-u5` 下跑。
    DanglingCarriedRowsCarryHolds,
    DanglingCarriedRowsNoCarryDangles,
    DanglingCarriedRowsKeepWithRootDangles,
}

/// 新 U5 的三条阳性对照（`dangling-u5` 下接在 `POSITIVE_CONTROLS` 后面跑）。
pub(super) const DANGLING_CARRIED_ROWS_POSITIVE_CONTROLS: [PositiveControl; 3] =
    [PositiveControl::DanglingCarriedRowsCarryHolds, PositiveControl::DanglingCarriedRowsNoCarryDangles, PositiveControl::DanglingCarriedRowsKeepWithRootDangles];

pub(super) fn positive_controls_of(judgement: Judgement) -> Vec<PositiveControl> {
    let mut controls = POSITIVE_CONTROLS.to_vec();
    if judgement == Judgement::DanglingCarriedRowsClause {
        controls.extend(DANGLING_CARRIED_ROWS_POSITIVE_CONTROLS);
    }
    controls
}

pub(super) const POSITIVE_CONTROLS: [PositiveControl; 9] = [
    PositiveControl::IgnoreViolatesSemantics,
    PositiveControl::SkipFreesReferencedBlocks,
    PositiveControl::RejectEntryCountedSemanticsAddsOne,
    PositiveControl::RejectEntryNoCarryLosesTuples,
    PositiveControl::ReadOnlyNeverPublishes,
    PositiveControl::PerBitEqualsTheArmItReducesTo,
    PositiveControl::CarryWritesTheUnknownRows,
    PositiveControl::NoCarryLosesTuples,
    PositiveControl::KeepWithRootKeepsTheInitialRow,
];

impl PositiveControl {
    pub(super) fn label(self) -> &'static str {
        match self {
            PositiveControl::IgnoreViolatesSemantics => "Ignore",
            PositiveControl::SkipFreesReferencedBlocks => "Skip",
            PositiveControl::RejectEntryCountedSemanticsAddsOne => "RejectEntry_counted_semantics_adds_one",
            PositiveControl::RejectEntryNoCarryLosesTuples => "RejectEntry_no_carry_loses_tuples",
            PositiveControl::ReadOnlyNeverPublishes => "ReadOnly",
            PositiveControl::PerBitEqualsTheArmItReducesTo => "PerBit",
            PositiveControl::CarryWritesTheUnknownRows => "Carry",
            PositiveControl::NoCarryLosesTuples => "NoCarry",
            PositiveControl::KeepWithRootKeepsTheInitialRow => "KeepWithRoot",
            PositiveControl::DanglingCarriedRowsCarryHolds => "DanglingU5_Carry",
            PositiveControl::DanglingCarriedRowsNoCarryDangles => "DanglingU5_NoCarry",
            PositiveControl::DanglingCarriedRowsKeepWithRootDangles => "DanglingU5_KeepWithRoot",
        }
    }
}

/// 第五节阳性对照，每条臂一条；交回 (这一点上查了几格, 其中不成立的几格)。
pub(super) fn positive_control_failures(evaluated: &EvaluatedPoint, control: PositiveControl) -> (u64, u64) {
    let point = &evaluated.point;
    let mut checked = 0u64;
    let mut failures = 0u64;
    let mut judge = |holds: bool| {
        checked += 1;
        failures += u64::from(!holds);
    };
    let has_unknown_heads = point.unknown_writable_head_count > 0;
    let acting_flags: Vec<UnknownFlagsPolicy> = FLAGS_ARMS.iter().copied().filter(|flags_policy| accounting_arm_acts_under(*flags_policy, point)).collect();
    match control {
        PositiveControl::IgnoreViolatesSemantics => {
            for accounting_policy in ACCOUNTING_ARMS {
                let cell = evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Ignore, accounting_policy);
                judge(cell.read_only_violations > 0 && cell.revived_dead_trees == 1);
            }
        }
        PositiveControl::SkipFreesReferencedBlocks if point.invisible_window_generations > 0 => {
            for accounting_policy in ACCOUNTING_ARMS {
                judge(evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, accounting_policy).blocks_freed_while_referenced > 0);
            }
        }
        PositiveControl::RejectEntryCountedSemanticsAddsOne => {
            for accounting_policy in ACCOUNTING_ARMS {
                let counted = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
                let not_counted = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
                judge(counted[5] == not_counted[5] + 1);
            }
        }
        PositiveControl::RejectEntryNoCarryLosesTuples if has_unknown_heads && point.tree_dimension_statistic_count == 2 => {
            judge(evaluated.outcome_under_the_arms(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::NoCarry).rows_lost > 0);
        }
        PositiveControl::ReadOnlyNeverPublishes => {
            for accounting_policy in ACCOUNTING_ARMS {
                judge(evaluated.outcome_under_the_arms(UnknownFlagsPolicy::ReadOnly, accounting_policy).writable_generations == 0);
            }
        }
        PositiveControl::PerBitEqualsTheArmItReducesTo => {
            let reduces_to = if point.trees_with_registered_bit == 0 { UnknownFlagsPolicy::ReadOnly } else { UnknownFlagsPolicy::Skip };
            for accounting_policy in ACCOUNTING_ARMS {
                for reading in [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED] {
                    judge(evaluated.vector(UnknownFlagsPolicy::PerBit, accounting_policy, reading) == evaluated.vector(reduces_to, accounting_policy, reading));
                }
            }
        }
        PositiveControl::CarryWritesTheUnknownRows if has_unknown_heads => {
            for flags_policy in &acting_flags {
                let extra = point.unknown_writable_head_count * point.tree_dimension_statistic_count;
                let carry = evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::Carry).rows_written_this_generation;
                let no_carry = evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::NoCarry).rows_written_this_generation;
                judge(carry == no_carry + extra && extra > 0);
            }
        }
        PositiveControl::NoCarryLosesTuples if has_unknown_heads && (point.tree_dimension_statistic_count == 2 || !point.is_legal_under_todays_step_rows()) => {
            for flags_policy in &acting_flags {
                judge(evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::NoCarry).rows_lost > 0);
            }
        }
        PositiveControl::KeepWithRootKeepsTheInitialRow if has_unknown_heads && !point.initial_publish_changed_the_watermark => {
            for flags_policy in &acting_flags {
                let cell = evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::KeepWithRoot);
                let every_version_keeps_an_initial_row = cell.trajectory_unknown_rows_at_the_initial_txg.is_some_and(|trajectory| trajectory.positive_versions == point.published_generations);
                let row_generation_invariant_false = cell
                    .clause_false_trajectories
                    .iter()
                    .any(|(clause, trajectory)| *clause == ClauseUnit::InvariantGenerationEqualsTheVersionTxg && trajectory.is_some_and(|trajectory| trajectory.peak > 0));
                judge(every_version_keeps_an_initial_row && row_generation_invariant_false);
            }
        }
        // 新 U5 的三条（第二次跑第五节补的表）：`Carry` 每一版都为真，另两条每一版都为假（为正版数 = G）。
        PositiveControl::DanglingCarriedRowsCarryHolds if has_unknown_heads => {
            for flags_policy in &acting_flags {
                let trajectory = carried_rows_clause_false_trajectory_of(evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::Carry));
                judge(trajectory.is_some_and(|trajectory| trajectory.peak == 0 && trajectory.positive_versions == 0 && trajectory.final_value == 0));
            }
        }
        PositiveControl::DanglingCarriedRowsNoCarryDangles | PositiveControl::DanglingCarriedRowsKeepWithRootDangles if has_unknown_heads => {
            let accounting_policy = if control == PositiveControl::DanglingCarriedRowsNoCarryDangles { AccountingPolicy::NoCarry } else { AccountingPolicy::KeepWithRoot };
            for flags_policy in &acting_flags {
                let trajectory = carried_rows_clause_false_trajectory_of(evaluated.outcome_under_the_arms(*flags_policy, accounting_policy));
                judge(trajectory.is_some_and(|trajectory| trajectory.peak == 1 && trajectory.positive_versions == point.published_generations && trajectory.final_value == 1));
            }
        }
        // 带条件的那几行在条件不成立的点上不查（第五节「跑在哪些点」）。
        PositiveControl::SkipFreesReferencedBlocks
        | PositiveControl::RejectEntryNoCarryLosesTuples
        | PositiveControl::CarryWritesTheUnknownRows
        | PositiveControl::NoCarryLosesTuples
        | PositiveControl::KeepWithRootKeepsTheInitialRow
        | PositiveControl::DanglingCarriedRowsCarryHolds
        | PositiveControl::DanglingCarriedRowsNoCarryDangles
        | PositiveControl::DanglingCarriedRowsKeepWithRootDangles => {}
    }
    (checked, failures)
}

/// U5 在 R_1 … R_G 上的为假轨迹；没有新版本（只读挂载）交回 `None`。
pub(super) fn carried_rows_clause_false_trajectory_of(cell: &CellOutcome) -> Option<Trajectory> {
    cell.clause_false_trajectories
        .iter()
        .find(|(clause, _trajectory)| *clause == ClauseUnit::DecisionEightItemTwelveCarriedRows)
        .and_then(|(_clause, trajectory)| *trajectory)
}

/// 阴性对照甲：`w_gens = 0` 的点上任一格误释放 > 0 ⇒ 误释放是硬编码的。交回 (查了几格, 不成立几格)。
pub(super) fn negative_control_no_window_failures(evaluated: &EvaluatedPoint) -> (u64, u64) {
    if evaluated.point.invisible_window_generations > 0 {
        return (0, 0);
    }
    let failures: u64 = evaluated.cells.iter().filter(|cell| cell.blocks_freed_while_referenced > 0).count().try_into().expect("计数装得进 u64");
    (evaluated.cells.len().try_into().expect("计数装得进 u64"), failures)
}

/// 阴性对照乙：`h_new = 0` 的点上，同一 flags 臂下三条记账格七位（判的两种读法）有一位不同，或某条 U 为假 ⇒ 记账效应是硬编码的。
/// 「某条 U」只取读法 B1 下计的那几条（U1–U11，登记第十二节修订）：U12、U13 只在 B2 下计，是附带量，
/// 由 `negative_control_no_unknown_heads_incidental_clauses_counting_the_judged_items` 照报、不判。
pub(super) fn negative_control_no_unknown_heads_failures(evaluated: &EvaluatedPoint) -> (u64, u64) {
    if evaluated.point.unknown_writable_head_count > 0 {
        return (0, 0);
    }
    let mut checked = 0u64;
    let mut failures = 0u64;
    for flags_policy in FLAGS_ARMS {
        for reading in [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED] {
            let vectors: Vec<LexicographicVector> = ACCOUNTING_ARMS.iter().map(|accounting_policy| evaluated.vector(flags_policy, *accounting_policy, reading)).collect();
            checked += 1;
            failures += u64::from(first_separating_position(&vectors).is_some());
        }
    }
    for cell in &evaluated.cells {
        for (clause, trajectory) in &cell.clause_false_trajectories {
            if !clause.counts_when_the_judged_items_are_excluded() {
                continue;
            }
            checked += 1;
            failures += u64::from(trajectory.is_some_and(|trajectory| trajectory.peak > 0));
        }
    }
    (checked, failures)
}

/// 阴性对照乙的附带量：`h_new = 0` 的点上 U12（任一版为假）与 U13（按臂判）各有几格为假。照报、不判。
/// 交回 (查了几格, U12 为假几格, U13 为假几格)。
pub(super) fn negative_control_no_unknown_heads_incidental_clauses_counting_the_judged_items(evaluated: &EvaluatedPoint) -> (u64, u64, u64) {
    if evaluated.point.unknown_writable_head_count > 0 {
        return (0, 0, 0);
    }
    let mut checked = 0u64;
    let mut carry_item_false = 0u64;
    let mut reject_item_false = 0u64;
    for cell in &evaluated.cells {
        checked += 1;
        for (clause, trajectory) in &cell.clause_false_trajectories {
            match clause {
                ClauseUnit::DecisionEightItemEightCarryUnknownRows => carry_item_false += u64::from(trajectory.is_some_and(|trajectory| trajectory.peak > 0)),
                ClauseUnit::DecisionEightItemEightRejectUnknownBits => reject_item_false += u64::from(cell.flags_policy != UnknownFlagsPolicy::RejectEntry),
                ClauseUnit::DecisionFiveItemTwoGenerationAndSteps
                | ClauseUnit::DecisionFiveItemFourWatermarkRows
                | ClauseUnit::DecisionEightItemOneWholeBatch
                | ClauseUnit::DecisionEightItemSixWatermarkRowRule
                | ClauseUnit::DecisionEightItemTwelveCarriedRows
                | ClauseUnit::DecisionSixteenItemNineRewriteEachPublish
                | ClauseUnit::InvariantGenerationEqualsTheVersionTxg
                | ClauseUnit::InvariantWritableHeadHasAWatermarkRow
                | ClauseUnit::InvariantWatermarkNotBelowItsSteps
                | ClauseUnit::DecisionEighteenItemTwentyFourLineageSteps
                | ClauseUnit::DecisionTwentyThreeItemFourteenNoPointDeletion => {}
            }
        }
    }
    (checked, carry_item_false, reject_item_false)
}
