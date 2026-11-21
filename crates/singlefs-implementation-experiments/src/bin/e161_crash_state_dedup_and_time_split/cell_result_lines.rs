//! 一格的结果行（G1–G5、S5）。

use super::*;

// ───────────────────────────── 一格的结果行（G1–G5、S5） ─────────────────────────────

/// 一格报完之后留给判定、几何敏感性与判别力自证用的数。
#[derive(Clone, Debug)]
pub(super) struct CellSummary {
    pub(super) label: String,
    pub(super) states: u64,
    pub(super) distinct_unit_contents: u64,
    pub(super) per_state_unit_checks: u64,
    pub(super) arm_distinct_keys: [u64; 6],
    pub(super) arm_inconsistent: [u64; 6],
    pub(super) arm_hits: [u64; 6],
    pub(super) arm_constant_key_inconsistent: [u64; 6],
    arm_distinct_outcomes: [u64; 6],
    pub(super) arm_differing_pair: [Option<(u64, u64)>; 6],
    pub(super) arm_distinct_key_elements: [u64; 6],
    pub(super) state_nanoseconds: u128,
    pub(super) zone_nanoseconds: [u128; 6],
    pub(super) unit_events_replay_nanoseconds: u128,
    pub(super) walk_checker_nanoseconds: u128,
    pub(super) content_bytes_by_category: [u128; 4],
    unit_contents_seen: BTreeSet<Digest128>,
    pub(super) consult_report_differs: u64,
    pub(super) ignore_report_differs: u64,
    pub(super) verdicts_differ: u64,
    pub(super) walk_reads_outside_normal_reads: u64,
    pub(super) content_collisions: u64,
    pub(super) states_shorter_than_their_segments: u64,
    pub(super) states_replaying_longer_than_the_checker: u64,
    /// 结果行里不含计时的部分（V6 逐字比）。
    pub(super) lines_without_timing: Vec<String>,
}

impl CellSummary {
    fn empty(label: &str, states: u64) -> Self {
        Self {
            label: label.to_string(),
            states,
            distinct_unit_contents: 0,
            per_state_unit_checks: 0,
            arm_distinct_keys: [0; 6],
            arm_inconsistent: [0; 6],
            arm_hits: [0; 6],
            arm_constant_key_inconsistent: [0; 6],
            arm_distinct_outcomes: [0; 6],
            arm_differing_pair: [None; 6],
            arm_distinct_key_elements: [0; 6],
            state_nanoseconds: 0,
            zone_nanoseconds: [0; 6],
            unit_events_replay_nanoseconds: 0,
            walk_checker_nanoseconds: 0,
            content_bytes_by_category: [0; 4],
            unit_contents_seen: BTreeSet::new(),
            consult_report_differs: 0,
            ignore_report_differs: 0,
            verdicts_differ: 0,
            walk_reads_outside_normal_reads: 0,
            content_collisions: 0,
            states_shorter_than_their_segments: 0,
            states_replaying_longer_than_the_checker: 0,
            lines_without_timing: Vec::new(),
        }
    }
}

/// 结果行去掉计时字段（名字以 `_ns`、`_ratio_of_state` 结尾的，与线程数、机器负载有关）。
fn without_timing_fields(line: &str) -> String {
    line.split(' ')
        .filter(|field| {
            let name = field.split('=').next().unwrap_or("");
            !(name.ends_with("_elapsed_ns")
                || name.ends_with("_ratio")
                || name.starts_with("threads"))
        })
        .collect::<Vec<&str>>()
        .join(" ")
}

