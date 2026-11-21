//! E186：不认识的树·记账维重推 —— 旧写者遇到不认识的树时，记账那一维（`Carry` / `NoCarry` / 随根留着）与 flags 那一维
//! （五条臂）在今天的记账条款下怎么排；E113（不认识的树·补齐臂集） 的字典序在「每代写行」之前补一位「要改几条已定条款」。
//!
//! 跑前登记：`research/prompts/e186-preregistration.md`。读法写死：读法 B 取 B1（D8 已定项 8 的 ㊀ ㊁ 不进条款清单），
//! 读法 W 取 W2（每代写行 = 这一版记账树里的全部行）；B2、W1 只作附带量照报、不判。
//! 独立手写模型，不与 `crates/` 共用代码；与 `crates/` 逐项对拍只做登记第十一节停机条款的三项（行数、台阶行认法、I-3.13 的语法认法）。
//!
//! ## 谓词直接引的条款句子（整段原文在跑前登记第二节）
//!
//! - D5（快照 / 空间记账机制） 已定项 2：「每条根的记账树只装这条根那一代的行，不留多代」；「例外是 inode 号水位那一统计量的台阶行：随根一直留着」。
//! - D5（快照 / 空间记账机制） 已定项 2 射程：「记账在 inode 号水位变了的每次发布给这一统计量留一行台阶行 (g, H)」「克隆头的台阶行按 (统计量, 树) 分」
//!   ⇒ 台阶行按 (统计量, 树) 认、看的是那次发布那个头的水位变没变（`WatermarkChangeHistory`），不按代段语法认。
//! - D5（快照 / 空间记账机制） 已定项 4 射程：「读一个统计量的现行值 = 取该维度元组下代号最大的那一行，空表示从未维护过、不是 0」
//!   ⇒ 丢行按维度元组数（元组里还剩任一行，现行值读得出）。
//! - I-3.13（记账行代段等于这一版的 txg）：「每一行的代段……等于这一版的 `checkpoint_txg`……inode 号水位那一统计量的台阶行不判」。
//! - I-9.11（可写克隆头有水位行）：「每个可写克隆头的树 ID 在记账里都有「inode 号水位」行」。
//!
//! ## 选法（跑前写死，字典序，越小越好）
//!
//! 1 误释放 → 2 违反语义 → 3 丢行（元组） → 4 静默 → 5 不可写代数 → 6 要改几条已定条款 → 7 每代写行。
//! 不设「挂得上才比」的筛子：十五格每一格都进比较。
//!
//! ## 第二次跑（重跑登记 `research/prompts/e186-preregistration.md` 的「第二次跑（重跑登记）」一节）
//!
//! 同一个 bin 加参数 `--judgement <vacuous-u5|dangling-u5>`：不带参数等于 `vacuous-u5`，逐行照第一次跑的判据算，
//! 产物逐字节等于第一次跑的产物；`dangling-u5` 是第二次跑的判据（新 U5 谓词与新的反向接受条款）。
//! 与第一次跑比只改这两处，其余照旧。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

mod acceptance_forms_product_writer_and_summaries;
mod accounting_rows_and_versions;
mod cell_evaluation_and_readings;
mod geometries_policies_and_sample_points;
mod judgement_and_registered_constants;
mod point_analysis_and_rendering;
mod self_proofs_and_positive_controls;
mod sensitivity_pairs_and_anchors;
mod step_recognition_and_clause_units;

