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
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
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

mod differential_set_and_checks;
mod generated_pool;
mod one_cell_measurement;
mod resident_memory_window;
mod resident_prediction;
mod scan_journal_glue_arms;

use differential_set_and_checks::*;
use generated_pool::*;
use one_cell_measurement::*;
use resident_memory_window::*;
use resident_prediction::*;
use scan_journal_glue_arms::*;

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

/// A6：四个结构的 `size_of`，打进每一格的结果行，单测钉住。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StructureSizes {
    location_entry: u64,
    named_unit: u64,
    node_pointer: u64,
    journal_record: u64,
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

// ============================================================================================
// C881（可写挂载读阶段的读缓存随盘数涨）那一臂（登记修订一第五节）。`ReadStageCache` 是 `crates/singlefs-core/src/mount.rs` 里的私有结构，
// 装置调不到：两行标记之间逐字抄（S4 钉了那一段的 sha256），只做登记写明的三种替换——`crate::address::` 换成 `singlefs_core::address::`、
// 结构名后加 `Copy`、`PoolReader` 换成 `singlefs_core::recovery::PoolReader`。装置不读核心层的源码：抄本与仓里那一段做完这三种替换
// 逐字比由 `scripts/` 下的对比脚本做（按下面两行标记找抄本）。
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
    fn physical_block_size_probed_on_the_device(&self, device: DeviceIdentity) -> Option<PhysicalBlockSizeInBytes> {
        self.devices.physical_block_size_probed_on_the_device(device)
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
    fn physical_block_size_probed_on_the_device(&self, device: DeviceIdentity) -> Option<PhysicalBlockSizeInBytes> {
        self.devices.physical_block_size_probed_on_the_device(device)
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

// ============================================================================================
// 入口：参数、准入、打印。
// ============================================================================================

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；强制跑（`--force`）时交回摘要，`main` 把它作为头一行打到标准输出。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!(
        "{manifest_directory}/src/bin/e197_scan_journal_glue_memory_by_device_count/main.rs"
    );
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
mod tests;
