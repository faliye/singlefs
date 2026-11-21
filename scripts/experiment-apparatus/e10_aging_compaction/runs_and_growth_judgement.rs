//! 取样点、臂、一趟「填满 → 删 → 填满」的整趟、增长判定、轨迹统计与整理意图的错误计数。

use super::*;

/// 删段怎么挑（登记 M-9）：主取样点整文件删，反向点按单元随机删。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeletionMode {
    WholeFiles,
    RandomUnits,
}

/// 一个取样点：主取样点与第八节六个反向点，每个反向点只换一个旋钮。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SamplingPoint {
    pub(super) name: &'static str,
    pub(super) disk_slots: u64,
    pub(super) leaf_width_in_slots: u64,
    pub(super) remounts_every_round: bool,
    pub(super) deletion_mode: DeletionMode,
    pub(super) snapshot_limit: u64,
    pub(super) resident_base_multiplier: u64,
}

pub(super) const MAIN_SAMPLING_POINT: SamplingPoint = SamplingPoint {
    name: "main",
    disk_slots: FOUR_GIBIBYTE_DISK_SLOTS,
    leaf_width_in_slots: ALLOCATION_RECORD_TREE_LEAF_SLOTS,
    remounts_every_round: true,
    deletion_mode: DeletionMode::WholeFiles,
    snapshot_limit: 0,
    resident_base_multiplier: 3,
};

/// 主取样点在前，六个反向点按登记第八节表的次序；每个反向点只有一个字段与主取样点不同。
pub(super) const SAMPLING_POINTS: [SamplingPoint; 7] = [
    MAIN_SAMPLING_POINT,
    SamplingPoint {
        name: "disk_1gib",
        disk_slots: ONE_GIBIBYTE_DISK_SLOTS,
        leaf_width_in_slots: ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        remounts_every_round: true,
        deletion_mode: DeletionMode::WholeFiles,
        snapshot_limit: 0,
        resident_base_multiplier: 3,
    },
    SamplingPoint {
        name: "no_remount",
        disk_slots: FOUR_GIBIBYTE_DISK_SLOTS,
        leaf_width_in_slots: ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        remounts_every_round: false,
        deletion_mode: DeletionMode::WholeFiles,
        snapshot_limit: 0,
        resident_base_multiplier: 3,
    },
    SamplingPoint {
        name: "delete_units",
        disk_slots: FOUR_GIBIBYTE_DISK_SLOTS,
        leaf_width_in_slots: ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        remounts_every_round: true,
        deletion_mode: DeletionMode::RandomUnits,
        snapshot_limit: 0,
        resident_base_multiplier: 3,
    },
    SamplingPoint {
        name: "snapshots_10",
        disk_slots: FOUR_GIBIBYTE_DISK_SLOTS,
        leaf_width_in_slots: ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        remounts_every_round: true,
        deletion_mode: DeletionMode::WholeFiles,
        snapshot_limit: 10,
        resident_base_multiplier: 3,
    },
    SamplingPoint {
        name: "leaf_203",
        disk_slots: FOUR_GIBIBYTE_DISK_SLOTS,
        leaf_width_in_slots: REVERSE_POINT_LEAF_SLOTS,
        remounts_every_round: true,
        deletion_mode: DeletionMode::WholeFiles,
        snapshot_limit: 0,
        resident_base_multiplier: 3,
    },
    SamplingPoint {
        name: "base_6sc",
        disk_slots: FOUR_GIBIBYTE_DISK_SLOTS,
        leaf_width_in_slots: ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        remounts_every_round: true,
        deletion_mode: DeletionMode::WholeFiles,
        snapshot_limit: 0,
        resident_base_multiplier: 6,
    },
];

/// 第 4 行的三个候选与只报的 `N₀ = 0`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FloorArm {
    One,
    Two,
    Formula,
    Zero,
}

