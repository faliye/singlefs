//! 一次挂载的运行与会话的开始、位置指标、聚合、格摘要、按组运行与各段的分组。

use super::*;

/// 被测挂载：可写挂载（写行 + 暖机），然后 J 次覆盖写请求，不卸载（登记第一节「一次挂载」）。
pub(super) fn run_a_mount(history: &PoolAfterHistory, fault: Option<ReadFault>, settings: PoolSettings, arm: Arm, controls: Controls, tracing: bool) -> MountRun {
    let mut session = start_a_session(history, fault, settings, arm, controls, tracing);
    for _ in 0..requests_per_mount(settings.geometry) {
        let record = session.run_one_request();
        session.ledger.requests.push(record);
    }
    session.ledger
}

/// 可写挂载做完（写行 + 暖机）、还没发第一次请求的那一刻。
pub(super) fn start_a_session(history: &PoolAfterHistory, fault: Option<ReadFault>, settings: PoolSettings, arm: Arm, controls: Controls, tracing: bool) -> Session {
    let pool = MountedPool::mount(history, fault, settings.reserve_blocks(), settings.free_blocks_after_the_warm_up(), controls.reclaim_disabled);
    let candidate_states_at_the_start = candidate_states_at_the_floor(&pool.truth_ring, pool.floor_in_the_current_version);
    let ledger = MountRun {
        requests: Vec::new(),
        raises: Vec::new(),
        candidate_states_at_the_start,
        cap_active_passes: 0,
        cap_blocked_passes: 0,
        empty_publish_refusals: 0,
        raise_refused_after_an_empty_publish: 0,
        empty_publish_bound_exhausted: 0,
        lowest_free_blocks_in_the_first_raise: None,
        trace: Vec::new(),
    };
    Session { pool, arm, controls, ledger, request_index: 0, pushed_the_seeded_empty_publish: false, tracing }
}

// ── 读数的聚合：每个位置一组读数，每格按位置聚合 ──────────────────────────────────────

/// 登记第八节「轨迹」要的三样：吃光挂载时初始空闲之后（第一次「不够」之后）的峰值、为正的步数、期末值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Trajectory {
    pub(super) peak_after_the_first_short: Option<i64>,
    pub(super) positive_steps: i64,
    pub(super) final_value: i64,
}

pub(super) fn trajectory_of(series: &[i64], first_short_step: Option<usize>) -> Trajectory {
    Trajectory {
        peak_after_the_first_short: first_short_step.and_then(|step| series[step..].iter().copied().max()),
        positive_steps: i64::try_from(series.iter().filter(|value| **value > 0).count()).expect("步数"),
        final_value: *series.last().expect("至少一步"),
    }
}

pub(super) const TRAJECTORY_SERIES: usize = 6;
pub(super) const TRAJECTORY_NAMES: [&str; TRAJECTORY_SERIES] = ["rejected", "crossed", "lost", "empties", "floor", "free_above_reserve"];

/// 一个位置（一个读坏槽 j）上这一格的全部读数。整数一律 i64；没有的量用 −1 表示「没有」只出现在 `first_extra_step`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PositionMetrics {
    pub(super) rejected: i64,
    pub(super) extra_rejected: i64,
    pub(super) all_enospc: i64,
    pub(super) first_extra_step: i64,
    pub(super) accepted_after_the_first_extra: i64,
    pub(super) crossed: i64,
    pub(super) crossed_peak_in_one_raise: i64,
    pub(super) over_truth_steps: i64,
    pub(super) lost_total: i64,
    pub(super) most_states_lost_in_one_raise: i64,
    pub(super) candidates_lowest: i64,
    pub(super) candidates_final: i64,
    empties_total: i64,
    empty_blocks_total: i64,
    most_empties_in_one_admission: i64,
    empty_refusals: i64,
    pub(super) raise_refused_after_an_empty_publish: i64,
    empty_bound_exhausted: i64,
    pub(super) cap_active_passes: i64,
    pub(super) cap_blocked_passes: i64,
    most_publishes_in_one_admission: i64,
    pub(super) raise_sequences: i64,
    pub(super) rejected_by_reason: [i64; REJECTION_REASON_COUNT],
    pub(super) trajectories: [Trajectory; TRAJECTORY_SERIES],
}

pub(super) fn count_as_i64(count: usize) -> i64 {
    i64::try_from(count).expect("计数装得进 i64")
}

