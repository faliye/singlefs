//! 每次扫描都读的那一段：工作项、检查、判定行与稳定性行，以及整趟。

use super::*;

// ---------------------------------------------------------------------------------------------
// 第二段（登记 5.8）：H51、F10、F11 全部读者每次扫描都读；Q-B1、Q-B2、Q-X2-1 至 Q-X2-7、Q-X3-1 至 Q-X3-3
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SegmentTwoItem {
    pub(super) history: History,
    pub(super) cell: WritePathCell,
}

fn rollback_over_a_gap_histories() -> Vec<History> {
    GAP_EVENTS.iter().map(|event| History::RollBackOverAGap { event: *event }).collect()
}

pub(super) fn segment_two_items() -> Vec<SegmentTwoItem> {
    let mut items = Vec::new();
    for history in histories_replayed_from_the_e177_segment_one().into_iter().chain(many_objects_with_data_histories()).chain(rollback_over_a_gap_histories()) {
        for leaf_capacity in LEAF_CAPACITIES {
            for merge_threshold in merge_thresholds_of(leaf_capacity) {
                for borrow_arm in BORROW_ARMS {
                    items.push(SegmentTwoItem { history, cell: WritePathCell { borrow_arm, leaf_capacity, merge_threshold } });
                }
            }
        }
    }
    items
}

pub(super) fn reader_position(wanted: Reader) -> usize {
    every_reader().iter().position(|reader| *reader == wanted).expect("读者表里有它")
}

/// Q-X2-6、Q-X2-7：同一段历史、同一次发布、同一条 X3 臂下，T 那条墓碑写流比 Z 那条多写的码 3、码 2（第一节读法 3）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ExtraWrites {
    extra_code_three: Extremes,
    extra_code_two: Extremes,
    multi_write_publishes: u64,
    pub(super) publishes: u64,
    extra_without_one_and_one: u64,
}

fn extra_writes_of(publish_records: &[PublishRecord], effect: UnlinkEffectArm) -> ExtraWrites {
    let with_ranges = tombstone_stream_of(ShrinkArm::RangeTombstone, effect).position();
    let without_ranges = tombstone_stream_of(ShrinkArm::RecordSize, effect).position();
    let mut extra = ExtraWrites::default();
    for record in publish_records {
        extra.publishes += 1;
        let code_three = difference_or_marker(record.written.tombstone_containers[with_ranges], record.written.tombstone_containers[without_ranges]);
        let code_two = difference_or_marker(record.written.tombstone_roots[with_ranges], record.written.tombstone_roots[without_ranges]);
        if code_three > 0 || code_two > 0 {
            extra.multi_write_publishes += 1;
            extra.extra_code_three.observe(code_three);
            extra.extra_code_two.observe(code_two);
            extra.extra_without_one_and_one += u64::from(code_three != 1 || code_two != 1);
        }
    }
    extra
}

/// A4 的 F8 逐段值（第七节；`anchors.out` 的 `T_extra_*` 列）：E-I 下多写发布数 = [删前覆盖写且 n_u > 1]，E-L 两臂下再加不是最后一批的批数。
fn expected_extra_write_publishes(history: History, effect: UnlinkEffectArm) -> Option<u64> {
    let History::LargeFileRewriteAndDelete { data_units, overwrite_before_unlink, batch, .. } = history else {
        return None;
    };
    let shrink_publishes = u64::from(overwrite_before_unlink && data_units > 1);
    let data_units_at_the_intent = if overwrite_before_unlink { 1 } else { data_units };
    let batches_that_are_not_the_last = data_units_at_the_intent.div_ceil(batch).max(1) - 1;
    Some(match effect {
        UnlinkEffectArm::AtIntent => shrink_publishes,
        UnlinkEffectArm::AtLastBatchWritten | UnlinkEffectArm::AtLastBatchRead => shrink_publishes + batches_that_are_not_the_last,
    })
}

