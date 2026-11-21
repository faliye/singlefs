//! 一份历史的规格与随机数、删除次序、历史的整趟、汇总与按段并行。

use super::*;

/// 阳性对照的增量；读数反而变小（不该发生）时交回 `u64::MAX`，让它与任何期望都对不上。
pub(super) fn increase_or_marker(after: u64, before: u64) -> u64 {
    after.checked_sub(before).unwrap_or(u64::MAX)
}

// ---------------------------------------------------------------------------------------------
// 历史族（原文 5.2、修订 1 第 12.4 节）
// ---------------------------------------------------------------------------------------------

/// 带种子的确定性随机源（splitmix64）。
#[derive(Clone, Debug)]
struct SeededRandom {
    state: u64,
}

impl SeededRandom {
    pub(super) fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next_value() % bound
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeletionOrder {
    Ascending,
    EvenThenOdd,
    Random(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HistorySpecification {
    /// F1 建删空 inode：n 个 inode（含 inode 1），逐个 unlink inode 2..n。
    BuildAndDeleteEmptyInodes { inode_count: u64, order: DeletionOrder },
    /// F8 大文件改写删（12.4.2）。
    LargeFileRewriteAndDelete {
        data_units: u64,
        rewrites: u64,
        overwrite_before_unlink: bool,
        batch: u64,
    },
    /// F3 混合随机。
    MixedRandom { seed: u64 },
}

/// F3 的步数（原文 5.2）。
const MIXED_RANDOM_STEPS: u64 = 300;
/// H-环锚 的覆盖写次数（12.4.4）。
pub(super) const RING_ANCHOR_OVERWRITES: u64 = 22;
/// F1 每次建 50 个空 inode（原文 5.2、12.4.3）。
const NEW_INODES_PER_STEP: u64 = 50;

impl HistorySpecification {
    pub(super) const fn family(self) -> &'static str {
        match self {
            HistorySpecification::BuildAndDeleteEmptyInodes { .. } => "F1",
            HistorySpecification::LargeFileRewriteAndDelete { .. } => "F8",
            HistorySpecification::MixedRandom { .. } => "F3",
        }
    }

    pub(super) fn label(self) -> String {
        match self {
            HistorySpecification::BuildAndDeleteEmptyInodes { inode_count, order } => {
                let order_name = match order {
                    DeletionOrder::Ascending => "ascending".to_string(),
                    DeletionOrder::EvenThenOdd => "even_then_odd".to_string(),
                    DeletionOrder::Random(seed) => format!("random_seed_{seed}"),
                };
                format!("family=F1 n={inode_count} order={order_name}")
            }
            HistorySpecification::LargeFileRewriteAndDelete {
                data_units,
                rewrites,
                overwrite_before_unlink,
                batch,
            } => format!(
                "family=F8 n_u={data_units} k={rewrites} e={} batch={batch}",
                if overwrite_before_unlink { "OU" } else { "U" }
            ),
            HistorySpecification::MixedRandom { seed } => format!("family=F3 seed={seed}"),
        }
    }

    const fn batch(self) -> u64 {
        match self {
            HistorySpecification::LargeFileRewriteAndDelete { batch, .. } => batch,
            HistorySpecification::BuildAndDeleteEmptyInodes { .. } | HistorySpecification::MixedRandom { .. } => {
                PRODUCT_UNLINK_BATCH_DATA_UNITS
            }
        }
    }

    pub(super) const fn is_overwrite_before_unlink(self) -> bool {
        matches!(
            self,
            HistorySpecification::LargeFileRewriteAndDelete {
                overwrite_before_unlink: true,
                ..
            }
        )
    }
}

/// 第一段的 51 段：F1 14 段、F8 32 段、F3 5 段（12.5）。
pub(super) fn segment_one_histories() -> Vec<HistorySpecification> {
    let mut histories = Vec::new();
    for inode_count in [300, 900] {
        let mut orders = vec![DeletionOrder::Ascending, DeletionOrder::EvenThenOdd];
        orders.extend((1..=5).map(DeletionOrder::Random));
        for order in orders {
            histories.push(HistorySpecification::BuildAndDeleteEmptyInodes { inode_count, order });
        }
    }
    for data_units in [1, 3, 145, 300] {
        for rewrites in [0, 5] {
            for overwrite_before_unlink in [false, true] {
                for batch in [1, PRODUCT_UNLINK_BATCH_DATA_UNITS] {
                    histories.push(HistorySpecification::LargeFileRewriteAndDelete {
                        data_units,
                        rewrites,
                        overwrite_before_unlink,
                        batch,
                    });
                }
            }
        }
    }
    for seed in 1..=5 {
        histories.push(HistorySpecification::MixedRandom { seed });
    }
    histories
}

fn live_inodes_of(version: &Version) -> Vec<u64> {
    version
        .inode_containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|record| record.nlink > 0)
        .map(|record| record.inode)
        .collect()
}

fn deletion_order_of(inode_count: u64, order: DeletionOrder) -> Vec<u64> {
    let inodes: Vec<u64> = (FIRST_FILE_INODE + 1..=inode_count).collect();
    match order {
        DeletionOrder::Ascending => inodes,
        DeletionOrder::EvenThenOdd => inodes
            .iter()
            .copied()
            .filter(|inode| inode % 2 == 0)
            .chain(inodes.iter().copied().filter(|inode| inode % 2 == 1))
            .collect(),
        DeletionOrder::Random(seed) => {
            let mut shuffled = inodes;
            let mut random = SeededRandom::new(seed);
            for position in (1..shuffled.len()).rev() {
                let other = u64_as_index(random.below(length_as_u64(position) + 1));
                shuffled.swap(position, other);
            }
            shuffled
        }
    }
}

/// 照历史驱动一段模拟；顺带判只在某个族上能判的断言（K4、A5、A10、A12）。
fn run_history(specification: HistorySpecification, simulation: &mut Simulation) {
    match specification {
        HistorySpecification::BuildAndDeleteEmptyInodes { inode_count, order } => {
            let mut created = 1;
            while created < inode_count {
                let count = NEW_INODES_PER_STEP.min(inode_count - created);
                simulation.apply(UserChange::NewInodes { count });
                created += count;
            }
            let containers = &simulation.version.inode_containers;
            simulation.assertions.judge(
                "K4_containers_after_building",
                length_as_u64(containers.len()) == inode_count.div_ceil(INODE_LEAF_RECORDS),
            );
            let numbers_are_first_records = containers
                .iter()
                .all(|container| container.records.first().is_some_and(|first| first.inode == container.container_number));
            simulation.assertions.judge("K4_container_number_is_its_first_record", numbers_are_first_records);
            for inode in deletion_order_of(inode_count, order) {
                let emptied_container = simulation
                    .version
                    .inode_containers
                    .iter()
                    .find(|container| container.records.len() == 1 && container.records[0].inode == inode)
                    .map(|container| container.container_number);
                simulation.apply(UserChange::Unlink { inode });
                if let Some(number) = emptied_container {
                    let still_written = simulation
                        .version
                        .inode_containers
                        .iter()
                        .any(|container| container.container_number == number);
                    simulation.assertions.judge("K4_emptied_container_not_written", !still_written);
                }
            }
        }
        HistorySpecification::LargeFileRewriteAndDelete {
            data_units,
            rewrites,
            overwrite_before_unlink,
            ..
        } => {
            for _ in 0..=rewrites {
                simulation.apply(UserChange::SequentialWrite { data_units });
            }
            if overwrite_before_unlink {
                let extent_position = Namespace::Extent.position();
                let deaths_before = simulation.ever_seen[extent_position].len() - simulation.truth[extent_position].len();
                simulation.apply(UserChange::Overwrite);
                let txg = simulation.version.txg;
                let deaths_after = simulation.ever_seen[extent_position].len() - simulation.truth[extent_position].len();
                simulation.assertions.judge(
                    "A12_overwrite_kills_all_but_one_extent_key",
                    length_as_u64(deaths_after) == length_as_u64(deaths_before) + data_units - 1,
                );
                let released_data_units = simulation
                    .disk
                    .units
                    .iter()
                    .filter(|unit| unit.owner == UnitOwner::FileData && unit.release_txg == Some(txg))
                    .count();
                simulation
                    .assertions
                    .judge("A12_overwrite_releases_every_old_data_unit", length_as_u64(released_data_units) == data_units);
            }
            simulation.apply(UserChange::Unlink { inode: FIRST_FILE_INODE });
        }
        HistorySpecification::MixedRandom { seed } => {
            let mut random = SeededRandom::new(seed);
            for _ in 0..MIXED_RANDOM_STEPS {
                let draw = random.below(10);
                let change = if draw < 3 {
                    UserChange::NewInodes {
                        count: 1 + random.below(20),
                    }
                } else if draw < 5 {
                    UserChange::Overwrite
                } else if draw < 6 {
                    UserChange::SequentialWrite {
                        data_units: 1 + random.below(200),
                    }
                } else {
                    let live = live_inodes_of(&simulation.version);
                    if live.is_empty() {
                        simulation.refused_steps += 1;
                        continue;
                    }
                    UserChange::Unlink {
                        inode: live[u64_as_index(random.below(length_as_u64(live.len())))],
                    }
                };
                simulation.apply(change);
            }
        }
    }
    for count in &simulation.unlink_publish_counts {
        let expected = 1 + count.data_units_at_the_intent.div_ceil(simulation.batch_data_units).max(1);
        simulation.assertions.judge("A5_A10_unlink_publishes", count.publishes == expected);
        let unlinked_inode_is_gone = !simulation.version.has_live_inode(count.inode);
        simulation.assertions.judge("unlinked_inode_stays_dead", unlinked_inode_is_gone);
    }
}

// ---------------------------------------------------------------------------------------------
// 一段历史：三个几何 × 两条写路径，各跑一遍，留摘要与产物行
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct CellSummary {
    pub(super) geometry: Geometry,
    pub(super) arm: Arm,
    pub(super) reader: Reader,
    pub(super) tree: ReportedTree,
    pub(super) family: &'static str,
    pub(super) is_overwrite_before_unlink: bool,
    pub(super) trajectory: Trajectory,
    pub(super) tree_has_units: bool,
    /// G-环 超出装置前提、G-紧 装不下的段：不算结果（12.3、原文 5.1）。
    pub(super) is_excluded: bool,
}

#[derive(Clone, Debug)]
struct SimulationSummary {
    pub(super) write_path: WritePath,
    pub(super) lines: Vec<String>,
    pub(super) cells: Vec<CellSummary>,
    pub(super) positive_controls: Vec<(Geometry, PositiveControl)>,
    pub(super) assertions: AssertionTally,
    pub(super) placement_log: Vec<Vec<(u64, u64)>>,
    pub(super) scan_log: Vec<Vec<u64>>,
    pub(super) first_reuse_at: Option<u64>,
    pub(super) live_slot_peak: u64,
    pub(super) slots_held_during_a_publish_peak: u64,
}

fn optional_txg(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |txg| txg.to_string())
}

