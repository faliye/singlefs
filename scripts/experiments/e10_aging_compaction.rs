//! E10（老化：结构抗老化 vs 事后整理） 第 1 次跑的第一段：岔路单 `research/prompts/m4-gc-r1-forks.md` 第 4 行（`N₀` 的值）的计数模型那一半，
//! 第 2 行（停机谓词）逐条意图的四个量顺带出。跑前登记 `research/prompts/e10-r1-prereg.md`（下称「登记」），判据、臂、取样点都在那里写死。
//!
// admission: inputs-changed research/prompts/e10-r1-prereg.md ./lib.rs arguments
// run-condition: command python3
//!
//! 准入的输入：本文件（判法自带）、跑前登记、产物行的格式器 `lib.rs`，加上这一次的参数（分片之间参数不同，各判各的）。
//!
//! ## 它答什么
//!
//! 「填满 → 删 → 填满」（FDF）循环里，合取 7 的门槛 `N₀` 取 1、2、`⌈(3S − 1) × c ÷ 64⌉ + 1` 三个候选，在 S ∈ {4, 8, 16} × c ∈ {5, 10, 13}
//! 九格、主取样点与六个方向相反的取样点上：删除类发布的提交内生块数 `c_actual` 随轮数有没有发散（登记第六节第 3 行）、删除类发布
//! 发不出去的次数（第 4 行）；附带全空段数三个口径的轨迹（第 2 行）、回落计数（第 5 行）、被拒写入（第 6 行）、纯元数据稳态流里
//! 不可开段数的峰值（第 7 行），以及每条结束的整理意图上的 `P_R`、`P_G`、`Y1`、`Y2`（第 9、10、13 行的输入）。
//!
//! ## 臂（登记第五节，逐字同义）
//!
//! - N1 / N2 / NF：合取 7 取 `N₀ = 1` / `2` / 那一格的算式值，只判前台写入类发布；整理开、选段 greedy、停机照停乙。
//! - `N₀ = 0`（只报）与上面四档各自「整理关掉」（OFF，阳性对照）。
//!
//! ## 它不答什么（跑前写死）
//!
//! 不答实现臂（`crates/` 那份代码在循环里的性质，第三段）；不答选段三臂与停不下的起点（第二段）；模型里根槽写不失败、不崩溃
//! （登记第十一节已登记差异 ⑥）；文件树节点数恒为 `c − b`（差异 ③）；合取 1 是九项式子的简化（差异 ④）。
//!
//! ## 模型是确定性的
//!
//! 随机源只有种子，同一种子跑两遍逐字节相同；证据强度来自单测里钉死的绝对值与变异，不来自轮数（`test-discipline.md`「单次观测不算数」）。

use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use e7_index_bench::Emitter;

// ---- 本地常量：值抄自 `crates/`，与 `crates/` 逐项对拍过（登记第十一节 S1、S2；对拍的命令与原样输出在执行员报告里） ----

/// 聚簇段段长（槽）。`crates/singlefs-format/src/lib.rs` 的 `CLUSTER_SEGMENT_SLOTS`。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// 单元区起始槽号（默认 768 MiB 环）。同文件 `UNIT_AREA_START_SLOT`。
const UNIT_AREA_START_SLOT: u64 = 50176;
/// 根环的区数。同文件 `ROOT_RING_REGIONS`。
const ROOT_RING_REGIONS: u64 = 3;
/// 每区根槽数 S 的下界与上界。同文件 `ROOT_RING_SLOTS_PER_REGION_MINIMUM` / `_MAXIMUM`。
const ROOT_RING_SLOTS_PER_REGION_LOWER_BOUND: u64 = 4;
const ROOT_RING_SLOTS_PER_REGION_UPPER_BOUND: u64 = 16;
/// 分配记录树一片叶罩的槽数。同文件 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// 分配记录树内部节点的扇出。同文件 `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`。
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// 两块盘同槽（D2（RAID 条带策略） 已定项 10）：分配记录树的叶与层级 1 节点每块盘一份。
const DISK_COUNT: u64 = 2;

// ---- 登记写死的负载与判据常量（登记第五、六节） ----

/// 一次删除类发布至多删几个单元（登记 M-9 的 q）。
const UNITS_PER_DELETION_PUBLISH: usize = 64;
/// 一个新文件的单元数在 1..=32 里按种子均匀取（登记 M-9）。
const FILE_UNITS_UPPER_BOUND: u64 = 32;
/// FDF 跑的轮数；判「有界」的再续到加倍复核（登记 M-9 与第六节第 3 行）。
const ROUNDS_FIRST_PASS: u64 = 60;
const ROUNDS_DOUBLED_CHECK: u64 = 120;
/// 每格每臂的种子数（登记读法第 11 条）。
const SEEDS_PER_CELL: u64 = 5;
/// 一批至多搬几个单元（主取样点 K = 1，登记 M-7）。
const COMPACTION_BATCH_UNITS: usize = 1;
/// 纯元数据稳态流的空发布次数（登记第六节第 7 行）。
const METADATA_STREAM_EMPTY_PUBLISHES: u64 = 1000;
/// 抬 F 要连发的空发布次数（登记 M-6 ①，D16（发布语义） 已定项 1「抬 F 那一串」）。
const EMPTY_PUBLISHES_FOR_FLOOR_TO_TAKE_EFFECT: u64 = 2;
/// 抬 F 抬到第几新的非空根（D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」）。
const RETAINED_NON_EMPTY_STATES: usize = 4;
/// 第六节第 9 行：`P_R` 判真而 `Y1` 假的占比到这个百分数就记。
const PREDICATE_FALSE_POSITIVE_SHARE_PERCENT: u64 = 5;

/// 九格的 S 与 c（登记第一节读法第 2、4 条）。
const ROOT_RING_SLOTS_PER_REGION_CHOICES: [u64; 3] = [4, 8, 16];
const COMMIT_GENERATED_BLOCKS_CHOICES: [u64; 3] = [5, 10, 13];

/// 4 GiB 与 1 GiB 盘的槽数（16 KiB 一槽）。
const FOUR_GIBIBYTE_DISK_SLOTS: u64 = 262_144;
const ONE_GIBIBYTE_DISK_SLOTS: u64 = 65_536;
/// 反向点的叶宽（登记第八节「叶宽（只在模型）」）。
const REVERSE_POINT_LEAF_SLOTS: u64 = 203;

/// 一个池的几何与这一格的参数：S、c、叶宽、盘大小。由它推出段数、树高、b 与文件树节点数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PoolGeometry {
    disk_slots: u64,
    leaf_width_in_slots: u64,
    root_ring_slots_per_region: u64,
    commit_generated_blocks_per_publish: u64,
}

impl PoolGeometry {
    fn new(
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

    fn unit_area_slots(&self) -> u64 {
        self.disk_slots - UNIT_AREA_START_SLOT
    }

    fn segment_count(&self) -> u64 {
        self.unit_area_slots() / CLUSTER_SEGMENT_SLOTS
    }

    /// 层级 1 节点罩的槽数：`W × 169`（`crates/singlefs-core/src/allocation_record_tree.rs` 模块文档的 S_1）。
    fn level_one_width_in_slots(&self) -> u64 {
        self.leaf_width_in_slots * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
    }

    /// 根的层级：最小的 R ≥ 1 使 Σ_盘 ⌈盘槽数 ÷ S_{R−1}⌉ ≤ 169（`allocation_record_tree.rs` 模块文档「根」那一条）。
    fn record_tree_root_level(&self) -> u64 {
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

    fn has_level_one_nodes_below_the_root(&self) -> bool {
        self.record_tree_height() == 3
    }

    /// b：脏槽只落在一片叶、一个层级 1 节点时分配记录树写出的块数，`2 × (树高 − 1) + 1`（登记 M-3）。
    fn record_tree_blocks_per_publish_at_one_leaf(&self) -> u64 {
        DISK_COUNT * (self.record_tree_height() - 1) + 1
    }

    /// 文件树与中央映射树的节点数：恒为 `c − b`（登记 M-3 的登记取法，已登记差异 ③）。
    fn file_tree_nodes_per_publish(&self) -> u64 {
        self.commit_generated_blocks_per_publish - self.record_tree_blocks_per_publish_at_one_leaf()
    }

    fn leaf_of_absolute_slot(&self, absolute_slot: u64) -> u64 {
        absolute_slot / self.leaf_width_in_slots
    }

    fn level_one_node_of_absolute_slot(&self, absolute_slot: u64) -> u64 {
        absolute_slot / self.level_one_width_in_slots()
    }

    /// 单元区里有几片叶（第一片与最后一片之间，含两端）。
    fn leaves_covering_the_unit_area(&self) -> u64 {
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
    fn structural_commit_generated_block_bound(&self) -> u64 {
        self.file_tree_nodes_per_publish()
            + DISK_COUNT * self.leaves_covering_the_unit_area()
            + DISK_COUNT * self.level_one_nodes_covering_the_unit_area()
            + 1
    }

    /// 保持期：一个释放代 t 的槽在发布 txg ≥ t + 3S − 1 时才可回收（登记 M-2：环满之后最旧有效根 = 现在的 txg − 3S + 1）。
    fn retention_publishes(&self) -> u64 {
        ROOT_RING_REGIONS * self.root_ring_slots_per_region - 1
    }
}

/// 候选 NF 的算式值 `⌈(3S − 1) × c ÷ 64⌉ + 1`（D3（空间分配） 已定项 7 登记表第 7 行）。
fn formula_floor_segments(root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64) -> u64 {
    ((ROOT_RING_REGIONS * root_ring_slots_per_region - 1) * commit_generated_blocks_per_publish)
        .div_ceil(CLUSTER_SEGMENT_SLOTS)
        + 1
}

/// 整理入口甲一次调用的发布预算 `B_gc = max(1, 3S − 4 − e)`，e = 离最近一条非空根已经过去的发布数（判决 G8，登记 M-7）。
fn compaction_call_publish_budget(root_ring_slots_per_region: u64, publishes_since_last_non_empty_root: u64) -> u64 {
    (ROOT_RING_REGIONS * root_ring_slots_per_region)
        .saturating_sub(4)
        .saturating_sub(publishes_since_last_non_empty_root)
        .max(1)
}

/// 一个已释放的槽在某次发布时可不可回收：释放代 ≤ max(F_生效, 环里最旧有效根)（D16（发布语义） 已定项 1；登记 M-2）。
fn reclaim_threshold_at_publish(publish_txg: u64, effective_floor_txg: u64, geometry: &PoolGeometry) -> u64 {
    let oldest_valid_root_txg = publish_txg.saturating_sub(geometry.retention_publishes());
    effective_floor_txg.max(oldest_valid_root_txg)
}

/// 抬 F 的目标：环里第 4 新的非空根的 txg；不足 4 个取环里最旧有效根（D16（发布语义） 已定项 1「抬 F 的上限」）。
/// `non_empty_root_txgs_in_ring` 从旧到新排。
fn floor_raise_target(non_empty_root_txgs_in_ring: &VecDeque<u64>, oldest_valid_root_txg: u64) -> u64 {
    if non_empty_root_txgs_in_ring.len() < RETAINED_NON_EMPTY_STATES {
        return oldest_valid_root_txg;
    }
    non_empty_root_txgs_in_ring[non_empty_root_txgs_in_ring.len() - RETAINED_NON_EMPTY_STATES]
}

/// 种子化的随机源（SplitMix64）。模型里唯一的随机源。
struct SeededRandom {
    state: u64,
}

impl SeededRandom {
    fn new(seed: u64) -> Self {
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
    fn below(&mut self, bound: u64) -> u64 {
        assert!(bound >= 1, "空区间里取不出数");
        self.next_word() % bound
    }
}

/// 一块提交内生块是谁：文件树（含中央映射树）节点、分配记录树的叶、层级 1 节点（每块盘一份，按位置）、根（登记 M-3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BlockIdentity {
    FileTreeNode { node: u64 },
    RecordTreeLeaf { disk: u64, leaf: u64 },
    RecordTreeLevelOneNode { disk: u64, node: u64 },
    RecordTreeRoot,
}

/// 一个已分配槽上那条记录罩住的是什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotOwner {
    /// mkfs 的实例表两槽：建池时钉住、整段跑下来不释放不搬（登记 M-2 的登记取法）。
    InstanceTable,
    /// 用户数据单元，两槽；`unit` 是单元的身份（中央映射 key 的模型）。
    UserDataUnit { unit: u32 },
    CommitGeneratedBlock { block: BlockIdentity },
    /// 被快照钉住的那一版文件树节点：被快照根直接指着，区域扫描记「钉住」（登记 M-8）。
    SnapshotPinnedFileTreeNode,
}

impl SlotOwner {
    fn is_pinned(self) -> bool {
        match self {
            SlotOwner::InstanceTable | SlotOwner::SnapshotPinnedFileTreeNode => true,
            SlotOwner::UserDataUnit { .. } | SlotOwner::CommitGeneratedBlock { .. } => false,
        }
    }

    fn is_user_data(self) -> bool {
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
enum SlotState {
    Free,
    Live { owner: SlotOwner, allocation_txg: u64 },
    Released { release_txg: u64 },
}

impl SlotState {
    fn is_free(self) -> bool {
        self == SlotState::Free
    }

    fn is_live(self) -> bool {
        match self {
            SlotState::Live { .. } => true,
            SlotState::Free | SlotState::Released { .. } => false,
        }
    }

    fn is_released(self) -> bool {
        match self {
            SlotState::Released { .. } => true,
            SlotState::Free | SlotState::Live { .. } => false,
        }
    }

    fn live_owner(self) -> Option<SlotOwner> {
        match self {
            SlotState::Live { owner, .. } => Some(owner),
            SlotState::Free | SlotState::Released { .. } => None,
        }
    }
}

/// 一个聚簇段的增量计数。全扫重算与它逐项比（作废条款 V4）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SegmentCounters {
    free_slots: u64,
    /// 段内 32 个偶数起点两槽对里两槽都空闲的对数。
    free_pairs: u64,
    /// 未释放记录罩住的槽数（greedy 的打分，`compaction_plan.rs` 的 `greedy_score`）。
    live_slots: u64,
    pinned_live_slots: u64,
    /// 段内未释放的用户数据单元数（按单元的起点槽数）。
    live_user_units: u64,
    released_slots: u64,
}

/// 一个段此刻属于哪几个集合。由计数与「这次挂载开过没有」唯一推出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SegmentMembership {
    /// 用户数据的候选：有空闲两槽对、不在这次挂载开过的段里（D3（空间分配） 已定项 8 第 1、2 条，已定项 10 ②）。
    is_user_placement_candidate: bool,
    /// 占用口径全空：64 槽都空闲（`allocator.rs` 的 `empty_segments`）。
    is_fully_free: bool,
    /// 至少一个空闲槽：提交内生块回落时从这些段里取槽号最小的。
    has_a_free_slot: bool,
    /// 记录口径全空：段内没有未释放记录（D3（空间分配） 已定项 10 ①，C372（全空聚簇段数两份定义相反） 收口的那一个）。
    is_record_empty: bool,
    /// 整理的目的地（登记 M-7）：不是全空段、段里至少一个未释放的用户数据单元、不在这次挂载开过的段里、有空闲两槽对。
    is_compaction_destination: bool,
}

/// runs 数的「空闲」：已释放而还没回收的槽不算空闲（登记读法第 6 条，按 `allocator.rs` 回收时才把槽记回空闲的取法）。
fn is_free_for_runs(state: SlotState) -> bool {
    state.is_free()
}

fn membership_of(counters: &SegmentCounters, opened_this_mount: bool) -> SegmentMembership {
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
enum PublishKind {
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
    fn is_non_empty_root(self) -> bool {
        match self {
            PublishKind::ForegroundWrite | PublishKind::Deletion | PublishKind::SnapshotCreation => true,
            PublishKind::CompactionBatch | PublishKind::Empty => false,
        }
    }

    /// 合取 1 判前台写入类、整理批与建快照；删除类与空发布不判（登记 M-5）。
    fn is_judged_by_conjunct_one(self) -> bool {
        match self {
            PublishKind::ForegroundWrite | PublishKind::CompactionBatch | PublishKind::SnapshotCreation => true,
            PublishKind::Deletion | PublishKind::Empty => false,
        }
    }

    /// 合取 7 只判前台写入类（D3（空间分配） 已定项 7 第 7 行，用户 2026-10-13 定）。
    fn is_judged_by_conjunct_seven(self) -> bool {
        match self {
            PublishKind::ForegroundWrite => true,
            PublishKind::Deletion => false,
            PublishKind::CompactionBatch | PublishKind::Empty | PublishKind::SnapshotCreation => false,
        }
    }
}

/// 一次发布被拒的原因。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishRefusal {
    /// 合取 1：预演之后空闲可回收槽（排除 R）不到常驻底，或用户数据找不到落点。
    ConjunctOne,
    /// 合取 7：全空聚簇段数（记录口径）< `N₀`。
    ConjunctSeven,
    /// 提交内生块一个槽都找不到（删除类与空发布记「删除发不出去」，登记 M-5）。
    NoSlotForCommitGeneratedBlock,
}

/// 一个用户数据单元（两槽）。身份不随搬迁变（岔路单第 1 行取乙：搬迁只改映射条目）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnitRecord {
    /// 第一槽在单元区里的偏移（偶数）。
    first_slot_offset: u64,
    birth_txg: u64,
    is_alive_in_head: bool,
    /// 钉住它的快照数。头一侧删掉时有快照钉着就不释放，最后一个快照销毁时才释放（D3（空间分配） 已定项 7「释放」的口径）。
    snapshot_pins: u64,
    /// 两槽的记录已释放。
    is_released: bool,
}

impl UnitRecord {
    /// 只被快照引用、头一侧已删的单元：算进起跑水位的不可回收量（登记 M-7）。
    fn is_referenced_only_by_snapshots(&self) -> bool {
        !self.is_alive_in_head && self.snapshot_pins > 0 && !self.is_released
    }

    fn can_be_recycled(&self) -> bool {
        !self.is_alive_in_head && self.snapshot_pins == 0 && self.is_released
    }
}

/// 开放聚簇段与它的 bump 游标（只住内存，重挂作废，D3（空间分配） 已定项 10 ①）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OpenSegmentCursor {
    segment: u64,
    next_offset_in_segment: u64,
}

/// 事务里改过的一样东西，回滚时照原样放回（预演被拒的发布一个状态都不留）。
#[derive(Clone, Copy, Debug)]
enum UndoEntry {
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
struct CompactionIntent {
    region_segment: u64,
    /// `c0`：签发那一刻的分配代（签发那次发布之前最新的 txg）。
    allocation_txg_at_issue: u64,
    /// `segs0`：签发那次发布之后的全空段数（记录口径）。
    record_empty_segments_at_issue: u64,
    openable_segments_at_issue: u64,
}

/// 一条结束了的意图（目标态达成而删意图，或因 R 里只剩钉住的而丢掉）上记的四个量（登记读法第 7 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EndedIntent {
    round: u64,
    is_completed: bool,
    /// `P_R`：删意图那一刻 R 按记录全空。
    region_is_record_empty: bool,
    /// `P_G`：全空段数（记录口径）− segs0 ≥ 1。
    global_difference_is_positive: bool,
    /// `Y1`：R 真空出来了（按可开聚簇段口径），晚 3S − 1 次发布判；跑完还没到判的时刻是 None。
    region_became_openable: Option<bool>,
    /// `Y2`：同一时刻可开聚簇段数 − 签发那一刻 ≥ 1。
    openable_count_rose: Option<bool>,
}