#[derive(Clone, Debug)]
pub(super) struct SegmentTwoResult {
    pub(super) item: SegmentTwoItem,
    pub(super) trajectories: Vec<ReaderTrajectory>,
    extra_writes: Vec<(UnlinkEffectArm, ExtraWrites)>,
    pub(super) positive_controls: Vec<PositiveControlOutcome>,
    pub(super) rollback: Option<RollbackOutcome>,
    pub(super) assertions: AssertionTally,
    pub(super) lines: Vec<String>,
    pub(super) two_runs_identical: bool,
    pub(super) last_scan_paths_agree: bool,
    path_covered: bool,
}

/// 每段第二遍跑完之后判的断言与对照（A4、A7、A9、K4、PC-B1、PC-G0、PC-B2），记进断言账、写一行产物。
fn segment_two_checks(item: SegmentTwoItem, simulation: &Simulation, rollback: Option<RollbackOutcome>, assertions: &mut AssertionTally) -> Vec<String> {
    let mut lines = Vec::new();
    let label = format!("{} {}", item.history.label(), item.cell.label());
    let last_counts = simulation.scans.last().map(|record| record.counts.clone()).unwrap_or_default();
    // PC-B1：B-off-ret 在 W = ∞ 下期末的容器 r = 身份消失次数（期末不在真值里的退役过的身份数）；W = 23 下每次身份消失那次扫描容器 r ≥ 1。
    let final_identities: HashSet<ContainerIdentity> = simulation.version.leaves.iter().map(|leaf| leaf.identity).collect();
    let disappeared: BTreeSet<ContainerIdentity> = simulation
        .publish_records
        .iter()
        .flat_map(|record| record.retired_identities.iter().copied())
        .filter(|identity| !final_identities.contains(identity))
        .collect();
    let without_retirement_forever = reader_position(Reader { base: BaseArm::WithoutRetirement, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever });
    let without_retirement_ring = reader_position(Reader { base: BaseArm::WithoutRetirement, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::RingWindow });
    let final_container_resurrected = last_counts.get(without_retirement_forever).map_or(0, |counts| counts.container_resurrected);
    let disappearance_scans_without_resurrection = simulation
        .publish_records
        .iter()
        .zip(&simulation.scans)
        .filter(|(record, _)| !record.retired_identities.is_empty())
        .filter(|(_, scan)| scan.counts[without_retirement_ring].container_resurrected == 0)
        .count();
    if !disappeared.is_empty() {
        let holds = final_container_resurrected == length_as_u64(disappeared.len()) && disappearance_scans_without_resurrection == 0;
        assertions.judge("PC-B1_without_retirement_resurrects_every_disappeared_identity", holds);
        lines.push(format!(
            "name=positive_control id=PC-B1 {label} disappeared_identities={} final_container_r_inf={final_container_resurrected} disappearance_scans_without_r_23={disappearance_scans_without_resurrection} holds={holds}",
            disappeared.len()
        ));
        if let (History::BuildAndDeleteEmptyInodes { inode_count, .. }, 0) = (item.history, item.cell.merge_threshold) {
            let expected = inode_count.div_ceil(item.cell.leaf_capacity) - 1;
            assertions.judge("A9_without_retirement_final_container_r", final_container_resurrected == expected);
        }
    }
    // PC-G0（A3）：F8 删前覆盖写、n_u > 1 的段，覆盖写之后那次扫描 G0 的 extent r = n_u − 1。
    if let History::LargeFileRewriteAndDelete { data_units, overwrite_before_unlink: true, .. } = item.history {
        if data_units > 1 {
            let overwrite_scan = simulation.publish_records.iter().zip(&simulation.scans).find(|(record, _)| record.shrink.is_some()).map(|(_, scan)| scan);
            let observed: Vec<u64> = overwrite_scan.map_or_else(Vec::new, |scan| {
                every_reader()
                    .iter()
                    .zip(&scan.counts)
                    .filter(|(reader, _)| reader.shrink == ShrinkArm::ObjectGateOnly)
                    .map(|(_, counts)| counts.extent_resurrected)
                    .collect()
            });
            let holds = !observed.is_empty() && observed.iter().all(|value| *value == data_units - 1);
            assertions.judge("A3_object_gate_only_resurrects_the_overwritten_units", holds);
            lines.push(format!("name=positive_control id=PC-G0 {label} expected={} observed={observed:?} holds={holds}", data_units - 1));
        }
    }
    // A4：T 每次多写的发布多写码 3 一个、码 2 一个；F8 的多写发布数逐段等于登记的式子。
    for effect in UNLINK_EFFECT_ARMS {
        let extra = extra_writes_of(&simulation.publish_records, effect);
        assertions.judge("A4_extra_writes_are_one_and_one", extra.extra_without_one_and_one == 0);
        if let Some(expected) = expected_extra_write_publishes(item.history, effect) {
            assertions.judge("A4_extra_write_publishes", extra.multi_write_publishes == expected);
        }
    }
    // A7：F8（不删前覆盖写）E-I 下 unlink 途中每次扫描 inode m = 1、extent m = 这一版剩下的单元数（B-on 的四个 E-I 主格读者）。
    if let History::LargeFileRewriteAndDelete { overwrite_before_unlink: false, .. } = item.history {
        let intent_readers: Vec<usize> = every_reader()
            .iter()
            .enumerate()
            .filter(|(_, reader)| reader.is_a_main_cell() && reader.effect == UnlinkEffectArm::AtIntent)
            .map(|(position, _)| position)
            .collect();
        let mut sequence = Vec::new();
        for scan in simulation.scans.iter().filter(|scan| scan.inside_an_unlink) {
            let counts = scan.counts[intent_readers[0]];
            sequence.push(counts.extent_missed);
            let every_reader_agrees = intent_readers.iter().all(|position| scan.counts[*position].inode_missed == 1 && scan.counts[*position].extent_missed == counts.extent_missed);
            assertions.judge("A7_intent_effect_misses_the_record_and_every_remaining_unit", every_reader_agrees);
        }
        lines.push(format!("name=anchor id=A7 {label} in_window_scans={} extent_m_sequence_head={:?}", sequence.len(), sequence.iter().take(3).collect::<Vec<&u64>>()));
    }
    // K4：E-I、E-L-w 下每次 unlink 恰好一条整对象墓碑。
    let unlinks = length_as_u64(simulation.intent_txgs.len());
    let whole_at_intent = simulation.version.tombstone_trees[TombstoneStream::WholeAtIntent.position()].record_count();
    let whole_at_final = simulation.version.tombstone_trees[TombstoneStream::WholeAtFinal.position()].record_count();
    if rollback.is_none() {
        assertions.judge("K4_one_whole_object_tombstone_per_unlink", whole_at_intent == unlinks && whole_at_final == unlinks);
    }
    // PC-B2：F11 回退之后 B-off-lin 的容器 m 与缺口里退役、而在 R_old 里有的身份个数并排报（不当门槛，两路对拍在 PC 行与末次扫描上）。
    if let Some(outcome) = rollback {
        let lineage_ignored = reader_position(Reader { base: BaseArm::RetirementIgnoringLineage, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever });
        let on_lineage = reader_position(Reader { base: BaseArm::RetirementOnLineage, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window: ReadableWindow::Forever });
        let after = |position: usize| simulation.scans.iter().filter(|scan| scan.after_a_rollback).map(|scan| scan.counts[position].container_missed).max().unwrap_or(0);
        lines.push(format!(
            "name=positive_control id=PC-B2 {label} event_found={} rollback_distance={} too_far={} expected_from_the_write_path={} b_off_lin_container_m_peak_after_rollback={} b_on_container_m_peak_after_rollback={} retirement_records_after_rollback={}",
            outcome.event_found,
            outcome.distance_in_publishes,
            outcome.too_far,
            outcome.retired_in_the_gap_and_present_in_the_old_version,
            after(lineage_ignored),
            after(on_lineage),
            outcome.retirement_records_after_the_rollback,
        ));
    }
    lines
}

