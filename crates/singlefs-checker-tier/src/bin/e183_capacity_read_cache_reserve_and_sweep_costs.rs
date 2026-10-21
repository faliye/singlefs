//! E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数） 第一段（登记 `research/prompts/e183-preregistration.md` 第五节 5.5「第一段」）：
//! 回答问题单 `research/prompts/m3-close-cost-measurement-questions.md` 的 H4 一行——最小能 mkfs 并可写挂载的盘上，准入式子（D28（挂载期承诺量）
//! 已定项 1）里除容量、已分配、不可回收之外的六项各多大、谁占大头。
//!
// admission: always 每跑一次都在今天这份 crates 上现量一遍代价数（H4 的读数、H2 的耗时与堆量、H5 的挂载历史），计时每次都是新观测，上一次跑完不替这一次作保
// run-condition: command python3
//!
//! 这一段做的事：本地常量与 `crates/` 回比（登记表头常量表）、计数的全局分配器与 PC-堆、按区间数读写的块设备包装（E31）、
//! 几何锚点的本地移植（A30、A31、A33）、H4 的最小盘搜索（X14：三个最小盘、单调核）、准入读数逐项报（X15）、PC-准入、K30–K32 与 G7 的对读数的核。
//!
//! 第二段（登记 5.5「第二段」，问题单 H2）：E33 的容量格与记录格（X4–X10，文件后端稀疏镜像，每格每轮一个进程、一片结果文件）与 E37 里 H2 的判定
//! （X10 的斜率与类、5 轮同类才写类、PC-线性、PC-平、几何敏感性、判别力自证 ①）。入口 `h2 list-shards`、`h2 shard`、`h2 merge`。
//! 第三段（登记 5.5「第三段」，问题单 H5）：E36 的清扫历史（X16–X21，24 格加「交给后台」对照 1 格，一格一片）、锚点模型的本地移植（A34、A35）、
//! K33、K34、A36、PC-不扫、PC-差分，与 E37 里 H5 的判定（R5b 甲、R5c、几何敏感性、判别力自证 ②）。入口 `h5 list-shards`、`h5 shard`、`h5 merge`。
//! 第四段（H3 读缓存）这一段不做。
//!
//! 只读驱动 `singlefs-core` 的公开入口，不改 `crates/` 里别的任何文件；产物按 `E7RESULT` 行打印到标准输出，每项耗时打到标准错误（产物逐字节可复跑）。
//! 第一段入口：`--list-items`、`--item <项>`、`all`（输出与第一段的入库产物逐字节相同，单测 `segment_one_all_output_equals_the_registered_product_byte_for_byte` 证）。

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::{
    admission_reading_before_a_publish, admission_reading_of_a_writable_mount, AdmissionReading,
    SpaceAdmission,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, FileBackedBlockDevice, PhysicalBlockSizeInBytes,
    WriteDurability,
};
use singlefs_core::instance_table_sweep::{
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    SweepPublishPlacements,
};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_instance_table_sweep_configuration,
    mount_writable_with_space_admission, unmount, MountSpaceAdmission, Mounted, ShadowLedger,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{instance_table_chain_of_root, readable_roots_with_ring_slots};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, FirstFile, PoolVersion, PoolWriter, TransactionUnit,
};
use singlefs_harness::memory_pool::SparseBlockDevice;

/// 这个装置的固定 fsid：与别的 `eNNN` 装置各自独立取一份。
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x38, 0x33, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const PHYSICAL_BLOCK_BYTES: u32 = 512;
const FIXED_STRUCTURE_SLOT_SPACING_BYTES: u32 = 4096;
const MEBIBYTE: u64 = 1 << 20;
const GIBIBYTE: u64 = 1 << 30;

// ============================================================================================
// 本地常量（登记表头常量表）：值抄自下面各行点名的 kb 登记位，不从 `crates/` 引；`constants_against_the_implementation` 逐个回比。
// ============================================================================================

/// 每片实例表数据行：D28（挂载期承诺量） 已定项 3「行宽 88 ⇒ 369 行」；D18（块里携带什么信息） 已定项 11「数据行 369」。
const INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE: u64 = 369;
/// N_switch、暖机至多 R：`28-挂载期承诺量.md` 已定项 3。
const INSTANCE_SWITCHES_ALLOWED_PER_MOUNT: u64 = 3;
const WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH: u64 = 3;
/// 实例表一片的字节：`28-挂载期承诺量.md` 已定项 3「32768 × max(1, …)」。
const INSTANCE_TABLE_PAGE_BYTES: u64 = 32768;
/// 清扫阈值的占位：`.claude/kb/checks-owed.md` C690 那一行「阈值是占位值 369」。
const SWEEP_THRESHOLD_PLACEHOLDER_ROWS: u64 = 369;
/// journal 记录宽、安全系数、默认环长：D23（journal 的角色与格式） 已定项 18、19（环 ≤ 容量 ÷ 4，默认 768 MiB）。
const JOURNAL_RECORD_BYTES: u64 = 4096;
const JOURNAL_SAFETY_FACTOR: u64 = 3;
const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// 分配记录树叶罩槽数、内部扇出、删除批 K：D8（核心索引结构） 已定项 14、16；D28（挂载期承诺量） 已定项 1「环窗口」一条。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
const DELETION_BATCH_DATA_UNITS: u64 = 144;
/// 默认环长下的单元区起点：D3（空间分配） 已定项 10 ④（环末尾的下一个槽）。
const UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING: u64 = 50176;
/// mkfs 默认每区槽数 S：`22-单元原子性怎么合成.md` 已定项 1 字段表；`layout/01-first-txn.md` mkfs 写 8。
const SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
/// 几何锚点（`anchors_e183.py`）与读写计数包装要的布局常量：槽宽 16384、journal 环从槽 1024 起（`23-journal的角色与格式.md` 已定项 2 的布局）、
/// 根环三个区域从槽 64 起每区域隔 3 个 1 MiB chunk（`22-单元原子性怎么合成.md` 已定项 2）、系统配置两槽。
const SLOT_BYTES: u64 = 16384;
const JOURNAL_RING_START_SLOT: u64 = 1024;
const ROOT_RING_REGIONS: u64 = 3;
const ROOT_RING_BASE_SLOT: u64 = 64;
const ROOT_RING_CHUNK_BYTES: u64 = 1_048_576;
const ROOT_RING_PRIME_STEP: u64 = 3;
const SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: u64 = 2;
/// 池是两块盘（第一个可运行目标，D2（RAID 条带策略） 已定项 9）。
const POOL_DEVICE_COUNT: u64 = 2;

// ============================================================================================
// E30：计数的全局分配器与 PC-堆。纯 std，不碰 `crates/`。
// ============================================================================================

/// 包 `std::alloc::System`，原子地记在用字节与峰值。峰值取「出现过的最大在用字节」，所以先占后放之后读它还是那个最大值。
pub struct CountingAllocator {
    in_use_bytes: AtomicU64,
    peak_bytes: AtomicU64,
}

impl CountingAllocator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            in_use_bytes: AtomicU64::new(0),
            peak_bytes: AtomicU64::new(0),
        }
    }

    fn record_growth(&self, bytes: u64) {
        let in_use_now = self.in_use_bytes.fetch_add(bytes, Ordering::Relaxed) + bytes;
        self.peak_bytes.fetch_max(in_use_now, Ordering::Relaxed);
    }

    fn record_shrink(&self, bytes: u64) {
        self.in_use_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    #[must_use]
    pub fn in_use(&self) -> u64 {
        self.in_use_bytes.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn peak(&self) -> u64 {
        self.peak_bytes.load(Ordering::Relaxed)
    }

    /// 把峰值重置成此刻的在用字节：一个量窗从这里开始。
    pub fn reset_peak_to_in_use(&self) {
        self.peak_bytes
            .store(self.in_use_bytes.load(Ordering::Relaxed), Ordering::Relaxed);
    }
}

impl Default for CountingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

fn bytes_of_layout(layout: Layout) -> u64 {
    u64::try_from(layout.size()).unwrap_or(u64::MAX)
}

// SAFETY: 每个方法原样转给 `System`，只在成功之后改计数；计数是原子的，不碰被分配的内存。
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: 调用方保证 `layout` 合法；转给系统分配器，约束不变。
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            self.record_growth(bytes_of_layout(layout));
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: 同 `alloc`。
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            self.record_growth(bytes_of_layout(layout));
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: 调用方保证 `pointer` 是用同一个 `layout` 从这个分配器分出来的；它来自 `System`。
        unsafe { System.dealloc(pointer, layout) };
        self.record_shrink(bytes_of_layout(layout));
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: 调用方保证 `pointer`、`layout`、`new_size` 满足 `GlobalAlloc::realloc` 的契约；转给系统分配器。
        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        if !new_pointer.is_null() {
            let old_bytes = bytes_of_layout(layout);
            let new_bytes = u64::try_from(new_size).unwrap_or(u64::MAX);
            if new_bytes >= old_bytes {
                self.record_growth(new_bytes - old_bytes);
            } else {
                self.record_shrink(old_bytes - new_bytes);
            }
        }
        new_pointer
    }
}

#[global_allocator]
static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator::new();

/// PC-堆（登记 5.3）：量窗里分配并逐页写满一份 64 MiB 的 `Vec<u8>`，峰值多占必须落在 `[64 MiB, 64 MiB + 64 KiB]`。
const HEAP_POSITIVE_CONTROL_BYTES: u64 = 64 << 20;
const HEAP_POSITIVE_CONTROL_SLACK_BYTES: u64 = 65536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HeapPositiveControl {
    peak_over_baseline_bytes: u64,
    holds: bool,
}

fn heap_positive_control(
    allocator: &CountingAllocator,
    upper_slack_bytes: u64,
) -> HeapPositiveControl {
    let baseline_bytes = allocator.in_use();
    allocator.reset_peak_to_in_use();
    let mut buffer =
        vec![0u8; usize::try_from(HEAP_POSITIVE_CONTROL_BYTES).expect("64 MiB 装得进 usize")];
    buffer.fill(0xA5);
    std::hint::black_box(&buffer);
    let peak_over_baseline_bytes = allocator.peak().saturating_sub(baseline_bytes);
    drop(buffer);
    HeapPositiveControl {
        peak_over_baseline_bytes,
        holds: heap_control_holds(peak_over_baseline_bytes, upper_slack_bytes),
    }
}

/// 峰值多占必须落在 `[64 MiB, 64 MiB + 上沿]`。
fn heap_control_holds(peak_over_baseline_bytes: u64, upper_slack_bytes: u64) -> bool {
    peak_over_baseline_bytes >= HEAP_POSITIVE_CONTROL_BYTES
        && peak_over_baseline_bytes <= HEAP_POSITIVE_CONTROL_BYTES + upper_slack_bytes
}

// ============================================================================================
// 几何锚点的本地移植（登记 7.2 A30、A31、A33；独立的那一份是 `anchors_e183.py`，命令一）。
// 只用上面的本地常量与条款句：D8 已定项 14（分配记录树根层级）、D28 已定项 1（W 与环窗口）、已定项 3（链重写）。不读 `crates/` 的几何函数。
// ============================================================================================

/// 一块盘上单元区的头一槽与末一槽：单元区起点 = journal 环末尾的下一个槽，末一槽 = 盘的槽数 − 1。
fn unit_area_first_and_last_slot(device_bytes: u64, journal_ring_bytes: u64) -> (u64, u64) {
    (
        JOURNAL_RING_START_SLOT + journal_ring_bytes / SLOT_BYTES,
        device_bytes / SLOT_BYTES - 1,
    )
}

fn unit_area_slots(device_bytes: u64, journal_ring_bytes: u64) -> u64 {
    let (first_slot, last_slot) = unit_area_first_and_last_slot(device_bytes, journal_ring_bytes);
    last_slot - first_slot + 1
}

/// 一片叶之上第 `level` 层一个位置罩多少槽：812 × 169^level。
fn slots_covered_by_one_position_at_level(level: u32) -> u64 {
    ALLOCATION_RECORD_TREE_LEAF_SLOTS * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level)
}

/// 分配记录树根层级：最小的 R ≥ 1 使两块盘合计的 ⌈盘的槽数 ÷ (812 × 169^(R−1))⌉ 不超过 169（D8 已定项 14；盘的槽数是绝对槽数）。
fn allocation_record_tree_root_level(device_bytes: u64) -> u32 {
    let device_slots = device_bytes / SLOT_BYTES;
    let mut root_level = 1u32;
    while POOL_DEVICE_COUNT
        * device_slots.div_ceil(slots_covered_by_one_position_at_level(root_level - 1))
        > ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
    {
        root_level += 1;
    }
    root_level
}

/// N_ℓ：第 ℓ 层（0 … 根层级 − 1）两块盘合计罩到的位置数，每块盘按单元区首末槽 ⌊末 ÷ 跨度⌋ − ⌊首 ÷ 跨度⌋ + 1。
fn positions_at_each_level(device_bytes: u64, journal_ring_bytes: u64) -> Vec<u64> {
    let (first_slot, last_slot) = unit_area_first_and_last_slot(device_bytes, journal_ring_bytes);
    (0..allocation_record_tree_root_level(device_bytes))
        .map(|level| {
            let span = slots_covered_by_one_position_at_level(level);
            POOL_DEVICE_COUNT * (last_slot / span - first_slot / span + 1)
        })
        .collect()
}

/// 一次删除类发布每盘新占槽数的上界 W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))，K = 144（D28 已定项 1「环窗口」一条）。
fn slots_one_deletion_publish_takes(
    positions: &[u64],
    retirement_tree_internal_levels: u64,
) -> u64 {
    let bound_of_one_level = 2 * (DELETION_BATCH_DATA_UNITS + 26);
    24 + 6
        + retirement_tree_internal_levels
        + positions
            .iter()
            .map(|count| (*count).min(bound_of_one_level))
            .sum::<u64>()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RingWindowByTheClauses {
    unit_area_first_slot: u64,
    unit_area_slots: u64,
    root_level: u32,
    positions_at_each_level: Vec<u64>,
    deletion_publish_slots: u64,
    ring_window_slots: u64,
}

impl RingWindowByTheClauses {
    fn ring_window_bytes(&self) -> u64 {
        self.ring_window_slots * SLOT_BYTES
    }
}

/// 环窗口 = (3S + 1) × W 个槽（D28 已定项 1「环窗口」一条）。
fn ring_window_by_the_clauses(
    device_bytes: u64,
    journal_ring_bytes: u64,
    slots_per_region: u64,
    retirement_tree_internal_levels: u64,
) -> RingWindowByTheClauses {
    let positions = positions_at_each_level(device_bytes, journal_ring_bytes);
    let deletion_publish_slots =
        slots_one_deletion_publish_takes(&positions, retirement_tree_internal_levels);
    let (first_slot, _) = unit_area_first_and_last_slot(device_bytes, journal_ring_bytes);
    RingWindowByTheClauses {
        unit_area_first_slot: first_slot,
        unit_area_slots: unit_area_slots(device_bytes, journal_ring_bytes),
        root_level: allocation_record_tree_root_level(device_bytes),
        deletion_publish_slots,
        ring_window_slots: (ROOT_RING_REGIONS * slots_per_region + 1) * deletion_publish_slots,
        positions_at_each_level: positions,
    }
}

/// 实例切换预留里链重写那一半，每块盘：(N_switch + 1) × 32768 × max(1, ⌈(rows0 + N_switch) ÷ 369⌉)（D28 已定项 3）。
fn instance_table_chain_pages(instance_rows_of_the_switch_reserve: u64) -> u64 {
    (instance_rows_of_the_switch_reserve + INSTANCE_SWITCHES_ALLOWED_PER_MOUNT)
        .div_ceil(INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE)
        .max(1)
}

fn chain_rewrite_bytes_of_the_switch_reserve(instance_rows_of_the_switch_reserve: u64) -> u64 {
    (INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1)
        * INSTANCE_TABLE_PAGE_BYTES
        * instance_table_chain_pages(instance_rows_of_the_switch_reserve)
}

// ============================================================================================
// E31：只记不改的块设备包装，按落点区间分别数读写的次数与字节。区间的分法照 `crates/singlefs-harness/src/segments.rs` 的分类读法另写一份。
// ============================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoredRegion {
    SystemConfiguration,
    RootRing,
    JournalRing,
    UnitAreaAndEverythingElse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CallsAndBytes {
    calls: u64,
    bytes: u64,
}

impl CallsAndBytes {
    fn minus(self, earlier: CallsAndBytes) -> CallsAndBytes {
        CallsAndBytes {
            calls: self.calls - earlier.calls,
            bytes: self.bytes - earlier.bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ReadsAndWrites {
    reads: CallsAndBytes,
    writes: CallsAndBytes,
}

impl ReadsAndWrites {
    fn minus(self, earlier: ReadsAndWrites) -> ReadsAndWrites {
        ReadsAndWrites {
            reads: self.reads.minus(earlier.reads),
            writes: self.writes.minus(earlier.writes),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DeviceTally {
    system_configuration: ReadsAndWrites,
    root_ring: ReadsAndWrites,
    journal_ring: ReadsAndWrites,
    unit_area: ReadsAndWrites,
}

impl DeviceTally {
    fn region_mut(&mut self, region: StoredRegion) -> &mut ReadsAndWrites {
        match region {
            StoredRegion::SystemConfiguration => &mut self.system_configuration,
            StoredRegion::RootRing => &mut self.root_ring,
            StoredRegion::JournalRing => &mut self.journal_ring,
            StoredRegion::UnitAreaAndEverythingElse => &mut self.unit_area,
        }
    }

    fn minus(self, earlier: DeviceTally) -> DeviceTally {
        DeviceTally {
            system_configuration: self
                .system_configuration
                .minus(earlier.system_configuration),
            root_ring: self.root_ring.minus(earlier.root_ring),
            journal_ring: self.journal_ring.minus(earlier.journal_ring),
            unit_area: self.unit_area.minus(earlier.unit_area),
        }
    }
}

/// 固定结构在一块盘上的区间边界（字节偏移）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FixedRegions {
    system_configuration_end: u64,
    root_ring_start: u64,
    root_ring_end: u64,
    journal_ring_start: u64,
    journal_ring_end: u64,
}

impl FixedRegions {
    fn of_geometry(
        slot_spacing_bytes: u64,
        slots_per_region: u64,
        journal_ring_bytes: u64,
    ) -> Self {
        let root_ring_start = ROOT_RING_BASE_SLOT * SLOT_BYTES;
        let journal_ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        FixedRegions {
            system_configuration_end: SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE * slot_spacing_bytes,
            root_ring_start,
            root_ring_end: root_ring_start
                + (ROOT_RING_REGIONS - 1) * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES
                + slots_per_region * slot_spacing_bytes,
            journal_ring_start,
            journal_ring_end: journal_ring_start + journal_ring_bytes,
        }
    }

    fn region_of(&self, offset: u64) -> StoredRegion {
        if offset < self.system_configuration_end {
            StoredRegion::SystemConfiguration
        } else if offset >= self.root_ring_start && offset < self.root_ring_end {
            StoredRegion::RootRing
        } else if offset >= self.journal_ring_start && offset < self.journal_ring_end {
            StoredRegion::JournalRing
        } else {
            StoredRegion::UnitAreaAndEverythingElse
        }
    }
}

/// 包在一块盘外面，读写原样转交，只数。读是 `&self`，所以走 `RefCell`（单线程用）。
struct CountingBlockDevice<Inner: BlockDevice> {
    inner: Inner,
    regions: FixedRegions,
    tally: RefCell<DeviceTally>,
}

impl<Inner: BlockDevice> CountingBlockDevice<Inner> {
    fn new(inner: Inner, regions: FixedRegions) -> Self {
        Self {
            inner,
            regions,
            tally: RefCell::new(DeviceTally::default()),
        }
    }

    fn tally_now(&self) -> DeviceTally {
        *self.tally.borrow()
    }

    fn count_read(&self, offset: DeviceOffsetInBytes, length: usize) {
        let region = self.regions.region_of(offset.0);
        let mut tally = self.tally.borrow_mut();
        let reads = &mut tally.region_mut(region).reads;
        reads.calls += 1;
        reads.bytes += u64::try_from(length).expect("读长装得进 u64");
    }

    fn count_write(&mut self, offset: DeviceOffsetInBytes, length: u64) {
        let region = self.regions.region_of(offset.0);
        let writes = &mut self.tally.get_mut().region_mut(region).writes;
        writes.calls += 1;
        writes.bytes += length;
    }
}

impl<Inner: BlockDevice> BlockDevice for CountingBlockDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.count_read(offset, buffer.len());
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.count_write(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"));
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.count_write(offset, length);
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.inner.discard_at(offset, length)
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

    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.count_read(offset, buffer.len());
        self.inner.reread_at_after_a_bad_read(offset, buffer)
    }

    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        self.inner.keep_the_self_verified_reread(offset, bytes);
    }

    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        self.inner
            .note_the_read_failing_self_verification_and_forget_its_bytes(offset, length_in_bytes);
    }

    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.count_read(offset, buffer.len());
        self.inner.read_at_straight_from_the_disk(offset, buffer)
    }
}

// ============================================================================================
// 池的造法：参数、两块等大的稀疏盘（包上计数）、mkfs、挂载结局的分类。
// ============================================================================================

type CountedDevice = CountingBlockDevice<SparseBlockDevice>;
type CountedPool = Vec<(DeviceIdentity, CountedDevice)>;

fn make_filesystem_parameters(
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: PHYSICAL_BLOCK_BYTES,
            minimum_input_output_bytes: PHYSICAL_BLOCK_BYTES,
            fixed_structure_slot_spacing: FIXED_STRUCTURE_SLOT_SPACING_BYTES,
            journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                slots_per_region,
            )
            .expect("每区槽数落在 4..16 闭区间"),
        },
    }
}

fn counted_pool(device_bytes: u64, journal_ring_bytes: u64, slots_per_region: u64) -> CountedPool {
    let regions = FixedRegions::of_geometry(
        u64::from(FIXED_STRUCTURE_SLOT_SPACING_BYTES),
        slots_per_region,
        journal_ring_bytes,
    );
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                CountingBlockDevice::new(
                    SparseBlockDevice::new(
                        device_bytes,
                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                    ),
                    regions,
                ),
            )
        })
        .collect()
}

/// 两块盘各自此刻的计数，按盘序。
fn tallies_of(pool: &CountedPool) -> Vec<DeviceTally> {
    pool.iter().map(|(_, device)| device.tally_now()).collect()
}

/// 每块盘从 `earlier` 到此刻的计数之差。
fn tally_difference(pool: &CountedPool, earlier: &[DeviceTally]) -> Vec<DeviceTally> {
    pool.iter()
        .zip(earlier)
        .map(|((_, device), before)| device.tally_now().minus(*before))
        .collect()
}

/// 第一个文件的内容：定长 3000 字节，随序号变（新池新建文件的内容）。
fn first_file_content() -> Vec<u8> {
    (0..3000usize)
        .map(|index| u8::try_from((index + 3) % 251).expect("小于 256"))
        .collect()
}

/// 报错只取头一个标识符（变体名）：`Debug` 全文可能带整张结构，产物里不留。
fn variant_name_of(debug_text: &str) -> String {
    debug_text
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect()
}

// ============================================================================================
// X15：准入读数逐项（D28 已定项 1 式子里除「容量」「已分配」「不可回收」之外的六项）。
// ============================================================================================

/// 两个读数入口（登记 X15）：可写挂载那一判（rows0 取这次挂载的）、下一次发布那一判（rows0 + 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadingEntry {
    OfAWritableMount,
    BeforeAPublish,
}

impl ReadingEntry {
    const BOTH: [ReadingEntry; 2] = [ReadingEntry::OfAWritableMount, ReadingEntry::BeforeAPublish];

    fn name(self) -> &'static str {
        match self {
            ReadingEntry::OfAWritableMount => "writable_mount",
            ReadingEntry::BeforeAPublish => "before_a_publish",
        }
    }

    /// D28 已定项 3：「发布路径用 rows0 + 1」，可写挂载自己判时照旧按这次的 rows0。
    fn instance_rows_of_the_switch_reserve(self, instance_rows_of_this_mount: u64) -> u64 {
        match self {
            ReadingEntry::OfAWritableMount => instance_rows_of_this_mount,
            ReadingEntry::BeforeAPublish => instance_rows_of_this_mount + 1,
        }
    }
}

/// 一块盘上读数的各项，全按字节；全池三项已摊成「副本之和 ÷ 副本数」。
#[derive(Clone, Debug, PartialEq, Eq)]
struct DeviceTerms {
    device: u32,
    capacity_bytes: u64,
    allocated_bytes: u64,
    unreclaimable_bytes: u64,
    mount_time_commitment_bytes: u64,
    abandoned_root_exclusive_bytes: u64,
    pending_delete_share_bytes: u64,
    committed_reservation_share_bytes: u64,
    checkpoint_reserve_pool_share_bytes: u64,
    ring_window_bytes: u64,
    available_bytes: i128,
}