pub(super) const FLOOR_ARMS: [FloorArm; 4] = [FloorArm::One, FloorArm::Two, FloorArm::Formula, FloorArm::Zero];
pub(super) const CANDIDATE_FLOOR_ARMS: [FloorArm; 3] = [FloorArm::One, FloorArm::Two, FloorArm::Formula];

impl FloorArm {
    pub(super) fn floor_segments(self, root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64) -> u64 {
        match self {
            FloorArm::One => 1,
            FloorArm::Two => 2,
            FloorArm::Formula => formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish),
            FloorArm::Zero => 0,
        }
    }

    pub(super) fn tag(self) -> &'static str {
        match self {
            FloorArm::One => "segment_floor_one",
            FloorArm::Two => "segment_floor_two",
            FloorArm::Formula => "segment_floor_formula",
            FloorArm::Zero => "segment_floor_zero",
        }
    }
}

/// 一条正式跑的键（登记第五节「分片单元」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RunKey {
    pub(super) point: SamplingPoint,
    pub(super) root_ring_slots_per_region: u64,
    pub(super) commit_generated_blocks_per_publish: u64,
    pub(super) arm: FloorArm,
    pub(super) is_compaction_enabled: bool,
    pub(super) seed: u64,
}

impl RunKey {
    pub(super) fn geometry(&self) -> PoolGeometry {
        PoolGeometry::new(
            self.point.disk_slots,
            self.point.leaf_width_in_slots,
            self.root_ring_slots_per_region,
            self.commit_generated_blocks_per_publish,
        )
    }

    pub(super) fn label(&self) -> String {
        format!(
            "point={} root_ring_slots_per_region={} commit_generated_blocks_per_publish={} arm={} compaction={} seed={}",
            self.point.name,
            self.root_ring_slots_per_region,
            self.commit_generated_blocks_per_publish,
            self.arm.tag(),
            if self.is_compaction_enabled { "on" } else { "off" },
            self.seed
        )
    }
}

/// 删除队列里的一批：头一侧删单元，或销毁快照的一批。
#[derive(Clone, Debug, PartialEq, Eq)]
enum DeletionBatch {
    HeadUnits(Vec<u32>),
    SnapshotDestruction { units: Vec<u32>, pinned_file_tree_offsets: Vec<u64> },
}

/// 一个活快照钉住的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
struct Snapshot {
    pub(super) units: Vec<u32>,
    pinned_file_tree_offsets: Vec<u64>,
}

/// 一轮结束时记下的一行（逐轮值整列写进产物，登记第八节「轨迹」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RoundRow {
    pub(super) round: u64,
    pub(super) metrics: RoundMetrics,
    pub(super) free_runs: u64,
    pub(super) live_user_units: u64,
    pub(super) unrecoverable_slots: u64,
    pub(super) record_empty_segments_at_end: u64,
}

/// 一条 FDF 跑（登记 M-9）：每轮 ① 填满 ② 有快照时建一个 ③ 删一半 ④ 调一次甲 ⑤ 重挂。
pub(super) struct FillDeleteFillRun {
    pub(super) key: RunKey,
    pub(super) pool: Pool,
    random: SeededRandom,
    /// 整文件删时用：头一侧还活着、没进删除队列的文件，每个文件是它的单元。
    live_files: Vec<Vec<u32>>,
    pending_deletions: VecDeque<DeletionBatch>,
    snapshots: VecDeque<Snapshot>,
    pub(super) rows: Vec<RoundRow>,
}

impl FillDeleteFillRun {
    pub(super) fn new(key: RunKey) -> Self {
        let geometry = key.geometry();
        let settings = PoolSettings {
            floor_segments: key.arm.floor_segments(key.root_ring_slots_per_region, key.commit_generated_blocks_per_publish),
            is_compaction_enabled: key.is_compaction_enabled,
            stop_arm: StopArm::LocalRegionPredicate,
            resident_base_multiplier: key.point.resident_base_multiplier,
        };
        Self {
            key,
            pool: Pool::made_by_make_filesystem(geometry, settings),
            random: SeededRandom::new(key.seed),
            live_files: Vec::new(),
            pending_deletions: VecDeque::new(),
            snapshots: VecDeque::new(),
            rows: Vec::new(),
        }
    }

