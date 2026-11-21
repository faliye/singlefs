//! 池的几何与派生公式、随机源、槽与段的状态类型、一次发布的种类与拒绝、整理意图、池本身的结构体。

use super::*;

/// 一个池的几何与这一格的参数：S、c、叶宽、盘大小。由它推出段数、树高、b 与文件树节点数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PoolGeometry {
    pub(super) disk_slots: u64,
    pub(super) leaf_width_in_slots: u64,
    pub(super) root_ring_slots_per_region: u64,
    pub(super) commit_generated_blocks_per_publish: u64,
}

impl PoolGeometry {
    pub(super) fn new(
        disk_slots: u64,
        leaf_width_in_slots: u64,
        root_ring_slots_per_region: u64,
        commit_generated_blocks_per_publish: u64,
    ) -> Self {
        assert!(
            (ROOT_RING_SLOTS_PER_REGION_LOWER_BOUND..=ROOT_RING_SLOTS_PER_REGION_UPPER_BOUND)
                .contains(&root_ring_slots_per_region),
            "S 在 [4, 16] 里（crates/singlefs-format/src/lib.rs 的上下界）"
        );
        assert!(disk_slots > UNIT_AREA_START_SLOT, "盘要比单元区起点大");
        assert_eq!(
            (disk_slots - UNIT_AREA_START_SLOT) % CLUSTER_SEGMENT_SLOTS,
            0,
            "模型只建整段的单元区"
        );
        let geometry = Self {
            disk_slots,
            leaf_width_in_slots,
            root_ring_slots_per_region,
            commit_generated_blocks_per_publish,
        };
        assert!(
            geometry.record_tree_height() <= 3,
            "模型只建叶、层级 1 与根三层（登记 M-3）"
        );
        assert!(
            commit_generated_blocks_per_publish >= geometry.record_tree_blocks_per_publish_at_one_leaf(),
            "c 至少要装下分配记录树那 b 块"
        );
        geometry
    }

    pub(super) fn unit_area_slots(&self) -> u64 {
        self.disk_slots - UNIT_AREA_START_SLOT
    }

    pub(super) fn segment_count(&self) -> u64 {
        self.unit_area_slots() / CLUSTER_SEGMENT_SLOTS
    }

    /// 层级 1 节点罩的槽数：`W × 169`（`crates/singlefs-core/src/allocation_record_tree.rs` 模块文档的 S_1）。
    pub(super) fn level_one_width_in_slots(&self) -> u64 {
        self.leaf_width_in_slots * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
    }

    /// 根的层级：最小的 R ≥ 1 使 Σ_盘 ⌈盘槽数 ÷ S_{R−1}⌉ ≤ 169（`allocation_record_tree.rs` 模块文档「根」那一条）。
    pub(super) fn record_tree_root_level(&self) -> u64 {
        let mut level = 1;
        let mut width_below_root = self.leaf_width_in_slots;
        while DISK_COUNT * self.disk_slots.div_ceil(width_below_root) > ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
            level += 1;
            width_below_root *= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
        }
        level
    }

    fn record_tree_height(&self) -> u64 {
        self.record_tree_root_level() + 1
    }

    pub(super) fn has_level_one_nodes_below_the_root(&self) -> bool {
        self.record_tree_height() == 3
    }

    /// b：脏槽只落在一片叶、一个层级 1 节点时分配记录树写出的块数，`2 × (树高 − 1) + 1`（登记 M-3）。
    pub(super) fn record_tree_blocks_per_publish_at_one_leaf(&self) -> u64 {
        DISK_COUNT * (self.record_tree_height() - 1) + 1
    }

    /// 文件树与中央映射树的节点数：恒为 `c − b`（登记 M-3 的登记取法，已登记差异 ③）。
    pub(super) fn file_tree_nodes_per_publish(&self) -> u64 {
        self.commit_generated_blocks_per_publish - self.record_tree_blocks_per_publish_at_one_leaf()
    }

    pub(super) fn leaf_of_absolute_slot(&self, absolute_slot: u64) -> u64 {
        absolute_slot / self.leaf_width_in_slots
    }

    pub(super) fn level_one_node_of_absolute_slot(&self, absolute_slot: u64) -> u64 {
        absolute_slot / self.level_one_width_in_slots()
    }

    /// 单元区里有几片叶（第一片与最后一片之间，含两端）。
    pub(super) fn leaves_covering_the_unit_area(&self) -> u64 {
        self.leaf_of_absolute_slot(self.disk_slots - 1) - self.leaf_of_absolute_slot(UNIT_AREA_START_SLOT) + 1
    }

    fn level_one_nodes_covering_the_unit_area(&self) -> u64 {
        if !self.has_level_one_nodes_below_the_root() {
            return 0;
        }
        self.level_one_node_of_absolute_slot(self.disk_slots - 1)
            - self.level_one_node_of_absolute_slot(UNIT_AREA_START_SLOT)
            + 1
    }

    /// 结构上限：(c − b) + 2 × 叶数 + 2 × 层级 1 节点数 + 1（登记第六节第 3 行）。
    pub(super) fn structural_commit_generated_block_bound(&self) -> u64 {
        self.file_tree_nodes_per_publish()
            + DISK_COUNT * self.leaves_covering_the_unit_area()
            + DISK_COUNT * self.level_one_nodes_covering_the_unit_area()
            + 1
    }

    /// 保持期：一个释放代 t 的槽在发布 txg ≥ t + 3S − 1 时才可回收（登记 M-2：环满之后最旧有效根 = 现在的 txg − 3S + 1）。
    pub(super) fn retention_publishes(&self) -> u64 {
        ROOT_RING_REGIONS * self.root_ring_slots_per_region - 1
    }
}

