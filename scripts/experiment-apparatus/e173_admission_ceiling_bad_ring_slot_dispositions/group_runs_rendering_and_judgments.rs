//! 按组并行、输出行与格的渲染、点的汇总、各判定及其输出。

use super::*;

pub(super) fn run_groups_in_parallel(groups: &[GroupSettings]) -> Vec<GroupResult> {
    let next_group = std::sync::atomic::AtomicUsize::new(0);
    let results: Vec<std::sync::Mutex<Option<GroupResult>>> = groups.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..worker_thread_count().min(groups.len().max(1)) {
            scope.spawn(|| loop {
                let group_index = next_group.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if group_index >= groups.len() {
                    break;
                }
                let result = run_a_group(&groups[group_index]);
                *results[group_index].lock().expect("结果槽没被毒化") = Some(result);
            });
        }
    });
    results.into_iter().map(|slot| slot.into_inner().expect("结果槽没被毒化").expect("每一组都算过")).collect()
}

// ── 产物行 ──────────────────────────────────────────────────────────────────────────────

pub(super) fn emit(emitter: &mut Emitter, body: String) {
    println!("{}", emitter.emit_raw(&body));
}

pub(super) fn yes_no(condition: bool) -> &'static str {
    if condition {
        "yes"
    } else {
        "no"
    }
}

pub(super) fn label_of_the_persistence(persistence: Option<FaultPersistence>) -> &'static str {
    persistence.map_or("none", FaultPersistence::label)
}

pub(super) fn label_of_the_fault_kind(kind: Option<ReadFaultKind>) -> &'static str {
    kind.map_or("none", ReadFaultKind::label)
}

fn render_the_key(key: &CellKey) -> String {
    format!(
        "family={} kind={} persistence={} S={} c_max={} reserve={} pool={} q1={} q2={}{}",
        key.family.label(),
        label_of_the_fault_kind(key.fault_kind),
        label_of_the_persistence(key.persistence),
        key.slots_per_region,
        key.metadata_blocks,
        key.reserve_reading.label(),
        key.fullness.label(),
        key.arm.stuck_ceiling.label(),
        key.arm.all_zero_slot.label(),
        key.reserve_reading.extra_key_fields()
    )
}

pub(super) fn render_a_cell(segment: &str, summary: &CellSummary) -> String {
    let mut text = format!(
        "name=cell segment={segment} {} positions={} expected_positions={} non_empty_slot_positions={} baseline_rejected={} requests={} crossed_at_non_empty={}",
        render_the_key(&summary.key),
        summary.positions,
        summary.expected_positions,
        summary.non_empty_slot_positions,
        summary.baseline_rejected,
        requests_per_mount(RingGeometry { slots_per_region: summary.key.slots_per_region }),
        summary.crossed_at_non_empty_slot_positions.render(),
    );
    for (name, aggregate) in &summary.quantities {
        text.push_str(&format!(" {name}={}", aggregate.render()));
    }
    for (series, name) in TRAJECTORY_NAMES.iter().enumerate() {
        text.push_str(&format!(" traj_{name}={}", summary.trajectories[series].render()));
    }
    text
}

// ── 判定：每个量各占一行、各报各的判定（登记第六节），按 (臂, S, P保留, c_max) 各判一次（登记第八节「判决不稳定的记法」） ──

/// 作废条款（登记第十一节 V1–V6）与停机条款（装置与 `crates/` 对不上、条款与装置对不上）各记各的。
#[derive(Default)]
pub(super) struct RunTally {
    pub(super) void_conditions: Vec<String>,
    pub(super) halt_conditions: Vec<String>,
    pub(super) self_checks_passed: usize,
}

impl RunTally {
    pub(super) fn self_check(&mut self, passed: bool, void_label: &str) {
        if passed {
            self.self_checks_passed += 1;
        } else {
            self.void_conditions.push(void_label.to_string());
        }
    }