impl DeviceTerms {
    /// 准入预留的六项（名字，字节），次序固定。
    fn admission_reserve_items(&self) -> [(&'static str, u64); 6] {
        [
            ("mount_time_commitment", self.mount_time_commitment_bytes),
            (
                "abandoned_root_exclusive",
                self.abandoned_root_exclusive_bytes,
            ),
            ("pending_delete_share", self.pending_delete_share_bytes),
            (
                "committed_reservation_share",
                self.committed_reservation_share_bytes,
            ),
            (
                "checkpoint_reserve_pool_share",
                self.checkpoint_reserve_pool_share_bytes,
            ),
            ("ring_window", self.ring_window_bytes),
        ]
    }

    /// 准入预留(d) = 六项之和（登记 X15）。
    fn admission_reserve_bytes(&self) -> u64 {
        self.mount_time_commitment_bytes
            + self.abandoned_root_exclusive_bytes
            + self.pending_delete_share_bytes
            + self.committed_reservation_share_bytes
            + self.checkpoint_reserve_pool_share_bytes
            + self.ring_window_bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReadingSnapshot {
    entry: ReadingEntry,
    /// 读数里切换预留按哪个行数算（这次挂载的 rows0，或它加一）。
    instance_rows_of_the_switch_reserve: u64,
    replicas: u64,
    checkpoint_reserve_pool_bytes: u64,
    pending_delete_bytes: u64,
    committed_reservation_bytes: u64,
    /// 要接的那一版有树时它的退役树内部层数（环窗口 W 里的 H_退役）；树表 0 条的一版是 `None`。
    retirement_tree_internal_levels: Option<u64>,
    devices: Vec<DeviceTerms>,
}

fn snapshot_of_a_reading(
    entry: ReadingEntry,
    reading: &AdmissionReading,
    instance_rows_of_this_mount: u64,
    retirement_tree_internal_levels: Option<u64>,
) -> ReadingSnapshot {
    let replicas = reading.replicas();
    let pool_wide = reading.pool_wide();
    let available_by_device = reading.available_on_each_device();
    let devices = reading
        .per_device()
        .iter()
        .zip(available_by_device.iter())
        .map(|(terms, (available_device, available))| {
            assert_eq!(
                terms.device, *available_device,
                "可用(d) 与逐盘各项按同一个盘序"
            );
            DeviceTerms {
                device: terms.device.0,
                capacity_bytes: terms.capacity.0,
                allocated_bytes: terms.allocated.0,
                unreclaimable_bytes: terms.unreclaimable.0,
                mount_time_commitment_bytes: terms.mount_time_commitment.0,
                abandoned_root_exclusive_bytes: terms
                    .isolated_for_abandoned_roots_and_unapplied_journal_records
                    .0,
                pending_delete_share_bytes: pool_wide
                    .pending_delete
                    .share_of_one_device(replicas)
                    .0,
                committed_reservation_share_bytes: pool_wide
                    .committed_reservation
                    .share_of_one_device(replicas)
                    .0,
                checkpoint_reserve_pool_share_bytes: pool_wide
                    .checkpoint_reserve_pool
                    .share_of_one_device(replicas)
                    .0,
                ring_window_bytes: terms.ring_window.0,
                available_bytes: available.0,
            }
        })
        .collect();
    ReadingSnapshot {
        entry,
        instance_rows_of_the_switch_reserve: entry
            .instance_rows_of_the_switch_reserve(instance_rows_of_this_mount),
        replicas: replicas.replicas(),
        checkpoint_reserve_pool_bytes: pool_wide.checkpoint_reserve_pool.0,
        pending_delete_bytes: pool_wide.pending_delete.0,
        committed_reservation_bytes: pool_wide.committed_reservation.0,
        retirement_tree_internal_levels,
        devices,
    }
}

/// 两个入口各取一份读数：读数在 `mount_writable` 交回那一刻取。
fn both_readings_of_a_mount(
    mounted: &Mounted,
    slots_per_region: RootRingSlotsPerRegion,
) -> Vec<ReadingSnapshot> {
    let version_of_trees = mounted.current.version_of_trees();
    let instance_rows_of_this_mount = mounted
        .allocator
        .instance_rows_after_this_mounts_row_publish();
    let retirement_tree_internal_levels =
        version_of_trees.map(|version| version.retirement_tree_internal_levels());
    let of_a_writable_mount = admission_reading_of_a_writable_mount(
        &mounted.allocator,
        version_of_trees,
        slots_per_region,
    );
    let before_a_publish = admission_reading_before_a_publish(&mounted.allocator, version_of_trees);
    vec![
        snapshot_of_a_reading(
            ReadingEntry::OfAWritableMount,
            &of_a_writable_mount,
            instance_rows_of_this_mount,
            retirement_tree_internal_levels,
        ),
        snapshot_of_a_reading(
            ReadingEntry::BeforeAPublish,
            &before_a_publish,
            instance_rows_of_this_mount,
            retirement_tree_internal_levels,
        ),
    ]
}

/// R4c（登记第六节）：准入预留六项各占总和的份额，最大一项份额 ≥ 50% 记「主导」，否则「没有一项过半」。
#[derive(Clone, Debug, PartialEq, Eq)]
enum Dominance {
    Dominant {
        item: &'static str,
        share_permille: u64,
    },
    NoItemAboveHalf {
        largest_item: &'static str,
        share_permille: u64,
    },
}

impl Dominance {
    fn is_dominant(&self) -> bool {
        match self {
            Dominance::Dominant { .. } => true,
            Dominance::NoItemAboveHalf { .. } => false,
        }
    }

    fn verdict_text(&self) -> String {
        match self {
            Dominance::Dominant {
                item,
                share_permille,
            } => {
                format!("dominant:{item}:{share_permille}permille")
            }
            Dominance::NoItemAboveHalf {
                largest_item,
                share_permille,
            } => format!("no_item_above_half:{largest_item}:{share_permille}permille"),
        }
    }
}

fn share_permille(part: u64, total: u64) -> u64 {
    if total == 0 {
        return 0;
    }
    u64::try_from(u128::from(part) * 1000 / u128::from(total)).expect("千分比不超过 1000")
}

/// R4c 的门槛：份额 ≥ 50%（登记第六节），写成千分比。
const DOMINANCE_THRESHOLD_PERMILLE: u64 = 500;

fn dominance_of(items: &[(&'static str, u64)]) -> Dominance {
    dominance_with_threshold(items, DOMINANCE_THRESHOLD_PERMILLE)
}

fn dominance_with_threshold(items: &[(&'static str, u64)], threshold_permille: u64) -> Dominance {
    let total: u64 = items.iter().map(|(_, bytes)| *bytes).sum();
    let (largest_item, largest_bytes) = items
        .iter()
        .copied()
        .max_by_key(|(_, bytes)| *bytes)
        .expect("六项不为空");
    let share_permille = share_permille(largest_bytes, total);
    if total > 0
        && 1000 * u128::from(largest_bytes) >= u128::from(threshold_permille) * u128::from(total)
    {
        Dominance::Dominant {
            item: largest_item,
            share_permille,
        }
    } else {
        Dominance::NoItemAboveHalf {
            largest_item,
            share_permille,
        }
    }
}

/// 判别力自证（登记第八节 ③）：把门槛挪到最大一项份额的另一侧——主导的挪到刚好高过最大份额，没过半的挪到最大份额的整千分点——判定必须转。
fn moved_threshold_permille(items: &[(&'static str, u64)]) -> u64 {
    let total: u64 = items.iter().map(|(_, bytes)| *bytes).sum();
    let largest_bytes = items
        .iter()
        .map(|(_, bytes)| *bytes)
        .max()
        .expect("六项不为空");
    let largest_share_permille = share_permille(largest_bytes, total);
    if dominance_of(items).is_dominant() {
        largest_share_permille + 1
    } else {
        largest_share_permille
    }
}

fn dominance_flips_when_the_threshold_is_moved(items: &[(&'static str, u64)]) -> bool {
    dominance_of(items).is_dominant()
        != dominance_with_threshold(items, moved_threshold_permille(items)).is_dominant()
}

/// 对一份读数的 K30、K31、K32 与 G7（登记 7.1、第十节）。字段为 `false` 的格都在判决行点名。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReadingChecks {
    /// K30：待删占用 = 0。
    pending_delete_is_zero: bool,
    /// K31 用到的 c：checkpoint 保留池 ÷ (16384 × 副本数)，除不尽记 `None`。
    checkpoint_cost_blocks_by_the_pool: Option<u64>,
    /// K31 反过来从挂载期承诺量解出的 c（每块盘一个），解不出整数记 `None`。
    checkpoint_cost_blocks_by_the_commitment: Vec<Option<u64>>,
    /// K31：每块盘的挂载期承诺量 = 4 × (32768 × max(1, ⌈(rows + 3) ÷ 369⌉) + 3 × c × 16384)。
    mount_time_commitment_by_the_clauses: bool,
    /// K32：环窗口(d) 与条款句现算的相同（树表 0 条的一版上必须是 0）。
    ring_window_by_the_clauses: bool,
    /// 容量 = 本地几何算出的单元区槽数 × 16384。
    capacity_by_the_local_geometry: bool,
}

impl ReadingChecks {
    fn all_hold(&self) -> bool {
        self.pending_delete_is_zero
            && self.checkpoint_cost_blocks_by_the_pool.is_some()
            && self
                .checkpoint_cost_blocks_by_the_commitment
                .iter()
                .all(|blocks| {
                    blocks.is_some() && *blocks == self.checkpoint_cost_blocks_by_the_pool
                })
            && self.mount_time_commitment_by_the_clauses
            && self.ring_window_by_the_clauses
            && self.capacity_by_the_local_geometry
    }
}

/// 条款句现算的挂载期承诺量，每块盘（D28 已定项 3，c 是 checkpoint 保留池折成的块数）。
fn mount_time_commitment_by_the_clauses(
    instance_rows_of_the_switch_reserve: u64,
    checkpoint_cost_blocks: u64,
) -> u64 {
    (INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1)
        * (INSTANCE_TABLE_PAGE_BYTES
            * instance_table_chain_pages(instance_rows_of_the_switch_reserve)
            + WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH
                * checkpoint_cost_blocks
                * SLOT_BYTES)
}

/// 从承诺量解出 c：先去掉链重写那一半，剩下的除以 (R × 16384)，除不尽记 `None`。
fn checkpoint_cost_blocks_solved_from_the_commitment(
    commitment_bytes: u64,
    instance_rows_of_the_switch_reserve: u64,
) -> Option<u64> {
    let per_switch = commitment_bytes.checked_div(INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1)?;
    if per_switch * (INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1) != commitment_bytes {
        return None;
    }
    let warm_up_bytes = per_switch.checked_sub(
        INSTANCE_TABLE_PAGE_BYTES * instance_table_chain_pages(instance_rows_of_the_switch_reserve),
    )?;
    let unit = WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH * SLOT_BYTES;
    (warm_up_bytes % unit == 0).then_some(warm_up_bytes / unit)
}

fn checks_of_a_reading(
    snapshot: &ReadingSnapshot,
    device_bytes: u64,
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> ReadingChecks {
    let pool_bytes_per_block = SLOT_BYTES * snapshot.replicas;
    let checkpoint_cost_blocks_by_the_pool = (snapshot
        .checkpoint_reserve_pool_bytes
        .is_multiple_of(pool_bytes_per_block))
    .then_some(snapshot.checkpoint_reserve_pool_bytes / pool_bytes_per_block);
    let rows = snapshot.instance_rows_of_the_switch_reserve;
    let ring_window_expected_bytes = match snapshot.retirement_tree_internal_levels {
        None => 0,
        Some(levels) => {
            ring_window_by_the_clauses(device_bytes, journal_ring_bytes, slots_per_region, levels)
                .ring_window_bytes()
        }
    };
    ReadingChecks {
        pending_delete_is_zero: snapshot.pending_delete_bytes == 0
            && snapshot
                .devices
                .iter()
                .all(|terms| terms.pending_delete_share_bytes == 0),
        checkpoint_cost_blocks_by_the_pool,
        checkpoint_cost_blocks_by_the_commitment: snapshot
            .devices
            .iter()
            .map(|terms| {
                checkpoint_cost_blocks_solved_from_the_commitment(
                    terms.mount_time_commitment_bytes,
                    rows,
                )
            })
            .collect(),
        mount_time_commitment_by_the_clauses: checkpoint_cost_blocks_by_the_pool.is_some_and(
            |blocks| {
                snapshot.devices.iter().all(|terms| {
                    terms.mount_time_commitment_bytes
                        == mount_time_commitment_by_the_clauses(rows, blocks)
                })
            },
        ),
        ring_window_by_the_clauses: snapshot
            .devices
            .iter()
            .all(|terms| terms.ring_window_bytes == ring_window_expected_bytes),
        capacity_by_the_local_geometry: snapshot.devices.iter().all(|terms| {
            terms.capacity_bytes == unit_area_slots(device_bytes, journal_ring_bytes) * SLOT_BYTES
        }),
    }
}

// ============================================================================================
// X14：一次尝试 = mkfs → 可写挂载 → 首个文件 → 正常卸载 → 可写挂载；每一步成不成、挂载的准入怎么判的、读数。
// ============================================================================================

/// 可写挂载的准入结局（`MountSpaceAdmission` 的四个成员，名字写成报得出的字符串）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdmissionClass {
    AdmittedBeforeAcquisition,
    AdmittedAfterTheFloorRaises,
    StillShortAfterTheFloorRaises,
    NotJudgedByTheTestOnlySwitch,
}

impl AdmissionClass {
    fn of_the_mount(space_admission: &MountSpaceAdmission) -> Self {
        match space_admission {
            MountSpaceAdmission::AdmittedBeforeAcquisition => {
                AdmissionClass::AdmittedBeforeAcquisition
            }
            MountSpaceAdmission::AdmittedAfterTheFloorRaises { .. } => {
                AdmissionClass::AdmittedAfterTheFloorRaises
            }
            MountSpaceAdmission::StillShortAfterTheFloorRaises { .. } => {
                AdmissionClass::StillShortAfterTheFloorRaises
            }
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => {
                AdmissionClass::NotJudgedByTheTestOnlySwitch
            }
        }
    }

    fn name(self) -> &'static str {
        match self {
            AdmissionClass::AdmittedBeforeAcquisition => "AdmittedBeforeAcquisition",
            AdmissionClass::AdmittedAfterTheFloorRaises => "AdmittedAfterTheFloorRaises",
            AdmissionClass::StillShortAfterTheFloorRaises => "StillShortAfterTheFloorRaises",
            AdmissionClass::NotJudgedByTheTestOnlySwitch => "NotJudgedByTheTestOnlySwitch",
        }
    }

    /// X14：「成」只认放行的两种；推满仍不够不算成（另记），没判的也不算（它只在 PC-准入 里出现）。
    fn counts_as_admitted(self) -> bool {
        match self {
            AdmissionClass::AdmittedBeforeAcquisition => true,
            AdmissionClass::AdmittedAfterTheFloorRaises => true,
            AdmissionClass::StillShortAfterTheFloorRaises => false,
            AdmissionClass::NotJudgedByTheTestOnlySwitch => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StageOutcome<Success> {
    Succeeded(Success),
    /// 这一步报错（变体名）或 panic。
    Failed(String),
    /// 前一步没成，没走到。
    NotReached,
}

impl<Success> StageOutcome<Success> {
    fn text(&self, describe: impl Fn(&Success) -> String) -> String {
        match self {
            StageOutcome::Succeeded(success) => describe(success),
            StageOutcome::Failed(reason) => format!("failed:{reason}"),
            StageOutcome::NotReached => "not_reached".to_string(),
        }
    }
}

/// 一次可写挂载做成之后留下的东西：准入结局、两份读数、这次挂载的读写计数、盘上实例表链的行数、此刻两块盘上可用(d) 的最小值。
#[derive(Clone, Debug, PartialEq, Eq)]
struct MountReport {
    class: AdmissionClass,
    readings: Vec<ReadingSnapshot>,
    instance_rows_by_the_allocator: u64,
    instance_rows_read_back_from_the_disk: Option<u64>,
    mount_tallies: Vec<DeviceTally>,
    available_minimum_bytes: i128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Trial {
    device_bytes: u64,
    make_filesystem: StageOutcome<()>,
    empty_mount: StageOutcome<MountReport>,
    first_file: StageOutcome<()>,
    unmount: StageOutcome<()>,
    file_mount: StageOutcome<MountReport>,
}

/// 搜索只看这几个布尔（登记 X14 的三个最小盘与「挂上但推满仍不够」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TrialFlags {
    device_bytes: u64,
    make_filesystem_succeeded: bool,
    empty_mount_admitted: bool,
    file_mount_admitted: bool,
    empty_mount_still_short: bool,
    file_mount_still_short: bool,
}

impl Trial {
    fn flags(&self) -> TrialFlags {
        let admitted = |outcome: &StageOutcome<MountReport>| matches!(outcome, StageOutcome::Succeeded(report) if report.class.counts_as_admitted());
        let still_short = |outcome: &StageOutcome<MountReport>| {
            matches!(outcome, StageOutcome::Succeeded(report)
                if report.class == AdmissionClass::StillShortAfterTheFloorRaises)
        };
        let file_chain_succeeded = matches!(self.first_file, StageOutcome::Succeeded(()))
            && matches!(self.unmount, StageOutcome::Succeeded(()));
        TrialFlags {
            device_bytes: self.device_bytes,
            make_filesystem_succeeded: matches!(self.make_filesystem, StageOutcome::Succeeded(())),
            empty_mount_admitted: admitted(&self.empty_mount),
            file_mount_admitted: admitted(&self.empty_mount)
                && file_chain_succeeded
                && admitted(&self.file_mount),
            empty_mount_still_short: still_short(&self.empty_mount),
            file_mount_still_short: file_chain_succeeded && still_short(&self.file_mount),
        }
    }
}

fn report_of_a_mount(
    mounted: &Mounted,
    parameters: &MakeFilesystemParameters,
    pool: &CountedPool,
    tallies_before_the_mount: &[DeviceTally],
) -> MountReport {
    let slots_per_region = parameters.geometry.root_ring_slots_per_region;
    let readings = both_readings_of_a_mount(mounted, slots_per_region);
    let available_minimum_bytes = readings[0]
        .devices
        .iter()
        .map(|terms| terms.available_bytes)
        .min()
        .expect("两块盘");
    let instance_rows_read_back_from_the_disk =
        instance_table_chain_of_root(pool, mounted.current.root())
            .ok()
            .map(|chain| u64::try_from(chain.records.rows.len()).expect("行数装得进 u64"));
    MountReport {
        class: AdmissionClass::of_the_mount(&mounted.output.space_admission),
        readings,
        instance_rows_by_the_allocator: mounted
            .allocator
            .instance_rows_after_this_mounts_row_publish(),
        instance_rows_read_back_from_the_disk,
        mount_tallies: tally_difference(pool, tallies_before_the_mount),
        available_minimum_bytes,
    }
}

/// panic 也记成这一步没成（极小的盘上可能走到实现里的断言），文字只留 panic 消息头一行。
fn guarded<Success>(step: impl FnOnce() -> Result<Success, String>) -> Result<Success, String> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(step)) {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("panic_without_message");
            Err(format!(
                "panicked:{}",
                message.lines().next().unwrap_or("").replace(' ', "_")
            ))
        }
    }
}

/// 一次尝试：同一对稀疏盘上依次 mkfs、可写挂载、首个文件、正常卸载、可写挂载；某一步没成，后面的记「没走到」。
fn run_trial(device_bytes: u64, journal_ring_bytes: u64, slots_per_region: u64) -> Trial {
    let parameters = make_filesystem_parameters(journal_ring_bytes, slots_per_region);
    let mut pool = counted_pool(device_bytes, journal_ring_bytes, slots_per_region);
    let mut trial = Trial {
        device_bytes,
        make_filesystem: StageOutcome::NotReached,
        empty_mount: StageOutcome::NotReached,
        first_file: StageOutcome::NotReached,
        unmount: StageOutcome::NotReached,
        file_mount: StageOutcome::NotReached,
    };
    let made = guarded(|| {
        make_filesystem(&parameters, &mut pool)
            .map(|_| ())
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    });
    match made {
        Ok(()) => trial.make_filesystem = StageOutcome::Succeeded(()),
        Err(reason) => {
            trial.make_filesystem = StageOutcome::Failed(reason);
            return trial;
        }
    }
    let tallies_before_the_first_mount = tallies_of(&pool);
    let first_mount = guarded(|| {
        mount_writable(&parameters, &mut pool)
            .map(|mounted| {
                let report = report_of_a_mount(
                    &mounted,
                    &parameters,
                    &pool,
                    &tallies_before_the_first_mount,
                );
                (mounted, report)
            })
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    });
    let (mounted, empty_report) = match first_mount {
        Ok(pair) => pair,
        Err(reason) => {
            trial.empty_mount = StageOutcome::Failed(reason);
            return trial;
        }
    };
    trial.empty_mount = StageOutcome::Succeeded(empty_report);
    if !trial.flags().empty_mount_admitted {
        return trial;
    }
    let content = first_file_content();
    let published = guarded(|| {
        let Mounted {
            output,
            mut allocator,
            current,
        } = mounted;
        let mut writer = PoolWriter::new(&parameters, pool.as_mut_slice());
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            output.instance,
            current.record_bytes(),
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))?;
        Ok((allocator, PoolVersion::of_a_version_of_trees(first)))
    });
    let (mut allocator, mut current) = match published {
        Ok(pair) => pair,
        Err(reason) => {
            trial.first_file = StageOutcome::Failed(reason);
            return trial;
        }
    };
    trial.first_file = StageOutcome::Succeeded(());
    let unmounted = guarded(|| {
        unmount(
            &parameters,
            &mut pool,
            &mut allocator,
            &mut current,
            ShadowLedger::On,
        )
        .map(|_| ())
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    });
    match unmounted {
        Ok(()) => trial.unmount = StageOutcome::Succeeded(()),
        Err(reason) => {
            trial.unmount = StageOutcome::Failed(reason);
            return trial;
        }
    }
    let tallies_before_the_second_mount = tallies_of(&pool);
    let second_mount = guarded(|| {
        mount_writable(&parameters, &mut pool)
            .map(|remounted| {
                report_of_a_mount(
                    &remounted,
                    &parameters,
                    &pool,
                    &tallies_before_the_second_mount,
                )
            })
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    });
    trial.file_mount = match second_mount {
        Ok(report) => StageOutcome::Succeeded(report),
        Err(reason) => StageOutcome::Failed(reason),
    };
    trial
}

// ============================================================================================
// X14 的搜索：C 从 4 MiB 起按 1 MiB 步进；三个最小盘、「挂上但推满仍不够」的最小盘、找到之后再核 C + 1 … C + 16 MiB 的单调核。
// ============================================================================================

const SEARCH_START_DEVICE_BYTES: u64 = 4 * MEBIBYTE;
const SEARCH_STEP_BYTES: u64 = MEBIBYTE;
/// 找到之后再核几步：C + 1 … C + 16 MiB。
const MONOTONICITY_WINDOW_BYTES: u64 = 16 * MEBIBYTE;
/// 搜索的上沿：到这里还没找全，记「≤ 上沿 未找到」，不把上沿当极值。
const SEARCH_CEILING_DEVICE_BYTES: u64 = 1024 * MEBIBYTE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Milestone {
    MakeFilesystemSucceeds,
    EmptyMountAdmitted,
    FileMountAdmitted,
    EmptyMountStillShort,
    FileMountStillShort,
}

impl Milestone {
    const THREE_MINIMUM_DISKS: [Milestone; 3] = [
        Milestone::MakeFilesystemSucceeds,
        Milestone::EmptyMountAdmitted,
        Milestone::FileMountAdmitted,
    ];

    fn name(self) -> &'static str {
        match self {
            Milestone::MakeFilesystemSucceeds => "make_filesystem",
            Milestone::EmptyMountAdmitted => "empty_mount",
            Milestone::FileMountAdmitted => "file_mount",
            Milestone::EmptyMountStillShort => "empty_mount_still_short",
            Milestone::FileMountStillShort => "file_mount_still_short",
        }
    }

    fn holds_in(self, trial: &TrialFlags) -> bool {
        match self {
            Milestone::MakeFilesystemSucceeds => trial.make_filesystem_succeeded,
            Milestone::EmptyMountAdmitted => trial.empty_mount_admitted,
            Milestone::FileMountAdmitted => trial.file_mount_admitted,
            Milestone::EmptyMountStillShort => trial.empty_mount_still_short,
            Milestone::FileMountStillShort => trial.file_mount_still_short,
        }
    }
}

/// 第一个让 `milestone` 成立的盘字节数。
fn minimum_device_bytes(trials: &[TrialFlags], milestone: Milestone) -> Option<u64> {
    trials
        .iter()
        .find(|trial| milestone.holds_in(trial))
        .map(|trial| trial.device_bytes)
}

/// 单调核（登记 X14、失败条款 G4）：最小盘之后 C + 1 … C + 16 MiB 里每一个都得成；不成的点列全。
#[derive(Clone, Debug, PartialEq, Eq)]
struct Monotonicity {
    minimum_device_bytes: Option<u64>,
    /// 窗口里没成的点。
    failures_in_the_window: Vec<u64>,
    /// 窗口里应当有尝试而没有的点（搜索没跑到那里）。
    window_points_never_tried: Vec<u64>,
}

impl Monotonicity {
    fn is_monotone(&self) -> bool {
        self.minimum_device_bytes.is_some()
            && self.failures_in_the_window.is_empty()
            && self.window_points_never_tried.is_empty()
    }
}

fn monotonicity_of(trials: &[TrialFlags], milestone: Milestone) -> Monotonicity {
    let minimum = minimum_device_bytes(trials, milestone);
    let mut failures_in_the_window = Vec::new();
    let mut window_points_never_tried = Vec::new();
    if let Some(minimum_bytes) = minimum {
        let mut point = minimum_bytes + SEARCH_STEP_BYTES;
        while point <= minimum_bytes + MONOTONICITY_WINDOW_BYTES {
            match trials.iter().find(|trial| trial.device_bytes == point) {
                Some(trial) if milestone.holds_in(trial) => {}
                Some(_) => failures_in_the_window.push(point),
                None => window_points_never_tried.push(point),
            }
            point += SEARCH_STEP_BYTES;
        }
    }
    Monotonicity {
        minimum_device_bytes: minimum,
        failures_in_the_window,
        window_points_never_tried,
    }
}

/// 搜索什么时候停：三个最小盘都找到，并且已经试到其中最大的那个再加 16 MiB；或者到了上沿。
fn search_may_stop(trials: &[TrialFlags]) -> bool {
    let Some(last_tried) = trials.last().map(|trial| trial.device_bytes) else {
        return false;
    };
    if last_tried >= SEARCH_CEILING_DEVICE_BYTES {
        return true;
    }
    let minima: Option<Vec<u64>> = Milestone::THREE_MINIMUM_DISKS
        .iter()
        .map(|milestone| minimum_device_bytes(trials, *milestone))
        .collect();
    minima.is_some_and(|found| {
        let largest = found.into_iter().max().expect("三个最小盘");
        last_tried >= largest + MONOTONICITY_WINDOW_BYTES
    })
}

fn search_small_disks(journal_ring_bytes: u64, slots_per_region: u64) -> Vec<Trial> {
    let mut trials: Vec<Trial> = Vec::new();
    let mut flags: Vec<TrialFlags> = Vec::new();
    let mut device_bytes = SEARCH_START_DEVICE_BYTES;
    loop {
        let trial = run_trial(device_bytes, journal_ring_bytes, slots_per_region);
        flags.push(trial.flags());
        trials.push(trial);
        if search_may_stop(&flags) {
            return trials;
        }
        device_bytes += SEARCH_STEP_BYTES;
    }
}

// ============================================================================================
// 产物行
// ============================================================================================

/// 产物行写到哪：第一段与合并直接打到标准输出；第二、三段的一片先收在内存里，整片做完再原子地写成片文件（登记 E32、5.5「续跑」）。
enum EmitterSink {
    StandardOutput,
    CollectedLines(Vec<String>),
}

struct Emitter {
    emitted: u64,
    sink: EmitterSink,
}

impl Emitter {
    fn to_standard_output() -> Self {
        Emitter {
            emitted: 0,
            sink: EmitterSink::StandardOutput,
        }
    }

    fn collecting_lines() -> Self {
        Emitter {
            emitted: 0,
            sink: EmitterSink::CollectedLines(Vec::new()),
        }
    }

    fn write_line(&mut self, line: String) {
        match &mut self.sink {
            EmitterSink::StandardOutput => println!("{line}"),
            EmitterSink::CollectedLines(lines) => lines.push(line),
        }
    }

    fn emit(&mut self, body: &str) {
        self.emitted += 1;
        self.write_line(format!("E7RESULT {body}"));
    }

    fn finish(&mut self) {
        self.emitted += 1;
        let line = format!("E7RESULT name=done emitted={}", self.emitted);
        self.write_line(line);
    }

    /// 收在内存里的行（打到标准输出的那一种交回空表）。
    fn into_collected_lines(self) -> Vec<String> {
        match self.sink {
            EmitterSink::StandardOutput => Vec::new(),
            EmitterSink::CollectedLines(lines) => lines,
        }
    }
}

fn mebibytes_text(bytes: u64) -> String {
    assert_eq!(bytes % MEBIBYTE, 0, "盘字节数落在 MiB 整数倍上");
    (bytes / MEBIBYTE).to_string()
}

fn optional_mebibytes_text(bytes: Option<u64>) -> String {
    bytes.map_or_else(|| "not_found".to_string(), mebibytes_text)
}

fn list_of_mebibytes_text(points: &[u64]) -> String {
    if points.is_empty() {
        return "none".to_string();
    }
    points
        .iter()
        .map(|bytes| mebibytes_text(*bytes))
        .collect::<Vec<_>>()
        .join(",")
}

fn optional_number_text(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| number.to_string())
}

fn mount_text(outcome: &StageOutcome<MountReport>) -> String {
    outcome.text(|report| report.class.name().to_string())
}

fn available_text(outcome: &StageOutcome<MountReport>) -> String {
    match outcome {
        StageOutcome::Succeeded(report) => report.available_minimum_bytes.to_string(),
        StageOutcome::Failed(_) | StageOutcome::NotReached => "none".to_string(),
    }
}

fn tally_text(reads_and_writes: &ReadsAndWrites) -> String {
    format!(
        "r{}/{},w{}/{}",
        reads_and_writes.reads.calls,
        reads_and_writes.reads.bytes,
        reads_and_writes.writes.calls,
        reads_and_writes.writes.bytes
    )
}

fn emit_trial(emitter: &mut Emitter, item: &str, trial: &Trial) {
    emitter.emit(&format!(
        "name=h4_trial item={item} device_mebibytes={} make_filesystem={} empty_mount={} first_file={} unmount={} file_mount={} empty_available_minimum_bytes={} file_available_minimum_bytes={}",
        mebibytes_text(trial.device_bytes),
        trial.make_filesystem.text(|()| "ok".to_string()),
        mount_text(&trial.empty_mount),
        trial.first_file.text(|()| "ok".to_string()),
        trial.unmount.text(|()| "ok".to_string()),
        mount_text(&trial.file_mount),
        available_text(&trial.empty_mount),
        available_text(&trial.file_mount),
    ));
}

/// 一份读数所在的位置：哪个项、哪种状态、哪块盘（字节数）、哪个几何。
struct ReadingLabel<'label> {
    item: &'label str,
    state: StateKind,
    device_bytes: u64,
    journal_ring_bytes: u64,
    slots_per_region: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StateKind {
    Empty,
    File,
}

impl StateKind {
    fn name(self) -> &'static str {
        match self {
            StateKind::Empty => "empty",
            StateKind::File => "file",
        }
    }
}

fn parts_per_million(part: u64, total: u64) -> u64 {
    if total == 0 {
        return 0;
    }
    u64::try_from(u128::from(part) * 1_000_000 / u128::from(total)).expect("百万分比装得进 u64")
}

/// 一份读数的各行：读数头、逐盘各项（含准入预留、比例、六项份额、主导）、对 K30–K32 与 G7 的核。返回盘 0 的主导判定。
fn emit_reading(
    emitter: &mut Emitter,
    label: &ReadingLabel<'_>,
    report: &MountReport,
    snapshot: &ReadingSnapshot,
) -> (Dominance, [(&'static str, u64); 6]) {
    let item = label.item;
    let state = label.state.name();
    let entry = snapshot.entry.name();
    emitter.emit(&format!(
        "name=h4_reading item={item} state={state} entry={entry} device_mebibytes={} class={} instance_rows_of_the_switch_reserve={} replicas={} checkpoint_reserve_pool_bytes={} pending_delete_bytes={} committed_reservation_bytes={} retirement_tree_internal_levels={}",
        mebibytes_text(label.device_bytes),
        report.class.name(),
        snapshot.instance_rows_of_the_switch_reserve,
        snapshot.replicas,
        snapshot.checkpoint_reserve_pool_bytes,
        snapshot.pending_delete_bytes,
        snapshot.committed_reservation_bytes,
        optional_number_text(snapshot.retirement_tree_internal_levels),
    ));
    let mut dominance_of_the_first_device = None;
    for terms in &snapshot.devices {
        let items = terms.admission_reserve_items();
        let reserve_bytes = terms.admission_reserve_bytes();
        let dominance = dominance_of(&items);
        let shares = items
            .iter()
            .map(|(name, bytes)| {
                format!(
                    "share_permille_{name}={}",
                    share_permille(*bytes, reserve_bytes)
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        emitter.emit(&format!(
            "name=h4_reading_device item={item} state={state} entry={entry} device={} capacity_bytes={} allocated_bytes={} unreclaimable_bytes={} mount_time_commitment_bytes={} abandoned_root_exclusive_bytes={} pending_delete_share_bytes={} committed_reservation_share_bytes={} checkpoint_reserve_pool_share_bytes={} ring_window_bytes={} admission_reserve_bytes={} available_bytes={} reserve_over_capacity_parts_per_million={} {shares} dominance={}",
            terms.device,
            terms.capacity_bytes,
            terms.allocated_bytes,
            terms.unreclaimable_bytes,
            terms.mount_time_commitment_bytes,
            terms.abandoned_root_exclusive_bytes,
            terms.pending_delete_share_bytes,
            terms.committed_reservation_share_bytes,
            terms.checkpoint_reserve_pool_share_bytes,
            terms.ring_window_bytes,
            reserve_bytes,
            terms.available_bytes,
            parts_per_million(reserve_bytes, terms.capacity_bytes),
            dominance.verdict_text(),
        ));
        if dominance_of_the_first_device.is_none() {
            dominance_of_the_first_device = Some((dominance, items));
        }
    }
    let checks = checks_of_a_reading(
        snapshot,
        label.device_bytes,
        label.journal_ring_bytes,
        label.slots_per_region,
    );
    let ring_window_if_deducted_bytes = ring_window_by_the_clauses(
        label.device_bytes,
        label.journal_ring_bytes,
        label.slots_per_region,
        snapshot.retirement_tree_internal_levels.unwrap_or(0),
    )
    .ring_window_bytes();
    emitter.emit(&format!(
        "name=h4_reading_checks item={item} state={state} entry={entry} pending_delete_is_zero={} checkpoint_cost_blocks_by_the_pool={} checkpoint_cost_blocks_by_the_commitment={} mount_time_commitment_matches_the_clauses={} ring_window_matches_the_clauses={} ring_window_by_the_clauses_if_deducted_bytes={ring_window_if_deducted_bytes} capacity_by_the_local_geometry={} all_hold={}",
        checks.pending_delete_is_zero,
        optional_number_text(checks.checkpoint_cost_blocks_by_the_pool),
        checks
            .checkpoint_cost_blocks_by_the_commitment
            .iter()
            .map(|blocks| optional_number_text(*blocks))
            .collect::<Vec<_>>()
            .join(","),
        checks.mount_time_commitment_by_the_clauses,
        checks.ring_window_by_the_clauses,
        checks.capacity_by_the_local_geometry,
        checks.all_hold(),
    ));
    dominance_of_the_first_device.expect("两块盘")
}

fn emit_mount_tally(emitter: &mut Emitter, item: &str, state: &str, report: &MountReport) {
    for (device, tally) in report.mount_tallies.iter().enumerate() {
        emitter.emit(&format!(
            "name=h4_mount_tally item={item} state={state} device={device} system_configuration={} root_ring={} journal_ring={} unit_area={}",
            tally_text(&tally.system_configuration),
            tally_text(&tally.root_ring),
            tally_text(&tally.journal_ring),
            tally_text(&tally.unit_area),
        ));
    }
    emitter.emit(&format!(
        "name=h4_instance_rows item={item} state={state} rows_by_the_allocator={} rows_read_back_from_the_disk={}",
        report.instance_rows_by_the_allocator,
        optional_number_text(report.instance_rows_read_back_from_the_disk),
    ));
}

/// 一个入口的读数：入口、主导判定、准入预留六项（项名，字节）。
type DominanceOfAnEntry = (ReadingEntry, Dominance, [(&'static str, u64); 6]);

/// 一种状态（空池 / 有文件）上的全部读数行，返回每个入口的主导判定与各份读数的核是否全过。
struct StateOutcome {
    dominance_by_entry: Vec<DominanceOfAnEntry>,
    checks_hold: bool,
}

fn emit_state(
    emitter: &mut Emitter,
    label: &ReadingLabel<'_>,
    report: &MountReport,
) -> StateOutcome {
    let mut dominance_by_entry = Vec::new();
    let mut checks_hold = true;
    for snapshot in &report.readings {
        let (dominance, reserve_items) = emit_reading(emitter, label, report, snapshot);
        let checks = checks_of_a_reading(
            snapshot,
            label.device_bytes,
            label.journal_ring_bytes,
            label.slots_per_region,
        );
        checks_hold &= checks.all_hold();
        dominance_by_entry.push((snapshot.entry, dominance, reserve_items));
    }
    emit_mount_tally(emitter, label.item, label.state.name(), report);
    StateOutcome {
        dominance_by_entry,
        checks_hold,
    }
}

/// 一个项跑完留给汇总的东西：各状态各入口的主导判定、设备有效性的红数（T2、T3、PC-堆，决定退出码）。
struct ItemOutcome {
    dominance_points: Vec<DominancePoint>,
    invalid_device_conditions: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DominancePoint {
    item: String,
    state: &'static str,
    entry: ReadingEntry,
    dominance: Dominance,
    reserve_items: [(&'static str, u64); 6],
}

/// 要读数的一种状态与它所在的那次尝试（空池读第一次挂载，有文件读第二次挂载）；尝试没有（最小盘没找到）就是 `None`。
struct StateToRead<'trial> {
    state: StateKind,
    trial: Option<&'trial Trial>,
}

fn emit_states_of_a_trial(
    emitter: &mut Emitter,
    item: &str,
    journal_ring_bytes: u64,
    slots_per_region: u64,
    states: [StateToRead<'_>; 2],
) -> (Vec<DominancePoint>, bool) {
    let mut points = Vec::new();
    let mut checks_hold = true;
    for StateToRead { state, trial } in states {
        let state_name = state.name();
        let Some(trial) = trial else {
            emitter.emit(&format!(
                "name=h4_reading item={item} state={state_name} reading=not_run reason=minimum_disk_not_found"
            ));
            continue;
        };
        let outcome = match state {
            StateKind::Empty => &trial.empty_mount,
            StateKind::File => &trial.file_mount,
        };
        let StageOutcome::Succeeded(report) = outcome else {
            emitter.emit(&format!(
                "name=h4_reading item={item} state={state_name} reading=not_run reason=mount_did_not_succeed"
            ));
            continue;
        };
        let label = ReadingLabel {
            item,
            state,
            device_bytes: trial.device_bytes,
            journal_ring_bytes,
            slots_per_region,
        };
        let state_outcome = emit_state(emitter, &label, report);
        checks_hold &= state_outcome.checks_hold;
        for (entry, dominance, reserve_items) in state_outcome.dominance_by_entry {
            points.push(DominancePoint {
                item: item.to_string(),
                state: state_name,
                entry,
                dominance,
                reserve_items,
            });
        }
    }
    (points, checks_hold)
}

// ============================================================================================
// E38：本地常量与 `crates/` 回比（T2）；T3：装置现算的几何与锚点脚本输出逐项比（A30、A31、A33）。
// ============================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConstantComparison {
    name: &'static str,
    local: u64,
    implementation: u64,
}

impl ConstantComparison {
    fn is_equal(&self) -> bool {
        self.local == self.implementation
    }
}

fn constants_against_the_implementation() -> Vec<ConstantComparison> {
    let compare = |name, local, implementation| ConstantComparison {
        name,
        local,
        implementation,
    };
    vec![
        compare(
            "instance_rows_per_instance_table_page",
            INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE,
            singlefs_core::admission::INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE,
        ),
        compare(
            "instance_switches_allowed_per_mount",
            INSTANCE_SWITCHES_ALLOWED_PER_MOUNT,
            singlefs_core::admission::INSTANCE_SWITCHES_ALLOWED_PER_MOUNT,
        ),
        compare(
            "warm_up_empty_publishes_at_most_per_instance_switch",
            WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH,
            singlefs_core::admission::WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH,
        ),
        compare(
            "instance_table_page_bytes",
            INSTANCE_TABLE_PAGE_BYTES,
            TransactionUnit::InstanceTable.span_slots() * singlefs_format::SLOT_BYTES,
        ),
        compare(
            "sweep_threshold_placeholder_rows",
            SWEEP_THRESHOLD_PLACEHOLDER_ROWS,
            InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690.0,
        ),
        compare(
            "journal_record_bytes",
            JOURNAL_RECORD_BYTES,
            singlefs_format::JOURNAL_RECORD_BYTES,
        ),
        compare(
            "journal_safety_factor",
            JOURNAL_SAFETY_FACTOR,
            singlefs_format::JOURNAL_SAFETY_FACTOR,
        ),
        compare(
            "journal_ring_default_bytes",
            JOURNAL_RING_DEFAULT_BYTES,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        compare(
            "allocation_record_tree_leaf_slots",
            ALLOCATION_RECORD_TREE_LEAF_SLOTS,
            singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        ),
        compare(
            "allocation_record_tree_internal_fanout",
            ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
            singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
        ),
        compare(
            "deletion_batch_data_units",
            DELETION_BATCH_DATA_UNITS,
            singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
        ),
        compare(
            "unit_area_start_slot_at_the_default_ring",
            UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        compare(
            "slots_per_region_at_make_filesystem",
            SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
            RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
        ),
        compare("slot_bytes", SLOT_BYTES, singlefs_format::SLOT_BYTES),
        compare(
            "journal_ring_start_slot",
            JOURNAL_RING_START_SLOT,
            singlefs_format::JOURNAL_RING_START_SLOT,
        ),
        compare(
            "root_ring_regions",
            ROOT_RING_REGIONS,
            singlefs_format::ROOT_RING_REGIONS,
        ),
        compare(
            "root_ring_base_slot",
            ROOT_RING_BASE_SLOT,
            singlefs_format::ROOT_RING_BASE_SLOT,
        ),
        compare(
            "root_ring_chunk_bytes",
            ROOT_RING_CHUNK_BYTES,
            singlefs_format::ROOT_RING_CHUNK_BYTES,
        ),
        compare(
            "root_ring_prime_step",
            ROOT_RING_PRIME_STEP,
            singlefs_format::ROOT_RING_PRIME_STEP,
        ),
        compare(
            "system_configuration_slots_per_device",
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
        ),
        compare(
            "fixed_structure_slot_spacing_bytes",
            u64::from(FIXED_STRUCTURE_SLOT_SPACING_BYTES),
            singlefs_format::FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
        ),
    ]
}

#[derive(Clone, Copy)]
struct RegisteredRingWindowAnchor {
    device_bytes: u64,
    journal_ring_bytes: u64,
    slots_per_region: u64,
    unit_area_first_slot: u64,
    unit_area_slots: u64,
    root_level: u32,
    positions_at_each_level: &'static [u64],
    deletion_publish_slots: u64,
    ring_window_slots: u64,
}

impl RegisteredRingWindowAnchor {
    fn is_reproduced_by(&self, local: &RingWindowByTheClauses) -> bool {
        local.unit_area_first_slot == self.unit_area_first_slot
            && local.unit_area_slots == self.unit_area_slots
            && local.root_level == self.root_level
            && local.positions_at_each_level == self.positions_at_each_level
            && local.deletion_publish_slots == self.deletion_publish_slots
            && local.ring_window_slots == self.ring_window_slots
    }
}

/// 登记 A33：链重写那一半（每块盘）在 rows0 = 1、366、367、370、739 上的字节数，命令一 `H4 chain_bytes` 行原样。
const REGISTERED_CHAIN_REWRITE_BYTES: [(u64, u64); 5] = [
    (1, 131_072),
    (366, 131_072),
    (367, 262_144),
    (370, 262_144),
    (739, 393_216),
];
/// 登记第十三节命令一 `anchors_e183.py` 原样输出里的 15 行 `H4 C=… L=… S=…`（五档容量 × S = 4 / 8 / 16，L = 768 MiB）：
/// （盘字节数，环长，每区槽数，单元区起点槽，单元区槽数，根层级，各层位置数，W，环窗口槽数）。装置现算的与它逐项比（T3）。
const REGISTERED_RING_WINDOW_ANCHORS: [RegisteredRingWindowAnchor; 15] = [
    RegisteredRingWindowAnchor {
        device_bytes: 4294967296,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 211968,
        root_level: 2,
        positions_at_each_level: &[524, 4],
        deletion_publish_slots: 374,
        ring_window_slots: 4862,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4294967296,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 211968,
        root_level: 2,
        positions_at_each_level: &[524, 4],
        deletion_publish_slots: 374,
        ring_window_slots: 9350,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4294967296,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 211968,
        root_level: 2,
        positions_at_each_level: &[524, 4],
        deletion_publish_slots: 374,
        ring_window_slots: 18326,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 68719476736,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 4144128,
        root_level: 2,
        positions_at_each_level: &[10210, 62],
        deletion_publish_slots: 432,
        ring_window_slots: 5616,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 68719476736,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 4144128,
        root_level: 2,
        positions_at_each_level: &[10210, 62],
        deletion_publish_slots: 432,
        ring_window_slots: 10800,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 68719476736,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 4144128,
        root_level: 2,
        positions_at_each_level: &[10210, 62],
        deletion_publish_slots: 432,
        ring_window_slots: 21168,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 1099511627776,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 67058688,
        root_level: 3,
        positions_at_each_level: &[165172, 980, 6],
        deletion_publish_slots: 716,
        ring_window_slots: 9308,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 1099511627776,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 67058688,
        root_level: 3,
        positions_at_each_level: &[165172, 980, 6],
        deletion_publish_slots: 716,
        ring_window_slots: 17900,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 1099511627776,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 67058688,
        root_level: 3,
        positions_at_each_level: &[165172, 980, 6],
        deletion_publish_slots: 716,
        ring_window_slots: 35084,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4398046511104,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 268385280,
        root_level: 3,
        positions_at_each_level: &[661050, 3914, 24],
        deletion_publish_slots: 734,
        ring_window_slots: 9542,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4398046511104,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 268385280,
        root_level: 3,
        positions_at_each_level: &[661050, 3914, 24],
        deletion_publish_slots: 734,
        ring_window_slots: 18350,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 4398046511104,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 268385280,
        root_level: 3,
        positions_at_each_level: &[661050, 3914, 24],
        deletion_publish_slots: 734,
        ring_window_slots: 35966,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 17592184995840,
        journal_ring_bytes: 805306368,
        slots_per_region: 4,
        unit_area_first_slot: 50176,
        unit_area_slots: 1073691584,
        root_level: 3,
        positions_at_each_level: &[2644564, 15650, 94],
        deletion_publish_slots: 804,
        ring_window_slots: 10452,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 17592184995840,
        journal_ring_bytes: 805306368,
        slots_per_region: 8,
        unit_area_first_slot: 50176,
        unit_area_slots: 1073691584,
        root_level: 3,
        positions_at_each_level: &[2644564, 15650, 94],
        deletion_publish_slots: 804,
        ring_window_slots: 20100,
    },
    RegisteredRingWindowAnchor {
        device_bytes: 17592184995840,
        journal_ring_bytes: 805306368,
        slots_per_region: 16,
        unit_area_first_slot: 50176,
        unit_area_slots: 1073691584,
        root_level: 3,
        positions_at_each_level: &[2644564, 15650, 94],
        deletion_publish_slots: 804,
        ring_window_slots: 39396,
    },
];

// ============================================================================================
// 各项的运行：constants、positive-control-heap、anchors、h4-small-*、h4-default-*。
// ============================================================================================

const DEFAULT_DEVICE_BYTES: u64 = 4 * GIBIBYTE;
const SLOTS_PER_REGION_SAMPLING_POINTS: [u64; 3] = [4, 8, 16];
/// 几何敏感性的方向相反的取样点（登记 8.2）：S = 8 上环长 4 MiB。
const ALTERNATIVE_JOURNAL_RING_BYTES: u64 = 4 * MEBIBYTE;
const SMALL_DISK_JOURNAL_RING_BYTES: u64 = MEBIBYTE;

fn run_constants_item(emitter: &mut Emitter) -> u64 {
    let comparisons = constants_against_the_implementation();
    let mut mismatches = 0;
    for comparison in &comparisons {
        emitter.emit(&format!(
            "name=constant_compare constant={} local={} implementation={} equal={}",
            comparison.name,
            comparison.local,
            comparison.implementation,
            comparison.is_equal(),
        ));
        mismatches += u64::from(!comparison.is_equal());
    }
    emitter.emit(&format!(
        "name=verdict item=constants constants_compared={} constants_mismatched={mismatches}",
        comparisons.len()
    ));
    mismatches
}

fn run_positive_control_heap_item(emitter: &mut Emitter) -> u64 {
    let control = heap_positive_control(&COUNTING_ALLOCATOR, HEAP_POSITIVE_CONTROL_SLACK_BYTES);
    emitter.emit(&format!(
        "name=positive_control_heap peak_over_baseline_bytes={} lower_bytes={} upper_bytes={} holds={}",
        control.peak_over_baseline_bytes,
        HEAP_POSITIVE_CONTROL_BYTES,
        HEAP_POSITIVE_CONTROL_BYTES + HEAP_POSITIVE_CONTROL_SLACK_BYTES,
        control.holds,
    ));
    emitter.emit(&format!(
        "name=verdict item=positive-control-heap positive_control_heap_holds={}",
        control.holds
    ));
    u64::from(!control.holds)
}

fn run_anchors_item(emitter: &mut Emitter) -> u64 {
    let mut mismatches = 0;
    for anchor in &REGISTERED_RING_WINDOW_ANCHORS {
        let local = ring_window_by_the_clauses(
            anchor.device_bytes,
            anchor.journal_ring_bytes,
            anchor.slots_per_region,
            0,
        );
        let reproduced = anchor.is_reproduced_by(&local);
        mismatches += u64::from(!reproduced);
        emitter.emit(&format!(
            "name=anchor_geometry device_bytes={} journal_ring_bytes={} slots_per_region={} unit_area_first_slot={} unit_area_slots={} root_level={} positions_at_each_level={:?} deletion_publish_slots={} ring_window_slots={} ring_window_bytes={} matches_the_registered_script_output={reproduced}",
            anchor.device_bytes,
            anchor.journal_ring_bytes,
            anchor.slots_per_region,
            local.unit_area_first_slot,
            local.unit_area_slots,
            local.root_level,
            local.positions_at_each_level,
            local.deletion_publish_slots,
            local.ring_window_slots,
            local.ring_window_bytes(),
        ));
    }
    for (rows, registered_bytes) in REGISTERED_CHAIN_REWRITE_BYTES {
        let local_bytes = chain_rewrite_bytes_of_the_switch_reserve(rows);
        mismatches += u64::from(local_bytes != registered_bytes);
        emitter.emit(&format!(
            "name=anchor_chain_rewrite instance_rows={rows} local_bytes={local_bytes} registered_bytes={registered_bytes} matches_the_registered_script_output={}",
            local_bytes == registered_bytes,
        ));
    }
    let default_ring_start_slot = JOURNAL_RING_START_SLOT + JOURNAL_RING_DEFAULT_BYTES / SLOT_BYTES;
    emitter.emit(&format!(
        "name=anchor_default_ring_unit_area_start local_slot={default_ring_start_slot} registered_slot={UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING} equal={}",
        default_ring_start_slot == UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING,
    ));
    mismatches += u64::from(default_ring_start_slot != UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING);
    emitter.emit(&format!(
        "name=verdict item=anchors anchor_mismatches={mismatches}"
    ));
    mismatches
}

/// PC-准入（登记 5.3）：在 C_mkfs 上用只供测试的开关挂载得上；在 C_挂空 − 1 MiB 上产品路径报 `SpaceAdmissionRefusedBeforeAcquisition`。
#[derive(Clone, Debug, PartialEq, Eq)]
struct AdmissionPositiveControl {
    mount_with_the_test_only_switch_at_the_make_filesystem_minimum: String,
    one_step_below_the_empty_mount_minimum_mebibytes: Option<u64>,
    empty_mount_one_step_below: String,
    file_mount_one_step_below_the_file_minimum: String,
    holds: bool,
}

const REFUSAL_BEFORE_ACQUISITION_VARIANT: &str = "SpaceAdmissionRefusedBeforeAcquisition";

/// 判 PC-准入的纯函数：开关挂载得上，且低一步的那次尝试里空池挂载败在准入拒绝上。
fn admission_positive_control_holds(
    switch_mount_succeeded: bool,
    empty_mount_one_step_below: &StageOutcome<MountReport>,
) -> bool {
    switch_mount_succeeded
        && matches!(empty_mount_one_step_below, StageOutcome::Failed(variant)
            if variant == REFUSAL_BEFORE_ACQUISITION_VARIANT)
}

fn admission_positive_control(
    trials: &[Trial],
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> AdmissionPositiveControl {
    let flags: Vec<TrialFlags> = trials.iter().map(Trial::flags).collect();
    let make_filesystem_minimum = minimum_device_bytes(&flags, Milestone::MakeFilesystemSucceeds);
    let empty_mount_minimum = minimum_device_bytes(&flags, Milestone::EmptyMountAdmitted);
    let file_mount_minimum = minimum_device_bytes(&flags, Milestone::FileMountAdmitted);
    let switch_mount = match make_filesystem_minimum {
        None => (false, "not_run".to_string()),
        Some(device_bytes) => {
            let parameters = make_filesystem_parameters(journal_ring_bytes, slots_per_region);
            let mut pool = counted_pool(device_bytes, journal_ring_bytes, slots_per_region);
            let mounted = guarded(|| {
                make_filesystem(&parameters, &mut pool)
                    .map_err(|error| variant_name_of(&format!("{error:?}")))?;
                mount_writable_with_space_admission(
                    &parameters,
                    &mut pool,
                    SpaceAdmission::SkippedByTheTestOnlySwitch,
                )
                .map(|mounted| AdmissionClass::of_the_mount(&mounted.output.space_admission))
                .map_err(|error| variant_name_of(&format!("{error:?}")))
            });
            match mounted {
                Ok(class) => (true, format!("ok:{}", class.name())),
                Err(reason) => (false, format!("failed:{reason}")),
            }
        }
    };
    let trial_one_step_below = |device_bytes: Option<u64>| {
        device_bytes
            .and_then(|bytes| bytes.checked_sub(SEARCH_STEP_BYTES))
            .and_then(|below| trials.iter().find(|trial| trial.device_bytes == below))
    };
    let below_empty = trial_one_step_below(empty_mount_minimum);
    let empty_mount_one_step_below =
        below_empty.map_or(StageOutcome::NotReached, |trial| trial.empty_mount.clone());
    let file_mount_one_step_below_text = trial_one_step_below(file_mount_minimum)
        .map_or("not_run".to_string(), |trial| mount_text(&trial.file_mount));
    AdmissionPositiveControl {
        mount_with_the_test_only_switch_at_the_make_filesystem_minimum: switch_mount.1,
        one_step_below_the_empty_mount_minimum_mebibytes: below_empty
            .map(|trial| trial.device_bytes / MEBIBYTE),
        empty_mount_one_step_below: mount_text(&empty_mount_one_step_below),
        file_mount_one_step_below_the_file_minimum: file_mount_one_step_below_text,
        holds: admission_positive_control_holds(switch_mount.0, &empty_mount_one_step_below),
    }
}

fn trial_at(trials: &[Trial], device_bytes: Option<u64>) -> Option<&Trial> {
    device_bytes.and_then(|bytes| trials.iter().find(|trial| trial.device_bytes == bytes))
}

fn ratio_text(numerator: Option<u64>, denominator: Option<u64>) -> String {
    match (numerator, denominator) {
        (Some(top), Some(bottom)) if bottom > 0 => {
            format!("{}permille", u128::from(top) * 1000 / u128::from(bottom))
        }
        (_, _) => "none".to_string(),
    }
}

fn run_small_disk_item(
    emitter: &mut Emitter,
    item: &str,
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> ItemOutcome {
    emitter.emit(&format!(
        "name=h4_config item={item} slots_per_region={slots_per_region} journal_ring_bytes={journal_ring_bytes} start_device_mebibytes={} step_mebibytes={} window_mebibytes={} ceiling_mebibytes={}",
        SEARCH_START_DEVICE_BYTES / MEBIBYTE,
        SEARCH_STEP_BYTES / MEBIBYTE,
        MONOTONICITY_WINDOW_BYTES / MEBIBYTE,
        SEARCH_CEILING_DEVICE_BYTES / MEBIBYTE,
    ));
    let trials = search_small_disks(journal_ring_bytes, slots_per_region);
    for trial in &trials {
        emit_trial(emitter, item, trial);
    }
    let flags: Vec<TrialFlags> = trials.iter().map(Trial::flags).collect();
    let minimum = |milestone| minimum_device_bytes(&flags, milestone);
    emitter.emit(&format!(
        "name=h4_minimum item={item} make_filesystem_mebibytes={} empty_mount_mebibytes={} file_mount_mebibytes={} empty_mount_still_short_mebibytes={} file_mount_still_short_mebibytes={} file_mount_over_make_filesystem={} trials={} panicked_trials={}",
        optional_mebibytes_text(minimum(Milestone::MakeFilesystemSucceeds)),
        optional_mebibytes_text(minimum(Milestone::EmptyMountAdmitted)),
        optional_mebibytes_text(minimum(Milestone::FileMountAdmitted)),
        optional_mebibytes_text(minimum(Milestone::EmptyMountStillShort)),
        optional_mebibytes_text(minimum(Milestone::FileMountStillShort)),
        ratio_text(
            minimum(Milestone::FileMountAdmitted),
            minimum(Milestone::MakeFilesystemSucceeds)
        ),
        trials.len(),
        panicked_stage_count(&trials),
    ));
    let mut every_milestone_monotone = true;
    for milestone in Milestone::THREE_MINIMUM_DISKS {
        let monotonicity = monotonicity_of(&flags, milestone);
        every_milestone_monotone &= monotonicity.is_monotone();
        emitter.emit(&format!(
            "name=h4_monotonicity item={item} milestone={} minimum_mebibytes={} failures_in_the_window_mebibytes={} never_tried_mebibytes={} monotone={}",
            milestone.name(),
            optional_mebibytes_text(monotonicity.minimum_device_bytes),
            list_of_mebibytes_text(&monotonicity.failures_in_the_window),
            list_of_mebibytes_text(&monotonicity.window_points_never_tried),
            monotonicity.is_monotone(),
        ));
    }
    let control = admission_positive_control(&trials, journal_ring_bytes, slots_per_region);
    emitter.emit(&format!(
        "name=positive_control_admission item={item} mount_with_the_test_only_switch_at_the_make_filesystem_minimum={} one_step_below_the_empty_mount_minimum_mebibytes={} empty_mount_one_step_below={} file_mount_one_step_below_the_file_minimum={} holds={}",
        control.mount_with_the_test_only_switch_at_the_make_filesystem_minimum,
        optional_number_text(control.one_step_below_the_empty_mount_minimum_mebibytes),
        control.empty_mount_one_step_below,
        control.file_mount_one_step_below_the_file_minimum,
        control.holds,
    ));
    let (dominance_points, reading_checks_hold) = emit_states_of_a_trial(
        emitter,
        item,
        journal_ring_bytes,
        slots_per_region,
        [
            StateToRead {
                state: StateKind::Empty,
                trial: trial_at(&trials, minimum(Milestone::EmptyMountAdmitted)),
            },
            StateToRead {
                state: StateKind::File,
                trial: trial_at(&trials, minimum(Milestone::FileMountAdmitted)),
            },
        ],
    );
    emitter.emit(&format!(
        "name=verdict item={item} positive_control_admission_holds={} make_filesystem_minimum_found={} empty_mount_minimum_found={} file_mount_minimum_found={} every_minimum_disk_monotone_over_the_window={every_milestone_monotone} reading_checks_hold={reading_checks_hold} panicked_trials={}",
        control.holds,
        minimum(Milestone::MakeFilesystemSucceeds).is_some(),
        minimum(Milestone::EmptyMountAdmitted).is_some(),
        minimum(Milestone::FileMountAdmitted).is_some(),
        panicked_stage_count(&trials),
    ));
    ItemOutcome {
        dominance_points,
        invalid_device_conditions: 0,
    }
}

/// 尝试里 panic 掉的步数（`guarded` 把 panic 记成 `failed:panicked:…`）。
fn panicked_stage_count(trials: &[Trial]) -> usize {
    let is_panic = |reason: &String| reason.starts_with("panicked:");
    trials
        .iter()
        .map(|trial| {
            let failed_texts = [
                matches!(&trial.make_filesystem, StageOutcome::Failed(reason) if is_panic(reason)),
                matches!(&trial.empty_mount, StageOutcome::Failed(reason) if is_panic(reason)),
                matches!(&trial.first_file, StageOutcome::Failed(reason) if is_panic(reason)),
                matches!(&trial.unmount, StageOutcome::Failed(reason) if is_panic(reason)),
                matches!(&trial.file_mount, StageOutcome::Failed(reason) if is_panic(reason)),
            ];
            failed_texts.iter().filter(|failed| **failed).count()
        })
        .sum()
}

fn run_default_disk_item(emitter: &mut Emitter, item: &str, slots_per_region: u64) -> ItemOutcome {
    emitter.emit(&format!(
        "name=h4_config item={item} slots_per_region={slots_per_region} journal_ring_bytes={JOURNAL_RING_DEFAULT_BYTES} device_mebibytes={}",
        DEFAULT_DEVICE_BYTES / MEBIBYTE,
    ));
    let trial = run_trial(
        DEFAULT_DEVICE_BYTES,
        JOURNAL_RING_DEFAULT_BYTES,
        slots_per_region,
    );
    emit_trial(emitter, item, &trial);
    let states_hold = trial.flags().empty_mount_admitted && trial.flags().file_mount_admitted;
    let (dominance_points, reading_checks_hold) = emit_states_of_a_trial(
        emitter,
        item,
        JOURNAL_RING_DEFAULT_BYTES,
        slots_per_region,
        [
            StateToRead {
                state: StateKind::Empty,
                trial: Some(&trial),
            },
            StateToRead {
                state: StateKind::File,
                trial: Some(&trial),
            },
        ],
    );
    emitter.emit(&format!(
        "name=verdict item={item} both_states_admitted={states_hold} reading_checks_hold={reading_checks_hold} panicked_trials={}",
        panicked_stage_count(std::slice::from_ref(&trial)),
    ));
    ItemOutcome {
        dominance_points,
        invalid_device_conditions: 0,
    }
}

/// R4c 跨点（登记 8.2）：同一状态同一入口上，各取样点的主导项是不是同一项；不同记「不稳定」。
fn emit_dominance_across_points(
    emitter: &mut Emitter,
    group: &str,
    points: &[DominancePoint],
) -> bool {
    let mut every_group_stable = true;
    for state in ["empty", "file"] {
        for entry in ReadingEntry::BOTH {
            let selected: Vec<&DominancePoint> = points
                .iter()
                .filter(|point| point.state == state && point.entry == entry)
                .collect();
            if selected.is_empty() {
                continue;
            }
            let names: BTreeSet<&str> = selected
                .iter()
                .map(|point| match &point.dominance {
                    Dominance::Dominant { item, .. } => *item,
                    Dominance::NoItemAboveHalf { .. } => "no_item_above_half",
                })
                .collect();
            let stable = names.len() == 1;
            every_group_stable &= stable;
            emitter.emit(&format!(
                "name=h4_dominance_across_points group={group} state={state} entry={} points={} outcomes={} stable={stable}",
                entry.name(),
                selected.len(),
                selected
                    .iter()
                    .map(|point| format!("{}={}", point.item, point.dominance.verdict_text()))
                    .collect::<Vec<_>>()
                    .join(";"),
            ));
        }
    }
    every_group_stable
}

/// 登记第八节「判别力自证」③：把 R4c 的门槛挪到真产物每份读数最大一项份额的另一侧，判定必须转；一份没转，这个判定的几何敏感性作废（V36）。
fn emit_discrimination_self_proof(emitter: &mut Emitter, points: &[DominancePoint]) -> bool {
    let flipped = points
        .iter()
        .filter(|point| dominance_flips_when_the_threshold_is_moved(&point.reserve_items))
        .count();
    let holds = !points.is_empty() && flipped == points.len();
    emitter.emit(&format!(
        "name=discrimination_self_proof judgement=r4c_dominance readings={} flipped={flipped} holds={holds}",
        points.len(),
    ));
    holds
}

// ============================================================================================
// 第二、三段共用：片文件（一片一份，写完才在末行写 `name=shard_done`，先写临时名再改名）与片文件里的产物行读回。
// ============================================================================================

/// 片文件末行：`E7RESULT name=shard_done shard=<片名> lines=<整份行数，含这一行>`。
fn shard_done_line(shard: &str, line_count_including_this_one: usize) -> String {
    format!("E7RESULT name=shard_done shard={shard} lines={line_count_including_this_one}")
}

/// 一片的行（不含末行）连同末行写进 `<路径>.partial-<进程号>`、刷盘，再改名成 `<路径>`：被打断时片文件要么不在、要么整份在。
fn write_shard_file_atomically(path: &Path, shard: &str, lines: &[String]) -> std::io::Result<()> {
    let partial = PathBuf::from(format!("{}.partial-{}", path.display(), std::process::id()));
    {
        let mut file = std::fs::File::create(&partial)?;
        for line in lines {
            writeln!(file, "{line}")?;
        }
        writeln!(file, "{}", shard_done_line(shard, lines.len() + 1))?;
        file.sync_all()?;
    }
    std::fs::rename(&partial, path)
}

/// 片文件做完了：末行逐字等于这一片的完成行（行数对得上）。做完的交回不含末行的那些行。
fn lines_of_a_finished_shard(text: &str, shard: &str) -> Option<Vec<String>> {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let last = lines.last()?;
    (*last == shard_done_line(shard, lines.len())).then(|| lines[..lines.len() - 1].to_vec())
}

/// 一行产物：`E7RESULT name=<名> <键>=<值> …`。第二、三段打的行值里都没有空格，按空白切、按第一个 `=` 分键值。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ResultLine {
    fields: Vec<(String, String)>,
}

impl ResultLine {
    fn parse(line: &str) -> Option<ResultLine> {
        let body = line.strip_prefix("E7RESULT ")?;
        let fields = body
            .split_whitespace()
            .filter_map(|word| word.split_once('='))
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();
        Some(ResultLine { fields })
    }

    fn field(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(field_key, _)| field_key == key)
            .map(|(_, value)| value.as_str())
    }

    fn name(&self) -> Option<&str> {
        self.field("name")
    }

    fn number(&self, key: &str) -> Option<u64> {
        self.field(key).and_then(|value| value.parse().ok())
    }

    fn truth(&self, key: &str) -> Option<bool> {
        match self.field(key) {
            Some("true") => Some(true),
            Some("false") => Some(false),
            Some(_) | None => None,
        }
    }
}

/// 一片里叫这个名字的行。
fn lines_named<'lines>(lines: &'lines [ResultLine], name: &str) -> Vec<&'lines ResultLine> {
    lines
        .iter()
        .filter(|line| line.name() == Some(name))
        .collect()
}

/// 确定性核（登记第五节「另在每段第一格多跑 1 轮核两轮逐字段相同」、停机 T4）要去掉的字段：挂钟、进程的高水位与片名、程序文件的 sha256。
fn is_a_field_left_out_of_the_determinism_check(key: &str) -> bool {
    key.ends_with("nanoseconds")
        || key == "vm_high_water_mark_kibibytes"
        || key == "shard"
        || key == "binary_sha256"
}

/// 两份片逐行、逐字段比（去掉 [`is_a_field_left_out_of_the_determinism_check`] 那几样）；交回第一处不同的行号（从 1 起），全同交回 `None`。
fn first_line_differing_outside_the_timing_fields(
    first: &[ResultLine],
    second: &[ResultLine],
) -> Option<usize> {
    let kept = |line: &ResultLine| -> Vec<(String, String)> {
        line.fields
            .iter()
            .filter(|(key, _)| !is_a_field_left_out_of_the_determinism_check(key))
            .cloned()
            .collect()
    };
    let longest = first.len().max(second.len());
    (0..longest)
        .find(|index| match (first.get(*index), second.get(*index)) {
            (Some(left), Some(right)) => kept(left) != kept(right),
            (Some(_), None) | (None, Some(_)) | (None, None) => true,
        })
        .map(|index| index + 1)
}

/// 进程此刻的内存高水位（KiB）：`/proc/self/status` 的 VmHWM，在 Linux 上与 `getrusage` 的 `ru_maxrss` 是同一个数（登记 X7「`ru_maxrss` 附带报」）。
fn vm_high_water_mark_kibibytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|number| number.parse().ok())
}

// ============================================================================================
// E37 里 H2 的判定（纯函数）：X10 的斜率、斜率的类、5 轮同类才写类、判别力自证 ①。
// ============================================================================================

/// X10：(log₂ x, log₂ y) 的最小二乘斜率；y 为 0 的点按 1 取对数，另记「有 0 点」。
#[derive(Clone, Copy, Debug, PartialEq)]
struct SlopeReading {
    slope: f64,
    has_zero_point: bool,
    points: usize,
}

/// 点数不足两个、或 x 全相同（斜率没定义）交回 `None`。
fn log_log_least_squares_slope(points: &[(u64, u64)]) -> Option<SlopeReading> {
    let has_zero_point = points.iter().any(|(_, ordinate)| *ordinate == 0);
    let logarithms: Vec<(f64, f64)> = points
        .iter()
        .map(|(abscissa, ordinate)| {
            (
                log2_of_a_count(*abscissa),
                log2_of_a_count((*ordinate).max(1)),
            )
        })
        .collect();
    let count = logarithms.len();
    if count < 2 {
        return None;
    }
    let count_as_float = f64::from(u32::try_from(count).expect("点数装得进 u32"));
    let mean_abscissa =
        logarithms.iter().map(|(abscissa, _)| abscissa).sum::<f64>() / count_as_float;
    let mean_ordinate =
        logarithms.iter().map(|(_, ordinate)| ordinate).sum::<f64>() / count_as_float;
    let abscissa_spread: f64 = logarithms
        .iter()
        .map(|(abscissa, _)| (abscissa - mean_abscissa) * (abscissa - mean_abscissa))
        .sum();
    if abscissa_spread == 0.0 {
        return None;
    }
    let covariance: f64 = logarithms
        .iter()
        .map(|(abscissa, ordinate)| (abscissa - mean_abscissa) * (ordinate - mean_ordinate))
        .sum();
    Some(SlopeReading {
        slope: covariance / abscissa_spread,
        has_zero_point,
        points: count,
    })
}

/// 计数取以 2 为底的对数（计数都在 2⁵³ 以内，转成浮点不丢值的那一段由断言守着）。
fn log2_of_a_count(count: u64) -> f64 {
    assert!(count > 0, "取对数的计数要大于 0：0 点在调用方按 1 取");
    assert!(count < (1u64 << 53), "计数装得进双精度的整数段");
    #[allow(
        clippy::cast_precision_loss,
        reason = "上一行断言了计数小于 2 的 53 次方，转成双精度不丢值"
    )]
    let as_float = count as f64;
    as_float.log2()
}

/// 斜率的类（登记第六节开头）：b ≥ 0.8 随 X 线性，0.2 ≤ b < 0.8 次线性，b < 0.2 与 X 基本无关。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SlopeClass {
    BasicallyUnrelated,
    Sublinear,
    Linear,
}

impl SlopeClass {
    fn name(self) -> &'static str {
        match self {
            SlopeClass::BasicallyUnrelated => "basically_unrelated",
            SlopeClass::Sublinear => "sublinear",
            SlopeClass::Linear => "linear",
        }
    }
}

const LINEAR_SLOPE_BOUNDARY: f64 = 0.8;
const UNRELATED_SLOPE_BOUNDARY: f64 = 0.2;

fn slope_class(slope: f64) -> SlopeClass {
    slope_class_with_linear_boundary(slope, LINEAR_SLOPE_BOUNDARY)
}

fn slope_class_with_linear_boundary(slope: f64, linear_boundary: f64) -> SlopeClass {
    if slope >= linear_boundary {
        SlopeClass::Linear
    } else if slope >= UNRELATED_SLOPE_BOUNDARY {
        SlopeClass::Sublinear
    } else {
        SlopeClass::BasicallyUnrelated
    }
}

/// 有随机性的量每轮各算一个 b，5 轮落在同一类才写那一类，否则记「不稳定」；有一轮算不出 b（点不够）记「判不出」。
#[derive(Clone, Debug, PartialEq)]
enum ClassAcrossRounds {
    Stable(SlopeClass),
    Unstable,
    NotJudged {
        rounds_with_a_slope: usize,
    },
    /// 登记的 X5（覆盖写第 0 个数据单元以内的 4096 字节）在这一组格上造不出：不是 0、不是「基本无关」，这一行不出类（登记第十二节修订）。
    RegisteredPublishCannotBeMade,
}

impl ClassAcrossRounds {
    fn text(&self) -> String {
        match self {
            ClassAcrossRounds::Stable(class) => class.name().to_string(),
            ClassAcrossRounds::Unstable => "unstable".to_string(),
            ClassAcrossRounds::RegisteredPublishCannotBeMade => {
                "x5_as_registered_cannot_be_made".to_string()
            }
            ClassAcrossRounds::NotJudged {
                rounds_with_a_slope,
            } => {
                format!("not_judged_rounds_with_a_slope_{rounds_with_a_slope}")
            }
        }
    }
}

fn class_across_rounds(slopes: &[Option<f64>], rounds_required: usize) -> ClassAcrossRounds {
    let known: Vec<f64> = slopes.iter().flatten().copied().collect();
    if known.len() < rounds_required {
        return ClassAcrossRounds::NotJudged {
            rounds_with_a_slope: known.len(),
        };
    }
    let classes: BTreeSet<SlopeClass> = known.iter().map(|slope| slope_class(*slope)).collect();
    match classes.iter().next() {
        Some(class) if classes.len() == 1 => ClassAcrossRounds::Stable(*class),
        Some(_) | None => ClassAcrossRounds::Unstable,
    }
}

/// 斜率写进产物：千分之一精度的定点数，免得浮点的末位随平台变。
fn slope_text(slope: Option<f64>) -> String {
    slope.map_or_else(|| "none".to_string(), |value| format!("{value:.3}"))
}

/// 判别力自证 ①（登记第八节）：把 0.8 那条分界挪到真产物里两个相邻的 b 之间，至少一个 b 的类必须转。
/// 挑的是「挪过去之后至少转一个」的相邻两点里中点离 0.8 最近的那一对；一对都没有（b 只有一个值，或只跨着 0.8 的那一对）交回 `None`。
#[derive(Clone, Copy, Debug, PartialEq)]
struct MovedBoundary {
    boundary: f64,
    flipped: usize,
}

fn linear_boundary_moved_between_real_slopes(slopes: &[f64]) -> Option<MovedBoundary> {
    let mut sorted: Vec<f64> = slopes.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted.dedup();
    sorted
        .windows(2)
        .map(|pair| (pair[0] + pair[1]) / 2.0)
        .map(|boundary| MovedBoundary {
            boundary,
            flipped: slopes
                .iter()
                .filter(|slope| {
                    slope_class(**slope) != slope_class_with_linear_boundary(**slope, boundary)
                })
                .count(),
        })
        .filter(|moved| moved.flipped > 0)
        .min_by(|left, right| {
            (left.boundary - LINEAR_SLOPE_BOUNDARY)
                .abs()
                .total_cmp(&(right.boundary - LINEAR_SLOPE_BOUNDARY).abs())
        })
}

/// 真产物的 b 挪不出一次转面时的代用（登记第八节「喂合成的 (b = 0.5, 0.9) 必须判出两类」）。
const SYNTHETIC_SLOPES_FOR_THE_SELF_PROOF: [f64; 2] = [0.5, 0.9];

fn synthetic_slopes_fall_in_two_classes() -> bool {
    slope_class(SYNTHETIC_SLOPES_FOR_THE_SELF_PROOF[0])
        != slope_class(SYNTHETIC_SLOPES_FOR_THE_SELF_PROOF[1])
}

/// 中位数（偶数个时取靠下的那一个，量都是整数，不取平均）。空表交回 `None`。
fn median_of(values: &[u64]) -> Option<u64> {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.get(sorted.len().saturating_sub(1) / 2).copied()
}

// ============================================================================================
// E33：H2 的容量格与记录格（登记 X2–X10、5.2）。文件后端稀疏镜像，一格一轮一个进程、一片结果文件。
// ============================================================================================

/// 数据单元的载荷容量：D18（块里携带什么信息） 已定项 11 登记表码 1 那一行「明文头 105 / 含预留位 134」，单元 32768
/// ⇒ 32768 − 134 = 32634；记录格顺序写 n 个单元的内容长度按它算。回比 `singlefs_core::unit::data_unit_payload_capacity`（`h2_payload_constant` 行）。
const DATA_UNIT_PAYLOAD_BYTES: u64 = 32634;
const TEBIBYTE: u64 = 1 << 40;
/// 容量阶梯（登记 5.2、X3）。
const CAPACITY_LADDER_DEVICE_BYTES: [u64; 5] = [
    4 * GIBIBYTE,
    64 * GIBIBYTE,
    TEBIBYTE,
    4 * TEBIBYTE,
    16 * TEBIBYTE - MEBIBYTE,
];
/// 记录格：C = 4 GiB 上首个文件之后顺序写的单元数（登记 5.2）。
const RECORD_CELL_SEQUENTIALLY_WRITTEN_UNIT_COUNTS: [u64; 3] = [0, 1024, 10240];
/// 几何敏感性（登记 8.2）：记录格 n = 10240 那一状态在这三档容量上再求一次 b。
const OCCUPANCY_SENSITIVITY_DEVICE_BYTES: [u64; 3] = [4 * GIBIBYTE, 64 * GIBIBYTE, TEBIBYTE];
const OCCUPANCY_SENSITIVITY_UNITS: u64 = 10240;
/// 每格 5 个进程（5 轮），每个进程里 X4、X5 各 5 次（登记 X4、X5、5.2）。
const CAPACITY_CELL_ROUNDS: u64 = 5;
const MEASURED_CALLS_PER_ROUND: u64 = 5;
/// PC-平（登记 5.3）：量窗里分配并丢掉一份 1 MiB。
const FLAT_CONTROL_BYTES: usize = 1 << 20;
/// X5：覆盖写第一个文件第 0 个数据单元以内的 4096 字节。
const MEASURED_OVERWRITE_BYTES: usize = 4096;

/// 一个格：盘容量与首个文件之后顺序写的单元数（容量格是 0）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CapacityCell {
    device_bytes: u64,
    sequentially_written_units: u64,
}

impl CapacityCell {
    fn name(self) -> String {
        format!(
            "h2-device-{}-mebibytes-units-{}",
            self.device_bytes / MEBIBYTE,
            self.sequentially_written_units
        )
    }
}

/// 全部格，次序固定：容量阶梯（n = 0），记录格里 n > 0 的两格，几何敏感性里还没出现过的两格。同一格只出现一次（4 GiB、n = 0 既是容量格也是记录格）。
fn capacity_cells() -> Vec<CapacityCell> {
    let ladder = CAPACITY_LADDER_DEVICE_BYTES.map(|device_bytes| CapacityCell {
        device_bytes,
        sequentially_written_units: 0,
    });
    let records = RECORD_CELL_SEQUENTIALLY_WRITTEN_UNIT_COUNTS.map(|units| CapacityCell {
        device_bytes: 4 * GIBIBYTE,
        sequentially_written_units: units,
    });
    let occupancy = OCCUPANCY_SENSITIVITY_DEVICE_BYTES.map(|device_bytes| CapacityCell {
        device_bytes,
        sequentially_written_units: OCCUPANCY_SENSITIVITY_UNITS,
    });
    let mut cells: Vec<CapacityCell> = Vec::new();
    for cell in ladder.into_iter().chain(records).chain(occupancy) {
        if !cells.contains(&cell) {
            cells.push(cell);
        }
    }
    cells
}

/// 全部片，轮在外层（第 1 轮的每一格、第 2 轮的每一格……）：挂钟随时间的漂移摊到每一格上，不集中在某一格。
fn capacity_shards() -> Vec<(String, CapacityCell, u64)> {
    (1..=CAPACITY_CELL_ROUNDS)
        .flat_map(|round| {
            capacity_cells()
                .into_iter()
                .map(move |cell| (format!("{}-round-{round}", cell.name()), cell, round))
        })
        .collect()
}

fn capacity_shard_by_name(name: &str) -> Option<(CapacityCell, u64)> {
    capacity_shards()
        .into_iter()
        .find(|(shard, _, _)| shard == name)
        .map(|(_, cell, round)| (cell, round))
}

/// X5 第 `call_index` 次覆盖写的内容：4096 字节，随轮次与序号变（同 E156 `overwrite_content` 的造法；首个文件是 0 号）。
fn measured_overwrite_content(round: u64, call_index: u64) -> Vec<u8> {
    let salt = usize::try_from(1 + round * MEASURED_CALLS_PER_ROUND + call_index).expect("序号小");
    (0..MEASURED_OVERWRITE_BYTES)
        .map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256"))
        .collect()
}

/// 记录格顺序写的内容：`units` 个单元的载荷，随字节序号变。
fn sequential_write_content(units: u64) -> Vec<u8> {
    let length =
        usize::try_from(units * DATA_UNIT_PAYLOAD_BYTES).expect("至多 10240 个单元，装得进 usize");
    (0..length)
        .map(|index| u8::try_from((index / 4096 + index) % 251).expect("小于 256"))
        .collect()
}

/// 一个量窗（登记 X6、X7）：被测进程里 `Instant` 包住那一调；开始时把峰值重置成此刻的在用字节 B₀，结束读峰值 P，报 P − B₀、B₀ 与结束时的在用字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MeasuredWindow {
    nanoseconds: u64,
    peak_over_baseline_bytes: u64,
    baseline_bytes: u64,
    in_use_after_bytes: u64,
}

impl MeasuredWindow {
    fn text(&self) -> String {
        format!(
            "nanoseconds={} peak_over_baseline_bytes={} baseline_bytes={} in_use_after_bytes={}",
            self.nanoseconds,
            self.peak_over_baseline_bytes,
            self.baseline_bytes,
            self.in_use_after_bytes
        )
    }
}

fn measure_in_a_window<Output>(
    allocator: &CountingAllocator,
    call: impl FnOnce() -> Output,
) -> (Output, MeasuredWindow) {
    let baseline_bytes = allocator.in_use();
    allocator.reset_peak_to_in_use();
    let started = Instant::now();
    let output = call();
    let nanoseconds = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let window = MeasuredWindow {
        nanoseconds,
        peak_over_baseline_bytes: allocator.peak().saturating_sub(baseline_bytes),
        baseline_bytes,
        in_use_after_bytes: allocator.in_use(),
    };
    (output, window)
}

/// 这个进程建的镜像文件：进程结束（正常返回或 panic 展开）时删掉；删不掉不报（镜像在草稿目录里，跑法脚本起片之前按片名清一遍）。
struct ImageFilesRemovedOnDrop {
    paths: Vec<PathBuf>,
}

impl Drop for ImageFilesRemovedOnDrop {
    fn drop(&mut self) {
        for path in &self.paths {
            let removed = std::fs::remove_file(path);
            std::hint::black_box(removed.is_ok());
        }
    }
}

type FilePool = Vec<(DeviceIdentity, FileBackedBlockDevice)>;

/// 镜像文件名：片名加进程号（同一片被杀之后重起不会撞上一份没删的镜像，`create_image_file_exclusively` 不许接着用旧盘面）。
fn capacity_image_path(image_directory: &Path, shard: &str, device_number: u32) -> PathBuf {
    image_directory.join(format!(
        "e183-{shard}-process-{}-device-{device_number}.img",
        std::process::id()
    ))
}

/// 报错前面加上是哪一步：`<步名>:<原因>`。
fn failed_at(step: &'static str) -> impl Fn(String) -> String {
    move |reason| format!("{step}:{reason}")
}

/// 测量那一刻之前的几步：mkfs → 可写挂载 → 首个文件 →（记录格）顺序写 n 个单元 → 正常卸载 → 可写挂载（登记 5.2）。
/// 哪一步没成交回 `Err("<步名>:<原因>")`，后面不走。
fn prepare_the_measured_mount(
    cell: CapacityCell,
    parameters: &MakeFilesystemParameters,
    pool: &mut FilePool,
) -> Result<Mounted, String> {
    guarded(|| {
        make_filesystem(parameters, pool)
            .map(|_| ())
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("make_filesystem"))?;
    let mounted = guarded(|| {
        mount_writable(parameters, pool).map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("first_mount"))?;
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let first_file = first_file_content();
    let first = guarded(|| {
        let mut writer = PoolWriter::new(parameters, pool.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut session.allocator,
            session.current.root(),
            FirstFile {
                content: &first_file,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            session.instance,
            session.current.record_bytes(),
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("first_file"))?;
    session.current = PoolVersion::of_a_version_of_trees(first);
    if cell.sequentially_written_units > 0 {
        let content = sequential_write_content(cell.sequentially_written_units);
        guarded(|| {
            session
                .publish_user_change(
                    pool,
                    UserChange::SequentialWrite(FirstFile {
                        content: &content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                    }),
                )
                .map(|_| ())
                .map_err(|error| variant_name_of(&format!("{error:?}")))
        })
        .map_err(failed_at("sequential_write"))?;
    }
    guarded(|| {
        unmount(
            parameters,
            pool,
            &mut session.allocator,
            &mut session.current,
            session.shadow_ledger,
        )
        .map(|_| ())
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("unmount"))?;
    guarded(|| {
        mount_writable(parameters, pool).map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("measured_mount"))
}

/// 一片的结局：做完（片文件照写，没走到测量的那几步也算做完，合并时按「那一格这一轮作废」处置）；停机（片文件不写，退 3）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum ShardEnd {
    Finished,
    Stopped { reason: String },
}

/// 一片 H2：PC-堆、载荷常量回比（不等 ⇒ 停机 T2）、建两块镜像（建不出记「本机文件系统拒」，X3）、走到测量那一刻、X8、X4 五次、PC-平、X5 五次、进程高水位。
fn run_capacity_cell(
    shard: &str,
    cell: CapacityCell,
    round: u64,
    image_directory: &Path,
    binary_sha256: &str,
    emitter: &mut Emitter,
) -> ShardEnd {
    run_capacity_cell_with_a_journal_ring(
        shard,
        cell,
        round,
        JOURNAL_RING_DEFAULT_BYTES,
        image_directory,
        binary_sha256,
        emitter,
    )
}

/// 同 [`run_capacity_cell`]，环长由调用方给：产物取 768 MiB（登记 5.2），单测在小盘、小环上走同一条路。
fn run_capacity_cell_with_a_journal_ring(
    shard: &str,
    cell: CapacityCell,
    round: u64,
    journal_ring_bytes: u64,
    image_directory: &Path,
    binary_sha256: &str,
    emitter: &mut Emitter,
) -> ShardEnd {
    let unit_area_start = unit_area_first_and_last_slot(cell.device_bytes, journal_ring_bytes).0;
    let slots_of_the_unit_area = unit_area_slots(cell.device_bytes, journal_ring_bytes);
    emitter.emit(&format!(
        "name=h2_config shard={shard} device_bytes={} sequentially_written_units={} round={round} journal_ring_bytes={journal_ring_bytes} slots_per_region={SLOTS_PER_REGION_AT_MAKE_FILESYSTEM} unit_area_slots={slots_of_the_unit_area} binary_sha256={binary_sha256}",
        cell.device_bytes, cell.sequentially_written_units,
    ));
    let implementation_payload = u64::try_from(singlefs_core::unit::data_unit_payload_capacity())
        .expect("载荷容量装得进 u64");
    emitter.emit(&format!(
        "name=h2_payload_constant shard={shard} local={DATA_UNIT_PAYLOAD_BYTES} implementation={implementation_payload} equal={}",
        implementation_payload == DATA_UNIT_PAYLOAD_BYTES
    ));
    if implementation_payload != DATA_UNIT_PAYLOAD_BYTES {
        return ShardEnd::Stopped {
            reason: "t2_payload_constant_differs".to_string(),
        };
    }
    let heap = heap_positive_control(&COUNTING_ALLOCATOR, HEAP_POSITIVE_CONTROL_SLACK_BYTES);
    emitter.emit(&format!(
        "name=positive_control_heap shard={shard} peak_over_baseline_bytes={} holds={}",
        heap.peak_over_baseline_bytes, heap.holds
    ));
    let mut images = ImageFilesRemovedOnDrop { paths: Vec::new() };
    let mut pool: FilePool = Vec::new();
    for device_number in 0..2u32 {
        let path = capacity_image_path(image_directory, shard, device_number);
        match FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            cell.device_bytes,
            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
        ) {
            Ok(device) => {
                images.paths.push(path);
                pool.push((DeviceIdentity(device_number), device));
            }
            Err(error) => {
                emitter.emit(&format!(
                    "name=h2_steps shard={shard} outcome=filesystem_refused:{}",
                    variant_name_of(&format!("{error:?}"))
                ));
                return ShardEnd::Finished;
            }
        }
    }
    let parameters =
        make_filesystem_parameters(journal_ring_bytes, SLOTS_PER_REGION_AT_MAKE_FILESYSTEM);
    let mounted = match prepare_the_measured_mount(cell, &parameters, &mut pool) {
        Ok(mounted) => mounted,
        Err(failed_step) => {
            emitter.emit(&format!(
                "name=h2_steps shard={shard} outcome=failed_at:{failed_step}"
            ));
            return ShardEnd::Finished;
        }
    };
    emitter.emit(&format!("name=h2_steps shard={shard} outcome=ok"));
    let records = mounted.allocator.records().len();
    let (allocator_copy, copy_window) =
        measure_in_a_window(&COUNTING_ALLOCATOR, || mounted.allocator.clone());
    drop(allocator_copy);
    let copy_bytes = copy_window
        .in_use_after_bytes
        .saturating_sub(copy_window.baseline_bytes);
    emitter.emit(&format!(
        "name=h2_allocator_copy shard={shard} records={records} copy_bytes={copy_bytes} copy_nanoseconds={} bytes_per_slot_per_device_millionths={}",
        copy_window.nanoseconds,
        u128::from(copy_bytes) * 1_000_000 / u128::from(POOL_DEVICE_COUNT * slots_of_the_unit_area),
    ));
    let generation = CheckpointTxg(mounted.current.root().checkpoint_txg.0 + 1);
    for call_index in 0..MEASURED_CALLS_PER_ROUND {
        let mut copy = mounted.allocator.clone();
        let (placement, window) =
            measure_in_a_window(&COUNTING_ALLOCATOR, || copy.allocate_user_data(generation));
        drop(copy);
        emitter.emit(&format!(
            "name=h2_allocate shard={shard} call={call_index} {} slots_from_unit_area_start={}",
            window.text(),
            placement.map_or_else(
                || "refused".to_string(),
                |placed| placed.slot.0.saturating_sub(unit_area_start).to_string()
            ),
        ));
    }
    let ((), flat_window) = measure_in_a_window(&COUNTING_ALLOCATOR, || {
        let buffer = vec![0x5Au8; FLAT_CONTROL_BYTES];
        std::hint::black_box(&buffer);
    });
    emitter.emit(&format!(
        "name=positive_control_flat shard={shard} {}",
        flat_window.text()
    ));
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    for call_index in 0..MEASURED_CALLS_PER_ROUND {
        let content = measured_overwrite_content(round, call_index);
        let write_time_seconds =
            FIXED_WRITE_TIME_SECONDS + 2 + round * MEASURED_CALLS_PER_ROUND + call_index;
        let (published, window) = measure_in_a_window(&COUNTING_ALLOCATOR, || {
            session.publish_user_change(
                &mut pool,
                UserChange::Overwrite(FirstFile {
                    content: &content,
                    write_time_seconds,
                }),
            )
        });
        let (outcome, floor_raises) = match &published {
            Ok(change) => ("ok".to_string(), change.floor_raises.len()),
            Err(refusal) => (
                format!("failed:{}", variant_name_of(&format!("{refusal:?}"))),
                0,
            ),
        };
        drop(published);
        emitter.emit(&format!(
            "name=h2_publish shard={shard} call={call_index} outcome={outcome} floor_raises={floor_raises} {} records_after={}",
            window.text(),
            session.allocator.records().len(),
        ));
    }
    emitter.emit(&format!(
        "name=h2_process shard={shard} vm_high_water_mark_kibibytes={}",
        optional_number_text(vm_high_water_mark_kibibytes())
    ));
    drop(images);
    ShardEnd::Finished
}

// ============================================================================================
// H2 的合并：读回 45 片，算 E37 里 H2 的判定（R2a–R2f、PC-线性、PC-平、几何敏感性、判别力自证 ①、T4、G1、G2、G6）。
// ============================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AllocateCall {
    window: MeasuredWindow,
    /// 落点离单元区起点的槽数；分配被拒是 `None`。
    slots_from_unit_area_start: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PublishCall {
    window: MeasuredWindow,
    published: bool,
}

/// 一格一轮量到的东西（一片里读回的）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct CellRoundReading {
    heap_control_holds: bool,
    records: u64,
    copy_bytes: u64,
    copy_nanoseconds: u64,
    allocate: Vec<AllocateCall>,
    flat_peak_over_baseline_bytes: u64,
    publish: Vec<PublishCall>,
}

/// 内存包装的退出码（`scripts/hosts/run-with-memory-cap.sh` 文件头「退出码」）：250 是撞了这一条自己的上限（X3「内存上限之外」）。
const WRAPPER_EXIT_OF_HITTING_ITS_OWN_MEMORY_LIMIT: u64 = 250;
/// 251 用法错、252 内存不够排不上、253 超限时、254 被总上限挤掉：命令没跑完，也不是这一片撞了自己的上限，不记「内存上限之外」，记这一片没跑成（登记第十二节修订：比 X3 字面严）。
const WRAPPER_EXITS_OF_A_SHARD_THAT_WAS_NOT_RUN: std::ops::RangeInclusive<u64> = 251..=254;

/// 一格一轮的结局：量到了；没走到测量（哪一步、本机文件系统拒）；进程撞了自己的内存上限（包装退 250，X3「内存上限之外」）；
/// 包装没让它跑完（退 251–254）；片不在；程序文件对不上。
#[derive(Clone, Debug, PartialEq, Eq)]
enum CellRoundStatus {
    Measured(CellRoundReading),
    NotMeasured(String),
    BeyondTheMemoryUpperLimit(u64),
    NotRunByTheWrapper(u64),
    Missing,
    BinaryOtherThanTheMerge(String),
}

impl CellRoundStatus {
    fn text(&self) -> String {
        match self {
            CellRoundStatus::Measured(_) => "measured".to_string(),
            CellRoundStatus::NotMeasured(outcome) => format!("not_measured:{outcome}"),
            CellRoundStatus::BeyondTheMemoryUpperLimit(exit_code) => {
                format!("beyond_the_memory_upper_limit:wrapper_exit_{exit_code}")
            }
            CellRoundStatus::NotRunByTheWrapper(exit_code) => {
                format!("not_run_by_the_wrapper:wrapper_exit_{exit_code}")
            }
            CellRoundStatus::Missing => "missing".to_string(),
            CellRoundStatus::BinaryOtherThanTheMerge(sha256) => {
                format!("binary_other_than_the_merge:{sha256}")
            }
        }
    }

    fn reading(&self) -> Option<&CellRoundReading> {
        match self {
            CellRoundStatus::Measured(reading) => Some(reading),
            CellRoundStatus::NotMeasured(_)
            | CellRoundStatus::BeyondTheMemoryUpperLimit(_)
            | CellRoundStatus::NotRunByTheWrapper(_)
            | CellRoundStatus::Missing
            | CellRoundStatus::BinaryOtherThanTheMerge(_) => None,
        }
    }
}

fn window_of_a_line(line: &ResultLine) -> Option<MeasuredWindow> {
    Some(MeasuredWindow {
        nanoseconds: line.number("nanoseconds")?,
        peak_over_baseline_bytes: line.number("peak_over_baseline_bytes")?,
        baseline_bytes: line.number("baseline_bytes")?,
        in_use_after_bytes: line.number("in_use_after_bytes")?,
    })
}

/// 一片的行读成一格一轮的结局。缺行、缺字段（V37：读不到 ≠ 读到 0）交回 `NotMeasured("fields_missing")`，不当 0。
fn status_of_a_capacity_shard(lines: &[ResultLine], binary_sha256: &str) -> CellRoundStatus {
    let fields_missing = || CellRoundStatus::NotMeasured("fields_missing".to_string());
    let Some(config) = lines_named(lines, "h2_config").into_iter().next() else {
        return fields_missing();
    };
    match config.field("binary_sha256") {
        Some(sha256) if sha256 == binary_sha256 => {}
        Some(sha256) => return CellRoundStatus::BinaryOtherThanTheMerge(sha256.to_string()),
        None => return fields_missing(),
    }
    let Some(steps) = lines_named(lines, "h2_steps").into_iter().next() else {
        return fields_missing();
    };
    match steps.field("outcome") {
        Some("ok") => {}
        Some(outcome) => return CellRoundStatus::NotMeasured(outcome.to_string()),
        None => return fields_missing(),
    }
    let parsed = (|| {
        let heap = lines_named(lines, "positive_control_heap")
            .into_iter()
            .next()?;
        let copy = lines_named(lines, "h2_allocator_copy").into_iter().next()?;
        let flat = lines_named(lines, "positive_control_flat")
            .into_iter()
            .next()?;
        let allocate: Option<Vec<AllocateCall>> = lines_named(lines, "h2_allocate")
            .into_iter()
            .map(|line| {
                Some(AllocateCall {
                    window: window_of_a_line(line)?,
                    slots_from_unit_area_start: line.number("slots_from_unit_area_start"),
                })
            })
            .collect();
        let publish: Option<Vec<PublishCall>> = lines_named(lines, "h2_publish")
            .into_iter()
            .map(|line| {
                Some(PublishCall {
                    window: window_of_a_line(line)?,
                    published: line.field("outcome")? == "ok",
                })
            })
            .collect();
        Some(CellRoundReading {
            heap_control_holds: heap.truth("holds")?,
            records: copy.number("records")?,
            copy_bytes: copy.number("copy_bytes")?,
            copy_nanoseconds: copy.number("copy_nanoseconds")?,
            allocate: allocate?,
            flat_peak_over_baseline_bytes: window_of_a_line(flat)?.peak_over_baseline_bytes,
            publish: publish?,
        })
    })();
    let calls = usize::try_from(MEASURED_CALLS_PER_ROUND).expect("5");
    match parsed {
        Some(reading) if reading.allocate.len() == calls && reading.publish.len() == calls => {
            CellRoundStatus::Measured(reading)
        }
        Some(_) | None => fields_missing(),
    }
}

/// 一格一轮报的量（登记第六节 R2a–R2e、5.3 的 PC-线性、PC-平）。堆量在 PC-堆 没过的那一片上不出（V30）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CapacityQuantity {
    AllocateNanoseconds,
    PublishNanoseconds,
    AllocatePeakOverBaselineBytes,
    PublishPeakOverBaselineBytes,
    AllocatorCopyBytes,
    AllocatorCopyNanoseconds,
    FlatControlPeakOverBaselineBytes,
}

impl CapacityQuantity {
    const EVERY: [CapacityQuantity; 7] = [
        CapacityQuantity::AllocateNanoseconds,
        CapacityQuantity::PublishNanoseconds,
        CapacityQuantity::AllocatePeakOverBaselineBytes,
        CapacityQuantity::PublishPeakOverBaselineBytes,
        CapacityQuantity::AllocatorCopyBytes,
        CapacityQuantity::AllocatorCopyNanoseconds,
        CapacityQuantity::FlatControlPeakOverBaselineBytes,
    ];
    /// R2a、R2b、R2d 两行：对容量、对记录数都求的那四样。
    const JUDGED: [CapacityQuantity; 4] = [
        CapacityQuantity::AllocateNanoseconds,
        CapacityQuantity::PublishNanoseconds,
        CapacityQuantity::AllocatePeakOverBaselineBytes,
        CapacityQuantity::PublishPeakOverBaselineBytes,
    ];

    fn name(self) -> &'static str {
        match self {
            CapacityQuantity::AllocateNanoseconds => "allocate_nanoseconds",
            CapacityQuantity::PublishNanoseconds => "publish_nanoseconds",
            CapacityQuantity::AllocatePeakOverBaselineBytes => "allocate_peak_over_baseline_bytes",
            CapacityQuantity::PublishPeakOverBaselineBytes => "publish_peak_over_baseline_bytes",
            CapacityQuantity::AllocatorCopyBytes => "allocator_copy_bytes",
            CapacityQuantity::AllocatorCopyNanoseconds => "allocator_copy_nanoseconds",
            CapacityQuantity::FlatControlPeakOverBaselineBytes => {
                "flat_control_peak_over_baseline_bytes"
            }
        }
    }

    /// 这一量在一格一轮里的每一个读数（X4、X5 各 5 次；X8、PC-平 各 1 次）。有一次分配被拒或发布没发成，那一轮这一量整个不出。
    fn samples(self, reading: &CellRoundReading) -> Option<Vec<u64>> {
        let heap_counts = reading.heap_control_holds;
        let every_allocation_placed = reading
            .allocate
            .iter()
            .all(|call| call.slots_from_unit_area_start.is_some());
        let every_publish_done = reading.publish.iter().all(|call| call.published);
        let of_allocate = |pick: fn(&MeasuredWindow) -> u64| {
            reading
                .allocate
                .iter()
                .map(|call| pick(&call.window))
                .collect::<Vec<u64>>()
        };
        let of_publish = |pick: fn(&MeasuredWindow) -> u64| {
            reading
                .publish
                .iter()
                .map(|call| pick(&call.window))
                .collect::<Vec<u64>>()
        };
        match self {
            CapacityQuantity::AllocateNanoseconds => {
                every_allocation_placed.then(|| of_allocate(|window| window.nanoseconds))
            }
            CapacityQuantity::PublishNanoseconds => {
                every_publish_done.then(|| of_publish(|window| window.nanoseconds))
            }
            CapacityQuantity::AllocatePeakOverBaselineBytes => (heap_counts
                && every_allocation_placed)
                .then(|| of_allocate(|window| window.peak_over_baseline_bytes)),
            CapacityQuantity::PublishPeakOverBaselineBytes => (heap_counts && every_publish_done)
                .then(|| of_publish(|window| window.peak_over_baseline_bytes)),
            CapacityQuantity::AllocatorCopyBytes => heap_counts.then(|| vec![reading.copy_bytes]),
            CapacityQuantity::AllocatorCopyNanoseconds => Some(vec![reading.copy_nanoseconds]),
            CapacityQuantity::FlatControlPeakOverBaselineBytes => {
                heap_counts.then(|| vec![reading.flat_peak_over_baseline_bytes])
            }
        }
    }

    /// 轮内中位数（登记 R2a「X4 的 5 次取轮内中位数」）。
    fn round_median(self, reading: &CellRoundReading) -> Option<u64> {
        self.samples(reading)
            .and_then(|samples| median_of(&samples))
    }
}

/// 求斜率的那一维：盘容量，或记录数（X9 的实际值）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlopeAxis {
    Capacity,
    Records,
}

impl SlopeAxis {
    fn name(self) -> &'static str {
        match self {
            SlopeAxis::Capacity => "capacity",
            SlopeAxis::Records => "records",
        }
    }

    fn abscissa_of(self, cell: CapacityCell, reading: &CellRoundReading) -> u64 {
        match self {
            SlopeAxis::Capacity => cell.device_bytes,
            SlopeAxis::Records => reading.records,
        }
    }
}

/// 读回的全部片：（格，轮，结局）。
type CapacityStatuses = Vec<(CapacityCell, u64, CellRoundStatus)>;

fn status_of(
    statuses: &CapacityStatuses,
    cell: CapacityCell,
    round: u64,
) -> Option<&CellRoundStatus> {
    statuses
        .iter()
        .find(|(status_cell, status_round, _)| *status_cell == cell && *status_round == round)
        .map(|(_, _, status)| status)
}

/// 一轮里给定几格上这一量的斜率：只取这一轮量到了、这一量也出了的格（「一轮里全部跑完的容量档」，X10）。
fn slope_in_one_round(
    statuses: &CapacityStatuses,
    cells: &[CapacityCell],
    round: u64,
    quantity: CapacityQuantity,
    axis: SlopeAxis,
) -> Option<SlopeReading> {
    let points: Vec<(u64, u64)> = cells
        .iter()
        .filter_map(|cell| {
            let reading = status_of(statuses, *cell, round)?.reading()?;
            Some((
                axis.abscissa_of(*cell, reading),
                quantity.round_median(reading)?,
            ))
        })
        .collect();
    log_log_least_squares_slope(&points)
}

fn ladder_cells() -> Vec<CapacityCell> {
    CAPACITY_LADDER_DEVICE_BYTES
        .map(|device_bytes| CapacityCell {
            device_bytes,
            sequentially_written_units: 0,
        })
        .to_vec()
}

fn record_cells() -> Vec<CapacityCell> {
    RECORD_CELL_SEQUENTIALLY_WRITTEN_UNIT_COUNTS
        .map(|units| CapacityCell {
            device_bytes: 4 * GIBIBYTE,
            sequentially_written_units: units,
        })
        .to_vec()
}

fn occupied_capacity_cells() -> Vec<CapacityCell> {
    OCCUPANCY_SENSITIVITY_DEVICE_BYTES
        .map(|device_bytes| CapacityCell {
            device_bytes,
            sequentially_written_units: OCCUPANCY_SENSITIVITY_UNITS,
        })
        .to_vec()
}

/// 一行斜率判定：哪一量、哪一维、哪几格、对应登记第六节的哪一行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SlopeRow {
    row: &'static str,
    quantity: CapacityQuantity,
    axis: SlopeAxis,
    group: SlopeGroup,
}

/// 求斜率用哪几格：容量阶梯（n = 0）、记录格（4 GiB 上 n 三档）、几何敏感性那三档容量（n = 10240）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlopeGroup {
    CapacityLadder,
    RecordCells,
    OccupiedCapacityCells,
}

impl SlopeGroup {
    fn name(self) -> &'static str {
        match self {
            SlopeGroup::CapacityLadder => "capacity_ladder",
            SlopeGroup::RecordCells => "record_cells",
            SlopeGroup::OccupiedCapacityCells => "occupied_capacity_cells",
        }
    }

    fn cells(self) -> Vec<CapacityCell> {
        match self {
            SlopeGroup::CapacityLadder => ladder_cells(),
            SlopeGroup::RecordCells => record_cells(),
            SlopeGroup::OccupiedCapacityCells => occupied_capacity_cells(),
        }
    }
}

fn capacity_slope_rows() -> Vec<SlopeRow> {
    let on_the_ladder = |row, quantity| SlopeRow {
        row,
        quantity,
        axis: SlopeAxis::Capacity,
        group: SlopeGroup::CapacityLadder,
    };
    let on_the_records = |row, quantity| SlopeRow {
        row,
        quantity,
        axis: SlopeAxis::Records,
        group: SlopeGroup::RecordCells,
    };
    let on_the_occupied = |row, quantity| SlopeRow {
        row,
        quantity,
        axis: SlopeAxis::Capacity,
        group: SlopeGroup::OccupiedCapacityCells,
    };
    vec![
        on_the_ladder("r2a", CapacityQuantity::AllocateNanoseconds),
        on_the_ladder("r2b", CapacityQuantity::PublishNanoseconds),
        on_the_ladder(
            "r2d_allocate",
            CapacityQuantity::AllocatePeakOverBaselineBytes,
        ),
        on_the_ladder(
            "r2d_publish",
            CapacityQuantity::PublishPeakOverBaselineBytes,
        ),
        on_the_ladder("r2c", CapacityQuantity::AllocatorCopyBytes),
        on_the_ladder(
            "positive_control_linear_copy_time",
            CapacityQuantity::AllocatorCopyNanoseconds,
        ),
        on_the_ladder(
            "positive_control_flat",
            CapacityQuantity::FlatControlPeakOverBaselineBytes,
        ),
        on_the_records("r2e_allocate", CapacityQuantity::AllocateNanoseconds),
        on_the_records("r2e_publish", CapacityQuantity::PublishNanoseconds),
        on_the_records(
            "r2e_allocate_peak",
            CapacityQuantity::AllocatePeakOverBaselineBytes,
        ),
        on_the_records(
            "r2e_publish_peak",
            CapacityQuantity::PublishPeakOverBaselineBytes,
        ),
        on_the_occupied("r2a_at_occupancy", CapacityQuantity::AllocateNanoseconds),
        on_the_occupied("r2b_at_occupancy", CapacityQuantity::PublishNanoseconds),
    ]
}

/// 一行的 5 轮斜率与类。
#[derive(Clone, Debug, PartialEq)]
struct SlopeRowJudgement {
    row: SlopeRow,
    slopes: Vec<Option<SlopeReading>>,
    class: ClassAcrossRounds,
}

impl CapacityQuantity {
    /// 这一量来自 X5 的发布。
    fn comes_from_the_publish(self) -> bool {
        match self {
            CapacityQuantity::PublishNanoseconds
            | CapacityQuantity::PublishPeakOverBaselineBytes => true,
            CapacityQuantity::AllocateNanoseconds
            | CapacityQuantity::AllocatePeakOverBaselineBytes
            | CapacityQuantity::AllocatorCopyBytes
            | CapacityQuantity::AllocatorCopyNanoseconds
            | CapacityQuantity::FlatControlPeakOverBaselineBytes => false,
        }
    }
}

impl SlopeGroup {
    /// 这一组格上的发布是不是登记的 X5：容量阶梯的首个文件一个数据单元，覆盖写它就是 X5；记录格与占用格的文件有 n 个单元，
    /// `UserChange` 里没有只改第 0 个单元以内 4096 字节的入口（登记第十二节修订附 grep 与零命中），整份 `Overwrite` 把文件缩成一个单元，不是 X5。
    fn publish_is_the_registered_step(self) -> bool {
        match self {
            SlopeGroup::CapacityLadder => true,
            SlopeGroup::RecordCells | SlopeGroup::OccupiedCapacityCells => false,
        }
    }
}

fn judge_slope_row(statuses: &CapacityStatuses, row: SlopeRow) -> SlopeRowJudgement {
    if row.quantity.comes_from_the_publish() && !row.group.publish_is_the_registered_step() {
        return SlopeRowJudgement {
            row,
            slopes: vec![None; usize::try_from(CAPACITY_CELL_ROUNDS).expect("5")],
            class: ClassAcrossRounds::RegisteredPublishCannotBeMade,
        };
    }
    let cells = row.group.cells();
    let slopes: Vec<Option<SlopeReading>> = (1..=CAPACITY_CELL_ROUNDS)
        .map(|round| slope_in_one_round(statuses, &cells, round, row.quantity, row.axis))
        .collect();
    let class = class_across_rounds(
        &slopes
            .iter()
            .map(|slope| slope.map(|reading| reading.slope))
            .collect::<Vec<_>>(),
        usize::try_from(CAPACITY_CELL_ROUNDS).expect("5"),
    );
    SlopeRowJudgement { row, slopes, class }
}

/// PC-线性 的拷贝字节与拷贝耗时、PC-平：5 轮每一轮的斜率都要在门槛那一侧（缺一轮也不算过）。
fn every_round_slope_satisfies(judgement: &SlopeRowJudgement, holds: fn(f64) -> bool) -> bool {
    judgement
        .slopes
        .iter()
        .all(|slope| slope.is_some_and(|reading| holds(reading.slope)))
}

/// A37：分配器拷贝字节 ÷ (2 × 单元区槽数) ∈ [3.5, 4.0]，按百万分之一比。
const A37_LOWER_MILLIONTHS: u128 = 3_500_000;
const A37_UPPER_MILLIONTHS: u128 = 4_000_000;

fn bytes_per_slot_per_device_millionths(copy_bytes: u64, device_bytes: u64) -> u128 {
    u128::from(copy_bytes) * 1_000_000
        / u128::from(POOL_DEVICE_COUNT * unit_area_slots(device_bytes, JOURNAL_RING_DEFAULT_BYTES))
}

fn a37_holds(millionths: u128) -> bool {
    (A37_LOWER_MILLIONTHS..=A37_UPPER_MILLIONTHS).contains(&millionths)
}

/// 一格跨轮比的确定性字段（T4；登记 R2d「确定性的量，5 轮应逐字段相同」）：哪几样不同。
fn fields_differing_across_rounds(readings: &[&CellRoundReading]) -> Vec<&'static str> {
    let Some(first) = readings.first() else {
        return Vec::new();
    };
    let allocate_peaks = |reading: &CellRoundReading| {
        reading
            .allocate
            .iter()
            .map(|call| call.window.peak_over_baseline_bytes)
            .collect::<Vec<_>>()
    };
    let allocate_slots = |reading: &CellRoundReading| {
        reading
            .allocate
            .iter()
            .map(|call| call.slots_from_unit_area_start)
            .collect::<Vec<_>>()
    };
    let publish_peaks = |reading: &CellRoundReading| {
        reading
            .publish
            .iter()
            .map(|call| call.window.peak_over_baseline_bytes)
            .collect::<Vec<_>>()
    };
    let mut differing = Vec::new();
    let mut note = |name: &'static str, equal: bool| {
        if !equal {
            differing.push(name);
        }
    };
    note(
        "records",
        readings
            .iter()
            .all(|reading| reading.records == first.records),
    );
    note(
        "copy_bytes",
        readings
            .iter()
            .all(|reading| reading.copy_bytes == first.copy_bytes),
    );
    note(
        "allocate_peaks",
        readings
            .iter()
            .all(|reading| allocate_peaks(reading) == allocate_peaks(first)),
    );
    note(
        "allocate_slots",
        readings
            .iter()
            .all(|reading| allocate_slots(reading) == allocate_slots(first)),
    );
    note(
        "publish_peaks",
        readings
            .iter()
            .all(|reading| publish_peaks(reading) == publish_peaks(first)),
    );
    note(
        "flat_peak",
        readings.iter().all(|reading| {
            reading.flat_peak_over_baseline_bytes == first.flat_peak_over_baseline_bytes
        }),
    );
    differing
}

fn joined_or_none<Item: ToString>(items: &[Item]) -> String {
    if items.is_empty() {
        return "none".to_string();
    }
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// R2f 的绝对值表与每格的 T4、A37（登记第六节 R2f、第七节 A37）；交回 T4 不过的格数。
fn emit_capacity_cell_values(emitter: &mut Emitter, statuses: &CapacityStatuses) -> usize {
    let mut hidden_state_cells = 0;
    for cell in capacity_cells() {
        let readings: Vec<&CellRoundReading> = (1..=CAPACITY_CELL_ROUNDS)
            .filter_map(|round| status_of(statuses, cell, round)?.reading())
            .collect();
        let mut fields = vec![
            format!("name=h2_cell_values cell={}", cell.name()),
            format!("device_bytes={}", cell.device_bytes),
            format!(
                "sequentially_written_units={}",
                cell.sequentially_written_units
            ),
            format!(
                "publish_is_the_registered_single_unit_overwrite={}",
                cell.sequentially_written_units == 0
            ),
            format!("rounds_measured={}", readings.len()),
            format!(
                "records={}",
                joined_or_none(
                    &readings
                        .iter()
                        .map(|reading| reading.records)
                        .collect::<Vec<_>>()
                )
            ),
        ];
        for quantity in CapacityQuantity::EVERY {
            let samples: Vec<u64> = readings
                .iter()
                .filter_map(|reading| quantity.samples(reading))
                .flatten()
                .collect();
            fields.push(format!(
                "{name}_median={} {name}_minimum={} {name}_maximum={}",
                optional_number_text(median_of(&samples)),
                optional_number_text(samples.iter().min().copied()),
                optional_number_text(samples.iter().max().copied()),
                name = quantity.name(),
            ));
        }
        let slots: BTreeSet<u64> = readings
            .iter()
            .flat_map(|reading| {
                reading
                    .allocate
                    .iter()
                    .filter_map(|call| call.slots_from_unit_area_start)
            })
            .collect();
        fields.push(format!(
            "slots_from_unit_area_start={}",
            joined_or_none(&slots.into_iter().collect::<Vec<_>>())
        ));
        let per_slot: Vec<u128> = readings
            .iter()
            .filter(|reading| reading.heap_control_holds)
            .map(|reading| {
                bytes_per_slot_per_device_millionths(reading.copy_bytes, cell.device_bytes)
            })
            .collect();
        fields.push(format!(
            "bytes_per_slot_per_device_millionths={} bytes_per_slot_per_device_within_the_inferred_interval={}",
            joined_or_none(&per_slot),
            !per_slot.is_empty() && per_slot.iter().all(|millionths| a37_holds(*millionths)),
        ));
        emitter.emit(&fields.join(" "));
        let differing = fields_differing_across_rounds(&readings);
        hidden_state_cells += usize::from(!differing.is_empty());
        emitter.emit(&format!(
            "name=h2_cell_determinism cell={} rounds_compared={} equal_across_rounds={} differing_fields={}",
            cell.name(),
            readings.len(),
            differing.is_empty(),
            joined_or_none(&differing),
        ));
    }
    hidden_state_cells
}

fn class_of_the_row<'judgements>(
    judgements: &'judgements [SlopeRowJudgement],
    row: &str,
) -> &'judgements ClassAcrossRounds {
    &judgements
        .iter()
        .find(|judgement| judgement.row.row == row)
        .expect("行名取自 capacity_slope_rows")
        .class
}

