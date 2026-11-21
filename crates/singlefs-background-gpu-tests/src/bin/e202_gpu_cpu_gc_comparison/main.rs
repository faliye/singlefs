// admission: always 每跑一次都在今天这份 crates 上现造池、现量挂钟与 CPU，计时每次都是新观测，上一次跑完不替这一次作保
// task-server-gpu-arguments: lockstep concurrent sensitivity stage-three-cost
// task-server-resumable: lockstep concurrent sensitivity
// task-server-gpu-cards: all
//! E202（GPU与CPU两种模式GC对照组）的装置：跑前登记 `research/prompts/e202-preregistration.md`。
//!
//! 同一负载、同一池，核心层后台入口 `MountedSession::run_compaction_in_the_background` 在 `background_compute=cpu` 与 `background_compute=gpu`
//! 两种挂载参数下各跑，量 GC 挂钟（量 W）、GC 调用窗口里本进程用掉的主机 CPU（进程口径，量 C）、两者成对相减省下的（量 S）、省下的随盘容量的走向
//! （量 T，`judge-trend` 读三档产物判）与阳性对照（量 P：每条臂配一条把每个确认的条目在 CPU 上再判一遍的多判臂）；并发跑另量前台 fsync 延迟（量 F）。
//!
//! 子命令（登记第十四节「起法」）：
//! - `geometry-check --product <文件>`：三档各一条线程造起步镜像、核几何（登记第五节「负载」第 4 条），每档另在 CPU 臂上不计时地交替跑一次、
//!   按第六节「前提 N」的阶梯数出每次跑的批数；不要卡。
//! - `lockstep --reading occupancy --unit-area <2288MiB|4GiB|8GiB> [--user-changes <次数>] [--batch-units <个数>] [--cards <张数>] --product <文件>`：交替跑六条臂；
//!   `--cards` 只管「全部卡」那两条臂开几张（见下面「全部卡」臂那一段），`--arms` 里没点这两条臂之一的给了算用法错。别的子命令都不认 `--cards`。
//! - `concurrent --reading occupancy --unit-area <…> --foreground-interval-milliseconds <毫秒> --product <文件>`：并发跑三条臂。
//! - `sensitivity --batch-units 16 --foreground-interval-milliseconds 500 --product <文件>`：中档几何敏感性两点（登记第八节）。
//! - `judge-trend --product <文件> <三档 lockstep 产物…>`：量 T 的分类与停机条款第 2 条（三档的 crates 版本相同）。
//! - `stage-three-cost --unit-area <2288MiB|4GiB|8GiB> --product <文件>`：丙段代价（登记第十二节修订第 9 条，只供用户定岔路、不进判据）：
//!   几何核同一份起步镜像上 CPU 臂与 GPU 点名一张卡臂各数五遍每段活槽数取中位，再各照产品 greedy 跑一次 GC 调用、报挂钟与其中丙段那一截。
//! - `construction-timing --unit-area <…> --live-data-units <P,…> --chunks <each-device-copies|shared> --product <文件>`：造池分阶段计时
//!   （盘取那一档的大小，P 取给的几个值，照同一套造法按给的次序串着造，每个 P 一行 `name=construction_timing`，每阶段一行 `name=construction_phase`：
//!   挂钟、步数、每步挂钟的头尾八分之一平均、版本大小、常驻内存、内存盘块字节、这次挂载开过的聚簇段数与单元区段数）；不跑 GC、不要卡。
//!   出的是登记第十二节修订第 14 条的量 B（造成、池形、用时与内存、聚簇段）：造池里一次发布被核心层拒打 `name=construction_refused`、
//!   发布的形状与登记的算式不符、或几何核没过，都打 `name=stopped` 退 3（修订第 14 条追加的停机条款第 7、8 条），后面的 P 不再造。
//!
//! 产物是标准输出与 `--product` 那份文件上同样的结果行（`E202 name=<种类> 键=值 …`），末行 `name=done emitted=<行数>`；进度打到标准错误。
//! `--product` 那份文件只追加：同一条命令重起时读它里面已有的 `name=repeat_done` 行，跳过这一趟所在的那一组里做完的重复、只补这一组没做完的
//! （登记第十四节「续跑入口」与第十二节修订第 16 条）。
//!
//! 退出码：0 正常；2 命令行不对；3 停机条款触发（登记第十一节「停机」，产物照打）；4 阳性对照没过（产物照打）；
//! 77 一条臂都没跑到（服务交的卡里没有这一台的）；78 准入或运行条件不满足（CPU 钟分辨率自检不过、读不到登记的线程数、写不了产物、
//! 服务既没交卡也没给领卡通道、领卡没成、交来的卡对不上本机的独显）。
//!
//! 卡只用服务交的（[`cards_handed_to_the_apparatus`]）：环境变量 `GPU_CARDS_BY_PRIORITY` 有卡串就只开那几张；没有而有领卡通道时，
//! 起步镜像造完、开设备之前那一刻才经验证包的按需领卡领（`singlefs_verification::gpu_card_configuration::cards_handed_over_or_leased_on_demand`）；
//! 两样都没有报错退出，不自己枚举本机的卡。要卡的四个子命令都先做完只用 CPU 的活（读产物、开跑检查、造起步镜像、丙段代价的装置自数），再取卡。
//! 「全部卡」臂开几张每次跑时由 `lockstep --cards <张数>` 设：给了 N 只开服务交来的卡里排在前头的 N 张（先后照服务交来的卡串），
//! 不给开交来的全部；`background_gpu_devices` 点名开的那几张，本机没交的卡不开。N 小于 2（只开一张时全部卡臂与点名一张卡臂是同一条臂）、
//! 不是数、多于交来的张数都退 2（命令行不对），不截成交来的张数；这一台一张都没交的照「服务交的卡里没有这一台的」退 77。
//! 读数依赖的硬件逐趟记进产物、按硬件组判读（登记第十二节修订第 16 条）：每个子命令开头 `name=host_hardware` 记这一台的 CPU 型号与逻辑 CPU 数
//! （读不成退 78）；交替跑与并发跑判完哪几条臂这一趟真跑之后，真跑的用卡的臂开的卡逐张记进产物——点名一张卡臂那一张的地址、型号、驱动、
//! 驱动信息打在 `name=one_card_arm_card` 一行，全部卡臂开了几张、照什么开的与每张的这四样打在 `name=every_card_arm_cards` 一行——再打
//! `name=hardware_group`（[`hardware_group_of`]：CPU 型号、逻辑 CPU 数与这两条臂的卡拼成组键，组标识是它 SHA-256 的前 16 个十六进制字）。
//! 续跑不因换了卡、换了张数、换了驱动或换了一台而停：产物里已有的做完的重复按它那一趟的硬件组归组，这一趟只补自己那一组的
//! （[`plan_of_the_run_after_the_cards`]）；收尾判读与 `judge-trend` 只在同一组之内凑数（各判读行带 `group=`）。唯一的续跑拒绝是产物里有一趟有结果行
//! 而没有 `name=hardware_group` 行（`name=stopped clause=product_session_without_a_hardware_group`，退 2）。丙段代价只记点名一张卡臂那一张、不归组。
//! 这份源文件头的张数声明写 `all`：服务把那台此刻的空卡都交来，装置照 `--cards` 只开其中 N 张；没给 `--cards` 而只交来一张时这条臂与点名一张卡臂相同，报本次未跑。
//! `lockstep`、`concurrent`、`sensitivity` 与 `stage-three-cost` 不拆成先录后判（理由写在各自入口旁）；
//! 只比结果、不量挂钟的那一条内联真卡测试走先录后判：输入与装置自数的参照先在 CPU 上写进判定库，判段读回才取卡（测试包 `common_pairing_input_through_the_library`）。
//!
//! 换不掉的那几处：真卡（GPU 臂经产品 GPU 包开服务交的独显）、真钟（单调钟与 CPU 钟）、`/proc`（机器负载与开卡线程）；
//! 判法与计数写成纯函数，同包测试里喂合成输入。
//! 装置住产品 GPU 的测试包（它同时依赖产品 GPU 包与验证包，按需领卡与判定库都经验证包那一份）：只读、只驱动与观测，不改 `crates/` 下别的文件。
//! 决定几何与判据门槛的常量写成本地常量、开跑先与核心层公开的同一个量回比（[`local_constants_disagreeing_with_the_core`]），对不上走停机条款第 1 条。
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{BufRead, Write};
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex, Weak};
use std::time::{Duration, Instant};

