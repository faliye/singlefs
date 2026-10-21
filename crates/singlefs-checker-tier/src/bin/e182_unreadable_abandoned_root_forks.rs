//! E182（被抛弃根读不出的三条岔路）跑前登记第一段（单盘择根段）：入库装置。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`。本文件回答岔路单两行（登记 5.4 第一段）：
//! B1「被藏根的记录也读不出、单盘恢复择中它」（丙：今天；甲一：单盘择根时剩下那块盘上读得出更新实例的记录、按它的新根段读得出的
//! 实例表判所选根被抛弃就不择它；甲二：剩下那块盘系统配置记着的实例代号比所选根新就不择它），
//! B4「收严新实例第一次发布避开读不出的根槽对应的 txg」（不改实现，按状态分类）。
//!
//! 臂的名字（产物里写这些，登记里的叫法对应如下）：丙 = `today`；甲一 = `newer_instance_record`；甲二 = `configuration_instance`。
//! 岔路单 B1 的「乙」没有定义「更保守的集合」，登记不建臂。
//!
//! 几何、工作量与判据门槛一律用本文件的本地常量，值抄自跑前登记第五节并注明出处；`run_constants_and_anchors` 逐个与实现侧同名常量回比，
//! 不等就作废（登记第十一节）。不从 `crates/` 引决定工作量或判据门槛的常量。
//!
//! 模式（第一个参数）：
//!   `anchors`            常量回比、第七节锚点、源文件指纹、闭式计数（不建历史）
//!   `list-items`         逐行列出可点名的历史名（登记修订 12.4 的读法 18 条，再接补臂 9 条，名字带 `units-kept.` 前缀）
//!   `measure <历史名>`   在这台机器上量一条历史的状态数与一百次恢复的挂钟（登记 5.5 最后一条；挂钟不进产物）
//!   `single-device [--full] [--item <历史名>]…`
//!                        第一段：建每条历史、按片判（一片 = 一条历史 × 它录制流里的一个段）、片结果按键存取、合并计数、打产物。
//!                        不带参数：剪枝跑（键相同的片直接取库里的）；`--full`：不读库、全部现判、照键写回；`--item`：只跑点名的历史。
//!   `single-device-units-kept [--full] [--item <历史名>]…`
//!                        登记修订 12.11 的补臂：同一套判法，历史换成「实例 1 的每次覆盖写都是 s 个单元的顺序写」的 9 条（只取 s = 64 的起点），
//!                        产物里的历史名与「族 × 几何」合并标签带 `units-kept.` 前缀，config 行多一个 `overwrite_shape` 字段；与 `single-device`
//!                        并列、互不改动。
//!
//! 环境变量：`E182_THREADS`（工作线程数，没设取 `SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）；
//! `E182_VERDICT_LIBRARY`（片结果库目录，没设取 `<git 公共目录>/singlefs-unit-verdicts/e182-single-device/`）；
//! `E182_SHARD_SHARE=<第几份>/<共几份>`（多机分片：片号对份数取余等于第几份的片归这一台，只判、只写库、不打产物）。
//!
//! 复用键 = 本文件判法入口（`record_the_history`、`judge_the_shard`）按 `singlefs-unit-graph` 算出的函数单元闭包指纹（去注释），
//! 加工具链行、历史名、段号、展开方式；文件级指纹与按 HEAD 算的指纹不进键（派发提示定，登记 5.5 的文件级 sha256 键由此替换）。
//! 闭包拒算时这一趟不复用、不入库，stderr 报原因。
//!
//! 准入写 `always` 而不写 `inputs-changed`：复跑登记（`replay.sh` 的 `E182` 两行）与长跑脚本（`--full` 重做全部片、出错后原样重起同一片）
//! 都要在输入不变时再跑同一段、拿同一份输入核产物逐字节相同；写 `inputs-changed` 的话，第二次跑会被判「输入没变」拒绝（退出码 78），
//! 复跑与重做都跑不了。强制跑（`--force`）时产物开头多一行 `name=preflight forced=…`（`list-items` 的标准输出是历史名清单，那一行改写到标准错误），
//! 满足时不打，产物与没有这一层时逐字节相同。准入判法退 78 时装置也退 78。

// admission: always 复跑登记与长跑脚本都要在输入不变时再跑同一段、比逐字节相同的产物，上一次跑完不替这一次作保
// run-condition: command python3

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_checker::image as checker_image;
use singlefs_checker::image::{ImageReader, PoolGeometry};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::JournalRecord;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches, Mounted, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    instance_table_chain_of_root, instance_table_page_pointers_as_far_as_readable, readable_roots,
    recover, root_is_abandoned_by_the_instance_table, scan_journal, JournalPolicy, PoolReader,
    RecoveryOutcome, RecoveryReport,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, publish_sequential_write, FirstFile, PoolVersion,
    PoolWriter, PublishError, TransactionOutput,
};
use singlefs_core::unit::{data_unit_payload_capacity, unit_filesystem_identifier};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::heavy_threads::own_variable_or_default_heavy_threads;
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::sha256::sha256_digest;
use singlefs_harness::{
    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
};

// ============================================================================
// 一、本地常量：值抄自 `research/prompts/e182-preregistration.md` 第五节（"要读 `crates/` 常量的量改成本地常量加回比断言"）。
//    回比在 `run_constants_and_anchors`。
// ============================================================================

/// 落点粒度：登记第五节「槽 16384」。
const SLOT_BYTES_LOCAL: u64 = 16384;
/// 单元区起点（默认环）：登记第五节「U = 50176」。
const UNIT_AREA_START_SLOT_LOCAL: u64 = 50176;
/// 默认 journal 环字节数：登记第五节「默认环 805306368」。
const JOURNAL_RING_BYTES_LOCAL: u64 = 805_306_368;
/// 每区根环槽数 S：登记第五节「S = 8」。
const ROOT_RING_SLOTS_PER_REGION_LOCAL: u64 = 8;
/// 根环区域数：登记第五节「根环区域 = txg mod 3」。
const ROOT_RING_REGIONS_LOCAL: u64 = 3;
/// 根环区域归盘 0 / 1 / 0：登记第五节（D16 已定项 1「正常卸载」一行）。
const REGION_DEVICES_LOCAL: [u32; 3] = [0, 1, 0];
/// 可写挂载暖机零单元发布至多几次：登记第五节「暖机至多 R = 3 次」（D16 已定项 8）。
const WARM_UP_PUBLISHES_AT_MOST_LOCAL: usize = 3;
/// 段内写数不超过它的段全量展开（任意真子集）：登记第五节「全量展开的段内写数上界 16（本登记定，不取 `crates/` 的值）」。
const FULL_EXPANSION_WRITES_AT_MOST_LOCAL: usize = 16;
/// 一个数据单元的净荷字节数：登记 E180 第七节引用的 32634，与 `data_unit_payload_capacity()` 回比。
const DATA_UNIT_PAYLOAD_BYTES_LOCAL: usize = 32634;
/// 默认几何每块盘字节数：登记第一节「两块 4 GiB 盘」。
const DEVICE_BYTES_DEFAULT_LOCAL: u64 = 4 << 30;
/// 窄几何每块盘字节数：登记第一节「两块 3 GiB + 33 槽」。
const DEVICE_BYTES_NARROW_LOCAL: u64 = (3 << 30) + 33 * SLOT_BYTES_LOCAL;
/// 实例 2 挂载之后接着做的覆盖写次数：登记第一节「之后 3 次 `Overwrite`」。
const OVERWRITES_AFTER_THE_MOUNT_LOCAL: u64 = 3;
/// 固定结构槽距（系统配置槽与根环槽）：与 mkfs 默认参数一致。
const FIXED_STRUCTURE_SLOT_SPACING_LOCAL: u32 = 4096;
/// 系统配置槽字节数。
const SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL: usize = 4096;
/// 物理块字节数（内存盘的扇区）。
const PHYSICAL_BLOCK_BYTES_LOCAL: u32 = 512;
/// 一次恢复至多连着避开几条根（甲一、甲二的循环上界；根环只有 3 × 8 个槽）。
const AVOIDED_ROOTS_AT_MOST: usize = 3 * 8;
/// 固定 fsid，与别的实验装置的不同，免得两个进程各自建的镜像撞出同一份「历史上出现过」的巧合。
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x45, 0x31, 0x38, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_800_000_000;

/// 产物完整性闸口径（`scripts/experiments/replay.sh` 文件头）：每条结果行以 `E7RESULT` 起头，收尾 `E7RESULT name=done emitted=N`，
/// N = 本进程打过的 `E7RESULT` 行数（含收尾这一条）。
static EMITTED_RESULT_LINES: AtomicU64 = AtomicU64::new(0);

fn emit_result(line: &str) {
    EMITTED_RESULT_LINES.fetch_add(1, Ordering::SeqCst);
    println!("E7RESULT {line}");
}

fn emitted_result_line_count() -> u64 {
    EMITTED_RESULT_LINES.load(Ordering::SeqCst)
}

// ============================================================================
// 二、历史的名字与几何：一条历史 = (族, 几何, 第一个文件单元数, 藏根之前的覆盖写次数)。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DiskGeometry {
    Default,
    Narrow,
}

impl DiskGeometry {
    fn name(self) -> &'static str {
        match self {
            DiskGeometry::Default => "default",
            DiskGeometry::Narrow => "narrow",
        }
    }
    fn device_bytes(self) -> u64 {
        match self {
            DiskGeometry::Default => DEVICE_BYTES_DEFAULT_LOCAL,
            DiskGeometry::Narrow => DEVICE_BYTES_NARROW_LOCAL,
        }
    }
}

/// 被藏根那一次发布有没有整次返回（登记第一节「两档确认」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Confirmation {
    /// 录制流截在被藏根的根槽 FUA 写之后、系统配置轮换之前（发布没返回）。
    Unconfirmed,
    /// 被藏根那次发布整次返回之后才截；另把每块盘较新的那个系统配置槽也读回全零。
    Confirmed,
}

impl Confirmation {
    fn name(self) -> &'static str {
        match self {
            Confirmation::Unconfirmed => "unconfirmed",
            Confirmation::Confirmed => "confirmed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Family {
    /// 带藏根的历史：被藏根的根槽、那次发布的全部 journal 记录、那次发布新写的全部单元在实例 2 挂载的整个进程里读回全零。
    HiddenRoot(Confirmation),
    /// 阳性对照（登记 5.1「路径真跑到」的补跑）：记录照常读得出，核心层只供测试的影子账开关关着。
    HiddenRootRecordsReadableShadowLedgerOff(Confirmation),
    /// 对照族「无藏根」：同一组起点，实例 2 正常可写挂载，不注任何故障。
    NoHiddenRoot,
}

impl Family {
    fn name(self) -> String {
        match self {
            Family::HiddenRoot(confirmation) => format!("hidden-root-{}", confirmation.name()),
            Family::HiddenRootRecordsReadableShadowLedgerOff(confirmation) => {
                format!("records-readable-shadow-off-{}", confirmation.name())
            }
            Family::NoHiddenRoot => "no-hidden-root".to_string(),
        }
    }
    fn confirmation(self) -> Confirmation {
        match self {
            Family::HiddenRoot(confirmation)
            | Family::HiddenRootRecordsReadableShadowLedgerOff(confirmation) => confirmation,
            Family::NoHiddenRoot => Confirmation::Confirmed,
        }
    }
    fn hides_a_root(self) -> bool {
        match self {
            Family::HiddenRoot(_) | Family::HiddenRootRecordsReadableShadowLedgerOff(_) => true,
            Family::NoHiddenRoot => false,
        }
    }
}

/// 实例 1 里「p 次覆盖写」每一次发布的形状（登记修订 12.4 的读法与修订 12.11 的补臂）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OverwriteShape {
    /// 登记修订 12.4 的读法：每次覆盖写是 `publish_overwrite`，内容一个数据单元，第一次覆盖写之后文件只剩一个单元。
    OneUnit,
    /// 登记修订 12.11 的补臂：每次覆盖写是 `publish_sequential_write`，内容整份 s 个数据单元，文件在每次覆盖写之后仍是 s 个单元，
    /// 被藏根（最后一次覆盖写）引用 s 个数据单元。实例 2 挂载之后的 3 次覆盖写不变（仍是 `publish_overwrite`）。
    KeepingTheFirstFileUnits,
}

impl OverwriteShape {
    /// 产物里历史名与「族 × 几何」合并标签的前缀；登记修订 12.4 的读法不带前缀，名字与第一段产物逐字相同。
    fn label_prefix(self) -> &'static str {
        match self {
            OverwriteShape::OneUnit => "",
            OverwriteShape::KeepingTheFirstFileUnits => "units-kept.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct HistorySpec {
    family: Family,
    geometry: DiskGeometry,
    first_file_units: u64,
    overwrites_before_the_hidden_publish: u64,
    overwrite_shape: OverwriteShape,
}

fn history_name(spec: &HistorySpec) -> String {
    format!(
        "{}{}.{}.units-{}.overwrites-{}",
        spec.overwrite_shape.label_prefix(),
        spec.family.name(),
        spec.geometry.name(),
        spec.first_file_units,
        spec.overwrites_before_the_hidden_publish
    )
}

/// 登记第一节「起点」：G默认 四个起点 (s, p) ∈ {1, 64} × {5, 10}；G窄 取 (64, 10) 与 (1, 5) 两个。
/// 覆盖写保持 s 个单元的补臂（修订 12.11）只取 s = 64 的起点：s = 1 时两种形状的内容同是一个数据单元，不另建。
fn starting_points(geometry: DiskGeometry, shape: OverwriteShape) -> Vec<(u64, u64)> {
    match (shape, geometry) {
        (OverwriteShape::OneUnit, DiskGeometry::Default) => {
            vec![(1, 5), (1, 10), (64, 5), (64, 10)]
        }
        (OverwriteShape::OneUnit, DiskGeometry::Narrow) => vec![(64, 10), (1, 5)],
        (OverwriteShape::KeepingTheFirstFileUnits, DiskGeometry::Default) => {
            vec![(64, 5), (64, 10)]
        }
        (OverwriteShape::KeepingTheFirstFileUnits, DiskGeometry::Narrow) => vec![(64, 10)],
    }
}

/// 某个覆盖写形状的全部历史：每个起点三族（两档确认的带藏根 + 对照族）。
/// 登记修订 12.4 的读法共 12 + 6 = 18 条；覆盖写保持 s 个单元的补臂共 (2 + 1) × 3 = 9 条。
fn history_specs_of_the_shape(shape: OverwriteShape) -> Vec<HistorySpec> {
    let mut specs = Vec::new();
    for geometry in [DiskGeometry::Default, DiskGeometry::Narrow] {
        for (first_file_units, overwrites) in starting_points(geometry, shape) {
            for family in [
                Family::HiddenRoot(Confirmation::Unconfirmed),
                Family::HiddenRoot(Confirmation::Confirmed),
                Family::NoHiddenRoot,
            ] {
                specs.push(HistorySpec {
                    family,
                    geometry,
                    first_file_units,
                    overwrites_before_the_hidden_publish: overwrites,
                    overwrite_shape: shape,
                });
            }
        }
    }
    specs
}

/// 第一段的全部历史（登记修订 12.4 的读法）：共 18 条。
fn main_history_specs() -> Vec<HistorySpec> {
    history_specs_of_the_shape(OverwriteShape::OneUnit)
}

/// 补臂（修订 12.11）的全部历史：共 9 条，与第一段的 18 条并列、名字带前缀 `units-kept.`。
fn units_kept_history_specs() -> Vec<HistorySpec> {
    history_specs_of_the_shape(OverwriteShape::KeepingTheFirstFileUnits)
}

/// 两个形状的全部历史（列项与样本模式按名字找历史用）。
fn every_history_spec() -> Vec<HistorySpec> {
    let mut specs = main_history_specs();
    specs.extend(units_kept_history_specs());
    specs
}

/// 登记 5.1 阳性对照「路径真跑到」的补跑：G默认 的起点、两档确认、记录读得出、影子账关着。
fn shadow_ledger_off_history_specs(shape: OverwriteShape) -> Vec<HistorySpec> {
    let mut specs = Vec::new();
    for (first_file_units, overwrites) in starting_points(DiskGeometry::Default, shape) {
        for confirmation in [Confirmation::Unconfirmed, Confirmation::Confirmed] {
            specs.push(HistorySpec {
                family: Family::HiddenRootRecordsReadableShadowLedgerOff(confirmation),
                geometry: DiskGeometry::Default,
                first_file_units,
                overwrites_before_the_hidden_publish: overwrites,
                overwrite_shape: shape,
            });
        }
    }
    specs
}

fn device_identities() -> [DeviceIdentity; 2] {
    [DeviceIdentity(0), DeviceIdentity(1)]
}

fn pool_parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES_LOCAL,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES_LOCAL,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING_LOCAL,
            journal_ring_bytes: JOURNAL_RING_BYTES_LOCAL,
            root_ring_slots_per_region: root_ring_slots_per_region(),
        },
    }
}

fn root_ring_slots_per_region() -> RootRingSlotsPerRegion {
    RootRingSlotsPerRegion::from_system_configuration_field(ROOT_RING_SLOTS_PER_REGION_LOCAL)
        .expect("S = 8 在 [4, 16] 区间内")
}

fn fixed_geometry_of_the_pool() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING_LOCAL,
        journal_ring_bytes: JOURNAL_RING_BYTES_LOCAL,
        root_ring_slots_per_region: root_ring_slots_per_region(),
    }
}

fn empty_pool(device_bytes: u64) -> MemoryPool {
    MemoryPool::with_devices(&device_identities(), device_bytes)
}

/// 确定性内容：同一个标签同一个长度，两次逐字节相同（不同标签的内容互不相同，被盖的单元读出来认得出是谁的）。
fn deterministic_content(tag: u64, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| {
            let value = u64::try_from(index)
                .expect("下标装得进 u64")
                .wrapping_mul(131)
                .wrapping_add(tag.wrapping_mul(977))
                .wrapping_add(7);
            u8::try_from(value % 251).expect("模 251 装得进 u8")
        })
        .collect()
}

// ============================================================================
// 三、读回全零的设备包装（`singlefs_harness::fault_injection` 里只有读报错、读回翻位，没有读回全零；登记「读法写死」第一条）。
//    按字节区间做；这个区间上一旦有写落下，这个进程之后读它读到的就是真字节（与 E158 装置的 `written_over` 同一个口径）。
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReadsZeroRange {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
    label: &'static str,
}

#[derive(Default)]
struct ReadsZeroState {
    ranges: Vec<ReadsZeroRange>,
    written_over: Vec<bool>,
    intercepted_reads: Vec<u64>,
}

impl ReadsZeroState {
    fn with_ranges(ranges: Vec<ReadsZeroRange>) -> Self {
        let count = ranges.len();
        ReadsZeroState {
            ranges,
            written_over: vec![false; count],
            intercepted_reads: vec![0; count],
        }
    }

    fn mark_written_over(&mut self, device: DeviceIdentity, offset: u64, length: u64) {
        for (range, written_over) in self.ranges.iter().zip(self.written_over.iter_mut()) {
            if range.device == device
                && range.offset < offset + length
                && offset < range.offset + range.length
            {
                *written_over = true;
            }
        }
    }
}

struct ReadsZeroDevice {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    state: Rc<RefCell<ReadsZeroState>>,
}

impl BlockDevice for ReadsZeroDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        let mut state = self.state.borrow_mut();
        let state = &mut *state;
        for ((range, written_over), intercepted) in state
            .ranges
            .iter()
            .zip(state.written_over.iter())
            .zip(state.intercepted_reads.iter_mut())
        {
            if *written_over || range.device != self.identity {
                continue;
            }
            let start = range.offset.max(offset.0);
            let end = (range.offset + range.length).min(offset.0 + length);
            if start >= end {
                continue;
            }
            *intercepted += 1;
            let first = usize::try_from(start - offset.0).expect("缓冲区内偏移");
            let last = usize::try_from(end - offset.0).expect("缓冲区内偏移");
            buffer[first..last].fill(0);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)?;
        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        self.state
            .borrow_mut()
            .mark_written_over(self.identity, offset.0, length);
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)?;
        self.state
            .borrow_mut()
            .mark_written_over(self.identity, offset.0, length);
        Ok(())
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        let outcome = self.inner.discard_at(offset, length)?;
        self.state
            .borrow_mut()
            .mark_written_over(self.identity, offset.0, length);
        Ok(outcome)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

type RecordedDevices = Vec<(DeviceIdentity, RecordingBlockDevice<ReadsZeroDevice>)>;

/// 以 `base` 的镜像为起点的两块录制盘，共用一条录制流与一份读回全零的状态。
fn recording_devices(
    base: &MemoryPool,
    stream: &SharedStream,
    state: &Rc<RefCell<ReadsZeroState>>,
) -> RecordedDevices {
    device_identities()
        .into_iter()
        .map(|identity| {
            let mut inner = SparseBlockDevice::new(
                base.device_size_in_bytes,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES_LOCAL),
            );
            if let Some(image) = base.devices.get(&identity) {
                inner.image = image.clone();
            }
            let faulting = ReadsZeroDevice {
                identity,
                inner,
                state: Rc::clone(state),
            };
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, faulting, stream.clone()),
            )
        })
        .collect()
}

/// 录制盘用完之后的镜像（盘上真有的字节，与读回全零无关）。
fn pool_of_the_recorded_devices(devices: RecordedDevices, device_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| {
                let (faulting, _operations) = device.into_inner_and_operations();
                (identity, faulting.inner.image)
            })
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

// ============================================================================
// 四、全知的历史日志：每条发布过的根（按 (实例代号, txg) 认）引用的每个单元，在这条根的根槽写落下那一刻盘上的字节。
//    「被盖」就是恢复择到的根引用的某个单元，在用到的盘上的字节与这份日志里的不同（逐字节比，经读）。
// ============================================================================

type RootKey = (u32, u64);