    /// ① 填满段：一次前台写入类发布写一个新文件，被拒就照 M-6 做内部动作再试一次，再被拒就记一次被拒写入、填满段结束。
    fn fill(&mut self) {
        loop {
            let unit_count = 1 + self.random.below(FILE_UNITS_UPPER_BOUND);
            if let Ok(units) = self.pool.publish_new_file(unit_count) {
                self.remember_written_file(units);
                continue;
            }
            self.pool.raise_floor();
            if self.pool.settings.is_compaction_enabled {
                self.pool.call_compaction_entry();
            }
            match self.pool.publish_new_file(unit_count) {
                Ok(units) => self.remember_written_file(units),
                Err(refusal) => {
                    let metrics = &mut self.pool.round_metrics;
                    match refusal {
                        PublishRefusal::ConjunctSeven => metrics.refused_writes_by_conjunct_seven += 1,
                        PublishRefusal::ConjunctOne | PublishRefusal::NoSlotForCommitGeneratedBlock => {
                            metrics.refused_writes_by_conjunct_one += 1;
                        }
                    }
                    let free_slots_now = self.pool.free_slots_outside_region();
                    self.pool.round_metrics.free_slots_at_last_refusal = Some(free_slots_now);
                    return;
                }
            }
        }
    }

    fn remember_written_file(&mut self, units: Vec<u32>) {
        if self.key.point.deletion_mode == DeletionMode::WholeFiles {
            self.live_files.push(units);
        }
        self.pool.run_compaction_after_publish();
    }

    /// 删除队列从头发，一批发不出去就记一次「删除发不出去」、这一段停下，剩下的留到下一次（登记 M-5）。返回队列是否发空了。
    fn run_pending_deletions(&mut self) -> bool {
        while let Some(batch) = self.pending_deletions.front() {
            let result = match batch {
                DeletionBatch::HeadUnits(units) => {
                    let units = units.clone();
                    self.pool.publish_head_deletion(&units)
                }
                DeletionBatch::SnapshotDestruction { units, pinned_file_tree_offsets } => {
                    let (units, pinned_file_tree_offsets) = (units.clone(), pinned_file_tree_offsets.clone());
                    self.pool.publish_snapshot_destruction_batch(&units, &pinned_file_tree_offsets)
                }
            };
            if result.is_err() {
                self.pool.round_metrics.deletion_publish_failures += 1;
                return false;
            }
            self.pending_deletions.pop_front();
            self.pool.run_compaction_after_publish();
        }
        true
    }

    /// ② 有快照时建一个；到上限先销毁最旧的那个（按批走删除类发布，一批至多 q 个单元）。
    fn take_snapshot(&mut self) {
        let snapshot_limit = self.key.point.snapshot_limit;
        if snapshot_limit == 0 {
            return;
        }
        if self.snapshots.len() as u64 >= snapshot_limit {
            let oldest = self.snapshots.pop_front().expect("到上限时至少有一个快照");
            let mut unit_batches: Vec<Vec<u32>> =
                oldest.units.chunks(UNITS_PER_DELETION_PUBLISH).map(<[u32]>::to_vec).collect();
            if unit_batches.is_empty() {
                unit_batches.push(Vec::new());
            }
            for (batch_index, units) in unit_batches.into_iter().enumerate() {
                let pinned_file_tree_offsets =
                    if batch_index == 0 { oldest.pinned_file_tree_offsets.clone() } else { Vec::new() };
                self.pending_deletions.push_back(DeletionBatch::SnapshotDestruction { units, pinned_file_tree_offsets });
            }
        }
        if !self.run_pending_deletions() {
            return;
        }
        if let Ok((units, pinned_file_tree_offsets)) = self.pool.publish_snapshot_creation() {
            self.snapshots.push_back(Snapshot { units, pinned_file_tree_offsets });
        }
    }

