//! E197（扫环胶水内存随盘数）：回答岔路单 `research/prompts/m3-close-c653-forks.md` 的甲（上界）、乙（改法）两行——
//! 在默认环几何（每盘 196 608 个记录槽）上，`singlefs_core::recovery::scan_journal` 的内存峰值随盘数怎么涨，
//! 胶水改成逐盘合并（乙二）之后涨不涨。跑前登记 `research/prompts/e197-preregistration.md`。
//!
// admission: always 每跑一次都在今天这份 crates 上现量一遍内存峰值，上一次跑完不替这一次作保
// run-condition: command python3
//!
//! 一个进程就是一格：`--arm <臂> --devices <盘数> --fill <full|half> --named <点名项数> --repeat <重复号>`。
//! 臂：`baseline`（直接调 `scan_journal`）、`baseline_pc`（收齐之后不释放的阳性对照）、`split`（乙二：核不改、胶水逐盘合并）、
//! `split_pc`（逐盘不释放的阳性对照）、`empty`（只建池、不扫环，cgroup 峰值从各臂里扣掉）、`measurement_pc`（量窗里分配并逐页写 256 MiB，测量路径的对照）。
//! 另有 `--diff-set`（差分集 D1–D4：三份胶水各与 `scan_journal` 逐键逐值比）与 `--checks`（本地常量与 `crates/` 回比、锚点、`size_of`）。
//!
//! 进程内量的是分配器口径（计数分配器按 `Layout` 的字节数记，`realloc` 记新旧之差，计数按线程存：扫环在主线程里跑，单测各在自己的线程里量，互不串）；
//! 判的数是内存包装量到的 cgroup 峰值，由驱动 `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` 从峰值表读，出表脚本 `report.py` 判。
//! 只读驱动 `singlefs-core` 的公开入口与扫环核，不改 `crates/` 里别的任何文件；产物按 `E7RESULT` 行打印到标准输出。

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::time::Instant;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::{
    JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, NamedUnit,
};
use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use singlefs_core::recovery::{scan_journal, PoolReader};
use singlefs_core::root_record::{PinnedRingSlot, UnmountMarker};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    InodeRecordTruncationFieldsOfThePool, SystemConfiguration, SystemImmutableConfiguration,
    SystemImmutableSizes, SystemMutableConfiguration, SystemRuntimeConfiguration,
    SystemRuntimeQuantities,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_replay_prefix::first_copy_of_each_record_key;

// ============================================================================================
// 本地常量（登记第五节）：值抄自下面各行点名的 kb 登记位，不从 `crates/` 引；`constants_against_the_implementation` 逐个回比。
// ============================================================================================

/// 默认环长 768 MiB：`.claude/kb/layout/01-first-txn.md:134`（`format-const: JOURNAL_RING_DEFAULT_BYTES = 805306368`）与
/// `.claude/kb/decisions/23-journal的角色与格式.md:536`（D23（journal 的角色与格式） 已定项 19 ③）。
const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// 记录尺寸 4 KiB 定长：`.claude/kb/decisions/23-journal的角色与格式.md:317`（D23（journal 的角色与格式） 已定项 12）。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// journal 环起点的 16 KiB 槽号：`.claude/kb/layout/01-first-txn.md:133`。
const JOURNAL_RING_START_SLOT: u64 = 1024;
/// 槽宽 16 KiB（槽号 = 设备内偏移 ÷ 16384）：`.claude/kb/layout/01-first-txn.md:37`。
const SLOT_BYTES: u64 = 16384;
/// 一条记录装得下的最多点名项：`.claude/kb/decisions/23-journal的角色与格式.md:321`（`format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD = 64`）。
const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 64;
/// 预想镜像 4 GiB：`.claude/kb/layout/01-first-txn.md:48`（「镜像 4 GiB」）与 `.claude/kb/decisions/23-journal的角色与格式.md:540`（D23（journal 的角色与格式） 已定项 19 射程）。
const TEST_IMAGE_DEFAULT_BYTES: u64 = 4_294_967_296;

/// 默认环每盘的记录槽数（登记第七节 C1、A1）：805 306 368 ÷ 4096。
const RING_SLOTS: u64 = 196_608;
/// 登记第七节 A8：measurement_pc 量窗里分配并逐页写的字节数，与峰值落点的上沿余量。
const MEASUREMENT_POSITIVE_CONTROL_BYTES: u64 = 268_435_456;
#[cfg(test)]
const MEASUREMENT_POSITIVE_CONTROL_SLACK_BYTES: u64 = 65_536;
/// 登记第七节 A7：分配器峰值与推算值的容差；`predicted_allocator_peak_bytes` 的 B 树每叶留的键数（升序插入、标准库分裂点）。
#[cfg(test)]
const PREDICTION_TOLERANCE: f64 = 0.05;
#[cfg(test)]
const ASCENDING_INSERT_KEYS_PER_TREE_LEAF: f64 = 6.0;
/// 装置造的每条记录的实例代号；系统配置的 fsid（16 字节，低 8 字节就是记录头里那 8 字节）。
const PRODUCTION_INSTANCE: u32 = 1;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"E197-scan-glue!!";

/// 登记第六节：每格 `--repeat` 取 1–5（第一轮）与 6–10（作废重跑的第二轮）。
const LAST_REPEAT_NUMBER: u32 = 10;

// ============================================================================================
// 计数的全局分配器（口径同 E183 `CountingAllocator`：按 `Layout` 字节记、不含 malloc 块头，`realloc` 记新旧之差）。
// 计数按线程存（const 初始化、无析构的线程局部量，分配器里读写它不会再分配）：扫环单线程，单测各在自己的线程里量，别的用例的分配串不进来。
// ============================================================================================

thread_local! {
    static IN_USE_BYTES: Cell<i64> = const { Cell::new(0) };
    static PEAK_BYTES: Cell<i64> = const { Cell::new(0) };
}

fn record_growth(bytes: u64) {
    let delta = i64::try_from(bytes).unwrap_or(i64::MAX);
    let _ = IN_USE_BYTES.try_with(|in_use| {
        let now = in_use.get().saturating_add(delta);
        in_use.set(now);
        let _ = PEAK_BYTES.try_with(|peak| {
            if now > peak.get() {
                peak.set(now);
            }
        });
    });
}

fn record_shrink(bytes: u64) {
    let delta = i64::try_from(bytes).unwrap_or(i64::MAX);
    let _ = IN_USE_BYTES.try_with(|in_use| in_use.set(in_use.get().saturating_sub(delta)));
}

fn record_resize(old_bytes: u64, new_bytes: u64) {
    if new_bytes >= old_bytes {
        record_growth(new_bytes - old_bytes);
    } else {
        record_shrink(old_bytes - new_bytes);
    }
}

/// 此刻本线程的在用字节（相对进程开头的累计，可为负：别的线程分的内存在这里放掉时）。
fn in_use_bytes() -> i64 {
    IN_USE_BYTES.try_with(Cell::get).unwrap_or(0)
}

/// 本线程出现过的最大在用字节；先占后放之后读它还是那个最大值。
fn peak_bytes() -> i64 {
    PEAK_BYTES.try_with(Cell::get).unwrap_or(0)
}

/// 把峰值重置成此刻的在用字节：一个量窗从这里开始。
fn reset_peak_to_in_use() {
    let now = in_use_bytes();
    let _ = PEAK_BYTES.try_with(|peak| peak.set(now));
}

fn bytes_of_layout(layout: Layout) -> u64 {
    u64::try_from(layout.size()).unwrap_or(u64::MAX)
}

pub struct CountingAllocator;

// SAFETY: 每个方法原样转给 `System`，只在成功之后改线程局部计数；计数不碰被分配的内存，也不再分配。
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: 调用方保证 `layout` 合法；转给系统分配器，约束不变。
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record_growth(bytes_of_layout(layout));
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: 同 `alloc`。
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record_growth(bytes_of_layout(layout));
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: 调用方保证 `pointer` 是用同一个 `layout` 从这个分配器分出来的；它来自 `System`。
        unsafe { System.dealloc(pointer, layout) };
        record_shrink(bytes_of_layout(layout));
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: 调用方保证 `pointer`、`layout`、`new_size` 满足 `GlobalAlloc::realloc` 的契约；转给系统分配器。
        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        if !new_pointer.is_null() {
            let old_bytes = bytes_of_layout(layout);
            let new_bytes = u64::try_from(new_size).unwrap_or(u64::MAX);
            record_resize(old_bytes, new_bytes);
        }
        new_pointer
    }
}

#[global_allocator]
static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator;

// ============================================================================================
// 本地常量回比、几何锚点（登记第七节 A1–A6）、分配器峰值的推算（A7）。
// ============================================================================================

/// 本地常量与 `crates/` 的同名常量逐个回比（登记第五节）；每个进程开头调一次，单测同样调。
fn constants_against_the_implementation() {
    assert_eq!(
        JOURNAL_RING_DEFAULT_BYTES,
        singlefs_format::JOURNAL_RING_DEFAULT_BYTES
    );
    assert_eq!(JOURNAL_RECORD_BYTES, singlefs_format::JOURNAL_RECORD_BYTES);
    assert_eq!(
        JOURNAL_RING_START_SLOT,
        singlefs_format::JOURNAL_RING_START_SLOT
    );
    assert_eq!(SLOT_BYTES, singlefs_format::SLOT_BYTES);
    assert_eq!(
        JOURNAL_NAMED_ENTRIES_PER_RECORD,
        singlefs_format::JOURNAL_NAMED_ENTRIES_PER_RECORD
    );
    assert_eq!(
        TEST_IMAGE_DEFAULT_BYTES,
        singlefs_format::TEST_IMAGE_DEFAULT_BYTES
    );
    // C1：默认环每盘 196 608 个记录槽，除得尽（A1）。
    assert_eq!(
        RING_SLOTS,
        JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES
    );
    assert_eq!(JOURNAL_RING_DEFAULT_BYTES % JOURNAL_RECORD_BYTES, 0);
}

