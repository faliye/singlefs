//! 模式 stop-clauses：S1–S6 与 A1–A7（跑产物之前）。

use super::*;

// ───────────────────────────── 模式 stop-clauses：S1–S6 与 A1–A7（跑产物之前） ─────────────────────────────

/// S3 + S4 在一个域上：crates 的枚举与装置自己照跑的计数逐项比。交回 S3、S4 过没过；crates 那一遍判段没跑的，打一行这一格本次未跑
/// （连同崩溃点重放判段本次未跑那一行记号）、交回 [`CratesComparisonClauses::JudgeStageNotRun`]，装置自己那一遍不跑。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：输出、格名、输入、计划、序号、crates 的展开、线程数"
)]
pub(super) fn report_crates_comparison(
    output: &mut ResultLines,
    label: &str,
    context: &EvaluationContext<'_>,
    plan: &LocalPlan<'_>,
    ordinals: &[u64],
    expansion_of: &dyn Fn(usize) -> CrashPointReplaySegmentExpansion,
    threads: usize,
) -> CratesComparisonClauses {
    let Some(comparison) =
        compare_with_the_crates(context.stream, context.table, plan, ordinals, expansion_of)
    else {
        output.line(format!(
            "name=stop_clause_crates cell={label} domain_states={} judge_stage=not_run",
            ordinals.len()
        ));
        println!(
            "{}",
            singlefs_verification::crash_state_enumeration::crash_point_replay_judge_not_run_line(
                "e161_crates_comparison"
            )
        );
        return CratesComparisonClauses::JudgeStageNotRun;
    };
    let device = evaluate_stream_domain(&StreamDomainRun {
        label: format!("{label}-baseline"),
        context,
        plan,
        ordinals,
        control_ordinals: &BTreeSet::new(),
        timing: TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
        threads,
        depth: EvaluationDepth::BaselineOnly,
    });
    let device_tally = tally_with_every_write_persisted(context, &device.tally);
    let device_text = tally_text(&device_tally);
    let crates_text = tally_text(&comparison.crates_tally);
    let expected_observed = u64::try_from(ordinals.len()).expect("个数")
        + u64::from(!includes_every_write_persisted(context.stream.name));
    let enumeration_clause_holds = !comparison.writes_table_differs
        && comparison.persisted_mismatches == 0
        && comparison.compared_states == u64::try_from(ordinals.len()).expect("个数")
        && comparison.observed_states == expected_observed;
    let tally_clause_holds = device_text == crates_text;
    output.line(format!(
        "name=stop_clause_crates cell={label} domain_states={} crates_observed_states={} compared_states={} writes_table_differs={} persisted_mismatches={} clause_S3_holds={} clause_S4_holds={}",
        ordinals.len(),
        comparison.observed_states,
        comparison.compared_states,
        bool_text(comparison.writes_table_differs),
        comparison.persisted_mismatches,
        bool_text(enumeration_clause_holds),
        bool_text(tally_clause_holds),
    ));
    output.line(format!(
        "name=stop_clause_tally cell={label} side=device {device_text}"
    ));
    output.line(format!(
        "name=stop_clause_tally cell={label} side=crates {crates_text}"
    ));
    CratesComparisonClauses::Judged {
        enumeration_clause_holds,
        tally_clause_holds,
    }
}

