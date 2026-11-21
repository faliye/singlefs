//! 抬上限的读数与真值、空间账、保留读数、池大小与满度、上限卡住与全零槽的处置、挂载后的池。

use super::*;

/// 进程内重读一个钉着的槽读到了什么（冻结树 `allocator.rs:1193` 的四类，被抛弃那一类这个模型不建：单实例）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RereadOfAPinnedSlot {
    ValidRoot { checkpoint_txg: u64 },
    AllZero,
    StillUnreadableOrNotSelfVerified,
}

/// 环里最旧有效根，钉着的槽先重读：有效根与钉着的槽里最小的若是一个钉着的槽，重读那一槽，读出有效根就换成它的 txg，
/// 仍读不出或读回全零才按钉着的 txg 算（冻结树 `allocator.rs:1219` `oldest_txg_in_the_ring_rereading_slots_bad_at_mount`）。
/// 循环：每轮要么换掉一个钉着的槽、要么返回；轮数至多钉着的槽数加一。
fn oldest_txg_rereading_pinned_slots(
    valid_root_txgs: &[u64],
    mut pinned: BTreeMap<RingSlot, u64>,
    reread: &mut dyn FnMut(RingSlot) -> RereadOfAPinnedSlot,
) -> (Option<u64>, Vec<(RingSlot, RereadOfAPinnedSlot)>) {
    let mut valid_root_txgs: Vec<u64> = valid_root_txgs.to_vec();
    let mut rereads = Vec::new();
    loop {
        let oldest_valid_root = valid_root_txgs.iter().copied().min();
        let oldest_pinned_slot = pinned.iter().min_by_key(|(_, pinned_txg)| **pinned_txg).map(|(slot, pinned_txg)| (*slot, *pinned_txg));
        let Some((slot, pinned_txg)) = oldest_pinned_slot else {
            return (oldest_valid_root, rereads);
        };
        if oldest_valid_root.is_some_and(|oldest| oldest < pinned_txg) {
            return (oldest_valid_root, rereads);
        }
        let reread_of_the_slot = reread(slot);
        rereads.push((slot, reread_of_the_slot));
        match reread_of_the_slot {
            RereadOfAPinnedSlot::ValidRoot { checkpoint_txg } => {
                pinned.remove(&slot);
                valid_root_txgs.push(checkpoint_txg);
            }
            RereadOfAPinnedSlot::StillUnreadableOrNotSelfVerified | RereadOfAPinnedSlot::AllZero => {
                return (Some(pinned_txg), rereads);
            }
        }
    }
}

/// 挂载时读坏（或读回全零）的槽各能住哪几个 txg（冻结树 `mount.rs:1774` `txgs_ring_slots_bad_at_mount_can_hold`，登记第三节）：
/// 最大的取 (N − 3S, first_txg − 1] 里落到这一槽的最大 txg，最旧的取 [0, min(3S, first_txg)) 里落到这一槽的最小 txg；两个都没有的槽不交回。
/// 循环：最大 txg 只走区间尾 min(3S, 区间长) 个 txg，最旧 txg 只走 [0, min(3S, first_txg))，跨轮只往按槽的表里写。
fn txgs_ring_slots_bad_at_mount_can_hold(
    bad_slots: &BTreeSet<RingSlot>,
    window_anchor: u64,
    first_txg: u64,
    geometry: RingGeometry,
) -> BTreeMap<RingSlot, (u64, u64)> {
    let ring_length_in_txgs = geometry.ring_length_in_txgs();
    let lowest_txg_inside_the_window = window_anchor.checked_sub(ring_length_in_txgs).map_or(0, |below_the_window| below_the_window + 1);
    let interval_length_in_txgs = first_txg.saturating_sub(lowest_txg_inside_the_window);
    let txgs_walked_at_the_tail = interval_length_in_txgs.min(ring_length_in_txgs);
    let mut highest_txg_of_each_slot: BTreeMap<RingSlot, u64> = BTreeMap::new();
    for txg in first_txg - txgs_walked_at_the_tail..first_txg {
        let slot = geometry.slot_of_txg(txg);
        if bad_slots.contains(&slot) {
            highest_txg_of_each_slot.insert(slot, txg);
        }
    }
    let mut oldest_txg_of_each_slot: BTreeMap<RingSlot, u64> = BTreeMap::new();
    for txg in 0..first_txg.min(ring_length_in_txgs) {
        let slot = geometry.slot_of_txg(txg);
        if bad_slots.contains(&slot) {
            oldest_txg_of_each_slot.entry(slot).or_insert(txg);
        }
    }
    oldest_txg_of_each_slot
        .into_iter()
        .map(|(slot, oldest_txg)| {
            let highest_txg = *highest_txg_of_each_slot.get(&slot).expect("这个槽在 [0, first_txg) 里住得下一个 txg，区间尾也住得下一个");
            (slot, (oldest_txg, highest_txg))
        })
        .collect()
}