/// 等着判 `Y1`、`Y2` 的意图。
#[derive(Clone, Copy, Debug)]
struct PendingTruthJudgement {
    ended_intent_index: usize,
    region_segment: u64,
    end_txg: u64,
    last_release_txg_in_region: u64,
    openable_segments_at_issue: u64,
}

/// 停机臂（登记第五节第 2 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StopArm {
    /// 停乙（用户已定）：签发前判空洞够不够；一条意图完成而 `P_G` 假就不签下一条。
    LocalRegionPredicate,
    /// 停甲（维持全局差值）：不判空洞；`P_G` 真就返回，假就接着签，到 `B_gc` 用完为止。
    #[cfg_attr(not(test), expect(dead_code, reason = "第一段的正式跑只跑停乙；停甲在第二段跑，这一段只在单测的阳性对照 P2、P3 里构造"))]
    GlobalDifference,
}

/// 一条正式跑里池这一侧的旋钮。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PoolSettings {
    /// 合取 7 的 `N₀`。
    floor_segments: u64,
    is_compaction_enabled: bool,
    stop_arm: StopArm,
    /// 常驻底 = 这个倍数 × S × c 槽（主取样点 3，反向点 6）。
    resident_base_multiplier: u64,
}

/// 一轮里攒的计数（登记第六节第 2–6 行的逐轮值）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RoundMetrics {
    lowest_record_empty_segments: u64,
    lowest_fully_free_segments: u64,
    lowest_openable_segments: u64,
    deletion_publishes: u64,
    deletion_commit_generated_block_sum: u64,
    deletion_commit_generated_block_peak: u64,
    fallback_blocks: u64,
    /// 删除类发布要的槽找不到一个（登记 M-5 那一条）。
    deletion_publish_failures: u64,
    /// 抬 F 的空发布要的槽找不到一个（同一条，另列一栏）。
    empty_publish_failures: u64,
    refused_writes_by_conjunct_one: u64,
    refused_writes_by_conjunct_seven: u64,
    /// 最近一次被拒写入那一刻的空闲可回收槽数（排除 R）。
    free_slots_at_last_refusal: Option<u64>,
    foreground_writes: u64,
    compaction_batches: u64,
    units_moved: u64,
    empty_publishes: u64,
    intents_completed: u64,
    intents_discarded: u64,
}

impl RoundMetrics {
    fn starting_at(record_empty: u64, fully_free: u64, openable: u64) -> Self {
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
struct Pool {
    geometry: PoolGeometry,
    settings: PoolSettings,
    /// 按单元区偏移（绝对槽号 − 50176）排。
    slot_states: Vec<SlotState>,
    segments: Vec<SegmentCounters>,
    is_opened_this_mount: Vec<bool>,
    segments_opened_this_mount: Vec<u64>,
    user_placement_segments: BTreeSet<u64>,
    fully_free_segments: BTreeSet<u64>,
    segments_with_a_free_slot: BTreeSet<u64>,
    record_empty_segment_count: u64,
    compaction_destination_free_pairs: u64,
    free_slot_total: u64,
    released_slot_total: u64,
    live_user_unit_total: u64,
    snapshot_only_unit_total: u64,
    free_runs: u64,
    /// (释放代, 单元区偏移)，按释放代不减排（释放只发生在当前 txg）。
    reclaim_queue: VecDeque<(u64, u64)>,
    txg: u64,
    effective_floor_txg: u64,
    /// 抬 F 进行中：(目标, 已发的空发布数)。
    pending_floor_raise: Option<(u64, u64)>,
    /// 环里非空根的 txg，从旧到新。
    non_empty_root_txgs_in_ring: VecDeque<u64>,
    last_non_empty_root_txg: u64,
    open_cursor: Option<OpenSegmentCursor>,
    file_tree_node_slots: Vec<Option<u64>>,
    leaf_slots: Vec<Option<u64>>,
    level_one_slots: Vec<Option<u64>>,
    root_slot: Option<u64>,
    units: Vec<UnitRecord>,
    free_unit_indices: Vec<u32>,
    units_to_recycle_after_commit: Vec<u32>,
    active_intent: Option<CompactionIntent>,
    ended_intents: Vec<EndedIntent>,
    pending_truth_judgements: Vec<PendingTruthJudgement>,
    undo_log: Vec<UndoEntry>,
    is_transaction_open: bool,
    /// mkfs 那一版正在写（它的 txg 是 0）。
    is_making_filesystem: bool,
    /// 这次发布里记录被改写的槽（分配或释放），用来推脏叶（登记 M-3 的 X）。
    dirty_slot_offsets: Vec<u64>,
    round_metrics: RoundMetrics,
    current_round: u64,
}

fn usize_of(value: u64) -> usize {
    usize::try_from(value).expect("模型里的下标装得进 usize")
}

impl Pool {
    /// 一个全空闲的池：没有 mkfs、没有任何块。单测在它上面直接摆状态。
    fn blank(geometry: PoolGeometry, settings: PoolSettings) -> Self {
        let unit_area_slots = geometry.unit_area_slots();
        let segment_count = geometry.segment_count();
        let all_free_segment = SegmentCounters {
            free_slots: CLUSTER_SEGMENT_SLOTS,
            free_pairs: CLUSTER_SEGMENT_SLOTS / 2,
            live_slots: 0,
            pinned_live_slots: 0,
            live_user_units: 0,
            released_slots: 0,
        };
        let leaf_positions = usize_of(DISK_COUNT * (geometry.leaf_of_absolute_slot(geometry.disk_slots - 1) + 1));
        let level_one_positions =
            usize_of(DISK_COUNT * (geometry.level_one_node_of_absolute_slot(geometry.disk_slots - 1) + 1));
        Self {
            geometry,
            settings,
            slot_states: vec![SlotState::Free; usize_of(unit_area_slots)],
            segments: vec![all_free_segment; usize_of(segment_count)],
            is_opened_this_mount: vec![false; usize_of(segment_count)],
            segments_opened_this_mount: Vec::new(),
            user_placement_segments: (0..segment_count).collect(),
            fully_free_segments: (0..segment_count).collect(),
            segments_with_a_free_slot: (0..segment_count).collect(),
            record_empty_segment_count: segment_count,
            compaction_destination_free_pairs: 0,
            free_slot_total: unit_area_slots,
            released_slot_total: 0,
            live_user_unit_total: 0,
            snapshot_only_unit_total: 0,
            free_runs: 1,
            reclaim_queue: VecDeque::new(),
            txg: 0,
            effective_floor_txg: 0,
            pending_floor_raise: None,
            non_empty_root_txgs_in_ring: VecDeque::new(),
            last_non_empty_root_txg: 0,
            open_cursor: None,
            file_tree_node_slots: vec![None; usize_of(geometry.file_tree_nodes_per_publish())],
            leaf_slots: vec![None; leaf_positions],
            level_one_slots: vec![None; level_one_positions],
            root_slot: None,
            units: Vec::new(),
            free_unit_indices: Vec::new(),
            units_to_recycle_after_commit: Vec::new(),
            active_intent: None,
            ended_intents: Vec::new(),
            pending_truth_judgements: Vec::new(),
            undo_log: Vec::new(),
            is_transaction_open: false,
            is_making_filesystem: false,
            dirty_slot_offsets: Vec::new(),
            round_metrics: RoundMetrics::starting_at(segment_count, segment_count, segment_count),
            current_round: 0,
        }
    }
}

// ---- 槽的状态改动：全部经 set_slot_state，计数、runs、集合与回滚日志在这一处一起改 ----
impl Pool {
    fn segment_of_offset(offset: u64) -> u64 {
        offset / CLUSTER_SEGMENT_SLOTS
    }

    fn slot_state(&self, offset: u64) -> SlotState {
        self.slot_states[usize_of(offset)]
    }

    fn is_pair_free(&self, even_offset: u64) -> bool {
        self.slot_state(even_offset).is_free() && self.slot_state(even_offset + 1).is_free()
    }

    fn membership_of_segment(&self, segment: u64) -> SegmentMembership {
        membership_of(&self.segments[usize_of(segment)], self.is_opened_this_mount[usize_of(segment)])
    }

    fn active_region_segment(&self) -> Option<u64> {
        self.active_intent.map(|intent| intent.region_segment)
    }

    /// 可开聚簇段数：占用口径全空且不是活意图的 R（模型不建隔离与扣住位，登记已登记差异 ⑥）。
    fn openable_segment_count(&self) -> u64 {
        let region_is_fully_free = self
            .active_region_segment()
            .is_some_and(|region| self.membership_of_segment(region).is_fully_free);
        self.fully_free_segments.len() as u64 - u64::from(region_is_fully_free)
    }

    fn refresh_segment_membership(&mut self, segment: u64, before: SegmentMembership, free_pairs_before: u64) {
        let after = self.membership_of_segment(segment);
        if before.is_user_placement_candidate != after.is_user_placement_candidate {
            if after.is_user_placement_candidate {
                self.user_placement_segments.insert(segment);
            } else {
                self.user_placement_segments.remove(&segment);
            }
        }
        if before.is_fully_free != after.is_fully_free {
            if after.is_fully_free {
                self.fully_free_segments.insert(segment);
            } else {
                self.fully_free_segments.remove(&segment);
            }
        }
        if before.has_a_free_slot != after.has_a_free_slot {
            if after.has_a_free_slot {
                self.segments_with_a_free_slot.insert(segment);
            } else {
                self.segments_with_a_free_slot.remove(&segment);
            }
        }
        if before.is_record_empty != after.is_record_empty {
            if after.is_record_empty {
                self.record_empty_segment_count += 1;
            } else {
                self.record_empty_segment_count -= 1;
            }
        }
        if before.is_compaction_destination {
            self.compaction_destination_free_pairs -= free_pairs_before;
        }
        if after.is_compaction_destination {
            self.compaction_destination_free_pairs += self.segments[usize_of(segment)].free_pairs;
        }
    }

    fn add_state_contribution(&mut self, offset: u64, state: SlotState, sign_is_positive: bool) {
        let counters = &mut self.segments[usize_of(Self::segment_of_offset(offset))];
        let step = |count: &mut u64| {
            if sign_is_positive {
                *count += 1;
            } else {
                *count -= 1;
            }
        };
        match state {
            SlotState::Free => {
                step(&mut counters.free_slots);
                step(&mut self.free_slot_total);
            }
            SlotState::Live { owner, .. } => {
                step(&mut counters.live_slots);
                if owner.is_pinned() {
                    step(&mut counters.pinned_live_slots);
                }
                if owner.is_user_data() && offset % 2 == 0 {
                    step(&mut counters.live_user_units);
                    step(&mut self.live_user_unit_total);
                }
            }
            SlotState::Released { .. } => {
                step(&mut counters.released_slots);
                step(&mut self.released_slot_total);
            }
        }
    }

    /// 改一个槽的状态。事务开着时记回滚日志；记录被改写（分配、释放）的槽进脏槽集 X。
    fn set_slot_state(&mut self, offset: u64, new_state: SlotState) {
        let previous = self.slot_state(offset);
        if previous == new_state {
            return;
        }
        let segment = Self::segment_of_offset(offset);
        let membership_before = self.membership_of_segment(segment);
        let free_pairs_before = self.segments[usize_of(segment)].free_pairs;
        let pair_offset = offset - offset % 2;
        let pair_was_free = self.is_pair_free(pair_offset);
        if is_free_for_runs(previous) != is_free_for_runs(new_state) {
            let left_is_free = offset > 0 && is_free_for_runs(self.slot_state(offset - 1));
            let right_is_free =
                offset + 1 < self.geometry.unit_area_slots() && is_free_for_runs(self.slot_state(offset + 1));
            let free_neighbours = u64::from(left_is_free) + u64::from(right_is_free);
            // runs 的增量（`allocator.rs` 的 mark_allocated / mark_reclaimed 同形）：变空闲时两边都空是并段（−1）、两边都占是新开一段（+1）。
            self.free_runs = if is_free_for_runs(new_state) {
                (self.free_runs + 1).checked_sub(free_neighbours).expect("runs 不为负")
            } else {
                (self.free_runs + free_neighbours).checked_sub(1).expect("占掉一个空闲槽之前至少有一段空闲")
            };
        }
        self.add_state_contribution(offset, previous, false);
        self.add_state_contribution(offset, new_state, true);
        self.slot_states[usize_of(offset)] = new_state;
        let pair_is_free = self.is_pair_free(pair_offset);
        if pair_was_free != pair_is_free {
            let counters = &mut self.segments[usize_of(segment)];
            if pair_is_free {
                counters.free_pairs += 1;
            } else {
                counters.free_pairs -= 1;
            }
        }
        self.refresh_segment_membership(segment, membership_before, free_pairs_before);
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::SlotState { offset, previous });
            let record_is_rewritten = (new_state.is_live() && !previous.is_live())
                || (new_state.is_released() && previous.is_live());
            if record_is_rewritten {
                self.dirty_slot_offsets.push(offset);
            }
        }
    }

    fn set_segment_opened_this_mount(&mut self, segment: u64, is_opened: bool) {
        let membership_before = self.membership_of_segment(segment);
        let free_pairs_before = self.segments[usize_of(segment)].free_pairs;
        self.is_opened_this_mount[usize_of(segment)] = is_opened;
        self.refresh_segment_membership(segment, membership_before, free_pairs_before);
    }
}

// ---- 事务：预演与回滚、单元与块的位置 ----
impl Pool {
    /// 这次发布要用的 txg（mkfs 那一版是 txg 0）。
    fn publishing_txg(&self) -> u64 {
        if self.is_making_filesystem {
            0
        } else {
            self.txg + 1
        }
    }

    fn begin_transaction(&mut self) {
        assert!(!self.is_transaction_open, "事务不嵌套");
        self.is_transaction_open = true;
        self.undo_log.clear();
        self.dirty_slot_offsets.clear();
        self.units_to_recycle_after_commit.clear();
    }

    /// 把这次事务改过的每一样照日志倒着放回去。
    fn roll_back_transaction(&mut self) {
        assert!(self.is_transaction_open, "没有开着的事务");
        self.is_transaction_open = false;
        while let Some(entry) = self.undo_log.pop() {
            match entry {
                UndoEntry::SlotState { offset, previous } => self.set_slot_state(offset, previous),
                UndoEntry::ReclaimQueuePushedBack => {
                    self.reclaim_queue.pop_back().expect("事务里推进去的那一条还在队尾");
                }
                UndoEntry::OpenCursor { previous } => self.open_cursor = previous,
                UndoEntry::SegmentOpenedThisMount { segment } => {
                    self.set_segment_opened_this_mount(segment, false);
                    self.segments_opened_this_mount.pop().expect("事务里开的段在名单末尾");
                }
                UndoEntry::BlockPosition { block, previous } => *self.block_position_mut(block) = previous,
                UndoEntry::UnitRecordChanged { unit, previous } => self.restore_unit_record(unit, previous),
                UndoEntry::UnitAppendedToArena => {
                    let record = self.units.pop().expect("事务里追加的单元在末尾");
                    assert!(!record.is_referenced_only_by_snapshots(), "新建的单元不会只被快照引用");
                }
                UndoEntry::UnitTakenFromFreeList { unit, previous } => {
                    assert!(previous.can_be_recycled(), "空闲名单里的记录是可回收的那一版");
                    self.units[usize_of(u64::from(unit))] = previous;
                    self.free_unit_indices.push(unit);
                }
                UndoEntry::FallbackBlockCounted => self.round_metrics.fallback_blocks -= 1,
            }
        }
        self.dirty_slot_offsets.clear();
        self.units_to_recycle_after_commit.clear();
    }

    fn block_position_mut(&mut self, block: BlockIdentity) -> &mut Option<u64> {
        let leaf_positions_per_disk = self.leaf_slots.len() as u64 / DISK_COUNT;
        let level_one_positions_per_disk = self.level_one_slots.len() as u64 / DISK_COUNT;
        match block {
            BlockIdentity::FileTreeNode { node } => &mut self.file_tree_node_slots[usize_of(node)],
            BlockIdentity::RecordTreeLeaf { disk, leaf } => {
                &mut self.leaf_slots[usize_of(disk * leaf_positions_per_disk + leaf)]
            }
            BlockIdentity::RecordTreeLevelOneNode { disk, node } => {
                &mut self.level_one_slots[usize_of(disk * level_one_positions_per_disk + node)]
            }
            BlockIdentity::RecordTreeRoot => &mut self.root_slot,
        }
    }

    fn set_block_position(&mut self, block: BlockIdentity, offset: u64) {
        let slot = self.block_position_mut(block);
        let previous = *slot;
        *slot = Some(offset);
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::BlockPosition { block, previous });
        }
    }

    /// 改一个单元的记录，连带「只被快照引用」的计数；事务开着时记回滚日志。
    fn set_unit_record(&mut self, unit: u32, new_record: UnitRecord) {
        let previous = self.units[usize_of(u64::from(unit))];
        self.restore_unit_record(unit, new_record);
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::UnitRecordChanged { unit, previous });
        }
        if new_record.can_be_recycled() {
            self.units_to_recycle_after_commit.push(unit);
        }
    }

    fn restore_unit_record(&mut self, unit: u32, record: UnitRecord) {
        let slot = &mut self.units[usize_of(u64::from(unit))];
        let was_snapshot_only = slot.is_referenced_only_by_snapshots();
        *slot = record;
        let is_snapshot_only = record.is_referenced_only_by_snapshots();
        if was_snapshot_only && !is_snapshot_only {
            self.snapshot_only_unit_total -= 1;
        } else if !was_snapshot_only && is_snapshot_only {
            self.snapshot_only_unit_total += 1;
        }
    }

    /// 新建一个单元的记录（落点由调用方随后分配）。
    fn create_unit(&mut self, first_slot_offset: u64, birth_txg: u64) -> u32 {
        assert!(self.is_transaction_open, "单元只在发布的事务里新建");
        let record = UnitRecord {
            first_slot_offset,
            birth_txg,
            is_alive_in_head: true,
            snapshot_pins: 0,
            is_released: false,
        };
        if let Some(unit) = self.free_unit_indices.pop() {
            let previous = self.units[usize_of(u64::from(unit))];
            assert!(previous.can_be_recycled(), "空闲名单里只有可回收的单元");
            self.units[usize_of(u64::from(unit))] = record;
            self.undo_log.push(UndoEntry::UnitTakenFromFreeList { unit, previous });
            unit
        } else {
            self.units.push(record);
            self.undo_log.push(UndoEntry::UnitAppendedToArena);
            u32::try_from(self.units.len() - 1).expect("单元数装得进 u32")
        }
    }

    /// 释放一个已分配的槽：记录改成「已释放 + 释放代 = 这次的 txg」，进 defer 队列。
    fn release_slot(&mut self, offset: u64) {
        assert!(self.slot_state(offset).is_live(), "只释放已分配的槽");
        let release_txg = self.publishing_txg();
        self.set_slot_state(offset, SlotState::Released { release_txg });
        self.reclaim_queue.push_back((release_txg, offset));
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::ReclaimQueuePushedBack);
        }
    }

    fn allocate_slot(&mut self, offset: u64, owner: SlotOwner, allocation_txg: u64) {
        assert!(self.slot_state(offset).is_free(), "只分配空闲的槽");
        self.set_slot_state(offset, SlotState::Live { owner, allocation_txg });
    }

    fn commit_transaction(&mut self) {
        assert!(self.is_transaction_open, "没有开着的事务");
        self.is_transaction_open = false;
        self.undo_log.clear();
        self.dirty_slot_offsets.clear();
        let mut recycled = std::mem::take(&mut self.units_to_recycle_after_commit);
        recycled.sort_unstable();
        recycled.dedup();
        for unit in recycled {
            if self.units[usize_of(u64::from(unit))].can_be_recycled() {
                self.free_unit_indices.push(unit);
            }
        }
    }
}