use self::acceptance_forms_product_writer_and_summaries::*;
use self::accounting_rows_and_versions::*;
use self::cell_evaluation_and_readings::*;
use self::geometries_policies_and_sample_points::*;
use self::judgement_and_registered_constants::*;
use self::point_analysis_and_rendering::*;
use self::self_proofs_and_positive_controls::*;
use self::sensitivity_pairs_and_anchors::*;
use self::step_recognition_and_clause_units::*;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let judgement = match Judgement::from_arguments(&arguments) {
        Ok(judgement) => judgement,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let standard_output = std::io::stdout();
    let mut writer = ProductWriter { emitter: Emitter::new(), output: std::io::BufWriter::new(standard_output.lock()) };
    let evaluated_points: Vec<EvaluatedPoint> = all_sample_points().into_iter().map(|point| evaluate_point(point, judgement)).collect();
    let point_count: u64 = evaluated_points.len().try_into().expect("计数装得进 u64");
    // 不带参数（第一次跑的判据）时配置行一个字不加，产物逐字节等于第一次跑的产物；第二次跑的判据另报这两个字段。
    let judgement_fields = match judgement {
        Judgement::VacuousCarriedRowsClause => String::new(),
        Judgement::DanglingCarriedRowsClause => format!(
            " judgement={} u5_predicate=n_zero_true_n_positive_nothing_written_false_else_every_written_row_at_this_txg acceptance_judged_points=h_new_positive_legal_only",
            judgement.label()
        ),
    };
    writer.line(&format!(
        "name=config experiment=e186_unknown_tree_accounting_rederive model=arithmetic threads=1 points={point_count} cells_per_point={CELLS_PER_POINT} \
         initial_txg={INITIAL_VERSION_TXG} t_known={KNOWN_TREE_COUNT} t_new={UNKNOWN_TREE_COUNT} t_new_rdonly={UNKNOWN_READ_ONLY_TREE_COUNT} t_new_dead={UNKNOWN_HALF_DELETED_TREE_COUNT} \
         blocks_per_gen={BLOCKS_BORN_PER_GENERATION} e113_rows_without_tree_dimension={E113_ROWS_WITHOUT_TREE_DIMENSION} e113_h_known={E113_KNOWN_WRITABLE_HEAD_COUNT} \
         e113_known_head_steps={E113_KNOWN_HEAD_STEP_ROWS} crates_pool_wide_rows={CRATES_POOL_WIDE_ROWS} crates_per_device_rows={CRATES_PER_DEVICE_ROWS} \
         crates_devices={CRATES_DEVICE_COUNT} crates_h_known={CRATES_KNOWN_WRITABLE_HEAD_COUNT} crates_known_head_steps={CRATES_KNOWN_HEAD_STEP_ROWS} \
         judged_readings={}+{} incidental_readings={} \
         vector_positions=blocks_freed_while_referenced,rdonly_violations+dead_revived,rows_lost_tuples,silent,g_gens_minus_writable_gens,clause_units_to_change,rows_written_per_gen \
         trajectory_fields=peak/positive_versions/final_over_r1_to_rg clauses_false=listed_when_peak_is_1_others_are_0/0/0_u13_judged_per_arm \
         step_recognition=watermark_change_history_by_statistic_and_tree{judgement_fields}",
        JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED.label(),
        JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED.label(),
        INCIDENTAL_READINGS.iter().map(|reading| reading.label()).collect::<Vec<String>>().join("+"),
    ));

    let mut summaries: BTreeMap<SummaryKey, PointSetDescription> = BTreeMap::new();
    let mut flags_question_tallies: BTreeMap<String, FlagsQuestionGroupTally> = BTreeMap::new();
    let mut ranked_cell_count = 0u64;
    let mut ledger_mismatches_excluding_the_judged_items = 0u64;
    let mut ledger_mismatches_counting_the_judged_items = 0u64;
    let mut points_where_the_accounting_winners_depend_on_the_reject_entry_reading = 0u64;
    let mut non_distinguishing_points_counted = 0u64;
    let mut non_distinguishing_points_not_counted = 0u64;
    let mut analyses: Vec<(PointAnalysis, PointAnalysis)> = Vec::with_capacity(evaluated_points.len());    for (point_index, evaluated) in evaluated_points.iter().enumerate() {
        let point = &evaluated.point;
        let cell_count: u64 = evaluated.cells.len().try_into().expect("计数装得进 u64");
        ranked_cell_count += cell_count;
        for cell in &evaluated.cells {
            writer.line(&render_cell_line(point_index, evaluated, cell));
            ledger_mismatches_excluding_the_judged_items += u64::from(cell.clause_units_excluding_the_judged_items != clause_units_from_the_hand_ledger(point, cell.flags_policy, cell.accounting_policy, ClauseReading::ExcludesTheJudgedItems, judgement));
            ledger_mismatches_counting_the_judged_items += u64::from(cell.clause_units_counting_the_judged_items != clause_units_from_the_hand_ledger(point, cell.flags_policy, cell.accounting_policy, ClauseReading::CountsTheJudgedItems, judgement));
        }
        let counted = analyse_point(evaluated, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
        let not_counted = analyse_point(evaluated, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
        points_where_the_accounting_winners_depend_on_the_reject_entry_reading += u64::from(counted.accounting_winners_by_flags_arm != not_counted.accounting_winners_by_flags_arm);
        non_distinguishing_points_counted += u64::from(counted.all_fifteen_cells_equal);
        non_distinguishing_points_not_counted += u64::from(not_counted.all_fifteen_cells_equal);
        writer.line(&render_point_verdict_line(point_index, evaluated, &counted, &not_counted));
        add_analysis_to_summaries(&mut summaries, point, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, &counted);
        add_analysis_to_summaries(&mut summaries, point, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED, &not_counted);
        let incidental_analyses: Vec<PointAnalysis> = INCIDENTAL_READINGS.iter().map(|reading| analyse_point(evaluated, *reading)).collect();
        for (reading, analysis) in INCIDENTAL_READINGS.iter().zip(&incidental_analyses) {
            add_analysis_to_summaries(&mut summaries, point, *reading, analysis);
        }
        let changed_judged = flags_winners_signature(&counted) != flags_winners_signature(&not_counted);
        let changed_incidental_counting_the_judged_items = flags_winners_signature(&incidental_analyses[0]) != flags_winners_signature(&incidental_analyses[1]);
        summaries.entry(("q2_flags_winners_changed", "b1_w2".to_string(), group_of(point), "all".to_string(), u8::from(changed_judged).to_string())).or_default().add(point);
        summaries.entry(("q2_flags_winners_changed", "b2_w2".to_string(), group_of(point), "all".to_string(), u8::from(changed_incidental_counting_the_judged_items).to_string())).or_default().add(point);
        let tally = flags_question_tallies.entry(group_of(point)).or_default();
        tally.points += 1;
        tally.separable_points_counted += u64::from(counted.flags_winners_by_accounting_arm.iter().any(|arm| arm.separable_at_position_six));
        tally.separable_points_not_counted += u64::from(not_counted.flags_winners_by_accounting_arm.iter().any(|arm| arm.separable_at_position_six));
        tally.changed_points_judged += u64::from(changed_judged);
        tally.changed_points_incidental_counting_the_judged_items += u64::from(changed_incidental_counting_the_judged_items);
        analyses.push((counted, not_counted));
    }

    for ((kind, reading_label, group, arm, value), description) in &summaries {
        writer.line(&format!("name=summary kind={kind} reading={reading_label} group={group} arm={arm} value={value} {}", description.render("")));
    }

    let expected_cell_count = EXPECTED_POINT_COUNT * CELLS_PER_POINT;
    let count_holds = point_count == EXPECTED_POINT_COUNT && ranked_cell_count == expected_cell_count;
    writer.line(&format!(
        "name=verdict check=point_and_cell_count points={point_count} expected_points={EXPECTED_POINT_COUNT} ranked_cells={ranked_cell_count} expected_cells={expected_cell_count} count_ok={count_holds}"
    ));

    for control in positive_controls_of(judgement) {
        let (checked, failures) = evaluated_points.iter().map(|evaluated| positive_control_failures(evaluated, control)).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
        writer.line(&format!("name=verdict check=positive_control control={} checked={checked} failures={failures} holds={}", control.label(), render_verdict_holds(failures)));
    }
    let (checked, failures) = evaluated_points.iter().map(negative_control_no_window_failures).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
    writer.line(&format!("name=verdict check=negative_control_no_window_no_wrong_free checked={checked} failures={failures} holds={}", render_verdict_holds(failures)));
    let (unknown_heads_checked, unknown_heads_failures) = evaluated_points.iter().map(negative_control_no_unknown_heads_failures).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
    writer.line(&format!("name=verdict check=negative_control_no_unknown_heads_no_accounting_effect clauses_judged=b1_only checked={unknown_heads_checked} failures={unknown_heads_failures} holds={}", render_verdict_holds(unknown_heads_failures)));
    let (incidental_checked, carry_item_false, reject_item_false) = evaluated_points
        .iter()
        .map(negative_control_no_unknown_heads_incidental_clauses_counting_the_judged_items)
        .fold((0, 0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1, sum.2 + part.2));
    writer.line(&format!(
        "name=incidental check=negative_control_no_unknown_heads_b2_only_clauses judged=no cells={incidental_checked} u12_false_cells={carry_item_false} u13_false_cells={reject_item_false}"
    ));
    writer.line("name=incidental check=incidental_quantities judged=no fields=per_generation_rows_dropped,per_generation_rows_dropped_total,i313_syntactic_violations,crates_step_reading_differences source=cell_lines");

    let index_by_description: BTreeMap<String, usize> = evaluated_points.iter().enumerate().map(|(index, evaluated)| (evaluated.point.describe(), index)).collect();
    let partner_index = |point: &SamplePoint| -> usize { index_by_description[&point.describe()] };
    let all_readings: Vec<Reading> = [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED].into_iter().chain(INCIDENTAL_READINGS).collect();
    let mut kept_generation_count_checked = 0u64;
    let mut kept_generation_count_failures = 0u64;
    let mut published_generations_checked = 0u64;
    let mut published_generations_mismatches = 0u64;
    for evaluated in &evaluated_points {
        let point = &evaluated.point;
        if point.retired_kept_generation_count != RETIRED_KEPT_GENERATION_COUNT_VALUES[0] {
            let partner = &evaluated_points[partner_index(&point_with(point, PointChange::RetiredKeptGenerationCount(RETIRED_KEPT_GENERATION_COUNT_VALUES[0])))];
            for cell in &evaluated.cells {
                for reading in &all_readings {
                    kept_generation_count_checked += 1;
                    kept_generation_count_failures += u64::from(vector_under(cell, point, *reading) != partner.vector(cell.flags_policy, cell.accounting_policy, *reading));
                }
            }
        }
        if point.published_generations == 1 {
            let partner = &evaluated_points[partner_index(&point_with(point, PointChange::PublishedGenerations(10)))];
            for flags_policy in FLAGS_ARMS {
                published_generations_checked += 1;
                published_generations_mismatches += u64::from(evaluated.outcome_under_the_arms(flags_policy, AccountingPolicy::NoCarry).rows_lost != partner.outcome_under_the_arms(flags_policy, AccountingPolicy::NoCarry).rows_lost);
            }
        }
    }
    writer.line(&format!(
        "name=verdict check=negative_control_k_keep_not_read checked={kept_generation_count_checked} failures={kept_generation_count_failures} holds={}",
        render_verdict_holds(kept_generation_count_failures)
    ));
    writer.line(&format!(
        "name=verdict check=ledger_two_paths cells={ranked_cell_count} mismatches_b1={ledger_mismatches_excluding_the_judged_items} mismatches_b2={ledger_mismatches_counting_the_judged_items} holds={}",
        render_verdict_holds(ledger_mismatches_excluding_the_judged_items + ledger_mismatches_counting_the_judged_items)
    ));
    for (name, anchor_checked, mismatches) in anchor_failures(&evaluated_points) {
        writer.line(&format!("name=verdict check=anchor anchor={name} checked={anchor_checked} mismatches={mismatches} holds={}", render_verdict_holds(mismatches)));
    }
    writer.line(&format!(
        "name=verdict check=anchor anchor=no_carry_rows_lost_same_at_g_gens_1_and_10 checked={published_generations_checked} mismatches={published_generations_mismatches} holds={}",
        render_verdict_holds(published_generations_mismatches)
    ));
    writer.line(&format!(
        "name=verdict check=anchor anchor=today_tree_dimension_statistics_is_1 checked=1 mismatches={} holds={}",
        u64::from(TREE_DIMENSION_STATISTIC_COUNT_VALUES[0] != 1),
        render_verdict_holds(u64::from(TREE_DIMENSION_STATISTIC_COUNT_VALUES[0] != 1))
    ));

    // 停机条款第 1、2 项的模型一侧：实现两盘几何、只有认识的那一个头时，行数与台阶认法要与 crates/ 那两段读出的相同。
    let crates_rows_with_one_step = CRATES_POOL_WIDE_ROWS + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT + CRATES_KNOWN_HEAD_STEP_ROWS;
    let mut stop_rows_checked = 0u64;
    let mut stop_rows_mismatches = 0u64;
    let mut stop_steps_checked = 0u64;
    let mut stop_steps_mismatches = 0u64;
    for evaluated in evaluated_points.iter().filter(|evaluated| evaluated.point.geometry == Geometry::ImplementationTwoDevices && evaluated.point.unknown_writable_head_count == 0) {
        for cell in evaluated.cells.iter().filter(|cell| cell.writable_generations > 0) {
            if evaluated.point.tree_dimension_statistic_count == 1 {
                stop_rows_checked += 1;
                stop_rows_mismatches += u64::from(cell.rows_written_this_generation != CRATES_POOL_WIDE_ROWS + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT || cell.rows_written_all_rows != crates_rows_with_one_step);
            }
            stop_steps_checked += 1;
            stop_steps_mismatches += u64::from(cell.trajectory_crates_step_reading_differences.is_none_or(|trajectory| trajectory.peak > 0));
        }
    }
    writer.line(&format!(
        "name=verdict check=stop_clause_rows_match_crates checked={stop_rows_checked} mismatches={stop_rows_mismatches} w1_expected={} w2_expected={crates_rows_with_one_step} holds={}",
        CRATES_POOL_WIDE_ROWS + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT,
        render_verdict_holds(stop_rows_mismatches)
    ));
    writer.line(&format!(
        "name=verdict check=stop_clause_step_reading_matches_crates checked={stop_steps_checked} mismatches={stop_steps_mismatches} holds={}",
        render_verdict_holds(stop_steps_mismatches)
    ));

    // 第八节判别力自证：弄坏之后还绿的格数（not_turning_red）不为 0，那一位在那一对上没有判别力（第十节）。
    let mut flags_question_checked = 0u64;
    let mut flags_question_not_turning_red = 0u64;
    let mut accounting_question_position_six_checked = 0u64;
    let mut accounting_question_position_six_not_turning_red = 0u64;
    let mut accounting_question_position_seven_checked = 0u64;
    let mut accounting_question_dangling_carried_rows_checked = 0u64;
    let mut accounting_question_dangling_carried_rows_not_turning_red = 0u64;
    let mut accounting_question_position_seven_not_turning_red = 0u64;
    for evaluated in &evaluated_points {
        let point = &evaluated.point;
        if point.invisible_window_generations == 0 {
            for accounting_policy in ACCOUNTING_ARMS {
                flags_question_checked += 1;
                flags_question_not_turning_red += u64::from(!flags_question_self_proof_turns_red(evaluated, accounting_policy));
            }
        }
        let acting_flags = FLAGS_ARMS.iter().copied().filter(|flags_policy| accounting_arm_acts_under(*flags_policy, point));
        if point.tree_dimension_statistic_count == 1 && point.unknown_writable_head_count > 0 {
            for flags_policy in acting_flags {
                if point.unknown_writable_head_count == 2 && !point.initial_publish_changed_the_watermark {
                    accounting_question_position_six_checked += 1;
                    accounting_question_position_six_not_turning_red += u64::from(!accounting_question_position_six_self_proof_turns_red(evaluated, flags_policy));
                }
                if point.initial_publish_changed_the_watermark {
                    accounting_question_position_seven_checked += 1;
                    accounting_question_position_seven_not_turning_red += u64::from(!accounting_question_position_seven_self_proof_turns_red(evaluated, flags_policy));
                }
                if is_a_point_of_the_dangling_carried_rows_self_proof(point) {
                    accounting_question_dangling_carried_rows_checked += 1;
                    accounting_question_dangling_carried_rows_not_turning_red += u64::from(!accounting_question_dangling_carried_rows_self_proof_turns_red(evaluated, flags_policy));
                }
            }
        }
    }
    let mut self_proofs = vec![
        ("self_proof_q2_reject_entry_versus_skip_position_six_where_w_gens_0", flags_question_checked, flags_question_not_turning_red),
        ("self_proof_q1_carry_versus_keep_with_root_position_six_where_h_new_2_lpc_0_r_tree_dim_1", accounting_question_position_six_checked, accounting_question_position_six_not_turning_red),
        ("self_proof_q1_carry_versus_keep_with_root_position_seven_where_h_new_positive_lpc_1_r_tree_dim_1", accounting_question_position_seven_checked, accounting_question_position_seven_not_turning_red),
    ];
    if judgement == Judgement::DanglingCarriedRowsClause {
        self_proofs.push(("self_proof_q1_carry_versus_no_carry_position_six_where_h_new_2_lpc_1_r_tree_dim_1", accounting_question_dangling_carried_rows_checked, accounting_question_dangling_carried_rows_not_turning_red));
    }
    for (name, self_proof_checked, not_turning_red) in self_proofs {
        writer.line(&format!("name=verdict check={name} checked={self_proof_checked} not_turning_red={not_turning_red} turns_red={}", render_verdict_holds(not_turning_red)));
    }
    writer.line(&format!(
        "name=verdict check=non_distinguishing_points points_all_fifteen_cells_equal_q2_counted={non_distinguishing_points_counted} points_all_fifteen_cells_equal_q2_not_counted={non_distinguishing_points_not_counted}"
    ));
    writer.line(&format!("name=verdict check=q1_independent_of_q2_reading points_where_q1_differs={points_where_the_accounting_winners_depend_on_the_reject_entry_reading} holds={}", render_verdict_holds(points_where_the_accounting_winners_depend_on_the_reject_entry_reading)));

    // Q1 判定：一个分组、一条 flags 臂下，各点的记账胜者是同一个集合 ⇒ 记这个集合；随点变 ⇒ 依赖取样点（翻转点在 summary 行里按集合列）。
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let mut winner_sets: BTreeMap<(&'static str, String, String, String), Vec<(String, u64)>> = BTreeMap::new();
    for ((kind, reading_label, group, arm, value), description) in &summaries {
        if *kind == "q1_winners" || *kind == "q1_overall_winner_accounting_arms" || *kind == "q2_flags_winners" || *kind == "q2_overall_winner_flags_arms" {
            winner_sets.entry((*kind, reading_label.clone(), group.clone(), arm.clone())).or_default().push((value.clone(), description.point_count));
        }
    }
    for ((kind, reading_label, group, arm), sets) in &winner_sets {
        let winner_sets_judgement = if sets.len() == 1 { "same_set_everywhere" } else { "depends_on_point" };
        let rendered_sets = sets.iter().map(|(set, count)| format!("{set}:{count}")).collect::<Vec<String>>().join("|");
        writer.line(&format!("name=verdict check={kind} reading={reading_label} group={group} arm={arm} distinct_winner_sets={} judgement={winner_sets_judgement} winner_sets={rendered_sets}", sets.len()));
    }
    for (group, tally) in &flags_question_tallies {
        writer.line(&format!(
            "name=verdict check=q2_reject_entry_versus_skip group={group} points={} separable_points_q2_counted={} separable_points_q2_not_counted={} \
             flags_winners_changed_points_b1_w2={} flags_winners_changed_points_b2_w2={} judgement_b1_w2={}",
            tally.points,
            tally.separable_points_counted,
            tally.separable_points_not_counted,
            tally.changed_points_judged,
            tally.changed_points_incidental_counting_the_judged_items,
            if tally.changed_points_judged > 0 { "changes_tier" } else { "same_tier" }
        ));
    }

    for pair in sensitivity_pairs() {
        let mut pair_count = 0u64;
        let mut flipped = PointSetDescription::default();
        for (index, evaluated) in evaluated_points.iter().enumerate() {
            let Some(partner) = partner_of(&pair, &evaluated.point) else {
                continue;
            };
            let partner_analyses = &analyses[partner_index(&partner)];
            let own_analyses = &analyses[index];
            pair_count += 1;
            let differs = match pair.verdict {
                SensitivityVerdict::AccountingWinners => accounting_winners_signature(&own_analyses.0) != accounting_winners_signature(&partner_analyses.0),
                SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip => {
                    let signature = |point_analyses: &(PointAnalysis, PointAnalysis)| {
                        (
                            flags_winners_signature(&point_analyses.0),
                            flags_winners_signature(&point_analyses.1),
                            point_analyses.0.flags_winners_by_accounting_arm.iter().map(|arm| (arm.position_six_reject_entry_versus_skip, arm.separable_at_position_six)).collect::<Vec<_>>(),
                            point_analyses.1.flags_winners_by_accounting_arm.iter().map(|arm| (arm.position_six_reject_entry_versus_skip, arm.separable_at_position_six)).collect::<Vec<_>>(),
                        )
                    };
                    signature(own_analyses) != signature(partner_analyses)
                }
            };
            if differs {
                flipped.add(&evaluated.point);
            }
        }
        let verdict_label = match pair.verdict {
            SensitivityVerdict::AccountingWinners => "q1_accounting_winners",
            SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip => "q2_flags_winners_and_reject_entry_versus_skip",
        };
        let flipped_judgement = if flipped.point_count > 0 { "depends_on_point" } else { "two_points_same" };
        writer.line(&format!(
            "name=verdict check=sensitivity pair={} verdict={verdict_label} pairs={pair_count} judgement={flipped_judgement} {}",
            pair.name,
            flipped.render("flipped_")
        ));
    }
    if judgement == Judgement::DanglingCarriedRowsClause {
        // 第二次跑第六节「加一个量」：判 ㊀ 只看 `legal_pooled` 那一行，两个 `r_tree_dim` 分组照报；E113 口径两组单报、标不判。
        let tallies_by_group = acceptance_tallies(&evaluated_points);
        for (group, judged) in ACCEPTANCE_GROUPS {
            writer.line(&render_acceptance_line(judgement, group, judged, &tallies_by_group[group]));
        }
        let acceptance_count_mismatches: u64 = EXPECTED_ACCEPTANCE_POINT_COUNTS
            .iter()
            .map(|(group, judged_points, excluded_points)| {
                let tally = &tallies_by_group[group];
                u64::from(tally.judged_points != *judged_points || tally.excluded_points_without_unknown_writable_heads != *excluded_points || tally.form == AcceptanceForm::NoJudgedPoints)
            })
            .sum();
        writer.line(&format!(
            "name=verdict check=acceptance_point_counts_and_form_present checked={} mismatches={acceptance_count_mismatches} holds={}",
            EXPECTED_ACCEPTANCE_POINT_COUNTS.len(),
            render_verdict_holds(acceptance_count_mismatches)
        ));

        // 停机条款第 6 项：两种判据逐（点, 格）比，U5 以外一格都不许不同；Q2 判定一点都不许不同。
        let mut cells_compared = 0u64;
        let mut cell_mismatches = 0u64;
        let mut flags_question_mismatches = 0u64;
        for evaluated in &evaluated_points {
            let vacuous = evaluate_point(evaluated.point, Judgement::VacuousCarriedRowsClause);
            for (dangling_cell, vacuous_cell) in evaluated.cells.iter().zip(&vacuous.cells) {
                cells_compared += 1;
                cell_mismatches += u64::from(!cells_differ_only_in_the_carried_rows_clause(vacuous_cell, dangling_cell, &evaluated.point));
            }
            flags_question_mismatches += u64::from(!flags_question_judgements_agree(&vacuous, evaluated));
        }
        writer.line(&format!(
            "name=verdict check=judgements_differ_only_in_u5 cells={cells_compared} mismatches={cell_mismatches} holds={}",
            render_verdict_holds(cell_mismatches)
        ));
        writer.line(&format!(
            "name=verdict check=judgements_q2_identical points={point_count} mismatches={flags_question_mismatches} holds={}",
            render_verdict_holds(flags_question_mismatches)
        ));

        // 停机条款第 7 项：新 U5 照 N 判与照 `h_new > 0` 判在每个点上一致。
        let object_mismatches: u64 = evaluated_points.iter().map(|evaluated| u64::from(!carried_rows_object_count_matches_the_unknown_writable_heads(&evaluated.point))).sum();
        writer.line(&format!(
            "name=verdict check=u5_object_count_matches_h_new points={point_count} mismatches={object_mismatches} holds={}",
            render_verdict_holds(object_mismatches)
        ));
    }
    writer.finish();
}

#[cfg(test)]
mod tests;
