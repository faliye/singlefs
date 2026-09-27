//! E156（alloc-basis 四条岔路的代价数） 第 4 次重跑（登记 `research/prompts/e156-r4-prereg.md`）：回退改成挂着时的一次向前发布、
//! 挂载内回收、准入改式之后，岔路单第 1 行（记账第 1 项与 I-3.1（已分配统计对得上） 的形态，环上有洞怎么办）的代价数，
//! 连同问题单第一节三个前提在这个装置上的最小复现。
//!
//! - 第一段（登记第五节 5.5）：PQ1-前 / PQ1-后（前提 1：记录已持久、根槽没持久的崩溃之后可写挂载，环上出洞）与 PC-c2、
//!   PQ2（前提 2：抬 F 那一串里 F_生效 什么时候取到新值）、G-adm（前提 3 的间接影响）、U11 与 U13 的新形态（管理员回退是挂着时的
//!   一次向前发布）。第一段的停机条款（SP1、S1、S12）任一触发，第二段不跑。
//! - 第二段：Hh(k, 位置, S, ρ) 38 条排得下的历史，每条在「实」（真实分配器）与「每」（每次发布之前按谓词现算）两个回收时点上
//!   量甲-T1 与 G12 两条臂的多扣（Q1a、Q1b）与它们的差 Δ，按洞的计数分组判（Q1c、Q1c-洞、Q1c-前、Q1c-后、Q1d）。
//!
//! 只读驱动 `singlefs-core` / `singlefs-checker` 的公开入口，不改这两个 crate 的生产代码；产物按 `E7RESULT` 行打印，复跑登记在
//! `research/scripts/replay.sh`。洞的 txg、挂载做完的 txg、终点 txg 从本地 S 与装置里的锚点模型（[`anchor_model`]，登记第七节 7.2
//! 命令四 `anchors_e156_r4.py` 的逐行移植）算，不从 `crates/` 算；装置跑的时候从盘上现量，两边比。
//!
//! 判定写在装置里（[`judge_peak_against_threshold`] 那一组函数，单测 U18），不另写 Python 判定脚本：执行员的写范围不含
//! `research/scripts/` 与门禁 47 号，理由与改动记在登记第十二节。

use std::collections::{BTreeMap, BTreeSet, HashMap};

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemOutput,
    MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish,
    MountSpaceAdmission, RaiseToTheAdmissionCeiling, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, effective_rollback_floor,
    instance_table_chain_of_root, read_root_ring_slot, readable_roots,
    readable_roots_with_ring_slots, root_is_abandoned_by_the_instance_table, PoolReader,
    RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{
    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_version, warm_up, FirstFile,
    InstanceTablePlan, PoolWriter, PublishError, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice, SECTOR_BYTES};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// E156 装置的固定 fsid：与别的 `eNNN` 装置各自独立取一份，登记在这里方便复跑时核对。
const E156_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x35, 0x36, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const IMAGE_BYTES: u64 = 4 << 30;
/// 一个单元的槽数（跨度 2 的数据单元 / inode 叶容器 / 实例表单元，登记「一」读法写死）。
const INSTANCE_TABLE_SPAN_SLOTS: u64 = 2;
/// β0、U11、U13 那几段历史的每区槽数 S：**本地常量**，值抄自 `.claude/kb/layout/01-first-txn.md:138`「每区槽数 S」那一行
/// （`| 几何 | 每区槽数 S | 1 | 8 |`）。[`main`] 开头那条断言把它与实现侧 mkfs 默认回比。
const E156_SLOTS_PER_REGION: u64 = 8;
/// 岔路 1 的 S 三个取样点（第 4 次重跑登记表头第 1 行，全部都跑）：值抄自 `.claude/kb/decisions/22-单元原子性怎么合成.md:58`
/// 「每区槽数 S」那一行（下界 4、上界 16，「4..16 之间取值」）与 `.claude/kb/layout/01-first-txn.md:138`（mkfs 写 8）。
/// [`main`] 开头回比实现侧的上下界与 mkfs 默认；每格另核「系统配置里读回来的 S = 本地值」（K8）。
const E156_SLOTS_PER_REGION_SAMPLING_POINTS: [u64; 3] = [4, 8, 16];
const E156_SLOTS_PER_REGION_MINIMUM: u64 = 4;
const E156_SLOTS_PER_REGION_MAXIMUM: u64 = 16;
/// 区域设备（登记表头第 2 行，D22（单元原子性怎么合成） 已定项 1 的区域归属 [0, 1, 0]）；[`main`] 开头与 mkfs 参数回比。
const E156_REGION_DEVICE_NUMBERS: [u32; 3] = [0, 1, 0];
/// 区域数 R（登记表头第 2 行；`.claude/kb/decisions/22-单元原子性怎么合成.md:55`「区域数 R | 3」）：根环 3S 槽、U13 (a) 的 3N 次覆盖写都从它算；
/// [`main`] 开头与 `singlefs_format::ROOT_RING_REGIONS` 回比。
const E156_ROOT_RING_REGIONS: u64 = 3;
/// 判据门槛（登记表头第 3 行）：一个跨度 2 的单元 2 槽、跨度 1 的单元 1 槽（原登记「一」读法写死「一个单元的槽数」行；
/// D3（空间分配） 已定项 7 落点粒度一个 16 KiB 槽、跨度 1 或 2）。只住装置，没有实现侧的对应物，不回比。
const E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS: i64 = 2;
const E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS: i64 = 1;
/// A1：单元区起点槽号。抄自 D3（空间分配） 已定项 7 依据表「落点」行（`.claude/kb/decisions/03-空间分配.md`；
/// D18（块里携带什么信息） 已定项 9 逼出的粒度）与 `crates/singlefs-format` 的同名常量——[`main`] 第一条断言把它与
/// `UNIT_AREA_START_SLOT` 回比。
const E156_UNIT_AREA_START_SLOT: u64 = 50176;
/// A7：分配记录节点容量。抄自 `research/prompts/e156-r2-prereg.md` 表头「分配记录节点容量」行的算式
/// `⌊(16384 − 135) ÷ 20⌋`；节点头 135 字节 = 86（固定头）+ 2 × 10（子节点指针，容量算式的一部分）+ 29（其余头字段）。
const E156_ALLOCATION_RECORD_NODE_HEADER_BYTES: u64 = 86 + 2 * 10 + 29;
const E156_ALLOCATION_RECORD_BYTES: u64 = 20;
/// S1(f)：第一个事务之后的分配记录条数（E156 第 3 次重跑登记「七」7.2 K1-2：28 = 2 × 14）。
const E156_FIRST_TRANSACTION_EXPECTED_RECORD_COUNT: usize = 28;
/// K1-1（第 3 次重跑登记「七」7.2，第 4 次重跑登记第十三节命令三重跑逐行相同）：第一个事务之后 D0 记账第 1 项 17 槽、第 5 项 1 槽。
const E156_FIRST_TRANSACTION_ITEM1_SLOTS: u64 = 17;
const E156_FIRST_TRANSACTION_ITEM5_SLOTS: u64 = 1;
/// K1-1 的落点表（同一条命令三的输出 `first_txn = [...]`，按 (槽, 跨度) 升序）：数据 2、extent 1、inode 叶 2、inode 根 1、
/// 分配记录树 5 个节点各 1、记账 1、映射 1、树表 1。
const E156_FIRST_TRANSACTION_PLACEMENTS: [(u64, u64); 12] = [
    (50180, 2),
    (50240, 1),
    (50242, 2),
    (50244, 1),
    (50245, 1),
    (50246, 1),
    (50247, 1),
    (50248, 1),
    (50249, 1),
    (50250, 1),
    (50251, 1),
    (50252, 1),
];
/// R-4（第 3 次重跑登记「七」7.2，命令三重跑逐行相同）：Hh(0, —, S = 8, ρ = 1) 前 4 次覆盖写（环还没转过）每次 D0 释放 14 槽、
/// D0 + D1 记录 + 24。
const E156_OVERWRITES_CHECKED_BEFORE_THE_RING_WRAPS: u64 = 4;
const E156_OVERWRITE_RELEASED_SLOTS_BEFORE_THE_RING_WRAPS: u64 = 14;
const E156_OVERWRITE_RECORD_DELTA_BEFORE_THE_RING_WRAPS: u64 = 24;
/// R-3 本地常量①：分配记录树叶宽 W。**本地常量，值抄自 kb，不从 `crates/` 引**：抄自 `.claude/kb/decisions/08-核心索引结构.md:251`
/// `<!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->`。[`main`] 里 `anchor_a_d8` 那一行把它
/// 与 `singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS` 回比（只观测，F21：对不上不作废、不停机）。
const E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// R-3 本地常量②：分配记录树内部扇出 F。抄自 `.claude/kb/decisions/08-核心索引结构.md:254`
/// `<!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->`。
const E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// R-3：一次覆盖写里，非分配记录树自身的「其余单元」在 D0 上的释放槽数（E156 第 3 次重跑登记「七」7.2
/// R-3：数据 2 + extent 1 + inode 叶 2 + inode 根 1 + 记账 1 + 映射 1 + 树表 1 = 9）。
/// R-3 这一族闭式与它们的函数只剩单测 U10、U12 与 PC-闭式在用（第 4 次重跑的 `main` 不跑 H0），编成只在测试里有。
#[cfg(test)]
const E156_OVERWRITE_OTHER_UNITS_RELEASED_SLOTS: u64 = 9;
/// R-3：同一批「其余单元」里会产生新分配记录条目的槽数（不含数据：数据槽被 `data_slot()` 复用同一个
/// 已有的记录条目，不产生新条目，登记「七」7.2 R-4 的算术）。
#[cfg(test)]
const E156_OVERWRITE_OTHER_UNITS_NEW_RECORD_SLOTS: u64 = 7;
/// R-3：一次空发布里，非分配记录树自身的「其余单元」（记账 1 + 映射 1 + 树表 1 = 3，三者都走 bump、
/// 每次都是新槽，释放与新增同值，登记「七」7.2 R-5）。
#[cfg(test)]
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

/// 岔路 1 的 S 这一维：除 `root_ring_slots_per_region` 外与 [`parameters`] 逐字段相同——S 走 mkfs 参数，不必在仓副本里改常量重编。
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

fn new_pool_devices() -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect()
}

// ============================================================================================
// R-3（E156 第 3 次重跑登记「七」7.2）：分配记录树按位置寻址之后，节点数不再是常数，只能从「这次发布
// 实际改动的记录的槽号」现算。这一段只用 [`E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS`] /
// [`E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`] 两个本地常量与 `allocator.records()` 的真实读数，
// 不调用 `crates/singlefs-core::allocation_record_tree` 的任何几何函数——那些函数就是被测的实装本身，
// 拿它们来算「期望值」会让期望值与实装共用同一处错误。
// ============================================================================================

/// 一个槽所在的分配记录树叶位置 ⌊s ÷ W⌋（本地常量 W）。
#[cfg(test)]
fn e156_leaf_position_of_slot(slot: u64) -> u64 {
    slot / E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
}

/// 一个叶位置所在的层级 1 位置 ⌊k ÷ F⌋（本地常量 F）。
#[cfg(test)]
fn e156_level1_position_of_leaf(leaf: u64) -> u64 {
    leaf / E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
}

/// R-3：一次发布里分配记录树自身新写的节点数 = `devices × 叶数 + devices × 层级 1 数 + 1`（根只有一份，
/// 两块盘共享；叶与层级 1 逐盘各一份，登记「七」7.2 R-3 逐字）。
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
fn e156_existing_leaf_positions(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
) -> BTreeSet<u64> {
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
#[cfg(test)]
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
#[cfg(test)]
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

/// Q7d-1（R2 ⑤）：一组「这次发布自己的释放」样本的 (最小值, 最大值)——U12 单测用它，M30（Q7d-1 覆盖写那一组退回字面 10）改这里。
#[cfg(test)]
fn e156_minimum_and_maximum(samples: &[u64]) -> (u64, u64) {
    (
        samples.iter().min().copied().unwrap_or(0),
        samples.iter().max().copied().unwrap_or(0),
    )
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

/// 一块盘的记账第 1/2/5 项，槽数：直接读真实 `PoolAllocator`（K1-1 与单测用）。
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
#[cfg(test)]
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

/// R3：G27 一律读镜像上的记账行（第 1/2/5 项），单位槽。记账单元是镜像对（w = 2），
/// `corrupt_device_zero_accounting` 只改盘 0 那一份，这里也只读盘 0，两处同一个读法。
fn mirror_accounting_row_slots(pool: &MemoryPool, accounting_slot: u64) -> (u64, u64, u64) {
    (
        read_accounting_entry_bytes_from_mirror(pool, accounting_slot, 1) / SLOT_BYTES,
        read_accounting_entry_bytes_from_mirror(pool, accounting_slot, 2) / SLOT_BYTES,
        read_accounting_entry_bytes_from_mirror(pool, accounting_slot, 5) / SLOT_BYTES,
    )
}

/// G27 正式判定（R3）：逐盘（只判盘 0，两盘镜像同值）`第 1 项 − 第 5 项 == 最新根走读引用`，一律读镜像字节。
/// 第 4 次重跑的 `main` 不量岔路 7（用户已定），只剩单测 U8 在用。
#[cfg(test)]
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

/// 根环全部槽的 (设备, 偏移)：按公开的 `root_ring` 几何现算，不写死字节偏移（β0 与 β_syn 那一段的 S = 8 池）。
fn root_ring_slots(region_devices: &[DeviceIdentity; 3], spacing: u32) -> Vec<(u32, u64)> {
    (0..E156_ROOT_RING_REGIONS)
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
/// `entries` 是这次要同时改的 (统计量, delta 字节) 列表。
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

/// 反方向：从一份镜像重建出可写的设备 Vec（c2 崩溃之后在切出来的镜像上可写挂载要这一条）。
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

/// 同一份镜像的一套可写副本，每块盘外面包一层共用 `stream` 的录制器（W6 录写切段、PQ2 录抬 F 那一串）。
fn recording_copies_of(
    image: &MemoryPool,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    devices_from_memory_pool(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect()
}

/// 两份镜像读回来不同的扇区：(盘, 扇区起点的字节偏移)，按盘、偏移升序。
fn sectors_read_back_differently(first: &MemoryPool, second: &MemoryPool) -> Vec<(u32, u64)> {
    let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
    let mut differing = Vec::new();
    for (identity, first_device) in &first.devices {
        let Some(second_device) = second.devices.get(identity) else {
            continue;
        };
        let written_sectors: BTreeSet<u64> = first_device
            .written_sectors_in(DeviceOffsetInBytes(0), first.device_size_in_bytes)
            .into_iter()
            .chain(
                second_device
                    .written_sectors_in(DeviceOffsetInBytes(0), second.device_size_in_bytes),
            )
            .collect();
        for sector in written_sectors {
            let offset = DeviceOffsetInBytes(sector * SECTOR_BYTES);
            if first_device.read(offset, sector_bytes) != second_device.read(offset, sector_bytes) {
                differing.push((identity.0, offset.0));
            }
        }
    }
    differing
}

/// 两份镜像读回来的字节是不是逐字节相同：两边写过的扇区取并，逐扇区读回比（稀疏盘上「写过一扇区全 0」与「没写过」读回相同，
/// 按读回的字节比，不按内部表示比）。
fn memory_pools_read_back_identically(first: &MemoryPool, second: &MemoryPool) -> bool {
    if !first.devices.keys().eq(second.devices.keys()) {
        return false;
    }
    first.devices.iter().all(|(identity, first_device)| {
        let second_device = &second.devices[identity];
        let written_sectors: BTreeSet<u64> = first_device
            .written_sectors_in(DeviceOffsetInBytes(0), first.device_size_in_bytes)
            .into_iter()
            .chain(
                second_device
                    .written_sectors_in(DeviceOffsetInBytes(0), second.device_size_in_bytes),
            )
            .collect();
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        written_sectors.iter().all(|sector| {
            let offset = DeviceOffsetInBytes(sector * SECTOR_BYTES);
            first_device.read(offset, sector_bytes) == second_device.read(offset, sector_bytes)
        })
    })
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
fn verdict_label(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "holds",
        InvariantVerdict::Violated(_) => "violated",
        InvariantVerdict::NotApplicable(_) => "not_applicable",
    }
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

/// 工作负载的一次发布是哪一种（骨架负载：段内第 k 次是 O ⟺ `k mod m == 0`，ρ = 1 ⇒ m = 1，ρ = 1/4 ⇒ m = 4，其余是 E）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WorkloadPublishKind {
    Overwrite,
    EmptyPublish,
}

impl WorkloadPublishKind {
    fn label(self) -> &'static str {
        match self {
            WorkloadPublishKind::Overwrite => "overwrite",
            WorkloadPublishKind::EmptyPublish => "empty_publish",
        }
    }
}

/// 工作负载的一次发布：O 走 `publish_overwrite`，E 走 [`publish_empty`]。`content_salt` 同时定内容与写入时刻，逐次不同。
fn publish_one_workload<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
    kind: WorkloadPublishKind,
    content_salt: u64,
) -> Result<TransactionOutput, PublishError> {
    match kind {
        WorkloadPublishKind::Overwrite => {
            let content = overwrite_content(content_salt);
            let mut pool = PoolWriter::new(parameters, devices);
            publish_overwrite(
                &mut pool,
                allocator,
                previous,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + content_salt,
                },
                instance,
            )
        }
        WorkloadPublishKind::EmptyPublish => {
            publish_empty(parameters, devices, allocator, previous, instance)
        }
    }
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

/// 发完再改回的两份对照镜像（第 4 次重跑登记 W6：只留作对照）。
struct RestoredAfterPublishing {
    /// 只把这次的根槽（512 字节，单设备不镜像）改回发布之前的旧内容：装置原有的造法。它自称与「在根槽那一次写之前截断」
    /// 逐字节相同，前提是根槽是那次发布最后一笔写；第 4 次重跑第一段量到这个前提不成立——实现在根槽 FUA 之后还轮换每块盘的
    /// 系统配置槽（`transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`，D16（发布语义） 已定项 7 的持久顺序）。
    root_slot_only: MemoryPool,
    /// 根槽与每块盘的系统配置两槽都改回发布之前（登记第十二节修订 2）：按实现的持久顺序根槽之后只有系统配置槽轮换，
    /// W6 录写切段镜像与它逐字节比（停机 S1(i)）——比的是「根槽之前的写，录写切段与真发一字不差，根槽之后除系统配置轮换外没有别的写」。
    root_slot_and_system_configuration: MemoryPool,
}

/// 系统配置每块盘几槽（`.claude/kb/decisions/22-单元原子性怎么合成.md:26`「系统配置每盘 2 槽按世代号 mod 2 轮换」）、一槽几字节
/// （同一文件第 60 行「系统配置槽宽 = 格式常量 4096」）：本地常量，[`emit_static_anchors`] 与 `singlefs_format` 同名常量回比。
const E156_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: u64 = 2;
const E156_SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;

/// 真发一次，再按 [`RestoredAfterPublishing`] 的两种办法改回。
#[allow(
    clippy::too_many_arguments,
    reason = "c2 崩溃要凑齐设备、分配器、上一版、发布种类、内容、实例、几何七类各自独立的参数"
)]
fn crash_before_root_persists(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    kind: WorkloadPublishKind,
    content_salt: u64,
    instance: InstanceGeneration,
    slots_per_region: RootRingSlotsPerRegion,
) -> Result<RestoredAfterPublishing, PublishError> {
    let before = memory_pool_snapshot(devices);
    let published = publish_one_workload(
        parameters,
        devices.as_mut_slice(),
        allocator,
        previous,
        instance,
        kind,
        content_salt,
    )?;
    let (root_device, root_offset) = root_slot_location(
        published.root.checkpoint_txg,
        &parameters.region_devices,
        parameters.geometry.fixed_structure_slot_spacing,
        slots_per_region,
    );
    let old_root_bytes = mp_read(&before, root_device, root_offset, 512);
    let mut root_slot_only = memory_pool_snapshot(devices);
    mp_write(
        &mut root_slot_only,
        root_device,
        root_offset,
        &old_root_bytes,
    );
    let mut root_slot_and_system_configuration = root_slot_only.clone();
    let slot_bytes = usize::try_from(E156_SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in before
        .devices
        .keys()
        .map(|identity| identity.0)
        .collect::<Vec<u32>>()
    {
        for slot_index in 0..E156_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing);
            let old_bytes = mp_read(&before, device, offset, slot_bytes);
            mp_write(
                &mut root_slot_and_system_configuration,
                device,
                offset,
                &old_bytes,
            );
        }
    }
    Ok(RestoredAfterPublishing {
        root_slot_only,
        root_slot_and_system_configuration,
    })
}

/// W6 切段点：根槽那一次写之前（登记写死），或 PC-c2 把根槽那一次写也留下。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CrashCutPoint {
    BeforeTheRootSlotWrite,
    AfterTheRootSlotWritePositiveControl,
}

/// 一次 c2 崩溃的两份镜像与这次发布的输出。
struct CrashedPublish {
    cut_image: MemoryPool,
    /// W6 的录写切段镜像与「发完把根槽与每块盘的系统配置两槽改回」的镜像读回来逐字节相同（停机 S1(i) 的对拍项，登记第十二节修订 2；
    /// PC-c2 那一格按构造不同）。
    cut_matches_the_restored_image: bool,
    /// 录写切段镜像与装置原有的「只改回根槽」镜像读回来不同的 (盘, 扇区起点字节偏移)：S1(i) 两边查时量出来的那一格，照报。
    sectors_differing_from_the_root_slot_only_image: Vec<(u32, u64)>,
    /// 写日志里根槽那一次写之后还有的步（`StepKind::name`）：「发完改回根槽」那一造法的前提是它为空。
    steps_after_the_root_slot_write: Vec<&'static str>,
    root_slot_writes_in_the_publish: usize,
    crashed_output: TransactionOutput,
    allocator_after_the_crashed_publish: PoolAllocator,
}

