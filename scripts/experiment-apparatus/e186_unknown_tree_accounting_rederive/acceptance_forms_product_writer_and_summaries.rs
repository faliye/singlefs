//! 接受的形态与后果、接受行与计数、产物的写出与汇总的累加。

use super::*;

/// 第二次跑第十节新表的形态（`form=`）：从被判点上的胜者集合列表机械推出。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum AcceptanceForm {
    UniqueCarry,
    UniqueOtherArm,
    TieWithCarry,
    TieWithoutCarry,
    DependsOnPointCarryEverywhere,
    DependsOnPointCarryNowhere,
    DependsOnPointMixed,
    NoJudgedPoints,
}

impl AcceptanceForm {
    pub(super) fn label(self) -> &'static str {
        match self {
            AcceptanceForm::UniqueCarry => "unique_carry",
            AcceptanceForm::UniqueOtherArm => "unique_other_arm",
            AcceptanceForm::TieWithCarry => "tie_with_carry",
            AcceptanceForm::TieWithoutCarry => "tie_without_carry",
            AcceptanceForm::DependsOnPointCarryEverywhere => "depends_on_point_carry_everywhere",
            AcceptanceForm::DependsOnPointCarryNowhere => "depends_on_point_carry_nowhere",
            AcceptanceForm::DependsOnPointMixed => "depends_on_point_mixed",
            AcceptanceForm::NoJudgedPoints => "no_judged_points",
        }
    }
}

/// 第十节新表里 ㊀ 的后果（`consequence=`）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Consequence {
    /// ㊀ 不翻面。
    CarryStands,
    /// ㊀ 不翻面，另几条与 `Carry` 在第 1–7 位都相等，作为「打平的另一条路」交用户知情。
    CarryStandsTieKnownToUser,
    /// ㊀ 翻面成这一条臂。
    FlipsTo(AccountingPolicy),
    /// ㊀ 翻面，换成哪一条交用户判。
    FlipChoiceToUser,
    /// 不判翻面也不判站住，交用户。
    LeftToUser,
    /// 没有被判的点，读不到，作废。
    NoJudgedPoints,
}

impl Consequence {
    pub(super) fn label(self) -> String {
        match self {
            Consequence::CarryStands => "carry_stands".to_string(),
            Consequence::CarryStandsTieKnownToUser => "carry_stands_tie_known_to_user".to_string(),
            Consequence::FlipsTo(accounting_policy) => format!("flips_to_{}", accounting_label(accounting_policy)),
            Consequence::FlipChoiceToUser => "flips_choice_to_user".to_string(),
            Consequence::LeftToUser => "left_to_user".to_string(),
            Consequence::NoJudgedPoints => "none".to_string(),
        }
    }
}

/// 一组取样点上 Q1 反向接受条款的判定（被判点数、不进判的 `h_new = 0` 点数、形态、后果、各胜者集合的点数）。
pub(super) struct AcceptanceTally {
    pub(super) judged_points: u64,
    pub(super) excluded_points_without_unknown_writable_heads: u64,
    pub(super) form: AcceptanceForm,
    pub(super) consequence: Consequence,
    pub(super) winner_sets: BTreeMap<Vec<AccountingPolicy>, u64>,
}

/// 被判点上至少有一个胜者集合时，按集合有几种、含不含 `Carry` 归形态。
fn form_of_the_winner_sets(sets: &BTreeMap<Vec<AccountingPolicy>, u64>) -> (AcceptanceForm, Consequence) {
    if sets.len() == 1 {
        let only = sets.keys().next().expect("恰有一种集合");
        if only.as_slice() == [AccountingPolicy::Carry] {
            (AcceptanceForm::UniqueCarry, Consequence::CarryStands)
        } else if only.len() == 1 {
            (AcceptanceForm::UniqueOtherArm, Consequence::FlipsTo(only[0]))
        } else if only.contains(&AccountingPolicy::Carry) {
            (AcceptanceForm::TieWithCarry, Consequence::CarryStandsTieKnownToUser)
        } else {
            (AcceptanceForm::TieWithoutCarry, Consequence::FlipChoiceToUser)
        }
    } else {
        let sets_with_carry = sets.keys().filter(|set| set.contains(&AccountingPolicy::Carry)).count();
        if sets_with_carry == sets.len() {
            (AcceptanceForm::DependsOnPointCarryEverywhere, Consequence::CarryStands)
        } else if sets_with_carry == 0 {
            (AcceptanceForm::DependsOnPointCarryNowhere, Consequence::FlipChoiceToUser)
        } else {
            (AcceptanceForm::DependsOnPointMixed, Consequence::LeftToUser)
        }
    }
}