pub(super) fn run_segment_two_item(item: SegmentTwoItem) -> SegmentTwoResult {
    let (first, _) = simulate(item.history, item.cell, true);
    let plan = positive_control_plan_of(&first.scans);
    let first_scans = first.scans;
    let mut second = Simulation::start(item.cell, item.history.batch(), true);
    second.positive_control_plan = plan;
    let rollback = run_history(item.history, &mut second);
    let two_runs_identical = second.scans == first_scans;
    let last_publish = second.publish_sequence;
    let last_scan_paths_agree = second.group_index_equals_the_full_scan(last_publish);
    let mut positive_controls = second.positive_control_outcomes.clone();
    positive_controls.extend(second.erasure_positive_controls(last_publish));
    let mut assertions = second.assertions.clone();
    let readers = every_reader();
    let trajectories = trajectories_of(&second.scans, readers.len());
    let extra_writes: Vec<(UnlinkEffectArm, ExtraWrites)> =
        UNLINK_EFFECT_ARMS.iter().map(|effect| (*effect, extra_writes_of(&second.publish_records, *effect))).collect();
    let label = format!("{} {}", item.history.label(), item.cell.label());
    let (borrows, absorbs, drops) = second.count_merges();
    let path_covered = second.moved_then_shortened > 0 && second.moved_then_unlinked > 0 && borrows > 0;
    let mut lines = vec![format!(
        "name=simulation {label} publishes={} scans={} refused_steps={} borrows={borrows} absorbs={absorbs} drops={drops} moved_then_shortened={} moved_then_unlinked={} retirement_records={} does_not_fit_at={} two_runs_identical={two_runs_identical} last_scan_paths_agree={last_scan_paths_agree}",
        second.publishes,
        second.scans.len(),
        second.refused_steps,
        second.moved_then_shortened,
        second.moved_then_unlinked,
        second.total_written().retirement_records,
        second.does_not_fit_at.map_or_else(|| "none".to_string(), |txg| txg.to_string()),
    )];
    let scopes_reported = if rollback.is_some() { SCAN_SCOPES.len() } else { 3 };
    for (reader, trajectory) in readers.iter().zip(&trajectories) {
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
    for (effect, extra) in &extra_writes {
        lines.push(format!(
            "name=extra_writes {label} effect={} multi_write_publishes={} publishes={} {} {} not_one_and_one={}",
            effect.label(),
            extra.multi_write_publishes,
            extra.publishes,
            extra.extra_code_three.fields("extra_code3"),
            extra.extra_code_two.fields("extra_code2"),
            extra.extra_without_one_and_one,
        ));
    }
    for outcome in &positive_controls {
        lines.push(outcome.line(&label));
    }
    lines.extend(segment_two_checks(item, &second, rollback, &mut assertions));
    SegmentTwoResult {
        item,
        trajectories,
        extra_writes,
        positive_controls,
        rollback,
        assertions,
        lines,
        two_runs_identical,
        last_scan_paths_agree,
        path_covered,
    }
}

/// 判定格：写路径格 × 批大小（第八节的 W、θ、容量、b 都在这一格的键里或读者里）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct VerdictKey {
    pub(super) cell: WritePathCell,
    pub(super) batch: u64,
}

