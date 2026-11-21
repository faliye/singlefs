//! 模式 segment-one：一条流上的 D1 与 D1-quick。

use super::*;

// ───────────────────────────── 模式 segment-one：一条流上的 D1 与 D1-quick ─────────────────────────────

/// 一条流跑完第一段留下的东西。
pub(super) struct StreamOutcome {
    pub(super) name: StreamName,
    pub(super) write_table_length: usize,
    pub(super) small: CellSummary,
    pub(super) stride: CellSummary,
    pub(super) union: CellSummary,
    pub(super) quick: CellSummary,
    pub(super) weighted: UnitShareEstimate,
    pub(super) pc1: Option<(bool, bool, u64)>,
}

pub(super) fn stream_label(stream: StreamName, domain: &str) -> String {
    format!("{}_{domain}", stream.label())
}

/// 报出 PC2c / PC4c 的一行。
pub(super) fn report_discriminating_positions(
    output: &mut ResultLines,
    label: &str,
    control: &[ControlStateData],
    threads: usize,
) -> Vec<(Arm, bool)> {
    let mut found = Vec::new();
    for arm in [
        Arm::RecoveryConsult,
        Arm::RecoveryIgnore,
        Arm::WalkUnits,
        Arm::AllUnits,
        Arm::FullReadSet,
    ] {
        let result = discriminating_positions(control, arm, threads);
        let first_ten: Vec<String> = result
            .discriminating
            .iter()
            .take(10)
            .map(PositionDescription::text)
            .collect();
        let has_one = !result.discriminating.is_empty();
        output.line(format!(
            "name=positive_control_remove_one_position cell={label} arm={} control_states={} positions={} discriminating_positions={} first_ten=[{}] inconsistent_without_removal={} found_at_least_one={}",
            arm.label(),
            control.len(),
            result.positions,
            result.discriminating.len(),
            first_ten.join(","),
            result.baseline_inconsistent,
            bool_text(has_one),
        ));
        found.push((arm, has_one));
    }
    found
}