    /// 从 `candidates` 里按种子不放回地挑一半（部分 Fisher–Yates），挑中的在前、其余在后，次序由种子定。
    fn choose_half<Item>(random: &mut SeededRandom, candidates: &mut [Item]) -> usize {
        let chosen_count = candidates.len() / 2;
        for chosen_index in 0..chosen_count {
            let remaining = (candidates.len() - chosen_index) as u64;
            let picked = chosen_index + usize_of(random.below(remaining));
            candidates.swap(chosen_index, picked);
        }
        chosen_count
    }

    /// ③ 删段：删掉活的一半（主取样点整文件，反向点按单元），一次删除类发布至多 q 个单元。队列里还有上一次没发出去的就只重试它们。
    fn delete_half(&mut self) {
        if self.pending_deletions.is_empty() {
            match self.key.point.deletion_mode {
                DeletionMode::WholeFiles => {
                    let mut files = std::mem::take(&mut self.live_files);
                    let chosen_count = Self::choose_half(&mut self.random, &mut files);
                    self.live_files = files.split_off(chosen_count);
                    let mut batch: Vec<u32> = Vec::new();
                    for file in files {
                        if batch.len() + file.len() > UNITS_PER_DELETION_PUBLISH {
                            self.pending_deletions.push_back(DeletionBatch::HeadUnits(std::mem::take(&mut batch)));
                        }
                        batch.extend(file);
                    }
                    if !batch.is_empty() {
                        self.pending_deletions.push_back(DeletionBatch::HeadUnits(batch));
                    }
                }
                DeletionMode::RandomUnits => {
                    let mut alive_units: Vec<u32> = (0..self.pool.units.len())
                        .filter(|unit| self.pool.units[*unit].is_alive_in_head)
                        .map(|unit| u32::try_from(unit).expect("单元数装得进 u32"))
                        .collect();
                    let chosen_count = Self::choose_half(&mut self.random, &mut alive_units);
                    for units in alive_units[..chosen_count].chunks(UNITS_PER_DELETION_PUBLISH) {
                        self.pending_deletions.push_back(DeletionBatch::HeadUnits(units.to_vec()));
                    }
                }
            }
        }
        self.run_pending_deletions();
    }

    pub(super) fn run_round(&mut self, round: u64) {
        let pool = &mut self.pool;
        pool.current_round = round;
        pool.round_metrics =
            RoundMetrics::starting_at(pool.record_empty_segment_count, pool.fully_free_segments.len() as u64, pool.openable_segment_count());
        self.fill();
        self.take_snapshot();
        self.delete_half();
        if self.pool.settings.is_compaction_enabled {
            self.pool.call_compaction_entry();
        }
        if self.key.point.remounts_every_round {
            self.pool.remount();
        }
        self.pool.sample_segment_counts();
        self.pool.verify_incremental_counters_against_a_full_scan();
        let read_only_pool = &self.pool;
        self.rows.push(RoundRow {
            round,
            metrics: read_only_pool.round_metrics,
            free_runs: read_only_pool.free_runs,
            live_user_units: read_only_pool.live_user_unit_total,
            unrecoverable_slots: read_only_pool.released_slot_total + 2 * read_only_pool.snapshot_only_unit_total,
            record_empty_segments_at_end: read_only_pool.record_empty_segment_count,
        });
    }