fn root_key_of(root: &RootRecord) -> RootKey {
    (root.instance.0, root.checkpoint_txg.0)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UnitLog {
    device: u32,
    offset_in_bytes: u64,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RootLog {
    units: Vec<UnitLog>,
    /// 这条根的分配记录树读得出（读不出时只记根记录直接指着的几个单元）。
    account_readable: bool,
}

/// 一条根引用的落点：它的账里没释放的分配记录（`recovery::allocation_records_under_root`），加根记录直接指着的树表、映射树根、
/// 分配记录树根与实例表链认得出的各片。账读不出时只有后一半，`account_readable` 记假。
fn placements_referenced_by_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> (BTreeSet<(u32, u64, u64)>, bool) {
    let mut placements = BTreeSet::new();
    let account = allocation_records_under_root(reader, root);
    let account_readable = account.is_ok();
    if let Ok(records) = account {
        for record in records.iter().filter(|record| !record.is_released) {
            placements.insert((record.device.0, record.slot.0, u64::from(record.span_slots)));
        }
    }
    let instance_table_pages = instance_table_page_pointers_as_far_as_readable(reader, root);
    for pointer in [
        &root.tree_table,
        &root.mapping_root,
        &root.allocation_record_tree_root,
    ]
    .into_iter()
    .chain(instance_table_pages.iter())
    {
        for location in &pointer.locations {
            if location.slot.0 != 0 {
                placements.insert((location.device.0, location.slot.0, 1));
            }
        }
    }
    (placements, account_readable)
}

fn unit_log_of_root(pool: &MemoryPool, root: &RootRecord) -> RootLog {
    let (placements, account_readable) = placements_referenced_by_root(pool, root);
    let units = placements
        .into_iter()
        .filter_map(|(device, slot, span)| {
            let offset_in_bytes = slot * SLOT_BYTES_LOCAL;
            let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长度装得进 usize");
            PoolReader::read(
                pool,
                DeviceIdentity(device),
                DeviceOffsetInBytes(offset_in_bytes),
                length,
            )
            .map(|bytes| UnitLog {
                device,
                offset_in_bytes,
                bytes,
            })
        })
        .collect();
    RootLog {
        units,
        account_readable,
    }
}

fn is_a_root_slot_write(operation: &RetainedOperation, fixed: &FixedGeometry) -> bool {
    matches!(
        operation.operation.kind,
        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
    ) && fixed.classify(&operation.operation) == StepKind::RootRecordFua
}

fn root_written_by(operation: &RetainedOperation) -> Option<RootRecord> {
    RootRecord::parse_slot(operation.contents.as_ref()?, &FILESYSTEM_IDENTIFIER)
}

/// 在 `start` 上依次施加录制流，每遇到一次根槽写就记下这条根引用的每个单元此刻的字节。
fn log_the_roots_of_a_stream(
    start: &MemoryPool,
    operations: &[RetainedOperation],
) -> BTreeMap<RootKey, RootLog> {
    let fixed = fixed_geometry_of_the_pool();
    let mut pool = start.clone();
    let mut log = BTreeMap::new();
    for operation in operations {
        pool.apply(std::slice::from_ref(operation));
        if !is_a_root_slot_write(operation, &fixed) {
            continue;
        }
        if let Some(root) = root_written_by(operation) {
            log.entry(root_key_of(&root))
                .or_insert_with(|| unit_log_of_root(&pool, &root));
        }
    }
    log
}

// ============================================================================
// 五、实例 1：mkfs → 可写挂载 → 第一个文件 s 单元 → p 次 `Overwrite`，最后一次发布就是被藏根 R_h。
//    登记第一节：第一个文件 s 单元 = 先发一个单元的第一个文件（`publish_first_file` 只收一个数据单元），s 大于 1 再顺序写 s 个单元的整份内容；
//    `Overwrite` 是 `publish_overwrite`（一个数据单元以内的内容，所以第一次覆盖写之后文件只剩一个单元）。
// ============================================================================

/// 实例 1 里每次覆盖写要写几个数据单元：登记修订 12.4 的读法是一个（`publish_overwrite` 的契约），
/// 修订 12.11 的补臂是第一个文件的 s 个。
fn replacing_write_unit_count(spec: &HistorySpec) -> u64 {
    match spec.overwrite_shape {
        OverwriteShape::OneUnit => 1,
        OverwriteShape::KeepingTheFirstFileUnits => spec.first_file_units,
    }
}

/// 实例 1 里的一次覆盖写：读法 12.4 走 `publish_overwrite`（一个数据单元）；补臂 12.11 走 `publish_sequential_write`
/// （整份 s 个数据单元，文件仍是 s 个单元）。内容按标签与单元数确定，两次逐字节相同。
fn publish_the_replacing_write(
    writer: &mut PoolWriter<'_, RecordingBlockDevice<ReadsZeroDevice>>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    spec: &HistorySpec,
    content_tag: u64,
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let content = deterministic_content(
        content_tag,
        usize::try_from(replacing_write_unit_count(spec)).expect("单元数装得进 usize")
            * DATA_UNIT_PAYLOAD_BYTES_LOCAL,
    );
    let file = FirstFile {
        content: &content,
        write_time_seconds,
    };
    match spec.overwrite_shape {
        OverwriteShape::OneUnit => publish_overwrite(writer, allocator, previous, file, instance),
        OverwriteShape::KeepingTheFirstFileUnits => {
            publish_sequential_write(writer, allocator, previous, file, instance)
        }
    }
}

struct FirstInstance {
    /// R_h 那次发布之前的镜像。
    pool_before_the_last_publish: MemoryPool,
    operations: Vec<RetainedOperation>,
    last_publish_start: usize,
    last_root: RootKey,
    roots: BTreeMap<RootKey, RootLog>,
}

fn run_the_first_instance(spec: &HistorySpec) -> Result<FirstInstance, String> {
    let parameters = pool_parameters();
    let device_bytes = spec.geometry.device_bytes();
    let stream = SharedStream::retaining_contents();
    let state = Rc::new(RefCell::new(ReadsZeroState::default()));
    let mut devices = recording_devices(&empty_pool(device_bytes), &stream, &state);
    make_filesystem(&parameters, devices.as_mut_slice())
        .map_err(|error| format!("实例 1 mkfs：{error:?}"))?;
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices)
        .map_err(|error| format!("实例 1 可写挂载：{error:?}"))?;
    let instance = output.instance;
    let PoolVersion::WithoutFile(version) = &current else {
        return Err("实例 1 可写挂载之后现行那一版应当还没有文件".to_string());
    };
    let first_content = deterministic_content(0, DATA_UNIT_PAYLOAD_BYTES_LOCAL);
    let mut previous: TransactionOutput = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &first_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            current.record_bytes(),
        )
        .map_err(|error| format!("第一个文件：{error:?}"))?
    };
    if spec.first_file_units > 1 {
        let content = deterministic_content(
            1,
            usize::try_from(spec.first_file_units).expect("单元数装得进 usize")
                * DATA_UNIT_PAYLOAD_BYTES_LOCAL,
        );
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        previous = publish_sequential_write(
            &mut writer,
            &mut allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .map_err(|error| format!("顺序写 {} 个单元：{error:?}", spec.first_file_units))?;
    }
    let mut last_publish_start = stream.operation_count();
    for overwrite_index in 0..spec.overwrites_before_the_hidden_publish {
        last_publish_start = stream.operation_count();
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        previous = publish_the_replacing_write(
            &mut writer,
            &mut allocator,
            &previous,
            spec,
            100 + overwrite_index,
            FIXED_WRITE_TIME_SECONDS + 120 + overwrite_index,
            instance,
        )
        .map_err(|error| format!("覆盖写 {overwrite_index}：{error:?}"))?;
    }
    let operations = stream.retained_operations();
    let mut pool_before_the_last_publish = empty_pool(device_bytes);
    pool_before_the_last_publish.apply(&operations[..last_publish_start]);
    let roots = log_the_roots_of_a_stream(&empty_pool(device_bytes), &operations);
    Ok(FirstInstance {
        pool_before_the_last_publish,
        operations,
        last_publish_start,
        last_root: root_key_of(&previous.root),
        roots,
    })
}

// ============================================================================
// 六、实例 2：在 R_h 的读回全零下可写挂载（取号、写行、暖机），接着 3 次 `Overwrite`；录制流就是要枚举崩溃状态的那条流。
// ============================================================================

/// 被藏根读回全零的字节区间（登记「读法写死」第一条）：R_h 那次发布写下的每个单元、每条 journal 记录、根槽；
/// 「已确认」档再加那次发布轮换写下的系统配置槽（每块盘较新的那一个）。`applied` 是已经落进基镜像的那次发布的录制流。
/// 记录照常读得出的阳性对照族不罩 journal 记录。
fn fault_ranges_of_the_last_publish(
    family: Family,
    applied: &[RetainedOperation],
    fixed: &FixedGeometry,
) -> Vec<ReadsZeroRange> {
    if !family.hides_a_root() {
        return Vec::new();
    }
    let records_are_hidden_too = matches!(family, Family::HiddenRoot(_));
    let newer_configuration_is_hidden_too = family.confirmation() == Confirmation::Confirmed;
    applied
        .iter()
        .filter_map(|operation| {
            if !matches!(
                operation.operation.kind,
                RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
            ) {
                return None;
            }
            let label = match fixed.classify(&operation.operation) {
                StepKind::UnitWrite => "unit",
                StepKind::JournalRecord if records_are_hidden_too => "journal_record",
                StepKind::JournalRecord => return None,
                StepKind::RootRecordFua => "root_slot",
                StepKind::SystemConfigurationSlot if newer_configuration_is_hidden_too => {
                    "newer_system_configuration_slot"
                }
                StepKind::SystemConfigurationSlot => return None,
                StepKind::ZeroFill | StepKind::Discard | StepKind::Barrier => return None,
            };
            Some(ReadsZeroRange {
                device: operation.operation.device,
                offset: operation.operation.offset.0,
                length: operation.operation.length,
                label,
            })
        })
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct MountReport {
    /// 可写挂载被拒时的错误（拒在取号之前，盘上不动）。
    refused: Option<String>,
    /// 挂载做成之后接着做的覆盖写停下的原因（没停下是 `None`）。
    stopped_after_the_mount: Option<String>,
    instance: Option<u32>,
    row_publish_root: Option<RootKey>,
    warm_up_publishes: usize,
    shadow_ledger_branch: String,
    isolated_slots_per_device: Vec<(u32, u64)>,
    abandoned_roots_unreadable: u64,
    /// 核心层记着「槽里住着一条被抛弃的根」的根环槽 (区域, 槽)。
    ring_slots_recorded_as_holding_an_abandoned_root: Vec<(u64, u64)>,
    /// 读回全零的区间各被读拦了几次，按标签合计。
    intercepted_reads_by_label: BTreeMap<String, u64>,
    /// 这个进程之后写盖了的区间（读回全零就此解开），按标签合计。
    ranges_written_over_by_label: BTreeMap<String, u64>,
}

struct SecondInstance {
    operations: Vec<RetainedOperation>,
    mount: MountReport,
    /// 基镜像加上整条录制流 = 盘上真有的字节（录制流重建镜像的自检）。
    replay_equals_the_devices: bool,
    /// 盘上真有的字节与基镜像逐字节相同（挂载被拒、一个字节没写时为真；A4 用）。
    devices_equal_the_base: bool,
}

fn run_the_second_instance(
    spec: &HistorySpec,
    base: &MemoryPool,
    ranges: Vec<ReadsZeroRange>,
) -> SecondInstance {
    let parameters = pool_parameters();
    let stream = SharedStream::retaining_contents();
    let state = Rc::new(RefCell::new(ReadsZeroState::with_ranges(ranges)));
    let mut devices = recording_devices(base, &stream, &state);
    let mut report = MountReport::default();
    let shadow_ledger_is_off = matches!(
        spec.family,
        Family::HiddenRootRecordsReadableShadowLedgerOff(_)
    );
    let mount_result = if shadow_ledger_is_off {
        mount_writable_with_test_only_switches(
            &parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::Off,
        )
    } else {
        mount_writable(&parameters, &mut devices)
    };
    match mount_result {
        Err(error) => report.refused = Some(format!("{error:?}")),
        Ok(mounted) => keep_going_after_the_mount(&parameters, &mut devices, mounted, &mut report),
    }
    for (range, (intercepted, written_over)) in {
        let state = state.borrow();
        state
            .ranges
            .iter()
            .cloned()
            .zip(
                state
                    .intercepted_reads
                    .iter()
                    .copied()
                    .zip(state.written_over.iter().copied()),
            )
            .collect::<Vec<_>>()
    } {
        *report
            .intercepted_reads_by_label
            .entry(range.label.to_string())
            .or_insert(0) += intercepted;
        *report
            .ranges_written_over_by_label
            .entry(range.label.to_string())
            .or_insert(0) += u64::from(written_over);
    }
    let operations = stream.retained_operations();
    let mut replayed = base.clone();
    replayed.apply(&operations);
    let on_the_devices = pool_of_the_recorded_devices(devices, base.device_size_in_bytes);
    SecondInstance {
        operations,
        mount: report,
        replay_equals_the_devices: replayed == on_the_devices,
        devices_equal_the_base: &on_the_devices == base,
    }
}

/// 挂载做成之后：记下挂载报的东西，再接着做 3 次 `Overwrite`（录制流接着往下记）。
fn keep_going_after_the_mount(
    parameters: &MakeFilesystemParameters,
    devices: &mut RecordedDevices,
    mounted: Mounted,
    report: &mut MountReport,
) {
    let Mounted {
        output,
        mut allocator,
        current,
    } = mounted;
    report.instance = Some(output.instance.0);
    report.row_publish_root = Some(root_key_of(output.row_publish.root()));
    report.warm_up_publishes = output.warm_up_publishes.len();
    report.shadow_ledger_branch = output.shadow_ledger_branch.to_string();
    report.isolated_slots_per_device = output
        .isolated_slots_per_device
        .iter()
        .map(|(device, slots)| (device.0, *slots))
        .collect();
    report.abandoned_roots_unreadable = output.abandoned_roots_unreadable;
    report.ring_slots_recorded_as_holding_an_abandoned_root = allocator
        .root_ring_occupancy()
        .map(|occupancy| {
            occupancy
                .ring_slots_recorded_as_holding_an_abandoned_root()
                .keys()
                .map(|ring_slot| (ring_slot.region, ring_slot.slot))
                .collect()
        })
        .unwrap_or_default();
    let instance = output.instance;
    let mut previous = match current {
        PoolVersion::WithFile(version) => version,
        PoolVersion::WithTreesAndNoFile(_) | PoolVersion::WithoutFile(_) => {
            report.stopped_after_the_mount =
                Some("挂载之后现行那一版没有可覆盖写的文件".to_string());
            return;
        }
    };
    for overwrite_index in 0..OVERWRITES_AFTER_THE_MOUNT_LOCAL {
        let content = deterministic_content(1000 + overwrite_index, DATA_UNIT_PAYLOAD_BYTES_LOCAL);
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        match publish_overwrite(
            &mut writer,
            &mut allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 10_000 + overwrite_index,
            },
            instance,
        ) {
            Ok(next) => previous = next,
            Err(error) => {
                report.stopped_after_the_mount =
                    Some(format!("挂载之后第 {overwrite_index} 次覆盖写：{error:?}"));
                return;
            }
        }
    }
}

// ============================================================================
// 七、一条录好的历史：基镜像、实例 2 的写表与段、全知日志、被藏根、挂载报告。
// ============================================================================

struct RecordedHistory {
    spec: HistorySpec,
    pool_geometry: PoolGeometry,
    /// 实例 2 起头的镜像（实例 1 的盘，已按确认档截好）。
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    roots: BTreeMap<RootKey, RootLog>,
    /// 全知日志认的被抛弃根（带藏根的族是 R_h，对照族没有）。
    hidden_root: Option<RootKey>,
    fault_ranges: Vec<ReadsZeroRange>,
    mount: MountReport,
    /// 实例 2 的第一次根槽写（写行那次发布）在写表里的下标。
    row_publish_root_write: Option<usize>,
    /// 自检里没过的（录制流重建的镜像与盘上真字节不同……），空才算这条历史可信。
    consistency_problems: Vec<String>,
}

fn independent_geometry(pool: &MemoryPool) -> Result<PoolGeometry, String> {
    let chosen = checker_image::chosen_system_configurations(pool);
    let mut geometries = Vec::new();
    for (device, entry) in &chosen {
        match entry {
            Some((_, geometry)) => geometries.push(*geometry),
            None => return Err(format!("设备 {device} 没有自证的系统配置")),
        }
    }
    let first = *geometries.first().ok_or_else(|| "池里没有盘".to_string())?;
    if geometries.iter().any(|other| other != &first) {
        return Err("两盘系统配置解出的几何不一致".to_string());
    }
    Ok(first)
}

/// 实例 2 起头的镜像（实例 1 的盘，按确认档截好）与被藏根读回全零的字节区间。
fn base_and_fault_ranges(
    spec: &HistorySpec,
    first: &FirstInstance,
) -> Result<(MemoryPool, Vec<ReadsZeroRange>), String> {
    let fixed = fixed_geometry_of_the_pool();
    let final_operations = &first.operations[first.last_publish_start..];
    let root_write_position = final_operations
        .iter()
        .position(|operation| {
            is_a_root_slot_write(operation, &fixed)
                && root_written_by(operation).map(|root| root_key_of(&root))
                    == Some(first.last_root)
        })
        .ok_or_else(|| "被藏根那次发布的录制流里没有它的根槽写".to_string())?;
    let applied_final = match spec.family.confirmation() {
        Confirmation::Unconfirmed => &final_operations[..=root_write_position],
        Confirmation::Confirmed => final_operations,
    };
    let mut base = first.pool_before_the_last_publish.clone();
    base.apply(applied_final);
    let fault_ranges = fault_ranges_of_the_last_publish(spec.family, applied_final, &fixed);
    Ok((base, fault_ranges))
}

/// 判法入口之一（函数单元闭包指纹的入口，见文件头）：建一条历史。
fn record_the_history(spec: &HistorySpec) -> Result<RecordedHistory, String> {
    let first = run_the_first_instance(spec)?;
    let fixed = fixed_geometry_of_the_pool();
    let (base, fault_ranges) = base_and_fault_ranges(spec, &first)?;
    let second = run_the_second_instance(spec, &base, fault_ranges.clone());
    let (writes, segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(&second.operations, &fixed);
    let mut roots = first.roots;
    roots.extend(log_the_roots_of_a_stream(&base, &second.operations));
    let row_publish_root_write = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua);
    let mut consistency_problems = Vec::new();
    if !second.replay_equals_the_devices {
        consistency_problems.push("基镜像加录制流重建的镜像与盘上真字节不同".to_string());
    }
    let pool_geometry = independent_geometry(&base)?;
    Ok(RecordedHistory {
        spec: *spec,
        pool_geometry,
        base,
        writes,
        segments,
        roots,
        hidden_root: spec.family.hides_a_root().then_some(first.last_root),
        fault_ranges,
        mount: second.mount,
        row_publish_root_write,
        consistency_problems,
    })
}

// ============================================================================
// 八、崩溃状态：按崩溃点重放同一套取法切段（`writes_and_segments_with_stream_indexes`）；一个状态 = 之前的段全持久 + 这一段的一个子集。
//    段内写不超过 `FULL_EXPANSION_WRITES_AT_MOST_LOCAL` 条的段全量展开（任意真子集）；超过的取崩溃点重放快档的取法
//    （`crash::quick_tier_expansion` 的 `InPlaceSubsetsWithCopyOnWriteNoneOrAll`：段内原地写任意子集 × 单元写只取全不落或全落）。
//    全持久那个状态是下一段的空子集，整条流末尾另算一个。这里的状态只有「没持久 / 持久」两态（登记「全部崩溃状态」写的是任意子集，
//    I2 的 2^m 也是这个口径），不取原地覆写的第三态「新旧都读不出」。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expansion {
    FullSubsets,
    QuickInPlaceSubsetsTimesUnitWritesNoneOrAll,
    FinalState,
}

impl Expansion {
    fn name(self) -> &'static str {
        match self {
            Expansion::FullSubsets => "full_subsets",
            Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll => {
                "quick_in_place_subsets_times_unit_writes_none_or_all"
            }
            Expansion::FinalState => "final_state",
        }
    }
}

fn expansion_of_segment(segment_write_count: usize) -> Expansion {
    if segment_write_count <= FULL_EXPANSION_WRITES_AT_MOST_LOCAL {
        Expansion::FullSubsets
    } else {
        Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll
    }
}

/// 一段里的原地写与单元写（单元写落在新分出来的槽上，其余都原地覆盖；与崩溃点重放引擎同一条分法）。
fn in_place_and_unit_writes_of(
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    segment
        .iter()
        .copied()
        .partition(|write_index| writes[*write_index].kind != StepKind::UnitWrite)
}

/// 这一段按它的展开方式出几个状态（全持久那个不算）；展不开（指数越过 62）交 `None`。
fn state_count_of_segment(writes: &[RetainedWrite], segment: &[usize]) -> Option<u64> {
    match expansion_of_segment(segment.len()) {
        Expansion::FullSubsets => {
            let exponent = u32::try_from(segment.len())
                .ok()
                .filter(|bits| *bits <= 62)?;
            Some((1u64 << exponent) - 1)
        }
        Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll => {
            let (in_place, unit_writes) = in_place_and_unit_writes_of(writes, segment);
            let exponent = u32::try_from(in_place.len())
                .ok()
                .filter(|bits| *bits <= 40)?;
            let unit_write_choices: u64 = if unit_writes.is_empty() { 1 } else { 2 };
            Some((1u64 << exponent) * unit_write_choices - 1)
        }
        Expansion::FinalState => Some(0),
    }
}

/// 第 `segment_index` 段的每个状态：交 `visit` 一份「写表里每次写持不持久」。返回出了几个状态。
/// `segment_index` 等于段数时是整条流末尾那个全持久的状态。
fn for_each_state_of_segment(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    segment_index: usize,
    visit: &mut dyn FnMut(&[bool]) -> Result<(), String>,
) -> Result<u64, String> {
    let mut persisted = vec![false; writes.len()];
    for earlier in segments.iter().take(segment_index) {
        for write_index in earlier {
            persisted[*write_index] = true;
        }
    }
    let Some(segment) = segments.get(segment_index) else {
        visit(&persisted_with_every_write(writes.len()))?;
        return Ok(1);
    };
    let expected = state_count_of_segment(writes, segment).ok_or_else(|| {
        format!(
            "第 {segment_index} 段（{} 次写）的状态数展不开",
            segment.len()
        )
    })?;
    let mut visited: u64 = 0;
    match expansion_of_segment(segment.len()) {
        Expansion::FullSubsets => {
            // 全 1 的掩码 2^m − 1（= expected）就是全持久，已经是下一段的空子集，所以只取 0..expected。
            for mask in 0..expected {
                for (bit, write_index) in segment.iter().enumerate() {
                    persisted[*write_index] = (mask >> bit) & 1 == 1;
                }
                visit(&persisted)?;
                visited += 1;
            }
        }
        Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll => {
            let (in_place, unit_writes) = in_place_and_unit_writes_of(writes, segment);
            let unit_write_choices: u64 = if unit_writes.is_empty() { 1 } else { 2 };
            let in_place_masks: u64 = 1u64 << in_place.len();
            for unit_choice in 0..unit_write_choices {
                for write_index in &unit_writes {
                    persisted[*write_index] = unit_choice == 1;
                }
                for mask in 0..in_place_masks {
                    if unit_choice + 1 == unit_write_choices && mask + 1 == in_place_masks {
                        continue;
                    }
                    for (bit, write_index) in in_place.iter().enumerate() {
                        persisted[*write_index] = (mask >> bit) & 1 == 1;
                    }
                    visit(&persisted)?;
                    visited += 1;
                }
            }
        }
        Expansion::FinalState => {}
    }
    if visited != expected {
        return Err(format!(
            "第 {segment_index} 段枚举出 {visited} 个状态，闭式是 {expected}"
        ));
    }
    Ok(visited)
}

fn persisted_with_every_write(write_count: usize) -> Vec<bool> {
    vec![true; write_count]
}