#[allow(
    clippy::too_many_lines,
    reason = "一条流上第一段的全部格按登记 5.5 的次序排下来：D1-small、D1-stride、并格、加权 G3、G5、D1-quick、PC1、PC2c / PC4c"
)]
pub(super) fn process_stream(
    output: &mut ResultLines,
    stream: &PreparedStream,
    threads: usize,
) -> (StreamOutcome, Vec<(Arm, bool)>) {
    let table = TornWriteTable::of(stream);
    let context = EvaluationContext::new(stream, &table);
    let full = LocalPlan::new(
        stream,
        &table,
        domain_expansions(stream, LocalExpansion::EveryCombination),
    );
    let small = small_domain_ordinals(stream, &full);
    let stride = stride_domain_ordinals(stream, &full);
    let mut domain: Vec<u64> = small.iter().chain(&stride).copied().collect();
    domain.sort_unstable();
    let control = control_ordinals(stream, &full, &domain);
    let strided_segments: BTreeSet<usize> = stride
        .iter()
        .map(|ordinal| full.segment_of(*ordinal))
        .collect();
    let run =
        |label: &str, ordinals: &[u64], control_ordinals: &BTreeSet<u64>, plan: &LocalPlan<'_>| {
            evaluate_stream_domain(&StreamDomainRun {
                label: label.to_string(),
                context: &context,
                plan,
                ordinals,
                control_ordinals,
                timing: TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
                threads,
                depth: EvaluationDepth::FourSteps,
            })
        };
    let mut small_totals = run(&stream_label(stream.name, "small"), &small, &control, &full);
    let mut stride_totals = run(
        &stream_label(stream.name, "stride"),
        &stride,
        &control,
        &full,
    );
    let mut union_stores: Vec<ArmRecordStore> = ARMS
        .iter()
        .map(|_arm| ArmRecordStore::new(ARM_RECORDS_IN_MEMORY_BEFORE_SPILL))
        .collect();
    let small_stores = std::mem::take(&mut small_totals.arm_stores);
    let small_summary = report_cell(
        output,
        &stream_label(stream.name, "small"),
        &small_totals,
        small_stores,
        &TrajectoryIntervals::of(&full, &small, &strided_segments),
        Some(&mut union_stores),
    );
    let stride_stores = std::mem::take(&mut stride_totals.arm_stores);
    let stride_summary = report_cell(
        output,
        &stream_label(stream.name, "stride"),
        &stride_totals,
        stride_stores,
        &TrajectoryIntervals::of(&full, &stride, &strided_segments),
        Some(&mut union_stores),
    );
    let mut union_totals = small_totals;
    union_totals.absorb(stride_totals);
    let union_summary = report_cell(
        output,
        &stream_label(stream.name, "d1"),
        &union_totals,
        union_stores,
        &TrajectoryIntervals::of(&full, &domain, &strided_segments),
        None,
    );
    let weighted = weighted_unit_share(&union_totals.timing_by_segment, &full);
    output.line(format!(
        "name=g3_weighted stream={} unit_share_lower_ratio={:.6} unit_share_upper_ratio={:.6}",
        stream.name.label(),
        weighted.lower,
        weighted.upper,
    ));
    for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
        let _verdicts = report_device_memory(
            output,
            &union_summary,
            table.mask_width(),
            SECOND_STREAM_WRITES + ANCHOR_SECOND_STREAM_IN_PLACE_OVERWRITES,
            arm,
        );
    }
    let _device_memory_control_holds = report_device_memory_control(
        output,
        &stream_label(stream.name, "d1"),
        &mut union_totals.tables,
        &union_summary,
        table.mask_width(),
    );
    let quick_plan = LocalPlan::new(
        stream,
        &table,
        domain_expansions(
            stream,
            LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
        ),
    );
    let quick_ordinals = every_ordinal(stream, &quick_plan);
    let mut quick_totals = run(
        &stream_label(stream.name, "quick"),
        &quick_ordinals,
        &BTreeSet::new(),
        &quick_plan,
    );
    let quick_stores = std::mem::take(&mut quick_totals.arm_stores);
    let quick_summary = report_cell(
        output,
        &stream_label(stream.name, "quick"),
        &quick_totals,
        quick_stores,
        &TrajectoryIntervals::of(&quick_plan, &quick_ordinals, &BTreeSet::new()),
        None,
    );
    for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
        let _verdicts = report_device_memory(
            output,
            &quick_summary,
            table.mask_width(),
            SECOND_STREAM_WRITES + ANCHOR_SECOND_STREAM_IN_PLACE_OVERWRITES,
            arm,
        );
    }
    let discriminating = report_discriminating_positions(
        output,
        &stream_label(stream.name, "d1_control"),
        &union_totals.control,
        threads,
    );
    let pc1 = includes_every_write_persisted(stream.name).then(|| {
        refeed_every_write_persisted(
            &context,
            &full,
            &union_totals.tables,
            union_totals.whole_unit_positions,
        )
    });
    if let Some((distinct_unchanged, checks_grew_exactly, positions)) = pc1 {
        output.line(format!(
            "name=positive_control_refeed cell={} refed_state_unit_positions={positions} distinct_unit_contents_unchanged={} per_state_checks_grew_by_exactly_its_positions={} pc1_holds={}",
            stream_label(stream.name, "d1"),
            bool_text(distinct_unchanged),
            bool_text(checks_grew_exactly),
            bool_text(distinct_unchanged && checks_grew_exactly),
        ));
    }
    let outcome = StreamOutcome {
        name: stream.name,
        write_table_length: table.mask_width(),
        small: small_summary,
        stride: stride_summary,
        union: union_summary,
        quick: quick_summary,
        weighted,
        pc1,
    };
    (outcome, discriminating)
}

/// PC1：新池新建文件流全部持久那个状态当一个额外状态再喂一遍（新的序号），数去重臂的不同内容数变没变、逐状态臂加了多少。
/// 交回（不同内容数没变，逐状态次数恰好加上它的不同单元位置数，它的不同单元位置数）。
pub(super) fn refeed_every_write_persisted(
    context: &EvaluationContext<'_>,
    plan: &LocalPlan<'_>,
    union_tables: &ContentTables,
    per_state_checks_before: u64,
) -> (bool, bool, u64) {
    let refed_ordinal = u64::MAX - 1;
    let tables = RefCell::new(ContentTables::default());
    let mut tally = CrashPointReplayTally::default();
    let result = evaluate_state(
        context,
        refed_ordinal,
        context.stream.segments.len(),
        plan.persisted_of(plan.expanded_state_count),
        TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
        false,
        &tables,
        &mut tally,
    );
    let checker_contents = |table: &ContentTable| -> BTreeSet<Digest128> {
        table
            .entries
            .iter()
            .filter(|(_, entry)| entry.first_ordinal_checker_normal_pass.is_some())
            .map(|(digest, _)| *digest)
            .collect()
    };
    let before = checker_contents(&union_tables.by_category[ReadCategory::WholeUnit.slot()]);
    let refed = checker_contents(&tables.into_inner().by_category[ReadCategory::WholeUnit.slot()]);
    let after: BTreeSet<Digest128> = before.union(&refed).copied().collect();
    let positions = result.whole_unit_positions;
    let mut extra = StreamDomainTotals::new();
    extra.absorb_state(result);
    let per_state_checks_after = per_state_checks_before + extra.whole_unit_positions;
    (
        after.len() == before.len(),
        per_state_checks_after == per_state_checks_before + positions && positions > 0,
        positions,
    )
}

#[cfg(test)]
mod tests;