    /// 跑 60 轮；删除类发布的块数判「有界」的续到 120 轮做加倍复核（登记第六节第 3 行）。
    pub(super) fn run_to_judgement(mut self) -> FinishedRun {
        for round in 1..=ROUNDS_FIRST_PASS {
            self.run_round(round);
        }
        let structural_bound = self.pool.geometry.structural_commit_generated_block_bound();
        let first_pass = judge_deletion_block_growth(&self.rows, structural_bound, &GROWTH_THRESHOLDS);
        let growth = if first_pass == GrowthVerdict::Bounded {
            for round in ROUNDS_FIRST_PASS + 1..=ROUNDS_DOUBLED_CHECK {
                self.run_round(round);
            }
            judge_doubled_growth_check(&self.rows)
        } else {
            first_pass
        };
        FinishedRun { key: self.key, rows: self.rows, ended_intents: self.pool.ended_intents, growth }
    }
}

/// 跑完的一条：逐轮行、逐意图的四个量与第 3 行的判定。
pub(super) struct FinishedRun {
    pub(super) key: RunKey,
    pub(super) rows: Vec<RoundRow>,
    pub(super) ended_intents: Vec<EndedIntent>,
    pub(super) growth: GrowthVerdict,
}

// ---- 判定（登记第六节第 1 组；跑前写死的门槛） ----

/// 第 3 行：删除类发布的 `c_actual` 随轮数是否发散。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GrowthVerdict {
    Bounded,
    Diverging,
    /// 窗口里一轮删除类发布都没有：读不到，不当 0（`test-discipline.md`「读不到 ≠ 读到 0」）。
    Unreadable,
}

impl GrowthVerdict {
    pub(super) fn tag(self) -> &'static str {
        match self {
            GrowthVerdict::Bounded => "bounded",
            GrowthVerdict::Diverging => "diverging",
            GrowthVerdict::Unreadable => "unreadable",
        }
    }
}

/// 第 3 行的两个门槛，写成参数只为判别力自证（登记第八节末段）能把它挪到两点之间。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GrowthThresholds {
    /// 后窗口均值 ≤ 这个百分数 × 前窗口均值。
    pub(super) mean_growth_percent: u64,
    /// 后窗口峰值 < 这个十分数 × 结构上限。
    pub(super) structural_bound_tenths: u64,
}

pub(super) const GROWTH_THRESHOLDS: GrowthThresholds = GrowthThresholds { mean_growth_percent: 105, structural_bound_tenths: 9 };

/// 一个窗口（轮的下标区间，0 起）里删除类发布 `c_actual` 的峰值与逐轮均值的均值；一轮都读不到就是 None。
fn window_peak_and_mean(rows: &[RoundRow], round_indices: std::ops::Range<usize>) -> Option<(u64, f64)> {
    let readable: Vec<&RoundRow> =
        rows[round_indices].iter().filter(|row| row.metrics.deletion_publishes > 0).collect();
    if readable.is_empty() {
        return None;
    }
    let peak = readable.iter().map(|row| row.metrics.deletion_commit_generated_block_peak).max().expect("非空");
    let mean_of_round_means = readable
        .iter()
        .map(|row| row.metrics.deletion_commit_generated_block_sum as f64 / row.metrics.deletion_publishes as f64)
        .sum::<f64>()
        / readable.len() as f64;
    Some((peak, mean_of_round_means))
}

/// 60 轮时的判定：有界 ⟺ max m(41–60) ≤ max m(21–40) 且 mean a(41–60) ≤ 1.05 × mean a(21–40) 且 max m(41–60) < 0.9 × 结构上限。
pub(super) fn judge_deletion_block_growth(rows: &[RoundRow], structural_bound: u64, thresholds: &GrowthThresholds) -> GrowthVerdict {
    assert!(rows.len() as u64 >= ROUNDS_FIRST_PASS, "60 轮跑完才判");
    let earlier_window_round_indices = 20..40;
    let later_window_round_indices = 40..60;
    let (Some((earlier_peak, earlier_mean)), Some((later_peak, later_mean))) = (
        window_peak_and_mean(rows, earlier_window_round_indices),
        window_peak_and_mean(rows, later_window_round_indices),
    ) else {
        return GrowthVerdict::Unreadable;
    };
    let peak_does_not_grow = later_peak <= earlier_peak;
    let mean_does_not_grow = later_mean * 100.0 <= earlier_mean * thresholds.mean_growth_percent as f64;
    let peak_stays_below_structural_bound = later_peak * 10 < structural_bound * thresholds.structural_bound_tenths;
    if peak_does_not_grow && mean_does_not_grow && peak_stays_below_structural_bound {
        GrowthVerdict::Bounded
    } else {
        GrowthVerdict::Diverging
    }
}