// ── 抬 F 的上限（冻结树 `mount.rs:2381` `rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`、`mount.rs:2518`） ──

/// 一次算上限读到的两个数：不并任何槽的 C0；并了读坏的槽之后的 C并（Q2 并时再加读回全零的槽）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CeilingReading {
    pub(super) unmerged: u64,
    pub(super) merged: u64,
}

/// 算上限那一遍读到「知道住着根的槽」重读仍坏：抬 F 报自己的错（冻结树 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SlotKnownToHoldARootStillBadAfterOneReread {
    pub(super) slot: RingSlot,
}

/// 第 N 新的非空有效根与「4 个不同状态」：有效根（按 txg 排）里每一条与它前一条有效根的状态不同才算非空，
/// 最旧的一条拿「没有文件」比；从新到旧数，状态号去重（冻结树 `mount.rs:2255`、`2490`）。交回计入的那几条的 txg，从新到旧。
fn newest_first_distinct_non_empty_txgs(valid_roots_sorted_by_txg: &[RootContent]) -> Vec<u64> {
    let mut non_empty_roots: Vec<RootContent> = Vec::new();
    for (position, root) in valid_roots_sorted_by_txg.iter().enumerate() {
        let state_of_the_previous_root = if position == 0 { STATE_WITHOUT_FILE } else { valid_roots_sorted_by_txg[position - 1].state_number };
        if root.state_number != state_of_the_previous_root {
            non_empty_roots.push(*root);
        }
    }
    let mut counted_states: Vec<u32> = Vec::new();
    let mut distinct_txgs = Vec::new();
    for root in non_empty_roots.iter().rev() {
        if counted_states.contains(&root.state_number) {
            continue;
        }
        counted_states.push(root.state_number);
        distinct_txgs.push(root.checkpoint_txg);
    }
    distinct_txgs
}

/// 上限 = min(每块盘上最新的有效根, 第 4 新的不同状态的非空有效根)，去重之后不足 4 个取最旧的有效根（冻结树 `mount.rs:2518`；登记第二节「抬 F 的上限」）。
/// `valid_roots` 是已按「txg ≥ F」滤过的读得出的根。一条有效根都没有交回 `None`。
fn ceiling_of_the_valid_roots(valid_roots: &[RootContent]) -> Option<u64> {
    let mut sorted_by_txg: Vec<RootContent> = valid_roots.to_vec();
    sorted_by_txg.sort_by_key(|root| root.checkpoint_txg);
    let mut newest_per_device: [Option<u64>; DEVICES_IN_THE_POOL] = [None; DEVICES_IN_THE_POOL];
    for root in &sorted_by_txg {
        let device = device_of_txg(root.checkpoint_txg);
        newest_per_device[device] = Some(newest_per_device[device].map_or(root.checkpoint_txg, |newest| newest.max(root.checkpoint_txg)));
    }
    let newest_on_every_device = newest_per_device.iter().flatten().copied().min()?;
    let distinct_non_empty_txgs = newest_first_distinct_non_empty_txgs(&sorted_by_txg);
    let oldest_valid_root = sorted_by_txg.first().map(|root| root.checkpoint_txg);
    let fourth_newest = distinct_non_empty_txgs.get(NEWEST_DISTINCT_STATES_THE_CEILING_KEEPS - 1).copied().or(oldest_valid_root)?;
    Some(newest_on_every_device.min(fourth_newest))
}

/// 真值盘面上的回退候选数 K(F)：真值有效根（txg ≥ F）带的不同带文件状态数（登记第一节「F 越过的状态数」与「回退候选数」）。
pub(super) fn candidate_states_at_the_floor(truth_ring: &[Option<RootContent>], floor: u64) -> usize {
    let states: BTreeSet<u32> = truth_ring
        .iter()
        .flatten()
        .filter(|root| root.checkpoint_txg >= floor && root.state_number != STATE_WITHOUT_FILE)
        .map(|root| root.state_number)
        .collect();
    states.len()
}