/// W6：录下那次发布的设备写，切在根槽那一次写之前（`crash.rs` 的写日志，`FixedGeometry::classify` 认根槽写），
/// 切出来的镜像就是「记录已持久、根槽没持久」的 c2 崩溃镜像。同一次发布另按 [`crash_before_root_persists`] 造一份对照镜像。
#[allow(
    clippy::too_many_arguments,
    reason = "c2 崩溃要凑齐设备、分配器、上一版、发布种类、内容、实例、切段点七类各自独立的参数"
)]
fn crash_publish_before_its_root_persists(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    allocator: &PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
    kind: WorkloadPublishKind,
    content_salt: u64,
    cut_point: CrashCutPoint,
) -> Result<CrashedPublish, PublishError> {
    let before = memory_pool_snapshot(devices);
    let stream = SharedStream::retaining_contents();
    let mut recording_devices = recording_copies_of(&before, &stream);
    let mut recording_allocator = allocator.clone();
    let crashed_output = publish_one_workload(
        parameters,
        recording_devices.as_mut_slice(),
        &mut recording_allocator,
        previous,
        instance,
        kind,
        content_salt,
    )?;
    let operations = stream.retained_operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let root_slot_write_indexes: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| {
            matches!(
                geometry.classify(&retained.operation),
                StepKind::RootRecordFua
            )
        })
        .map(|(index, _)| index)
        .collect();
    let root_slot_writes_in_the_publish = root_slot_write_indexes.len();
    let root_slot_write_index = *root_slot_write_indexes
        .first()
        .expect("一次发布恰有一次根槽写（单设备不镜像）");
    let kept_operations = match cut_point {
        CrashCutPoint::BeforeTheRootSlotWrite => root_slot_write_index,
        CrashCutPoint::AfterTheRootSlotWritePositiveControl => root_slot_write_index + 1,
    };
    let mut restore_devices = devices_from_memory_pool(&before);
    let mut cut_image = before;
    cut_image.apply(&operations[..kept_operations]);

    let mut restore_allocator = allocator.clone();
    let restored_image = crash_before_root_persists(
        parameters,
        &mut restore_devices,
        &mut restore_allocator,
        previous,
        kind,
        content_salt,
        instance,
        parameters.geometry.root_ring_slots_per_region,
    )?;
    let steps_after_the_root_slot_write: Vec<&'static str> = operations
        [root_slot_write_index + 1..]
        .iter()
        .map(|retained| geometry.classify(&retained.operation).name())
        .collect();
    Ok(CrashedPublish {
        cut_matches_the_restored_image: memory_pools_read_back_identically(
            &cut_image,
            &restored_image.root_slot_and_system_configuration,
        ),
        sectors_differing_from_the_root_slot_only_image: sectors_read_back_differently(
            &cut_image,
            &restored_image.root_slot_only,
        ),
        steps_after_the_root_slot_write,
        cut_image,
        root_slot_writes_in_the_publish,
        crashed_output,
        allocator_after_the_crashed_publish: recording_allocator,
    })
}

/// Q7c①②：判别力自证。②在 `deferred == 0` 的基底上（β_syn）无从执行（Bd(−1) 会把 item5 减成负数），记 `not_applicable=true`。
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
/// 分配代因此只能从 `before` 里找；账本里查不到的按 0（genesis）记，这是保守默认，不是量出来的数（产物逐格报查不到的条数）。
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

// ============================================================================================
// 锚点模型：第 4 次重跑登记第十三节命令四 `anchors_e156_r4.py`（第三版）的逐行移植。不 import 装置别的部分、不读 `crates/`，
// 只用脚本头那几句条款：根环 txg u 落槽 u mod 3S；可再分配 = 已释放 ∧ 释放代 ≤ 环里最旧有效根（Hh 里 F_生效 = 0）；
// 每一次发布（连空发布）都重写树表单元，mkfs 之后两次暖机是零单元发布；暖机直到两块盘上各有一条本实例的根；
// c2 之后新实例第一个 txg = max(环里最高, 记录最高) + 1。产物里 `name=anchor_dump` 那几行去掉前缀之后与 Python 脚本
// `--dump` 的 `DUMP` 行逐行比（报告里的命令）。
// ============================================================================================
mod anchor_model {
    use std::collections::{BTreeMap, BTreeSet};

    /// 区域设备 [0, 1, 0]（D22（单元原子性怎么合成） 已定项 1）。
    const REGION_DEVICE_NUMBERS: [u64; 3] = [0, 1, 0];
    /// 造洞之后隔几次工作负载发布再造下一个（登记表头第 4 行）。
    const WORKLOAD_PUBLISHES_BETWEEN_HOLES: u64 = 2;
    const FIRST_TRANSACTION_TXG: u64 = 3;
    /// 「前」位置第一个洞最早落的 txg（C380 逐字的 txg 5）。
    const FRONT_HOLE_EARLIEST_TXG: u64 = 5;
    const ZERO_UNIT_WARM_UP_TXGS: [u64; 2] = [1, 2];

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum AnchorPublishRole {
        MakeFilesystemWarmUpOrFirstTransaction,
        Workload,
        Crash,
        InstanceRow,
        WarmUpAfterRecovery,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct AnchorRow {
        pub txg: u64,
        pub floor: u64,
        pub tree_table_lower_bound: u64,
        pub interval_rule_tree_table_over_withheld: u64,
        pub missing_txgs: u64,
        pub live_holes: u64,
        pub live_holes_closed_form: u64,
        pub missing_txgs_closed_form: u64,
        pub is_observed: bool,
    }

    #[derive(Clone, Debug)]
    pub struct AnchorCell {
        pub rows: Vec<AnchorRow>,
        pub holes: Vec<u64>,
        pub recovery_ends: Vec<u64>,
        pub fits: bool,
        pub end: u64,
    }

    impl AnchorCell {
        pub fn observed_row(&self, txg: u64) -> Option<&AnchorRow> {
            self.rows
                .iter()
                .find(|row| row.txg == txg && row.is_observed)
        }
    }

    fn device_of_txg(txg: u64) -> u64 {
        REGION_DEVICE_NUMBERS[usize::try_from(txg % 3).expect("对 3 取模落在 usize 里")]
    }

    fn build_history(
        slots_per_region: u64,
        hole_count: u64,
        front: bool,
    ) -> (BTreeMap<u64, AnchorPublishRole>, Vec<u64>, Vec<u64>) {
        let ring_length = 3 * slots_per_region;
        let mut roles = BTreeMap::new();
        for setup_txg in 0..=FIRST_TRANSACTION_TXG {
            roles.insert(
                setup_txg,
                AnchorPublishRole::MakeFilesystemWarmUpOrFirstTransaction,
            );
        }
        let mut holes: Vec<u64> = Vec::new();
        let mut recovery_ends: Vec<u64> = Vec::new();
        let mut latest_txg = FIRST_TRANSACTION_TXG;
        let first_hole_at = if front {
            FRONT_HOLE_EARLIEST_TXG
        } else {
            ring_length + FIRST_TRANSACTION_TXG + 1
        };
        let mut workload_publishes_since_the_last_hole = 0u64;
        while u64::try_from(holes.len()).expect("洞数落在 u64 内") < hole_count {
            let gap_is_long_enough = holes.is_empty()
                || workload_publishes_since_the_last_hole >= WORKLOAD_PUBLISHES_BETWEEN_HOLES;
            if latest_txg + 1 >= first_hole_at && gap_is_long_enough {
                latest_txg += 1;
                roles.insert(latest_txg, AnchorPublishRole::Crash);
                holes.push(latest_txg);
                latest_txg += 1;
                roles.insert(latest_txg, AnchorPublishRole::InstanceRow);
                let mut covered_devices = BTreeSet::from([device_of_txg(latest_txg)]);
                while covered_devices.len() < 2 {
                    latest_txg += 1;
                    roles.insert(latest_txg, AnchorPublishRole::WarmUpAfterRecovery);
                    covered_devices.insert(device_of_txg(latest_txg));
                }
                recovery_ends.push(latest_txg);
                workload_publishes_since_the_last_hole = 0;
            } else {
                latest_txg += 1;
                roles.insert(latest_txg, AnchorPublishRole::Workload);
                workload_publishes_since_the_last_hole = if holes.is_empty() {
                    0
                } else {
                    workload_publishes_since_the_last_hole + 1
                };
            }
        }
        (roles, holes, recovery_ends)
    }

    /// 脚本的 `run(S, k, pos, end)`：`end_override` 给了就跑到它，没给就跑到最后一个洞 + 3S + 2。
    pub fn run(
        slots_per_region: u64,
        hole_count: u64,
        front: bool,
        end_override: Option<u64>,
    ) -> AnchorCell {
        let ring_length = 3 * slots_per_region;
        let (mut roles, holes, recovery_ends) = build_history(slots_per_region, hole_count, front);
        let last_txg = *roles.keys().next_back().expect("至少有 mkfs 那一条");
        let end = end_override
            .unwrap_or_else(|| holes.last().expect("没给终点的格至少有一个洞") + ring_length + 2);
        for padding_txg in last_txg + 1..=end {
            roles.insert(padding_txg, AnchorPublishRole::Workload);
        }
        let mut ring: BTreeMap<u64, u64> = BTreeMap::new();
        let mut tree_tables: BTreeMap<u64, Option<u64>> = BTreeMap::from([(0, None)]);
        let mut live_tree_table = 0u64;
        let mut rows = Vec::new();
        for txg in 0..=end {
            let role = roles[&txg];
            if !ZERO_UNIT_WARM_UP_TXGS.contains(&txg) && txg > 0 {
                tree_tables.insert(live_tree_table, Some(txg));
                tree_tables.insert(txg, None);
                live_tree_table = txg;
            }
            if role != AnchorPublishRole::Crash {
                ring.insert(txg % ring_length, txg);
            }
            if role == AnchorPublishRole::Crash || txg < FIRST_TRANSACTION_TXG {
                continue;
            }
            let oldest = *ring.values().min().expect("环里至少有 mkfs 那一条根");
            let floor = oldest;
            let roots: BTreeSet<u64> = ring.values().copied().collect();
            let is_referenced = |allocation: u64, release: Option<u64>| match release {
                None => true,
                Some(release) => roots
                    .iter()
                    .any(|root| allocation <= *root && *root < release),
            };
            let withheld: BTreeSet<u64> = tree_tables
                .iter()
                .filter(|(_, release)| match release {
                    None => true,
                    Some(release) => *release > floor,
                })
                .map(|(allocation, _)| *allocation)
                .collect();
            let referenced: BTreeSet<u64> = tree_tables
                .iter()
                .filter(|(allocation, release)| is_referenced(**allocation, **release))
                .map(|(allocation, _)| *allocation)
                .collect();
            let interval_rule_withheld = referenced.clone();
            let missing_txgs = (oldest..=txg)
                .filter(|candidate| !roots.contains(candidate))
                .count();
            let live_holes = holes
                .iter()
                .filter(|hole| {
                    oldest <= **hole
                        && **hole <= txg
                        && !roots.contains(hole)
                        && ring
                            .get(&(**hole % ring_length))
                            .is_none_or(|occupant| *occupant < **hole)
                })
                .count();
            let live_closed_form: Vec<u64> = holes
                .iter()
                .copied()
                .filter(|hole| {
                    *hole <= txg && txg < hole + ring_length - u64::from(*hole < ring_length)
                })
                .collect();
            let first_live_hole_after_one_ring = live_closed_form
                .iter()
                .copied()
                .filter(|hole| *hole >= ring_length)
                .min();
            let missing_txgs_closed_form = match first_live_hole_after_one_ring {
                Some(hole) => txg - hole + 1,
                None => u64::try_from(live_closed_form.len()).expect("洞数落在 u64 内"),
            };
            rows.push(AnchorRow {
                txg,
                floor,
                tree_table_lower_bound: u64::try_from(withheld.difference(&referenced).count())
                    .expect("落在 u64 内"),
                interval_rule_tree_table_over_withheld: u64::try_from(
                    interval_rule_withheld.difference(&referenced).count(),
                )
                .expect("落在 u64 内"),
                missing_txgs: u64::try_from(missing_txgs).expect("落在 u64 内"),
                live_holes: u64::try_from(live_holes).expect("落在 u64 内"),
                live_holes_closed_form: u64::try_from(live_closed_form.len()).expect("落在 u64 内"),
                missing_txgs_closed_form,
                is_observed: txg == FIRST_TRANSACTION_TXG
                    || role == AnchorPublishRole::Workload
                    || recovery_ends.contains(&txg),
            });
        }
        let pairs_do_not_collide = holes.iter().enumerate().all(|(index, first)| {
            holes[index + 1..].iter().all(|second| {
                !(first % ring_length == second % ring_length
                    && first.abs_diff(*second) < ring_length)
            })
        });
        let no_hole_one_ring_before_another = holes
            .iter()
            .filter(|hole| **hole >= ring_length)
            .all(|hole| !holes.contains(&(hole - ring_length)));
        AnchorCell {
            rows,
            holes,
            recovery_ends,
            fits: pairs_do_not_collide && no_hole_one_ring_before_another,
            end,
        }
    }

    /// k = 0 的终点：同一 S 下 k ∈ {1, 2, 4} × {前, 后} 各族终点的最大值（脚本的 `k0_end`，排不下的格也算进去）。
    pub fn without_holes_end(slots_per_region: u64) -> u64 {
        [false, true]
            .iter()
            .flat_map(|front| {
                [1u64, 2, 4].map(|hole_count| run(slots_per_region, hole_count, *front, None).end)
            })
            .max()
            .expect("六个格")
    }

    /// 脚本的 `a10`：每个树表单元第一次能被分配的那次发布的 txg − 它的释放 txg。
    pub fn tree_table_reclaim_lags(slots_per_region: u64, end: u64) -> BTreeSet<u64> {
        let ring_length = 3 * slots_per_region;
        let mut lags = BTreeSet::new();
        let mut ring: BTreeMap<u64, u64> = BTreeMap::new();
        let mut released: BTreeMap<u64, u64> = BTreeMap::new();
        let mut live_tree_table = 0u64;
        let mut seen: BTreeSet<(u64, u64)> = BTreeSet::new();
        for txg in 0..=end {
            if !ZERO_UNIT_WARM_UP_TXGS.contains(&txg) && txg > 0 {
                released.insert(live_tree_table, txg);
                live_tree_table = txg;
            }
            ring.insert(txg % ring_length, txg);
            let floor = *ring.values().min().expect("环里有根");
            for (allocation, release) in &released {
                if *release <= floor && seen.insert((*allocation, *release)) {
                    lags.insert(txg + 1 - release);
                }
            }
        }
        lags
    }

    /// 脚本 `--dump` 的那几行（`DUMP` 换成调用方加的前缀），格序照脚本：先按位置标签（`-` < `back` < `front`）、再按 k。
    pub fn dump_lines(slots_per_region: u64) -> Vec<String> {
        let without_holes = run(
            slots_per_region,
            0,
            false,
            Some(without_holes_end(slots_per_region)),
        );
        let mut cells: Vec<(u64, &'static str, AnchorCell)> = vec![(0, "-", without_holes)];
        for (front, label) in [(false, "back"), (true, "front")] {
            for hole_count in [1u64, 2, 4] {
                cells.push((
                    hole_count,
                    label,
                    run(slots_per_region, hole_count, front, None),
                ));
            }
        }
        let mut lines = Vec::new();
        for (hole_count, label, cell) in &cells {
            let fits_label = if cell.fits { "True" } else { "False" };
            for row in cell.rows.iter().filter(|row| row.is_observed) {
                lines.push(format!(
                    "S={slots_per_region} k={hole_count} pos={label} fits={fits_label} txg={} floor={} h_que={} h_dong={} q1a_tree_table_lower_bound={}",
                    row.txg, row.floor, row.missing_txgs, row.live_holes, row.tree_table_lower_bound
                ));
            }
        }
        lines
    }
}

// ============================================================================================
// 岔路 1 的族 Hh(k, 位置, S, ρ)：第 4 次重跑登记第五节 5.3、第一节 1.3 W1–W9。
// ============================================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HolePosition {
    Front,
    Back,
}

impl HolePosition {
    fn label(self) -> &'static str {
        match self {
            HolePosition::Front => "front",
            HolePosition::Back => "back",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WorkloadDensity {
    EveryPublishOverwrites,
    OneOverwriteInFourPublishes,
}

impl WorkloadDensity {
    fn label(self) -> &'static str {
        match self {
            WorkloadDensity::EveryPublishOverwrites => "1",
            WorkloadDensity::OneOverwriteInFourPublishes => "1/4",
        }
    }
    fn publishes_per_overwrite(self) -> u64 {
        match self {
            WorkloadDensity::EveryPublishOverwrites => 1,
            WorkloadDensity::OneOverwriteInFourPublishes => 4,
        }
    }
}

/// W7：分配器从哪来。产品路径（mkfs 同一个进程里装根环表，之后用每次可写挂载交回的）；PC-分配器换回不装根环表的那一种。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HhAllocatorSource {
    ProductPathWithRootRing,
    WithoutRootRingPositiveControl,
}

/// 不装根环表的分配器（PC-分配器）：`PoolAllocator::new` 加 mkfs 那两个单元，与装置原有的做法相同。
fn allocator_without_the_root_ring(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    genesis: &MakeFilesystemOutput,
) -> PoolAllocator {
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: genesis_slot_on_device_zero(&genesis.root.instance_table.locations),
            span: INSTANCE_TABLE_SPAN_SLOTS,
        },
        Placement {
            slot: genesis_slot_on_device_zero(&genesis.root.tree_table.locations),
            span: TransactionUnit::TreeTable.span_slots(),
        },
    );
    allocator
}

fn genesis_slot_on_device_zero(
    locations: &[singlefs_core::pointer::LocationEntry; 2],
) -> SlotNumber {
    locations
        .iter()
        .find(|location| location.device == DeviceIdentity(0))
        .map(|location| location.slot)
        .expect("两份位置条目里有一份在盘 0")
}

fn allocator_for_hh(
    source: HhAllocatorSource,
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    genesis: &MakeFilesystemOutput,
) -> PoolAllocator {
    match source {
        HhAllocatorSource::ProductPathWithRootRing => {
            allocator_after_make_filesystem(parameters, devices, genesis)
        }
        HhAllocatorSource::WithoutRootRingPositiveControl => {
            allocator_without_the_root_ring(devices, genesis)
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct HhCellShape {
    slots_per_region: u64,
    density: WorkloadDensity,
    hole_count: u64,
    position: Option<HolePosition>,
}

impl HhCellShape {
    fn position_label(&self) -> &'static str {
        self.position.map_or("-", HolePosition::label)
    }
    fn fields(&self) -> String {
        format!(
            "s={} rho={} k={} position={}",
            self.slots_per_region,
            self.density.label(),
            self.hole_count,
            self.position_label()
        )
    }
}

/// 一个 txg 的那一版在装置这边留下的事实（W9 的独立路径「装置自己为那个 txg 留的发布输出」）：它引用的槽（两盘同槽）与树表落点。
#[derive(Clone, Debug)]
struct PublishedVersionFacts {
    referenced_slots: BTreeSet<u64>,
    tree_table_slot: u64,
}

fn facts_of_published_version(output: &TransactionOutput) -> PublishedVersionFacts {
    let mut referenced_slots = BTreeSet::new();
    let mut has_instance_table = false;
    for unit in &output.units {
        if matches!(unit.identity, TransactionUnit::InstanceTable) {
            has_instance_table = true;
        }
        for slot in unit.slot.0..unit.slot.0 + unit.identity.span_slots() {
            referenced_slots.insert(slot);
        }
    }
    if !has_instance_table {
        let instance_table_slot =
            genesis_slot_on_device_zero(&output.root.instance_table.locations).0;
        for slot in instance_table_slot..instance_table_slot + INSTANCE_TABLE_SPAN_SLOTS {
            referenced_slots.insert(slot);
        }
    }
    PublishedVersionFacts {
        referenced_slots,
        tree_table_slot: output.unit(TransactionUnit::TreeTable).slot.0,
    }
}

/// 零单元那几版（mkfs 的第 0 代与之后两次暖机）：引用的是根记录两条指针直接指着的实例表与第 0 版树表。
fn facts_of_zero_unit_version(root: &RootRecord) -> PublishedVersionFacts {
    let tree_table_slot = genesis_slot_on_device_zero(&root.tree_table.locations).0;
    let instance_table_slot = genesis_slot_on_device_zero(&root.instance_table.locations).0;
    let mut referenced_slots: BTreeSet<u64> =
        (instance_table_slot..instance_table_slot + INSTANCE_TABLE_SPAN_SLOTS).collect();
    for slot in tree_table_slot..tree_table_slot + TransactionUnit::TreeTable.span_slots() {
        referenced_slots.insert(slot);
    }
    PublishedVersionFacts {
        referenced_slots,
        tree_table_slot,
    }
}

/// W9 的盘上路径：同一条根，按 `recovery::allocation_records_under_root` 读出的仍分配记录，展开成这块盘上的槽。
/// 没有分配记录树的那几版（`allocation_records_under_root` 对第 0 代树表给空）按根记录里实例表与树表两条指针的这块盘那一份算。
#[allow(
    clippy::ptr_arg,
    reason = "`allocation_records_under_root` 收 `&dyn PoolReader`，切片没有大小不能转成它，要传 Vec 本身"
)]
fn referenced_slots_read_from_disk(
    devices: &Vec<(DeviceIdentity, SparseBlockDevice)>,
    root: &RootRecord,
    device: DeviceIdentity,
) -> Result<BTreeSet<u64>, String> {
    let records =
        allocation_records_under_root(devices, root).map_err(|failure| format!("{failure:?}"))?;
    if records.is_empty() {
        let mut slots = BTreeSet::new();
        for (locations, span) in [
            (&root.instance_table.locations, INSTANCE_TABLE_SPAN_SLOTS),
            (
                &root.tree_table.locations,
                TransactionUnit::TreeTable.span_slots(),
            ),
        ] {
            for location in locations
                .iter()
                .filter(|location| location.device == device)
            {
                slots.extend(location.slot.0..location.slot.0 + span);
            }
        }
        return Ok(slots);
    }
    Ok(records
        .iter()
        .filter(|record| record.device == device && !record.is_released)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect())
}

/// 回收门槛 `max(F_生效, 环里最旧有效根)`，从镜像的根环读（D16（发布语义） 已定项 1「可再分配」「环里最旧有效根」两行）。
/// 一条有效根都没有时按 0 算（Hh 里不出现）。
fn reclaim_threshold_read_from_the_ring_image(
    effective_floor: u64,
    valid_root_txgs: &BTreeSet<u64>,
) -> u64 {
    valid_root_txgs
        .iter()
        .min()
        .copied()
        .unwrap_or(0)
        .max(effective_floor)
}

/// h_缺：[环里最旧有效根.txg, 最新持久根.txg] 里环里没有自证合法根的 txg 个数（原登记「一」第 2 行字面）。
fn missing_txg_count(
    oldest_valid_root_txg: u64,
    newest_root_txg: u64,
    root_txgs: &BTreeSet<u64>,
) -> u64 {
    let missing = (oldest_valid_root_txg..=newest_root_txg)
        .filter(|candidate| !root_txgs.contains(candidate))
        .count();
    u64::try_from(missing).expect("落在 u64 内")
}

/// h_洞（r2 R2 第二读法）：注入过的洞里，落在 [环里最旧有效根, 最新根] 之间、环里没有它的根、且它的根环槽还没被更新的根盖掉的个数。
fn live_hole_count(
    injected_holes: &[u64],
    oldest_valid_root_txg: u64,
    newest_root_txg: u64,
    root_txgs: &BTreeSet<u64>,
    slot_occupant_txg: impl Fn(u64) -> Option<u64>,
) -> u64 {
    let live = injected_holes
        .iter()
        .copied()
        .filter(|hole| {
            let slot_not_overwritten_by_a_newer_root =
                slot_occupant_txg(*hole).is_none_or(|occupant| occupant < *hole);
            oldest_valid_root_txg <= *hole
                && *hole <= newest_root_txg
                && !root_txgs.contains(hole)
                && slot_not_overwritten_by_a_newer_root
        })
        .count();
    u64::try_from(live).expect("落在 u64 内")
}

/// G12 的谓词（原登记 5.3 第一张表）：已释放的记录只要环里有一条算数的根的 txg 落在它的寿命 [分配代, 释放代) 里就扣住。
fn lifetime_interval_rule_withholds(
    allocation_generation: u64,
    release_generation: u64,
    counted_root_txgs: &BTreeSet<u64>,
) -> bool {
    counted_root_txgs
        .iter()
        .any(|root_txg| *root_txg >= allocation_generation && *root_txg < release_generation)
}

