//! 一格的结果与求值、各种读法、字典序向量与按读法取向量。

use super::*;

#[derive(Clone, Debug)]
pub(super) struct CellOutcome {
    pub(super) flags_policy: UnknownFlagsPolicy,
    pub(super) accounting_policy: AccountingPolicy,
    pub(super) writable_generations: u64,
    pub(super) blocks_freed_while_referenced: u64,
    pub(super) read_only_violations: u64,
    pub(super) revived_dead_trees: u64,
    /// 元组口径（第一节第 4 条）：R 里有行、R_G 里一行都没有的不认识的树的维度元组数。
    pub(super) rows_lost: u64,
    /// 行数口径的附带量 `per_generation_rows_dropped` 的总和：每一版里不认识的树每个元组比上一版少了几行，逐版相加。
    pub(super) rows_dropped_over_all_generations: u64,
    pub(super) silent_harm_count: u64,
    /// U1–U13 每一条在每一版上为假（1）还是真（0）的轨迹。
    pub(super) clause_false_trajectories: Vec<(ClauseUnit, Option<Trajectory>)>,
    pub(super) clause_units_excluding_the_judged_items: u64,
    pub(super) clause_units_counting_the_judged_items: u64,
    /// W2：R_g 记账树里的全部行数，R_1 … R_G 上取最大值；没有新版本记 0（一行都没写）。
    pub(super) rows_written_all_rows: u64,
    /// W1：代段等于 R_g 的 txg 的行数（E113 口径），同上取最大值。
    pub(super) rows_written_this_generation: u64,
    pub(super) trajectory_unknown_tuples_remaining: Option<Trajectory>,
    pub(super) trajectory_rows_lost: Option<Trajectory>,
    pub(super) trajectory_rows_written_this_generation: Option<Trajectory>,
    pub(super) trajectory_rows_written_all_rows: Option<Trajectory>,
    pub(super) trajectory_unknown_rows_off_this_txg: Option<Trajectory>,
    /// 不认识的树代段等于 R 的 txg 的行数（随根留着的阳性对照：每一版都有一行，为正版数 = G）。
    pub(super) trajectory_unknown_rows_at_the_initial_txg: Option<Trajectory>,
    pub(super) trajectory_rows_dropped_per_generation: Option<Trajectory>,
    pub(super) trajectory_row_generation_invariant_syntactic_violations: Option<Trajectory>,
    pub(super) trajectory_crates_step_reading_differences: Option<Trajectory>,
}

fn blocks_wrongly_freed(invisible_window_generations: u64) -> u64 {
    BLOCKS_BORN_PER_GENERATION * invisible_window_generations
}

