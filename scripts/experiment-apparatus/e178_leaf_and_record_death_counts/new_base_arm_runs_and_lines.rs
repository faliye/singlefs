//! 新底座臂的构造与阳性对照、一条路径的整趟、轨迹与差量行与各项行。

use super::*;

/// 修订 1 的 PC-B2-direct 构造（容量 16、θ = 0、N1：建 1..64，删 34..48 当 R_old，缺口里删 33，回退，再删 49..64）在给定写路径上跑一遍。
fn direct_rollback_construction(container_death: ContainerDeathWritePath, readers: Vec<Reader>) -> (Simulation, bool) {
    let write_path = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: TEST_LEAF_CAPACITY, merge_threshold: 0 };
    let mut simulation = Simulation::start_with(write_path, PRODUCT_UNLINK_BATCH_DATA_UNITS, false, container_death, readers);
    simulation.apply(UserChange::NewInodes { count: 63 });
    for object in 34..=48 {
        simulation.apply(UserChange::Unlink { object });
    }
    let old_version = simulation.version.clone();
    simulation.apply(UserChange::Unlink { object: 33 });
    let rolled_back = simulation.roll_back_to(old_version);
    for object in 49..=64 {
        simulation.apply(UserChange::Unlink { object });
    }
    (simulation, rolled_back)
}

/// PC-Bs2（修订 2 之六）：构造期末，Z × E-I × W = ∞。B-stub 容器 m 0、inode m 0；B-stub-lin-off 容器 m 1、inode m 1、extent m 0。
/// 交回 (成立, 产物行的字段)；B-stub 那两个数也是 Q-Bs-2 要另报的「PC-Bs2 构造上 B-stub 的值」。
fn stub_lineage_positive_control() -> (bool, String) {
    let (simulation, rolled_back) = direct_rollback_construction(ContainerDeathWritePath::ZeroRecordStubs, segment_four_stub_path_readers());
    let readers = [
        Reader { base: BaseArm::ZeroRecordStubs, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever },
        Reader { base: BaseArm::ZeroRecordStubsIgnoringLineage, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever },
    ];
    let group_index_readings = simulation.read_readers(&readers, &Overlay::default(), simulation.publish_sequence, SelectionPath::GroupIndex);
    let reference = simulation.read_readers(&readers, &Overlay::default(), simulation.publish_sequence, SelectionPath::FullScan);
    let paths_agree = group_index_readings == reference;
    let stub = group_index_readings[0];
    let lineage_off = group_index_readings[1];
    let holds = rolled_back
        && paths_agree
        && (stub.container_missed, stub.inode_missed) == (0, 0)
        && (lineage_off.container_missed, lineage_off.inode_missed, lineage_off.extent_missed) == (1, 1, 0);
    (
        holds,
        format!(
            "rolled_back={rolled_back} b_stub_container_m={} b_stub_inode_m={} b_stub_extent_m={} b_stub_lin_off_container_m={} b_stub_lin_off_inode_m={} b_stub_lin_off_extent_m={} expected=0,0,1,1,0 paths_agree={paths_agree}",
            stub.container_missed, stub.inode_missed, stub.extent_missed, lineage_off.container_missed, lineage_off.inode_missed, lineage_off.extent_missed
        ),
    )
}

/// PC-Bt3（修订 2 之六）：构造期末，B-tsz × Z × E-I × W = ∞，整对象墓碑不按谱系滤：inode m + 1（缺口里 inode 33 的墓碑被施行），不改时 0。
fn tombstone_lineage_positive_control() -> (bool, String) {
    let (simulation, rolled_back) = direct_rollback_construction(ContainerDeathWritePath::RetirementRecords, segment_four_original_path_readers());
    let readers = [Reader { base: BaseArm::TombstonesOnly, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever }];
    let overlay = Overlay::ignoring_the_lineage_of_whole_object_tombstones();
    let read = |overlay_to_read: &Overlay, path: SelectionPath| simulation.read_readers(&readers, overlay_to_read, simulation.publish_sequence, path)[0];
    let baseline = read(&Overlay::default(), SelectionPath::GroupIndex);
    let modified = read(&overlay, SelectionPath::GroupIndex);
    let paths_agree = baseline == read(&Overlay::default(), SelectionPath::FullScan) && modified == read(&overlay, SelectionPath::FullScan);
    let holds = rolled_back && paths_agree && baseline.inode_missed == 0 && modified.inode_missed == baseline.inode_missed + 1;
    (
        holds,
        format!(
            "rolled_back={rolled_back} b_tsz_inode_m={} b_tsz_container_m={} b_tsz_extent_m={} b_tsz_inode_m_ignoring_lineage={} expected=0,1 paths_agree={paths_agree}",
            baseline.inode_missed, baseline.container_missed, baseline.extent_missed, modified.inode_missed
        ),
    )
}