/// 候选 NF 的算式值 `⌈(3S − 1) × c ÷ 64⌉ + 1`（D3（空间分配） 已定项 7 登记表第 7 行）。
pub(super) fn formula_floor_segments(root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64) -> u64 {
    ((ROOT_RING_REGIONS * root_ring_slots_per_region - 1) * commit_generated_blocks_per_publish)
        .div_ceil(CLUSTER_SEGMENT_SLOTS)
        + 1
}

/// 整理入口甲一次调用的发布预算 `B_gc = max(1, 3S − 4 − e)`，e = 离最近一条非空根已经过去的发布数（判决 G8，登记 M-7）。
pub(super) fn compaction_call_publish_budget(root_ring_slots_per_region: u64, publishes_since_last_non_empty_root: u64) -> u64 {
    (ROOT_RING_REGIONS * root_ring_slots_per_region)
        .saturating_sub(4)
        .saturating_sub(publishes_since_last_non_empty_root)
        .max(1)
}

/// 一个已释放的槽在某次发布时可不可回收：释放代 ≤ max(F_生效, 环里最旧有效根)（D16（发布语义） 已定项 1；登记 M-2）。
pub(super) fn reclaim_threshold_at_publish(publish_txg: u64, effective_floor_txg: u64, geometry: &PoolGeometry) -> u64 {
    let oldest_valid_root_txg = publish_txg.saturating_sub(geometry.retention_publishes());
    effective_floor_txg.max(oldest_valid_root_txg)
}

/// 抬 F 的目标：环里第 4 新的非空根的 txg；不足 4 个取环里最旧有效根（D16（发布语义） 已定项 1「抬 F 的上限」）。
/// `non_empty_root_txgs_in_ring` 从旧到新排。
pub(super) fn floor_raise_target(non_empty_root_txgs_in_ring: &VecDeque<u64>, oldest_valid_root_txg: u64) -> u64 {
    if non_empty_root_txgs_in_ring.len() < RETAINED_NON_EMPTY_STATES {
        return oldest_valid_root_txg;
    }
    non_empty_root_txgs_in_ring[non_empty_root_txgs_in_ring.len() - RETAINED_NON_EMPTY_STATES]
}

/// 种子化的随机源（SplitMix64）。模型里唯一的随机源。
pub(super) struct SeededRandom {
    state: u64,
}

impl SeededRandom {
    pub(super) fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_word(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    /// [0, bound) 里均匀取一个数（bound ≥ 1）。取模偏差在 bound ≪ 2⁶⁴ 时可略，模型只要确定性。
    pub(super) fn below(&mut self, bound: u64) -> u64 {
        assert!(bound >= 1, "空区间里取不出数");
        self.next_word() % bound
    }
}

/// 一块提交内生块是谁：文件树（含中央映射树）节点、分配记录树的叶、层级 1 节点（每块盘一份，按位置）、根（登记 M-3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum BlockIdentity {
    FileTreeNode { node: u64 },
    RecordTreeLeaf { disk: u64, leaf: u64 },
    RecordTreeLevelOneNode { disk: u64, node: u64 },
    RecordTreeRoot,
}

