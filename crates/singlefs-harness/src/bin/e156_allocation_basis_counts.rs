//! E156 重跑（第 2 次）：alloc-basis 四条岔路的代价数。第一段（岔路 7、defer 的检查）与它前置的 S1/锚点，
//! 登记在 `research/prompts/e156-r2-prereg.md`「五、5.7」第一段。第二段（岔路 3、两个 F 的口径）与第三段
//! （岔路 1、记账第 1 项与洞）仍是**缩小范围版**：第二段（HF）只在 S = 8（mkfs 默认）、ρ = 1 一个几何取样点
//! 上跑一条代表性历史，不是登记「五、5.2」HF 的 18 格全扫；第三段（Hh）在 ρ = 1、回收时点「实」、洞位置
//! 「后」固定的前提下，S 这一维跑了 S = 8（mkfs 默认）与 S = 4（下界，方向相反）两个取样点、k ∈ {0, 1, 2}，
//! 不是 S ∈ {4, 8, 16} × ρ ∈ {1, 1/4} × (k, 位置) 的全部取样点组合——
//! 缩小范围的理由与还差什么写在 `research/prompts/e156-r2-prereg.md` 的「修订」与交回报告里，不在这里重复。
//! 只读驱动 `singlefs-core` / `singlefs-checker` 的公开入口（mkfs、暖机、第一个事务、覆盖写、空发布、
//! 可写挂载、管理员回退、抬回退下界），不改这两个 crate 的生产代码；产物按 `E7RESULT` 行打印，复跑登记在
//! `research/scripts/replay.sh`。
//!
//! R3（G27 改读镜像）：这一版 G27 的判定统一由 [`allocated_minus_deferred_mismatches_referenced`] 给出，只解析**镜像字节**（记账树叶里的
//! 统计量行），不读内存里的 `PoolAllocator`；`allocated_minus_deferred_matches_referenced` 保留成内存读法，
//! 只给 U8 的变异反面用（`crates/mutations.tsv` M21）。

use std::collections::{BTreeMap, BTreeSet, HashMap};

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{
    DeviceFreeMap, Placement, PoolAllocator, ReclaimedReuse, ReuseWindow,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, reclaim_floor, MountError,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_system_configuration, effective_rollback_floor, readable_roots, scan_journal, PoolReader,
};
use singlefs_core::root_ring::{
    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_version, warm_up, FirstFile,
    InstanceTablePlan, PoolWriter, PublishError, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
    UNIT_AREA_START_SLOT,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};

/// E156 装置的固定 fsid：与别的 `eNNN` 装置各自独立取一份，登记在这里方便复跑时核对。
const E156_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x35, 0x36, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const IMAGE_BYTES: u64 = 4 << 30;
/// 一个单元的槽数（跨度 2 的数据单元 / inode 叶容器 / 实例表单元，登记「一」读法写死）。
const INSTANCE_TABLE_SPAN_SLOTS: u64 = 2;
/// 这个装置建的池的每区槽数 S：**本地常量**，值抄自 `.claude/kb/layout/01-first-txn.md` 一「系统配置字段表」
/// 的「每区槽数 S」那一行（`| 几何 | 每区槽数 S | 1 | 8 |`）。
///
/// 不从 `crates/` 引它（`.claude/agents/experiment-runner.md` 入库装置第 ① 条）：装置的工作负载长度与实现的环长
/// 是同一个 S 的两个下游，[`main`] 开头那条断言把它与实现侧现在真用的那个值（读自系统配置）回比。
const E156_SLOTS_PER_REGION: u64 = 8;
/// A1：单元区起点槽号。抄自 D3（空间分配） 已定项 7 依据表「落点」行（`.claude/kb/decisions/03-空间分配.md`；
/// D18（块里携带什么信息） 已定项 9 逼出的粒度）与 `crates/singlefs-format` 的同名常量——下面第一条断言把它与
/// `UNIT_AREA_START_SLOT` 回比。
const E156_UNIT_AREA_START_SLOT: u64 = 50176;
/// A7：分配记录节点容量。抄自 `research/prompts/e156-r2-prereg.md` 表头「分配记录节点容量」行的算式
/// `⌊(16384 − 135) ÷ 20⌋`；节点头 135 字节 = 86（固定头）+ 2 × 10（子节点指针，容量算式的一部分）+ 29（其余头字段）。
const E156_ALLOCATION_RECORD_NODE_HEADER_BYTES: u64 = 86 + 2 * 10 + 29;
const E156_ALLOCATION_RECORD_BYTES: u64 = 20;
/// S1(f)：第一个事务之后的分配记录条数。E156 第 3 次重跑登记「七」7.2 K1-2（分配记录树按位置寻址之后，
/// 五个树节点各占一条记录，28 = 2 × 14）；不再是登记第一、二版的 20（那时树只占 1 条记录）。
const E156_FIRST_TRANSACTION_EXPECTED_RECORD_COUNT: usize = 28;
/// R-3 本地常量①：分配记录树叶宽 W。**本地常量，值抄自 kb，不从 `crates/` 引**（`.claude/agents/
/// experiment-runner.md` 入库装置第 ① 条）：抄自 `.claude/kb/decisions/08-核心索引结构.md:251`
/// `<!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->`。下面 `anchor_a_d8` 那一行把它
/// 与 `singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS` 回比（只观测，F21：对不上不作废、不停机）。
const E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// R-3 本地常量②：分配记录树内部扇出 F。抄自 `.claude/kb/decisions/08-核心索引结构.md:254`
/// `<!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->`。
const E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// R-3：一次覆盖写里，非分配记录树自身的「其余单元」在 D0 上的释放槽数（E156 第 3 次重跑登记「七」7.2
/// R-3：数据 2 + extent 1 + inode 叶 2 + inode 根 1 + 记账 1 + 映射 1 + 树表 1 = 9）。
const E156_OVERWRITE_OTHER_UNITS_RELEASED_SLOTS: u64 = 9;
/// R-3：同一批「其余单元」里会产生新分配记录条目的槽数（不含数据：数据槽被 `data_slot()` 复用同一个
/// 已有的记录条目，不产生新条目，登记「七」7.2 R-4 的算术）。
const E156_OVERWRITE_OTHER_UNITS_NEW_RECORD_SLOTS: u64 = 7;
/// R-3：一次空发布里，非分配记录树自身的「其余单元」（记账 1 + 映射 1 + 树表 1 = 3，三者都走 bump、
/// 每次都是新槽，释放与新增同值，登记「七」7.2 R-5）。
const E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS: u64 = 3;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E156_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 「五、5.6」几何敏感性第六类：**S** 这一维的另一个取样点（方向与 mkfs 默认 8 相反，取下界 4）。
/// 除 `root_ring_slots_per_region` 外与 [`parameters`] 逐字段相同——S 走 mkfs 参数（登记「三」I10），
/// 不必在仓副本里改常量重编。
fn parameters_with_slots_per_region(count: u64) -> MakeFilesystemParameters {
    let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(count)
        .expect("S 落在 4..16 闭区间");
    MakeFilesystemParameters {
        filesystem_identifier: E156_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: slots_per_region,
        },
    }
}

// ============================================================================================
// R-3（E156 第 3 次重跑登记「七」7.2）：分配记录树按位置寻址之后，节点数不再是常数，只能从「这次发布
// 实际改动的记录的槽号」现算。这一段只用 [`E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS`] /
// [`E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`] 两个本地常量与 `allocator.records()` 的真实读数，
// 不调用 `crates/singlefs-core::allocation_record_tree` 的任何几何函数——那些函数就是被测的实装本身，
// 拿它们来算「期望值」会让期望值与实装共用同一处错误。
// ============================================================================================

/// 一个槽所在的分配记录树叶位置 ⌊s ÷ W⌋（本地常量 W）。
fn e156_leaf_position_of_slot(slot: u64) -> u64 {
    slot / E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
}

/// 一个叶位置所在的层级 1 位置 ⌊k ÷ F⌋（本地常量 F）。
fn e156_level1_position_of_leaf(leaf: u64) -> u64 {
    leaf / E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
}

/// R-3：一次发布里分配记录树自身新写的节点数 = `devices × 叶数 + devices × 层级 1 数 + 1`（根只有一份，
/// 两块盘共享；叶与层级 1 逐盘各一份，登记「七」7.2 R-3 逐字）。
fn e156_allocation_record_tree_new_node_count(touched_leaves: &BTreeSet<u64>, devices: u64) -> u64 {
    let touched_level1: BTreeSet<u64> = touched_leaves
        .iter()
        .map(|&leaf| e156_level1_position_of_leaf(leaf))
        .collect();
    devices * u64::try_from(touched_leaves.len()).expect("叶数落在 u64 内")
        + devices * u64::try_from(touched_level1.len()).expect("层级 1 数落在 u64 内")
        + 1
}

/// R-3：这次发布里被换下（COW 释放）的分配记录树旧节点数——只有这次触达、且这个位置在这次发布之前
/// 已经有节点（叶位置 ∈ `existing_leaves`，或它的层级 1 位置 ∈ 现有层级 1 集合）的那些才会释放旧版本。
fn e156_allocation_record_tree_replaced_node_count(
    touched_leaves: &BTreeSet<u64>,
    existing_leaves: &BTreeSet<u64>,
    devices: u64,
) -> u64 {
    let touched_level1: BTreeSet<u64> = touched_leaves
        .iter()
        .map(|&leaf| e156_level1_position_of_leaf(leaf))
        .collect();
    let existing_level1: BTreeSet<u64> = existing_leaves
        .iter()
        .map(|&leaf| e156_level1_position_of_leaf(leaf))
        .collect();
    let leaves_kept = touched_leaves.intersection(existing_leaves).count();
    let level1_kept = touched_level1.intersection(&existing_level1).count();
    devices * u64::try_from(leaves_kept).expect("叶交集数落在 u64 内")
        + devices * u64::try_from(level1_kept).expect("层级 1 交集数落在 u64 内")
        + 1
}

/// R-3：D0 上这次发布之前，全部分配记录（不论已释放还是仍分配）所在的叶位置集合——「这个位置本来
/// 有没有节点」的判据（D8（核心索引结构） 已定项 14「没有记录的一段 = 全空闲：那片叶不写」）。
fn e156_existing_leaf_positions(allocator: &PoolAllocator, device: DeviceIdentity) -> BTreeSet<u64> {
    allocator
        .records()
        .iter()
        .filter(|record| record.device == device)
        .map(|record| e156_leaf_position_of_slot(record.slot.0))
        .collect()
}

/// R-3：D0 上这次发布翻成已释放或新加的全部记录所在的叶位置集合——真实记录的 `generation` 字段
/// 在被触达的那一刻（无论新分配还是刚被释放）都改写成这次的 txg（D3（空间分配） 已定项 7 逐字），
/// 用它筛出「这次改动的记录」不需要另外做前后快照 diff。
fn e156_touched_leaf_positions(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
    txg: CheckpointTxg,
) -> BTreeSet<u64> {
    allocator
        .records()
        .iter()
        .filter(|record| record.device == device && record.generation == txg)
        .map(|record| e156_leaf_position_of_slot(record.slot.0))
        .collect()
}

/// R-3：一次发布（覆盖写或空发布）D0 释放槽数与 D0+D1 记录增量的闭式期望值。
/// `existing_leaves`：这次发布之前 D0 上已有记录的叶位置集合；`touched_leaves`：这次发布之后
/// D0 上 `generation == 本次 txg` 的记录所在的叶位置集合。
fn e156_closed_form_expected(
    existing_leaves: &BTreeSet<u64>,
    touched_leaves: &BTreeSet<u64>,
    is_overwrite: bool,
) -> (u64, u64) {
    let new_nodes = e156_allocation_record_tree_new_node_count(touched_leaves, 2);
    let replaced_nodes =
        e156_allocation_record_tree_replaced_node_count(touched_leaves, existing_leaves, 2);
    let (other_released, other_new_records) = if is_overwrite {
        (
            E156_OVERWRITE_OTHER_UNITS_RELEASED_SLOTS,
            E156_OVERWRITE_OTHER_UNITS_NEW_RECORD_SLOTS,
        )
    } else {
        (
            E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS,
            E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS,
        )
    };
    (
        other_released + replaced_nodes,
        2 * (other_new_records + new_nodes),
    )
}

/// R-7：一次推空发布（只改一片叶时）自己的固定点槽数——单设备的「新增记录」那一半，不乘 2（登记
/// 「七」7.2 R-5「一次推空发布只改一片叶时的固定点 8 槽」，与 D0+D1 合计的记录增量相差一个 devices 因子）。
fn e156_empty_publish_fixed_point_slots(existing_leaves: &BTreeSet<u64>) -> u64 {
    E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS
        + e156_allocation_record_tree_new_node_count(existing_leaves, 2)
}

/// R-3 本地常量③：根层级规则——最小的 R ≥ 1 使 Σ_盘 ⌈盘上槽数 ÷ (W × F^(R−1))⌉ ≤ F（D8（核心索引结构）
/// 已定项 14「根罩整个 key 空间、按盘分流」那一行）。只用来与实装的 `AllocationRecordTreeGeometry`
/// 回比（A-D8，只观测，F21：对不上不作废、不停机），不供 R-3 的节点数闭式使用。
fn e156_root_level_for_symmetric_devices(unit_area_slots_per_device: u64, device_count: u64) -> u8 {
    let mut level: u32 = 1;
    loop {
        let child_span = E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
            * E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level - 1);
        let total_cells = device_count * unit_area_slots_per_device.div_ceil(child_span);
        if total_cells <= E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
            return u8::try_from(level).expect("根层级落在 u8 内（池子不会大到需要 256 层）");
        }
        level += 1;
    }
}

/// Q7d-1（R2 ⑤）：一组「这次发布自己的释放」样本的 (最小值, 最大值)——main() 与 U12 单测共用同一个
/// 函数，M30（Q7d-1 覆盖写那一组退回字面 10）改这里，两处才会一起红。
fn e156_minimum_and_maximum(samples: &[u64]) -> (u64, u64) {
    (
        samples.iter().min().copied().unwrap_or(0),
        samples.iter().max().copied().unwrap_or(0),
    )
}

/// R-7：一块盘开放聚簇段里当前的空闲槽数（登记「五」5.1 HY「e ≥ max(8, f)」的 e）。
fn e156_open_segment_free_slots(allocator: &PoolAllocator, device: DeviceIdentity) -> u64 {
    let Some(open_segment_start) = allocator.open_segment() else {
        return 0;
    };
    let device_map = allocator
        .devices
        .iter()
        .find(|map| map.device == device)
        .expect("这块盘在池里");
    (0..CLUSTER_SEGMENT_SLOTS)
        .filter(|offset| device_map.is_free(SlotNumber(open_segment_start.0 + offset)))
        .count() as u64
}

/// 覆盖写第 `step` 次的文件内容：定长 3000 字节，字节序列随 `step` 变，保证每次覆盖写都改变用户可见状态（骨架发布 的 O）。
fn overwrite_content(step: u64) -> Vec<u8> {
    let salt = usize::try_from(step % 251).expect("对 251 取模落在 usize 里");
    (0..3000usize)
        .map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256"))
        .collect()
}