pub(super) fn simulate(specification: HistorySpecification, geometry: Geometry, write_path: WritePath, tight_unit_area_slots: Option<u64>, cross_checks_every_scan: bool) -> Simulation {
    let mut simulation = Simulation::start(geometry, write_path, specification.batch(), tight_unit_area_slots, cross_checks_every_scan);
    run_history(specification, &mut simulation);
    if !simulation.has_stopped() {
        simulation.cross_check_the_unit_scan();
    }
    simulation
}

fn summarize(specification: HistorySpecification, simulation: &Simulation) -> SimulationSummary {
    let label = specification.label();
    let geometry = simulation.geometry;
    let write_path_name = match simulation.write_path {
        WritePath::WithoutTombstones => "N_P",
        WritePath::WithTombstones => "B",
    };
    let mut assertions = simulation.assertions.clone();
    assertions.judge("scans_equal_publishes", simulation.scans == simulation.publishes);
    let is_excluded = simulation.exceeded_the_premise_at.is_some() || simulation.has_stopped();
    let mut lines = vec![format!(
        "name=simulation {label} geometry={} write_path={write_path_name} publishes={} scans={} refused_steps={} units_written={} units_made_unreadable={} readable_units={} occupancy_peak={} live_slot_peak={} first_reuse_at={} reuses={} exceeded_the_premise_at={} does_not_fit_at={} unit_area_slots={}",
        geometry.name(),
        simulation.publishes,
        simulation.scans,
        simulation.refused_steps,
        simulation.disk.units_written,
        simulation.disk.units_made_unreadable,
        simulation.disk.readable_units_by_scan(),
        simulation.occupancy_peak,
        simulation.live_slot_peak,
        optional_txg(simulation.allocator.first_reuse_at),
        simulation.allocator.reuse_placements,
        optional_txg(simulation.exceeded_the_premise_at),
        optional_txg(simulation.does_not_fit_at),
        simulation.allocator.unit_area_slots.map_or_else(|| "unbounded".to_string(), |slots| slots.to_string()),
    )];
    let mut cells = Vec::new();
    for arm in simulation.write_path.arms() {
        for reader in READERS {
            for tree in REPORTED_TREES {
                let trajectory = simulation
                    .trajectories
                    .get(&(arm, reader, tree))
                    .cloned()
                    .unwrap_or_default();
                let tree_has_units = simulation.tree_kinds_with_units.contains(&tree.kind());
                let mut line = format!(
                    "name=trajectory {label} geometry={} arm={} reader={} tree={} kind_code={} publishes={} r_peak={} r_peak_at={} r_positive_scans={} r_final={} m_peak={} m_positive_scans={} m_positive_in_unlink={} m_final={} d_peak={} d_final={} tree_has_units={tree_has_units}",
                    geometry.name(),
                    arm.name(),
                    reader.name(),
                    tree.name(),
                    tree.kind().kind_code(),
                    simulation.publishes,
                    trajectory.resurrected_peak,
                    trajectory.resurrected_peak_at,
                    trajectory.resurrected_positive_scans,
                    trajectory.resurrected_final,
                    trajectory.missed_peak,
                    trajectory.missed_positive_scans,
                    trajectory.missed_positive_scans_in_unlink,
                    trajectory.missed_final,
                    trajectory.deaths_peak,
                    trajectory.deaths_final,
                );
                if geometry == Geometry::Ring {
                    line.push_str(&format!(
                        " r_in_peak={} r_in_positive_scans={} r_out_peak={} r_out_positive_scans={} first_reuse_at={} reuses={} occupancy_peak={}",
                        trajectory.in_window_peak,
                        trajectory.in_window_positive_scans,
                        trajectory.outside_window_peak,
                        trajectory.outside_window_positive_scans,
                        optional_txg(simulation.allocator.first_reuse_at),
                        simulation.allocator.reuse_placements,
                        simulation.occupancy_peak,
                    ));
                }
                lines.push(line);
                cells.push(CellSummary {
                    geometry,
                    arm,
                    reader,
                    tree,
                    family: specification.family(),
                    is_overwrite_before_unlink: specification.is_overwrite_before_unlink(),
                    trajectory,
                    tree_has_units,
                    is_excluded,
                });
            }
        }
    }
    let positive_controls = if simulation.has_stopped() {
        Vec::new()
    } else {
        simulation.positive_controls()
    };
    for control in &positive_controls {
        let (measured, matches) = match control.measured {
            Some(measured) => (measured.to_string(), (measured == control.expected).to_string()),
            None => ("not_plantable".to_string(), "not_plantable".to_string()),
        };
        lines.push(format!(
            "name=positive_control {label} geometry={} arm={} reader={} tree={} kind={} expected={} measured={measured} matches={matches}",
            geometry.name(),
            control.arm.name(),
            control.reader.name(),
            control.tree.name(),
            control.kind,
            control.expected,
        ));
    }
    let deaths_final_of = |arm: Arm, tree: ReportedTree| {
        simulation
            .trajectories
            .get(&(arm, Reader::UnitScan, tree))
            .map_or(0, |trajectory| trajectory.deaths_final)
    };
    let deaths_peak_of = |arm: Arm, tree: ReportedTree| {
        simulation
            .trajectories
            .get(&(arm, Reader::UnitScan, tree))
            .map_or(0, |trajectory| trajectory.deaths_peak)
    };
    if !simulation.has_stopped() {
        for arm in simulation.write_path.arms() {
            let accounting_versions = simulation.scans;
            assertions.judge(
                "A6_accounting_deaths_at_the_end",
                deaths_final_of(arm, ReportedTree::Accounting)
                    == (ACCOUNTING_POOL_WIDE_ROWS + ACCOUNTING_PER_DEVICE_ROWS * u64::from(DEVICE_COUNT)) * (accounting_versions - 1),
            );
            assertions.judge("tombstone_tree_never_loses_a_key", deaths_peak_of(arm, ReportedTree::Tombstone) == 0);
            match specification {
                HistorySpecification::BuildAndDeleteEmptyInodes { inode_count, .. } => {
                    assertions.judge("A7_inode_deaths_at_the_end", deaths_final_of(arm, ReportedTree::Inode) == inode_count - 1);
                    assertions.judge(
                        "A7_deleted_inodes_deaths_at_the_end",
                        deaths_final_of(arm, ReportedTree::DeletedInodes) == inode_count - 1,
                    );
                    if simulation.write_path == WritePath::WithTombstones {
                        assertions.judge(
                            "K1_tombstone_containers",
                            length_as_u64(simulation.version.tombstone_containers.len())
                                == (inode_count - 1).div_ceil(TOMBSTONE_CONTAINER_RECORDS),
                        );
                    }
                }
                HistorySpecification::LargeFileRewriteAndDelete { data_units, .. } => {
                    assertions.judge("A11_extent_deaths_at_the_end", deaths_final_of(arm, ReportedTree::Extent) == data_units);
                }
                HistorySpecification::MixedRandom { .. } => {}
            }
        }
    }
    let unit_free_kinds_without_units = [
        TreeKind::Livelist,
        TreeKind::SparseSideTable,
        TreeKind::Deadlist,
        TreeKind::LoggedOperations,
        TreeKind::Lineage,
    ]
    .iter()
    .all(|kind| !simulation.tree_kinds_with_units.contains(kind));
    assertions.judge("kinds_6_7_8_11_12_have_no_units", unit_free_kinds_without_units);
    if geometry == Geometry::Ring && simulation.publishes >= 30 {
        assertions.judge("ring_reuses_at_least_once_in_a_long_history", simulation.allocator.first_reuse_at.is_some());
    }
    for control in &positive_controls {
        if let Some(measured) = control.measured {
            assertions.judge("positive_control_matches", measured == control.expected);
        }
    }
    let rendered: Vec<String> = assertions
        .judged
        .iter()
        .map(|(name, judged)| format!("{name}={judged}:{}", assertions.violations_of(name)))
        .collect();
    lines.push(format!(
        "name=assertions {label} geometry={} write_path={write_path_name} {}",
        geometry.name(),
        rendered.join(" ")
    ));
    SimulationSummary {
        write_path: simulation.write_path,
        lines,
        cells,
        positive_controls: positive_controls.into_iter().map(|control| (geometry, control)).collect(),
        assertions,
        placement_log: simulation.placement_log.clone(),
        scan_log: simulation.scan_log.clone(),
        first_reuse_at: simulation.allocator.first_reuse_at,
        live_slot_peak: simulation.live_slot_peak,
        slots_held_during_a_publish_peak: simulation.slots_held_during_a_publish_peak,
    }
}