use singlefs_background_gpu::adapter_inventory::{
    gpu_device_identifier_of_the_pci_bus_address, AdapterDeviceKind, BackgroundGpuSelection,
};
use singlefs_background_gpu::background_gpu::{
    host_cpu_time_of_this_thread, monotonic_raw_time_now,
    open_background_gpu_with_the_memory_leaked_in_this_process, OpeningsNotReturnedInThisProcess,
    VulkanInstanceOfThisProcess,
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
use singlefs_core::admission::LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE;
use singlefs_core::allocator::{
    unit_area_slots_of_device, AllocationRecord, AllocationRecordQuarantineMark, DeviceFreeMap,
    PoolAllocator, UnitAreaStart,
};
use singlefs_core::background_compute::{
    admit_gpu_settings_against_the_inventory, AdmittedGpuSettings, BackgroundComputeBackend,
    BackgroundComputeOfOneBatch, BackgroundComputePathTaken, BackgroundComputeSettings,
    BackgroundGpuCompute, BackgroundGpuComputeFailed, BackgroundGpuComputeHandle,
    ClusterSegmentEntryKey, GpuDeviceIdentifier, GpuEntryDecision, GpuInventory, GpuMemoryInBytes,
    GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch, LatestGpuSettingsHandedIn,
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
    region_scans_of_the_segments_without_an_unreleased_record, AnswerOfTheCallerBeforeTheNextBatch,
    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionBatchSize,
    CompactionBatchUnits, CompactionPacing, CompactionRunForcing, CompactionRunStop,
    CompactionSwitches, CompactionTimerFallback, CompactionWriteBandwidthLimit, RegionChoice,
    RegionChoiceCriterion, SegmentsRefusedByTheRegionScan, UnitAreaOfOneDevice,
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
    DeviceShortEvenWithEverythingAFloorRaiseCouldRelease, MountedSession, UserChange,
    UserChangeRefused,
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
use singlefs_verification::crash_library::PairingJudged;
use singlefs_verification::gpu_card_configuration::{
    cards_handed_over_or_leased_on_demand, CardLeaseChannelOfTheCaller,
};

use common_discrete_gpus_of_this_host::{
    handed_over_gpus_or_report_not_run, note_the_cards_leased_on_demand_for_this_process,
    VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER,
};
use common_pairing_input_through_the_library::{
    append_count, append_number, judgement_of_the_assertions_on_the_cards, ComparisonOnTheCards,
    PairingInputBeingRead,
};

/// 本包测试共用的取卡模块：给每张参与的卡的显存上限（登记第五节常量表最后一行回比的对象），服务交的卡对上本机的 Vulkan 独显。
#[path = "../../../tests/common_discrete_gpus_of_this_host/mod.rs"]
mod common_discrete_gpus_of_this_host;
/// 本包测试共用的对拍过库：内联真卡测试先录后判、对拍结论入库按键复用（键与存法都是验证包那一份）。
#[path = "../../../tests/common_pairing_input_through_the_library/mod.rs"]
mod common_pairing_input_through_the_library;

mod arms_and_background_compute_settings;
mod cards_of_the_command;
mod clocks_and_host_load;
mod command_opening_and_construction_lines;
mod concurrent_runs;
mod concurrent_subcommand;
mod construction_phases_and_timing;
mod construction_registration_and_stops;
mod construction_timing_subcommand;
mod crates_digest_and_checks_before_the_run;
mod forced_segment_fallback;
mod gc_call_records_and_anchors;
mod gc_call_windows;
mod gpu_opening_and_measuring_mount;
mod host_hardware_and_groups;
mod judge_trend_subcommand;
mod live_data_contents_and_reading;
mod lockstep_runs;
mod lockstep_subcommand;
mod lockstep_tier_judgement_and_repeats;
mod pool_geometry_and_segment_choice;
#[cfg(test)]
mod pools_cards_and_sessions_for_tests;
mod recording_memory_devices;
mod rejudging_arms;
mod result_lines_of_the_product;
mod run_resumed_by_hardware_group;
mod sensitivity_and_geometry_check_subcommands;
mod sha256_digest;
mod stage_three_cost_subcommand;
mod starting_image_construction;
mod statistics_and_judgement;
mod unit_area_tiers_and_local_constants;

use self::arms_and_background_compute_settings::*;
use self::cards_of_the_command::*;
use self::clocks_and_host_load::*;
use self::command_opening_and_construction_lines::*;
use self::concurrent_runs::*;
use self::concurrent_subcommand::*;
use self::construction_phases_and_timing::*;
use self::construction_registration_and_stops::*;
use self::construction_timing_subcommand::*;
use self::crates_digest_and_checks_before_the_run::*;
use self::forced_segment_fallback::*;
use self::gc_call_records_and_anchors::*;
use self::gc_call_windows::*;
use self::gpu_opening_and_measuring_mount::*;
use self::host_hardware_and_groups::*;
use self::judge_trend_subcommand::*;
use self::live_data_contents_and_reading::*;
use self::lockstep_runs::*;
use self::lockstep_subcommand::*;
use self::lockstep_tier_judgement_and_repeats::*;
use self::pool_geometry_and_segment_choice::*;
#[cfg(test)]
use self::pools_cards_and_sessions_for_tests::*;
use self::recording_memory_devices::*;
use self::rejudging_arms::*;
use self::result_lines_of_the_product::*;
use self::run_resumed_by_hardware_group::*;
use self::sensitivity_and_geometry_check_subcommands::*;
use self::sha256_digest::*;
use self::stage_three_cost_subcommand::*;
use self::starting_image_construction::*;
use self::statistics_and_judgement::*;
use self::unit_area_tiers_and_local_constants::*;

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
/// 每张参与的卡的显存上限 1 GiB（第 10 行；问题单「装置要求」第一条）。
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

const EXIT_SUCCESS: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_STOPPED: i32 = 3;
const EXIT_POSITIVE_CONTROL_FAILED: i32 = 4;
const EXIT_NO_ARM_RAN: i32 = 77;
const EXIT_RUN_CONDITION_NOT_MET: i32 = 78;

// ---- 内联真卡测试的先录后判：小池上 GPU 点名一张卡臂与装置自数的逐段活槽数对拍 ----
// 只录那一段在 CPU 上造输入（起步镜像那一版的分配记录与盘 0 的单元区）、装置自数参照、写进判定库，不取卡；判段读回之后才取卡、只跑 GPU、比，
// 对拍结论按键入库复用（测试包 `common_pairing_input_through_the_library::pairing_concluded_on_the_cards_through_the_pairing_library`，键与存法都是验证包那一份）。
// 这几个 fn 不挂 `cfg(test)`：两侧的入口要编进 bin 本体，构建脚本经函数单元图收它们的闭包时 bin 不开 `cfg(test)`。

/// 参照一侧那一组的组名（登记在 `src/reference_units_entries.rs`，入口是 [`reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment`]）。
#[cfg_attr(
    not(all(test, feature = "tests-on-a-real-gpu")),
    allow(
        dead_code,
        reason = "只给真卡那一条内联测试用；要编进 bin 本体，构建脚本收参照一侧的闭包时 bin 不开 cfg(test)"
    )
)]
const REFERENCE_UNITS_GROUP_OF_THE_APPARATUS_COUNT: &str =
    "apparatus_count_of_the_live_slots_per_segment_in_the_gpu_cpu_gc_comparison";

