//! 取样点的变动、灵敏度成对的判定与锚点的失败条目。

use super::*;

/// 取样点只改一维（几何敏感性、阴性对照丙、`G` 的甲类锚点找配对的点用）。
#[derive(Clone, Copy, Debug)]
pub(super) enum PointChange {
    Geometry(Geometry),
    RetiredKeptGenerationCount(u64),
    InvisibleWindowGenerations(u64),
    UnknownWritableHeads(u64),
    RegisteredTrees(u64),
    PublishedGenerations(u64),
    StepHistory(u64, bool),
    TreeDimensionStatistics(u64),
}

pub(super) fn point_with(point: &SamplePoint, change: PointChange) -> SamplePoint {
    let mut changed = *point;
    match change {
        PointChange::Geometry(geometry) => changed.geometry = geometry,
        PointChange::RetiredKeptGenerationCount(retired_kept_generation_count) => changed.retired_kept_generation_count = retired_kept_generation_count,
        PointChange::InvisibleWindowGenerations(generations) => changed.invisible_window_generations = generations,
        PointChange::UnknownWritableHeads(head_count) => changed.unknown_writable_head_count = head_count,
        PointChange::RegisteredTrees(tree_count) => changed.trees_with_registered_bit = tree_count,
        PointChange::PublishedGenerations(generations) => changed.published_generations = generations,
        PointChange::StepHistory(earlier_step_count, initial_publish_changed_the_watermark) => {
            changed.earlier_step_count = earlier_step_count;
            changed.initial_publish_changed_the_watermark = initial_publish_changed_the_watermark;
        }
        PointChange::TreeDimensionStatistics(statistic_count) => changed.tree_dimension_statistic_count = statistic_count,
    }
    changed
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum SensitivityVerdict {
    AccountingWinners,
    FlagsWinnersAndRejectEntryVersusSkip,
}

/// 第八节「几何敏感性」表：每对两点方向相反，判决那一格在两点上都报。
pub(super) struct SensitivityPair {
    pub(super) name: &'static str,
    pub(super) verdict: SensitivityVerdict,
    applies_to: fn(&SamplePoint) -> bool,
    change: PointChange,
}

pub(super) fn sensitivity_pairs() -> Vec<SensitivityPair> {
    let mut pairs = vec![
        SensitivityPair {
            name: "lpc_0_to_1_where_h_new_positive_and_r_tree_dim_1",
            verdict: SensitivityVerdict::AccountingWinners,
            applies_to: |point| point.unknown_writable_head_count > 0 && point.tree_dimension_statistic_count == 1 && !point.initial_publish_changed_the_watermark,
            change: PointChange::StepHistory(0, true),
        },
        SensitivityPair { name: "r_tree_dim_1_to_2", verdict: SensitivityVerdict::AccountingWinners, applies_to: |point| point.tree_dimension_statistic_count == 1, change: PointChange::TreeDimensionStatistics(2) },
        SensitivityPair { name: "registered_0_to_4", verdict: SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip, applies_to: |point| point.trees_with_registered_bit == 0, change: PointChange::RegisteredTrees(UNKNOWN_TREE_COUNT) },
    ];
    // lpc 0 → 1 保留各点自己的 s：StepHistory 的 s 在用时按那一点改写（见 `partner_of`）。
    for (earlier_step_count, initial_publish_changed_the_watermark) in STEP_HISTORY_VALUES.into_iter().skip(1) {
        pairs.push(SensitivityPair {
            name: match (earlier_step_count, initial_publish_changed_the_watermark) {
                (0, true) => "s_lpc_0_0_to_0_1",
                (1, false) => "s_lpc_0_0_to_1_0",
                (1, true) => "s_lpc_0_0_to_1_1",
                (3, false) => "s_lpc_0_0_to_3_0",
                _ => "s_lpc_0_0_to_3_1",
            },
            verdict: SensitivityVerdict::AccountingWinners,
            applies_to: |point| !point.is_legal_under_todays_step_rows(),
            change: PointChange::StepHistory(earlier_step_count, initial_publish_changed_the_watermark),
        });
    }
    for (name, generations) in [("w_gens_0_to_1", 1), ("w_gens_0_to_3", 3), ("w_gens_0_to_8", 8)] {
        pairs.push(SensitivityPair { name, verdict: SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip, applies_to: |point| point.invisible_window_generations == 0, change: PointChange::InvisibleWindowGenerations(generations) });
    }
    for (name, head_count) in [("h_new_0_to_2", 2), ("h_new_0_to_4", 4)] {
        pairs.push(SensitivityPair { name, verdict: SensitivityVerdict::AccountingWinners, applies_to: |point| point.unknown_writable_head_count == 0, change: PointChange::UnknownWritableHeads(head_count) });
    }
    for verdict in [SensitivityVerdict::AccountingWinners, SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip] {
        pairs.push(SensitivityPair {
            name: "geometry_e113_to_implementation_two_devices",
            verdict,
            applies_to: |point| point.geometry == Geometry::UnknownTreeFullArms,
            change: PointChange::Geometry(Geometry::ImplementationTwoDevices),
        });
        pairs.push(SensitivityPair { name: "g_gens_1_to_10", verdict, applies_to: |point| point.published_generations == 1, change: PointChange::PublishedGenerations(10) });
        pairs.push(SensitivityPair { name: "k_keep_1_to_20", verdict, applies_to: |point| point.retired_kept_generation_count == 1, change: PointChange::RetiredKeptGenerationCount(20) });
    }
    pairs
}

/// 配对的那一点：lpc 0 → 1 那一对只翻 lpc、保留这一点自己的 s；别的照 `change` 改。
pub(super) fn partner_of(pair: &SensitivityPair, point: &SamplePoint) -> Option<SamplePoint> {
    if !(pair.applies_to)(point) {
        return None;
    }
    let change = match (pair.name, pair.change) {
        ("lpc_0_to_1_where_h_new_positive_and_r_tree_dim_1", PointChange::StepHistory(_, _)) => PointChange::StepHistory(point.earlier_step_count, true),
        (_, change) => change,
    };
    Some(point_with(point, change))
}

/// 第七节乙类（独立算出、用命令核过）与甲类的绝对值，逐条在全部取样点里对；交回 (名字, 查了几格, 对不上几格)。
pub(super) fn anchor_failures(evaluated_points: &[EvaluatedPoint]) -> Vec<(String, u64, u64)> {
    let mut results = Vec::new();
    let mut check = |name: String, select: &dyn Fn(&SamplePoint) -> bool, value_of: &dyn Fn(&EvaluatedPoint) -> u64, expected: &dyn Fn(&SamplePoint) -> u64| {
        let mut checked = 0u64;
        let mut mismatches = 0u64;
        for evaluated in evaluated_points.iter().filter(|evaluated| select(&evaluated.point)) {
            checked += 1;
            mismatches += u64::from(value_of(evaluated) != expected(&evaluated.point));
        }
        results.push((name, checked, mismatches));
    };
    check(
        "own_rows_r_tree_dim_1_e113_17_implementation_15".to_string(),
        &|point| point.tree_dimension_statistic_count == 1,
        &|evaluated| evaluated.point.geometry.rows_without_tree_dimension() + evaluated.point.geometry.known_writable_head_count(),
        &|point| match point.geometry {
            Geometry::UnknownTreeFullArms => 17,
            Geometry::ImplementationTwoDevices => 15,
        },
    );
    check(
        "skip_blocks_freed_w_gens_0_1_3_8_is_0_64_192_512".to_string(),
        &|_point| true,
        &|evaluated| evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry).blocks_freed_while_referenced,
        &|point| match point.invisible_window_generations {
            0 => 0,
            1 => 64,
            3 => 192,
            _ => 512,
        },
    );
    check(
        "ignore_rdonly_violations_g_gens_1_4_5_10_is_1_4_5_10".to_string(),
        &|_point| true,
        &|evaluated| evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Ignore, AccountingPolicy::Carry).read_only_violations,
        &|point| match point.published_generations {
            1 => 1,
            4 => 4,
            5 => 5,
            _ => 10,
        },
    );
    for (head_count, statistic_count, extra) in [(2u64, 1u64, 2u64), (4, 2, 8)] {
        // 互比写成加法（`carry == no_carry + extra`），成立记 1、钉成 1。
        check(
            format!("w1_carry_equals_no_carry_plus_{extra}_at_h_new_{head_count}_r_tree_dim_{statistic_count}"),
            &move |point| point.unknown_writable_head_count == head_count && point.tree_dimension_statistic_count == statistic_count,
            &move |evaluated| {
                let carry = evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry).rows_written_this_generation;
                let no_carry = evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry).rows_written_this_generation;
                u64::from(carry == no_carry + extra)
            },
            &|_point| 1,
        );
    }
    check(
        "baseline_reject_entry_carry_w1_e113_h_new_2_r_tree_dim_1_is_19".to_string(),
        &|point| point.geometry == Geometry::UnknownTreeFullArms && point.unknown_writable_head_count == 2 && point.tree_dimension_statistic_count == 1,
        &|evaluated| evaluated.outcome_under_the_arms(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::Carry).rows_written_this_generation,
        &|_point| 19,
    );
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let closed_form_points: [(Geometry, u64, u64, bool, u64, RowsWrittenReading, [u64; 3]); 5] = [
        (Geometry::UnknownTreeFullArms, 2, 1, true, 1, RowsWrittenReading::AllRowsOfTheVersion, [23, 21, 21]),
        (Geometry::ImplementationTwoDevices, 4, 3, false, 2, RowsWrittenReading::AllRowsOfTheVersion, [37, 29, 37]),
        (Geometry::ImplementationTwoDevices, 4, 3, false, 2, RowsWrittenReading::ThisGenerationRowsOnly, [24, 16, 16]),
        (Geometry::ImplementationTwoDevices, 2, 0, false, 1, RowsWrittenReading::AllRowsOfTheVersion, [18, 16, 18]),
        (Geometry::ImplementationTwoDevices, 2, 0, false, 1, RowsWrittenReading::ThisGenerationRowsOnly, [17, 15, 15]),
    ];
    for (geometry, head_count, earlier_step_count, initial_publish_changed_the_watermark, statistic_count, rows_written, expected_by_accounting_arm) in closed_form_points {
        for (accounting_policy, expected) in ACCOUNTING_ARMS.into_iter().zip(expected_by_accounting_arm) {
            check(
                format!(
                    "closed_form_{}_{}_h_new_{head_count}_s_{earlier_step_count}_lpc_{}_r_tree_dim_{statistic_count}_{}_is_{expected}",
                    match rows_written {
                        RowsWrittenReading::AllRowsOfTheVersion => "w2",
                        RowsWrittenReading::ThisGenerationRowsOnly => "w1",
                    },
                    geometry.label(),
                    u8::from(initial_publish_changed_the_watermark),
                    accounting_label(accounting_policy)
                ),
                &move |point| {
                    point.geometry == geometry
                        && point.unknown_writable_head_count == head_count
                        && point.earlier_step_count == earlier_step_count
                        && point.initial_publish_changed_the_watermark == initial_publish_changed_the_watermark
                        && point.tree_dimension_statistic_count == statistic_count
                },
                &move |evaluated| {
                    let cell = evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, accounting_policy);
                    match rows_written {
                        RowsWrittenReading::AllRowsOfTheVersion => cell.rows_written_all_rows,
                        RowsWrittenReading::ThisGenerationRowsOnly => cell.rows_written_this_generation,
                    }
                },
                &move |_point| expected,
            );
        }
    }
    check(
        "clause_anchor_implementation_h_new_0_r_tree_dim_1_w1_is_15".to_string(),
        &|point| point.geometry == Geometry::ImplementationTwoDevices && point.unknown_writable_head_count == 0 && point.tree_dimension_statistic_count == 1,
        &|evaluated| evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry).rows_written_this_generation,
        &|_point| 15,
    );
    results
}
