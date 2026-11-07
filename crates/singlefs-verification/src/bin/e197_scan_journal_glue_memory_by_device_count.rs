//! E197（扫环胶水内存随盘数）：回答岔路单 `research/prompts/m3-close-c653-forks.md` 的甲（上界）、乙（改法）、丙（读缓存上界）三行——
//! 在默认环几何（每盘 196 608 个记录槽）上，`singlefs_core::recovery::scan_journal` 的内存峰值随盘数怎么涨，
//! 胶水改成逐盘合并（乙二）之后涨不涨，可写挂载读阶段的读缓存（`ReadStageCache`）扫环那一截随盘数怎么涨。
//! 跑前登记 `research/prompts/e197-preregistration.md`（丙那一臂、服务分片的跑法见第十二节修订一）。
//!
// admission: always 每跑一次都在今天这份 crates 上现量一遍内存峰值，上一次跑完不替这一次作保
// run-condition: command python3
//!
//! 一个进程就是一格：`--arm <臂> --devices <盘数> --fill <full|half> --named <点名项数> --repeat <重复号>`。
//! 臂：`baseline`（直接调 `scan_journal`）、`baseline_pc`（收齐之后不释放的阳性对照）、`split`（乙二：核不改、胶水逐盘合并）、
//! `split_pc`（逐盘不释放的阳性对照）、`read_stage_cache`（池外套一层 `ReadStageCache` 的抄本再调不改的 `scan_journal`，C881 那一臂）、
//! `read_stage_cache_keeps_zeros_pc`（缓存连读回全 0 的也收，C881 那一臂的阳性对照）、
//! `empty`（只建池、不扫环，峰值从各臂里扣掉）、`measurement_pc`（量窗里分配并逐页写 256 MiB，测量路径的对照）。
//! 另有 `--diff-set`（差分集 D1–D4：五份胶水各与 `scan_journal` 逐键逐值比）与 `--checks`（本地常量与 `crates/` 回比、锚点、`size_of`）。
//!
//! 进程内量的是常驻页峰值 H（登记修订三）：量窗开头往 `/proc/self/clear_refs` 写 `5` 把这个进程的 `VmHWM` 重置成此刻的常驻量，读 `/proc/self/status` 的 `VmRSS` 当起点；
//! 交回之后、放掉交回值之前（经读缓存的两臂在放掉缓存之前）读 `VmHWM`，H = (VmHWM − 起点) × 1024 字节；口径照 `crates/singlefs-verification/src/gpu_unit_checks.rs` 的 `hold_host_memory_for_the_test_on_this_thread`。
//! 写不进、读不出的那一次，结果行那几个字段写 `unreadable`（登记修订三 V11：那一次作废，不当 0）。H 是整个进程的数：单测里凡要读 H 的，各在自己的子进程里量一格。
//! 装置里没有计数分配器，也没有不安全代码块。
//! 判的数是统一任务服务记下的那一次尝试的 `memory_peak`（scope 的 `memory.peak`），驱动
//! `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py` 让每一片带上任务号与第几次，出表脚本 `report.py` 去状态存档里找。
//! 只读驱动 `singlefs-core` 的公开入口与扫环核，不改 `crates/` 里别的任何文件；产物按 `E7RESULT` 行打印到标准输出。

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
    SystemConfiguration, SystemImmutableConfiguration, SystemImmutableSizes,
    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
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
/// 登记第五节 measurement_pc：量窗里分配并逐页写的字节数（256 MiB）。
const MEASUREMENT_POSITIVE_CONTROL_BYTES: u64 = 268_435_456;
/// 登记修订三第七节 A8：measurement_pc 的 H 落在 [254 MiB, 260 MiB]。往下留 2 MiB 是内核计数的攒批，往上留 4 MiB 是页粒度、读状态文件新碰的堆页与透明大页多出的一页（推的）。
#[cfg(test)]
const MEASUREMENT_WINDOW_LOW_BYTES: i64 = 266_338_304;
#[cfg(test)]
const MEASUREMENT_WINDOW_HIGH_BYTES: i64 = 272_629_760;
/// 登记修订三第七节 A7：H 与常驻页推算值的容差；`predicted_resident_peak_bytes` 的 B 树每叶留的键数（升序插入、标准库分裂点）。
#[cfg(test)]
const PREDICTION_TOLERANCE: f64 = 0.05;
#[cfg(test)]
const ASCENDING_INSERT_KEYS_PER_TREE_LEAF: f64 = 6.0;
/// 装置造的每条记录的实例代号；系统配置的 fsid（16 字节，低 8 字节就是记录头里那 8 字节）。
const PRODUCTION_INSTANCE: u32 = 1;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"E197-scan-glue!!";

/// 登记第六节：每格 `--repeat` 取 1–5 与 6–10（作废重跑的）。
const LAST_REPEAT_NUMBER: u32 = 10;

// ============================================================================================
// 进程内的常驻页峰值 H（登记修订三「进程内那一条量法 H」）：只经 `std::fs` 写 `/proc/self/clear_refs`、读 `/proc/self/status`。
// 量窗开头写 `5` 把 `VmHWM` 重置成此刻的常驻量，读 `VmRSS` 当起点；交回之后读 `VmHWM`；H = (VmHWM − 起点) × 1024。
// 写不进、读不出都交 `None`，结果行写 `unreadable`（V11：那一次作废，不当 0）。
// ============================================================================================

const PROC_SELF_STATUS_FILE: &str = "/proc/self/status";
const PROC_SELF_CLEAR_REFS_FILE: &str = "/proc/self/clear_refs";
const VM_RESIDENT_FIELD: &str = "VmRSS:";
const VM_HIGH_WATER_MARK_FIELD: &str = "VmHWM:";
const BYTES_PER_KIBIBYTE: i64 = 1024;

/// `/proc/self/status` 文本里某个字段（`VmRSS:`、`VmHWM:`）的 KiB 数；字段不在、不是 `kB` 单位、读不成数都交 `None`。
fn kibibytes_of_the_status_field(status_text: &str, field: &str) -> Option<u64> {
    let rest = status_text
        .lines()
        .find_map(|line| line.strip_prefix(field))?;
    let mut words = rest.split_whitespace();
    let kibibytes: u64 = words.next()?.parse().ok()?;
    (words.next()? == "kB").then_some(kibibytes)
}

fn signed_kibibytes(kibibytes: u64) -> i64 {
    i64::try_from(kibibytes).expect("KiB 数装得进 i64")
}

/// 量窗的起点：峰值已重置、此刻的 `VmRSS`（KiB）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ResidentWindow {
    rss_start_kib: u64,
}

/// 量窗的终点：同一次读 `/proc/self/status` 得到的 `VmHWM` 与 `VmRSS`（KiB）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ResidentWindowEnd {
    high_water_mark_kib: u64,
    resident_kib: u64,
}

impl ResidentWindow {
    /// 开窗：往 `/proc/self/clear_refs` 写 `5`，随即读 `VmRSS`。写不进或读不出交 `None`。
    fn open() -> Option<ResidentWindow> {
        ResidentWindow::open_with(PROC_SELF_CLEAR_REFS_FILE, PROC_SELF_STATUS_FILE)
    }

    fn open_with(clear_refs_file: &str, status_file: &str) -> Option<ResidentWindow> {
        std::fs::write(clear_refs_file, "5").ok()?;
        let status_text = std::fs::read_to_string(status_file).ok()?;
        Some(ResidentWindow {
            rss_start_kib: kibibytes_of_the_status_field(&status_text, VM_RESIDENT_FIELD)?,
        })
    }

    /// H：交回之后的 `VmHWM` 减起点，换成字节。
    fn peak_over_the_start_bytes(&self, end: &ResidentWindowEnd) -> i64 {
        (signed_kibibytes(end.high_water_mark_kib) - signed_kibibytes(self.rss_start_kib))
            * BYTES_PER_KIBIBYTE
    }

    /// 某一刻的 `VmRSS`（KiB）减起点，换成字节：轨迹与期末值。
    fn resident_over_the_start_bytes(&self, resident_kib: u64) -> i64 {
        (signed_kibibytes(resident_kib) - signed_kibibytes(self.rss_start_kib)) * BYTES_PER_KIBIBYTE
    }
}

impl ResidentWindowEnd {
    fn read_now() -> Option<ResidentWindowEnd> {
        ResidentWindowEnd::read_from(PROC_SELF_STATUS_FILE)
    }

    fn read_from(status_file: &str) -> Option<ResidentWindowEnd> {
        let status_text = std::fs::read_to_string(status_file).ok()?;
        Some(ResidentWindowEnd {
            high_water_mark_kib: kibibytes_of_the_status_field(
                &status_text,
                VM_HIGH_WATER_MARK_FIELD,
            )?,
            resident_kib: kibibytes_of_the_status_field(&status_text, VM_RESIDENT_FIELD)?,
        })
    }
}

/// 此刻的 `VmRSS`（KiB）；读不出交 `None`。池在每块盘最后一槽被读到时调它（轨迹）。
fn vm_rss_kib_now() -> Option<u64> {
    let status_text = std::fs::read_to_string(PROC_SELF_STATUS_FILE).ok()?;
    kibibytes_of_the_status_field(&status_text, VM_RESIDENT_FIELD)
}