/// 分配记录与单元区编成对拍输入的字节：单元区的盘号、起点槽、槽数，记录条数，逐条记录是盘号、槽号、跨几个槽、代、释放没有、隔离没有
/// （后两样写 0 或 1），都是 8 字节小端（`common_pairing_input_through_the_library` 的 `append_number`、`append_count`）。
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "只给内联测试用；要编进 bin 本体，构建脚本收参照一侧的闭包时 bin 不开 cfg(test)"
    )
)]
fn pairing_input_bytes_of_the_allocation_records_and_the_unit_area(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Vec<u8> {
    let mut input_bytes = Vec::new();
    append_number(&mut input_bytes, u64::from(unit_area.device.0));
    append_number(&mut input_bytes, unit_area.start.slot().0);
    append_number(&mut input_bytes, unit_area.slots);
    append_count(&mut input_bytes, allocation_records.len());
    for record in allocation_records {
        append_number(&mut input_bytes, u64::from(record.device.0));
        append_number(&mut input_bytes, record.slot.0);
        append_number(&mut input_bytes, u64::from(record.span_slots));
        append_number(&mut input_bytes, record.generation.0);
        append_number(&mut input_bytes, u64::from(record.is_released));
        append_number(
            &mut input_bytes,
            match record.quarantine_mark {
                AllocationRecordQuarantineMark::NotQuarantined => 0,
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck => 1,
            },
        );
    }
    input_bytes
}

/// 写下的 0 或 1 读回成「是不是」。
///
/// # Panics
/// 读回的既不是 0 也不是 1：录的与读的布局对不上。
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "只给内联测试用；要编进 bin 本体，构建脚本收两侧的闭包时 bin 不开 cfg(test)"
    )
)]
fn whether_of_the_number_written(number: u64, field_name: &str) -> bool {
    match number {
        0 => false,
        1 => true,
        other => panic!("{field_name}只写 0 或 1，读回 {other}：录的与读的布局对不上"),
    }
}