struct Emitter {
    emitted: u64,
}
impl Emitter {
    fn emit(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

/// 一块盘的记账第 1/2/5 项，槽数：直接读真实 `PoolAllocator`。只给 S1/Q7d-1 的「这一刻分配器自己的账」用，
/// **不给 G27 用**（G27 一律读镜像，见 [`allocated_minus_deferred_mismatches_referenced`]）。
fn accounting_row_slots(allocator: &PoolAllocator, device: DeviceIdentity) -> (u64, u64, u64) {
    let map = allocator
        .devices
        .iter()
        .find(|map| map.device == device)
        .expect("两块盘都在");
    (
        map.allocated_slots(),
        map.free_slots(),
        map.deferred_slots(),
    )
}

/// G27 的「最新根走读引用」：`current.units` 是这一版全部角色的落点，实例表单元在它第一次被重写之前不进这张表
/// （`transaction.rs` 文档字面），这里据此补上它——与字节级走读（`walk::check_pool_image`）是两条独立路径。
fn referenced_slots(current: &TransactionOutput) -> u64 {
    let has_instance_table = current
        .units
        .iter()
        .any(|unit| matches!(unit.identity, TransactionUnit::InstanceTable));
    let mut total: u64 = current
        .units
        .iter()
        .map(|unit| unit.identity.span_slots())
        .sum();
    if !has_instance_table {
        total += INSTANCE_TABLE_SPAN_SLOTS;
    }
    total
}

/// 内存读法（R3 的反面）：只给 `crates/mutations.tsv` 的 M21 当反面用（U8 的调用点被换成它），
/// 正式判定路径谁都不叫它。
fn allocated_minus_deferred_matches_referenced(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
    referenced: u64,
) -> bool {
    let (allocated, _free, deferred) = accounting_row_slots(allocator, device);
    allocated == deferred + referenced
}

fn mp_read(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(
        pool,
        DeviceIdentity(device),
        DeviceOffsetInBytes(offset),
        length,
    )
    .expect("读")
}
fn mp_write(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    pool.devices
        .get_mut(&DeviceIdentity(device))
        .expect("盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

/// 记账树叶里 (统计量, 设备 0) 那一行的值，**单位字节**：直接从**镜像字节**解析（与 `adjust_accounting_entry`
/// 同一份布局：条目区从 115 + 2 × 22 = 159 起，每条 34：统计量 2、树 8、设备 4、代 8、值 8、seq 4），只读不改。
fn read_accounting_entry_bytes_from_mirror(
    pool: &MemoryPool,
    accounting_slot: u64,
    statistic: u16,
) -> u64 {
    let bytes = mp_read(pool, 0, accounting_slot * SLOT_BYTES, 16384);
    let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
    for index in 0..count {
        let row = 159 + 34 * index;
        if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
            && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == 0
        {
            return u64::from_le_bytes(bytes[row + 22..row + 30].try_into().expect("8"));
        }
    }
    panic!("记账树里没有 (统计量 {statistic}, 设备 0) 这一行");
}

/// R3：G27、Q7b、Q7d 一律读镜像上的记账行（第 1/2/5 项），单位槽。记账单元是镜像对（w = 2），
/// `corrupt_device_zero_accounting` 只改盘 0 那一份，这里也只读盘 0，两处同一个读法。
fn mirror_accounting_row_slots(pool: &MemoryPool, accounting_slot: u64) -> (u64, u64, u64) {
    (
        read_accounting_entry_bytes_from_mirror(pool, accounting_slot, 1) / SLOT_BYTES,
        read_accounting_entry_bytes_from_mirror(pool, accounting_slot, 2) / SLOT_BYTES,
        read_accounting_entry_bytes_from_mirror(pool, accounting_slot, 5) / SLOT_BYTES,
    )
}

/// G27 正式判定（R3）：逐盘（只判盘 0，两盘镜像同值）`第 1 项 − 第 5 项 == 最新根走读引用`，一律读镜像字节。
fn allocated_minus_deferred_mismatches_referenced(
    pool: &MemoryPool,
    accounting_slot: u64,
    referenced: u64,
) -> bool {
    let (allocated, _free, deferred) = mirror_accounting_row_slots(pool, accounting_slot);
    allocated != deferred + referenced
}

/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
/// 与 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `reseal_unit` 同一份布局（两处各自独立实现，字段表出自
/// `singlefs-format`，不是互相抄）。
fn reseal_unit(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[6] {
        1 => (105usize, 101usize),
        3 => (107, 89),
        _ => (
            86 + 2 * usize::from(bytes[51]),
            76 + 2 * usize::from(bytes[51]),
        ),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

/// 记账树叶里 (统计量, 设备) 那一行的值加 `delta_bytes`（与 `checker_known_bad_images.rs` 的 `adjust_accounting` 同一份布局：
/// 条目区从 115 + 2 × 22 = 159 起，每条 34：统计量 2、树 8、设备 4、代 8、值 8、seq 4）。
fn adjust_accounting_entry(bytes: &mut [u8], statistic: u16, device: u32, delta_bytes: i64) {
    let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
    for index in 0..count {
        let row = 159 + 34 * index;
        if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
            && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
        {
            let value = u64::from_le_bytes(bytes[row + 22..row + 30].try_into().expect("8"));
            let new_value = value.wrapping_add_signed(delta_bytes);
            bytes[row + 22..row + 30].copy_from_slice(&new_value.to_le_bytes());
            return;
        }
    }
    panic!("记账树里没有 ({statistic}, {device}) 这一行");
}

fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    [
        device.to_le_bytes().to_vec(),
        slot.to_le_bytes()[..6].to_vec(),
        checksum.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
    let mut changed = false;
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        if haystack[index..index + needle.len()] == *needle {
            haystack[index..index + needle.len()].copy_from_slice(replacement);
            changed = true;
            index += needle.len();
        } else {
            index += 1;
        }
    }
    changed
}

/// 根环全部槽的 (设备, 偏移)：按公开的 `root_ring` 几何现算，不写死字节偏移。
fn root_ring_slots(region_devices: &[DeviceIdentity; 3], spacing: u32) -> Vec<(u32, u64)> {
    (0..ROOT_RING_REGIONS)
        .flat_map(|region| {
            let device = region_devices[usize::try_from(region).expect("区域号落在 3 以内")].0;
            (0..E156_SLOTS_PER_REGION).map(move |slot| {
                (
                    device,
                    slot_offset(RootRingSlot { region, slot }, spacing).0,
                )
            })
        })
        .collect()
}

/// 只改盘 0 那一份记账节点：重封盘 0 那一份的两道校验和，把新的整单元校验和补进盘 0 自己那份树表与根槽——
/// 盘 1 一个字节都不碰（`checker_known_bad_images.rs` 的 `mutate_one_device_copy_of_unit` 同一条改法，这里独立实现）。
/// Bd(k) 与 Bk1/Bk2 都走这一条：`entries` 是这次要同时改的 (统计量, delta 字节) 列表。
fn corrupt_device_zero_accounting(
    base: &MemoryPool,
    accounting_slot: u64,
    tree_table_slot: u64,
    region_devices: &[DeviceIdentity; 3],
    spacing: u32,
    entries: &[(u16, i64)],
) -> MemoryPool {
    let mut pool = base.clone();
    let device = 0u32;
    let before = mp_read(&pool, device, accounting_slot * SLOT_BYTES, 16384);
    let mut bytes = before.clone();
    for (statistic, delta_bytes) in entries.iter().copied() {
        adjust_accounting_entry(&mut bytes, statistic, device, delta_bytes);
    }
    reseal_unit(&mut bytes);
    mp_write(&mut pool, device, accounting_slot * SLOT_BYTES, &bytes);
    let (old_checksum, new_checksum) = (crc32_castagnoli(&before), crc32_castagnoli(&bytes));

    // 树表：只改盘 0 那一份，重封，再把新校验和补进盘 0 与盘 1 各自的树表条目里指着「盘 0 那份记账」的那一条位置条目
    // （两块盘的树表内容原本相同，位置条目本身按设备分两条，各自独立记着盘 0 与盘 1 各一份的校验和）。
    let mut tree_table_changed_on = Vec::new();
    for tree_table_device in [0u32, 1u32] {
        let tree_table_before = mp_read(
            &pool,
            tree_table_device,
            tree_table_slot * SLOT_BYTES,
            16384,
        );
        let mut tree_table_bytes = tree_table_before.clone();
        if replace_all(
            &mut tree_table_bytes,
            &location_pattern(device, accounting_slot, old_checksum),
            &location_pattern(device, accounting_slot, new_checksum),
        ) {
            reseal_unit(&mut tree_table_bytes);
            mp_write(
                &mut pool,
                tree_table_device,
                tree_table_slot * SLOT_BYTES,
                &tree_table_bytes,
            );
            tree_table_changed_on.push((
                tree_table_device,
                crc32_castagnoli(&tree_table_before),
                crc32_castagnoli(&tree_table_bytes),
            ));
        }
    }
    for (root_device, offset) in root_ring_slots(region_devices, spacing) {
        let mut root_bytes = mp_read(&pool, root_device, offset, 512);
        let mut touched = false;
        for (tree_table_device, old_tt, new_tt) in tree_table_changed_on.iter().copied() {
            touched |= replace_all(
                &mut root_bytes,
                &location_pattern(tree_table_device, tree_table_slot, old_tt),
                &location_pattern(tree_table_device, tree_table_slot, new_tt),
            );
        }
        if touched {
            let digest = wide_checksum_with_field_zeroed(&root_bytes, 512, 138);
            root_bytes[138..170].copy_from_slice(&digest);
            mp_write(&mut pool, root_device, offset, &root_bytes);
        }
    }
    pool
}

fn memory_pool_snapshot(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 反方向：从一份镜像重建出可写的设备 Vec（c2 崩溃恢复、S1(c) 重开都要这一条）。
fn devices_from_memory_pool(pool: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    pool.devices
        .iter()
        .map(|(identity, image)| {
            let mut device =
                SparseBlockDevice::new(pool.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = image.clone();
            (*identity, device)
        })
        .collect()
}

fn verdict(pool: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(pool)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .map(|(_, verdict)| verdict)
        .unwrap_or(InvariantVerdict::NotApplicable("checker 没有报这一条"))
}
fn is_red(verdict: &InvariantVerdict) -> bool {
    matches!(verdict, InvariantVerdict::Violated(_))
}

/// 一次发布落在根环上的 (设备, 偏移)：`target_for_publish` + `slot_offset` 都是公开的几何函数，
/// 不是登记第 ① 条禁的「决定工作量/几何/门槛的常量」。
fn root_slot_location(
    txg: CheckpointTxg,
    region_devices: &[DeviceIdentity; 3],
    spacing: u32,
    slots_per_region: RootRingSlotsPerRegion,
) -> (u32, u64) {
    let target = target_for_publish(txg, slots_per_region);
    let device = region_devices[usize::try_from(target.region).expect("区域号落在 3 以内")].0;
    (device, slot_offset(target, spacing).0)
}

/// 空发布 E：`publish_version` 带 `file: None`、`new_inode_records` 为空（骨架发布 字面），与 `publish_overwrite`
/// 同一条发布路径，只是不带文件计划（形态照 `transaction.rs` 里 `publish_new_inodes` 构造 `PublishPlan` 的写法）。
fn publish_empty<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
    let mut pool = PoolWriter::new(parameters, devices);
    publish_version(
        &mut pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// R8「这次发布自己的释放」：D0 上 `is_released` 且 `generation == txg` 的记录，跨度求和，单位槽（登记「六」R8 字面）。
fn self_release_slots_of_this_publish(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
    txg: CheckpointTxg,
) -> u64 {
    allocator
        .records()
        .iter()
        .filter(|record| record.device == device && record.is_released && record.generation == txg)
        .map(|record| u64::from(record.span_slots))
        .sum()
}

/// c2 崩溃（记录已持久、根槽没持久）：正常发一次真实覆盖写（record + root 都写），再把**这次的根槽**（512 字节，
/// 单设备不镜像）改回发布之前的旧内容——效果与在这次发布的写序列里、根槽那一步之前截断逐字节相同（两者只是
/// 写序列真正落盘的先后不同，最终镜像字节一样），比重新实现一整套录制/分段机制更直接、也更贴 C380 的措辞
/// 「一个根槽从没写过的 txg」。
#[allow(
    clippy::too_many_arguments,
    reason = "c2 崩溃要凑齐设备、分配器、上一版、内容、时间、实例、几何六类各自独立的参数"
)]
fn crash_before_root_persists(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    content: &[u8],
    write_time_seconds: u64,
    instance: InstanceGeneration,
    region_devices: &[DeviceIdentity; 3],
    spacing: u32,
    slots_per_region: RootRingSlotsPerRegion,
) -> MemoryPool {
    let before = memory_pool_snapshot(devices);
    let published = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            allocator,
            previous,
            FirstFile {
                content,
                write_time_seconds,
            },
            instance,
        )
        .expect("HK c2 崩溃之前的覆盖写（记录会持久，根槽本函数之后手动撤回）")
    };
    let (root_device, root_offset) = root_slot_location(
        published.root.checkpoint_txg,
        region_devices,
        spacing,
        slots_per_region,
    );
    let old_root_bytes = mp_read(&before, root_device, root_offset, 512);
    let mut c2_pool = memory_pool_snapshot(devices);
    mp_write(&mut c2_pool, root_device, root_offset, &old_root_bytes);
    c2_pool
}

/// 一条合法状态（登记「五」骨架逐行「逐发布一行」+ Q7b/Q7d 要的那几个量），G27 一律读镜像（R3）。
#[derive(Clone)]
struct LegalState {
    family: String,
    kind: &'static str,
    txg: u64,
    item1: u64,
    item2: u64,
    item5: u64,
    referenced: u64,
    check_is_red: bool,
}

/// 一个「基底」：Q7a/Q7c/PC-检查/Q7f 共用的快照（镜像 + 它的记账/树表单元槽号 + 走读引用）。
#[derive(Clone)]
struct BasisSnapshot {
    label: String,
    pool: MemoryPool,
    accounting_slot: u64,
    tree_table_slot: u64,
    referenced: u64,
    /// 这个基底是不是「可达」（登记 R7）：只由产品路径、真实发布走出来。β_syn、β_F0 记 `false`。
    reachable: bool,
}

fn basis_of(
    label: &str,
    pool: &MemoryPool,
    current: &TransactionOutput,
    reachable: bool,
) -> BasisSnapshot {
    BasisSnapshot {
        label: label.to_string(),
        pool: pool.clone(),
        accounting_slot: current.unit(TransactionUnit::AccountingTree).slot.0,
        tree_table_slot: current.unit(TransactionUnit::TreeTable).slot.0,
        referenced: referenced_slots(current),
        reachable,
    }
}

/// 记一条合法状态：推进共享列表、顺带打一行 `E7RESULT`（Q7b 的红数能被一条命令从产物里数出来）。
fn record_legal_state(
    emitter: &mut Emitter,
    legal_states: &mut Vec<LegalState>,
    family: &str,
    kind: &'static str,
    current: &TransactionOutput,
    pool: &MemoryPool,
) -> LegalState {
    let accounting_slot = current.unit(TransactionUnit::AccountingTree).slot.0;
    let referenced = referenced_slots(current);
    let (item1, item2, item5) = mirror_accounting_row_slots(pool, accounting_slot);
    let check_is_red = item1 != item5 + referenced;
    let state = LegalState {
        family: family.to_string(),
        kind,
        txg: current.root.checkpoint_txg.0,
        item1,
        item2,
        item5,
        referenced,
        check_is_red,
    };
    emitter.emit(&format!(
        "name=legal_state family={} kind={} txg={} item1={} item2={} item5={} referenced={} check_is_red={}",
        state.family, state.kind, state.txg, state.item1, state.item2, state.item5, state.referenced, state.check_is_red
    ));
    legal_states.push(state.clone());
    state
}

/// Q7a 一格：造坏镜像 Bd(delta_slots)，报基底与坏镜像各自的 G27、今天两条检查。
/// 返回 (坏镜像上 G27 判红, 今天两条检查是不是都仍是绿的)，供 q7a_summary 用。
#[allow(
    clippy::too_many_arguments,
    reason = "凑齐基底、几何、delta 三类各自独立的参数"
)]
fn run_q7a_cell(
    emitter: &mut Emitter,
    basis: &str,
    delta_label: &str,
    base_pool: &MemoryPool,
    accounting_slot: u64,
    tree_table_slot: u64,
    region_devices: &[DeviceIdentity; 3],
    spacing: u32,
    referenced: u64,
    delta_slots: i64,
) -> (bool, bool) {
    let corrupted_pool = corrupt_device_zero_accounting(
        base_pool,
        accounting_slot,
        tree_table_slot,
        region_devices,
        spacing,
        &[(
            5u16,
            delta_slots * i64::try_from(SLOT_BYTES).expect("16384 落在 i64 里"),
        )],
    );
    let base_red =
        allocated_minus_deferred_mismatches_referenced(base_pool, accounting_slot, referenced);
    let corrupted_red = allocated_minus_deferred_mismatches_referenced(
        &corrupted_pool,
        accounting_slot,
        referenced,
    );
    let base_allocated_statistic_is_red = is_red(&verdict(base_pool, "I-3.1"));
    let base_free_statistic_is_red = is_red(&verdict(base_pool, "I-5.2"));
    let corrupted_allocated_statistic_is_red = is_red(&verdict(&corrupted_pool, "I-3.1"));
    let corrupted_free_statistic_is_red = is_red(&verdict(&corrupted_pool, "I-5.2"));
    emitter.emit(&format!(
        "name=q7a basis={basis} defer_delta={delta_label} base_check_is_red={base_red} base_i31_red={base_allocated_statistic_is_red} base_i52_red={base_free_statistic_is_red} corrupted_check_is_red={corrupted_red} corrupted_i31_red={corrupted_allocated_statistic_is_red} corrupted_i52_red={corrupted_free_statistic_is_red}"
    ));
    (
        corrupted_red,
        !corrupted_allocated_statistic_is_red && !corrupted_free_statistic_is_red,
    )
}

/// PC-检查-Bk1/Bk2/Bk3：三种坏镜像各判一次（原登记 5.5 字面）。
#[allow(
    clippy::too_many_arguments,
    reason = "凑齐基底与几何两类各自独立的参数"
)]
fn run_pc_check(
    emitter: &mut Emitter,
    basis: &str,
    base_pool: &MemoryPool,
    accounting_slot: u64,
    tree_table_slot: u64,
    region_devices: &[DeviceIdentity; 3],
    spacing: u32,
    referenced: u64,
) {
    let bk1_pool = corrupt_device_zero_accounting(
        base_pool,
        accounting_slot,
        tree_table_slot,
        region_devices,
        spacing,
        &[
            (1u16, i64::try_from(SLOT_BYTES).expect("16384")),
            (2u16, -i64::try_from(SLOT_BYTES).expect("16384")),
        ],
    );
    emitter.emit(&format!(
        "name=pc_check_bk1 basis={basis} check_is_red={} i31_red={} i52_red={}",
        allocated_minus_deferred_mismatches_referenced(&bk1_pool, accounting_slot, referenced),
        is_red(&verdict(&bk1_pool, "I-3.1")),
        is_red(&verdict(&bk1_pool, "I-5.2")),
    ));
    let bk2_pool = corrupt_device_zero_accounting(
        base_pool,
        accounting_slot,
        tree_table_slot,
        region_devices,
        spacing,
        &[(2u16, -i64::try_from(SLOT_BYTES).expect("16384"))],
    );
    emitter.emit(&format!(
        "name=pc_check_bk2 basis={basis} check_is_red={} i31_red={} i52_red={}",
        allocated_minus_deferred_mismatches_referenced(&bk2_pool, accounting_slot, referenced),
        is_red(&verdict(&bk2_pool, "I-3.1")),
        is_red(&verdict(&bk2_pool, "I-5.2")),
    ));
    // Bk3 就是 Bd(+1)：单独再造一次镜像，字段名与 q7a 的 corrupted_* 对齐，方便并排读。
    let bk3_pool = corrupt_device_zero_accounting(
        base_pool,
        accounting_slot,
        tree_table_slot,
        region_devices,
        spacing,
        &[(5u16, i64::try_from(SLOT_BYTES).expect("16384"))],
    );
    emitter.emit(&format!(
        "name=pc_check_bk3_is_bd_plus_one basis={basis} check_is_red={} i31_red={} i52_red={}",
        allocated_minus_deferred_mismatches_referenced(&bk3_pool, accounting_slot, referenced),
        is_red(&verdict(&bk3_pool, "I-3.1")),
        is_red(&verdict(&bk3_pool, "I-5.2")),
    ));
}

/// Q7c①②：判别力自证。②在 `deferred == 0` 的基底上（β_syn、β_F0，以及第 10 个基底 `beta_hr_rollback_row`，
/// 见「十二」修订 4）无从执行（Bd(−1) 会把 item5 减成负数，这是写装置时读出的一种真正的未定义输入，不是两种
/// 读法的分歧——见「十二」修订，这里记 `not_applicable=true`，不算进任何门槛）。①在 `beta_hr_rollback_row`
/// 上第一次在**可达**基底上转色（此前 β0/β1/β2/βK-w/r/f/l2 七个可达基底 `deferred` 都 ≥ 1，①恒不转色）。
fn q7c_self_test(
    emitter: &mut Emitter,
    basis: &str,
    base_pool: &MemoryPool,
    accounting_slot: u64,
    referenced: u64,
) -> (bool, bool) {
    let (allocated, _free, deferred) = mirror_accounting_row_slots(base_pool, accounting_slot);
    // ① 不减第 5 项：Bd(+1)。
    let corrupted_deferred_plus1 = deferred + 1;
    let real_check_red_plus1 = allocated != corrupted_deferred_plus1 + referenced;
    let without_subtracting_defer_is_red = allocated != referenced;
    let flip1 = real_check_red_plus1 && !without_subtracting_defer_is_red;
    emitter.emit(&format!(
        "name=q7c1_not_subtracting_defer basis={basis} k=1 real_check_is_red={real_check_red_plus1} without_subtracting_defer_is_red={without_subtracting_defer_is_red} flips_red_to_green={flip1}"
    ));
    // ② `==` → `>=`：Bd(−1)。
    let flip2 = if deferred == 0 {
        emitter.emit(&format!(
            "name=q7c2_threshold_at_least basis={basis} k=-1 not_applicable=true reason=deferred_is_zero_cannot_subtract_one"
        ));
        false
    } else {
        let corrupted_deferred_minus1 = deferred - 1;
        let real_check_red_minus1 = allocated != corrupted_deferred_minus1 + referenced;
        let at_least_holds = allocated >= corrupted_deferred_minus1 + referenced;
        let flip = real_check_red_minus1 && at_least_holds;
        emitter.emit(&format!(
            "name=q7c2_threshold_at_least basis={basis} k=-1 real_check_is_red={real_check_red_minus1} at_least_holds={at_least_holds} flips_red_to_green={flip}"
        ));
        flip
    };
    (flip1, flip2)
}

/// HK / HK-F0 共用的四个观测点快照 + S1(e) 的 E 释放槽数。
struct HkOutcome {
    beta_first: BasisSnapshot,
    beta_after_reopen: BasisSnapshot,
    beta_after_rollback: BasisSnapshot,
    beta_after_raise_floor: BasisSnapshot,
    beta_after_crash_recovery: BasisSnapshot,
    empty_publish_release_slots: u64,
}