fn aggregate_trajectories(results: &[SegmentTwoResult], family_filter: &dyn Fn(&str) -> bool) -> BTreeMap<VerdictKey, Vec<ReaderTrajectory>> {
    let reader_count = every_reader().len();
    let mut aggregated: BTreeMap<VerdictKey, Vec<ReaderTrajectory>> = BTreeMap::new();
    for result in results.iter().filter(|result| family_filter(result.item.history.family())) {
        let key = VerdictKey { cell: result.item.cell, batch: result.item.history.batch() };
        let entry = aggregated.entry(key).or_insert_with(|| vec![ReaderTrajectory::default(); reader_count]);
        for (aggregate, trajectory) in entry.iter_mut().zip(&result.trajectories) {
            for scope_position in 0..SCAN_SCOPES.len() {
                for quantity_position in 0..COUNTED_QUANTITIES.len() {
                    aggregate.by_scope[scope_position][quantity_position].absorb(trajectory.by_scope[scope_position][quantity_position]);
                }
            }
            aggregate.read_out_empty_scans += trajectory.read_out_empty_scans;
        }
    }
    aggregated
}

pub(super) const SCOPE_ALL: usize = 0;
pub(super) const SCOPE_INSIDE_UNLINK: usize = 1;
pub(super) const SCOPE_OUTSIDE_UNLINK: usize = 2;
pub(super) const SCOPE_AFTER_ROLLBACK_OUTSIDE_UNLINK: usize = 4;
pub(super) const QUANTITY_INODE_RESURRECTED: usize = 0;
pub(super) const QUANTITY_INODE_MISSED: usize = 1;
pub(super) const QUANTITY_EXTENT_RESURRECTED: usize = 2;
pub(super) const QUANTITY_EXTENT_MISSED: usize = 3;
pub(super) const QUANTITY_CONTAINER_RESURRECTED: usize = 4;
pub(super) const QUANTITY_CONTAINER_MISSED: usize = 5;