/// 第四段一条写路径跑两遍（第十一节 V4：逐次扫描比；第一遍定对照的扫描），留第二遍。
pub(super) struct SegmentFourPathRun {
    pub(super) simulation: Simulation,
    pub(super) rollback: Option<RollbackOutcome>,
    pub(super) two_runs_identical: bool,
    pub(super) last_scan_paths_agree: bool,
    pub(super) positive_controls: Vec<PositiveControlOutcome>,
}

pub(super) fn run_segment_four_path(history: History, cell: WritePathCell, container_death: ContainerDeathWritePath, readers: &[Reader]) -> SegmentFourPathRun {
    let reversed_provenance_applies = container_death == ContainerDeathWritePath::RetirementRecords
        && history.family() == "F10"
        && cell.merge_threshold > 0;
    let start = || {
        let mut simulation = Simulation::start_with(cell, history.batch(), true, container_death, readers.to_vec());
        simulation.segment_four = Some(SegmentFourProbes {
            plan: SegmentFourPlan::default(),
            observed: SegmentFourPlan::default(),
            outcomes: Vec::new(),
            reversed_provenance_applies,
        });
        simulation
    };
    let mut first = start();
    run_history(history, &mut first);
    let plan = positive_control_plan_of(&first.scans);
    let four_plan = first.segment_four.as_ref().map_or_else(SegmentFourPlan::default, |probes| probes.observed);
    let first_scans = std::mem::take(&mut first.scans);
    drop(first);
    let mut second = start();
    second.positive_control_plan = plan;
    if let Some(probes) = second.segment_four.as_mut() {
        probes.plan = four_plan;
    }
    let rollback = run_history(history, &mut second);
    let two_runs_identical = second.scans == first_scans;
    let last_publish = second.publish_sequence;
    let last_scan_paths_agree = second.group_index_equals_the_full_scan(last_publish);
    let mut positive_controls = second.positive_control_outcomes.clone();
    positive_controls.extend(second.segment_four.as_ref().map(|probes| probes.outcomes.clone()).unwrap_or_default());
    positive_controls.extend(second.erasure_positive_controls(last_publish));
    if reversed_provenance_applies {
        positive_controls.push(second.reversed_provenance_positive_control(last_publish));
    }
    SegmentFourPathRun { simulation: second, rollback, two_runs_identical, last_scan_paths_agree, positive_controls }
}

/// 轨迹行，与第二段 `run_segment_two_item` 写的同一个形态（第四段 V-Bt2 按行比 B-on、B-off-ret 那几行）。
fn trajectory_lines(label: &str, readers: &[Reader], trajectories: &[ReaderTrajectory], scopes_reported: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for (reader, trajectory) in readers.iter().zip(trajectories) {
        for (scope_position, scope) in SCAN_SCOPES.iter().enumerate().take(scopes_reported) {
            let fields: Vec<String> = COUNTED_QUANTITIES
                .iter()
                .zip(&trajectory.by_scope[scope_position])
                .map(|(quantity, quantity_trajectory)| quantity_trajectory.fields(quantity))
                .collect();
            lines.push(format!(
                "name=trajectory {label} {} scope={scope} scans={} {} read_out_empty_scans={}",
                reader.label(),
                trajectory.by_scope[scope_position][0].scans,
                fields.join(" "),
                trajectory.read_out_empty_scans
            ));
        }
    }
    lines
}

