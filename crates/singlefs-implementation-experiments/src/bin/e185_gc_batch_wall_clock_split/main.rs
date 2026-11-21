//! E185（GC一批搬迁的挂钟拆分）全部四个子命令（跑前登记 `research/prompts/e185-preregistration.md` 第五节「装置住哪」写死）：
//! 在 `crates/` 今天的 GC 路径（`MountedSession::run_compaction`）上量一批搬迁的挂钟，拆成等盘读写与 CPU，再把 CPU 拆成三段
//! （甲 = 校验和核对、乙 = 区域扫描里其余的解头与映射反查、丙 = 选段打分），量 GPU 一次提交的固定开销与每字节开销、GC 单跑的写带宽。
//! 回答问题单 `research/prompts/m4-gpugc-measure-forks.md` 的四行。
//!
// admission: always 每跑一次都在今天这份 crates 上现造池、现量挂钟，计时每次都是新观测，上一次跑完不替这一次作保
// run-condition: command python3
//!
//! 要卡的四个子命令（`gpu-submission`、`host-cpu-per-submission`、`recompute-disagreement`、`host-cpu-drift-control`）只用服务交给这个进程的卡
//! （`cards_and_pairing_of_the_gpu_segments::card_text_handed_over_now`）：环境变量 `GPU_CARDS_BY_PRIORITY` 交了卡串就照它；没交（没设、空串或只有空白）
//! 报没经服务给卡那一句、退 78，不借卡（这个装置不在 GPU 批处理验证服务的借卡白名单里）、不枚举本机的卡。`--cards` 只在这一台交来的卡里按给的次序挑
//! （`nvidia-smi` 序号），点到交来的卡之外当场报错、退 78（`cards_and_pairing_of_the_gpu_segments::cards_picked_among_those_handed_over`）。
//! `gpu-submission` 与带卡单测 `resident_kernels_agree_with_the_cpu_and_kernel_one_agrees_with_the_crates_on_seeded_units` 只比结果，走先录后判
//! （段由 `SINGLEFS_CRASH_AMPLIFICATION_STAGE` 定：`record` 只录、`judge` 只判、不设是同一个进程里先录后判）：只录那一段在 CPU 上造输入、算 CPU 参照，
//! 经验证包 `pairing_input_and_reference_through_the_pairing_library` 写进判定库，不取卡；判段读回之后才取卡、只跑 GPU、比读回的参照。
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
//!       没编 `gpu` 特性时退 78，报「本次未跑：没编 gpu 特性」。整条命令经 `task-server/task submit` 交，卡与清场由服务定。
//!       只录那一段（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）造一个数据单元、算两种段长的 CPU 参照 CRC、写进判定库，打 `name=recorded_only`、退 0，不取卡。
//!   foreground-fsync --image-directory <目录> --point <活槽占比>:<池的数据量> --batch-units <K> [--seconds <秒>]
//!       第四段（附带，只在 Q16 > 10% 时跑）：前台「4 KiB 直接写 + fdatasync」循环的延迟，GC 并发与不跑 GC 各 5 轮。
//!   geometry-check [--point <活槽占比>:<池的数据量>]… [--overwrite-order slot-order]
//!       单点核（第 2 次重跑登记 `research/prompts/e185-r2-prereg.md` 第五节「单点核」）：只造池、核几何，打 `name=geometry_attempt`、
//!       `name=geometry`、`name=segment_composition`、`name=overwrite_landing`、`name=pool`，不量。不给 `--overwrite-order` 时照改法甲造
//!       （占比 0.25 槽号最大的那一整段留到最后覆盖写）；`slot-order` 照第 1 次重跑那一版造（全部按旧槽号升序，不数副本落点）。
//!   host-cpu-drift-control --cards <序号,…> --stage-split-product <文件> --attack-ledger <文件>
//!       第 4 次重跑控漂移段（登记 `research/prompts/e185-r4-prereg.md` 第五节「控漂移段怎么量」，编了 `gpu` 特性才有）：复现臂照第 3 次重跑第一段
//!       原样量头一张卡；受控臂绑核、读频率、暖到稳、每格暖到稳、两张卡逐轮交替；每轮末尾两格漂移探针；两条臂各自照第 3 次重跑算点，再算 D1–D7。
//!       没编 `gpu` 特性时退 78。整条命令经 `task-server/task submit` 交，卡与清场由服务定。
//!   cpu-writings [--point <活槽占比>:<池的数据量>]…
//!       第 4 次重跑写法与拆分段（同一节「写法与拆分段怎么量」；拆法乙那一半主 agent 在装置写之前撤了，登记第一节「主 agent 定」第 6 条）：
//!       照第二段造池、量意图、重放三个阶段，每次调用之后另量九种 CPU 写法的计时件（硬件 CRC 单路与三路、按身份查的区域扫描与建索引），
//!       与 crates 逐个对拍（停机 K-S1、K-S2），每个点每轮做一次 PW1、PW2，收尾打每个点的写法系数 φ。替代写法只给计时用，不改 `crates/`。
//!
//! 第 2 次重跑（登记第五节「这一趟装置改什么」改动一至七）：造池用改法甲的覆盖写次序（[`overwrite_order_of`]）；每个点头一次造池之后逐段打成分
//! 与覆盖写副本的落点；第一、二段每条意图前后做一次校准（16 MiB 缓冲的 CRC-32C 连做 3 遍取中位、64 MiB 追指针一遍）并读线程与机器的状态，
//! 校准比超过 1.10 的意图作废，作废之后先等机器回到常态再重量（登记第十一节 R2-1 至 R2-4）；段开头与段末各做 21 遍校准的基线，段开头另做
//! 一遍加倍的活当校准自己的阳性对照；丙段的判定从签发那一行收（`name=choose_judgement`）；另报「照原作废规则」的并列判定。新读数一律另起一行，
//! 不往已有的行里加键。
//!
//! 线程数取环境变量 `E185_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值（编进来的那一份）。
//! 产物是标准输出上的 `E7RESULT` 行，末行 `name=done emitted=<行数>`；进度打到标准错误。
//! 退出码：0 正常；2 命令行不对；3 停机条款触发（装置与 `crates/` 对不上，或开跑自检判线程 CPU 时钟分辨率不够，或一段里校准锚点不符累计超过
//! 3 次；产物照打、交主 agent）；4 阳性对照没过（那条臂整段作废，产物照打；校准分不出两倍的活时那一段一个点都不量）；
//! 5 等别的性能测量（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）等满上限仍没等到，或一段里等机器回到常态累计满 600 秒；
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

#![deny(unsafe_code)]