pub(super) fn rejected_count(run: &MountRun) -> usize {
    run.requests.iter().filter(|request| !request.accepted).count()
}

/// `run` 是注了读故障的那一次，`baseline` 是同一格去掉读故障的那一次（盘面、池、请求流逐项相同，登记第一节「因读坏多拒」）。
pub(super) fn metrics_of(run: &MountRun, baseline: &MountRun) -> PositionMetrics {
    assert_eq!(run.requests.len(), baseline.requests.len(), "同一格的两次请求数相同");
    let rejected = count_as_i64(rejected_count(run));
    let extra_rejected = rejected - count_as_i64(rejected_count(baseline));
    let first_extra_step = run
        .requests
        .iter()
        .zip(&baseline.requests)
        .position(|(with_fault, without_fault)| !with_fault.accepted && without_fault.accepted)
        .or_else(|| if extra_rejected >= 1 { run.requests.iter().position(|request| !request.accepted) } else { None });
    let accepted_after_the_first_extra = first_extra_step.map_or(0, |step| run.requests[step..].iter().filter(|request| request.accepted).count());
    let all_enospc = extra_rejected >= 1 && first_extra_step.is_some() && accepted_after_the_first_extra == 0;
    let mut rejected_by_reason = [0i64; REJECTION_REASON_COUNT];
    for request in &run.requests {
        if let Some(reason) = request.rejection {
            rejected_by_reason[reason.index()] += 1;
        }
    }
    let first_short_step = run.requests.iter().position(|request| request.was_judged_short);
    let series_of = |value_of: &dyn Fn(&RequestRecord) -> i64| -> Vec<i64> { run.requests.iter().map(value_of).collect() };
    let trajectories = [
        trajectory_of(&series_of(&|request| i64::from(!request.accepted)), first_short_step),
        trajectory_of(&series_of(&|request| i64::from(request.crossed_non_empty_roots)), first_short_step),
        trajectory_of(&series_of(&|request| count_as_i64(request.states_lost)), first_short_step),
        trajectory_of(&series_of(&|request| i64::from(request.empty_publishes)), first_short_step),
        trajectory_of(&series_of(&|request| i64::try_from(request.floor_after).expect("F")), first_short_step),
        trajectory_of(&series_of(&|request| request.free_above_the_reserve_after), first_short_step),
    ];
    let candidates_lowest = run.requests.iter().map(|request| request.candidate_states_after).chain(std::iter::once(run.candidate_states_at_the_start)).min().expect("至少一项");
    PositionMetrics {
        rejected,
        extra_rejected,
        all_enospc: i64::from(all_enospc),
        first_extra_step: first_extra_step.map_or(-1, count_as_i64),
        accepted_after_the_first_extra: count_as_i64(accepted_after_the_first_extra),
        crossed: run.raises.iter().map(|raise| i64::from(raise.crossed_non_empty_roots)).sum(),
        crossed_peak_in_one_raise: run.raises.iter().map(|raise| i64::from(raise.crossed_non_empty_roots)).max().unwrap_or(0),
        over_truth_steps: count_as_i64(run.raises.iter().filter(|raise| raise.floor_is_above_the_truth_ceiling()).count()),
        lost_total: run.raises.iter().map(|raise| count_as_i64(raise.states_lost)).sum(),
        most_states_lost_in_one_raise: run.raises.iter().map(|raise| count_as_i64(raise.states_lost)).max().unwrap_or(0),
        candidates_lowest: count_as_i64(candidates_lowest),
        candidates_final: count_as_i64(run.requests.last().expect("至少一次请求").candidate_states_after),
        empties_total: run.requests.iter().map(|request| i64::from(request.empty_publishes)).sum(),
        empty_blocks_total: run.requests.iter().map(|request| request.empty_publish_blocks).sum(),
        most_empties_in_one_admission: run.requests.iter().map(|request| i64::from(request.empty_publishes)).max().unwrap_or(0),
        empty_refusals: i64::from(run.empty_publish_refusals),
        raise_refused_after_an_empty_publish: i64::from(run.raise_refused_after_an_empty_publish),
        empty_bound_exhausted: i64::from(run.empty_publish_bound_exhausted),
        cap_active_passes: i64::from(run.cap_active_passes),
        cap_blocked_passes: i64::from(run.cap_blocked_passes),
        most_publishes_in_one_admission: count_as_i64(run.requests.iter().map(|request| request.publishes_in_the_admission).max().unwrap_or(0)),
        raise_sequences: count_as_i64(run.raises.len()),
        rejected_by_reason,
        trajectories,
    }
}