/// 「差」的四条轨迹：同一次扫描、同一 (X2, X3, W) 下新臂减 B-on，两个方向各一条（修订 2 之七）。
pub(super) const DIFFERENCE_QUANTITIES: [&str; 4] = ["extent_r_more", "extent_r_less", "extent_m_more", "extent_m_less"];
pub(super) const DIFFERENCE_EXTENT_RESURRECTED_MORE: usize = 0;
pub(super) const DIFFERENCE_EXTENT_MISSED_MORE: usize = 2;

fn difference_trajectories(new_scans: &[ScanRecord], new_position: usize, base_scans: &[ScanRecord], base_position: usize) -> [[QuantityTrajectory; 4]; 5] {
    let mut trajectories = [[QuantityTrajectory::default(); 4]; 5];
    for (new_scan, base_scan) in new_scans.iter().zip(base_scans) {
        let new_counts = new_scan.counts[new_position];
        let base_counts = base_scan.counts[base_position];
        let values = [
            new_counts.extent_resurrected.saturating_sub(base_counts.extent_resurrected),
            base_counts.extent_resurrected.saturating_sub(new_counts.extent_resurrected),
            new_counts.extent_missed.saturating_sub(base_counts.extent_missed),
            base_counts.extent_missed.saturating_sub(new_counts.extent_missed),
        ];
        for (scope_position, in_scope) in scan_scopes_of(base_scan).iter().enumerate() {
            if *in_scope {
                for (quantity_position, value) in values.iter().enumerate() {
                    trajectories[scope_position][quantity_position].observe(*value, base_scan.publish_sequence);
                }
            }
        }
    }
    trajectories
}

fn difference_lines(label: &str, reader: Reader, trajectories: &[[QuantityTrajectory; 4]; 5], scopes_reported: usize) -> Vec<String> {
    SCAN_SCOPES
        .iter()
        .enumerate()
        .take(scopes_reported)
        .map(|(scope_position, scope)| {
            let fields: Vec<String> =
                DIFFERENCE_QUANTITIES.iter().zip(&trajectories[scope_position]).map(|(quantity, trajectory)| trajectory.fields(quantity)).collect();
            format!(
                "name=difference {label} {} paired_base=B-on scope={scope} scans={} {}",
                reader.label(),
                trajectories[scope_position][0].scans,
                fields.join(" ")
            )
        })
        .collect()
}

/// 第四段的阳性对照行：同第二段的形态，PC-M 另报登记字面（每个读者 extent m + 1；抹容器那一次 inode m + 记录数、容器 m + 1）成不成立。
fn segment_four_positive_control_line(outcome: &PositiveControlOutcome, context: &str) -> String {
    let literal = match outcome.control_identifier {
        "PC-M-data" => Some(outcome.main_deltas.iter().all(|delta| delta[DELTA_EXTENT_MISSED] == 1)),
        "PC-M-leaf" => {
            let records: Option<i64> = outcome.detail.split(' ').find_map(|field| field.strip_prefix("records=")).and_then(|value| value.parse().ok());
            Some(records.is_some_and(|records| outcome.main_deltas.iter().all(|delta| delta[DELTA_INODE_MISSED] == records && delta[DELTA_CONTAINER_MISSED] == 1)))
        }
        _ => None,
    };
    match literal {
        Some(holds) => format!("{} registered_literal_holds={holds}", outcome.line(context)),
        None => outcome.line(context),
    }
}

/// A11 的闭式（修订 2 之八、`anchors3.py`）：H51 F1 升序、θ = 0，第一个桩出现在第几次发布（新池新建文件是第 1 次）与有桩的版本数。
/// 第一个删空的是第二片（号 容量 + 1 .. 2 × 容量），删到它最大那个号的最后一批那次。
fn first_stub_publish_and_versions_with_a_stub(inode_count: u64, leaf_capacity: u64) -> (u64, u64) {
    let creation_publishes = (inode_count - 1).div_ceil(NEW_INODES_PER_STEP);
    let total_publishes = 3 + creation_publishes + 2 * (inode_count - 1);
    let first_emptied_inode = inode_count.min(2 * leaf_capacity);
    let first_stub_publish = 3 + creation_publishes + 2 * (first_emptied_inode - 1);
    (first_stub_publish, total_publishes - first_stub_publish + 1)
}