/// 真值盘面上、有效 = txg ≥ F 旧时的真值上限 C真，与 F 新越过它时越过的真值非空根数（登记第一节「F 越过的非空根数」）。
/// 交回 (C真, 越过的非空根数)；F 新不高于 C真 时越过数是 0。
pub(super) fn truth_ceiling_and_crossed_roots(truth_ring: &[Option<RootContent>], old_floor: u64, new_floor: u64) -> (u64, u32) {
    let mut valid_truth_roots: Vec<RootContent> = truth_ring.iter().flatten().filter(|root| root.checkpoint_txg >= old_floor).copied().collect();
    valid_truth_roots.sort_by_key(|root| root.checkpoint_txg);
    let truth_ceiling = ceiling_of_the_valid_roots(&valid_truth_roots).expect("真值盘面上至少有一条有效根");
    if new_floor <= truth_ceiling {
        return (truth_ceiling, 0);
    }
    let mut crossed_non_empty_roots = 0;
    for (position, root) in valid_truth_roots.iter().enumerate() {
        let state_of_the_previous_root = if position == 0 { STATE_WITHOUT_FILE } else { valid_truth_roots[position - 1].state_number };
        let is_non_empty_in_the_truth = root.state_number != state_of_the_previous_root;
        if is_non_empty_in_the_truth && root.checkpoint_txg >= truth_ceiling && root.checkpoint_txg < new_floor {
            crossed_non_empty_roots += 1;
        }
    }
    (truth_ceiling, crossed_non_empty_roots)
}

// ── 空间：空闲、待回收、扣住 ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReclaimedReuse {
    Immediately,
    HeldUntilTheFloorTakesEffect,
}

/// 块的账：空闲（可发出去的）、扣住（回收了、抬 F 那一串做完才放开的）、待回收（释放代 → 块数）。
#[derive(Clone, Debug)]
pub(super) struct SpaceBook {
    pub(super) free_blocks: i64,
    held_blocks: i64,
    pending_by_release_txg: BTreeMap<u64, i64>,
    pub(super) reclaim_disabled: bool,
}

impl SpaceBook {
    fn record_release(&mut self, released_at_txg: u64, blocks: i64) {
        *self.pending_by_release_txg.entry(released_at_txg).or_insert(0) += blocks;
    }

    /// 已释放 ∧ 释放代 ≤ 门槛 的转成空闲（或扣住）；交回这一次回收的块数。
    pub(super) fn reclaim_up_to(&mut self, gate: u64, reuse: ReclaimedReuse) -> i64 {
        if self.reclaim_disabled {
            return 0;
        }
        let reclaimable_release_txgs: Vec<u64> = self.pending_by_release_txg.range(..=gate).map(|(release_txg, _)| *release_txg).collect();
        let mut reclaimed_blocks = 0;
        for release_txg in reclaimable_release_txgs {
            reclaimed_blocks += self.pending_by_release_txg.remove(&release_txg).expect("刚列出来的释放代");
        }
        match reuse {
            ReclaimedReuse::Immediately => self.free_blocks += reclaimed_blocks,
            ReclaimedReuse::HeldUntilTheFloorTakesEffect => self.held_blocks += reclaimed_blocks,
        }
        reclaimed_blocks
    }

    pub(super) fn release_the_holds(&mut self) {
        self.free_blocks += self.held_blocks;
        self.held_blocks = 0;
    }

    pub(super) fn pending_blocks(&self) -> i64 {
        self.pending_by_release_txg.values().sum()
    }
}

// ── 池的松紧（登记第五节「池」） ────────────────────────────────────────────────────────

/// 保留池的读法。原登记第五节「准入」的两种：代码读法 c，条款读法 10 + 7 c。
/// 重跑登记 `research/prompts/e173-r1-prereg.md` 第五节「第二段的两条臂」另加两种按 ckpt_cost 现算的（κ_G 与 c实_G 是第一段在那个几何上量出来的数，
/// 常量表 [`CHECKPOINT_COST_GEOMETRIES`]），与第十二节修订加的判别力对照（保留池改回 10 + 7 c实）；这三种只在 `checkpoint-cost-reserve` 模式里用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ReserveReading {
    CodeReadingOneEmptyPublish,
    ClauseReadingTenPlusSevenEmptyPublishes,
    /// 代码读法（现算）：P代 = κ_G。
    CheckpointCostCodeReading { pool_size: SampledPoolSize, checkpoint_cost_blocks: i64 },
    /// 条款读法（现算）：P条 = 2 × 5 + (B − 1) × κ_G = 10 + 7 κ_G（D16 已定项 1「`df`」那一行，c_max 照 D28 已定项 4 末条取 κ_G）。
    CheckpointCostClauseReading { pool_size: SampledPoolSize, checkpoint_cost_blocks: i64 },
    /// 判别力对照：同一个几何、同一个 c实，保留池改回 10 + 7 c实（原装置条款读法那一式）；读数必须与按 κ_G 的两条臂不同。
    RevertedToTenPlusSevenActualEmptyPublishes { pool_size: SampledPoolSize, checkpoint_cost_blocks: i64 },
}

