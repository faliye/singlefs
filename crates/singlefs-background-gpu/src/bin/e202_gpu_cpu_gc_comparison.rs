// admission: always 每跑一次都在今天这份 crates 上现造池、现量挂钟与 CPU，计时每次都是新观测，上一次跑完不替这一次作保
// task-server-gpu-arguments: lockstep concurrent sensitivity stage-three-cost
// task-server-resumable: lockstep concurrent sensitivity
//! E202（GPU与CPU两种模式GC对照组）的装置：跑前登记 `research/prompts/e202-preregistration.md`。
//!
//! 同一负载、同一池，核心层后台入口 `MountedSession::run_compaction_in_the_background` 在 `background_compute=cpu` 与 `background_compute=gpu`
//! 两种挂载参数下各跑，量 GC 挂钟（量 W）、GC 调用窗口里本进程用掉的主机 CPU（进程口径，量 C）、两者成对相减省下的（量 S）、省下的随盘容量的走向
//! （量 T，`judge-trend` 读三档产物判）与阳性对照（量 P：每条臂配一条把每个确认的条目在 CPU 上再判一遍的多判臂）；并发跑另量前台 fsync 延迟（量 F）。
//!
//! 子命令（登记第十四节「起法」）：
//! - `geometry-check --product <文件>`：三档各一条线程造起步镜像、核几何（登记第五节「负载」第 4 条），每档另在 CPU 臂上不计时地交替跑一次、
//!   按第六节「前提 N」的阶梯数出每次跑的批数；不要卡。
//! - `lockstep --reading occupancy --unit-area <2288MiB|4GiB|8GiB> [--user-changes <次数>] [--batch-units <个数>] --product <文件>`：交替跑六条臂。
//! - `concurrent --reading occupancy --unit-area <…> --foreground-interval-milliseconds <毫秒> --product <文件>`：并发跑三条臂。
//! - `sensitivity --batch-units 16 --foreground-interval-milliseconds 500 --product <文件>`：中档几何敏感性两点（登记第八节）。
//! - `judge-trend --product <文件> <三档 lockstep 产物…>`：量 T 的分类与停机条款第 2 条（三档的 crates 版本相同）。
//! - `stage-three-cost --unit-area <2288MiB|4GiB|8GiB> --product <文件>`：丙段代价（登记第十二节修订第 9 条，只供用户定岔路、不进判据）：
//!   几何核同一份起步镜像上 CPU 臂与 GPU 点名一张卡臂各数五遍每段活槽数取中位，再各照产品 greedy 跑一次 GC 调用、报挂钟与其中丙段那一截。
//! - `construction-timing --unit-area <…> --live-data-units <P,…> --chunks <each-device-copies|shared> --product <文件>`：造池分阶段计时
//!   （盘取那一档的大小，P 取给的几个小值，照同一套造法串着造，每阶段一行 `name=construction_phase`：挂钟、步数、每步挂钟的头尾八分之一平均、
//!   版本大小、常驻内存与内存盘块字节）；不跑 GC、不要卡、不进判据，只给造池提速与内存上限定数。
//!
//! 产物是标准输出与 `--product` 那份文件上同样的结果行（`E202 name=<种类> 键=值 …`），末行 `name=done emitted=<行数>`；进度打到标准错误。
//! `--product` 那份文件只追加：同一条命令重起时读它里面已有的 `name=repeat_done` 行，跳过做完的重复（登记第十四节「续跑入口」）。
//!
//! 退出码：0 正常；2 命令行不对；3 停机条款触发（登记第十一节「停机」，产物照打）；4 阳性对照没过（产物照打）；
//! 77 一条臂都没跑到（看不到独显）；78 准入或运行条件不满足（CPU 钟分辨率自检不过、读不到登记的线程数、写不了产物）。
//!
//! 换不掉的那几处：真卡（GPU 臂经产品 GPU 包开本进程看得到的独显）、真钟（单调钟与 CPU 钟）、`/proc`（机器负载与开卡线程）；
//! 判法与计数写成纯函数，同包测试里喂合成输入。
//! 装置住产品 GPU 包（登记第五节「装置住哪」）：只读、只驱动与观测，不改 `crates/` 下别的文件。决定几何与判据门槛的常量写成本地常量、
//! 开跑先与核心层公开的同一个量回比（[`local_constants_disagreeing_with_the_core`]），对不上走停机条款第 1 条。
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{BufRead, Write};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex, Weak};
use std::time::{Duration, Instant};

use singlefs_background_gpu::adapter_inventory::{
    gpu_device_identifier_of_the_pci_bus_address, AdapterDeviceKind, BackgroundGpuSelection,
};
use singlefs_background_gpu::background_gpu::{
    host_cpu_time_of_this_thread, monotonic_raw_time_now,
    open_background_gpu_with_the_memory_leaked_in_this_process, VulkanInstanceOfThisProcess,
};
use singlefs_background_gpu::background_gpu_compute::{
    open_background_gpu_compute, BackgroundGpuComputeOnTheParticipatingGpus,
};
use singlefs_background_gpu::gpus_of_this_host::{
    GpuMemoryLeakedInThisProcess, GpusLeakedWhenTheBackgroundGpuIsDropped,
};
use singlefs_background_gpu::region_record_kernels::cpu_reference_classification;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::admission::LINEAGE_CONTAINER_BLOCKS_OF_ONE_COW;
use singlefs_core::allocator::{
    unit_area_slots_of_device, AllocationRecord, DeviceFreeMap, PoolAllocator, UnitAreaStart,
};
use singlefs_core::background_compute::{
    admit_gpu_settings_against_the_inventory, AdmittedGpuSettings, BackgroundComputeBackend,
    BackgroundComputeOfOneBatch, BackgroundComputePathTaken, BackgroundComputeSettings,
    BackgroundGpuCompute, BackgroundGpuComputeFailed, BackgroundGpuComputeHandle,
    ClusterSegmentEntryKey, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuEntryDecision,
    GpuInventory, GpuMemoryInBytes, GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch,
    ReasonTheBatchRanOnTheCpu, RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome,
    UnitChecksumToVerifyOnTheGpu, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::{
    choose_region, cluster_segment_entry_keys_of_the_unit_area,
    live_slots_per_cluster_segment_by_the_records, publish_budget_of_a_compaction_run,
    region_scans_of_the_segments_without_an_unreleased_record, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionBatchSize, CompactionBatchUnits, CompactionPacing,
    CompactionRunForcing, CompactionRunStop, CompactionSwitches, CompactionTimerFallback,
    CompactionWriteBandwidthLimit, ForegroundFsync, RegionChoice, RegionChoiceCriterion,
    SegmentsRefusedByTheRegionScan, UnitAreaOfOneDevice,
};
use singlefs_core::inode_tree::{
    leaf_containers_one_root_node_holds, write_records_into_leaf_containers,
    InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_background_compute_settings,
    raise_rollback_floor_to_the_admission_ceiling,
};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::mounted_session::{
    central_mapping_as_the_compaction_reads_it, BackgroundCompactionCall, CompactionRunOutcome,
    MountedSession, UserChange,
};
use singlefs_core::records::{
    InodeRecord, InodeRecordTruncationField, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
};
use singlefs_core::recovery::PoolReader;
use singlefs_core::region_scan::{
    scan_region, scan_region_over_the_identity_index, AllocationInRegion, CentralMappingEntry,
    CentralMappingIdentityIndex,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    SystemImmutableSizes, JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PartialWriteBytes, PoolVersion,
    PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;

/// 产品 GPU 包测试共用模块里给每张参与的卡的显存上限（登记第五节常量表最后一行回比的对象）。
#[path = "../../tests/common_discrete_gpus_of_this_host/mod.rs"]
mod common_discrete_gpus_of_this_host;

// ---- 本地常量（登记第五节「装置读 `crates/` 的这些常量」那张表；值抄自那张表，开跑回比） ----

/// 槽字节（登记第五节常量表第 1 行，`crates/singlefs-format/src/lib.rs` 的 `SLOT_BYTES`）。
const LOCAL_SLOT_BYTES: u64 = 16384;
/// 码 1 数据单元字节（第 2 行，`DATA_UNIT_BYTES`）。
const LOCAL_DATA_UNIT_BYTES: u64 = 32768;
/// 聚簇段槽数（第 3 行，`CLUSTER_SEGMENT_SLOTS`）。
const LOCAL_CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// 每区根槽数 S（第 4 行，`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`）。
const LOCAL_ROOT_RING_SLOTS_PER_REGION: u64 = 8;
/// 默认环长字节（第 5 行，`JOURNAL_RING_DEFAULT_BYTES`）。
const LOCAL_JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// 环起点槽（第 5 行，`JOURNAL_RING_START_SLOT`）。
const LOCAL_JOURNAL_RING_START_SLOT: u64 = 1024;
/// 默认环长下单元区起点槽（第 6 行，`UnitAreaStart::of_the_default_journal_ring`）。
const LOCAL_UNIT_AREA_START_SLOT: u64 = 50176;
/// 一次用户改动给的整理发布预算 `3S − 4`（第 7 行；第七节乙类锚点 B_gc）。
const LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE: u64 = 20;
/// 等 GPU 的上限默认（第 9 行，`BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT`）。
const LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT: Duration = Duration::from_secs(10);
/// 每张参与的卡的显存上限 1 GiB（第 10 行；问题单「装置要求」第一条，用户 2026-11-01 定）。
const LOCAL_MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU: u64 = 1 << 30;
/// 每块盘字节 = 784 MiB + 单元区（登记第一节「盘容量的两种读法与各档取值」）；784 MiB 是默认环长下单元区的起点。
const LOCAL_BYTES_BEFORE_THE_UNIT_AREA: u64 = 784 << 20;
/// 内存盘的扇区，也是物理块大小（同产品 GPU 包测试共用的内存盘）。
const SECTOR_BYTES: u64 = 512;
/// 内存盘按块懒分配，一块这么多字节（登记第五节「负载」第 1 条）。
const MEMORY_DEVICE_CHUNK_BYTES: u64 = 64 * 1024;
/// 登记第十二节修订第 4 条现算的每档单元区槽数（每块盘 MiB → 槽数），本地式子与核心层 `unit_area_slots_of_device` 都要等于它。
const REGISTERED_UNIT_AREA_SLOTS_BY_DEVICE_MEBIBYTES: [(u64, u64); 3] =
    [(3072, 146_432), (4880, 262_144), (8976, 524_288)];
/// 发布预算回比的取样点：已经过去的发布数 → `max(1, 3S − 4 − p)`（登记第五节常量表第 7 行）。
const REGISTERED_PUBLISH_BUDGETS: [(u64, u64); 4] = [(0, 20), (19, 1), (20, 1), (1000, 1)];

/// 本实验自己的 16 字节 fsid（登记第五节「负载」第 1 条）。
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-e202gc1";
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 新池新建文件那一份内容的字节数（同产品 GPU 包测试共用模块）。
const FIRST_FILE_BYTES: usize = 3000;
/// 前台那串用户改动的种子（登记第五节「前台那串用户改动」）。
const FOREGROUND_SEED: u64 = 0xE202;
/// 每档每臂的重复次数（登记第一节「读法写死」的「重复次数」）。
const REPEATS_PER_ARM: u32 = 5;
/// 交替跑一次的用户改动次数与不到 64 批时的阶梯（登记第六节「前提 N」）。
const LOCKSTEP_USER_CHANGES_LADDER: [u64; 3] = [24, 48, 96];
/// 每档每臂计数要的最少批数（登记第六节「前提 N」）。
const MINIMUM_BATCHES_PER_RUN: u64 = 64;
/// 主取样点与几何敏感性那一点的批大小（登记第五节「量挂载」、第八节）。
const BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT: usize = 4;
const BATCH_UNITS_AT_THE_SENSITIVITY_POINT: usize = 16;
/// 一条意图搬的数据单元数：碎片段里活的 16 个（登记第四节「前提事件」那一行）。
const UNITS_MOVED_BY_ONE_INTENT: usize = 16;
/// 交替跑里调用方的假钟每次调用前进的量；定时兜底的间隔（登记第五节「量挂载」「两种跑法」）。
const LOCKSTEP_FAKE_CLOCK_STEP_PER_CALL: Duration = Duration::from_millis(1);
const TIMER_FALLBACK_MILLISECONDS: u64 = 1;
/// 阳性对照：Δ 的中位要落在 E 的中位的这个倍数区间里（登记第五节「阳性对照」判法）。
const POSITIVE_CONTROL_RATIO_LOWER: f64 = 0.8;
const POSITIVE_CONTROL_RATIO_UPPER: f64 = 1.25;
/// 量 T 的「平」：顶档与最小档中位之差的绝对值不超过两者绝对值大者的这一份（登记第六节量 T）。
const FLAT_TREND_WIDTH: f64 = 0.10;
/// CPU 臂（与 CPU 多判臂）每次跑 GC 线程 CPU ÷ GC 挂钟至少这么多（登记第七节乙类）。
const CPU_ARM_THREAD_CPU_SHARE_OF_THE_WALL_AT_LEAST: f64 = 0.9;
/// 这次跑期间本进程以外平均占的核数超过它，那一次重复作废（登记第十一节作废第 3 条）。
const HOST_LOAD_CORES_AT_MOST: f64 = 16.0;
/// 一档里作废的重复多于这么多次，那一档交主 agent（作废第 3 条）。
const VOIDED_REPEATS_PER_TIER_AT_MOST: u32 = 2;
/// 一档里 GPU 臂出错放弃一批累计到这么多次停下（作废第 4 条）。
const GPU_ERRORS_PER_TIER_TO_STOP: u32 = 3;
/// 几何核：造数据挂载写的段里至少这一份恰好 32 槽活（登记第五节「负载」第 4 条）。
const SHARE_OF_WRITTEN_SEGMENTS_WITH_HALF_LIVE_AT_LEAST: f64 = 0.9;
const LIVE_SLOTS_OF_A_FRAGMENTED_SEGMENT: u64 = 32;
/// 造碎片之后抬 F 至多几轮（迭代上界；同 E185（GC一批搬迁的挂钟拆分） 装置的做法）。
const RELEASE_ROUNDS_AT_MOST: u64 = 16;
/// 开跑自检：本线程的 CPU 钟 1 ms 里至少前进这么多次（登记第十一节、第四节第 3a 步第 2 项）。
const CPU_CLOCK_ADVANCES_PER_MILLISECOND_AT_LEAST: u64 = 100;
/// 产品 GPU 包挂载开卡另起的那条线程的名字（`waiting_for_the_gpu.rs` 的 `singlefs-opening-a-gpu`，登记第十条第 6 行）。
const OPENING_GPU_THREAD_NAME: &str = "singlefs-opening-a-gpu";
/// Linux 的线程名（`TASK_COMM_LEN` 16，含收尾的 0）最多这么多字节。
const LINUX_THREAD_NAME_BYTES_AT_MOST: usize = 15;
/// 结果行的前缀。
const RESULT_LINE_PREFIX: &str = "E202";

/// 盘容量的一档：单元区多大（登记第一节，定占用读法，活数据占单元区 50%；三档的取值照登记第十二节修订第 4 条，都是 mkfs 认的）。
/// 封闭集合，按单元区从小到大。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum UnitAreaTier {
    TwoThousandTwoHundredEightyEightMebibytes,
    FourGibibytes,
    EightGibibytes,
}

impl UnitAreaTier {
    const EVERY_TIER: [UnitAreaTier; 3] = [
        UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
        UnitAreaTier::FourGibibytes,
        UnitAreaTier::EightGibibytes,
    ];

    /// 命令行与结果行里的写法（登记第十四节起法里的 `--unit-area`）。
    const fn name(self) -> &'static str {
        match self {
            UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes => "2288MiB",
            UnitAreaTier::FourGibibytes => "4GiB",
            UnitAreaTier::EightGibibytes => "8GiB",
        }
    }

    fn of_the_name(name: &str) -> Option<UnitAreaTier> {
        UnitAreaTier::EVERY_TIER
            .into_iter()
            .find(|tier| tier.name() == name)
    }

    const fn unit_area_bytes(self) -> u64 {
        match self {
            UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes => 2288 << 20,
            UnitAreaTier::FourGibibytes => 4 << 30,
            UnitAreaTier::EightGibibytes => 8 << 30,
        }
    }

    const fn device_bytes(self) -> u64 {
        LOCAL_BYTES_BEFORE_THE_UNIT_AREA + self.unit_area_bytes()
    }

    /// 活的码 1 数据单元 P：单元区能装的数据单元的一半（定占用读法 50%）。
    const fn live_data_units(self) -> u64 {
        self.unit_area_bytes() / LOCAL_DATA_UNIT_BYTES / 2
    }

    /// 每块盘单元区的整段数（登记第七节乙类第 1 行）。
    const fn unit_area_segments(self) -> u64 {
        local_unit_area_slots_of_device(self.device_bytes()) / LOCAL_CLUSTER_SEGMENT_SLOTS
    }

    /// 造碎片之后的碎片段数 P ÷ 32（登记第七节乙类第 3 行）：一段 64 槽装 32 个数据单元，造数据挂载写的 P 个单元里一半留下。
    const fn fragment_segments(self) -> u64 {
        self.live_data_units() / data_units_of_a_cluster_segment()
    }
}

/// 一个聚簇段装几个码 1 数据单元（64 槽 × 16 KiB ÷ 32 KiB = 32）。
const fn data_units_of_a_cluster_segment() -> u64 {
    LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_SLOT_BYTES / LOCAL_DATA_UNIT_BYTES
}

const fn local_unit_area_slots_of_device(device_bytes: u64) -> u64 {
    device_bytes / LOCAL_SLOT_BYTES - LOCAL_UNIT_AREA_START_SLOT
}

/// mkfs 认的每块盘至少是 journal 环的这么多倍：环超过设备容量的四分之一就拒（`crates/singlefs-core/src/make_filesystem.rs` 的
/// `JournalRingTooLargeForDevice`，D23（journal 的角色与格式） 已定项 19）。同包测试 `the_make_filesystem_refuses_a_device_one_sector_short_of_four_rings`
/// 拿真 mkfs 回比这个数。
const LOCAL_DEVICE_BYTES_PER_JOURNAL_RING_BYTES_AT_LEAST: u64 = 4;

/// 默认环长下 mkfs 认的最小一块盘（3072 MiB）。
const fn smallest_device_bytes_the_make_filesystem_accepts() -> u64 {
    LOCAL_DEVICE_BYTES_PER_JOURNAL_RING_BYTES_AT_LEAST * LOCAL_JOURNAL_RING_DEFAULT_BYTES
}

/// 这一档登记的每块盘字节 mkfs 认不认（登记第一节「每块盘字节 = 784 MiB + 单元区」在默认环长下，单元区不到 2288 MiB 的档 mkfs 都拒）。
/// 不认的那一档不造池、照第十一节作废第 6 条交主 agent（登记的算式与 mkfs 对不上）。
const fn the_make_filesystem_accepts_the_device_bytes(device_bytes: u64) -> bool {
    device_bytes >= smallest_device_bytes_the_make_filesystem_accepts()
}

/// 本地式子算的整理发布预算 `max(1, 3S − 4 − p)`。
fn local_publish_budget(publishes_already_passed: u64) -> u64 {
    (3 * LOCAL_ROOT_RING_SLOTS_PER_REGION)
        .saturating_sub(4)
        .saturating_sub(publishes_already_passed)
        .max(1)
}

/// 一条意图要发几次：签发一次，加搬完它那 16 个单元的批数（登记第四节「前提事件」：批大小 4 时 4 批加签发一次 = 5 次）。
fn publishes_of_one_intent(batch_units: usize) -> usize {
    1 + UNITS_MOVED_BY_ONE_INTENT.div_ceil(batch_units)
}

/// 本地常量与核心层（或产品 GPU 包测试共用模块）公开的同一个量对不上的一项。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ConstantMismatch {
    name: String,
    local: u64,
    of_the_crates: u64,
}

/// 第五节常量表逐项回比（停机条款第 1 条）。`singlefs-format` 不是本包的直接依赖，格式常量经核心层公开的等价量取：
/// 槽字节取槽 1 的设备偏移，数据单元字节取 `LINEAGE_CONTAINER_BLOCKS_OF_ONE_COW`（= 数据单元字节 ÷ 槽字节）乘槽字节，
/// 聚簇段槽数取核心层每段活槽数那一步给 640 槽单元区分出的段数，环长取单元区起点与环起点两个偏移之差。
fn local_constants_disagreeing_with_the_core() -> Vec<ConstantMismatch> {
    let mut mismatches = Vec::new();
    let mut compare = |name: String, local: u64, of_the_crates: u64| {
        if local != of_the_crates {
            mismatches.push(ConstantMismatch {
                name,
                local,
                of_the_crates,
            });
        }
    };
    let slot_bytes_of_the_core = SlotNumber(1).to_device_offset().0;
    compare(
        "slot_bytes".to_string(),
        LOCAL_SLOT_BYTES,
        slot_bytes_of_the_core,
    );
    compare(
        "data_unit_bytes".to_string(),
        LOCAL_DATA_UNIT_BYTES,
        LINEAGE_CONTAINER_BLOCKS_OF_ONE_COW * slot_bytes_of_the_core,
    );
    let probe_unit_area_slots = 10 * LOCAL_CLUSTER_SEGMENT_SLOTS;
    let probe_segments = live_slots_per_cluster_segment_by_the_records(
        &[],
        UnitAreaOfOneDevice {
            device: DeviceIdentity(0),
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: probe_unit_area_slots,
        },
    )
    .len();
    compare(
        "cluster_segment_slots".to_string(),
        LOCAL_CLUSTER_SEGMENT_SLOTS,
        probe_unit_area_slots / u64::try_from(probe_segments.max(1)).expect("段数装得进 u64"),
    );
    compare(
        "root_ring_slots_per_region".to_string(),
        LOCAL_ROOT_RING_SLOTS_PER_REGION,
        RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
    );
    compare(
        "journal_ring_start_slot".to_string(),
        LOCAL_JOURNAL_RING_START_SLOT,
        JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION.0,
    );
    let unit_area_start_of_the_core = UnitAreaStart::of_the_default_journal_ring().slot();
    compare(
        "journal_ring_default_bytes".to_string(),
        LOCAL_JOURNAL_RING_DEFAULT_BYTES,
        unit_area_start_of_the_core.to_device_offset().0
            - JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION
                .to_device_offset()
                .0,
    );
    compare(
        "unit_area_start_slot".to_string(),
        LOCAL_UNIT_AREA_START_SLOT,
        unit_area_start_of_the_core.0,
    );
    for (publishes_already_passed, registered_budget) in REGISTERED_PUBLISH_BUDGETS {
        let name = format!("publish_budget_after_{publishes_already_passed}_publishes");
        compare(
            name.clone(),
            local_publish_budget(publishes_already_passed),
            publish_budget_of_a_compaction_run(
                RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
                publishes_already_passed,
            ),
        );
        compare(
            format!("{name}_as_registered"),
            registered_budget,
            local_publish_budget(publishes_already_passed),
        );
    }
    compare(
        "publish_budget_given_by_a_user_change".to_string(),
        LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE,
        local_publish_budget(0),
    );
    for (device_mebibytes, registered_slots) in REGISTERED_UNIT_AREA_SLOTS_BY_DEVICE_MEBIBYTES {
        let device_bytes = device_mebibytes << 20;
        let name = format!("unit_area_slots_of_a_{device_mebibytes}_mebibyte_device");
        compare(
            name.clone(),
            local_unit_area_slots_of_device(device_bytes),
            unit_area_slots_of_device(device_bytes),
        );
        compare(
            format!("{name}_as_registered"),
            registered_slots,
            local_unit_area_slots_of_device(device_bytes),
        );
    }
    compare(
        "gpu_wait_limit_by_default_milliseconds".to_string(),
        u64::try_from(LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT.as_millis()).expect("10 秒"),
        u64::try_from(BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT.as_millis()).expect("10 秒"),
    );
    compare(
        "memory_limit_of_each_participating_gpu".to_string(),
        LOCAL_MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU,
        common_discrete_gpus_of_this_host::MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU.0,
    );
    mismatches
}

// ---- SHA-256（FIPS 180-4）：录制流与读到的内容的摘要。本包不依赖哈希库、派发不许改 Cargo.toml，在这里写一份，单测钉标准的已知答案 ----

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];
const SHA256_INITIAL_STATE: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];
const SHA256_BLOCK_BYTES: usize = 64;

/// 一份进行中的 SHA-256：状态、没凑满一块的尾巴、喂过的总字节数。
#[derive(Clone)]
struct Sha256Digest {
    state: [u32; 8],
    pending: Vec<u8>,
    bytes_fed: u64,
}

impl Sha256Digest {
    fn new() -> Sha256Digest {
        Sha256Digest {
            state: SHA256_INITIAL_STATE,
            pending: Vec::with_capacity(SHA256_BLOCK_BYTES),
            bytes_fed: 0,
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
        self.bytes_fed += u64::try_from(bytes.len()).expect("一段字节数装得进 u64");
        let mut rest = bytes;
        if !self.pending.is_empty() {
            let wanted = SHA256_BLOCK_BYTES - self.pending.len();
            let taken = wanted.min(rest.len());
            self.pending.extend_from_slice(&rest[..taken]);
            rest = &rest[taken..];
            if self.pending.len() == SHA256_BLOCK_BYTES {
                let block: [u8; SHA256_BLOCK_BYTES] =
                    self.pending.as_slice().try_into().expect("凑满了一块");
                sha256_compress(&mut self.state, &block);
                self.pending.clear();
            }
        }
        let (whole_blocks, bytes_short_of_a_block) = rest.as_chunks::<SHA256_BLOCK_BYTES>();
        for block in whole_blocks {
            sha256_compress(&mut self.state, block);
        }
        self.pending.extend_from_slice(bytes_short_of_a_block);
    }

    fn feed_u64(&mut self, value: u64) {
        self.feed(&value.to_le_bytes());
    }

    /// 收尾：补位、长度，交回 64 个十六进制字符。
    fn finish_as_hex(mut self) -> String {
        let bit_length = self.bytes_fed.wrapping_mul(8);
        let mut padding = vec![0x80u8];
        let pending_after_the_marker = (self.pending.len() + 1) % SHA256_BLOCK_BYTES;
        let zero_bytes = (SHA256_BLOCK_BYTES + 56 - pending_after_the_marker) % SHA256_BLOCK_BYTES;
        padding.extend(std::iter::repeat_n(0u8, zero_bytes));
        padding.extend_from_slice(&bit_length.to_be_bytes());
        let bytes_fed_before_the_padding = self.bytes_fed;
        self.feed(&padding);
        self.bytes_fed = bytes_fed_before_the_padding;
        assert!(self.pending.is_empty(), "补位之后正好凑满整块");
        self.state
            .iter()
            .map(|word| format!("{word:08x}"))
            .collect()
    }
}

fn sha256_compress(state: &mut [u32; 8], block: &[u8; SHA256_BLOCK_BYTES]) {
    let mut schedule = [0u32; 64];
    let (words_of_the_block, bytes_short_of_a_word) = block.as_chunks::<4>();
    assert!(
        bytes_short_of_a_word.is_empty(),
        "一块 64 字节正好是 16 个四字节的字"
    );
    for (word_index, word) in words_of_the_block.iter().enumerate() {
        schedule[word_index] = u32::from_be_bytes(*word);
    }
    for round in 16..64 {
        let word_fifteen_rounds_earlier = schedule[round - 15];
        let word_two_rounds_earlier = schedule[round - 2];
        let small_sigma_of_the_older_word = word_fifteen_rounds_earlier.rotate_right(7)
            ^ word_fifteen_rounds_earlier.rotate_right(18)
            ^ (word_fifteen_rounds_earlier >> 3);
        let small_sigma_of_the_newer_word = word_two_rounds_earlier.rotate_right(17)
            ^ word_two_rounds_earlier.rotate_right(19)
            ^ (word_two_rounds_earlier >> 10);
        schedule[round] = schedule[round - 16]
            .wrapping_add(small_sigma_of_the_older_word)
            .wrapping_add(schedule[round - 7])
            .wrapping_add(small_sigma_of_the_newer_word);
    }
    // 八个工作字是两条移位链：每一轮只算出链头的新值，其余各字是链头一轮、两轮、三轮之前的值。
    let mut working = *state;
    for round in 0..64 {
        let [head_word, head_word_one_round_ago, head_word_two_rounds_ago, head_word_three_rounds_ago, middle_word, middle_word_one_round_ago, middle_word_two_rounds_ago, middle_word_three_rounds_ago] =
            working;
        let big_sigma_of_the_middle_word = middle_word.rotate_right(6)
            ^ middle_word.rotate_right(11)
            ^ middle_word.rotate_right(25);
        let choice =
            (middle_word & middle_word_one_round_ago) ^ (!middle_word & middle_word_two_rounds_ago);
        let mixed_into_both_chains = middle_word_three_rounds_ago
            .wrapping_add(big_sigma_of_the_middle_word)
            .wrapping_add(choice)
            .wrapping_add(SHA256_ROUND_CONSTANTS[round])
            .wrapping_add(schedule[round]);
        let big_sigma_of_the_head_word =
            head_word.rotate_right(2) ^ head_word.rotate_right(13) ^ head_word.rotate_right(22);
        let majority = (head_word & head_word_one_round_ago)
            ^ (head_word & head_word_two_rounds_ago)
            ^ (head_word_one_round_ago & head_word_two_rounds_ago);
        let mixed_into_the_head_chain = big_sigma_of_the_head_word.wrapping_add(majority);
        working = [
            mixed_into_both_chains.wrapping_add(mixed_into_the_head_chain),
            head_word,
            head_word_one_round_ago,
            head_word_two_rounds_ago,
            head_word_three_rounds_ago.wrapping_add(mixed_into_both_chains),
            middle_word,
            middle_word_one_round_ago,
            middle_word_two_rounds_ago,
        ];
    }
    for (state_word, working_word) in state.iter_mut().zip(working) {
        *state_word = state_word.wrapping_add(working_word);
    }
}

// ---- 钟（登记第五节「计时与计数怎么做」）：读不到不等于读到 0 ----
// 装置住 `src/bin/`，它自己是一个 crate 根，门禁 code-source-discipline 的 unsafe-allowances 格不许那里写 unsafe：
// 这条线程的 CPU 钟用产品 GPU 包的 `host_cpu_time_of_this_thread`（`CLOCK_THREAD_CPUTIME_ID`）；别的线程用掉的 CPU 读
// `/proc/self/task/<线程号>/schedstat` 的第一个字段（纳秒，内核记的 `sum_exec_runtime`）；本进程的滴答读 `/proc/self/stat`。
// 每次 GC 调用的挂钟（量 W 与包含自检）与并发跑的时刻读 `StartOnTheMonotonicRawClock`（登记第十二节修订第 10 条）；
// 造池、点名、丙段代价、机器负载与开跑自检那几处只报不判包含关系，照旧读 `Instant`。

/// 挂钟的起点：读产品 GPU 包的 `monotonic_raw_time_now`（`CLOCK_MONOTONIC_RAW`，不受 NTP 调频）。`Instant` 读的 `CLOCK_MONOTONIC`
/// 跟着 NTP 调频走、线程 CPU 钟不跟，长调用上挂钟比 GC 线程 CPU 少走几 ppm，包含自检「挂钟 ≥ GC 线程 CPU」就判不过（登记第十二节修订第 10 条）。
#[derive(Clone, Copy, Debug)]
struct StartOnTheMonotonicRawClock {
    monotonic_raw_time_at_the_start: Duration,
}

impl StartOnTheMonotonicRawClock {
    fn now() -> StartOnTheMonotonicRawClock {
        StartOnTheMonotonicRawClock {
            monotonic_raw_time_at_the_start: monotonic_raw_time_now(),
        }
    }

    fn elapsed(self) -> Duration {
        monotonic_raw_time_now()
            .checked_sub(self.monotonic_raw_time_at_the_start)
            .expect("CLOCK_MONOTONIC_RAW 单调不减：此刻不早于起点")
    }
}

/// 读一样钟或 `/proc` 里的一样计数读不到（登记第十一节作废第 1 条）：哪一样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClockReadFailed {
    reading: &'static str,
}

impl ClockReadFailed {
    fn description(self) -> String {
        format!("{}读不到", self.reading)
    }
}

/// 这条线程在 `work` 里用掉的 CPU；钟往回走交 `None`。
fn thread_cpu_spent_in<Outcome>(work: impl FnOnce() -> Outcome) -> (Outcome, Option<Duration>) {
    let before = host_cpu_time_of_this_thread();
    let outcome = work();
    let after = host_cpu_time_of_this_thread();
    (outcome, after.checked_sub(before))
}

/// 这条线程的线程号（`/proc/thread-self` 指向 `<进程号>/task/<线程号>`）：交给 GC 线程读这条线程用掉的 CPU。
fn task_identifier_of_this_thread() -> Result<u64, ClockReadFailed> {
    let failed = ClockReadFailed {
        reading: "/proc/thread-self",
    };
    let target = std::fs::read_link("/proc/thread-self").map_err(|_error| failed)?;
    target
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.parse::<u64>().ok())
        .ok_or(failed)
}

/// `schedstat` 一行的第一个字段：这条线程到此刻在 CPU 上的纳秒数。
fn cpu_of_the_schedstat_line(line: &str) -> Option<Duration> {
    let nanoseconds = line.split_whitespace().next()?.parse::<u64>().ok()?;
    Some(Duration::from_nanos(nanoseconds))
}

/// 本进程此刻每条线程用掉的 CPU（线程号 → CPU）。列出之后、读之前退出的线程不在里面。
fn cpu_of_every_task_of_this_process() -> Result<BTreeMap<u64, Duration>, ClockReadFailed> {
    let failed = ClockReadFailed {
        reading: "/proc/self/task",
    };
    let mut cpu_by_task: BTreeMap<u64, Duration> = BTreeMap::new();
    for task in std::fs::read_dir("/proc/self/task").map_err(|_error| failed)? {
        let task = task.map_err(|_error| failed)?;
        let Some(task_identifier) = task
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u64>().ok())
        else {
            return Err(failed);
        };
        match std::fs::read_to_string(task.path().join("schedstat")) {
            Ok(line) => {
                cpu_by_task.insert(
                    task_identifier,
                    cpu_of_the_schedstat_line(&line).ok_or(failed)?,
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_error) => return Err(failed),
        }
    }
    Ok(cpu_by_task)
}

/// 开跑自检（登记第十一节、第四节第 3a 步第 2 项）：本线程的 CPU 钟 1 ms 挂钟里读出的不同值有几个。
fn advances_of_the_thread_cpu_clock_in_a_millisecond() -> u64 {
    let started = Instant::now();
    let mut last_reading = host_cpu_time_of_this_thread();
    let mut advances = 0_u64;
    while started.elapsed() < Duration::from_millis(1) {
        let reading = host_cpu_time_of_this_thread();
        if reading != last_reading {
            advances += 1;
            last_reading = reading;
        }
    }
    advances
}

/// 内核经 `/proc/stat` 与 `/proc/<进程>/stat` 报给用户空间的滴答，每秒这么多（Linux 的 `USER_HZ`，x86-64 与 aarch64 上固定）。
const KERNEL_TICKS_PER_SECOND_REPORTED_TO_USER_SPACE: u64 = 100;

/// 全机忙时与本进程 CPU 的一次读数（登记第五节「计时与计数」最后一条，量 H），都按滴答。
#[derive(Clone, Copy, Debug)]
struct HostLoadReading {
    busy_ticks_of_every_core: u64,
    ticks_of_this_process: u64,
    at: Instant,
}

/// `/proc/stat` 头一行（`cpu  user nice system idle iowait irq softirq steal …`）里的忙时：user + nice + system + irq + softirq + steal。
fn busy_ticks_of_the_kernel_cpu_counter_line(line: &str) -> Option<u64> {
    let mut fields = line.split_whitespace();
    if fields.next() != Some("cpu") {
        return None;
    }
    let counters: Vec<u64> = fields
        .take(8)
        .map(|field| field.parse::<u64>().ok())
        .collect::<Option<Vec<u64>>>()?;
    let [user, nice, system, _idle, _waiting_for_input_output, interrupts, soft_interrupts, stolen] =
        counters.as_slice().try_into().ok()?;
    Some(user + nice + system + interrupts + soft_interrupts + stolen)
}

/// `/proc/self/stat` 那一行里本进程的 utime + stime（第 14、15 个字段；第 2 个字段是括号里的线程名，可以带空格，从最后一个右括号之后数）。
fn ticks_of_the_process_status_line(line: &str) -> Option<u64> {
    let (_before, after_the_name) = line.rsplit_once(')')?;
    let fields: Vec<&str> = after_the_name.split_whitespace().collect();
    let user_ticks = fields.get(11)?.parse::<u64>().ok()?;
    let system_ticks = fields.get(12)?.parse::<u64>().ok()?;
    Some(user_ticks + system_ticks)
}

fn host_load_reading() -> Result<HostLoadReading, String> {
    let kernel_cpu_counters =
        std::fs::read_to_string("/proc/stat").map_err(|error| format!("读 /proc/stat：{error}"))?;
    let busy_ticks_of_every_core = kernel_cpu_counters
        .lines()
        .next()
        .and_then(busy_ticks_of_the_kernel_cpu_counter_line)
        .ok_or_else(|| "/proc/stat 头一行认不出".to_string())?;
    let process_status = std::fs::read_to_string("/proc/self/stat")
        .map_err(|error| format!("读 /proc/self/stat：{error}"))?;
    let ticks_of_this_process = ticks_of_the_process_status_line(&process_status)
        .ok_or_else(|| "/proc/self/stat 认不出".to_string())?;
    Ok(HostLoadReading {
        busy_ticks_of_every_core,
        ticks_of_this_process,
        at: Instant::now(),
    })
}

/// 两次读数之间本进程以外平均占了几个核（量 H）：全机忙时减本进程，除以挂钟。
fn cores_used_outside_this_process(start: HostLoadReading, end: HostLoadReading) -> f64 {
    let busy_ticks = end
        .busy_ticks_of_every_core
        .saturating_sub(start.busy_ticks_of_every_core);
    let process_ticks = end
        .ticks_of_this_process
        .saturating_sub(start.ticks_of_this_process);
    let outside_seconds = count_as_float(busy_ticks.saturating_sub(process_ticks))
        / count_as_float(KERNEL_TICKS_PER_SECOND_REPORTED_TO_USER_SPACE);
    outside_seconds
        / end
            .at
            .duration_since(start.at)
            .as_secs_f64()
            .max(f64::MIN_POSITIVE)
}

/// 本进程此刻有没有挂载开卡另起的那条线程（登记第十条第 6 行）。线程名在 `/proc/self/task/*/comm` 里截到 15 个字节。
fn the_gpu_opening_thread_is_still_running() -> Result<bool, String> {
    let tasks = std::fs::read_dir("/proc/self/task")
        .map_err(|error| format!("读 /proc/self/task：{error}"))?;
    for task in tasks {
        let task = task.map_err(|error| format!("读 /proc/self/task 的一项：{error}"))?;
        // 线程在读的这一刻退出了：读不到它的名字就是它已经不在。
        if let Ok(comm_line) = std::fs::read_to_string(task.path().join("comm")) {
            if is_the_comm_line_of_the_gpu_opening_thread(&comm_line) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// 内核记的线程名截到 15 个字节（`/proc/self/task/*/comm` 一行，末尾带换行）：比的是开卡线程名的前 15 个字节（作废第 7 条）。
fn is_the_comm_line_of_the_gpu_opening_thread(comm_line: &str) -> bool {
    comm_line.trim_end() == &OPENING_GPU_THREAD_NAME[..LINUX_THREAD_NAME_BYTES_AT_MOST]
}

// ---- 内存盘与录制摘要（登记第五节「负载」第 1 条、「装置约束」第三行）----

/// 改盘的一个动作（录制摘要按先后喂这一种、盘、偏移、长度与字节）。
#[derive(Clone, Copy, Debug)]
enum RecordedDeviceOperation<'bytes> {
    Write {
        device: DeviceIdentity,
        offset_in_bytes: u64,
        bytes: &'bytes [u8],
        durability: WriteDurability,
    },
    WriteZeroes {
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    },
    Discard {
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    },
    Barrier {
        device: DeviceIdentity,
    },
}

/// 把一个改盘动作喂进录制摘要：种类、盘、偏移、长度，写的另喂持久方式与字节（第九节变异「录制摘要只喂长度」改的是这里）。
fn feed_the_operation_into_the_digest(
    digest: &mut Sha256Digest,
    operation: RecordedDeviceOperation<'_>,
) {
    match operation {
        RecordedDeviceOperation::Write {
            device,
            offset_in_bytes,
            bytes,
            durability,
        } => {
            digest.feed(&[1]);
            digest.feed(&device.0.to_le_bytes());
            digest.feed_u64(offset_in_bytes);
            digest.feed_u64(u64::try_from(bytes.len()).expect("写长装得进 u64"));
            digest.feed(&[match durability {
                WriteDurability::Plain => 0,
                WriteDurability::ForceUnitAccess => 1,
            }]);
            digest.feed(bytes);
        }
        RecordedDeviceOperation::WriteZeroes {
            device,
            offset_in_bytes,
            length_in_bytes,
        } => {
            digest.feed(&[2]);
            digest.feed(&device.0.to_le_bytes());
            digest.feed_u64(offset_in_bytes);
            digest.feed_u64(length_in_bytes);
        }
        RecordedDeviceOperation::Discard {
            device,
            offset_in_bytes,
            length_in_bytes,
        } => {
            digest.feed(&[3]);
            digest.feed(&device.0.to_le_bytes());
            digest.feed_u64(offset_in_bytes);
            digest.feed_u64(length_in_bytes);
        }
        RecordedDeviceOperation::Barrier { device } => {
            digest.feed(&[4]);
            digest.feed(&device.0.to_le_bytes());
        }
    }
}

/// 一个池两块盘共用的录制摘要，连同喂它花掉的线程 CPU（`name=recording_overhead`）与读 CPU 钟失败的次数。
struct RecordingDigest {
    is_fed: bool,
    digest: Sha256Digest,
    operations: u64,
    cpu_spent_feeding: Duration,
    clock_read_failures: u64,
}

impl RecordingDigest {
    fn empty() -> RecordingDigest {
        RecordingDigest {
            is_fed: true,
            digest: Sha256Digest::new(),
            operations: 0,
            cpu_spent_feeding: Duration::ZERO,
            clock_read_failures: 0,
        }
    }

    /// 造起步镜像时不喂摘要（造池不比录制流，省下给几 GiB 字节算摘要的 CPU）。
    fn not_fed() -> RecordingDigest {
        RecordingDigest {
            is_fed: false,
            digest: Sha256Digest::new(),
            operations: 0,
            cpu_spent_feeding: Duration::ZERO,
            clock_read_failures: 0,
        }
    }

    fn note(&mut self, operation: RecordedDeviceOperation<'_>) {
        if !self.is_fed {
            return;
        }
        let digest = &mut self.digest;
        let ((), spent) =
            thread_cpu_spent_in(|| feed_the_operation_into_the_digest(digest, operation));
        self.operations += 1;
        match spent {
            Some(spent) => self.cpu_spent_feeding += spent,
            None => self.clock_read_failures += 1,
        }
    }
}

type SharedRecordingDigest = Arc<Mutex<RecordingDigest>>;

/// 内存盘的块怎么存（登记第五节「负载」第 1 条的稀疏内存盘）。两种存法读写的字节逐个相同，只差占多少内存。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChunkStorage {
    /// 每块盘各存各的块，拷镜像整份复制：改前的存法，只留给 `construction-timing` 对照与同包测试对拍。
    EachDeviceCopiesItsChunks,
    /// 同一个池里同号、逐字节相同的块共用一份：核心层把每个单元与每条 journal 记录同一份字节写到每块盘的同一偏移
    /// （`singlefs_core::transaction` 的 `CommitStep::WriteUnitToEveryDevice`、`WriteJournalRecordToEveryDevice`），后写的那块盘认出同号块与
    /// 先写的那块相同就改指那一份；全是零的块不存（没写过的读回 0）；拷镜像只拷块的引用，哪块盘写到共用的块才复制那一块。
    SharedWhenTheBytesAreEqual,
}

/// 一个池两块盘共用的「每个块号最近写成的那一份」：后写同号块的那块盘拿它比字节，相同就共用。每个块号只留一项，项数不超过盘的块数。
type LatestChunksOfThePool = Arc<Mutex<HashMap<u64, Weak<Vec<u8>>>>>;

/// 内存里的一块稀疏盘：按 64 KiB 块懒分配，没写过的读回 0；每个改盘动作喂进这个池共用的录制摘要。
struct RecordingMemoryDevice {
    identity: DeviceIdentity,
    size_in_bytes: u64,
    chunks: HashMap<u64, Arc<Vec<u8>>>,
    storage: ChunkStorage,
    latest_chunks: LatestChunksOfThePool,
    recording: SharedRecordingDigest,
}

impl RecordingMemoryDevice {
    fn note(&self, operation: RecordedDeviceOperation<'_>) {
        self.recording
            .lock()
            .expect("录制摘要只在借着盘的那条线程上喂，没在别处 panic 过")
            .note(operation);
    }

    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) -> Result<(), BlockDeviceError> {
        if !offset.0.is_multiple_of(SECTOR_BYTES) || !length_in_bytes.is_multiple_of(SECTOR_BYTES) {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length: length_in_bytes,
                physical_block_size: u32::try_from(SECTOR_BYTES).expect("512"),
            });
        }
        if offset
            .0
            .checked_add(length_in_bytes)
            .is_none_or(|end| end > self.size_in_bytes)
        {
            return Err(BlockDeviceError::OutOfRange {
                offset,
                length: length_in_bytes,
                device_size: self.size_in_bytes,
            });
        }
        Ok(())
    }

    /// `[offset, offset + length)` 按块切开：块号、块内起点、这一截的长度、这一截在请求里的起点。
    fn pieces_of_the_range(
        offset_in_bytes: u64,
        length_in_bytes: u64,
    ) -> Vec<(u64, usize, usize, usize)> {
        let mut pieces = Vec::new();
        let mut position = offset_in_bytes;
        let end = offset_in_bytes + length_in_bytes;
        while position < end {
            let chunk_index = position / MEMORY_DEVICE_CHUNK_BYTES;
            let start_in_the_chunk = position % MEMORY_DEVICE_CHUNK_BYTES;
            let length_in_the_chunk =
                (MEMORY_DEVICE_CHUNK_BYTES - start_in_the_chunk).min(end - position);
            pieces.push((
                chunk_index,
                usize::try_from(start_in_the_chunk).expect("块内偏移"),
                usize::try_from(length_in_the_chunk).expect("块内长度"),
                usize::try_from(position - offset_in_bytes).expect("请求里的偏移"),
            ));
            position += length_in_the_chunk;
        }
        pieces
    }

    fn zero_the_range(&mut self, offset_in_bytes: u64, length_in_bytes: u64) {
        for (chunk_index, start_in_the_chunk, length_in_the_chunk, _start_in_the_request) in
            Self::pieces_of_the_range(offset_in_bytes, length_in_bytes)
        {
            if length_in_the_chunk == usize::try_from(MEMORY_DEVICE_CHUNK_BYTES).expect("64 KiB") {
                self.chunks.remove(&chunk_index);
            } else if let Some(chunk) = self.chunks.get_mut(&chunk_index) {
                Arc::make_mut(chunk)[start_in_the_chunk..start_in_the_chunk + length_in_the_chunk]
                    .fill(0);
                self.settle_the_chunk(chunk_index);
            }
        }
    }

    /// 写过块 `chunk_index` 之后按存法收拾它（[`ChunkStorage::SharedWhenTheBytesAreEqual`]）：全是零就不存；同号块最近写成的那一份
    /// 与它逐字节相同就改指那一份，不同就把它记成这个块号最近写成的那一份。各存各的那一种什么都不做。
    fn settle_the_chunk(&mut self, chunk_index: u64) {
        match self.storage {
            ChunkStorage::EachDeviceCopiesItsChunks => {}
            ChunkStorage::SharedWhenTheBytesAreEqual => {
                let Some(chunk) = self.chunks.get_mut(&chunk_index) else {
                    return;
                };
                if chunk.iter().all(|byte| *byte == 0) {
                    self.chunks.remove(&chunk_index);
                    return;
                }
                let mut latest_chunks = self
                    .latest_chunks
                    .lock()
                    .expect("块表只在借着盘的那条线程上改，没在别处 panic 过");
                let latest_of_the_index = latest_chunks.get(&chunk_index).and_then(Weak::upgrade);
                match latest_of_the_index {
                    Some(latest) if Arc::ptr_eq(&latest, chunk) => {}
                    Some(latest) if latest.as_slice() == chunk.as_slice() => *chunk = latest,
                    Some(_) | None => {
                        latest_chunks.insert(chunk_index, Arc::downgrade(chunk));
                    }
                }
            }
        }
    }

    /// 同一份字节的另一块盘，记进 `recording`、共用 `latest_chunks`（拷起步镜像：登记第五节「量挂载」从起步镜像拷一份）。
    /// 共用的存法只拷块的引用（写到哪一块才复制那一块）；各存各的存法整份复制。
    fn copied_into(
        &self,
        recording: &SharedRecordingDigest,
        latest_chunks: &LatestChunksOfThePool,
    ) -> RecordingMemoryDevice {
        let chunks = match self.storage {
            ChunkStorage::EachDeviceCopiesItsChunks => self
                .chunks
                .iter()
                .map(|(chunk_index, chunk)| (*chunk_index, Arc::new(Vec::clone(chunk))))
                .collect(),
            ChunkStorage::SharedWhenTheBytesAreEqual => self.chunks.clone(),
        };
        RecordingMemoryDevice {
            identity: self.identity,
            size_in_bytes: self.size_in_bytes,
            chunks,
            storage: self.storage,
            latest_chunks: Arc::clone(latest_chunks),
            recording: Arc::clone(recording),
        }
    }
}

/// 一个池的内存盘此刻装着的块字节：各块盘各自装着的（每块盘各数一遍），与真占内存的（共用的一份只数一次）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ChunkBytesOfThePool {
    held_by_the_devices: u64,
    stored: u64,
}

fn chunk_bytes_of_the_pool(devices: &PoolDevices) -> ChunkBytesOfThePool {
    let mut stored_chunks: HashSet<*const Vec<u8>> = HashSet::new();
    let mut held_by_the_devices = 0_u64;
    for (_identity, device) in devices {
        for chunk in device.chunks.values() {
            held_by_the_devices += MEMORY_DEVICE_CHUNK_BYTES;
            stored_chunks.insert(Arc::as_ptr(chunk));
        }
    }
    ChunkBytesOfThePool {
        held_by_the_devices,
        stored: u64::try_from(stored_chunks.len()).expect("块数装得进 u64")
            * MEMORY_DEVICE_CHUNK_BYTES,
    }
}

impl BlockDevice for RecordingMemoryDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length_in_bytes)?;
        for (chunk_index, start_in_the_chunk, length_in_the_chunk, start_in_the_request) in
            Self::pieces_of_the_range(offset.0, length_in_bytes)
        {
            let target =
                &mut buffer[start_in_the_request..start_in_the_request + length_in_the_chunk];
            match self.chunks.get(&chunk_index) {
                Some(chunk) => target.copy_from_slice(
                    &chunk[start_in_the_chunk..start_in_the_chunk + length_in_the_chunk],
                ),
                None => target.fill(0),
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(bytes.len()).expect("写长装得进 u64");
        self.check_request(offset, length_in_bytes)?;
        let chunk_bytes = usize::try_from(MEMORY_DEVICE_CHUNK_BYTES).expect("64 KiB");
        for (chunk_index, start_in_the_chunk, length_in_the_chunk, start_in_the_request) in
            Self::pieces_of_the_range(offset.0, length_in_bytes)
        {
            let chunk = self
                .chunks
                .entry(chunk_index)
                .or_insert_with(|| Arc::new(vec![0u8; chunk_bytes]));
            Arc::make_mut(chunk)[start_in_the_chunk..start_in_the_chunk + length_in_the_chunk]
                .copy_from_slice(
                    &bytes[start_in_the_request..start_in_the_request + length_in_the_chunk],
                );
            self.settle_the_chunk(chunk_index);
        }
        self.note(RecordedDeviceOperation::Write {
            device: self.identity,
            offset_in_bytes: offset.0,
            bytes,
            durability,
        });
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_the_range(offset.0, length);
        self.note(RecordedDeviceOperation::WriteZeroes {
            device: self.identity,
            offset_in_bytes: offset.0,
            length_in_bytes: length,
        });
        Ok(())
    }

    /// 内存盘认 discard，当整段清零应用，交回「交给了底层」（同产品 GPU 包测试共用的内存盘）。
    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_the_range(offset.0, length);
        self.note(RecordedDeviceOperation::Discard {
            device: self.identity,
            offset_in_bytes: offset.0,
            length_in_bytes: length,
        });
        Ok(DiscardOutcome::Applied)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.note(RecordedDeviceOperation::Barrier {
            device: self.identity,
        });
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(u32::try_from(SECTOR_BYTES).expect("512"))
    }

    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }
}

type PoolDevices = Vec<(DeviceIdentity, RecordingMemoryDevice)>;

/// 两块空盘，记进同一份录制摘要，块照共用的存法存（[`ChunkStorage::SharedWhenTheBytesAreEqual`]）。只有同包测试用。
#[cfg(test)]
fn empty_pool_devices(device_bytes: u64, recording: &SharedRecordingDigest) -> PoolDevices {
    empty_pool_devices_storing_the_chunks(
        device_bytes,
        recording,
        ChunkStorage::SharedWhenTheBytesAreEqual,
    )
}

/// 两块空盘，记进同一份录制摘要，块照 `storage` 存；两块盘共用一份块表。
fn empty_pool_devices_storing_the_chunks(
    device_bytes: u64,
    recording: &SharedRecordingDigest,
    storage: ChunkStorage,
) -> PoolDevices {
    let latest_chunks: LatestChunksOfThePool = Arc::new(Mutex::new(HashMap::new()));
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                RecordingMemoryDevice {
                    identity,
                    size_in_bytes: device_bytes,
                    chunks: HashMap::new(),
                    storage,
                    latest_chunks: Arc::clone(&latest_chunks),
                    recording: Arc::clone(recording),
                },
            )
        })
        .collect()
}

/// 同一份字节的另一个池，记进一份新的空录制摘要（「同一历史」的另一个副本），两块盘另共用一份新的块表。
fn copied_pool_devices(devices: &PoolDevices) -> (PoolDevices, SharedRecordingDigest) {
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
    let latest_chunks: LatestChunksOfThePool = Arc::new(Mutex::new(HashMap::new()));
    let copied = devices
        .iter()
        .map(|(identity, device)| (*identity, device.copied_into(&recording, &latest_chunks)))
        .collect();
    (copied, recording)
}

// ---- 活数据单元的内容（登记第五节「负载」「前台那串用户改动」）：每个数据单元的字节由单元下标与第几次写定 ----

/// 造数据挂载写活数据单元是第 0 次，垫片文件的单元是第 1 次（按垫片里的单元序号定内容），前台第 k 次用户改动写的是第 2 + k 次。
const WRITE_GENERATION_OF_THE_DATA_MOUNT: u64 = 0;
const WRITE_GENERATION_OF_THE_SPACER_FILE: u64 = 1;
const WRITE_GENERATION_OF_THE_FIRST_USER_CHANGE: u64 = 2;

fn splitmix64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^ (mixed >> 31)
}

/// 活数据单元第 `unit_index` 个（垫片文件里按它自己的单元序号）第 `write_generation` 次写的内容，填满 `buffer`（一个数据单元的净荷容量那么长）。
fn fill_the_content_of_a_data_unit(unit_index: u64, write_generation: u64, buffer: &mut [u8]) {
    let mut state = FOREGROUND_SEED ^ unit_index.rotate_left(24) ^ write_generation.rotate_left(48);
    for eight_bytes in buffer.chunks_mut(8) {
        let word = splitmix64_next(&mut state).to_le_bytes();
        eight_bytes.copy_from_slice(&word[..eight_bytes.len()]);
    }
}

/// 前台第 `change_number` 次用户改动覆盖写哪一个单元：种子 0xE202 与序号定，在 P ÷ K 个活数据文件里均匀取一个文件，覆盖写它的最后一个单元
/// （K = [`LIVE_DATA_UNITS_PER_FILE`]）。按偏移写从偏移所在的单元重写到文件末尾（`transaction::publish_partial_write` 文档 R4），
/// 取最后一个单元才只重写这一个；取文件里别的单元一次要重写好几个，不是登记第五节「覆盖写那一个单元」。
fn unit_index_of_the_user_change(change_number: u64, live_data_units: u64) -> u64 {
    let mut state = FOREGROUND_SEED.wrapping_add(change_number.wrapping_mul(0x1000_0000_0001));
    let drawn = splitmix64_next(&mut state);
    let file =
        u64::try_from((u128::from(drawn) * u128::from(live_data_files(live_data_units))) >> 64)
            .expect("小于活数据文件数");
    file * LIVE_DATA_UNITS_PER_FILE + (LIVE_DATA_UNITS_PER_FILE - 1)
}

/// 活数据单元按每个单元最后一次写算出的期望内容的摘要（按单元下标次序，即 inode 号次序）。
fn expected_content_digest(write_generation_of_each_unit: &[u64]) -> String {
    let mut digest = Sha256Digest::new();
    let mut buffer = vec![0u8; data_unit_payload_capacity()];
    for (unit_index, write_generation) in (0_u64..).zip(write_generation_of_each_unit) {
        fill_the_content_of_a_data_unit(unit_index, *write_generation, &mut buffer);
        digest.feed(&buffer);
    }
    digest.finish_as_hex()
}

/// 读盘那一侧（读路径）看两块内存盘：读不到（没有这块盘、越界）交 `None`，不给 journal 偏移提示。
struct ReaderOfTheMemoryPool<'devices> {
    devices: &'devices [(DeviceIdentity, RecordingMemoryDevice)],
}

impl PoolReader for ReaderOfTheMemoryPool<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.devices
            .iter()
            .map(|(identity, _device)| *identity)
            .collect()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.devices
            .iter()
            .find(|(identity, _device)| *identity == device)
            .map(|(_identity, memory_device)| memory_device.size_in_bytes)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let (_identity, memory_device) = self
            .devices
            .iter()
            .find(|(identity, _device)| *identity == device)?;
        let mut bytes = vec![0u8; length];
        memory_device.read_at(offset, &mut bytes).ok()?;
        Some(bytes)
    }

    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

/// 按现行那一版读回活数据单元那 P ÷ K 个文件（inode `lowest_live_data_inode` 起，每个 K 个单元），按单元下标次序逐个喂摘要
/// （登记第五节「装置约束」第三行：跑完读每个文件的内容摘要）。哪个文件不是恰好 K 个数据单元、读回来不是 K 个单元的净荷那么长，就交回读不成。
fn read_content_digest(
    devices: &PoolDevices,
    version: &TransactionOutput,
    lowest_live_data_inode: u64,
    live_data_units: u64,
) -> Result<String, String> {
    let reader = ReaderOfTheMemoryPool {
        devices: devices.as_slice(),
    };
    let reader: &dyn PoolReader = &reader;
    let opened_pool = open_pool_for_read(reader, &version.root)
        .map_err(|failure| format!("打开池读：{failure:?}"))?;
    let capacity = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
    let file_bytes = capacity * LIVE_DATA_UNITS_PER_FILE;
    let mut digest = Sha256Digest::new();
    for file in 0..live_data_files(live_data_units) {
        let inode = lowest_live_data_inode + file;
        let opened_file = opened_pool
            .open_file(reader, InodeNumber(inode))
            .map_err(|failure| format!("打开 inode {inode}：{failure:?}"))?;
        if opened_file.data_unit_count() != LIVE_DATA_UNITS_PER_FILE {
            return Err(format!(
                "inode {inode} 有 {} 个数据单元，不是 {LIVE_DATA_UNITS_PER_FILE} 个",
                opened_file.data_unit_count()
            ));
        }
        let read = opened_file
            .read_at(reader, FileOffsetInBytes(0), file_bytes)
            .map_err(|failure| format!("读 inode {inode}：{failure:?}"))?;
        if u64::try_from(read.bytes.len()).expect("读回的字节数装得进 u64") != file_bytes {
            return Err(format!(
                "inode {inode} 读回 {} 字节，不是 {file_bytes} 字节",
                read.bytes.len()
            ));
        }
        digest.feed(&read.bytes);
    }
    Ok(digest.finish_as_hex())
}

// ---- 造起步镜像（登记第五节「负载：池怎么造」）----

/// 默认环长：从第一版的环起点到默认环长下单元区起点之间那么多字节（照核心层的两个起点算，同产品 GPU 包测试共用模块）。
fn default_journal_ring_bytes() -> u64 {
    UnitAreaStart::of_the_default_journal_ring()
        .slot()
        .to_device_offset()
        .0
        - JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION
            .to_device_offset()
            .0
}

/// mkfs 参数：照产品 GPU 包测试共用模块的 `parameters()`（region 设备 0、1、0，默认环长，512 字节扇区），fsid 换成本实验的。
fn make_filesystem_parameters_of_the_pool() -> MakeFilesystemParameters {
    let sector = u32::try_from(SECTOR_BYTES).expect("512");
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: sector,
            minimum_input_output_bytes: sector,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(sector),
            journal_ring_bytes: default_journal_ring_bytes(),
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 有树的那一版（第一个文件发过之后每一版都有树）。
fn version_of_trees(session: &MountedSession) -> &TransactionOutput {
    session
        .current
        .version_of_trees()
        .expect("第一个文件发过之后，会话的现行那一版都有树")
}

/// mkfs → 取号 → 暖机 → 新池新建文件（同产品 GPU 包测试共用模块 `pool_after_a_sequential_write_in_an_earlier_mount` 那一串），块照 `storage` 存。
fn new_pool_with_the_first_file(device_bytes: u64, storage: ChunkStorage) -> PoolDevices {
    let parameters = make_filesystem_parameters_of_the_pool();
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::not_fed()));
    let mut devices = empty_pool_devices_storing_the_chunks(device_bytes, &recording, storage);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
        DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
    ]);
    allocator.mark_format_time_units(
        genesis.instance_table_placement(),
        genesis.tree_table_genesis_placement(),
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let first_file_content: Vec<u8> = (0..FIRST_FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect();
    let mut writer = PoolWriter::new(&parameters, &mut devices);
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    publish_first_file(
        &mut writer,
        &mut allocator,
        warmed_up.roots.last().expect("暖机两代根"),
        FirstFile {
            content: &first_file_content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed_up.last_record_bytes,
    )
    .expect("新池新建文件");
    drop(writer);
    devices
}

/// 现行那一版里 inode 号落在 `[first_inode, first_inode + inode_count)` 的文件每个数据单元的槽（盘 0；两盘同槽）：
/// （inode 号，文件里的单元序号）→ 槽。只在造池的几个节点上读一次（扫整份 `units`），不逐次发布读。
fn data_unit_slots_of_the_inodes(
    version: &TransactionOutput,
    first_inode: u64,
    inode_count: u64,
) -> BTreeMap<(u64, u64), u64> {
    let inodes = first_inode..first_inode + inode_count;
    version
        .units
        .iter()
        .filter_map(|published| {
            if let TransactionUnit::DataOfAFileOtherThanTheFirst { inode, unit } =
                published.identity
            {
                inodes
                    .contains(&inode.0)
                    .then_some(((inode.0, unit.0), published.slot.0))
            } else {
                None
            }
        })
        .collect()
}

/// 盘 0 上被未释放记录罩住的槽。
fn slots_under_an_unreleased_record_on_device_zero(records: &[AllocationRecord]) -> BTreeSet<u64> {
    records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && !record.is_released)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// 盘 0 的单元区（量挂载时选段与活槽段看的那一块）。
fn unit_area_of_device_zero(session: &MountedSession) -> UnitAreaOfOneDevice {
    let device_zero = &session.allocator.devices[0];
    UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: device_zero.unit_area_start(),
        slots: device_zero.unit_area_slots(),
    }
}

/// 段的起点槽（单元区起点在聚簇段边界上）。
const fn segment_start_of(slot: u64, unit_area_start_slot: u64) -> u64 {
    unit_area_start_slot
        + (slot - unit_area_start_slot) / LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_CLUSTER_SEGMENT_SLOTS
}

/// 几何核的读数（登记第五节「负载」第 4 条）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct GeometryReading {
    /// 造数据挂载里垫片文件与活数据单元交错写满的段（碎片段，P ÷ 32 段），与其中活槽数恰好 32 的段数。
    written_segments: u64,
    written_segments_with_half_live: u64,
    /// 量挂载重挂之后 greedy 要选的那一段（照核心层签发那一步重放，[`segment_the_greedy_choice_picks`]）：只报，不进判法。
    greedy: SegmentOfTheChoice,
    /// 强制选段后备头一条意图点名的那一段（[`fragment_segment_to_force`]，登记第十二节修订第 8 条）：几何核后一半判它。
    forced: SegmentOfTheChoice,
    /// 每段活槽数的直方图：活槽数 → 段数。
    live_slot_histogram: BTreeMap<u64, u64>,
}

impl GeometryReading {
    /// 几何核的两半都成立（登记第五节「负载」第 4 条，后一半照第十二节修订第 8 条换成强制选定的那一段）：碎片段造成了，
    /// 强制选段后备点名的那一段只装活数据单元、活槽不多于 32。
    fn is_formed(&self) -> bool {
        self.fragment_segments_are_formed() && self.forced.is_fragment_like()
    }

    /// 前一半：碎片段里至少九成恰好 32 槽活。
    fn fragment_segments_are_formed(&self) -> bool {
        self.written_segments > 0
            && count_as_float(self.written_segments_with_half_live)
                >= SHARE_OF_WRITTEN_SEGMENTS_WITH_HALF_LIVE_AT_LEAST
                    * count_as_float(self.written_segments)
    }
}

/// 选中的一段的读数（greedy 照核心层重放要选的，或强制选段后备点名的）：起点（一段都没选中是 `None`）、活槽数、未释放记录是不是全是活数据单元（码 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SegmentOfTheChoice {
    start_slot: Option<u64>,
    live_slots: u64,
    holds_only_data_units: bool,
}

impl SegmentOfTheChoice {
    /// 登记第五节「负载」第 4 条的判法：选中了一段，它活槽不多于 32、未释放记录全是活数据单元。
    fn is_fragment_like(self) -> bool {
        self.start_slot.is_some()
            && self.live_slots <= LIVE_SLOTS_OF_A_FRAGMENTED_SEGMENT
            && self.holds_only_data_units
    }
}

/// 单元区里装着不是活数据单元的未释放记录的段（按段起点），一遍扫完全部记录。
fn segments_holding_a_record_other_than_a_data_unit(
    records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    data_slots_now: &BTreeSet<u64>,
) -> BTreeSet<u64> {
    let unit_area_start_slot = unit_area.start.slot().0;
    records
        .iter()
        .filter(|record| {
            record.device == unit_area.device
                && !record.is_released
                && record.slot.0 >= unit_area_start_slot
                && !data_slots_now.contains(&record.slot.0)
        })
        .map(|record| segment_start_of(record.slot.0, unit_area_start_slot))
        .collect()
}

/// 起点 `start_slot` 那一段的读数。`live_slots_of_each_segment` 按段起点从小到大一段一个（`live_slots_per_cluster_segment_by_the_records`）。
fn segment_of_the_choice(
    start_slot: Option<u64>,
    unit_area_start_slot: u64,
    live_slots_of_each_segment: &[u64],
    segments_holding_other_records: &BTreeSet<u64>,
) -> SegmentOfTheChoice {
    let live_slots = start_slot
        .and_then(|start| start.checked_sub(unit_area_start_slot))
        .and_then(|offset| usize::try_from(offset / LOCAL_CLUSTER_SEGMENT_SLOTS).ok())
        .and_then(|segment_index| live_slots_of_each_segment.get(segment_index).copied())
        .unwrap_or(0);
    SegmentOfTheChoice {
        start_slot,
        live_slots,
        holds_only_data_units: start_slot
            .is_some_and(|start| !segments_holding_other_records.contains(&start)),
    }
}

/// 强制选段后备（登记第十二节修订第 8 条）这一次点名的段：碎片段里还没点过名的、只装活数据单元、有活槽而不多于 32 的段里起点最小的那一段
/// （判法同第五节「负载」第 4 条；没有活槽的段核心层签发时照「一个可搬的单元都没有」拒，不点它）。一段都没有（碎片段用完）交回 `None`。
fn fragment_segment_to_force(
    fragment_segment_starts: &BTreeSet<u64>,
    segments_already_forced: &BTreeSet<u64>,
    unit_area_start_slot: u64,
    live_slots_of_each_segment: &[u64],
    segments_holding_other_records: &BTreeSet<u64>,
) -> Option<SegmentOfTheChoice> {
    fragment_segment_starts
        .iter()
        .filter(|start| !segments_already_forced.contains(start))
        .map(|start| {
            segment_of_the_choice(
                Some(*start),
                unit_area_start_slot,
                live_slots_of_each_segment,
                segments_holding_other_records,
            )
        })
        .find(|segment| segment.live_slots > 0 && segment.is_fragment_like())
}

/// 几何核的纯函数：盘 0 的记录、单元区、碎片段（起点）、此刻活数据单元的槽、greedy 要选的那一段与强制选段后备点名的那一段（起点）。
fn geometry_of_the_records(
    records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    written_segment_starts: &BTreeSet<u64>,
    data_slots_now: &BTreeSet<u64>,
    greedy_segment_start_slot: Option<u64>,
    forced_segment_start_slot: Option<u64>,
) -> GeometryReading {
    let unit_area_start_slot = unit_area.start.slot().0;
    let live_slots = live_slots_per_cluster_segment_by_the_records(records, unit_area);
    let segments_holding_other_records =
        segments_holding_a_record_other_than_a_data_unit(records, unit_area, data_slots_now);
    let mut live_slot_histogram: BTreeMap<u64, u64> = BTreeMap::new();
    let mut written_segments_with_half_live = 0_u64;
    for (segment_start_slot, live) in (0_u64..)
        .map(|segment_index| unit_area_start_slot + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS)
        .zip(live_slots.iter().copied())
    {
        *live_slot_histogram.entry(live).or_insert(0) += 1;
        if written_segment_starts.contains(&segment_start_slot)
            && live == LIVE_SLOTS_OF_A_FRAGMENTED_SEGMENT
        {
            written_segments_with_half_live += 1;
        }
    }
    GeometryReading {
        written_segments: u64::try_from(written_segment_starts.len()).expect("段数"),
        written_segments_with_half_live,
        greedy: segment_of_the_choice(
            greedy_segment_start_slot,
            unit_area_start_slot,
            &live_slots,
            &segments_holding_other_records,
        ),
        forced: segment_of_the_choice(
            forced_segment_start_slot,
            unit_area_start_slot,
            &live_slots,
            &segments_holding_other_records,
        ),
        live_slot_histogram,
    }
}

/// 整理不许搬、罩住它就不选那一段的槽：现行那一版里中央映射树各节点、树表、实例表链各片（核心层 `mounted_session.rs` 私有的
/// `version_as_the_compaction_reads_it` 那一份 `pinned_slots`，这里另写一份）。
fn slots_exempt_from_the_mapping(version: &TransactionOutput) -> BTreeSet<SlotNumber> {
    version
        .units
        .iter()
        .filter(|published| {
            matches!(
                published.identity,
                TransactionUnit::MappingTreeNodeBelowTheRoot(_)
                    | TransactionUnit::MappingTree
                    | TransactionUnit::TreeTable
                    | TransactionUnit::InstanceTable
                    | TransactionUnit::InstanceTablePageAfterTheFirst(_)
            )
        })
        .flat_map(|published| {
            (published.slot.0..published.slot.0 + published.identity.span_slots()).map(SlotNumber)
        })
        .collect()
}

/// greedy 此刻要选的那一段（登记第五节「负载」第 4 条；第十二节修订第 8 条要每条意图照打）：照核心层签发那一步重放 `choose_region` 与「先扫」那一圈
/// （`mounted_session.rs` 签发那一支，判据 Greedy）。`segments_opened_by_this_mount` 是这次挂载开过的聚簇段（重挂之后还没开过的取空），
/// `refused` 是会话此刻记着的扫描拒过的段；钉住的槽与常驻隔离集合起来是不许选的槽；读单元从两块内存盘读。一段都没资格交回 `None`。
fn segment_the_greedy_choice_picks(
    version: &TransactionOutput,
    unit_area: UnitAreaOfOneDevice,
    segments_opened_by_this_mount: &BTreeSet<SlotNumber>,
    refused: SegmentsRefusedByTheRegionScan,
    quarantined_slots: &BTreeSet<SlotNumber>,
    devices: &PoolDevices,
) -> Option<u64> {
    let mapping_entries = central_mapping_as_the_compaction_reads_it(version);
    let pinned_slots = slots_exempt_from_the_mapping(version);
    let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> =
        pinned_slots.union(quarantined_slots).copied().collect();
    let reader = ReaderOfTheMemoryPool {
        devices: devices.as_slice(),
    };
    let mut region_scans = region_scans_of_the_segments_without_an_unreleased_record(
        &version.allocation_records,
        unit_area,
    );
    let mut refused = refused;
    // 迭代上界：每答一次「先扫」下一次就多一段有结论，不超过单元区的整段数（同核心层那一圈）。
    for _choice in 0..=unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS {
        match choose_region(
            &version.allocation_records,
            unit_area,
            segments_opened_by_this_mount,
            None,
            &slots_that_disqualify_a_segment,
            &region_scans,
            &refused,
            RegionChoiceCriterion::Greedy,
        ) {
            RegionChoice::Chosen(region) => return Some(region.start_slot().0),
            RegionChoice::ScanBeforeChoosing(region) => match scan_region(
                region,
                &version.allocation_records,
                &mapping_entries,
                &pinned_slots,
                quarantined_slots,
                |device, slot: SlotNumber, length| {
                    reader.read(device, slot.to_device_offset(), length)
                },
            ) {
                Ok(scan) => region_scans.push(scan),
                Err(refusal) => refused.note_refusal(region, refusal),
            },
            RegionChoice::NoEligibleSegment => return None,
        }
    }
    None
}

/// 一档的起步镜像：两块内存盘（关掉、没卸载）、活数据文件那 P ÷ K 个 inode 从几号起、每个单元最后一次写的是第几次、碎片段，与造的经过里要报的数。
/// 活数据单元第 u 个住 inode `lowest_live_data_inode + u ÷ K`，是那个文件里第 u mod K 个单元（[`file_and_index_of_a_live_data_unit`]）。
struct StartingImage {
    tier: UnitAreaTier,
    devices: PoolDevices,
    lowest_live_data_inode: u64,
    /// 交错那一段写满的段（碎片段，按段起点）：强制选段后备从这里点名（登记第十二节修订第 8 条）。
    fragment_segment_starts: BTreeSet<u64>,
    write_generation_of_each_unit: Vec<u64>,
    next_write_time_seconds: u64,
    construction: ConstructionReport,
}

#[derive(Clone, Debug)]
struct ConstructionReport {
    construction_seconds: f64,
    publishes: u64,
    floor_raises: u64,
    release_rounds: u64,
    /// 垫片文件删掉之后，它每个单元的旧落点都按记录已释放。
    is_every_spacer_slot_released: bool,
    /// 垫片文件的单元数，其中补齐段边界的那几个（不算进碎片段）。
    spacer_units: u64,
    alignment_spacer_units: u64,
    /// 文件的条数（登记第五节「负载」第 4 条改的造法：活数据文件与垫片文件各 K 个单元，一层 inode 树装得下）：活数据文件 P ÷ K 个、
    /// 垫片文件 ⌈垫片单元数 ÷ K⌉ 个、造数据挂载之前树里已有的 inode 记录，与造数据挂载写完那一刻（造池的峰）树里的记录条数与叶容器片数。
    live_data_files: u64,
    spacer_files: u64,
    inode_records_before_the_data_mount: u64,
    inode_records_at_the_peak: u64,
    inode_leaf_containers_at_the_peak: u64,
    /// 造完那一版里数到的活数据单元个数（应等于 P）。
    live_data_units_found: u64,
    empty_segments: u64,
    mapped_units: u64,
    geometry: GeometryReading,
    /// 各阶段量到的（[`ConstructionClock`]），按造的先后一阶段一项。
    phases: Vec<TimingOfAConstructionPhase>,
}

// ---- 造池分阶段计时与进度行 ----

/// 造池的进度间隔（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」一节：进度间隔 5 分钟）。
const CONSTRUCTION_PROGRESS_INTERVAL: Duration = Duration::from_secs(300);

/// 造池的阶段，按造的先后（分阶段计时与进度行按这一份报）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConstructionPhase {
    MakeFilesystemAndTheFirstFile,
    DataMount,
    CreatingTheLiveDataInodes,
    SequentialHalf,
    AlignmentSpacer,
    InterleavedHalf,
    FragmentMount,
    UnlinkingTheSpacerFiles,
    ReleasingTheSpacerSlots,
    ReadingTheGeometry,
}

impl ConstructionPhase {
    #[cfg(test)]
    const EVERY_PHASE: [ConstructionPhase; 10] = [
        ConstructionPhase::MakeFilesystemAndTheFirstFile,
        ConstructionPhase::DataMount,
        ConstructionPhase::CreatingTheLiveDataInodes,
        ConstructionPhase::SequentialHalf,
        ConstructionPhase::AlignmentSpacer,
        ConstructionPhase::InterleavedHalf,
        ConstructionPhase::FragmentMount,
        ConstructionPhase::UnlinkingTheSpacerFiles,
        ConstructionPhase::ReleasingTheSpacerSlots,
        ConstructionPhase::ReadingTheGeometry,
    ];

    const fn name(self) -> &'static str {
        match self {
            ConstructionPhase::MakeFilesystemAndTheFirstFile => {
                "make_filesystem_and_the_first_file"
            }
            ConstructionPhase::DataMount => "data_mount",
            ConstructionPhase::CreatingTheLiveDataInodes => "creating_the_live_data_inodes",
            ConstructionPhase::SequentialHalf => "sequential_half",
            ConstructionPhase::AlignmentSpacer => "alignment_spacer",
            ConstructionPhase::InterleavedHalf => "interleaved_half",
            ConstructionPhase::FragmentMount => "fragment_mount",
            ConstructionPhase::UnlinkingTheSpacerFiles => "unlinking_the_spacer_files",
            ConstructionPhase::ReleasingTheSpacerSlots => "releasing_the_spacer_slots",
            ConstructionPhase::ReadingTheGeometry => "reading_the_geometry",
        }
    }
}

/// 本进程此刻的常驻内存（`/proc/self/status` 的 `VmRSS`）；读不到交 `None`。
fn resident_bytes_of_this_process() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    resident_bytes_of_the_status(&status)
}

/// `/proc/<pid>/status` 正文里 `VmRSS:   <千字节> kB` 那一行换成字节。
fn resident_bytes_of_the_status(status: &str) -> Option<u64> {
    let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
    let kibibytes: u64 = line
        .strip_prefix("VmRSS:")?
        .trim()
        .strip_suffix("kB")?
        .trim()
        .parse()
        .ok()?;
    kibibytes.checked_mul(1024)
}

/// 正在造的那一阶段：起点、做完的步数与它至多几步、每一步的挂钟（秒）、开始时已发成的版数。
struct PhaseInProgress {
    phase: ConstructionPhase,
    started: Instant,
    last_step_ended: Instant,
    planned_steps_at_most: u64,
    step_seconds: Vec<f64>,
    publishes_before_the_phase: u64,
}

/// 一个阶段量到的：挂钟、步数（一步是一个单元、一次建 inode、一个删掉的文件、一轮抬 F，见 [`ConstructionPhase`]）、这一阶段发成的版数、
/// 每一步的挂钟（头八分之一、末八分之一的平均与最长那一步：看一步的用时随池里的东西多起来涨不涨），阶段末了现行那一版的大小、
/// 本进程常驻内存与内存盘的块字节。
#[derive(Clone, Debug)]
struct TimingOfAConstructionPhase {
    phase: ConstructionPhase,
    seconds: f64,
    steps: u64,
    planned_steps_at_most: u64,
    publishes: u64,
    step_milliseconds_mean_of_the_earliest_eighth: f64,
    step_milliseconds_mean_of_the_latest_eighth: f64,
    step_milliseconds_maximum: f64,
    units_in_the_version: Option<u64>,
    allocation_records_in_the_version: Option<u64>,
    resident_bytes: Option<u64>,
    chunk_bytes: ChunkBytesOfThePool,
}

/// 几步的平均毫秒；一步都没有交 0。
fn mean_milliseconds_of(seconds: &[f64]) -> f64 {
    if seconds.is_empty() {
        return 0.0;
    }
    seconds.iter().sum::<f64>() * 1000.0
        / count_as_float(u64::try_from(seconds.len()).expect("步数装得进 u64"))
}

impl TimingOfAConstructionPhase {
    fn of_the_phase(
        phase: PhaseInProgress,
        devices: &PoolDevices,
        version: Option<&TransactionOutput>,
        publishes_after_the_phase: u64,
    ) -> TimingOfAConstructionPhase {
        let steps = phase.step_seconds.len();
        // 头尾各取八分之一，至少一步；一步都没有就是空的。
        let eighth = if steps == 0 { 0 } else { (steps / 8).max(1) };
        TimingOfAConstructionPhase {
            phase: phase.phase,
            seconds: phase.started.elapsed().as_secs_f64(),
            steps: u64::try_from(steps).expect("步数装得进 u64"),
            planned_steps_at_most: phase.planned_steps_at_most,
            publishes: publishes_after_the_phase - phase.publishes_before_the_phase,
            step_milliseconds_mean_of_the_earliest_eighth: mean_milliseconds_of(
                &phase.step_seconds[..eighth],
            ),
            step_milliseconds_mean_of_the_latest_eighth: mean_milliseconds_of(
                &phase.step_seconds[steps - eighth..],
            ),
            step_milliseconds_maximum: phase.step_seconds.iter().copied().fold(0.0, f64::max)
                * 1000.0,
            units_in_the_version: version
                .map(|at_the_end| u64::try_from(at_the_end.units.len()).expect("条数装得进 u64")),
            allocation_records_in_the_version: version.map(|at_the_end| {
                u64::try_from(at_the_end.allocation_records.len()).expect("条数装得进 u64")
            }),
            resident_bytes: resident_bytes_of_this_process(),
            chunk_bytes: chunk_bytes_of_the_pool(devices),
        }
    }

    /// 结果行（`name=construction_phase`）的字段。
    fn fields(&self) -> String {
        let optional = |value: Option<u64>| {
            value.map_or_else(|| "none".to_string(), |count| count.to_string())
        };
        format!(
            "phase={} seconds={:.3} steps={} planned_steps_at_most={} publishes={} \
             step_milliseconds_mean_of_the_earliest_eighth={:.3} step_milliseconds_mean_of_the_latest_eighth={:.3} \
             step_milliseconds_maximum={:.3} units_in_the_version={} allocation_records_in_the_version={} \
             resident_bytes={} chunk_bytes_held_by_the_devices={} chunk_bytes_stored={}",
            self.phase.name(),
            self.seconds,
            self.steps,
            self.planned_steps_at_most,
            self.publishes,
            self.step_milliseconds_mean_of_the_earliest_eighth,
            self.step_milliseconds_mean_of_the_latest_eighth,
            self.step_milliseconds_maximum,
            optional(self.units_in_the_version),
            optional(self.allocation_records_in_the_version),
            optional(self.resident_bytes),
            self.chunk_bytes.held_by_the_devices,
            self.chunk_bytes.stored,
        )
    }
}

/// 造池的钟：分阶段计时（每阶段一项 [`TimingOfAConstructionPhase`]），每过一个进度间隔往 `progress` 打一行进度
/// （阶段、已做/至多几步、开造起已用秒数、此刻常驻内存；`.claude/rules/test-script-discipline.md`「编译与算法都要快」那一节的进度行）。
/// 只在阶段开头、每做完一步与阶段末了查一次到没到间隔；头一次查就打。计时都在被测进程里、造池那条线程上读单调钟。
struct ConstructionClock<'progress> {
    pool: String,
    started: Instant,
    progress_interval: Duration,
    last_progress: Option<Instant>,
    progress: &'progress mut dyn FnMut(&str),
    phase_in_progress: Option<PhaseInProgress>,
    timings: Vec<TimingOfAConstructionPhase>,
}

impl<'progress> ConstructionClock<'progress> {
    fn new(
        pool: String,
        progress_interval: Duration,
        progress: &'progress mut dyn FnMut(&str),
    ) -> ConstructionClock<'progress> {
        ConstructionClock {
            pool,
            started: Instant::now(),
            progress_interval,
            last_progress: None,
            progress,
            phase_in_progress: None,
            timings: Vec::new(),
        }
    }

    /// 开一个阶段：它至多几步、开始时已发成几版（造池那一串的累计）。
    fn begin(&mut self, phase: ConstructionPhase, planned_steps_at_most: u64, publishes: u64) {
        let now = Instant::now();
        self.phase_in_progress = Some(PhaseInProgress {
            phase,
            started: now,
            last_step_ended: now,
            planned_steps_at_most,
            step_seconds: Vec::new(),
            publishes_before_the_phase: publishes,
        });
        self.report_if_the_interval_passed();
    }

    /// 做完一步：这一步的挂钟从上一步做完（或阶段开头）量起。
    fn step_done(&mut self) {
        let now = Instant::now();
        if let Some(phase) = &mut self.phase_in_progress {
            phase
                .step_seconds
                .push(now.duration_since(phase.last_step_ended).as_secs_f64());
            phase.last_step_ended = now;
        }
        self.report_if_the_interval_passed();
    }

    /// 正在造的那一阶段收尾：量阶段末了的版本大小、内存与块字节，记成一项。`publishes` 是此刻已发成的版数（造池那一串的累计）。
    fn finish(
        &mut self,
        devices: &PoolDevices,
        version: Option<&TransactionOutput>,
        publishes: u64,
    ) {
        self.report_if_the_interval_passed();
        if let Some(phase) = self.phase_in_progress.take() {
            let timing =
                TimingOfAConstructionPhase::of_the_phase(phase, devices, version, publishes);
            self.timings.push(timing);
        }
    }

    fn report_if_the_interval_passed(&mut self) {
        let now = Instant::now();
        let is_due = self
            .last_progress
            .is_none_or(|last| now.duration_since(last) >= self.progress_interval);
        if is_due {
            let (phase_name, steps, planned_steps_at_most) = self
                .phase_in_progress
                .as_ref()
                .map_or(("none", 0, 0), |phase| {
                    (
                        phase.phase.name(),
                        phase.step_seconds.len(),
                        phase.planned_steps_at_most,
                    )
                });
            let line = format!(
                "{RESULT_LINE_PREFIX} 造池进度 pool={} phase={phase_name} done={steps} planned_at_most={planned_steps_at_most} \
                 elapsed_seconds={:.1} resident_bytes={}",
                self.pool,
                now.duration_since(self.started).as_secs_f64(),
                resident_bytes_of_this_process()
                    .map_or_else(|| "unread".to_string(), |bytes| bytes.to_string()),
            );
            (self.progress)(&line);
            self.last_progress = Some(now);
        }
    }
}

/// 建池会话发一次用户改动：交回发成的版数与推的抬 F 串数。
fn publish_while_constructing(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    change: UserChange<'_>,
) -> (u64, u64) {
    let published = session
        .publish_user_change(devices, change)
        .unwrap_or_else(|refusal| panic!("建池的用户改动发得成：{refusal:?}"));
    (
        u64::try_from(published.versions_published.len()).expect("版数"),
        u64::try_from(published.floor_raises.len()).expect("抬 F 串数"),
    )
}

/// 抬 F 到准入上限（会话的分配器与现行那一版，同 E185（GC一批搬迁的挂钟拆分） 装置的做法）。
fn raise_the_floor_to_the_admission_ceiling(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
) {
    let mut current = version_of_trees(session).clone();
    let _raised = raise_rollback_floor_to_the_admission_ceiling(
        &make_filesystem_parameters_of_the_pool(),
        devices,
        &mut session.allocator,
        &mut current,
        session.shadow_ledger,
    )
    .expect("抬 F 到准入上限");
    session.current = PoolVersion::of_a_version_of_trees(current);
}

/// 造数据挂载建空 inode 时一次发布至多建几个（一片 inode 叶容器装 233 条，4096 个约 18 片；文件一次建完的那一次发布太大）。
const INODES_CREATED_BY_ONE_PUBLISH: u64 = 4096;

/// 一个活数据文件装几个数据单元 K（登记第五节「负载」第 4 条「几何没造成就改负载重造」改的造法）。P 个一单元文件装不进一层 inode 树：
/// 树还不会长高（`crates/singlefs-core/src/inode_tree.rs` 的 `InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds`），一层装几条由
/// [`inode_records_one_level_tree_holds`] 照核心层现算（一片叶容器 233 条 × 一个码 2 根扇出 135 = 31455），三档的 P 是 36608、65536、131072。
/// 取 16：P 是 32 的倍数，P ÷ 2 就是 16 的倍数，顺序写那一半与交错那一半各自从文件边界起；16 不超过 extent 下段一片叶罩的 144 个单元，
/// 每个文件的下段只有一片叶。
const LIVE_DATA_UNITS_PER_FILE: u64 = 16;

/// 一个垫片文件装几个单元：与活数据文件等长。
const SPACER_UNITS_PER_FILE: u64 = LIVE_DATA_UNITS_PER_FILE;

/// 活数据文件的个数 P ÷ K。
const fn live_data_files(live_data_units: u64) -> u64 {
    live_data_units / LIVE_DATA_UNITS_PER_FILE
}

/// 活数据单元第 `unit_index` 个住第几个活数据文件（从 `lowest_live_data_inode` 数起）、是那个文件里第几个单元。
const fn file_and_index_of_a_live_data_unit(unit_index: u64) -> (u64, u64) {
    (
        unit_index / LIVE_DATA_UNITS_PER_FILE,
        unit_index % LIVE_DATA_UNITS_PER_FILE,
    )
}

/// 垫片单元至多几个：交错那一段 P ÷ 2 个，加补齐段边界的至多一段那么多（造数据挂载补齐那一圈的迭代上界）。
const fn spacer_units_at_most(live_data_units: u64) -> u64 {
    live_data_units / 2 + data_units_of_a_cluster_segment()
}

/// 垫片文件至多几个。
const fn spacer_files_at_most(live_data_units: u64) -> u64 {
    spacer_units_at_most(live_data_units).div_ceil(SPACER_UNITS_PER_FILE)
}

/// 造一档的池 inode 树至多要装几条记录：造数据挂载之前已有的、活数据文件、垫片文件至多那么多，加造碎片挂载推发布建的空 inode 至多
/// [`RELEASE_ROUNDS_AT_MOST`] 个（那时垫片文件已删掉，这里照样加上，往大里算）。
const fn inode_records_of_the_construction_at_most(
    live_data_units: u64,
    inode_records_before_the_data_mount: u64,
) -> u64 {
    inode_records_before_the_data_mount
        + live_data_files(live_data_units)
        + spacer_files_at_most(live_data_units)
        + RELEASE_ROUNDS_AT_MOST
}

/// 号从 1 起连着的 `records` 条 inode 记录一次写进空树，核心层写记录那一步（`inode_tree::write_records_into_leaf_containers`）交回什么。
/// 叶只在右端满了才分裂，片数只由条数定；出生树、出生代只进容器身份，不影响片数。
fn inode_records_written_into_an_empty_tree(
    records: u64,
) -> Result<InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal> {
    let records: Vec<InodeRecord> = (1..=records)
        .map(|inode| InodeRecord {
            inode,
            object_birth: CheckpointTxg(1),
            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size: 0,
            change_count: 0,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
        })
        .collect();
    write_records_into_leaf_containers(&[], &records, CheckpointTxg(1), TreeIdentifier(0))
}

/// `records` 条记录在一层 inode 树里占几片叶容器；装不下（片数超过一个码 2 根的扇出）交回核心层的拒绝。
fn inode_leaf_containers_of_a_one_level_tree(records: u64) -> Result<u64, InodeTreeWriteRefusal> {
    inode_records_written_into_an_empty_tree(records)
        .map(|after| u64::try_from(after.containers.len()).expect("片数装得进 u64"))
}

/// 一层 inode 树至多装几条记录：一片叶容器的条数（核心层写一条进空树之后那一片的条数加它的空位）乘一个码 2 根的扇出。
fn inode_records_one_level_tree_holds() -> u64 {
    let after = inode_records_written_into_an_empty_tree(1).expect("一条记录装得进空树");
    let container = &after.containers[0];
    let records_of_a_container = u64::try_from(container.records.len()).expect("条数装得进 u64")
        + container.free_record_slots();
    records_of_a_container
        * u64::try_from(leaf_containers_one_root_node_holds()).expect("扇出装得进 u64")
}

/// 这一版 inode 树里的记录条数。
fn inode_records_of_the_version(version: &TransactionOutput) -> u64 {
    version
        .inode_leaf_containers
        .iter()
        .map(|container| u64::try_from(container.contents.records.len()).expect("条数装得进 u64"))
        .sum()
}

/// 往一个文件末尾接一个单元（文件已有 `index_in_file` 个整单元，写在第 `index_in_file` 个单元的起点），内容是第 `content_unit_index` 个单元
/// 第 `write_generation` 次写的。按偏移写只重写偏移所在的单元到写之后的最后一个单元，接在末尾就只写这一个新单元
/// （`transaction::publish_partial_write` 文档 R4）。交回发成的版数与推的抬 F 串数。
fn append_a_unit_to_the_end_of_a_file(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    inode: u64,
    index_in_file: u64,
    content_unit_index: u64,
    write_generation: u64,
    write_time_seconds: u64,
) -> (u64, u64) {
    let mut unit_bytes = vec![0u8; data_unit_payload_capacity()];
    fill_the_content_of_a_data_unit(content_unit_index, write_generation, &mut unit_bytes);
    let capacity = u64::try_from(unit_bytes.len()).expect("净荷容量装得进 u64");
    publish_while_constructing(
        session,
        devices,
        UserChange::WriteAtOffset {
            inode: InodeNumber(inode),
            offset: FileOffsetInBytes(index_in_file * capacity),
            bytes: PartialWriteBytes::new(&unit_bytes).expect("一个单元的净荷不是空的"),
            write_time_seconds,
        },
    )
}

/// 给活数据单元第 `unit_index` 个写它的内容（第 0 次写）：接在它那个活数据文件末尾（文件里它前面的单元都已写过）。交回发成的版数与推的抬 F 串数。
fn write_a_live_data_unit(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    lowest_live_data_inode: u64,
    unit_index: u64,
    write_time_seconds: u64,
) -> (u64, u64) {
    let (file, index_in_file) = file_and_index_of_a_live_data_unit(unit_index);
    append_a_unit_to_the_end_of_a_file(
        session,
        devices,
        lowest_live_data_inode + file,
        index_in_file,
        unit_index,
        WRITE_GENERATION_OF_THE_DATA_MOUNT,
        write_time_seconds,
    )
}

/// 接第 `spacer_index` 个垫片单元（垫片单元按接的次序连着编号，第 s 个住第 s ÷ K 个垫片文件、是那个文件里第 s mod K 个单元）。
/// 要接的是一个垫片文件的头一个单元时，先发一次建一个空 inode 的改动建这个文件（号接着水位发，核过等于 `first_spacer_inode` 加文件序号）。
/// 交回发成的版数与推的抬 F 串数（两次改动合起来）。
fn append_a_spacer_unit(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    first_spacer_inode: u64,
    spacer_index: u64,
    next_write_time: &mut impl FnMut() -> u64,
) -> (u64, u64) {
    let spacer_file = spacer_index / SPACER_UNITS_PER_FILE;
    let index_in_file = spacer_index % SPACER_UNITS_PER_FILE;
    let mut publishes = 0_u64;
    let mut floor_raises = 0_u64;
    if index_in_file == 0 {
        let (published_by_the_new_file, raised_by_the_new_file) = publish_while_constructing(
            session,
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: next_write_time(),
            },
        );
        publishes += published_by_the_new_file;
        floor_raises += raised_by_the_new_file;
        assert_eq!(
            version_of_trees(session).inode_number_watermark(),
            first_spacer_inode + spacer_file + 1,
            "垫片文件的号接着活数据文件连着发"
        );
    }
    let (published_by_the_unit, raised_by_the_unit) = append_a_unit_to_the_end_of_a_file(
        session,
        devices,
        first_spacer_inode + spacer_file,
        index_in_file,
        spacer_index,
        WRITE_GENERATION_OF_THE_SPACER_FILE,
        next_write_time(),
    );
    (
        publishes + published_by_the_unit,
        floor_raises + raised_by_the_unit,
    )
}

/// 造一档的起步镜像（登记第五节「负载」；第 4 条「几何没造成就改负载重造」改过的造法，原因见本段末尾）：
/// 1. 新池新建文件；
/// 2. 造数据挂载：建 P ÷ K 个空 inode 当活数据文件（K = [`LIVE_DATA_UNITS_PER_FILE`]；活数据单元第 u 个住 inode `first + u ÷ K`、
///    是那个文件里第 u mod K 个单元），每个单元都接在它那个文件末尾写（按偏移写只写这一个新单元，R4）。
///    先把后一半活数据单元一个一个顺序写（每次一次发布、用户数据取最低的空槽对，连着写满 P ÷ 64 段，前面挂载留下的空洞也在这时填掉）；
///    下一个用户数据落点不在段边界上就接垫片单元补齐；再把前一半活数据单元与垫片单元交错着写（一个活的、一个垫片），
///    写满 P ÷ 32 段，每段 16 个活的、16 个垫片；垫片单元连着装进与活数据文件等长的垫片文件，要接一个垫片文件的头一个单元时才建那个文件；
///    会话丢掉；
/// 3. 造碎片挂载：重挂，逐个删垫片文件，抬 F 到准入上限直到垫片的旧落点按记录都已释放（迭代上界 [`RELEASE_ROUNDS_AT_MOST`]；
///    每轮没放完再发一次建一个 inode 的改动推一次发布），会话丢掉。这一步不再写用户数据，空出来的槽没有谁来填；
/// 4. 几何核：碎片段（交错那一段里有垫片单元的段）活槽恰好 32 的占比，greedy 照核心层重放要选的那一段。
///
/// 不用「按偏移覆盖写一个大文件里下标为奇数的单元」：核心层按偏移写把偏移所在的单元到文件末尾一起重写（R4），覆盖写第 u 个单元
/// 要重写 P − u 个，段里留不下一半活槽；前台用户改动同理只覆盖写一个活数据文件的最后一个单元（[`publish_the_user_change`]）。
/// 不用 P 个一单元文件：三档的 P 都超过一层 inode 树装得下的文件数（[`inode_records_one_level_tree_holds`]），建池那一步核心层拒；
/// 开造之前先照核心层算一遍这一档至多要几条 inode 记录（[`inode_records_of_the_construction_at_most`]）装不装得下。
///
/// 关掉（不卸载）交回两块内存盘；量挂载那一侧再可写挂载，这样造出来的段不是那次挂载开过的段（`choose_region` 的资格）。
fn construct_the_starting_image(tier: UnitAreaTier) -> StartingImage {
    construct_a_starting_image_of_the_size(tier, tier.device_bytes(), tier.live_data_units())
}

/// 同 [`construct_the_starting_image`]，盘大小与活数据单元数由调用方给（同包测试在小池上走一遍整条路）。块照共用的存法存，
/// 每 [`CONSTRUCTION_PROGRESS_INTERVAL`] 往标准错误打一行进度。
fn construct_a_starting_image_of_the_size(
    tier: UnitAreaTier,
    device_bytes: u64,
    live_data_units: u64,
) -> StartingImage {
    let mut progress_to_the_standard_error = |line: &str| eprintln!("{line}");
    let mut clock = ConstructionClock::new(
        format!("{}-P{live_data_units}", tier.name()),
        CONSTRUCTION_PROGRESS_INTERVAL,
        &mut progress_to_the_standard_error,
    );
    construct_a_starting_image_on_the_clock(
        tier,
        device_bytes,
        live_data_units,
        ChunkStorage::SharedWhenTheBytesAreEqual,
        &mut clock,
    )
}

/// 同 [`construct_a_starting_image_of_the_size`]，块照 `storage` 存，分阶段计时与进度行走 `clock`（各阶段的读数取走放进
/// [`ConstructionReport::phases`]）。两种存法发的改动、读写的字节逐个相同。
fn construct_a_starting_image_on_the_clock(
    tier: UnitAreaTier,
    device_bytes: u64,
    live_data_units: u64,
    storage: ChunkStorage,
    clock: &mut ConstructionClock<'_>,
) -> StartingImage {
    assert_eq!(
        live_data_units % data_units_of_a_cluster_segment(),
        0,
        "P 是 32 的倍数：交错那一段一半活的、一半垫片，正好写满整段"
    );
    assert_eq!(
        (live_data_units / 2) % LIVE_DATA_UNITS_PER_FILE,
        0,
        "P ÷ 2 是 K 的倍数：顺序写那一半与交错那一半各自从文件边界起"
    );
    let started = Instant::now();
    let parameters = make_filesystem_parameters_of_the_pool();
    clock.begin(ConstructionPhase::MakeFilesystemAndTheFirstFile, 1, 0);
    let mut devices = new_pool_with_the_first_file(device_bytes, storage);
    clock.step_done();
    clock.finish(&devices, None, 0);
    let mut publishes = 0_u64;
    let mut floor_raises = 0_u64;
    let mut write_time_seconds = FIXED_WRITE_TIME_SECONDS;
    let mut next_write_time = || {
        write_time_seconds += 1;
        write_time_seconds
    };
    let units_interleaved_with_the_spacer = live_data_units / 2;
    let live_data_files_of_the_pool = live_data_files(live_data_units);
    let lowest_live_data_inode: u64;
    let first_spacer_inode: u64;
    let mut spacer_units = 0_u64;
    let alignment_spacer_units: u64;
    let spacer_files: u64;
    let spacer_slots: BTreeMap<(u64, u64), u64>;
    let inode_records_before_the_data_mount: u64;
    let inode_records_at_the_peak: u64;
    let inode_leaf_containers_at_the_peak: u64;
    {
        clock.begin(ConstructionPhase::DataMount, 1, publishes);
        let mounted = mount_writable(&parameters, &mut devices).expect("造数据挂载那一次可写挂载");
        let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
        clock.step_done();
        clock.finish(&devices, Some(version_of_trees(&session)), publishes);
        lowest_live_data_inode = version_of_trees(&session).inode_number_watermark();
        inode_records_before_the_data_mount =
            inode_records_of_the_version(version_of_trees(&session));
        let inode_records_at_most = inode_records_of_the_construction_at_most(
            live_data_units,
            inode_records_before_the_data_mount,
        );
        if let Err(refusal) = inode_leaf_containers_of_a_one_level_tree(inode_records_at_most) {
            panic!(
                "造池至多要 {inode_records_at_most} 条 inode 记录，一层 inode 树装不下（至多 {} 条）：{refusal:?}",
                inode_records_one_level_tree_holds()
            );
        }
        let inodes_to_create = live_data_files_of_the_pool;
        let mut inodes_created = 0_u64;
        clock.begin(
            ConstructionPhase::CreatingTheLiveDataInodes,
            inodes_to_create.div_ceil(INODES_CREATED_BY_ONE_PUBLISH),
            publishes,
        );
        // 迭代上界：每次至少建一个，至多 ⌈(P ÷ K) ÷ 4096⌉ 次。
        while inodes_created < inodes_to_create {
            let count = (inodes_to_create - inodes_created).min(INODES_CREATED_BY_ONE_PUBLISH);
            let (published, raised) = publish_while_constructing(
                &mut session,
                &mut devices,
                UserChange::NewInodes {
                    count,
                    write_time_seconds: next_write_time(),
                },
            );
            publishes += published;
            floor_raises += raised;
            inodes_created += count;
            clock.step_done();
        }
        clock.finish(&devices, Some(version_of_trees(&session)), publishes);
        assert_eq!(
            version_of_trees(&session).inode_number_watermark(),
            lowest_live_data_inode + inodes_to_create,
            "建的 inode 号从水位起连着发"
        );
        first_spacer_inode = lowest_live_data_inode + live_data_files_of_the_pool;
        clock.begin(
            ConstructionPhase::SequentialHalf,
            live_data_units - units_interleaved_with_the_spacer,
            publishes,
        );
        for unit_index in units_interleaved_with_the_spacer..live_data_units {
            let (published, raised) = write_a_live_data_unit(
                &mut session,
                &mut devices,
                lowest_live_data_inode,
                unit_index,
                next_write_time(),
            );
            publishes += published;
            floor_raises += raised;
            clock.step_done();
        }
        clock.finish(&devices, Some(version_of_trees(&session)), publishes);
        let unit_area_start_slot = unit_area_of_device_zero(&session).start.slot().0;
        clock.begin(
            ConstructionPhase::AlignmentSpacer,
            data_units_of_a_cluster_segment(),
            publishes,
        );
        // 迭代上界：一段至多 32 个数据单元。落点政策只跳过整段（聚簇段），补齐之后交错那一段从段起点开始。
        for _alignment in 0..data_units_of_a_cluster_segment() {
            let next_data_slot = session.allocator.devices[0]
                .lowest_user_data_slot(session.allocator.cluster_segments())
                .expect("单元区装得下下一个用户数据单元");
            if (next_data_slot.0 - unit_area_start_slot).is_multiple_of(LOCAL_CLUSTER_SEGMENT_SLOTS)
            {
                break;
            }
            let (published, raised) = append_a_spacer_unit(
                &mut session,
                &mut devices,
                first_spacer_inode,
                spacer_units,
                &mut next_write_time,
            );
            publishes += published;
            floor_raises += raised;
            spacer_units += 1;
            clock.step_done();
        }
        clock.finish(&devices, Some(version_of_trees(&session)), publishes);
        alignment_spacer_units = spacer_units;
        clock.begin(
            ConstructionPhase::InterleavedHalf,
            units_interleaved_with_the_spacer,
            publishes,
        );
        for unit_index in 0..units_interleaved_with_the_spacer {
            let (published, raised) = write_a_live_data_unit(
                &mut session,
                &mut devices,
                lowest_live_data_inode,
                unit_index,
                next_write_time(),
            );
            publishes += published;
            floor_raises += raised;
            let (published_by_the_spacer, raised_by_the_spacer) = append_a_spacer_unit(
                &mut session,
                &mut devices,
                first_spacer_inode,
                spacer_units,
                &mut next_write_time,
            );
            publishes += published_by_the_spacer;
            floor_raises += raised_by_the_spacer;
            spacer_units += 1;
            clock.step_done();
        }
        clock.finish(&devices, Some(version_of_trees(&session)), publishes);
        spacer_files = spacer_units.div_ceil(SPACER_UNITS_PER_FILE);
        let version = version_of_trees(&session);
        spacer_slots = data_unit_slots_of_the_inodes(version, first_spacer_inode, spacer_files);
        inode_records_at_the_peak = inode_records_of_the_version(version);
        inode_leaf_containers_at_the_peak =
            u64::try_from(version.inode_leaf_containers.len()).expect("片数装得进 u64");
    }
    clock.begin(ConstructionPhase::FragmentMount, 1, publishes);
    let mounted = mount_writable(&parameters, &mut devices).expect("造碎片挂载那一次可写挂载");
    let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
    clock.step_done();
    clock.finish(&devices, Some(version_of_trees(&session)), publishes);
    let unit_area_start_slot_of_the_pool = unit_area_of_device_zero(&session).start.slot().0;
    let written_segment_starts: BTreeSet<u64> = spacer_slots
        .iter()
        .filter(|((inode, index_in_file), _slot)| {
            (inode - first_spacer_inode) * SPACER_UNITS_PER_FILE + index_in_file
                >= alignment_spacer_units
        })
        .map(|(_key, slot)| segment_start_of(*slot, unit_area_start_slot_of_the_pool))
        .collect();
    clock.begin(
        ConstructionPhase::UnlinkingTheSpacerFiles,
        spacer_files,
        publishes,
    );
    for spacer_file in 0..spacer_files {
        let (published, raised) = publish_while_constructing(
            &mut session,
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(first_spacer_inode + spacer_file),
                write_time_seconds: next_write_time(),
            },
        );
        publishes += published;
        floor_raises += raised;
        clock.step_done();
    }
    clock.finish(&devices, Some(version_of_trees(&session)), publishes);
    let mut release_rounds = 0_u64;
    let mut is_every_spacer_slot_released = false;
    clock.begin(
        ConstructionPhase::ReleasingTheSpacerSlots,
        RELEASE_ROUNDS_AT_MOST,
        publishes,
    );
    while release_rounds < RELEASE_ROUNDS_AT_MOST {
        release_rounds += 1;
        raise_the_floor_to_the_admission_ceiling(&mut session, &mut devices);
        clock.step_done();
        let under_a_record = slots_under_an_unreleased_record_on_device_zero(
            &version_of_trees(&session).allocation_records,
        );
        is_every_spacer_slot_released = spacer_slots
            .values()
            .all(|slot| !under_a_record.contains(slot));
        if is_every_spacer_slot_released {
            break;
        }
        let (published_by_the_new_inode, raised_by_the_new_inode) = publish_while_constructing(
            &mut session,
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: next_write_time(),
            },
        );
        publishes += published_by_the_new_inode;
        floor_raises += raised_by_the_new_inode;
    }
    clock.finish(&devices, Some(version_of_trees(&session)), publishes);
    clock.begin(ConstructionPhase::ReadingTheGeometry, 1, publishes);
    let version = version_of_trees(&session);
    let data_slots_now: BTreeSet<u64> =
        data_unit_slots_of_the_inodes(version, lowest_live_data_inode, live_data_files_of_the_pool)
            .into_values()
            .collect();
    let unit_area = unit_area_of_device_zero(&session);
    // 量挂载是重挂，那一次挂载还没开过聚簇段、还没扫拒过哪一段。
    let greedy_segment_start_slot = segment_the_greedy_choice_picks(
        version,
        unit_area,
        &BTreeSet::new(),
        SegmentsRefusedByTheRegionScan::default(),
        session.compaction.quarantined_slots(),
        &devices,
    );
    let forced_segment = fragment_segment_to_force(
        &written_segment_starts,
        &BTreeSet::new(),
        unit_area.start.slot().0,
        &live_slots_per_cluster_segment_by_the_records(&version.allocation_records, unit_area),
        &segments_holding_a_record_other_than_a_data_unit(
            &version.allocation_records,
            unit_area,
            &data_slots_now,
        ),
    );
    let geometry = geometry_of_the_records(
        &version.allocation_records,
        unit_area,
        &written_segment_starts,
        &data_slots_now,
        greedy_segment_start_slot,
        forced_segment.and_then(|segment| segment.start_slot),
    );
    clock.step_done();
    clock.finish(&devices, Some(version), publishes);
    let construction = ConstructionReport {
        construction_seconds: started.elapsed().as_secs_f64(),
        publishes,
        floor_raises,
        release_rounds,
        is_every_spacer_slot_released,
        spacer_units,
        alignment_spacer_units,
        live_data_files: live_data_files_of_the_pool,
        spacer_files,
        inode_records_before_the_data_mount,
        inode_records_at_the_peak,
        inode_leaf_containers_at_the_peak,
        live_data_units_found: u64::try_from(data_slots_now.len()).expect("单元数"),
        empty_segments: session.allocator.devices[0].empty_segments(),
        mapped_units: u64::try_from(version.mapped_units.len()).expect("映射单元条数"),
        geometry,
        phases: std::mem::take(&mut clock.timings),
    };
    drop(session);
    StartingImage {
        tier,
        devices,
        lowest_live_data_inode,
        fragment_segment_starts: written_segment_starts,
        write_generation_of_each_unit: vec![
            WRITE_GENERATION_OF_THE_DATA_MOUNT;
            usize::try_from(live_data_units)
                .expect("P 装得进 usize")
        ],
        next_write_time_seconds: next_write_time(),
        construction,
    }
}
// ---- 臂与卡（登记第五节「臂」「阳性对照」，第一节「卡」）----

/// 一条臂在哪算。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ComputeOfTheArm {
    /// `background_compute=cpu`，卡与上限不给（产品默认）。
    Cpu,
    /// `gpu`，只点本进程看得到的独显里 PCI 地址最小的那一张，上限 1 GiB。
    GpuOnTheCardWithTheLowestPciAddress,
    /// `gpu`，不给 `background_gpu_devices`，每张卡上限 1 GiB。
    GpuOnEveryCard,
}

/// 多判不多判（阳性对照）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum RejudgingOfTheArm {
    NotRejudged,
    /// 每个确认的条目在 GC 线程上照核心层的 CPU 写法再判一遍（GPU 臂经包装、CPU 臂在每次 GC 调用返回之后、窗口之内）。
    EveryConfirmedEntryRejudgedOnTheCpu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct LockstepArm {
    compute: ComputeOfTheArm,
    rejudging: RejudgingOfTheArm,
}

/// 交替跑的六条臂，按轮换的基准次序（第 r 次重复从第 r 条起轮换，登记第十四节）。
const LOCKSTEP_ARMS: [LockstepArm; 6] = [
    LockstepArm {
        compute: ComputeOfTheArm::Cpu,
        rejudging: RejudgingOfTheArm::NotRejudged,
    },
    LockstepArm {
        compute: ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        rejudging: RejudgingOfTheArm::NotRejudged,
    },
    LockstepArm {
        compute: ComputeOfTheArm::GpuOnEveryCard,
        rejudging: RejudgingOfTheArm::NotRejudged,
    },
    LockstepArm {
        compute: ComputeOfTheArm::Cpu,
        rejudging: RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu,
    },
    LockstepArm {
        compute: ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        rejudging: RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu,
    },
    LockstepArm {
        compute: ComputeOfTheArm::GpuOnEveryCard,
        rejudging: RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu,
    },
];

impl ComputeOfTheArm {
    const fn name(self) -> &'static str {
        match self {
            ComputeOfTheArm::Cpu => "cpu",
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress => "gpu_one_card",
            ComputeOfTheArm::GpuOnEveryCard => "gpu_every_card",
        }
    }

    const fn uses_the_gpu(self) -> bool {
        match self {
            ComputeOfTheArm::Cpu => false,
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress
            | ComputeOfTheArm::GpuOnEveryCard => true,
        }
    }
}

impl LockstepArm {
    fn name(self) -> String {
        match self.rejudging {
            RejudgingOfTheArm::NotRejudged => self.compute.name().to_string(),
            RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu => {
                format!("{}_rejudged", self.compute.name())
            }
        }
    }

    fn of_the_name(name: &str) -> Option<LockstepArm> {
        LOCKSTEP_ARMS.into_iter().find(|arm| arm.name() == name)
    }
}

/// 本进程看得到的一张独立显卡：PCI 总线地址（卡的标识，按 wgpu 报的原样）与型号。
#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscreteGpuSeenByThisProcess {
    pci_bus_address: String,
    model: String,
}

/// 本进程看得到的独立显卡，按 PCI 地址次序（产品没点名时填满的次序）：在这个进程的那一份 Vulkan 实例（开卡也用它）上枚举一次，
/// 判法同产品（集显、虚拟显卡、软件 Vulkan、认不出类型的与没有总线地址的不算）。
fn discrete_gpus_seen_by_this_process(
    vulkan_instance: &VulkanInstanceOfThisProcess,
) -> Result<Vec<DiscreteGpuSeenByThisProcess>, String> {
    let adapters = vulkan_instance
        .vulkan_adapters_enumerated_within(LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT)
        .map_err(|cause| format!("枚举适配器没在等的上限之内交回：{cause:?}"))?;
    let mut discrete_gpus: Vec<DiscreteGpuSeenByThisProcess> = adapters
        .iter()
        .map(wgpu::Adapter::get_info)
        .filter(|information| {
            AdapterDeviceKind::of_the_wgpu_device_type(information.device_type)
                .counts_as_a_background_gpu()
                && !information.device_pci_bus_id.is_empty()
        })
        .map(|information| DiscreteGpuSeenByThisProcess {
            pci_bus_address: information.device_pci_bus_id,
            model: information.name,
        })
        .collect();
    discrete_gpus.sort_by(|left, right| left.pci_bus_address.cmp(&right.pci_bus_address));
    Ok(discrete_gpus)
}

/// 这条臂在看得到 `gpu_count` 张独显的任务上跑不跑得了；跑不了的交回 `name=not_run` 那一行的理由（登记第十一节作废第 8 条）。
fn reason_the_arm_is_not_run(compute: ComputeOfTheArm, gpu_count: usize) -> Option<&'static str> {
    match compute {
        ComputeOfTheArm::Cpu => None,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress if gpu_count == 0 => {
            Some("本进程看得到的独立显卡一张都没有")
        }
        ComputeOfTheArm::GpuOnEveryCard if gpu_count == 0 => {
            Some("本进程看得到的独立显卡一张都没有")
        }
        ComputeOfTheArm::GpuOnEveryCard if gpu_count == 1 => {
            Some("本进程只看得到一张独立显卡，全部卡臂与点名一张卡臂相同")
        }
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress | ComputeOfTheArm::GpuOnEveryCard => {
            None
        }
    }
}

/// 这条臂的挂载参数（登记第五节「臂」）：只差 `background_compute` 与 GPU 那几项；每张参与的卡上限 1 GiB。
fn background_compute_settings_of_the_arm(
    compute: ComputeOfTheArm,
    discrete_gpus: &[DiscreteGpuSeenByThisProcess],
) -> BackgroundComputeSettings {
    let memory_limit = GpuMemoryInBytes(LOCAL_MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU);
    match compute {
        ComputeOfTheArm::Cpu => BackgroundComputeSettings::PRODUCT,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress => BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(
                discrete_gpus
                    .iter()
                    .take(1)
                    .map(|gpu| gpu_device_identifier_of_the_pci_bus_address(&gpu.pci_bus_address))
                    .collect(),
            ),
            background_gpu_memory_limit: Some(vec![memory_limit]),
            background_gpu_wait_limit: LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT,
        },
        ComputeOfTheArm::GpuOnEveryCard => BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: None,
            background_gpu_memory_limit: Some(vec![memory_limit; discrete_gpus.len()]),
            background_gpu_wait_limit: LOCAL_GPU_WAIT_LIMIT_BY_DEFAULT,
        },
    }
}

// ---- GPU 多判臂的包装（登记第五节「阳性对照」第 1 条）：每个方法原样转给产品那一份，交回之后把每条确认的在 CPU 上再判一遍 ----

/// 一批里三段各自交进去的条目数（第七节甲类第 3 行：确认 + 不确定 + 交 CPU 判 = 交进去的条目数）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct EntriesHandedIn {
    unit_checksums: u64,
    region_records: u64,
    cluster_segments: u64,
}

/// 再判一段的结果：再判了几条、与 GPU 确认的结果对不上几条。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RejudgeTally {
    entries_rejudged: u64,
    mismatches: u64,
}

impl RejudgeTally {
    fn add(&mut self, later: RejudgeTally) {
        self.entries_rejudged += later.entries_rejudged;
        self.mismatches += later.mismatches;
    }
}

/// 包装在 GC 线程上记下的：多判花掉的线程 CPU（里层计时 E）、再判与对不上的条数、每批交进去的条目数、活槽段每次调用交进去的段数对不对。
#[derive(Debug, Default)]
struct RejudgeLog {
    inner_rejudge_cpu: Duration,
    tally: RejudgeTally,
    clock_read_failures: u64,
    handed_in_since_the_last_batch: EntriesHandedIn,
    handed_in_of_each_batch: Vec<EntriesHandedIn>,
    live_slot_calls_not_handing_in_every_full_segment: u64,
}

type SharedRejudgeLog = Arc<Mutex<RejudgeLog>>;

/// 校验和段的再判：`crc32_castagnoli(单元字节)` 与 `expected_crc32c` 比，与 GPU 确认的结局逐条比。
fn rejudged_unit_checksums(
    units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    decisions: &[GpuEntryDecision<UnitChecksumOutcome>],
) -> RejudgeTally {
    let mut tally = RejudgeTally::default();
    for (unit, decision) in units.iter().zip(decisions) {
        if let GpuEntryDecision::ConfirmedByBothKernels(outcome) = decision {
            let computed_crc32c = crc32_castagnoli(unit.unit_bytes);
            let cpu_outcome = UnitChecksumOutcome {
                computed_crc32c,
                matches_the_expected_crc32c: computed_crc32c == unit.expected_crc32c,
            };
            tally.entries_rejudged += 1;
            if cpu_outcome != *outcome {
                tally.mismatches += 1;
            }
        }
    }
    tally
}

/// 区域记录段的再判：映射条目照这一次交进来的那一份建按身份的索引（每次调用建一次，算在多判的活里），
/// 对每条确认的记录单独调 `scan_region_over_the_identity_index`（产品 GPU 包的 CPU 参照 `cpu_reference_classification` 就是这一调）。
fn rejudged_region_records(
    region: CompactionRegion,
    records: &[RegionRecordToClassifyOnTheGpu<'_>],
    mapping_entries: &[CentralMappingEntry],
    decisions: &[GpuEntryDecision<AllocationInRegion>],
) -> RejudgeTally {
    let mut tally = RejudgeTally::default();
    let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(mapping_entries);
    for (record, decision) in records.iter().zip(decisions) {
        if let GpuEntryDecision::ConfirmedByBothKernels(class) = decision {
            let cpu_class = cpu_reference_classification(
                region,
                record.record,
                &mapping_index,
                record.unit_bytes,
            );
            tally.entries_rejudged += 1;
            if cpu_class != *class {
                tally.mismatches += 1;
            }
        }
    }
    tally
}

/// 活槽段的再判：对全部分配记录调一次 `live_slots_per_cluster_segment_by_the_records`，取每个确认段的那一个数。
fn rejudged_live_slots(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    segments: &[ClusterSegmentEntryKey],
    decisions: &[GpuEntryDecision<u64>],
) -> RejudgeTally {
    let mut tally = RejudgeTally::default();
    let live_slots = live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area);
    let unit_area_start_slot = unit_area.start.slot().0;
    for (segment, decision) in segments.iter().zip(decisions) {
        if let GpuEntryDecision::ConfirmedByBothKernels(gpu_live_slots) = decision {
            let segment_index =
                (segment.start_slot.0 - unit_area_start_slot) / LOCAL_CLUSTER_SEGMENT_SLOTS;
            let cpu_live_slots = live_slots
                .get(usize::try_from(segment_index).expect("段下标"))
                .copied();
            tally.entries_rejudged += 1;
            if cpu_live_slots != Some(*gpu_live_slots) {
                tally.mismatches += 1;
            }
        }
    }
    tally
}

/// GPU 多判臂的那一份实现：里面是产品那一份（`BackgroundGpuComputeOnTheParticipatingGpus`；测试里换成在 CPU 上算的替身）。
struct GpuComputeRejudgedOnTheCpu<Inner> {
    inner: Inner,
    log: SharedRejudgeLog,
}

impl<Inner> GpuComputeRejudgedOnTheCpu<Inner> {
    fn locked_log(&self) -> std::sync::MutexGuard<'_, RejudgeLog> {
        self.log
            .lock()
            .expect("多判记录只在 GC 线程上写，没在别处 panic 过")
    }

    /// 在这条线程的 CPU 钟里量 `rejudge` 用掉的 CPU，记进里层计时 E。
    fn timed_rejudge(&self, rejudge: impl FnOnce() -> RejudgeTally) {
        let (tally, spent) = thread_cpu_spent_in(rejudge);
        let mut log = self.locked_log();
        log.tally.add(tally);
        match spent {
            Some(spent) => log.inner_rejudge_cpu += spent,
            None => log.clock_read_failures += 1,
        }
    }
}

impl<Inner: GpuInventory> GpuInventory for GpuComputeRejudgedOnTheCpu<Inner> {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.inner.devices_with_total_memory()
    }

    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
        self.inner.devices_not_opened_at_the_mount()
    }
}

impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeRejudgedOnTheCpu<Inner> {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        let decisions = self.inner.verify_unit_checksums(units)?;
        self.locked_log()
            .handed_in_since_the_last_batch
            .unit_checksums += u64::try_from(units.len()).expect("条目数");
        self.timed_rejudge(|| rejudged_unit_checksums(units, &decisions));
        Ok(decisions)
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        let decisions = self
            .inner
            .classify_region_records(region, records, mapping_entries)?;
        self.locked_log()
            .handed_in_since_the_last_batch
            .region_records += u64::try_from(records.len()).expect("条目数");
        self.timed_rejudge(|| {
            rejudged_region_records(region, records, mapping_entries, &decisions)
        });
        Ok(decisions)
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        let decisions = self.inner.count_live_slots_per_cluster_segment(
            allocation_records,
            unit_area,
            segments,
        )?;
        {
            let mut log = self.locked_log();
            log.handed_in_since_the_last_batch.cluster_segments +=
                u64::try_from(segments.len()).expect("条目数");
            if u64::try_from(segments.len()).expect("段数")
                != unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS
            {
                log.live_slot_calls_not_handing_in_every_full_segment += 1;
            }
        }
        self.timed_rejudge(|| {
            rejudged_live_slots(allocation_records, unit_area, segments, &decisions)
        });
        Ok(decisions)
    }

    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.inner.use_the_settings_from_the_next_batch(settings);
    }

    fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
        self.inner.failure_under_the_settings_handed_in()
    }

    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        let submissions = self.inner.take_the_submissions_of_the_batch();
        let mut log = self.locked_log();
        let handed_in = std::mem::take(&mut log.handed_in_since_the_last_batch);
        log.handed_in_of_each_batch.push(handed_in);
        submissions
    }
}

// ---- CPU 多判臂（登记第五节「阳性对照」第 2 条）：每次 GC 调用返回之后、窗口之内，对这次调用发出的每一批把三段的 CPU 写法各再跑一遍 ----

/// 一批搬的单元所在的 R：第一个搬走的单元的源落点所在的那一段。
fn region_of_the_batch(
    version: &TransactionOutput,
    unit_area_start_slot: u64,
) -> Option<CompactionRegion> {
    let relocated = version.units_relocated_by_this_publish.first()?;
    let source = relocated.source[0];
    CompactionRegion::of_one_cluster_segment(
        source.device,
        SlotNumber(segment_start_of(source.slot.0, unit_area_start_slot)),
    )
    .ok()
}

/// 三段的 CPU 写法在一批上各跑一遍：R 的区域扫描（映射取 `central_mapping_as_the_compaction_reads_it`，读单元从内存盘取）、
/// 对扫描判成可搬的每个单元算整单元 CRC 与位置项比、对每块盘的分配记录各调一次活槽段。交回再判了几条、对不上几条。
fn rejudged_batch_on_the_cpu(
    version: &TransactionOutput,
    devices: &PoolDevices,
    unit_areas: &[UnitAreaOfOneDevice],
) -> RejudgeTally {
    let mut tally = RejudgeTally::default();
    let Some(unit_area_start) = unit_areas.first().map(|unit_area| unit_area.start.slot().0) else {
        return tally;
    };
    let Some(region) = region_of_the_batch(version, unit_area_start) else {
        return tally;
    };
    let mapping_entries = central_mapping_as_the_compaction_reads_it(version);
    let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(&mapping_entries);
    let read_unit_bytes =
        |device: DeviceIdentity, slot: SlotNumber, length: usize| -> Option<Vec<u8>> {
            let (_identity, memory_device) = devices
                .iter()
                .find(|(identity, _device)| *identity == device)?;
            let mut bytes = vec![0u8; length];
            memory_device
                .read_at(slot.to_device_offset(), &mut bytes)
                .ok()?;
            Some(bytes)
        };
    let scan = scan_region_over_the_identity_index(
        region,
        &version.allocation_records,
        &mapping_index,
        &BTreeSet::new(),
        &BTreeSet::new(),
        read_unit_bytes,
    );
    if let Ok(scan) = scan {
        let expected_crc_by_slot: HashMap<u64, u32> = mapping_entries
            .iter()
            .flat_map(|entry| entry.locations.iter())
            .filter(|location| location.device == region.device())
            .map(|location| (location.slot.0, location.unit_checksum))
            .collect();
        for (placement, _unit) in scan.movable_units() {
            let length = usize::try_from(placement.span * LOCAL_SLOT_BYTES).expect("单元字节");
            let Some(bytes) = read_unit_bytes(region.device(), placement.slot, length) else {
                tally.mismatches += 1;
                continue;
            };
            tally.entries_rejudged += 1;
            if expected_crc_by_slot.get(&placement.slot.0) != Some(&crc32_castagnoli(&bytes)) {
                tally.mismatches += 1;
            }
        }
    } else {
        tally.mismatches += 1;
    }
    for unit_area in unit_areas {
        let device_records: Vec<AllocationRecord> = version
            .allocation_records
            .iter()
            .filter(|record| record.device == unit_area.device)
            .copied()
            .collect();
        let live_slots = live_slots_per_cluster_segment_by_the_records(&device_records, *unit_area);
        tally.entries_rejudged += u64::try_from(live_slots.len()).expect("段数");
    }
    tally
}

// ---- 一次 GC 调用的计时窗口（登记第一节「GC 挂钟」「进程口径」「GC 线程口径」，第五节「计时与计数怎么做」）----

/// 窗口一端的一次读数：单调钟、进程 CPU 钟、GC 线程自己的 CPU 钟、前台线程与主线程各自的 CPU 钟，
/// 与这一端读到的本进程每条线程的 schedstat（线程号 → CPU；拿两端比，认出窗口里退出的线程）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct CallWindowReading {
    wall: Duration,
    process_cpu: Duration,
    gc_thread_cpu: Duration,
    foreground_thread_cpu: Duration,
    main_thread_cpu: Duration,
    cpu_by_task: BTreeMap<u64, Duration>,
}

/// 窗口两端的读数从哪来：产品是本进程的钟，测试换成假的。
trait CallWindowClocks {
    fn read(&self, end: EndOfTheWindow) -> Result<CallWindowReading, ClockReadFailed>;
}

/// 读的是窗口的哪一端：读 `/proc` 那几个系统调用的开销放在窗口外（开窗那一端先读 `/proc` 再读钟，关窗那一端先读钟再读 `/proc`）；
/// 两样钟里挂钟在外、GC 线程的 CPU 钟在里（开窗先读挂钟再读 CPU 钟，关窗先读 CPU 钟再读挂钟），挂钟的窗口罩住 CPU 钟的窗口，
/// 包含自检「挂钟 ≥ GC 线程 CPU」比的才是同一段时间（读 CPU 钟那一次系统调用的开销落在挂钟里，不落在挂钟外）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndOfTheWindow {
    Opening,
    Closing,
}

/// GC 线程上读的真钟：挂钟从 `started` 起算（`CLOCK_MONOTONIC_RAW`）；GC 线程自己的 CPU 读线程 CPU 钟，进程口径是它加上本进程别的线程各自的 schedstat，
/// 前台线程与主线程按它们各自交来的线程号读（登记第一节「进程口径」：窗口里进程 CPU 减前台线程与主线程的增量）。
struct ClocksOfTheGcThread {
    started: StartOnTheMonotonicRawClock,
    /// 读不到 GC 线程自己的线程号时是 `None`：这次跑每个窗口都读不成，整次作废。
    gc_thread_task: Option<u64>,
    foreground_thread_task: u64,
    main_thread_task: u64,
}

impl CallWindowClocks for ClocksOfTheGcThread {
    fn read(&self, end: EndOfTheWindow) -> Result<CallWindowReading, ClockReadFailed> {
        let gc_thread_task = self.gc_thread_task.ok_or(ClockReadFailed {
            reading: "GC 线程的线程号",
        })?;
        let (wall, gc_thread_cpu, cpu_by_task) = match end {
            EndOfTheWindow::Opening => {
                let cpu_by_task = cpu_of_every_task_of_this_process()?;
                let wall = self.started.elapsed();
                (wall, host_cpu_time_of_this_thread(), cpu_by_task)
            }
            EndOfTheWindow::Closing => {
                let gc_thread_cpu = host_cpu_time_of_this_thread();
                let wall = self.started.elapsed();
                (wall, gc_thread_cpu, cpu_of_every_task_of_this_process()?)
            }
        };
        Ok(CallWindowReading {
            wall,
            process_cpu: gc_thread_cpu
                + cpu_by_task
                    .iter()
                    .filter(|(task, _cpu)| **task != gc_thread_task)
                    .map(|(_task, cpu)| *cpu)
                    .sum::<Duration>(),
            gc_thread_cpu,
            foreground_thread_cpu: *cpu_by_task.get(&self.foreground_thread_task).ok_or(
                ClockReadFailed {
                    reading: "前台线程的 schedstat",
                },
            )?,
            main_thread_cpu: *cpu_by_task
                .get(&self.main_thread_task)
                .ok_or(ClockReadFailed {
                    reading: "主线程的 schedstat",
                })?,
            cpu_by_task,
        })
    }
}

/// 窗口两端各读一次的每条线程的 schedstat 里，有没有开窗时在、关窗时不在的线程（窗口里退出了），或关窗时比开窗时少的线程
/// （线程号让窗口里新起的线程重用了）。有就是进程口径读不全：退出的线程到开窗为止的 CPU 在关窗那一端读不到，两端之和相减少掉它，
/// 少得多了进程口径比 GC 线程口径还小（登记第十二节修订第 1 条：这一窗照第十一节作废第 1 条作废）。
fn any_task_read_at_the_opening_is_missing_or_behind_at_the_closing(
    opening: &BTreeMap<u64, Duration>,
    closing: &BTreeMap<u64, Duration>,
) -> bool {
    opening.iter().any(|(task, opening_cpu)| {
        closing
            .get(task)
            .is_none_or(|closing_cpu| closing_cpu < opening_cpu)
    })
}

/// 一次 GC 调用的窗口：挂钟、进程口径（进程 CPU 减前台线程与主线程的增量）、GC 线程口径与两条减掉的增量。
/// 任一增量是负的、或窗口里有线程退出的为 `None`（作废第 1 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CallWindow {
    wall: Duration,
    process_cpu: Duration,
    gc_thread_cpu: Duration,
    foreground_thread_cpu: Duration,
    main_thread_cpu: Duration,
}

/// 两端读数之差（第九节变异「进程口径只读 GC 线程钟」改的是 `process_cpu` 那一算）。
fn call_window_between(before: CallWindowReading, after: CallWindowReading) -> Option<CallWindow> {
    if any_task_read_at_the_opening_is_missing_or_behind_at_the_closing(
        &before.cpu_by_task,
        &after.cpu_by_task,
    ) {
        return None;
    }
    let foreground_thread_cpu = after
        .foreground_thread_cpu
        .checked_sub(before.foreground_thread_cpu)?;
    let main_thread_cpu = after.main_thread_cpu.checked_sub(before.main_thread_cpu)?;
    let process_cpu = after
        .process_cpu
        .checked_sub(before.process_cpu)?
        .checked_sub(foreground_thread_cpu)?
        .checked_sub(main_thread_cpu)?;
    Some(CallWindow {
        wall: after.wall.checked_sub(before.wall)?,
        process_cpu,
        gc_thread_cpu: after.gc_thread_cpu.checked_sub(before.gc_thread_cpu)?,
        foreground_thread_cpu,
        main_thread_cpu,
    })
}

/// 窗口量出的一次调用：调用交回的、窗口（钟读错、增量为负、里层计时读不到都是 `None`）、窗口之内多判那一段的里层计时与再判结果。
struct MeasuredCall<Outcome> {
    outcome: Outcome,
    window: Option<CallWindow>,
    inner_rejudge_cpu: Duration,
    rejudge_tally: RejudgeTally,
}

/// 量一次 GC 调用：窗口一端读钟 → 调用 → 窗口之内的多判（CPU 多判臂；别的臂什么都不做）→ 窗口另一端读钟
/// （第九节变异「CPU 多判放到计时窗口外」改的是这里的次序）。
fn measure_one_gc_call<State, Outcome>(
    clocks: &impl CallWindowClocks,
    state: &mut State,
    call: impl FnOnce(&mut State) -> Outcome,
    rejudge_inside_the_window: impl FnOnce(&mut State, &Outcome) -> (Option<Duration>, RejudgeTally),
) -> MeasuredCall<Outcome> {
    let before = clocks.read(EndOfTheWindow::Opening);
    let outcome = call(state);
    let (inner_rejudge_cpu, rejudge_tally) = rejudge_inside_the_window(state, &outcome);
    let after = clocks.read(EndOfTheWindow::Closing);
    let window = match (before, after, inner_rejudge_cpu) {
        (Ok(before), Ok(after), Some(_inner)) => call_window_between(before, after),
        (Err(_), _, _) | (_, Err(_), _) | (_, _, None) => None,
    };
    MeasuredCall {
        outcome,
        window,
        inner_rejudge_cpu: inner_rejudge_cpu.unwrap_or(Duration::ZERO),
        rejudge_tally,
    }
}

/// 交替跑里调用方的假钟与前台：每次调用前进 1 毫秒（定时兜底每次都到点），前台恒答没有 fsync 在等（登记第五节「交替跑」）。
struct CallerOfTheLockstepRun {
    reading: Cell<Duration>,
}

impl CallerOfTheLockstepRun {
    fn starting_at_zero() -> CallerOfTheLockstepRun {
        CallerOfTheLockstepRun {
            reading: Cell::new(Duration::ZERO),
        }
    }

    /// 下一次调用之前把钟往前拨一步（第九节变异「交替跑的假钟不前进」改的是这里）。
    fn advance_before_the_next_call(&self) {
        self.reading
            .set(self.reading.get() + LOCKSTEP_FAKE_CLOCK_STEP_PER_CALL);
    }
}

impl CallerOfTheBackgroundCompaction for CallerOfTheLockstepRun {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(self.reading.get())
    }

    fn foreground_fsync(&self) -> ForegroundFsync {
        ForegroundFsync::NoneWaiting
    }
}

/// 后台入口的节奏：定时兜底每 1 毫秒、写带宽不限（登记第五节「量挂载」）。
fn pacing_of_the_measuring_mount() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::EveryMilliseconds(
            NonZeroU64::new(TIMER_FALLBACK_MILLISECONDS).expect("1 毫秒不是 0"),
        ),
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

/// 量挂载装上的整理只供测试的开关：批大小 `batch_units`，起跑强制开着，其余照产品（同产品 GPU 包测试共用模块的写法）。
fn compaction_switches_of_the_measuring_mount(batch_units: usize) -> CompactionSwitches {
    let CompactionSwitches {
        batch_size: _,
        forced_region,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(batch_units).expect("批大小至少 1"),
        ),
        forced_region,
        run_forcing: CompactionRunForcing::ForcedToStartForTests,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    }
}

// ---- 一次跑里每次 GC 调用记下的，与第七节甲类、乙类锚点 ----

/// 一次 GC 调用记下的（窗口外读，登记第五节「计时与计数」第 2 条）。
struct GcCallRecord {
    /// 这次调用是第几次用户改动之后的（交替跑）或 0（并发跑不分组）。
    after_the_user_change: u64,
    window: Option<CallWindow>,
    inner_rejudge_cpu: Duration,
    rejudge_tally: RejudgeTally,
    started: bool,
    compaction_publishes: u64,
    batches: u64,
    units_relocated_of_each_batch: Vec<u64>,
    stop_name: String,
    compute_records: Vec<BackgroundComputeOfOneBatch>,
    has_the_alarm: bool,
    /// 每块盘的全空聚簇段数，这次调用开始前与返回后各一个读数（按盘的次序；登记第十二节修订第 5 条，窗口外读）。
    empty_segments_before_the_call: Vec<u64>,
    empty_segments_after_the_call: Vec<u64>,
}

/// 每块盘此刻的全空聚簇段数，按盘的次序（第八节轨迹，登记第十二节修订第 5 条）。
fn empty_segments_of_every_device(session: &MountedSession) -> Vec<u64> {
    session
        .allocator
        .devices
        .iter()
        .map(|device| device.empty_segments())
        .collect()
}

/// 从后台入口交回的那一份读出记账（窗口外）。每块盘全空段数的前后读数由调用方另填。
fn record_of_the_call(
    call: &BackgroundCompactionCall,
    after_the_user_change: u64,
    measured_window: Option<CallWindow>,
    inner_rejudge_cpu: Duration,
    rejudge_tally: RejudgeTally,
) -> GcCallRecord {
    let has_the_alarm = call
        .alarm_of_the_compaction_stopped_after_a_compute_error()
        .is_some();
    match call {
        BackgroundCompactionCall::NotStarted { reason, .. } => GcCallRecord {
            after_the_user_change,
            window: measured_window,
            inner_rejudge_cpu,
            rejudge_tally,
            started: false,
            compaction_publishes: 0,
            batches: 0,
            units_relocated_of_each_batch: Vec::new(),
            stop_name: format!("not_started_{reason:?}"),
            compute_records: Vec::new(),
            has_the_alarm,
            empty_segments_before_the_call: Vec::new(),
            empty_segments_after_the_call: Vec::new(),
        },
        BackgroundCompactionCall::Ran { run, .. } => GcCallRecord {
            after_the_user_change,
            window: measured_window,
            inner_rejudge_cpu,
            rejudge_tally,
            started: true,
            compaction_publishes: u64::try_from(run.versions_published.len()).expect("版数"),
            batches: run.batches_published,
            units_relocated_of_each_batch: run
                .versions_published
                .iter()
                .filter(|version| !version.units_relocated_by_this_publish.is_empty())
                .map(|version| {
                    u64::try_from(version.units_relocated_by_this_publish.len()).expect("单元数")
                })
                .collect(),
            stop_name: match &run.outcome {
                CompactionRunOutcome::Stopped(stop) => stop.name().to_string(),
                CompactionRunOutcome::Failed(failure) => format!("failed_{failure:?}"),
            },
            compute_records: run.background_compute_of_each_batch.clone(),
            has_the_alarm,
            empty_segments_before_the_call: Vec::new(),
            empty_segments_after_the_call: Vec::new(),
        },
    }
}

/// 这次调用起跑而一次整理发布都没发，或没起跑：GC 线程停下、通知前台发下一次（登记第五节「交替跑」）。
fn the_call_published_nothing(record: &GcCallRecord) -> bool {
    record.compaction_publishes == 0
}

/// 第七节甲类第 1 行（登记第十二节修订第 2 条）：每一项记账（签发、搬一批、删意图、停在判据上的那一次都是一项）的 `path_taken`
/// 照这条臂的设置；条数不少于这次跑的批数 N。条数与 N 两个数逐次报（`name=run` 的 `compute_records=` 与 `batches=`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PathAnchor {
    compute_records: u64,
    compute_records_on_the_path_of_the_settings: u64,
    batches: u64,
}

impl PathAnchor {
    /// 条数少于 N 照第十条「条款与实现对不上」判（停机条款第 4 条）。
    fn holds(self) -> bool {
        self.compute_records == self.compute_records_on_the_path_of_the_settings
            && self.compute_records >= self.batches
    }
}

fn path_anchor(records: &[GcCallRecord], compute: ComputeOfTheArm) -> PathAnchor {
    let compute_records: Vec<&BackgroundComputeOfOneBatch> = records
        .iter()
        .flat_map(|record| record.compute_records.iter())
        .collect();
    let on_the_path = compute_records
        .iter()
        .filter(
            |compute_record| match (&compute_record.path_taken, compute.uses_the_gpu()) {
                (
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings,
                    ),
                    false,
                )
                | (BackgroundComputePathTaken::Gpu(_), true) => true,
                (BackgroundComputePathTaken::Cpu(_), _)
                | (BackgroundComputePathTaken::Gpu(_), false) => false,
            },
        )
        .count();
    PathAnchor {
        compute_records: u64::try_from(compute_records.len()).expect("条数"),
        compute_records_on_the_path_of_the_settings: u64::try_from(on_the_path).expect("条数"),
        batches: records.iter().map(|record| record.batches).sum(),
    }
}

/// 第七节甲类第 2、3 行与第十条第 2、3 行：分到活的卡、切没切片、两份核比对的条数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GpuWorkAnchors {
    gpu_records: u64,
    segments_given_to_a_card_other_than_the_lowest: u64,
    segments_without_a_submission: u64,
    recuts_after_running_out_of_memory: u64,
    segments_with_pieces_other_than_submissions: u64,
    uncertain_entries: u64,
    entries_judged_by_the_cpu_reference: u64,
    batches_abandoned_after_a_compute_error: u64,
}

impl GpuWorkAnchors {
    fn hold(self) -> bool {
        self.segments_given_to_a_card_other_than_the_lowest == 0
            && self.recuts_after_running_out_of_memory == 0
            && self.segments_with_pieces_other_than_submissions == 0
            && self.uncertain_entries == 0
            && self.entries_judged_by_the_cpu_reference == 0
    }
}

/// 每批三段：交了提交的段分到活的卡恰好是 `lowest_card` 那一张，没交提交的段一张卡都不列；重切 0；片数等于提交数；不确定与交 CPU 判都是 0
/// （第九节变异「甲类锚点『分到活的卡』改成不核」改的是这里）。
fn gpu_work_anchors(records: &[GcCallRecord], lowest_card: &GpuDeviceIdentifier) -> GpuWorkAnchors {
    let mut anchors = GpuWorkAnchors::default();
    for compute_record in records
        .iter()
        .flat_map(|record| record.compute_records.iter())
    {
        if compute_record
            .compute_error_that_abandoned_the_batch
            .is_some()
        {
            anchors.batches_abandoned_after_a_compute_error += 1;
        }
        let BackgroundComputePathTaken::Gpu(work) = &compute_record.path_taken else {
            continue;
        };
        anchors.gpu_records += 1;
        let entries = work.entries_of_the_three_segments();
        anchors.uncertain_entries += entries.uncertain_this_round;
        anchors.entries_judged_by_the_cpu_reference += entries.judged_by_the_cpu_reference;
        let submissions = &work.submissions_on_the_gpus;
        for segment in [
            &submissions.unit_checksum_verification,
            &submissions.region_record_classification,
            &submissions.live_slots_per_cluster_segment,
        ] {
            note_the_segment_submissions(&mut anchors, segment, lowest_card);
        }
    }
    anchors
}

fn note_the_segment_submissions(
    anchors: &mut GpuWorkAnchors,
    segment: &GpuSubmissionsOfOneSegment,
    lowest_card: &GpuDeviceIdentifier,
) {
    if segment.submissions_completed == 0 {
        anchors.segments_without_a_submission += 1;
        if !segment.devices_given_work.is_empty() {
            anchors.segments_given_to_a_card_other_than_the_lowest += 1;
        }
    } else if segment.devices_given_work.as_slice() != std::slice::from_ref(lowest_card) {
        anchors.segments_given_to_a_card_other_than_the_lowest += 1;
    }
    anchors.recuts_after_running_out_of_memory += segment.recuts_after_running_out_of_memory;
    if segment.pieces_submitted != segment.submissions_completed {
        anchors.segments_with_pieces_other_than_submissions += 1;
    }
}

/// 第七节甲类第 3 行（多判臂上逐批核）：每批三段各自 确认 + 不确定 + 交 CPU 判 = 交进去的条目数。包装每批记下的交进去的条数与 GPU 那一臂的
/// 记账按先后一一对应；条数对不上（包装记下的批数与记账里走 GPU 的批数不等）也算不成立。交回不成立的批数。
fn batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(
    records: &[GcCallRecord],
    handed_in_of_each_batch: &[EntriesHandedIn],
) -> u64 {
    let gpu_works: Vec<_> = records
        .iter()
        .flat_map(|record| record.compute_records.iter())
        .filter_map(|compute_record| match &compute_record.path_taken {
            BackgroundComputePathTaken::Gpu(work) => Some(work),
            BackgroundComputePathTaken::Cpu(_) => None,
        })
        .collect();
    let count_mismatch =
        u64::try_from(gpu_works.len().abs_diff(handed_in_of_each_batch.len())).expect("批数");
    let mismatched_batches = gpu_works
        .iter()
        .zip(handed_in_of_each_batch)
        .filter(|(work, handed_in)| {
            let added = |counts: singlefs_core::background_compute::GpuEntryCounts| {
                counts.confirmed_by_both_kernels
                    + counts.uncertain_this_round
                    + counts.judged_by_the_cpu_reference
            };
            added(work.unit_checksum_verification) != handed_in.unit_checksums
                || added(work.region_record_classification) != handed_in.region_records
                || added(work.live_slots_per_cluster_segment) != handed_in.cluster_segments
        })
        .count();
    count_mismatch + u64::try_from(mismatched_batches).expect("批数")
}

/// 第七节乙类第 2 行（不多判的 GPU 臂上只核得到相加形式）：每批活槽段的条目数是那块盘整段数的整数倍。交回不成立的批数。
fn batches_whose_live_slot_entries_are_not_whole_device_passes(
    records: &[GcCallRecord],
    full_segments_of_a_device: u64,
) -> u64 {
    let not_whole = records
        .iter()
        .flat_map(|record| record.compute_records.iter())
        .filter_map(|compute_record| match &compute_record.path_taken {
            BackgroundComputePathTaken::Gpu(work) => Some(work.live_slots_per_cluster_segment),
            BackgroundComputePathTaken::Cpu(_) => None,
        })
        .filter(|counts| {
            let entries = counts.confirmed_by_both_kernels
                + counts.uncertain_this_round
                + counts.judged_by_the_cpu_reference;
            entries % full_segments_of_a_device != 0
        })
        .count();
    u64::try_from(not_whole).expect("批数")
}

/// 第八节「轨迹」三样（被谓词消费的量）：峰值、读数大于 0 的次数、期末值（最后一个读数）。没有读数时三个都是 0。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TrajectoryOfTheReadings {
    peak: u64,
    positive_readings: u64,
    last: u64,
}

fn trajectory_of_the_readings(readings: &[u64]) -> TrajectoryOfTheReadings {
    TrajectoryOfTheReadings {
        peak: readings.iter().copied().max().unwrap_or(0),
        positive_readings: u64::try_from(readings.iter().filter(|reading| **reading > 0).count())
            .expect("读数个数"),
        last: readings.last().copied().unwrap_or(0),
    }
}

/// 第八节轨迹「每块盘的全空聚簇段数」的一块盘（登记第十二节修订第 5 条：每次 GC 调用开始前、返回后各一个读数，不按意图对齐）：
/// 读数的峰值、返回后比开始前多的调用数、期末值（最后一次调用返回后那一个）。没有读数时三个都是 0。只描述，不进任何判据。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct EmptySegmentsTrajectoryOfADevice {
    peak: u64,
    calls_gaining_empty_segments: u64,
    last: u64,
}

fn empty_segments_trajectory_of_each_device(
    records: &[GcCallRecord],
) -> Vec<EmptySegmentsTrajectoryOfADevice> {
    let devices = records
        .iter()
        .map(|record| record.empty_segments_after_the_call.len())
        .max()
        .unwrap_or(0);
    (0..devices)
        .map(|device| {
            let mut trajectory = EmptySegmentsTrajectoryOfADevice::default();
            for record in records {
                let (Some(before), Some(after)) = (
                    record.empty_segments_before_the_call.get(device),
                    record.empty_segments_after_the_call.get(device),
                ) else {
                    continue;
                };
                trajectory.peak = trajectory.peak.max(*before).max(*after);
                if after > before {
                    trajectory.calls_gaining_empty_segments += 1;
                }
                trajectory.last = *after;
            }
            trajectory
        })
        .collect()
}

fn empty_segments_fields(trajectories: &[EmptySegmentsTrajectoryOfADevice]) -> String {
    let joined = |value: fn(&EmptySegmentsTrajectoryOfADevice) -> u64| {
        field_value(
            &trajectories
                .iter()
                .map(|trajectory| value(trajectory).to_string())
                .collect::<Vec<String>>()
                .join(","),
        )
    };
    format!(
        "empty_segments_peak={} empty_segments_calls_gaining={} empty_segments_last={}",
        joined(|trajectory| trajectory.peak),
        joined(|trajectory| trajectory.calls_gaining_empty_segments),
        joined(|trajectory| trajectory.last),
    )
}

fn trajectory_fields(name: &str, trajectory: TrajectoryOfTheReadings) -> String {
    format!(
        "{name}_peak={} {name}_positive_readings={} {name}_last={}",
        trajectory.peak, trajectory.positive_readings, trajectory.last
    )
}

/// 第八节轨迹「上一次用户改动给的预算还剩几次」：每次 GC 调用前后各一个读数，照核心层的式子（`B_gc` 减这次用户改动之后到此刻的整理发布数，
/// 不到 0 记 0）由记账推出，不是从核心层读的。
fn budget_left_before_and_after_each_call(records: &[GcCallRecord]) -> Vec<u64> {
    let mut readings: Vec<u64> = Vec::new();
    let mut publishes_since_the_user_change: BTreeMap<u64, u64> = BTreeMap::new();
    for record in records {
        let publishes = publishes_since_the_user_change
            .entry(record.after_the_user_change)
            .or_insert(0);
        readings.push(LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE.saturating_sub(*publishes));
        *publishes += record.compaction_publishes;
        readings.push(LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE.saturating_sub(*publishes));
    }
    readings
}

/// 第八节轨迹「每批三段的不确定条目数」：每一项走 GPU 的记账一个读数（健康的卡上预期全 0）。
fn uncertain_entries_of_each_gpu_record(records: &[GcCallRecord]) -> Vec<u64> {
    records
        .iter()
        .flat_map(|record| record.compute_records.iter())
        .filter_map(|compute_record| match &compute_record.path_taken {
            BackgroundComputePathTaken::Gpu(work) => {
                Some(work.entries_of_the_three_segments().uncertain_this_round)
            }
            BackgroundComputePathTaken::Cpu(_) => None,
        })
        .collect()
}

/// 第七节甲类第 4 行：相邻两次用户改动之间的整理发布次数的最大值（≤ B_gc = 20）。
fn most_compaction_publishes_between_two_user_changes(records: &[GcCallRecord]) -> u64 {
    let mut publishes_after_each_change: BTreeMap<u64, u64> = BTreeMap::new();
    for record in records {
        *publishes_after_each_change
            .entry(record.after_the_user_change)
            .or_insert(0) += record.compaction_publishes;
    }
    publishes_after_each_change
        .values()
        .copied()
        .max()
        .unwrap_or(0)
}

/// 量 W：这次跑每一次 GC 调用的挂钟相加，起不起跑的都算（第九节变异「只计发了批的 GC 调用」改的是这里）。
fn gc_wall_clock_of_the_run(windows: &[CallWindow], records: &[GcCallRecord]) -> Duration {
    windows
        .iter()
        .zip(records)
        .map(|(window, _record)| window.wall)
        .sum()
}

/// 第五节包含自检（作废第 2 条）：每次调用挂钟 ≥ GC 线程 CPU；进程口径 ≥ GC 线程口径。交回不成立的调用数。
fn calls_failing_the_inclusion_checks(windows: &[CallWindow]) -> u64 {
    let failing = windows
        .iter()
        .filter(|window| {
            window.wall < window.gc_thread_cpu || window.process_cpu < window.gc_thread_cpu
        })
        .count();
    u64::try_from(failing).expect("调用数")
}

/// CPU 臂（与 CPU 多判臂）每次跑 GC 线程 CPU ÷ GC 挂钟 ≥ 0.9（第七节乙类）。
fn cpu_arm_thread_cpu_covers_the_wall(gc_thread_cpu: Duration, gc_wall: Duration) -> bool {
    gc_thread_cpu.as_secs_f64()
        >= CPU_ARM_THREAD_CPU_SHARE_OF_THE_WALL_AT_LEAST * gc_wall.as_secs_f64()
}

// ---- 统计与判法（登记第六节）：纯函数，同包测试喂合成输入 ----

/// 纳秒数换成浮点算统计：十亿量级以内的整数在 f64 里是精确的（一次跑的纳秒数远小于 2⁵³）。
#[allow(
    clippy::cast_precision_loss,
    reason = "统计口径的纳秒数远小于 2^53，f64 里精确"
)]
fn nanoseconds_as_float(duration: Duration) -> f64 {
    duration.as_nanos() as f64
}

#[allow(clippy::cast_precision_loss, reason = "计数远小于 2^53，f64 里精确")]
fn count_as_float(count: u64) -> f64 {
    count as f64
}

fn median_of(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    match sorted.len() {
        0 => f64::NAN,
        length if length % 2 == 1 => sorted[length / 2],
        length => (sorted[length / 2 - 1] + sorted[length / 2]) / 2.0,
    }
}

fn minimum_of(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::INFINITY, f64::min)
}

fn maximum_of(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn median_absolute_deviation_of(values: &[f64]) -> f64 {
    let median = median_of(values);
    let deviations: Vec<f64> = values.iter().map(|value| (value - median).abs()).collect();
    median_of(&deviations)
}

/// 中位、最小、最大、中位绝对偏差（登记第一节「离散度」）。
fn spread_fields(name: &str, values: &[f64]) -> String {
    format!(
        "{name}_median={:.0} {name}_minimum={:.0} {name}_maximum={:.0} {name}_median_absolute_deviation={:.0}",
        median_of(values),
        minimum_of(values),
        maximum_of(values),
        median_absolute_deviation_of(values)
    )
}

/// 成对差的正负（登记第一节「离散度」）：都 > 0 记「正」，都 < 0 记「负」，有正有负（或有 0）记「不稳定」。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SignOfThePairedDifferences {
    EveryOnePositive,
    EveryOneNegative,
    Unstable,
}

impl SignOfThePairedDifferences {
    const fn name(self) -> &'static str {
        match self {
            SignOfThePairedDifferences::EveryOnePositive => "positive",
            SignOfThePairedDifferences::EveryOneNegative => "negative",
            SignOfThePairedDifferences::Unstable => "unstable",
        }
    }
}

/// 成对差的正负，门槛 `threshold`（判据用 0；第八节判别力自证把它挪到两个取样点的中位之间）。
fn sign_of_the_paired_differences(
    differences: &[f64],
    threshold: f64,
) -> SignOfThePairedDifferences {
    if !differences.is_empty() && differences.iter().all(|difference| *difference > threshold) {
        SignOfThePairedDifferences::EveryOnePositive
    } else if !differences.is_empty()
        && differences.iter().all(|difference| *difference < threshold)
    {
        SignOfThePairedDifferences::EveryOneNegative
    } else {
        SignOfThePairedDifferences::Unstable
    }
}

/// 同一档同一次重复成对相减：基线那一臂减被比那一臂（量 S 是 CPU 臂减 GPU 臂；第九节变异「成对差方向反了」改的是这里）。
fn paired_differences(baseline: &[f64], compared: &[f64]) -> Vec<f64> {
    baseline
        .iter()
        .zip(compared)
        .map(|(baseline_value, compared_value)| baseline_value - compared_value)
        .collect()
}

/// 量 T 相邻两档的一步（登记第六节量 T）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StepBetweenTwoTiers {
    Rises,
    Falls,
    Overlaps,
}

fn step_between_two_tiers(smaller_tier: &[f64], larger_tier: &[f64]) -> StepBetweenTwoTiers {
    if minimum_of(larger_tier) > maximum_of(smaller_tier) {
        StepBetweenTwoTiers::Rises
    } else if maximum_of(larger_tier) < minimum_of(smaller_tier) {
        StepBetweenTwoTiers::Falls
    } else {
        StepBetweenTwoTiers::Overlaps
    }
}

/// 量 T 的分类。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrendOverTheTiers {
    Rising,
    Flat,
    Falling,
    CannotTell,
}

impl TrendOverTheTiers {
    const fn name(self) -> &'static str {
        match self {
            TrendOverTheTiers::Rising => "rising",
            TrendOverTheTiers::Flat => "flat",
            TrendOverTheTiers::Falling => "falling",
            TrendOverTheTiers::CannotTell => "cannot_tell",
        }
    }
}

/// 三档（按单元区从小到大）各自 S 的每批值：两步都「升」判「涨」，都「落」判「降」；两步都「交叠」且顶档与最小档中位之差的绝对值
/// ≤ 两者绝对值大者的 `FLAT_TREND_WIDTH` 判「平」；别的判「说不出」（第九节变异「平的宽度 10% 改成 50%」改的是那个常量）。
fn trend_over_the_tiers(saving_per_batch_by_tier: &[Vec<f64>; 3]) -> TrendOverTheTiers {
    let [smallest, middle, top] = saving_per_batch_by_tier;
    let lower_step = step_between_two_tiers(smallest, middle);
    let upper_step = step_between_two_tiers(middle, top);
    match (lower_step, upper_step) {
        (StepBetweenTwoTiers::Rises, StepBetweenTwoTiers::Rises) => TrendOverTheTiers::Rising,
        (StepBetweenTwoTiers::Falls, StepBetweenTwoTiers::Falls) => TrendOverTheTiers::Falling,
        (StepBetweenTwoTiers::Overlaps, StepBetweenTwoTiers::Overlaps) => {
            let smallest_median = median_of(smallest);
            let top_median = median_of(top);
            let width = FLAT_TREND_WIDTH * smallest_median.abs().max(top_median.abs());
            if (top_median - smallest_median).abs() <= width {
                TrendOverTheTiers::Flat
            } else {
                TrendOverTheTiers::CannotTell
            }
        }
        (
            StepBetweenTwoTiers::Rises | StepBetweenTwoTiers::Falls | StepBetweenTwoTiers::Overlaps,
            _,
        ) => TrendOverTheTiers::CannotTell,
    }
}

/// 量 P 一条臂的判定（登记第五节「阳性对照」判法、第六节量 P）：各项分开报，不合成一个真假。
#[derive(Clone, Copy, Debug, PartialEq)]
struct PositiveControlOfAnArm {
    every_delta_positive: bool,
    median_delta: f64,
    median_inner_rejudge_cpu: f64,
    median_delta_within_the_band_of_the_inner_timing: bool,
    /// 两条 GPU 臂：5 对里 CPU 臂减 GPU 多判臂都比 CPU 臂减 GPU 臂少；CPU 臂没有这一项。
    saving_with_rejudging_below_the_saving_in_every_pair: Option<bool>,
}

impl PositiveControlOfAnArm {
    fn holds(self) -> bool {
        self.every_delta_positive
            && self.median_delta_within_the_band_of_the_inner_timing
            && self.saving_with_rejudging_below_the_saving_in_every_pair != Some(false)
    }
}

/// Δ = 多判臂 C − 不多判臂 C（成对）；E = 多判臂的里层计时；GPU 臂另给 CPU 臂的 C 算 S 多判与 S。
fn positive_control_of_an_arm(
    process_cpu_not_rejudged: &[f64],
    process_cpu_rejudged: &[f64],
    inner_rejudge_cpu: &[f64],
    process_cpu_of_the_cpu_arm: Option<&[f64]>,
) -> PositiveControlOfAnArm {
    let deltas = paired_differences(process_cpu_rejudged, process_cpu_not_rejudged);
    let median_delta = median_of(&deltas);
    let median_inner_rejudge_cpu = median_of(inner_rejudge_cpu);
    let saving_with_rejudging_below_the_saving_in_every_pair =
        process_cpu_of_the_cpu_arm.map(|cpu_arm| {
            let savings = paired_differences(cpu_arm, process_cpu_not_rejudged);
            let savings_with_rejudging = paired_differences(cpu_arm, process_cpu_rejudged);
            !savings.is_empty()
                && savings
                    .iter()
                    .zip(&savings_with_rejudging)
                    .all(|(saving, saving_with_rejudging)| saving_with_rejudging < saving)
        });
    PositiveControlOfAnArm {
        every_delta_positive: sign_of_the_paired_differences(&deltas, 0.0)
            == SignOfThePairedDifferences::EveryOnePositive,
        median_delta,
        median_inner_rejudge_cpu,
        median_delta_within_the_band_of_the_inner_timing: median_delta
            >= POSITIVE_CONTROL_RATIO_LOWER * median_inner_rejudge_cpu
            && median_delta <= POSITIVE_CONTROL_RATIO_UPPER * median_inner_rejudge_cpu,
        saving_with_rejudging_below_the_saving_in_every_pair,
    }
}

// ---- 结果行：写、读回（续跑） ----

/// 结果行里的值不带空白：空白换成下划线。
fn field_value(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_whitespace() {
                '_'
            } else {
                character
            }
        })
        .collect()
}

/// 产物里的一条结果行：种类（`name=` 的值）与其余的键值。
type KindAndFieldsOfAResultLine = (String, BTreeMap<String, String>);

/// 一行结果拆成种类与键值；不是本装置的结果行交回 `None`。
fn fields_of_a_result_line(line: &str) -> Option<KindAndFieldsOfAResultLine> {
    let mut words = line.split_whitespace();
    if words.next() != Some(RESULT_LINE_PREFIX) {
        return None;
    }
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    for word in words {
        let (key, value) = word.split_once('=')?;
        fields.insert(key.to_string(), value.to_string());
    }
    let name = fields.remove("name")?;
    Some((name, fields))
}

/// 产物：标准输出与 `--product` 那份文件上同样的结果行；文件只追加。
struct ProductWriter {
    file: std::fs::File,
    emitted: u64,
}

impl ProductWriter {
    fn appending_to(path: &Path) -> Result<ProductWriter, String> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("建产物目录 {}：{error}", parent.display()))?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| format!("打开产物 {}：{error}", path.display()))?;
        Ok(ProductWriter { file, emitted: 0 })
    }

    fn emit(&mut self, name: &str, fields: &str) {
        let line = if fields.is_empty() {
            format!("{RESULT_LINE_PREFIX} name={name}")
        } else {
            format!("{RESULT_LINE_PREFIX} name={name} {fields}")
        };
        println!("{line}");
        if let Err(error) = writeln!(self.file, "{line}").and_then(|()| self.file.flush()) {
            eprintln!("写产物失败：{error}");
            std::process::exit(EXIT_RUN_CONDITION_NOT_MET);
        }
        self.emitted += 1;
    }

    fn finish(mut self) {
        let emitted = self.emitted + 1;
        self.emit("done", &format!("emitted={emitted}"));
    }
}

/// 读回产物里已有的结果行（续跑）；文件不在交回空。
fn result_lines_already_in_the_product(
    path: &Path,
) -> Result<Vec<KindAndFieldsOfAResultLine>, String> {
    match std::fs::File::open(path) {
        Ok(file) => std::io::BufReader::new(file)
            .lines()
            .map(|line| line.map_err(|error| format!("读产物 {}：{error}", path.display())))
            .filter_map(|line| match line {
                Ok(line) => fields_of_a_result_line(&line).map(Ok),
                Err(error) => Some(Err(error)),
            })
            .collect(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("打开产物 {}：{error}", path.display())),
    }
}

const EXIT_SUCCESS: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_STOPPED: i32 = 3;
const EXIT_POSITIVE_CONTROL_FAILED: i32 = 4;
const EXIT_NO_ARM_RAN: i32 = 77;
const EXIT_RUN_CONDITION_NOT_MET: i32 = 78;

// ---- 开跑之前的三样（停机条款第 1、2 条，开跑自检）----

/// 产品 GPU 包与核心层 `src/` 下每个文件（路径相对仓根，按路径排）与它的 sha256。
fn source_files_and_digests() -> Result<Vec<(String, String)>, String> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut directories: Vec<PathBuf> = vec![
        PathBuf::from("crates/singlefs-background-gpu/src"),
        PathBuf::from("crates/singlefs-core/src"),
    ];
    while let Some(directory) = directories.pop() {
        let entries = std::fs::read_dir(&directory)
            .map_err(|error| format!("读 {}（装置要从仓根起）：{error}", directory.display()))?;
        for entry in entries {
            let path = entry
                .map_err(|error| format!("读 {} 的一项：{error}", directory.display()))?
                .path();
            if path.is_dir() {
                directories.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let bytes =
                std::fs::read(&path).map_err(|error| format!("读 {}：{error}", path.display()))?;
            let mut digest = Sha256Digest::new();
            digest.feed(&bytes);
            Ok((path.display().to_string(), digest.finish_as_hex()))
        })
        .collect()
}

/// 每个文件的摘要连同路径按次序喂成一个合起来的摘要（`name=crates_version` 的 `combined_sha256`）。
fn combined_digest_of_the_sources(files: &[(String, String)]) -> String {
    let mut digest = Sha256Digest::new();
    for (path, file_digest) in files {
        digest.feed(path.as_bytes());
        digest.feed(&[0]);
        digest.feed(file_digest.as_bytes());
        digest.feed(&[0]);
    }
    digest.finish_as_hex()
}

/// 打 `name=crates_file` 每个文件一行与 `name=crates_version` 一行，交回合起来的摘要。
fn emit_the_crates_version(product: &mut ProductWriter) -> Result<String, String> {
    let files = source_files_and_digests()?;
    for (path, digest) in &files {
        product.emit(
            "crates_file",
            &format!("path={} sha256={digest}", field_value(path)),
        );
    }
    let combined = combined_digest_of_the_sources(&files);
    product.emit(
        "crates_version",
        &format!("files={} combined_sha256={combined}", files.len()),
    );
    Ok(combined)
}

/// 开跑之前的三样：常量回比（停机条款第 1 条）、本线程 CPU 钟的分辨率（自检）、读登记的每台默认重活线程数。
/// 交回登记的线程数；常量对不上交回 `Err(EXIT_STOPPED)`，钟或线程数不行交回 `Err(EXIT_RUN_CONDITION_NOT_MET)`。
fn checks_before_the_run(product: &mut ProductWriter) -> Result<usize, i32> {
    let mismatches = local_constants_disagreeing_with_the_core();
    for mismatch in &mismatches {
        product.emit(
            "constant_mismatch",
            &format!(
                "constant={} local={} of_the_crates={}",
                mismatch.name, mismatch.local, mismatch.of_the_crates
            ),
        );
    }
    product.emit(
        "constants",
        &format!(
            "compared_against_the_crates=true mismatches={}",
            mismatches.len()
        ),
    );
    if !mismatches.is_empty() {
        return Err(EXIT_STOPPED);
    }
    let advances = advances_of_the_thread_cpu_clock_in_a_millisecond();
    let is_fine_enough = the_thread_cpu_clock_is_fine_enough(advances);
    product.emit(
        "cpu_clock_check",
        &format!("advances_in_a_millisecond={advances} fine_enough={is_fine_enough}"),
    );
    if !is_fine_enough {
        return Err(EXIT_RUN_CONDITION_NOT_MET);
    }
    registered_heavy_threads().map_err(|reason| {
        product.emit(
            "heavy_threads",
            &format!("readable=false reason={}", field_value(&reason)),
        );
        EXIT_RUN_CONDITION_NOT_MET
    })
}

/// 开跑自检的门槛：1 ms 里至少前进 100 次（登记第十一节、第四节第 3a 步第 2 项）。
fn the_thread_cpu_clock_is_fine_enough(advances_in_a_millisecond: u64) -> bool {
    advances_in_a_millisecond >= CPU_CLOCK_ADVANCES_PER_MILLISECOND_AT_LEAST
}

/// 产物里已有的 `name=crates_version` 与这一次合起来的摘要有一份不同（停机条款第 2 条：`crates/` 在同一段的几条命令之间被改了）。
fn the_crates_changed_since_the_earlier_commands(
    lines_already_in_the_product: &[KindAndFieldsOfAResultLine],
    combined_digest_now: &str,
) -> bool {
    lines_already_in_the_product
        .iter()
        .filter(|(name, _fields)| name == "crates_version")
        .filter_map(|(_name, fields)| fields.get("combined_sha256"))
        .any(|earlier| earlier != combined_digest_now)
}

/// 登记的每台默认重活线程数（`.claude/heavy-thread-default` 第一条非注释行的那个正整数；登记第十四节「线程」）。
fn registered_heavy_threads() -> Result<usize, String> {
    let text = std::fs::read_to_string(".claude/heavy-thread-default")
        .map_err(|error| format!("读 .claude/heavy-thread-default（装置要从仓根起）：{error}"))?;
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .and_then(|line| line.split_whitespace().next())
        .and_then(|number| number.parse::<usize>().ok())
        .filter(|threads| *threads > 0)
        .ok_or_else(|| ".claude/heavy-thread-default 第一条非注释行不是正整数".to_string())
}

// ---- 一次跑：开卡、挂载、起线程 ----

/// 量挂载那一刻开出来的 GPU：交挂载的把手、多判包装的记录（多判的 GPU 臂才有），与卸载之后泄漏了哪几张卡。
struct GpuOfTheRun {
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    rejudge_log: Option<SharedRejudgeLog>,
    gpus_leaked_when_dropped: Option<GpusLeakedWhenTheBackgroundGpuIsDropped>,
}

/// GPU 臂开卡用的那一份进程级 Vulkan 实例（这条命令枚举独显时建的）；这条命令没建（不枚举独显）交回 `Err`。
fn vulkan_instance_for_the_gpu_arm(
    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
) -> Result<&VulkanInstanceOfThisProcess, String> {
    vulkan_instance
        .ok_or_else(|| "GPU 臂要开卡，这条命令却没建进程级 Vulkan 实例（没枚举独显）".to_string())
}

/// 照这条臂开卡（挂载与开卡都在第一次 GC 调用之前做完、在窗口外：登记第一节「不含挂载那一刻量代价」），在 `vulkan_instance` 上开。
fn open_the_gpu_of_the_arm(
    arm: LockstepArm,
    settings: &BackgroundComputeSettings,
    memory_leaked: &GpuMemoryLeakedInThisProcess,
    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
) -> Result<GpuOfTheRun, String> {
    if !arm.compute.uses_the_gpu() {
        return Ok(GpuOfTheRun {
            gpu_compute: None,
            rejudge_log: None,
            gpus_leaked_when_dropped: None,
        });
    }
    let vulkan_instance = vulkan_instance_for_the_gpu_arm(vulkan_instance)?;
    let selection = BackgroundGpuSelection::of_the_settings(settings);
    match arm.rejudging {
        RejudgingOfTheArm::NotRejudged => {
            let opened = open_background_gpu_compute(&selection, memory_leaked, vulkan_instance)
                .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
            Ok(GpuOfTheRun {
                gpu_compute: Some(opened.gpu_compute),
                rejudge_log: None,
                gpus_leaked_when_dropped: Some(opened.gpus_leaked_when_dropped),
            })
        }
        RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu => {
            let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
                &selection,
                memory_leaked,
                vulkan_instance,
            )
            .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
            let gpus_leaked_when_dropped = background_gpu.gpus_leaked_when_dropped();
            let rejudge_log: SharedRejudgeLog = Arc::new(Mutex::new(RejudgeLog::default()));
            let rejudged = GpuComputeRejudgedOnTheCpu {
                inner: BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(
                    background_gpu,
                ),
                log: Arc::clone(&rejudge_log),
            };
            Ok(GpuOfTheRun {
                gpu_compute: Some(BackgroundGpuComputeHandle::of_an_implementation(rejudged)),
                rejudge_log: Some(rejudge_log),
                gpus_leaked_when_dropped: Some(gpus_leaked_when_dropped),
            })
        }
    }
}

/// 从起步镜像拷一份、照设置可写挂载、装上开关（登记第五节「量挂载」）。
fn measuring_mount(
    image: &StartingImage,
    settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    batch_units: usize,
) -> Result<(MountedSession, PoolDevices, SharedRecordingDigest), String> {
    let (mut devices, recording) = copied_pool_devices(&image.devices);
    let mounted = mount_writable_with_background_compute_settings(
        &make_filesystem_parameters_of_the_pool(),
        &mut devices,
        settings,
        gpu_compute,
    )
    .map_err(|refused| format!("量挂载：{refused:?}"))?;
    let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
    session
        .allocator
        .set_compaction_switches(compaction_switches_of_the_measuring_mount(batch_units));
    Ok((session, devices, recording))
}

// ---- 强制选段后备（登记第十二节修订第 8 条：几何核后一半照 E185（GC一批搬迁的挂钟拆分） 走强制选段，六条臂同一点名法、同一序列）----

/// 强制选段后备的一条意图：点名的那一段、同一刻 greedy 照核心层重放要选的那一段、核心层签没签出去（没签出去是它停在「一段都没资格」上，
/// 照资格五条判这一段不合格），与签出去的那一条意图的段起点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ForcedChoiceOfAnIntent {
    forced: SegmentOfTheChoice,
    greedy: SegmentOfTheChoice,
    issued: bool,
    issued_region_start_slot: Option<u64>,
}

/// 一次跑里强制选段后备记下的：每一条意图（按先后）、碎片段用完没有、点名用的挂钟（在 GC 线程上、计时窗口外；并发跑里拿着会话锁）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ForcedFallbackReading {
    choices: Vec<ForcedChoiceOfAnIntent>,
    fragment_segments_ran_out: bool,
    choosing_wall: Duration,
}

impl ForcedFallbackReading {
    fn intents_issued(&self) -> u64 {
        u64::try_from(self.choices.iter().filter(|choice| choice.issued).count()).expect("意图数")
    }

    fn segments_refused_by_the_core(&self) -> u64 {
        u64::try_from(self.choices.iter().filter(|choice| !choice.issued).count()).expect("意图数")
    }

    /// 签出去的意图不在点名的那一段上（开关没装上、或核心层没照强制的段签）的条数；应是 0。
    fn intents_issued_off_the_forced_region(&self) -> u64 {
        u64::try_from(
            self.choices
                .iter()
                .filter(|choice| {
                    choice.issued && choice.issued_region_start_slot != choice.forced.start_slot
                })
                .count(),
        )
        .expect("意图数")
    }

    /// `name=run` 与 `name=concurrent_run` 行里的那几段。
    fn fields(&self) -> String {
        format!(
            "choice=forced_fallback forced_intents_issued={} forced_segments_refused_by_the_core={} \
             intents_issued_off_the_forced_region={} fragment_segments_ran_out={} forced_choice_wall_nanoseconds={}",
            self.intents_issued(),
            self.segments_refused_by_the_core(),
            self.intents_issued_off_the_forced_region(),
            self.fragment_segments_ran_out,
            self.choosing_wall.as_nanos(),
        )
    }

    /// 每条意图一行 `name=forced_choice`（`prefix` 是档、臂、重复、会话那几段）。
    fn choice_lines(&self, prefix: &str) -> Vec<String> {
        let start = |segment: SegmentOfTheChoice| {
            segment
                .start_slot
                .map_or_else(|| "none".to_string(), |slot| slot.to_string())
        };
        self.choices
            .iter()
            .enumerate()
            .map(|(intent, choice)| {
                format!(
                    "{prefix} intent={intent} choice=forced_fallback region_start_slot={} live_slots={} \
                     holds_only_data_units={} issued={} issued_region_start_slot={} greedy_region_start_slot={} \
                     greedy_live_slots={} greedy_holds_only_data_units={}",
                    start(choice.forced),
                    choice.forced.live_slots,
                    choice.forced.holds_only_data_units,
                    choice.issued,
                    choice
                        .issued_region_start_slot
                        .map_or_else(|| "none".to_string(), |slot| slot.to_string()),
                    start(choice.greedy),
                    choice.greedy.live_slots,
                    choice.greedy.holds_only_data_units,
                )
            })
            .collect()
    }
}

/// 一次跑里强制选段后备的状态：碎片段、点过名的段、还没签出去的那一次点名（连同点名时现行那一版的 checkpoint 号）与记下的。
struct ForcedFallbackOfTheRun {
    fragment_segment_starts: BTreeSet<u64>,
    lowest_live_data_inode: u64,
    live_data_units: u64,
    segments_already_forced: BTreeSet<u64>,
    pending: Option<(CheckpointTxg, ForcedChoiceOfAnIntent)>,
    reading: ForcedFallbackReading,
}

impl ForcedFallbackOfTheRun {
    fn of_the_image(image: &StartingImage) -> ForcedFallbackOfTheRun {
        ForcedFallbackOfTheRun {
            fragment_segment_starts: image.fragment_segment_starts.clone(),
            lowest_live_data_inode: image.lowest_live_data_inode,
            live_data_units: u64::try_from(image.write_generation_of_each_unit.len()).expect("P"),
            segments_already_forced: BTreeSet::new(),
            pending: None,
            reading: ForcedFallbackReading::default(),
        }
    }

    /// 照现行那一版点名：碎片段里下一段（[`fragment_segment_to_force`]），与同一刻 greedy 要选的那一段。碎片段用完交回 `None`。
    fn choose(
        &self,
        session: &MountedSession,
        devices: &PoolDevices,
    ) -> Option<ForcedChoiceOfAnIntent> {
        let version = version_of_trees(session);
        let unit_area = unit_area_of_device_zero(session);
        let unit_area_start_slot = unit_area.start.slot().0;
        let data_slots_now: BTreeSet<u64> = data_unit_slots_of_the_inodes(
            version,
            self.lowest_live_data_inode,
            live_data_files(self.live_data_units),
        )
        .into_values()
        .collect();
        let live_slots =
            live_slots_per_cluster_segment_by_the_records(&version.allocation_records, unit_area);
        let segments_holding_other_records = segments_holding_a_record_other_than_a_data_unit(
            &version.allocation_records,
            unit_area,
            &data_slots_now,
        );
        let forced = fragment_segment_to_force(
            &self.fragment_segment_starts,
            &self.segments_already_forced,
            unit_area_start_slot,
            &live_slots,
            &segments_holding_other_records,
        )?;
        let greedy_segment_start_slot = segment_the_greedy_choice_picks(
            version,
            unit_area,
            session.allocator.cluster_segments(),
            session
                .compaction
                .segments_refused_by_the_region_scan
                .clone(),
            session.compaction.quarantined_slots(),
            devices,
        );
        Some(ForcedChoiceOfAnIntent {
            forced,
            greedy: segment_of_the_choice(
                greedy_segment_start_slot,
                unit_area_start_slot,
                &live_slots,
                &segments_holding_other_records,
            ),
            issued: false,
            issued_region_start_slot: None,
        })
    }

    /// 一次 GC 调用之前（计时窗口外）：活意图的段记成点过名；还没点名、点的段已点过名、或现行那一版换过了，就照现行那一版重新点名；
    /// 把点名的段装进开关的 `forced_region`（碎片段用完装 `None`，之后不再强制）。交回调用之前的签发计数。
    fn before_the_call(&mut self, session: &mut MountedSession, devices: &PoolDevices) -> u64 {
        let started = Instant::now();
        if let Some(live) = session.allocator.compaction().live_intent() {
            self.segments_already_forced
                .insert(live.intent.region.start_slot().0);
        }
        let checkpoint_txg = version_of_trees(session).root.checkpoint_txg;
        let pending_is_current = self.pending.is_some_and(|(chosen_at, choice)| {
            chosen_at == checkpoint_txg
                && choice
                    .forced
                    .start_slot
                    .is_some_and(|start| !self.segments_already_forced.contains(&start))
        });
        if !pending_is_current && !self.reading.fragment_segments_ran_out {
            self.pending = self
                .choose(session, devices)
                .map(|choice| (checkpoint_txg, choice));
            self.reading.fragment_segments_ran_out = self.pending.is_none();
        }
        let forced_region = self
            .pending
            .and_then(|(_chosen_at, choice)| choice.forced.start_slot)
            .map(|start| {
                CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(start))
                    .expect("碎片段起点在聚簇段边界上、装得进 6 字节")
            });
        session.allocator.compaction_mut().switches.forced_region = forced_region;
        self.reading.choosing_wall += started.elapsed();
        session.compaction.observations.intents_issued
    }

    /// 一次 GC 调用之后：签发计数涨了，那一次点名签出去了（签出去的段取活意图的段；签了又在同一次调用里做完的取上一条做完的意图的段）；
    /// 没涨而这次调用停在「一段都没资格」上，核心层照资格五条拒了点名的那一段。两种都记成一条意图、那一段记成点过名。
    fn after_the_call(
        &mut self,
        session: &MountedSession,
        intents_issued_before_the_call: u64,
        call: &BackgroundCompactionCall,
    ) {
        let Some((_chosen_at, mut choice)) = self.pending else {
            return;
        };
        if session.compaction.observations.intents_issued > intents_issued_before_the_call {
            choice.issued = true;
            choice.issued_region_start_slot = session
                .allocator
                .compaction()
                .live_intent()
                .map(|live| live.intent.region)
                .or(session
                    .compaction
                    .last_completed_intent
                    .map(|intent| intent.region))
                .map(|region| region.start_slot().0);
        } else if !the_call_stopped_with_no_eligible_segment(call) {
            return;
        }
        self.segments_already_forced
            .extend(choice.forced.start_slot);
        self.reading.choices.push(choice);
        self.pending = None;
    }
}

/// 这次调用起跑了、停在「一段都没资格」上（强制选的那一段不合格也停在这里）。
fn the_call_stopped_with_no_eligible_segment(call: &BackgroundCompactionCall) -> bool {
    matches!(
        call,
        BackgroundCompactionCall::Ran { run, .. }
            if matches!(run.outcome, CompactionRunOutcome::Stopped(CompactionRunStop::NoEligibleSegment))
    )
}

/// 前台第 `change_number` 次用户改动（种子与序号定单元与内容）：按那个单元在它那个活数据文件里的偏移覆盖写它
/// （[`unit_index_of_the_user_change`] 只取文件的最后一个单元，只重写这一个），记下这个单元最后一次写是第几次。
fn publish_the_user_change(
    session: &mut MountedSession,
    devices: &mut PoolDevices,
    lowest_live_data_inode: u64,
    change_number: u64,
    write_generation_of_each_unit: &mut [u64],
    write_time_seconds: u64,
) {
    let live_data_units = u64::try_from(write_generation_of_each_unit.len()).expect("P");
    let unit_index = unit_index_of_the_user_change(change_number, live_data_units);
    let (file, index_in_file) = file_and_index_of_a_live_data_unit(unit_index);
    assert_eq!(
        index_in_file + 1,
        LIVE_DATA_UNITS_PER_FILE,
        "前台用户改动只覆盖写文件的最后一个单元（按偏移写重写到文件末尾，R4）"
    );
    let write_generation = WRITE_GENERATION_OF_THE_FIRST_USER_CHANGE + change_number;
    let mut unit_bytes = vec![0u8; data_unit_payload_capacity()];
    fill_the_content_of_a_data_unit(unit_index, write_generation, &mut unit_bytes);
    let capacity = u64::try_from(unit_bytes.len()).expect("净荷容量装得进 u64");
    session
        .publish_user_change(
            devices,
            UserChange::WriteAtOffset {
                inode: InodeNumber(lowest_live_data_inode + file),
                offset: FileOffsetInBytes(index_in_file * capacity),
                bytes: PartialWriteBytes::new(&unit_bytes).expect("一个单元的净荷不是空的"),
                write_time_seconds,
            },
        )
        .unwrap_or_else(|refusal| panic!("前台第 {change_number} 次用户改动发得成：{refusal:?}"));
    write_generation_of_each_unit[usize::try_from(unit_index).expect("单元下标")] =
        write_generation;
}

/// 每块盘的单元区（CPU 多判臂对每块盘的分配记录各调一次活槽段）。
fn unit_areas_of_every_device(session: &MountedSession) -> Vec<UnitAreaOfOneDevice> {
    session
        .allocator
        .devices
        .iter()
        .zip([DeviceIdentity(0), DeviceIdentity(1)])
        .map(|(free_map, device)| UnitAreaOfOneDevice {
            device,
            start: free_map.unit_area_start(),
            slots: free_map.unit_area_slots(),
        })
        .collect()
}

/// CPU 多判臂在窗口之内做的那一段：这次调用发出的每一批把三段的 CPU 写法各再跑一遍，里层计时在这条线程的 CPU 钟上量。
fn rejudge_the_batches_of_the_call_on_the_cpu(
    call: &BackgroundCompactionCall,
    devices: &PoolDevices,
    unit_areas: &[UnitAreaOfOneDevice],
) -> (Option<Duration>, RejudgeTally) {
    let BackgroundCompactionCall::Ran { run, .. } = call else {
        return (Some(Duration::ZERO), RejudgeTally::default());
    };
    let (tally, spent) = thread_cpu_spent_in(|| {
        let mut tally = RejudgeTally::default();
        for version in run
            .versions_published
            .iter()
            .filter(|version| !version.units_relocated_by_this_publish.is_empty())
        {
            tally.add(rejudged_batch_on_the_cpu(version, devices, unit_areas));
        }
        tally
    });
    (spent, tally)
}

/// 交替跑里前台线程与 GC 线程之间来回交的那一份：会话与两块盘（同一时刻只在一条线程手里）。
struct SessionAndDevices {
    session: MountedSession,
    devices: PoolDevices,
}

/// 前台线程交给 GC 线程的。封闭集合。
enum ToTheGcThread {
    /// 第 `after_the_user_change` 次用户改动发成了，GC 线程一次次调后台入口直到一次整理发布都没发，再交回来。
    RunAfterTheUserChange {
        state: Box<SessionAndDevices>,
        after_the_user_change: u64,
        foreground_thread_task: u64,
    },
    /// 用户改动都做完了。
    Finish,
}

/// 一次交替跑记下的（窗口外读）。
struct LockstepRunRecords {
    records: Vec<GcCallRecord>,
    forced_fallback: ForcedFallbackReading,
    the_opening_thread_was_running_at_the_first_call: bool,
    state: SessionAndDevices,
    write_generation_of_each_unit: Vec<u64>,
}

/// 交替跑里 GC 线程交回的：每次调用的记录、头一次调用时开卡线程还在不在跑、强制选段后备记下的。
type GcThreadRecordsOfALockstepRun = (Vec<GcCallRecord>, bool, ForcedFallbackReading);

/// 交替跑一次（登记第五节「两种跑法」）：主线程起前台线程与 GC 线程，两条线程经通道交替；GC 调用期间前台线程阻塞在通道上。
fn lockstep_threads(
    session_and_devices_at_the_start: SessionAndDevices,
    lowest_live_data_inode: u64,
    write_generation_of_each_unit_at_the_start: Vec<u64>,
    first_write_time_seconds: u64,
    user_changes: u64,
    arm: LockstepArm,
    forced_fallback_of_the_run: ForcedFallbackOfTheRun,
) -> Result<LockstepRunRecords, ClockReadFailed> {
    let main_thread_task = task_identifier_of_this_thread()?;
    let unit_areas = unit_areas_of_every_device(&session_and_devices_at_the_start.session);
    let (to_the_gc_thread, gc_thread_inbox) = mpsc::channel::<ToTheGcThread>();
    let (to_the_foreground_thread, foreground_thread_inbox) = mpsc::channel::<SessionAndDevices>();
    std::thread::scope(|scope| {
        let foreground = scope.spawn(
            move || -> Result<(SessionAndDevices, Vec<u64>), ClockReadFailed> {
                let foreground_thread_task = task_identifier_of_this_thread()?;
                let mut state = session_and_devices_at_the_start;
                let mut write_generation_of_each_unit = write_generation_of_each_unit_at_the_start;
                for change_number in 0..user_changes {
                    publish_the_user_change(
                        &mut state.session,
                        &mut state.devices,
                        lowest_live_data_inode,
                        change_number,
                        &mut write_generation_of_each_unit,
                        first_write_time_seconds + change_number,
                    );
                    to_the_gc_thread
                        .send(ToTheGcThread::RunAfterTheUserChange {
                            state: Box::new(state),
                            after_the_user_change: change_number,
                            foreground_thread_task,
                        })
                        .expect("GC 线程在等");
                    state = foreground_thread_inbox.recv().expect("GC 线程交回会话");
                }
                to_the_gc_thread
                    .send(ToTheGcThread::Finish)
                    .expect("GC 线程在等");
                Ok((state, write_generation_of_each_unit))
            },
        );
        let gc = scope.spawn(move || -> GcThreadRecordsOfALockstepRun {
            let mut forced_fallback = forced_fallback_of_the_run;
            let started = StartOnTheMonotonicRawClock::now();
            let gc_thread_task = task_identifier_of_this_thread().ok();
            let caller = CallerOfTheLockstepRun::starting_at_zero();
            let pacing = pacing_of_the_measuring_mount();
            let mut records: Vec<GcCallRecord> = Vec::new();
            let mut the_opening_thread_was_running_at_the_first_call = false;
            while let Ok(ToTheGcThread::RunAfterTheUserChange {
                mut state,
                after_the_user_change,
                foreground_thread_task,
            }) = gc_thread_inbox.recv()
            {
                let clocks = ClocksOfTheGcThread {
                    started,
                    gc_thread_task,
                    foreground_thread_task,
                    main_thread_task,
                };
                loop {
                    if records.is_empty() {
                        the_opening_thread_was_running_at_the_first_call =
                            the_gpu_opening_thread_is_still_running().unwrap_or(true);
                    }
                    caller.advance_before_the_next_call();
                    let empty_segments_before_the_call =
                        empty_segments_of_every_device(&state.session);
                    let intents_issued_before_the_call =
                        forced_fallback.before_the_call(&mut state.session, &state.devices);
                    let measured = measure_one_gc_call(
                        &clocks,
                        &mut state,
                        |state| {
                            state
                                .session
                                .run_compaction_in_the_background(
                                    &mut state.devices,
                                    pacing,
                                    &caller,
                                )
                                .expect("盘表是挂载核过的那一份、现行那一版有树：后台入口不拒")
                        },
                        |state, call| match arm.rejudging {
                            RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu
                                if !arm.compute.uses_the_gpu() =>
                            {
                                rejudge_the_batches_of_the_call_on_the_cpu(
                                    call,
                                    &state.devices,
                                    &unit_areas,
                                )
                            }
                            RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu
                            | RejudgingOfTheArm::NotRejudged => {
                                (Some(Duration::ZERO), RejudgeTally::default())
                            }
                        },
                    );
                    forced_fallback.after_the_call(
                        &state.session,
                        intents_issued_before_the_call,
                        &measured.outcome,
                    );
                    let mut record = record_of_the_call(
                        &measured.outcome,
                        after_the_user_change,
                        measured.window,
                        measured.inner_rejudge_cpu,
                        measured.rejudge_tally,
                    );
                    record.empty_segments_before_the_call = empty_segments_before_the_call;
                    record.empty_segments_after_the_call =
                        empty_segments_of_every_device(&state.session);
                    let is_done = the_call_published_nothing(&record);
                    records.push(record);
                    if is_done {
                        break;
                    }
                }
                to_the_foreground_thread.send(*state).expect("前台线程在等");
            }
            (
                records,
                the_opening_thread_was_running_at_the_first_call,
                forced_fallback.reading,
            )
        });
        let (records, the_opening_thread_was_running_at_the_first_call, forced_fallback) =
            gc.join().expect("GC 线程没有 panic");
        let (state, write_generation_of_each_unit) =
            foreground.join().expect("前台线程没有 panic")?;
        Ok(LockstepRunRecords {
            records,
            forced_fallback,
            the_opening_thread_was_running_at_the_first_call,
            state,
            write_generation_of_each_unit,
        })
    })
}

/// 一次交替跑的结果：`name=run` 一行的数、锚点与作废原因。
struct LockstepRunSummary {
    arm: LockstepArm,
    repeat: u32,
    calls: u64,
    calls_started: u64,
    batches: u64,
    units_relocated: u64,
    gc_wall: Duration,
    process_cpu: Duration,
    gc_thread_cpu: Duration,
    foreground_thread_cpu: Duration,
    main_thread_cpu: Duration,
    inner_rejudge_cpu: Duration,
    rejudge: RejudgeTally,
    recording_digest: String,
    recording_operations: u64,
    recording_overhead: Duration,
    content_digest: String,
    content_matches_the_expectation: bool,
    host_other_cores: f64,
    path_anchor: PathAnchor,
    gpu_work_anchors: GpuWorkAnchors,
    batches_not_adding_up_to_the_entries_handed_in: u64,
    live_slot_calls_not_handing_in_every_full_segment: u64,
    batches_with_live_slot_entries_not_whole_device_passes: u64,
    most_compaction_publishes_between_two_user_changes: u64,
    calls_failing_the_inclusion_checks: u64,
    /// 判不过包含自检的那几次调用的窗口（不进产物行；交替跑每次尝试之后打到标准错误，小池用例判红时也打出来，看是哪一条不等式、差多少）。
    windows_failing_the_inclusion_checks: Vec<CallWindow>,
    cpu_arm_thread_cpu_covers_the_wall: Option<bool>,
    gpus_leaked: u64,
    calls_with_the_alarm: u64,
    budget_left_trajectory: TrajectoryOfTheReadings,
    uncertain_entries_trajectory: TrajectoryOfTheReadings,
    empty_segments_trajectory: Vec<EmptySegmentsTrajectoryOfADevice>,
    forced_fallback: ForcedFallbackReading,
    stops: BTreeMap<String, u64>,
    void_reason: Option<String>,
}

impl LockstepRunSummary {
    /// 第七节甲类、乙类锚点都成立（停机条款第 4 条看甲类；乙类不符那一档作废）。
    fn class_one_anchors_hold(&self) -> bool {
        class_one_anchors_of_a_lockstep_run_hold(
            self.path_anchor,
            self.arm.compute,
            self.gpu_work_anchors,
            self.batches_not_adding_up_to_the_entries_handed_in,
            self.most_compaction_publishes_between_two_user_changes,
        )
    }

    fn class_two_anchors_hold(&self) -> bool {
        self.live_slot_calls_not_handing_in_every_full_segment == 0
            && self.batches_with_live_slot_entries_not_whole_device_passes == 0
    }

    fn fields(&self, tier: UnitAreaTier, session_number: u64, gpu_models: &str) -> String {
        let stops: Vec<String> = self
            .stops
            .iter()
            .map(|(stop, count)| format!("{stop}:{count}"))
            .collect();
        format!(
            "tier={} arm={} repeat={} session={session_number} calls={} calls_started={} batches={} units_relocated={} \
             gc_wall_nanoseconds={} gc_wall_nanoseconds_per_batch={} process_cpu_nanoseconds={} gc_thread_cpu_nanoseconds={} \
             foreground_thread_cpu_nanoseconds={} main_thread_cpu_nanoseconds={} inner_rejudge_cpu_nanoseconds={} \
             rejudged_entries={} rejudge_mismatches={} recording_digest={} recording_operations={} recording_overhead_nanoseconds={} \
             content_digest={} content_matches_the_expectation={} host_other_cores={:.2} gpu_models={} \
             compute_records={} compute_records_on_the_path_of_the_settings={} path_anchor_holds={} \
             gpu_records={} segments_given_to_a_card_other_than_the_lowest={} segments_without_a_submission={} \
             recuts_after_running_out_of_memory={} segments_with_pieces_other_than_submissions={} uncertain_entries={} \
             entries_judged_by_the_cpu_reference={} batches_abandoned_after_a_compute_error={} \
             batches_not_adding_up_to_the_entries_handed_in={} live_slot_calls_not_handing_in_every_full_segment={} \
             batches_with_live_slot_entries_not_whole_device_passes={} most_compaction_publishes_between_two_user_changes={} \
             calls_failing_the_inclusion_checks={} cpu_arm_thread_cpu_covers_the_wall={} class_one_anchors_hold={} \
             class_two_anchors_hold={} gpus_leaked={} calls_with_the_alarm={} {} {} {} {} stops={} void={}",
            tier.name(),
            self.arm.name(),
            self.repeat,
            self.calls,
            self.calls_started,
            self.batches,
            self.units_relocated,
            self.gc_wall.as_nanos(),
            self.gc_wall.as_nanos() / u128::from(self.batches.max(1)),
            self.process_cpu.as_nanos(),
            self.gc_thread_cpu.as_nanos(),
            self.foreground_thread_cpu.as_nanos(),
            self.main_thread_cpu.as_nanos(),
            self.inner_rejudge_cpu.as_nanos(),
            self.rejudge.entries_rejudged,
            self.rejudge.mismatches,
            self.recording_digest,
            self.recording_operations,
            self.recording_overhead.as_nanos(),
            self.content_digest,
            self.content_matches_the_expectation,
            self.host_other_cores,
            gpu_models,
            self.path_anchor.compute_records,
            self.path_anchor.compute_records_on_the_path_of_the_settings,
            self.path_anchor.holds(),
            self.gpu_work_anchors.gpu_records,
            self.gpu_work_anchors.segments_given_to_a_card_other_than_the_lowest,
            self.gpu_work_anchors.segments_without_a_submission,
            self.gpu_work_anchors.recuts_after_running_out_of_memory,
            self.gpu_work_anchors.segments_with_pieces_other_than_submissions,
            self.gpu_work_anchors.uncertain_entries,
            self.gpu_work_anchors.entries_judged_by_the_cpu_reference,
            self.gpu_work_anchors.batches_abandoned_after_a_compute_error,
            self.batches_not_adding_up_to_the_entries_handed_in,
            self.live_slot_calls_not_handing_in_every_full_segment,
            self.batches_with_live_slot_entries_not_whole_device_passes,
            self.most_compaction_publishes_between_two_user_changes,
            self.calls_failing_the_inclusion_checks,
            self.cpu_arm_thread_cpu_covers_the_wall
                .map_or_else(|| "not_a_cpu_arm".to_string(), |covers| covers.to_string()),
            self.class_one_anchors_hold(),
            self.class_two_anchors_hold(),
            self.gpus_leaked,
            self.calls_with_the_alarm,
            trajectory_fields("budget_left", self.budget_left_trajectory),
            trajectory_fields("uncertain_entries", self.uncertain_entries_trajectory),
            empty_segments_fields(&self.empty_segments_trajectory),
            self.forced_fallback.fields(),
            field_value(&stops.join(",")),
            field_value(self.void_reason.as_deref().unwrap_or("no")),
        )
    }
}

/// 交替跑一次的甲类锚点（第七节甲类第 1–4 行）都成立：每一项记账照设置走、条数不少于批数；GPU 臂另要分到活的卡、切片与两份核比对那几样；
/// 多判的 GPU 臂每批三段的判定加起来等于交进去的；相邻两次用户改动之间的整理发布不超过 B_gc。
fn class_one_anchors_of_a_lockstep_run_hold(
    path_anchor: PathAnchor,
    compute: ComputeOfTheArm,
    gpu_work_anchors: GpuWorkAnchors,
    batches_not_adding_up_to_the_entries_handed_in: u64,
    most_compaction_publishes_between_two_user_changes: u64,
) -> bool {
    path_anchor.holds()
        && (!compute.uses_the_gpu() || gpu_work_anchors.hold())
        && batches_not_adding_up_to_the_entries_handed_in == 0
        && most_compaction_publishes_between_two_user_changes
            <= LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE
}

/// 一次交替跑的设定。
struct LockstepRunSetting<'run> {
    image: &'run StartingImage,
    discrete_gpus: &'run [DiscreteGpuSeenByThisProcess],
    memory_leaked: &'run GpuMemoryLeakedInThisProcess,
    /// 开卡用的那一份进程级 Vulkan 实例；只跑 CPU 臂的没有。
    vulkan_instance: Option<&'run VulkanInstanceOfThisProcess>,
    user_changes: u64,
    batch_units: usize,
}

/// 交替跑一次：拷池、开卡、挂载、两条线程交替、读摘要与内容、核锚点、记机器负载。开卡或挂载不成交回 `Err`（交主 agent，不当跑过）。
fn run_lockstep_once(
    setting: &LockstepRunSetting<'_>,
    arm: LockstepArm,
    repeat: u32,
) -> Result<LockstepRunSummary, String> {
    let host_at_the_start = host_load_reading()?;
    let settings = background_compute_settings_of_the_arm(arm.compute, setting.discrete_gpus);
    let gpu = open_the_gpu_of_the_arm(
        arm,
        &settings,
        setting.memory_leaked,
        setting.vulkan_instance,
    )?;
    let (session, devices, recording) = measuring_mount(
        setting.image,
        settings,
        gpu.gpu_compute,
        setting.batch_units,
    )?;
    let threads = lockstep_threads(
        SessionAndDevices { session, devices },
        setting.image.lowest_live_data_inode,
        setting.image.write_generation_of_each_unit.clone(),
        setting.image.next_write_time_seconds,
        setting.user_changes,
        arm,
        ForcedFallbackOfTheRun::of_the_image(setting.image),
    );
    let LockstepRunRecords {
        records,
        forced_fallback,
        the_opening_thread_was_running_at_the_first_call,
        state,
        write_generation_of_each_unit,
    } = threads.map_err(|failed| format!("拿线程的 CPU 钟：{}", failed.description()))?;
    let (recording_digest, recording_operations, recording_overhead, recording_clock_failures) = {
        let recording = recording.lock().expect("录制摘要没在别处 panic 过");
        (
            recording.digest.clone().finish_as_hex(),
            recording.operations,
            recording.cpu_spent_feeding,
            recording.clock_read_failures,
        )
    };
    let content_digest = read_content_digest(
        &state.devices,
        version_of_trees(&state.session),
        setting.image.lowest_live_data_inode,
        u64::try_from(write_generation_of_each_unit.len()).expect("P"),
    )?;
    let content_matches_the_expectation =
        content_digest == expected_content_digest(&write_generation_of_each_unit);
    let full_segments_of_a_device =
        unit_area_of_device_zero(&state.session).slots / LOCAL_CLUSTER_SEGMENT_SLOTS;
    drop(state);
    let gpus_leaked = match gpu
        .gpus_leaked_when_dropped
        .as_ref()
        .and_then(GpusLeakedWhenTheBackgroundGpuIsDropped::gpus_leaked)
    {
        Some(leaked) => {
            setting
                .memory_leaked
                .note_the_gpus_leaked_when_dropped(&leaked);
            u64::try_from(leaked.len()).expect("卡数")
        }
        None => 0,
    };
    let windows: Vec<CallWindow> = records.iter().filter_map(|record| record.window).collect();
    let every_window_read = windows.len() == records.len();
    let (
        handed_in_of_each_batch,
        live_slot_calls_not_handing_in_every_full_segment,
        rejudge_clock_failures,
        wrapper_tally,
        wrapper_inner,
    ) = match &gpu.rejudge_log {
        Some(log) => {
            let log = log.lock().expect("多判记录没在别处 panic 过");
            (
                Some(log.handed_in_of_each_batch.clone()),
                log.live_slot_calls_not_handing_in_every_full_segment,
                log.clock_read_failures,
                log.tally,
                log.inner_rejudge_cpu,
            )
        }
        None => (None, 0, 0, RejudgeTally::default(), Duration::ZERO),
    };
    let mut rejudge = wrapper_tally;
    for record in &records {
        rejudge.add(record.rejudge_tally);
    }
    let gc_wall = gc_wall_clock_of_the_run(&windows, &records);
    let gc_thread_cpu: Duration = windows.iter().map(|window| window.gc_thread_cpu).sum();
    let cpu_arm_thread_cpu_covers_the_wall = (!arm.compute.uses_the_gpu())
        .then(|| cpu_arm_thread_cpu_covers_the_wall(gc_thread_cpu, gc_wall));
    let count_of_calls_failing_the_inclusion_checks = calls_failing_the_inclusion_checks(&windows);
    let windows_failing_the_inclusion_checks: Vec<CallWindow> = windows
        .iter()
        .filter(|window| calls_failing_the_inclusion_checks(std::slice::from_ref(*window)) > 0)
        .copied()
        .collect();
    let lowest_card = setting.discrete_gpus.first().map(|lowest_discrete_gpu| {
        gpu_device_identifier_of_the_pci_bus_address(&lowest_discrete_gpu.pci_bus_address)
    });
    let gpu_work_anchors = lowest_card
        .as_ref()
        .map_or_else(GpuWorkAnchors::default, |lowest_card| {
            gpu_work_anchors(&records, lowest_card)
        });
    let void_reason =
        if !every_window_read || recording_clock_failures > 0 || rejudge_clock_failures > 0 {
            Some("a_clock_read_failed_or_went_backwards".to_string())
        } else if count_of_calls_failing_the_inclusion_checks > 0
            || cpu_arm_thread_cpu_covers_the_wall == Some(false)
        {
            Some("an_inclusion_check_failed".to_string())
        } else if the_opening_thread_was_running_at_the_first_call {
            Some("the_gpu_opening_thread_was_running_at_the_first_call".to_string())
        } else {
            None
        };
    let mut stops: BTreeMap<String, u64> = BTreeMap::new();
    for record in &records {
        *stops.entry(record.stop_name.clone()).or_insert(0) += 1;
    }
    let host_at_the_end = host_load_reading()?;
    Ok(LockstepRunSummary {
        arm,
        repeat,
        calls: u64::try_from(records.len()).expect("调用数"),
        calls_started: u64::try_from(records.iter().filter(|record| record.started).count())
            .expect("调用数"),
        batches: records.iter().map(|record| record.batches).sum(),
        units_relocated: records
            .iter()
            .flat_map(|record| record.units_relocated_of_each_batch.iter())
            .sum(),
        gc_wall,
        process_cpu: windows.iter().map(|window| window.process_cpu).sum(),
        gc_thread_cpu,
        foreground_thread_cpu: windows
            .iter()
            .map(|window| window.foreground_thread_cpu)
            .sum(),
        main_thread_cpu: windows.iter().map(|window| window.main_thread_cpu).sum(),
        inner_rejudge_cpu: wrapper_inner
            + records
                .iter()
                .map(|record| record.inner_rejudge_cpu)
                .sum::<Duration>(),
        rejudge,
        recording_digest,
        recording_operations,
        recording_overhead,
        content_digest,
        content_matches_the_expectation,
        host_other_cores: cores_used_outside_this_process(host_at_the_start, host_at_the_end),
        path_anchor: path_anchor(&records, arm.compute),
        gpu_work_anchors,
        batches_not_adding_up_to_the_entries_handed_in: handed_in_of_each_batch.map_or(
            0,
            |handed_in| {
                batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(&records, &handed_in)
            },
        ),
        live_slot_calls_not_handing_in_every_full_segment,
        batches_with_live_slot_entries_not_whole_device_passes:
            batches_whose_live_slot_entries_are_not_whole_device_passes(
                &records,
                full_segments_of_a_device,
            ),
        most_compaction_publishes_between_two_user_changes:
            most_compaction_publishes_between_two_user_changes(&records),
        calls_failing_the_inclusion_checks: count_of_calls_failing_the_inclusion_checks,
        windows_failing_the_inclusion_checks,
        cpu_arm_thread_cpu_covers_the_wall,
        gpus_leaked,
        calls_with_the_alarm: u64::try_from(
            records.iter().filter(|record| record.has_the_alarm).count(),
        )
        .expect("调用数"),
        budget_left_trajectory: trajectory_of_the_readings(
            &budget_left_before_and_after_each_call(&records),
        ),
        uncertain_entries_trajectory: trajectory_of_the_readings(
            &uncertain_entries_of_each_gpu_record(&records),
        ),
        empty_segments_trajectory: empty_segments_trajectory_of_each_device(&records),
        forced_fallback,
        stops,
        void_reason,
    })
}

// ---- 一档交替跑的判定（登记第六节量 W、C、S、P；量 T 要三档，在 `judge-trend`）----

/// 判定用的那几个数：这一次跑刚量出的，或续跑时从产物的 `name=run` 行读回的。
#[derive(Clone, Copy, Debug, PartialEq)]
struct RunNumbers {
    arm: LockstepArm,
    repeat: u32,
    batches: u64,
    gc_wall_nanoseconds: f64,
    process_cpu_nanoseconds: f64,
    inner_rejudge_cpu_nanoseconds: f64,
}

impl RunNumbers {
    fn of_the_summary(summary: &LockstepRunSummary) -> RunNumbers {
        RunNumbers {
            arm: summary.arm,
            repeat: summary.repeat,
            batches: summary.batches,
            gc_wall_nanoseconds: nanoseconds_as_float(summary.gc_wall),
            process_cpu_nanoseconds: nanoseconds_as_float(summary.process_cpu),
            inner_rejudge_cpu_nanoseconds: nanoseconds_as_float(summary.inner_rejudge_cpu),
        }
    }

    fn of_the_fields(fields: &BTreeMap<String, String>) -> Option<RunNumbers> {
        let number = |key: &str| fields.get(key).and_then(|value| value.parse::<f64>().ok());
        Some(RunNumbers {
            arm: LockstepArm::of_the_name(fields.get("arm")?)?,
            repeat: fields.get("repeat")?.parse().ok()?,
            batches: fields.get("batches")?.parse().ok()?,
            gc_wall_nanoseconds: number("gc_wall_nanoseconds")?,
            process_cpu_nanoseconds: number("process_cpu_nanoseconds")?,
            inner_rejudge_cpu_nanoseconds: number("inner_rejudge_cpu_nanoseconds")?,
        })
    }
}

/// 一条臂按重复次序排好的那几个数。
fn numbers_of_the_arm(
    numbers: &[RunNumbers],
    arm: LockstepArm,
    value: impl Fn(&RunNumbers) -> f64,
) -> Vec<f64> {
    let mut of_the_arm: Vec<&RunNumbers> = numbers.iter().filter(|run| run.arm == arm).collect();
    of_the_arm.sort_by_key(|run| run.repeat);
    of_the_arm.into_iter().map(value).collect()
}

fn joined_values(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format!("{value:.0}"))
        .collect::<Vec<String>>()
        .join(",")
}

/// 一档交替跑的判定行与阳性对照过没过。
struct JudgementOfATier {
    lines: Vec<(String, String)>,
    positive_control_holds_on_every_arm_run: bool,
}

fn judgement_of_a_lockstep_tier(
    tier: UnitAreaTier,
    numbers: &[RunNumbers],
    arms_run: &[LockstepArm],
) -> JudgementOfATier {
    let mut lines: Vec<(String, String)> = Vec::new();
    for arm in arms_run {
        let walls = numbers_of_the_arm(numbers, *arm, |run| run.gc_wall_nanoseconds);
        let processes = numbers_of_the_arm(numbers, *arm, |run| run.process_cpu_nanoseconds);
        lines.push((
            "arm_spread".to_string(),
            format!(
                "tier={} arm={} repeats={} {} {}",
                tier.name(),
                arm.name(),
                walls.len(),
                spread_fields("gc_wall_nanoseconds", &walls),
                spread_fields("process_cpu_nanoseconds", &processes)
            ),
        ));
    }
    let cpu_arm = LOCKSTEP_ARMS[0];
    let cpu_walls = numbers_of_the_arm(numbers, cpu_arm, |run| run.gc_wall_nanoseconds);
    let cpu_processes = numbers_of_the_arm(numbers, cpu_arm, |run| run.process_cpu_nanoseconds);
    let cpu_batches = numbers_of_the_arm(numbers, cpu_arm, |run| count_as_float(run.batches));
    let mut positive_control_holds_on_every_arm_run = true;
    for compared in arms_run
        .iter()
        .filter(|arm| arm.compute.uses_the_gpu() && arm.rejudging == RejudgingOfTheArm::NotRejudged)
    {
        let wall_differences = paired_differences(
            &cpu_walls,
            &numbers_of_the_arm(numbers, *compared, |run| run.gc_wall_nanoseconds),
        );
        lines.push((
            "wall_clock_comparison".to_string(),
            format!(
                "tier={} against={} cpu_minus_gpu_nanoseconds={} sign={}",
                tier.name(),
                compared.name(),
                joined_values(&wall_differences),
                sign_of_the_paired_differences(&wall_differences, 0.0).name()
            ),
        ));
        let savings = paired_differences(
            &cpu_processes,
            &numbers_of_the_arm(numbers, *compared, |run| run.process_cpu_nanoseconds),
        );
        let savings_per_batch: Vec<f64> = savings
            .iter()
            .zip(&cpu_batches)
            .map(|(saving, batches)| saving / batches.max(1.0))
            .collect();
        lines.push((
            "saving".to_string(),
            format!(
                "tier={} against={} cpu_minus_gpu_nanoseconds={} median={:.0} minimum={:.0} maximum={:.0} sign={}",
                tier.name(),
                compared.name(),
                joined_values(&savings),
                median_of(&savings),
                minimum_of(&savings),
                maximum_of(&savings),
                sign_of_the_paired_differences(&savings, 0.0).name()
            ),
        ));
        lines.push((
            "saving_per_batch".to_string(),
            format!(
                "tier={} against={} values={}",
                tier.name(),
                compared.name(),
                joined_values(&savings_per_batch)
            ),
        ));
    }
    for rejudged in arms_run
        .iter()
        .filter(|arm| arm.rejudging == RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu)
    {
        let plain = LockstepArm {
            compute: rejudged.compute,
            rejudging: RejudgingOfTheArm::NotRejudged,
        };
        let control = positive_control_of_an_arm(
            &numbers_of_the_arm(numbers, plain, |run| run.process_cpu_nanoseconds),
            &numbers_of_the_arm(numbers, *rejudged, |run| run.process_cpu_nanoseconds),
            &numbers_of_the_arm(numbers, *rejudged, |run| run.inner_rejudge_cpu_nanoseconds),
            rejudged
                .compute
                .uses_the_gpu()
                .then_some(cpu_processes.as_slice()),
        );
        positive_control_holds_on_every_arm_run &= control.holds();
        lines.push((
            "positive_control".to_string(),
            format!(
                "tier={} arm={} every_delta_positive={} median_delta_nanoseconds={:.0} median_inner_rejudge_cpu_nanoseconds={:.0} \
                 median_delta_within_the_band_of_the_inner_timing={} saving_with_rejudging_below_the_saving_in_every_pair={} holds={}",
                tier.name(),
                plain.name(),
                control.every_delta_positive,
                control.median_delta,
                control.median_inner_rejudge_cpu,
                control.median_delta_within_the_band_of_the_inner_timing,
                control
                    .saving_with_rejudging_below_the_saving_in_every_pair
                    .map_or_else(|| "not_a_gpu_arm".to_string(), |below| below.to_string()),
                control.holds()
            ),
        ));
    }
    JudgementOfATier {
        lines,
        positive_control_holds_on_every_arm_run,
    }
}

// ---- 一次重复（六条臂按轮换次序串着跑）与它的前提、作废、停机 ----

/// 一次重复的结局。封闭集合。
enum RepeatOutcome {
    /// 前提都过、锚点都成立：这几次跑计数。
    Done(Vec<LockstepRunSummary>),
    /// 作废（作废第 1–4、7 条）：同一档重排一次；`gpu_error` 是作废第 4 条那一种。
    Voided { reason: String, gpu_error: bool },
    /// 停机条款触发（或交主 agent 的那几种）：产物照打，退 3。
    Stopped { reason: String },
}

/// 同一次重复里各臂的录制摘要与读到的内容摘要逐个相同、读到的等于期望、多判对不上的条数都是 0（前提 I）。
fn the_runs_of_the_repeat_are_identical(runs: &[LockstepRunSummary]) -> bool {
    let Some(first_run) = runs.first() else {
        return false;
    };
    runs.iter().all(|run| {
        run.recording_digest == first_run.recording_digest
            && run.content_digest == first_run.content_digest
            && run.content_matches_the_expectation
            && run.rejudge.mismatches == 0
    })
}

/// 一次重复里各臂的批数都相等且至少 64（前提 N）。
fn the_runs_of_the_repeat_have_enough_batches(runs: &[LockstepRunSummary]) -> bool {
    let batches_of_each_run: Vec<u64> = runs.iter().map(|run| run.batches).collect();
    batches_are_equal_and_at_least_the_minimum(&batches_of_each_run)
}

/// 前提 N 的纯函数：每次跑的批数都等于头一次跑的、且不少于 [`MINIMUM_BATCHES_PER_RUN`]；一次跑都没有不算够。
fn batches_are_equal_and_at_least_the_minimum(batches_of_each_run: &[u64]) -> bool {
    batches_of_each_run.first().is_some_and(|first_batches| {
        batches_of_each_run
            .iter()
            .all(|batches| batches == first_batches && *batches >= MINIMUM_BATCHES_PER_RUN)
    })
}

/// 一档里作废的重复多于 2 次（作废第 3 条），或 GPU 臂出错放弃一批累计 3 次（作废第 4 条）：这一档停下交主 agent。
/// 出错放弃一批的那一次重复也算一次作废。
fn the_tier_stops_after_the_voided_repeats(voided_repeats: u32, gpu_errors: u32) -> bool {
    gpu_errors >= GPU_ERRORS_PER_TIER_TO_STOP || voided_repeats > VOIDED_REPEATS_PER_TIER_AT_MOST
}

/// 一次重复的各臂都跑完之后那几样核的读数。
#[derive(Clone, Copy, Debug, PartialEq)]
struct ChecksOfARepeat {
    /// 这次重复里各次跑期间本进程以外平均占的核数，取最大的那次（量 H）。
    busiest_host_other_cores: f64,
    /// 前提 I（交替跑：录制流与内容摘要各臂相同、读到的等于期望、多判对不上 0 条；并发跑：读到的等于期望）。
    is_identical: bool,
    /// 前提 N（并发跑不看批数，恒为真）。
    has_enough_batches: bool,
    class_one_anchors_hold: bool,
    class_two_anchors_hold: bool,
}

/// 那几样核的结局。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VerdictOfTheRepeatChecks {
    Counted,
    /// 作废第 3 条：同一档重排一次。
    Voided(&'static str),
    /// 停机条款第 4、5 条，前提 N 的阶梯走完，作废第 6 条（那一档作废、交主 agent）：产物照打，退 3。
    Stopped(&'static str),
}

/// 先看机器忙不忙（忙的那一次重复整组作废，别的读数不作数），再依次看前提 I、前提 N、甲类、乙类（停机条款第 4 条看的是甲类那一样）。
fn verdict_of_the_repeat_checks(checks: ChecksOfARepeat) -> VerdictOfTheRepeatChecks {
    if checks.busiest_host_other_cores > HOST_LOAD_CORES_AT_MOST {
        VerdictOfTheRepeatChecks::Voided("the_host_was_busy")
    } else if !checks.is_identical {
        VerdictOfTheRepeatChecks::Stopped("the_arms_of_the_repeat_did_not_write_and_read_the_same")
    } else if !checks.has_enough_batches {
        VerdictOfTheRepeatChecks::Stopped(
            "fewer_than_64_batches_or_unequal_batches_climb_the_user_change_ladder",
        )
    } else if !checks.class_one_anchors_hold {
        VerdictOfTheRepeatChecks::Stopped("a_class_one_anchor_does_not_hold")
    } else if !checks.class_two_anchors_hold {
        VerdictOfTheRepeatChecks::Stopped("a_class_two_anchor_does_not_hold_the_tier_is_void")
    } else {
        VerdictOfTheRepeatChecks::Counted
    }
}

/// 第 `repeat` 次重复的轮换次序：从第 `repeat` 条臂起（登记第十四节）。
fn arms_in_the_rotation_of_the_repeat(arms_run: &[LockstepArm], repeat: u32) -> Vec<LockstepArm> {
    let start = usize::try_from(repeat).expect("重复次数") % arms_run.len().max(1);
    arms_run[start..]
        .iter()
        .chain(&arms_run[..start])
        .copied()
        .collect()
}

fn run_one_lockstep_repeat(
    setting: &LockstepRunSetting<'_>,
    arms_run: &[LockstepArm],
    repeat: u32,
    session_number: u64,
    product: &mut ProductWriter,
) -> RepeatOutcome {
    let tier = setting.image.tier;
    let gpu_models: String = setting
        .discrete_gpus
        .iter()
        .map(|gpu| field_value(&gpu.model))
        .collect::<Vec<String>>()
        .join("+");
    let mut runs: Vec<LockstepRunSummary> = Vec::new();
    for arm in arms_in_the_rotation_of_the_repeat(arms_run, repeat) {
        let models_of_the_arm = if arm.compute.uses_the_gpu() {
            gpu_models.as_str()
        } else {
            "none"
        };
        let mut kept: Option<LockstepRunSummary> = None;
        for attempt in 0..2 {
            eprintln!(
                "E202 交替跑 {} 第 {repeat} 次重复 {} 第 {attempt} 次尝试",
                tier.name(),
                arm.name()
            );
            let summary = match run_lockstep_once(setting, arm, repeat) {
                Ok(summary) => summary,
                Err(reason) => return RepeatOutcome::Stopped { reason },
            };
            product.emit(
                "run",
                &summary.fields(tier, session_number, models_of_the_arm),
            );
            for window in &summary.windows_failing_the_inclusion_checks {
                eprintln!(
                    "E202 交替跑 {} 第 {repeat} 次重复 {} 第 {attempt} 次尝试判不过包含自检的窗口：{window:?}",
                    tier.name(),
                    arm.name()
                );
            }
            for line in summary.forced_fallback.choice_lines(&format!(
                "tier={} arm={} repeat={repeat} session={session_number}",
                tier.name(),
                arm.name()
            )) {
                product.emit("forced_choice", &line);
            }
            if summary
                .gpu_work_anchors
                .batches_abandoned_after_a_compute_error
                > 0
            {
                return RepeatOutcome::Voided {
                    reason: format!("{}_abandoned_a_batch_after_a_compute_error", arm.name()),
                    gpu_error: true,
                };
            }
            let is_void = summary.void_reason.is_some();
            kept = Some(summary);
            if !is_void {
                break;
            }
        }
        match kept {
            Some(summary) if summary.void_reason.is_none() => runs.push(summary),
            Some(summary) => {
                return RepeatOutcome::Voided {
                    reason: format!(
                        "{}_void_twice_{}",
                        arm.name(),
                        summary.void_reason.unwrap_or_default()
                    ),
                    gpu_error: false,
                }
            }
            None => {
                return RepeatOutcome::Stopped {
                    reason: "no_attempt_ran".to_string(),
                }
            }
        }
    }
    let busiest_host = runs
        .iter()
        .map(|run| run.host_other_cores)
        .fold(0.0, f64::max);
    let is_identical = the_runs_of_the_repeat_are_identical(&runs);
    let has_enough_batches = the_runs_of_the_repeat_have_enough_batches(&runs);
    let class_one_anchors_hold = runs.iter().all(LockstepRunSummary::class_one_anchors_hold);
    let class_two_anchors_hold = runs.iter().all(LockstepRunSummary::class_two_anchors_hold);
    product.emit(
        "premise",
        &format!(
            "tier={} repeat={repeat} session={session_number} identical={is_identical} enough_batches={has_enough_batches} \
             batches={} class_one_anchors_hold={class_one_anchors_hold} class_two_anchors_hold={class_two_anchors_hold} \
             busiest_host_other_cores={busiest_host:.2}",
            tier.name(),
            runs.iter().map(|run| run.batches.to_string()).collect::<Vec<String>>().join(",")
        ),
    );
    match verdict_of_the_repeat_checks(ChecksOfARepeat {
        busiest_host_other_cores: busiest_host,
        is_identical,
        has_enough_batches,
        class_one_anchors_hold,
        class_two_anchors_hold,
    }) {
        VerdictOfTheRepeatChecks::Counted => {}
        VerdictOfTheRepeatChecks::Voided(reason) => {
            return RepeatOutcome::Voided {
                reason: reason.to_string(),
                gpu_error: false,
            }
        }
        VerdictOfTheRepeatChecks::Stopped(reason) => {
            return RepeatOutcome::Stopped {
                reason: reason.to_string(),
            }
        }
    }
    product.emit(
        "repeat_done",
        &format!(
            "tier={} repeat={repeat} session={session_number}",
            tier.name()
        ),
    );
    RepeatOutcome::Done(runs)
}

// ---- 子命令共用的开头：产物、开跑检查、crates 版本（停机条款第 2 条：与产物里已有的比）、卡 ----

/// 子命令开头读到的：产物里已有的结果行、这是第几次起这条命令、合起来的 crates 摘要、本进程看得到的独显，
/// 与枚举它们的那一份进程级 Vulkan 实例（这条命令里开卡都用它，不另建；不枚举独显的命令不建，是 `None`）。
struct OpeningOfTheCommand {
    lines_already_in_the_product: Vec<KindAndFieldsOfAResultLine>,
    session_number: u64,
    registered_heavy_threads: usize,
    discrete_gpus: Vec<DiscreteGpuSeenByThisProcess>,
    vulkan_instance: Option<VulkanInstanceOfThisProcess>,
}

/// 这条命令要不要枚举本进程看得到的独显（几何核不要卡）：要的带着这个进程的那一份 Vulkan 实例（`main` 照要卡的子命令建，一个进程一份，
/// 枚举与这条命令里开卡都用它）。封闭集合。
#[derive(Clone, Copy, Debug)]
enum GpusOfTheCommand<'process> {
    EnumeratedOn(&'process VulkanInstanceOfThisProcess),
    NotNeeded,
}

fn opening_of_the_command(
    product: &mut ProductWriter,
    product_path: &Path,
    command_fields: &str,
    gpus: GpusOfTheCommand<'_>,
) -> Result<OpeningOfTheCommand, i32> {
    let lines_already_in_the_product =
        result_lines_already_in_the_product(product_path).map_err(|reason| {
            eprintln!("{reason}");
            EXIT_RUN_CONDITION_NOT_MET
        })?;
    let session_number = 1 + u64::try_from(
        lines_already_in_the_product
            .iter()
            .filter(|(name, _fields)| name == "session_start")
            .count(),
    )
    .expect("次数");
    product.emit(
        "session_start",
        &format!("session={session_number} {command_fields}"),
    );
    let registered_heavy_threads = checks_before_the_run(product)?;
    let combined = emit_the_crates_version(product).map_err(|reason| {
        product.emit(
            "crates_version",
            &format!("readable=false reason={}", field_value(&reason)),
        );
        EXIT_RUN_CONDITION_NOT_MET
    })?;
    if the_crates_changed_since_the_earlier_commands(&lines_already_in_the_product, &combined) {
        product.emit(
            "stopped",
            "clause=crates_changed_between_the_commands_of_the_segment reason=产物里已有的crates版本与这一次的不同",
        );
        return Err(EXIT_STOPPED);
    }
    let (vulkan_instance, discrete_gpus) = match gpus {
        GpusOfTheCommand::EnumeratedOn(vulkan_instance) => {
            let discrete_gpus =
                discrete_gpus_seen_by_this_process(vulkan_instance).map_err(|reason| {
                    product.emit(
                        "gpus",
                        &format!("enumerated=false reason={}", field_value(&reason)),
                    );
                    EXIT_RUN_CONDITION_NOT_MET
                })?;
            (Some(vulkan_instance.clone()), discrete_gpus)
        }
        GpusOfTheCommand::NotNeeded => (None, Vec::new()),
    };
    product.emit(
        "gpus",
        &format!(
            "enumerated={} count={} pci_bus_addresses={} models={}",
            vulkan_instance.is_some(),
            discrete_gpus.len(),
            field_value(
                &discrete_gpus
                    .iter()
                    .map(|gpu| gpu.pci_bus_address.as_str())
                    .collect::<Vec<&str>>()
                    .join(",")
            ),
            field_value(
                &discrete_gpus
                    .iter()
                    .map(|gpu| gpu.model.as_str())
                    .collect::<Vec<&str>>()
                    .join(",")
            )
        ),
    );
    Ok(OpeningOfTheCommand {
        lines_already_in_the_product,
        session_number,
        registered_heavy_threads,
        discrete_gpus,
        vulkan_instance,
    })
}

/// 打 `name=tier_device` 一行：这一档登记的每块盘字节与 mkfs 认的最小一块盘；mkfs 不认交回 `false`（第十一节作废第 6 条：那一档作废，交主 agent）。
fn emit_whether_the_make_filesystem_accepts_the_tier(
    product: &mut ProductWriter,
    tier: UnitAreaTier,
) -> bool {
    let is_accepted = the_make_filesystem_accepts_the_device_bytes(tier.device_bytes());
    product.emit(
        "tier_device",
        &format!(
            "tier={} device_bytes={} smallest_device_bytes_the_make_filesystem_accepts={} accepted_by_the_make_filesystem={is_accepted}",
            tier.name(),
            tier.device_bytes(),
            smallest_device_bytes_the_make_filesystem_accepts()
        ),
    );
    if !is_accepted {
        product.emit(
            "tier_voided",
            &format!(
                "tier={} clause=class_two_anchor_the_make_filesystem_refuses_the_registered_device_bytes reason=登记的每块盘字节小于默认环长的四倍，交主agent",
                tier.name()
            ),
        );
    }
    is_accepted
}

/// 造起步镜像并打 `name=construction` 一行；几何没造成交回 `false`（登记第五节「负载」第 4 条：交主 agent 改负载重造）。
fn emit_the_construction(product: &mut ProductWriter, image: &StartingImage) -> bool {
    let construction = &image.construction;
    let geometry = &construction.geometry;
    let histogram: Vec<String> = geometry
        .live_slot_histogram
        .iter()
        .map(|(live, segments)| format!("{live}:{segments}"))
        .collect();
    product.emit(
        "construction",
        &format!(
            "tier={} device_bytes={} live_data_units={} live_data_units_found={} construction_seconds={:.3} publishes={} \
             floor_raises={} release_rounds={} spacer_units={} alignment_spacer_units={} every_spacer_slot_released={} \
             units_per_file={} live_data_files={} spacer_files={} inode_records_before_the_data_mount={} \
             inode_records_at_the_peak={} inode_leaf_containers_at_the_peak={} inode_records_one_level_tree_holds={} \
             unit_area_segments={} empty_segments={} mapped_units={} registered_fragment_segments={} written_segments={} \
             written_segments_with_half_live={} choice=forced_fallback forced_segment_start_slot={} forced_live_slots={} \
             forced_holds_only_data_units={} greedy_segment_start_slot={} greedy_live_slots={} greedy_holds_only_data_units={} \
             live_slot_histogram={} geometry_formed={}",
            image.tier.name(),
            image.tier.device_bytes(),
            image.tier.live_data_units(),
            construction.live_data_units_found,
            construction.construction_seconds,
            construction.publishes,
            construction.floor_raises,
            construction.release_rounds,
            construction.spacer_units,
            construction.alignment_spacer_units,
            construction.is_every_spacer_slot_released,
            LIVE_DATA_UNITS_PER_FILE,
            construction.live_data_files,
            construction.spacer_files,
            construction.inode_records_before_the_data_mount,
            construction.inode_records_at_the_peak,
            construction.inode_leaf_containers_at_the_peak,
            inode_records_one_level_tree_holds(),
            image.tier.unit_area_segments(),
            construction.empty_segments,
            construction.mapped_units,
            image.tier.fragment_segments(),
            geometry.written_segments,
            geometry.written_segments_with_half_live,
            geometry
                .forced
                .start_slot
                .map_or_else(|| "none".to_string(), |slot| slot.to_string()),
            geometry.forced.live_slots,
            geometry.forced.holds_only_data_units,
            geometry
                .greedy
                .start_slot
                .map_or_else(|| "none".to_string(), |slot| slot.to_string()),
            geometry.greedy.live_slots,
            geometry.greedy.holds_only_data_units,
            histogram.join(","),
            geometry.is_formed()
        ),
    );
    for timing in &construction.phases {
        product.emit(
            "construction_phase",
            &format!("tier={} {}", image.tier.name(), timing.fields()),
        );
    }
    geometry.is_formed()
}

// ---- 子命令 lockstep（登记第五节「交替跑」，补问题单第 1、2、4、5 行）----

struct LockstepCommand {
    tier: UnitAreaTier,
    user_changes: u64,
    batch_units: usize,
    arms_requested: Vec<LockstepArm>,
    product: PathBuf,
}

/// 续跑：产物里这一档、这一组设定已做完的重复（`name=repeat_done`）与它们 `void=no` 的 `name=run` 行读回的数。
fn repeats_already_done(
    lines: &[(String, BTreeMap<String, String>)],
    tier: UnitAreaTier,
) -> (BTreeMap<u32, String>, Vec<RunNumbers>, u32) {
    let done: BTreeMap<u32, String> = lines
        .iter()
        .filter(|(name, fields)| {
            name == "repeat_done" && fields.get("tier").map(String::as_str) == Some(tier.name())
        })
        .filter_map(|(_name, fields)| {
            Some((
                fields.get("repeat")?.parse::<u32>().ok()?,
                fields.get("session")?.clone(),
            ))
        })
        .collect();
    let numbers: Vec<RunNumbers> = lines
        .iter()
        .filter(|(name, fields)| {
            name == "run"
                && fields.get("tier").map(String::as_str) == Some(tier.name())
                && fields.get("void").map(String::as_str) == Some("no")
                && fields
                    .get("repeat")
                    .and_then(|repeat| repeat.parse::<u32>().ok())
                    .and_then(|repeat| done.get(&repeat))
                    == fields.get("session")
        })
        .filter_map(|(_name, fields)| RunNumbers::of_the_fields(fields))
        .collect();
    let next_repeat = lines
        .iter()
        .filter(|(name, _fields)| name == "run" || name == "repeat_done")
        .filter_map(|(_name, fields)| fields.get("repeat")?.parse::<u32>().ok())
        .max()
        .map_or(0, |repeat| repeat + 1);
    (done, numbers, next_repeat)
}

fn lockstep_command(
    command: &LockstepCommand,
    vulkan_instance: &VulkanInstanceOfThisProcess,
) -> i32 {
    let mut product = match ProductWriter::appending_to(&command.product) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let command_fields = format!(
        "command=lockstep reading=occupancy tier={} user_changes={} batch_units={} predicted_publishes_of_one_intent={} arms={}",
        command.tier.name(),
        command.user_changes,
        command.batch_units,
        publishes_of_one_intent(command.batch_units),
        command.arms_requested.iter().map(|arm| arm.name()).collect::<Vec<String>>().join(",")
    );
    let opening = match opening_of_the_command(
        &mut product,
        &command.product,
        &command_fields,
        GpusOfTheCommand::EnumeratedOn(vulkan_instance),
    ) {
        Ok(opening) => opening,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    if !emit_whether_the_make_filesystem_accepts_the_tier(&mut product, command.tier) {
        product.finish();
        return EXIT_STOPPED;
    }
    let mut arms_run: Vec<LockstepArm> = Vec::new();
    for arm in &command.arms_requested {
        match reason_the_arm_is_not_run(arm.compute, opening.discrete_gpus.len()) {
            Some(reason) => product.emit(
                "not_run",
                &format!(
                    "tier={} arm={} reason={reason}",
                    command.tier.name(),
                    arm.name()
                ),
            ),
            None => arms_run.push(*arm),
        }
    }
    if opening.discrete_gpus.is_empty() {
        for arm in &arms_run {
            product.emit(
                "not_run",
                &format!(
                    "tier={} arm={} reason=本进程看得到的独立显卡一张都没有",
                    command.tier.name(),
                    arm.name()
                ),
            );
        }
        product.finish();
        return EXIT_NO_ARM_RAN;
    }
    let image = construct_the_starting_image(command.tier);
    if !emit_the_construction(&mut product, &image) {
        product.emit(
            "stopped",
            "clause=geometry_not_formed reason=几何没造成，改负载重造交主agent",
        );
        product.finish();
        return EXIT_STOPPED;
    }
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let setting = LockstepRunSetting {
        image: &image,
        discrete_gpus: &opening.discrete_gpus,
        memory_leaked: &memory_leaked,
        vulkan_instance: opening.vulkan_instance.as_ref(),
        user_changes: command.user_changes,
        batch_units: command.batch_units,
    };
    let (done, mut numbers, mut next_repeat) =
        repeats_already_done(&opening.lines_already_in_the_product, command.tier);
    let mut repeats_done = u32::try_from(done.len()).expect("重复次数");
    product.emit(
        "resume",
        &format!(
            "tier={} repeats_already_done={repeats_done} next_repeat={next_repeat}",
            command.tier.name()
        ),
    );
    let mut voided_repeats = 0_u32;
    let mut gpu_errors = 0_u32;
    while repeats_done < REPEATS_PER_ARM {
        let repeat = next_repeat;
        next_repeat += 1;
        match run_one_lockstep_repeat(
            &setting,
            &arms_run,
            repeat,
            opening.session_number,
            &mut product,
        ) {
            RepeatOutcome::Done(runs) => {
                numbers.extend(runs.iter().map(RunNumbers::of_the_summary));
                repeats_done += 1;
            }
            RepeatOutcome::Voided { reason, gpu_error } => {
                voided_repeats += 1;
                gpu_errors += u32::from(gpu_error);
                product.emit(
                    "repeat_voided",
                    &format!(
                        "tier={} repeat={repeat} reason={}",
                        command.tier.name(),
                        field_value(&reason)
                    ),
                );
                if the_tier_stops_after_the_voided_repeats(voided_repeats, gpu_errors) {
                    product.emit("stopped", &format!("clause=too_many_voided_repeats voided={voided_repeats} gpu_errors={gpu_errors}"));
                    product.finish();
                    return EXIT_STOPPED;
                }
            }
            RepeatOutcome::Stopped { reason } => {
                product.emit(
                    "stopped",
                    &format!(
                        "tier={} repeat={repeat} clause={}",
                        command.tier.name(),
                        field_value(&reason)
                    ),
                );
                product.finish();
                return EXIT_STOPPED;
            }
        }
    }
    let judgement = judgement_of_a_lockstep_tier(command.tier, &numbers, &arms_run);
    for (name, fields) in &judgement.lines {
        product.emit(name, fields);
    }
    product.emit(
        "verdict",
        &format!(
            "tier={} repeats={repeats_done} arms_run={} positive_control_holds_on_every_arm_run={} heavy_threads_registered={}",
            command.tier.name(),
            arms_run.len(),
            judgement.positive_control_holds_on_every_arm_run,
            opening.registered_heavy_threads
        ),
    );
    product.finish();
    if judgement.positive_control_holds_on_every_arm_run {
        EXIT_SUCCESS
    } else {
        EXIT_POSITIVE_CONTROL_FAILED
    }
}

// ---- 并发跑（登记第五节「两种跑法」的并发跑，补问题单第 3 行；量 F、量 F 底，第七节甲类第 5 行，第十条第 4 行）----

/// 并发跑一次的用户改动次数（登记第五节「前台那串用户改动」）。
const CONCURRENT_USER_CHANGES: u64 = 256;
/// 前台计划时刻的间隔：主取样点与几何敏感性那一点（登记第五节「并发跑」、第八节）。
const FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_MAIN_SAMPLING_POINT: u64 = 50;
const FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_SENSITIVITY_POINT: u64 = 500;
/// GC 线程一次调用一次整理发布都没发时，在条件变量上至多等这么久（登记第五节「并发跑」）。
const CONCURRENT_GC_IDLE_WAIT: Duration = Duration::from_millis(1);
/// 第十条第 4 行：一次用户改动「拿到锁 − 置上标志」不许超过（最长一批的挂钟 + 最长一次 discard）的这么多倍。
const FLAG_TO_LOCK_WAIT_PER_LONGEST_BATCH_AND_DISCARD_AT_MOST: u32 = 2;
/// 量 F 报的高分位：第 99 百分位，按最近秩取（排好序的第 ⌈99 × n ÷ 100⌉ 个，从 1 数）。
const HIGH_PERCENTILE_PER_HUNDRED: usize = 99;

/// 并发跑的三条臂（登记第五节「真实基线」：CPU 臂、GPU 点名一张卡臂与不跑 GC 臂）。封闭集合，次序是轮换的基准次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ConcurrentArm {
    Cpu,
    GpuOnTheCardWithTheLowestPciAddress,
    /// GC 线程不起，同一串前台改动（量 F 底）。挂载照 CPU 臂的设置。
    WithoutCompaction,
}

const CONCURRENT_ARMS: [ConcurrentArm; 3] = [
    ConcurrentArm::Cpu,
    ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
    ConcurrentArm::WithoutCompaction,
];

impl ConcurrentArm {
    const fn name(self) -> &'static str {
        match self {
            ConcurrentArm::Cpu => "cpu",
            ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress => "gpu_one_card",
            ConcurrentArm::WithoutCompaction => "without_compaction",
        }
    }

    fn of_the_name(name: &str) -> Option<ConcurrentArm> {
        CONCURRENT_ARMS.into_iter().find(|arm| arm.name() == name)
    }

    /// 挂载用哪一种后台计算（不跑 GC 臂照 CPU 臂挂）。
    const fn compute_of_the_arm(self) -> ComputeOfTheArm {
        match self {
            ConcurrentArm::Cpu | ConcurrentArm::WithoutCompaction => ComputeOfTheArm::Cpu,
            ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress => {
                ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress
            }
        }
    }

    const fn runs_the_compaction(self) -> bool {
        match self {
            ConcurrentArm::Cpu | ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress => true,
            ConcurrentArm::WithoutCompaction => false,
        }
    }

    /// 开卡与多判包装照交替跑那一份的臂（并发跑不多判）。
    const fn as_a_lockstep_arm(self) -> LockstepArm {
        LockstepArm {
            compute: self.compute_of_the_arm(),
            rejudging: RejudgingOfTheArm::NotRejudged,
        }
    }
}

/// 一次用户改动的四个时刻，都从这次跑开跑那一刻起算（登记第五节「计时与计数」第 3 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UserChangeTimings {
    planned: Duration,
    flag_set: Duration,
    lock_acquired: Duration,
    publish_returned: Duration,
}

/// 量 F：延迟 = 发布返回 − 计划时刻（开环，落后了照算，登记第一节「前台 fsync」；第九节变异「延迟从拿到锁算起」改的是这里）。
/// 读出负的交 `None`（作废第 1 条）。
fn latency_of_the_user_change(timings: UserChangeTimings) -> Option<Duration> {
    timings.publish_returned.checked_sub(timings.planned)
}

/// 锁里发布用了多久 = 发布返回 − 拿到锁。
fn publish_under_the_lock_of_the_user_change(timings: UserChangeTimings) -> Option<Duration> {
    timings.publish_returned.checked_sub(timings.lock_acquired)
}

/// 第十条第 4 行的读数：拿到锁 − 置上标志。
fn wait_from_the_flag_to_the_lock(timings: UserChangeTimings) -> Option<Duration> {
    timings.lock_acquired.checked_sub(timings.flag_set)
}

/// 最近秩的高分位（第 99 百分位）：`sorted_values` 从小到大排好、至少一个。
fn high_percentile_of_the_sorted(sorted_values: &[f64]) -> f64 {
    let rank = (HIGH_PERCENTILE_PER_HUNDRED * sorted_values.len()).div_ceil(100);
    sorted_values[rank.max(1) - 1]
}

/// 一份延迟分布：条数、中位、第 99 百分位、最大（登记第六节量 F）。
#[derive(Clone, Copy, Debug, PartialEq)]
struct LatencyDistribution {
    count: usize,
    median: f64,
    high_percentile: f64,
    maximum: f64,
}

/// 从一组延迟的分布里取一个统计量（第 99 百分位、最大值）。
type StatisticOfALatencyDistribution = fn(LatencyDistribution) -> f64;

fn latency_distribution_of(values_in_nanoseconds: &[f64]) -> Option<LatencyDistribution> {
    if values_in_nanoseconds.is_empty() {
        return None;
    }
    let mut sorted_values = values_in_nanoseconds.to_vec();
    sorted_values.sort_by(f64::total_cmp);
    Some(LatencyDistribution {
        count: sorted_values.len(),
        median: median_of(&sorted_values),
        high_percentile: high_percentile_of_the_sorted(&sorted_values),
        maximum: maximum_of(&sorted_values),
    })
}

/// `name=` 行里一份分布的四个字段（没有数时四个都写 `none`）。
fn distribution_fields(name: &str, distribution: Option<LatencyDistribution>) -> String {
    match distribution {
        Some(distribution) => format!(
            "{name}_count={} {name}_median={:.0} {name}_ninety_ninth_percentile={:.0} {name}_maximum={:.0}",
            distribution.count,
            distribution.median,
            distribution.high_percentile,
            distribution.maximum
        ),
        None => format!(
            "{name}_count=0 {name}_median=none {name}_ninety_ninth_percentile=none {name}_maximum=none"
        ),
    }
}

/// 并发跑里调用方的钟与前台：钟是真的单调钟（`CLOCK_MONOTONIC_RAW`，从开跑起算），`foreground_fsync` 读前台置的那个标志，每问一次记下问的时刻与答的（窗口外读）。
struct CallerOfTheConcurrentRun<'run> {
    started: StartOnTheMonotonicRawClock,
    fsync_is_waiting: &'run AtomicBool,
    queries_of_the_call: RefCell<Vec<(Duration, ForegroundFsync)>>,
}

impl CallerOfTheBackgroundCompaction for CallerOfTheConcurrentRun<'_> {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(self.started.elapsed())
    }

    fn foreground_fsync(&self) -> ForegroundFsync {
        let answer = if self.fsync_is_waiting.load(Ordering::SeqCst) {
            ForegroundFsync::Waiting
        } else {
            ForegroundFsync::NoneWaiting
        };
        self.queries_of_the_call
            .borrow_mut()
            .push((self.started.elapsed(), answer));
        answer
    }
}

/// 并发跑里一次 GC 调用记下的：整理记账（同交替跑那一份），调用开始与返回的时刻，这次调用里每一次问前台的时刻与答的。
struct ConcurrentCallRecord {
    call: GcCallRecord,
    started_at: Duration,
    returned_at: Duration,
    fsync_queries: Vec<(Duration, ForegroundFsync)>,
}

/// 第七节甲类第 5 行：答了「在等」的那次调用只答了这一次、它是这次调用问的最后一次（答了之后不再发批），停机原因是让路。
fn the_call_yielded_after_answering_waiting(call: &ConcurrentCallRecord) -> bool {
    let waiting_answers = call
        .fsync_queries
        .iter()
        .filter(|(_at, answer)| *answer == ForegroundFsync::Waiting)
        .count();
    match waiting_answers {
        0 => true,
        1 => {
            call.fsync_queries
                .last()
                .is_some_and(|(_at, answer)| *answer == ForegroundFsync::Waiting)
                && call.call.stop_name == CompactionRunStop::YieldedToAForegroundFsync.name()
        }
        _more_than_one => false,
    }
}

/// 甲类第 5 行不成立的调用数。
fn calls_not_yielding_after_answering_waiting(calls: &[ConcurrentCallRecord]) -> u64 {
    let not_yielding = calls
        .iter()
        .filter(|call| !the_call_yielded_after_answering_waiting(call))
        .count();
    u64::try_from(not_yielding).expect("调用数")
}

/// 第八节轨迹「前台『在等』标志被答成『在等』的次数」：一次调用里的峰值、答成「在等」的调用次数、期末值（最后一次调用的）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct WaitingAnswersTrajectory {
    peak_in_one_call: u64,
    calls_answered_waiting: u64,
    in_the_last_call: u64,
}

fn waiting_answers_trajectory(calls: &[ConcurrentCallRecord]) -> WaitingAnswersTrajectory {
    let answers_of_each_call: Vec<u64> = calls
        .iter()
        .map(|call| {
            let waiting = call
                .fsync_queries
                .iter()
                .filter(|(_at, answer)| *answer == ForegroundFsync::Waiting)
                .count();
            u64::try_from(waiting).expect("次数")
        })
        .collect();
    WaitingAnswersTrajectory {
        peak_in_one_call: answers_of_each_call.iter().copied().max().unwrap_or(0),
        calls_answered_waiting: u64::try_from(
            answers_of_each_call
                .iter()
                .filter(|answers| **answers > 0)
                .count(),
        )
        .expect("调用数"),
        in_the_last_call: answers_of_each_call.last().copied().unwrap_or(0),
    }
}

/// 第十条第 4 行的上界取法：核心层每一批之前问一次前台，一次调用里相邻两次「问」之间（连同调用开始到头一次问）的那一段至多是一批
/// （连同选段与签发）的挂钟；最后一次问到调用返回（一次都没问的调用是整次调用）那一段含调用末尾的 discard。交回（最长一批，最长收尾）。
fn longest_batch_and_longest_tail_of_the_calls(
    calls: &[ConcurrentCallRecord],
) -> (Duration, Duration) {
    let mut longest_batch = Duration::ZERO;
    let mut longest_tail = Duration::ZERO;
    for call in calls {
        let mut previous = call.started_at;
        for (asked_at, _answer) in &call.fsync_queries {
            longest_batch = longest_batch.max(asked_at.saturating_sub(previous));
            previous = *asked_at;
        }
        longest_tail = longest_tail.max(call.returned_at.saturating_sub(previous));
    }
    (longest_batch, longest_tail)
}

/// 第十条第 4 行：前台等锁比（最长一批 + 最长收尾）的两倍还久（条款与实现对不上，停机条款第 4 条）。
fn the_wait_exceeds_twice_the_longest_batch_and_tail(
    longest_wait_from_the_flag_to_the_lock: Duration,
    longest_batch: Duration,
    longest_tail: Duration,
) -> bool {
    longest_wait_from_the_flag_to_the_lock
        > (longest_batch + longest_tail) * FLAG_TO_LOCK_WAIT_PER_LONGEST_BATCH_AND_DISCARD_AT_MOST
}

/// 第十条第 4 行这一次跑的结论（登记第十二节修订第 6 条）：最长一批与最长收尾取的是上界（相邻两次问前台之间、最后一次问到返回），
/// 判得比登记口径宽；打中照登记判，没打中只说「上界口径下没打中」，不说登记口径下没打中。
fn yield_bound_failure_clause_reading(
    the_foreground_waited_past_the_yield_bound: bool,
) -> &'static str {
    if the_foreground_waited_past_the_yield_bound {
        "hit"
    } else {
        "not_hit_under_the_upper_bound_reading"
    }
}

/// 并发跑 GC 线程放锁之后让一下（登记第十二节修订第 7 条）；产物头一行照它写 `gc_thread_yields_after_unlock=`。
const GC_THREAD_YIELDS_AFTER_UNLOCK: bool = true;

/// 一次并发跑记下的（窗口外读）。
struct ConcurrentRunRecords {
    calls: Vec<ConcurrentCallRecord>,
    forced_fallback: ForcedFallbackReading,
    timings: Vec<UserChangeTimings>,
    the_opening_thread_was_running_at_the_first_call: bool,
    state: SessionAndDevices,
    write_generation_of_each_unit: Vec<u64>,
}

/// 一次并发跑的节奏：几次用户改动、隔多久一次、起不起 GC 线程。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConcurrentRunPacing {
    user_changes: u64,
    foreground_interval: Duration,
    runs_the_compaction: bool,
}

/// 并发跑一次（登记第五节「两种跑法」）：前台线程照计划时刻发用户改动，发之前置上「有 fsync 在等」、拿到会话锁、发布返回之后撤下；
/// GC 线程循环拿会话锁调一次后台入口、放锁，这次调用一次整理发布都没发就在条件变量上等下一次用户改动发成或 1 毫秒，前台做完就停。
/// 会话与两块盘放在一把锁里，前台与 GC 不同时动它们。
fn concurrent_threads(
    session_and_devices_at_the_start: SessionAndDevices,
    lowest_live_data_inode: u64,
    write_generation_of_each_unit_at_the_start: Vec<u64>,
    first_write_time_seconds: u64,
    run_pacing: ConcurrentRunPacing,
    forced_fallback_of_the_run: ForcedFallbackOfTheRun,
) -> ConcurrentRunRecords {
    let session_and_devices = Mutex::new(session_and_devices_at_the_start);
    let fsync_is_waiting = AtomicBool::new(false);
    let foreground_is_done = AtomicBool::new(false);
    let user_changes_published = Mutex::new(0_u64);
    let user_change_published_condition = Condvar::new();
    // 前台的计划时刻、置标志、拿到锁、发布返回与 GC 调用的开始、返回、问前台都读这一个起点：跨度挂钟与第十条第 4 行拿它们互相比。
    let started = StartOnTheMonotonicRawClock::now();
    let (
        (calls, the_opening_thread_was_running_at_the_first_call, forced_fallback),
        timings,
        write_generation_of_each_unit,
    ) = std::thread::scope(|scope| {
        let foreground = scope.spawn(|| {
            let mut write_generation_of_each_unit = write_generation_of_each_unit_at_the_start;
            let mut timings: Vec<UserChangeTimings> = Vec::new();
            for change_number in 0..run_pacing.user_changes {
                let planned = run_pacing.foreground_interval
                    * u32::try_from(change_number).expect("用户改动次数装得进 u32");
                if let Some(ahead) = planned.checked_sub(started.elapsed()) {
                    std::thread::sleep(ahead);
                }
                fsync_is_waiting.store(true, Ordering::SeqCst);
                let flag_set = started.elapsed();
                let mut guard = session_and_devices.lock().expect("会话锁没在别处 panic 过");
                let lock_acquired = started.elapsed();
                let state = &mut *guard;
                publish_the_user_change(
                    &mut state.session,
                    &mut state.devices,
                    lowest_live_data_inode,
                    change_number,
                    &mut write_generation_of_each_unit,
                    first_write_time_seconds + change_number,
                );
                let publish_returned = started.elapsed();
                fsync_is_waiting.store(false, Ordering::SeqCst);
                drop(guard);
                *user_changes_published
                    .lock()
                    .expect("计数锁没在别处 panic 过") += 1;
                user_change_published_condition.notify_all();
                timings.push(UserChangeTimings {
                    planned,
                    flag_set,
                    lock_acquired,
                    publish_returned,
                });
            }
            foreground_is_done.store(true, Ordering::SeqCst);
            user_change_published_condition.notify_all();
            (timings, write_generation_of_each_unit)
        });
        let gc = run_pacing.runs_the_compaction.then(|| {
            scope.spawn(|| {
                // 强制选段后备（登记第十二节修订第 8 条）：点名在拿着会话锁、调后台入口之前做，点名用的挂钟另报（`forced_choice_wall_nanoseconds`）。
                let mut forced_fallback = forced_fallback_of_the_run;
                let caller = CallerOfTheConcurrentRun {
                    started,
                    fsync_is_waiting: &fsync_is_waiting,
                    queries_of_the_call: RefCell::new(Vec::new()),
                };
                let pacing = pacing_of_the_measuring_mount();
                let mut calls: Vec<ConcurrentCallRecord> = Vec::new();
                let mut the_opening_thread_was_running_at_the_first_call = false;
                while !foreground_is_done.load(Ordering::SeqCst) {
                    let published_before_the_call = *user_changes_published
                        .lock()
                        .expect("计数锁没在别处 panic 过");
                    if calls.is_empty() {
                        the_opening_thread_was_running_at_the_first_call =
                            the_gpu_opening_thread_is_still_running().unwrap_or(true);
                    }
                    let mut guard = session_and_devices.lock().expect("会话锁没在别处 panic 过");
                    let state = &mut *guard;
                    let empty_segments_before_the_call =
                        empty_segments_of_every_device(&state.session);
                    let intents_issued_before_the_call =
                        forced_fallback.before_the_call(&mut state.session, &state.devices);
                    let started_at = started.elapsed();
                    let call = state
                        .session
                        .run_compaction_in_the_background(&mut state.devices, pacing, &caller)
                        .expect("盘表是挂载核过的那一份、现行那一版有树：后台入口不拒");
                    let returned_at = started.elapsed();
                    forced_fallback.after_the_call(
                        &state.session,
                        intents_issued_before_the_call,
                        &call,
                    );
                    let empty_segments_after_the_call =
                        empty_segments_of_every_device(&state.session);
                    drop(guard);
                    // 放锁之后让一下（登记第十二节修订第 7 条）：标准库的锁不排队，刚放锁的这条线程马上再拿常常抢在等着的前台前面；
                    // 让一下不改登记的循环，只少记一截不是核心层造成的等待。产物头一行照同一个常量写 `gc_thread_yields_after_unlock=`。
                    if GC_THREAD_YIELDS_AFTER_UNLOCK {
                        std::thread::yield_now();
                    }
                    let mut record = record_of_the_call(
                        &call,
                        published_before_the_call,
                        None,
                        Duration::ZERO,
                        RejudgeTally::default(),
                    );
                    record.empty_segments_before_the_call = empty_segments_before_the_call;
                    record.empty_segments_after_the_call = empty_segments_after_the_call;
                    let published_nothing = the_call_published_nothing(&record);
                    calls.push(ConcurrentCallRecord {
                        call: record,
                        started_at,
                        returned_at,
                        fsync_queries: caller.queries_of_the_call.take(),
                    });
                    if published_nothing {
                        let published_count_guard = user_changes_published
                            .lock()
                            .expect("计数锁没在别处 panic 过");
                        let _still_the_same = user_change_published_condition
                            .wait_timeout_while(
                                published_count_guard,
                                CONCURRENT_GC_IDLE_WAIT,
                                |published| {
                                    *published == published_before_the_call
                                        && !foreground_is_done.load(Ordering::SeqCst)
                                },
                            )
                            .expect("计数锁没在别处 panic 过");
                    }
                }
                (
                    calls,
                    the_opening_thread_was_running_at_the_first_call,
                    forced_fallback.reading,
                )
            })
        });
        let gc_thread_records = gc
            .map(|handle| handle.join().expect("GC 线程没有 panic"))
            .unwrap_or_default();
        let (timings, write_generation_of_each_unit) =
            foreground.join().expect("前台线程没有 panic");
        (gc_thread_records, timings, write_generation_of_each_unit)
    });
    ConcurrentRunRecords {
        calls,
        forced_fallback,
        timings,
        the_opening_thread_was_running_at_the_first_call,
        state: session_and_devices
            .into_inner()
            .expect("会话锁没在别处 panic 过"),
        write_generation_of_each_unit,
    }
}

/// 第七节乙类最后一行（包含关系）：每次用户改动的延迟 ≥ 锁里发布用的时间。交回不成立（或读出负的）的改动数（作废第 1、2 条）。
fn user_changes_failing_the_inclusion_check(timings: &[UserChangeTimings]) -> u64 {
    let failing = timings
        .iter()
        .filter(|timings| {
            match (
                latency_of_the_user_change(**timings),
                publish_under_the_lock_of_the_user_change(**timings),
                wait_from_the_flag_to_the_lock(**timings),
            ) {
                (Some(latency), Some(publish_under_the_lock), Some(_wait)) => {
                    latency < publish_under_the_lock
                }
                (None, _, _) | (_, None, _) | (_, _, None) => true,
            }
        })
        .count();
    u64::try_from(failing).expect("改动数")
}

/// 并发跑一次的甲类锚点（第七节甲类第 1、2、5 行与第十条第 4 行）：每一项记账照设置走、条数不少于批数；GPU 臂分到活的卡与切片那几样；
/// 答了「在等」的调用都让了路；前台等锁没超过（最长一批 + 最长收尾）的两倍。
fn class_one_anchors_of_a_concurrent_run_hold(
    path_anchor: PathAnchor,
    compute: ComputeOfTheArm,
    gpu_work_anchors: GpuWorkAnchors,
    calls_not_yielding_after_answering_waiting: u64,
    the_foreground_waited_past_the_yield_bound: bool,
) -> bool {
    path_anchor.holds()
        && (!compute.uses_the_gpu() || gpu_work_anchors.hold())
        && calls_not_yielding_after_answering_waiting == 0
        && !the_foreground_waited_past_the_yield_bound
}

/// 一次并发跑的结果：`name=concurrent_run` 一行的数、锚点与作废原因。
struct ConcurrentRunSummary {
    arm: ConcurrentArm,
    repeat: u32,
    gc_calls: u64,
    gc_calls_started: u64,
    batches: u64,
    units_relocated: u64,
    latencies: Vec<f64>,
    publishes_under_the_lock: Vec<f64>,
    longest_wait_from_the_flag_to_the_lock: Duration,
    longest_batch: Duration,
    longest_tail: Duration,
    the_foreground_waited_past_the_yield_bound: bool,
    calls_not_yielding_after_answering_waiting: u64,
    waiting_answers: WaitingAnswersTrajectory,
    empty_segments_trajectory: Vec<EmptySegmentsTrajectoryOfADevice>,
    forced_fallback: ForcedFallbackReading,
    user_changes_failing_the_inclusion_check: u64,
    content_matches_the_expectation: bool,
    host_other_cores: f64,
    path_anchor: PathAnchor,
    gpu_work_anchors: GpuWorkAnchors,
    gpus_leaked: u64,
    stops: BTreeMap<String, u64>,
    void_reason: Option<String>,
}

impl ConcurrentRunSummary {
    fn class_one_anchors_hold(&self) -> bool {
        class_one_anchors_of_a_concurrent_run_hold(
            self.path_anchor,
            self.arm.compute_of_the_arm(),
            self.gpu_work_anchors,
            self.calls_not_yielding_after_answering_waiting,
            self.the_foreground_waited_past_the_yield_bound,
        )
    }

    fn fields(
        &self,
        tier: UnitAreaTier,
        session_number: u64,
        foreground_interval: Duration,
    ) -> String {
        let stops: Vec<String> = self
            .stops
            .iter()
            .map(|(stop, count)| format!("{stop}:{count}"))
            .collect();
        format!(
            "tier={} arm={} repeat={} session={session_number} foreground_interval_milliseconds={} user_changes={} \
             gc_calls={} gc_calls_started={} batches={} units_relocated={} {} {} latencies_nanoseconds={} \
             publishes_under_the_lock_nanoseconds={} longest_wait_from_the_flag_to_the_lock_nanoseconds={} \
             longest_batch_upper_bound_nanoseconds={} longest_tail_upper_bound_nanoseconds={} \
             yield_bound_reading=upper_bound yield_bound_failure_clause={} \
             calls_not_yielding_after_answering_waiting={} waiting_answers_peak_in_one_call={} calls_answered_waiting={} \
             waiting_answers_in_the_last_call={} {} user_changes_failing_the_inclusion_check={} \
             content_matches_the_expectation={} host_other_cores={:.2} compute_records={} \
             compute_records_on_the_path_of_the_settings={} path_anchor_holds={} gpu_records={} \
             segments_given_to_a_card_other_than_the_lowest={} recuts_after_running_out_of_memory={} \
             segments_with_pieces_other_than_submissions={} uncertain_entries={} entries_judged_by_the_cpu_reference={} \
             batches_abandoned_after_a_compute_error={} class_one_anchors_hold={} gpus_leaked={} {} stops={} void={}",
            tier.name(),
            self.arm.name(),
            self.repeat,
            foreground_interval.as_millis(),
            self.latencies.len(),
            self.gc_calls,
            self.gc_calls_started,
            self.batches,
            self.units_relocated,
            distribution_fields("latency_nanoseconds", latency_distribution_of(&self.latencies)),
            distribution_fields(
                "publish_under_the_lock_nanoseconds",
                latency_distribution_of(&self.publishes_under_the_lock)
            ),
            joined_values(&self.latencies),
            joined_values(&self.publishes_under_the_lock),
            self.longest_wait_from_the_flag_to_the_lock.as_nanos(),
            self.longest_batch.as_nanos(),
            self.longest_tail.as_nanos(),
            yield_bound_failure_clause_reading(self.the_foreground_waited_past_the_yield_bound),
            self.calls_not_yielding_after_answering_waiting,
            self.waiting_answers.peak_in_one_call,
            self.waiting_answers.calls_answered_waiting,
            self.waiting_answers.in_the_last_call,
            empty_segments_fields(&self.empty_segments_trajectory),
            self.user_changes_failing_the_inclusion_check,
            self.content_matches_the_expectation,
            self.host_other_cores,
            self.path_anchor.compute_records,
            self.path_anchor.compute_records_on_the_path_of_the_settings,
            self.path_anchor.holds(),
            self.gpu_work_anchors.gpu_records,
            self.gpu_work_anchors.segments_given_to_a_card_other_than_the_lowest,
            self.gpu_work_anchors.recuts_after_running_out_of_memory,
            self.gpu_work_anchors.segments_with_pieces_other_than_submissions,
            self.gpu_work_anchors.uncertain_entries,
            self.gpu_work_anchors.entries_judged_by_the_cpu_reference,
            self.gpu_work_anchors.batches_abandoned_after_a_compute_error,
            self.class_one_anchors_hold(),
            self.gpus_leaked,
            self.forced_fallback.fields(),
            field_value(&stops.join(",")),
            field_value(self.void_reason.as_deref().unwrap_or("no")),
        )
    }
}

/// 一次并发跑的设定。
struct ConcurrentRunSetting<'run> {
    image: &'run StartingImage,
    discrete_gpus: &'run [DiscreteGpuSeenByThisProcess],
    memory_leaked: &'run GpuMemoryLeakedInThisProcess,
    /// 开卡用的那一份进程级 Vulkan 实例；只跑 CPU 臂的没有。
    vulkan_instance: Option<&'run VulkanInstanceOfThisProcess>,
    user_changes: u64,
    foreground_interval: Duration,
}

/// 并发跑一次：拷池、开卡、挂载（批大小取主取样点）、两条线程并发、读内容、核锚点、记机器负载。开卡或挂载不成交回 `Err`（交主 agent）。
fn run_concurrent_once(
    setting: &ConcurrentRunSetting<'_>,
    arm: ConcurrentArm,
    repeat: u32,
) -> Result<ConcurrentRunSummary, String> {
    let host_at_the_start = host_load_reading()?;
    let settings =
        background_compute_settings_of_the_arm(arm.compute_of_the_arm(), setting.discrete_gpus);
    let gpu = open_the_gpu_of_the_arm(
        arm.as_a_lockstep_arm(),
        &settings,
        setting.memory_leaked,
        setting.vulkan_instance,
    )?;
    let (session, devices, _recording) = measuring_mount(
        setting.image,
        settings,
        gpu.gpu_compute,
        BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
    )?;
    let ConcurrentRunRecords {
        calls,
        forced_fallback,
        timings,
        the_opening_thread_was_running_at_the_first_call,
        state,
        write_generation_of_each_unit,
    } = concurrent_threads(
        SessionAndDevices { session, devices },
        setting.image.lowest_live_data_inode,
        setting.image.write_generation_of_each_unit.clone(),
        setting.image.next_write_time_seconds,
        ConcurrentRunPacing {
            user_changes: setting.user_changes,
            foreground_interval: setting.foreground_interval,
            runs_the_compaction: arm.runs_the_compaction(),
        },
        ForcedFallbackOfTheRun::of_the_image(setting.image),
    );
    let content_digest = read_content_digest(
        &state.devices,
        version_of_trees(&state.session),
        setting.image.lowest_live_data_inode,
        u64::try_from(write_generation_of_each_unit.len()).expect("P"),
    )?;
    let content_matches_the_expectation =
        content_digest == expected_content_digest(&write_generation_of_each_unit);
    drop(state);
    let gpus_leaked = match gpu
        .gpus_leaked_when_dropped
        .as_ref()
        .and_then(GpusLeakedWhenTheBackgroundGpuIsDropped::gpus_leaked)
    {
        Some(leaked) => {
            setting
                .memory_leaked
                .note_the_gpus_leaked_when_dropped(&leaked);
            u64::try_from(leaked.len()).expect("卡数")
        }
        None => 0,
    };
    let latencies: Vec<f64> = timings
        .iter()
        .filter_map(|timings| latency_of_the_user_change(*timings))
        .map(nanoseconds_as_float)
        .collect();
    let publishes_under_the_lock: Vec<f64> = timings
        .iter()
        .filter_map(|timings| publish_under_the_lock_of_the_user_change(*timings))
        .map(nanoseconds_as_float)
        .collect();
    let longest_wait_from_the_flag_to_the_lock = timings
        .iter()
        .filter_map(|timings| wait_from_the_flag_to_the_lock(*timings))
        .max()
        .unwrap_or(Duration::ZERO);
    let (longest_batch, longest_tail) = longest_batch_and_longest_tail_of_the_calls(&calls);
    let the_foreground_waited_past_the_yield_bound = arm.runs_the_compaction()
        && the_wait_exceeds_twice_the_longest_batch_and_tail(
            longest_wait_from_the_flag_to_the_lock,
            longest_batch,
            longest_tail,
        );
    let user_changes_failing_the_inclusion_check =
        user_changes_failing_the_inclusion_check(&timings);
    let calls_not_yielding_after_answering_waiting =
        calls_not_yielding_after_answering_waiting(&calls);
    let waiting_answers = waiting_answers_trajectory(&calls);
    let records: Vec<GcCallRecord> = calls.into_iter().map(|call| call.call).collect();
    let empty_segments_trajectory = empty_segments_trajectory_of_each_device(&records);
    let lowest_card = setting.discrete_gpus.first().map(|lowest_discrete_gpu| {
        gpu_device_identifier_of_the_pci_bus_address(&lowest_discrete_gpu.pci_bus_address)
    });
    let gpu_work_anchors = lowest_card
        .as_ref()
        .map_or_else(GpuWorkAnchors::default, |lowest_card| {
            gpu_work_anchors(&records, lowest_card)
        });
    let void_reason = if user_changes_failing_the_inclusion_check > 0 {
        Some(
            "a_latency_shorter_than_its_publish_under_the_lock_or_a_clock_went_backwards"
                .to_string(),
        )
    } else if the_opening_thread_was_running_at_the_first_call {
        Some("the_gpu_opening_thread_was_running_at_the_first_call".to_string())
    } else {
        None
    };
    let mut stops: BTreeMap<String, u64> = BTreeMap::new();
    for record in &records {
        *stops.entry(record.stop_name.clone()).or_insert(0) += 1;
    }
    let host_at_the_end = host_load_reading()?;
    Ok(ConcurrentRunSummary {
        arm,
        repeat,
        gc_calls: u64::try_from(records.len()).expect("调用数"),
        gc_calls_started: u64::try_from(records.iter().filter(|record| record.started).count())
            .expect("调用数"),
        batches: records.iter().map(|record| record.batches).sum(),
        units_relocated: records
            .iter()
            .flat_map(|record| record.units_relocated_of_each_batch.iter())
            .sum(),
        latencies,
        publishes_under_the_lock,
        longest_wait_from_the_flag_to_the_lock,
        longest_batch,
        longest_tail,
        the_foreground_waited_past_the_yield_bound,
        calls_not_yielding_after_answering_waiting,
        waiting_answers,
        empty_segments_trajectory,
        forced_fallback,
        user_changes_failing_the_inclusion_check,
        content_matches_the_expectation,
        host_other_cores: cores_used_outside_this_process(host_at_the_start, host_at_the_end),
        path_anchor: path_anchor(&records, arm.compute_of_the_arm()),
        gpu_work_anchors,
        gpus_leaked,
        stops,
        void_reason,
    })
}

/// 并发跑判定用的数：这一次跑刚量出的，或续跑时从产物的 `name=concurrent_run` 行读回的。
#[derive(Clone, Debug, PartialEq)]
struct ConcurrentRunNumbers {
    arm: ConcurrentArm,
    repeat: u32,
    latencies: Vec<f64>,
    publishes_under_the_lock: Vec<f64>,
}

/// `joined_values` 写出的一串纳秒数读回来；空串是一个都没有。
fn values_of_a_joined_field(joined: &str) -> Option<Vec<f64>> {
    if joined.is_empty() {
        return Some(Vec::new());
    }
    joined
        .split(',')
        .map(|value| value.parse::<f64>().ok())
        .collect()
}

impl ConcurrentRunNumbers {
    fn of_the_summary(summary: &ConcurrentRunSummary) -> ConcurrentRunNumbers {
        ConcurrentRunNumbers {
            arm: summary.arm,
            repeat: summary.repeat,
            latencies: summary.latencies.clone(),
            publishes_under_the_lock: summary.publishes_under_the_lock.clone(),
        }
    }

    fn of_the_fields(fields: &BTreeMap<String, String>) -> Option<ConcurrentRunNumbers> {
        Some(ConcurrentRunNumbers {
            arm: ConcurrentArm::of_the_name(fields.get("arm")?)?,
            repeat: fields.get("repeat")?.parse().ok()?,
            latencies: values_of_a_joined_field(fields.get("latencies_nanoseconds")?)?,
            publishes_under_the_lock: values_of_a_joined_field(
                fields.get("publishes_under_the_lock_nanoseconds")?,
            )?,
        })
    }
}

/// 一条臂按重复次序排好的那几次跑。
fn concurrent_runs_of_the_arm(
    numbers: &[ConcurrentRunNumbers],
    arm: ConcurrentArm,
) -> Vec<&ConcurrentRunNumbers> {
    let mut of_the_arm: Vec<&ConcurrentRunNumbers> =
        numbers.iter().filter(|run| run.arm == arm).collect();
    of_the_arm.sort_by_key(|run| run.repeat);
    of_the_arm
}

/// 一档并发跑的判定行（登记第六节量 F、量 F 底）：每条臂 5 次跑合起来的延迟分布与锁里发布用时的分布、每次跑各自的中位与第 99 百分位；
/// GPU 点名一张卡臂减 CPU 臂的第 99 百分位与最大值（同一次重复成对），5 个同号记方向，有正有负记「不稳定」。只报分布，不判谁好。
fn judgement_of_a_concurrent_tier(
    tier: UnitAreaTier,
    foreground_interval: Duration,
    numbers: &[ConcurrentRunNumbers],
    arms_run: &[ConcurrentArm],
) -> Vec<(String, String)> {
    let mut lines: Vec<(String, String)> = Vec::new();
    let per_run =
        |runs: &[&ConcurrentRunNumbers], value: fn(LatencyDistribution) -> f64| -> Vec<f64> {
            runs.iter()
                .filter_map(|run| latency_distribution_of(&run.latencies).map(value))
                .collect()
        };
    for arm in arms_run {
        let runs = concurrent_runs_of_the_arm(numbers, *arm);
        let pooled_latencies: Vec<f64> = runs
            .iter()
            .flat_map(|run| run.latencies.iter().copied())
            .collect();
        let pooled_publishes: Vec<f64> = runs
            .iter()
            .flat_map(|run| run.publishes_under_the_lock.iter().copied())
            .collect();
        lines.push((
            "latency_distribution".to_string(),
            format!(
                "tier={} arm={} foreground_interval_milliseconds={} runs={} {} {} median_of_each_run={} ninety_ninth_percentile_of_each_run={}",
                tier.name(),
                arm.name(),
                foreground_interval.as_millis(),
                runs.len(),
                distribution_fields("latency_nanoseconds", latency_distribution_of(&pooled_latencies)),
                distribution_fields(
                    "publish_under_the_lock_nanoseconds",
                    latency_distribution_of(&pooled_publishes)
                ),
                joined_values(&per_run(&runs, |distribution| distribution.median)),
                joined_values(&per_run(&runs, |distribution| distribution.high_percentile)),
            ),
        ));
    }
    let cpu_runs = concurrent_runs_of_the_arm(numbers, ConcurrentArm::Cpu);
    let gpu_runs =
        concurrent_runs_of_the_arm(numbers, ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress);
    if arms_run.contains(&ConcurrentArm::Cpu)
        && arms_run.contains(&ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress)
    {
        let statistics: [(&str, StatisticOfALatencyDistribution); 2] = [
            ("ninety_ninth_percentile", |distribution| {
                distribution.high_percentile
            }),
            ("maximum", |distribution| distribution.maximum),
        ];
        for (statistic, value) in statistics {
            let gpu_minus_cpu =
                paired_differences(&per_run(&gpu_runs, value), &per_run(&cpu_runs, value));
            lines.push((
                "latency_comparison".to_string(),
                format!(
                    "tier={} foreground_interval_milliseconds={} statistic={statistic} gpu_one_card_minus_cpu_nanoseconds={} sign={}",
                    tier.name(),
                    foreground_interval.as_millis(),
                    joined_values(&gpu_minus_cpu),
                    sign_of_the_paired_differences(&gpu_minus_cpu, 0.0).name()
                ),
            ));
        }
    }
    lines
}

/// 并发跑一次重复的结局。封闭集合。
enum ConcurrentRepeatOutcome {
    Done(Vec<ConcurrentRunSummary>),
    Voided { reason: String, gpu_error: bool },
    Stopped { reason: String },
}

/// 并发跑一次重复：三条臂按轮换次序串着跑，作废的那一臂重跑一次；跑完核机器负载、读到的内容、甲类（作废第 1–4、7 条，停机第 4、5 条）。
fn run_one_concurrent_repeat(
    setting: &ConcurrentRunSetting<'_>,
    arms_run: &[ConcurrentArm],
    repeat: u32,
    session_number: u64,
    product: &mut ProductWriter,
) -> ConcurrentRepeatOutcome {
    let tier = setting.image.tier;
    let start = usize::try_from(repeat).expect("重复次数") % arms_run.len().max(1);
    let rotation: Vec<ConcurrentArm> = arms_run[start..]
        .iter()
        .chain(&arms_run[..start])
        .copied()
        .collect();
    let mut runs: Vec<ConcurrentRunSummary> = Vec::new();
    for arm in rotation {
        let mut kept: Option<ConcurrentRunSummary> = None;
        for attempt in 0..2 {
            eprintln!(
                "E202 并发跑 {} 间隔 {} 毫秒 第 {repeat} 次重复 {} 第 {attempt} 次尝试",
                tier.name(),
                setting.foreground_interval.as_millis(),
                arm.name()
            );
            let summary = match run_concurrent_once(setting, arm, repeat) {
                Ok(summary) => summary,
                Err(reason) => return ConcurrentRepeatOutcome::Stopped { reason },
            };
            product.emit(
                "concurrent_run",
                &summary.fields(tier, session_number, setting.foreground_interval),
            );
            for line in summary.forced_fallback.choice_lines(&format!(
                "tier={} arm={} repeat={repeat} session={session_number} foreground_interval_milliseconds={}",
                tier.name(),
                arm.name(),
                setting.foreground_interval.as_millis()
            )) {
                product.emit("forced_choice", &line);
            }
            if summary
                .gpu_work_anchors
                .batches_abandoned_after_a_compute_error
                > 0
            {
                return ConcurrentRepeatOutcome::Voided {
                    reason: format!("{}_abandoned_a_batch_after_a_compute_error", arm.name()),
                    gpu_error: true,
                };
            }
            let is_void = summary.void_reason.is_some();
            kept = Some(summary);
            if !is_void {
                break;
            }
        }
        match kept {
            Some(summary) if summary.void_reason.is_none() => runs.push(summary),
            Some(summary) => {
                return ConcurrentRepeatOutcome::Voided {
                    reason: format!(
                        "{}_void_twice_{}",
                        arm.name(),
                        summary.void_reason.unwrap_or_default()
                    ),
                    gpu_error: false,
                }
            }
            None => {
                return ConcurrentRepeatOutcome::Stopped {
                    reason: "no_attempt_ran".to_string(),
                }
            }
        }
    }
    let busiest_host = runs
        .iter()
        .map(|run| run.host_other_cores)
        .fold(0.0, f64::max);
    let is_identical =
        !runs.is_empty() && runs.iter().all(|run| run.content_matches_the_expectation);
    let class_one_anchors_hold = runs
        .iter()
        .all(ConcurrentRunSummary::class_one_anchors_hold);
    product.emit(
        "concurrent_premise",
        &format!(
            "tier={} repeat={repeat} session={session_number} foreground_interval_milliseconds={} \
             content_matches_the_expectation={is_identical} class_one_anchors_hold={class_one_anchors_hold} \
             busiest_host_other_cores={busiest_host:.2}",
            tier.name(),
            setting.foreground_interval.as_millis()
        ),
    );
    match verdict_of_the_repeat_checks(ChecksOfARepeat {
        busiest_host_other_cores: busiest_host,
        is_identical,
        has_enough_batches: true,
        class_one_anchors_hold,
        class_two_anchors_hold: true,
    }) {
        VerdictOfTheRepeatChecks::Counted => {}
        VerdictOfTheRepeatChecks::Voided(reason) => {
            return ConcurrentRepeatOutcome::Voided {
                reason: reason.to_string(),
                gpu_error: false,
            }
        }
        VerdictOfTheRepeatChecks::Stopped(reason) => {
            return ConcurrentRepeatOutcome::Stopped {
                reason: reason.to_string(),
            }
        }
    }
    product.emit(
        "concurrent_repeat_done",
        &format!(
            "tier={} repeat={repeat} session={session_number} foreground_interval_milliseconds={}",
            tier.name(),
            setting.foreground_interval.as_millis()
        ),
    );
    ConcurrentRepeatOutcome::Done(runs)
}

/// 续跑：产物里这一档、这个间隔已做完的并发跑重复与它们 `void=no` 的 `name=concurrent_run` 行读回的数，与下一次重复的序号。
fn concurrent_repeats_already_done(
    lines: &[(String, BTreeMap<String, String>)],
    tier: UnitAreaTier,
    foreground_interval: Duration,
) -> (BTreeMap<u32, String>, Vec<ConcurrentRunNumbers>, u32) {
    let interval = foreground_interval.as_millis().to_string();
    let is_of_this_setting = |fields: &BTreeMap<String, String>| {
        fields.get("tier").map(String::as_str) == Some(tier.name())
            && fields.get("foreground_interval_milliseconds") == Some(&interval)
    };
    let done: BTreeMap<u32, String> = lines
        .iter()
        .filter(|(name, fields)| name == "concurrent_repeat_done" && is_of_this_setting(fields))
        .filter_map(|(_name, fields)| {
            Some((
                fields.get("repeat")?.parse::<u32>().ok()?,
                fields.get("session")?.clone(),
            ))
        })
        .collect();
    let numbers: Vec<ConcurrentRunNumbers> = lines
        .iter()
        .filter(|(name, fields)| {
            name == "concurrent_run"
                && is_of_this_setting(fields)
                && fields.get("void").map(String::as_str) == Some("no")
                && fields
                    .get("repeat")
                    .and_then(|repeat| repeat.parse::<u32>().ok())
                    .and_then(|repeat| done.get(&repeat))
                    == fields.get("session")
        })
        .filter_map(|(_name, fields)| ConcurrentRunNumbers::of_the_fields(fields))
        .collect();
    let next_repeat = lines
        .iter()
        .filter(|(name, fields)| {
            (name == "concurrent_run" || name == "concurrent_repeat_done")
                && is_of_this_setting(fields)
        })
        .filter_map(|(_name, fields)| fields.get("repeat")?.parse::<u32>().ok())
        .max()
        .map_or(0, |repeat| repeat + 1);
    (done, numbers, next_repeat)
}

/// 子命令 concurrent 的设定（几何敏感性那一点也走它）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ConcurrentCommand {
    tier: UnitAreaTier,
    foreground_interval: Duration,
    arms_requested: Vec<ConcurrentArm>,
    product: PathBuf,
}

/// 并发跑那条命令打在产物头一行（`name=session_start`）上的设定，连同 GC 线程放锁之后让不让（登记第十二节修订第 7 条）。
fn command_fields_of_a_concurrent_command(command: &ConcurrentCommand) -> String {
    format!(
        "command=concurrent reading=occupancy tier={} foreground_interval_milliseconds={} user_changes={CONCURRENT_USER_CHANGES} \
         batch_units={BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT} gc_thread_yields_after_unlock={GC_THREAD_YIELDS_AFTER_UNLOCK} arms={}",
        command.tier.name(),
        command.foreground_interval.as_millis(),
        command
            .arms_requested
            .iter()
            .map(|arm| arm.name())
            .collect::<Vec<&str>>()
            .join(",")
    )
}

fn concurrent_command(
    command: &ConcurrentCommand,
    vulkan_instance: &VulkanInstanceOfThisProcess,
) -> i32 {
    let mut product = match ProductWriter::appending_to(&command.product) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let command_fields = command_fields_of_a_concurrent_command(command);
    let opening = match opening_of_the_command(
        &mut product,
        &command.product,
        &command_fields,
        GpusOfTheCommand::EnumeratedOn(vulkan_instance),
    ) {
        Ok(opening) => opening,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    if !emit_whether_the_make_filesystem_accepts_the_tier(&mut product, command.tier) {
        product.finish();
        return EXIT_STOPPED;
    }
    let mut arms_run: Vec<ConcurrentArm> = Vec::new();
    for arm in &command.arms_requested {
        match reason_the_arm_is_not_run(arm.compute_of_the_arm(), opening.discrete_gpus.len()) {
            Some(reason) => product.emit(
                "not_run",
                &format!(
                    "tier={} arm={} reason={reason}",
                    command.tier.name(),
                    arm.name()
                ),
            ),
            None => arms_run.push(*arm),
        }
    }
    if opening.discrete_gpus.is_empty() {
        for arm in &arms_run {
            product.emit(
                "not_run",
                &format!(
                    "tier={} arm={} reason=本进程看得到的独立显卡一张都没有",
                    command.tier.name(),
                    arm.name()
                ),
            );
        }
        product.finish();
        return EXIT_NO_ARM_RAN;
    }
    let image = construct_the_starting_image(command.tier);
    if !emit_the_construction(&mut product, &image) {
        product.emit(
            "stopped",
            "clause=geometry_not_formed reason=几何没造成，改负载重造交主agent",
        );
        product.finish();
        return EXIT_STOPPED;
    }
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let setting = ConcurrentRunSetting {
        image: &image,
        discrete_gpus: &opening.discrete_gpus,
        memory_leaked: &memory_leaked,
        vulkan_instance: opening.vulkan_instance.as_ref(),
        user_changes: CONCURRENT_USER_CHANGES,
        foreground_interval: command.foreground_interval,
    };
    let (done, mut numbers, mut next_repeat) = concurrent_repeats_already_done(
        &opening.lines_already_in_the_product,
        command.tier,
        command.foreground_interval,
    );
    let mut repeats_done = u32::try_from(done.len()).expect("重复次数");
    product.emit(
        "resume",
        &format!(
            "command=concurrent tier={} foreground_interval_milliseconds={} repeats_already_done={repeats_done} next_repeat={next_repeat}",
            command.tier.name(),
            command.foreground_interval.as_millis()
        ),
    );
    let mut voided_repeats = 0_u32;
    let mut gpu_errors = 0_u32;
    while repeats_done < REPEATS_PER_ARM {
        let repeat = next_repeat;
        next_repeat += 1;
        match run_one_concurrent_repeat(
            &setting,
            &arms_run,
            repeat,
            opening.session_number,
            &mut product,
        ) {
            ConcurrentRepeatOutcome::Done(runs) => {
                numbers.extend(runs.iter().map(ConcurrentRunNumbers::of_the_summary));
                repeats_done += 1;
            }
            ConcurrentRepeatOutcome::Voided { reason, gpu_error } => {
                voided_repeats += 1;
                gpu_errors += u32::from(gpu_error);
                product.emit(
                    "repeat_voided",
                    &format!(
                        "command=concurrent tier={} repeat={repeat} reason={}",
                        command.tier.name(),
                        field_value(&reason)
                    ),
                );
                if the_tier_stops_after_the_voided_repeats(voided_repeats, gpu_errors) {
                    product.emit(
                        "stopped",
                        &format!("clause=too_many_voided_repeats voided={voided_repeats} gpu_errors={gpu_errors}"),
                    );
                    product.finish();
                    return EXIT_STOPPED;
                }
            }
            ConcurrentRepeatOutcome::Stopped { reason } => {
                product.emit(
                    "stopped",
                    &format!(
                        "command=concurrent tier={} repeat={repeat} clause={}",
                        command.tier.name(),
                        field_value(&reason)
                    ),
                );
                product.finish();
                return EXIT_STOPPED;
            }
        }
    }
    for (name, fields) in judgement_of_a_concurrent_tier(
        command.tier,
        command.foreground_interval,
        &numbers,
        &arms_run,
    ) {
        product.emit(&name, &fields);
    }
    product.emit(
        "verdict",
        &format!(
            "command=concurrent tier={} foreground_interval_milliseconds={} repeats={repeats_done} arms_run={} heavy_threads_registered={} \
             yield_bound_reading=upper_bound yield_bound_failure_clause={}",
            command.tier.name(),
            command.foreground_interval.as_millis(),
            arms_run.len(),
            opening.registered_heavy_threads,
            yield_bound_failure_clause_reading(false)
        ),
    );
    product.finish();
    EXIT_SUCCESS
}

// ---- 子命令 sensitivity（登记第八节「几何敏感性」两点：中档、批大小 16 的交替跑，前台间隔 500 毫秒的并发跑）----

/// 几何敏感性那两点取的档（中档）。
const SENSITIVITY_TIER: UnitAreaTier = UnitAreaTier::FourGibibytes;

/// 批大小 16 那一点：中档、CPU 臂与 GPU 点名一张卡臂各 5 次交替跑（量 S、量 W 的正负）。
fn sensitivity_lockstep_command(product: &Path, user_changes: u64) -> LockstepCommand {
    LockstepCommand {
        tier: SENSITIVITY_TIER,
        user_changes,
        batch_units: BATCH_UNITS_AT_THE_SENSITIVITY_POINT,
        arms_requested: vec![LOCKSTEP_ARMS[0], LOCKSTEP_ARMS[1]],
        product: product.to_path_buf(),
    }
}

/// 前台间隔 500 毫秒那一点：中档、三条臂各 5 次并发跑（量 F 的方向）。
fn sensitivity_concurrent_command(product: &Path) -> ConcurrentCommand {
    ConcurrentCommand {
        tier: SENSITIVITY_TIER,
        foreground_interval: Duration::from_millis(
            FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_SENSITIVITY_POINT,
        ),
        arms_requested: CONCURRENT_ARMS.to_vec(),
        product: product.to_path_buf(),
    }
}

/// 两点串着跑、写进同一份产物：交替跑那一点不是 0 就不跑并发那一点、交回它的退出码。
fn sensitivity_command(
    product: &Path,
    user_changes: u64,
    vulkan_instance: &VulkanInstanceOfThisProcess,
) -> i32 {
    let lockstep_exit = lockstep_command(
        &sensitivity_lockstep_command(product, user_changes),
        vulkan_instance,
    );
    if lockstep_exit != EXIT_SUCCESS {
        return lockstep_exit;
    }
    concurrent_command(&sensitivity_concurrent_command(product), vulkan_instance)
}

// ---- 子命令 geometry-check（登记第五节「分段」第一段：只造池、不计时、不要卡；第一节未定第 6 项：量每档造池耗时与每次跑的批数）----

/// 一档几何核的结局：起步镜像，与按阶梯在 CPU 臂上不计时地交替跑出的每一级的批数（够 64 批就不再往上）。
struct GeometryCheckOfATier {
    image: StartingImage,
    batches_by_user_changes: Vec<(u64, Result<u64, String>)>,
}

fn geometry_check_of_a_tier(tier: UnitAreaTier) -> GeometryCheckOfATier {
    let image = construct_the_starting_image(tier);
    let mut batches_by_user_changes: Vec<(u64, Result<u64, String>)> = Vec::new();
    if image.construction.geometry.is_formed() {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        for user_changes in LOCKSTEP_USER_CHANGES_LADDER {
            let setting = LockstepRunSetting {
                image: &image,
                discrete_gpus: &[],
                memory_leaked: &memory_leaked,
                vulkan_instance: None,
                user_changes,
                batch_units: BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
            };
            let batches =
                run_lockstep_once(&setting, LOCKSTEP_ARMS[0], 0).map(|summary| summary.batches);
            let is_enough = batches
                .as_ref()
                .is_ok_and(|batches| *batches >= MINIMUM_BATCHES_PER_RUN);
            let is_failed = batches.is_err();
            batches_by_user_changes.push((user_changes, batches));
            if is_enough || is_failed {
                break;
            }
        }
    }
    GeometryCheckOfATier {
        image,
        batches_by_user_changes,
    }
}

fn geometry_check_command(product_path: &Path) -> i32 {
    let mut product = match ProductWriter::appending_to(product_path) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let opening = match opening_of_the_command(
        &mut product,
        product_path,
        "command=geometry-check reading=occupancy",
        GpusOfTheCommand::NotNeeded,
    ) {
        Ok(opening) => opening,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    // mkfs 不认的那一档不造池（作废第 6 条，交主 agent），别的档照造。
    let mut is_every_tier_fine = true;
    let mut tiers_accepted: Vec<UnitAreaTier> = Vec::new();
    for tier in UnitAreaTier::EVERY_TIER {
        if emit_whether_the_make_filesystem_accepts_the_tier(&mut product, tier) {
            tiers_accepted.push(tier);
        } else {
            is_every_tier_fine = false;
        }
    }
    // 各档各一条线程、各建各的池（登记第十四节「独立单位」）；线程数取登记值，实际只用到档数那么多条。
    let threads = opening.registered_heavy_threads.min(tiers_accepted.len());
    let mut checks: Vec<GeometryCheckOfATier> = Vec::new();
    for tiers_of_a_wave in tiers_accepted.chunks(threads.max(1)) {
        let checks_of_the_wave: Vec<GeometryCheckOfATier> = std::thread::scope(|scope| {
            let handles: Vec<_> = tiers_of_a_wave
                .iter()
                .map(|tier| scope.spawn(move || geometry_check_of_a_tier(*tier)))
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("几何核那条线程没有 panic"))
                .collect()
        });
        checks.extend(checks_of_the_wave);
    }
    product.emit(
        "parallelism",
        &format!(
            "geometry_threads={threads} registered_heavy_threads={}",
            opening.registered_heavy_threads
        ),
    );
    for check in &checks {
        is_every_tier_fine &= emit_the_construction(&mut product, &check.image);
        let mut has_enough_batches = false;
        for (user_changes, batches) in &check.batches_by_user_changes {
            match batches {
                Ok(batches) => {
                    has_enough_batches = *batches >= MINIMUM_BATCHES_PER_RUN;
                    product.emit(
                        "geometry_batches",
                        &format!(
                            "tier={} user_changes={user_changes} batches={batches} enough={has_enough_batches}",
                            check.image.tier.name()
                        ),
                    );
                }
                Err(reason) => product.emit(
                    "geometry_batches",
                    &format!(
                        "tier={} user_changes={user_changes} failed={}",
                        check.image.tier.name(),
                        field_value(reason)
                    ),
                ),
            }
        }
        is_every_tier_fine &= has_enough_batches;
    }
    product.emit(
        "verdict",
        &format!(
            "command=geometry-check every_tier_formed_with_enough_batches={is_every_tier_fine}"
        ),
    );
    product.finish();
    if is_every_tier_fine {
        EXIT_SUCCESS
    } else {
        EXIT_STOPPED
    }
}

// ---- 子命令 construction-timing（造池分阶段计时：在给定的几个小池上照同一套造法造起步镜像，按阶段打用时与内存；不跑 GC、不要卡、不进判据）----

impl ChunkStorage {
    const fn name(self) -> &'static str {
        match self {
            ChunkStorage::EachDeviceCopiesItsChunks => "each-device-copies",
            ChunkStorage::SharedWhenTheBytesAreEqual => "shared",
        }
    }

    fn of_the_name(name: &str) -> Option<ChunkStorage> {
        [
            ChunkStorage::EachDeviceCopiesItsChunks,
            ChunkStorage::SharedWhenTheBytesAreEqual,
        ]
        .into_iter()
        .find(|storage| storage.name() == name)
    }
}

/// `construction-timing` 的设定：盘取哪一档的大小、造哪几个 P（各造一遍、按给的次序串着造，常驻内存读的是整个进程）、块怎么存。
struct ConstructionTimingCommand {
    tier: UnitAreaTier,
    live_data_units: Vec<u64>,
    storage: ChunkStorage,
    product: PathBuf,
}

/// 造池认的 P：大于 0、是 32 的倍数（交错那一段写满整段，P ÷ 2 也就是 K = 16 的倍数），不超过这一档登记的 P。
fn the_construction_accepts_the_live_data_units(tier: UnitAreaTier, live_data_units: u64) -> bool {
    live_data_units > 0
        && live_data_units.is_multiple_of(data_units_of_a_cluster_segment())
        && live_data_units <= tier.live_data_units()
}

fn construction_timing_command(command: &ConstructionTimingCommand) -> i32 {
    let mut product = match ProductWriter::appending_to(&command.product) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let joined_live_data_units: Vec<String> =
        command.live_data_units.iter().map(u64::to_string).collect();
    if let Err(exit_code) = opening_of_the_command(
        &mut product,
        &command.product,
        &format!(
            "command=construction-timing tier={} chunks={} live_data_units={}",
            command.tier.name(),
            command.storage.name(),
            joined_live_data_units.join(",")
        ),
        GpusOfTheCommand::NotNeeded,
    ) {
        product.finish();
        return exit_code;
    }
    for live_data_units in &command.live_data_units {
        let mut progress_to_the_standard_error = |line: &str| eprintln!("{line}");
        let mut clock = ConstructionClock::new(
            format!("{}-P{live_data_units}", command.tier.name()),
            CONSTRUCTION_PROGRESS_INTERVAL,
            &mut progress_to_the_standard_error,
        );
        let image = construct_a_starting_image_on_the_clock(
            command.tier,
            command.tier.device_bytes(),
            *live_data_units,
            command.storage,
            &mut clock,
        );
        let construction = &image.construction;
        let histogram: Vec<String> = construction
            .geometry
            .live_slot_histogram
            .iter()
            .map(|(live, segments)| format!("{live}:{segments}"))
            .collect();
        let chunk_bytes = chunk_bytes_of_the_pool(&image.devices);
        product.emit(
            "construction_timing",
            &format!(
                "tier={} device_bytes={} live_data_units={live_data_units} chunks={} construction_seconds={:.3} publishes={} \
                 floor_raises={} release_rounds={} live_data_units_found={} empty_segments={} mapped_units={} written_segments={} \
                 written_segments_with_half_live={} forced_segment_start_slot={} live_slot_histogram={} \
                 chunk_bytes_held_by_the_devices={} chunk_bytes_stored={} resident_bytes={}",
                command.tier.name(),
                command.tier.device_bytes(),
                command.storage.name(),
                construction.construction_seconds,
                construction.publishes,
                construction.floor_raises,
                construction.release_rounds,
                construction.live_data_units_found,
                construction.empty_segments,
                construction.mapped_units,
                construction.geometry.written_segments,
                construction.geometry.written_segments_with_half_live,
                construction
                    .geometry
                    .forced
                    .start_slot
                    .map_or_else(|| "none".to_string(), |slot| slot.to_string()),
                histogram.join(","),
                chunk_bytes.held_by_the_devices,
                chunk_bytes.stored,
                resident_bytes_of_this_process()
                    .map_or_else(|| "unread".to_string(), |bytes| bytes.to_string()),
            ),
        );
        for timing in &construction.phases {
            product.emit(
                "construction_phase",
                &format!(
                    "tier={} live_data_units={live_data_units} chunks={} {}",
                    command.tier.name(),
                    command.storage.name(),
                    timing.fields()
                ),
            );
        }
    }
    product.finish();
    EXIT_SUCCESS
}

// ---- 子命令 stage-three-cost（登记第十二节修订第 9 条：丙段代价另量一遍，只交用户定「强制选段之后对照缺不缺丙段」那一条岔路，不进判据）----

/// 每条臂数几遍、取中位（修订第 9 条「各量 5 次……取中位」）。
const STAGE_THREE_COST_REPEATS: usize = 5;

/// 丙段代价量的两条臂（修订第 9 条：CPU 臂与 GPU 点名一张卡臂；挂载参数同交替跑那两条臂，每张卡上限 1 GiB）。
const STAGE_THREE_COST_ARMS: [ComputeOfTheArm; 2] = [
    ComputeOfTheArm::Cpu,
    ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
];

/// CPU 臂一次 GC 调用里丙段那一截的读数：核心层在 `choose_region` 里自己数、不经任何从包外递得进去的实现，外面量不到。
const STAGE_THREE_OF_THE_CPU_ARM_NOT_REACHABLE: &str = "not_reachable_from_outside_the_core";

/// 装置自己从分配记录数 `unit_area` 那块盘上每个整段的活槽数，按段起点从小到大一段一个。不调核心层与产品 GPU 包：
/// 按每条记录的槽区间与段区间求交，口径同核心层的 CPU 写法（只数这块盘上未释放的记录、只数落在整段里的槽、末尾凑不满一段的那一截不算）。
fn live_slots_of_each_segment_counted_by_the_apparatus(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Vec<u64> {
    let unit_area_start_slot = unit_area.start.slot().0;
    let full_segments = unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS;
    let full_segments_end_slot = unit_area_start_slot + full_segments * LOCAL_CLUSTER_SEGMENT_SLOTS;
    let mut live_slots =
        vec![0_u64; usize::try_from(full_segments).expect("段数装得进 usize")];
    for record in allocation_records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        let first_slot = record.slot.0.max(unit_area_start_slot);
        let end_slot = (record.slot.0 + u64::from(record.span_slots)).min(full_segments_end_slot);
        // 迭代上界：这条记录跨过的段数（跨度至多 u16 能装的槽数）；每一轮至少前进一槽。
        let mut slot = first_slot;
        while slot < end_slot {
            let segment = (slot - unit_area_start_slot) / LOCAL_CLUSTER_SEGMENT_SLOTS;
            let segment_end_slot =
                unit_area_start_slot + (segment + 1) * LOCAL_CLUSTER_SEGMENT_SLOTS;
            let slots_in_the_segment = segment_end_slot.min(end_slot) - slot;
            live_slots[usize::try_from(segment).expect("段下标")] += slots_in_the_segment;
            slot += slots_in_the_segment;
        }
    }
    live_slots
}

/// 一条臂数一遍每段活槽数用的那一份。封闭集合。
enum StageThreeCounter<Gpu> {
    /// 核心层的 CPU 写法（`live_slots_per_cluster_segment_by_the_records`，`choose_region` 打分前数的就是它）。
    OnTheCpu,
    /// 产品 GPU 包公开的丙段（`BackgroundGpuCompute::count_live_slots_per_cluster_segment`：两份核各数一次、比对，连续不确定的交 CPU 判）。
    OnTheGpu(Gpu),
}

impl<Gpu: BackgroundGpuCompute> StageThreeCounter<Gpu> {
    /// 数一遍：交回挂钟与每段的活槽数（GPU 那一份这一轮不确定的段是 `None`）。挂钟只罩数的那一次调用（GPU 那一份交卡、等卡、两份核比对都在里面），
    /// 不罩换成 `Option` 那一步与读这一遍的交卡记录。GPU 那一份出错交回 `Err`。
    fn count_once(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<(Duration, Vec<Option<u64>>), String> {
        match self {
            StageThreeCounter::OnTheCpu => {
                let started = Instant::now();
                let live_slots =
                    live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area);
                let wall = started.elapsed();
                Ok((wall, live_slots.into_iter().map(Some).collect()))
            }
            StageThreeCounter::OnTheGpu(gpu) => {
                let started = Instant::now();
                let counted = gpu.count_live_slots_per_cluster_segment(
                    allocation_records,
                    unit_area,
                    segments,
                );
                let wall = started.elapsed();
                // 核心层每一批三段算完之后读一次交卡记录（读了才清零）；这里每一遍之后读一次，不让它攒着。
                let _submissions_of_the_pass = gpu.take_the_submissions_of_the_batch();
                let decisions = counted.map_err(|failed| format!("GPU 丙段出错：{failed:?}"))?;
                Ok((
                    wall,
                    decisions
                        .into_iter()
                        .map(|decision| match decision {
                            GpuEntryDecision::ConfirmedByBothKernels(live_slots)
                            | GpuEntryDecision::JudgedByTheCpuReference(live_slots) => {
                                Some(live_slots)
                            }
                            GpuEntryDecision::UncertainThisRound => None,
                        })
                        .collect(),
                ))
            }
        }
    }
}

/// 一条臂数 [`STAGE_THREE_COST_REPEATS`] 遍的读数：每一遍的挂钟与数出的每段活槽数，与头一遍之前挂载开卡另起的那条线程还在不在跑。
#[derive(Clone, Debug, PartialEq, Eq)]
struct StageThreeCountsOfTheArm {
    walls: Vec<Duration>,
    live_slots_of_each_pass: Vec<Vec<Option<u64>>>,
    the_opening_thread_was_running_at_the_first_pass: bool,
}

/// 同一份记录、同一块单元区上数 [`STAGE_THREE_COST_REPEATS`] 遍（修订第 9 条「数一遍全部分配记录的每段活槽数」）。
fn stage_three_counts_of_the_counter<Gpu: BackgroundGpuCompute>(
    counter: &mut StageThreeCounter<Gpu>,
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Result<StageThreeCountsOfTheArm, String> {
    let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
    let the_opening_thread_was_running_at_the_first_pass =
        the_gpu_opening_thread_is_still_running().unwrap_or(true);
    let mut walls: Vec<Duration> = Vec::new();
    let mut live_slots_of_each_pass: Vec<Vec<Option<u64>>> = Vec::new();
    for _pass in 0..STAGE_THREE_COST_REPEATS {
        let (wall, live_slots) = counter.count_once(allocation_records, unit_area, &segments)?;
        walls.push(wall);
        live_slots_of_each_pass.push(live_slots);
    }
    Ok(StageThreeCountsOfTheArm {
        walls,
        live_slots_of_each_pass,
        the_opening_thread_was_running_at_the_first_pass,
    })
}

/// 几遍挂钟的中位：奇数遍取正中那一遍，偶数遍取正中两遍的平均；一遍都没有是零。
fn median_wall_of(walls: &[Duration]) -> Duration {
    let mut sorted = walls.to_vec();
    sorted.sort();
    match sorted.len() {
        0 => Duration::ZERO,
        length if length % 2 == 1 => sorted[length / 2],
        length => (sorted[length / 2 - 1] + sorted[length / 2]) / 2,
    }
}

/// 每一遍数出的每段活槽数都与装置自己数的逐段相同（这一轮不确定的段、段数对不上都算不同）。
fn every_pass_agrees_with_the_apparatus_count(
    counts: &StageThreeCountsOfTheArm,
    counted_by_the_apparatus: &[u64],
) -> bool {
    counts.live_slots_of_each_pass.iter().all(|live_slots| {
        live_slots.len() == counted_by_the_apparatus.len()
            && live_slots
                .iter()
                .zip(counted_by_the_apparatus)
                .all(|(counted, by_the_apparatus)| *counted == Some(*by_the_apparatus))
    })
}

/// `name=stage_three_cost` 一行（修订第 9 条写的四个键在前、次序照写，其后是装置另报的几样）。
fn stage_three_cost_fields(
    tier: UnitAreaTier,
    arm: ComputeOfTheArm,
    counts: &StageThreeCountsOfTheArm,
    counted_by_the_apparatus: &[u64],
) -> String {
    format!(
        "tier={} arm={} median_nanoseconds={} repeats={} nanoseconds_of_each_repeat={} segments={} live_slots={} \
         every_repeat_agrees_with_the_apparatus_count={} gpu_opening_thread_was_running_at_the_first_repeat={}",
        tier.name(),
        arm.name(),
        median_wall_of(&counts.walls).as_nanos(),
        counts.walls.len(),
        counts
            .walls
            .iter()
            .map(|wall| wall.as_nanos().to_string())
            .collect::<Vec<String>>()
            .join(","),
        counted_by_the_apparatus.len(),
        counted_by_the_apparatus.iter().sum::<u64>(),
        every_pass_agrees_with_the_apparatus_count(counts, counted_by_the_apparatus),
        counts.the_opening_thread_was_running_at_the_first_pass,
    )
}

/// 丙段在一次 GC 调用里被调了几次、合起来用了多少挂钟（GPU 臂外面套的计时记下的）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StageThreeTiming {
    calls: u64,
    wall: Duration,
}

type SharedStageThreeTiming = Arc<Mutex<StageThreeTiming>>;

/// 产品 GPU 包那一份外面套的计时：每次丙段（`count_live_slots_per_cluster_segment`）的挂钟与次数记进共享的那一份，别的方法原样转。
struct GpuComputeTimingTheStageThree<Inner> {
    inner: Inner,
    timing: SharedStageThreeTiming,
}

impl<Inner: GpuInventory> GpuInventory for GpuComputeTimingTheStageThree<Inner> {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.inner.devices_with_total_memory()
    }

    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
        self.inner.devices_not_opened_at_the_mount()
    }
}

impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeTimingTheStageThree<Inner> {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        self.inner.verify_unit_checksums(units)
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        self.inner
            .classify_region_records(region, records, mapping_entries)
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        let started = Instant::now();
        let counted = self.inner.count_live_slots_per_cluster_segment(
            allocation_records,
            unit_area,
            segments,
        );
        let wall = started.elapsed();
        let mut timing = self
            .timing
            .lock()
            .expect("丙段计时只在调后台入口的那条线程上写，没在别处 panic 过");
        timing.calls += 1;
        timing.wall += wall;
        counted
    }

    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.inner.use_the_settings_from_the_next_batch(settings);
    }

    fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
        self.inner.failure_under_the_settings_handed_in()
    }

    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        self.inner.take_the_submissions_of_the_batch()
    }
}

/// 同一镜像上不强制选段、照产品 greedy 跑的那一次 GC 调用：挂钟、丙段那一截（CPU 臂量不到，`None`）、这次调用搬了几批，
/// 与调用之前挂载开卡另起的那条线程还在不在跑。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StageThreeShareOfOneGreedyCall {
    gc_call_wall: Duration,
    stage_three: Option<StageThreeTiming>,
    batches: u64,
    the_opening_thread_was_running_at_the_call: bool,
}

/// `name=stage_three_share` 一行（修订第 9 条写的四个键在前、次序照写；CPU 臂丙段那一截写 [`STAGE_THREE_OF_THE_CPU_ARM_NOT_REACHABLE`]）。
fn stage_three_share_fields(
    tier: UnitAreaTier,
    arm: ComputeOfTheArm,
    share: &StageThreeShareOfOneGreedyCall,
) -> String {
    let (stage_three_nanoseconds, stage_three_calls) = share.stage_three.map_or_else(
        || {
            (
                STAGE_THREE_OF_THE_CPU_ARM_NOT_REACHABLE.to_string(),
                STAGE_THREE_OF_THE_CPU_ARM_NOT_REACHABLE.to_string(),
            )
        },
        |timing| (timing.wall.as_nanos().to_string(), timing.calls.to_string()),
    );
    format!(
        "tier={} arm={} gc_call_nanoseconds={} stage_three_nanoseconds={stage_three_nanoseconds} \
         stage_three_calls={stage_three_calls} batches={} gpu_opening_thread_was_running_at_the_call={}",
        tier.name(),
        arm.name(),
        share.gc_call_wall.as_nanos(),
        share.batches,
        share.the_opening_thread_was_running_at_the_call,
    )
}

/// 修订第 9 条后一半：从起步镜像量挂载（照这条臂的设置，GPU 臂外面套丙段计时；不装强制选段，核心层照产品 greedy 选段），
/// 前台先发一次用户改动（同交替跑头一次调用之前那一次），再调一次后台入口、量这一次的挂钟。开卡或挂载不成交回 `Err`。
fn stage_three_share_of_one_greedy_call(
    image: &StartingImage,
    arm: ComputeOfTheArm,
    discrete_gpus: &[DiscreteGpuSeenByThisProcess],
    memory_leaked: &GpuMemoryLeakedInThisProcess,
    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
) -> Result<StageThreeShareOfOneGreedyCall, String> {
    let settings = background_compute_settings_of_the_arm(arm, discrete_gpus);
    let (gpu_compute, timing, gpus_leaked_when_dropped) = if arm.uses_the_gpu() {
        let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
            &BackgroundGpuSelection::of_the_settings(&settings),
            memory_leaked,
            vulkan_instance_for_the_gpu_arm(vulkan_instance)?,
        )
        .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
        let gpus_leaked_when_the_gpu_is_dropped = background_gpu.gpus_leaked_when_dropped();
        let shared_timing: SharedStageThreeTiming =
            Arc::new(Mutex::new(StageThreeTiming::default()));
        let timed = GpuComputeTimingTheStageThree {
            inner: BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(
                background_gpu,
            ),
            timing: Arc::clone(&shared_timing),
        };
        (
            Some(BackgroundGpuComputeHandle::of_an_implementation(timed)),
            Some(shared_timing),
            Some(gpus_leaked_when_the_gpu_is_dropped),
        )
    } else {
        (None, None, None)
    };
    let (mut session, mut devices, _recording) = measuring_mount(
        image,
        settings,
        gpu_compute,
        BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
    )?;
    let mut write_generation_of_each_unit = image.write_generation_of_each_unit.clone();
    publish_the_user_change(
        &mut session,
        &mut devices,
        image.lowest_live_data_inode,
        0,
        &mut write_generation_of_each_unit,
        image.next_write_time_seconds,
    );
    let the_opening_thread_was_running_at_the_call =
        the_gpu_opening_thread_is_still_running().unwrap_or(true);
    let timing_before_the_call = timing.as_ref().map(|shared_timing| {
        *shared_timing
            .lock()
            .expect("丙段计时只在这条线程上写，没在别处 panic 过")
    });
    let caller = CallerOfTheLockstepRun::starting_at_zero();
    caller.advance_before_the_next_call();
    let started = Instant::now();
    let call = session
        .run_compaction_in_the_background(&mut devices, pacing_of_the_measuring_mount(), &caller)
        .expect("盘表是挂载核过的那一份、现行那一版有树：后台入口不拒");
    let gc_call_wall = started.elapsed();
    let stage_three =
        timing
            .as_ref()
            .zip(timing_before_the_call)
            .map(|(shared_timing, before_the_call)| {
                let after_the_call = *shared_timing
                    .lock()
                    .expect("丙段计时只在这条线程上写，没在别处 panic 过");
                StageThreeTiming {
                    calls: after_the_call.calls - before_the_call.calls,
                    wall: after_the_call.wall - before_the_call.wall,
                }
            });
    let batches =
        record_of_the_call(&call, 0, None, Duration::ZERO, RejudgeTally::default()).batches;
    drop(session);
    if let Some(leaked) = gpus_leaked_when_dropped
        .as_ref()
        .and_then(GpusLeakedWhenTheBackgroundGpuIsDropped::gpus_leaked)
    {
        memory_leaked.note_the_gpus_leaked_when_dropped(&leaked);
    }
    Ok(StageThreeShareOfOneGreedyCall {
        gc_call_wall,
        stage_three,
        batches,
        the_opening_thread_was_running_at_the_call,
    })
}

/// 修订第 9 条前一半：照这条臂数 [`STAGE_THREE_COST_REPEATS`] 遍。GPU 臂照交替跑那条臂的设置开产品那一份（不挂载：照核心层挂载时的做法
/// 校验设置、交给实现），数完关掉、记下卸载泄漏的卡。开卡、校验不成或这组设置下一张卡都用不上交回 `Err`。
fn stage_three_counts_on_the_arm(
    arm: ComputeOfTheArm,
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    discrete_gpus: &[DiscreteGpuSeenByThisProcess],
    memory_leaked: &GpuMemoryLeakedInThisProcess,
    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
) -> Result<StageThreeCountsOfTheArm, String> {
    match arm {
        ComputeOfTheArm::Cpu => stage_three_counts_of_the_counter(
            &mut StageThreeCounter::<BackgroundGpuComputeOnTheParticipatingGpus>::OnTheCpu,
            allocation_records,
            unit_area,
        ),
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress | ComputeOfTheArm::GpuOnEveryCard => {
            let settings = background_compute_settings_of_the_arm(arm, discrete_gpus);
            let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
                &BackgroundGpuSelection::of_the_settings(&settings),
                memory_leaked,
                vulkan_instance_for_the_gpu_arm(vulkan_instance)?,
            )
            .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
            let gpus_leaked_when_the_gpu_is_dropped = background_gpu.gpus_leaked_when_dropped();
            let mut gpu =
                BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(background_gpu);
            let admitted = admit_gpu_settings_against_the_inventory(&settings, &gpu)
                .map_err(|refused| format!("校验 GPU 设置：{refused:?}"))?;
            gpu.use_the_settings_from_the_next_batch(&admitted);
            if let Some(failure) = gpu.failure_under_the_settings_handed_in() {
                return Err(format!("这组设置下一张卡都用不上：{failure:?}"));
            }
            let mut counter = StageThreeCounter::OnTheGpu(gpu);
            let counts =
                stage_three_counts_of_the_counter(&mut counter, allocation_records, unit_area);
            drop(counter);
            if let Some(leaked) = gpus_leaked_when_the_gpu_is_dropped.gpus_leaked() {
                memory_leaked.note_the_gpus_leaked_when_dropped(&leaked);
            }
            counts
        }
    }
}

/// 起步镜像在量挂载那一刻那一版的分配记录与盘 0 的单元区（核心层选段数活槽的就是这一块；挂载照产品设置，不开卡）。挂载不成交回 `Err`。
fn allocation_records_and_unit_area_of_the_image(
    image: &StartingImage,
) -> Result<(Vec<AllocationRecord>, UnitAreaOfOneDevice), String> {
    let (session, _devices, _recording) = measuring_mount(
        image,
        BackgroundComputeSettings::PRODUCT,
        None,
        BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
    )?;
    Ok((
        version_of_trees(&session).allocation_records.to_vec(),
        unit_area_of_device_zero(&session),
    ))
}

fn stage_three_cost_command(
    tier: UnitAreaTier,
    product_path: &Path,
    vulkan_instance: &VulkanInstanceOfThisProcess,
) -> i32 {
    let mut product = match ProductWriter::appending_to(product_path) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let opening = match opening_of_the_command(
        &mut product,
        product_path,
        &format!(
            "command=stage-three-cost tier={} repeats={STAGE_THREE_COST_REPEATS}",
            tier.name()
        ),
        GpusOfTheCommand::EnumeratedOn(vulkan_instance),
    ) {
        Ok(opening) => opening,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    if !emit_whether_the_make_filesystem_accepts_the_tier(&mut product, tier) {
        product.finish();
        return EXIT_STOPPED;
    }
    // 几何核同一份起步镜像（修订第 9 条）；几何成没成照打、不判：丙段数的是全部记录，不看碎片段。
    let image = construct_the_starting_image(tier);
    let _is_the_geometry_formed = emit_the_construction(&mut product, &image);
    let (allocation_records, unit_area) =
        match allocation_records_and_unit_area_of_the_image(&image) {
            Ok(records_and_unit_area) => records_and_unit_area,
            Err(reason) => {
                product.emit(
                    "stopped",
                    &format!(
                        "clause=measuring_mount_refused reason={}",
                        field_value(&reason)
                    ),
                );
                product.finish();
                return EXIT_RUN_CONDITION_NOT_MET;
            }
        };
    let counted_by_the_apparatus =
        live_slots_of_each_segment_counted_by_the_apparatus(&allocation_records, unit_area);
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let mut arms_run: Vec<&str> = Vec::new();
    let mut does_every_arm_agree = true;
    for arm in STAGE_THREE_COST_ARMS {
        if let Some(reason) = reason_the_arm_is_not_run(arm, opening.discrete_gpus.len()) {
            product.emit(
                "not_run",
                &format!("tier={} arm={} reason={reason}", tier.name(), arm.name()),
            );
            continue;
        }
        eprintln!("E202 丙段代价 {} {}", tier.name(), arm.name());
        let measured = stage_three_counts_on_the_arm(
            arm,
            &allocation_records,
            unit_area,
            &opening.discrete_gpus,
            &memory_leaked,
            opening.vulkan_instance.as_ref(),
        )
        .and_then(|counts| {
            stage_three_share_of_one_greedy_call(
                &image,
                arm,
                &opening.discrete_gpus,
                &memory_leaked,
                opening.vulkan_instance.as_ref(),
            )
            .map(|share| (counts, share))
        });
        let (counts, share) = match measured {
            Ok(counts_and_share) => counts_and_share,
            Err(reason) => {
                product.emit(
                    "stopped",
                    &format!(
                        "clause=arm_failed tier={} arm={} reason={}",
                        tier.name(),
                        arm.name(),
                        field_value(&reason)
                    ),
                );
                product.finish();
                return EXIT_STOPPED;
            }
        };
        does_every_arm_agree &=
            every_pass_agrees_with_the_apparatus_count(&counts, &counted_by_the_apparatus);
        product.emit(
            "stage_three_cost",
            &stage_three_cost_fields(tier, arm, &counts, &counted_by_the_apparatus),
        );
        product.emit(
            "stage_three_share",
            &stage_three_share_fields(tier, arm, &share),
        );
        arms_run.push(arm.name());
    }
    product.emit(
        "verdict",
        &format!(
            "command=stage-three-cost tier={} arms_run={} every_arm_agrees_with_the_apparatus_count={does_every_arm_agree}",
            tier.name(),
            arms_run.join(",")
        ),
    );
    product.finish();
    if does_every_arm_agree {
        EXIT_SUCCESS
    } else {
        EXIT_STOPPED
    }
}

// ---- 子命令 judge-trend（登记第六节量 T；停机条款第 2 条：三档产物的 crates 版本相同）----

fn judge_trend_command(product_path: &Path, lockstep_products: &[PathBuf]) -> i32 {
    let mut product = match ProductWriter::appending_to(product_path) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let mut crates_versions: BTreeSet<String> = BTreeSet::new();
    let mut savings_per_batch: BTreeMap<(String, UnitAreaTier), Vec<f64>> = BTreeMap::new();
    for lockstep_product in lockstep_products {
        let lines = match result_lines_already_in_the_product(lockstep_product) {
            Ok(lines) => lines,
            Err(reason) => {
                product.emit(
                    "input_unreadable",
                    &format!("reason={}", field_value(&reason)),
                );
                product.finish();
                return EXIT_RUN_CONDITION_NOT_MET;
            }
        };
        for (name, fields) in &lines {
            match name.as_str() {
                "crates_version" => crates_versions.extend(fields.get("combined_sha256").cloned()),
                "saving_per_batch" => {
                    let tier = fields
                        .get("tier")
                        .and_then(|tier| UnitAreaTier::of_the_name(tier));
                    let values: Option<Vec<f64>> = fields.get("values").map(|values| {
                        values
                            .split(',')
                            .filter_map(|value| value.parse::<f64>().ok())
                            .collect()
                    });
                    if let (Some(tier), Some(against), Some(values)) =
                        (tier, fields.get("against"), values)
                    {
                        savings_per_batch.insert((against.clone(), tier), values);
                    }
                }
                _other_result_lines => {}
            }
        }
    }
    product.emit(
        "crates_versions_of_the_inputs",
        &format!("distinct={}", crates_versions.len()),
    );
    if crates_versions.len() != 1 {
        product.emit(
            "stopped",
            "clause=crates_changed_between_the_commands_of_the_segment",
        );
        product.finish();
        return EXIT_STOPPED;
    }
    let arms_compared: BTreeSet<String> = savings_per_batch
        .keys()
        .map(|(against, _tier)| against.clone())
        .collect();
    for against in arms_compared {
        let by_tier: Option<Vec<Vec<f64>>> = UnitAreaTier::EVERY_TIER
            .iter()
            .map(|tier| savings_per_batch.get(&(against.clone(), *tier)).cloned())
            .collect();
        match by_tier.and_then(|by_tier| <[Vec<f64>; 3]>::try_from(by_tier).ok()) {
            Some(by_tier) => product.emit(
                "trend",
                &format!(
                    "against={against} tiers_with_a_saving=3 trend={}",
                    trend_over_the_tiers(&by_tier).name()
                ),
            ),
            None => product.emit(
                "trend",
                &format!(
                    "against={against} tiers_with_a_saving_fewer_than_three=true trend=not_judged"
                ),
            ),
        }
    }
    product.finish();
    EXIT_SUCCESS
}

// ---- 命令行 ----

const USAGE: &str = "用法：
  e202_gpu_cpu_gc_comparison geometry-check --product <文件>
  e202_gpu_cpu_gc_comparison lockstep --reading occupancy --unit-area <2288MiB|4GiB|8GiB> [--user-changes <24|48|96>] [--batch-units <4|16>] [--arms <臂,…>] --product <文件>
  e202_gpu_cpu_gc_comparison concurrent --reading occupancy --unit-area <2288MiB|4GiB|8GiB> [--foreground-interval-milliseconds <50|500>] [--arms <臂,…>] --product <文件>
  e202_gpu_cpu_gc_comparison sensitivity [--user-changes <24|48|96>] [--batch-units 16] [--foreground-interval-milliseconds 500] --product <文件>
  e202_gpu_cpu_gc_comparison judge-trend --product <文件> <三档 lockstep 产物…>
  e202_gpu_cpu_gc_comparison stage-three-cost --unit-area <2288MiB|4GiB|8GiB> --product <文件>
  e202_gpu_cpu_gc_comparison construction-timing --unit-area <2288MiB|4GiB|8GiB> --live-data-units <P,…> --chunks <each-device-copies|shared> --product <文件>";

/// 命令行认出的一条命令。封闭集合。
enum CommandOfTheApparatus {
    GeometryCheck {
        product: PathBuf,
    },
    Lockstep(LockstepCommand),
    Concurrent(ConcurrentCommand),
    Sensitivity {
        product: PathBuf,
        user_changes: u64,
    },
    JudgeTrend {
        product: PathBuf,
        lockstep_products: Vec<PathBuf>,
    },
    StageThreeCost {
        tier: UnitAreaTier,
        product: PathBuf,
    },
    ConstructionTiming(ConstructionTimingCommand),
}

/// 认 `--名字 值` 形式的选项与剩下的位置参数；同一个选项给两次、选项缺值都算用法错。
fn options_and_positionals(
    arguments: &[String],
) -> Result<(BTreeMap<String, String>, Vec<String>), String> {
    let mut options: BTreeMap<String, String> = BTreeMap::new();
    let mut positionals: Vec<String> = Vec::new();
    let mut remaining = arguments.iter();
    while let Some(argument) = remaining.next() {
        if let Some(option) = argument.strip_prefix("--") {
            let option_value = remaining.next().ok_or_else(|| format!("--{option} 缺值"))?;
            if options
                .insert(option.to_string(), option_value.clone())
                .is_some()
            {
                return Err(format!("--{option} 给了两次"));
            }
        } else {
            positionals.push(argument.clone());
        }
    }
    Ok((options, positionals))
}

/// `--reading` 只认定占用读法。
fn the_reading_option_is_occupancy(options: &mut BTreeMap<String, String>) -> Result<(), String> {
    match options.remove("reading").as_deref() {
        Some("occupancy") => Ok(()),
        Some(other_reading) => Err(format!("--reading {other_reading} 认不得")),
        None => Err("缺 --reading".to_string()),
    }
}

fn tier_of_the_options(options: &mut BTreeMap<String, String>) -> Result<UnitAreaTier, String> {
    let tier_name = options.remove("unit-area").ok_or("缺 --unit-area")?;
    UnitAreaTier::of_the_name(&tier_name).ok_or_else(|| format!("--unit-area {tier_name} 认不得"))
}

/// `--user-changes` 只认前提 N 阶梯上的一级，不给取头一级。
fn user_changes_of_the_options(options: &mut BTreeMap<String, String>) -> Result<u64, String> {
    match options.remove("user-changes") {
        Some(text) => text
            .parse::<u64>()
            .ok()
            .filter(|changes| LOCKSTEP_USER_CHANGES_LADDER.contains(changes))
            .ok_or_else(|| format!("--user-changes {text} 不是阶梯上的一级")),
        None => Ok(LOCKSTEP_USER_CHANGES_LADDER[0]),
    }
}

fn command_of_the_arguments(arguments: &[String]) -> Result<CommandOfTheApparatus, String> {
    let (subcommand, rest) = arguments.split_first().ok_or("缺子命令")?;
    let (mut options, positionals) = options_and_positionals(rest)?;
    let product = PathBuf::from(options.remove("product").ok_or("缺 --product")?);
    let command = match subcommand.as_str() {
        "geometry-check" => CommandOfTheApparatus::GeometryCheck { product },
        "lockstep" => {
            the_reading_option_is_occupancy(&mut options)?;
            let tier = tier_of_the_options(&mut options)?;
            let user_changes = user_changes_of_the_options(&mut options)?;
            let batch_units = match options.remove("batch-units") {
                Some(text) => text
                    .parse::<usize>()
                    .ok()
                    .filter(|units| {
                        [
                            BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
                            BATCH_UNITS_AT_THE_SENSITIVITY_POINT,
                        ]
                        .contains(units)
                    })
                    .ok_or_else(|| format!("--batch-units {text} 不是登记的取样点"))?,
                None => BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
            };
            let arms_requested = match options.remove("arms") {
                Some(text) => text
                    .split(',')
                    .map(|name| {
                        LockstepArm::of_the_name(name).ok_or_else(|| format!("臂 {name} 认不得"))
                    })
                    .collect::<Result<Vec<LockstepArm>, String>>()?,
                None => LOCKSTEP_ARMS.to_vec(),
            };
            CommandOfTheApparatus::Lockstep(LockstepCommand {
                tier,
                user_changes,
                batch_units,
                arms_requested,
                product,
            })
        }
        "concurrent" => {
            the_reading_option_is_occupancy(&mut options)?;
            let tier = tier_of_the_options(&mut options)?;
            let foreground_interval = match options.remove("foreground-interval-milliseconds") {
                Some(text) => text
                    .parse::<u64>()
                    .ok()
                    .filter(|milliseconds| {
                        [
                            FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_MAIN_SAMPLING_POINT,
                            FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_SENSITIVITY_POINT,
                        ]
                        .contains(milliseconds)
                    })
                    .map(Duration::from_millis)
                    .ok_or_else(|| {
                        format!("--foreground-interval-milliseconds {text} 不是登记的取样点")
                    })?,
                None => Duration::from_millis(
                    FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_MAIN_SAMPLING_POINT,
                ),
            };
            let arms_requested = match options.remove("arms") {
                Some(text) => text
                    .split(',')
                    .map(|name| {
                        ConcurrentArm::of_the_name(name).ok_or_else(|| format!("臂 {name} 认不得"))
                    })
                    .collect::<Result<Vec<ConcurrentArm>, String>>()?,
                None => CONCURRENT_ARMS.to_vec(),
            };
            CommandOfTheApparatus::Concurrent(ConcurrentCommand {
                tier,
                foreground_interval,
                arms_requested,
                product,
            })
        }
        "sensitivity" => {
            // 两点的取值是登记写死的：给了别的值算用法错（文件头起法照写这两个选项）。
            if let Some(text) = options.remove("batch-units") {
                if text != BATCH_UNITS_AT_THE_SENSITIVITY_POINT.to_string() {
                    return Err(format!("sensitivity 的 --batch-units 只认 {BATCH_UNITS_AT_THE_SENSITIVITY_POINT}，给的是 {text}"));
                }
            }
            if let Some(text) = options.remove("foreground-interval-milliseconds") {
                if text != FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_SENSITIVITY_POINT.to_string() {
                    return Err(format!(
                        "sensitivity 的 --foreground-interval-milliseconds 只认 {FOREGROUND_INTERVAL_MILLISECONDS_AT_THE_SENSITIVITY_POINT}，给的是 {text}"
                    ));
                }
            }
            CommandOfTheApparatus::Sensitivity {
                user_changes: user_changes_of_the_options(&mut options)?,
                product,
            }
        }
        "judge-trend" => CommandOfTheApparatus::JudgeTrend {
            product,
            lockstep_products: positionals.iter().map(PathBuf::from).collect(),
        },
        "stage-three-cost" => CommandOfTheApparatus::StageThreeCost {
            tier: tier_of_the_options(&mut options)?,
            product,
        },
        "construction-timing" => {
            let tier = tier_of_the_options(&mut options)?;
            let live_data_units = options
                .remove("live-data-units")
                .ok_or("缺 --live-data-units")?
                .split(',')
                .map(|text| {
                    text.parse::<u64>()
                        .ok()
                        .filter(|units| the_construction_accepts_the_live_data_units(tier, *units))
                        .ok_or_else(|| {
                            format!(
                                "--live-data-units 里的 {text} 不是大于 0、32 的倍数、不超过 {} 的 P",
                                tier.live_data_units()
                            )
                        })
                })
                .collect::<Result<Vec<u64>, String>>()?;
            let storage_name = options.remove("chunks").ok_or("缺 --chunks")?;
            let storage = ChunkStorage::of_the_name(&storage_name)
                .ok_or_else(|| format!("--chunks {storage_name} 认不得"))?;
            CommandOfTheApparatus::ConstructionTiming(ConstructionTimingCommand {
                tier,
                live_data_units,
                storage,
                product,
            })
        }
        other_subcommand => return Err(format!("子命令 {other_subcommand} 认不得")),
    };
    if let Some(unknown_option) = options.keys().next() {
        return Err(format!("--{unknown_option} 认不得"));
    }
    match &command {
        CommandOfTheApparatus::JudgeTrend {
            lockstep_products, ..
        } if lockstep_products.is_empty() => {
            return Err("judge-trend 要至少一份 lockstep 产物".to_string())
        }
        CommandOfTheApparatus::GeometryCheck { .. }
        | CommandOfTheApparatus::Lockstep(_)
        | CommandOfTheApparatus::Concurrent(_)
        | CommandOfTheApparatus::Sensitivity { .. }
        | CommandOfTheApparatus::StageThreeCost { .. }
        | CommandOfTheApparatus::ConstructionTiming(_)
            if !positionals.is_empty() =>
        {
            return Err(format!("多出的参数：{}", positionals.join(" ")))
        }
        CommandOfTheApparatus::GeometryCheck { .. }
        | CommandOfTheApparatus::Lockstep(_)
        | CommandOfTheApparatus::Concurrent(_)
        | CommandOfTheApparatus::Sensitivity { .. }
        | CommandOfTheApparatus::JudgeTrend { .. }
        | CommandOfTheApparatus::StageThreeCost { .. }
        | CommandOfTheApparatus::ConstructionTiming(_) => {}
    }
    Ok(command)
}

/// 一个进程只起一条子命令；要卡的子命令在这里建这个进程的那一份 Vulkan 实例交下去（敏感度那一条的交替跑与并发跑共用它），不要卡的不建。
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let exit_code = match command_of_the_arguments(&arguments) {
        Ok(CommandOfTheApparatus::GeometryCheck { product }) => geometry_check_command(&product),
        Ok(CommandOfTheApparatus::Lockstep(command)) => lockstep_command(
            &command,
            &VulkanInstanceOfThisProcess::created_for_this_process(),
        ),
        Ok(CommandOfTheApparatus::Concurrent(command)) => concurrent_command(
            &command,
            &VulkanInstanceOfThisProcess::created_for_this_process(),
        ),
        Ok(CommandOfTheApparatus::Sensitivity {
            product,
            user_changes,
        }) => sensitivity_command(
            &product,
            user_changes,
            &VulkanInstanceOfThisProcess::created_for_this_process(),
        ),
        Ok(CommandOfTheApparatus::JudgeTrend {
            product,
            lockstep_products,
        }) => judge_trend_command(&product, &lockstep_products),
        Ok(CommandOfTheApparatus::StageThreeCost { tier, product }) => stage_three_cost_command(
            tier,
            &product,
            &VulkanInstanceOfThisProcess::created_for_this_process(),
        ),
        Ok(CommandOfTheApparatus::ConstructionTiming(command)) => {
            construction_timing_command(&command)
        }
        Err(reason) => {
            eprintln!("{reason}\n{USAGE}");
            EXIT_USAGE
        }
    };
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 造数据挂载之前树里已有的 inode 记录条数：新池新建文件建的第一个文件那一条（推的，读 `publish_first_file`；小池用例拿造池时量的回比）。
    /// 造池本身现量这个数，不用它；只有一层树的算术用例与小池用例用它，所以只在测试里定义。
    const INODE_RECORDS_BEFORE_THE_DATA_MOUNT: u64 = 1;

    fn sha256_hex_of(bytes: &[u8]) -> String {
        let mut digest = Sha256Digest::new();
        digest.feed(bytes);
        digest.finish_as_hex()
    }

    #[test]
    fn sha256_of_the_standard_messages_matches_the_known_answers() {
        assert_eq!(
            sha256_hex_of(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex_of(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex_of(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn sha256_fed_in_uneven_pieces_equals_the_digest_fed_at_once() {
        let message: Vec<u8> = (0..1000_u32)
            .map(|index| u8::try_from(index % 251).expect("小于 256"))
            .collect();
        let mut digest = Sha256Digest::new();
        for piece in message.chunks(37) {
            digest.feed(piece);
        }
        assert_eq!(digest.finish_as_hex(), sha256_hex_of(&message));
    }

    #[test]
    fn local_constants_agree_with_the_core() {
        assert_eq!(local_constants_disagreeing_with_the_core(), Vec::new());
    }

    #[test]
    fn publishes_of_one_intent_at_the_main_sampling_point_are_five() {
        assert_eq!(
            publishes_of_one_intent(BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT),
            1 + 4
        );
        assert_eq!(
            publishes_of_one_intent(BATCH_UNITS_AT_THE_SENSITIVITY_POINT),
            1 + 1
        );
    }

    #[test]
    fn the_tiers_hold_half_their_unit_area_in_live_data_units() {
        let live: Vec<u64> = UnitAreaTier::EVERY_TIER
            .iter()
            .map(|tier| tier.live_data_units())
            .collect();
        assert_eq!(live, vec![36608, 65536, 131_072]);
        let devices: Vec<u64> = UnitAreaTier::EVERY_TIER
            .iter()
            .map(|tier| tier.device_bytes() >> 20)
            .collect();
        assert_eq!(devices, vec![3072, 4880, 8976]);
        let names: Vec<&str> = UnitAreaTier::EVERY_TIER
            .iter()
            .map(|tier| tier.name())
            .collect();
        assert_eq!(names, vec!["2288MiB", "4GiB", "8GiB"]);
    }

    /// 登记第十二节修订第 4 条随档改的乙类锚点：每块盘单元区槽数、整段数、碎片段数（P ÷ 32）。
    #[test]
    fn the_tiers_have_the_registered_slots_segments_and_fragment_segments() {
        let numbers: Vec<(u64, u64, u64)> = UnitAreaTier::EVERY_TIER
            .iter()
            .map(|tier| {
                (
                    local_unit_area_slots_of_device(tier.device_bytes()),
                    tier.unit_area_segments(),
                    tier.fragment_segments(),
                )
            })
            .collect();
        assert_eq!(
            numbers,
            vec![
                (146_432, 2288, 1144),
                (262_144, 4096, 2048),
                (524_288, 8192, 4096)
            ]
        );
        assert_eq!(data_units_of_a_cluster_segment(), 32);
    }

    fn write_of(bytes: &[u8]) -> RecordedDeviceOperation<'_> {
        RecordedDeviceOperation::Write {
            device: DeviceIdentity(0),
            offset_in_bytes: 4096,
            bytes,
            durability: WriteDurability::Plain,
        }
    }

    #[test]
    fn the_recording_digest_tells_apart_two_writes_differing_in_one_byte() {
        let mut one_write = Sha256Digest::new();
        feed_the_operation_into_the_digest(&mut one_write, write_of(&[1, 2, 3, 4]));
        let mut other_write = Sha256Digest::new();
        feed_the_operation_into_the_digest(&mut other_write, write_of(&[1, 2, 3, 5]));
        assert_ne!(one_write.finish_as_hex(), other_write.finish_as_hex());
    }

    #[test]
    fn the_memory_device_reads_back_what_was_written_and_zeroes_after_a_discard() {
        let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
        let mut devices = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
        let (_identity, device) = &mut devices[0];
        let written: Vec<u8> = (0..1024_u32)
            .map(|index| u8::try_from(index % 256).expect("小于 256"))
            .collect();
        let offset = DeviceOffsetInBytes(MEMORY_DEVICE_CHUNK_BYTES - 512);
        device
            .write_at(offset, &written, WriteDurability::Plain)
            .expect("跨块写");
        let mut read_back = vec![0u8; 1024];
        device.read_at(offset, &mut read_back).expect("跨块读");
        assert_eq!(read_back, written);
        device
            .discard_at(
                DeviceOffsetInBytes(MEMORY_DEVICE_CHUNK_BYTES),
                MEMORY_DEVICE_CHUNK_BYTES,
            )
            .expect("整块 discard");
        device.read_at(offset, &mut read_back).expect("读");
        assert_eq!(&read_back[..512], &written[..512]);
        assert_eq!(&read_back[512..], &[0u8; 512][..]);
        assert_eq!(recording.lock().expect("录制").operations, 2);
    }

    /// 一个块那么长、没有一个字节是零的内容（下标 i 处是 i mod 251 + 1 再加 `shift`）。
    fn chunk_content(shift: u8) -> Vec<u8> {
        (0..MEMORY_DEVICE_CHUNK_BYTES)
            .map(|index| {
                u8::try_from(index % 251 + 1)
                    .expect("小于 256")
                    .wrapping_add(shift)
                    .max(1)
            })
            .collect()
    }

    fn read_a_chunk(device: &RecordingMemoryDevice, chunk_index: u64) -> Vec<u8> {
        let mut read_back = vec![0u8; usize::try_from(MEMORY_DEVICE_CHUNK_BYTES).expect("64 KiB")];
        device
            .read_at(
                DeviceOffsetInBytes(chunk_index * MEMORY_DEVICE_CHUNK_BYTES),
                &mut read_back,
            )
            .expect("块内读");
        read_back
    }

    /// 共用的存法（[`ChunkStorage::SharedWhenTheBytesAreEqual`]）：两块盘先后往同一偏移写同一份字节（核心层每个单元两块盘各写一遍），
    /// 按半块写两次也一样，存下的只有一份，两块盘读回的都是写的字节。
    #[test]
    fn two_devices_writing_the_same_bytes_at_the_same_offset_store_one_chunk() {
        let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
        let mut devices = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
        let written = chunk_content(0);
        let half = written.len() / 2;
        for (offset, bytes) in [
            (MEMORY_DEVICE_CHUNK_BYTES, &written[..half]),
            (
                MEMORY_DEVICE_CHUNK_BYTES + u64::try_from(half).expect("半块"),
                &written[half..],
            ),
        ] {
            for (_identity, device) in &mut devices {
                device
                    .write_at(DeviceOffsetInBytes(offset), bytes, WriteDurability::Plain)
                    .expect("半块写");
            }
        }
        assert_eq!(
            chunk_bytes_of_the_pool(&devices),
            ChunkBytesOfThePool {
                held_by_the_devices: 2 * MEMORY_DEVICE_CHUNK_BYTES,
                stored: MEMORY_DEVICE_CHUNK_BYTES,
            }
        );
        for (_identity, device) in &devices {
            assert_eq!(read_a_chunk(device, 1), written);
        }
    }

    /// 共用的存法：写成全是零的块不存，读回照旧是零；各存各的存法照存。
    #[test]
    fn chunk_written_back_to_zeroes_is_not_stored_and_reads_back_zero() {
        let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
        let mut shared = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
        let mut copied = empty_pool_devices_storing_the_chunks(
            4 * MEMORY_DEVICE_CHUNK_BYTES,
            &recording,
            ChunkStorage::EachDeviceCopiesItsChunks,
        );
        for devices in [&mut shared, &mut copied] {
            let (_identity, device) = &mut devices[0];
            device
                .write_at(DeviceOffsetInBytes(0), &[7u8; 512], WriteDurability::Plain)
                .expect("写");
            device
                .write_at(DeviceOffsetInBytes(0), &[0u8; 512], WriteDurability::Plain)
                .expect("写回零");
            assert!(read_a_chunk(device, 0).iter().all(|byte| *byte == 0));
        }
        assert_eq!(
            chunk_bytes_of_the_pool(&shared),
            ChunkBytesOfThePool::default()
        );
        assert_eq!(
            chunk_bytes_of_the_pool(&copied).stored,
            MEMORY_DEVICE_CHUNK_BYTES
        );
    }

    /// 共用的存法：拷一个池只拷块的引用（拷出来的与原来的指同一份），拷出来的那个池写到一块才复制那一块，原来的读回照旧。
    #[test]
    fn copied_pool_shares_the_chunks_until_one_of_its_devices_writes_a_chunk() {
        let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::empty()));
        let mut devices = empty_pool_devices(4 * MEMORY_DEVICE_CHUNK_BYTES, &recording);
        let written = chunk_content(0);
        for (_identity, device) in &mut devices {
            device
                .write_at(DeviceOffsetInBytes(0), &written, WriteDurability::Plain)
                .expect("整块写");
        }
        let (mut copied, _copied_recording) = copied_pool_devices(&devices);
        assert!(Arc::ptr_eq(
            &devices[0].1.chunks[&0],
            &copied[0].1.chunks[&0]
        ));
        let rewritten = chunk_content(1);
        copied[0]
            .1
            .write_at(DeviceOffsetInBytes(0), &rewritten, WriteDurability::Plain)
            .expect("拷出来的池写一块");
        assert_eq!(read_a_chunk(&copied[0].1, 0), rewritten);
        assert_eq!(read_a_chunk(&copied[1].1, 0), written);
        for (_identity, device) in &devices {
            assert_eq!(read_a_chunk(device, 0), written);
        }
        assert_eq!(
            chunk_bytes_of_the_pool(&devices).stored,
            MEMORY_DEVICE_CHUNK_BYTES
        );
        assert_eq!(
            chunk_bytes_of_the_pool(&copied).stored,
            2 * MEMORY_DEVICE_CHUNK_BYTES
        );
    }

    /// 造法对拍用的最小池：P = 64（碎片段 2 段），盘取默认环长下 mkfs 认的最小一块；造得快，只核造法、不核几何成没成形。
    const LIVE_DATA_UNITS_OF_THE_SMALLEST_POOL: u64 = 64;

    fn smallest_pool_on_the_clock(
        storage: ChunkStorage,
        clock: &mut ConstructionClock<'_>,
    ) -> StartingImage {
        construct_a_starting_image_on_the_clock(
            UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
            smallest_device_bytes_the_make_filesystem_accepts(),
            LIVE_DATA_UNITS_OF_THE_SMALLEST_POOL,
            storage,
            clock,
        )
    }

    /// 改前（各存各的）与改后（同内容块共用、零块不存）两种存法照同一串改动造同一个池：发成的版数、抬 F、几何读数、碎片段、全空段数、
    /// 映射条数逐项相同，两块盘上每一个块读回的字节逐个相同；共用的那一份占的块字节少于各存各的。
    #[test]
    fn the_two_chunk_storages_build_the_same_pool_from_the_same_changes() {
        let build = |storage: ChunkStorage| {
            let mut progress_dropped = |_line: &str| {};
            let mut clock = ConstructionClock::new(
                format!("对拍-{}", storage.name()),
                CONSTRUCTION_PROGRESS_INTERVAL,
                &mut progress_dropped,
            );
            smallest_pool_on_the_clock(storage, &mut clock)
        };
        let copied = build(ChunkStorage::EachDeviceCopiesItsChunks);
        let shared = build(ChunkStorage::SharedWhenTheBytesAreEqual);
        let numbers = |image: &StartingImage| {
            let construction = &image.construction;
            (
                construction.publishes,
                construction.floor_raises,
                construction.release_rounds,
                construction.spacer_units,
                construction.alignment_spacer_units,
                construction.live_data_units_found,
                construction.empty_segments,
                construction.mapped_units,
            )
        };
        assert_eq!(numbers(&copied), numbers(&shared));
        assert_eq!(copied.construction.geometry, shared.construction.geometry);
        assert_eq!(
            copied.fragment_segment_starts,
            shared.fragment_segment_starts
        );
        assert_eq!(copied.lowest_live_data_inode, shared.lowest_live_data_inode);
        for ((copied_identity, copied_device), (shared_identity, shared_device)) in
            copied.devices.iter().zip(&shared.devices)
        {
            assert_eq!(copied_identity, shared_identity);
            let chunk_indexes: BTreeSet<u64> = copied_device
                .chunks
                .keys()
                .chain(shared_device.chunks.keys())
                .copied()
                .collect();
            for chunk_index in chunk_indexes {
                assert!(
                    read_a_chunk(copied_device, chunk_index)
                        == read_a_chunk(shared_device, chunk_index),
                    "盘 {copied_identity:?} 块 {chunk_index} 两种存法读回的字节不同"
                );
            }
        }
        assert!(
            chunk_bytes_of_the_pool(&shared.devices).stored
                < chunk_bytes_of_the_pool(&copied.devices).stored
        );
    }

    /// 进度间隔取 0：造池的每一个阶段都打出进度行（阶段名、已做/至多几步、已用秒数、常驻内存），各阶段的读数按造的先后一阶段一项，
    /// 各阶段发成的版数加起来等于造池一共发成的，顺序写那一半与交错那一半各 P ÷ 2 步。
    #[test]
    fn every_construction_phase_prints_a_progress_line_when_the_interval_is_zero() {
        let mut lines: Vec<String> = Vec::new();
        let mut collect_the_line = |line: &str| lines.push(line.to_string());
        let image = {
            let mut clock =
                ConstructionClock::new("进度".to_string(), Duration::ZERO, &mut collect_the_line);
            smallest_pool_on_the_clock(ChunkStorage::SharedWhenTheBytesAreEqual, &mut clock)
        };
        for phase in ConstructionPhase::EVERY_PHASE {
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains(&format!(" phase={} ", phase.name()))),
                "阶段 {} 没有进度行",
                phase.name()
            );
        }
        assert!(lines
            .iter()
            .all(|line| line.starts_with("E202 造池进度 pool=进度 ")
                && line.contains(" elapsed_seconds=")
                && line.contains(" resident_bytes=")));
        let phases = &image.construction.phases;
        assert_eq!(
            phases.iter().map(|timing| timing.phase).collect::<Vec<_>>(),
            ConstructionPhase::EVERY_PHASE.to_vec()
        );
        assert_eq!(
            phases.iter().map(|timing| timing.publishes).sum::<u64>(),
            image.construction.publishes
        );
        let steps_of = |phase: ConstructionPhase| {
            phases
                .iter()
                .find(|timing| timing.phase == phase)
                .map(|timing| timing.steps)
        };
        assert_eq!(
            steps_of(ConstructionPhase::SequentialHalf),
            Some(LIVE_DATA_UNITS_OF_THE_SMALLEST_POOL / 2)
        );
        assert_eq!(
            steps_of(ConstructionPhase::InterleavedHalf),
            Some(LIVE_DATA_UNITS_OF_THE_SMALLEST_POOL / 2)
        );
        assert_eq!(
            steps_of(ConstructionPhase::UnlinkingTheSpacerFiles),
            Some(image.construction.spacer_files)
        );
    }

    #[test]
    fn the_resident_bytes_are_read_off_the_resident_set_size_line() {
        assert_eq!(
            resident_bytes_of_the_status("Name:\tx\nVmRSS:\t   2048 kB\nThreads:\t1\n"),
            Some(2048 * 1024)
        );
        assert_eq!(resident_bytes_of_the_status("Name:\tx\n"), None);
        assert!(resident_bytes_of_this_process().is_some_and(|bytes| bytes > 0));
    }

    fn reading(
        process_milliseconds: u64,
        gc_thread_milliseconds: u64,
        foreground_milliseconds: u64,
        main_milliseconds: u64,
    ) -> CallWindowReading {
        CallWindowReading {
            wall: Duration::from_millis(1000 + process_milliseconds),
            process_cpu: Duration::from_millis(process_milliseconds),
            gc_thread_cpu: Duration::from_millis(gc_thread_milliseconds),
            foreground_thread_cpu: Duration::from_millis(foreground_milliseconds),
            main_thread_cpu: Duration::from_millis(main_milliseconds),
            cpu_by_task: BTreeMap::new(),
        }
    }

    /// 同 [`reading`]，另带这一端读到的每条线程的 schedstat（线程号，毫秒）。
    fn reading_with_the_tasks(
        process_milliseconds: u64,
        gc_thread_milliseconds: u64,
        tasks: &[(u64, u64)],
    ) -> CallWindowReading {
        CallWindowReading {
            cpu_by_task: tasks
                .iter()
                .map(|(task, milliseconds)| (*task, Duration::from_millis(*milliseconds)))
                .collect(),
            ..reading(process_milliseconds, gc_thread_milliseconds, 0, 0)
        }
    }

    /// 登记第十二节修订第 1 条：窗口里有线程退出、或线程号让新线程重用了（关窗时那一号的 CPU 比开窗时少）的那一窗作废（第十一节作废第 1 条），
    /// 两端之和相减仍不是负的也作废。小池单测（任务 6790）读到过这样一窗：进程口径 170.84191ms、GC 线程口径 214.0609ms。
    /// 这里线程 7 开窗时用掉 60 毫秒、窗口里退出：GC 线程（线程 3）走了 40、线程 8 走了 20，两端之和只多 0，进程口径 0 小于 GC 线程口径 40。
    #[test]
    fn the_window_over_which_any_task_exited_or_its_number_was_reused_is_void() {
        let opening = reading_with_the_tasks(150, 50, &[(3, 50), (7, 60), (8, 40)]);
        let every_task_stays = reading_with_the_tasks(210, 90, &[(3, 90), (7, 60), (8, 60)]);
        let window = call_window_between(opening.clone(), every_task_stays).expect("没有线程退出");
        assert_eq!(window.process_cpu, Duration::from_millis(40 + 20));
        assert_eq!(window.gc_thread_cpu, Duration::from_millis(40));
        let task_seven_exited = reading_with_the_tasks(150, 90, &[(3, 90), (8, 60)]);
        assert_eq!(
            call_window_between(opening.clone(), task_seven_exited),
            None
        );
        let task_seven_reused = reading_with_the_tasks(155, 90, &[(3, 90), (7, 5), (8, 60)]);
        assert_eq!(call_window_between(opening, task_seven_reused), None);
    }

    #[test]
    fn process_cpu_of_the_window_subtracts_the_foreground_and_main_threads_but_keeps_the_driver_threads(
    ) {
        let window = call_window_between(reading(100, 50, 10, 5), reading(200, 90, 11, 7))
            .expect("增量都不是负的");
        assert_eq!(window.process_cpu, Duration::from_millis(100 - 1 - 2));
        assert_eq!(window.gc_thread_cpu, Duration::from_millis(40));
        assert_eq!(window.wall, Duration::from_millis(100));
    }

    #[test]
    fn the_window_whose_clock_goes_backwards_is_not_counted() {
        assert_eq!(
            call_window_between(reading(200, 50, 10, 5), reading(100, 90, 11, 7)),
            None
        );
    }

    /// 假的窗口钟：进程 CPU 由调用与多判两段各自往前拨。
    struct ClocksAdvancedByTheWork {
        process_cpu: Cell<Duration>,
    }

    impl CallWindowClocks for ClocksAdvancedByTheWork {
        fn read(&self, _end: EndOfTheWindow) -> Result<CallWindowReading, ClockReadFailed> {
            Ok(CallWindowReading {
                wall: self.process_cpu.get(),
                process_cpu: self.process_cpu.get(),
                gc_thread_cpu: self.process_cpu.get(),
                foreground_thread_cpu: Duration::ZERO,
                main_thread_cpu: Duration::ZERO,
                cpu_by_task: BTreeMap::new(),
            })
        }
    }

    #[test]
    fn the_cpu_rejudging_inside_the_window_counts_toward_the_process_cpu_of_the_call() {
        let clocks = ClocksAdvancedByTheWork {
            process_cpu: Cell::new(Duration::ZERO),
        };
        let mut calls = 0_u32;
        let measured = measure_one_gc_call(
            &clocks,
            &mut calls,
            |calls| {
                *calls += 1;
                clocks
                    .process_cpu
                    .set(clocks.process_cpu.get() + Duration::from_millis(3));
            },
            |_calls, ()| {
                clocks
                    .process_cpu
                    .set(clocks.process_cpu.get() + Duration::from_millis(7));
                (Some(Duration::from_millis(7)), RejudgeTally::default())
            },
        );
        assert_eq!(calls, 1);
        let window = measured.window.expect("钟都读得到");
        assert_eq!(window.process_cpu, Duration::from_millis(3 + 7));
        assert_eq!(measured.inner_rejudge_cpu, Duration::from_millis(7));
    }

    #[test]
    fn an_unreadable_inner_timing_voids_the_window() {
        let clocks = ClocksAdvancedByTheWork {
            process_cpu: Cell::new(Duration::ZERO),
        };
        let measured = measure_one_gc_call(
            &clocks,
            &mut (),
            |_state| (),
            |_state, ()| (None, RejudgeTally::default()),
        );
        assert!(measured.window.is_none());
    }

    #[test]
    fn the_saving_is_the_cpu_arm_minus_the_gpu_arm() {
        assert_eq!(
            paired_differences(&[10.0, 20.0], &[4.0, 25.0]),
            vec![6.0, -5.0]
        );
    }

    #[test]
    fn paired_differences_all_positive_all_negative_or_mixed_are_told_apart() {
        assert_eq!(
            sign_of_the_paired_differences(&[1.0, 2.0, 3.0, 4.0, 5.0], 0.0),
            SignOfThePairedDifferences::EveryOnePositive
        );
        assert_eq!(
            sign_of_the_paired_differences(&[-1.0, -2.0, -3.0, -4.0, -5.0], 0.0),
            SignOfThePairedDifferences::EveryOneNegative
        );
        assert_eq!(
            sign_of_the_paired_differences(&[1.0, -2.0, 3.0, 4.0, 5.0], 0.0),
            SignOfThePairedDifferences::Unstable
        );
        assert_eq!(
            sign_of_the_paired_differences(&[1.0, 0.0, 3.0, 4.0, 5.0], 0.0),
            SignOfThePairedDifferences::Unstable
        );
        assert_eq!(
            sign_of_the_paired_differences(&[], 0.0),
            SignOfThePairedDifferences::Unstable
        );
    }

    /// 登记第八节判别力自证：把量 S 的判定门槛从 0 挪到两个取样点的中位之间，主取样点与相反取样点一绿一红。
    #[test]
    fn moving_the_saving_threshold_between_the_two_sampling_points_turns_one_green_and_one_red() {
        let main_sampling_point = [5.0, 6.0, 7.0, 6.0, 5.5];
        let opposite_sampling_point = [1.0, 1.2, 0.8, 1.1, 0.9];
        assert_eq!(
            sign_of_the_paired_differences(&main_sampling_point, 0.0),
            SignOfThePairedDifferences::EveryOnePositive
        );
        assert_eq!(
            sign_of_the_paired_differences(&opposite_sampling_point, 0.0),
            SignOfThePairedDifferences::EveryOnePositive
        );
        let between = f64::midpoint(
            median_of(&main_sampling_point),
            median_of(&opposite_sampling_point),
        );
        assert_eq!(
            sign_of_the_paired_differences(&main_sampling_point, between),
            SignOfThePairedDifferences::EveryOnePositive
        );
        assert_eq!(
            sign_of_the_paired_differences(&opposite_sampling_point, between),
            SignOfThePairedDifferences::EveryOneNegative
        );
    }

    #[test]
    fn median_minimum_maximum_and_median_absolute_deviation_of_five_values() {
        let values = [3.0, 1.0, 4.0, 1.0, 5.0];
        assert_eq!(median_of(&values), 3.0);
        assert_eq!(minimum_of(&values), 1.0);
        assert_eq!(maximum_of(&values), 5.0);
        assert_eq!(median_absolute_deviation_of(&values), 2.0);
        assert_eq!(median_of(&[1.0, 2.0, 3.0, 10.0]), 2.5);
    }

    fn five_around(center: f64) -> Vec<f64> {
        vec![
            center - 2.0,
            center - 1.0,
            center,
            center + 1.0,
            center + 2.0,
        ]
    }

    #[test]
    fn the_trend_rises_or_falls_when_both_steps_go_the_same_way() {
        assert_eq!(
            trend_over_the_tiers(&[five_around(10.0), five_around(20.0), five_around(30.0)]),
            TrendOverTheTiers::Rising
        );
        assert_eq!(
            trend_over_the_tiers(&[five_around(30.0), five_around(20.0), five_around(10.0)]),
            TrendOverTheTiers::Falling
        );
        assert_eq!(
            trend_over_the_tiers(&[five_around(10.0), five_around(20.0), five_around(10.0)]),
            TrendOverTheTiers::CannotTell
        );
    }

    #[test]
    fn overlapping_tiers_within_ten_percent_are_flat() {
        assert_eq!(
            trend_over_the_tiers(&[five_around(100.0), five_around(102.0), five_around(105.0)]),
            TrendOverTheTiers::Flat
        );
    }

    /// 第九节变异「平的宽度 10% 改成 50%」在这里红：顶档与最小档中位差 30% 而三档都交叠，是「说不出」不是「平」。
    #[test]
    fn thirty_percent_between_the_end_tiers_is_not_flat() {
        let wide = |center: f64| {
            vec![
                center - 40.0,
                center - 20.0,
                center,
                center + 20.0,
                center + 40.0,
            ]
        };
        assert_eq!(
            trend_over_the_tiers(&[wide(100.0), wide(115.0), wide(130.0)]),
            TrendOverTheTiers::CannotTell
        );
    }

    #[test]
    fn positive_control_holds_when_the_delta_matches_the_inner_timing() {
        let plain = [100.0, 110.0, 105.0, 95.0, 100.0];
        let rejudged = [150.0, 160.0, 154.0, 146.0, 150.0];
        let inner = [50.0, 50.0, 50.0, 50.0, 50.0];
        let cpu_arm = [200.0, 210.0, 205.0, 195.0, 200.0];
        let control = positive_control_of_an_arm(&plain, &rejudged, &inner, Some(&cpu_arm));
        assert!(control.every_delta_positive);
        assert_eq!(control.median_delta, 50.0);
        assert!(control.median_delta_within_the_band_of_the_inner_timing);
        assert_eq!(
            control.saving_with_rejudging_below_the_saving_in_every_pair,
            Some(true)
        );
        assert!(control.holds());
    }

    #[test]
    fn positive_control_fails_when_the_rejudging_adds_nothing_or_far_more_than_the_inner_timing() {
        let plain = [100.0, 110.0, 105.0, 95.0, 100.0];
        let inner = [50.0, 50.0, 50.0, 50.0, 50.0];
        let adding_nothing = positive_control_of_an_arm(&plain, &plain, &inner, None);
        assert!(!adding_nothing.every_delta_positive);
        assert!(!adding_nothing.holds());
        let far_more = [300.0, 310.0, 305.0, 295.0, 300.0];
        let adding_far_more = positive_control_of_an_arm(&plain, &far_more, &inner, None);
        assert!(adding_far_more.every_delta_positive);
        assert!(!adding_far_more.median_delta_within_the_band_of_the_inner_timing);
        assert!(!adding_far_more.holds());
    }

    fn call_record_with(
        batches: u64,
        compute_records: Vec<BackgroundComputeOfOneBatch>,
    ) -> GcCallRecord {
        GcCallRecord {
            after_the_user_change: 0,
            window: None,
            inner_rejudge_cpu: Duration::ZERO,
            rejudge_tally: RejudgeTally::default(),
            started: batches > 0,
            compaction_publishes: batches,
            batches,
            units_relocated_of_each_batch: Vec::new(),
            stop_name: String::new(),
            compute_records,
            has_the_alarm: false,
            empty_segments_before_the_call: Vec::new(),
            empty_segments_after_the_call: Vec::new(),
        }
    }

    fn window_of_wall(wall: Duration) -> CallWindow {
        CallWindow {
            wall,
            process_cpu: Duration::ZERO,
            gc_thread_cpu: Duration::ZERO,
            foreground_thread_cpu: Duration::ZERO,
            main_thread_cpu: Duration::ZERO,
        }
    }

    /// 第九节变异「只计发了批的 GC 调用」在这里红：不起跑的那一次调用的 1 秒照算进量 W。
    #[test]
    fn the_gc_wall_clock_counts_the_calls_that_published_no_batch() {
        let windows = [
            window_of_wall(Duration::from_millis(250)),
            window_of_wall(Duration::from_secs(1)),
        ];
        let records = [
            call_record_with(3, Vec::new()),
            call_record_with(0, Vec::new()),
        ];
        assert_eq!(
            gc_wall_clock_of_the_run(&windows, &records),
            Duration::from_millis(1250)
        );
    }

    #[test]
    fn the_most_compaction_publishes_between_two_user_changes_are_summed_per_change() {
        let mut after_the_first_change = call_record_with(12, Vec::new());
        after_the_first_change.after_the_user_change = 0;
        let mut again_after_the_first_change = call_record_with(8, Vec::new());
        again_after_the_first_change.after_the_user_change = 0;
        let mut after_the_next_change = call_record_with(5, Vec::new());
        after_the_next_change.after_the_user_change = 1;
        assert_eq!(
            most_compaction_publishes_between_two_user_changes(&[
                after_the_first_change,
                again_after_the_first_change,
                after_the_next_change
            ]),
            12 + 8
        );
    }

    fn two_discrete_gpus() -> Vec<DiscreteGpuSeenByThisProcess> {
        vec![
            DiscreteGpuSeenByThisProcess {
                pci_bus_address: "0000:01:00.0".to_string(),
                model: "lower".to_string(),
            },
            DiscreteGpuSeenByThisProcess {
                pci_bus_address: "0000:02:00.0".to_string(),
                model: "higher".to_string(),
            },
        ]
    }

    /// 第九节变异「全部卡臂也只点一张」在这里红。
    #[test]
    fn the_every_card_arm_names_no_card_and_gives_one_limit_per_card() {
        let settings = background_compute_settings_of_the_arm(
            ComputeOfTheArm::GpuOnEveryCard,
            &two_discrete_gpus(),
        );
        assert_eq!(settings.background_compute, BackgroundComputeBackend::Gpu);
        assert_eq!(settings.background_gpu_devices, None);
        assert_eq!(
            settings.background_gpu_memory_limit,
            Some(vec![GpuMemoryInBytes(1 << 30); 2])
        );
    }

    #[test]
    fn the_one_card_arm_names_the_card_with_the_lowest_pci_address() {
        let settings = background_compute_settings_of_the_arm(
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
            &two_discrete_gpus(),
        );
        assert_eq!(
            settings.background_gpu_devices,
            Some(vec![gpu_device_identifier_of_the_pci_bus_address(
                "0000:01:00.0"
            )])
        );
        assert_eq!(
            settings.background_gpu_memory_limit,
            Some(vec![GpuMemoryInBytes(1 << 30)])
        );
        assert_eq!(
            background_compute_settings_of_the_arm(ComputeOfTheArm::Cpu, &[]),
            BackgroundComputeSettings::PRODUCT
        );
    }

    #[test]
    fn arms_needing_more_cards_than_seen_are_not_run() {
        assert_eq!(reason_the_arm_is_not_run(ComputeOfTheArm::Cpu, 0), None);
        assert!(
            reason_the_arm_is_not_run(ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress, 0)
                .is_some()
        );
        assert_eq!(
            reason_the_arm_is_not_run(ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress, 1),
            None
        );
        assert!(reason_the_arm_is_not_run(ComputeOfTheArm::GpuOnEveryCard, 1).is_some());
        assert_eq!(
            reason_the_arm_is_not_run(ComputeOfTheArm::GpuOnEveryCard, 2),
            None
        );
    }

    /// 第九节变异「交替跑的假钟不前进」在这里红。
    #[test]
    fn the_fake_clock_of_the_lockstep_run_advances_one_millisecond_per_call() {
        let caller = CallerOfTheLockstepRun::starting_at_zero();
        caller.advance_before_the_next_call();
        caller.advance_before_the_next_call();
        assert_eq!(
            caller.clock_reading(),
            CallerMonotonicClockReading(Duration::from_millis(2))
        );
        assert_eq!(caller.foreground_fsync(), ForegroundFsync::NoneWaiting);
    }

    fn gpu_record_giving_work_to(cards: Vec<GpuDeviceIdentifier>) -> BackgroundComputeOfOneBatch {
        let mut work = singlefs_core::background_compute::GpuWorkOfTheBatch::default();
        work.submissions_on_the_gpus.unit_checksum_verification = GpuSubmissionsOfOneSegment {
            submissions_completed: 1,
            pieces_submitted: 1,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: cards,
        };
        BackgroundComputeOfOneBatch {
            task: singlefs_core::background_compute::BackgroundTask::CompactionRelocation,
            requested_backend: BackgroundComputeBackend::Gpu,
            path_taken: BackgroundComputePathTaken::Gpu(work),
            compute_error_that_abandoned_the_batch: None,
        }
    }

    /// 第九节变异「甲类锚点『分到活的卡』改成不核」在这里红。
    #[test]
    fn the_batch_giving_work_to_a_second_card_breaks_the_anchor() {
        let lowest = gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0");
        let higher = gpu_device_identifier_of_the_pci_bus_address("0000:02:00.0");
        let on_the_lowest = [call_record_with(
            1,
            vec![gpu_record_giving_work_to(vec![lowest.clone()])],
        )];
        assert!(gpu_work_anchors(&on_the_lowest, &lowest).hold());
        let on_both = [call_record_with(
            1,
            vec![gpu_record_giving_work_to(vec![lowest.clone(), higher])],
        )];
        let anchors = gpu_work_anchors(&on_both, &lowest);
        assert_eq!(anchors.segments_given_to_a_card_other_than_the_lowest, 1);
        assert!(!anchors.hold());
    }

    #[test]
    fn the_path_anchor_counts_cpu_batches_on_a_cpu_arm_and_gpu_batches_on_a_gpu_arm() {
        let lowest = gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0");
        let records = [call_record_with(
            1,
            vec![gpu_record_giving_work_to(vec![lowest])],
        )];
        let on_a_gpu_arm = path_anchor(
            &records,
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        );
        assert_eq!(on_a_gpu_arm.compute_records_on_the_path_of_the_settings, 1);
        assert!(on_a_gpu_arm.holds());
        let on_a_cpu_arm = path_anchor(&records, ComputeOfTheArm::Cpu);
        assert_eq!(on_a_cpu_arm.compute_records_on_the_path_of_the_settings, 0);
        assert!(!on_a_cpu_arm.holds());
    }

    fn cpu_record_requested_by_the_settings() -> BackgroundComputeOfOneBatch {
        BackgroundComputeOfOneBatch {
            task: singlefs_core::background_compute::BackgroundTask::CompactionRelocation,
            requested_backend: BackgroundComputeBackend::Cpu,
            path_taken: BackgroundComputePathTaken::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings,
            ),
            compute_error_that_abandoned_the_batch: None,
        }
    }

    /// 登记第十二节修订第 2 条：条数比批数多（签发、删意图、停在判据上的那几项也记账）而每一项都照设置走，判过；有一项走错，
    /// 或条数比批数少，各判不过。
    #[test]
    fn the_path_anchor_holds_with_more_records_than_batches_and_breaks_on_a_wrong_path_or_fewer_records(
    ) {
        let lowest = gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0");
        let three_on_the_gpu = [call_record_with(
            1,
            vec![
                gpu_record_giving_work_to(vec![lowest.clone()]),
                gpu_record_giving_work_to(vec![lowest.clone()]),
                gpu_record_giving_work_to(vec![lowest.clone()]),
            ],
        )];
        let more_records_than_batches = path_anchor(
            &three_on_the_gpu,
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        );
        assert_eq!(more_records_than_batches.compute_records, 3);
        assert_eq!(more_records_than_batches.batches, 1);
        assert!(more_records_than_batches.holds());
        let one_on_the_cpu = [call_record_with(
            1,
            vec![
                gpu_record_giving_work_to(vec![lowest.clone()]),
                cpu_record_requested_by_the_settings(),
                gpu_record_giving_work_to(vec![lowest.clone()]),
            ],
        )];
        let one_on_the_wrong_path = path_anchor(
            &one_on_the_cpu,
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        );
        assert_eq!(
            one_on_the_wrong_path.compute_records_on_the_path_of_the_settings,
            2
        );
        assert!(!one_on_the_wrong_path.holds());
        let one_record_for_two_batches = [call_record_with(
            2,
            vec![gpu_record_giving_work_to(vec![lowest])],
        )];
        let fewer_records_than_batches = path_anchor(
            &one_record_for_two_batches,
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        );
        assert_eq!(fewer_records_than_batches.compute_records, 1);
        assert_eq!(fewer_records_than_batches.batches, 2);
        assert!(!fewer_records_than_batches.holds());
    }

    /// 在 CPU 上算的替身：校验和段与活槽段照 CPU 写法交回「两份核确认」，`corrupted_crc32c` 为真时把校验和段的结果算错一位。
    struct CpuBackedStandIn {
        corrupted_crc32c: bool,
    }

    impl GpuInventory for CpuBackedStandIn {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            Vec::new()
        }
    }

    impl BackgroundGpuCompute for CpuBackedStandIn {
        fn verify_unit_checksums(
            &mut self,
            units: &[UnitChecksumToVerifyOnTheGpu<'_>],
        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
        {
            Ok(units
                .iter()
                .map(|unit| {
                    let computed =
                        crc32_castagnoli(unit.unit_bytes) ^ u32::from(self.corrupted_crc32c);
                    GpuEntryDecision::ConfirmedByBothKernels(UnitChecksumOutcome {
                        computed_crc32c: computed,
                        matches_the_expected_crc32c: computed == unit.expected_crc32c,
                    })
                })
                .collect())
        }

        fn classify_region_records(
            &mut self,
            _region: CompactionRegion,
            records: &[RegionRecordToClassifyOnTheGpu<'_>],
            _mapping_entries: &[CentralMappingEntry],
        ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
            Ok(records
                .iter()
                .map(|_record| GpuEntryDecision::UncertainThisRound)
                .collect())
        }

        fn count_live_slots_per_cluster_segment(
            &mut self,
            allocation_records: &[AllocationRecord],
            unit_area: UnitAreaOfOneDevice,
            segments: &[ClusterSegmentEntryKey],
        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
            let live_slots =
                live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area);
            Ok(segments
                .iter()
                .zip(live_slots)
                .map(|(_segment, live)| GpuEntryDecision::ConfirmedByBothKernels(live))
                .collect())
        }

        fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}
    }

    fn units_to_verify(contents: &[Vec<u8>]) -> Vec<UnitChecksumToVerifyOnTheGpu<'_>> {
        contents
            .iter()
            .zip(0_u64..)
            .map(|(content, slot)| UnitChecksumToVerifyOnTheGpu {
                entry_key: singlefs_core::background_compute::RegionRecordEntryKey {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(LOCAL_UNIT_AREA_START_SLOT + 2 * slot),
                    span_slots: 2,
                },
                unit_bytes: content,
                expected_crc32c: crc32_castagnoli(content),
            })
            .collect()
    }

    /// 第九节变异「GPU 多判包装不再判」在这里红：包装把每条确认的校验和都在 CPU 上再判了一遍，对不上的条数是 0。
    #[test]
    fn the_rejudging_wrapper_rejudges_every_confirmed_checksum_on_the_cpu() {
        let log: SharedRejudgeLog = Arc::new(Mutex::new(RejudgeLog::default()));
        let mut wrapper = GpuComputeRejudgedOnTheCpu {
            inner: CpuBackedStandIn {
                corrupted_crc32c: false,
            },
            log: Arc::clone(&log),
        };
        let contents: Vec<Vec<u8>> = (0..3_u8).map(|seed| vec![seed; 32768]).collect();
        let decisions = wrapper
            .verify_unit_checksums(&units_to_verify(&contents))
            .expect("替身不出错");
        assert_eq!(decisions.len(), 3);
        let _submissions = wrapper.take_the_submissions_of_the_batch();
        let log = log.lock().expect("记录");
        assert_eq!(
            log.tally,
            RejudgeTally {
                entries_rejudged: 3,
                mismatches: 0
            }
        );
        assert_eq!(
            log.handed_in_of_each_batch,
            vec![EntriesHandedIn {
                unit_checksums: 3,
                region_records: 0,
                cluster_segments: 0
            }]
        );
        assert_eq!(log.clock_read_failures, 0);
    }

    #[test]
    fn the_rejudging_wrapper_counts_a_checksum_the_gpu_confirmed_wrongly() {
        let log: SharedRejudgeLog = Arc::new(Mutex::new(RejudgeLog::default()));
        let mut wrapper = GpuComputeRejudgedOnTheCpu {
            inner: CpuBackedStandIn {
                corrupted_crc32c: true,
            },
            log: Arc::clone(&log),
        };
        let contents: Vec<Vec<u8>> = (0..2_u8).map(|seed| vec![seed; 32768]).collect();
        let _decisions = wrapper
            .verify_unit_checksums(&units_to_verify(&contents))
            .expect("替身不出错");
        assert_eq!(
            log.lock().expect("记录").tally,
            RejudgeTally {
                entries_rejudged: 2,
                mismatches: 2
            }
        );
    }

    #[test]
    fn the_rejudging_wrapper_rejudges_every_confirmed_live_slot_count_and_checks_every_full_segment_was_handed_in(
    ) {
        let log: SharedRejudgeLog = Arc::new(Mutex::new(RejudgeLog::default()));
        let mut wrapper = GpuComputeRejudgedOnTheCpu {
            inner: CpuBackedStandIn {
                corrupted_crc32c: false,
            },
            log: Arc::clone(&log),
        };
        let unit_area = UnitAreaOfOneDevice {
            device: DeviceIdentity(0),
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 4 * LOCAL_CLUSTER_SEGMENT_SLOTS,
        };
        let segments: Vec<ClusterSegmentEntryKey> = (0..4_u64)
            .map(|segment_index| ClusterSegmentEntryKey {
                device: DeviceIdentity(0),
                start_slot: SlotNumber(
                    LOCAL_UNIT_AREA_START_SLOT + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS,
                ),
            })
            .collect();
        let _decisions = wrapper
            .count_live_slots_per_cluster_segment(&[], unit_area, &segments)
            .expect("替身不出错");
        let _fewer = wrapper
            .count_live_slots_per_cluster_segment(&[], unit_area, &segments[..3])
            .expect("替身不出错");
        let log = log.lock().expect("记录");
        assert_eq!(
            log.tally,
            RejudgeTally {
                entries_rejudged: 4 + 3,
                mismatches: 0
            }
        );
        assert_eq!(log.live_slot_calls_not_handing_in_every_full_segment, 1);
    }

    #[test]
    fn the_kernel_cpu_counter_line_gives_the_busy_ticks() {
        assert_eq!(
            busy_ticks_of_the_kernel_cpu_counter_line("cpu  10 20 30 1000 40 50 60 70 0 0"),
            Some(10 + 20 + 30 + 50 + 60 + 70)
        );
        assert_eq!(
            busy_ticks_of_the_kernel_cpu_counter_line("cpu0 1 2 3 4 5 6 7 8"),
            None
        );
        assert_eq!(busy_ticks_of_the_kernel_cpu_counter_line("cpu 1 2"), None);
    }

    #[test]
    fn the_process_status_line_gives_user_and_system_ticks_even_with_a_parenthesis_in_the_name() {
        let line =
            "4242 (e202 (gc) x) S 1 4242 4242 0 -1 4194304 100 0 0 0 1234 567 0 0 20 0 9 0 100 0 0";
        assert_eq!(ticks_of_the_process_status_line(line), Some(1234 + 567));
        assert_eq!(ticks_of_the_process_status_line("4242 (short) S 1"), None);
    }

    #[test]
    fn the_schedstat_line_gives_the_nanoseconds_on_the_cpu() {
        assert_eq!(
            cpu_of_the_schedstat_line("123456789 1000 7\n"),
            Some(Duration::from_nanos(123_456_789))
        );
        assert_eq!(cpu_of_the_schedstat_line(""), None);
    }

    #[test]
    fn this_process_lists_the_calling_thread_among_its_tasks() {
        let task = task_identifier_of_this_thread().expect("读得到 /proc/thread-self");
        let cpu_by_task = cpu_of_every_task_of_this_process().expect("读得到 /proc/self/task");
        assert!(cpu_by_task.contains_key(&task));
    }

    #[test]
    fn the_numbers_of_a_run_line_read_back_as_written() {
        let line = "E202 name=run tier=4GiB arm=gpu_one_card_rejudged repeat=3 session=2 batches=144 gc_wall_nanoseconds=900 \
                    process_cpu_nanoseconds=700 inner_rejudge_cpu_nanoseconds=50 void=no";
        let (name, fields) = fields_of_a_result_line(line).expect("是本装置的结果行");
        assert_eq!(name, "run");
        assert_eq!(
            RunNumbers::of_the_fields(&fields),
            Some(RunNumbers {
                arm: LOCKSTEP_ARMS[4],
                repeat: 3,
                batches: 144,
                gc_wall_nanoseconds: 900.0,
                process_cpu_nanoseconds: 700.0,
                inner_rejudge_cpu_nanoseconds: 50.0,
            })
        );
        assert_eq!(fields_of_a_result_line("E185 name=run"), None);
    }

    #[test]
    fn the_rotation_of_a_repeat_starts_at_the_arm_with_the_repeat_number() {
        let rotation: Vec<String> = arms_in_the_rotation_of_the_repeat(&LOCKSTEP_ARMS, 2)
            .iter()
            .map(|arm| arm.name())
            .collect();
        assert_eq!(
            rotation,
            [
                "gpu_every_card",
                "cpu_rejudged",
                "gpu_one_card_rejudged",
                "gpu_every_card_rejudged",
                "cpu",
                "gpu_one_card"
            ]
        );
    }

    fn data_unit_records_in_the_segment(segment_index: u64, units: u64) -> Vec<AllocationRecord> {
        (0..units)
            .map(|unit_index| AllocationRecord {
                device: DeviceIdentity(0),
                slot: SlotNumber(
                    LOCAL_UNIT_AREA_START_SLOT
                        + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS
                        + 2 * unit_index,
                ),
                span_slots: 2,
                generation: singlefs_core::address::CheckpointTxg(7),
                is_released: false,
                quarantine_mark:
                    singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
            })
            .collect()
    }

    /// 几何核两半（登记第五节「负载」第 4 条，后一半照第十二节修订第 8 条判强制选段后备点名的那一段）：碎片段九成以上恰好 32 槽活，
    /// 点名的那一段只装活数据单元、活槽不多于 32；greedy 那一段只报，它选中整段满的也不让几何判不成。
    #[test]
    fn the_geometry_is_formed_only_when_the_written_segments_are_half_live_and_the_forced_segment_is_a_half_live_data_segment(
    ) {
        let records: Vec<AllocationRecord> = [(0, 16), (1, 32), (3, 8)]
            .into_iter()
            .flat_map(|(segment_index, units)| {
                data_unit_records_in_the_segment(segment_index, units)
            })
            .collect();
        let unit_area = UnitAreaOfOneDevice {
            device: DeviceIdentity(0),
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 4 * LOCAL_CLUSTER_SEGMENT_SLOTS,
        };
        let data_slots: BTreeSet<u64> = records.iter().map(|record| record.slot.0).collect();
        let segment_start = |segment_index: u64| {
            LOCAL_UNIT_AREA_START_SLOT + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS
        };
        let half_formed = geometry_of_the_records(
            &records,
            unit_area,
            &[segment_start(0), segment_start(1)].into(),
            &data_slots,
            Some(segment_start(1)),
            Some(segment_start(3)),
        );
        assert_eq!(half_formed.written_segments, 2);
        assert_eq!(half_formed.written_segments_with_half_live, 1);
        assert_eq!(
            half_formed.forced,
            SegmentOfTheChoice {
                start_slot: Some(segment_start(3)),
                live_slots: 16,
                holds_only_data_units: true,
            }
        );
        assert_eq!(
            half_formed.greedy,
            SegmentOfTheChoice {
                start_slot: Some(segment_start(1)),
                live_slots: 64,
                holds_only_data_units: true,
            }
        );
        assert_eq!(
            half_formed.live_slot_histogram,
            BTreeMap::from([(0, 1), (16, 1), (32, 1), (64, 1)])
        );
        assert!(!half_formed.is_formed());
        // greedy 选中整段满的那一段，强制点名的是只装数据、16 槽活的那一段：照点名的那一段判，几何成。
        let formed = geometry_of_the_records(
            &records,
            unit_area,
            &[segment_start(0)].into(),
            &data_slots,
            Some(segment_start(1)),
            Some(segment_start(3)),
        );
        assert!(!formed.greedy.is_fragment_like());
        assert!(formed.is_formed());
        // 点名的那一段装着别的单元（不在活数据单元的槽里）：判不成。
        let forced_on_other_units = geometry_of_the_records(
            &records,
            unit_area,
            &[segment_start(0)].into(),
            &BTreeSet::new(),
            Some(segment_start(3)),
            Some(segment_start(3)),
        );
        assert!(!forced_on_other_units.forced.holds_only_data_units);
        assert!(!forced_on_other_units.is_formed());
        // 点名的那一段活槽超过 32：判不成。
        let forced_on_a_full_segment = geometry_of_the_records(
            &records,
            unit_area,
            &[segment_start(0)].into(),
            &data_slots,
            Some(segment_start(3)),
            Some(segment_start(1)),
        );
        assert_eq!(forced_on_a_full_segment.forced.live_slots, 64);
        assert!(!forced_on_a_full_segment.is_formed());
        let nothing_forced = geometry_of_the_records(
            &records,
            unit_area,
            &[segment_start(0)].into(),
            &data_slots,
            Some(segment_start(3)),
            None,
        );
        assert!(!nothing_forced.is_formed());
    }

    /// 强制选段后备的点名法（登记第十二节修订第 8 条）：碎片段里还没点过名的、只装活数据单元、有活槽而不多于 32 的段里起点最小的那一段；
    /// 用完交回 `None`。
    #[test]
    fn the_forced_fallback_names_the_lowest_fragment_segment_not_yet_forced_holding_only_data_units_and_at_most_half_live(
    ) {
        let records: Vec<AllocationRecord> = [(0, 16), (1, 32), (2, 16), (3, 8)]
            .into_iter()
            .flat_map(|(segment_index, units)| {
                data_unit_records_in_the_segment(segment_index, units)
            })
            .collect();
        let unit_area = UnitAreaOfOneDevice {
            device: DeviceIdentity(0),
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 5 * LOCAL_CLUSTER_SEGMENT_SLOTS,
        };
        let segment_start = |segment_index: u64| {
            LOCAL_UNIT_AREA_START_SLOT + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS
        };
        // 头一段里有一个单元不是活数据单元；第二段整段满；最后一段空着。
        let data_slots: BTreeSet<u64> = records
            .iter()
            .map(|record| record.slot.0)
            .filter(|slot| *slot != segment_start(0))
            .collect();
        let live_slots = live_slots_per_cluster_segment_by_the_records(&records, unit_area);
        let segments_holding_other_records =
            segments_holding_a_record_other_than_a_data_unit(&records, unit_area, &data_slots);
        assert_eq!(
            segments_holding_other_records,
            BTreeSet::from([segment_start(0)])
        );
        let fragment_segment_starts: BTreeSet<u64> = (0..5).map(segment_start).collect();
        let forced_after = |already_forced: &[u64]| {
            fragment_segment_to_force(
                &fragment_segment_starts,
                &already_forced.iter().copied().collect::<BTreeSet<u64>>(),
                LOCAL_UNIT_AREA_START_SLOT,
                &live_slots,
                &segments_holding_other_records,
            )
        };
        assert_eq!(
            forced_after(&[]),
            Some(SegmentOfTheChoice {
                start_slot: Some(segment_start(2)),
                live_slots: 32,
                holds_only_data_units: true,
            })
        );
        assert_eq!(
            forced_after(&[segment_start(2)]).and_then(|segment| segment.start_slot),
            Some(segment_start(3))
        );
        assert_eq!(forced_after(&[segment_start(2), segment_start(3)]), None);
    }

    /// 小池的活数据单元数：碎片段 64 段（P ÷ 32），一段的偏差压不过几何核的 90%。
    const SMALL_POOL_LIVE_DATA_UNITS: u64 = 2048;

    /// 两条小池用例共用的起步镜像：每块盘取默认环长下 mkfs 认的最小一块（3072 MiB，即最小档的盘），P = 2048。
    fn small_pool_image() -> StartingImage {
        construct_a_starting_image_of_the_size(
            UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
            smallest_device_bytes_the_make_filesystem_accepts(),
            SMALL_POOL_LIVE_DATA_UNITS,
        )
    }

    /// 整条路在一个小池上走一遍（每块盘取默认环长下 mkfs 认的最小一块 3072 MiB、P = 2048：内存盘懒分配，没写过的块不占内存；
    /// 真实三档的造池太久，进不了同包测试）：造起步镜像（16 单元的活数据文件与等长垫片文件交错接单元、删垫片）、几何成形（碎片段九成以上恰好 32 槽活、
    /// greedy 照核心层重放选中只装活数据单元的段），CPU 臂与 CPU 多判臂各交替跑一次，
    /// 都真搬了批、写的与读到的逐个相同、读到的等于期望，每一项记账照设置走 CPU、条数不少于批数，多判臂的里层计时大于 0、对不上的条数是 0。
    /// 一个场景：同一份起步镜像上两条臂按次序跑。
    #[test]
    fn the_small_pool_relocates_batches_and_the_cpu_arms_write_and_read_the_same() {
        let image = small_pool_image();
        // 几何核两半都断言：后一半判强制选段后备点名的那一段（登记第十二节修订第 8 条）；greedy 照核心层重放要选的那一段只打出来。
        eprintln!(
            "E202 小池几何：greedy_picks_a_fragment_like_segment={} {:?}",
            image.construction.geometry.greedy.is_fragment_like(),
            image.construction.geometry
        );
        assert!(
            image.construction.geometry.is_formed(),
            "{:?}",
            image.construction
        );
        assert!(
            image.construction.is_every_spacer_slot_released,
            "{:?}",
            image.construction
        );
        assert_eq!(
            image.construction.live_data_units_found,
            SMALL_POOL_LIVE_DATA_UNITS
        );
        // 造池量到的 inode 记录：造数据挂载之前那几条等于三档算术用的本地常量；峰值是它加活数据文件 P ÷ K（小池 2048 ÷ 16 = 128 个）
        // 加用到的垫片文件，片数等于核心层写同样条数进空树的片数。
        let construction = &image.construction;
        assert_eq!(
            construction.inode_records_before_the_data_mount, INODE_RECORDS_BEFORE_THE_DATA_MOUNT,
            "{construction:?}"
        );
        assert_eq!(construction.live_data_files, 128, "{construction:?}");
        assert_eq!(
            construction.spacer_files,
            construction.spacer_units.div_ceil(SPACER_UNITS_PER_FILE),
            "{construction:?}"
        );
        assert!(
            construction.spacer_files <= spacer_files_at_most(SMALL_POOL_LIVE_DATA_UNITS),
            "{construction:?}"
        );
        assert_eq!(
            construction.inode_records_at_the_peak,
            INODE_RECORDS_BEFORE_THE_DATA_MOUNT + 128 + construction.spacer_files,
            "{construction:?}"
        );
        assert_eq!(
            inode_leaf_containers_of_a_one_level_tree(construction.inode_records_at_the_peak),
            Ok(construction.inode_leaf_containers_at_the_peak),
            "{construction:?}"
        );
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let setting = LockstepRunSetting {
            image: &image,
            discrete_gpus: &[],
            memory_leaked: &memory_leaked,
            vulkan_instance: None,
            user_changes: LOCKSTEP_USER_CHANGES_LADDER[0],
            batch_units: BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
        };
        let plain = run_lockstep_once(&setting, LOCKSTEP_ARMS[0], 0).expect("CPU 臂跑得成");
        let rejudged = run_lockstep_once(&setting, LOCKSTEP_ARMS[3], 0).expect("CPU 多判臂跑得成");
        for run in [&plain, &rejudged] {
            assert!(run.batches > 0, "{} 搬了批", run.arm.name());
            assert!(run.content_matches_the_expectation);
            assert_eq!(
                run.path_anchor.compute_records_on_the_path_of_the_settings,
                run.path_anchor.compute_records
            );
            assert!(
                run.path_anchor.compute_records >= run.batches,
                "每一批至少一项记账：{:?}",
                run.path_anchor
            );
            assert!(
                run.most_compaction_publishes_between_two_user_changes
                    <= LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE
            );
            assert_eq!(
                run.calls_failing_the_inclusion_checks, 0,
                "{:?}",
                run.windows_failing_the_inclusion_checks
            );
            assert!(
                run.forced_fallback.intents_issued() > 0,
                "{:?}",
                run.forced_fallback
            );
            assert_eq!(
                run.forced_fallback.intents_issued_off_the_forced_region(),
                0,
                "{:?}",
                run.forced_fallback
            );
            assert!(
                run.forced_fallback
                    .choices
                    .iter()
                    .all(|choice| choice.forced.is_fragment_like()),
                "{:?}",
                run.forced_fallback
            );
        }
        let forced_region_starts = |run: &LockstepRunSummary| -> Vec<u64> {
            run.forced_fallback
                .choices
                .iter()
                .filter_map(|choice| choice.forced.start_slot)
                .collect()
        };
        assert_eq!(
            forced_region_starts(&plain),
            forced_region_starts(&rejudged)
        );
        assert_eq!(plain.recording_digest, rejudged.recording_digest);
        assert_eq!(plain.content_digest, rejudged.content_digest);
        assert_eq!(plain.batches, rejudged.batches);
        assert!(rejudged.inner_rejudge_cpu > Duration::ZERO);
        assert!(rejudged.rejudge.entries_rejudged > 0);
        assert_eq!(rejudged.rejudge.mismatches, 0);
        assert_eq!(plain.inner_rejudge_cpu, Duration::ZERO);
    }

    /// 包含自检的窗口两端（登记第五节「计时与计数怎么做」）：挂钟的窗口罩住 GC 线程 CPU 钟的窗口，一次什么都不做的调用挂钟也不短于 CPU 钟。
    /// 读序反过来（CPU 钟在外、挂钟在里）时，读 CPU 钟那两次系统调用的开销只落在 CPU 钟里，这样的调用几乎每一次都判不过。
    #[test]
    fn the_wall_of_a_call_window_encloses_the_gc_thread_cpu_of_an_empty_call() {
        const EMPTY_CALLS: usize = 200;
        let both_stand_ins_wait = std::sync::Barrier::new(3);
        let (to_the_test, task_identifiers) = mpsc::channel::<u64>();
        std::thread::scope(|scope| {
            for _stand_in in ["前台线程", "主线程"] {
                let to_the_test = to_the_test.clone();
                let both_stand_ins_wait = &both_stand_ins_wait;
                scope.spawn(move || {
                    to_the_test
                        .send(task_identifier_of_this_thread().expect("读得到线程号"))
                        .expect("测试线程在等");
                    both_stand_ins_wait.wait();
                });
            }
            drop(to_the_test);
            let clocks = ClocksOfTheGcThread {
                started: StartOnTheMonotonicRawClock::now(),
                gc_thread_task: task_identifier_of_this_thread().ok(),
                foreground_thread_task: task_identifiers.recv().expect("前台线程替身的线程号"),
                main_thread_task: task_identifiers.recv().expect("主线程替身的线程号"),
            };
            let windows: Vec<CallWindow> = (0..EMPTY_CALLS)
                .filter_map(|_call| {
                    measure_one_gc_call(
                        &clocks,
                        &mut (),
                        |_state: &mut ()| {},
                        |_state: &mut (), _outcome: &()| {
                            (Some(Duration::ZERO), RejudgeTally::default())
                        },
                    )
                    .window
                })
                .collect();
            both_stand_ins_wait.wait();
            assert!(windows.len() * 2 >= EMPTY_CALLS, "{}", windows.len());
            let wall_shorter_than_the_cpu: Vec<&CallWindow> = windows
                .iter()
                .filter(|window| window.wall < window.gc_thread_cpu)
                .collect();
            assert!(
                wall_shorter_than_the_cpu.is_empty(),
                "{wall_shorter_than_the_cpu:?}"
            );
        });
    }

    /// 包含自检「每次调用挂钟 ≥ GC 线程 CPU」在长调用上（登记第十二节修订第 10 条；任务 6790 挂钟读 `CLOCK_MONOTONIC` 时 22 窗里 21 窗
    /// GC 线程 CPU 比挂钟多 0.6–22 ppm）：一次调用自旋 100 ms 的线程 CPU，两端照 GC 调用的窗口读（挂钟在外、CPU 钟在里），每次挂钟都不短于 GC 线程 CPU。
    /// 两端的读数直接相减，不经 `call_window_between`：同一进程里别的用例的线程在窗口里退出时那一窗作废，这里比的只是两样钟。
    /// 挂钟读 `CLOCK_MONOTONIC` 时，本机的 NTP 正把钟调慢十几 ppm 的那一阵，一次调用少走一两微秒，比读钟开销留下的余量大，这一条判不过；
    /// NTP 没在调频时两种钟都过：单测改不了本机的调频，所以这一条分得出两种钟只在调频的那一阵，不是每次都分得出。
    #[test]
    fn the_wall_of_a_call_window_encloses_the_gc_thread_cpu_of_a_long_spinning_call() {
        const SPINNING_CALLS: usize = 20;
        const GC_THREAD_CPU_OF_ONE_CALL: Duration = Duration::from_millis(100);
        let both_stand_ins_wait = std::sync::Barrier::new(3);
        let (to_the_test, task_identifiers) = mpsc::channel::<u64>();
        std::thread::scope(|scope| {
            for _stand_in in ["前台线程", "主线程"] {
                let to_the_test = to_the_test.clone();
                let both_stand_ins_wait = &both_stand_ins_wait;
                scope.spawn(move || {
                    to_the_test
                        .send(task_identifier_of_this_thread().expect("读得到线程号"))
                        .expect("测试线程在等");
                    both_stand_ins_wait.wait();
                });
            }
            drop(to_the_test);
            let clocks = ClocksOfTheGcThread {
                started: StartOnTheMonotonicRawClock::now(),
                gc_thread_task: task_identifier_of_this_thread().ok(),
                foreground_thread_task: task_identifiers.recv().expect("前台线程替身的线程号"),
                main_thread_task: task_identifiers.recv().expect("主线程替身的线程号"),
            };
            let mut calls_with_the_wall_behind_the_gc_thread_cpu: Vec<(Duration, Duration)> =
                Vec::new();
            for _spinning_call in 0..SPINNING_CALLS {
                let opening = clocks
                    .read(EndOfTheWindow::Opening)
                    .expect("两个替身都活着，读得到窗口开端的钟");
                let gc_thread_cpu_at_the_start_of_the_spin = host_cpu_time_of_this_thread();
                while host_cpu_time_of_this_thread()
                    .saturating_sub(gc_thread_cpu_at_the_start_of_the_spin)
                    < GC_THREAD_CPU_OF_ONE_CALL
                {
                    std::hint::spin_loop();
                }
                let closing = clocks
                    .read(EndOfTheWindow::Closing)
                    .expect("两个替身都活着，读得到窗口末端的钟");
                let wall = closing
                    .wall
                    .checked_sub(opening.wall)
                    .expect("CLOCK_MONOTONIC_RAW 单调不减");
                let gc_thread_cpu = closing
                    .gc_thread_cpu
                    .checked_sub(opening.gc_thread_cpu)
                    .expect("线程 CPU 钟单调不减");
                if wall < gc_thread_cpu {
                    calls_with_the_wall_behind_the_gc_thread_cpu.push((wall, gc_thread_cpu));
                }
            }
            both_stand_ins_wait.wait();
            assert!(
                calls_with_the_wall_behind_the_gc_thread_cpu.is_empty(),
                "这几次调用（挂钟，GC 线程 CPU）里挂钟比 GC 线程 CPU 短：{calls_with_the_wall_behind_the_gc_thread_cpu:?}"
            );
        });
    }

    /// 挂钟的起点与走过的都读 `CLOCK_MONOTONIC_RAW`（登记第十二节修订第 10 条）：起点夹在它前后两次直接读的读数之间，
    /// 走过的夹在两对直接读数给的上下界之间。起点读的若是别的钟（线程 CPU 钟），起点落不进前一对读数之间；走过的若没减起点，大过上界。
    #[test]
    fn the_start_and_the_elapsed_of_the_wall_clock_are_read_on_the_monotonic_raw_clock() {
        let monotonic_raw_time_before_the_start = monotonic_raw_time_now();
        let start = StartOnTheMonotonicRawClock::now();
        let monotonic_raw_time_after_the_start = monotonic_raw_time_now();
        assert!(
            monotonic_raw_time_before_the_start <= start.monotonic_raw_time_at_the_start
                && start.monotonic_raw_time_at_the_start <= monotonic_raw_time_after_the_start,
            "起点 {start:?} 不在前后两次直接读数 {monotonic_raw_time_before_the_start:?}、{monotonic_raw_time_after_the_start:?} 之间"
        );
        std::thread::sleep(Duration::from_millis(5));
        let monotonic_raw_time_before_the_elapsed = monotonic_raw_time_now();
        let elapsed = start.elapsed();
        let monotonic_raw_time_after_the_elapsed = monotonic_raw_time_now();
        let elapsed_at_least = monotonic_raw_time_before_the_elapsed
            .checked_sub(monotonic_raw_time_after_the_start)
            .expect("CLOCK_MONOTONIC_RAW 单调不减");
        let elapsed_at_most = monotonic_raw_time_after_the_elapsed
            .checked_sub(monotonic_raw_time_before_the_start)
            .expect("CLOCK_MONOTONIC_RAW 单调不减");
        assert!(
            elapsed_at_least <= elapsed && elapsed <= elapsed_at_most,
            "走过的 {elapsed:?} 不在 {elapsed_at_least:?} 到 {elapsed_at_most:?} 之间"
        );
    }

    /// 包含自检用的几样钟走速差多少（这一条只量钟速、不判钟速；判的只是量的前提）。纯自旋的窗口里读四样：挂钟 `Instant`（`CLOCK_MONOTONIC`，
    /// 受 NTP 调频）、装置的挂钟 `StartOnTheMonotonicRawClock`（`CLOCK_MONOTONIC_RAW`，不受调频，登记第十二节修订第 10 条）、
    /// 这条线程的 CPU 钟（`CLOCK_THREAD_CPUTIME_ID`，走内核的调度钟，不受调频）、这条线程 schedstat 的第一个字段（同一份调度钟，
    /// 读 CPU 钟时内核刚把它记到那一刻，紧跟着读就是同一刻的值）。读序同 GC 调用的窗口：挂钟在外（`Instant` 最外、`CLOCK_MONOTONIC_RAW` 次之）、CPU 钟在里。
    /// 每窗打一行「E202 钟速」，末了打一行「E202 钟速汇总」：两两之差的最小、中位、最大，两样挂钟各自短于 CPU 钟的窗数（`--nocapture` 看）。
    /// 判的前提：每一窗都读得到；至少一半的窗里 CPU 钟走了挂钟的 0.9 以上（不在 CPU 上的窗太多，读数说明不了钟速）。
    /// 后一条不成立（机器忙、这条线程常被换下）时打一行「本次未跑」就返回，不判红、不打汇总（[`reading_of_the_spin_windows`]）。
    #[test]
    fn the_window_clocks_on_a_spin_loop_print_their_pairwise_differences() {
        const SPIN_WINDOWS: usize = 40;
        const SPIN_OF_ONE_WINDOW: Duration = Duration::from_millis(50);
        let schedstat_of_this_thread = || -> Duration {
            let line = std::fs::read_to_string("/proc/thread-self/schedstat")
                .expect("读得到这条线程的 schedstat");
            cpu_of_the_schedstat_line(&line).expect("schedstat 认得出")
        };
        let signed_nanoseconds =
            |duration: Duration| i128::try_from(duration.as_nanos()).expect("纳秒数装得进 i128");
        let mut wall_minus_thread_cpu: Vec<i128> = Vec::with_capacity(SPIN_WINDOWS);
        let mut wall_minus_thread_cpu_in_millionths: Vec<f64> = Vec::with_capacity(SPIN_WINDOWS);
        let mut thread_cpu_minus_schedstat: Vec<i128> = Vec::with_capacity(SPIN_WINDOWS);
        let mut monotonic_raw_minus_thread_cpu: Vec<i128> = Vec::with_capacity(SPIN_WINDOWS);
        let mut monotonic_raw_minus_thread_cpu_in_millionths: Vec<f64> =
            Vec::with_capacity(SPIN_WINDOWS);
        let mut windows_mostly_on_the_cpu = 0_usize;
        for window in 0..SPIN_WINDOWS {
            let wall_at_the_opening = Instant::now();
            let monotonic_raw_at_the_opening = StartOnTheMonotonicRawClock::now();
            let thread_cpu_at_the_opening = host_cpu_time_of_this_thread();
            let schedstat_at_the_opening = schedstat_of_this_thread();
            while wall_at_the_opening.elapsed() < SPIN_OF_ONE_WINDOW {
                std::hint::spin_loop();
            }
            let thread_cpu_at_the_closing = host_cpu_time_of_this_thread();
            let schedstat_at_the_closing = schedstat_of_this_thread();
            let monotonic_raw_wall = monotonic_raw_at_the_opening.elapsed();
            let wall = wall_at_the_opening.elapsed();
            let thread_cpu = thread_cpu_at_the_closing
                .checked_sub(thread_cpu_at_the_opening)
                .expect("线程 CPU 钟不往回走");
            let schedstat = schedstat_at_the_closing
                .checked_sub(schedstat_at_the_opening)
                .expect("schedstat 不往回走");
            let millionths =
                (wall.as_secs_f64() - thread_cpu.as_secs_f64()) / wall.as_secs_f64() * 1e6;
            let monotonic_raw_millionths = (monotonic_raw_wall.as_secs_f64()
                - thread_cpu.as_secs_f64())
                / monotonic_raw_wall.as_secs_f64()
                * 1e6;
            eprintln!(
                "E202 钟速 window={window} wall_nanoseconds={} monotonic_raw_nanoseconds={} thread_cpu_nanoseconds={} \
                 schedstat_nanoseconds={} wall_minus_thread_cpu_nanoseconds={} wall_minus_thread_cpu_ppm={millionths:.3} \
                 monotonic_raw_minus_thread_cpu_nanoseconds={} monotonic_raw_minus_thread_cpu_ppm={monotonic_raw_millionths:.3}",
                wall.as_nanos(),
                monotonic_raw_wall.as_nanos(),
                thread_cpu.as_nanos(),
                schedstat.as_nanos(),
                signed_nanoseconds(wall) - signed_nanoseconds(thread_cpu),
                signed_nanoseconds(monotonic_raw_wall) - signed_nanoseconds(thread_cpu),
            );
            if thread_cpu.as_secs_f64() >= 0.9 * wall.as_secs_f64() {
                windows_mostly_on_the_cpu += 1;
            }
            wall_minus_thread_cpu.push(signed_nanoseconds(wall) - signed_nanoseconds(thread_cpu));
            wall_minus_thread_cpu_in_millionths.push(millionths);
            thread_cpu_minus_schedstat
                .push(signed_nanoseconds(thread_cpu) - signed_nanoseconds(schedstat));
            monotonic_raw_minus_thread_cpu
                .push(signed_nanoseconds(monotonic_raw_wall) - signed_nanoseconds(thread_cpu));
            monotonic_raw_minus_thread_cpu_in_millionths.push(monotonic_raw_millionths);
        }
        match reading_of_the_spin_windows(windows_mostly_on_the_cpu, SPIN_WINDOWS) {
            ReadingOfTheSpinWindows::NotRun => {
                eprintln!(
                    "E202 钟速 本次未跑：{SPIN_WINDOWS} 窗里只有 {windows_mostly_on_the_cpu} 窗 CPU 钟走了挂钟的 0.9 以上，\
                     不到一半，这条线程常被换下，读数说明不了钟速"
                );
                return;
            }
            ReadingOfTheSpinWindows::Measured => {}
        }
        wall_minus_thread_cpu.sort_unstable();
        wall_minus_thread_cpu_in_millionths.sort_by(f64::total_cmp);
        thread_cpu_minus_schedstat.sort_unstable();
        monotonic_raw_minus_thread_cpu.sort_unstable();
        monotonic_raw_minus_thread_cpu_in_millionths.sort_by(f64::total_cmp);
        let middle = SPIN_WINDOWS / 2;
        eprintln!(
            "E202 钟速汇总 windows={SPIN_WINDOWS} windows_mostly_on_the_cpu={windows_mostly_on_the_cpu} \
             windows_wall_below_thread_cpu={} wall_minus_thread_cpu_nanoseconds_minimum={} median={} maximum={} \
             wall_minus_thread_cpu_ppm_minimum={:.3} median={:.3} maximum={:.3} \
             thread_cpu_minus_schedstat_nanoseconds_minimum={} median={} maximum={} \
             windows_monotonic_raw_below_thread_cpu={} monotonic_raw_minus_thread_cpu_nanoseconds_minimum={} median={} maximum={} \
             monotonic_raw_minus_thread_cpu_ppm_minimum={:.3} median={:.3} maximum={:.3}",
            wall_minus_thread_cpu
                .iter()
                .filter(|difference| **difference < 0)
                .count(),
            wall_minus_thread_cpu[0],
            wall_minus_thread_cpu[middle],
            wall_minus_thread_cpu[SPIN_WINDOWS - 1],
            wall_minus_thread_cpu_in_millionths[0],
            wall_minus_thread_cpu_in_millionths[middle],
            wall_minus_thread_cpu_in_millionths[SPIN_WINDOWS - 1],
            thread_cpu_minus_schedstat[0],
            thread_cpu_minus_schedstat[middle],
            thread_cpu_minus_schedstat[SPIN_WINDOWS - 1],
            monotonic_raw_minus_thread_cpu
                .iter()
                .filter(|difference| **difference < 0)
                .count(),
            monotonic_raw_minus_thread_cpu[0],
            monotonic_raw_minus_thread_cpu[middle],
            monotonic_raw_minus_thread_cpu[SPIN_WINDOWS - 1],
            monotonic_raw_minus_thread_cpu_in_millionths[0],
            monotonic_raw_minus_thread_cpu_in_millionths[middle],
            monotonic_raw_minus_thread_cpu_in_millionths[SPIN_WINDOWS - 1],
        );
    }

    /// 钟速单测这一次读得出钟速没有。封闭集合。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ReadingOfTheSpinWindows {
        /// 至少一半的窗里 CPU 钟走了挂钟的 0.9 以上：照打汇总。
        Measured,
        /// 不到一半：本次未跑，不判红。
        NotRun,
    }

    /// `windows` 窗里有 `windows_mostly_on_the_cpu` 窗 CPU 钟走了挂钟的 0.9 以上，钟速单测这一次读得出钟速没有。
    fn reading_of_the_spin_windows(
        windows_mostly_on_the_cpu: usize,
        windows: usize,
    ) -> ReadingOfTheSpinWindows {
        if windows_mostly_on_the_cpu * 2 >= windows {
            ReadingOfTheSpinWindows::Measured
        } else {
            ReadingOfTheSpinWindows::NotRun
        }
    }

    /// 钟速单测的前提不成立（机器忙）是本次未跑，不是红：40 窗里 19 窗在 CPU 上交「本次未跑」，20 窗交「量了」。
    #[test]
    fn spin_windows_mostly_off_the_cpu_are_not_run_rather_than_failed() {
        assert_eq!(
            reading_of_the_spin_windows(19, 40),
            ReadingOfTheSpinWindows::NotRun
        );
        assert_eq!(
            reading_of_the_spin_windows(20, 40),
            ReadingOfTheSpinWindows::Measured
        );
        assert_eq!(
            reading_of_the_spin_windows(0, 40),
            ReadingOfTheSpinWindows::NotRun
        );
    }

    /// mkfs 那一条（环超过设备容量的四分之一就拒）拿真 mkfs 回比：差一个扇区不到四倍环长的盘拒，正好四倍的认。
    #[test]
    fn the_make_filesystem_refuses_a_device_one_sector_short_of_four_rings() {
        let parameters = make_filesystem_parameters_of_the_pool();
        let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::not_fed()));
        let smallest = smallest_device_bytes_the_make_filesystem_accepts();
        assert_eq!(smallest, 3072 << 20);
        let mut one_sector_short = empty_pool_devices(smallest - SECTOR_BYTES, &recording);
        assert!(matches!(
            make_filesystem(&parameters, &mut one_sector_short),
            Err(
                singlefs_core::make_filesystem::MakeFilesystemError::JournalRingTooLargeForDevice { .. }
            )
        ));
        let mut just_four_rings = empty_pool_devices(smallest, &recording);
        assert!(make_filesystem(&parameters, &mut just_four_rings).is_ok());
        assert!(the_make_filesystem_accepts_the_device_bytes(smallest));
        assert!(!the_make_filesystem_accepts_the_device_bytes(
            smallest - SECTOR_BYTES
        ));
    }

    /// 登记第十二节修订第 4 条的三档（每块盘 3072、4880、8976 MiB）在默认环长下 mkfs 全认：档预检判认，真 mkfs 在两块空内存盘上也做成。
    #[test]
    fn the_registered_device_bytes_of_every_tier_are_accepted_by_the_make_filesystem() {
        let parameters = make_filesystem_parameters_of_the_pool();
        let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::not_fed()));
        let accepted: Vec<(u64, bool, bool)> = UnitAreaTier::EVERY_TIER
            .into_iter()
            .map(|tier| {
                let mut devices = empty_pool_devices(tier.device_bytes(), &recording);
                (
                    tier.device_bytes() >> 20,
                    the_make_filesystem_accepts_the_device_bytes(tier.device_bytes()),
                    make_filesystem(&parameters, &mut devices).is_ok(),
                )
            })
            .collect();
        assert_eq!(
            accepted,
            [(3072, true, true), (4880, true, true), (8976, true, true)]
        );
    }

    /// 前提 N：批数都相等且不少于 64 才够。
    #[test]
    fn batches_equal_and_at_least_sixty_four_are_enough_and_nothing_else_is() {
        assert!(batches_are_equal_and_at_least_the_minimum(&[64, 64, 64]));
        assert!(batches_are_equal_and_at_least_the_minimum(&[144]));
        assert!(!batches_are_equal_and_at_least_the_minimum(&[63, 63]));
        assert!(!batches_are_equal_and_at_least_the_minimum(&[64, 65]));
        assert!(!batches_are_equal_and_at_least_the_minimum(&[]));
    }

    fn checks_where_everything_holds() -> ChecksOfARepeat {
        ChecksOfARepeat {
            busiest_host_other_cores: 1.0,
            is_identical: true,
            has_enough_batches: true,
            class_one_anchors_hold: true,
            class_two_anchors_hold: true,
        }
    }

    /// 停机条款第 4 条与作废第 3 条：甲类不成立停机；机器忙（> 16 核）整组作废、排在别的核之前；都成立才计数。
    #[test]
    fn the_repeat_checks_void_a_busy_host_first_and_stop_on_a_broken_class_one_anchor() {
        assert_eq!(
            verdict_of_the_repeat_checks(checks_where_everything_holds()),
            VerdictOfTheRepeatChecks::Counted
        );
        let broken_class_one = ChecksOfARepeat {
            class_one_anchors_hold: false,
            ..checks_where_everything_holds()
        };
        assert_eq!(
            verdict_of_the_repeat_checks(broken_class_one),
            VerdictOfTheRepeatChecks::Stopped("a_class_one_anchor_does_not_hold")
        );
        let busy_and_broken = ChecksOfARepeat {
            busiest_host_other_cores: 16.5,
            ..broken_class_one
        };
        assert_eq!(
            verdict_of_the_repeat_checks(busy_and_broken),
            VerdictOfTheRepeatChecks::Voided("the_host_was_busy")
        );
        let sixteen_cores = ChecksOfARepeat {
            busiest_host_other_cores: 16.0,
            ..checks_where_everything_holds()
        };
        assert_eq!(
            verdict_of_the_repeat_checks(sixteen_cores),
            VerdictOfTheRepeatChecks::Counted
        );
        let not_identical = ChecksOfARepeat {
            is_identical: false,
            ..checks_where_everything_holds()
        };
        assert_eq!(
            verdict_of_the_repeat_checks(not_identical),
            VerdictOfTheRepeatChecks::Stopped(
                "the_arms_of_the_repeat_did_not_write_and_read_the_same"
            )
        );
        let broken_class_two = ChecksOfARepeat {
            class_two_anchors_hold: false,
            ..checks_where_everything_holds()
        };
        assert_eq!(
            verdict_of_the_repeat_checks(broken_class_two),
            VerdictOfTheRepeatChecks::Stopped("a_class_two_anchor_does_not_hold_the_tier_is_void")
        );
    }

    /// 甲类第 1–4 行合判：相邻两次用户改动之间 21 次整理发布、某一项记账走错、多判的批判定加不齐，各判不成立。
    #[test]
    fn the_class_one_anchors_of_a_lockstep_run_break_past_twenty_publishes_between_two_user_changes(
    ) {
        let holding_path = PathAnchor {
            compute_records: 5,
            compute_records_on_the_path_of_the_settings: 5,
            batches: 2,
        };
        let anchors = GpuWorkAnchors::default();
        assert!(class_one_anchors_of_a_lockstep_run_hold(
            holding_path,
            ComputeOfTheArm::Cpu,
            anchors,
            0,
            20
        ));
        assert!(!class_one_anchors_of_a_lockstep_run_hold(
            holding_path,
            ComputeOfTheArm::Cpu,
            anchors,
            0,
            21
        ));
        assert!(!class_one_anchors_of_a_lockstep_run_hold(
            PathAnchor {
                compute_records_on_the_path_of_the_settings: 4,
                ..holding_path
            },
            ComputeOfTheArm::Cpu,
            anchors,
            0,
            20
        ));
        assert!(!class_one_anchors_of_a_lockstep_run_hold(
            holding_path,
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
            anchors,
            1,
            20
        ));
    }

    fn gpu_record_with_unit_checksum_entries(
        confirmed_by_both_kernels: u64,
        uncertain_this_round: u64,
        judged_by_the_cpu_reference: u64,
    ) -> BackgroundComputeOfOneBatch {
        let work = singlefs_core::background_compute::GpuWorkOfTheBatch {
            unit_checksum_verification: singlefs_core::background_compute::GpuEntryCounts {
                confirmed_by_both_kernels,
                uncertain_this_round,
                judged_by_the_cpu_reference,
            },
            region_record_classification:
                singlefs_core::background_compute::GpuEntryCounts::default(),
            live_slots_per_cluster_segment:
                singlefs_core::background_compute::GpuEntryCounts::default(),
            units_whose_checksum_mismatched_on_the_gpu: 0,
            submissions_on_the_gpus: GpuSubmissionsOfTheBatch::default(),
        };
        BackgroundComputeOfOneBatch {
            task: singlefs_core::background_compute::BackgroundTask::CompactionRelocation,
            requested_backend: BackgroundComputeBackend::Gpu,
            path_taken: BackgroundComputePathTaken::Gpu(work),
            compute_error_that_abandoned_the_batch: None,
        }
    }

    /// 甲类第 3 行：确认 + 不确定 + 交 CPU 判 = 交进去的条目数；少一条、或包装记下的批数与记账里走 GPU 的批数不等，各记一批不成立。
    #[test]
    fn the_decisions_of_a_rejudged_batch_must_add_up_to_the_entries_handed_in() {
        let records = [call_record_with(
            1,
            vec![gpu_record_with_unit_checksum_entries(2, 1, 1)],
        )];
        let four_checksums = EntriesHandedIn {
            unit_checksums: 4,
            region_records: 0,
            cluster_segments: 0,
        };
        assert_eq!(
            batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(
                &records,
                &[four_checksums]
            ),
            0
        );
        let five_checksums = EntriesHandedIn {
            unit_checksums: 5,
            ..four_checksums
        };
        assert_eq!(
            batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(
                &records,
                &[five_checksums]
            ),
            1
        );
        assert_eq!(
            batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(
                &records,
                &[four_checksums, four_checksums]
            ),
            1
        );
    }

    /// 乙类第 5 行：CPU 臂 GC 线程 CPU ÷ GC 挂钟 ≥ 0.9。
    #[test]
    fn the_cpu_arm_thread_cpu_covers_the_wall_at_ninety_percent_and_not_below() {
        assert!(cpu_arm_thread_cpu_covers_the_wall(
            Duration::from_millis(900),
            Duration::from_millis(1000)
        ));
        assert!(!cpu_arm_thread_cpu_covers_the_wall(
            Duration::from_millis(899),
            Duration::from_millis(1000)
        ));
    }

    /// 乙类第 6 行与作废第 2 条：挂钟 < GC 线程 CPU、进程口径 < GC 线程口径的调用各记一次。
    #[test]
    fn the_inclusion_checks_count_a_call_whose_thread_cpu_exceeds_its_wall_or_its_process_cpu() {
        let window =
            |wall_milliseconds: u64, process_milliseconds: u64, gc_thread_milliseconds: u64| {
                CallWindow {
                    wall: Duration::from_millis(wall_milliseconds),
                    process_cpu: Duration::from_millis(process_milliseconds),
                    gc_thread_cpu: Duration::from_millis(gc_thread_milliseconds),
                    foreground_thread_cpu: Duration::ZERO,
                    main_thread_cpu: Duration::ZERO,
                }
            };
        assert_eq!(calls_failing_the_inclusion_checks(&[window(10, 9, 8)]), 0);
        assert_eq!(
            calls_failing_the_inclusion_checks(&[
                window(10, 9, 8),
                window(7, 9, 8),
                window(10, 7, 8)
            ]),
            2
        );
    }

    /// 作废第 4 条：出错放弃的一批记进锚点；作废的重复第 3 次（出错放弃的也算一次）这一档停下。
    #[test]
    fn the_gpu_work_anchors_count_a_batch_abandoned_after_a_compute_error_and_the_third_void_stops_the_tier(
    ) {
        let mut abandoned = gpu_record_with_unit_checksum_entries(1, 0, 0);
        abandoned.compute_error_that_abandoned_the_batch = Some(
            singlefs_core::background_compute::BackgroundComputeError::InjectedByTheTestOnlyHook,
        );
        let lowest = gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0");
        let anchors = gpu_work_anchors(&[call_record_with(1, vec![abandoned])], &lowest);
        assert_eq!(anchors.batches_abandoned_after_a_compute_error, 1);
        assert!(!the_tier_stops_after_the_voided_repeats(2, 0));
        assert!(the_tier_stops_after_the_voided_repeats(3, 0));
        assert!(the_tier_stops_after_the_voided_repeats(3, 3));
    }

    /// 作废第 7 条：内核把线程名截到 15 个字节，`comm` 一行带换行。
    #[test]
    fn the_comm_line_of_the_gpu_opening_thread_matches_on_its_first_fifteen_bytes() {
        assert!(is_the_comm_line_of_the_gpu_opening_thread(
            "singlefs-openin\n"
        ));
        assert!(!is_the_comm_line_of_the_gpu_opening_thread(
            "singlefs-opening-a-gpu\n"
        ));
        assert!(!is_the_comm_line_of_the_gpu_opening_thread(
            "e202_gpu_cpu_gc\n"
        ));
    }

    /// 停机条款第 2 条：产物里早先那几条命令的 crates 摘要有一份与这一次的不同就停。
    #[test]
    fn the_crates_change_is_seen_when_an_earlier_command_ran_on_another_digest() {
        let version_line = |digest: &str| {
            (
                "crates_version".to_string(),
                BTreeMap::from([("combined_sha256".to_string(), digest.to_string())]),
            )
        };
        assert!(!the_crates_changed_since_the_earlier_commands(&[], "aaaa"));
        assert!(!the_crates_changed_since_the_earlier_commands(
            &[version_line("aaaa"), version_line("aaaa")],
            "aaaa"
        ));
        assert!(the_crates_changed_since_the_earlier_commands(
            &[version_line("aaaa"), version_line("bbbb")],
            "aaaa"
        ));
    }

    /// 第四节第 3a 步第 2 项：1 ms 里前进 100 次才够细。
    #[test]
    fn the_thread_cpu_clock_check_needs_a_hundred_advances_in_a_millisecond() {
        assert!(the_thread_cpu_clock_is_fine_enough(100));
        assert!(!the_thread_cpu_clock_is_fine_enough(99));
    }

    fn timings_in_milliseconds(
        planned: u64,
        flag_set: u64,
        lock_acquired: u64,
        publish_returned: u64,
    ) -> UserChangeTimings {
        UserChangeTimings {
            planned: Duration::from_millis(planned),
            flag_set: Duration::from_millis(flag_set),
            lock_acquired: Duration::from_millis(lock_acquired),
            publish_returned: Duration::from_millis(publish_returned),
        }
    }

    /// 第九节变异「延迟从拿到锁算起」在这里红：前台落后计划 20 毫秒、等锁 10 毫秒、锁里发布 10 毫秒，延迟是 40 毫秒不是 10 毫秒。
    #[test]
    fn the_latency_counts_from_the_planned_time_and_the_publish_under_the_lock_from_the_lock() {
        let behind_the_plan = timings_in_milliseconds(100, 120, 130, 140);
        assert_eq!(
            latency_of_the_user_change(behind_the_plan),
            Some(Duration::from_millis(40))
        );
        assert_eq!(
            publish_under_the_lock_of_the_user_change(behind_the_plan),
            Some(Duration::from_millis(10))
        );
        assert_eq!(
            wait_from_the_flag_to_the_lock(behind_the_plan),
            Some(Duration::from_millis(10))
        );
        assert_eq!(
            user_changes_failing_the_inclusion_check(&[behind_the_plan]),
            0
        );
        let lock_before_the_flag = timings_in_milliseconds(100, 120, 110, 140);
        assert_eq!(
            user_changes_failing_the_inclusion_check(&[behind_the_plan, lock_before_the_flag]),
            1
        );
    }

    /// 第 99 百分位取最近秩：1..=100 取 99，1280 个数取第 1268 个，一个数取它自己。
    #[test]
    fn the_ninety_ninth_percentile_takes_the_nearest_rank() {
        let hundred: Vec<f64> = (1..=100).map(f64::from).collect();
        let distribution = latency_distribution_of(&hundred).expect("有数");
        assert_eq!(distribution.count, 100);
        assert_eq!(distribution.high_percentile, 99.0);
        assert_eq!(distribution.median, 50.5);
        assert_eq!(distribution.maximum, 100.0);
        let pooled: Vec<f64> = (1..=1280).rev().map(f64::from).collect();
        assert_eq!(
            latency_distribution_of(&pooled)
                .expect("有数")
                .high_percentile,
            1268.0
        );
        assert_eq!(
            latency_distribution_of(&[7.0])
                .expect("有数")
                .high_percentile,
            7.0
        );
        assert_eq!(latency_distribution_of(&[]), None);
    }

    fn concurrent_call_with(
        stop_name: &str,
        started_at_milliseconds: u64,
        queries: &[(u64, ForegroundFsync)],
        returned_at_milliseconds: u64,
    ) -> ConcurrentCallRecord {
        let mut call = call_record_with(1, Vec::new());
        call.stop_name = stop_name.to_string();
        ConcurrentCallRecord {
            call,
            started_at: Duration::from_millis(started_at_milliseconds),
            returned_at: Duration::from_millis(returned_at_milliseconds),
            fsync_queries: queries
                .iter()
                .map(|(at, answer)| (Duration::from_millis(*at), *answer))
                .collect(),
        }
    }

    /// 甲类第 5 行：答了「在等」就得是这次调用问的最后一次、只答一次、停在让路上；没答过「在等」的调用不核。
    #[test]
    fn the_yield_anchor_holds_only_when_the_waiting_answer_ends_the_call_with_the_yield_stop() {
        let yield_stop = CompactionRunStop::YieldedToAForegroundFsync.name();
        let budget_stop = "compaction_stopped_publish_budget_spent";
        let yielded = concurrent_call_with(
            yield_stop,
            0,
            &[
                (1, ForegroundFsync::NoneWaiting),
                (2, ForegroundFsync::Waiting),
            ],
            3,
        );
        let never_asked_to_wait =
            concurrent_call_with(budget_stop, 0, &[(1, ForegroundFsync::NoneWaiting)], 2);
        assert_eq!(
            calls_not_yielding_after_answering_waiting(&[yielded, never_asked_to_wait]),
            0
        );
        let went_on_after_waiting = concurrent_call_with(
            budget_stop,
            0,
            &[
                (1, ForegroundFsync::Waiting),
                (2, ForegroundFsync::NoneWaiting),
            ],
            3,
        );
        let waiting_but_another_stop =
            concurrent_call_with(budget_stop, 0, &[(1, ForegroundFsync::Waiting)], 2);
        let waiting_twice = concurrent_call_with(
            yield_stop,
            0,
            &[(1, ForegroundFsync::Waiting), (2, ForegroundFsync::Waiting)],
            3,
        );
        let trajectory = waiting_answers_trajectory(&[
            went_on_after_waiting,
            waiting_but_another_stop,
            waiting_twice,
        ]);
        assert_eq!(
            trajectory,
            WaitingAnswersTrajectory {
                peak_in_one_call: 2,
                calls_answered_waiting: 3,
                in_the_last_call: 2,
            }
        );
        for broken in [
            concurrent_call_with(
                yield_stop,
                0,
                &[
                    (1, ForegroundFsync::Waiting),
                    (2, ForegroundFsync::NoneWaiting),
                ],
                3,
            ),
            concurrent_call_with(budget_stop, 0, &[(1, ForegroundFsync::Waiting)], 2),
            concurrent_call_with(
                yield_stop,
                0,
                &[(1, ForegroundFsync::Waiting), (2, ForegroundFsync::Waiting)],
                3,
            ),
        ] {
            assert_eq!(calls_not_yielding_after_answering_waiting(&[broken]), 1);
        }
    }

    /// 第十条第 4 行：一批的上界取相邻两次问前台之间（连同调用开始到头一次问）最长的一段，收尾取最后一次问到返回；前台等锁超过两者之和的两倍才算。
    #[test]
    fn the_longest_batch_and_tail_come_from_the_gaps_between_the_foreground_queries() {
        let calls = [
            concurrent_call_with(
                "compaction_stopped_publish_budget_spent",
                0,
                &[
                    (1, ForegroundFsync::NoneWaiting),
                    (4, ForegroundFsync::NoneWaiting),
                    (10, ForegroundFsync::NoneWaiting),
                ],
                15,
            ),
            concurrent_call_with("not_started_NothingToDo", 20, &[], 22),
        ];
        let (longest_batch, longest_tail) = longest_batch_and_longest_tail_of_the_calls(&calls);
        assert_eq!(longest_batch, Duration::from_millis(6));
        assert_eq!(longest_tail, Duration::from_millis(5));
        assert!(!the_wait_exceeds_twice_the_longest_batch_and_tail(
            Duration::from_millis(22),
            longest_batch,
            longest_tail
        ));
        assert!(the_wait_exceeds_twice_the_longest_batch_and_tail(
            Duration::from_millis(23),
            longest_batch,
            longest_tail
        ));
    }

    /// 并发跑的甲类合判：有一次答了「在等」而没让路、前台等锁过了界、某一项记账走错，各判不成立。
    #[test]
    fn the_class_one_anchors_of_a_concurrent_run_break_on_a_call_not_yielding_or_a_wait_past_the_bound(
    ) {
        let holding_path = PathAnchor {
            compute_records: 3,
            compute_records_on_the_path_of_the_settings: 3,
            batches: 1,
        };
        let anchors = GpuWorkAnchors::default();
        assert!(class_one_anchors_of_a_concurrent_run_hold(
            holding_path,
            ComputeOfTheArm::Cpu,
            anchors,
            0,
            false
        ));
        assert!(!class_one_anchors_of_a_concurrent_run_hold(
            holding_path,
            ComputeOfTheArm::Cpu,
            anchors,
            1,
            false
        ));
        assert!(!class_one_anchors_of_a_concurrent_run_hold(
            holding_path,
            ComputeOfTheArm::Cpu,
            anchors,
            0,
            true
        ));
        assert!(!class_one_anchors_of_a_concurrent_run_hold(
            PathAnchor {
                compute_records: 0,
                compute_records_on_the_path_of_the_settings: 0,
                batches: 1,
            },
            ComputeOfTheArm::Cpu,
            anchors,
            0,
            false
        ));
    }

    fn concurrent_numbers(
        arm: ConcurrentArm,
        repeat: u32,
        latencies: &[f64],
    ) -> ConcurrentRunNumbers {
        ConcurrentRunNumbers {
            arm,
            repeat,
            latencies: latencies.to_vec(),
            publishes_under_the_lock: latencies.iter().map(|latency| latency / 2.0).collect(),
        }
    }

    /// 量 F：每条臂几次跑合起来一份分布、每次跑各自的中位与第 99 百分位；GPU 点名一张卡臂减 CPU 臂按重复成对（钉绝对值）。
    #[test]
    fn the_concurrent_tier_judgement_pools_the_runs_and_pairs_the_gpu_arm_against_the_cpu_arm() {
        let numbers = [
            concurrent_numbers(ConcurrentArm::Cpu, 1, &[10.0, 30.0]),
            concurrent_numbers(ConcurrentArm::Cpu, 0, &[20.0, 40.0]),
            concurrent_numbers(
                ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
                0,
                &[25.0, 45.0],
            ),
            concurrent_numbers(
                ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
                1,
                &[5.0, 50.0],
            ),
        ];
        let lines = judgement_of_a_concurrent_tier(
            UnitAreaTier::FourGibibytes,
            Duration::from_millis(50),
            &numbers,
            &[
                ConcurrentArm::Cpu,
                ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
            ],
        );
        let names: Vec<&str> = lines.iter().map(|(name, _fields)| name.as_str()).collect();
        assert_eq!(
            names,
            [
                "latency_distribution",
                "latency_distribution",
                "latency_comparison",
                "latency_comparison"
            ]
        );
        assert!(lines[0].1.contains("arm=cpu "));
        assert!(lines[0].1.contains(
            "latency_nanoseconds_count=4 latency_nanoseconds_median=25 latency_nanoseconds_ninety_ninth_percentile=40 latency_nanoseconds_maximum=40"
        ));
        assert!(lines[0]
            .1
            .contains("publish_under_the_lock_nanoseconds_maximum=20"));
        assert!(lines[0]
            .1
            .contains("median_of_each_run=30,20 ninety_ninth_percentile_of_each_run=40,30"));
        assert!(lines[2].1.contains(
            "statistic=ninety_ninth_percentile gpu_one_card_minus_cpu_nanoseconds=5,20 sign=positive"
        ));
        assert!(lines[3]
            .1
            .contains("statistic=maximum gpu_one_card_minus_cpu_nanoseconds=5,20 sign=positive"));
    }

    /// 续跑：`name=concurrent_run` 行里的两串纳秒数读回来与写的相同，空串读成一个都没有。
    #[test]
    fn the_numbers_of_a_concurrent_run_line_read_back_as_written() {
        let (name, fields) = fields_of_a_result_line(
            "E202 name=concurrent_run arm=gpu_one_card repeat=3 latencies_nanoseconds=12,7 publishes_under_the_lock_nanoseconds= void=no",
        )
        .expect("是本装置的结果行");
        assert_eq!(name, "concurrent_run");
        assert_eq!(
            ConcurrentRunNumbers::of_the_fields(&fields),
            Some(ConcurrentRunNumbers {
                arm: ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
                repeat: 3,
                latencies: vec![12.0, 7.0],
                publishes_under_the_lock: Vec::new(),
            })
        );
    }

    fn arguments_of(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| (*word).to_string()).collect()
    }

    /// 登记第八节两点：中档、批大小 16 的交替跑（CPU 臂与 GPU 点名一张卡臂），前台间隔 500 毫秒的并发跑（三条臂）；命令行只认这两个取值。
    #[test]
    fn the_sensitivity_points_are_the_middle_tier_with_sixteen_unit_batches_and_five_hundred_millisecond_intervals(
    ) {
        let product = Path::new("sensitivity.out");
        let lockstep = sensitivity_lockstep_command(product, 24);
        assert_eq!(lockstep.tier, UnitAreaTier::FourGibibytes);
        assert_eq!(lockstep.batch_units, 16);
        assert_eq!(lockstep.user_changes, 24);
        assert_eq!(
            lockstep
                .arms_requested
                .iter()
                .map(|arm| arm.name())
                .collect::<Vec<String>>(),
            ["cpu", "gpu_one_card"]
        );
        assert_eq!(
            sensitivity_concurrent_command(product),
            ConcurrentCommand {
                tier: UnitAreaTier::FourGibibytes,
                foreground_interval: Duration::from_millis(500),
                arms_requested: vec![
                    ConcurrentArm::Cpu,
                    ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
                    ConcurrentArm::WithoutCompaction
                ],
                product: PathBuf::from("sensitivity.out"),
            }
        );
        assert!(matches!(
            command_of_the_arguments(&arguments_of(&[
                "sensitivity",
                "--batch-units",
                "16",
                "--foreground-interval-milliseconds",
                "500",
                "--product",
                "sensitivity.out"
            ])),
            Ok(CommandOfTheApparatus::Sensitivity {
                user_changes: 24,
                ..
            })
        ));
        assert!(command_of_the_arguments(&arguments_of(&[
            "sensitivity",
            "--batch-units",
            "4",
            "--product",
            "sensitivity.out"
        ]))
        .is_err());
        match command_of_the_arguments(&arguments_of(&[
            "concurrent",
            "--reading",
            "occupancy",
            "--unit-area",
            "4GiB",
            "--product",
            "concurrent.out",
        ])) {
            Ok(CommandOfTheApparatus::Concurrent(command)) => {
                assert_eq!(command.foreground_interval, Duration::from_millis(50));
                assert_eq!(command.arms_requested, CONCURRENT_ARMS.to_vec());
            }
            Ok(_) | Err(_) => panic!("concurrent 认得出、间隔默认 50 毫秒"),
        }
        assert!(command_of_the_arguments(&arguments_of(&[
            "concurrent",
            "--reading",
            "occupancy",
            "--unit-area",
            "4GiB",
            "--foreground-interval-milliseconds",
            "70",
            "--product",
            "concurrent.out",
        ]))
        .is_err());
    }

    /// 并发跑整条路在小池上走一遍（同交替跑那条小池用例的池；8 次用户改动、间隔 5 毫秒，真实的 256 次与 50 毫秒进不了同包测试）：
    /// CPU 臂与不跑 GC 臂各一次，读到的等于前台写的，每次改动的延迟不短于锁里发布用的时间，答了「在等」的调用都让了路，每一项记账照设置走。
    #[test]
    fn the_small_pool_concurrent_run_reads_back_what_the_foreground_wrote_and_every_waiting_answer_yields(
    ) {
        let image = small_pool_image();
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let setting = ConcurrentRunSetting {
            image: &image,
            discrete_gpus: &[],
            memory_leaked: &memory_leaked,
            vulkan_instance: None,
            user_changes: 8,
            foreground_interval: Duration::from_millis(5),
        };
        for arm in [ConcurrentArm::Cpu, ConcurrentArm::WithoutCompaction] {
            let run = run_concurrent_once(&setting, arm, 0).expect("CPU 上的两条臂跑得成");
            assert!(run.content_matches_the_expectation, "{}", arm.name());
            assert_eq!(run.latencies.len(), 8);
            assert_eq!(run.publishes_under_the_lock.len(), 8);
            assert_eq!(run.user_changes_failing_the_inclusion_check, 0);
            assert_eq!(run.calls_not_yielding_after_answering_waiting, 0);
            assert!(run.path_anchor.holds(), "{:?}", run.path_anchor);
            assert!(!run.the_foreground_waited_past_the_yield_bound);
            assert_eq!(run.gc_calls > 0, arm.runs_the_compaction());
        }
    }

    /// 第八节轨迹「每块盘的全空聚簇段数」（登记第十二节修订第 5 条）：每次调用开始前、返回后各一个读数，按盘各报峰值、
    /// 返回后比开始前多的调用数（相等不算）、期末值（最后一次调用返回后那一个）。
    #[test]
    fn the_empty_segment_trajectory_counts_calls_ending_with_more_empty_segments_per_device() {
        let with_readings = |before: Vec<u64>, after: Vec<u64>| {
            let mut record = call_record_with(1, Vec::new());
            record.empty_segments_before_the_call = before;
            record.empty_segments_after_the_call = after;
            record
        };
        let records = [
            with_readings(vec![10, 20], vec![9, 20]),
            with_readings(vec![9, 20], vec![12, 21]),
            with_readings(vec![12, 21], vec![12, 19]),
        ];
        let trajectories = empty_segments_trajectory_of_each_device(&records);
        assert_eq!(
            trajectories,
            vec![
                EmptySegmentsTrajectoryOfADevice {
                    peak: 12,
                    calls_gaining_empty_segments: 1,
                    last: 12,
                },
                EmptySegmentsTrajectoryOfADevice {
                    peak: 21,
                    calls_gaining_empty_segments: 1,
                    last: 19,
                },
            ]
        );
        assert_eq!(
            empty_segments_fields(&trajectories),
            "empty_segments_peak=12,21 empty_segments_calls_gaining=1,1 empty_segments_last=12,19"
        );
        assert!(empty_segments_trajectory_of_each_device(&[]).is_empty());
    }

    /// 第十条第 4 行照上界取法判（登记第十二节修订第 6 条）：打中记 `hit`，没打中只记「上界口径下没打中」；产物行写明是上界口径。
    #[test]
    fn the_yield_bound_clause_not_hit_is_reported_under_the_upper_bound_reading() {
        assert_eq!(yield_bound_failure_clause_reading(true), "hit");
        assert_eq!(
            yield_bound_failure_clause_reading(false),
            "not_hit_under_the_upper_bound_reading"
        );
    }

    /// 并发跑产物头一行写 `gc_thread_yields_after_unlock=true`（登记第十二节修订第 7 条：GC 线程放锁之后让一下）。
    #[test]
    fn the_head_line_of_a_concurrent_product_says_the_gc_thread_yields_after_unlock() {
        let fields = command_fields_of_a_concurrent_command(&ConcurrentCommand {
            tier: UnitAreaTier::FourGibibytes,
            foreground_interval: Duration::from_millis(50),
            arms_requested: vec![ConcurrentArm::Cpu],
            product: PathBuf::from("concurrent.out"),
        });
        assert!(
            fields
                .split(' ')
                .any(|field| field == "gc_thread_yields_after_unlock=true"),
            "{fields}"
        );
        assert!(fields.starts_with("command=concurrent reading=occupancy tier=4GiB "));
    }

    /// 第八节轨迹：预算读数照「B_gc 减这次用户改动之后的整理发布数」推，每次调用前后各一个；不确定条目每一项 GPU 记账一个；峰值、大于 0 的次数、期末值。
    #[test]
    fn the_trajectories_report_the_peak_the_positive_readings_and_the_last_reading() {
        let mut first_call = call_record_with(12, Vec::new());
        first_call.after_the_user_change = 0;
        let mut second_call = call_record_with(8, Vec::new());
        second_call.after_the_user_change = 0;
        let mut after_the_next_change = call_record_with(0, Vec::new());
        after_the_next_change.after_the_user_change = 1;
        let records = [first_call, second_call, after_the_next_change];
        assert_eq!(
            budget_left_before_and_after_each_call(&records),
            [20, 8, 8, 0, 20, 20]
        );
        assert_eq!(
            trajectory_of_the_readings(&budget_left_before_and_after_each_call(&records)),
            TrajectoryOfTheReadings {
                peak: 20,
                positive_readings: 5,
                last: 20,
            }
        );
        let gpu_records = [call_record_with(
            2,
            vec![
                gpu_record_with_unit_checksum_entries(3, 2, 0),
                gpu_record_with_unit_checksum_entries(5, 0, 0),
            ],
        )];
        assert_eq!(uncertain_entries_of_each_gpu_record(&gpu_records), [2, 0]);
        assert_eq!(
            trajectory_of_the_readings(&[]),
            TrajectoryOfTheReadings::default()
        );
    }

    // ---- 丙段代价（登记第十二节修订第 9 条）----

    /// 盘 `device` 上从单元区起点偏 `offset_from_the_unit_area_start` 槽（可以是负的：落在单元区之前）起、跨 `span_slots` 槽的一条记录。
    fn record_at_the_offset(
        device: u32,
        offset_from_the_unit_area_start: i64,
        span_slots: u16,
        is_released: bool,
    ) -> AllocationRecord {
        AllocationRecord {
            device: DeviceIdentity(device),
            slot: SlotNumber(
                LOCAL_UNIT_AREA_START_SLOT
                    .checked_add_signed(offset_from_the_unit_area_start)
                    .expect("槽号不为负"),
            ),
            span_slots,
            generation: singlefs_core::address::CheckpointTxg(7),
            is_released,
            quarantine_mark:
                singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        }
    }

    /// 装置自己数的每段活槽数钉死在手算的数上，并与核心层的 CPU 写法逐段相同：单元区三个整段加 10 槽的尾巴；
    /// 起在单元区之前的那条只数落进来的 2 槽，跨两段的那条两段各 4 槽，已释放的与别的盘上的不数，伸进尾巴的那条只数整段里的 2 槽。
    #[test]
    fn the_apparatus_count_of_live_slots_per_segment_pins_the_hand_counted_values_and_agrees_with_the_core(
    ) {
        let unit_area = UnitAreaOfOneDevice {
            device: DeviceIdentity(0),
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 3 * LOCAL_CLUSTER_SEGMENT_SLOTS + 10,
        };
        let records = [
            record_at_the_offset(0, -4, 6, false),
            record_at_the_offset(0, 60, 8, false),
            record_at_the_offset(0, 70, 2, true),
            record_at_the_offset(1, 64, 2, false),
            record_at_the_offset(0, 130, 2, false),
            record_at_the_offset(0, 190, 6, false),
        ];
        let by_the_apparatus =
            live_slots_of_each_segment_counted_by_the_apparatus(&records, unit_area);
        assert_eq!(by_the_apparatus, [2_u64 + 4, 4, 2 + 2]);
        assert_eq!(
            by_the_apparatus,
            live_slots_per_cluster_segment_by_the_records(&records, unit_area)
        );
    }

    /// 五遍取正中那一遍，四遍取正中两遍的平均，一遍都没有是零（修订第 9 条「取中位」）。
    #[test]
    fn the_median_wall_of_five_repeats_is_the_middle_one_and_of_four_the_mean_of_the_middle_two() {
        let nanoseconds = |values: &[u64]| -> Vec<Duration> {
            values
                .iter()
                .map(|value| Duration::from_nanos(*value))
                .collect()
        };
        assert_eq!(
            median_wall_of(&nanoseconds(&[500, 100, 400, 200, 300])),
            Duration::from_nanos(300)
        );
        assert_eq!(
            median_wall_of(&nanoseconds(&[40, 10, 30, 20])),
            Duration::from_nanos(25)
        );
        assert_eq!(median_wall_of(&[]), Duration::ZERO);
    }

    fn counts_of_the_passes(
        live_slots_of_each_pass: Vec<Vec<Option<u64>>>,
    ) -> StageThreeCountsOfTheArm {
        StageThreeCountsOfTheArm {
            walls: [500, 100, 400, 200, 300]
                .into_iter()
                .map(Duration::from_nanos)
                .take(live_slots_of_each_pass.len())
                .collect(),
            live_slots_of_each_pass,
            the_opening_thread_was_running_at_the_first_pass: false,
        }
    }

    /// 小池上（同几何核那两条小池用例的起步镜像）两条臂各数五遍：每一遍每段的活槽数逐段相同、都等于装置自己数的，活槽合计不少于
    /// P 个活数据单元的槽数；CPU 臂照产品 greedy 跑的那一次 GC 调用量得到挂钟、丙段那一截量不到。GPU 臂在这里是在 CPU 上算的替身
    /// （真卡那一条带 tests-on-a-real-gpu 另跑）。一个场景：造池太久，同一份起步镜像上按次序做两步。
    #[test]
    fn the_two_arms_count_the_same_live_slots_per_segment_on_the_small_pool_as_the_apparatus_and_a_greedy_call_is_timed(
    ) {
        let image = small_pool_image();
        let (allocation_records, unit_area) =
            allocation_records_and_unit_area_of_the_image(&image).expect("量挂载成");
        let by_the_apparatus =
            live_slots_of_each_segment_counted_by_the_apparatus(&allocation_records, unit_area);
        assert_eq!(
            u64::try_from(by_the_apparatus.len()).expect("段数"),
            unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS
        );
        assert!(
            by_the_apparatus.iter().sum::<u64>()
                >= SMALL_POOL_LIVE_DATA_UNITS * (LOCAL_DATA_UNIT_BYTES / LOCAL_SLOT_BYTES),
            "{}",
            by_the_apparatus.iter().sum::<u64>()
        );
        let on_the_cpu = stage_three_counts_of_the_counter(
            &mut StageThreeCounter::<CpuBackedStandIn>::OnTheCpu,
            &allocation_records,
            unit_area,
        )
        .expect("CPU 臂不出错");
        let on_the_stand_in = stage_three_counts_of_the_counter(
            &mut StageThreeCounter::OnTheGpu(CpuBackedStandIn {
                corrupted_crc32c: false,
            }),
            &allocation_records,
            unit_area,
        )
        .expect("替身不出错");
        let expected_pass: Vec<Option<u64>> = by_the_apparatus.iter().copied().map(Some).collect();
        for counts in [&on_the_cpu, &on_the_stand_in] {
            assert_eq!(counts.walls.len(), 5, "修订第 9 条：各量 5 次");
            assert_eq!(counts.live_slots_of_each_pass.len(), 5);
            assert!(
                counts
                    .live_slots_of_each_pass
                    .iter()
                    .all(|pass| *pass == expected_pass),
                "{counts:?}"
            );
            assert!(every_pass_agrees_with_the_apparatus_count(
                counts,
                &by_the_apparatus
            ));
        }
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let share = stage_three_share_of_one_greedy_call(
            &image,
            ComputeOfTheArm::Cpu,
            &[],
            &memory_leaked,
            None,
        )
        .expect("CPU 臂挂载得成");
        assert!(share.gc_call_wall > Duration::ZERO, "{share:?}");
        assert_eq!(share.stage_three, None);
    }

    /// 同上一条，GPU 点名一张卡臂换成真卡上产品那一份：五遍每段的活槽数都等于装置自己数的；greedy 那一次 GC 调用里丙段至少调了一次、
    /// 那一截不长于整次调用。要本进程看得到至少一张独显。
    #[cfg(feature = "tests-on-a-real-gpu")]
    #[test]
    fn the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool(
    ) {
        let vulkan_instance = VulkanInstanceOfThisProcess::created_for_this_process();
        let discrete_gpus =
            discrete_gpus_seen_by_this_process(&vulkan_instance).expect("枚举得到适配器");
        assert!(
            !discrete_gpus.is_empty(),
            "带 tests-on-a-real-gpu 跑，本进程要看得到至少一张独显"
        );
        let image = small_pool_image();
        let (allocation_records, unit_area) =
            allocation_records_and_unit_area_of_the_image(&image).expect("量挂载成");
        let by_the_apparatus =
            live_slots_of_each_segment_counted_by_the_apparatus(&allocation_records, unit_area);
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let on_the_card = stage_three_counts_on_the_arm(
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
            &allocation_records,
            unit_area,
            &discrete_gpus,
            &memory_leaked,
            Some(&vulkan_instance),
        )
        .expect("GPU 点名一张卡臂数得成");
        assert_eq!(on_the_card.walls.len(), 5);
        assert!(
            every_pass_agrees_with_the_apparatus_count(&on_the_card, &by_the_apparatus),
            "{on_the_card:?}"
        );
        let share = stage_three_share_of_one_greedy_call(
            &image,
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
            &discrete_gpus,
            &memory_leaked,
            Some(&vulkan_instance),
        )
        .expect("GPU 点名一张卡臂挂载得成");
        let stage_three = share.stage_three.expect("GPU 臂外面套了丙段计时");
        assert!(stage_three.calls >= 1, "{share:?}");
        assert!(stage_three.wall <= share.gc_call_wall, "{share:?}");
    }

    /// 两种产物行的形态：修订第 9 条写的四个键在前、次序照写，装置另报的在后；CPU 臂丙段那一截写量不到；结果行读回来认得出种类与键。
    #[test]
    fn the_stage_three_lines_carry_the_registered_keys_first_and_in_order() {
        let agreeing = vec![Some(6), Some(4), Some(4)];
        let cost_fields = stage_three_cost_fields(
            UnitAreaTier::FourGibibytes,
            ComputeOfTheArm::Cpu,
            &counts_of_the_passes(vec![agreeing; 5]),
            &[6, 4, 4],
        );
        assert_eq!(
            cost_fields,
            "tier=4GiB arm=cpu median_nanoseconds=300 repeats=5 nanoseconds_of_each_repeat=500,100,400,200,300 \
             segments=3 live_slots=14 every_repeat_agrees_with_the_apparatus_count=true \
             gpu_opening_thread_was_running_at_the_first_repeat=false"
        );
        let share_without_the_stage_three = StageThreeShareOfOneGreedyCall {
            gc_call_wall: Duration::from_nanos(7000),
            stage_three: None,
            batches: 3,
            the_opening_thread_was_running_at_the_call: false,
        };
        assert_eq!(
            stage_three_share_fields(
                UnitAreaTier::FourGibibytes,
                ComputeOfTheArm::Cpu,
                &share_without_the_stage_three
            ),
            "tier=4GiB arm=cpu gc_call_nanoseconds=7000 stage_three_nanoseconds=not_reachable_from_outside_the_core \
             stage_three_calls=not_reachable_from_outside_the_core batches=3 gpu_opening_thread_was_running_at_the_call=false"
        );
        let share_fields = stage_three_share_fields(
            UnitAreaTier::FourGibibytes,
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
            &StageThreeShareOfOneGreedyCall {
                gc_call_wall: Duration::from_nanos(7000),
                stage_three: Some(StageThreeTiming {
                    calls: 2,
                    wall: Duration::from_nanos(900),
                }),
                batches: 3,
                the_opening_thread_was_running_at_the_call: false,
            },
        );
        assert_eq!(
            share_fields,
            "tier=4GiB arm=gpu_one_card gc_call_nanoseconds=7000 stage_three_nanoseconds=900 stage_three_calls=2 \
             batches=3 gpu_opening_thread_was_running_at_the_call=false"
        );
        let (kind, fields) = fields_of_a_result_line(&format!(
            "{RESULT_LINE_PREFIX} name=stage_three_share {share_fields}"
        ))
        .expect("认得出是结果行");
        assert_eq!(kind, "stage_three_share");
        assert_eq!(
            fields.get("stage_three_nanoseconds").map(String::as_str),
            Some("900")
        );
    }

    /// 丙段计时的包装：每次丙段记一次、判定原样交回（里面是在 CPU 上算的替身）；别的段不记。
    #[test]
    fn the_stage_three_timing_wrapper_counts_every_live_slot_call_and_hands_the_decisions_through()
    {
        let records: Vec<AllocationRecord> = [(0, 16), (1, 32)]
            .into_iter()
            .flat_map(|(segment_index, units)| {
                data_unit_records_in_the_segment(segment_index, units)
            })
            .collect();
        let unit_area = UnitAreaOfOneDevice {
            device: DeviceIdentity(0),
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 4 * LOCAL_CLUSTER_SEGMENT_SLOTS,
        };
        let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
        let timing: SharedStageThreeTiming = Arc::new(Mutex::new(StageThreeTiming::default()));
        let mut wrapper = GpuComputeTimingTheStageThree {
            inner: CpuBackedStandIn {
                corrupted_crc32c: false,
            },
            timing: Arc::clone(&timing),
        };
        let contents: [Vec<u8>; 1] = [vec![1_u8; 32768]];
        let _checksums = wrapper
            .verify_unit_checksums(&units_to_verify(&contents))
            .expect("替身不出错");
        for _call in 0..2 {
            let decisions = wrapper
                .count_live_slots_per_cluster_segment(&records, unit_area, &segments)
                .expect("替身不出错");
            assert_eq!(
                decisions,
                [32, 64, 0, 0].map(GpuEntryDecision::ConfirmedByBothKernels)
            );
        }
        assert_eq!(timing.lock().expect("没在别处 panic 过").calls, 2);
    }

    /// 每一遍逐段相同才算对得上：有一遍差一段、有一段这一轮不确定、段数少一段，都对不上。
    #[test]
    fn one_pass_differing_from_the_apparatus_in_one_segment_or_left_uncertain_does_not_agree() {
        let by_the_apparatus = [6_u64, 4, 4];
        let agreeing = vec![Some(6), Some(4), Some(4)];
        assert!(every_pass_agrees_with_the_apparatus_count(
            &counts_of_the_passes(vec![agreeing.clone(); 5]),
            &by_the_apparatus
        ));
        for odd_pass in [
            vec![Some(6), Some(5), Some(4)],
            vec![Some(6), None, Some(4)],
            vec![Some(6), Some(4)],
        ] {
            let mut passes = vec![agreeing.clone(); 4];
            passes.push(odd_pass.clone());
            assert!(
                !every_pass_agrees_with_the_apparatus_count(
                    &counts_of_the_passes(passes),
                    &by_the_apparatus
                ),
                "{odd_pass:?}"
            );
        }
    }

    /// 三档造池要的 inode 记录都装得进一层 inode 树（登记第五节「负载」第 4 条改的造法；树还不会长高）：一层装几条、每档要几条、
    /// 占几片叶容器，都照核心层写记录那一步现算（`inode_tree::write_records_into_leaf_containers` 与 `leaf_containers_one_root_node_holds`），
    /// 再拿另外手算的绝对值钉住：一片 233 条 × 扇出 135 = 31455 条；活数据文件 P ÷ 16，垫片文件至多 ⌈(P ÷ 2 + 32) ÷ 16⌉，
    /// 加造数据挂载之前那 1 条与造碎片挂载推发布的至多 16 个空 inode，片数 ⌈条数 ÷ 233⌉。
    /// 判别力：P 个一单元文件（上一造法）三档都装不下，核心层拒成「叶容器多于一个码 2 根装得下的」，同三档 `stage-three-cost` 在建池那一步撞上的拒绝。
    #[test]
    fn the_construction_of_every_tier_fits_a_one_level_inode_tree() {
        let one_level_tree_holds = inode_records_one_level_tree_holds();
        assert_eq!(one_level_tree_holds, 233 * 135);
        let containers_one_root_node_holds =
            u64::try_from(leaf_containers_one_root_node_holds()).expect("扇出装得进 u64");
        for (tier, files, spacer_files, containers) in [
            (
                UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
                2288,
                1146,
                15,
            ),
            (UnitAreaTier::FourGibibytes, 4096, 2050, 27),
            (UnitAreaTier::EightGibibytes, 8192, 4098, 53),
        ] {
            let live_data_units = tier.live_data_units();
            assert_eq!(live_data_files(live_data_units), files, "{}", tier.name());
            assert_eq!(
                spacer_files_at_most(live_data_units),
                spacer_files,
                "{}",
                tier.name()
            );
            let records = inode_records_of_the_construction_at_most(
                live_data_units,
                INODE_RECORDS_BEFORE_THE_DATA_MOUNT,
            );
            assert_eq!(records, 1 + files + spacer_files + 16, "{}", tier.name());
            assert!(records <= one_level_tree_holds, "{}", tier.name());
            assert_eq!(
                inode_leaf_containers_of_a_one_level_tree(records),
                Ok(containers),
                "{}",
                tier.name()
            );
            assert!(
                containers <= containers_one_root_node_holds,
                "{}",
                tier.name()
            );
        }
        for tier in UnitAreaTier::EVERY_TIER {
            let records_of_one_unit_files =
                INODE_RECORDS_BEFORE_THE_DATA_MOUNT + tier.live_data_units() + 1;
            assert!(
                records_of_one_unit_files > one_level_tree_holds,
                "{}",
                tier.name()
            );
            assert!(
                matches!(
                    inode_leaf_containers_of_a_one_level_tree(records_of_one_unit_files),
                    Err(InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { .. })
                ),
                "{}",
                tier.name()
            );
        }
    }

    /// 前台用户改动只覆盖写活数据文件的最后一个单元（按偏移写重写到文件末尾，R4：取别的单元一次要重写好几个），文件按种子在 P ÷ 16 个里均匀取。
    /// 头几次的单元下标钉死（另用同一个 splitmix64 式子手算）；256 次在小池上落到 100 个以上的文件。
    #[test]
    fn the_user_change_overwrites_the_last_unit_of_a_live_data_file_drawn_by_the_seed() {
        let drawn = |live_data_units: u64| -> Vec<u64> {
            (0..6)
                .map(|change_number| unit_index_of_the_user_change(change_number, live_data_units))
                .collect()
        };
        assert_eq!(
            drawn(SMALL_POOL_LIVE_DATA_UNITS),
            [831_u64, 1743, 511, 1343, 1967, 1535]
        );
        assert_eq!(
            drawn(UnitAreaTier::EightGibibytes.live_data_units()),
            [52799_u64, 110959, 31823, 85487, 125055, 97759]
        );
        for live_data_units in [
            SMALL_POOL_LIVE_DATA_UNITS,
            UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes.live_data_units(),
        ] {
            let mut files = BTreeSet::new();
            for change_number in 0..256 {
                let unit_index = unit_index_of_the_user_change(change_number, live_data_units);
                let (file, index_in_file) = file_and_index_of_a_live_data_unit(unit_index);
                assert!(unit_index < live_data_units);
                assert_eq!(index_in_file + 1, LIVE_DATA_UNITS_PER_FILE);
                files.insert(file);
            }
            assert!(files.len() > 100, "{live_data_units}：{}", files.len());
        }
    }
}
