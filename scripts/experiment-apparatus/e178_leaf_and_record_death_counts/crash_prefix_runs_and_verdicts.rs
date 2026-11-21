//! 崩溃那一半：停机核对、前缀、崩溃支、汇总与判定行与整趟。

use super::*;

/// 停机对拍 S4：意图那次（`crates/singlefs-core/src/transaction.rs:4065-4071`、`records.rs:119-147`）——记录 nlink = 0、size 不变、
/// 追加一条整对象墓碑 (N, b, [0, 2⁶⁴ − 1])，死亡 txg = 意图 txg（今天那条写流）。
pub(super) fn stop_check_intent_matches_the_implementation() -> (bool, String) {
    let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: INODE_LEAF_RECORDS, merge_threshold: 0 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    simulation.apply(UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 3 });
    let size_before = simulation.version.record_of(FIRST_FILE_INODE).map(|record| record.size_bytes);
    simulation.publish(&Operation::UnlinkIntent { object: FIRST_FILE_INODE });
    let intent_txg = simulation.version.txg;
    let record = simulation.version.record_of(FIRST_FILE_INODE);
    let today = &simulation.version.tombstone_trees[TombstoneStream::WholeAtIntent.position()];
    let tombstones: Vec<TombstoneRecord> = today.containers.iter().flat_map(|container| container.records.iter().copied()).collect();
    let holds = record.is_some_and(|record| record.nlink == 0 && Some(record.size_bytes) == size_before)
        && tombstones.len() == 1
        && tombstones[0].object == FIRST_FILE_INODE
        && tombstones[0].range_start_bytes == WHOLE_OBJECT_RANGE_START_BYTES
        && tombstones[0].death_txg == intent_txg
        && simulation.version.deleted_queue == Some(FIRST_FILE_INODE)
        && simulation.version.tombstone_trees[TombstoneStream::WholeAtFinal.position()].record_count() == 0;
    (holds, format!("record={record:?} tombstones={tombstones:?} intent_txg={intent_txg}"))
}

/// 停机对拍 S5：墓碑树（`crates/singlefs-core/src/tombstone_tree.rs:117` 起的追加）——开放容器追加、满 815 开新容器、号 = 最大号 + 1、
/// 有新记录那次才重写。建 1 + 816 个空 inode 再逐个 unlink 816 个：第 816 次意图开第 2 个容器。
pub(super) fn stop_check_tombstone_tree_matches_the_implementation() -> (bool, String) {
    let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: INODE_LEAF_RECORDS, merge_threshold: 0 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    let unlinks = TOMBSTONE_CONTAINER_RECORDS + 1;
    simulation.apply(UserChange::NewInodes { count: unlinks });
    for object in 2..=unlinks + 1 {
        simulation.apply(UserChange::Unlink { object });
    }
    let today = &simulation.version.tombstone_trees[TombstoneStream::WholeAtIntent.position()];
    let sizes: Vec<usize> = today.containers.iter().map(|container| container.records.len()).collect();
    let numbers: Vec<u64> = today.containers.iter().map(|container| container.identity.number).collect();
    let position = TombstoneStream::WholeAtIntent.position();
    let rewrites_only_with_new_records = simulation.publish_records.iter().all(|record| {
        let wrote_a_whole_tombstone = record.written.tombstone_containers[position] > 0;
        wrote_a_whole_tombstone == (record.written.tombstone_roots[position] == 1)
    });
    let intent_publishes = simulation.publish_records.iter().filter(|record| record.written.tombstone_containers[position] == 1).count();
    let holds = sizes == vec![815, 1] && numbers == vec![1, 2] && rewrites_only_with_new_records && length_as_u64(intent_publishes) == unlinks;
    (holds, format!("container_sizes={sizes:?} container_numbers={numbers:?} intent_publishes={intent_publishes}"))
}