// ============================================================================
// 九、单盘恢复的读者：崩溃镜像上只留一块盘（另一块整块掉了）；甲一、甲二避开某条根时，把那条根的根槽读成全零（再择就取下一条）。
// ============================================================================

struct KeptDiskView<'image, 'base> {
    image: &'image CrashImage<'base>,
    device: DeviceIdentity,
    /// 被避开的根的根槽字节区间 (偏移, 长度)，读回全零。
    masked_root_slots: Vec<(u64, u64)>,
}

impl PoolReader for KeptDiskView<'_, '_> {
    /// 掉了的那块盘的身份还在盘表里，只是读不到（与 harness 档「盘整块掉了」的两种形态同一个口径：
    /// `crates/singlefs-harness/tests/whole_device_loss_after_warm_up.rs`）。把它的身份也拿掉，分配记录树的走读会按一块盘的几何判，
    /// 每个状态都报 `InvariantViolated` I-1.1（装置第一次样本里的读数），那不是单盘恢复。
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.image)
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.image, device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        if device != self.device {
            return None;
        }
        let mut bytes = PoolReader::read(self.image, device, offset, length)?;
        let read_end = offset.0 + u64::try_from(length).expect("读长装得进 u64");
        for (masked_offset, masked_length) in &self.masked_root_slots {
            let start = (*masked_offset).max(offset.0);
            let end = (masked_offset + masked_length).min(read_end);
            if start < end {
                let first = usize::try_from(start - offset.0).expect("缓冲区内偏移");
                let last = usize::try_from(end - offset.0).expect("缓冲区内偏移");
                bytes[first..last].fill(0);
            }
        }
        Some(bytes)
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        if device != self.device {
            return Some(Vec::new());
        }
        PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes)
    }
}

impl ImageReader for KeptDiskView<'_, '_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.image)
    }

    fn device_bytes(&self, device: u32) -> Option<u64> {
        PoolReader::device_size_in_bytes(self, DeviceIdentity(device))
    }

    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        PoolReader::read(
            self,
            DeviceIdentity(device),
            DeviceOffsetInBytes(offset),
            length,
        )
    }

    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        if device == self.device.0 {
            ImageReader::candidate_unit_slots(self.image, device)
        } else {
            Some(Vec::new())
        }
    }

    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        if device == self.device.0 {
            ImageReader::candidate_journal_slots(self.image, device)
        } else {
            Some(Vec::new())
        }
    }
}

fn kept_disk<'image, 'base>(
    image: &'image CrashImage<'base>,
    device: DeviceIdentity,
) -> KeptDiskView<'image, 'base> {
    KeptDiskView {
        image,
        device,
        masked_root_slots: Vec::new(),
    }
}

// ----------------------------------------------------------------------------
// 被盖：恢复择到的根引用的某个单元，在用到的盘上的字节与全知日志里这条根写下时的字节不同（逐字节比，经读）。
// ----------------------------------------------------------------------------

fn unit_differs_in_the_image(image: &CrashImage<'_>, unit: &UnitLog) -> bool {
    match PoolReader::read(
        image,
        DeviceIdentity(unit.device),
        DeviceOffsetInBytes(unit.offset_in_bytes),
        unit.bytes.len(),
    ) {
        Some(read_back) => read_back != unit.bytes,
        None => true,
    }
}

/// 这条根引用的单元里有没有被盖的；`only_on_device` 给了盘号就只看落在那块盘上的单元（单盘恢复用得到的那一份），不给就两块盘都看。
fn some_unit_of_the_root_is_covered(
    image: &CrashImage<'_>,
    log: &RootLog,
    only_on_device: Option<u32>,
) -> bool {
    log.units
        .iter()
        .filter(|unit| only_on_device.is_none_or(|device| unit.device == device))
        .any(|unit| unit_differs_in_the_image(image, unit))
}

// ============================================================================
// 十、计数、三条臂（丙 / 甲一 / 甲二）与一个状态上的分类。
// ============================================================================

/// 一片（或整条历史）的计数：名字 → 个数，按名字排序；合并是逐名相加，所以与片的次序无关。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Counters(BTreeMap<String, u64>);

impl Counters {
    fn add(&mut self, name: &str, amount: u64) {
        *self.0.entry(name.to_string()).or_insert(0) += amount;
    }

    fn get(&self, name: &str) -> u64 {
        self.0.get(name).copied().unwrap_or(0)
    }

    fn merge(&mut self, other: &Counters) {
        for (name, amount) in &other.0 {
            self.add(name, *amount);
        }
    }

    fn to_text(&self) -> String {
        self.0
            .iter()
            .map(|(name, amount)| format!("{name}\t{amount}\n"))
            .collect()
    }

    fn from_text(text: &str) -> Result<Counters, String> {
        let mut counters = Counters::default();
        for line in text.lines().filter(|line| !line.starts_with('#')) {
            let (name, amount) = line
                .split_once('\t')
                .ok_or_else(|| format!("计数行没有制表符：{line}"))?;
            let amount = amount
                .parse::<u64>()
                .map_err(|error| format!("计数行 {name} 的值不是整数：{error}"))?;
            counters.add(name, amount);
        }
        Ok(counters)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    /// 丙（今天，真实基线）：照 `recovery::recover` 择。
    Today,
    /// 甲一：剩下那块盘上读得出更新实例的记录、按它的新根段读得出的实例表判所选根被抛弃，就不择它。
    NewerInstanceRecord,
    /// 甲二：剩下那块盘上系统配置记着的实例代号比所选根新，就不择它。
    ConfigurationInstance,
}

impl Arm {
    const EVERY_ARM: [Arm; 3] = [
        Arm::Today,
        Arm::NewerInstanceRecord,
        Arm::ConfigurationInstance,
    ];

    fn name(self) -> &'static str {
        match self {
            Arm::Today => "today",
            Arm::NewerInstanceRecord => "newer_instance_record",
            Arm::ConfigurationInstance => "configuration_instance",
        }
    }
}

/// 甲二的判：系统配置里记着的实例代号严格大于所选根的实例代号才避开。
fn avoided_by_configuration_instance(configuration_instance: u32, root_instance: u32) -> bool {
    configuration_instance > root_instance
}

/// 剩下那块盘上自证过的系统配置槽里记着的实例代号的最大值（两个槽都读）。
fn newest_configuration_instance_on_the_kept_disk(
    reader: &dyn PoolReader,
    device: DeviceIdentity,
) -> Option<u32> {
    (0..2u64)
        .filter_map(|slot_index| {
            let bytes = reader.read(
                device,
                DeviceOffsetInBytes(slot_index * u64::from(FIXED_STRUCTURE_SLOT_SPACING_LOCAL)),
                SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
            )?;
            SystemConfiguration::parse_slot(&bytes).ok()
        })
        .map(|configuration| configuration.quantities.journal_instance.0)
        .max()
}

/// 甲一的判：读者只给剩下那一块盘。找实例代号比所选根新、自证过的记录里最新的一条，读它新根段点名的实例表，按实例表的行判所选根被抛弃。
/// 看不见更新实例的记录、或表读不出，都不避开（与今天相同）。
fn avoided_by_newer_instance_record(reader: &dyn PoolReader, root: &RootRecord) -> bool {
    let Ok(configuration) = choose_system_configuration(reader) else {
        return false;
    };
    let records = scan_journal(reader, &configuration);
    let Some(newest_newer_record) = records
        .values()
        .filter(|record| record.instance.0 > root.instance.0)
        .max_by_key(|record| (record.instance.0, record.counter))
    else {
        return false;
    };
    let mut root_with_the_newer_instance_table = *root;
    root_with_the_newer_instance_table.instance_table = newest_newer_record.new_instance_table;
    match instance_table_chain_of_root(reader, &root_with_the_newer_instance_table) {
        Ok(chain) => root_is_abandoned_by_the_instance_table(root, &chain.records),
        Err(_) => false,
    }
}

fn outcome_class(outcome: &RecoveryOutcome) -> &'static str {
    match outcome {
        RecoveryOutcome::FileRead { .. } => "file_read",
        RecoveryOutcome::NoFile { .. } => "no_file",
        RecoveryOutcome::Failed { root: Some(_), .. } => "failed_with_root",
        RecoveryOutcome::Failed { root: None, .. } => "failed_without_root",
    }
}

/// 恢复报告里的根（所选的那一条，或退回见证到的那一版时退到的那一条）。
fn outcome_root(outcome: &RecoveryOutcome) -> Option<RootKey> {
    let key_of =
        |root: &(InstanceGeneration, singlefs_core::address::CheckpointTxg)| (root.0 .0, root.1 .0);
    match outcome {
        RecoveryOutcome::FileRead { root, .. } | RecoveryOutcome::NoFile { root } => {
            Some(key_of(root))
        }
        RecoveryOutcome::Failed { root, .. } => root.as_ref().map(key_of),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootPick {
    /// 择到的是被藏根，它引用的某个单元在用到的盘上已被盖。
    HiddenRootCovered,
    /// 择到的是被藏根，单元都还是写下时的字节。
    HiddenRootIntact,
    /// 择到的不是被藏根，它引用的某个单元被盖了。
    OtherRootCovered,
    /// 合法单盘恢复：择到的根不是被抛弃根，引用的单元逐字节等于写下时的字节。
    Legal,
}

impl RootPick {
    fn name(self) -> &'static str {
        match self {
            RootPick::HiddenRootCovered => "hidden_root_covered",
            RootPick::HiddenRootIntact => "hidden_root_intact",
            RootPick::OtherRootCovered => "other_root_covered",
            RootPick::Legal => "legal",
        }
    }
}

fn classify_root_pick(key: RootKey, hidden_root: Option<RootKey>, covered: bool) -> RootPick {
    match (Some(key) == hidden_root, covered) {
        (true, true) => RootPick::HiddenRootCovered,
        (true, false) => RootPick::HiddenRootIntact,
        (false, true) => RootPick::OtherRootCovered,
        (false, false) => RootPick::Legal,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ArmResult {
    key: Option<RootKey>,
    outcome: &'static str,
    /// 这条臂连着避开了几条根（丙恒 0）。
    avoided_roots: usize,
    /// 要避开却找不到那条根的根槽位置，没能避开。
    could_not_mask: bool,
}

fn root_record_of(reader: &dyn PoolReader, key: RootKey) -> Option<RootRecord> {
    let configuration = choose_system_configuration(reader).ok()?;
    readable_roots(
        reader,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| root_key_of(root) == key)
}

/// 一条根的根槽在哪块盘的哪个偏移（装置自己的独立解码：checker 的 `valid_roots` 与 `root_slot_positions`），按字节区间 (偏移, 长度)。
fn root_slot_position(
    view: &KeptDiskView<'_, '_>,
    geometry: &PoolGeometry,
    key: RootKey,
) -> Option<(u64, u64)> {
    let positions = checker_image::root_slot_positions(geometry);
    checker_image::valid_roots(view, geometry)
        .into_iter()
        .find(|(_, _, root_view)| (root_view.instance, root_view.checkpoint_txg) == key)
        .and_then(|(region, slot, _)| {
            positions
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    *candidate_region == region && *candidate_slot == slot
                })
                .map(|(_, _, _, offset)| (*offset, u64::from(geometry.physical_block_size)))
        })
}

/// 一条臂在一块剩下的盘上的结局。`first_report` 是不避开任何根时 `recover` 的报告（丙就是它）；甲一、甲二避开一条根就把它的根槽读成全零、
/// 再跑一遍 `recover`（于是后面的施加记录、退回见证到的那一版都照今天的做），直到择到的根不再被避开、或择不到根。
fn arm_result_on_the_kept_disk(
    image: &CrashImage<'_>,
    device: DeviceIdentity,
    geometry: &PoolGeometry,
    arm: Arm,
    first_report: &RecoveryReport,
) -> ArmResult {
    let mut masked_root_slots: Vec<(u64, u64)> = Vec::new();
    let mut avoided_roots = 0usize;
    let mut could_not_mask = false;
    let mut key = outcome_root(&first_report.outcome);
    let mut outcome = outcome_class(&first_report.outcome);
    while let Some(picked) = key {
        let view = KeptDiskView {
            image,
            device,
            masked_root_slots: masked_root_slots.clone(),
        };
        let avoid = match arm {
            Arm::Today => false,
            Arm::NewerInstanceRecord => match root_record_of(&view, picked) {
                Some(root) => avoided_by_newer_instance_record(&view, &root),
                None => false,
            },
            Arm::ConfigurationInstance => newest_configuration_instance_on_the_kept_disk(
                &view, device,
            )
            .is_some_and(|configuration_instance| {
                avoided_by_configuration_instance(configuration_instance, picked.0)
            }),
        };
        if !avoid || avoided_roots >= AVOIDED_ROOTS_AT_MOST {
            break;
        }
        let Some(position) = root_slot_position(&view, geometry, picked) else {
            could_not_mask = true;
            break;
        };
        masked_root_slots.push(position);
        avoided_roots += 1;
        let next_report = recover(
            &KeptDiskView {
                image,
                device,
                masked_root_slots: masked_root_slots.clone(),
            },
            JournalPolicy::Consult,
        );
        key = outcome_root(&next_report.outcome);
        outcome = outcome_class(&next_report.outcome);
    }
    ArmResult {
        key,
        outcome,
        avoided_roots,
        could_not_mask,
    }
}

// ============================================================================
// 十一、一个崩溃状态上的全部读数：两块盘都在时的恢复、每块盘单独留下的三条臂、B4 的分类。
// ============================================================================

/// 核心层 `choose_root` 在这块盘上择到的根（不看 journal、不退回见证到的那一版）。
fn crates_choice(reader: &dyn PoolReader) -> Option<RootKey> {
    let configuration = choose_system_configuration(reader).ok()?;
    choose_root(reader, &configuration).map(|root| root_key_of(&root))
}

/// 装置自己读根环、判自证的那一份（checker 的独立解码）：自证根里 (txg, 实例代号) 最大的。
fn independent_choice(view: &KeptDiskView<'_, '_>, geometry: &PoolGeometry) -> Option<RootKey> {
    checker_image::valid_roots(view, geometry)
        .into_iter()
        .map(|(_, _, root_view)| (root_view.instance, root_view.checkpoint_txg))
        .max_by_key(|(instance, txg)| (*txg, *instance))
}

/// 实例代号之差 `配置 − 所选根` 写进计数名：负数前面写 minus。
fn signed_difference_label(configuration_instance: u32, root_instance: u32) -> String {
    if configuration_instance >= root_instance {
        format!("{}", configuration_instance - root_instance)
    } else {
        format!("minus_{}", root_instance - configuration_instance)
    }
}

fn pick_of(
    history: &RecordedHistory,
    image: &CrashImage<'_>,
    key: Option<RootKey>,
    only_on_device: Option<u32>,
) -> Result<Option<RootPick>, String> {
    let Some(key) = key else {
        return Ok(None);
    };
    let log = history
        .roots
        .get(&key)
        .ok_or_else(|| format!("停机：恢复择到的根 {key:?} 不在全知日志里"))?;
    let covered = some_unit_of_the_root_is_covered(image, log, only_on_device);
    Ok(Some(classify_root_pick(key, history.hidden_root, covered)))
}

/// B4 的类（登记 5.2）：类 a = 被藏根已确认且它引用的单元两块盘上都没被盖；类 b = 被藏根引用的单元至少一个被盖了；其余记类 c。
fn class_of_the_hidden_root(
    confirmation: Confirmation,
    some_unit_is_covered: bool,
) -> &'static str {
    match (some_unit_is_covered, confirmation) {
        (true, _) => "class_some_unit_covered",
        (false, Confirmation::Confirmed) => "class_confirmed_none_covered",
        (false, Confirmation::Unconfirmed) => "class_unconfirmed_none_covered",
    }
}

/// 一个状态里被藏根属于 B4 的哪一类：两块盘上它引用的单元都看（不只看某一块盘）。
fn class_of_the_hidden_root_in_the_state(
    image: &CrashImage<'_>,
    hidden_log: &RootLog,
    confirmation: Confirmation,
) -> &'static str {
    let some_unit_is_covered = some_unit_of_the_root_is_covered(image, hidden_log, None);
    class_of_the_hidden_root(confirmation, some_unit_is_covered)
}

/// 根环里区域 / 槽的落点公式（登记第五节「根环区域 = txg mod 3」）：第 txg 次发布写区域 txg mod 3 的槽 (txg div 3) mod S。
fn ring_slot_of_txg(txg: u64) -> (u64, u64) {
    (
        txg % ROOT_RING_REGIONS_LOCAL,
        (txg / ROOT_RING_REGIONS_LOCAL) % ROOT_RING_SLOTS_PER_REGION_LOCAL,
    )
}

fn count_the_pick(counters: &mut Counters, prefix: &str, pick: Option<RootPick>) {
    match pick {
        None => counters.add(&format!("{prefix}pick.none"), 1),
        Some(root_pick) => counters.add(&format!("{prefix}pick.{}", root_pick.name()), 1),
    }
}

fn count_the_arm_against_today(
    counters: &mut Counters,
    prefix: &str,
    today_pick: Option<RootPick>,
    arm_pick: Option<RootPick>,
    arm: &ArmResult,
) {
    let avoided = arm.avoided_roots > 0;
    if avoided {
        counters.add(&format!("{prefix}avoided"), 1);
    }
    if arm.could_not_mask {
        counters.add(&format!("{prefix}could_not_mask"), 1);
    }
    let after = match arm_pick {
        None => "none",
        Some(RootPick::Legal) => "legal",
        Some(RootPick::HiddenRootIntact) => "hidden_root_intact",
        Some(RootPick::HiddenRootCovered) => "hidden_root_covered",
        Some(RootPick::OtherRootCovered) => "other_root_covered",
    };
    match today_pick {
        Some(RootPick::Legal) if avoided => {
            counters.add(&format!("{prefix}avoided_a_legal_root"), 1);
            counters.add(&format!("{prefix}avoided_a_legal_root.then_{after}"), 1);
        }
        Some(RootPick::HiddenRootIntact) if avoided => {
            counters.add(&format!("{prefix}avoided_an_intact_hidden_root"), 1);
            counters.add(
                &format!("{prefix}avoided_an_intact_hidden_root.then_{after}"),
                1,
            );
        }
        Some(RootPick::HiddenRootCovered) => {
            counters.add(&format!("{prefix}on_a_covered_hidden_root"), 1);
            counters.add(&format!("{prefix}on_a_covered_hidden_root.then_{after}"), 1);
        }
        Some(RootPick::OtherRootCovered) if avoided => {
            counters.add(&format!("{prefix}avoided_another_covered_root"), 1);
            counters.add(
                &format!("{prefix}avoided_another_covered_root.then_{after}"),
                1,
            );
        }
        Some(RootPick::Legal | RootPick::HiddenRootIntact | RootPick::OtherRootCovered) | None => {}
    }
}

fn evaluate_one_state(
    history: &RecordedHistory,
    persisted: &[bool],
    counters: &mut Counters,
) -> Result<(), String> {
    let image = CrashImage {
        base: &history.base,
        writes: &history.writes,
        persisted: persisted.to_vec(),
    };
    counters.add("states", 1);
    // 两块盘都在时的恢复（对照：C554 行写的「两块盘都在时 0 失败」）。
    let both_report = recover(&image, JournalPolicy::Consult);
    counters.add(
        &format!("both.outcome.{}", outcome_class(&both_report.outcome)),
        1,
    );
    let both_pick = pick_of(history, &image, outcome_root(&both_report.outcome), None)?;
    count_the_pick(counters, "both.", both_pick);
    for device_number in [0u32, 1u32] {
        let device = DeviceIdentity(device_number);
        let kept = kept_disk(&image, device);
        let by_crates = crates_choice(&kept);
        let by_the_device = independent_choice(&kept, &history.pool_geometry);
        if by_crates != by_the_device {
            return Err(format!(
                "停机：只留盘 {device_number} 时核心层择根 {by_crates:?} 与装置独立解码的 {by_the_device:?} 不同"
            ));
        }
        let today_report = recover(&kept, JournalPolicy::Consult);
        let today = arm_result_on_the_kept_disk(
            &image,
            device,
            &history.pool_geometry,
            Arm::Today,
            &today_report,
        );
        let today_pick = pick_of(history, &image, today.key, Some(device_number))?;
        let configuration_instance_on_the_disk =
            newest_configuration_instance_on_the_kept_disk(&kept, device);
        for arm in Arm::EVERY_ARM {
            let result = match arm {
                Arm::Today => today.clone(),
                Arm::NewerInstanceRecord | Arm::ConfigurationInstance => {
                    arm_result_on_the_kept_disk(
                        &image,
                        device,
                        &history.pool_geometry,
                        arm,
                        &today_report,
                    )
                }
            };
            let arm_pick = pick_of(history, &image, result.key, Some(device_number))?;
            let prefix = format!("{}.kept{device_number}.", arm.name());
            counters.add(&format!("{prefix}states"), 1);
            counters.add(&format!("{prefix}outcome.{}", result.outcome), 1);
            count_the_pick(counters, &prefix, arm_pick);
            if arm != Arm::Today {
                count_the_arm_against_today(counters, &prefix, today_pick, arm_pick, &result);
            }
        }
        if let (Some(configuration_instance), Some((root_instance, _))) =
            (configuration_instance_on_the_disk, today.key)
        {
            counters.add(
                &format!(
                    "configuration_instance.kept{device_number}.difference.{}",
                    signed_difference_label(configuration_instance, root_instance)
                ),
                1,
            );
        }
    }
    count_the_hidden_root_classification(history, &image, persisted, counters);
    Ok(())
}

/// B4：实例 2 写行那次发布持久之后的状态，按被藏根的单元有没有被盖分类；另记今天第一次发布的 txg 落在被藏根那一槽（txg 撞上）的状态。
fn count_the_hidden_root_classification(
    history: &RecordedHistory,
    image: &CrashImage<'_>,
    persisted: &[bool],
    counters: &mut Counters,
) {
    let (Some(hidden_root), Some(row_write_index)) =
        (history.hidden_root, history.row_publish_root_write)
    else {
        return;
    };
    if !persisted[row_write_index] {
        return;
    }
    let Some(hidden_log) = history.roots.get(&hidden_root) else {
        return;
    };
    let class = class_of_the_hidden_root_in_the_state(
        image,
        hidden_log,
        history.spec.family.confirmation(),
    );
    counters.add("after_the_row_publish.states", 1);
    counters.add(&format!("after_the_row_publish.{class}"), 1);
    let row_root = history.mount.row_publish_root;
    let collides = row_root
        .is_some_and(|(_, row_txg)| ring_slot_of_txg(row_txg) == ring_slot_of_txg(hidden_root.1));
    if collides {
        counters.add(
            "after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot",
            1,
        );
        counters.add(
            &format!(
                "after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.{class}"
            ),
            1,
        );
    }
    let hidden_root_still_in_the_ring = checker_image::valid_roots(image, &history.pool_geometry)
        .into_iter()
        .any(|(_, _, root_view)| (root_view.instance, root_view.checkpoint_txg) == hidden_root);
    if !hidden_root_still_in_the_ring {
        counters.add("after_the_row_publish.hidden_root_slot_overwritten", 1);
        counters.add(
            &format!("after_the_row_publish.hidden_root_slot_overwritten.{class}"),
            1,
        );
    }
}