/// 一组脏槽（绝对槽号）弄脏的分配记录树的叶与层级 1 节点，两块盘各一份（登记 M-3）。
fn record_tree_blocks_dirtied_by(geometry: &PoolGeometry, absolute_slots: &[u64]) -> BTreeSet<BlockIdentity> {
    let mut blocks = BTreeSet::new();
    for absolute_slot in absolute_slots.iter().copied() {
        for disk in 0..DISK_COUNT {
            blocks.insert(BlockIdentity::RecordTreeLeaf { disk, leaf: geometry.leaf_of_absolute_slot(absolute_slot) });
            if geometry.has_level_one_nodes_below_the_root() {
                blocks.insert(BlockIdentity::RecordTreeLevelOneNode {
                    disk,
                    node: geometry.level_one_node_of_absolute_slot(absolute_slot),
                });
            }
        }
    }
    blocks
}

/// 只按一组脏槽算一次发布写出的提交内生块数、不做不动点那一步：`(c − b) + 2 × |叶(X)| + 2 × |层级 1(X)| + 1`（登记 M-3，第七节乙 2）。
/// 正式跑走的是 `rewrite_commit_generated_blocks` 的不动点，它与这里共用 `record_tree_blocks_dirtied_by`；这个函数只给单测钉乙 2。
#[cfg_attr(not(test), expect(dead_code, reason = "只在单测里钉乙 2 的绝对值；正式跑经不动点调同一个 record_tree_blocks_dirtied_by"))]
fn commit_generated_block_count_for_dirty_slots(geometry: &PoolGeometry, absolute_slots: &[u64]) -> u64 {
    geometry.file_tree_nodes_per_publish() + record_tree_blocks_dirtied_by(geometry, absolute_slots).len() as u64 + 1
}

// ---- 落点：用户数据、整理目的地、提交内生块的 bump 与回落 ----
impl Pool {
    fn set_open_cursor(&mut self, cursor: Option<OpenSegmentCursor>) {
        let previous = self.open_cursor;
        self.open_cursor = cursor;
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::OpenCursor { previous });
        }
    }

    fn open_segment(&mut self, segment: u64) {
        if !self.is_opened_this_mount[usize_of(segment)] {
            self.set_segment_opened_this_mount(segment, true);
            self.segments_opened_this_mount.push(segment);
            if self.is_transaction_open {
                self.undo_log.push(UndoEntry::SegmentOpenedThisMount { segment });
            }
        }
        self.set_open_cursor(Some(OpenSegmentCursor { segment, next_offset_in_segment: 0 }));
    }

    /// 用户数据落点：起点最小的偶数槽、两槽都空闲、不在这次挂载开过的段里、不在活意图的 R 里（登记 M-4）。
    fn lowest_user_data_pair(&self) -> Option<u64> {
        let region = self.active_region_segment();
        self.user_placement_segments
            .iter()
            .copied()
            .filter(|segment| Some(*segment) != region)
            .find_map(|segment| self.lowest_free_pair_in_segment(segment))
    }

    fn lowest_free_pair_in_segment(&self, segment: u64) -> Option<u64> {
        let segment_start = segment * CLUSTER_SEGMENT_SLOTS;
        (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
            .step_by(2)
            .find(|even_offset| self.is_pair_free(*even_offset))
    }

    /// 整理目的地（登记 M-7）：R 外、目的地四条成立的段里最低的空闲两槽。
    fn lowest_compaction_destination_pair(&self) -> Option<u64> {
        let region = self.active_region_segment();
        (0..self.geometry.segment_count())
            .filter(|segment| Some(*segment) != region && self.membership_of_segment(*segment).is_compaction_destination)
            .find_map(|segment| self.lowest_free_pair_in_segment(segment))
    }

    /// R 外合格段的空闲两槽数（停乙签发前判用，登记第五节第 2 行）。
    fn compaction_destination_free_pairs_outside_region(&self, region_segment: u64) -> u64 {
        let region_contribution = if self.membership_of_segment(region_segment).is_compaction_destination {
            self.segments[usize_of(region_segment)].free_pairs
        } else {
            0
        };
        self.compaction_destination_free_pairs - region_contribution
    }

    /// 提交内生块一块一槽：从开放段 bump；满了开可开聚簇段里最低的那一段；一段都没有就回落到槽号最小的空闲槽（排除 R），
    /// 回落计数加一（登记 M-4，D3（空间分配） 已定项 8 第 2 条）。
    fn take_commit_generated_block_slot(&mut self) -> Option<u64> {
        let region = self.active_region_segment();
        loop {
            if let Some(cursor) = self.open_cursor {
                if cursor.next_offset_in_segment < CLUSTER_SEGMENT_SLOTS {
                    let offset = cursor.segment * CLUSTER_SEGMENT_SLOTS + cursor.next_offset_in_segment;
                    self.set_open_cursor(Some(OpenSegmentCursor {
                        segment: cursor.segment,
                        next_offset_in_segment: cursor.next_offset_in_segment + 1,
                    }));
                    if self.slot_state(offset).is_free() {
                        return Some(offset);
                    }
                    continue;
                }
            }
            let lowest_openable = self.fully_free_segments.iter().copied().find(|segment| Some(*segment) != region);
            match lowest_openable {
                Some(segment) => self.open_segment(segment),
                None => return self.fallback_commit_generated_block_slot(region),
            }
        }
    }

    fn fallback_commit_generated_block_slot(&mut self, region: Option<u64>) -> Option<u64> {
        let fallback_offset = self
            .segments_with_a_free_slot
            .iter()
            .copied()
            .filter(|segment| Some(*segment) != region)
            .find_map(|segment| {
                let segment_start = segment * CLUSTER_SEGMENT_SLOTS;
                (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS).find(|offset| self.slot_state(*offset).is_free())
            })?;
        self.round_metrics.fallback_blocks += 1;
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::FallbackBlockCounted);
        }
        Some(fallback_offset)
    }

    /// 重写一块提交内生块：新落点 bump，旧版本在这次发布里释放（被快照钉住的那一版不释放）。
    fn rewrite_commit_generated_block(&mut self, block: BlockIdentity) -> Result<(), PublishRefusal> {
        let new_offset = self.take_commit_generated_block_slot().ok_or(PublishRefusal::NoSlotForCommitGeneratedBlock)?;
        self.allocate_slot(new_offset, SlotOwner::CommitGeneratedBlock { block }, self.publishing_txg());
        if let Some(old_offset) = *self.block_position_mut(block) {
            match self.slot_state(old_offset).live_owner() {
                Some(SlotOwner::SnapshotPinnedFileTreeNode) => {}
                Some(SlotOwner::CommitGeneratedBlock { block: owner_block }) if owner_block == block => {
                    self.release_slot(old_offset);
                }
                other => panic!("块 {block:?} 的旧位置 {old_offset} 上不是它自己：{other:?}"),
            }
        }
        self.set_block_position(block, new_offset);
        Ok(())
    }

    /// 这次发布写出的全部提交内生块：文件树节点与根每次都写，另加点名强制重写的块，再加脏槽弄脏的叶与层级 1 节点；
    /// 写出的块自己的落点与它们换下的旧版本并进脏槽再算，直到不变（登记 M-3 的不动点）。返回 `c_actual`。
    fn rewrite_commit_generated_blocks(&mut self, forced_blocks: &[BlockIdentity]) -> Result<u64, PublishRefusal> {
        let mut rewritten: BTreeSet<BlockIdentity> = BTreeSet::new();
        let mut wave: BTreeSet<BlockIdentity> = (0..self.geometry.file_tree_nodes_per_publish())
            .map(|node| BlockIdentity::FileTreeNode { node })
            .chain(forced_blocks.iter().copied())
            .chain(std::iter::once(BlockIdentity::RecordTreeRoot))
            .collect();
        let mut dirty_slots_already_seen = 0;
        let mut completed_waves = 0u64;
        loop {
            let newly_dirty_absolute: Vec<u64> = self.dirty_slot_offsets[dirty_slots_already_seen..]
                .iter()
                .map(|offset| offset + UNIT_AREA_START_SLOT)
                .collect();
            dirty_slots_already_seen = self.dirty_slot_offsets.len();
            wave.extend(record_tree_blocks_dirtied_by(&self.geometry, &newly_dirty_absolute));
            wave.retain(|block| !rewritten.contains(block));
            if wave.is_empty() {
                break;
            }
            for block in std::mem::take(&mut wave) {
                self.rewrite_commit_generated_block(block)?;
                rewritten.insert(block);
            }
            completed_waves += 1;
        }
        assert!(completed_waves >= 1, "每次发布至少写一波（根恒写）");
        Ok(rewritten.len() as u64)
    }
}

// ---- 发布：回收、准入、根环、F ----
impl Pool {
    /// 合取 7：全空聚簇段数（记录口径）≥ `N₀`（D3（空间分配） 已定项 7 第 7 行）。
    fn conjunct_seven_admits(&self) -> bool {
        self.record_empty_segment_count >= self.settings.floor_segments
    }

    fn free_slots_outside_region(&self) -> u64 {
        let region_free = self
            .active_region_segment()
            .map_or(0, |region| self.segments[usize_of(region)].free_slots);
        self.free_slot_total - region_free
    }

    fn resident_base_slots(&self) -> u64 {
        self.settings.resident_base_multiplier
            * self.geometry.root_ring_slots_per_region
            * self.geometry.commit_generated_blocks_per_publish
    }

    /// 合取 1 的登记取法：这次发布预演之后，空闲可回收槽（排除 R）≥ 常驻底（登记 M-5，已登记差异 ④）。
    /// 可回收的在这次发布开头已经回收成空闲，所以只数空闲。
    fn conjunct_one_admits(&self) -> bool {
        self.free_slots_outside_region() >= self.resident_base_slots()
    }

    fn oldest_valid_root_txg_at(&self, publish_txg: u64) -> u64 {
        publish_txg.saturating_sub(self.geometry.retention_publishes())
    }

    /// 下一次发布开头的回收：释放代 ≤ max(F_生效, 环里最旧有效根) 的槽回到空闲。阈值只增不减，提前到这里做与在那次发布里做
    /// 得到同一个状态（被拒的写入不推 txg，下一次发布的 txg 仍是这一个）。顺带判到了时刻的 `Y1`、`Y2`。
    fn reclaim_for_the_next_publish(&mut self) {
        let publish_txg = self.publishing_txg();
        let threshold = reclaim_threshold_at_publish(publish_txg, self.effective_floor_txg, &self.geometry);
        while let Some(&(release_txg, offset)) = self.reclaim_queue.front() {
            if release_txg > threshold {
                break;
            }
            self.reclaim_queue.pop_front();
            assert_eq!(self.slot_state(offset), SlotState::Released { release_txg }, "defer 队列里的槽还是已释放");
            self.set_slot_state(offset, SlotState::Free);
        }
        self.judge_due_truths(publish_txg);
    }

    /// `Y1`：删意图那次发布之后再过 3S − 1 次发布（其间抬过 F 就提前到 F 越过 R 里最后一个释放代那一次），R 按可开聚簇段口径
    /// 是一个可开的段（登记读法第 7 条）；在那次发布的回收之后、分配之前判。`Y2` 同一时刻判。
    fn judge_due_truths(&mut self, publish_txg: u64) {
        let pending = std::mem::take(&mut self.pending_truth_judgements);
        for judgement in pending {
            let is_due = publish_txg > judgement.end_txg
                && (publish_txg >= judgement.end_txg + self.geometry.retention_publishes()
                    || self.effective_floor_txg >= judgement.last_release_txg_in_region);
            if is_due {
                self.record_truth_judgement(judgement);
            } else {
                self.pending_truth_judgements.push(judgement);
            }
        }
    }

    fn record_truth_judgement(&mut self, judgement: PendingTruthJudgement) {
        let region_is_openable = self.membership_of_segment(judgement.region_segment).is_fully_free
            && self.active_region_segment() != Some(judgement.region_segment);
        let openable_now = self.openable_segment_count();
        let ended = &mut self.ended_intents[judgement.ended_intent_index];
        ended.region_became_openable = Some(region_is_openable);
        ended.openable_count_rose = Some(openable_now > judgement.openable_segments_at_issue);
    }

    /// 发布开头：回收、合取 7（只判前台写入类）、开事务。
    fn begin_publish(&mut self, kind: PublishKind) -> Result<(), PublishRefusal> {
        self.reclaim_for_the_next_publish();
        if kind.is_judged_by_conjunct_seven() && !self.conjunct_seven_admits() {
            return Err(PublishRefusal::ConjunctSeven);
        }
        self.begin_transaction();
        Ok(())
    }

    /// 发布收尾：写提交内生块、合取 1、提交或回滚；提交之后推 txg、记根、记这一轮的计数。返回 `c_actual`。
    fn finish_publish(&mut self, kind: PublishKind, forced_blocks: &[BlockIdentity]) -> Result<u64, PublishRefusal> {
        let commit_generated_blocks = match self.rewrite_commit_generated_blocks(forced_blocks) {
            Ok(blocks) => blocks,
            Err(refusal) => {
                self.roll_back_transaction();
                return Err(refusal);
            }
        };
        if kind.is_judged_by_conjunct_one() && !self.conjunct_one_admits() {
            self.roll_back_transaction();
            return Err(PublishRefusal::ConjunctOne);
        }
        self.commit_transaction();
        self.txg += 1;
        self.record_root(kind);
        if kind == PublishKind::Deletion {
            let metrics = &mut self.round_metrics;
            metrics.deletion_publishes += 1;
            metrics.deletion_commit_generated_block_sum += commit_generated_blocks;
            metrics.deletion_commit_generated_block_peak =
                metrics.deletion_commit_generated_block_peak.max(commit_generated_blocks);
        }
        self.sample_segment_counts();
        Ok(commit_generated_blocks)
    }

    fn sample_segment_counts(&mut self) {
        let fully_free = self.fully_free_segments.len() as u64;
        let openable = self.openable_segment_count();
        let metrics = &mut self.round_metrics;
        metrics.lowest_record_empty_segments = metrics.lowest_record_empty_segments.min(self.record_empty_segment_count);
        metrics.lowest_fully_free_segments = metrics.lowest_fully_free_segments.min(fully_free);
        metrics.lowest_openable_segments = metrics.lowest_openable_segments.min(openable);
    }

    /// 根环：每次发布一条根；非空根进非空名单；空发布推进进行中的抬 F，连发够数之后 F 生效（登记 M-6 ①）。
    fn record_root(&mut self, kind: PublishKind) {
        if kind.is_non_empty_root() {
            self.non_empty_root_txgs_in_ring.push_back(self.txg);
            self.last_non_empty_root_txg = self.txg;
        }
        let oldest_valid_root_txg = self.oldest_valid_root_txg_at(self.txg);
        while self.non_empty_root_txgs_in_ring.front().is_some_and(|txg| *txg < oldest_valid_root_txg) {
            self.non_empty_root_txgs_in_ring.pop_front();
        }
        if kind == PublishKind::Empty {
            if let Some((target, empty_publishes_done)) = self.pending_floor_raise {
                let empty_publishes_done = empty_publishes_done + 1;
                if empty_publishes_done == EMPTY_PUBLISHES_FOR_FLOOR_TO_TAKE_EFFECT {
                    self.effective_floor_txg = self.effective_floor_txg.max(target);
                    self.pending_floor_raise = None;
                } else {
                    self.pending_floor_raise = Some((target, empty_publishes_done));
                }
            }
        }
    }
}

// ---- 各类发布 ----
impl Pool {
    /// 前台写入类：写一个 `unit_count` 个单元的新文件。用户数据找不到落点按合取 1 拒（可用不够需求）。
    fn publish_new_file(&mut self, unit_count: u64) -> Result<Vec<u32>, PublishRefusal> {
        self.begin_publish(PublishKind::ForegroundWrite)?;
        let txg = self.publishing_txg();
        let mut created_units = Vec::new();
        for _unit_of_the_file in 0..unit_count {
            let Some(even_offset) = self.lowest_user_data_pair() else {
                self.roll_back_transaction();
                return Err(PublishRefusal::ConjunctOne);
            };
            let unit = self.create_unit(even_offset, txg);
            self.allocate_slot(even_offset, SlotOwner::UserDataUnit { unit }, txg);
            self.allocate_slot(even_offset + 1, SlotOwner::UserDataUnit { unit }, txg);
            created_units.push(unit);
        }
        self.finish_publish(PublishKind::ForegroundWrite, &[])?;
        self.round_metrics.foreground_writes += 1;
        Ok(created_units)
    }

    fn release_unit_slots(&mut self, unit: u32) {
        let first_slot_offset = self.units[usize_of(u64::from(unit))].first_slot_offset;
        self.release_slot(first_slot_offset);
        self.release_slot(first_slot_offset + 1);
    }

    /// 删除类：头一侧删掉这几个单元；没有快照钉着的当场释放，钉着的进 deadlist、不进 defer（D3（空间分配） 已定项 7）。
    fn publish_head_deletion(&mut self, units: &[u32]) -> Result<u64, PublishRefusal> {
        self.begin_publish(PublishKind::Deletion)?;
        for unit in units.iter().copied() {
            let mut record = self.units[usize_of(u64::from(unit))];
            assert!(record.is_alive_in_head, "删的是头一侧还活着的单元");
            record.is_alive_in_head = false;
            if record.snapshot_pins == 0 {
                self.release_unit_slots(unit);
                record.is_released = true;
            }
            self.set_unit_record(unit, record);
        }
        self.finish_publish(PublishKind::Deletion, &[])
    }

    /// 删除类：销毁快照的一批（登记 M-8）：放掉这几个单元上这个快照的钉；没人钉、头一侧也删了的释放；这个快照钉住的文件树节点一起释放。
    fn publish_snapshot_destruction_batch(
        &mut self,
        units: &[u32],
        pinned_file_tree_offsets: &[u64],
    ) -> Result<u64, PublishRefusal> {
        self.begin_publish(PublishKind::Deletion)?;
        for unit in units.iter().copied() {
            let mut record = self.units[usize_of(u64::from(unit))];
            assert!(record.snapshot_pins > 0, "销毁的快照钉着它");
            record.snapshot_pins -= 1;
            if record.snapshot_pins == 0 && !record.is_alive_in_head {
                self.release_unit_slots(unit);
                record.is_released = true;
            }
            self.set_unit_record(unit, record);
        }
        for offset in pinned_file_tree_offsets.iter().copied() {
            assert_eq!(
                self.slot_state(offset).live_owner(),
                Some(SlotOwner::SnapshotPinnedFileTreeNode),
                "快照钉住的文件树节点还在"
            );
            self.release_slot(offset);
        }
        self.finish_publish(PublishKind::Deletion, &[])
    }

    fn publish_empty(&mut self) -> Result<u64, PublishRefusal> {
        self.begin_publish(PublishKind::Empty)?;
        let blocks = self.finish_publish(PublishKind::Empty, &[])?;
        self.round_metrics.empty_publishes += 1;
        Ok(blocks)
    }