/// 一格里一个量在全部位置上的聚合：最大、取到最大的位置、最小、量 > 0 的位置数、位置总数（登记第六节开头）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Aggregate {
    pub(super) maximum: i64,
    pub(super) maximum_at_position: usize,
    pub(super) minimum: i64,
    pub(super) positive_positions: usize,
    pub(super) positions: usize,
}

impl Aggregate {
    pub(super) fn empty() -> Self {
        Self { maximum: i64::MIN, maximum_at_position: 0, minimum: i64::MAX, positive_positions: 0, positions: 0 }
    }

    pub(super) fn add(&mut self, position: usize, value: i64) {
        if value > self.maximum {
            self.maximum = value;
            self.maximum_at_position = position;
        }
        self.minimum = self.minimum.min(value);
        if value > 0 {
            self.positive_positions += 1;
        }
        self.positions += 1;
    }

    pub(super) fn render(&self) -> String {
        if self.positions == 0 {
            return "none".to_string();
        }
        format!("{}@{}/{}/{}/{}", self.maximum, self.maximum_at_position, self.minimum, self.positive_positions, self.positions)
    }
}

/// 一格里「按位置聚合的轨迹三样」：峰值取最大与最小（没有第一次「不够」的位置数单报）、为正的步数取最大与最小、期末值取最大与最小。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TrajectoryAggregate {
    pub(super) peak_maximum: Option<i64>,
    pub(super) peak_minimum: Option<i64>,
    pub(super) positions_never_short: usize,
    positive_steps_maximum: i64,
    positive_steps_minimum: i64,
    final_maximum: i64,
    final_minimum: i64,
}

impl TrajectoryAggregate {
    pub(super) fn empty() -> Self {
        Self {
            peak_maximum: None,
            peak_minimum: None,
            positions_never_short: 0,
            positive_steps_maximum: i64::MIN,
            positive_steps_minimum: i64::MAX,
            final_maximum: i64::MIN,
            final_minimum: i64::MAX,
        }
    }

    pub(super) fn add(&mut self, trajectory: Trajectory) {
        match trajectory.peak_after_the_first_short {
            Some(peak) => {
                self.peak_maximum = Some(self.peak_maximum.map_or(peak, |current| current.max(peak)));
                self.peak_minimum = Some(self.peak_minimum.map_or(peak, |current| current.min(peak)));
            }
            None => self.positions_never_short += 1,
        }
        self.positive_steps_maximum = self.positive_steps_maximum.max(trajectory.positive_steps);
        self.positive_steps_minimum = self.positive_steps_minimum.min(trajectory.positive_steps);
        self.final_maximum = self.final_maximum.max(trajectory.final_value);
        self.final_minimum = self.final_minimum.min(trajectory.final_value);
    }

    pub(super) fn render(&self) -> String {
        let peak = |value: Option<i64>| value.map_or("none".to_string(), |number| number.to_string());
        format!(
            "peak:{}/{}/never_short={}|positive:{}/{}|final:{}/{}",
            peak(self.peak_maximum),
            peak(self.peak_minimum),
            self.positions_never_short,
            self.positive_steps_maximum,
            self.positive_steps_minimum,
            self.final_maximum,
            self.final_minimum
        )
    }
}

fn quantities_of(metrics: &PositionMetrics) -> Vec<(&'static str, i64)> {
    let mut quantities = vec![
        ("extra_rejected", metrics.extra_rejected),
        ("rejected", metrics.rejected),
        ("all_enospc", metrics.all_enospc),
        ("crossed", metrics.crossed),
        ("crossed_peak", metrics.crossed_peak_in_one_raise),
        ("over_truth", metrics.over_truth_steps),
        ("lost_total", metrics.lost_total),
        ("lost_max", metrics.most_states_lost_in_one_raise),
        ("candidates_lowest", metrics.candidates_lowest),
        ("candidates_final", metrics.candidates_final),
        ("empties", metrics.empties_total),
        ("empty_blocks", metrics.empty_blocks_total),
        ("empties_max_in_admission", metrics.most_empties_in_one_admission),
        ("empty_refusals", metrics.empty_refusals),
        ("raise_refused_after_empty", metrics.raise_refused_after_an_empty_publish),
        ("empty_bound_exhausted", metrics.empty_bound_exhausted),
        ("cap_active", metrics.cap_active_passes),
        ("cap_blocked", metrics.cap_blocked_passes),
        ("publishes_max_in_admission", metrics.most_publishes_in_one_admission),
        ("raise_sequences", metrics.raise_sequences),
    ];
    for reason in REJECTION_REASONS {
        quantities.push((reason.rejected_quantity_name(), metrics.rejected_by_reason[reason.index()]));
    }
    if metrics.first_extra_step >= 0 {
        quantities.push(("first_extra_step", metrics.first_extra_step));
        quantities.push(("accepted_after_first_extra", metrics.accepted_after_the_first_extra));
    }
    quantities
}