/// 判法入口之一（函数单元闭包指纹的入口，见文件头）：判一片 = 一条历史的一个段（段号等于段数时是整条流末尾那个全持久的状态）。
fn judge_the_shard(history: &RecordedHistory, shard_index: usize) -> Result<Counters, String> {
    let mut counters = Counters::default();
    for_each_state_of_segment(
        &history.writes,
        &history.segments,
        shard_index,
        &mut |persisted| evaluate_one_state(history, persisted, &mut counters).map(|_trace| ()),
    )?;
    Ok(counters)
}

// ============================================================================
// 十二、片结果库与复用键：键 = 判法入口按 `singlefs-unit-graph` 算的函数单元闭包指纹（去注释）+ 工具链行 + 历史名 + 段号 + 展开方式。
//    一片判完才原子落盘（同目录写临时文件再改名）；被打断后用同一条命令重起，键相同的片直接取库里的，只补没判的。
// ============================================================================

/// 判法入口（闭包的根）：登记第五节「装置源码」那一半由这两个函数的闭包代替，不再用文件级 sha256。
const JUDGING_ENTRY_FUNCTIONS: [&str; 2] = [
    "singlefs_checker_tier::record_the_history",
    "singlefs_checker_tier::judge_the_shard",
];
const THIS_BINARY_TARGET: &str = "bin:e182_unreadable_abandoned_root_forks";
const THREADS_ENVIRONMENT_VARIABLE: &str = "E182_THREADS";
const LIBRARY_ENVIRONMENT_VARIABLE: &str = "E182_VERDICT_LIBRARY";
const SHARD_SHARE_ENVIRONMENT_VARIABLE: &str = "E182_SHARD_SHARE";
const REPOSITORY_ROOT_ENVIRONMENT_VARIABLE: &str = "E182_REPOSITORY_ROOT";
const BUILD_PROFILE_ENVIRONMENT_VARIABLE: &str = "E182_BUILD_PROFILE";

#[derive(Clone, Debug, PartialEq, Eq)]
struct ClosureFingerprint {
    key_hex: String,
    toolchain_lines: Vec<String>,
}

fn output_of(program: &str, argument: &str) -> Result<String, String> {
    let output = std::process::Command::new(program)
        .arg(argument)
        .output()
        .map_err(|error| format!("跑不了 {program} {argument}：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {argument} 退 {:?}",
            output.status.code()
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("{program} {argument} 的输出不是 UTF-8"))
}

fn repository_root() -> std::path::PathBuf {
    match std::env::var(REPOSITORY_ROOT_ENVIRONMENT_VARIABLE) {
        Ok(root) if !root.is_empty() => std::path::PathBuf::from(root),
        Ok(_) | Err(_) => std::env::current_dir().expect("当前目录读得出（从仓根起装置）"),
    }
}

fn worker_threads() -> usize {
    own_variable_or_default_heavy_threads(
        THREADS_ENVIRONMENT_VARIABLE,
        std::env::var(THREADS_ENVIRONMENT_VARIABLE).ok().as_deref(),
        std::env::var("SINGLEFS_THREAD_CAP").ok().as_deref(),
    )
    .get()
}

/// 本文件两个判法入口的函数单元闭包指纹（去注释）与这一趟的工具链行。闭包里有拒算时交 `Err`（调用方这一趟不复用、不入库）。
fn closure_fingerprint() -> Result<ClosureFingerprint, String> {
    use singlefs_unit_graph::build_toolchain::{
        effective_profile_lines, toolchain_lines_of_the_build, BuildToolchainInputs,
    };
    use singlefs_unit_graph::entry_groups::{
        entry_group_key, EntryGroupRequest, EntryGroupVerdict,
    };
    use singlefs_unit_graph::{load_workspace, parse_lock_file, RuleSwitches, TargetSelector};
    let root = repository_root();
    let switches = RuleSwitches::every_rule_intact();
    let threads = std::num::NonZeroUsize::new(worker_threads()).expect("线程数是正数");
    let workspace = load_workspace(&root, threads, &switches)
        .map_err(|error| format!("装不起工作区：{error}"))?;
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let rustc_version_text = output_of(&rustc, "-vV")?;
    let cargo_version_text = output_of(&cargo, "-V")?;
    let target_triple = rustc_version_text
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::trim)
        .ok_or_else(|| "rustc -vV 里没有 host: 那一行".to_string())?
        .to_string();
    let profile_name =
        std::env::var(BUILD_PROFILE_ENVIRONMENT_VARIABLE).unwrap_or_else(|_| "release".to_string());
    let root_manifest_text = std::fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|error| format!("读不了根 Cargo.toml：{error}"))?;
    let profile_lines = effective_profile_lines(&root_manifest_text, &profile_name, &|name| {
        std::env::var(name).ok()
    })?;
    let toolchain_lines = toolchain_lines_of_the_build(&BuildToolchainInputs {
        rustc_version_text: &rustc_version_text,
        cargo_version_text: &cargo_version_text,
        target_triple: &target_triple,
        encoded_rustflags: &std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default(),
        profile_lines: &profile_lines,
    });
    let lock_text = std::fs::read_to_string(root.join("Cargo.lock"))
        .map_err(|error| format!("读不了 Cargo.lock：{error}"))?;
    let locked_packages =
        parse_lock_file(&lock_text).map_err(|error| format!("Cargo.lock 读不了：{error}"))?;
    let request = EntryGroupRequest {
        group_name: "e182_single_device_judgement".to_string(),
        target: TargetSelector::parse(THIS_BINARY_TARGET)
            .map_err(|text| format!("编译目标写法认不得：{text}"))?,
        entry_paths: JUDGING_ENTRY_FUNCTIONS
            .iter()
            .map(|entry| (*entry).to_string())
            .collect(),
        features: vec!["verdict-store".to_string()],
    };
    let group = entry_group_key(
        &workspace,
        &request,
        &toolchain_lines,
        &locked_packages,
        &switches,
    )
    .map_err(|error| format!("入口写错了：{error}"))?;
    match group.verdict {
        EntryGroupVerdict::Keyed { key } => Ok(ClosureFingerprint {
            key_hex: hexadecimal_text(&key.0),
            toolchain_lines,
        }),
        EntryGroupVerdict::Refused { refusal_lines } => {
            Err(format!("闭包里有拒算：{}", refusal_lines.join(" | ")))
        }
        EntryGroupVerdict::NotCompiled { reason } => Err(format!("入口不编进来：{reason}")),
    }
}

fn library_directory() -> Result<std::path::PathBuf, String> {
    let directory = match std::env::var(LIBRARY_ENVIRONMENT_VARIABLE) {
        Ok(directory) if !directory.is_empty() => std::path::PathBuf::from(directory),
        Ok(_) | Err(_) => {
            let output = std::process::Command::new("git")
                .args(["rev-parse", "--git-common-dir"])
                .current_dir(repository_root())
                .output()
                .map_err(|error| format!("跑不了 git rev-parse：{error}"))?;
            if !output.status.success() {
                return Err("git rev-parse --git-common-dir 失败：不在 git 工作树里，设 E182_VERDICT_LIBRARY".to_string());
            }
            let common =
                String::from_utf8(output.stdout).map_err(|_| "git 的输出不是 UTF-8".to_string())?;
            repository_root()
                .join(common.trim())
                .join("singlefs-unit-verdicts")
                .join("e182-single-device")
        }
    };
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("建不了片结果库目录 {}：{error}", directory.display()))?;
    Ok(directory)
}

fn shard_key_text(
    fingerprint: &ClosureFingerprint,
    history_name: &str,
    shard_index: usize,
    expansion: Expansion,
) -> String {
    let mut text = String::new();
    text.push_str("e182-single-device-shard-v1\n");
    text.push_str(&format!("closure={}\n", fingerprint.key_hex));
    for line in &fingerprint.toolchain_lines {
        text.push_str(&format!("toolchain={line}\n"));
    }
    text.push_str(&format!(
        "history={history_name}\nsegment={shard_index}\nexpansion={}\n",
        expansion.name()
    ));
    text
}

fn shard_file_name(key_text: &str) -> String {
    format!(
        "{}.shard",
        hexadecimal_text(&sha256_digest(key_text.as_bytes()))
    )
}

fn commented_key(key_text: &str) -> String {
    key_text.lines().map(|line| format!("# {line}\n")).collect()
}

fn read_stored_shard(directory: &std::path::Path, key_text: &str) -> Option<Counters> {
    let text = std::fs::read_to_string(directory.join(shard_file_name(key_text))).ok()?;
    let body = text.strip_prefix(&commented_key(key_text))?;
    Counters::from_text(body).ok()
}

/// 一片判完才原子落盘：同目录写临时文件再改名换上。
fn store_shard(
    directory: &std::path::Path,
    key_text: &str,
    counters: &Counters,
) -> Result<(), String> {
    let name = shard_file_name(key_text);
    let temporary = directory.join(format!(".{name}.{}.temporary", std::process::id()));
    let mut text = commented_key(key_text);
    text.push_str(&counters.to_text());
    std::fs::write(&temporary, text)
        .map_err(|error| format!("写不了 {}：{error}", temporary.display()))?;
    std::fs::rename(&temporary, directory.join(&name))
        .map_err(|error| format!("改名不成 {name}：{error}"))
}

struct ShardJob<'history> {
    history: &'history RecordedHistory,
    history_name: String,
    shard_index: usize,
    expansion: Expansion,
    /// 全部历史按次序、每条历史的片按段号排下来的片号（多机分片按它对份数取余）。
    global_number: usize,
}

fn expansion_of_shard(history: &RecordedHistory, shard_index: usize) -> Expansion {
    match history.segments.get(shard_index) {
        Some(segment) => expansion_of_segment(segment.len()),
        None => Expansion::FinalState,
    }
}

fn plan_the_shards(histories: &[RecordedHistory]) -> Vec<ShardJob<'_>> {
    let mut jobs = Vec::new();
    for history in histories {
        for shard_index in 0..=history.segments.len() {
            jobs.push(ShardJob {
                history,
                history_name: history_name(&history.spec),
                shard_index,
                expansion: expansion_of_shard(history, shard_index),
                global_number: jobs.len(),
            });
        }
    }
    jobs
}

/// `E182_SHARD_SHARE=<第几份>/<共几份>`；没设是 `None`。
fn shard_share_from_the_environment() -> Result<Option<(usize, usize)>, String> {
    let Ok(text) = std::env::var(SHARD_SHARE_ENVIRONMENT_VARIABLE) else {
        return Ok(None);
    };
    let (which, of) = text.split_once('/').ok_or_else(|| {
        format!("{SHARD_SHARE_ENVIRONMENT_VARIABLE} 要写「第几份/共几份」，给的是 {text}")
    })?;
    let which = which
        .parse::<usize>()
        .map_err(|error| format!("{text}：{error}"))?;
    let of = of
        .parse::<usize>()
        .map_err(|error| format!("{text}：{error}"))?;
    if of == 0 || which >= of {
        return Err(format!(
            "{SHARD_SHARE_ENVIRONMENT_VARIABLE}={text}：第几份要小于共几份、共几份要大于 0"
        ));
    }
    Ok(Some((which, of)))
}

struct ShardResult {
    counters: Counters,
    reused: bool,
}

struct RunOptions<'options> {
    library: Option<&'options std::path::Path>,
    fingerprint: Option<&'options ClosureFingerprint>,
    /// `false` 是 `--full`：不读库、全部现判，现判出的照键写回。
    read_library: bool,
    share: Option<(usize, usize)>,
}

struct RunReport {
    /// 与 `jobs` 同序；不归这一份的片（多机分片）与停机之后没跑的片是 `None`。
    results: Vec<Option<ShardResult>>,
    stop: Option<String>,
}

fn key_text_of_the_job(job: &ShardJob<'_>, options: &RunOptions<'_>) -> Option<String> {
    options.fingerprint.map(|fingerprint| {
        shard_key_text(
            fingerprint,
            &job.history_name,
            job.shard_index,
            job.expansion,
        )
    })
}

fn process_one_shard(job: &ShardJob<'_>, options: &RunOptions<'_>) -> Result<ShardResult, String> {
    let key_text = key_text_of_the_job(job, options);
    if options.read_library {
        if let (Some(library), Some(key_text)) = (options.library, &key_text) {
            if let Some(counters) = read_stored_shard(library, key_text) {
                return Ok(ShardResult {
                    counters,
                    reused: true,
                });
            }
        }
    }
    let started = std::time::Instant::now();
    let counters = judge_the_shard(job.history, job.shard_index)?;
    if let (Some(library), Some(key_text)) = (options.library, &key_text) {
        store_shard(library, key_text, &counters)?;
    }
    eprintln!(
        "progress: shard {} {} segment={} expansion={} states={} seconds={:.1}",
        job.global_number,
        job.history_name,
        job.shard_index,
        job.expansion.name(),
        counters.get("states"),
        started.elapsed().as_secs_f64()
    );
    Ok(ShardResult {
        counters,
        reused: false,
    })
}

fn states_the_shard_will_judge(job: &ShardJob<'_>) -> u64 {
    job.history
        .segments
        .get(job.shard_index)
        .map_or(1, |segment| {
            state_count_of_segment(&job.history.writes, segment).unwrap_or(0)
        })
}

/// 把 `jobs` 里归这一份的片分给工作线程判（或从库里取）。出现停机条件（该停下交回主 agent 的）就让各线程做完手头那一片后收手。
fn run_the_shards(jobs: &[ShardJob<'_>], options: &RunOptions<'_>) -> RunReport {
    use std::sync::atomic::AtomicBool;
    use std::sync::Mutex;
    let next_job = std::sync::atomic::AtomicUsize::new(0);
    let stop_requested = AtomicBool::new(false);
    let stop_reason: Mutex<Option<String>> = Mutex::new(None);
    let slots: Mutex<Vec<Option<ShardResult>>> =
        Mutex::new((0..jobs.len()).map(|_| None).collect());
    // 状态多的片先派：最大的几片各占一个线程，尾巴上不剩一片慢的拖着整趟（只改派的次序，结果按片号存）。
    let mut dispatch_order: Vec<usize> = (0..jobs.len()).collect();
    dispatch_order
        .sort_by_key(|index| std::cmp::Reverse(states_the_shard_will_judge(&jobs[*index])));
    std::thread::scope(|scope| {
        for _ in 0..worker_threads() {
            scope.spawn(|| loop {
                if stop_requested.load(Ordering::SeqCst) {
                    break;
                }
                let position = next_job.fetch_add(1, Ordering::SeqCst);
                let Some(index) = dispatch_order.get(position).copied() else {
                    break;
                };
                let job = &jobs[index];
                if let Some((which, of)) = options.share {
                    if job.global_number % of != which {
                        continue;
                    }
                }
                match process_one_shard(job, options) {
                    Ok(result) => {
                        slots.lock().expect("片结果表的锁没被毒化")[index] = Some(result);
                    }
                    Err(reason) => {
                        *stop_reason.lock().expect("停机原因的锁没被毒化") = Some(reason);
                        stop_requested.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            });
        }
    });
    let results = slots.into_inner().expect("片结果表的锁没被毒化");
    RunReport {
        results,
        stop: stop_reason.into_inner().expect("停机原因的锁没被毒化"),
    }
}

/// 哨兵（`.claude/rules/test-script-discipline.md`「判定可复用」）：从取自库里的片里挑至多 `SENTINELS_AT_MOST` 片（状态最少的，同数取片号小的）
/// 重新现判，与库里的逐名相同；不同就是键没罩住输入，停机。
const SENTINELS_AT_MOST: usize = 3;

fn recheck_the_sentinels(jobs: &[ShardJob<'_>], report: &RunReport) -> Result<usize, String> {
    let mut reused: Vec<(u64, usize)> = report
        .results
        .iter()
        .enumerate()
        .filter_map(|(index, result)| {
            result
                .as_ref()
                .filter(|result| result.reused && result.counters.get("states") > 0)
                .map(|result| (result.counters.get("states"), index))
        })
        .collect();
    reused.sort_unstable();
    let mut checked = 0;
    for (_, index) in reused.into_iter().take(SENTINELS_AT_MOST) {
        let job = &jobs[index];
        let rejudged = judge_the_shard(job.history, job.shard_index)?;
        let stored = &report.results[index]
            .as_ref()
            .expect("哨兵是取自库里的片，结果在")
            .counters;
        if &rejudged != stored {
            return Err(format!(
                "停机：哨兵 {} 段 {} 现判的计数与库里的不同（复用键没罩住输入）",
                job.history_name, job.shard_index
            ));
        }
        checked += 1;
    }
    Ok(checked)
}

// ============================================================================
// 十三、常量回比、第七节锚点、闭式计数、源文件指纹。出自被测条款的锚点（A1–A4）不符走失败条款 Z7（条款可能错）；
//    独立算出的（I1、I2）与常量回比不符整段作废（登记第十一节）。
// ============================================================================

struct ConstantCheck {
    constant: &'static str,
    local_value: u64,
    crates_value: u64,
    crates_source: &'static str,
}

fn local_constant_checks() -> Vec<ConstantCheck> {
    vec![
        ConstantCheck {
            constant: "slot_bytes",
            local_value: SLOT_BYTES_LOCAL,
            crates_value: SLOT_BYTES,
            crates_source: "singlefs_format::SLOT_BYTES",
        },
        ConstantCheck {
            constant: "unit_area_start_slot",
            local_value: UNIT_AREA_START_SLOT_LOCAL,
            crates_value: UNIT_AREA_START_SLOT,
            crates_source: "singlefs_format::UNIT_AREA_START_SLOT",
        },
        ConstantCheck {
            constant: "journal_ring_bytes",
            local_value: JOURNAL_RING_BYTES_LOCAL,
            crates_value: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            crates_source: "singlefs_format::JOURNAL_RING_DEFAULT_BYTES",
        },
        ConstantCheck {
            constant: "root_ring_slots_per_region",
            local_value: ROOT_RING_SLOTS_PER_REGION_LOCAL,
            crates_value: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
            crates_source: "RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM",
        },
        ConstantCheck {
            constant: "root_ring_regions",
            local_value: ROOT_RING_REGIONS_LOCAL,
            crates_value: singlefs_format::ROOT_RING_REGIONS,
            crates_source: "singlefs_format::ROOT_RING_REGIONS",
        },
        ConstantCheck {
            constant: "warm_up_publishes_at_most",
            local_value: u64::try_from(WARM_UP_PUBLISHES_AT_MOST_LOCAL).expect("3 装得进 u64"),
            crates_value:
                singlefs_core::admission::WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH,
            crates_source:
                "singlefs_core::admission::WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH",
        },
        ConstantCheck {
            constant: "data_unit_payload_bytes",
            local_value: u64::try_from(DATA_UNIT_PAYLOAD_BYTES_LOCAL).expect("32634 装得进 u64"),
            crates_value: u64::try_from(data_unit_payload_capacity()).expect("装得进 u64"),
            crates_source: "singlefs_core::unit::data_unit_payload_capacity()",
        },
        ConstantCheck {
            constant: "system_configuration_slot_bytes",
            local_value: u64::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096"),
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES,
            crates_source: "singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES",
        },
        ConstantCheck {
            constant: "fixed_structure_slot_spacing_minimum",
            local_value: u64::from(FIXED_STRUCTURE_SLOT_SPACING_LOCAL),
            crates_value: singlefs_format::FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
            crates_source: "singlefs_format::FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES",
        },
    ]
}

/// I1：每块盘的单元区槽数 = 盘字节数 ÷ 槽字节数 − 单元区起点槽号（登记第七节，与 E180 第七节 B1 同一个算式；两个字面数是用命令算过的）。
fn unit_area_slots_of(geometry: DiskGeometry) -> u64 {
    geometry.device_bytes() / SLOT_BYTES_LOCAL - UNIT_AREA_START_SLOT_LOCAL
}

const UNIT_AREA_SLOTS_DEFAULT_PINNED: u64 = 211_968;
const UNIT_AREA_SLOTS_NARROW_PINNED: u64 = 146_465;

/// 一个只有 `kind` 种类的合成写（闭式计数的取样点只看写的种类，不看字节）。
fn synthetic_write(kind: StepKind) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(0),
        kind,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(0),
        contents: singlefs_harness::memory_pool::WrittenContents::Bytes(vec![0; 512]),
    }
}

/// I2：一段里 `in_place` 次原地写、`unit` 次单元写，按装置的枚举器数出几个状态，与独立的闭式比。
/// 全量展开的段（写数不超过上界）闭式是 2^m − 1（m = 写数，全持久那个留给下一段）；快档是 2^k × (单元写非空 ? 2 : 1) − 1。
fn enumerated_and_closed_form_state_counts(in_place: usize, unit: usize) -> (u64, u64) {
    let mut writes: Vec<RetainedWrite> = Vec::new();
    for _ in 0..in_place {
        writes.push(synthetic_write(StepKind::JournalRecord));
    }
    for _ in 0..unit {
        writes.push(synthetic_write(StepKind::UnitWrite));
    }
    let segments = vec![(0..writes.len()).collect::<Vec<usize>>()];
    let mut seen: u64 = 0;
    let enumeration = for_each_state_of_segment(&writes, &segments, 0, &mut |_persisted| {
        seen += 1;
        Ok(())
    });
    if enumeration.is_err() {
        // 枚举器自己核出个数与它的闭式不同：用一个必然对不上的数把这一格报出来。
        seen = u64::MAX;
    }
    let total = in_place + unit;
    let closed_form = if total <= FULL_EXPANSION_WRITES_AT_MOST_LOCAL {
        (1u64 << total) - 1
    } else {
        (1u64 << in_place) * if unit == 0 { 1 } else { 2 } - 1
    };
    (seen, closed_form)
}

/// 一条锚点的读数：成立 / 不成立 / 这条历史上不适用（挂载被拒、没有可比的对象）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct AnchorReading {
    anchor: String,
    subject: String,
    applicable: bool,
    holds: bool,
    detail: String,
}

/// A1：实例 2 每一次根槽写落在 txg mod 3 那个区域、按区域归属 0 / 1 / 0 归盘，偏移是 checker 独立解码出的那个槽位。
fn anchor_root_slot_writes_land_by_the_ring_rule(history: &RecordedHistory) -> AnchorReading {
    let positions = checker_image::root_slot_positions(&history.pool_geometry);
    let mut checked = 0usize;
    let mut violations: Vec<String> = Vec::new();
    for write in history
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
    {
        let Some(root) = write
            .bytes()
            .and_then(|bytes| RootRecord::parse_slot(bytes, &FILESYSTEM_IDENTIFIER))
        else {
            violations.push("根槽写解不出根记录".to_string());
            continue;
        };
        checked += 1;
        let (region, slot) = ring_slot_of_txg(root.checkpoint_txg.0);
        let expected_device = REGION_DEVICES_LOCAL[usize::try_from(region).expect("区域号")];
        let position = positions
            .iter()
            .find(|(candidate_region, candidate_slot, _, _)| {
                *candidate_region == region && *candidate_slot == slot
            });
        let landed_right = position.is_some_and(|(_, _, device, offset)| {
            *device == expected_device
                && write.device.0 == expected_device
                && write.offset.0 == *offset
        });
        if !landed_right {
            violations.push(format!(
                "txg {} 的根槽写落在盘 {} 偏移 {}",
                root.checkpoint_txg.0, write.device.0, write.offset.0
            ));
        }
    }
    AnchorReading {
        anchor: "root_slot_writes_land_by_the_ring_rule".to_string(),
        subject: history_name(&history.spec),
        applicable: checked > 0,
        holds: violations.is_empty(),
        detail: format!(
            "root_slot_writes_checked={checked} violations={}",
            violations.join(";")
        ),
    }
}

/// A2：每次可写挂载的暖机零单元发布至多 3 次。
fn anchor_warm_up_publishes_are_at_most_three(history: &RecordedHistory) -> AnchorReading {
    AnchorReading {
        anchor: "warm_up_publishes_are_at_most_three".to_string(),
        subject: history_name(&history.spec),
        applicable: history.mount.refused.is_none(),
        holds: history.mount.warm_up_publishes <= WARM_UP_PUBLISHES_AT_MOST_LOCAL,
        detail: format!("warm_up_publishes={}", history.mount.warm_up_publishes),
    }
}

/// A3 的期望：实例 2 第一次发布的 txg = max(装置日志里根环中读得出的根的 txg, 读得出的记录的 txg) + 1，读回全零的不算。
fn first_publish_txg_expected_by_the_clause(history: &RecordedHistory) -> Option<u64> {
    let hidden = history.hidden_root;
    let records_are_hidden_too = matches!(history.spec.family, Family::HiddenRoot(_));
    let newest_root_txg = checker_image::valid_roots(&history.base, &history.pool_geometry)
        .into_iter()
        .map(|(_, _, root_view)| (root_view.instance, root_view.checkpoint_txg))
        .filter(|key| Some(*key) != hidden)
        .map(|(_, txg)| txg)
        .max()?;
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let mut newest_record_txg = 0u64;
    for device in [0u32, 1u32] {
        for slot in ImageReader::candidate_journal_slots(&history.base, device).unwrap_or_default()
        {
            let offset = ring_start + slot * JOURNAL_RECORD_BYTES;
            let Some(bytes) = ImageReader::read(&history.base, device, offset, record_bytes) else {
                continue;
            };
            let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) else {
                continue;
            };
            let key = (record.instance.0, record.checkpoint_txg.0);
            if records_are_hidden_too && Some(key) == hidden {
                continue;
            }
            newest_record_txg = newest_record_txg.max(record.checkpoint_txg.0);
        }
    }
    Some(newest_root_txg.max(newest_record_txg) + 1)
}