/// HK / HK-F0：登记「五、5.2」的固定脚本，`reuse_window` 是唯一的差别（HK-F0 每个分配器都先
/// `set_reuse_window(ForcedToZero)`）。每一种发布（骨架发布）至少出现一次，每次发布之后的状态都推进 `legal_states`
/// （HK 推进外部共享的 `legal_states`；HK-F0 调用方传一个只给它自己用的空列表，不混进「可达」的 Q7b/Q7d 统计）。
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "登记「五、5.2」要求 HK/HK-F0 是同一串真实操作，拆开反而难对拍"
)]
fn run_hk_family(
    parameters: &MakeFilesystemParameters,
    region_devices: &[DeviceIdentity; 3],
    spacing: u32,
    slots_per_region: RootRingSlotsPerRegion,
    label: &str,
    reuse_window: ReuseWindow,
    reachable: bool,
    emitter: &mut Emitter,
    legal_states: &mut Vec<LegalState>,
) -> HkOutcome {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(parameters, &mut devices).expect("HK mkfs");
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator.set_reuse_window(reuse_window);
    let mut instance = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("HK 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("HK 暖机")
    };
    let mut current: TransactionOutput = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("HK 第一个事务")
    };
    record_legal_state(
        emitter,
        legal_states,
        label,
        "first_file",
        &current,
        &memory_pool_snapshot(&devices),
    );
    let beta_first = basis_of(
        &format!("{label}_first"),
        &memory_pool_snapshot(&devices),
        &current,
        reachable,
    );

    // O
    current = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &current,
            FirstFile {
                content: &overwrite_content(1),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 61,
            },
            instance,
        )
        .expect("HK 第一次 O")
    };
    record_legal_state(
        emitter,
        legal_states,
        label,
        "O",
        &current,
        &memory_pool_snapshot(&devices),
    );

    // E（空发布）
    let existing_leaves_before_empty_publish =
        e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
    let records_before_empty_publish = allocator.records().len();
    current = publish_empty(
        parameters,
        devices.as_mut_slice(),
        &mut allocator,
        &current,
        instance,
    )
    .expect("HK 空发布 E");
    let empty_publish_release_slots = self_release_slots_of_this_publish(
        &allocator,
        DeviceIdentity(0),
        current.root.checkpoint_txg,
    );
    // R-5（第七节 7.2）：HK 那一次空发布 D0 释放槽数与记录增量的闭式核对，只在 `label == "HK"`（真实、
    // 可达的那一臂）上钉硬断言；HK-F0 只观测，不 panic（它的可达性本来就不参与 Q7b/Q7d 的统计）。
    let touched_leaves_empty_publish = e156_touched_leaf_positions(
        &allocator,
        DeviceIdentity(0),
        current.root.checkpoint_txg,
    );
    let (expected_empty_publish_released, expected_empty_publish_record_delta) =
        e156_closed_form_expected(
            &existing_leaves_before_empty_publish,
            &touched_leaves_empty_publish,
            false,
        );
    let empty_publish_record_delta = u64::try_from(
        allocator.records().len() - records_before_empty_publish,
    )
    .expect("空发布的记录增量落在 u64 内");
    emitter.emit(&format!(
        "name=r5_empty_publish_closed_form label={label} released_d0={empty_publish_release_slots} expected_released_d0={expected_empty_publish_released} record_delta={empty_publish_record_delta} expected_record_delta={expected_empty_publish_record_delta} fixed_point_slots_one_leaf={}",
        e156_empty_publish_fixed_point_slots(&existing_leaves_before_empty_publish),
    ));
    if label == "HK" {
        assert_eq!(
            empty_publish_release_slots, expected_empty_publish_released,
            "R-5：HK 空发布 D0 释放槽数应等于闭式"
        );
        assert_eq!(
            empty_publish_record_delta, expected_empty_publish_record_delta,
            "R-5：HK 空发布记录增量应等于闭式"
        );
    }
    record_legal_state(
        emitter,
        legal_states,
        label,
        "E",
        &current,
        &memory_pool_snapshot(&devices),
    );

    // 无崩溃重开（W，暖机）
    let mounted = mount_writable(parameters, &mut devices).expect("HK 无崩溃重开");
    allocator = mounted.allocator;
    // `mount_writable` 内部按 `rebuilt_allocator` 重建分配器，复用窗口恒回落到产品默认
    // `GatedByTheRollbackFloor`（`allocator.rs` I14「产品路径恒 GatedByTheRollbackFloor」）——
    // 每次挂载/回退之后都要重新设一次，不是设一次就一直生效。
    allocator.set_reuse_window(reuse_window);
    instance = mounted.output.instance;
    record_legal_state(
        emitter,
        legal_states,
        label,
        "W_row",
        mounted
            .output
            .row_publish
            .file_version()
            .expect("写行那一版带文件"),
        &memory_pool_snapshot(&devices),
    );
    for warm in &mounted.output.warm_up_publishes {
        record_legal_state(
            emitter,
            legal_states,
            label,
            "warmup",
            warm.file_version().expect("暖机那一版带文件"),
            &memory_pool_snapshot(&devices),
        );
    }
    current = mounted
        .current
        .into_file_version()
        .expect("重开后现行版本带文件");
    let beta_after_reopen = basis_of(
        &format!("{label}_w"),
        &memory_pool_snapshot(&devices),
        &current,
        reachable,
    );

    // O × 4
    for step in 0..4u64 {
        current = {
            let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut pool,
                &mut allocator,
                &current,
                FirstFile {
                    content: &overwrite_content(10 + step),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 70 + step,
                },
                instance,
            )
            .expect("HK O×4")
        };
        record_legal_state(
            emitter,
            legal_states,
            label,
            "O",
            &current,
            &memory_pool_snapshot(&devices),
        );
    }

    // 抬 F 到上限（R5：先探上限，再抬到那里）。S9（停机，登记「十一」）：ForcedToZero 下前面几步已经把
    // genesis 的树表单元回收挪作他用（每次 `release()` 会把释放代 ≤ 这次 txg 的全部已释放记录一并回收，
    // 见 `allocator.rs` `ReuseWindow::ForcedToZero` 分支），`rollback_floor_ceiling` 判非空要读候选根的树表，
    // 读不到就报错——这正是 I-7.4（近 K 代块未被复用） 要挡的那类复用，HK-F0 故意不可达，撞上了就停这一格，
    // 不当结果、不作废（S9 原文：「那一格停，记错误原文；全部格都拒 ⇒ 交主 agent」）。
    let probe_result = raise_rollback_floor(
        parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        CheckpointTxg(u64::MAX),
        ShadowLedger::On,
    );
    let probe_ceiling = match probe_result {
        Err(MountError::RollbackFloorAboveCeiling { ceiling, .. }) => ceiling,
        Err(other) => {
            emitter.emit(&format!("name=s9_raise_floor_refused family={label} error={other:?}"));
            return HkOutcome {
                beta_first: beta_first.clone(),
                beta_after_reopen: beta_first.clone(),
                beta_after_rollback: beta_first.clone(),
                beta_after_raise_floor: beta_first.clone(),
                beta_after_crash_recovery: beta_first,
                empty_publish_release_slots,
            };
        }
        Ok(_) => panic!("HK 抬 F 探测上限期望 RollbackFloorAboveCeiling，用 u64::MAX 探测却成功了（S4：写装置时读出的分歧，交主 agent）"),
    };
    let raised = raise_rollback_floor(
        parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        probe_ceiling,
        ShadowLedger::On,
    )
    .expect("HK 抬 F 到上限");
    for publish in &raised.publishes {
        record_legal_state(
            emitter,
            legal_states,
            label,
            "raise_floor_pump",
            publish,
            &memory_pool_snapshot(&devices),
        );
    }
    let beta_after_raise_floor = basis_of(
        &format!("{label}_f"),
        &memory_pool_snapshot(&devices),
        &current,
        reachable,
    );

    // O
    current = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &current,
            FirstFile {
                content: &overwrite_content(20),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 80,
            },
            instance,
        )
        .expect("HK 抬 F 之后的 O")
    };
    record_legal_state(
        emitter,
        legal_states,
        label,
        "O",
        &current,
        &memory_pool_snapshot(&devices),
    );

    // mount_rollback 到候选集里 txg 最小的根（回退那次发布、暖机）
    // 回退候选集 = 按实例表判仍然有效 ∧ txg ≥ F_生效（D16（发布语义） 已定项 1；HK 已抬过 F，`readable_roots`
    // 不做这层过滤，直接拿它的最小 txg 会选到早已被 F 挡在外面的 genesis 根，「三」S1 的 R7 候选集读法要求这里过滤）。
    let hk_effective_floor = effective_rollback_floor(
        &devices,
        region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let candidates = readable_roots(
        &devices,
        region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let oldest = *candidates
        .iter()
        .filter(|root| root.checkpoint_txg.0 >= hk_effective_floor.0)
        .min_by_key(|root| root.checkpoint_txg.0)
        .expect("HK 回退候选集至少一个根");
    let rollback_target = RollbackTarget {
        instance: oldest.instance,
        checkpoint_txg: oldest.checkpoint_txg,
    };
    let rollback_mounted =
        mount_rollback(parameters, &mut devices, rollback_target, ShadowLedger::On)
            .expect("HK 回退");
    allocator = rollback_mounted.allocator;
    allocator.set_reuse_window(reuse_window);
    instance = rollback_mounted.output.instance;
    record_legal_state(
        emitter,
        legal_states,
        label,
        "rollback_row",
        rollback_mounted
            .output
            .row_publish
            .file_version()
            .expect("回退写行那一版带文件"),
        &memory_pool_snapshot(&devices),
    );
    for warm in &rollback_mounted.output.warm_up_publishes {
        record_legal_state(
            emitter,
            legal_states,
            label,
            "warmup",
            warm.file_version().expect("暖机那一版带文件"),
            &memory_pool_snapshot(&devices),
        );
    }
    current = rollback_mounted
        .current
        .into_file_version()
        .expect("回退后现行版本带文件");
    let beta_after_rollback = basis_of(
        &format!("{label}_r"),
        &memory_pool_snapshot(&devices),
        &current,
        reachable,
    );

    // O
    current = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &current,
            FirstFile {
                content: &overwrite_content(21),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 90,
            },
            instance,
        )
        .expect("HK 回退之后的 O")
    };
    record_legal_state(
        emitter,
        legal_states,
        label,
        "O",
        &current,
        &memory_pool_snapshot(&devices),
    );

    // 在下一次 O 上 c2 崩溃并恢复（W，暖机，L2）
    let c2_pool = crash_before_root_persists(
        parameters,
        &mut devices,
        &mut allocator,
        &current,
        &overwrite_content(30),
        FIXED_WRITE_TIME_SECONDS + 100,
        instance,
        region_devices,
        spacing,
        slots_per_region,
    );
    let mut recovered_devices = devices_from_memory_pool(&c2_pool);
    let recovered = mount_writable(parameters, &mut recovered_devices).expect("HK c2 恢复");
    devices = recovered_devices;
    allocator = recovered.allocator;
    allocator.set_reuse_window(reuse_window);
    instance = recovered.output.instance;
    record_legal_state(
        emitter,
        legal_states,
        label,
        "W_row_after_c2",
        recovered
            .output
            .row_publish
            .file_version()
            .expect("c2 恢复写行那一版带文件"),
        &memory_pool_snapshot(&devices),
    );
    for warm in &recovered.output.warm_up_publishes {
        record_legal_state(
            emitter,
            legal_states,
            label,
            "warmup",
            warm.file_version().expect("暖机那一版带文件"),
            &memory_pool_snapshot(&devices),
        );
    }
    current = recovered
        .current
        .into_file_version()
        .expect("c2 恢复后现行版本带文件");
    let beta_after_crash_recovery = basis_of(
        &format!("{label}_l2"),
        &memory_pool_snapshot(&devices),
        &current,
        reachable,
    );

    // O（L2 之后再来一次，确认恢复之后照常能发布）
    current = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &current,
            FirstFile {
                content: &overwrite_content(31),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 110,
            },
            instance,
        )
        .expect("HK c2 恢复之后的 O")
    };
    record_legal_state(
        emitter,
        legal_states,
        label,
        "O",
        &current,
        &memory_pool_snapshot(&devices),
    );

    HkOutcome {
        beta_first,
        beta_after_reopen,
        beta_after_rollback,
        beta_after_raise_floor,
        beta_after_crash_recovery,
        empty_publish_release_slots,
    }
}

/// S1(c)：重放 `second_transaction_step_four_rollback.rs` 的
/// `rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation`
/// 场景（长度用它的常量 4100 / 2500 字节，本装置自己的内容生成器——数值锚点只挂在「几个单元、几槽」上，
/// 不挂在字节内容上，见「三」I7「一个文件对象恒一个数据单元」）：A → B（实例 1）→ 重开取号 2、写行、暖机
/// → C（实例 2）→ 回退到 A → 普通重开，核 jsn 9、五条记录还在环里、`isolated_slots_per_device == [(0,34),(1,34)]`、
/// `instance == 4`、`abandoned_roots_unreadable == 0`。
fn run_s1c_rollback_isolation_scenario(
    emitter: &mut Emitter,
    parameters: &MakeFilesystemParameters,
) {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(parameters, &mut devices).expect("S1c mkfs");
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let instance1 = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("S1c 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance1).expect("S1c 暖机")
    };
    let mut current = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance1,
            &warm_up_output.last_record_bytes,
        )
        .expect("S1c A")
    };
    // B：4100 字节（这次发布之后 `current` 不再被读，重开直接从 `devices` 的真实字节重建，B 的返回值只留作旁证）。
    let _second_overwrite_result = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &current,
            FirstFile {
                content: &vec![0u8; 4100],
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance1,
        )
        .expect("S1c B")
    };
    // 重开取号 2、写行、暖机。
    let mounted = mount_writable(parameters, &mut devices).expect("S1c 重开取号 2");
    allocator = mounted.allocator;
    let instance2 = mounted.output.instance;
    assert_eq!(
        instance2,
        InstanceGeneration(2),
        "S1c：B 之后重开应当取到实例 2"
    );
    current = mounted
        .current
        .into_file_version()
        .expect("S1c 重开之后现行版本带文件");
    // C：2500 字节，实例 2。
    current = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &current,
            FirstFile {
                content: &vec![0u8; 2500],
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            instance2,
        )
        .expect("S1c C")
    };
    let third_overwrite_counter = current.record.counter;

    // 回退到 A（实例 1，txg 3）。
    let rolled_back = mount_rollback(
        parameters,
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("S1c 回退到 A");
    let rollback_row_publish_counter = rolled_back.output.row_publish.record().counter;
    emitter.emit(&format!("name=s1c_rollback_row_counter c_counter={third_overwrite_counter} d_counter={rollback_row_publish_counter}"));
    assert_eq!(
        rollback_row_publish_counter, 9,
        "S1c：D 的 jsn 应当接在 C 的 8 之后"
    );

    let pool_after_rollback = memory_pool_snapshot(&devices);
    let system_configuration =
        choose_system_configuration(&pool_after_rollback).expect("S1c 系统配置");
    let records = scan_journal(&pool_after_rollback, &system_configuration);
    let expected_jsn: [(u32, u64); 5] = [(1, 4), (2, 5), (2, 8), (3, 9), (3, 10)];
    for (instance, counter) in expected_jsn {
        let present = records.contains_key(&(InstanceGeneration(instance), counter));
        emitter.emit(&format!(
            "name=s1c_record_present instance={instance} counter={counter} present={present}"
        ));
        assert!(
            present,
            "S1c：记录 ({instance}, jsn {counter}) 该原样在环里"
        );
    }

    // 普通重开。
    let remounted = mount_writable(parameters, &mut devices).expect("S1c 回退之后普通重开");
    emitter.emit(&format!(
        "name=s1c_remount instance={} isolated=[(0,{}),(1,{})] abandoned_roots_unreadable={}",
        remounted.output.instance.0,
        remounted
            .output
            .isolated_slots_per_device
            .iter()
            .find(|(identity, _)| *identity == DeviceIdentity(0))
            .map_or(0, |(_, slots)| *slots),
        remounted
            .output
            .isolated_slots_per_device
            .iter()
            .find(|(identity, _)| *identity == DeviceIdentity(1))
            .map_or(0, |(_, slots)| *slots),
        remounted.output.abandoned_roots_unreadable,
    ));
    assert_eq!(
        remounted.output.instance,
        InstanceGeneration(4),
        "S1c：回退之后普通重开应当取到实例 4"
    );
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 54), (DeviceIdentity(1), 54)],
        "S1c：按 D 那一版实例表判被抛弃的根引用的槽，普通重开照样隔离（R-6，第七节 7.2：分配记录树\
         按位置寻址之后每盘 54 槽，不再是原登记的 34）"
    );
    assert_eq!(
        remounted.output.abandoned_roots_unreadable, 0,
        "S1c：这条历史里没有读不出的被抛弃根"
    );
}

// ============================================================================================
// 岔路 3（第 11 行）：HF、G7 的公开入口重组、D_rel。缩小范围：只跑 S = 8、ρ = 1 一个几何取样点
// （登记「五、5.2」HF 的 18 格全扫未做，见交回报告岔路表）。
// ============================================================================================

/// 一次抬 F 处置里，某个口径的推空发布序列（R4：G7 用公开入口重组，F-扣 用真实 `raise_rollback_floor`）。
struct RaiseFloorPumpOutcome {
    /// 每一次推空发布持久之后的 txg，按发生顺序。
    publish_txgs: Vec<u64>,
}

/// G7（登记 R4）：与真实 `raise_rollback_floor`（F-扣）在抬 F 之前逐字节相同的历史上，用公开入口重组
/// 「回收等 F 真生效」这条臂——先推带新 F 的空发布到每块盘都盖到，**之后**才
/// `reclaim_released_up_to(max(新 F, 环里最旧有效根), ReclaimedReuse::Immediately)`，不扣住。
/// 不改真实分配器的回收判定本身，只换调用顺序与「立刻可发」（R4 逐字）。
fn raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
) -> RaiseFloorPumpOutcome {
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let mut covered: Vec<DeviceIdentity> = Vec::new();
    let mut publish_txgs = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publish_txgs.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next = {
            let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
            publish_version(
                &mut pool,
                allocator,
                PublishPlan {
                    txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                    counter: current.record.counter + 1,
                    transaction: 0,
                    highest_transaction_number_before_this_publish: current
                        .highest_transaction_number_in_this_instance,
                    instance: current.root.instance,
                    back_chain: back_chain_of(&current.record_bytes),
                    file: None,
                    new_inode_records: &[],
                    instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                    tree_birth_txg: current.tree_birth_txg(),
                    tree_identifier_watermark: current.root.tree_identifier_watermark,
                    rollback_floor: new_floor,
                },
                Some(&*current),
            )
            .expect("G7 重组抬 F 的推空发布")
        };
        let target = target_for_publish(
            next.root.checkpoint_txg,
            parameters.geometry.root_ring_slots_per_region,
        );
        let device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
        if !covered.contains(&device) {
            covered.push(device);
        }
        publish_txgs.push(next.root.checkpoint_txg.0);
        *current = next;
    }
    // G7：F_生效之后才回收（这一刻每块盘都已经落了带新 F 的根），且不扣住（R4：真实分配器判定不改，只换顺序）。
    let region_devices = parameters.region_devices;
    let oldest_valid_root = readable_roots(
        &*devices,
        &region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root.checkpoint_txg)
    .min();
    allocator.reclaim_released_up_to(
        reclaim_floor(new_floor, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );
    RaiseFloorPumpOutcome { publish_txgs }
}

