//! 一个取样点的求值与分析、获胜集、各行的渲染。

use super::*;

pub(super) struct EvaluatedPoint {
    pub(super) point: SamplePoint,
    /// 这些格是按哪一种 U5 判据算的（手写账本要按同一种对）。
    judgement: Judgement,
    /// 进字典序比较的格，次序是 flags 臂在外、记账臂在内。
    pub(super) cells: Vec<CellOutcome>,
}

impl EvaluatedPoint {
    pub(super) fn outcome_under_the_arms(&self, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy) -> &CellOutcome {
        self.cells
            .iter()
            .find(|cell| cell.flags_policy == flags_policy && cell.accounting_policy == accounting_policy)
            .expect("十五格每一格都进比较（登记第五节：不许有挂得上才比的筛子）")
    }

    pub(super) fn vector(&self, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy, reading: Reading) -> LexicographicVector {
        vector_under(self.outcome_under_the_arms(flags_policy, accounting_policy), &self.point, reading)
    }
}

pub(super) fn evaluate_point(point: SamplePoint, judgement: Judgement) -> EvaluatedPoint {
    let history = watermark_change_history_of(&point);
    let mut cells = Vec::new();
    for flags_policy in FLAGS_ARMS {
        for accounting_policy in ACCOUNTING_ARMS {
            let outcome = evaluate_cell(&point, &history, flags_policy, accounting_policy, judgement);
            cells.push(outcome);
        }
    }
    EvaluatedPoint { point, judgement, cells }
}

pub(super) fn flags_label(flags_policy: UnknownFlagsPolicy) -> &'static str {
    match flags_policy {
        UnknownFlagsPolicy::Ignore => "Ignore",
        UnknownFlagsPolicy::Skip => "Skip",
        UnknownFlagsPolicy::RejectEntry => "RejectEntry",
        UnknownFlagsPolicy::ReadOnly => "ReadOnly",
        UnknownFlagsPolicy::PerBit => "PerBit",
    }
}

pub(super) fn accounting_label(accounting_policy: AccountingPolicy) -> &'static str {
    match accounting_policy {
        AccountingPolicy::Carry => "Carry",
        AccountingPolicy::NoCarry => "NoCarry",
        AccountingPolicy::KeepWithRoot => "KeepWithRoot",
    }
}

/// 第一个不全相等的位号（从 1 数）；七位全等交回 `None`。
pub(super) fn first_separating_position(vectors: &[LexicographicVector]) -> Option<usize> {
    let reference = vectors.first()?;
    (0..reference.len()).find(|position| vectors.iter().any(|vector| vector[*position] != reference[*position])).map(|position| position + 1)
}

pub(super) fn render_position(position: Option<usize>) -> String {
    match position {
        Some(position) => position.to_string(),
        None => "none".to_string(),
    }
}

/// 七位最小的那几个的下标（并列全报）。
pub(super) fn winner_indices(vectors: &[LexicographicVector]) -> Vec<usize> {
    let Some(best) = vectors.iter().min() else {
        return Vec::new();
    };
    (0..vectors.len()).filter(|index| vectors[*index] == *best).collect()
}