/// 一格的全部结果行。`stores` 是这一格的六条臂的记录（报完即耗掉）；`union_stores` 给了就把每条记录也抄进去（D1 并格用）。
pub(super) fn report_cell(
    output: &mut ResultLines,
    label: &str,
    totals: &StreamDomainTotals,
    stores: Vec<ArmRecordStore>,
    intervals: &TrajectoryIntervals,
    union_stores: Option<&mut Vec<ArmRecordStore>>,
) -> CellSummary {
    let mut summary = CellSummary::empty(label, totals.states());
    let emit =
        |destination: &mut ResultLines, lines_without_timing: &mut Vec<String>, line: String| {
            lines_without_timing.push(without_timing_fields(&line));
            destination.line(line);
        };
    let mut union_stores = union_stores;
    let mut arm_counts = Vec::new();
    for (slot, store) in stores.into_iter().enumerate() {
        let target = union_stores.as_deref_mut().map(|union| &mut union[slot]);
        arm_counts.push(count_arm(store, intervals, target));
    }
    for (slot, count) in arm_counts.iter().enumerate() {
        summary.arm_distinct_keys[slot] = count.distinct_keys;
        summary.arm_inconsistent[slot] = count.inconsistent;
        summary.arm_hits[slot] = count.hits;
        summary.arm_constant_key_inconsistent[slot] = count.constant_key_inconsistent;
        summary.arm_distinct_outcomes[slot] = count.distinct_outcomes;
        summary.arm_differing_pair[slot] = count.differing_pair;
        summary.arm_distinct_key_elements[slot] = count.distinct_key_elements;
    }
    // G1（Q1a–Q1f）与内容表。
    let unit_table = &totals.tables.by_category[ReadCategory::WholeUnit.slot()];
    let mut unit_trajectory = Trajectory::new(intervals);
    for (digest, entry) in &unit_table.entries {
        if let Some(first) = entry.first_ordinal_checker_normal_pass {
            summary.distinct_unit_contents += 1;
            summary.unit_contents_seen.insert(*digest);
            unit_trajectory.add(intervals.interval_of(first), 1);
        }
    }
    summary.per_state_unit_checks = totals.whole_unit_positions;
    let checker_only_distinct = |category: ReadCategory| {
        totals.tables.by_category[category.slot()]
            .entries
            .values()
            .filter(|entry| entry.first_ordinal_checker_normal_pass.is_some())
            .count()
    };
    let header_distinct = checker_only_distinct(ReadCategory::UnitHeaderScan);
    let ring_distinct = checker_only_distinct(ReadCategory::JournalRing);
    summary.content_collisions = totals.tables.collisions();
    emit(output, &mut summary.lines_without_timing, format!(
        "name=g1 cell={label} states={} distinct_unit_contents={} per_state_unit_checks={} r1={} r1_at_or_above_half={} unit_read_events={} states_without_unit_reads={} header_scan_events={} header_scan_per_state_positions={} header_scan_distinct_contents={header_distinct} ring_record_events={} ring_record_per_state_positions={} ring_record_distinct_contents={ring_distinct} ring_record_r={} content_collisions={} {}",
        summary.states,
        summary.distinct_unit_contents,
        summary.per_state_unit_checks,
        ratio_text(u128::from(summary.distinct_unit_contents), u128::from(summary.per_state_unit_checks)),
        reaches_half(u128::from(summary.distinct_unit_contents), u128::from(summary.per_state_unit_checks)),
        totals.whole_unit_events,
        totals.states_without_whole_unit_reads,
        totals.header_scan_events,
        totals.header_scan_positions,
        totals.ring_record_events,
        totals.ring_record_positions,
        ratio_text(u128::try_from(ring_distinct).expect("个数"), u128::from(totals.ring_record_positions)),
        summary.content_collisions,
        unit_trajectory.fields("q1a"),
    ));
    // G2（两遍各一套 + 成对键）与 G4（三条臂各一套）。
    for arm in ARMS {
        let count = &arm_counts[arm.slot()];
        emit(output, &mut summary.lines_without_timing, format!(
            "name=reuse_arm cell={label} arm={} states={} inconsistent={} hits={} distinct_keys={} keys_over_states={} keys_at_or_above_half={} constant_key_inconsistent={} distinct_outcomes={} {} {}",
            arm.label(),
            count.states,
            count.inconsistent,
            count.hits,
            count.distinct_keys,
            ratio_text(u128::from(count.distinct_keys), u128::from(count.states)),
            reaches_half(u128::from(count.distinct_keys), u128::from(count.states)),
            count.constant_key_inconsistent,
            count.distinct_outcomes,
            count.new_keys.fields("new_keys"),
            count.inconsistencies.fields("inconsistent"),
        ));
    }
    // S5 与 Q4e（S6）。
    summary.consult_report_differs = totals.consult_report_differs;
    summary.ignore_report_differs = totals.ignore_report_differs;
    summary.verdicts_differ = totals.verdicts_differ;
    summary.walk_reads_outside_normal_reads = totals.walk_reads_outside_normal_reads;
    emit(output, &mut summary.lines_without_timing, format!(
        "name=determinism cell={label} states={} recorded_consult_report_differs={} recorded_ignore_report_differs={} recorded_verdicts_differ={} walk_unit_reads_outside_normal_unit_reads={}",
        summary.states,
        totals.consult_report_differs,
        totals.ignore_report_differs,
        totals.verdicts_differ,
        totals.walk_reads_outside_normal_reads,
    ));
    // P1 四类与它的轨迹。
    let mut content_trajectory = Trajectory::new(intervals);
    let mut category_fields = Vec::new();
    for category in READ_CATEGORIES {
        let table = &totals.tables.by_category[category.slot()];
        let mut bytes = 0u128;
        let mut distinct = 0u64;
        for entry in table.entries.values() {
            if let Some(first) = entry.first_ordinal_any_pass {
                let length = u128::try_from(entry.bytes.len()).expect("长度");
                bytes += length;
                distinct += 1;
                content_trajectory.add(intervals.interval_of(first), length);
            }
        }
        summary.content_bytes_by_category[category.slot()] = bytes;
        category_fields.push(format!(
            "{}_bytes={bytes} {}_distinct={distinct}",
            category.label(),
            category.label()
        ));
    }
    emit(
        output,
        &mut summary.lines_without_timing,
        format!(
            "name=g5_contents cell={label} states={} {} p1_bytes={} {}",
            summary.states,
            category_fields.join(" "),
            summary.content_bytes_by_category.iter().sum::<u128>(),
            content_trajectory.fields("p1"),
        ),
    );
    // G3 这一格的总时长与占比（未加权）。
    let mut sums = TimingSums::default();
    for segment_sums in totals.timing_by_segment.values() {
        sums.absorb(segment_sums);
    }
    summary.state_nanoseconds = sums.state_nanoseconds;
    summary.zone_nanoseconds = sums.zone_nanoseconds;
    summary.unit_events_replay_nanoseconds = sums.unit_events_replay_nanoseconds;
    summary.walk_checker_nanoseconds = sums.walk_checker_nanoseconds;
    summary.states_shorter_than_their_segments = sums.states_shorter_than_their_segments;
    summary.states_replaying_longer_than_the_checker =
        sums.states_replaying_longer_than_the_checker;
    output.line(timing_line(label, &sums));
    summary
}