    pub(super) fn halt_check(&mut self, passed: bool, halt_label: &str) {
        if passed {
            self.self_checks_passed += 1;
        } else {
            self.halt_conditions.push(halt_label.to_string());
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PointKey {
    pub(super) arm: &'static str,
    pub(super) slots_per_region: u64,
    reserve: &'static str,
    pub(super) metadata_blocks: i64,
    /// 按 ckpt_cost 现算的那几种读法带的池大小；原两种是空串（排在最后一个字段，原模式的点的次序不变）。
    pub(super) pool_size: &'static str,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct PointTotals {
    pub(super) observed_maximum: Option<i64>,
    pub(super) cells_hit: usize,
    pub(super) cells: usize,
    cells_not_applicable: usize,
    cells_with_path_evidence: usize,
}

/// 一条判定：谁的值、门槛 1（数 ≥ 1 才算打中）、路径证据（F6：阴性结果要能和「代码没跑到」分开）。
pub(super) struct JudgmentSettings<'settings> {
    pub(super) judgment_label: &'static str,
    pub(super) scope: &'static str,
    pub(super) value: &'settings dyn Fn(&CellSummary) -> Option<i64>,
    pub(super) path_evidence: &'settings dyn Fn(&CellSummary) -> i64,
}

pub(super) fn collect_the_points(cells: &[&CellSummary], settings: &JudgmentSettings<'_>) -> BTreeMap<PointKey, PointTotals> {
    let mut points: BTreeMap<PointKey, PointTotals> = BTreeMap::new();
    for cell in cells {
        let key = PointKey {
            arm: cell.key.arm.stuck_ceiling.label(),
            slots_per_region: cell.key.slots_per_region,
            reserve: cell.key.reserve_reading.label(),
            metadata_blocks: cell.key.metadata_blocks,
            pool_size: cell.key.reserve_reading.pool_size_label(),
        };
        let totals = points.entry(key).or_default();
        totals.cells += 1;
        if (settings.path_evidence)(cell) > 0 {
            totals.cells_with_path_evidence += 1;
        }
        match (settings.value)(cell) {
            None => totals.cells_not_applicable += 1,
            Some(value) => {
                totals.observed_maximum = Some(totals.observed_maximum.map_or(value, |current| current.max(value)));
                if value >= 1 {
                    totals.cells_hit += 1;
                }
            }
        }
    }
    points
}

/// 发这一条判定的各点与每条臂的稳定性行；交回 (打中的点数, 点数) 按臂分的表，供判别力自证用。
pub(super) fn emit_a_judgment(emitter: &mut Emitter, segment: &str, cells: &[&CellSummary], settings: &JudgmentSettings<'_>) -> BTreeMap<&'static str, (usize, usize, Option<i64>)> {
    let points = collect_the_points(cells, settings);
    let mut per_arm: BTreeMap<&'static str, (usize, usize, Option<i64>)> = BTreeMap::new();
    for (key, totals) in &points {
        let hit = totals.observed_maximum.is_some_and(|value| value >= 1);
        let no_hit_claim = if hit {
            "not_applicable"
        } else if totals.cells_with_path_evidence == 0 {
            "path_not_exercised"
        } else {
            "path_exercised"
        };
        emit(
            emitter,
            format!(
                "name=judgment segment={segment} id={} scope={} arm={} S={} reserve={} c_max={}{} observed_maximum={} cells_hit={} cells={} cells_not_applicable={} cells_with_path_evidence={} hit={} no_hit_claim={no_hit_claim}",
                settings.judgment_label,
                settings.scope,
                key.arm,
                key.slots_per_region,
                key.reserve,
                key.metadata_blocks,
                if key.pool_size.is_empty() { String::new() } else { format!(" pool_size={}", key.pool_size) },
                totals.observed_maximum.map_or("none".to_string(), |value| value.to_string()),
                totals.cells_hit,
                totals.cells,
                totals.cells_not_applicable,
                totals.cells_with_path_evidence,
                yes_no(hit)
            ),
        );
        let entry = per_arm.entry(key.arm).or_insert((0, 0, None));
        entry.0 += usize::from(hit);
        entry.1 += 1;
        entry.2 = match (entry.2, totals.observed_maximum) {
            (Some(left), Some(right)) => Some(left.max(right)),
            (Some(left), None) => Some(left),
            (None, right) => right,
        };
    }
    for (arm, (hit_points, total_points, observed_maximum)) in &per_arm {
        let stability = if *hit_points == *total_points {
            "stable_hit"
        } else if *hit_points == 0 {
            "stable_no_hit"
        } else {
            "unstable"
        };
        emit(
            emitter,
            format!(
                "name=stability segment={segment} id={} scope={} arm={arm} hit_points={hit_points} points={total_points} observed_maximum={} stability={stability}",
                settings.judgment_label,
                settings.scope,
                observed_maximum.map_or("none".to_string(), |value| value.to_string())
            ),
        );
    }
    per_arm
}

/// 判别力自证（登记第八节）：把门槛从 1 挪到「观测到的最大值 + 1」，判定必须由真转假；观测到的最大值是 0 的判定本来就是假，不适用。
pub(super) fn emit_the_threshold_shift_check(emitter: &mut Emitter, tally: &mut RunTally, segment: &str, judgment_label: &str, per_arm: &BTreeMap<&'static str, (usize, usize, Option<i64>)>) {
    for (arm, (_, _, observed_maximum)) in per_arm {
        let Some(observed_maximum) = observed_maximum else {
            continue;
        };
        let hit_at_the_original_threshold = *observed_maximum >= 1;
        let shifted_threshold = observed_maximum + 1;
        let hit_at_the_shifted_threshold = *observed_maximum >= shifted_threshold;
        let applicable = hit_at_the_original_threshold;
        if applicable {
            tally.self_check(!hit_at_the_shifted_threshold, "threshold_shift_does_not_flip");
        }
        emit(
            emitter,
            format!(
                "name=threshold_shift segment={segment} id={judgment_label} arm={arm} observed_maximum={observed_maximum} hit_at_threshold_1={} shifted_threshold={shifted_threshold} hit_at_shifted_threshold={} flips={}",
                yes_no(hit_at_the_original_threshold),
                yes_no(hit_at_the_shifted_threshold),
                if applicable { yes_no(hit_at_the_original_threshold && !hit_at_the_shifted_threshold) } else { "not_applicable_observed_below_1" }
            ),
        );
    }
}

#[allow(clippy::too_many_arguments, reason = "这些参数是这一次计算彼此独立的输入，合成只用一次的结构体只多一层包装")]
fn judge(
    emitter: &mut Emitter,
    tally: &mut RunTally,
    segment: &str,
    judgment_label: &'static str,
    scope: &'static str,
    cells: &[&CellSummary],
    value: &dyn Fn(&CellSummary) -> Option<i64>,
    path_evidence: &dyn Fn(&CellSummary) -> i64,
) {
    let settings = JudgmentSettings { judgment_label, scope, value, path_evidence };
    let per_arm = emit_a_judgment(emitter, segment, cells, &settings);
    emit_the_threshold_shift_check(emitter, tally, segment, judgment_label, &per_arm);
}

pub(super) fn maximum_of(cell: &CellSummary, quantity_name: &str) -> i64 {
    let aggregate = cell.quantity(quantity_name);
    if aggregate.positions == 0 {
        0
    } else {
        aggregate.maximum
    }
}

fn positive_positions_of(cell: &CellSummary, quantity_name: &str) -> i64 {
    count_as_i64(cell.quantity(quantity_name).positive_positions)
}

fn crossed_at_non_empty_slots(cell: &CellSummary) -> Option<i64> {
    if cell.non_empty_slot_positions == 0 {
        None
    } else {
        Some(cell.crossed_at_non_empty_slot_positions.maximum)
    }
}

fn is_a_persistent_fault(cell: &CellSummary) -> bool {
    matches!(cell.key.persistence, Some(FaultPersistence::UntilTheProcessWritesTheSlot | FaultPersistence::EvenAfterTheProcessWritesTheSlot))
}

fn is_a_transient_fault(cell: &CellSummary) -> bool {
    matches!(cell.key.persistence, Some(FaultPersistence::DuringTheMountOnly | FaultPersistence::UntilTheFirstFloorRaiseIsDone))
}

fn is_an_empty_pushing_arm(cell: &CellSummary) -> bool {
    matches!(cell.key.arm.stuck_ceiling, StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget)
}

fn admission_path_evidence(cell: &CellSummary) -> i64 {
    maximum_of(cell, "raise_sequences").max(maximum_of(cell, "cap_blocked")).max(maximum_of(cell, "empties"))
}

/// Q1 的一条判定：哪些格进来（范围）、每格的值、路径证据。第一段与 `checkpoint-cost-reserve` 模式共用这一张表，
/// 后者的读法分辨格（重跑登记第六节 Z1）逐格拿同一个值判打不打中。
pub(super) struct StuckCeilingJudgment {
    pub(super) judgment_label: &'static str,
    scope: &'static str,
    pub(super) belongs_to_the_scope: fn(&CellSummary) -> bool,
    pub(super) value: fn(&CellSummary) -> Option<i64>,
    path_evidence: fn(&CellSummary) -> i64,
}

fn persistent_near_full(cell: &CellSummary) -> bool {
    is_a_persistent_fault(cell) && cell.key.fullness.is_nearly_full()
}

fn transient_near_full(cell: &CellSummary) -> bool {
    is_a_transient_fault(cell) && cell.key.fullness.is_nearly_full()
}

fn roomy_pool(cell: &CellSummary) -> bool {
    cell.key.fullness == PoolFullness::Roomy
}

fn empty_pushing_arm_near_full(cell: &CellSummary) -> bool {
    is_an_empty_pushing_arm(cell) && cell.key.fullness.is_nearly_full()
}

fn every_cell(_: &CellSummary) -> bool {
    true
}

fn extra_rejected_maximum(cell: &CellSummary) -> Option<i64> {
    Some(maximum_of(cell, "extra_rejected"))
}

fn non_empty_slot_positions_of(cell: &CellSummary) -> i64 {
    count_as_i64(cell.non_empty_slot_positions)
}

fn raise_sequences_of(cell: &CellSummary) -> i64 {
    maximum_of(cell, "raise_sequences")
}

fn empties_of(cell: &CellSummary) -> i64 {
    maximum_of(cell, "empties")
}

/// 登记第六节 Q1-1 到 Q1-6，次序就是产物里判定行的次序。
pub(super) const STUCK_CEILING_JUDGMENTS: [StuckCeilingJudgment; 15] = [
    StuckCeilingJudgment { judgment_label: "Q1-1", scope: "persistent_near_full", belongs_to_the_scope: persistent_near_full, value: extra_rejected_maximum, path_evidence: admission_path_evidence },
    StuckCeilingJudgment { judgment_label: "Q1-1b", scope: "persistent_near_full", belongs_to_the_scope: persistent_near_full, value: |cell| Some(positive_positions_of(cell, "all_enospc")), path_evidence: admission_path_evidence },
    StuckCeilingJudgment { judgment_label: "Q1-2", scope: "every_persistence_every_pool", belongs_to_the_scope: every_cell, value: crossed_at_non_empty_slots, path_evidence: non_empty_slot_positions_of },
    StuckCeilingJudgment { judgment_label: "Q1-2b", scope: "every_persistence_every_pool", belongs_to_the_scope: every_cell, value: |cell| Some(maximum_of(cell, "over_truth")), path_evidence: raise_sequences_of },
    StuckCeilingJudgment { judgment_label: "Q1-3", scope: "empty_pushing_arms_near_full", belongs_to_the_scope: empty_pushing_arm_near_full, value: |cell| Some(maximum_of(cell, "empty_refusals")), path_evidence: empties_of },
    StuckCeilingJudgment { judgment_label: "Q1-3-raise-refused-after-empty", scope: "empty_pushing_arms_near_full", belongs_to_the_scope: empty_pushing_arm_near_full, value: |cell| Some(maximum_of(cell, "raise_refused_after_empty")), path_evidence: empties_of },
    StuckCeilingJudgment { judgment_label: "Q1-3-empties-in-one-admission", scope: "empty_pushing_arms_near_full", belongs_to_the_scope: empty_pushing_arm_near_full, value: |cell| Some(maximum_of(cell, "empties_max_in_admission")), path_evidence: empties_of },
    StuckCeilingJudgment { judgment_label: "Q1-3-empties-in-one-mount", scope: "empty_pushing_arms_near_full", belongs_to_the_scope: empty_pushing_arm_near_full, value: |cell| Some(maximum_of(cell, "empties")), path_evidence: empties_of },
    StuckCeilingJudgment { judgment_label: "Q1-3-empty-blocks-in-one-mount", scope: "empty_pushing_arms_near_full", belongs_to_the_scope: empty_pushing_arm_near_full, value: |cell| Some(maximum_of(cell, "empty_blocks")), path_evidence: empties_of },
    StuckCeilingJudgment { judgment_label: "Q1-4-extra-rejected", scope: "transient_near_full", belongs_to_the_scope: transient_near_full, value: extra_rejected_maximum, path_evidence: admission_path_evidence },
    StuckCeilingJudgment { judgment_label: "Q1-4-crossed", scope: "transient_every_pool", belongs_to_the_scope: is_a_transient_fault, value: crossed_at_non_empty_slots, path_evidence: non_empty_slot_positions_of },
    StuckCeilingJudgment { judgment_label: "Q1-5-extra-rejected", scope: "roomy", belongs_to_the_scope: roomy_pool, value: extra_rejected_maximum, path_evidence: admission_path_evidence },
    StuckCeilingJudgment { judgment_label: "Q1-5-crossed", scope: "roomy", belongs_to_the_scope: roomy_pool, value: crossed_at_non_empty_slots, path_evidence: non_empty_slot_positions_of },
    StuckCeilingJudgment { judgment_label: "Q1-6-cap-active", scope: "every_persistence_every_pool", belongs_to_the_scope: every_cell, value: |cell| Some(maximum_of(cell, "cap_active")), path_evidence: raise_sequences_of },
    StuckCeilingJudgment { judgment_label: "Q1-6-cap-blocked", scope: "every_persistence_every_pool", belongs_to_the_scope: every_cell, value: |cell| Some(maximum_of(cell, "cap_blocked")), path_evidence: raise_sequences_of },
];

/// 第一段的判定：Q1-1 到 Q1-6（登记第六节）。
pub(super) fn emit_the_first_segment_judgments(emitter: &mut Emitter, tally: &mut RunTally, results: &[GroupResult]) {
    emit_the_stuck_ceiling_judgments(emitter, tally, "first", results);
}

/// Q1 的判定，`segment` 是产物行里的段名（原第一段 `first`；`checkpoint-cost-reserve` 模式里 `ckpt-first`）。
pub(super) fn emit_the_stuck_ceiling_judgments(emitter: &mut Emitter, tally: &mut RunTally, segment: &str, results: &[GroupResult]) {
    let cells: Vec<&CellSummary> = results.iter().map(|result| &result.not_merged).collect();
    for judgment in &STUCK_CEILING_JUDGMENTS {
        let cells_in_the_scope: Vec<&CellSummary> = cells.iter().copied().filter(|cell| (judgment.belongs_to_the_scope)(cell)).collect();
        judge(emitter, tally, segment, judgment.judgment_label, judgment.scope, &cells_in_the_scope, &judgment.value, &judgment.path_evidence);
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ZeroSlotTotals {
    pub(super) cells: usize,
    pub(super) cells_where_not_merged_has_more: usize,
    pub(super) cells_where_merged_has_more: usize,
    pub(super) largest_crossed_excess_of_not_merged: i64,
    pub(super) largest_lost_excess_of_not_merged: i64,
    pub(super) crossed_maximum_not_merged: i64,
    pub(super) crossed_maximum_merged: i64,
    pub(super) lost_maximum_not_merged: i64,
    pub(super) lost_maximum_merged: i64,
    pub(super) all_enospc_positions_not_merged: usize,
    pub(super) all_enospc_positions_merged: usize,
    pub(super) extra_rejected_maximum_not_merged: i64,
    pub(super) extra_rejected_maximum_merged: i64,
    pub(super) positions_merged_is_all_enospc_against_not_merged: usize,
    pub(super) positions_merged_rejects_more: usize,
    pub(super) largest_rejected_excess_of_merged: i64,
    pub(super) lowest_candidates_not_merged: i64,
    pub(super) lowest_candidates_merged: i64,
    pub(super) lowest_final_candidates_not_merged: i64,
    pub(super) lowest_final_candidates_merged: i64,
}

impl ZeroSlotTotals {
    pub(super) fn start() -> Self {
        Self {
            cells: 0,
            cells_where_not_merged_has_more: 0,
            cells_where_merged_has_more: 0,
            largest_crossed_excess_of_not_merged: i64::MIN,
            largest_lost_excess_of_not_merged: i64::MIN,
            crossed_maximum_not_merged: 0,
            crossed_maximum_merged: 0,
            lost_maximum_not_merged: 0,
            lost_maximum_merged: 0,
            all_enospc_positions_not_merged: 0,
            all_enospc_positions_merged: 0,
            extra_rejected_maximum_not_merged: i64::MIN,
            extra_rejected_maximum_merged: i64::MIN,
            positions_merged_is_all_enospc_against_not_merged: 0,
            positions_merged_rejects_more: 0,
            largest_rejected_excess_of_merged: i64::MIN,
            lowest_candidates_not_merged: i64::MAX,
            lowest_candidates_merged: i64::MAX,
            lowest_final_candidates_not_merged: i64::MAX,
            lowest_final_candidates_merged: i64::MAX,
        }
    }

    pub(super) fn add(&mut self, result: &GroupResult) {
        let (not_merged, merged, pair) = (&result.not_merged, &result.merged, &result.pair);
        self.cells += 1;
        self.cells_where_not_merged_has_more += usize::from(pair.not_merged_crossed_more + pair.not_merged_lost_more > 0);
        self.cells_where_merged_has_more += usize::from(pair.merged_crossed_more + pair.merged_lost_more > 0);
        self.largest_crossed_excess_of_not_merged = self.largest_crossed_excess_of_not_merged.max(pair.largest_crossed_excess_of_not_merged);
        self.largest_lost_excess_of_not_merged = self.largest_lost_excess_of_not_merged.max(pair.largest_lost_excess_of_not_merged);
        self.crossed_maximum_not_merged = self.crossed_maximum_not_merged.max(maximum_of(not_merged, "crossed"));
        self.crossed_maximum_merged = self.crossed_maximum_merged.max(maximum_of(merged, "crossed"));
        self.lost_maximum_not_merged = self.lost_maximum_not_merged.max(maximum_of(not_merged, "lost_total"));
        self.lost_maximum_merged = self.lost_maximum_merged.max(maximum_of(merged, "lost_total"));
        self.all_enospc_positions_not_merged += not_merged.quantity("all_enospc").positive_positions;
        self.all_enospc_positions_merged += merged.quantity("all_enospc").positive_positions;
        self.extra_rejected_maximum_not_merged = self.extra_rejected_maximum_not_merged.max(maximum_of(not_merged, "extra_rejected"));
        self.extra_rejected_maximum_merged = self.extra_rejected_maximum_merged.max(maximum_of(merged, "extra_rejected"));
        self.positions_merged_is_all_enospc_against_not_merged += pair.positions_merged_is_all_enospc_against_not_merged;
        self.positions_merged_rejects_more += pair.positions_merged_rejects_more;
        self.largest_rejected_excess_of_merged = self.largest_rejected_excess_of_merged.max(pair.largest_rejected_excess_of_merged);
        self.lowest_candidates_not_merged = self.lowest_candidates_not_merged.min(not_merged.quantity("candidates_lowest").minimum);
        self.lowest_candidates_merged = self.lowest_candidates_merged.min(merged.quantity("candidates_lowest").minimum);
        self.lowest_final_candidates_not_merged = self.lowest_final_candidates_not_merged.min(not_merged.quantity("candidates_final").minimum);
        self.lowest_final_candidates_merged = self.lowest_final_candidates_merged.min(merged.quantity("candidates_final").minimum);
    }
}