const REJECTION_REASONS: [RejectionReason; REJECTION_REASON_COUNT] = [
    RejectionReason::AdmissionBudgetUsedUp,
    RejectionReason::FloorAlreadyAtTheCeiling,
    RejectionReason::FloorRaiseSequenceFoundNoPlacement,
    RejectionReason::EmptyPublishFoundNoPlacement,
    RejectionReason::FloorRaiseReportedItsOwnError,
];

impl RejectionReason {
    fn rejected_quantity_name(self) -> &'static str {
        match self {
            Self::AdmissionBudgetUsedUp => "rej_budget",
            Self::FloorAlreadyAtTheCeiling => "rej_floor_at_ceiling",
            Self::FloorRaiseSequenceFoundNoPlacement => "rej_raise_no_placement",
            Self::EmptyPublishFoundNoPlacement => "rej_empty_no_placement",
            Self::FloorRaiseReportedItsOwnError => "rej_own_error",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CellKey {
    pub(super) family: HistoryFamily,
    pub(super) fault_kind: Option<ReadFaultKind>,
    pub(super) persistence: Option<FaultPersistence>,
    pub(super) slots_per_region: u64,
    pub(super) metadata_blocks: i64,
    pub(super) reserve_reading: ReserveReading,
    pub(super) fullness: PoolFullness,
    pub(super) arm: Arm,
}

#[derive(Clone, Debug)]
pub(super) struct CellSummary {
    pub(super) key: CellKey,
    pub(super) positions: usize,
    pub(super) expected_positions: usize,
    pub(super) quantities: BTreeMap<&'static str, Aggregate>,
    pub(super) trajectories: [TrajectoryAggregate; TRAJECTORY_SERIES],
    pub(super) non_empty_slot_positions: usize,
    pub(super) crossed_at_non_empty_slot_positions: Aggregate,
    pub(super) baseline_rejected: i64,
    /// 去掉读故障那一次第一次被拒落在第几次请求（从 0 数；一次都没拒是 None）。重跑登记第六节 Z3 要报；原模式不打印它。
    pub(super) baseline_first_rejected_request: Option<usize>,
}

impl CellSummary {
    pub(super) fn new(key: CellKey, expected_positions: usize, baseline_rejected: i64) -> Self {
        Self {
            key,
            positions: 0,
            expected_positions,
            quantities: BTreeMap::new(),
            trajectories: [TrajectoryAggregate::empty(); TRAJECTORY_SERIES],
            non_empty_slot_positions: 0,
            crossed_at_non_empty_slot_positions: Aggregate::empty(),
            baseline_rejected,
            baseline_first_rejected_request: None,
        }
    }

    fn with_the_baseline(key: CellKey, expected_positions: usize, baseline: &MountRun) -> Self {
        let mut summary = Self::new(key, expected_positions, count_as_i64(rejected_count(baseline)));
        summary.baseline_first_rejected_request = baseline.requests.iter().position(|request| !request.accepted);
        summary
    }

    fn add_position(&mut self, position: usize, metrics: &PositionMetrics, slot_holds_a_non_empty_root: bool) {
        for (name, value) in quantities_of(metrics) {
            self.quantities.entry(name).or_insert_with(Aggregate::empty).add(position, value);
        }
        for (series, trajectory) in metrics.trajectories.iter().enumerate() {
            self.trajectories[series].add(*trajectory);
        }
        if slot_holds_a_non_empty_root {
            self.non_empty_slot_positions += 1;
            self.crossed_at_non_empty_slot_positions.add(position, metrics.crossed);
        }
        self.positions += 1;
    }

    pub(super) fn quantity(&self, name: &str) -> Aggregate {
        self.quantities.get(name).copied().unwrap_or_else(Aggregate::empty)
    }
}

// ── 格与位置的枚举 ──────────────────────────────────────────────────────────────────────

/// 读坏槽 j 在这段历史里的真值是不是一条非空有效根（有效 = txg ≥ F_生效；非空 = 状态与它真值前一条有效根的不同，最旧的拿「没有文件」比）。
fn slot_holds_a_non_empty_valid_root(history: &PoolAfterHistory, root_txg: u64) -> bool {
    let mut valid_roots: Vec<RootContent> = history.ring.iter().flatten().filter(|root| root.checkpoint_txg >= history.effective_floor).copied().collect();
    valid_roots.sort_by_key(|root| root.checkpoint_txg);
    match valid_roots.iter().position(|root| root.checkpoint_txg == root_txg) {
        None => false,
        Some(position) => {
            let state_of_the_previous_root = if position == 0 { STATE_WITHOUT_FILE } else { valid_roots[position - 1].state_number };
            valid_roots[position].state_number != state_of_the_previous_root
        }
    }
}

/// 一格里要扫的读坏位置 j：H1–H4 是 1..3S − 1（N − j 是读坏槽里真住着的那条根的 txg）；H5 不注读故障，只有一个位置（记作 0）。
fn positions_of(family: HistoryFamily, geometry: RingGeometry) -> Vec<usize> {
    match family {
        HistoryFamily::YoungRingWithCrashHoles => vec![0],
        HistoryFamily::CrashedAfterOverwrites
        | HistoryFamily::FloorRaisedAfterOverwrites
        | HistoryFamily::UnmountedAfterOverwrites
        | HistoryFamily::OverwritesAndEmptiesAlternating => {
            (1..usize::try_from(geometry.ring_length_in_txgs()).expect("环长")).collect()
        }
    }
}

/// 同一 Q1 臂下两条 Q2 臂在每个位置上逐项比（登记第六节 Q2-1、Q2-2；F5 自检）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ZeroSlotPairComparison {
    pub(super) positions: usize,
    pub(super) identical_positions: usize,
    pub(super) not_merged_crossed_more: usize,
    pub(super) merged_crossed_more: usize,
    pub(super) not_merged_lost_more: usize,
    pub(super) merged_lost_more: usize,
    pub(super) largest_crossed_excess_of_not_merged: i64,
    pub(super) largest_lost_excess_of_not_merged: i64,
    pub(super) positions_merged_rejects_more: usize,
    pub(super) largest_rejected_excess_of_merged: i64,
    pub(super) positions_merged_is_all_enospc_against_not_merged: usize,
}

#[derive(Clone, Debug)]
pub(super) struct GroupSettings {
    pub(super) family: HistoryFamily,
    pub(super) fault_kind: Option<ReadFaultKind>,
    pub(super) persistence: Option<FaultPersistence>,
    pub(super) pool: PoolSettings,
    pub(super) stuck_ceiling: StuckCeilingDisposition,
}

pub(super) struct GroupResult {
    pub(super) not_merged: CellSummary,
    pub(super) merged: CellSummary,
    pub(super) pair: ZeroSlotPairComparison,
    pub(super) runs: usize,
    /// 去掉读故障的那一次的逐请求读数：接受与否、F、空闲 − P保留、回退候选数（H1–H4 上各臂必须逐项相同，登记第五节「真实基线」）。
    pub(super) baseline_signature: Vec<(bool, u64, i64, usize)>,
    pub(super) baselines_of_the_two_zero_slot_arms_are_identical: bool,
}

fn signature_of_a_run(run: &MountRun) -> Vec<(bool, u64, i64, usize)> {
    run.requests
        .iter()
        .map(|request| (request.accepted, request.floor_after, request.free_above_the_reserve_after, request.candidate_states_after))
        .collect()
}

/// 与「全零不并」那一次比：「全零并」第一次多拒（它拒而不并发成）之后一次请求都没发成。
fn merged_is_all_enospc_against(merged: &MountRun, not_merged: &MountRun) -> bool {
    let first_extra_step = merged.requests.iter().zip(&not_merged.requests).position(|(with_merge, without_merge)| !with_merge.accepted && without_merge.accepted);
    first_extra_step.is_some_and(|step| merged.requests[step..].iter().all(|request| !request.accepted))
}

pub(super) fn run_a_group(group: &GroupSettings) -> GroupResult {
    let geometry = group.pool.geometry;
    let history = group.family.build_pool_after_history(geometry, group.pool.metadata_blocks);
    let positions = positions_of(group.family, geometry);
    let newest_txg = history.highest_txg_used;
    let arm_of = |zero: AllZeroSlotDisposition| Arm { stuck_ceiling: group.stuck_ceiling, all_zero_slot: zero };
    let key_of = |zero: AllZeroSlotDisposition| CellKey {
        family: group.family,
        fault_kind: group.fault_kind,
        persistence: group.persistence,
        slots_per_region: geometry.slots_per_region,
        metadata_blocks: group.pool.metadata_blocks,
        reserve_reading: group.pool.reserve_reading,
        fullness: group.pool.fullness,
        arm: arm_of(zero),
    };
    let baseline_not_merged = run_a_mount(&history, None, group.pool, arm_of(AllZeroSlotDisposition::NotMerged), Controls::none(), false);
    let baseline_merged = run_a_mount(&history, None, group.pool, arm_of(AllZeroSlotDisposition::Merged), Controls::none(), false);
    let mut not_merged = CellSummary::with_the_baseline(key_of(AllZeroSlotDisposition::NotMerged), positions.len(), &baseline_not_merged);
    let mut merged = CellSummary::with_the_baseline(key_of(AllZeroSlotDisposition::Merged), positions.len(), &baseline_merged);
    let mut pair = ZeroSlotPairComparison {
        positions: 0,
        identical_positions: 0,
        not_merged_crossed_more: 0,
        merged_crossed_more: 0,
        not_merged_lost_more: 0,
        merged_lost_more: 0,
        largest_crossed_excess_of_not_merged: i64::MIN,
        largest_lost_excess_of_not_merged: i64::MIN,
        positions_merged_rejects_more: 0,
        largest_rejected_excess_of_merged: i64::MIN,
        positions_merged_is_all_enospc_against_not_merged: 0,
    };
    let mut runs = 2;
    for position in &positions {
        let fault = group.fault_kind.zip(group.persistence).map(|(kind, persistence)| ReadFault {
            slot: geometry.slot_of_txg(newest_txg - u64::try_from(*position).expect("位置")),
            kind,
            persistence,
        });
        let holds_a_non_empty_root = fault.is_some() && slot_holds_a_non_empty_valid_root(&history, newest_txg - u64::try_from(*position).expect("位置"));
        let run_not_merged = run_a_mount(&history, fault, group.pool, arm_of(AllZeroSlotDisposition::NotMerged), Controls::none(), false);
        let run_merged = run_a_mount(&history, fault, group.pool, arm_of(AllZeroSlotDisposition::Merged), Controls::none(), false);
        runs += 2;
        let metrics_not_merged = metrics_of(&run_not_merged, &baseline_not_merged);
        let metrics_merged = metrics_of(&run_merged, &baseline_merged);
        not_merged.add_position(*position, &metrics_not_merged, holds_a_non_empty_root);
        merged.add_position(*position, &metrics_merged, holds_a_non_empty_root);
        pair.positions += 1;
        pair.identical_positions += usize::from(metrics_not_merged == metrics_merged);
        pair.not_merged_crossed_more += usize::from(metrics_not_merged.crossed > metrics_merged.crossed);
        pair.merged_crossed_more += usize::from(metrics_merged.crossed > metrics_not_merged.crossed);
        pair.not_merged_lost_more += usize::from(metrics_not_merged.lost_total > metrics_merged.lost_total);
        pair.merged_lost_more += usize::from(metrics_merged.lost_total > metrics_not_merged.lost_total);
        pair.largest_crossed_excess_of_not_merged = pair.largest_crossed_excess_of_not_merged.max(metrics_not_merged.crossed - metrics_merged.crossed);
        pair.largest_lost_excess_of_not_merged = pair.largest_lost_excess_of_not_merged.max(metrics_not_merged.lost_total - metrics_merged.lost_total);
        pair.positions_merged_rejects_more += usize::from(metrics_merged.rejected > metrics_not_merged.rejected);
        pair.largest_rejected_excess_of_merged = pair.largest_rejected_excess_of_merged.max(metrics_merged.rejected - metrics_not_merged.rejected);
        pair.positions_merged_is_all_enospc_against_not_merged += usize::from(merged_is_all_enospc_against(&run_merged, &run_not_merged));
    }
    let baseline_signature = signature_of_a_run(&baseline_not_merged);
    let baselines_of_the_two_zero_slot_arms_are_identical = baseline_signature == signature_of_a_run(&baseline_merged);
    GroupResult { not_merged, merged, pair, runs, baseline_signature, baselines_of_the_two_zero_slot_arms_are_identical }
}

pub(super) const RESERVE_READINGS: [ReserveReading; 2] =
    [ReserveReading::CodeReadingOneEmptyPublish, ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes];
pub(super) const POOL_FULLNESSES: [PoolFullness; 3] =
    [PoolFullness::NearlyFullWithNoSpareOverwrite, PoolFullness::NearlyFullWithTwoSpareOverwrites, PoolFullness::Roomy];
const FAULT_PERSISTENCES: [FaultPersistence; 4] = [
    FaultPersistence::UntilTheProcessWritesTheSlot,
    FaultPersistence::EvenAfterTheProcessWritesTheSlot,
    FaultPersistence::DuringTheMountOnly,
    FaultPersistence::UntilTheFirstFloorRaiseIsDone,
];
pub(super) const FAMILIES_WITH_A_FAULT_TO_INJECT: [HistoryFamily; 4] = [
    HistoryFamily::CrashedAfterOverwrites,
    HistoryFamily::FloorRaisedAfterOverwrites,
    HistoryFamily::UnmountedAfterOverwrites,
    HistoryFamily::OverwritesAndEmptiesAlternating,
];

pub(super) fn pool_settings_list() -> Vec<PoolSettings> {
    let mut pool_settings = Vec::new();
    for slots_per_region in SLOTS_PER_REGION_CHOICES {
        for metadata_blocks in METADATA_BLOCKS_CHOICES {
            for reserve_reading in RESERVE_READINGS {
                for fullness in POOL_FULLNESSES {
                    pool_settings.push(PoolSettings { geometry: RingGeometry { slots_per_region }, metadata_blocks, reserve_reading, fullness });
                }
            }
        }
    }
    pool_settings
}

/// 第一段（补 Q1）：H1–H4 × e / g × P1 / P2 / T1 / T2 × 三种池 × 四条 Q1 臂，Q2 固定在全零不并（登记第五节「分段」）。
pub(super) fn groups_of_the_first_segment() -> Vec<GroupSettings> {
    stuck_ceiling_groups_over(&pool_settings_list())
}

/// 第一段那一组格（e / g 读坏），池取 `pools`（原第一段是 [`pool_settings_list`]，`checkpoint-cost-reserve` 模式是按几何现算的那一张）。
pub(super) fn stuck_ceiling_groups_over(pools: &[PoolSettings]) -> Vec<GroupSettings> {
    let mut groups = Vec::new();
    for family in FAMILIES_WITH_A_FAULT_TO_INJECT {
        for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes] {
            for persistence in FAULT_PERSISTENCES {
                for pool in pools.iter().copied() {
                    for stuck_ceiling in StuckCeilingDisposition::every_arm() {
                        groups.push(GroupSettings { family, fault_kind: Some(kind), persistence: Some(persistence), pool, stuck_ceiling });
                    }
                }
            }
        }
    }
    groups
}