/// 一个已分配槽上那条记录罩住的是什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SlotOwner {
    /// mkfs 的实例表两槽：建池时钉住、整段跑下来不释放不搬（登记 M-2 的登记取法）。
    InstanceTable,
    /// 用户数据单元，两槽；`unit` 是单元的身份（中央映射 key 的模型）。
    UserDataUnit { unit: u32 },
    CommitGeneratedBlock { block: BlockIdentity },
    /// 被快照钉住的那一版文件树节点：被快照根直接指着，区域扫描记「钉住」（登记 M-8）。
    SnapshotPinnedFileTreeNode,
}

impl SlotOwner {
    pub(super) fn is_pinned(self) -> bool {
        match self {
            SlotOwner::InstanceTable | SlotOwner::SnapshotPinnedFileTreeNode => true,
            SlotOwner::UserDataUnit { .. } | SlotOwner::CommitGeneratedBlock { .. } => false,
        }
    }

    pub(super) fn is_user_data(self) -> bool {
        match self {
            SlotOwner::UserDataUnit { .. } => true,
            SlotOwner::InstanceTable
            | SlotOwner::SnapshotPinnedFileTreeNode
            | SlotOwner::CommitGeneratedBlock { .. } => false,
        }
    }
}

/// 槽的三态（登记 M-2）：空闲、已分配（未释放记录罩着）、已释放（带释放代，回收之后才空闲）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SlotState {
    Free,
    Live { owner: SlotOwner, allocation_txg: u64 },
    Released { release_txg: u64 },
}

impl SlotState {
    pub(super) fn is_free(self) -> bool {
        self == SlotState::Free
    }

    pub(super) fn is_live(self) -> bool {
        match self {
            SlotState::Live { .. } => true,
            SlotState::Free | SlotState::Released { .. } => false,
        }
    }

    pub(super) fn is_released(self) -> bool {
        match self {
            SlotState::Released { .. } => true,
            SlotState::Free | SlotState::Live { .. } => false,
        }
    }

    pub(super) fn live_owner(self) -> Option<SlotOwner> {
        match self {
            SlotState::Live { owner, .. } => Some(owner),
            SlotState::Free | SlotState::Released { .. } => None,
        }
    }
}

/// 一个聚簇段的增量计数。全扫重算与它逐项比（作废条款 V4）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SegmentCounters {
    pub(super) free_slots: u64,
    /// 段内 32 个偶数起点两槽对里两槽都空闲的对数。
    pub(super) free_pairs: u64,
    /// 未释放记录罩住的槽数（greedy 的打分，`compaction_plan.rs` 的 `greedy_score`）。
    pub(super) live_slots: u64,
    pub(super) pinned_live_slots: u64,
    /// 段内未释放的用户数据单元数（按单元的起点槽数）。
    pub(super) live_user_units: u64,
    pub(super) released_slots: u64,
}

/// 一个段此刻属于哪几个集合。由计数与「这次挂载开过没有」唯一推出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SegmentMembership {
    /// 用户数据的候选：有空闲两槽对、不在这次挂载开过的段里（D3（空间分配） 已定项 8 第 1、2 条，已定项 10 ②）。
    pub(super) is_user_placement_candidate: bool,
    /// 占用口径全空：64 槽都空闲（`allocator.rs` 的 `empty_segments`）。
    pub(super) is_fully_free: bool,
    /// 至少一个空闲槽：提交内生块回落时从这些段里取槽号最小的。
    pub(super) has_a_free_slot: bool,
    /// 记录口径全空：段内没有未释放记录（D3（空间分配） 已定项 10 ①，C372（全空聚簇段数两份定义相反） 收口的那一个）。
    pub(super) is_record_empty: bool,
    /// 整理的目的地（登记 M-7）：不是全空段、段里至少一个未释放的用户数据单元、不在这次挂载开过的段里、有空闲两槽对。
    pub(super) is_compaction_destination: bool,
}