use singlefs_implementation_experiments::experiment_pool;

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
use singlefs_core::mounted_session::{
    central_mapping_as_the_compaction_reads_it, CompactionRunOutcome, MountedSession, UserChange,
};
use singlefs_core::recovery::PoolReader;
use singlefs_core::region_scan::{
    scan_region, AllocationInRegion, CentralMappingEntry, RegionScan,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file, warm_up,
    FirstFile, MappingLookup, PoolVersion, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{
    data_unit_payload_capacity, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::sha256::sha256_digest;

mod calibration;
mod call_kinds_and_byte_anchors;
mod constants_and_anchors_check;
mod cpu_and_wall_clocks;
mod cpu_writings_and_split_segment;
mod foreground_fsync_segment;
mod function_split_segment;
mod geometry_check_segment;
mod gpu_side_arithmetic;
mod inclusion_and_judgements;
mod intent_measurement;
mod intent_voiding_and_result_lines;
mod pool_construction;
mod real_disk_images;
mod replay_inputs;
mod sampling_point_pool_and_geometry;
mod stage_split_segment;
#[cfg(test)]
mod synthetic_inputs_and_fakes;
mod thread_and_machine_readings;
mod timed_block_device;
mod untimed_preparation;
mod waiting_for_the_normal_machine;
mod wall_clock_segment;
mod wall_clock_summary;

use calibration::*;
use call_kinds_and_byte_anchors::*;
use constants_and_anchors_check::*;
use cpu_and_wall_clocks::*;
use cpu_writings_and_split_segment::*;
use foreground_fsync_segment::*;
use function_split_segment::*;
use geometry_check_segment::*;
use gpu_side_arithmetic::*;
use inclusion_and_judgements::*;
use intent_measurement::*;
use intent_voiding_and_result_lines::*;
use pool_construction::*;
use real_disk_images::*;
use replay_inputs::*;
use sampling_point_pool_and_geometry::*;
use stage_split_segment::*;
#[cfg(test)]
use synthetic_inputs_and_fakes::*;
use thread_and_machine_readings::*;
use timed_block_device::*;
use untimed_preparation::*;
use waiting_for_the_normal_machine::*;
use wall_clock_segment::*;
use wall_clock_summary::*;

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
/// 每块盘字节：harness 测试共用模块的 `IMAGE_BYTES`（回比验证包库给这一类池定的盘宽 `history::HistoryDeviceWidth::FourGibibytes`，
/// 库里写明与那份文件镜像同宽；另回比建出来的内存盘的 `size_in_bytes()`）。
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
/// 根环条数：值抄自第 2 次重跑登记第五节「装置读 `crates/` 的常量」那张表（回比
/// `singlefs_format::ROOT_RING_REGIONS × singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`）。只用来打第七节「推的预测」那一栏。
const LOCAL_ROOT_RING_ENTRIES: u64 = 24;
/// 第 4 次重跑写法丙抄的那一份区域扫描用到的三个类标签（登记 `research/prompts/e185-r4-prereg.md` 第五节「入库装置这一趟新读 crates 的东西」：
/// 回比 `crates/singlefs-core/src/unit.rs` 的 `UNIT_CLASS_DATA`、`UNIT_CLASS_INDEX_NODE`、`UNIT_CLASS_PACKED`）。
const LOCAL_UNIT_CLASS_DATA: u8 = 1;
const LOCAL_UNIT_CLASS_INDEX_NODE: u8 = 2;
const LOCAL_UNIT_CLASS_PACKED: u8 = 3;

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
    compare(
        "device_bytes",
        LOCAL_DEVICE_BYTES,
        singlefs_verification::history::HistoryDeviceWidth::FourGibibytes.device_bytes(),
    );
    for (name, local_tag, crates_tag) in [
        ("unit_class_data", LOCAL_UNIT_CLASS_DATA, UNIT_CLASS_DATA),
        (
            "unit_class_index_node",
            LOCAL_UNIT_CLASS_INDEX_NODE,
            UNIT_CLASS_INDEX_NODE,
        ),
        (
            "unit_class_packed",
            LOCAL_UNIT_CLASS_PACKED,
            UNIT_CLASS_PACKED,
        ),
    ] {
        compare(name, u64::from(local_tag), u64::from(crates_tag));
    }
    compare(
        "root_ring_entries",
        LOCAL_ROOT_RING_ENTRIES,
        singlefs_format::ROOT_RING_REGIONS
            * singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
    );
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
            singlefs_verification::gpu_unit_checks::bytes_crc32c_of_ranges_needs(
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
struct Emitter {
    emitted: u64,
}

impl Emitter {
    fn new() -> Emitter {
        Emitter { emitted: 0 }
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

// ───────────────────────────── 要卡的子命令取卡、只比结果的那几处先录后判（不编 gpu 特性也有单测） ─────────────────────────────

/// 参照一侧判法单元那一组的组名：实验包 `singlefs_implementation_experiments::judging_units_entries::JUDGING_UNITS_ENTRY_GROUPS` 登记、
/// 实验包的构建脚本算键，入口是
/// [`cpu_reference_of_the_submission_unit`] 与 [`cpu_reference_of_the_slots`]。参照的代码改了，录入键跟着变，库里旧的那一份参照不再被读到。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "过库的产品路（gpu 特性）才用；没编 gpu 特性时不过库"
    )
)]
const CPU_REFERENCE_UNITS_GROUP: &str = "e185_cpu_reference_crcs";

/// gpu-submission 每次提交里一段的字节：一个整数据单元，与码 2 节点大小那一组（`crc32c_of_ranges` 按 32 KiB 一槽上传，两组的 t 应相同，报出来、不判）。
#[cfg_attr(
    not(any(feature = "gpu", test)),
    allow(dead_code, reason = "gpu-submission（gpu 特性）与单测才用")
)]
const SUBMISSION_RANGE_LENGTHS: [u64; 2] = [32_768, 16_384];

/// gpu-submission 的 CPU 参照（只录那一段算、写进库）：一个数据单元按 [`SUBMISSION_RANGE_LENGTHS`] 每种段长取开头那一截的 CRC-32C，依次 4 字节小端。
///
/// # Panics
/// 数据单元短过最长的段长（交进来的是 `one_data_unit_of_the_workload` 读回的一整个数据单元）。
#[cfg_attr(
    not(any(feature = "gpu", test)),
    allow(dead_code, reason = "gpu-submission（gpu 特性）录入段与单测才用")
)]
fn cpu_reference_of_the_submission_unit(unit: &[u8]) -> Vec<u8> {
    SUBMISSION_RANGE_LENGTHS
        .iter()
        .flat_map(|range_length| {
            let prefix = unit
                .get(..usize::try_from(*range_length).expect("段长装得进 usize"))
                .expect("数据单元不短于最长的段长（交进来的是一整个数据单元）");
            crc32_castagnoli(prefix).to_le_bytes()
        })
        .collect()
}

/// 带卡单测的 CPU 参照（只录那一段算、写进库）：[`cards_and_pairing_of_the_gpu_segments::pairing_input_of_the_slots`] 写的对拍输入里，
/// 每个单元在自己的 32 KiB 槽里取长度那一截的 CRC-32C，依次 4 字节小端。
///
/// # Panics
/// 输入不是 [`cards_and_pairing_of_the_gpu_segments::pairing_input_of_the_slots`] 写的样子。
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "带卡单测的录入段与单测才用，装置的子命令不用")
)]
fn cpu_reference_of_the_slots(input: &[u8]) -> Vec<u8> {
    let (slots, lengths) = cards_and_pairing_of_the_gpu_segments::slots_of_the_pairing_input(input);
    let slot_bytes = usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768");
    slots
        .chunks(slot_bytes)
        .zip(&lengths)
        .flat_map(|(slot, length)| {
            crc32_castagnoli(&slot[..usize::try_from(*length).expect("长度装得进 usize")])
                .to_le_bytes()
        })
        .collect()
}