pub(super) fn theta_class_of(cell: WritePathCell) -> usize {
    merge_thresholds_of(cell.leaf_capacity).iter().position(|threshold| *threshold == cell.merge_threshold).expect("θ 取自三档之一")
}

/// 一个带门槛的判定，记下它所在的取样点，供第八节的几何敏感性逐对比。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct JudgedStatus {
    pub(super) question_and_arms: String,
    pub(super) borrow_arm: BorrowArm,
    pub(super) leaf_capacity: u64,
    pub(super) theta_class: usize,
    pub(super) batch: u64,
    pub(super) window: ReadableWindow,
    pub(super) status: &'static str,
}

/// 第六节的门槛：峰值大于它就判那一格「复活 / 误判死亡 / 漏活」（第八节「判定函数把门槛当参数」）。
const JUDGEMENT_THRESHOLD: u64 = 0;

const fn status_above(peak: u64, threshold: u64, positive: &'static str) -> &'static str {
    if peak > threshold {
        positive
    } else {
        "zero"
    }
}

pub(super) const fn status_by_threshold(peak: u64, positive: &'static str) -> &'static str {
    status_above(peak, JUDGEMENT_THRESHOLD, positive)
}

fn segment_two_verdict_lines(results: &[SegmentTwoResult]) -> (Vec<String>, Vec<JudgedStatus>) {
    let mut lines = Vec::new();
    let mut statuses = Vec::new();
    let all_families = aggregate_trajectories(results, &|_| true);
    let rollback_only = aggregate_trajectories(results, &|family| family == "F11");
    for (key, trajectories) in &all_families {
        let key_label = format!("{} batch={}", key.cell.label(), key.batch);
        for window in READABLE_WINDOWS {
            let mut judged = |question_and_arms: String, status: &'static str| {
                statuses.push(JudgedStatus {
                    question_and_arms,
                    borrow_arm: key.cell.borrow_arm,
                    leaf_capacity: key.cell.leaf_capacity,
                    theta_class: theta_class_of(key.cell),
                    batch: key.batch,
                    window,
                    status,
                });
            };
            let base_reader = reader_position(Reader { base: BaseArm::RetirementOnLineage, shrink: ShrinkArm::RecordSize, effect: UnlinkEffectArm::AtIntent, window });
            let base_container_resurrected = trajectories[base_reader].by_scope[SCOPE_ALL][QUANTITY_CONTAINER_RESURRECTED];
            let base_resurrection_status = status_by_threshold(base_container_resurrected.peak, "resurrects");
            judged("Q-B1".to_string(), base_resurrection_status);
            let rollback_trajectories = rollback_only.get(key);
            let after_rollback = |quantity: usize| {
                rollback_trajectories.map_or_else(QuantityTrajectory::default, |rollback_family_trajectories| rollback_family_trajectories[base_reader].by_scope[SCOPE_AFTER_ROLLBACK_OUTSIDE_UNLINK][quantity])
            };
            let base_misjudged_death_status = status_by_threshold(after_rollback(QUANTITY_CONTAINER_MISSED).peak, "misjudges_death");
            if rollback_trajectories.is_some() {
                judged("Q-B2".to_string(), base_misjudged_death_status);
            }
            let excluded_by_fl1 = base_resurrection_status != "zero" || (rollback_trajectories.is_some() && base_misjudged_death_status != "zero");
            lines.push(format!(
                "name=verdict question=Q-B1 {key_label} window={} {} status={base_resurrection_status}",
                window.label(),
                base_container_resurrected.fields("container_r")
            ));
            lines.push(format!(
                "name=verdict question=Q-B2 {key_label} window={} rollback_histories={} {} {} status={}",
                window.label(),
                rollback_trajectories.is_some(),
                after_rollback(QUANTITY_CONTAINER_MISSED).fields("container_m"),
                after_rollback(QUANTITY_INODE_MISSED).fields("inode_m"),
                if rollback_trajectories.is_some() { base_misjudged_death_status } else { "not_run" }
            ));
            for effect in UNLINK_EFFECT_ARMS {
                for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
                    let reader = Reader { base: BaseArm::RetirementOnLineage, shrink, effect, window };
                    let trajectory = &trajectories[reader_position(reader)];
                    let arms = format!("shrink={} effect={}", shrink.label(), effect.label());
                    let resurrection = trajectory.by_scope[SCOPE_ALL][QUANTITY_EXTENT_RESURRECTED];
                    let resurrection_question = if shrink == ShrinkArm::RecordSize { "Q-X2-1" } else { "Q-X2-2" };
                    let resurrection_status = status_by_threshold(resurrection.peak, "resurrects");
                    judged(format!("{resurrection_question} {arms}"), resurrection_status);
                    lines.push(format!(
                        "name=verdict question={resurrection_question} {key_label} {} {} {} status={resurrection_status} fl1_cell_excluded={excluded_by_fl1}",
                        reader.label(),
                        resurrection.fields("extent_r"),
                        trajectory.by_scope[SCOPE_ALL][QUANTITY_INODE_RESURRECTED].fields("inode_r"),
                    ));
                    let misjudged = trajectory.by_scope[SCOPE_OUTSIDE_UNLINK][QUANTITY_EXTENT_MISSED];
                    let misjudged_question = if shrink == ShrinkArm::RecordSize { "Q-X2-3" } else { "Q-X2-4" };
                    let misjudged_status = status_by_threshold(misjudged.peak, "misjudges_death");
                    judged(format!("{misjudged_question} {arms}"), misjudged_status);
                    lines.push(format!(
                        "name=verdict question={misjudged_question} {key_label} {} {} {} status={misjudged_status} fl1_cell_excluded={excluded_by_fl1}",
                        reader.label(),
                        misjudged.fields("extent_m_outside_unlink"),
                        trajectory.by_scope[SCOPE_OUTSIDE_UNLINK][QUANTITY_INODE_MISSED].fields("inode_m_outside_unlink"),
                    ));
                    let inside_extent_missed = trajectory.by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_EXTENT_MISSED];
                    let inside_inode_missed = trajectory.by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_MISSED];
                    if effect != UnlinkEffectArm::AtIntent {
                        let batch_status = status_by_threshold(inside_extent_missed.peak, "misjudges_in_batches");
                        judged(format!("Q-X2-5 {arms}"), batch_status);
                        lines.push(format!(
                            "name=verdict question=Q-X2-5 {key_label} {} {} status={batch_status} fl1_cell_excluded={excluded_by_fl1}",
                            reader.label(),
                            inside_extent_missed.fields("extent_m_inside_unlink"),
                        ));
                    }
                    for (question, quantity) in [("Q-X3-1", inside_inode_missed), ("Q-X3-2", inside_extent_missed)] {
                        let status = if effect == UnlinkEffectArm::AtIntent { "reported" } else { status_by_threshold(quantity.peak, "leaks") };
                        if effect != UnlinkEffectArm::AtIntent {
                            judged(format!("{question} {arms}"), status);
                        }
                        lines.push(format!(
                            "name=verdict question={question} {key_label} {} {} status={status} fl1_cell_excluded={excluded_by_fl1}",
                            reader.label(),
                            quantity.fields("inside_unlink"),
                        ));
                    }
                    lines.push(format!(
                        "name=verdict question=Q-X3-3 {key_label} {} {} {} status=incidental",
                        reader.label(),
                        trajectory.by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_INODE_RESURRECTED].fields("inode_r_inside_unlink"),
                        trajectory.by_scope[SCOPE_INSIDE_UNLINK][QUANTITY_EXTENT_RESURRECTED].fields("extent_r_inside_unlink"),
                    ));
                }
            }
        }
    }
    (lines, statuses)
}