/// 对照样本上跑两遍（5.1 的四件事），第二遍与第一遍逐状态比键与结局（PC2a、PC4a）、两遍的照跑结局（PC2d、PC4d）；
/// 第一遍自己核 S5（记录跑与照跑逐项相同）与 S6（Q4e）。交回（S5 过没过，S6 过没过）。
pub(super) fn report_control_double_run(
    output: &mut ResultLines,
    label: &str,
    context: &EvaluationContext<'_>,
    plan: &LocalPlan<'_>,
    control: &BTreeSet<u64>,
    threads: usize,
) -> (bool, bool) {
    let ordinals: Vec<u64> = control.iter().copied().collect();
    let run_once = |pass: &str| {
        evaluate_stream_domain(&StreamDomainRun {
            label: format!("{label}-control-{pass}"),
            context,
            plan,
            ordinals: &ordinals,
            control_ordinals: control,
            timing: TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
            threads,
            depth: EvaluationDepth::FourSteps,
        })
    };
    let first = run_once("first");
    let second = run_once("second");
    let keys_of = |data: &ControlStateData| {
        [
            sequence_key(&data.consult_calls),
            sequence_key(&data.ignore_calls),
            set_key(data.root, &data.walk_elements),
            set_key(data.root, &data.unit_elements),
            sequence_key(&data.full_calls),
        ]
    };
    let mut recovery_key_changed = 0u64;
    let mut checker_key_changed = 0u64;
    let mut recovery_outcome_changed = 0u64;
    let mut verdict_changed = 0u64;
    for (left, right) in first.control.iter().zip(&second.control) {
        assert_eq!(left.ordinal, right.ordinal, "两遍对照样本按同一个次序");
        let (left_keys, right_keys) = (keys_of(left), keys_of(right));
        recovery_key_changed += u64::from(left_keys[..2] != right_keys[..2]);
        checker_key_changed += u64::from(left_keys[2..] != right_keys[2..]);
        recovery_outcome_changed += u64::from(
            left.consult_outcome != right.consult_outcome
                || left.ignore_outcome != right.ignore_outcome,
        );
        verdict_changed += u64::from(left.verdict_outcome != right.verdict_outcome);
    }
    let compared = u64::try_from(first.control.len().min(second.control.len())).expect("个数");
    let determinism_clause_holds = first.consult_report_differs == 0
        && first.ignore_report_differs == 0
        && first.verdicts_differ == 0
        && recovery_outcome_changed == 0
        && verdict_changed == 0
        && compared == u64::try_from(ordinals.len()).expect("个数");
    let walk_subset_clause_holds = first.walk_reads_outside_normal_reads == 0;
    output.line(format!(
        "name=stop_clause_determinism cell={label} control_states={} compared_states={compared} recorded_consult_report_differs={} recorded_ignore_report_differs={} recorded_verdicts_differ={} second_run_recovery_outcome_changed={recovery_outcome_changed} second_run_verdict_changed={verdict_changed} walk_unit_reads_outside_normal_unit_reads={} clause_S5_holds={} clause_S6_holds={}",
        ordinals.len(),
        first.consult_report_differs,
        first.ignore_report_differs,
        first.verdicts_differ,
        first.walk_reads_outside_normal_reads,
        bool_text(determinism_clause_holds),
        bool_text(walk_subset_clause_holds),
    ));
    output.line(format!(
        "name=positive_control_same_state_twice cell={label} states={compared} pc2a_recovery_keys_changed={recovery_key_changed} pc4a_checker_keys_changed={checker_key_changed} pc2d_recovery_reports_changed={recovery_outcome_changed} pc4d_verdicts_changed={verdict_changed} pc2a_holds={} pc4a_holds={} pc2d_holds={} pc4d_holds={}",
        bool_text(recovery_key_changed == 0 && compared > 0),
        bool_text(checker_key_changed == 0 && compared > 0),
        bool_text(recovery_outcome_changed == 0 && compared > 0),
        bool_text(verdict_changed == 0 && compared > 0),
    ));
    (determinism_clause_holds, walk_subset_clause_holds)
}

