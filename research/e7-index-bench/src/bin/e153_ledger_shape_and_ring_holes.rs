//! E153：账本形态与环上有洞的代价 —— 记账第 1 项与可再分配谓词两种形态、各配两种影子账（四条臂）
//! 与真实基线「今」的确定性计数模型。
//!
//! 判据、失败条款、变异表的权威登记在 research/prompts/e153-preregistration.md（跑前写死 2026-09-17）。
//! 本文件只实现登记里定义的模型；不与 crates/ 共用代码（.claude/rules/implementation-first.md 第 4 条）。
//!
//! ## 骨架（对应登记第五节 M0）
//!
//! 两块盘 Disk0/Disk1，一个单元两盘同槽；根环 R=3 区域、区域设备归属 [Disk0,Disk1,Disk0]、槽序跨区轮转；
//! 每区槽数 S（几何旋钮）、N=3S。九个「角色」构成一次发布触碰的对象：实例表单元、树表单元、
//! 中央映射树、extent 根、inode 根、分配记录树、记账树、数据单元、inode 叶容器；
//! O 触碰其中 8 个（放 10 槽）、E 触碰其中 4 个（放 4 槽）、W = E 的 4 个加实例表单元（放 6 槽）。
//!
//! 记账第 1/5 项对「甲-T1」「G12」两条主谓词按登记第五节的「环′」公式现算（与物理是否已回收无关）；
//! 第 2 项走独立的事件路径（分配减、记录由不可再分配变可再分配时加）。基线臂「今」的记账三项直接
//! 反映分配器的物理状态（`allocated_slots` / `free_slots` / `deferred_slots`，与 `crates/singlefs-core/src/allocator.rs`
//! 同构），回收只在挂载 / 切换 / 回退重建 / 抬 F 时发生。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet, HashMap};

// ---------------------------------------------------------------------
// 常量（登记第一节「池与根环」、第七节 A1/A3）
// ---------------------------------------------------------------------

/// 单元区起始绝对槽号（`allocator.rs` 706-712 的单测几何）。
const UNIT_AREA_START_SLOT: u64 = 50176;
/// 单元区容量：每盘 4 GiB / 16384 − 50176（A3）。
const UNIT_AREA_CAPACITY_SLOTS: u64 = 211968;
/// 根环区域数（D22 已定项 2）。
const ROOT_RING_REGIONS: u64 = 3;
/// 区域按 txg 落哪块设备：0/1/0（D16 已定项 8）。
const ROOT_RING_REGION_DEVICE: [Device; 3] = [Device::Disk0, Device::Disk1, Device::Disk0];
/// 分配记录条目宽：甲-T1 与基线 20 字节（K2）。
const RECORD_WIDTH_BYTES_SINGLE_GENERATION: u64 = 20;
/// 分配记录条目宽：G12（同时存分配代与释放代）28 字节（K2）。
const RECORD_WIDTH_BYTES_DUAL_GENERATION: u64 = 28;
/// 分配记录树节点头宽：86 + 2×10 + 29（A1）。
const RECORD_TREE_NODE_HEADER_BYTES: u64 = 86 + 2 * 10 + 29;
/// 单节点能装的 20 字节条目数（A1）。
const RECORDS_PER_NODE_SINGLE_GENERATION: u64 = (16384 - RECORD_TREE_NODE_HEADER_BYTES) / RECORD_WIDTH_BYTES_SINGLE_GENERATION;
/// 单节点能装的 28 字节条目数（A1）。
const RECORDS_PER_NODE_DUAL_GENERATION: u64 = (16384 - RECORD_TREE_NODE_HEADER_BYTES) / RECORD_WIDTH_BYTES_DUAL_GENERATION;
/// defer 窗口最少保留的可退状态数（D16 已定项 1）。
const MINIMUM_RETAINED_STATES: u64 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
enum Device {
    Disk0,
    Disk1,
}

/// 根环槽在哪个区域：`txg mod 3`。
fn root_ring_region(txg: u64) -> u64 {
    txg % ROOT_RING_REGIONS
}

/// 根环槽落哪块设备：区域的设备归属（D16 已定项 8）。
fn root_ring_device(txg: u64) -> Device {
    ROOT_RING_REGION_DEVICE[root_ring_region(txg) as usize]
}

/// 区内槽号：`(txg div 3) mod S`（D22 已定项 2）。
fn root_ring_slot_in_region(txg: u64, slots_per_region: u64) -> u64 {
    (txg / ROOT_RING_REGIONS) % slots_per_region
}

/// 展平之后的环坐标（0..N），只用于本模型内部索引，不是格式意义上的槽号。
fn root_ring_flat_index(txg: u64, slots_per_region: u64) -> u64 {
    root_ring_region(txg) * slots_per_region + root_ring_slot_in_region(txg, slots_per_region)
}

// ---------------------------------------------------------------------
// 角色（登记第五节 M0.2 / M0.4 的骨架推导）
// ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
enum Role {
    InstanceTable,
    TableUnit,
    CentralMapTree,
    ExtentRoot,
    InodeRoot,
    AllocationTree,
    AcctTree,
    DataUnit,
    InodeLeaf,
}

impl Role {
    fn span_in_slots(self) -> u64 {
        match self {
            Role::InstanceTable | Role::DataUnit | Role::InodeLeaf => 2,
            Role::TableUnit | Role::CentralMapTree | Role::ExtentRoot | Role::InodeRoot | Role::AllocationTree | Role::AcctTree => 1,
        }
    }
}

/// E 触碰的四个角色（提交内生块的「记账/索引」那一半）。
const EMPTY_PUBLISH_ROLES: [Role; 4] = [Role::TableUnit, Role::CentralMapTree, Role::AllocationTree, Role::AcctTree];
/// O 在 E 之外多触碰的四个「文件」角色。
const OVERWRITE_EXTRA_FILE_ROLES: [Role; 4] = [Role::ExtentRoot, Role::InodeRoot, Role::DataUnit, Role::InodeLeaf];

fn overwrite_roles() -> Vec<Role> {
    let mut roles: Vec<Role> = EMPTY_PUBLISH_ROLES.to_vec();
    roles.extend_from_slice(&OVERWRITE_EXTRA_FILE_ROLES);
    roles
}

fn write_row_roles() -> Vec<Role> {
    let mut roles: Vec<Role> = EMPTY_PUBLISH_ROLES.to_vec();
    roles.push(Role::InstanceTable);
    roles
}

/// 实例切换里被重发的在飞发布是 O 时，写行发布再加 O 的四个文件角色（登记 M0.4）。
fn write_row_with_redo_overwrite_roles() -> Vec<Role> {
    let mut roles: Vec<Role> = write_row_roles();
    roles.extend_from_slice(&OVERWRITE_EXTRA_FILE_ROLES);
    roles
}

/// 一次发布分配次序（`transaction.rs` 810-835）：数据单元、实例表单元、extent 根、inode 叶容器、
/// inode 根、分配记录树、记账树、中央映射树、树表单元。只用于落点政策 P-impl 的段内次序。
const ALLOCATION_ORDER: [Role; 9] =
    [Role::DataUnit, Role::InstanceTable, Role::ExtentRoot, Role::InodeLeaf, Role::InodeRoot, Role::AllocationTree, Role::AcctTree, Role::CentralMapTree, Role::TableUnit];

fn ordered_roles(roles: &[Role]) -> Vec<Role> {
    ALLOCATION_ORDER.iter().copied().filter(|role| roles.contains(role)).collect()
}

// ---------------------------------------------------------------------
// 记账账本：一条记录 = (span, 分配代, 已释放, 释放代[, 释放代2 仅 G12])
// ---------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct LedgerEntry {
    span_in_slots: u64,
    allocation_generation: u64,
    released: bool,
    release_generation: u64,
}

impl LedgerEntry {
    fn allocated(span_in_slots: u64, allocation_generation: u64) -> LedgerEntry {
        LedgerEntry { span_in_slots, allocation_generation, released: false, release_generation: 0 }
    }
}

// ---------------------------------------------------------------------
// 池级状态：根环、实例表、F —— 与臂无关（登记第一节各定义均只依赖这三样）
// ---------------------------------------------------------------------

#[derive(Clone, Debug)]
struct RingEntry {
    txg: u64,
    instance: u64,
    rollback_floor: u64,
    is_nonempty: bool,
    /// 这个根发布时实例表的长度（供回退取「R_old 指着的那一版实例表」用：truncate 到这个长度）。
    instance_table_length_at_publish: usize,
}

#[derive(Clone, Debug)]
struct PoolState {
    slots_per_region: u64,
    ring: Vec<Option<RingEntry>>,
    /// 实例表：行 (实例, valid_up_to)；无该实例的行 = 该实例的一切根都有效。
    instance_table_rows: Vec<(u64, u64)>,
    current_txg: u64,
    current_instance: u64,
    next_instance_identifier: u64,
    current_rollback_floor: u64,
}

impl PoolState {
    fn new(slots_per_region: u64) -> PoolState {
        let capacity = (ROOT_RING_REGIONS * slots_per_region) as usize;
        let genesis = RingEntry { txg: 0, instance: 0, rollback_floor: 0, is_nonempty: false, instance_table_length_at_publish: 0 };
        let mut ring = vec![None; capacity];
        ring[root_ring_flat_index(0, slots_per_region) as usize] = Some(genesis);
        PoolState { slots_per_region, ring, instance_table_rows: Vec::new(), current_txg: 0, current_instance: 0, next_instance_identifier: 1, current_rollback_floor: 0 }
    }

    fn entry_at(&self, txg: u64) -> Option<&RingEntry> {
        self.ring[root_ring_flat_index(txg, self.slots_per_region) as usize].as_ref().filter(|entry| entry.txg == txg)
    }

    /// 环里全部自证合法（此刻仍占着该槽）的根，按 txg 升序。
    fn live_roots(&self) -> Vec<RingEntry> {
        let mut roots: Vec<RingEntry> = self.ring.iter().flatten().cloned().collect();
        roots.sort_by_key(|entry| entry.txg);
        roots
    }

    /// 一条根按最新持久根指着的实例表判是否被抛弃：有它实例的行 (i,T) 且 txg > T（`mount.rs` 185-192）。
    fn is_abandoned(&self, instance: u64, txg: u64) -> bool {
        self.instance_table_rows.iter().any(|(row_instance, valid_up_to)| *row_instance == instance && txg > *valid_up_to)
    }

    fn is_valid(&self, instance: u64, txg: u64) -> bool {
        !self.is_abandoned(instance, txg)
    }

    /// 环里最旧有效根的 txg（D16 已定项 1「环里最旧有效根」；写失败的槽按旧内容算，在飞发布不算，
    /// 因为本模型只在持久之后才把根写进 ring）。
    fn oldest_valid_root_txg(&self) -> u64 {
        self.live_roots().iter().filter(|entry| self.is_valid(entry.instance, entry.txg)).map(|entry| entry.txg).min().expect("环里至少有 mkfs 的根")
    }

    /// F_生效 = 各盘上根所带 F 最大值的最小值（一块盘没有根就不算它）。
    fn effective_floor(&self) -> u64 {
        let mut per_device: Vec<u64> = Vec::new();
        for device in [Device::Disk0, Device::Disk1] {
            let maximum_floor = self.live_roots().iter().filter(|entry| root_ring_device(entry.txg) == device).map(|entry| entry.rollback_floor).max();
            if let Some(value) = maximum_floor {
                per_device.push(value);
            }
        }
        per_device.into_iter().min().unwrap_or(0)
    }

    /// 甲-T1 与基线共用的门槛：max(F_生效, 环里最旧有效根)（D16 已定项 1「可再分配」）。
    /// 单一来源——回收步与记账公式都调这一处，别在调用点各写一份（U1 的变异锚点）。
    fn reclaim_threshold(&self) -> u64 {
        self.oldest_valid_root_txg().max(self.effective_floor())
    }

    /// 回退候选集：有效 ∧ txg ≥ F_生效。
    fn candidate_roots(&self) -> Vec<RingEntry> {
        let floor = self.effective_floor();
        self.live_roots().into_iter().filter(|entry| self.is_valid(entry.instance, entry.txg) && entry.txg >= floor).collect()
    }

    /// 被抛弃根：按最新持久根指着的实例表判不有效的根（不含回退重建那一刻额外加的那一条，另用
    /// `abandoned_roots_including_rollback_extension`）。
    fn abandoned_roots(&self) -> Vec<RingEntry> {
        self.live_roots().into_iter().filter(|entry| self.is_abandoned(entry.instance, entry.txg)).collect()
    }

    /// 回退重建那一刻的「被抛弃」：按最新持久根判被抛弃 ∪ (txg,实例) 大于 R_old 的根。
    fn abandoned_roots_including_rollback_extension(&self, rollback_target_txg: u64, rollback_target_instance: u64) -> Vec<RingEntry> {
        let mut set: BTreeMap<u64, RingEntry> = BTreeMap::new();
        for entry in self.abandoned_roots() {
            set.insert(entry.txg, entry);
        }
        for entry in self.live_roots() {
            if (entry.txg, entry.instance) > (rollback_target_txg, rollback_target_instance) {
                set.insert(entry.txg, entry);
            }
        }
        set.into_values().collect()
    }

    /// 「blocking」txg 集合，供 G12 的区间谓词用：候选集 ∪ 被抛弃根的 txg（登记第五节主谓词 G12）。
    fn interval_blocking_generations(&self) -> BTreeSet<u64> {
        let mut set: BTreeSet<u64> = BTreeSet::new();
        for entry in self.candidate_roots() {
            set.insert(entry.txg);
        }
        for entry in self.abandoned_roots() {
            set.insert(entry.txg);
        }
        set
    }

    /// 写一条根：把它放进环里对应的坐标（region/slot 由 txg 决定），推进 current_*。写失败时不调用本方法
    /// （ring 保留旧内容），只推进 current_txg 之外的调用方状态。
    fn commit_root(&mut self, txg: u64, instance: u64, rollback_floor: u64, is_nonempty: bool) {
        let index = root_ring_flat_index(txg, self.slots_per_region) as usize;
        let instance_table_length_at_publish = self.instance_table_rows.len();
        self.ring[index] = Some(RingEntry { txg, instance, rollback_floor, is_nonempty, instance_table_length_at_publish });
        self.current_txg = txg;
        self.current_instance = instance;
        self.current_rollback_floor = rollback_floor;
    }

    fn take_next_instance(&mut self) -> u64 {
        let instance = self.next_instance_identifier;
        self.next_instance_identifier += 1;
        instance
    }
}

// ---------------------------------------------------------------------
// 账户查询：I-3.1 的候选集去重求和（登记 walk.rs 46-72 的去重键 (设备, 起点槽, 跨度)）
// ---------------------------------------------------------------------

/// 一个引用：(起点槽, 跨度)，去重键含设备（这里只在单设备视角内去重，设备信息由调用方分开算）。
type Reference = (u64, u64);

fn sum_spans(references: &BTreeSet<Reference>) -> u64 {
    references.iter().map(|(_, span)| span).sum()
}

// ---------------------------------------------------------------------
// 主谓词
// ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Predicate {
    /// 甲-T1：已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)。
    ThresholdFloor,
    /// G12：已释放 ∧ 环里没有根 r：txg(r) ∈ [分配代, 释放代) ∧ (r 被抛弃 ∨ txg(r) ≥ F_生效)。
    IntervalBlocking,
}

/// 影子账形态。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ShadowLedger {
    /// 保守读法：环里每条被抛弃根的账里未释放的槽全部隔离，含当前账 / 候选账也引用的。
    Conservative,
    /// G5″：只隔离「被抛弃根引用 − 候选根引用」的差集。
    Narrow,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    ThresholdConservative,
    ThresholdNarrow,
    IntervalConservative,
    IntervalNarrow,
    /// 真实基线「今」：分配记录同甲-T1（20 字节一个代），回收只在挂载/切换/回退重建/抬 F 时发生，
    /// 记账三项直接反映分配器物理状态；影子账取豁免-候选集形态（`mount.rs` 201-258）。
    Baseline,
}

impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::ThresholdConservative => "threshold_conservative",
            Arm::ThresholdNarrow => "threshold_narrow",
            Arm::IntervalConservative => "interval_conservative",
            Arm::IntervalNarrow => "interval_narrow",
            Arm::Baseline => "baseline_today",
        }
    }

    fn predicate(self) -> Option<Predicate> {
        match self {
            Arm::ThresholdConservative | Arm::ThresholdNarrow | Arm::Baseline => Some(Predicate::ThresholdFloor),
            Arm::IntervalConservative | Arm::IntervalNarrow => Some(Predicate::IntervalBlocking),
        }
    }

    fn record_width_bytes(self) -> u64 {
        match self {
            Arm::ThresholdConservative | Arm::ThresholdNarrow | Arm::Baseline => RECORD_WIDTH_BYTES_SINGLE_GENERATION,
            Arm::IntervalConservative | Arm::IntervalNarrow => RECORD_WIDTH_BYTES_DUAL_GENERATION,
        }
    }

    fn records_per_node(self) -> u64 {
        match self {
            Arm::ThresholdConservative | Arm::ThresholdNarrow | Arm::Baseline => RECORDS_PER_NODE_SINGLE_GENERATION,
            Arm::IntervalConservative | Arm::IntervalNarrow => RECORDS_PER_NODE_DUAL_GENERATION,
        }
    }

    fn is_measured(self) -> bool {
        !matches!(self, Arm::Baseline)
    }
}

// ---------------------------------------------------------------------
// 落点政策
// ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PlacementPolicy {
    /// 每个单元取单元区内最低的、跨度内每槽都可发、跨度 2 时起点偶数的位置；没有段（M0.17）。
    Low,
    /// 用户数据取最低偶数对；提交内生块从开放段 bump，段 = 最低的、64 槽对齐、段内无占用的段（M0.16）。
    Impl,
}

// ---------------------------------------------------------------------
// 一个几何格
// ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GeometryCell {
    slots_per_region: u64,
    placement: PlacementPolicy,
    workload_period: u64, // ρ = 1/period；period=1 时 ρ=1，period=4 时 ρ=1/4
}

impl GeometryCell {
    fn ring_capacity(self) -> u64 {
        ROOT_RING_REGIONS * self.slots_per_region
    }
    fn tag(self) -> String {
        let placement = match self.placement {
            PlacementPolicy::Low => "low",
            PlacementPolicy::Impl => "impl",
        };
        format!("s{}_{}_rho{}", self.slots_per_region, placement, self.workload_period)
    }
}

const BASE_GEOMETRY: GeometryCell = GeometryCell { slots_per_region: 8, placement: PlacementPolicy::Impl, workload_period: 1 };

fn all_geometry_cells() -> Vec<GeometryCell> {
    let mut cells = Vec::new();
    for slots_per_region in [4u64, 8, 16] {
        for placement in [PlacementPolicy::Impl, PlacementPolicy::Low] {
            for workload_period in [1u64, 4] {
                cells.push(GeometryCell { slots_per_region, placement, workload_period });
            }
        }
    }
    cells
}

// ---------------------------------------------------------------------
// 一条根的账（供影子账、回退恢复、违例检查用）：role → (slot, span)。
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
struct RootAccount {
    slots: BTreeMap<u64, u64>, // slot -> span
}

impl RootAccount {
    /// 展开成账里覆盖的每一个物理槽号（跨度 2 的角色要展开成两个槽，不能只取起点）。
    fn expanded_slots(&self) -> impl Iterator<Item = u64> + '_ {
        self.slots.iter().flat_map(|(&start, &span)| (0..span).map(move |offset| start + offset))
    }
}

fn account_from_refs(refs: &HashMap<Role, u64>) -> RootAccount {
    let mut slots = BTreeMap::new();
    for (role, slot) in refs {
        slots.insert(*slot, role.span_in_slots());
    }
    RootAccount { slots }
}

/// Q1/Q2 用：一批 txg 的账在这条臂上引用的物理槽并集大小（去重，第一节「I-3.1（臂内读法）」的
/// 「按 (设备, 起点槽, 跨度) 去重，跨度求和」——本模型单设备视角下按物理槽本身去重即可，
/// 两盘同槽（M0.1）意味着两盘各自算一遍、槽号相同，不需要另带设备维）。
/// Q9 轨迹用：五条臂当前的分配记录条目数（Q9a）与两个读数计数器（Q9b，`reads_last_reclaim_step`
/// 是每次发布都会更新的「发布前步」读数；`reads_last_shadow_recompute` 只在重建 / 抬 F 时更新，
/// 调用方按 `is_rebuild_step` 决定这一格是不是「重建步」的采样点——见 `PublishOutcome` 字段注释）。
fn snapshot_allocation_record_read_counters(arms: &[ArmExecutor; 5]) -> ([u64; 5], [u64; 5], [u64; 5]) {
    let mut entries = [0u64; 5];
    let mut reclaim = [0u64; 5];
    let mut shadow = [0u64; 5];
    for (index, arm) in arms.iter().enumerate() {
        entries[index] = arm.ledger.len() as u64;
        reclaim[index] = arm.reads_last_reclaim_step;
        shadow[index] = arm.reads_last_shadow_recompute;
    }
    (entries, reclaim, shadow)
}

/// Q9a：一条臂某一时刻的条目数，推出条目字节与叶节点数（登记第六节 Q9a「条目字节 = 条目数 ×
/// 条目宽；叶节点数 = ⌈条目数 ÷ 每节点条目数⌉」）。
fn allocation_record_bytes_and_leaf_nodes(entries: u64, record_width_bytes: u64, records_per_node: u64) -> (u64, u64) {
    (entries * record_width_bytes, entries.div_ceil(records_per_node))
}

/// Q9 轨迹的峰值 / 均值 / 末值三元组；`values` 为空时三项都是 0（调用方另行判断样本数是不是 0，
/// 均值为 0 不代表「量到了 0」，见 Q9b「重建步」样本数可能为 0 的情形）。
fn peak_mean_end(values: &[u64]) -> (u64, f64, u64) {
    if values.is_empty() {
        return (0, 0.0, 0);
    }
    let peak = *values.iter().max().expect("非空");
    let mean = values.iter().sum::<u64>() as f64 / values.len() as f64;
    let end = *values.last().expect("非空");
    (peak, mean, end)
}

fn candidate_reference_union(arm: &ArmExecutor, txgs: &[u64]) -> BTreeSet<u64> {
    let mut union: BTreeSet<u64> = BTreeSet::new();
    for txg in txgs {
        if let Some(account) = arm.root_accounts.get(txg) {
            union.extend(account.expanded_slots());
        }
    }
    union
}

fn candidate_reference_union_size(arm: &ArmExecutor, txgs: &[u64]) -> u64 {
    candidate_reference_union(arm, txgs).len() as u64
}

/// Q10：G12×G5″ 的候选引用推法（登记第五节「G12」定义：「候选根 r 引用槽 x ⟺ 当前账里 x 的记录
/// 分配代 ≤ txg(r) ∧（未释放 ∨ txg(r) < 释放代）」）——从这条臂自己的账本直接推，不读候选根自己的
/// 账（`root_accounts`）。与 `candidate_reference_union`（M0.6 走读）比较得到 Q10。
fn interval_inferred_candidate_references(arm: &ArmExecutor, candidate_txgs: &[u64]) -> BTreeSet<u64> {
    let mut inferred: BTreeSet<u64> = BTreeSet::new();
    for (&slot, entry) in &arm.ledger {
        let referenced = candidate_txgs.iter().any(|&txg| entry.allocation_generation <= txg && (!entry.released || txg < entry.release_generation));
        if referenced {
            for offset in 0..entry.span_in_slots {
                inferred.insert(slot + offset);
            }
        }
    }
    inferred
}

