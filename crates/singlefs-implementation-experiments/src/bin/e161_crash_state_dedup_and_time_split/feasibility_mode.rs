//! 模式 feasibility：可行性档（登记 12.2）。

use super::*;

// ───────────────────────────── 模式 feasibility：可行性档（登记 12.2） ─────────────────────────────

/// 可行性档里大段（24 写段）取段内序号最小的这么多个状态（登记 12.2；重跑登记 1.3「段头」）。
const FEASIBILITY_HEAD_STATES: u64 = 4_096;

/// 取样段里大段（24 写段）的段头：段内序号 0 到 4095（按全量计划的序号）。
pub(super) fn strided_segment_heads(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
    let mut ordinals = Vec::new();
    for (segment_index, segment) in stream.segments.iter().enumerate() {
        if is_sampled_segment(stream.name, segment_index) && segment.len() == STRIDED_SEGMENT_WRITES
        {
            let first = plan.first_ordinal_by_segment[segment_index];
            let count = plan.count_by_segment[segment_index].min(FEASIBILITY_HEAD_STATES);
            ordinals.extend(first..first + count);
        }
    }
    ordinals
}

/// S4 在大段段头上的形态：crates 按段枚举要把整段 1677 万个跑完，这里改用 crates 的单状态入口
/// `evaluate_state_for_versions`，在装置给的同一批持久集合上逐个评，计数与装置照跑的逐项比。交回相不相同。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
fn report_per_state_crates_comparison(
    output: &mut ResultLines,
    label: &str,
    context: &EvaluationContext<'_>,
    plan: &LocalPlan<'_>,
    ordinals: &[u64],
    threads: usize,
) -> bool {
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
    let chunk = ordinals.len().div_ceil(threads.max(1)).max(1);
    let merged = Mutex::new(CrashPointReplayTally::default());
    std::thread::scope(|scope| {
        for part in ordinals.chunks(chunk) {
            let merged = &merged;
            scope.spawn(move || {
                let mut tally = CrashPointReplayTally::default();
                for ordinal in part {
                    let _report = singlefs_verification::crash::evaluate_state_for_versions(
                        &context.stream.base,
                        &context.table.writes,
                        plan.persisted_of(*ordinal),
                        context.stream.judged_root_index,
                        &context.stream.versions,
                        &mut tally,
                    );
                }
                add_tally(&mut merged.lock().expect("并计数的锁"), &tally);
            });
        }
    });
    let crates_tally = merged.into_inner().expect("并计数的锁");
    let device_text = tally_text(&device.tally);
    let crates_text = tally_text(&crates_tally);
    let holds = device_text == crates_text;
    output.line(format!(
        "name=stop_clause_crates_per_state cell={label} states={} clause_S4_holds={}",
        ordinals.len(),
        bool_text(holds)
    ));
    output.line(format!(
        "name=stop_clause_tally cell={label} side=device {device_text}"
    ));
    output.line(format!(
        "name=stop_clause_tally cell={label} side=crates {crates_text}"
    ));
    holds
}