/// G3 的一行（全部是计时字段，不进 V6 的逐字比）：五段与单元级重放的总时长与占 ΣT_state 的比、Q3g、Q3h、Q3i、Q3j、V3 的两个数。
fn timing_line(label: &str, sums: &TimingSums) -> String {
    let share = |nanoseconds: u128| ratio_text(nanoseconds, sums.state_nanoseconds);
    let zone_fields: Vec<String> = TIMING_ZONES
        .iter()
        .enumerate()
        .map(|(slot, zone)| {
            format!(
                "{}_elapsed_ns={} {}_share_ratio={}",
                zone.label(),
                sums.zone_nanoseconds[slot],
                zone.label(),
                share(sums.zone_nanoseconds[slot])
            )
        })
        .collect();
    let checker = sums.zone_nanoseconds[3];
    let unit_positions = sums.zone_nanoseconds[5];
    format!(
        "name=g3 cell={label} states={} state_elapsed_ns={} {} unit_events_replay_elapsed_ns={} unit_events_share_ratio={} walk_and_judge_lower_elapsed_ns={} walk_and_judge_upper_elapsed_ns={} walk_checker_elapsed_ns={} walk_checker_of_checker_share_ratio={} state_shorter_than_its_segments_share_ratio={} replaying_longer_than_the_checker_share_ratio={}",
        sums.states,
        sums.state_nanoseconds,
        zone_fields.join(" "),
        sums.unit_events_replay_nanoseconds,
        share(sums.unit_events_replay_nanoseconds),
        checker.saturating_sub(sums.unit_events_replay_nanoseconds),
        checker.saturating_sub(unit_positions),
        sums.walk_checker_nanoseconds,
        ratio_text(sums.walk_checker_nanoseconds, checker),
        ratio_text(u128::from(sums.states_shorter_than_their_segments), u128::from(sums.states)),
        ratio_text(u128::from(sums.states_replaying_longer_than_the_checker), u128::from(sums.states)),
    )
}

#[cfg(test)]
mod tests;