/// Q6b：环里被抛弃根各自的「记账第 1 项」代理值——本模型没有为每条历史根单独存一份「它持久那一刻
/// 记的 item1」，只存了它的引用集合（`root_accounts`）；对一条仍在候选集 / 环里活着、按定义没有被
/// 破坏的根，它持久那一刻的 item1 与它自己的引用集合大小相等（这正是 I-3.1 要成立的那件事），所以
/// 用引用集合大小当代理值——登记第十二节修订记这一条简化。逐臂返回 (环里 txg 最大的被抛弃根的值,
/// 全部被抛弃根里的最大值)；环里没有被抛弃根时两项都是 `None`。
fn abandoned_root_item1_bounds(arms: &[ArmExecutor; 5], abandoned_txgs: &[u64]) -> [(Option<u64>, Option<u64>); 5] {
    let mut result: [(Option<u64>, Option<u64>); 5] = [(None, None); 5];
    if abandoned_txgs.is_empty() {
        return result;
    }
    let newest_txg = *abandoned_txgs.iter().max().expect("非空");
    for (index, arm) in arms.iter().enumerate() {
        let mut newest_value: Option<u64> = None;
        let mut maximum_value: Option<u64> = None;
        for &txg in abandoned_txgs {
            if let Some(account) = arm.root_accounts.get(&txg) {
                let size = account.expanded_slots().count() as u64;
                if txg == newest_txg {
                    newest_value = Some(size);
                }
                maximum_value = Some(maximum_value.map_or(size, |current: u64| current.max(size)));
            }
        }
        result[index] = (newest_value, maximum_value);
    }
    result
}

/// Q8（坏镜像矩阵）：一个 (臂, 格) 组合在某个合法基底状态上的记账三项与三种引用集合大小——
/// 臂内候选并集（I-3.1 臂内读法）、今天读法候选并集（I-3.1 今天 checker 读法）、最新根自己的
/// 引用集合（G8′）。坏镜像只改 `accounting`，三种引用集合大小保持基底不变（登记：B1a/B1b/B2/Bk1/Bk2
/// 只改 D0 的记账，不改盘上单元本身，所以走读得到的引用集合不受影响）。
#[derive(Clone, Copy, Debug, Default)]
struct BaseImageState {
    accounting: AccountingRow,
    candidate_by_effective_floor: u64,
    candidate_by_today_reading: u64,
    newest_reference_size: u64,
}

/// 四个候选检查在一份（可能被坏镜像改过的）状态上是否判红。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
struct AccountingChecks {
    allocated_statistic_mismatch_by_arm_reading: bool,
    allocated_statistic_mismatch_by_today_reading: bool,
    free_statistic_mismatch: bool,
    allocated_minus_deferred_red: bool,
}

fn accounting_checks(state: BaseImageState) -> AccountingChecks {
    let row = state.accounting;
    AccountingChecks {
        allocated_statistic_mismatch_by_arm_reading: row.item1_occupied_slots != state.candidate_by_effective_floor,
        allocated_statistic_mismatch_by_today_reading: row.item1_occupied_slots != state.candidate_by_today_reading,
        free_statistic_mismatch: row.item1_occupied_slots + row.item2_free_slots != UNIT_AREA_CAPACITY_SLOTS,
        allocated_minus_deferred_red: row.item1_occupied_slots.saturating_sub(row.item5_deferred_slots) != state.newest_reference_size,
    }
}

/// 五条臂在这一刻的 `BaseImageState`（登记第五节坏镜像的「基底」：β1 = H1 主历史最后一次发布之后，
/// β2 = H4dNm2 主历史回退之后第 S 次工作负载发布之后）。
fn base_image_states(world: &World) -> [BaseImageState; 5] {
    let candidate_by_effective_floor: Vec<u64> = world.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
    let floor_of_newest_root = world.pool.current_rollback_floor;
    let candidate_by_today_reading: Vec<u64> = world
        .pool
        .live_roots()
        .iter()
        .filter(|entry| world.pool.is_valid(entry.instance, entry.txg) && entry.txg >= floor_of_newest_root)
        .map(|entry| entry.txg)
        .collect();
    let mut states = [BaseImageState::default(); 5];
    for (index, arm) in world.arms.iter().enumerate() {
        states[index] = BaseImageState {
            accounting: arm.accounting_row(),
            candidate_by_effective_floor: candidate_reference_union_size(arm, &candidate_by_effective_floor),
            candidate_by_today_reading: candidate_reference_union_size(arm, &candidate_by_today_reading),
            newest_reference_size: account_from_refs(&arm.refs).expanded_slots().count() as u64,
        };
    }
    states
}

/// B1a（登记第五节坏镜像）：活单元被记成已释放，只翻记录标志——记账三项与引用集合都不受影响，
/// 走的是「记录本身没人读」这条口径（第三节「checker 不读记账第 5 项、不读分配记录的已释放标志」）。
fn bad_image_b1a(base: BaseImageState) -> BaseImageState {
    base
}

/// B1b：B1a 再把第 5 项加 2 槽（登记：「第 1 项、第 2 项不动」）。
fn bad_image_b1b(base: BaseImageState) -> BaseImageState {
    let mut modified = base;
    modified.accounting.item5_deferred_slots += 2;
    modified
}

/// B2：第 5 项单独多报一槽，别的不动。
fn bad_image_deferred_overcount_by_one(base: BaseImageState) -> BaseImageState {
    let mut modified = base;
    modified.accounting.item5_deferred_slots += 1;
    modified
}

/// PC-F 的 Bk1（阳性对照）：D0 第 1 项多记 1 槽。
fn bad_image_bk1(base: BaseImageState) -> BaseImageState {
    let mut modified = base;
    modified.accounting.item1_occupied_slots += 1;
    modified
}

/// PC-F 的 Bk2（阳性对照）：D0 第 2 项少记 1 槽。
fn bad_image_bk2(base: BaseImageState) -> BaseImageState {
    let mut modified = base;
    modified.accounting.item2_free_slots = modified.accounting.item2_free_slots.saturating_sub(1);
    modified
}

/// Q6b 一次采样的判定结果（登记第六节 Q6b-甲/乙）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AbandonedRootBoundCheck {
    isolated_exceeds_newest_bound: bool,
    newest_bound_negative: bool,
    isolated_exceeds_maximum_bound: bool,
    maximum_bound_negative: bool,
}

/// Q6b：把「这一刻的隔离槽数」与两种读法的上界比较（读法甲用环里 txg 最大的被抛弃根，读法乙用
/// 全部被抛弃根里的最大值；两者都减去 R_old 的代理值）。纯函数，main() 的采样循环与单测共用。
fn abandoned_root_bound_check(isolated_slots: u64, newest_abandoned_item1: u64, maximum_abandoned_item1: u64, rollback_target_reference_count: u64) -> AbandonedRootBoundCheck {
    let isolated = isolated_slots as i64;
    let newest_bound = newest_abandoned_item1 as i64 - rollback_target_reference_count as i64;
    let maximum_bound = maximum_abandoned_item1 as i64 - rollback_target_reference_count as i64;
    AbandonedRootBoundCheck {
        isolated_exceeds_newest_bound: isolated > newest_bound,
        newest_bound_negative: newest_bound < 0,
        isolated_exceeds_maximum_bound: isolated > maximum_bound,
        maximum_bound_negative: maximum_bound < 0,
    }
}

/// Q10 的一次核对：只在 IntervalNarrow（G12×G5″）这条臂上有意义（登记第五节 G12 的「支持它的人认的
/// 样子」只对这条组合许诺「不另读候选根的树」）；别的臂调用这个函数不做任何事。核对不一致时累计
/// `interval_candidate_inference_mismatch_count`，不改变任何实际状态——它是一次旁路核对，不参与隔离集计算
/// （隔离集仍按 `recompute_shadow` / `isolate_additional_for_raised_floor` 里已经写好的走读路径算，
/// 本函数只是多算一遍、拿去比对，避免「一改就影响真实结果」这类风险）。
fn check_interval_candidate_inference_against_walk(arm: &mut ArmExecutor, candidate_txgs: &[u64]) {
    if arm.arm != Arm::IntervalNarrow {
        return;
    }
    let inferred = interval_inferred_candidate_references(arm, candidate_txgs);
    let walked = candidate_reference_union(arm, candidate_txgs);
    if inferred != walked {
        arm.interval_candidate_inference_mismatch_count += 1;
    }
}

// ---------------------------------------------------------------------
// 每一次发布记账三项快照
// ---------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
struct AccountingRow {
    item1_occupied_slots: u64,
    item2_free_slots: u64,
    item5_deferred_slots: u64,
}

/// 一次发布之前（该臂的回收步做完之后、这次发布释放与分配之前）的快照：多扣、影子账隔离槽数。
#[derive(Clone, Copy, Debug, Default)]
struct PreAllocationSnapshot {
    over_held_slots: i64, // 「分配器不发的槽」-「有效根∪被抛弃根引用的槽」
    isolated_slots: u64,
}

// ---------------------------------------------------------------------
// 臂执行器：账本、物理占用集合、落点、影子账 —— 全部只属于这一条臂。
// ---------------------------------------------------------------------

#[derive(Clone)]
struct ArmExecutor {
    arm: Arm,
    placement: PlacementPolicy,
    refs: HashMap<Role, u64>,
    /// 全历史账本：slot -> 记录，只在再分配时被覆写，从不因回收而删除（登记：回收不改记录内容）。
    ledger: HashMap<u64, LedgerEntry>,
    /// 「分配器不发的槽」：仍分配 ∪ 已释放未回收(物理) ∪ 影子账隔离 ∪ 基线抬 F 扣住。
    occupied: BTreeSet<u64>,
    isolated: BTreeSet<u64>,
    withheld: BTreeSet<u64>,
    /// 测量臂：未释放记录跨度和（增量维护）。
    allocated_span_total: u64,
    /// 测量臂：已释放、物理上尚未越过门槛的记录——驱动 occupied / 多扣 / 落点，门槛用「环」（这一刻，
    /// p 还没有自己的账）。
    pending: Vec<PendingRecord>,
    /// 测量臂：独立事件路径的空闲计数，与 pending 同一门槛（物理，驱动多扣）。
    item2_total: u64,
    /// 测量臂：记账行公式专用——与 pending 同源（release 时一起 push），但门槛用「环′」
    /// （p 自己的账已经换进环之后）。只影响写进根里的第 1/5 项，不影响物理占用（登记第五节
    /// 「发布 p 的记账行」：环′ 是记账用的环，不是回收步物理用的环）。
    formula_pending: Vec<PendingRecord>,
    /// 测量臂：记账第 2 项公式专用，独立事件路径，门槛同 formula_pending（环′）。
    formula_item2_total: u64,
    /// 基线：占着（含 defer）、空闲、defer 三个物理计数器，直接对应 allocator.rs 三个字段。
    baseline_allocated_slots: u64,
    baseline_free_slots: u64,
    baseline_deferred_slots: u64,
    /// 基线：released 且物理上还没被回收步清掉的槽（回收步只扫描这个集合，不扫整本账）。
    baseline_deferred_set: BTreeSet<u64>,
    /// 落点政策 P-impl 的开放段游标（段起点，段内 bump）。
    open_segment_start: Option<u64>,
    /// 段内下一次扫描的起点：只在段内单调前进，跳过的槽不回填（登记 M0.16「游标 bump」）。
    open_segment_cursor: u64,
    /// 每条仍在环里的根，这条臂自己的账快照（供影子账、回退、违例检查）。
    root_accounts: BTreeMap<u64, RootAccount>,
    /// 供回退 / 切换基准恢复用的整臂快照。
    full_snapshots: BTreeMap<u64, ArmFullSnapshot>,
    /// 每个回收步实际读的分配记录树棵数（Q9b 的累计器，重建步单独记）。
    reads_last_reclaim_step: u64,
    reads_last_shadow_recompute: u64,
    /// 违例计数（Q4i 候选根那一半、Q4ii 被抛弃根那一半）。
    violations_candidate: u64,
    violations_abandoned: u64,
    /// Q10：G12×G5″ 的候选引用推法（登记第五节「G12」支持者形态：「候选根 r 引用槽 x ⟺ 当前账里 x 的
    /// 记录分配代 ≤ txg(r) ∧（未释放 ∨ txg(r) < 释放代）」，不另读候选根的树）与按 M0.6 走读候选根账
    /// 得到的并集不一致的回收步数。只在 IntervalNarrow 这条臂上累计（G12 与 G5″ 都齐才有意义）。
    interval_candidate_inference_mismatch_count: u64,
}

#[derive(Clone)]
struct PendingRecord {
    slot: u64,
    span: u64,
    allocation_generation: u64,
    release_generation: u64,
}

/// 两种主谓词的可再分配判定（登记第五节「主谓词 甲-T1 / G12」）：
/// 甲-T1 = 已释放 ∧ 释放代 ≤ 门槛；G12 = 已释放 ∧ 环里没有根 r：txg(r) ∈ [分配代, 释放代) 且 r 属于
/// blocking 集合（候选集 ∪ 被抛弃根，由调用方按环′或环算好传入）。两条重算路径（物理回收、记账
/// 公式）共用这一处判定，改一处两边同步（M1–M6 的变异锚点）。
fn is_record_reclaimable(predicate: Predicate, threshold: u64, blocking: Option<&BTreeSet<u64>>, record: &PendingRecord) -> bool {
    match predicate {
        Predicate::ThresholdFloor => record.release_generation <= threshold,
        Predicate::IntervalBlocking => {
            let blocking = blocking.expect("G12 需要 blocking 集合");
            blocking.range(record.allocation_generation..record.release_generation).next().is_none()
        }
    }
}

#[derive(Clone)]
struct ArmFullSnapshot {
    refs: HashMap<Role, u64>,
    ledger: HashMap<u64, LedgerEntry>,
    occupied: BTreeSet<u64>,
    isolated: BTreeSet<u64>,
    withheld: BTreeSet<u64>,
    allocated_span_total: u64,
    pending: Vec<PendingRecord>,
    item2_total: u64,
    formula_pending: Vec<PendingRecord>,
    formula_item2_total: u64,
    baseline_allocated_slots: u64,
    baseline_free_slots: u64,
    baseline_deferred_slots: u64,
    baseline_deferred_set: BTreeSet<u64>,
    open_segment_start: Option<u64>,
    open_segment_cursor: u64,
    // root_accounts 不在这份快照里：它按自己的注释（496 行）该一直反映「每条仍在环里的根」，
    // 由 World::prune_evicted_accounts 独立维护生命周期，不随 rebuild_allocator 时光倒流
    // （2026-09-17 S1 对拍发现：把它塞进快照、restore_full 时整份覆盖，会让回退之后
    // recompute_shadow 查不到写行/暖机/C 这些「快照时点还没发生」的 txg 的账，隔离集恒空）。
}

impl ArmExecutor {
    fn new(arm: Arm, placement: PlacementPolicy) -> ArmExecutor {
        ArmExecutor {
            arm,
            placement,
            refs: HashMap::new(),
            ledger: HashMap::new(),
            occupied: BTreeSet::new(),
            isolated: BTreeSet::new(),
            withheld: BTreeSet::new(),
            allocated_span_total: 0,
            pending: Vec::new(),
            item2_total: UNIT_AREA_CAPACITY_SLOTS,
            formula_pending: Vec::new(),
            formula_item2_total: UNIT_AREA_CAPACITY_SLOTS,
            baseline_allocated_slots: 0,
            baseline_free_slots: UNIT_AREA_CAPACITY_SLOTS,
            baseline_deferred_slots: 0,
            baseline_deferred_set: BTreeSet::new(),
            open_segment_start: None,
            open_segment_cursor: 0,
            root_accounts: BTreeMap::new(),
            full_snapshots: BTreeMap::new(),
            reads_last_reclaim_step: 0,
            reads_last_shadow_recompute: 0,
            violations_candidate: 0,
            violations_abandoned: 0,
            interval_candidate_inference_mismatch_count: 0,
        }
    }

    fn full_snapshot(&self) -> ArmFullSnapshot {
        ArmFullSnapshot {
            refs: self.refs.clone(),
            ledger: self.ledger.clone(),
            occupied: self.occupied.clone(),
            isolated: self.isolated.clone(),
            withheld: self.withheld.clone(),
            allocated_span_total: self.allocated_span_total,
            pending: self.pending.clone(),
            item2_total: self.item2_total,
            formula_pending: self.formula_pending.clone(),
            formula_item2_total: self.formula_item2_total,
            baseline_allocated_slots: self.baseline_allocated_slots,
            baseline_free_slots: self.baseline_free_slots,
            baseline_deferred_slots: self.baseline_deferred_slots,
            baseline_deferred_set: self.baseline_deferred_set.clone(),
            open_segment_start: self.open_segment_start,
            open_segment_cursor: self.open_segment_cursor,
        }
    }

    fn restore_full(&mut self, snapshot: &ArmFullSnapshot) {
        self.refs = snapshot.refs.clone();
        self.ledger = snapshot.ledger.clone();
        self.occupied = snapshot.occupied.clone();
        self.isolated = snapshot.isolated.clone();
        self.withheld = snapshot.withheld.clone();
        self.allocated_span_total = snapshot.allocated_span_total;
        self.pending = snapshot.pending.clone();
        self.item2_total = snapshot.item2_total;
        self.formula_pending = snapshot.formula_pending.clone();
        self.formula_item2_total = snapshot.formula_item2_total;
        self.baseline_allocated_slots = snapshot.baseline_allocated_slots;
        self.baseline_free_slots = snapshot.baseline_free_slots;
        self.baseline_deferred_slots = snapshot.baseline_deferred_slots;
        self.baseline_deferred_set = snapshot.baseline_deferred_set.clone();
        self.open_segment_start = snapshot.open_segment_start;
        self.open_segment_cursor = snapshot.open_segment_cursor;
        // root_accounts 不恢复：见 ArmFullSnapshot 定义处的注释。
    }

    /// mkfs：实例表单元 instance_table_slot @ 起点、树表单元 table_unit_slot = instance_table_slot + 2，第 0 代根引用 {instance_table_slot,table_unit_slot}，分配代 0。
    fn mkfs(&mut self) {
        let instance_table_slot = UNIT_AREA_START_SLOT;
        let table_unit_slot = UNIT_AREA_START_SLOT + 2;
        self.ledger.insert(instance_table_slot, LedgerEntry::allocated(Role::InstanceTable.span_in_slots(), 0));
        self.ledger.insert(table_unit_slot, LedgerEntry::allocated(Role::TableUnit.span_in_slots(), 0));
        self.occupied.insert(instance_table_slot);
        self.occupied.insert(instance_table_slot + 1);
        self.occupied.insert(table_unit_slot);
        self.allocated_span_total = Role::InstanceTable.span_in_slots() + Role::TableUnit.span_in_slots();
        self.item2_total = UNIT_AREA_CAPACITY_SLOTS - self.allocated_span_total;
        self.formula_item2_total = self.item2_total;
        self.baseline_allocated_slots = self.allocated_span_total;
        self.baseline_free_slots = self.item2_total;
        self.refs.insert(Role::InstanceTable, instance_table_slot);
        self.refs.insert(Role::TableUnit, table_unit_slot);
        self.root_accounts.insert(0, account_from_refs(&self.refs));
    }

    /// 落点：从「不在 occupied 里」的槽里按政策选。P-low：单元区内最低、跨度内每槽都空、跨度 2 起点偶数。
    /// P-impl：数据单元同 P-low 的偶数对规则；提交内生块从开放段 bump（64 槽对齐段，段内无占用才开新段）。
    fn allocate_slot(&mut self, role: Role) -> u64 {
        let span = role.span_in_slots();
        match (self.placement, role) {
            (_, Role::DataUnit) | (PlacementPolicy::Low, _) => self.lowest_free_span(span, span == 2),
            (PlacementPolicy::Impl, _) => self.bump_from_open_segment(span, span == 2),
        }
    }

    fn is_free(&self, slot: u64) -> bool {
        !self.occupied.contains(&slot)
    }

    fn lowest_free_span(&mut self, span: u64, start_must_be_even: bool) -> u64 {
        let mut candidate = UNIT_AREA_START_SLOT;
        loop {
            if start_must_be_even && candidate % 2 != 0 {
                candidate += 1;
                continue;
            }
            let fits = (0..span).all(|offset| self.is_free(candidate + offset));
            if fits {
                return candidate;
            }
            candidate += if start_must_be_even { 2 } else { 1 };
        }
    }

    /// 开放段 = 单元区内最低的、64 槽对齐、段内无任何占用的段；游标只在段内单调前进，被挡（或按
    /// 对齐跳过）的槽从此不再回填——下一次调用从上一次留下的游标位置接着扫，不重新从段首扫
    /// （登记 M0.16「游标 bump」；实测第一个事务锚点 K1 证明了这一点：槽 50241 被 inode 叶容器的
    /// 偶数对齐跳过之后，inode 根落在 50244 而不是回填 50241）。
    /// 数据单元以外的跨度 2 单元按 32768 字节（= 2 槽）对齐取，即起点必须是偶数（M0.16）。
    fn bump_from_open_segment(&mut self, span: u64, start_must_be_even: bool) -> u64 {
        const SEGMENT_SLOTS: u64 = 64;
        loop {
            if self.open_segment_start.is_none() {
                let mut candidate = UNIT_AREA_START_SLOT;
                loop {
                    let aligned = candidate.div_ceil(SEGMENT_SLOTS) * SEGMENT_SLOTS;
                    if (0..SEGMENT_SLOTS).all(|offset| self.is_free(aligned + offset)) {
                        self.open_segment_start = Some(aligned);
                        self.open_segment_cursor = aligned;
                        break;
                    }
                    candidate = aligned + SEGMENT_SLOTS;
                }
            }
            let segment_start = self.open_segment_start.expect("刚刚赋值过");
            let segment_end = segment_start + SEGMENT_SLOTS;
            while self.open_segment_cursor + span <= segment_end {
                let cursor = self.open_segment_cursor;
                if start_must_be_even && cursor % 2 != 0 {
                    self.open_segment_cursor += 1;
                    continue;
                }
                if (0..span).all(|offset| self.is_free(cursor + offset)) {
                    self.open_segment_cursor = cursor + span;
                    return cursor;
                }
                self.open_segment_cursor += if start_must_be_even { 2 } else { 1 };
            }
            // 段满：开下一段。
            self.open_segment_start = None;
        }
    }