// ============================================================================================
// 本地常量回比、几何锚点（登记第七节 A1–A6）、常驻页峰值的推算（登记修订三第七节 A7）。
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
    /// C881（可写挂载读阶段的读缓存随盘数涨）那一臂：池外套一层 `ReadStageCache` 的抄本（登记修订一第五节）。
    ReadStageCache,
    /// 它的阳性对照：缓存连读回全 0 的也收。
    ReadStageCacheKeepsZerosPc,
}

const ALL_ARMS: [Arm; 8] = [
    Arm::Empty,
    Arm::MeasurementPc,
    Arm::Baseline,
    Arm::BaselinePc,
    Arm::Split,
    Arm::SplitPc,
    Arm::ReadStageCache,
    Arm::ReadStageCacheKeepsZerosPc,
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
            Arm::ReadStageCache => "read_stage_cache",
            Arm::ReadStageCacheKeepsZerosPc => "read_stage_cache_keeps_zeros_pc",
        }
    }

    fn from_name(name: &str) -> Option<Arm> {
        ALL_ARMS.into_iter().find(|arm| arm.name() == name)
    }

    /// 扫环的六个臂；空跑与测量路径对照不扫环。
    fn scans(self) -> bool {
        matches!(
            self,
            Arm::Baseline
                | Arm::BaselinePc
                | Arm::Split
                | Arm::SplitPc
                | Arm::ReadStageCache
                | Arm::ReadStageCacheKeepsZerosPc
        )
    }

    /// 经读缓存扫环的两个臂（C881 那一臂与它的阳性对照）。
    fn reads_through_the_cache(self) -> bool {
        matches!(self, Arm::ReadStageCache | Arm::ReadStageCacheKeepsZerosPc)
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

/// glibc 一块的大小：请求字节加 8 字节块头，向上取到 16 的倍数，最小 32（登记修订三第四节式子；推的，没量）。
#[cfg(test)]
fn glibc_chunk_bytes(request: u64) -> u64 {
    (request + 8).next_multiple_of(16).max(32)
}

/// 登记修订三第七节 A7：每格 H 的常驻页推算 R（`model/resident_prediction.py` 的式子，按量到的 `size_of` 代入）。
/// 记录 Vec、键表过了 glibc 的 mmap 门槛之后按 `mremap` 长，没写过的容量页不常驻，所以只算 len × 元素大小，不算 2 的幂容量；
/// 每条记录的点名项堆算一块 chunk(K × 点名项大小)；两张 B 树按每叶 6 个键摊（叶 = 16 + 11 × 键 + 11 × 值，各算一块 chunk）。
/// baseline 与 split_pc = N·s·(记录 + 点名项块 + 16) + 树；baseline_pc 再多一份首份的点名项堆 s·块；
/// split 在 1 盘同 baseline，2 盘起 = 树 + 2·s·块 + s·记录 + N·s·16（取最后一盘那一刻：首份的点名项堆、这一盘的点名项堆与 Vec、到此为止的键表、两张树）。
#[cfg(test)]
fn predicted_resident_peak_bytes(
    arm: Arm,
    devices: u64,
    filled_slots: u64,
    named_per_record: u64,
    sizes: StructureSizes,
) -> f64 {
    let record = sizes.journal_record;
    let named_heap = glibc_chunk_bytes(named_per_record * sizes.named_unit);
    let leaf_of_records = glibc_chunk_bytes(16 + 11 * 16 + 11 * record);
    let leaf_of_positions = glibc_chunk_bytes(16 + 11 * 16 + 11 * 8);
    let trees = (filled_slots * (leaf_of_records + leaf_of_positions)) as f64
        / ASCENDING_INSERT_KEYS_PER_TREE_LEAF;
    let base = |device_count: u64| {
        (device_count * filled_slots * (record + named_heap + 16)) as f64 + trees
    };
    match arm {
        Arm::Baseline | Arm::SplitPc => base(devices),
        Arm::BaselinePc => base(devices) + (filled_slots * named_heap) as f64,
        Arm::Split => {
            if devices == 1 {
                base(1)
            } else {
                trees
                    + (2 * filled_slots * named_heap) as f64
                    + (filled_slots * record) as f64
                    + (devices * filled_slots * 16) as f64
            }
        }
        // 经读缓存的两臂调的是不改的 `scan_journal`：胶水那一截同基线；缓存那一截由 `predicted_cache_bytes` 另算（登记修订三第七节 A11）。
        Arm::ReadStageCache | Arm::ReadStageCacheKeepsZerosPc => base(devices),
        Arm::Empty | Arm::MeasurementPc => 0.0,
    }
}

/// 登记修订三第七节 A11、A12：读缓存每项 = chunk(4096) 字节的值 + 树叶的摊分（叶 = 16 + 11 × 24 + 11 × 24 = 544 字节的请求，一块 560，升序插入每叶留 6 个键）。
#[cfg(test)]
fn predicted_cache_bytes(entries: u64) -> f64 {
    let leaf_of_the_cache = glibc_chunk_bytes(16 + 11 * 24 + 11 * 24);
    entries as f64
        * (glibc_chunk_bytes(JOURNAL_RECORD_BYTES) as f64
            + leaf_of_the_cache as f64 / ASCENDING_INSERT_KEYS_PER_TREE_LEAF)
}

/// 登记修订三第八节：最后一块盘最后一槽被读到那一刻的常驻页（轨迹的最后一个值）= 读到的记录数 × (记录 + 点名项块) + 环槽数 × 8（这一盘列出的偏移）；
/// 读到的记录数 = 盘数 × 每盘填满槽数，满槽时减去最后一槽（读到它的那一刻这一槽还没解）。
#[cfg(test)]
fn predicted_resident_at_the_last_device_end_bytes(
    devices: u64,
    filled_slots: u64,
    fill: Fill,
    ring_slots: u64,
    named_per_record: u64,
    sizes: StructureSizes,
) -> f64 {
    let read = devices * filled_slots - u64::from(fill == Fill::Full);
    let per_record = sizes.journal_record + glibc_chunk_bytes(named_per_record * sizes.named_unit);
    (read * per_record + ring_slots * 8) as f64
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
    /// 轨迹（登记修订三第八节）：每块盘最后一个环内槽被读到那一刻的 `VmRSS`（KiB，读不出是 `None`）；减量窗起点由跑格那一步做。
    resident_kib_at_each_device_end: RefCell<Vec<Option<u64>>>,
}

impl GeneratedPool {
    fn new(device_count: u32, device_bytes: u64, ring_slots: u64, slot_source: SlotSource) -> Self {
        GeneratedPool {
            device_count,
            device_bytes,
            ring_slots,
            slot_source,
            reads: Cell::new(0),
            resident_kib_at_each_device_end: RefCell::new(Vec::with_capacity(
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
            self.resident_kib_at_each_device_end
                .borrow_mut()
                .push(vm_rss_kib_now());
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
// C881（可写挂载读阶段的读缓存随盘数涨）那一臂（登记修订一第五节）。`ReadStageCache` 是 `crates/singlefs-core/src/mount.rs` 里的私有结构，
// 装置调不到：两行标记之间逐字抄（S4 钉了那一段的 sha256），只做登记写明的三种替换——`crate::address::` 换成 `singlefs_core::address::`、
// 结构名后加 `Copy`、`PoolReader` 换成 `singlefs_core::recovery::PoolReader`；单测
// `the_read_stage_cache_copy_equals_the_mount_section_after_the_three_registered_replacements` 拿仓里那一段做完这三种替换逐字比。
// 整个模块不交 rustfmt 排（`rustfmt::skip`），抄本的换行与缩进才留得住原样。
// ============================================================================================

#[rustfmt::skip]
mod read_stage_cache_copied_from_mount {
use super::*;

// 抄本起：crates/singlefs-core/src/mount.rs 的 ReadStageCache，只做登记修订一第五节许的三种替换
/// 这次挂载内有效的读缓存（C554 乙：E158 第 3 次跑登记第 363 行「乙-配置」那一格；第 4 次跑登记 5.2 第 1 条与 5.5 第 1 条主 agent 的认定）：
/// 读阶段（择根、扫 journal、重放）两遍都经它读。只收读成、而且不是全零的落点的字节；读失败的与读回全零的不收，重读那一遍照样打到盘上——
/// 于是重读那一遍真正打到盘上的，是第一遍读失败或读回全零的落点，与重做时新走到的落点。读阶段判完之后的读直接读盘、不经它。
/// 代价（推的，没量）：扫 journal 时环里写过的每个记录槽都进缓存，环写满一圈之后是环长乘盘数（默认环两块盘约 1.5 GiB），活到读阶段判完为止。
struct ReadStageCacheCopy<'devices, Reader: singlefs_core::recovery::PoolReader> {
    devices: &'devices Reader,
    bytes_read_by_placement: std::cell::RefCell<
        BTreeMap<(DeviceIdentity, singlefs_core::address::DeviceOffsetInBytes, usize), Vec<u8>>,
    >,
}

impl<'devices, Reader: singlefs_core::recovery::PoolReader> ReadStageCacheCopy<'devices, Reader> {
    fn empty_over(devices: &'devices Reader) -> Self {
        ReadStageCacheCopy {
            devices,
            bytes_read_by_placement: std::cell::RefCell::new(BTreeMap::new()),
        }
    }
}

impl<Reader: singlefs_core::recovery::PoolReader> singlefs_core::recovery::PoolReader for ReadStageCacheCopy<'_, Reader> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.devices.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.devices.device_size_in_bytes(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let placement = (device, offset, length);
        if let Some(bytes) = self.bytes_read_by_placement.borrow().get(&placement) {
            return Some(bytes.clone());
        }
        let bytes = self.devices.read(device, offset, length)?;
        if bytes.iter().any(|byte| *byte != 0) {
            self.bytes_read_by_placement
                .borrow_mut()
                .insert(placement, bytes.clone());
        }
        Some(bytes)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: singlefs_core::address::DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<singlefs_core::address::DeviceOffsetInBytes>> {
        self.devices
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}
// 抄本止

/// read_stage_cache_keeps_zeros_pc（登记修订一第五节）：与抄本同，只去掉「有非 0 字节才收」那个条件，读回全 0 的也收。
struct ReadStageCacheKeepingZeros<'devices, Reader: PoolReader> {
    devices: &'devices Reader,
    kept_bytes_by_placement: RefCell<BTreeMap<(DeviceIdentity, DeviceOffsetInBytes, usize), Vec<u8>>>,
}

impl<Reader: PoolReader> PoolReader for ReadStageCacheKeepingZeros<'_, Reader> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.devices.device_identities()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.devices.device_size_in_bytes(device)
    }

    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        let key = (device, offset, length);
        if let Some(bytes) = self.kept_bytes_by_placement.borrow().get(&key) {
            return Some(bytes.clone());
        }
        let bytes = self.devices.read(device, offset, length)?;
        // 阳性对照改的那一处：抄本在这一步前面判 `bytes.iter().any(|byte| *byte != 0)`，这里不判，全 0 的也收。
        self.kept_bytes_by_placement.borrow_mut().insert(key, bytes.clone());
        Some(bytes)
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.devices.journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

/// 缓存里此刻有几项、值一共多少字节（登记修订一第七节 A10）。
trait CachedBytes {
    fn entries_and_value_bytes(&self) -> (u64, u64);
}

fn count_entries_and_value_bytes<Key>(map: &RefCell<BTreeMap<Key, Vec<u8>>>) -> (u64, u64) {
    let map = map.borrow();
    let entries = u64::try_from(map.len()).expect("项数装得进 u64");
    let value_bytes = map.values().map(|bytes| u64::try_from(bytes.len()).expect("一项装得进 u64")).sum();
    (entries, value_bytes)
}

impl<Reader: PoolReader> CachedBytes for ReadStageCacheCopy<'_, Reader> {
    fn entries_and_value_bytes(&self) -> (u64, u64) {
        count_entries_and_value_bytes(&self.bytes_read_by_placement)
    }
}

impl<Reader: PoolReader> CachedBytes for ReadStageCacheKeepingZeros<'_, Reader> {
    fn entries_and_value_bytes(&self) -> (u64, u64) {
        count_entries_and_value_bytes(&self.kept_bytes_by_placement)
    }
}

/// 经缓存扫完环之后、放掉缓存之前的读数：项数、值的字节和、同一刻的 `VmHWM` 与 `VmRSS`（交回的表与缓存都还在，登记修订一第八节「期末值」、修订三 H）。
/// 读不出的 `VmHWM` / `VmRSS` 是 `None`（结果行写 `unreadable`，V11）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CacheReading {
    pub(super) entries: u64,
    pub(super) value_bytes: u64,
    pub(super) resident_end: Option<ResidentWindowEnd>,
}

/// 缓存活到读完期末值（同 `read_stage_settled_at_most_on_the_one_reread`：缓存活到读阶段判完）；读完再放。
fn scan_through<Cache: PoolReader + CachedBytes>(cache: Cache, system_configuration: &SystemConfiguration) -> (RecordTable, CacheReading) {
    let table = scan_journal(&cache, system_configuration);
    let (entries, value_bytes) = cache.entries_and_value_bytes();
    let resident_end = ResidentWindowEnd::read_now();
    drop(cache);
    (table, CacheReading { entries, value_bytes, resident_end })
}

/// read_stage_cache：空缓存起（同 `mount.rs` 里 `read_stage_settled_at_most_on_the_one_reread` 建它的那一步），调不改的 `scan_journal(&缓存, 系统配置)`。
pub(super) fn scan_journal_through_the_read_stage_cache<Reader: PoolReader>(pool: &Reader, system_configuration: &SystemConfiguration) -> (RecordTable, CacheReading) {
    scan_through(ReadStageCacheCopy::empty_over(pool), system_configuration)
}

/// read_stage_cache_keeps_zeros_pc：同上，缓存换成连全 0 也收的那一份。
pub(super) fn scan_journal_through_the_cache_keeping_zeros<Reader: PoolReader>(pool: &Reader, system_configuration: &SystemConfiguration) -> (RecordTable, CacheReading) {
    let cache = ReadStageCacheKeepingZeros { devices: pool, kept_bytes_by_placement: RefCell::new(BTreeMap::new()) };
    scan_through(cache, system_configuration)
}
}