    /// 建快照（登记 M-8）：钉住那一刻全部活的用户数据单元与这次发布写下的那 `c − b` 个文件树节点（不钉分配记录树）。
    fn publish_snapshot_creation(&mut self) -> Result<(Vec<u32>, Vec<u64>), PublishRefusal> {
        self.begin_publish(PublishKind::SnapshotCreation)?;
        let alive_units: Vec<u32> = (0..self.units.len())
            .filter(|unit| self.units[*unit].is_alive_in_head)
            .map(|unit| u32::try_from(unit).expect("单元数装得进 u32"))
            .collect();
        for unit in alive_units.iter().copied() {
            let mut record = self.units[usize_of(u64::from(unit))];
            record.snapshot_pins += 1;
            self.set_unit_record(unit, record);
        }
        self.finish_publish(PublishKind::SnapshotCreation, &[])?;
        let pinned_file_tree_offsets: Vec<u64> = self.file_tree_node_slots.iter().flatten().copied().collect();
        for offset in pinned_file_tree_offsets.iter().copied() {
            let SlotState::Live { allocation_txg, .. } = self.slot_state(offset) else {
                panic!("文件树节点的现行版本是已分配的");
            };
            self.set_slot_state(offset, SlotState::Live { owner: SlotOwner::SnapshotPinnedFileTreeNode, allocation_txg });
        }
        Ok((alive_units, pinned_file_tree_offsets))
    }

    /// 抬 F（登记 M-6 ①）：目标是环里第 4 新的非空根；F 已经不低于它就跳过；否则连发 2 次空发布，第 2 次之后生效。
    /// 空发布要的槽找不到一个记「删除发不出去」那一栏的空发布一列（登记 M-5）。
    fn raise_floor(&mut self) {
        let oldest_valid_root_txg = self.oldest_valid_root_txg_at(self.txg);
        let target = floor_raise_target(&self.non_empty_root_txgs_in_ring, oldest_valid_root_txg);
        if self.effective_floor_txg >= target {
            return;
        }
        self.pending_floor_raise = Some((target, 0));
        for _empty_publish in 0..EMPTY_PUBLISHES_FOR_FLOOR_TO_TAKE_EFFECT {
            if self.publish_empty().is_err() {
                self.round_metrics.empty_publish_failures += 1;
                return;
            }
        }
    }
}

/// 一批整理发布之后意图怎样了。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BatchOutcome {
    /// 一个都没搬成、R 里还有可搬的（没有目的地）：没发布，意图留着（暂停）。
    NothingMoved,
    /// 发了一批；意图结束了的话带它在 `ended_intents` 里的下标。
    Published { ended_intent_index: Option<usize> },
}

// ---- 整理（自建 GC，登记 M-7）：选段、签发、一批、收、两种停机、甲与丁 ----
impl Pool {
    /// greedy 选段（`compaction_plan.rs` 的 `choose_region`）：资格是不在这次挂载开过的段里、不是活意图的 R、不含钉住槽、
    /// 至少一个未释放记录；活槽最少的先，同分取起点最小。
    fn choose_region_greedy(&self) -> Option<u64> {
        let region = self.active_region_segment();
        let mut chosen: Option<(u64, u64)> = None;
        for segment in 0..self.geometry.segment_count() {
            let counters = &self.segments[usize_of(segment)];
            let opened_this_mount = self.is_opened_this_mount[usize_of(segment)];
            let is_eligible = !opened_this_mount && counters.pinned_live_slots == 0 && counters.live_slots > 0;
            if !is_eligible || Some(segment) == region {
                continue;
            }
            let live_slots = counters.live_slots;
            let is_better = chosen.is_none_or(|(best_live_slots, _best_segment)| live_slots < best_live_slots);
            if is_better {
                chosen = Some((live_slots, segment));
            }
        }
        chosen.map(|(_live_slots, segment)| segment)
    }

    /// 停乙的签发前判：R 外合格段的空闲两槽数 ≥ R 里可搬的用户数据单元数（D26（后台整理与放置回收） 已定项 1 第 2 条）。
    fn pre_issue_check_passes(&self, region_segment: u64) -> bool {
        self.compaction_destination_free_pairs_outside_region(region_segment)
            >= self.segments[usize_of(region_segment)].live_user_units
    }

    /// 三个起跑水位任一越线（登记 M-7 的登记取法）：全空段数（记录口径）< `N₀` + 2；runs ≥ L ÷ 4；不可回收槽 ≥ 单元区的 10%。
    fn any_start_watermark_crossed(&self) -> bool {
        let empty_segments_are_low = self.record_empty_segment_count < self.settings.floor_segments + 2;
        let runs_are_high = self.free_runs >= self.live_user_unit_total / 4;
        let unrecoverable_slots = self.released_slot_total + 2 * self.snapshot_only_unit_total;
        let unrecoverable_is_high = unrecoverable_slots >= self.geometry.unit_area_slots() / 10;
        empty_segments_are_low || runs_are_high || unrecoverable_is_high
    }

    /// R 里分配代 ≤ c0 的未释放记录：(总数, 其中钉住的)。
    fn region_live_records_at_or_before(&self, region_segment: u64, allocation_txg_at_issue: u64) -> (u64, u64) {
        let segment_start = region_segment * CLUSTER_SEGMENT_SLOTS;
        let mut live_records = 0;
        let mut pinned_records = 0;
        for offset in segment_start..segment_start + CLUSTER_SEGMENT_SLOTS {
            if let SlotState::Live { owner, allocation_txg } = self.slot_state(offset) {
                if allocation_txg <= allocation_txg_at_issue {
                    live_records += 1;
                    if owner.is_pinned() {
                        pinned_records += 1;
                    }
                }
            }
        }
        (live_records, pinned_records)
    }

    /// 一次调用整理入口甲：预算 `B_gc`；返回签了几条意图。
    fn call_compaction_entry(&mut self) -> u64 {
        let publishes_since_last_non_empty_root = self.txg - self.last_non_empty_root_txg;
        let budget = compaction_call_publish_budget(self.geometry.root_ring_slots_per_region, publishes_since_last_non_empty_root);
        self.run_compaction(budget)
    }

    /// 丁：前台写入类或删除类发布成功之后，起跑水位有一个越线就跑至多一批（登记 M-7；每次当一次预算为 1 的调用）。
    fn run_compaction_after_publish(&mut self) {
        if self.settings.is_compaction_enabled && self.any_start_watermark_crossed() {
            self.run_compaction(1);
        }
    }

    fn run_compaction(&mut self, publish_budget: u64) -> u64 {
        let mut publishes_used = 0;
        let mut intents_signed = 0;
        while publishes_used < publish_budget {
            let new_region = if self.active_intent.is_some() {
                None
            } else {
                let Some(region) = self.choose_region_greedy() else {
                    break;
                };
                let must_check_holes = self.settings.stop_arm == StopArm::LocalRegionPredicate;
                if must_check_holes && !self.pre_issue_check_passes(region) {
                    break;
                }
                Some(region)
            };
            let outcome = match self.publish_compaction_batch(new_region) {
                Ok(outcome) => outcome,
                Err(_refusal) => break,
            };
            let BatchOutcome::Published { ended_intent_index } = outcome else {
                break;
            };
            publishes_used += 1;
            intents_signed += u64::from(new_region.is_some());
            if let Some(index) = ended_intent_index {
                let global_difference_is_positive = self.ended_intents[index].global_difference_is_positive;
                let must_stop = match self.settings.stop_arm {
                    StopArm::LocalRegionPredicate => !global_difference_is_positive,
                    StopArm::GlobalDifference => global_difference_is_positive,
                };
                if must_stop {
                    break;
                }
            }
        }
        intents_signed
    }