/// 第二次跑第十节：只判 `h_new > 0` 的点（输入每个点一对：`h_new > 0` 与这个点十五格总胜者里的记账臂集合），`h_new = 0` 的点照数、不进判。
pub(super) fn carry_acceptance_of(points: &[(bool, Vec<AccountingPolicy>)]) -> AcceptanceTally {
    let judged: Vec<&Vec<AccountingPolicy>> = points.iter().filter(|(has_unknown_writable_heads, _winners)| *has_unknown_writable_heads).map(|(_has_unknown_writable_heads, winners)| winners).collect();
    let mut winner_sets: BTreeMap<Vec<AccountingPolicy>, u64> = BTreeMap::new();
    for winners in &judged {
        *winner_sets.entry((*winners).clone()).or_default() += 1;
    }
    let judged_points: u64 = judged.len().try_into().expect("计数装得进 u64");
    let total_points: u64 = points.len().try_into().expect("计数装得进 u64");
    let (form, consequence) = if judged.is_empty() { (AcceptanceForm::NoJudgedPoints, Consequence::NoJudgedPoints) } else { form_of_the_winner_sets(&winner_sets) };
    AcceptanceTally { judged_points, excluded_points_without_unknown_writable_heads: total_points - judged_points, form, consequence, winner_sets }
}

/// 一个点十五格总胜者里的记账臂（去重、按臂次序）。
fn overall_winner_accounting_arms(analysis: &PointAnalysis) -> Vec<AccountingPolicy> {
    analysis.overall_winners.iter().map(|(_flags_policy, accounting_policy)| *accounting_policy).collect::<BTreeSet<AccountingPolicy>>().into_iter().collect()
}

pub(super) fn render_acceptance_line(judgement: Judgement, group: &str, judged: bool, tally: &AcceptanceTally) -> String {
    let winner_sets =
        tally.winner_sets.iter().map(|(set, count)| format!("{}:{count}", join_labels(set, |policy| accounting_label(*policy).to_string()))).collect::<Vec<String>>().join("|");
    format!(
        "name=verdict check=q1_carry_acceptance_form judgement={} reading={} group={group} judged={} judged_points={} excluded_points_h_new_0={} form={} consequence={} winner_sets={winner_sets}",
        judgement.label(),
        JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED.label(),
        if judged { "yes" } else { "no" },
        tally.judged_points,
        tally.excluded_points_without_unknown_writable_heads,
        tally.form.label(),
        tally.consequence.label(),
    )
}

/// 打哪几组、判不判：判 ㊀ 只看 `legal_pooled`，两个 `r_tree_dim` 分组照报；E113 口径两组单报、标不判（登记第二次跑第一节读法第 11 条）。
pub(super) const ACCEPTANCE_GROUPS: [(&str, bool); 5] =
    [("legal_r_tree_dim_1", true), ("legal_r_tree_dim_2", true), ("legal_pooled", true), ("e113_style_r_tree_dim_1", false), ("e113_style_r_tree_dim_2", false)];