use read_stage_cache_copied_from_mount::{
    scan_journal_through_the_cache_keeping_zeros, scan_journal_through_the_read_stage_cache,
    CacheReading,
};

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
    /// 量窗读数（登记修订三）；写 `5` 失败或任一项读不出是 `None`，结果行那几个字段写 `unreadable`（V11）。
    resident: Option<ResidentReading>,
    scan_seconds: f64,
    table_digest: String,
    /// 经读缓存的两臂：交回之后、放掉缓存之前缓存的项数与值的字节和（登记修订一第七节 A10）；别的臂 `None`（结果行写 none）。
    cache_entries: Option<u64>,
    cache_entries_expected: Option<u64>,
    cache_value_bytes: Option<u64>,
}

/// 一格的量窗读数（登记修订三「进程内那一条量法 H」与「结果行字段」）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ResidentReading {
    /// H：交回之后的 `VmHWM` 减起点，字节。
    peak_over_start_bytes: i64,
    /// 轨迹：每块盘最后一槽被读到那一刻的 `VmRSS` 减起点，字节；不扫环的格是空。
    at_each_device_end_bytes: Vec<i64>,
    /// 期末值：交回之后、放掉交回值之前的 `VmRSS` 减起点，字节。
    after_return_bytes: i64,
    rss_start_kib: u64,
    high_water_mark_kib: u64,
}

impl CellOutcome {
    /// 读数 A2（读者被调 `read` 的次数 = 盘数 × 环槽数，不扫环的格是 0）与 A3（交回的表的键数 = 每盘填满槽数，不扫环的格是 0）都对得上；
    /// 经读缓存的两臂另要 A10：项数 = 登记的项数，值的字节和 = 项数 × 4096。
    fn anchors_hold(&self) -> bool {
        let cache_holds = match (
            self.cache_entries,
            self.cache_entries_expected,
            self.cache_value_bytes,
        ) {
            (None, None, None) => true,
            (Some(entries), Some(expected), Some(value_bytes)) => {
                entries == expected && value_bytes == entries * JOURNAL_RECORD_BYTES
            }
            _ => false,
        };
        self.reads == self.reads_expected
            && self.records_returned == self.records_expected
            && cache_holds
    }