/// [`pairing_input_bytes_of_the_allocation_records_and_the_unit_area`] 的反向：单元区起点照 journal 环长重算（起点在环末尾的下一个槽）。
///
/// # Panics
/// 字节与这份布局对不上（短了、取完还有剩、数装不下）；起点不落在聚簇段边界上。
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "只给内联测试用；要编进 bin 本体，构建脚本收两侧的闭包时 bin 不开 cfg(test)"
    )
)]
fn allocation_records_and_the_unit_area_of_the_pairing_input_bytes(
    input_bytes: &[u8],
) -> (Vec<AllocationRecord>, UnitAreaOfOneDevice) {
    let mut input = PairingInputBeingRead::from_the_start_of(input_bytes);
    let device = DeviceIdentity(u32::try_from(input.take_number()).expect("写下的盘号装得进 u32"));
    let start_slot = SlotNumber(input.take_number());
    let journal_ring_bytes = start_slot.to_device_offset().0
        - JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION
            .to_device_offset()
            .0;
    let start = UnitAreaStart::following_the_journal_ring(journal_ring_bytes)
        .unwrap_or_else(|refused| panic!("读回的单元区起点不落在聚簇段边界上：{refused:?}"));
    let unit_area = UnitAreaOfOneDevice {
        device,
        start,
        slots: input.take_number(),
    };
    let record_count = input.take_count();
    let allocation_records = (0..record_count)
        .map(|_record| AllocationRecord {
            device: DeviceIdentity(
                u32::try_from(input.take_number()).expect("写下的盘号装得进 u32"),
            ),
            slot: SlotNumber(input.take_number()),
            span_slots: u16::try_from(input.take_number()).expect("写下的跨槽数装得进 u16"),
            generation: CheckpointTxg(input.take_number()),
            is_released: whether_of_the_number_written(input.take_number(), "释放没有"),
            quarantine_mark: if whether_of_the_number_written(input.take_number(), "隔离没有") {
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck
            } else {
                AllocationRecordQuarantineMark::NotQuarantined
            },
        })
        .collect();
    input.finish();
    (allocation_records, unit_area)
}