    /// 释放一个角色（若此前存在）：记录改写成已释放，物理上先留在 occupied（回收步再清）。
    fn release_role_if_present(&mut self, role: Role, release_generation: u64) {
        if let Some(slot) = self.refs.remove(&role) {
            let span = role.span_in_slots();
            if self.arm.is_measured() {
                self.allocated_span_total -= span;
                let allocation_generation = self.ledger[&slot].allocation_generation;
                self.pending.push(PendingRecord { slot, span, allocation_generation, release_generation });
                self.formula_pending.push(PendingRecord { slot, span, allocation_generation, release_generation });
            } else {
                self.baseline_deferred_slots += span;
                self.baseline_deferred_set.insert(slot);
            }
            if let Some(entry) = self.ledger.get_mut(&slot) {
                entry.released = true;
                entry.release_generation = release_generation;
            }
        }
    }

    fn allocate_role(&mut self, role: Role, allocation_generation: u64) {
        let slot = self.allocate_slot(role);
        let span = role.span_in_slots();
        self.ledger.insert(slot, LedgerEntry::allocated(span, allocation_generation));
        for offset in 0..span {
            self.occupied.insert(slot + offset);
        }
        if self.arm.is_measured() {
            self.allocated_span_total += span;
            self.formula_item2_total -= span;
        } else {
            self.baseline_allocated_slots += span;
            self.baseline_free_slots -= span;
        }
        self.item2_total -= span;
        self.refs.insert(role, slot);
    }

    /// 测量臂的回收步：用给定门槛（甲-T1）或给定 blocking 集合（G12）把 pending 里越过门槛的记录
    /// 从 item5 移到 item2，物理上把对应槽从 occupied 里摘掉（除非仍被隔离/扣住）。
    fn reclaim_measured(&mut self, threshold: u64, blocking: Option<&BTreeSet<u64>>) {
        let predicate = self.arm.predicate().expect("测量臂才调用");
        let mut reads = 0u64;
        let mut still_pending = Vec::with_capacity(self.pending.len());
        for record in self.pending.drain(..) {
            reads += 1;
            if is_record_reclaimable(predicate, threshold, blocking, &record) {
                self.item2_total += record.span;
                for offset in 0..record.span {
                    let physical_slot = record.slot + offset;
                    if !self.isolated.contains(&physical_slot) && !self.withheld.contains(&physical_slot) {
                        self.occupied.remove(&physical_slot);
                    }
                }
            } else {
                still_pending.push(record);
            }
        }
        self.pending = still_pending;
        self.reads_last_reclaim_step = reads;
    }

    /// 记账行公式专用的重算：用环′门槛把 formula_pending 里越过门槛的记录移出（登记第五节「发布 p
    /// 的记账行」）。只影响 formula_item2_total / formula_pending，不碰 occupied——它不是物理回收，
    /// 只是「如果現在就按这条根持久之后的环重新判一次，第 1/5 项该是多少」。
    fn drain_formula_pending(&mut self, threshold: u64, blocking: Option<&BTreeSet<u64>>) {
        let predicate = self.arm.predicate().expect("测量臂才调用");
        let mut still_pending = Vec::with_capacity(self.formula_pending.len());
        for record in self.formula_pending.drain(..) {
            if is_record_reclaimable(predicate, threshold, blocking, &record) {
                self.formula_item2_total += record.span;
            } else {
                still_pending.push(record);
            }
        }
        self.formula_pending = still_pending;
    }

    /// 基线的回收：只在挂载 / 切换 / 回退重建 / 抬 F 时调用；门槛为 max(floor, oldest_valid)。
    /// 只扫描「已释放、物理上还没被回收步清掉」的集合，不扫整本账。
    /// **修过（2026-09-17，第三段，见跑前登记第十二节）**：此前 `baseline_free_slots` 只在
    /// `withhold_instead_of_free == false` 时才加，扣住期间账目上的第 2 项（空闲）不涨——但真实实现
    /// `allocator.rs` 250–269 的 `mark_reclaimed` 与 611–626 的 `reclaim_released_up_to` 显示
    /// `free_slots += span` 与 `hold_until_floor_takes_effect(...)` 是两件独立的事：`free_slots`
    /// 无条件在回收那一刻就涨，「不许发出去」完全靠 `held_until_floor_takes_effect` 这个独立位图
    /// （`is_free` 另外查它，239 行），不影响 `free_slots`/`allocated_slots` 这两个记账计数器。
    /// 旧写法让抬 F 扣住期间 I-5.2（item1+item2==容量）判红，被冒烟测试
    /// `all_history_families_run_to_completion_on_every_geometry_cell` 的 H2 抓到（S=4，txg=39）。
    fn reclaim_baseline(&mut self, threshold: u64, withhold_instead_of_free: bool) {
        let mut reads = 0u64;
        let mut still_deferred: BTreeSet<u64> = BTreeSet::new();
        for slot in std::mem::take(&mut self.baseline_deferred_set) {
            reads += 1;
            let entry = self.ledger[&slot];
            if entry.release_generation <= threshold {
                let span = entry.span_in_slots;
                self.baseline_allocated_slots -= span;
                self.baseline_deferred_slots -= span;
                self.baseline_free_slots += span;
                if withhold_instead_of_free {
                    for offset in 0..span {
                        self.withheld.insert(slot + offset);
                    }
                } else {
                    for offset in 0..span {
                        let physical_slot = slot + offset;
                        if !self.isolated.contains(&physical_slot) && !self.withheld.contains(&physical_slot) {
                            self.occupied.remove(&physical_slot);
                        }
                    }
                }
            } else {
                still_deferred.insert(slot);
            }
        }
        self.baseline_deferred_set = still_deferred;
        self.reads_last_reclaim_step = reads;
    }

    /// 抬 F 放开时，把 withheld 里门槛已经 <= 新 F_生效 的槽放行（`mount.rs` 抬 F 收尾那一步）。
    fn release_withheld(&mut self) {
        for slot in std::mem::take(&mut self.withheld) {
            if !self.isolated.contains(&slot) {
                self.occupied.remove(&slot);
            }
        }
    }

    fn accounting_row(&self) -> AccountingRow {
        if self.arm.is_measured() {
            let item5 = self.formula_pending.iter().map(|record| record.span).sum::<u64>();
            AccountingRow { item1_occupied_slots: self.allocated_span_total + item5, item2_free_slots: self.formula_item2_total, item5_deferred_slots: item5 }
        } else {
            AccountingRow { item1_occupied_slots: self.baseline_allocated_slots, item2_free_slots: self.baseline_free_slots, item5_deferred_slots: self.baseline_deferred_slots }
        }
    }

    fn shadow_kind(&self) -> ShadowLedger {
        match self.arm {
            Arm::ThresholdConservative | Arm::IntervalConservative => ShadowLedger::Conservative,
            Arm::ThresholdNarrow | Arm::IntervalNarrow | Arm::Baseline => ShadowLedger::Narrow,
        }
    }

    /// 重算隔离集（整个替换）：保守 = ∪被抛弃根账；G5″/基线 = ∪被抛弃根账 − ∪候选根账。
    /// 摘掉不再隔离的槽时，若它已经越过测量臂的门槛（已经不在 pending 里、也不在 occupied 之外的
    /// 已回收记录里）就从 occupied 摘掉；仍未达门槛的保持在 occupied（因为它本来就该被回收步扣着）。
    fn recompute_shadow(&mut self, abandoned_txgs: &[u64], candidate_txgs: &[u64]) {
        check_interval_candidate_inference_against_walk(self, candidate_txgs);
        let mut abandoned_union: BTreeSet<u64> = BTreeSet::new();
        for txg in abandoned_txgs {
            if let Some(account) = self.root_accounts.get(txg) {
                abandoned_union.extend(account.expanded_slots());
            }
        }
        let new_isolated: BTreeSet<u64> = match self.shadow_kind() {
            ShadowLedger::Conservative => abandoned_union,
            ShadowLedger::Narrow => {
                let mut candidate_union: BTreeSet<u64> = BTreeSet::new();
                for txg in candidate_txgs {
                    if let Some(account) = self.root_accounts.get(txg) {
                        candidate_union.extend(account.expanded_slots());
                    }
                }
                abandoned_union.difference(&candidate_union).copied().collect()
            }
        };
        let reads = abandoned_txgs.len() as u64 + if self.shadow_kind() == ShadowLedger::Narrow { candidate_txgs.len() as u64 } else { 0 };
        // 放开不再隔离的槽：若它不是「已分配未释放」「已释放未越过门槛」，才真正摘出 occupied。
        // 跨度 2 的记录只用起点做键，物理槽可能是起点或起点 + 1，先找到这个槽所属记录的起点。
        for slot in self.isolated.difference(&new_isolated).copied().collect::<Vec<u64>>() {
            let record_start = if self.ledger.contains_key(&slot) {
                Some(slot)
            } else if slot > 0 && self.ledger.get(&(slot - 1)).is_some_and(|entry| entry.span_in_slots == 2) {
                Some(slot - 1)
            } else {
                None
            };
            let still_held_for_other_reason = record_start.is_some_and(|start| {
                let entry = self.ledger[&start];
                !entry.released || (self.arm.is_measured() && self.pending.iter().any(|record| record.slot == start)) || (!self.arm.is_measured() && self.baseline_deferred_set.contains(&start))
            }) || self.withheld.contains(&slot);
            if !still_held_for_other_reason {
                self.occupied.remove(&slot);
            }
        }
        for slot in &new_isolated {
            self.occupied.insert(*slot);
        }
        self.isolated = new_isolated;
        self.reads_last_shadow_recompute = reads;
    }

    /// 抬 F 专用的追加式隔离重算（`mount.rs` 609–628「候选集按新 F 缩小，只被被抛弃根引用的槽会变多，
    /// 回收之前先按新 F 重算影子账，不然那个槽回收之后就发得出去」）：只增不减——
    /// `isolate_slots_referenced_only_by_abandoned_roots` 只调用 `isolate_abandoned`，不撤销已经
    /// 隔离的槽（隔离位在一次挂载之内只增不清，第三节）。豁免集 = 当前账里还分配着的槽 ∪ 传入的候选账
    /// （候选按目标 F 过滤，调用方算好传入，与 mount.rs 611-628 同：floor 用还没生效的新 F，不是 F_生效）。
    fn isolate_additional_for_raised_floor(&mut self, abandoned_txgs: &[u64], candidate_txgs: &[u64]) {
        check_interval_candidate_inference_against_walk(self, candidate_txgs);
        let mut exempt: BTreeSet<u64> = BTreeSet::new();
        for (&slot, entry) in &self.ledger {
            if !entry.released {
                for offset in 0..entry.span_in_slots {
                    exempt.insert(slot + offset);
                }
            }
        }
        for txg in candidate_txgs {
            if let Some(account) = self.root_accounts.get(txg) {
                exempt.extend(account.expanded_slots());
            }
        }
        let mut abandoned_union: BTreeSet<u64> = BTreeSet::new();
        for txg in abandoned_txgs {
            if let Some(account) = self.root_accounts.get(txg) {
                abandoned_union.extend(account.expanded_slots());
            }
        }
        let newly_isolated: Vec<u64> = abandoned_union.difference(&exempt).copied().collect();
        self.reads_last_shadow_recompute = abandoned_txgs.len() as u64 + candidate_txgs.len() as u64;
        for slot in newly_isolated {
            self.isolated.insert(slot);
            self.occupied.insert(slot);
        }
    }

    /// 一次挂载 / 切换 / 回退重建之后的整体重建：从给定的历史账（一份 ArmFullSnapshot 的账本部分）
    /// 重新载入分配账（M0.13「分配账 = R_old 那一版」；M0.14「分配账退回失败那次所基于的根那一版」），
    /// 但落点游标 / occupied 物理集合按重建规则重算：先按 threshold 立刻回收，再重算影子账。
    fn rebuild_allocator(&mut self, base: &ArmFullSnapshot, threshold: u64, blocking: Option<&BTreeSet<u64>>, abandoned_txgs: &[u64], candidate_txgs: &[u64]) {
        self.restore_full(base);
        self.open_segment_start = None;
        self.open_segment_cursor = 0;
        // occupied 需要从账本重新推导：仍分配 ∪（已释放但未达门槛）。
        self.occupied.clear();
        self.pending.clear();
        self.item2_total = UNIT_AREA_CAPACITY_SLOTS;
        self.baseline_allocated_slots = 0;
        self.baseline_free_slots = UNIT_AREA_CAPACITY_SLOTS;
        self.baseline_deferred_slots = 0;
        self.baseline_deferred_set.clear();
        self.withheld.clear();
        for (&slot, entry) in self.ledger.clone().iter() {
            if !entry.released {
                for offset in 0..entry.span_in_slots {
                    self.occupied.insert(slot + offset);
                }
                self.item2_total -= entry.span_in_slots;
                if self.arm.is_measured() {
                    // allocated_span_total 已经由 restore_full 带回（它是账本的派生量，这里重新推导以防不一致）。
                } else {
                    self.baseline_allocated_slots += entry.span_in_slots;
                    self.baseline_free_slots -= entry.span_in_slots;
                }
            } else {
                let reclaimable = if self.arm.is_measured() {
                    match self.arm.predicate().expect("测量臂") {
                        Predicate::ThresholdFloor => entry.release_generation <= threshold,
                        Predicate::IntervalBlocking => !blocking.expect("G12 需要 blocking").range(entry.allocation_generation..entry.release_generation).next().is_some(),
                    }
                } else {
                    entry.release_generation <= threshold
                };
                if reclaimable {
                    // item2_total 已初始化为容量，「已释放且可再分配」的槽不参与任何扣减，自然计入空闲。
                } else {
                    for offset in 0..entry.span_in_slots {
                        self.occupied.insert(slot + offset);
                    }
                    self.item2_total -= entry.span_in_slots;
                    if self.arm.is_measured() {
                        self.pending.push(PendingRecord { slot, span: entry.span_in_slots, allocation_generation: entry.allocation_generation, release_generation: entry.release_generation });
                    } else {
                        self.baseline_allocated_slots += entry.span_in_slots;
                        self.baseline_deferred_slots += entry.span_in_slots;
                        self.baseline_deferred_set.insert(slot);
                        // 修过（2026-09-17，第三段，见跑前登记第十二节）：此前这一支漏了这一行——
                        // 「已释放但还没达到门槛」的槽仍然占着（进了 item1/deferred），baseline_free_slots
                        // 却没有跟着扣，让 item1+item2 比容量多出「这一支累计的槽数」（被冒烟测试
                        // all_history_families_run_to_completion_on_every_geometry_cell 的 H2 抓到，
                        // S=4 时多 110，与 `self.item2_total -= entry.span_in_slots;` 那一行是同一件事的
                        // 两份账，baseline 那份漏抄了）。
                        self.baseline_free_slots -= entry.span_in_slots;
                    }
                }
            }
        }
        if self.arm.is_measured() {
            self.allocated_span_total = self.ledger.values().filter(|entry| !entry.released).map(|entry| entry.span_in_slots).sum();
            // 重建那一刻，记账公式与物理回收共用同一个门槛（还没有「环′ 比环多一代」这回事——
            // 那个差异是接下来正常发布之后才会长出来的），所以两份 pending/item2 先置成一样。
            self.formula_pending = self.pending.clone();
            self.formula_item2_total = self.item2_total;
        }
        self.recompute_shadow(abandoned_txgs, candidate_txgs);
    }
}

// ---------------------------------------------------------------------
// 世界：池级状态 + 五条臂，按 Arm 枚举定的固定次序。
// ---------------------------------------------------------------------

const ARM_ORDER: [Arm; 5] = [Arm::ThresholdConservative, Arm::ThresholdNarrow, Arm::IntervalConservative, Arm::IntervalNarrow, Arm::Baseline];

#[derive(Clone, Copy)]
enum PublishKind {
    Overwrite,
    Empty,
}

/// Q1/Q2/Q3/Q11 在一次发布持久之后（这一刻）的快照——第六节四个量各占一行，不做合取。
#[derive(Clone, Copy, Default)]
struct InvariantSnapshot {
    /// Q1：I-3.1（臂内读法）——候选集用 F_生效。
    allocated_statistic_mismatch_by_arm_reading: [bool; 5],
    /// Q2：I-3.1（今天 checker 读法）——候选集用最新根自己带的 F（`current_rollback_floor`）。
    allocated_statistic_mismatch_by_today_checker_reading: [bool; 5],
    /// Q3：I-5.2——item1 + item2 == 容量。
    free_statistic_mismatch: [bool; 5],
    /// Q11：环里最旧有效根的滞后 = 上一次持久根的 txg − 环里最旧有效根的 txg。
    oldest_valid_root_lag: u64,
    /// Q12：最新持久根带的 F（`current_rollback_floor`）> F_生效（跨盘取最小）。
    floor_only_on_one_disk: bool,
    /// Q7：G8′（记账第 1 项 − 第 5 项 == 最新根走读引用，去重求和）判红。
    allocated_minus_deferred_reference_mismatch: [bool; 5],
}

#[derive(Clone)]
struct PublishOutcome {
    txg: u64,
    instance: u64,
    is_nonempty: bool,
    label: &'static str,
    pre: [PreAllocationSnapshot; 5],
    accounting: [AccountingRow; 5],
    violations_candidate: [u64; 5],
    violations_abandoned: [u64; 5],
    invariant_snapshot: InvariantSnapshot,
    /// Q9 轨迹（第四段，见跑前登记第十二节）：这次发布持久之后，每条臂分配记录的条目数
    /// （`arm.ledger.len()`，「曾分配过、尚未被覆盖的落点数」，D3 已定项 7）。
    allocation_record_entries: [u64; 5],
    /// Q9b：这次发布自己的回收步扫描的 pending 记录数（`reads_last_reclaim_step`，每次发布都有一次，
    /// 与是否重建无关——登记第五节「回收步的时点」：每次发布释放与分配之前一步）。
    reclaim_step_reads: [u64; 5],
    /// Q9b：`reads_last_shadow_recompute` 在这次发布产生之前的取值。只有 `is_rebuild_step` 为真时
    /// 才是「重建步」的读数（挂载 / 切换 / 回退重建 / 抬 F 触发的那一次重算）；模型里普通发布之间不
    /// 触发影子账重算（`recompute_shadow` 只在 `rebuild_allocator` 里调用，`isolate_additional_for_raised_floor`
    /// 只在 `raise_floor` 开头调用一次），所以非重建步这个字段是上一次重建留下的陈值，不单独使用。
    shadow_recompute_reads: [u64; 5],
    /// 这次发布是不是紧跟着一次 `rebuild_allocator`（挂载 / 切换 / 回退）或 `raise_floor` 开头那一次
    /// `isolate_additional_for_raised_floor` 之后的第一次发布——Q9b「重建步」的采样点。
    is_rebuild_step: bool,
    /// Q6b：这一刻（这次发布释放与分配之前，与 `pre` 同一时点）环里被抛弃根的 item1 代理值
    /// （`abandoned_root_item1_bounds`：(环里 txg 最大的被抛弃根, 全部被抛弃根里的最大值)）。
    abandoned_item1_bounds: [(Option<u64>, Option<u64>); 5],
    /// Q6b：这一刻「最近一次回退」的 R_old 的 item1 代理值（回退发生时算好、存进 `World`，一直带到下
    /// 一次回退为止）；这条历史从没回退过时是 `None`。
    rollback_target_reference_count: Option<[u64; 5]>,
}

struct World {
    geometry: GeometryCell,
    pool: PoolState,
    arms: [ArmExecutor; 5],
    /// Q6b：最近一次回退的 R_old 的 item1 代理值（登记第十二节修订：用 R_old 自己的引用集合大小当
    /// 「它那一刻的记账第 1 项」，回退选中 R_old 那一刻算好、逐臂各存一份，直到下一次回退才更新）。
    last_rollback_target_reference_count: Option<[u64; 5]>,
}

impl World {
    fn mkfs(geometry: GeometryCell) -> World {
        let mut arms: Vec<ArmExecutor> = ARM_ORDER.iter().map(|&arm| ArmExecutor::new(arm, geometry.placement)).collect();
        for arm in &mut arms {
            arm.mkfs();
        }
        let mut world =
            World { geometry, pool: PoolState::new(geometry.slots_per_region), arms: arms.try_into().unwrap_or_else(|_| unreachable!()), last_rollback_target_reference_count: None };
        world.snapshot_current_txg(); // 第 0 代根（mkfs）自己不经过任何 publish_* 路径，这里补一次。
        world
    }

    fn arm_index(arm: Arm) -> usize {
        ARM_ORDER.iter().position(|candidate| *candidate == arm).expect("Arm 必在 ARM_ORDER 里")
    }

    /// 一次普通发布（O 或 E）：五条臂各自回收步 → 释放 → 分配 → 记账；根槽照常持久（写不失败）。
    fn publish(&mut self, kind: PublishKind) -> PublishOutcome {
        self.publish_inner(kind, false)
    }

    /// 一次根槽写失败的发布：单元与记录照常持久（分配照做），但根槽保留旧内容（ring 不提交这个 txg）。
    fn publish_with_root_slot_failure(&mut self, kind: PublishKind) -> PublishOutcome {
        self.publish_inner(kind, true)
    }