fn expanded_slots<'records>(
    records: impl Iterator<Item = &'records AllocationRecord>,
) -> BTreeSet<u64> {
    records
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

fn size_of_difference(first: &BTreeSet<u64>, second: &BTreeSet<u64>) -> u64 {
    u64::try_from(first.difference(second).count()).expect("落在 u64 内")
}

/// 一块盘上两条臂、两个回收时点的「不发」与多扣（Q1a、Q1b 与 Q1a⁺、Q1b⁺；登记第六节 6.2，r2 Q1a / Q1b 字面）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeviceOverWithheld {
    floor_rule_real: u64,
    floor_rule_every_publish: u64,
    interval_rule_real: u64,
    interval_rule_every_publish: u64,
    floor_rule_real_referenced_but_free: u64,
    floor_rule_every_publish_referenced_but_free: u64,
    interval_rule_referenced_but_free: u64,
    no_reclaim_floor_rule_real: u64,
    no_reclaim_floor_rule_every_publish: u64,
    no_reclaim_interval_rule_real: u64,
    no_reclaim_interval_rule_every_publish: u64,
    reclaim_rule_violations: u64,
    released_records_without_allocation_generation: u64,
}

struct DeviceOverWithheldInputs<'context> {
    allocator: &'context PoolAllocator,
    device: DeviceIdentity,
    referenced: &'context BTreeSet<u64>,
    ring_threshold: u64,
    every_publish_floor: u64,
    interval_rule_root_txgs: &'context BTreeSet<u64>,
    allocation_generations: &'context HashMap<u64, u64>,
}

fn over_withheld_on_device(inputs: &DeviceOverWithheldInputs<'_>) -> DeviceOverWithheld {
    let free_map = inputs
        .allocator
        .devices
        .iter()
        .find(|map| map.device == inputs.device)
        .expect("这块盘在池里");
    let records: Vec<&AllocationRecord> = inputs
        .allocator
        .records()
        .iter()
        .filter(|record| record.device == inputs.device)
        .collect();
    let still_allocated =
        expanded_slots(records.iter().copied().filter(|record| !record.is_released));
    let released_not_reclaimed = expanded_slots(
        records
            .iter()
            .copied()
            .filter(|record| record.is_released && !free_map.is_free(record.slot)),
    );
    let every_publish_withheld_released =
        expanded_slots(records.iter().copied().filter(|record| {
            record.is_released && record.generation.0 > inputs.every_publish_floor
        }));
    let allocation_generation_of =
        |record: &AllocationRecord| inputs.allocation_generations.get(&record.slot.0).copied();
    let interval_rule_withheld_released =
        expanded_slots(records.iter().copied().filter(|record| {
            record.is_released
                && lifetime_interval_rule_withholds(
                    allocation_generation_of(record).unwrap_or(0),
                    record.generation.0,
                    inputs.interval_rule_root_txgs,
                )
        }));
    let every_released =
        expanded_slots(records.iter().copied().filter(|record| record.is_released));
    let union = |extra: &BTreeSet<u64>| -> BTreeSet<u64> {
        still_allocated.union(extra).copied().collect()
    };
    let floor_rule_real_withheld = union(&released_not_reclaimed);
    let floor_rule_every_publish_withheld = union(&every_publish_withheld_released);
    let interval_rule_withheld = union(&interval_rule_withheld_released);
    let no_reclaim_withheld = union(&every_released);
    let reclaim_rule_violations = records
        .iter()
        .filter(|record| record.is_released)
        .filter(|record| {
            let reclaimed = free_map.is_free(record.slot);
            (reclaimed && record.generation.0 > inputs.ring_threshold)
                || (!reclaimed && record.generation.0 <= inputs.ring_threshold)
        })
        .count();
    let released_records_without_allocation_generation = records
        .iter()
        .filter(|record| record.is_released && allocation_generation_of(record).is_none())
        .count();
    DeviceOverWithheld {
        floor_rule_real: size_of_difference(&floor_rule_real_withheld, inputs.referenced),
        floor_rule_every_publish: size_of_difference(
            &floor_rule_every_publish_withheld,
            inputs.referenced,
        ),
        interval_rule_real: size_of_difference(&interval_rule_withheld, inputs.referenced),
        interval_rule_every_publish: size_of_difference(&interval_rule_withheld, inputs.referenced),
        floor_rule_real_referenced_but_free: size_of_difference(
            inputs.referenced,
            &floor_rule_real_withheld,
        ),
        floor_rule_every_publish_referenced_but_free: size_of_difference(
            inputs.referenced,
            &floor_rule_every_publish_withheld,
        ),
        interval_rule_referenced_but_free: size_of_difference(
            inputs.referenced,
            &interval_rule_withheld,
        ),
        no_reclaim_floor_rule_real: size_of_difference(&no_reclaim_withheld, inputs.referenced),
        no_reclaim_floor_rule_every_publish: size_of_difference(
            &no_reclaim_withheld,
            inputs.referenced,
        ),
        no_reclaim_interval_rule_real: size_of_difference(&no_reclaim_withheld, inputs.referenced),
        no_reclaim_interval_rule_every_publish: size_of_difference(
            &no_reclaim_withheld,
            inputs.referenced,
        ),
        reclaim_rule_violations: u64::try_from(reclaim_rule_violations).expect("落在 u64 内"),
        released_records_without_allocation_generation: u64::try_from(
            released_records_without_allocation_generation,
        )
        .expect("落在 u64 内"),
    }
}

/// 一个观测点上从镜像与分配器读到的全部量（登记第五节 5.3「逐发布一行的列」）。
#[derive(Clone, Debug)]
struct HhStateReading {
    newest_root_txg: u64,
    missing_txgs: u64,
    live_holes: u64,
    ring_threshold: u64,
    every_publish_floor: u64,
    effective_floor: u64,
    abandoned_roots: u64,
    roots_without_version_facts: u64,
    device_zero: DeviceOverWithheld,
    devices_equal: bool,
    allocated_slots: u64,
    deferred_slots: u64,
    isolated_slots: u64,
    not_free_beyond_allocated: u64,
    root_ring_installed: bool,
}

struct HhStateInputs<'context> {
    parameters: &'context MakeFilesystemParameters,
    devices: &'context [(DeviceIdentity, SparseBlockDevice)],
    allocator: &'context PoolAllocator,
    versions: &'context BTreeMap<u64, PublishedVersionFacts>,
    allocation_generations: &'context HashMap<u64, u64>,
    injected_holes: &'context [u64],
}

fn read_hh_state(inputs: &HhStateInputs<'_>) -> HhStateReading {
    let geometry = &inputs.parameters.geometry;
    let region_devices = &inputs.parameters.region_devices;
    let filesystem_identifier = &inputs.parameters.filesystem_identifier;
    let ring = readable_roots_with_ring_slots(
        inputs.devices,
        region_devices,
        geometry,
        filesystem_identifier,
    );
    let ring_root_txgs: BTreeSet<u64> =
        ring.iter().map(|(_, root)| root.checkpoint_txg.0).collect();
    let newest_root = ring
        .iter()
        .map(|(_, root)| *root)
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .expect("环里至少有一条根");
    let newest_root_table = instance_table_chain_of_root(inputs.devices, &newest_root)
        .ok()
        .map(|chain| chain.records);
    let abandoned_root_txgs: BTreeSet<u64> = ring
        .iter()
        .filter(|(_, root)| {
            newest_root_table
                .as_ref()
                .is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table))
        })
        .map(|(_, root)| root.checkpoint_txg.0)
        .collect();
    let valid_root_txgs: BTreeSet<u64> = ring_root_txgs
        .difference(&abandoned_root_txgs)
        .copied()
        .collect();
    let effective_floor = effective_rollback_floor(
        inputs.devices,
        region_devices,
        geometry,
        filesystem_identifier,
    )
    .0;
    let newest_root_txg = newest_root.checkpoint_txg.0;
    let slots_per_region = geometry.root_ring_slots_per_region;
    let ring_threshold =
        reclaim_threshold_read_from_the_ring_image(effective_floor, &valid_root_txgs);
    let oldest_valid_root_txg = valid_root_txgs.iter().min().copied().unwrap_or(0);
    let missing_txgs = missing_txg_count(oldest_valid_root_txg, newest_root_txg, &ring_root_txgs);
    let occupant_by_ring_slot: BTreeMap<RootRingSlot, u64> = ring
        .iter()
        .map(|(ring_slot, root)| (*ring_slot, root.checkpoint_txg.0))
        .collect();
    let live_holes = live_hole_count(
        inputs.injected_holes,
        oldest_valid_root_txg,
        newest_root_txg,
        &ring_root_txgs,
        |hole| {
            occupant_by_ring_slot
                .get(&target_for_publish(CheckpointTxg(hole), slots_per_region))
                .copied()
        },
    );
    let every_publish_floor = ring_threshold;
    let interval_rule_root_txgs: BTreeSet<u64> = ring_root_txgs
        .iter()
        .copied()
        .filter(|txg| abandoned_root_txgs.contains(txg) || *txg >= effective_floor)
        .collect();
    let mut referenced = BTreeSet::new();
    let mut roots_without_version_facts = 0u64;
    for txg in &ring_root_txgs {
        match inputs.versions.get(txg) {
            Some(facts) => referenced.extend(facts.referenced_slots.iter().copied()),
            None => roots_without_version_facts += 1,
        }
    }
    let reading_on = |device: DeviceIdentity| {
        over_withheld_on_device(&DeviceOverWithheldInputs {
            allocator: inputs.allocator,
            device,
            referenced: &referenced,
            ring_threshold,
            every_publish_floor,
            interval_rule_root_txgs: &interval_rule_root_txgs,
            allocation_generations: inputs.allocation_generations,
        })
    };
    let device_zero = reading_on(DeviceIdentity(0));
    let device_one = reading_on(DeviceIdentity(1));
    let free_map_zero = inputs
        .allocator
        .devices
        .iter()
        .find(|map| map.device == DeviceIdentity(0))
        .expect("盘 0 在池里");
    let not_free_record_slots = expanded_slots(
        inputs
            .allocator
            .records()
            .iter()
            .filter(|record| record.device == DeviceIdentity(0)),
    )
    .into_iter()
    .filter(|slot| !free_map_zero.is_free(SlotNumber(*slot)))
    .count();
    HhStateReading {
        newest_root_txg,
        missing_txgs,
        live_holes,
        ring_threshold,
        every_publish_floor,
        effective_floor,
        abandoned_roots: u64::try_from(abandoned_root_txgs.len()).expect("落在 u64 内"),
        roots_without_version_facts,
        device_zero,
        devices_equal: device_zero == device_one,
        allocated_slots: free_map_zero.allocated_slots(),
        deferred_slots: free_map_zero.deferred_slots(),
        isolated_slots: free_map_zero.isolated_slots(),
        not_free_beyond_allocated: u64::try_from(not_free_record_slots)
            .expect("落在 u64 内")
            .saturating_sub(free_map_zero.allocated_slots()),
        root_ring_installed: inputs.allocator.root_ring_occupancy().is_some(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ObservationPoint {
    FirstTransaction,
    Workload(WorkloadPublishKind),
    MountReturn,
}

impl ObservationPoint {
    fn label(self) -> &'static str {
        match self {
            ObservationPoint::FirstTransaction => "first_transaction",
            ObservationPoint::Workload(kind) => kind.label(),
            ObservationPoint::MountReturn => "mount_return",
        }
    }
}

/// 逐观测点一行（产物 `name=q1_observation`）。
#[derive(Clone, Debug)]
struct HhObservation {
    txg: u64,
    point: ObservationPoint,
    workload_publishes: u64,
    holes_injected: u64,
    reading: HhStateReading,
    self_reclaimed_real: Option<u64>,
    anchor: Option<anchor_model::AnchorRow>,
}

impl HhObservation {
    fn over_withheld(&self, arm: Arm, timing: ReclaimTiming) -> u64 {
        let device_zero = &self.reading.device_zero;
        match (arm, timing) {
            (Arm::FloorRule, ReclaimTiming::Real) => device_zero.floor_rule_real,
            (Arm::FloorRule, ReclaimTiming::EveryPublish) => device_zero.floor_rule_every_publish,
            (Arm::LifetimeIntervalRule, ReclaimTiming::Real) => device_zero.interval_rule_real,
            (Arm::LifetimeIntervalRule, ReclaimTiming::EveryPublish) => {
                device_zero.interval_rule_every_publish
            }
        }
    }
    fn delta(&self, timing: ReclaimTiming) -> i64 {
        i64::try_from(self.over_withheld(Arm::FloorRule, timing)).expect("落在 i64 内")
            - i64::try_from(self.over_withheld(Arm::LifetimeIntervalRule, timing))
                .expect("落在 i64 内")
    }
}

/// 两条臂：甲-T1（回收按门槛 max(F_生效, 环里最旧有效根)，今天的实现）与 G12（按寿命区间）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    FloorRule,
    LifetimeIntervalRule,
}

/// 两个回收时点（W1、W2）。
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
enum ReclaimTiming {
    Real,
    EveryPublish,
}

impl ReclaimTiming {
    fn label(self) -> &'static str {
        match self {
            ReclaimTiming::Real => "real",
            ReclaimTiming::EveryPublish => "every_publish",
        }
    }
}

/// 一次 c2 之后那一次可写挂载的报告（PQ1 的 ①–④、K14 / S1(k)、G-adm）。
#[derive(Clone, Debug)]
struct HoleRecovery {
    hole_txg: u64,
    crashed_publish_kind: WorkloadPublishKind,
    prefix_applied: usize,
    first_txg_of_the_new_instance: u64,
    last_txg_of_the_mount: u64,
    publishes_in_the_mount: u64,
    ring_has_a_root_at_the_hole_txg: bool,
    hole_slot_content: String,
    cut_matches_the_restored_image: bool,
    sectors_differing_from_the_root_slot_only_image: Vec<(u32, u64)>,
    steps_after_the_root_slot_write: Vec<&'static str>,
    root_slot_writes_in_the_crashed_publish: usize,
    mount_admission: &'static str,
    mount_floor_raise_sequences: u64,
}

#[derive(Clone, Debug)]
struct HhRunOutcome {
    family: &'static str,
    shape: HhCellShape,
    allocator_source: HhAllocatorSource,
    cut_point: CrashCutPoint,
    registered_end: u64,
    actual_end: u64,
    truncation: Option<(u64, String)>,
    observations: Vec<HhObservation>,
    recoveries: Vec<HoleRecovery>,
    genesis_root_in_ring: bool,
    slots_per_region_read_back: u64,
    first_transaction_matches_the_registered_anchor: bool,
    ring_wrap_overwrites_checked: u64,
    ring_wrap_overwrite_mismatches: u64,
    tree_table_release_checks: u64,
    tree_table_release_violations: u64,
    mount_schedule_mismatches: u64,
    disk_cross_checked_roots: u64,
    disk_cross_check_mismatches: u64,
    disk_cross_check_failures: Vec<String>,
    admission_refusals: u64,
    tree_table_lags_real: BTreeSet<u64>,
    tree_table_lags_every_publish: BTreeSet<u64>,
    checker_red_on_hole_states: u64,
    checker_judged_hole_states: u64,
}

/// 一格要怎么跑（第 4 次重跑登记第五节 5.3、5.4）。
struct HhRunRequest<'anchor> {
    family: &'static str,
    shape: HhCellShape,
    allocator_source: HhAllocatorSource,
    cut_point: CrashCutPoint,
    anchor: &'anchor anchor_model::AnchorCell,
    check_overwrites_before_the_ring_wraps: bool,
    judge_checker_on_hole_states: bool,
    emit_positive_control_no_reclaim: bool,
}

/// 这一格挂在第 4 次重跑登记第十一节停机 S1(k) 的 K13：每一版换下上一版的树表单元，释放代 = 这一版的 txg。
struct TreeTableLineage {
    previous_tree_table_slot: u64,
    pending_release_checks: Vec<(u64, u64)>,
    releases: Vec<(u64, u64)>,
}

impl TreeTableLineage {
    fn note_version(
        &mut self,
        versions: &mut BTreeMap<u64, PublishedVersionFacts>,
        output: &TransactionOutput,
    ) {
        let facts = facts_of_published_version(output);
        let txg = output.root.checkpoint_txg.0;
        self.pending_release_checks
            .push((self.previous_tree_table_slot, txg));
        self.releases.push((self.previous_tree_table_slot, txg));
        self.previous_tree_table_slot = facts.tree_table_slot;
        versions.insert(txg, facts);
    }
}

fn describe_the_root_ring_slot_of_txg(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    parameters: &MakeFilesystemParameters,
    txg: u64,
) -> String {
    let ring_slot = target_for_publish(
        CheckpointTxg(txg),
        parameters.geometry.root_ring_slots_per_region,
    );
    match read_root_ring_slot(
        devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
        ring_slot,
    ) {
        RootRingSlotReading::SelfVerified(root) => format!("root_txg_{}", root.checkpoint_txg.0),
        RootRingSlotReading::Bad(_) => {
            let device = parameters.region_devices
                [usize::try_from(ring_slot.region).expect("区域号落在 3 以内")];
            let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
            let root_slot_bytes =
                usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
            match PoolReader::read(devices, device, offset, root_slot_bytes) {
                Some(bytes) if bytes.iter().all(|byte| *byte == 0) => "never_written".to_string(),
                Some(_) => "not_self_verified".to_string(),
                None => "unreadable".to_string(),
            }
        }
    }
}

fn mount_admission_label(admission: &MountSpaceAdmission) -> &'static str {
    match admission {
        MountSpaceAdmission::AdmittedBeforeAcquisition => "admitted_before_acquisition",
        MountSpaceAdmission::AdmittedAfterTheFloorRaises { .. } => {
            "admitted_after_the_floor_raises"
        }
        MountSpaceAdmission::StillShortAfterTheFloorRaises { .. } => {
            "still_short_after_the_floor_raises"
        }
        MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => "not_judged_by_the_test_only_switch",
    }
}

fn first_transaction_matches_the_registered_anchor(
    allocator: &PoolAllocator,
    first: &TransactionOutput,
) -> bool {
    let (allocated, _free, deferred) = accounting_row_slots(allocator, DeviceIdentity(0));
    let mut placements: Vec<(u64, u64)> = first
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.identity.span_slots()))
        .collect();
    placements.sort_unstable();
    first.root.checkpoint_txg.0 == 3
        && allocated == E156_FIRST_TRANSACTION_ITEM1_SLOTS
        && deferred == E156_FIRST_TRANSACTION_ITEM5_SLOTS
        && placements == E156_FIRST_TRANSACTION_PLACEMENTS
        && allocator.records().len() == E156_FIRST_TRANSACTION_EXPECTED_RECORD_COUNT
}

fn released_not_reclaimed_slots_on_device_zero(allocator: &PoolAllocator) -> BTreeSet<u64> {
    let free_map = allocator
        .devices
        .iter()
        .find(|map| map.device == DeviceIdentity(0))
        .expect("盘 0 在池里");
    expanded_slots(allocator.records().iter().filter(|record| {
        record.device == DeviceIdentity(0) && record.is_released && !free_map.is_free(record.slot)
    }))
}

/// 一格跑完之前的状态：设备、分配器、现行那一版与各本账。只给 [`run_hh_history`] 用。
struct HhRunState {
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    current: TransactionOutput,
    instance: InstanceGeneration,
    versions: BTreeMap<u64, PublishedVersionFacts>,
    allocation_generations: HashMap<u64, u64>,
    lineage: TreeTableLineage,
    injected_holes: Vec<u64>,
    workload_publishes: u64,
    tree_table_lag_seen_real: BTreeSet<(u64, u64)>,
    tree_table_lag_seen_every_publish: BTreeSet<(u64, u64)>,
}

impl HhRunState {
    /// 观测：读镜像与分配器、K13 与 A10 的记账、Q1e、与锚点那一行并排，打一行 `name=q1_observation`。
    fn observe(
        &mut self,
        emitter: &mut Emitter,
        request: &HhRunRequest<'_>,
        outcome: &mut HhRunOutcome,
        point: ObservationPoint,
        self_reclaimed_real: Option<u64>,
    ) {
        let reading = read_hh_state(&HhStateInputs {
            parameters: &self.parameters,
            devices: &self.devices,
            allocator: &self.allocator,
            versions: &self.versions,
            allocation_generations: &self.allocation_generations,
            injected_holes: &self.injected_holes,
        });
        let txg = self.current.root.checkpoint_txg.0;
        let free_map_zero = self
            .allocator
            .devices
            .iter()
            .find(|map| map.device == DeviceIdentity(0))
            .expect("盘 0 在池里");
        for (slot, expected_release) in std::mem::take(&mut self.lineage.pending_release_checks) {
            outcome.tree_table_release_checks += 1;
            let released_as_expected = self.allocator.records().iter().any(|record| {
                record.device == DeviceIdentity(0)
                    && record.slot.0 == slot
                    && record.is_released
                    && record.generation.0 == expected_release
            });
            if !released_as_expected {
                outcome.tree_table_release_violations += 1;
            }
        }
        for (slot, release) in self.lineage.releases.iter().copied() {
            if release > txg {
                continue;
            }
            if !self.tree_table_lag_seen_real.contains(&(slot, release))
                && free_map_zero.is_free(SlotNumber(slot))
            {
                self.tree_table_lag_seen_real.insert((slot, release));
                outcome.tree_table_lags_real.insert(txg + 1 - release);
            }
            if !self
                .tree_table_lag_seen_every_publish
                .contains(&(slot, release))
                && release <= reading.every_publish_floor
            {
                self.tree_table_lag_seen_every_publish
                    .insert((slot, release));
                outcome
                    .tree_table_lags_every_publish
                    .insert(txg + 1 - release);
            }
        }
        let checker_on_hole = if request.judge_checker_on_hole_states && reading.missing_txgs > 0 {
            let red = is_red(&verdict(&memory_pool_snapshot(&self.devices), "I-3.1"));
            outcome.checker_judged_hole_states += 1;
            if red {
                outcome.checker_red_on_hole_states += 1;
            }
            if red {
                "red"
            } else {
                "green"
            }
        } else {
            "not_judged"
        };
        let anchor = request.anchor.observed_row(txg).copied();
        let observation = HhObservation {
            txg,
            point,
            workload_publishes: self.workload_publishes,
            holes_injected: u64::try_from(self.injected_holes.len()).expect("落在 u64 内"),
            reading,
            self_reclaimed_real,
            anchor,
        };
        emit_observation(emitter, request, &observation, checker_on_hole);
        outcome.observations.push(observation);
    }

    /// W9：每条环里的根，装置自己那一份引用与盘上 `allocation_records_under_root` 读出的仍分配记录逐条比（两块盘各比一次）。
    fn cross_check_references_against_the_disk(&self, outcome: &mut HhRunOutcome) {
        let ring = readable_roots(
            &self.devices,
            &self.parameters.region_devices,
            &self.parameters.geometry,
            &self.parameters.filesystem_identifier,
        );
        for root in &ring {
            outcome.disk_cross_checked_roots += 1;
            let Some(facts) = self.versions.get(&root.checkpoint_txg.0) else {
                outcome.disk_cross_check_mismatches += 1;
                outcome
                    .disk_cross_check_failures
                    .push(format!("txg{}:no_version_facts", root.checkpoint_txg.0));
                continue;
            };
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                match referenced_slots_read_from_disk(&self.devices, root, device) {
                    Ok(slots) if slots == facts.referenced_slots => {}
                    Ok(slots) => {
                        outcome.disk_cross_check_mismatches += 1;
                        outcome.disk_cross_check_failures.push(format!(
                            "txg{}:device{}:disk_only={}:device_only={}",
                            root.checkpoint_txg.0,
                            device.0,
                            slots.difference(&facts.referenced_slots).count(),
                            facts.referenced_slots.difference(&slots).count()
                        ));
                    }
                    Err(failure) => {
                        outcome.disk_cross_check_mismatches += 1;
                        outcome.disk_cross_check_failures.push(format!(
                            "txg{}:device{}:unreadable:{}",
                            root.checkpoint_txg.0,
                            device.0,
                            failure.replace(' ', "_")
                        ));
                    }
                }
            }
        }
    }
}