fn anchor_first_publish_txg_follows_the_newest_readable(
    history: &RecordedHistory,
) -> AnchorReading {
    let expected = first_publish_txg_expected_by_the_clause(history);
    let actual = history.mount.row_publish_root.map(|(_, txg)| txg);
    AnchorReading {
        anchor: "first_publish_txg_follows_the_newest_readable".to_string(),
        subject: history_name(&history.spec),
        applicable: history.mount.refused.is_none(),
        holds: expected.is_some() && expected == actual,
        detail: format!("expected={expected:?} actual={actual:?}"),
    }
}

/// A4：已确认档里只把 R_h 的根槽与记录读回全零、不罩较新的系统配置槽时，实例 2 的可写挂载在取号之前被拒、两块盘逐字节不变。
fn anchor_confirmed_without_the_newer_configuration_is_refused_before_any_write(
) -> Result<AnchorReading, String> {
    let spec = HistorySpec {
        family: Family::HiddenRoot(Confirmation::Confirmed),
        geometry: DiskGeometry::Default,
        first_file_units: 1,
        overwrites_before_the_hidden_publish: 5,
        overwrite_shape: OverwriteShape::OneUnit,
    };
    let first = run_the_first_instance(&spec)?;
    let (base, fault_ranges) = base_and_fault_ranges(&spec, &first)?;
    let without_the_newer_configuration: Vec<ReadsZeroRange> = fault_ranges
        .into_iter()
        .filter(|range| range.label != "newer_system_configuration_slot")
        .collect();
    let second = run_the_second_instance(&spec, &base, without_the_newer_configuration);
    let wrote_nothing = second
        .operations
        .iter()
        .all(|operation| matches!(operation.operation.kind, RecordedOperationKind::Barrier));
    Ok(AnchorReading {
        anchor: "confirmed_without_the_newer_configuration_is_refused_before_any_write".to_string(),
        subject: history_name(&spec),
        applicable: true,
        holds: second.mount.refused.is_some() && second.devices_equal_the_base && wrote_nothing,
        detail: format!(
            "refused={:?} devices_equal_the_base={} operations={}",
            second
                .mount
                .refused
                .as_deref()
                .map(|text| text.chars().take(80).collect::<String>()),
            second.devices_equal_the_base,
            second.operations.len()
        ),
    })
}

/// I1 与 I2：不建历史就能算的两类锚点。
fn static_anchors() -> Vec<AnchorReading> {
    let mut readings = Vec::new();
    for (geometry, pinned) in [
        (DiskGeometry::Default, UNIT_AREA_SLOTS_DEFAULT_PINNED),
        (DiskGeometry::Narrow, UNIT_AREA_SLOTS_NARROW_PINNED),
    ] {
        let computed = unit_area_slots_of(geometry);
        readings.push(AnchorReading {
            anchor: "unit_area_slots_pinned".to_string(),
            subject: geometry.name().to_string(),
            applicable: true,
            holds: computed == pinned,
            detail: format!("unit_area_slots={computed} pinned={pinned}"),
        });
    }
    for (in_place, unit) in [
        (0usize, 0usize),
        (3, 0),
        (8, 8),
        (16, 0),
        (0, 16),
        (6, 11),
        (17, 0),
        (0, 17),
        (5, 40),
        (9, 30),
    ] {
        let (seen, closed_form) = enumerated_and_closed_form_state_counts(in_place, unit);
        readings.push(AnchorReading {
            anchor: "enumerated_state_count_equals_the_closed_form".to_string(),
            subject: format!("in_place={in_place},unit={unit}"),
            applicable: true,
            holds: seen == closed_form,
            detail: format!("enumerated={seen} closed_form={closed_form}"),
        });
    }
    readings
}

/// 登记第三节的源文件与读时 sha256 前 16 位（开写前重核；不同且那几处行为变了走停机条款，行为变没变由主 agent 判）。
const REGISTERED_SOURCE_FILES: [(&str, &str); 7] = [
    ("crates/singlefs-core/src/mount.rs", "19b6902a9167d5c6"),
    ("crates/singlefs-core/src/recovery.rs", "3e4df6e02c97c5d2"),
    (
        "crates/singlefs-core/src/transaction.rs",
        "3312ecc2d6edb000",
    ),
    ("crates/singlefs-core/src/allocator.rs", "c376167f3edbf38b"),
    ("crates/singlefs-checker/src/walk.rs", "ee5e1663c2241de8"),
    (
        "crates/singlefs-checker-tier/src/crash.rs",
        "c1a4284c8bc8dfc4",
    ),
    (
        "crates/singlefs-harness/src/fault_injection.rs",
        "da629a5abaffd27e",
    ),
];

fn token_of(text: &str) -> String {
    text.replace([' ', '\n', '\t'], "_")
}

fn emit_anchor_reading(reading: &AnchorReading) {
    let status = match (reading.applicable, reading.holds) {
        (false, _) => "not_applicable",
        (true, true) => "holds",
        (true, false) => "violated",
    };
    emit_result(&format!(
        "name=anchor anchor={} subject={} status={status} detail={}",
        reading.anchor,
        token_of(&reading.subject),
        token_of(&reading.detail)
    ));
}

/// 源文件指纹：每份读出 sha256 前 16 位，与登记比；不同的只报，不停（行为变没变是读代码的判断）。返回不同的份数。
fn report_the_source_file_fingerprints() -> u64 {
    let root = repository_root();
    let mut differing = 0u64;
    for (path, registered) in REGISTERED_SOURCE_FILES {
        let actual = match std::fs::read(root.join(path)) {
            Ok(bytes) => hexadecimal_text(&sha256_digest(&bytes))[..16].to_string(),
            Err(error) => format!("unreadable:{}", token_of(&error.to_string())),
        };
        let same = actual == registered;
        if !same {
            differing += 1;
        }
        emit_result(&format!(
            "name=source_file path={path} registered={registered} actual={actual} same={same}"
        ));
    }
    differing
}

/// 模式 `anchors` 与单测 `constants_and_anchors_all_pass` 共用：常量回比、I1、I2、A4。交回没过的条数。
fn run_constants_and_anchors() -> u64 {
    let mut mismatches = 0u64;
    for check in local_constant_checks() {
        let matches = check.local_value == check.crates_value;
        if !matches {
            mismatches += 1;
        }
        emit_result(&format!(
            "name=constant_check constant={} local={} crates={} crates_source={} match={matches}",
            check.constant, check.local_value, check.crates_value, check.crates_source
        ));
    }
    emit_result(&format!(
        "name=constant_check constant=full_expansion_writes_at_most local={FULL_EXPANSION_WRITES_AT_MOST_LOCAL} crates=none crates_source=registration_5_decided_here match=not_comparable"
    ));
    for reading in static_anchors() {
        if !reading.holds {
            mismatches += 1;
        }
        emit_anchor_reading(&reading);
    }
    match anchor_confirmed_without_the_newer_configuration_is_refused_before_any_write() {
        Ok(reading) => {
            if !reading.holds {
                mismatches += 1;
            }
            emit_anchor_reading(&reading);
        }
        Err(reason) => {
            mismatches += 1;
            emit_result(&format!(
                "name=anchor anchor=confirmed_without_the_newer_configuration_is_refused_before_any_write subject=build status=violated detail={}",
                token_of(&reason)
            ));
        }
    }
    mismatches
}

// ============================================================================
// 十四、阳性对照（登记 5.1、5.2，每个起点都跑；一次没中，那一段作废）。
// ============================================================================

/// 取三个崩溃状态：什么都没持久、前一半的段全持久、整条流全持久。
fn three_states_for_the_controls(history: &RecordedHistory) -> Vec<Vec<bool>> {
    let mut halfway = vec![false; history.writes.len()];
    for segment in history.segments.iter().take(history.segments.len() / 2) {
        for write_index in segment {
            halfway[*write_index] = true;
        }
    }
    vec![
        vec![false; history.writes.len()],
        halfway,
        persisted_with_every_write(history.writes.len()),
    ]
}

/// 这个状态里有没有一次持久了的写，与这个单元在它那块盘上的字节区间相交（相交的单元被换了字节也会被这次写盖回去，换字节就不起作用）。
fn a_persisted_write_overlaps_the_unit(
    history: &RecordedHistory,
    persisted: &[bool],
    unit: &UnitLog,
) -> bool {
    let unit_end =
        unit.offset_in_bytes + u64::try_from(unit.bytes.len()).expect("单元长度装得进 u64");
    history
        .writes
        .iter()
        .zip(persisted)
        .any(|(write, is_persisted)| {
            *is_persisted
                && write.device.0 == unit.device
                && write.offset.0 < unit_end
                && unit.offset_in_bytes < write.offset.0 + write.length_in_bytes()
        })
}

/// 检测对照：每个状态、每块留下的盘，把今天择到的根引用的一个单元（落在这块盘上、没被这个状态里持久了的写盖住的）在这块盘上换成别的字节，
/// 「被盖」必须判出。交回 (试了几次, 判出几次, 没有可换的单元的次数)。
fn detection_control(history: &RecordedHistory) -> (u64, u64, u64) {
    let (mut trials, mut detected, mut without_a_unit) = (0u64, 0u64, 0u64);
    for persisted in three_states_for_the_controls(history) {
        let image = CrashImage {
            base: &history.base,
            writes: &history.writes,
            persisted: persisted.clone(),
        };
        for device_number in [0u32, 1u32] {
            let device = DeviceIdentity(device_number);
            let report = recover(&kept_disk(&image, device), JournalPolicy::Consult);
            let Some(log) = outcome_root(&report.outcome).and_then(|key| history.roots.get(&key))
            else {
                continue;
            };
            let untouched_unit = log.units.iter().find(|unit| {
                unit.device == device_number
                    && !unit_differs_in_the_image(&image, unit)
                    && !a_persisted_write_overlaps_the_unit(history, &persisted, unit)
            });
            let Some(unit) = untouched_unit else {
                without_a_unit += 1;
                continue;
            };
            let mut changed_base = history.base.clone();
            changed_base.flip_byte(device, DeviceOffsetInBytes(unit.offset_in_bytes), 0);
            let changed_image = CrashImage {
                base: &changed_base,
                writes: &history.writes,
                persisted: persisted.clone(),
            };
            trials += 1;
            if some_unit_of_the_root_is_covered(&changed_image, log, Some(device_number)) {
                detected += 1;
            }
        }
    }
    (trials, detected, without_a_unit)
}

fn emit_detection_control(history: &RecordedHistory) -> bool {
    let (trials, detected, without_a_unit) = detection_control(history);
    let holds = trials > 0 && trials == detected;
    emit_result(&format!(
        "name=positive_control control=detect_covered history={} trials={trials} detected={detected} no_unit_to_change={without_a_unit} status={}",
        history_name(&history.spec),
        if holds { "holds" } else { "violated" }
    ));
    holds
}

// ----------------------------------------------------------------------------
// B4 的阳性对照（登记 5.2 最后一句，修订 12.12 补进装置）：每个起点取 2 个类 a 状态，把被藏根引用的一个单元在盘 0 上那一份换掉，必须改判类 b；
// 不改判，整段作废。类 a 只出在已确认档的带藏根历史里（登记 5.2：类 a = 被藏根已确认且它引用的单元两块盘上都没被盖），所以只在这一族上跑。
// ----------------------------------------------------------------------------

/// 登记 5.2 写的「盘 0」：对照换字节的那一份在设备号 0 的盘上。
const CONFIRMED_NONE_COVERED_CONTROL_DEVICE_NUMBER: u32 = 0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ConfirmedNoneCoveredControlReading {
    /// 挑到的类 a 状态数（至多 2）。
    class_confirmed_none_covered_states_chosen: u64,
    /// 其中找得到一个可换的单元（落在盘 0 上、没被这个状态里持久了的写盖住）的状态数，即试了几次。
    trials: u64,
    /// 换之前重新判一遍仍是类 a 的次数（对照自检：不是类 a 的状态上换字节说明不了什么）。
    class_confirmed_none_covered_before_the_change: u64,
    /// 换之后判成类 b 的次数。
    changed_to_class_some_unit_covered: u64,
    /// 挑到的类 a 状态里盘 0 上没有可换的单元的状态数。
    no_unit_to_change: u64,
}

/// 对照的判定：没试成（0 次）记 not_made，不当通过；每次换之前是类 a、换之后是类 b 才算 holds；其余 violated。
fn the_confirmed_none_covered_control_status(
    trials: u64,
    class_confirmed_none_covered_before_the_change: u64,
    changed_to_class_some_unit_covered: u64,
) -> &'static str {
    if trials == 0 {
        "not_made"
    } else if class_confirmed_none_covered_before_the_change == trials
        && changed_to_class_some_unit_covered == trials
    {
        "holds"
    } else {
        "violated"
    }
}

/// 被藏根在这个状态里的类，镜像的基底单给（对照要换基底上的字节）。
fn the_hidden_root_class_with_the_base(
    history: &RecordedHistory,
    base: &MemoryPool,
    persisted: &[bool],
    hidden_log: &RootLog,
) -> &'static str {
    let image = CrashImage {
        base,
        writes: &history.writes,
        persisted: persisted.to_vec(),
    };
    class_of_the_hidden_root_in_the_state(&image, hidden_log, history.spec.family.confirmation())
}

/// 按段的次序找第一个类 a 状态（实例 2 写行那次发布持久之后、被藏根的单元两块盘上都没被盖）。
fn first_class_confirmed_none_covered_state_in_the_order(
    history: &RecordedHistory,
    hidden_log: &RootLog,
    row_publish_write_index: usize,
    shard_order: impl Iterator<Item = usize>,
) -> Result<Option<Vec<bool>>, String> {
    for shard_index in shard_order {
        let mut found: Option<Vec<bool>> = None;
        for_each_state_of_segment(
            &history.writes,
            &history.segments,
            shard_index,
            &mut |persisted| {
                if found.is_none()
                    && persisted[row_publish_write_index]
                    && the_hidden_root_class_with_the_base(
                        history,
                        &history.base,
                        persisted,
                        hidden_log,
                    ) == "class_confirmed_none_covered"
                {
                    found = Some(persisted.to_vec());
                }
                Ok(())
            },
        )?;
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

/// 对照用的类 a 状态：段的次序里最早的一个、倒着数最晚的一个（整条流末尾那个全持久的状态是类 a 时就是它），两个相同时只取一个。
fn class_confirmed_none_covered_states_for_the_control(
    history: &RecordedHistory,
) -> Result<Vec<Vec<bool>>, String> {
    let (Some(hidden_root), Some(row_publish_write_index)) =
        (history.hidden_root, history.row_publish_root_write)
    else {
        return Ok(Vec::new());
    };
    let Some(hidden_log) = history.roots.get(&hidden_root) else {
        return Ok(Vec::new());
    };
    let segment_count = history.segments.len();
    let earliest = first_class_confirmed_none_covered_state_in_the_order(
        history,
        hidden_log,
        row_publish_write_index,
        0..=segment_count,
    )?;
    let latest = first_class_confirmed_none_covered_state_in_the_order(
        history,
        hidden_log,
        row_publish_write_index,
        (0..=segment_count).rev(),
    )?;
    let mut chosen: Vec<Vec<bool>> = Vec::new();
    for state in [earliest, latest].into_iter().flatten() {
        if !chosen.contains(&state) {
            chosen.push(state);
        }
    }
    Ok(chosen)
}

/// 登记 5.2 最后一句的对照：每个类 a 状态里，把被藏根引用的一个单元在盘 0 上那一份换掉（翻一个字节），必须改判类 b。
fn class_confirmed_none_covered_to_class_some_unit_covered_control(
    history: &RecordedHistory,
) -> Result<ConfirmedNoneCoveredControlReading, String> {
    let mut reading = ConfirmedNoneCoveredControlReading::default();
    let Some(hidden_log) = history
        .hidden_root
        .and_then(|hidden_root| history.roots.get(&hidden_root))
    else {
        return Ok(reading);
    };
    for persisted in class_confirmed_none_covered_states_for_the_control(history)? {
        reading.class_confirmed_none_covered_states_chosen += 1;
        let image = CrashImage {
            base: &history.base,
            writes: &history.writes,
            persisted: persisted.clone(),
        };
        let untouched_unit = hidden_log.units.iter().find(|unit| {
            unit.device == CONFIRMED_NONE_COVERED_CONTROL_DEVICE_NUMBER
                && !unit_differs_in_the_image(&image, unit)
                && !a_persisted_write_overlaps_the_unit(history, &persisted, unit)
        });
        let Some(unit) = untouched_unit else {
            reading.no_unit_to_change += 1;
            continue;
        };
        reading.trials += 1;
        if the_hidden_root_class_with_the_base(history, &history.base, &persisted, hidden_log)
            == "class_confirmed_none_covered"
        {
            reading.class_confirmed_none_covered_before_the_change += 1;
        }
        let mut changed_base = history.base.clone();
        changed_base.flip_byte(
            DeviceIdentity(CONFIRMED_NONE_COVERED_CONTROL_DEVICE_NUMBER),
            DeviceOffsetInBytes(unit.offset_in_bytes),
            0,
        );
        if the_hidden_root_class_with_the_base(history, &changed_base, &persisted, hidden_log)
            == "class_some_unit_covered"
        {
            reading.changed_to_class_some_unit_covered += 1;
        }
    }
    Ok(reading)
}

/// 所有已确认档带藏根的历史合起来的对照读数（产物里的汇总行）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ConfirmedNoneCoveredControlTotal {
    histories: u64,
    histories_not_made: u64,
    histories_violated: u64,
    trials: u64,
    changed_to_class_some_unit_covered: u64,
}

impl ConfirmedNoneCoveredControlTotal {
    /// 汇总行的判定与判决行里算进检测对照失败数的条数：有一条历史 violated 或停机算 1 条；一条都没试成（总试数 0）也算 1 条，不当通过。
    fn failures(&self) -> u64 {
        self.histories_violated + u64::from(self.histories > 0 && self.trials == 0)
    }

    fn status(&self) -> &'static str {
        if self.histories_violated > 0 {
            "violated"
        } else if self.trials == 0 {
            "not_made"
        } else {
            "holds"
        }
    }
}

/// 在产物里打这条历史的对照行，并把读数记进汇总。
fn emit_the_confirmed_none_covered_control(
    history: &RecordedHistory,
    total: &mut ConfirmedNoneCoveredControlTotal,
) {
    total.histories += 1;
    match class_confirmed_none_covered_to_class_some_unit_covered_control(history) {
        Ok(reading) => {
            let status = the_confirmed_none_covered_control_status(
                reading.trials,
                reading.class_confirmed_none_covered_before_the_change,
                reading.changed_to_class_some_unit_covered,
            );
            total.trials += reading.trials;
            total.changed_to_class_some_unit_covered += reading.changed_to_class_some_unit_covered;
            match status {
                "not_made" => total.histories_not_made += 1,
                "violated" => total.histories_violated += 1,
                _ => {}
            }
            emit_result(&format!(
                "name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history={} class_confirmed_none_covered_states_chosen={} trials={} class_confirmed_none_covered_before_the_change={} changed_to_class_some_unit_covered={} no_unit_to_change={} status={status}",
                history_name(&history.spec),
                reading.class_confirmed_none_covered_states_chosen,
                reading.trials,
                reading.class_confirmed_none_covered_before_the_change,
                reading.changed_to_class_some_unit_covered,
                reading.no_unit_to_change
            ));
        }
        Err(reason) => {
            total.histories_violated += 1;
            emit_result(&format!(
                "name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history={} status=stopped detail={}",
                history_name(&history.spec),
                token_of(&reason)
            ));
        }
    }
}

fn emit_the_confirmed_none_covered_control_total(total: &ConfirmedNoneCoveredControlTotal) {
    emit_result(&format!(
        "name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered_total histories={} histories_not_made={} histories_violated={} trials={} changed_to_class_some_unit_covered={} status={}",
        total.histories,
        total.histories_not_made,
        total.histories_violated,
        total.trials,
        total.changed_to_class_some_unit_covered,
        total.status()
    ));
}

/// 剩下那块盘上有没有实例代号比 `instance` 新的、自证过的 journal 记录（装置自己扫：候选槽 + `JournalRecord::parse`，不经核心层的 `scan_journal`）。
fn kept_disk_holds_a_record_of_a_newer_instance(
    image: &CrashImage<'_>,
    device_number: u32,
    instance: u32,
) -> bool {
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    ImageReader::candidate_journal_slots(image, device_number)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|slot| {
            ImageReader::read(
                image,
                device_number,
                ring_start + slot * JOURNAL_RECORD_BYTES,
                record_bytes,
            )
        })
        .filter_map(|bytes| JournalRecord::parse(&bytes, expected_filesystem_identifier))
        .any(|record| record.instance.0 > instance)
}