/// Q-X2-6、Q-X2-7：T 每次多写的单元数（每份），与多写发布占比（按族、X3 臂分列；比例按百万分之一计）。
fn extra_write_verdict_lines(results: &[SegmentTwoResult]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut by_cell_family: BTreeMap<(WritePathCell, &'static str, UnlinkEffectArm), ExtraWrites> = BTreeMap::new();
    for result in results {
        for (effect, extra) in &result.extra_writes {
            let entry = by_cell_family.entry((result.item.cell, result.item.history.family(), *effect)).or_default();
            entry.extra_code_three.absorb(extra.extra_code_three);
            entry.extra_code_two.absorb(extra.extra_code_two);
            entry.multi_write_publishes += extra.multi_write_publishes;
            entry.publishes += extra.publishes;
            entry.extra_without_one_and_one += extra.extra_without_one_and_one;
        }
    }
    let mut by_cell: BTreeMap<(WritePathCell, UnlinkEffectArm), ExtraWrites> = BTreeMap::new();
    for ((cell, family, effect), extra) in &by_cell_family {
        let share = (extra.multi_write_publishes * 1_000_000).checked_div(extra.publishes).unwrap_or(0);
        lines.push(format!(
            "name=verdict question=Q-X2-7 {} family={family} effect={} multi_write_publishes={} publishes={} share_ppm={share} judgement=reported",
            cell.label(),
            effect.label(),
            extra.multi_write_publishes,
            extra.publishes,
        ));
        let entry = by_cell.entry((*cell, *effect)).or_default();
        entry.extra_code_three.absorb(extra.extra_code_three);
        entry.extra_code_two.absorb(extra.extra_code_two);
        entry.multi_write_publishes += extra.multi_write_publishes;
        entry.publishes += extra.publishes;
        entry.extra_without_one_and_one += extra.extra_without_one_and_one;
    }
    for ((cell, effect), extra) in &by_cell {
        lines.push(format!(
            "name=verdict question=Q-X2-6 {} effect={} {} {} not_one_and_one={} judgement=reported",
            cell.label(),
            effect.label(),
            extra.extra_code_three.fields("extra_code3_per_multi_write_publish"),
            extra.extra_code_two.fields("extra_code2_per_multi_write_publish"),
            extra.extra_without_one_and_one,
        ));
    }
    lines
}

/// 第八节：同一个判定在一对方向相反的取样点上翻不翻面（W、θ 四分与二分、容量 233 与 16、b 1 与 144）。
pub(super) fn stability_lines(statuses: &[JudgedStatus]) -> Vec<String> {
    let mut by_question: BTreeMap<&str, Vec<&JudgedStatus>> = BTreeMap::new();
    for status in statuses {
        by_question.entry(status.question_and_arms.as_str()).or_default().push(status);
    }
    let mut lines = Vec::new();
    for (question_and_arms, judged) in by_question {
        let flips_along = |differs_only_in_the_knob: &dyn Fn(&JudgedStatus, &JudgedStatus) -> bool| {
            let mut pairs = 0_u64;
            let mut flips = 0_u64;
            for first in &judged {
                for second in &judged {
                    if first < second && differs_only_in_the_knob(first, second) {
                        pairs += 1;
                        flips += u64::from(first.status != second.status);
                    }
                }
            }
            (pairs, flips)
        };
        let same_except = |first: &JudgedStatus, second: &JudgedStatus, window: bool, theta: bool, capacity: bool, batch: bool| {
            first.borrow_arm == second.borrow_arm
                && (window || first.window == second.window)
                && (theta || first.theta_class == second.theta_class)
                && (capacity || first.leaf_capacity == second.leaf_capacity)
                && (batch || first.batch == second.batch)
        };
        let window_flips = flips_along(&|first, second| same_except(first, second, true, false, false, false) && first.window != second.window);
        let theta_flips = flips_along(&|first, second| {
            same_except(first, second, false, true, false, false) && first.theta_class == 1 && second.theta_class == 2
        });
        let capacity_flips = flips_along(&|first, second| same_except(first, second, false, false, true, false) && first.leaf_capacity != second.leaf_capacity);
        let batch_flips = flips_along(&|first, second| same_except(first, second, false, false, false, true) && first.batch != second.batch);
        let every_status: BTreeSet<&str> = judged.iter().map(|status| status.status).collect();
        let stable = window_flips.1 == 0 && theta_flips.1 == 0 && capacity_flips.1 == 0 && batch_flips.1 == 0;
        lines.push(format!(
            "name=stability question={question_and_arms} statuses={} window_pairs={} window_flips={} theta_pairs={} theta_flips={} capacity_pairs={} capacity_flips={} batch_pairs={} batch_flips={} stable={stable}",
            every_status.iter().copied().collect::<Vec<&str>>().join(","),
            window_flips.0,
            window_flips.1,
            theta_flips.0,
            theta_flips.1,
            capacity_flips.0,
            capacity_flips.1,
            batch_flips.0,
            batch_flips.1,
        ));
    }
    lines
}

pub(super) fn run_segment_two(threads: usize) -> Vec<String> {
    let items = segment_two_items();
    let mut lines = vec![format!(
        "name=config experiment=E178 segment=two work_items={} histories_h51={} histories_f10={} histories_f11={} readers={} capacities=233,16 thetas=0,quarter,half borrow_arms=N1,N2 windows=inf,23 ring_window={READABLE_WINDOW_RING_PUBLISHES}",
        items.len(),
        histories_replayed_from_the_e177_segment_one().len(),
        many_objects_with_data_histories().len(),
        rollback_over_a_gap_histories().len(),
        every_reader().len(),
    )];
    lines.extend(constant_anchor_lines());
    for (name, (holds, detail)) in [
        ("S4", stop_check_intent_matches_the_implementation()),
        ("S5", stop_check_tombstone_tree_matches_the_implementation()),
    ] {
        lines.push(format!("name=stop_check id={name} holds={holds} detail={detail}"));
    }
    let (direct_holds, direct_detail) = direct_rollback_positive_control_holds();
    lines.push(format!("name=positive_control id=PC-B2-direct {direct_detail} holds={direct_holds}"));
    let results = run_in_parallel(&items, threads, run_segment_two_item);
    let mut tally = AssertionTally::default();
    let mut every_run_identical = true;
    let mut every_path_agrees = true;
    let mut positive_controls_agree = true;
    let mut positive_controls_closed_form = true;
    let mut uncovered_many_objects_cells = 0;
    for result in &results {
        lines.extend(result.lines.iter().cloned());
        tally.merge(&result.assertions);
        every_run_identical &= result.two_runs_identical;
        every_path_agrees &= result.last_scan_paths_agree;
        positive_controls_agree &= result.positive_controls.iter().all(|outcome| outcome.paths_agree);
        positive_controls_closed_form &= result.positive_controls.iter().all(|outcome| outcome.closed_form_holds);
        if result.item.history.family() == "F10" && result.item.cell.merge_threshold > 0 && !result.path_covered {
            uncovered_many_objects_cells += 1;
        }
    }
    positive_controls_closed_form &= direct_holds;
    let (verdict_lines, statuses) = segment_two_verdict_lines(&results);
    lines.extend(verdict_lines);
    lines.extend(extra_write_verdict_lines(&results));
    lines.extend(stability_lines(&statuses));
    let mut every_assertion_holds = true;
    for (name, judged) in &tally.judged {
        let violated = tally.violations_of(name);
        every_assertion_holds &= violated == 0;
        lines.push(format!("name=assertion assertion={name} judged={judged} violated={violated}"));
    }
    let rollbacks_too_far = results.iter().filter(|result| result.rollback.is_some_and(|outcome| outcome.too_far)).count();
    let rollback_events_missing = results.iter().filter(|result| result.rollback.is_some_and(|outcome| !outcome.event_found)).count();
    lines.push(format!(
        "name=verdict question=summary v2_positive_controls_paths_agree={positive_controls_agree} positive_controls_closed_form_holds={positive_controls_closed_form} v3_last_scan_paths_agree={every_path_agrees} v4_two_runs_identical={every_run_identical} v5_anchors_hold={every_assertion_holds} v6_f10_cells_without_coverage={uncovered_many_objects_cells} f11_rollbacks_too_far={rollbacks_too_far} f11_events_missing={rollback_events_missing}"
    ));
    lines
}

#[cfg(test)]
#[path = "missed_deaths_runs_and_verdicts_tests.rs"]
mod tests;