/// 岔路 3：在同一条历史（β0 → 无崩溃重开 → 8 次非空覆盖写）上，克隆出两条完全相同的抬 F 前状态，
/// 分别按 F-扣（真实 `raise_rollback_floor`）与 G7（上面的公开入口重组）抬到同一个上限，报 D_rel（登记「六」Q3a/Q3b/Q3c）
/// 与 K9（新批次 D_rel_非空 应为 3）。只跑 S = 8、ρ = 1 一个几何取样点，一次处置。
fn run_hf_single_cell(parameters: &MakeFilesystemParameters, emitter: &mut Emitter) {
    // ---- 共用前缀：β0 → 无崩溃重开 → 8 次非空覆盖写（骨架发布 O，ρ = 1） ----
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(parameters, &mut devices).expect("HF mkfs");
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let mut instance = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("HF 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("HF 暖机")
    };
    let first_file_txg = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("HF 第一个事务")
        .root
        .checkpoint_txg
        .0
    };
    // 无崩溃重开：给「现行版本里有重写过的实例表单元」（I4 的前提）。
    let mounted = mount_writable(parameters, &mut devices).expect("HF 无崩溃重开");
    allocator = mounted.allocator;
    instance = mounted.output.instance;
    let mut current = mounted
        .current
        .into_file_version()
        .expect("HF 重开后现行版本带文件");
    // 8 次非空覆盖写（每次释放上一次的落点，release 的 generation = 这次发布的 txg）。
    for step in 0..8u64 {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut pool,
            &mut allocator,
            &current,
            FirstFile {
                content: &overwrite_content(500 + step),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 500 + step,
            },
            instance,
        )
        .expect("HF 前缀的 8 次覆盖写");
    }

    // K9 的前提（登记「七」，任务这一段第 3 条要核的那一句）：抬 F 那一刻环里非空有效根数 ≥ 4。
    // 这段历史里「非空」的来源只有两处（其余都是 mkfs / 暖机 / 写行这类不改变用户可见内容的发布）：
    // first_file 那一次、与 8 次覆盖写各一次（`overwrite_content` 逐次换盐，内容逐次不同）。在抬 F
    // 之前（三条分支各自重建之前）现读一次 `readable_roots`，数这些 txg 里还留在环上、按实例表判仍然
    // 有效的有几条——不是照抄「五、5.2」HF 那句「8 次非空覆盖写」当结论，是真的从环上数一遍。
    let known_non_empty_txgs: std::collections::HashSet<u64> = std::iter::once(first_file_txg)
        .chain((current.root.checkpoint_txg.0 - 7)..=current.root.checkpoint_txg.0)
        .collect();
    let ring_roots_before_raise = readable_roots(
        &devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let non_empty_valid_root_count_before_raise = ring_roots_before_raise
        .iter()
        .filter(|root| known_non_empty_txgs.contains(&root.checkpoint_txg.0))
        .count();
    emitter.emit(&format!(
        "name=k9_precondition non_empty_valid_root_count={non_empty_valid_root_count_before_raise} threshold=4 holds={} valid_root_count={} known_non_empty_txgs_defined={}",
        non_empty_valid_root_count_before_raise >= 4,
        ring_roots_before_raise.len(),
        known_non_empty_txgs.len(),
    ));

    // ---- 三条分支（探上限、F-扣、G7）各从同一份快照重建，抬 F 之前逐字节相同（V13 的自证）----
    let pre_raise_snapshot = memory_pool_snapshot(&devices);
    // 「释放代 = 新 F 的那一批（最年轻的一批）」（K9）：抬 F 之前，D0 上已释放记录里最大的那个 generation。
    let newest_released_generation_before_raise = allocator
        .records()
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && record.is_released)
        .map(|record| record.generation.0)
        .max()
        .expect("HF 前缀应当已经产生至少一次释放");

    // `SparseBlockDevice` 没有 `Clone`，三条分支各自从同一份 `pre_raise_snapshot` 重建（`devices_from_memory_pool`，
    // 与全文件其余地方克隆一条历史用的是同一条路）；重建之后、抬 F 之前先各拍一次快照核 V13，再各自去抬 F。
    let mut probe_devices = devices_from_memory_pool(&pre_raise_snapshot);
    let mut real_raise_devices = devices_from_memory_pool(&pre_raise_snapshot);
    let mut reconstructed_raise_devices = devices_from_memory_pool(&pre_raise_snapshot);
    let prefix_identical_before_raising_the_floor = memory_pool_snapshot(&probe_devices).devices
        == pre_raise_snapshot.devices
        && memory_pool_snapshot(&real_raise_devices).devices == pre_raise_snapshot.devices
        && memory_pool_snapshot(&reconstructed_raise_devices).devices == pre_raise_snapshot.devices;
    emitter.emit(&format!(
        "name=v13_hf_prefix_identical holds={prefix_identical_before_raising_the_floor}"
    ));
    assert!(
        prefix_identical_before_raising_the_floor,
        "V13：G7 与 F-扣 两条历史在抬 F 之前的设备字节应当相同"
    );

    // 探上限：两条口径共用同一个 new_floor（R5：都用 rollback_floor_ceiling 给的上限）。
    let mut probe_allocator = allocator.clone();
    let mut probe_current = current.clone();
    let probe_result = raise_rollback_floor(
        parameters,
        &mut probe_devices,
        &mut probe_allocator,
        &mut probe_current,
        CheckpointTxg(u64::MAX),
        ShadowLedger::On,
    );
    let ceiling = match probe_result {
        Err(MountError::RollbackFloorAboveCeiling { ceiling, .. }) => ceiling,
        Err(other) => panic!("HF 探上限期望 RollbackFloorAboveCeiling，得到别的错误 {other:?}"),
        Ok(_) => panic!("HF 探上限用 u64::MAX 探测却成功了（S4：写装置时读出的分歧，交主 agent）"),
    };

    // F-扣（真实 raise_rollback_floor）
    let mut real_raise_allocator = allocator.clone();
    let mut real_raise_current = current.clone();
    let real_raise_outcome = raise_rollback_floor(
        parameters,
        &mut real_raise_devices,
        &mut real_raise_allocator,
        &mut real_raise_current,
        ceiling,
        ShadowLedger::On,
    )
    .expect("HF F-扣 抬 F 到上限");
    let real_raise_dispensable_txg = real_raise_outcome
        .publishes
        .last()
        .map(|publish| publish.root.checkpoint_txg.0)
        .unwrap_or(current.root.checkpoint_txg.0);

    // G7（公开入口重组）
    let mut reconstructed_raise_allocator = allocator.clone();
    let mut reconstructed_raise_current = current.clone();
    let reconstructed_raise_outcome =
        raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect(
            parameters,
            &mut reconstructed_raise_devices,
            &mut reconstructed_raise_allocator,
            &mut reconstructed_raise_current,
            ceiling,
        );
    let reconstructed_raise_dispensable_txg = reconstructed_raise_outcome
        .publish_txgs
        .last()
        .copied()
        .unwrap_or(current.root.checkpoint_txg.0);

    // D_rel：release_generation = 释放它的那次发布的 txg，dispensable_txg = 该口径最后一次覆盖到的推空发布的 txg
    // （登记「六」Q3a/Q3b）。
    let release_generation = newest_released_generation_before_raise;
    // 前缀 8 次覆盖写各自的 txg：从 warm-up 之后开始连续 8 个 txg（无崩溃重开只有一次写行 + 暖机，不产生分叉）。
    // 重开的写行/暖机与第一个事务都不改变用户可见的文件内容（暖机只重放同一个文件版本），不算「非空」（登记
    // 「非空」口径：树表里 inode/extent 树根指针与前一条有效根不同）；8 次覆盖写每次都改变内容，逐次都算非空。
    let overwrite_txg_start = current.root.checkpoint_txg.0 - 7; // 8 次连续覆盖写的第一个 txg
    let overwrite_txgs: Vec<u64> = (overwrite_txg_start..=current.root.checkpoint_txg.0).collect();
    let count_non_empty_between = |from_generation: u64, dispensable_txg: u64| -> u64 {
        overwrite_txgs
            .iter()
            .filter(|&&txg| txg > from_generation && txg < dispensable_txg)
            .count() as u64
    };
    let count_all_between = |from_generation: u64, dispensable_txg: u64| -> u64 {
        if dispensable_txg > from_generation + 1 {
            dispensable_txg - from_generation - 1
        } else {
            0
        }
    };

    let released_to_dispensable_non_empty_publishes_reconstructed =
        count_non_empty_between(release_generation, reconstructed_raise_dispensable_txg);
    let released_to_dispensable_all_publishes_reconstructed =
        count_all_between(release_generation, reconstructed_raise_dispensable_txg);
    let released_to_dispensable_non_empty_publishes_real =
        count_non_empty_between(release_generation, real_raise_dispensable_txg);
    let released_to_dispensable_all_publishes_real =
        count_all_between(release_generation, real_raise_dispensable_txg);
    let maximum_difference = released_to_dispensable_non_empty_publishes_real
        .abs_diff(released_to_dispensable_non_empty_publishes_reconstructed);

    emitter.emit(&format!(
        "name=q3a_d_rel_g7 release_generation={release_generation} dispensable_txg={reconstructed_raise_dispensable_txg} d_rel_non_empty={released_to_dispensable_non_empty_publishes_reconstructed} d_rel_all={released_to_dispensable_all_publishes_reconstructed} pump_publishes={}",
        reconstructed_raise_outcome.publish_txgs.len()
    ));
    emitter.emit(&format!(
        "name=q3b_d_rel_f_kou release_generation={release_generation} dispensable_txg={real_raise_dispensable_txg} d_rel_non_empty={released_to_dispensable_non_empty_publishes_real} d_rel_all={released_to_dispensable_all_publishes_real} pump_publishes={}",
        real_raise_outcome.publishes.len()
    ));
    emitter.emit(&format!(
        "name=q3c_diff max_diff={maximum_difference} threshold=3 verdict={}",
        if maximum_difference <= 3 {
            "difference_not_large"
        } else {
            "difference_matters"
        }
    ));
    // K9（新，released 记录唯一的一批）：前提「抬 F 那一刻环里非空有效根数 ≥ 4」由上面的
    // `name=k9_precondition` 现算现核（S = 8、ρ = 1 这一格量出 9 ≥ 4），不是照抄预估。
    // 只报数，不因为不等于 3 就作废——F3 的处置交主 agent（见「四」P21(e) 已经预估这一格可能不是 3；
    // 这一格量出 0：release_generation 与 dispensable_txg 之间的 3 次持久发布恰好都是抬 F 自己的
    // 推空发布，本来就不算非空，与 P21(e)「越早释放等得越久」是同一个方向上的现象，不是矛盾）。
    emitter.emit(&format!(
        "name=k9_newest_batch_d_rel_non_empty g7={released_to_dispensable_non_empty_publishes_reconstructed} f_kou={released_to_dispensable_non_empty_publishes_real} expected=3 note=see_p21e_prediction"
    ));

    // Q3d（主 agent 续派第 2 条「Q3d 一并做」；口径与限定见 `q3d_derived` 的文档注释）。
    let (
        accounting_free_index_real_raise,
        allocator_dispensable_index_real_raise,
        accounting_free_index_reconstructed_raise,
        allocator_dispensable_index_reconstructed_raise,
    ) = q3d_derived(
        u64::try_from(real_raise_outcome.publishes.len()).unwrap_or(0),
        u64::try_from(reconstructed_raise_outcome.publish_txgs.len()).unwrap_or(0),
    );
    emitter.emit(&format!(
        "name=q3d_d_acct_d_alloc d_acct_f_kou={accounting_free_index_real_raise} d_alloc_f_kou={allocator_dispensable_index_real_raise} d_acct_g7={accounting_free_index_reconstructed_raise} d_alloc_g7={allocator_dispensable_index_reconstructed_raise} method=derived_from_documented_call_order_not_per_push_measured"
    ));

    // Q3e（X8-A）：另在独立的小池上跑（`run_small_pool_cell`，主 agent 续派第 2 条），不复用这一格的历史——
    // X8-A 要的是 P5 那个 128 槽两段小池几何，与这里 4 GiB 主几何是两个不同的池。
}

/// Q3d（E154 登记「六」Q4 口径，P7/P8）：以第一条带新 F 的持久根为第 0 次，`D_acct` = 第一条记账行把
/// 这批槽算空闲的持久根序号，`D_alloc` = 分配器第一次能发这批槽之前最后那条持久根的序号。**这是从
/// `mount.rs`/这个装置 R4 重组函数的已读代码顺序推出来的，不是逐次持久根现读记账行量出来的**（据实
/// 标注，不假装比实际做到的更精）：
/// - F-扣（真实 `raise_rollback_floor`）：回收并扣住发生在**第一次推空发布之前**（`mount.rs` 812–855
///   行一带，reclaim 调用先于 `PoolWriter`/推空循环），记账行第一次把这批槽算空闲就是第 0 次persistent
///   根本身 ⇒ `D_acct = 0`；`release_reclaim_holds` 在推空循环**全部**成功之后才调，分配器真正能发是
///   最后一次推空之后 ⇒ `D_alloc = pump_publishes − 1`。
/// - G7（这个装置 R4 重组）：`reclaim_released_up_to(..., Immediately)` 在推空循环**全部**结束之后才调，
///   记账与「能发」在同一次调用里一起生效 ⇒ `D_acct = D_alloc = pump_publishes − 1`。
fn q3d_derived(
    pump_publishes_real: u64,
    pump_publishes_reconstructed: u64,
) -> (i64, i64, i64, i64) {
    let accounting_free_index_real_raise = 0i64;
    let allocator_dispensable_index_real_raise =
        i64::try_from(pump_publishes_real).unwrap_or(i64::MAX) - 1;
    let accounting_free_index_reconstructed_raise =
        i64::try_from(pump_publishes_reconstructed).unwrap_or(i64::MAX) - 1;
    let allocator_dispensable_index_reconstructed_raise = accounting_free_index_reconstructed_raise;
    (
        accounting_free_index_real_raise,
        allocator_dispensable_index_real_raise,
        accounting_free_index_reconstructed_raise,
        allocator_dispensable_index_reconstructed_raise,
    )
}

// ============================================================================================
// 岔路 3（第 11 行）续派第 2 条：Q3e（X8-A）。P5 的小池几何（128 槽两段）：设备字节 =
// (`UNIT_AREA_START_SLOT` + 128) × `SLOT_BYTES`，journal 环调小到 mkfs 收得下（登记「五、5.2」HX 逐字）。
// 两个口径各自走公开入口（F-扣：真实 `raise_rollback_floor`；G7：下面这个可失败版本的重组），报「六」
// Q3e 的七样：①③④直接量（`x8a_held_measure`）、①⑤从真实调用的返回值读、⑥⑦另发布/另挂载现测。
// ============================================================================================

/// 小池的单元区槽数（P5「128 槽两段」逐字）；两段 = 128 / `CLUSTER_SEGMENT_SLOTS`(64) = 2。
const SMALL_POOL_UNIT_AREA_SLOTS: u64 = 128;
/// 设备字节：单元区起点（`UNIT_AREA_START_SLOT`）+ 128 槽（登记「五、5.2」HX 逐字）。
const SMALL_POOL_IMAGE_BYTES: u64 =
    (UNIT_AREA_START_SLOT + SMALL_POOL_UNIT_AREA_SLOTS) * SLOT_BYTES;
/// journal 环调小到 mkfs 收得下：`check_geometry` 要求 `journal_ring_bytes ≤ 设备字节 ÷ 4`
/// （`SMALL_POOL_IMAGE_BYTES` 约 786 MiB，上限约 196 MiB），4 MiB 远小于上限，`journal_in_flight_record_limit`
/// 也给得出一个正常的正数（不是 0）。
const SMALL_POOL_JOURNAL_RING_BYTES: u64 = 4 * 1024 * 1024;

fn small_pool_parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E156_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: SMALL_POOL_JOURNAL_RING_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// G7 的 X8A 专用版本：算法与 `raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect`
/// 逐字相同（先推空到每块盘都盖到，之后才 `Immediately` 回收，不扣住），只把 `.expect(...)` 换成 `?`——
/// X8A 要看的正是「推空发布本身分配不到固定点会怎样」，不能让这一格 panic。
fn raise_rollback_floor_by_reconstructing_fallible(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
) -> Result<RaiseFloorPumpOutcome, PublishError> {
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let mut covered: Vec<DeviceIdentity> = Vec::new();
    let mut publish_txgs = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publish_txgs.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next = {
            let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
            publish_version(
                &mut pool,
                allocator,
                PublishPlan {
                    txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                    counter: current.record.counter + 1,
                    transaction: 0,
                    highest_transaction_number_before_this_publish: current
                        .highest_transaction_number_in_this_instance,
                    instance: current.root.instance,
                    back_chain: back_chain_of(&current.record_bytes),
                    file: None,
                    new_inode_records: &[],
                    instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                    tree_birth_txg: current.tree_birth_txg(),
                    tree_identifier_watermark: current.root.tree_identifier_watermark,
                    rollback_floor: new_floor,
                },
                Some(&*current),
            )?
        };
        let target = target_for_publish(
            next.root.checkpoint_txg,
            parameters.geometry.root_ring_slots_per_region,
        );
        let device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
        if !covered.contains(&device) {
            covered.push(device);
        }
        publish_txgs.push(next.root.checkpoint_txg.0);
        *current = next;
    }
    let region_devices = parameters.region_devices;
    let oldest_valid_root = readable_roots(
        &*devices,
        &region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root.checkpoint_txg)
    .min();
    allocator.reclaim_released_up_to(
        reclaim_floor(new_floor, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );
    Ok(RaiseFloorPumpOutcome { publish_txgs })
}

/// 把 P5 的小池填到「装不下下一次覆盖写」为止：mkfs → 取号 → 暖机 → 第一个事务 → 无崩溃重开 → 反复
/// 覆盖写直到 `publish_overwrite` 第一次报错，停在最后一次成功的状态（拒绝时分配器不动，「三」I 系列
/// 同一条纪律：`try_allocate_*` 与发布路径的准入闸都是「一样都没动」）。`maximum_overwrites`：`None` 时
/// 跑到真耗尽（HX，登记 P5 的几何）；`Some(cap)` 时只跑 `cap` 次就停，即便还能继续（HY，登记「五、5.2」
/// 「抬 F 那一刻开放段里还有 ≥ 8 个空槽」那个对照——不追求刚好 8 个，只要足够宽松让两个口径都推得动）。
fn build_small_pool_prefix(
    parameters: &MakeFilesystemParameters,
    maximum_overwrites: Option<u64>,
) -> (
    Vec<(DeviceIdentity, SparseBlockDevice)>,
    PoolAllocator,
    InstanceGeneration,
    TransactionOutput,
    u64,
    String,
) {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(SMALL_POOL_IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(parameters, &mut devices).expect("X8A mkfs");
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let mut instance = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("X8A 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("X8A 暖机")
    };
    {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("X8A 第一个事务");
    }
    // 无崩溃重开：给「现行版本里有重写过的实例表单元」（I4 的前提，与 HF 单格同一处理由）。
    let mounted = mount_writable(parameters, &mut devices).expect("X8A 无崩溃重开");
    allocator = mounted.allocator;
    instance = mounted.output.instance;
    let mut current = mounted
        .current
        .into_file_version()
        .expect("X8A 重开后现行版本带文件");
    let mut overwrite_count = 0u64;
    let stop_reason;
    loop {
        if let Some(cap) = maximum_overwrites {
            if overwrite_count >= cap {
                stop_reason = format!("stopped_at_cap_{cap}_by_request");
                break;
            }
        }
        let attempt = {
            let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut pool,
                &mut allocator,
                &current,
                FirstFile {
                    content: &overwrite_content(700 + overwrite_count),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 700 + overwrite_count,
                },
                instance,
            )
        };
        match attempt {
            Ok(next) => {
                current = next;
                overwrite_count += 1;
            }
            Err(error) => {
                stop_reason = format!("{error:?}");
                break;
            }
        }
        if overwrite_count > 40 {
            stop_reason = "safety_cap_40_reached_without_exhaustion".to_string();
            break;
        }
    }
    (
        devices,
        allocator,
        instance,
        current,
        overwrite_count,
        stop_reason,
    )
}

/// 在**不受任何抬 F 尝试影响**的原始前缀上，接着试 4 次覆盖写，各自成不成（P5「连发 4 次」逐字）。
/// `content_salt_base` 避开与前缀历史、与另一个口径的探测用过的盐重合。
fn probe_four_more_overwrites(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    instance: InstanceGeneration,
    content_salt_base: u64,
) -> Vec<String> {
    let mut probe_devices = devices_from_memory_pool(&memory_pool_snapshot(devices));
    let mut probe_allocator = allocator.clone();
    let mut probe_current = current.clone();
    (0..4u64)
        .map(|attempt| {
            let mut pool = PoolWriter::new(parameters, probe_devices.as_mut_slice());
            let result = publish_overwrite(
                &mut pool,
                &mut probe_allocator,
                &probe_current,
                FirstFile {
                    content: &overwrite_content(content_salt_base + attempt),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + content_salt_base + attempt,
                },
                instance,
            );
            match result {
                Ok(next) => {
                    probe_current = next;
                    "ok".to_string()
                }
                Err(error) => format!("failed:{error:?}"),
            }
        })
        .collect()
}

/// 岔路 3：Q3e（X8-A / X8-B 对照）。`pool_label`：`"hx"`（P5 的几何，`maximum_overwrites=None` 填到耗尽）
/// 或 `"hy"`（同一个小池几何，`maximum_overwrites=Some(cap)` 提前停手，抬 F 那一刻还留着空间，登记「五、5.2」
/// HY 的定义——两个口径在 HY 上都不该卡，当 HX 的阳性对照）。两个口径各自抬 F 到同一个上限，报「六」
/// Q3e 的七样，每一行都带 `pool={pool_label}`。
fn run_small_pool_cell(emitter: &mut Emitter, pool_label: &str, maximum_overwrites: Option<u64>) {
    let parameters = small_pool_parameters();
    let (devices, allocator, instance, current, overwrite_count, stop_reason) =
        build_small_pool_prefix(&parameters, maximum_overwrites);
    let (allocated, free, deferred) = accounting_row_slots(&allocator, DeviceIdentity(0));
    emitter.emit(&format!(
        "name=x8a_prefix pool={pool_label} overwrite_count={overwrite_count} stop_reason={stop_reason} allocated={allocated} free={free} deferred={deferred}"
    ));
    // R-7（第七节 7.2）：HY 的定义改成 e ≥ max(8, f)——e 是这一刻开放段里的空槽数，f 是一次推空发布
    // （只改一片叶时）自己的固定点槽数，两者都现算，不再是「≥ 8」这句原文的字面。HX 上同样报这一行，
    // 只作观测（HX 的定义是填到耗尽，这一条件在 HX 上多半不成立，不构成对照失败）。
    let open_segment_free_slots = e156_open_segment_free_slots(&allocator, DeviceIdentity(0));
    let existing_leaves_at_raise = e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
    let fixed_point_slots = e156_empty_publish_fixed_point_slots(&existing_leaves_at_raise);
    let hy_threshold = fixed_point_slots.max(8);
    emitter.emit(&format!(
        "name=r7_hy_condition pool={pool_label} open_segment_free_slots={open_segment_free_slots} fixed_point_slots={fixed_point_slots} threshold={hy_threshold} holds={}",
        open_segment_free_slots >= hy_threshold
    ));

    // 探上限：与 HF 单格同一个办法。
    let mut probe_devices = devices_from_memory_pool(&memory_pool_snapshot(&devices));
    let mut probe_allocator = allocator.clone();
    let mut probe_current = current.clone();
    let probe_result = raise_rollback_floor(
        &parameters,
        &mut probe_devices,
        &mut probe_allocator,
        &mut probe_current,
        CheckpointTxg(u64::MAX),
        ShadowLedger::On,
    );
    let ceiling = match probe_result {
        Err(MountError::RollbackFloorAboveCeiling { ceiling, .. }) => ceiling,
        Err(other) => {
            emitter.emit(&format!(
                "name=q3e_x8a pool={pool_label} status=not_done reason=probe_ceiling_unexpected_error error={other:?}"
            ));
            return;
        }
        Ok(_) => {
            emitter.emit(&format!(
                "name=q3e_x8a pool={pool_label} status=not_done reason=probe_ceiling_succeeded_with_u64_max"
            ));
            return;
        }
    };

    // 续派第 2 条②③④：单独在一份克隆上重放「回收并扣住」这一步（与 `raise_rollback_floor` 内部的算式
    // 逐字相同：`reclaim_floor(new_floor, oldest_valid_root)`，`oldest_valid_root` 不过滤被抛弃根——
    // 这段历史没有回退，过滤恒真，R4 的射程），拿公开的 `DeviceFreeMap` 读数直接量，不用猜。
    let mut held_measure_allocator = allocator.clone();
    let region_devices = parameters.region_devices;
    let oldest_valid_root_for_hold = readable_roots(
        &devices,
        &region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root.checkpoint_txg)
    .min();
    let reclaimed_and_held = held_measure_allocator.reclaim_released_up_to(
        reclaim_floor(ceiling, oldest_valid_root_for_hold),
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );
    let held_span_sum: u64 = reclaimed_and_held
        .iter()
        .map(|placement| placement.span)
        .sum();
    let (allocated_after_hold, free_after_hold, deferred_after_hold) =
        accounting_row_slots(&held_measure_allocator, DeviceIdentity(0));
    let lowest_empty_segment_after_hold = held_measure_allocator
        .devices
        .iter()
        .find(|map| map.device == DeviceIdentity(0))
        .and_then(DeviceFreeMap::lowest_empty_segment)
        .map_or("none".to_string(), |slot| slot.0.to_string());
    emitter.emit(&format!(
        "name=x8a_held_measure pool={pool_label} ceiling={} reclaimed_and_held_records={} reclaimed_and_held_span={held_span_sum} allocated_after_hold={allocated_after_hold} free_after_hold={free_after_hold} deferred_after_hold={deferred_after_hold} lowest_empty_segment_after_hold={lowest_empty_segment_after_hold}",
        ceiling.0, reclaimed_and_held.len(),
    ));

    // F-扣（真实 raise_rollback_floor）
    let mut real_devices = devices_from_memory_pool(&memory_pool_snapshot(&devices));
    let mut real_allocator = allocator.clone();
    let mut real_current = current.clone();
    let real_result = raise_rollback_floor(
        &parameters,
        &mut real_devices,
        &mut real_allocator,
        &mut real_current,
        ceiling,
        ShadowLedger::On,
    );
    let (real_first_push_result, real_holds_released) = match &real_result {
        Ok(_) => ("succeeded".to_string(), true),
        Err(MountError::Publish(publish_error)) => {
            (format!("failed_placement_refused:{publish_error:?}"), false)
        }
        Err(other) => (format!("failed_other:{other:?}"), false),
    };
    emitter.emit(&format!(
        "name=q3e_f_kou pool={pool_label} first_push_result={real_first_push_result} release_reclaim_holds_reached={real_holds_released}"
    ));

    // 失败之后接着在**同一份未受影响的前缀**（`devices`/`allocator`/`current`，抬 F 失败按纪律不改
    // 分配器、不落盘）上试着再发 4 次覆盖写，看是不是「此后每次发布都分配不到固定点」（P5 逐字）。
    let real_stuck = real_result.is_err();
    let real_four_more = if real_stuck {
        probe_four_more_overwrites(&parameters, &devices, &allocator, &current, instance, 8000)
    } else {
        vec!["not_applicable_raise_succeeded".to_string(); 4]
    };
    emitter.emit(&format!(
        "name=q3e_f_kou_four_more pool={pool_label} results={}",
        real_four_more.join(",")
    ));
    let real_remount = {
        let mut remount_devices = devices_from_memory_pool(&memory_pool_snapshot(&devices));
        match mount_writable(&parameters, &mut remount_devices) {
            Ok(_) => "remounted_ok".to_string(),
            Err(error) => format!("remount_failed:{error:?}"),
        }
    };
    emitter.emit(&format!(
        "name=q3e_f_kou_remount pool={pool_label} result={real_remount}"
    ));

    // G7（公开入口重组，可失败版本）
    let mut reconstructed_raise_devices = devices_from_memory_pool(&memory_pool_snapshot(&devices));
    let mut reconstructed_raise_allocator = allocator.clone();
    let mut reconstructed_raise_current = current.clone();
    let reconstructed_raise_result = raise_rollback_floor_by_reconstructing_fallible(
        &parameters,
        &mut reconstructed_raise_devices,
        &mut reconstructed_raise_allocator,
        &mut reconstructed_raise_current,
        ceiling,
    );
    let reconstructed_raise_first_push_result = match &reconstructed_raise_result {
        Ok(_) => "succeeded".to_string(),
        Err(publish_error) => format!("failed_placement_refused:{publish_error:?}"),
    };
    // G7 的重组函数里回收永远是 `Immediately`、从不 `HeldUntilFloorTakesEffect`：这个口径没有
    // 「扣住」这个状态，`release_reclaim_holds` 这个函数它根本不调——不是「有没有走到」，是「不适用」。
    emitter.emit(&format!(
        "name=q3e_g7 pool={pool_label} first_push_result={reconstructed_raise_first_push_result} release_reclaim_holds_reached=not_applicable_g7_never_holds"
    ));
    let reconstructed_raise_stuck = reconstructed_raise_result.is_err();
    let reconstructed_raise_four_more = if reconstructed_raise_stuck {
        probe_four_more_overwrites(&parameters, &devices, &allocator, &current, instance, 9000)
    } else {
        vec!["not_applicable_raise_succeeded".to_string(); 4]
    };
    emitter.emit(&format!(
        "name=q3e_g7_four_more pool={pool_label} results={}",
        reconstructed_raise_four_more.join(",")
    ));
    let reconstructed_raise_remount = {
        let mut remount_devices = devices_from_memory_pool(&memory_pool_snapshot(&devices));
        match mount_writable(&parameters, &mut remount_devices) {
            Ok(_) => "remounted_ok".to_string(),
            Err(error) => format!("remount_failed:{error:?}"),
        }
    };
    emitter.emit(&format!(
        "name=q3e_g7_remount pool={pool_label} result={reconstructed_raise_remount}"
    ));

    emitter.emit(&format!(
        "name=q3e_x8a pool={pool_label} status=done f_kou_stuck={real_stuck} reconstructed_raise_stuck={reconstructed_raise_stuck} both_stuck_at_same_step={} note=see_x8a_held_measure_and_f_kou_g7_lines",
        real_stuck == reconstructed_raise_stuck,
    ));
}