/// 每组的反向接受条款判定：每个点取 Q2「计」读法下十五格总胜者里的记账臂（第一次跑产物判 Q1 的就是这一类，读法 B1、W2）。
pub(super) fn acceptance_tallies(evaluated_points: &[EvaluatedPoint]) -> BTreeMap<&'static str, AcceptanceTally> {
    let mut points_by_group: BTreeMap<String, Vec<(bool, Vec<AccountingPolicy>)>> = BTreeMap::new();
    for evaluated in evaluated_points {
        let point = &evaluated.point;
        let acceptance_point = (point.unknown_writable_head_count > 0, overall_winner_accounting_arms(&analyse_point(evaluated, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)));
        points_by_group.entry(group_of(point)).or_default().push(acceptance_point.clone());
        if point.is_legal_under_todays_step_rows() {
            points_by_group.entry("legal_pooled".to_string()).or_default().push(acceptance_point);
        }
    }
    ACCEPTANCE_GROUPS.iter().map(|(group, _judged)| (*group, carry_acceptance_of(points_by_group.get(*group).map(Vec::as_slice).unwrap_or(&[])))).collect()
}

/// 登记第二次跑第七节乙类：每个 `r_tree_dim` 分组 (被判点, 不进判的点) 是 (3200, 1600)，合起来 (6400, 3200)。
pub(super) const EXPECTED_ACCEPTANCE_POINT_COUNTS: [(&str, u64, u64); 3] = [("legal_r_tree_dim_1", 3200, 1600), ("legal_r_tree_dim_2", 3200, 1600), ("legal_pooled", 6400, 3200)];

pub(super) fn all_readings() -> Vec<Reading> {
    [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED].into_iter().chain(INCIDENTAL_READINGS).collect()
}

/// 停机条款第 6 项的格一侧：两种判据下 U5 以外的条款轨迹相同、七位向量除第 6 位外相同、第 6 位之差等于新旧 U5 记数之差（六种读法都比）。
pub(super) fn cells_differ_only_in_the_carried_rows_clause(vacuous: &CellOutcome, dangling: &CellOutcome, point: &SamplePoint) -> bool {
    let other_clause_trajectories_agree = vacuous
        .clause_false_trajectories
        .iter()
        .zip(&dangling.clause_false_trajectories)
        .all(|(before, after)| before.0 == after.0 && (before.0 == ClauseUnit::DecisionEightItemTwelveCarriedRows || before.1 == after.1));
    let carried_rows_clause_count_of = |cell: &CellOutcome| u64::from(carried_rows_clause_false_trajectory_of(cell).is_some_and(|trajectory| trajectory.peak > 0));
    let (vacuous_carried_rows_clause_count, dangling_carried_rows_clause_count) = (carried_rows_clause_count_of(vacuous), carried_rows_clause_count_of(dangling));
    let vectors_agree = all_readings().into_iter().all(|reading| {
        let before = vector_under(vacuous, point, reading);
        let after = vector_under(dangling, point, reading);
        [0usize, 1, 2, 3, 4, 6].iter().all(|position| before[*position] == after[*position]) && after[5] + vacuous_carried_rows_clause_count == before[5] + dangling_carried_rows_clause_count
    });
    other_clause_trajectories_agree && vectors_agree
}

/// 停机条款第 6 项的 Q2 一侧：六种读法下，每条记账臂下 flags 臂的比较与十五格总胜者里的 flags 臂集合在两种判据下相同。
pub(super) fn flags_question_judgements_agree(vacuous: &EvaluatedPoint, dangling: &EvaluatedPoint) -> bool {
    all_readings().into_iter().all(|reading| {
        let before = analyse_point(vacuous, reading);
        let after = analyse_point(dangling, reading);
        before.flags_winners_by_accounting_arm == after.flags_winners_by_accounting_arm && flags_winners_signature(&before).1 == flags_winners_signature(&after).1
    })
}

/// 停机条款第 7 项：新 U5 照 N（R 里不认识的树的行数）判与照 `h_new > 0` 判，在这一点上一致。
pub(super) fn carried_rows_object_count_matches_the_unknown_writable_heads(point: &SamplePoint) -> bool {
    (unknown_tree_row_count_of(&initial_version(point)) > 0) == (point.unknown_writable_head_count > 0)
}