/// H2 合并的结局：T4 不过 ⇒ 停机（产物不算，两轮的片原样交回）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum MergeEnd {
    Complete,
    Stopped { reason: String },
}

/// E37 里 H2 的判定，接在每片的结局行之后（产物的后半）。
fn emit_capacity_summary(emitter: &mut Emitter, statuses: &CapacityStatuses) -> MergeEnd {
    for (cell, round, status) in statuses {
        emitter.emit(&format!(
            "name=h2_shard_status cell={} round={round} status={}",
            cell.name(),
            status.text()
        ));
    }
    let hidden_state_cells = emit_capacity_cell_values(emitter, statuses);
    let fully_measured = |cell: &CapacityCell| {
        (1..=CAPACITY_CELL_ROUNDS).all(|round| {
            status_of(statuses, *cell, round).is_some_and(|status| status.reading().is_some())
        })
    };
    let completed: Vec<u64> = ladder_cells()
        .iter()
        .filter(|cell| fully_measured(cell))
        .map(|cell| cell.device_bytes / MEBIBYTE)
        .collect();
    let beyond_the_upper_limit: Vec<u64> = ladder_cells()
        .iter()
        .filter(|cell| {
            (1..=CAPACITY_CELL_ROUNDS).any(|round| {
                matches!(
                    status_of(statuses, **cell, round),
                    Some(CellRoundStatus::BeyondTheMemoryUpperLimit(_))
                )
            })
        })
        .map(|cell| cell.device_bytes / MEBIBYTE)
        .collect();
    let shards_not_run_by_the_wrapper = statuses
        .iter()
        .filter(|(_, _, status)| matches!(status, CellRoundStatus::NotRunByTheWrapper(_)))
        .count();
    let ladder_endpoint =
        CAPACITY_LADDER_DEVICE_BYTES[CAPACITY_LADDER_DEVICE_BYTES.len() - 1] / MEBIBYTE;
    let fewer_than_four = completed.len() < 4;
    emitter.emit(&format!(
        "name=h2_largest_disk completed_capacities_mebibytes={} completed_count={} largest_completed_mebibytes={} largest_is_the_ladder_endpoint={} beyond_the_memory_upper_limit_mebibytes={} fewer_than_four_capacities={fewer_than_four}",
        joined_or_none(&completed),
        completed.len(),
        optional_number_text(completed.iter().max().copied()),
        completed.iter().max() == Some(&ladder_endpoint),
        joined_or_none(&beyond_the_upper_limit),
    ));
    let judgements: Vec<SlopeRowJudgement> = capacity_slope_rows()
        .into_iter()
        .map(|row| judge_slope_row(statuses, row))
        .collect();
    for judgement in &judgements {
        for (round_index, slope) in judgement.slopes.iter().enumerate() {
            emitter.emit(&format!(
                "name=h2_slope row={} quantity={} axis={} group={} round={} slope={} has_zero_point={} points={}",
                judgement.row.row,
                judgement.row.quantity.name(),
                judgement.row.axis.name(),
                judgement.row.group.name(),
                round_index + 1,
                slope_text(slope.map(|reading| reading.slope)),
                slope.is_some_and(|reading| reading.has_zero_point),
                slope.map_or(0, |reading| reading.points),
            ));
        }
        emitter.emit(&format!(
            "name=h2_class row={} quantity={} axis={} group={} class={}",
            judgement.row.row,
            judgement.row.quantity.name(),
            judgement.row.axis.name(),
            judgement.row.group.name(),
            judgement.class.text(),
        ));
    }
    let row_judgement = |row: &str| {
        judgements
            .iter()
            .find(|judgement| judgement.row.row == row)
            .expect("行名取自 capacity_slope_rows")
    };
    let copy_bytes_holds =
        every_round_slope_satisfies(row_judgement("r2c"), |slope| slope >= LINEAR_SLOPE_BOUNDARY);
    let copy_time_holds = every_round_slope_satisfies(
        row_judgement("positive_control_linear_copy_time"),
        |slope| slope >= LINEAR_SLOPE_BOUNDARY,
    );
    let flat_holds = every_round_slope_satisfies(row_judgement("positive_control_flat"), |slope| {
        slope < UNRELATED_SLOPE_BOUNDARY
    });
    emitter.emit(&format!(
        "name=positive_control_linear copy_bytes_slope_at_least_0_8_every_round={copy_bytes_holds} copy_time_slope_at_least_0_8_every_round={copy_time_holds}"
    ));
    emitter.emit(&format!(
        "name=positive_control_flat_slope slope_below_0_2_every_round={flat_holds}"
    ));
    for (baseline, occupied) in [("r2a", "r2a_at_occupancy"), ("r2b", "r2b_at_occupancy")] {
        let baseline_class = class_of_the_row(&judgements, baseline);
        let occupied_class = class_of_the_row(&judgements, occupied);
        let agreement = if *occupied_class == ClassAcrossRounds::RegisteredPublishCannotBeMade {
            "not_judged_x5_as_registered_cannot_be_made"
        } else if baseline_class == occupied_class {
            "two_points_agree"
        } else {
            "unstable_depends_on_occupancy"
        };
        emitter.emit(&format!(
            "name=h2_occupancy_sensitivity row={baseline} near_empty_class={} occupied_class={} agreement={agreement}",
            baseline_class.text(),
            occupied_class.text(),
        ));
    }
    let judged_slopes: Vec<f64> = judgements
        .iter()
        .filter(|judgement| CapacityQuantity::JUDGED.contains(&judgement.row.quantity))
        .flat_map(|judgement| {
            judgement
                .slopes
                .iter()
                .flatten()
                .map(|reading| reading.slope)
        })
        .collect();
    let moved = linear_boundary_moved_between_real_slopes(&judged_slopes);
    let synthetic = synthetic_slopes_fall_in_two_classes();
    let self_proof_holds = moved.is_some() || synthetic;
    emitter.emit(&format!(
        "name=discrimination_self_proof judgement=h2_slope_class real_slopes={} moved_boundary={} flipped={} synthetic_slopes_fall_in_two_classes={synthetic} used={} holds={self_proof_holds}",
        judged_slopes.len(),
        slope_text(moved.map(|boundary| boundary.boundary)),
        moved.map_or(0, |boundary| boundary.flipped),
        if moved.is_some() { "real_slopes" } else { "synthetic_slopes" },
    ));
    emit_capacity_verdict(
        emitter,
        statuses,
        &judgements,
        CapacityVerdictInputs {
            completed_count: completed.len(),
            fewer_than_four,
            copy_bytes_holds,
            copy_time_holds,
            flat_holds,
            self_proof_holds,
            hidden_state_cells,
            shards_not_run_by_the_wrapper,
        },
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CapacityVerdictInputs {
    completed_count: usize,
    fewer_than_four: bool,
    copy_bytes_holds: bool,
    copy_time_holds: bool,
    flat_holds: bool,
    self_proof_holds: bool,
    hidden_state_cells: usize,
    /// 包装退 251–254 没让它跑完的片数（不是撞了自己的上限；要重跑，登记第十二节修订）。
    shards_not_run_by_the_wrapper: usize,
}

/// H2 的判决行：每个判定各占一个字段，不合取（登记第六节开头）；G1、G2（第四节 Q8 推的与量出的不同）、G6、V30、V32、T4 各报各的。
fn emit_capacity_verdict(
    emitter: &mut Emitter,
    statuses: &CapacityStatuses,
    judgements: &[SlopeRowJudgement],
    inputs: CapacityVerdictInputs,
) -> MergeEnd {
    let class = |row: &str| class_of_the_row(judgements, row);
    let unstable_rows: Vec<&str> = ["r2a", "r2b", "r2e_allocate", "r2e_publish"]
        .into_iter()
        .filter(|row| *class(row) == ClassAcrossRounds::Unstable)
        .collect();
    let ladder_readings: Vec<(CapacityCell, &CellRoundReading)> = statuses
        .iter()
        .filter(|(cell, _, _)| cell.sequentially_written_units == 0)
        .filter_map(|(cell, _, status)| Some((*cell, status.reading()?)))
        .collect();
    let a37_outside = ladder_readings.iter().any(|(cell, reading)| {
        reading.heap_control_holds
            && !a37_holds(bytes_per_slot_per_device_millionths(
                reading.copy_bytes,
                cell.device_bytes,
            ))
    });
    let mut contradicted: Vec<&str> = Vec::new();
    if *class("r2c") != ClassAcrossRounds::Stable(SlopeClass::Linear) || a37_outside {
        contradicted.push("r2c_or_a37");
    }
    if *class("r2a") != ClassAcrossRounds::Stable(SlopeClass::BasicallyUnrelated) {
        contradicted.push("r2a_not_basically_unrelated");
    }
    if *class("r2e_allocate") != ClassAcrossRounds::Stable(SlopeClass::Linear) {
        contradicted.push("r2e_allocate_not_linear");
    }
    let heap_control_failed_shards = statuses
        .iter()
        .filter(|(_, _, status)| {
            status
                .reading()
                .is_some_and(|reading| !reading.heap_control_holds)
        })
        .count();
    emitter.emit(&format!(
        "name=verdict item=h2_summary r2a_class={} r2b_class={} r2d_allocate_class={} r2d_publish_class={} r2e_allocate_class={} r2e_publish_class={} r2e_allocate_peak_class={} r2e_publish_peak_class={} r2c_class={} completed_capacities={} fewer_than_four_capacities={} positive_control_linear_copy_bytes_holds={} positive_control_linear_copy_time_holds={} positive_control_flat_holds={} heap_control_failed_shards={heap_control_failed_shards} unstable_slope_rows={} predictions_contradicted={} discrimination_self_proof_holds={} hidden_state_cells={} shards_not_run_by_the_wrapper={}",
        class("r2a").text(),
        class("r2b").text(),
        class("r2d_allocate").text(),
        class("r2d_publish").text(),
        class("r2e_allocate").text(),
        class("r2e_publish").text(),
        class("r2e_allocate_peak").text(),
        class("r2e_publish_peak").text(),
        class("r2c").text(),
        inputs.completed_count,
        inputs.fewer_than_four,
        inputs.copy_bytes_holds,
        inputs.copy_time_holds,
        inputs.flat_holds,
        joined_or_none(&unstable_rows),
        joined_or_none(&contradicted),
        inputs.self_proof_holds,
        inputs.hidden_state_cells,
        inputs.shards_not_run_by_the_wrapper,
    ));
    if inputs.hidden_state_cells > 0 {
        return MergeEnd::Stopped {
            reason: "t4_deterministic_fields_differ_across_rounds".to_string(),
        };
    }
    MergeEnd::Complete
}

/// 没做完的一片的结局，只看跑法脚本写下的包装退出码。
fn status_of_an_unfinished_shard(wrapper_exit: Option<u64>) -> CellRoundStatus {
    match wrapper_exit {
        Some(exit_code) if exit_code == WRAPPER_EXIT_OF_HITTING_ITS_OWN_MEMORY_LIMIT => {
            CellRoundStatus::BeyondTheMemoryUpperLimit(exit_code)
        }
        Some(exit_code) if WRAPPER_EXITS_OF_A_SHARD_THAT_WAS_NOT_RUN.contains(&exit_code) => {
            CellRoundStatus::NotRunByTheWrapper(exit_code)
        }
        Some(_) | None => CellRoundStatus::Missing,
    }
}

/// 读回片目录里的 45 片：做完的读行；不在或没做完的看跑法脚本写的 `<片名>.wrapper-exit`（内存包装的退出码）：250 记「内存上限之外」，251–254 记「包装没让它跑完」。
fn read_capacity_statuses(directory: &Path, binary_sha256: &str) -> CapacityStatuses {
    capacity_shards()
        .into_iter()
        .map(|(shard, cell, round)| {
            let text = std::fs::read_to_string(directory.join(&shard)).ok();
            let finished = text
                .as_deref()
                .and_then(|text| lines_of_a_finished_shard(text, &shard));
            let status = match finished {
                Some(lines) => {
                    let parsed: Vec<ResultLine> = lines
                        .iter()
                        .filter_map(|line| ResultLine::parse(line))
                        .collect();
                    status_of_a_capacity_shard(&parsed, binary_sha256)
                }
                None => {
                    let wrapper_exit =
                        std::fs::read_to_string(directory.join(format!("{shard}.wrapper-exit")))
                            .ok()
                            .and_then(|exit_text| exit_text.trim().parse::<u64>().ok());
                    status_of_an_unfinished_shard(wrapper_exit)
                }
            };
            (cell, round, status)
        })
        .collect()
}

// ============================================================================================
// E36：H5 的清扫历史（登记 X16–X21、5.2、5.3）。4 GiB × 2、L = 768 MiB、内存稀疏盘；一格一片。
// ============================================================================================

/// 阈值候选（登记 X17）。
const SWEEP_THRESHOLD_CANDIDATE_ROWS: [u64; 4] = [64, 365, 369, 738];
/// 「交给后台」对照（登记 5.2）：S = 8、正常卸载、400 次挂载。
const SWEEP_CONTROL_SLOTS_PER_REGION: u64 = 8;
const SWEEP_CONTROL_MOUNTS: u64 = 400;
/// 读法甲（登记 X19）：链重写那一项取一片 ⟺ rows0 ≤ 366。
const ROWS_KEPT_ON_ONE_CHAIN_PAGE: u64 = 366;
/// 判别力自证与 R5c 的门槛：W_θ ÷ 四个 θ 里最小的 W ≤ 1.10（登记第六节 R5c），按千分比。
const WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE: u128 = 1100;
/// 清扫历史结束于第二次清扫发布（登记 X16）。
const SWEEPS_TO_FINISH_A_HISTORY: usize = 2;
/// 根环区域落在哪块盘上：[0, 1, 0]（登记 X1，同 `make_filesystem_parameters` 的 `region_devices`）。
const ROOT_RING_REGION_DEVICES: [u64; 3] = [0, 1, 0];
/// 锚点模型里 mkfs 那个进程占的 txg：mkfs 0、暖机 1 与 2、首个文件 3，都是实例 1（`anchors_e183.py` 的 `owner`）。
const MAKE_FILESYSTEM_PROCESS_LAST_TXG: u64 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnmountKind {
    Normal,
    Crash,
}

impl UnmountKind {
    fn name(self) -> &'static str {
        match self {
            UnmountKind::Normal => "normal_unmount",
            UnmountKind::Crash => "crash",
        }
    }
}

/// 清扫在挂载末尾怎么走：开关关着、阈值 θ（四条臂）；开关开着、交给后台（对照臂）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SweepArm {
    AtTheEndOfTheMountAboveTheThreshold { threshold_rows: u64 },
    HandedToTheBackground,
}