    /// 一批（登记 M-7）：签发（`new_region` 给了的话，随这一批一起发）；按槽升序搬 R 里至多 K 个可搬单元到目的地，
    /// R 里的提交内生块随这一批重写；之后 R 里没有分配代 ≤ c0 的未释放记录就同一次发布里删意图，只剩钉住的就丢意图。
    fn publish_compaction_batch(&mut self, new_region: Option<u64>) -> Result<BatchOutcome, PublishRefusal> {
        if let Some(region_segment) = new_region {
            self.active_intent = Some(CompactionIntent {
                region_segment,
                allocation_txg_at_issue: self.txg,
                record_empty_segments_at_issue: 0,
                openable_segments_at_issue: 0,
            });
        }
        let intent = self.active_intent.expect("一批属于一条活意图");
        let region = intent.region_segment;
        if let Err(refusal) = self.begin_publish(PublishKind::CompactionBatch) {
            self.forget_intent_issued_by_a_batch_that_did_not_publish(new_region);
            return Err(refusal);
        }
        let segment_start = region * CLUSTER_SEGMENT_SLOTS;
        let user_units_in_region: Vec<u32> = (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
            .step_by(2)
            .filter_map(|even_offset| match self.slot_state(even_offset).live_owner() {
                Some(SlotOwner::UserDataUnit { unit }) => Some(unit),
                Some(SlotOwner::InstanceTable | SlotOwner::SnapshotPinnedFileTreeNode | SlotOwner::CommitGeneratedBlock { .. })
                | None => None,
            })
            .collect();
        let mut units_moved = 0;
        for unit in user_units_in_region.iter().copied().take(COMPACTION_BATCH_UNITS) {
            let Some(destination) = self.lowest_compaction_destination_pair() else {
                break;
            };
            let mut record = self.units[usize_of(u64::from(unit))];
            // 目的地那条分配记录的分配代写单元的诞生代（D26（后台整理与放置回收） 已定项 5 第 2 条）。
            self.allocate_slot(destination, SlotOwner::UserDataUnit { unit }, record.birth_txg);
            self.allocate_slot(destination + 1, SlotOwner::UserDataUnit { unit }, record.birth_txg);
            self.release_unit_slots(unit);
            record.first_slot_offset = destination;
            self.set_unit_record(unit, record);
            units_moved += 1;
        }
        let forced_blocks: Vec<BlockIdentity> = (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
            .filter_map(|offset| match self.slot_state(offset).live_owner() {
                Some(SlotOwner::CommitGeneratedBlock { block }) => Some(block),
                Some(SlotOwner::InstanceTable | SlotOwner::SnapshotPinnedFileTreeNode | SlotOwner::UserDataUnit { .. })
                | None => None,
            })
            .collect();
        if units_moved == 0 && forced_blocks.is_empty() {
            let (live_records, pinned_records) =
                self.region_live_records_at_or_before(region, intent.allocation_txg_at_issue);
            let region_can_end = live_records == pinned_records;
            if !region_can_end {
                self.roll_back_transaction();
                self.forget_intent_issued_by_a_batch_that_did_not_publish(new_region);
                return Ok(BatchOutcome::NothingMoved);
            }
        }
        if let Err(refusal) = self.finish_publish(PublishKind::CompactionBatch, &forced_blocks) {
            self.forget_intent_issued_by_a_batch_that_did_not_publish(new_region);
            return Err(refusal);
        }
        self.round_metrics.compaction_batches += 1;
        self.round_metrics.units_moved += units_moved;
        if new_region.is_some() {
            let record_empty_segments_at_issue = self.record_empty_segment_count;
            let openable_segments_at_issue = self.openable_segment_count();
            let issued = self.active_intent.as_mut().expect("刚签发的意图");
            issued.record_empty_segments_at_issue = record_empty_segments_at_issue;
            issued.openable_segments_at_issue = openable_segments_at_issue;
        }
        let intent = self.active_intent.expect("一批之后意图还在");
        let (live_records, pinned_records) = self.region_live_records_at_or_before(region, intent.allocation_txg_at_issue);
        let ended_intent_index = if live_records == 0 {
            Some(self.end_intent(intent, true))
        } else if live_records == pinned_records {
            Some(self.end_intent(intent, false))
        } else {
            None
        };
        Ok(BatchOutcome::Published { ended_intent_index })
    }

    fn forget_intent_issued_by_a_batch_that_did_not_publish(&mut self, new_region: Option<u64>) {
        if new_region.is_some() {
            self.active_intent = None;
        }
    }

    /// 删意图（或丢意图），记 `P_R`、`P_G`，排上 `Y1`、`Y2` 的判定（登记读法第 7 条）。
    fn end_intent(&mut self, intent: CompactionIntent, is_completed: bool) -> usize {
        let region = intent.region_segment;
        self.active_intent = None;
        let region_is_record_empty = self.membership_of_segment(region).is_record_empty;
        let global_difference = i128::from(self.record_empty_segment_count) - i128::from(intent.record_empty_segments_at_issue);
        let global_difference_is_positive = global_difference >= 1;
        let segment_start = region * CLUSTER_SEGMENT_SLOTS;
        let last_release_txg_in_region = (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
            .filter_map(|offset| match self.slot_state(offset) {
                SlotState::Released { release_txg } => Some(release_txg),
                SlotState::Free | SlotState::Live { .. } => None,
            })
            .max()
            .unwrap_or(0);
        self.ended_intents.push(EndedIntent {
            round: self.current_round,
            is_completed,
            region_is_record_empty,
            global_difference_is_positive,
            region_became_openable: None,
            openable_count_rose: None,
        });
        let ended_intent_index = self.ended_intents.len() - 1;
        self.pending_truth_judgements.push(PendingTruthJudgement {
            ended_intent_index,
            region_segment: region,
            end_txg: self.txg,
            last_release_txg_in_region,
            openable_segments_at_issue: intent.openable_segments_at_issue,
        });
        if is_completed {
            self.round_metrics.intents_completed += 1;
        } else {
            self.round_metrics.intents_discarded += 1;
        }
        ended_intent_index
    }
}

// ---- mkfs、重挂与守恒自检 ----
impl Pool {
    /// mkfs（txg 0）：实例表两槽钉住在单元区起点；第一次发布的提交内生块从最低的可开聚簇段 bump；之后挂载一次。
    fn made_by_make_filesystem(geometry: PoolGeometry, settings: PoolSettings) -> Self {
        let mut pool = Self::blank(geometry, settings);
        pool.begin_transaction();
        pool.allocate_slot(0, SlotOwner::InstanceTable, 0);
        pool.allocate_slot(1, SlotOwner::InstanceTable, 0);
        pool.is_making_filesystem = true;
        pool.rewrite_commit_generated_blocks(&[]).expect("空池上 mkfs 那一版的块放得下");
        pool.is_making_filesystem = false;
        pool.commit_transaction();
        assert!(pool.reclaim_queue.is_empty(), "mkfs 那一版没有旧版本可释放");
        pool.remount();
        pool
    }

    /// 重挂：「这次挂载开过的段」清空、bump 游标作废，下一块提交内生块开新段（登记 M-4）。
    fn remount(&mut self) {
        assert!(!self.is_transaction_open, "重挂不在事务里");
        let opened = std::mem::take(&mut self.segments_opened_this_mount);
        for segment in opened {
            self.set_segment_opened_this_mount(segment, false);
        }
        self.open_cursor = None;
    }

    /// 全扫重算，与增量计数逐项比（作废条款 V4）；不等就是装置错，当场停。
    fn verify_incremental_counters_against_a_full_scan(&self) {
        let mut segments = vec![SegmentCounters::default(); usize_of(self.geometry.segment_count())];
        let mut free_runs = 0;
        let mut previous_is_free = false;
        let mut live_user_units = 0;
        for offset in 0..self.geometry.unit_area_slots() {
            let state = self.slot_state(offset);
            let counters = &mut segments[usize_of(Self::segment_of_offset(offset))];
            match state {
                SlotState::Free => counters.free_slots += 1,
                SlotState::Live { owner, .. } => {
                    counters.live_slots += 1;
                    if owner.is_pinned() {
                        counters.pinned_live_slots += 1;
                    }
                    if owner.is_user_data() && offset % 2 == 0 {
                        counters.live_user_units += 1;
                        live_user_units += 1;
                    }
                }
                SlotState::Released { .. } => counters.released_slots += 1,
            }
            if offset % 2 == 1 && self.is_pair_free(offset - 1) {
                counters.free_pairs += 1;
            }
            if is_free_for_runs(state) && !previous_is_free {
                free_runs += 1;
            }
            previous_is_free = is_free_for_runs(state);
        }
        assert_eq!(segments, self.segments, "逐段计数与全扫重算对得上（V4）");
        assert_eq!(free_runs, self.free_runs, "runs 增量与全扫重算对得上（V4）");
        assert_eq!(live_user_units, self.live_user_unit_total, "活的用户数据单元数对得上（V4）");
        let free_total: u64 = segments.iter().map(|counters| counters.free_slots).sum();
        let live_total: u64 = segments.iter().map(|counters| counters.live_slots).sum();
        let released_total: u64 = segments.iter().map(|counters| counters.released_slots).sum();
        assert_eq!(free_total + live_total + released_total, self.geometry.unit_area_slots(), "三态守恒（V4）");
        assert_eq!((free_total, released_total), (self.free_slot_total, self.released_slot_total), "全池计数（V4）");
        let record_empty = segments.iter().filter(|counters| counters.live_slots == 0).count() as u64;
        assert_eq!(record_empty, self.record_empty_segment_count, "记录口径全空段数（V4）");
        let fully_free = segments.iter().filter(|counters| counters.free_slots == CLUSTER_SEGMENT_SLOTS).count() as u64;
        assert_eq!(fully_free, self.fully_free_segments.len() as u64, "占用口径全空段数（V4）");
        let destination_pairs: u64 = (0..self.geometry.segment_count())
            .filter(|segment| self.membership_of_segment(*segment).is_compaction_destination)
            .map(|segment| segments[usize_of(segment)].free_pairs)
            .sum();
        assert_eq!(destination_pairs, self.compaction_destination_free_pairs, "整理目的地的空闲两槽数（V4）");
        let user_placement: BTreeSet<u64> = (0..self.geometry.segment_count())
            .filter(|segment| self.membership_of_segment(*segment).is_user_placement_candidate)
            .collect();
        assert_eq!(user_placement, self.user_placement_segments, "用户数据候选段集合（V4）");
        let with_a_free_slot: BTreeSet<u64> = (0..self.geometry.segment_count())
            .filter(|segment| segments[usize_of(*segment)].free_slots > 0)
            .collect();
        assert_eq!(with_a_free_slot, self.segments_with_a_free_slot, "有空闲槽的段集合（V4）");
        let snapshot_only = self.units.iter().filter(|record| record.is_referenced_only_by_snapshots()).count() as u64;
        assert_eq!(snapshot_only, self.snapshot_only_unit_total, "只被快照引用的单元数（V4）");
    }
}

/// 删段怎么挑（登记 M-9）：主取样点整文件删，反向点按单元随机删。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeletionMode {
    WholeFiles,
    RandomUnits,
}

/// 一个取样点：主取样点与第八节六个反向点，每个反向点只换一个旋钮。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SamplingPoint {
    name: &'static str,
    disk_slots: u64,
    leaf_width_in_slots: u64,
    remounts_every_round: bool,
    deletion_mode: DeletionMode,
    snapshot_limit: u64,
    resident_base_multiplier: u64,
}

const MAIN_SAMPLING_POINT: SamplingPoint = SamplingPoint {
    name: "main",
    disk_slots: FOUR_GIBIBYTE_DISK_SLOTS,
    leaf_width_in_slots: ALLOCATION_RECORD_TREE_LEAF_SLOTS,
    remounts_every_round: true,
    deletion_mode: DeletionMode::WholeFiles,
    snapshot_limit: 0,
    resident_base_multiplier: 3,
};

/// 主取样点在前，六个反向点按登记第八节表的次序；每个反向点只有一个字段与主取样点不同。
const SAMPLING_POINTS: [SamplingPoint; 7] = [
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
enum FloorArm {
    One,
    Two,
    Formula,
    Zero,
}

const FLOOR_ARMS: [FloorArm; 4] = [FloorArm::One, FloorArm::Two, FloorArm::Formula, FloorArm::Zero];
const CANDIDATE_FLOOR_ARMS: [FloorArm; 3] = [FloorArm::One, FloorArm::Two, FloorArm::Formula];

impl FloorArm {
    fn floor_segments(self, root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64) -> u64 {
        match self {
            FloorArm::One => 1,
            FloorArm::Two => 2,
            FloorArm::Formula => formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish),
            FloorArm::Zero => 0,
        }
    }

    fn tag(self) -> &'static str {
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
struct RunKey {
    point: SamplingPoint,
    root_ring_slots_per_region: u64,
    commit_generated_blocks_per_publish: u64,
    arm: FloorArm,
    is_compaction_enabled: bool,
    seed: u64,
}

impl RunKey {
    fn geometry(&self) -> PoolGeometry {
        PoolGeometry::new(
            self.point.disk_slots,
            self.point.leaf_width_in_slots,
            self.root_ring_slots_per_region,
            self.commit_generated_blocks_per_publish,
        )
    }

    fn label(&self) -> String {
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
    units: Vec<u32>,
    pinned_file_tree_offsets: Vec<u64>,
}

/// 一轮结束时记下的一行（逐轮值整列写进产物，登记第八节「轨迹」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RoundRow {
    round: u64,
    metrics: RoundMetrics,
    free_runs: u64,
    live_user_units: u64,
    unrecoverable_slots: u64,
    record_empty_segments_at_end: u64,
}

/// 一条 FDF 跑（登记 M-9）：每轮 ① 填满 ② 有快照时建一个 ③ 删一半 ④ 调一次甲 ⑤ 重挂。
struct FillDeleteFillRun {
    key: RunKey,
    pool: Pool,
    random: SeededRandom,
    /// 整文件删时用：头一侧还活着、没进删除队列的文件，每个文件是它的单元。
    live_files: Vec<Vec<u32>>,
    pending_deletions: VecDeque<DeletionBatch>,
    snapshots: VecDeque<Snapshot>,
    rows: Vec<RoundRow>,
}

impl FillDeleteFillRun {
    fn new(key: RunKey) -> Self {
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

    fn run_round(&mut self, round: u64) {
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
        let pool = &self.pool;
        self.rows.push(RoundRow {
            round,
            metrics: pool.round_metrics,
            free_runs: pool.free_runs,
            live_user_units: pool.live_user_unit_total,
            unrecoverable_slots: pool.released_slot_total + 2 * pool.snapshot_only_unit_total,
            record_empty_segments_at_end: pool.record_empty_segment_count,
        });
    }

    /// 跑 60 轮；删除类发布的块数判「有界」的续到 120 轮做加倍复核（登记第六节第 3 行）。
    fn run_to_judgement(mut self) -> FinishedRun {
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
struct FinishedRun {
    key: RunKey,
    rows: Vec<RoundRow>,
    ended_intents: Vec<EndedIntent>,
    growth: GrowthVerdict,
}

// ---- 判定（登记第六节第 1 组；跑前写死的门槛） ----

/// 第 3 行：删除类发布的 `c_actual` 随轮数是否发散。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GrowthVerdict {
    Bounded,
    Diverging,
    /// 窗口里一轮删除类发布都没有：读不到，不当 0（`test-discipline.md`「读不到 ≠ 读到 0」）。
    Unreadable,
}

impl GrowthVerdict {
    fn tag(self) -> &'static str {
        match self {
            GrowthVerdict::Bounded => "bounded",
            GrowthVerdict::Diverging => "diverging",
            GrowthVerdict::Unreadable => "unreadable",
        }
    }
}

/// 第 3 行的两个门槛，写成参数只为判别力自证（登记第八节末段）能把它挪到两点之间。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GrowthThresholds {
    /// 后窗口均值 ≤ 这个百分数 × 前窗口均值。
    mean_growth_percent: u64,
    /// 后窗口峰值 < 这个十分数 × 结构上限。
    structural_bound_tenths: u64,
}

const GROWTH_THRESHOLDS: GrowthThresholds = GrowthThresholds { mean_growth_percent: 105, structural_bound_tenths: 9 };

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
fn judge_deletion_block_growth(rows: &[RoundRow], structural_bound: u64, thresholds: &GrowthThresholds) -> GrowthVerdict {
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
fn judge_doubled_growth_check(rows: &[RoundRow]) -> GrowthVerdict {
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
fn deletion_failures_of(rows: &[RoundRow]) -> u64 {
    rows.iter().map(|row| row.metrics.deletion_publish_failures + row.metrics.empty_publish_failures).sum()
}

/// 五个种子合成一格一臂的判定：五个都判同一个值才写那个值，否则「不稳定」（登记第六节开头）。
fn combine_seed_tags(tags: &[&'static str]) -> &'static str {
    assert_eq!(tags.len() as u64, SEEDS_PER_CELL, "一格一臂恰好五个种子");
    if tags.iter().all(|tag| *tag == tags[0]) {
        tags[0]
    } else {
        "unstable"
    }
}

fn failure_tag(deletion_failures: u64) -> &'static str {
    if deletion_failures == 0 {
        "none"
    } else {
        "failure"
    }
}

/// 一条被谓词消费的量的轨迹（`test-discipline.md`「端点不是轨迹」）：吃光初始供给之后的峰值（第 2 轮起）、为正的轮数、
/// 低于 `N₀` 的轮数、期末值、全程最小值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TrajectoryStatistics {
    peak_from_round_two: u64,
    rounds_positive: u64,
    rounds_below_floor: u64,
    final_value: u64,
    lowest: u64,
}

fn trajectory_statistics(values: &[u64], floor_segments: u64) -> TrajectoryStatistics {
    assert!(values.len() >= 2, "至少两轮才有「第 2 轮起」");
    TrajectoryStatistics {
        peak_from_round_two: values[1..].iter().copied().max().expect("非空"),
        rounds_positive: values.iter().filter(|value| **value > 0).count() as u64,
        rounds_below_floor: values.iter().filter(|value| **value < floor_segments).count() as u64,
        final_value: *values.last().expect("非空"),
        lowest: values.iter().copied().min().expect("非空"),
    }
}

fn trajectory_fields(prefix: &str, statistics: &TrajectoryStatistics) -> String {
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
struct IntentErrorCounts {
    ended: u64,
    discarded: u64,
    judged: u64,
    region_predicate_true_truth_false: u64,
    region_predicate_false_truth_true: u64,
    global_difference_true_truth_false: u64,
    global_difference_false_truth_true: u64,
    global_difference_true_openable_rise_false: u64,
    global_difference_false_openable_rise_true: u64,
}

impl IntentErrorCounts {
    fn of(ended_intents: &[EndedIntent]) -> Self {
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

    fn add(&mut self, other: &Self) {
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

    fn fields(&self) -> String {
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
    fn region_predicate_false_positive_share_reaches_threshold(&self) -> Option<bool> {
        if self.judged == 0 {
            return None;
        }
        Some(self.region_predicate_true_truth_false * 100 >= PREDICATE_FALSE_POSITIVE_SHARE_PERCENT * self.judged)
    }
}

fn bit_column<Item>(items: &[Item], bit_of: impl Fn(&Item) -> Option<bool>) -> String {
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

/// 一条跑交给格一级判定的东西。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RunSummary {
    key: RunKey,
    growth: GrowthVerdict,
    deletion_failures: u64,
    intent_errors: IntentErrorCounts,
}

fn round_line(label: &str, row: &RoundRow) -> String {
    let metrics = &row.metrics;
    let free_at_refusal = metrics.free_slots_at_last_refusal.map_or_else(|| "NA".to_string(), |slots| slots.to_string());
    format!(
        "name=round {label} round={} lowest_record_empty_segments={} lowest_fully_free_segments={} lowest_openable_segments={} deletion_publishes={} deletion_commit_generated_block_peak={} deletion_commit_generated_block_sum={} fallback={} deletion_publish_failures={} empty_publish_failures={} refused_writes_by_conjunct_one={} refused_writes_by_conjunct_seven={} free_slots_at_last_refusal={free_at_refusal} writes={} batches={} moved={} empties={} intents_done={} intents_dropped={} runs={} live_units={} unrecoverable={} record_empty_segments_at_end={}",
        row.round,
        metrics.lowest_record_empty_segments,
        metrics.lowest_fully_free_segments,
        metrics.lowest_openable_segments,
        metrics.deletion_publishes,
        metrics.deletion_commit_generated_block_peak,
        metrics.deletion_commit_generated_block_sum,
        metrics.fallback_blocks,
        metrics.deletion_publish_failures,
        metrics.empty_publish_failures,
        metrics.refused_writes_by_conjunct_one,
        metrics.refused_writes_by_conjunct_seven,
        metrics.foreground_writes,
        metrics.compaction_batches,
        metrics.units_moved,
        metrics.empty_publishes,
        metrics.intents_completed,
        metrics.intents_discarded,
        row.free_runs,
        row.live_user_units,
        row.unrecoverable_slots,
        row.record_empty_segments_at_end
    )
}

/// 一条跑的全部产物行（逐轮行、逐意图列、汇总行）与交给格一级的汇总。
fn lines_of_finished_run(run: &FinishedRun) -> (Vec<String>, RunSummary) {
    let label = run.key.label();
    let floor_segments = run.key.arm.floor_segments(run.key.root_ring_slots_per_region, run.key.commit_generated_blocks_per_publish);
    let mut lines: Vec<String> = run.rows.iter().map(|row| round_line(&label, row)).collect();
    let intents = &run.ended_intents;
    lines.push(format!(
        "name=intents {label} count={} completed={} region_is_record_empty={} global_difference_is_positive={} region_became_openable={} openable_count_rose={}",
        intents.len(),
        bit_column(intents, |intent| Some(intent.is_completed)),
        bit_column(intents, |intent| Some(intent.region_is_record_empty)),
        bit_column(intents, |intent| Some(intent.global_difference_is_positive)),
        bit_column(intents, |intent| intent.region_became_openable),
        bit_column(intents, |intent| intent.openable_count_rose)
    ));
    let series = |value_of: fn(&RoundRow) -> u64| -> Vec<u64> { run.rows.iter().map(value_of).collect() };
    let record_empty = trajectory_statistics(&series(|row| row.metrics.lowest_record_empty_segments), floor_segments);
    let fully_free = trajectory_statistics(&series(|row| row.metrics.lowest_fully_free_segments), floor_segments);
    let openable = trajectory_statistics(&series(|row| row.metrics.lowest_openable_segments), floor_segments);
    let fallback = trajectory_statistics(&series(|row| row.metrics.fallback_blocks), 0);
    let deletion_failures = deletion_failures_of(&run.rows);
    let deletion_publish_failures: u64 = run.rows.iter().map(|row| row.metrics.deletion_publish_failures).sum();
    let empty_publish_failures: u64 = run.rows.iter().map(|row| row.metrics.empty_publish_failures).sum();
    let refused_by_one: u64 = run.rows.iter().map(|row| row.metrics.refused_writes_by_conjunct_one).sum();
    let refused_by_seven: u64 = run.rows.iter().map(|row| row.metrics.refused_writes_by_conjunct_seven).sum();
    let intent_errors = IntentErrorCounts::of(intents);
    lines.push(format!(
        "name=run {label} rounds={} segment_floor={floor_segments} deletion_block_growth={} deletion_failures_verdict={} deletion_failures={deletion_failures} deletion_publish_failures={deletion_publish_failures} empty_publish_failures={empty_publish_failures} {} {} {} fallback_peak_from_round_two={} fallback_positive_rounds={} fallback_final={} refused_writes_by_conjunct_one={refused_by_one} refused_writes_by_conjunct_seven={refused_by_seven} {}",
        run.rows.len(),
        run.growth.tag(),
        failure_tag(deletion_failures),
        trajectory_fields("record_empty_segments", &record_empty),
        trajectory_fields("fully_free_segments", &fully_free),
        trajectory_fields("openable_segments", &openable),
        fallback.peak_from_round_two,
        fallback.rounds_positive,
        fallback.final_value,
        intent_errors.fields()
    ));
    let summary = RunSummary { key: run.key, growth: run.growth, deletion_failures, intent_errors };
    (lines, summary)
}

/// 第 7 行：空池上只发空发布 1000 次，记不可开聚簇段数（开放段 + 装着未回收块的段）的峰值。
fn metadata_stream_peak_unopenable_segments(geometry: PoolGeometry) -> u64 {
    let settings = PoolSettings {
        floor_segments: 0,
        is_compaction_enabled: false,
        stop_arm: StopArm::LocalRegionPredicate,
        resident_base_multiplier: 3,
    };
    let mut pool = Pool::made_by_make_filesystem(geometry, settings);
    let mut peak = 0;
    for _empty_publish in 0..METADATA_STREAM_EMPTY_PUBLISHES {
        pool.publish_empty().expect("空池上的空发布放得下");
        let open_segment = pool.open_cursor.map(|cursor| cursor.segment);
        let unopenable = (0..geometry.segment_count())
            .filter(|segment| Some(*segment) == open_segment || pool.segments[usize_of(*segment)].released_slots > 0)
            .count() as u64;
        peak = peak.max(unopenable);
    }
    peak
}

/// 一份分片跑哪些：一个取样点，可再限一个 S（不带参数的全跑是 None）。分片按（取样点, S）的次序拼起来就是全跑的次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShardSelection<'name> {
    point: &'name str,
    root_ring_slots_per_region: Option<u64>,
}

fn is_selected(selection: Option<ShardSelection>, point: &SamplingPoint, root_ring_slots_per_region: u64) -> bool {
    selection.is_none_or(|shard| {
        shard.point == point.name && shard.root_ring_slots_per_region.is_none_or(|wanted| wanted == root_ring_slots_per_region)
    })
}

/// 第一段的全部正式跑：7 个取样点 × 9 格 × 4 档 `N₀` × 整理开关 × 5 个种子，按这个次序排（产物行的次序就是它）。
fn all_run_keys(selection: Option<ShardSelection>) -> Vec<RunKey> {
    let mut keys = Vec::new();
    for point in SAMPLING_POINTS.iter().copied() {
        for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
            if !is_selected(selection, &point, root_ring_slots_per_region) {
                continue;
            }
            for commit_generated_blocks_per_publish in COMMIT_GENERATED_BLOCKS_CHOICES {
                for arm in FLOOR_ARMS {
                    for is_compaction_enabled in [true, false] {
                        for seed in 1..=SEEDS_PER_CELL {
                            keys.push(RunKey {
                                point,
                                root_ring_slots_per_region,
                                commit_generated_blocks_per_publish,
                                arm,
                                is_compaction_enabled,
                                seed,
                            });
                        }
                    }
                }
            }
        }
    }
    keys
}

/// 线程数：`E10_THREADS`，没设取 `SINGLEFS_THREAD_CAP`（`scripts/hosts/capped.sh` 设的），都没设取 1。输出与线程数无关。
fn worker_thread_count() -> usize {
    ["E10_THREADS", "SINGLEFS_THREAD_CAP"]
        .iter()
        .find_map(|variable| std::env::var(variable).ok())
        .map_or(1, |text| text.parse::<usize>().expect("线程数是正整数").max(1))
}

/// 并行跑全部正式跑，按键的次序交回（各片互不共享状态，合并按下标，确定）。
fn run_fill_delete_fill_runs_in_parallel(keys: &[RunKey], threads: usize) -> Vec<(Vec<String>, RunSummary)> {
    let next_index = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<(Vec<String>, RunSummary)>>> = Mutex::new(vec![None; keys.len()]);
    std::thread::scope(|scope| {
        for _worker in 0..threads {
            scope.spawn(|| loop {
                let index = next_index.fetch_add(1, Ordering::Relaxed);
                let Some(key) = keys.get(index) else {
                    break;
                };
                let finished = FillDeleteFillRun::new(*key).run_to_judgement();
                let produced = lines_of_finished_run(&finished);
                results.lock().expect("没有工作线程在持锁时 panic")[index] = Some(produced);
            });
        }
    });
    results
        .into_inner()
        .expect("工作线程都已结束")
        .into_iter()
        .map(|produced| produced.expect("每条跑都交回了"))
        .collect()
}

/// 一格一臂一个整理开关上五个种子合成的判定。
#[derive(Clone, Debug, PartialEq, Eq)]
struct CellArmVerdict {
    growth: &'static str,
    failures: &'static str,
    growth_seeds: String,
    failure_seeds: String,
}

fn cell_arm_verdict(summaries: &[&RunSummary]) -> CellArmVerdict {
    let growth_tags: Vec<&'static str> = summaries.iter().map(|summary| summary.growth.tag()).collect();
    let failure_tags: Vec<&'static str> = summaries.iter().map(|summary| failure_tag(summary.deletion_failures)).collect();
    CellArmVerdict {
        growth: combine_seed_tags(&growth_tags),
        failures: combine_seed_tags(&failure_tags),
        growth_seeds: growth_tags.join("/"),
        failure_seeds: summaries.iter().map(|summary| summary.deletion_failures.to_string()).collect::<Vec<_>>().join("/"),
    }
}

/// 阳性对照（登记第五节）：同一臂整理关掉要退化——第 3 行「发散」或第 4 行「失败」。
fn off_control_degrades(off: &CellArmVerdict) -> bool {
    off.growth == GrowthVerdict::Diverging.tag() || off.failures == "failure"
}

/// 一格一个候选的判定：第 3 行发散或第 4 行失败 ⇒ 不够；第 3 行有界、第 4 行无、对照成立、没打中 F3 ⇒ 过；其余不够判。
fn candidate_cell_verdict(on: (&str, &str), control_holds: bool, shared_premise_hit: bool) -> &'static str {
    let (growth, failures) = on;
    if growth == GrowthVerdict::Diverging.tag() || failures == "failure" {
        "fail"
    } else if growth == GrowthVerdict::Bounded.tag() && failures == "none" && control_holds && !shared_premise_hit {
        "pass"
    } else {
        "not_judgeable"
    }
}

/// 几何敏感性（登记第八节、F7）：主取样点与六个反向点判定都一样才取那个值，否则「不稳定」。
fn across_sampling_points(tags: &[&'static str]) -> &'static str {
    if tags.iter().all(|tag| *tag == tags[0]) {
        tags[0]
    } else {
        "unstable"
    }
}

/// 格一级与跨取样点的判定行（登记第六节第 1、2 组，第八节，第十节 F1–F3、F7）。`has_all_points` 为假（只跑了一个取样点）时不出跨点的行。
fn cell_and_verdict_lines(summaries: &[RunSummary], has_all_points: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let pick = |point: &str, root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64, arm: FloorArm, is_compaction_enabled: bool| {
        summaries
            .iter()
            .filter(|summary| {
                let key = &summary.key;
                key.point.name == point
                    && key.root_ring_slots_per_region == root_ring_slots_per_region
                    && key.commit_generated_blocks_per_publish == commit_generated_blocks_per_publish
                    && key.arm == arm
                    && key.is_compaction_enabled == is_compaction_enabled
            })
            .collect::<Vec<&RunSummary>>()
    };
    let present_points: Vec<SamplingPoint> = SAMPLING_POINTS
        .iter()
        .copied()
        .filter(|point| summaries.iter().any(|summary| summary.key.point.name == point.name))
        .collect();
    // (S, c, 候选下标) → 每个取样点一项 (第 3 行, 第 4 行, 对照成立, F3)，按取样点次序。
    let mut across_points: Vec<((u64, u64, usize), Vec<(&'static str, &'static str, bool, bool)>)> = Vec::new();
    let mut flagged_intent_cells = 0;
    let mut unreadable_intent_cells = 0;
    let mut pooled_all_points = IntentErrorCounts::default();
    for point in &present_points {
        for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
            for commit_generated_blocks_per_publish in COMMIT_GENERATED_BLOCKS_CHOICES {
                let cell = format!(
                    "point={} root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish}",
                    point.name
                );
                let mut on_verdicts = Vec::new();
                let mut off_verdicts = Vec::new();
                for arm in FLOOR_ARMS {
                    for is_compaction_enabled in [true, false] {
                        let verdict = cell_arm_verdict(&pick(
                            point.name,
                            root_ring_slots_per_region,
                            commit_generated_blocks_per_publish,
                            arm,
                            is_compaction_enabled,
                        ));
                        lines.push(format!(
                            "name=cell_arm {cell} arm={} compaction={} deletion_block_growth={} deletion_failures_verdict={} deletion_block_growth_seeds={} deletion_failures_seeds={}",
                            arm.tag(),
                            if is_compaction_enabled { "on" } else { "off" },
                            verdict.growth,
                            verdict.failures,
                            verdict.growth_seeds,
                            verdict.failure_seeds
                        ));
                        if is_compaction_enabled {
                            on_verdicts.push(verdict);
                        } else {
                            off_verdicts.push(verdict);
                        }
                    }
                }
                let diverging = GrowthVerdict::Diverging.tag();
                let shared_premise_hit = on_verdicts.iter().chain(off_verdicts.iter()).all(|verdict| verdict.growth == diverging);
                lines.push(format!("name=shared_premise {cell} all_eight_deletion_block_growth_diverging={shared_premise_hit}"));
                for (candidate_index, arm) in CANDIDATE_FLOOR_ARMS.iter().enumerate() {
                    let control_holds = off_control_degrades(&off_verdicts[candidate_index]);
                    let on = &on_verdicts[candidate_index];
                    let verdict = candidate_cell_verdict((on.growth, on.failures), control_holds, shared_premise_hit);
                    lines.push(format!(
                        "name=control {cell} arm={} off_deletion_block_growth={} off_deletion_failures_verdict={} off_control_degrades={control_holds}",
                        arm.tag(),
                        off_verdicts[candidate_index].growth,
                        off_verdicts[candidate_index].failures
                    ));
                    lines.push(format!(
                        "name=cell_candidate {cell} arm={} deletion_block_growth={} deletion_failures_verdict={} verdict={verdict}",
                        arm.tag(),
                        on.growth,
                        on.failures
                    ));
                    let entry_key = (root_ring_slots_per_region, commit_generated_blocks_per_publish, candidate_index);
                    let entry = (on.growth, on.failures, control_holds, shared_premise_hit);
                    match across_points.iter_mut().find(|(key, _entries)| *key == entry_key) {
                        Some((_key, entries)) => entries.push(entry),
                        None => across_points.push((entry_key, vec![entry])),
                    }
                }
                let mut pooled = IntentErrorCounts::default();
                for arm in FLOOR_ARMS {
                    for summary in pick(point.name, root_ring_slots_per_region, commit_generated_blocks_per_publish, arm, true) {
                        pooled.add(&summary.intent_errors);
                    }
                }
                pooled_all_points.add(&pooled);
                let share_flag = match pooled.region_predicate_false_positive_share_reaches_threshold() {
                    Some(true) => {
                        flagged_intent_cells += 1;
                        "true"
                    }
                    Some(false) => "false",
                    None => {
                        unreadable_intent_cells += 1;
                        "unreadable"
                    }
                };
                lines.push(format!(
                    "name=intent_errors {cell} pooled=compaction_on_all_segment_floor_arms_all_seeds {} region_is_record_empty_true_region_became_openable_false_share_at_least_5_percent={share_flag}",
                    pooled.fields()
                ));
            }
        }
    }
    lines.push(format!(
        "name=verdict_row2_segment1 points={} cells_with_region_is_record_empty_true_region_became_openable_false_share_at_least_5_percent={flagged_intent_cells} cells_unreadable={unreadable_intent_cells} pooled_{} stop_unable_to_stop_starting_points=not_run_in_segment1",
        present_points.len(),
        pooled_all_points.fields()
    ));
    if has_all_points {
        lines.extend(across_point_lines(&across_points));
    }
    lines
}

type AcrossPointEntries = Vec<((u64, u64, usize), Vec<(&'static str, &'static str, bool, bool)>)>;

/// 跨取样点：每格每个候选第 3、4 行在七个取样点上翻不翻面（F7），合成九格的判定，再按登记第六节第 1 组末段取第 4 行模型那一半的候选。
fn across_point_lines(across_points: &AcrossPointEntries) -> Vec<String> {
    let mut lines = Vec::new();
    let mut verdicts_by_candidate: Vec<Vec<&'static str>> = vec![Vec::new(); CANDIDATE_FLOOR_ARMS.len()];
    let mut cells_all_three_fail = 0;
    for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
        for commit_generated_blocks_per_publish in COMMIT_GENERATED_BLOCKS_CHOICES {
            let mut cell_verdicts = Vec::new();
            for (candidate_index, arm) in CANDIDATE_FLOOR_ARMS.iter().enumerate() {
                let entries = &across_points
                    .iter()
                    .find(|(key, _entries)| *key == (root_ring_slots_per_region, commit_generated_blocks_per_publish, candidate_index))
                    .expect("每格每个候选都有")
                    .1;
                assert_eq!(entries.len(), SAMPLING_POINTS.len(), "七个取样点都有");
                let growth_tags: Vec<&'static str> = entries.iter().map(|entry| entry.0).collect();
                let failure_tags: Vec<&'static str> = entries.iter().map(|entry| entry.1).collect();
                let final_growth = across_sampling_points(&growth_tags);
                let final_failures = across_sampling_points(&failure_tags);
                let controls_hold_everywhere = entries.iter().all(|entry| entry.2);
                let shared_premise_hit_somewhere = entries.iter().any(|entry| entry.3);
                for (quantity, tags, final_tag) in
                    [("deletion_block_growth", &growth_tags, final_growth), ("deletion_failures_verdict", &failure_tags, final_failures)]
                {
                    let per_point: Vec<String> = SAMPLING_POINTS
                        .iter()
                        .zip(tags.iter())
                        .map(|(point, tag)| format!("{}={tag}", point.name))
                        .collect();
                    lines.push(format!(
                        "name=sensitivity root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish} arm={} quantity={quantity} {} final={final_tag}",
                        arm.tag(),
                        per_point.join(" ")
                    ));
                }
                let verdict = candidate_cell_verdict(
                    (final_growth, final_failures),
                    controls_hold_everywhere,
                    shared_premise_hit_somewhere,
                );
                verdicts_by_candidate[candidate_index].push(verdict);
                cell_verdicts.push(verdict);
            }
            let all_three_fail = cell_verdicts.iter().all(|verdict| *verdict == "fail");
            cells_all_three_fail += u64::from(all_three_fail);
            lines.push(format!(
                "name=fork_row4_cell root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish} segment_floor_one={} segment_floor_two={} segment_floor_formula={} segment_floor_formula_value={} segment_floor_formula_is_the_same_arm_as_segment_floor_two={} all_three_candidates_insufficient={all_three_fail}",
                cell_verdicts[0],
                cell_verdicts[1],
                cell_verdicts[2],
                formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish),
                formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish) == 2
            ));
        }
    }
    let count_of = |verdicts: &[&'static str], wanted: &str| verdicts.iter().filter(|verdict| **verdict == wanted).count();
    let mut fields = Vec::new();
    for (candidate_index, arm) in CANDIDATE_FLOOR_ARMS.iter().enumerate() {
        let verdicts = &verdicts_by_candidate[candidate_index];
        fields.push(format!(
            "{}_cells_pass={} {}_cells_fail={} {}_cells_not_judgeable={}",
            arm.tag(),
            count_of(verdicts, "pass"),
            arm.tag(),
            count_of(verdicts, "fail"),
            arm.tag(),
            count_of(verdicts, "not_judgeable")
        ));
    }
    let candidate_names = ["1", "2", "formula"];
    let smallest_passing_all_nine = (0..CANDIDATE_FLOOR_ARMS.len())
        .find(|candidate_index| verdicts_by_candidate[*candidate_index].iter().all(|verdict| *verdict == "pass"))
        .map_or("none", |candidate_index| candidate_names[candidate_index]);
    let smallest_without_failing_cell = (0..CANDIDATE_FLOOR_ARMS.len())
        .find(|candidate_index| verdicts_by_candidate[*candidate_index].iter().all(|verdict| *verdict != "fail"))
        .map_or("none", |candidate_index| candidate_names[candidate_index]);
    lines.push(format!(
        "name=verdict_row4_model_half {} cells_all_three_insufficient={cells_all_three_fail} smallest_candidate_passing_all_nine_cells={smallest_passing_all_nine} smallest_candidate_without_a_failing_cell={smallest_without_failing_cell} implementation_arm=not_run_segment3",
        fields.join(" ")
    ));
    lines
}

/// 登记第七节甲 1 的九格算式值 (S, c, 值)：跑前由设计员算出、执行员另算一遍对上（报告里有命令）。
const FORMULA_VALUES_REGISTERED_BEFORE_THE_RUN: [(u64, u64, u64); 9] = [
    (4, 5, 2),
    (4, 10, 3),
    (4, 13, 4),
    (8, 5, 3),
    (8, 10, 5),
    (8, 13, 6),
    (16, 5, 5),
    (16, 10, 9),
    (16, 13, 11),
];

/// 第 1 行：九格算式值与跑前登记的值比。
fn formula_lines() -> Vec<String> {
    FORMULA_VALUES_REGISTERED_BEFORE_THE_RUN
        .iter()
        .map(|(root_ring_slots_per_region, commit_generated_blocks_per_publish, registered)| {
            let value = formula_floor_segments(*root_ring_slots_per_region, *commit_generated_blocks_per_publish);
            format!(
                "name=formula root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish} ring_slots_minus_one_times_commit_generated_blocks={} segment_floor_formula={value} registered={registered} matches_registered={}",
                (ROOT_RING_REGIONS * root_ring_slots_per_region - 1) * commit_generated_blocks_per_publish,
                value == *registered
            )
        })
        .collect()
}

/// 第 7 行：只在几何不同的三个取样点上量（另外四个反向点的旋钮不碰空发布流）。
fn metadata_stream_lines(selection: Option<ShardSelection>) -> Vec<String> {
    let mut lines = Vec::new();
    for point in SAMPLING_POINTS.iter().filter(|point| ["main", "disk_1gib", "leaf_203"].contains(&point.name)) {
        for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
            if !is_selected(selection, point, root_ring_slots_per_region) {
                continue;
            }
            for commit_generated_blocks_per_publish in COMMIT_GENERATED_BLOCKS_CHOICES {
                let geometry = PoolGeometry::new(
                    point.disk_slots,
                    point.leaf_width_in_slots,
                    root_ring_slots_per_region,
                    commit_generated_blocks_per_publish,
                );
                let peak = metadata_stream_peak_unopenable_segments(geometry);
                let formula = formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish);
                lines.push(format!(
                    "name=metadata_stream point={} root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish} empty_publishes={METADATA_STREAM_EMPTY_PUBLISHES} peak_unopenable_segments={peak} segment_floor_formula={formula} peak_minus_formula={} formula_undercounts={}",
                    point.name,
                    i128::from(peak) - i128::from(formula),
                    peak > formula
                ));
            }
        }
    }
    lines
}

/// 从一行 `name=run …` 里取回格一级判定要的那几样（`--merge` 用；单测钉住 格式化 → 取回 逐项相同）。
fn run_summary_of_line(line: &str) -> Option<RunSummary> {
    let body = line.strip_prefix("E7RESULT ").unwrap_or(line);
    if !body.starts_with("name=run ") {
        return None;
    }
    let field = |wanted: &str| -> &str {
        body.split(' ')
            .find_map(|token| token.strip_prefix(wanted).and_then(|rest| rest.strip_prefix('=')))
            .unwrap_or_else(|| panic!("run 行缺字段 {wanted}：{body}"))
    };
    let number = |wanted: &str| -> u64 { field(wanted).parse().unwrap_or_else(|_| panic!("{wanted} 不是数：{body}")) };
    let point = *SAMPLING_POINTS.iter().find(|point| point.name == field("point")).expect("认得的取样点");
    let arm = *FLOOR_ARMS.iter().find(|arm| arm.tag() == field("arm")).expect("认得的臂");
    let growth = [GrowthVerdict::Bounded, GrowthVerdict::Diverging, GrowthVerdict::Unreadable]
        .into_iter()
        .find(|verdict| verdict.tag() == field("deletion_block_growth"))
        .expect("认得的第 3 行判定");
    let key = RunKey {
        point,
        root_ring_slots_per_region: number("root_ring_slots_per_region"),
        commit_generated_blocks_per_publish: number("commit_generated_blocks_per_publish"),
        arm,
        is_compaction_enabled: field("compaction") == "on",
        seed: number("seed"),
    };
    let intent_errors = IntentErrorCounts {
        ended: number("intents_ended"),
        discarded: number("intents_discarded"),
        judged: number("intents_judged"),
        region_predicate_true_truth_false: number("region_is_record_empty_true_region_became_openable_false"),
        region_predicate_false_truth_true: number("region_is_record_empty_false_region_became_openable_true"),
        global_difference_true_truth_false: number("global_difference_is_positive_true_region_became_openable_false"),
        global_difference_false_truth_true: number("global_difference_is_positive_false_region_became_openable_true"),
        global_difference_true_openable_rise_false: number("global_difference_is_positive_true_openable_count_rose_false"),
        global_difference_false_openable_rise_true: number("global_difference_is_positive_false_openable_count_rose_true"),
    };
    Some(RunSummary { key, growth, deletion_failures: number("deletion_failures"), intent_errors })
}

fn header_lines() -> Vec<String> {
    let mut lines = vec![format!(
        "name=config segment=1 points={} root_ring_slots_per_region_choices=4/8/16 commit_generated_blocks_per_publish_choices=5/10/13 arms=segment_floor_one/segment_floor_two/segment_floor_formula/segment_floor_zero compaction=on/off seeds={SEEDS_PER_CELL} rounds={ROUNDS_FIRST_PASS}/{ROUNDS_DOUBLED_CHECK} units_per_deletion_publish={UNITS_PER_DELETION_PUBLISH} file_units=1..{FILE_UNITS_UPPER_BOUND} compaction_batch_units={COMPACTION_BATCH_UNITS} stop_arm=local_region_predicate selection=greedy unit_area_start={UNIT_AREA_START_SLOT} segment_slots={CLUSTER_SEGMENT_SLOTS} leaf_slots={ALLOCATION_RECORD_TREE_LEAF_SLOTS} fanout={ALLOCATION_RECORD_TREE_INTERNAL_FANOUT}",
        SAMPLING_POINTS.iter().map(|point| point.name).collect::<Vec<_>>().join("/")
    )];
    lines.extend(formula_lines());
    lines
}

/// 一个取样点（或全部）的第 7 行与全部正式跑的行，连同交给格一级的汇总。
fn measured_lines(selection: Option<ShardSelection>) -> (Vec<String>, Vec<RunSummary>) {
    let mut lines = metadata_stream_lines(selection);
    let keys = all_run_keys(selection);
    let mut summaries = Vec::new();
    for (run_lines, summary) in run_fill_delete_fill_runs_in_parallel(&keys, worker_thread_count()) {
        lines.extend(run_lines);
        summaries.push(summary);
    }
    (lines, summaries)
}

/// 读一份分片产物：末行必须是 `name=done emitted=N` 且 N 等于行数（作废条款 V3），交回去掉收尾行的那些行。
fn read_shard(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("读不了分片 {path}：{error}"));
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let done_line = lines.pop().expect("分片不空");
    assert_eq!(done_line, format!("E7RESULT name=done emitted={}", lines.len() + 1), "分片 {path} 的收尾行（V3）");
    lines
        .into_iter()
        .map(|line| line.strip_prefix("E7RESULT ").map(str::to_string).expect("分片的每一行都是 E7RESULT 行"))
        .collect()
}

/// 先打这一次准入的产物行（强制跑时一行 `name=preflight forced=…`，满足时不打），再打 `lines`，最后是收尾行。
fn print_with_emitter(preflight_lines: &[String], lines: &[String]) {
    let mut emitter = Emitter::new();
    for line in preflight_lines.iter().chain(lines) {
        println!("{}", emitter.emit_raw(line));
    }
    println!("{}", emitter.finish());
}

/// 开跑时判的准入：满足时带上开跑时的输入指纹（成功跑完记它）；`--force` 强制跑时带没满足的条件摘要（写进产物、不记成功）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum PreflightOutcome {
    Met { input_fingerprint_at_start: String },
    Forced { unmet_conditions_summary: String },
}

