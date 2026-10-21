//! E185（GC一批搬迁的挂钟拆分）全部四个子命令（跑前登记 `research/prompts/e185-preregistration.md` 第五节「装置住哪」写死）：
//! 在 `crates/` 今天的 GC 路径（`MountedSession::run_compaction`）上量一批搬迁的挂钟，拆成等盘读写与 CPU，再把 CPU 拆成三段
//! （甲 = 校验和核对、乙 = 区域扫描里其余的解头与映射反查、丙 = 选段打分），量 GPU 一次提交的固定开销与每字节开销、GC 单跑的写带宽。
//! 回答问题单 `research/prompts/m4-gpugc-measure-forks.md` 的四行。
//!
// admission: always 每跑一次都在今天这份 crates 上现造池、现量挂钟，计时每次都是新观测，上一次跑完不替这一次作保
// run-condition: command python3
//!
//! 子命令：
//!   wall-clock --image-directory <目录> [--point <活槽占比>:<池的数据量>]…
//!       第一段：常量回比、造池并核几何、全部取样点在内存臂与真盘臂上各 5 轮、第一行两条臂的阳性对照（对照点上与基线逐轮交错，登记第十二节 12.9）、
//!       设备顺序写带宽 Q15 与它的对照、
//!       第七节锚点与第十一节停机条款逐项对拍。真盘臂的镜像放 `<目录>` 下（要在本机那块 NVMe 的 ext4 上，不放开机清空的目录）。
//!       `<目录>` 给相对路径时按当前目录解析；产物不打这个路径（`name=configuration` 只打它的文件系统种类、在不在开机清空的地方与设备号）。
//!   stage-split [--point <活槽占比>:<池的数据量>]…
//!       第二段：内存臂上三个阶段的重放计时与 2 ms 对照、饱和吞吐 1 / 16 / N 线程、丙段的盘大小敏感性、停机条款第 4 条。
//!   gpu-submission --wall-clock-product <文件> --stage-split-product <文件> --cards <nvidia-smi 序号,…>
//!       第三段（编了 `gpu` 特性才有）：每张卡一次提交的 a、b、物理下限与 CRC 对拍，用第一段的 B_批 算 φ 与 n*，用第二段的饱和吞吐算 Q9。
//!       没编 `gpu` 特性时退 78，报「本次未跑：没编 gpu 特性」。整条命令经 `bash scripts/crash/gpu-quiesce-run.sh -- …` 起。
//!   foreground-fsync --image-directory <目录> --point <活槽占比>:<池的数据量> --batch-units <K> [--seconds <秒>]
//!       第四段（附带，只在 Q16 > 10% 时跑）：前台「4 KiB 直接写 + fdatasync」循环的延迟，GC 并发与不跑 GC 各 5 轮。
//!
//! 线程数取环境变量 `E185_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值（编进来的那一份）。
//! 产物是标准输出上的 `E7RESULT` 行，末行 `name=done emitted=<行数>`；进度打到标准错误。
//! 退出码：0 正常；2 命令行不对；3 停机条款触发（装置与 `crates/` 对不上，或开跑自检判线程 CPU 时钟分辨率不够；产物照打、交主 agent）；4 阳性对照没过（那条臂整段作废，产物照打）；
//! 5 等别的性能测量（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）等满上限仍没等到；
//! 78 准入与运行条件没满足（`preflight` 拒绝，带 `--force` 才照跑），或 `gpu-submission` 没编 `gpu` 特性、清场或取卡没过。
//!
//! 决定工作量、几何或判据门槛的常量都是本地常量（值抄自登记第五节「装置读 `crates/` 的这些常量」那张表），每个子命令开跑先与
//! `crates/` 的同名常量回比（`constants_disagreeing_with_the_crates`，停机条款第 1 条）。CPU 时间一律读本线程的
//! `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`（登记第一节「CPU 时间」那一行，第十二节 12.2 修订），`wall-clock`、`stage-split`、`foreground-fsync`
//! 开跑先在本线程上自检这个钟的分辨率（1 ms 里前进不到 100 次就退 3）。
//!
//! 不计时的准备（登记第十二节 12.10「并行与挂钟」、12.11 全局队列）：`wall-clock`、`stage-split` 一段里全部准备单位（各点造池与几何核、
//! 各点第 2–5 轮重造、第一段各点的真盘镜像）进一个队列，按量的次序派到 min(线程数, 内存预算 ÷ 一份峰值) 条线程上，每份一条线程、各自建状态；
//! 重造不等几何核，按头一次的配方推测着先造，几何核改了负载的那一点推测造的作废、按新配方重造。量的那一侧要哪一份就取哪一份，
//! 还没好就跑一次准备：装得下全部就一次做完，装不下就分批，批内并行、批间串行量；真盘镜像在量那一点之前建，盘上同一时刻只有一点的镜像。
//! 线程数同上；内存上限取环境变量 `E185_MEMORY_CAP`（`28G` 这种写法），没设取本进程控制组的 `memory.max`，再没有取 28G。
//! 计时窗口开始之前核本进程只剩一条线程（`/proc/self/status`）。准备时的产物行先攒着，轮到那个点量的时候照串行的次序打；每次准备另打一行 `name=preparation`。
//!
//! 串行段当场声明（登记第十二节 12.11）：设了环境变量 `SINGLEFS_SERIAL_DECLARATIONS` 时，每次准备开始往它指的文件追加 `<unix 秒>\t此刻并行`，
//! 每个计时窗口开始之前追加 `<unix 秒>\t此刻串行：<理由>`（同一段不重写；饱和吞吐本身多线程，开始前追加一行并行）；写法在
//! `scripts/hosts/core_usage.py` 文件头「声明」。没设就不写；设了而写不进，打一行 `name=stopped reason=serial_declaration_not_written:…`、退 3。

#[path = "../../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::os::unix::fs::{FileExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InodeNumber, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DirectInputOutputBlockDevice, DiscardOutcome, PageCachePolicy,
    PhysicalBlockSizeInBytes, PhysicalBlockSizeSource, WriteDurability,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::{
    choose_region, publish_budget_of_a_compaction_run,
    region_scans_of_the_segments_without_an_unreleased_record, CompactionBatchSize,
    CompactionBatchUnits, CompactionSwitches, RegionChoice, RegionChoiceCriterion,
    SegmentsRefusedByTheRegionScan, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable_with_compaction_switches, raise_rollback_floor_to_the_admission_ceiling,
};
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession, UserChange};
use singlefs_core::recovery::PoolReader;
use singlefs_core::region_scan::{
    scan_region, AllocationInRegion, CentralMappingEntry, RegionScan,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file, warm_up,
    FirstFile, MappingLookup, PoolVersion, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{data_unit_payload_capacity, UNIT_CLASS_DATA, UNIT_CLASS_PACKED};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::sha256::sha256_digest;

// ───────────────────────────── 本地常量（停机条款第 1 条回比） ─────────────────────────────

/// 槽字节：值抄自登记第五节常量表（回比 `singlefs_format::SLOT_BYTES`）。
const LOCAL_SLOT_BYTES: u64 = 16_384;
/// 码 2 节点字节（回比 `singlefs_format::NODE_BYTES`）。
const LOCAL_NODE_BYTES: u64 = 16_384;
/// 码 1 数据单元字节（回比 `singlefs_format::DATA_UNIT_BYTES`）。
const LOCAL_DATA_UNIT_BYTES: u64 = 32_768;
/// 聚簇段槽数（回比 `singlefs_format::CLUSTER_SEGMENT_SLOTS`）。
const LOCAL_CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// 分配记录一条的字节（回比 `singlefs_format::ALLOCATION_RECORD_BYTES`）。
const LOCAL_ALLOCATION_RECORD_BYTES: u64 = 20;
/// 每块盘字节：harness 测试共用模块的 `IMAGE_BYTES`（回比它，另回比建出来的内存盘的 `size_in_bytes()`）。
const LOCAL_DEVICE_BYTES: u64 = 4_294_967_296;
/// GPU 一段的槽字节、结果条目字节、派活参数字节（`gpu_unit_checks` 里私有，回比公开的 `bytes_crc32c_of_ranges_needs(n)`）。
const LOCAL_GPU_SLOT_BYTES: u64 = 32_768;
const LOCAL_GPU_RESULT_ENTRY_BYTES: u64 = 8;
const LOCAL_GPU_PARAMETER_BYTES: u64 = 16;
/// 发布预算回比的两个「已经过去的发布数」取值（登记第五节常量表）。
const PUBLISHES_ALREADY_PASSED_COMPARED: [u64; 2] = [0, 1000];
/// 装置把会话的「已经过去的发布数」设成它，发布预算落到下限 1（登记第一节「一次调用一次发布」）。
const PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE: u64 = 1000;
/// 物理块（内存盘与真盘臂的声明值，登记第一节「真盘」那一行的逻辑块 512）。
const PHYSICAL_BLOCK_BYTES: u32 = 512;

/// 本地的发布预算算式 `max(1, 3S − 4 − p)`（登记第五节常量表那一行；回比 `publish_budget_of_a_compaction_run`）。
fn local_publish_budget(root_ring_slots_per_region: u64, publishes_already_passed: u64) -> u64 {
    let budget = (3 * root_ring_slots_per_region).saturating_sub(4);
    budget.saturating_sub(publishes_already_passed).max(1)
}

/// 本地的 GPU 一次调用建的缓冲字节：`n × (32768 + 4 + 2 × 8) + 16`（登记第五节常量表、第七节乙类）。
fn local_gpu_buffer_bytes(range_count: u64) -> u64 {
    range_count * (LOCAL_GPU_SLOT_BYTES + 4 + 2 * LOCAL_GPU_RESULT_ENTRY_BYTES)
        + LOCAL_GPU_PARAMETER_BYTES
}

/// 本地常量与 `crates/` 同名常量回比（停机条款第 1 条）：对不上的逐条交回（名字、本地值、实现值）。GPU 那几个只在编了 `gpu` 特性时比，
/// 没编时调用方另报一行 `status=not_run`（不算对不上）。
fn constants_disagreeing_with_the_crates() -> Vec<(String, String, String)> {
    let mut disagreeing = Vec::new();
    let mut compare = |name: &str, local_value: u64, crates_value: u64| {
        if local_value != crates_value {
            disagreeing.push((
                name.to_string(),
                local_value.to_string(),
                crates_value.to_string(),
            ));
        }
    };
    compare("slot_bytes", LOCAL_SLOT_BYTES, singlefs_format::SLOT_BYTES);
    compare("node_bytes", LOCAL_NODE_BYTES, singlefs_format::NODE_BYTES);
    compare(
        "data_unit_bytes",
        LOCAL_DATA_UNIT_BYTES,
        singlefs_format::DATA_UNIT_BYTES,
    );
    compare(
        "cluster_segment_slots",
        LOCAL_CLUSTER_SEGMENT_SLOTS,
        singlefs_format::CLUSTER_SEGMENT_SLOTS,
    );
    compare(
        "allocation_record_bytes",
        LOCAL_ALLOCATION_RECORD_BYTES,
        singlefs_format::ALLOCATION_RECORD_BYTES,
    );
    compare("device_bytes", LOCAL_DEVICE_BYTES, common::IMAGE_BYTES);
    let memory_device = SparseBlockDevice::new(
        LOCAL_DEVICE_BYTES,
        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
    );
    compare(
        "device_bytes_of_the_memory_device",
        LOCAL_DEVICE_BYTES,
        memory_device.size_in_bytes(),
    );
    let root_ring_slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count();
    for publishes_already_passed in PUBLISHES_ALREADY_PASSED_COMPARED {
        compare(
            &format!("publish_budget_at_{publishes_already_passed}"),
            local_publish_budget(root_ring_slots_per_region, publishes_already_passed),
            publish_budget_of_a_compaction_run(
                root_ring_slots_per_region,
                publishes_already_passed,
            ),
        );
    }
    #[cfg(feature = "gpu")]
    for range_count in [1_u64, 1024, 4096] {
        compare(
            &format!("gpu_buffer_bytes_of_{range_count}_ranges"),
            local_gpu_buffer_bytes(range_count),
            singlefs_checker_tier::gpu_unit_checks::bytes_crc32c_of_ranges_needs(
                usize::try_from(range_count).expect("几千段装得进 usize"),
            ),
        );
    }
    disagreeing
}

// ───────────────────────────── 取样点与判据门槛（登记第六、八节，跑前写死） ─────────────────────────────

/// 段内活槽占比 f（登记第八节）：按槽序每 4 个单元保留 1 个（0.25）或 3 个（0.75），活槽数 = 64 × f。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum LiveSlotFraction {
    OneQuarter,
    ThreeQuarters,
}

impl LiveSlotFraction {
    const ALL: [LiveSlotFraction; 2] = [
        LiveSlotFraction::OneQuarter,
        LiveSlotFraction::ThreeQuarters,
    ];

    const fn label(self) -> &'static str {
        match self {
            LiveSlotFraction::OneQuarter => "0.25",
            LiveSlotFraction::ThreeQuarters => "0.75",
        }
    }

    fn of_label(label: &str) -> Option<LiveSlotFraction> {
        LiveSlotFraction::ALL
            .into_iter()
            .find(|fraction| fraction.label() == label)
    }

    /// 每 4 个单元留下几个。
    const fn kept_units_of_every_four(self) -> u64 {
        match self {
            LiveSlotFraction::OneQuarter => 1,
            LiveSlotFraction::ThreeQuarters => 3,
        }
    }

    /// 一段里活槽数 64 × f（R 那块盘上未释放记录罩住的槽）。
    const fn live_slots_per_segment(self) -> u64 {
        LOCAL_CLUSTER_SEGMENT_SLOTS * self.kept_units_of_every_four() / 4
    }

    /// 一段里活的数据单元数 m（每个数据单元跨 2 槽）。
    const fn data_units_per_segment(self) -> u64 {
        self.live_slots_per_segment() * LOCAL_SLOT_BYTES / LOCAL_DATA_UNIT_BYTES
    }

    /// 一段里按槽序第 `position_in_segment` 个数据单元要不要覆盖写掉（覆盖写掉的就是这一段里被释放的）。
    const fn is_overwritten(self, position_in_segment: u64) -> bool {
        match self {
            LiveSlotFraction::OneQuarter => !position_in_segment.is_multiple_of(4),
            LiveSlotFraction::ThreeQuarters => position_in_segment.is_multiple_of(4),
        }
    }
}

/// 一个取样点：活槽占比与池的数据量 P（建池时写入的码 1 数据单元个数，覆盖写之前）。批大小与设备另算。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SamplingPoint {
    fraction: LiveSlotFraction,
    pool_data_units: u64,
}

impl SamplingPoint {
    fn label(self) -> String {
        format!(
            "fraction={} pool_data_units={}",
            self.fraction.label(),
            self.pool_data_units
        )
    }

    /// 命令行的 `<活槽占比>:<池的数据量>`。
    fn of_argument(argument: &str) -> Option<SamplingPoint> {
        let (fraction_text, units_text) = argument.split_once(':')?;
        let fraction = LiveSlotFraction::of_label(fraction_text)?;
        let pool_data_units = units_text.parse::<u64>().ok()?;
        POOL_DATA_UNITS
            .contains(&pool_data_units)
            .then_some(SamplingPoint {
                fraction,
                pool_data_units,
            })
    }
}

/// 池的数据量 P 的两点（登记第八节）。
const POOL_DATA_UNITS: [u64; 2] = [1024, 8192];
/// 批大小 K 的三点（登记第八节）。
const BATCH_SIZES: [u64; 3] = [4, 8, 16];
/// 每个取样点、每个设备的轮数（`test-discipline.md`「单次观测不算数」）。
const ROUNDS: usize = 5;
/// 一轮作废之后最多重跑几次（登记第十一节作废条款「那一轮作废、重跑」）；用完仍作废的那一轮记作废，不补。
const ATTEMPTS_PER_ROUND_AT_MOST: usize = 3;
/// 几何没造成时最多改几次负载重造（登记第五节「改了三次仍造不成的」）。
const WORKLOAD_CHANGES_AT_MOST: usize = 3;

/// 全部取样点，按（活槽占比、P）的次序。
fn every_sampling_point() -> Vec<SamplingPoint> {
    LiveSlotFraction::ALL
        .into_iter()
        .flat_map(|fraction| {
            POOL_DATA_UNITS
                .into_iter()
                .map(move |pool_data_units| SamplingPoint {
                    fraction,
                    pool_data_units,
                })
        })
        .collect()
}

/// 第一行两条臂的阳性对照取样点：f = 0.75、K = 8、P = 1024（登记第五节阳性对照表第 1 行）。
const CONTROL_POINT: SamplingPoint = SamplingPoint {
    fraction: LiveSlotFraction::ThreeQuarters,
    pool_data_units: 1024,
};
const CONTROL_BATCH_UNITS: u64 = 8;
/// 阳性对照往每次 `write_at` 里加的那 2 ms。
const INJECTED_MILLISECONDS_PER_WRITE: u64 = 2;

/// Q1：ρ ≤ 它判「盘」（登记第六节；「理想卸载省下 ≤ 1/3」那条边）。
const RHO_AT_MOST_FOR_DISK: f64 = 0.5;
/// Q1：ρ ≥ 它判「CPU」（「省下 ≥ 2/3」那条边）。
const RHO_AT_LEAST_FOR_CPU: f64 = 2.0;
/// Q2：真盘 ÷ 内存的挂钟比 ≥ 它判「盘」。
const WALL_RATIO_AT_LEAST_FOR_DISK: f64 = 3.0;
/// Q2：≤ 它判「CPU」。
const WALL_RATIO_AT_MOST_FOR_CPU: f64 = 1.5;
/// Q3：两条路的 CPU 对账差 ≤ 它判「两路相容」。
const CPU_RECONCILIATION_AT_MOST: f64 = 0.20;
/// Q5–Q7：份额 ≥ 它判「值得」。
const STAGE_SHARE_AT_LEAST_FOR_WORTH: f64 = 0.10;
/// Q9：D24 形式的比 < 它判「这一段在 GPU 上输」（D24 已定项 3 逐字「小于 1 就否掉那一格」）。
const SIXTEEN_THREAD_TO_GPU_RATIO_BELOW_FOR_GPU_LOSES: f64 = 1.0;
/// Q13：φ ≤ 它判「一批一交够」。
const FIXED_SHARE_AT_MOST_FOR_ONE_BATCH: f64 = 0.10;
/// Q13：φ ≥ 它判「要攒批」。
const FIXED_SHARE_AT_LEAST_FOR_MERGING: f64 = 0.50;
/// Q16：GC 单跑写带宽 ÷ 设备顺序写带宽 ≤ 它判「单跑够」。
const BANDWIDTH_RATIO_AT_MOST_FOR_SINGLE_RUN: f64 = 0.10;
/// Q17：并发时前台 p99 ÷ 不跑 GC 时的 p99 ≥ 它判「GC 明显拖前台」。
const FOREGROUND_PERCENTILE_99_RATIO_AT_LEAST_FOR_DRAG: f64 = 2.0;
/// 包含自检：挂钟 − CPU − 等盘超过挂钟的这一份，那一轮作废（登记第十一节「机器被别的活挤」）。
const UNACCOUNTED_SHARE_ABOVE_FOR_INVALID: f64 = 0.10;
/// 第一行阳性对照：盘臂等盘增量落在注入时间的 [1.0, 1.5] 倍里；CPU 变化、等盘变化在 ±10% 以内。
const DISK_CONTROL_RATIO_LOWER: f64 = 1.0;
const DISK_CONTROL_RATIO_UPPER: f64 = 1.5;
const CONTROL_TOLERANCE: f64 = 0.10;
/// 第二行阳性对照：甲算两遍的时间是一遍的 [1.8, 2.2] 倍。
const DOUBLE_CRC_RATIO_LOWER: f64 = 1.8;
const DOUBLE_CRC_RATIO_UPPER: f64 = 2.2;
/// 第四行阳性对照：走页缓存不同步的带宽 ≥ `O_DIRECT` 带同步的 1.15 倍。
const PAGE_CACHE_SPEEDUP_AT_LEAST: f64 = 1.15;

/// 退出码（文件头）。
const EXIT_CODE_OK: i32 = 0;
const EXIT_CODE_USAGE: i32 = 2;
const EXIT_CODE_STOP_CLAUSE: i32 = 3;
const EXIT_CODE_POSITIVE_CONTROL_FAILED: i32 = 4;
const EXIT_CODE_OTHER_MEASUREMENT_STILL_RUNNING: i32 = 5;
const EXIT_CODE_NOT_RUN: i32 = 78;

// ───────────────────────────── 产物行、线程数、小工具 ─────────────────────────────

/// 产物行：每行 `E7RESULT ` 起头，收尾 `name=done emitted=<行数>`（收尾行自己也数进去）。
/// 串行段声明写到哪：`scripts/hosts/multi-host-run.sh --peer` 与 `scripts/hosts/detached-run.sh start` 设的那个文件
/// （写法在 `scripts/hosts/core_usage.py` 文件头「声明」）。
const SERIAL_DECLARATIONS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_SERIAL_DECLARATIONS";
const SERIAL_DECLARATION_MARK: &str = "此刻串行：";
const PARALLEL_DECLARATION_MARK: &str = "此刻并行";
/// `core_usage.py` 不认理由不到 8 个字的串行声明。
const SERIAL_REASON_CHARACTERS_AT_LEAST: usize = 8;

/// 当场声明的一段。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeclaredSegment {
    /// 从这一刻起是串行段，带理由。
    Serial(&'static str),
    /// 串行段到这一刻结束。
    Parallel,
}

/// 声明文件里的一行：`<unix 秒>\t此刻串行：<理由>` 或 `<unix 秒>\t此刻并行`。
///
/// # Panics
/// 串行段的理由不到 8 个字（`core_usage.py` 不认它）。
fn declaration_line(segment: DeclaredSegment, unix_seconds: f64) -> String {
    match segment {
        DeclaredSegment::Serial(reason) => {
            assert!(
                reason.chars().count() >= SERIAL_REASON_CHARACTERS_AT_LEAST,
                "串行段的理由至少 {SERIAL_REASON_CHARACTERS_AT_LEAST} 个字：「{reason}」"
            );
            format!("{unix_seconds:.3}\t{SERIAL_DECLARATION_MARK}{reason}")
        }
        DeclaredSegment::Parallel => format!("{unix_seconds:.3}\t{PARALLEL_DECLARATION_MARK}"),
    }
}

/// 往声明文件里追加串行、并行段（登记第十二节 12.11）。没设文件就不写；与上一条相同的不再写。
struct SerialDeclarations {
    path: Option<PathBuf>,
    last_declared: Option<DeclaredSegment>,
}

impl SerialDeclarations {
    fn from_the_environment() -> SerialDeclarations {
        SerialDeclarations {
            path: std::env::var_os(SERIAL_DECLARATIONS_ENVIRONMENT_VARIABLE)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from),
            last_declared: None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code, reason = "只有单测指一个临时文件"))]
    fn into_the_file(path: PathBuf) -> SerialDeclarations {
        SerialDeclarations {
            path: Some(path),
            last_declared: None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code, reason = "只有单测用不写文件的声明"))]
    fn nowhere() -> SerialDeclarations {
        SerialDeclarations {
            path: None,
            last_declared: None,
        }
    }

    /// 声明一段，交回这一次写没写（没设文件、与上一条相同都不写）。
    ///
    /// # Errors
    /// 声明文件打不开或写不进。
    fn declare(&mut self, segment: DeclaredSegment) -> std::io::Result<bool> {
        let Some(path) = &self.path else {
            return Ok(false);
        };
        if self.last_declared == Some(segment) {
            return Ok(false);
        }
        let unix_seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(std::io::Error::other)?
            .as_secs_f64();
        let line = declaration_line(segment, unix_seconds);
        let mut file = OpenOptions::new().append(true).create(true).open(path)?;
        file.write_all(format!("{line}\n").as_bytes())?;
        self.last_declared = Some(segment);
        Ok(true)
    }
}

struct Emitter {
    emitted: u64,
    declarations: SerialDeclarations,
}

impl Emitter {
    /// 串行段声明写到 `SINGLEFS_SERIAL_DECLARATIONS` 指的文件（没设就不写）。
    fn new() -> Emitter {
        Emitter::with_declarations(SerialDeclarations::from_the_environment())
    }

    fn with_declarations(declarations: SerialDeclarations) -> Emitter {
        Emitter {
            emitted: 0,
            declarations,
        }
    }

    /// 当场声明串行或并行段。写不进声明文件就照停机条款的样子停下：打一行 `name=stopped`、收尾，退 3。
    fn declare(&mut self, segment: DeclaredSegment) {
        if let Err(error) = self.declarations.declare(segment) {
            self.line(&format!(
                "name=stopped reason=serial_declaration_not_written:{}",
                field_token(&error.to_string())
            ));
            self.finish();
            std::process::exit(EXIT_CODE_STOP_CLAUSE);
        }
    }

    fn line(&mut self, text: &str) {
        self.emitted += 1;
        let mut standard_output = std::io::stdout().lock();
        writeln!(standard_output, "E7RESULT {text}").expect("标准输出写得进");
        standard_output.flush().expect("标准输出冲得出");
    }

    fn finish(&mut self) {
        let emitted_with_this_line = self.emitted + 1;
        self.line(&format!("name=done emitted={emitted_with_this_line}"));
    }
}

/// 一段说明文字放进产物行的一个字段：空白换成下划线，等号换成冒号（一行按空格、等号切字段）。
fn field_token(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            ' ' | '\t' | '\n' => '_',
            '=' => ':',
            other => other,
        })
        .collect()
}

const THREADS_ENVIRONMENT_VARIABLE: &str = "E185_THREADS";
const THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";
/// 每台默认重活线程数的登记原文：构建时编进来，运行时不找仓根（同 E184 装置的读法）。
const HEAVY_THREAD_REGISTRATION_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/",
    "../../.claude/heavy-thread-default"
));

/// 饱和吞吐的 N 线程那一档：`E185_THREADS` 设了照它，没设取 `SINGLEFS_THREAD_CAP`，再没设取登记值。设了而不是正整数就停下。
fn worker_threads_from_the_environment() -> usize {
    worker_threads_given(
        std::env::var(THREADS_ENVIRONMENT_VARIABLE).ok().as_deref(),
        std::env::var(THREAD_CAP_ENVIRONMENT_VARIABLE)
            .ok()
            .as_deref(),
        HEAVY_THREAD_REGISTRATION_TEXT,
    )
}

/// [`worker_threads_from_the_environment`] 的判法（`None` 是没设）。
///
/// # Panics
/// 设了的变量不是正整数；登记原文里没有「<正整数>」起头的非注释行。
fn worker_threads_given(
    own_value: Option<&str>,
    thread_cap_value: Option<&str>,
    registration_text: &str,
) -> usize {
    let positive_integer =
        |text: &str| text.trim().parse::<usize>().ok().filter(|count| *count > 0);
    if let Some(value) = own_value {
        return positive_integer(value).unwrap_or_else(|| {
            panic!("{THREADS_ENVIRONMENT_VARIABLE} 要是正整数，拿到「{value}」")
        });
    }
    if let Some(value) = thread_cap_value {
        return positive_integer(value).unwrap_or_else(|| {
            panic!("{THREAD_CAP_ENVIRONMENT_VARIABLE} 要是正整数，拿到「{value}」")
        });
    }
    registration_text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .and_then(|line| line.split_whitespace().next())
        .and_then(positive_integer)
        .expect("`.claude/heavy-thread-default` 第一条非注释行以正整数起头（那份文件的写法）")
}

/// 一串数的中位（偶数个取中间两个的平均）；空的交回 `None`。
fn median_of(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    Some(if sorted.len() % 2 == 1 {
        sorted[middle]
    } else {
        f64::midpoint(sorted[middle - 1], sorted[middle])
    })
}

/// 一串数里按升序第 ⌈percent·n ÷ 100⌉ 个（percent = 99 就是 p99）；空的交回 `None`。
fn percentile_of(values: &[f64], percent: usize) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = (percent * sorted.len())
        .div_ceil(100)
        .clamp(1, sorted.len());
    Some(sorted[rank - 1])
}

/// 5 轮（或几轮）的判定：全相同才交回那个判定，不全相同交回「不稳定」（`test-discipline.md`「单次观测不算数」；登记第十条 F4）。
fn judgement_of_the_rounds(judgements: &[&'static str]) -> &'static str {
    match judgements.first() {
        None => "not_run",
        Some(first) if judgements.iter().all(|judgement| judgement == first) => first,
        Some(_) => "unstable",
    }
}

/// 别的会话的性能测量（登记第十一节「机器被别的活挤」）：读 `/proc/<进程号>/cmdline`，按命令行里的名字认，只读、不动它们。
const MEASUREMENTS_THAT_HOLD_THE_ROUND: [&str; 4] = [
    "qemu-system",
    "vm-bench.sh",
    "e152-file-system-benchmark",
    "fio",
];
/// 等它们的步长与最多等几步（30 秒 × 120 = 一小时）。
const SECONDS_BETWEEN_LOOKS_AT_OTHER_MEASUREMENTS: u64 = 30;
const LOOKS_AT_OTHER_MEASUREMENTS_AT_MOST: u64 = 120;

/// 此刻在跑的别的性能测量（进程号、命中的名字）。`fio` 只认命令行第一段的文件名恰好是 `fio`，免得把路径里带这三个字母的都算进去。
fn other_measurements_running_now() -> Vec<(u32, &'static str)> {
    let own_process = std::process::id();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    let mut running = Vec::new();
    for entry in entries.flatten() {
        let Some(process) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        if process == own_process {
            continue;
        }
        let Ok(command_line) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        let words: Vec<String> = command_line
            .split(|byte| *byte == 0)
            .filter(|word| !word.is_empty())
            .map(|word| String::from_utf8_lossy(word).into_owned())
            .collect();
        for name in MEASUREMENTS_THAT_HOLD_THE_ROUND {
            let hits = if name == "fio" {
                words.first().is_some_and(|program| {
                    Path::new(program)
                        .file_name()
                        .is_some_and(|file| file == "fio")
                })
            } else {
                words.iter().any(|word| word.contains(name))
            };
            if hits {
                running.push((process, name));
            }
        }
    }
    running
}

/// 开轮之前等别的性能测量跑完；等满上限仍在跑就退 [`EXIT_CODE_OTHER_MEASUREMENT_STILL_RUNNING`]。
fn wait_until_no_other_measurement_runs(emitter: &mut Emitter) {
    for look in 0..LOOKS_AT_OTHER_MEASUREMENTS_AT_MOST {
        let running = other_measurements_running_now();
        if running.is_empty() {
            return;
        }
        eprintln!("在等别的性能测量：{running:?}（第 {look} 次看）");
        std::thread::sleep(Duration::from_secs(
            SECONDS_BETWEEN_LOOKS_AT_OTHER_MEASUREMENTS,
        ));
    }
    emitter.line("name=stopped reason=other_measurements_still_running_after_an_hour");
    emitter.finish();
    std::process::exit(EXIT_CODE_OTHER_MEASUREMENT_STILL_RUNNING);
}

// ───────────────────────────── 本线程的 CPU 时间（登记第一节「CPU 时间」，第十二节 12.2 修订） ─────────────────────────────

// `ThreadCpuClockReading` 照 Linux 64 位的 `struct timespec` 排布（`time_t` 与 `long` 都是 64 位）；别的平台排布不同，编不过比读错强。
#[cfg(not(all(target_os = "linux", target_pointer_width = "64")))]
compile_error!("E185 的线程 CPU 时钟只照 Linux 64 位的 struct timespec 排布写了");

/// Linux 的 `CLOCK_THREAD_CPUTIME_ID`（`<linux/time.h>` 里是 3）：本线程在 CPU 上跑过的时间，读的那一刻现算到纳秒（换钟的依据在登记第十二节 12.2）。
const CLOCK_IDENTIFIER_OF_THE_THREAD_CPU_TIME: i32 = 3;

const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;

/// `clock_gettime` 填的 `struct timespec`：整秒与秒内纳秒，两个 64 位有符号整数，次序就是声明的次序。
#[repr(C)]
struct ThreadCpuClockReading {
    whole_seconds: i64,
    nanoseconds_within_the_second: i64,
}

// std 已经链着 C 库，这里只声明、不加依赖；签名照 POSIX `int clock_gettime(clockid_t, struct timespec *)`，Linux 上 `clockid_t` 是 `int`。
extern "C" {
    fn clock_gettime(clock_id: i32, reading: *mut ThreadCpuClockReading) -> i32;
}

/// 本线程到此刻在 CPU 上跑过的纳秒；系统调用报错、读出负数的交回 `None`，调用方把那一轮作废（登记第十一节「读不到」）。
fn thread_cpu_nanoseconds() -> Option<u64> {
    let mut reading = ThreadCpuClockReading {
        whole_seconds: 0,
        nanoseconds_within_the_second: 0,
    };
    // SAFETY: `reading` 是这个栈帧里活着的变量，`#[repr(C)]` 照 `struct timespec` 排布（平台由文件里的 compile_error 守着）；
    // 指针只在这一次调用里用，内核只往这 16 字节里写；clock id 不合法时内核交回 −1、不写内存。
    let status = unsafe {
        clock_gettime(
            CLOCK_IDENTIFIER_OF_THE_THREAD_CPU_TIME,
            std::ptr::addr_of_mut!(reading),
        )
    };
    if status != 0 {
        return None;
    }
    let whole_seconds = u64::try_from(reading.whole_seconds).ok()?;
    let nanoseconds_within_the_second =
        u64::try_from(reading.nanoseconds_within_the_second).ok()?;
    whole_seconds
        .checked_mul(NANOSECONDS_PER_SECOND)?
        .checked_add(nanoseconds_within_the_second)
}

/// Linux 的 `CLOCK_MONOTONIC_RAW`（`<linux/time.h>` 里是 4）：与 CPU 读数配对的挂钟。不受 NTP 调速，`CLOCK_MONOTONIC`（`Instant`）
/// 比它慢约 7 ppm，长批上线程 CPU 读数会超过 `Instant` 量的挂钟、误触停机条款第 6 条（登记第十二节 12.4）。
const CLOCK_IDENTIFIER_OF_THE_RAW_MONOTONIC_WALL_CLOCK: i32 = 4;

/// 与线程 CPU 读数配对的一次挂钟读数（`CLOCK_MONOTONIC_RAW` 的纳秒）。
#[derive(Clone, Copy)]
struct RawWallClockReading(u64);

impl RawWallClockReading {
    fn now() -> Self {
        let mut reading = ThreadCpuClockReading {
            whole_seconds: 0,
            nanoseconds_within_the_second: 0,
        };
        // SAFETY: 同 `thread_cpu_nanoseconds`：`reading` 是这个栈帧里活着的变量、照 `struct timespec` 排布，指针只在这一次调用里用。
        let status = unsafe {
            clock_gettime(
                CLOCK_IDENTIFIER_OF_THE_RAW_MONOTONIC_WALL_CLOCK,
                std::ptr::addr_of_mut!(reading),
            )
        };
        assert_eq!(
            status, 0,
            "CLOCK_MONOTONIC_RAW 在 Linux 上恒可读；读不到说明平台不对"
        );
        let whole_seconds = u64::try_from(reading.whole_seconds).expect("单调钟的整秒不是负数");
        let nanoseconds_within_the_second =
            u64::try_from(reading.nanoseconds_within_the_second).expect("单调钟的秒内纳秒不是负数");
        RawWallClockReading(whole_seconds * NANOSECONDS_PER_SECOND + nanoseconds_within_the_second)
    }

    fn elapsed(self) -> Duration {
        Duration::from_nanos(Self::now().0.saturating_sub(self.0))
    }
}

/// 在本线程上自旋到线程 CPU 时钟多出 `nanoseconds`（第一行 CPU 臂的阳性对照）；读不到交回 `false`。
fn spin_until_the_thread_cpu_advances(nanoseconds: u64) -> bool {
    let Some(start) = thread_cpu_nanoseconds() else {
        return false;
    };
    // 迭代上界：自旋一直占着 CPU，线程 CPU 时钟每读一次都现算到那一刻，跑满 `nanoseconds` 的 CPU 就越过；读不到就停。
    loop {
        match thread_cpu_nanoseconds() {
            Some(now) if now >= start + nanoseconds => return true,
            Some(_) => std::hint::spin_loop(),
            None => return false,
        }
    }
}

/// 开跑自检（登记第十二节 12.2，收严）：自旋这么多纳秒（按被检的钟自己算）里，钟至少要前进 [`CLOCK_ADVANCES_WITHIN_THE_SPIN_AT_LEAST`] 次。
const CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS: u64 = 1_000_000;
/// 1 ms 里前进不到 100 次的钟分不出一次设备调用（几微秒到几十微秒）的 CPU，停机退 3。
const CLOCK_ADVANCES_WITHIN_THE_SPIN_AT_LEAST: u64 = 100;
/// 自检至多读几次钟：钟停着不走也要停下（停着不走 = 前进 0 次、没自旋满，判不够）。
const CLOCK_RESOLUTION_CHECK_READS_AT_MOST: u64 = 10_000_000;

/// 自检读数：读了几次、前进了几次（读数大过此前读到的最大值算一次）、自旋覆盖了钟上的多少纳秒。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClockResolutionReading {
    reads: u64,
    advances: u64,
    spanned_nanoseconds: u64,
}

/// 连读 `read_the_clock`，读到比第一读多出 [`CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS`] 就停（或读满上限），数它前进了几次；
/// 有一次读不到交回 `None`。钟从参数传进来，单测换成按 1 ms 前进的假钟。
fn clock_advances_while_spinning(
    mut read_the_clock: impl FnMut() -> Option<u64>,
) -> Option<ClockResolutionReading> {
    let start = read_the_clock()?;
    let mut largest_seen = start;
    let mut reading = ClockResolutionReading {
        reads: 1,
        advances: 0,
        spanned_nanoseconds: 0,
    };
    // 迭代上界 CLOCK_RESOLUTION_CHECK_READS_AT_MOST；跨轮携带的 largest_seen 只增不减，spanned_nanoseconds = largest_seen − start。
    while reading.reads < CLOCK_RESOLUTION_CHECK_READS_AT_MOST {
        let now = read_the_clock()?;
        reading.reads += 1;
        if now > largest_seen {
            reading.advances += 1;
            largest_seen = now;
        }
        reading.spanned_nanoseconds = largest_seen - start;
        if reading.spanned_nanoseconds >= CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS {
            break;
        }
    }
    Some(reading)
}

/// 自检判定：自旋满 1 ms、其间前进够 100 次才算过；读满上限还没自旋满（钟停着）同样不过。
fn is_the_clock_resolution_enough(reading: ClockResolutionReading) -> bool {
    reading.spanned_nanoseconds >= CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS
        && reading.advances >= CLOCK_ADVANCES_WITHIN_THE_SPIN_AT_LEAST
}

/// 开跑自检：在调用它的线程上连读线程 CPU 时钟（`wall-clock`、`stage-split` 里就是之后量 GC 的那条线程；`foreground-fsync` 的 GC
/// 在另起的线程上，自检的是同一种钟），打一行 `name=clock_resolution_check`；
/// 不过再打一行 `name=stop_clause`，交回 `false`，调用方停机退 3。
fn check_the_thread_cpu_clock_resolution(emitter: &mut Emitter) -> bool {
    let reading = clock_advances_while_spinning(thread_cpu_nanoseconds);
    let holds = reading.is_some_and(is_the_clock_resolution_enough);
    let field = |value_of: fn(ClockResolutionReading) -> u64| {
        reading.map_or_else(
            || "unreadable".to_string(),
            |measured| value_of(measured).to_string(),
        )
    };
    emitter.line(&format!(
        "name=clock_resolution_check clock=thread_cputime clock_identifier={CLOCK_IDENTIFIER_OF_THE_THREAD_CPU_TIME} reads={} advances={} spanned_nanoseconds={} spin_nanoseconds={CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS} advances_at_least={CLOCK_ADVANCES_WITHIN_THE_SPIN_AT_LEAST} holds={holds}",
        field(|measured| measured.reads),
        field(|measured| measured.advances),
        field(|measured| measured.spanned_nanoseconds),
    ));
    if !holds {
        emitter.line("name=stop_clause clause=clock_resolution reason=thread_cpu_clock_advanced_fewer_times_than_required_within_the_spin_or_was_unreadable");
    }
    holds
}

/// 自检没过：这一段一个取样点都不量，打判决行、退 3。
fn stop_for_the_clock_resolution(emitter: &mut Emitter, segment: &str) -> i32 {
    emitter.line(&format!(
        "name=verdict segment={segment} clock_resolution_holds=false stop_clause_findings=1"
    ));
    emitter.finish();
    EXIT_CODE_STOP_CLAUSE
}

/// 按挂钟忙等 `duration`（第二行阶段计时器的 2 ms 对照：单线程忙等，挂钟就是 CPU 时间）。
fn busy_wait_for(duration: Duration) {
    let started = Instant::now();
    while started.elapsed() < duration {
        std::hint::spin_loop();
    }
}

// ───────────────────────────── 计时包装设备（登记第五节「计时与计数怎么做」） ─────────────────────────────

/// 包装转给里面那块盘的每一种调用。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DeviceMethod {
    Read,
    Write,
    WriteZeroes,
    Barrier,
    Discard,
    RereadAfterABadRead,
    ReadStraightFromTheDisk,
    KeepTheSelfVerifiedReread,
    NoteTheReadFailingSelfVerification,
    JournalRecordOffsetsHint,
}

impl DeviceMethod {
    const ALL: [DeviceMethod; 10] = [
        DeviceMethod::Read,
        DeviceMethod::Write,
        DeviceMethod::WriteZeroes,
        DeviceMethod::Barrier,
        DeviceMethod::Discard,
        DeviceMethod::RereadAfterABadRead,
        DeviceMethod::ReadStraightFromTheDisk,
        DeviceMethod::KeepTheSelfVerifiedReread,
        DeviceMethod::NoteTheReadFailingSelfVerification,
        DeviceMethod::JournalRecordOffsetsHint,
    ];

    const fn label(self) -> &'static str {
        match self {
            DeviceMethod::Read => "read",
            DeviceMethod::Write => "write",
            DeviceMethod::WriteZeroes => "write_zeroes",
            DeviceMethod::Barrier => "barrier",
            DeviceMethod::Discard => "discard",
            DeviceMethod::RereadAfterABadRead => "reread_after_a_bad_read",
            DeviceMethod::ReadStraightFromTheDisk => "read_straight_from_the_disk",
            DeviceMethod::KeepTheSelfVerifiedReread => "keep_the_self_verified_reread",
            DeviceMethod::NoteTheReadFailingSelfVerification => {
                "note_the_read_failing_self_verification"
            }
            DeviceMethod::JournalRecordOffsetsHint => "journal_record_offsets_hint",
        }
    }
}

/// 一种调用在一个计时窗口里的累计：次数、字节、挂钟、这期间本线程的 CPU、等盘（挂钟减 CPU，可以是负的：设备调用这一层 CPU 读数包着挂钟读数，纯 CPU 的调用里 CPU 多出读钟的工夫）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MethodTotals {
    calls: u64,
    bytes: u64,
    wall_nanoseconds: u64,
    cpu_nanoseconds: u64,
    wait_nanoseconds: i64,
}

/// 设备调用里的等盘时间：这次调用的挂钟减去同一调用里本线程 CPU 时间的增量（登记第一节「等盘读写时间」）。
fn device_wait_nanoseconds(wall_nanoseconds: u64, cpu_nanoseconds: u64) -> i64 {
    i64::try_from(wall_nanoseconds).expect("一次设备调用的挂钟装得进 i64")
        - i64::try_from(cpu_nanoseconds).expect("一次设备调用的 CPU 装得进 i64")
}

/// 记下的一次读：哪块盘、偏移、长度、是不是这个窗口里第一次写之前读的。第一次写之前的读有两样落在 R 的槽上：区域扫描对 R 那块盘
/// 每个未释放单元读一遍，随后发布在 `publish_admitted` 之前对这一批每个搬迁源按两条位置项各读一遍做释放前读盘核
/// （`transaction.rs` 里调 `copies_of_a_released_unit_failing_the_release_checksum_check` 的那一圈）；发布写下之后的读与 discard 的读
/// 在第一次写之后。两样怎么分开见 [`split_the_reads_of_a_call`]（登记第十二节 12.5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecordedRead {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
    is_before_the_first_write: bool,
}

/// 记下的一次写（`write_at`）：哪块盘、偏移、长度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecordedWrite {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
}

/// 阳性对照往每次 `write_at` 里加的东西（登记第五节阳性对照表第 1 行）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteInjection {
    Nothing,
    SleepTwoMilliseconds,
    SpinTwoMillisecondsOfThreadCpu,
}

impl WriteInjection {
    const fn label(self) -> &'static str {
        match self {
            WriteInjection::Nothing => "none",
            WriteInjection::SleepTwoMilliseconds => "sleep_2_milliseconds_per_write",
            WriteInjection::SpinTwoMillisecondsOfThreadCpu => "spin_2_milliseconds_per_write",
        }
    }
}

/// 一组盘共用的账（两块盘一份）：一个计时窗口里每种调用的累计、读写的落点；另带跨窗口的两样——改过的 4 MiB 块（真盘臂下一轮只拷回这些块）
/// 与阳性对照的注入。窗口由调用方在一次 `run_compaction` 之前清（[`CallLedger::open_a_new_window`]）。
#[derive(Debug)]
struct CallLedger {
    totals: BTreeMap<DeviceMethod, MethodTotals>,
    reads: Vec<RecordedRead>,
    writes: Vec<RecordedWrite>,
    has_a_write_in_this_window: bool,
    is_the_cpu_unreadable_in_this_window: bool,
    chunks_written_since_the_copy: BTreeSet<(DeviceIdentity, u64)>,
    injection: WriteInjection,
}

/// 真盘臂拷回与算摘要的块大小。
const IMAGE_CHUNK_BYTES: u64 = 4 << 20;

impl CallLedger {
    fn new(injection: WriteInjection) -> CallLedger {
        CallLedger {
            totals: BTreeMap::new(),
            reads: Vec::new(),
            writes: Vec::new(),
            has_a_write_in_this_window: false,
            is_the_cpu_unreadable_in_this_window: false,
            chunks_written_since_the_copy: BTreeSet::new(),
            injection,
        }
    }

    fn open_a_new_window(&mut self) {
        self.totals.clear();
        self.reads.clear();
        self.writes.clear();
        self.has_a_write_in_this_window = false;
        self.is_the_cpu_unreadable_in_this_window = false;
    }

    fn note_the_call(
        &mut self,
        method: DeviceMethod,
        bytes: u64,
        wall: Duration,
        cpu_before: Option<u64>,
        cpu_after: Option<u64>,
    ) {
        let wall_nanoseconds =
            u64::try_from(wall.as_nanos()).expect("一次设备调用的挂钟装得进 u64");
        let cpu_nanoseconds = match (cpu_before, cpu_after) {
            (Some(before), Some(after)) => after.saturating_sub(before),
            (None, _) | (_, None) => {
                self.is_the_cpu_unreadable_in_this_window = true;
                0
            }
        };
        let totals = self.totals.entry(method).or_default();
        totals.calls += 1;
        totals.bytes += bytes;
        totals.wall_nanoseconds += wall_nanoseconds;
        totals.cpu_nanoseconds += cpu_nanoseconds;
        totals.wait_nanoseconds += device_wait_nanoseconds(wall_nanoseconds, cpu_nanoseconds);
    }

    fn note_the_changed_range(&mut self, device: DeviceIdentity, offset: u64, length: u64) {
        if length == 0 {
            return;
        }
        for chunk in offset / IMAGE_CHUNK_BYTES..=(offset + length - 1) / IMAGE_CHUNK_BYTES {
            self.chunks_written_since_the_copy.insert((device, chunk));
        }
    }

    #[cfg(test)]
    fn totals_of(&self, method: DeviceMethod) -> MethodTotals {
        self.totals.get(&method).copied().unwrap_or_default()
    }

    /// 这个窗口里全部设备调用的累计（等盘就是各调用等盘之和）。
    fn totals_of_every_method(&self) -> MethodTotals {
        let mut sum = MethodTotals::default();
        for totals in self.totals.values() {
            sum.calls += totals.calls;
            sum.bytes += totals.bytes;
            sum.wall_nanoseconds += totals.wall_nanoseconds;
            sum.cpu_nanoseconds += totals.cpu_nanoseconds;
            sum.wait_nanoseconds += totals.wait_nanoseconds;
        }
        sum
    }
}

/// 真盘臂的 journal 记录偏移提示：可写挂载那一段里把单元区之前（系统配置、根环、journal 环）的写照抄进一块内存盘，挂载照它答哪几个记录槽
/// 可能有字节（同内存盘的答法 `SparseBlockDevice::journal_record_offsets_hint`），不在 `O_DIRECT` 镜像上把 768 MiB 的环逐槽读一遍。
/// 只在挂载里照抄（`is_mirroring`）；计时窗口里一个字节都不抄，那时再要提示就转给真盘（答 `None`、全环扫描）。
struct JournalHintMirror {
    image: SparseBlockDevice,
    below_offset: u64,
    is_mirroring: bool,
}

/// 计时包装：每一种调用都转给里面那块盘，前后各读一次 `Instant` 与本线程的线程 CPU 时钟，记进共用的账。
struct TimedBlockDevice<Inner: BlockDevice> {
    inner: Inner,
    identity: DeviceIdentity,
    ledger: Rc<RefCell<CallLedger>>,
    journal_hint_mirror: Option<JournalHintMirror>,
}

/// 计一次设备调用：这一层 CPU 读数包着挂钟读数，一次 `run_compaction` 调用那一层反过来挂钟读数包着 CPU 读数（`measure_one_intent`）。
/// 两个钟走得一样快时这样嵌套，`CPU + 等盘 ≤ 挂钟` 不会因为读钟的先后被打破：等盘 = 设备挂钟 − 设备 CPU 少算了设备那一层读钟的工夫，
/// 调用那一层的挂钟多算了这一层读钟的工夫；包含自检反过来（停机条款第 6 条）只剩计时口径真错了这一种来路。
fn timed_call<Answer>(
    ledger: &RefCell<CallLedger>,
    method: DeviceMethod,
    bytes: u64,
    call: impl FnOnce() -> Answer,
) -> Answer {
    let cpu_before = thread_cpu_nanoseconds();
    let started = RawWallClockReading::now();
    let answer = call();
    let wall = started.elapsed();
    let cpu_after = thread_cpu_nanoseconds();
    ledger
        .borrow_mut()
        .note_the_call(method, bytes, wall, cpu_before, cpu_after);
    answer
}

fn length_in_bytes(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).expect("一次设备调用的长度装得进 u64")
}

impl<Inner: BlockDevice> TimedBlockDevice<Inner> {
    fn new(inner: Inner, identity: DeviceIdentity, ledger: &Rc<RefCell<CallLedger>>) -> Self {
        TimedBlockDevice {
            inner,
            identity,
            ledger: Rc::clone(ledger),
            journal_hint_mirror: None,
        }
    }

    fn note_the_read(&self, offset: DeviceOffsetInBytes, length: u64) {
        let mut ledger = self.ledger.borrow_mut();
        let is_before_the_first_write = !ledger.has_a_write_in_this_window;
        ledger.reads.push(RecordedRead {
            device: self.identity,
            offset: offset.0,
            length,
            is_before_the_first_write,
        });
    }

    fn stop_mirroring_the_journal_writes(&mut self) {
        if let Some(mirror) = self.journal_hint_mirror.as_mut() {
            mirror.is_mirroring = false;
        }
    }
}

impl<Inner: BlockDevice> BlockDevice for TimedBlockDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = length_in_bytes(buffer);
        let answer = timed_call(&self.ledger, DeviceMethod::Read, length, || {
            self.inner.read_at(offset, buffer)
        });
        self.note_the_read(offset, length);
        answer
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = length_in_bytes(bytes);
        let injection = self.ledger.borrow().injection;
        let inner = &mut self.inner;
        let answer = timed_call(&self.ledger, DeviceMethod::Write, length, || {
            let written = inner.write_at(offset, bytes, durability);
            match injection {
                WriteInjection::Nothing => {}
                WriteInjection::SleepTwoMilliseconds => {
                    std::thread::sleep(Duration::from_millis(INJECTED_MILLISECONDS_PER_WRITE));
                }
                WriteInjection::SpinTwoMillisecondsOfThreadCpu => {
                    let spun = spin_until_the_thread_cpu_advances(
                        INJECTED_MILLISECONDS_PER_WRITE * 1_000_000,
                    );
                    assert!(
                        spun,
                        "线程 CPU 时钟读不到：CPU 臂的阳性对照做不了（读不到不当 0）"
                    );
                }
            }
            written
        });
        if let Some(mirror) = self.journal_hint_mirror.as_mut() {
            if mirror.is_mirroring && offset.0 < mirror.below_offset {
                mirror
                    .image
                    .write_at(offset, bytes, durability)
                    .expect("照抄进内存盘：偏移与长度已由真盘核过");
            }
        }
        let mut ledger = self.ledger.borrow_mut();
        ledger.has_a_write_in_this_window = true;
        ledger.writes.push(RecordedWrite {
            device: self.identity,
            offset: offset.0,
            length,
        });
        ledger.note_the_changed_range(self.identity, offset.0, length);
        answer
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let inner = &mut self.inner;
        let answer = timed_call(&self.ledger, DeviceMethod::WriteZeroes, length, || {
            inner.write_zeroes_at(offset, length)
        });
        if let Some(mirror) = self.journal_hint_mirror.as_mut() {
            if mirror.is_mirroring && offset.0 < mirror.below_offset {
                mirror
                    .image
                    .write_zeroes_at(offset, length)
                    .expect("照抄进内存盘：偏移与长度已由真盘核过");
            }
        }
        let mut ledger = self.ledger.borrow_mut();
        ledger.has_a_write_in_this_window = true;
        ledger.note_the_changed_range(self.identity, offset.0, length);
        answer
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        let inner = &mut self.inner;
        let answer = timed_call(&self.ledger, DeviceMethod::Discard, length, || {
            inner.discard_at(offset, length)
        });
        if matches!(answer, Ok(DiscardOutcome::Applied)) {
            self.ledger
                .borrow_mut()
                .note_the_changed_range(self.identity, offset.0, length);
        }
        answer
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let inner = &mut self.inner;
        timed_call(&self.ledger, DeviceMethod::Barrier, 0, || inner.barrier())
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
        timed_call(
            &self.ledger,
            DeviceMethod::JournalRecordOffsetsHint,
            0,
            || match &self.journal_hint_mirror {
                Some(mirror) if mirror.is_mirroring => mirror
                    .image
                    .journal_record_offsets_hint(ring_start, ring_bytes),
                Some(_) | None => self
                    .inner
                    .journal_record_offsets_hint(ring_start, ring_bytes),
            },
        )
    }

    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = length_in_bytes(buffer);
        let answer = timed_call(
            &self.ledger,
            DeviceMethod::RereadAfterABadRead,
            length,
            || self.inner.reread_at_after_a_bad_read(offset, buffer),
        );
        self.note_the_read(offset, length);
        answer
    }

    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        timed_call(
            &self.ledger,
            DeviceMethod::KeepTheSelfVerifiedReread,
            length_in_bytes(bytes),
            || self.inner.keep_the_self_verified_reread(offset, bytes),
        );
    }

    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes_of_the_read: u64,
    ) {
        timed_call(
            &self.ledger,
            DeviceMethod::NoteTheReadFailingSelfVerification,
            length_in_bytes_of_the_read,
            || {
                self.inner
                    .note_the_read_failing_self_verification_and_forget_its_bytes(
                        offset,
                        length_in_bytes_of_the_read,
                    );
            },
        );
    }

    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = length_in_bytes(buffer);
        let answer = timed_call(
            &self.ledger,
            DeviceMethod::ReadStraightFromTheDisk,
            length,
            || self.inner.read_at_straight_from_the_disk(offset, buffer),
        );
        self.note_the_read(offset, length);
        answer
    }
}

/// 一组盘套上计时包装，共用一份账。
fn timed_devices<Inner: BlockDevice>(
    devices: Vec<(DeviceIdentity, Inner)>,
    ledger: &Rc<RefCell<CallLedger>>,
) -> Vec<(DeviceIdentity, TimedBlockDevice<Inner>)> {
    devices
        .into_iter()
        .map(|(identity, inner)| (identity, TimedBlockDevice::new(inner, identity, ledger)))
        .collect()
}

// ───────────────────────────── 一次调用是哪一种、第七节乙类锚点 ─────────────────────────────

/// 一次 `run_compaction` 调用（发布预算压到 1）发成了什么（登记第一节「一批」那一行的口径）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallKind {
    /// 签发：只把意图写进 logged_ops，不搬单元；单列，不算批。
    Issue,
    /// 一批（意图还在）。
    Batch,
    /// 最后一批：同一次发布删意图。
    FinalBatch,
    /// 删意图、丢意图而一个没搬：不算批。
    IntentDeletedWithoutABatch,
    /// 一次都没发（停了）。
    NoPublish,
}

impl CallKind {
    const fn label(self) -> &'static str {
        match self {
            CallKind::Issue => "issue",
            CallKind::Batch => "batch",
            CallKind::FinalBatch => "final_batch",
            CallKind::IntentDeletedWithoutABatch => "intent_deleted_without_a_batch",
            CallKind::NoPublish => "no_publish",
        }
    }
}

/// 按发成的那一版判：logged_ops 还有没有意图、这一次搬了几个单元（同 harness 整理流前缀的判法）。
fn kind_of_the_call(publishes: usize, intent_remains: bool, units_relocated: usize) -> CallKind {
    match (publishes, intent_remains, units_relocated > 0) {
        (0, _, _) => CallKind::NoPublish,
        (_, true, false) => CallKind::Issue,
        (_, true, true) => CallKind::Batch,
        (_, false, true) => CallKind::FinalBatch,
        (_, false, false) => CallKind::IntentDeletedWithoutABatch,
    }
}

/// 一批 = 搬了单元的那一次（会话 `observations.batches_published` 同一口径）；签发不算。
fn counts_as_a_batch(kind: CallKind) -> bool {
    match kind {
        CallKind::Batch | CallKind::FinalBatch => true,
        CallKind::Issue | CallKind::IntentDeletedWithoutABatch | CallKind::NoPublish => false,
    }
}

/// 两盘同槽镜像：一个单元搬一次写两份。
const DEVICES_HOLDING_A_COPY: u64 = 2;

/// R 一块盘的字节：64 槽 × 16384（锚点 1048576）。
const fn region_bytes_per_device() -> u64 {
    LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_SLOT_BYTES
}

/// 签发那一步扫被选中的 R 读到的字节：m 个数据单元（f = 0.25 → 262144，f = 0.75 → 786432）。
const fn issue_scan_bytes(data_units: u64) -> u64 {
    data_units * LOCAL_DATA_UNIT_BYTES
}

/// 一条意图的批数 ⌈m ÷ K⌉。
const fn batches_of_one_intent(data_units: u64, batch_units: u64) -> u64 {
    data_units.div_ceil(batch_units)
}

/// 一条意图签发加各批扫描读到的字节之和：每批之前对整个 R 重扫，第 j 批扫 m − jK 个单元。
fn intent_scan_bytes(data_units: u64, batch_units: u64) -> u64 {
    let batch_scan_units: u64 = (0..batches_of_one_intent(data_units, batch_units))
        .map(|batch_index| data_units - batch_index * batch_units)
        .sum();
    (data_units + batch_scan_units) * LOCAL_DATA_UNIT_BYTES
}

/// 第一批写到这一批目的地的搬迁副本字节（两块盘）：2 × min(K, m) × 32768。
const fn first_batch_copy_bytes(data_units: u64, batch_units: u64) -> u64 {
    let first_batch_units = if batch_units < data_units {
        batch_units
    } else {
        data_units
    };
    DEVICES_HOLDING_A_COPY * first_batch_units * LOCAL_DATA_UNIT_BYTES
}

/// 一条意图的搬迁副本字节之和：2 × m × 32768。
const fn intent_copy_bytes(data_units: u64) -> u64 {
    DEVICES_HOLDING_A_COPY * data_units * LOCAL_DATA_UNIT_BYTES
}

/// 这一批扫描之前现行那一版 R 那块盘上未释放记录的 Σ 跨度 × 16384（停机条款第 2 条的右边）。
fn expected_scan_read_bytes(records: &[AllocationRecord], region: CompactionRegion) -> u64 {
    records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && region.contains_slot(record.slot)
        })
        .map(|record| u64::from(record.span_slots) * LOCAL_SLOT_BYTES)
        .sum()
}

/// Q4 的点对 (K, 2K)：只取 2K ≤ m 的（登记第八节：f = 0.25 时 8→16 两边第一批都是 8 个单元，是取样点不敏感的一对）。
fn doubling_pairs_within(data_units: u64) -> Vec<(u64, u64)> {
    BATCH_SIZES
        .iter()
        .flat_map(|smaller| {
            BATCH_SIZES
                .iter()
                .filter(move |larger| **larger == 2 * *smaller)
                .map(move |larger| (*smaller, *larger))
        })
        .filter(|(_smaller, larger)| *larger <= data_units)
        .collect()
}

/// 一批写到目的地的字节：每次 `write_at` 与这一批每个目的单元（盘、起点偏移、长度）重叠的字节之和；另交回每个目的单元是否被写满。
fn relocation_copy_bytes(
    writes: &[RecordedWrite],
    destinations: &[(DeviceIdentity, u64, u64)],
) -> (u64, bool) {
    let mut copied = 0_u64;
    let mut is_every_destination_written_whole = true;
    for (device, start, length) in destinations {
        let end = start + length;
        let covered: u64 = writes
            .iter()
            .filter(|write| write.device == *device)
            .map(|write| {
                let overlap_start = write.offset.max(*start);
                let overlap_end = (write.offset + write.length).min(end);
                overlap_end.saturating_sub(overlap_start)
            })
            .sum();
        copied += covered;
        if covered != *length {
            is_every_destination_written_whole = false;
        }
    }
    (copied, is_every_destination_written_whole)
}

// ───────────────────────────── 包含自检与各量的判定（登记第六节） ─────────────────────────────

/// 包含自检（登记第五节）：一批的挂钟要包住 CPU + 等盘。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InclusionVerdict {
    Holds,
    /// 挂钟 − CPU − 等盘超过挂钟 10%：那一轮作废重跑（机器被别的活挤）。
    UnaccountedAboveTenPercent,
    /// CPU + 等盘大过挂钟：计时口径有错（停机条款第 6 条）。
    CpuPlusWaitAboveTheWall,
}

impl InclusionVerdict {
    const fn label(self) -> &'static str {
        match self {
            InclusionVerdict::Holds => "holds",
            InclusionVerdict::UnaccountedAboveTenPercent => "unaccounted_above_ten_percent",
            InclusionVerdict::CpuPlusWaitAboveTheWall => "cpu_plus_wait_above_the_wall",
        }
    }
}

fn inclusion_verdict(
    wall_nanoseconds: u64,
    cpu_nanoseconds: u64,
    wait_nanoseconds: i64,
) -> InclusionVerdict {
    let accounted = i128::from(cpu_nanoseconds) + i128::from(wait_nanoseconds);
    let wall = i128::from(wall_nanoseconds);
    if accounted > wall {
        return InclusionVerdict::CpuPlusWaitAboveTheWall;
    }
    let unaccounted = (wall - accounted) as f64;
    if unaccounted > UNACCOUNTED_SHARE_ABOVE_FOR_INVALID * wall as f64 {
        InclusionVerdict::UnaccountedAboveTenPercent
    } else {
        InclusionVerdict::Holds
    }
}

/// Q1 的判法（门槛由调用方给：判别力自证把门槛挪到两点之间再判一次）。
fn judgement_of_rho(rho: f64, at_most_for_disk: f64, at_least_for_cpu: f64) -> &'static str {
    if rho <= at_most_for_disk {
        "disk"
    } else if rho >= at_least_for_cpu {
        "cpu"
    } else {
        "both"
    }
}

/// Q1：一批（或一条意图）的 CPU 时间 ÷ 等盘时间，按登记的两条边判。
fn judgement_of_rho_at_the_registered_edges(rho: f64) -> &'static str {
    judgement_of_rho(rho, RHO_AT_MOST_FOR_DISK, RHO_AT_LEAST_FOR_CPU)
}

/// Q2 的判法：真盘 ÷ 内存的挂钟比。
fn judgement_of_wall_ratio(
    ratio: f64,
    at_least_for_disk: f64,
    at_most_for_cpu: f64,
) -> &'static str {
    if ratio >= at_least_for_disk {
        "disk"
    } else if ratio <= at_most_for_cpu {
        "cpu"
    } else {
        "both"
    }
}

fn judgement_of_wall_ratio_at_the_registered_edges(ratio: f64) -> &'static str {
    judgement_of_wall_ratio(
        ratio,
        WALL_RATIO_AT_LEAST_FOR_DISK,
        WALL_RATIO_AT_MOST_FOR_CPU,
    )
}

/// Q3：|CPU_真盘 − 挂钟_内存| ÷ 挂钟_内存。
fn cpu_reconciliation(cpu_on_the_real_disk: f64, wall_on_the_memory_device: f64) -> f64 {
    (cpu_on_the_real_disk - wall_on_the_memory_device).abs() / wall_on_the_memory_device
}

fn judgement_of_the_cpu_reconciliation(reconciliation: f64) -> &'static str {
    if reconciliation <= CPU_RECONCILIATION_AT_MOST {
        "compatible"
    } else {
        "paths_disagree"
    }
}

/// Q4：每轮 `挂钟(2K) ÷ 挂钟(K)`；5 轮中位 > 1 且每轮都 > 1 判「跟着涨」。
fn judgement_of_the_batch_doubling(ratios_per_round: &[f64]) -> &'static str {
    let every_round_rises =
        !ratios_per_round.is_empty() && ratios_per_round.iter().all(|ratio| *ratio > 1.0);
    match median_of(ratios_per_round) {
        Some(median) if median > 1.0 && every_round_rises => "rises",
        Some(_) => "does_not_rise",
        None => "not_run",
    }
}

/// Q5–Q7：份额 ≥ 10% 判「值得」。
fn judgement_of_share(share: f64, at_least_for_worth: f64) -> &'static str {
    if share >= at_least_for_worth {
        "worth"
    } else {
        "not_worth"
    }
}

/// Q9：D24 形式的比 < 1 判「这一段在 GPU 上输」。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
fn judgement_of_the_sixteen_thread_to_gpu_ratio(ratio: f64) -> &'static str {
    if ratio < SIXTEEN_THREAD_TO_GPU_RATIO_BELOW_FOR_GPU_LOSES {
        "gpu_loses"
    } else {
        "gpu_has_a_chance"
    }
}

/// Q13 的判法：φ。
fn judgement_of_fixed_share(
    share: f64,
    at_most_for_one_batch: f64,
    at_least_for_merging: f64,
) -> &'static str {
    if share <= at_most_for_one_batch {
        "one_batch_per_submission"
    } else if share >= at_least_for_merging {
        "merge_batches"
    } else {
        "merging_optional"
    }
}

#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
fn judgement_of_fixed_share_at_the_registered_edges(share: f64) -> &'static str {
    judgement_of_fixed_share(
        share,
        FIXED_SHARE_AT_MOST_FOR_ONE_BATCH,
        FIXED_SHARE_AT_LEAST_FOR_MERGING,
    )
}

/// Q16 的判法：GC 单跑写带宽 ÷ 设备顺序写带宽。
fn judgement_of_bandwidth_ratio(ratio: f64, at_most_for_single_run: f64) -> &'static str {
    if ratio <= at_most_for_single_run {
        "single_run_enough"
    } else {
        "needs_the_concurrent_measurement"
    }
}

fn judgement_of_bandwidth_ratio_at_the_registered_edge(ratio: f64) -> &'static str {
    judgement_of_bandwidth_ratio(ratio, BANDWIDTH_RATIO_AT_MOST_FOR_SINGLE_RUN)
}

/// Q17：并发时前台 p99 ÷ 不跑 GC 时的 p99。
fn judgement_of_the_foreground_fsync_latency_ratio(ratio: f64) -> &'static str {
    if ratio >= FOREGROUND_PERCENTILE_99_RATIO_AT_LEAST_FOR_DRAG {
        "gc_drags_the_foreground"
    } else {
        "gc_drags_little"
    }
}

/// 判别力自证（登记第八节）：门槛挪到量到的两点之间，两点的判定必须不同。交回（挪到的门槛、低点判定、高点判定）。
fn judgements_with_the_threshold_moved_between(
    lower_value: f64,
    higher_value: f64,
    judge: impl Fn(f64, f64) -> &'static str,
) -> (f64, &'static str, &'static str) {
    let moved = f64::midpoint(lower_value, higher_value);
    (moved, judge(lower_value, moved), judge(higher_value, moved))
}

/// 相对变化 (对照 − 基线) ÷ 基线。
fn relative_change(control: f64, baseline: f64) -> f64 {
    (control - baseline) / baseline
}

/// 第一行盘臂对照：等盘增量 ÷（写调用次数 × 2 ms）落在 [1.0, 1.5]，CPU 变化在 ±10% 以内。
fn disk_arm_control_holds(
    wait_increment_nanoseconds: f64,
    write_calls: f64,
    cpu_change: f64,
) -> bool {
    let injected = write_calls * (INJECTED_MILLISECONDS_PER_WRITE * 1_000_000) as f64;
    let ratio = wait_increment_nanoseconds / injected;
    (DISK_CONTROL_RATIO_LOWER..=DISK_CONTROL_RATIO_UPPER).contains(&ratio)
        && cpu_change.abs() <= CONTROL_TOLERANCE
}

/// 第一行 CPU 臂对照 CPU 那一句：CPU 增量落在写调用次数 × 2 ms 的 ±10% 里（登记第五节原文，没改）。
fn cpu_arm_cpu_clause_holds(cpu_increment_nanoseconds: f64, injected_nanoseconds: f64) -> bool {
    relative_change(cpu_increment_nanoseconds, injected_nanoseconds).abs() <= CONTROL_TOLERANCE
}

/// 第一行 CPU 臂对照等盘那一句（登记第十二节 12.9 的新判法）：写调用的等盘增量的绝对值 ≤ 写调用次数 × 2 ms × 10%。
/// 只算注入所在的写调用，分母是注入量：内存盘上的基线等盘是读钟嵌套留下的负的小数，拿它当分母分不开装置正确与 V1（等盘不减 CPU）。
fn cpu_arm_write_wait_clause_holds(
    write_wait_increment_nanoseconds: f64,
    injected_nanoseconds: f64,
) -> bool {
    write_wait_increment_nanoseconds.abs() <= injected_nanoseconds * CONTROL_TOLERANCE
}

/// 第一行 CPU 臂对照等盘那一句的原判据（登记第五节原文「等盘时间的变化在 ±10% 以内」）：交错跑的那一趟起照算照报，不作判定用（12.9）。
fn cpu_arm_original_wait_clause_holds(wait_change: f64) -> bool {
    wait_change.abs() <= CONTROL_TOLERANCE
}

/// 第一行阳性对照一格里一轮整条意图的读数（从 [`StoredIntent`] 取；单测直接喂基线先跑完、对照后跑的那一趟产物里 `name=call` 行按轮加出来的数）。
#[derive(Clone, Copy, Debug, PartialEq)]
struct RowOneControlRound {
    round: usize,
    cpu_nanoseconds: f64,
    wait_nanoseconds: f64,
    write_calls: f64,
    write_wait_nanoseconds: f64,
}

/// 第一行阳性对照一格的判定。盘臂那一行没改（登记第五节原文，一个合取）；CPU 臂两句各自报，原判据那一句只报不判（12.9）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RowOneControlJudgement {
    DiskArm {
        holds: bool,
    },
    CpuArm {
        cpu_clause_holds: bool,
        write_wait_clause_holds: bool,
        wait_clause_under_the_original_criterion_holds: bool,
    },
}

impl RowOneControlJudgement {
    const fn holds(self) -> bool {
        match self {
            RowOneControlJudgement::DiskArm { holds } => holds,
            RowOneControlJudgement::CpuArm {
                cpu_clause_holds,
                write_wait_clause_holds,
                wait_clause_under_the_original_criterion_holds: _,
            } => cpu_clause_holds && write_wait_clause_holds,
        }
    }
}

/// 第一行阳性对照一格（设备 × 注入）的读数与判定。增量一律取「对照各轮的中位 − 基线各轮的中位」（与基线先跑完、对照后跑的那一趟取法相同，没改）；
/// 按轮号配对的增量中位另报，不作判定用（12.9）。
#[derive(Clone, Copy, Debug, PartialEq)]
struct RowOneControlReading {
    write_calls: f64,
    injected_nanoseconds: f64,
    cpu_increment_nanoseconds: f64,
    wait_increment_nanoseconds: f64,
    write_wait_increment_nanoseconds: f64,
    cpu_change: f64,
    wait_change: f64,
    paired_rounds: usize,
    cpu_increment_paired_median_nanoseconds: Option<f64>,
    write_wait_increment_paired_median_nanoseconds: Option<f64>,
    judgement: RowOneControlJudgement,
}

/// 同一轮号的「对照 − 基线」之差（只取两边都有的轮号，按对照的次序）。
fn paired_round_differences(
    baseline: &[RowOneControlRound],
    control: &[RowOneControlRound],
    value_of: fn(&RowOneControlRound) -> f64,
) -> Vec<f64> {
    control
        .iter()
        .filter_map(|control_round| {
            baseline
                .iter()
                .find(|baseline_round| baseline_round.round == control_round.round)
                .map(|baseline_round| value_of(control_round) - value_of(baseline_round))
        })
        .collect()
}

/// 判一格阳性对照。基线或对照一轮都没有交回 `None`（那一格记 `not_run`）；注入是 `Nothing` 也交回 `None`（基线对基线不是对照）。
fn row_one_control_reading(
    injection: WriteInjection,
    baseline: &[RowOneControlRound],
    control: &[RowOneControlRound],
) -> Option<RowOneControlReading> {
    let median_over = |rounds: &[RowOneControlRound], value_of: fn(&RowOneControlRound) -> f64| {
        median_of(&rounds.iter().map(value_of).collect::<Vec<_>>())
    };
    let baseline_cpu = median_over(baseline, |round| round.cpu_nanoseconds)?;
    let control_cpu = median_over(control, |round| round.cpu_nanoseconds)?;
    let baseline_wait = median_over(baseline, |round| round.wait_nanoseconds)?;
    let control_wait = median_over(control, |round| round.wait_nanoseconds)?;
    let baseline_write_wait = median_over(baseline, |round| round.write_wait_nanoseconds)?;
    let control_write_wait = median_over(control, |round| round.write_wait_nanoseconds)?;
    let write_calls = median_over(control, |round| round.write_calls)?;
    let injected_nanoseconds = write_calls * (INJECTED_MILLISECONDS_PER_WRITE * 1_000_000) as f64;
    let cpu_increment_nanoseconds = control_cpu - baseline_cpu;
    let wait_increment_nanoseconds = control_wait - baseline_wait;
    let write_wait_increment_nanoseconds = control_write_wait - baseline_write_wait;
    let cpu_change = relative_change(control_cpu, baseline_cpu);
    let wait_change = relative_change(control_wait, baseline_wait);
    let judgement = match injection {
        WriteInjection::SleepTwoMilliseconds => RowOneControlJudgement::DiskArm {
            holds: disk_arm_control_holds(wait_increment_nanoseconds, write_calls, cpu_change),
        },
        WriteInjection::SpinTwoMillisecondsOfThreadCpu => {
            let cpu_clause_holds =
                cpu_arm_cpu_clause_holds(cpu_increment_nanoseconds, injected_nanoseconds);
            let write_wait_clause_holds = cpu_arm_write_wait_clause_holds(
                write_wait_increment_nanoseconds,
                injected_nanoseconds,
            );
            let wait_clause_under_the_original_criterion_holds =
                cpu_arm_original_wait_clause_holds(wait_change);
            RowOneControlJudgement::CpuArm {
                cpu_clause_holds,
                write_wait_clause_holds,
                wait_clause_under_the_original_criterion_holds,
            }
        }
        WriteInjection::Nothing => return None,
    };
    let cpu_differences =
        paired_round_differences(baseline, control, |round| round.cpu_nanoseconds);
    let write_wait_differences =
        paired_round_differences(baseline, control, |round| round.write_wait_nanoseconds);
    Some(RowOneControlReading {
        write_calls,
        injected_nanoseconds,
        cpu_increment_nanoseconds,
        wait_increment_nanoseconds,
        write_wait_increment_nanoseconds,
        cpu_change,
        wait_change,
        paired_rounds: write_wait_differences.len(),
        cpu_increment_paired_median_nanoseconds: median_of(&cpu_differences),
        write_wait_increment_paired_median_nanoseconds: median_of(&write_wait_differences),
        judgement,
    })
}

/// 第二行阶段对照：加 2 ms 忙等之后的增量在 2 ms ± 10% 里。
fn stage_control_holds(increment_nanoseconds: f64) -> bool {
    let injected = (INJECTED_MILLISECONDS_PER_WRITE * 1_000_000) as f64;
    relative_change(increment_nanoseconds, injected).abs() <= CONTROL_TOLERANCE
}

/// 第二行甲的对照：算两遍是一遍的 [1.8, 2.2] 倍。
fn double_crc_control_holds(twice: f64, once: f64) -> bool {
    (DOUBLE_CRC_RATIO_LOWER..=DOUBLE_CRC_RATIO_UPPER).contains(&(twice / once))
}

/// 第四行对照：走页缓存不同步的带宽 ≥ `O_DIRECT` 带同步的 1.15 倍。
fn page_cache_control_holds(
    page_cache_bytes_per_second: f64,
    direct_bytes_per_second: f64,
) -> bool {
    page_cache_bytes_per_second >= PAGE_CACHE_SPEEDUP_AT_LEAST * direct_bytes_per_second
}

// ───────────────────────────── GPU 一侧的算术（第三段；纯函数，不编 gpu 特性也有单测） ─────────────────────────────

/// PCIe 链路一个方向的峰值（字节 / 秒）：每条 lane 的传输率 × lane 数 × 编码效率 ÷ 8。1、2 代 8b/10b，3 到 5 代 128b/130b；别的代数交回 `None`
/// （读不到，那张卡作废，登记第十一节「读不到」）。5 代 x16 是 63015384615.38…，x8 是 31507692307.69…（登记第七节乙类按四舍五入写）。
fn pcie_link_peak_bytes_per_second(generation: u32, lanes: u32) -> Option<f64> {
    let (giga_transfers_per_lane, payload_bits, line_bits) = match generation {
        1 => (2.5, 8.0, 10.0),
        2 => (5.0, 8.0, 10.0),
        3 => (8.0, 128.0, 130.0),
        4 => (16.0, 128.0, 130.0),
        5 => (32.0, 128.0, 130.0),
        _ => return None,
    };
    Some(giga_transfers_per_lane * 1e9 * f64::from(lanes) * payload_bits / line_bits / 8.0)
}

/// 一次调用没快过物理下限：`t ≥ 输入字节 ÷ 链路峰值`（快过就是没等到提交做完，那张卡作废）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
fn respects_the_physical_floor(
    seconds: f64,
    input_bytes: u64,
    link_peak_bytes_per_second: f64,
) -> bool {
    seconds >= input_bytes as f64 / link_peak_bytes_per_second
}

/// 最小二乘拟合 `t = a + b·B`：交回 (a 截距秒, b 斜率秒 / 字节)；少于两个不同的 B 交回 `None`。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
fn least_squares_intercept_and_slope(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    let count = points.len() as f64;
    let mean_bytes = points.iter().map(|(bytes, _seconds)| bytes).sum::<f64>() / count;
    let mean_seconds = points.iter().map(|(_bytes, seconds)| seconds).sum::<f64>() / count;
    let covariance: f64 = points
        .iter()
        .map(|(bytes, seconds)| (bytes - mean_bytes) * (seconds - mean_seconds))
        .sum();
    let variance: f64 = points
        .iter()
        .map(|(bytes, _seconds)| (bytes - mean_bytes).powi(2))
        .sum();
    if points.len() < 2 || variance == 0.0 {
        return None;
    }
    let slope = covariance / variance;
    let intercept = mean_seconds - slope * mean_bytes;
    Some((intercept, slope))
}

/// Q13 的 φ = a ÷ (a + b·B_批)，B_批 按字节代。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
fn fixed_overhead_share(
    intercept_seconds: f64,
    slope_seconds_per_byte: f64,
    batch_bytes: u64,
) -> f64 {
    let batch_bytes_wide = batch_bytes as f64;
    intercept_seconds / (intercept_seconds + slope_seconds_per_byte * batch_bytes_wide)
}

/// Q13：φ > 10% 时要合几批 n* = ⌈9a ÷ (b·B_批)⌉（合 n* 批之后 φ ≤ 10%）。交回浮点数，打印时按整数写。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
fn batches_to_merge(intercept_seconds: f64, slope_seconds_per_byte: f64, batch_bytes: u64) -> f64 {
    (9.0 * intercept_seconds / (slope_seconds_per_byte * batch_bytes as f64)).ceil()
}

/// GPU 交回的 CRC 与 CPU 逐段相同（有一段不同整张卡作废，登记第五节「GPU 一侧怎么量」）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
fn gpu_crcs_agree_with_the_cpu(gpu_results: &[u32], cpu_results: &[u32]) -> bool {
    gpu_results.len() == cpu_results.len()
        && gpu_results
            .iter()
            .zip(cpu_results)
            .all(|(gpu_value, cpu_value)| gpu_value == cpu_value)
}

/// 公开校验值：CRC-32C("123456789") = 0xE3069283。
const CHECK_STRING: &[u8] = b"123456789";
const CHECK_STRING_CRC32_CASTAGNOLI: u32 = 0xE306_9283;

// ───────────────────────────── 阶段计时与饱和吞吐（第二段） ─────────────────────────────

/// 一个纯阶段在一次重放里的读数：每次重复的 CPU 纳秒与挂钟纳秒、重复了几次。
#[derive(Clone, Copy, Debug, PartialEq)]
struct StageTiming {
    cpu_nanoseconds_per_repetition: f64,
    wall_nanoseconds_per_repetition: f64,
    repetitions: u64,
}

/// 一次重放：先不计时跑一遍热身（被测那一批里扫描读到的字节刚过缓存，热身让重放的缓存状态接近那一次），再计时跑一遍，交回这一遍的
/// CPU 与挂钟（`repetitions` 是 1）；线程 CPU 时钟读不到交回 `None`。只计时一遍（登记第五节「每个阶段重放 5 次取中位」、阳性对照表
/// 第 2 行「各跑一次」）：线程 CPU 时钟读的那一刻现算到纳秒，单遍的阶段读得出数，不用为了钟的分辨率把同一份输入重复跑很多遍。
/// 挂钟读数包着 CPU 读数，与一次 `run_compaction` 调用那一层同一个嵌套。
fn time_a_pure_stage(mut stage: impl FnMut()) -> Option<StageTiming> {
    stage();
    let started = RawWallClockReading::now();
    let cpu_before = thread_cpu_nanoseconds()?;
    stage();
    let cpu_after = thread_cpu_nanoseconds()?;
    let wall = started.elapsed();
    Some(StageTiming {
        cpu_nanoseconds_per_repetition: cpu_after.saturating_sub(cpu_before) as f64,
        wall_nanoseconds_per_repetition: wall.as_nanos() as f64,
        repetitions: 1,
    })
}

/// 甲段：对每个被区域扫描读到的单元的整单元字节算 CRC-32C；交回算过的字节数与各段 CRC 的异或（不让优化器删掉这段活）。
fn crc_stage_over_the_units(units: &[Vec<u8>]) -> (u64, u32) {
    let mut covered_bytes = 0_u64;
    let mut folded = 0_u32;
    for unit in units {
        let covered: &[u8] = unit.as_slice();
        covered_bytes += length_in_bytes(covered);
        folded ^= crc32_castagnoli(std::hint::black_box(covered));
    }
    (covered_bytes, folded)
}

/// 饱和吞吐的一档：合计字节 ÷ 挂钟、起了几条不同的线程、合计重复次数。
#[derive(Clone, Copy, Debug, PartialEq)]
struct SaturationReading {
    bytes_per_second: f64,
    distinct_threads: usize,
    repetitions: u64,
}

/// `thread_count` 条线程各自重复跑同一阶段函数 `duration`（登记第五节「饱和吞吐」）；装置核起了几条不同的线程。
fn saturated_throughput(
    thread_count: usize,
    duration: Duration,
    bytes_per_repetition: u64,
    stage: &(dyn Fn() + Sync),
) -> SaturationReading {
    let repetitions = AtomicU64::new(0);
    let started = Instant::now();
    let thread_identities: Vec<std::thread::ThreadId> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..thread_count)
            .map(|_worker_index| {
                scope.spawn(|| {
                    // 迭代上界：挂钟到 `duration` 就停；每次重复是一次完整的阶段函数。
                    loop {
                        stage();
                        repetitions.fetch_add(1, Ordering::Relaxed);
                        if started.elapsed() >= duration {
                            break;
                        }
                    }
                    std::thread::current().id()
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("饱和吞吐的线程跑完、没 panic"))
            .collect()
    });
    let wall_seconds = started.elapsed().as_secs_f64();
    let total_repetitions = repetitions.load(Ordering::Relaxed);
    let distinct: HashSet<std::thread::ThreadId> = thread_identities.into_iter().collect();
    SaturationReading {
        bytes_per_second: (total_repetitions * bytes_per_repetition) as f64 / wall_seconds,
        distinct_threads: distinct.len(),
        repetitions: total_repetitions,
    }
}

// ───────────────────────────── 造池（登记第五节「负载：池怎么造」） ─────────────────────────────

/// 建 inode 时在 P 个之外多建的空 inode：几何核之前把开着的那一段用户数据补满用（[`construct_the_pool`] 第 4 步）。
const SPARE_INODES_FOR_THE_FRONTIER: u64 = 64;
/// 抬 F 到准入上限、再建一个 inode 那一圈最多转几圈（每圈之后核被覆盖写的旧落点都已释放、回到空闲）。
const RELEASE_ROUNDS_AT_MOST: u64 = 16;
/// 建池时用户改动写进记录的时间：第一个文件之后 600 秒起，每次改动加一。
const SECONDS_FROM_THE_FIRST_FILE_TO_THE_WORKLOAD: u64 = 600;

/// 造池的配方：取样点，加上几何没造成时累计改负载多覆盖写掉的 inode（登记第五节「先改负载重造」）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct PoolRecipe {
    point: SamplingPoint,
    extra_overwritten_inodes: BTreeSet<u64>,
}

/// 造好的池：两块内存盘（关掉、没卸载）与造的经过里要报的数。
struct ConstructedPool {
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    /// 现行那一版里这次写的数据单元：槽（盘 0）→ inode。
    live_data_inode_by_slot: BTreeMap<u64, u64>,
    unit_area_start_slot: u64,
    overwritten_units: u64,
    filler_units: u64,
    floor_raises_pushed_by_the_session: u64,
    release_rounds: u64,
    is_every_overwritten_slot_released: bool,
    construction_seconds: f64,
}

/// 一个数据单元装满的内容：按 inode 与第几次写的不同而不同（净荷容量那么长，正好一个数据单元）。
fn content_of_a_data_unit(inode: u64, write_generation: u64) -> Vec<u8> {
    let capacity = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
    (0..capacity)
        .map(|position| {
            u8::try_from((inode * 131 + write_generation * 17 + position * 7) % 251)
                .expect("小于 251")
        })
        .collect()
}

/// 有树的那一版（建池与测量的会话里，第一个文件之后每一版都有树）。
fn version_of_trees(session: &MountedSession) -> &TransactionOutput {
    session
        .current
        .version_of_trees()
        .expect("第一个文件发过之后，会话的现行那一版都有树")
}

/// 这一版里 `inode` 那个文件第一个数据单元的槽（盘 0）。
fn data_unit_slot_of(version: &TransactionOutput, inode: u64) -> u64 {
    let file = version
        .files_other_than_the_first
        .iter()
        .find(|file| file.inode == InodeNumber(inode))
        .expect("刚写过数据的 inode 在这一版的文件表里");
    version.unit(file.data_role_at(0)).slot.0
}

/// 一段的起点槽（单元区起点在聚簇段边界上）。
const fn segment_start_of(slot: u64, unit_area_start_slot: u64) -> u64 {
    unit_area_start_slot
        + (slot - unit_area_start_slot) / LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_CLUSTER_SEGMENT_SLOTS
}

/// 建池会话发一次用户改动，计它推的抬 F 串数。
fn publish_while_constructing(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
    change: UserChange<'_>,
    floor_raises: &mut u64,
) {
    let published = session
        .publish_user_change(devices, change)
        .unwrap_or_else(|refusal| panic!("建池的用户改动发得成：{refusal:?}"));
    *floor_raises += u64::try_from(published.floor_raises.len()).expect("抬 F 串数");
}

/// 抬 F 到准入上限（会话的分配器与现行那一版，同 harness 整理流前缀的做法）。
fn raise_the_floor_to_the_admission_ceiling<Device: BlockDevice>(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) {
    let mut current = version_of_trees(session).clone();
    let _raised = raise_rollback_floor_to_the_admission_ceiling(
        &common::parameters(),
        devices,
        &mut session.allocator,
        &mut current,
        session.shadow_ledger,
    )
    .expect("抬 F 到准入上限");
    session.current = PoolVersion::of_a_version_of_trees(current);
}

/// 盘 0 上被未释放记录罩住的槽。
fn slots_under_an_unreleased_record_on_device_zero(version: &TransactionOutput) -> BTreeSet<u64> {
    version
        .allocation_records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && !record.is_released)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// mkfs → 取号 → 暖机 → 新池新建文件（同 harness `common::build_pool`，只是落在两块内存盘上、不录流），再可写挂载开会话。
fn formatted_pool_with_the_first_file_and_a_session(
) -> (Vec<(DeviceIdentity, SparseBlockDevice)>, MountedSession) {
    let parameters = common::parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0_u32, 1]
        .into_iter()
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(
                    LOCAL_DEVICE_BYTES,
                    PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
    ]);
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
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file_content = common::file_content();
        let _first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_file_content,
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable_with_compaction_switches(
        &parameters,
        &mut devices,
        CompactionSwitches::PRODUCT,
    )
    .expect("建池那一次可写挂载");
    let (_mount_output, session) = MountedSession::of_the_mount(mounted);
    (devices, session)
}

/// 按配方造一个池（登记第五节「负载：池怎么造」）：
/// 1. 建 P + 64 个空 inode，前 P 个各顺序写一个数据单元（按段顺序写满 ⌈P ÷ 32⌉ 段；用户数据取最低的空槽对）；
/// 2. 整段 32 个数据单元都是这次写的段，按槽序每 4 个覆盖写其中 3 个（f = 0.25）或 1 个（f = 0.75）；没写满一整段的段（与别的单元混住的）
///    这次写的单元全覆盖写掉，配方里累计改负载点名的 inode 也覆盖写掉；
/// 3. 下一个用户数据落点还在最高那一段里时，用多建的空 inode 一个一个写，把那一段补满（免得留一段活槽少的新数据段）；
/// 4. 抬 F 到准入上限、建一个 inode，转到被覆盖写的旧落点都按记录已释放、回到空闲（或已被别的这次写的单元占回）为止。
///
/// 关掉（不卸载）交回两块内存盘；测量那一侧再 `mount_writable_with_compaction_switches` 可写挂载，这样造出来的段不是那次挂载开过的段。
fn construct_the_pool(recipe: &PoolRecipe) -> ConstructedPool {
    let started = Instant::now();
    let (mut devices, mut session) = formatted_pool_with_the_first_file_and_a_session();
    let mut floor_raises = 0_u64;
    let mut write_time_seconds =
        common::FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_FIRST_FILE_TO_THE_WORKLOAD;
    let mut next_write_time = || {
        write_time_seconds += 1;
        write_time_seconds
    };
    let first_inode = version_of_trees(&session).inode_number_watermark();
    let pool_data_units = recipe.point.pool_data_units;
    publish_while_constructing(
        &mut session,
        &mut devices,
        UserChange::NewInodes {
            count: pool_data_units + SPARE_INODES_FOR_THE_FRONTIER,
            write_time_seconds: next_write_time(),
        },
        &mut floor_raises,
    );
    let mut workload = WorkloadUnits {
        live_data_inode_by_slot: BTreeMap::new(),
        slot_of_inode: BTreeMap::new(),
        floor_raises,
    };
    for inode in first_inode..first_inode + pool_data_units {
        let write_time = next_write_time();
        workload.write_one_unit(&mut session, &mut devices, inode, 0, write_time);
    }
    let unit_area_start_slot = session.allocator.devices[0].unit_area_start().slot().0;
    let full_segment_units = LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_SLOT_BYTES / LOCAL_DATA_UNIT_BYTES;
    let mut units_by_segment: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for (slot, inode) in &workload.live_data_inode_by_slot {
        units_by_segment
            .entry(segment_start_of(*slot, unit_area_start_slot))
            .or_default()
            .push(*inode);
    }
    let mut inodes_to_overwrite: Vec<(u64, u64)> = Vec::new();
    for inodes_in_slot_order in units_by_segment.values() {
        let is_full_of_this_workload =
            u64::try_from(inodes_in_slot_order.len()).expect("一段的单元数") == full_segment_units;
        for (position, inode) in (0_u64..).zip(inodes_in_slot_order) {
            if !is_full_of_this_workload
                || recipe.point.fraction.is_overwritten(position)
                || recipe.extra_overwritten_inodes.contains(inode)
            {
                inodes_to_overwrite.push((workload.slot_of_inode[inode], *inode));
            }
        }
    }
    inodes_to_overwrite.sort_unstable();
    let old_slots: Vec<u64> = inodes_to_overwrite
        .iter()
        .map(|(slot, _inode)| *slot)
        .collect();
    for (_old_slot, inode) in &inodes_to_overwrite {
        let write_time = next_write_time();
        workload.write_one_unit(&mut session, &mut devices, *inode, 1, write_time);
    }
    let mut filler_units = 0_u64;
    let frontier_segment = workload
        .live_data_inode_by_slot
        .keys()
        .max()
        .map(|slot| segment_start_of(*slot, unit_area_start_slot));
    let first_spare_inode = first_inode + pool_data_units;
    let mut spare_inodes = first_spare_inode..first_spare_inode + SPARE_INODES_FOR_THE_FRONTIER;
    // 迭代上界：一段至多 32 个数据单元；多建的空 inode 用完也停。
    for _filler in 0..full_segment_units {
        let next_data_slot = session.allocator.devices[0]
            .lowest_user_data_slot(session.allocator.cluster_segments());
        let (Some(frontier), Some(next_slot)) = (frontier_segment, next_data_slot) else {
            break;
        };
        if segment_start_of(next_slot.0, unit_area_start_slot) != frontier {
            break;
        }
        let Some(spare_inode) = spare_inodes.next() else {
            break;
        };
        let write_time = next_write_time();
        workload.write_one_unit(&mut session, &mut devices, spare_inode, 0, write_time);
        filler_units += 1;
    }
    let mut release_rounds = 0_u64;
    let mut is_every_overwritten_slot_released = false;
    while release_rounds < RELEASE_ROUNDS_AT_MOST {
        release_rounds += 1;
        raise_the_floor_to_the_admission_ceiling(&mut session, &mut devices);
        let under_a_record =
            slots_under_an_unreleased_record_on_device_zero(version_of_trees(&session));
        is_every_overwritten_slot_released = old_slots.iter().all(|slot| {
            workload.live_data_inode_by_slot.contains_key(slot)
                || (!under_a_record.contains(slot)
                    && session.allocator.devices[0].is_free(SlotNumber(*slot)))
        });
        if is_every_overwritten_slot_released {
            break;
        }
        let write_time = next_write_time();
        publish_while_constructing(
            &mut session,
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: write_time,
            },
            &mut workload.floor_raises,
        );
    }
    drop(session);
    ConstructedPool {
        devices,
        live_data_inode_by_slot: workload.live_data_inode_by_slot,
        unit_area_start_slot,
        overwritten_units: u64::try_from(old_slots.len()).expect("覆盖写的单元数"),
        filler_units,
        floor_raises_pushed_by_the_session: workload.floor_raises,
        release_rounds,
        is_every_overwritten_slot_released,
        construction_seconds: started.elapsed().as_secs_f64(),
    }
}

/// 建池时这次写的数据单元在哪（槽 → inode、inode → 槽）与会话推的抬 F 串数。
struct WorkloadUnits {
    live_data_inode_by_slot: BTreeMap<u64, u64>,
    slot_of_inode: BTreeMap<u64, u64>,
    floor_raises: u64,
}

impl WorkloadUnits {
    /// 给 `inode` 写一个数据单元：第 0 次是顺序写（空 inode），之后是覆盖写；记下它的新槽，旧槽上还记着它的就抹掉。
    fn write_one_unit(
        &mut self,
        session: &mut MountedSession,
        devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
        inode: u64,
        write_generation: u64,
        write_time: u64,
    ) {
        let content = content_of_a_data_unit(inode, write_generation);
        let change = if write_generation == 0 {
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(inode),
                content: &content,
                write_time_seconds: write_time,
            }
        } else {
            UserChange::OverwriteOfAnInode {
                inode: InodeNumber(inode),
                content: &content,
                write_time_seconds: write_time,
            }
        };
        publish_while_constructing(session, devices, change, &mut self.floor_raises);
        let new_slot = data_unit_slot_of(version_of_trees(session), inode);
        if let Some(old_slot) = self.slot_of_inode.insert(inode, new_slot) {
            if self.live_data_inode_by_slot.get(&old_slot) == Some(&inode) {
                self.live_data_inode_by_slot.remove(&old_slot);
            }
        }
        self.live_data_inode_by_slot.insert(new_slot, inode);
    }
}

// ───────────────────────────── 重放的输入（登记第五节「阶段拆分」：会话的取法另写一份） ─────────────────────────────

/// 一版里整理要读的几样，装置照 `mounted_session.rs` 的私有函数 `version_as_the_compaction_reads_it` 另写一份：每个进映射单元的映射条目
/// （按 key 在这一版的映射树里查，连同留存给快照的）、钉住的槽（映射树各节点、树表、实例表链）。取对没有，由停机条款第 4 条核
/// （取出的条目与会话那一批判成可搬的 key 对不上就停）。
struct VersionAsTheReplayReadsIt {
    allocation_records: Vec<AllocationRecord>,
    mapping_entries: Vec<CentralMappingEntry>,
    pinned_slots: BTreeSet<SlotNumber>,
}

fn version_as_the_replay_reads_it(version: &TransactionOutput) -> VersionAsTheReplayReadsIt {
    let mut mapping_entries: Vec<CentralMappingEntry> = version
        .mapped_units
        .iter()
        .filter_map(
            |(_, key)| match central_mapping_locations_in_the_version(version, key) {
                MappingLookup::Found(locations) => Some(CentralMappingEntry {
                    key: key.clone(),
                    locations,
                }),
                MappingLookup::NodeMalformedOrNoEntryWithThisKey
                | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
            },
        )
        .collect();
    mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(
        |(key, locations)| CentralMappingEntry {
            key: key.clone(),
            locations: *locations,
        },
    ));
    let mut pinned_slots = BTreeSet::new();
    for unit in &version.units {
        let is_exempt_from_the_mapping = match unit.identity {
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => true,
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_) => false,
        };
        if is_exempt_from_the_mapping {
            for slot in unit.slot.0..unit.slot.0 + unit.identity.span_slots() {
                pinned_slots.insert(SlotNumber(slot));
            }
        }
    }
    VersionAsTheReplayReadsIt {
        allocation_records: version.allocation_records.clone(),
        mapping_entries,
        pinned_slots,
    }
}

/// 会话这一侧选段要的几样（调用之前那一刻的拷贝）：盘 0 的单元区、这次挂载开过的段、常驻隔离集、区域扫描拒过的段。
#[derive(Clone)]
struct SessionStateForTheChoice {
    unit_area: UnitAreaOfOneDevice,
    cluster_segments: BTreeSet<SlotNumber>,
    quarantined_slots: BTreeSet<SlotNumber>,
    segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
}

fn session_state_for_the_choice(session: &MountedSession) -> SessionStateForTheChoice {
    let device_map = session
        .allocator
        .devices
        .first()
        .expect("池里至少一块盘：选段按第一块盘的记录做");
    SessionStateForTheChoice {
        unit_area: UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        },
        cluster_segments: session.allocator.cluster_segments().clone(),
        quarantined_slots: session.compaction.quarantined_slots().clone(),
        segments_refused_by_the_region_scan: session
            .compaction
            .segments_refused_by_the_region_scan
            .clone(),
    }
}

/// 选段那一圈的重放（照会话签发那一步：`choose_region` 与「先扫」交替）：选中哪一段、每次 `choose_region` 调用时的输入（丙段计时照它重放）、
/// 先扫了哪几段。
struct GreedyReplay {
    chosen: Option<CompactionRegion>,
    inputs_of_each_call: Vec<(Vec<RegionScan>, SegmentsRefusedByTheRegionScan)>,
    scanned_regions: Vec<CompactionRegion>,
}

fn replay_the_greedy_choice(
    version: &VersionAsTheReplayReadsIt,
    state: &SessionStateForTheChoice,
    mut read_unit: impl FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
) -> GreedyReplay {
    let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = version
        .pinned_slots
        .union(&state.quarantined_slots)
        .copied()
        .collect();
    let mut region_scans = region_scans_of_the_segments_without_an_unreleased_record(
        &version.allocation_records,
        state.unit_area,
    );
    let mut refused = state.segments_refused_by_the_region_scan.clone();
    let mut replay = GreedyReplay {
        chosen: None,
        inputs_of_each_call: Vec::new(),
        scanned_regions: Vec::new(),
    };
    // 迭代上界：每答一次「先扫」下一次就多一段有结论，不超过单元区的整段数（同会话那一圈）。
    for _call in 0..=state.unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS {
        replay
            .inputs_of_each_call
            .push((region_scans.clone(), refused.clone()));
        match choose_region(
            &version.allocation_records,
            state.unit_area,
            &state.cluster_segments,
            None,
            &slots_that_disqualify_a_segment,
            &region_scans,
            &refused,
            RegionChoiceCriterion::Greedy,
        ) {
            RegionChoice::Chosen(region) => {
                replay.chosen = Some(region);
                return replay;
            }
            RegionChoice::ScanBeforeChoosing(region) => {
                replay.scanned_regions.push(region);
                match scan_region(
                    region,
                    &version.allocation_records,
                    &version.mapping_entries,
                    &version.pinned_slots,
                    &state.quarantined_slots,
                    &mut read_unit,
                ) {
                    Ok(scan) => region_scans.push(scan),
                    Err(refusal) => refused.note_refusal(region, refusal),
                }
            }
            RegionChoice::NoEligibleSegment => return replay,
        }
    }
    replay
}

/// 一段的样子（几何核用）：R 那块盘上未释放记录罩住的活槽数、其中这次写的数据单元占的槽、别的槽（节点、打包单元、别人的数据）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SegmentComposition {
    start_slot: u64,
    live_slots: u64,
    slots_of_this_workload: u64,
    other_slots: u64,
}

fn composition_of_the_segment(
    records: &[AllocationRecord],
    region: CompactionRegion,
    live_data_inode_by_slot: &BTreeMap<u64, u64>,
) -> SegmentComposition {
    let mut composition = SegmentComposition {
        start_slot: region.start_slot().0,
        live_slots: 0,
        slots_of_this_workload: 0,
        other_slots: 0,
    };
    for record in records.iter().filter(|record| {
        record.device == region.device() && !record.is_released && region.contains_slot(record.slot)
    }) {
        let span = u64::from(record.span_slots);
        composition.live_slots += span;
        if span * LOCAL_SLOT_BYTES == LOCAL_DATA_UNIT_BYTES
            && live_data_inode_by_slot.contains_key(&record.slot.0)
        {
            composition.slots_of_this_workload += span;
        } else {
            composition.other_slots += span;
        }
    }
    composition
}

/// 几何造成了：R 里全是这次写的码 1 数据单元、活槽 = 64f（登记第七节乙类的前提）。
fn is_the_geometry_formed(composition: SegmentComposition, fraction: LiveSlotFraction) -> bool {
    composition.other_slots == 0 && composition.live_slots == fraction.live_slots_per_segment()
}

// ───────────────────────────── 每个取样点的池与几何（登记第五节「挂好之后、第一次 run_compaction 之前，装置核三样」） ─────────────────────────────

/// 一块内存盘的整份拷贝（`SparseBlockDevice` 不能克隆，拷它的镜像）。
fn copy_of_the_memory_devices(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy =
                SparseBlockDevice::new(device.size_in_bytes(), device.probe_physical_block_size());
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

/// 测量会话装的整理开关：`CompactionSwitches::PRODUCT`，只把批大小换成只供测试的 `ForcedForTests(K)`；几何没造成走后备时另点名 R
/// （登记第五节「真实基线」与「负载」那一段的后备）。
fn measured_switches(
    batch_units: u64,
    forced_region: Option<CompactionRegion>,
) -> CompactionSwitches {
    let mut switches = CompactionSwitches::PRODUCT;
    switches.batch_size = CompactionBatchSize::ForcedForTests(
        CompactionBatchUnits::new(usize::try_from(batch_units).expect("批大小装得进 usize"))
            .expect("批大小至少 1"),
    );
    switches.forced_region = forced_region;
    switches
}

/// R 怎么选：产品的 greedy，或几何造不成时登记好的后备（只供测试的强制选段，点名一个 f 段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChoiceOfTheRegion {
    Greedy,
    ForcedFallback(CompactionRegion),
}

impl ChoiceOfTheRegion {
    const fn label(self) -> &'static str {
        match self {
            ChoiceOfTheRegion::Greedy => "greedy",
            ChoiceOfTheRegion::ForcedFallback(_) => "forced_fallback",
        }
    }

    const fn forced_region(self) -> Option<CompactionRegion> {
        match self {
            ChoiceOfTheRegion::Greedy => None,
            ChoiceOfTheRegion::ForcedFallback(region) => Some(region),
        }
    }
}

/// 在一份拷贝上可写挂载一次、照会话的选段那一圈重放，看 greedy 将选哪一段、它的样子，另列出全部 f 段（活槽 = 64f、全是这次写的数据单元）。
struct GreedyPrediction {
    greedy_region: Option<CompactionRegion>,
    greedy_composition: Option<SegmentComposition>,
    geometry_formed_segments: Vec<CompactionRegion>,
    mapped_units: usize,
}

fn predict_the_greedy_choice(
    pool: &ConstructedPool,
    fraction: LiveSlotFraction,
) -> GreedyPrediction {
    let mut devices = copy_of_the_memory_devices(&pool.devices);
    let mounted = mount_writable_with_compaction_switches(
        &common::parameters(),
        &mut devices,
        measured_switches(BATCH_SIZES[0], None),
    )
    .expect("几何核那一次可写挂载");
    let (_mount_output, session) = MountedSession::of_the_mount(mounted);
    let version = version_as_the_replay_reads_it(version_of_trees(&session));
    let state = session_state_for_the_choice(&session);
    let replay = replay_the_greedy_choice(&version, &state, |device, slot, length| {
        PoolReader::read(devices.as_slice(), device, slot.to_device_offset(), length)
    });
    let segment_starts: BTreeSet<u64> = pool
        .live_data_inode_by_slot
        .keys()
        .map(|slot| segment_start_of(*slot, pool.unit_area_start_slot))
        .collect();
    let geometry_formed_segments = segment_starts
        .into_iter()
        .map(|start| {
            CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(start))
                .expect("段起点在聚簇段边界上、装得进 6 字节")
        })
        .filter(|region| {
            is_the_geometry_formed(
                composition_of_the_segment(
                    &version.allocation_records,
                    *region,
                    &pool.live_data_inode_by_slot,
                ),
                fraction,
            )
        })
        .collect();
    GreedyPrediction {
        greedy_region: replay.chosen,
        greedy_composition: replay.chosen.map(|region| {
            composition_of_the_segment(
                &version.allocation_records,
                region,
                &pool.live_data_inode_by_slot,
            )
        }),
        geometry_formed_segments,
        mapped_units: version_of_trees(&session).mapped_units.len(),
    }
}

/// 一个取样点准备好的样子：最后用的配方、R 怎么选、几何造没造成（没造成的点不进判定）、第一次造的池（给第一轮用）与它的分块摘要，
/// 以及准备时要打的产物行（准备在别的线程上跑，行先攒着，轮到这个点量的时候按原来的次序打出去，登记第十二节 12.10）。
struct PreparedPoint {
    recipe: PoolRecipe,
    choice: ChoiceOfTheRegion,
    is_the_geometry_formed: bool,
    pool: ConstructedPool,
    reference_digests: Vec<ChunkDigest>,
    lines: Vec<String>,
}

/// 造池并核几何：greedy 将选的段是 f 段就用 greedy；不是而那一段全是这次写的数据单元，就把它们并进配方多覆盖写掉重造，至多改三次；
/// 那一段装着别的单元（节点、打包单元）时没有登记好的负载改法，直接走后备（点名槽号最小的 f 段）。一个 f 段都没有，几何没造成。
/// 产物行不直接打，攒进交回的 [`PreparedPoint::lines`]：同一个点的几次重造前后依赖，在一条线程上按次序跑；不同的点互不依赖，由
/// [`PreparationQueue`] 与别的准备单位一起分到几条线程上。
fn prepare_the_sampling_point(point: SamplingPoint) -> PreparedPoint {
    let mut lines = Vec::new();
    let mut recipe = PoolRecipe {
        point,
        extra_overwritten_inodes: BTreeSet::new(),
    };
    let mut workload_changes = 0_usize;
    loop {
        let pool = construct_the_pool(&recipe);
        let prediction = predict_the_greedy_choice(&pool, point.fraction);
        let greedy = prediction.greedy_composition;
        let is_greedy_formed =
            greedy.is_some_and(|composition| is_the_geometry_formed(composition, point.fraction));
        lines.push(format!(
            "name=geometry_attempt {} workload_changes={workload_changes} construction_seconds={:.3} overwritten_units={} filler_units={} floor_raises_pushed_by_the_session={} release_rounds={} every_overwritten_slot_released={} mapped_units={} geometry_formed_segments={} greedy_region_start_slot={} greedy_live_slots={} greedy_slots_of_this_workload={} greedy_other_slots={} greedy_formed={is_greedy_formed}",
            point.label(),
            pool.construction_seconds,
            pool.overwritten_units,
            pool.filler_units,
            pool.floor_raises_pushed_by_the_session,
            pool.release_rounds,
            pool.is_every_overwritten_slot_released,
            prediction.mapped_units,
            prediction.geometry_formed_segments.len(),
            greedy.map_or_else(|| "none".to_string(), |composition| composition.start_slot.to_string()),
            greedy.map_or_else(|| "none".to_string(), |composition| composition.live_slots.to_string()),
            greedy.map_or_else(|| "none".to_string(), |composition| composition.slots_of_this_workload.to_string()),
            greedy.map_or_else(|| "none".to_string(), |composition| composition.other_slots.to_string()),
        ));
        if is_greedy_formed {
            lines.push(format!(
                "name=geometry {} choice=greedy region_start_slot={} formed=true workload_changes={workload_changes}",
                point.label(),
                prediction
                    .greedy_region
                    .map_or_else(|| "none".to_string(), |region| region.start_slot().0.to_string()),
            ));
            let reference_digests = digests_of_the_pool(&pool);
            return PreparedPoint {
                recipe,
                choice: ChoiceOfTheRegion::Greedy,
                is_the_geometry_formed: true,
                pool,
                reference_digests,
                lines,
            };
        }
        let competitor_inodes: Vec<u64> = match (prediction.greedy_region, greedy) {
            (Some(region), Some(composition)) if composition.other_slots == 0 => pool
                .live_data_inode_by_slot
                .iter()
                .filter(|(slot, _inode)| region.contains_slot(SlotNumber(**slot)))
                .map(|(_slot, inode)| *inode)
                .collect(),
            (Some(_), Some(_)) | (None, _) | (Some(_), None) => Vec::new(),
        };
        if workload_changes < WORKLOAD_CHANGES_AT_MOST && !competitor_inodes.is_empty() {
            workload_changes += 1;
            recipe.extra_overwritten_inodes.extend(competitor_inodes);
            continue;
        }
        let fallback = prediction.geometry_formed_segments.first().copied();
        lines.push(format!(
            "name=geometry {} choice={} fallback_region_start_slot={} formed={} reason={}",
            point.label(),
            if fallback.is_some() {
                "forced_fallback"
            } else {
                "none"
            },
            fallback.map_or_else(
                || "none".to_string(),
                |region| region.start_slot().0.to_string()
            ),
            fallback.is_some(),
            if competitor_inodes.is_empty() {
                "greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change"
            } else {
                "workload_changed_three_times"
            },
        ));
        let reference_digests = digests_of_the_pool(&pool);
        return PreparedPoint {
            recipe,
            choice: fallback.map_or(ChoiceOfTheRegion::Greedy, ChoiceOfTheRegion::ForcedFallback),
            is_the_geometry_formed: fallback.is_some(),
            pool,
            reference_digests,
            lines,
        };
    }
}

// ───────────────────────────── 量一条意图（登记第五节「计时与计数怎么做」） ─────────────────────────────

/// 一条意图至多几次调用：签发、R 里至多 32 个数据单元一批一个、再留两次余量；超了就是整理停不下来，交回已有的。
const CALLS_OF_ONE_INTENT_AT_MOST: usize = 1 + 64 + 2;

/// 设备臂（问题单第 1 行的两臂）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DeviceArm {
    Memory,
    RealDisk,
}

impl DeviceArm {
    const ALL: [DeviceArm; 2] = [DeviceArm::Memory, DeviceArm::RealDisk];

    const fn label(self) -> &'static str {
        match self {
            DeviceArm::Memory => "memory",
            DeviceArm::RealDisk => "real_disk",
        }
    }
}

/// 一次调用的读数。
#[derive(Clone, Debug)]
struct CallReading {
    kind: CallKind,
    outcome: String,
    wall_nanoseconds: u64,
    cpu_nanoseconds: u64,
    is_the_cpu_unreadable: bool,
    device_totals: MethodTotals,
    per_method: BTreeMap<DeviceMethod, MethodTotals>,
    scan_read_bytes_in_the_region: u64,
    scan_read_bytes_elsewhere: u64,
    /// 释放前读盘核读这一批搬迁源的字节（两块盘），不进扫描读的两栏（登记第十二节 12.5）。
    release_check_read_bytes: u64,
    read_bytes_after_the_first_write: u64,
    write_bytes: u64,
    relocation_copy_bytes: u64,
    is_every_destination_written_whole: bool,
    relocated_units: u64,
    expected_scan_read_bytes: Option<u64>,
    batches_observed: u64,
    inclusion: InclusionVerdict,
}

impl CallReading {
    fn wait_nanoseconds(&self) -> i64 {
        self.device_totals.wait_nanoseconds
    }
}

/// 第二段重放要的材料（只在 `ReplayCapture::On` 时收）：调用之前那一版的重放输入与会话状态、扫描读到的单元字节（批：调用之前从盘上照 R
/// 里的记录读一份；签发：调用之后照记下的读重读一份）、这一次记下的扫描读、搬走的单元的映射 key；另带释放前读盘核那几次读与它们读到的
/// 副本字节（调用之前从每块盘同号槽读一份，按读的次序取；甲段第二个子项重放的就是这些字节，登记第十二节 12.5）。
struct CallReplayMaterial {
    call_index: usize,
    kind: CallKind,
    version: VersionAsTheReplayReadsIt,
    state: SessionStateForTheChoice,
    unit_bytes_by_slot: BTreeMap<u64, Vec<u8>>,
    scan_reads_in_the_region: Vec<RecordedRead>,
    release_check_reads: Vec<RecordedRead>,
    release_check_copies: Vec<Vec<u8>>,
    relocated_keys: Vec<Vec<u8>>,
}

/// 收不收第二段的重放材料。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplayCapture {
    Off,
    On,
}

/// 一条意图的读数：R、每次调用、挂载没做成的原因、重放材料。
struct IntentReading {
    region: Option<CompactionRegion>,
    calls: Vec<CallReading>,
    mount_failure: Option<String>,
    replay_material: Vec<CallReplayMaterial>,
}

/// 照这一版 R 那块盘上的未释放记录，从盘上读一份 R 里每个单元的字节（计时窗口外）。
fn unit_bytes_of_the_region<Inner: BlockDevice>(
    devices: &[(DeviceIdentity, TimedBlockDevice<Inner>)],
    region: CompactionRegion,
    records: &[AllocationRecord],
) -> BTreeMap<u64, Vec<u8>> {
    records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && region.contains_slot(record.slot)
        })
        .filter_map(|record| {
            let length = usize::try_from(u64::from(record.span_slots) * LOCAL_SLOT_BYTES)
                .expect("一个单元的字节");
            PoolReader::read(
                devices,
                record.device,
                record.slot.to_device_offset(),
                length,
            )
            .map(|bytes| (record.slot.0, bytes))
        })
        .collect()
}

/// 一次读落在 R 里没有（盘、槽）。
fn read_is_inside(read: &RecordedRead, region: CompactionRegion) -> bool {
    read.device == region.device()
        && region.contains_slot(SlotNumber(read.offset / LOCAL_SLOT_BYTES))
}

/// 一次调用里记下的读分成的四栏（登记第十二节 12.5）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ReadsOfOneCall {
    /// 第一次写之前、落在 R 那块盘 R 里、不是释放前读盘核的读（停机条款第 2 条比的就是它们的字节）。
    scan_reads_in_the_region: Vec<RecordedRead>,
    /// 第一次写之前、落在这一批搬迁源的槽上（两块盘同号槽都算）、扣掉区域扫描对 R 那块盘每个槽的第一次读之后剩下的读。
    release_check_reads: Vec<RecordedRead>,
    scan_read_bytes_elsewhere: u64,
    read_bytes_after_the_first_write: u64,
}

/// 把一次调用里记下的读按读的次序分栏。区域扫描对 R 那块盘每个未释放单元只读一遍、读在释放前读盘核之前，所以 R 那块盘上一个槽的
/// 第一次读归扫描；同一个槽在 R 那块盘上再读、而这个槽是这一批的搬迁源，归释放前读盘核；别的盘上搬迁源那几个槽的读全归释放前读盘核
/// （扫描只读 R 那块盘）。不是搬迁源的槽上再读，照旧归扫描，停机条款第 2 条照样看得见。
fn split_the_reads_of_a_call(
    reads: &[RecordedRead],
    region: Option<CompactionRegion>,
    relocated_source_slots: &BTreeSet<u64>,
) -> ReadsOfOneCall {
    let mut split = ReadsOfOneCall::default();
    let mut slots_read_by_the_scan: BTreeSet<u64> = BTreeSet::new();
    for read in reads {
        if !read.is_before_the_first_write {
            split.read_bytes_after_the_first_write += read.length;
            continue;
        }
        let slot = read.offset / LOCAL_SLOT_BYTES;
        let is_on_a_relocated_source = relocated_source_slots.contains(&slot);
        match region {
            Some(scanned_region) if read_is_inside(read, scanned_region) => {
                let is_the_first_read_of_the_slot = slots_read_by_the_scan.insert(slot);
                if is_the_first_read_of_the_slot || !is_on_a_relocated_source {
                    split.scan_reads_in_the_region.push(*read);
                } else {
                    split.release_check_reads.push(*read);
                }
            }
            Some(_) | None => {
                if is_on_a_relocated_source {
                    split.release_check_reads.push(*read);
                } else {
                    split.scan_read_bytes_elsewhere += read.length;
                }
            }
        }
    }
    split
}

/// 照这一版 R 那块盘上的未释放记录，从每块盘同号槽各读一份（计时窗口外；释放前读盘核读的是两块盘的副本，第二段要这些字节重放它的 CRC）。
fn copies_of_the_region_on_every_device<Inner: BlockDevice>(
    devices: &[(DeviceIdentity, TimedBlockDevice<Inner>)],
    region: CompactionRegion,
    records: &[AllocationRecord],
) -> BTreeMap<(DeviceIdentity, u64), Vec<u8>> {
    let mut copies = BTreeMap::new();
    for record in records.iter().filter(|record| {
        record.device == region.device() && !record.is_released && region.contains_slot(record.slot)
    }) {
        let length = usize::try_from(u64::from(record.span_slots) * LOCAL_SLOT_BYTES)
            .expect("一个单元的字节");
        for (identity, _device) in devices {
            if let Some(bytes) =
                PoolReader::read(devices, *identity, record.slot.to_device_offset(), length)
            {
                copies.insert((*identity, record.slot.0), bytes);
            }
        }
    }
    copies
}

/// 搬走的单元在目的地占几字节：码 1 数据单元与打包单元 32768，码 2 节点 16384（按映射 key 的类标签）。
fn unit_bytes_of_the_mapping_key(mapping_key: &[u8]) -> u64 {
    if mapping_key.first() == Some(&UNIT_CLASS_DATA)
        || mapping_key.first() == Some(&UNIT_CLASS_PACKED)
    {
        LOCAL_DATA_UNIT_BYTES
    } else {
        LOCAL_NODE_BYTES
    }
}

/// 在一组套了计时包装的盘上可写挂载、装上开关，一次一次调 `run_compaction`（每次之前把「已经过去的发布数」设成 1000，一次调用恰好发一次或停），
/// 直到最后一批删了意图、或停了。每次调用：清账、读 `Instant` 与线程 CPU 时钟、调用、再按反的次序读（挂钟读数包着 CPU 读数，理由在 `timed_call`）；调用返回之后（计时窗口外）判这一次是哪一种、拆读写。
fn measure_one_intent<Inner: BlockDevice>(
    devices: &mut Vec<(DeviceIdentity, TimedBlockDevice<Inner>)>,
    ledger: &Rc<RefCell<CallLedger>>,
    switches: CompactionSwitches,
    capture: ReplayCapture,
) -> IntentReading {
    let mut reading = IntentReading {
        region: switches.forced_region,
        calls: Vec::new(),
        mount_failure: None,
        replay_material: Vec::new(),
    };
    let mounted =
        match mount_writable_with_compaction_switches(&common::parameters(), devices, switches) {
            Ok(mounted) => mounted,
            Err(error) => {
                reading.mount_failure = Some(field_token(&format!("{error:?}")));
                return reading;
            }
        };
    for (_identity, device) in devices.iter_mut() {
        device.stop_mirroring_the_journal_writes();
    }
    let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
    for call_index in 0..CALLS_OF_ONE_INTENT_AT_MOST {
        let records_before = version_of_trees(&session).allocation_records.clone();
        let batches_before = session.compaction.observations.batches_published;
        let captured_before = match capture {
            ReplayCapture::Off => None,
            ReplayCapture::On => Some((
                version_as_the_replay_reads_it(version_of_trees(&session)),
                session_state_for_the_choice(&session),
                reading
                    .region
                    .map(|region| {
                        unit_bytes_of_the_region(devices.as_slice(), region, &records_before)
                    })
                    .unwrap_or_default(),
                reading
                    .region
                    .map(|region| {
                        copies_of_the_region_on_every_device(
                            devices.as_slice(),
                            region,
                            &records_before,
                        )
                    })
                    .unwrap_or_default(),
            )),
        };
        session.compaction.publishes_since_the_last_user_change =
            PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
        ledger.borrow_mut().open_a_new_window();
        let started = RawWallClockReading::now();
        let cpu_before = thread_cpu_nanoseconds();
        let run = session.run_compaction(devices);
        let cpu_after = thread_cpu_nanoseconds();
        let wall = started.elapsed();
        let (device_totals, per_method, reads, writes, is_the_device_cpu_unreadable) = {
            let window = ledger.borrow();
            (
                window.totals_of_every_method(),
                window.totals.clone(),
                window.reads.clone(),
                window.writes.clone(),
                window.is_the_cpu_unreadable_in_this_window,
            )
        };
        let (cpu_nanoseconds, is_the_call_cpu_unreadable) = match (cpu_before, cpu_after) {
            (Some(before), Some(after)) => (after.saturating_sub(before), false),
            (None, _) | (_, None) => (0, true),
        };
        let wall_nanoseconds = u64::try_from(wall.as_nanos()).expect("一次调用的挂钟装得进 u64");
        let Ok(run) = run else {
            reading.calls.push(CallReading {
                kind: CallKind::NoPublish,
                outcome: "refused_before_any_read_or_write".to_string(),
                wall_nanoseconds,
                cpu_nanoseconds,
                is_the_cpu_unreadable: is_the_call_cpu_unreadable || is_the_device_cpu_unreadable,
                device_totals,
                per_method,
                scan_read_bytes_in_the_region: 0,
                scan_read_bytes_elsewhere: 0,
                release_check_read_bytes: 0,
                read_bytes_after_the_first_write: 0,
                write_bytes: 0,
                relocation_copy_bytes: 0,
                is_every_destination_written_whole: true,
                relocated_units: 0,
                expected_scan_read_bytes: None,
                batches_observed: 0,
                inclusion: inclusion_verdict(
                    wall_nanoseconds,
                    cpu_nanoseconds,
                    device_totals.wait_nanoseconds,
                ),
            });
            break;
        };
        let published = run.versions_published.last();
        let intent_remains = published.is_some_and(|version| !version.logged_operations.is_empty());
        let relocated: Vec<&singlefs_core::transaction::UnitRelocatedByThisPublish> = published
            .map(|version| version.units_relocated_by_this_publish.iter().collect())
            .unwrap_or_default();
        let kind = kind_of_the_call(
            run.versions_published.len(),
            intent_remains,
            relocated.len(),
        );
        if kind == CallKind::Issue && reading.region.is_none() {
            reading.region = session
                .allocator
                .compaction()
                .live_intent()
                .map(|live| live.intent.region);
        }
        let destinations: Vec<(DeviceIdentity, u64, u64)> = relocated
            .iter()
            .flat_map(|unit| {
                let unit_bytes = unit_bytes_of_the_mapping_key(&unit.mapping_key);
                unit.destination.iter().map(move |location| {
                    (
                        location.device,
                        location.slot.to_device_offset().0,
                        unit_bytes,
                    )
                })
            })
            .collect();
        let (relocation_copy_bytes, is_every_destination_written_whole) =
            relocation_copy_bytes(&writes, &destinations);
        let relocated_source_slots: BTreeSet<u64> = relocated
            .iter()
            .flat_map(|unit| unit.source.iter().map(|location| location.slot.0))
            .collect();
        let ReadsOfOneCall {
            scan_reads_in_the_region,
            release_check_reads,
            scan_read_bytes_elsewhere,
            read_bytes_after_the_first_write,
        } = split_the_reads_of_a_call(&reads, reading.region, &relocated_source_slots);
        let scan_read_bytes_in_the_region: u64 = scan_reads_in_the_region
            .iter()
            .map(|read| read.length)
            .sum();
        let release_check_read_bytes: u64 =
            release_check_reads.iter().map(|read| read.length).sum();
        let expected_scan_read_bytes = match kind {
            CallKind::Issue | CallKind::Batch | CallKind::FinalBatch => reading
                .region
                .map(|region| expected_scan_read_bytes(&records_before, region)),
            CallKind::IntentDeletedWithoutABatch | CallKind::NoPublish => None,
        };
        if let (ReplayCapture::On, Some((version, state, region_bytes, copies_before))) =
            (capture, captured_before)
        {
            let release_check_copies: Vec<Vec<u8>> = release_check_reads
                .iter()
                .filter_map(|read| {
                    copies_before
                        .get(&(read.device, read.offset / LOCAL_SLOT_BYTES))
                        .filter(|bytes| length_in_bytes(bytes) == read.length)
                        .cloned()
                })
                .collect();
            let unit_bytes_by_slot = match kind {
                CallKind::Issue => reads
                    .iter()
                    .filter(|read| {
                        read.is_before_the_first_write && read.device == DeviceIdentity(0)
                    })
                    .filter_map(|read| {
                        let length = usize::try_from(read.length).expect("一次读的长度");
                        PoolReader::read(
                            devices.as_slice(),
                            read.device,
                            DeviceOffsetInBytes(read.offset),
                            length,
                        )
                        .map(|bytes| (read.offset / LOCAL_SLOT_BYTES, bytes))
                    })
                    .collect(),
                CallKind::Batch
                | CallKind::FinalBatch
                | CallKind::IntentDeletedWithoutABatch
                | CallKind::NoPublish => region_bytes,
            };
            reading.replay_material.push(CallReplayMaterial {
                call_index,
                kind,
                version,
                state,
                unit_bytes_by_slot,
                scan_reads_in_the_region: scan_reads_in_the_region.clone(),
                release_check_reads: release_check_reads.clone(),
                release_check_copies,
                relocated_keys: relocated
                    .iter()
                    .map(|unit| unit.mapping_key.clone())
                    .collect(),
            });
        }
        reading.calls.push(CallReading {
            kind,
            outcome: field_token(&format!("{:?}", run.outcome)),
            wall_nanoseconds,
            cpu_nanoseconds,
            is_the_cpu_unreadable: is_the_call_cpu_unreadable || is_the_device_cpu_unreadable,
            device_totals,
            per_method,
            scan_read_bytes_in_the_region,
            scan_read_bytes_elsewhere,
            release_check_read_bytes,
            read_bytes_after_the_first_write,
            write_bytes: writes.iter().map(|write| write.length).sum(),
            relocation_copy_bytes,
            is_every_destination_written_whole,
            relocated_units: u64::try_from(relocated.len()).expect("一批的单元数"),
            expected_scan_read_bytes,
            batches_observed: session.compaction.observations.batches_published - batches_before,
            inclusion: inclusion_verdict(
                wall_nanoseconds,
                cpu_nanoseconds,
                device_totals.wait_nanoseconds,
            ),
        });
        let is_the_intent_over = match kind {
            CallKind::Issue | CallKind::Batch => {
                matches!(run.outcome, CompactionRunOutcome::Failed(_))
            }
            CallKind::FinalBatch | CallKind::IntentDeletedWithoutABatch | CallKind::NoPublish => {
                true
            }
        };
        if is_the_intent_over {
            break;
        }
    }
    reading
}

// ───────────────────────────── 真盘臂的镜像（登记第五节：起始件、每轮拷回、核摘要） ─────────────────────────────

/// 一块 4 MiB 块的 SHA-256。
type ChunkDigest = [u8; 32];

fn image_chunk_length() -> usize {
    usize::try_from(IMAGE_CHUNK_BYTES).expect("4 MiB 装得进 usize")
}

/// 全 0 那一块的摘要（算一次）。
fn digest_of_a_zero_chunk() -> ChunkDigest {
    static DIGEST: std::sync::OnceLock<ChunkDigest> = std::sync::OnceLock::new();
    *DIGEST.get_or_init(|| sha256_digest(&vec![0_u8; image_chunk_length()]))
}

/// 镜像的分块摘要：每 4 MiB 一块的 SHA-256 依次接起来再算一次 SHA-256。任何一个字节不同，摘要就不同；内存盘与镜像文件按同一个定义算。
fn digest_of_the_chunk_digests(chunk_digests: &[ChunkDigest]) -> ChunkDigest {
    let joined: Vec<u8> = chunk_digests
        .iter()
        .flat_map(|digest| digest.iter().copied())
        .collect();
    sha256_digest(&joined)
}

fn hexadecimal_of(digest: &ChunkDigest) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 一块内存盘的分块摘要（没写过扇区的块就是全 0 块）。
fn digest_of_the_memory_device(device: &SparseBlockDevice) -> ChunkDigest {
    let mut buffer = vec![0_u8; image_chunk_length()];
    let chunk_digests: Vec<ChunkDigest> = (0..device.size_in_bytes() / IMAGE_CHUNK_BYTES)
        .map(|chunk| {
            let offset = DeviceOffsetInBytes(chunk * IMAGE_CHUNK_BYTES);
            if device
                .image
                .written_sectors_in(offset, IMAGE_CHUNK_BYTES)
                .is_empty()
            {
                digest_of_a_zero_chunk()
            } else {
                device.image.read_into(offset, &mut buffer);
                sha256_digest(&buffer)
            }
        })
        .collect();
    digest_of_the_chunk_digests(&chunk_digests)
}

/// 一个镜像文件的分块摘要。
fn digest_of_the_image_file(path: &Path) -> std::io::Result<ChunkDigest> {
    let file = File::open(path)?;
    let length = file.metadata()?.len();
    let mut buffer = vec![0_u8; image_chunk_length()];
    let mut chunk_digests = Vec::new();
    for chunk in 0..length / IMAGE_CHUNK_BYTES {
        file.read_exact_at(&mut buffer, chunk * IMAGE_CHUNK_BYTES)?;
        chunk_digests.push(if buffer.iter().all(|byte| *byte == 0) {
            digest_of_a_zero_chunk()
        } else {
            sha256_digest(&buffer)
        });
    }
    Ok(digest_of_the_chunk_digests(&chunk_digests))
}

/// 建一个整份写满零并 `fsync` 的镜像文件（不是稀疏、不是 `fallocate` 的未写区段），再核占用块数 × 512 ≥ 文件字节数
/// （登记第五节；`stat -c '%b %B %s'` 的同一组数）。
/// 镜像文件的文件名（不带目录）：报错进产物行的 `reason=`，目录是执行的那台机器的私有路径，不进产物。
fn name_of_the_image_file(path: &Path) -> String {
    path.file_name().map_or_else(
        || "unnamed_image".to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn create_a_zero_filled_image(path: &Path, bytes: u64) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let zeros = vec![0_u8; image_chunk_length()];
    for _chunk in 0..bytes / IMAGE_CHUNK_BYTES {
        file.write_all(&zeros)?;
    }
    file.sync_all()?;
    let metadata = file.metadata()?;
    if metadata.blocks() * 512 < metadata.len() {
        return Err(std::io::Error::other(format!(
            "{} 占用块数 {} × 512 小于文件字节数 {}：是稀疏文件",
            name_of_the_image_file(path),
            metadata.blocks(),
            metadata.len()
        )));
    }
    Ok(())
}

/// 把一块内存盘写过的扇区原样写进一个全零的镜像文件（相邻扇区并成一次写），再 `fsync`。
fn write_the_memory_device_into(path: &Path, device: &SparseBlockDevice) -> std::io::Result<()> {
    let file = OpenOptions::new().write(true).open(path)?;
    let sector_bytes = u64::from(PHYSICAL_BLOCK_BYTES);
    let mut run_start: Option<u64> = None;
    let mut run_bytes: Vec<u8> = Vec::new();
    for (sector, bytes) in device.image.written_sectors() {
        let offset = sector * sector_bytes;
        let continues_the_run =
            run_start.is_some_and(|start| start + length_in_bytes(&run_bytes) == offset);
        if !continues_the_run || run_bytes.len() >= image_chunk_length() {
            if let Some(start) = run_start {
                file.write_all_at(&run_bytes, start)?;
            }
            run_start = Some(offset);
            run_bytes.clear();
        }
        run_bytes.extend_from_slice(bytes);
    }
    if let Some(start) = run_start {
        file.write_all_at(&run_bytes, start)?;
    }
    file.sync_all()
}

/// 从起始件把点名的块拷回工作件，再 `fsync`；交回拷了几块。
fn restore_the_chunks(start: &Path, work: &Path, chunks: &BTreeSet<u64>) -> std::io::Result<u64> {
    let source = File::open(start)?;
    let destination = OpenOptions::new().write(true).open(work)?;
    let mut buffer = vec![0_u8; image_chunk_length()];
    for chunk in chunks {
        source.read_exact_at(&mut buffer, chunk * IMAGE_CHUNK_BYTES)?;
        destination.write_all_at(&buffer, chunk * IMAGE_CHUNK_BYTES)?;
    }
    destination.sync_all()?;
    Ok(u64::try_from(chunks.len()).expect("块数"))
}

/// Linux 的 `POSIX_FADV_DONTNEED`（`<linux/fadvise.h>` 里是 4）：把文件这一段干净的页缓存扔掉。
const POSIX_FADVISE_ADVICE_DONT_NEED: i32 = 4;

// 签名照 POSIX `int posix_fadvise(int fd, off_t offset, off_t len, int advice)`，Linux 64 位上 `off_t` 是 64 位；出错交回错误号（不设 errno）。
extern "C" {
    fn posix_fadvise(file_descriptor: i32, offset: i64, length: i64, advice: i32) -> i32;
}

/// 先 `fsync` 再把整个文件的页缓存扔掉（脏页扔不掉，所以先同步）。真盘臂每次测量之前对工作件做一次，量的那一刻工作件一页都不在页缓存里，
/// 与工作件在这之前被谁读过、内存紧不紧（准备阶段并行时同时攥着几份池）无关（登记第十二节 12.10）。
fn drop_the_page_cache_of_the_file(path: &Path) -> std::io::Result<()> {
    use std::os::fd::AsRawFd as _;
    let file = File::open(path)?;
    file.sync_all()?;
    // SAFETY: 文件描述符来自上一行打开、这一帧里还活着的 `file`；偏移与长度都是 0（整个文件），不传指针，内核不往本进程内存里写。
    let status = unsafe { posix_fadvise(file.as_raw_fd(), 0, 0, POSIX_FADVISE_ADVICE_DONT_NEED) };
    if status == 0 {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(status))
    }
}

/// 单元区之前（系统配置、根环、journal 环）写过的扇区抄进一块新的内存盘：真盘臂挂载时照它答 journal 记录偏移的提示。
fn image_of_the_area_before_the_units(
    device: &SparseBlockDevice,
    unit_area_start_slot: u64,
) -> SparseBlockDevice {
    let boundary = unit_area_start_slot * LOCAL_SLOT_BYTES;
    let sector_bytes = u64::from(PHYSICAL_BLOCK_BYTES);
    let mut copy =
        SparseBlockDevice::new(device.size_in_bytes(), device.probe_physical_block_size());
    for (sector, bytes) in device.image.written_sectors() {
        let offset = sector * sector_bytes;
        if offset < boundary {
            copy.image.write(DeviceOffsetInBytes(offset), bytes);
        }
    }
    copy
}

/// 一个取样点在真盘上的两份镜像：起始件（造好的池，写满零之后写进去）与工作件（每次测量之前把上一次改过的块从起始件拷回、核摘要）。
struct RealDiskImages {
    start_paths: Vec<PathBuf>,
    work_paths: Vec<PathBuf>,
    start_digests: Vec<ChunkDigest>,
    chunks_to_restore: Vec<BTreeSet<u64>>,
    hint_images: Vec<SparseBlockDevice>,
    unit_area_start_slot: u64,
}

impl RealDiskImages {
    /// 建起始件与工作件：两份都先整份写满零并 `fsync`，起始件再写进造好的池；核起始件的摘要与内存盘相同。
    fn create(
        directory: &Path,
        point: SamplingPoint,
        pool: &ConstructedPool,
    ) -> std::io::Result<RealDiskImages> {
        std::fs::create_dir_all(directory)?;
        let mut images = RealDiskImages {
            start_paths: Vec::new(),
            work_paths: Vec::new(),
            start_digests: Vec::new(),
            chunks_to_restore: Vec::new(),
            hint_images: Vec::new(),
            unit_area_start_slot: pool.unit_area_start_slot,
        };
        for (identity, device) in &pool.devices {
            let stem = format!(
                "e185-fraction-{}-pool-data-units-{}-device-{}",
                point.fraction.label(),
                point.pool_data_units,
                identity.0
            );
            let start_path = directory.join(format!("{stem}-start.img"));
            let work_path = directory.join(format!("{stem}-work.img"));
            for path in [&start_path, &work_path] {
                if path.exists() {
                    std::fs::remove_file(path)?;
                }
                create_a_zero_filled_image(path, device.size_in_bytes())?;
            }
            write_the_memory_device_into(&start_path, device)?;
            let start_digest = digest_of_the_image_file(&start_path)?;
            if start_digest != digest_of_the_memory_device(device) {
                return Err(std::io::Error::other(format!(
                    "起始件 {} 的摘要与内存盘不同",
                    name_of_the_image_file(&start_path)
                )));
            }
            images.start_paths.push(start_path);
            images.work_paths.push(work_path);
            images.start_digests.push(start_digest);
            images
                .chunks_to_restore
                .push((0..device.size_in_bytes() / IMAGE_CHUNK_BYTES).collect());
            images.hint_images.push(image_of_the_area_before_the_units(
                device,
                pool.unit_area_start_slot,
            ));
        }
        Ok(images)
    }

    /// 把上一次改过的块从起始件拷回工作件，再核工作件的摘要与起始件相同（登记第十一节「起始镜像不同」那一轮作废），最后扔掉工作件的页缓存
    /// （[`drop_the_page_cache_of_the_file`]）。交回各盘拷回的块数。
    fn restore_and_check(&mut self) -> Result<Vec<u64>, String> {
        let mut restored = Vec::new();
        for device_index in 0..self.work_paths.len() {
            let chunks = std::mem::take(&mut self.chunks_to_restore[device_index]);
            restored.push(
                restore_the_chunks(
                    &self.start_paths[device_index],
                    &self.work_paths[device_index],
                    &chunks,
                )
                .map_err(|error| field_token(&format!("restore_failed:{error}")))?,
            );
            let work_digest = digest_of_the_image_file(&self.work_paths[device_index])
                .map_err(|error| field_token(&format!("digest_failed:{error}")))?;
            if work_digest != self.start_digests[device_index] {
                self.chunks_to_restore[device_index] =
                    (0..LOCAL_DEVICE_BYTES / IMAGE_CHUNK_BYTES).collect();
                return Err("start_image_digest_differs".to_string());
            }
            drop_the_page_cache_of_the_file(&self.work_paths[device_index])
                .map_err(|error| field_token(&format!("page_cache_drop_failed:{error}")))?;
        }
        Ok(restored)
    }

    /// 以 `O_DIRECT` 打开两份工作件、套上计时包装与 journal 提示的照抄。
    fn open_timed(
        &self,
        ledger: &Rc<RefCell<CallLedger>>,
    ) -> Result<
        Vec<(
            DeviceIdentity,
            TimedBlockDevice<DirectInputOutputBlockDevice>,
        )>,
        String,
    > {
        let mut devices = Vec::new();
        for (device_index, path) in self.work_paths.iter().enumerate() {
            let identity = DeviceIdentity(u32::try_from(device_index).expect("两块盘"));
            let inner = DirectInputOutputBlockDevice::open_existing(
                path,
                PageCachePolicy::BypassWithDirectInputOutput,
                PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)),
            )
            .map_err(|error| field_token(&format!("open_failed:{error:?}")))?;
            let mut device = TimedBlockDevice::new(inner, identity, ledger);
            let mut hint_image = SparseBlockDevice::new(
                self.hint_images[device_index].size_in_bytes(),
                self.hint_images[device_index].probe_physical_block_size(),
            );
            hint_image.image = self.hint_images[device_index].image.clone();
            device.journal_hint_mirror = Some(JournalHintMirror {
                image: hint_image,
                below_offset: self.unit_area_start_slot * LOCAL_SLOT_BYTES,
                is_mirroring: true,
            });
            devices.push((identity, device));
        }
        Ok(devices)
    }

    /// 一次测量之后，把它改过的块记下，下一次之前拷回。
    fn note_the_changed_chunks(&mut self, ledger: &Rc<RefCell<CallLedger>>) {
        let changed = std::mem::take(&mut ledger.borrow_mut().chunks_written_since_the_copy);
        for (device, chunk) in changed {
            let device_index = usize::try_from(device.0).expect("盘号装得进 usize");
            self.chunks_to_restore[device_index].insert(chunk);
        }
    }

    /// 一个取样点量完删掉两份镜像（草稿目录里自己建的东西）。
    fn remove(self) -> std::io::Result<()> {
        for path in self.start_paths.iter().chain(&self.work_paths) {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

/// 在造好的池的一份拷贝上量一条意图（内存臂）。
fn measure_on_the_memory_device(
    pool: &ConstructedPool,
    switches: CompactionSwitches,
    injection: WriteInjection,
    capture: ReplayCapture,
) -> IntentReading {
    let ledger = Rc::new(RefCell::new(CallLedger::new(injection)));
    let mut devices = timed_devices(copy_of_the_memory_devices(&pool.devices), &ledger);
    measure_one_intent(&mut devices, &ledger, switches, capture)
}

/// 在真盘上量一条意图：工作件拷回、核摘要、打开、量、记下改过的块。拷回或打开不成交回原因（那一轮作废）。
fn measure_on_the_real_disk(
    images: &mut RealDiskImages,
    switches: CompactionSwitches,
    injection: WriteInjection,
) -> Result<IntentReading, String> {
    images.restore_and_check()?;
    let ledger = Rc::new(RefCell::new(CallLedger::new(injection)));
    let mut devices = images.open_timed(&ledger)?;
    let reading = measure_one_intent(&mut devices, &ledger, switches, ReplayCapture::Off);
    drop(devices);
    images.note_the_changed_chunks(&ledger);
    Ok(reading)
}

// ───────────────────────────── 一条意图的作废、停机与产物行 ─────────────────────────────

/// 一条意图的读数要不要作废（登记第十一节作废条款里「那一轮作废、重跑」那几条）；交回第一条原因。
fn invalid_reason_of(reading: &IntentReading) -> Option<&'static str> {
    if reading.mount_failure.is_some() {
        return Some("mount_failed");
    }
    if reading.calls.iter().any(|call| call.is_the_cpu_unreadable) {
        return Some("thread_cpu_clock_unreadable");
    }
    if reading
        .calls
        .iter()
        .any(|call| call.inclusion == InclusionVerdict::UnaccountedAboveTenPercent)
    {
        return Some("unaccounted_time_above_ten_percent_of_the_wall");
    }
    if reading.calls.iter().any(|call| {
        call.scan_read_bytes_in_the_region == 0
            && call
                .expected_scan_read_bytes
                .is_some_and(|expected| expected > 0)
    }) {
        return Some("no_scan_read_while_the_region_holds_live_units");
    }
    None
}

/// 停机条款第 2、3、5、6 条在一条意图上的判定（第 1 条在开跑前、第 4 条在第二段）：交回触发了的那几条，每条一句。
/// `data_units` 是几何造成了时 R 里的数据单元数 m（没造成交 `None`：第 3 条的 ⌈m ÷ K⌉ 与第 5 条只在几何造成了的点上判）。
fn stop_findings_of(
    reading: &IntentReading,
    data_units: Option<u64>,
    batch_units: u64,
) -> Vec<String> {
    let mut findings = Vec::new();
    for (call_index, call) in reading.calls.iter().enumerate() {
        if call.inclusion == InclusionVerdict::CpuPlusWaitAboveTheWall {
            findings.push(format!(
                "clause_6_cpu_plus_wait_above_the_wall_at_call_{call_index}:wall={}:cpu={}:wait={}",
                call.wall_nanoseconds,
                call.cpu_nanoseconds,
                call.wait_nanoseconds()
            ));
        }
        if let Some(expected) = call.expected_scan_read_bytes {
            if call.scan_read_bytes_in_the_region != expected {
                findings.push(format!(
                    "clause_2_scan_read_bytes_at_call_{call_index}:read={}:records={expected}",
                    call.scan_read_bytes_in_the_region
                ));
            }
        }
    }
    let batches_counted = u64::try_from(
        reading
            .calls
            .iter()
            .filter(|call| counts_as_a_batch(call.kind))
            .count(),
    )
    .expect("批数");
    let batches_observed: u64 = reading.calls.iter().map(|call| call.batches_observed).sum();
    if batches_counted != batches_observed {
        findings.push(format!(
            "clause_3_batches_counted_by_the_device_{batches_counted}_observed_by_the_session_{batches_observed}"
        ));
    }
    if let Some(data_units) = data_units {
        let expected_batches = batches_of_one_intent(data_units, batch_units);
        if batches_counted != expected_batches {
            findings.push(format!(
                "clause_3_batches_{batches_counted}_not_the_ceiling_of_m_over_k_{expected_batches}"
            ));
        }
        if let Some(first_batch) = reading
            .calls
            .iter()
            .find(|call| counts_as_a_batch(call.kind))
        {
            let expected_copy = first_batch_copy_bytes(data_units, batch_units);
            if first_batch.relocation_copy_bytes != expected_copy
                || !first_batch.is_every_destination_written_whole
            {
                findings.push(format!(
                    "clause_5_first_batch_copy_bytes_{}_expected_{expected_copy}_destinations_whole_{}",
                    first_batch.relocation_copy_bytes, first_batch.is_every_destination_written_whole
                ));
            }
        }
    }
    findings
}

/// 一次测量是在哪儿、怎么量的（产物行的前缀）。
#[derive(Clone, Copy, Debug)]
struct MeasurementLabel {
    arm: DeviceArm,
    point: SamplingPoint,
    batch_units: u64,
    round: usize,
    attempt: usize,
    injection: WriteInjection,
    choice: ChoiceOfTheRegion,
}

impl MeasurementLabel {
    fn text(&self) -> String {
        format!(
            "arm={} {} batch_units={} round={} attempt={} injection={} choice={}",
            self.arm.label(),
            self.point.label(),
            self.batch_units,
            self.round,
            self.attempt,
            self.injection.label(),
            self.choice.label()
        )
    }
}

/// 一条意图的产物行：每次调用一行 `name=call`（含各种设备调用的累计），一行 `name=intent`。
fn emit_the_intent(
    emitter: &mut Emitter,
    label: &MeasurementLabel,
    reading: &IntentReading,
    invalid_reason: Option<&str>,
) {
    let region_text = reading.region.map_or_else(
        || "none".to_string(),
        |region| region.start_slot().0.to_string(),
    );
    for (call_index, call) in reading.calls.iter().enumerate() {
        let methods: String = DeviceMethod::ALL
            .iter()
            .map(|method| {
                let totals = call.per_method.get(method).copied().unwrap_or_default();
                format!(
                    " {name}_calls={} {name}_bytes={} {name}_wall_nanoseconds={} {name}_cpu_nanoseconds={} {name}_wait_nanoseconds={}",
                    totals.calls,
                    totals.bytes,
                    totals.wall_nanoseconds,
                    totals.cpu_nanoseconds,
                    totals.wait_nanoseconds,
                    name = method.label()
                )
            })
            .collect();
        emitter.line(&format!(
            "name=call {} call_index={call_index} kind={} region_start_slot={region_text} wall_nanoseconds={} cpu_nanoseconds={} wait_nanoseconds={} device_wall_nanoseconds={} device_cpu_nanoseconds={} scan_read_bytes_in_the_region={} expected_scan_read_bytes={} scan_read_bytes_elsewhere={} release_check_read_bytes={} read_bytes_after_the_first_write={} write_bytes={} relocation_copy_bytes={} metadata_write_bytes={} destinations_written_whole={} relocated_units={} batches_observed={} inclusion={} cpu_unreadable={} outcome={}{methods}",
            label.text(),
            call.kind.label(),
            call.wall_nanoseconds,
            call.cpu_nanoseconds,
            call.wait_nanoseconds(),
            call.device_totals.wall_nanoseconds,
            call.device_totals.cpu_nanoseconds,
            call.scan_read_bytes_in_the_region,
            call.expected_scan_read_bytes
                .map_or_else(|| "none".to_string(), |expected| expected.to_string()),
            call.scan_read_bytes_elsewhere,
            call.release_check_read_bytes,
            call.read_bytes_after_the_first_write,
            call.write_bytes,
            call.relocation_copy_bytes,
            call.write_bytes - call.relocation_copy_bytes.min(call.write_bytes),
            call.is_every_destination_written_whole,
            call.relocated_units,
            call.batches_observed,
            call.inclusion.label(),
            call.is_the_cpu_unreadable,
            call.outcome,
        ));
    }
    emitter.line(&format!(
        "name=intent {} region_start_slot={region_text} calls={} batches={} valid={} invalid_reason={} mount_failure={}",
        label.text(),
        reading.calls.len(),
        reading.calls.iter().filter(|call| counts_as_a_batch(call.kind)).count(),
        invalid_reason.is_none(),
        invalid_reason.unwrap_or("none"),
        reading.mount_failure.as_deref().unwrap_or("none"),
    ));
}

// ───────────────────────────── 第一段 wall-clock：测量 ─────────────────────────────

/// 存下来的一条有效意图（汇总用）。
struct StoredIntent {
    arm: DeviceArm,
    point: SamplingPoint,
    batch_units: u64,
    round: usize,
    injection: WriteInjection,
    is_the_geometry_formed: bool,
    calls: Vec<CallReading>,
}

impl StoredIntent {
    fn batch_calls(&self) -> impl Iterator<Item = &CallReading> {
        self.calls
            .iter()
            .filter(|call| counts_as_a_batch(call.kind))
    }

    fn batch_cpu_nanoseconds(&self) -> f64 {
        self.batch_calls()
            .map(|call| call.cpu_nanoseconds as f64)
            .sum()
    }

    fn batch_wait_nanoseconds(&self) -> f64 {
        self.batch_calls()
            .map(|call| call.wait_nanoseconds() as f64)
            .sum()
    }

    fn batch_wall_nanoseconds(&self) -> f64 {
        self.batch_calls()
            .map(|call| call.wall_nanoseconds as f64)
            .sum()
    }

    fn batch_write_bytes(&self) -> u64 {
        self.batch_calls().map(|call| call.write_bytes).sum()
    }

    fn batch_relocation_copy_bytes(&self) -> u64 {
        self.batch_calls()
            .map(|call| call.relocation_copy_bytes)
            .sum()
    }

    fn first_batch_wall_nanoseconds(&self) -> Option<f64> {
        self.batch_calls()
            .next()
            .map(|call| call.wall_nanoseconds as f64)
    }

    /// 整条意图（签发与各批）的 CPU、等盘与写调用次数：阳性对照的注入在每次 `write_at` 里，按整条意图比。
    fn intent_cpu_nanoseconds(&self) -> f64 {
        self.calls
            .iter()
            .map(|call| call.cpu_nanoseconds as f64)
            .sum()
    }

    fn intent_wait_nanoseconds(&self) -> f64 {
        self.calls
            .iter()
            .map(|call| call.wait_nanoseconds() as f64)
            .sum()
    }

    fn intent_write_calls(&self) -> f64 {
        self.calls
            .iter()
            .map(|call| {
                call.per_method
                    .get(&DeviceMethod::Write)
                    .map_or(0, |totals| totals.calls) as f64
            })
            .sum()
    }

    /// 整条意图里写调用（`write_at`，阳性对照注入所在的那一种调用）的等盘之和（登记第十二节 12.9）。
    fn intent_write_wait_nanoseconds(&self) -> f64 {
        self.calls
            .iter()
            .map(|call| {
                call.per_method
                    .get(&DeviceMethod::Write)
                    .map_or(0, |totals| totals.wait_nanoseconds) as f64
            })
            .sum()
    }

    fn row_one_control_round(&self) -> RowOneControlRound {
        RowOneControlRound {
            round: self.round,
            cpu_nanoseconds: self.intent_cpu_nanoseconds(),
            wait_nanoseconds: self.intent_wait_nanoseconds(),
            write_calls: self.intent_write_calls(),
            write_wait_nanoseconds: self.intent_write_wait_nanoseconds(),
        }
    }
}

/// 一个取样点、一个设备、一个批大小、一轮（或对照的一轮）：照登记重跑至多三次，作废的照打 `valid=false`，第一份有效的存下；
/// 交回停机条款触发了的那几条。
fn measure_with_retries(
    emitter: &mut Emitter,
    stored: &mut Vec<StoredIntent>,
    images: Option<&mut RealDiskImages>,
    label: MeasurementLabel,
    is_the_geometry_formed: bool,
    data_units: Option<u64>,
    pool_of_the_round: Option<&ConstructedPool>,
) -> Vec<String> {
    let switches = measured_switches(label.batch_units, label.choice.forced_region());
    let mut images = images;
    for attempt in 1..=ATTEMPTS_PER_ROUND_AT_MOST {
        enter_the_timing_window(emitter, TimingWindow::WallClockCompactionRun);
        let mut attempt_label = label;
        attempt_label.attempt = attempt;
        let reading = match (label.arm, pool_of_the_round, images.as_deref_mut()) {
            (DeviceArm::Memory, Some(pool), _) => Ok(measure_on_the_memory_device(
                pool,
                switches,
                label.injection,
                ReplayCapture::Off,
            )),
            (DeviceArm::RealDisk, _, Some(real_disk_images)) => {
                measure_on_the_real_disk(real_disk_images, switches, label.injection)
            }
            (DeviceArm::Memory, None, _) | (DeviceArm::RealDisk, _, None) => {
                Err("no_device_for_this_arm".to_string())
            }
        };
        let reading = match reading {
            Ok(reading) => reading,
            Err(reason) => {
                emitter.line(&format!(
                    "name=intent {} valid=false invalid_reason={reason}",
                    attempt_label.text()
                ));
                continue;
            }
        };
        let invalid_reason = invalid_reason_of(&reading);
        emit_the_intent(emitter, &attempt_label, &reading, invalid_reason);
        let findings = stop_findings_of(&reading, data_units, label.batch_units);
        for finding in &findings {
            emitter.line(&format!(
                "name=stop_clause {} finding={finding}",
                attempt_label.text()
            ));
        }
        if !findings.is_empty() {
            return findings;
        }
        if invalid_reason.is_none() {
            stored.push(StoredIntent {
                arm: label.arm,
                point: label.point,
                batch_units: label.batch_units,
                round: label.round,
                injection: label.injection,
                is_the_geometry_formed,
                calls: reading.calls,
            });
            return Vec::new();
        }
    }
    emitter.line(&format!(
        "name=round_invalid {} attempts={ATTEMPTS_PER_ROUND_AT_MOST}",
        label.text()
    ));
    Vec::new()
}

/// 每块盘的分块摘要（内存盘）。
fn digests_of_the_pool(pool: &ConstructedPool) -> Vec<ChunkDigest> {
    pool.devices
        .iter()
        .map(|(_identity, device)| digest_of_the_memory_device(device))
        .collect()
}

// ───────────────────────────── 不计时的准备并行（登记第十二节 12.10「并行与挂钟」） ─────────────────────────────

const MEMORY_CAP_ENVIRONMENT_VARIABLE: &str = "E185_MEMORY_CAP";
/// 环境变量与本进程控制组的 `memory.max` 都没给时的内存上限：第二台起跑命令给 `run-with-memory-cap.sh` 的 28G。
const DEFAULT_MEMORY_CAP_BYTES: u64 = 28 << 30;
/// 内存盘每写过一个 512 字节扇区占的内存上界（推的）：扇区本身 512 字节、堆分配头 16 字节、`Vec` 头 24 字节、键 8 字节，
/// 再加 B 树节点按半满摊到每个条目的约 64 字节，凑整到 640。
const MEMORY_BYTES_PER_WRITTEN_SECTOR_AT_MOST: u64 = 640;
/// 造池之前估一份池的内存时，元数据（节点、journal、根环）按数据单元字节的同等量留（推的）；造完之后按真写过的扇区数算，不再用它。
const METADATA_BYTES_PER_DATA_BYTE_AT_MOST: u64 = 1;
/// 准备一个取样点时同时攥着几份池：造好的那一份，与几何核时可写挂载用的拷贝（[`predict_the_greedy_choice`]）。
const POOLS_HELD_WHILE_PREPARING_A_SAMPLING_POINT: u64 = 2;
/// 重造一份池的峰值按池的 5/4 算：池本身，加建池会话的版本、分配器与树（推的）。
const REBUILD_PEAK_NUMERATOR: u64 = 5;
const REBUILD_PEAK_DENOMINATOR: u64 = 4;
/// 准备之后的串行测量另要的余量，按那一点的池几份算：内存臂每次测量拷一份池，另有重放材料与镜像提示的工作集（推的）。
const POOLS_OF_HEADROOM_FOR_THE_MEASUREMENT: u64 = 2;

/// 内存上限从哪取。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemoryCapSource {
    Environment,
    ControlGroup,
    Default,
}

impl MemoryCapSource {
    const fn label(self) -> &'static str {
        match self {
            MemoryCapSource::Environment => "environment_variable",
            MemoryCapSource::ControlGroup => "control_group_memory_limit",
            MemoryCapSource::Default => "default_twenty_eight_gibibytes",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MemoryCap {
    bytes: u64,
    source: MemoryCapSource,
}

/// `28G`、`512M` 这种 systemd 的 MemoryMax 写法（1024 进制），或不带单位的字节数（控制组 `memory.max` 的写法）；零与读不懂的交回 `None`。
fn bytes_of_a_memory_size_text(text: &str) -> Option<u64> {
    let trimmed = text.trim();
    let units: [(char, u64); 4] = [
        ('K', 1 << 10),
        ('M', 1 << 20),
        ('G', 1 << 30),
        ('T', 1 << 40),
    ];
    let (digits, bytes_per_unit) = units
        .iter()
        .find_map(|(suffix, bytes_per_unit)| {
            trimmed
                .strip_suffix(*suffix)
                .map(|digits| (digits, *bytes_per_unit))
        })
        .unwrap_or((trimmed, 1));
    digits
        .parse::<u64>()
        .ok()
        .filter(|count| *count > 0)?
        .checked_mul(bytes_per_unit)
}

/// 内存上限的取法：`E185_MEMORY_CAP` 设了照它，没设取本进程控制组的 `memory.max`（`run-with-memory-cap.sh` 给的上限就落在那里），
/// 那里是 `max` 或读不到时取 28G。
///
/// # Panics
/// `E185_MEMORY_CAP` 设了而读不懂。
fn memory_cap_given(
    environment_value: Option<&str>,
    control_group_memory_limit_text: Option<&str>,
) -> MemoryCap {
    if let Some(value) = environment_value {
        let bytes = bytes_of_a_memory_size_text(value).unwrap_or_else(|| {
            panic!("{MEMORY_CAP_ENVIRONMENT_VARIABLE} 要写成正整数加 K / M / G / T，或字节数，拿到「{value}」")
        });
        return MemoryCap {
            bytes,
            source: MemoryCapSource::Environment,
        };
    }
    match control_group_memory_limit_text.and_then(bytes_of_a_memory_size_text) {
        Some(bytes) => MemoryCap {
            bytes,
            source: MemoryCapSource::ControlGroup,
        },
        None => MemoryCap {
            bytes: DEFAULT_MEMORY_CAP_BYTES,
            source: MemoryCapSource::Default,
        },
    }
}

/// 本进程所在控制组（cgroup v2，`/proc/self/cgroup` 里 `0::` 那一行）的 `memory.max` 原文。
fn control_group_memory_limit_of_this_process() -> Option<String> {
    let membership = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let group = membership
        .lines()
        .find_map(|line| line.strip_prefix("0::"))?;
    std::fs::read_to_string(format!("/sys/fs/cgroup{group}/memory.max")).ok()
}

/// 准备阶段的设定：线程数（与饱和吞吐的 N 同一个取法）与内存上限。
struct PreparationSettings {
    threads: usize,
    memory_cap: MemoryCap,
}

impl PreparationSettings {
    fn from_the_environment() -> PreparationSettings {
        PreparationSettings {
            threads: worker_threads_from_the_environment(),
            memory_cap: memory_cap_given(
                std::env::var(MEMORY_CAP_ENVIRONMENT_VARIABLE)
                    .ok()
                    .as_deref(),
                control_group_memory_limit_of_this_process().as_deref(),
            ),
        }
    }
}

/// 造池之前估的一份池的内存（字节）：两块盘、每个数据单元至多写两次（建池一次、覆盖写一次），加多建的空 inode，元数据按同等量留，
/// 再按写过的扇区数折成内存。登记的两点：P = 1024 是 346030080，P = 8192 是 2694840320。
fn estimated_pool_footprint_bytes_before_construction(point: SamplingPoint) -> u64 {
    let data_bytes = DEVICES_HOLDING_A_COPY
        * (2 * point.pool_data_units + SPARE_INODES_FOR_THE_FRONTIER)
        * LOCAL_DATA_UNIT_BYTES;
    let written_bytes = data_bytes * (1 + METADATA_BYTES_PER_DATA_BYTE_AT_MOST);
    written_bytes / u64::from(PHYSICAL_BLOCK_BYTES) * MEMORY_BYTES_PER_WRITTEN_SECTOR_AT_MOST
}

/// 造好的池的内存（字节）：两块内存盘写过的扇区数 × 每扇区的上界。
fn footprint_bytes_of_the_pool(pool: &ConstructedPool) -> u64 {
    pool.devices
        .iter()
        .map(|(_identity, device)| {
            u64::try_from(device.image.written_sectors().len()).expect("扇区数装得进 u64")
        })
        .sum::<u64>()
        * MEMORY_BYTES_PER_WRITTEN_SECTOR_AT_MOST
}

/// 这一刻还能再派的内存：上限减去已经记在账上的（在跑的各份按峰值、造好还没取走的池按真写过的扇区）与之后串行测量要的余量。
fn preparation_budget_bytes(memory_cap_bytes: u64, held_bytes: u64, headroom_bytes: u64) -> u64 {
    memory_cap_bytes.saturating_sub(held_bytes.saturating_add(headroom_bytes))
}

/// 这一刻能再派几份：按次序往里加，份数不超过空着的线程数、各份峰值之和不超过还能派的内存；装不下的那一份之后的都不派
/// （不跳过它去派后面的：免得后量的点占住内存、先量的点一直等）。各份峰值相同时就是 min(线程数, 预算 ÷ 一份峰值)。
/// 一份都装不下时的保底不在这里，在 [`PreparationQueue::units_to_start`]：什么都没在跑、要的那一份还没好时才保底一份。
fn tasks_of_the_next_group(
    peak_bytes_in_order: &[u64],
    threads: usize,
    budget_bytes: u64,
) -> usize {
    let mut total_bytes = 0_u64;
    peak_bytes_in_order
        .iter()
        .take(threads)
        .take_while(|peak_bytes| {
            total_bytes = total_bytes.saturating_add(**peak_bytes);
            total_bytes <= budget_bytes
        })
        .count()
}

/// `/proc/self/status` 的 `Threads:` 那一行：本进程此刻的线程数。
fn threads_in_the_process_status(status_text: &str) -> Option<usize> {
    status_text
        .lines()
        .find_map(|line| line.strip_prefix("Threads:"))
        .and_then(|count| count.trim().parse().ok())
}

/// 本进程只剩量的那一条线程。
fn is_only_the_measuring_thread_left(status_text: &str) -> bool {
    threads_in_the_process_status(status_text) == Some(1)
}

/// 等准备线程从 `/proc/self/task` 里消失至多看几次（每毫秒一次）：join 交回之后，退出的线程消失得比它晚一点。
const LOOKS_FOR_THE_PREPARATION_THREADS_TO_LEAVE_AT_MOST: u32 = 1000;

/// 计时窗口（每次 `run_compaction`、设备顺序写带宽、第二段的阶段重放与 2 ms 对照）开始之前，本进程只剩一条线程（饱和吞吐自己起的线程除外，
/// 它在自己的计时里）。
///
/// # Panics
/// 等满仍不是一条（准备线程漏进了计时窗口），或读不到 `/proc/self/status`。
fn assert_this_process_runs_one_thread(moment: &str) {
    let mut status_text = None;
    for _look in 0..LOOKS_FOR_THE_PREPARATION_THREADS_TO_LEAVE_AT_MOST {
        status_text = std::fs::read_to_string("/proc/self/status").ok();
        if status_text
            .as_deref()
            .is_some_and(is_only_the_measuring_thread_left)
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!(
        "计时窗口（{moment}）开始之前本进程应只剩一条线程，读到 {:?} 条",
        status_text
            .as_deref()
            .and_then(threads_in_the_process_status)
    );
}

/// 计时窗口（登记第十二节 12.10 ②、12.11 串行声明）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimingWindow {
    WallClockCompactionRun,
    DeviceSequentialWrite,
    StageSplitCompactionRun,
    StageReplay,
    StageControls,
    DiskSizeSensitivity,
}

impl TimingWindow {
    #[cfg_attr(not(test), allow(dead_code, reason = "只有单测逐个核各窗口的理由"))]
    const ALL: [TimingWindow; 6] = [
        TimingWindow::WallClockCompactionRun,
        TimingWindow::DeviceSequentialWrite,
        TimingWindow::StageSplitCompactionRun,
        TimingWindow::StageReplay,
        TimingWindow::StageControls,
        TimingWindow::DiskSizeSensitivity,
    ];

    /// 线程核没过时 panic 消息里点名的那一处。
    const fn moment(self) -> &'static str {
        match self {
            TimingWindow::WallClockCompactionRun | TimingWindow::StageSplitCompactionRun => {
                "run_compaction"
            }
            TimingWindow::DeviceSequentialWrite => "device_sequential_write",
            TimingWindow::StageReplay => "stage_replay",
            TimingWindow::StageControls => "stage_controls",
            TimingWindow::DiskSizeSensitivity => "disk_size_sensitivity",
        }
    }

    /// 声明串行段的理由（`scripts/hosts/core_usage.py` 文件头「声明」：至少 8 个字）。第二段的几种窗口共用一句，挨着的窗口之间不另写一行。
    const fn serial_reason(self) -> &'static str {
        match self {
            TimingWindow::WallClockCompactionRun => {
                "E185 计时窗口：第一段一批搬迁的挂钟、CPU 与等盘要在没有别的线程的进程里量"
            }
            TimingWindow::DeviceSequentialWrite => {
                "E185 计时窗口：设备顺序写带宽 Q15 要在没有别的线程的进程里量"
            }
            TimingWindow::StageSplitCompactionRun
            | TimingWindow::StageReplay
            | TimingWindow::StageControls
            | TimingWindow::DiskSizeSensitivity => {
                "E185 计时窗口：第二段的一批搬迁、阶段重放、2 ms 对照与盘大小敏感性要在没有别的线程的进程里量"
            }
        }
    }
}

/// 进一个计时窗口：本进程只剩一条线程（[`assert_this_process_runs_one_thread`]），再当场声明串行段（[`Emitter::declare`]）。
fn enter_the_timing_window(emitter: &mut Emitter, window: TimingWindow) {
    assert_this_process_runs_one_thread(window.moment());
    emitter.declare(DeclaredSegment::Serial(window.serial_reason()));
}

// ───────────────────────────── 不计时的准备：一个全局队列（登记第十二节 12.11） ─────────────────────────────

/// 一份不计时的准备。封闭集合。派生的 `Ord` 只给记账的表当键用；派活的次序是 [`preparation_units_in_measurement_order`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PreparationUnit {
    /// 一个取样点的造池与几何核（[`prepare_the_sampling_point`]；同一点的几次改负载重造前后依赖，在一条线程上按次序）。
    PrepareThePoint { point_index: usize },
    /// 一个取样点的真盘镜像（第一段）：起始件与工作件写满零、写进池、核摘要。
    BuildTheRealDiskImages { point_index: usize },
    /// 一个取样点第 `round` 轮（2–5）的内存池重造。
    RebuildForTheRound { point_index: usize, round: usize },
}

impl PreparationUnit {
    const fn point_index(self) -> usize {
        match self {
            PreparationUnit::PrepareThePoint { point_index }
            | PreparationUnit::BuildTheRealDiskImages { point_index }
            | PreparationUnit::RebuildForTheRound { point_index, .. } => point_index,
        }
    }

    /// 产物行里点名这一份：`prepare:<活槽占比>:<池的数据量>`、`real_disk_images:…`、`rebuild:…:<轮>`。
    fn label(self, points: &[SamplingPoint]) -> String {
        let point = points[self.point_index()];
        let point_text = format!("{}:{}", point.fraction.label(), point.pool_data_units);
        match self {
            PreparationUnit::PrepareThePoint { .. } => format!("prepare:{point_text}"),
            PreparationUnit::BuildTheRealDiskImages { .. } => {
                format!("real_disk_images:{point_text}")
            }
            PreparationUnit::RebuildForTheRound { round, .. } => {
                format!("rebuild:{point_text}:{round}")
            }
        }
    }
}

/// 一段里全部准备单位，按量的次序：每个取样点先造池与几何核，再建真盘镜像（第一段），再第 2–5 轮重造。
/// 内存紧时按这个次序派（[`tasks_of_the_next_group`]），先量的先造好。
fn preparation_units_in_measurement_order(
    points: usize,
    with_real_disk_images: bool,
) -> Vec<PreparationUnit> {
    (0..points)
        .flat_map(|point_index| {
            std::iter::once(PreparationUnit::PrepareThePoint { point_index })
                .chain(
                    with_real_disk_images
                        .then_some(PreparationUnit::BuildTheRealDiskImages { point_index }),
                )
                .chain(
                    (2..=ROUNDS).map(move |round| PreparationUnit::RebuildForTheRound {
                        point_index,
                        round,
                    }),
                )
        })
        .collect()
}

/// 派给一条准备线程的东西：都是它自有的值或共享只读的，线程之间不共享可变的东西。
enum PreparationWork {
    PrepareThePoint(SamplingPoint),
    BuildTheRealDiskImages {
        directory: PathBuf,
        prepared: Arc<PreparedPoint>,
    },
    Rebuild(PoolRecipe),
}

/// 重造的一份池：用的配方（那一点还没造好时是头一次的配方，推测着先造）、池与它的分块摘要。
struct RebuiltPool {
    recipe: PoolRecipe,
    pool: ConstructedPool,
    digests: Vec<ChunkDigest>,
}

/// 一份准备的结果。池的内存（[`footprint_bytes_of_the_pool`]）在准备线程上算好带回来。真盘镜像建不成的原因已经写成产物行的字段。
enum PreparationOutcome {
    Prepared {
        prepared: PreparedPoint,
        footprint_bytes: u64,
    },
    RealDiskImages(Result<RealDiskImages, String>),
    Rebuilt {
        rebuilt: RebuiltPool,
        footprint_bytes: u64,
    },
}

/// 头一次的配方：只有取样点，没有改负载多覆盖写的 inode。几何核还没定时，重造按它推测着先造。
fn first_recipe_of(point: SamplingPoint) -> PoolRecipe {
    PoolRecipe {
        point,
        extra_overwritten_inodes: BTreeSet::new(),
    }
}

/// 真做一份准备（[`PreparationQueue`] 平常派的就是它；单测换成假的）。
fn run_the_preparation_work(work: PreparationWork) -> PreparationOutcome {
    match work {
        PreparationWork::PrepareThePoint(point) => {
            let prepared = prepare_the_sampling_point(point);
            let footprint_bytes = footprint_bytes_of_the_pool(&prepared.pool);
            PreparationOutcome::Prepared {
                prepared,
                footprint_bytes,
            }
        }
        PreparationWork::BuildTheRealDiskImages {
            directory,
            prepared,
        } => PreparationOutcome::RealDiskImages(
            RealDiskImages::create(&directory, prepared.recipe.point, &prepared.pool)
                .map_err(|error| field_token(&error.to_string())),
        ),
        PreparationWork::Rebuild(recipe) => {
            let pool = construct_the_pool(&recipe);
            let digests = digests_of_the_pool(&pool);
            let footprint_bytes = footprint_bytes_of_the_pool(&pool);
            PreparationOutcome::Rebuilt {
                rebuilt: RebuiltPool {
                    recipe,
                    pool,
                    digests,
                },
                footprint_bytes,
            }
        }
    }
}

/// 派准备的那个函数：平常是 [`run_the_preparation_work`]，单测换成假的。
type PreparationWorker<'context> = &'context (dyn Fn(PreparationWork) -> PreparationOutcome + Sync);

/// 一份准备走到哪了。
enum UnitProgress<Outcome> {
    Waiting,
    Running,
    Done(Outcome),
    /// 量的那一侧取走了（或那一点量完、放掉了）。
    Taken,
}

/// 队列里一个取样点的几份准备。
struct PointInTheQueue {
    point: SamplingPoint,
    prepared: UnitProgress<Arc<PreparedPoint>>,
    real_disk_images: UnitProgress<Result<RealDiskImages, String>>,
    rebuilt_by_round: BTreeMap<usize, UnitProgress<RebuiltPool>>,
    is_measured: bool,
    speculative_rebuilds_voided: usize,
}

/// 一段里全部不计时的准备：一个队列，按量的次序派到 min(线程数, 内存预算 ÷ 一份峰值) 条线程上（登记第十二节 12.11）。
/// 量的那一侧要哪一份就取哪一份（[`PreparationQueue::prepared_point`]、[`PreparationQueue::take_the_rebuilt_pool`]、
/// [`PreparationQueue::take_the_real_disk_images`]），还没好就跑一次准备（[`run_the_preparation_phase`]）：内存装得下全部就一次全派出去，
/// 装不下就派到装满为止，要的那一份好了就不再派新的，等在跑的都停下交回；之后量的那一侧串行量，下一次缺了再跑一次（批内并行、批间串行量）。
struct PreparationQueue<'context> {
    segment: &'static str,
    settings: &'context PreparationSettings,
    worker: PreparationWorker<'context>,
    image_directory: Option<PathBuf>,
    points: Vec<PointInTheQueue>,
    order: Vec<PreparationUnit>,
    /// 记在账上的内存：在跑的按峰值，造好还没取走的按池的内存（[`footprint_bytes_of_the_pool`]）。
    accounted_bytes_by_unit: BTreeMap<PreparationUnit, u64>,
    footprint_bytes_of_the_prepared: BTreeMap<usize, u64>,
    phases: Vec<PreparationPhaseReading>,
}

impl<'context> PreparationQueue<'context> {
    /// 第一段带 `image_directory`（每点另有一份真盘镜像），第二段不带。
    fn new(
        segment: &'static str,
        points: &[SamplingPoint],
        image_directory: Option<&Path>,
        settings: &'context PreparationSettings,
        worker: PreparationWorker<'context>,
    ) -> Self {
        PreparationQueue {
            segment,
            settings,
            worker,
            image_directory: image_directory.map(Path::to_path_buf),
            points: points
                .iter()
                .map(|point| PointInTheQueue {
                    point: *point,
                    prepared: UnitProgress::Waiting,
                    real_disk_images: UnitProgress::Waiting,
                    rebuilt_by_round: (2..=ROUNDS)
                        .map(|round| (round, UnitProgress::Waiting))
                        .collect(),
                    is_measured: false,
                    speculative_rebuilds_voided: 0,
                })
                .collect(),
            order: preparation_units_in_measurement_order(points.len(), image_directory.is_some()),
            accounted_bytes_by_unit: BTreeMap::new(),
            footprint_bytes_of_the_prepared: BTreeMap::new(),
            phases: Vec::new(),
        }
    }

    fn sampling_points(&self) -> Vec<SamplingPoint> {
        self.points
            .iter()
            .map(|point_state| point_state.point)
            .collect()
    }

    fn accounted_bytes(&self) -> u64 {
        self.accounted_bytes_by_unit.values().sum()
    }

    /// 一份池的内存：那一点造好了按真写过的扇区，没造好按估的。
    fn pool_bytes_of(&self, point_index: usize) -> u64 {
        self.footprint_bytes_of_the_prepared
            .get(&point_index)
            .copied()
            .unwrap_or_else(|| {
                estimated_pool_footprint_bytes_before_construction(self.points[point_index].point)
            })
    }

    /// 之后串行测量要留的余量：还没量完的点里最大一份池的几份（内存臂每次测量拷一份池，另有重放材料与镜像提示的工作集）。
    fn headroom_bytes(&self) -> u64 {
        (0..self.points.len())
            .filter(|point_index| !self.points[*point_index].is_measured)
            .map(|point_index| self.pool_bytes_of(point_index))
            .max()
            .unwrap_or(0)
            * POOLS_OF_HEADROOM_FOR_THE_MEASUREMENT
    }

    /// 一份在跑时的峰值内存：造池与几何核 2 份池（造好的与几何核的拷贝），重造 5/4 份，建镜像 1 份（提示盘至多是池那么大）。
    fn peak_bytes_of(&self, unit: PreparationUnit) -> u64 {
        match unit {
            PreparationUnit::PrepareThePoint { point_index } => {
                estimated_pool_footprint_bytes_before_construction(self.points[point_index].point)
                    * POOLS_HELD_WHILE_PREPARING_A_SAMPLING_POINT
            }
            PreparationUnit::BuildTheRealDiskImages { point_index } => {
                self.pool_bytes_of(point_index)
            }
            PreparationUnit::RebuildForTheRound { point_index, .. } => {
                self.pool_bytes_of(point_index) * REBUILD_PEAK_NUMERATOR / REBUILD_PEAK_DENOMINATOR
            }
        }
    }

    fn is_point_prepared(&self, point_index: usize) -> bool {
        matches!(self.points[point_index].prepared, UnitProgress::Done(_))
    }

    /// 这一份好了（造好了、或已被取走）。
    fn is_done(&self, unit: PreparationUnit) -> bool {
        let point_state = &self.points[unit.point_index()];
        match unit {
            PreparationUnit::PrepareThePoint { .. } => matches!(
                point_state.prepared,
                UnitProgress::Done(_) | UnitProgress::Taken
            ),
            PreparationUnit::BuildTheRealDiskImages { .. } => matches!(
                point_state.real_disk_images,
                UnitProgress::Done(_) | UnitProgress::Taken
            ),
            PreparationUnit::RebuildForTheRound { round, .. } => matches!(
                point_state.rebuilt_by_round.get(&round),
                Some(UnitProgress::Done(_) | UnitProgress::Taken)
            ),
        }
    }

    /// 能派了：还在等，且它要的都有了。真盘镜像要那一点已造好、它之前的点都已量完（盘上同一时刻只有正在量的那一点的镜像，
    /// 镜像在量那一点之前建）。重造不等那一点造好（推测着先造，[`PreparationQueue::start`]）。
    fn is_ready(&self, unit: PreparationUnit) -> bool {
        let point_state = &self.points[unit.point_index()];
        match unit {
            PreparationUnit::PrepareThePoint { .. } => {
                matches!(point_state.prepared, UnitProgress::Waiting)
            }
            PreparationUnit::BuildTheRealDiskImages { point_index } => {
                matches!(point_state.real_disk_images, UnitProgress::Waiting)
                    && self.is_point_prepared(point_index)
                    && self.points[..point_index]
                        .iter()
                        .all(|earlier_point| earlier_point.is_measured)
            }
            PreparationUnit::RebuildForTheRound { round, .. } => matches!(
                point_state.rebuilt_by_round.get(&round),
                Some(UnitProgress::Waiting)
            ),
        }
    }

    /// 这一刻派哪几份：能派的按量的次序排。最先量的那一点（能派的里头一份所在的点）取 [`tasks_of_the_next_group`] 份，可以只派一部分；
    /// 它整点派出去了，后面的点要整点装得下（线程与内存）才派，一点装不下就停。整点派：一点的几份一起派、一起做完，不让半点的长活
    /// （例如后量的那一点只派出造池、重造留到下一次）把这一次准备拖长又在下一次再拖一遍。一份都装不下而 `is_guaranteed_one`（什么都没在跑、
    /// 要的那一份还没好）时保底派最前面那一份，照串行那样一次只做一份。交回（派哪几份、是不是保底派的）。
    fn units_to_start(
        &self,
        free_threads: usize,
        is_guaranteed_one: bool,
    ) -> (Vec<PreparationUnit>, bool) {
        let ready: Vec<PreparationUnit> = self
            .order
            .iter()
            .copied()
            .filter(|unit| self.is_ready(*unit))
            .collect();
        let peaks: Vec<u64> = ready.iter().map(|unit| self.peak_bytes_of(*unit)).collect();
        let free_bytes = preparation_budget_bytes(
            self.settings.memory_cap.bytes,
            self.accounted_bytes(),
            self.headroom_bytes(),
        );
        let units_of_the_point_at = |start: usize| {
            ready[start..]
                .iter()
                .take_while(|unit| unit.point_index() == ready[start].point_index())
                .count()
        };
        let leading_units = if ready.is_empty() {
            0
        } else {
            units_of_the_point_at(0)
        };
        let mut fitting =
            tasks_of_the_next_group(&peaks[..leading_units], free_threads, free_bytes);
        if fitting == leading_units {
            let mut threads_left = free_threads.saturating_sub(fitting);
            let mut bytes_left = free_bytes.saturating_sub(peaks[..fitting].iter().sum());
            // 迭代上界：每一圈派出整整一点，至多 ready.len() 圈
            while fitting < ready.len() {
                let group_units = units_of_the_point_at(fitting);
                let group_bytes: u64 = peaks[fitting..fitting + group_units].iter().sum();
                if group_units > threads_left || group_bytes > bytes_left {
                    break;
                }
                threads_left -= group_units;
                bytes_left -= group_bytes;
                fitting += group_units;
            }
        }
        let is_guaranteed = fitting == 0 && is_guaranteed_one && !ready.is_empty();
        let count = if is_guaranteed { 1 } else { fitting };
        (ready.into_iter().take(count).collect(), is_guaranteed)
    }

    /// 记下开派一份（在跑，按峰值记账），交回给线程的东西。重造用这一刻的配方：那一点已造好就是它最后的配方，没造好就是
    /// 头一次的配方（推测着先造；那一点造好而配方改了，[`PreparationQueue::void_the_stale_rebuilds`] 把它作废）。
    fn start(&mut self, unit: PreparationUnit) -> PreparationWork {
        let peak_bytes = self.peak_bytes_of(unit);
        self.accounted_bytes_by_unit.insert(unit, peak_bytes);
        let point_state = &mut self.points[unit.point_index()];
        match unit {
            PreparationUnit::PrepareThePoint { .. } => {
                point_state.prepared = UnitProgress::Running;
                PreparationWork::PrepareThePoint(point_state.point)
            }
            PreparationUnit::BuildTheRealDiskImages { .. } => {
                let prepared = match &point_state.prepared {
                    UnitProgress::Done(prepared) => Arc::clone(prepared),
                    UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => {
                        panic!("真盘镜像只在那一点造好之后、量完之前派（is_ready）")
                    }
                };
                point_state.real_disk_images = UnitProgress::Running;
                PreparationWork::BuildTheRealDiskImages {
                    directory: self
                        .image_directory
                        .clone()
                        .expect("只有带镜像目录的队列（第一段）才排真盘镜像这一份"),
                    prepared,
                }
            }
            PreparationUnit::RebuildForTheRound { round, .. } => {
                let recipe = match &point_state.prepared {
                    UnitProgress::Done(prepared) => prepared.recipe.clone(),
                    UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => {
                        first_recipe_of(point_state.point)
                    }
                };
                point_state
                    .rebuilt_by_round
                    .insert(round, UnitProgress::Running);
                PreparationWork::Rebuild(recipe)
            }
        }
    }

    /// 记下做完一份：账从峰值换成池的内存（真盘镜像不留账）；之后作废那一点配方对不上的重造。
    fn finish(&mut self, unit: PreparationUnit, outcome: PreparationOutcome) {
        self.accounted_bytes_by_unit.remove(&unit);
        let point_index = unit.point_index();
        match (unit, outcome) {
            (
                PreparationUnit::PrepareThePoint { .. },
                PreparationOutcome::Prepared {
                    prepared,
                    footprint_bytes,
                },
            ) => {
                self.footprint_bytes_of_the_prepared
                    .insert(point_index, footprint_bytes);
                self.accounted_bytes_by_unit.insert(unit, footprint_bytes);
                self.points[point_index].prepared = UnitProgress::Done(Arc::new(prepared));
            }
            (
                PreparationUnit::BuildTheRealDiskImages { .. },
                PreparationOutcome::RealDiskImages(images),
            ) => {
                self.points[point_index].real_disk_images = UnitProgress::Done(images);
            }
            (
                PreparationUnit::RebuildForTheRound { round, .. },
                PreparationOutcome::Rebuilt {
                    rebuilt,
                    footprint_bytes,
                },
            ) => {
                self.accounted_bytes_by_unit.insert(unit, footprint_bytes);
                self.points[point_index]
                    .rebuilt_by_round
                    .insert(round, UnitProgress::Done(rebuilt));
            }
            (
                PreparationUnit::PrepareThePoint { .. },
                PreparationOutcome::RealDiskImages(_) | PreparationOutcome::Rebuilt { .. },
            )
            | (
                PreparationUnit::BuildTheRealDiskImages { .. },
                PreparationOutcome::Prepared { .. } | PreparationOutcome::Rebuilt { .. },
            )
            | (
                PreparationUnit::RebuildForTheRound { .. },
                PreparationOutcome::Prepared { .. } | PreparationOutcome::RealDiskImages(_),
            ) => panic!("准备线程交回的结果与派的那一份对不上"),
        }
        self.void_the_stale_rebuilds(point_index);
    }

    /// 那一点造好了：配方与它不同的重造（头一次的配方推测着先造的，几何核改了负载）作废、回去重排，按新配方再造。
    /// 还在跑的那几份交回时（[`PreparationQueue::finish`]）再判一次。
    fn void_the_stale_rebuilds(&mut self, point_index: usize) {
        let point_state = &mut self.points[point_index];
        let UnitProgress::Done(prepared) = &point_state.prepared else {
            return;
        };
        let stale_rounds: Vec<usize> = point_state
            .rebuilt_by_round
            .iter()
            .filter_map(|(round, progress)| match progress {
                UnitProgress::Done(rebuilt) => {
                    (rebuilt.recipe != prepared.recipe).then_some(*round)
                }
                UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => None,
            })
            .collect();
        for round in stale_rounds {
            point_state
                .rebuilt_by_round
                .insert(round, UnitProgress::Waiting);
            point_state.speculative_rebuilds_voided += 1;
            self.accounted_bytes_by_unit
                .remove(&PreparationUnit::RebuildForTheRound { point_index, round });
        }
    }

    fn speculative_rebuilds_voided(&self) -> usize {
        self.points
            .iter()
            .map(|point_state| point_state.speculative_rebuilds_voided)
            .sum()
    }

    /// 要的那一份还没好就跑一次准备（[`run_the_preparation_phase`]）。
    ///
    /// # Panics
    /// 跑完那一次它仍没好（它要的点还没量完就要它的真盘镜像这类次序错）。
    fn make_ready(&mut self, unit: PreparationUnit, emitter: &mut Emitter) {
        if !self.is_done(unit) {
            run_the_preparation_phase(self, emitter, unit);
        }
        assert!(
            self.is_done(unit),
            "一次准备跑完，要的那一份（{}）应已好",
            unit.label(&self.sampling_points())
        );
    }

    /// 一个取样点造好的样子（造池与几何核）；那一点量完之前队列里留着一份（[`PreparationQueue::finish_the_point`] 放掉）。
    fn prepared_point(&mut self, point_index: usize, emitter: &mut Emitter) -> Arc<PreparedPoint> {
        self.make_ready(PreparationUnit::PrepareThePoint { point_index }, emitter);
        match &self.points[point_index].prepared {
            UnitProgress::Done(prepared) => Arc::clone(prepared),
            UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => {
                panic!("那一点量完之后不再取它造好的池")
            }
        }
    }

    /// 取走一个取样点的真盘镜像（第一段，量那一点之前）。
    fn take_the_real_disk_images(
        &mut self,
        point_index: usize,
        emitter: &mut Emitter,
    ) -> Result<RealDiskImages, String> {
        let unit = PreparationUnit::BuildTheRealDiskImages { point_index };
        self.make_ready(unit, emitter);
        self.accounted_bytes_by_unit.remove(&unit);
        match std::mem::replace(
            &mut self.points[point_index].real_disk_images,
            UnitProgress::Taken,
        ) {
            UnitProgress::Done(images) => images,
            UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken => {
                panic!("一个取样点的真盘镜像只取一次")
            }
        }
    }

    /// 取走一个取样点第 `round` 轮的重造池（量那一轮之前）。
    fn take_the_rebuilt_pool(
        &mut self,
        point_index: usize,
        round: usize,
        emitter: &mut Emitter,
    ) -> RebuiltPool {
        let unit = PreparationUnit::RebuildForTheRound { point_index, round };
        self.make_ready(unit, emitter);
        self.accounted_bytes_by_unit.remove(&unit);
        match self.points[point_index]
            .rebuilt_by_round
            .insert(round, UnitProgress::Taken)
        {
            Some(UnitProgress::Done(rebuilt)) => rebuilt,
            Some(UnitProgress::Waiting | UnitProgress::Running | UnitProgress::Taken) | None => {
                panic!("一个取样点一轮的重造池只取一次，轮号在 2..=ROUNDS 里")
            }
        }
    }

    /// 那一点量完了：放掉它造好的池与没取走的重造（预算不再算它们），它之后的那一点可以建真盘镜像了。
    fn finish_the_point(&mut self, point_index: usize) {
        let point_state = &mut self.points[point_index];
        point_state.is_measured = true;
        point_state.prepared = UnitProgress::Taken;
        for progress in point_state.rebuilt_by_round.values_mut() {
            *progress = UnitProgress::Taken;
        }
        self.accounted_bytes_by_unit
            .retain(|unit, _bytes| unit.point_index() != point_index);
    }
}

/// 一次准备的读数（产物 `name=preparation` 一行）。
#[derive(Clone, Debug, PartialEq)]
struct PreparationPhaseReading {
    target: PreparationUnit,
    started_prepare_units: usize,
    started_real_disk_image_units: usize,
    started_rebuild_units: usize,
    started_speculative_rebuild_units: usize,
    speculative_rebuilds_voided: usize,
    guaranteed_units: usize,
    most_threads_at_once: usize,
    distinct_threads: usize,
    memory_headroom_bytes: u64,
    largest_accounted_bytes: u64,
    wall_seconds: f64,
}

/// 一份准备交回主线程的：哪一份、在哪条线程上做的、结果（panic 了带着 panic 的内容，主线程再抛）。
type FinishedPreparation = (
    PreparationUnit,
    std::thread::ThreadId,
    std::thread::Result<PreparationOutcome>,
);

/// 跑一次准备：先当场声明并行段，再按 [`PreparationQueue::units_to_start`] 派，每份各起一条线程、各自建状态；收回一份就再看能不能派。
/// `target`（量的那一侧此刻要的那一份）好了就不再派新的，等在跑的都交回、全部线程停下（`std::thread::scope`）才回来，之后的计时窗口里没有准备线程。
/// 打一行 `name=preparation`，读数另记进 [`PreparationQueue::phases`]。
fn run_the_preparation_phase(
    queue: &mut PreparationQueue<'_>,
    emitter: &mut Emitter,
    target: PreparationUnit,
) {
    // 准备阶段多线程，在这里声明串行段结束（下一个计时窗口再声明串行）
    emitter.declare(DeclaredSegment::Parallel);
    let started = Instant::now();
    let voided_before = queue.speculative_rebuilds_voided();
    let threads = queue.settings.threads;
    let worker = queue.worker;
    let mut reading = PreparationPhaseReading {
        target,
        started_prepare_units: 0,
        started_real_disk_image_units: 0,
        started_rebuild_units: 0,
        started_speculative_rebuild_units: 0,
        speculative_rebuilds_voided: 0,
        guaranteed_units: 0,
        most_threads_at_once: 0,
        distinct_threads: 0,
        memory_headroom_bytes: queue.headroom_bytes(),
        largest_accounted_bytes: queue.accounted_bytes(),
        wall_seconds: 0.0,
    };
    let mut distinct: HashSet<std::thread::ThreadId> = HashSet::new();
    let (sender, receiver) = std::sync::mpsc::channel::<FinishedPreparation>();
    std::thread::scope(|scope| {
        let mut running = 0_usize;
        // 迭代上界：每一圈至少收回一份（running 为 0 就出去）；要的那一份没好之前派的每一份至多被作废重派一次（作废只发生在那一点
        // 造好的那一刻与之前推测着派的那几份上），所以派出的份数有上界。跨圈的状态：running = 派出去还没收回的份数，不超过 threads。
        loop {
            if !queue.is_done(target) {
                let (units, is_guaranteed) =
                    queue.units_to_start(threads.saturating_sub(running), running == 0);
                if is_guaranteed {
                    reading.guaranteed_units += units.len();
                }
                for unit in units {
                    match unit {
                        PreparationUnit::PrepareThePoint { .. } => {
                            reading.started_prepare_units += 1;
                        }
                        PreparationUnit::BuildTheRealDiskImages { .. } => {
                            reading.started_real_disk_image_units += 1;
                        }
                        PreparationUnit::RebuildForTheRound { point_index, .. } => {
                            reading.started_rebuild_units += 1;
                            if !queue.is_point_prepared(point_index) {
                                reading.started_speculative_rebuild_units += 1;
                            }
                        }
                    }
                    let work = queue.start(unit);
                    let sender_of_this_unit = sender.clone();
                    scope.spawn(move || {
                        let outcome =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker(work)));
                        sender_of_this_unit
                            .send((unit, std::thread::current().id(), outcome))
                            .expect("收的一方等到派出去的每一份都交回");
                    });
                    running += 1;
                }
                reading.most_threads_at_once = reading.most_threads_at_once.max(running);
                reading.largest_accounted_bytes =
                    reading.largest_accounted_bytes.max(queue.accounted_bytes());
            }
            if running == 0 {
                break;
            }
            let (unit, thread, outcome) = receiver
                .recv()
                .expect("还有派出去没收回的，每一份交回之前发的一方都在");
            running -= 1;
            distinct.insert(thread);
            match outcome {
                Ok(outcome) => queue.finish(unit, outcome),
                Err(panic_content) => std::panic::resume_unwind(panic_content),
            }
        }
    });
    reading.distinct_threads = distinct.len();
    reading.speculative_rebuilds_voided = queue.speculative_rebuilds_voided() - voided_before;
    reading.wall_seconds = started.elapsed().as_secs_f64();
    emitter.line(&format!(
        "name=preparation segment={} target={} started_units={} started_prepare_units={} started_real_disk_image_units={} started_rebuild_units={} started_speculative_rebuild_units={} speculative_rebuilds_voided={} guaranteed_units={} most_threads_at_once={} distinct_threads={} thread_limit={threads} memory_cap_bytes={} memory_headroom_bytes={} largest_accounted_bytes={} wall_seconds={:.3}",
        queue.segment,
        target.label(&queue.sampling_points()),
        reading.started_prepare_units + reading.started_real_disk_image_units + reading.started_rebuild_units,
        reading.started_prepare_units,
        reading.started_real_disk_image_units,
        reading.started_rebuild_units,
        reading.started_speculative_rebuild_units,
        reading.speculative_rebuilds_voided,
        reading.guaranteed_units,
        reading.most_threads_at_once,
        reading.distinct_threads,
        queue.settings.memory_cap.bytes,
        reading.memory_headroom_bytes,
        reading.largest_accounted_bytes,
        reading.wall_seconds,
    ));
    queue.phases.push(reading);
}

/// 配置行里准备阶段那几段：线程数、内存上限与它从哪取、一共几份准备、按造池前的估算头一次能同时派几份、由此装不装得下全部
/// （`all_at_once`：除真盘镜像外全部一次派出；`in_batches`：分几批，批内并行、批间串行量）。真盘镜像那几份不在头一次里（要等前一点量完）。
fn preparation_configuration_fields(
    points: &[SamplingPoint],
    image_directory: Option<&Path>,
    settings: &PreparationSettings,
) -> String {
    let queue = PreparationQueue::new(
        "configuration",
        points,
        image_directory,
        settings,
        &run_the_preparation_work,
    );
    let (started_at_once, _is_guaranteed) = queue.units_to_start(settings.threads, true);
    let real_disk_image_units = queue
        .order
        .iter()
        .filter(|unit| matches!(unit, PreparationUnit::BuildTheRealDiskImages { .. }))
        .count();
    let is_all_at_once = started_at_once.len() + real_disk_image_units == queue.order.len();
    format!(
        "preparation_threads={} preparation_memory_cap_bytes={} preparation_memory_cap_source={} preparation_units={} preparation_units_started_at_once_by_the_estimates={} preparation_plan_by_the_estimates={}",
        settings.threads,
        settings.memory_cap.bytes,
        settings.memory_cap.source.label(),
        queue.order.len(),
        started_at_once.len(),
        if is_all_at_once { "all_at_once" } else { "in_batches" },
    )
}

/// 一轮里第一行阳性对照三种注入的次序（登记第十二节 12.9）：基线、盘臂对照、CPU 臂对照每轮各一次，次序按轮号轮换。
/// 基线 5 轮先跑完、对照后跑的那一趟（产物 `e185-gc-batch-wall-clock-split-wall-clock.out`），时段的差别整个算进了「对照 − 基线」；交错之后每一轮的三种注入挨着量。
fn row_one_control_injections_of_the_round(round: usize) -> [WriteInjection; 3] {
    let in_registration_order = [
        WriteInjection::Nothing,
        WriteInjection::SleepTwoMilliseconds,
        WriteInjection::SpinTwoMillisecondsOfThreadCpu,
    ];
    let shift = round.checked_sub(1).expect("轮号从 1 起") % in_registration_order.len();
    let mut order = in_registration_order;
    order.rotate_left(shift);
    order
}

/// 一个取样点一轮里要量的（批大小、设备臂、注入），按量的次序。对照点 K = 8 那一格，每个设备臂挨着量三种注入
/// （[`row_one_control_injections_of_the_round`] 的次序）；别的格只量基线。
fn measurements_of_the_round(
    point: SamplingPoint,
    round: usize,
) -> Vec<(u64, DeviceArm, WriteInjection)> {
    let mut plan = Vec::new();
    for batch_units in BATCH_SIZES {
        let is_the_control_cell = point == CONTROL_POINT && batch_units == CONTROL_BATCH_UNITS;
        for arm in DeviceArm::ALL {
            if is_the_control_cell {
                for injection in row_one_control_injections_of_the_round(round) {
                    plan.push((batch_units, arm, injection));
                }
            } else {
                plan.push((batch_units, arm, WriteInjection::Nothing));
            }
        }
    }
    plan
}

/// 第一段一个取样点：从准备队列取它的真盘镜像（镜像在量这一点之前建，连同造池与几何核；还没好就跑一次准备，[`PreparationQueue`]），
/// 打准备时攥着的行，再串行量 5 轮 × 两臂 × 三个批大小；是对照点的，每一轮在 K = 8 那一格里另跑第一行两条臂的阳性对照
/// （与基线交错，[`measurements_of_the_round`]）。第 2 轮起的重造池从队列里取，还没造好就在那一轮开头跑一次准备（量的时候不造）。
/// 量完放掉这一点的池与镜像。交回停机条款的触发。
fn measure_one_sampling_point(
    emitter: &mut Emitter,
    stored: &mut Vec<StoredIntent>,
    queue: &mut PreparationQueue<'_>,
    point_index: usize,
) -> Vec<String> {
    let real_disk_images_or_reason = queue.take_the_real_disk_images(point_index, emitter);
    let prepared = queue.prepared_point(point_index, emitter);
    let point = prepared.recipe.point;
    for line in &prepared.lines {
        emitter.line(line);
    }
    emitter.line(&format!(
        "name=pool {} choice={} geometry_formed={} speculative_rebuilds_voided={} digests={}",
        point.label(),
        prepared.choice.label(),
        prepared.is_the_geometry_formed,
        queue.points[point_index].speculative_rebuilds_voided,
        prepared
            .reference_digests
            .iter()
            .map(hexadecimal_of)
            .collect::<Vec<_>>()
            .join(",")
    ));
    let data_units = prepared
        .is_the_geometry_formed
        .then_some(point.fraction.data_units_per_segment());
    let mut real_disk_images = match real_disk_images_or_reason {
        Ok(images) => Some(images),
        Err(reason) => {
            emitter.line(&format!(
                "name=real_disk_images {} created=false reason={reason}",
                point.label(),
            ));
            None
        }
    };
    let mut findings = Vec::new();
    let mut rebuilt_pool_of_the_round: Option<ConstructedPool> = None;
    for round in 1..=ROUNDS {
        wait_until_no_other_measurement_runs(emitter);
        if round > 1 {
            let rebuilt = queue.take_the_rebuilt_pool(point_index, round, emitter);
            let is_identical = rebuilt.digests == prepared.reference_digests;
            emitter.line(&format!(
                "name=memory_rebuild {} round={round} construction_seconds={:.3} digests_identical={is_identical}",
                point.label(),
                rebuilt.pool.construction_seconds
            ));
            rebuilt_pool_of_the_round = is_identical.then_some(rebuilt.pool);
        }
        let pool_of_the_round = if round == 1 {
            Some(&prepared.pool)
        } else {
            rebuilt_pool_of_the_round.as_ref()
        };
        for (batch_units, arm, injection) in measurements_of_the_round(point, round) {
            let label = MeasurementLabel {
                arm,
                point,
                batch_units,
                round,
                attempt: 0,
                injection,
                choice: prepared.choice,
            };
            findings.extend(measure_with_retries(
                emitter,
                stored,
                real_disk_images.as_mut(),
                label,
                prepared.is_the_geometry_formed,
                data_units,
                pool_of_the_round,
            ));
            if !findings.is_empty() {
                return findings;
            }
        }
    }
    if let Some(images) = real_disk_images {
        if let Err(error) = images.remove() {
            emitter.line(&format!(
                "name=real_disk_images {} removed=false reason={}",
                point.label(),
                field_token(&error.to_string())
            ));
        }
    }
    queue.finish_the_point(point_index);
    findings
}

/// 设备顺序写带宽的一轮读数（字节 / 秒）：`O_DIRECT` 每次 4 MiB 写满 1 GiB、末尾一次 `fdatasync`；走页缓存同样写、不同步（第四行对照）。
const SEQUENTIAL_WRITE_BYTES: u64 = 1 << 30;

fn sequential_write_bytes_per_second(path: &Path, policy: PageCachePolicy) -> Result<f64, String> {
    let mut device = DirectInputOutputBlockDevice::open_existing(
        path,
        policy,
        PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)),
    )
    .map_err(|error| field_token(&format!("open_failed:{error:?}")))?;
    let pattern: Vec<u8> = (0..IMAGE_CHUNK_BYTES)
        .map(|position| u8::try_from(position % 251).expect("小于 251"))
        .collect();
    let started = Instant::now();
    for chunk in 0..SEQUENTIAL_WRITE_BYTES / IMAGE_CHUNK_BYTES {
        device
            .write_at(
                DeviceOffsetInBytes(chunk * IMAGE_CHUNK_BYTES),
                &pattern,
                WriteDurability::Plain,
            )
            .map_err(|error| field_token(&format!("write_failed:{error:?}")))?;
    }
    let is_synced = match policy {
        PageCachePolicy::BypassWithDirectInputOutput => {
            device
                .barrier()
                .map_err(|error| field_token(&format!("barrier_failed:{error:?}")))?;
            true
        }
        PageCachePolicy::GoThroughPageCache => false,
    };
    let seconds = started.elapsed().as_secs_f64();
    if !is_synced {
        // 走页缓存那一臂的脏页在计时之外落盘，免得拖到下一轮的 `O_DIRECT` 计时里
        device
            .barrier()
            .map_err(|error| field_token(&format!("barrier_failed:{error:?}")))?;
    }
    Ok(SEQUENTIAL_WRITE_BYTES as f64 / seconds)
}

/// Q15 与第四行对照：各 5 轮，交回（`O_DIRECT` 带同步的中位、走页缓存不同步的中位）。
fn measure_the_device_sequential_write(
    emitter: &mut Emitter,
    image_directory: &Path,
) -> Option<(f64, f64)> {
    let path = image_directory.join("e185-sequential-write.img");
    if path.exists() {
        std::fs::remove_file(&path).ok()?;
    }
    if let Err(error) = create_a_zero_filled_image(&path, SEQUENTIAL_WRITE_BYTES) {
        emitter.line(&format!(
            "name=q15 created=false reason={}",
            field_token(&error.to_string())
        ));
        return None;
    }
    let mut direct = Vec::new();
    let mut page_cache = Vec::new();
    for round in 1..=ROUNDS {
        wait_until_no_other_measurement_runs(emitter);
        enter_the_timing_window(emitter, TimingWindow::DeviceSequentialWrite);
        for (policy, label, readings) in [
            (
                PageCachePolicy::BypassWithDirectInputOutput,
                "direct_with_fdatasync",
                &mut direct,
            ),
            (
                PageCachePolicy::GoThroughPageCache,
                "page_cache_without_sync",
                &mut page_cache,
            ),
        ] {
            match sequential_write_bytes_per_second(&path, policy) {
                Ok(bytes_per_second) => {
                    emitter.line(&format!("name=q15_round round={round} write_path={label} bytes_per_second={bytes_per_second:.1}"));
                    readings.push(bytes_per_second);
                }
                Err(reason) => emitter.line(&format!(
                    "name=q15_round round={round} write_path={label} valid=false reason={reason}"
                )),
            }
        }
    }
    std::fs::remove_file(&path).ok();
    Some((median_of(&direct)?, median_of(&page_cache)?))
}

// ───────────────────────────── 第一段 wall-clock：汇总（Q1–Q4、Q14–Q16、对照、轨迹、几何敏感性、判别力自证） ─────────────────────────────

/// 某一格（臂、取样点、K、注入）的全部有效意图，按轮排。
fn intents_of(
    stored: &[StoredIntent],
    arm: DeviceArm,
    point: SamplingPoint,
    batch_units: u64,
    injection: WriteInjection,
) -> Vec<&StoredIntent> {
    let mut intents: Vec<&StoredIntent> = stored
        .iter()
        .filter(|intent| {
            intent.arm == arm
                && intent.point == point
                && intent.batch_units == batch_units
                && intent.injection == injection
        })
        .collect();
    intents.sort_by_key(|intent| intent.round);
    intents
}

/// 同一轮号的两份读数配对（内存臂与真盘臂、K 与 2K）。
fn paired_by_round<'stored>(
    left: &[&'stored StoredIntent],
    right: &[&'stored StoredIntent],
) -> Vec<(&'stored StoredIntent, &'stored StoredIntent)> {
    left.iter()
        .filter_map(|left_intent| {
            right
                .iter()
                .find(|right_intent| right_intent.round == left_intent.round)
                .map(|right_intent| (*left_intent, *right_intent))
        })
        .collect()
}

fn list_of(values: &[f64]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values
        .iter()
        .map(|value| format!("{value:.6}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn optional_number(value: Option<f64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| format!("{number:.6}"))
}

/// 取样点的键：（活槽占比、P、K）。
type SamplingPointKey = (LiveSlotFraction, u64, u64);

/// 一个旋钮取另一端时的那个取样点；这一点本来就在另一端、或没有对应点时交回 `None`。
type SamplingPointAtTheOtherEndOfTheKnob = fn(&SamplingPointKey) -> Option<SamplingPointKey>;

/// 一个量在一个旋钮两端的翻面（登记第八节「几何敏感性」）：键是（活槽占比、P、K），旋钮是三者之一；只比另两样相同、这一样取两端的那几对。
fn emit_the_sensitivity(
    emitter: &mut Emitter,
    quantity: &str,
    judgements: &BTreeMap<SamplingPointKey, &'static str>,
) {
    let knobs: [(&str, SamplingPointAtTheOtherEndOfTheKnob); 3] = [
        ("fraction", |key| {
            (key.0 == LiveSlotFraction::OneQuarter).then_some((
                LiveSlotFraction::ThreeQuarters,
                key.1,
                key.2,
            ))
        }),
        ("pool_data_units", |key| {
            (key.1 == POOL_DATA_UNITS[0]).then_some((key.0, POOL_DATA_UNITS[1], key.2))
        }),
        ("batch_units", |key| {
            (key.2 == BATCH_SIZES[0]).then_some((key.0, key.1, BATCH_SIZES[2]))
        }),
    ];
    for (knob, partner_of) in knobs {
        let mut pairs = 0_usize;
        let mut flips = Vec::new();
        for (key, judgement) in judgements {
            let Some(partner) = partner_of(key) else {
                continue;
            };
            let Some(partner_judgement) = judgements.get(&partner) else {
                continue;
            };
            pairs += 1;
            if judgement != partner_judgement {
                flips.push(format!(
                    "{}:{}:{}:{judgement}->{}:{}:{}:{partner_judgement}",
                    key.0.label(),
                    key.1,
                    key.2,
                    partner.0.label(),
                    partner.1,
                    partner.2
                ));
            }
        }
        emitter.line(&format!(
            "name=sensitivity quantity={quantity} knob={knob} pairs={pairs} flips={} statement={} flipped_pairs={}",
            flips.len(),
            if flips.is_empty() { "does_not_flip_at_these_points" } else { "flips_with_this_knob" },
            if flips.is_empty() { "none".to_string() } else { flips.join(",") }
        ));
    }
}

/// 判别力自证（产物里那一份）：量到的最低、最高两点之间挪门槛，两点判定必须不同。
fn emit_the_discrimination(
    emitter: &mut Emitter,
    quantity: &str,
    values: &[f64],
    judge: impl Fn(f64, f64) -> &'static str,
) -> bool {
    let lowest = values.iter().copied().reduce(f64::min);
    let highest = values.iter().copied().reduce(f64::max);
    match (lowest, highest) {
        (Some(lower_value), Some(higher_value)) if lower_value < higher_value => {
            let (moved, lower_judgement, higher_judgement) =
                judgements_with_the_threshold_moved_between(lower_value, higher_value, judge);
            let holds = lower_judgement != higher_judgement;
            emitter.line(&format!(
                "name=discrimination quantity={quantity} lower_value={lower_value:.6} higher_value={higher_value:.6} moved_threshold={moved:.6} lower_judgement={lower_judgement} higher_judgement={higher_judgement} holds={holds}"
            ));
            holds
        }
        (Some(_), Some(_)) | (None, _) | (Some(_), None) => {
            emitter.line(&format!(
                "name=discrimination quantity={quantity} holds=not_run reason=fewer_than_two_distinct_values"
            ));
            true
        }
    }
}

/// 第一段汇总交回给判决行的几样。
struct WallClockSummary {
    positive_control_failures: Vec<String>,
    gc_to_sequential_write_bandwidth: Option<(f64, &'static str)>,
    discrimination_failures: usize,
    clause_wall_clock_not_rising_with_the_batch_triggered: usize,
    clause_two_cpu_paths_disagree_points: usize,
}

#[allow(
    clippy::too_many_lines,
    reason = "一段汇总按登记第六节的量逐个报（Q1–Q4、Q14–Q16、两行对照、轨迹、敏感性、判别力自证），拆开之后每一段都要同一组取样点与读数"
)]
fn summarize_the_wall_clock_segment(
    emitter: &mut Emitter,
    stored: &[StoredIntent],
    points: &[SamplingPoint],
    sequential_write: Option<(f64, f64)>,
) -> WallClockSummary {
    let none = WriteInjection::Nothing;
    let mut summary = WallClockSummary {
        positive_control_failures: Vec::new(),
        gc_to_sequential_write_bandwidth: None,
        discrimination_failures: 0,
        clause_wall_clock_not_rising_with_the_batch_triggered: 0,
        clause_two_cpu_paths_disagree_points: 0,
    };
    let mut rho_judgements_by_point = BTreeMap::new();
    let mut wall_ratio_judgements_by_point = BTreeMap::new();
    let mut bandwidth_ratio_judgements_by_point = BTreeMap::new();
    let mut rho_medians = Vec::new();
    let mut wall_ratio_medians = Vec::new();
    let mut gc_write_bandwidth_medians: Vec<(SamplingPoint, u64, f64)> = Vec::new();
    for point in points {
        for batch_units in BATCH_SIZES {
            let memory = intents_of(stored, DeviceArm::Memory, *point, batch_units, none);
            let real = intents_of(stored, DeviceArm::RealDisk, *point, batch_units, none);
            let formed = real
                .first()
                .or(memory.first())
                .is_some_and(|intent| intent.is_the_geometry_formed);
            let rhos: Vec<f64> = real
                .iter()
                .map(|intent| intent.batch_cpu_nanoseconds() / intent.batch_wait_nanoseconds())
                .collect();
            let rho_judgements: Vec<&'static str> = rhos
                .iter()
                .map(|rho| judgement_of_rho_at_the_registered_edges(*rho))
                .collect();
            let rho_judgement = judgement_of_the_rounds(&rho_judgements);
            emitter.line(&format!(
                "name=q1 {} batch_units={batch_units} geometry_formed={formed} rounds={} rho_per_round={} rho_median={} judgement_per_round={} judgement={rho_judgement}",
                point.label(),
                rhos.len(),
                list_of(&rhos),
                optional_number(median_of(&rhos)),
                rho_judgements.join(","),
            ));
            rho_judgements_by_point.insert(
                (point.fraction, point.pool_data_units, batch_units),
                rho_judgement,
            );
            if let Some(median) = median_of(&rhos) {
                rho_medians.push(median);
            }
            let pairs = paired_by_round(&memory, &real);
            let wall_ratios: Vec<f64> = pairs
                .iter()
                .map(|(memory_intent, real_intent)| {
                    real_intent.batch_wall_nanoseconds() / memory_intent.batch_wall_nanoseconds()
                })
                .collect();
            let batch_count = pairs
                .iter()
                .map(|(memory_intent, real_intent)| {
                    memory_intent
                        .batch_calls()
                        .count()
                        .min(real_intent.batch_calls().count())
                })
                .min()
                .unwrap_or(0);
            let per_batch_median_ratios: Vec<f64> = (0..batch_count)
                .filter_map(|batch_index| {
                    let ratios: Vec<f64> = pairs
                        .iter()
                        .filter_map(|(memory_intent, real_intent)| {
                            let memory_wall = memory_intent
                                .batch_calls()
                                .nth(batch_index)?
                                .wall_nanoseconds
                                as f64;
                            let real_wall =
                                real_intent.batch_calls().nth(batch_index)?.wall_nanoseconds as f64;
                            Some(real_wall / memory_wall)
                        })
                        .collect();
                    median_of(&ratios)
                })
                .collect();
            let ratio_judgements: Vec<&'static str> = wall_ratios
                .iter()
                .map(|ratio| judgement_of_wall_ratio_at_the_registered_edges(*ratio))
                .collect();
            let wall_ratio_judgement = judgement_of_the_rounds(&ratio_judgements);
            emitter.line(&format!(
                "name=q2 {} batch_units={batch_units} geometry_formed={formed} rounds={} intent_wall_ratio_per_round={} intent_wall_ratio_median={} per_batch_median_ratio={} judgement_per_round={} judgement={wall_ratio_judgement}",
                point.label(),
                wall_ratios.len(),
                list_of(&wall_ratios),
                optional_number(median_of(&wall_ratios)),
                list_of(&per_batch_median_ratios),
                ratio_judgements.join(","),
            ));
            wall_ratio_judgements_by_point.insert(
                (point.fraction, point.pool_data_units, batch_units),
                wall_ratio_judgement,
            );
            if let Some(median) = median_of(&wall_ratios) {
                wall_ratio_medians.push(median);
            }
            let reconciliations: Vec<f64> = pairs
                .iter()
                .map(|(memory_intent, real_intent)| {
                    cpu_reconciliation(
                        real_intent.batch_cpu_nanoseconds(),
                        memory_intent.batch_wall_nanoseconds(),
                    )
                })
                .collect();
            let reconciliation_judgements: Vec<&'static str> = reconciliations
                .iter()
                .map(|value| judgement_of_the_cpu_reconciliation(*value))
                .collect();
            let reconciliation_judgement = judgement_of_the_rounds(&reconciliation_judgements);
            if reconciliation_judgement != "compatible" {
                summary.clause_two_cpu_paths_disagree_points += 1;
            }
            emitter.line(&format!(
                "name=q3 {} batch_units={batch_units} rounds={} reconciliation_per_round={} judgement_per_round={} judgement={reconciliation_judgement} row_one_judgeable_here={}",
                point.label(),
                reconciliations.len(),
                list_of(&reconciliations),
                reconciliation_judgements.join(","),
                reconciliation_judgement == "compatible",
            ));
            let bandwidths: Vec<f64> = real
                .iter()
                .map(|intent| {
                    intent.batch_write_bytes() as f64 / (intent.batch_wall_nanoseconds() / 1e9)
                })
                .collect();
            let relocation_shares: Vec<f64> = real
                .iter()
                .map(|intent| {
                    intent.batch_relocation_copy_bytes() as f64 / intent.batch_write_bytes() as f64
                })
                .collect();
            emitter.line(&format!(
                "name=q14 {} batch_units={batch_units} rounds={} write_bytes_per_second_per_round={} write_bytes_per_second_median={} relocation_copy_share_per_round={} write_bytes_per_round={} relocation_copy_bytes_per_round={}",
                point.label(),
                bandwidths.len(),
                list_of(&bandwidths),
                optional_number(median_of(&bandwidths)),
                list_of(&relocation_shares),
                real.iter().map(|intent| intent.batch_write_bytes().to_string()).collect::<Vec<_>>().join(","),
                real.iter().map(|intent| intent.batch_relocation_copy_bytes().to_string()).collect::<Vec<_>>().join(","),
            ));
            if let Some(median) = median_of(&bandwidths) {
                gc_write_bandwidth_medians.push((*point, batch_units, median));
            }
        }
    }
    emit_the_sensitivity(emitter, "Q1", &rho_judgements_by_point);
    emit_the_sensitivity(emitter, "Q2", &wall_ratio_judgements_by_point);
    if !emit_the_discrimination(emitter, "Q1", &rho_medians, |value, threshold| {
        judgement_of_rho(value, threshold, f64::INFINITY)
    }) {
        summary.discrimination_failures += 1;
    }
    if !emit_the_discrimination(emitter, "Q2", &wall_ratio_medians, |value, threshold| {
        judgement_of_wall_ratio(value, threshold, f64::NEG_INFINITY)
    }) {
        summary.discrimination_failures += 1;
    }
    summarize_the_batch_doubling(emitter, stored, points, &mut summary);
    match sequential_write {
        Some((direct, page_cache)) => {
            let control_holds = page_cache_control_holds(page_cache, direct);
            emitter.line(&format!(
                "name=q15 direct_with_fdatasync_bytes_per_second_median={direct:.1} page_cache_without_sync_bytes_per_second_median={page_cache:.1} positive_control_row_four_holds={control_holds}"
            ));
            if !control_holds {
                summary
                    .positive_control_failures
                    .push("row_four_page_cache".to_string());
            }
            let ratios: Vec<f64> = gc_write_bandwidth_medians
                .iter()
                .map(|(_point, _batch_units, bandwidth)| bandwidth / direct)
                .collect();
            for (point, batch_units, bandwidth) in &gc_write_bandwidth_medians {
                let ratio = bandwidth / direct;
                bandwidth_ratio_judgements_by_point.insert(
                    (point.fraction, point.pool_data_units, *batch_units),
                    judgement_of_bandwidth_ratio_at_the_registered_edge(ratio),
                );
                emitter.line(&format!(
                    "name=q16_point {} batch_units={batch_units} ratio={ratio:.6} judgement={}",
                    point.label(),
                    judgement_of_bandwidth_ratio_at_the_registered_edge(ratio)
                ));
            }
            if let Some(maximum) = ratios.iter().copied().reduce(f64::max) {
                summary.gc_to_sequential_write_bandwidth = Some((
                    maximum,
                    judgement_of_bandwidth_ratio_at_the_registered_edge(maximum),
                ));
                emitter.line(&format!(
                    "name=q16 maximum_ratio={maximum:.6} judgement={} fourth_segment={}",
                    judgement_of_bandwidth_ratio_at_the_registered_edge(maximum),
                    if judgement_of_bandwidth_ratio_at_the_registered_edge(maximum)
                        == "single_run_enough"
                    {
                        "not_run_after_enough_to_judge"
                    } else {
                        "needed"
                    }
                ));
            }
            emit_the_sensitivity(emitter, "Q16", &bandwidth_ratio_judgements_by_point);
            if !emit_the_discrimination(emitter, "Q16", &ratios, judgement_of_bandwidth_ratio) {
                summary.discrimination_failures += 1;
            }
        }
        None => emitter.line("name=q15 valid=false reason=sequential_write_not_measured"),
    }
    summarize_the_row_one_controls(emitter, stored, &mut summary);
    emit_the_trajectories(emitter, stored);
    summary
}

/// Q4（批大小翻倍，C412 检查列）：同一 f、P、设备，第一批挂钟的比 `挂钟(2K) ÷ 挂钟(K)`，只取 2K ≤ m 的点对；不跟着涨而搬迁副本字节锚点
/// 确实翻倍的，记第十条 F1 触发（条款可能错，不作废）。
fn summarize_the_batch_doubling(
    emitter: &mut Emitter,
    stored: &[StoredIntent],
    points: &[SamplingPoint],
    summary: &mut WallClockSummary,
) {
    for arm in DeviceArm::ALL {
        for point in points {
            let data_units = point.fraction.data_units_per_segment();
            for (smaller, larger) in doubling_pairs_within(data_units) {
                let small = intents_of(stored, arm, *point, smaller, WriteInjection::Nothing);
                let large = intents_of(stored, arm, *point, larger, WriteInjection::Nothing);
                let ratios: Vec<f64> = paired_by_round(&small, &large)
                    .iter()
                    .filter_map(|(small_intent, large_intent)| {
                        Some(
                            large_intent.first_batch_wall_nanoseconds()?
                                / small_intent.first_batch_wall_nanoseconds()?,
                        )
                    })
                    .collect();
                let judgement = judgement_of_the_batch_doubling(&ratios);
                let copy_bytes_double = first_batch_copy_bytes(data_units, larger)
                    == 2 * first_batch_copy_bytes(data_units, smaller);
                let is_the_wall_clock_not_rising_clause_triggered =
                    judgement == "does_not_rise" && copy_bytes_double;
                if is_the_wall_clock_not_rising_clause_triggered {
                    summary.clause_wall_clock_not_rising_with_the_batch_triggered += 1;
                }
                emitter.line(&format!(
                    "name=q4 arm={} {} batch_units_pair={smaller}->{larger} rounds={} first_batch_wall_ratio_per_round={} median={} judgement={judgement} copy_bytes_double={copy_bytes_double} clause_wall_clock_not_rising_with_the_batch_triggered={is_the_wall_clock_not_rising_clause_triggered}",
                    arm.label(),
                    point.label(),
                    ratios.len(),
                    list_of(&ratios),
                    optional_number(median_of(&ratios)),
                ));
            }
        }
    }
}

/// 第一行两条臂的阳性对照（登记第五节阳性对照表第 1 行，CPU 臂等盘那一句照第十二节 12.9）：对照点上 K = 8，按整条意图比注入前后的中位；
/// CPU 臂另报写调用的等盘增量、两句各自的判定与原判据那一句（只报不判），每格另报按轮号配对的增量中位（不判）。
fn summarize_the_row_one_controls(
    emitter: &mut Emitter,
    stored: &[StoredIntent],
    summary: &mut WallClockSummary,
) {
    for arm in DeviceArm::ALL {
        let rounds_of = |injection: WriteInjection| -> Vec<RowOneControlRound> {
            intents_of(stored, arm, CONTROL_POINT, CONTROL_BATCH_UNITS, injection)
                .iter()
                .map(|intent| intent.row_one_control_round())
                .collect()
        };
        let baseline = rounds_of(WriteInjection::Nothing);
        for injection in [
            WriteInjection::SleepTwoMilliseconds,
            WriteInjection::SpinTwoMillisecondsOfThreadCpu,
        ] {
            let control = rounds_of(injection);
            let Some(reading) = row_one_control_reading(injection, &baseline, &control) else {
                emitter.line(&format!(
                    "name=positive_control row=1 device={} injection={} holds=not_run reason=missing_rounds baseline_rounds={} control_rounds={}",
                    arm.label(),
                    injection.label(),
                    baseline.len(),
                    control.len()
                ));
                summary.positive_control_failures.push(format!(
                    "row_one_{}_{}_not_run",
                    arm.label(),
                    injection.label()
                ));
                continue;
            };
            let (arm_of_the_question, clause_text) = match reading.judgement {
                RowOneControlJudgement::DiskArm { holds: _ } => ("disk", String::new()),
                RowOneControlJudgement::CpuArm {
                    cpu_clause_holds,
                    write_wait_clause_holds,
                    wait_clause_under_the_original_criterion_holds,
                } => (
                    "cpu",
                    format!(
                        " cpu_increment_over_injected={:.6} cpu_clause_holds={cpu_clause_holds} write_wait_increment_over_injected={:.6} write_wait_clause_holds={write_wait_clause_holds} wait_clause_under_the_original_criterion_holds={wait_clause_under_the_original_criterion_holds} original_criterion_is_reported_only=true",
                        reading.cpu_increment_nanoseconds / reading.injected_nanoseconds,
                        reading.write_wait_increment_nanoseconds / reading.injected_nanoseconds,
                    ),
                ),
            };
            let holds = reading.judgement.holds();
            emitter.line(&format!(
                "name=positive_control row=1 arm_of_the_question={arm_of_the_question} device={} injection={} baseline_rounds={} control_rounds={} write_calls_median={:.1} injected_nanoseconds={:.1} cpu_increment_nanoseconds={:.1} wait_increment_nanoseconds={:.1} write_wait_increment_nanoseconds={:.1} cpu_change={:.6} wait_change={:.6}{clause_text} paired_rounds={} cpu_increment_paired_median_nanoseconds={} write_wait_increment_paired_median_nanoseconds={} holds={holds}",
                arm.label(),
                injection.label(),
                baseline.len(),
                control.len(),
                reading.write_calls,
                reading.injected_nanoseconds,
                reading.cpu_increment_nanoseconds,
                reading.wait_increment_nanoseconds,
                reading.write_wait_increment_nanoseconds,
                reading.cpu_change,
                reading.wait_change,
                reading.paired_rounds,
                optional_number(reading.cpu_increment_paired_median_nanoseconds),
                optional_number(reading.write_wait_increment_paired_median_nanoseconds),
            ));
            if !holds {
                summary.positive_control_failures.push(format!(
                    "row_one_{arm_of_the_question}_arm_on_{}",
                    arm.label()
                ));
            }
        }
    }
}

/// 峰值、为正的个数、期末值（登记第八节「轨迹」：被谓词消费的量报这三样）。
fn trajectory_text(name: &str, values: &[f64]) -> String {
    let peak = values.iter().copied().reduce(f64::max);
    let positive = values.iter().filter(|value| **value > 0.0).count();
    format!(
        "{name}_peak={} {name}_positive_batches={positive} {name}_final={} {name}_per_batch={}",
        optional_number(peak),
        optional_number(values.last().copied()),
        list_of(values)
    )
}

/// 每条有效意图逐批的轨迹：B_批（扫描读到 R 的字节）、每批写带宽、每批 ρ、每批等盘时间。
fn emit_the_trajectories(emitter: &mut Emitter, stored: &[StoredIntent]) {
    for intent in stored
        .iter()
        .filter(|intent| intent.injection == WriteInjection::Nothing)
    {
        let batch_bytes: Vec<f64> = intent
            .batch_calls()
            .map(|call| call.scan_read_bytes_in_the_region as f64)
            .collect();
        let write_bandwidths: Vec<f64> = intent
            .batch_calls()
            .map(|call| call.write_bytes as f64 / (call.wall_nanoseconds as f64 / 1e9))
            .collect();
        let rhos: Vec<f64> = intent
            .batch_calls()
            .map(|call| call.cpu_nanoseconds as f64 / call.wait_nanoseconds() as f64)
            .collect();
        let waits: Vec<f64> = intent
            .batch_calls()
            .map(|call| call.wait_nanoseconds() as f64)
            .collect();
        let rho_at_least_for_cpu = rhos
            .iter()
            .filter(|rho| **rho >= RHO_AT_LEAST_FOR_CPU)
            .count();
        let rho_at_most_for_disk = rhos
            .iter()
            .filter(|rho| **rho <= RHO_AT_MOST_FOR_DISK)
            .count();
        emitter.line(&format!(
            "name=trajectory arm={} {} batch_units={} round={} {} {} {} rho_batches_at_least_two={rho_at_least_for_cpu} rho_batches_at_most_half={rho_at_most_for_disk} {}",
            intent.arm.label(),
            intent.point.label(),
            intent.batch_units,
            intent.round,
            trajectory_text("batch_scan_bytes", &batch_bytes),
            trajectory_text("write_bytes_per_second", &write_bandwidths),
            trajectory_text("rho", &rhos),
            trajectory_text("wait_nanoseconds", &waits),
        ));
    }
}

// ───────────────────────────── 开跑前：常量回比与第七节乙类锚点 ─────────────────────────────

/// 第七节乙类锚点（登记第十三节那条 python 命令的输出原样抄来的数）：（f、K、批数、签发加各批扫描读字节、第一批搬迁副本字节）。
const INTENT_ANCHORS: [(LiveSlotFraction, u64, u64, u64, u64); 6] = [
    (LiveSlotFraction::OneQuarter, 4, 2, 655_360, 262_144),
    (LiveSlotFraction::OneQuarter, 8, 1, 524_288, 524_288),
    (LiveSlotFraction::OneQuarter, 16, 1, 524_288, 524_288),
    (LiveSlotFraction::ThreeQuarters, 4, 6, 3_538_944, 262_144),
    (LiveSlotFraction::ThreeQuarters, 8, 3, 2_359_296, 524_288),
    (LiveSlotFraction::ThreeQuarters, 16, 2, 1_835_008, 1_048_576),
];
/// （f、签发那一步扫 R 读到的字节、一条意图的搬迁副本字节之和）。
const FRACTION_ANCHORS: [(LiveSlotFraction, u64, u64); 2] = [
    (LiveSlotFraction::OneQuarter, 262_144, 524_288),
    (LiveSlotFraction::ThreeQuarters, 786_432, 1_572_864),
];
/// （GPU 段数 n、输入缓冲字节、一次调用建的缓冲字节）。
const GPU_ANCHORS: [(u64, u64, u64); 4] = [
    (1, 32_768, 32_804),
    (1024, 33_554_432, 33_574_928),
    (4096, 134_217_728, 134_299_664),
    (16_384, 536_870_912, 537_198_608),
];
/// R 一块盘的字节。
const REGION_BYTES_ANCHOR: u64 = 1_048_576;
/// PCIe 5.0 x16、x8 的峰值（四舍五入到字节 / 秒）。
const PCIE_ANCHORS: [(u32, u32, u64); 2] = [(5, 16, 63_015_384_615), (5, 8, 31_507_692_308)];

/// 常量回比（停机条款第 1 条）与第七节乙类锚点逐条打出来；交回对不上的条数（常量对不上的另记一行 `name=stop_clause`）。
fn check_the_constants_and_the_anchors(emitter: &mut Emitter) -> usize {
    let disagreeing = constants_disagreeing_with_the_crates();
    for (name, local_value, crates_value) in &disagreeing {
        emitter.line(&format!(
            "name=stop_clause clause=1 constant={name} local={local_value} crates={crates_value}"
        ));
    }
    if cfg!(feature = "gpu") {
        emitter.line("name=constant_check item=gpu_buffer_bytes status=compared");
    } else {
        emitter.line("name=constant_check item=gpu_buffer_bytes status=not_run reason=gpu_feature_not_compiled");
    }
    let mut mismatches = disagreeing.len();
    let mut anchor = |item: String, value: u64, expected: u64| {
        let holds = value == expected;
        if !holds {
            mismatches += 1;
        }
        emitter.line(&format!(
            "name=anchor item={item} value={value} expected={expected} holds={holds}"
        ));
    };
    anchor(
        "region_bytes_per_device".to_string(),
        region_bytes_per_device(),
        REGION_BYTES_ANCHOR,
    );
    for (fraction, issue_bytes, intent_copy) in FRACTION_ANCHORS {
        let data_units = fraction.data_units_per_segment();
        anchor(
            format!("issue_scan_bytes_fraction_{}", fraction.label()),
            issue_scan_bytes(data_units),
            issue_bytes,
        );
        anchor(
            format!("intent_copy_bytes_fraction_{}", fraction.label()),
            intent_copy_bytes(data_units),
            intent_copy,
        );
    }
    for (fraction, batch_units, batches, scan_bytes, first_copy) in INTENT_ANCHORS {
        let data_units = fraction.data_units_per_segment();
        let label = format!("fraction_{}_batch_units_{batch_units}", fraction.label());
        anchor(
            format!("batches_{label}"),
            batches_of_one_intent(data_units, batch_units),
            batches,
        );
        anchor(
            format!("intent_scan_bytes_{label}"),
            intent_scan_bytes(data_units, batch_units),
            scan_bytes,
        );
        anchor(
            format!("first_batch_copy_bytes_{label}"),
            first_batch_copy_bytes(data_units, batch_units),
            first_copy,
        );
    }
    for (range_count, input_bytes, buffer_bytes) in GPU_ANCHORS {
        anchor(
            format!("gpu_input_bytes_{range_count}_ranges"),
            range_count * LOCAL_GPU_SLOT_BYTES,
            input_bytes,
        );
        anchor(
            format!("gpu_buffer_bytes_{range_count}_ranges"),
            local_gpu_buffer_bytes(range_count),
            buffer_bytes,
        );
    }
    anchor(
        "cpu_crc32c_of_the_check_string".to_string(),
        u64::from(crc32_castagnoli(CHECK_STRING)),
        u64::from(CHECK_STRING_CRC32_CASTAGNOLI),
    );
    for (generation, lanes, peak) in PCIE_ANCHORS {
        let computed = pcie_link_peak_bytes_per_second(generation, lanes).map_or(0.0, f64::round);
        let computed_text = format!("{computed:.0}");
        anchor(
            format!("pcie_generation_{generation}_by_{lanes}_peak_bytes_per_second"),
            computed_text.parse::<u64>().unwrap_or(0),
            peak,
        );
    }
    mismatches
}

// ───────────────────────────── 第一段 wall-clock：入口 ─────────────────────────────

/// 开机会清空的目录：Ubuntu 的 systemd-tmpfiles（`tmp.conf` 里 `D /tmp`）开机清 `/tmp`，不论它挂在什么文件系统上。
const DIRECTORIES_CLEARED_AT_BOOT: [&str; 1] = ["/tmp"];
/// 开机会清空的文件系统种类：内容只在内存里。
const FILESYSTEM_TYPES_CLEARED_AT_BOOT: [&str; 2] = ["tmpfs", "ramfs"];

/// `st_dev` 拆成主、次设备号（glibc `gnu_dev_major` / `gnu_dev_minor` 的编码），与 `/proc/self/mountinfo` 第三栏「主:次」同一口径。
fn major_and_minor_of_the_device_number(device_number: u64) -> (u64, u64) {
    let major = ((device_number >> 8) & 0xfff) | ((device_number >> 32) & 0xffff_f000);
    let minor = (device_number & 0xff) | ((device_number >> 12) & 0xffff_ff00);
    (major, minor)
}

/// 在 `/proc/self/mountinfo` 的正文里找「主:次」等于这个设备号的那一行，交回它的文件系统种类（` - ` 之后第一栏）；找不到交回 `None`。
fn filesystem_type_of_the_device(device_number: u64, mount_information: &str) -> Option<String> {
    let (major, minor) = major_and_minor_of_the_device_number(device_number);
    let wanted = format!("{major}:{minor}");
    mount_information.lines().find_map(|line| {
        let (mount_fields, filesystem_fields) = line.split_once(" - ")?;
        if mount_fields.split(' ').nth(2)? != wanted {
            return None;
        }
        filesystem_fields.split(' ').next().map(str::to_string)
    })
}

/// 镜像目录在不在开机清空的地方：文件系统是 tmpfs / ramfs，或真路径落在 `/tmp` 下，任一样成立就是；
/// 两样都读到而都不成立才不是；读不全又没有一样成立交回 `None`。
fn is_cleared_at_boot(
    filesystem_type: Option<&str>,
    canonical_directory: Option<&Path>,
) -> Option<bool> {
    let is_a_type_cleared_at_boot =
        filesystem_type.map(|kind| FILESYSTEM_TYPES_CLEARED_AT_BOOT.contains(&kind));
    let is_under_a_directory_cleared_at_boot = canonical_directory.map(|directory| {
        DIRECTORIES_CLEARED_AT_BOOT
            .iter()
            .any(|cleared| directory.starts_with(cleared))
    });
    match (
        is_a_type_cleared_at_boot,
        is_under_a_directory_cleared_at_boot,
    ) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        (Some(false), None) | (None, Some(false)) | (None, None) => None,
    }
}

/// 第一段的配置行。`image_directory` 是相对路径时按当前目录解析（`std::fs` 本来的读法，装置不换当前目录）；
/// 行里不打这个路径（在第二台跑时那是第二台的私有目录名，产物要进仓），只打它所在文件系统的种类、在不在开机清空的地方与设备号。
fn wall_clock_configuration_line(
    image_directory: &Path,
    points: &[SamplingPoint],
    settings: &PreparationSettings,
) -> String {
    let metadata =
        std::fs::create_dir_all(image_directory).and_then(|()| std::fs::metadata(image_directory));
    let filesystem_type = metadata.as_ref().ok().and_then(|directory_metadata| {
        std::fs::read_to_string("/proc/self/mountinfo")
            .ok()
            .and_then(|mount_information| {
                filesystem_type_of_the_device(directory_metadata.dev(), &mount_information)
            })
    });
    let canonical_directory = std::fs::canonicalize(image_directory).ok();
    format!(
        "name=configuration segment=wall_clock points={} batch_sizes={} rounds={ROUNDS} image_directory_filesystem={} image_directory_is_cleared_at_boot={} image_device={} worker_threads={} cpu_clock=thread_cputime cpu_clock_identifier={CLOCK_IDENTIFIER_OF_THE_THREAD_CPU_TIME} {}",
        points.iter().map(|point| format!("{}:{}", point.fraction.label(), point.pool_data_units)).collect::<Vec<_>>().join(","),
        BATCH_SIZES.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        filesystem_type.as_deref().map_or_else(|| "unreadable".to_string(), field_token),
        is_cleared_at_boot(filesystem_type.as_deref(), canonical_directory.as_deref())
            .map_or_else(|| "unknown".to_string(), |is_cleared| format!("{is_cleared}")),
        metadata.map_or_else(|error| field_token(&error.to_string()), |directory_metadata| directory_metadata.dev().to_string()),
        worker_threads_from_the_environment(),
        preparation_configuration_fields(points, Some(image_directory), settings),
    )
}

fn run_the_wall_clock_segment(image_directory: &Path, points: &[SamplingPoint]) -> i32 {
    let settings = PreparationSettings::from_the_environment();
    let mut emitter = Emitter::new();
    emitter.line(&wall_clock_configuration_line(
        image_directory,
        points,
        &settings,
    ));
    assert_this_process_runs_one_thread("segment_start");
    if !check_the_thread_cpu_clock_resolution(&mut emitter) {
        return stop_for_the_clock_resolution(&mut emitter, "wall_clock");
    }
    let mismatches = check_the_constants_and_the_anchors(&mut emitter);
    let mut stored = Vec::new();
    let mut stop_findings: Vec<String> = Vec::new();
    if mismatches == 0 {
        let mut queue = PreparationQueue::new(
            "wall_clock",
            points,
            Some(image_directory),
            &settings,
            &run_the_preparation_work,
        );
        for point_index in 0..points.len() {
            stop_findings.extend(measure_one_sampling_point(
                &mut emitter,
                &mut stored,
                &mut queue,
                point_index,
            ));
            if !stop_findings.is_empty() {
                break;
            }
        }
    }
    let sequential_write = if mismatches == 0 && stop_findings.is_empty() {
        measure_the_device_sequential_write(&mut emitter, image_directory)
    } else {
        None
    };
    let summary = summarize_the_wall_clock_segment(&mut emitter, &stored, points, sequential_write);
    emitter.line(&format!(
        "name=verdict segment=wall_clock constant_or_anchor_mismatches={mismatches} stop_clause_findings={} positive_control_failures={} positive_control_failure_items={} discrimination_failures={} clause_wall_clock_not_rising_with_the_batch_triggered={} clause_two_cpu_paths_disagree_points={} gc_to_sequential_write_bandwidth_maximum_ratio={} gc_to_sequential_write_bandwidth_judgement={} stored_intents={}",
        stop_findings.len(),
        summary.positive_control_failures.len(),
        if summary.positive_control_failures.is_empty() { "none".to_string() } else { summary.positive_control_failures.join(",") },
        summary.discrimination_failures,
        summary.clause_wall_clock_not_rising_with_the_batch_triggered,
        summary.clause_two_cpu_paths_disagree_points,
        summary.gc_to_sequential_write_bandwidth.map_or_else(|| "none".to_string(), |(ratio, _judgement)| format!("{ratio:.6}")),
        summary.gc_to_sequential_write_bandwidth.map_or("not_run", |(_ratio, judgement)| judgement),
        stored.len(),
    ));
    emitter.finish();
    if mismatches > 0 || !stop_findings.is_empty() {
        EXIT_CODE_STOP_CLAUSE
    } else if !summary.positive_control_failures.is_empty() {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

// ───────────────────────────── 第二段 stage-split：阶段重放（登记第五节「阶段拆分」「饱和吞吐」） ─────────────────────────────

/// 每个阶段重放几次取中位（纯函数、内存里：几次只说明没有隐藏状态）。
const STAGE_REPLAYS: usize = 5;
/// 饱和吞吐每一档跑多久。
const SATURATION_SECONDS: u64 = 2;
/// D24 已定项 3 的「CPU 16 线程」。
const CPU_THREADS_OF_THE_GPU_OFFLOAD_COMPARISON: usize = 16;
/// 丙段盘大小敏感性：把单元区的分配记录平铺这么多份（4 GiB → 64 GiB）。
const DISK_SIZE_MULTIPLIER: u64 = 16;

/// 一次调用里扫描读到 R 的那几个单元的字节（按记下的读，从重放材料里取）。
fn units_read_by_the_scan(material: &CallReplayMaterial) -> Vec<Vec<u8>> {
    material
        .scan_reads_in_the_region
        .iter()
        .filter_map(|read| {
            material
                .unit_bytes_by_slot
                .get(&(read.offset / LOCAL_SLOT_BYTES))
                .cloned()
        })
        .collect()
}

/// 重放时读单元字节：先取材料里记下的，没有的从这一轮的池（调用之前的样子：签发与批不动别处的数据单元）读。
fn replay_reader<'material>(
    material: &'material CallReplayMaterial,
    pool_devices: &'material [(DeviceIdentity, SparseBlockDevice)],
) -> impl FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>> + 'material {
    move |device, slot, length| {
        material
            .unit_bytes_by_slot
            .get(&slot.0)
            .filter(|bytes| bytes.len() == length && device == DeviceIdentity(0))
            .cloned()
            .or_else(|| PoolReader::read(pool_devices, device, slot.to_device_offset(), length))
    }
}

/// 一个阶段 5 次重放的中位：（每次重复的 CPU 纳秒中位、挂钟纳秒中位、重复次数中位）；有一次线程 CPU 时钟读不到就交回 `None`（那一次作废）。
fn median_stage_timing(mut stage: impl FnMut()) -> Option<StageTiming> {
    let mut timings = Vec::new();
    for _replay in 0..STAGE_REPLAYS {
        timings.push(time_a_pure_stage(&mut stage)?);
    }
    let cpu: Vec<f64> = timings
        .iter()
        .map(|timing| timing.cpu_nanoseconds_per_repetition)
        .collect();
    let wall: Vec<f64> = timings
        .iter()
        .map(|timing| timing.wall_nanoseconds_per_repetition)
        .collect();
    let repetitions: Vec<f64> = timings
        .iter()
        .map(|timing| timing.repetitions as f64)
        .collect();
    let median_repetitions = median_of(&repetitions)?;
    Some(StageTiming {
        cpu_nanoseconds_per_repetition: median_of(&cpu)?,
        wall_nanoseconds_per_repetition: median_of(&wall)?,
        repetitions: timings
            .iter()
            .map(|timing| timing.repetitions)
            .find(|count| *count as f64 >= median_repetitions)
            .unwrap_or(0),
    })
}

/// 一次调用的阶段重放读数。甲段两个子项：`crc` 与 `crc_bytes` 是区域扫描读到的单元（登记第一节「甲段的 CPU 时间与字节」那一项），
/// `release_check_crc` 与 `release_check_crc_bytes` 是释放前读盘核读的那几份副本（第十二节 12.5 补的子项）。
struct CallStageReplay {
    crc: Option<StageTiming>,
    release_check_crc: Option<StageTiming>,
    scan: Option<StageTiming>,
    choose: Option<StageTiming>,
    choose_calls: usize,
    crc_bytes: u64,
    release_check_crc_bytes: u64,
    records_read_per_choice: u64,
    full_segments: u64,
    stop_finding: Option<String>,
}

/// 甲段两个子项的字节锚点（登记第七节「甲段的字节 = 这一批计时包装记的扫描读字节」，第十二节 12.5 补的「甲段释放前读盘核那一项的字节
/// = 这一批 `release_check_read_bytes`」）：有一条对不上就交回一句（同停机条款第 4 条那一路报），原来那一项先判；两条都对上交 `None`。
fn crc_byte_anchor_finding(
    call_index: usize,
    crc_bytes: u64,
    scan_read_bytes: u64,
    release_check_crc_bytes: u64,
    release_check_read_bytes: u64,
) -> Option<String> {
    if crc_bytes != scan_read_bytes {
        return Some(format!(
            "clause_4_crc_bytes_{crc_bytes}_differ_from_the_scan_read_bytes_{scan_read_bytes}_at_call_{call_index}"
        ));
    }
    if release_check_crc_bytes != release_check_read_bytes {
        return Some(format!(
            "clause_4_release_check_crc_bytes_{release_check_crc_bytes}_differ_from_the_release_check_read_bytes_{release_check_read_bytes}_at_call_{call_index}"
        ));
    }
    None
}

/// 照会话重放一次调用：甲（每个被读单元整单元 CRC；另一个子项是释放前读盘核那几份副本的整单元 CRC）、乙（整个 `scan_region`）、
/// 丙（签发那一步的选段那一圈）；另核停机条款第 4 条与甲段两个子项的字节锚点。
fn replay_the_stages_of_a_call(
    material: &CallReplayMaterial,
    region: Option<CompactionRegion>,
    choice: ChoiceOfTheRegion,
    batch_units: u64,
    pool_devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> CallStageReplay {
    let units = units_read_by_the_scan(material);
    let (crc_bytes, _folded) = crc_stage_over_the_units(&units);
    let scan_read_bytes: u64 = material
        .scan_reads_in_the_region
        .iter()
        .map(|read| read.length)
        .sum();
    let (release_check_crc_bytes, _release_check_folded) =
        crc_stage_over_the_units(&material.release_check_copies);
    let release_check_read_bytes: u64 = material
        .release_check_reads
        .iter()
        .map(|read| read.length)
        .sum();
    let mut stop_finding = crc_byte_anchor_finding(
        material.call_index,
        crc_bytes,
        scan_read_bytes,
        release_check_crc_bytes,
        release_check_read_bytes,
    );
    let crc = median_stage_timing(|| {
        std::hint::black_box(crc_stage_over_the_units(&units));
    });
    let release_check_crc = median_stage_timing(|| {
        std::hint::black_box(crc_stage_over_the_units(&material.release_check_copies));
    });
    let scan = region.and_then(|scanned_region| {
        let version = &material.version;
        let state = &material.state;
        let replayed = scan_region(
            scanned_region,
            &version.allocation_records,
            &version.mapping_entries,
            &version.pinned_slots,
            &state.quarantined_slots,
            replay_reader(material, pool_devices),
        );
        if counts_as_a_batch(material.kind) {
            let movable_keys: Vec<Vec<u8>> = match &replayed {
                Ok(scan) => scan
                    .movable_units()
                    .take(usize::try_from(batch_units).expect("批大小"))
                    .map(|(_placement, unit)| unit.mapping_key.clone())
                    .collect(),
                Err(_refusal) => Vec::new(),
            };
            let inconsistent = replayed.as_ref().map_or(0, |scan| {
                scan.allocations
                    .iter()
                    .filter(|allocation| matches!(allocation.class, AllocationInRegion::Inconsistent(_)))
                    .count()
            });
            if (movable_keys != material.relocated_keys || inconsistent > 0) && stop_finding.is_none() {
                stop_finding = Some(format!(
                    "clause_4_replayed_movable_keys_differ_from_the_relocated_keys_at_call_{}:replayed={}:relocated={}:inconsistent={inconsistent}",
                    material.call_index,
                    movable_keys.len(),
                    material.relocated_keys.len()
                ));
            }
        }
        median_stage_timing(|| {
            std::hint::black_box(
                scan_region(
                    scanned_region,
                    &version.allocation_records,
                    &version.mapping_entries,
                    &version.pinned_slots,
                    &state.quarantined_slots,
                    replay_reader(material, pool_devices),
                )
                .is_ok(),
            );
        })
    });
    let mut choose = None;
    let mut choose_calls = 0;
    if material.kind == CallKind::Issue {
        let replay = replay_the_greedy_choice(
            &material.version,
            &material.state,
            replay_reader(material, pool_devices),
        );
        choose_calls = replay.inputs_of_each_call.len();
        if choice == ChoiceOfTheRegion::Greedy && replay.chosen != region && stop_finding.is_none()
        {
            stop_finding = Some(format!(
                "clause_4_replayed_greedy_choice_{}_differs_from_the_issued_region_{}",
                replay.chosen.map_or_else(
                    || "none".to_string(),
                    |chosen| chosen.start_slot().0.to_string()
                ),
                region.map_or_else(
                    || "none".to_string(),
                    |issued| issued.start_slot().0.to_string()
                )
            ));
        }
        let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = material
            .version
            .pinned_slots
            .union(&material.state.quarantined_slots)
            .copied()
            .collect();
        choose = median_stage_timing(|| {
            for (region_scans, refused) in &replay.inputs_of_each_call {
                std::hint::black_box(choose_region(
                    &material.version.allocation_records,
                    material.state.unit_area,
                    &material.state.cluster_segments,
                    None,
                    &slots_that_disqualify_a_segment,
                    region_scans,
                    refused,
                    RegionChoiceCriterion::Greedy,
                ));
            }
        });
    }
    CallStageReplay {
        crc,
        release_check_crc,
        scan,
        choose,
        choose_calls,
        crc_bytes,
        release_check_crc_bytes,
        records_read_per_choice: u64::try_from(
            material
                .version
                .allocation_records
                .iter()
                .filter(|record| {
                    record.device == material.state.unit_area.device && !record.is_released
                })
                .count(),
        )
        .expect("记录条数"),
        full_segments: material.state.unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS,
        stop_finding,
    }
}

/// 饱和吞吐三档（1、16、N 线程）各一行；交回 16 线程那一档（给第三段的 Q9）与「起了几条不同的线程」的核过没过。
fn emit_the_saturation(
    emitter: &mut Emitter,
    prefix: &str,
    stage_name: &str,
    bytes_per_repetition: u64,
    stage: &(dyn Fn() + Sync),
) -> bool {
    // 饱和吞吐本身就是多线程的量：串行段到这里结束，下一个计时窗口再声明串行
    emitter.declare(DeclaredSegment::Parallel);
    let mut is_every_thread_count_distinct = true;
    let worker_threads = worker_threads_from_the_environment();
    for thread_count in [1, CPU_THREADS_OF_THE_GPU_OFFLOAD_COMPARISON, worker_threads] {
        let reading = saturated_throughput(
            thread_count,
            Duration::from_secs(SATURATION_SECONDS),
            bytes_per_repetition,
            stage,
        );
        let distinct_holds = reading.distinct_threads == thread_count;
        is_every_thread_count_distinct &= distinct_holds;
        emitter.line(&format!(
            "name=saturation {prefix} stage={stage_name} threads={thread_count} bytes_per_repetition={bytes_per_repetition} repetitions={} bytes_per_second={:.1} distinct_threads={} distinct_threads_holds={distinct_holds}",
            reading.repetitions, reading.bytes_per_second, reading.distinct_threads
        ));
    }
    is_every_thread_count_distinct
}

/// 丙段的盘大小敏感性（登记第八节）：同一份选段输入，分配记录平铺 16 份（单元区 16 倍）喂 `choose_region`，报每整段的 CPU 斜率；TB 级盘照斜率外推、标「推的」。
fn emit_the_disk_size_sensitivity(
    emitter: &mut Emitter,
    prefix: &str,
    material: &CallReplayMaterial,
    replay: &GreedyReplay,
) {
    let Some((final_scans, final_refused)) = replay.inputs_of_each_call.last() else {
        return;
    };
    let state = &material.state;
    let disqualify: BTreeSet<SlotNumber> = material
        .version
        .pinned_slots
        .union(&state.quarantined_slots)
        .copied()
        .collect();
    let time_one_choice =
        |records: &[AllocationRecord], unit_area: UnitAreaOfOneDevice, scans: &[RegionScan]| {
            median_stage_timing(|| {
                std::hint::black_box(choose_region(
                    records,
                    unit_area,
                    &state.cluster_segments,
                    None,
                    &disqualify,
                    scans,
                    final_refused,
                    RegionChoiceCriterion::Greedy,
                ));
            })
        };
    let small = time_one_choice(
        &material.version.allocation_records,
        state.unit_area,
        final_scans,
    );
    let mut tiled_records = Vec::new();
    for copy in 0..DISK_SIZE_MULTIPLIER {
        tiled_records.extend(material.version.allocation_records.iter().map(|record| {
            AllocationRecord {
                device: record.device,
                slot: SlotNumber(record.slot.0 + copy * state.unit_area.slots),
                span_slots: record.span_slots,
                generation: record.generation,
                is_released: record.is_released,
            }
        }));
    }
    let large_area = UnitAreaOfOneDevice {
        device: state.unit_area.device,
        start: state.unit_area.start,
        slots: state.unit_area.slots * DISK_SIZE_MULTIPLIER,
    };
    let mut large_scans =
        region_scans_of_the_segments_without_an_unreleased_record(&tiled_records, large_area);
    large_scans.extend(
        final_scans
            .iter()
            .filter(|scan| !scan.allocations.is_empty())
            .cloned(),
    );
    let large = time_one_choice(&tiled_records, large_area, &large_scans);
    let small_segments = state.unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS;
    let large_segments = large_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS;
    let slope = match (small, large) {
        (Some(small_timing), Some(large_timing)) => Some(
            (large_timing.cpu_nanoseconds_per_repetition
                - small_timing.cpu_nanoseconds_per_repetition)
                / (large_segments - small_segments) as f64,
        ),
        (None, _) | (_, None) => None,
    };
    emitter.line(&format!(
        "name=choose_disk_size {prefix} scanned_segments_before_choosing={} small_full_segments={small_segments} small_records={} small_cpu_nanoseconds_per_call={} large_full_segments={large_segments} large_records={} large_cpu_nanoseconds_per_call={} cpu_nanoseconds_per_full_segment={} one_tebibyte_extrapolated_nanoseconds_per_call={} extrapolation=inferred",
        replay.scanned_regions.len(),
        material.version.allocation_records.len(),
        optional_number(small.map(|timing| timing.cpu_nanoseconds_per_repetition)),
        tiled_records.len(),
        optional_number(large.map(|timing| timing.cpu_nanoseconds_per_repetition)),
        optional_number(slope),
        optional_number(slope.map(|per_segment| per_segment * ((1_u64 << 40) / (LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_SLOT_BYTES)) as f64)),
    ));
}

/// 第二行阶段对照（登记第五节阳性对照表第 2 行）：三个阶段计时器各加一段 2 ms 忙等，各跑一次；甲另算两遍。交回没过的几条。
fn emit_the_stage_controls(
    emitter: &mut Emitter,
    material: &CallReplayMaterial,
    region: CompactionRegion,
    pool_devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Vec<String> {
    let units = units_read_by_the_scan(material);
    let injected = Duration::from_millis(INJECTED_MILLISECONDS_PER_WRITE);
    let crc_stage = || {
        std::hint::black_box(crc_stage_over_the_units(&units));
    };
    let scan_stage = || {
        std::hint::black_box(
            scan_region(
                region,
                &material.version.allocation_records,
                &material.version.mapping_entries,
                &material.version.pinned_slots,
                &material.state.quarantined_slots,
                replay_reader(material, pool_devices),
            )
            .is_ok(),
        );
    };
    let disqualify: BTreeSet<SlotNumber> = material
        .version
        .pinned_slots
        .union(&material.state.quarantined_slots)
        .copied()
        .collect();
    let choose_stage = || {
        std::hint::black_box(choose_region(
            &material.version.allocation_records,
            material.state.unit_area,
            &material.state.cluster_segments,
            None,
            &disqualify,
            &[],
            &material.state.segments_refused_by_the_region_scan,
            RegionChoiceCriterion::Greedy,
        ));
    };
    let mut failures = Vec::new();
    let stages: [(&str, &dyn Fn()); 3] = [
        ("crc", &crc_stage),
        ("scan", &scan_stage),
        ("choose", &choose_stage),
    ];
    for (stage_name, stage) in stages {
        let plain = median_stage_timing(stage);
        let with_the_wait = median_stage_timing(|| {
            stage();
            busy_wait_for(injected);
        });
        let increment = match (plain, with_the_wait) {
            (Some(plain_timing), Some(waited_timing)) => Some(
                waited_timing.cpu_nanoseconds_per_repetition
                    - plain_timing.cpu_nanoseconds_per_repetition,
            ),
            (None, _) | (_, None) => None,
        };
        let holds = increment.is_some_and(stage_control_holds);
        if !holds {
            failures.push(format!("row_two_{stage_name}"));
        }
        emitter.line(&format!(
            "name=positive_control row=2 stage={stage_name} increment_nanoseconds={} injected_nanoseconds={} holds={holds}",
            optional_number(increment),
            INJECTED_MILLISECONDS_PER_WRITE * 1_000_000
        ));
    }
    let once = median_stage_timing(crc_stage);
    let twice = median_stage_timing(|| {
        crc_stage();
        crc_stage();
    });
    let double_holds = match (once, twice) {
        (Some(once_timing), Some(twice_timing)) => double_crc_control_holds(
            twice_timing.cpu_nanoseconds_per_repetition,
            once_timing.cpu_nanoseconds_per_repetition,
        ),
        (None, _) | (_, None) => false,
    };
    if !double_holds {
        failures.push("row_two_crc_twice".to_string());
    }
    emitter.line(&format!(
        "name=positive_control row=2 stage=crc_twice once_nanoseconds={} twice_nanoseconds={} holds={double_holds}",
        optional_number(once.map(|timing| timing.cpu_nanoseconds_per_repetition)),
        optional_number(twice.map(|timing| timing.cpu_nanoseconds_per_repetition)),
    ));
    failures
}

/// 第二段一次调用的份额与吞吐（Q5–Q8）：份额的分母是这一次被测调用的 CPU（线程 CPU 时钟），分子是阶段重放每次重复的 CPU。
/// Q5 与 Q8 只用甲段原来那一项（区域扫描读到的单元）；释放前读盘核那一项与两项之和另列在同一行末尾，不进判定（登记第十二节 12.5）。
fn emit_the_stage_shares(
    emitter: &mut Emitter,
    prefix: &str,
    call: &CallReading,
    material: &CallReplayMaterial,
    replay: &CallStageReplay,
    batches_of_the_intent: u64,
    median_batch_cpu: Option<f64>,
) -> (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
) {
    let call_cpu = call.cpu_nanoseconds as f64;
    let crc_cpu = replay
        .crc
        .map(|timing| timing.cpu_nanoseconds_per_repetition);
    let scan_cpu = replay
        .scan
        .map(|timing| timing.cpu_nanoseconds_per_repetition);
    let scan_increment = match (scan_cpu, crc_cpu) {
        (Some(scan), Some(crc)) => Some(scan - crc),
        (None, _) | (_, None) => None,
    };
    let choose_cpu = replay
        .choose
        .map(|timing| timing.cpu_nanoseconds_per_repetition);
    let release_check_crc_cpu = replay
        .release_check_crc
        .map(|timing| timing.cpu_nanoseconds_per_repetition);
    let crc_with_the_release_check_cpu = match (crc_cpu, release_check_crc_cpu) {
        (Some(crc), Some(release_check_crc)) => Some(crc + release_check_crc),
        (None, _) | (_, None) => None,
    };
    let crc_share = crc_cpu.map(|crc| crc / call_cpu);
    let scan_increment_share = scan_increment.map(|increment| increment / call_cpu);
    let choose_share_per_batch = match (choose_cpu, median_batch_cpu) {
        (Some(choose), Some(batch_cpu)) if batches_of_the_intent > 0 => {
            Some(choose / batches_of_the_intent as f64 / batch_cpu)
        }
        (Some(_), Some(_)) | (None, _) | (_, None) => None,
    };
    let crc_bytes = replay.crc_bytes as f64;
    let choose_bytes = (replay.records_read_per_choice * LOCAL_ALLOCATION_RECORD_BYTES) as f64
        * replay.choose_calls as f64;
    emitter.line(&format!(
        "name=stage {prefix} call_index={} kind={} call_cpu_nanoseconds={} crc_cpu_nanoseconds={} crc_wall_nanoseconds={} crc_repetitions={} scan_cpu_nanoseconds={} scan_increment_cpu_nanoseconds={} choose_cpu_nanoseconds={} choose_calls={} crc_bytes={} records_read_per_choice={} full_segments={} crc_share={} scan_increment_share={} choose_share_per_batch={} choose_share_of_the_issue={} crc_bytes_per_second={} scan_increment_bytes_per_second={} choose_record_bytes_per_second={} choose_records_per_second={} choose_full_segments_per_second={} snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds={} release_check_crc_wall_nanoseconds={} release_check_crc_repetitions={} release_check_crc_bytes={} crc_with_the_release_check_cpu_nanoseconds={} crc_with_the_release_check_bytes={}",
        material.call_index,
        material.kind.label(),
        call.cpu_nanoseconds,
        optional_number(crc_cpu),
        optional_number(replay.crc.map(|timing| timing.wall_nanoseconds_per_repetition)),
        replay.crc.map_or(0, |timing| timing.repetitions),
        optional_number(scan_cpu),
        optional_number(scan_increment),
        optional_number(choose_cpu),
        replay.choose_calls,
        replay.crc_bytes,
        replay.records_read_per_choice,
        replay.full_segments,
        optional_number(crc_share),
        optional_number(scan_increment_share),
        optional_number(choose_share_per_batch),
        optional_number(choose_cpu.map(|choose| choose / call_cpu)),
        optional_number(crc_cpu.map(|crc| crc_bytes / (crc / 1e9))),
        optional_number(scan_increment.map(|increment| crc_bytes / (increment / 1e9))),
        optional_number(choose_cpu.map(|choose| choose_bytes / (choose / 1e9))),
        optional_number(choose_cpu.map(|choose| (replay.records_read_per_choice as f64 * replay.choose_calls as f64) / (choose / 1e9))),
        optional_number(choose_cpu.map(|choose| (replay.full_segments as f64 * replay.choose_calls as f64) / (choose / 1e9))),
        optional_number(release_check_crc_cpu),
        optional_number(replay.release_check_crc.map(|timing| timing.wall_nanoseconds_per_repetition)),
        replay.release_check_crc.map_or(0, |timing| timing.repetitions),
        replay.release_check_crc_bytes,
        optional_number(crc_with_the_release_check_cpu),
        replay.crc_bytes + replay.release_check_crc_bytes,
    ));
    (
        crc_share.map(|share| judgement_of_share(share, STAGE_SHARE_AT_LEAST_FOR_WORTH)),
        scan_increment_share.map(|share| judgement_of_share(share, STAGE_SHARE_AT_LEAST_FOR_WORTH)),
        choose_share_per_batch
            .map(|share| judgement_of_share(share, STAGE_SHARE_AT_LEAST_FOR_WORTH)),
    )
}

/// 第二段一个取样点、一个 K、一轮：量一条意图（收重放材料），逐次调用重放三个阶段；第一轮另量饱和吞吐、盘大小敏感性，第一个点第一轮另跑第二行对照。
/// 交回（停机条款第 4 条的触发、对照没过的几条、这一轮每批的 Q5/Q6/Q7 判定（批序号 → 判定））。
#[allow(
    clippy::type_complexity,
    reason = "交回值是三样各自独立的东西：停机触发、对照失败、按批序号的三个判定"
)]
fn stage_split_one_intent(
    emitter: &mut Emitter,
    point: SamplingPoint,
    prepared_choice: ChoiceOfTheRegion,
    batch_units: u64,
    round: usize,
    pool: &ConstructedPool,
    with_the_controls: bool,
) -> (
    Vec<String>,
    Vec<String>,
    Vec<(usize, [Option<&'static str>; 3])>,
) {
    let switches = measured_switches(batch_units, prepared_choice.forced_region());
    let mut stop_findings = Vec::new();
    let mut control_failures = Vec::new();
    let mut judgements = Vec::new();
    for attempt in 1..=ATTEMPTS_PER_ROUND_AT_MOST {
        enter_the_timing_window(emitter, TimingWindow::StageSplitCompactionRun);
        let reading = measure_on_the_memory_device(
            pool,
            switches,
            WriteInjection::Nothing,
            ReplayCapture::On,
        );
        let label = MeasurementLabel {
            arm: DeviceArm::Memory,
            point,
            batch_units,
            round,
            attempt,
            injection: WriteInjection::Nothing,
            choice: prepared_choice,
        };
        let invalid_reason = invalid_reason_of(&reading);
        emit_the_intent(emitter, &label, &reading, invalid_reason);
        if invalid_reason.is_some() {
            continue;
        }
        let prefix = format!("{} batch_units={batch_units} round={round}", point.label());
        let batch_cpus: Vec<f64> = reading
            .calls
            .iter()
            .filter(|call| counts_as_a_batch(call.kind))
            .map(|call| call.cpu_nanoseconds as f64)
            .collect();
        let batches_of_the_intent = u64::try_from(batch_cpus.len()).expect("批数");
        let median_batch_cpu = median_of(&batch_cpus);
        let mut batch_index = 0_usize;
        for material in &reading.replay_material {
            let call = &reading.calls[material.call_index];
            enter_the_timing_window(emitter, TimingWindow::StageReplay);
            let replay = replay_the_stages_of_a_call(
                material,
                reading.region,
                prepared_choice,
                batch_units,
                &pool.devices,
            );
            if let Some(finding) = &replay.stop_finding {
                emitter.line(&format!(
                    "name=stop_clause clause=4 {prefix} finding={finding}"
                ));
                stop_findings.push(finding.clone());
            }
            let (crc_share_judgement, scan_increment_share_judgement, choose_share_judgement) =
                emit_the_stage_shares(
                    emitter,
                    &prefix,
                    call,
                    material,
                    &replay,
                    batches_of_the_intent,
                    median_batch_cpu,
                );
            if counts_as_a_batch(material.kind) {
                judgements.push((
                    batch_index,
                    [
                        crc_share_judgement,
                        scan_increment_share_judgement,
                        choose_share_judgement,
                    ],
                ));
                batch_index += 1;
            }
            if round == 1 {
                let units = units_read_by_the_scan(material);
                if counts_as_a_batch(material.kind) && batch_index == 1 {
                    if let Some(region) = reading.region {
                        let crc_stage = || {
                            std::hint::black_box(crc_stage_over_the_units(&units));
                        };
                        let scan_stage = || {
                            std::hint::black_box(
                                scan_region(
                                    region,
                                    &material.version.allocation_records,
                                    &material.version.mapping_entries,
                                    &material.version.pinned_slots,
                                    &material.state.quarantined_slots,
                                    |_device, slot, length| {
                                        material
                                            .unit_bytes_by_slot
                                            .get(&slot.0)
                                            .filter(|bytes| bytes.len() == length)
                                            .cloned()
                                    },
                                )
                                .is_ok(),
                            );
                        };
                        let bytes = replay.crc_bytes;
                        if !emit_the_saturation(emitter, &prefix, "crc", bytes, &crc_stage)
                            || !emit_the_saturation(emitter, &prefix, "scan", bytes, &scan_stage)
                        {
                            control_failures.push("saturation_threads_not_distinct".to_string());
                        }
                        if with_the_controls {
                            enter_the_timing_window(emitter, TimingWindow::StageControls);
                            control_failures.extend(emit_the_stage_controls(
                                emitter,
                                material,
                                region,
                                &pool.devices,
                            ));
                        }
                    }
                }
                if material.kind == CallKind::Issue {
                    let greedy = replay_the_greedy_choice(
                        &material.version,
                        &material.state,
                        replay_reader(material, &pool.devices),
                    );
                    let disqualify: BTreeSet<SlotNumber> = material
                        .version
                        .pinned_slots
                        .union(&material.state.quarantined_slots)
                        .copied()
                        .collect();
                    let choose_stage = || {
                        for (region_scans, refused) in &greedy.inputs_of_each_call {
                            std::hint::black_box(choose_region(
                                &material.version.allocation_records,
                                material.state.unit_area,
                                &material.state.cluster_segments,
                                None,
                                &disqualify,
                                region_scans,
                                refused,
                                RegionChoiceCriterion::Greedy,
                            ));
                        }
                    };
                    let choose_bytes = replay.records_read_per_choice
                        * LOCAL_ALLOCATION_RECORD_BYTES
                        * u64::try_from(greedy.inputs_of_each_call.len()).expect("调用次数");
                    if !emit_the_saturation(emitter, &prefix, "choose", choose_bytes, &choose_stage)
                    {
                        control_failures.push("saturation_threads_not_distinct".to_string());
                    }
                    enter_the_timing_window(emitter, TimingWindow::DiskSizeSensitivity);
                    emit_the_disk_size_sensitivity(emitter, &prefix, material, &greedy);
                }
            }
        }
        return (stop_findings, control_failures, judgements);
    }
    emitter.line(&format!(
        "name=round_invalid arm=memory {} batch_units={batch_units} round={round} attempts={ATTEMPTS_PER_ROUND_AT_MOST}",
        point.label()
    ));
    (stop_findings, control_failures, judgements)
}

fn run_the_stage_split_segment(points: &[SamplingPoint]) -> i32 {
    let settings = PreparationSettings::from_the_environment();
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=configuration segment=stage_split points={} batch_sizes={} rounds={ROUNDS} stage_replays={STAGE_REPLAYS} stage_timed_runs_per_replay=1 saturation_seconds={SATURATION_SECONDS} worker_threads={} {}",
        points.iter().map(|point| format!("{}:{}", point.fraction.label(), point.pool_data_units)).collect::<Vec<_>>().join(","),
        BATCH_SIZES.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        worker_threads_from_the_environment(),
        preparation_configuration_fields(points, None, &settings),
    ));
    assert_this_process_runs_one_thread("segment_start");
    if !check_the_thread_cpu_clock_resolution(&mut emitter) {
        return stop_for_the_clock_resolution(&mut emitter, "stage_split");
    }
    let mismatches = check_the_constants_and_the_anchors(&mut emitter);
    let mut stop_findings: Vec<String> = Vec::new();
    let mut control_failures: Vec<String> = Vec::new();
    let mut is_the_first_intent = true;
    if mismatches == 0 {
        let mut queue = PreparationQueue::new(
            "stage_split",
            points,
            None,
            &settings,
            &run_the_preparation_work,
        );
        'points: for point_index in 0..points.len() {
            // 从准备队列取这一点造好的池（还没好就跑一次准备），量完放掉（`finish_the_point`）
            let prepared = queue.prepared_point(point_index, &mut emitter);
            let point = &prepared.recipe.point;
            for line in &prepared.lines {
                emitter.line(line);
            }
            let choice = prepared.choice;
            let mut rebuilt_pool_of_the_round: Option<ConstructedPool> = None;
            let mut judgements_by_cell: BTreeMap<(u64, usize), Vec<[Option<&'static str>; 3]>> =
                BTreeMap::new();
            for round in 1..=ROUNDS {
                wait_until_no_other_measurement_runs(&mut emitter);
                if round > 1 {
                    let rebuilt = queue.take_the_rebuilt_pool(point_index, round, &mut emitter);
                    let is_identical = rebuilt.digests == prepared.reference_digests;
                    emitter.line(&format!(
                        "name=memory_rebuild {} round={round} construction_seconds={:.3} digests_identical={is_identical}",
                        point.label(),
                        rebuilt.pool.construction_seconds
                    ));
                    rebuilt_pool_of_the_round = is_identical.then_some(rebuilt.pool);
                }
                let pool_of_the_round = if round == 1 {
                    Some(&prepared.pool)
                } else {
                    rebuilt_pool_of_the_round.as_ref()
                };
                let Some(pool) = pool_of_the_round else {
                    emitter.line(&format!("name=round_invalid arm=memory {} round={round} reason=start_image_digest_differs", point.label()));
                    continue;
                };
                for batch_units in BATCH_SIZES {
                    let (findings, failures, judgements) = stage_split_one_intent(
                        &mut emitter,
                        *point,
                        choice,
                        batch_units,
                        round,
                        pool,
                        is_the_first_intent,
                    );
                    is_the_first_intent = false;
                    control_failures.extend(failures);
                    for (batch_index, cell) in judgements {
                        judgements_by_cell
                            .entry((batch_units, batch_index))
                            .or_default()
                            .push(cell);
                    }
                    if !findings.is_empty() {
                        stop_findings.extend(findings);
                        break 'points;
                    }
                }
            }
            for ((batch_units, batch_index), cells) in &judgements_by_cell {
                let stable = |position: usize| {
                    let per_round: Vec<&'static str> = cells
                        .iter()
                        .map(|cell| cell[position].unwrap_or("not_run"))
                        .collect();
                    (per_round.join(","), judgement_of_the_rounds(&per_round))
                };
                let (crc_share_judgement_per_round, crc_share_judgement) = stable(0);
                let (scan_increment_share_judgement_per_round, scan_increment_share_judgement) =
                    stable(1);
                let (choose_share_judgement_per_round, choose_share_judgement) = stable(2);
                emitter.line(&format!(
                    "name=stage_judgement {} batch_units={batch_units} batch_index={batch_index} rounds={} crc_share_judgement_per_round={crc_share_judgement_per_round} crc_share_judgement={crc_share_judgement} scan_increment_share_judgement_per_round={scan_increment_share_judgement_per_round} scan_increment_share_judgement={scan_increment_share_judgement} choose_share_judgement_per_round={choose_share_judgement_per_round} choose_share_judgement={choose_share_judgement}",
                    point.label(),
                    cells.len()
                ));
            }
            queue.finish_the_point(point_index);
        }
    }
    emitter.line(&format!(
        "name=verdict segment=stage_split constant_or_anchor_mismatches={mismatches} stop_clause_findings={} positive_control_failures={} positive_control_failure_items={}",
        stop_findings.len(),
        control_failures.len(),
        if control_failures.is_empty() { "none".to_string() } else { control_failures.join(",") },
    ));
    emitter.finish();
    if mismatches > 0 || !stop_findings.is_empty() {
        EXIT_CODE_STOP_CLAUSE
    } else if !control_failures.is_empty() {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

// ───────────────────────────── 第三段 gpu-submission（登记第五节「GPU 一侧怎么量」；只在编了 gpu 特性时有） ─────────────────────────────

/// 一个产物文件里的 `E7RESULT` 行，各拆成「键 → 值」（第三段读第一、二段的产物）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(dead_code, reason = "只有第三段读产物，没编 gpu 特性时只在单测里用")
)]
fn result_lines_of(text: &str) -> Vec<BTreeMap<String, String>> {
    text.lines()
        .filter_map(|line| line.strip_prefix("E7RESULT "))
        .map(|fields| {
            fields
                .split(' ')
                .filter_map(|field| field.split_once('='))
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect()
        })
        .collect()
}

#[cfg(feature = "gpu")]
mod gpu_submission {
    use super::*;
    use singlefs_checker_tier::gpu_unit_checks::{
        bytes_crc32c_of_ranges_needs, gpu_cards_by_priority, GpuCard,
    };

    /// 段数 n 的取值（登记第五节）。
    const RANGE_COUNTS: [u64; 8] = [1, 4, 16, 64, 256, 1024, 4096, 16_384];
    /// 每段的字节：一个整数据单元，与码 2 节点大小那一组（`crc32c_of_ranges` 按 32 KiB 一槽上传，两组的 t 应相同，报出来、不判）。
    const RANGE_LENGTHS: [u64; 2] = [32_768, 16_384];
    /// 每档计时几次取中位（先丢一次编管线、建上下文）。
    const TIMED_SUBMISSIONS: usize = 5;
    /// 每张卡起设备时给的显存额度（MiB，登记第五节 `gpu_cards_by_priority(&[(序号, 4096)])`）。
    const CARD_QUOTA_MEBIBYTES: u64 = 4096;

    /// 造池时写的一个真数据单元的整单元字节（32768）：mkfs、第一个文件、建一个 inode、顺序写一个数据单元，从盘上读回。
    fn one_data_unit_of_the_workload() -> Vec<u8> {
        let (mut devices, mut session) = formatted_pool_with_the_first_file_and_a_session();
        let inode = version_of_trees(&session).inode_number_watermark();
        let mut floor_raises = 0_u64;
        publish_while_constructing(
            &mut session,
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS
                    + SECONDS_FROM_THE_FIRST_FILE_TO_THE_WORKLOAD,
            },
            &mut floor_raises,
        );
        let mut workload = WorkloadUnits {
            live_data_inode_by_slot: BTreeMap::new(),
            slot_of_inode: BTreeMap::new(),
            floor_raises,
        };
        workload.write_one_unit(
            &mut session,
            &mut devices,
            inode,
            0,
            common::FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_FIRST_FILE_TO_THE_WORKLOAD + 1,
        );
        let slot = workload.slot_of_inode[&inode];
        PoolReader::read(
            devices.as_slice(),
            DeviceIdentity(0),
            SlotNumber(slot).to_device_offset(),
            usize::try_from(LOCAL_DATA_UNIT_BYTES).expect("32768"),
        )
        .expect("刚写的数据单元读得回")
    }

    /// `nvidia-smi` 报的每张卡的 PCIe 链路代数与宽度（序号 → (代数, 宽度)）；读不到交回空表。
    fn pcie_links_reported_by_nvidia_smi() -> BTreeMap<u32, (u32, u32)> {
        let Ok(output) = std::process::Command::new("nvidia-smi")
            .args([
                "--query-gpu=index,pcie.link.gen.max,pcie.link.width.max",
                "--format=csv,noheader,nounits",
            ])
            .output()
        else {
            return BTreeMap::new();
        };
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let fields: Vec<u32> = line
                    .split(',')
                    .filter_map(|field| field.trim().parse().ok())
                    .collect();
                match fields.as_slice() {
                    [index, generation, width] => Some((*index, (*generation, *width))),
                    _ => None,
                }
            })
            .collect()
    }

    /// 一张卡一组（段长）一个 n 的读数：5 次计时的中位、每次是否没快过物理下限、CRC 与 CPU 是否逐段相同。
    struct SubmissionReading {
        range_count: u64,
        median_seconds: f64,
        respects_the_floor: bool,
        crcs_agree: bool,
    }

    fn time_the_submissions(
        card: &GpuCard,
        segment: &[u8],
        range_count: u64,
        link_peak: f64,
    ) -> SubmissionReading {
        let count = usize::try_from(range_count).expect("段数装得进 usize");
        let ranges: Vec<&[u8]> = vec![segment; count];
        let cpu_crc = crc32_castagnoli(segment);
        let expected = vec![cpu_crc; count];
        let _warm_up = card.crc32c_of_ranges(&ranges);
        let mut seconds = Vec::new();
        let mut respects_the_floor = true;
        let mut crcs_agree = true;
        for _submission in 0..TIMED_SUBMISSIONS {
            let started = Instant::now();
            let results = card.crc32c_of_ranges(&ranges);
            let elapsed = started.elapsed().as_secs_f64();
            respects_the_floor &=
                respects_the_physical_floor(elapsed, range_count * LOCAL_GPU_SLOT_BYTES, link_peak);
            crcs_agree &= gpu_crcs_agree_with_the_cpu(&results, &expected);
            seconds.push(elapsed);
        }
        SubmissionReading {
            range_count,
            median_seconds: median_of(&seconds).expect("计了 5 次"),
            respects_the_floor,
            crcs_agree,
        }
    }

    /// 第一段产物里每批交给 GPU 的字节 B_批（内存臂、不注入、第 1 轮的批），键（活槽占比、P、K、调用序号）。
    fn batch_bytes_of_the_wall_clock_product(
        lines: &[BTreeMap<String, String>],
    ) -> BTreeMap<(String, String, String, String), u64> {
        lines
            .iter()
            .filter(|fields| {
                fields.get("name").map(String::as_str) == Some("call")
                    && fields.get("arm").map(String::as_str) == Some("memory")
                    && fields.get("injection").map(String::as_str) == Some("none")
                    && fields.get("round").map(String::as_str) == Some("1")
                    && matches!(
                        fields.get("kind").map(String::as_str),
                        Some("batch" | "final_batch")
                    )
            })
            .filter_map(|fields| {
                Some((
                    (
                        fields.get("fraction")?.clone(),
                        fields.get("pool_data_units")?.clone(),
                        fields.get("batch_units")?.clone(),
                        fields.get("call_index")?.clone(),
                    ),
                    fields.get("scan_read_bytes_in_the_region")?.parse().ok()?,
                ))
            })
            .collect()
    }

    /// 第二段产物里的饱和吞吐，键（活槽占比、P、K、阶段、线程数）→ 字节 / 秒；另交回第二段的阶段字节（键（活槽占比、P、K、阶段）→ 字节）。
    #[allow(clippy::type_complexity, reason = "两张表各是一种量：吞吐与字节")]
    fn readings_of_the_stage_split_product(
        lines: &[BTreeMap<String, String>],
    ) -> (
        BTreeMap<(String, String, String, String, String), f64>,
        BTreeMap<(String, String, String, String), f64>,
    ) {
        let mut throughput = BTreeMap::new();
        let mut stage_bytes = BTreeMap::new();
        for fields in lines {
            let key_of = |stage: &str| -> Option<(String, String, String, String)> {
                Some((
                    fields.get("fraction")?.clone(),
                    fields.get("pool_data_units")?.clone(),
                    fields.get("batch_units")?.clone(),
                    stage.to_string(),
                ))
            };
            match fields.get("name").map(String::as_str) {
                Some("saturation") if fields.get("round").map(String::as_str) == Some("1") => {
                    let (Some(stage), Some(threads), Some(bytes_per_second)) = (
                        fields.get("stage"),
                        fields.get("threads"),
                        fields
                            .get("bytes_per_second")
                            .and_then(|value| value.parse::<f64>().ok()),
                    ) else {
                        continue;
                    };
                    if let Some((fraction, pool, batch, stage_name)) = key_of(stage) {
                        throughput.insert(
                            (fraction, pool, batch, stage_name, threads.clone()),
                            bytes_per_second,
                        );
                    }
                }
                Some("stage") if fields.get("round").map(String::as_str) == Some("1") => {
                    let kind = fields.get("kind").map(String::as_str);
                    if matches!(kind, Some("batch" | "final_batch"))
                        && !stage_bytes.contains_key(&key_of("crc").unwrap_or_default())
                    {
                        if let (Some(key), Some(bytes)) = (
                            key_of("crc"),
                            fields
                                .get("crc_bytes")
                                .and_then(|value| value.parse::<f64>().ok()),
                        ) {
                            stage_bytes.insert(key, bytes);
                        }
                    }
                    if kind == Some("issue") {
                        let records = fields
                            .get("records_read_per_choice")
                            .and_then(|value| value.parse::<f64>().ok());
                        let calls = fields
                            .get("choose_calls")
                            .and_then(|value| value.parse::<f64>().ok());
                        if let (Some(key), Some(records), Some(calls)) =
                            (key_of("choose"), records, calls)
                        {
                            stage_bytes.insert(
                                key,
                                records * LOCAL_ALLOCATION_RECORD_BYTES as f64 * calls,
                            );
                        }
                    }
                }
                Some(_) | None => {}
            }
        }
        (throughput, stage_bytes)
    }

    pub(super) fn run(
        wall_clock_product: &Path,
        stage_split_product: &Path,
        card_indices: &[u32],
    ) -> i32 {
        let mut emitter = Emitter::new();
        emitter.line(&format!(
            "name=configuration segment=gpu_submission cards={} range_counts={} range_lengths={} timed_submissions={TIMED_SUBMISSIONS}",
            card_indices.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
            RANGE_COUNTS.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
            RANGE_LENGTHS.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        ));
        let mismatches = check_the_constants_and_the_anchors(&mut emitter);
        let (Ok(wall_clock_text), Ok(stage_split_text)) = (
            std::fs::read_to_string(wall_clock_product),
            std::fs::read_to_string(stage_split_product),
        ) else {
            emitter.line(
                "name=verdict segment=gpu_submission status=not_run reason=products_unreadable",
            );
            emitter.finish();
            return EXIT_CODE_USAGE;
        };
        let batch_bytes = batch_bytes_of_the_wall_clock_product(&result_lines_of(&wall_clock_text));
        let (throughput, stage_bytes) =
            readings_of_the_stage_split_product(&result_lines_of(&stage_split_text));
        let unit = one_data_unit_of_the_workload();
        let links = pcie_links_reported_by_nvidia_smi();
        let mut card_failures: Vec<String> = Vec::new();
        let mut cards_measured = 0_usize;
        let mut fixed_share_values = Vec::new();
        for card_index in card_indices {
            let Some(card) = gpu_cards_by_priority(&[(*card_index, CARD_QUOTA_MEBIBYTES)])
                .into_iter()
                .next()
            else {
                emitter.line(&format!(
                    "name=gpu_card index={card_index} status=not_run reason=card_not_usable"
                ));
                continue;
            };
            let Some(link_peak) = links.get(card_index).and_then(|(generation, width)| {
                pcie_link_peak_bytes_per_second(*generation, *width)
            }) else {
                emitter.line(&format!(
                    "name=gpu_card index={card_index} status=invalid reason=pcie_link_unreadable"
                ));
                card_failures.push(format!("card_{card_index}_link_unreadable"));
                continue;
            };
            cards_measured += 1;
            let binding_limit = card.device().limits().max_storage_buffer_binding_size;
            let check_string_result = card.crc32c_of_ranges(&[CHECK_STRING]);
            let check_string_holds = check_string_result == [CHECK_STRING_CRC32_CASTAGNOLI];
            emitter.line(&format!(
                "name=gpu_card index={card_index} card_name={} driver={} free_memory_mebibytes={} link_peak_bytes_per_second={link_peak:.0} max_storage_buffer_binding_size={binding_limit} check_string_crc32c={} check_string_holds={check_string_holds}", // naming-lint:external max_storage_buffer_binding_size 是 wgpu 的 Limits 字段名，照原名打，读产物的人拿它直接对 wgpu 文档
                field_token(&card.name),
                field_token(&card.driver_version),
                card.free_memory_mebibytes,
                check_string_result.first().map_or_else(|| "none".to_string(), |crc| format!("{crc:#010x}")),
            ));
            let mut is_the_card_valid = check_string_holds;
            let mut fit_points = Vec::new();
            let mut median_of_one_range = None;
            for range_length in RANGE_LENGTHS {
                let segment = &unit[..usize::try_from(range_length).expect("段长")];
                for range_count in RANGE_COUNTS
                    .iter()
                    .copied()
                    .filter(|count| count * LOCAL_GPU_SLOT_BYTES <= binding_limit)
                {
                    let reading = time_the_submissions(&card, segment, range_count, link_peak);
                    is_the_card_valid &= reading.respects_the_floor && reading.crcs_agree;
                    emitter.line(&format!(
                        "name=gpu_submission index={card_index} range_length={range_length} range_count={} input_bytes={} buffer_bytes={} median_seconds={:.9} respects_the_physical_floor={} crcs_agree={}",
                        reading.range_count,
                        reading.range_count * LOCAL_GPU_SLOT_BYTES,
                        bytes_crc32c_of_ranges_needs(usize::try_from(reading.range_count).expect("段数")),
                        reading.median_seconds,
                        reading.respects_the_floor,
                        reading.crcs_agree,
                    ));
                    if range_length == LOCAL_GPU_SLOT_BYTES {
                        fit_points.push((
                            (reading.range_count * LOCAL_GPU_SLOT_BYTES) as f64,
                            reading.median_seconds,
                        ));
                        if reading.range_count == 1 {
                            median_of_one_range = Some(reading.median_seconds);
                        }
                    }
                }
            }
            if !is_the_card_valid {
                card_failures.push(format!("card_{card_index}_result_or_floor"));
            }
            let fit = least_squares_intercept_and_slope(&fit_points);
            let largest_two_slope = match fit_points.as_slice() {
                [.., (smaller_bytes, smaller_seconds), (larger_bytes, larger_seconds)] => {
                    Some((larger_seconds - smaller_seconds) / (larger_bytes - smaller_bytes))
                }
                _ => None,
            };
            emitter.line(&format!(
                "name=q11_q12 index={card_index} card_valid={is_the_card_valid} intercept_seconds={} slope_seconds_per_byte={} one_range_median_seconds={} largest_two_points_slope_seconds_per_byte={}",
                optional_number(fit.map(|(intercept, _slope)| intercept)),
                fit.map_or_else(|| "none".to_string(), |(_intercept, slope)| format!("{slope:.6e}")),
                optional_number(median_of_one_range),
                largest_two_slope.map_or_else(|| "none".to_string(), |slope| format!("{slope:.6e}")),
            ));
            let Some((intercept, slope)) = fit.filter(|_fit| is_the_card_valid) else {
                continue;
            };
            for ((fraction, pool, batch, call_index), bytes) in &batch_bytes {
                let share = fixed_overhead_share(intercept, slope, *bytes);
                fixed_share_values.push(share);
                let merge = (share > FIXED_SHARE_AT_MOST_FOR_ONE_BATCH)
                    .then(|| batches_to_merge(intercept, slope, *bytes));
                let merged_buffer_bytes = merge.map(|batches| {
                    let units_per_batch = bytes / LOCAL_GPU_SLOT_BYTES;
                    let merged_units =
                        format!("{batches:.0}").parse::<u64>().unwrap_or(0) * units_per_batch;
                    bytes_crc32c_of_ranges_needs(
                        usize::try_from(merged_units).expect("合并后的段数"),
                    )
                });
                emitter.line(&format!(
                    "name=q13 index={card_index} fraction={fraction} pool_data_units={pool} batch_units={batch} call_index={call_index} batch_bytes={bytes} fixed_overhead_share={share:.6} judgement={} batches_to_merge={} merged_buffer_bytes={} card_free_bytes={}",
                    judgement_of_fixed_share_at_the_registered_edges(share),
                    merge.map_or_else(|| "none".to_string(), |batches| format!("{batches:.0}")),
                    merged_buffer_bytes.map_or_else(|| "none".to_string(), |buffer| buffer.to_string()),
                    card.free_memory_mebibytes * 1024 * 1024,
                ));
            }
            for ((fraction, pool, batch, stage), bytes) in &stage_bytes {
                let throughput_of = |stage_name: &str, threads: &str| {
                    throughput
                        .get(&(
                            fraction.clone(),
                            pool.clone(),
                            batch.clone(),
                            stage_name.to_string(),
                            threads.to_string(),
                        ))
                        .copied()
                };
                let cpu_seconds_at = |threads: &str| -> Option<f64> {
                    match stage.as_str() {
                        "crc" => Some(bytes / throughput_of("crc", threads)?),
                        "choose" => Some(bytes / throughput_of("choose", threads)?),
                        _ => None,
                    }
                };
                let gpu_seconds = intercept + slope * bytes;
                let scan_increment_at = |threads: &str| -> Option<f64> {
                    Some(
                        bytes / throughput_of("scan", threads)?
                            - bytes / throughput_of("crc", threads)?,
                    )
                };
                for (stage_label, cpu_sixteen, cpu_one) in [
                    (stage.clone(), cpu_seconds_at("16"), cpu_seconds_at("1")),
                    (
                        "scan_increment".to_string(),
                        scan_increment_at("16"),
                        scan_increment_at("1"),
                    ),
                ] {
                    if stage_label == "scan_increment" && stage != "crc" {
                        continue;
                    }
                    let ratio_sixteen = cpu_sixteen.map(|cpu| cpu / gpu_seconds);
                    let ratio_one = cpu_one.map(|cpu| cpu / gpu_seconds);
                    let is_the_sixteen_thread_form_clause_triggered =
                        match (cpu_sixteen, cpu_one, ratio_sixteen, ratio_one) {
                            (
                                Some(sixteen),
                                Some(one),
                                Some(ratio_at_sixteen),
                                Some(ratio_at_one),
                            ) => {
                                one / sixteen < 2.0
                                    || judgement_of_the_sixteen_thread_to_gpu_ratio(
                                        ratio_at_sixteen,
                                    ) != judgement_of_the_sixteen_thread_to_gpu_ratio(
                                        ratio_at_one,
                                    )
                            }
                            (None, _, _, _)
                            | (_, None, _, _)
                            | (_, _, None, _)
                            | (_, _, _, None) => false,
                        };
                    emitter.line(&format!(
                        "name=q9 index={card_index} fraction={fraction} pool_data_units={pool} batch_units={batch} stage={stage_label} stage_bytes={bytes:.0} cpu_sixteen_threads_seconds={} gpu_seconds={gpu_seconds:.9} ratio={} judgement={} cpu_one_thread_seconds={} ratio_one_thread={} judgement_one_thread={} clause_sixteen_thread_form_misfits_the_gc_triggered={is_the_sixteen_thread_form_clause_triggered}",
                        optional_number(cpu_sixteen),
                        optional_number(ratio_sixteen),
                        ratio_sixteen.map_or("not_run", judgement_of_the_sixteen_thread_to_gpu_ratio),
                        optional_number(cpu_one),
                        optional_number(ratio_one),
                        ratio_one.map_or("not_run", judgement_of_the_sixteen_thread_to_gpu_ratio),
                    ));
                }
            }
        }
        let discrimination_holds = emit_the_discrimination(
            &mut emitter,
            "Q13",
            &fixed_share_values,
            |value, threshold| judgement_of_fixed_share(value, threshold, f64::INFINITY),
        );
        emitter.line(&format!(
            "name=verdict segment=gpu_submission constant_or_anchor_mismatches={mismatches} cards_measured={cards_measured} card_failures={} card_failure_items={} discrimination_holds={discrimination_holds}",
            card_failures.len(),
            if card_failures.is_empty() { "none".to_string() } else { card_failures.join(",") },
        ));
        emitter.finish();
        if mismatches > 0 {
            EXIT_CODE_STOP_CLAUSE
        } else if cards_measured == 0 {
            EXIT_CODE_NOT_RUN
        } else if !card_failures.is_empty() || !discrimination_holds {
            EXIT_CODE_POSITIVE_CONTROL_FAILED
        } else {
            EXIT_CODE_OK
        }
    }
}

// ───────────────────────────── 第四段 foreground-fsync（附带，登记第五节分段表第四段；只在 Q16 > 10% 时跑） ─────────────────────────────

/// 前台一次写的字节。
const FOREGROUND_WRITE_BYTES: u64 = 4096;
/// 前台写的镜像文件多大（写在里面轮着走）。
const FOREGROUND_IMAGE_BYTES: u64 = 64 << 20;
/// 每轮默认几秒（登记第五节：各 5 轮、每轮 60 秒）。
const FOREGROUND_SECONDS_DEFAULT: u64 = 60;

/// 前台循环「4 KiB `O_DIRECT` 写 + `fdatasync`」到 `deadline`，交回每次的延迟（纳秒）。
fn foreground_write_latencies(path: &Path, deadline: Instant) -> Result<Vec<f64>, String> {
    let mut device = DirectInputOutputBlockDevice::open_existing(
        path,
        PageCachePolicy::BypassWithDirectInputOutput,
        PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)),
    )
    .map_err(|error| field_token(&format!("open_failed:{error:?}")))?;
    let block: Vec<u8> = (0..FOREGROUND_WRITE_BYTES)
        .map(|position| u8::try_from(position % 241).expect("小于 241"))
        .collect();
    let mut latencies = Vec::new();
    let mut write_index = 0_u64;
    // 迭代上界：挂钟到 `deadline` 就停。
    while Instant::now() < deadline {
        let offset = (write_index % (FOREGROUND_IMAGE_BYTES / FOREGROUND_WRITE_BYTES))
            * FOREGROUND_WRITE_BYTES;
        let started = Instant::now();
        device
            .write_at(DeviceOffsetInBytes(offset), &block, WriteDurability::Plain)
            .map_err(|error| field_token(&format!("write_failed:{error:?}")))?;
        device
            .barrier()
            .map_err(|error| field_token(&format!("barrier_failed:{error:?}")))?;
        latencies.push(started.elapsed().as_nanos() as f64);
        write_index += 1;
    }
    Ok(latencies)
}

/// GC 那条线程一轮做了什么：调了几次、发了几次、搬了几批、在挂钟到之前停没停（停了之后这一轮余下的时间 GC 不在跑）。
struct BackgroundCompaction {
    calls: u64,
    publishes: u64,
    batches: u64,
    stopped_before_the_deadline_after_seconds: Option<f64>,
}

/// 在真盘工作件上可写挂载、等前台就位，然后一次一次调 `run_compaction` 到 `seconds` 秒。
fn background_compaction(
    images: &mut RealDiskImages,
    switches: CompactionSwitches,
    start_together: &std::sync::Barrier,
    seconds: u64,
) -> Result<BackgroundCompaction, String> {
    let restored = images.restore_and_check();
    let ledger = Rc::new(RefCell::new(CallLedger::new(WriteInjection::Nothing)));
    let opened = restored.and_then(|_chunks| images.open_timed(&ledger));
    let mounted = opened.and_then(|mut devices| {
        mount_writable_with_compaction_switches(&common::parameters(), &mut devices, switches)
            .map(|mounted| (devices, mounted))
            .map_err(|error| field_token(&format!("mount_failed:{error:?}")))
    });
    start_together.wait();
    let (mut devices, mounted) = mounted?;
    let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
    let started = Instant::now();
    let deadline = started + Duration::from_secs(seconds);
    let mut background = BackgroundCompaction {
        calls: 0,
        publishes: 0,
        batches: 0,
        stopped_before_the_deadline_after_seconds: None,
    };
    // 迭代上界：挂钟到 `deadline` 就停；整理停了（没有可选的段、没有产出）也停。
    while Instant::now() < deadline {
        session.compaction.publishes_since_the_last_user_change =
            PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
        let Ok(run) = session.run_compaction(&mut devices) else {
            background.stopped_before_the_deadline_after_seconds =
                Some(started.elapsed().as_secs_f64());
            break;
        };
        background.calls += 1;
        background.publishes += u64::try_from(run.versions_published.len()).expect("发布次数");
        background.batches += run.batches_published;
        if run.versions_published.is_empty() {
            background.stopped_before_the_deadline_after_seconds =
                Some(started.elapsed().as_secs_f64());
            break;
        }
    }
    drop(devices);
    images.note_the_changed_chunks(&ledger);
    Ok(background)
}

fn run_the_foreground_fsync_segment(
    image_directory: &Path,
    point: SamplingPoint,
    batch_units: u64,
    seconds: u64,
) -> i32 {
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=configuration segment=foreground_fsync {} batch_units={batch_units} seconds_per_round={seconds} rounds={ROUNDS} foreground_write_bytes={FOREGROUND_WRITE_BYTES}",
        point.label()
    ));
    if !check_the_thread_cpu_clock_resolution(&mut emitter) {
        return stop_for_the_clock_resolution(&mut emitter, "foreground_fsync");
    }
    let mismatches = check_the_constants_and_the_anchors(&mut emitter);
    if mismatches > 0 {
        emitter.line(&format!(
            "name=verdict segment=foreground_fsync constant_or_anchor_mismatches={mismatches}"
        ));
        emitter.finish();
        return EXIT_CODE_STOP_CLAUSE;
    }
    // 第四段只有一个点、一份池，准备照串行做（不起准备线程）。
    let prepared = prepare_the_sampling_point(point);
    for line in &prepared.lines {
        emitter.line(line);
    }
    let switches = measured_switches(batch_units, prepared.choice.forced_region());
    let mut images = match RealDiskImages::create(image_directory, point, &prepared.pool) {
        Ok(images) => images,
        Err(error) => {
            emitter.line(&format!(
                "name=verdict segment=foreground_fsync status=not_run reason={}",
                field_token(&error.to_string())
            ));
            emitter.finish();
            return EXIT_CODE_NOT_RUN;
        }
    };
    drop(prepared);
    let foreground_path = image_directory.join("e185-foreground.img");
    if foreground_path.exists() {
        std::fs::remove_file(&foreground_path).ok();
    }
    if let Err(error) = create_a_zero_filled_image(&foreground_path, FOREGROUND_IMAGE_BYTES) {
        emitter.line(&format!(
            "name=verdict segment=foreground_fsync status=not_run reason={}",
            field_token(&error.to_string())
        ));
        emitter.finish();
        return EXIT_CODE_NOT_RUN;
    }
    let mut judgements = Vec::new();
    for round in 1..=ROUNDS {
        wait_until_no_other_measurement_runs(&mut emitter);
        let quiet = foreground_write_latencies(
            &foreground_path,
            Instant::now() + Duration::from_secs(seconds),
        );
        let start_together = std::sync::Barrier::new(2);
        let (concurrent, background) = std::thread::scope(|scope| {
            let compaction = scope
                .spawn(|| background_compaction(&mut images, switches, &start_together, seconds));
            start_together.wait();
            let latencies = foreground_write_latencies(
                &foreground_path,
                Instant::now() + Duration::from_secs(seconds),
            );
            (
                latencies,
                compaction.join().expect("GC 那条线程跑完、没 panic"),
            )
        });
        match (quiet, concurrent, background) {
            (Ok(quiet_latencies), Ok(concurrent_latencies), Ok(background_run)) => {
                let quiet_percentile_99 = percentile_of(&quiet_latencies, 99);
                let concurrent_percentile_99 = percentile_of(&concurrent_latencies, 99);
                let ratio = match (quiet_percentile_99, concurrent_percentile_99) {
                    (Some(quiet_value), Some(concurrent_value)) => {
                        Some(concurrent_value / quiet_value)
                    }
                    (None, _) | (_, None) => None,
                };
                let judgement =
                    ratio.map_or("not_run", judgement_of_the_foreground_fsync_latency_ratio);
                judgements.push(judgement);
                emitter.line(&format!(
                    "name=q17_round round={round} quiet_writes={} quiet_percentile_99_nanoseconds={} concurrent_writes={} concurrent_percentile_99_nanoseconds={} ratio={} judgement={judgement} gc_calls={} gc_publishes={} gc_batches={} gc_stopped_after_seconds={}",
                    quiet_latencies.len(),
                    optional_number(quiet_percentile_99),
                    concurrent_latencies.len(),
                    optional_number(concurrent_percentile_99),
                    optional_number(ratio),
                    background_run.calls,
                    background_run.publishes,
                    background_run.batches,
                    optional_number(background_run.stopped_before_the_deadline_after_seconds),
                ));
            }
            (quiet_result, concurrent_result, background_result) => {
                emitter.line(&format!(
                    "name=q17_round round={round} valid=false quiet={} concurrent={} background={}",
                    quiet_result.err().unwrap_or_else(|| "ok".to_string()),
                    concurrent_result.err().unwrap_or_else(|| "ok".to_string()),
                    background_result.err().unwrap_or_else(|| "ok".to_string()),
                ));
            }
        }
    }
    std::fs::remove_file(&foreground_path).ok();
    let removal = images.remove();
    let foreground_fsync_latency_judgement = judgement_of_the_rounds(&judgements);
    emitter.line(&format!(
        "name=verdict segment=foreground_fsync constant_or_anchor_mismatches=0 valid_rounds={} foreground_fsync_latency_judgement_per_round={} foreground_fsync_latency_judgement={foreground_fsync_latency_judgement} images_removed={}",
        judgements.len(),
        judgements.join(","),
        removal.is_ok()
    ));
    emitter.finish();
    EXIT_CODE_OK
}

// ───────────────────────────── 准入、命令行、入口 ─────────────────────────────

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；强制跑（`--force`）时交回摘要，`main` 把它作为头一行打到标准输出（同 E183 装置的写法）。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e185_gc_batch_wall_clock_split.rs");
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
        std::process::exit(EXIT_CODE_NOT_RUN)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .and_then(|first_line| first_line.strip_prefix("forced"))
        .map(|summary| summary.trim().replace(' ', "_"))
}

fn usage() {
    eprintln!("用法：");
    eprintln!("  e185_gc_batch_wall_clock_split wall-clock --image-directory <目录> [--point <0.25|0.75>:<1024|8192>]…");
    eprintln!("  e185_gc_batch_wall_clock_split stage-split [--point <0.25|0.75>:<1024|8192>]…");
    eprintln!("  e185_gc_batch_wall_clock_split gpu-submission --wall-clock-product <文件> --stage-split-product <文件> --cards <序号,…>");
    eprintln!("  e185_gc_batch_wall_clock_split foreground-fsync --image-directory <目录> --point <0.25|0.75>:<1024|8192> --batch-units <4|8|16> [--seconds <秒>]");
}

/// 一个子命令的选项：`--名字 值` 成对，`--point` 可以给几次。认不得的、缺值的交回 `None`。
fn options_of(words: &[&str]) -> Option<BTreeMap<String, Vec<String>>> {
    let mut options: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut remaining = words.iter();
    while let Some(word) = remaining.next() {
        let name = word.strip_prefix("--")?;
        let value = remaining.next()?;
        options
            .entry(name.to_string())
            .or_default()
            .push((*value).to_string());
    }
    Some(options)
}

/// `--point` 给了就照给的（每个都要认得），没给就是全部取样点。
fn points_of(options: &BTreeMap<String, Vec<String>>) -> Option<Vec<SamplingPoint>> {
    match options.get("point") {
        None => Some(every_sampling_point()),
        Some(values) => values
            .iter()
            .map(|value| SamplingPoint::of_argument(value))
            .collect(),
    }
}

fn single_option<'options>(
    options: &'options BTreeMap<String, Vec<String>>,
    name: &str,
) -> Option<&'options str> {
    match options.get(name).map(Vec::as_slice) {
        Some([value]) => Some(value.as_str()),
        Some(_) | None => None,
    }
}

fn run_the_subcommand(words: &[&str]) -> i32 {
    let Some((subcommand, rest)) = words.split_first() else {
        usage();
        return EXIT_CODE_USAGE;
    };
    let Some(options) = options_of(rest) else {
        usage();
        return EXIT_CODE_USAGE;
    };
    let known = |names: &[&str]| options.keys().all(|key| names.contains(&key.as_str()));
    match *subcommand {
        "wall-clock" if known(&["image-directory", "point"]) => {
            match (
                single_option(&options, "image-directory"),
                points_of(&options),
            ) {
                (Some(directory), Some(points)) => {
                    run_the_wall_clock_segment(Path::new(directory), &points)
                }
                (None, _) | (_, None) => {
                    usage();
                    EXIT_CODE_USAGE
                }
            }
        }
        "stage-split" if known(&["point"]) => match points_of(&options) {
            Some(points) => run_the_stage_split_segment(&points),
            None => {
                usage();
                EXIT_CODE_USAGE
            }
        },
        "gpu-submission" if known(&["wall-clock-product", "stage-split-product", "cards"]) => {
            let cards: Option<Vec<u32>> = single_option(&options, "cards").and_then(|text| {
                text.split(',')
                    .map(|index| index.trim().parse::<u32>().ok())
                    .collect()
            });
            match (
                single_option(&options, "wall-clock-product"),
                single_option(&options, "stage-split-product"),
                cards,
            ) {
                (Some(wall_clock_product), Some(stage_split_product), Some(card_indices)) => {
                    run_the_gpu_submission_segment(
                        Path::new(wall_clock_product),
                        Path::new(stage_split_product),
                        &card_indices,
                    )
                }
                (None, _, _) | (_, None, _) | (_, _, None) => {
                    usage();
                    EXIT_CODE_USAGE
                }
            }
        }
        "foreground-fsync" if known(&["image-directory", "point", "batch-units", "seconds"]) => {
            let point = single_option(&options, "point").and_then(SamplingPoint::of_argument);
            let batch_units = single_option(&options, "batch-units")
                .and_then(|text| text.parse::<u64>().ok())
                .filter(|units| BATCH_SIZES.contains(units));
            let seconds = match single_option(&options, "seconds") {
                Some(text) => text.parse::<u64>().ok().filter(|value| *value > 0),
                None => Some(FOREGROUND_SECONDS_DEFAULT),
            };
            match (
                single_option(&options, "image-directory"),
                point,
                batch_units,
                seconds,
            ) {
                (Some(directory), Some(point), Some(batch_units), Some(seconds)) => {
                    run_the_foreground_fsync_segment(
                        Path::new(directory),
                        point,
                        batch_units,
                        seconds,
                    )
                }
                (None, _, _, _) | (_, None, _, _) | (_, _, None, _) | (_, _, _, None) => {
                    usage();
                    EXIT_CODE_USAGE
                }
            }
        }
        _ => {
            usage();
            EXIT_CODE_USAGE
        }
    }
}

#[cfg(feature = "gpu")]
fn run_the_gpu_submission_segment(
    wall_clock_product: &Path,
    stage_split_product: &Path,
    card_indices: &[u32],
) -> i32 {
    gpu_submission::run(wall_clock_product, stage_split_product, card_indices)
}

/// 没编 `gpu` 特性：第三段本次未跑（退 78），不悄悄退回 CPU。
#[cfg(not(feature = "gpu"))]
fn run_the_gpu_submission_segment(
    _wall_clock_product: &Path,
    _stage_split_product: &Path,
    card_indices: &[u32],
) -> i32 {
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=verdict segment=gpu_submission status=not_run reason=gpu_feature_not_compiled cards={}",
        card_indices.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
    ));
    emitter.finish();
    eprintln!("本次未跑：没编 gpu 特性（带 --features gpu 编这个 bin，经 scripts/crash/gpu-quiesce-run.sh 起）");
    EXIT_CODE_NOT_RUN
}

fn main() {
    if let Some(summary) = preflight() {
        println!("E7RESULT name=preflight forced={summary}");
    }
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    std::process::exit(run_the_subcommand(&words));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_constants_equal_the_crates_constants() {
        assert_eq!(
            constants_disagreeing_with_the_crates(),
            Vec::<(String, String, String)>::new()
        );
    }

    #[test]
    fn local_publish_budget_equals_the_crates_function_at_zero_and_a_thousand() {
        let root_ring_slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count();
        for publishes_already_passed in [0_u64, 1, 3, 1000] {
            assert_eq!(
                local_publish_budget(root_ring_slots_per_region, publishes_already_passed),
                publish_budget_of_a_compaction_run(
                    root_ring_slots_per_region,
                    publishes_already_passed
                ),
                "已经过去 {publishes_already_passed} 次发布时两边的预算相同"
            );
        }
        assert_eq!(
            local_publish_budget(root_ring_slots_per_region, 1000),
            1,
            "已经过去 1000 次发布，预算落到下限 1"
        );
    }

    #[test]
    fn live_slot_fractions_keep_one_or_three_of_every_four_units() {
        assert_eq!(LiveSlotFraction::OneQuarter.live_slots_per_segment(), 16);
        assert_eq!(LiveSlotFraction::ThreeQuarters.live_slots_per_segment(), 48);
        assert_eq!(LiveSlotFraction::OneQuarter.data_units_per_segment(), 8);
        assert_eq!(LiveSlotFraction::ThreeQuarters.data_units_per_segment(), 24);
        for fraction in LiveSlotFraction::ALL {
            let kept = (0_u64..32)
                .filter(|position| !fraction.is_overwritten(*position))
                .count();
            assert_eq!(
                u64::try_from(kept).expect("几个"),
                fraction.data_units_per_segment(),
                "整段 32 个单元覆盖写之后留下 m 个"
            );
        }
    }

    #[test]
    fn region_of_one_segment_holds_one_mebibyte_on_each_device() {
        assert_eq!(region_bytes_per_device(), 1_048_576);
    }

    #[test]
    fn issue_scan_reads_262144_bytes_at_one_quarter_and_786432_at_three_quarters() {
        assert_eq!(
            issue_scan_bytes(LiveSlotFraction::OneQuarter.data_units_per_segment()),
            262_144
        );
        assert_eq!(
            issue_scan_bytes(LiveSlotFraction::ThreeQuarters.data_units_per_segment()),
            786_432
        );
    }

    #[test]
    fn batches_of_one_intent_are_the_ceiling_of_the_data_units_over_the_batch_units() {
        let expected = [
            ((8, 4), 2),
            ((8, 8), 1),
            ((8, 16), 1),
            ((24, 4), 6),
            ((24, 8), 3),
            ((24, 16), 2),
        ];
        for ((data_units, batch_units), batches) in expected {
            assert_eq!(
                batches_of_one_intent(data_units, batch_units),
                batches,
                "m = {data_units}、K = {batch_units}"
            );
        }
    }

    #[test]
    fn intent_scan_bytes_add_the_issue_scan_to_every_batch_rescan() {
        let expected = [
            ((8, 4), 655_360),
            ((8, 8), 524_288),
            ((8, 16), 524_288),
            ((24, 4), 3_538_944),
            ((24, 8), 2_359_296),
            ((24, 16), 1_835_008),
        ];
        for ((data_units, batch_units), bytes) in expected {
            assert_eq!(
                intent_scan_bytes(data_units, batch_units),
                bytes,
                "m = {data_units}、K = {batch_units}"
            );
        }
    }

    #[test]
    fn first_batch_copy_bytes_count_both_devices() {
        assert_eq!(first_batch_copy_bytes(24, 4), 262_144);
        assert_eq!(first_batch_copy_bytes(24, 8), 524_288);
        assert_eq!(first_batch_copy_bytes(24, 16), 1_048_576);
        assert_eq!(
            first_batch_copy_bytes(8, 16),
            524_288,
            "f = 0.25 时 K = 16 也只有 8 个单元"
        );
    }

    #[test]
    fn intent_copy_bytes_are_524288_and_1572864() {
        assert_eq!(intent_copy_bytes(8), 524_288);
        assert_eq!(intent_copy_bytes(24), 1_572_864);
    }

    fn record(slot: u64, span_slots: u16, is_released: bool) -> AllocationRecord {
        AllocationRecord {
            device: DeviceIdentity(0),
            slot: SlotNumber(slot),
            span_slots,
            generation: singlefs_core::address::CheckpointTxg(7),
            is_released,
        }
    }

    fn region_at(start_slot: u64) -> CompactionRegion {
        CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(start_slot))
            .expect("64 的倍数")
    }

    #[test]
    fn expected_scan_read_bytes_count_sixteen_kibibytes_per_slot() {
        let records = [
            record(50_304, 2, false),
            record(50_306, 2, false),
            record(50_308, 2, true),
            record(50_310, 1, false),
            record(50_368, 2, false),
        ];
        assert_eq!(
            expected_scan_read_bytes(&records, region_at(50_304)),
            2 * 32_768 + 16_384,
            "R 里两个未释放的数据单元与一个节点；已释放的、R 外的不算"
        );
    }

    #[test]
    fn crc_stage_covers_every_byte_of_every_unit() {
        let units = vec![vec![7_u8; 32_768], vec![9_u8; 32_768]];
        let (covered_bytes, folded) = crc_stage_over_the_units(&units);
        assert_eq!(covered_bytes, 65_536, "甲段字节等于扫描读到的字节");
        assert_eq!(
            folded,
            crc32_castagnoli(&units[0]) ^ crc32_castagnoli(&units[1]),
            "整单元算 CRC"
        );
    }

    #[test]
    fn gpu_input_and_buffer_bytes_match_the_preregistered_anchors() {
        assert_eq!(1024 * LOCAL_GPU_SLOT_BYTES, 33_554_432);
        assert_eq!(local_gpu_buffer_bytes(1), 32_804);
        assert_eq!(local_gpu_buffer_bytes(1024), 33_574_928);
        assert_eq!(local_gpu_buffer_bytes(4096), 134_299_664);
        assert_eq!(local_gpu_buffer_bytes(16_384), 537_198_608);
    }

    #[test]
    fn cpu_crc32c_of_the_check_string_is_e3069283() {
        assert_eq!(crc32_castagnoli(CHECK_STRING), 0xE306_9283);
        assert_eq!(CHECK_STRING_CRC32_CASTAGNOLI, 0xE306_9283);
    }

    #[test]
    fn pcie_peaks_of_generation_five_by_sixteen_and_by_eight() {
        let by_sixteen = pcie_link_peak_bytes_per_second(5, 16).expect("5 代");
        let by_eight = pcie_link_peak_bytes_per_second(5, 8).expect("5 代");
        assert!(
            (by_sixteen - 63_015_384_615.38).abs() < 1.0,
            "5 代 x16：{by_sixteen}"
        );
        assert!(
            (by_eight.round() - 31_507_692_308.0).abs() < 0.5,
            "5 代 x8：{by_eight}"
        );
        assert!(
            pcie_link_peak_bytes_per_second(7, 16).is_none(),
            "认不得的代数读不到"
        );
    }

    #[test]
    fn submission_faster_than_the_link_allows_breaks_the_physical_floor() {
        let peak = pcie_link_peak_bytes_per_second(5, 16).expect("5 代");
        let bytes = 128 << 20;
        assert!(
            !respects_the_physical_floor(bytes as f64 / (2.0 * peak), bytes, peak),
            "快过下限一倍"
        );
        assert!(
            respects_the_physical_floor(bytes as f64 / peak * 1.01, bytes, peak),
            "比下限慢一点"
        );
    }

    #[test]
    fn least_squares_recovers_a_one_millisecond_intercept_and_a_one_nanosecond_slope() {
        let points: Vec<(f64, f64)> = [32_768_u64, 131_072, 1_048_576, 8_388_608]
            .iter()
            .map(|bytes| (*bytes as f64, 1e-3 + 1e-9 * *bytes as f64))
            .collect();
        let (intercept, slope) = least_squares_intercept_and_slope(&points).expect("四个点");
        assert!((intercept - 1e-3).abs() < 1e-5, "截距 {intercept}");
        assert!((slope - 1e-9).abs() < 1e-11, "斜率 {slope}");
        assert!(
            least_squares_intercept_and_slope(&points[..1]).is_none(),
            "一个点拟不出"
        );
    }

    #[test]
    fn fixed_overhead_share_of_one_mebibyte_is_about_0_488() {
        let share = fixed_overhead_share(1e-3, 1e-9, 1_048_576);
        assert!((share - 0.488_14).abs() < 1e-4, "φ = {share}");
    }

    #[test]
    fn batches_to_merge_round_nine_intercepts_over_the_batch_time_up() {
        let merge = batches_to_merge(1e-3, 1e-9, 1_048_576);
        assert!(
            (merge - 9.0).abs() < 1e-9,
            "⌈9 × 1 ms ÷ 1.048576 ms⌉ = 9，拿到 {merge}"
        );
        assert!(
            fixed_overhead_share(1e-3, 1e-9, 1_048_576 * 9) <= FIXED_SHARE_AT_MOST_FOR_ONE_BATCH,
            "合 9 批之后 φ ≤ 10%"
        );
    }

    #[test]
    fn judgements_land_on_the_registered_side_of_every_threshold() {
        assert_eq!(judgement_of_rho_at_the_registered_edges(0.4), "disk");
        assert_eq!(judgement_of_rho_at_the_registered_edges(1.0), "both");
        assert_eq!(judgement_of_rho_at_the_registered_edges(3.0), "cpu");
        assert_eq!(judgement_of_wall_ratio_at_the_registered_edges(3.5), "disk");
        assert_eq!(judgement_of_wall_ratio_at_the_registered_edges(2.0), "both");
        assert_eq!(judgement_of_wall_ratio_at_the_registered_edges(1.2), "cpu");
        assert_eq!(
            judgement_of_fixed_share_at_the_registered_edges(0.05),
            "one_batch_per_submission"
        );
        assert_eq!(
            judgement_of_fixed_share_at_the_registered_edges(0.3),
            "merging_optional"
        );
        assert_eq!(
            judgement_of_fixed_share_at_the_registered_edges(0.7),
            "merge_batches"
        );
        assert_eq!(
            judgement_of_bandwidth_ratio_at_the_registered_edge(0.05),
            "single_run_enough"
        );
        assert_eq!(
            judgement_of_bandwidth_ratio_at_the_registered_edge(0.2),
            "needs_the_concurrent_measurement"
        );
        assert_eq!(
            judgement_of_the_sixteen_thread_to_gpu_ratio(0.78),
            "gpu_loses"
        );
        assert_eq!(
            judgement_of_the_sixteen_thread_to_gpu_ratio(1.5),
            "gpu_has_a_chance"
        );
        assert_eq!(
            judgement_of_the_foreground_fsync_latency_ratio(2.5),
            "gc_drags_the_foreground"
        );
        assert_eq!(
            judgement_of_share(0.09, STAGE_SHARE_AT_LEAST_FOR_WORTH),
            "not_worth"
        );
        assert_eq!(
            judgement_of_share(0.1, STAGE_SHARE_AT_LEAST_FOR_WORTH),
            "worth"
        );
        assert_eq!(judgement_of_the_cpu_reconciliation(0.25), "paths_disagree");
        assert_eq!(
            judgement_of_the_batch_doubling(&[1.2, 1.1, 1.3, 1.25, 1.05]),
            "rises"
        );
        assert_eq!(
            judgement_of_the_batch_doubling(&[1.2, 0.9, 1.3, 1.25, 1.05]),
            "does_not_rise",
            "有一轮没涨"
        );
    }

    #[test]
    fn rho_wall_ratio_fixed_share_and_bandwidth_ratio_judgements_flip_when_the_threshold_moves_between_two_points(
    ) {
        let (_, lower_rho_judgement, higher_rho_judgement) =
            judgements_with_the_threshold_moved_between(0.8, 1.6, |value, threshold| {
                judgement_of_rho(value, threshold, f64::INFINITY)
            });
        assert_ne!(
            lower_rho_judgement, higher_rho_judgement,
            "Q1 的门槛挪到 0.8 与 1.6 之间"
        );
        assert_eq!(
            (
                judgement_of_rho_at_the_registered_edges(0.8),
                judgement_of_rho_at_the_registered_edges(1.6)
            ),
            ("both", "both"),
            "不挪门槛两点同判"
        );
        let (_, lower_wall_ratio_judgement, higher_wall_ratio_judgement) =
            judgements_with_the_threshold_moved_between(1.8, 2.6, |value, threshold| {
                judgement_of_wall_ratio(value, threshold, f64::NEG_INFINITY)
            });
        assert_ne!(
            lower_wall_ratio_judgement, higher_wall_ratio_judgement,
            "Q2"
        );
        let (_, lower_fixed_share_judgement, higher_fixed_share_judgement) =
            judgements_with_the_threshold_moved_between(0.2, 0.4, |value, threshold| {
                judgement_of_fixed_share(value, threshold, f64::INFINITY)
            });
        assert_ne!(
            lower_fixed_share_judgement, higher_fixed_share_judgement,
            "Q13"
        );
        let (_, lower_bandwidth_ratio_judgement, higher_bandwidth_ratio_judgement) =
            judgements_with_the_threshold_moved_between(0.02, 0.05, judgement_of_bandwidth_ratio);
        assert_ne!(
            lower_bandwidth_ratio_judgement, higher_bandwidth_ratio_judgement,
            "Q16"
        );
    }

    #[test]
    fn doubling_pairs_keep_only_pairs_within_the_movable_units() {
        assert_eq!(doubling_pairs_within(8), vec![(4, 8)]);
        assert_eq!(doubling_pairs_within(24), vec![(4, 8), (8, 16)]);
    }

    #[test]
    fn the_issue_call_is_not_counted_as_a_batch() {
        let calls = [kind_of_the_call(1, true, 0), kind_of_the_call(1, false, 8)];
        assert_eq!(calls, [CallKind::Issue, CallKind::FinalBatch]);
        let batches = calls
            .iter()
            .filter(|kind| counts_as_a_batch(**kind))
            .count();
        assert_eq!(batches, 1, "f = 0.25、K = 8：签发一次、一批（最后一批）");
        assert_eq!(kind_of_the_call(1, true, 4), CallKind::Batch);
        assert_eq!(kind_of_the_call(0, true, 0), CallKind::NoPublish);
        assert_eq!(
            kind_of_the_call(1, false, 0),
            CallKind::IntentDeletedWithoutABatch
        );
    }

    #[test]
    fn inclusion_check_flags_cpu_plus_wait_above_the_wall() {
        assert_eq!(
            inclusion_verdict(1_000_000, 800_000, 500_000),
            InclusionVerdict::CpuPlusWaitAboveTheWall,
            "挂钟 1 ms、CPU 0.8 ms、等盘 0.5 ms"
        );
    }

    #[test]
    fn inclusion_check_flags_unaccounted_time_above_ten_percent() {
        assert_eq!(
            inclusion_verdict(1_000_000, 500_000, 300_000),
            InclusionVerdict::UnaccountedAboveTenPercent
        );
        assert_eq!(
            inclusion_verdict(1_000_000, 600_000, 350_000),
            InclusionVerdict::Holds
        );
    }

    /// 假钟：每读一次，真时间走 100 ns；交回的读数按 `step_nanoseconds` 向下取整（`step_nanoseconds` = 1 ms 时照一个调度节拍才前进一次的钟）。
    fn clock_stepping_every(step_nanoseconds: u64) -> impl FnMut() -> Option<u64> {
        let mut true_nanoseconds = 0_u64;
        move || {
            true_nanoseconds += 100;
            Some(true_nanoseconds / step_nanoseconds * step_nanoseconds)
        }
    }

    #[test]
    fn the_clock_resolution_check_refuses_a_clock_that_advances_once_a_millisecond() {
        let reading =
            clock_advances_while_spinning(clock_stepping_every(1_000_000)).expect("假钟读得到");
        assert_eq!(
            reading,
            ClockResolutionReading {
                reads: 10_000,
                advances: 1,
                spanned_nanoseconds: 1_000_000
            },
            "真时间走到 1 ms 那一读才前进，1 ms 里只前进一次"
        );
        assert!(!is_the_clock_resolution_enough(reading), "{reading:?}");
    }

    #[test]
    fn the_clock_resolution_check_passes_a_clock_that_advances_on_every_read() {
        let reading = clock_advances_while_spinning(clock_stepping_every(1)).expect("假钟读得到");
        assert_eq!(
            reading,
            ClockResolutionReading {
                reads: 10_001,
                advances: 10_000,
                spanned_nanoseconds: 1_000_000
            }
        );
        assert!(is_the_clock_resolution_enough(reading), "{reading:?}");
    }

    /// 门槛两边各一个假钟：每 10000 ns 前进一次的钟在 1 ms 里前进 100 次（过），每 10102 ns 前进一次的前进 99 次（不过）。
    #[test]
    fn the_clock_resolution_check_draws_the_line_at_one_hundred_advances() {
        let at_the_line =
            clock_advances_while_spinning(clock_stepping_every(10_000)).expect("假钟读得到");
        assert_eq!(at_the_line.advances, 100, "{at_the_line:?}");
        assert!(
            is_the_clock_resolution_enough(at_the_line),
            "{at_the_line:?}"
        );
        let below_the_line =
            clock_advances_while_spinning(clock_stepping_every(10_102)).expect("假钟读得到");
        assert_eq!(below_the_line.advances, 99, "{below_the_line:?}");
        assert!(
            !is_the_clock_resolution_enough(below_the_line),
            "{below_the_line:?}"
        );
    }

    #[test]
    fn the_clock_resolution_check_refuses_a_stuck_clock_and_an_unreadable_one() {
        let stuck = clock_advances_while_spinning(|| Some(42)).expect("假钟读得到");
        assert_eq!(
            stuck,
            ClockResolutionReading {
                reads: CLOCK_RESOLUTION_CHECK_READS_AT_MOST,
                advances: 0,
                spanned_nanoseconds: 0
            },
            "停着不走的钟读满上限就停"
        );
        assert!(!is_the_clock_resolution_enough(stuck));
        assert_eq!(clock_advances_while_spinning(|| None), None, "读不到不当 0");
    }

    #[test]
    fn the_clock_resolution_check_passes_the_thread_cpu_clock() {
        let reading =
            clock_advances_while_spinning(thread_cpu_nanoseconds).expect("线程 CPU 时钟读得到");
        assert!(is_the_clock_resolution_enough(reading), "{reading:?}");
        let mut emitter = Emitter::new();
        assert!(
            check_the_thread_cpu_clock_resolution(&mut emitter),
            "开跑自检在本线程上过"
        );
    }

    #[test]
    fn device_wait_subtracts_the_cpu_spent_inside_the_call() {
        assert_eq!(device_wait_nanoseconds(5_000_000, 2_000_000), 3_000_000);
        assert_eq!(
            device_wait_nanoseconds(10_000, 1_000_000),
            -990_000,
            "纯 CPU 的设备调用里 CPU 读数包着挂钟读数，可以是负的"
        );
        let mut ledger = CallLedger::new(WriteInjection::Nothing);
        ledger.note_the_call(
            DeviceMethod::Write,
            4096,
            Duration::from_millis(5),
            Some(10_000_000),
            Some(12_000_000),
        );
        let totals = ledger.totals_of(DeviceMethod::Write);
        assert_eq!(
            (
                totals.calls,
                totals.bytes,
                totals.cpu_nanoseconds,
                totals.wait_nanoseconds
            ),
            (1, 4096, 2_000_000, 3_000_000)
        );
        ledger.note_the_call(
            DeviceMethod::Read,
            512,
            Duration::from_millis(1),
            None,
            Some(1),
        );
        assert!(
            ledger.is_the_cpu_unreadable_in_this_window,
            "读不到线程 CPU 时钟记下来，那一轮作废"
        );
    }

    #[test]
    fn gpu_results_that_differ_in_one_range_are_refused() {
        let cpu = vec![1_u32, 2, 3, 4];
        assert!(gpu_crcs_agree_with_the_cpu(&cpu, &cpu));
        let mut gpu = cpu.clone();
        gpu[2] ^= 1;
        assert!(
            !gpu_crcs_agree_with_the_cpu(&gpu, &cpu),
            "换掉一段回读值，对拍要红"
        );
        assert!(!gpu_crcs_agree_with_the_cpu(&cpu[..3], &cpu), "少一段也红");
    }

    #[test]
    fn saturation_workers_run_on_distinct_threads() {
        let stage = || {
            std::hint::black_box(crc32_castagnoli(&[5_u8; 4096]));
        };
        let reading = saturated_throughput(4, Duration::from_millis(30), 4096, &stage);
        assert_eq!(reading.distinct_threads, 4, "起了 4 条不同的线程");
        assert!(reading.repetitions >= 4 && reading.bytes_per_second > 0.0);
    }

    #[test]
    fn relocation_copy_bytes_count_only_the_overlap_with_the_destinations() {
        let writes = [
            RecordedWrite {
                device: DeviceIdentity(0),
                offset: 1 << 30,
                length: 32_768,
            },
            RecordedWrite {
                device: DeviceIdentity(1),
                offset: 1 << 30,
                length: 32_768,
            },
            RecordedWrite {
                device: DeviceIdentity(0),
                offset: 2 << 30,
                length: 16_384,
            },
        ];
        let destinations = [
            (DeviceIdentity(0), 1 << 30, 32_768),
            (DeviceIdentity(1), 1 << 30, 32_768),
        ];
        assert_eq!(
            relocation_copy_bytes(&writes, &destinations),
            (65_536, true)
        );
        assert_eq!(
            relocation_copy_bytes(&writes[..1], &destinations),
            (32_768, false),
            "只写了一块盘"
        );
    }

    #[test]
    fn rounds_with_differing_judgements_are_unstable() {
        assert_eq!(judgement_of_the_rounds(&["disk"; 5]), "disk");
        assert_eq!(
            judgement_of_the_rounds(&["disk", "disk", "both", "disk", "disk"]),
            "unstable"
        );
        assert_eq!(judgement_of_the_rounds(&[]), "not_run");
    }

    #[test]
    fn median_and_percentile_of_small_lists() {
        assert_eq!(median_of(&[3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median_of(&[4.0, 1.0, 2.0, 3.0]), Some(2.5));
        assert_eq!(median_of(&[]), None);
        let hundred: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(percentile_of(&hundred, 99), Some(99.0));
        assert_eq!(percentile_of(&[7.0], 99), Some(7.0));
    }

    #[test]
    fn positive_control_judges_hold_inside_their_bands_only() {
        assert!(
            disk_arm_control_holds(24_000_000.0, 10.0, 0.05),
            "10 次写 × 2 ms，等盘多 24 ms（1.2 倍）"
        );
        assert!(
            !disk_arm_control_holds(16_000_000.0, 10.0, 0.0),
            "只多 0.8 倍"
        );
        assert!(
            !disk_arm_control_holds(24_000_000.0, 10.0, 0.2),
            "CPU 变了 20%"
        );
        assert!(
            cpu_arm_cpu_clause_holds(20_500_000.0, 20_000_000.0),
            "10 次写 × 2 ms，CPU 多 20.5 ms"
        );
        assert!(!cpu_arm_cpu_clause_holds(25_000_000.0, 20_000_000.0));
        assert!(
            cpu_arm_write_wait_clause_holds(1_900_000.0, 20_000_000.0),
            "写调用等盘多 1.9 ms，是注入量的 9.5%"
        );
        assert!(
            !cpu_arm_write_wait_clause_holds(-2_100_000.0, 20_000_000.0),
            "少 2.1 ms 也超出：判的是绝对值"
        );
        assert!(!cpu_arm_write_wait_clause_holds(2_100_000.0, 20_000_000.0));
        assert!(cpu_arm_original_wait_clause_holds(-0.05));
        assert!(!cpu_arm_original_wait_clause_holds(0.2));
        assert!(stage_control_holds(2_100_000.0));
        assert!(!stage_control_holds(2_500_000.0));
        assert!(double_crc_control_holds(2.0, 1.0));
        assert!(!double_crc_control_holds(1.5, 1.0));
        assert!(page_cache_control_holds(3.0e9, 2.0e9));
        assert!(!page_cache_control_holds(2.1e9, 2.0e9));
    }

    #[test]
    fn worker_threads_take_the_own_variable_then_the_cap_then_the_registration() {
        let registration = "# 注释\n31  # 理由\n";
        assert_eq!(worker_threads_given(Some("8"), Some("4"), registration), 8);
        assert_eq!(worker_threads_given(None, Some("4"), registration), 4);
        assert_eq!(worker_threads_given(None, None, registration), 31);
    }

    #[test]
    #[should_panic(expected = "E185_THREADS 要是正整数")]
    fn worker_threads_refuse_zero_in_the_own_variable() {
        let _threads = worker_threads_given(Some("0"), None, "31\n");
    }

    #[test]
    fn thread_cpu_time_advances_while_the_thread_spins() {
        let before = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
        busy_wait_for(Duration::from_millis(30));
        let after = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
        assert!(
            after >= before + 15_000_000,
            "忙等 30 ms，线程 CPU 时钟至少前进 15 ms：{before} → {after}"
        );
    }

    /// 睡着的线程不在 CPU 上：线程 CPU 时钟几乎不动（第一行盘臂阳性对照「写里多睡 2 ms 不算 CPU」靠的就是这一点）。
    #[test]
    fn thread_cpu_time_never_exceeds_the_raw_monotonic_wall_clock_wrapped_around_it() {
        // 登记第十二节 12.4：挂钟读数包着 CPU 读数，忙等 200 ms，CPU 增量不许大过挂钟增量（`Instant` 慢约 7 ppm 时这里会反过来）。
        let started = RawWallClockReading::now();
        let cpu_before = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
        let busy_until = RawWallClockReading::now().0 + 200_000_000;
        while RawWallClockReading::now().0 < busy_until {
            std::hint::spin_loop();
        }
        let cpu_after = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
        let wall_nanoseconds =
            u64::try_from(started.elapsed().as_nanos()).expect("200 ms 装得进 u64");
        assert!(
            cpu_after - cpu_before <= wall_nanoseconds,
            "CPU {} 纳秒大过包着它的挂钟 {} 纳秒",
            cpu_after - cpu_before,
            wall_nanoseconds
        );
    }

    #[test]
    fn thread_cpu_time_stands_still_while_the_thread_sleeps() {
        let before = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
        std::thread::sleep(Duration::from_millis(20));
        let after = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
        assert!(
            after < before + 2_000_000,
            "睡 20 ms，线程 CPU 时钟前进不到 2 ms：{before} → {after}"
        );
    }

    #[test]
    fn timed_device_records_reads_before_the_first_write_and_the_injected_sleep() {
        let ledger = Rc::new(RefCell::new(CallLedger::new(
            WriteInjection::SleepTwoMilliseconds,
        )));
        let mut devices = timed_devices(
            vec![(
                DeviceIdentity(0),
                SparseBlockDevice::new(8 << 20, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)),
            )],
            &ledger,
        );
        let device = &mut devices[0].1;
        let mut buffer = vec![0_u8; 32_768];
        device
            .read_at(DeviceOffsetInBytes(0), &mut buffer)
            .expect("读");
        device
            .write_at(
                DeviceOffsetInBytes(32_768),
                &[1_u8; 4096],
                WriteDurability::Plain,
            )
            .expect("写");
        device
            .read_at(DeviceOffsetInBytes(32_768), &mut buffer[..4096])
            .expect("读");
        let window = ledger.borrow();
        assert_eq!(
            window
                .reads
                .iter()
                .map(|read| read.is_before_the_first_write)
                .collect::<Vec<_>>(),
            vec![true, false],
            "第一次写之前那一读算扫描读"
        );
        let write_totals = window.totals_of(DeviceMethod::Write);
        assert_eq!((write_totals.calls, write_totals.bytes), (1, 4096));
        assert!(
            write_totals.wall_nanoseconds >= 2_000_000,
            "写里多睡的 2 ms 计进挂钟"
        );
        assert!(window
            .chunks_written_since_the_copy
            .contains(&(DeviceIdentity(0), 0)));
    }

    fn call_reading(
        kind: CallKind,
        scan_read_bytes: u64,
        expected: Option<u64>,
        copy_bytes: u64,
        batches_observed: u64,
    ) -> CallReading {
        CallReading {
            kind,
            outcome: "test".to_string(),
            wall_nanoseconds: 10_000_000,
            cpu_nanoseconds: 4_000_000,
            is_the_cpu_unreadable: false,
            device_totals: MethodTotals {
                calls: 3,
                bytes: 0,
                wall_nanoseconds: 5_500_000,
                cpu_nanoseconds: 0,
                wait_nanoseconds: 5_500_000,
            },
            per_method: BTreeMap::new(),
            scan_read_bytes_in_the_region: scan_read_bytes,
            scan_read_bytes_elsewhere: 0,
            // 搬迁副本与释放前读盘核都是每个搬走的数据单元两块盘各一份整单元，字节相同。
            release_check_read_bytes: copy_bytes,
            read_bytes_after_the_first_write: 0,
            write_bytes: copy_bytes + 65_536,
            relocation_copy_bytes: copy_bytes,
            is_every_destination_written_whole: true,
            relocated_units: copy_bytes / 65_536,
            expected_scan_read_bytes: expected,
            batches_observed,
            inclusion: inclusion_verdict(10_000_000, 4_000_000, 5_500_000),
        }
    }

    fn intent_of(calls: Vec<CallReading>) -> IntentReading {
        IntentReading {
            region: Some(region_at(50_304)),
            calls,
            mount_failure: None,
            replay_material: Vec::new(),
        }
    }

    /// f = 0.25（m = 8）、K = 8 照锚点走的一条意图：签发一次、最后一批搬 8 个单元。
    fn intent_following_the_anchors() -> Vec<CallReading> {
        vec![
            call_reading(CallKind::Issue, 262_144, Some(262_144), 0, 0),
            call_reading(CallKind::FinalBatch, 262_144, Some(262_144), 524_288, 1),
        ]
    }

    #[test]
    fn stop_findings_stay_empty_on_an_intent_following_the_anchors() {
        assert_eq!(
            stop_findings_of(&intent_of(intent_following_the_anchors()), Some(8), 8),
            Vec::<String>::new()
        );
    }

    #[test]
    fn stop_findings_flag_scan_reads_that_differ_from_the_records() {
        let mut calls = intent_following_the_anchors();
        calls[1].scan_read_bytes_in_the_region = 229_376;
        let findings = stop_findings_of(&intent_of(calls), Some(8), 8);
        assert!(
            findings
                .iter()
                .any(|finding| finding.starts_with("clause_2")),
            "{findings:?}"
        );
    }

    #[test]
    fn stop_findings_flag_batches_that_differ_from_the_session_or_the_ceiling() {
        let mut calls = intent_following_the_anchors();
        calls[1].batches_observed = 2;
        let findings = stop_findings_of(&intent_of(calls), Some(8), 8);
        assert!(
            findings
                .iter()
                .any(|finding| finding.starts_with("clause_3_batches_counted")),
            "{findings:?}"
        );
        let findings_at_the_ceiling_of_two_batches =
            stop_findings_of(&intent_of(intent_following_the_anchors()), Some(8), 4);
        assert!(
            findings_at_the_ceiling_of_two_batches
                .iter()
                .any(|finding| finding.contains("ceiling_of_m_over_k")),
            "K = 4 时该有两批：{findings_at_the_ceiling_of_two_batches:?}"
        );
    }

    #[test]
    fn stop_findings_flag_a_first_batch_that_writes_one_device_only() {
        let mut calls = intent_following_the_anchors();
        calls[1].relocation_copy_bytes = 262_144;
        calls[1].is_every_destination_written_whole = false;
        let findings = stop_findings_of(&intent_of(calls), Some(8), 8);
        assert!(
            findings
                .iter()
                .any(|finding| finding.starts_with("clause_5")),
            "{findings:?}"
        );
    }

    #[test]
    fn stop_findings_flag_a_reversed_inclusion() {
        let mut calls = intent_following_the_anchors();
        calls[0].inclusion = inclusion_verdict(1_000_000, 800_000, 500_000);
        let findings = stop_findings_of(&intent_of(calls), Some(8), 8);
        assert!(
            findings
                .iter()
                .any(|finding| finding.starts_with("clause_6")),
            "{findings:?}"
        );
    }

    /// 一批里释放前读盘核读的字节：每个搬走的数据单元两块盘各读一份整单元。
    const fn release_check_read_bytes_of_a_batch(relocated_data_units: u64) -> u64 {
        DEVICES_HOLDING_A_COPY * relocated_data_units * LOCAL_DATA_UNIT_BYTES
    }

    fn recorded_read(
        device: DeviceIdentity,
        slot: u64,
        length: u64,
        is_before_the_first_write: bool,
    ) -> RecordedRead {
        RecordedRead {
            device,
            offset: slot * LOCAL_SLOT_BYTES,
            length,
            is_before_the_first_write,
        }
    }

    /// 一批的读照发布的次序排：区域扫描读 R 那块盘上每个活单元一遍，释放前读盘核读每个搬迁源两块盘各一遍，然后第一次写。
    fn reads_of_a_batch(live_slots: &[u64], relocated_slots: &[u64]) -> Vec<RecordedRead> {
        let mut reads: Vec<RecordedRead> = live_slots
            .iter()
            .map(|slot| recorded_read(DeviceIdentity(0), *slot, LOCAL_DATA_UNIT_BYTES, true))
            .collect();
        for slot in relocated_slots {
            reads.push(recorded_read(
                DeviceIdentity(0),
                *slot,
                LOCAL_DATA_UNIT_BYTES,
                true,
            ));
            reads.push(recorded_read(
                DeviceIdentity(1),
                *slot,
                LOCAL_DATA_UNIT_BYTES,
                true,
            ));
        }
        reads
    }

    #[test]
    fn release_check_read_bytes_of_a_batch_count_both_devices() {
        assert_eq!(release_check_read_bytes_of_a_batch(4), 262_144);
        assert_eq!(release_check_read_bytes_of_a_batch(8), 524_288);
        assert_eq!(release_check_read_bytes_of_a_batch(16), 1_048_576);
    }

    /// 扫描三个活单元、搬走其中两个：R 那块盘上搬迁源的再读与另一块盘上的那两读归释放前读盘核；不是搬迁源的槽上再读照旧归扫描；
    /// R 外的读归别处；第一次写之后的读单列。
    #[test]
    fn reads_of_a_call_split_the_release_check_copies_from_the_scan_reads() {
        let mut reads = reads_of_a_batch(&[50_304, 50_306, 50_308], &[50_304, 50_306]);
        reads.push(recorded_read(
            DeviceIdentity(0),
            50_308,
            LOCAL_DATA_UNIT_BYTES,
            true,
        ));
        reads.push(recorded_read(
            DeviceIdentity(0),
            100,
            LOCAL_NODE_BYTES,
            true,
        ));
        reads.push(recorded_read(
            DeviceIdentity(1),
            100,
            LOCAL_NODE_BYTES,
            true,
        ));
        reads.push(recorded_read(
            DeviceIdentity(0),
            50_304,
            LOCAL_DATA_UNIT_BYTES,
            false,
        ));
        let relocated: BTreeSet<u64> = [50_304, 50_306].into_iter().collect();
        let split = split_the_reads_of_a_call(&reads, Some(region_at(50_304)), &relocated);
        let scan_bytes: u64 = split
            .scan_reads_in_the_region
            .iter()
            .map(|read| read.length)
            .sum();
        let release_check_bytes: u64 = split
            .release_check_reads
            .iter()
            .map(|read| read.length)
            .sum();
        assert_eq!(
            scan_bytes, 131_072,
            "三个活单元各一读，加上不是搬迁源的那一次再读"
        );
        assert_eq!(
            release_check_bytes,
            release_check_read_bytes_of_a_batch(2),
            "两个搬迁源两块盘各一份"
        );
        assert_eq!(
            split
                .release_check_reads
                .iter()
                .map(|read| read.device)
                .collect::<Vec<_>>(),
            vec![
                DeviceIdentity(0),
                DeviceIdentity(1),
                DeviceIdentity(0),
                DeviceIdentity(1)
            ]
        );
        assert_eq!(
            split.scan_read_bytes_elsewhere, 32_768,
            "R 外两块盘各一个节点"
        );
        assert_eq!(split.read_bytes_after_the_first_write, 32_768);
        let before_the_publish = split_the_reads_of_a_call(&reads, None, &BTreeSet::new());
        assert_eq!(
            before_the_publish.release_check_reads,
            Vec::<RecordedRead>::new(),
            "没有搬迁源就没有释放前读盘核"
        );
    }

    /// 反例：释放前读盘核的读并回扫描读（没有搬迁源可扣，等于这一栏没分出来），停机条款第 2 条必须红；分出来之后不红。
    /// f = 0.25（m = 8）、K = 8：最后一批扫 8 个单元、搬 8 个。
    #[test]
    fn merging_the_release_check_reads_back_into_the_scan_trips_stop_clause_two() {
        let live_slots: Vec<u64> = (0..8).map(|unit| 50_304 + 8 * unit).collect();
        let reads = reads_of_a_batch(&live_slots, &live_slots);
        let relocated: BTreeSet<u64> = live_slots.iter().copied().collect();
        let scan_bytes_of = |split: &ReadsOfOneCall| -> u64 {
            split
                .scan_reads_in_the_region
                .iter()
                .map(|read| read.length)
                .sum()
        };
        let separated = split_the_reads_of_a_call(&reads, Some(region_at(50_304)), &relocated);
        let merged = split_the_reads_of_a_call(&reads, Some(region_at(50_304)), &BTreeSet::new());
        assert_eq!(
            separated
                .release_check_reads
                .iter()
                .map(|read| read.length)
                .sum::<u64>(),
            release_check_read_bytes_of_a_batch(8)
        );
        let intent_with_the_scan_bytes = |scan_bytes: u64| {
            intent_of(vec![
                call_reading(CallKind::Issue, 262_144, Some(262_144), 0, 0),
                call_reading(CallKind::FinalBatch, scan_bytes, Some(262_144), 524_288, 1),
            ])
        };
        assert_eq!(
            stop_findings_of(
                &intent_with_the_scan_bytes(scan_bytes_of(&separated)),
                Some(8),
                8
            ),
            Vec::<String>::new()
        );
        let findings = stop_findings_of(
            &intent_with_the_scan_bytes(scan_bytes_of(&merged)),
            Some(8),
            8,
        );
        assert_eq!(
            findings,
            vec!["clause_2_scan_read_bytes_at_call_1:read=524288:records=262144".to_string()]
        );
    }

    #[test]
    fn crc_byte_anchors_hold_only_when_both_items_match_their_reads() {
        assert_eq!(
            crc_byte_anchor_finding(1, 262_144, 262_144, 524_288, 524_288),
            None
        );
        let release_check_short = crc_byte_anchor_finding(1, 262_144, 262_144, 262_144, 524_288);
        assert!(
            release_check_short.as_deref().is_some_and(
                |finding| finding.starts_with("clause_4_release_check_crc_bytes_262144")
            ),
            "{release_check_short:?}"
        );
        let release_check_long = crc_byte_anchor_finding(1, 262_144, 262_144, 786_432, 524_288);
        assert!(release_check_long.is_some(), "多算了也不对");
        let scan_short = crc_byte_anchor_finding(1, 229_376, 262_144, 524_288, 524_288);
        assert!(
            scan_short
                .as_deref()
                .is_some_and(|finding| finding.starts_with("clause_4_crc_bytes_229376")),
            "{scan_short:?}"
        );
    }

    #[test]
    fn chunk_digests_of_a_memory_device_and_its_image_file_agree_and_one_byte_changes_them() {
        let mut device = SparseBlockDevice::new(
            3 * IMAGE_CHUNK_BYTES,
            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
        );
        device
            .write_at(
                DeviceOffsetInBytes(IMAGE_CHUNK_BYTES + 512),
                &[3_u8; 1024],
                WriteDurability::Plain,
            )
            .expect("写");
        let path =
            std::env::temp_dir().join(format!("e185-digest-test-{}.img", std::process::id()));
        let _ = std::fs::remove_file(&path);
        create_a_zero_filled_image(&path, 3 * IMAGE_CHUNK_BYTES).expect("建全零镜像");
        write_the_memory_device_into(&path, &device).expect("写进镜像");
        let file_digest = digest_of_the_image_file(&path).expect("算摘要");
        assert_eq!(
            file_digest,
            digest_of_the_memory_device(&device),
            "内存盘与镜像文件按同一个定义算"
        );
        let file = OpenOptions::new().write(true).open(&path).expect("打开");
        file.write_all_at(&[1_u8], 2 * IMAGE_CHUNK_BYTES + 7)
            .expect("改一个字节");
        drop(file);
        assert_ne!(
            digest_of_the_image_file(&path).expect("算摘要"),
            file_digest,
            "改一个字节摘要就不同"
        );
        std::fs::remove_file(&path).expect("删掉自己建的镜像");
    }

    /// 端到端（内存盘、小池）：造池、核几何、量一条意图，第七节锚点与停机条款第 2、3、5、6 条不触发，第二段的重放（停机条款第 4 条）对得上。
    #[test]
    fn small_pool_relocates_one_intent_with_the_anchors_and_the_replay_holding() {
        let point = SamplingPoint {
            fraction: LiveSlotFraction::ThreeQuarters,
            pool_data_units: 128,
        };
        let prepared = prepare_the_sampling_point(point);
        assert!(
            prepared.is_the_geometry_formed,
            "128 个数据单元造得出至少一个 f 段"
        );
        assert!(
            prepared.pool.is_every_overwritten_slot_released,
            "被覆盖写的旧落点都已释放"
        );
        let batch_units = 8;
        let reading = measure_on_the_memory_device(
            &prepared.pool,
            measured_switches(batch_units, prepared.choice.forced_region()),
            WriteInjection::Nothing,
            ReplayCapture::On,
        );
        assert_eq!(reading.mount_failure, None);
        let findings = stop_findings_of(&reading, Some(24), batch_units);
        assert_eq!(findings, Vec::<String>::new());
        assert_eq!(
            reading.calls.first().map(|call| call.kind),
            Some(CallKind::Issue)
        );
        assert_eq!(
            reading
                .calls
                .iter()
                .filter(|call| counts_as_a_batch(call.kind))
                .count(),
            3,
            "m = 24、K = 8：三批"
        );
        assert_eq!(
            reading
                .calls
                .iter()
                .map(|call| call.scan_read_bytes_in_the_region)
                .sum::<u64>(),
            intent_scan_bytes(24, batch_units),
            "签发加各批扫描读到 R 的字节之和等于锚点"
        );
        for call in &reading.calls {
            assert_eq!(
                call.release_check_read_bytes,
                release_check_read_bytes_of_a_batch(call.relocated_units),
                "释放前读盘核：这一批每个搬迁源两块盘各读一份整单元（{:?}）",
                call.kind
            );
        }
        assert_eq!(
            reading
                .calls
                .iter()
                .map(|call| call.release_check_read_bytes)
                .sum::<u64>(),
            release_check_read_bytes_of_a_batch(24),
            "一条意图搬走 24 个数据单元，释放前读盘核读 2 × 24 × 32768"
        );
        for material in &reading.replay_material {
            let replay = replay_the_stages_of_a_call(
                material,
                reading.region,
                prepared.choice,
                batch_units,
                &prepared.pool.devices,
            );
            assert_eq!(
                replay.stop_finding, None,
                "第 {} 次调用的重放",
                material.call_index
            );
            assert_eq!(
                replay.crc_bytes,
                material
                    .scan_reads_in_the_region
                    .iter()
                    .map(|read| read.length)
                    .sum::<u64>(),
                "甲段字节等于扫描读字节"
            );
            assert_eq!(
                replay.release_check_crc_bytes,
                reading.calls[material.call_index].release_check_read_bytes,
                "甲段释放前读盘核那一项的字节等于这一批 release_check_read_bytes（第 {} 次调用）",
                material.call_index
            );
        }
    }

    #[test]
    fn result_lines_split_into_keys_and_values() {
        let lines = result_lines_of("noise\nE7RESULT name=call arm=memory scan_read_bytes_in_the_region=262144\nE7RESULT name=done emitted=2\n");
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0]
                .get("scan_read_bytes_in_the_region")
                .map(String::as_str),
            Some("262144")
        );
        assert_eq!(lines[1].get("name").map(String::as_str), Some("done"));
    }

    #[test]
    fn sampling_points_parse_only_the_registered_values() {
        assert_eq!(
            SamplingPoint::of_argument("0.75:1024"),
            Some(SamplingPoint {
                fraction: LiveSlotFraction::ThreeQuarters,
                pool_data_units: 1024
            })
        );
        assert_eq!(SamplingPoint::of_argument("0.5:1024"), None);
        assert_eq!(SamplingPoint::of_argument("0.25:4096"), None);
        assert_eq!(every_sampling_point().len(), 4);
    }

    #[test]
    fn wall_clock_configuration_line_prints_no_part_of_the_image_directory() {
        let private_name = format!(
            "e185-configuration-line-private-directory-{}",
            std::process::id()
        );
        let directory = std::env::temp_dir().join(&private_name);
        let line = wall_clock_configuration_line(
            &directory,
            &every_sampling_point(),
            &preparation_settings_with(31, 28 << 30),
        );
        let device_number = std::fs::metadata(&directory)
            .expect("配置行那一步建了这个目录")
            .dev();
        std::fs::remove_dir(&directory).expect("删掉这条单测建的空目录");
        let directory_text = directory.display().to_string();
        assert!(
            !line.contains(&directory_text),
            "配置行不打传入的目录 {directory_text}：{line}"
        );
        assert!(
            !line.contains(&private_name),
            "配置行连目录名的末段也不打：{line}"
        );
        assert!(
            !line.contains("image_directory="),
            "配置行没有 image_directory 这一栏：{line}"
        );
        assert!(
            line.contains(&format!(" image_device={device_number} ")),
            "配置行照打目录所在的设备号 {device_number}：{line}"
        );
        assert!(
            line.contains(" image_directory_is_cleared_at_boot="),
            "配置行打目录在不在开机清空的地方：{line}"
        );
        assert!(
            line.ends_with(" preparation_threads=31 preparation_memory_cap_bytes=30064771072 preparation_memory_cap_source=environment_variable preparation_units=24 preparation_units_started_at_once_by_the_estimates=15 preparation_plan_by_the_estimates=in_batches"),
            "配置行打准备阶段的线程数、内存上限与它从哪取、一共几份准备、按估算头一次派几份与分不分批：{line}"
        );
    }

    #[test]
    fn device_numbers_split_into_the_major_and_minor_that_mountinfo_prints() {
        // 三个设备号由 makedev 的编码另算（python 的 os.major / os.minor 核过）：259:2、0:300、4096:0、0:65536
        assert_eq!(major_and_minor_of_the_device_number(66_306), (259, 2));
        assert_eq!(major_and_minor_of_the_device_number(1_048_620), (0, 300));
        assert_eq!(
            major_and_minor_of_the_device_number(17_592_186_044_416),
            (4096, 0)
        );
        assert_eq!(
            major_and_minor_of_the_device_number(268_435_456),
            (0, 65_536)
        );
    }

    #[test]
    fn filesystem_type_comes_from_the_mountinfo_line_of_that_device() {
        let mount_information =
            "26 32 0:23 / /sys rw,nosuid,nodev,noexec,relatime shared:7 - sysfs sysfs rw\n\
                                 32 1 259:2 / / rw,relatime shared:1 - ext4 /dev/sample-disk rw\n\
                                 40 32 0:300 / /tmp rw,nosuid,nodev - tmpfs tmpfs rw,size=1024k\n";
        assert_eq!(
            filesystem_type_of_the_device(66_306, mount_information).as_deref(),
            Some("ext4")
        );
        assert_eq!(
            filesystem_type_of_the_device(1_048_620, mount_information).as_deref(),
            Some("tmpfs")
        );
        assert_eq!(
            filesystem_type_of_the_device(66_307, mount_information),
            None
        );
    }

    #[test]
    fn image_directories_under_the_temporary_directory_or_on_a_memory_file_system_count_as_cleared_at_boot(
    ) {
        assert_eq!(
            is_cleared_at_boot(Some("ext4"), Some(Path::new("/tmp/e185-images"))),
            Some(true)
        );
        assert_eq!(
            is_cleared_at_boot(Some("tmpfs"), Some(Path::new("/srv/e185-images"))),
            Some(true)
        );
        assert_eq!(
            is_cleared_at_boot(Some("ext4"), Some(Path::new("/srv/e185-images"))),
            Some(false)
        );
        assert_eq!(
            is_cleared_at_boot(Some("ext4"), Some(Path::new("/tmpfiles/e185-images"))),
            Some(false),
            "按路径分量比，/tmpfiles 不在 /tmp 下"
        );
        assert_eq!(
            is_cleared_at_boot(None, Some(Path::new("/srv/e185-images"))),
            None
        );
        assert_eq!(
            is_cleared_at_boot(None, Some(Path::new("/tmp"))),
            Some(true)
        );
    }

    /// 基线先跑完、对照后跑的那一趟产物（`e185-gc-batch-wall-clock-split-wall-clock.out`）对照点 f = 0.75、P = 1024、K = 8 每轮整条意图的读数，由那一轮四次调用的
    /// `name=call` 行按栏加出来：（轮号、调用 CPU、等盘、写调用次数、写调用等盘、写调用挂钟、设备调用挂钟）。
    /// 最后两栏是 V1（等盘不减设备调用里的 CPU）会读到的写调用等盘与整条意图等盘。
    type SeparatedScheduleRound = (usize, f64, f64, f64, f64, f64, f64);

    #[rustfmt::skip]
    const SEPARATED_SCHEDULE_MEMORY_BASELINE: [SeparatedScheduleRound; 5] = [
        (1, 106_650_857.0, -120_419.0, 176.0, -19_440.0, 799_443.0, 1_271_745.0),
        (2, 106_204_453.0, -118_649.0, 176.0, -19_041.0, 816_543.0, 1_275_655.0),
        (3, 106_781_276.0, -119_583.0, 176.0, -19_380.0, 799_744.0, 1_273_445.0),
        (4, 106_644_964.0, -122_455.0, 176.0, -19_270.0, 804_394.0, 1_259_889.0),
        (5, 106_632_087.0, -119_043.0, 176.0, -19_181.0, 818_224.0, 1_266_807.0),
    ];
    #[rustfmt::skip]
    const SEPARATED_SCHEDULE_MEMORY_SLEEP: [SeparatedScheduleRound; 5] = [
        (1, 107_321_255.0, 361_903_916.0, 176.0, 362_011_585.0, 363_937_453.0, 364_415_787.0),
        (2, 108_514_827.0, 361_867_449.0, 176.0, 361_975_619.0, 363_881_382.0, 364_353_044.0),
        (3, 108_618_464.0, 361_879_634.0, 176.0, 361_987_757.0, 363_938_593.0, 364_422_204.0),
        (4, 107_897_310.0, 361_584_541.0, 176.0, 361_691_513.0, 363_453_732.0, 363_925_104.0),
        (5, 108_574_516.0, 361_889_292.0, 176.0, 361_998_301.0, 363_928_584.0, 364_409_338.0),
    ];
    #[rustfmt::skip]
    const SEPARATED_SCHEDULE_MEMORY_SPIN: [SeparatedScheduleRound; 5] = [
        (1, 458_878_127.0, -81_616.0, 176.0, 18_418.0, 352_899_546.0, 353_359_687.0),
        (2, 458_955_739.0, -79_847.0, 176.0, 20_514.0, 352_885_629.0, 353_348_721.0),
        (3, 459_127_621.0, 25_568.0, 176.0, 127_010.0, 352_998_957.0, 353_461_187.0),
        (4, 460_322_754.0, 31_784.0, 176.0, 132_072.0, 353_031_469.0, 353_504_572.0),
        (5, 459_040_019.0, -56_116.0, 176.0, 44_294.0, 352_915_809.0, 353_370_413.0),
    ];
    #[rustfmt::skip]
    const SEPARATED_SCHEDULE_REAL_DISK_BASELINE: [SeparatedScheduleRound; 5] = [
        (1, 109_092_461.0, 94_772_746.0, 176.0, 22_022_149.0, 23_272_218.0, 99_037_220.0),
        (2, 116_147_957.0, 241_725_690.0, 176.0, 66_655_889.0, 68_048_987.0, 246_513_279.0),
        (3, 116_614_314.0, 341_369_727.0, 176.0, 104_965_004.0, 106_504_352.0, 346_530_879.0),
        (4, 116_404_811.0, 50_426_296.0, 176.0, 5_518_406.0, 6_876_064.0, 54_918_191.0),
        (5, 119_740_390.0, 335_425_318.0, 176.0, 87_655_725.0, 89_148_566.0, 340_540_194.0),
    ];
    #[rustfmt::skip]
    const SEPARATED_SCHEDULE_REAL_DISK_SLEEP: [SeparatedScheduleRound; 5] = [
        (1, 119_998_852.0, 618_946_708.0, 176.0, 400_368_017.0, 403_069_041.0, 625_195_352.0),
        (2, 119_374_356.0, 648_815_686.0, 176.0, 431_612_652.0, 434_321_325.0, 654_967_575.0),
        (3, 121_477_226.0, 645_589_647.0, 176.0, 426_420_624.0, 429_068_835.0, 651_841_556.0),
        (4, 119_360_821.0, 615_122_931.0, 176.0, 428_582_878.0, 431_193_597.0, 621_116_652.0),
        (5, 120_276_080.0, 681_350_924.0, 176.0, 425_225_299.0, 427_879_238.0, 687_481_321.0),
    ];
    #[rustfmt::skip]
    const SEPARATED_SCHEDULE_REAL_DISK_SPIN: [SeparatedScheduleRound; 5] = [
        (1, 470_584_575.0, 282_688_855.0, 176.0, 69_994_372.0, 424_142_393.0, 640_223_561.0),
        (2, 471_986_271.0, 335_232_227.0, 176.0, 122_034_216.0, 476_227_747.0, 692_923_103.0),
        (3, 469_560_156.0, 54_841_237.0, 176.0, 6_815_812.0, 360_956_439.0, 412_162_281.0),
        (4, 471_007_281.0, 305_723_259.0, 176.0, 68_484_488.0, 422_628_281.0, 663_299_405.0),
        (5, 471_091_665.0, 311_975_618.0, 176.0, 61_572_246.0, 415_830_508.0, 669_657_825.0),
    ];

    /// 等盘怎么读：装置今天的读法，或 V1 的读法（等盘 = 设备调用挂钟）。
    #[derive(Clone, Copy)]
    enum WaitReadingOfTheFixture {
        AsBuilt,
        DeviceWallWithoutTheCpuSubtracted,
    }

    fn separated_schedule_rounds(
        rows: &[SeparatedScheduleRound],
        wait_reading: WaitReadingOfTheFixture,
    ) -> Vec<RowOneControlRound> {
        rows.iter()
            .map(
                |&(round, cpu, wait, write_calls, write_wait, write_wall, device_wall)| {
                    let (wait_nanoseconds, write_wait_nanoseconds) = match wait_reading {
                        WaitReadingOfTheFixture::AsBuilt => (wait, write_wait),
                        WaitReadingOfTheFixture::DeviceWallWithoutTheCpuSubtracted => {
                            (device_wall, write_wall)
                        }
                    };
                    RowOneControlRound {
                        round,
                        cpu_nanoseconds: cpu,
                        wait_nanoseconds,
                        write_calls,
                        write_wait_nanoseconds,
                    }
                },
            )
            .collect()
    }

    fn separated_schedule_reading(
        injection: WriteInjection,
        baseline: &[SeparatedScheduleRound],
        control: &[SeparatedScheduleRound],
        wait_reading: WaitReadingOfTheFixture,
    ) -> RowOneControlReading {
        row_one_control_reading(
            injection,
            &separated_schedule_rounds(baseline, wait_reading),
            &separated_schedule_rounds(control, wait_reading),
        )
        .expect("基线与对照各 5 轮")
    }

    /// 登记第十二节 12.9：基线先跑完、对照后跑那一趟的 CPU 臂两格（产物 `name=positive_control row=1 arm_of_the_question=cpu` 那两行）照原判据不过，
    /// 照新判法（写调用的等盘增量 ÷ 注入量）过；CPU 那一句两格都过。
    #[test]
    fn separated_schedule_cpu_arm_cells_hold_by_the_write_wait_over_the_injected_and_fail_by_the_original_change(
    ) {
        let as_built = WaitReadingOfTheFixture::AsBuilt;
        let memory = separated_schedule_reading(
            WriteInjection::SpinTwoMillisecondsOfThreadCpu,
            &SEPARATED_SCHEDULE_MEMORY_BASELINE,
            &SEPARATED_SCHEDULE_MEMORY_SPIN,
            as_built,
        );
        let real_disk = separated_schedule_reading(
            WriteInjection::SpinTwoMillisecondsOfThreadCpu,
            &SEPARATED_SCHEDULE_REAL_DISK_BASELINE,
            &SEPARATED_SCHEDULE_REAL_DISK_SPIN,
            as_built,
        );
        assert_eq!(
            (memory.write_calls, memory.injected_nanoseconds),
            (176.0, 176.0 * 2_000_000.0)
        );
        assert_eq!(memory.cpu_increment_nanoseconds, 352_395_055.0);
        assert_eq!(memory.wait_increment_nanoseconds, 63_467.0);
        assert_eq!(memory.write_wait_increment_nanoseconds, 63_564.0);
        assert_eq!(format!("{:.6}", memory.wait_change), "-0.530736");
        assert_eq!(real_disk.cpu_increment_nanoseconds, 354_602_470.0);
        assert_eq!(real_disk.wait_increment_nanoseconds, 63_997_569.0);
        assert_eq!(real_disk.write_wait_increment_nanoseconds, 1_828_599.0);
        assert_eq!(format!("{:.6}", real_disk.wait_change), "0.264753");
        for (device, reading) in [("memory", memory), ("real_disk", real_disk)] {
            assert_eq!(
                reading.judgement,
                RowOneControlJudgement::CpuArm {
                    cpu_clause_holds: true,
                    write_wait_clause_holds: true,
                    wait_clause_under_the_original_criterion_holds: false,
                },
                "{device}：新判法过、原判据那一句照算不过"
            );
            assert!(reading.judgement.holds(), "{device}：这一格按新判法算过");
        }
    }

    /// V1（等盘 = 设备调用挂钟，不减设备调用里的 CPU）在那一趟两格上读到的写调用等盘增量约等于注入量，新判法两格都判红。
    #[test]
    fn device_wait_without_the_cpu_subtracted_fails_the_write_wait_clause_on_both_separated_schedule_cells(
    ) {
        let without_the_cpu = WaitReadingOfTheFixture::DeviceWallWithoutTheCpuSubtracted;
        let memory = separated_schedule_reading(
            WriteInjection::SpinTwoMillisecondsOfThreadCpu,
            &SEPARATED_SCHEDULE_MEMORY_BASELINE,
            &SEPARATED_SCHEDULE_MEMORY_SPIN,
            without_the_cpu,
        );
        let real_disk = separated_schedule_reading(
            WriteInjection::SpinTwoMillisecondsOfThreadCpu,
            &SEPARATED_SCHEDULE_REAL_DISK_BASELINE,
            &SEPARATED_SCHEDULE_REAL_DISK_SPIN,
            without_the_cpu,
        );
        assert_eq!(memory.write_wait_increment_nanoseconds, 352_111_415.0);
        assert_eq!(real_disk.write_wait_increment_nanoseconds, 354_579_294.0);
        for (device, reading) in [("memory", memory), ("real_disk", real_disk)] {
            assert_eq!(
                reading.judgement,
                RowOneControlJudgement::CpuArm {
                    cpu_clause_holds: true,
                    write_wait_clause_holds: false,
                    wait_clause_under_the_original_criterion_holds: false,
                },
                "{device}：漏减 CPU 时写调用等盘多出一整份注入量"
            );
            assert!(!reading.judgement.holds(), "{device}：V1 下这一格判红");
        }
    }

    /// 盘臂那一行没改：那一趟两格（产物 `arm_of_the_question=disk` 那两行）照旧判过，增量与产物相同。
    #[test]
    fn separated_schedule_disk_arm_cells_keep_their_judgement() {
        let memory = separated_schedule_reading(
            WriteInjection::SleepTwoMilliseconds,
            &SEPARATED_SCHEDULE_MEMORY_BASELINE,
            &SEPARATED_SCHEDULE_MEMORY_SLEEP,
            WaitReadingOfTheFixture::AsBuilt,
        );
        let real_disk = separated_schedule_reading(
            WriteInjection::SleepTwoMilliseconds,
            &SEPARATED_SCHEDULE_REAL_DISK_BASELINE,
            &SEPARATED_SCHEDULE_REAL_DISK_SLEEP,
            WaitReadingOfTheFixture::AsBuilt,
        );
        assert_eq!(
            (
                memory.cpu_increment_nanoseconds,
                memory.wait_increment_nanoseconds
            ),
            (1_869_863.0, 361_999_217.0)
        );
        assert_eq!(
            (
                real_disk.cpu_increment_nanoseconds,
                real_disk.wait_increment_nanoseconds
            ),
            (3_594_041.0, 403_863_957.0)
        );
        assert_eq!(
            memory.judgement,
            RowOneControlJudgement::DiskArm { holds: true }
        );
        assert_eq!(
            real_disk.judgement,
            RowOneControlJudgement::DiskArm { holds: true }
        );
    }

    /// 按轮号配对的增量中位只报不判：取同一轮号的「对照 − 基线」，缺了一边的轮号不进。
    #[test]
    fn paired_round_increments_take_the_same_round_number_from_both_sides() {
        let reading = separated_schedule_reading(
            WriteInjection::SpinTwoMillisecondsOfThreadCpu,
            &SEPARATED_SCHEDULE_REAL_DISK_BASELINE,
            &SEPARATED_SCHEDULE_REAL_DISK_SPIN,
            WaitReadingOfTheFixture::AsBuilt,
        );
        assert_eq!(reading.paired_rounds, 5);
        assert_eq!(
            reading.cpu_increment_paired_median_nanoseconds,
            Some(354_602_470.0)
        );
        assert_eq!(
            reading.write_wait_increment_paired_median_nanoseconds,
            Some(47_972_223.0)
        );
        let baseline_missing_one_round: Vec<SeparatedScheduleRound> =
            SEPARATED_SCHEDULE_MEMORY_BASELINE
                .into_iter()
                .filter(|row| row.0 != 3)
                .collect();
        let reading_with_one_baseline_round_missing = separated_schedule_reading(
            WriteInjection::SpinTwoMillisecondsOfThreadCpu,
            &baseline_missing_one_round,
            &SEPARATED_SCHEDULE_MEMORY_SPIN,
            WaitReadingOfTheFixture::AsBuilt,
        );
        assert_eq!(reading_with_one_baseline_round_missing.paired_rounds, 4);
        assert_eq!(
            reading_with_one_baseline_round_missing.write_wait_increment_paired_median_nanoseconds,
            Some(f64::midpoint(39_555.0, 63_475.0))
        );
    }

    /// 登记第十二节 12.9：对照点每一轮在 K = 8 那一格里，每个设备臂挨着量基线、盘臂对照、CPU 臂对照各一次，次序按轮号轮换；
    /// 别的格与别的取样点只量基线。
    #[test]
    fn row_one_controls_interleave_the_three_injections_inside_every_round() {
        let none = WriteInjection::Nothing;
        let sleep = WriteInjection::SleepTwoMilliseconds;
        let spin = WriteInjection::SpinTwoMillisecondsOfThreadCpu;
        assert_eq!(
            (1..=ROUNDS)
                .map(row_one_control_injections_of_the_round)
                .collect::<Vec<_>>(),
            vec![
                [none, sleep, spin],
                [sleep, spin, none],
                [spin, none, sleep],
                [none, sleep, spin],
                [sleep, spin, none],
            ]
        );
        for round in 1..=ROUNDS {
            let plan = measurements_of_the_round(CONTROL_POINT, round);
            assert_eq!(
                plan.len(),
                2 + 2 * 3 + 2,
                "第 {round} 轮：K = 4、16 各两臂，K = 8 两臂各三种注入"
            );
            for arm in DeviceArm::ALL {
                let positions: Vec<usize> = plan
                    .iter()
                    .enumerate()
                    .filter(|(_position, (batch_units, planned_arm, _injection))| {
                        *batch_units == CONTROL_BATCH_UNITS && *planned_arm == arm
                    })
                    .map(|(position, _measurement)| position)
                    .collect();
                assert_eq!(
                    positions.len(),
                    3,
                    "第 {round} 轮 {}：三种注入各一次",
                    arm.label()
                );
                assert_eq!(
                    positions[2],
                    positions[0] + 2,
                    "第 {round} 轮 {}：三种注入挨着",
                    arm.label()
                );
                assert_eq!(
                    positions
                        .iter()
                        .map(|position| plan[*position].2)
                        .collect::<Vec<_>>(),
                    row_one_control_injections_of_the_round(round).to_vec()
                );
            }
            assert!(plan
                .iter()
                .filter(|(batch_units, _arm, _injection)| *batch_units != CONTROL_BATCH_UNITS)
                .all(|(_batch_units, _arm, injection)| *injection == none));
        }
        for point in every_sampling_point()
            .into_iter()
            .filter(|point| *point != CONTROL_POINT)
        {
            let plan = measurements_of_the_round(point, 1);
            assert_eq!(plan.len(), BATCH_SIZES.len() * DeviceArm::ALL.len());
            assert!(plan
                .iter()
                .all(|(_batch_units, _arm, injection)| *injection == none));
        }
    }

    // ───────────── 不计时的准备并行（登记第十二节 12.10） ─────────────

    #[test]
    fn memory_cap_takes_the_environment_then_the_control_group_then_twenty_eight_gibibytes() {
        assert_eq!(
            memory_cap_given(Some("16G"), Some("8589934592\n")),
            MemoryCap {
                bytes: 17_179_869_184,
                source: MemoryCapSource::Environment
            }
        );
        assert_eq!(
            memory_cap_given(None, Some("8589934592\n")),
            MemoryCap {
                bytes: 8_589_934_592,
                source: MemoryCapSource::ControlGroup
            }
        );
        let default_cap = MemoryCap {
            bytes: 30_064_771_072,
            source: MemoryCapSource::Default,
        };
        assert_eq!(memory_cap_given(None, Some("max\n")), default_cap);
        assert_eq!(memory_cap_given(None, None), default_cap);
        assert_eq!(bytes_of_a_memory_size_text("512M"), Some(536_870_912));
        assert_eq!(bytes_of_a_memory_size_text("28"), Some(28));
        assert_eq!(bytes_of_a_memory_size_text("0G"), None);
        assert_eq!(bytes_of_a_memory_size_text("lots"), None);
    }

    #[test]
    #[should_panic(expected = "E185_MEMORY_CAP")]
    fn memory_cap_refuses_an_unreadable_environment_value() {
        let _cap = memory_cap_given(Some("lots"), None);
    }

    #[test]
    fn pool_footprint_estimates_before_construction_at_the_registered_pool_sizes() {
        let estimate = |pool_data_units: u64| {
            estimated_pool_footprint_bytes_before_construction(SamplingPoint {
                fraction: LiveSlotFraction::OneQuarter,
                pool_data_units,
            })
        };
        // 两块盘 × (2P + 64) 个单元 × 32768 字节 × 2（元数据同等量）÷ 512 × 640
        assert_eq!(estimate(1024), 346_030_080);
        assert_eq!(estimate(8192), 2_694_840_320);
    }

    #[test]
    fn pool_footprint_counts_six_hundred_forty_bytes_per_written_sector() {
        let mut devices = Vec::new();
        for (device_number, sectors) in [(0_u32, 3_u64), (1, 2)] {
            let mut device = SparseBlockDevice::new(
                LOCAL_DEVICE_BYTES,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            );
            for sector in 0..sectors {
                device
                    .image
                    .write(DeviceOffsetInBytes(sector * 4096), &[7_u8; 512]);
            }
            devices.push((DeviceIdentity(device_number), device));
        }
        let pool = ConstructedPool {
            devices,
            live_data_inode_by_slot: BTreeMap::new(),
            unit_area_start_slot: 0,
            overwritten_units: 0,
            filler_units: 0,
            floor_raises_pushed_by_the_session: 0,
            release_rounds: 0,
            is_every_overwritten_slot_released: true,
            construction_seconds: 0.0,
        };
        assert_eq!(footprint_bytes_of_the_pool(&pool), 3200, "5 个扇区 × 640");
    }

    #[test]
    fn preparation_budget_subtracts_the_held_pools_and_the_measurement_headroom() {
        assert_eq!(
            preparation_budget_bytes(28 << 30, 4 << 30, 2 << 30),
            22 << 30
        );
        assert_eq!(preparation_budget_bytes(28 << 30, 0, 0), 28 << 30);
        assert_eq!(preparation_budget_bytes(4 << 30, 3 << 30, 2 << 30), 0);
    }

    #[test]
    fn next_group_takes_the_smaller_of_the_threads_and_the_memory_budget_over_one_peak() {
        let peaks = vec![2_u64 << 30; 20];
        assert_eq!(
            tasks_of_the_next_group(&peaks, 31, 28 << 30),
            14,
            "28 GiB ÷ 2 GiB = 14 份，少于 31 条线程"
        );
        assert_eq!(
            tasks_of_the_next_group(&peaks, 4, 28 << 30),
            4,
            "线程少时按线程"
        );
        assert_eq!(
            tasks_of_the_next_group(&peaks[..3], 31, 28 << 30),
            3,
            "份数少时按份数"
        );
        assert_eq!(
            tasks_of_the_next_group(&[5 << 30, 5 << 30, 1 << 30, 1 << 30], 31, 11 << 30),
            3,
            "峰值不同时按次序累加到预算"
        );
    }

    #[test]
    fn next_group_holds_none_when_one_peak_exceeds_the_budget_or_nothing_is_left() {
        assert_eq!(
            tasks_of_the_next_group(&[3 << 30, 3 << 30], 31, 2 << 30),
            0,
            "保底一份不在这里，在队列里"
        );
        assert_eq!(
            tasks_of_the_next_group(&[1 << 30, 3 << 30, 1 << 30], 31, 2 << 30),
            1,
            "装不下的那一份之后的都不派"
        );
        assert_eq!(tasks_of_the_next_group(&[], 31, 28 << 30), 0);
    }

    fn preparation_settings_with(threads: usize, memory_cap_bytes: u64) -> PreparationSettings {
        PreparationSettings {
            threads,
            memory_cap: MemoryCap {
                bytes: memory_cap_bytes,
                source: MemoryCapSource::Environment,
            },
        }
    }

    fn small_point(pool_data_units: u64) -> SamplingPoint {
        SamplingPoint {
            fraction: LiveSlotFraction::ThreeQuarters,
            pool_data_units,
        }
    }

    fn constructed_pool_without_devices() -> ConstructedPool {
        ConstructedPool {
            devices: Vec::new(),
            live_data_inode_by_slot: BTreeMap::new(),
            unit_area_start_slot: 0,
            overwritten_units: 0,
            filler_units: 0,
            floor_raises_pushed_by_the_session: 0,
            release_rounds: 0,
            is_every_overwritten_slot_released: true,
            construction_seconds: 0.0,
        }
    }

    /// 假的准备：不造池、不建镜像。`point_whose_workload_changes` 那一点的几何核交回改过的配方（多覆盖写 inode 7），池的内存都报 `footprint_bytes`。
    fn fake_preparation_outcome(
        work: PreparationWork,
        point_whose_workload_changes: Option<SamplingPoint>,
        footprint_bytes: u64,
    ) -> PreparationOutcome {
        match work {
            PreparationWork::PrepareThePoint(point) => {
                let mut recipe = first_recipe_of(point);
                if point_whose_workload_changes == Some(point) {
                    recipe.extra_overwritten_inodes.insert(7);
                }
                PreparationOutcome::Prepared {
                    prepared: PreparedPoint {
                        recipe,
                        choice: ChoiceOfTheRegion::Greedy,
                        is_the_geometry_formed: true,
                        pool: constructed_pool_without_devices(),
                        reference_digests: Vec::new(),
                        lines: vec![format!("name=geometry_attempt {}", point.label())],
                    },
                    footprint_bytes,
                }
            }
            PreparationWork::BuildTheRealDiskImages { .. } => {
                PreparationOutcome::RealDiskImages(Err("fake_images_not_built".to_string()))
            }
            PreparationWork::Rebuild(recipe) => PreparationOutcome::Rebuilt {
                rebuilt: RebuiltPool {
                    recipe,
                    pool: constructed_pool_without_devices(),
                    digests: Vec::new(),
                },
                footprint_bytes,
            },
        }
    }

    #[test]
    fn preparation_units_follow_the_measurement_order_with_the_images_before_rounds_two_to_five() {
        let rounds_of = |point_index: usize| {
            (2..=5).map(move |round| PreparationUnit::RebuildForTheRound { point_index, round })
        };
        let mut expected = Vec::new();
        for point_index in 0..2 {
            expected.push(PreparationUnit::PrepareThePoint { point_index });
            expected.push(PreparationUnit::BuildTheRealDiskImages { point_index });
            expected.extend(rounds_of(point_index));
        }
        assert_eq!(preparation_units_in_measurement_order(2, true), expected);
        assert_eq!(
            preparation_units_in_measurement_order(1, false),
            std::iter::once(PreparationUnit::PrepareThePoint { point_index: 0 })
                .chain(rounds_of(0))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn preparation_unit_peaks_are_two_pools_to_prepare_five_quarters_to_rebuild_and_one_pool_for_the_images(
    ) {
        let points = every_sampling_point();
        let settings = preparation_settings_with(31, 28 << 30);
        let worker = |work: PreparationWork| fake_preparation_outcome(work, None, 400_000_000);
        let directory = Path::new("e185-peak-directory-never-created");
        let mut queue =
            PreparationQueue::new("wall_clock", &points, Some(directory), &settings, &worker);
        let prepare_the_smallest = PreparationUnit::PrepareThePoint { point_index: 0 };
        assert_eq!(
            queue.peak_bytes_of(prepare_the_smallest),
            692_060_160,
            "两份池（造好的与几何核的拷贝），按估的"
        );
        assert_eq!(
            queue.peak_bytes_of(PreparationUnit::RebuildForTheRound {
                point_index: 1,
                round: 2
            }),
            3_368_550_400,
            "还没造好的点按估的 5/4"
        );
        assert_eq!(
            queue.headroom_bytes(),
            5_389_680_640,
            "余量是最大一份池（估的）的两倍"
        );
        let work = queue.start(prepare_the_smallest);
        queue.finish(prepare_the_smallest, worker(work));
        let rebuild = PreparationUnit::RebuildForTheRound {
            point_index: 0,
            round: 2,
        };
        assert_eq!(
            queue.peak_bytes_of(rebuild),
            500_000_000,
            "造好了按真写过的 5/4"
        );
        assert_eq!(
            queue.peak_bytes_of(PreparationUnit::BuildTheRealDiskImages { point_index: 0 }),
            400_000_000
        );
        assert_eq!(queue.accounted_bytes(), 400_000_000, "造好的池留在账上");
    }

    #[test]
    fn first_start_under_twenty_eight_gibibytes_takes_fifteen_units_in_the_measurement_order_and_under_eight_takes_five(
    ) {
        let points = every_sampling_point();
        let directory = Path::new("e185-first-start-directory-never-created");
        let settings = preparation_settings_with(31, 28 << 30);
        let queue = PreparationQueue::new(
            "wall_clock",
            &points,
            Some(directory),
            &settings,
            &run_the_preparation_work,
        );
        let (units, is_guaranteed) = queue.units_to_start(31, true);
        let expected: Vec<PreparationUnit> = queue
            .order
            .iter()
            .copied()
            .filter(|unit| !matches!(unit, PreparationUnit::BuildTheRealDiskImages { .. }))
            .take(15)
            .collect();
        assert_eq!(
            units, expected,
            "按量的次序派；镜像要等那一点造好；第四点那五份装不下"
        );
        assert!(!is_guaranteed);
        assert_eq!(
            units
                .iter()
                .map(|unit| queue.peak_bytes_of(*unit))
                .sum::<u64>(),
            23_708_303_360,
            "前三点各两份池加四份 5/4"
        );
        assert_eq!(
            queue.headroom_bytes() + 24_675_090_432,
            30_064_771_072,
            "预算 = 28 GiB − 两份最大的池"
        );
        assert_eq!(queue.units_to_start(4, true).0.len(), 4, "线程少时按线程");
        let settings_under_eight = preparation_settings_with(31, 8 << 30);
        let queue_under_eight = PreparationQueue::new(
            "wall_clock",
            &points,
            Some(directory),
            &settings_under_eight,
            &run_the_preparation_work,
        );
        assert_eq!(queue_under_eight.units_to_start(31, true).0.len(), 5);
        assert_eq!(
            preparation_configuration_fields(&points, Some(directory), &settings),
            "preparation_threads=31 preparation_memory_cap_bytes=30064771072 preparation_memory_cap_source=environment_variable preparation_units=24 preparation_units_started_at_once_by_the_estimates=15 preparation_plan_by_the_estimates=in_batches"
        );
        assert_eq!(
            preparation_configuration_fields(&points, None, &preparation_settings_with(31, 64 << 30)),
            "preparation_threads=31 preparation_memory_cap_bytes=68719476736 preparation_memory_cap_source=environment_variable preparation_units=20 preparation_units_started_at_once_by_the_estimates=20 preparation_plan_by_the_estimates=all_at_once"
        );
    }

    #[test]
    fn real_disk_images_of_a_point_wait_for_its_pool_and_for_every_point_before_it_to_be_measured()
    {
        let points = [small_point(96), small_point(128)];
        let settings = preparation_settings_with(31, 28 << 30);
        let worker = |work: PreparationWork| fake_preparation_outcome(work, None, 1000);
        let directory = Path::new("e185-image-order-directory-never-created");
        let mut queue =
            PreparationQueue::new("wall_clock", &points, Some(directory), &settings, &worker);
        let images_of_the_larger = PreparationUnit::BuildTheRealDiskImages { point_index: 1 };
        assert!(!queue.is_ready(images_of_the_larger), "那一点还没造好");
        for point_index in 0..2 {
            let unit = PreparationUnit::PrepareThePoint { point_index };
            let work = queue.start(unit);
            queue.finish(unit, worker(work));
        }
        assert!(queue.is_ready(PreparationUnit::BuildTheRealDiskImages { point_index: 0 }));
        assert!(
            !queue.is_ready(images_of_the_larger),
            "前一点还没量完，盘上不建第二份镜像"
        );
        queue.finish_the_point(0);
        assert!(queue.is_ready(images_of_the_larger));
        assert_eq!(queue.accounted_bytes(), 1000, "量完的那一点不再记账");
    }

    #[test]
    fn later_points_start_only_when_all_of_their_units_fit() {
        let point = small_point(96);
        let pool_bytes = estimated_pool_footprint_bytes_before_construction(point);
        let worker = |work: PreparationWork| fake_preparation_outcome(work, None, pool_bytes);
        // 余量 2 份池；一点整点是 2 份（造池与几何核）+ 4 × 5/4 份（重造）= 7 份。上限 12 份时第二点只装得下造池那一份，整点不派
        for (cap_in_pools, expected_units) in [(12, 5), (16, 10)] {
            let settings = preparation_settings_with(31, cap_in_pools * pool_bytes);
            let queue =
                PreparationQueue::new("stage_split", &[point, point], None, &settings, &worker);
            let (units, is_guaranteed) = queue.units_to_start(31, true);
            assert_eq!(units.len(), expected_units, "上限 {cap_in_pools} 份池");
            assert!(!is_guaranteed);
        }
    }

    #[test]
    fn serial_declarations_append_the_parallel_and_serial_lines_in_order_and_skip_repeats() {
        assert_eq!(
            declaration_line(
                DeclaredSegment::Serial("计时窗口必须单线程跑完"),
                1_700_000_000.5
            ),
            "1700000000.500\t此刻串行：计时窗口必须单线程跑完"
        );
        assert_eq!(
            declaration_line(DeclaredSegment::Parallel, 1_700_000_000.0),
            "1700000000.000\t此刻并行"
        );
        let path =
            std::env::temp_dir().join(format!("e185-serial-declarations-{}", std::process::id()));
        let _absent_before = std::fs::remove_file(&path);
        let mut declarations = SerialDeclarations::into_the_file(path.clone());
        let wall_clock =
            DeclaredSegment::Serial(TimingWindow::WallClockCompactionRun.serial_reason());
        let sequential =
            DeclaredSegment::Serial(TimingWindow::DeviceSequentialWrite.serial_reason());
        let written: Vec<bool> = [
            DeclaredSegment::Parallel,
            wall_clock,
            wall_clock,
            sequential,
            DeclaredSegment::Parallel,
        ]
        .into_iter()
        .map(|segment| declarations.declare(segment).expect("写得进临时文件"))
        .collect();
        assert_eq!(written, vec![true, true, false, true, true], "同一段不重写");
        let text = std::fs::read_to_string(&path).expect("读得回声明文件");
        std::fs::remove_file(&path).expect("删掉这条单测建的文件");
        assert!(text.ends_with('\n'), "每行带换行");
        let split: Vec<(f64, &str)> = text
            .lines()
            .map(|line| {
                let (seconds, body) = line.split_once('\t').expect("秒数与正文之间一个制表符");
                (seconds.parse::<f64>().expect("开头是 unix 秒"), body)
            })
            .collect();
        let bodies: Vec<String> = split
            .iter()
            .map(|(_seconds, body)| (*body).to_string())
            .collect();
        assert_eq!(
            bodies,
            vec![
                "此刻并行".to_string(),
                format!(
                    "此刻串行：{}",
                    TimingWindow::WallClockCompactionRun.serial_reason()
                ),
                format!(
                    "此刻串行：{}",
                    TimingWindow::DeviceSequentialWrite.serial_reason()
                ),
                "此刻并行".to_string(),
            ]
        );
        assert!(
            split.windows(2).all(|pair| pair[0].0 <= pair[1].0),
            "秒数不倒退"
        );
        assert!(
            split
                .iter()
                .all(|(seconds, _body)| *seconds > 1_700_000_000.0),
            "unix 秒"
        );
    }

    #[test]
    fn serial_declarations_write_nothing_when_unset_and_fail_on_an_unwritable_path() {
        let mut nowhere = SerialDeclarations::nowhere();
        assert!(!nowhere
            .declare(DeclaredSegment::Parallel)
            .expect("没设就不写"));
        let mut unwritable = SerialDeclarations::into_the_file(
            std::env::temp_dir()
                .join(format!("e185-no-such-directory-{}", std::process::id()))
                .join("serial"),
        );
        assert!(
            unwritable.declare(DeclaredSegment::Parallel).is_err(),
            "写不进报错，不静默"
        );
    }

    #[test]
    fn every_timing_window_gives_a_serial_reason_of_at_least_eight_characters() {
        for window in TimingWindow::ALL {
            assert!(
                window.serial_reason().chars().count() >= SERIAL_REASON_CHARACTERS_AT_LEAST,
                "{window:?}"
            );
        }
    }

    #[test]
    fn process_status_gives_the_thread_count_and_only_one_thread_counts_as_alone() {
        let status = "Name:\te185\nThreads:\t1\nSigQ:\t0/1\n";
        assert_eq!(threads_in_the_process_status(status), Some(1));
        assert!(is_only_the_measuring_thread_left(status));
        assert!(!is_only_the_measuring_thread_left(
            "Name:\te185\nThreads:\t3\n"
        ));
        assert!(!is_only_the_measuring_thread_left("Name:\te185\n"));
        let own_status =
            std::fs::read_to_string("/proc/self/status").expect("读得到本进程的 status");
        assert!(
            threads_in_the_process_status(&own_status).is_some_and(|threads| threads >= 1),
            "本进程的 status 里读得出线程数"
        );
    }

    #[test]
    fn global_queue_runs_every_unit_of_both_points_at_once_on_its_own_thread_after_declaring_parallel(
    ) {
        let points = [small_point(96), small_point(128)];
        let units_in_the_queue = 10_u64;
        let arrived = AtomicU64::new(0);
        let running = AtomicU64::new(0);
        let most_running = AtomicU64::new(0);
        let worker = |work: PreparationWork| {
            let now_running = running.fetch_add(1, Ordering::SeqCst) + 1;
            most_running.fetch_max(now_running, Ordering::SeqCst);
            arrived.fetch_add(1, Ordering::SeqCst);
            let waited = Instant::now();
            // 等十份都起来（至多 10 秒）：同时在跑的份数才看得出来
            while arrived.load(Ordering::SeqCst) < units_in_the_queue
                && waited.elapsed() < Duration::from_secs(10)
            {
                std::thread::sleep(Duration::from_millis(1));
            }
            running.fetch_sub(1, Ordering::SeqCst);
            fake_preparation_outcome(work, None, 1000)
        };
        let declarations_path =
            std::env::temp_dir().join(format!("e185-phase-declarations-{}", std::process::id()));
        let _absent_before = std::fs::remove_file(&declarations_path);
        let mut emitter = Emitter::with_declarations(SerialDeclarations::into_the_file(
            declarations_path.clone(),
        ));
        let settings = preparation_settings_with(31, 28 << 30);
        let mut queue = PreparationQueue::new("stage_split", &points, None, &settings, &worker);
        let _prepared = queue.prepared_point(0, &mut emitter);
        for point_index in 0..2 {
            for round in 2..=ROUNDS {
                let _rebuilt = queue.take_the_rebuilt_pool(point_index, round, &mut emitter);
            }
        }
        assert_eq!(queue.phases.len(), 1, "一次准备就把两点十份全做完");
        let phase = &queue.phases[0];
        assert_eq!(
            (
                phase.started_prepare_units,
                phase.started_rebuild_units,
                phase.started_real_disk_image_units
            ),
            (2, 8, 0)
        );
        assert_eq!(
            phase.most_threads_at_once, 10,
            "十份同时派出，不是每点一组的至多五份"
        );
        assert_eq!(phase.distinct_threads, 10, "每份各一条线程");
        assert_eq!(most_running.load(Ordering::SeqCst), 10, "十份真同时在跑");
        assert_eq!(running.load(Ordering::SeqCst), 0, "交回时准备线程都已停");
        let declarations = std::fs::read_to_string(&declarations_path).expect("准备开始时写了声明");
        std::fs::remove_file(&declarations_path).expect("删掉这条单测建的文件");
        let bodies: Vec<&str> = declarations
            .lines()
            .map(|line| line.split_once('\t').map_or("", |(_seconds, body)| body))
            .collect();
        assert_eq!(
            bodies,
            vec![PARALLEL_DECLARATION_MARK],
            "准备开始时声明一次并行"
        );
    }

    #[test]
    fn speculative_rebuilds_of_a_point_whose_geometry_changed_the_workload_are_voided_and_rebuilt_with_the_new_recipe(
    ) {
        let changed = small_point(96);
        let unchanged = small_point(128);
        let points = [changed, unchanged];
        let worker = |work: PreparationWork| fake_preparation_outcome(work, Some(changed), 1000);
        let settings = preparation_settings_with(31, 28 << 30);
        let mut emitter = Emitter::with_declarations(SerialDeclarations::nowhere());
        let mut queue = PreparationQueue::new("stage_split", &points, None, &settings, &worker);
        let prepared_changed = queue.prepared_point(0, &mut emitter);
        assert_eq!(
            prepared_changed.recipe.extra_overwritten_inodes,
            BTreeSet::from([7])
        );
        let phase_that_prepared_the_changed_point = queue.phases[0].clone();
        assert_eq!(
            phase_that_prepared_the_changed_point.started_speculative_rebuild_units, 8,
            "两点的重造都在几何核定之前推测着派出"
        );
        assert_eq!(
            phase_that_prepared_the_changed_point.speculative_rebuilds_voided, 4,
            "改了负载的那一点四份作废"
        );
        assert_eq!(queue.points[0].speculative_rebuilds_voided, 4);
        assert_eq!(queue.points[1].speculative_rebuilds_voided, 0);
        for round in 2..=ROUNDS {
            let rebuilt = queue.take_the_rebuilt_pool(0, round, &mut emitter);
            assert_eq!(
                rebuilt.recipe, prepared_changed.recipe,
                "第 {round} 轮按新配方重造"
            );
        }
        assert_eq!(queue.phases.len(), 2, "作废的四份在下一次准备里重造");
        assert_eq!(
            (
                queue.phases[1].started_rebuild_units,
                queue.phases[1].started_speculative_rebuild_units
            ),
            (4, 0)
        );
        queue.finish_the_point(0);
        let prepared_unchanged = queue.prepared_point(1, &mut emitter);
        assert_eq!(prepared_unchanged.recipe, first_recipe_of(unchanged));
        for round in 2..=ROUNDS {
            let rebuilt = queue.take_the_rebuilt_pool(1, round, &mut emitter);
            assert_eq!(rebuilt.recipe, prepared_unchanged.recipe);
        }
        assert_eq!(queue.phases.len(), 2, "没改负载的那一点推测造的照用");
    }

    #[test]
    fn global_queue_keeps_the_pools_in_hand_under_the_memory_cap_and_measures_in_batches() {
        let point = small_point(96);
        let points = [point, point];
        let pool_bytes = estimated_pool_footprint_bytes_before_construction(point);
        assert_eq!(pool_bytes, 41_943_040);
        // 在手的池按四分之一份计：造池与几何核在跑 8，重造在跑 5，造好没取走的 4
        let quarters_in_hand = AtomicU64::new(0);
        let most_quarters_in_hand = AtomicU64::new(0);
        let worker = |work: PreparationWork| {
            let peak_quarters = match &work {
                PreparationWork::PrepareThePoint(_) => 8,
                PreparationWork::Rebuild(_) => 5,
                PreparationWork::BuildTheRealDiskImages { .. } => 4,
            };
            let now_in_hand =
                quarters_in_hand.fetch_add(peak_quarters, Ordering::SeqCst) + peak_quarters;
            most_quarters_in_hand.fetch_max(now_in_hand, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(5));
            let outcome = fake_preparation_outcome(work, None, pool_bytes);
            quarters_in_hand.fetch_sub(peak_quarters, Ordering::SeqCst);
            quarters_in_hand.fetch_add(4, Ordering::SeqCst);
            outcome
        };
        // 预算 = 6 份 − 余量 2 份 = 4 份 = 16 个四分之一
        let settings = preparation_settings_with(31, 6 * pool_bytes);
        let mut emitter = Emitter::with_declarations(SerialDeclarations::nowhere());
        let mut queue = PreparationQueue::new("stage_split", &points, None, &settings, &worker);
        for point_index in 0..2 {
            let _prepared = queue.prepared_point(point_index, &mut emitter);
            for round in 2..=ROUNDS {
                let _rebuilt = queue.take_the_rebuilt_pool(point_index, round, &mut emitter);
                quarters_in_hand.fetch_sub(4, Ordering::SeqCst);
            }
            queue.finish_the_point(point_index);
            quarters_in_hand.fetch_sub(4, Ordering::SeqCst);
        }
        assert!(
            most_quarters_in_hand.load(Ordering::SeqCst) <= 16,
            "同时在手的不超过预算：{} 个四分之一",
            most_quarters_in_hand.load(Ordering::SeqCst)
        );
        assert_eq!(quarters_in_hand.load(Ordering::SeqCst), 0);
        let started_per_phase: Vec<(usize, usize)> = queue
            .phases
            .iter()
            .map(|phase| (phase.started_prepare_units, phase.started_rebuild_units))
            .collect();
        assert_eq!(
            started_per_phase,
            vec![(1, 1), (0, 2), (0, 1), (1, 1), (0, 2), (0, 1)],
            "装不下就分批：批内并行、批间串行量；要的那一份好了就不再派新的"
        );
        assert_eq!(
            queue.phases[0].largest_accounted_bytes,
            2 * pool_bytes + pool_bytes * 5 / 4
        );
        assert!(queue
            .phases
            .iter()
            .all(|phase| phase.largest_accounted_bytes <= 4 * pool_bytes
                && phase.guaranteed_units == 0));
    }

    #[test]
    fn queue_with_no_memory_to_spare_still_prepares_one_unit_at_a_time() {
        let points = [small_point(96)];
        let worker = |work: PreparationWork| fake_preparation_outcome(work, None, 1000);
        let settings = preparation_settings_with(31, 1);
        let mut emitter = Emitter::with_declarations(SerialDeclarations::nowhere());
        let mut queue = PreparationQueue::new("stage_split", &points, None, &settings, &worker);
        let _prepared = queue.prepared_point(0, &mut emitter);
        for round in 2..=ROUNDS {
            let _rebuilt = queue.take_the_rebuilt_pool(0, round, &mut emitter);
        }
        assert_eq!(queue.phases.len(), 5);
        assert!(queue
            .phases
            .iter()
            .all(|phase| phase.guaranteed_units == 1 && phase.most_threads_at_once == 1));
    }

    #[test]
    fn phase_starts_no_new_unit_once_the_wanted_one_is_ready() {
        let point = small_point(96);
        let pool_bytes = estimated_pool_footprint_bytes_before_construction(point);
        let started = AtomicU64::new(0);
        // 重造等到第三份开派（要的那一份好了之后还在派新的）或 300 毫秒：造池与几何核（要的那一份）早已做完、交回
        let worker = |work: PreparationWork| {
            started.fetch_add(1, Ordering::SeqCst);
            if matches!(work, PreparationWork::Rebuild(_)) {
                let waited = Instant::now();
                while started.load(Ordering::SeqCst) < 3
                    && waited.elapsed() < Duration::from_millis(300)
                {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            fake_preparation_outcome(work, None, pool_bytes)
        };
        // 预算 4 份池：开头派造池与几何核（2 份）与第 2 轮重造（5/4 份）；造好之后账上只剩 1 份池加在跑的重造，第 3 轮重造装得下
        let settings = preparation_settings_with(31, 6 * pool_bytes);
        let mut emitter = Emitter::with_declarations(SerialDeclarations::nowhere());
        let mut queue = PreparationQueue::new("stage_split", &[point], None, &settings, &worker);
        let _prepared = queue.prepared_point(0, &mut emitter);
        assert_eq!(
            (
                queue.phases[0].started_prepare_units,
                queue.phases[0].started_rebuild_units
            ),
            (1, 1),
            "要的那一份好了就不再派新的，等在跑的交回"
        );
        assert_eq!(started.load(Ordering::SeqCst), 2);
    }

    /// 准备行里去掉造池计时那一段（计时每次不同，其余逐字比）。
    fn lines_without_the_construction_seconds<'prepared>(
        prepared_points: impl IntoIterator<Item = &'prepared PreparedPoint>,
    ) -> Vec<String> {
        prepared_points
            .into_iter()
            .flat_map(|prepared| prepared.lines.iter())
            .map(|line| {
                line.split(' ')
                    .filter(|field| !field.starts_with("construction_seconds="))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    }

    #[test]
    fn global_queue_of_small_pools_gives_the_serial_digests_and_lines_in_the_point_order() {
        let points = [small_point(128), small_point(96)];
        let serial: Vec<PreparedPoint> = points
            .iter()
            .map(|point| prepare_the_sampling_point(*point))
            .collect();
        let settings = preparation_settings_with(31, 28 << 30);
        let mut emitter = Emitter::with_declarations(SerialDeclarations::nowhere());
        let mut queue = PreparationQueue::new(
            "stage_split",
            &points,
            None,
            &settings,
            &run_the_preparation_work,
        );
        let mut queued: Vec<Arc<PreparedPoint>> = Vec::new();
        for (point_index, serial_point) in serial.iter().enumerate() {
            let prepared = queue.prepared_point(point_index, &mut emitter);
            assert_eq!(prepared.recipe, serial_point.recipe);
            assert_eq!(
                digests_of_the_pool(&prepared.pool),
                digests_of_the_pool(&serial_point.pool),
                "队列里造的池与串行造的逐块相同（{}）",
                serial_point.recipe.point.label()
            );
            assert_eq!(prepared.reference_digests, serial_point.reference_digests);
            for round in 2..=ROUNDS {
                let rebuilt = queue.take_the_rebuilt_pool(point_index, round, &mut emitter);
                assert_eq!(rebuilt.recipe, serial_point.recipe);
                assert_eq!(
                    rebuilt.digests, serial_point.reference_digests,
                    "第 {round} 轮的重造与串行造的相同"
                );
            }
            queue.finish_the_point(point_index);
            queued.push(prepared);
        }
        let queued_lines = lines_without_the_construction_seconds(queued.iter().map(Arc::as_ref));
        assert_eq!(
            queued_lines,
            lines_without_the_construction_seconds(&serial)
        );
        assert!(
            queued_lines
                .first()
                .is_some_and(|line| line
                    .starts_with("name=geometry_attempt fraction=0.75 pool_data_units=128 ")),
            "头一行是先量的那一点：{queued_lines:?}"
        );
        let changed_points = serial
            .iter()
            .filter(|serial_point| {
                serial_point.recipe != first_recipe_of(serial_point.recipe.point)
            })
            .count();
        let phase = &queue.phases[0];
        assert_eq!(phase.most_threads_at_once, 10, "两点十份一次派出");
        assert_eq!(phase.started_speculative_rebuild_units, 8);
        assert_eq!(phase.speculative_rebuilds_voided, changed_points * 4);
    }

    #[test]
    fn dropping_the_page_cache_of_a_file_keeps_its_bytes_and_a_missing_file_is_an_error() {
        let path =
            std::env::temp_dir().join(format!("e185-page-cache-drop-{}", std::process::id()));
        std::fs::write(&path, vec![9_u8; 8192]).expect("写得进临时文件");
        drop_the_page_cache_of_the_file(&path).expect("扔页缓存成功");
        assert_eq!(std::fs::read(&path).expect("读得回"), vec![9_u8; 8192]);
        std::fs::remove_file(&path).expect("删掉这条单测建的文件");
        assert!(
            drop_the_page_cache_of_the_file(&path).is_err(),
            "文件不在时报错"
        );
    }
}