impl SweepArm {
    fn configuration(self) -> InstanceTableSweepConfiguration {
        match self {
            SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows } => {
                InstanceTableSweepConfiguration {
                    switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
                    row_count_threshold: InstanceTableRowCountThreshold(threshold_rows),
                    sweep_publish_placements:
                        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
                }
            }
            SweepArm::HandedToTheBackground => InstanceTableSweepConfiguration {
                switch: InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted,
                row_count_threshold: InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690,
                sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
            },
        }
    }

    fn threshold_rows(self) -> Option<u64> {
        match self {
            SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows } => {
                Some(threshold_rows)
            }
            SweepArm::HandedToTheBackground => None,
        }
    }
}

/// K33 的装置侧判法：开关关着时，发了清扫发布 ⟺ 那一次 rows0 > θ。
fn sweep_is_due(rows0: u64, threshold_rows: u64) -> bool {
    rows0 > threshold_rows
}

/// 一格：S、清扫臂、卸载方式、挂载次数上限（四条臂 2θ + 3S + 20，对照 400）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SweepHistoryCell {
    slots_per_region: u64,
    arm: SweepArm,
    unmount_kind: UnmountKind,
    mounts_at_most: u64,
}

impl SweepHistoryCell {
    fn name(self) -> String {
        match self.arm {
            SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows } => format!(
                "h5-slots-per-region-{}-threshold-{threshold_rows}-{}",
                self.slots_per_region,
                self.unmount_kind.name()
            ),
            SweepArm::HandedToTheBackground => format!(
                "h5-control-slots-per-region-{}-handed-to-the-background-{}",
                self.slots_per_region,
                self.unmount_kind.name()
            ),
        }
    }
}