/// 两条写路径的配对前提（修订 2 之五、V-Bs1）：用户改动序列、发布数、每一版的 txg、活叶（身份、分隔 key、记录）、数据单元、
/// 每次扫描的发布序号、txg 与真值的记录数都逐个相同。
fn pairing_holds(original: &Simulation, stub: &Simulation) -> bool {
    original.user_changes == stub.user_changes
        && original.publishes == stub.publishes
        && original.publish_records.len() == stub.publish_records.len()
        && original.publish_records.iter().zip(&stub.publish_records).all(|(original_record, stub_record)| {
            original_record.txg == stub_record.txg
                && original_record.publish_sequence == stub_record.publish_sequence
                && original_record.leaf_digest == stub_record.leaf_digest
                && original_record.extent_digest == stub_record.extent_digest
                && std::mem::discriminant(&original_record.merge) == std::mem::discriminant(&stub_record.merge)
        })
        && original.scans.len() == stub.scans.len()
        && original.scans.iter().zip(&stub.scans).all(|(original_scan, stub_scan)| {
            (original_scan.publish_sequence, original_scan.txg, original_scan.truth_inode_keys, original_scan.inside_an_unlink)
                == (stub_scan.publish_sequence, stub_scan.txg, stub_scan.truth_inode_keys, stub_scan.inside_an_unlink)
        })
}

/// Q-Bs-5、Q-Bs-6：B-stub 写路径每一版的叶片总数、桩数、桩占比（百万分之一）、做完的 unlink 数；撞 I-9.13 的版本数（修订 2 之七）。
fn stub_leaves_line(label: &str, stub: &Simulation, original: &Simulation) -> String {
    let mut total_leaves = QuantityTrajectory::default();
    let mut stubs = QuantityTrajectory::default();
    let mut stub_share = QuantityTrajectory::default();
    let mut live_leaves = QuantityTrajectory::default();
    let mut versions_over_the_root = 0_u64;
    let mut first_over_the_root: Option<u64> = None;
    let mut first_stub_publish: Option<u64> = None;
    for (position, record) in stub.publish_records.iter().enumerate() {
        let publish_number = length_as_u64(position) + 1;
        total_leaves.observe(record.leaves_in_the_leaf_order, publish_number);
        stubs.observe(record.stubs_in_the_leaf_order, publish_number);
        let share = (record.stubs_in_the_leaf_order * 1_000_000).checked_div(record.leaves_in_the_leaf_order).unwrap_or(0);
        stub_share.observe(share, publish_number);
        // 活叶数 = 叶片总数 − 桩数；配对成立时它逐版等于同格原写路径的叶容器数（V-Bs1）。
        live_leaves.observe(difference_or_marker(record.leaves_in_the_leaf_order, record.stubs_in_the_leaf_order), publish_number);
        if record.leaves_in_the_leaf_order > ROOT_NODE_LEAF_CONTAINERS {
            versions_over_the_root += 1;
            first_over_the_root.get_or_insert(publish_number);
        }
        if record.stubs_in_the_leaf_order > 0 {
            first_stub_publish.get_or_insert(publish_number);
        }
    }
    let shown = |value: Option<u64>| value.map_or_else(|| "none".to_string(), |known| known.to_string());
    let versions_with_an_empty_leaf = stub.publish_records.iter().filter(|record| record.holds_an_empty_inode_leaf).count();
    format!(
        "name=stub_leaves {label} versions={} versions_with_an_empty_inode_leaf={versions_with_an_empty_leaf} first_stub_publish={} stubs_written={} {} {} {} {} versions_over_the_root={versions_over_the_root} first_over_the_root={} unlinks_completed={} original_live_leaves_last={}",
        stub.publish_records.len(),
        shown(first_stub_publish),
        stub.publish_records.iter().map(|record| record.stubs_written).sum::<u64>(),
        total_leaves.fields("leaves"),
        stubs.fields("stubs"),
        stub_share.fields("stub_share_ppm"),
        live_leaves.fields("live_leaves"),
        shown(first_over_the_root),
        stub.unlinks_completed,
        original.version.leaves.len(),
    )
}