/// A4：环 ≤ 设备容量 ÷ 4（D23 已定项 19 ③），且环末端不越过设备末尾，`scan_journal` 里 `ring_bytes_inside_the_device` 才取满环长。
fn device_holds_the_ring(device_bytes: u64, ring_bytes: u64) -> bool {
    ring_bytes <= device_bytes / 4
        && JOURNAL_RING_START_SLOT * SLOT_BYTES + ring_bytes <= device_bytes
}

/// A5：`Vec::new()` 加逐条 push 到 `count` 条时的容量（元素不大于 1024 字节时最小 4，每次翻倍）。
fn vec_capacity_after_pushes(count: u64) -> u64 {
    let mut capacity = 4;
    while capacity < count {
        capacity *= 2;
    }
    capacity
}

/// A6：四个结构的 `size_of`，打进每一格的结果行，单测钉住。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StructureSizes {
    location_entry: u64,
    named_unit: u64,
    node_pointer: u64,
    journal_record: u64,
}

fn structure_sizes() -> StructureSizes {
    let size = |bytes: usize| u64::try_from(bytes).expect("结构大小装得进 u64");
    StructureSizes {
        location_entry: size(std::mem::size_of::<LocationEntry>()),
        named_unit: size(std::mem::size_of::<NamedUnit>()),
        node_pointer: size(std::mem::size_of::<NodePointer>()),
        journal_record: size(std::mem::size_of::<JournalRecord>()),
    }
}

/// 臂的名字、是否扫环。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Empty,
    MeasurementPc,
    Baseline,
    BaselinePc,
    Split,
    SplitPc,
}

const ALL_ARMS: [Arm; 6] = [
    Arm::Empty,
    Arm::MeasurementPc,
    Arm::Baseline,
    Arm::BaselinePc,
    Arm::Split,
    Arm::SplitPc,
];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Empty => "empty",
            Arm::MeasurementPc => "measurement_pc",
            Arm::Baseline => "baseline",
            Arm::BaselinePc => "baseline_pc",
            Arm::Split => "split",
            Arm::SplitPc => "split_pc",
        }
    }

    fn from_name(name: &str) -> Option<Arm> {
        ALL_ARMS.into_iter().find(|arm| arm.name() == name)
    }

    /// 扫环的四个臂；空跑与测量路径对照不扫环。
    fn scans(self) -> bool {
        matches!(
            self,
            Arm::Baseline | Arm::BaselinePc | Arm::Split | Arm::SplitPc
        )
    }
}

/// 记录槽填满的两档：每盘全部槽，或前一半槽（别处从没写过，全 0）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fill {
    Full,
    Half,
}

impl Fill {
    fn name(self) -> &'static str {
        match self {
            Fill::Full => "full",
            Fill::Half => "half",
        }
    }

    fn from_name(name: &str) -> Option<Fill> {
        [Fill::Full, Fill::Half]
            .into_iter()
            .find(|fill| fill.name() == name)
    }

    fn filled_slots_per_device(self, ring_slots: u64) -> u64 {
        match self {
            Fill::Full => ring_slots,
            Fill::Half => ring_slots / 2,
        }
    }
}

/// A7：每格分配器峰值减起点的推算（登记第四节 `predict.py` 的式子，按量到的 `size_of` 代入）。
/// 基线：Vec 容量 × 记录大小 + 全部点名项堆 + 键表 + 两张 B 树（升序插入每叶留 6 个键）；
/// baseline_pc 多一份首份的点名项堆；split 取第三盘起的那一刻（一张表 + 一盘的 Vec 与点名项 + 键表 + 全局记录表）；
/// split_pc 登记写「同基线」，装置按它的代码多算两个 Vec 的容量：留到交回之前的重复记录 Vec 与最后一盘那一遍的 Vec，代替基线那一个总 Vec
/// （两式在 1、2 盘上相等，4 盘上差约 2.4%，3 盘上 K = 64 差约 −3%、K = 1 差约 −16%）。
#[cfg(test)]
fn predicted_allocator_peak_bytes(
    arm: Arm,
    devices: u64,
    filled_slots: u64,
    named_per_record: u64,
    sizes: StructureSizes,
) -> f64 {
    let as_float = |value: u64| value as f64;
    let record = as_float(sizes.journal_record);
    let named = as_float(sizes.named_unit);
    let leaf_of_records = 8.0 + 2.0 + 2.0 + 4.0 + 11.0 * 16.0 + 11.0 * record;
    let leaf_of_positions = 8.0 + 2.0 + 2.0 + 4.0 + 11.0 * 16.0 + 11.0 * 8.0;
    let unique = as_float(filled_slots);
    let named_entries = as_float(named_per_record);
    let tree_of_records = unique * leaf_of_records / ASCENDING_INSERT_KEYS_PER_TREE_LEAF;
    let tree_of_positions = unique * leaf_of_positions / ASCENDING_INSERT_KEYS_PER_TREE_LEAF;
    let base = |device_count: u64| {
        let read_records = device_count * filled_slots;
        as_float(vec_capacity_after_pushes(read_records)) * record
            + as_float(read_records) * named_entries * named
            + as_float(read_records) * 16.0
            + tree_of_positions
            + tree_of_records
    };
    match arm {
        Arm::Baseline => base(devices),
        Arm::SplitPc => {
            let duplicates = (devices - 1) * filled_slots;
            let duplicates_capacity = if duplicates == 0 {
                0
            } else {
                vec_capacity_after_pushes(duplicates)
            };
            tree_of_records
                + as_float(devices * filled_slots) * named_entries * named
                + as_float(duplicates_capacity) * record
                + as_float(vec_capacity_after_pushes(filled_slots)) * record
                + as_float(devices * filled_slots) * 16.0
                + tree_of_positions
        }
        Arm::BaselinePc => base(devices) + unique * named_entries * named,
        Arm::Split => {
            if devices == 1 {
                base(1)
            } else {
                tree_of_records
                    + unique * named_entries * named
                    + as_float(vec_capacity_after_pushes(filled_slots)) * record
                    + unique * named_entries * named
                    + as_float(devices * filled_slots) * 16.0
                    + tree_of_positions
            }
        }
        Arm::Empty | Arm::MeasurementPc => 0.0,
    }
}

// ============================================================================================
// 造记录与系统配置（登记第五节「装置的池」）。记录由 `JournalRecord::to_bytes` 现编，过 `parse` 往返。
// ============================================================================================

/// 一条过得了 `to_bytes` / `parse` 往返的节点指针：两条位置条目分在 0、1 号盘同一个槽。
fn node_pointer_at_slot(slot: u64, birth_txg: u64) -> NodePointer {
    let location_on = |device: u32| LocationEntry {
        device: DeviceIdentity(device),
        slot: SlotNumber(slot),
        unit_checksum: u32::try_from(slot % 0xffff_fff1).expect("取过模，装得进 32 位"),
    };
    NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(11),
            birth_txg: CheckpointTxg(birth_txg),
        },
        locations: [location_on(0), location_on(1)],
        instance: InstanceGeneration(PRODUCTION_INSTANCE),
        birth_sequence: BirthSequence(1),
    }
}

/// 计数器 `counter`、事务号 `transaction`、`named_count` 个点名项的一条记录；同一参数造出同一条记录（每块盘同一槽字节相同）。
fn journal_record_for(
    instance: u32,
    counter: u64,
    transaction: u64,
    named_count: usize,
) -> JournalRecord {
    let named = (0..named_count)
        .map(|index| {
            let slot = 100_000 + counter * 64 + u64::try_from(index).expect("下标装得进 u64");
            let location_on = |device: u32| LocationEntry {
                device: DeviceIdentity(device),
                slot: SlotNumber(slot),
                unit_checksum: u32::try_from(slot % 0xffff_fff1).expect("取过模，装得进 32 位"),
            };
            NamedUnit {
                locations: [location_on(0), location_on(1)],
                unit_class: 3,
                birth_tree: TreeIdentifier(11),
                birth_txg: CheckpointTxg(counter),
                key_tail: [u8::try_from(index % 251).expect("取过模 251"); 10],
            }
        })
        .collect();
    JournalRecord {
        instance: InstanceGeneration(instance),
        counter,
        checkpoint_txg: CheckpointTxg(counter),
        transaction,
        is_commit: true,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
        back_chain: 0,
        filesystem_identifier: unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER),
        new_tree_table: node_pointer_at_slot(70_000 + counter, counter),
        new_mapping_root: node_pointer_at_slot(80_000 + counter, counter),
        new_tree_identifier_watermark: 1000 + counter,
        new_rollback_floor: CheckpointTxg(10 + counter),
        new_instance_table: node_pointer_at_slot(60_000, 1),
        new_allocation_record_tree_root: node_pointer_at_slot(60_001, 1),
        new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        new_reclaim_threshold: CheckpointTxg(0),
        new_pinned_ring_slot: PinnedRingSlot::NO_SLOT_PINNED,
        named,
    }
}

/// 喂给 `scan_journal` 的系统配置：fsid 取 [`FILESYSTEM_IDENTIFIER`]，journal 环长取 `ring_bytes`，其余字段 `scan_journal` 不读（值随便取，只求造得出结构体）。
fn system_configuration_for(ring_bytes: u64, device_count: u32) -> SystemConfiguration {
    SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: FILESYSTEM_IDENTIFIER,
            this_device: DeviceIdentity(0),
            device_count,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            sizes: SystemImmutableSizes {
                physical_block_size: 4096,
                minimum_input_output_bytes: 4096,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: ring_bytes,
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
            journal_ring_start_slot: SlotNumber(JOURNAL_RING_START_SLOT),
            root_ring_base_slot: SlotNumber(0),
            inode_record_truncation_fields:
                InodeRecordTruncationFieldsOfThePool::WRITTEN_BY_THIS_WRITER,
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: 1,
            journal_tail: 0,
            journal_instance: InstanceGeneration(PRODUCTION_INSTANCE),
            rollback_floor: CheckpointTxg(0),
        },
    }
}