/// 24 格（次序同锚点脚本：S 在外、θ、卸载方式）加对照 1 格。
fn sweep_history_cells() -> Vec<SweepHistoryCell> {
    let mut cells = Vec::new();
    for slots_per_region in SLOTS_PER_REGION_SAMPLING_POINTS {
        for threshold_rows in SWEEP_THRESHOLD_CANDIDATE_ROWS {
            for unmount_kind in [UnmountKind::Normal, UnmountKind::Crash] {
                cells.push(SweepHistoryCell {
                    slots_per_region,
                    arm: SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows },
                    unmount_kind,
                    mounts_at_most: 2 * threshold_rows + ROOT_RING_REGIONS * slots_per_region + 20,
                });
            }
        }
    }
    cells.push(SweepHistoryCell {
        slots_per_region: SWEEP_CONTROL_SLOTS_PER_REGION,
        arm: SweepArm::HandedToTheBackground,
        unmount_kind: UnmountKind::Normal,
        mounts_at_most: SWEEP_CONTROL_MOUNTS,
    });
    cells
}

/// 确定性核（登记第五节「段首那一格多跑 1 轮」）：第一格再跑一片，片名前缀 `h5-repeat-of-`。
const SWEEP_REPEAT_SHARD_PREFIX: &str = "h5-repeat-of-";

/// 全部片：25 格各一片，加段首那一格的重跑一片。
fn sweep_shards() -> Vec<(String, SweepHistoryCell)> {
    let cells = sweep_history_cells();
    let mut shards: Vec<(String, SweepHistoryCell)> =
        cells.iter().map(|cell| (cell.name(), *cell)).collect();
    shards.push((
        format!("{SWEEP_REPEAT_SHARD_PREFIX}{}", cells[0].name()),
        cells[0],
    ));
    shards
}

/// 锚点模型（`anchors_e183.py` 的 `h5`，登记第十三节命令一，本地移植；与登记那 24 行逐项比，`REGISTERED_SWEEP_HISTORY_ANCHORS`）里的一次挂载。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AnchorModelMount {
    rows0: u64,
    /// 这次挂载发了清扫：删了几行、删完剩几行。
    sweep: Option<(u64, u64)>,
    /// 这次挂载交回时（清扫发布之后、卸载之前）现行那一版的 txg。
    checkpoint_txg_when_the_mount_returns: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AnchorModelHistory {
    mounts: Vec<AnchorModelMount>,
    final_txg: u64,
}

impl AnchorModelHistory {
    fn sweeps(&self) -> Vec<(u64, u64, u64)> {
        self.mounts
            .iter()
            .zip(1u64..)
            .filter_map(|(mount, number)| {
                mount
                    .sweep
                    .map(|(deleted, residual)| (number, deleted, residual))
            })
            .collect()
    }

    fn peak_rows0(&self) -> u64 {
        self.mounts
            .iter()
            .map(|mount| mount.rows0)
            .max()
            .unwrap_or(0)
    }
}

/// 移植 `anchors_e183.py` 的 `h5`：每次可写挂载取新实例号、写上一个实例那一行、暖机到两块盘上各有一条本实例的根（txg u 的根落盘
/// `ROOT_RING_REGION_DEVICES[u % 3]`）；行数 > θ 就清扫，删去根环（最近 3S 个 txg）里一条根都没有的实例的行，清扫发布占一个 txg；
/// 正常卸载推空发布：现行 txg 除以 3 余 1 时 3 次、否则 2 次。`threshold_rows` 为 `None` 是「交给后台」（从不清扫）。
fn sweep_history_by_the_anchor_model(
    slots_per_region: u64,
    threshold_rows: Option<u64>,
    unmount_kind: UnmountKind,
    mounts_at_most: u64,
) -> AnchorModelHistory {
    let ring_roots = ROOT_RING_REGIONS * slots_per_region;
    let mut owner: BTreeMap<u64, u64> = (0..=MAKE_FILESYSTEM_PROCESS_LAST_TXG)
        .map(|txg| (txg, 1))
        .collect();
    let mut txg = MAKE_FILESYSTEM_PROCESS_LAST_TXG;
    let mut instance = 1u64;
    let push_empty_publishes_of_an_unmount =
        |current_txg: &mut u64, owners: &mut BTreeMap<u64, u64>, unmounting_instance: u64| {
            let publishes = if *current_txg % 3 == 1 { 3 } else { 2 };
            for _ in 0..publishes {
                *current_txg += 1;
                owners.insert(*current_txg, unmounting_instance);
            }
        };
    push_empty_publishes_of_an_unmount(&mut txg, &mut owner, instance);
    let mut rows: BTreeSet<u64> = BTreeSet::new();
    let mut mounts = Vec::new();
    let mut sweeps_done = 0;
    for _ in 0..mounts_at_most {
        instance += 1;
        rows.insert(instance - 1);
        txg += 1;
        owner.insert(txg, instance);
        let mut covered: BTreeSet<u64> =
            BTreeSet::from([ROOT_RING_REGION_DEVICES[usize::try_from(txg % 3).expect("小于 3")]]);
        while covered.len() < 2 {
            txg += 1;
            owner.insert(txg, instance);
            covered.insert(ROOT_RING_REGION_DEVICES[usize::try_from(txg % 3).expect("小于 3")]);
        }
        let rows0 = u64::try_from(rows.len()).expect("行数装得进 u64");
        let mut sweep = None;
        if threshold_rows.is_some_and(|threshold| sweep_is_due(rows0, threshold)) {
            let first_in_the_ring = (txg + 1).saturating_sub(ring_roots);
            let in_the_ring: BTreeSet<u64> = (first_in_the_ring..=txg)
                .map(|ring_txg| owner[&ring_txg])
                .collect();
            let deletable: BTreeSet<u64> = rows.difference(&in_the_ring).copied().collect();
            rows.retain(|row| !deletable.contains(row));
            txg += 1;
            owner.insert(txg, instance);
            sweep = Some((
                u64::try_from(deletable.len()).expect("行数"),
                u64::try_from(rows.len()).expect("行数"),
            ));
            sweeps_done += 1;
        }
        mounts.push(AnchorModelMount {
            rows0,
            sweep,
            checkpoint_txg_when_the_mount_returns: txg,
        });
        if sweeps_done == SWEEPS_TO_FINISH_A_HISTORY {
            break;
        }
        match unmount_kind {
            UnmountKind::Normal => {
                push_empty_publishes_of_an_unmount(&mut txg, &mut owner, instance)
            }
            UnmountKind::Crash => {}
        }
    }
    AnchorModelHistory {
        mounts,
        final_txg: txg,
    }
}

/// 登记第十三节命令一 H5 那 24 行（原样的数）：S、θ、卸载方式、挂了几次、两次清扫各删几行、各剩几行、在第几次挂载、rows0 峰值、片数峰值、末了的 txg。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RegisteredSweepHistoryAnchor {
    slots_per_region: u64,
    threshold_rows: u64,
    unmount_kind: UnmountKind,
    mounts: u64,
    rows_deleted: [u64; 2],
    rows_left: [u64; 2],
    mounts_at_sweep: [u64; 2],
    peak_rows0: u64,
    peak_pages: u64,
    final_txg: u64,
}

impl RegisteredSweepHistoryAnchor {
    fn is_reproduced_by(&self, history: &AnchorModelHistory) -> bool {
        let sweeps = history.sweeps();
        sweeps.len() == 2
            && u64::try_from(history.mounts.len()).ok() == Some(self.mounts)
            && sweeps
                .iter()
                .map(|(_, deleted, _)| *deleted)
                .collect::<Vec<_>>()
                == self.rows_deleted
            && sweeps.iter().map(|(_, _, left)| *left).collect::<Vec<_>>() == self.rows_left
            && sweeps
                .iter()
                .map(|(mount, _, _)| *mount)
                .collect::<Vec<_>>()
                == self.mounts_at_sweep
            && history.peak_rows0() == self.peak_rows0
            && instance_table_chain_pages(history.peak_rows0()) == self.peak_pages
            && history.final_txg == self.final_txg
    }
}

/// 登记第十三节命令一的原样输出（`anchors_e183.py` H5 那 24 行）；`sweep_history_by_the_anchor_model` 必须逐行重现（停机 T3 的同类核）。
const REGISTERED_SWEEP_HISTORY_ANCHORS: [RegisteredSweepHistoryAnchor; 24] = [
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Normal,
        mounts: 128,
        rows_deleted: [63, 63],
        rows_left: [2, 2],
        mounts_at_sweep: [65, 128],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 770,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Crash,
        mounts: 127,
        rows_deleted: [62, 62],
        rows_left: [3, 3],
        mounts_at_sweep: [65, 127],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 386,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Normal,
        mounts: 730,
        rows_deleted: [364, 364],
        rows_left: [2, 2],
        mounts_at_sweep: [366, 730],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 4382,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Crash,
        mounts: 729,
        rows_deleted: [363, 363],
        rows_left: [3, 3],
        mounts_at_sweep: [366, 729],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 2192,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Normal,
        mounts: 738,
        rows_deleted: [368, 368],
        rows_left: [2, 2],
        mounts_at_sweep: [370, 738],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 4430,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Crash,
        mounts: 737,
        rows_deleted: [367, 367],
        rows_left: [3, 3],
        mounts_at_sweep: [370, 737],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 2216,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Normal,
        mounts: 1476,
        rows_deleted: [737, 737],
        rows_left: [2, 2],
        mounts_at_sweep: [739, 1476],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 8858,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 4,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Crash,
        mounts: 1475,
        rows_deleted: [736, 736],
        rows_left: [3, 3],
        mounts_at_sweep: [739, 1475],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 4430,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Normal,
        mounts: 126,
        rows_deleted: [61, 61],
        rows_left: [4, 4],
        mounts_at_sweep: [65, 126],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 758,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Crash,
        mounts: 123,
        rows_deleted: [58, 58],
        rows_left: [7, 7],
        mounts_at_sweep: [65, 123],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 374,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Normal,
        mounts: 728,
        rows_deleted: [362, 362],
        rows_left: [4, 4],
        mounts_at_sweep: [366, 728],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 4370,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Crash,
        mounts: 725,
        rows_deleted: [359, 359],
        rows_left: [7, 7],
        mounts_at_sweep: [366, 725],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 2180,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Normal,
        mounts: 736,
        rows_deleted: [366, 366],
        rows_left: [4, 4],
        mounts_at_sweep: [370, 736],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 4418,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Crash,
        mounts: 733,
        rows_deleted: [363, 363],
        rows_left: [7, 7],
        mounts_at_sweep: [370, 733],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 2204,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Normal,
        mounts: 1474,
        rows_deleted: [735, 735],
        rows_left: [4, 4],
        mounts_at_sweep: [739, 1474],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 8846,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 8,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Crash,
        mounts: 1471,
        rows_deleted: [732, 732],
        rows_left: [7, 7],
        mounts_at_sweep: [739, 1471],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 4418,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Normal,
        mounts: 122,
        rows_deleted: [57, 57],
        rows_left: [8, 8],
        mounts_at_sweep: [65, 122],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 734,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 64,
        unmount_kind: UnmountKind::Crash,
        mounts: 115,
        rows_deleted: [50, 50],
        rows_left: [15, 15],
        mounts_at_sweep: [65, 115],
        peak_rows0: 65,
        peak_pages: 1,
        final_txg: 350,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Normal,
        mounts: 724,
        rows_deleted: [358, 358],
        rows_left: [8, 8],
        mounts_at_sweep: [366, 724],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 4346,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 365,
        unmount_kind: UnmountKind::Crash,
        mounts: 717,
        rows_deleted: [351, 351],
        rows_left: [15, 15],
        mounts_at_sweep: [366, 717],
        peak_rows0: 366,
        peak_pages: 1,
        final_txg: 2156,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Normal,
        mounts: 732,
        rows_deleted: [362, 362],
        rows_left: [8, 8],
        mounts_at_sweep: [370, 732],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 4394,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 369,
        unmount_kind: UnmountKind::Crash,
        mounts: 725,
        rows_deleted: [355, 355],
        rows_left: [15, 15],
        mounts_at_sweep: [370, 725],
        peak_rows0: 370,
        peak_pages: 2,
        final_txg: 2180,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Normal,
        mounts: 1470,
        rows_deleted: [731, 731],
        rows_left: [8, 8],
        mounts_at_sweep: [739, 1470],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 8822,
    },
    RegisteredSweepHistoryAnchor {
        slots_per_region: 16,
        threshold_rows: 738,
        unmount_kind: UnmountKind::Crash,
        mounts: 1463,
        rows_deleted: [724, 724],
        rows_left: [15, 15],
        mounts_at_sweep: [739, 1463],
        peak_rows0: 739,
        peak_pages: 3,
        final_txg: 4394,
    },
];

/// 两块盘计数里全部区间的写字节与读字节之和（X21 的「设备写字节」「读字节」）。
fn written_and_read_bytes(tallies: &[DeviceTally]) -> (u64, u64) {
    tallies.iter().fold((0, 0), |(written, read), tally| {
        let regions = [
            tally.system_configuration,
            tally.root_ring,
            tally.journal_ring,
            tally.unit_area,
        ];
        (
            written
                + regions
                    .iter()
                    .map(|region| region.writes.bytes)
                    .sum::<u64>(),
            read + regions.iter().map(|region| region.reads.bytes).sum::<u64>(),
        )
    })
}

/// 同一份盘面的一份拷贝（X21：两次挂载各从同一份镜像副本起）：盘面照抄，计数从 0 起。
fn copy_of_the_pool(pool: &CountedPool, device_bytes: u64) -> CountedPool {
    pool.iter()
        .map(|(identity, device)| {
            let mut copied = SparseBlockDevice::new(
                device_bytes,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            );
            copied.image = device.inner.image.clone();
            (*identity, CountingBlockDevice::new(copied, device.regions))
        })
        .collect()
}

/// 根环里读得出的每条根：(区域, 槽, txg, 实例)，按根环槽的次序。
fn ring_roots_of(
    pool: &CountedPool,
    parameters: &MakeFilesystemParameters,
) -> Vec<(u64, u64, u64, u32)> {
    readable_roots_with_ring_slots(
        pool,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .into_iter()
    .map(|(ring_slot, root)| {
        (
            ring_slot.region,
            ring_slot.slot,
            root.checkpoint_txg.0,
            root.instance.0,
        )
    })
    .collect()
}

/// 每次读根环，把读到的 txg → 实例记下来（K34 要「清扫判的那一刻根环里有哪几个实例」：那一刻的环 = 清扫发布之前最近 3S 个 txg 的根）。
fn note_ring_owners(owner_by_txg: &mut BTreeMap<u64, u32>, roots: &[(u64, u64, u64, u32)]) {
    for (_, _, txg, instance) in roots {
        owner_by_txg.insert(*txg, *instance);
    }
}

/// 清扫判的那一刻根环里有根的实例：清扫发布的 txg 是 `sweep_txg`，环里是它之前的 3S 个 txg；有一个 txg 没读到过交回 `None`（K34 判不了）。
fn instances_in_the_ring_when_the_sweep_judges(
    owner_by_txg: &BTreeMap<u64, u32>,
    sweep_txg: u64,
    ring_roots: u64,
) -> Option<BTreeSet<u32>> {
    (sweep_txg.saturating_sub(ring_roots)..sweep_txg)
        .map(|txg| owner_by_txg.get(&txg).copied())
        .collect()
}

/// K34 的装置侧判法：该删的行 = 清扫之前的行里，实例在那一刻根环里一条根都没有的那些。
fn rows_the_sweep_should_delete(
    rows_before: &[u32],
    instances_in_the_ring: &BTreeSet<u32>,
) -> BTreeSet<u32> {
    rows_before
        .iter()
        .filter(|instance| !instances_in_the_ring.contains(instance))
        .copied()
        .collect()
}

/// 一版的实例表里有哪几个实例的行；读不出交回 `None`。
fn instance_rows_of(
    pool: &CountedPool,
    root: &singlefs_core::root_record::RootRecord,
) -> Option<Vec<u32>> {
    instance_table_chain_of_root(pool, root).ok().map(|chain| {
        chain
            .records
            .rows
            .iter()
            .map(|row| row.instance.0)
            .collect()
    })
}

/// 挂载末尾那一判清扫的结局，按装置要判的几样拆开。
#[derive(Clone, Debug, PartialEq, Eq)]
struct SweepAtTheMount {
    text: String,
    /// 清扫发布发成了（删了行）：K33 的「发了清扫发布」。
    rows_deleted: Option<Vec<u32>>,
    heads_erased: usize,
    handed_to_the_background: bool,
}

fn sweep_at_the_mount(outcome: &InstanceTableSweepAtTheEndOfTheMount) -> SweepAtTheMount {
    let not_published = |text: String, heads_erased: usize| SweepAtTheMount {
        text,
        rows_deleted: None,
        heads_erased,
        handed_to_the_background: false,
    };
    match outcome {
        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. } => {
            not_published("not_swept_at_or_below_the_threshold".to_string(), 0)
        }
        InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted => {
            SweepAtTheMount {
                text: "handed_to_the_background".to_string(),
                rows_deleted: None,
                heads_erased: 0,
                handed_to_the_background: true,
            }
        }
        InstanceTableSweepAtTheEndOfTheMount::Swept(swept) => match swept {
            InstanceTableSweepOutcome::RowsDeleted {
                rows_deleted,
                heads_erased,
            } => SweepAtTheMount {
                text: "rows_deleted".to_string(),
                rows_deleted: Some(rows_deleted.iter().map(|row| row.instance.0).collect()),
                heads_erased: heads_erased.len(),
                handed_to_the_background: false,
            },
            InstanceTableSweepOutcome::NoRowDeleted(reason) => not_published(
                format!("no_row_deleted:{}", variant_name_of(&format!("{reason:?}"))),
                0,
            ),
            InstanceTableSweepOutcome::SweepPublishRefusedForSpace { heads_erased, .. } => {
                not_published(
                    "sweep_publish_refused_for_space".to_string(),
                    heads_erased.len(),
                )
            }
            InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
                heads_erased,
                ..
            } => not_published(
                "stopped_by_a_block_device_error_while_erasing".to_string(),
                heads_erased.len(),
            ),
            InstanceTableSweepOutcome::SweepPublishFailed { heads_erased, .. } => {
                not_published("sweep_publish_failed".to_string(), heads_erased.len())
            }
        },
    }
}