    fn result_line(&self, sizes: StructureSizes) -> String {
        let unreadable = || "unreadable".to_string();
        let (peak, ends, after_return, rss_start, high_water_mark, reset) = match &self.resident {
            Some(reading) => (
                reading.peak_over_start_bytes.to_string(),
                if reading.at_each_device_end_bytes.is_empty() {
                    "none".to_string()
                } else {
                    reading
                        .at_each_device_end_bytes
                        .iter()
                        .map(i64::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                },
                reading.after_return_bytes.to_string(),
                reading.rss_start_kib.to_string(),
                reading.high_water_mark_kib.to_string(),
                "ok".to_string(),
            ),
            None => (
                unreadable(),
                unreadable(),
                unreadable(),
                unreadable(),
                unreadable(),
                unreadable(),
            ),
        };
        let number_or_none = |value: Option<u64>| {
            value.map_or_else(|| "none".to_string(), |count| count.to_string())
        };
        format!(
            "E7RESULT name=e197 {} ring_slots={} filled_slots={} reads={} reads_expected={} records_returned={} records_expected={} anchors_hold={} resident_peak_over_start_bytes={} resident_at_each_device_end_bytes={} resident_after_return_bytes={} vm_rss_at_start_kib={} vm_hwm_kib={} high_water_mark_reset={} scan_seconds={:.6} table_digest={} cache_entries={} cache_entries_expected={} cache_value_bytes={} size_location_entry={} size_named_unit={} size_node_pointer={} size_journal_record={}",
            self.specification_text,
            self.ring_slots,
            self.filled_slots,
            self.reads,
            self.reads_expected,
            self.records_returned,
            self.records_expected,
            self.anchors_hold(),
            peak,
            ends,
            after_return,
            rss_start,
            high_water_mark,
            reset,
            self.scan_seconds,
            self.table_digest,
            number_or_none(self.cache_entries),
            number_or_none(self.cache_entries_expected),
            number_or_none(self.cache_value_bytes),
            sizes.location_entry,
            sizes.named_unit,
            sizes.node_pointer,
            sizes.journal_record,
        )
    }
}

/// 量窗起点、终点和池记下的轨迹合成一格的读数；开窗失败、终点读不出、轨迹里有读不出的，都交 `None`（V11：不当 0）。
fn resident_reading_of(
    window: Option<ResidentWindow>,
    end: Option<ResidentWindowEnd>,
    pool: &GeneratedPool,
) -> Option<ResidentReading> {
    let (window, end) = (window?, end?);
    let at_each_device_end_bytes = pool
        .resident_kib_at_each_device_end
        .borrow()
        .iter()
        .map(|kibibytes| kibibytes.map(|kibibytes| window.resident_over_the_start_bytes(kibibytes)))
        .collect::<Option<Vec<i64>>>()?;
    Some(ResidentReading {
        peak_over_start_bytes: window.peak_over_the_start_bytes(&end),
        at_each_device_end_bytes,
        after_return_bytes: window.resident_over_the_start_bytes(end.resident_kib),
        rss_start_kib: window.rss_start_kib,
        high_water_mark_kib: end.high_water_mark_kib,
    })
}

/// measurement_pc（登记第五节、修订三第七节 A8）：量窗里分配并逐页写一份 256 MiB 的 `Vec<u8>`；缓冲还在的时候读量窗终点。
fn measure_the_positive_control_allocation() -> Option<ResidentWindowEnd> {
    let mut buffer =
        vec![0u8; usize::try_from(MEASUREMENT_POSITIVE_CONTROL_BYTES).expect("256 MiB")];
    for page_start in (0..buffer.len()).step_by(4096) {
        buffer[page_start] = 0xA5;
    }
    std::hint::black_box(&buffer);
    let end = ResidentWindowEnd::read_now();
    drop(buffer);
    end
}

/// 登记修订三第七节 A8：measurement_pc 的 H 落在 [254 MiB, 260 MiB]。
#[cfg(test)]
fn measurement_peak_is_in_the_registered_window(peak_over_start: i64) -> bool {
    (MEASUREMENT_WINDOW_LOW_BYTES..=MEASUREMENT_WINDOW_HIGH_BYTES).contains(&peak_over_start)
}

/// 跑一臂：交回的表，与经读缓存的两臂放掉缓存之前的读数（别的臂 `None`）。
fn run_the_arm_over_the_pool(
    arm: Arm,
    pool: &GeneratedPool,
    system_configuration: &SystemConfiguration,
) -> (RecordTable, Option<CacheReading>) {
    let without_a_cache = |table: RecordTable| (table, None);
    match arm {
        Arm::Baseline => without_a_cache(scan_journal(pool, system_configuration)),
        Arm::BaselinePc => without_a_cache(scan_journal_keeping_the_scan_until_return(
            pool,
            system_configuration,
        )),
        Arm::Split => without_a_cache(scan_journal_merging_device_by_device(
            pool,
            system_configuration,
        )),
        Arm::SplitPc => without_a_cache(
            scan_journal_merging_device_by_device_keeping_the_duplicates(
                pool,
                system_configuration,
            ),
        ),
        Arm::ReadStageCache => {
            let (table, reading) =
                scan_journal_through_the_read_stage_cache(pool, system_configuration);
            (table, Some(reading))
        }
        Arm::ReadStageCacheKeepsZerosPc => {
            let (table, reading) =
                scan_journal_through_the_cache_keeping_zeros(pool, system_configuration);
            (table, Some(reading))
        }
        Arm::Empty | Arm::MeasurementPc => without_a_cache(RecordTable::new()),
    }
}

/// A10：经读缓存的两臂登记的项数。read_stage_cache 只收非 0 的槽：盘数 × 每盘填满槽数；阳性对照全收：盘数 × 环槽数。
fn expected_cache_entries(
    arm: Arm,
    devices: u64,
    filled_slots: u64,
    ring_slots: u64,
) -> Option<u64> {
    if !arm.reads_through_the_cache() {
        return None;
    }
    match arm {
        Arm::ReadStageCache => Some(devices * filled_slots),
        Arm::ReadStageCacheKeepsZerosPc => Some(devices * ring_slots),
        Arm::Empty
        | Arm::MeasurementPc
        | Arm::Baseline
        | Arm::BaselinePc
        | Arm::Split
        | Arm::SplitPc => None,
    }
}

/// 跑一格。`ring_slots` 生产取 [`RING_SLOTS`]，单测取小环。池与系统配置在量窗之前建好；
/// 空跑格还编一条记录（让编码那段代码进内存），量窗里只有扫环那一截。
/// 量窗（登记修订三）：建池、编探针记录之后、调臂之前开窗（重置 `VmHWM`、读 `VmRSS` 当起点）；交回之后、放掉交回值之前读终点
/// （经读缓存的两臂在放掉缓存之前，那一刻由 `scan_through` 读）。
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
        resident: None,
        scan_seconds: 0.0,
        table_digest: "none".to_string(),
        cache_entries: None,
        cache_entries_expected: expected_cache_entries(
            specification.arm,
            devices,
            filled_slots,
            ring_slots,
        ),
        cache_value_bytes: None,
    };
    let window = ResidentWindow::open();
    if specification.arm == Arm::MeasurementPc {
        let end = measure_the_positive_control_allocation();
        outcome.resident = resident_reading_of(window, end, &pool);
        return outcome;
    }
    if !scans {
        outcome.resident = resident_reading_of(window, ResidentWindowEnd::read_now(), &pool);
        return outcome;
    }
    let clock = Instant::now();
    let (table, cache_reading) =
        run_the_arm_over_the_pool(specification.arm, &pool, &system_configuration);
    outcome.scan_seconds = clock.elapsed().as_secs_f64();
    // H 与期末值（登记修订三）：交回之后、放掉交回值之前；经读缓存的两臂取放掉缓存之前那一刻（缓存活到读阶段判完，登记修订一第五节）。
    let end = match cache_reading {
        Some(reading) => reading.resident_end,
        None => ResidentWindowEnd::read_now(),
    };
    outcome.resident = resident_reading_of(window, end, &pool);
    outcome.cache_entries = cache_reading.map(|reading| reading.entries);
    outcome.cache_value_bytes = cache_reading.map(|reading| reading.value_bytes);
    outcome.reads = pool.reads.get();
    outcome.records_returned = u64::try_from(table.len()).expect("表长装得进 u64");
    outcome.table_digest = table_digest(&table);
    outcome
}

// ============================================================================================
// 差分集 D（登记第九节）：小环（64 个槽），盘 4 GiB，不给提示；五份胶水（原三份加登记修订一的两条读缓存臂）各与 `scan_journal` 逐键逐值比。
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
const GLUE_ARMS_COMPARED_WITH_SCAN_JOURNAL: [Arm; 5] = [
    Arm::BaselinePc,
    Arm::Split,
    Arm::SplitPc,
    Arm::ReadStageCache,
    Arm::ReadStageCacheKeepsZerosPc,
];

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
    let (candidate, _) = run_the_arm_over_the_pool(arm, &pool, &system_configuration);
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
    let (candidate, _) = run_the_arm_over_the_pool(specification.arm, &pool, &system_configuration);
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
            "E7RESULT name=e197_sizes size_location_entry={} size_named_unit={} size_node_pointer={} size_journal_record={} size_cache_key={}",
            sizes.location_entry,
            sizes.named_unit,
            sizes.node_pointer,
            sizes.journal_record,
            cache_key_bytes()
        ),
    ]
}