/// PC-B2 的直接构造（登记修订 1）：容量 16、θ = 0，建 inode 1..64（四片 L1–L4），删 34..48 之后那一版当 R_old（队列空）；
/// 缺口里删 33（L3 删空、退役记录死亡 txg 在缺口里，L4 左移重写）；回退到 R_old（L3、L4 回到缺口之前那一版）；
/// 再删 49..64（最右的 L4 删空、不左移），退役容器这一次在新父链上重写、装着缺口里那条。期末扫描：B-on 的容器 m = 0，B-off-lin 的 = 1。
pub(super) fn direct_rollback_positive_control_holds() -> (bool, String) {
    let write_path = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: TEST_LEAF_CAPACITY, merge_threshold: 0 };
    let mut simulation = Simulation::start(write_path, PRODUCT_UNLINK_BATCH_DATA_UNITS, true);
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
    let readers = every_reader();
    let container_missed = |base: BaseArm| {
        let position = readers
            .iter()
            .position(|reader| *reader == Reader { base, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever })
            .expect("读者表里有它");
        simulation.scans.last().map_or(u64::MAX, |scan| scan.counts[position].container_missed)
    };
    let on_lineage = container_missed(BaseArm::RetirementOnLineage);
    let ignoring_lineage = container_missed(BaseArm::RetirementIgnoringLineage);
    let without_retirement = container_missed(BaseArm::WithoutRetirement);
    let paths_agree = simulation.group_index_equals_the_full_scan(simulation.publish_sequence);
    let holds = rolled_back && on_lineage == 0 && ignoring_lineage == 1 && without_retirement == 0 && paths_agree;
    (
        holds,
        format!(
            "rolled_back={rolled_back} b_on_container_m={on_lineage} b_off_lin_container_m={ignoring_lineage} b_off_ret_container_m={without_retirement} expected=0,1,0 paths_agree={paths_agree}"
        ),
    )
}

// ---------------------------------------------------------------------------------------------
// 第三段（登记 5.8）：F12 意图到最后一批之间崩溃的全部崩溃支；Q-X3-O2、Q-X3-4、Q-X3-5；PC-O、PC-M（崩溃点那一份）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CrashPrefix {
    /// P-1：新池新建文件、写行、暖机，顺序写 inode 1 共 u 个单元。
    FirstFileOnly,
    /// P-m：F10 的建与写（n = 300），按升序删到第一个记录被借走或被吸收过的活对象 X 出现，把 X 顺序写成 u 个单元。
    MovedObject,
}