/// 要卡的子命令取卡与先录后判过库：纯判法在这里（不编 gpu 特性也有单测），读调用方的卡串、开真库的那一层只在编了 gpu 特性时有。
mod cards_and_pairing_of_the_gpu_segments {
    #![cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "不编 gpu 特性时只有单测用；编了 gpu 特性时带卡单测那几个（槽的编解码与过库）只有单测用"
        )
    )]
    use super::{
        cpu_reference_of_the_slots, cpu_reference_of_the_submission_unit, LOCAL_GPU_SLOT_BYTES,
        SUBMISSION_RANGE_LENGTHS,
    };
    use singlefs_verification::crash_library::{PairingInputAndReferenceReadBack, StageOutcome};
    use singlefs_verification::gpu_card_configuration::{
        card_string_handed_over_by_the_service, cards_by_priority_from, GpuQuiesce,
    };
    use std::path::Path;

    /// 环境里没交卡串，多机配置又没开 GPU、没写显卡先后时报的那一句（多机配置开了或写了时报验证包 `cards_by_priority_from` 那一句）。
    pub(super) const NO_CARDS_HANDED_OVER_SENTENCE: &str = "没经服务给卡（环境里没有 GPU_CARDS_BY_PRIORITY，或是空白）：不借卡、不枚举本机的卡（运行条件不满足，退 78）。出路：卡只归 GPU 批处理验证服务，这个装置不在它的借卡白名单里；要卡的段交它代跑（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）";

    /// gpu-submission 过库的项名（对拍那一类库里的索引，不进键）。
    pub(super) const SUBMISSION_PAIRING_ITEM: &str =
        "e185_gc_batch_wall_clock_split/gpu_submission/one_data_unit_of_the_workload";

    /// 这个进程此刻有的卡串（与 `GPU_CARDS_BY_PRIORITY` 同格式）：`environment_value`（环境变量 `GPU_CARDS_BY_PRIORITY` 的值）交了卡串
    /// （验证包 `card_string_handed_over_by_the_service`：去掉两头空白不空）照它；没交（没设、空串或只有空白）交验证包没经服务给卡那一句
    /// （`configuration_file` 那份多机配置开了 GPU 或写了显卡先后时 `cards_by_priority_from` 报的那一句，没有时 [`NO_CARDS_HANDED_OVER_SENTENCE`]），
    /// 不借卡、不枚举本机的卡。
    ///
    /// # Errors
    /// 没交卡串。
    pub(super) fn card_text_handed_over_now(
        environment_value: Option<String>,
        configuration_file: Option<&Path>,
        quiesce: GpuQuiesce,
    ) -> Result<String, String> {
        match card_string_handed_over_by_the_service(environment_value) {
            Some(text) => Ok(text),
            None => {
                // 多机配置开了 GPU 或写了显卡先后：照验证包那一句报（环境值给的是 None，卡项一项都不读）
                cards_by_priority_from(None, configuration_file, quiesce, &|| {
                    Err("没经服务给卡：不读卡项".to_string())
                })?;
                Err(NO_CARDS_HANDED_OVER_SENTENCE.to_string())
            }
        }
    }

    /// `--cards` 点名的 `nvidia-smi` 序号（按给的次序）都要在这一台交来的卡（`handed_over`）里：交回照给的次序的序号表。
    ///
    /// # Errors
    /// 点名了交来的卡之外的序号：逐个列出，连同这一台交来的那几张。
    pub(super) fn cards_picked_among_those_handed_over(
        requested: &[u32],
        handed_over: &[u32],
    ) -> Result<Vec<u32>, String> {
        let outside: Vec<u32> = requested
            .iter()
            .copied()
            .filter(|index| !handed_over.contains(index))
            .collect();
        if outside.is_empty() {
            Ok(requested.to_vec())
        } else {
            Err(format!(
                "--cards 点了服务交来的卡之外的序号 {outside:?}（这一台交来的是 {handed_over:?}）：只许在交来的卡里挑（运行条件不满足，退 78）。出路：--cards 只写交来的卡，或整条重交、让服务交够这几张"
            ))
        }
    }

    /// 一处先录后判：过库那一步交回的结局定取不取卡。只录那一趟交回 `None`、`take_the_cards` 一次都不调；判段读回输入与参照之后才调它，
    /// 交回读回的那一份与取卡的结局。
    ///
    /// # Panics
    /// 过库交回只判了这一台那一份：对拍输入过库不分片，验证包 `pairing_input_and_reference_through_the_pairing_library` 只交只录与判全两种。
    pub(super) fn read_back_before_the_cards<Cards>(
        outcome: StageOutcome<PairingInputAndReferenceReadBack>,
        take_the_cards: impl FnOnce() -> Cards,
    ) -> Option<(PairingInputAndReferenceReadBack, Cards)> {
        match outcome {
            StageOutcome::RecordedOnly => None,
            StageOutcome::Judged(read_back) => Some((read_back, take_the_cards())),
            StageOutcome::JudgedMyShareOnly {
                blocks_judged_here,
                blocks_left_to_other_hosts,
            } => panic!(
                "对拍输入过库不分片，不该交回只判了这一台那一份（这台 {blocks_judged_here} 块、别的台 {blocks_left_to_other_hosts} 块）"
            ),
        }
    }

    /// 参照字节里依次 4 字节小端的 CRC-32C（[`cpu_reference_of_the_submission_unit`]、[`cpu_reference_of_the_slots`] 写的）。
    ///
    /// # Panics
    /// 字节数不是 4 的倍数。
    pub(super) fn crcs_of_the_reference_bytes(bytes: &[u8]) -> Vec<u32> {
        assert!(
            bytes.len().is_multiple_of(4),
            "参照字节是一串 4 字节小端的 CRC-32C，长度 {} 不是 4 的倍数",
            bytes.len()
        );
        bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|crc| u32::from_le_bytes(*crc))
            .collect()
    }

    /// gpu-submission 从库里读回的：数据单元的字节，与每种段长（[`SUBMISSION_RANGE_LENGTHS`] 的次序）的 CPU 参照 CRC。
    pub(super) struct SubmissionUnitReadBack {
        pub(super) unit: Vec<u8>,
        pub(super) cpu_crcs_by_range_length: Vec<u32>,
    }

    /// gpu-submission 过库那一步（[`read_back_before_the_cards`]）：项名 [`SUBMISSION_PAIRING_ITEM`]，输入是那一个数据单元，
    /// 参照是 [`cpu_reference_of_the_submission_unit`]。`through_the_library` 的产品路是 [`input_and_reference_through_the_pairing_library`]。
    ///
    /// # Panics
    /// 读回的参照不是每种段长一个 CRC。
    #[allow(
        clippy::type_complexity,
        reason = "过库那一步写成闭包参数：产品路交验证包的过库函数，单测交假的"
    )]
    pub(super) fn submission_unit_read_back_before_the_cards<Cards>(
        unit: &[u8],
        through_the_library: impl FnOnce(
            &str,
            &[u8],
            fn(&[u8]) -> Vec<u8>,
        ) -> StageOutcome<PairingInputAndReferenceReadBack>,
        take_the_cards: impl FnOnce() -> Cards,
    ) -> Option<(SubmissionUnitReadBack, Cards)> {
        read_back_before_the_cards(
            through_the_library(
                SUBMISSION_PAIRING_ITEM,
                unit,
                cpu_reference_of_the_submission_unit,
            ),
            take_the_cards,
        )
        .map(|(read_back, cards)| {
            let cpu_crcs_by_range_length = crcs_of_the_reference_bytes(read_back.reference.bytes());
            assert_eq!(
                cpu_crcs_by_range_length.len(),
                SUBMISSION_RANGE_LENGTHS.len(),
                "gpu-submission 读回的参照每种段长一个 CRC"
            );
            (
                SubmissionUnitReadBack {
                    unit: read_back.input.bytes().to_vec(),
                    cpu_crcs_by_range_length,
                },
                cards,
            )
        })
    }

    /// 带卡单测的对拍输入：槽数（8 字节小端）、每个单元的长度（各 4 字节小端）、全部 32 KiB 槽连放的字节。
    ///
    /// # Panics
    /// 槽的字节数不是槽数乘 32 KiB。
    pub(super) fn pairing_input_of_the_slots(slots: &[u8], lengths: &[u32]) -> Vec<u8> {
        let slot_bytes = usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768");
        assert_eq!(
            slots.len(),
            lengths.len() * slot_bytes,
            "每个单元一个 32 KiB 槽"
        );
        let mut input = u64::try_from(lengths.len())
            .expect("槽数装得进 u64")
            .to_le_bytes()
            .to_vec();
        for length in lengths {
            input.extend_from_slice(&length.to_le_bytes());
        }
        input.extend_from_slice(slots);
        input
    }

    /// [`pairing_input_of_the_slots`] 写的字节解回（槽连放的字节，长度表）。
    ///
    /// # Panics
    /// 字节不是它写的样子（短了、长了、长度比槽长）。
    pub(super) fn slots_of_the_pairing_input(input: &[u8]) -> (Vec<u8>, Vec<u32>) {
        let slot_bytes = usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768");
        let (count_bytes, rest) = input
            .split_at_checked(8)
            .expect("对拍输入开头是 8 字节的槽数");
        let slot_count = usize::try_from(u64::from_le_bytes(
            count_bytes
                .try_into()
                .expect("split_at_checked(8) 前一半 8 字节"),
        ))
        .expect("槽数装得进 usize");
        let (length_bytes, slots) = rest
            .split_at_checked(slot_count * 4)
            .expect("槽数之后是每个单元 4 字节的长度");
        assert_eq!(
            slots.len(),
            slot_count * slot_bytes,
            "长度表之后是每个单元一个 32 KiB 槽"
        );
        let lengths: Vec<u32> = length_bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|length| u32::from_le_bytes(*length))
            .collect();
        assert!(
            lengths.iter().all(|length| usize::try_from(*length)
                .is_ok_and(|length_in_bytes| length_in_bytes <= slot_bytes)),
            "每个单元的长度不超过一个槽"
        );
        (slots.to_vec(), lengths)
    }

    /// 带卡单测从库里读回的：槽连放的字节、长度表、每个单元的 CPU 参照 CRC。
    pub(super) struct SlotsReadBack {
        pub(super) slots: Vec<u8>,
        pub(super) lengths: Vec<u32>,
        pub(super) cpu_crcs: Vec<u32>,
    }

    /// 带卡单测过库那一步（[`read_back_before_the_cards`]）：项名 `item_name`，输入是 [`pairing_input_of_the_slots`] 写的那一份，
    /// 参照是 [`cpu_reference_of_the_slots`]。`through_the_library` 的产品路是 [`input_and_reference_through_the_pairing_library`]。
    ///
    /// # Panics
    /// 读回的参照不是每个单元一个 CRC。
    #[allow(
        clippy::type_complexity,
        reason = "过库那一步写成闭包参数：产品路交验证包的过库函数，单测交假的"
    )]
    pub(super) fn slots_read_back_before_the_card<Card>(
        item_name: &str,
        slots: &[u8],
        lengths: &[u32],
        through_the_library: impl FnOnce(
            &str,
            &[u8],
            fn(&[u8]) -> Vec<u8>,
        ) -> StageOutcome<PairingInputAndReferenceReadBack>,
        take_the_card: impl FnOnce() -> Card,
    ) -> Option<(SlotsReadBack, Card)> {
        read_back_before_the_cards(
            through_the_library(
                item_name,
                &pairing_input_of_the_slots(slots, lengths),
                cpu_reference_of_the_slots,
            ),
            take_the_card,
        )
        .map(|(read_back, card)| {
            let (slots_read_back, lengths_read_back) =
                slots_of_the_pairing_input(read_back.input.bytes());
            let cpu_crcs = crcs_of_the_reference_bytes(read_back.reference.bytes());
            assert_eq!(
                cpu_crcs.len(),
                lengths_read_back.len(),
                "带卡单测读回的参照每个单元一个 CRC"
            );
            (
                SlotsReadBack {
                    slots: slots_read_back,
                    lengths: lengths_read_back,
                    cpu_crcs,
                },
                card,
            )
        })
    }

    /// 过库的产品路：验证包 `pairing_input_and_reference_through_the_pairing_library`（段由 `SINGLEFS_CRASH_AMPLIFICATION_STAGE` 定，
    /// 键是验证包那一套），参照一侧的判法单元取实验包编进来的那一张键表里 [`super::CPU_REFERENCE_UNITS_GROUP`] 那一组
    /// （`singlefs_implementation_experiments::JUDGING_UNITS_KEYS_TABLE_TEXT`；拒算时照验证包的处置：照录、不凭命中跳过）。
    ///
    /// # Panics
    /// 同验证包 `pairing_input_and_reference_through_the_pairing_library`：项名不合规矩；库开不了、写不进、读不回。
    #[cfg(feature = "gpu")]
    pub(super) fn input_and_reference_through_the_pairing_library(
        item_name: &str,
        input_bytes: &[u8],
        reference_of_the_input: fn(&[u8]) -> Vec<u8>,
    ) -> StageOutcome<PairingInputAndReferenceReadBack> {
        use singlefs_verification::crash_identity::judging_units_key_of_the_group_in_the_table_text;
        use singlefs_verification::crash_library::pairing_input_and_reference_through_the_pairing_library;
        pairing_input_and_reference_through_the_pairing_library(
            item_name,
            input_bytes,
            &judging_units_key_of_the_group_in_the_table_text(
                singlefs_implementation_experiments::JUDGING_UNITS_KEYS_TABLE_TEXT,
                super::CPU_REFERENCE_UNITS_GROUP,
            ),
            reference_of_the_input,
        )
    }

    /// 编这份二进制的那棵树的仓根（包目录往上两层）：多机配置照它找（同验证包 `GpuConfigurationOfTheCaller::multi_host_configuration_file`）。
    #[cfg(feature = "gpu")]
    const REPOSITORY_ROOT_OF_THE_BUILD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

    /// 开设备前一刻取卡：服务交给这个进程的卡串（[`card_text_handed_over_now`]；环境变量取调用方给这个进程的那一份，
    /// 验证包 `GpuConfigurationOfTheCaller::of_this_process`）里这一台的那几张，`--cards` 点名的照 [`cards_picked_among_those_handed_over`] 挑。
    ///
    /// # Errors
    /// 多机配置找不到；没交卡串；卡串读不成（写法不对、地址不在 `HOSTS` 里、判法认不出这一台）；`--cards` 点到交来的卡之外。
    #[cfg(feature = "gpu")]
    pub(super) fn card_indices_handed_over_to_the_segment(
        requested: &[u32],
    ) -> Result<Vec<u32>, String> {
        use singlefs_verification::gpu_card_configuration::{
            multi_host_addresses_of_the_caller, parse_cards_by_priority,
            GpuConfigurationOfTheCaller,
        };
        let caller = GpuConfigurationOfTheCaller::of_this_process();
        let configuration_file =
            caller.multi_host_configuration_file(Path::new(REPOSITORY_ROOT_OF_THE_BUILD))?;
        let text = card_text_handed_over_now(
            caller.cards_handed_out_by_the_service().map(str::to_string),
            configuration_file.as_deref(),
            GpuQuiesce::of_this_process(),
        )?;
        let handed_over: Vec<u32> =
            parse_cards_by_priority(&text, &multi_host_addresses_of_the_caller()?)?
                .cards_of_this_host()
                .into_iter()
                .map(|(index, _memory_quota_mebibytes)| index)
                .collect();
        cards_picked_among_those_handed_over(requested, &handed_over)
    }

    /// 取卡没成（[`card_indices_handed_over_to_the_segment`] 的错）：原话打到标准错误，产物打一行 `name=verdict … status=not_run reason=cards_not_handed_over`、
    /// 收尾，交回退出码 78（运行条件不满足）。
    #[cfg(feature = "gpu")]
    pub(super) fn stop_without_the_cards_handed_over(
        segment: &str,
        refusal: &str,
        emitter: &mut super::Emitter,
    ) -> i32 {
        eprintln!("{segment}：取卡没成：{refusal}");
        emitter.line(&format!(
            "name=verdict segment={segment} status=not_run reason=cards_not_handed_over detail={}",
            super::field_token(refusal)
        ));
        emitter.finish();
        super::EXIT_CODE_NOT_RUN
    }
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