// ============================================================================================
// 装置的池：只按 (盘, 偏移) 现算字节、不存镜像、不缓存任何一槽的字节（登记第五节「装置的池」）。
// ============================================================================================

/// 某块盘（盘次序里的下标）的某个环内槽上放着哪条记录；`None` 是从没写过的全 0 槽。
type SlotSource = Box<dyn Fn(u32, u64) -> Option<JournalRecord>>;

fn slot_source_of(source: impl Fn(u32, u64) -> Option<JournalRecord> + 'static) -> SlotSource {
    Box::new(source)
}

/// 各臂共用的生产槽：每块盘同一槽字节相同；前 `filled_slots` 个槽是计数器 = 槽号 + 1、带 `named_count` 个点名项的记录，别处全 0。
fn production_slot_source(filled_slots: u64, named_count: usize) -> SlotSource {
    Box::new(move |_device, slot| {
        if slot < filled_slots {
            Some(journal_record_for(
                PRODUCTION_INSTANCE,
                slot + 1,
                slot + 1,
                named_count,
            ))
        } else {
            None
        }
    })
}

struct GeneratedPool {
    device_count: u32,
    device_bytes: u64,
    ring_slots: u64,
    slot_source: SlotSource,
    /// `read` 被调了几次。
    reads: Cell<u64>,
    /// 轨迹（登记第八节）：每块盘最后一个环内槽被读到那一刻分配器的在用字节减 `trace_start`。
    trace_start: Cell<i64>,
    live_at_each_device_end: RefCell<Vec<i64>>,
}

impl GeneratedPool {
    fn new(device_count: u32, device_bytes: u64, ring_slots: u64, slot_source: SlotSource) -> Self {
        GeneratedPool {
            device_count,
            device_bytes,
            ring_slots,
            slot_source,
            reads: Cell::new(0),
            trace_start: Cell::new(0),
            live_at_each_device_end: RefCell::new(Vec::with_capacity(
                usize::try_from(device_count).expect("盘数装得进 usize"),
            )),
        }
    }
}

impl PoolReader for GeneratedPool {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        (0..self.device_count).map(DeviceIdentity).collect()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        (device.0 < self.device_count).then_some(self.device_bytes)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reads.set(self.reads.get() + 1);
        let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        if device.0 >= self.device_count || offset.0 < ring_start {
            return None;
        }
        let offset_in_ring = offset.0 - ring_start;
        let is_a_whole_record_slot = offset_in_ring.is_multiple_of(JOURNAL_RECORD_BYTES)
            && u64::try_from(length).ok() == Some(JOURNAL_RECORD_BYTES);
        let slot = offset_in_ring / JOURNAL_RECORD_BYTES;
        if !is_a_whole_record_slot || slot >= self.ring_slots {
            return None;
        }
        if slot + 1 == self.ring_slots {
            self.live_at_each_device_end
                .borrow_mut()
                .push(in_use_bytes() - self.trace_start.get());
        }
        match (self.slot_source)(device.0, slot) {
            Some(record) => Some(record.to_bytes()),
            None => Some(vec![0u8; length]),
        }
    }

    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        // 提示交 None：扫整环（与文件后端的真盘同，登记第五节）。
        None
    }
}

// ============================================================================================
// 胶水的各臂（登记第五节）。基线臂直接调 `scan_journal`；其余三份胶水照 `crates/singlefs-core/src/recovery.rs` 的 `scan_journal`
// 函数体（S1 钉了它的 sha256）抄进来，只改登记写明的那一处；建表的核一律是不改的 `first_copy_of_each_record_key`。
// ============================================================================================

type RecordTable = BTreeMap<(InstanceGeneration, u64), JournalRecord>;

/// `scan_journal` 开头从系统配置里取的三样。
struct ScanSetup {
    expected_filesystem_identifier: u64,
    ring_start: DeviceOffsetInBytes,
    ring_bytes: u64,
}

fn scan_setup(system_configuration: &SystemConfiguration) -> ScanSetup {
    ScanSetup {
        expected_filesystem_identifier: unit_filesystem_identifier(
            &system_configuration.immutable.filesystem_identifier,
        ),
        ring_start: DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES),
        ring_bytes: system_configuration.immutable.sizes.journal_ring_bytes,
    }
}

fn key_of(record: &JournalRecord) -> (u32, u64) {
    (record.instance.0, record.counter)
}

/// `scan_journal` 里「对一块盘逐槽列偏移、逐槽读、解得过的记录压进 Vec」那一截，逐句照抄（`recovery.rs` 4436–4467 行）。
fn read_the_records_of_one_device(
    reader: &dyn PoolReader,
    device: DeviceIdentity,
    setup: &ScanSetup,
    destination: &mut Vec<JournalRecord>,
) {
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_bytes_inside_the_device =
        reader
            .device_size_in_bytes(device)
            .map_or(0, |device_bytes| {
                setup
                    .ring_bytes
                    .min(device_bytes.saturating_sub(setup.ring_start.0))
            });
    let offsets = reader
        .journal_record_offsets_hint(device, setup.ring_start, setup.ring_bytes)
        .unwrap_or_else(|| {
            (0..ring_bytes_inside_the_device / JOURNAL_RECORD_BYTES)
                .map(|record_index| {
                    DeviceOffsetInBytes(setup.ring_start.0 + record_index * JOURNAL_RECORD_BYTES)
                })
                .collect()
        });
    for offset in offsets {
        let Some(bytes) = reader.read(device, offset, record_bytes) else {
            continue;
        };
        if let Some(record) = JournalRecord::parse(&bytes, setup.expected_filesystem_identifier) {
            destination.push(record);
        }
    }
}