/// runs 数的「空闲」：已释放而还没回收的槽不算空闲（登记读法第 6 条，按 `allocator.rs` 回收时才把槽记回空闲的取法）。
pub(super) fn is_free_for_runs(state: SlotState) -> bool {
    state.is_free()
}

pub(super) fn membership_of(counters: &SegmentCounters, opened_this_mount: bool) -> SegmentMembership {
    SegmentMembership {
        is_user_placement_candidate: counters.free_pairs > 0 && !opened_this_mount,
        is_fully_free: counters.free_slots == CLUSTER_SEGMENT_SLOTS,
        has_a_free_slot: counters.free_slots > 0,
        is_record_empty: counters.live_slots == 0,
        is_compaction_destination: counters.free_pairs > 0 && !opened_this_mount && counters.live_user_units > 0,
    }
}

/// 一次发布是哪一类（登记 M-5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PublishKind {
    ForegroundWrite,
    /// 删除类：头一侧删文件或删单元，以及销毁快照的各批。
    Deletion,
    CompactionBatch,
    /// 抬 F 用的空发布。
    Empty,
    SnapshotCreation,
}

impl PublishKind {
    /// 改用户可见状态的发布才算非空根（登记 M-6 ①）。
    pub(super) fn is_non_empty_root(self) -> bool {
        match self {
            PublishKind::ForegroundWrite | PublishKind::Deletion | PublishKind::SnapshotCreation => true,
            PublishKind::CompactionBatch | PublishKind::Empty => false,
        }
    }

    /// 合取 1 判前台写入类、整理批与建快照；删除类与空发布不判（登记 M-5）。
    pub(super) fn is_judged_by_conjunct_one(self) -> bool {
        match self {
            PublishKind::ForegroundWrite | PublishKind::CompactionBatch | PublishKind::SnapshotCreation => true,
            PublishKind::Deletion | PublishKind::Empty => false,
        }
    }

    /// 合取 7 只判前台写入类（D3（空间分配） 已定项 7 第 7 行）。
    pub(super) fn is_judged_by_conjunct_seven(self) -> bool {
        match self {
            PublishKind::ForegroundWrite => true,
            PublishKind::Deletion => false,
            PublishKind::CompactionBatch | PublishKind::Empty | PublishKind::SnapshotCreation => false,
        }
    }
}

/// 一次发布被拒的原因。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PublishRefusal {
    /// 合取 1：预演之后空闲可回收槽（排除 R）不到常驻底，或用户数据找不到落点。
    ConjunctOne,
    /// 合取 7：全空聚簇段数（记录口径）< `N₀`。
    ConjunctSeven,
    /// 提交内生块一个槽都找不到（删除类与空发布记「删除发不出去」，登记 M-5）。
    NoSlotForCommitGeneratedBlock,
}

/// 一个用户数据单元（两槽）。身份不随搬迁变（岔路单第 1 行取乙：搬迁只改映射条目）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UnitRecord {
    /// 第一槽在单元区里的偏移（偶数）。
    pub(super) first_slot_offset: u64,
    pub(super) birth_txg: u64,
    pub(super) is_alive_in_head: bool,
    /// 钉住它的快照数。头一侧删掉时有快照钉着就不释放，最后一个快照销毁时才释放（D3（空间分配） 已定项 7「释放」的口径）。
    pub(super) snapshot_pins: u64,
    /// 两槽的记录已释放。
    pub(super) is_released: bool,
}

impl UnitRecord {
    /// 只被快照引用、头一侧已删的单元：算进起跑水位的不可回收量（登记 M-7）。
    pub(super) fn is_referenced_only_by_snapshots(&self) -> bool {
        !self.is_alive_in_head && self.snapshot_pins > 0 && !self.is_released
    }

    pub(super) fn can_be_recycled(&self) -> bool {
        !self.is_alive_in_head && self.snapshot_pins == 0 && self.is_released
    }
}

/// 开放聚簇段与它的 bump 游标（只住内存，重挂作废，D3（空间分配） 已定项 10 ①）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct OpenSegmentCursor {
    pub(super) segment: u64,
    pub(super) next_offset_in_segment: u64,
}