/// R-7（第一节 R1、第七节 7.2）：HY 原来靠「只跑 3 次覆盖写」留出空间，`e ≥ 8` 从没被装置核过；
/// 这一次改成 `e ≥ max(8, f)` 现核——3 次不满足就减少覆盖写次数直到满足（登记「五」5.1 逐字）。
/// 只建前缀、量 `e`/`f`，不跑 `run_small_pool_cell` 的其余部分（避免为找 cap 打印一堆用不上的行）。
fn e156_find_hy_cap_satisfying_open_segment_condition(parameters: &MakeFilesystemParameters) -> u64 {
    for cap in (0..=3u64).rev() {
        let (_devices, allocator, _instance, _current, _overwrite_count, _stop_reason) =
            build_small_pool_prefix(parameters, Some(cap));
        let open_segment_free_slots = e156_open_segment_free_slots(&allocator, DeviceIdentity(0));
        let existing_leaves = e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
        let fixed_point_slots = e156_empty_publish_fixed_point_slots(&existing_leaves);
        if open_segment_free_slots >= fixed_point_slots.max(8) {
            return cap;
        }
    }
    0
}

// ============================================================================================
// 岔路 1（第 9 行）：Hh(k)，甲-T1（真实基线）与 G12（旁路评估）多扣的差随洞数怎么长。缩小范围：
// ρ = 1、回收时点固定「实」、洞位置固定「后」；S 这一维跑 S = 8（mkfs 默认）与 S = 4（下界，方向相反，
// 「八」第六类）两个取样点，各自 k ∈ {0, 1, 2}（登记「五、5.2」Hh(k, 位置, S, ρ) 的 S = 16、ρ = 1/4、
// 回收时点「每」、洞位置「前」、k = 4 这几维未扫，见交回报告岔路表）。
// ============================================================================================

/// D0 上「仍分配」记录的 (槽 → 分配代)：只给岔路 1 的分配代账本用（G12 需要一个真实记录里没有的分配代字段，
/// 登记「三」N2；装置自己维护一份影子表，见登记 M6）。
fn snapshot_allocated_generations(allocator: &PoolAllocator) -> HashMap<u64, u64> {
    allocator
        .records()
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && !record.is_released)
        .map(|record| (record.slot.0, record.generation.0))
        .collect()
}

/// 把这次发布新产生的释放记进分配代账本：`before` 是这次发布之前 D0 全部「仍分配」记录的 (槽 → 分配代)，
/// `allocator` 是这次发布之后的状态。真实记录的 `generation` 字段在释放那一刻被改写成释放代（登记「三」I1），
/// 分配代因此只能从 `before` 里找；账本里查不到的（mkfs 格式期分配、从没被本函数追踪过）按 0（genesis）记，
/// 这是保守默认，不是量出来的数（写进交回报告）。
fn record_release_generations(
    ledger: &mut HashMap<u64, u64>,
    before: &HashMap<u64, u64>,
    allocator: &PoolAllocator,
) {
    for record in allocator
        .records()
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && record.is_released)
    {
        if let Some(&allocation_generation) = before.get(&record.slot.0) {
            ledger.insert(record.slot.0, allocation_generation);
        }
    }
}

/// 一格 Hh(k) 的读数：Q1a（甲-T1 多扣）、Q1b（G12 多扣）、Δ、h_缺。只在最后一次无崩溃重开（「实」读法的
/// 回收点）之后测一次，ρ = 1，位置固定「后」（洞造在环转过一圈之后）；`slots_per_region` 这一维按「五、5.6」
/// 第六类取 S = 8（mkfs 默认）与 S = 4（下界，方向相反）两个取样点，见交回报告岔路表：ρ、回收时点「每」、
/// 洞位置「前」、S = 16 这几维这一段仍未扫。
struct HhCell {
    slots_per_region: u64,
    holes: u64,
    /// 主 agent 续派第 1 条：步数对齐的对照组用这个字段标「这一格是 k=0，但工作负载发布次数补到与
    /// holes=`matched_to_holes` 那格相同」；真实的 Hh(k) 格这个字段是 0（不是对照）。
    matched_to_holes: u64,
    txg: u64,
    q1a_over_withheld: u64,
    q1b_over_withheld: u64,
    delta: u64,
    missing_txg_count: u64,
}

/// 一次「造洞」（c2 崩溃 + 恢复 + 2 次间隔发布）固定消耗的 txg 数——2026-09-24 现测（`research/results/
/// e156-alloc-basis-counts-2026-09-24-stage3.out` 的 `q1_hh` 行）：S = 8 与 S = 4 上 holes=1 相对
/// holes=0（环转门槛那一刻）都恰好多 6 个 txg，holes=2 恰好多 12 个（与 S 无关——「造洞」本身不受环深
/// 影响，环深只决定「环转过一圈」门槛与洞的计数）。步数对齐对照（主 agent 续派第 1 条）靠这个常数换算。
const HOLE_TXG_COST: u64 = 6;