/// 甲一、甲二的判法本身（登记 5.1 第三条）：在这条历史里按日志挑一个「新实例的记录与系统配置都已落到剩下那块盘、被藏根仍在环里」的状态，
/// 甲一、甲二都必须避开被藏根。没挑到状态记「没造出」（量作废，不当通过）。交回读数。
fn arm_witness_control(history: &RecordedHistory) -> AnchorReading {
    let subject = history_name(&history.spec);
    let Some(hidden_root) = history.hidden_root else {
        return AnchorReading {
            anchor: "arm_witness_control".to_string(),
            subject,
            applicable: false,
            holds: true,
            detail: "no_hidden_root".to_string(),
        };
    };
    let mut examined = 0u64;
    let mut found: Option<String> = None;
    let mut failure: Option<String> = None;
    'segments: for shard_index in 0..=history.segments.len() {
        let mut candidate_states: Vec<Vec<bool>> = Vec::new();
        let collected = for_each_state_of_segment(
            &history.writes,
            &history.segments,
            shard_index,
            &mut |persisted| {
                let configuration_persisted = |device: u32| {
                    history
                        .writes
                        .iter()
                        .zip(persisted)
                        .any(|(write, is_persisted)| {
                            *is_persisted
                                && write.kind == StepKind::SystemConfigurationSlot
                                && write.device.0 == device
                        })
                };
                let record_persisted = |device: u32| {
                    history
                        .writes
                        .iter()
                        .zip(persisted)
                        .any(|(write, is_persisted)| {
                            *is_persisted
                                && write.kind == StepKind::JournalRecord
                                && write.device.0 == device
                        })
                };
                if [0u32, 1u32]
                    .into_iter()
                    .any(|device| configuration_persisted(device) && record_persisted(device))
                {
                    candidate_states.push(persisted.to_vec());
                }
                Ok(())
            },
        );
        if collected.is_err() {
            continue;
        }
        for persisted in candidate_states {
            let image = CrashImage {
                base: &history.base,
                writes: &history.writes,
                persisted,
            };
            for device_number in [0u32, 1u32] {
                let device = DeviceIdentity(device_number);
                let kept = kept_disk(&image, device);
                let today_report = recover(&kept, JournalPolicy::Consult);
                if outcome_root(&today_report.outcome) != Some(hidden_root)
                    || !kept_disk_holds_a_record_of_a_newer_instance(
                        &image,
                        device_number,
                        hidden_root.0,
                    )
                    || newest_configuration_instance_on_the_kept_disk(&kept, device)
                        .is_none_or(|instance| instance <= hidden_root.0)
                {
                    continue;
                }
                examined += 1;
                for arm in [Arm::NewerInstanceRecord, Arm::ConfigurationInstance] {
                    let result = arm_result_on_the_kept_disk(
                        &image,
                        device,
                        &history.pool_geometry,
                        arm,
                        &today_report,
                    );
                    if result.avoided_roots == 0 {
                        failure = Some(format!(
                            "{} 没避开被藏根（只留盘 {device_number}，段 {shard_index}）",
                            arm.name()
                        ));
                    }
                }
                found = Some(format!("segment={shard_index},kept_disk={device_number}"));
                break 'segments;
            }
        }
    }
    AnchorReading {
        anchor: "arm_witness_control".to_string(),
        subject,
        applicable: found.is_some(),
        holds: failure.is_none(),
        detail: format!(
            "state={} examined={examined} failure={}",
            found.as_deref().unwrap_or("none_found"),
            failure.as_deref().unwrap_or("none")
        ),
    }
}

// ============================================================================
// 十五、产物：每条历史一行摘要、一行全部计数、按登记第六节的那几种比例逐个报（`question` 写分子那个计数的名字；分子、分母、万分比），再按族与几何合并报一遍。
//    比例只报数，判定语言（Z1–Z8）留给报告；这里只在判决行里放「没过」类的计数。
// ============================================================================

fn emit_ratio(question: &str, scope: &str, label: &str, numerator: u64, denominator: u64) {
    let per_ten_thousand = if denominator == 0 {
        "none".to_string()
    } else {
        format!(
            "{}",
            u128::from(numerator) * 10_000 / u128::from(denominator)
        )
    };
    emit_result(&format!(
        "name=ratio question={question} scope={scope} label={label} numerator={numerator} denominator={denominator} per_ten_thousand={per_ten_thousand}"
    ));
}

fn sum_over_kept_disks(counters: &Counters, suffix_of: &dyn Fn(u32) -> String) -> u64 {
    [0u32, 1u32]
        .into_iter()
        .map(|device| counters.get(&suffix_of(device)))
        .sum()
}

/// 一份计数（一条历史，或一族一几何合起来的）上登记第六节要的那几种比例。每一行的 `question` 就是分子那个计数的名字
/// （按留哪块盘分开的计数去掉 `kept<盘号>` 那一段、按 `label` 末尾的 `kept_<…>` 加起来），`label` 写这份计数是谁。
fn emit_questions(scope: &str, label: &str, counters: &Counters) {
    for (kept_label, devices) in [
        ("0", vec![0u32]),
        ("1", vec![1u32]),
        ("both_disks_summed", vec![0u32, 1u32]),
    ] {
        let sum = |name_of: &dyn Fn(u32) -> String| -> u64 {
            devices
                .iter()
                .map(|device| counters.get(&name_of(*device)))
                .sum()
        };
        let states = sum(&|device| format!("today.kept{device}.states"));
        let question_label = format!("{label}.kept_{kept_label}");
        emit_ratio(
            "today.pick.hidden_root_covered",
            scope,
            &question_label,
            sum(&|device| format!("today.kept{device}.pick.hidden_root_covered")),
            states,
        );
        emit_ratio(
            "today.outcome.failed_without_root",
            scope,
            &question_label,
            sum(&|device| format!("today.kept{device}.outcome.failed_without_root")),
            states,
        );
        emit_ratio(
            "today.outcome.failed_with_root",
            scope,
            &question_label,
            sum(&|device| format!("today.kept{device}.outcome.failed_with_root")),
            states,
        );
        emit_ratio(
            "today.pick.hidden_root",
            scope,
            &question_label,
            sum(&|device| format!("today.kept{device}.pick.hidden_root_covered"))
                + sum(&|device| format!("today.kept{device}.pick.hidden_root_intact")),
            states,
        );
        let legal_today = sum(&|device| format!("today.kept{device}.pick.legal"));
        let covered_hidden_today =
            sum(&|device| format!("today.kept{device}.pick.hidden_root_covered"));
        for arm in [Arm::ConfigurationInstance, Arm::NewerInstanceRecord] {
            let arm_name = arm.name();
            let arm_label = format!("{question_label}.{arm_name}");
            emit_ratio(
                &format!("{arm_name}.avoided_a_legal_root"),
                scope,
                &arm_label,
                sum(&|device| format!("{arm_name}.kept{device}.avoided_a_legal_root")),
                legal_today,
            );
            emit_ratio(
                &format!("{arm_name}.avoided_a_legal_root.then_none"),
                scope,
                &arm_label,
                sum(&|device| format!("{arm_name}.kept{device}.avoided_a_legal_root.then_none")),
                legal_today,
            );
            emit_ratio(
                &format!("{arm_name}.on_a_covered_hidden_root.then_legal"),
                scope,
                &arm_label,
                sum(&|device| {
                    format!("{arm_name}.kept{device}.on_a_covered_hidden_root.then_legal")
                }),
                covered_hidden_today,
            );
            emit_ratio(
                &format!("{arm_name}.on_a_covered_hidden_root.then_hidden_root_covered"),
                scope,
                &arm_label,
                sum(&|device| {
                    format!(
                        "{arm_name}.kept{device}.on_a_covered_hidden_root.then_hidden_root_covered"
                    )
                }),
                covered_hidden_today,
            );
        }
    }
    let after_the_row_publish = counters.get("after_the_row_publish.states");
    for class in [
        "class_confirmed_none_covered",
        "class_some_unit_covered",
        "class_unconfirmed_none_covered",
    ] {
        let counter_name = format!("after_the_row_publish.{class}");
        emit_ratio(
            &counter_name,
            scope,
            label,
            counters.get(&counter_name),
            after_the_row_publish,
        );
    }
    for suffix in [
        "",
        ".class_confirmed_none_covered",
        ".class_some_unit_covered",
        ".class_unconfirmed_none_covered",
    ] {
        let counter_name = format!(
            "after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot{suffix}"
        );
        emit_ratio(
            &counter_name,
            scope,
            label,
            counters.get(&counter_name),
            after_the_row_publish,
        );
    }
    emit_ratio(
        "after_the_row_publish.hidden_root_slot_overwritten",
        scope,
        label,
        counters.get("after_the_row_publish.hidden_root_slot_overwritten"),
        after_the_row_publish,
    );
}

fn emit_history_summary(history: &RecordedHistory) {
    let expansions: BTreeMap<&'static str, usize> =
        (0..=history.segments.len()).fold(BTreeMap::new(), |mut map, shard_index| {
            *map.entry(expansion_of_shard(history, shard_index).name())
                .or_insert(0) += 1;
            map
        });
    let total_states: u64 = (0..=history.segments.len())
        .map(|shard_index| {
            history.segments.get(shard_index).map_or(1, |segment| {
                state_count_of_segment(&history.writes, segment).unwrap_or(0)
            })
        })
        .sum();
    let ranges_by_label: BTreeMap<&str, usize> =
        history
            .fault_ranges
            .iter()
            .fold(BTreeMap::new(), |mut map, range| {
                *map.entry(range.label).or_insert(0) += 1;
                map
            });
    emit_result(&format!(
        "name=history history={} hidden_root={} fault_ranges={} segments={} writes={} states_closed_form={total_states} expansions={} mount_refused={} mount_stopped_after={} instance={} first_publish_root={} warm_up_publishes={} shadow_ledger_branch={} isolated_slots_per_device={} abandoned_roots_unreadable={} core_ring_slots_holding_an_abandoned_root={} reads_intercepted={} ranges_written_over={} consistency_problems={}",
        history_name(&history.spec),
        history.hidden_root.map_or("none".to_string(), |(instance, txg)| format!("{instance}:{txg}")),
        token_of(&format!("{ranges_by_label:?}")),
        history.segments.len(),
        history.writes.len(),
        token_of(&format!("{expansions:?}")),
        token_of(&format!("{:?}", history.mount.refused.as_deref().map(|text| text.chars().take(120).collect::<String>()))),
        token_of(&format!("{:?}", history.mount.stopped_after_the_mount)),
        history.mount.instance.map_or("none".to_string(), |instance| instance.to_string()),
        history.mount.row_publish_root.map_or("none".to_string(), |(instance, txg)| format!("{instance}:{txg}")),
        history.mount.warm_up_publishes,
        token_of(&history.mount.shadow_ledger_branch),
        token_of(&format!("{:?}", history.mount.isolated_slots_per_device)),
        history.mount.abandoned_roots_unreadable,
        token_of(&format!("{:?}", history.mount.ring_slots_recorded_as_holding_an_abandoned_root)),
        token_of(&format!("{:?}", history.mount.intercepted_reads_by_label)),
        token_of(&format!("{:?}", history.mount.ranges_written_over_by_label)),
        token_of(&format!("{:?}", history.consistency_problems)),
    ));
}

// ============================================================================
// 十六、模式：建历史、按片判、合并、打产物。
// ============================================================================

/// 并行建历史（每条历史自己的实例 1、实例 2 与全知日志）；按 `specs` 的次序交回。
fn build_the_histories(specs: &[HistorySpec]) -> Result<Vec<RecordedHistory>, String> {
    use std::sync::Mutex;
    let next_spec = std::sync::atomic::AtomicUsize::new(0);
    let slots: Mutex<Vec<Option<Result<RecordedHistory, String>>>> =
        Mutex::new((0..specs.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..worker_threads().min(specs.len().max(1)) {
            scope.spawn(|| loop {
                let index = next_spec.fetch_add(1, Ordering::SeqCst);
                let Some(spec) = specs.get(index) else {
                    break;
                };
                let built = record_the_history(spec);
                slots.lock().expect("历史表的锁没被毒化")[index] = Some(built);
            });
        }
    });
    slots
        .into_inner()
        .expect("历史表的锁没被毒化")
        .into_iter()
        .zip(specs)
        .map(|(built, spec)| {
            built
                .expect("每个下标都有线程建过")
                .map_err(|reason| format!("{}：{reason}", history_name(spec)))
        })
        .collect()
}

struct CommandLine {
    full: bool,
    items: Vec<String>,
}

fn parse_the_command_line(arguments: &[String]) -> Result<CommandLine, String> {
    let mut command_line = CommandLine {
        full: false,
        items: Vec::new(),
    };
    let mut remaining = arguments.iter();
    while let Some(argument) = remaining.next() {
        match argument.as_str() {
            "--full" => command_line.full = true,
            "--item" => command_line.items.push(
                remaining
                    .next()
                    .ok_or_else(|| "--item 后面缺历史名".to_string())?
                    .clone(),
            ),
            unknown => return Err(format!("认不得的参数 {unknown}")),
        }
    }
    Ok(command_line)
}

fn selected_specs(items: &[String], shape: OverwriteShape) -> Result<Vec<HistorySpec>, String> {
    let every_spec = history_specs_of_the_shape(shape);
    if items.is_empty() {
        return Ok(every_spec);
    }
    let mut selected = Vec::new();
    for item in items {
        let spec = every_spec
            .iter()
            .find(|spec| &history_name(spec) == item)
            .ok_or_else(|| {
                format!(
                    "没有历史叫 {item}；可点名的：{}",
                    every_spec
                        .iter()
                        .map(history_name)
                        .collect::<Vec<_>>()
                        .join("、")
                )
            })?;
        if !selected.contains(spec) {
            selected.push(*spec);
        }
    }
    Ok(selected)
}

/// 按片判完（或取库）的一批历史：合并后每条历史的计数，按历史次序。
struct JudgedHistories {
    counters_by_history: Vec<Counters>,
    judged: u64,
    reused: u64,
}

fn merge_the_report(
    histories: &[RecordedHistory],
    jobs: &[ShardJob<'_>],
    report: &RunReport,
) -> JudgedHistories {
    let names: Vec<String> = histories
        .iter()
        .map(|history| history_name(&history.spec))
        .collect();
    let mut counters_by_history = vec![Counters::default(); histories.len()];
    let (mut judged, mut reused) = (0u64, 0u64);
    for (job, result) in jobs.iter().zip(&report.results) {
        let Some(result) = result else {
            continue;
        };
        if result.reused {
            reused += 1;
        } else {
            judged += 1;
        }
        let position = names
            .iter()
            .position(|name| name == &job.history_name)
            .expect("片属于建好的某条历史");
        counters_by_history[position].merge(&result.counters);
    }
    JudgedHistories {
        counters_by_history,
        judged,
        reused,
    }
}

/// 建好 `specs` 的历史、按片判（或取库）、合并。`Err` 里是该停下交回的原因。
fn judge_the_histories(
    specs: &[HistorySpec],
    command_line: &CommandLine,
    sentinel_check: bool,
) -> Result<(Vec<RecordedHistory>, JudgedHistories), String> {
    let histories = build_the_histories(specs)?;
    let fingerprint = match closure_fingerprint() {
        Ok(fingerprint) => Some(fingerprint),
        Err(reason) => {
            eprintln!("reuse_disabled: {reason}");
            None
        }
    };
    let library = if fingerprint.is_some() {
        match library_directory() {
            Ok(directory) => Some(directory),
            Err(reason) => {
                eprintln!("reuse_disabled: {reason}");
                None
            }
        }
    } else {
        None
    };
    let share = shard_share_from_the_environment()?;
    let jobs = plan_the_shards(&histories);
    let options = RunOptions {
        library: library.as_deref(),
        fingerprint: fingerprint.as_ref().filter(|_| library.is_some()),
        read_library: !command_line.full,
        share,
    };
    let report = run_the_shards(&jobs, &options);
    if let Some(reason) = &report.stop {
        return Err(reason.clone());
    }
    let judged_histories = merge_the_report(&histories, &jobs, &report);
    eprintln!(
        "现判 {}、复用 {}（片 {} 个，这一份 {:?}）",
        judged_histories.judged,
        judged_histories.reused,
        jobs.len(),
        share
    );
    if sentinel_check && share.is_none() {
        let checked = recheck_the_sentinels(&jobs, &report)?;
        eprintln!("哨兵重判 {checked} 片，与库里的逐名相同");
    }
    drop(jobs);
    Ok((histories, judged_histories))
}

fn emit_counts(history_label: &str, counters: &Counters) {
    let fields: String = counters
        .0
        .iter()
        .map(|(name, amount)| format!(" {name}={amount}"))
        .collect();
    emit_result(&format!("name=counts history={history_label}{fields}"));
}

/// 同一族同一几何的历史合起来的计数（"hidden-root-unconfirmed.default" 这样的名字）。
fn aggregate_by_family_and_geometry(
    histories: &[RecordedHistory],
    counters_by_history: &[Counters],
) -> BTreeMap<String, Counters> {
    let mut aggregates: BTreeMap<String, Counters> = BTreeMap::new();
    for (history, counters) in histories.iter().zip(counters_by_history) {
        aggregates
            .entry(format!(
                "{}{}.{}",
                history.spec.overwrite_shape.label_prefix(),
                history.spec.family.name(),
                history.spec.geometry.name()
            ))
            .or_default()
            .merge(counters);
    }
    aggregates
}

const ARM_WITNESS_HISTORY_NAME: &str = "hidden-root-unconfirmed.default.units-64.overwrites-10";

/// 甲一、甲二判法对照挑状态用的那条历史：两个覆盖写形状各有一条（补臂的带 `units-kept.` 前缀）。
fn arm_witness_history_name(shape: OverwriteShape) -> String {
    format!("{}{ARM_WITNESS_HISTORY_NAME}", shape.label_prefix())
}

/// 登记 5.1 路径真跑到的补跑：Q1 在 G默认 合计是 0 时，同一组起点、影子账关着、记录读得出再跑一遍，报「择中而被盖」合计。
fn emit_the_shadow_ledger_off_control(
    command_line: &CommandLine,
    shape: OverwriteShape,
) -> Result<u64, String> {
    let specs = shadow_ledger_off_history_specs(shape);
    let (histories, judged) = judge_the_histories(&specs, command_line, false)?;
    let mut covered_total = 0u64;
    for (history, counters) in histories.iter().zip(&judged.counters_by_history) {
        emit_history_summary(history);
        emit_counts(&history_name(&history.spec), counters);
        emit_questions("history", &history_name(&history.spec), counters);
        covered_total += sum_over_kept_disks(counters, &|device| {
            format!("today.kept{device}.pick.hidden_root_covered")
        });
    }
    emit_result(&format!(
        "name=positive_control control=shadow_ledger_off_covered_total histories={} hidden_root_covered_total={covered_total} status={}",
        histories.len(),
        if covered_total > 0 {
            "covered_states_exist_without_the_protection"
        } else {
            "no_covered_state_even_without_the_protection"
        }
    ));
    Ok(covered_total)
}

fn emit_the_product(
    histories: &[RecordedHistory],
    judged: &JudgedHistories,
    command_line: &CommandLine,
    shape: OverwriteShape,
) -> Result<(), String> {
    // 登记修订 12.4 的读法不加字段（第一段产物的 config 行逐字不变）；补臂（修订 12.11）加一个字段说明覆盖写的形状。
    let shape_field = match shape {
        OverwriteShape::OneUnit => "",
        OverwriteShape::KeepingTheFirstFileUnits => {
            " overwrite_shape=replacing_writes_keep_the_first_file_units_before_the_hidden_publish"
        }
    };
    emit_result(&format!(
        "name=config histories={} disk_geometries=default:{}+narrow:{} journal_ring_bytes={JOURNAL_RING_BYTES_LOCAL} root_ring_slots_per_region={ROOT_RING_SLOTS_PER_REGION_LOCAL} full_expansion_writes_at_most={FULL_EXPANSION_WRITES_AT_MOST_LOCAL} overwrites_after_the_mount={OVERWRITES_AFTER_THE_MOUNT_LOCAL} arms=today,newer_instance_record,configuration_instance{shape_field}",
        histories.len(),
        DEVICE_BYTES_DEFAULT_LOCAL,
        DEVICE_BYTES_NARROW_LOCAL
    ));
    let mut mismatches = run_constants_and_anchors();
    let differing_source_files = report_the_source_file_fingerprints();
    let (mut consistency_problems, mut refused_mounts, mut detection_failures) = (0u64, 0u64, 0u64);
    let mut confirmed_none_covered_control = ConfirmedNoneCoveredControlTotal::default();
    for history in histories {
        emit_history_summary(history);
        consistency_problems += u64::from(!history.consistency_problems.is_empty());
        refused_mounts += u64::from(history.mount.refused.is_some());
        for reading in [
            anchor_root_slot_writes_land_by_the_ring_rule(history),
            anchor_warm_up_publishes_are_at_most_three(history),
            anchor_first_publish_txg_follows_the_newest_readable(history),
        ] {
            mismatches += u64::from(reading.applicable && !reading.holds);
            emit_anchor_reading(&reading);
        }
        detection_failures += u64::from(!emit_detection_control(history));
        // 登记修订 12.12：B4 的类 a 到类 b 对照，只在已确认档带藏根的历史上跑（类 a 只出在这一族）。
        if matches!(
            history.spec.family,
            Family::HiddenRoot(Confirmation::Confirmed)
        ) {
            emit_the_confirmed_none_covered_control(history, &mut confirmed_none_covered_control);
        }
    }
    if confirmed_none_covered_control.histories > 0 {
        emit_the_confirmed_none_covered_control_total(&confirmed_none_covered_control);
        // 对照没过（violated、停机）或一次都没试成，都算进判决行的「检测对照失败」条数；全过时这个数不变。
        detection_failures += confirmed_none_covered_control.failures();
    }
    let witness_name = arm_witness_history_name(shape);
    let witness = histories
        .iter()
        .find(|history| history_name(&history.spec) == witness_name)
        .map(arm_witness_control);
    let witness_status = match &witness {
        None => "not_run",
        Some(reading) if !reading.applicable => "not_run",
        Some(reading) if reading.holds => "holds",
        Some(_) => "violated",
    };
    if let Some(reading) = &witness {
        emit_anchor_reading(reading);
    }
    for (history, counters) in histories.iter().zip(&judged.counters_by_history) {
        emit_counts(&history_name(&history.spec), counters);
        emit_questions("history", &history_name(&history.spec), counters);
    }
    let aggregates = aggregate_by_family_and_geometry(histories, &judged.counters_by_history);
    for (label, counters) in &aggregates {
        emit_counts(label, counters);
        emit_questions("aggregate", label, counters);
    }
    let covered_on_the_default_geometry: u64 = aggregates
        .iter()
        .filter(|(label, _)| {
            label
                .strip_prefix(shape.label_prefix())
                .is_some_and(|rest| rest.starts_with("hidden-root-") && rest.ends_with(".default"))
        })
        .map(|(_, counters)| {
            sum_over_kept_disks(counters, &|device| {
                format!("today.kept{device}.pick.hidden_root_covered")
            })
        })
        .sum();
    let default_unconfirmed_label =
        format!("{}hidden-root-unconfirmed.default", shape.label_prefix());
    let hidden_root_picks_on_the_default_unconfirmed: Option<u64> =
        aggregates.get(&default_unconfirmed_label).map(|counters| {
            sum_over_kept_disks(counters, &|device| {
                format!("today.kept{device}.pick.hidden_root_covered")
            }) + sum_over_kept_disks(counters, &|device| {
                format!("today.kept{device}.pick.hidden_root_intact")
            })
        });
    let off_switch_covered = if covered_on_the_default_geometry == 0
        && hidden_root_picks_on_the_default_unconfirmed.is_some()
    {
        Some(emit_the_shadow_ledger_off_control(command_line, shape)?)
    } else {
        None
    };
    let slot_hit_states: u64 = aggregates
        .values()
        .map(|counters| {
            counters.get("after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot")
        })
        .sum();
    emit_result(&format!(
        "name=verdict constants_and_anchor_mismatches={mismatches} histories_with_consistency_problems={consistency_problems} histories_whose_mount_was_refused={refused_mounts} detection_control_failures={detection_failures} arm_witness_control={witness_status} source_files_differing_from_registration={differing_source_files} path_reached_on_the_default_unconfirmed_histories={} hidden_root_covered_states_on_the_default_geometry={covered_on_the_default_geometry} shadow_ledger_off_covered_states={} first_publish_txg_hits_the_hidden_root_slot_states={slot_hit_states}",
        hidden_root_picks_on_the_default_unconfirmed.map_or("not_run".to_string(), |picks| (picks > 0).to_string()),
        off_switch_covered.map_or("not_run".to_string(), |covered| covered.to_string()),
    ));
    Ok(())
}

fn run_single_device(arguments: &[String], shape: OverwriteShape) -> i32 {
    let command_line = match parse_the_command_line(arguments) {
        Ok(command_line) => command_line,
        Err(reason) => {
            eprintln!("用法错：{reason}");
            return 2;
        }
    };
    let specs = match selected_specs(&command_line.items, shape) {
        Ok(specs) => specs,
        Err(reason) => {
            eprintln!("用法错：{reason}");
            return 2;
        }
    };
    let (histories, judged) = match judge_the_histories(&specs, &command_line, true) {
        Ok(judged_histories) => judged_histories,
        Err(reason) => {
            eprintln!("STOP: {reason}");
            return 3;
        }
    };
    if matches!(shard_share_from_the_environment(), Ok(Some(_))) {
        return 0;
    }
    if let Err(reason) = emit_the_product(&histories, &judged, &command_line, shape) {
        eprintln!("STOP: {reason}");
        return 3;
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    0
}

fn run_anchors_mode() -> i32 {
    let mismatches = run_constants_and_anchors();
    let differing_source_files = report_the_source_file_fingerprints();
    match closure_fingerprint() {
        Ok(fingerprint) => emit_result(&format!(
            "name=closure_fingerprint status=keyed key={} toolchain_lines={}",
            fingerprint.key_hex,
            fingerprint.toolchain_lines.len()
        )),
        Err(reason) => emit_result(&format!(
            "name=closure_fingerprint status=refused detail={}",
            token_of(&reason.chars().take(400).collect::<String>())
        )),
    }
    emit_result(&format!(
        "name=verdict constants_and_anchor_mismatches={mismatches} source_files_differing_from_registration={differing_source_files}"
    ));
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    i32::from(mismatches > 0)
}

fn run_list_items() -> i32 {
    for spec in every_history_spec() {
        println!("{}", history_name(&spec));
    }
    0
}

/// 登记 5.5 最后一条：建全部历史报状态数（闭式），再在点名的历史上抽 100 个状态量完整判一个状态与单独一次 `recover` 的挂钟，按实测外推。
/// 挂钟不进产物，这一个模式的输出不入库。
fn run_measure(arguments: &[String]) -> i32 {
    let measured_name = arguments
        .first()
        .map_or(ARM_WITNESS_HISTORY_NAME, String::as_str);
    let specs = main_history_specs();
    let started = std::time::Instant::now();
    let histories = match build_the_histories(&specs) {
        Ok(histories) => histories,
        Err(reason) => {
            eprintln!("建历史失败：{reason}");
            return 2;
        }
    };
    eprintln!(
        "建 {} 条历史用了 {:.1} 秒",
        histories.len(),
        started.elapsed().as_secs_f64()
    );
    let mut total_states = 0u64;
    for history in &histories {
        let states: u64 = (0..=history.segments.len())
            .map(|shard_index| {
                history.segments.get(shard_index).map_or(1, |segment| {
                    state_count_of_segment(&history.writes, segment).unwrap_or(0)
                })
            })
            .sum();
        total_states += states;
        emit_result(&format!(
            "name=measure_history history={} segments={} writes={} states_closed_form={states} mount_refused={}",
            history_name(&history.spec),
            history.segments.len(),
            history.writes.len(),
            history.mount.refused.is_some()
        ));
    }
    let Some(measured) = histories
        .iter()
        .find(|history| history_name(&history.spec) == measured_name)
    else {
        eprintln!("没有历史叫 {measured_name}");
        return 2;
    };
    let measured_states: u64 = (0..=measured.segments.len())
        .map(|shard_index| {
            measured.segments.get(shard_index).map_or(1, |segment| {
                state_count_of_segment(&measured.writes, segment).unwrap_or(0)
            })
        })
        .sum();
    let step = (measured_states / 100).max(1);
    let (mut visited, mut sampled) = (0u64, 0u64);
    let (mut evaluate_seconds, mut recover_seconds) = (0f64, 0f64);
    for shard_index in 0..=measured.segments.len() {
        let walked = for_each_state_of_segment(
            &measured.writes,
            &measured.segments,
            shard_index,
            &mut |persisted| {
                visited += 1;
                if visited % step != 0 || sampled >= 100 {
                    return Ok(());
                }
                sampled += 1;
                let image = CrashImage {
                    base: &measured.base,
                    writes: &measured.writes,
                    persisted: persisted.to_vec(),
                };
                let recover_started = std::time::Instant::now();
                let _report = recover(
                    &kept_disk(&image, DeviceIdentity(0)),
                    JournalPolicy::Consult,
                );
                recover_seconds += recover_started.elapsed().as_secs_f64();
                let evaluate_started = std::time::Instant::now();
                evaluate_one_state(measured, persisted, &mut Counters::default())?;
                evaluate_seconds += evaluate_started.elapsed().as_secs_f64();
                Ok(())
            },
        );
        if let Err(reason) = walked {
            eprintln!("STOP: {reason}");
            return 3;
        }
    }
    let seconds_per_state =
        evaluate_seconds / f64::from(u32::try_from(sampled.max(1)).expect("抽样数装得进 u32"));
    #[allow(
        clippy::cast_precision_loss,
        reason = "外推只要量级，状态总数远小于 2^52"
    )]
    let single_thread_seconds = total_states as f64 * seconds_per_state;
    let threads = worker_threads();
    #[allow(clippy::cast_precision_loss, reason = "线程数远小于 2^52")]
    let spread_seconds = single_thread_seconds / threads as f64;
    emit_result(&format!(
        "name=measure_timing history={measured_name} states_of_this_history={measured_states} sampled_states={sampled} evaluate_seconds_for_the_sample={evaluate_seconds:.3} one_recover_seconds_for_the_sample={recover_seconds:.3} seconds_per_evaluated_state={seconds_per_state:.6}"
    ));
    emit_result(&format!(
        "name=measure_extrapolation states_all_histories={total_states} single_thread_seconds={single_thread_seconds:.0} threads={threads} seconds_spread_over_the_threads={spread_seconds:.0} hours_spread_over_the_threads={:.2}",
        spread_seconds / 3600.0
    ));
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    0
}