pub(super) const ACCOUNTING_PAIRS: [(AccountingPolicy, AccountingPolicy); 3] = [
    (AccountingPolicy::Carry, AccountingPolicy::NoCarry),
    (AccountingPolicy::Carry, AccountingPolicy::KeepWithRoot),
    (AccountingPolicy::NoCarry, AccountingPolicy::KeepWithRoot),
];

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct AccountingWinnersUnderFlagsArm {
    pub(super) flags_policy: UnknownFlagsPolicy,
    pub(super) first_separating_position: Option<usize>,
    pub(super) pair_first_separating_positions: Vec<Option<usize>>,
    pub(super) winners: Vec<AccountingPolicy>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct FlagsWinnersUnderAccountingArm {
    pub(super) accounting_policy: AccountingPolicy,
    pub(super) reject_entry_versus_skip_first_separating_position: Option<usize>,
    /// `RejectEntry` 那一格第 6 位对 `Skip` 那一格第 6 位。
    pub(super) position_six_reject_entry_versus_skip: std::cmp::Ordering,
    /// 第 1–5 位全平而第 6 位分开。
    pub(super) separable_at_position_six: bool,
    pub(super) flags_winners: Vec<UnknownFlagsPolicy>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct PointAnalysis {
    pub(super) accounting_winners_by_flags_arm: Vec<AccountingWinnersUnderFlagsArm>,
    pub(super) overall_winners: Vec<(UnknownFlagsPolicy, AccountingPolicy)>,
    pub(super) flags_winners_by_accounting_arm: Vec<FlagsWinnersUnderAccountingArm>,
    pub(super) all_fifteen_cells_equal: bool,
}

pub(super) fn analyse_point(evaluated: &EvaluatedPoint, reading: Reading) -> PointAnalysis {
    let accounting_winners_by_flags_arm = FLAGS_ARMS
        .iter()
        .map(|flags_policy| {
            let vectors: Vec<LexicographicVector> = ACCOUNTING_ARMS.iter().map(|accounting_policy| evaluated.vector(*flags_policy, *accounting_policy, reading)).collect();
            let pair_first_separating_positions = ACCOUNTING_PAIRS
                .iter()
                .map(|(left, right)| first_separating_position(&[evaluated.vector(*flags_policy, *left, reading), evaluated.vector(*flags_policy, *right, reading)]))
                .collect();
            AccountingWinnersUnderFlagsArm {
                flags_policy: *flags_policy,
                first_separating_position: first_separating_position(&vectors),
                pair_first_separating_positions,
                winners: winner_indices(&vectors).into_iter().map(|index| ACCOUNTING_ARMS[index]).collect(),
            }
        })
        .collect();
    let all_cells: Vec<(UnknownFlagsPolicy, AccountingPolicy)> =
        FLAGS_ARMS.iter().flat_map(|flags_policy| ACCOUNTING_ARMS.iter().map(move |accounting_policy| (*flags_policy, *accounting_policy))).collect();
    let all_vectors: Vec<LexicographicVector> = all_cells.iter().map(|(flags_policy, accounting_policy)| evaluated.vector(*flags_policy, *accounting_policy, reading)).collect();
    let overall_winners = winner_indices(&all_vectors).into_iter().map(|index| all_cells[index]).collect();
    let flags_winners_by_accounting_arm = ACCOUNTING_ARMS
        .iter()
        .map(|accounting_policy| {
            let reject_entry = evaluated.vector(UnknownFlagsPolicy::RejectEntry, *accounting_policy, reading);
            let skip = evaluated.vector(UnknownFlagsPolicy::Skip, *accounting_policy, reading);
            let flags_vectors: Vec<LexicographicVector> = FLAGS_ARMS.iter().map(|flags_policy| evaluated.vector(*flags_policy, *accounting_policy, reading)).collect();
            FlagsWinnersUnderAccountingArm {
                accounting_policy: *accounting_policy,
                reject_entry_versus_skip_first_separating_position: first_separating_position(&[reject_entry, skip]),
                position_six_reject_entry_versus_skip: reject_entry[5].cmp(&skip[5]),
                separable_at_position_six: reject_entry[..5] == skip[..5] && reject_entry[5] != skip[5],
                flags_winners: winner_indices(&flags_vectors).into_iter().map(|index| FLAGS_ARMS[index]).collect(),
            }
        })
        .collect();
    PointAnalysis { accounting_winners_by_flags_arm, overall_winners, flags_winners_by_accounting_arm, all_fifteen_cells_equal: first_separating_position(&all_vectors).is_none() }
}

fn cell_label(flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy) -> String {
    format!("{}_{}", flags_label(flags_policy), accounting_label(accounting_policy))
}

fn render_vector(vector: &LexicographicVector) -> String {
    vector.iter().map(u64::to_string).collect::<Vec<String>>().join(",")
}

pub(super) fn join_labels<Item>(items: &[Item], label_of: impl Fn(&Item) -> String) -> String {
    if items.is_empty() {
        return "none".to_string();
    }
    items.iter().map(label_of).collect::<Vec<String>>().join("+")
}

/// 为假过的条（U13 按臂判，记 `U13:arm`），其余写 `U<n>:峰值/为正版数/期末值`；一条都没有写 `none`。没列出的条三数都是 0。
fn render_clauses_false(outcome: &CellOutcome) -> String {
    let mut parts = Vec::new();
    for (clause, trajectory) in &outcome.clause_false_trajectories {
        if *clause == ClauseUnit::DecisionEightItemEightRejectUnknownBits {
            if outcome.flags_policy != UnknownFlagsPolicy::RejectEntry {
                parts.push(format!("{}:arm", clause.registered_label()));
            }
        } else if let Some(trajectory) = trajectory {
            if trajectory.peak > 0 {
                parts.push(format!("{}:{}", clause.registered_label(), render_trajectory(Some(*trajectory))));
            }
        }
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join("+")
    }
}

pub(super) fn render_cell_line(point_index: usize, evaluated: &EvaluatedPoint, outcome: &CellOutcome) -> String {
    let point = &evaluated.point;
    format!(
        "name=cell point={point_index} arm={} writable_gens={} rdonly_violations={} dead_revived={} vector_q2_counted={} vector_q2_not_counted={} \
         clause_units_b1={} ledger_b1={} clause_units_b2={} ledger_b2={} rows_written_w2={} rows_written_w1={} clauses_false={} \
         unknown_tuples_remaining={} rows_lost_trajectory={} w1_trajectory={} w2_trajectory={} unknown_rows_off_this_txg={} unknown_rows_at_initial_txg={} \
         per_generation_rows_dropped={} per_generation_rows_dropped_total={} i313_syntactic_violations={} crates_step_reading_differences={}",
        cell_label(outcome.flags_policy, outcome.accounting_policy),
        outcome.writable_generations,
        outcome.read_only_violations,
        outcome.revived_dead_trees,
        render_vector(&vector_under(outcome, point, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)),
        render_vector(&vector_under(outcome, point, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED)),
        outcome.clause_units_excluding_the_judged_items,
        clause_units_from_the_hand_ledger(point, outcome.flags_policy, outcome.accounting_policy, ClauseReading::ExcludesTheJudgedItems, evaluated.judgement),
        outcome.clause_units_counting_the_judged_items,
        clause_units_from_the_hand_ledger(point, outcome.flags_policy, outcome.accounting_policy, ClauseReading::CountsTheJudgedItems, evaluated.judgement),
        outcome.rows_written_all_rows,
        outcome.rows_written_this_generation,
        render_clauses_false(outcome),
        render_trajectory(outcome.trajectory_unknown_tuples_remaining),
        render_trajectory(outcome.trajectory_rows_lost),
        render_trajectory(outcome.trajectory_rows_written_this_generation),
        render_trajectory(outcome.trajectory_rows_written_all_rows),
        render_trajectory(outcome.trajectory_unknown_rows_off_this_txg),
        render_trajectory(outcome.trajectory_unknown_rows_at_the_initial_txg),
        render_trajectory(outcome.trajectory_rows_dropped_per_generation),
        outcome.rows_dropped_over_all_generations,
        render_trajectory(outcome.trajectory_row_generation_invariant_syntactic_violations),
        render_trajectory(outcome.trajectory_crates_step_reading_differences),
    )
}

pub(super) fn render_ordering_of_reject_entry_against_skip(ordering: std::cmp::Ordering) -> &'static str {
    match ordering {
        std::cmp::Ordering::Less => "reject_entry_smaller",
        std::cmp::Ordering::Equal => "equal",
        std::cmp::Ordering::Greater => "skip_smaller",
    }
}

pub(super) fn render_point_verdict_line(point_index: usize, evaluated: &EvaluatedPoint, counted: &PointAnalysis, not_counted: &PointAnalysis) -> String {
    let accounting_winners = &counted.accounting_winners_by_flags_arm;
    let accounting_winners_first_separating = accounting_winners.iter().map(|arm| format!("{}:{}", flags_label(arm.flags_policy), render_position(arm.first_separating_position))).collect::<Vec<_>>().join(",");
    let accounting_winner_pairs = accounting_winners
        .iter()
        .map(|arm| {
            let pairs = ACCOUNTING_PAIRS
                .iter()
                .zip(&arm.pair_first_separating_positions)
                .map(|((left, right), position)| format!("{}~{}={}", accounting_label(*left), accounting_label(*right), render_position(*position)))
                .collect::<Vec<_>>()
                .join("/");
            format!("{}:{pairs}", flags_label(arm.flags_policy))
        })
        .collect::<Vec<_>>()
        .join(",");
    let accounting_winner_sets =
        accounting_winners.iter().map(|arm| format!("{}:{}", flags_label(arm.flags_policy), join_labels(&arm.winners, |policy| accounting_label(*policy).to_string()))).collect::<Vec<_>>().join(",");
    let overall = |analysis: &PointAnalysis| join_labels(&analysis.overall_winners, |(flags_policy, accounting_policy)| cell_label(*flags_policy, *accounting_policy));
    let flags_winners_field = |analysis: &PointAnalysis, render: &dyn Fn(&FlagsWinnersUnderAccountingArm) -> String| {
        analysis.flags_winners_by_accounting_arm.iter().map(|arm| format!("{}:{}", accounting_label(arm.accounting_policy), render(arm))).collect::<Vec<_>>().join(",")
    };
    let first = |arm: &FlagsWinnersUnderAccountingArm| render_position(arm.reject_entry_versus_skip_first_separating_position);
    let six = |arm: &FlagsWinnersUnderAccountingArm| render_ordering_of_reject_entry_against_skip(arm.position_six_reject_entry_versus_skip).to_string();
    let separable = |arm: &FlagsWinnersUnderAccountingArm| u8::from(arm.separable_at_position_six).to_string();
    let flags_winners = |arm: &FlagsWinnersUnderAccountingArm| join_labels(&arm.flags_winners, |policy| flags_label(*policy).to_string());
    format!(
        "name=point_verdict point={point_index} {} legal={} q1_first_separating={accounting_winners_first_separating} q1_pair_first_separating={accounting_winner_pairs} q1_winners={accounting_winner_sets} \
         overall_winners_q2_counted={} overall_winners_q2_not_counted={} \
         q2_reject_vs_skip_first_separating_counted={} q2_reject_vs_skip_first_separating_not_counted={} \
         q2_position_six_counted={} q2_position_six_not_counted={} q2_separable_counted={} q2_separable_not_counted={} \
         q2_flags_winners_counted={} q2_flags_winners_not_counted={} q2_flags_winners_changed={} all_fifteen_cells_equal_counted={} all_fifteen_cells_equal_not_counted={}",
        evaluated.point.describe(),
        u8::from(evaluated.point.is_legal_under_todays_step_rows()),
        overall(counted),
        overall(not_counted),
        flags_winners_field(counted, &first),
        flags_winners_field(not_counted, &first),
        flags_winners_field(counted, &six),
        flags_winners_field(not_counted, &six),
        flags_winners_field(counted, &separable),
        flags_winners_field(not_counted, &separable),
        flags_winners_field(counted, &flags_winners),
        flags_winners_field(not_counted, &flags_winners),
        u8::from(flags_winners_signature(counted) != flags_winners_signature(not_counted)),
        u8::from(counted.all_fifteen_cells_equal),
        u8::from(not_counted.all_fifteen_cells_equal),
    )
}

/// Q2 后半「胜者是否换档」比的东西：每条记账臂下 flags 五格的胜者，加十五格总胜者里的 flags 臂。
pub(super) fn flags_winners_signature(analysis: &PointAnalysis) -> (Vec<Vec<UnknownFlagsPolicy>>, BTreeSet<UnknownFlagsPolicy>) {
    (
        analysis.flags_winners_by_accounting_arm.iter().map(|arm| arm.flags_winners.clone()).collect(),
        analysis.overall_winners.iter().map(|(flags_policy, _accounting_policy)| *flags_policy).collect(),
    )
}

pub(super) fn accounting_winners_signature(analysis: &PointAnalysis) -> Vec<Vec<AccountingPolicy>> {
    analysis.accounting_winners_by_flags_arm.iter().map(|arm| arm.winners.clone()).collect()
}

/// 一组取样点的描述：几个点，每一维取到了哪些值（报「依赖取样点」时列翻转点用）。
#[derive(Default)]
pub(super) struct PointSetDescription {
    pub(super) point_count: u64,
    values_by_coordinate: BTreeMap<&'static str, BTreeSet<String>>,
}

impl PointSetDescription {
    pub(super) fn add(&mut self, point: &SamplePoint) {
        self.point_count += 1;
        let coordinates: [(&'static str, String); 9] = [
            ("geometry", point.geometry.label().to_string()),
            ("k_keep", point.retired_kept_generation_count.to_string()),
            ("w_gens", point.invisible_window_generations.to_string()),
            ("h_new", point.unknown_writable_head_count.to_string()),
            ("registered", point.trees_with_registered_bit.to_string()),
            ("g_gens", point.published_generations.to_string()),
            ("s", point.earlier_step_count.to_string()),
            ("lpc", u8::from(point.initial_publish_changed_the_watermark).to_string()),
            ("r_tree_dim", point.tree_dimension_statistic_count.to_string()),
        ];
        for (coordinate, value) in coordinates {
            self.values_by_coordinate.entry(coordinate).or_default().insert(value);
        }
    }

    /// 每个字段名前加 `prefix`（几何敏感性那一行写 `flipped_`，免得与这一行自己的 `pairs=` 混淆）。
    pub(super) fn render(&self, prefix: &str) -> String {
        let mut text = format!("{prefix}points={}", self.point_count);
        for coordinate in ["geometry", "k_keep", "w_gens", "h_new", "registered", "g_gens", "s", "lpc", "r_tree_dim"] {
            let values = self.values_by_coordinate.get(coordinate).map(|values| values.iter().cloned().collect::<Vec<String>>().join("+")).unwrap_or_else(|| "none".to_string());
            text.push_str(&format!(" {prefix}{coordinate}={values}"));
        }
        text
    }
}

/// 判决分组：合法点与 E113 口径点（(s, lpc) = (0, 0)）分开，`r_tree_dim` 两档分开（登记第一节第 9 条）。
pub(super) fn group_of(point: &SamplePoint) -> String {
    let legality = if point.is_legal_under_todays_step_rows() { "legal" } else { "e113_style" };
    format!("{legality}_r_tree_dim_{}", point.tree_dimension_statistic_count)
}

/// 记账臂起作用的 flags 格（第五节阳性对照表「flags 臂同上」那一组）：`Skip`、`RejectEntry`，`PerBit` 只在全部登记时。
pub(super) fn accounting_arm_acts_under(flags_policy: UnknownFlagsPolicy, point: &SamplePoint) -> bool {
    matches!(effective_flags_policy_of(flags_policy, point), UnknownFlagsPolicy::Skip | UnknownFlagsPolicy::RejectEntry)
}