pub(super) fn run_stop_clauses(output: &mut ResultLines) -> i32 {
    let threads = threads_from_environment();
    output.line(format!(
        "name=config experiment=E161 mode=stop_clauses threads={threads} registration=research/prompts/e161-preregistration.md"
    ));
    let constants = local_constants_against_the_crates();
    let constants_clause_holds = constants
        .iter()
        .all(|(_name, local, crates)| local == crates);
    let constant_fields: Vec<String> = constants
        .iter()
        .map(|(name, local, crates)| format!("{name}={local}/{crates}"))
        .collect();
    output.line(format!(
        "name=stop_clause_constants {} clause_S1_holds={}",
        constant_fields.join(" "),
        bool_text(constants_clause_holds)
    ));
    let first = prepare_first_stream();
    let second = prepare_second_stream();
    let (bases_identical, prefix_identical) = shared_prefix_is_identical(&first, &second);
    let mut streams_clause_holds = bases_identical && prefix_identical;
    for stream in [&first, &second] {
        let check = check_stream_shape(stream);
        streams_clause_holds &=
            check.segment_lengths_match && check.writes_match && check.roots_match;
        output.line(format!(
            "name=stop_clause_stream stream={} writes={} segments={} roots={} segment_lengths_match={} writes_match={} roots_match={}",
            stream.name.label(),
            stream.writes.len(),
            stream.segments.len(),
            stream.root_write_count,
            bool_text(check.segment_lengths_match),
            bool_text(check.writes_match),
            bool_text(check.roots_match),
        ));
    }
    output.line(format!(
        "name=stop_clause_shared_prefix bases_identical={} shared_prefix_segments={SHARED_PREFIX_SEGMENTS} shared_prefix_segments_identical={} clause_S2_holds={}",
        bool_text(bases_identical),
        bool_text(prefix_identical),
        bool_text(streams_clause_holds)
    ));
    let mut anchors_hold = capacity_anchors_hold();
    output.line(format!(
        "name=anchor_capacity a6_a7_hold={}",
        bool_text(anchors_hold)
    ));
    let mut enumeration_clause_holds = true;
    let mut tally_clause_holds = true;
    let mut crates_comparison_verdicts = CratesComparisonVerdicts::new();
    let mut determinism_clause_holds = true;
    let mut walk_subset_clause_holds = true;
    for stream in [&first, &second] {
        let table = TornWriteTable::of(stream);
        anchors_hold &= report_stream_anchors(output, stream, &table).every_anchor_holds;
        let context = EvaluationContext::new(stream, &table);
        let full = LocalPlan::new(
            stream,
            &table,
            domain_expansions(stream, LocalExpansion::EveryCombination),
        );
        let small = small_domain_ordinals(stream, &full);
        let small_segments: BTreeSet<usize> = small
            .iter()
            .map(|ordinal| full.segment_of(*ordinal))
            .collect();
        crates_comparison_verdicts.absorb(report_crates_comparison(
            output,
            &format!("{}_small", stream.name.label()),
            &context,
            &full,
            &small,
            &|segment_index| {
                if small_segments.contains(&segment_index) {
                    CrashPointReplaySegmentExpansion::EveryProperSubset
                } else {
                    CrashPointReplaySegmentExpansion::NotExpanded
                }
            },
            threads,
        ));
        let quick = LocalPlan::new(
            stream,
            &table,
            domain_expansions(
                stream,
                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
            ),
        );
        let quick_ordinals = every_ordinal(stream, &quick);
        crates_comparison_verdicts.absorb(report_crates_comparison(
            output,
            &format!("{}_quick", stream.name.label()),
            &context,
            &quick,
            &quick_ordinals,
            &|segment_index| quick.expansion_by_segment[segment_index].crates_expansion(),
            threads,
        ));
        let mut domain: Vec<u64> = small
            .iter()
            .chain(&stride_domain_ordinals(stream, &full))
            .copied()
            .collect();
        domain.sort_unstable();
        let control = control_ordinals(stream, &full, &domain);
        let (control_determinism_holds, control_walk_subset_holds) = report_control_double_run(
            output,
            &format!("{}_d1", stream.name.label()),
            &context,
            &full,
            &control,
            threads,
        );
        determinism_clause_holds &= control_determinism_holds;
        walk_subset_clause_holds &= control_walk_subset_holds;
    }
    enumeration_clause_holds &= crates_comparison_verdicts.enumeration_clause_holds;
    tally_clause_holds &= crates_comparison_verdicts.tally_clause_holds;
    if crates_comparison_verdicts.judge_stage_not_run {
        output.line(
            "name=verdict scope=stop_clauses clause_S3_holds=not_run clause_S4_holds=not_run judge_stage=not_run"
                .to_string(),
        );
        return CRATES_COMPARISON_NOT_RUN_EXIT_CODE;
    }
    output.line(format!(
        "name=verdict scope=stop_clauses clause_S1_holds={} clause_S2_holds={} clause_S3_holds={} clause_S4_holds={} clause_S5_holds={} clause_S6_holds={} anchors_a1_to_a7_hold={}",
        bool_text(constants_clause_holds),
        bool_text(streams_clause_holds),
        bool_text(enumeration_clause_holds),
        bool_text(tally_clause_holds),
        bool_text(determinism_clause_holds),
        bool_text(walk_subset_clause_holds),
        bool_text(anchors_hold),
    ));
    if constants_clause_holds
        && streams_clause_holds
        && enumeration_clause_holds
        && tally_clause_holds
        && determinism_clause_holds
        && anchors_hold
    {
        0
    } else {
        3
    }
}

/// 开发用：只在新池新建文件流与覆盖写固定脚本流的甲二域上跑 S3、S4 与对照样本（新池新建文件流）的两遍，几分钟之内看出装置有没有写错；不出产物。
pub(super) fn run_first_stream_stop_clause_probe(output: &mut ResultLines) -> i32 {
    let threads = threads_from_environment();
    let first = prepare_first_stream();
    let second = prepare_second_stream();
    for stream in [&first, &second] {
        let table = TornWriteTable::of(stream);
        let _anchors = report_stream_anchors(output, stream, &table);
        let context = EvaluationContext::new(stream, &table);
        let quick = LocalPlan::new(
            stream,
            &table,
            domain_expansions(
                stream,
                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
            ),
        );
        let quick_ordinals = every_ordinal(stream, &quick);
        let _quick = report_crates_comparison(
            output,
            &format!("{}_quick", stream.name.label()),
            &context,
            &quick,
            &quick_ordinals,
            &|segment_index| quick.expansion_by_segment[segment_index].crates_expansion(),
            threads,
        );
    }
    let table = TornWriteTable::of(&first);
    let context = EvaluationContext::new(&first, &table);
    let full = LocalPlan::new(
        &first,
        &table,
        domain_expansions(&first, LocalExpansion::EveryCombination),
    );
    let small = small_domain_ordinals(&first, &full);
    let small_segments: BTreeSet<usize> = small
        .iter()
        .map(|ordinal| full.segment_of(*ordinal))
        .collect();
    let _small = report_crates_comparison(
        output,
        "first_small",
        &context,
        &full,
        &small,
        &|segment_index| {
            if small_segments.contains(&segment_index) {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        },
        threads,
    );
    let control: BTreeSet<u64> = small.iter().copied().collect();
    let _control =
        report_control_double_run(output, "first_small", &context, &full, &control, threads);
    0
}

#[cfg(test)]
mod tests;