/// 事务里改过的一样东西，回滚时照原样放回（预演被拒的发布一个状态都不留）。
#[derive(Clone, Copy, Debug)]
pub(super) enum UndoEntry {
    SlotState { offset: u64, previous: SlotState },
    ReclaimQueuePushedBack,
    OpenCursor { previous: Option<OpenSegmentCursor> },
    SegmentOpenedThisMount { segment: u64 },
    BlockPosition { block: BlockIdentity, previous: Option<u64> },
    UnitRecordChanged { unit: u32, previous: UnitRecord },
    UnitAppendedToArena,
    /// 回滚时把空闲名单里那条旧记录（可回收的那一版）放回去，再把下标还给空闲名单。
    UnitTakenFromFreeList { unit: u32, previous: UnitRecord },
    FallbackBlockCounted,
}

/// 一条活的整理意图（同时至多一条，D26（后台整理与放置回收） 已定项 1 第 3 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CompactionIntent {
    pub(super) region_segment: u64,
    /// `c0`：签发那一刻的分配代（签发那次发布之前最新的 txg）。
    pub(super) allocation_txg_at_issue: u64,
    /// `segs0`：签发那次发布之后的全空段数（记录口径）。
    pub(super) record_empty_segments_at_issue: u64,
    pub(super) openable_segments_at_issue: u64,
}

/// 一条结束了的意图（目标态达成而删意图，或因 R 里只剩钉住的而丢掉）上记的四个量（登记读法第 7 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EndedIntent {
    pub(super) round: u64,
    pub(super) is_completed: bool,
    /// `P_R`：删意图那一刻 R 按记录全空。
    pub(super) region_is_record_empty: bool,
    /// `P_G`：全空段数（记录口径）− segs0 ≥ 1。
    pub(super) global_difference_is_positive: bool,
    /// `Y1`：R 真空出来了（按可开聚簇段口径），晚 3S − 1 次发布判；跑完还没到判的时刻是 None。
    pub(super) region_became_openable: Option<bool>,
    /// `Y2`：同一时刻可开聚簇段数 − 签发那一刻 ≥ 1。
    pub(super) openable_count_rose: Option<bool>,
}

/// 等着判 `Y1`、`Y2` 的意图。
#[derive(Clone, Copy, Debug)]
pub(super) struct PendingTruthJudgement {
    pub(super) ended_intent_index: usize,
    pub(super) region_segment: u64,
    pub(super) end_txg: u64,
    pub(super) last_release_txg_in_region: u64,
    pub(super) openable_segments_at_issue: u64,
}

/// 停机臂（登记第五节第 2 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StopArm {
    /// 停乙：签发前判空洞够不够；一条意图完成而 `P_G` 假就不签下一条。
    LocalRegionPredicate,
    /// 停甲（维持全局差值）：不判空洞；`P_G` 真就返回，假就接着签，到 `B_gc` 用完为止。
    #[cfg_attr(not(test), expect(dead_code, reason = "第一段的正式跑只跑停乙；停甲在第二段跑，这一段只在单测的阳性对照 P2、P3 里构造"))]
    GlobalDifference,
}

/// 一条正式跑里池这一侧的旋钮。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PoolSettings {
    /// 合取 7 的 `N₀`。
    pub(super) floor_segments: u64,
    pub(super) is_compaction_enabled: bool,
    pub(super) stop_arm: StopArm,
    /// 常驻底 = 这个倍数 × S × c 槽（主取样点 3，反向点 6）。
    pub(super) resident_base_multiplier: u64,
}

/// 一轮里攒的计数（登记第六节第 2–6 行的逐轮值）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RoundMetrics {
    pub(super) lowest_record_empty_segments: u64,
    pub(super) lowest_fully_free_segments: u64,
    pub(super) lowest_openable_segments: u64,
    pub(super) deletion_publishes: u64,
    pub(super) deletion_commit_generated_block_sum: u64,
    pub(super) deletion_commit_generated_block_peak: u64,
    pub(super) fallback_blocks: u64,
    /// 删除类发布要的槽找不到一个（登记 M-5 那一条）。
    pub(super) deletion_publish_failures: u64,
    /// 抬 F 的空发布要的槽找不到一个（同一条，另列一栏）。
    pub(super) empty_publish_failures: u64,
    pub(super) refused_writes_by_conjunct_one: u64,
    pub(super) refused_writes_by_conjunct_seven: u64,
    /// 最近一次被拒写入那一刻的空闲可回收槽数（排除 R）。
    pub(super) free_slots_at_last_refusal: Option<u64>,
    pub(super) foreground_writes: u64,
    pub(super) compaction_batches: u64,
    pub(super) units_moved: u64,
    pub(super) empty_publishes: u64,
    pub(super) intents_completed: u64,
    pub(super) intents_discarded: u64,
}