/// 120 轮的加倍复核：max m(61–120) ≤ max m(21–60)，不过改判「发散」。
pub(super) fn judge_doubled_growth_check(rows: &[RoundRow]) -> GrowthVerdict {
    assert!(rows.len() as u64 >= ROUNDS_DOUBLED_CHECK, "120 轮跑完才复核");
    match (window_peak_and_mean(rows, 20..60), window_peak_and_mean(rows, 60..120)) {
        (Some((earlier_peak, _earlier_mean)), Some((later_peak, _later_mean))) => {
            if later_peak <= earlier_peak {
                GrowthVerdict::Bounded
            } else {
                GrowthVerdict::Diverging
            }
        }
        (None, _) | (_, None) => GrowthVerdict::Unreadable,
    }
}

/// 第 4 行：全程「删除发不出去」的次数（删除类与抬 F 的空发布两栏之和，登记 M-5 那一条）。
pub(super) fn deletion_failures_of(rows: &[RoundRow]) -> u64 {
    rows.iter().map(|row| row.metrics.deletion_publish_failures + row.metrics.empty_publish_failures).sum()
}

/// 五个种子合成一格一臂的判定：五个都判同一个值才写那个值，否则「不稳定」（登记第六节开头）。
pub(super) fn combine_seed_tags(tags: &[&'static str]) -> &'static str {
    assert_eq!(tags.len() as u64, SEEDS_PER_CELL, "一格一臂恰好五个种子");
    if tags.iter().all(|tag| *tag == tags[0]) {
        tags[0]
    } else {
        "unstable"
    }
}

pub(super) fn failure_tag(deletion_failures: u64) -> &'static str {
    if deletion_failures == 0 {
        "none"
    } else {
        "failure"
    }
}

/// 一条被谓词消费的量的轨迹（`test-discipline.md`「端点不是轨迹」）：吃光初始供给之后的峰值（第 2 轮起）、为正的轮数、
/// 低于 `N₀` 的轮数、期末值、全程最小值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TrajectoryStatistics {
    pub(super) peak_from_round_two: u64,
    pub(super) rounds_positive: u64,
    rounds_below_floor: u64,
    pub(super) final_value: u64,
    lowest: u64,
}

pub(super) fn trajectory_statistics(values: &[u64], floor_segments: u64) -> TrajectoryStatistics {
    assert!(values.len() >= 2, "至少两轮才有「第 2 轮起」");
    TrajectoryStatistics {
        peak_from_round_two: values[1..].iter().copied().max().expect("非空"),
        rounds_positive: values.iter().filter(|value| **value > 0).count() as u64,
        rounds_below_floor: values.iter().filter(|value| **value < floor_segments).count() as u64,
        final_value: *values.last().expect("非空"),
        lowest: values.iter().copied().min().expect("非空"),
    }
}

pub(super) fn trajectory_fields(prefix: &str, statistics: &TrajectoryStatistics) -> String {
    format!(
        "{prefix}_peak_from_round_two={} {prefix}_positive_rounds={} {prefix}_rounds_below_segment_floor={} {prefix}_final={} {prefix}_lowest={}",
        statistics.peak_from_round_two,
        statistics.rounds_positive,
        statistics.rounds_below_floor,
        statistics.final_value,
        statistics.lowest
    )
}