impl ReserveReading {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::CodeReadingOneEmptyPublish => "code",
            Self::ClauseReadingTenPlusSevenEmptyPublishes => "clause",
            Self::CheckpointCostCodeReading { .. } => "ckpt-code",
            Self::CheckpointCostClauseReading { .. } => "ckpt-clause",
            Self::RevertedToTenPlusSevenActualEmptyPublishes { .. } => "reverted-10+7c",
        }
    }

    /// 条款读法：2 × 5 + (B − 1) × c_max（D16 已定项 1「`df`」那一行；登记 A2：c_max = 5 / 10 时 45 / 80）。
    pub(super) fn reserve_blocks(self, metadata_blocks: i64) -> i64 {
        let empty_publishes_of_one_admission = i64::try_from(PUBLISHES_PER_ADMISSION_AT_MOST - 1).expect("B");
        match self {
            Self::CodeReadingOneEmptyPublish => metadata_blocks,
            Self::ClauseReadingTenPlusSevenEmptyPublishes => {
                2 * 5 + i64::try_from(PUBLISHES_PER_ADMISSION_AT_MOST - 1).expect("B") * metadata_blocks
            }
            Self::CheckpointCostCodeReading { checkpoint_cost_blocks, .. } => checkpoint_cost_blocks,
            Self::CheckpointCostClauseReading { checkpoint_cost_blocks, .. } => 2 * 5 + empty_publishes_of_one_admission * checkpoint_cost_blocks,
            Self::RevertedToTenPlusSevenActualEmptyPublishes { .. } => 2 * 5 + empty_publishes_of_one_admission * metadata_blocks,
        }
    }

    /// 按 ckpt_cost 现算的三种带着几何的池大小；原两种没有这一维，交回空串（产物行里不多打字段，原三个模式的输出逐字节不变，停机条款 S1）。
    pub(super) fn pool_size_label(self) -> &'static str {
        match self {
            Self::CodeReadingOneEmptyPublish | Self::ClauseReadingTenPlusSevenEmptyPublishes => "",
            Self::CheckpointCostCodeReading { pool_size, .. }
            | Self::CheckpointCostClauseReading { pool_size, .. }
            | Self::RevertedToTenPlusSevenActualEmptyPublishes { pool_size, .. } => pool_size.label(),
        }
    }

    /// 键后面多打的几个字段：池大小与 κ_G（原两种交回空串）。
    pub(super) fn extra_key_fields(self) -> String {
        match self {
            Self::CodeReadingOneEmptyPublish | Self::ClauseReadingTenPlusSevenEmptyPublishes => String::new(),
            Self::CheckpointCostCodeReading { pool_size, checkpoint_cost_blocks }
            | Self::CheckpointCostClauseReading { pool_size, checkpoint_cost_blocks }
            | Self::RevertedToTenPlusSevenActualEmptyPublishes { pool_size, checkpoint_cost_blocks } => {
                format!(" pool_size={} kappa={checkpoint_cost_blocks}", pool_size.label())
            }
        }
    }
}

/// 第一段量过的三档池大小（重跑登记第一节「几何」）：G-min、两块各 1 GiB、两块各 4 GiB。G-min 带它取到的每块盘 MiB 数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum SampledPoolSize {
    SmallestCompleting { mebibytes: u64 },
    OneGibibyte,
    FourGibibytes,
}

impl SampledPoolSize {
    /// 与第一段产物 `name=geometry` 行的 `pool_size=` 同一写法（入库装置 `PoolSize::label`）。
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::SmallestCompleting { mebibytes: 32 } => "g_min_32mib", // naming-lint:external 第一段入库装置 PoolSize::label 定的几何写法，读它的产物要逐字一样
            Self::SmallestCompleting { mebibytes: 64 } => "g_min_64mib", // naming-lint:external 第一段入库装置 PoolSize::label 定的几何写法，读它的产物要逐字一样
            Self::SmallestCompleting { mebibytes: 128 } => "g_min_128mib", // naming-lint:external 第一段入库装置 PoolSize::label 定的几何写法，读它的产物要逐字一样
            Self::SmallestCompleting { mebibytes: 256 } => "g_min_256mib", // naming-lint:external 第一段入库装置 PoolSize::label 定的几何写法，读它的产物要逐字一样
            Self::SmallestCompleting { mebibytes: 512 } => "g_min_512mib", // naming-lint:external 第一段入库装置 PoolSize::label 定的几何写法，读它的产物要逐字一样
            Self::SmallestCompleting { .. } => panic!("G-min 只在 32、64、128、256、512 MiB 里取（重跑登记第五节「G-min 的找法」）"),
            Self::OneGibibyte => "g_1", // naming-lint:external 第一段入库装置 PoolSize::label 定的几何写法，读它的产物要逐字一样
            Self::FourGibibytes => "g_4", // naming-lint:external 第一段入库装置 PoolSize::label 定的几何写法，读它的产物要逐字一样
        }
    }

    /// 从第一段产物的几何写法 `s<S>_<池大小>` 读回 (S, 池大小)；认不出的交回 None。
    pub(super) fn of_the_geometry_label(geometry_label: &str) -> Option<(u64, Self)> {
        let (slots_text, pool_text) = geometry_label.strip_prefix('s')?.split_once('_')?;
        let slots_per_region = slots_text.parse::<u64>().ok()?;
        let pool_size = match pool_text {
            "g_1" => Self::OneGibibyte,
            "g_4" => Self::FourGibibytes,
            other => Self::SmallestCompleting { mebibytes: other.strip_prefix("g_min_")?.strip_suffix("mib")?.parse::<u64>().ok()? },
        };
        Some((slots_per_region, pool_size))
    }
}