/// 样本模式：在点名的历史上取三个状态（什么都没持久、前一半的段全持久、整条流全持久），逐个报两块盘都在与各只留一块盘时 `recover` 的结局
/// （失败成员原样），好看单盘恢复为什么成功或失败。输出不入库。
fn run_outcome_samples(arguments: &[String]) -> i32 {
    let name = arguments
        .first()
        .map_or(ARM_WITNESS_HISTORY_NAME, String::as_str);
    let Some(spec) = every_history_spec()
        .into_iter()
        .find(|spec| history_name(spec) == name)
    else {
        eprintln!("没有历史叫 {name}");
        return 2;
    };
    let history = match record_the_history(&spec) {
        Ok(history) => history,
        Err(reason) => {
            eprintln!("建历史失败：{reason}");
            return 2;
        }
    };
    for (label, persisted) in [
        "nothing_persisted",
        "first_half_of_the_segments",
        "everything_persisted",
    ]
    .into_iter()
    .zip(three_states_for_the_controls(&history))
    {
        let image = CrashImage {
            base: &history.base,
            writes: &history.writes,
            persisted,
        };
        let both = recover(&image, JournalPolicy::Consult);
        let mut reports = vec![("both_disks".to_string(), both)];
        for device_number in [0u32, 1u32] {
            reports.push((
                format!("only_disk_{device_number}"),
                recover(
                    &kept_disk(&image, DeviceIdentity(device_number)),
                    JournalPolicy::Consult,
                ),
            ));
        }
        for (kept, report) in reports {
            let failure = match &report.outcome {
                RecoveryOutcome::Failed { failure, .. } => format!("{failure:?}"),
                RecoveryOutcome::FileRead { .. } | RecoveryOutcome::NoFile { .. } => {
                    "none".to_string()
                }
            };
            emit_result(&format!(
                "name=outcome_sample history={name} state={label} kept={kept} class={} root={} effective_root={} failure={} journal={}",
                outcome_class(&report.outcome),
                token_of(&format!("{:?}", outcome_root(&report.outcome))),
                token_of(&format!(
                    "{:?}",
                    report.effective_root.map(|(instance, txg)| (instance.0, txg.0))
                )),
                token_of(&failure),
                token_of(&format!("{:?}", report.journal))
            ));
        }
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    0
}

/// 准入判法退 78（拒绝）时装置也退 78；别的非 0 退 1（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）。
const PREFLIGHT_REFUSED_EXIT_CODE: i32 = 78;

/// `main` 的第一句：直接起 `python3 preflight.py check <本文件> [--force] -- <参数…>`（不经 `sh -c`）判准入与运行条件，
/// 不满足就照判法的退出码退出、一步都不做；强制跑（判法的第一行以 `forced` 起头）时交回没满足的条件摘要，要写进产物。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e182_unreadable_abandoned_root_forks.rs");
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
        std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE)
    });
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    match output.status.code() {
        Some(0) => {}
        Some(PREFLIGHT_REFUSED_EXIT_CODE) => std::process::exit(PREFLIGHT_REFUSED_EXIT_CODE),
        Some(_) | None => std::process::exit(1),
    }
    let verdict_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    forced_summary_of_the_preflight_verdict(&verdict_line)
}

/// 准入判法 stdout 第一行（制表符分段）以 `forced` 起头时，交回没满足的条件摘要（最后一段，空白换成下划线），要写进产物；
/// 满足（`met`）或没声明时是 `None`。
fn forced_summary_of_the_preflight_verdict(verdict_line: &str) -> Option<String> {
    if verdict_line.starts_with("forced") {
        let last_segment = verdict_line.rsplit('\t').next().unwrap_or("");
        Some(
            last_segment
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("_"),
        )
    } else {
        None
    }
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    if let Some(summary) = forced_summary {
        if arguments.first().map(String::as_str) == Some("list-items") {
            eprintln!("name=preflight forced={summary}");
        } else {
            emit_result(&format!("name=preflight forced={summary}"));
        }
    }
    let exit_code = match arguments.first().map(String::as_str) {
        Some("anchors") => run_anchors_mode(),
        Some("list-items") => run_list_items(),
        Some("measure") => run_measure(&arguments[1..]),
        Some("outcome-samples") => run_outcome_samples(&arguments[1..]),
        Some("single-device") => run_single_device(&arguments[1..], OverwriteShape::OneUnit),
        Some("single-device-units-kept") => {
            run_single_device(&arguments[1..], OverwriteShape::KeepingTheFirstFileUnits)
        }
        Some(_) | None => {
            eprintln!("用法：e182_unreadable_abandoned_root_forks anchors | list-items | measure [历史名] | outcome-samples [历史名] | single-device [--full] [--item <历史名>]… | single-device-units-kept [--full] [--item <历史名>]…");
            2
        }
    };
    std::process::exit(exit_code);
}