impl CrashPrefix {
    pub(super) const fn label(self) -> &'static str {
        match self {
            CrashPrefix::FirstFileOnly => "P-1",
            CrashPrefix::MovedObject => "P-m",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SegmentThreeItem {
    prefix: CrashPrefix,
    pub(super) cell: WritePathCell,
}

/// P-1 与 θ 无关，只跑 θ = 0；P-m 要有借或吸收，只跑 θ > 0。
fn segment_three_items() -> Vec<SegmentThreeItem> {
    let mut items = Vec::new();
    for leaf_capacity in LEAF_CAPACITIES {
        for borrow_arm in BORROW_ARMS {
            items.push(SegmentThreeItem { prefix: CrashPrefix::FirstFileOnly, cell: WritePathCell { borrow_arm, leaf_capacity, merge_threshold: 0 } });
        }
        for merge_threshold in merge_thresholds_of(leaf_capacity).into_iter().skip(1) {
            for borrow_arm in BORROW_ARMS {
                items.push(SegmentThreeItem { prefix: CrashPrefix::MovedObject, cell: WritePathCell { borrow_arm, leaf_capacity, merge_threshold } });
            }
        }
    }
    items
}

/// 前缀只模拟一次（按前缀剪），交回前缀之后的模拟与要删的对象；P-m 删光也没出现 X 时交回 None。
fn build_crash_prefix(prefix: CrashPrefix, cell: WritePathCell) -> Option<(Simulation, u64)> {
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    match prefix {
        CrashPrefix::FirstFileOnly => Some((simulation, FIRST_FILE_INODE)),
        CrashPrefix::MovedObject => {
            build_and_write_many_objects(&mut simulation, INTENT_CRASH_INODE_COUNT);
            loop {
                let moved_and_live = simulation.moved_objects.iter().copied().find(|object| simulation.version.is_live(*object));
                if let Some(object) = moved_and_live {
                    return Some((simulation, object));
                }
                let next_victim = simulation.version.live_inodes().into_iter().find(|inode| *inode > FIRST_FILE_INODE)?;
                simulation.apply(UserChange::Unlink { object: next_victim });
                if simulation.has_stopped() {
                    return None;
                }
            }
        }
    }
}

/// 一个崩溃支留下的东西。
#[derive(Clone, Debug)]
struct CrashBranchOutcome {
    crash_counts: Vec<ReaderCounts>,
    crash_paths_agree: bool,
    pub(super) positive_controls: Vec<PositiveControlOutcome>,
    recovery_trajectories: Vec<ReaderTrajectory>,
    recovery_paths_agree: bool,
}

fn run_one_crash_branch(before_the_crash: &Simulation, object: u64) -> Option<CrashBranchOutcome> {
    let mut branch = before_the_crash.clone();
    let record = branch.version.record_of(object)?;
    let intent_txg = branch.intent_txgs.get(&object).copied()?;
    let crash_publish = branch.crash_with_the_next_unlink_step_written()?;
    let crash_counts = branch.read_every_reader(&Overlay::default(), crash_publish, SelectionPath::GroupIndex);
    let crash_paths_agree = crash_counts == branch.read_every_reader(&Overlay::default(), crash_publish, SelectionPath::FullScan);
    let planted = InodeRecord { inode: object, object_birth: record.object_birth, nlink: 1, size_bytes: record.size_bytes, change_counter: 0 };
    let plant = Overlay::planting(vec![Simulation::planted_leaf(planted, branch.newest_write_order(), crash_publish)]);
    let plant_closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
        readers.iter().zip(deltas).all(|(reader, delta)| match reader.effect {
            UnlinkEffectArm::AtIntent => delta[DELTA_LIVE_RECORDS_BEYOND_THE_TRUTH] == 0,
            UnlinkEffectArm::AtLastBatchWritten | UnlinkEffectArm::AtLastBatchRead => delta[DELTA_LIVE_RECORDS_BEYOND_THE_TRUTH] == 1,
        })
    };
    let removal = Overlay::removing_a_tombstone(TombstoneRecord {
        object,
        object_birth: record.object_birth,
        range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
        death_txg: intent_txg,
    });
    let removal_closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
        readers.iter().zip(deltas).all(|(reader, delta)| reader.effect != UnlinkEffectArm::AtIntent || delta[DELTA_RECORDS_WHOSE_INTENT_IS_PUBLISHED] == 1)
    };
    let mut positive_controls = vec![
        branch.positive_control("PC-O-plant", &plant, crash_publish, format!("object={object}"), &plant_closed_form),
        branch.positive_control("PC-O-remove", &removal, crash_publish, format!("object={object} intent_txg={intent_txg}"), &removal_closed_form),
    ];
    positive_controls.extend(branch.erasure_positive_controls(crash_publish));
    branch.scanning = true;
    branch.recover_after_the_crash();
    branch.walk_the_deletion_queue_empty();
    branch.publish(&Operation::NewInodes { count: INTENT_CRASH_NEW_INODES_AFTER_RECOVERY });
    let recovery_trajectories = trajectories_of(&branch.scans, every_reader().len());
    let recovery_paths_agree = branch.group_index_equals_the_full_scan(branch.publish_sequence);
    Some(CrashBranchOutcome { crash_counts, crash_paths_agree, positive_controls, recovery_trajectories, recovery_paths_agree })
}

/// 一个前缀上的一个 (u, b)：在意图那一版与每个不是最后一批的批上各分一支。
fn run_crash_branches(prefix: &Simulation, object: u64, data_units: u64, batch: u64) -> Vec<CrashBranchOutcome> {
    let mut simulation = prefix.clone();
    simulation.batch_data_units = batch;
    simulation.apply(UserChange::WriteFile { object, data_units });
    if !simulation.publish(&Operation::UnlinkIntent { object }) {
        return Vec::new();
    }
    let mut outcomes = Vec::new();
    while simulation.version.deleted_queue == Some(object) {
        outcomes.extend(run_one_crash_branch(&simulation, object));
        let step = simulation.next_unlink_step(object);
        if !simulation.publish(&step) {
            break;
        }
    }
    outcomes
}

const INTENT_CRASH_DATA_UNITS: [u64; 3] = [3, 145, 300];
const INTENT_CRASH_BATCHES: [u64; 2] = [1, PRODUCT_UNLINK_BATCH_DATA_UNITS];

/// 一个 (前缀, 写路径格, u, b) 上，每个读者跨全部崩溃支的汇总。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CrashSummary {
    crash_points: u64,
    pub(super) inode_missed: Extremes,
    pub(super) extent_missed: Extremes,
    inode_missed_positive: u64,
    extent_missed_positive: u64,
    pub(super) live_records_beyond_the_truth: Extremes,
    pub(super) records_whose_intent_is_published: Extremes,
    records_whose_intent_is_published_positive: u64,
    recovery_inode_resurrected_peak: u64,
    recovery_extent_resurrected_peak: u64,
    recovery_container_resurrected_peak: u64,
}