pub(super) fn evaluate_cell(point: &SamplePoint, history: &WatermarkChangeHistory, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy, judgement: Judgement) -> CellOutcome {
    let published = published_history_of(point, history, flags_policy, accounting_policy);
    let effective_flags_policy = published.effective_flags_policy;
    let writable_generations = published.writable_generations;

    // 只有「树对写者隐形」才会认错 deadlist 的邻居；RejectEntry 下条目在、只是不可用 ⇒ 写者不做跨它的合并。
    let blocks_freed_while_referenced = if effective_flags_policy == UnknownFlagsPolicy::Skip && writable_generations > 0 {
        blocks_wrongly_freed(point.invisible_window_generations)
    } else {
        0
    };
    // 只有「自以为认识」才会把只读树当可写、把半删树当活的。
    let (read_only_violations, revived_dead_trees) = if effective_flags_policy == UnknownFlagsPolicy::Ignore {
        (UNKNOWN_READ_ONLY_TREE_COUNT * writable_generations, UNKNOWN_HALF_DELETED_TREE_COUNT)
    } else {
        (0, 0)
    };

    let unknown_tuples: Vec<(Statistic, TreeOwner)> = latest_value_of_each_unknown_tuple(&published.initial).into_keys().collect();
    let mut unknown_tuples_remaining_by_version = Vec::new();
    let mut rows_lost_by_version = Vec::new();
    let mut rows_written_this_generation_by_version = Vec::new();
    let mut rows_written_all_rows_by_version = Vec::new();
    let mut unknown_rows_off_this_txg_by_version = Vec::new();
    let mut unknown_rows_at_the_initial_txg_by_version = Vec::new();
    let mut rows_dropped_by_version = Vec::new();
    let mut row_generation_invariant_syntactic_violations_by_version = Vec::new();
    let mut crates_step_reading_differences_by_version = Vec::new();
    let mut clause_false_by_version: BTreeMap<ClauseUnit, Vec<u64>> = ALL_CLAUSE_UNITS.iter().map(|clause| (*clause, Vec::new())).collect();
    for (version_index, version) in published.versions.iter().enumerate() {
        let previous = if version_index == 0 { &published.initial } else { &published.versions[version_index - 1] };
        let remaining: u64 = unknown_tuples.iter().filter(|tuple| rows_of_tuple(version, **tuple) > 0).count().try_into().expect("计数装得进 u64");
        unknown_tuples_remaining_by_version.push(remaining);
        let unknown_tuple_count: u64 = unknown_tuples.len().try_into().expect("计数装得进 u64");
        rows_lost_by_version.push(unknown_tuple_count - remaining);
        rows_written_this_generation_by_version.push(version.rows.iter().filter(|row| row.generation_txg == version.txg).count().try_into().expect("计数装得进 u64"));
        rows_written_all_rows_by_version.push(version.rows.len().try_into().expect("计数装得进 u64"));
        unknown_rows_off_this_txg_by_version
            .push(version.rows.iter().filter(|row| row.belongs_to_an_unknown_tree() && row.generation_txg != version.txg).count().try_into().expect("计数装得进 u64"));
        unknown_rows_at_the_initial_txg_by_version
            .push(version.rows.iter().filter(|row| row.belongs_to_an_unknown_tree() && row.generation_txg == INITIAL_VERSION_TXG).count().try_into().expect("计数装得进 u64"));
        rows_dropped_by_version.push(unknown_tuples.iter().map(|tuple| rows_of_tuple(previous, *tuple).saturating_sub(rows_of_tuple(version, *tuple))).sum::<u64>());
        row_generation_invariant_syntactic_violations_by_version.push(row_generation_invariant_violations_under_the_checker_syntax(version, history));
        let crates_steps: BTreeSet<AccountingRow> = steps_by_the_crates_reading(version).into_iter().collect();
        let change_history_steps: BTreeSet<AccountingRow> = steps_by_the_watermark_change_history(version, history).into_iter().collect();
        crates_step_reading_differences_by_version.push(crates_steps.symmetric_difference(&change_history_steps).count().try_into().expect("计数装得进 u64"));
        for clause in ALL_CLAUSE_UNITS {
            let holds = clause_holds_on_version(clause, &published.initial, previous, version, point, history, flags_policy, judgement);
            clause_false_by_version.get_mut(&clause).expect("每一条都在表里").push(u64::from(!holds));
        }
    }

    let tuples_lost_by_the_final_version = rows_lost_by_version.last().copied().unwrap_or(0);
    let rows_dropped_over_all_generations: u64 = rows_dropped_by_version.iter().sum();
    let rows_lost = tuples_lost_by_the_final_version;
    let final_version = published.versions.last().unwrap_or(&published.initial);
    let lost_tuples_of = |statistic: Statistic| -> u64 {
        unknown_tuples.iter().filter(|tuple| tuple.0 == statistic && rows_of_tuple(final_version, **tuple) == 0).count().try_into().expect("计数装得进 u64")
    };
    let watermark_tuples_lost = lost_tuples_of(Statistic::InodeNumberWatermark);
    let second_statistic_tuples_lost = lost_tuples_of(Statistic::SecondTreeDimension);

    let silent_if_uncovered = |harm: u64, has_invariant: bool| -> u64 { u64::from(harm > 0 && !has_invariant) };
    let silent_harm_count = silent_if_uncovered(blocks_freed_while_referenced, WRONG_FREE_HAS_AN_INVARIANT)
        + silent_if_uncovered(read_only_violations, false)
        + silent_if_uncovered(revived_dead_trees, false)
        + silent_if_uncovered(watermark_tuples_lost, WATERMARK_TUPLE_LOSS_HAS_AN_INVARIANT)
        + silent_if_uncovered(second_statistic_tuples_lost, SECOND_TREE_STATISTIC_LOSS_HAS_AN_INVARIANT);

    let clause_false_trajectories: Vec<(ClauseUnit, Option<Trajectory>)> =
        ALL_CLAUSE_UNITS.iter().map(|clause| (*clause, trajectory_of(&clause_false_by_version[clause]))).collect();
    // U13 按臂判（flags 臂对未知位的处置是不是「该条目 EIO」），不看有没有新版本；别的条在任一版上为假记 1，没有新版本空真。
    let clause_counts = |clause: ClauseUnit, trajectory: Option<Trajectory>| -> bool {
        if clause == ClauseUnit::DecisionEightItemEightRejectUnknownBits {
            flags_policy != UnknownFlagsPolicy::RejectEntry
        } else {
            trajectory.is_some_and(|trajectory| trajectory.peak > 0)
        }
    };
    let clause_units_excluding_the_judged_items: u64 =
        clause_false_trajectories.iter().filter(|(clause, trajectory)| clause.counts_when_the_judged_items_are_excluded() && clause_counts(*clause, *trajectory)).count().try_into().expect("计数装得进 u64");
    let clause_units_counting_the_judged_items: u64 = clause_false_trajectories.iter().filter(|(clause, trajectory)| clause_counts(*clause, *trajectory)).count().try_into().expect("计数装得进 u64");

    CellOutcome {
        flags_policy,
        accounting_policy,
        writable_generations,
        blocks_freed_while_referenced,
        read_only_violations,
        revived_dead_trees,
        rows_lost,
        rows_dropped_over_all_generations,
        silent_harm_count,
        clause_false_trajectories,
        clause_units_excluding_the_judged_items,
        clause_units_counting_the_judged_items,
        rows_written_all_rows: rows_written_all_rows_by_version.iter().copied().max().unwrap_or(0),
        rows_written_this_generation: rows_written_this_generation_by_version.iter().copied().max().unwrap_or(0),
        trajectory_unknown_tuples_remaining: trajectory_of(&unknown_tuples_remaining_by_version),
        trajectory_rows_lost: trajectory_of(&rows_lost_by_version),
        trajectory_rows_written_this_generation: trajectory_of(&rows_written_this_generation_by_version),
        trajectory_rows_written_all_rows: trajectory_of(&rows_written_all_rows_by_version),
        trajectory_unknown_rows_off_this_txg: trajectory_of(&unknown_rows_off_this_txg_by_version),
        trajectory_unknown_rows_at_the_initial_txg: trajectory_of(&unknown_rows_at_the_initial_txg_by_version),
        trajectory_rows_dropped_per_generation: trajectory_of(&rows_dropped_by_version),
        trajectory_row_generation_invariant_syntactic_violations: trajectory_of(&row_generation_invariant_syntactic_violations_by_version),
        trajectory_crates_step_reading_differences: trajectory_of(&crates_step_reading_differences_by_version),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum RowsWrittenReading {
    /// 判的那一种：R_g 记账树里的全部行数。
    AllRowsOfTheVersion,
    /// 附带量：代段等于 R_g 的 txg 的行数（E113 口径）。
    ThisGenerationRowsOnly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum RejectEntrySemanticsReading {
    /// Q2「计」：`RejectEntry` 每一格多记一条要新定义的语义。
    Counted,
    /// Q2「不计」：维持 E113 原判。
    NotCounted,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Reading {
    clause: ClauseReading,
    pub(super) rows_written: RowsWrittenReading,
    reject_entry_semantics: RejectEntrySemanticsReading,
}

impl Reading {
    pub(super) fn label(self) -> String {
        let clause = match self.clause {
            ClauseReading::ExcludesTheJudgedItems => "b1",
            ClauseReading::CountsTheJudgedItems => "b2",
        };
        let rows_written = match self.rows_written {
            RowsWrittenReading::AllRowsOfTheVersion => "w2",
            RowsWrittenReading::ThisGenerationRowsOnly => "w1",
        };
        let reject_entry_semantics = match self.reject_entry_semantics {
            RejectEntrySemanticsReading::Counted => "q2_counted",
            RejectEntrySemanticsReading::NotCounted => "q2_not_counted",
        };
        format!("{clause}_{rows_written}_{reject_entry_semantics}")
    }

    pub(super) fn with_reject_entry_semantics(self, reject_entry_semantics: RejectEntrySemanticsReading) -> Reading {
        Reading { clause: self.clause, rows_written: self.rows_written, reject_entry_semantics }
    }
}

pub(super) const JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED: Reading =
    Reading { clause: ClauseReading::ExcludesTheJudgedItems, rows_written: RowsWrittenReading::AllRowsOfTheVersion, reject_entry_semantics: RejectEntrySemanticsReading::Counted };
pub(super) const JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED: Reading =
    Reading { clause: ClauseReading::ExcludesTheJudgedItems, rows_written: RowsWrittenReading::AllRowsOfTheVersion, reject_entry_semantics: RejectEntrySemanticsReading::NotCounted };
pub(super) const INCIDENTAL_READINGS: [Reading; 4] = [
    Reading { clause: ClauseReading::CountsTheJudgedItems, rows_written: RowsWrittenReading::AllRowsOfTheVersion, reject_entry_semantics: RejectEntrySemanticsReading::Counted },
    Reading { clause: ClauseReading::CountsTheJudgedItems, rows_written: RowsWrittenReading::AllRowsOfTheVersion, reject_entry_semantics: RejectEntrySemanticsReading::NotCounted },
    Reading { clause: ClauseReading::ExcludesTheJudgedItems, rows_written: RowsWrittenReading::ThisGenerationRowsOnly, reject_entry_semantics: RejectEntrySemanticsReading::Counted },
    Reading { clause: ClauseReading::ExcludesTheJudgedItems, rows_written: RowsWrittenReading::ThisGenerationRowsOnly, reject_entry_semantics: RejectEntrySemanticsReading::NotCounted },
];

pub(super) type LexicographicVector = [u64; 7];

fn reject_entry_semantics_extra_clause_units(flags_policy: UnknownFlagsPolicy, reject_entry_semantics: RejectEntrySemanticsReading) -> u64 {
    if reject_entry_semantics == RejectEntrySemanticsReading::Counted && flags_policy == UnknownFlagsPolicy::RejectEntry {
        REJECT_ENTRY_NEW_SEMANTICS_CLAUSE_UNITS
    } else {
        0
    }
}

/// 跑前写死的字典序，七位逐位比、越小越好。
fn lexicographic_vector(outcome: &CellOutcome, published_generations: u64, clause_units_to_change: u64, rows_written_per_generation: u64) -> LexicographicVector {
    [
        outcome.blocks_freed_while_referenced,
        outcome.read_only_violations + outcome.revived_dead_trees,
        outcome.rows_lost,
        outcome.silent_harm_count,
        published_generations - outcome.writable_generations,
        clause_units_to_change, rows_written_per_generation,
    ]
}

pub(super) fn vector_under(outcome: &CellOutcome, point: &SamplePoint, reading: Reading) -> LexicographicVector {
    let clause_units_of_the_list = match reading.clause {
        ClauseReading::ExcludesTheJudgedItems => outcome.clause_units_excluding_the_judged_items,
        ClauseReading::CountsTheJudgedItems => outcome.clause_units_counting_the_judged_items,
    };
    let clause_units_to_change = clause_units_of_the_list + reject_entry_semantics_extra_clause_units(outcome.flags_policy, reading.reject_entry_semantics);
    let rows_written_per_generation = match reading.rows_written {
        RowsWrittenReading::AllRowsOfTheVersion => outcome.rows_written_all_rows,
        RowsWrittenReading::ThisGenerationRowsOnly => outcome.rows_written_this_generation,
    };
    lexicographic_vector(outcome, point.published_generations, clause_units_to_change, rows_written_per_generation)
}