// ───────────────────────────── 第 4 次重跑：CPU 模式的四种写法（登记 research/prompts/e185-r4-prereg.md 第一节第 1 行「读法写死」；只给计时用，不改 crates/） ─────────────────────────────

mod cpu_writings;

use cpu_writings::*;

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

/// 每张卡起设备时给的显存额度（MiB，登记第五节 `gpu_cards_by_priority(&[(序号, 4096)])`）：起卡的几段共用这一份。
#[cfg(feature = "gpu")]
const CARD_QUOTA_MEBIBYTES: u64 = 4096;

#[cfg(feature = "gpu")]
mod gpu_submission;

// ───────────────────────────── host-cpu-per-submission 的算术（第 3 次重跑登记 `research/prompts/e185-r3-prereg.md` 第一、五、六节；纯函数，不编 gpu 特性也有单测） ─────────────────────────────

/// 两段 GPU 子命令（第 3 次重跑第一、二段）的纯算术与判定行：不编 gpu 特性时只有单测用它们。
mod gpu_segment_arithmetic;

use gpu_segment_arithmetic::*;

// ───────────────────────────── 第 4 次重跑控漂移段的算术（登记 research/prompts/e185-r4-prereg.md 第一节第 2 行、第五节「控漂移段怎么量」、第六节 D1–D7；不用卡，单测喂假钟） ─────────────────────────────