/// 池的松紧：写行与暖机做完那一刻量（登记第五节「池」：近满 k = 0 与 k = 2，宽松按整条请求流不靠回收也发得完）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum PoolFullness {
    NearlyFullWithNoSpareOverwrite,
    NearlyFullWithTwoSpareOverwrites,
    Roomy,
    /// 只给阳性对照 PC1、PC3 用（登记第五节：近满 k = 8），不进扫描。
    NearlyFullWithEightSpareOverwritesForControlsOnly,
}

impl PoolFullness {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::NearlyFullWithNoSpareOverwrite => "near0",
            Self::NearlyFullWithTwoSpareOverwrites => "near2",
            Self::NearlyFullWithEightSpareOverwritesForControlsOnly => "near8",
            Self::Roomy => "roomy",
        }
    }

    pub(super) fn is_nearly_full(self) -> bool {
        match self {
            Self::NearlyFullWithNoSpareOverwrite
            | Self::NearlyFullWithTwoSpareOverwrites
            | Self::NearlyFullWithEightSpareOverwritesForControlsOnly => true,
            Self::Roomy => false,
        }
    }

    pub(super) fn free_blocks_after_the_warm_up(self, geometry: RingGeometry, metadata_blocks: i64, reserve_blocks: i64) -> i64 {
        let blocks_per_overwrite = DATA_BLOCKS_PER_OVERWRITE + metadata_blocks;
        match self {
            Self::NearlyFullWithNoSpareOverwrite => reserve_blocks,
            Self::NearlyFullWithTwoSpareOverwrites => reserve_blocks + 2 * blocks_per_overwrite,
            Self::NearlyFullWithEightSpareOverwritesForControlsOnly => reserve_blocks + 8 * blocks_per_overwrite,
            Self::Roomy => {
                let requests = i64::try_from(requests_per_mount(geometry)).expect("请求数");
                let ring_length = i64::try_from(geometry.ring_length_in_txgs()).expect("环长");
                reserve_blocks + (requests + ring_length) * blocks_per_overwrite + 100
            }
        }
    }
}

// ── 臂与对照钩子 ─────────────────────────────────────────────────────────────────────────

/// Q1 四条臂（登记第五节「Q1 四条臂」）：上限卡在读坏槽上时怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StuckCeilingDisposition {
    AsToday,
    PushEmptyWithinBudget,
    PushEmptyBeyondBudget,
    DoNotMergeTheBadSlots,
}

impl StuckCeilingDisposition {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::AsToday => "today",
            Self::PushEmptyWithinBudget => "pushB",
            Self::PushEmptyBeyondBudget => "pushW",
            Self::DoNotMergeTheBadSlots => "nomerge",
        }
    }

    pub(super) fn every_arm() -> [Self; 4] {
        [Self::AsToday, Self::PushEmptyWithinBudget, Self::PushEmptyBeyondBudget, Self::DoNotMergeTheBadSlots]
    }
}

/// Q2 两条臂（登记第五节「Q2 两条臂」）：读回全零的槽并不并进上限。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AllZeroSlotDisposition {
    NotMerged,
    Merged,
}

impl AllZeroSlotDisposition {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::NotMerged => "zeronot",
            Self::Merged => "zeromerge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Arm {
    pub(super) stuck_ceiling: StuckCeilingDisposition,
    pub(super) all_zero_slot: AllZeroSlotDisposition,
}

/// 阳性对照与单测用的钩子（登记第五节「阳性对照」）：种下一个已知效应，看计数器读得出它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Controls {
    /// PC2：被测挂载里回收一次都不做。
    pub(super) reclaim_disabled: bool,
    /// PC1：第一次抬 F 把目标改成「它算出的上限 + 这个数」。
    pub(super) first_raise_target_above_the_ceiling_by: Option<u64>,
    /// PC3：第一次抬 F 把目标改成现行那一版的 txg。
    pub(super) first_raise_target_is_the_current_txg: bool,
    /// PC4：第一次请求里每一次判够不够都当「不够」。
    pub(super) first_request_is_judged_short_throughout: bool,
    /// PC4：第一次 `FloorAlreadyAtTheCeiling` 时先推一次空发布再判。
    pub(super) push_one_empty_publish_on_the_first_floor_already_at_the_ceiling: bool,
    /// 一次准入最多推几次发布（产品值 B）；只有单测拿它找「B 差一」会改变读数的格。
    pub(super) publishes_per_admission_at_most: usize,
}