impl PreflightOutcome {
    /// 产物里记这一次准入的行：强制跑写一行，满足不写（满足时产物与没有准入这一层时逐字节相同）。
    fn product_lines(&self) -> Vec<String> {
        match self {
            PreflightOutcome::Met { .. } => Vec::new(),
            PreflightOutcome::Forced { unmet_conditions_summary } => {
                vec![format!("name=preflight forced={}", unmet_conditions_summary.split_whitespace().collect::<Vec<_>>().join("_"))]
            }
        }
    }
}

/// 准入判法的入口与本文件：都按编译时的清单目录取绝对路径（判法要的是源文件的路径，不是二进制的）。
fn preflight_script_and_source_paths() -> (String, String) {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    (
        format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py"),
        format!("{manifest_directory}/e10_aging_compaction.rs"),
    )
}

/// 准入判法退 78（拒绝）时装置也退 78；别的非 0 退 1（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）。
const PREFLIGHT_REFUSED_EXIT_CODE: i32 = 78;

/// `main` 的第一句：直接起 `python3 preflight.py check <本文件> [--force] -- <参数…>`（不经 sh -c）判准入与运行条件，
/// 不满足就照判法的退出码退出、一步都不做。
fn preflight() -> PreflightOutcome {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (script, source) = preflight_script_and_source_paths();
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE)
    });
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    match output.status.code() {
        Some(0) => {}
        Some(PREFLIGHT_REFUSED_EXIT_CODE) => std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE),
        _other_exit => std::process::exit(1),
    }
    let verdict_line = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").to_string();
    let last_segment = verdict_line.rsplit('\t').next().unwrap_or("").to_string();
    if verdict_line.starts_with("met") {
        PreflightOutcome::Met { input_fingerprint_at_start: last_segment }
    } else if verdict_line.starts_with("forced") {
        PreflightOutcome::Forced { unmet_conditions_summary: last_segment }
    } else {
        eprintln!("  ✗ 准入判法放行了，却没给 met 或 forced 那一行：「{verdict_line}」");
        eprintln!("  → 怎么办：查 e10_aging_compaction.rs 文件头的 admission 与 run-condition 两行写法");
        std::process::exit(1)
    }
}

/// 成功跑完、退出之前记下开跑时判的那份输入指纹（`preflight.py record`）；记不下只在 stderr 说为什么，不改这一次的退出码。
fn preflight_record_success(input_fingerprint_at_start: &str, arguments: &[String]) {
    let (script, source) = preflight_script_and_source_paths();
    let recorded = std::process::Command::new("python3")
        .arg(&script)
        .arg("record")
        .arg(&source)
        .arg("--fingerprint")
        .arg(input_fingerprint_at_start)
        .arg("--")
        .args(arguments)
        .status();
    match recorded {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("  ! 没记下这一次的输入指纹：preflight.py record 退 {status}"),
        Err(error) => eprintln!("  ! 没记下这一次的输入指纹：起不了 python3（{error}）"),
    }
}