mod drift_control_arithmetic;

#[cfg_attr(
    not(feature = "gpu"),
    allow(
        unused_imports,
        reason = "控漂移段（gpu 特性）才从顶层用它；没编 gpu 特性时只有单测用"
    )
)]
use drift_control_arithmetic::*;

// ───────────────────────────── 常驻在一张卡上的两份核（第 3 次重跑登记第一节「核一、核二」「常驻写法」；只在编了 gpu 特性时有） ─────────────────────────────

#[cfg(feature = "gpu")]
mod resident_gpu_kernels;

// ───────────────────────────── 第 3 次重跑第一段 host-cpu-per-submission（登记第五节「第一段怎么量」；只在编了 gpu 特性时有） ─────────────────────────────

/// 第一段装置自己不起任何线程（起卡之后驱动有自己的线程，不归装置管，它们的 CPU 由进程钟单报）；计时区间之外每格前后报本进程线程数。
/// 单测 `the_host_cpu_segment_starts_no_thread_of_its_own` 核这个模块的源码里没有起线程的写法。
#[cfg(feature = "gpu")]
mod host_cpu_per_submission_segment;

// ───────────────────────────── 第 3 次重跑第二段 recompute-disagreement（登记第五节「第二段怎么量」；只在编了 gpu 特性时有） ─────────────────────────────

#[cfg(feature = "gpu")]
mod recompute_disagreement_segment;

// ───────────────────────────── 第 4 次重跑控漂移段 host-cpu-drift-control（登记 research/prompts/e185-r4-prereg.md 第五节「控漂移段怎么量」；只在编了 gpu 特性时有） ─────────────────────────────

#[cfg(feature = "gpu")]
mod host_cpu_drift_control_segment;

// ───────────────────────────── 准入、命令行、入口 ─────────────────────────────

// ───────────────────────────── 停机 7：装置自己的 sha256（第 2 次重跑登记第十一节停机 7） ─────────────────────────────

/// 编进这个二进制的装置源码：每段开头打出它的 sha256，与输入清单第一行、登记第十二节写的那一个对拍（停机 7）。
const OWN_SOURCE_TEXT: &str = include_str!("main.rs");
/// 设了就与编进来的源码 sha256 比（起跑脚本把登记第十二节写的那一个设进来），不同就停、退 3。
const REGISTERED_APPARATUS_SHA256_ENVIRONMENT_VARIABLE: &str = "E185_REGISTERED_APPARATUS_SHA256";

fn own_source_sha256() -> String {
    hexadecimal_of(&sha256_digest(OWN_SOURCE_TEXT.as_bytes()))
}

/// 停机 7 的判法：登记的值设了而与编进来的源码 sha256 不同（不分大小写），交回登记的那一个；没设、相同交回 `None`。
fn registered_apparatus_sha256_mismatch(registered: Option<&str>, own: &str) -> Option<String> {
    let registered = registered?.trim().to_ascii_lowercase();
    (registered != own).then_some(registered)
}

/// 每段开头打一行 `name=apparatus`；登记的 sha256 设了而对不上就打 `name=stop_clause clause=7`、收尾，交回 `false`（调用方退 3）。
fn check_the_apparatus_sha256(emitter: &mut Emitter) -> bool {
    let own = own_source_sha256();
    let registered = std::env::var(REGISTERED_APPARATUS_SHA256_ENVIRONMENT_VARIABLE).ok();
    emitter.line(&format!(
        "name=apparatus source_sha256={own} registered_sha256={}",
        registered
            .as_deref()
            .map_or_else(|| "not_set".to_string(), field_token)
    ));
    if let Some(registered_value) =
        registered_apparatus_sha256_mismatch(registered.as_deref(), &own)
    {
        emitter.line(&format!(
            "name=stop_clause clause=7 registered_sha256={} source_sha256={own}",
            field_token(&registered_value)
        ));
        emitter.finish();
        return false;
    }
    true
}

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；强制跑（`--force`）时交回摘要，`main` 把它作为头一行打到标准输出（同 E183 装置的写法）。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e185_gc_batch_wall_clock_split/main.rs");
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
    eprintln!("  e185_gc_batch_wall_clock_split geometry-check [--point <0.25|0.75>:<1024|8192>]… [--overwrite-order slot-order]");
    eprintln!("  e185_gc_batch_wall_clock_split host-cpu-per-submission --cards <序号,…> --stage-split-product <文件> --attack-ledger <文件>");
    eprintln!("  e185_gc_batch_wall_clock_split recompute-disagreement --cards <序号,…>");
    eprintln!("  e185_gc_batch_wall_clock_split function-split [--point <0.25|0.75>:<1024|8192>]…");
    eprintln!("  e185_gc_batch_wall_clock_split host-cpu-drift-control --cards <序号,…> --stage-split-product <文件> --attack-ledger <文件>");
    eprintln!("  e185_gc_batch_wall_clock_split cpu-writings [--point <0.25|0.75>:<1024|8192>]…");
}