/// 第四段一个 (历史段, 写路径格) 工作项的全部产物行：原写路径 18 个读者，N1 的格另跑 B-stub 写路径 12 个读者（问法一 N2-none）。
fn segment_four_history_item_lines(history: History, cell: WritePathCell) -> Vec<String> {
    let label = format!("{} {}", history.label(), cell.label());
    let original_readers = segment_four_original_path_readers();
    let original = run_segment_four_path(history, cell, ContainerDeathWritePath::RetirementRecords, &original_readers);
    let mut checks = AssertionTally::default();
    let mut lines = Vec::new();
    let scopes_reported = if original.rollback.is_some() { SCAN_SCOPES.len() } else { 3 };
    let simulation = &original.simulation;
    let (borrows, absorbs, drops) = simulation.count_merges();
    lines.push(format!(
        "name=simulation segment=four {label} path=original publishes={} scans={} refused_steps={} borrows={borrows} absorbs={absorbs} drops={drops} moved_then_shortened={} moved_then_unlinked={} retirement_records={} does_not_fit_at={} rollback_event_found={} rollback_too_far={} two_runs_identical={} last_scan_paths_agree={}",
        simulation.publishes,
        simulation.scans.len(),
        simulation.refused_steps,
        simulation.moved_then_shortened,
        simulation.moved_then_unlinked,
        simulation.total_written().retirement_records,
        simulation.does_not_fit_at.map_or_else(|| "none".to_string(), |txg| txg.to_string()),
        original.rollback.map_or_else(|| "none".to_string(), |outcome| outcome.event_found.to_string()),
        original.rollback.map_or_else(|| "none".to_string(), |outcome| outcome.too_far.to_string()),
        original.two_runs_identical,
        original.last_scan_paths_agree,
    ));
    let original_trajectories = trajectories_of(&simulation.scans, original_readers.len());
    lines.extend(trajectory_lines(&label, &original_readers, &original_trajectories, scopes_reported));
    let without_retirement_position = |window: ReadableWindow| {
        original_readers
            .iter()
            .position(|reader| *reader == Reader { base: BaseArm::WithoutRetirement, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window })
            .expect("原写路径读者表里有 B-off-ret")
    };
    for (position, reader) in original_readers.iter().enumerate().filter(|(_, reader)| reader.base == BaseArm::TombstonesOnly) {
        let paired = Reader { base: BaseArm::RetirementOnLineage, shrink: reader.shrink, effect: reader.effect, window: reader.window };
        let paired_position = original_readers.iter().position(|candidate| *candidate == paired).expect("B-on 同一 (X2, X3, W) 在表里");
        let differences = difference_trajectories(&simulation.scans, position, &simulation.scans, paired_position);
        lines.extend(difference_lines(&label, *reader, &differences, scopes_reported));
        let provenance_difference_scans = simulation.scans.iter().filter(|scan| scan.counts[position].provenance_differences > 0).count();
        let provenance_differences_total: u64 = simulation.scans.iter().map(|scan| scan.counts[position].provenance_differences).sum();
        lines.push(format!(
            "name=reader_extras {label} {} provenance_difference_scans={provenance_difference_scans} provenance_differences_total={provenance_differences_total}",
            reader.label()
        ));
        // A12：B-tsz 的容器 r 与同 W 的 B-off-ret × Z × E-I 逐次扫描相等。
        let comparison = without_retirement_position(reader.window);
        for scan in &simulation.scans {
            checks.judge("A12_tombstones_only_container_r_equals_without_retirement", scan.counts[position].container_resurrected == scan.counts[comparison].container_resurrected);
        }
        if let (History::BuildAndDeleteEmptyInodes { inode_count, .. }, 0, ReadableWindow::Forever) = (history, cell.merge_threshold, reader.window) {
            let expected = inode_count.div_ceil(cell.leaf_capacity) - 1;
            let last = simulation.scans.last().map_or(u64::MAX, |scan| scan.counts[position].container_resurrected);
            checks.judge("A12_tombstones_only_final_container_r_equals_a8", last == expected);
        }
    }
    for outcome in &original.positive_controls {
        lines.push(segment_four_positive_control_line(outcome, &format!("{label} path=original")));
        checks.judge("V2_positive_control_paths_agree", outcome.paths_agree);
        checks.judge("V2_positive_control_closed_form_holds", outcome.closed_form_holds);
    }
    for (name, judged) in &simulation.assertions.judged {
        lines.push(format!("name=item_assertion {label} path=original assertion={name} judged={judged} violated={}", simulation.assertions.violations_of(name)));
    }
    if cell.borrow_arm == BorrowArm::RaiseSeparator {
        lines.extend(stub_path_lines(history, cell, &label, &original, &mut checks));
    }
    for (name, judged) in &checks.judged {
        lines.push(format!("name=item_assertion {label} path=segment_four assertion={name} judged={judged} violated={}", checks.violations_of(name)));
    }
    lines
}