#[derive(Clone, Debug)]
struct SegmentThreeResult {
    pub(super) item: SegmentThreeItem,
    pub(super) lines: Vec<String>,
    summaries: Vec<((u64, u64), Vec<CrashSummary>)>,
    every_path_agrees: bool,
    positive_controls_agree: bool,
    positive_controls_closed_form: bool,
    prefix_found: bool,
}

fn run_segment_three_item(item: SegmentThreeItem) -> SegmentThreeResult {
    let readers = every_reader();
    let label = format!("prefix={} {}", item.prefix.label(), item.cell.label());
    let Some((prefix, object)) = build_crash_prefix(item.prefix, item.cell) else {
        return SegmentThreeResult {
            item,
            lines: vec![format!("name=crash_prefix {label} found=false")],
            summaries: Vec::new(),
            every_path_agrees: true,
            positive_controls_agree: true,
            positive_controls_closed_form: true,
            prefix_found: false,
        };
    };
    let mut lines = vec![format!("name=crash_prefix {label} found=true object={object} prefix_publishes={}", prefix.publishes)];
    let mut summaries = Vec::new();
    let mut every_path_agrees = true;
    let mut positive_controls_agree = true;
    let mut positive_controls_closed_form = true;
    for data_units in INTENT_CRASH_DATA_UNITS {
        for batch in INTENT_CRASH_BATCHES {
            let outcomes = run_crash_branches(&prefix, object, data_units, batch);
            let mut per_reader = vec![CrashSummary::default(); readers.len()];
            for outcome in &outcomes {
                every_path_agrees &= outcome.crash_paths_agree && outcome.recovery_paths_agree;
                positive_controls_agree &= outcome.positive_controls.iter().all(|control| control.paths_agree);
                positive_controls_closed_form &= outcome.positive_controls.iter().all(|control| control.closed_form_holds);
                for (position, summary) in per_reader.iter_mut().enumerate() {
                    let counts = outcome.crash_counts[position];
                    summary.crash_points += 1;
                    summary.inode_missed.observe(counts.inode_missed);
                    summary.extent_missed.observe(counts.extent_missed);
                    summary.inode_missed_positive += u64::from(counts.inode_missed > 0);
                    summary.extent_missed_positive += u64::from(counts.extent_missed > 0);
                    summary.live_records_beyond_the_truth.observe(counts.live_records_beyond_the_truth);
                    summary.records_whose_intent_is_published.observe(counts.records_whose_intent_is_published);
                    summary.records_whose_intent_is_published_positive += u64::from(counts.records_whose_intent_is_published > 0);
                    let recovery = &outcome.recovery_trajectories[position].by_scope[SCOPE_ALL];
                    summary.recovery_inode_resurrected_peak = summary.recovery_inode_resurrected_peak.max(recovery[QUANTITY_INODE_RESURRECTED].peak);
                    summary.recovery_extent_resurrected_peak = summary.recovery_extent_resurrected_peak.max(recovery[QUANTITY_EXTENT_RESURRECTED].peak);
                    summary.recovery_container_resurrected_peak = summary.recovery_container_resurrected_peak.max(recovery[QUANTITY_CONTAINER_RESURRECTED].peak);
                }
            }
            for (reader, summary) in readers.iter().zip(&per_reader) {
                lines.push(format!(
                    "name=crash_summary {label} u={data_units} batch={batch} {} crash_points={} {} inode_m_positive={} {} extent_m_positive={} {} {} o2_positive={} recovery_inode_r_peak={} recovery_extent_r_peak={} recovery_container_r_peak={}",
                    reader.label(),
                    summary.crash_points,
                    summary.inode_missed.fields("inode_m"),
                    summary.inode_missed_positive,
                    summary.extent_missed.fields("extent_m"),
                    summary.extent_missed_positive,
                    summary.live_records_beyond_the_truth.fields("o1"),
                    summary.records_whose_intent_is_published.fields("o2"),
                    summary.records_whose_intent_is_published_positive,
                    summary.recovery_inode_resurrected_peak,
                    summary.recovery_extent_resurrected_peak,
                    summary.recovery_container_resurrected_peak,
                ));
            }
            let controls: Vec<&PositiveControlOutcome> = outcomes.iter().flat_map(|outcome| outcome.positive_controls.iter()).collect();
            for control_identifier in ["PC-O-plant", "PC-O-remove", "PC-M-data", "PC-M-leaf"] {
                let outcomes_of_the_control: Vec<&&PositiveControlOutcome> = controls.iter().filter(|control| control.control_identifier == control_identifier).collect();
                lines.push(format!(
                    "name=positive_control id={control_identifier} {label} u={data_units} batch={batch} crash_points={} paths_agree={} closed_form_holds={}",
                    outcomes_of_the_control.len(),
                    outcomes_of_the_control.iter().filter(|control| control.paths_agree).count(),
                    outcomes_of_the_control.iter().filter(|control| control.closed_form_holds).count(),
                ));
            }
            summaries.push(((data_units, batch), per_reader));
        }
    }
    SegmentThreeResult { item, lines, summaries, every_path_agrees, positive_controls_agree, positive_controls_closed_form, prefix_found: true }
}