/// 一格历史的公共前缀：mkfs → 可写挂载 → 首个文件 →（记下 W_θ 的起点计数）→ 正常卸载（登记 X16）。交回 W_θ 起点那一刻的计数。
fn sweep_history_prefix(
    parameters: &MakeFilesystemParameters,
    pool: &mut CountedPool,
    configuration: InstanceTableSweepConfiguration,
) -> Result<Vec<DeviceTally>, String> {
    guarded(|| {
        make_filesystem(parameters, pool)
            .map(|_| ())
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("make_filesystem"))?;
    let mounted = guarded(|| {
        mount_writable_with_instance_table_sweep_configuration(
            parameters,
            pool,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            configuration,
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("first_mount"))?;
    let Mounted {
        output,
        mut allocator,
        current,
    } = mounted;
    let first_file = first_file_content();
    let first = guarded(|| {
        let mut writer = PoolWriter::new(parameters, pool.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &first_file,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            output.instance,
            current.record_bytes(),
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("first_file"))?;
    let mut version_with_the_first_file = PoolVersion::of_a_version_of_trees(first);
    let tallies_after_the_first_file = tallies_of(pool);
    guarded(|| {
        unmount(
            parameters,
            pool,
            &mut allocator,
            &mut version_with_the_first_file,
            output.shadow_ledger,
        )
        .map(|_| ())
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("prefix_unmount"))?;
    Ok(tallies_after_the_first_file)
}

/// 一次挂载的读数（R5a 那一行、K33、K34、A36、与锚点模型逐次比）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct HistoryMount {
    number: u64,
    instance: u32,
    rows_written: u64,
    rows0: u64,
    rows0_by_the_allocator: u64,
    rows_before_the_sweep: Vec<u32>,
    sweep: SweepAtTheMount,
    rows_left: Option<u64>,
    checkpoint_txg: u64,
    mount_nanoseconds: u64,
    written_and_read_bytes: (u64, u64),
}

/// 这次挂载对得上的几样：`None` 是这一格不适用（没有清扫、对照臂）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HistoryMountChecks {
    sweep_matches_the_threshold: bool,
    deleted_rows_match_the_ring: Option<bool>,
    no_head_erased: Option<bool>,
    matches_the_model: bool,
}

impl HistoryMountChecks {
    fn all_hold(&self) -> bool {
        self.sweep_matches_the_threshold
            && self.deleted_rows_match_the_ring != Some(false)
            && self.no_head_erased != Some(false)
            && self.matches_the_model
    }

    fn failed_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if !self.sweep_matches_the_threshold {
            names.push("sweep_matches_the_threshold");
        }
        if self.deleted_rows_match_the_ring == Some(false) {
            names.push("deleted_rows_match_the_ring");
        }
        if self.no_head_erased == Some(false) {
            names.push("no_head_erased");
        }
        if !self.matches_the_model {
            names.push("a34_a35_trajectory");
        }
        names
    }
}

fn checks_of_a_history_mount(
    mount: &HistoryMount,
    arm: SweepArm,
    model: Option<&AnchorModelMount>,
    instances_in_the_ring: Option<&BTreeSet<u32>>,
) -> HistoryMountChecks {
    let sweep_matches_the_threshold = match arm.threshold_rows() {
        Some(threshold) => {
            mount.sweep.rows_deleted.is_some() == sweep_is_due(mount.rows0, threshold)
        }
        None => mount.sweep.handed_to_the_background,
    };
    let deleted_rows_match_the_ring = mount.sweep.rows_deleted.as_ref().map(|deleted| {
        instances_in_the_ring.is_some_and(|in_the_ring| {
            rows_the_sweep_should_delete(&mount.rows_before_the_sweep, in_the_ring)
                == deleted.iter().copied().collect::<BTreeSet<u32>>()
        })
    });
    let no_head_erased = mount
        .sweep
        .rows_deleted
        .as_ref()
        .map(|_| mount.sweep.heads_erased == 0);
    let actual_sweep = mount.sweep.rows_deleted.as_ref().map(|deleted| {
        (
            u64::try_from(deleted.len()).expect("行数"),
            mount.rows_left.unwrap_or(u64::MAX),
        )
    });
    let matches_the_model = model
        .is_some_and(|expected| expected.rows0 == mount.rows0 && expected.sweep == actual_sweep);
    HistoryMountChecks {
        sweep_matches_the_threshold,
        deleted_rows_match_the_ring,
        no_head_erased,
        matches_the_model,
    }
}

/// 一次可写挂载（清扫配置照这一格的臂）连同它的读数；挂载没成交回 `Err`。
fn mount_once_in_the_history(
    parameters: &MakeFilesystemParameters,
    pool: &mut CountedPool,
    configuration: InstanceTableSweepConfiguration,
    number: u64,
) -> Result<(Mounted, HistoryMount), String> {
    let tallies_before = tallies_of(pool);
    let started = Instant::now();
    let mounted = guarded(|| {
        mount_writable_with_instance_table_sweep_configuration(
            parameters,
            pool,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            configuration,
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })?;
    let mount_nanoseconds = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let written_and_read = written_and_read_bytes(&tally_difference(pool, &tallies_before));
    let rows_before_the_sweep = instance_rows_of(pool, mounted.output.row_publish.root())
        .ok_or("row_publish_table_unreadable")?;
    let sweep = sweep_at_the_mount(&mounted.output.instance_table_sweep);
    let rows_left = sweep
        .rows_deleted
        .as_ref()
        .and_then(|_| instance_rows_of(pool, mounted.current.root()))
        .map(|rows| u64::try_from(rows.len()).expect("行数"));
    let mount = HistoryMount {
        number,
        instance: mounted.output.instance.0,
        rows_written: u64::try_from(mounted.output.rows_written.len()).expect("行数"),
        rows0: u64::try_from(rows_before_the_sweep.len()).expect("行数"),
        rows0_by_the_allocator: mounted
            .allocator
            .instance_rows_after_this_mounts_row_publish(),
        rows_before_the_sweep,
        sweep,
        rows_left,
        checkpoint_txg: mounted.current.root().checkpoint_txg.0,
        mount_nanoseconds,
        written_and_read_bytes: written_and_read,
    };
    Ok((mounted, mount))
}

fn optional_truth_text(value: Option<bool>) -> String {
    value.map_or_else(|| "not_applicable".to_string(), |truth| truth.to_string())
}

fn emit_history_mount(
    emitter: &mut Emitter,
    shard: &str,
    mount: &HistoryMount,
    model: Option<&AnchorModelMount>,
    checks: HistoryMountChecks,
) {
    emitter.emit(&format!(
        "name=h5_mount shard={shard} mount={} instance={} rows_at_mount={} rows0={} rows0_by_the_allocator={} model_rows0={} chain_pages={} sweep={} rows_deleted={} rows_left={} heads_erased={} checkpoint_txg={} model_checkpoint_txg={} sweep_matches_the_threshold={} deleted_rows_match_the_ring={} no_head_erased={} trajectory_matches_the_model={}",
        mount.number,
        mount.instance,
        mount.rows0.saturating_sub(mount.rows_written),
        mount.rows0,
        mount.rows0_by_the_allocator,
        optional_number_text(model.map(|expected| expected.rows0)),
        instance_table_chain_pages(mount.rows0),
        mount.sweep.text,
        mount.sweep.rows_deleted.as_ref().map_or_else(|| "none".to_string(), |deleted| deleted.len().to_string()),
        optional_number_text(mount.rows_left),
        mount.sweep.heads_erased,
        mount.checkpoint_txg,
        optional_number_text(model.map(|expected| expected.checkpoint_txg_when_the_mount_returns)),
        checks.sweep_matches_the_threshold,
        optional_truth_text(checks.deleted_rows_match_the_ring),
        optional_truth_text(checks.no_head_erased),
        checks.matches_the_model,
    ));
}

/// T5 停机：把那一刻根环每槽的 (txg, 实例)、实例表的行、清扫结局原样写出（登记第十一节 T5「处置」）。
fn emit_history_stop(
    emitter: &mut Emitter,
    shard: &str,
    mount: &HistoryMount,
    reasons: &[&str],
    ring: &[(u64, u64, u64, u32)],
) -> ShardEnd {
    for (region, slot, txg, instance) in ring {
        emitter.emit(&format!(
            "name=h5_ring_slot shard={shard} mount={} region={region} slot={slot} checkpoint_txg={txg} instance={instance}",
            mount.number
        ));
    }
    emitter.emit(&format!(
        "name=h5_rows_before_the_sweep shard={shard} mount={} instances={}",
        mount.number,
        joined_or_none(&mount.rows_before_the_sweep)
    ));
    ShardEnd::Stopped {
        reason: format!("t5_at_mount_{}:{}", mount.number, reasons.join(",")),
    }
}

/// X21 ①：同一份挂载前镜像上，只差清扫开关的两次挂载（阈值 θ 那一次 − 「交给后台」那一次）的写字节、读字节之差。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SweepDifferential {
    write_bytes_difference: i128,
    read_bytes_difference: i128,
}

/// X21 ① 的差：阈值那一次的（写字节，读字节）减「交给后台」那一次的。
fn sweep_differential(
    with_the_threshold: (u64, u64),
    handed_to_the_background: (u64, u64),
) -> SweepDifferential {
    SweepDifferential {
        write_bytes_difference: i128::from(with_the_threshold.0)
            - i128::from(handed_to_the_background.0),
        read_bytes_difference: i128::from(with_the_threshold.1)
            - i128::from(handed_to_the_background.1),
    }
}

/// 「交给后台」那一次挂载在拷贝上做；做不成交回 `None`（那一次的差分不出，V35 按格判）。
fn background_mount_on_a_copy(
    parameters: &MakeFilesystemParameters,
    pool: &CountedPool,
    device_bytes: u64,
) -> Option<(u64, u64)> {
    let mut copy = copy_of_the_pool(pool, device_bytes);
    let configuration = SweepArm::HandedToTheBackground.configuration();
    let mounted = mount_once_in_the_history(parameters, &mut copy, configuration, 0).ok()?;
    Some(mounted.1.written_and_read_bytes)
}

/// PC-差分（登记 5.3）：第一次没越过阈值的那一次挂载两样差都是 0；第一次越过的那一次写字节之差 > 0。
/// 读字节那一半按登记字面判、不放宽：阈值那一次挂载末尾要读实例表链数行数（`sweep_the_instance_table_at_the_end_of_the_mount`），「交给后台」那一次不读，
/// 推的读字节之差不为 0，那一格就是 V35 打中、记一次输（R5c、R5d 不出，登记第十二节修订）；汇总行另把写、读两样差分开报，
/// 读字节之差另有新臂 `chain_read_differential_holds` 各自判。
fn differential_control_below_holds(differential: Option<SweepDifferential>) -> bool {
    differential.is_some_and(|difference| {
        difference.write_bytes_difference == 0 && difference.read_bytes_difference == 0
    })
}

fn differential_control_above_holds(differential: Option<SweepDifferential>) -> bool {
    differential.is_some_and(|difference| difference.write_bytes_difference > 0)
}

/// 沿实例表链读一遍要读几片：`max(1, ⌈行数 ÷ 369⌉)`（一片 369 行）。与 `instance_table_chain_pages` 不是一个量：那个是切换预留的式子（行数加 N_switch），
/// 这里是阈值那一次挂载末尾数行数时真读的片数，行数就是链上此刻的行数。
fn instance_table_chain_pages_read(rows: u64) -> u64 {
    rows.div_ceil(INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE).max(1)
}

/// 阈值那一次挂载在末尾读实例表链的字节数，另算、不取装置的读数：每片读一次 `INSTANCE_TABLE_PAGE_BYTES`
/// （`recovery::read_unit_via_locations` 读第一条位置条目、校验和对就收，不读第二份），片数照 `instance_table_chain_pages_read`。
fn instance_table_chain_read_bytes(rows: u64) -> u64 {
    instance_table_chain_pages_read(rows) * INSTANCE_TABLE_PAGE_BYTES
}

/// PC-差分的新臂（登记第十二节修订加的，原臂 `differential_control_below_holds` 照字面另判、两臂不合成一个真假）：没越过阈值的那一次，
/// 读字节之差等于阈值那一次读实例表链的字节数；差没做成、期望没算出都不算过。
fn chain_read_differential_holds(
    differential: Option<SweepDifferential>,
    expected_chain_read_bytes: Option<u64>,
) -> bool {
    match (differential, expected_chain_read_bytes) {
        (Some(difference), Some(expected)) => {
            difference.read_bytes_difference == i128::from(expected)
        }
        (None, _) | (_, None) => false,
    }
}

/// 新臂的结果行：每格一行，在没越过阈值的那一次挂载上；期望字节取锚点模型这一次的 rows0，不取装置读到的。
fn chain_read_differential_line(
    shard: &str,
    mount: u64,
    differential: Option<SweepDifferential>,
    model_rows0: Option<u64>,
) -> String {
    let expected = model_rows0.map(instance_table_chain_read_bytes);
    format!(
        "name=h5_chain_read_differential shard={shard} mount={mount} read_bytes_difference={} model_rows0={} expected_chain_pages={} expected_chain_read_bytes={} read_bytes_difference_equals_the_chain_read={}",
        differential.map_or_else(|| "none".to_string(), |difference| difference.read_bytes_difference.to_string()),
        optional_number_text(model_rows0),
        optional_number_text(model_rows0.map(instance_table_chain_pages_read)),
        optional_number_text(expected),
        chain_read_differential_holds(differential, expected),
    )
}

/// R5a、R5b 甲、R5c、PC-不扫 要的整段历史的汇总。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct HistoryTally {
    mounts: u64,
    sweeps_done: usize,
    peak_rows0: u64,
    mounts_with_rows0_above_the_threshold: u64,
    mounts_with_rows0_above_366: u64,
    peak_chain_pages: u64,
    mounts_with_two_or_more_chain_pages: u64,
    final_rows: u64,
    last_rows0: u64,
    /// PC-不扫：第一次挂载 rows0 = 1，之后每次挂载比上一次多 1。
    rows0_plus_one_each_mount: bool,
    /// 每次清扫的写、读字节之差；那一次的差分没做成是 `None`（读不到 ≠ 读到 0）。
    sweep_write_bytes: Vec<Option<i128>>,
    sweep_read_bytes: Vec<Option<i128>>,
    sweep_mount_nanoseconds: Vec<u64>,
    differential_below_the_threshold: Option<SweepDifferential>,
    differential_at_the_first_sweep: Option<SweepDifferential>,
}

impl HistoryTally {
    fn note(&mut self, mount: &HistoryMount, arm: SweepArm) {
        if self.mounts == 0 {
            self.rows0_plus_one_each_mount = mount.rows0 == 1;
        } else if mount.rows0 != self.last_rows0 + 1 {
            self.rows0_plus_one_each_mount = false;
        }
        self.mounts += 1;
        self.peak_rows0 = self.peak_rows0.max(mount.rows0);
        self.mounts_with_rows0_above_the_threshold += u64::from(
            arm.threshold_rows()
                .is_some_and(|threshold| mount.rows0 > threshold),
        );
        self.mounts_with_rows0_above_366 += u64::from(mount.rows0 > ROWS_KEPT_ON_ONE_CHAIN_PAGE);
        let pages = instance_table_chain_pages(mount.rows0);
        self.peak_chain_pages = self.peak_chain_pages.max(pages);
        self.mounts_with_two_or_more_chain_pages += u64::from(pages >= 2);
        self.final_rows = mount.rows_left.unwrap_or(mount.rows0);
        self.last_rows0 = mount.rows0;
        self.sweeps_done += usize::from(mount.sweep.rows_deleted.is_some());
    }
}

/// 一片 H5：锚点模型与登记 24 行比（不符 ⇒ 停机 T3）、公共前缀、逐次挂载（每次读根环、比模型、判 K33/K34/A36；第一次挂载与模型预计清扫的那几次
/// 另在拷贝上做「交给后台」那一次挂载取差分）、正常卸载或崩溃、汇总。任一次挂载对不上 ⇒ 停机 T5（片文件不写，那一刻的根环与行原样打出）。
fn run_sweep_history_shard(
    shard: &str,
    cell: SweepHistoryCell,
    binary_sha256: &str,
    emitter: &mut Emitter,
) -> ShardEnd {
    run_sweep_history_on_a_pool(
        shard,
        cell,
        PoolGeometry {
            device_bytes: DEFAULT_DEVICE_BYTES,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
        binary_sha256,
        emitter,
    )
}

/// 盘的几何：产物取 4 GiB、768 MiB 环（登记 X16）；单测在小盘上走同一条路。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PoolGeometry {
    device_bytes: u64,
    journal_ring_bytes: u64,
}

fn run_sweep_history_on_a_pool(
    shard: &str,
    cell: SweepHistoryCell,
    geometry: PoolGeometry,
    binary_sha256: &str,
    emitter: &mut Emitter,
) -> ShardEnd {
    emitter.emit(&format!(
        "name=h5_config shard={shard} slots_per_region={} threshold_rows={} unmount_kind={} mounts_at_most={} device_bytes={} journal_ring_bytes={} binary_sha256={binary_sha256}",
        cell.slots_per_region,
        optional_number_text(cell.arm.threshold_rows()),
        cell.unmount_kind.name(),
        cell.mounts_at_most,
        geometry.device_bytes,
        geometry.journal_ring_bytes,
    ));
    let model = sweep_history_by_the_anchor_model(
        cell.slots_per_region,
        cell.arm.threshold_rows(),
        cell.unmount_kind,
        cell.mounts_at_most,
    );
    if let Some(threshold) = cell.arm.threshold_rows() {
        let registered = REGISTERED_SWEEP_HISTORY_ANCHORS.iter().find(|anchor| {
            anchor.slots_per_region == cell.slots_per_region
                && anchor.threshold_rows == threshold
                && anchor.unmount_kind == cell.unmount_kind
        });
        let reproduced = registered.map(|anchor| anchor.is_reproduced_by(&model));
        emitter.emit(&format!(
            "name=h5_anchor_model shard={shard} model_mounts={} model_sweeps_mount_deleted_left={} model_final_txg={} matches_the_registered_script_output={}",
            model.mounts.len(),
            joined_or_none(
                &model
                    .sweeps()
                    .iter()
                    .map(|(mount, deleted, left)| format!("{mount}:{deleted}:{left}"))
                    .collect::<Vec<_>>()
            ),
            model.final_txg,
            match reproduced {
                Some(true) => "true",
                Some(false) => "false",
                None => "not_registered",
            },
        ));
        if reproduced == Some(false) {
            return ShardEnd::Stopped {
                reason: "t3_anchor_model_differs_from_the_registered_rows".to_string(),
            };
        }
    }
    let parameters = make_filesystem_parameters(geometry.journal_ring_bytes, cell.slots_per_region);
    let mut pool = counted_pool(
        geometry.device_bytes,
        geometry.journal_ring_bytes,
        cell.slots_per_region,
    );
    let configuration = cell.arm.configuration();
    let tallies_after_the_first_file =
        match sweep_history_prefix(&parameters, &mut pool, configuration) {
            Ok(tallies) => tallies,
            Err(failed_step) => {
                return ShardEnd::Stopped {
                    reason: format!("history_prefix_failed:{failed_step}"),
                }
            }
        };
    let mut owner_by_txg: BTreeMap<u64, u32> = BTreeMap::new();
    note_ring_owners(&mut owner_by_txg, &ring_roots_of(&pool, &parameters));
    let model_sweep_mounts: BTreeSet<u64> =
        model.sweeps().iter().map(|(mount, _, _)| *mount).collect();
    let mut tally = HistoryTally::default();
    for number in 1..=cell.mounts_at_most {
        let take_a_differential = cell.arm.threshold_rows().is_some()
            && (number == 1 || model_sweep_mounts.contains(&number));
        let background = if take_a_differential {
            background_mount_on_a_copy(&parameters, &pool, geometry.device_bytes)
        } else {
            None
        };
        let (mut mounted, mount) =
            match mount_once_in_the_history(&parameters, &mut pool, configuration, number) {
                Ok(pair) => pair,
                Err(reason) => {
                    return ShardEnd::Stopped {
                        reason: format!("mount_{number}_failed:{reason}"),
                    }
                }
            };
        let ring = ring_roots_of(&pool, &parameters);
        note_ring_owners(&mut owner_by_txg, &ring);
        let in_the_ring = instances_in_the_ring_when_the_sweep_judges(
            &owner_by_txg,
            mount.checkpoint_txg,
            ROOT_RING_REGIONS * cell.slots_per_region,
        );
        let model_mount = model
            .mounts
            .get(usize::try_from(number - 1).expect("挂载序号"));
        let checks = checks_of_a_history_mount(&mount, cell.arm, model_mount, in_the_ring.as_ref());
        emit_history_mount(emitter, shard, &mount, model_mount, checks);
        if take_a_differential {
            let differential = background.map(|background_bytes| {
                sweep_differential(mount.written_and_read_bytes, background_bytes)
            });
            emitter.emit(&format!(
                "name=h5_differential shard={shard} mount={number} swept={} write_bytes_difference={} read_bytes_difference={} mount_nanoseconds={}",
                mount.sweep.rows_deleted.is_some(),
                differential.map_or_else(|| "none".to_string(), |difference| difference.write_bytes_difference.to_string()),
                differential.map_or_else(|| "none".to_string(), |difference| difference.read_bytes_difference.to_string()),
                mount.mount_nanoseconds,
            ));
            if number == 1 {
                tally.differential_below_the_threshold = differential;
                emitter.emit(&chain_read_differential_line(
                    shard,
                    number,
                    differential,
                    model_mount.map(|expected| expected.rows0),
                ));
            }
            if mount.sweep.rows_deleted.is_some() {
                if tally.sweeps_done == 0 {
                    tally.differential_at_the_first_sweep = differential;
                }
                tally
                    .sweep_write_bytes
                    .push(differential.map(|difference| difference.write_bytes_difference));
                tally
                    .sweep_read_bytes
                    .push(differential.map(|difference| difference.read_bytes_difference));
                tally.sweep_mount_nanoseconds.push(mount.mount_nanoseconds);
            }
        }
        if !checks.all_hold() {
            return emit_history_stop(emitter, shard, &mount, &checks.failed_names(), &ring);
        }
        tally.note(&mount, cell.arm);
        if cell.arm.threshold_rows().is_some() && tally.sweeps_done == SWEEPS_TO_FINISH_A_HISTORY {
            break;
        }
        match cell.unmount_kind {
            UnmountKind::Normal => {
                let unmounted = guarded(|| {
                    unmount(
                        &parameters,
                        &mut pool,
                        &mut mounted.allocator,
                        &mut mounted.current,
                        mounted.output.shadow_ledger,
                    )
                    .map(|_| ())
                    .map_err(|error| variant_name_of(&format!("{error:?}")))
                });
                if let Err(reason) = unmounted {
                    return ShardEnd::Stopped {
                        reason: format!("unmount_after_mount_{number}_failed:{reason}"),
                    };
                }
                note_ring_owners(&mut owner_by_txg, &ring_roots_of(&pool, &parameters));
            }
            UnmountKind::Crash => drop(mounted),
        }
    }
    let total_written =
        written_and_read_bytes(&tally_difference(&pool, &tallies_after_the_first_file)).0;
    emit_history_summary(emitter, shard, cell, &tally, total_written);
    ShardEnd::Finished
}

/// 一列可能缺的数：缺的写 `none`，整列为空写 `none`。
fn joined_optional(values: &[Option<i128>]) -> String {
    joined_or_none(
        &values
            .iter()
            .map(|value| value.map_or_else(|| "none".to_string(), |number| number.to_string()))
            .collect::<Vec<_>>(),
    )
}

/// PC-不扫（登记 5.3）跑前写的是「400 次挂载之后 rows0 到 401」，与登记自己的锚点读法（第 m 次挂载 rows0 = m）对不上；
/// 登记第十二节修订把判据改成锚点模型给的 400（依据：单测 `anchor_model_without_a_threshold_never_sweeps_and_rows0_counts_the_mounts` 的读数，时点在产物之前）。
/// 跑前写的 401 只在产物里作为 `original_registered_final_rows0` 照抄一份，不再判。
const SWEEP_CONTROL_REVISED_FINAL_ROWS0: u64 = 400;
const SWEEP_CONTROL_ORIGINALLY_REGISTERED_FINAL_ROWS0: u64 = 401;

/// 「交给后台」对照的 rows0 期末值是不是修订后的登记值。
fn control_final_rows0_equals_the_revised_registered_value(last_rows0: u64) -> bool {
    last_rows0 == SWEEP_CONTROL_REVISED_FINAL_ROWS0
}

/// 一格历史的汇总行（R5a 的峰值、> θ 与 > 366 的次数、期末值；R5b 甲 的片数峰值与 ≥ 2 片的次数；X21 的写字节与挂载次数；PC-差分；对照格的 PC-不扫）。
fn emit_history_summary(
    emitter: &mut Emitter,
    shard: &str,
    cell: SweepHistoryCell,
    tally: &HistoryTally,
    total_write_bytes: u64,
) {
    let mut fields = vec![
        format!("name=h5_cell_summary shard={shard}"),
        format!("slots_per_region={}", cell.slots_per_region),
        format!(
            "threshold_rows={}",
            optional_number_text(cell.arm.threshold_rows())
        ),
        format!("unmount_kind={}", cell.unmount_kind.name()),
        format!("writable_mounts={}", tally.mounts),
        format!("sweeps_done={}", tally.sweeps_done),
        format!("peak_rows0={}", tally.peak_rows0),
        format!(
            "mounts_with_rows0_above_the_threshold={}",
            tally.mounts_with_rows0_above_the_threshold
        ),
        format!(
            "mounts_with_rows0_above_366={}",
            tally.mounts_with_rows0_above_366
        ),
        format!("peak_chain_pages={}", tally.peak_chain_pages),
        format!(
            "mounts_with_two_or_more_chain_pages={}",
            tally.mounts_with_two_or_more_chain_pages
        ),
        format!("final_rows={}", tally.final_rows),
        format!("total_write_bytes={total_write_bytes}"),
        format!(
            "sweep_write_bytes={}",
            joined_optional(&tally.sweep_write_bytes)
        ),
        format!(
            "sweep_read_bytes={}",
            joined_optional(&tally.sweep_read_bytes)
        ),
        format!(
            "sweep_mount_nanoseconds={}",
            joined_or_none(&tally.sweep_mount_nanoseconds)
        ),
    ];
    match cell.arm {
        SweepArm::AtTheEndOfTheMountAboveTheThreshold { .. } => {
            fields.push(format!(
                "positive_control_differential_below_the_threshold={} positive_control_differential_at_the_first_sweep={} differential_below_the_threshold_write_bytes={} differential_below_the_threshold_read_bytes={}",
                differential_control_below_holds(tally.differential_below_the_threshold),
                differential_control_above_holds(tally.differential_at_the_first_sweep),
                joined_optional(&[tally
                    .differential_below_the_threshold
                    .map(|difference| difference.write_bytes_difference)]),
                joined_optional(&[tally
                    .differential_below_the_threshold
                    .map(|difference| difference.read_bytes_difference)]),
            ));
        }
        SweepArm::HandedToTheBackground => {
            fields.push(format!(
                "positive_control_no_sweep_publish={} positive_control_rows0_plus_one_each_mount={} final_rows0={} registered_final_rows0={SWEEP_CONTROL_REVISED_FINAL_ROWS0} original_registered_final_rows0={SWEEP_CONTROL_ORIGINALLY_REGISTERED_FINAL_ROWS0} final_rows0_equals_the_registered_value={}",
                tally.sweeps_done == 0,
                tally.rows0_plus_one_each_mount,
                tally.last_rows0,
                control_final_rows0_equals_the_revised_registered_value(tally.last_rows0),
            ));
        }
    }
    emitter.emit(&fields.join(" "));
}

// ============================================================================================
// H5 的合并：片齐了按片的次序接成产物，再算 E37 里 H5 的判定（R5b 甲、R5c、G5、几何敏感性、判别力自证 ②、PC-不扫、V35、T4）。
// ============================================================================================

/// 一格阈值臂的汇总（读回 `h5_cell_summary` 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThresholdCellSummary {
    cell: SweepHistoryCell,
    threshold_rows: u64,
    writable_mounts: u64,
    total_write_bytes: u64,
    peak_chain_pages: u64,
    differential_control_holds: bool,
}

fn threshold_cell_summary(
    cell: SweepHistoryCell,
    line: &ResultLine,
) -> Option<ThresholdCellSummary> {
    Some(ThresholdCellSummary {
        cell,
        threshold_rows: cell.arm.threshold_rows()?,
        writable_mounts: line.number("writable_mounts")?,
        total_write_bytes: line.number("total_write_bytes")?,
        peak_chain_pages: line.number("peak_chain_pages")?,
        differential_control_holds: line
            .truth("positive_control_differential_below_the_threshold")?
            && line.truth("positive_control_differential_at_the_first_sweep")?,
    })
}

/// R5b 甲：链重写那一项的片数峰值 = 1 ⇒ 不越过读法甲的上界。
fn chain_pages_stay_on_one_page(peak_chain_pages: u64) -> bool {
    peak_chain_pages == 1
}

/// R5c：W_θ ÷ W_最小 的千分比（W = 整段历史写字节 ÷ 可写挂载次数，交叉相乘免得先除丢精度）。
fn write_amplification_ratio_permille(
    cell: &ThresholdCellSummary,
    smallest: &ThresholdCellSummary,
) -> u128 {
    u128::from(cell.total_write_bytes) * u128::from(smallest.writable_mounts) * 1000
        / (u128::from(cell.writable_mounts) * u128::from(smallest.total_write_bytes)).max(1)
}

fn write_amplification_acceptable(ratio_permille: u128, boundary_permille: u128) -> bool {
    ratio_permille <= boundary_permille
}

/// 同一 (S, 卸载方式) 里 W 最小的那一格（PC-差分 没过的格不参加，V35）。
fn smallest_write_per_mount(cells: &[ThresholdCellSummary]) -> Option<ThresholdCellSummary> {
    cells
        .iter()
        .filter(|cell| cell.differential_control_holds && cell.writable_mounts > 0)
        .min_by(|left, right| {
            (u128::from(left.total_write_bytes) * u128::from(right.writable_mounts))
                .cmp(&(u128::from(right.total_write_bytes) * u128::from(left.writable_mounts)))
        })
        .copied()
}

/// 判别力自证 ②（登记第八节）：把 1.10 挪到真产物两个相邻比值之间（取中点，按两倍千分比算免得取整），至少一个 θ 的判定必须转。
fn acceptance_boundary_moved_between_real_ratios(ratios: &[u128]) -> Option<(u128, usize)> {
    let mut sorted = ratios.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
        .windows(2)
        .map(|pair| pair[0] + pair[1])
        .map(|doubled_boundary| {
            let flipped = ratios
                .iter()
                .filter(|ratio| {
                    write_amplification_acceptable(**ratio, WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE)
                        != (2 * **ratio <= doubled_boundary)
                })
                .count();
            (doubled_boundary, flipped)
        })
        .filter(|(_, flipped)| *flipped > 0)
        .min_by_key(|(doubled_boundary, _)| {
            doubled_boundary.abs_diff(2 * WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE)
        })
}

/// 读回片目录里的一片：做完的交回不含末行的那些行；不在、没做完交回 `None`。
fn finished_shard_lines(directory: &Path, shard: &str) -> Option<Vec<String>> {
    let text = std::fs::read_to_string(directory.join(shard)).ok()?;
    lines_of_a_finished_shard(&text, shard)
}

/// 读回的一片：片名、格、行。
type HistoryShard = (String, SweepHistoryCell, Vec<ResultLine>);

/// E37 里 H5 的判定，接在全部片的行之后。
fn emit_sweep_summary(emitter: &mut Emitter, shards: &[HistoryShard]) -> MergeEnd {
    let summaries: Vec<ThresholdCellSummary> = shards
        .iter()
        .filter(|(shard, _, _)| !shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX))
        .filter_map(|(_, cell, lines)| {
            threshold_cell_summary(
                *cell,
                lines_named(lines, "h5_cell_summary").into_iter().next()?,
            )
        })
        .collect();
    let mut ratios: Vec<u128> = Vec::new();
    let mut verdicts_by_threshold: BTreeMap<u64, Vec<(bool, Option<bool>)>> = BTreeMap::new();
    let mut points_without_a_candidate = 0;
    let mut differential_control_failed_cells = 0;
    for slots_per_region in SLOTS_PER_REGION_SAMPLING_POINTS {
        for unmount_kind in [UnmountKind::Normal, UnmountKind::Crash] {
            let point: Vec<ThresholdCellSummary> = summaries
                .iter()
                .filter(|summary| {
                    summary.cell.slots_per_region == slots_per_region
                        && summary.cell.unmount_kind == unmount_kind
                })
                .copied()
                .collect();
            let smallest = smallest_write_per_mount(&point);
            let mut passing_both = Vec::new();
            for summary in &point {
                let one_page = chain_pages_stay_on_one_page(summary.peak_chain_pages);
                differential_control_failed_cells +=
                    usize::from(!summary.differential_control_holds);
                let ratio = smallest
                    .filter(|_| summary.differential_control_holds)
                    .map(|smallest| write_amplification_ratio_permille(summary, &smallest));
                let acceptable = ratio.map(|ratio| {
                    write_amplification_acceptable(ratio, WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE)
                });
                ratios.extend(ratio);
                if one_page && acceptable == Some(true) {
                    passing_both.push(summary.threshold_rows);
                }
                verdicts_by_threshold
                    .entry(summary.threshold_rows)
                    .or_default()
                    .push((one_page, acceptable));
                emitter.emit(&format!(
                    "name=h5_threshold_verdicts cell={} slots_per_region={slots_per_region} unmount_kind={} threshold_rows={} peak_chain_pages={} r5b_reading_one_page={} writable_mounts={} total_write_bytes={} ratio_to_the_smallest_write_per_mount_permille={} r5c={}",
                    summary.cell.name(),
                    unmount_kind.name(),
                    summary.threshold_rows,
                    summary.peak_chain_pages,
                    if one_page { "not_crossing" } else { "crossing" },
                    summary.writable_mounts,
                    summary.total_write_bytes,
                    ratio.map_or_else(|| "none".to_string(), |ratio| ratio.to_string()),
                    match acceptable {
                        Some(true) => "acceptable",
                        Some(false) => "not_acceptable",
                        None => "void_differential_control_failed",
                    },
                ));
            }
            points_without_a_candidate += usize::from(passing_both.is_empty());
            emitter.emit(&format!(
                "name=h5_point slots_per_region={slots_per_region} unmount_kind={} cells={} thresholds_passing_both={} no_candidate_passes_both={}",
                unmount_kind.name(),
                point.len(),
                joined_or_none(&passing_both),
                passing_both.is_empty(),
            ));
        }
    }
    let mut geometry_unstable = Vec::new();
    for (threshold, verdicts) in &verdicts_by_threshold {
        let page_verdicts: BTreeSet<bool> =
            verdicts.iter().map(|(one_page, _)| *one_page).collect();
        let write_amplification_verdicts: BTreeSet<Option<bool>> =
            verdicts.iter().map(|(_, acceptable)| *acceptable).collect();
        if page_verdicts.len() > 1 || write_amplification_verdicts.len() > 1 {
            geometry_unstable.push(*threshold);
        }
        emitter.emit(&format!(
            "name=h5_geometry_sensitivity threshold_rows={threshold} points={} r5b_stable_across_points={} r5c_stable_across_points={}",
            verdicts.len(),
            page_verdicts.len() == 1,
            write_amplification_verdicts.len() == 1,
        ));
    }
    let moved = acceptance_boundary_moved_between_real_ratios(&ratios);
    emitter.emit(&format!(
        "name=discrimination_self_proof judgement=h5_r5c_write_amplification real_ratios={} moved_doubled_boundary_permille={} flipped={} holds={}",
        ratios.len(),
        moved.map_or_else(|| "none".to_string(), |(boundary, _)| boundary.to_string()),
        moved.map_or(0, |(_, flipped)| flipped),
        moved.is_some(),
    ));
    emit_sweep_verdict(
        emitter,
        shards,
        SweepVerdictInputs {
            differential_control_failed_cells,
            chain_read_differential_failed_cells: chain_read_differential_failed_cells(shards),
            points_without_a_candidate,
            geometry_unstable,
            self_proof_holds: moved.is_some(),
        },
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SweepVerdictInputs {
    differential_control_failed_cells: usize,
    chain_read_differential_failed_cells: usize,
    points_without_a_candidate: usize,
    geometry_unstable: Vec<u64>,
    self_proof_holds: bool,
}

/// PC-差分新臂没过的格数：阈值臂的每一格（段首重跑的那一片不算）要有一条 `h5_chain_read_differential` 行，行里的判定是真；缺行按没过。
fn chain_read_differential_failed_cells(shards: &[HistoryShard]) -> usize {
    shards
        .iter()
        .filter(|(shard, cell, _)| {
            !shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX) && cell.arm.threshold_rows().is_some()
        })
        .filter(|(_, _, lines)| {
            lines_named(lines, "h5_chain_read_differential")
                .into_iter()
                .next()
                .and_then(|line| line.truth("read_bytes_difference_equals_the_chain_read"))
                != Some(true)
        })
        .count()
}

/// H5 的判决行：PC-不扫（V34）、PC-差分（V35）、G5、几何敏感性、判别力自证 ②、T4（段首那一格的重跑与原片逐字段比）各报各的。
fn emit_sweep_verdict(
    emitter: &mut Emitter,
    shards: &[HistoryShard],
    inputs: SweepVerdictInputs,
) -> MergeEnd {
    let control = shards
        .iter()
        .find(|(_, cell, _)| cell.arm == SweepArm::HandedToTheBackground)
        .and_then(|(_, _, lines)| {
            lines_named(lines, "h5_cell_summary")
                .into_iter()
                .next()
                .cloned()
        });
    let control_field = |key: &str| {
        control
            .as_ref()
            .and_then(|line| line.field(key))
            .unwrap_or("not_run")
            .to_string()
    };
    let first = shards
        .iter()
        .find(|(shard, _, _)| !shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX));
    let repeat = shards
        .iter()
        .find(|(shard, _, _)| shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX));
    let repeat_difference = match (first, repeat) {
        (Some((_, _, first_lines)), Some((_, _, repeat_lines))) => {
            first_line_differing_outside_the_timing_fields(first_lines, repeat_lines)
        }
        (Some(_), None) | (None, Some(_)) | (None, None) => Some(0),
    };
    let crossing: BTreeSet<u64> = shards
        .iter()
        .filter(|(shard, _, _)| !shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX))
        .filter_map(|(_, cell, lines)| {
            let summary = threshold_cell_summary(
                *cell,
                lines_named(lines, "h5_cell_summary").into_iter().next()?,
            )?;
            (!chain_pages_stay_on_one_page(summary.peak_chain_pages))
                .then_some(summary.threshold_rows)
        })
        .collect();
    emitter.emit(&format!(
        "name=verdict item=h5_summary positive_control_no_sweep_publish={} positive_control_rows0_plus_one_each_mount={} control_final_rows0={} control_final_rows0_equals_the_revised_registered_value={} differential_control_failed_cells={} chain_read_differential_failed_cells={} r5b_crossing_thresholds={} points_without_a_candidate={} geometry_unstable_thresholds={} discrimination_self_proof_holds={} repeat_of_the_first_shard_equal_outside_timing={} repeat_first_differing_line={}",
        control_field("positive_control_no_sweep_publish"),
        control_field("positive_control_rows0_plus_one_each_mount"),
        control_field("final_rows0"),
        control_field("final_rows0_equals_the_registered_value"),
        inputs.differential_control_failed_cells,
        inputs.chain_read_differential_failed_cells,
        joined_or_none(&crossing.into_iter().collect::<Vec<_>>()),
        inputs.points_without_a_candidate,
        joined_or_none(&inputs.geometry_unstable),
        inputs.self_proof_holds,
        repeat_difference.is_none(),
        optional_number_text(repeat_difference.map(|line| u64::try_from(line).expect("行号"))),
    ));
    match repeat_difference {
        None => MergeEnd::Complete,
        Some(_) => MergeEnd::Stopped {
            reason: "t4_repeat_of_the_first_shard_differs".to_string(),
        },
    }
}

/// `h5 merge`：片齐了（每一片的程序文件 sha256 与合并给的相同）按片的次序接成产物打到标准输出，接着打判定；缺片或程序文件不同 ⇒ 停机行、退 3。
fn run_sweep_merge(directory: &Path, binary_sha256: &str) -> i32 {
    let mut emitter = Emitter::to_standard_output();
    emitter.emit(&format!(
        "name=config experiment=e183 segment=3 scope=h5 shards={} binary_sha256={binary_sha256}",
        sweep_shards().len()
    ));
    let mut shards: Vec<HistoryShard> = Vec::new();
    for (shard, cell) in sweep_shards() {
        let Some(lines) = finished_shard_lines(directory, &shard) else {
            emitter.emit(&format!(
                "name=stop reason=shard_missing_or_unfinished shard={shard}"
            ));
            emitter.finish();
            return 3;
        };
        let parsed: Vec<ResultLine> = lines
            .iter()
            .filter_map(|line| ResultLine::parse(line))
            .collect();
        let sha256 = lines_named(&parsed, "h5_config")
            .into_iter()
            .next()
            .and_then(|line| line.field("binary_sha256"))
            .unwrap_or("none")
            .to_string();
        if sha256 != binary_sha256 {
            emitter.emit(&format!(
                "name=stop reason=binary_sha256_differs shard={shard} shard_binary_sha256={sha256}"
            ));
            emitter.finish();
            return 3;
        }
        for line in &lines {
            let body = line.strip_prefix("E7RESULT ").unwrap_or(line);
            emitter.emit(body);
        }
        shards.push((shard, cell, parsed));
    }
    let end = emit_sweep_summary(&mut emitter, &shards);
    finish_a_merge(&mut emitter, end)
}

/// 合并的收尾：停机就打停机行、退 3；否则打完成行、退 0。
fn finish_a_merge(emitter: &mut Emitter, end: MergeEnd) -> i32 {
    match end {
        MergeEnd::Complete => {
            emitter.finish();
            0
        }
        MergeEnd::Stopped { reason } => {
            emitter.emit(&format!("name=stop reason={reason}"));
            emitter.finish();
            3
        }
    }
}

/// `h2 merge`：45 片按片的次序接成产物（没做完的片记结局、不接行），接着打判定。
fn run_capacity_merge(directory: &Path, binary_sha256: &str) -> i32 {
    let mut emitter = Emitter::to_standard_output();
    emitter.emit(&format!(
        "name=config experiment=e183 segment=2 scope=h2 shards={} binary_sha256={binary_sha256}",
        capacity_shards().len()
    ));
    for (shard, _, _) in capacity_shards() {
        for line in finished_shard_lines(directory, &shard).unwrap_or_default() {
            let body = line.strip_prefix("E7RESULT ").unwrap_or(&line).to_string();
            emitter.emit(&body);
        }
    }
    let statuses = read_capacity_statuses(directory, binary_sha256);
    let end = emit_capacity_summary(&mut emitter, &statuses);
    finish_a_merge(&mut emitter, end)
}

/// 一片跑完之后：做完 ⇒ 原子地写片文件、标准输出报一行 `shard_written`、退 0；停机 ⇒ 片文件不写，收着的行与停机行打到标准输出、退 3；
/// 片文件写不成退 1。耗时打到标准错误。
fn finish_a_shard(
    shard: &str,
    output: &Path,
    emitter: Emitter,
    end: ShardEnd,
    started: Instant,
) -> i32 {
    let lines = emitter.into_collected_lines();
    let exit_code = match end {
        ShardEnd::Finished => match write_shard_file_atomically(output, shard, &lines) {
            Ok(()) => {
                println!(
                    "E7RESULT name=shard_written shard={shard} lines={}",
                    lines.len() + 1
                );
                0
            }
            Err(error) => {
                eprintln!("片文件写不成：{} {error}", output.display());
                1
            }
        },
        ShardEnd::Stopped { reason } => {
            for line in &lines {
                println!("{line}");
            }
            println!("E7RESULT name=stop shard={shard} reason={reason}");
            3
        }
    };
    eprintln!(
        "name=item_done item={shard} exit={exit_code} judged={} red={} reused=不复用 seconds={:.1}",
        lines.len(),
        i32::from(exit_code != 0),
        started.elapsed().as_secs_f64()
    );
    exit_code
}

fn run_capacity_shard_command(
    shard: &str,
    output: &Path,
    image_directory: &Path,
    binary_sha256: &str,
) -> i32 {
    let Some((cell, round)) = capacity_shard_by_name(shard) else {
        eprintln!("认不得的 H2 片名：{shard}（`h2 list-shards` 列全部片）");
        return 2;
    };
    let started = Instant::now();
    let mut emitter = Emitter::collecting_lines();
    let end = run_capacity_cell(
        shard,
        cell,
        round,
        image_directory,
        binary_sha256,
        &mut emitter,
    );
    finish_a_shard(shard, output, emitter, end, started)
}

fn run_sweep_shard_command(shard: &str, output: &Path, binary_sha256: &str) -> i32 {
    let Some(cell) = sweep_shards()
        .into_iter()
        .find(|(name, _)| name == shard)
        .map(|(_, cell)| cell)
    else {
        eprintln!("认不得的 H5 片名：{shard}（`h5 list-shards` 列全部片）");
        return 2;
    };
    let started = Instant::now();
    let mut emitter = Emitter::collecting_lines();
    let end = run_sweep_history_shard(shard, cell, binary_sha256, &mut emitter);
    finish_a_shard(shard, output, emitter, end, started)
}

/// 第二、三段的入口（`h2 …`、`h5 …`）；不是这两段的参数交回 `None`，由第一段的入口接着认。
fn run_segment_two_or_three(arguments: &[&str]) -> Option<i32> {
    Some(match arguments {
        ["h2", "list-shards"] => {
            for (shard, _, _) in capacity_shards() {
                println!("{shard}\t1");
            }
            0
        }
        ["h2", "shard", shard, "--shard-output", output, "--image-directory", images, "--binary-sha256", sha256] => {
            run_capacity_shard_command(shard, Path::new(output), Path::new(images), sha256)
        }
        ["h2", "merge", directory, "--binary-sha256", sha256] => {
            run_capacity_merge(Path::new(directory), sha256)
        }
        ["h5", "list-shards"] => {
            for (shard, _) in sweep_shards() {
                println!("{shard}\t1");
            }
            0
        }
        ["h5", "shard", shard, "--shard-output", output, "--binary-sha256", sha256] => {
            run_sweep_shard_command(shard, Path::new(output), sha256)
        }
        ["h5", "merge", directory, "--binary-sha256", sha256] => {
            run_sweep_merge(Path::new(directory), sha256)
        }
        _ => return None,
    })
}

const ITEM_NAMES: [&str; 10] = [
    "constants",
    "positive-control-heap",
    "anchors",
    "h4-small-disk-slots-per-region-4",
    "h4-small-disk-slots-per-region-8",
    "h4-small-disk-slots-per-region-16",
    "h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes",
    "h4-default-disk-slots-per-region-4",
    "h4-default-disk-slots-per-region-8",
    "h4-default-disk-slots-per-region-16",
];

/// 项名到它的运行；认不得的项名交回 `None`。
fn run_item(emitter: &mut Emitter, item: &str) -> Option<ItemOutcome> {
    let device_condition_only = |invalid_device_conditions| ItemOutcome {
        dominance_points: Vec::new(),
        invalid_device_conditions,
    };
    Some(match item {
        "constants" => device_condition_only(run_constants_item(emitter)),
        "positive-control-heap" => device_condition_only(run_positive_control_heap_item(emitter)),
        "anchors" => device_condition_only(run_anchors_item(emitter)),
        "h4-small-disk-slots-per-region-4" => {
            run_small_disk_item(emitter, item, SMALL_DISK_JOURNAL_RING_BYTES, 4)
        }
        "h4-small-disk-slots-per-region-8" => {
            run_small_disk_item(emitter, item, SMALL_DISK_JOURNAL_RING_BYTES, 8)
        }
        "h4-small-disk-slots-per-region-16" => {
            run_small_disk_item(emitter, item, SMALL_DISK_JOURNAL_RING_BYTES, 16)
        }
        "h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes" => {
            run_small_disk_item(emitter, item, ALTERNATIVE_JOURNAL_RING_BYTES, 8)
        }
        "h4-default-disk-slots-per-region-4" => run_default_disk_item(emitter, item, 4),
        "h4-default-disk-slots-per-region-8" => run_default_disk_item(emitter, item, 8),
        "h4-default-disk-slots-per-region-16" => run_default_disk_item(emitter, item, 16),
        _ => return None,
    })
}

/// 跑给定的项，`with_summary` 时在末尾跨点汇总；返回退出码（设备有效性的红 > 0 退 1）。
fn run_items(items: &[&str], with_summary: bool) -> i32 {
    let mut emitter = Emitter::to_standard_output();
    emitter.emit(&format!(
        "name=config experiment=e183 segment=1 scope=h4 items={} slots_per_region_sampling_points={:?} default_device_mebibytes={} small_disk_journal_ring_bytes={SMALL_DISK_JOURNAL_RING_BYTES} alternative_journal_ring_bytes={ALTERNATIVE_JOURNAL_RING_BYTES}",
        items.join(","),
        SLOTS_PER_REGION_SAMPLING_POINTS,
        DEFAULT_DEVICE_BYTES / MEBIBYTE,
    ));
    let mut invalid_device_conditions = 0;
    let mut dominance_points: Vec<DominancePoint> = Vec::new();
    for item in items {
        let started = Instant::now();
        let lines_before = emitter.emitted;
        let outcome = run_item(&mut emitter, item).expect("项名在入口处核过");
        invalid_device_conditions += outcome.invalid_device_conditions;
        dominance_points.extend(outcome.dominance_points);
        emitter.emit(&format!("name=shard_done item={item}"));
        eprintln!(
            "name=item_done item={item} exit={} judged={} red={} reused=不复用 seconds={:.1}",
            i32::from(outcome.invalid_device_conditions > 0),
            emitter.emitted - lines_before,
            outcome.invalid_device_conditions,
            started.elapsed().as_secs_f64(),
        );
        let is_a_device_check = matches!(*item, "constants" | "positive-control-heap" | "anchors");
        if is_a_device_check && outcome.invalid_device_conditions > 0 {
            emitter.emit(&format!(
                "name=stop item={item} reason=device_check_failed invalid_device_conditions={invalid_device_conditions}"
            ));
            emitter.finish();
            return 1;
        }
    }
    if with_summary {
        let small_points: Vec<DominancePoint> = dominance_points
            .iter()
            .filter(|point| point.item.starts_with("h4-small"))
            .cloned()
            .collect();
        let default_points: Vec<DominancePoint> = dominance_points
            .iter()
            .filter(|point| point.item.starts_with("h4-default"))
            .cloned()
            .collect();
        let small_stable = emit_dominance_across_points(&mut emitter, "small_disks", &small_points);
        let default_stable =
            emit_dominance_across_points(&mut emitter, "default_disks", &default_points);
        let self_proof_holds = emit_discrimination_self_proof(&mut emitter, &dominance_points);
        emitter.emit(&format!(
            "name=verdict item=h4_summary dominant_item_stable_across_small_disk_points={small_stable} dominant_item_stable_across_default_disk_points={default_stable} discrimination_self_proof_holds={self_proof_holds} invalid_device_conditions={invalid_device_conditions}"
        ));
    }
    emitter.finish();
    i32::from(invalid_device_conditions > 0)
}

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；强制跑（`--force`）时交回摘要，`main` 把它作为头一行打到标准输出。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source =
        format!("{manifest_directory}/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs");
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
    forced_summary_of(&first_line)
}