/// B-stub 写路径那一半（N1 的格）：12 个读者的轨迹、与原写路径同 (X2, X3, W) 的 B-on 配对算差、V-Bs1、V-Bs3、A10、A11、Q-Bs-5、Q-Bs-6。
fn stub_path_lines(history: History, cell: WritePathCell, label: &str, original: &SegmentFourPathRun, checks: &mut AssertionTally) -> Vec<String> {
    let stub_readers = segment_four_stub_path_readers();
    let original_readers = segment_four_original_path_readers();
    let stub = run_segment_four_path(history, cell, ContainerDeathWritePath::ZeroRecordStubs, &stub_readers);
    let simulation = &stub.simulation;
    let scopes_reported = if stub.rollback.is_some() { SCAN_SCOPES.len() } else { 3 };
    let paired = pairing_holds(&original.simulation, simulation);
    checks.judge("V-Bs1_pairing_holds", paired);
    let mut stub_read_mismatches = 0_u64;
    let mut lines = Vec::new();
    for (position, reader) in stub_readers.iter().enumerate() {
        let mismatched_scans = simulation
            .scans
            .iter()
            .zip(&simulation.publish_records)
            .filter(|(scan, record)| scan.counts[position].stubs_read != record.stubs_in_the_leaf_order)
            .count();
        stub_read_mismatches += length_as_u64(mismatched_scans);
        lines.push(format!(
            "name=reader_extras {label} {} stub_read_mismatch_scans={mismatched_scans} stubs_read_last={}",
            reader.label(),
            simulation.scans.last().map_or(0, |scan| scan.counts[position].stubs_read)
        ));
    }
    checks.judge("V-Bs3_stubs_read_equal_the_current_stubs", stub_read_mismatches == 0 && simulation.scans.len() == simulation.publish_records.len());
    lines.insert(
        0,
        format!(
            "name=simulation segment=four {label} path=stub publishes={} scans={} refused_steps={} stubs_written={} retirement_records={} does_not_fit_at={} pairing_holds={paired} stub_read_mismatch_scans={stub_read_mismatches} two_runs_identical={} last_scan_paths_agree={}",
            simulation.publishes,
            simulation.scans.len(),
            simulation.refused_steps,
            simulation.publish_records.iter().map(|record| record.stubs_written).sum::<u64>(),
            simulation.total_written().retirement_records,
            simulation.does_not_fit_at.map_or_else(|| "none".to_string(), |txg| txg.to_string()),
            stub.two_runs_identical,
            stub.last_scan_paths_agree,
        ),
    );
    let stub_trajectories = trajectories_of(&simulation.scans, stub_readers.len());
    lines.extend(trajectory_lines(label, &stub_readers, &stub_trajectories, scopes_reported));
    if paired {
        for (position, reader) in stub_readers.iter().enumerate() {
            let base = Reader { base: BaseArm::RetirementOnLineage, shrink: reader.shrink, effect: reader.effect, window: reader.window };
            let base_position = original_readers.iter().position(|candidate| *candidate == base).expect("B-on 同一 (X2, X3, W) 在表里");
            let differences = difference_trajectories(&simulation.scans, position, &original.simulation.scans, base_position);
            lines.extend(difference_lines(label, *reader, &differences, scopes_reported));
        }
    }
    lines.push(stub_leaves_line(label, simulation, &original.simulation));
    if let (History::BuildAndDeleteEmptyInodes { inode_count, order }, 0) = (history, cell.merge_threshold) {
        let disappearances = inode_count.div_ceil(cell.leaf_capacity) - 1;
        let stubs_written: u64 = simulation.publish_records.iter().map(|record| record.stubs_written).sum();
        let final_leaves = length_as_u64(simulation.version.leaves.len());
        checks.judge(
            "A10_stubs_written_equal_final_stubs_equal_a8",
            stubs_written == disappearances && simulation.version.stub_count() == disappearances && final_leaves == inode_count.div_ceil(cell.leaf_capacity),
        );
        if order == DeletionOrder::Ascending {
            let (expected_first, expected_versions) = first_stub_publish_and_versions_with_a_stub(inode_count, cell.leaf_capacity);
            let first = simulation.publish_records.iter().position(|record| record.stubs_in_the_leaf_order > 0).map(|position| length_as_u64(position) + 1);
            let versions = length_as_u64(simulation.publish_records.iter().filter(|record| record.holds_an_empty_inode_leaf).count());
            checks.judge("A11_first_stub_publish_and_versions_with_a_stub", first == Some(expected_first) && versions == expected_versions);
        }
    }
    for outcome in &stub.positive_controls {
        lines.push(segment_four_positive_control_line(outcome, &format!("{label} path=stub")));
        checks.judge("V2_positive_control_paths_agree", outcome.paths_agree);
        checks.judge("V2_positive_control_closed_form_holds", outcome.closed_form_holds);
    }
    for (name, judged) in &simulation.assertions.judged {
        lines.push(format!("name=item_assertion {label} path=stub assertion={name} judged={judged} violated={}", simulation.assertions.violations_of(name)));
    }
    lines
}