/// 登记修订一第七节 A11：读缓存的键 `(DeviceIdentity, DeviceOffsetInBytes, usize)` 的 `size_of`（推 24：字段重排后 8 + 8 + 4 按 8 对齐）。
fn cache_key_bytes() -> u64 {
    u64::try_from(std::mem::size_of::<(
        DeviceIdentity,
        DeviceOffsetInBytes,
        usize,
    )>())
    .expect("键的大小装得进 u64")
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
    eprintln!("用法：e197_scan_journal_glue_memory_by_device_count --arm <empty|measurement_pc|baseline|baseline_pc|split|split_pc|read_stage_cache|read_stage_cache_keeps_zeros_pc> --devices <N> --fill <full|half> --named <K> --repeat <1..10>");
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
    /// 登记修订一的两条读缓存臂（C881 那一臂与它的阳性对照）。
    const READ_STAGE_CACHE_ARMS: [Arm; 2] = [Arm::ReadStageCache, Arm::ReadStageCacheKeepsZerosPc];

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

    /// 登记修订三第四节 `resident_prediction.py` 用的四个大小（记录 320、点名项 64、位置条目 16、节点指针 56），装置单测 `structure_sizes_match_the_field_by_field_layout_of_the_current_structs` 钉着当前结构体就是这一组。
    const RESIDENT_DESIGN_SIZES: StructureSizes = StructureSizes {
        location_entry: 16,
        named_unit: 64,
        node_pointer: 56,
        journal_record: 320,
    };

    // ----------------------------------------------------------------------------------------
    // H 是整个进程的数：libtest 并行跑的别的用例会串进来，所以凡要读 H 的单测，每一格各起一个只跑一条用例的子进程（测试二进制自己，`--exact` 点名下面那条）量。
    // 一个子进程只量一格：同一个进程里连着量几格，前一格放掉的堆页还常驻，后一格的 H 会偏小。
    // ----------------------------------------------------------------------------------------

    /// 子进程里要量的那一格：值是 `臂,盘数,填满档,点名项数,环槽数`；环境里没有它的用例（不是子进程）什么都不量。
    const CELL_REQUEST_VARIABLE: &str = "E197_TEST_CELL_REQUEST";
    const CELL_CHILD_TEST: &str =
        "tests::the_child_process_measures_the_one_cell_its_environment_names";

    #[test]
    fn the_child_process_measures_the_one_cell_its_environment_names() {
        let Ok(request) = std::env::var(CELL_REQUEST_VARIABLE) else {
            return;
        };
        let words: Vec<&str> = request.split(',').collect();
        let [arm, devices, fill, named, ring_slots] = words.as_slice() else {
            panic!("{CELL_REQUEST_VARIABLE} 要五段（臂,盘数,填满档,点名项数,环槽数）：{request}");
        };
        let specification = CellSpecification {
            arm: Arm::from_name(arm).expect("认得的臂"),
            devices: devices.parse().expect("盘数"),
            fill: Fill::from_name(fill).expect("认得的填满档"),
            named: named.parse().expect("点名项数"),
            repeat: 1,
        };
        let outcome =
            measure_one_cell_in_one_process(specification, ring_slots.parse().expect("环槽数"));
        println!("{}", outcome.result_line(structure_sizes()));
    }

    /// 子进程交回的一格结果行的字段。
    struct MeasuredCell {
        fields: BTreeMap<String, String>,
    }

    impl MeasuredCell {
        fn number(&self, name: &str) -> i64 {
            self.fields[name]
                .parse()
                .unwrap_or_else(|_| panic!("字段 {name} 不是整数：{}", self.fields[name]))
        }

        fn numbers(&self, name: &str) -> Vec<i64> {
            self.fields[name]
                .split(',')
                .map(|word| word.parse().expect("轨迹的每一项是整数"))
                .collect()
        }

        /// H（字节）。
        fn peak(&self) -> f64 {
            self.number("resident_peak_over_start_bytes") as f64
        }
    }

    fn measure_in_a_process_of_its_own(
        arm: Arm,
        devices: u32,
        fill: Fill,
        named: usize,
        ring_slots: u64,
    ) -> MeasuredCell {
        measure_in_a_process_of_its_own_with_environment(arm, devices, fill, named, ring_slots, &[])
    }

    /// 同上，子进程另带 `extra_environment` 里的环境变量（A7 用它固定 glibc 的 mmap 门槛）。
    fn measure_in_a_process_of_its_own_with_environment(
        arm: Arm,
        devices: u32,
        fill: Fill,
        named: usize,
        ring_slots: u64,
        extra_environment: &[(&str, &str)],
    ) -> MeasuredCell {
        let output =
            std::process::Command::new(std::env::current_exe().expect("找得到测试二进制自己"))
                .args([
                    "--exact",
                    CELL_CHILD_TEST,
                    "--nocapture",
                    "--test-threads=1",
                ])
                .envs(extra_environment.iter().copied())
                .env(
                    CELL_REQUEST_VARIABLE,
                    format!(
                        "{},{devices},{},{named},{ring_slots}",
                        arm.name(),
                        fill.name()
                    ),
                )
                .output()
                .expect("起得了子进程");
        let standard_output = String::from_utf8_lossy(&output.stdout).to_string();
        let standard_error = String::from_utf8_lossy(&output.stderr).to_string();
        assert!(
            output.status.success(),
            "子进程量 {} {devices} 盘 {} K={named} 没成：{standard_output}{standard_error}",
            arm.name(),
            fill.name()
        );
        assert!(
            standard_output.contains("running 1 test"),
            "--exact 没点中那一条子进程用例：{standard_output}"
        );
        let line = standard_output
            .lines()
            // 测试框架先打「test <名> ... 」不换行，结果行接在同一行后面
            .find_map(|line| line.find("E7RESULT name=e197 ").map(|start| &line[start..]))
            .unwrap_or_else(|| panic!("子进程没打结果行：{standard_output}{standard_error}"));
        let fields = line
            .strip_prefix("E7RESULT ")
            .expect("以 E7RESULT 起头")
            .split_whitespace()
            .filter_map(|word| word.split_once('='))
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        MeasuredCell { fields }
    }

    /// 单测里写 `/proc/self/status` 的样子与 `clear_refs` 的替身，放在系统临时目录里，用完删。
    fn sample_status_file(label: &str, text: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("e197-{label}-{}.status", std::process::id()));
        std::fs::write(&path, text).expect("写得了临时目录");
        path
    }

    const SAMPLE_STATUS_TEXT: &str = "Name:\te197\nVmPeak:\t  999999 kB\nVmHWM:\t    1300 kB\nVmRSS:\t     900 kB\nRssAnon:\t     100 kB\n";

    #[test]
    fn the_status_fields_are_read_in_kibibytes_and_a_missing_field_or_a_foreign_unit_is_unreadable()
    {
        assert_eq!(
            kibibytes_of_the_status_field(SAMPLE_STATUS_TEXT, VM_HIGH_WATER_MARK_FIELD),
            Some(1300)
        );
        assert_eq!(
            kibibytes_of_the_status_field(SAMPLE_STATUS_TEXT, VM_RESIDENT_FIELD),
            Some(900)
        );
        assert_eq!(
            kibibytes_of_the_status_field(SAMPLE_STATUS_TEXT, "VmSwap:"),
            None
        );
        assert_eq!(
            kibibytes_of_the_status_field("VmRSS:\t12 MB\n", VM_RESIDENT_FIELD),
            None
        );
        assert_eq!(
            kibibytes_of_the_status_field("VmRSS:\tabc kB\n", VM_RESIDENT_FIELD),
            None
        );
        assert_eq!(
            kibibytes_of_the_status_field("VmRSS:\t12\n", VM_RESIDENT_FIELD),
            None
        );
    }

    #[test]
    fn the_window_turns_kibibytes_into_bytes_for_the_peak_and_the_end_values() {
        let window = ResidentWindow {
            rss_start_kib: 1000,
        };
        let end = ResidentWindowEnd {
            high_water_mark_kib: 1300,
            resident_kib: 900,
        };
        assert_eq!(window.peak_over_the_start_bytes(&end), 300 * 1024);
        assert_eq!(
            window.resident_over_the_start_bytes(end.resident_kib),
            -100 * 1024
        );
    }

    #[test]
    fn the_end_reading_takes_the_high_water_mark_field_and_not_the_resident_field() {
        let path = sample_status_file("end", SAMPLE_STATUS_TEXT);
        let end = ResidentWindowEnd::read_from(path.to_str().expect("路径是 UTF-8"));
        std::fs::remove_file(&path).expect("删得掉临时文件");
        assert_eq!(
            end,
            Some(ResidentWindowEnd {
                high_water_mark_kib: 1300,
                resident_kib: 900
            })
        );
        let without_the_field = sample_status_file("end-missing", "Name:\te197\nVmRSS:\t 900 kB\n");
        let missing =
            ResidentWindowEnd::read_from(without_the_field.to_str().expect("路径是 UTF-8"));
        std::fs::remove_file(&without_the_field).expect("删得掉临时文件");
        assert_eq!(missing, None);
        assert_eq!(
            ResidentWindowEnd::read_from("/nonexistent-e197/status"),
            None
        );
    }

    #[test]
    fn a_window_that_cannot_reset_or_read_is_unreadable_and_the_result_line_says_so() {
        let status = sample_status_file("open", SAMPLE_STATUS_TEXT);
        let status_path = status.to_str().expect("路径是 UTF-8");
        let clear_refs = sample_status_file("open-clear-refs", "");
        let clear_refs_path = clear_refs.to_str().expect("路径是 UTF-8");
        assert_eq!(
            ResidentWindow::open_with(clear_refs_path, status_path),
            Some(ResidentWindow { rss_start_kib: 900 })
        );
        assert_eq!(std::fs::read_to_string(&clear_refs).expect("读得回"), "5");
        assert_eq!(
            ResidentWindow::open_with("/nonexistent-e197/clear_refs", status_path),
            None,
            "写不进 clear_refs：不当 0"
        );
        assert_eq!(
            ResidentWindow::open_with(clear_refs_path, "/nonexistent-e197/status"),
            None,
            "读不出 VmRSS：不当 0"
        );
        std::fs::remove_file(&status).expect("删得掉临时文件");
        std::fs::remove_file(&clear_refs).expect("删得掉临时文件");
        let mut outcome = small_cell(Arm::Baseline, 1, Fill::Full, 1, 256);
        let readable_line = outcome.result_line(structure_sizes());
        assert!(
            readable_line.contains(" high_water_mark_reset=ok "),
            "{readable_line}"
        );
        outcome.resident = None;
        let line = outcome.result_line(structure_sizes());
        for field in [
            "resident_peak_over_start_bytes",
            "resident_at_each_device_end_bytes",
            "resident_after_return_bytes",
            "vm_rss_at_start_kib",
            "vm_hwm_kib",
            "high_water_mark_reset",
        ] {
            assert!(
                line.contains(&format!(" {field}=unreadable ")),
                "{field}：{line}"
            );
        }
        let pool = GeneratedPool::new(
            1,
            TEST_IMAGE_DEFAULT_BYTES,
            16,
            production_slot_source(8, 1),
        );
        assert_eq!(
            resident_reading_of(None, ResidentWindowEnd::read_now(), &pool),
            None
        );
        assert_eq!(
            resident_reading_of(Some(ResidentWindow { rss_start_kib: 1 }), None, &pool),
            None
        );
        pool.resident_kib_at_each_device_end.borrow_mut().push(None);
        assert_eq!(
            resident_reading_of(
                Some(ResidentWindow { rss_start_kib: 1 }),
                Some(ResidentWindowEnd {
                    high_water_mark_kib: 2,
                    resident_kib: 2
                }),
                &pool
            ),
            None,
            "轨迹里有一项读不出：整格 unreadable"
        );
    }

    #[test]
    fn this_environment_lets_the_process_reset_and_read_its_own_high_water_mark() {
        let window = ResidentWindow::open()
            .expect("写得进 /proc/self/clear_refs、读得出 VmRSS（读不出时每一格都按 V11 作废）");
        let end = ResidentWindowEnd::read_now().expect("读得出 VmHWM 与 VmRSS");
        assert!(window.rss_start_kib > 0);
        assert!(end.high_water_mark_kib >= end.resident_kib);
    }

    #[test]
    fn the_glibc_chunk_sizes_the_resident_prediction_uses_are_pinned() {
        for (request, chunk) in [
            (1u64, 32u64),
            (24, 32),
            (64, 80),
            (280, 288),
            (544, 560),
            (3712, 3728),
            (4096, 4112),
            (64 * 64, 4112),
        ] {
            assert_eq!(glibc_chunk_bytes(request), chunk, "请求 {request} 字节");
        }
    }

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
    fn measurement_positive_control_lands_in_the_registered_window() {
        // 登记修订三第七节 A8：measurement_pc 的 H 落在 [254 MiB, 260 MiB]；H 是整个进程的数，所以在子进程里量。
        let cell = measure_in_a_process_of_its_own(
            Arm::MeasurementPc,
            1,
            Fill::Full,
            64,
            SMALL_RING_SLOTS,
        );
        assert_eq!(cell.fields["high_water_mark_reset"], "ok");
        let peak_over_start = cell.number("resident_peak_over_start_bytes");
        assert!(
            measurement_peak_is_in_the_registered_window(peak_over_start),
            "测量路径对照的 H {peak_over_start}"
        );
        assert!(measurement_peak_is_in_the_registered_window(266_338_304));
        assert!(measurement_peak_is_in_the_registered_window(272_629_760));
        assert!(!measurement_peak_is_in_the_registered_window(266_338_303));
        assert!(!measurement_peak_is_in_the_registered_window(272_629_761));
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
        for arm in SCANNING_ARMS.into_iter().chain(READ_STAGE_CACHE_ARMS) {
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
                    let reading = outcome
                        .resident
                        .as_ref()
                        .unwrap_or_else(|| panic!("{context} 量窗读不出（V11）"));
                    assert_eq!(
                        reading.at_each_device_end_bytes.len(),
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
        assert_eq!(
            Arm::from_name("read_stage_cache"),
            Some(Arm::ReadStageCache)
        );
        assert_eq!(
            Arm::from_name("read_stage_cache_keeps_zeros_pc"),
            Some(Arm::ReadStageCacheKeepsZerosPc)
        );
        assert!(READ_STAGE_CACHE_ARMS
            .iter()
            .all(|arm| arm.scans() && arm.reads_through_the_cache()));
        assert!(SCANNING_ARMS
            .iter()
            .all(|arm| !arm.reads_through_the_cache()));
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
        // 四例 × 五份胶水（原三份 12 行，登记修订一补两条读缓存臂 8 行）。
        assert_eq!(lines.len(), 20);
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
        let baseline =
            measure_in_a_process_of_its_own(Arm::Baseline, 2, Fill::Full, named, SMALL_RING_SLOTS);
        let control = measure_in_a_process_of_its_own(
            Arm::BaselinePc,
            2,
            Fill::Full,
            named,
            SMALL_RING_SLOTS,
        );
        let extra = control.peak() - baseline.peak();
        // 每条首份记录多一份点名项堆：一块 chunk(K × 点名项大小)（常驻页口径，登记修订三第七节 A7）。
        let expected_extra = (SMALL_RING_SLOTS
            * glibc_chunk_bytes(u64::try_from(named).expect("64") * structure_sizes().named_unit))
            as f64;
        assert!(
            control.peak() * 10.0 > baseline.peak() * 11.0,
            "阳性对照要比基线高出 10% 以上：{} 对 {}",
            control.peak(),
            baseline.peak()
        );
        assert!(
            (extra - expected_extra).abs() * 20.0 < expected_extra,
            "多占的应是首份点名项堆 {expected_extra}（±5%），量到 {extra}"
        );
        assert_eq!(
            baseline.fields["table_digest"],
            control.fields["table_digest"]
        );
    }

    #[test]
    fn merging_device_by_device_stops_the_peak_growing_with_the_device_count_and_keeping_the_duplicates_does_not(
    ) {
        let peak = |arm: Arm, devices: u32| {
            measure_in_a_process_of_its_own(arm, devices, Fill::Full, 64, SMALL_RING_SLOTS).peak()
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
    fn the_prediction_reproduces_the_resident_design_numbers_for_the_default_ring() {
        // 登记修订三第四节 `resident_prediction.py` 输出的 `resident` 各行（记录 320、点名项 64）：满槽 K = 64、半满与 K = 1 各抽几格，单位 GiB，容差 0.002。
        let gibibytes = |arm: Arm, devices: u64, filled: u64, named: u64| {
            predicted_resident_peak_bytes(arm, devices, filled, named, RESIDENT_DESIGN_SIZES)
                / f64::from(1u32 << 30)
        };
        let cases: [(Arm, u64, u64, u64, f64); 12] = [
            (Arm::Baseline, 1, RING_SLOTS, 64, 0.937),
            (Arm::Baseline, 2, RING_SLOTS, 64, 1.751),
            (Arm::Baseline, 3, RING_SLOTS, 64, 2.566),
            (Arm::Baseline, 4, RING_SLOTS, 64, 3.380),
            (Arm::BaselinePc, 4, RING_SLOTS, 64, 4.133),
            (Arm::Split, 2, RING_SLOTS, 64, 1.693),
            (Arm::Split, 4, RING_SLOTS, 64, 1.699),
            (Arm::SplitPc, 4, RING_SLOTS, 64, 3.380),
            (Arm::Baseline, 4, RING_SLOTS / 2, 64, 1.690),
            (Arm::Split, 4, RING_SLOTS / 2, 64, 0.849),
            (Arm::Baseline, 4, RING_SLOTS, 1, 0.427),
            (Arm::Split, 4, RING_SLOTS, 1, 0.222),
        ];
        for (arm, devices, filled, named, expected) in cases {
            let predicted = gibibytes(arm, devices, filled, named);
            assert!(
                (predicted - expected).abs() < 0.002,
                "{} {devices} 盘 {filled} 槽 K={named}：推 {predicted:.4} GiB，登记 {expected}",
                arm.name()
            );
        }
        // 经读缓存的两臂：胶水那一截同基线，另加缓存那一截（登记修订三第四节 `read_stage_cache` 行）。
        for (devices, named, expected) in [(4u64, 64u64, 6.460), (4, 1, 3.507)] {
            let predicted = gibibytes(Arm::ReadStageCache, devices, RING_SLOTS, named)
                + predicted_cache_bytes(devices * RING_SLOTS) / f64::from(1u32 << 30);
            assert!(
                (predicted - expected).abs() < 0.002,
                "read_stage_cache {devices} 盘 K={named}：推 {predicted:.4} GiB，登记 {expected}"
            );
        }
        assert_eq!(
            predicted_resident_peak_bytes(Arm::Empty, 4, RING_SLOTS, 64, RESIDENT_DESIGN_SIZES),
            0.0
        );
    }

    #[test]
    fn the_split_pc_and_baseline_resident_predictions_are_the_same_and_split_equals_baseline_on_one_device(
    ) {
        // 登记修订三第四节：常驻页口径下 split_pc 与 baseline 同式；split 在 1 盘上同 baseline（A9 的依据）。
        for named in [64u64, 1] {
            for devices in 1..=4u64 {
                assert_eq!(
                    predicted_resident_peak_bytes(
                        Arm::SplitPc,
                        devices,
                        RING_SLOTS,
                        named,
                        RESIDENT_DESIGN_SIZES
                    ),
                    predicted_resident_peak_bytes(
                        Arm::Baseline,
                        devices,
                        RING_SLOTS,
                        named,
                        RESIDENT_DESIGN_SIZES
                    ),
                    "{devices} 盘 K={named}"
                );
            }
            assert_eq!(
                predicted_resident_peak_bytes(
                    Arm::Split,
                    1,
                    RING_SLOTS,
                    named,
                    RESIDENT_DESIGN_SIZES
                ),
                predicted_resident_peak_bytes(
                    Arm::Baseline,
                    1,
                    RING_SLOTS,
                    named,
                    RESIDENT_DESIGN_SIZES
                )
            );
        }
    }

    #[test]
    fn the_trajectory_expectation_reproduces_the_registered_last_device_end_numbers() {
        // 登记修订三第四节 `trajectory_last_device_end` 各行，单位 MiB，容差 0.05。
        let mebibytes = |devices: u64, filled: u64, fill: Fill, named: u64| {
            predicted_resident_at_the_last_device_end_bytes(
                devices,
                filled,
                fill,
                RING_SLOTS,
                named,
                RESIDENT_DESIGN_SIZES,
            ) / f64::from(1u32 << 20)
        };
        let cases: [(u64, u64, Fill, u64, f64); 7] = [
            (1, RING_SLOTS, Fill::Full, 64, 832.5),
            (4, RING_SLOTS, Fill::Full, 64, 3325.5),
            (1, RING_SLOTS / 2, Fill::Half, 64, 417.0),
            (4, RING_SLOTS / 2, Fill::Half, 64, 1663.5),
            (1, RING_SLOTS, Fill::Full, 1, 76.5),
            (4, RING_SLOTS, Fill::Full, 1, 301.5),
            (2, RING_SLOTS, Fill::Full, 64, 1663.5),
        ];
        for (devices, filled, fill, named, expected) in cases {
            let predicted = mebibytes(devices, filled, fill, named);
            assert!(
                (predicted - expected).abs() < 0.05,
                "{devices} 盘 {filled} 槽 K={named}：推 {predicted:.2} MiB，登记 {expected}"
            );
        }
    }

    /// 一个扫环格量到的 H 与登记式子推的 R（登记修订三第七节 A7）。
    struct ScanningCellReading {
        label: String,
        arm: Arm,
        devices: u32,
        named: usize,
        ring_slots: u64,
        cell: MeasuredCell,
        predicted: f64,
    }

    impl ScanningCellReading {
        /// H ÷ R − 1。
        fn deviation(&self) -> f64 {
            self.cell.peak() / self.predicted - 1.0
        }

        /// 一格一行：量到的 H、推的 R、比值、差、轨迹、期末值与内核两个计数的原始读数。
        fn row(&self) -> String {
            format!(
                "[{}] {} {}盘 K={} 环={}槽：H={} R={:.0} H÷R−1={:+.4} H−R={:+.0} 轨迹={} 期末={} rss起点KiB={} VmHWM_KiB={}",
                self.label,
                self.arm.name(),
                self.devices,
                self.named,
                self.ring_slots,
                self.cell.number("resident_peak_over_start_bytes"),
                self.predicted,
                self.deviation(),
                self.cell.peak() - self.predicted,
                self.cell.fields["resident_at_each_device_end_bytes"],
                self.cell.number("resident_after_return_bytes"),
                self.cell.number("vm_rss_at_start_kib"),
                self.cell.number("vm_hwm_kib"),
            )
        }
    }

    /// 四个胶水臂在满槽、给定几何与盘数上各量一格（每格一个子进程），带上登记式子推的 R。
    fn read_the_scanning_cells(
        label: &str,
        geometries: &[(usize, u64)],
        device_counts: &[u32],
        extra_environment: &[(&str, &str)],
    ) -> Vec<ScanningCellReading> {
        let sizes = structure_sizes();
        let mut readings = Vec::new();
        for &(named, ring_slots) in geometries {
            for arm in SCANNING_ARMS {
                for &devices in device_counts {
                    let cell = measure_in_a_process_of_its_own_with_environment(
                        arm,
                        devices,
                        Fill::Full,
                        named,
                        ring_slots,
                        extra_environment,
                    );
                    let predicted = predicted_resident_peak_bytes(
                        arm,
                        u64::from(devices),
                        ring_slots,
                        u64::try_from(named).expect("点名项数装得进 u64"),
                        sizes,
                    );
                    readings.push(ScanningCellReading {
                        label: label.to_string(),
                        arm,
                        devices,
                        named,
                        ring_slots,
                        cell,
                        predicted,
                    });
                }
            }
        }
        readings
    }

    /// 单测里 A7 取的几何：（每条记录点名项数 K，环槽数）。
    const A7_UNIT_GEOMETRIES: [(usize, u64); 2] = [(64, 8192), (1, 32768)];

    /// 读数表的表头：这一趟跑在哪种内核、哪几个在线的处理器上（H 与内核的页计数、透明大页有关，两台各不相同时靠它认）。
    /// 处理器只记进表头认机器、不定并行度，与另两样一样从内核的文件里原样读（`/sys/devices/system/cpu/online`，如 `0-31`）。
    fn kernel_facts_line() -> String {
        let text_of = |path: &str| {
            std::fs::read_to_string(path)
                .map_or_else(|_| "读不出".to_string(), |text| text.trim().to_string())
        };
        format!(
            "环境：透明大页={} 内核={} 在线处理器={}",
            text_of("/sys/kernel/mm/transparent_hugepage/enabled"),
            text_of("/proc/sys/kernel/osrelease"),
            text_of("/sys/devices/system/cpu/online")
        )
    }

    /// 单测里量 A7 的子进程另带的环境：把 glibc 的 mmap 门槛钉在 128 KiB。
    /// 默认环境下 glibc 每放掉一块 mmap 出来的大块就把门槛抬到那一块的大小（上限 32 MiB），
    /// 之后中等大小的 Vec 走堆、翻倍留洞，留下的洞随臂、盘数、环槽数没有稳定的形状（登记修订四）；
    /// 钉住门槛，登记 A7 的式子（大 Vec 按 `mremap` 长、没写过的页不常驻）才是单测要核的那个口径。
    const FIXED_MMAP_THRESHOLD_ENVIRONMENT: [(&str, &str); 1] =
        [("MALLOC_MMAP_THRESHOLD_", "131072")];

    #[test]
    fn the_resident_peak_of_every_scanning_arm_stays_within_the_prediction() {
        // 登记修订三第七节 A7：H 对常驻页推算 R 的容差 ±5%，登记没有给 K = 1 单开的容差
        // （这里取登记的 ±5%，不对 K = 1 另放宽）。
        // 每一格都先量完、打出来，再一并判：第一处出界不停，别的格的读数同样要看。
        // 子进程固定 glibc 的 mmap 门槛（登记修订四）：默认环境下 split_pc 3 盘 K = 1 量到的 H 比 R 高约 29%，
        // 固定之后同一格低约 0.9%，二十四格最大偏差约 1.0%。
        let readings = read_the_scanning_cells(
            "mmap门槛固定131072",
            &A7_UNIT_GEOMETRIES,
            &[1, 2, 3],
            &FIXED_MMAP_THRESHOLD_ENVIRONMENT,
        );
        assert_eq!(
            readings.len(),
            A7_UNIT_GEOMETRIES.len() * SCANNING_ARMS.len() * 3
        );
        let table = std::iter::once(kernel_facts_line())
            .chain(readings.iter().map(ScanningCellReading::row))
            .collect::<Vec<_>>()
            .join("\n");
        println!("{table}");
        let outside: Vec<String> = readings
            .iter()
            .filter(|reading| reading.deviation().abs() > PREDICTION_TOLERANCE)
            .map(ScanningCellReading::row)
            .collect();
        assert!(
            outside.is_empty(),
            "A7 的 ±5% 之外 {} 格（共 {} 格）：\n{}\n全部格的读数：\n{table}",
            outside.len(),
            readings.len(),
            outside.join("\n")
        );
    }

    #[test]
    fn on_one_device_split_and_baseline_peak_within_one_percent_of_each_other() {
        // 登记修订三第七节 A9：1 盘上 split 与 baseline 做的是同一串分配，H 之差 ≤ baseline 那一格 H 的 1%。取环大一点，1% 才比内核计数的攒批大。
        for (named, ring_slots) in [(64usize, 32768u64), (1, 131072)] {
            let baseline =
                measure_in_a_process_of_its_own(Arm::Baseline, 1, Fill::Full, named, ring_slots);
            let split =
                measure_in_a_process_of_its_own(Arm::Split, 1, Fill::Full, named, ring_slots);
            let difference = (split.peak() - baseline.peak()).abs() / baseline.peak();
            assert!(
                difference <= 0.01,
                "K={named}：split {} 对 baseline {}，差 {difference}",
                split.peak(),
                baseline.peak()
            );
            assert_eq!(
                baseline.fields["table_digest"],
                split.fields["table_digest"]
            );
        }
    }

    #[test]
    fn the_trajectory_ends_at_the_registered_resident_pages_and_the_peak_is_at_least_the_largest_trajectory_value(
    ) {
        // 登记修订三第八节：baseline 与 baseline_pc 最后一盘那个轨迹值对「读到那一刻的常驻页」落在 ±3%；H ≥ 这一格轨迹里最大的那个数。
        let sizes = structure_sizes();
        for (named, ring_slots) in [(64usize, 8192u64), (1, 65536)] {
            for arm in [Arm::Baseline, Arm::BaselinePc] {
                for devices in [1u32, 2, 3] {
                    let cell = measure_in_a_process_of_its_own(
                        arm,
                        devices,
                        Fill::Full,
                        named,
                        ring_slots,
                    );
                    let ends = cell.numbers("resident_at_each_device_end_bytes");
                    let context = format!("{} {devices} 盘 K={named}", arm.name());
                    assert_eq!(
                        ends.len(),
                        usize::try_from(devices).expect("盘数"),
                        "{context} 轨迹点数"
                    );
                    let last = *ends.last().expect("至少一块盘");
                    let predicted = predicted_resident_at_the_last_device_end_bytes(
                        u64::from(devices),
                        ring_slots,
                        Fill::Full,
                        ring_slots,
                        u64::try_from(named).expect("点名项数装得进 u64"),
                        sizes,
                    );
                    assert!(
                        (last as f64 / predicted - 1.0).abs() <= 0.03,
                        "{context}：最后一盘的轨迹值 {last}，推 {predicted:.0}"
                    );
                    let largest = *ends.iter().max().expect("至少一块盘");
                    assert!(
                        cell.number("resident_peak_over_start_bytes") >= largest,
                        "{context}：H {} 小于轨迹最大值 {largest}",
                        cell.number("resident_peak_over_start_bytes")
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
            "resident_peak_over_start_bytes",
            "resident_at_each_device_end_bytes",
            "resident_after_return_bytes",
            "vm_rss_at_start_kib",
            "vm_hwm_kib",
            "high_water_mark_reset",
            "scan_seconds",
            "table_digest",
            "cache_entries",
            "cache_entries_expected",
            "cache_value_bytes",
            "size_location_entry",
            "size_named_unit",
            "size_node_pointer",
            "size_journal_record",
        ] {
            assert!(fields.contains_key(key), "结果行缺字段 {key}：{line}");
        }
        assert!(
            !fields.keys().any(|key| key.starts_with("allocator_")),
            "登记修订三删掉了计数分配器的三个字段，结果行不再带：{line}"
        );
        assert_eq!(fields["cache_entries"], "none");
        let cached = small_cell(Arm::ReadStageCache, 2, Fill::Half, 3, 256);
        let cached_line = cached.result_line(structure_sizes());
        assert!(
            cached_line.contains(
                " cache_entries=256 cache_entries_expected=256 cache_value_bytes=1048576 "
            ),
            "读缓存臂 2 盘半满（每盘 128 个非 0 槽）：{cached_line}"
        );
        assert_eq!(fields["name"], "e197");
        assert_eq!(fields["arm"], "split");
        assert_eq!(fields["fill"], "half");
        assert_eq!(fields["records_returned"], "128");
        assert_eq!(fields["reads"], "512");
        assert_eq!(fields["anchors_hold"], "true");
        assert_eq!(fields["high_water_mark_reset"], "ok");
        assert_eq!(
            fields["resident_at_each_device_end_bytes"]
                .split(',')
                .count(),
            2
        );
        for name in [
            "resident_peak_over_start_bytes",
            "resident_after_return_bytes",
            "vm_rss_at_start_kib",
            "vm_hwm_kib",
        ] {
            assert!(
                fields[name].parse::<i64>().is_ok(),
                "{name} 该是整数：{}",
                fields[name]
            );
        }
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
        // 登记修订一第七节 A11：读缓存的键 24 字节。
        assert!(lines[1].ends_with(" size_cache_key=24"), "{}", lines[1]);
        assert_eq!(cache_key_bytes(), 24);
    }

    /// 抄本前后的两行标记，与 `read_stage_cache_copied_from_mount` 模块里那两行逐字相同（这里是字符串，所在的行不等于标记行）。
    const COPY_START_MARKER: &str =
        "// 抄本起：crates/singlefs-core/src/mount.rs 的 ReadStageCache，只做登记修订一第五节许的三种替换";
    const COPY_END_MARKER: &str = "// 抄本止";
    /// 登记修订一第十一节 S4：`mount.rs` 那一段从文档注释起共 55 行，`struct ReadStageCache<` 是其中第 5 行（往前 4 行起）。
    const REGISTERED_MOUNT_SECTION_LINES: usize = 55;
    const STRUCT_LINE_OFFSET_IN_THE_SECTION: usize = 4;

    #[test]
    fn the_read_stage_cache_copy_equals_the_mount_section_after_the_three_registered_replacements()
    {
        let mount = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../singlefs-core/src/mount.rs"
        ))
        .expect("读得到 crates/singlefs-core/src/mount.rs");
        let mount_lines: Vec<&str> = mount.split('\n').collect();
        let struct_lines: Vec<usize> = mount_lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.starts_with("struct ReadStageCache<"))
            .map(|(index, _)| index)
            .collect();
        assert_eq!(
            struct_lines.len(),
            1,
            "mount.rs 里 `struct ReadStageCache<` 起头的行要恰好一行"
        );
        let start = struct_lines[0] - STRUCT_LINE_OFFSET_IN_THE_SECTION;
        let section = mount_lines[start..start + REGISTERED_MOUNT_SECTION_LINES].join("\n");
        let replaced = section
            .replace("crate::address::", "singlefs_core::address::")
            .replace("ReadStageCache", "ReadStageCacheCopy")
            .replace("PoolReader", "singlefs_core::recovery::PoolReader");
        let own = include_str!("e197_scan_journal_glue_memory_by_device_count.rs");
        let own_lines: Vec<&str> = own.split('\n').collect();
        let begin = own_lines
            .iter()
            .position(|line| *line == COPY_START_MARKER)
            .expect("装置源文件里有抄本起的标记行");
        let end = own_lines
            .iter()
            .position(|line| *line == COPY_END_MARKER)
            .expect("装置源文件里有抄本止的标记行");
        assert!(begin < end, "标记次序");
        let copy = own_lines[begin + 1..end].join("\n");
        assert_eq!(
            copy, replaced,
            "抄本与 mount.rs 那一段做完三种替换之后不同（S4：正文变了就停，抄本按新实现重抄）"
        );
    }

    #[test]
    fn the_read_stage_cache_holds_one_entry_per_nonzero_slot_per_device_and_its_control_holds_every_slot(
    ) {
        let ring_slots = 256;
        for devices in 1..=3u32 {
            for (fill, filled) in [(Fill::Full, 256u64), (Fill::Half, 128)] {
                let arm = small_cell(Arm::ReadStageCache, devices, fill, 2, ring_slots);
                let control = small_cell(
                    Arm::ReadStageCacheKeepsZerosPc,
                    devices,
                    fill,
                    2,
                    ring_slots,
                );
                let device_count = u64::from(devices);
                let context = format!("{devices} 盘 {}", fill.name());
                assert_eq!(
                    arm.cache_entries,
                    Some(device_count * filled),
                    "{context} 读缓存的项数（A10）"
                );
                assert_eq!(
                    arm.cache_value_bytes,
                    Some(device_count * filled * JOURNAL_RECORD_BYTES),
                    "{context} 读缓存值的字节和（A10）"
                );
                assert_eq!(
                    control.cache_entries,
                    Some(device_count * ring_slots),
                    "{context} 阳性对照的项数"
                );
                assert_eq!(
                    control.cache_value_bytes,
                    Some(device_count * ring_slots * JOURNAL_RECORD_BYTES),
                    "{context} 阳性对照值的字节和"
                );
                assert!(arm.anchors_hold() && control.anchors_hold(), "{context}");
                assert_eq!(arm.table_digest, control.table_digest, "{context}");
            }
        }
    }

    #[test]
    fn the_read_stage_cache_raises_the_peak_and_the_end_value_above_the_baseline_by_the_cache() {
        let named = 3;
        for devices in [1u32, 2] {
            let baseline = measure_in_a_process_of_its_own(
                Arm::Baseline,
                devices,
                Fill::Full,
                named,
                SMALL_RING_SLOTS,
            );
            let cached = measure_in_a_process_of_its_own(
                Arm::ReadStageCache,
                devices,
                Fill::Full,
                named,
                SMALL_RING_SLOTS,
            );
            let predicted = predicted_cache_bytes(u64::from(devices) * SMALL_RING_SLOTS);
            let peak_extra = cached.peak() - baseline.peak();
            let end_extra = (cached.number("resident_after_return_bytes")
                - baseline.number("resident_after_return_bytes"))
                as f64;
            assert!(
                (peak_extra / predicted - 1.0).abs() <= PREDICTION_TOLERANCE,
                "{devices} 盘：H 多出 {peak_extra}，推 {predicted:.0}（A11）"
            );
            assert!(
                (end_extra / predicted - 1.0).abs() <= PREDICTION_TOLERANCE,
                "{devices} 盘：期末值多出 {end_extra}，推 {predicted:.0}（A12）"
            );
        }
    }
}