impl Controls {
    pub(super) fn none() -> Self {
        Self {
            reclaim_disabled: false,
            first_raise_target_above_the_ceiling_by: None,
            first_raise_target_is_the_current_txg: false,
            first_request_is_judged_short_throughout: false,
            push_one_empty_publish_on_the_first_floor_already_at_the_ceiling: false,
            publishes_per_admission_at_most: PUBLISHES_PER_ADMISSION_AT_MOST,
        }
    }
}

// ── 一次被测挂载里的池 ──────────────────────────────────────────────────────────────────

/// 可再分配谓词的门槛（D16 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」；冻结树 `mount.rs:1409` `reclaim_floor`）。
pub(super) fn reclaim_floor(effective_floor: u64, oldest_valid_root: Option<u64>) -> u64 {
    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
}

fn reread_of_a_pinned_slot(
    truth_ring: &[Option<RootContent>],
    geometry: RingGeometry,
    fault: Option<ReadFault>,
    clock: FaultClock,
    slot: RingSlot,
) -> RereadOfAPinnedSlot {
    match read_slot(truth_ring, geometry, fault, clock, slot) {
        SlotReading::Root(root) => RereadOfAPinnedSlot::ValidRoot { checkpoint_txg: root.checkpoint_txg },
        SlotReading::AllZero => RereadOfAPinnedSlot::AllZero,
        SlotReading::DeviceError | SlotReading::NotSelfVerified => RereadOfAPinnedSlot::StillUnreadableOrNotSelfVerified,
    }
}

#[derive(Clone, Debug)]
pub(super) struct MountedPool {
    pub(super) geometry: RingGeometry,
    pub(super) metadata_blocks: i64,
    reserve_blocks: i64,
    pub(super) truth_ring: Vec<Option<RootContent>>,
    pub(super) table: RootRingTable,
    pub(super) space: SpaceBook,
    pub(super) current_txg: u64,
    pub(super) current_state_number: u32,
    pub(super) floor_in_the_current_version: u64,
    fault: Option<ReadFault>,
    pub(super) clock: FaultClock,
}

impl MountedPool {
    /// 重建分配器（读根环、记根环表），再写行与暖机（登记第五节「模型·根环表」「模型·回收」；冻结树 `mount.rs:1858`）。
    /// 写行与暖机从切换预留出、不动空闲；做完那一刻空闲取 `free_blocks_after_the_warm_up`，待回收是释放代高于这一刻门槛的那些。
    pub(super) fn mount(
        history: &PoolAfterHistory,
        fault: Option<ReadFault>,
        reserve_blocks: i64,
        free_blocks_after_the_warm_up: i64,
        reclaim_disabled: bool,
    ) -> Self {
        let geometry = history.geometry;
        let first_readings: Vec<(RingSlot, SlotReading)> = geometry
            .every_slot()
            .into_iter()
            .map(|slot| (slot, read_slot(&history.ring, geometry, fault, FaultClock::at_the_start_of_the_mount(), slot)))
            .collect();
        let window_anchor = first_readings
            .iter()
            .filter_map(|(_, reading)| match reading {
                SlotReading::Root(root) => Some(root.checkpoint_txg),
                SlotReading::DeviceError | SlotReading::NotSelfVerified | SlotReading::AllZero => None,
            })
            .max()
            .expect("读阶段至少读出一条有效根");
        let first_txg = history.highest_txg_used + 1;
        let slots_reading = |wanted: fn(&SlotReading) -> bool| -> BTreeSet<RingSlot> {
            first_readings.iter().filter(|(_, reading)| wanted(reading)).map(|(slot, _)| *slot).collect()
        };
        let bad_and_not_all_zero = slots_reading(|reading| matches!(reading, SlotReading::DeviceError | SlotReading::NotSelfVerified));
        let reading_back_all_zero = slots_reading(|reading| matches!(reading, SlotReading::AllZero));
        let mut occupants: BTreeMap<RingSlot, Occupant> = BTreeMap::new();
        for (slot, (oldest, highest)) in txgs_ring_slots_bad_at_mount_can_hold(&bad_and_not_all_zero, window_anchor, first_txg, geometry) {
            occupants.insert(slot, Occupant::BadAtMount { oldest_txg_it_can_have_held: oldest, highest_txg_it_can_hold: highest });
        }
        for (slot, (oldest, highest)) in txgs_ring_slots_bad_at_mount_can_hold(&reading_back_all_zero, window_anchor, first_txg, geometry) {
            occupants.insert(slot, Occupant::AllZeroAtMount { oldest_txg_it_can_have_held: oldest, highest_txg_it_can_hold: highest });
        }
        for (slot, reading) in &first_readings {
            if let SlotReading::Root(root) = reading {
                occupants.insert(*slot, Occupant::ValidRoot { checkpoint_txg: root.checkpoint_txg });
            }
        }
        let mut space = SpaceBook { free_blocks: 0, held_blocks: 0, pending_by_release_txg: BTreeMap::new(), reclaim_disabled };
        for release in &history.releases {
            space.record_release(release.released_at_txg, release.blocks);
        }
        let mut pool = Self {
            geometry,
            metadata_blocks: history.metadata_blocks,
            reserve_blocks,
            truth_ring: history.ring.clone(),
            table: RootRingTable {
                occupants,
                effective_floor_at_the_mount: history.effective_floor,
                latest_recorded_txg: first_txg - 1,
            },
            space,
            current_txg: history.highest_txg_used,
            current_state_number: history.newest_state_number,
            floor_in_the_current_version: history.effective_floor,
            fault,
            clock: FaultClock::at_the_start_of_the_mount(),
        };
        let mount_publish_txgs: Vec<u64> = std::iter::once(first_txg).chain(warm_up_publish_txgs(first_txg)).collect();
        for mount_publish_txg in mount_publish_txgs {
            let state_number = pool.current_state_number;
            let floor = pool.floor_in_the_current_version;
            let metadata_blocks = pool.metadata_blocks;
            pool.publish_root(mount_publish_txg, state_number, 0, metadata_blocks, floor, true);
        }
        pool.clock.warm_up_done = true;
        pool.space.free_blocks = free_blocks_after_the_warm_up;
        pool.space.held_blocks = 0;
        pool
    }