/// 第四段的工作项（修订 2 之十三：H51 51 段 + F10 28 段 + F11 4 段，各 × 12 格，加 PC-B2-direct 构造 1 项，共 997 项）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SegmentFourItem {
    HistoryCell { history: History, cell: WritePathCell },
    DirectRollbackConstruction,
}

impl SegmentFourItem {
    pub(super) fn label(self) -> String {
        match self {
            SegmentFourItem::HistoryCell { history, cell } => format!("{} {}", history.label(), cell.label()),
            SegmentFourItem::DirectRollbackConstruction => "construction=PC-B2-direct".to_string(),
        }
    }
}

pub(super) fn segment_four_items() -> Vec<SegmentFourItem> {
    let mut items: Vec<SegmentFourItem> =
        segment_two_items().into_iter().map(|item| SegmentFourItem::HistoryCell { history: item.history, cell: item.cell }).collect();
    items.push(SegmentFourItem::DirectRollbackConstruction);
    items
}

pub(super) fn segment_four_item_lines(item: SegmentFourItem) -> Vec<String> {
    match item {
        SegmentFourItem::HistoryCell { history, cell } => segment_four_history_item_lines(history, cell),
        SegmentFourItem::DirectRollbackConstruction => {
            let (stub_holds, stub_detail) = stub_lineage_positive_control();
            let (tombstone_holds, tombstone_detail) = tombstone_lineage_positive_control();
            vec![
                format!("name=positive_control id=PC-Bs2 {stub_detail} holds={stub_holds}"),
                format!("name=positive_control id=PC-Bt3 {tombstone_detail} holds={tombstone_holds}"),
            ]
        }
    }
}

#[cfg(test)]
#[path = "new_base_arm_runs_and_lines_tests.rs"]
mod tests;