/// 岔路 1 的一格：造 `holes` 个洞（C380 逐字：c2 崩溃 + 一次可写挂载恢复），每个洞之间隔 2 次工作负载发布，
/// 最后再无崩溃重开一次（甲-T1「实」读法的回收点），在那一刻测 Q1a/Q1b/Δ/h_缺。
///
/// `matched_to_holes`（主 agent 续派第 1 条，步数对齐对照）：非零时这一格**不造任何洞**（`holes` 必须为
/// 0），但把「环转过一圈」的门槛多加 `matched_to_holes * HOLE_TXG_COST` 步工作负载，让总发布步数与
/// `holes = matched_to_holes` 那一格对齐（两格都只在最后无崩溃重开一次）——用来把「Δ 随洞数长」与
/// 「Δ 随步数长」这两件事分开。真实的 Hh(k) 格传 0。
fn run_hh_cell(
    parameters: &MakeFilesystemParameters,
    region_devices: &[DeviceIdentity; 3],
    spacing: u32,
    slots_per_region: RootRingSlotsPerRegion,
    holes: u64,
    matched_to_holes: u64,
    emitter: &mut Emitter,
) -> HhCell {
    assert!(
        holes == 0 || matched_to_holes == 0,
        "步数对齐对照只对 holes=0 这一格定义，真实的 Hh(k) 格不许再对齐"
    );
    let extra_ring_fill_steps = matched_to_holes * HOLE_TXG_COST;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(parameters, &mut devices).expect("Hh mkfs");
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let mut instance = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("Hh 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("Hh 暖机")
    };
    let mut current: TransactionOutput = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("Hh 第一个事务")
    };

    let mut ledger: HashMap<u64, u64> = HashMap::new();
    let slots_per_region_count = slots_per_region.count();
    let ring_length = 3 * slots_per_region_count;

    // 工作负载直到环转过一圈（位置 = 后：最新持久根 txg ≥ 3S + 3 之后才许造洞，登记「五」5.2 Hh 字面）；
    // `extra_ring_fill_steps` > 0 时（步数对齐对照）多跑这些步，不影响门槛本身的定义，只是把这一格的
    // 工作负载段拉长到与某个 holes=k 格总步数相同。
    let mut step = 0u64;
    while current.root.checkpoint_txg.0 < ring_length + 3 + extra_ring_fill_steps {
        let before_ring_fill = snapshot_allocated_generations(&allocator);
        current = {
            let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut pool,
                &mut allocator,
                &current,
                FirstFile {
                    content: &overwrite_content(1000 + step),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1000 + step,
                },
                instance,
            )
            .expect("Hh 环转前的工作负载")
        };
        record_release_generations(&mut ledger, &before_ring_fill, &allocator);
        step += 1;
    }

    // 造 `holes` 个洞：c2 崩溃 + 一次可写挂载恢复，每个洞之间隔 2 次工作负载发布（读法写死「洞」：C380 逐字）。
    let mut holes_made = 0u64;
    while holes_made < holes {
        let before_hole = snapshot_allocated_generations(&allocator);
        let c2_pool = crash_before_root_persists(
            parameters,
            &mut devices,
            &mut allocator,
            &current,
            &overwrite_content(2000 + holes_made),
            FIXED_WRITE_TIME_SECONDS + 2000 + holes_made,
            instance,
            region_devices,
            spacing,
            slots_per_region,
        );
        record_release_generations(&mut ledger, &before_hole, &allocator);
        let mut recovered_devices = devices_from_memory_pool(&c2_pool);
        let recovered = mount_writable(parameters, &mut recovered_devices).expect("Hh c2 恢复");
        devices = recovered_devices;
        allocator = recovered.allocator;
        instance = recovered.output.instance;
        current = recovered
            .current
            .into_file_version()
            .expect("Hh c2 恢复后现行版本带文件");
        holes_made += 1;
        for gap in 0..2u64 {
            let before_gap = snapshot_allocated_generations(&allocator);
            current = {
                let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
                publish_overwrite(
                    &mut pool,
                    &mut allocator,
                    &current,
                    FirstFile {
                        content: &overwrite_content(3000 + holes_made * 10 + gap),
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3000 + holes_made * 10 + gap,
                    },
                    instance,
                )
                .expect("Hh 两个洞之间的工作负载")
            };
            record_release_generations(&mut ledger, &before_gap, &allocator);
        }
    }

    // 最后无崩溃重开一次：甲-T1「实」读法的回收点。
    let mounted = mount_writable(parameters, &mut devices).expect("Hh 最终无崩溃重开");
    allocator = mounted.allocator;
    current = mounted
        .current
        .into_file_version()
        .expect("Hh 重开后现行版本带文件");

    // Q1a（甲-T1 多扣）：占着（item1）− 这一刻走读引用 == 已释放未回收（item5，健康状态下）。
    let referenced = referenced_slots(&current);
    let (allocated, _free, deferred) = accounting_row_slots(&allocator, DeviceIdentity(0));
    let q1a_over_withheld = allocated.saturating_sub(referenced);

    // Q1b（G12 多扣）：遍历 D0 上「已释放」的记录，按 G12 的区间谓词判断这一刻还扣不扣住；
    // Hh 没有回退、没有抬 F ⇒「被抛弃 ∨ txg ≥ F_生效」恒真（F_生效 恒 0），谓词退化成「环里有没有一条根的
    // txg 落进 [分配代, 释放代)」。
    let ring_roots = readable_roots(
        &devices,
        region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let mut q1b_over_withheld = 0u64;
    let mut unblocked_span_sum = 0u64;
    let mut unblocked_record_count = 0u64;
    let mut blocked_record_count = 0u64;
    let mut total_released_record_count = 0u64;
    let mut ledger_miss_span_sum = 0u64;
    let mut stale_reclaimed_record_count = 0u64;
    let mut stale_reclaimed_span_sum = 0u64;
    let oldest_ring_txg = ring_roots
        .iter()
        .map(|root| root.checkpoint_txg.0)
        .min()
        .unwrap_or(0);
    let newest_ring_txg = ring_roots
        .iter()
        .map(|root| root.checkpoint_txg.0)
        .max()
        .unwrap_or(0);
    // 派发第 1 条查出的来源：`PoolAllocator::reclaim_released_up_to` 回收之后**不删记录**（把回收过的
    // (device, slot) 记进它私有的 `reclaimed` 集合，位图与 `deferred_slots` 计数器都已经改成「空闲」），
    // 记录本身要等下一次 `record()` 落在同一个起点槽上才被改写（I8 字面）。这个装置的诊断循环原来直接
    // 按 `is_released` 过全部记录，把「已经被真实分配器回收、只是记录条目还没被覆盖」的槽也算成「还扣着」，
    // 于是 Q1b（以及这条 delta 诊断）比 `deferred_slots()`（真实分配器自己的计数器）多算了这些槽。
    // `reclaimed` 集合不是 `pub`，这里改用公开的 `DeviceFreeMap::is_free` 现查：已经被回收又没被再分配的槽
    // 此刻是空闲的（没有扣住、没有隔离），据此把它们从「还扣着」的集合里去掉——不改变量的定义，只补上
    // 诊断代码自己漏掉的这道过滤。
    let device_free_map_for_device_zero = allocator
        .devices
        .iter()
        .find(|map| map.device == DeviceIdentity(0))
        .expect("D0 在池里");
    for record in allocator
        .records()
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && record.is_released)
    {
        if device_free_map_for_device_zero.is_free(record.slot) {
            // 已经被真实分配器回收（位图已清、`deferred_slots` 已扣掉），只是记录条目还没被下一次
            // `record()` 覆盖：这个槽此刻既不占着甲-T1 的账、也不占着 G12 的账，两条臂都不该数它。
            stale_reclaimed_record_count += 1;
            stale_reclaimed_span_sum += u64::from(record.span_slots);
            continue;
        }
        total_released_record_count += 1;
        let release_generation = record.generation.0;
        let ledger_hit = ledger.contains_key(&record.slot.0);
        let allocation_generation = ledger.get(&record.slot.0).copied().unwrap_or(0);
        let blocked = ring_roots.iter().any(|root| {
            let txg = root.checkpoint_txg.0;
            txg >= allocation_generation && txg < release_generation
        });
        if blocked {
            q1b_over_withheld += u64::from(record.span_slots);
            blocked_record_count += 1;
        } else {
            unblocked_span_sum += u64::from(record.span_slots);
            unblocked_record_count += 1;
            if !ledger_hit {
                ledger_miss_span_sum += u64::from(record.span_slots);
            }
            // 任务这一段第 1 条：逐槽列出 G12 判「不再扣住」、甲-T1 仍算在第 1 项里的那些槽
            // （只在这一格逐记录报，不进 Q1a/Q1b 的判据本身，只给主 agent 核 delta 的来源用）。
            emitter.emit(&format!(
                "name=q1_delta_record s={slots_per_region_count} holes={holes} matched_to_holes={matched_to_holes} slot={} span={} allocation_generation={allocation_generation} ledger_hit={ledger_hit} release_generation={release_generation} oldest_ring_txg={oldest_ring_txg} newest_ring_txg={newest_ring_txg}",
                record.slot.0, record.span_slots,
            ));
        }
    }
    // 派发第 1 条的溯源诊断：q1a_over_withheld（deferred 计数）与「records() 里 is_released 的记录」
    // 这两条独立路径是否一致；ledger_miss_span_sum 单独拆出「影子分配代账本找不到分配代」这一种来源
    // （不进任何判据，只给「装置的错 / 影子账的错 / 口径没对齐」这个分类用）。
    emitter.emit(&format!(
        "name=q1_delta_debug s={slots_per_region_count} holes={holes} matched_to_holes={matched_to_holes} deferred={deferred} q1b_blocked_span={q1b_over_withheld} unblocked_span={unblocked_span_sum} blocked_record_count={blocked_record_count} unblocked_record_count={unblocked_record_count} total_released_record_count={total_released_record_count} ledger_miss_span_sum={ledger_miss_span_sum} stale_reclaimed_record_count={stale_reclaimed_record_count} stale_reclaimed_span_sum={stale_reclaimed_span_sum} deferred_minus_q1b={} unblocked_minus_deferred_minus_q1b={}",
        deferred.saturating_sub(q1b_over_withheld),
        unblocked_span_sum.cast_signed() - (deferred.cast_signed() - q1b_over_withheld.cast_signed())
    ));

    // h_缺：[环里最旧有效根.txg, 最新持久根.txg] 里没有自证合法根的 txg 个数（读法写死「洞」逐字）。
    let root_txgs: std::collections::HashSet<u64> = ring_roots
        .iter()
        .map(|root| root.checkpoint_txg.0)
        .collect();
    let oldest_txg = ring_roots
        .iter()
        .map(|root| root.checkpoint_txg.0)
        .min()
        .unwrap_or(0);
    let newest_txg = current.root.checkpoint_txg.0;
    let missing_txg_count = if newest_txg >= oldest_txg {
        (oldest_txg..=newest_txg)
            .filter(|txg| !root_txgs.contains(txg))
            .count() as u64
    } else {
        0
    };

    let delta = q1a_over_withheld.saturating_sub(q1b_over_withheld);
    emitter.emit(&format!(
        "name=q1_hh s={slots_per_region_count} holes={holes} matched_to_holes={matched_to_holes} txg={newest_txg} allocated={allocated} deferred={deferred} referenced={referenced} q1a_t1_over_withheld={q1a_over_withheld} q1b_g12_over_withheld={q1b_over_withheld} delta={delta} h_missing={missing_txg_count}"
    ));
    HhCell {
        slots_per_region: slots_per_region_count,
        holes,
        matched_to_holes,
        txg: newest_txg,
        q1a_over_withheld,
        q1b_over_withheld,
        delta,
        missing_txg_count,
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "跑前登记「五、5.7」第一段的全部量集中在一个可复跑的二进制里，拆开反而难对拍"
)]
fn main() {
    let mut emitter = Emitter { emitted: 0 };
    let parameters = parameters();
    let region_devices = parameters.region_devices;
    let spacing = parameters.geometry.fixed_structure_slot_spacing;
    let slots_per_region = parameters.geometry.root_ring_slots_per_region;
    // 装置的本地 S 与这个池真用的 S 回比（入库装置第 ① 条）。
    assert_eq!(
        E156_SLOTS_PER_REGION,
        slots_per_region.count(),
        "装置写死的每区槽数 S 与这个池写进系统配置的那个不等"
    );
    // A1：单元区起点与实现侧同名常量回比（入库装置第 ① 条）。
    assert_eq!(
        E156_UNIT_AREA_START_SLOT, UNIT_AREA_START_SLOT,
        "A1 的单元区起点常量与实现侧不等"
    );
    let unit_area_slot_count = IMAGE_BYTES / SLOT_BYTES - E156_UNIT_AREA_START_SLOT;
    emitter.emit(&format!(
        "name=anchor_a1 unit_area_slots={unit_area_slot_count}"
    ));
    assert_eq!(
        unit_area_slot_count, 211968,
        "A1：4 GiB 设备的单元区槽数应为 211968"
    );
    // A7：分配记录节点容量。
    let allocation_record_node_capacity =
        (16384 - E156_ALLOCATION_RECORD_NODE_HEADER_BYTES) / E156_ALLOCATION_RECORD_BYTES;
    emitter.emit(&format!(
        "name=anchor_a7 capacity={allocation_record_node_capacity}"
    ));
    assert_eq!(
        allocation_record_node_capacity, 812,
        "A7：分配记录节点容量应为 812"
    );
    // A-D8（第七节 7.1）：本地叶宽 / 扇出与实装的 `singlefs_format` 同名常量回比，本地「根层级规则」
    // 与实装的 `AllocationRecordTreeGeometry::of_allocator` 回比——只观测，不 assert：这两条出自被测
    // 条款本身，对不上走 F21，不作废、不停机（第十节 F21、第七节 7.1 表头）。
    let leaf_slots_match_format_crate = E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
        == singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS;
    let internal_fanout_match_format_crate = E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
        == singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
    let my_root_level = e156_root_level_for_symmetric_devices(unit_area_slot_count, 2);
    emitter.emit(&format!(
        "name=anchor_a_d8 local_leaf_slots={E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS} local_internal_fanout={E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT} local_leaf_slots_matches_format_crate={leaf_slots_match_format_crate} local_internal_fanout_matches_format_crate={internal_fanout_match_format_crate} my_root_level={my_root_level}"
    ));

    // ===== mkfs + 取号 + 暖机 + 第一个事务：K1、S1(a)(g)、H0/HR/HK 共用的起点 =====
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");

    // S1(g)：mkfs 之后 txg 0 的根在环里。
    let genesis_roots = readable_roots(
        &devices,
        &region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let s1g_holds = genesis_roots.iter().any(|root| root.checkpoint_txg.0 == 0);
    emitter.emit(&format!("name=s1g_mkfs_root_in_ring holds={s1g_holds}"));
    assert!(s1g_holds, "S1(g)：mkfs 之后 txg 0 的根应当在环里");

    // A2：根环 3S 槽、txg 落的槽与 target_for_publish 的映射自洽（用真实几何函数核，不是拿常量比常量）。
    let root_ring_slot_count = ROOT_RING_REGIONS * E156_SLOTS_PER_REGION;
    let mut root_ring_mapping_mismatches = 0u64;
    for probe_txg in 0..(root_ring_slot_count * 2) {
        let target = target_for_publish(CheckpointTxg(probe_txg), slots_per_region);
        let expect_region = probe_txg % ROOT_RING_REGIONS;
        let expect_slot = (probe_txg / ROOT_RING_REGIONS) % E156_SLOTS_PER_REGION;
        if target.region != expect_region || target.slot != expect_slot {
            root_ring_mapping_mismatches += 1;
        }
    }
    emitter.emit(&format!("name=anchor_a2 ring_slots={root_ring_slot_count} mismatches={root_ring_mapping_mismatches}"));
    assert_eq!(
        root_ring_mapping_mismatches, 0,
        "A2：target_for_publish 与 u mod 3S 的映射应当逐点一致"
    );

    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    // A-D8（续）：实装的根层级只由每块盘的槽数决定，与记录内容无关，mkfs 之后就能读——与上面本地
    // 算出的 `my_root_level` 回比（只观测，F21）。
    let real_root_level =
        singlefs_core::allocation_record_tree::AllocationRecordTreeGeometry::of_allocator(
            &allocator,
        )
        .root_level();
    emitter.emit(&format!(
        "name=anchor_a_d8_root_level my_root_level={my_root_level} real_root_level={real_root_level} matches={}",
        my_root_level == real_root_level
    ));
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("取号")
    };
    assert_eq!(instance, InstanceGeneration(1));
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("暖机")
    };
    let mut current: TransactionOutput = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("第一个事务")
    };

    // K1：txg 3 之后 D0 的记账第 1/2/5 项，与登记里钉的绝对值比对（S1(a)）。K1-1（第七节 7.2）：
    // 分配记录树按位置寻址之后第 1 项为 17（不再是原登记的 13：多出的 4 槽是这五个树节点里比原来
    // 单节点多出的那四个），第 5 项仍是 1。
    let (allocated0, free0, deferred0) = accounting_row_slots(&allocator, DeviceIdentity(0));
    let k1_1_matches_registered_item1_of_17_and_item5_of_1 = allocated0 == 17 && deferred0 == 1;
    emitter.emit(&format!(
        "name=k1_after_first_transaction txg={} allocated_slots={allocated0} free_slots={free0} deferred_slots={deferred0} registered_item1_slots=17 registered_item5_slots=1 matches_registered={k1_1_matches_registered_item1_of_17_and_item5_of_1}",
        current.root.checkpoint_txg.0,
    ));
    assert!(
        k1_1_matches_registered_item1_of_17_and_item5_of_1,
        "S1(a)：K1-1 应当逐字匹配登记（第七节 7.2，17/1）"
    );
    let mut placements: Vec<(u64, u64)> = current
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.identity.span_slots()))
        .collect();
    placements.sort_unstable();
    let placements_text: Vec<String> = placements
        .iter()
        .map(|(slot, span)| format!("{slot}:{span}"))
        .collect();
    emitter.emit(&format!(
        "name=k1_placements txg={} units={}",
        current.root.checkpoint_txg.0,
        placements_text.join(",")
    ));

    // S1(f)：第一个事务之后的分配记录条数（D0 + D1 合计）。
    let previous_record_count = allocator.records().len();
    emitter.emit(&format!(
        "name=s1f_record_count txg={} count={previous_record_count}",
        current.root.checkpoint_txg.0
    ));
    assert_eq!(
        previous_record_count, E156_FIRST_TRANSACTION_EXPECTED_RECORD_COUNT,
        "S1(f)：第一个事务之后的分配记录条数应为 20"
    );

    let mut legal_states: Vec<LegalState> = Vec::new();
    record_legal_state(
        &mut emitter,
        &mut legal_states,
        "beta0",
        "first_file",
        &current,
        &memory_pool_snapshot(&devices),
    );
    let beta0_basis = basis_of("beta0_k1", &memory_pool_snapshot(&devices), &current, true);
    // 镜像读法与内存读法在未 corrupt 的 β0 上应当一致（内部自洽核）。
    let memory_holds = allocated_minus_deferred_matches_referenced(
        &allocator,
        DeviceIdentity(0),
        beta0_basis.referenced,
    );
    let mirror_red = allocated_minus_deferred_mismatches_referenced(
        &beta0_basis.pool,
        beta0_basis.accounting_slot,
        beta0_basis.referenced,
    );
    emitter.emit(&format!(
        "name=beta0_mirror_vs_memory memory_holds={memory_holds} mirror_check_is_red={mirror_red}"
    ));
    assert_eq!(
        memory_holds, !mirror_red,
        "β0 上镜像读法与内存读法应当一致（都还没 corrupt）"
    );

    // ===== H0：暖机之后连续覆盖写 6N 次（N = 3S，S 是每区槽数；ρ = 1，每次都是 O），写失败即截断 =====
    let ring_length = ROOT_RING_REGIONS * E156_SLOTS_PER_REGION;
    let baseline_workload_length = 6 * ring_length;
    let hr_prefix_length = 3 * ring_length;
    let hr_tail_length = 3 * ring_length;
    emitter.emit(&format!(
        "name=geometry ring_length={ring_length} baseline_workload_length={baseline_workload_length} hr_prefix_length={hr_prefix_length} hr_tail_length={hr_tail_length}"
    ));

    let s1d_cutoff = 3 * E156_SLOTS_PER_REGION + 6;
    let mut baseline_workload_actual_length: u64 = 0;
    // Q3r.2（第七节 R-3）：闭式不再是常数，逐次核对，出不符不 panic（S1(e)(f) 现在是「两边都查」的
    // 观测型停机，不是硬 panic：一次不符就让整轮产物都出不来，反而没法看后面每一步的读数）。
    let mut overwrite_release_samples: Vec<u64> = Vec::new();
    let mut s1ef_mismatches: u64 = 0;
    let mut s1ef_leaf_count_histogram: BTreeMap<usize, u64> = BTreeMap::new();
    for step in 1..=baseline_workload_length {
        let existing_leaves_before_step = e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
        let records_before_step = allocator.records().len();
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        let outcome = publish_overwrite(
            &mut pool,
            &mut allocator,
            &current,
            FirstFile {
                content: &overwrite_content(step),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 + step,
            },
            instance,
        );
        let published = match outcome {
            Ok(published) => published,
            Err(failure) => {
                // 分配记录树按位置寻址之后（D8（核心索引结构） 已定项 14）没有一个节点 812 条那道墙，截断只剩别的失败。
                emitter.emit(&format!(
                    "name=baseline_workload_truncated_by_write_failure requested_length={baseline_workload_length} actual_length={baseline_workload_actual_length} failed_at_step={step} failure={failure:?}"
                ));
                break;
            }
        };
        current = published;
        baseline_workload_actual_length = step;
        // R-3a/R-3b（第七节 7.2）：一次 O 的释放槽数与记录增量按闭式逐次核对（Q3r.2）。
        let released = self_release_slots_of_this_publish(
            &allocator,
            DeviceIdentity(0),
            current.root.checkpoint_txg,
        );
        let touched_leaves_this_step =
            e156_touched_leaf_positions(&allocator, DeviceIdentity(0), current.root.checkpoint_txg);
        let (expected_released, expected_record_delta) = e156_closed_form_expected(
            &existing_leaves_before_step,
            &touched_leaves_this_step,
            true,
        );
        let record_count = allocator.records().len();
        let record_delta = u64::try_from(record_count - records_before_step)
            .expect("这一步的记录增量落在 u64 内");
        let step_matches = released == expected_released && record_delta == expected_record_delta;
        if !step_matches {
            s1ef_mismatches += 1;
        }
        let changed_internal_count = touched_leaves_this_step
            .iter()
            .map(|&leaf| e156_level1_position_of_leaf(leaf))
            .collect::<BTreeSet<_>>()
            .len();
        *s1ef_leaf_count_histogram
            .entry(touched_leaves_this_step.len())
            .or_insert(0) += 1;
        emitter.emit(&format!(
            "name=s1ef_step step={step} txg={} changed_leaves={} changed_internal={changed_internal_count} released_d0={released} expected_released_d0={expected_released} record_delta={record_delta} expected_record_delta={expected_record_delta} matches={step_matches}",
            current.root.checkpoint_txg.0,
            touched_leaves_this_step.len(),
        ));
        overwrite_release_samples.push(released);
        let pool_snapshot = memory_pool_snapshot(&devices);
        // S1(d)：前 3S + 6 次逐次报第 1/2/5 项与今天两条检查（应当全绿：这些都是合法状态）。
        if step <= s1d_cutoff {
            let (item1, item2, item5) = mirror_accounting_row_slots(
                &pool_snapshot,
                current.unit(TransactionUnit::AccountingTree).slot.0,
            );
            emitter.emit(&format!(
                "name=s1d_step txg={} item1={item1} item2={item2} item5={item5} i31_red={} i52_red={}",
                current.root.checkpoint_txg.0,
                is_red(&verdict(&pool_snapshot, "I-3.1")),
                is_red(&verdict(&pool_snapshot, "I-5.2")),
            ));
        }
        record_legal_state(
            &mut emitter,
            &mut legal_states,
            "H0",
            "O",
            &current,
            &pool_snapshot,
        );
    }
    if baseline_workload_actual_length == baseline_workload_length {
        emitter.emit(&format!(
            "name=baseline_workload_completed_full_length length={baseline_workload_length}"
        ));
    }
    // Q3r.2 汇总：H0 上逐次核对的不符次数与「改动落在几片叶」的直方图（第八节 8.2 要求两个方向都要有：
    // 落在 1 片叶与落在 ≥ 2 片叶的步各至少一次）。
    let s1ef_histogram_text: Vec<String> = s1ef_leaf_count_histogram
        .iter()
        .map(|(leaf_count, steps)| format!("{leaf_count}:{steps}"))
        .collect();
    emitter.emit(&format!(
        "name=s1ef_summary steps={baseline_workload_actual_length} mismatches={s1ef_mismatches} steps_by_changed_leaf_count={}",
        s1ef_histogram_text.join(",")
    ));
    let beta1_basis = basis_of("beta1_h0", &memory_pool_snapshot(&devices), &current, true);
    emitter.emit(&format!(
        "name=beta1_h0_end txg={} accounting_slot={} referenced={}",
        current.root.checkpoint_txg.0, beta1_basis.accounting_slot, beta1_basis.referenced
    ));

    // ===== HR：H0 前 3N 次（与 H0 共享前缀，从同一条真实历史分叉）→ 管理员回退到候选集里 txg 最小的根（影子账 On）
    //          → 工作负载段 3N 次 =====
    // 为了不与 H0 共用同一份可变设备状态（分叉不能回改已经跑过的 H0），HR 从 mkfs 重新独立跑一遍前 3N 步。
    let mut hr_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let hr_genesis = make_filesystem(&parameters, &mut hr_devices).expect("HR mkfs");
    let mut hr_allocator = PoolAllocator::new(
        hr_devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    hr_allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let hr_instance = {
        let mut pool = PoolWriter::new(&parameters, hr_devices.as_mut_slice());
        acquire_instance(&mut pool).expect("HR 取号")
    };
    let hr_warm_up = {
        let mut pool = PoolWriter::new(&parameters, hr_devices.as_mut_slice());
        warm_up(&mut pool, &hr_genesis.root, hr_instance).expect("HR 暖机")
    };
    let mut hr_current: TransactionOutput = {
        let mut pool = PoolWriter::new(&parameters, hr_devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut hr_allocator,
            hr_warm_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            hr_instance,
            &hr_warm_up.last_record_bytes,
        )
        .expect("HR 第一个事务")
    };
    record_legal_state(
        &mut emitter,
        &mut legal_states,
        "HR",
        "first_file",
        &hr_current,
        &memory_pool_snapshot(&hr_devices),
    );
    // 同 H0：分配记录树按位置寻址之后（D8（核心索引结构） 已定项 14）没有一个节点 812 条那道墙，截断只剩别的失败。
    let mut hr_prefix_actual_length: u64 = 0;
    for step in 1..=hr_prefix_length {
        let mut pool = PoolWriter::new(&parameters, hr_devices.as_mut_slice());
        let outcome = publish_overwrite(
            &mut pool,
            &mut hr_allocator,
            &hr_current,
            FirstFile {
                content: &overwrite_content(step),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 + step,
            },
            hr_instance,
        );
        let published = match outcome {
            Ok(published) => published,
            Err(failure) => {
                emitter.emit(&format!(
                    "name=hr_prefix_truncated_by_write_failure requested_length={hr_prefix_length} actual_length={hr_prefix_actual_length} failed_at_step={step} failure={failure:?}"
                ));
                break;
            }
        };
        hr_current = published;
        hr_prefix_actual_length = step;
        // Q7d-1（R2 ⑤）：H0 与 HR 的每一次 O 都要计进「这次发布自己的释放」的取样；HR 这里只量测，
        // 不逐次核闭式（Q3r.2 只在 H0 一条历史上核，第八节 8.2）。
        overwrite_release_samples.push(self_release_slots_of_this_publish(
            &hr_allocator,
            DeviceIdentity(0),
            hr_current.root.checkpoint_txg,
        ));
        record_legal_state(
            &mut emitter,
            &mut legal_states,
            "HR",
            "O",
            &hr_current,
            &memory_pool_snapshot(&hr_devices),
        );
    }
    // 回退候选集 = 按实例表判仍然有效 ∧ txg ≥ F_生效（D16（发布语义） 已定项 1）；HR 没抬过 F（F_生效 = 0），
    // 这一条过滤在 HR 上恒真，写出来只为与 HK 那一处同一个读法，不是两处各判各的。
    let hr_effective_floor = effective_rollback_floor(
        &hr_devices,
        &region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let candidates = readable_roots(
        &hr_devices,
        &region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let oldest_candidate = candidates
        .iter()
        .filter(|root| root.checkpoint_txg.0 >= hr_effective_floor.0)
        .min_by_key(|root| root.checkpoint_txg.0)
        .expect("回退候选集至少一个根");
    let rollback_target = RollbackTarget {
        instance: oldest_candidate.instance,
        checkpoint_txg: oldest_candidate.checkpoint_txg,
    };
    emitter.emit(&format!(
        "name=hr_rollback_target instance={} txg={} candidate_count={}",
        rollback_target.instance.0,
        rollback_target.checkpoint_txg.0,
        candidates.len()
    ));
    let mounted = mount_rollback(
        &parameters,
        &mut hr_devices,
        rollback_target,
        ShadowLedger::On,
    )
    .expect("HR 管理员回退");
    hr_allocator = mounted.allocator;
    record_legal_state(
        &mut emitter,
        &mut legal_states,
        "HR",
        "rollback_row",
        mounted
            .output
            .row_publish
            .file_version()
            .expect("回退写行那一版带文件"),
        &memory_pool_snapshot(&hr_devices),
    );
    // 第 10 个基底（E156 第 3 次重跑登记「十二」修订 4）：这一步是这个装置迄今第一个第 5 项恰为 0 的
    // 可达合法状态（`q7d2_min_item5` 在这一步之后会读到 `family=HR kind=rollback_row txg=76`）——
    // 岔路 7 Q7c① 的判别力自证隐含要求「基底第 5 项恰为 0」，此前 7 个可达基底都不满足，这里第一次有对象。
    let beta_hr_rollback_row = basis_of(
        "beta_hr_rollback_row",
        &memory_pool_snapshot(&hr_devices),
        mounted
            .output
            .row_publish
            .file_version()
            .expect("回退写行那一版带文件"),
        true,
    );
    for warm in &mounted.output.warm_up_publishes {
        record_legal_state(
            &mut emitter,
            &mut legal_states,
            "HR",
            "warmup",
            warm.file_version().expect("暖机那一版带文件"),
            &memory_pool_snapshot(&hr_devices),
        );
    }
    hr_current = mounted
        .current
        .file_version()
        .expect("回退目标带文件版本")
        .clone();
    let hr_instance_after_rollback = mounted.output.instance;

    let mut hr_tail_actual_length: u64 = 0;
    for step in 1..=hr_tail_length {
        let mut pool = PoolWriter::new(&parameters, hr_devices.as_mut_slice());
        let outcome = publish_overwrite(
            &mut pool,
            &mut hr_allocator,
            &hr_current,
            FirstFile {
                content: &overwrite_content(hr_prefix_length + step),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 + hr_prefix_length + step,
            },
            hr_instance_after_rollback,
        );
        let published = match outcome {
            Ok(published) => published,
            Err(failure) => {
                emitter.emit(&format!(
                    "name=hr_tail_truncated_by_write_failure requested_length={hr_tail_length} actual_length={hr_tail_actual_length} failed_at_step={step} failure={failure:?}"
                ));
                break;
            }
        };
        hr_current = published;
        hr_tail_actual_length = step;
        overwrite_release_samples.push(self_release_slots_of_this_publish(
            &hr_allocator,
            DeviceIdentity(0),
            hr_current.root.checkpoint_txg,
        ));
        record_legal_state(
            &mut emitter,
            &mut legal_states,
            "HR",
            "O",
            &hr_current,
            &memory_pool_snapshot(&hr_devices),
        );
    }
    emitter.emit(&format!(
        "name=hr_lengths requested_prefix={hr_prefix_length} actual_prefix={hr_prefix_actual_length} requested_tail={hr_tail_length} actual_tail={hr_tail_actual_length}"
    ));
    let beta2_basis = basis_of(
        "beta2_hr",
        &memory_pool_snapshot(&hr_devices),
        &hr_current,
        true,
    );
    emitter.emit(&format!(
        "name=beta2_hr_end txg={} instance={} accounting_slot={} referenced={}",
        hr_current.root.checkpoint_txg.0,
        hr_instance_after_rollback.0,
        beta2_basis.accounting_slot,
        beta2_basis.referenced
    ));

    // ===== HK（真实：GatedByTheRollbackFloor）与 HK-F0（不可达阳性对照：ForcedToZero） =====
    let hk = run_hk_family(
        &parameters,
        &region_devices,
        spacing,
        slots_per_region,
        "HK",
        ReuseWindow::GatedByTheRollbackFloor,
        true,
        &mut emitter,
        &mut legal_states,
    );
    // S1(e)：空发布 E 自己的释放槽数不再是登记第一、二版钉的常数 4（分配记录树按位置寻址之后
    // 不再是一个节点，R-5：这一次 = 8）；闭式核对已经在 `run_hk_family` 里对 `label == "HK"` 做过
    // （`name=r5_empty_publish_closed_form`），这里只留一行观测方便直接搜。
    emitter.emit(&format!(
        "name=s1e_empty_publish_release slots={}",
        hk.empty_publish_release_slots
    ));

    let mut hk_forced_to_zero_legal_states: Vec<LegalState> = Vec::new();
    let hk_forced_to_zero = run_hk_family(
        &parameters,
        &region_devices,
        spacing,
        slots_per_region,
        "HK-F0",
        ReuseWindow::ForcedToZero,
        false,
        &mut emitter,
        &mut hk_forced_to_zero_legal_states,
    );
    let beta_forced_to_zero = BasisSnapshot {
        label: "beta_F0".to_string(),
        reachable: false,
        ..hk_forced_to_zero.beta_first
    };

    // ===== β_syn：β0 的镜像做 (第 1 项 −1、第 5 项 −1、第 2 项 +1) =====
    let beta_syn_pool = corrupt_device_zero_accounting(
        &beta0_basis.pool,
        beta0_basis.accounting_slot,
        beta0_basis.tree_table_slot,
        &region_devices,
        spacing,
        &[
            (1u16, -i64::try_from(SLOT_BYTES).expect("16384")),
            (5u16, -i64::try_from(SLOT_BYTES).expect("16384")),
            (2u16, i64::try_from(SLOT_BYTES).expect("16384")),
        ],
    );
    let beta_syn = BasisSnapshot {
        label: "beta_syn".to_string(),
        pool: beta_syn_pool,
        accounting_slot: beta0_basis.accounting_slot,
        tree_table_slot: beta0_basis.tree_table_slot,
        referenced: beta0_basis.referenced,
        reachable: false,
    };
    let (syn_item1, syn_item2, syn_item5) =
        mirror_accounting_row_slots(&beta_syn.pool, beta_syn.accounting_slot);
    emitter.emit(&format!(
        "name=beta_syn_row item1={syn_item1} item2={syn_item2} item5={syn_item5} referenced={}",
        beta_syn.referenced
    ));

    // ===== S1(c) =====
    run_s1c_rollback_isolation_scenario(&mut emitter, &parameters);

    // ===== Q7b：HK、H0、HR 全部合法状态上 G27 判红的状态数（L1、L2 分开数） =====
    let (
        mut red_count_before_crash_recovery,
        mut total_count_before_crash_recovery,
        mut red_count_after_crash_recovery,
        mut total_count_after_crash_recovery,
    ) = (0u64, 0u64, 0u64, 0u64);
    let mut first_red_legal_states: Vec<String> = Vec::new();
    for state in &legal_states {
        let is_after_crash_recovery = state.kind.contains("after_c2");
        if is_after_crash_recovery {
            total_count_after_crash_recovery += 1;
        } else {
            total_count_before_crash_recovery += 1;
        }
        if state.check_is_red {
            if is_after_crash_recovery {
                red_count_after_crash_recovery += 1;
            } else {
                red_count_before_crash_recovery += 1;
            }
            if first_red_legal_states.len() < 5 {
                first_red_legal_states
                    .push(format!("{}:{}:txg={}", state.family, state.kind, state.txg));
            }
        }
    }
    emitter.emit(&format!(
        "name=q7b red_l1={red_count_before_crash_recovery} total_l1={total_count_before_crash_recovery} red_l2={red_count_after_crash_recovery} total_l2={total_count_after_crash_recovery} first_red={}",
        first_red_legal_states.join(" | ")
    ));

    // ===== Q7d-1（R2 ⑤，逐次现量）：按发布种类分组的「这次发布自己的释放」（min/max/count）。
    // O 不再是常数：`overwrite_release_samples` 逐次收自 H0 与 HR（前缀 + 尾段）的每一次 O；
    // E 的唯一样本来自 HK（`hk.empty_publish_release_slots`）。 =====
    let (overwrite_release_minimum, overwrite_release_maximum) =
        e156_minimum_and_maximum(&overwrite_release_samples);
    emitter.emit(&format!(
        "name=q7d1_by_kind kind=O min={overwrite_release_minimum} max={overwrite_release_maximum} count={}",
        overwrite_release_samples.len()
    ));
    emitter.emit(&format!(
        "name=q7d1_by_kind kind=E min={0} max={0} count=1",
        hk.empty_publish_release_slots
    ));

    // ===== Q7d-2：可达合法状态（H0、HR、HK）上第 5 项的最小值 =====
    let smallest_item5_state = legal_states
        .iter()
        .min_by_key(|state| state.item5)
        .expect("legal_states 非空");
    emitter.emit(&format!(
        "name=q7d2_min_item5 min_item5={} family={} kind={} txg={}",
        smallest_item5_state.item5,
        smallest_item5_state.family,
        smallest_item5_state.kind,
        smallest_item5_state.txg
    ));

    // ===== Q7d-3（= PC-可达）：HK-F0 全部发布之后状态与 β_syn 上的同一个量必须 = 0 =====
    let hk_forced_to_zero_smallest_item5 = hk_forced_to_zero_legal_states
        .iter()
        .map(|state| state.item5)
        .min()
        .unwrap_or(u64::MAX);
    let q7d3_holds = hk_forced_to_zero_smallest_item5 == 0 && syn_item5 == 0;
    emitter.emit(&format!("name=q7d3_pc_reachable hk_forced_to_zero_smallest_item5_item5={hk_forced_to_zero_smallest_item5} beta_syn_item5={syn_item5} holds={q7d3_holds}"));

    // ===== 统一的基底列表：Q7a（三档 delta）、Q7c①②、PC-检查、Q7f =====
    // 第 10 个（beta_hr_rollback_row，E156 第 3 次重跑登记「十二」修订 4）：唯一一个第 5 项恰为 0 的
    // 可达基底，Q7c① 的判别力自证第一次在它上面有对象（见插入这个基底那一处的注释）。
    let bases: [&BasisSnapshot; 10] = [
        &beta0_basis,
        &beta1_basis,
        &beta2_basis,
        &hk.beta_after_reopen,
        &hk.beta_after_rollback,
        &hk.beta_after_raise_floor,
        &hk.beta_after_crash_recovery,
        &beta_hr_rollback_row,
        &beta_syn,
        &beta_forced_to_zero,
    ];
    let mut q7a_all_red_count = 0u64;
    let mut flips: Vec<(String, bool, bool)> = Vec::new();
    let mut q7f_count = 0u64;
    for basis in bases {
        let (item1, _item2, item5) =
            mirror_accounting_row_slots(&basis.pool, basis.accounting_slot);
        emitter.emit(&format!(
            "name=basis_snapshot label={} item1={item1} item5={item5} referenced={} reachable={}",
            basis.label, basis.referenced, basis.reachable
        ));
        for delta in [1i64, -1, 8] {
            let (_allocated, _free, deferred) =
                mirror_accounting_row_slots(&basis.pool, basis.accounting_slot);
            if delta < 0 && u64::try_from(-delta).expect("delta 的绝对值") > deferred {
                emitter.emit(&format!("name=q7a basis={} defer_delta={delta} not_applicable=true reason=deferred_underflow", basis.label));
                continue;
            }
            let (corrupted_red, both_green) = run_q7a_cell(
                &mut emitter,
                &basis.label,
                &delta.to_string(),
                &basis.pool,
                basis.accounting_slot,
                basis.tree_table_slot,
                &region_devices,
                spacing,
                basis.referenced,
                delta,
            );
            if corrupted_red && both_green {
                q7a_all_red_count += 1;
            }
        }
        let (flip1, flip2) = q7c_self_test(
            &mut emitter,
            &basis.label,
            &basis.pool,
            basis.accounting_slot,
            basis.referenced,
        );
        flips.push((basis.label.clone(), flip1, flip2));
        run_pc_check(
            &mut emitter,
            &basis.label,
            &basis.pool,
            basis.accounting_slot,
            basis.tree_table_slot,
            &region_devices,
            spacing,
            basis.referenced,
        );

        // Q7f（附带）：只在可达基底上，真实 G27 绿 ∧「不减第 5 项」变体红。
        if basis.reachable {
            let real_red = allocated_minus_deferred_mismatches_referenced(
                &basis.pool,
                basis.accounting_slot,
                basis.referenced,
            );
            let variant_red = item1 != basis.referenced;
            if !real_red && variant_red {
                q7f_count += 1;
            }
        }
    }
    let q7c1_flip_seen = flips.iter().any(|(_, flip1, _)| *flip1);
    let q7c2_flip_seen = flips.iter().any(|(_, _, flip2)| *flip2);
    emitter.emit(&format!("name=q7a_summary all_red_count={q7a_all_red_count} q7c1_flip_seen={q7c1_flip_seen} q7c2_flip_seen={q7c2_flip_seen}"));
    emitter.emit(&format!("name=q7f_count count={q7f_count}"));

    // ===== Q7e：Q7c①②③ 在 β_F0、β_syn 上转不转色（从上面的 flips 里挑出来，另加今天两条检查的判定） =====
    let (_, beta_forced_to_zero_flip1, beta_forced_to_zero_flip2) = flips
        .iter()
        .find(|(label, _, _)| label == "beta_F0")
        .cloned()
        .unwrap_or((String::new(), false, false));
    let (_, beta_syn_flip1, beta_syn_flip2) = flips
        .iter()
        .find(|(label, _, _)| label == "beta_syn")
        .cloned()
        .unwrap_or((String::new(), false, false));
    emitter.emit(&format!(
        "name=q7e beta_forced_to_zero_flip1={beta_forced_to_zero_flip1} beta_forced_to_zero_flip2={beta_forced_to_zero_flip2} beta_syn_flip1={beta_syn_flip1} beta_syn_flip2={beta_syn_flip2} beta_forced_to_zero_i31_red={} beta_forced_to_zero_i52_red={} beta_syn_i31_red={} beta_syn_i52_red={}",
        is_red(&verdict(&beta_forced_to_zero.pool, "I-3.1")), is_red(&verdict(&beta_forced_to_zero.pool, "I-5.2")),
        is_red(&verdict(&beta_syn.pool, "I-3.1")), is_red(&verdict(&beta_syn.pool, "I-5.2")),
    ));

    // ===== 完整性闸（V7）：族数、取样点数、逐发布行数（岔路 7，第一段）=====
    emitter.emit(&format!(
        "name=integrity families=3 legal_state_rows={} hk_forced_to_zero_rows={} basis_count=10",
        legal_states.len(),
        hk_forced_to_zero_legal_states.len(),
    ));

    // ===== 岔路 3（第二段，缩小范围：S = 8、ρ = 1 一个几何取样点，见文件顶注释）=====
    run_hf_single_cell(&parameters, &mut emitter);
    run_small_pool_cell(&mut emitter, "hx", None);
    // R-7：HY 的覆盖写次数不再写死 3——3 次不满足 e ≥ max(8, f) 就减少直到满足（第十二节记录用了几次）。
    let hy_cap = e156_find_hy_cap_satisfying_open_segment_condition(&small_pool_parameters());
    emitter.emit(&format!("name=r7_hy_cap chosen_cap={hy_cap}"));
    run_small_pool_cell(&mut emitter, "hy", Some(hy_cap));

    // ===== 岔路 1（第三段 a 的一部分，S 这一维：S = 8（mkfs 默认）与 S = 4（下界，方向相反，
    // 「五、5.6」第六类「至少一个方向相反的取样点」），ρ = 1、回收时点固定「实」、洞位置固定「后」，
    // k ∈ {0, 1, 2}；ρ = 1/4、回收时点「每」、洞位置「前」、S = 16 这几维见交回报告岔路表，这一段未扫）=====
    let parameters_at_four_slots_per_region = parameters_with_slots_per_region(4);
    let slots_per_region_at_four = parameters_at_four_slots_per_region
        .geometry
        .root_ring_slots_per_region;
    // K8（新，登记「七」）：mkfs 喂的系统配置字段与本地打算跑的 S 一致。这里核的是「喂给 mkfs 的值」，
    // 不是从盘上字节读回来的值——与文件顶那条 S = 8 的基线检查（`E156_SLOTS_PER_REGION` 那条 assert）
    // 是同一个严格程度，不是从盘上重新解析系统配置槽。
    assert_eq!(
        4,
        slots_per_region_at_four.count(),
        "K8：S = 4 这一格，本地打算跑的 S 与喂给 mkfs 的系统配置字段不等"
    );
    emitter.emit("name=anchor_k8 configured=4 local=4 holds=true");

    let mut hh_cells: Vec<HhCell> = Vec::new();
    for (geometry_parameters, geometry_slots_per_region) in [
        (&parameters, slots_per_region),
        (
            &parameters_at_four_slots_per_region,
            slots_per_region_at_four,
        ),
    ] {
        for holes in [0u64, 1, 2] {
            let cell = run_hh_cell(
                geometry_parameters,
                &region_devices,
                spacing,
                geometry_slots_per_region,
                holes,
                0,
                &mut emitter,
            );
            hh_cells.push(cell);
        }
    }
    // Q1d：相邻两点的差，第一次让 Δ 越过 2 槽 / 1 槽的 (h_missing, txg)；另报 q1a/q1b 各自的轨迹与 h_missing，
    // 免得只看 Δ 看不出是哪一边在长（`test-discipline.md`「端点不是轨迹」）。按 S 分组比较：跨 S 比较
    // holes 相邻两点没有意义（S 一变，ring_length、造洞的时点全变），只在同一个 S 内部比。
    let mut crossing_two_slots_by_slots_per_region: Vec<(u64, bool)> = Vec::new();
    for group_slots_per_region in [8u64, 4u64] {
        let group: Vec<&HhCell> = hh_cells
            .iter()
            .filter(|cell| cell.slots_per_region == group_slots_per_region)
            .collect();
        for window in group.windows(2) {
            let (previous, next) = (window[0], window[1]);
            emitter.emit(&format!(
                "name=q1d_adjacent_diff s={group_slots_per_region} holes_from={} holes_to={} q1a_from={} q1a_to={} q1b_from={} q1b_to={} delta_from={} delta_to={} diff={} h_missing_from={} h_missing_to={}",
                previous.holes, next.holes, previous.q1a_over_withheld, next.q1a_over_withheld,
                previous.q1b_over_withheld, next.q1b_over_withheld, previous.delta, next.delta,
                i64::try_from(next.delta).unwrap_or(i64::MAX) - i64::try_from(previous.delta).unwrap_or(0),
                previous.missing_txg_count, next.missing_txg_count,
            ));
        }
        let first_crossing_2 = group.iter().find(|cell| cell.delta > 2);
        let first_crossing_1 = group.iter().find(|cell| cell.delta > 1);
        crossing_two_slots_by_slots_per_region
            .push((group_slots_per_region, first_crossing_2.is_some()));
        emitter.emit(&format!(
            "name=q1d_first_crossing s={group_slots_per_region} threshold_2_slots={} threshold_1_slot={}",
            first_crossing_2.map_or("not_crossed_in_sampled_range".to_string(), |cell| format!(
                "holes={} txg={} delta={}",
                cell.holes, cell.txg, cell.delta
            )),
            first_crossing_1.map_or("not_crossed_in_sampled_range".to_string(), |cell| format!(
                "holes={} txg={} delta={}",
                cell.holes, cell.txg, cell.delta
            )),
        ));
        let monotonic = group
            .windows(2)
            .all(|window| window[1].delta >= window[0].delta);
        emitter.emit(&format!(
            "name=q1d_monotonic s={group_slots_per_region} holds={monotonic}"
        ));
    }
    // 几何敏感性（「八」，第六类）：S = 8 与 S = 4 这两个取样点上，「Δ 越过 2 槽」这条判定是不是同一个值。
    // 相同 ⇒ 报「2 个取样点一致」；不同 ⇒ 报「不稳定（依赖几何）」，正文不许把它写成臂或检查的性质（F9）。
    let crossing_2_values: Vec<bool> = crossing_two_slots_by_slots_per_region
        .iter()
        .map(|(_, crossed)| *crossed)
        .collect();
    let crossing_2_stable = crossing_2_values
        .windows(2)
        .all(|window| window[0] == window[1]);
    emitter.emit(&format!(
        "name=q1_geometry_sensitivity_s crossing_two_slots_by_slots_per_region={} stable={crossing_2_stable}",
        crossing_two_slots_by_slots_per_region
            .iter()
            .map(|(slots_per_region_value, crossed)| format!("s{slots_per_region_value}={crossed}"))
            .collect::<Vec<_>>()
            .join(","),
    ));

    // ===== 主 agent 续派第 1 条：步数对齐对照——k=0，但工作负载发布次数补到与 holes=1 / holes=2 相同
    // （同样只在最后无崩溃重开一次），S = 8 与 S = 4 各一组。目的：把「Δ 随洞数长」与「Δ 随步数长」分开。=====
    let mut matched_cells: Vec<HhCell> = Vec::new();
    for (geometry_parameters, geometry_slots_per_region) in [
        (&parameters, slots_per_region),
        (
            &parameters_at_four_slots_per_region,
            slots_per_region_at_four,
        ),
    ] {
        for matched_to_holes in [1u64, 2] {
            let cell = run_hh_cell(
                geometry_parameters,
                &region_devices,
                spacing,
                geometry_slots_per_region,
                0,
                matched_to_holes,
                &mut emitter,
            );
            matched_cells.push(cell);
        }
    }
    // Δ(k) − Δ(0, 步数对齐)：真实 holes=k 格与「没有洞、但总步数相同」那格的 Δ 之差。差 ≈ 0 ⇒ Δ 的
    // 增长主要来自步数、与洞本身无关；差远大于 0 ⇒ 洞本身在同样的步数下额外贡献了这么多多扣。
    for group_slots_per_region in [8u64, 4u64] {
        for matched_to_holes in [1u64, 2] {
            let real_cell = hh_cells
                .iter()
                .find(|cell| {
                    cell.slots_per_region == group_slots_per_region
                        && cell.holes == matched_to_holes
                })
                .expect("真实 holes=k 格应当已经跑过");
            let matched_cell = matched_cells
                .iter()
                .find(|cell| {
                    cell.slots_per_region == group_slots_per_region
                        && cell.matched_to_holes == matched_to_holes
                })
                .expect("步数对齐对照格应当已经跑过");
            assert_eq!(
                real_cell.txg, matched_cell.txg,
                "步数对齐没对上：真实 holes={matched_to_holes}（s={group_slots_per_region}）与它的步数对照格终点 txg 应当相同"
            );
            let diff = i64::try_from(real_cell.delta).unwrap_or(i64::MAX)
                - i64::try_from(matched_cell.delta).unwrap_or(0);
            emitter.emit(&format!(
                "name=q1_step_matched_diff s={group_slots_per_region} k={matched_to_holes} txg={} delta_k={} delta_0_matched={} diff={diff} h_missing_k={} h_missing_0_matched={}",
                real_cell.txg, real_cell.delta, matched_cell.delta,
                real_cell.missing_txg_count, matched_cell.missing_txg_count,
            ));
        }
    }

    // ===== 完整性闸（V7，第二/三段追加）：这一轮新增的取样点数 =====
    emitter.emit(&format!(
        "name=integrity_r2_segments hf_cells=1 hh_cells={} hh_holes_sampled={} matched_cells={} matched_sampled={}",
        hh_cells.len(),
        hh_cells
            .iter()
            .map(|cell| format!("s{}h{}", cell.slots_per_region, cell.holes))
            .collect::<Vec<_>>()
            .join(","),
        matched_cells.len(),
        matched_cells
            .iter()
            .map(|cell| format!("s{}m{}", cell.slots_per_region, cell.matched_to_holes))
            .collect::<Vec<_>>()
            .join(","),
    ));

    emitter.finish();
}

#[cfg(test)]
mod tests {
    //! K1（第一个事务之后的记账三项）、G27（`referenced_slots`）、R8（U7）、R3 的判别力（U8）的最小单测。
    //! `crates/mutations.tsv` 里 `e156_referenced_slots_drops_instance_table` 那条钉着 `referenced_slots`
    //! 的实例表兜底分支：删掉它，第一条测试必须红（`assert_eq!` 那一行断言 12，变异之后会算出 10）。
    use super::{
        accounting_row_slots, allocated_minus_deferred_matches_referenced,
        allocated_minus_deferred_mismatches_referenced, basis_of, corrupt_device_zero_accounting,
        e156_allocation_record_tree_new_node_count, e156_closed_form_expected,
        e156_existing_leaf_positions, e156_touched_leaf_positions,
        memory_pool_snapshot, mirror_accounting_row_slots, parameters, q7c_self_test,
        referenced_slots, run_s1c_rollback_isolation_scenario,
        self_release_slots_of_this_publish, Emitter, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
    };
    use std::collections::BTreeSet;
    use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
    use singlefs_core::allocator::{
        AllocationRecord, DeviceFreeMap, Placement, PoolAllocator, ReclaimedReuse,
    };
    use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
    use singlefs_core::make_filesystem::{
        make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
    };
    use singlefs_core::mount::{mount_rollback, RollbackTarget, ShadowLedger};
    use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
    use singlefs_core::transaction::{
        acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
        TransactionOutput, TransactionUnit,
    };
    use singlefs_format::{ROOT_RING_REGIONS, SLOT_BYTES};
    use singlefs_harness::crash::SparseBlockDevice;

    fn first_transaction_state() -> (
        PoolAllocator,
        TransactionOutput,
        Vec<(DeviceIdentity, SparseBlockDevice)>,
    ) {
        let parameters = parameters();
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
            .map(|number| {
                (
                    DeviceIdentity(number),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = PoolAllocator::new(
            devices
                .iter()
                .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
                .collect(),
        );
        allocator.mark_format_time_units(
            Placement {
                slot: INSTANCE_TABLE_SLOT,
                span: 2,
            },
            Placement {
                slot: TREE_TABLE_GENESIS_SLOT,
                span: 1,
            },
        );
        let instance = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            acquire_instance(&mut pool).expect("取号")
        };
        let warm_up_output = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            warm_up(&mut pool, &genesis.root, instance).expect("暖机")
        };
        let first = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            publish_first_file(
                &mut pool,
                &mut allocator,
                warm_up_output.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &super::overwrite_content(0),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &warm_up_output.last_record_bytes,
            )
            .expect("第一个事务")
        };
        (allocator, first, devices)
    }

    /// K1-1（E156 第 3 次重跑登记「七」7.2）：分配记录树按位置寻址之后第一个事务的记账第 1 项是 17
    /// （五个树节点占五条记录，不再是登记第一、二版的单节点 13），第 5 项仍是 1。
    #[test]
    fn accounting_after_first_transaction_matches_the_registered_anchor() {
        let (allocator, first, _devices) = first_transaction_state();
        assert_eq!(first.root.checkpoint_txg.0, 3);
        let (allocated, _free, deferred) = accounting_row_slots(&allocator, DeviceIdentity(0));
        assert_eq!(allocated, 17, "K1-1 第 1 项");
        assert_eq!(deferred, 1, "K1-1 第 5 项");
    }

    /// K1-3：β0 的最新根走读引用 = 16（实例表 2 槽 + 第一个事务九个角色共 14 槽，五个分配记录树节点
    /// 记在这 14 槽里），不再是登记第一、二版的 12。
    #[test]
    fn referenced_slots_counts_the_carried_instance_table_before_its_first_rewrite() {
        let (allocator, first, _devices) = first_transaction_state();
        let referenced = referenced_slots(&first);
        assert_eq!(
            referenced, 16,
            "实例表 2 槽（未进 units，靠兜底加回）+ 第一个事务九个角色共 14 槽（K1-3）"
        );
        assert!(
            allocated_minus_deferred_matches_referenced(&allocator, DeviceIdentity(0), referenced),
            "第一个事务之后 G27 应当成立：17 − 1 == 16（K1-3）"
        );
    }

    /// U7：第一个事务那一次发布的 R8「自己的释放」= 1 槽（mkfs 的树表单元）。
    #[test]
    fn first_transaction_self_release_is_one_slot() {
        let (allocator, first, _devices) = first_transaction_state();
        let released = self_release_slots_of_this_publish(
            &allocator,
            DeviceIdentity(0),
            first.root.checkpoint_txg,
        );
        assert_eq!(
            released, 1,
            "U7：第一个事务自己的释放应为 1 槽（mkfs 的树表单元）"
        );
    }

    /// U8：β0 的镜像做 Bd(+1)：G27（R3，读镜像）判红；同一镜像上内存分配器的行没变。
    /// `crates/mutations.tsv` 的 M21 把下面这一行的调用换成内存读法的反面，这条测试必须由绿转红。
    #[test]
    fn red_check_reads_the_corrupted_mirror_not_the_live_allocator() {
        let (allocator, first, devices) = first_transaction_state();
        let referenced = referenced_slots(&first);
        let accounting_slot = first.unit(TransactionUnit::AccountingTree).slot.0;
        let tree_table_slot = first.unit(TransactionUnit::TreeTable).slot.0;
        let region_devices = parameters().region_devices;
        let spacing = parameters().geometry.fixed_structure_slot_spacing;
        let base_pool = memory_pool_snapshot(&devices);
        let corrupted_pool = corrupt_device_zero_accounting(
            &base_pool,
            accounting_slot,
            tree_table_slot,
            &region_devices,
            spacing,
            &[(5u16, i64::try_from(SLOT_BYTES).expect("16384"))],
        );
        let mirror_red = allocated_minus_deferred_mismatches_referenced(
            &corrupted_pool,
            accounting_slot,
            referenced,
        );
        assert!(mirror_red, "U8：G27（R3，读镜像）在 Bd(+1) 上必须判红");
        let (memory_allocated, _free, memory_deferred) =
            accounting_row_slots(&allocator, DeviceIdentity(0));
        assert_eq!(
            (memory_allocated, memory_deferred),
            (17, 1),
            "U8：同一镜像上内存分配器的行没变（只改了镜像字节，K1-1：17/1）"
        );
    }

    /// 岔路 1 的分配代账本（G12 要的那 8 字节，真实记录里没有，登记「三」N2）：一次覆盖写释放的记录，账本里查到
    /// 的分配代应等于释放前它在「仍分配」快照里的那个值——不是这次发布自己的释放代（真实记录的 `generation`
    /// 字段释放那一刻被改写成释放代，登记「三」I1，装置只能另存一份）。
    #[test]
    fn allocation_generation_ledger_recovers_the_pre_release_generation() {
        let parameters = parameters();
        let (mut allocator, first, mut devices) = first_transaction_state();
        let before = super::snapshot_allocated_generations(&allocator);
        assert!(!before.is_empty(), "第一个事务之后应当有「仍分配」的记录");
        let instance = singlefs_core::address::InstanceGeneration(1);
        let overwritten = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            singlefs_core::transaction::publish_overwrite(
                &mut pool,
                &mut allocator,
                &first,
                FirstFile {
                    content: &super::overwrite_content(1),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 61,
                },
                instance,
            )
            .expect("一次覆盖写")
        };
        // 这次覆盖写自己释放的记录（device 0）：每一条在覆盖写之前必然是「仍分配」——它们理应全部出现在
        // `before` 这份「仍分配」快照里；`snapshot_allocated_generations` 认反方向（改成只收「已释放」的）
        // 就会漏光这些槽，这条断言必须抓到那种反向。
        let released_this_step: Vec<u64> = allocator
            .records()
            .iter()
            .filter(|record| {
                record.device == DeviceIdentity(0)
                    && record.is_released
                    && record.generation == overwritten.root.checkpoint_txg
            })
            .map(|record| record.slot.0)
            .collect();
        assert!(
            !released_this_step.is_empty(),
            "S1(e)：一次覆盖写应当释放至少一条记录"
        );
        for slot in &released_this_step {
            assert!(
                before.contains_key(slot),
                "这次覆盖写释放的槽 {slot} 在覆盖写之前必然『仍分配』，应当出现在 `before` 快照里"
            );
        }
        let mut ledger = std::collections::HashMap::new();
        super::record_release_generations(&mut ledger, &before, &allocator);
        assert!(
            !ledger.is_empty(),
            "一次覆盖写应当释放至少一条记录（S1(e)：应为 10 槽对应的记录）"
        );
        for slot in &released_this_step {
            assert!(
                ledger.contains_key(slot),
                "这次覆盖写释放的槽 {slot} 应当被记进分配代账本"
            );
        }
        for (slot, allocation_generation) in &ledger {
            assert_eq!(
                before.get(slot).copied(),
                Some(*allocation_generation),
                "账本记的分配代应与释放前『仍分配』快照里的那个值一致，槽 {slot}"
            );
        }
    }

    #[test]
    fn reclaimed_records_are_not_removed_but_their_slots_become_free() {
        // 派发第 1 条溯源出的事实（`PoolAllocator::reclaim_released_up_to` 不删记录，`allocator.rs` I8 字面）：
        // 一条记录被真实分配器回收之后，`is_released` 仍是 `true`、条目还留在 `records()` 里，直到下一次
        // `record()` 落在同一个起点槽上才被改写；但它的槽此刻已经是空闲的（`DeviceFreeMap::is_free`）。
        // `run_hh_cell` 的 Q1b／delta 诊断原来直接按 `is_released` 过全部记录，把这类「回收过但记录条目还没
        // 被覆盖」的槽也算成「还扣着」（holes=0 那一格因此把 unblocked 集合多算了 11 槽，见交回报告）；
        // 本轮改成额外核 `is_free` 排除它们。这条测试钉住这份诊断赖以成立的前提本身：前提变了（比如哪天
        // 回收改成即时删记录），这条测试必须先红，而不是等 Q1a/Q1b 的数字悄悄错掉才被发现。
        let (mut allocator, first, mut devices) = first_transaction_state();
        let parameters = parameters();
        let instance = singlefs_core::address::InstanceGeneration(1);
        let overwritten = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            singlefs_core::transaction::publish_overwrite(
                &mut pool,
                &mut allocator,
                &first,
                FirstFile {
                    content: &super::overwrite_content(1),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 61,
                },
                instance,
            )
            .expect("一次覆盖写")
        };
        let released_slot = allocator
            .records()
            .iter()
            .find(|record| {
                record.device == DeviceIdentity(0)
                    && record.is_released
                    && record.generation == overwritten.root.checkpoint_txg
            })
            .map(|record| record.slot)
            .expect("这次覆盖写应当释放至少一条记录");
        allocator
            .reclaim_released_up_to(overwritten.root.checkpoint_txg, ReclaimedReuse::Immediately);
        let still_marked_released = allocator.records().iter().any(|record| {
            record.device == DeviceIdentity(0) && record.slot == released_slot && record.is_released
        });
        assert!(
            still_marked_released,
            "回收之后记录条目应当仍留着、`is_released` 仍是 true（I8：不删、不点删）"
        );
        let device_free_map_for_device_zero = allocator
            .devices
            .iter()
            .find(|map| map.device == DeviceIdentity(0))
            .expect("D0 在池里");
        assert!(
            device_free_map_for_device_zero.is_free(released_slot),
            "回收之后这个槽的位图应当已经清空（`is_free` 为 true），即使记录条目还标着 `is_released`"
        );
    }

    // U5（岔路 3，登记逐字「第一次推空之后那一行记账仍含这批槽」）这一轮没有补——`raise_rollback_floor_via_g7`
    // 不暴露推空循环中途的钩子，而空发布本身会重写树表单元（连带新分配 + 释放旧树表槽，登记 D16（发布语义）
    // 已定项 1「非空」段落逐字），使「推空前后记账第 1 项该差多少」不是一条简单算式，贸然钉一个数风险比价值大；
    // 写进交回报告的岔路表，留给下一段。

    /// PC-闭式自测第一、二、四组（E156 第 3 次重跑登记「七」7.2 R-3c）：只在合成的叶位置集合上单测
    /// 闭式函数本身，不依赖真实分配器。M27（闭式漏掉根）、M28（闭式每个改动的叶位置只算一块盘）应当
    /// 分别让第一组从 5 变成 4 与 3。
    #[test]
    fn pc_closed_form_matches_the_registered_anchor_r3c() {
        let one_leaf: BTreeSet<u64> = [61].into_iter().collect();
        let two_leaves: BTreeSet<u64> = [61, 62].into_iter().collect();
        let leaves_in_two_internal: BTreeSet<u64> = [61, 170].into_iter().collect();
        assert_eq!(
            e156_allocation_record_tree_new_node_count(&one_leaf, 2),
            5,
            "R-3c：只在第 61 片叶（两盘）"
        );
        assert_eq!(
            e156_allocation_record_tree_new_node_count(&two_leaves, 2),
            7,
            "R-3c：第 61、62 片叶"
        );
        assert_eq!(
            e156_allocation_record_tree_new_node_count(&leaves_in_two_internal, 2),
            9,
            "R-3c：第 61 与第 170 片叶（第 170 片在第二个层级 1 节点里）"
        );
        assert_eq!(
            e156_allocation_record_tree_new_node_count(&one_leaf, 1),
            3,
            "R-3c：一盘、只在第 61 片叶"
        );
        assert!(
            170 * 812 >= 812 * 169,
            "R-3c：170 号叶应落在第二个层级 1 节点里"
        );
    }

    /// PC-闭式第三组（R-3c，M29 的取样点）：合成一份记录——一条在第 61 片叶已释放、一条在第 170 片叶
    /// 仍分配，两条 `generation` 都是这次的 txg。`e156_touched_leaf_positions` 应当把两条都算进
    /// 「这次改动」，给出 9；M29（闭式的「改动的记录」只取这次分配的、不取这次释放的）应当让这一格
    /// 只剩第 170 片叶一条，变成 5。
    #[test]
    fn pc_closed_form_third_group_counts_released_and_allocated_records() {
        let devices = vec![DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES)];
        let txg = CheckpointTxg(9);
        let released_slot = 50245u64;
        let allocated_slot = 170u64 * 812;
        let records = vec![
            AllocationRecord {
                device: DeviceIdentity(0),
                slot: SlotNumber(released_slot),
                span_slots: 1,
                generation: txg,
                is_released: true,
            },
            AllocationRecord {
                device: DeviceIdentity(0),
                slot: SlotNumber(allocated_slot),
                span_slots: 1,
                generation: txg,
                is_released: false,
            },
        ];
        let allocator = PoolAllocator::rebuild_from_records(devices, records);
        let touched = e156_touched_leaf_positions(&allocator, DeviceIdentity(0), txg);
        assert_eq!(
            touched, BTreeSet::from([61, 170]),
            "PC-闭式第三组：应当同时看到释放在第 61 片、分配在第 170 片两条"
        );
        assert_eq!(
            e156_allocation_record_tree_new_node_count(&touched, 2),
            9,
            "R-3c：第 61 与第 170 片叶应给出 9"
        );
    }

    /// U10（第九节）：β0 之后连续覆盖写，逐次核对 R-3 的闭式；次序不钉死，只保证跑够多步、两个方向
    /// （落在 1 片叶 / 落在 ≥ 2 片叶）都至少出现一次（第八节 8.2）。前 4 次另外钉 R-4 的绝对值
    /// （released_d0=14、record_delta=24）。U12（Q7d-1，R2 ⑤）：这段历史上逐次现量的最小值 14、
    /// 最大值 ≥ 16——M30（Q7d-1 覆盖写那一组退回字面 10）应当让最小值变成 10。
    #[test]
    fn overwrite_steps_match_the_closed_form_and_cross_a_second_leaf() {
        let (mut allocator, first, mut devices) = first_transaction_state();
        let parameters = parameters();
        let instance = InstanceGeneration(1);
        let mut current = first;
        let mut release_samples: Vec<u64> = Vec::new();
        let mut saw_single_leaf_step = false;
        let mut saw_multi_leaf_step = false;
        for step in 1..=12u64 {
            let existing_leaves = e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
            let records_before = allocator.records().len();
            current = {
                let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                singlefs_core::transaction::publish_overwrite(
                    &mut pool,
                    &mut allocator,
                    &current,
                    FirstFile {
                        content: &super::overwrite_content(step),
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 + step,
                    },
                    instance,
                )
                .expect("U10 覆盖写")
            };
            let released = self_release_slots_of_this_publish(
                &allocator,
                DeviceIdentity(0),
                current.root.checkpoint_txg,
            );
            let touched = e156_touched_leaf_positions(
                &allocator,
                DeviceIdentity(0),
                current.root.checkpoint_txg,
            );
            let (expected_released, expected_delta) =
                e156_closed_form_expected(&existing_leaves, &touched, true);
            let record_count = allocator.records().len();
            let delta =
                u64::try_from(record_count - records_before).expect("这一步的记录增量落在 u64 内");
            assert_eq!(
                released, expected_released,
                "U10：第 {step} 次覆盖写释放槽数应等于闭式"
            );
            assert_eq!(
                delta, expected_delta,
                "U10：第 {step} 次覆盖写记录增量应等于闭式"
            );
            if step <= 4 {
                assert_eq!(released, 14, "R-4：第 {step} 次覆盖写 D0 释放槽数应为 14");
                assert_eq!(delta, 24, "R-4：第 {step} 次覆盖写记录增量应为 24");
            }
            release_samples.push(released);
            if touched.len() <= 1 {
                saw_single_leaf_step = true;
            } else {
                saw_multi_leaf_step = true;
            }
        }
        assert!(saw_single_leaf_step, "U10：应当至少有一步只落在 1 片叶");
        assert!(
            saw_multi_leaf_step,
            "U10：应当至少有一步跨到 ≥ 2 片叶（第八节 8.2）"
        );
        let (minimum, maximum) = super::e156_minimum_and_maximum(&release_samples);
        assert_eq!(minimum, 14, "U12：Q7d-1 覆盖写那一组的最小值应为 14");
        assert!(
            maximum >= 16,
            "U12：Q7d-1 覆盖写那一组的最大值应 ≥ 16（跨叶时更贵）"
        );
    }

    /// U11（第九节，R-6）：重放 S1(c) 场景（`run_s1c_rollback_isolation_scenario` 内部的
    /// `assert_eq!` 已经改成 54），只是从单测里再触发一次，M32（S1(c) 的隔离槽数退回 34）应当让这条
    /// 测试红。
    #[test]
    fn rollback_isolation_scenario_matches_the_new_layout() {
        let mut emitter = Emitter { emitted: 0 };
        run_s1c_rollback_isolation_scenario(&mut emitter, &parameters());
    }

    /// U13（E156 第 3 次重跑登记「十二」修订 4，岔路 7 Q7c①）：独立重放 HR 家族到管理员回退写行那一步
    /// （mkfs → 第一个事务 → 3N 次前缀覆盖写 → 回退到候选集里最旧的根，N = `ROOT_RING_REGIONS` ×
    /// `E156_SLOTS_PER_REGION`），不调用 `main()` 里那一段代码，只借同样的公开入口独立走一遍。
    /// 这一步应当落在 txg = 76、第 5 项恰为 0——此前 7 个可达基底（β0/β1/β2/βK-w/r/f/l2）第 5 项都
    /// ≥ 1，`q7c_self_test` 的①在它们身上恒不转色（见 `q7c_self_test` 的推导：`flip1` 要求
    /// `deferred == 0` 时 `without_subtracting_defer_is_red` 才会是 `false`）；这是第一次有一个
    /// **可达**基底能让①转色。M33/M34（`crates/mutations.tsv`）分别改坏 `q7c_self_test` 里
    /// `without_subtracting_defer_is_red` 与 `real_check_red_plus1` 的比较符号，应当让这条测试红。
    #[test]
    fn hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test() {
        let parameters = parameters();
        let (mut allocator, first, mut devices) = first_transaction_state();
        let instance = InstanceGeneration(1);
        let mut current = first;
        let ring_length = ROOT_RING_REGIONS * super::E156_SLOTS_PER_REGION;
        let prefix_length = 3 * ring_length;
        let mut actual_length: u64 = 0;
        for step in 1..=prefix_length {
            let outcome = {
                let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_overwrite(
                    &mut pool,
                    &mut allocator,
                    &current,
                    FirstFile {
                        content: &super::overwrite_content(step),
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 + step,
                    },
                    instance,
                )
            };
            match outcome {
                Ok(published) => {
                    current = published;
                    actual_length = step;
                }
                Err(failure) => panic!(
                    "U13：HR 前缀第 {step} 步写失败：{failure:?}（这一格依赖跑满 {prefix_length} 步才能到 txg=76）"
                ),
            }
        }
        assert_eq!(
            actual_length, prefix_length,
            "U13：HR 前缀应跑满 {prefix_length} 步，不应撞上分配记录树的任何上限"
        );

        let region_devices = parameters.region_devices;
        let effective_floor = effective_rollback_floor(
            &devices,
            &region_devices,
            &parameters.geometry,
            &parameters.filesystem_identifier,
        );
        let candidates = readable_roots(
            &devices,
            &region_devices,
            &parameters.geometry,
            &parameters.filesystem_identifier,
        );
        let oldest = candidates
            .iter()
            .filter(|root| root.checkpoint_txg.0 >= effective_floor.0)
            .min_by_key(|root| root.checkpoint_txg.0)
            .expect("U13：回退候选集至少一个根");
        let rollback_target = RollbackTarget {
            instance: oldest.instance,
            checkpoint_txg: oldest.checkpoint_txg,
        };
        let mounted = mount_rollback(&parameters, &mut devices, rollback_target, ShadowLedger::On)
            .expect("U13：HR 管理员回退");
        let row_publish = mounted
            .output
            .row_publish
            .file_version()
            .expect("U13：回退写行那一版带文件");
        assert_eq!(
            row_publish.root.checkpoint_txg.0, 76,
            "U13：回退写行那一版应落在 txg=76（这个装置今天实测到的取样点）"
        );

        let pool = memory_pool_snapshot(&devices);
        let basis = basis_of("test_beta_hr_rollback_row", &pool, row_publish, true);
        let (item1, _item2, item5) = mirror_accounting_row_slots(&basis.pool, basis.accounting_slot);
        assert_eq!(item5, 0, "U13：这一格的第 5 项应恰为 0（Q7c① 判别力自证的前提）");
        let check_is_red = item1 != item5 + basis.referenced;
        assert!(
            !check_is_red,
            "U13：G27 在这个基底上应当判绿——它是一个真实的合法状态，不是坏镜像"
        );

        let mut emitter = Emitter { emitted: 0 };
        let (flip1, flip2) = q7c_self_test(
            &mut emitter,
            "test_beta_hr_rollback_row",
            &basis.pool,
            basis.accounting_slot,
            basis.referenced,
        );
        assert!(
            flip1,
            "U13：在这个第 5 项恰为 0 的可达基底上，Q7c① 必须转色——去掉『减 defer』那一步之后 \
             corrupted 镜像的判定必须从红变绿，而 G27（减 defer 的真实检查）仍判红"
        );
        assert!(
            !flip2,
            "U13：Q7c② 在 deferred == 0 的基底上不适用（Bd(−1) 会让 item5 减成负数），不应转色"
        );
    }
}