/// 第二段（补 Q2）：z 与 H5 进来，两条 Q2 臂 × 四条 Q1 臂。H5 不注读故障（洞里真是全零）。
pub(super) fn groups_of_the_second_segment() -> Vec<GroupSettings> {
    zero_slot_groups_over(&pool_settings_list())
}

/// 第二段那一组格（z 与 H5），池取 `pools`。
pub(super) fn zero_slot_groups_over(pools: &[PoolSettings]) -> Vec<GroupSettings> {
    let mut groups = Vec::new();
    for family in FAMILIES_WITH_A_FAULT_TO_INJECT {
        for persistence in FAULT_PERSISTENCES {
            for pool in pools.iter().copied() {
                for stuck_ceiling in StuckCeilingDisposition::every_arm() {
                    groups.push(GroupSettings {
                        family,
                        fault_kind: Some(ReadFaultKind::AllZeroBytes),
                        persistence: Some(persistence),
                        pool,
                        stuck_ceiling,
                    });
                }
            }
        }
    }
    for pool in pools.iter().copied() {
        for stuck_ceiling in StuckCeilingDisposition::every_arm() {
            groups.push(GroupSettings { family: HistoryFamily::YoungRingWithCrashHoles, fault_kind: None, persistence: None, pool, stuck_ceiling });
        }
    }
    groups
}