    fn publish_inner(&mut self, kind: PublishKind, root_slot_write_fails: bool) -> PublishOutcome {
        let next_txg = self.pool.current_txg + 1;
        let roles = ordered_roles(&match kind { PublishKind::Overwrite => overwrite_roles(), PublishKind::Empty => EMPTY_PUBLISH_ROLES.to_vec() });
        let is_nonempty = matches!(kind, PublishKind::Overwrite);
        let rollback_floor = self.pool.current_rollback_floor;
        let instance = self.pool.current_instance;

        // 「多扣」与违例检查用的候选 / 被抛弃账：这一刻（这次发布释放与分配之前）的环，
        // 也就是「环」而不是「环′」——p 自己还没有账，它这一刻的内容仍是它前一个根的账，
        // 已经由前一个根自己的 txg 代表，不需要再算一次（登记第一节「多扣」：「这一刻」）。
        // 物理回收步（驱动 occupied / 多扣 / 落点）也用这份「环」门槛——它与记账行公式（环′）不是
        // 同一件事：见 ArmExecutor::drain_formula_pending 的注。
        let candidate_txgs_before: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs_before: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_item1_bounds = abandoned_root_item1_bounds(&self.arms, &abandoned_txgs_before);
        let old_ring_threshold = self.pool.reclaim_threshold();
        let old_ring_blocking = self.pool.interval_blocking_generations();

        // 环′：把 p 的根槽先换成根 p（登记第五节「发布 p 的记账行」），记账三项的公式门槛用这份环。
        // 写失败时不做这一步（ring 保留旧内容）。
        if !root_slot_write_fails {
            self.pool.commit_root(next_txg, instance, rollback_floor, is_nonempty);
        }
        let ring_prime_threshold = self.pool.reclaim_threshold();
        let ring_prime_blocking = self.pool.interval_blocking_generations();

        let mut pre = [PreAllocationSnapshot::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            // 物理回收步（测量臂：每次发布之前一步，门槛用「环」；基线：不在这里回收，回收只在
            // 挂载/切换/回退/抬F）。
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&old_ring_blocking);
                arm.reclaim_measured(old_ring_threshold, blocking);
            }
            let referenced_union: BTreeSet<u64> = {
                let mut set = BTreeSet::new();
                for txg in candidate_txgs_before.iter().chain(abandoned_txgs_before.iter()) {
                    if let Some(account) = arm.root_accounts.get(txg) {
                        set.extend(account.expanded_slots());
                    }
                }
                set
            };
            pre[index] = PreAllocationSnapshot { over_held_slots: arm.occupied.len() as i64 - referenced_union.len() as i64, isolated_slots: arm.isolated.len() as u64 };
        }

        let mut accounting = [AccountingRow::default(); 5];
        let mut violations_candidate = [0u64; 5];
        let mut violations_abandoned = [0u64; 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            for &role in &roles {
                arm.release_role_if_present(role, next_txg);
            }
            let candidate_union: BTreeSet<u64> = candidate_txgs_before.iter().filter_map(|txg| arm.root_accounts.get(txg)).flat_map(|account| account.expanded_slots()).collect();
            let abandoned_union: BTreeSet<u64> = abandoned_txgs_before.iter().filter_map(|txg| arm.root_accounts.get(txg)).flat_map(|account| account.expanded_slots()).collect();
            for &role in &roles {
                let span = role.span_in_slots();
                let start = {
                    arm.allocate_role(role, next_txg);
                    arm.refs[&role]
                };
                for offset in 0..span {
                    let slot = start + offset;
                    if candidate_union.contains(&slot) {
                        violations_candidate[index] += 1;
                    }
                    if abandoned_union.contains(&slot) {
                        violations_abandoned[index] += 1;
                    }
                }
            }
            arm.violations_candidate += violations_candidate[index];
            arm.violations_abandoned += violations_abandoned[index];
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&ring_prime_blocking);
                arm.drain_formula_pending(ring_prime_threshold, blocking);
            }
            accounting[index] = arm.accounting_row();
        }

        let invariant_snapshot = if !root_slot_write_fails {
            for arm in &mut self.arms {
                arm.root_accounts.insert(next_txg, account_from_refs(&arm.refs));
            }
            self.prune_evicted_accounts();
            self.snapshot_current_txg();
            self.invariant_snapshot()
        } else {
            // 写失败：根槽保留旧内容（不进环），但这个 txg 已经「用掉」——切换写行发布的 txg 是它 + 1。
            // 环没有变化，Q1/Q2/Q3/Q11 这一刻没有新状态可判（第一节「合法状态」不含写失败那一刻本身）。
            self.pool.current_txg = next_txg;
            InvariantSnapshot::default()
        };

        let label = if root_slot_write_fails {
            "root_slot_write_failure"
        } else {
            match kind {
                PublishKind::Overwrite => "overwrite",
                PublishKind::Empty => "empty",
            }
        };
        let (allocation_record_entries, reclaim_step_reads, shadow_recompute_reads) = snapshot_allocation_record_read_counters(&self.arms);
        PublishOutcome {
            txg: next_txg,
            instance,
            is_nonempty,
            label,
            pre,
            accounting,
            violations_candidate,
            violations_abandoned,
            invariant_snapshot,
            allocation_record_entries,
            reclaim_step_reads,
            shadow_recompute_reads,
            is_rebuild_step: false,
            abandoned_item1_bounds,
            rollback_target_reference_count: self.last_rollback_target_reference_count,
        }
    }

    /// 剪掉不再「环里自证合法」的历史账快照（ring 里已经没有对应 txg 的条目）。
    fn prune_evicted_accounts(&mut self) {
        let live: BTreeSet<u64> = self.pool.live_roots().iter().map(|entry| entry.txg).collect();
        for arm in &mut self.arms {
            arm.root_accounts.retain(|txg, _| live.contains(txg));
            arm.full_snapshots.retain(|txg, _| live.contains(txg));
        }
    }

    /// 这一刻（最新持久根之后、下一次发布之前）的 Q1/Q2/Q3/Q11 快照。要在
    /// `prune_evicted_accounts()` 之后调用——root_accounts 要是这一刻真正live的那份。
    fn invariant_snapshot(&self) -> InvariantSnapshot {
        let candidate_by_effective_floor: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let floor_of_newest_root = self.pool.current_rollback_floor;
        let candidate_by_newest_root_floor: Vec<u64> = self
            .pool
            .live_roots()
            .iter()
            .filter(|entry| self.pool.is_valid(entry.instance, entry.txg) && entry.txg >= floor_of_newest_root)
            .map(|entry| entry.txg)
            .collect();
        let mut snapshot = InvariantSnapshot::default();
        for (index, arm) in self.arms.iter().enumerate() {
            let row = arm.accounting_row();
            let union_by_effective_floor = candidate_reference_union_size(arm, &candidate_by_effective_floor);
            let union_by_newest_root_floor = candidate_reference_union_size(arm, &candidate_by_newest_root_floor);
            snapshot.allocated_statistic_mismatch_by_arm_reading[index] = row.item1_occupied_slots != union_by_effective_floor;
            snapshot.allocated_statistic_mismatch_by_today_checker_reading[index] = row.item1_occupied_slots != union_by_newest_root_floor;
            snapshot.free_statistic_mismatch[index] = row.item1_occupied_slots + row.item2_free_slots != UNIT_AREA_CAPACITY_SLOTS;
            // G8′（登记第一节）：逐盘 第 1 项 − 第 5 项 == 最新根走读引用（去重求和）。「最新根走读引用」=
            // 这条臂现在的 refs 展开（就是 root_accounts.get(current_txg) 的账，因为 current_txg 这一刻
            // 的账刚由 account_from_refs(&arm.refs) 写入，见 publish_inner/publish_empty_no_roles/
            // publish_write_row_inner 收尾那几行）。
            let newest_reference_size = account_from_refs(&arm.refs).expanded_slots().count() as u64;
            snapshot.allocated_minus_deferred_reference_mismatch[index] = row.item1_occupied_slots.saturating_sub(row.item5_deferred_slots) != newest_reference_size;
        }
        let newest_txg = self.pool.current_txg;
        let oldest_valid = self.pool.oldest_valid_root_txg();
        snapshot.oldest_valid_root_lag = newest_txg.saturating_sub(oldest_valid);
        snapshot.floor_only_on_one_disk = floor_of_newest_root > self.pool.effective_floor();
        snapshot
    }

    /// 给当前 txg 拍一份整臂快照（供回退 / 切换基准恢复）。每个提交点都要调用一次，不能只在
    /// 一批发布跑完之后补一次——那样只会重复拍最后那个 txg，之前的历史帧全部缺失。
    fn snapshot_current_txg(&mut self) {
        let txg = self.pool.current_txg;
        for arm in &mut self.arms {
            let snapshot = arm.full_snapshot();
            arm.full_snapshots.insert(txg, snapshot);
        }
    }

    /// 首次挂载暖机：取号（mkfs 是实例 0，首次挂载取实例 1，不写行，M0.9）、txg 1、2，零单元，
    /// 根照引用 mkfs 的账（M0.4）。取号要发生在两次空发布之前——`crates/singlefs-core/src/mount.rs`
    /// 785-811 的 `establish_instance` 对只做过 mkfs 的池同样先 `acquire_instance` 再暖机；这里若不取号，
    /// `current_instance` 会一直停在 mkfs 的 0，第一次真正的实例切换（回退 / 写失败重发）就会错误地把
    /// 新实例取成 1（应为 2），与登记 M0.9「首次挂载实例 1」不符。
    fn first_mount_warmup(&mut self) -> Vec<PublishOutcome> {
        let instance = self.pool.take_next_instance();
        self.pool.current_instance = instance;
        let mut outcomes = Vec::new();
        for _ in 0..2 {
            outcomes.push(self.publish_empty_no_roles());
        }
        outcomes
    }

    /// 「零单元」的空发布：不触碰任何角色，只推进 txg 并把当前 refs 原样写进新根。
    fn publish_empty_no_roles(&mut self) -> PublishOutcome {
        let next_txg = self.pool.current_txg + 1;
        let rollback_floor = self.pool.current_rollback_floor;
        let instance = self.pool.current_instance;
        // 「多扣」与物理回收步用「环」（这一刻，p 还没有账）；记账公式用「环′」（先换根之后）——
        // 两个不同的环，见 publish_inner 的注。
        let candidate_txgs_before: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs_before: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_item1_bounds = abandoned_root_item1_bounds(&self.arms, &abandoned_txgs_before);
        let old_ring_threshold = self.pool.reclaim_threshold();
        let old_ring_blocking = self.pool.interval_blocking_generations();
        self.pool.commit_root(next_txg, instance, rollback_floor, false);
        let ring_prime_threshold = self.pool.reclaim_threshold();
        let ring_prime_blocking = self.pool.interval_blocking_generations();
        let mut pre = [PreAllocationSnapshot::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&old_ring_blocking);
                arm.reclaim_measured(old_ring_threshold, blocking);
            }
            let referenced_union: BTreeSet<u64> = {
                let mut set = BTreeSet::new();
                for txg in candidate_txgs_before.iter().chain(abandoned_txgs_before.iter()) {
                    if let Some(account) = arm.root_accounts.get(txg) {
                        set.extend(account.expanded_slots());
                    }
                }
                set
            };
            pre[index] = PreAllocationSnapshot { over_held_slots: arm.occupied.len() as i64 - referenced_union.len() as i64, isolated_slots: arm.isolated.len() as u64 };
        }
        let mut accounting = [AccountingRow::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&ring_prime_blocking);
                arm.drain_formula_pending(ring_prime_threshold, blocking);
            }
            accounting[index] = arm.accounting_row();
        }
        for arm in &mut self.arms {
            arm.root_accounts.insert(next_txg, account_from_refs(&arm.refs));
        }
        self.prune_evicted_accounts();
        self.snapshot_current_txg();
        let invariant_snapshot = self.invariant_snapshot();
        let (allocation_record_entries, reclaim_step_reads, shadow_recompute_reads) = snapshot_allocation_record_read_counters(&self.arms);
        PublishOutcome {
            txg: next_txg,
            instance,
            is_nonempty: false,
            label: "warmup",
            pre,
            accounting,
            violations_candidate: [0; 5],
            violations_abandoned: [0; 5],
            invariant_snapshot,
            allocation_record_entries,
            reclaim_step_reads,
            shadow_recompute_reads,
            is_rebuild_step: false,
            abandoned_item1_bounds,
            rollback_target_reference_count: self.last_rollback_target_reference_count,
        }
    }

    /// 暖机：写行发布之后连推 E，直到本实例的根落到两块盘上（A2：至多 2 次）。
    fn warmup_until_both_devices(&mut self) -> Vec<PublishOutcome> {
        let mut outcomes = Vec::new();
        let mut covered: BTreeSet<Device> = BTreeSet::new();
        covered.insert(root_ring_device(self.pool.current_txg));
        while covered.len() < 2 {
            let outcome = self.publish(PublishKind::Empty);
            covered.insert(root_ring_device(outcome.txg));
            outcomes.push(outcome);
        }
        outcomes
    }

    /// 普通挂载（M0.12，无崩溃、无回退）：进程重开、所选根 = 当前状态——`crates/singlefs-core/src/mount.rs`
    /// 889-957 `mount_writable` 在没有崩溃时 `effective_root == chosen_root == 当前根`，`replay_journal`
    /// 施加的记录前缀为空，`rebuilt_allocator` 按挂载读法门槛（`reclaim_floor(F_生效, 最旧有效根)`）重建，
    /// `establish_instance`（`mount.rs` 785-811）给上一个实例写一行 `(instance, T=恢复后生效根的 txg, W, false)`
    /// （mkfs 实例 0 不写行；本模型 W 恒 0，见 M0.9），取新实例，写行发布，暖机到两块盘。
    /// S1(b)(c) 两段历史的「重开取号」那一步靠它：登记第五节 M0.12 定义过这个发布种类，此前没有驱动方法。
    fn plain_remount(&mut self) -> Vec<PublishOutcome> {
        let current_txg = self.pool.current_txg;
        let outgoing_instance = self.pool.current_instance;
        let new_instance = self.pool.take_next_instance();

        let candidate_txgs: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let blocking: BTreeSet<u64> = candidate_txgs.iter().chain(abandoned_txgs.iter()).copied().collect();
        for arm in &mut self.arms {
            let snapshot = arm.full_snapshots.get(&current_txg).cloned().expect("当前 txg 必有整臂快照");
            arm.rebuild_allocator(&snapshot, self.pool.effective_floor().max(self.pool.oldest_valid_root_txg()), Some(&blocking), &abandoned_txgs, &candidate_txgs);
        }
        // 上一个实例的行：mkfs（实例 0）不写行（D18 已定项 11，M0.9）；行表按 InstanceTableRecords 同型累加
        // （`mount.rs` 240-245 的 `.any()` 语义：同一实例多条行取并集里最紧的那个上限）。
        if outgoing_instance >= 1 {
            self.pool.instance_table_rows.push((outgoing_instance, current_txg));
        }
        self.pool.current_instance = new_instance;

        let mut outcomes = self.publish_write_row(current_txg + 1, false);
        outcomes.extend(self.warmup_until_both_devices());
        outcomes
    }

    /// 抬 F：目标 = min(每块盘上最新有效根, 第 4 新的非空有效根)；不足 4 个非空有效根取最旧有效根；
    /// 连推带新 F 的空发布直到每块盘都有，期间测量臂门槛用新 F、基线扣住待放行。
    fn raise_floor(&mut self) -> Vec<PublishOutcome> {
        let candidates = self.pool.candidate_roots();
        let per_device_newest: HashMap<Device, u64> = {
            let mut map: HashMap<Device, u64> = HashMap::new();
            for entry in self.pool.live_roots() {
                if self.pool.is_valid(entry.instance, entry.txg) {
                    let device = root_ring_device(entry.txg);
                    let slot = map.entry(device).or_insert(entry.txg);
                    if entry.txg > *slot {
                        *slot = entry.txg;
                    }
                }
            }
            map
        };
        let ceiling_per_device = per_device_newest.values().copied().min().unwrap_or(0);
        let mut nonempty: Vec<u64> = candidates.iter().filter(|entry| entry.is_nonempty).map(|entry| entry.txg).collect();
        nonempty.sort_unstable();
        let fourth_newest_nonempty = if nonempty.len() >= 4 { nonempty[nonempty.len() - 4] } else { candidates.iter().map(|entry| entry.txg).min().unwrap_or(0) };
        let target_floor = ceiling_per_device.min(fourth_newest_nonempty).max(self.pool.current_rollback_floor);

        // 抬 F 先按目标 F（还没生效）重算影子账（`mount.rs` 609–628「候选集按新 F 缩小……回收之前先按新 F
        // 重算影子账，不然那个槽回收之后就发得出去」）：只对 Narrow 读法追加隔离——保守读法的重算时点
        // 不含 F 触发（第五节「影子账 保守读法」表「重算时点」一行没有 F 这个条件）；只增不清
        // （隔离位在一次挂载之内只增不清，第三节「隔离位在一次挂载之内只增不清」）。
        let candidate_txgs_at_target_floor: Vec<u64> = self
            .pool
            .live_roots()
            .iter()
            .filter(|entry| self.pool.is_valid(entry.instance, entry.txg) && entry.txg >= target_floor)
            .map(|entry| entry.txg)
            .collect();
        let abandoned_txgs_now: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        for arm in &mut self.arms {
            if arm.shadow_kind() == ShadowLedger::Narrow {
                arm.isolate_additional_for_raised_floor(&abandoned_txgs_now, &candidate_txgs_at_target_floor);
            }
        }

        // 先给每条臂按新 F 扣住可回收的槽（withhold_instead_of_free = true）——只有基线这样做：
        // 测量臂的回收门槛用 F_生效（不是目标 F），F_生效 的公式本身要等新 F 在每块盘都持久才生效，
        // 天然不会提前动用还没生效的目标 F。
        for arm in &mut self.arms {
            if !arm.arm.is_measured() {
                arm.reclaim_baseline(target_floor.max(self.pool.oldest_valid_root_txg()), true);
            }
        }
        let mut outcomes = Vec::new();
        let mut covered: BTreeSet<Device> = BTreeSet::new();
        let mut is_first_raise_floor_publish = true; // Q9b「重建步」采样点：抬 F 开头那次影子账重算只发生一次
        loop {
            let next_txg = self.pool.current_txg + 1;
            let instance = self.pool.current_instance;
            let candidate_txgs_before: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
            let abandoned_txgs_before: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
            let abandoned_item1_bounds = abandoned_root_item1_bounds(&self.arms, &abandoned_txgs_before);
            let old_ring_threshold = self.pool.reclaim_threshold();
            let old_ring_blocking = self.pool.interval_blocking_generations();
            self.pool.commit_root(next_txg, instance, target_floor, false);
            let ring_prime_threshold = self.pool.reclaim_threshold();
            let ring_prime_blocking = self.pool.interval_blocking_generations();
            let mut pre = [PreAllocationSnapshot::default(); 5];
            for (index, arm) in self.arms.iter_mut().enumerate() {
                if arm.arm.is_measured() {
                    let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&old_ring_blocking);
                    arm.reclaim_measured(old_ring_threshold, blocking);
                }
                let referenced_union: BTreeSet<u64> = {
                    let mut set = BTreeSet::new();
                    for txg in candidate_txgs_before.iter().chain(abandoned_txgs_before.iter()) {
                        if let Some(account) = arm.root_accounts.get(txg) {
                            set.extend(account.expanded_slots());
                        }
                    }
                    set
                };
                pre[index] = PreAllocationSnapshot { over_held_slots: arm.occupied.len() as i64 - referenced_union.len() as i64, isolated_slots: arm.isolated.len() as u64 };
            }
            for (_index, arm) in self.arms.iter_mut().enumerate() {
                for &role in &EMPTY_PUBLISH_ROLES {
                    arm.release_role_if_present(role, next_txg);
                }
                for &role in &ordered_roles(&EMPTY_PUBLISH_ROLES.to_vec()) {
                    arm.allocate_role(role, next_txg);
                }
            }
            let mut accounting = [AccountingRow::default(); 5];
            for (index, arm) in self.arms.iter_mut().enumerate() {
                if arm.arm.is_measured() {
                    let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&ring_prime_blocking);
                    arm.drain_formula_pending(ring_prime_threshold, blocking);
                }
                accounting[index] = arm.accounting_row();
            }
            for arm in &mut self.arms {
                arm.root_accounts.insert(next_txg, account_from_refs(&arm.refs));
            }
            self.prune_evicted_accounts();
            self.snapshot_current_txg();
            covered.insert(root_ring_device(next_txg));
            let invariant_snapshot = self.invariant_snapshot();
            let (allocation_record_entries, reclaim_step_reads, shadow_recompute_reads) = snapshot_allocation_record_read_counters(&self.arms);
            outcomes.push(PublishOutcome {
                txg: next_txg,
                instance,
                is_nonempty: false,
                label: "raise_floor_empty",
                pre,
                accounting,
                violations_candidate: [0; 5],
                violations_abandoned: [0; 5],
                invariant_snapshot,
                allocation_record_entries,
                reclaim_step_reads,
                shadow_recompute_reads,
                is_rebuild_step: is_first_raise_floor_publish,
                abandoned_item1_bounds,
                rollback_target_reference_count: self.last_rollback_target_reference_count,
            });
            is_first_raise_floor_publish = false;
            if covered.len() >= 2 && self.pool.effective_floor() >= target_floor {
                break;
            }
        }
        for arm in &mut self.arms {
            arm.release_withheld();
        }
        outcomes
    }

    /// 回退到候选集里的某个 txg（由调用方给定，登记里各族历史写死选哪条）。
    fn rollback_to(&mut self, target_txg: u64) -> Vec<PublishOutcome> {
        let target_entry = self.pool.entry_at(target_txg).expect("回退目标必须在环里").clone();
        let target_instance = target_entry.instance;
        let target_table_length = target_entry.instance_table_length_at_publish;
        let abandoned_extended: Vec<u64> = self.pool.abandoned_roots_including_rollback_extension(target_txg, target_instance).iter().map(|entry| entry.txg).collect();
        let new_txg_minus_one = self.pool.live_roots().iter().map(|entry| entry.txg).max().unwrap_or(target_txg);
        let new_instance = self.pool.take_next_instance();

        // 中间实例（在 target 与当前之间、既非 target 也未被上面的扩展抛弃集合覆盖到的实例）：
        // 本实验各族历史没有切换事件夹在回退之间，这个集合恒空，仍按定义算一遍以防将来族用到。
        let mut abandoned_instances: BTreeSet<u64> = BTreeSet::new();
        for entry in self.pool.live_roots() {
            if (entry.txg, entry.instance) > (target_txg, target_instance) && entry.instance != target_instance {
                abandoned_instances.insert(entry.instance);
            }
        }

        // 候选集要按「回退重建那一刻」的实例表算，而回退行此时还没写进 self.pool.instance_table_rows
        // （几行之后才 push）——candidate_roots() 用的是旧表，会把 abandoned_extended 里那些「按旧表
        // 还没被判被抛弃、但一旦回退行写下就会被抛弃」的 txg（包括回退自己抛弃的尾巴）算成候选，
        // 与 mount.rs 的 `isolate_slots_referenced_only_by_abandoned_roots` 用同一个 `is_abandoned`
        // 闭包（含 `extra_abandoned`）过滤候选不同（2026-09-17，S1 对拍第二处发现，见跑前登记第十二节）。
        let candidate_txgs: Vec<u64> =
            self.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| !abandoned_extended.contains(txg)).collect();
        let blocking: BTreeSet<u64> = abandoned_extended.iter().chain(candidate_txgs.iter()).copied().collect();
        for arm in &mut self.arms {
            let snapshot = arm.full_snapshots.get(&target_txg).cloned().expect("回退目标必须有整臂快照");
            arm.rebuild_allocator(&snapshot, self.pool.effective_floor().max(self.pool.oldest_valid_root_txg()), Some(&blocking), &abandoned_extended, &candidate_txgs);
        }

        // Q6b：R_old 的 item1 代理值——回退选中 R_old 那一刻算好存住（`abandoned_root_item1_bounds`
        // 同一份逻辑：用 R_old 自己的引用集合大小当它那一刻的 item1）；target_txg 此刻仍在环里，
        // root_accounts 必有它（rebuild_allocator 不碰 root_accounts）。
        let mut rollback_target_reference_counts = [0u64; 5];
        for (index, arm) in self.arms.iter().enumerate() {
            rollback_target_reference_counts[index] = arm.root_accounts.get(&target_txg).expect("回退目标必有账").expanded_slots().count() as u64;
        }
        self.last_rollback_target_reference_count = Some(rollback_target_reference_counts);

        // 在 R_old 指着的那一版实例表上写回退行：先截断回 R_old 发布时的表长（丢掉之后新加的行）。
        self.pool.instance_table_rows.truncate(target_table_length);
        self.pool.instance_table_rows.push((target_instance, target_txg));
        for instance in &abandoned_instances {
            self.pool.instance_table_rows.push((*instance, 0));
        }
        self.pool.current_txg = new_txg_minus_one;
        self.pool.current_instance = new_instance;

        let mut outcomes = self.publish_write_row(new_txg_minus_one + 1, false);
        outcomes.extend(self.warmup_until_both_devices());
        outcomes
    }

    /// 写行发布：txg 由调用方给定（挂载恢复 / 切换 / 回退各自算好）；`redo_overwrite` 为真时
    /// 同一次发布再加 O 的四个文件角色（实例切换里被重发的在飞发布是 O 的情形）。
    fn publish_write_row(&mut self, txg: u64, redo_overwrite: bool) -> Vec<PublishOutcome> {
        self.publish_write_row_inner(txg, redo_overwrite, false)
    }

    /// H3² 用：写行发布自己的根槽也写失败一次（连续两次失败，第五节 H3² 定义）。
    fn publish_write_row_with_root_slot_failure(&mut self, txg: u64, redo_overwrite: bool) -> Vec<PublishOutcome> {
        self.publish_write_row_inner(txg, redo_overwrite, true)
    }

    fn publish_write_row_inner(&mut self, txg: u64, redo_overwrite: bool, root_slot_write_fails: bool) -> Vec<PublishOutcome> {
        assert_eq!(txg, self.pool.current_txg + 1, "写行发布的 txg 必须紧接当前 txg");
        let roles = ordered_roles(&if redo_overwrite { write_row_with_redo_overwrite_roles() } else { write_row_roles() });
        let rollback_floor = self.pool.current_rollback_floor;
        let instance = self.pool.current_instance;
        let candidate_txgs_before: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs_before: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_item1_bounds = abandoned_root_item1_bounds(&self.arms, &abandoned_txgs_before);
        let old_ring_threshold = self.pool.reclaim_threshold();
        let old_ring_blocking = self.pool.interval_blocking_generations();
        if !root_slot_write_fails {
            self.pool.commit_root(txg, instance, rollback_floor, redo_overwrite);
        }
        let ring_prime_threshold = self.pool.reclaim_threshold();
        let ring_prime_blocking = self.pool.interval_blocking_generations();
        let mut pre = [PreAllocationSnapshot::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&old_ring_blocking);
                arm.reclaim_measured(old_ring_threshold, blocking);
            }
            let referenced_union: BTreeSet<u64> = {
                let mut set = BTreeSet::new();
                for txg in candidate_txgs_before.iter().chain(abandoned_txgs_before.iter()) {
                    if let Some(account) = arm.root_accounts.get(txg) {
                        set.extend(account.expanded_slots());
                    }
                }
                set
            };
            pre[index] = PreAllocationSnapshot { over_held_slots: arm.occupied.len() as i64 - referenced_union.len() as i64, isolated_slots: arm.isolated.len() as u64 };
        }
        for arm in &mut self.arms {
            for &role in &roles {
                arm.release_role_if_present(role, txg);
            }
            for &role in &roles {
                arm.allocate_role(role, txg);
            }
        }
        let mut accounting = [AccountingRow::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&ring_prime_blocking);
                arm.drain_formula_pending(ring_prime_threshold, blocking);
            }
            accounting[index] = arm.accounting_row();
        }
        let invariant_snapshot = if !root_slot_write_fails {
            for arm in &mut self.arms {
                arm.root_accounts.insert(txg, account_from_refs(&arm.refs));
            }
            self.prune_evicted_accounts();
            self.snapshot_current_txg();
            self.invariant_snapshot()
        } else {
            self.pool.current_txg = txg;
            InvariantSnapshot::default()
        };
        let label = if root_slot_write_fails { "write_row_root_slot_write_failure" } else { "write_row" };
        // 写行发布的调用方（`rollback_to` / `switch_rebuild_and_take_instance` / `plain_remount`）都是先
        // `rebuild_allocator` 再调这个函数——写行发布永远是紧跟重建的那一次，Q9b「重建步」采样点。
        let (allocation_record_entries, reclaim_step_reads, shadow_recompute_reads) = snapshot_allocation_record_read_counters(&self.arms);
        vec![PublishOutcome {
            txg,
            instance,
            is_nonempty: redo_overwrite,
            label,
            pre,
            accounting,
            violations_candidate: [0; 5],
            violations_abandoned: [0; 5],
            invariant_snapshot,
            allocation_record_entries,
            reclaim_step_reads,
            shadow_recompute_reads,
            is_rebuild_step: true,
            abandoned_item1_bounds,
            rollback_target_reference_count: self.last_rollback_target_reference_count,
        }]
    }

    /// 切换的共有前半段：从 `allocator_basis_txg` 那一版重建分配账（M0.14「分配账退回失败那次所基于
    /// 的根那一版」）、取新实例；被换下的实例的行写 `(outgoing_row_txg)`——正常切换时是它最后成功发布
    /// 的 txg，H3² 的第二次切换时是 0（换下的实例一次都没发布成功，「没发过根的那个实例行 (i, 0, 0)」）。
    fn switch_rebuild_and_take_instance(&mut self, allocator_basis_txg: u64, outgoing_row_txg: u64) {
        let candidate_txgs: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let threshold = self.pool.reclaim_threshold();
        let blocking: BTreeSet<u64> = abandoned_txgs.iter().chain(candidate_txgs.iter()).copied().collect();
        for arm in &mut self.arms {
            let snapshot = arm.full_snapshots.get(&allocator_basis_txg).cloned().expect("切换基准必须有整臂快照");
            arm.rebuild_allocator(&snapshot, threshold, Some(&blocking), &abandoned_txgs, &candidate_txgs);
        }
        let new_instance = self.pool.take_next_instance();
        self.pool.instance_table_rows.push((self.pool.current_instance, outgoing_row_txg));
        self.pool.current_instance = new_instance;
    }

    /// 根槽写失败之后的实例切换：分配账退回失败那次所基于的根那一版（M0.14），取新实例、写行
    /// （redo 失败那次若是 O 就带上 O 的四个文件角色），暖机。
    fn switch_after_write_failure(&mut self, basis_txg: u64, redo_overwrite: bool) -> Vec<PublishOutcome> {
        self.switch_rebuild_and_take_instance(basis_txg, basis_txg);
        // current_txg 保持不变：写行发布的 txg = 失败那次的 txg + 1（M0.10「切换 = 失败那次的 txg + 1」）。
        let mut outcomes = self.publish_write_row(self.pool.current_txg + 1, redo_overwrite);
        outcomes.extend(self.warmup_until_both_devices());
        outcomes
    }

    /// H3²：切换自己的写行发布（重发那个 W）也根槽写失败一次，再切一次（D23（journal 的角色与格式）
    /// 已定项 14「连续两次失败：W 的根槽也写失败，再切一次，txg 再 + 1，没发过根的那个实例行 (i, 0, 0)」）。
    fn switch_after_write_failure_twice(&mut self, basis_txg: u64, redo_overwrite: bool) -> Vec<PublishOutcome> {
        self.switch_rebuild_and_take_instance(basis_txg, basis_txg);
        let mut outcomes = self.publish_write_row_with_root_slot_failure(self.pool.current_txg + 1, redo_overwrite);
        // 第一次切换取的新实例一次都没发布成功：它的行是 (i, 0)，不是 (i, basis_txg)。
        self.switch_rebuild_and_take_instance(basis_txg, 0);
        outcomes.extend(self.publish_write_row(self.pool.current_txg + 1, redo_overwrite));
        outcomes.extend(self.warmup_until_both_devices());
        outcomes
    }

    /// 工作负载段：连续 `length` 次发布，段内第 k 次（k 从 0 起）是 O ⟺ k mod period == 0
    /// （period=1 ⇒ ρ=1，period=4 ⇒ ρ=1/4）。
    fn run_workload_segment(&mut self, length: u64, period: u64) -> Vec<PublishOutcome> {
        let mut outcomes = Vec::with_capacity(length as usize);
        for step in 0..length {
            let kind = if step % period == 0 { PublishKind::Overwrite } else { PublishKind::Empty };
            outcomes.push(self.publish(kind));
        }
        outcomes
    }

    // -----------------------------------------------------------------
    // 第五节「历史族、几何格与支线」表列出的族，第三段新增 H2/H3φ/H3²/H5早/H5晚；
    // H1/H4/H6 也在这里重写成方法，供第三段统一的 Q 系列扫描调用（此前只在 main()/单测里各写一份）。
    // -----------------------------------------------------------------

    /// H1：工作负载段 6N 次。
    fn history_continuous_overwrite(&mut self) -> Vec<PublishOutcome> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        self.run_workload_segment(6 * ring_capacity, period)
    }

    /// H2：工作负载段 3N 次 → 普通挂载（恢复、W、暖机）→ 工作负载段 3N 次。
    fn history_plain_remount_midway(&mut self) -> Vec<PublishOutcome> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        let mut outcomes = self.run_workload_segment(3 * ring_capacity, period);
        outcomes.extend(self.plain_remount());
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        outcomes
    }

    /// H3φ / H3²：工作负载段 3N+φ 次 → 下一次发布（按段内序号定 O/E）根槽写失败一次 → 实例切换
    /// （`twice` 为真时切换自己的写行也失败一次，再切一次，即 H3²，φ 恒 0）→ 工作负载段 3N 次。
    fn history_root_slot_write_failure_and_switch(&mut self, phi: u64, twice: bool) -> Vec<PublishOutcome> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        let mut outcomes = self.run_workload_segment(3 * ring_capacity + phi, period);
        let basis_txg = self.pool.current_txg;
        let next_step_index = 3 * ring_capacity + phi;
        let kind = if next_step_index % period == 0 { PublishKind::Overwrite } else { PublishKind::Empty };
        let redo_overwrite = matches!(kind, PublishKind::Overwrite);
        outcomes.push(self.publish_with_root_slot_failure(kind));
        if twice {
            outcomes.extend(self.switch_after_write_failure_twice(basis_txg, redo_overwrite));
        } else {
            outcomes.extend(self.switch_after_write_failure(basis_txg, redo_overwrite));
        }
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        outcomes
    }

    /// H4 的前半段（H4d 自己与 H5 共用）：工作负载段 3N 次 → 回退到「txg ≤ 最新根 txg − d 的候选根里
    /// txg 最大的那条」。**不含 H4d 自己的尾段工作负载**——H5早 / H5晚 紧接着要在这条根还活在环里
    /// （`target` / `write_row_txg` 仍是候选）的时候做第二次回退；若把 H4d 自己的尾段 3N 也跑一遍再
    /// 返回，环早已转过好几圈，`target` 与 `write_row_txg` 必然已被轮转挤出候选集——H5早 / H5晚 的
    /// 第二次回退在任何几何格上都找不到候选，两族 24 行 q_family 全部 `q13_no_candidate=true`
    /// （2026-09-17，第四段，见跑前登记第十二节；此前 `history_single_rollback` 自带尾段 3N，
    /// H5 两族借它当「H4（d=S）」前缀时把这段也借了进去）。写死的回退目标不在候选集里（第十一节
    /// V8「无候选」）时返回 `None`。返回值另带 (R_old 的 txg, 回退那次写行发布的 txg)。
    fn history_rollback_prefix(&mut self, rollback_depth: u64) -> Option<(Vec<PublishOutcome>, u64, u64)> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        let mut outcomes = self.run_workload_segment(3 * ring_capacity, period);
        let newest_txg = self.pool.current_txg;
        let ceiling = newest_txg.saturating_sub(rollback_depth);
        let target = self.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| *txg <= ceiling).max()?;
        let rollback_outcomes = self.rollback_to(target);
        let write_row_txg = rollback_outcomes.iter().find(|outcome| outcome.label == "write_row").map(|outcome| outcome.txg).unwrap_or(0);
        outcomes.extend(rollback_outcomes);
        Some((outcomes, target, write_row_txg))
    }

    /// H4d：`history_rollback_prefix` 之后再接工作负载段 3N 次（第五节 H4d 的完整定义）。写死的回退
    /// 目标不在候选集里（第十一节 V8「无候选」）时返回 `None`，调用方单列条数（Q13）。
    fn history_single_rollback(&mut self, rollback_depth: u64) -> Option<(Vec<PublishOutcome>, u64, u64)> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        let (mut outcomes, target, write_row_txg) = self.history_rollback_prefix(rollback_depth)?;
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some((outcomes, target, write_row_txg))
    }

    /// H5早：H4（d=S）的前半段（`history_rollback_prefix`，不含 H4d 自己的尾段 3N）→ 工作负载段 1 次
    /// → 第二次回退到「txg 小于第一次 R_old 的候选根里 txg 最大的那条」→ 工作负载段 3N 次。任一次回退
    /// 的目标不在候选集里时返回 `None`。
    fn history_double_rollback_shallower_second(&mut self) -> Option<Vec<PublishOutcome>> {
        let ring_capacity = self.geometry.ring_capacity();
        let slots_per_region = self.geometry.slots_per_region;
        let period = self.geometry.workload_period;
        let (mut outcomes, first_target, _first_write_row_txg) = self.history_rollback_prefix(slots_per_region)?;
        outcomes.extend(self.run_workload_segment(1, period));
        let second_target = self.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| *txg < first_target).max()?;
        outcomes.extend(self.rollback_to(second_target));
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some(outcomes)
    }

    /// H5晚：H4（d=S）的前半段（`history_rollback_prefix`，不含 H4d 自己的尾段 3N）→ 工作负载段 S 次
    /// → 第二次回退到「第一次回退那条 W 的根（新时间线上最早的根）」→ 工作负载段 3N 次。那条根不再是
    /// 候选（已经被抬 F / 轮转挤出）时返回 `None`。
    fn history_double_rollback_to_first_rollback_write_row(&mut self) -> Option<Vec<PublishOutcome>> {
        let ring_capacity = self.geometry.ring_capacity();
        let slots_per_region = self.geometry.slots_per_region;
        let period = self.geometry.workload_period;
        let (mut outcomes, _first_target, first_write_row_txg) = self.history_rollback_prefix(slots_per_region)?;
        outcomes.extend(self.run_workload_segment(slots_per_region, period));
        let still_candidate = self.pool.candidate_roots().iter().any(|entry| entry.txg == first_write_row_txg);
        if !still_candidate {
            return None;
        }
        outcomes.extend(self.rollback_to(first_write_row_txg));
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some(outcomes)
    }

    /// H6：工作负载段 3N 次 → 抬 F → 工作负载段 S 次 → 回退到候选集里 txg 最小的那条根 → 工作负载段
    /// 3N 次。抬 F 之后候选集为空（不该发生，环里至少有刚抬完 F 的那条根）时返回 `None`。
    fn history_raise_floor_then_rollback(&mut self) -> Option<Vec<PublishOutcome>> {
        let ring_capacity = self.geometry.ring_capacity();
        let slots_per_region = self.geometry.slots_per_region;
        let period = self.geometry.workload_period;
        let mut outcomes = self.run_workload_segment(3 * ring_capacity, period);
        outcomes.extend(self.raise_floor());
        outcomes.extend(self.run_workload_segment(slots_per_region, period));
        let target = self.pool.candidate_roots().iter().map(|entry| entry.txg).min()?;
        outcomes.extend(self.rollback_to(target));
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some(outcomes)
    }

    /// H7：H4（d=N−2）的前半段（`history_rollback_prefix`，不含 H4dNm2 自己的尾段 3N——理由与 H5早 /
    /// H5晚 同：H7 的抬 F 要在回退那条根还活在环里、且回退带来的被抛弃根仍在被隔离的时候发生，
    /// 才测得到「回退 + 抬 F 交错」；若先把 H4dNm2 自己的尾段 3N 跑完，回退那条根早已被轮转挤出，
    /// 抬 F 面对的就是一段与单纯 H1 无异的稳态历史）→ 工作负载段 S 次 → 抬 F → 工作负载段 3N 次。
    /// 回退目标不在候选集里时返回 `None`（抬 F 本身不失败，见 `raise_floor` 的实现）。
    fn history_rollback_deep_then_raise_floor(&mut self) -> Option<Vec<PublishOutcome>> {
        let ring_capacity = self.geometry.ring_capacity();
        let slots_per_region = self.geometry.slots_per_region;
        let period = self.geometry.workload_period;
        let rollback_depth = ring_capacity - 2; // 与 H4dNm2 同一个回退深度（登记第五节 H4dNm2 = H4，d = N-2）
        let (mut outcomes, _target, _write_row_txg) = self.history_rollback_prefix(rollback_depth)?;
        outcomes.extend(self.run_workload_segment(slots_per_region, period));
        outcomes.extend(self.raise_floor());
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some(outcomes)
    }
}