fn emit_observation(
    emitter: &mut Emitter,
    request: &HhRunRequest<'_>,
    observation: &HhObservation,
    checker_on_hole: &str,
) {
    let reading = &observation.reading;
    let device_zero = &reading.device_zero;
    let anchor_fields = observation.anchor.map_or_else(
        || "anchor_row=missing".to_string(),
        |row| {
            format!(
                "anchor_floor={} anchor_h_que={} anchor_h_dong={} tree_table_lower_bound={}",
                row.floor, row.missing_txgs, row.live_holes, row.tree_table_lower_bound
            )
        },
    );
    let positive_control_fields = if request.emit_positive_control_no_reclaim {
        format!(
            " pc_no_reclaim_q1a_real={} pc_no_reclaim_q1a_every={} pc_no_reclaim_q1b_real={} pc_no_reclaim_q1b_every={}",
            device_zero.no_reclaim_floor_rule_real,
            device_zero.no_reclaim_floor_rule_every_publish,
            device_zero.no_reclaim_interval_rule_real,
            device_zero.no_reclaim_interval_rule_every_publish
        )
    } else {
        String::new()
    };
    emitter.emit(&format!(
        "name=q1_observation family={} {} txg={} point={} workload_publishes={} holes_injected={} h_que={} h_dong={} floor={} f_effective={} q1a_real={} q1a_every={} q1b_real={} q1b_every={} delta_real={} delta_every={} q1a_plus_real={} q1a_plus_every={} q1b_plus={} self_reclaimed_real={} allocated={} deferred={} isolated={} not_free_beyond_allocated={} devices_equal={} root_ring_installed={} k10_violations={} abandoned_roots={} roots_without_version_facts={} released_without_allocation_generation={} i31_on_hole={checker_on_hole} {anchor_fields}{positive_control_fields}",
        request.family,
        request.shape.fields(),
        observation.txg,
        observation.point.label(),
        observation.workload_publishes,
        observation.holes_injected,
        reading.missing_txgs,
        reading.live_holes,
        reading.ring_threshold,
        reading.effective_floor,
        device_zero.floor_rule_real,
        device_zero.floor_rule_every_publish,
        device_zero.interval_rule_real,
        device_zero.interval_rule_every_publish,
        observation.delta(ReclaimTiming::Real),
        observation.delta(ReclaimTiming::EveryPublish),
        device_zero.floor_rule_real_referenced_but_free,
        device_zero.floor_rule_every_publish_referenced_but_free,
        device_zero.interval_rule_referenced_but_free,
        observation
            .self_reclaimed_real
            .map_or_else(|| "not_a_publish".to_string(), |count| count.to_string()),
        reading.allocated_slots,
        reading.deferred_slots,
        reading.isolated_slots,
        reading.not_free_beyond_allocated,
        reading.devices_equal,
        reading.root_ring_installed,
        device_zero.reclaim_rule_violations,
        reading.abandoned_roots,
        reading.roots_without_version_facts,
        device_zero.released_records_without_allocation_generation,
    ));
}