impl RoundMetrics {
    pub(super) fn starting_at(record_empty: u64, fully_free: u64, openable: u64) -> Self {
        Self {
            lowest_record_empty_segments: record_empty,
            lowest_fully_free_segments: fully_free,
            lowest_openable_segments: openable,
            deletion_publishes: 0,
            deletion_commit_generated_block_sum: 0,
            deletion_commit_generated_block_peak: 0,
            fallback_blocks: 0,
            deletion_publish_failures: 0,
            empty_publish_failures: 0,
            refused_writes_by_conjunct_one: 0,
            refused_writes_by_conjunct_seven: 0,
            free_slots_at_last_refusal: None,
            foreground_writes: 0,
            compaction_batches: 0,
            units_moved: 0,
            empty_publishes: 0,
            intents_completed: 0,
            intents_discarded: 0,
        }
    }
}

/// 计数模型 M 的池（登记第五节 M-1–M-8）：一张槽表代两块同槽的盘。
pub(super) struct Pool {
    pub(super) geometry: PoolGeometry,
    pub(super) settings: PoolSettings,
    /// 按单元区偏移（绝对槽号 − 50176）排。
    pub(super) slot_states: Vec<SlotState>,
    pub(super) segments: Vec<SegmentCounters>,
    pub(super) is_opened_this_mount: Vec<bool>,
    pub(super) segments_opened_this_mount: Vec<u64>,
    pub(super) user_placement_segments: BTreeSet<u64>,
    pub(super) fully_free_segments: BTreeSet<u64>,
    pub(super) segments_with_a_free_slot: BTreeSet<u64>,
    pub(super) record_empty_segment_count: u64,
    pub(super) compaction_destination_free_pairs: u64,
    pub(super) free_slot_total: u64,
    pub(super) released_slot_total: u64,
    pub(super) live_user_unit_total: u64,
    pub(super) snapshot_only_unit_total: u64,
    pub(super) free_runs: u64,
    /// (释放代, 单元区偏移)，按释放代不减排（释放只发生在当前 txg）。
    pub(super) reclaim_queue: VecDeque<(u64, u64)>,
    pub(super) txg: u64,
    pub(super) effective_floor_txg: u64,
    /// 抬 F 进行中：(目标, 已发的空发布数)。
    pub(super) pending_floor_raise: Option<(u64, u64)>,
    /// 环里非空根的 txg，从旧到新。
    pub(super) non_empty_root_txgs_in_ring: VecDeque<u64>,
    pub(super) last_non_empty_root_txg: u64,
    pub(super) open_cursor: Option<OpenSegmentCursor>,
    pub(super) file_tree_node_slots: Vec<Option<u64>>,
    pub(super) leaf_slots: Vec<Option<u64>>,
    pub(super) level_one_slots: Vec<Option<u64>>,
    pub(super) root_slot: Option<u64>,
    pub(super) units: Vec<UnitRecord>,
    pub(super) free_unit_indices: Vec<u32>,
    pub(super) units_to_recycle_after_commit: Vec<u32>,
    pub(super) active_intent: Option<CompactionIntent>,
    pub(super) ended_intents: Vec<EndedIntent>,
    pub(super) pending_truth_judgements: Vec<PendingTruthJudgement>,
    pub(super) undo_log: Vec<UndoEntry>,
    pub(super) is_transaction_open: bool,
    /// mkfs 那一版正在写（它的 txg 是 0）。
    pub(super) is_making_filesystem: bool,
    /// 这次发布里记录被改写的槽（分配或释放），用来推脏叶（登记 M-3 的 X）。
    pub(super) dirty_slot_offsets: Vec<u64>,
    pub(super) round_metrics: RoundMetrics,
    pub(super) current_round: u64,
}

pub(super) fn usize_of(value: u64) -> usize {
    usize::try_from(value).expect("模型里的下标装得进 usize")
}
