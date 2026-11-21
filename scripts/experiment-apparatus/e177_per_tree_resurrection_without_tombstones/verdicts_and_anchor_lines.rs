//! 复活的判定阈值、树的状态、判定格、判定行、环锚点与汇总行。

use super::*;

// ---------------------------------------------------------------------------------------------
// 判定（第六节、修订 1 第 12.5 节）：门槛「峰值 > 0」，51 段取并、每个几何各判
// ---------------------------------------------------------------------------------------------

/// 判「复活」的门槛：r 峰值大于它（第六节）。
const RESURRECTION_THRESHOLD: u64 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeStatus {
    NoObject,
    NoDeath,
    MeasuredZero,
    ZeroWithoutPositiveControl,
    ZeroButMissedLive,
    Resurrected,
    NothingJudged,
}

impl TreeStatus {
    pub(super) const fn name(self) -> &'static str {
        match self {
            TreeStatus::NoObject => "no_object",
            TreeStatus::NoDeath => "no_death",
            TreeStatus::MeasuredZero => "measured_zero",
            TreeStatus::ZeroWithoutPositiveControl => "zero_without_positive_control",
            TreeStatus::ZeroButMissedLive => "zero_but_missed_live",
            TreeStatus::Resurrected => "resurrected",
            TreeStatus::NothingJudged => "nothing_judged",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PositiveControlTally {
    matched: u64,
    mismatched: u64,
    not_plantable: u64,
}

/// 一棵树在一组段上的状态（第六节四种之一，另加「r = 0 但漏活」「量到 0 而阳性对照没植成」「全被排除」）。
fn status_of(cells: &[&CellSummary], positive_controls: PositiveControlTally, threshold: u64) -> TreeStatus {
    let judged: Vec<&&CellSummary> = cells.iter().filter(|cell| !cell.is_excluded).collect();
    if judged.is_empty() {
        return TreeStatus::NothingJudged;
    }
    if !judged.iter().any(|cell| cell.tree_has_units) {
        return TreeStatus::NoObject;
    }
    if judged.iter().all(|cell| cell.trajectory.deaths_peak == 0) {
        return TreeStatus::NoDeath;
    }
    if judged.iter().any(|cell| cell.trajectory.resurrected_peak > threshold) {
        return TreeStatus::Resurrected;
    }
    if judged.iter().any(|cell| cell.trajectory.missed_peak > 0) {
        return TreeStatus::ZeroButMissedLive;
    }
    if positive_controls.matched > 0 && positive_controls.mismatched == 0 {
        TreeStatus::MeasuredZero
    } else {
        TreeStatus::ZeroWithoutPositiveControl
    }
}

/// 一个判定格：问题、几何、臂、读法、树，与只取 F8 e = OU 那 16 段与否（Q11 另列那一行）。
#[derive(Clone, Copy, Debug)]
struct VerdictCell {
    question: &'static str,
    pub(super) geometry: Geometry,
    pub(super) arm: Arm,
    pub(super) reader: Reader,
    pub(super) tree: ReportedTree,
    only_overwrite_before_unlink: bool,
}

fn eleven_tree_rows() -> Vec<ReportedTree> {
    REPORTED_TREES
        .iter()
        .copied()
        .filter(|tree| *tree != ReportedTree::CentralMapping)
        .collect()
}

fn verdict_cells() -> Vec<VerdictCell> {
    let mut cells = Vec::new();
    let mut add = |question: &'static str, geometry: Geometry, arms: &[Arm], readers: &[Reader], trees: &[ReportedTree], only_overwrite_before_unlink: bool| {
        for arm in arms {
            for reader in readers {
                for tree in trees {
                    cells.push(VerdictCell {
                        question,
                        geometry,
                        arm: *arm,
                        reader: *reader,
                        tree: *tree,
                        only_overwrite_before_unlink,
                    });
                }
            }
        }
    };
    let eleven = eleven_tree_rows();
    let mapping = [ReportedTree::CentralMapping];
    let both_selecting = [Reader::SelectCurrent, Reader::SelectCurrentGatedByObject];
    let without_tombstones = [Arm::WithoutTombstones];
    let todays_tombstones = [Arm::TodaysTombstones];
    let both_writing_arms = [Arm::WithoutTombstones, Arm::TodaysTombstones];
    add("Q1", Geometry::Wide, &without_tombstones, &[Reader::SelectCurrent], &eleven, false);
    add("Q2", Geometry::Wide, &without_tombstones, &[Reader::SelectCurrentGatedByObject], &eleven, false);
    add("Q4", Geometry::Wide, &todays_tombstones, &both_selecting, &eleven, false);
    add("Q6", Geometry::Wide, &both_writing_arms, &both_selecting, &mapping, false);
    add("Q8_Q1", Geometry::Tight, &without_tombstones, &[Reader::SelectCurrent], &eleven, false);
    add("Q8_Q2", Geometry::Tight, &without_tombstones, &[Reader::SelectCurrentGatedByObject], &eleven, false);
    add("Q8_Q4", Geometry::Tight, &todays_tombstones, &both_selecting, &eleven, false);
    add("Q8_Q6", Geometry::Tight, &both_writing_arms, &both_selecting, &mapping, false);
    add("Q9", Geometry::Ring, &without_tombstones, &[Reader::SelectCurrent], &eleven, false);
    add("Q10", Geometry::Ring, &without_tombstones, &[Reader::SelectCurrentGatedByObject], &eleven, false);
    add("Q11", Geometry::Ring, &todays_tombstones, &both_selecting, &eleven, false);
    add("Q11_OU", Geometry::Ring, &todays_tombstones, &both_selecting, &eleven, true);
    add("Q12", Geometry::Ring, &both_writing_arms, &both_selecting, &mapping, false);
    cells
}

fn positive_control_tally(controls: &[(Geometry, PositiveControl)], geometry: Geometry, arm: Arm, reader: Reader, tree: ReportedTree) -> PositiveControlTally {
    let mut tally = PositiveControlTally::default();
    for (control_geometry, control) in controls {
        if *control_geometry != geometry || control.arm != arm || control.reader != reader || control.tree != tree {
            continue;
        }
        match control.measured {
            Some(measured) if measured == control.expected => tally.matched += 1,
            Some(_) => tally.mismatched += 1,
            None => tally.not_plantable += 1,
        }
    }
    tally
}

/// 判定行与 Q5、Q13、Q14、F5、F7、F9、F10 的标注。
pub(super) fn verdict_lines(results: &[(SegmentResult, bool)]) -> Vec<String> {
    let all_cells: Vec<&CellSummary> = results.iter().flat_map(|(result, _)| result.cells.iter()).collect();
    let all_controls: Vec<(Geometry, PositiveControl)> = results
        .iter()
        .flat_map(|(result, _)| result.positive_controls.iter().cloned())
        .collect();
    let mut lines = Vec::new();
    let mut status_by_cell: BTreeMap<(&'static str, Arm, Reader, ReportedTree), TreeStatus> = BTreeMap::new();
    for verdict in verdict_cells() {
        let cells: Vec<&CellSummary> = all_cells
            .iter()
            .copied()
            .filter(|cell| {
                cell.geometry == verdict.geometry
                    && cell.arm == verdict.arm
                    && cell.reader == verdict.reader
                    && cell.tree == verdict.tree
                    && (!verdict.only_overwrite_before_unlink || cell.is_overwrite_before_unlink)
            })
            .collect();
        let tally = positive_control_tally(&all_controls, verdict.geometry, verdict.arm, verdict.reader, verdict.tree);
        let status = status_of(&cells, tally, RESURRECTION_THRESHOLD);
        status_by_cell.insert((verdict.question, verdict.arm, verdict.reader, verdict.tree), status);
        let judged: Vec<&&CellSummary> = cells.iter().filter(|cell| !cell.is_excluded).collect();
        let families: BTreeSet<&'static str> = judged
            .iter()
            .filter(|cell| cell.trajectory.resurrected_peak > RESURRECTION_THRESHOLD)
            .map(|cell| cell.family)
            .collect();
        let peak = |pick: fn(&Trajectory) -> u64| judged.iter().map(|cell| pick(&cell.trajectory)).max().unwrap_or(0);
        let mut line = format!(
            "name=verdict question={} geometry={} arm={} reader={} tree={} status={} r_peak={} m_peak={} d_peak={} segments_judged={} segments_excluded={} families_with_resurrection={} pc_matched={} pc_mismatched={} pc_not_plantable={}",
            verdict.question,
            verdict.geometry.name(),
            verdict.arm.name(),
            verdict.reader.name(),
            verdict.tree.name(),
            status.name(),
            peak(|trajectory| trajectory.resurrected_peak),
            peak(|trajectory| trajectory.missed_peak),
            peak(|trajectory| trajectory.deaths_peak),
            judged.len(),
            cells.len() - judged.len(),
            if families.is_empty() {
                "none".to_string()
            } else {
                families.iter().copied().collect::<Vec<_>>().join(",")
            },
            tally.matched,
            tally.mismatched,
            tally.not_plantable,
        );
        if verdict.geometry == Geometry::Ring {
            let in_window = peak(|trajectory| trajectory.in_window_peak);
            let outside = peak(|trajectory| trajectory.outside_window_peak);
            let depends_on_placement = status == TreeStatus::Resurrected && in_window == 0 && outside > 0;
            line.push_str(&format!(" r_in_peak={in_window} r_out_peak={outside} F10_depends_on_placement={depends_on_placement}"));
        }
        if families.len() == 1 {
            line.push_str(" F7_only_one_family=true");
        }
        lines.push(line);
    }
    for (first, ring_question, tight_question) in [("Q1", "Q9", "Q8_Q1"), ("Q2", "Q10", "Q8_Q2"), ("Q4", "Q11", "Q8_Q4"), ("Q6", "Q12", "Q8_Q6")] {
        for ((question, arm, reader, tree), wide_status) in &status_by_cell {
            if *question != first {
                continue;
            }
            let ring_status = status_by_cell.get(&(ring_question, *arm, *reader, *tree)).copied();
            let tight_status = status_by_cell.get(&(tight_question, *arm, *reader, *tree)).copied();
            let same = ring_status == Some(*wide_status) && tight_status == Some(*wide_status);
            lines.push(format!(
                "name=verdict question=Q13 cells={first}/{ring_question}/{tight_question} arm={} reader={} tree={} wide={} ring={} tight={} three_point_same={same} F9_flips_with_geometry={}",
                arm.name(),
                reader.name(),
                tree.name(),
                wide_status.name(),
                ring_status.map_or("missing", TreeStatus::name),
                tight_status.map_or("missing", TreeStatus::name),
                !same,
            ));
        }
    }
    for geometry in GEOMETRIES {
        for arm in ARMS {
            for reader in READERS {
                for tree in REPORTED_TREES {
                    let cells: Vec<&&CellSummary> = all_cells
                        .iter()
                        .filter(|cell| {
                            cell.geometry == geometry && cell.arm == arm && cell.reader == reader && cell.tree == tree && !cell.is_excluded
                        })
                        .collect();
                    let missed_peak = cells.iter().map(|cell| cell.trajectory.missed_peak).max().unwrap_or(0);
                    if missed_peak == 0 {
                        continue;
                    }
                    let in_unlink: u64 = cells.iter().map(|cell| cell.trajectory.missed_positive_scans_in_unlink).sum();
                    let positive: u64 = cells.iter().map(|cell| cell.trajectory.missed_positive_scans).sum();
                    lines.push(format!(
                        "name=verdict question=Q5 geometry={} arm={} reader={} tree={} m_peak={missed_peak} m_positive_scans={positive} m_positive_scans_in_unlink={in_unlink} m_positive_scans_outside_unlink={}",
                        geometry.name(),
                        arm.name(),
                        reader.name(),
                        tree.name(),
                        positive - in_unlink,
                    ));
                }
            }
        }
    }
    lines
}

// ---------------------------------------------------------------------------------------------
// H-环锚（A14、A15、A17；对不上按 S-环2 停机）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct AnchorOutcome {
    pub(super) lines: Vec<String>,
    every_anchor_holds: bool,
}

pub(super) fn run_ring_anchor() -> AnchorOutcome {
    let mut simulation = Simulation::start(
        Geometry::Ring,
        WritePath::WithoutTombstones,
        PRODUCT_UNLINK_BATCH_DATA_UNITS,
        None,
        true,
    );
    let mut lines = Vec::new();
    let mut every_anchor_holds = true;
    let mut record = |assertion: &str, detail: String, holds: bool, output_lines: &mut Vec<String>| {
        every_anchor_holds &= holds;
        output_lines.push(format!("name=anchor history=H-ring-anchor assertion={assertion} {detail} holds={holds}"));
    };
    let instance_table = &simulation.disk.units[0];
    let tree_table = &simulation.disk.units[1];
    let start_holds = simulation.version.txg == 5
        && simulation.publishes == 3
        && instance_table.slot == INSTANCE_TABLE_GENESIS_SLOT
        && instance_table.span_slots == 2
        && tree_table.slot == TREE_TABLE_GENESIS_SLOT
        && tree_table.span_slots == 1;
    record(
        "A17_start",
        format!(
            "txg_after_the_mount={} publishes_from_txg_3={} instance_table_slot={} instance_table_span={} tree_table_slot={} tree_table_span={} effective_floor=0 isolated_slots=0",
            simulation.version.txg, simulation.publishes, instance_table.slot, instance_table.span_slots, tree_table.slot, tree_table.span_slots
        ),
        start_holds,
        &mut lines,
    );
    let first_file_data_slot = simulation.placement_log[0][0].0;
    record(
        "byte_table_new_pool_file_data_unit",
        format!("txg=3 slot={first_file_data_slot}"),
        first_file_data_slot == UNIT_AREA_START_SLOT + 4,
        &mut lines,
    );
    for _ in 0..RING_ANCHOR_OVERWRITES {
        simulation.apply(UserChange::Overwrite);
        let txg = simulation.version.txg;
        if txg == 25 {
            let is_free = simulation.allocator.is_free(TREE_TABLE_GENESIS_SLOT);
            record("A14_i", format!("txg=25 slot={TREE_TABLE_GENESIS_SLOT} free={is_free}"), !is_free, &mut lines);
        }
        if txg == 26 {
            let is_free = simulation.allocator.is_free(TREE_TABLE_GENESIS_SLOT);
            let reclaimed = simulation.slots_reclaimed_by_publish.get(&26).copied().unwrap_or(0);
            record(
                "A14_ii",
                format!("txg=26 slot={TREE_TABLE_GENESIS_SLOT} free={is_free} slots_reclaimed={reclaimed}"),
                is_free && reclaimed == 1,
                &mut lines,
            );
        }
        if txg == 27 {
            let data_slot = simulation.placement_log.last().expect("txg 27 发过")[0].0;
            record("A14_iii", format!("txg=27 data_unit_slot={data_slot}"), data_slot == TREE_TABLE_GENESIS_SLOT, &mut lines);
        }
    }
    let equality = simulation
        .reuse_at_exactly_the_ring_length
        .contains(&(27, TREE_TABLE_GENESIS_SLOT));
    record(
        "A15_equality",
        format!("txg=27 slot={TREE_TABLE_GENESIS_SLOT} released_at=3 ring_length={RING_LENGTH_IN_ROOTS_FOR_THE_REUSE_ASSERTION}"),
        equality,
        &mut lines,
    );
    let inequality_violations = simulation.assertions.violations_of("A15_reuse_waits_a_ring_length");
    record("A15_inequality", format!("violations={inequality_violations}"), inequality_violations == 0, &mut lines);
    let incremental_violations = simulation.assertions.violations_of("unit_scan_incremental_equals_brute_force");
    record("unit_scan_cross_check_every_scan", format!("violations={incremental_violations}"), incremental_violations == 0, &mut lines);
    AnchorOutcome {
        lines,
        every_anchor_holds,
    }
}

// ---------------------------------------------------------------------------------------------
// 作废与停机的总判定（第十一节、12.10）与主程序
// ---------------------------------------------------------------------------------------------

/// 7.2 类断言与别的作废条件：任一条有违反就整轮作废（7.1 类 K 不符走 F1，不作废）。
const VOIDING_ASSERTIONS: [&str; 14] = [
    "A2_accounting_rows_per_version",
    "A3_unit_scan_on_wide_reads_every_death",
    "A4_death_records_zero",
    "A5_A10_unlink_publishes",
    "A6_accounting_deaths_at_the_end",
    "A7_inode_deaths_at_the_end",
    "A11_extent_deaths_at_the_end",
    "A12_overwrite_kills_all_but_one_extent_key",
    "A13_death_records_zero_on_ring",
    "A16_ring_equals_wide_before_the_first_reuse",
    "A18_user_data_outside_cluster_segments",
    "A19_readable_units_equal_written_minus_overwritten",
    "positive_control_matches",
    "scans_equal_publishes",
];

const FURTHER_VOIDING_ASSERTIONS: [&str; 7] = [
    "A7_deleted_inodes_deaths_at_the_end",
    "A12_overwrite_releases_every_old_data_unit",
    "A15_reuse_waits_a_ring_length",
    "A18_opened_segment_was_empty",
    "unit_scan_incremental_equals_brute_force",
    "kinds_6_7_8_11_12_have_no_units",
    "ring_reuses_at_least_once_in_a_long_history",
];

const CLAUSE_ASSERTIONS: [&str; 5] = [
    "K1_tombstone_containers",
    "K3_tombstone_death_txg_is_the_intent_txg",
    "K4_containers_after_building",
    "K4_container_number_is_its_first_record",
    "K4_emptied_container_not_written",
];

pub(super) fn summary_lines(results: &[(SegmentResult, bool)], anchor: &AnchorOutcome) -> Vec<String> {
    let mut total = AssertionTally::default();
    for (result, _) in results {
        total.merge(&result.assertions);
    }
    let mut lines = Vec::new();
    for name in VOIDING_ASSERTIONS.iter().chain(&FURTHER_VOIDING_ASSERTIONS).chain(&CLAUSE_ASSERTIONS) {
        lines.push(format!(
            "name=assertion_total assertion={name} judged={} violated={}",
            total.judged.get(name).copied().unwrap_or(0),
            total.violations_of(name)
        ));
    }
    let voiding_violations: u64 = VOIDING_ASSERTIONS
        .iter()
        .chain(&FURTHER_VOIDING_ASSERTIONS)
        .map(|name| total.violations_of(name))
        .sum();
    let clause_violations: u64 = CLAUSE_ASSERTIONS.iter().map(|name| total.violations_of(name)).sum();
    let nondeterministic = results.iter().filter(|(_, identical)| !identical).count();
    let eleven_kinds_match = ELEVEN_TREE_KINDS.len() == 11
        && ELEVEN_TREE_KINDS
            .iter()
            .map(|kind| kind.kind_code())
            .collect::<BTreeSet<u16>>()
            == [1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12].into_iter().collect::<BTreeSet<u16>>()
        && length_as_u64(ELEVEN_TREE_KINDS.len()) == TREE_TABLE_ENTRIES;
    lines.push(format!(
        "name=verdict scope=voiding voiding_assertion_violations={voiding_violations} nondeterministic_segments={nondeterministic} A1_K5_eleven_tree_kinds={eleven_kinds_match} void={}",
        voiding_violations > 0 || nondeterministic > 0 || !eleven_kinds_match
    ));
    lines.push(format!(
        "name=verdict scope=clause K_violations={clause_violations} F1_clause_may_be_wrong={}",
        clause_violations > 0
    ));
    lines.push(format!(
        "name=verdict scope=stop S_ring_2_anchors_hold={} stop={}",
        anchor.every_anchor_holds, !anchor.every_anchor_holds
    ));
    lines
}

#[cfg(test)]
#[path = "verdicts_and_anchor_lines_tests.rs"]
mod tests;