// ============================================================================
// 单测：钉绝对值的断言（登记第七节）、每条臂与每个检测各一条会红的用例（变异表 `crates/mutations.tsv` 里 E182 的行点名这里的测试名）。
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn small_spec() -> HistorySpec {
        HistorySpec {
            family: Family::HiddenRoot(Confirmation::Unconfirmed),
            geometry: DiskGeometry::Default,
            first_file_units: 1,
            overwrites_before_the_hidden_publish: 5,
            overwrite_shape: OverwriteShape::OneUnit,
        }
    }

    /// 一条小历史（G默认、一个单元、5 次覆盖写、未确认档）：几条用例共用，建一次。
    fn small_history() -> &'static RecordedHistory {
        static HISTORY: OnceLock<RecordedHistory> = OnceLock::new();
        HISTORY.get_or_init(|| record_the_history(&small_spec()).expect("小历史建得起来"))
    }

    fn two_device_pool_with_pattern_at(offset: u64, length: usize) -> MemoryPool {
        let mut pool = empty_pool(1 << 22);
        for device in device_identities() {
            pool.devices
                .get_mut(&device)
                .expect("池里有这块盘")
                .write(DeviceOffsetInBytes(offset), &vec![0xA5; length]);
        }
        pool
    }

    #[test]
    fn constants_and_anchors_all_pass() {
        assert_eq!(
            run_constants_and_anchors(),
            0,
            "常量回比、单元区槽数钉值、枚举状态数等于闭式计数、已确认档缺较新系统配置时写之前被拒，都要成立"
        );
    }

    #[test]
    fn enumerated_state_counts_are_pinned_to_independently_computed_numbers() {
        for ((in_place, unit), pinned) in [
            ((0usize, 0usize), 0u64),
            ((3, 0), 7),
            ((16, 0), 65_535),
            ((0, 16), 65_535),
            ((0, 17), 1),
            ((6, 11), 127),
            ((17, 0), 131_071),
        ] {
            let (seen, closed_form) = enumerated_and_closed_form_state_counts(in_place, unit);
            assert_eq!(
                seen, pinned,
                "在原地写 {in_place}、单元写 {unit} 的段里枚举出的状态数"
            );
            assert_eq!(closed_form, pinned, "同一段的闭式");
        }
    }

    #[test]
    fn a_segment_of_exactly_sixteen_writes_is_expanded_in_full_and_seventeen_takes_the_quick_tier()
    {
        assert_eq!(expansion_of_segment(16), Expansion::FullSubsets);
        assert_eq!(
            expansion_of_segment(17),
            Expansion::QuickInPlaceSubsetsTimesUnitWritesNoneOrAll
        );
        assert_eq!(expansion_of_segment(0), Expansion::FullSubsets);
    }

    #[test]
    fn the_unit_area_slot_counts_are_the_pinned_ones() {
        assert_eq!(unit_area_slots_of(DiskGeometry::Default), 211_968);
        assert_eq!(unit_area_slots_of(DiskGeometry::Narrow), 146_465);
    }

    #[test]
    fn the_configuration_instance_arm_avoids_only_a_strictly_newer_instance() {
        assert!(
            !avoided_by_configuration_instance(1, 1),
            "实例代号相等不避开"
        );
        assert!(
            avoided_by_configuration_instance(2, 1),
            "配置里的实例代号更新就避开"
        );
        assert!(
            !avoided_by_configuration_instance(1, 2),
            "配置里的实例代号更旧不避开"
        );
    }

    #[test]
    fn a_range_that_reads_zero_is_cleared_for_its_own_disk_only_and_forgotten_once_written_over() {
        let mut inner = SparseBlockDevice::new(
            1 << 20,
            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES_LOCAL),
        );
        inner
            .write_at(
                DeviceOffsetInBytes(4096),
                &[0x77; 2048],
                WriteDurability::Plain,
            )
            .expect("内存盘写得进");
        let state = Rc::new(RefCell::new(ReadsZeroState::with_ranges(vec![
            ReadsZeroRange {
                device: DeviceIdentity(0),
                offset: 4608,
                length: 512,
                label: "test",
            },
        ])));
        let mut device = ReadsZeroDevice {
            identity: DeviceIdentity(0),
            inner,
            state: Rc::clone(&state),
        };
        let mut buffer = [0xFFu8; 2048];
        device
            .read_at(DeviceOffsetInBytes(4096), &mut buffer)
            .expect("读得出");
        assert!(
            buffer[..512].iter().all(|byte| *byte == 0x77),
            "区间之前读回真字节"
        );
        assert!(
            buffer[512..1024].iter().all(|byte| *byte == 0),
            "区间里读回全零"
        );
        assert!(
            buffer[1024..].iter().all(|byte| *byte == 0x77),
            "区间之后读回真字节"
        );
        assert_eq!(state.borrow().intercepted_reads, vec![1], "读拦了一次");
        let other_disk = ReadsZeroDevice {
            identity: DeviceIdentity(1),
            inner: SparseBlockDevice::new(
                1 << 20,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES_LOCAL),
            ),
            state: Rc::clone(&state),
        };
        let mut untouched = [0xFFu8; 512];
        other_disk
            .read_at(DeviceOffsetInBytes(4608), &mut untouched)
            .expect("读得出");
        assert_eq!(
            state.borrow().intercepted_reads,
            vec![1],
            "别的盘上同一偏移不拦"
        );
        device
            .write_at(
                DeviceOffsetInBytes(4608),
                &[0x11; 512],
                WriteDurability::Plain,
            )
            .expect("写得进");
        device
            .read_at(DeviceOffsetInBytes(4608), &mut untouched)
            .expect("读得出");
        assert!(
            untouched.iter().all(|byte| *byte == 0x11),
            "写盖过之后读回写下的字节"
        );
    }

    #[test]
    fn the_kept_disk_view_hides_the_other_disk_and_zeroes_a_masked_root_slot() {
        let pool = two_device_pool_with_pattern_at(8192, 1024);
        let image = CrashImage {
            base: &pool,
            writes: &[],
            persisted: Vec::new(),
        };
        let mut view = kept_disk(&image, DeviceIdentity(1));
        assert_eq!(
            PoolReader::device_identities(&view),
            vec![DeviceIdentity(0), DeviceIdentity(1)],
            "掉了的盘身份还在盘表里"
        );
        assert_eq!(
            PoolReader::read(&view, DeviceIdentity(0), DeviceOffsetInBytes(8192), 512),
            None,
            "另一块盘整块掉了，身份在、读不到"
        );
        assert_eq!(
            PoolReader::read(&view, DeviceIdentity(1), DeviceOffsetInBytes(8192), 1024),
            Some(vec![0xA5; 1024])
        );
        view.masked_root_slots.push((8192 + 512, 512));
        let read_back = PoolReader::read(&view, DeviceIdentity(1), DeviceOffsetInBytes(8192), 1024)
            .expect("留下的盘读得到");
        assert!(
            read_back[..512].iter().all(|byte| *byte == 0xA5),
            "没罩住的一半不动"
        );
        assert!(
            read_back[512..].iter().all(|byte| *byte == 0),
            "被避开的根槽读回全零"
        );
    }

    /// 被盖按单元自己所在的那块盘判：盘 1 上的单元被改，只留盘 0 看不出、只留盘 1 看得出。
    #[test]
    fn a_unit_is_covered_when_its_own_disks_bytes_differ_from_the_log() {
        let mut pool = two_device_pool_with_pattern_at(16384, 512);
        let log = RootLog {
            units: vec![
                UnitLog {
                    device: 0,
                    offset_in_bytes: 16384,
                    bytes: vec![0xA5; 512],
                },
                UnitLog {
                    device: 1,
                    offset_in_bytes: 16384,
                    bytes: vec![0xA5; 512],
                },
            ],
            account_readable: true,
        };
        pool.flip_byte(DeviceIdentity(1), DeviceOffsetInBytes(16384), 0);
        let image = CrashImage {
            base: &pool,
            writes: &[],
            persisted: Vec::new(),
        };
        assert!(
            !unit_differs_in_the_image(&image, &log.units[0]),
            "盘 0 上的单元没动"
        );
        assert!(
            unit_differs_in_the_image(&image, &log.units[1]),
            "盘 1 上的单元被改了，要读盘 1 才看得出"
        );
        assert!(!some_unit_of_the_root_is_covered(&image, &log, Some(0)));
        assert!(some_unit_of_the_root_is_covered(&image, &log, Some(1)));
        assert!(some_unit_of_the_root_is_covered(&image, &log, None));
    }

    /// B4 的类看两块盘：被盖的单元只在盘 1 上，也是类 b；没被盖时已确认档是类 a、未确认档是类 c。
    #[test]
    fn the_hidden_root_class_looks_at_the_units_on_both_disks() {
        let pool = two_device_pool_with_pattern_at(16384, 512);
        let log = RootLog {
            units: vec![
                UnitLog {
                    device: 0,
                    offset_in_bytes: 16384,
                    bytes: vec![0xA5; 512],
                },
                UnitLog {
                    device: 1,
                    offset_in_bytes: 16384,
                    bytes: vec![0xA5; 512],
                },
            ],
            account_readable: true,
        };
        let intact = CrashImage {
            base: &pool,
            writes: &[],
            persisted: Vec::new(),
        };
        assert_eq!(
            class_of_the_hidden_root_in_the_state(&intact, &log, Confirmation::Confirmed),
            "class_confirmed_none_covered"
        );
        assert_eq!(
            class_of_the_hidden_root_in_the_state(&intact, &log, Confirmation::Unconfirmed),
            "class_unconfirmed_none_covered"
        );
        let mut covered_pool = pool.clone();
        covered_pool.flip_byte(DeviceIdentity(1), DeviceOffsetInBytes(16384), 0);
        let covered = CrashImage {
            base: &covered_pool,
            writes: &[],
            persisted: Vec::new(),
        };
        assert_eq!(
            class_of_the_hidden_root_in_the_state(&covered, &log, Confirmation::Confirmed),
            "class_some_unit_covered"
        );
        assert_eq!(
            class_of_the_hidden_root_in_the_state(&covered, &log, Confirmation::Unconfirmed),
            "class_some_unit_covered"
        );
    }

    #[test]
    fn counters_round_trip_through_their_text_and_merge_by_name() {
        let mut counters = Counters::default();
        counters.add("a.b", 3);
        counters.add("c", 5);
        let mut other = Counters::default();
        other.add("a.b", 4);
        other.add("d", 1);
        let mut merged = counters.clone();
        merged.merge(&other);
        assert_eq!(merged.get("a.b"), 7);
        assert_eq!(merged.get("c"), 5);
        assert_eq!(merged.get("d"), 1);
        assert_eq!(merged.get("missing"), 0);
        assert_eq!(
            Counters::from_text(&merged.to_text()).expect("文本读得回"),
            merged
        );
        let mut merged_the_other_way = other.clone();
        merged_the_other_way.merge(&counters);
        assert_eq!(merged_the_other_way, merged, "合并与次序无关");
    }

    #[test]
    fn a_stored_shard_is_read_back_only_under_the_same_key() {
        let directory =
            std::env::temp_dir().join(format!("e182-library-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("建得起临时目录");
        let fingerprint = ClosureFingerprint {
            key_hex: "ab".repeat(32),
            toolchain_lines: vec!["rustc 1".to_string()],
        };
        let key = shard_key_text(&fingerprint, "history-a", 3, Expansion::FullSubsets);
        let mut counters = Counters::default();
        counters.add("states", 9);
        store_shard(&directory, &key, &counters).expect("存得进");
        assert_eq!(read_stored_shard(&directory, &key), Some(counters));
        for other_key in [
            shard_key_text(&fingerprint, "history-b", 3, Expansion::FullSubsets),
            shard_key_text(&fingerprint, "history-a", 4, Expansion::FullSubsets),
            shard_key_text(&fingerprint, "history-a", 3, Expansion::FinalState),
        ] {
            assert_eq!(
                read_stored_shard(&directory, &other_key),
                None,
                "键里任何一段不同都取不到"
            );
        }
        std::fs::remove_dir_all(&directory).expect("删得掉临时目录");
    }

    fn is_a_write(operation: &RetainedOperation) -> bool {
        matches!(
            operation.operation.kind,
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
        )
    }

    /// 被藏根那次发布写下的每个单元、每条 journal 记录、根槽都在读回全零的区间里（未确认档不罩系统配置，已确认档罩每块盘较新的那一槽）。
    #[test]
    fn the_unreadable_ranges_cover_every_unit_record_and_root_slot_of_the_hidden_publish() {
        let first = run_the_first_instance(&small_spec()).expect("实例 1 跑得完");
        let fixed = fixed_geometry_of_the_pool();
        let (_, unconfirmed_ranges) =
            base_and_fault_ranges(&small_spec(), &first).expect("基镜像建得起来");
        let confirmed_spec = HistorySpec {
            family: Family::HiddenRoot(Confirmation::Confirmed),
            ..small_spec()
        };
        let (_, confirmed_ranges) =
            base_and_fault_ranges(&confirmed_spec, &first).expect("基镜像建得起来");
        let final_operations = &first.operations[first.last_publish_start..];
        for (kind, label) in [
            (StepKind::UnitWrite, "unit"),
            (StepKind::JournalRecord, "journal_record"),
            (StepKind::RootRecordFua, "root_slot"),
        ] {
            let writes: Vec<(u32, u64, u64)> = final_operations
                .iter()
                .filter(|operation| {
                    is_a_write(operation) && fixed.classify(&operation.operation) == kind
                })
                .map(|operation| {
                    (
                        operation.operation.device.0,
                        operation.operation.offset.0,
                        operation.operation.length,
                    )
                })
                .collect();
            assert!(!writes.is_empty(), "被藏根那次发布里有 {label} 这一种写");
            for (device, offset, length) in writes {
                for ranges in [&unconfirmed_ranges, &confirmed_ranges] {
                    assert!(
                        ranges.iter().any(|range| range.label == label
                            && range.device.0 == device
                            && range.offset == offset
                            && range.length == length),
                        "{label} 写（盘 {device}，偏移 {offset}）没被读回全零的区间罩住"
                    );
                }
            }
        }
        let count_of = |ranges: &[ReadsZeroRange], label: &str| {
            ranges.iter().filter(|range| range.label == label).count()
        };
        assert_eq!(
            count_of(&unconfirmed_ranges, "root_slot"),
            1,
            "被藏根只有一个根槽写"
        );
        assert_eq!(
            count_of(&unconfirmed_ranges, "newer_system_configuration_slot"),
            0,
            "未确认档不罩系统配置槽"
        );
        assert_eq!(
            count_of(&confirmed_ranges, "newer_system_configuration_slot"),
            2,
            "已确认档罩每块盘较新的那一个系统配置槽"
        );
    }

    #[test]
    fn the_small_history_follows_the_registered_anchors() {
        let history = small_history();
        assert!(
            history.consistency_problems.is_empty(),
            "{:?}",
            history.consistency_problems
        );
        assert_eq!(history.mount.refused, None, "未确认档的可写挂载做得成");
        assert_eq!(history.mount.stopped_after_the_mount, None);
        assert_eq!(history.mount.instance, Some(2), "实例 2");
        assert!(history.hidden_root.is_some());
        for reading in [
            anchor_root_slot_writes_land_by_the_ring_rule(history),
            anchor_warm_up_publishes_are_at_most_three(history),
            anchor_first_publish_txg_follows_the_newest_readable(history),
        ] {
            assert!(reading.applicable && reading.holds, "{reading:?}");
        }
        assert!(
            history
                .mount
                .ring_slots_recorded_as_holding_an_abandoned_root
                .is_empty(),
            "被藏根读回全零，核心层的表里没有它的根槽（停机条款第二条，原因是读回全零）"
        );
    }

    // ------------------------------------------------------------------------
    // 覆盖写保持 s 个单元的补臂（登记修订 12.11）。
    // ------------------------------------------------------------------------

    /// 补臂的 spec：G默认、第一个文件 64 个单元、2 次覆盖写、未确认档（用例里只取实例 1 与一条小历史，p 取 2 省时间）。
    fn spec_of_the_shape(shape: OverwriteShape) -> HistorySpec {
        HistorySpec {
            first_file_units: 64,
            overwrites_before_the_hidden_publish: 2,
            overwrite_shape: shape,
            ..small_spec()
        }
    }

    fn small_units_kept_history() -> &'static RecordedHistory {
        static HISTORY: OnceLock<RecordedHistory> = OnceLock::new();
        HISTORY.get_or_init(|| {
            record_the_history(&spec_of_the_shape(OverwriteShape::KeepingTheFirstFileUnits))
                .expect("补臂的小历史建得起来")
        })
    }

    #[test]
    fn the_units_kept_arm_has_nine_prefixed_histories_all_at_sixty_four_units_next_to_the_eighteen_registered_ones(
    ) {
        let kept = units_kept_history_specs();
        assert_eq!(kept.len(), 9, "(默认 2 个起点 + 窄盘 1 个起点) × 三族");
        assert_eq!(main_history_specs().len(), 18, "第一段的 18 条不动");
        for spec in &kept {
            assert_eq!(
                spec.first_file_units, 64,
                "s = 1 时两种形状同是一个单元，不另建"
            );
            assert_eq!(
                spec.overwrite_shape,
                OverwriteShape::KeepingTheFirstFileUnits
            );
            assert!(history_name(spec).starts_with("units-kept."));
        }
        let kept_names: BTreeSet<String> = kept.iter().map(history_name).collect();
        let main_names: BTreeSet<String> = main_history_specs().iter().map(history_name).collect();
        assert_eq!(kept_names.len(), 9, "补臂的历史名互不相同");
        assert!(
            kept_names.is_disjoint(&main_names),
            "补臂的历史名与第一段的 18 条不相交（片结果库的键含历史名）"
        );
        assert!(
            kept_names.contains(&arm_witness_history_name(
                OverwriteShape::KeepingTheFirstFileUnits
            )),
            "甲一、甲二判法对照挑状态的那条历史在补臂里"
        );
    }

    /// 实例 1 结束时恢复读回的文件内容就是最后一次覆盖写的整份内容：补臂是 s 个数据单元，第一段的读法是一个数据单元（登记修订 12.4）。
    #[test]
    fn the_file_after_the_replacing_writes_has_all_its_units_in_the_units_kept_arm_and_one_unit_in_the_registered_reading(
    ) {
        for (shape, units) in [
            (OverwriteShape::OneUnit, 1usize),
            (OverwriteShape::KeepingTheFirstFileUnits, 64usize),
        ] {
            let spec = spec_of_the_shape(shape);
            let first = run_the_first_instance(&spec).expect("实例 1 跑得完");
            let mut pool = empty_pool(spec.geometry.device_bytes());
            pool.apply(&first.operations);
            let RecoveryOutcome::FileRead { content, .. } =
                recover(&pool, JournalPolicy::Consult).outcome
            else {
                panic!("{shape:?}：实例 1 结束的镜像上恢复应当读回文件");
            };
            let expected_length = units * DATA_UNIT_PAYLOAD_BYTES_LOCAL;
            assert_eq!(content.len(), expected_length, "{shape:?}：文件的字节数");
            assert_eq!(
                content,
                deterministic_content(101, expected_length),
                "{shape:?}：内容是最后一次覆盖写（标签 100 + 1）的整份内容"
            );
        }
    }

    /// 补臂的被藏根引用的单元比第一段的读法多至少 63 个（64 个数据单元对 1 个）；绝对值另钉：至少 64 个。
    #[test]
    fn the_hidden_root_of_the_units_kept_arm_references_at_least_sixty_three_more_units_than_the_registered_reading(
    ) {
        let referenced_units = |shape: OverwriteShape| -> usize {
            let first = run_the_first_instance(&spec_of_the_shape(shape)).expect("实例 1 跑得完");
            first
                .roots
                .get(&first.last_root)
                .expect("被藏根在全知日志里")
                .units
                .len()
        };
        let one_unit = referenced_units(OverwriteShape::OneUnit);
        let kept = referenced_units(OverwriteShape::KeepingTheFirstFileUnits);
        assert!(kept >= 64, "补臂的被藏根至少引用 64 个落点，实际 {kept}");
        assert!(
            kept >= one_unit + 63,
            "补臂 {kept} 个落点，第一段的读法 {one_unit} 个"
        );
    }

    /// 补臂的小历史跟登记的锚点：挂载做成、可写、被藏根在、每段的状态数展得开（快档里原地写不超过 40 条）、检测对照每次都判得出。
    #[test]
    fn the_small_units_kept_history_follows_the_registered_anchors_and_every_segment_can_be_enumerated(
    ) {
        let history = small_units_kept_history();
        assert!(
            history.consistency_problems.is_empty(),
            "{:?}",
            history.consistency_problems
        );
        assert_eq!(history.mount.refused, None, "未确认档的可写挂载做得成");
        assert_eq!(
            history.mount.stopped_after_the_mount, None,
            "挂载之后的 3 次覆盖写做得完"
        );
        assert_eq!(history.mount.instance, Some(2), "实例 2");
        assert!(history.hidden_root.is_some());
        for reading in [
            anchor_root_slot_writes_land_by_the_ring_rule(history),
            anchor_warm_up_publishes_are_at_most_three(history),
            anchor_first_publish_txg_follows_the_newest_readable(history),
        ] {
            assert!(reading.applicable && reading.holds, "{reading:?}");
        }
        for (segment_number, segment) in history.segments.iter().enumerate() {
            assert!(
                state_count_of_segment(&history.writes, segment).is_some(),
                "第 {segment_number} 段（{} 次写）的状态数展得开",
                segment.len()
            );
        }
        let hidden = history.hidden_root.expect("带藏根的历史");
        assert!(
            history
                .roots
                .get(&hidden)
                .expect("被藏根在全知日志里")
                .units
                .len()
                >= 64,
            "被藏根至少引用 64 个落点"
        );
        let (trials, detected, _without_a_unit) = detection_control(history);
        assert!(trials > 0, "至少试了一次");
        assert_eq!(detected, trials, "每次改了单元的字节「被盖」都判得出");
    }

    #[test]
    fn the_detection_control_finds_every_changed_unit() {
        let (trials, detected, _without_a_unit) = detection_control(small_history());
        assert!(trials > 0, "至少试了一次");
        assert_eq!(detected, trials, "每次改了单元的字节「被盖」都判得出");
    }

    // ------------------------------------------------------------------------
    // B4 的阳性对照（登记 5.2 最后一句，修订 12.12）。
    // ------------------------------------------------------------------------

    /// 已确认档带藏根的小历史（G默认、64 个单元、2 次覆盖写）：类 a 只出在这一族里。
    fn confirmed_spec_of_the_shape(shape: OverwriteShape) -> HistorySpec {
        HistorySpec {
            family: Family::HiddenRoot(Confirmation::Confirmed),
            ..spec_of_the_shape(shape)
        }
    }

    fn small_confirmed_history() -> &'static RecordedHistory {
        static HISTORY: OnceLock<RecordedHistory> = OnceLock::new();
        HISTORY.get_or_init(|| {
            record_the_history(&confirmed_spec_of_the_shape(OverwriteShape::OneUnit))
                .expect("已确认档的小历史建得起来")
        })
    }

    fn small_confirmed_units_kept_history() -> &'static RecordedHistory {
        static HISTORY: OnceLock<RecordedHistory> = OnceLock::new();
        HISTORY.get_or_init(|| {
            record_the_history(&confirmed_spec_of_the_shape(
                OverwriteShape::KeepingTheFirstFileUnits,
            ))
            .expect("补臂的已确认档小历史建得起来")
        })
    }

    /// 对照在已确认档的小历史上：挑到两个类 a 状态，每个状态里盘 0 上都换得了一个单元，换之前是类 a、换之后是类 b。
    #[test]
    fn the_confirmed_none_covered_control_turns_two_chosen_class_confirmed_none_covered_states_into_class_some_unit_covered_by_changing_a_unit_on_device_zero(
    ) {
        let history = small_confirmed_history();
        assert!(
            history.consistency_problems.is_empty(),
            "{:?}",
            history.consistency_problems
        );
        let reading = class_confirmed_none_covered_to_class_some_unit_covered_control(history)
            .expect("枚举得完");
        assert_eq!(
            reading.class_confirmed_none_covered_states_chosen, 2,
            "{reading:?}"
        );
        assert_eq!(reading.no_unit_to_change, 0, "{reading:?}");
        assert_eq!(reading.trials, 2, "{reading:?}");
        assert_eq!(
            reading.class_confirmed_none_covered_before_the_change, 2,
            "换之前是「已确认、没被盖」类：{reading:?}"
        );
        assert_eq!(
            reading.changed_to_class_some_unit_covered, 2,
            "换之后是「有单元被盖」类：{reading:?}"
        );
        assert_eq!(
            the_confirmed_none_covered_control_status(
                reading.trials,
                reading.class_confirmed_none_covered_before_the_change,
                reading.changed_to_class_some_unit_covered
            ),
            "holds"
        );
    }

    /// 补臂的已确认档小历史上同样（被藏根引用 64 个以上的单元，挑到的类 a 状态里盘 0 上换得了一个）。挑不到类 a 状态时本用例不判（产物里记 not_made）。
    #[test]
    fn the_confirmed_none_covered_control_holds_on_the_units_kept_arm_wherever_it_can_be_made() {
        let history = small_confirmed_units_kept_history();
        assert!(
            history.consistency_problems.is_empty(),
            "{:?}",
            history.consistency_problems
        );
        let reading = class_confirmed_none_covered_to_class_some_unit_covered_control(history)
            .expect("枚举得完");
        assert_eq!(
            reading.class_confirmed_none_covered_before_the_change, reading.trials,
            "试过的状态换之前都是「已确认、没被盖」类：{reading:?}"
        );
        assert_eq!(
            reading.changed_to_class_some_unit_covered, reading.trials,
            "试过的状态换之后都判成「有单元被盖」类：{reading:?}"
        );
    }

    /// 判定本身：没试成不当通过；换之前不是类 a 或换之后不是类 b，都是 violated。
    #[test]
    fn the_confirmed_none_covered_control_status_holds_only_when_every_trial_went_from_class_confirmed_none_covered_to_class_some_unit_covered(
    ) {
        assert_eq!(
            the_confirmed_none_covered_control_status(0, 0, 0),
            "not_made"
        );
        assert_eq!(the_confirmed_none_covered_control_status(2, 2, 2), "holds");
        assert_eq!(
            the_confirmed_none_covered_control_status(2, 2, 1),
            "violated"
        );
        assert_eq!(
            the_confirmed_none_covered_control_status(2, 2, 0),
            "violated"
        );
        assert_eq!(
            the_confirmed_none_covered_control_status(2, 1, 2),
            "violated"
        );
        let one_made_and_held = ConfirmedNoneCoveredControlTotal {
            histories: 3,
            histories_not_made: 2,
            histories_violated: 0,
            trials: 2,
            changed_to_class_some_unit_covered: 2,
        };
        assert_eq!(one_made_and_held.status(), "holds");
        assert_eq!(one_made_and_held.failures(), 0);
        let none_made = ConfirmedNoneCoveredControlTotal {
            histories: 3,
            histories_not_made: 3,
            ..ConfirmedNoneCoveredControlTotal::default()
        };
        assert_eq!(none_made.status(), "not_made");
        assert_eq!(none_made.failures(), 1, "一次都没试成不当通过");
        let one_violated = ConfirmedNoneCoveredControlTotal {
            histories_violated: 1,
            trials: 4,
            changed_to_class_some_unit_covered: 3,
            ..one_made_and_held
        };
        assert_eq!(one_violated.status(), "violated");
        assert_eq!(one_violated.failures(), 1);
    }

    #[test]
    fn both_arms_avoid_the_hidden_root_in_a_state_where_the_new_instance_reached_the_kept_disk() {
        let reading = arm_witness_control(small_history());
        assert!(reading.applicable, "挑得到一个状态：{reading:?}");
        assert!(reading.holds, "{reading:?}");
    }

    /// 甲一只读剩下那块盘：新实例的记录只落在掉了的那块盘上时，不避开。拿两块盘一起读的判法对照，确有这样的状态会被两块盘一起读骗到。
    #[test]
    fn the_newer_instance_record_arm_reads_only_the_kept_disk() {
        let history = small_history();
        let hidden = history.hidden_root.expect("带藏根的历史");
        let (mut examined, mut would_be_fooled_by_the_other_disk) = (0u64, 0u64);
        'segments: for shard_index in 0..=history.segments.len() {
            let mut candidates: Vec<(Vec<bool>, u32)> = Vec::new();
            for_each_state_of_segment(
                &history.writes,
                &history.segments,
                shard_index,
                &mut |persisted| {
                    let record_on = |device: u32| {
                        history
                            .writes
                            .iter()
                            .zip(persisted)
                            .any(|(write, is_persisted)| {
                                *is_persisted
                                    && write.kind == StepKind::JournalRecord
                                    && write.device.0 == device
                            })
                    };
                    for kept in [0u32, 1u32] {
                        if record_on(1 - kept) && !record_on(kept) && candidates.len() < 300 {
                            candidates.push((persisted.to_vec(), kept));
                        }
                    }
                    Ok(())
                },
            )
            .expect("枚举得完");
            for (persisted, kept) in candidates {
                let image = CrashImage {
                    base: &history.base,
                    writes: &history.writes,
                    persisted,
                };
                let device = DeviceIdentity(kept);
                let kept_view = kept_disk(&image, device);
                let report = recover(&kept_view, JournalPolicy::Consult);
                if outcome_root(&report.outcome) != Some(hidden) {
                    continue;
                }
                let result = arm_result_on_the_kept_disk(
                    &image,
                    device,
                    &history.pool_geometry,
                    Arm::NewerInstanceRecord,
                    &report,
                );
                assert_eq!(
                    result.avoided_roots, 0,
                    "更新实例的记录只在另一块盘上，剩下这块盘上看不见，甲一不该避开"
                );
                let root =
                    root_record_of(&kept_view, hidden).expect("被藏根在剩下那块盘的根环里读得出");
                if avoided_by_newer_instance_record(&image, &root) {
                    would_be_fooled_by_the_other_disk += 1;
                }
                examined += 1;
                if examined >= 3000 {
                    break 'segments;
                }
            }
        }
        assert!(
            examined > 0,
            "找得到被藏根被择中、新实例的记录只在另一块盘上的状态"
        );
        assert!(
            would_be_fooled_by_the_other_disk > 0,
            "这些状态里至少有一个会被两块盘一起读骗到，这条用例才分得出读几块盘"
        );
    }

    #[test]
    fn a_shard_is_judged_the_same_way_twice_and_counts_the_states_of_its_segment() {
        let history = small_history();
        let first = judge_the_shard(history, 0).expect("第 0 段判得完");
        let second = judge_the_shard(history, 0).expect("第 0 段判得完");
        assert_eq!(first, second, "同一片判两次，计数逐名相同");
        let expected =
            state_count_of_segment(&history.writes, &history.segments[0]).expect("展得开");
        assert_eq!(first.get("states"), expected);
        assert_eq!(
            first.get("both.outcome.file_read")
                + first.get("both.outcome.no_file")
                + first.get("both.outcome.failed_with_root")
                + first.get("both.outcome.failed_without_root"),
            expected
        );
        for arm in Arm::EVERY_ARM {
            for device in 0..2 {
                assert_eq!(
                    first.get(&format!("{}.kept{device}.states", arm.name())),
                    expected
                );
            }
        }
    }

    /// 准入判法的第一行以 `forced` 起头时才交回摘要（最后一段，空白换成下划线）；满足或没声明时不交。
    #[test]
    fn the_preflight_verdict_line_gives_a_summary_only_when_forced() {
        assert_eq!(
            forced_summary_of_the_preflight_verdict("forced\tinputs-unchanged\ta b\tc d"),
            Some("c_d".to_string())
        );
        assert_eq!(forced_summary_of_the_preflight_verdict("met\tabc123"), None);
        assert_eq!(forced_summary_of_the_preflight_verdict("undeclared"), None);
        assert_eq!(forced_summary_of_the_preflight_verdict(""), None);
    }

    /// 文件头写了准入与运行条件（在第一行代码之前），`main` 的第一句是 `preflight`（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」）。
    #[test]
    fn the_source_declares_admission_and_run_condition_before_the_code_and_main_starts_with_preflight(
    ) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/bin/e182_unreadable_abandoned_root_forks.rs");
        let text = std::fs::read_to_string(&path).expect("preflight 要判的源文件读得到");
        let first_code_line = text
            .lines()
            .position(|line| line.starts_with("#![") || line.starts_with("use "))
            .expect("有第一行代码");
        let header: Vec<&str> = text.lines().take(first_code_line).collect();
        assert_eq!(
            header
                .iter()
                .filter(|line| line.starts_with("// admission: always "))
                .count(),
            1,
            "文件头恰有一行准入"
        );
        assert_eq!(
            header
                .iter()
                .filter(|line| line.trim_end() == "// run-condition: command python3")
                .count(),
            1,
            "文件头恰有一行运行条件"
        );
        let main_marker = ["fn ", "main() {\n"].concat();
        let after_main = text.split(main_marker.as_str()).nth(1).expect("有入口函数");
        assert!(
            after_main.starts_with("    let forced_summary = preflight();\n"),
            "入口的第一句是 preflight"
        );
    }
}