/// `preflight.py check` 打的头一行以 `forced` 起头时，把后面的摘要当成强制跑的标记（空格换下划线，免得一行里拆成几个字段）。
fn forced_summary_of(first_line: &str) -> Option<String> {
    first_line
        .strip_prefix("forced")
        .map(|summary| summary.trim().replace(' ', "_"))
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
    let usage = || {
        eprintln!("用法：e183_capacity_read_cache_reserve_and_sweep_costs --list-items | --item <项> | all");
        eprintln!("项：{}", ITEM_NAMES.join(" "));
        eprintln!("第二段：h2 list-shards | h2 shard <片名> --shard-output <片文件> --image-directory <镜像目录> --binary-sha256 <程序文件 sha256> | h2 merge <片目录> --binary-sha256 <…>");
        eprintln!("第三段：h5 list-shards | h5 shard <片名> --shard-output <片文件> --binary-sha256 <…> | h5 merge <片目录> --binary-sha256 <…>");
    };
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let Some(exit_code) = run_segment_two_or_three(&words) {
        std::process::exit(exit_code);
    }
    let exit_code = match words.as_slice() {
        ["--list-items"] => {
            for item in ITEM_NAMES {
                println!("{item}");
            }
            0
        }
        ["--item", item] if ITEM_NAMES.contains(item) => run_items(&[*item], false),
        ["all"] => run_items(&ITEM_NAMES, true),
        _ => {
            usage();
            2
        }
    };
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 计数的全局分配器计的是整个测试进程：同进程里别的用例（锁拦不住不拿锁的）在量窗里释放内存，「峰值减基线」就少掉一截，
    /// 量窗的下沿断言因此会随用例数与调度时红时绿。断言下沿的量窗用例各在一个只跑它自己的子进程里量（开关设了、单线程）：
    /// 在测试进程里被调到时起那个子进程并要求它跑过；在子进程里被调到时才真量。
    const QUIET_WINDOW_CHILD_SWITCH: &str = "E183_ALLOCATOR_WINDOW_IN_THIS_PROCESS";

    /// 当前进程就是只跑一条量窗用例的子进程吗。
    fn is_the_quiet_window_child() -> bool {
        std::env::var_os(QUIET_WINDOW_CHILD_SWITCH).is_some()
    }

    /// 起子进程只跑 `test_name` 这一条（`--exact`、单线程），要求它退 0 且报「1 passed」（名字写错时 `--exact` 什么都不跑也退 0，靠这一句拦住）。
    fn assert_the_quiet_window_child_passes(test_name: &str) {
        let output = std::process::Command::new(std::env::current_exe().expect("测试程序的路径"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(QUIET_WINDOW_CHILD_SWITCH, "1")
            .output()
            .expect("子进程起得来");
        let printed = String::from_utf8_lossy(&output.stdout);
        let failed_text = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "{test_name} 的子进程退 {:?}：\n{printed}\n{failed_text}",
            output.status
        );
        assert!(
            printed.contains("test result: ok. 1 passed;"),
            "{test_name} 的子进程没有恰好跑过 1 条：\n{printed}"
        );
    }

    const COUNTING_ALLOCATOR_PEAK_TEST: &str =
        "tests::counting_allocator_peak_stays_at_the_maximum_after_the_memory_is_freed";
    const HEAP_POSITIVE_CONTROL_TEST: &str =
        "tests::heap_positive_control_sees_a_sixty_four_mebibyte_buffer_inside_the_registered_window";
    const MEASURED_WINDOW_TEST: &str =
        "tests::a_measured_window_reports_the_buffer_allocated_inside_it";

    #[test]
    fn counting_allocator_peak_stays_at_the_maximum_after_the_memory_is_freed() {
        if !is_the_quiet_window_child() {
            assert_the_quiet_window_child_passes(COUNTING_ALLOCATOR_PEAK_TEST);
            return;
        }
        let baseline_bytes = COUNTING_ALLOCATOR.in_use();
        COUNTING_ALLOCATOR.reset_peak_to_in_use();
        let big = vec![1u8; 64 << 20];
        std::hint::black_box(&big);
        drop(big);
        let small = vec![2u8; 4096];
        std::hint::black_box(&small);
        let peak_over_baseline_bytes = COUNTING_ALLOCATOR.peak().saturating_sub(baseline_bytes);
        assert!(
            peak_over_baseline_bytes >= 64 << 20,
            "先占 64 MiB 再放掉、再小分配一次之后，峰值仍是出现过的最大值：{peak_over_baseline_bytes}"
        );
    }

    #[test]
    fn heap_positive_control_sees_a_sixty_four_mebibyte_buffer_inside_the_registered_window() {
        if !is_the_quiet_window_child() {
            assert_the_quiet_window_child_passes(HEAP_POSITIVE_CONTROL_TEST);
            return;
        }
        let control = heap_positive_control(&COUNTING_ALLOCATOR, HEAP_POSITIVE_CONTROL_SLACK_BYTES);
        assert!(control.holds, "{control:?}");
        assert!(control.peak_over_baseline_bytes >= HEAP_POSITIVE_CONTROL_BYTES);
    }

    #[test]
    fn heap_positive_control_judges_both_edges_of_the_registered_window() {
        let slack = HEAP_POSITIVE_CONTROL_SLACK_BYTES;
        assert!(!heap_control_holds(HEAP_POSITIVE_CONTROL_BYTES - 1, slack));
        assert!(heap_control_holds(HEAP_POSITIVE_CONTROL_BYTES, slack));
        assert!(heap_control_holds(
            HEAP_POSITIVE_CONTROL_BYTES + slack,
            slack
        ));
        assert!(!heap_control_holds(
            HEAP_POSITIVE_CONTROL_BYTES + slack + 1,
            slack
        ));
    }

    #[test]
    fn local_geometry_reproduces_every_registered_script_row() {
        for anchor in &REGISTERED_RING_WINDOW_ANCHORS {
            let local = ring_window_by_the_clauses(
                anchor.device_bytes,
                anchor.journal_ring_bytes,
                anchor.slots_per_region,
                0,
            );
            assert!(
                anchor.is_reproduced_by(&local),
                "{} 字节、每区槽数 {}：{local:?}",
                anchor.device_bytes,
                anchor.slots_per_region
            );
        }
    }

    #[test]
    fn default_disk_ring_window_equals_the_registered_absolute_bytes() {
        let expected = [(4u64, 79_659_008u64), (8, 153_190_400), (16, 300_253_184)];
        for (slots_per_region, bytes) in expected {
            let window = ring_window_by_the_clauses(
                4 * GIBIBYTE,
                JOURNAL_RING_DEFAULT_BYTES,
                slots_per_region,
                0,
            );
            assert_eq!(window.ring_window_bytes(), bytes, "S = {slots_per_region}");
            assert_eq!(window.unit_area_slots, 211_968);
            assert_eq!(window.deletion_publish_slots, 374);
        }
        let largest =
            ring_window_by_the_clauses((16 << 40) - MEBIBYTE, JOURNAL_RING_DEFAULT_BYTES, 8, 0);
        assert_eq!(largest.deletion_publish_slots, 804);
        assert_eq!(largest.root_level, 3);
    }

    #[test]
    fn retirement_tree_levels_add_one_slot_each_to_the_deletion_publish() {
        let without = ring_window_by_the_clauses(4 * GIBIBYTE, JOURNAL_RING_DEFAULT_BYTES, 8, 0);
        let with_two_levels =
            ring_window_by_the_clauses(4 * GIBIBYTE, JOURNAL_RING_DEFAULT_BYTES, 8, 2);
        assert_eq!(
            with_two_levels.deletion_publish_slots,
            without.deletion_publish_slots + 2
        );
    }

    #[test]
    fn chain_rewrite_bytes_cross_to_a_second_page_exactly_at_the_registered_row() {
        for (rows, bytes) in REGISTERED_CHAIN_REWRITE_BYTES {
            assert_eq!(
                chain_rewrite_bytes_of_the_switch_reserve(rows),
                bytes,
                "rows0 = {rows}"
            );
        }
        assert_eq!(instance_table_chain_pages(366), 1);
        assert_eq!(instance_table_chain_pages(367), 2);
    }

    #[test]
    fn fixed_regions_classify_offsets_by_their_landing_zone() {
        let regions = FixedRegions::of_geometry(4096, 8, JOURNAL_RING_DEFAULT_BYTES);
        assert_eq!(regions.region_of(0), StoredRegion::SystemConfiguration);
        assert_eq!(regions.region_of(8191), StoredRegion::SystemConfiguration);
        assert_eq!(
            regions.region_of(8192),
            StoredRegion::UnitAreaAndEverythingElse
        );
        assert_eq!(regions.region_of(1 << 20), StoredRegion::RootRing);
        assert_eq!(regions.region_of(16 << 20), StoredRegion::JournalRing);
        assert_eq!(
            regions
                .region_of(JOURNAL_RING_START_SLOT * SLOT_BYTES + JOURNAL_RING_DEFAULT_BYTES - 1),
            StoredRegion::JournalRing
        );
        assert_eq!(
            regions.region_of(UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING * SLOT_BYTES),
            StoredRegion::UnitAreaAndEverythingElse
        );
    }

    #[test]
    fn counting_block_device_counts_reads_and_writes_in_the_region_they_land_in() {
        let regions = FixedRegions::of_geometry(4096, 8, MEBIBYTE);
        let mut device = CountingBlockDevice::new(
            SparseBlockDevice::new(64 * MEBIBYTE, PhysicalBlockSizeInBytes(512)),
            regions,
        );
        let journal_offset = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
        device
            .write_at(journal_offset, &[7u8; 4096], WriteDurability::Plain)
            .expect("journal 环里写 4096 字节");
        let mut buffer = [0u8; 512];
        device.read_at(journal_offset, &mut buffer).expect("读回");
        device
            .read_at(DeviceOffsetInBytes(0), &mut buffer)
            .expect("读系统配置槽");
        assert_eq!(buffer, [0u8; 512]);
        let tally = device.tally_now();
        assert_eq!(
            tally.journal_ring.writes,
            CallsAndBytes {
                calls: 1,
                bytes: 4096
            }
        );
        assert_eq!(
            tally.journal_ring.reads,
            CallsAndBytes {
                calls: 1,
                bytes: 512
            }
        );
        assert_eq!(
            tally.system_configuration.reads,
            CallsAndBytes {
                calls: 1,
                bytes: 512
            }
        );
        assert_eq!(tally.unit_area, ReadsAndWrites::default());
        assert_eq!(tally.root_ring, ReadsAndWrites::default());
        let before = tally;
        device
            .write_at(journal_offset, &[8u8; 512], WriteDurability::Plain)
            .expect("再写一次");
        assert_eq!(
            device.tally_now().minus(before).journal_ring.writes,
            CallsAndBytes {
                calls: 1,
                bytes: 512
            }
        );
    }

    fn terms_with_distinct_powers_of_two() -> DeviceTerms {
        DeviceTerms {
            device: 0,
            capacity_bytes: 1 << 30,
            allocated_bytes: 1 << 20,
            unreclaimable_bytes: 0,
            mount_time_commitment_bytes: 1,
            abandoned_root_exclusive_bytes: 2,
            pending_delete_share_bytes: 4,
            committed_reservation_share_bytes: 8,
            checkpoint_reserve_pool_share_bytes: 16,
            ring_window_bytes: 32,
            available_bytes: 0,
        }
    }

    #[test]
    fn admission_reserve_adds_all_six_items_including_the_ring_window() {
        let terms = terms_with_distinct_powers_of_two();
        assert_eq!(terms.admission_reserve_bytes(), 1 + 2 + 4 + 8 + 16 + 32);
        let item_sum: u64 = terms
            .admission_reserve_items()
            .iter()
            .map(|(_, bytes)| *bytes)
            .sum();
        assert_eq!(item_sum, 1 + 2 + 4 + 8 + 16 + 32);
    }

    #[test]
    fn dominance_needs_at_least_half_of_the_total() {
        let exactly_half = [
            ("heaviest_item", 50),
            ("middle_item", 30),
            ("lightest_item", 20),
        ];
        assert_eq!(
            dominance_of(&exactly_half),
            Dominance::Dominant {
                item: "heaviest_item",
                share_permille: 500
            }
        );
        let just_below_half = [
            ("heaviest_item", 49),
            ("middle_item", 30),
            ("lightest_item", 21),
        ];
        assert_eq!(
            dominance_of(&just_below_half),
            Dominance::NoItemAboveHalf {
                largest_item: "heaviest_item",
                share_permille: 490
            }
        );
        let nothing_reserved = [("heaviest_item", 0), ("middle_item", 0)];
        assert!(matches!(
            dominance_of(&nothing_reserved),
            Dominance::NoItemAboveHalf {
                share_permille: 0,
                ..
            }
        ));
    }

    #[test]
    fn moving_the_dominance_threshold_across_the_largest_share_flips_the_class() {
        let dominant = [
            ("heaviest_item", 60),
            ("middle_item", 25),
            ("lightest_item", 15),
        ];
        assert!(dominance_of(&dominant).is_dominant());
        assert_eq!(moved_threshold_permille(&dominant), 601);
        assert!(dominance_flips_when_the_threshold_is_moved(&dominant));
        let spread = [
            ("heaviest_item", 40),
            ("middle_item", 35),
            ("lightest_item", 25),
        ];
        assert!(!dominance_of(&spread).is_dominant());
        assert_eq!(moved_threshold_permille(&spread), 400);
        assert!(dominance_flips_when_the_threshold_is_moved(&spread));
        assert!(dominance_with_threshold(&spread, 400).is_dominant());
        assert!(!dominance_with_threshold(&spread, 401).is_dominant());
    }

    #[test]
    fn mount_time_commitment_by_the_clauses_matches_the_hand_computed_value() {
        assert_eq!(
            mount_time_commitment_by_the_clauses(0, 10),
            4 * (32_768 + 3 * 10 * 16_384)
        );
        assert_eq!(
            mount_time_commitment_by_the_clauses(367, 10),
            4 * (2 * 32_768 + 3 * 10 * 16_384)
        );
        assert_eq!(
            checkpoint_cost_blocks_solved_from_the_commitment(
                mount_time_commitment_by_the_clauses(5, 13),
                5
            ),
            Some(13)
        );
        assert_eq!(
            checkpoint_cost_blocks_solved_from_the_commitment(
                mount_time_commitment_by_the_clauses(5, 13) + 1,
                5
            ),
            None
        );
    }

    fn snapshot_by_the_clauses(entry: ReadingEntry, rows: u64, blocks: u64) -> ReadingSnapshot {
        let device = |number: u32| DeviceTerms {
            device: number,
            capacity_bytes: unit_area_slots(64 * MEBIBYTE, MEBIBYTE) * SLOT_BYTES,
            allocated_bytes: 0,
            unreclaimable_bytes: 0,
            mount_time_commitment_bytes: mount_time_commitment_by_the_clauses(rows, blocks),
            abandoned_root_exclusive_bytes: 0,
            pending_delete_share_bytes: 0,
            committed_reservation_share_bytes: 0,
            checkpoint_reserve_pool_share_bytes: blocks * SLOT_BYTES,
            ring_window_bytes: 0,
            available_bytes: 0,
        };
        ReadingSnapshot {
            entry,
            instance_rows_of_the_switch_reserve: rows,
            replicas: 2,
            checkpoint_reserve_pool_bytes: blocks * SLOT_BYTES * 2,
            pending_delete_bytes: 0,
            committed_reservation_bytes: 0,
            retirement_tree_internal_levels: None,
            devices: vec![device(0), device(1)],
        }
    }

    #[test]
    fn reading_checks_hold_on_a_reading_built_by_the_clauses_and_fail_when_one_item_is_off() {
        let consistent = snapshot_by_the_clauses(ReadingEntry::OfAWritableMount, 0, 10);
        let checks = checks_of_a_reading(&consistent, 64 * MEBIBYTE, MEBIBYTE, 8);
        assert!(checks.all_hold(), "{checks:?}");
        assert_eq!(checks.checkpoint_cost_blocks_by_the_pool, Some(10));
        let mut commitment_off = consistent.clone();
        commitment_off.devices[1].mount_time_commitment_bytes += SLOT_BYTES;
        assert!(!checks_of_a_reading(&commitment_off, 64 * MEBIBYTE, MEBIBYTE, 8).all_hold());
        let mut deleting_pending = consistent.clone();
        deleting_pending.pending_delete_bytes = 2;
        assert!(!checks_of_a_reading(&deleting_pending, 64 * MEBIBYTE, MEBIBYTE, 8).all_hold());
        let mut window_on_an_empty_pool = consistent;
        window_on_an_empty_pool.devices[0].ring_window_bytes = SLOT_BYTES;
        assert!(
            !checks_of_a_reading(&window_on_an_empty_pool, 64 * MEBIBYTE, MEBIBYTE, 8).all_hold()
        );
    }

    #[test]
    fn the_publish_entry_reads_one_more_instance_row_than_the_mount_entry() {
        assert_eq!(
            ReadingEntry::OfAWritableMount.instance_rows_of_the_switch_reserve(366),
            366
        );
        assert_eq!(
            ReadingEntry::BeforeAPublish.instance_rows_of_the_switch_reserve(366),
            367
        );
    }

    fn flags_at(device_mebibytes: u64, empty: bool, file: bool) -> TrialFlags {
        TrialFlags {
            device_bytes: device_mebibytes * MEBIBYTE,
            make_filesystem_succeeded: device_mebibytes >= 18,
            empty_mount_admitted: empty,
            file_mount_admitted: file,
            empty_mount_still_short: false,
            file_mount_still_short: false,
        }
    }

    #[test]
    fn minimum_is_the_first_disk_where_the_milestone_holds_even_if_later_ones_fail() {
        let trials = vec![
            flags_at(17, false, false),
            flags_at(18, false, false),
            flags_at(19, true, false),
            flags_at(20, false, false),
            flags_at(21, true, true),
        ];
        assert_eq!(
            minimum_device_bytes(&trials, Milestone::MakeFilesystemSucceeds),
            Some(18 * MEBIBYTE)
        );
        assert_eq!(
            minimum_device_bytes(&trials, Milestone::EmptyMountAdmitted),
            Some(19 * MEBIBYTE)
        );
        assert_eq!(
            minimum_device_bytes(&trials, Milestone::FileMountAdmitted),
            Some(21 * MEBIBYTE)
        );
        assert_eq!(
            minimum_device_bytes(&trials, Milestone::EmptyMountStillShort),
            None
        );
    }

    #[test]
    fn monotonicity_lists_every_failure_in_the_sixteen_mebibyte_window_and_nothing_beyond() {
        let mut trials: Vec<TrialFlags> = (17..=60)
            .map(|mebibytes| flags_at(mebibytes, mebibytes >= 19, mebibytes >= 21))
            .collect();
        let clean = monotonicity_of(&trials, Milestone::EmptyMountAdmitted);
        assert!(clean.is_monotone(), "{clean:?}");
        trials[(19 + 16 - 17) as usize].empty_mount_admitted = false;
        trials[(19 + 17 - 17) as usize].empty_mount_admitted = false;
        let broken = monotonicity_of(&trials, Milestone::EmptyMountAdmitted);
        assert_eq!(broken.failures_in_the_window, vec![35 * MEBIBYTE]);
        assert!(!broken.is_monotone());
        let short: Vec<TrialFlags> = (17..=30)
            .map(|mebibytes| flags_at(mebibytes, mebibytes >= 19, true))
            .collect();
        let unverified = monotonicity_of(&short, Milestone::EmptyMountAdmitted);
        assert_eq!(
            unverified.window_points_never_tried,
            vec![
                31 * MEBIBYTE,
                32 * MEBIBYTE,
                33 * MEBIBYTE,
                34 * MEBIBYTE,
                35 * MEBIBYTE
            ]
        );
        assert!(!unverified.is_monotone());
    }

    #[test]
    fn search_stops_sixteen_mebibytes_after_the_largest_of_the_three_minima() {
        let up_to = |last_mebibytes: u64| -> Vec<TrialFlags> {
            (4..=last_mebibytes)
                .map(|mebibytes| flags_at(mebibytes, mebibytes >= 19, mebibytes >= 25))
                .collect()
        };
        assert!(!search_may_stop(&[]));
        assert!(!search_may_stop(&up_to(40)));
        assert!(search_may_stop(&up_to(41)));
        let never_found: Vec<TrialFlags> = (4..=100)
            .map(|mebibytes| flags_at(mebibytes, false, false))
            .collect();
        assert!(!search_may_stop(&never_found));
        let at_the_ceiling: Vec<TrialFlags> = vec![flags_at(1024, false, false)];
        assert!(search_may_stop(&at_the_ceiling));
    }

    fn synthetic_report(class: AdmissionClass) -> MountReport {
        MountReport {
            class,
            readings: Vec::new(),
            instance_rows_by_the_allocator: 0,
            instance_rows_read_back_from_the_disk: None,
            mount_tallies: Vec::new(),
            available_minimum_bytes: 0,
        }
    }

    fn trial_whose_empty_mount_ends_as(device_mebibytes: u64, class: AdmissionClass) -> Trial {
        Trial {
            device_bytes: device_mebibytes * MEBIBYTE,
            make_filesystem: StageOutcome::Succeeded(()),
            empty_mount: StageOutcome::Succeeded(synthetic_report(class)),
            first_file: StageOutcome::NotReached,
            unmount: StageOutcome::NotReached,
            file_mount: StageOutcome::NotReached,
        }
    }

    #[test]
    fn a_mount_that_stays_short_after_raising_the_floor_is_not_counted_as_admitted() {
        assert!(AdmissionClass::AdmittedBeforeAcquisition.counts_as_admitted());
        assert!(AdmissionClass::AdmittedAfterTheFloorRaises.counts_as_admitted());
        assert!(!AdmissionClass::StillShortAfterTheFloorRaises.counts_as_admitted());
        assert!(!AdmissionClass::NotJudgedByTheTestOnlySwitch.counts_as_admitted());
        let trials = [
            trial_whose_empty_mount_ends_as(20, AdmissionClass::StillShortAfterTheFloorRaises),
            trial_whose_empty_mount_ends_as(21, AdmissionClass::AdmittedAfterTheFloorRaises),
        ];
        let flags: Vec<TrialFlags> = trials.iter().map(Trial::flags).collect();
        assert_eq!(
            minimum_device_bytes(&flags, Milestone::EmptyMountAdmitted),
            Some(21 * MEBIBYTE)
        );
        assert_eq!(
            minimum_device_bytes(&flags, Milestone::EmptyMountStillShort),
            Some(20 * MEBIBYTE)
        );
    }

    #[test]
    fn the_file_state_needs_every_step_of_the_chain_to_have_succeeded() {
        let mut trial =
            trial_whose_empty_mount_ends_as(30, AdmissionClass::AdmittedBeforeAcquisition);
        trial.file_mount =
            StageOutcome::Succeeded(synthetic_report(AdmissionClass::AdmittedBeforeAcquisition));
        assert!(
            !trial.flags().file_mount_admitted,
            "首个文件和卸载都没走到时不算成"
        );
        trial.first_file = StageOutcome::Succeeded(());
        trial.unmount = StageOutcome::Succeeded(());
        assert!(trial.flags().file_mount_admitted);
        trial.first_file = StageOutcome::Failed("PlacementRefused".to_string());
        assert!(!trial.flags().file_mount_admitted);
    }

    #[test]
    fn admission_positive_control_needs_the_switch_mount_and_the_admission_refusal_variant() {
        let refused: StageOutcome<MountReport> =
            StageOutcome::Failed("SpaceAdmissionRefusedBeforeAcquisition".to_string());
        let other_failure: StageOutcome<MountReport> = StageOutcome::Failed("Publish".to_string());
        assert!(admission_positive_control_holds(true, &refused));
        assert!(!admission_positive_control_holds(false, &refused));
        assert!(!admission_positive_control_holds(true, &other_failure));
        assert!(!admission_positive_control_holds(
            true,
            &StageOutcome::NotReached
        ));
    }

    #[test]
    fn local_constants_equal_the_implementation_constants() {
        for comparison in constants_against_the_implementation() {
            assert!(
                comparison.is_equal(),
                "{}：本地 {} 实现 {}",
                comparison.name,
                comparison.local,
                comparison.implementation
            );
        }
    }

    #[test]
    fn variant_name_keeps_only_the_leading_identifier() {
        assert_eq!(
            variant_name_of("SpaceAdmissionRefusedBeforeAcquisition { instance: 3 }"),
            "SpaceAdmissionRefusedBeforeAcquisition"
        );
        assert_eq!(variant_name_of("Recovery(Foo)"), "Recovery");
    }

    #[test]
    fn a_roomy_disk_reaches_every_stage_and_every_reading_check_holds() {
        let trial = run_trial(256 * MEBIBYTE, MEBIBYTE, 8);
        let flags = trial.flags();
        assert!(flags.make_filesystem_succeeded, "{trial:?}");
        assert!(flags.empty_mount_admitted, "{:?}", trial.empty_mount);
        assert!(flags.file_mount_admitted, "{:?}", trial.file_mount);
        for outcome in [&trial.empty_mount, &trial.file_mount] {
            let StageOutcome::Succeeded(report) = outcome else {
                panic!("挂载没成：{outcome:?}");
            };
            assert_eq!(report.readings.len(), 2);
            for snapshot in &report.readings {
                let checks = checks_of_a_reading(snapshot, 256 * MEBIBYTE, MEBIBYTE, 8);
                assert!(
                    checks.mount_time_commitment_by_the_clauses
                        && checks.pending_delete_is_zero
                        && checks.capacity_by_the_local_geometry,
                    "{:?} {checks:?}",
                    snapshot.entry
                );
            }
        }
    }

    /// 第一段 `all` 输出的钉死期望值（测试不读 research/ 下的产物，research/ 可以整个删掉）：
    /// 输出里每行从 `E7RESULT ` 起截下来、各加一个换行，依次连成的字节流的 SHA-256（小写十六进制）与行数。
    /// 来源：2026-10-18 入库的第一段 `all` 产物（`research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-18.out`，
    /// 459 行、152434 字节、每行都以 `E7RESULT ` 起头），对整个文件现算：`sha256sum <产物>` 与 `wc -l <产物>`。
    /// 2026-10-20 数据形态名字改名（欠账 C430）之后重钉：产物里 `name=h4_reading_checks` 那三个键名照装置一起换过，别的字节不动。
    const SEGMENT_ONE_PINNED_SHA256: &str =
        "7ed2e16c6cc42b914f740f943a05440bef6a43c0917baac0114d5084f558adc8";
    const SEGMENT_ONE_PINNED_LINE_COUNT: usize = 459;

    /// SHA-256（FIPS 180-4），小写十六进制。测试里自带一份，免得给 checker 档包加依赖；
    /// 正确性由逐字节比对用例里的已知向量（空串、`abc`）先核。
    fn sha256_hex_of(bytes: &[u8]) -> String {
        /// 前 64 个素数的立方根小数部分的前 32 位。
        const ROUND_CONSTANTS: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut digest_words: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];
        let mut padded = bytes.to_vec();
        padded.push(0x80);
        while padded.len() % 64 != 56 {
            padded.push(0);
        }
        let bit_length = u64::try_from(bytes.len()).expect("长度装得进 u64") * 8;
        padded.extend_from_slice(&bit_length.to_be_bytes());
        let (blocks, _) = padded.as_chunks::<64>();
        for block in blocks {
            let mut schedule = [0u32; 64];
            let (block_words, _) = block.as_chunks::<4>();
            for (word, chunk) in schedule.iter_mut().zip(block_words) {
                *word = u32::from_be_bytes(*chunk);
            }
            for index in 16..64 {
                let early = schedule[index - 15];
                let late = schedule[index - 2];
                let early_mix = early.rotate_right(7) ^ early.rotate_right(18) ^ (early >> 3);
                let late_mix = late.rotate_right(17) ^ late.rotate_right(19) ^ (late >> 10);
                schedule[index] = schedule[index - 16]
                    .wrapping_add(early_mix)
                    .wrapping_add(schedule[index - 7])
                    .wrapping_add(late_mix);
            }
            let mut working = digest_words;
            for index in 0..64 {
                let word_e = working[4];
                let word_a = working[0];
                let choose = (word_e & working[5]) ^ (!word_e & working[6]);
                let majority =
                    (word_a & working[1]) ^ (word_a & working[2]) ^ (working[1] & working[2]);
                let mix_e =
                    word_e.rotate_right(6) ^ word_e.rotate_right(11) ^ word_e.rotate_right(25);
                let mix_a =
                    word_a.rotate_right(2) ^ word_a.rotate_right(13) ^ word_a.rotate_right(22);
                let sum_for_e = working[7]
                    .wrapping_add(mix_e)
                    .wrapping_add(choose)
                    .wrapping_add(ROUND_CONSTANTS[index])
                    .wrapping_add(schedule[index]);
                let sum_for_a = sum_for_e.wrapping_add(mix_a).wrapping_add(majority);
                working = [
                    sum_for_a,
                    working[0],
                    working[1],
                    working[2],
                    working[3].wrapping_add(sum_for_e),
                    working[4],
                    working[5],
                    working[6],
                ];
            }
            for (digest_word, worked) in digest_words.iter_mut().zip(working) {
                *digest_word = digest_word.wrapping_add(worked);
            }
        }
        digest_words
            .iter()
            .map(|word| format!("{word:08x}"))
            .collect()
    }

    /// 设了它，下面那条子进程用例才真跑第一段 `all`。
    const SEGMENT_ONE_CHILD_SWITCH: &str = "E183_SEGMENT_ONE_ALL_IN_THIS_PROCESS";
    const SEGMENT_ONE_CHILD_TEST: &str =
        "tests::segment_one_all_printed_in_a_child_process_when_the_comparison_spawns_it";

    /// 只在逐字节比对的用例起的子进程里跑（设了开关）：那个子进程只跑这一条，计数分配器的量窗里没有别的用例的分配；
    /// 直接跑全量时没设开关，什么都不做（第一段的输出由下一条用例在子进程里取）。
    #[test]
    fn segment_one_all_printed_in_a_child_process_when_the_comparison_spawns_it() {
        if std::env::var_os(SEGMENT_ONE_CHILD_SWITCH).is_none() {
            return;
        }
        assert_eq!(run_items(&ITEM_NAMES, true), 0, "第一段 all 退 0");
    }

    /// 派发要求：加了第二、三段之后，第一段 `all` 的输出与入库产物逐字节相同（比的是源码里钉死的 SHA-256 与行数，不读产物文件）。
    #[test]
    fn segment_one_all_output_equals_the_registered_product_byte_for_byte() {
        assert_eq!(
            sha256_hex_of(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "SHA-256 的已知向量（空串）"
        );
        assert_eq!(
            sha256_hex_of(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "SHA-256 的已知向量（abc）"
        );
        let output = std::process::Command::new(std::env::current_exe().expect("测试程序的路径"))
            .args([
                "--exact",
                SEGMENT_ONE_CHILD_TEST,
                "--nocapture",
                "--test-threads=1",
            ])
            .env(SEGMENT_ONE_CHILD_SWITCH, "1")
            .output()
            .expect("子进程起得来");
        assert!(
            output.status.success(),
            "子进程退 {:?}：{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        let printed = String::from_utf8(output.stdout).expect("输出是 UTF-8");
        let printed_lines: Vec<&str> = printed
            .lines()
            .filter_map(|line| line.find("E7RESULT ").map(|start| &line[start..]))
            .collect();
        assert_eq!(
            printed_lines.len(),
            SEGMENT_ONE_PINNED_LINE_COUNT,
            "现跑的 E7RESULT 行数与钉死的入库产物行数不同"
        );
        let printed_bytes: Vec<u8> = printed_lines
            .iter()
            .flat_map(|line| line.bytes().chain(std::iter::once(b'\n')))
            .collect();
        assert_eq!(
            sha256_hex_of(&printed_bytes),
            SEGMENT_ONE_PINNED_SHA256,
            "逐字节：现跑每行加一个换行连成的字节流的 SHA-256 与钉死的入库产物不同（{} 行、{} 字节）",
            printed_lines.len(),
            printed_bytes.len()
        );
    }

    #[test]
    fn log_log_slope_of_a_doubling_series_is_exactly_one_and_of_a_constant_is_zero() {
        let doubling: Vec<(u64, u64)> =
            (0..5).map(|power| (1u64 << power, 1u64 << power)).collect();
        let slope = log_log_least_squares_slope(&doubling).expect("五个点");
        assert!((slope.slope - 1.0).abs() < 1e-12, "{slope:?}");
        assert_eq!(slope.points, 5);
        assert!(!slope.has_zero_point);
        let squares: Vec<(u64, u64)> = (1..6)
            .map(|power| (1u64 << power, 1u64 << (2 * power)))
            .collect();
        assert!((log_log_least_squares_slope(&squares).expect("五个点").slope - 2.0).abs() < 1e-12);
        let constant: Vec<(u64, u64)> = (1..6).map(|power| (1u64 << (10 * power), 4096)).collect();
        assert!(
            log_log_least_squares_slope(&constant)
                .expect("五个点")
                .slope
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn a_zero_reading_is_taken_as_one_byte_and_flagged() {
        let slope = log_log_least_squares_slope(&[(2, 0), (4, 4)]).expect("两个点");
        assert!(slope.has_zero_point);
        assert!(
            (slope.slope - 2.0).abs() < 1e-12,
            "0 按 1 取：log₂ 1 = 0 到 log₂ 4 = 2，跨 1 个翻倍"
        );
        assert!(
            log_log_least_squares_slope(&[(2, 5)]).is_none(),
            "一个点没有斜率"
        );
        assert!(
            log_log_least_squares_slope(&[(2, 5), (2, 9)]).is_none(),
            "x 相同没有斜率"
        );
    }

    #[test]
    fn slope_class_puts_exactly_0_8_in_linear_and_exactly_0_2_in_sublinear() {
        assert_eq!(slope_class(0.8), SlopeClass::Linear);
        assert_eq!(slope_class(0.799), SlopeClass::Sublinear);
        assert_eq!(slope_class(0.2), SlopeClass::Sublinear);
        assert_eq!(slope_class(0.199), SlopeClass::BasicallyUnrelated);
        assert_eq!(slope_class(-0.3), SlopeClass::BasicallyUnrelated);
    }

    #[test]
    fn five_rounds_give_a_class_only_when_every_round_falls_in_it() {
        let stable = [Some(0.9), Some(1.1), Some(0.85), Some(1.0), Some(0.95)];
        assert_eq!(
            class_across_rounds(&stable, 5),
            ClassAcrossRounds::Stable(SlopeClass::Linear)
        );
        let crossing = [Some(0.9), Some(0.7), Some(0.85), Some(1.0), Some(0.95)];
        assert_eq!(
            class_across_rounds(&crossing, 5),
            ClassAcrossRounds::Unstable
        );
        let short = [Some(0.9), None, Some(0.85), Some(1.0), Some(0.95)];
        assert_eq!(
            class_across_rounds(&short, 5),
            ClassAcrossRounds::NotJudged {
                rounds_with_a_slope: 4
            }
        );
    }

    #[test]
    fn moving_the_linear_boundary_between_two_real_slopes_flips_one_class() {
        let moved = linear_boundary_moved_between_real_slopes(&[0.05, 0.95, 1.02]).expect("挪得出");
        assert!((moved.boundary - 0.985).abs() < 1e-12, "{moved:?}");
        assert_eq!(moved.flipped, 1, "0.95 从线性转成次线性");
        assert!(
            linear_boundary_moved_between_real_slopes(&[0.1, 0.9]).is_none(),
            "只有跨着 0.8 的那一对：挪到中点 0.5 一个类都不转"
        );
        assert!(synthetic_slopes_fall_in_two_classes());
    }

    #[test]
    fn median_takes_the_middle_value_and_the_lower_one_of_an_even_count() {
        assert_eq!(median_of(&[9, 1, 5, 7, 3]), Some(5));
        assert_eq!(median_of(&[4, 2, 8, 6]), Some(4));
        assert_eq!(median_of(&[]), None);
    }

    #[test]
    fn capacity_cells_are_nine_distinct_cells_and_the_shards_are_forty_five_round_major() {
        let cells = capacity_cells();
        assert_eq!(cells.len(), 9);
        assert_eq!(cells.iter().collect::<BTreeSet<_>>().len(), 9);
        assert_eq!(cells[4].device_bytes + MEBIBYTE, 16 * TEBIBYTE);
        let shards = capacity_shards();
        assert_eq!(shards.len(), 45);
        assert_eq!(shards[0].0, "h2-device-4096-mebibytes-units-0-round-1");
        assert_eq!(shards[9].2, 2, "第 10 片已是第 2 轮");
        assert_eq!(
            capacity_shard_by_name("h2-device-65536-mebibytes-units-10240-round-5")
                .map(|(cell, round)| (cell.sequentially_written_units, round)),
            Some((10240, 5))
        );
        assert_eq!(
            capacity_shard_by_name("h2-device-65536-mebibytes-units-1024-round-1"),
            None,
            "64 GiB 上没有 n = 1024 的格"
        );
    }

    #[test]
    fn written_contents_have_the_registered_lengths_and_differ_between_calls() {
        assert_eq!(sequential_write_content(3).len(), 3 * 32634);
        assert_eq!(measured_overwrite_content(1, 0).len(), 4096);
        assert_ne!(
            measured_overwrite_content(1, 0),
            measured_overwrite_content(1, 1)
        );
        assert_ne!(
            measured_overwrite_content(1, 4),
            measured_overwrite_content(2, 0)
        );
        assert_ne!(
            measured_overwrite_content(1, 0)[..3000],
            first_file_content()[..]
        );
    }

    #[test]
    fn local_payload_constant_equals_the_implementation() {
        assert_eq!(
            u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("载荷容量"),
            DATA_UNIT_PAYLOAD_BYTES
        );
    }

    #[test]
    fn a37_interval_includes_both_edges() {
        assert!(a37_holds(3_500_000));
        assert!(a37_holds(4_000_000));
        assert!(!a37_holds(3_499_999));
        assert!(!a37_holds(4_000_001));
        assert_eq!(
            bytes_per_slot_per_device_millionths(2 * 211_968 * 29 / 8, 4 * GIBIBYTE),
            3_625_000,
            "4 GiB 盘单元区 211968 槽，每盘每槽 3.625 字节"
        );
    }

    #[test]
    fn a_shard_file_counts_as_finished_only_when_its_last_line_counts_every_line() {
        let directory =
            std::env::temp_dir().join(format!("e183-shard-file-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("临时目录");
        let path = directory.join("h5-example");
        let lines = vec!["E7RESULT name=a".to_string(), "E7RESULT name=b".to_string()];
        write_shard_file_atomically(&path, "h5-example", &lines).expect("写得成");
        let text = std::fs::read_to_string(&path).expect("读得回");
        assert_eq!(
            text.lines().last(),
            Some("E7RESULT name=shard_done shard=h5-example lines=3")
        );
        assert_eq!(
            lines_of_a_finished_shard(&text, "h5-example"),
            Some(lines.clone())
        );
        assert_eq!(
            lines_of_a_finished_shard(&text, "h5-other"),
            None,
            "片名对不上不算"
        );
        let cut: String = text
            .lines()
            .skip(1)
            .map(|line| format!("{line}\n"))
            .collect();
        assert_eq!(
            lines_of_a_finished_shard(&cut, "h5-example"),
            None,
            "行数对不上不算"
        );
        let leftovers = std::fs::read_dir(&directory).expect("列目录").count();
        std::fs::remove_dir_all(&directory).expect("删临时目录");
        assert_eq!(leftovers, 1, "临时名已改名，目录里只剩片文件");
    }

    #[test]
    fn result_lines_parse_into_named_fields_and_the_determinism_check_skips_only_timing_fields() {
        let line = ResultLine::parse(
            "E7RESULT name=h5_mount shard=x mount=3 sweep_matches_the_threshold=true",
        )
        .expect("产物行");
        assert_eq!(line.name(), Some("h5_mount"));
        assert_eq!(line.number("mount"), Some(3));
        assert_eq!(line.truth("sweep_matches_the_threshold"), Some(true));
        assert_eq!(ResultLine::parse("name=no_prefix"), None);
        let first = [
            ResultLine::parse("E7RESULT name=a shard=one mount_nanoseconds=5 rows0=7").expect("行"),
        ];
        let same = [
            ResultLine::parse("E7RESULT name=a shard=two mount_nanoseconds=9 rows0=7").expect("行"),
        ];
        let other = [
            ResultLine::parse("E7RESULT name=a shard=two mount_nanoseconds=9 rows0=8").expect("行"),
        ];
        assert_eq!(
            first_line_differing_outside_the_timing_fields(&first, &same),
            None
        );
        assert_eq!(
            first_line_differing_outside_the_timing_fields(&first, &other),
            Some(1)
        );
        assert_eq!(
            first_line_differing_outside_the_timing_fields(&first, &[]),
            Some(1),
            "少一行算不同"
        );
    }

    #[test]
    fn a_measured_window_reports_the_buffer_allocated_inside_it() {
        if !is_the_quiet_window_child() {
            assert_the_quiet_window_child_passes(MEASURED_WINDOW_TEST);
            return;
        }
        let (length, window) = measure_in_a_window(&COUNTING_ALLOCATOR, || {
            let buffer = vec![3u8; 8 << 20];
            std::hint::black_box(&buffer);
            buffer.len()
        });
        assert_eq!(length, 8 << 20);
        assert!(window.peak_over_baseline_bytes >= 8 << 20, "{window:?}");
    }

    /// 合成的一片 H2：分配耗时 = 10 × 记录数（与容量无关、随记录数线性），发布耗时与发布峰值 ∝ 容量，分配峰值恒 64，
    /// 拷贝字节 = 每盘每槽 3.625 字节（A37 区间里）、拷贝耗时 ∝ 容量，PC-平 恒 1 MiB。
    fn synthetic_capacity_shard_lines(cell: CapacityCell) -> Vec<String> {
        let records = 100 + cell.sequentially_written_units;
        let mebibytes = cell.device_bytes / MEBIBYTE;
        let copy_bytes =
            2 * unit_area_slots(cell.device_bytes, JOURNAL_RING_DEFAULT_BYTES) * 29 / 8;
        let mut lines = vec![
            "E7RESULT name=h2_config shard=synthetic binary_sha256=same".to_string(),
            "E7RESULT name=h2_steps shard=synthetic outcome=ok".to_string(),
            "E7RESULT name=positive_control_heap shard=synthetic peak_over_baseline_bytes=67108864 holds=true".to_string(),
            format!("E7RESULT name=h2_allocator_copy shard=synthetic records={records} copy_bytes={copy_bytes} copy_nanoseconds={mebibytes} bytes_per_slot_per_device_millionths=3625000"),
        ];
        for call in 0..5 {
            lines.push(format!("E7RESULT name=h2_allocate shard=synthetic call={call} nanoseconds={} peak_over_baseline_bytes=64 baseline_bytes=7 in_use_after_bytes=7 slots_from_unit_area_start=0", 10 * records));
        }
        lines.push("E7RESULT name=positive_control_flat shard=synthetic nanoseconds=1 peak_over_baseline_bytes=1048576 baseline_bytes=7 in_use_after_bytes=7".to_string());
        for call in 0..5 {
            lines.push(format!("E7RESULT name=h2_publish shard=synthetic call={call} outcome=ok floor_raises=0 nanoseconds={mebibytes} peak_over_baseline_bytes={} baseline_bytes=7 in_use_after_bytes=7 records_after={records}", copy_bytes));
        }
        lines
    }

    fn parsed(lines: &[String]) -> Vec<ResultLine> {
        lines
            .iter()
            .filter_map(|line| ResultLine::parse(line))
            .collect()
    }

    fn synthetic_capacity_statuses() -> CapacityStatuses {
        capacity_shards()
            .into_iter()
            .map(|(_, cell, round)| {
                (
                    cell,
                    round,
                    status_of_a_capacity_shard(
                        &parsed(&synthetic_capacity_shard_lines(cell)),
                        "same",
                    ),
                )
            })
            .collect()
    }

    #[test]
    fn a_capacity_shard_reads_back_every_measured_field() {
        let cell = capacity_cells()[1];
        let CellRoundStatus::Measured(reading) =
            status_of_a_capacity_shard(&parsed(&synthetic_capacity_shard_lines(cell)), "same")
        else {
            panic!("合成的一片应当读成「量到了」");
        };
        assert_eq!(reading.records, 100);
        assert_eq!(reading.allocate.len(), 5);
        assert_eq!(reading.publish.len(), 5);
        assert_eq!(
            CapacityQuantity::AllocateNanoseconds.round_median(&reading),
            Some(1000)
        );
        assert_eq!(
            CapacityQuantity::PublishNanoseconds.round_median(&reading),
            Some(65536)
        );
        assert_eq!(
            CapacityQuantity::FlatControlPeakOverBaselineBytes.round_median(&reading),
            Some(1 << 20)
        );
    }

    #[test]
    fn a_capacity_shard_with_a_missing_line_or_another_binary_is_not_measured_rather_than_zero() {
        let cell = capacity_cells()[0];
        let mut lines = synthetic_capacity_shard_lines(cell);
        lines.retain(|line| !line.contains("name=h2_allocator_copy"));
        assert_eq!(
            status_of_a_capacity_shard(&parsed(&lines), "same"),
            CellRoundStatus::NotMeasured("fields_missing".to_string())
        );
        assert_eq!(
            status_of_a_capacity_shard(&parsed(&synthetic_capacity_shard_lines(cell)), "other"),
            CellRoundStatus::BinaryOtherThanTheMerge("same".to_string())
        );
    }

    #[test]
    fn a_round_whose_publish_failed_gives_no_publish_quantity() {
        let cell = capacity_cells()[0];
        let lines: Vec<String> = synthetic_capacity_shard_lines(cell)
            .into_iter()
            .map(|line| {
                if line.contains("call=2 outcome=ok") {
                    line.replace("outcome=ok", "outcome=failed:NoSpace")
                } else {
                    line
                }
            })
            .collect();
        let status = status_of_a_capacity_shard(&parsed(&lines), "same");
        let reading = status.reading().expect("量到了");
        assert_eq!(
            CapacityQuantity::PublishNanoseconds.round_median(reading),
            None
        );
        assert_eq!(
            CapacityQuantity::PublishPeakOverBaselineBytes.round_median(reading),
            None
        );
        assert_eq!(
            CapacityQuantity::AllocateNanoseconds.round_median(reading),
            Some(1000)
        );
    }

    #[test]
    fn rounds_that_differ_in_a_deterministic_field_are_named() {
        let cell = capacity_cells()[0];
        let first =
            status_of_a_capacity_shard(&parsed(&synthetic_capacity_shard_lines(cell)), "same");
        let changed_lines: Vec<String> = synthetic_capacity_shard_lines(cell)
            .into_iter()
            .map(|line| {
                if line.contains("name=h2_allocate shard=synthetic call=4") {
                    line.replace("peak_over_baseline_bytes=64", "peak_over_baseline_bytes=65")
                } else {
                    line
                }
            })
            .collect();
        let second = status_of_a_capacity_shard(&parsed(&changed_lines), "same");
        let readings = [
            first.reading().expect("量到了"),
            second.reading().expect("量到了"),
        ];
        assert_eq!(
            fields_differing_across_rounds(&readings),
            vec!["allocate_peaks"]
        );
        assert!(fields_differing_across_rounds(&[readings[0], readings[0]]).is_empty());
    }

    fn verdict_line(lines: &[String], item: &str) -> ResultLine {
        parsed(lines)
            .into_iter()
            .find(|line| line.name() == Some("verdict") && line.field("item") == Some(item))
            .expect("有判决行")
    }

    #[test]
    fn capacity_summary_judges_every_row_of_answer_known_synthetic_readings() {
        let mut emitter = Emitter::collecting_lines();
        let end = emit_capacity_summary(&mut emitter, &synthetic_capacity_statuses());
        assert_eq!(end, MergeEnd::Complete);
        let lines = emitter.into_collected_lines();
        let verdict = verdict_line(&lines, "h2_summary");
        let expected = [
            ("r2a_class", "basically_unrelated"),
            ("r2b_class", "linear"),
            ("r2d_allocate_class", "basically_unrelated"),
            ("r2d_publish_class", "linear"),
            ("r2e_allocate_class", "linear"),
            ("r2e_publish_class", "x5_as_registered_cannot_be_made"),
            ("r2e_publish_peak_class", "x5_as_registered_cannot_be_made"),
            ("r2c_class", "linear"),
            ("completed_capacities", "5"),
            ("fewer_than_four_capacities", "false"),
            ("positive_control_linear_copy_bytes_holds", "true"),
            ("positive_control_linear_copy_time_holds", "true"),
            ("positive_control_flat_holds", "true"),
            ("heap_control_failed_shards", "0"),
            ("unstable_slope_rows", "none"),
            ("predictions_contradicted", "none"),
            ("discrimination_self_proof_holds", "true"),
            ("hidden_state_cells", "0"),
            ("shards_not_run_by_the_wrapper", "0"),
        ];
        for (key, value) in expected {
            assert_eq!(verdict.field(key), Some(value), "{key}");
        }
        let largest = parsed(&lines)
            .into_iter()
            .find(|line| line.name() == Some("h2_largest_disk"))
            .expect("有这一行");
        assert_eq!(
            largest.field("largest_is_the_ladder_endpoint"),
            Some("true")
        );
        let occupancy: Vec<ResultLine> = parsed(&lines)
            .into_iter()
            .filter(|line| line.name() == Some("h2_occupancy_sensitivity"))
            .collect();
        assert_eq!(occupancy.len(), 2);
        let agreement_of = |row: &str| {
            occupancy
                .iter()
                .find(|line| line.field("row") == Some(row))
                .and_then(|line| line.field("agreement"))
        };
        assert_eq!(agreement_of("r2a"), Some("two_points_agree"));
        assert_eq!(
            agreement_of("r2b"),
            Some("not_judged_x5_as_registered_cannot_be_made"),
            "占用格上的发布不是登记的 X5：不拿它同基线比类"
        );
    }

    #[test]
    fn the_publish_rows_on_cells_with_many_units_have_no_class_while_the_ladder_keeps_its_class() {
        let statuses = synthetic_capacity_statuses();
        let judgement_of = |row: &str| {
            capacity_slope_rows()
                .into_iter()
                .find(|slope_row| slope_row.row == row)
                .map(|slope_row| judge_slope_row(&statuses, slope_row))
                .expect("行名取自 capacity_slope_rows")
        };
        for row in ["r2e_publish", "r2e_publish_peak", "r2b_at_occupancy"] {
            let judgement = judgement_of(row);
            assert_eq!(
                judgement.class,
                ClassAcrossRounds::RegisteredPublishCannotBeMade,
                "{row}"
            );
            assert!(
                judgement.slopes.iter().all(Option::is_none),
                "{row} 不当 0 也不出斜率"
            );
        }
        for row in ["r2b", "r2d_publish"] {
            assert_eq!(
                judgement_of(row).class,
                ClassAcrossRounds::Stable(SlopeClass::Linear),
                "{row}"
            );
        }
        assert_eq!(
            judgement_of("r2e_allocate").class,
            ClassAcrossRounds::Stable(SlopeClass::Linear),
            "分配那一行不受 X5 影响"
        );
    }

    #[test]
    fn a_cell_value_line_says_whether_its_publish_is_the_registered_x5() {
        let mut emitter = Emitter::collecting_lines();
        emit_capacity_summary(&mut emitter, &synthetic_capacity_statuses());
        let cells: Vec<ResultLine> = parsed(&emitter.into_collected_lines())
            .into_iter()
            .filter(|line| line.name() == Some("h2_cell_values"))
            .collect();
        assert_eq!(cells.len(), 9);
        for line in &cells {
            let units = line.number("sequentially_written_units").expect("有单元数");
            assert_eq!(
                line.truth("publish_is_the_registered_single_unit_overwrite"),
                Some(units == 0),
                "{:?}",
                line.field("cell")
            );
        }
    }

    #[test]
    fn a_largest_capacity_beyond_the_memory_upper_limit_leaves_four_completed_capacities() {
        let statuses: CapacityStatuses = synthetic_capacity_statuses()
            .into_iter()
            .map(|(cell, round, status)| {
                if cell.device_bytes == 16 * TEBIBYTE - MEBIBYTE {
                    (cell, round, CellRoundStatus::BeyondTheMemoryUpperLimit(250))
                } else {
                    (cell, round, status)
                }
            })
            .collect();
        let mut emitter = Emitter::collecting_lines();
        emit_capacity_summary(&mut emitter, &statuses);
        let lines = emitter.into_collected_lines();
        let verdict = verdict_line(&lines, "h2_summary");
        assert_eq!(verdict.field("completed_capacities"), Some("4"));
        assert_eq!(verdict.field("fewer_than_four_capacities"), Some("false"));
        assert_eq!(verdict.field("r2b_class"), Some("linear"), "四档照样判出类");
        let largest = parsed(&lines)
            .into_iter()
            .find(|line| line.name() == Some("h2_largest_disk"))
            .expect("有这一行");
        assert_eq!(
            largest.field("largest_completed_mebibytes"),
            Some("4194304")
        );
        assert_eq!(
            largest.field("beyond_the_memory_upper_limit_mebibytes"),
            Some("16777215")
        );
    }

    #[test]
    fn a_deterministic_field_that_changes_between_rounds_stops_the_capacity_merge() {
        let statuses: CapacityStatuses = synthetic_capacity_statuses()
            .into_iter()
            .map(|(cell, round, status)| {
                let changed = match status {
                    CellRoundStatus::Measured(mut reading)
                        if round == 3 && cell == capacity_cells()[0] =>
                    {
                        reading.copy_bytes += 1;
                        CellRoundStatus::Measured(reading)
                    }
                    unchanged @ (CellRoundStatus::Measured(_)
                    | CellRoundStatus::NotMeasured(_)
                    | CellRoundStatus::BeyondTheMemoryUpperLimit(_)
                    | CellRoundStatus::NotRunByTheWrapper(_)
                    | CellRoundStatus::Missing
                    | CellRoundStatus::BinaryOtherThanTheMerge(_)) => unchanged,
                };
                (cell, round, changed)
            })
            .collect();
        let mut emitter = Emitter::collecting_lines();
        let end = emit_capacity_summary(&mut emitter, &statuses);
        assert!(matches!(end, MergeEnd::Stopped { .. }), "{end:?}");
        assert_eq!(
            verdict_line(&emitter.into_collected_lines(), "h2_summary").field("hidden_state_cells"),
            Some("1")
        );
    }

    #[test]
    fn a_missing_shard_with_a_wrapper_exit_of_250_reads_as_beyond_the_memory_upper_limit() {
        let directory =
            std::env::temp_dir().join(format!("e183-h2-statuses-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("临时目录");
        let (first_shard, _, _) = capacity_shards()[0].clone();
        let (second_shard, _, _) = capacity_shards()[1].clone();
        std::fs::write(
            directory.join(format!("{first_shard}.wrapper-exit")),
            "250\n",
        )
        .expect("写退出码");
        std::fs::write(
            directory.join(format!("{second_shard}.wrapper-exit")),
            "1\n",
        )
        .expect("写退出码");
        let (third_shard, _, _) = capacity_shards()[2].clone();
        std::fs::write(
            directory.join(format!("{third_shard}.wrapper-exit")),
            "252\n",
        )
        .expect("写退出码");
        let (fourth_shard, _, _) = capacity_shards()[3].clone();
        std::fs::write(
            directory.join(format!("{fourth_shard}.wrapper-exit")),
            "254\n",
        )
        .expect("写退出码");
        let statuses = read_capacity_statuses(&directory, "same");
        std::fs::remove_dir_all(&directory).expect("删临时目录");
        assert_eq!(
            statuses[0].2,
            CellRoundStatus::BeyondTheMemoryUpperLimit(250)
        );
        assert_eq!(statuses[1].2, CellRoundStatus::Missing);
        assert_eq!(
            statuses[2].2,
            CellRoundStatus::NotRunByTheWrapper(252),
            "排不上不是这一片撞了自己的上限"
        );
        assert_eq!(statuses[3].2, CellRoundStatus::NotRunByTheWrapper(254));
    }

    #[test]
    fn only_a_wrapper_exit_of_250_reads_as_beyond_the_memory_upper_limit() {
        assert_eq!(
            status_of_an_unfinished_shard(Some(250)),
            CellRoundStatus::BeyondTheMemoryUpperLimit(250)
        );
        for exit_code in [251, 252, 253, 254] {
            assert_eq!(
                status_of_an_unfinished_shard(Some(exit_code)),
                CellRoundStatus::NotRunByTheWrapper(exit_code)
            );
        }
        for other in [Some(0), Some(1), Some(137), Some(249), Some(255), None] {
            assert_eq!(
                status_of_an_unfinished_shard(other),
                CellRoundStatus::Missing
            );
        }
    }

    #[test]
    fn a_largest_capacity_the_wrapper_did_not_run_is_not_beyond_the_limit_and_is_counted() {
        let statuses: CapacityStatuses = synthetic_capacity_statuses()
            .into_iter()
            .map(|(cell, round, status)| {
                if cell.device_bytes == 16 * TEBIBYTE - MEBIBYTE {
                    (cell, round, CellRoundStatus::NotRunByTheWrapper(252))
                } else {
                    (cell, round, status)
                }
            })
            .collect();
        let mut emitter = Emitter::collecting_lines();
        emit_capacity_summary(&mut emitter, &statuses);
        let lines = emitter.into_collected_lines();
        let verdict = verdict_line(&lines, "h2_summary");
        assert_eq!(verdict.field("shards_not_run_by_the_wrapper"), Some("5"));
        assert_eq!(verdict.field("completed_capacities"), Some("4"));
        let largest = parsed(&lines)
            .into_iter()
            .find(|line| line.name() == Some("h2_largest_disk"))
            .expect("有这一行");
        assert_eq!(
            largest.field("beyond_the_memory_upper_limit_mebibytes"),
            Some("none"),
            "252 不记「内存上限之外」"
        );
        assert!(parsed(&lines).iter().any(|line| {
            line.name() == Some("h2_shard_status")
                && line.field("status") == Some("not_run_by_the_wrapper:wrapper_exit_252")
        }));
    }

    #[test]
    fn anchor_model_reproduces_every_registered_script_row() {
        for anchor in &REGISTERED_SWEEP_HISTORY_ANCHORS {
            let history = sweep_history_by_the_anchor_model(
                anchor.slots_per_region,
                Some(anchor.threshold_rows),
                anchor.unmount_kind,
                2 * anchor.threshold_rows + 3 * anchor.slots_per_region + 20,
            );
            assert!(
                anchor.is_reproduced_by(&history),
                "{anchor:?} 模型给的：{:?} 末 txg {}",
                history.sweeps(),
                history.final_txg
            );
        }
    }

    #[test]
    fn anchor_model_without_a_threshold_never_sweeps_and_rows0_counts_the_mounts() {
        let history =
            sweep_history_by_the_anchor_model(8, None, UnmountKind::Normal, SWEEP_CONTROL_MOUNTS);
        assert_eq!(history.mounts.len(), 400);
        assert!(history.sweeps().is_empty());
        for (index, mount) in history.mounts.iter().enumerate() {
            assert_eq!(
                mount.rows0,
                u64::try_from(index).expect("序号") + 1,
                "第 {} 次挂载",
                index + 1
            );
        }
        assert_eq!(
            history.peak_rows0(),
            400,
            "400 次挂载 rows0 到 400，跑前登记 PC-不扫 写的是 401，修订改成这个数"
        );
        assert_eq!(
            history.mounts.last().map(|mount| mount.rows0),
            Some(SWEEP_CONTROL_REVISED_FINAL_ROWS0),
            "装置判的修订值就是锚点模型第 400 次挂载的 rows0"
        );
        assert_eq!(SWEEP_CONTROL_ORIGINALLY_REGISTERED_FINAL_ROWS0, 401);
    }

    #[test]
    fn a_forced_preflight_line_gives_its_summary_with_underscores_and_a_met_line_gives_none() {
        assert_eq!(
            forced_summary_of("forced inputs unchanged since last run"),
            Some("inputs_unchanged_since_last_run".to_string())
        );
        assert_eq!(forced_summary_of("met abc123"), None);
        assert_eq!(forced_summary_of(""), None);
    }

    #[test]
    fn the_no_sweep_control_judges_its_final_rows0_against_the_revised_value_only() {
        assert!(control_final_rows0_equals_the_revised_registered_value(400));
        assert!(!control_final_rows0_equals_the_revised_registered_value(
            401
        ));
        assert!(!control_final_rows0_equals_the_revised_registered_value(
            399
        ));
    }

    #[test]
    fn the_chain_read_is_one_unit_per_page_of_369_rows_and_never_less_than_one_page() {
        assert_eq!(instance_table_chain_read_bytes(0), 32768);
        assert_eq!(instance_table_chain_read_bytes(1), 32768);
        assert_eq!(instance_table_chain_read_bytes(369), 32768);
        assert_eq!(instance_table_chain_read_bytes(370), 2 * 32768);
        assert_eq!(instance_table_chain_read_bytes(738), 2 * 32768);
        assert_eq!(instance_table_chain_read_bytes(739), 3 * 32768);
        assert_eq!(instance_table_chain_pages_read(739), 3);
    }

    #[test]
    fn the_chain_read_arm_holds_only_when_the_read_difference_equals_the_expected_bytes() {
        let difference = |read_bytes_difference| {
            Some(SweepDifferential {
                write_bytes_difference: 0,
                read_bytes_difference,
            })
        };
        assert!(chain_read_differential_holds(
            difference(32768),
            Some(32768)
        ));
        assert!(!chain_read_differential_holds(difference(0), Some(32768)));
        assert!(!chain_read_differential_holds(
            difference(32769),
            Some(32768)
        ));
        assert!(!chain_read_differential_holds(
            difference(32767),
            Some(32768)
        ));
        assert!(!chain_read_differential_holds(None, Some(32768)));
        assert!(!chain_read_differential_holds(difference(32768), None));
    }

    #[test]
    fn the_chain_read_line_takes_the_expected_bytes_from_the_model_rows_and_not_from_the_reading() {
        let reading = Some(SweepDifferential {
            write_bytes_difference: 0,
            read_bytes_difference: 32768,
        });
        let line = chain_read_differential_line("cell", 1, reading, Some(1));
        assert_eq!(
            line,
            "name=h5_chain_read_differential shard=cell mount=1 read_bytes_difference=32768 model_rows0=1 expected_chain_pages=1 expected_chain_read_bytes=32768 read_bytes_difference_equals_the_chain_read=true"
        );
        let other_rows = chain_read_differential_line("cell", 1, reading, Some(370));
        assert!(
            other_rows.contains(
                "expected_chain_read_bytes=65536 read_bytes_difference_equals_the_chain_read=false"
            ),
            "{other_rows}"
        );
        let no_model = chain_read_differential_line("cell", 1, reading, None);
        assert!(
            no_model.contains(
                "expected_chain_read_bytes=none read_bytes_difference_equals_the_chain_read=false"
            ),
            "{no_model}"
        );
    }

    #[test]
    fn a_sweep_is_due_only_strictly_above_the_threshold() {
        assert!(!sweep_is_due(369, 369));
        assert!(sweep_is_due(370, 369));
        assert!(!sweep_is_due(0, 0));
    }

    #[test]
    fn the_ring_at_the_judgement_is_the_ring_roots_before_the_sweep_publish() {
        let owners: BTreeMap<u64, u32> = (0..30u64)
            .map(|txg| (txg, u32::try_from(txg / 5).expect("小")))
            .collect();
        let in_the_ring =
            instances_in_the_ring_when_the_sweep_judges(&owners, 25, 12).expect("都读到过");
        assert_eq!(in_the_ring, BTreeSet::from([2, 3, 4]), "txg 13 … 24 的实例");
        let mut missing = owners.clone();
        missing.remove(&13);
        assert_eq!(
            instances_in_the_ring_when_the_sweep_judges(&missing, 25, 12),
            None
        );
        assert_eq!(
            instances_in_the_ring_when_the_sweep_judges(&missing, 26, 12),
            Some(BTreeSet::from([2, 3, 4, 5])),
            "环是 txg 14 … 25，13 不在环里"
        );
        let should_delete = rows_the_sweep_should_delete(&[1, 2, 3, 4, 5], &in_the_ring);
        assert_eq!(should_delete, BTreeSet::from([1, 5]));
    }

    fn history_mount(
        rows0: u64,
        deleted: Option<Vec<u32>>,
        rows_left: Option<u64>,
        heads_erased: usize,
    ) -> HistoryMount {
        HistoryMount {
            number: 1,
            instance: 9,
            rows_written: 1,
            rows0,
            rows0_by_the_allocator: rows0,
            rows_before_the_sweep: (1..=u32::try_from(rows0).expect("小")).collect(),
            sweep: SweepAtTheMount {
                text: if deleted.is_some() {
                    "rows_deleted".to_string()
                } else {
                    "not_swept_at_or_below_the_threshold".to_string()
                },
                rows_deleted: deleted,
                heads_erased,
                handed_to_the_background: false,
            },
            rows_left,
            checkpoint_txg: 40,
            mount_nanoseconds: 1,
            written_and_read_bytes: (10, 20),
        }
    }

    #[test]
    fn each_mount_check_turns_red_on_its_own_fault() {
        let arm = SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows: 3 };
        let ring = BTreeSet::from([3u32, 4]);
        let model = AnchorModelMount {
            rows0: 4,
            sweep: Some((2, 2)),
            checkpoint_txg_when_the_mount_returns: 40,
        };
        let good = history_mount(4, Some(vec![1, 2]), Some(2), 0);
        let checks = checks_of_a_history_mount(&good, arm, Some(&model), Some(&ring));
        assert!(checks.all_hold(), "{checks:?}");
        let wrong_rows = history_mount(4, Some(vec![1]), Some(3), 0);
        let wrong_rows_checks =
            checks_of_a_history_mount(&wrong_rows, arm, Some(&model), Some(&ring));
        assert_eq!(
            wrong_rows_checks.failed_names(),
            vec!["deleted_rows_match_the_ring", "a34_a35_trajectory"]
        );
        let one_row_more = history_mount(5, Some(vec![1, 2]), Some(2), 0);
        assert!(
            checks_of_a_history_mount(&one_row_more, arm, Some(&model), Some(&ring))
                .failed_names()
                .contains(&"a34_a35_trajectory"),
            "rows0 比模型多一行：轨迹对不上"
        );
        let erased = history_mount(4, Some(vec![1, 2]), Some(2), 1);
        assert_eq!(
            checks_of_a_history_mount(&erased, arm, Some(&model), Some(&ring)).failed_names(),
            vec!["no_head_erased"]
        );
        let swept_at_the_threshold = history_mount(3, Some(vec![1]), Some(2), 0);
        assert!(
            checks_of_a_history_mount(&swept_at_the_threshold, arm, None, Some(&ring))
                .failed_names()
                .contains(&"sweep_matches_the_threshold")
        );
        let not_swept_above = history_mount(4, None, None, 0);
        assert!(
            checks_of_a_history_mount(&not_swept_above, arm, Some(&model), Some(&ring))
                .failed_names()
                .contains(&"sweep_matches_the_threshold")
        );
        let unknown_ring = checks_of_a_history_mount(&good, arm, Some(&model), None);
        assert_eq!(
            unknown_ring.deleted_rows_match_the_ring,
            Some(false),
            "环里有 txg 没读到过：K34 判不了，按不符停"
        );
    }

    #[test]
    fn history_tally_counts_rows_above_366_two_page_mounts_and_the_plus_one_run() {
        let arm = SweepArm::AtTheEndOfTheMountAboveTheThreshold {
            threshold_rows: 366,
        };
        let mut tally = HistoryTally::default();
        for rows0 in 365..=368 {
            tally.note(&history_mount(rows0, None, None, 0), arm);
        }
        assert_eq!(tally.mounts, 4);
        assert_eq!(tally.peak_rows0, 368);
        assert_eq!(tally.mounts_with_rows0_above_366, 2);
        assert_eq!(tally.mounts_with_rows0_above_the_threshold, 2);
        assert_eq!(tally.peak_chain_pages, 2);
        assert_eq!(tally.mounts_with_two_or_more_chain_pages, 2);
        assert!(!tally.rows0_plus_one_each_mount, "第一次挂载 rows0 不是 1");
        let mut counting = HistoryTally::default();
        for rows0 in 1..=5 {
            counting.note(
                &history_mount(rows0, None, None, 0),
                SweepArm::HandedToTheBackground,
            );
        }
        assert!(counting.rows0_plus_one_each_mount);
        counting.note(
            &history_mount(7, None, None, 0),
            SweepArm::HandedToTheBackground,
        );
        assert!(!counting.rows0_plus_one_each_mount, "跳了一行");
    }

    fn threshold_summary(
        threshold_rows: u64,
        writable_mounts: u64,
        total_write_bytes: u64,
    ) -> ThresholdCellSummary {
        ThresholdCellSummary {
            cell: sweep_history_cells()[0],
            threshold_rows,
            writable_mounts,
            total_write_bytes,
            peak_chain_pages: 1,
            differential_control_holds: true,
        }
    }

    #[test]
    fn write_amplification_divides_total_writes_by_writable_mounts_and_accepts_exactly_1_10() {
        let smallest = threshold_summary(64, 100, 1_000_000);
        let exactly = threshold_summary(365, 200, 2_200_000);
        let above = threshold_summary(369, 200, 2_200_200);
        assert_eq!(
            write_amplification_ratio_permille(&exactly, &smallest),
            1100
        );
        assert!(write_amplification_acceptable(
            1100,
            WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE
        ));
        assert_eq!(
            write_amplification_ratio_permille(&above, &smallest),
            1100,
            "千分比向下取整"
        );
        assert!(!write_amplification_acceptable(
            1101,
            WRITE_AMPLIFICATION_ACCEPTABLE_PERMILLE
        ));
        let picked = smallest_write_per_mount(&[exactly, smallest, above]).expect("有一格");
        assert_eq!(picked.threshold_rows, 64, "每次挂载 10000 字节最小");
        let mut void = smallest;
        void.differential_control_holds = false;
        assert_eq!(
            smallest_write_per_mount(&[void, exactly]).map(|cell| cell.threshold_rows),
            Some(365),
            "PC-差分 没过的格不参加"
        );
    }

    #[test]
    fn the_differential_subtracts_the_background_mount_from_the_threshold_mount() {
        let difference = sweep_differential((5000, 300), (1000, 100));
        assert_eq!(
            difference,
            SweepDifferential {
                write_bytes_difference: 4000,
                read_bytes_difference: 200
            }
        );
        assert!(differential_control_above_holds(Some(difference)));
        assert!(!differential_control_below_holds(Some(difference)));
        let none = sweep_differential((1000, 100), (1000, 100));
        assert!(differential_control_below_holds(Some(none)));
        let read_only = sweep_differential((1000, 300), (1000, 100));
        assert!(
            !differential_control_below_holds(Some(read_only)),
            "写字节之差为 0 而读字节之差不为 0：原臂照字面不过，不放宽"
        );
        assert!(!differential_control_above_holds(Some(none)));
        assert!(!differential_control_below_holds(None), "差分没做成不算过");
    }

    #[test]
    fn moving_the_acceptance_boundary_between_two_real_ratios_flips_a_verdict() {
        let (doubled_boundary, flipped) =
            acceptance_boundary_moved_between_real_ratios(&[1000, 1050, 1300]).expect("挪得出");
        assert_eq!(doubled_boundary, 2050, "1000 与 1050 的中点 1025（两倍记）");
        assert_eq!(flipped, 1, "1050 从可接受转成不可接受");
        assert_eq!(
            acceptance_boundary_moved_between_real_ratios(&[1000, 1000]),
            None
        );
        assert!(chain_pages_stay_on_one_page(1));
        assert!(!chain_pages_stay_on_one_page(2));
    }

    /// 合成的 26 片 H5：可写挂载 θ = 64 / 365 / 369 / 738 为 128 / 730 / 738 / 1476 次（与锚点同量级、各不相同，W 除错了分母就看得出），
    /// 每次挂载写字节 12000 / 10500 / 10000 / 10000，片数峰值 1 / 1 / 2 / 3；
    /// 答案：只有 365 两关都过，越过上界的是 369、738，各点判定相同。
    fn synthetic_history_shards(repeat_rows0: u64) -> Vec<HistoryShard> {
        synthetic_history_shards_with_the_two_differential_arms(repeat_rows0, true, true)
    }

    /// 同上，原臂（写、读字节之差都 0、越过那次写字节之差 > 0）与新臂（读字节之差等于读链字节数）各给各的答案，两臂互不相干。
    fn synthetic_history_shards_with_the_two_differential_arms(
        repeat_rows0: u64,
        original_arm_holds: bool,
        chain_read_arm_holds: bool,
    ) -> Vec<HistoryShard> {
        sweep_shards()
            .into_iter()
            .map(|(shard, cell)| {
                let summary = match cell.arm {
                    SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows } => {
                        let (mounts, per_mount, pages) = match threshold_rows {
                            64 => (128, 12_000, 1),
                            365 => (730, 10_500, 1),
                            369 => (738, 10_000, 2),
                            _ => (1476, 10_000, 3),
                        };
                        format!("E7RESULT name=h5_cell_summary shard={shard} writable_mounts={mounts} sweeps_done=2 total_write_bytes={} peak_chain_pages={pages} positive_control_differential_below_the_threshold={original_arm_holds} positive_control_differential_at_the_first_sweep=true sweep_mount_nanoseconds=7", mounts * per_mount)
                    }
                    SweepArm::HandedToTheBackground => format!("E7RESULT name=h5_cell_summary shard={shard} writable_mounts=400 total_write_bytes=1 peak_chain_pages=2 positive_control_no_sweep_publish=true positive_control_rows0_plus_one_each_mount=true final_rows0=400 final_rows0_equals_the_registered_value=true"),
                };
                let rows0 = if shard.starts_with(SWEEP_REPEAT_SHARD_PREFIX) { repeat_rows0 } else { 1 };
                let mut lines = vec![
                    format!("E7RESULT name=h5_config shard={shard} binary_sha256=same"),
                    format!("E7RESULT name=h5_mount shard={shard} mount=1 rows0={rows0} mount_nanoseconds=3"),
                ];
                if cell.arm.threshold_rows().is_some() {
                    lines.push(format!("E7RESULT name=h5_chain_read_differential shard={shard} mount=1 read_bytes_difference=32768 model_rows0=1 expected_chain_pages=1 expected_chain_read_bytes=32768 read_bytes_difference_equals_the_chain_read={chain_read_arm_holds}"));
                }
                lines.push(summary);
                (shard, cell, parsed(&lines))
            })
            .collect()
    }

    #[test]
    fn sweep_summary_judges_the_page_reading_the_write_amplification_and_the_candidate_clause_on_answer_known_synthetic_shards(
    ) {
        let mut emitter = Emitter::collecting_lines();
        let end = emit_sweep_summary(&mut emitter, &synthetic_history_shards(1));
        assert_eq!(end, MergeEnd::Complete);
        let lines = emitter.into_collected_lines();
        let verdict = verdict_line(&lines, "h5_summary");
        let expected = [
            ("positive_control_no_sweep_publish", "true"),
            ("positive_control_rows0_plus_one_each_mount", "true"),
            ("control_final_rows0", "400"),
            (
                "control_final_rows0_equals_the_revised_registered_value",
                "true",
            ),
            ("differential_control_failed_cells", "0"),
            ("chain_read_differential_failed_cells", "0"),
            ("r5b_crossing_thresholds", "369,738"),
            ("points_without_a_candidate", "0"),
            ("geometry_unstable_thresholds", "none"),
            ("discrimination_self_proof_holds", "true"),
            ("repeat_of_the_first_shard_equal_outside_timing", "true"),
        ];
        for (key, value) in expected {
            assert_eq!(verdict.field(key), Some(value), "{key}");
        }
        let points: Vec<ResultLine> = parsed(&lines)
            .into_iter()
            .filter(|line| line.name() == Some("h5_point"))
            .collect();
        assert_eq!(points.len(), 6);
        assert!(points
            .iter()
            .all(|line| line.field("thresholds_passing_both") == Some("365")));
        let sixty_four = parsed(&lines)
            .into_iter()
            .find(|line| {
                line.name() == Some("h5_threshold_verdicts")
                    && line.field("threshold_rows") == Some("64")
            })
            .expect("有 θ = 64 的判定行");
        assert_eq!(
            sixty_four.field("ratio_to_the_smallest_write_per_mount_permille"),
            Some("1200")
        );
        assert_eq!(sixty_four.field("r5c"), Some("not_acceptable"));
    }

    #[test]
    fn the_chain_read_arm_and_the_original_differential_arm_are_counted_apart() {
        let verdict_of = |original_arm_holds, chain_read_arm_holds| {
            let mut emitter = Emitter::collecting_lines();
            emit_sweep_summary(
                &mut emitter,
                &synthetic_history_shards_with_the_two_differential_arms(
                    1,
                    original_arm_holds,
                    chain_read_arm_holds,
                ),
            );
            verdict_line(&emitter.into_collected_lines(), "h5_summary")
        };
        let only_the_new_arm_fails = verdict_of(true, false);
        assert_eq!(
            only_the_new_arm_fails.field("differential_control_failed_cells"),
            Some("0")
        );
        assert_eq!(
            only_the_new_arm_fails.field("chain_read_differential_failed_cells"),
            Some("24")
        );
        let only_the_original_arm_fails = verdict_of(false, true);
        assert_eq!(
            only_the_original_arm_fails.field("differential_control_failed_cells"),
            Some("24")
        );
        assert_eq!(
            only_the_original_arm_fails.field("chain_read_differential_failed_cells"),
            Some("0")
        );
    }

    #[test]
    fn a_threshold_cell_without_a_chain_read_line_counts_as_failed_for_the_new_arm() {
        let mut shards = synthetic_history_shards(1);
        let (_, _, lines) = &mut shards[0];
        lines.retain(|line| line.name() != Some("h5_chain_read_differential"));
        assert_eq!(chain_read_differential_failed_cells(&shards), 1);
    }

    #[test]
    fn a_repeat_of_the_first_shard_that_differs_outside_timing_stops_the_sweep_merge() {
        let mut emitter = Emitter::collecting_lines();
        let end = emit_sweep_summary(&mut emitter, &synthetic_history_shards(2));
        assert!(matches!(end, MergeEnd::Stopped { .. }), "{end:?}");
        let verdict = verdict_line(&emitter.into_collected_lines(), "h5_summary");
        assert_eq!(verdict.field("repeat_first_differing_line"), Some("2"));
    }

    #[test]
    fn sweep_shards_are_twenty_four_cells_the_control_and_the_repeat_of_the_first() {
        let shards = sweep_shards();
        assert_eq!(shards.len(), 26);
        assert_eq!(
            shards[0].0,
            "h5-slots-per-region-4-threshold-64-normal_unmount"
        );
        assert_eq!(shards[23].0, "h5-slots-per-region-16-threshold-738-crash");
        assert_eq!(shards[23].1.mounts_at_most, 2 * 738 + 3 * 16 + 20);
        assert_eq!(shards[24].1.arm, SweepArm::HandedToTheBackground);
        assert_eq!(shards[24].1.mounts_at_most, 400);
        assert_eq!(
            shards[25].0,
            format!("{SWEEP_REPEAT_SHARD_PREFIX}{}", shards[0].0)
        );
    }

    /// 小盘（256 MiB、环 1 MiB）上的清扫历史走产物同一条路：θ = 4、S = 4，两种卸载方式都要走到第二次清扫、每次挂载与锚点模型对得上
    /// （不对就是停机 T5 的那一种：锚点脚本的读法与 `crates/` 的实现不一致，交主 agent 两边查）。
    #[test]
    fn a_short_sweep_history_on_a_small_pool_runs_to_its_second_sweep_in_both_unmount_kinds() {
        for unmount_kind in [UnmountKind::Normal, UnmountKind::Crash] {
            let cell = SweepHistoryCell {
                slots_per_region: 4,
                arm: SweepArm::AtTheEndOfTheMountAboveTheThreshold { threshold_rows: 4 },
                unmount_kind,
                mounts_at_most: 2 * 4 + 3 * 4 + 20,
            };
            let geometry = PoolGeometry {
                device_bytes: 256 * MEBIBYTE,
                journal_ring_bytes: MEBIBYTE,
            };
            let mut emitter = Emitter::collecting_lines();
            let end = run_sweep_history_on_a_pool("small", cell, geometry, "same", &mut emitter);
            let lines = emitter.into_collected_lines();
            assert_eq!(
                end,
                ShardEnd::Finished,
                "{}：\n{}",
                unmount_kind.name(),
                lines.join("\n")
            );
            let summary = parsed(&lines)
                .into_iter()
                .find(|line| line.name() == Some("h5_cell_summary"))
                .expect("有汇总行");
            assert_eq!(
                summary.number("sweeps_done"),
                Some(2),
                "{}",
                unmount_kind.name()
            );
            assert_eq!(
                summary.field("differential_below_the_threshold_write_bytes"),
                Some("0"),
                "没越过阈值的那一次，两次挂载写字节相同"
            );
            assert_eq!(
                summary.truth("positive_control_differential_at_the_first_sweep"),
                Some(true)
            );
            // 新臂的绝对值：第一次挂载（rows0 = 1）阈值那一次比「交给后台」那一次多读一片实例表（32768 字节），这个数另算（片数 × 一片的字节），
            // 不取自被测的读数；原臂（读字节之差 = 0）在这一格上不成立，照字面判、记一次输。
            let chain_read = parsed(&lines)
                .into_iter()
                .find(|line| line.name() == Some("h5_chain_read_differential"))
                .expect("每个阈值格有一条新臂的行");
            assert_eq!(
                chain_read.number("expected_chain_read_bytes"),
                Some(32768),
                "{}",
                unmount_kind.name()
            );
            assert_eq!(
                chain_read.truth("read_bytes_difference_equals_the_chain_read"),
                Some(true),
                "{}：{}",
                unmount_kind.name(),
                lines.join("\n")
            );
            assert_eq!(
                summary.truth("positive_control_differential_below_the_threshold"),
                Some(false),
                "原臂照字面判：读字节之差不为 0，这一格 V35 打中"
            );
            assert_eq!(
                summary.number("peak_rows0"),
                Some(5),
                "rows0 峰值 = θ + 1（A34）"
            );
        }
    }

    #[test]
    fn a_short_history_handed_to_the_background_never_sweeps_and_counts_one_row_per_mount() {
        let cell = SweepHistoryCell {
            slots_per_region: 8,
            arm: SweepArm::HandedToTheBackground,
            unmount_kind: UnmountKind::Normal,
            mounts_at_most: 6,
        };
        let geometry = PoolGeometry {
            device_bytes: 256 * MEBIBYTE,
            journal_ring_bytes: MEBIBYTE,
        };
        let mut emitter = Emitter::collecting_lines();
        let end = run_sweep_history_on_a_pool("small", cell, geometry, "same", &mut emitter);
        let lines = emitter.into_collected_lines();
        assert_eq!(end, ShardEnd::Finished, "{}", lines.join("\n"));
        let summary = parsed(&lines)
            .into_iter()
            .find(|line| line.name() == Some("h5_cell_summary"))
            .expect("有汇总行");
        assert_eq!(
            summary.truth("positive_control_no_sweep_publish"),
            Some(true)
        );
        assert_eq!(
            summary.truth("positive_control_rows0_plus_one_each_mount"),
            Some(true)
        );
        assert_eq!(summary.number("final_rows0"), Some(6));
    }

    /// 小盘（256 MiB、环 1 MiB）文件镜像上的 H2 一格走产物同一条路：n = 0 时 X4 五次都分到、X5 五次都发成；n = 16 时 X4 五次都分到、
    /// 记录数比 n = 0 多出至少 16 条（记录格 X5 的结局照报、不在这里判：多单元文件上覆盖写的行为由产物给）；镜像文件跑完就删。
    #[test]
    fn a_small_capacity_cell_on_file_images_reaches_every_measurement() {
        let mut records_by_units = Vec::new();
        for units in [0u64, 16] {
            let directory =
                std::env::temp_dir().join(format!("e183-h2-cell-{}-{units}", std::process::id()));
            std::fs::create_dir_all(&directory).expect("临时目录");
            let cell = CapacityCell {
                device_bytes: 256 * MEBIBYTE,
                sequentially_written_units: units,
            };
            let mut emitter = Emitter::collecting_lines();
            let end = run_capacity_cell_with_a_journal_ring(
                "small",
                cell,
                1,
                MEBIBYTE,
                &directory,
                "same",
                &mut emitter,
            );
            let left_behind = std::fs::read_dir(&directory).expect("列目录").count();
            std::fs::remove_dir_all(&directory).expect("删临时目录");
            let lines = emitter.into_collected_lines();
            assert_eq!(end, ShardEnd::Finished);
            assert_eq!(left_behind, 0, "镜像文件跑完就删");
            let status = status_of_a_capacity_shard(&parsed(&lines), "same");
            let reading = status
                .reading()
                .unwrap_or_else(|| panic!("n = {units}：{status:?}\n{}", lines.join("\n")));
            assert!(
                reading
                    .allocate
                    .iter()
                    .all(|call| call.slots_from_unit_area_start.is_some()),
                "{lines:?}"
            );
            if units == 0 {
                assert!(
                    reading.publish.iter().all(|call| call.published),
                    "{lines:?}"
                );
            }
            assert!(reading.copy_bytes > 0);
            records_by_units.push(reading.records);
        }
        assert!(
            records_by_units[1] >= records_by_units[0] + 16,
            "{records_by_units:?}"
        );
    }
}