/// `--cards 0,1` 的序号表；写错交回 `None`。
fn cards_of(options: &BTreeMap<String, Vec<String>>) -> Option<Vec<u32>> {
    single_option(options, "cards").and_then(|text| {
        text.split(',')
            .map(|index| index.trim().parse::<u32>().ok())
            .collect()
    })
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
        "geometry-check" if known(&["point", "overwrite-order"]) => {
            let overwrite_order = match options.get("overwrite-order") {
                None => Some(OverwriteOrder::HighestFullSegmentLast),
                Some(_) => {
                    single_option(&options, "overwrite-order").and_then(OverwriteOrder::of_argument)
                }
            };
            match (points_of(&options), overwrite_order) {
                (Some(points), Some(overwrite_order)) => {
                    run_the_geometry_check(&points, overwrite_order)
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
        "host-cpu-per-submission" if known(&["cards", "stage-split-product", "attack-ledger"]) => {
            match (
                cards_of(&options),
                single_option(&options, "stage-split-product"),
                single_option(&options, "attack-ledger"),
            ) {
                (Some(card_indices), Some(stage_split_product), Some(attack_ledger)) => {
                    run_the_host_cpu_per_submission_segment(
                        Path::new(stage_split_product),
                        Path::new(attack_ledger),
                        &card_indices,
                    )
                }
                (None, _, _) | (_, None, _) | (_, _, None) => {
                    usage();
                    EXIT_CODE_USAGE
                }
            }
        }
        "recompute-disagreement" if known(&["cards"]) => match cards_of(&options) {
            Some(card_indices) => run_the_recompute_disagreement_segment(&card_indices),
            None => {
                usage();
                EXIT_CODE_USAGE
            }
        },
        "host-cpu-drift-control" if known(&["cards", "stage-split-product", "attack-ledger"]) => {
            match (
                cards_of(&options),
                single_option(&options, "stage-split-product"),
                single_option(&options, "attack-ledger"),
            ) {
                (Some(card_indices), Some(stage_split_product), Some(attack_ledger)) => {
                    run_the_host_cpu_drift_control_segment(
                        Path::new(stage_split_product),
                        Path::new(attack_ledger),
                        &card_indices,
                    )
                }
                (None, _, _) | (_, None, _) | (_, _, None) => {
                    usage();
                    EXIT_CODE_USAGE
                }
            }
        }
        "cpu-writings" if known(&["point"]) => match points_of(&options) {
            Some(points) => run_the_cpu_writings_segment(&points),
            None => {
                usage();
                EXIT_CODE_USAGE
            }
        },
        "function-split" if known(&["point"]) => match points_of(&options) {
            Some(points) => run_the_function_split_segment(&points),
            None => {
                usage();
                EXIT_CODE_USAGE
            }
        },
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
    gpu_submission::measure_gpu_submission_segment(
        wall_clock_product,
        stage_split_product,
        card_indices,
    )
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
    eprintln!("本次未跑：没编 gpu 特性（带 --features gpu 编这个 bin，整条经 task-server/task submit 交）");
    EXIT_CODE_NOT_RUN
}

#[cfg(feature = "gpu")]
fn run_the_host_cpu_per_submission_segment(
    stage_split_product: &Path,
    attack_ledger: &Path,
    card_indices: &[u32],
) -> i32 {
    host_cpu_per_submission_segment::measure_host_cpu_per_submission_segment(
        stage_split_product,
        attack_ledger,
        card_indices,
    )
}

#[cfg(feature = "gpu")]
fn run_the_recompute_disagreement_segment(card_indices: &[u32]) -> i32 {
    recompute_disagreement_segment::measure_recompute_disagreement_segment(card_indices)
}

#[cfg(feature = "gpu")]
fn run_the_host_cpu_drift_control_segment(
    stage_split_product: &Path,
    attack_ledger: &Path,
    card_indices: &[u32],
) -> i32 {
    host_cpu_drift_control_segment::measure_host_cpu_drift_control_segment(
        stage_split_product,
        attack_ledger,
        card_indices,
    )
}

/// 没编 `gpu` 特性：第 3 次重跑的两段 GPU 子命令本次未跑（退 78），同原 `gpu-submission`。
#[cfg(not(feature = "gpu"))]
fn gpu_segment_not_compiled(segment: &str, card_indices: &[u32]) -> i32 {
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=verdict segment={segment} status=not_run reason=gpu_feature_not_compiled cards={}",
        card_indices
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    ));
    emitter.finish();
    eprintln!("本次未跑：没编 gpu 特性（带 --features gpu 编这个 bin，整条经 task-server/task submit 交）");
    EXIT_CODE_NOT_RUN
}

#[cfg(not(feature = "gpu"))]
fn run_the_host_cpu_per_submission_segment(
    _stage_split_product: &Path,
    _attack_ledger: &Path,
    card_indices: &[u32],
) -> i32 {
    gpu_segment_not_compiled("host_cpu_per_submission", card_indices)
}

#[cfg(not(feature = "gpu"))]
fn run_the_recompute_disagreement_segment(card_indices: &[u32]) -> i32 {
    gpu_segment_not_compiled("recompute_disagreement", card_indices)
}

#[cfg(not(feature = "gpu"))]
fn run_the_host_cpu_drift_control_segment(
    _stage_split_product: &Path,
    _attack_ledger: &Path,
    card_indices: &[u32],
) -> i32 {
    gpu_segment_not_compiled("host_cpu_drift_control", card_indices)
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
    fn gpu_input_and_buffer_bytes_match_the_preregistered_anchors() {
        assert_eq!(1024 * LOCAL_GPU_SLOT_BYTES, 33_554_432);
        assert_eq!(local_gpu_buffer_bytes(1), 32_804);
        assert_eq!(local_gpu_buffer_bytes(1024), 33_574_928);
        assert_eq!(local_gpu_buffer_bytes(4096), 134_299_664);
        assert_eq!(local_gpu_buffer_bytes(16_384), 537_198_608);
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

    /// `geometry-check --overwrite-order` 只认 `slot-order`；认不得的值退 2（不起造池）。
    #[test]
    fn geometry_check_takes_only_slot_order_for_the_overwrite_order() {
        assert_eq!(
            OverwriteOrder::of_argument("slot-order"),
            Some(OverwriteOrder::SlotOrder)
        );
        assert_eq!(
            OverwriteOrder::of_argument("highest-full-segment-last"),
            None
        );
        assert_eq!(
            run_the_subcommand(&["geometry-check", "--overwrite-order", "alpha"]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_the_subcommand(&["geometry-check", "--point", "0.5:1024"]),
            EXIT_CODE_USAGE
        );
    }

    /// 停机 7：登记的 sha256 设了而与编进来的源码不同才停（不分大小写）；没设不停；编进来的是 64 位十六进制。
    #[test]
    fn apparatus_sha256_stops_only_when_the_registered_value_differs() {
        let own = own_source_sha256();
        assert_eq!(own.len(), 64);
        assert_eq!(registered_apparatus_sha256_mismatch(None, &own), None);
        assert_eq!(
            registered_apparatus_sha256_mismatch(Some(own.to_ascii_uppercase().as_str()), &own),
            None
        );
        assert_eq!(
            registered_apparatus_sha256_mismatch(Some("eadb929d"), &own),
            Some("eadb929d".to_string())
        );
    }

    #[test]
    fn the_host_cpu_segment_starts_no_thread_of_its_own() {
        let segment = include_str!("host_cpu_per_submission_segment.rs");
        for spawning in ["thread::spawn", "thread::scope", "thread::Builder"] {
            assert!(!segment.contains(spawning), "第一段里有 {spawning}");
        }
        assert!(
            segment.contains("fn measure_the_cells_of_the_card"),
            "量的那一段在这个模块里"
        );
    }

    #[test]
    fn the_three_new_subcommands_refuse_missing_and_unknown_options() {
        assert_eq!(
            run_the_subcommand(&["host-cpu-per-submission", "--cards", "0,1"]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_the_subcommand(&["recompute-disagreement", "--cards", "zero"]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_the_subcommand(&["recompute-disagreement", "--card", "0"]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_the_subcommand(&["function-split", "--point", "0.5:1024"]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_the_subcommand(&["function-split", "--batch-units", "8"]),
            EXIT_CODE_USAGE
        );
    }

    // ───────────── 要卡的子命令取卡、先录后判过库（不要真卡、不开真库：过库是用例给的假的） ─────────────

    /// 过库那一步判段交回的样子：输入与参照照原样当成从库里读回的（摘要照它们现算）。
    fn judged_with_the_input_and_reference_read_back(
        input: &[u8],
        reference: &[u8],
    ) -> singlefs_verification::crash_library::StageOutcome<
        singlefs_verification::crash_library::PairingInputAndReferenceReadBack,
    > {
        use singlefs_verification::crash_library::{
            PairingInputAndReferenceReadBack, PairingInputReadBack, StageOutcome,
        };
        StageOutcome::Judged(PairingInputAndReferenceReadBack {
            input: PairingInputReadBack::from_the_library_bytes(
                input.to_vec(),
                &sha256_digest(input),
            )
            .expect("摘要照字节现算，对得上"),
            reference: PairingInputReadBack::from_the_library_bytes(
                reference.to_vec(),
                &sha256_digest(reference),
            )
            .expect("摘要照字节现算，对得上"),
        })
    }

    /// 一个 32 KiB 的数据单元（字节按下标取模，两种段长的开头那一截 CRC 不同）。
    fn submission_unit_of_the_unit_tests() -> Vec<u8> {
        (0..u32::try_from(LOCAL_DATA_UNIT_BYTES).expect("32768"))
            .map(|offset| u8::try_from(offset % 251).expect("取模 251 装得进 u8"))
            .collect()
    }

    /// 环境里有服务交的卡串：照它（与 `GPU_CARDS_BY_PRIORITY` 同格式，原样交）。
    #[test]
    fn the_card_text_in_the_environment_is_used_as_handed_over() {
        assert_eq!(
            cards_and_pairing_of_the_gpu_segments::card_text_handed_over_now(
                Some("192.0.2.93:4:2M".to_string()),
                None,
                singlefs_verification::gpu_card_configuration::GpuQuiesce::NotQuiesced,
            ),
            Ok("192.0.2.93:4:2M".to_string()),
            "环境里有卡串：照它"
        );
    }

    /// 环境里没有卡串：报没经服务给卡那一句（多机配置没给），不借卡、不交任何卡串（不退回本机枚举）。
    /// 判别力（`crates/mutations.tsv`「E185 V240 没经服务给卡也交一个空卡串往下走」）：交回 `Ok` 的空串。
    #[test]
    fn without_a_card_text_the_refusal_is_the_no_cards_handed_over_sentence() {
        assert_eq!(
            cards_and_pairing_of_the_gpu_segments::card_text_handed_over_now(
                None,
                None,
                singlefs_verification::gpu_card_configuration::GpuQuiesce::NotQuiesced,
            ),
            Err(cards_and_pairing_of_the_gpu_segments::NO_CARDS_HANDED_OVER_SENTENCE.to_string()),
        );
    }

    /// 环境里的卡串是空串或只有空白：与没有卡串同样报没经服务给卡那一句，不当「本机能用的卡全用」往下走。
    /// 判别力（`crates/mutations.tsv`「E185 V248 环境里的卡串是空白也当交了卡」）：空白卡串原样交回 `Ok`，这条红。
    #[test]
    fn a_blank_card_text_in_the_environment_is_refused_like_no_card_text() {
        for blank in ["", "  "] {
            assert_eq!(
                cards_and_pairing_of_the_gpu_segments::card_text_handed_over_now(
                    Some(blank.to_string()),
                    None,
                    singlefs_verification::gpu_card_configuration::GpuQuiesce::NotQuiesced,
                ),
                Err(
                    cards_and_pairing_of_the_gpu_segments::NO_CARDS_HANDED_OVER_SENTENCE
                        .to_string()
                ),
                "卡串 {blank:?}：当没交卡串"
            );
        }
    }

    /// 场景：扫这份装置源码里测试模块之前的那一段（顶格的 `#[cfg(test)]` 与 `mod tests {` 两行之前；测试模块在文件末尾）。
    /// 预期：验证包借卡客户端的几个入口一处都不出现：这个装置不在 GPU 批处理验证服务的借卡白名单里，要卡的子命令只照服务交的卡串取卡
    /// （`cards_and_pairing_of_the_gpu_segments::card_text_handed_over_now`）。不要卡，不起装置。
    /// 判别力（`crates/mutations.tsv`「E185 V249 没交卡串时经借卡通道借」）：取卡改回经借卡通道借，那一段出现借卡入口，这条红。
    #[test]
    fn the_apparatus_outside_its_tests_module_calls_no_card_lease_entry() {
        let (outside_the_tests_module, _tests_module) = include_str!("main.rs")
            .split_once("\n#[cfg(test)]\nmod tests {\n")
            .expect("装置源码里有顶格的测试模块头两行");
        for lease_entry in [
            "cards_leased_by_this_process",
            "lease_cards_through_the_client",
            "cards_handed_over_or_leased_on_demand",
            "CardLeaseBorrower",
        ] {
            assert!(
                !outside_the_tests_module.contains(lease_entry),
                "装置测试模块之前的那一段出现了借卡客户端的入口 {lease_entry}：这个装置不借卡"
            );
        }
    }

    /// `--cards` 只在交来的卡里按给的次序挑：都在交来的卡里的照给的次序交回；点到交来的卡之外的报错，点名那几个序号与交来的那几张。
    /// 判别力（`crates/mutations.tsv`「E185 V242 --cards 点到交来的卡之外也照收」）：点了 0 而交来的是 3、1 照样交回。
    #[test]
    fn cards_outside_those_handed_over_are_refused_and_those_inside_are_picked_in_the_given_order()
    {
        assert_eq!(
            cards_and_pairing_of_the_gpu_segments::cards_picked_among_those_handed_over(
                &[1, 3],
                &[3, 1]
            ),
            Ok(vec![1, 3]),
            "都在交来的卡里：照 --cards 给的次序"
        );
        let refusal = cards_and_pairing_of_the_gpu_segments::cards_picked_among_those_handed_over(
            &[1, 0],
            &[3, 1],
        )
        .expect_err("点了 0、交来的是 3 与 1：报错");
        assert!(
            refusal.contains("[0]") && refusal.contains("[3, 1]"),
            "报错点名交来的卡之外的序号与交来的那几张：{refusal}"
        );
    }

    /// gpu-submission 只录那一趟：数据单元与参照交给过库那一步，项名是 gpu-submission 那一项；过库交回只录，不取卡、交回 `None`。
    /// 判别力（`crates/mutations.tsv`「E185 V243 只录那一趟也取卡」）：只录那一趟取卡的次数是 1。
    #[test]
    fn the_submission_unit_is_recorded_without_taking_a_card_when_the_stage_records_only() {
        use singlefs_verification::crash_library::StageOutcome;
        let unit = submission_unit_of_the_unit_tests();
        let items_through_the_library = std::cell::RefCell::new(Vec::new());
        let cards_taken = std::cell::Cell::new(0_u32);
        let read_back =
            cards_and_pairing_of_the_gpu_segments::submission_unit_read_back_before_the_cards(
                &unit,
                |item_name, input_bytes, reference_of_the_input| {
                    items_through_the_library.borrow_mut().push((
                        item_name.to_string(),
                        input_bytes.to_vec(),
                        reference_of_the_input(input_bytes),
                    ));
                    StageOutcome::RecordedOnly
                },
                || cards_taken.set(cards_taken.get() + 1),
            );
        assert!(read_back.is_none(), "只录那一趟交回 None");
        assert_eq!(cards_taken.get(), 0, "只录那一趟不取卡");
        assert_eq!(
            items_through_the_library.into_inner(),
            vec![(
                cards_and_pairing_of_the_gpu_segments::SUBMISSION_PAIRING_ITEM.to_string(),
                unit.clone(),
                cpu_reference_of_the_submission_unit(&unit),
            )],
            "过库一回：gpu-submission 那一项、数据单元原样、参照是两种段长的 CRC"
        );
    }

    /// gpu-submission 判段：读回之后才取卡（取一回），交回读回的数据单元与每种段长的 CPU 参照 CRC（32 KiB 整段、16 KiB 开头那一截）。
    /// 判别力（`crates/mutations.tsv`「E185 V245 gpu-submission 的参照两种段长都按整个数据单元算」）：16 KiB 那一种的参照不等于开头那一截的 CRC。
    #[test]
    fn the_submission_unit_and_its_cpu_reference_are_read_back_before_the_card_is_taken() {
        let unit = submission_unit_of_the_unit_tests();
        let cards_taken = std::cell::Cell::new(0_u32);
        let (read_back, ()) =
            cards_and_pairing_of_the_gpu_segments::submission_unit_read_back_before_the_cards(
                &unit,
                |_item_name, input_bytes, reference_of_the_input| {
                    judged_with_the_input_and_reference_read_back(
                        input_bytes,
                        &reference_of_the_input(input_bytes),
                    )
                },
                || cards_taken.set(cards_taken.get() + 1),
            )
            .expect("判段交回读回的那一份");
        assert_eq!(cards_taken.get(), 1, "判段读回之后取一回卡");
        assert_eq!(read_back.unit, unit, "读回的数据单元原样");
        assert_eq!(
            read_back.cpu_crcs_by_range_length,
            vec![crc32_castagnoli(&unit), crc32_castagnoli(&unit[..16_384])],
            "每种段长一个参照：32 KiB 整段、16 KiB 开头那一截"
        );
    }

    /// 带卡单测只录那一趟：槽与长度表编成对拍输入交给过库那一步，过库交回只录，不取卡、交回 `None`。
    /// 判别力（`crates/mutations.tsv`「E185 V244 带卡单测只录那一趟也取卡」）：只录那一趟取卡的次数是 1。
    #[test]
    fn the_seeded_slots_are_recorded_without_taking_a_card_when_the_stage_records_only() {
        use singlefs_verification::crash_library::StageOutcome;
        let (slots, lengths) = seeded_slots(4);
        let cards_taken = std::cell::Cell::new(0_u32);
        let inputs_through_the_library = std::cell::RefCell::new(Vec::new());
        let read_back = cards_and_pairing_of_the_gpu_segments::slots_read_back_before_the_card(
            &format!("e185_gc_batch_wall_clock_split/{SEEDED_UNITS_TEST_NAME}"),
            &slots,
            &lengths,
            |_item_name, input_bytes, _reference_of_the_input| {
                inputs_through_the_library
                    .borrow_mut()
                    .push(input_bytes.to_vec());
                StageOutcome::RecordedOnly
            },
            || cards_taken.set(cards_taken.get() + 1),
        );
        assert!(read_back.is_none(), "只录那一趟交回 None");
        assert_eq!(cards_taken.get(), 0, "只录那一趟不取卡");
        assert_eq!(
            inputs_through_the_library.into_inner(),
            vec![
                cards_and_pairing_of_the_gpu_segments::pairing_input_of_the_slots(&slots, &lengths)
            ],
            "过库一回，输入是槽与长度表编成的那一份"
        );
    }

    /// 带卡单测判段：读回之后才取卡（取一回），交回读回的槽、长度表与每个单元在自己长度那一截上的 CPU 参照 CRC。
    /// 判别力（`crates/mutations.tsv`「E185 V246 带卡单测的参照按整个槽算」）：奇数号单元（16 KiB）的参照不等于它那一截的 CRC。
    #[test]
    fn the_seeded_slots_and_their_cpu_reference_are_read_back_before_the_card_is_taken() {
        let (slots, lengths) = seeded_slots(4);
        let cards_taken = std::cell::Cell::new(0_u32);
        let (read_back, ()) =
            cards_and_pairing_of_the_gpu_segments::slots_read_back_before_the_card(
                &format!("e185_gc_batch_wall_clock_split/{SEEDED_UNITS_TEST_NAME}"),
                &slots,
                &lengths,
                |_item_name, input_bytes, reference_of_the_input| {
                    judged_with_the_input_and_reference_read_back(
                        input_bytes,
                        &reference_of_the_input(input_bytes),
                    )
                },
                || cards_taken.set(cards_taken.get() + 1),
            )
            .expect("判段交回读回的那一份");
        assert_eq!(cards_taken.get(), 1, "判段读回之后取一回卡");
        assert_eq!(read_back.slots, slots, "读回的槽原样");
        assert_eq!(read_back.lengths, lengths, "读回的长度表原样");
        let slot_bytes = usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768");
        let expected: Vec<u32> = slots
            .chunks(slot_bytes)
            .zip(&lengths)
            .map(|(slot, length)| {
                crc32_castagnoli(&slot[..usize::try_from(*length).expect("长度装得进 usize")])
            })
            .collect();
        assert_eq!(
            read_back.cpu_crcs, expected,
            "每个单元在自己长度那一截上的 CRC"
        );
    }

    #[test]
    fn the_drift_control_segment_starts_no_thread_of_its_own() {
        let segment = include_str!("host_cpu_drift_control_segment.rs");
        for spawning in ["thread::spawn", "thread::scope", "thread::Builder"] {
            assert!(!segment.contains(spawning), "控漂移段里有 {spawning}");
        }
        assert!(
            segment.contains("fn measure_the_arm"),
            "量的那一段在这个模块里"
        );
    }

    #[test]
    fn the_drift_control_and_cpu_writings_subcommands_refuse_missing_and_unknown_options() {
        assert_eq!(
            run_the_subcommand(&["host-cpu-drift-control", "--cards", "0,1"]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_the_subcommand(&[
                "host-cpu-drift-control",
                "--cards",
                "zero",
                "--stage-split-product",
                "a",
                "--attack-ledger",
                "b"
            ]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_the_subcommand(&["cpu-writings", "--point", "0.5:1024"]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_the_subcommand(&["cpu-writings", "--cards", "0"]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_the_subcommand(&["cpu-writings-and-function-partition"]),
            EXIT_CODE_USAGE
        );
    }

    #[cfg(not(feature = "gpu"))]
    #[test]
    fn drift_control_without_the_gpu_feature_reports_not_run() {
        assert_eq!(
            run_the_subcommand(&[
                "host-cpu-drift-control",
                "--cards",
                "0,1",
                "--stage-split-product",
                "a",
                "--attack-ledger",
                "b"
            ]),
            EXIT_CODE_NOT_RUN
        );
    }
}