/// 首次挂载暖机 + 第一个事务（txg 3）：产出五条臂的执行器，供 K1/A6 现算用。
fn first_transaction(cell: GeometryCell) -> World {
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {}
    world.publish(PublishKind::Overwrite);
    world
}

/// Q8 基底 β1（登记第五节「坏镜像」）：H1 主历史最后一次发布之后。
fn beta_one_world(cell: GeometryCell) -> World {
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {}
    let _ = world.history_continuous_overwrite();
    world
}

/// Q8 基底 β2：H4（d = N − 2）主历史回退之后第 S 次工作负载发布之后（环里还有被抛弃根）。
/// 与 `history_rollback_deep_then_raise_floor`（H7）共用同一段前缀（`history_rollback_prefix`），
/// 但只跑 S 次工作负载、不抬 F——β2 要的是「刚回退完、正常发布几次」这个状态，不是 H7 那条完整历史。
/// 回退目标不在候选集里时返回 `None`（第十一节 V8「无候选」，H4dNm2 在全部 12 格历史上都有候选，
/// 第四段产物已验证过：`grep -c q13_no_candidate` 命中 0 次）。
fn beta_two_world(cell: GeometryCell) -> Option<World> {
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {}
    let rollback_depth = cell.ring_capacity() - 2;
    world.history_rollback_prefix(rollback_depth)?;
    let _ = world.run_workload_segment(cell.slots_per_region, cell.workload_period);
    Some(world)
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config unit_area_capacity_slots={UNIT_AREA_CAPACITY_SLOTS} ring_regions={ROOT_RING_REGIONS} record_width_single={RECORD_WIDTH_BYTES_SINGLE_GENERATION} record_width_dual={RECORD_WIDTH_BYTES_DUAL_GENERATION} records_per_node_single={RECORDS_PER_NODE_SINGLE_GENERATION} records_per_node_dual={RECORDS_PER_NODE_DUAL_GENERATION} model=counting file_ops=0"
        ))
    );
    println!(
        "{}",
        emitter.emit_raw(
            "name=scope covers=K1,K2,K3,K4,K5,A1,A2,A3,A4,A5,A6,A7,U1..U15,Q1,Q2,Q3,Q4i,Q4ii,Q5,Q6,Q6b(甲/乙两读法),Q7(G8′),Q9a(峰值/均值/末值),Q9b(峰值/均值/末值，发布前步/重建步分开),Q10,Q11,Q12,Q13(H1,H2,H3phi0/1/2,H3sq,H4d2,H4dNm2,H5early,H5late,H6,H7,12格),Q8(部分：B1a/B1b/B2/Bk1/Bk2，β1/β2 两基底) missing=Q8(部分：B3a/B3b/B4a/B4b/B4c 未实现，需要重新跑一遍历史（回收步一律不回收）或强制落点碰撞，本段未建），crash_branches(c1/c2/L2/L3，登记估过是千万发布级，本段未建：需要一个与 plain_remount 不同的『施加一条 dangling 记录』恢复路径，语义复杂度与规模都超出本段能安全完成的范围，量级见下） note=第五段新增 Q6b/Q7/Q10 与 Q8 的 B1a/B1b/B2/Bk1/Bk2 五种坏镜像（20 条新单测、10 条新变异，全部先证明会红）；G8′、Q10 推法分歧、Q6b 上界不成立三项发现见实验页「产物里的数」。崩溃支线（L2/L3）与 Q8 剩余三种坏镜像（B3a/B3b/B4a/B4b/B4c）留给下一段：前者需要新的恢复函数与千万级发布量的可行性评估（见跑前登记第十二节第五段），后者需要「重跑历史且禁用回收」与「强制落点碰撞」两种新机制"
        )
    );

    // K1 / A6：第一个事务（P-impl，S=8，六格里报 S=8 一格；六格数值相同，K1 的单测覆盖了另外五格）。
    let first_transaction_cell = BASE_GEOMETRY;
    let world = first_transaction(first_transaction_cell);
    let arm = &world.arms[World::arm_index(Arm::ThresholdConservative)];
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=k1_first_transaction_placement s={} data_unit={} extent_root={} inode_leaf={} inode_root={} alloc_tree={} acct_tree={} central_map_tree={} table_unit={} slot_50241_free={}",
            first_transaction_cell.slots_per_region,
            arm.refs[&Role::DataUnit],
            arm.refs[&Role::ExtentRoot],
            arm.refs[&Role::InodeLeaf],
            arm.refs[&Role::InodeRoot],
            arm.refs[&Role::AllocationTree],
            arm.refs[&Role::AcctTree],
            arm.refs[&Role::CentralMapTree],
            arm.refs[&Role::TableUnit],
            arm.is_free(50241)
        ))
    );
    for arm in &world.arms {
        let row = arm.accounting_row();
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=a6_first_transaction_accounting arm={} item1={} item5={} item2={}",
                arm.arm.tag(),
                row.item1_occupied_slots,
                row.item5_deferred_slots,
                row.item2_free_slots
            ))
        );
    }

    // A4 / K4 / K5：对全部 12 格现算一遍，逐格写一行（与单测钉的绝对值同一批公式，产物给的是 12 格全量）。
    for cell in all_geometry_cells() {
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {}
        let ring_capacity = cell.ring_capacity();
        let mut last_overwrite_txg = None;
        let mut last_overwrite_accounting = [AccountingRow::default(); 5];
        for outcome in world.run_workload_segment(6 * ring_capacity, cell.workload_period) {
            if outcome.is_nonempty {
                last_overwrite_txg = Some(outcome.txg);
                last_overwrite_accounting = outcome.accounting;
            }
        }
        let measured_index = World::arm_index(Arm::ThresholdConservative);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=a4_steady_state s={} placement={:?} period={} last_overwrite_txg={} item1={} item5={}",
                cell.slots_per_region,
                cell.placement,
                cell.workload_period,
                last_overwrite_txg.unwrap_or(0),
                last_overwrite_accounting[measured_index].item1_occupied_slots,
                last_overwrite_accounting[measured_index].item5_deferred_slots
            ))
        );
    }

    // K4：全部 12 格现算一遍回退（d = 2），报写行发布 txg 与暖机次数。
    for cell in all_geometry_cells() {
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {}
        let ring_capacity = cell.ring_capacity();
        for _ in world.run_workload_segment(3 * ring_capacity, cell.workload_period) {}
        let newest_txg = world.pool.current_txg;
        let target_ceiling = newest_txg - 2;
        let target_txg = world.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| *txg <= target_ceiling).max();
        let Some(target_txg) = target_txg else {
            println!("{}", emitter.emit_raw(&format!("name=k4_rollback s={} placement={:?} period={} no_candidate=true", cell.slots_per_region, cell.placement, cell.workload_period)));
            continue;
        };
        let outcomes = world.rollback_to(target_txg);
        let write_row_txg = outcomes.iter().find(|outcome| outcome.label == "write_row").map(|outcome| outcome.txg).unwrap_or(0);
        let warm_up_publish_count = outcomes.iter().filter(|outcome| outcome.label == "empty").count();
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=k4_rollback s={} placement={:?} period={} write_row_txg={} warm_up_publish_count={}",
                cell.slots_per_region, cell.placement, cell.workload_period, write_row_txg, warm_up_publish_count
            ))
        );
    }

    // K5：ρ = 1 的 6 格现算一遍抬 F。
    for slots_per_region in [4u64, 8, 16] {
        for placement in [PlacementPolicy::Impl, PlacementPolicy::Low] {
            let cell = GeometryCell { slots_per_region, placement, workload_period: 1 };
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let ring_capacity = cell.ring_capacity();
            for _ in world.run_workload_segment(3 * ring_capacity, 1) {}
            let outcomes = world.raise_floor();
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=k5_raise_floor s={slots_per_region} placement={placement:?} target_floor={} publish_count={}",
                    world.pool.current_rollback_floor,
                    outcomes.len()
                ))
            );
        }
    }

    // 第四段：H1、H2、H3φ(0/1/2)、H3²、H4(d=2)、H4(d=N-2)、H5早、H5晚、H6、H7，全部 12 格，
    // Q1-Q6、Q9(峰值/均值/末值轨迹)、Q11-Q13。崩溃支线（L2/L3）、G8′/坏镜像矩阵（Q7/Q8/Q10）留到第五段
    // （见跑前登记第十二节第四段与实验页「它答不了的」）。
    let families: [(&str, fn(&mut World) -> Option<Vec<PublishOutcome>>); 9] = [
        ("H1", |world| Some(world.history_continuous_overwrite())),
        ("H2", |world| Some(world.history_plain_remount_midway())),
        ("H3phi0", |world| Some(world.history_root_slot_write_failure_and_switch(0, false))),
        ("H3phi1", |world| Some(world.history_root_slot_write_failure_and_switch(1, false))),
        ("H3phi2", |world| Some(world.history_root_slot_write_failure_and_switch(2, false))),
        ("H3sq", |world| Some(world.history_root_slot_write_failure_and_switch(0, true))),
        ("H4d2", |world| world.history_single_rollback(2).map(|(outcomes, _, _)| outcomes)),
        ("H4dNm2", |world| { let rollback_depth = world.geometry.ring_capacity() - 2; world.history_single_rollback(rollback_depth).map(|(outcomes, _, _)| outcomes) }),
        ("H5early", |world| world.history_double_rollback_shallower_second()),
    ];
    let families_needing_first_rollback_target: [(&str, fn(&mut World) -> Option<Vec<PublishOutcome>>); 3] = [
        ("H5late", |world| world.history_double_rollback_to_first_rollback_write_row()),
        ("H6", |world| world.history_raise_floor_then_rollback()),
        ("H7", |world| world.history_rollback_deep_then_raise_floor()),
    ];
    for cell in all_geometry_cells() {
        for (family, runner) in families.iter().chain(families_needing_first_rollback_target.iter()) {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {
            }
            let ring_capacity = cell.ring_capacity();
            match runner(&mut world) {
                None => {
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=q_family family={family} s={} placement={:?} period={} q13_no_candidate=true",
                            cell.slots_per_region, cell.placement, cell.workload_period
                        ))
                    );
                }
                Some(outcomes) => {
                    // Q9a：五条臂各自的条目数轨迹（峰值/均值/末值），供下面算「多写」用。
                    let mut entries_peak_by_index = [0u64; 5];
                    let mut entries_mean_by_index = [0.0f64; 5];
                    for index in 0..5 {
                        let arm_tag = ARM_ORDER[index].tag();
                        let mut allocated_mismatch_arm_reading_count = 0u64;
                        let mut allocated_mismatch_today_reading_count = 0u64;
                        let mut free_mismatch_count = 0u64;
                        let mut allocated_minus_deferred_reference_mismatch_count = 0u64;
                        let mut over_held_peak = i64::MIN;
                        let mut over_held_positive_publishes = 0u64;
                        let mut isolated_peak = 0u64;
                        let mut isolated_positive_publishes = 0u64;
                        let mut oldest_valid_root_lag_peak = 0u64;
                        let mut oldest_valid_root_lag_at_or_above_ring_capacity_publishes = 0u64;
                        let mut floor_only_on_one_disk_publishes = 0u64;
                        for outcome in &outcomes {
                            if outcome.invariant_snapshot.allocated_statistic_mismatch_by_arm_reading[index] {
                                allocated_mismatch_arm_reading_count += 1;
                            }
                            if outcome.invariant_snapshot.allocated_statistic_mismatch_by_today_checker_reading[index] {
                                allocated_mismatch_today_reading_count += 1;
                            }
                            if outcome.invariant_snapshot.free_statistic_mismatch[index] {
                                free_mismatch_count += 1;
                            }
                            if outcome.invariant_snapshot.allocated_minus_deferred_reference_mismatch[index] {
                                allocated_minus_deferred_reference_mismatch_count += 1;
                            }
                            let over_held = outcome.pre[index].over_held_slots;
                            over_held_peak = over_held_peak.max(over_held);
                            if over_held > 0 {
                                over_held_positive_publishes += 1;
                            }
                            let isolated = outcome.pre[index].isolated_slots;
                            isolated_peak = isolated_peak.max(isolated);
                            if isolated > 0 {
                                isolated_positive_publishes += 1;
                            }
                            oldest_valid_root_lag_peak = oldest_valid_root_lag_peak.max(outcome.invariant_snapshot.oldest_valid_root_lag);
                            if outcome.invariant_snapshot.oldest_valid_root_lag >= ring_capacity {
                                oldest_valid_root_lag_at_or_above_ring_capacity_publishes += 1;
                            }
                            if outcome.invariant_snapshot.floor_only_on_one_disk {
                                floor_only_on_one_disk_publishes += 1;
                            }
                        }
                        let over_held_end = outcomes.last().map_or(0, |outcome| outcome.pre[index].over_held_slots);
                        let isolated_end = outcomes.last().map_or(0, |outcome| outcome.pre[index].isolated_slots);
                        let oldest_valid_root_lag_end = outcomes.last().map_or(0, |outcome| outcome.invariant_snapshot.oldest_valid_root_lag);

                        // Q9a：条目数（两盘合计不适用于本模型——落点跨两盘同槽，条目按 M0.1 只在单设备
                        // 视角下数一遍，见 `LedgerEntry`；这里报的是单份账本条目数）、条目字节、叶节点数。
                        let arm = ARM_ORDER[index];
                        let record_width_bytes = arm.record_width_bytes();
                        let records_per_node = arm.records_per_node();
                        let entries_series: Vec<u64> = outcomes.iter().map(|outcome| outcome.allocation_record_entries[index]).collect();
                        let derived_series: Vec<(u64, u64)> =
                            entries_series.iter().map(|entries| allocation_record_bytes_and_leaf_nodes(*entries, record_width_bytes, records_per_node)).collect();
                        let bytes_series: Vec<u64> = derived_series.iter().map(|(bytes, _)| *bytes).collect();
                        let leaf_nodes_series: Vec<u64> = derived_series.iter().map(|(_, leaf_nodes)| *leaf_nodes).collect();
                        let (entries_peak, entries_mean, entries_end) = peak_mean_end(&entries_series);
                        let (bytes_peak, bytes_mean, bytes_end) = peak_mean_end(&bytes_series);
                        let (leaf_nodes_peak, leaf_nodes_mean, leaf_nodes_end) = peak_mean_end(&leaf_nodes_series);
                        entries_peak_by_index[index] = entries_peak;
                        entries_mean_by_index[index] = entries_mean;

                        // Q9b：「发布前步」= 每次发布都有的回收步（`reclaim_step_reads`，登记第五节「回收步
                        // 的时点」第一句），逐条采样；「重建步」= 紧跟 `rebuild_allocator` / `raise_floor`
                        // 开头那次重算的采样点（`is_rebuild_step`），样本数可能为 0（这一族历史没有触发
                        // 挂载/切换/回退/抬F）。
                        let reclaim_reads_series: Vec<u64> = outcomes.iter().map(|outcome| outcome.reclaim_step_reads[index]).collect();
                        let (reclaim_reads_peak, reclaim_reads_mean, reclaim_reads_end) = peak_mean_end(&reclaim_reads_series);
                        let rebuild_shadow_reads_series: Vec<u64> =
                            outcomes.iter().filter(|outcome| outcome.is_rebuild_step).map(|outcome| outcome.shadow_recompute_reads[index]).collect();
                        let rebuild_step_count = rebuild_shadow_reads_series.len() as u64;
                        let (rebuild_shadow_reads_peak, rebuild_shadow_reads_mean, rebuild_shadow_reads_end) = peak_mean_end(&rebuild_shadow_reads_series);

                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=q_family family={family} s={} placement={:?} period={} arm={arm_tag} publishes={} \
                                 q1_i31_arm_red_states={allocated_mismatch_arm_reading_count} q2_i31_today_red_states={allocated_mismatch_today_reading_count} q3_i52_red_states={free_mismatch_count} \
                                 q7_g8prime_red_states={allocated_minus_deferred_reference_mismatch_count} \
                                 q4i_violations_candidate={} q4ii_violations_abandoned={} \
                                 q5_over_held_peak={over_held_peak} q5_over_held_positive_publishes={over_held_positive_publishes} q5_over_held_end={over_held_end} \
                                 q6_isolated_peak={isolated_peak} q6_isolated_positive_publishes={isolated_positive_publishes} q6_isolated_end={isolated_end} \
                                 q9a_entries_peak={entries_peak} q9a_entries_mean={entries_mean:.3} q9a_entries_end={entries_end} \
                                 q9a_bytes_peak={bytes_peak} q9a_bytes_mean={bytes_mean:.3} q9a_bytes_end={bytes_end} \
                                 q9a_leaf_nodes_peak={leaf_nodes_peak} q9a_leaf_nodes_mean={leaf_nodes_mean:.3} q9a_leaf_nodes_end={leaf_nodes_end} \
                                 q9b_publish_reclaim_reads_peak={reclaim_reads_peak} q9b_publish_reclaim_reads_mean={reclaim_reads_mean:.3} q9b_publish_reclaim_reads_end={reclaim_reads_end} \
                                 q9b_rebuild_shadow_reads_peak={rebuild_shadow_reads_peak} q9b_rebuild_shadow_reads_mean={rebuild_shadow_reads_mean:.3} q9b_rebuild_shadow_reads_end={rebuild_shadow_reads_end} q9b_rebuild_step_count={rebuild_step_count} \
                                 q11_lag_peak={oldest_valid_root_lag_peak} q11_lag_ge_n_publishes={oldest_valid_root_lag_at_or_above_ring_capacity_publishes} q11_lag_end={oldest_valid_root_lag_end} \
                                 q12_floor_only_on_one_disk_publishes={floor_only_on_one_disk_publishes} \
                                 interval_candidate_inference_mismatch_count={}",
                                cell.slots_per_region,
                                cell.placement,
                                cell.workload_period,
                                outcomes.len(),
                                world.arms[index].violations_candidate,
                                world.arms[index].violations_abandoned,
                                world.arms[index].interval_candidate_inference_mismatch_count,
                            ))
                        );

                        // Q6b：读法甲/乙上界（登记第六节 Q6b-甲/乙）——只在「这条历史已经回退过」时有
                        // 意义（`rollback_target_reference_count` 是 `Some`）；只要环里此刻有被抛弃根就采样一次
                        // （不要求这一刻恰好紧跟回退），逐格分别报「隔离 > 上界」与「上界 < 0」的发布数。
                        let mut abandoned_root_bound_sampled_publishes = 0u64;
                        let mut newest_abandoned_bound_exceeded_publishes = 0u64;
                        let mut maximum_abandoned_bound_exceeded_publishes = 0u64;
                        let mut newest_abandoned_bound_negative_publishes = 0u64;
                        let mut maximum_abandoned_bound_negative_publishes = 0u64;
                        for outcome in &outcomes {
                            let Some(rollback_target_reference_counts) = outcome.rollback_target_reference_count else { continue };
                            let (Some(newest), Some(maximum_value)) = outcome.abandoned_item1_bounds[index] else { continue };
                            abandoned_root_bound_sampled_publishes += 1;
                            let isolated = outcome.pre[index].isolated_slots;
                            let check = abandoned_root_bound_check(isolated, newest, maximum_value, rollback_target_reference_counts[index]);
                            if check.newest_bound_negative {
                                newest_abandoned_bound_negative_publishes += 1;
                            }
                            if check.maximum_bound_negative {
                                maximum_abandoned_bound_negative_publishes += 1;
                            }
                            if check.isolated_exceeds_newest_bound {
                                newest_abandoned_bound_exceeded_publishes += 1;
                            }
                            if check.isolated_exceeds_maximum_bound {
                                maximum_abandoned_bound_exceeded_publishes += 1;
                            }
                        }
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=q6b family={family} s={} placement={:?} period={} arm={arm_tag} \
                                 sampled={abandoned_root_bound_sampled_publishes} a_isolated_exceeds_bound_publishes={newest_abandoned_bound_exceeded_publishes} a_bound_negative_publishes={newest_abandoned_bound_negative_publishes} \
                                 b_isolated_exceeds_bound_publishes={maximum_abandoned_bound_exceeded_publishes} b_bound_negative_publishes={maximum_abandoned_bound_negative_publishes}",
                                cell.slots_per_region, cell.placement, cell.workload_period,
                            ))
                        );
                    }
                    // Q9a「多写」= G12 一侧 − 甲-T1 一侧（同影子账、同族同格），峰值与均值各一行；
                    // 另报其中归因于宽度的部分 = G12 条目数（峰值）× 8（20→28 字节多出的那一半，登记第六节）。
                    let threshold_conservative = World::arm_index(Arm::ThresholdConservative);
                    let threshold_narrow = World::arm_index(Arm::ThresholdNarrow);
                    let interval_conservative = World::arm_index(Arm::IntervalConservative);
                    let interval_narrow = World::arm_index(Arm::IntervalNarrow);
                    for (shadow_tag, interval_arm_index, threshold_index) in
                        [("conservative", interval_conservative, threshold_conservative), ("narrow", interval_narrow, threshold_narrow)]
                    {
                        let extra_write_peak = entries_peak_by_index[interval_arm_index] as i64 - entries_peak_by_index[threshold_index] as i64;
                        let extra_write_mean = entries_mean_by_index[interval_arm_index] - entries_mean_by_index[threshold_index];
                        let width_attributed_bytes_at_peak = entries_peak_by_index[interval_arm_index] * 8;
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=q9a_extra_write family={family} s={} placement={:?} period={} shadow={shadow_tag} \
                                 entries_peak_extra={extra_write_peak} entries_mean_extra={extra_write_mean:.3} width_attributed_bytes_at_peak={width_attributed_bytes_at_peak}",
                                cell.slots_per_region, cell.placement, cell.workload_period,
                            ))
                        );
                    }
                }
            }
        }
    }

    // 第五段：Q8 坏镜像矩阵——β1（H1 末态）与 β2（H4dNm2 回退后第 S 次工作负载之后）两个基底，
    // 全部 12 格、全部 5 条臂，B1a/B1b/B2 与 PC-F 的 Bk1/Bk2（登记第五节坏镜像、第九节 PC-F）。
    // B3a/B3b/B4a/B4b/B4c 要重新跑一遍历史（回收步一律不回收）或强制落点碰撞，本段未实现
    // （见跑前登记第十二节第五段「没做什么」）。
    let bad_image_kinds: [(&str, fn(BaseImageState) -> BaseImageState); 5] =
        [("B1a", bad_image_b1a), ("B1b", bad_image_b1b), ("B2", bad_image_deferred_overcount_by_one), ("Bk1", bad_image_bk1), ("Bk2", bad_image_bk2)];
    for cell in all_geometry_cells() {
        let bases: [(&str, Option<World>); 2] = [("beta1", Some(beta_one_world(cell))), ("beta2", beta_two_world(cell))];
        for (base_tag, base_world) in bases {
            let Some(base_world) = base_world else {
                println!(
                    "{}",
                    emitter.emit_raw(&format!("name=q8_base base={base_tag} s={} placement={:?} period={} no_candidate=true", cell.slots_per_region, cell.placement, cell.workload_period))
                );
                continue;
            };
            let base_states = base_image_states(&base_world);
            for (index, arm_variant) in ARM_ORDER.iter().enumerate() {
                let arm_tag = arm_variant.tag();
                let base_checks = accounting_checks(base_states[index]);
                let base_already_red = base_checks != AccountingChecks::default();
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=q8_base_check base={base_tag} s={} placement={:?} period={} arm={arm_tag} \
                         allocated_statistic_mismatch_by_arm_reading={} allocated_statistic_mismatch_by_today_reading={} free_statistic_mismatch={} allocated_minus_deferred_red={}",
                        cell.slots_per_region,
                        cell.placement,
                        cell.workload_period,
                        base_checks.allocated_statistic_mismatch_by_arm_reading,
                        base_checks.allocated_statistic_mismatch_by_today_reading,
                        base_checks.free_statistic_mismatch,
                        base_checks.allocated_minus_deferred_red,
                    ))
                );
                for (image_tag, constructor) in bad_image_kinds {
                    if base_already_red {
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=q8 base={base_tag} image={image_tag} s={} placement={:?} period={} arm={arm_tag} base_already_red=true",
                                cell.slots_per_region, cell.placement, cell.workload_period
                            ))
                        );
                        continue;
                    }
                    let image_checks = accounting_checks(constructor(base_states[index]));
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=q8 base={base_tag} image={image_tag} s={} placement={:?} period={} arm={arm_tag} \
                             detected_by_allocated_statistic_arm_reading={} detected_by_allocated_statistic_today_reading={} detected_by_free_statistic={} detected_by_allocated_minus_deferred={}",
                            cell.slots_per_region,
                            cell.placement,
                            cell.workload_period,
                            image_checks.allocated_statistic_mismatch_by_arm_reading,
                            image_checks.allocated_statistic_mismatch_by_today_reading,
                            image_checks.free_statistic_mismatch,
                            image_checks.allocated_minus_deferred_red,
                        ))
                    );
                }
            }
        }
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_transaction_world(geometry: GeometryCell) -> World {
        let mut world = World::mkfs(geometry);
        for _ in world.first_mount_warmup() {
        }
        let _ = world.publish(PublishKind::Overwrite);
        world
    }

    /// K1：P-impl 的落点几何（三档 S 共 6 格，S 不影响单元区落点）。
    #[test]
    fn first_transaction_placements_match_registered_layout() {
        for slots_per_region in [4u64, 8, 16] {
            for workload_period in [1u64, 4] {
                let geometry = GeometryCell { slots_per_region, placement: PlacementPolicy::Impl, workload_period };
                let world = first_transaction_world(geometry);
                let arm = &world.arms[World::arm_index(Arm::ThresholdConservative)];
                assert_eq!(arm.refs[&Role::DataUnit], 50180, "S={slots_per_region} rho_period={workload_period}");
                assert_eq!(arm.refs[&Role::ExtentRoot], 50240);
                assert_eq!(arm.refs[&Role::InodeLeaf], 50242);
                assert_eq!(arm.refs[&Role::InodeRoot], 50244);
                assert_eq!(arm.refs[&Role::AllocationTree], 50245);
                assert_eq!(arm.refs[&Role::AcctTree], 50246);
                assert_eq!(arm.refs[&Role::CentralMapTree], 50247);
                assert_eq!(arm.refs[&Role::TableUnit], 50248);
                assert!(arm.is_free(50241), "槽 50241 在 txg 3 之后应当空着");
            }
        }
    }

    /// A6：全部臂、全部 12 格，txg 3 那一行 Disk0：第 1 项 13、第 5 项 1、第 2 项 211968-13。
    #[test]
    fn txg_three_accounting_row_matches_every_arm_and_cell() {
        for cell in all_geometry_cells() {
            let world = first_transaction_world(cell);
            for arm in &world.arms {
                let row = arm.accounting_row();
                assert_eq!(row.item1_occupied_slots, 13, "{:?} cell={:?}", arm.arm, cell);
                assert_eq!(row.item5_deferred_slots, 1, "{:?} cell={:?}", arm.arm, cell);
                assert_eq!(row.item2_free_slots, UNIT_AREA_CAPACITY_SLOTS - 13, "{:?} cell={:?}", arm.arm, cell);
            }
        }
    }

    /// A3：单元区容量。
    #[test]
    fn unit_area_capacity_matches_registered_value() {
        assert_eq!(UNIT_AREA_CAPACITY_SLOTS, 211968);
    }

    /// Q7/G8′（第五段）：第一个事务这一格，第 1 项 13 − 第 5 项 1 == 最新根走读引用 12（A6 已经钉过
    /// 13/1，这里补上 G8′ 自己的算式，五条臂、全部 12 格都要成立——U11 的锚点正是这一行）。
    #[test]
    fn allocated_minus_deferred_matches_reference_on_first_transaction_for_every_arm_and_cell() {
        for cell in all_geometry_cells() {
            let world = first_transaction_world(cell);
            let snapshot = world.invariant_snapshot();
            for (index, arm) in world.arms.iter().enumerate() {
                let row = arm.accounting_row();
                assert_eq!(row.item1_occupied_slots - row.item5_deferred_slots, 12, "{:?} cell={:?}", arm.arm, cell);
                assert!(!snapshot.allocated_minus_deferred_reference_mismatch[index], "G8′ 不该在第一个事务这一格误红：{:?} cell={:?}", arm.arm, cell);
            }
        }
    }

    /// U11（登记第九节 M19）：G8′ 不减第 5 项时，txg=3 这一行会误报——手写谓词核对变异表要抓的形状，
    /// 不依赖 `invariant_snapshot` 里已经写好的算式（那条算式本身就是被测对象）。
    #[test]
    fn without_subtracting_deferred_the_check_would_misreport_at_first_transaction() {
        let world = first_transaction_world(BASE_GEOMETRY);
        let arm = &world.arms[World::arm_index(Arm::ThresholdConservative)];
        let row = arm.accounting_row();
        let newest_reference_size = account_from_refs(&arm.refs).expanded_slots().count() as u64;
        assert_eq!(row.item1_occupied_slots, newest_reference_size + 1, "不减第 5 项时应该比引用多 1（M19 的变异形状）");
        assert_eq!(row.item1_occupied_slots - row.item5_deferred_slots, newest_reference_size, "减了第 5 项才对上");
    }

    /// Q10（登记第五节 G12「支持它的人认的样子」）：候选根 r 引用槽 x ⟺ 当前账里 x 的记录分配代 ≤
    /// txg(r) ∧（未释放 ∨ txg(r) < 释放代）——区间推法与实际走读（M0.6）在「记录还在但候选根自己的账
    /// 已经不再指着这个槽」时会分歧，这正是第四段手工诊断在 H4d2 上找到的那类假阳性（登记第十二节
    /// 第四段：槽 50317–50320，`actually_referenced_by=[]`）。
    #[test]
    fn interval_inference_diverges_from_walk_when_the_candidate_root_no_longer_references_the_slot() {
        let mut arm = ArmExecutor::new(Arm::IntervalNarrow, PlacementPolicy::Low);
        // 槽 100：分配代 5、释放代 9（已释放）——区间推法认为候选根 txg=6 落在 [5,9) 里，判「引用」。
        arm.ledger.insert(100, LedgerEntry { span_in_slots: 1, allocation_generation: 5, released: true, release_generation: 9 });
        // 但候选根 txg=6 自己存的账（走读得到的那份）已经不再引用槽 100（它在更晚的覆盖写里换到了别处）。
        let mut refs: HashMap<Role, u64> = HashMap::new();
        refs.insert(Role::ExtentRoot, 200);
        arm.root_accounts.insert(6, account_from_refs(&refs));

        let inferred = interval_inferred_candidate_references(&arm, &[6]);
        let walked = candidate_reference_union(&arm, &[6]);
        assert!(inferred.contains(&100), "区间推法应该认为槽 100 被候选根 6 引用（5 ≤ 6 < 9）");
        assert!(!walked.contains(&100), "候选根 6 自己的账里已经不再引用槽 100");
        assert_ne!(inferred, walked, "这就是 Q10 要抓的那类不一致");
    }

    /// Q10 的反面：账本与候选根自己的账一致时，两条路径给出同一个并集（不是恒等式，是「这份构造下
    /// 恰好一致」）。
    #[test]
    fn interval_inference_matches_walk_when_the_ledger_and_the_candidate_account_agree() {
        let mut arm = ArmExecutor::new(Arm::IntervalNarrow, PlacementPolicy::Low);
        arm.ledger.insert(100, LedgerEntry { span_in_slots: 1, allocation_generation: 5, released: false, release_generation: 0 });
        let mut refs: HashMap<Role, u64> = HashMap::new();
        refs.insert(Role::ExtentRoot, 100);
        arm.root_accounts.insert(6, account_from_refs(&refs));

        let inferred = interval_inferred_candidate_references(&arm, &[6]);
        let walked = candidate_reference_union(&arm, &[6]);
        assert_eq!(inferred, walked);
        assert!(inferred.contains(&100));
    }

    /// Q6b（登记第六节 Q6b-甲/乙）：`abandoned_root_item1_bounds` 逐臂返回 (txg 最大的被抛弃根的引用
    /// 集合大小, 全部被抛弃根里的最大值)——两个读法在「txg 最大的那条不是引用最多的那条」时会给出
    /// 不同的数，这正是读法甲、乙分开报的理由（登记第一节 Q6b-甲/乙「环里有几条被抛弃根时指哪一条
    /// 没写，两种读法各报一行」）。
    #[test]
    fn abandoned_root_item1_bounds_distinguishes_newest_from_maximum() {
        let placement = PlacementPolicy::Low;
        let mut arms: [ArmExecutor; 5] = ARM_ORDER.map(|arm| ArmExecutor::new(arm, placement));
        for arm in &mut arms {
            // txg 5（较旧）：引用 3 个槽——三个不同角色各占一个不同的起点槽。
            let mut older: HashMap<Role, u64> = HashMap::new();
            older.insert(Role::ExtentRoot, 100);
            older.insert(Role::InodeRoot, 102);
            older.insert(Role::AllocationTree, 104);
            arm.root_accounts.insert(5, account_from_refs(&older));
            // txg 8（最新，txg 最大）：只引用 1 个槽。
            let mut newer: HashMap<Role, u64> = HashMap::new();
            newer.insert(Role::ExtentRoot, 200);
            arm.root_accounts.insert(8, account_from_refs(&newer));
        }
        let bounds = abandoned_root_item1_bounds(&arms, &[5, 8]);
        for (newest, maximum_value) in bounds {
            assert_eq!(newest, Some(1), "txg 最大的那条（8）只引用 1 个槽");
            assert_eq!(maximum_value, Some(3), "全部被抛弃根里最大的是 txg 5 的 3 个槽");
        }
        let empty = abandoned_root_item1_bounds(&arms, &[]);
        for (newest, maximum_value) in empty {
            assert_eq!(newest, None, "环里没有被抛弃根时两项都该是 None");
            assert_eq!(maximum_value, None);
        }
    }

    /// Q6b：`abandoned_root_bound_check` 的四种边界——超过 / 不超过、上界为负 / 非负，逐个钉住。
    #[test]
    fn abandoned_root_bound_check_matches_hand_computed_cases() {
        // 隔离 5，读法甲上界 = 8 − 3 = 5（不超过，>= 不算超过，只有严格大于才算）；
        // 读法乙上界 = 10 − 3 = 7（更不超过）；两个上界都非负。
        let not_exceeded = abandoned_root_bound_check(5, 8, 10, 3);
        assert_eq!(
            not_exceeded,
            AbandonedRootBoundCheck { isolated_exceeds_newest_bound: false, newest_bound_negative: false, isolated_exceeds_maximum_bound: false, maximum_bound_negative: false }
        );
        // 隔离 6：读法甲上界 5，6 > 5 超过；读法乙上界 7，6 未超过。
        let exceeds_newest_only = abandoned_root_bound_check(6, 8, 10, 3);
        assert!(exceeds_newest_only.isolated_exceeds_newest_bound);
        assert!(!exceeds_newest_only.isolated_exceeds_maximum_bound);
        // R_old 比被抛弃根的代理值还大：上界为负，隔离只要 > 0 就一定超过。
        let negative_bound = abandoned_root_bound_check(1, 2, 2, 5);
        assert!(negative_bound.newest_bound_negative);
        assert!(negative_bound.maximum_bound_negative);
        assert!(negative_bound.isolated_exceeds_newest_bound);
        assert!(negative_bound.isolated_exceeds_maximum_bound);
        // 隔离 0、上界 0：0 > 0 为假，不算超过。
        let zero_bound_zero_isolated = abandoned_root_bound_check(0, 4, 4, 4);
        assert!(!zero_bound_zero_isolated.isolated_exceeds_newest_bound);
        assert!(!zero_bound_zero_isolated.newest_bound_negative);
    }

    /// 一份记账三项与引用集合都自洽（四个检查都判绿）的合成基底，供坏镜像单测用——不跑完整历史，
    /// 只要数值关系对得上：item1 + item2 == 容量，item1 == 两种候选并集，item1 − item5 == 最新引用。
    fn green_base_state() -> BaseImageState {
        let item1 = 20u64;
        let item5 = 2u64;
        BaseImageState {
            accounting: AccountingRow { item1_occupied_slots: item1, item2_free_slots: UNIT_AREA_CAPACITY_SLOTS - item1, item5_deferred_slots: item5 },
            candidate_by_effective_floor: item1,
            candidate_by_today_reading: item1,
            newest_reference_size: item1 - item5,
        }
    }

    /// Q8 起手式：合成的绿色基底四个检查都不判红（对应登记「先证明 G8′ 在全部合法状态上 0 误红，
    /// 再报坏镜像」那句话的最小形态）。
    #[test]
    fn accounting_checks_of_a_well_formed_base_state_are_all_green() {
        assert_eq!(accounting_checks(green_base_state()), AccountingChecks::default());
    }

    /// B1a（登记第五节坏镜像）：只翻记录标志，四个检查都判不出——它们都不读分配记录本身。
    #[test]
    fn bad_image_b1a_is_undetectable_by_any_of_the_four_checks() {
        let checks = accounting_checks(bad_image_b1a(green_base_state()));
        assert_eq!(checks, AccountingChecks::default(), "B1a 只翻记录标志，四个检查都不该判红");
    }

    /// B1b：B1a 再把第 5 项加 2——只有 G8′（读第 5 项）判得出，I-3.1/I-5.2 都不读第 5 项。
    #[test]
    fn bad_image_b1b_is_caught_only_by_the_allocated_minus_deferred_check() {
        let checks = accounting_checks(bad_image_b1b(green_base_state()));
        assert!(checks.allocated_minus_deferred_red, "第 5 项多 2，G8′ 该判红");
        assert!(!checks.allocated_statistic_mismatch_by_arm_reading, "I-3.1（臂内读法）不读第 5 项");
        assert!(!checks.allocated_statistic_mismatch_by_today_reading, "I-3.1（今天读法）不读第 5 项");
        assert!(!checks.free_statistic_mismatch, "I-5.2 不读第 5 项");
    }

    /// B2：第 5 项单独多报一槽——形状与 B1b 相同（只是幅度不同），同样只有 G8′ 判得出。
    #[test]
    fn bad_image_deferred_overcount_by_one_is_caught_only_by_the_allocated_minus_deferred_check() {
        let checks = accounting_checks(bad_image_deferred_overcount_by_one(green_base_state()));
        assert!(checks.allocated_minus_deferred_red);
        assert!(!checks.allocated_statistic_mismatch_by_arm_reading && !checks.allocated_statistic_mismatch_by_today_reading && !checks.free_statistic_mismatch);
    }

    /// PC-F 的 Bk1（阳性对照）：D0 第 1 项多 1 槽——三个基于「第 1 项 vs 引用集合大小」比较的检查
    /// 都该判红（I-3.1 两种读法与 G8′）；它同时也会破坏 I-5.2（第 1+2 项之和），这是数值上的必然
    /// 结果，不是登记「必须看到」那句要否定的东西。
    #[test]
    fn bad_image_bk1_is_caught_by_the_reference_comparison_checks() {
        let checks = accounting_checks(bad_image_bk1(green_base_state()));
        assert!(checks.allocated_statistic_mismatch_by_arm_reading && checks.allocated_statistic_mismatch_by_today_reading && checks.allocated_minus_deferred_red);
    }

    /// PC-F 的 Bk2（阳性对照）：D0 第 2 项少 1 槽——只有 I-5.2 判得出，其余三个检查只看第 1/5 项。
    #[test]
    fn bad_image_bk2_is_caught_only_by_the_free_statistic_check() {
        let checks = accounting_checks(bad_image_bk2(green_base_state()));
        assert!(checks.free_statistic_mismatch, "第 2 项少 1，I-5.2 该判红");
        assert!(!checks.allocated_statistic_mismatch_by_arm_reading && !checks.allocated_statistic_mismatch_by_today_reading, "候选并集比较用的是第 1 项，没变");
        assert!(!checks.allocated_minus_deferred_red, "G8′ 用第 1 项减第 5 项，都没变");
    }

    /// Q8 基底：β1（H1 末态）、β2（H4dNm2 回退后第 S 次工作负载之后）两个真实历史算出来的基底状态，
    /// 与第四段已经查明的两条事实对得上——甲-T1 在 H1 连续覆盖写下不该判红（每次发布前都回收），
    /// 基线「今」该判红（P4：一次挂载之内不回收）；G12（IntervalNarrow）在含回退的族上该判红（第四段
    /// 手工诊断的根因，见跑前登记第十二节第四段）。
    #[test]
    fn beta_one_and_beta_two_bases_match_known_shape() {
        let cell = BASE_GEOMETRY;
        let beta_one = beta_one_world(cell);
        let beta_one_states = base_image_states(&beta_one);
        let threshold_conservative = World::arm_index(Arm::ThresholdConservative);
        let baseline = World::arm_index(Arm::Baseline);
        assert!(
            !accounting_checks(beta_one_states[threshold_conservative]).allocated_statistic_mismatch_by_arm_reading,
            "甲-T1 每次发布前都回收，H1 连续覆盖写下不该在臂内读法上判红"
        );
        assert!(accounting_checks(beta_one_states[baseline]).allocated_statistic_mismatch_by_arm_reading, "基线『今』一次挂载之内不回收，H1 转过一圈之后该判红（P4）");

        let beta_two = beta_two_world(cell).expect("H4dNm2 在基准格上必有候选（第四段产物已验证过）");
        let beta_two_states = base_image_states(&beta_two);
        let interval_narrow = World::arm_index(Arm::IntervalNarrow);
        assert!(accounting_checks(beta_two_states[interval_narrow]).allocated_statistic_mismatch_by_arm_reading, "G12 在含回退的族上该判红（第四段手工诊断已查清根因）");
    }

    /// Q10：区间推法的窗口是半开的 `[分配代, 释放代)`（登记第一节「G12」谓词逐字），
    /// 两端各测一遍，同一条测试钉住两处边界。
    #[test]
    fn interval_inference_window_is_half_open_on_the_release_end() {
        let mut arm = ArmExecutor::new(Arm::IntervalNarrow, PlacementPolicy::Low);
        arm.ledger.insert(100, LedgerEntry { span_in_slots: 1, allocation_generation: 5, released: true, release_generation: 9 });
        assert!(!interval_inferred_candidate_references(&arm, &[9]).contains(&100), "释放代本身不在半开区间 [5, 9) 里");
        assert!(interval_inferred_candidate_references(&arm, &[8]).contains(&100), "释放代前一格在区间内");
        assert!(interval_inferred_candidate_references(&arm, &[5]).contains(&100), "分配代本身在区间内（闭区间左端）");
        assert!(!interval_inferred_candidate_references(&arm, &[4]).contains(&100), "分配代之前不在区间内");
    }

    /// Q9a（第四段，见跑前登记第十二节）：`peak_mean_end` 的基本行为——空序列三项都是 0；
    /// 非空序列峰值取最大、末值取最后一个、均值按算术平均（不是中位数或别的口径）。
    #[test]
    fn peak_mean_end_matches_arithmetic_definition() {
        assert_eq!(peak_mean_end(&[]), (0, 0.0, 0));
        let (peak, mean, end) = peak_mean_end(&[3, 7, 2]);
        assert_eq!(peak, 7, "峰值取最大值，不是首值或末值");
        assert!((mean - 4.0).abs() < 1e-9, "均值 = (3+7+2)/3 = 4，实得 {mean}");
        assert_eq!(end, 2, "末值取序列最后一个元素，不是最大或最小");
    }

    /// Q9a：条目字节 = 条目数 × 条目宽；叶节点数 = ⌈条目数 ÷ 每节点条目数⌉（向上取整，不是向下）。
    #[test]
    fn allocation_record_bytes_and_leaf_nodes_match_registered_formula() {
        assert_eq!(allocation_record_bytes_and_leaf_nodes(10, 20, 812), (200, 1));
        assert_eq!(allocation_record_bytes_and_leaf_nodes(812, 20, 812), (16240, 1), "整除时叶节点数不多算一个");
        assert_eq!(allocation_record_bytes_and_leaf_nodes(813, 20, 812), (16260, 2), "多 1 条也要多开一个叶节点，不能向下取整");
        assert_eq!(allocation_record_bytes_and_leaf_nodes(1160, 28, 580), (32480, 2));
    }

    /// Q9a：txg 3（第一个事务）之后，五条臂的分配记录条目数（`ledger.len()`）都是 10——
    /// mkfs 两个角色（实例表单元、树表单元）各留一条（树表单元那条在 txg 3 被覆盖释放但条目不删，
    /// D3 已定项 7），加上 txg 3 这次覆盖写新分配的 8 个角色，1+1+8=10；不随 S / 落点政策 / period 变。
    #[test]
    fn allocation_record_entries_after_first_transaction_is_ten_on_every_cell_and_arm() {
        for cell in all_geometry_cells() {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {
            }
            let outcome = world.publish(PublishKind::Overwrite);
            for (index, arm) in world.arms.iter().enumerate() {
                // 两条路径都要对上 10：`arm.ledger.len()` 是权威定义（M0.18），
                // `outcome.allocation_record_entries` 是 Q9a 轨迹实际读取的那个字段
                // （`snapshot_allocation_record_read_counters`）——只查前者会漏过后者算错的变异。
                assert_eq!(arm.ledger.len(), 10, "{:?} cell={:?}", arm.arm, cell);
                assert_eq!(outcome.allocation_record_entries[index], 10, "{:?} cell={:?}", arm.arm, cell);
            }
        }
    }

    /// Q9b（第四段）：H4d2（单次回退）的历史里，`is_rebuild_step` 只在回退那次写行发布上为真——
    /// 回退之前 / 之后的普通发布，以及回退后的暖机空发布，都不是紧跟 `rebuild_allocator` 的那一次。
    #[test]
    fn h4d2_history_has_exactly_one_rebuild_step_outcome() {
        let cell = BASE_GEOMETRY;
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {}
        let (outcomes, _target, write_row_txg) = world.history_single_rollback(2).expect("H4d2 在基准几何上必有候选");
        let rebuild_outcomes: Vec<&PublishOutcome> = outcomes.iter().filter(|outcome| outcome.is_rebuild_step).collect();
        assert_eq!(rebuild_outcomes.len(), 1, "H4d2 只回退一次，重建步只应出现一次");
        assert_eq!(rebuild_outcomes[0].txg, write_row_txg, "重建步的采样点必须是回退那次写行发布");
        assert_eq!(rebuild_outcomes[0].label, "write_row");
    }

    /// A1：分配记录树每节点条目数。
    #[test]
    fn records_per_node_matches_registered_value() {
        assert_eq!(RECORDS_PER_NODE_SINGLE_GENERATION, 812);
        assert_eq!(RECORDS_PER_NODE_DUAL_GENERATION, 580);
    }

    /// K2：条目宽度。
    #[test]
    fn record_width_bytes_match_registered_values() {
        assert_eq!(Arm::ThresholdConservative.record_width_bytes(), 20);
        assert_eq!(Arm::Baseline.record_width_bytes(), 20);
        assert_eq!(Arm::IntervalConservative.record_width_bytes(), 28);
        assert_eq!(Arm::IntervalNarrow.record_width_bytes(), 28);
    }

    /// M0.9（不在登记第九节的 U 编号里；S1 对拍之前主 agent 点名核实、由执行员补的单测）：
    /// mkfs 是实例 0、不写行；首次挂载取实例 1、不写行（登记「mkfs 实例 0；首次挂载实例 1，不写行」），
    /// 与 `crates/singlefs-core/src/mount.rs` 785-811 `establish_instance` 对只做过 mkfs 的池
    /// 「取号 1、不写行」一致。改坏点：`first_mount_warmup` 若不取号，`current_instance` 停在 mkfs
    /// 的 0，本测试第二条 `assert_eq!` 会从 `1` 变红成 `0`（已实测：注释掉取号那两行，此断言失败）。
    #[test]
    fn first_mount_warmup_acquires_instance_one_without_writing_a_row() {
        let mut world = World::mkfs(BASE_GEOMETRY);
        assert_eq!(world.pool.current_instance, 0, "mkfs 是实例 0");
        assert!(world.pool.instance_table_rows.is_empty(), "mkfs 不写行");
        for _ in world.first_mount_warmup() {
        }
        assert_eq!(world.pool.current_instance, 1, "首次挂载取实例 1（M0.9）");
        assert!(world.pool.instance_table_rows.is_empty(), "首次挂载不写行（M0.9：上一个实例是 0、要写的行为空）");
    }

    /// K3：根环坐标公式与暖机 txg。
    #[test]
    fn root_ring_coordinates_match_registered_formula() {
        assert_eq!(root_ring_region(3), 0);
        assert_eq!(root_ring_device(3), Device::Disk0);
        assert_eq!(root_ring_slot_in_region(3, 8), 1);
        let world = first_transaction_world(BASE_GEOMETRY);
        assert_eq!(world.pool.current_txg, 3, "首次挂载暖机恰 2 次（txg 1、2），第一个事务 txg 3");
    }

    fn run_continuous_overwrite_history(cell: GeometryCell) -> (World, Vec<PublishOutcome>) {
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {
        }
        let ring_capacity = cell.ring_capacity();
        let mut outcomes = Vec::new();
        for outcome in world.run_workload_segment(6 * ring_capacity, cell.workload_period) {
            outcomes.push(outcome);
        }
        (world, outcomes)
    }

    /// A4：H1 主历史最后一次 O 那一行，测量臂（甲-T1、G12 四条）的记账三项与该行 txg。
    /// 三档 S 的数值取自登记第十三节 [C4] 的原样输出（独立算出、用命令核过，K2 的第二类锚点）。
    #[test]
    fn steady_state_accounting_matches_closed_form() {
        let cases: [(u64, u64, u64, u64, u64, u64); 6] = [
            // (S, period, expected_item1, expected_item5, expected_row_txg, _unused)
            (4, 1, 122, 110, 74, 0),
            (4, 4, 74, 62, 71, 0),
            (8, 1, 242, 230, 146, 0),
            (8, 4, 140, 128, 143, 0),
            (16, 1, 482, 470, 290, 0),
            (16, 4, 272, 260, 287, 0),
        ];
        for (slots_per_region, workload_period, expected_item1, expected_item5, expected_row_txg, _) in cases {
            let cell = GeometryCell { slots_per_region, placement: PlacementPolicy::Impl, workload_period };
            let (_world, outcomes) = run_continuous_overwrite_history(cell);
            let last_overwrite = outcomes.iter().rev().find(|outcome| outcome.is_nonempty).expect("H1 至少有一次 O");
            assert_eq!(last_overwrite.txg, expected_row_txg, "S={slots_per_region} period={workload_period}");
            for index in ARM_ORDER.iter().enumerate().filter(|(_, arm)| arm.is_measured()).map(|(index, _)| index) {
                let row = last_overwrite.accounting[index];
                assert_eq!(row.item1_occupied_slots, expected_item1, "{:?} S={slots_per_region} period={workload_period}", ARM_ORDER[index]);
                assert_eq!(row.item5_deferred_slots, expected_item5, "{:?} S={slots_per_region} period={workload_period}", ARM_ORDER[index]);
                assert_eq!(row.item2_free_slots, UNIT_AREA_CAPACITY_SLOTS - expected_item1, "{:?}", ARM_ORDER[index]);
            }
        }
    }

    /// A5：基线在 H1、ρ=1 下，主历史最后一次发布之前那一刻的多扣（不回收，单调涨）。
    #[test]
    fn baseline_over_held_grows_unboundedly_without_per_publish_reclaim() {
        for (slots_per_region, expected) in [(4u64, 591u64), (8, 1191), (16, 2391)] {
            let cell = GeometryCell { slots_per_region, placement: PlacementPolicy::Impl, workload_period: 1 };
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {
            }
            let ring_capacity = cell.ring_capacity();
            let mut last_pre = None;
            for outcome in world.run_workload_segment(6 * ring_capacity, 1) {
                last_pre = Some(outcome.pre[World::arm_index(Arm::Baseline)]);
            }
            let baseline_pre = last_pre.expect("H1 至少一次发布");
            assert_eq!(baseline_pre.over_held_slots, expected as i64, "S={slots_per_region}");
            assert_eq!(baseline_pre.isolated_slots, 0);
        }
    }

    /// A7：测量臂在 H1 稳态（无被抛弃根、环连续）下，多扣与隔离恒为 0。
    #[test]
    fn measured_arms_over_held_and_isolated_are_zero_in_connected_ring() {
        let cell = BASE_GEOMETRY;
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {
        }
        let ring_capacity = cell.ring_capacity();
        let outcomes = world.run_workload_segment(6 * ring_capacity, 1);
        for outcome in outcomes.iter().filter(|outcome| outcome.txg > 3 + 2 * ring_capacity) {
            for index in ARM_ORDER.iter().enumerate().filter(|(_, arm)| arm.is_measured()).map(|(index, _)| index) {
                assert_eq!(outcome.pre[index].over_held_slots, 0, "txg={} arm_index={index}", outcome.txg);
                assert_eq!(outcome.pre[index].isolated_slots, 0, "txg={}", outcome.txg);
            }
        }
    }

    fn pending_record(slot: u64, allocation_generation: u64, release_generation: u64) -> PendingRecord {
        PendingRecord { slot, span: 1, allocation_generation, release_generation }
    }

    /// U1（M3 的锚点）：门槛用 max(F_生效, 最旧有效根)，不能只用最旧有效根。
    /// 环 txg 10..33 连续有效、F_生效 20，释放代 15 的记录：门槛应是 20，原式可再分配。
    #[test]
    fn threshold_predicate_uses_maximum_of_floor_and_oldest_valid_root() {
        let mut pool = PoolState::new(8);
        for txg in 10..=33u64 {
            pool.instance_table_rows.clear();
            pool.commit_root(txg, 0, 20, false);
        }
        let threshold = pool.reclaim_threshold();
        assert_eq!(threshold, 20, "F_生效 20 > 最旧有效根 10");
        let record = pending_record(0, 0, 15);
        assert!(is_record_reclaimable(Predicate::ThresholdFloor, threshold, None, &record), "释放代 15 ≤ 门槛 20，原式可再分配");
        // M3 的变异：门槛丢 F，只用最旧有效根 10 —— 15 ≤ 10 为假，应当抓到。
        assert!(!is_record_reclaimable(Predicate::ThresholdFloor, 10, None, &record), "变异门槛 10 下 15 ≤ 10 为假");
    }

    /// U2（M4 的锚点）：G12 区间是半开 [分配代, 释放代)，不是闭区间。
    /// 分配代 5、释放代 9、环 {9, 10}，原式可再分配（9 不在 [5,9) 里）。
    #[test]
    fn interval_predicate_is_half_open() {
        let blocking: BTreeSet<u64> = [9u64, 10].into_iter().collect();
        let record = pending_record(0, 5, 9);
        assert!(is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking), &record), "9 不在半开区间 [5,9) 里，原式可再分配");
        // M4 的变异：区间闭合成 [5,9]，9 落进去，应当抓到。
        let closed_blocking_hit = blocking.range(5..=9).next().is_some();
        assert!(closed_blocking_hit, "闭区间下 9 会落进 [5,9]，变异应当不可再分配");
    }

    /// U3（M5 的锚点）：G12 的 blocking 集合必须含被抛弃根，不能只看候选集。
    /// 记录 [5,20)、blocking={7,25}（7 是被抛弃根）：原式不可再分配。
    #[test]
    fn blocking_set_must_include_abandoned_roots() {
        let blocking_with_abandoned: BTreeSet<u64> = [7u64, 25].into_iter().collect();
        let record = pending_record(0, 5, 20);
        assert!(!is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking_with_abandoned), &record), "7 在 [5,20) 里且是被抛弃根，原式不可再分配");
        // M5 的变异：丢「被抛弃」子句，blocking 只剩候选集 {25}——25 不在 [5,20) 里，变异应当变成可再分配。
        let blocking_without_abandoned: BTreeSet<u64> = [25u64].into_iter().collect();
        assert!(is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking_without_abandoned), &record), "丢了被抛弃根之后 [5,20) 里没有 blocking 成员");
    }

    /// U4（M6 的锚点）：G12 的 blocking 集合里，有效根要 txg ≥ F_生效 才算 blocking，不是「有效根一律保护」。
    /// 记录 [5,20)、环 {12 有效, 25}、F_生效 15：12 < 15 不该 blocking，原式可再分配。
    #[test]
    fn valid_root_below_floor_does_not_block() {
        let blocking_correct: BTreeSet<u64> = [25u64].into_iter().collect(); // 12 valid 但 12 < F_生效=15，不进 blocking
        let record = pending_record(0, 5, 20);
        assert!(is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking_correct), &record), "12 未过 F_生效 门槛，不 blocking，原式可再分配");
        // M6 的变异：丢「txg ≥ F_生效」子句，有效根一律保护——12 也进 blocking，12 在 [5,20) 里，变异应当不可再分配。
        let blocking_mutated: BTreeSet<u64> = [12u64, 25].into_iter().collect();
        assert!(!is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking_mutated), &record));
    }

    /// K4：全部 12 格，H4 回退那次 W 的 txg = 3N + 3。
    #[test]
    fn rollback_write_row_txg_matches_registered_formula() {
        for slots_per_region in [4u64, 8, 16] {
            for placement in [PlacementPolicy::Impl, PlacementPolicy::Low] {
                for workload_period in [1u64, 4] {
                    let cell = GeometryCell { slots_per_region, placement, workload_period };
                    let mut world = World::mkfs(cell);
                    for _ in world.first_mount_warmup() {
                    }
                    let ring_capacity = cell.ring_capacity();
                    for _ in world.run_workload_segment(3 * ring_capacity, workload_period) {
                    }
                    let newest_txg = world.pool.current_txg;
                    let target_ceiling = newest_txg - 2; // d = 2
                    let target_txg = world.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| *txg <= target_ceiling).max().expect("候选集里必有一条 ≤ target_ceiling");
                    let outcomes = world.rollback_to(target_txg);
                    let write_row_txg = outcomes.iter().find(|outcome| outcome.label == "write_row").map(|outcome| outcome.txg).expect("回退必产出一次写行发布");
                    let expected_write_row_txg = 3 * ring_capacity + 3;
                    assert_eq!(write_row_txg, expected_write_row_txg, "S={slots_per_region} placement={placement:?} period={workload_period}");
                    // A2：warmup_until_both_devices() 真实产出的暖机次数要等于「按 txg mod 3 推」的公式
                    // （这条断言是 warmup_stops_after_one_device 那条变异唯一的锚点：只手算 A2 的公式不
                    // 会调用真实函数，抓不住把循环条件从 < 2 改成 < 1 这类破坏）。
                    let warm_up_publish_count = outcomes.iter().filter(|outcome| outcome.label == "empty").count() as u64;
                    let mut covered_after_write_row: BTreeSet<Device> = BTreeSet::new();
                    covered_after_write_row.insert(root_ring_device(write_row_txg));
                    let mut expected_warm_up_publish_count = 0u64;
                    let mut probe_txg = write_row_txg;
                    while covered_after_write_row.len() < 2 {
                        probe_txg += 1;
                        covered_after_write_row.insert(root_ring_device(probe_txg));
                        expected_warm_up_publish_count += 1;
                    }
                    assert_eq!(warm_up_publish_count, expected_warm_up_publish_count, "S={slots_per_region} placement={placement:?} period={workload_period}");
                }
            }
        }
    }

    /// K5：ρ = 1 的 6 格，H6 抬 F 那一刻：目标 F = 3N − 1（35 / 71 / 143），带新 F 的空发布推 2 次。
    #[test]
    fn raise_floor_target_and_publish_count_match_registered_values() {
        for slots_per_region in [4u64, 8, 16] {
            for placement in [PlacementPolicy::Impl, PlacementPolicy::Low] {
                let cell = GeometryCell { slots_per_region, placement, workload_period: 1 };
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {
                }
                let ring_capacity = cell.ring_capacity();
                for _ in world.run_workload_segment(3 * ring_capacity, 1) {
                }
                let outcomes = world.raise_floor();
                for outcome in &outcomes {
                }
                let expected_target_floor = 3 * ring_capacity - 1;
                assert_eq!(world.pool.current_rollback_floor, expected_target_floor, "S={slots_per_region} placement={placement:?}");
                assert_eq!(outcomes.len(), 2, "带新 F 的空发布推 2 次：S={slots_per_region} placement={placement:?}");
            }
        }
    }

    /// S1(b)（登记第十一节停机条款 S1，`plain_remount` 补的驱动）：A(txg3,inst1)、B(txg4,inst1，实例内覆盖写)、
    /// 普通挂载取号 2（写行 txg5、暖机 txg6-7）、C(txg8,inst2)、回退到 A 的根 (1,3)。
    /// 对拍目标（`crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` 已断言过的真实值，
    /// 逐字段见该文件 `rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content`
    /// 190-198 行与 `rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation`
    /// 223-235 行）：回退发布（D）落 txg 9、实例 3；写的行 = [(1,3),(2,0)]（回退行 + 中间实例，(instance,T) 分量）；
    /// 基线臂（「今」，Arm::Baseline）隔离槽数每盘 34。
    /// **修过（2026-09-17，第三段，见跑前登记第十二节）**：此前 `rebuild_allocator` 经 `ArmFullSnapshot`
    /// 把 `root_accounts` 一起「恢复」到回退目标（A，txg3）那一刻的快照，`recompute_shadow` 查
    /// `abandoned_extended=[5,6,7,8]`（写行、暖机×2、C）时全部落空、隔离集恒空，隔离槽数曾实测为 0。
    /// 改法：`ArmFullSnapshot` 不再携带 `root_accounts`，`restore_full` 不再覆盖它——它按自己的注释
    /// 该一直反映「每条仍在环里的根」，由 `World::prune_evicted_accounts` 独立维护，不随回退时光倒流。
    #[test]
    fn pairing_probe_segment_rollback_matches_real_implementation() {
        let cell = BASE_GEOMETRY;
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {
        }
        let first_overwrite = world.publish(PublishKind::Overwrite);
        assert_eq!(first_overwrite.txg, 3, "第一次覆盖写落 txg 3");
        let second_overwrite = world.publish(PublishKind::Overwrite);
        assert_eq!(second_overwrite.txg, 4, "第二次覆盖写落 txg 4，同实例覆盖写");
        let remount_outcomes = world.plain_remount();
        let write_row = remount_outcomes.iter().find(|outcome| outcome.label == "write_row").expect("普通挂载必有写行发布");
        assert_eq!(write_row.txg, 5, "写行发布落 txg 5");
        assert_eq!(write_row.instance, 2, "取号 2");
        let warm_up_count = remount_outcomes.iter().filter(|outcome| outcome.label == "empty").count();
        assert_eq!(warm_up_count, 2, "暖机两次覆盖两块盘");
        let third_overwrite = world.publish(PublishKind::Overwrite);
        assert_eq!(third_overwrite.txg, 8, "第三次覆盖写落 txg 8");
        assert_eq!(third_overwrite.instance, 2);

        let rollback_outcomes = world.rollback_to(3);
        let rollback_write_row = rollback_outcomes.iter().find(|outcome| outcome.label == "write_row").expect("回退必有写行发布");
        assert_eq!(rollback_write_row.txg, 9, "回退写行落 txg 9（真实实现 rollback_publish.root.checkpoint_txg 同为 9）");
        assert_eq!(rollback_write_row.instance, 3, "回退取实例 3");
        assert_eq!(
            world.pool.instance_table_rows,
            vec![(1, 3), (2, 0)],
            "写的行 = 回退行 (1,3) + 中间实例行 (2,0)（真实实现 rows_written 的 (instance,selected_root_txg) 分量同为这两条）"
        );

        let baseline_index = World::arm_index(Arm::Baseline);
        let isolated = world.arms[baseline_index].isolated.len();
        assert_eq!(isolated, 34, "基线臂隔离槽数每盘 34（真实实现 isolated_slots_per_device 同为每盘 34）");
    }

    /// 对拍段落后半（登记第十一节停机条款 S1）：在上一个对拍测试的回退之后再覆盖写四次（实例 3），
    /// 抬 F 到第一次释放代（=第一次覆盖写的 txg），比对回收的落点集合与之后一次复用的落点。
    /// 对拍目标（`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`
    /// `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`
    /// 144-192 行）：抬 F 上限 = 第一次覆盖写的 txg；两次带新 F 的空发布；复用发布落在最低可再分配的偶数槽对。
    #[test]
    fn pairing_probe_segment_raise_floor_and_reuse_matches_real_implementation() {
        let cell = BASE_GEOMETRY;
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {
        }
        let _first_overwrite = world.publish(PublishKind::Overwrite); // txg 3
        let _second_overwrite = world.publish(PublishKind::Overwrite); // txg 4
        let _remount = world.plain_remount(); // txg 5-7，实例 2
        let _third_overwrite = world.publish(PublishKind::Overwrite); // txg 8
        let _rollback = world.rollback_to(3); // txg 9-10，实例 3
        let baseline_index = World::arm_index(Arm::Baseline);
        // 「每次发布的记账三项」（S1(c) 条件文本）：与真实实现 `second_transaction_step_five_reuse.rs`
        // 重跑打印的 E153S1 tag=c_overwrite/c_after_raise/c_reuse 逐字节一致（第三段 S1 对拍，草稿目录
        // `/tmp/claude-1000/e153-runner-stage3/repo-copy/`，探针 `e153_s1_pairing_probe.rs` 段 c）。
        let expected_after_txg: [(u64, u64, u64, u64); 4] = [(11, 33, 211935, 21), (12, 43, 211925, 31), (13, 53, 211915, 41), (14, 63, 211905, 51)];
        for (expected_txg, expected_allocated, expected_free, expected_deferred) in expected_after_txg {
            let outcome = world.publish(PublishKind::Overwrite);
            assert_eq!(outcome.txg, expected_txg, "回退之后连续覆盖写的 txg 序列");
            let row = outcome.accounting[baseline_index];
            assert_eq!(row.item1_occupied_slots, expected_allocated, "txg={expected_txg} 第 1 项");
            assert_eq!(row.item2_free_slots, expected_free, "txg={expected_txg} 第 2 项");
            assert_eq!(row.item5_deferred_slots, expected_deferred, "txg={expected_txg} 第 5 项");
        }
        assert_eq!(world.pool.current_txg, 14, "回退之后四次覆盖写落 txg 11-14");

        let raise_outcomes = world.raise_floor();
        assert_eq!(world.pool.current_rollback_floor, 11, "抬 F 上限 = 第一次覆盖写的 txg 11（真实实现 raised.ceiling 同为 11）");
        assert_eq!(raise_outcomes.len(), 2, "两次带新 F 的空发布（真实实现 raised.publishes 同为两条，txg 15/16）");
        let after_raise = raise_outcomes.last().expect("两次空发布，至少一条").accounting[baseline_index];
        assert_eq!(after_raise.item1_occupied_slots, 50, "抬 F 生效之后（txg 16）第 1 项：真实实现 c_after_raise allocated=50");
        assert_eq!(after_raise.item2_free_slots, 211918, "抬 F 生效之后（txg 16）第 2 项：真实实现 c_after_raise free=211918（本段 raise_floor() 的 baseline_free_slots 修法要保这一行）");
        assert_eq!(after_raise.item5_deferred_slots, 38, "抬 F 生效之后（txg 16）第 5 项：真实实现 c_after_raise deferred=38");

        let reuse = world.publish(PublishKind::Overwrite); // txg 17
        assert_eq!(reuse.txg, 17, "复用发布落 txg 17（与真实实现 reuse.root.checkpoint_txg 同为 17）");
        let reuse_row = reuse.accounting[baseline_index];
        assert_eq!(reuse_row.item1_occupied_slots, 60, "复用发布之后第 1 项：真实实现 c_reuse allocated=60");
        assert_eq!(reuse_row.item2_free_slots, 211908, "复用发布之后第 2 项：真实实现 c_reuse free=211908");
        assert_eq!(reuse_row.item5_deferred_slots, 48, "复用发布之后第 5 项：真实实现 c_reuse deferred=48");
        let reused_slot = world.arms[baseline_index].refs[&Role::DataUnit];
        assert_eq!(
            reused_slot, 50178,
            "复用发布把数据单元落回 50178：mkfs 树表那 1 槽回收了、50179 从没分配过；mkfs 实例表那片 50176 也回收了但 B 的根还引用它、\
             影子账隔离着（真实实现 `second_transaction_step_five_reuse.rs` 文件头注释第 3 行同为 50178；\
             2026-09-17 第三段发现并修：raise_floor() 此前没有重算影子账，模型曾给出 50176，见跑前登记第十二节）"
        );
        let isolated = world.arms[baseline_index].isolated.contains(&50176);
        assert!(isolated, "mkfs 实例表片 50176 仍被 B 的根引用，应被影子账隔离（真实实现同一测试文件头注释）");
        let reused_entry = world.arms[baseline_index].ledger.get(&reused_slot).expect("复用的槽必有账目条目");
        println!(
            "S1C reuse_data_unit_slot={reused_slot} reused_record_allocation_generation={} reused_record_released={}",
            reused_entry.allocation_generation, reused_entry.released
        );
    }

    /// A2：W 的 txg mod 3 决定暖机次数。
    #[test]
    fn warm_up_publish_count_depends_on_txg_mod_three() {
        assert_eq!(ROOT_RING_REGION_DEVICE[0], Device::Disk0);
        assert_eq!(ROOT_RING_REGION_DEVICE[1], Device::Disk1);
        assert_eq!(ROOT_RING_REGION_DEVICE[2], Device::Disk0);
        // 模拟：从 txg u 开始暖机需要几次才覆盖两块盘（A2：0/1 → 1 次，2 → 2 次）。
        for (start_txg_mod_three, expected_warm_ups) in [(0u64, 1u64), (1, 1), (2, 2)] {
            let mut covered: BTreeSet<Device> = BTreeSet::new();
            covered.insert(ROOT_RING_REGION_DEVICE[start_txg_mod_three as usize]);
            let mut count = 0u64;
            let mut txg = start_txg_mod_three;
            while covered.len() < 2 {
                txg += 1;
                covered.insert(ROOT_RING_REGION_DEVICE[(txg % 3) as usize]);
                count += 1;
            }
            assert_eq!(count, expected_warm_ups, "start_txg_mod_three={start_txg_mod_three}");
        }
    }

    /// 回归测试（2026-09-17，第三段）：`rebuild_allocator` 的基线分支漏减 `baseline_free_slots`
    /// 那一处 bug（见该函数「修过」文档注释）——普通挂载重建之后，item1+item2 必须仍等于容量。
    /// S=4 时 3N=36 次连续覆盖写让环转三圈，released-但未达门槛的槽有 110 个，改前 sum 会多 110。
    #[test]
    fn rebuild_allocator_keeps_baseline_item1_plus_item2_at_capacity_across_remount() {
        let cell = GeometryCell { slots_per_region: 4, placement: PlacementPolicy::Impl, workload_period: 1 };
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {
        }
        let ring_capacity = cell.ring_capacity();
        for outcome in world.run_workload_segment(3 * ring_capacity, 1) {
            let row = outcome.accounting[World::arm_index(Arm::Baseline)];
            assert_eq!(row.item1_occupied_slots + row.item2_free_slots, UNIT_AREA_CAPACITY_SLOTS, "重开取号前 txg={}", outcome.txg);
        }
        for outcome in world.plain_remount() {
            let row = outcome.accounting[World::arm_index(Arm::Baseline)];
            assert_eq!(
                row.item1_occupied_slots + row.item2_free_slots,
                UNIT_AREA_CAPACITY_SLOTS,
                "重建（普通挂载）之后 label={} txg={}",
                outcome.label,
                outcome.txg
            );
        }
        for outcome in world.run_workload_segment(3 * ring_capacity, 1) {
            let row = outcome.accounting[World::arm_index(Arm::Baseline)];
            assert_eq!(row.item1_occupied_slots + row.item2_free_slots, UNIT_AREA_CAPACITY_SLOTS, "重开取号后 txg={}", outcome.txg);
        }
    }

    /// 第三段冒烟：新增的六族历史函数在全部 12 格上都跑得完、不 panic，且落地一批基本形状——
    /// 每族至少推进了 txg、Q3（I-5.2）全程不红（模型构造的两条累加路径应当恒等，PC-C′ 的变异改的是
    /// 「让它不恒等」那一条独立路径，不影响正常跑）。
    #[test]
    fn all_history_families_run_to_completion_on_every_geometry_cell() {
        for cell in all_geometry_cells() {
            {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let outcomes = world.history_plain_remount_midway();
                assert!(!outcomes.is_empty(), "H2 s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period);
                for outcome in &outcomes {
                    for index in 0..5 {
                        assert!(!outcome.invariant_snapshot.free_statistic_mismatch[index], "H2 I-5.2 红：s={} txg={} arm_index={index}", cell.slots_per_region, outcome.txg);
                    }
                }
            }
            for phi in [0u64, 1, 2] {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let outcomes = world.history_root_slot_write_failure_and_switch(phi, false);
                assert!(!outcomes.is_empty(), "H3phi={phi} s={}", cell.slots_per_region);
            }
            {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let outcomes = world.history_root_slot_write_failure_and_switch(0, true);
                assert!(!outcomes.is_empty(), "H3^2 s={}", cell.slots_per_region);
                assert!(
                    outcomes.iter().filter(|outcome| outcome.label == "write_row_root_slot_write_failure").count() >= 1,
                    "H3^2 必须先看到写行也失败一次：s={}",
                    cell.slots_per_region
                );
            }
            for rollback_depth in [2u64, cell.ring_capacity() - 2] {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let result = world.history_single_rollback(rollback_depth);
                assert!(result.is_some(), "H4 rollback_depth={rollback_depth} 不该无候选：s={} N={}", cell.slots_per_region, cell.ring_capacity());
            }
            // H5早/H5晚/H6 的第二次回退目标理论上仍可能因几何极端而不在候选集里——第十一节 V8「写死的
            // 回退目标不存在时记『无候选』，不计入任何量」，`None` 是合法结果，不是 panic 条件；这里只
            // 要求「调用不 panic」。**实测（第四段）在全部 12 格上两族都返回 `Some`**，见
            // `double_rollback_families_find_second_rollback_candidate_on_every_geometry_cell` 那条更强的断言；
            // Q13 的条数统计留给第三段主报告的族历史扫描（main()）。
            {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let _ = world.history_double_rollback_shallower_second();
            }
            {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let _ = world.history_double_rollback_to_first_rollback_write_row();
            }
            {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let _ = world.history_raise_floor_then_rollback();
            }
            {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let _ = world.history_rollback_deep_then_raise_floor();
            }
        }
    }

    /// 回归测试（2026-09-17，第四段，见跑前登记第十二节）：`history_double_rollback_shallower_second`
    /// 与 `history_double_rollback_to_first_rollback_write_row` 此前借用 `history_single_rollback`
    /// 当「H4（d=S）」前缀，把 H4d 自己的尾段工作负载段 3N 也借了进去——环转过好几圈之后第一次回退的
    /// 目标早被轮转挤出候选集，第二次回退在全部 12 格上都找不到候选，H5早 / H5晚 24 行 q_family 只有
    /// `q13_no_candidate=true`，5 条臂一次都没跑起来。改用不含尾段的 `history_rollback_prefix` 之后，
    /// 两族历史必须在全部 12 格上都跑出两次回退（各一条 `label == "write_row"`）。
    #[test]
    fn double_rollback_families_find_second_rollback_candidate_on_every_geometry_cell() {
        for cell in all_geometry_cells() {
            {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let outcomes = world.history_double_rollback_shallower_second();
                let outcomes = outcomes.unwrap_or_else(|| {
                    panic!("H5早 不该无候选：s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period)
                });
                let write_row_count = outcomes.iter().filter(|outcome| outcome.label == "write_row").count();
                assert_eq!(write_row_count, 2, "H5早 必须两次回退各一条写行发布：s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period);
            }
            {
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {}
                let outcomes = world.history_double_rollback_to_first_rollback_write_row();
                let outcomes = outcomes.unwrap_or_else(|| {
                    panic!("H5晚 不该无候选：s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period)
                });
                let write_row_count = outcomes.iter().filter(|outcome| outcome.label == "write_row").count();
                assert_eq!(write_row_count, 2, "H5晚 必须两次回退各一条写行发布：s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period);
            }
        }
    }
}