fn segment_three_verdict_lines(results: &[SegmentThreeResult]) -> Vec<String> {
    let readers = every_reader();
    let mut lines = Vec::new();
    for result in results.iter().filter(|result| result.prefix_found) {
        for ((data_units, batch), per_reader) in &result.summaries {
            for (reader, summary) in readers.iter().zip(per_reader).filter(|(reader, _)| reader.is_a_main_cell()) {
                let key = format!("prefix={} {} u={data_units} batch={batch} {}", result.item.prefix.label(), result.item.cell.label(), reader.label());
                let extra_objects = summary.records_whose_intent_is_published.largest.unwrap_or(0);
                let extra_status = match (reader.effect, extra_objects > 0) {
                    (UnlinkEffectArm::AtIntent, true) => "fl7_intent_effect_shows_extra_objects",
                    (UnlinkEffectArm::AtLastBatchWritten | UnlinkEffectArm::AtLastBatchRead, true) => "fl6_last_batch_effect_shows_extra_objects",
                    (UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchWritten | UnlinkEffectArm::AtLastBatchRead, false) => "zero",
                };
                lines.push(format!(
                    "name=verdict question=Q-X3-O2 {key} {} positive_crash_points={} status={extra_status}",
                    summary.records_whose_intent_is_published.fields("o2"),
                    summary.records_whose_intent_is_published_positive,
                ));
                let missed = summary.inode_missed.largest.unwrap_or(0).max(summary.extent_missed.largest.unwrap_or(0));
                let missed_status = if reader.effect == UnlinkEffectArm::AtIntent { "reported" } else { status_by_threshold(missed, "fl5_leaks_after_the_crash") };
                lines.push(format!(
                    "name=verdict question=Q-X3-4 {key} {} {} status={missed_status}",
                    summary.inode_missed.fields("inode_m"),
                    summary.extent_missed.fields("extent_m"),
                ));
                lines.push(format!(
                    "name=verdict question=Q-X3-5 {key} recovery_inode_r_peak={} recovery_extent_r_peak={} recovery_container_r_peak={} status=incidental",
                    summary.recovery_inode_resurrected_peak,
                    summary.recovery_extent_resurrected_peak,
                    summary.recovery_container_resurrected_peak,
                ));
            }
        }
    }
    lines
}

pub(super) fn run_segment_three(threads: usize) -> Vec<String> {
    let items = segment_three_items();
    let mut lines = vec![format!(
        "name=config experiment=E178 segment=three work_items={} data_units=3,145,300 batches=1,144 readers={} prefix_inode_count={INTENT_CRASH_INODE_COUNT} new_inodes_after_recovery={INTENT_CRASH_NEW_INODES_AFTER_RECOVERY}",
        items.len(),
        every_reader().len(),
    )];
    let results = run_in_parallel(&items, threads, run_segment_three_item);
    for result in &results {
        lines.extend(result.lines.iter().cloned());
    }
    lines.extend(segment_three_verdict_lines(&results));
    let every_path_agrees = results.iter().all(|result| result.every_path_agrees);
    let controls_agree = results.iter().all(|result| result.positive_controls_agree);
    let controls_closed_form = results.iter().all(|result| result.positive_controls_closed_form);
    let prefixes_missing = results.iter().filter(|result| !result.prefix_found).count();
    lines.push(format!(
        "name=verdict question=summary v2_positive_controls_paths_agree={controls_agree} positive_controls_closed_form_holds={controls_closed_form} v3_paths_agree={every_path_agrees} prefixes_missing={prefixes_missing}"
    ));
    lines
}

#[cfg(test)]
#[path = "crash_prefix_runs_and_verdicts_tests.rs"]
mod tests;