    /// 一次发布的根落盘：占 `placed_blocks`、放掉 `released_blocks`（释放代 = 这次发布的 txg），真值槽换成这条根，记进根环表。
    pub(super) fn publish_root(&mut self, checkpoint_txg: u64, state_number: u32, placed_blocks: i64, released_blocks: i64, carried_floor: u64, rereading: bool) {
        self.space.free_blocks -= placed_blocks;
        assert!(self.space.free_blocks >= 0, "发布占的块取得到落点：调用方先判过空闲");
        self.space.record_release(checkpoint_txg, released_blocks);
        let written_slot = self.geometry.slot_of_txg(checkpoint_txg);
        let flat = self.geometry.flat_index(written_slot);
        self.truth_ring[flat] = Some(RootContent { checkpoint_txg, state_number, carried_floor });
        if self.fault.is_some_and(|fault| fault.slot == written_slot) {
            self.clock.slot_written_by_the_process = true;
        }
        self.record_root(checkpoint_txg, rereading);
        self.current_txg = checkpoint_txg;
        self.current_state_number = state_number;
    }

    /// 这个进程写了一条根，记进根环表（冻结树 `allocator.rs:1914`、`1966`）：记之前先照进程内重读那一遍重读钉着的槽
    /// （`rereading`，预演不重读），再把这条根记成它那一槽的有效根，按门槛 max(挂载时的 F_生效, 环里最旧有效根) 回收。
    fn record_root(&mut self, checkpoint_txg: u64, rereading: bool) {
        assert_eq!(checkpoint_txg, self.table.latest_recorded_txg + 1, "装了根环表的进程写的根 txg 逐个加一");
        let written_slot = self.geometry.slot_of_txg(checkpoint_txg);
        if rereading {
            let mut valid_root_txgs: Vec<u64> = self
                .table
                .occupants
                .iter()
                .filter(|(slot, _)| **slot != written_slot)
                .filter_map(|(_, occupant)| match occupant {
                    Occupant::ValidRoot { checkpoint_txg: txg } | Occupant::ValidRootReadWhenRereading { checkpoint_txg: txg } => Some(*txg),
                    Occupant::BadAtMount { .. } | Occupant::AllZeroAtMount { .. } => None,
                })
                .collect();
            valid_root_txgs.push(checkpoint_txg);
            let mut pinned = self.table.pinned_oldest_txgs();
            pinned.remove(&written_slot);
            let (truth_ring, geometry, fault, clock) = (&self.truth_ring, self.geometry, self.fault, self.clock);
            let (_oldest, rereads) = oldest_txg_rereading_pinned_slots(&valid_root_txgs, pinned, &mut |slot| {
                reread_of_a_pinned_slot(truth_ring, geometry, fault, clock, slot)
            });
            for (slot, outcome) in rereads {
                match outcome {
                    RereadOfAPinnedSlot::ValidRoot { checkpoint_txg: txg_read } => {
                        self.table.occupants.insert(slot, Occupant::ValidRootReadWhenRereading { checkpoint_txg: txg_read });
                    }
                    RereadOfAPinnedSlot::AllZero | RereadOfAPinnedSlot::StillUnreadableOrNotSelfVerified => {}
                }
            }
        }
        self.table.occupants.insert(written_slot, Occupant::ValidRoot { checkpoint_txg });
        self.table.latest_recorded_txg = checkpoint_txg;
        let gate = reclaim_floor(self.table.effective_floor_at_the_mount, self.table.oldest_valid_root_in_the_ring());
        self.space.reclaim_up_to(gate, ReclaimedReuse::Immediately);
    }