pub(super) struct ProductWriter<Output: std::io::Write> {
    pub(super) emitter: Emitter,
    pub(super) output: Output,
}

impl<Output: std::io::Write> ProductWriter<Output> {
    pub(super) fn line(&mut self, body: &str) {
        writeln!(self.output, "{}", self.emitter.emit_raw(body)).expect("产物写得进标准输出");
    }

    pub(super) fn finish(mut self) {
        let done_line = self.emitter.finish();
        writeln!(self.output, "{done_line}").expect("产物写得进标准输出");
        self.output.flush().expect("产物写得进标准输出");
    }
}

/// (种类, 读法, 分组, 臂, 取值) → 落在这一格的取样点。
pub(super) type SummaryKey = (&'static str, String, String, String, String);

pub(super) fn add_analysis_to_summaries(summaries: &mut BTreeMap<SummaryKey, PointSetDescription>, point: &SamplePoint, reading: Reading, analysis: &PointAnalysis) {
    let reading_label = reading.label();
    let group = group_of(point);
    let mut add = |kind: &'static str, arm: String, value: String| {
        summaries.entry((kind, reading_label.clone(), group.clone(), arm, value)).or_default().add(point);
    };
    for arm in &analysis.accounting_winners_by_flags_arm {
        let flags = flags_label(arm.flags_policy);
        add("q1_first_separating", flags.to_string(), render_position(arm.first_separating_position));
        add("q1_winners", flags.to_string(), join_labels(&arm.winners, |policy| accounting_label(*policy).to_string()));
        for ((left, right), position) in ACCOUNTING_PAIRS.iter().zip(&arm.pair_first_separating_positions) {
            add("q1_pair_first_separating", format!("{flags}:{}~{}", accounting_label(*left), accounting_label(*right)), render_position(*position));
        }
    }
    let overall_accounting_arms: Vec<AccountingPolicy> =
        analysis.overall_winners.iter().map(|(_flags_policy, accounting_policy)| *accounting_policy).collect::<BTreeSet<AccountingPolicy>>().into_iter().collect();
    add("q1_overall_winner_accounting_arms", "all".to_string(), join_labels(&overall_accounting_arms, |policy| accounting_label(*policy).to_string()));
    for arm in &analysis.flags_winners_by_accounting_arm {
        let accounting = accounting_label(arm.accounting_policy).to_string();
        add("q2_reject_vs_skip_first_separating", accounting.clone(), render_position(arm.reject_entry_versus_skip_first_separating_position));
        add("q2_position_six", accounting.clone(), render_ordering_of_reject_entry_against_skip(arm.position_six_reject_entry_versus_skip).to_string());
        add("q2_separable", accounting.clone(), u8::from(arm.separable_at_position_six).to_string());
        add("q2_flags_winners", accounting, join_labels(&arm.flags_winners, |policy| flags_label(*policy).to_string()));
    }
    let overall_flags_arms: Vec<UnknownFlagsPolicy> =
        analysis.overall_winners.iter().map(|(flags_policy, _accounting_policy)| *flags_policy).collect::<BTreeSet<UnknownFlagsPolicy>>().into_iter().collect();
    add("q2_overall_winner_flags_arms", "all".to_string(), join_labels(&overall_flags_arms, |policy| flags_label(*policy).to_string()));
}

/// 每个分组的 Q2 计数：几个点，几个点上有一条记账臂 `RejectEntry` 对 `Skip` 在第 6 位分得开（两种读法各数），几个点上 flags 胜者随读法换档。
#[derive(Default)]
pub(super) struct FlagsQuestionGroupTally {
    pub(super) points: u64,
    pub(super) separable_points_counted: u64,
    pub(super) separable_points_not_counted: u64,
    pub(super) changed_points_judged: u64,
    pub(super) changed_points_incidental_counting_the_judged_items: u64,
}

pub(super) fn render_verdict_holds(failures: u64) -> &'static str {
    if failures == 0 {
        "true"
    } else {
        "false"
    }
}