/// 逐意图的四个量的错配计数（登记第六节第 9、10、13 行）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct IntentErrorCounts {
    pub(super) ended: u64,
    pub(super) discarded: u64,
    pub(super) judged: u64,
    pub(super) region_predicate_true_truth_false: u64,
    pub(super) region_predicate_false_truth_true: u64,
    pub(super) global_difference_true_truth_false: u64,
    pub(super) global_difference_false_truth_true: u64,
    pub(super) global_difference_true_openable_rise_false: u64,
    pub(super) global_difference_false_openable_rise_true: u64,
}

impl IntentErrorCounts {
    pub(super) fn of(ended_intents: &[EndedIntent]) -> Self {
        let mut counts = Self::default();
        for intent in ended_intents {
            counts.ended += 1;
            counts.discarded += u64::from(!intent.is_completed);
            let (Some(truth), Some(openable_rise)) = (intent.region_became_openable, intent.openable_count_rose) else {
                continue;
            };
            counts.judged += 1;
            let predicate = intent.region_is_record_empty;
            let difference = intent.global_difference_is_positive;
            counts.region_predicate_true_truth_false += u64::from(predicate && !truth);
            counts.region_predicate_false_truth_true += u64::from(!predicate && truth);
            counts.global_difference_true_truth_false += u64::from(difference && !truth);
            counts.global_difference_false_truth_true += u64::from(!difference && truth);
            counts.global_difference_true_openable_rise_false += u64::from(difference && !openable_rise);
            counts.global_difference_false_openable_rise_true += u64::from(!difference && openable_rise);
        }
        counts
    }

    pub(super) fn add(&mut self, other: &Self) {
        self.ended += other.ended;
        self.discarded += other.discarded;
        self.judged += other.judged;
        self.region_predicate_true_truth_false += other.region_predicate_true_truth_false;
        self.region_predicate_false_truth_true += other.region_predicate_false_truth_true;
        self.global_difference_true_truth_false += other.global_difference_true_truth_false;
        self.global_difference_false_truth_true += other.global_difference_false_truth_true;
        self.global_difference_true_openable_rise_false += other.global_difference_true_openable_rise_false;
        self.global_difference_false_openable_rise_true += other.global_difference_false_openable_rise_true;
    }

    pub(super) fn fields(&self) -> String {
        format!(
            "intents_ended={} intents_discarded={} intents_judged={} region_is_record_empty_true_region_became_openable_false={} region_is_record_empty_false_region_became_openable_true={} global_difference_is_positive_true_region_became_openable_false={} global_difference_is_positive_false_region_became_openable_true={} global_difference_is_positive_true_openable_count_rose_false={} global_difference_is_positive_false_openable_count_rose_true={}",
            self.ended,
            self.discarded,
            self.judged,
            self.region_predicate_true_truth_false,
            self.region_predicate_false_truth_true,
            self.global_difference_true_truth_false,
            self.global_difference_false_truth_true,
            self.global_difference_true_openable_rise_false,
            self.global_difference_false_openable_rise_true
        )
    }

    /// 第 9 行：`P_R` 判真而 `Y1` 假的占比 ≥ 5%（分母是判过 `Y1` 的意图数；一条都没判过是读不到）。
    pub(super) fn region_predicate_false_positive_share_reaches_threshold(&self) -> Option<bool> {
        if self.judged == 0 {
            return None;
        }
        Some(self.region_predicate_true_truth_false * 100 >= PREDICATE_FALSE_POSITIVE_SHARE_PERCENT * self.judged)
    }
}

pub(super) fn bit_column<Item>(items: &[Item], bit_of: impl Fn(&Item) -> Option<bool>) -> String {
    if items.is_empty() {
        return "empty".to_string();
    }
    items
        .iter()
        .map(|item| match bit_of(item) {
            Some(true) => '1',
            Some(false) => '0',
            None => '-',
        })
        .collect()
}