/// 岔路 1 一格：mkfs（产品路径分配器）→ 取号 → 暖机 → 第一个事务（txg 3）→ 工作负载，在锚点模型给的洞 txg 上造 c2 并可写挂载，
/// 跑到锚点模型给的终点。任一次发布或挂载报错即截断（W8）。
#[allow(
    clippy::too_many_lines,
    reason = "一格的主历史（mkfs、第一个事务、工作负载、c2 与挂载、逐观测点）连着写才对得上登记 5.3 的骨架逐行"
)]
fn run_hh_history(emitter: &mut Emitter, request: &HhRunRequest<'_>) -> HhRunOutcome {
    let shape = request.shape;
    let parameters = parameters_with_slots_per_region(shape.slots_per_region);
    let mut outcome = HhRunOutcome {
        family: request.family,
        shape,
        allocator_source: request.allocator_source,
        cut_point: request.cut_point,
        registered_end: request.anchor.end,
        actual_end: 0,
        truncation: None,
        observations: Vec::new(),
        recoveries: Vec::new(),
        genesis_root_in_ring: false,
        slots_per_region_read_back: 0,
        first_transaction_matches_the_registered_anchor: false,
        ring_wrap_overwrites_checked: 0,
        ring_wrap_overwrite_mismatches: 0,
        tree_table_release_checks: 0,
        tree_table_release_violations: 0,
        mount_schedule_mismatches: 0,
        disk_cross_checked_roots: 0,
        disk_cross_check_mismatches: 0,
        disk_cross_check_failures: Vec::new(),
        admission_refusals: 0,
        tree_table_lags_real: BTreeSet::new(),
        tree_table_lags_every_publish: BTreeSet::new(),
        checker_red_on_hole_states: 0,
        checker_judged_hole_states: 0,
    };
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("Hh mkfs");
    outcome.genesis_root_in_ring = readable_roots(
        &devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .iter()
    .any(|root| root.checkpoint_txg.0 == 0);
    outcome.slots_per_region_read_back = choose_system_configuration(&devices)
        .map(|configuration| {
            configuration
                .immutable
                .sizes
                .root_ring_slots_per_region
                .count()
        })
        .unwrap_or(0);
    let mut allocator = allocator_for_hh(request.allocator_source, &parameters, &devices, &genesis);
    let genesis_facts = facts_of_zero_unit_version(&genesis.root);
    let mut versions: BTreeMap<u64, PublishedVersionFacts> = BTreeMap::new();
    for zero_unit_txg in 0..=2u64 {
        versions.insert(zero_unit_txg, genesis_facts.clone());
    }
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("Hh 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("Hh 暖机")
    };
    let mut allocation_generations: HashMap<u64, u64> = HashMap::new();
    let before_first = snapshot_allocated_generations(&allocator);
    let first = {
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
    };
    let first = match first {
        Ok(first) => first,
        Err(error) => {
            outcome.truncation = Some((3, format!("{error:?}")));
            emit_cell_summary(emitter, &outcome);
            return outcome;
        }
    };
    record_release_generations(&mut allocation_generations, &before_first, &allocator);
    outcome.first_transaction_matches_the_registered_anchor =
        first_transaction_matches_the_registered_anchor(&allocator, &first);
    let mut state = HhRunState {
        parameters,
        devices,
        allocator,
        current: first.clone(),
        instance,
        versions,
        allocation_generations,
        lineage: TreeTableLineage {
            previous_tree_table_slot: genesis_facts.tree_table_slot,
            pending_release_checks: Vec::new(),
            releases: Vec::new(),
        },
        injected_holes: Vec::new(),
        workload_publishes: 0,
        tree_table_lag_seen_real: BTreeSet::new(),
        tree_table_lag_seen_every_publish: BTreeSet::new(),
    };
    state.lineage.note_version(&mut state.versions, &first);
    state.observe(
        emitter,
        request,
        &mut outcome,
        ObservationPoint::FirstTransaction,
        None,
    );

    let publishes_per_overwrite = shape.density.publishes_per_overwrite();
    let mut workload_index_in_segment = 0u64;
    let mut content_salt = 0u64;
    let mut overwrites_before_the_ring_wraps = 0u64;
    while state.current.root.checkpoint_txg.0 < request.anchor.end {
        let next_txg = state.current.root.checkpoint_txg.0 + 1;
        let kind = if workload_index_in_segment.is_multiple_of(publishes_per_overwrite) {
            WorkloadPublishKind::Overwrite
        } else {
            WorkloadPublishKind::EmptyPublish
        };
        content_salt += 1;
        if request.anchor.holes.contains(&next_txg) {
            let before_crash = snapshot_allocated_generations(&state.allocator);
            let crashed = match crash_publish_before_its_root_persists(
                &state.parameters,
                &state.devices,
                &state.allocator,
                &state.current,
                state.instance,
                kind,
                content_salt,
                request.cut_point,
            ) {
                Ok(crashed) => crashed,
                Err(error) => {
                    if matches!(error, PublishError::SpaceAdmissionRefused(_)) {
                        outcome.admission_refusals += 1;
                    }
                    outcome.truncation = Some((next_txg, format!("{error:?}")));
                    break;
                }
            };
            record_release_generations(
                &mut state.allocation_generations,
                &before_crash,
                &crashed.allocator_after_the_crashed_publish,
            );
            state
                .lineage
                .note_version(&mut state.versions, &crashed.crashed_output);
            state.injected_holes.push(next_txg);
            let mut recovered_devices = devices_from_memory_pool(&crashed.cut_image);
            let mounted = match mount_writable(&state.parameters, &mut recovered_devices) {
                Ok(mounted) => mounted,
                Err(error) => {
                    outcome.truncation = Some((next_txg, format!("{error:?}")));
                    break;
                }
            };
            state.devices = recovered_devices;
            let mut mount_versions: Vec<TransactionOutput> = Vec::new();
            if let Some(row) = mounted.output.row_publish.file_version() {
                mount_versions.push(row.clone());
            }
            for warm in &mounted.output.warm_up_publishes {
                if let Some(version) = warm.file_version() {
                    mount_versions.push(version.clone());
                }
            }
            let floor_raise_sequences = mounted.output.space_admission.floor_raises();
            for raised in floor_raise_sequences {
                mount_versions.extend(raised.publishes.iter().cloned());
            }
            let mut before_mount =
                snapshot_allocated_generations(&crashed.allocator_after_the_crashed_publish);
            for version in &mount_versions {
                for record in version
                    .allocation_records
                    .iter()
                    .filter(|record| !record.is_released && record.device == DeviceIdentity(0))
                {
                    before_mount.insert(record.slot.0, record.generation.0);
                }
            }
            record_release_generations(
                &mut state.allocation_generations,
                &before_mount,
                &mounted.allocator,
            );
            for version in &mount_versions {
                state.lineage.note_version(&mut state.versions, version);
            }
            let first_txg_of_the_new_instance = mounted.output.row_publish.root().checkpoint_txg.0;
            let publishes_in_the_mount = 1
                + u64::try_from(mounted.output.warm_up_publishes.len()).expect("落在 u64 内")
                + floor_raise_sequences
                    .iter()
                    .map(|raised| u64::try_from(raised.publishes.len()).expect("落在 u64 内"))
                    .sum::<u64>();
            let mount_admission = mount_admission_label(&mounted.output.space_admission);
            let mount_floor_raise_sequences =
                u64::try_from(floor_raise_sequences.len()).expect("落在 u64 内");
            let prefix_applied = mounted.output.journal.prefix_applied;
            state.allocator = mounted.allocator;
            state.instance = mounted.output.instance;
            state.current = mounted
                .current
                .into_file_version()
                .expect("Hh c2 恢复之后现行版本带文件");
            let last_txg_of_the_mount = state.current.root.checkpoint_txg.0;
            let hole_index = state.injected_holes.len() - 1;
            let planned_end = request.anchor.recovery_ends.get(hole_index).copied();
            if first_txg_of_the_new_instance != next_txg + 1
                || planned_end != Some(last_txg_of_the_mount)
                || planned_end.map(|end| end - next_txg) != Some(publishes_in_the_mount)
            {
                outcome.mount_schedule_mismatches += 1;
            }
            let ring_has_a_root_at_the_hole_txg = readable_roots(
                &state.devices,
                &state.parameters.region_devices,
                &state.parameters.geometry,
                &state.parameters.filesystem_identifier,
            )
            .iter()
            .any(|root| root.checkpoint_txg.0 == next_txg);
            let recovery = HoleRecovery {
                hole_txg: next_txg,
                crashed_publish_kind: kind,
                prefix_applied,
                first_txg_of_the_new_instance,
                last_txg_of_the_mount,
                publishes_in_the_mount,
                ring_has_a_root_at_the_hole_txg,
                hole_slot_content: describe_the_root_ring_slot_of_txg(
                    &state.devices,
                    &state.parameters,
                    next_txg,
                ),
                cut_matches_the_restored_image: crashed.cut_matches_the_restored_image,
                sectors_differing_from_the_root_slot_only_image: crashed
                    .sectors_differing_from_the_root_slot_only_image
                    .clone(),
                steps_after_the_root_slot_write: crashed.steps_after_the_root_slot_write.clone(),
                root_slot_writes_in_the_crashed_publish: crashed.root_slot_writes_in_the_publish,
                mount_admission,
                mount_floor_raise_sequences,
            };
            emitter.emit(&format!(
                "name=hole_recovery family={} {} hole_txg={} crashed_publish={} prefix_applied={} first_txg_of_new_instance={} last_txg_of_mount={} publishes_in_mount={} planned_last_txg_of_mount={} ring_has_root_at_hole_txg={} hole_slot_content={} cut_point={:?} cut_matches_root_and_system_configuration_restored_image={} sectors_differing_from_root_slot_only_restored_image={} steps_after_root_slot_write={} root_slot_writes_in_crashed_publish={} mount_admission={} mount_floor_raise_sequences={}",
                request.family,
                shape.fields(),
                recovery.hole_txg,
                recovery.crashed_publish_kind.label(),
                recovery.prefix_applied,
                recovery.first_txg_of_the_new_instance,
                recovery.last_txg_of_the_mount,
                recovery.publishes_in_the_mount,
                planned_end.map_or_else(|| "none".to_string(), |end| end.to_string()),
                recovery.ring_has_a_root_at_the_hole_txg,
                recovery.hole_slot_content,
                request.cut_point,
                recovery.cut_matches_the_restored_image,
                if recovery.sectors_differing_from_the_root_slot_only_image.is_empty() {
                    "none".to_string()
                } else {
                    recovery
                        .sectors_differing_from_the_root_slot_only_image
                        .iter()
                        .map(|(device, offset)| format!("d{device}@{offset}"))
                        .collect::<Vec<_>>()
                        .join(",")
                },
                if recovery.steps_after_the_root_slot_write.is_empty() {
                    "none".to_string()
                } else {
                    recovery.steps_after_the_root_slot_write.join(",")
                },
                recovery.root_slot_writes_in_the_crashed_publish,
                recovery.mount_admission,
                recovery.mount_floor_raise_sequences,
            ));
            outcome.recoveries.push(recovery);
            workload_index_in_segment = 0;
            state.cross_check_references_against_the_disk(&mut outcome);
            state.observe(
                emitter,
                request,
                &mut outcome,
                ObservationPoint::MountReturn,
                None,
            );
        } else {
            let before_publish = snapshot_allocated_generations(&state.allocator);
            let released_not_reclaimed_before =
                released_not_reclaimed_slots_on_device_zero(&state.allocator);
            let records_before = state.allocator.records().len();
            let published = publish_one_workload(
                &state.parameters,
                state.devices.as_mut_slice(),
                &mut state.allocator,
                &state.current,
                state.instance,
                kind,
                content_salt,
            );
            let published = match published {
                Ok(published) => published,
                Err(error) => {
                    if matches!(error, PublishError::SpaceAdmissionRefused(_)) {
                        outcome.admission_refusals += 1;
                    }
                    outcome.truncation = Some((next_txg, format!("{error:?}")));
                    break;
                }
            };
            state.current = published;
            record_release_generations(
                &mut state.allocation_generations,
                &before_publish,
                &state.allocator,
            );
            let current = state.current.clone();
            state.lineage.note_version(&mut state.versions, &current);
            workload_index_in_segment += 1;
            state.workload_publishes += 1;
            if request.check_overwrites_before_the_ring_wraps
                && kind == WorkloadPublishKind::Overwrite
                && overwrites_before_the_ring_wraps < E156_OVERWRITES_CHECKED_BEFORE_THE_RING_WRAPS
            {
                overwrites_before_the_ring_wraps += 1;
                outcome.ring_wrap_overwrites_checked += 1;
                let released = self_release_slots_of_this_publish(
                    &state.allocator,
                    DeviceIdentity(0),
                    state.current.root.checkpoint_txg,
                );
                let record_delta = u64::try_from(state.allocator.records().len() - records_before)
                    .expect("这一步的记录增量落在 u64 内");
                if released != E156_OVERWRITE_RELEASED_SLOTS_BEFORE_THE_RING_WRAPS
                    || record_delta != E156_OVERWRITE_RECORD_DELTA_BEFORE_THE_RING_WRAPS
                {
                    outcome.ring_wrap_overwrite_mismatches += 1;
                }
                emitter.emit(&format!(
                    "name=ring_wrap_overwrite_check family={} {} txg={} released_d0={released} record_delta={record_delta} registered_released_d0={E156_OVERWRITE_RELEASED_SLOTS_BEFORE_THE_RING_WRAPS} registered_record_delta={E156_OVERWRITE_RECORD_DELTA_BEFORE_THE_RING_WRAPS}",
                    request.family,
                    shape.fields(),
                    state.current.root.checkpoint_txg.0
                ));
            }
            let free_map_zero = state
                .allocator
                .devices
                .iter()
                .find(|map| map.device == DeviceIdentity(0))
                .expect("盘 0 在池里");
            let self_reclaimed = released_not_reclaimed_before
                .iter()
                .filter(|slot| free_map_zero.is_free(SlotNumber(**slot)))
                .count();
            state.observe(
                emitter,
                request,
                &mut outcome,
                ObservationPoint::Workload(kind),
                Some(u64::try_from(self_reclaimed).expect("落在 u64 内")),
            );
        }
    }
    outcome.actual_end = state.current.root.checkpoint_txg.0;
    state.cross_check_references_against_the_disk(&mut outcome);
    emit_cell_summary(emitter, &outcome);
    outcome
}

fn set_text(values: &BTreeSet<u64>) -> String {
    let parts: Vec<String> = values.iter().map(ToString::to_string).collect();
    format!("[{}]", parts.join(","))
}

/// 轨迹（登记第八节 8.1）：峰值、环转过第一个事务那一圈之后的峰值、为正的观测点数、期末值。
fn trajectory_text<Value: Copy + Ord + Default + std::fmt::Display>(
    values: &[(u64, Value)],
    after_the_first_ring_txg: u64,
) -> String {
    let zero = Value::default();
    let peak = values.iter().map(|(_, value)| *value).max().unwrap_or(zero);
    let peak_after_ring = values
        .iter()
        .filter(|(txg, _)| *txg >= after_the_first_ring_txg)
        .map(|(_, value)| *value)
        .max();
    let positive = values.iter().filter(|(_, value)| *value > zero).count();
    let final_value = values.last().map(|(_, value)| *value);
    format!(
        "peak={peak} peak_after_first_ring={} positive_points={positive} final={}",
        peak_after_ring.map_or_else(|| "none".to_string(), |value| value.to_string()),
        final_value.map_or_else(|| "none".to_string(), |value| value.to_string())
    )
}

fn emit_cell_summary(emitter: &mut Emitter, outcome: &HhRunOutcome) {
    let shape = outcome.shape;
    let observations = &outcome.observations;
    let anchor_missing_txg_mismatches = observations
        .iter()
        .filter(|observation| {
            observation
                .anchor
                .is_some_and(|row| row.missing_txgs != observation.reading.missing_txgs)
        })
        .count();
    let anchor_live_hole_mismatches = observations
        .iter()
        .filter(|observation| {
            observation
                .anchor
                .is_some_and(|row| row.live_holes != observation.reading.live_holes)
        })
        .count();
    let anchor_floor_mismatches = observations
        .iter()
        .filter(|observation| {
            observation
                .anchor
                .is_some_and(|row| row.floor != observation.reading.ring_threshold)
        })
        .count();
    let anchor_rows_missing = observations
        .iter()
        .filter(|observation| observation.anchor.is_none())
        .count();
    let below_bound = |timing: ReclaimTiming| {
        observations
            .iter()
            .filter(|observation| {
                observation.anchor.is_some_and(|row| {
                    observation.over_withheld(Arm::FloorRule, timing) < row.tree_table_lower_bound
                })
            })
            .count()
    };
    let positive = |pick: &dyn Fn(&HhObservation) -> u64| {
        observations
            .iter()
            .filter(|observation| pick(observation) > 0)
            .count()
    };
    let (truncated, truncation_txg, truncation_error) = match &outcome.truncation {
        Some((txg, error)) => (true, txg.to_string(), error.replace(' ', "_")),
        None => (false, "none".to_string(), "none".to_string()),
    };
    let holes: BTreeSet<u64> = outcome
        .recoveries
        .iter()
        .map(|recovery| recovery.hole_txg)
        .collect();
    let recovery_ends: BTreeSet<u64> = outcome
        .recoveries
        .iter()
        .map(|recovery| recovery.last_txg_of_the_mount)
        .collect();
    emitter.emit(&format!(
        "name=q1_cell family={} {} allocator={:?} cut_point={:?} registered_end={} actual_end={} truncated={truncated} truncation_txg={truncation_txg} truncation_error={truncation_error} observations={} holes_injected={} recovery_ends={} genesis_root_in_ring={} slots_per_region_read_back={} first_transaction_matches_k1_1={} ring_wrap_overwrites_checked={} ring_wrap_overwrite_mismatches={} tree_table_release_checks={} tree_table_release_violations={} mount_schedule_mismatches={} cut_mismatches_restored_image={} w9_roots_compared={} w9_mismatches={} w9_failures={} a11_h_que_mismatches={anchor_missing_txg_mismatches} a11_h_dong_mismatches={anchor_live_hole_mismatches} a14_floor_mismatches={anchor_floor_mismatches} anchor_rows_missing={anchor_rows_missing} a15_real_below_bound={} a15_every_below_bound={} k10_violation_points={} devices_unequal_points={} root_ring_missing_points={} abandoned_root_points={} roots_without_version_facts_points={} admission_refusals={} mount_admission_not_before_acquisition={} mount_floor_raise_sequences={} effective_floor_positive_points={} q1b_real_positive_points={} q1b_every_positive_points={} q1a_plus_real_positive_points={} q1a_plus_every_positive_points={} q1b_plus_positive_points={} a10_lags_real={} a10_lags_every={} i31_red_on_hole_states={} i31_judged_hole_states={}",
        outcome.family,
        shape.fields(),
        outcome.allocator_source,
        outcome.cut_point,
        outcome.registered_end,
        outcome.actual_end,
        observations.len(),
        set_text(&holes),
        set_text(&recovery_ends),
        outcome.genesis_root_in_ring,
        outcome.slots_per_region_read_back,
        outcome.first_transaction_matches_the_registered_anchor,
        outcome.ring_wrap_overwrites_checked,
        outcome.ring_wrap_overwrite_mismatches,
        outcome.tree_table_release_checks,
        outcome.tree_table_release_violations,
        outcome.mount_schedule_mismatches,
        outcome
            .recoveries
            .iter()
            .filter(|recovery| !recovery.cut_matches_the_restored_image)
            .count(),
        outcome.disk_cross_checked_roots,
        outcome.disk_cross_check_mismatches,
        if outcome.disk_cross_check_failures.is_empty() {
            "none".to_string()
        } else {
            outcome.disk_cross_check_failures.join(",")
        },
        below_bound(ReclaimTiming::Real),
        below_bound(ReclaimTiming::EveryPublish),
        positive(&|observation| observation.reading.device_zero.reclaim_rule_violations),
        observations
            .iter()
            .filter(|observation| !observation.reading.devices_equal)
            .count(),
        observations
            .iter()
            .filter(|observation| !observation.reading.root_ring_installed)
            .count(),
        positive(&|observation| observation.reading.abandoned_roots),
        positive(&|observation| observation.reading.roots_without_version_facts),
        outcome.admission_refusals,
        outcome
            .recoveries
            .iter()
            .filter(|recovery| recovery.mount_admission != "admitted_before_acquisition")
            .count(),
        outcome
            .recoveries
            .iter()
            .map(|recovery| recovery.mount_floor_raise_sequences)
            .sum::<u64>(),
        positive(&|observation| observation.reading.effective_floor),
        positive(&|observation| observation.reading.device_zero.interval_rule_real),
        positive(&|observation| observation.reading.device_zero.interval_rule_every_publish),
        positive(&|observation| observation.reading.device_zero.floor_rule_real_referenced_but_free),
        positive(&|observation| {
            observation
                .reading
                .device_zero
                .floor_rule_every_publish_referenced_but_free
        }),
        positive(&|observation| observation.reading.device_zero.interval_rule_referenced_but_free),
        set_text(&outcome.tree_table_lags_real),
        set_text(&outcome.tree_table_lags_every_publish),
        outcome.checker_red_on_hole_states,
        outcome.checker_judged_hole_states,
    ));
    let after_the_first_ring_txg = E156_ROOT_RING_REGIONS * shape.slots_per_region + 3;
    let series_u64 = |pick: &dyn Fn(&HhObservation) -> u64| -> Vec<(u64, u64)> {
        observations
            .iter()
            .map(|observation| (observation.txg, pick(observation)))
            .collect()
    };
    let series_i64 = |timing: ReclaimTiming| -> Vec<(u64, i64)> {
        observations
            .iter()
            .map(|observation| (observation.txg, observation.delta(timing)))
            .collect()
    };
    let trajectories: [(&str, String); 10] = [
        (
            "q1a_real",
            trajectory_text(
                &series_u64(&|observation| {
                    observation.over_withheld(Arm::FloorRule, ReclaimTiming::Real)
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "q1a_every",
            trajectory_text(
                &series_u64(&|observation| {
                    observation.over_withheld(Arm::FloorRule, ReclaimTiming::EveryPublish)
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "q1b_real",
            trajectory_text(
                &series_u64(&|observation| {
                    observation.over_withheld(Arm::LifetimeIntervalRule, ReclaimTiming::Real)
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "q1b_every",
            trajectory_text(
                &series_u64(&|observation| {
                    observation
                        .over_withheld(Arm::LifetimeIntervalRule, ReclaimTiming::EveryPublish)
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "delta_real",
            trajectory_text(&series_i64(ReclaimTiming::Real), after_the_first_ring_txg),
        ),
        (
            "delta_every",
            trajectory_text(
                &series_i64(ReclaimTiming::EveryPublish),
                after_the_first_ring_txg,
            ),
        ),
        (
            "threshold_lag",
            trajectory_text(
                &series_u64(&|observation| {
                    observation.reading.newest_root_txg - observation.reading.ring_threshold
                }),
                after_the_first_ring_txg,
            ),
        ),
        (
            "h_que",
            trajectory_text(
                &series_u64(&|observation| observation.reading.missing_txgs),
                after_the_first_ring_txg,
            ),
        ),
        (
            "h_dong",
            trajectory_text(
                &series_u64(&|observation| observation.reading.live_holes),
                after_the_first_ring_txg,
            ),
        ),
        (
            "self_reclaimed_real",
            trajectory_text(
                &observations
                    .iter()
                    .filter_map(|observation| {
                        observation
                            .self_reclaimed_real
                            .map(|count| (observation.txg, count))
                    })
                    .collect::<Vec<_>>(),
                after_the_first_ring_txg,
            ),
        ),
    ];
    for (quantity, text) in trajectories {
        emitter.emit(&format!(
            "name=q1_trajectory family={} {} quantity={quantity} {text}",
            outcome.family,
            shape.fields()
        ));
    }
    let ring_length = E156_ROOT_RING_REGIONS * shape.slots_per_region;
    let lag_at_least_ring = observations
        .iter()
        .filter(|observation| {
            observation.reading.newest_root_txg - observation.reading.ring_threshold >= ring_length
        })
        .count();
    let self_reclaimed_zero = observations
        .iter()
        .filter(|observation| observation.self_reclaimed_real == Some(0))
        .count();
    emitter.emit(&format!(
        "name=q1_trajectory_extra family={} {} threshold_lag_at_least_3s_points={lag_at_least_ring} self_reclaimed_real_zero_points={self_reclaimed_zero}",
        outcome.family,
        shape.fields()
    ));
}

// ============================================================================================
// PQ2（前提 2）：抬 F 那一串的设备写，切在系统配置写完之后、每条推空根落盘之后，读 F_生效。
// ============================================================================================

struct FloorRaiseTrace {
    raised: bool,
    floor_before: u64,
    new_floor: u64,
    effective_floor_before_any_write: u64,
    system_configuration_writes: usize,
    effective_floor_after_the_last_system_configuration_write: Option<u64>,
    system_configuration_writes_after_the_first_root: usize,
    pushed_roots: Vec<(u64, u64, u64)>,
    publishes: usize,
    reclaimed_placements: usize,
}

impl FloorRaiseTrace {
    /// 前提 2 的判定（登记第六节 6.1 PQ2 那一行）：① = 新 F 且 ② 每一条两数相等（= 新 F）⇒「两个口径合成一个」。
    fn the_two_readings_merge(&self) -> bool {
        self.raised
            && self.effective_floor_after_the_last_system_configuration_write
                == Some(self.new_floor)
            && !self.pushed_roots.is_empty()
            && self.pushed_roots.iter().all(|(_, root_floor, effective)| {
                *root_floor == self.new_floor && *effective == self.new_floor
            })
    }
}

/// PQ2：S = 8、ρ = 1，β0 → 覆盖写 8 次 → 走产品入口 `raise_rollback_floor_to_the_admission_ceiling` 抬 F，录下这一串的设备写。
fn run_floor_raise_trace() -> FloorRaiseTrace {
    let parameters = parameters_with_slots_per_region(E156_SLOTS_PER_REGION);
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("PQ2 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("PQ2 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("PQ2 暖机")
    };
    let mut current = {
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
        .expect("PQ2 第一个事务")
    };
    for step in 1..=8u64 {
        current = publish_one_workload(
            &parameters,
            devices.as_mut_slice(),
            &mut allocator,
            &current,
            instance,
            WorkloadPublishKind::Overwrite,
            step,
        )
        .expect("PQ2 覆盖写");
    }
    let floor_before = current.root.rollback_floor.0;
    let before_image = memory_pool_snapshot(&devices);
    let effective_floor_before_any_write = effective_rollback_floor(
        &before_image,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .0;
    let stream = SharedStream::retaining_contents();
    let mut recording_devices = recording_copies_of(&before_image, &stream);
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &parameters,
        &mut recording_devices,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("PQ2 抬 F");
    let (was_raised, new_floor, publishes, reclaimed_placements) = match &raised {
        RaiseToTheAdmissionCeiling::Raised(raised) => (
            true,
            raised.ceiling.0,
            raised.publishes.len(),
            raised.reclaimed.len(),
        ),
        RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { ceiling, .. } => {
            (false, ceiling.0, 0, 0)
        }
    };
    let operations = stream.retained_operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let kinds: Vec<StepKind> = operations
        .iter()
        .map(|retained| geometry.classify(&retained.operation))
        .collect();
    let first_root_write = kinds
        .iter()
        .position(|kind| matches!(kind, StepKind::RootRecordFua));
    let system_configuration_indexes: Vec<usize> = kinds
        .iter()
        .enumerate()
        .filter(|(_, kind)| matches!(kind, StepKind::SystemConfigurationSlot))
        .map(|(index, _)| index)
        .collect();
    let last_system_configuration_before_the_first_root = system_configuration_indexes
        .iter()
        .copied()
        .filter(|index| first_root_write.is_none_or(|first| *index < first))
        .max();
    let system_configuration_writes_after_the_first_root = system_configuration_indexes
        .iter()
        .filter(|index| first_root_write.is_some_and(|first| **index > first))
        .count();
    let image_up_to = |last_index: usize| {
        let mut image = before_image.clone();
        image.apply(&operations[..=last_index]);
        image
    };
    let effective_after = |image: &MemoryPool| {
        effective_rollback_floor(
            image,
            &parameters.region_devices,
            &parameters.geometry,
            &parameters.filesystem_identifier,
        )
        .0
    };
    let effective_floor_after_the_last_system_configuration_write =
        last_system_configuration_before_the_first_root
            .map(|index| effective_after(&image_up_to(index)));
    let pushed_roots: Vec<(u64, u64, u64)> = kinds
        .iter()
        .enumerate()
        .filter(|(_, kind)| matches!(kind, StepKind::RootRecordFua))
        .map(|(index, _)| {
            let bytes = operations[index]
                .contents
                .as_ref()
                .expect("开了内容保留的流");
            let root = RootRecord::parse_slot(bytes, &parameters.filesystem_identifier)
                .expect("推空那条根自证过");
            (
                root.checkpoint_txg.0,
                root.rollback_floor.0,
                effective_after(&image_up_to(index)),
            )
        })
        .collect();
    FloorRaiseTrace {
        raised: was_raised,
        floor_before,
        new_floor,
        effective_floor_before_any_write,
        system_configuration_writes: system_configuration_indexes.len(),
        effective_floor_after_the_last_system_configuration_write,
        system_configuration_writes_after_the_first_root,
        pushed_roots,
        publishes,
        reclaimed_placements,
    }
}

// ============================================================================================
// U11、U13 的新形态（第 4 次重跑登记第一节 1.2）：管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14）。
// ============================================================================================

struct ForwardRollbackScenario {
    txg_before_the_rollback: u64,
    txg_of_the_rollback_publish: u64,
    instance_before: u32,
    instance_after: u32,
    instance_table_rows_before: usize,
    instance_table_rows_after: usize,
    remount_instance: u32,
    isolated_after_the_plain_remount: Vec<(DeviceIdentity, u64)>,
    abandoned_roots_after_the_plain_remount: usize,
}

fn instance_table_row_count(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    root: &RootRecord,
) -> usize {
    instance_table_chain_of_root(devices, root)
        .expect("现行那一版的实例表读得出")
        .records
        .rows
        .len()
}

fn abandoned_root_count(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    parameters: &MakeFilesystemParameters,
) -> usize {
    let roots = readable_roots(
        devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let newest = roots
        .iter()
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .copied()
        .expect("环里有根");
    let table = instance_table_chain_of_root(devices, &newest)
        .expect("最新根的实例表读得出")
        .records;
    roots
        .iter()
        .filter(|root| root_is_abandoned_by_the_instance_table(root, &table))
        .count()
}

/// U11 的新形态（登记第一节 1.2 第二行）：A → B → 干净重开 → C → 同一次挂载里挂着回退到 A → 普通重开。
fn run_forward_rollback_scenario() -> ForwardRollbackScenario {
    let parameters = parameters();
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("U11 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("U11 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("U11 暖机")
    };
    let first = {
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
        .expect("U11 A")
    };
    let first_txg = first.root.checkpoint_txg;
    let first_instance = first.root.instance;
    {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &first,
            FirstFile {
                content: &vec![0u8; 4100],
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("U11 B");
    }
    let mounted = mount_writable(&parameters, &mut devices).expect("U11 干净重开");
    let mut reopened_allocator = mounted.allocator;
    let reopened_instance = mounted.output.instance;
    let reopened = mounted
        .current
        .into_file_version()
        .expect("U11 重开之后现行版本带文件");
    let mut current = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut reopened_allocator,
            &reopened,
            FirstFile {
                content: &vec![0u8; 2500],
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            reopened_instance,
        )
        .expect("U11 C")
    };
    let txg_before_the_rollback = current.root.checkpoint_txg.0;
    let instance_before = current.root.instance.0;
    let instance_table_rows_before = instance_table_row_count(&devices, &current.root);
    roll_back_by_a_forward_publish(
        &parameters,
        &mut devices,
        &mut reopened_allocator,
        &mut current,
        RollbackTarget {
            instance: first_instance,
            checkpoint_txg: first_txg,
        },
    )
    .expect("U11 挂着回退到 A");
    let txg_of_the_rollback_publish = current.root.checkpoint_txg.0;
    let instance_after = current.root.instance.0;
    let instance_table_rows_after = instance_table_row_count(&devices, &current.root);
    let remounted = mount_writable(&parameters, &mut devices).expect("U11 回退之后普通重开");
    ForwardRollbackScenario {
        txg_before_the_rollback,
        txg_of_the_rollback_publish,
        instance_before,
        instance_after,
        instance_table_rows_before,
        instance_table_rows_after,
        remount_instance: remounted.output.instance.0,
        isolated_after_the_plain_remount: remounted.output.isolated_slots_per_device,
        abandoned_roots_after_the_plain_remount: abandoned_root_count(&devices, &parameters),
    }
}

struct OldestCandidateRollback {
    overwrites: u64,
    target_txg: u64,
    txg_before: u64,
    txg_after: u64,
    instance_before: u32,
    instance_after: u32,
    instance_table_rows_before: usize,
    instance_table_rows_after: usize,
    item1_slots: u64,
    item5_slots: u64,
    allocated_minus_deferred_verdict: InvariantVerdict,
}

/// U13 (a)（登记第一节 1.2 第一行）：mkfs 同一个进程、产品路径分配器、S = 8，覆盖写 3N = 72 次（N = 3S）→ 同一个进程里挂着回退到
/// 候选集里 txg 最小的根。第 5 项只报数（附带，不进判据）。
fn run_rollback_to_the_oldest_candidate() -> OldestCandidateRollback {
    let parameters = parameters();
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("U13 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("U13 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("U13 暖机")
    };
    let mut current = {
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
        .expect("U13 第一个事务")
    };
    let overwrites = 3 * E156_ROOT_RING_REGIONS * E156_SLOTS_PER_REGION;
    for step in 1..=overwrites {
        current = publish_one_workload(
            &parameters,
            devices.as_mut_slice(),
            &mut allocator,
            &current,
            instance,
            WorkloadPublishKind::Overwrite,
            step,
        )
        .expect("U13 前缀覆盖写");
    }
    let effective_floor = effective_rollback_floor(
        &devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let current_table = instance_table_chain_of_root(&devices, &current.root)
        .expect("现行那一版的实例表读得出")
        .records;
    let oldest = readable_roots(
        &devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| {
        root.checkpoint_txg >= effective_floor
            && root.checkpoint_txg.0 >= 3
            && !root_is_abandoned_by_the_instance_table(root, &current_table)
    })
    .min_by_key(|root| root.checkpoint_txg)
    .expect("回退候选集至少一个根");
    let txg_before = current.root.checkpoint_txg.0;
    let instance_before = current.root.instance.0;
    let instance_table_rows_before = instance_table_row_count(&devices, &current.root);
    roll_back_by_a_forward_publish(
        &parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        RollbackTarget {
            instance: oldest.instance,
            checkpoint_txg: oldest.checkpoint_txg,
        },
    )
    .expect("U13 挂着回退到候选集里最旧的根");
    let pool = memory_pool_snapshot(&devices);
    let (item1_slots, _item2_slots, item5_slots) =
        mirror_accounting_row_slots(&pool, current.unit(TransactionUnit::AccountingTree).slot.0);
    OldestCandidateRollback {
        overwrites,
        target_txg: oldest.checkpoint_txg.0,
        txg_before,
        txg_after: current.root.checkpoint_txg.0,
        instance_before,
        instance_after: current.root.instance.0,
        instance_table_rows_before,
        instance_table_rows_after: instance_table_row_count(&devices, &current.root),
        item1_slots,
        item5_slots,
        allocated_minus_deferred_verdict: verdict(&pool, "I-3.11"),
    }
}

/// β0（S = 8、第一个事务之后）与 β_syn（β0 的镜像把 D0 第 1 项 − 1、第 5 项 − 1 成 0、第 2 项 + 1，重封）：U13 (b) 与 [`main`] 共用。
struct SyntheticBase {
    pool: MemoryPool,
    accounting_slot: u64,
    referenced: u64,
}

fn synthetic_base_with_zero_deferred() -> SyntheticBase {
    let parameters = parameters();
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("β0 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("β0 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("β0 暖机")
    };
    let first = {
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
        .expect("β0 第一个事务")
    };
    let accounting_slot = first.unit(TransactionUnit::AccountingTree).slot.0;
    let tree_table_slot = first.unit(TransactionUnit::TreeTable).slot.0;
    let slot_bytes = i64::try_from(SLOT_BYTES).expect("16384");
    let pool = corrupt_device_zero_accounting(
        &memory_pool_snapshot(&devices),
        accounting_slot,
        tree_table_slot,
        &parameters.region_devices,
        parameters.geometry.fixed_structure_slot_spacing,
        &[(1u16, -slot_bytes), (5u16, -slot_bytes), (2u16, slot_bytes)],
    );
    SyntheticBase {
        pool,
        accounting_slot,
        referenced: referenced_slots(&first),
    }
}

// ============================================================================================
// 判定（登记第六节 6.2、第八节 8.2；PC-判定器在单测 U18 与 [`main`] 里各跑一遍）。
// ============================================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PeakJudgement {
    AtMostTheThreshold,
    AboveTheThreshold,
    NothingObserved,
}

impl PeakJudgement {
    fn label(self) -> &'static str {
        match self {
            PeakJudgement::AtMostTheThreshold => "at_most",
            PeakJudgement::AboveTheThreshold => "above",
            PeakJudgement::NothingObserved => "nothing_observed",
        }
    }
}

/// 全部分组键上的峰值里有一个越过门槛 ⇒ 越过；一个都没有 ⇒ 没越过；一个观测都没有 ⇒ 没有观测。
fn judge_peak_against_threshold(peaks: &BTreeMap<u64, i64>, threshold_slots: i64) -> PeakJudgement {
    if peaks.is_empty() {
        return PeakJudgement::NothingObserved;
    }
    if peaks.values().any(|peak| *peak > threshold_slots) {
        PeakJudgement::AboveTheThreshold
    } else {
        PeakJudgement::AtMostTheThreshold
    }
}

fn smallest_key_above_threshold(peaks: &BTreeMap<u64, i64>, threshold_slots: i64) -> Option<u64> {
    peaks
        .iter()
        .find(|(_, peak)| **peak > threshold_slots)
        .map(|(key, _)| *key)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GeometrySensitivity {
    Consistent { sampling_points: usize },
    Unstable,
    NoSamplingPoint,
}

impl GeometrySensitivity {
    fn label(self) -> String {
        match self {
            GeometrySensitivity::Consistent { sampling_points } => {
                format!("consistent_on_{sampling_points}_sampling_points")
            }
            GeometrySensitivity::Unstable => "unstable".to_string(),
            GeometrySensitivity::NoSamplingPoint => "no_sampling_point".to_string(),
        }
    }
}

/// r2「八」字面：逐个取样点的判定值，任意两个不同 ⇒ 不稳定；全部相同 ⇒ N 个取样点一致。没有观测的取样点不算进去。
fn judge_geometry_sensitivity(judgements: &[PeakJudgement]) -> GeometrySensitivity {
    let judged: Vec<PeakJudgement> = judgements
        .iter()
        .copied()
        .filter(|judgement| *judgement != PeakJudgement::NothingObserved)
        .collect();
    match judged.first() {
        None => GeometrySensitivity::NoSamplingPoint,
        Some(first) if judged.iter().all(|judgement| judgement == first) => {
            GeometrySensitivity::Consistent {
                sampling_points: judged.len(),
            }
        }
        Some(_) => GeometrySensitivity::Unstable,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GrowthWithHoles {
    IndependentOfHoles,
    GrowsWithHoles,
    NotMonotonic,
    FewerThanTwoPoints,
}

impl GrowthWithHoles {
    fn label(self) -> &'static str {
        match self {
            GrowthWithHoles::IndependentOfHoles => "independent_of_holes",
            GrowthWithHoles::GrowsWithHoles => "grows_with_holes",
            GrowthWithHoles::NotMonotonic => "not_monotonic",
            GrowthWithHoles::FewerThanTwoPoints => "fewer_than_two_points",
        }
    }
}

/// Q1d：按洞的计数升序列峰值，相邻差全 0 ⇒ 与洞无关；全部 ≥ 0 且有一个 > 0 ⇒ 随洞增长；否则不单调。
fn judge_growth_with_holes(peaks: &BTreeMap<u64, i64>) -> GrowthWithHoles {
    let values: Vec<i64> = peaks.values().copied().collect();
    if values.len() < 2 {
        return GrowthWithHoles::FewerThanTwoPoints;
    }
    let differences: Vec<i64> = values.windows(2).map(|pair| pair[1] - pair[0]).collect();
    if differences.iter().all(|difference| *difference == 0) {
        GrowthWithHoles::IndependentOfHoles
    } else if differences.iter().all(|difference| *difference >= 0) {
        GrowthWithHoles::GrowsWithHoles
    } else {
        GrowthWithHoles::NotMonotonic
    }
}

/// PC-判定器的合成用例（登记第五节 5.4、第八节 8.2 判别力自证 ①）：(说明, 实际, 应得)。
fn judge_positive_control_cases() -> Vec<(&'static str, String, String)> {
    let peaks = |pairs: &[(u64, i64)]| -> BTreeMap<u64, i64> { pairs.iter().copied().collect() };
    vec![
        (
            "peak_three_against_two",
            judge_peak_against_threshold(&peaks(&[(1, 3)]), E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS)
                .label()
                .to_string(),
            "above".to_string(),
        ),
        (
            "peak_two_against_two",
            judge_peak_against_threshold(&peaks(&[(1, 2)]), E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS)
                .label()
                .to_string(),
            "at_most".to_string(),
        ),
        (
            "peak_one_against_one",
            judge_peak_against_threshold(&peaks(&[(1, 1)]), E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS)
                .label()
                .to_string(),
            "at_most".to_string(),
        ),
        (
            "peak_two_against_one",
            judge_peak_against_threshold(&peaks(&[(1, 2)]), E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS)
                .label()
                .to_string(),
            "above".to_string(),
        ),
        (
            "three_at_most_one_above",
            judge_geometry_sensitivity(&[
                PeakJudgement::AtMostTheThreshold,
                PeakJudgement::AtMostTheThreshold,
                PeakJudgement::AtMostTheThreshold,
                PeakJudgement::AboveTheThreshold,
            ])
            .label(),
            "unstable".to_string(),
        ),
        (
            "four_at_most",
            judge_geometry_sensitivity(&[PeakJudgement::AtMostTheThreshold; 4]).label(),
            "consistent_on_4_sampling_points".to_string(),
        ),
        (
            "synthetic_zero_and_five_against_two",
            judge_peak_against_threshold(
                &peaks(&[(1, 0), (2, 5)]),
                E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS,
            )
            .label()
            .to_string(),
            "above".to_string(),
        ),
        (
            "growth_flat",
            judge_growth_with_holes(&peaks(&[(1, 2), (2, 2)]))
                .label()
                .to_string(),
            "independent_of_holes".to_string(),
        ),
        (
            "growth_rising",
            judge_growth_with_holes(&peaks(&[(1, 1), (2, 3), (4, 3)]))
                .label()
                .to_string(),
            "grows_with_holes".to_string(),
        ),
        (
            "growth_falling",
            judge_growth_with_holes(&peaks(&[(1, 3), (2, 1)]))
                .label()
                .to_string(),
            "not_monotonic".to_string(),
        ),
    ]
}

/// 一个 (S, ρ, 回收时点) 分组里的一个 Δ 观测。
#[derive(Clone, Debug)]
struct DeltaSample {
    shape: HhCellShape,
    txg: u64,
    workload_publishes: u64,
    missing_txgs: u64,
    live_holes: u64,
    delta: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
enum DensityKey {
    EveryPublishOverwrites,
    OneOverwriteInFourPublishes,
}

fn density_key(density: WorkloadDensity) -> DensityKey {
    match density {
        WorkloadDensity::EveryPublishOverwrites => DensityKey::EveryPublishOverwrites,
        WorkloadDensity::OneOverwriteInFourPublishes => DensityKey::OneOverwriteInFourPublishes,
    }
}

type GroupKey = (u64, DensityKey, ReclaimTiming);

fn group_label(key: &GroupKey) -> String {
    let density = match key.1 {
        DensityKey::EveryPublishOverwrites => "1",
        DensityKey::OneOverwriteInFourPublishes => "1/4",
    };
    format!("s={} rho={density} timing={}", key.0, key.2.label())
}

/// 一组 Δ 按分组键（h_缺 或 h_洞 或 k）取峰值、和、条数、为正的条数。
fn group_statistics(
    samples: &[&DeltaSample],
    key_of: impl Fn(&DeltaSample) -> u64,
) -> BTreeMap<u64, (i64, i64, u64, u64)> {
    let mut statistics: BTreeMap<u64, (i64, i64, u64, u64)> = BTreeMap::new();
    for sample in samples {
        let entry = statistics
            .entry(key_of(sample))
            .or_insert((i64::MIN, 0, 0, 0));
        entry.0 = entry.0.max(sample.delta);
        entry.1 += sample.delta;
        entry.2 += 1;
        if sample.delta > 0 {
            entry.3 += 1;
        }
    }
    statistics
}

fn peaks_of(statistics: &BTreeMap<u64, (i64, i64, u64, u64)>) -> BTreeMap<u64, i64> {
    statistics
        .iter()
        .map(|(key, entry)| (*key, entry.0))
        .collect()
}

fn statistics_text(statistics: &BTreeMap<u64, (i64, i64, u64, u64)>) -> String {
    let parts: Vec<String> = statistics
        .iter()
        .map(|(key, (peak, sum, count, positive))| {
            format!("{key}:peak{peak}:sum{sum}:count{count}:positive{positive}")
        })
        .collect();
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(",")
    }
}

/// 第一次越过门槛的观测（最小的那个越过门槛的分组键里，工作负载发布数、txg 最小的一条）。
fn first_crossing_text(
    samples: &[&DeltaSample],
    key_of: impl Fn(&DeltaSample) -> u64,
    peaks: &BTreeMap<u64, i64>,
    threshold_slots: i64,
) -> String {
    let Some(key) = smallest_key_above_threshold(peaks, threshold_slots) else {
        return "not_crossed_before_the_end".to_string();
    };
    samples
        .iter()
        .filter(|sample| key_of(sample) == key && sample.delta > threshold_slots)
        .min_by_key(|sample| (sample.workload_publishes, sample.txg))
        .map_or_else(
            || "none".to_string(),
            |sample| {
                format!(
                    "key{key}:k{}:position{}:workload_publishes{}:txg{}",
                    sample.shape.hole_count,
                    sample.shape.position_label(),
                    sample.workload_publishes,
                    sample.txg
                )
            },
        )
}

/// 一个分组的四个 Q1c 读数与 Q1d（登记第六节 6.2 表里 Q1c、Q1c-洞、Q1c-前、Q1c-后、Q1d 各一行）。
#[derive(Clone, Copy, Debug)]
struct GroupJudgements {
    all_points: PeakJudgement,
    on_holes: PeakJudgement,
    not_on_holes: PeakJudgement,
    front_holes: PeakJudgement,
    back_holes: PeakJudgement,
    front_holes_one_slot: PeakJudgement,
    growth: GrowthWithHoles,
    front_peak_is_zero_everywhere: bool,
    back_peak_is_zero_everywhere: bool,
}

#[allow(
    clippy::too_many_lines,
    reason = "一个分组的 Q1c 四行、Q1d 三张表与判别力自证 ② 连着打，才能与登记第六节那张表逐行对"
)]
fn judge_group(emitter: &mut Emitter, key: &GroupKey, samples: &[DeltaSample]) -> GroupJudgements {
    let label = group_label(key);
    let all: Vec<&DeltaSample> = samples.iter().collect();
    let on_holes: Vec<&DeltaSample> = all
        .iter()
        .copied()
        .filter(|sample| sample.missing_txgs >= 1)
        .collect();
    let not_on_holes: Vec<&DeltaSample> = all
        .iter()
        .copied()
        .filter(|sample| sample.missing_txgs == 0)
        .collect();
    let front: Vec<&DeltaSample> = on_holes
        .iter()
        .copied()
        .filter(|sample| sample.shape.position == Some(HolePosition::Front))
        .collect();
    let back: Vec<&DeltaSample> = on_holes
        .iter()
        .copied()
        .filter(|sample| sample.shape.position == Some(HolePosition::Back))
        .collect();
    let by_missing = |sample: &DeltaSample| sample.missing_txgs;
    let mut judge_quantity = |quantity: &str,
                              subset: &[&DeltaSample]|
     -> (PeakJudgement, PeakJudgement, BTreeMap<u64, i64>) {
        let statistics = group_statistics(subset, by_missing);
        let peaks = peaks_of(&statistics);
        let two = judge_peak_against_threshold(&peaks, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS);
        let one = judge_peak_against_threshold(&peaks, E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS);
        emitter.emit(&format!(
            "name=q1c {label} quantity={quantity} samples={} judgement_two_slots={} judgement_one_slot={} smallest_h_que_above_two={} smallest_h_que_above_one={} first_crossing_two={} by_h_que={}",
            subset.len(),
            two.label(),
            one.label(),
            smallest_key_above_threshold(&peaks, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS)
                .map_or_else(|| "none".to_string(), |smallest_key| smallest_key.to_string()),
            smallest_key_above_threshold(&peaks, E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS)
                .map_or_else(|| "none".to_string(), |smallest_key| smallest_key.to_string()),
            first_crossing_text(subset, by_missing, &peaks, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS),
            statistics_text(&statistics)
        ));
        (two, one, peaks)
    };
    let (all_points, _, all_peaks) = judge_quantity("q1c", &all);
    let (on_holes_judgement, _, _) = judge_quantity("q1c_on_holes", &on_holes);
    let (not_on_holes_judgement, _, _) = judge_quantity("q1c_not_on_holes", &not_on_holes);
    let (front_judgement, front_one_slot, front_peaks) = judge_quantity("q1c_front", &front);
    let (back_judgement, _, back_peaks) = judge_quantity("q1c_back", &back);

    let growth = judge_growth_with_holes(&all_peaks);
    let adjacent: Vec<String> = all_peaks
        .iter()
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| format!("{}->{}:{}", pair[0].0, pair[1].0, pair[1].1 - pair[0].1))
        .collect();
    let by_live_holes = group_statistics(&all, |sample| sample.live_holes);
    let by_hole_count = group_statistics(&all, |sample| sample.shape.hole_count);
    emitter.emit(&format!(
        "name=q1d {label} growth_by_h_que={} adjacent_differences={} first_crossing_two={} first_crossing_one={} by_h_dong={} by_injected_k={}",
        growth.label(),
        if adjacent.is_empty() { "none".to_string() } else { adjacent.join(",") },
        first_crossing_text(&all, by_missing, &all_peaks, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS),
        first_crossing_text(&all, by_missing, &all_peaks, E156_THRESHOLD_ONE_SPAN_ONE_UNIT_SLOTS),
        statistics_text(&by_live_holes),
        statistics_text(&by_hole_count),
    ));

    // 判别力自证 ②（第八节 8.2）：把 Q1c-前 的门槛挪到真产物 Δ 峰值两个观测值之间，同一份真产物上的判定必须转一次。
    let distinct_front_peaks: BTreeSet<i64> = front_peaks.values().copied().collect();
    if distinct_front_peaks.len() >= 2 {
        let largest = *distinct_front_peaks.iter().next_back().expect("至少两个");
        let second_largest = *distinct_front_peaks.iter().rev().nth(1).expect("至少两个");
        let between = judge_peak_against_threshold(&front_peaks, second_largest);
        let at_largest = judge_peak_against_threshold(&front_peaks, largest);
        emitter.emit(&format!(
            "name=q1c_front_threshold_moved {label} threshold_between={second_largest} judgement_between={} threshold_at_largest={largest} judgement_at_largest={} flips={}",
            between.label(),
            at_largest.label(),
            between != at_largest
        ));
    } else {
        let synthetic: BTreeMap<u64, i64> = [(1u64, 0i64), (2, 5)].into_iter().collect();
        let synthetic_judgement =
            judge_peak_against_threshold(&synthetic, E156_THRESHOLD_ONE_SPAN_TWO_UNIT_SLOTS);
        emitter.emit(&format!(
            "name=q1c_front_threshold_moved {label} real_distinct_peaks={} real_product_has_no_two_points_to_move_between=true synthetic_zero_and_five_against_two={} synthetic_is_red={}",
            distinct_front_peaks.len(),
            synthetic_judgement.label(),
            synthetic_judgement == PeakJudgement::AboveTheThreshold
        ));
    }
    GroupJudgements {
        all_points,
        on_holes: on_holes_judgement,
        not_on_holes: not_on_holes_judgement,
        front_holes: front_judgement,
        back_holes: back_judgement,
        front_holes_one_slot: front_one_slot,
        growth,
        front_peak_is_zero_everywhere: front_peaks.values().all(|peak| *peak == 0),
        back_peak_is_zero_everywhere: back_peaks.values().all(|peak| *peak == 0),
    }
}

// ============================================================================================
// main
// ============================================================================================

fn emit_static_anchors(emitter: &mut Emitter) {
    let default_parameters = parameters();
    // 装置的本地 S 与实现侧回比（入库装置第 ① 条）：mkfs 默认、上下界、区域设备。
    assert_eq!(
        E156_SLOTS_PER_REGION,
        default_parameters
            .geometry
            .root_ring_slots_per_region
            .count(),
        "装置写死的 mkfs 默认每区槽数 S 与实现侧不等"
    );
    assert_eq!(
        (E156_SLOTS_PER_REGION_MINIMUM, E156_SLOTS_PER_REGION_MAXIMUM),
        (
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM
        ),
        "装置写死的 S 上下界与实现侧不等"
    );
    assert_eq!(
        E156_REGION_DEVICE_NUMBERS.map(DeviceIdentity),
        default_parameters.region_devices,
        "装置写死的区域设备与 mkfs 参数不等"
    );
    assert_eq!(
        E156_ROOT_RING_REGIONS, ROOT_RING_REGIONS,
        "装置写死的区域数 R 与实现侧不等"
    );
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        assert!(
            RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region).is_ok(),
            "K8：S = {slots_per_region} 应被接受"
        );
    }
    assert_eq!(
        (
            E156_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
            E156_SYSTEM_CONFIGURATION_SLOT_BYTES
        ),
        (
            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
            singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES
        ),
        "装置写死的系统配置槽数与槽宽与实现侧不等"
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
    let allocation_record_node_capacity =
        (16384 - E156_ALLOCATION_RECORD_NODE_HEADER_BYTES) / E156_ALLOCATION_RECORD_BYTES;
    emitter.emit(&format!(
        "name=anchor_a7 capacity={allocation_record_node_capacity}"
    ));
    assert_eq!(
        allocation_record_node_capacity, 812,
        "A7：分配记录节点容量应为 812"
    );
    let leaf_slots_match_format_crate = E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
        == singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS;
    let internal_fanout_match_format_crate = E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
        == singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
    let my_root_level = e156_root_level_for_symmetric_devices(unit_area_slot_count, 2);
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&default_parameters, &mut devices).expect("A-D8 mkfs");
    let allocator = allocator_after_make_filesystem(&default_parameters, &devices, &genesis);
    let real_root_level =
        singlefs_core::allocation_record_tree::AllocationRecordTreeGeometry::of_allocator(
            &allocator,
        )
        .root_level();
    emitter.emit(&format!(
        "name=anchor_a_d8 local_leaf_slots={E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS} local_internal_fanout={E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT} local_leaf_slots_matches_format_crate={leaf_slots_match_format_crate} local_internal_fanout_matches_format_crate={internal_fanout_match_format_crate} my_root_level={my_root_level} real_root_level={real_root_level} root_level_matches={}",
        my_root_level == real_root_level
    ));
    // A2：根环 3S 槽、txg 落的槽与 u mod 3S 一一对应（每个取样点的 S 各核一次，用真实几何函数）。
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        let geometry = RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
            .expect("S 落在区间里");
        let ring_length = E156_ROOT_RING_REGIONS * slots_per_region;
        let mut distinct_slots: BTreeSet<RootRingSlot> = BTreeSet::new();
        let mut mismatches = 0u64;
        for probe_txg in 0..(ring_length * 4) {
            let target = target_for_publish(CheckpointTxg(probe_txg), geometry);
            if probe_txg < ring_length {
                distinct_slots.insert(target);
            }
            let wrapped = target_for_publish(CheckpointTxg(probe_txg % ring_length), geometry);
            if target != wrapped {
                mismatches += 1;
            }
        }
        let holds =
            distinct_slots.len() == usize::try_from(ring_length).expect("环长") && mismatches == 0;
        emitter.emit(&format!(
            "name=anchor_a2 s={slots_per_region} ring_slots={ring_length} distinct_slots_in_one_ring={} wrap_mismatches={mismatches} holds={holds}",
            distinct_slots.len()
        ));
        assert!(
            holds,
            "A2：S = {slots_per_region} 时 txg 的槽与 u mod 3S 应一一对应"
        );
    }
}

/// 一个 S 下锚点模型的全部格：k = 0 与 (k, 位置) ∈ {1, 2, 4} × {前, 后}。
struct AnchorCellsOfOneSlotsPerRegion {
    without_holes: anchor_model::AnchorCell,
    cells: Vec<(u64, HolePosition, anchor_model::AnchorCell)>,
}

fn anchor_cells_of(slots_per_region: u64) -> AnchorCellsOfOneSlotsPerRegion {
    let without_holes_end = anchor_model::without_holes_end(slots_per_region);
    let mut cells = Vec::new();
    for position in [HolePosition::Front, HolePosition::Back] {
        for hole_count in [1u64, 2, 4] {
            cells.push((
                hole_count,
                position,
                anchor_model::run(
                    slots_per_region,
                    hole_count,
                    position == HolePosition::Front,
                    None,
                ),
            ));
        }
    }
    AnchorCellsOfOneSlotsPerRegion {
        without_holes: anchor_model::run(slots_per_region, 0, false, Some(without_holes_end)),
        cells,
    }
}

fn anchor_cell_for(
    anchors: &BTreeMap<u64, AnchorCellsOfOneSlotsPerRegion>,
    slots_per_region: u64,
    hole_count: u64,
    position: Option<HolePosition>,
) -> &anchor_model::AnchorCell {
    let of_slots = &anchors[&slots_per_region];
    match position {
        None => &of_slots.without_holes,
        Some(position) => of_slots
            .cells
            .iter()
            .find(|(count, cell_position, _)| *count == hole_count && *cell_position == position)
            .map(|(_, _, cell)| cell)
            .expect("锚点模型有这一格"),
    }
}

/// 一格的 A10–A16、K8、S1 与 V 系列核对的汇总（第二段的 V / S / F 判定从这里取）。
#[derive(Default, Clone, Copy)]
struct CellCheckTotals {
    hole_count_anchor_mismatches: u64,
    threshold_anchor_mismatches: u64,
    every_publish_below_tree_table_bound: u64,
    real_below_tree_table_bound: u64,
    every_publish_nonzero_without_holes: u64,
    real_nonzero_without_holes: u64,
    interval_rule_every_publish_nonzero: u64,
    interval_rule_real_nonzero: u64,
    reclaim_rule_violation_points: u64,
    devices_unequal: u64,
    root_ring_missing: u64,
    anchor_rows_missing: u64,
    real_every_differ: u64,
    q1a_plus_positive: u64,
    device_and_crates_disagreements: u64,
    admission_touches: u64,
    abandoned_or_unknown: u64,
    slots_per_region_read_back_mismatches: u64,
    tree_table_lag_every_publish_mismatch: u64,
    tree_table_lag_real_mismatch: u64,
}

impl CellCheckTotals {
    fn add(&mut self, other: CellCheckTotals) {
        self.hole_count_anchor_mismatches += other.hole_count_anchor_mismatches;
        self.threshold_anchor_mismatches += other.threshold_anchor_mismatches;
        self.every_publish_below_tree_table_bound += other.every_publish_below_tree_table_bound;
        self.real_below_tree_table_bound += other.real_below_tree_table_bound;
        self.every_publish_nonzero_without_holes += other.every_publish_nonzero_without_holes;
        self.real_nonzero_without_holes += other.real_nonzero_without_holes;
        self.interval_rule_every_publish_nonzero += other.interval_rule_every_publish_nonzero;
        self.interval_rule_real_nonzero += other.interval_rule_real_nonzero;
        self.reclaim_rule_violation_points += other.reclaim_rule_violation_points;
        self.devices_unequal += other.devices_unequal;
        self.root_ring_missing += other.root_ring_missing;
        self.anchor_rows_missing += other.anchor_rows_missing;
        self.real_every_differ += other.real_every_differ;
        self.q1a_plus_positive += other.q1a_plus_positive;
        self.device_and_crates_disagreements += other.device_and_crates_disagreements;
        self.admission_touches += other.admission_touches;
        self.abandoned_or_unknown += other.abandoned_or_unknown;
        self.slots_per_region_read_back_mismatches += other.slots_per_region_read_back_mismatches;
        self.tree_table_lag_every_publish_mismatch += other.tree_table_lag_every_publish_mismatch;
        self.tree_table_lag_real_mismatch += other.tree_table_lag_real_mismatch;
    }
}

fn count_where(observations: &[HhObservation], predicate: impl Fn(&HhObservation) -> bool) -> u64 {
    u64::try_from(
        observations
            .iter()
            .filter(|observation| predicate(observation))
            .count(),
    )
    .expect("落在 u64 内")
}

fn check_cell(
    outcome: &HhRunOutcome,
    expected_tree_table_lags: Option<&BTreeSet<u64>>,
) -> CellCheckTotals {
    let observations = &outcome.observations;
    let has_no_holes = outcome.shape.hole_count == 0;
    let floor_rule =
        |observation: &HhObservation, timing| observation.over_withheld(Arm::FloorRule, timing);
    let interval_rule = |observation: &HhObservation, timing| {
        observation.over_withheld(Arm::LifetimeIntervalRule, timing)
    };
    CellCheckTotals {
        hole_count_anchor_mismatches: count_where(observations, |observation| {
            observation.anchor.is_some_and(|row| {
                row.missing_txgs != observation.reading.missing_txgs
                    || row.live_holes != observation.reading.live_holes
            })
        }),
        threshold_anchor_mismatches: count_where(observations, |observation| {
            observation
                .anchor
                .is_some_and(|row| row.floor != observation.reading.ring_threshold)
        }),
        every_publish_below_tree_table_bound: count_where(observations, |observation| {
            observation.anchor.is_some_and(|row| {
                floor_rule(observation, ReclaimTiming::EveryPublish) < row.tree_table_lower_bound
            })
        }),
        real_below_tree_table_bound: count_where(observations, |observation| {
            observation.anchor.is_some_and(|row| {
                floor_rule(observation, ReclaimTiming::Real) < row.tree_table_lower_bound
            })
        }),
        every_publish_nonzero_without_holes: if has_no_holes {
            count_where(observations, |observation| {
                floor_rule(observation, ReclaimTiming::EveryPublish) != 0
                    || interval_rule(observation, ReclaimTiming::EveryPublish) != 0
            })
        } else {
            0
        },
        real_nonzero_without_holes: if has_no_holes {
            count_where(observations, |observation| {
                floor_rule(observation, ReclaimTiming::Real) != 0
                    || interval_rule(observation, ReclaimTiming::Real) != 0
            })
        } else {
            0
        },
        interval_rule_every_publish_nonzero: count_where(observations, |observation| {
            interval_rule(observation, ReclaimTiming::EveryPublish) != 0
        }),
        interval_rule_real_nonzero: count_where(observations, |observation| {
            interval_rule(observation, ReclaimTiming::Real) != 0
        }),
        reclaim_rule_violation_points: count_where(observations, |observation| {
            observation.reading.device_zero.reclaim_rule_violations > 0
        }),
        devices_unequal: count_where(observations, |observation| {
            !observation.reading.devices_equal
        }),
        root_ring_missing: count_where(observations, |observation| {
            !observation.reading.root_ring_installed
        }),
        anchor_rows_missing: count_where(observations, |observation| observation.anchor.is_none()),
        real_every_differ: count_where(observations, |observation| {
            floor_rule(observation, ReclaimTiming::Real)
                != floor_rule(observation, ReclaimTiming::EveryPublish)
                || interval_rule(observation, ReclaimTiming::Real)
                    != interval_rule(observation, ReclaimTiming::EveryPublish)
        }),
        q1a_plus_positive: count_where(observations, |observation| {
            let device_zero = &observation.reading.device_zero;
            device_zero.floor_rule_real_referenced_but_free > 0
                || device_zero.floor_rule_every_publish_referenced_but_free > 0
                || device_zero.interval_rule_referenced_but_free > 0
        }),
        device_and_crates_disagreements: u64::from(!outcome.genesis_root_in_ring)
            + u64::from(!outcome.first_transaction_matches_the_registered_anchor)
            + outcome.ring_wrap_overwrite_mismatches
            + outcome.mount_schedule_mismatches
            + outcome.disk_cross_check_mismatches
            + outcome.tree_table_release_violations
            + u64::try_from(
                outcome
                    .recoveries
                    .iter()
                    .filter(|recovery| {
                        outcome.cut_point == CrashCutPoint::BeforeTheRootSlotWrite
                            && (!recovery.cut_matches_the_restored_image
                                || recovery.root_slot_writes_in_the_crashed_publish != 1)
                    })
                    .count(),
            )
            .expect("落在 u64 内"),
        admission_touches: outcome.admission_refusals
            + u64::try_from(
                outcome
                    .recoveries
                    .iter()
                    .filter(|recovery| {
                        recovery.mount_admission != "admitted_before_acquisition"
                            || recovery.mount_floor_raise_sequences > 0
                    })
                    .count(),
            )
            .expect("落在 u64 内")
            + count_where(observations, |observation| {
                observation.reading.effective_floor > 0
            }),
        abandoned_or_unknown: count_where(observations, |observation| {
            observation.reading.abandoned_roots > 0
                || observation.reading.roots_without_version_facts > 0
        }),
        slots_per_region_read_back_mismatches: u64::from(
            outcome.slots_per_region_read_back != outcome.shape.slots_per_region,
        ),
        tree_table_lag_every_publish_mismatch: u64::from(
            expected_tree_table_lags
                .is_some_and(|lags| *lags != outcome.tree_table_lags_every_publish),
        ),
        tree_table_lag_real_mismatch: u64::from(
            expected_tree_table_lags.is_some_and(|lags| *lags != outcome.tree_table_lags_real),
        ),
    }
}

fn emit_cell_checks(emitter: &mut Emitter, outcome: &HhRunOutcome, totals: &CellCheckTotals) {
    emitter.emit(&format!(
        "name=q1_cell_checks family={} {} a11_mismatches={} a14_mismatches={} a15_every_below={} a15_real_below={} a12_every_nonzero={} a12_real_nonzero={} a13_every_nonzero={} a13_real_nonzero={} k10_violation_points={} devices_unequal_points={} root_ring_missing_points={} anchor_rows_missing={} real_every_differ_points={} referenced_but_free_points={} s1_failures={} g_adm_nonzero={} abandoned_or_unknown_root_points={} k8_mismatches={} a10_every_mismatch={} a10_real_mismatch={}",
        outcome.family,
        outcome.shape.fields(),
        totals.hole_count_anchor_mismatches,
        totals.threshold_anchor_mismatches,
        totals.every_publish_below_tree_table_bound,
        totals.real_below_tree_table_bound,
        totals.every_publish_nonzero_without_holes,
        totals.real_nonzero_without_holes,
        totals.interval_rule_every_publish_nonzero,
        totals.interval_rule_real_nonzero,
        totals.reclaim_rule_violation_points,
        totals.devices_unequal,
        totals.root_ring_missing,
        totals.anchor_rows_missing,
        totals.real_every_differ,
        totals.q1a_plus_positive,
        totals.device_and_crates_disagreements,
        totals.admission_touches,
        totals.abandoned_or_unknown,
        totals.slots_per_region_read_back_mismatches,
        totals.tree_table_lag_every_publish_mismatch,
        totals.tree_table_lag_real_mismatch,
    ));
}

/// PQ1 一段历史里「造出了洞」的判定（登记第六节 6.1：② = t + 1 且 ③ 没有）。
fn premise_one_reproduced(outcome: &HhRunOutcome) -> bool {
    !outcome.recoveries.is_empty()
        && outcome.recoveries.iter().all(|recovery| {
            recovery.first_txg_of_the_new_instance == recovery.hole_txg + 1
                && !recovery.ring_has_a_root_at_the_hole_txg
        })
}

fn hh_request<'anchor>(
    family: &'static str,
    shape: HhCellShape,
    anchor: &'anchor anchor_model::AnchorCell,
) -> HhRunRequest<'anchor> {
    let is_first_geometry = shape.slots_per_region == E156_SLOTS_PER_REGION
        && shape.density == WorkloadDensity::EveryPublishOverwrites;
    HhRunRequest {
        family,
        shape,
        allocator_source: HhAllocatorSource::ProductPathWithRootRing,
        cut_point: CrashCutPoint::BeforeTheRootSlotWrite,
        anchor,
        check_overwrites_before_the_ring_wraps: is_first_geometry && shape.hole_count == 0,
        judge_checker_on_hole_states: is_first_geometry,
        emit_positive_control_no_reclaim: shape.hole_count == 0
            || (shape.hole_count == 2 && shape.position == Some(HolePosition::Back)),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "第一段、第二段与全部判定行按登记第五节 5.5 的次序连着跑，拆开反而难与登记逐行对"
)]
fn main() {
    let mut emitter = Emitter { emitted: 0 };
    emit_static_anchors(&mut emitter);

    // 锚点模型（命令四的移植）：逐观测点那几行，去掉前缀之后与 Python 脚本的 `--dump` 逐行比（报告里的命令）。
    let mut anchors: BTreeMap<u64, AnchorCellsOfOneSlotsPerRegion> = BTreeMap::new();
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        for line in anchor_model::dump_lines(slots_per_region) {
            emitter.emit(&format!("name=anchor_dump {line}"));
        }
        let cells = anchor_cells_of(slots_per_region);
        let lags = anchor_model::tree_table_reclaim_lags(slots_per_region, cells.without_holes.end);
        // 命令四的另两样：排得下的格上 r2 的两条闭式与按环现数逐点相同（A11），k = 0 的 Q1a / Q1b 与全部格的 Q1b 恒 0（A12、A13）。
        let fitting_rows = || {
            cells
                .cells
                .iter()
                .filter(|(_, _, cell)| cell.fits)
                .flat_map(|(_, _, cell)| cell.rows.iter())
        };
        let closed_form_mismatches = fitting_rows()
            .filter(|row| {
                row.live_holes != row.live_holes_closed_form
                    || row.missing_txgs != row.missing_txgs_closed_form
            })
            .count();
        let interval_rule_maximum = fitting_rows()
            .chain(cells.without_holes.rows.iter())
            .map(|row| row.interval_rule_tree_table_over_withheld)
            .max()
            .unwrap_or(0);
        let without_holes_floor_rule_maximum = cells
            .without_holes
            .rows
            .iter()
            .map(|row| row.tree_table_lower_bound)
            .max()
            .unwrap_or(0);
        emitter.emit(&format!(
            "name=anchor_model s={slots_per_region} k0_end={} a10_lags={} a11_closed_form_mismatches_on_fitting_cells={closed_form_mismatches} a12_k0_max_q1a={without_holes_floor_rule_maximum} a13_max_q1b={interval_rule_maximum} fits={}",
            cells.without_holes.end,
            set_text(&lags),
            cells
                .cells
                .iter()
                .map(|(count, position, cell)| format!("{count}{}:{}", position.label(), cell.fits))
                .collect::<Vec<_>>()
                .join(",")
        ));
        anchors.insert(slots_per_region, cells);
    }

    // ===================== 第一段 =====================
    let shape_of = |slots_per_region: u64,
                    density: WorkloadDensity,
                    hole_count: u64,
                    position: Option<HolePosition>| HhCellShape {
        slots_per_region,
        density,
        hole_count,
        position,
    };
    let premise_front_shape = shape_of(
        4,
        WorkloadDensity::EveryPublishOverwrites,
        1,
        Some(HolePosition::Front),
    );
    let premise_back_shape = shape_of(
        4,
        WorkloadDensity::EveryPublishOverwrites,
        1,
        Some(HolePosition::Back),
    );
    let premise_front = run_hh_history(
        &mut emitter,
        &hh_request(
            "Hh",
            premise_front_shape,
            anchor_cell_for(&anchors, 4, 1, Some(HolePosition::Front)),
        ),
    );
    let premise_back = run_hh_history(
        &mut emitter,
        &hh_request(
            "Hh",
            premise_back_shape,
            anchor_cell_for(&anchors, 4, 1, Some(HolePosition::Back)),
        ),
    );
    for (label, outcome) in [("front", &premise_front), ("back", &premise_back)] {
        for recovery in &outcome.recoveries {
            emitter.emit(&format!(
                "name=pq1 position={label} hole_txg={} prefix_applied={} first_txg_of_new_instance={} ring_has_root_at_hole_txg={} hole_slot_content={} a16_applicable={}",
                recovery.hole_txg,
                recovery.prefix_applied,
                recovery.first_txg_of_the_new_instance,
                recovery.ring_has_a_root_at_the_hole_txg,
                recovery.hole_slot_content,
                recovery.prefix_applied > 0
            ));
        }
    }
    let positive_control_crash = {
        let mut request = hh_request(
            "PC-c2",
            premise_back_shape,
            anchor_cell_for(&anchors, 4, 1, Some(HolePosition::Back)),
        );
        request.cut_point = CrashCutPoint::AfterTheRootSlotWritePositiveControl;
        request.emit_positive_control_no_reclaim = false;
        run_hh_history(&mut emitter, &request)
    };
    let positive_control_crash_seen = positive_control_crash
        .recoveries
        .first()
        .is_some_and(|recovery| recovery.ring_has_a_root_at_the_hole_txg)
        && positive_control_crash
            .observations
            .iter()
            .find(|observation| observation.point == ObservationPoint::MountReturn)
            .is_some_and(|observation| observation.reading.missing_txgs == 0)
        && check_cell(&positive_control_crash, None).threshold_anchor_mismatches > 0;
    emitter.emit(&format!(
        "name=pc_c2 ring_has_root_at_hole_txg={} mount_return_h_que={} a14_mismatches={} seen={positive_control_crash_seen}",
        positive_control_crash
            .recoveries
            .first()
            .is_some_and(|recovery| recovery.ring_has_a_root_at_the_hole_txg),
        positive_control_crash
            .observations
            .iter()
            .find(|observation| observation.point == ObservationPoint::MountReturn)
            .map_or_else(|| "none".to_string(), |observation| observation.reading.missing_txgs.to_string()),
        check_cell(&positive_control_crash, None).threshold_anchor_mismatches
    ));

    let floor_raise = run_floor_raise_trace();
    let pushed_roots_text: Vec<String> = floor_raise
        .pushed_roots
        .iter()
        .map(|(txg, root_floor, effective)| {
            format!("txg{txg}:root_floor{root_floor}:effective{effective}")
        })
        .collect();
    let premise_two_merged = floor_raise.the_two_readings_merge();
    emitter.emit(&format!(
        "name=pq2 raised={} floor_before={} new_floor={} effective_floor_before_any_write={} system_configuration_writes={} effective_floor_after_last_system_configuration_write={} system_configuration_writes_after_first_root={} pushed_roots={} publishes={} reclaimed_placements={} the_two_readings_merge={premise_two_merged}",
        floor_raise.raised,
        floor_raise.floor_before,
        floor_raise.new_floor,
        floor_raise.effective_floor_before_any_write,
        floor_raise.system_configuration_writes,
        floor_raise
            .effective_floor_after_the_last_system_configuration_write
            .map_or_else(|| "none".to_string(), |floor| floor.to_string()),
        floor_raise.system_configuration_writes_after_the_first_root,
        pushed_roots_text.join(","),
        floor_raise.publishes,
        floor_raise.reclaimed_placements,
    ));

    let forward_rollback = run_forward_rollback_scenario();
    let forward_rollback_structure_holds = forward_rollback.txg_of_the_rollback_publish
        == forward_rollback.txg_before_the_rollback + 1
        && forward_rollback.instance_after == forward_rollback.instance_before
        && forward_rollback.instance_table_rows_after
            == forward_rollback.instance_table_rows_before
        && forward_rollback.isolated_after_the_plain_remount
            == vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)]
        && forward_rollback.abandoned_roots_after_the_plain_remount == 0;
    emitter.emit(&format!(
        "name=u11_forward_rollback txg_before={} txg_of_rollback_publish={} instance_before={} instance_after={} instance_table_rows_before={} instance_table_rows_after={} remount_instance={} isolated_after_plain_remount={:?} abandoned_roots_after_plain_remount={} forward_rollback_structure_holds={forward_rollback_structure_holds}",
        forward_rollback.txg_before_the_rollback,
        forward_rollback.txg_of_the_rollback_publish,
        forward_rollback.instance_before,
        forward_rollback.instance_after,
        forward_rollback.instance_table_rows_before,
        forward_rollback.instance_table_rows_after,
        forward_rollback.remount_instance,
        forward_rollback
            .isolated_after_the_plain_remount
            .iter()
            .map(|(device, slots)| (device.0, *slots))
            .collect::<Vec<_>>(),
        forward_rollback.abandoned_roots_after_the_plain_remount,
    ));
    let oldest_rollback = run_rollback_to_the_oldest_candidate();
    let u13a_structure_holds = oldest_rollback.txg_after == oldest_rollback.txg_before + 1
        && oldest_rollback.instance_after == oldest_rollback.instance_before
        && oldest_rollback.instance_table_rows_after == oldest_rollback.instance_table_rows_before;
    emitter.emit(&format!(
        "name=u13a_rollback_to_oldest_candidate overwrites={} target_txg={} txg_before={} txg_after={} instance_before={} instance_after={} instance_table_rows_before={} instance_table_rows_after={} item1_slots={} item5_slots={} i311={} structure_holds={u13a_structure_holds}",
        oldest_rollback.overwrites,
        oldest_rollback.target_txg,
        oldest_rollback.txg_before,
        oldest_rollback.txg_after,
        oldest_rollback.instance_before,
        oldest_rollback.instance_after,
        oldest_rollback.instance_table_rows_before,
        oldest_rollback.instance_table_rows_after,
        oldest_rollback.item1_slots,
        oldest_rollback.item5_slots,
        verdict_label(&oldest_rollback.allocated_minus_deferred_verdict),
    ));
    let synthetic = synthetic_base_with_zero_deferred();
    let (synthetic_flip1, synthetic_flip2) = q7c_self_test(
        &mut emitter,
        "beta_syn",
        &synthetic.pool,
        synthetic.accounting_slot,
        synthetic.referenced,
    );

    let premise_totals = {
        let mut totals = check_cell(&premise_front, None);
        totals.add(check_cell(&premise_back, None));
        totals
    };
    let premise_one_holds =
        premise_one_reproduced(&premise_front) && premise_one_reproduced(&premise_back);
    let stage_one_stop_reasons: Vec<&str> = [
        (!premise_one_holds, "sp1_premise_one_not_reproduced"),
        (
            premise_totals.device_and_crates_disagreements > 0,
            "s1_device_and_crates_disagree",
        ),
        (
            premise_totals.root_ring_missing > 0,
            "s1h_root_ring_not_installed",
        ),
        (
            premise_totals.slots_per_region_read_back_mismatches > 0,
            "k8_slots_per_region_read_back",
        ),
        (
            premise_totals.admission_touches > 0,
            "s12_admission_touched_the_history",
        ),
        (
            premise_totals.hole_count_anchor_mismatches > 0,
            "v6_hole_counts_differ_from_the_anchor",
        ),
        (
            premise_totals.threshold_anchor_mismatches > 0,
            "v2_threshold_differs_from_the_anchor",
        ),
        (!positive_control_crash_seen, "v2_pc_c2_not_seen"),
    ]
    .into_iter()
    .filter(|(triggered, _)| *triggered)
    .map(|(_, reason)| reason)
    .collect();
    emitter.emit(&format!(
        "name=verdict stage=first premise_one_reproduced={premise_one_holds} premise_two_the_two_readings_merge={premise_two_merged} premise_two_answer={} fork_three={} g_adm_nonzero={} s1_failures={} a11_mismatches={} a14_mismatches={} pc_c2_seen={positive_control_crash_seen} k11_holds={} forward_rollback_structure_holds={forward_rollback_structure_holds} u13a_structure_holds={u13a_structure_holds} u13b_flip1={synthetic_flip1} u13b_flip2_not_applicable={} stop_reasons={}",
        if premise_two_merged { "not_holds" } else { "holds" },
        if premise_two_merged { "void_not_measured" } else { "sp2_needs_a_new_registration" },
        premise_totals.admission_touches,
        premise_totals.device_and_crates_disagreements,
        premise_totals.hole_count_anchor_mismatches,
        premise_totals.threshold_anchor_mismatches,
        floor_raise.effective_floor_after_the_last_system_configuration_write == Some(floor_raise.new_floor),
        !synthetic_flip2,
        if stage_one_stop_reasons.is_empty() { "none".to_string() } else { stage_one_stop_reasons.join(",") },
    ));
    if !stage_one_stop_reasons.is_empty() {
        emitter.emit("name=stage_two_not_run reason=stage_one_stop");
        emitter.finish();
        return;
    }

    // ===================== 第二段 =====================
    let judge_cases = judge_positive_control_cases();
    let judge_wrong = judge_cases
        .iter()
        .filter(|(_, actual, expected)| actual != expected)
        .count();
    for (case, actual, expected) in &judge_cases {
        emitter.emit(&format!(
            "name=pc_judge case={case} actual={actual} expected={expected} holds={}",
            actual == expected
        ));
    }

    let mut outcomes: Vec<HhRunOutcome> = Vec::new();
    let mut grid_totals = CellCheckTotals::default();
    let mut every_publish_tree_table_lag_mismatch_cells = 0u64;
    let mut not_fitting = 0u64;
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        let expected_lags = anchor_model::tree_table_reclaim_lags(
            slots_per_region,
            anchors[&slots_per_region].without_holes.end,
        );
        for density in [
            WorkloadDensity::EveryPublishOverwrites,
            WorkloadDensity::OneOverwriteInFourPublishes,
        ] {
            let mut shapes = vec![shape_of(slots_per_region, density, 0, None)];
            for position in [HolePosition::Front, HolePosition::Back] {
                for hole_count in [1u64, 2, 4] {
                    shapes.push(shape_of(
                        slots_per_region,
                        density,
                        hole_count,
                        Some(position),
                    ));
                }
            }
            for shape in shapes {
                let anchor =
                    anchor_cell_for(&anchors, slots_per_region, shape.hole_count, shape.position);
                if !anchor.fits {
                    not_fitting += 1;
                    emitter.emit(&format!(
                        "name=q1_cell_not_run {} reason=geometry_does_not_fit",
                        shape.fields()
                    ));
                    continue;
                }
                let reuse = if slots_per_region == 4
                    && density == WorkloadDensity::EveryPublishOverwrites
                    && shape.hole_count == 1
                {
                    match shape.position {
                        Some(HolePosition::Front) => Some(premise_front.clone()),
                        Some(HolePosition::Back) => Some(premise_back.clone()),
                        None => None,
                    }
                } else {
                    None
                };
                let outcome = match reuse {
                    Some(outcome) => {
                        emitter.emit(&format!(
                            "name=q1_cell_reused {} from=stage_one_premise_history",
                            shape.fields()
                        ));
                        outcome
                    }
                    None => run_hh_history(&mut emitter, &hh_request("Hh", shape, anchor)),
                };
                let lags = if shape.hole_count == 0 {
                    Some(&expected_lags)
                } else {
                    None
                };
                let totals = check_cell(&outcome, lags);
                every_publish_tree_table_lag_mismatch_cells +=
                    totals.tree_table_lag_every_publish_mismatch;
                emit_cell_checks(&mut emitter, &outcome, &totals);
                grid_totals.add(totals);
                outcomes.push(outcome);
            }
        }
    }

    // PC-分配器：Hh(0, —, S = 4, ρ = 1) 换回不装根环表的分配器，环转过一圈之后 A10 应判红。
    let positive_control_allocator = {
        let shape = shape_of(4, WorkloadDensity::EveryPublishOverwrites, 0, None);
        let mut request = hh_request("PC-allocator", shape, anchor_cell_for(&anchors, 4, 0, None));
        request.allocator_source = HhAllocatorSource::WithoutRootRingPositiveControl;
        request.emit_positive_control_no_reclaim = false;
        run_hh_history(&mut emitter, &request)
    };
    let expected_lags_four =
        anchor_model::tree_table_reclaim_lags(4, anchors[&4].without_holes.end);
    let positive_control_allocator_seen = positive_control_allocator.tree_table_lags_real
        != expected_lags_four
        && positive_control_allocator
            .observations
            .iter()
            .all(|observation| !observation.reading.root_ring_installed);
    emitter.emit(&format!(
        "name=pc_allocator a10_lags_real={} expected={} root_ring_installed_points={} seen={positive_control_allocator_seen}",
        set_text(&positive_control_allocator.tree_table_lags_real),
        set_text(&expected_lags_four),
        positive_control_allocator
            .observations
            .iter()
            .filter(|observation| observation.reading.root_ring_installed)
            .count()
    ));

    // PC-多扣：每个 (S, ρ) 的 Hh(0, —) 与 Hh(2, 后)，四个组合的期末值都要 > 0。
    let mut over_withheld_controls_void = 0u64;
    for outcome in outcomes.iter().filter(|outcome| {
        outcome.shape.hole_count == 0
            || (outcome.shape.hole_count == 2 && outcome.shape.position == Some(HolePosition::Back))
    }) {
        let last = outcome
            .observations
            .last()
            .map(|observation| observation.reading.device_zero);
        let finals = last.map_or([0u64; 4], |device_zero| {
            [
                device_zero.no_reclaim_floor_rule_real,
                device_zero.no_reclaim_floor_rule_every_publish,
                device_zero.no_reclaim_interval_rule_real,
                device_zero.no_reclaim_interval_rule_every_publish,
            ]
        });
        let void = finals.iter().filter(|value| **value == 0).count();
        over_withheld_controls_void += u64::try_from(void).expect("落在 u64 内");
        emitter.emit(&format!(
            "name=pc_over_withheld {} final_floor_rule_real={} final_floor_rule_every={} final_interval_rule_real={} final_interval_rule_every={} seen={}",
            outcome.shape.fields(),
            finals[0],
            finals[1],
            finals[2],
            finals[3],
            void == 0
        ));
    }

    // PC-洞：h_缺、h_洞 两个计数器改成恒 0，A11 在每一条 k ≥ 1 的族上都要判红。
    let mut hole_controls_not_seen = 0u64;
    for outcome in outcomes
        .iter()
        .filter(|outcome| outcome.shape.hole_count >= 1)
    {
        let red_points = outcome
            .observations
            .iter()
            .filter(|observation| {
                observation
                    .anchor
                    .is_some_and(|row| row.missing_txgs != 0 || row.live_holes != 0)
            })
            .count();
        if red_points == 0 {
            hole_controls_not_seen += 1;
        }
        emitter.emit(&format!(
            "name=pc_hole_counters_zeroed {} a11_red_points={red_points} seen={}",
            outcome.shape.fields(),
            red_points > 0
        ));
    }

    // 分组、判定。
    let mut groups: BTreeMap<GroupKey, Vec<DeltaSample>> = BTreeMap::new();
    for outcome in &outcomes {
        if outcome.truncation.is_some() {
            emitter.emit(&format!(
                "name=q1_cell_truncated {} note=only_points_before_truncation_are_used",
                outcome.shape.fields()
            ));
        }
        for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
            let key = (
                outcome.shape.slots_per_region,
                density_key(outcome.shape.density),
                timing,
            );
            let entry = groups.entry(key).or_default();
            for observation in &outcome.observations {
                entry.push(DeltaSample {
                    shape: outcome.shape,
                    txg: observation.txg,
                    workload_publishes: observation.workload_publishes,
                    missing_txgs: observation.reading.missing_txgs,
                    live_holes: observation.reading.live_holes,
                    delta: observation.delta(timing),
                });
            }
        }
    }
    let mut judgements: BTreeMap<GroupKey, GroupJudgements> = BTreeMap::new();
    for (key, samples) in &groups {
        judgements.insert(*key, judge_group(&mut emitter, key, samples));
    }
    let sensitivity = |pick: &dyn Fn(&GroupJudgements) -> PeakJudgement| {
        let values: Vec<PeakJudgement> = judgements.values().map(pick).collect();
        let listing: Vec<String> = judgements
            .iter()
            .map(|(key, judgement)| {
                format!(
                    "{}:{}",
                    group_label(key).replace(' ', "_"),
                    pick(judgement).label()
                )
            })
            .collect();
        (judge_geometry_sensitivity(&values), listing.join(","))
    };
    for (quantity, pick) in [
        (
            "q1c",
            &(|judgement: &GroupJudgements| judgement.all_points)
                as &dyn Fn(&GroupJudgements) -> PeakJudgement,
        ),
        ("q1c_on_holes", &|judgement: &GroupJudgements| {
            judgement.on_holes
        }),
        ("q1c_front", &|judgement: &GroupJudgements| {
            judgement.front_holes
        }),
        ("q1c_back", &|judgement: &GroupJudgements| {
            judgement.back_holes
        }),
    ] {
        let (result, listing) = sensitivity(pick);
        emitter.emit(&format!(
            "name=geometry_sensitivity quantity={quantity} result={} per_sampling_point={listing}",
            result.label()
        ));
    }
    let growth_listing: Vec<String> = judgements
        .iter()
        .map(|(key, judgement)| {
            format!(
                "{}:{}",
                group_label(key).replace(' ', "_"),
                judgement.growth.label()
            )
        })
        .collect();
    let growth_values: BTreeSet<&str> = judgements
        .values()
        .map(|judgement| judgement.growth.label())
        .collect();
    emitter.emit(&format!(
        "name=geometry_sensitivity quantity=q1d_growth result={} per_sampling_point={}",
        if growth_values.len() == 1 {
            format!("consistent_on_{}_sampling_points", judgements.len())
        } else {
            "unstable".to_string()
        },
        growth_listing.join(",")
    ));

    // 失败条款（第十节）。
    let front_above_two_groups: Vec<String> = judgements
        .iter()
        .filter(|(_, judgement)| judgement.front_holes == PeakJudgement::AboveTheThreshold)
        .map(|(key, _)| group_label(key).replace(' ', "_"))
        .collect();
    let no_observable_difference = judgements.values().all(|judgement| {
        judgement.front_peak_is_zero_everywhere && judgement.back_peak_is_zero_everywhere
    });
    let front_back_differ_groups: Vec<String> = judgements
        .iter()
        .filter(|(_, judgement)| {
            judgement.front_holes != PeakJudgement::NothingObserved
                && judgement.back_holes != PeakJudgement::NothingObserved
                && judgement.front_holes != judgement.back_holes
        })
        .map(|(key, _)| group_label(key).replace(' ', "_"))
        .collect();
    let back_at_most_two_groups: Vec<String> = judgements
        .iter()
        .filter(|(_, judgement)| judgement.back_holes == PeakJudgement::AtMostTheThreshold)
        .map(|(key, _)| group_label(key).replace(' ', "_"))
        .collect();
    let front_one_slot: Vec<String> = judgements
        .iter()
        .map(|(key, judgement)| {
            format!(
                "{}:{}",
                group_label(key).replace(' ', "_"),
                judgement.front_holes_one_slot.label()
            )
        })
        .collect();
    let not_on_holes: Vec<String> = judgements
        .iter()
        .map(|(key, judgement)| {
            format!(
                "{}:{}",
                group_label(key).replace(' ', "_"),
                judgement.not_on_holes.label()
            )
        })
        .collect();
    let list_or_none = |values: &[String]| {
        if values.is_empty() {
            "none".to_string()
        } else {
            values.join(",")
        }
    };
    emitter.emit(&format!(
        "name=failure_clauses f1_front_above_two_groups={} f2_no_observable_difference={no_observable_difference} f22_front_and_back_differ_groups={} f23_back_at_most_two_groups={} f24_real_every_differ_points={} q1c_front_one_slot={} q1c_not_on_holes={}",
        list_or_none(&front_above_two_groups),
        list_or_none(&front_back_differ_groups),
        list_or_none(&back_at_most_two_groups),
        grid_totals.real_every_differ,
        front_one_slot.join(","),
        not_on_holes.join(","),
    ));

    // 作废与停机（第十一节）。
    let cells_run = outcomes.len();
    emitter.emit(&format!(
        "name=verdict stage=second cells_run={cells_run} cells_not_fitting={not_fitting} truncated_cells={} v1_over_withheld_controls_void={over_withheld_controls_void} v2_a10_every_mismatch_cells={every_publish_tree_table_lag_mismatch_cells} v2_a12_every_nonzero_points={} v2_a14_mismatch_points={} v2_a15_every_below_points={} v2_pc_allocator_seen={positive_control_allocator_seen} v2_pc_c2_seen={positive_control_crash_seen} v6_a11_mismatch_points={} v6_devices_unequal_points={} v6_pc_hole_not_seen_cells={hole_controls_not_seen} v16_judge_cases_wrong={judge_wrong} v17_a13_every_nonzero_points={} s1_failures={} s1h_root_ring_missing_points={} s11_a10_real_mismatch_cells={} s11_a12_real_nonzero_points={} s11_a15_real_below_points={} s11_k10_violation_points={} s12_g_adm_nonzero={} k8_mismatches={} referenced_but_free_points={} abandoned_or_unknown_root_points={} anchor_rows_missing={}",
        outcomes.iter().filter(|outcome| outcome.truncation.is_some()).count(),
        grid_totals.every_publish_nonzero_without_holes,
        grid_totals.threshold_anchor_mismatches,
        grid_totals.every_publish_below_tree_table_bound,
        grid_totals.hole_count_anchor_mismatches,
        grid_totals.devices_unequal,
        grid_totals.interval_rule_every_publish_nonzero,
        grid_totals.device_and_crates_disagreements,
        grid_totals.root_ring_missing,
        grid_totals.tree_table_lag_real_mismatch,
        grid_totals.real_nonzero_without_holes,
        grid_totals.real_below_tree_table_bound,
        grid_totals.reclaim_rule_violation_points,
        grid_totals.admission_touches,
        grid_totals.slots_per_region_read_back_mismatches,
        grid_totals.q1a_plus_positive,
        grid_totals.abandoned_or_unknown,
        grid_totals.anchor_rows_missing,
    ));
    for (key, judgement) in &judgements {
        emitter.emit(&format!(
            "name=verdict stage=second_group {} q1c={} q1c_on_holes={} q1c_front={} q1c_back={} q1d_growth={}",
            group_label(key),
            judgement.all_points.label(),
            judgement.on_holes.label(),
            judgement.front_holes.label(),
            judgement.back_holes.label(),
            judgement.growth.label(),
        ));
    }
    emitter.finish();
}

#[cfg(test)]
mod tests {
    //! K1（第一个事务之后的记账三项）、G27（`referenced_slots`）、R8（U7）、R3 的判别力（U8）、R-3 闭式（U10、U12）的最小单测，
    //! 与第 4 次重跑登记第九节的 U11、U13–U18。
    use super::{
        accounting_row_slots, allocated_minus_deferred_matches_referenced,
        allocated_minus_deferred_mismatches_referenced, anchor_cell_for, anchor_cells_of,
        corrupt_device_zero_accounting, e156_allocation_record_tree_new_node_count,
        e156_closed_form_expected, e156_existing_leaf_positions, e156_touched_leaf_positions,
        hh_request, judge_geometry_sensitivity, judge_growth_with_holes,
        judge_peak_against_threshold, judge_positive_control_cases, memory_pool_snapshot,
        parameters, q7c_self_test, referenced_slots, run_forward_rollback_scenario, run_hh_history,
        run_rollback_to_the_oldest_candidate, self_release_slots_of_this_publish,
        synthetic_base_with_zero_deferred, AnchorCellsOfOneSlotsPerRegion, Arm, Emitter,
        GeometrySensitivity, GrowthWithHoles, HhCellShape, HhRunOutcome, HolePosition,
        PeakJudgement, ReclaimTiming, WorkloadDensity, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
    };
    use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
    use singlefs_core::allocator::{
        AllocationRecord, DeviceFreeMap, Placement, PoolAllocator, ReclaimedReuse,
    };
    use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
    use singlefs_core::make_filesystem::{
        make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
    };
    use singlefs_core::transaction::{
        acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
        TransactionUnit,
    };
    use singlefs_format::SLOT_BYTES;
    use singlefs_harness::crash::SparseBlockDevice;
    use std::collections::{BTreeMap, BTreeSet};

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
        // `PoolAllocator::reclaim_released_up_to` 不删记录（`allocator.rs` I8 字面）：一条记录被真实分配器回收之后，
        // `is_released` 仍是 `true`、条目还留在 `records()` 里，直到下一次 `record()` 落在同一个起点槽上才被改写；
        // 但它的槽此刻已经是空闲的（`DeviceFreeMap::is_free`）。岔路 1 的「已释放未回收」按 `is_free` 现查，
        // 这条测试钉住这份读法赖以成立的前提本身。
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

    /// PC-闭式自测第一、二、四组（E156 第 3 次重跑登记「七」7.2 R-3c）：只在合成的叶位置集合上单测
    /// 闭式函数本身，不依赖真实分配器。M27（闭式漏掉根）、M28（闭式每个改动的叶位置只算一块盘）应当
    /// 分别让第一组从 5 变成 4 与 3。第三组的前提（61、170 号叶分落第一、第二个层级 1 节点）先用
    /// `e156_level1_position_of_leaf` 现算钉住，不写成两边都是字面量的比较。
    #[test]
    fn pc_closed_form_matches_the_registered_anchor_r3c() {
        let one_leaf: BTreeSet<u64> = [61].into_iter().collect();
        let two_leaves: BTreeSet<u64> = [61, 62].into_iter().collect();
        let leaves_in_two_internal: BTreeSet<u64> = [61, 170].into_iter().collect();
        assert_eq!(
            super::e156_level1_position_of_leaf(61),
            0,
            "R-3c：61 号叶应落在第一个层级 1 节点里"
        );
        assert_eq!(
            super::e156_level1_position_of_leaf(170),
            1,
            "R-3c：170 号叶应落在第二个层级 1 节点里"
        );
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
            touched,
            BTreeSet::from([61, 170]),
            "PC-闭式第三组：应当同时看到释放在第 61 片、分配在第 170 片两条"
        );
        assert_eq!(
            e156_allocation_record_tree_new_node_count(&touched, 2),
            9,
            "R-3c：第 61 与第 170 片叶应给出 9"
        );
    }

    /// U10（第 3 次重跑登记第九节）：β0 之后连续覆盖写，逐次核对 R-3 的闭式；两个方向（落在 1 片叶 / 落在 ≥ 2 片叶）
    /// 都至少出现一次。前 4 次另外钉 R-4 的绝对值（released_d0=14、record_delta=24）。U12（Q7d-1）：这段历史上逐次现量的
    /// 最小值 14、最大值 ≥ 16——M30（Q7d-1 覆盖写那一组退回字面 10）应当让最小值变成 10。
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

    /// U11（第 4 次重跑登记第一节 1.2、第九节）：A → B → 干净重开 → C → 同一次挂载里挂着回退到 A → 普通重开。
    /// K12：回退那次发布的 txg = C 的 txg + 1、实例代号不变、实例表行数不变；普通重开之后按实例表判被抛弃的根 0 条、每盘隔离 0 槽。
    /// M32（`crates/mutations.tsv`）把期望的隔离改回挂载时回退那一形的 54 槽，这条测试必须红。
    #[test]
    fn rolling_back_by_a_forward_publish_keeps_the_instance_and_isolates_nothing_after_a_plain_remount(
    ) {
        let scenario = run_forward_rollback_scenario();
        assert_eq!(
            scenario.txg_of_the_rollback_publish,
            scenario.txg_before_the_rollback + 1,
            "K12：回退那次发布的 txg = 现行 + 1"
        );
        assert_eq!(
            scenario.instance_after, scenario.instance_before,
            "K12：挂着回退不取号，实例代号不变"
        );
        assert_eq!(
            scenario.instance_table_rows_after, scenario.instance_table_rows_before,
            "K12：挂着回退不写行，实例表行数不变"
        );
        assert_eq!(
            scenario.isolated_after_the_plain_remount,
            vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
            "K12：挂着回退不抛弃任何根，普通重开之后每盘隔离 0 槽"
        );
        assert_eq!(
            scenario.abandoned_roots_after_the_plain_remount, 0,
            "K12：按实例表判被抛弃的根 0 条"
        );
    }

    /// U13 (a)（第 4 次重跑登记第一节 1.2、第九节）：覆盖写 72 次之后在同一个进程里挂着回退到候选集里 txg 最小的根：
    /// K12 的前三样成立。第 5 项只报数（产物 `name=u13a_rollback_to_oldest_candidate`），这里不钉。
    #[test]
    fn rolling_back_to_the_oldest_candidate_after_seventy_two_overwrites_is_one_forward_publish() {
        let rollback = run_rollback_to_the_oldest_candidate();
        assert_eq!(rollback.overwrites, 72, "3N = 3 × 3S = 72（S = 8）");
        assert_eq!(
            rollback.txg_after,
            rollback.txg_before + 1,
            "K12：txg = 现行 + 1"
        );
        assert_eq!(
            rollback.instance_after, rollback.instance_before,
            "K12：实例代号不变"
        );
        assert_eq!(
            rollback.instance_table_rows_after, rollback.instance_table_rows_before,
            "K12：实例表行数不变"
        );
    }

    /// U13 (b)（第 4 次重跑登记第一节 1.2、第九节）：Q7c① 的转色在 β_syn（造出来的基底，第 5 项为 0）上核：`flip1` 为真、`flip2` 为假。
    /// M33、M34（`crates/mutations.tsv`）分别改坏 `q7c_self_test` 里两处比较符号，这条测试必须红。
    #[test]
    fn synthetic_base_with_zero_deferred_flips_the_q7c1_self_test() {
        let base = synthetic_base_with_zero_deferred();
        let (_item1, _item2, item5) =
            super::mirror_accounting_row_slots(&base.pool, base.accounting_slot);
        assert_eq!(item5, 0, "β_syn 的第 5 项为 0（造法：β0 第 5 项 − 1）");
        let mut emitter = Emitter { emitted: 0 };
        let (flip1, flip2) = q7c_self_test(
            &mut emitter,
            "test_beta_syn",
            &base.pool,
            base.accounting_slot,
            base.referenced,
        );
        assert!(
            flip1,
            "U13 (b)：第 5 项为 0 的基底上 Q7c① 必须转色——去掉『减 defer』之后坏镜像由红变绿，G27 仍判红"
        );
        assert!(
            !flip2,
            "U13 (b)：Q7c② 在第 5 项为 0 的基底上不适用，不应转色"
        );
    }

    fn run_cell(
        anchors: &BTreeMap<u64, AnchorCellsOfOneSlotsPerRegion>,
        hole_count: u64,
        position: Option<HolePosition>,
    ) -> HhRunOutcome {
        let shape = HhCellShape {
            slots_per_region: 4,
            density: WorkloadDensity::EveryPublishOverwrites,
            hole_count,
            position,
        };
        let mut emitter = Emitter { emitted: 0 };
        run_hh_history(
            &mut emitter,
            &hh_request(
                "Hh",
                shape,
                anchor_cell_for(anchors, 4, hole_count, position),
            ),
        )
    }

    fn anchors_for_four_slots_per_region() -> BTreeMap<u64, AnchorCellsOfOneSlotsPerRegion> {
        BTreeMap::from([(4, anchor_cells_of(4))])
    }

    fn observation_at(outcome: &HhRunOutcome, txg: u64) -> &super::HhObservation {
        outcome
            .observations
            .iter()
            .find(|observation| observation.txg == txg)
            .unwrap_or_else(|| panic!("txg {txg} 有一个观测点"))
    }

    /// U14（第 4 次重跑登记第九节）：Hh(0, —, S = 4, ρ = 1) 跑到 txg 48：实与每两个时点 A10 的滞后全是 12、A12 每个观测点
    /// Q1a = Q1b = 0、每个观测点分配器装着根环表。M35（分配器退回不装根环表）、M42（「每」的门槛晚一次）在这里红。
    #[test]
    fn hh_without_holes_at_four_slots_per_region_reclaims_each_tree_table_one_ring_length_after_release(
    ) {
        let outcome = run_cell(&anchors_for_four_slots_per_region(), 0, None);
        assert_eq!(outcome.actual_end, 48, "Hh(0, —, S = 4) 跑到 txg 48");
        assert!(outcome.truncation.is_none(), "没有截断");
        assert_eq!(
            outcome.tree_table_lags_real,
            BTreeSet::from([12]),
            "A10（实）：滞后全是 3S = 12"
        );
        assert_eq!(
            outcome.tree_table_lags_every_publish,
            BTreeSet::from([12]),
            "A10（每）：滞后全是 3S = 12"
        );
        for observation in &outcome.observations {
            for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
                assert_eq!(
                    observation.over_withheld(Arm::FloorRule, timing),
                    0,
                    "A12：Q1a = 0，txg {}",
                    observation.txg
                );
                assert_eq!(
                    observation.over_withheld(Arm::LifetimeIntervalRule, timing),
                    0,
                    "A12：Q1b = 0，txg {}",
                    observation.txg
                );
            }
            assert!(
                observation.reading.root_ring_installed,
                "W7：每个观测点装着根环表，txg {}",
                observation.txg
            );
        }
    }

    /// U15（第 4 次重跑登记第九节）：PQ1-后（S = 4）：环里没有 txg 16 的根、槽里是 txg 4 的根；txg 19–27 门槛 = 4、txg 28 门槛 = 17；
    /// txg 19 Q1a ≥ 4、txg 27 Q1a ≥ 12；每个观测点 Q1b = 0；txg 30 h_缺 = 0。M36、M37、M38、M41 在这里红。
    #[test]
    fn hh_back_hole_at_four_slots_per_region_pins_the_threshold_to_txg_four_until_txg_twenty_eight()
    {
        let outcome = run_cell(
            &anchors_for_four_slots_per_region(),
            1,
            Some(HolePosition::Back),
        );
        let recovery = outcome.recoveries.first().expect("造了一个洞");
        assert_eq!(recovery.hole_txg, 16, "洞在 txg 16");
        assert!(
            !recovery.ring_has_a_root_at_the_hole_txg,
            "环里没有 txg 16 的根"
        );
        assert_eq!(
            recovery.hole_slot_content, "root_txg_4",
            "txg 16 的槽里仍是 txg 4 的根"
        );
        for txg in 19..=27u64 {
            assert_eq!(
                observation_at(&outcome, txg).reading.ring_threshold,
                4,
                "A14：txg {txg} 门槛 = 4"
            );
        }
        assert_eq!(
            observation_at(&outcome, 28).reading.ring_threshold,
            17,
            "A14：txg 28 门槛 = 17"
        );
        for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
            assert!(
                observation_at(&outcome, 19).over_withheld(Arm::FloorRule, timing) >= 4,
                "A16：txg 19 Q1a ≥ 4"
            );
            assert!(
                observation_at(&outcome, 27).over_withheld(Arm::FloorRule, timing) >= 12,
                "A15：txg 27 Q1a ≥ 12"
            );
        }
        for observation in &outcome.observations {
            for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
                assert_eq!(
                    observation.over_withheld(Arm::LifetimeIntervalRule, timing),
                    0,
                    "A13：Q1b = 0，txg {}",
                    observation.txg
                );
            }
        }
        assert_eq!(
            observation_at(&outcome, 30).reading.missing_txgs,
            0,
            "txg 30 环里只剩 19–30，h_缺 = 0"
        );
    }

    /// U16（第 4 次重跑登记第九节）：Hh(2, 后, S = 4, ρ = 1)：每个观测点 h_缺、h_洞 等于锚点模型那一格（txg 28–33 的 h_洞 = 1）。M39 在这里红。
    #[test]
    fn hh_two_back_holes_at_four_slots_per_region_match_the_anchor_hole_counts() {
        let outcome = run_cell(
            &anchors_for_four_slots_per_region(),
            2,
            Some(HolePosition::Back),
        );
        assert!(!outcome.observations.is_empty(), "至少一个观测点");
        for observation in &outcome.observations {
            let row = observation.anchor.expect("每个观测点都有锚点那一行");
            assert_eq!(
                observation.reading.missing_txgs, row.missing_txgs,
                "A11：h_缺，txg {}",
                observation.txg
            );
            assert_eq!(
                observation.reading.live_holes, row.live_holes,
                "A11：h_洞，txg {}",
                observation.txg
            );
        }
        for txg in 28..=33u64 {
            assert_eq!(
                observation_at(&outcome, txg).reading.live_holes,
                1,
                "txg {txg} 的 h_洞 = 1"
            );
        }
    }

    /// U17（第 4 次重跑登记第九节）：PQ1-前（S = 4）：txg 7 那一刻 h_缺 = h_洞 = 1、Q1a ≥ 1、Q1b = 0。M40 在这里红。
    #[test]
    fn hh_front_hole_at_four_slots_per_region_over_withholds_one_slot_under_the_floor_rule_only() {
        let outcome = run_cell(
            &anchors_for_four_slots_per_region(),
            1,
            Some(HolePosition::Front),
        );
        let recovery = outcome.recoveries.first().expect("造了一个洞");
        assert_eq!(recovery.hole_txg, 5, "洞在 txg 5");
        assert_eq!(
            recovery.hole_slot_content, "never_written",
            "txg 5 的槽从没写过"
        );
        let observation = observation_at(&outcome, 7);
        assert_eq!(observation.reading.missing_txgs, 1, "txg 7 h_缺 = 1");
        assert_eq!(observation.reading.live_holes, 1, "txg 7 h_洞 = 1");
        for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
            assert!(
                observation.over_withheld(Arm::FloorRule, timing) >= 1,
                "txg 7 Q1a ≥ 1"
            );
            assert_eq!(
                observation.over_withheld(Arm::LifetimeIntervalRule, timing),
                0,
                "txg 7 Q1b = 0"
            );
        }
    }

    /// U18（第 4 次重跑登记第九节，PC-判定器）：判定函数在合成用例上逐条判对。M43（「> 2」写成「≥ 2」）在这里红。
    #[test]
    fn judgement_of_peak_delta_against_threshold_and_geometry_sensitivity_on_synthetic_cases() {
        let peaks =
            |pairs: &[(u64, i64)]| -> BTreeMap<u64, i64> { pairs.iter().copied().collect() };
        assert_eq!(
            judge_peak_against_threshold(&peaks(&[(1, 3)]), 2),
            PeakJudgement::AboveTheThreshold
        );
        assert_eq!(
            judge_peak_against_threshold(&peaks(&[(1, 2)]), 2),
            PeakJudgement::AtMostTheThreshold
        );
        assert_eq!(
            judge_peak_against_threshold(&peaks(&[(1, 1)]), 1),
            PeakJudgement::AtMostTheThreshold
        );
        assert_eq!(
            judge_peak_against_threshold(&peaks(&[(1, 2)]), 1),
            PeakJudgement::AboveTheThreshold
        );
        assert_eq!(
            judge_peak_against_threshold(&peaks(&[]), 2),
            PeakJudgement::NothingObserved
        );
        assert_eq!(
            judge_geometry_sensitivity(&[
                PeakJudgement::AtMostTheThreshold,
                PeakJudgement::AtMostTheThreshold,
                PeakJudgement::AtMostTheThreshold,
                PeakJudgement::AboveTheThreshold,
            ]),
            GeometrySensitivity::Unstable
        );
        assert_eq!(
            judge_geometry_sensitivity(&[PeakJudgement::AtMostTheThreshold; 4]),
            GeometrySensitivity::Consistent { sampling_points: 4 }
        );
        assert_eq!(
            judge_growth_with_holes(&peaks(&[(1, 2), (2, 2)])),
            GrowthWithHoles::IndependentOfHoles
        );
        assert_eq!(
            judge_growth_with_holes(&peaks(&[(1, 1), (2, 3)])),
            GrowthWithHoles::GrowsWithHoles
        );
        assert_eq!(
            judge_growth_with_holes(&peaks(&[(1, 3), (2, 1)])),
            GrowthWithHoles::NotMonotonic
        );
        for (case, actual, expected) in judge_positive_control_cases() {
            assert_eq!(actual, expected, "PC-判定器用例 {case}");
        }
    }
}