    fn read(&self, slot: RingSlot) -> SlotReading {
        read_slot(&self.truth_ring, self.geometry, self.fault, self.clock, slot)
    }

    /// 算上限那一遍读根环（冻结树 `mount.rs:2273` `roots_of_the_ring_for_the_ceiling`、`mount.rs:2381`）：每槽读一次；读坏而不是「这个进程知道住着根的槽」的，
    /// 记进「这一遍读坏、当没有根」；知道住着根的重读一次，仍坏就是抬 F 报自己的错。
    /// 有效 = 读得出 ∧ txg ≥ 现行那一版带的 F。C0 不并任何槽；C并 并上「这一遍读坏」∩ 根环表里能封顶的槽，只留能住的最大 txg ≥ F 的，取最小者封顶。
    pub(super) fn ceiling_pass(&self, merge_all_zero_slots: bool) -> Result<CeilingReading, SlotKnownToHoldARootStillBadAfterOneReread> {
        let mut roots_read: Vec<RootContent> = Vec::new();
        let mut bad_slots_taken_as_holding_no_root: BTreeSet<RingSlot> = BTreeSet::new();
        for slot in self.geometry.every_slot() {
            match self.read(slot) {
                SlotReading::Root(root) => roots_read.push(root),
                SlotReading::DeviceError | SlotReading::NotSelfVerified | SlotReading::AllZero => {
                    if !self.table.knows_a_root_in(slot) {
                        bad_slots_taken_as_holding_no_root.insert(slot);
                        continue;
                    }
                    match self.read(slot) {
                        SlotReading::Root(root) => roots_read.push(root),
                        SlotReading::DeviceError | SlotReading::NotSelfVerified | SlotReading::AllZero => {
                            return Err(SlotKnownToHoldARootStillBadAfterOneReread { slot });
                        }
                    }
                }
            }
        }
        let valid_roots: Vec<RootContent> =
            roots_read.into_iter().filter(|root| root.checkpoint_txg >= self.floor_in_the_current_version).collect();
        let unmerged = ceiling_of_the_valid_roots(&valid_roots).expect("读得出的有效根至少一条");
        let lowest_highest_txg_a_bad_slot_can_hold = bad_slots_taken_as_holding_no_root
            .iter()
            .filter_map(|slot| self.table.highest_txg_the_cap_takes_from(*slot, merge_all_zero_slots))
            .filter(|highest_txg| *highest_txg >= self.floor_in_the_current_version)
            .min();
        let merged = lowest_highest_txg_a_bad_slot_can_hold.map_or(unmerged, |highest_txg| unmerged.min(highest_txg));
        Ok(CeilingReading { unmerged, merged })
    }

    /// 抬 F 那一遍的回收门槛里「环里最旧有效根」（冻结树 `mount.rs:3023`）：这一遍读出来的根并上根环表里钉着、这一遍也没在那一槽读出根的槽，
    /// 最小的是钉着的槽时先重读那一槽。只读，不改池。
    pub(super) fn oldest_valid_root_for_raising_the_floor(&self) -> Option<u64> {
        let roots_read: Vec<RootContent> = self
            .geometry
            .every_slot()
            .into_iter()
            .filter_map(|slot| match self.read(slot) {
                SlotReading::Root(root) => Some(root),
                SlotReading::DeviceError | SlotReading::NotSelfVerified | SlotReading::AllZero => None,
            })
            .collect();
        let valid_root_txgs: Vec<u64> = roots_read.iter().map(|root| root.checkpoint_txg).collect();
        let slots_read: BTreeSet<RingSlot> = roots_read.iter().map(|root| self.geometry.slot_of_txg(root.checkpoint_txg)).collect();
        let pinned: BTreeMap<RingSlot, u64> =
            self.table.pinned_oldest_txgs().into_iter().filter(|(slot, _)| !slots_read.contains(slot)).collect();
        let (truth_ring, geometry, fault, clock) = (&self.truth_ring, self.geometry, self.fault, self.clock);
        oldest_txg_rereading_pinned_slots(&valid_root_txgs, pinned, &mut |slot| reread_of_a_pinned_slot(truth_ring, geometry, fault, clock, slot)).0
    }

    pub(super) fn free_blocks_above_the_reserve(&self) -> i64 {
        self.space.free_blocks - self.reserve_blocks
    }
}