/// 可行性档：新池新建文件流 D1-small、两条流的 D1-quick、两个大段（24 写段）各取段头 4096 个状态；
/// 停机条款、阳性对照与 crates 的对拍都只在这个小域上做。G 行一律不判（只跑了可行性档）。
/// 要评的三个域的大小先与 A4 比，对不上一个状态都不评、退 3（重跑登记 I-7 的域守卫）。
#[allow(
    clippy::too_many_lines,
    reason = "可行性档的全部格按登记 12.2 的次序排下来：S1、S2、锚点、与 crates 对拍、两遍对照、各格结果行、阳性对照、判定行"
)]
pub(super) fn run_feasibility(output: &mut ResultLines) -> i32 {
    let threads = threads_from_environment();
    output.line(format!(
        "name=config experiment=E161 mode=feasibility threads={threads} registration=research/prompts/e161-preregistration.md rerun_registration=research/prompts/e161-r3-prereg.md domain=first_small_53+quick_54_60+largest_segment_heads_4096"
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
    let first_table = TornWriteTable::of(&first);
    let second_table = TornWriteTable::of(&second);
    let streams_and_tables = [(&first, &first_table), (&second, &second_table)];
    let mut anchors_hold = capacity_anchors_hold();
    let mut evaluated_domain_sizes_hold = true;
    for (stream, table) in streams_and_tables {
        let anchors = report_stream_anchors(output, stream, table);
        anchors_hold &= anchors.every_anchor_holds;
        evaluated_domain_sizes_hold &= anchors.evaluated_domain_sizes_hold;
    }
    // 域守卫（重跑登记 I-7）：要评的域与 A4 对不上时一个状态都不评。第 1 次的段形一变，「写数少于 26」把 1677 万个状态的段整段收进
    // D1-small，装置照样往下评、撞内存上限才停；先核域大小，对不上就打判决行、退 3（停机条款 S9）。
    if !evaluated_domain_sizes_hold {
        output.line(format!(
            "name=verdict scope=feasibility clause_S1_holds={} clause_S2_holds={} anchors_a1_to_a7_hold={} evaluated_domain_sizes_hold=false states_evaluated=false",
            bool_text(constants_clause_holds),
            bool_text(streams_clause_holds),
            bool_text(anchors_hold),
        ));
        return 3;
    }
    let mut enumeration_clause_holds = true;
    let mut tally_clause_holds = true;
    let mut crates_comparison_verdicts = CratesComparisonVerdicts::new();
    let mut determinism_clause_holds = true;
    let mut walk_subset_clause_holds = true;
    let mut positive_controls: Vec<(String, bool)> = Vec::new();
    for (stream, table) in streams_and_tables {
        let name = stream.name.label();
        let context = EvaluationContext::new(stream, table);
        let full = LocalPlan::new(
            stream,
            table,
            domain_expansions(stream, LocalExpansion::EveryCombination),
        );
        let quick = LocalPlan::new(
            stream,
            table,
            domain_expansions(
                stream,
                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
            ),
        );
        let small = match stream.name {
            StreamName::First => small_domain_ordinals(stream, &full),
            StreamName::Second => Vec::new(),
        };
        let heads = strided_segment_heads(stream, &full);
        let quick_ordinals = every_ordinal(stream, &quick);
        if !small.is_empty() {
            let small_segments: BTreeSet<usize> = small
                .iter()
                .map(|ordinal| full.segment_of(*ordinal))
                .collect();
            crates_comparison_verdicts.absorb(report_crates_comparison(
                output,
                &format!("{name}_small"),
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
        }
        crates_comparison_verdicts.absorb(report_crates_comparison(
            output,
            &format!("{name}_quick"),
            &context,
            &quick,
            &quick_ordinals,
            &|segment_index| quick.expansion_by_segment[segment_index].crates_expansion(),
            threads,
        ));
        tally_clause_holds &= report_per_state_crates_comparison(
            output,
            &format!("{name}_segment_head"),
            &context,
            &full,
            &heads,
            threads,
        );
        let control: BTreeSet<u64> = small.iter().chain(&heads).copied().collect();
        let (control_determinism_holds, control_walk_subset_holds) = report_control_double_run(
            output,
            &format!("{name}_small_and_head"),
            &context,
            &full,
            &control,
            threads,
        );
        determinism_clause_holds &= control_determinism_holds;
        walk_subset_clause_holds &= control_walk_subset_holds;
        let mut control_data: Vec<ControlStateData> = Vec::new();
        let mut cells: Vec<(&str, &[u64], &LocalPlan<'_>, bool)> = vec![
            ("segment_head", &heads, &full, true),
            ("quick", &quick_ordinals, &quick, false),
        ];
        if !small.is_empty() {
            cells.insert(0, ("small", &small, &full, true));
        }
        for (domain, ordinals, plan, keeps_control) in cells {
            let label = stream_label(stream.name, domain);
            let control_ordinals: BTreeSet<u64> = if keeps_control {
                ordinals.iter().copied().collect()
            } else {
                BTreeSet::new()
            };
            let mut totals = evaluate_stream_domain(&StreamDomainRun {
                label: label.clone(),
                context: &context,
                plan,
                ordinals,
                control_ordinals: &control_ordinals,
                timing: TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
                threads,
                depth: EvaluationDepth::FourSteps,
            });
            let stores = std::mem::take(&mut totals.arm_stores);
            let strided: BTreeSet<usize> = BTreeSet::new();
            let summary = report_cell(
                output,
                &label,
                &totals,
                stores,
                &TrajectoryIntervals::of(plan, ordinals, &strided),
                None,
            );
            determinism_clause_holds &= summary.consult_report_differs == 0
                && summary.ignore_report_differs == 0
                && summary.verdicts_differ == 0;
            for arm in [Arm::RecoveryConsult, Arm::RecoveryIgnore, Arm::FullReadSet] {
                positive_controls.push((
                    format!("{label}_{}_constant_key_separates", arm.label()),
                    summary.arm_constant_key_inconsistent[arm.slot()] >= 1
                        || summary.arm_differing_pair[arm.slot()].is_some(),
                ));
            }
            if domain == "small" {
                let (distinct_unchanged, checks_grew_exactly, positions) =
                    refeed_every_write_persisted(
                        &context,
                        &full,
                        &totals.tables,
                        totals.whole_unit_positions,
                    );
                output.line(format!(
                    "name=positive_control_refeed cell={label} refed_state_unit_positions={positions} distinct_unit_contents_unchanged={} per_state_checks_grew_by_exactly_its_positions={} pc1_holds={}",
                    bool_text(distinct_unchanged),
                    bool_text(checks_grew_exactly),
                    bool_text(distinct_unchanged && checks_grew_exactly),
                ));
                positive_controls.push((
                    format!("{label}_pc1"),
                    distinct_unchanged && checks_grew_exactly,
                ));
            }
            if domain == "segment_head" {
                let holds = report_device_memory_control(
                    output,
                    &label,
                    &mut totals.tables,
                    &summary,
                    table.mask_width(),
                );
                positive_controls.push((format!("{label}_pc5"), holds));
            }
            control_data.extend(totals.control);
        }
        control_data.sort_by_key(|data| data.ordinal);
        for (arm, found) in report_discriminating_positions(
            output,
            &format!("{name}_small_and_head"),
            &control_data,
            threads,
        ) {
            positive_controls.push((
                format!("{name}_{}_pc2c_pc4c_found_a_position", arm.label()),
                found,
            ));
        }
    }
    let timing_control_holds = report_timing_control(output, &second, threads);
    positive_controls.push((
        format!("second_segment_{TIMING_CONTROL_SEGMENT}_pc3"),
        timing_control_holds,
    ));
    let control_fields: Vec<String> = positive_controls
        .iter()
        .map(|(name, holds)| format!("{name}={}", bool_text(*holds)))
        .collect();
    enumeration_clause_holds &= crates_comparison_verdicts.enumeration_clause_holds;
    tally_clause_holds &= crates_comparison_verdicts.tally_clause_holds;
    if crates_comparison_verdicts.judge_stage_not_run {
        output.line(
            "name=verdict scope=feasibility clause_S3_holds=not_run clause_S4_holds=not_run judge_stage=not_run"
                .to_string(),
        );
        return CRATES_COMPARISON_NOT_RUN_EXIT_CODE;
    }
    output.line(format!(
        "name=verdict scope=feasibility clause_S1_holds={} clause_S2_holds={} clause_S3_holds={} clause_S4_holds={} clause_S5_holds={} clause_S6_holds={} anchors_a1_to_a7_hold={} {} g1=not_judged_feasibility_only g2=not_judged_feasibility_only g3=not_judged_feasibility_only g4=not_judged_feasibility_only g5=not_judged_feasibility_only",
        bool_text(constants_clause_holds),
        bool_text(streams_clause_holds),
        bool_text(enumeration_clause_holds),
        bool_text(tally_clause_holds),
        bool_text(determinism_clause_holds),
        bool_text(walk_subset_clause_holds),
        bool_text(anchors_hold),
        control_fields.join(" "),
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