#[derive(Clone, Debug)]
pub(super) struct SegmentResult {
    pub(super) lines: Vec<String>,
    pub(super) cells: Vec<CellSummary>,
    pub(super) positive_controls: Vec<(Geometry, PositiveControl)>,
    pub(super) assertions: AssertionTally,
}

/// G-紧 的单元区：G-宽 下峰值活槽数的 1.25 倍，向上取偶（原文 5.1、C7）。
pub(super) fn tight_unit_area_slots_from(live_slot_peak: u64) -> u64 {
    let raw = (live_slot_peak * 5).div_ceil(4);
    raw + raw % 2
}

const WRITE_PATHS: [WritePath; 2] = [WritePath::WithoutTombstones, WritePath::WithTombstones];

/// 一段历史：先 G-宽（G-紧 的盘按它定），再 G-环，再 G-紧；产物按 G-环、G-宽、G-紧 的次序（12.11 第 4 步）。
pub(super) fn run_segment(specification: HistorySpecification) -> SegmentResult {
    let wide: Vec<SimulationSummary> = WRITE_PATHS
        .iter()
        .map(|write_path| summarize(specification, &simulate(specification, Geometry::Wide, *write_path, None, false)))
        .collect();
    let ring: Vec<SimulationSummary> = WRITE_PATHS
        .iter()
        .map(|write_path| summarize(specification, &simulate(specification, Geometry::Ring, *write_path, None, false)))
        .collect();
    let tight: Vec<SimulationSummary> = wide
        .iter()
        .map(|wide_summary| {
            let slots = tight_unit_area_slots_from(wide_summary.slots_held_during_a_publish_peak);
            summarize(specification, &simulate(specification, Geometry::Tight, wide_summary.write_path, Some(slots), false))
        })
        .collect();
    let mut literal_tight_lines = Vec::new();
    for wide_summary in &wide {
        let literal_slots = tight_unit_area_slots_from(wide_summary.live_slot_peak);
        let literal = simulate(specification, Geometry::Tight, wide_summary.write_path, Some(literal_slots), false);
        literal_tight_lines.push(format!(
            "name=tight_literal_reading {} write_path={} live_slot_peak={} unit_area_slots={literal_slots} does_not_fit_at={} publishes={}",
            specification.label(),
            match wide_summary.write_path {
                WritePath::WithoutTombstones => "N_P",
                WritePath::WithTombstones => "B",
            },
            wide_summary.live_slot_peak,
            optional_txg(literal.does_not_fit_at),
            literal.publishes,
        ));
    }
    let mut assertions = AssertionTally::default();
    for (ring_summary, wide_summary) in ring.iter().zip(&wide) {
        let compared = ring_summary
            .first_reuse_at
            .map_or(ring_summary.placement_log.len(), |txg| u64_as_index(txg - NEW_POOL_FILE_CREATION_TXG))
            .min(ring_summary.placement_log.len())
            .min(wide_summary.placement_log.len());
        let placements_equal = ring_summary.placement_log[..compared] == wide_summary.placement_log[..compared];
        let scans_compared = compared.min(ring_summary.scan_log.len()).min(wide_summary.scan_log.len());
        let scans_equal = ring_summary.scan_log[..scans_compared] == wide_summary.scan_log[..scans_compared];
        assertions.judge("A16_ring_equals_wide_before_the_first_reuse", placements_equal && scans_equal);
    }
    let mut result = SegmentResult {
        lines: Vec::new(),
        cells: Vec::new(),
        positive_controls: Vec::new(),
        assertions,
    };
    for summary in ring.iter().chain(&wide).chain(&tight) {
        result.lines.extend(summary.lines.iter().cloned());
        result.cells.extend(summary.cells.iter().cloned());
        result.positive_controls.extend(summary.positive_controls.iter().cloned());
        result.assertions.merge(&summary.assertions);
    }
    result.lines.extend(literal_tight_lines);
    result.lines.push(format!(
        "name=segment_assertions {} A16_ring_equals_wide_before_the_first_reuse_violations={}",
        specification.label(),
        result.assertions.violations_of("A16_ring_equals_wide_before_the_first_reuse")
    ));
    result
}

/// 按段切片并行（`test-script-discipline.md`「编译与算法都要快」）：每段整段跑两遍、逐行比（C12）；按段的次序收回。
pub(super) fn run_segments_in_parallel(histories: &[HistorySpecification], threads: usize) -> Vec<(SegmentResult, bool)> {
    let next_segment = AtomicUsize::new(0);
    let collected: Mutex<Vec<Option<(SegmentResult, bool)>>> = Mutex::new(vec![None; histories.len()]);
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let position = next_segment.fetch_add(1, Ordering::SeqCst);
                if position >= histories.len() {
                    break;
                }
                let first = run_segment(histories[position]);
                let second = run_segment(histories[position]);
                let identical = first.lines == second.lines;
                collected.lock().expect("收结果的锁没有中毒：工作线程不在持锁时 panic")[position] = Some((first, identical));
            });
        }
    });
    collected
        .into_inner()
        .expect("收结果的锁没有中毒")
        .into_iter()
        .map(|result| result.expect("每一段都收回了"))
        .collect()
}