/// baseline_pc：与 `scan_journal` 同一份胶水，只改最后一步——不 `into_iter` move，按引用遍历、把首份 `clone()` 进表；读到的 Vec 留到交回之前才放。
fn scan_journal_keeping_the_scan_until_return(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> RecordTable {
    let setup = scan_setup(system_configuration);
    let mut scanned_records_in_read_order: Vec<JournalRecord> = Vec::new();
    for device in reader.device_identities() {
        read_the_records_of_one_device(reader, device, &setup, &mut scanned_records_in_read_order);
    }
    let scanned_record_keys: Vec<(u32, u64)> =
        scanned_records_in_read_order.iter().map(key_of).collect();
    let first_copies = first_copy_of_each_record_key(&scanned_record_keys);
    let mut records = BTreeMap::new();
    for (position, record) in scanned_records_in_read_order.iter().enumerate() {
        if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {
            records.insert((record.instance, record.counter), record.clone());
        }
    }
    std::hint::black_box(&scanned_records_in_read_order);
    records
}

/// split（乙二）：核不改、胶水逐盘合并。每读完一盘：这一盘的键接到全局键表后面；先放掉上一盘调核交回的表，再拿到此为止的全部键调核；
/// 这一盘的 Vec 里位置等于表里那一份的 move 进 `records`，别的丢掉；这一盘的 Vec 放掉，再读下一盘。
fn scan_journal_merging_device_by_device(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> RecordTable {
    let setup = scan_setup(system_configuration);
    let mut all_keys_so_far: Vec<(u32, u64)> = Vec::new();
    let mut records = BTreeMap::new();
    let mut records_before_this_device = 0usize;
    let mut previous_first_copies: Option<BTreeMap<(u32, u64), usize>> = None;
    for device in reader.device_identities() {
        let mut this_device: Vec<JournalRecord> = Vec::new();
        read_the_records_of_one_device(reader, device, &setup, &mut this_device);
        all_keys_so_far.extend(this_device.iter().map(key_of));
        drop(previous_first_copies.take());
        // 核看到的键：从全局键表的第 0 个起，即到此为止的全部键（登记第九节 M7 把它改成只从这一盘的第一个键起）。
        let first_key_given_to_the_core = 0usize;
        let first_copies =
            first_copy_of_each_record_key(&all_keys_so_far[first_key_given_to_the_core..]);
        for (index, record) in this_device.drain(..).enumerate() {
            let position = records_before_this_device + index - first_key_given_to_the_core;
            if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {
                records.insert((record.instance, record.counter), record);
            }
        }
        drop(this_device);
        records_before_this_device = all_keys_so_far.len();
        previous_first_copies = Some(first_copies);
    }
    records
}

/// split_pc：与 split 同，只改非首份的去处——不丢，push 进一个留到交回之前的 Vec（每盘的非首份连同点名项堆一直留着）。
fn scan_journal_merging_device_by_device_keeping_the_duplicates(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> RecordTable {
    let setup = scan_setup(system_configuration);
    let mut all_keys_so_far: Vec<(u32, u64)> = Vec::new();
    let mut records = BTreeMap::new();
    let mut duplicates_kept_until_return: Vec<JournalRecord> = Vec::new();
    let mut records_before_this_device = 0usize;
    let mut previous_first_copies: Option<BTreeMap<(u32, u64), usize>> = None;
    for device in reader.device_identities() {
        let mut this_device: Vec<JournalRecord> = Vec::new();
        read_the_records_of_one_device(reader, device, &setup, &mut this_device);
        all_keys_so_far.extend(this_device.iter().map(key_of));
        drop(previous_first_copies.take());
        let first_copies = first_copy_of_each_record_key(&all_keys_so_far);
        for (index, record) in this_device.drain(..).enumerate() {
            let position = records_before_this_device + index;
            if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {
                records.insert((record.instance, record.counter), record);
            } else {
                duplicates_kept_until_return.push(record);
            }
        }
        records_before_this_device = all_keys_so_far.len();
        previous_first_copies = Some(first_copies);
    }
    std::hint::black_box(&duplicates_kept_until_return);
    records
}

/// 一张表的摘要（登记第六节 S2 在生产格上的比法）：按键序把 (实例代号, 计数器, 整条记录 4096 字节编码的 CRC-32C) 折进一个 64 位数。
/// 生产格里不在同一个进程里再跑一遍 `scan_journal`（那会把峰值翻倍），各臂的摘要在出表时与基线臂同格的摘要比；差分集 D 用 `==` 逐键逐值比。
fn table_digest(table: &RecordTable) -> String {
    let mut state: u64 = 0xcbf2_9ce4_8422_2325;
    for ((instance, counter), record) in table {
        let encoding_checksum = crc32_castagnoli(&record.to_bytes());
        for word in [
            u64::from(instance.0),
            *counter,
            u64::from(encoding_checksum),
        ] {
            state = (state ^ word)
                .wrapping_mul(0x0000_0100_0000_01b3)
                .rotate_left(31);
        }
    }
    let length = u64::try_from(table.len()).expect("表长装得进 u64");
    format!("{:016x}", state ^ length)
}

/// 两张表逐键逐值相同（`JournalRecord` 派生了 `PartialEq`）。
fn tables_are_equal(left: &RecordTable, right: &RecordTable) -> bool {
    left == right
}

// ============================================================================================
// 一格：建池、量窗、跑臂、打结果行（登记第六节「量法」「结果行」）。
// ============================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CellSpecification {
    arm: Arm,
    devices: u32,
    fill: Fill,
    named: usize,
    repeat: u32,
}

#[derive(Clone, Debug)]
struct CellOutcome {
    specification_text: String,
    ring_slots: u64,
    filled_slots: u64,
    reads: u64,
    reads_expected: u64,
    records_returned: u64,
    records_expected: u64,
    allocator_peak_over_start_bytes: i64,
    allocator_live_at_each_device_end_bytes: Vec<i64>,
    allocator_live_after_return_bytes: i64,
    vm_hwm_kib: Option<u64>,
    scan_seconds: f64,
    table_digest: String,
}

impl CellOutcome {
    /// 读数 A2（读者被调 `read` 的次数 = 盘数 × 环槽数，不扫环的格是 0）与 A3（交回的表的键数 = 每盘填满槽数，不扫环的格是 0）都对得上。
    fn anchors_hold(&self) -> bool {
        self.reads == self.reads_expected && self.records_returned == self.records_expected
    }

    fn result_line(&self, sizes: StructureSizes) -> String {
        let live_at_device_ends = if self.allocator_live_at_each_device_end_bytes.is_empty() {
            "none".to_string()
        } else {
            self.allocator_live_at_each_device_end_bytes
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };
        let vm_hwm = self
            .vm_hwm_kib
            .map_or_else(|| "unreadable".to_string(), |kib| kib.to_string());
        format!(
            "E7RESULT name=e197 {} ring_slots={} filled_slots={} reads={} reads_expected={} records_returned={} records_expected={} anchors_hold={} allocator_peak_over_start_bytes={} allocator_live_at_each_device_end_bytes={} allocator_live_after_return_bytes={} vm_hwm_kib={} scan_seconds={:.6} table_digest={} size_location_entry={} size_named_unit={} size_node_pointer={} size_journal_record={}",
            self.specification_text,
            self.ring_slots,
            self.filled_slots,
            self.reads,
            self.reads_expected,
            self.records_returned,
            self.records_expected,
            self.anchors_hold(),
            self.allocator_peak_over_start_bytes,
            live_at_device_ends,
            self.allocator_live_after_return_bytes,
            vm_hwm,
            self.scan_seconds,
            self.table_digest,
            sizes.location_entry,
            sizes.named_unit,
            sizes.node_pointer,
            sizes.journal_record,
        )
    }
}

/// 交回之后、放掉交回值之前读 `/proc/self/status` 的 `VmHWM`（KiB）；读不到交 `None`（出表脚本当缺字段，不当 0）。
fn vm_hwm_kib() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

/// measurement_pc（登记第五节、第七节 A8）：量窗里分配并逐页写一份 256 MiB 的 `Vec<u8>`，交回分配器峰值减起点。
fn measure_the_positive_control_allocation() -> i64 {
    let start = in_use_bytes();
    reset_peak_to_in_use();
    let mut buffer =
        vec![0u8; usize::try_from(MEASUREMENT_POSITIVE_CONTROL_BYTES).expect("256 MiB")];
    for page_start in (0..buffer.len()).step_by(4096) {
        buffer[page_start] = 0xA5;
    }
    std::hint::black_box(&buffer);
    let peak_over_start = peak_bytes() - start;
    drop(buffer);
    peak_over_start
}

/// 登记第七节 A8：measurement_pc 的分配器峰值减起点落在 [256 MiB, 256 MiB + 64 KiB]。
#[cfg(test)]
fn measurement_peak_is_in_the_registered_window(peak_over_start: i64) -> bool {
    let low = i64::try_from(MEASUREMENT_POSITIVE_CONTROL_BYTES).expect("256 MiB");
    let high = i64::try_from(
        MEASUREMENT_POSITIVE_CONTROL_BYTES + MEASUREMENT_POSITIVE_CONTROL_SLACK_BYTES,
    )
    .expect("256 MiB 加余量");
    peak_over_start >= low && peak_over_start <= high
}

fn run_the_arm_over_the_pool(
    arm: Arm,
    pool: &GeneratedPool,
    system_configuration: &SystemConfiguration,
) -> RecordTable {
    match arm {
        Arm::Baseline => scan_journal(pool, system_configuration),
        Arm::BaselinePc => scan_journal_keeping_the_scan_until_return(pool, system_configuration),
        Arm::Split => scan_journal_merging_device_by_device(pool, system_configuration),
        Arm::SplitPc => {
            scan_journal_merging_device_by_device_keeping_the_duplicates(pool, system_configuration)
        }
        Arm::Empty | Arm::MeasurementPc => RecordTable::new(),
    }
}

/// 跑一格。`ring_slots` 生产取 [`RING_SLOTS`]，单测取小环。池与系统配置在量窗之前建好；
/// 空跑格还编一条记录（让编码那段代码进内存），量窗里只有扫环那一截。
fn measure_one_cell_in_one_process(
    specification: CellSpecification,
    ring_slots: u64,
) -> CellOutcome {
    let ring_bytes = ring_slots * JOURNAL_RECORD_BYTES;
    let filled_slots = specification.fill.filled_slots_per_device(ring_slots);
    let specification_text = format!(
        "arm={} devices={} fill={} named={} repeat={}",
        specification.arm.name(),
        specification.devices,
        specification.fill.name(),
        specification.named,
        specification.repeat
    );
    let system_configuration = system_configuration_for(ring_bytes, specification.devices);
    let pool = GeneratedPool::new(
        specification.devices,
        TEST_IMAGE_DEFAULT_BYTES,
        ring_slots,
        production_slot_source(filled_slots, specification.named),
    );
    let probe = journal_record_for(PRODUCTION_INSTANCE, 1, 1, specification.named).to_bytes();
    assert!(
        JournalRecord::parse(&probe, unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER)).is_some(),
        "装置造的记录过不了 JournalRecord::parse"
    );
    drop(probe);
    let scans = specification.arm.scans();
    let devices = u64::from(specification.devices);
    let mut outcome = CellOutcome {
        specification_text,
        ring_slots,
        filled_slots,
        reads: 0,
        reads_expected: if scans { devices * ring_slots } else { 0 },
        records_returned: 0,
        records_expected: if scans { filled_slots } else { 0 },
        allocator_peak_over_start_bytes: 0,
        allocator_live_at_each_device_end_bytes: Vec::new(),
        allocator_live_after_return_bytes: 0,
        vm_hwm_kib: None,
        scan_seconds: 0.0,
        table_digest: "none".to_string(),
    };
    if specification.arm == Arm::MeasurementPc {
        outcome.allocator_peak_over_start_bytes = measure_the_positive_control_allocation();
        outcome.vm_hwm_kib = vm_hwm_kib();
        return outcome;
    }
    if !scans {
        outcome.vm_hwm_kib = vm_hwm_kib();
        return outcome;
    }
    let start = in_use_bytes();
    pool.trace_start.set(start);
    reset_peak_to_in_use();
    let clock = Instant::now();
    let table = run_the_arm_over_the_pool(specification.arm, &pool, &system_configuration);
    outcome.scan_seconds = clock.elapsed().as_secs_f64();
    outcome.allocator_peak_over_start_bytes = peak_bytes() - start;
    outcome.allocator_live_after_return_bytes = in_use_bytes() - start;
    outcome.vm_hwm_kib = vm_hwm_kib();
    outcome.reads = pool.reads.get();
    outcome.records_returned = u64::try_from(table.len()).expect("表长装得进 u64");
    outcome.allocator_live_at_each_device_end_bytes = pool.live_at_each_device_end.borrow().clone();
    outcome.table_digest = table_digest(&table);
    outcome
}

// ============================================================================================
// 差分集 D（登记第九节）：小环（64 个槽），盘 4 GiB，不给提示；三份胶水各与 `scan_journal` 逐键逐值比。
// ============================================================================================

const DIFFERENTIAL_RING_SLOTS: u64 = 64;
const DIFFERENTIAL_NAMED_ENTRIES: usize = 3;

fn differential_record(counter: u64, transaction: u64) -> JournalRecord {
    journal_record_for(
        PRODUCTION_INSTANCE,
        counter,
        transaction,
        DIFFERENTIAL_NAMED_ENTRIES,
    )
}

/// 四例的盘数与「某块盘某个槽放哪条记录」。
fn differential_case(case: &str) -> Option<(u32, SlotSource)> {
    match case {
        // D1：2 盘，第一盘 0–31 槽有记录，第二盘 16–63 槽有记录，重叠处字节相同。
        "D1" => Some((
            2,
            slot_source_of(|device, slot| match device {
                0 if slot < 32 => Some(differential_record(slot + 1, slot + 1)),
                1 if slot >= 16 => Some(differential_record(slot + 1, slot + 1)),
                _ => None,
            }),
        )),
        // D2：2 盘，第 4 槽两盘都是（实例 1，计数器 5），事务号不同；`scan_journal` 留第一盘那一份。
        "D2" => Some((
            2,
            slot_source_of(|device, slot| {
                (slot == 4).then(|| differential_record(5, 100 + 100 * u64::from(device)))
            }),
        )),
        // D3：3 盘，第一盘全 0、第二盘半满、第三盘满。
        "D3" => Some((
            3,
            slot_source_of(|device, slot| match device {
                1 if slot < 32 => Some(differential_record(slot + 1, slot + 1)),
                2 => Some(differential_record(slot + 1, slot + 1)),
                _ => None,
            }),
        )),
        // D4：1 盘，计数器 3 的记录在第 2 槽与第 40 槽各有一份、内容不同，按偏移次序留前一份。
        "D4" => Some((
            1,
            slot_source_of(|_device, slot| match slot {
                2 => Some(differential_record(3, 300)),
                40 => Some(differential_record(3, 400)),
                _ => None,
            }),
        )),
        _ => None,
    }
}

const DIFFERENTIAL_CASES: [&str; 4] = ["D1", "D2", "D3", "D4"];
const GLUE_ARMS_COMPARED_WITH_SCAN_JOURNAL: [Arm; 3] = [Arm::BaselinePc, Arm::Split, Arm::SplitPc];

/// 一例一臂：交回的表与 `scan_journal` 在同一个池上交回的是否逐键逐值相同。
fn differential_result(case: &str, arm: Arm) -> (bool, u64) {
    let (devices, source) = differential_case(case).expect("差分例名是 D1–D4");
    let system_configuration =
        system_configuration_for(DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES, devices);
    let pool = GeneratedPool::new(
        devices,
        TEST_IMAGE_DEFAULT_BYTES,
        DIFFERENTIAL_RING_SLOTS,
        source,
    );
    let reference = scan_journal(&pool, &system_configuration);
    let candidate = run_the_arm_over_the_pool(arm, &pool, &system_configuration);
    (
        tables_are_equal(&reference, &candidate),
        u64::try_from(reference.len()).expect("表长装得进 u64"),
    )
}

/// 差分集 D 的结果行；返回是否全等。
fn run_the_differential_set() -> (Vec<String>, bool) {
    let mut lines = Vec::new();
    let mut all_equal = true;
    for case in DIFFERENTIAL_CASES {
        for arm in GLUE_ARMS_COMPARED_WITH_SCAN_JOURNAL {
            let (equal, reference_records) = differential_result(case, arm);
            all_equal &= equal;
            lines.push(format!(
                "E7RESULT name=e197_diff case={case} arm={} equal={equal} reference_records={reference_records}",
                arm.name()
            ));
        }
    }
    (lines, all_equal)
}

/// 生产格上的一臂与 `scan_journal` 在同一个池上逐键逐值比（登记第十一节 S2「任一格」）：先跑臂、留着它交回的表，再跑 `scan_journal`。
/// 不量峰值，所以放在量窗之外的专用进程里（`--compare`）；交 (是否全等, 基准表的键数)。
fn compare_arm_with_scan_journal(specification: CellSpecification, ring_slots: u64) -> (bool, u64) {
    let filled_slots = specification.fill.filled_slots_per_device(ring_slots);
    let system_configuration =
        system_configuration_for(ring_slots * JOURNAL_RECORD_BYTES, specification.devices);
    let pool = GeneratedPool::new(
        specification.devices,
        TEST_IMAGE_DEFAULT_BYTES,
        ring_slots,
        production_slot_source(filled_slots, specification.named),
    );
    let candidate = run_the_arm_over_the_pool(specification.arm, &pool, &system_configuration);
    let reference = scan_journal(&pool, &system_configuration);
    (
        tables_are_equal(&reference, &candidate),
        u64::try_from(reference.len()).expect("表长装得进 u64"),
    )
}

/// `--checks`：本地常量回比、A1、A4、A5、A6 的读数（打出来，单测另钉）。
fn run_the_checks() -> Vec<String> {
    constants_against_the_implementation();
    let sizes = structure_sizes();
    let default_ring_bytes = JOURNAL_RING_DEFAULT_BYTES;
    vec![
        format!(
            "E7RESULT name=e197_checks constants_match_the_implementation=true ring_slots={RING_SLOTS} ring_slots_remainder={} device_holds_the_default_ring={} capacity_after_196608_pushes={} capacity_after_786432_pushes={}",
            default_ring_bytes % JOURNAL_RECORD_BYTES,
            device_holds_the_ring(TEST_IMAGE_DEFAULT_BYTES, default_ring_bytes),
            vec_capacity_after_pushes(RING_SLOTS),
            vec_capacity_after_pushes(4 * RING_SLOTS),
        ),
        format!(
            "E7RESULT name=e197_sizes size_location_entry={} size_named_unit={} size_node_pointer={} size_journal_record={}",
            sizes.location_entry, sizes.named_unit, sizes.node_pointer, sizes.journal_record
        ),
    ]
}

// ============================================================================================
// 入口：参数、准入、打印。
// ============================================================================================

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；强制跑（`--force`）时交回摘要，`main` 把它作为头一行打到标准输出。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source =
        format!("{manifest_directory}/src/bin/e197_scan_journal_glue_memory_by_device_count.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let arguments: Vec<String> = std::env::args().skip(1).collect();
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
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
    let first_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    first_line
        .strip_prefix("forced")
        .map(|summary| summary.trim().replace(' ', "_"))
}

/// 命令行解出来的一件事。
#[derive(Debug, PartialEq, Eq)]
enum Request {
    Cell(CellSpecification),
    /// 生产格上的臂与 `scan_journal` 逐键逐值比（`--compare`）：不量峰值，登记第十一节 S2 在每一格上要的比法。
    Compare(CellSpecification),
    DifferentialSet,
    Checks,
}

/// `--arm <臂> --devices <盘数> --fill <full|half> --named <点名项数> --repeat <重复号>`（五个都要，另可带 `--compare`）、`--diff-set`、`--checks`。
fn parse_the_arguments(words: &[&str]) -> Result<Request, String> {
    match words {
        ["--diff-set"] => return Ok(Request::DifferentialSet),
        ["--checks"] => return Ok(Request::Checks),
        _ => {}
    }
    let mut arm = None;
    let mut devices = None;
    let mut fill = None;
    let mut named = None;
    let mut repeat = None;
    let mut compare = false;
    let mut index = 0;
    while index < words.len() {
        if words[index] == "--compare" {
            compare = true;
            index += 1;
            continue;
        }
        let value = *words
            .get(index + 1)
            .ok_or_else(|| format!("{} 后面缺值", words[index]))?;
        match words[index] {
            "--arm" => {
                arm = Some(Arm::from_name(value).ok_or_else(|| format!("不认识的臂 {value}"))?)
            }
            "--devices" => {
                devices = Some(
                    value
                        .parse::<u32>()
                        .map_err(|error| format!("盘数 {value}：{error}"))?,
                )
            }
            "--fill" => {
                fill =
                    Some(Fill::from_name(value).ok_or_else(|| format!("不认识的填满档 {value}"))?)
            }
            "--named" => {
                named = Some(
                    value
                        .parse::<usize>()
                        .map_err(|error| format!("点名项数 {value}：{error}"))?,
                )
            }
            "--repeat" => {
                repeat = Some(
                    value
                        .parse::<u32>()
                        .map_err(|error| format!("重复号 {value}：{error}"))?,
                )
            }
            other => return Err(format!("不认识的参数 {other}")),
        }
        index += 2;
    }
    let specification = CellSpecification {
        arm: arm.ok_or("缺 --arm")?,
        devices: devices.ok_or("缺 --devices")?,
        fill: fill.ok_or("缺 --fill")?,
        named: named.ok_or("缺 --named")?,
        repeat: repeat.ok_or("缺 --repeat")?,
    };
    if specification.devices == 0
        || specification.repeat == 0
        || specification.repeat > LAST_REPEAT_NUMBER
    {
        return Err(format!("盘数要 ≥ 1，重复号取 1–{LAST_REPEAT_NUMBER}"));
    }
    if u64::try_from(specification.named)
        .map_or(true, |count| count > JOURNAL_NAMED_ENTRIES_PER_RECORD)
    {
        return Err(format!("点名项数不超过 {JOURNAL_NAMED_ENTRIES_PER_RECORD}"));
    }
    Ok(if compare {
        Request::Compare(specification)
    } else {
        Request::Cell(specification)
    })
}

fn usage() {
    eprintln!("用法：e197_scan_journal_glue_memory_by_device_count --arm <empty|measurement_pc|baseline|baseline_pc|split|split_pc> --devices <N> --fill <full|half> --named <K> --repeat <1..10>");
    eprintln!("      （同五个参数再加 --compare：该臂的表与 scan_journal 逐键逐值比，不量峰值）");
    eprintln!("      e197_scan_journal_glue_memory_by_device_count --diff-set | --checks");
}

fn main() {
    let forced_summary = preflight();
    if let Some(summary) = forced_summary {
        println!("E7RESULT name=preflight forced={summary}");
    }
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    constants_against_the_implementation();
    let request = match parse_the_arguments(&words) {
        Ok(request) => request,
        Err(message) => {
            eprintln!("{message}");
            usage();
            std::process::exit(2);
        }
    };
    let exit_code = match request {
        Request::Cell(specification) => {
            let outcome = measure_one_cell_in_one_process(specification, RING_SLOTS);
            println!("{}", outcome.result_line(structure_sizes()));
            0
        }
        Request::Compare(specification) => {
            let (equal, records) = compare_arm_with_scan_journal(specification, RING_SLOTS);
            println!(
                "E7RESULT name=e197_compare arm={} devices={} fill={} named={} equal={equal} records={records}",
                specification.arm.name(),
                specification.devices,
                specification.fill.name(),
                specification.named
            );
            0
        }
        Request::DifferentialSet => {
            let (lines, all_equal) = run_the_differential_set();
            for line in &lines {
                println!("{line}");
            }
            i32::from(!all_equal)
        }
        Request::Checks => {
            for line in run_the_checks() {
                println!("{line}");
            }
            0
        }
    };
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_RING_SLOTS: u64 = 4096;
    const SCANNING_ARMS: [Arm; 4] = [Arm::Baseline, Arm::BaselinePc, Arm::Split, Arm::SplitPc];

    fn small_cell(
        arm: Arm,
        devices: u32,
        fill: Fill,
        named: usize,
        ring_slots: u64,
    ) -> CellOutcome {
        measure_one_cell_in_one_process(
            CellSpecification {
                arm,
                devices,
                fill,
                named,
                repeat: 1,
            },
            ring_slots,
        )
    }

    /// 登记第七节 A6 手算的四个大小（点名项 64、位置条目 16、节点指针 56、记录 376），推算脚本 `predict.py` 用的那一组。
    const REGISTERED_HAND_COUNTED_SIZES: StructureSizes = StructureSizes {
        location_entry: 16,
        named_unit: 64,
        node_pointer: 56,
        journal_record: 376,
    };

    #[test]
    fn local_constants_equal_the_implementation_constants_and_the_default_ring_has_196608_slots() {
        constants_against_the_implementation();
        assert_eq!(RING_SLOTS, 196_608);
        assert_eq!(JOURNAL_RING_DEFAULT_BYTES % JOURNAL_RECORD_BYTES, 0);
        assert_eq!(JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES, 196_608);
        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 64);
    }

    #[test]
    fn device_holds_the_ring_only_when_the_ring_is_at_most_one_quarter_of_it() {
        // 登记第七节 A4：4 GiB 装得下 768 MiB 的环（4 × 环长 = 3 221 225 472 ≤ 4 GiB）。
        assert_eq!(4 * JOURNAL_RING_DEFAULT_BYTES, 3_221_225_472);
        assert!(device_holds_the_ring(
            TEST_IMAGE_DEFAULT_BYTES,
            JOURNAL_RING_DEFAULT_BYTES
        ));
        assert!(device_holds_the_ring(
            3_221_225_472,
            JOURNAL_RING_DEFAULT_BYTES
        ));
        assert!(!device_holds_the_ring(
            3_221_225_471,
            JOURNAL_RING_DEFAULT_BYTES
        ));
        // 2 GiB 的盘装不下 768 MiB 的环（÷ 4 = 512 MiB），1 GiB 的更装不下。
        assert!(!device_holds_the_ring(2 << 30, JOURNAL_RING_DEFAULT_BYTES));
        assert!(!device_holds_the_ring(1 << 30, JOURNAL_RING_DEFAULT_BYTES));
        assert_eq!(
            JOURNAL_RING_START_SLOT * SLOT_BYTES + JOURNAL_RING_DEFAULT_BYTES,
            822_083_584
        );
    }

    #[test]
    fn vec_capacity_after_pushes_matches_the_registered_values_and_a_real_vec() {
        for (count, capacity) in [
            (98_304u64, 131_072u64),
            (196_608, 262_144),
            (294_912, 524_288),
            (393_216, 524_288),
            (589_824, 1_048_576),
            (786_432, 1_048_576),
        ] {
            assert_eq!(vec_capacity_after_pushes(count), capacity);
        }
        for count in [1usize, 4, 5, 8, 9, 1000, 4096, 5000] {
            let mut vector: Vec<[u8; 320]> = Vec::new();
            for _ in 0..count {
                vector.push([0u8; 320]);
            }
            assert_eq!(
                u64::try_from(vector.capacity()).expect("容量装得进 u64"),
                vec_capacity_after_pushes(u64::try_from(count).expect("条数装得进 u64")),
                "逐条 push 到 {count} 条时 Vec 的容量"
            );
        }
    }

    #[test]
    fn structure_sizes_match_the_field_by_field_layout_of_the_current_structs() {
        let sizes = structure_sizes();
        println!(
            "size_of：LocationEntry={} NamedUnit={} NodePointer={} JournalRecord={}（登记第七节 A6 手算：16、64、56、{}）",
            sizes.location_entry,
            sizes.named_unit,
            sizes.node_pointer,
            sizes.journal_record,
            REGISTERED_HAND_COUNTED_SIZES.journal_record
        );
        // 位置条目 4 + 8 + 4；点名项 2 × 16 + 1 + 8 + 8 + 10 = 59 按 8 对齐；节点指针 16 + 2 × 16 + 4 + 4；
        // 记录：结构体里是四条节点指针（树表、映射根、实例表、分配记录树根）4 × 56 + 七个 8 字节 + 三个 4 字节 + 四个一字节量 + 一个 Vec 24 = 320；登记手算写 376（按五条节点指针数的），
        // 逐字段重数是 320，以这条断言跑出的 size_of 为准，对不上照实报、不改登记。
        assert_eq!(
            sizes.location_entry,
            REGISTERED_HAND_COUNTED_SIZES.location_entry
        );
        assert_eq!(sizes.named_unit, REGISTERED_HAND_COUNTED_SIZES.named_unit);
        assert_eq!(
            sizes.node_pointer,
            REGISTERED_HAND_COUNTED_SIZES.node_pointer
        );
        assert_eq!(sizes.journal_record, 4 * 56 + 7 * 8 + 3 * 4 + 4 + 24);
    }

    #[test]
    fn counting_allocator_counts_a_realloc_by_the_size_difference_and_returns_to_the_start_after_the_free(
    ) {
        let start = in_use_bytes();
        let mut vector: Vec<u64> = Vec::new();
        for value in 0..10_000u64 {
            vector.push(value);
        }
        let capacity_bytes = i64::try_from(vector.capacity() * 8).expect("容量字节装得进 i64");
        assert_eq!(in_use_bytes() - start, capacity_bytes);
        drop(vector);
        assert_eq!(in_use_bytes(), start);
    }

    #[test]
    fn counting_allocator_peak_stays_at_the_maximum_after_the_memory_is_freed_and_resets_to_the_in_use(
    ) {
        let first = vec![0u8; 2 << 20];
        std::hint::black_box(&first);
        drop(first);
        let start = in_use_bytes();
        reset_peak_to_in_use();
        assert_eq!(peak_bytes(), start);
        let second = vec![0u8; 1 << 20];
        std::hint::black_box(&second);
        drop(second);
        let peak_over_start = peak_bytes() - start;
        assert!(((1 << 20)..(1 << 20) + 4096).contains(&peak_over_start));
        assert_eq!(in_use_bytes(), start);
    }

    #[test]
    fn measurement_positive_control_lands_in_the_registered_window() {
        let peak_over_start = measure_the_positive_control_allocation();
        assert!(
            (268_435_456..=268_500_992).contains(&peak_over_start),
            "测量路径对照的分配器峰值减起点 {peak_over_start}"
        );
        assert!(measurement_peak_is_in_the_registered_window(
            peak_over_start
        ));
        assert!(measurement_peak_is_in_the_registered_window(268_435_456));
        assert!(measurement_peak_is_in_the_registered_window(268_500_992));
        assert!(!measurement_peak_is_in_the_registered_window(268_435_455));
        assert!(!measurement_peak_is_in_the_registered_window(268_500_993));
    }

    #[test]
    fn the_reader_answers_only_whole_record_slots_inside_the_ring() {
        let pool = GeneratedPool::new(
            2,
            TEST_IMAGE_DEFAULT_BYTES,
            16,
            production_slot_source(8, 2),
        );
        let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
        let read = |device: u32, offset: u64, length: usize| {
            pool.read(DeviceIdentity(device), DeviceOffsetInBytes(offset), length)
        };
        assert!(read(0, ring_start - JOURNAL_RECORD_BYTES, record_bytes).is_none());
        assert!(read(0, ring_start + 16 * JOURNAL_RECORD_BYTES, record_bytes).is_none());
        assert!(read(0, ring_start + 1, record_bytes).is_none());
        assert!(read(0, ring_start, record_bytes - 1).is_none());
        assert!(read(2, ring_start, record_bytes).is_none());
        let written =
            read(1, ring_start + 7 * JOURNAL_RECORD_BYTES, record_bytes).expect("槽 7 有记录");
        let parsed =
            JournalRecord::parse(&written, unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER))
                .expect("槽 7 的字节过得了 parse");
        assert_eq!((parsed.counter, parsed.named.len()), (8, 2));
        let never_written =
            read(1, ring_start + 8 * JOURNAL_RECORD_BYTES, record_bytes).expect("槽 8 读得出");
        assert_eq!(never_written, vec![0u8; record_bytes]);
        assert_eq!(pool.reads.get(), 7);
        assert!(pool
            .journal_record_offsets_hint(
                DeviceIdentity(0),
                DeviceOffsetInBytes(ring_start),
                16 * JOURNAL_RECORD_BYTES
            )
            .is_none());
    }

    #[test]
    fn the_production_slot_source_follows_the_fill_and_the_named_count() {
        let source = production_slot_source(100, 7);
        assert_eq!(source(0, 99).expect("槽 99 有记录").named.len(), 7);
        assert_eq!(source(3, 0).expect("每块盘同一槽字节相同").counter, 1);
        assert!(source(0, 100).is_none());
        let single = production_slot_source(100, 1);
        assert_eq!(single(0, 0).expect("槽 0 有记录").named.len(), 1);
        let full_named = production_slot_source(100, 64);
        assert_eq!(full_named(1, 5).expect("槽 5 有记录").named.len(), 64);
        assert_eq!(source(0, 5), source(1, 5));
        assert_eq!(Fill::Full.filled_slots_per_device(RING_SLOTS), 196_608);
        assert_eq!(Fill::Half.filled_slots_per_device(RING_SLOTS), 98_304);
    }

    #[test]
    fn every_scanning_arm_reads_once_per_ring_slot_per_device_and_returns_one_key_per_filled_slot()
    {
        let ring_slots = 256;
        for arm in SCANNING_ARMS {
            for devices in 1..=3u32 {
                for (fill, filled) in [(Fill::Full, 256u64), (Fill::Half, 128)] {
                    let outcome = small_cell(arm, devices, fill, 2, ring_slots);
                    let context = format!("{} {devices} 盘 {}", arm.name(), fill.name());
                    assert_eq!(
                        outcome.reads,
                        u64::from(devices) * ring_slots,
                        "{context} 读数"
                    );
                    assert_eq!(
                        outcome.reads_expected,
                        u64::from(devices) * ring_slots,
                        "{context}"
                    );
                    assert_eq!(outcome.records_returned, filled, "{context} 键数");
                    assert!(outcome.anchors_hold(), "{context}");
                    assert_eq!(
                        outcome.allocator_live_at_each_device_end_bytes.len(),
                        usize::try_from(devices).expect("盘数装得进 usize"),
                        "{context} 轨迹点数"
                    );
                }
            }
        }
        for devices in 1..=3u32 {
            let empty = small_cell(Arm::Empty, devices, Fill::Full, 2, ring_slots);
            assert_eq!((empty.reads, empty.reads_expected), (0, 0));
            assert_eq!((empty.records_returned, empty.records_expected), (0, 0));
            assert!(empty.anchors_hold());
            assert_eq!(empty.table_digest, "none");
        }
    }

    #[test]
    fn the_default_ring_geometry_numbers_are_pinned_for_one_to_four_devices() {
        let expected_reads: Vec<u64> = (1..=4u64).map(|devices| devices * RING_SLOTS).collect();
        assert_eq!(expected_reads, [196_608, 393_216, 589_824, 786_432]);
        assert_eq!(Arm::from_name("split_pc"), Some(Arm::SplitPc));
        assert_eq!(Arm::from_name("split-pc"), None);
        assert!(Arm::Baseline.scans() && !Arm::Empty.scans() && !Arm::MeasurementPc.scans());
        assert_eq!(Fill::from_name("half"), Some(Fill::Half));
        assert_eq!(Fill::from_name("full"), Some(Fill::Full));
    }

    #[test]
    fn every_glue_arm_returns_the_table_scan_journal_returns_on_the_registered_differential_cases()
    {
        let expected_reference_records = [("D1", 64u64), ("D2", 1), ("D3", 64), ("D4", 1)];
        for (case, reference_records) in expected_reference_records {
            for arm in GLUE_ARMS_COMPARED_WITH_SCAN_JOURNAL {
                let (equal, records) = differential_result(case, arm);
                assert!(
                    equal,
                    "{case} 上 {} 交回的表与 scan_journal 不同",
                    arm.name()
                );
                assert_eq!(records, reference_records, "{case} 的键数");
            }
        }
        let (lines, all_equal) = run_the_differential_set();
        assert!(all_equal);
        assert_eq!(lines.len(), 12);
        assert!(lines
            .iter()
            .all(|line| line.starts_with("E7RESULT name=e197_diff case=D")));
    }

    #[test]
    fn every_glue_arm_returns_the_table_scan_journal_returns_on_the_generated_pool_and_the_comparison_can_say_no(
    ) {
        for arm in GLUE_ARMS_COMPARED_WITH_SCAN_JOURNAL {
            for devices in 1..=3u32 {
                for fill in [Fill::Full, Fill::Half] {
                    for named in [1usize, 3] {
                        let specification = CellSpecification {
                            arm,
                            devices,
                            fill,
                            named,
                            repeat: 1,
                        };
                        let (equal, records) = compare_arm_with_scan_journal(specification, 256);
                        let context =
                            format!("{} {devices} 盘 {} K={named}", arm.name(), fill.name());
                        assert!(equal, "{context}");
                        assert_eq!(records, fill.filled_slots_per_device(256), "{context}");
                    }
                }
            }
        }
        // 比法自己要能判不等：空跑臂交回空表，对 256 个键的基准表不等。
        let empty_arm = CellSpecification {
            arm: Arm::Empty,
            devices: 2,
            fill: Fill::Full,
            named: 1,
            repeat: 1,
        };
        assert_eq!(compare_arm_with_scan_journal(empty_arm, 256), (false, 256));
    }

    #[test]
    fn scan_journal_keeps_the_first_device_copy_of_each_key_so_the_conflicting_copy_case_tells_a_per_device_merge_apart(
    ) {
        let (devices, source) = differential_case("D2").expect("D2");
        let configuration =
            system_configuration_for(DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES, devices);
        let pool = GeneratedPool::new(
            devices,
            TEST_IMAGE_DEFAULT_BYTES,
            DIFFERENTIAL_RING_SLOTS,
            source,
        );
        let table = scan_journal(&pool, &configuration);
        assert_eq!(table.len(), 1);
        let kept = table
            .get(&(InstanceGeneration(PRODUCTION_INSTANCE), 5))
            .expect("计数器 5 的记录在");
        assert_eq!(kept.transaction, 100);
        let (later_devices, later_source) = differential_case("D4").expect("D4");
        let later_configuration = system_configuration_for(
            DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES,
            later_devices,
        );
        let later_pool = GeneratedPool::new(
            later_devices,
            TEST_IMAGE_DEFAULT_BYTES,
            DIFFERENTIAL_RING_SLOTS,
            later_source,
        );
        let later_table = scan_journal(&later_pool, &later_configuration);
        assert_eq!(
            later_table
                .get(&(InstanceGeneration(PRODUCTION_INSTANCE), 3))
                .expect("计数器 3 的记录在")
                .transaction,
            300
        );
        assert!(differential_case("D5").is_none());
    }

    #[test]
    fn table_comparison_and_digest_see_a_changed_value_a_missing_key_and_nothing_else() {
        let (devices, source) = differential_case("D1").expect("D1");
        let configuration =
            system_configuration_for(DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES, devices);
        let pool = GeneratedPool::new(
            devices,
            TEST_IMAGE_DEFAULT_BYTES,
            DIFFERENTIAL_RING_SLOTS,
            source,
        );
        let table = scan_journal(&pool, &configuration);
        let same = table.clone();
        assert!(tables_are_equal(&table, &same));
        assert_eq!(table_digest(&table), table_digest(&same));
        let key = (InstanceGeneration(PRODUCTION_INSTANCE), 7);
        let mut changed_value = table.clone();
        changed_value
            .get_mut(&key)
            .expect("计数器 7 在")
            .transaction += 1;
        assert!(!tables_are_equal(&table, &changed_value));
        assert_ne!(table_digest(&table), table_digest(&changed_value));
        let mut changed_named = table.clone();
        changed_named
            .get_mut(&key)
            .expect("计数器 7 在")
            .named
            .pop();
        assert!(!tables_are_equal(&table, &changed_named));
        let mut missing_key = table.clone();
        missing_key.remove(&key);
        assert!(!tables_are_equal(&table, &missing_key));
        assert_ne!(table_digest(&table), table_digest(&missing_key));
        let mut moved_key = table.clone();
        let record = moved_key.remove(&key).expect("计数器 7 在");
        moved_key.insert((InstanceGeneration(PRODUCTION_INSTANCE), 100), record);
        assert_ne!(table_digest(&table), table_digest(&moved_key));
    }

    #[test]
    fn the_digest_follows_the_encoding_checksum_of_each_value_and_not_only_the_keys() {
        let (devices, source) = differential_case("D3").expect("D3");
        let configuration =
            system_configuration_for(DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES, devices);
        let pool = GeneratedPool::new(
            devices,
            TEST_IMAGE_DEFAULT_BYTES,
            DIFFERENTIAL_RING_SLOTS,
            source,
        );
        let table = scan_journal(&pool, &configuration);
        let mut changed_value_only = table.clone();
        changed_value_only
            .get_mut(&(InstanceGeneration(PRODUCTION_INSTANCE), 3))
            .expect("计数器 3 在")
            .back_chain ^= 1;
        assert_eq!(
            table.keys().collect::<Vec<_>>(),
            changed_value_only.keys().collect::<Vec<_>>()
        );
        assert_ne!(table_digest(&table), table_digest(&changed_value_only));
    }

    #[test]
    fn keeping_the_scan_until_return_peaks_above_the_baseline_by_the_named_unit_heap_of_the_first_copies(
    ) {
        let named = 64;
        let baseline = small_cell(Arm::Baseline, 2, Fill::Full, named, SMALL_RING_SLOTS);
        let control = small_cell(Arm::BaselinePc, 2, Fill::Full, named, SMALL_RING_SLOTS);
        let extra =
            control.allocator_peak_over_start_bytes - baseline.allocator_peak_over_start_bytes;
        let expected_extra = i64::try_from(
            SMALL_RING_SLOTS * u64::try_from(named).expect("64") * structure_sizes().named_unit,
        )
        .expect("装得进 i64");
        assert!(
            control.allocator_peak_over_start_bytes * 10
                > baseline.allocator_peak_over_start_bytes * 11,
            "阳性对照要比基线高出 10% 以上：{} 对 {}",
            control.allocator_peak_over_start_bytes,
            baseline.allocator_peak_over_start_bytes
        );
        assert!(
            (extra - expected_extra).abs() * 20 < expected_extra,
            "多占的应是首份点名项堆 {expected_extra}（±5%），量到 {extra}"
        );
        assert_eq!(baseline.table_digest, control.table_digest);
    }

    #[test]
    fn merging_device_by_device_stops_the_peak_growing_with_the_device_count_and_keeping_the_duplicates_does_not(
    ) {
        let peak = |arm: Arm, devices: u32| {
            small_cell(arm, devices, Fill::Full, 64, SMALL_RING_SLOTS)
                .allocator_peak_over_start_bytes as f64
        };
        let slope = |arm: Arm| (peak(arm, 4) - peak(arm, 2)) / (2.0 * peak(arm, 1));
        let split_slope = slope(Arm::Split);
        let control_slope = slope(Arm::SplitPc);
        let baseline_slope = slope(Arm::Baseline);
        assert!(split_slope <= 0.10, "split 的 slope {split_slope}");
        assert!(control_slope > 0.50, "split_pc 的 slope {control_slope}");
        assert!(baseline_slope > 0.50, "baseline 的 slope {baseline_slope}");
    }

    #[test]
    fn the_prediction_reproduces_the_registered_design_numbers_for_the_default_ring() {
        // 登记第四节 `predict.py` 的输出（按手算的大小 376 / 64）：满槽 K = 64、半满与 K = 1 各抽几格，单位 GiB，容差 0.002。
        let gibibytes = |arm: Arm, devices: u64, filled: u64, named: u64| {
            predicted_allocator_peak_bytes(
                arm,
                devices,
                filled,
                named,
                REGISTERED_HAND_COUNTED_SIZES,
            ) / f64::from(1u32 << 30)
        };
        let cases: [(Arm, u64, u64, u64, f64); 11] = [
            (Arm::Baseline, 1, RING_SLOTS, 64, 0.985),
            (Arm::Baseline, 2, RING_SLOTS, 64, 1.830),
            (Arm::Baseline, 3, RING_SLOTS, 64, 2.767),
            (Arm::Baseline, 4, RING_SLOTS, 64, 3.520),
            (Arm::BaselinePc, 4, RING_SLOTS, 64, 4.270),
            (Arm::Split, 2, RING_SLOTS, 64, 1.738),
            (Arm::Split, 4, RING_SLOTS, 64, 1.744),
            (Arm::Baseline, 4, RING_SLOTS / 2, 64, 1.760),
            (Arm::Split, 4, RING_SLOTS / 2, 64, 0.872),
            (Arm::Baseline, 4, RING_SLOTS, 1, 0.566),
            (Arm::Split, 4, RING_SLOTS, 1, 0.268),
        ];
        for (arm, devices, filled, named, expected) in cases {
            let predicted = gibibytes(arm, devices, filled, named);
            assert!(
                (predicted - expected).abs() < 0.002,
                "{} {devices} 盘 {filled} 槽 K={named}：推 {predicted:.4} GiB，登记 {expected}",
                arm.name()
            );
        }
        assert_eq!(
            predicted_allocator_peak_bytes(
                Arm::Empty,
                4,
                RING_SLOTS,
                64,
                REGISTERED_HAND_COUNTED_SIZES
            ),
            0.0
        );
    }

    #[test]
    fn the_split_pc_prediction_replaces_the_one_big_vec_by_the_duplicates_vec_and_the_last_device_vec(
    ) {
        let sizes = REGISTERED_HAND_COUNTED_SIZES;
        let record = sizes.journal_record as f64;
        for (devices, named) in [(1u64, 64u64), (2, 64), (3, 64), (4, 64), (3, 1), (4, 1)] {
            let base =
                predicted_allocator_peak_bytes(Arm::Baseline, devices, RING_SLOTS, named, sizes);
            let control =
                predicted_allocator_peak_bytes(Arm::SplitPc, devices, RING_SLOTS, named, sizes);
            let one_vec = vec_capacity_after_pushes(devices * RING_SLOTS) as f64;
            let duplicates = (devices - 1) * RING_SLOTS;
            let duplicates_capacity = if duplicates == 0 {
                0
            } else {
                vec_capacity_after_pushes(duplicates)
            };
            let two_vecs =
                duplicates_capacity as f64 + vec_capacity_after_pushes(RING_SLOTS) as f64;
            let expected_difference = (two_vecs - one_vec) * record;
            assert!(
                (control - base - expected_difference).abs() < 1.0,
                "{devices} 盘 K={named}：两式之差 {} 对 {expected_difference}",
                control - base
            );
        }
        // 登记的「同基线」在 1、2 盘上就是这个式子（读到的记录数 196 608、393 216 的容量正好是两个 262 144）。
        for devices in [1u64, 2] {
            let control =
                predicted_allocator_peak_bytes(Arm::SplitPc, devices, RING_SLOTS, 64, sizes);
            let base =
                predicted_allocator_peak_bytes(Arm::Baseline, devices, RING_SLOTS, 64, sizes);
            assert!(
                (control - base).abs() < 1.0,
                "{devices} 盘：{control} 对 {base}"
            );
        }
    }

    #[test]
    fn the_allocator_peak_of_every_scanning_arm_stays_within_the_prediction() {
        let ring_slots = 8192;
        let sizes = structure_sizes();
        for (named, tolerance) in [
            (64usize, PREDICTION_TOLERANCE),
            (1, 2.0 * PREDICTION_TOLERANCE),
        ] {
            for arm in SCANNING_ARMS {
                for devices in [1u32, 2, 3] {
                    let outcome = small_cell(arm, devices, Fill::Full, named, ring_slots);
                    let predicted = predicted_allocator_peak_bytes(
                        arm,
                        u64::from(devices),
                        ring_slots,
                        u64::try_from(named).expect("点名项数装得进 u64"),
                        sizes,
                    );
                    let measured = outcome.allocator_peak_over_start_bytes as f64;
                    assert!(
                        (measured / predicted - 1.0).abs() <= tolerance,
                        "{} {devices} 盘 K={named}：量到 {measured}，推 {predicted:.0}",
                        arm.name()
                    );
                }
            }
        }
    }

    #[test]
    fn the_result_line_carries_every_field_the_report_reads() {
        let outcome = small_cell(Arm::Split, 2, Fill::Half, 3, 256);
        let line = outcome.result_line(structure_sizes());
        let body = line.strip_prefix("E7RESULT ").expect("以 E7RESULT 起头");
        let fields: BTreeMap<&str, &str> = body
            .split_whitespace()
            .filter_map(|word| word.split_once('='))
            .collect();
        for key in [
            "name",
            "arm",
            "devices",
            "fill",
            "named",
            "repeat",
            "ring_slots",
            "filled_slots",
            "reads",
            "reads_expected",
            "records_returned",
            "records_expected",
            "anchors_hold",
            "allocator_peak_over_start_bytes",
            "allocator_live_at_each_device_end_bytes",
            "allocator_live_after_return_bytes",
            "vm_hwm_kib",
            "scan_seconds",
            "table_digest",
            "size_location_entry",
            "size_named_unit",
            "size_node_pointer",
            "size_journal_record",
        ] {
            assert!(fields.contains_key(key), "结果行缺字段 {key}：{line}");
        }
        assert_eq!(fields["name"], "e197");
        assert_eq!(fields["arm"], "split");
        assert_eq!(fields["fill"], "half");
        assert_eq!(fields["records_returned"], "128");
        assert_eq!(fields["reads"], "512");
        assert_eq!(fields["anchors_hold"], "true");
        assert_eq!(
            fields["allocator_live_at_each_device_end_bytes"]
                .split(',')
                .count(),
            2
        );
        assert_eq!(fields["table_digest"].len(), 16);
    }

    #[test]
    fn the_arguments_parse_into_a_cell_and_the_wrong_ones_are_refused() {
        let good = [
            "--arm",
            "split",
            "--devices",
            "4",
            "--fill",
            "half",
            "--named",
            "64",
            "--repeat",
            "5",
        ];
        assert_eq!(
            parse_the_arguments(&good),
            Ok(Request::Cell(CellSpecification {
                arm: Arm::Split,
                devices: 4,
                fill: Fill::Half,
                named: 64,
                repeat: 5,
            }))
        );
        let compare = [
            "--arm",
            "split",
            "--devices",
            "4",
            "--fill",
            "half",
            "--named",
            "64",
            "--repeat",
            "5",
            "--compare",
        ];
        assert_eq!(
            parse_the_arguments(&compare),
            Ok(Request::Compare(CellSpecification {
                arm: Arm::Split,
                devices: 4,
                fill: Fill::Half,
                named: 64,
                repeat: 5,
            }))
        );
        assert_eq!(
            parse_the_arguments(&["--diff-set"]),
            Ok(Request::DifferentialSet)
        );
        assert_eq!(parse_the_arguments(&["--checks"]), Ok(Request::Checks));
        let refused: [&[&str]; 8] = [
            &[],
            &["--arm", "split"],
            &[
                "--arm",
                "nope",
                "--devices",
                "1",
                "--fill",
                "full",
                "--named",
                "1",
                "--repeat",
                "1",
            ],
            &[
                "--arm",
                "split",
                "--devices",
                "0",
                "--fill",
                "full",
                "--named",
                "1",
                "--repeat",
                "1",
            ],
            &[
                "--arm",
                "split",
                "--devices",
                "1",
                "--fill",
                "full",
                "--named",
                "65",
                "--repeat",
                "1",
            ],
            &[
                "--arm",
                "split",
                "--devices",
                "1",
                "--fill",
                "full",
                "--named",
                "1",
                "--repeat",
                "11",
            ],
            &[
                "--arm",
                "split",
                "--devices",
                "1",
                "--fill",
                "full",
                "--named",
                "1",
                "--repeat",
                "0",
            ],
            &[
                "--arm",
                "split",
                "--devices",
                "1",
                "--fill",
                "full",
                "--named",
                "1",
                "--bogus",
                "1",
            ],
        ];
        for words in refused {
            assert!(parse_the_arguments(words).is_err(), "该拒：{words:?}");
        }
    }

    #[test]
    fn the_checks_lines_state_the_constants_the_ring_and_the_sizes() {
        let lines = run_the_checks();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("constants_match_the_implementation=true"));
        assert!(lines[0].contains("ring_slots=196608 ring_slots_remainder=0"));
        assert!(lines[0].contains("device_holds_the_default_ring=true"));
        assert!(lines[0].contains("capacity_after_196608_pushes=262144"));
        assert!(lines[0].contains("capacity_after_786432_pushes=1048576"));
        assert!(lines[1]
            .starts_with("E7RESULT name=e197_sizes size_location_entry=16 size_named_unit=64"));
    }
}