/// 参照一侧的入口（登记在 `src/reference_units_entries.rs`）：对拍输入里那份分配记录与单元区上装置自数的每段活槽数
/// （[`live_slots_of_each_segment_counted_by_the_apparatus`]），编成字节：段数，逐段的活槽数，都是 8 字节小端。只录那一段调它。
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "只给内联测试用；要编进 bin 本体，构建脚本收参照一侧的闭包时 bin 不开 cfg(test)"
    )
)]
fn reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment(
    input_bytes: &[u8],
) -> Vec<u8> {
    let (allocation_records, unit_area) =
        allocation_records_and_the_unit_area_of_the_pairing_input_bytes(input_bytes);
    let by_the_apparatus =
        live_slots_of_each_segment_counted_by_the_apparatus(&allocation_records, unit_area);
    let mut reference_bytes = Vec::new();
    append_count(&mut reference_bytes, by_the_apparatus.len());
    for live_slots in by_the_apparatus {
        append_number(&mut reference_bytes, live_slots);
    }
    reference_bytes
}

/// [`reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment`] 写下的参照读回成每段活槽数。
///
/// # Panics
/// 字节与这份布局对不上。
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "只给内联测试用；要编进 bin 本体，构建脚本收 GPU 一侧的闭包时 bin 不开 cfg(test)"
    )
)]
fn live_slots_per_segment_of_the_reference_bytes(reference_bytes: &[u8]) -> Vec<u64> {
    let mut reference = PairingInputBeingRead::from_the_start_of(reference_bytes);
    let segment_count = reference.take_count();
    let live_slots = (0..segment_count)
        .map(|_segment| reference.take_number())
        .collect();
    reference.finish();
    live_slots
}