/// 三种用法：不带参数跑全部并出全部判定（产物与复跑就是这一种）；`--point <取样点> [--root-ring-slots-per-region <S>]` 只跑一个取样点
/// （或其中一个 S）、出一份分片；`--merge <分片…>` 把按（取样点, S）次序给的分片拼成与不带参数逐字节相同的产物（正式跑的次序与条数逐条核）。
/// 三种都可再带 `--force`：准入不满足也照跑（产物开头记一行 `name=preflight forced=…`，不记成功）。
fn main() {
    let preflight_outcome = preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    let preflight_lines = preflight_outcome.product_lines();
    let started = std::time::Instant::now();
    let shard_of = |name: &str, root_ring_slots_per_region: Option<u64>| {
        let point = SAMPLING_POINTS.iter().find(|point| point.name == name).unwrap_or_else(|| panic!("不认得的取样点 {name}"));
        assert!(
            root_ring_slots_per_region.is_none_or(|wanted| ROOT_RING_SLOTS_PER_REGION_CHOICES.contains(&wanted)),
            "S 只取 4 / 8 / 16"
        );
        ShardSelection { point: point.name, root_ring_slots_per_region }
    };
    match arguments.as_slice() {
        [] => {
            let mut lines = header_lines();
            let (measured, summaries) = measured_lines(None);
            lines.extend(measured);
            lines.extend(cell_and_verdict_lines(&summaries, true));
            print_with_emitter(&preflight_lines, &lines);
        }
        [flag, name] if flag == "--point" => print_with_emitter(&preflight_lines, &measured_lines(Some(shard_of(name, None))).0),
        [point_flag, name, root_ring_slots_flag, root_ring_slots_per_region]
            if point_flag == "--point" && root_ring_slots_flag == "--root-ring-slots-per-region" =>
        {
            let root_ring_slots_per_region: u64 = root_ring_slots_per_region.parse().expect("S 是正整数");
            print_with_emitter(&preflight_lines, &measured_lines(Some(shard_of(name, Some(root_ring_slots_per_region)))).0);
        }
        [flag, shard_paths @ ..] if flag == "--merge" => {
            let shards: Vec<Vec<String>> = shard_paths.iter().map(|path| read_shard(path)).collect();
            let mut lines = header_lines();
            for shard in &shards {
                lines.extend(shard.iter().filter(|line| line.starts_with("name=metadata_stream ")).cloned());
            }
            let mut summaries = Vec::new();
            for shard in &shards {
                for line in shard.iter().filter(|line| !line.starts_with("name=metadata_stream ")) {
                    summaries.extend(run_summary_of_line(line));
                    lines.push(line.clone());
                }
            }
            let merged_keys: Vec<RunKey> = summaries.iter().map(|summary| summary.key).collect();
            assert_eq!(merged_keys, all_run_keys(None), "分片拼起来的正式跑与全跑逐条同序、一条不缺");
            assert_eq!(
                lines.iter().filter(|line| line.starts_with("name=metadata_stream ")).count(),
                metadata_stream_lines(None).len(),
                "第 7 行一格不缺"
            );
            lines.extend(cell_and_verdict_lines(&summaries, true));
            print_with_emitter(&preflight_lines, &lines);
        }
        other => panic!(
            "用法：e10-aging-compaction [--force] [--point <取样点> [--root-ring-slots-per-region <S>] | --merge <分片…>]，收到 {other:?}"
        ),
    }
    eprintln!("e10-aging-compaction {arguments:?}：{:.1} 秒", started.elapsed().as_secs_f64());
    if let PreflightOutcome::Met { input_fingerprint_at_start } = &preflight_outcome {
        preflight_record_success(input_fingerprint_at_start, &arguments);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with_floor(floor_segments: u64) -> PoolSettings {
        PoolSettings {
            floor_segments,
            is_compaction_enabled: true,
            stop_arm: StopArm::LocalRegionPredicate,
            resident_base_multiplier: 3,
        }
    }

    /// 两块盘、每块 50176 + 64 × segments 槽；叶宽 812 ⇒ 树高 2、b = 3。
    fn small_geometry(segments: u64, root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64) -> PoolGeometry {
        PoolGeometry::new(
            UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS * segments,
            ALLOCATION_RECORD_TREE_LEAF_SLOTS,
            root_ring_slots_per_region,
            commit_generated_blocks_per_publish,
        )
    }

    fn four_gibibyte_geometry(root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64) -> PoolGeometry {
        PoolGeometry::new(
            FOUR_GIBIBYTE_DISK_SLOTS,
            ALLOCATION_RECORD_TREE_LEAF_SLOTS,
            root_ring_slots_per_region,
            commit_generated_blocks_per_publish,
        )
    }

    /// 在事务外直接摆一个用户数据单元（单测构造状态用）。
    fn place_user_unit(pool: &mut Pool, even_offset: u64, birth_txg: u64) -> u32 {
        pool.begin_transaction();
        let unit = pool.create_unit(even_offset, birth_txg);
        pool.allocate_slot(even_offset, SlotOwner::UserDataUnit { unit }, birth_txg);
        pool.allocate_slot(even_offset + 1, SlotOwner::UserDataUnit { unit }, birth_txg);
        pool.commit_transaction();
        unit
    }

    /// 在事务外直接摆一个占槽的提交内生块记录（只为让段有活槽，不进块位置表）。
    fn place_live_commit_generated_slot(pool: &mut Pool, offset: u64) {
        let block = BlockIdentity::FileTreeNode { node: 1_000_000 + offset };
        pool.set_slot_state(offset, SlotState::Live { owner: SlotOwner::CommitGeneratedBlock { block }, allocation_txg: 0 });
    }

    #[test]
    fn formula_values_in_the_nine_cells_match_the_values_registered_before_the_run() {
        assert_eq!(formula_floor_segments(4, 5), 1 + 1);
        assert_eq!(formula_floor_segments(4, 10), 2 + 1);
        assert_eq!(formula_floor_segments(4, 13), 3 + 1);
        assert_eq!(formula_floor_segments(8, 5), 2 + 1);
        assert_eq!(formula_floor_segments(8, 10), 4 + 1);
        assert_eq!(formula_floor_segments(8, 13), 5 + 1);
        assert_eq!(formula_floor_segments(16, 5), 4 + 1);
        assert_eq!(formula_floor_segments(16, 10), 8 + 1);
        assert_eq!(formula_floor_segments(16, 13), 10 + 1);
        for (root_ring_slots_per_region, commit_generated_blocks_per_publish, registered) in FORMULA_VALUES_REGISTERED_BEFORE_THE_RUN {
            assert_eq!(formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish), registered);
        }
        assert!(formula_lines().iter().all(|line| line.ends_with("matches_registered=true")));
    }

    #[test]
    fn ring_slots_minus_one_times_commit_generated_blocks_in_the_nine_cells() {
        let products: Vec<u64> = FORMULA_VALUES_REGISTERED_BEFORE_THE_RUN
            .iter()
            .map(|(root_ring_slots_per_region, commit_generated_blocks_per_publish, _registered)| {
                (ROOT_RING_REGIONS * root_ring_slots_per_region - 1) * commit_generated_blocks_per_publish
            })
            .collect();
        assert_eq!(products, vec![55, 110, 143, 115, 230, 299, 235, 470, 611]);
    }

    #[test]
    fn compaction_call_budget_matches_the_judgement_values() {
        assert_eq!(compaction_call_publish_budget(4, 0), 4 + 4);
        assert_eq!(compaction_call_publish_budget(8, 0), 16 + 4);
        assert_eq!(compaction_call_publish_budget(16, 0), 40 + 4);
        assert_eq!(compaction_call_publish_budget(4, 2), 2 + 4);
        assert_eq!(compaction_call_publish_budget(8, 2), 14 + 4);
        assert_eq!(compaction_call_publish_budget(16, 2), 38 + 4);
        for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
            let publishes = 3 * root_ring_slots_per_region - 4;
            assert_eq!(compaction_call_publish_budget(root_ring_slots_per_region, publishes), 1);
            assert_eq!(compaction_call_publish_budget(root_ring_slots_per_region, publishes + 7), 1);
        }
    }

    #[test]
    fn geometry_of_the_two_sampling_disk_sizes() {
        let four = four_gibibyte_geometry(8, 5);
        assert_eq!((four.unit_area_slots(), four.segment_count()), (211_968, 3_312));
        assert_eq!(four.leaf_of_absolute_slot(UNIT_AREA_START_SLOT), 61);
        assert_eq!(four.leaf_of_absolute_slot(FOUR_GIBIBYTE_DISK_SLOTS - 1), 322);
        assert_eq!(four.leaves_covering_the_unit_area(), 262);
        assert_eq!((four.record_tree_root_level(), four.record_tree_blocks_per_publish_at_one_leaf()), (2, 5));
        let one = PoolGeometry::new(ONE_GIBIBYTE_DISK_SLOTS, ALLOCATION_RECORD_TREE_LEAF_SLOTS, 8, 5);
        assert_eq!((one.unit_area_slots(), one.segment_count()), (15_360, 240));
        assert_eq!(one.leaf_of_absolute_slot(ONE_GIBIBYTE_DISK_SLOTS - 1), 80);
        assert_eq!(one.leaves_covering_the_unit_area(), 20);
        assert_eq!((one.record_tree_root_level(), one.record_tree_blocks_per_publish_at_one_leaf()), (1, 3));
        assert_eq!(four.structural_commit_generated_block_bound(), 2 * 262 + 2 * 2 + 1);
        assert_eq!(one.structural_commit_generated_block_bound(), 2 + 2 * 20 + 1);
    }

    #[test]
    fn dirty_slots_across_the_level_one_boundary_write_four_more_blocks() {
        let geometry = four_gibibyte_geometry(8, 10);
        assert_eq!(geometry.level_one_width_in_slots(), 137_228);
        assert_eq!((geometry.leaf_of_absolute_slot(136_416), geometry.leaf_of_absolute_slot(137_227)), (168, 168));
        assert_eq!(geometry.leaf_of_absolute_slot(137_228), 169);
        assert_eq!(commit_generated_block_count_for_dirty_slots(&geometry, &[136_416, 137_228]), 10 + 4);
        assert_eq!(commit_generated_block_count_for_dirty_slots(&geometry, &[136_416, 136_418]), 10);
    }

    #[test]
    fn released_slot_is_reclaimable_ring_slots_minus_one_publishes_after_its_release() {
        for (root_ring_slots_per_region, last_held, first_reclaimable) in [(4, 10, 11), (8, 22, 23), (16, 46, 47)] {
            let geometry = four_gibibyte_geometry(root_ring_slots_per_region, 5);
            let release_txg = 1_000;
            assert!(release_txg > reclaim_threshold_at_publish(release_txg + last_held, 0, &geometry));
            assert!(release_txg <= reclaim_threshold_at_publish(release_txg + first_reclaimable, 0, &geometry));
        }
    }

    #[test]
    fn new_pool_on_two_four_gibibyte_disks_writes_five_record_tree_blocks_from_slot_50240() {
        let pool = Pool::made_by_make_filesystem(four_gibibyte_geometry(8, 5), settings_with_floor(1));
        assert_eq!(pool.geometry.record_tree_blocks_per_publish_at_one_leaf(), 5);
        let mut block_slots: Vec<u64> = pool
            .leaf_slots
            .iter()
            .chain(pool.level_one_slots.iter())
            .chain(std::iter::once(&pool.root_slot))
            .flatten()
            .map(|offset| offset + UNIT_AREA_START_SLOT)
            .collect();
        block_slots.sort_unstable();
        assert_eq!(block_slots, vec![50_240, 50_241, 50_242, 50_243, 50_244]);
        assert_eq!(pool.slot_state(0).live_owner(), Some(SlotOwner::InstanceTable));
        assert_eq!(pool.slot_state(1).live_owner(), Some(SlotOwner::InstanceTable));
        assert_eq!(pool.segments[1].live_slots, 5);
        let with_file_tree = Pool::made_by_make_filesystem(four_gibibyte_geometry(8, 10), settings_with_floor(1));
        assert_eq!(with_file_tree.segments[1].live_slots, 10);
        assert_eq!(with_file_tree.txg, 0);
        assert!(with_file_tree.open_cursor.is_none(), "mkfs 之后挂载一次，游标作废");
    }

    /// 段 2–7 各一个用户数据单元：全空段数（记录口径）是 0，空闲槽还多。
    fn pool_without_record_empty_segments(floor_segments: u64) -> Pool {
        let mut pool = Pool::made_by_make_filesystem(small_geometry(8, 4, 3), settings_with_floor(floor_segments));
        for segment in 2..8 {
            place_user_unit(&mut pool, segment * CLUSTER_SEGMENT_SLOTS, 0);
        }
        assert_eq!(pool.record_empty_segment_count, 0);
        pool
    }

    #[test]
    fn conjunct_seven_refuses_only_foreground_writes_when_no_segment_is_record_empty() {
        let mut writing = pool_without_record_empty_segments(1);
        assert_eq!(writing.publish_new_file(1), Err(PublishRefusal::ConjunctSeven));
        let mut deleting = pool_without_record_empty_segments(1);
        let unit = match deleting.slot_state(2 * CLUSTER_SEGMENT_SLOTS).live_owner() {
            Some(SlotOwner::UserDataUnit { unit }) => unit,
            other => panic!("段 2 起点是用户数据单元：{other:?}"),
        };
        assert!(deleting.publish_head_deletion(&[unit]).is_ok(), "删除类发布不受合取 7 管");
        let mut emptying = pool_without_record_empty_segments(1);
        assert!(emptying.publish_empty().is_ok(), "空发布不受合取 7 管");
        let mut compacting = pool_without_record_empty_segments(1);
        let region = compacting.choose_region_greedy().expect("有可整理的段");
        assert!(
            matches!(compacting.publish_compaction_batch(Some(region)), Ok(BatchOutcome::Published { .. })),
            "整理批不受合取 7 管"
        );
    }

    #[test]
    fn runs_count_released_slots_as_occupied() {
        let mut pool = Pool::blank(small_geometry(2, 4, 3), settings_with_floor(1));
        for offset in (0..2).chain(6..CLUSTER_SEGMENT_SLOTS).chain(CLUSTER_SEGMENT_SLOTS..2 * CLUSTER_SEGMENT_SLOTS) {
            place_live_commit_generated_slot(&mut pool, offset);
        }
        place_live_commit_generated_slot(&mut pool, 4);
        pool.set_slot_state(4, SlotState::Released { release_txg: 1 });
        assert_eq!(pool.free_runs, 2, "段 A 的 2–3 与 5 两段空闲；4 已释放未回收不算空闲");
        pool.verify_incremental_counters_against_a_full_scan();
    }

    #[test]
    fn segment_of_released_slots_is_record_empty_but_neither_fully_free_nor_openable() {
        let mut pool = Pool::blank(small_geometry(2, 4, 3), settings_with_floor(1));
        for offset in 0..2 * CLUSTER_SEGMENT_SLOTS {
            place_live_commit_generated_slot(&mut pool, offset);
        }
        for offset in CLUSTER_SEGMENT_SLOTS..2 * CLUSTER_SEGMENT_SLOTS {
            pool.set_slot_state(offset, SlotState::Released { release_txg: 1 });
        }
        assert_eq!(pool.record_empty_segment_count, 1);
        assert_eq!(pool.fully_free_segments.len(), 0);
        assert_eq!(pool.openable_segment_count(), 0);
        assert_eq!(pool.free_runs, 0);
        assert!(pool.conjunct_seven_admits(), "合取 7 读记录口径：1 ≥ N₀ = 1");
        pool.verify_incremental_counters_against_a_full_scan();
    }

    #[test]
    fn floor_raise_targets_the_fourth_newest_non_empty_root() {
        let ring: VecDeque<u64> = [100, 103, 104, 110, 111].into_iter().collect();
        assert_eq!(floor_raise_target(&ring, 70), 100 + 3);
        let short_ring: VecDeque<u64> = [100, 103, 104].into_iter().collect();
        assert_eq!(floor_raise_target(&short_ring, 70), 70, "非空根不足 4 个取环里最旧有效根");
    }

    #[test]
    fn raised_floor_takes_effect_only_after_the_second_empty_publish() {
        let mut pool = Pool::blank(small_geometry(4, 16, 3), settings_with_floor(0));
        pool.txg = 112;
        pool.non_empty_root_txgs_in_ring = [100, 103, 104, 110, 111].into_iter().collect();
        let held_offset = 3 * CLUSTER_SEGMENT_SLOTS;
        place_live_commit_generated_slot(&mut pool, held_offset);
        pool.set_slot_state(held_offset, SlotState::Released { release_txg: 103 });
        pool.reclaim_queue.push_back((103, held_offset));
        pool.pending_floor_raise = Some((103, 0));
        pool.publish_empty().expect("第 1 次空发布");
        assert_eq!(pool.effective_floor_txg, 0);
        pool.reclaim_for_the_next_publish();
        assert!(pool.slot_state(held_offset).is_released(), "第 1 次之后还不可回收");
        pool.publish_empty().expect("第 2 次空发布");
        assert_eq!(pool.effective_floor_txg, 103);
        pool.reclaim_for_the_next_publish();
        assert!(pool.slot_state(held_offset).is_free(), "第 2 次之后释放代 ≤ 103 的槽可回收");
    }

    #[test]
    fn raise_floor_publishes_two_empty_publishes_up_to_the_fourth_newest_non_empty_root() {
        let mut pool = Pool::blank(small_geometry(4, 16, 3), settings_with_floor(0));
        pool.txg = 112;
        pool.non_empty_root_txgs_in_ring = [100, 103, 104, 110, 111].into_iter().collect();
        pool.raise_floor();
        assert_eq!((pool.txg, pool.effective_floor_txg), (112 + 2, 103));
        pool.raise_floor();
        assert_eq!(pool.txg, 114, "F 已不低于目标就跳过");
    }

    #[test]
    fn greedy_breaks_ties_by_the_lowest_segment() {
        let mut pool = Pool::blank(small_geometry(6, 4, 3), settings_with_floor(1));
        for (segment, live_slots) in [(1, 5), (2, 3), (3, 3), (4, 7)] {
            for slot_in_segment in 0..live_slots {
                place_live_commit_generated_slot(&mut pool, segment * CLUSTER_SEGMENT_SLOTS + slot_in_segment);
            }
        }
        assert_eq!(pool.choose_region_greedy(), Some(2));
    }

    #[test]
    fn greedy_skips_segments_opened_in_this_mount() {
        let mut pool = Pool::blank(small_geometry(4, 4, 3), settings_with_floor(1));
        place_live_commit_generated_slot(&mut pool, CLUSTER_SEGMENT_SLOTS);
        for slot_in_segment in 0..5 {
            place_live_commit_generated_slot(&mut pool, 2 * CLUSTER_SEGMENT_SLOTS + slot_in_segment);
        }
        pool.open_segment(1);
        assert_eq!(pool.choose_region_greedy(), Some(2), "段 1 活槽最少，但这次挂载开过");
    }

    /// R = 段 0 有 3 个用户数据单元；段 1 有用户数据单元与 `holes_in_destination` 个空闲两槽；段 2 只装提交内生块、有 10 个空闲两槽。
    fn pool_for_the_pre_issue_check(holes_in_destination: u64) -> Pool {
        let mut pool = Pool::blank(small_geometry(4, 4, 3), settings_with_floor(1));
        for unit_index in 0..3 {
            place_user_unit(&mut pool, 2 * unit_index, 0);
        }
        for offset in 6..CLUSTER_SEGMENT_SLOTS {
            place_live_commit_generated_slot(&mut pool, offset);
        }
        let destination_start = CLUSTER_SEGMENT_SLOTS;
        place_user_unit(&mut pool, destination_start, 0);
        for offset in destination_start + 2..2 * CLUSTER_SEGMENT_SLOTS - 2 * holes_in_destination {
            place_live_commit_generated_slot(&mut pool, offset);
        }
        for offset in 2 * CLUSTER_SEGMENT_SLOTS..3 * CLUSTER_SEGMENT_SLOTS - 2 * 10 {
            place_live_commit_generated_slot(&mut pool, offset);
        }
        for offset in 3 * CLUSTER_SEGMENT_SLOTS..4 * CLUSTER_SEGMENT_SLOTS {
            place_live_commit_generated_slot(&mut pool, offset);
        }
        assert_eq!(pool.segments[0].live_user_units, 3);
        assert_eq!((pool.segments[1].free_pairs, pool.segments[2].free_pairs), (holes_in_destination, 10));
        pool
    }

    #[test]
    fn pre_issue_check_counts_only_holes_in_destination_segments() {
        assert!(!pool_for_the_pre_issue_check(2).pre_issue_check_passes(0), "合格段 2 个两槽 < 3 个单元");
        assert!(pool_for_the_pre_issue_check(3).pre_issue_check_passes(0), "合格段 3 个两槽 ≥ 3 个单元");
    }

    #[test]
    fn the_fixpoint_rewrites_the_leaf_that_the_new_blocks_land_in() {
        let mut pool = Pool::blank(four_gibibyte_geometry(4, 5), settings_with_floor(0));
        // 叶 62 从绝对槽 50344（偏移 168）起，在段 2（偏移 128–191）里；游标停在 50341，这次的 5 块落在 50341–50345。
        pool.open_segment(2);
        pool.open_cursor = Some(OpenSegmentCursor { segment: 2, next_offset_in_segment: 50_341 - UNIT_AREA_START_SLOT - 128 });
        pool.publish_new_file(1).expect("空池上写一个单元");
        assert_eq!(pool.units[0].first_slot_offset, 0, "用户数据落在单元区起点（叶 61）");
        let written: Vec<u64> = (128..CLUSTER_SEGMENT_SLOTS * 3).filter(|offset| pool.slot_state(*offset).is_live()).collect();
        assert_eq!(written.len(), 5 + 2, "叶 61、层级 1 各两块与根落到 50341–50345，其中两块进了叶 62 ⇒ 叶 62 两块");
        assert!(pool.leaf_slots.iter().flatten().count() == 4, "叶 61 与叶 62 各两块盘一份");
    }

    #[test]
    fn user_data_never_lands_in_the_open_segment() {
        let mut pool = Pool::blank(small_geometry(4, 4, 3), settings_with_floor(0));
        pool.open_segment(0);
        let units = pool.publish_new_file(1).expect("写一个单元");
        assert_eq!(pool.units[usize_of(u64::from(units[0]))].first_slot_offset, CLUSTER_SEGMENT_SLOTS);
    }

    #[test]
    fn user_data_never_lands_in_the_live_intent_region() {
        let mut pool = Pool::blank(small_geometry(4, 4, 3), settings_with_floor(0));
        pool.active_intent = Some(CompactionIntent {
            region_segment: 0,
            allocation_txg_at_issue: 0,
            record_empty_segments_at_issue: 0,
            openable_segments_at_issue: 0,
        });
        let units = pool.publish_new_file(1).expect("写一个单元");
        assert_eq!(pool.units[usize_of(u64::from(units[0]))].first_slot_offset, CLUSTER_SEGMENT_SLOTS);
        assert!((0..CLUSTER_SEGMENT_SLOTS).all(|offset| pool.slot_state(offset).is_free()), "R 里一块提交内生块都没落");
    }

    #[test]
    fn unit_pinned_by_some_snapshot_keeps_its_slots_when_the_head_deletes_it() {
        let mut pool = Pool::made_by_make_filesystem(small_geometry(8, 4, 5), settings_with_floor(0));
        let units = pool.publish_new_file(1).expect("写一个单元");
        let (pinned_units, pinned_file_tree_offsets) = pool.publish_snapshot_creation().expect("建快照");
        assert_eq!((pinned_units.clone(), pinned_file_tree_offsets.len()), (units.clone(), 2), "c − b = 2 个文件树节点被钉");
        pool.publish_head_deletion(&units).expect("头一侧删掉");
        let first_slot_offset = pool.units[usize_of(u64::from(units[0]))].first_slot_offset;
        assert!(pool.slot_state(first_slot_offset).is_live(), "快照钉着，不释放");
        assert_eq!(pool.snapshot_only_unit_total, 1);
        pool.publish_snapshot_destruction_batch(&pinned_units, &pinned_file_tree_offsets).expect("销毁快照");
        assert!(pool.slot_state(first_slot_offset).is_released(), "最后一个快照销毁时释放");
        assert!(pinned_file_tree_offsets.iter().all(|offset| pool.slot_state(*offset).is_released()));
        assert_eq!(pool.snapshot_only_unit_total, 0);
        pool.verify_incremental_counters_against_a_full_scan();
    }

    /// 段 0 实例表、段 1 mkfs 的块；R 候选段 2 有两个用户数据单元；段 3 有一个用户数据单元（目的地）；段 4–9 全空。
    fn pool_with_a_two_unit_region(stop_arm: StopArm) -> Pool {
        let settings = PoolSettings { floor_segments: 0, is_compaction_enabled: true, stop_arm, resident_base_multiplier: 3 };
        let mut pool = Pool::made_by_make_filesystem(small_geometry(10, 4, 3), settings);
        place_user_unit(&mut pool, 2 * CLUSTER_SEGMENT_SLOTS, 0);
        place_user_unit(&mut pool, 2 * CLUSTER_SEGMENT_SLOTS + 2, 0);
        place_user_unit(&mut pool, 3 * CLUSTER_SEGMENT_SLOTS, 0);
        pool
    }

    fn published_without_ending(outcome: Result<BatchOutcome, PublishRefusal>) {
        assert_eq!(outcome, Ok(BatchOutcome::Published { ended_intent_index: None }), "第一批搬一个、意图还在");
    }

    #[test]
    fn opening_an_empty_segment_during_the_intent_makes_the_global_difference_false_while_the_region_truly_empties() {
        for stop_arm in [StopArm::LocalRegionPredicate, StopArm::GlobalDifference] {
            let mut pool = pool_with_a_two_unit_region(stop_arm);
            published_without_ending(pool.publish_compaction_batch(Some(2)));
            // 第一批的块落在段 4（开放段）；摆一条不会被重写的记录让它不因块搬走而变回全空，再把游标用尽，下一次写只能开段 5。
            place_live_commit_generated_slot(&mut pool, 4 * CLUSTER_SEGMENT_SLOTS + 40);
            pool.open_cursor = Some(OpenSegmentCursor { segment: 4, next_offset_in_segment: CLUSTER_SEGMENT_SLOTS });
            let record_empty_before_the_write = pool.record_empty_segment_count;
            pool.publish_new_file(1).expect("前台写一个单元");
            assert_eq!(pool.record_empty_segment_count + 1, record_empty_before_the_write, "这次写开掉了一个全空段");
            let outcome = pool.publish_compaction_batch(None).expect("第二批");
            assert_eq!(outcome, BatchOutcome::Published { ended_intent_index: Some(0) });
            let ended = pool.ended_intents[0];
            assert!(ended.is_completed && ended.region_is_record_empty, "{stop_arm:?}：P_R 真");
            assert!(!ended.global_difference_is_positive, "{stop_arm:?}：P_G 假（腾出 R 与开掉一段相抵）");
            for _empty_publish in 0..pool.geometry.retention_publishes() {
                pool.publish_empty().expect("空发布");
            }
            assert_eq!(pool.ended_intents[0].region_became_openable, Some(true), "{stop_arm:?}：Y1 真");
        }
    }

    #[test]
    fn pinned_slot_put_into_the_region_after_issue_makes_the_intent_discarded_with_the_region_not_record_empty() {
        for stop_arm in [StopArm::LocalRegionPredicate, StopArm::GlobalDifference] {
            let mut pool = pool_with_a_two_unit_region(stop_arm);
            published_without_ending(pool.publish_compaction_batch(Some(2)));
            let pinned_offset = 2 * CLUSTER_SEGMENT_SLOTS + 12;
            pool.set_slot_state(pinned_offset, SlotState::Live { owner: SlotOwner::InstanceTable, allocation_txg: 0 });
            let outcome = pool.publish_compaction_batch(None).expect("第二批");
            assert_eq!(outcome, BatchOutcome::Published { ended_intent_index: Some(0) });
            let ended = pool.ended_intents[0];
            assert!(!ended.is_completed, "{stop_arm:?}：只剩钉住的 ⇒ 丢意图");
            assert!(!ended.region_is_record_empty, "{stop_arm:?}：P_R 假");
        }
    }

    #[test]
    fn local_region_predicate_signs_no_next_intent_after_one_without_global_output() {
        let mut pool = pool_without_record_empty_segments(0);
        assert_eq!(pool.run_compaction(4), 1, "第一条意图在签发那一批里就收，P_G 假 ⇒ 不签下一条");
        assert_eq!(pool.ended_intents.len(), 1);
        assert!(!pool.ended_intents[0].global_difference_is_positive);
    }

    fn rows_with_deletion_blocks(per_round_publish_blocks: &[Vec<u64>]) -> Vec<RoundRow> {
        per_round_publish_blocks
            .iter()
            .enumerate()
            .map(|(round_index, blocks)| {
                let mut metrics = RoundMetrics::starting_at(0, 0, 0);
                metrics.deletion_publishes = blocks.len() as u64;
                metrics.deletion_commit_generated_block_sum = blocks.iter().sum();
                metrics.deletion_commit_generated_block_peak = blocks.iter().copied().max().unwrap_or(0);
                RoundRow {
                    round: round_index as u64 + 1,
                    metrics,
                    free_runs: 0,
                    live_user_units: 0,
                    unrecoverable_slots: 0,
                    record_empty_segments_at_end: 0,
                }
            })
            .collect()
    }

    #[test]
    fn growth_judgement_compares_rounds_41_to_60_with_rounds_21_to_40() {
        let low_then_flat: Vec<Vec<u64>> = (1..=60).map(|round| vec![if round <= 20 { 10 } else { 20 }]).collect();
        assert_eq!(
            judge_deletion_block_growth(&rows_with_deletion_blocks(&low_then_flat), 1_000, &GROWTH_THRESHOLDS),
            GrowthVerdict::Bounded
        );
        let rising: Vec<Vec<u64>> = (1..=60).map(|round| vec![round]).collect();
        assert_eq!(
            judge_deletion_block_growth(&rows_with_deletion_blocks(&rising), 1_000, &GROWTH_THRESHOLDS),
            GrowthVerdict::Diverging
        );
        let silent_late: Vec<Vec<u64>> = (1..=60).map(|round| if round > 40 { Vec::new() } else { vec![5] }).collect();
        assert_eq!(
            judge_deletion_block_growth(&rows_with_deletion_blocks(&silent_late), 1_000, &GROWTH_THRESHOLDS),
            GrowthVerdict::Unreadable
        );
    }

    #[test]
    fn moving_the_growth_thresholds_between_two_measured_values_flips_bounded_to_diverging() {
        // 21–40 轮每轮均值 150、41–60 轮 154.5（涨 3%）；峰值都是 200。
        let rounds: Vec<Vec<u64>> = (1..=60).map(|round| if round <= 40 { vec![200, 100] } else { vec![200, 109] }).collect();
        let rows = rows_with_deletion_blocks(&rounds);
        let loosened = GrowthThresholds { mean_growth_percent: 105, structural_bound_tenths: 9 };
        let between = GrowthThresholds { mean_growth_percent: 102, structural_bound_tenths: 9 };
        assert_eq!(judge_deletion_block_growth(&rows, 1_000, &loosened), GrowthVerdict::Bounded);
        assert_eq!(judge_deletion_block_growth(&rows, 1_000, &between), GrowthVerdict::Diverging);
        // 峰值 200：10 × 200 < 9 × 223 成立、< 9 × 222 不成立。
        assert_eq!(judge_deletion_block_growth(&rows, 223, &loosened), GrowthVerdict::Bounded);
        assert_eq!(judge_deletion_block_growth(&rows, 222, &loosened), GrowthVerdict::Diverging);
    }

    #[test]
    fn the_doubled_check_turns_a_late_rise_into_diverging() {
        let late_rise: Vec<Vec<u64>> = (1..=120).map(|round| vec![if round > 100 { 30 } else { 20 }]).collect();
        assert_eq!(judge_doubled_growth_check(&rows_with_deletion_blocks(&late_rise)), GrowthVerdict::Diverging);
        let flat: Vec<Vec<u64>> = (1..=120).map(|_round| vec![20]).collect();
        assert_eq!(judge_doubled_growth_check(&rows_with_deletion_blocks(&flat)), GrowthVerdict::Bounded);
    }

    #[test]
    fn five_seeds_combine_to_their_common_value_or_unstable() {
        assert_eq!(combine_seed_tags(&["bounded"; 5]), "bounded");
        assert_eq!(combine_seed_tags(&["bounded", "bounded", "diverging", "bounded", "bounded"]), "unstable");
        assert_eq!(across_sampling_points(&["none"; 7]), "none");
        assert_eq!(across_sampling_points(&["none", "none", "none", "failure", "none", "none", "none"]), "unstable");
    }

    #[test]
    fn candidate_passes_one_cell_only_when_bounded_without_failures_and_the_control_degrades() {
        assert_eq!(candidate_cell_verdict(("bounded", "none"), true, false), "pass");
        assert_eq!(candidate_cell_verdict(("bounded", "none"), false, false), "not_judgeable", "F1：对照不退化");
        assert_eq!(candidate_cell_verdict(("bounded", "none"), true, true), "not_judgeable", "F3：打中不分辨臂");
        assert_eq!(candidate_cell_verdict(("diverging", "none"), true, false), "fail");
        assert_eq!(candidate_cell_verdict(("bounded", "failure"), false, false), "fail");
        assert_eq!(candidate_cell_verdict(("unstable", "none"), true, false), "not_judgeable");
        assert_eq!(candidate_cell_verdict(("unreadable", "none"), true, false), "not_judgeable");
        let degrading = CellArmVerdict { growth: "bounded", failures: "failure", growth_seeds: String::new(), failure_seeds: String::new() };
        let steady = CellArmVerdict { growth: "bounded", failures: "none", growth_seeds: String::new(), failure_seeds: String::new() };
        assert!(off_control_degrades(&degrading));
        assert!(!off_control_degrades(&steady));
    }

    #[test]
    fn every_reverse_point_changes_exactly_one_knob_of_the_main_point() {
        for point in &SAMPLING_POINTS[1..] {
            let differing_knobs = [
                point.disk_slots != MAIN_SAMPLING_POINT.disk_slots,
                point.leaf_width_in_slots != MAIN_SAMPLING_POINT.leaf_width_in_slots,
                point.remounts_every_round != MAIN_SAMPLING_POINT.remounts_every_round,
                point.deletion_mode != MAIN_SAMPLING_POINT.deletion_mode,
                point.snapshot_limit != MAIN_SAMPLING_POINT.snapshot_limit,
                point.resident_base_multiplier != MAIN_SAMPLING_POINT.resident_base_multiplier,
            ];
            assert_eq!(differing_knobs.iter().filter(|differs| **differs).count(), 1, "{}", point.name);
        }
        assert_eq!(all_run_keys(None).len(), 7 * 9 * 4 * 2 * 5);
    }

    #[test]
    fn publish_refused_after_rehearsal_leaves_every_counter_as_it_was() {
        let settings = PoolSettings { floor_segments: 0, is_compaction_enabled: true, stop_arm: StopArm::LocalRegionPredicate, resident_base_multiplier: 1_000 };
        let mut pool = Pool::made_by_make_filesystem(small_geometry(8, 4, 3), settings);
        pool.reclaim_for_the_next_publish();
        let slot_states_before = pool.slot_states.clone();
        let segments_before = pool.segments.clone();
        let sets_before = (pool.user_placement_segments.clone(), pool.fully_free_segments.clone());
        let counts_before = (pool.txg, pool.free_runs, pool.record_empty_segment_count, pool.units.len(), pool.reclaim_queue.len());
        let positions_before = (pool.leaf_slots.clone(), pool.root_slot, pool.open_cursor);
        assert_eq!(pool.publish_new_file(3), Err(PublishRefusal::ConjunctOne));
        assert_eq!(pool.slot_states, slot_states_before);
        assert_eq!(pool.segments, segments_before);
        assert_eq!((pool.user_placement_segments.clone(), pool.fully_free_segments.clone()), sets_before);
        assert_eq!((pool.txg, pool.free_runs, pool.record_empty_segment_count, pool.units.len(), pool.reclaim_queue.len()), counts_before);
        assert_eq!((pool.leaf_slots.clone(), pool.root_slot, pool.open_cursor), positions_before);
        pool.verify_incremental_counters_against_a_full_scan();
    }

    fn short_run_key(point: SamplingPoint, is_compaction_enabled: bool) -> RunKey {
        RunKey {
            point,
            root_ring_slots_per_region: 4,
            commit_generated_blocks_per_publish: 5,
            arm: FloorArm::One,
            is_compaction_enabled,
            seed: 3,
        }
    }

    /// 1 GiB 上把快照、按单元删、不重挂一起开着跑几轮：每轮末的全扫自检（V4）都过，同一种子两遍逐行相同（V1 的单测形态）。
    #[test]
    fn short_runs_keep_the_counters_consistent_and_repeat_exactly() {
        let exercising_point = SamplingPoint {
            name: "test_one_gibibyte_snapshots_units_no_remount",
            disk_slots: ONE_GIBIBYTE_DISK_SLOTS,
            leaf_width_in_slots: ALLOCATION_RECORD_TREE_LEAF_SLOTS,
            remounts_every_round: false,
            deletion_mode: DeletionMode::RandomUnits,
            snapshot_limit: 2,
            resident_base_multiplier: 3,
        };
        let one_gibibyte_main = SAMPLING_POINTS[1];
        assert_eq!(one_gibibyte_main.name, "disk_1gib");
        for (point, is_compaction_enabled) in [(exercising_point, true), (exercising_point, false), (one_gibibyte_main, true)] {
            let mut rows_of_two_runs = Vec::new();
            for _repetition in 0..2 {
                let mut run = FillDeleteFillRun::new(short_run_key(point, is_compaction_enabled));
                for round in 1..=6 {
                    run.run_round(round);
                }
                assert!(run.rows.iter().any(|row| row.metrics.deletion_publishes > 0), "删段真的发了删除类发布");
                assert!(run.rows[0].metrics.foreground_writes > 0, "第一轮的填满段写了");
                rows_of_two_runs.push((run.rows, run.pool.ended_intents));
            }
            assert_eq!(rows_of_two_runs[0], rows_of_two_runs[1], "{} 同一种子两遍逐行相同", point.name);
        }
    }

    /// `--merge` 从分片的 run 行取回的汇总，与跑的时候交给格一级的那一份逐项相同。
    #[test]
    fn run_summary_read_back_from_its_line_equals_the_one_handed_to_the_cells() {
        let key = RunKey {
            point: SAMPLING_POINTS[4],
            root_ring_slots_per_region: 8,
            commit_generated_blocks_per_publish: 10,
            arm: FloorArm::Formula,
            is_compaction_enabled: true,
            seed: 2,
        };
        let mut run = FillDeleteFillRun::new(key);
        for round in 1..=3 {
            run.run_round(round);
        }
        let finished = FinishedRun { key, rows: run.rows, ended_intents: run.pool.ended_intents, growth: GrowthVerdict::Unreadable };
        let (lines, summary) = lines_of_finished_run(&finished);
        let read_back: Vec<RunSummary> = lines.iter().filter_map(|line| run_summary_of_line(line)).collect();
        assert_eq!(read_back, vec![summary]);
        assert!(summary.intent_errors.ended > 0 || summary.deletion_failures == 0);
    }
}