/// GPU 一侧的入口（登记在 `src/gpu_side_units_entries.rs`，组名是内联真卡测试的名字）：判段读回之后调。取这个进程的卡
/// （[`cards_of_this_process`]：环境变量给了卡照它，没给有领卡通道这一刻领），GPU 点名一张卡臂照丙段代价那一套在读回的分配记录上数
/// [`STAGE_THREE_COST_REPEATS`] 遍（[`stage_three_counts_on_the_arm`]），每一遍逐段与读回的参照比。服务没给卡的来路、或交的卡里没有这一台的，
/// 打 `NOT_RUN_NO_GPU_ADAPTER` 交回没跑成，不入库；给了来路而取不成（交来的卡对不上本机的独显）判红。
#[cfg_attr(
    not(all(test, feature = "tests-on-a-real-gpu")),
    allow(
        dead_code,
        reason = "只给真卡那一条内联测试用；要编进 bin 本体，构建脚本收 GPU 一侧的闭包时 bin 不开 cfg(test)"
    )
)]
fn judged_on_the_cards_that_the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let (allocation_records, unit_area) =
            allocation_records_and_the_unit_area_of_the_pairing_input_bytes(input_bytes);
        let by_the_apparatus = live_slots_per_segment_of_the_reference_bytes(reference_bytes);
        if !the_task_server_hands_over_cards_to_this_process() {
            println!(
                "NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={REASON_WITHOUT_CARDS_FROM_THE_TASK_SERVER}"
            );
            return ComparisonOnTheCards::NotRunOnTheseCards;
        }
        let cards = cards_of_this_process()
            .unwrap_or_else(|reason| panic!("判段要开卡，取卡没成：{reason}"));
        if cards.vulkan_instance.is_none() {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        }
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let on_the_card = stage_three_counts_on_the_arm(
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
            &allocation_records,
            unit_area,
            &cards.discrete_gpus,
            &memory_leaked,
            cards.vulkan_instance.as_ref(),
        )
        .unwrap_or_else(|reason| panic!("GPU 点名一张卡臂数不成：{reason}"));
        assert_eq!(
            on_the_card.walls.len(),
            STAGE_THREE_COST_REPEATS,
            "修订第 9 条：各量 5 次"
        );
        if !every_pass_agrees_with_the_apparatus_count(&on_the_card, &by_the_apparatus) {
            panic!("GPU 点名一张卡臂有一遍逐段与读回的装置自数不同：{on_the_card:?}，参照 {by_the_apparatus:?}");
        }
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

// ---- 命令行 ----

const USAGE: &str = "用法：
  e202_gpu_cpu_gc_comparison geometry-check --product <文件>
  e202_gpu_cpu_gc_comparison lockstep --reading occupancy --unit-area <2288MiB|4GiB|8GiB> [--user-changes <24|48|96>] [--batch-units <4|16>] [--arms <臂,…>] [--cards <全部卡臂开几张>] --product <文件>
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

/// `--cards` 只认不小于 [`CARDS_THE_EVERY_CARD_ARM_OPENS_AT_LEAST`] 的整数（0 与 1 同一句报错）；不给是开服务交来的全部。
/// 多于交来的张数要等取了卡才判得出（[`discrete_gpus_named_for_the_every_card_arm`]）。
fn cards_of_the_every_card_arm_of_the_options(
    options: &mut BTreeMap<String, String>,
) -> Result<CardsOfTheEveryCardArm, String> {
    match options.remove("cards") {
        None => Ok(CardsOfTheEveryCardArm::EveryHandedOverCard),
        Some(text) => {
            let requested_card_count = text
                .parse::<usize>()
                .map_err(|parse_failure| format!("--cards {text} 不是数（{parse_failure}）"))?;
            NonZeroUsize::new(requested_card_count)
                .filter(|named_card_count| {
                    named_card_count.get() >= CARDS_THE_EVERY_CARD_ARM_OPENS_AT_LEAST
                })
                .map(CardsOfTheEveryCardArm::FirstHandedOverCards)
                .ok_or_else(|| {
                    format!(
                        "--cards {text}：全部卡臂至少开 {CARDS_THE_EVERY_CARD_ARM_OPENS_AT_LEAST} 张（只开一张时它与点名一张卡臂是同一条臂）"
                    )
                })
        }
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
            let cards_of_the_every_card_arm =
                cards_of_the_every_card_arm_of_the_options(&mut options)?;
            if cards_of_the_every_card_arm != CardsOfTheEveryCardArm::EveryHandedOverCard
                && !arms_requested
                    .iter()
                    .any(|arm| arm.compute == ComputeOfTheArm::GpuOnEveryCard)
            {
                return Err("--cards 只管全部卡臂开几张，--arms 里没点全部卡臂".to_string());
            }
            CommandOfTheApparatus::Lockstep(LockstepCommand {
                tier,
                user_changes,
                batch_units,
                arms_requested,
                cards_of_the_every_card_arm,
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

/// 一个进程只起一条子命令。要卡的子命令自己在只用 CPU 的活做完、开设备之前取卡、建这个进程的那一份 Vulkan 实例（[`cards_seen_by_the_command`]），
/// 敏感度那一条的交替跑与并发跑共用它；不要卡的不取、不建。
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let exit_code = match command_of_the_arguments(&arguments) {
        Ok(CommandOfTheApparatus::GeometryCheck { product }) => geometry_check_command(&product),
        Ok(CommandOfTheApparatus::Lockstep(command)) => lockstep_command(&command),
        Ok(CommandOfTheApparatus::Concurrent(command)) => concurrent_command(&command),
        Ok(CommandOfTheApparatus::Sensitivity {
            product,
            user_changes,
        }) => sensitivity_command(&product, user_changes),
        Ok(CommandOfTheApparatus::JudgeTrend {
            product,
            lockstep_products,
        }) => judge_trend_command(&product, &lockstep_products),
        Ok(CommandOfTheApparatus::StageThreeCost { tier, product }) => {
            stage_three_cost_command(tier, &product)
        }
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
mod tests;
