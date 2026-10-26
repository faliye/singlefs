//! 挂着的会话：一次可写挂载之后接着发布的那一层（D16（发布语义） 已定项 1「准入」那一行的发布那一处，
//! C283（准入失败时不先推发布就报 ENOSPC））。它持着这次挂载的分配器、现行那一版、实例代号与影子账那一臂，用户的改动经它发布：
//! 发布路径报空间准入不够（`PublishError::SpaceAdmissionRefused`）、或准入放行而落点取不到（`PublishError::PlacementRefused`）时，
//! 先推一串抬 F 的空发布（`mount::push_one_floor_raise_within_the_admission_budget`：F 抬到准入上限，一次准入最多
//! `mount::PUBLISHES_PER_ADMISSION_AT_MOST` 次发布，算上这次用户发布），再重判；做满仍不够、或抬不动了，才报空间不够。
//! 写行那次发布之前不推：会话在可写挂载交回之后才有，写行那次早已发完（可写挂载那一处的推归 `mount::establish_instance`）。
//!
//! 为什么要这一层：发布路径（`transaction::publish_overwrite` 这一族）只拿到一个写入口（`PoolWriter`），拿不到抬 F 要的设备表与参数
//! （抬 F 那一串自己开写入口、读根环与系统配置，`mount::raise_rollback_floor`）——安全设计轮第三轮辩方报的架构缺口
//! （`research/prompts/m2-safety-r3-sonnet-output.md` 第二节）。这一层怎么放是实现员提的（交主 agent 定）：它持着挂载认下的池
//! （`mount::ParametersAndDeviceTableOfTheMount`：盘上择到的系统配置里的参数、挂载时核过的盘表），写入口只按那份参数建、不收调用方的参数；
//! 盘本身不持，每次调用由调用方把盘表交进来——用例在两次调用之间照样摸得到盘（拷镜像、注入故障）——交进来的盘表与挂载时的逐项比，
//! 对不上在任何读写之前拒；身份对得上，还要每块盘「可见」（两个系统配置槽里至少一份本池自证过的），
//! 有一块不可见就在任何写之前拒（代码三方 m2-closeout-code-r1 Z3-A 乙：只比身份时，换上去的一块空盘照样收到本池的写）；
//! 「可见」之后再核每块盘自证槽里的本盘设备号是盘表给它的身份、这块盘不落后于现行那一版又缺它的单元或它的根（D18（块里携带什么信息） 已定项 11
//! 「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定；代码三方 m2-closeout-code-r2「盘体对调的盘表」「同池旧快照的盘经入口洗白」；
//! 缺根那一半是 C585（会话发布放进发布途中取下的旧快照） 用户定的改法 C）。
//! 发布路径与抬 F 各自开写入口的做法不变。

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};

use crate::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
    InstanceGeneration, SlotNumber, TreeIdentifier,
};
use crate::allocator::{AllocationRecord, Placement, PlacementOnDevice, PoolAllocator};
use crate::background_compute::{
    decisions_of_one_segment, BackgroundComputeError, BackgroundComputeOfOneBatch,
    BackgroundComputeOfThisMount, BackgroundComputeSettings, BackgroundComputeSettingsRefused,
    BackgroundGpuCompute, BackgroundGpuComputeHandle, BackgroundTask, GpuComputedSegment,
    GpuEntryDecision, GpuWorkOfTheBatch, InTheMiddleOfEachBackgroundBatch,
};
use crate::block_device::{
    check_aligned_and_in_range, BlockDevice, BlockDeviceError, DiscardOutcome,
    PhysicalBlockSizeInBytes, WriteDurability,
};
use crate::compaction_plan::{
    choose_region, choose_region_over_the_live_slots, cluster_segment_entry_keys_of_the_unit_area,
    compaction_starts_by_the_watermarks, empty_cluster_segments_by_the_records,
    holes_admit_the_issue, holes_outside_the_region_in_segments_holding_user_data,
    plan_batch_skipping_the_units_this_publish_rewrites_anyway, publish_budget_of_a_compaction_run,
    publishes_left_of_the_budget_given_by_the_last_user_change, region_holds_a_quarantined_slot,
    region_is_empty_by_the_records, region_scans_of_the_segments_without_an_unreleased_record,
    the_completed_intent_yielded, CallerMonotonicClockReading, CallerOfTheBackgroundCompaction,
    CompactionBatchSize, CompactionBatchUnits, CompactionOfThisMount, CompactionPacing,
    CompactionRunForcing, CompactionRunStop, CompactionStartJudgement,
    CompactionWriteBandwidthLimit, ForegroundFsync, LiveSlotsOfAClusterSegment, PlannedBatch,
    PlannedRelocation, RegionChoice, RegionChoiceCriterion, TimerFallbackCheck,
    UnitAreaOfOneDevice,
};
use crate::instance_table::InstanceRow;
use crate::instance_table_sweep::unit_head_read_by_sweep;
use crate::journal::JournalBytesToWrite;
use crate::logged_operations::{
    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
};
use crate::make_filesystem::MakeFilesystemParameters;
use crate::mount::{
    devices_behind_the_current_version, devices_without_a_self_verified_system_configuration,
    empty_publish_plan_after_file_version,
    own_device_numbers_on_disk_differing_from_the_identities_handed_in,
    parameters_of_the_pool_agreeing_with_the_caller_inputs,
    push_one_floor_raise_within_the_admission_budget, DeviceBehindTheCurrentVersion,
    FloorRaisePushedWithinTheAdmissionBudget, FloorRaiseStop, MountError, MountOutput, Mounted,
    OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
    RaisedFloor, ShadowLedger,
};
use crate::recovery::{instance_table_chain_of_root, PoolReader, ReadPassOverThePool};
use crate::region_scan::{
    scan_region_on_the_gpu, scan_region_over_the_identity_index, AllocationInRegion,
    CentralMappingEntry, CentralMappingIdentityIndex, RegionScan, RegionScanRefusal,
    RegionScanWithTheRecordsUncertainThisRound,
};
use crate::root_record::{RootRecord, UnmountMarker};
use crate::root_ring::{target_for_publish, RootRingSlot};
use crate::system_configuration::PublishTriggerThresholds;
use crate::transaction::lookup_work::{self, LookupSite};
use crate::transaction::{
    journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write,
    prepare_the_version_publish, publish_compaction_step_in_the_session,
    publish_compaction_step_with_the_region_check_of_the_intent_deletion, publish_create_snapshot,
    publish_data_changes_published_together, publish_new_inodes, publish_overwrite,
    publish_overwrite_of_an_inode, publish_partial_write, publish_sequential_write,
    publish_sequential_write_of_an_inode, publish_size_change, publish_snapshot_destruction_intent,
    publish_unlink_intent, publish_version, walk_the_deletion_queue_until_it_is_empty,
    CentralMappingOfAVersionReadLeafByLeaf, CompactionIntentStep, CompactionStepRefused,
    DataChangesPublishedTogether, DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile,
    MappingLookup, NewInodesOfOneChange, PartialWriteBytes, PlannedUnitsRewrittenAnyway,
    PoolVersion, PoolWriter, PublishError, ReadFailedOnTheDisk, ReadFailingSelfVerification,
    RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck, RelocationBatchRefused,
    TransactionOutput, TransactionUnit, WholeWriteOfOneInode,
};
use crate::unit::{data_unit_payload_capacity, unit_filesystem_identifier, UNIT_CLASS_DATA};
use crate::unlink::DeletionSwitches;
use singlefs_format::ROOT_RING_REGIONS;

/// 一次可写挂载之后的会话：接下来的发布接在 `current` 后面、用这个分配器与实例代号。
#[derive(Debug)]
pub struct MountedSession {
    pub allocator: PoolAllocator,
    pub current: PoolVersion,
    pub instance: InstanceGeneration,
    /// 这次挂载走的影子账那一臂（`MountOutput::shadow_ledger`）：推抬 F 时照它重算影子账。
    pub shadow_ledger: ShadowLedger,
    /// 这次挂载认下的池（`MountOutput::parameters_and_device_table`；没经过可写挂载的会话读盘造，
    /// `ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`）：写入口只按它的参数建，交进来的盘表与它的逐项比。
    pub parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
    /// 整理在这次挂载里会话这一侧的状态（`compaction_plan::CompactionOfThisMount`：不合格段表、常驻隔离集、`B_gc` 数的发布数、观测；
    /// 丁 1 交判第 2 点）：生命周期一次挂载。可写挂载续做意图时建的那一份接着用（`mount::MountOutput::compaction_continuation`）；
    /// 没经过可写挂载的会话从 `CompactionOfThisMount::of_a_new_mount` 起。活意图不在这里：它在现行那一版的 logged_ops 里。
    pub compaction: CompactionOfThisMount,
}

/// 用户的一次改动，经会话发布（发布路径那三种接在带文件的一版后面的改动）。
#[derive(Clone, Copy, Debug)]
pub enum UserChange<'content> {
    /// 覆盖写一个数据单元以内的内容（`transaction::publish_overwrite`）。
    Overwrite(FirstFile<'content>),
    /// 顺序写整份内容，按切分纪律切成几个一单元事务（`transaction::publish_sequential_write`）。
    SequentialWrite(FirstFile<'content>),
    /// 建 `count` 个空 inode（`transaction::publish_new_inodes`）。
    NewInodes { count: u64, write_time_seconds: u64 },
    /// 删一个 inode（D8（核心索引结构） 已定项 16）：一次调用 = 先把现行那一版的 `deleted_inodes` 走空 + 意图那次发布 + 从尾部全部分批
    /// （`transaction::publish_unlink_intent`、`transaction::publish_unlink_steps_until_the_queue_is_empty`）。
    /// `write_time_seconds` 是意图那次写进这条记录三个时间秒的值，各批照抄它（用户 2026-10-03 定 Q-F 乙）。
    Unlink {
        inode: InodeNumber,
        write_time_seconds: u64,
    },
    /// 建一个只读快照（`transaction::publish_create_snapshot`，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2）：
    /// 先把现行那一版的 `deleted_inodes` 走空（同别的改动，D8（核心索引结构） 已定项 16 第 6 条），再发一次独立发布，不与别的改动合在一起。
    /// 现行那一版有树而文件集合为空时照常发布（用户 2026-10-16 定「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md`
    /// 第三节 F5）：快照抄的是那一刻的两棵空树，读它读到没有文件（`recovery::SnapshotFileReading::NoFile`）。
    CreateSnapshot,
    /// 删一个只读快照（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4 与第七节岔路 2、4）：快照按它码 14 条目的树号点名
    /// （岔路 5）。一次调用 = 先把现行那一版的队列走空 + 意图那次发布（`transaction::publish_snapshot_destruction_intent`）+ 全部批
    /// （会话的走空接着走它们，`transaction::walk_the_deletion_queue_until_it_is_empty`），与 unlink 同形、同准入：被空间拒推抬 F 再试，
    /// 不对「先走空」开例外。现行那一版有树而文件集合为空时照样删（删快照不依赖活头里有没有文件：文件删光之后快照照样删得掉，
    /// 丙 2 交判第 5 条，主 agent 定是缺陷；D8（核心索引结构） 已定项 16 第 5 条那一句管的是写文件的改动）。
    DestroySnapshot { snapshot: TreeIdentifier },
    /// 把一个 inode 的 size 改成 `new_size`（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2、K5、K12，
    /// `transaction::publish_size_change`）：一次调用 = 先把现行那一版的承诺类队列走空 + 这一次发布 + 截断时之后的全部批（会话的走空接着走它们，
    /// 次序 `deleted_inodes` → 销毁中的快照 → 截断，`transaction::walk_the_deletion_queue_until_it_is_empty`），与 unlink 同形：
    /// 空间不够推抬 F 再试。N = S 只改记录、N > S 变长一次发布、N < S 意图 + 各批 + 最后一批。`write_time_seconds` 是写进记录三个时间秒的值。
    Truncate {
        inode: InodeNumber,
        new_size: u64,
        write_time_seconds: u64,
    },
    /// 按 inode 号覆盖写一个数据单元以内的内容（`transaction::publish_overwrite_of_an_inode`，里程碑「删除与多次 COW」一·1，
    /// 用户 2026-10-18 定）：把 `inode` 那个文件整份换成 `content`。`inode` 是第一个文件时同 [`UserChange::Overwrite`]。
    /// `inode` 不在现行那一版的 inode 树里 ⇒ `UserChangeRefused::Publish` 带 `WriteToAnInodeThatIsNotInTheTree`。
    OverwriteOfAnInode {
        inode: InodeNumber,
        content: &'content [u8],
        write_time_seconds: u64,
    },
    /// 按 inode 号顺序写整份内容，按切分纪律切成几个一单元事务（`transaction::publish_sequential_write_of_an_inode`）。
    /// `inode` 是第一个文件时同 [`UserChange::SequentialWrite`]；拒绝同 [`UserChange::OverwriteOfAnInode`]。
    SequentialWriteOfAnInode {
        inode: InodeNumber,
        content: &'content [u8],
        write_time_seconds: u64,
    },
    /// 按偏移写一个活 inode（判决 `research/prompts/m4-partial-write-r1-main-verification.md`，`transaction::publish_partial_write`）：把 `bytes`
    /// 写到 `inode` 那个文件偏移 `offset` 起，一次发布；先把现行那一版的承诺类队列走空，同别的改动。拒绝同 `transaction::publish_partial_write`。
    WriteAtOffset {
        inode: InodeNumber,
        offset: FileOffsetInBytes,
        bytes: PartialWriteBytes<'content>,
        write_time_seconds: u64,
    },
    /// 几次数据写类改动并进一次发布（D16（发布语义） 已定项 10「数据写类可并：同一 inode 的写折叠、建 inode 求和」，
    /// `transaction::publish_data_changes_published_together`）：攒批的会话（[`MountedSessionWithPendingChanges`]）排空待发时交这一种。
    /// 先把现行那一版的承诺类队列走空，同别的改动；有树而文件集合为空的一版上只放只建 inode 的那一批（同 [`UserChange::NewInodes`]）。
    DataChangesPublishedTogether(DataChangesPublishedTogether<'content>),
}

/// 一次用户改动发成了：新的一版在会话的 `current` 里。
#[derive(Debug)]
pub struct UserChangePublished {
    /// 这次准入里推的那几串抬 F 的空发布，按先后；一次都没推的是空的。
    pub floor_raises: Vec<RaisedFloor>,
    /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）：`SpaceAdmissionRefused` 或 `PlacementRefused`。
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    /// 这次调用里推的抬 F 之外发成的每一版，按先后（[`VersionsPublishedInThisChange`]）。整理一批都不在这里：整理挪出了用户改动的调用线程
    /// （用户 2026-10-26 定），改动发成之后起跑判据为真只记成「该跑了」（`compaction_plan::CompactionStartJudgement`），
    /// 由后台入口 [`MountedSession::run_compaction_in_the_background`] 去跑。
    pub versions_published: VersionsPublishedInThisChange,
    /// 这次调用的发布都返回之后发的 discard（判决 G5，[`discard_the_segments_reusable_as_a_whole`]）。
    pub discards_after_the_change: DiscardsAfterThePublishes,
}

/// 发布返回之后发的 discard（判决 G5）：每块盘按 64 槽对齐的整段发，各段交回了哪一种结局。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscardsAfterThePublishes {
    /// 交回 `Applied` 的段：哪块盘、段起点。
    pub applied: Vec<(DeviceIdentity, SlotNumber)>,
    /// 交回 `Unsupported` 的段（后端不认 discard，一个字节没动）。
    pub unsupported: Vec<(DeviceIdentity, SlotNumber)>,
    /// 块设备报错的段：discard 只是告诉盘「这一段不用了」，报错不让发布算没发成，记下来报出。
    pub failed: Vec<(DeviceIdentity, SlotNumber)>,
    /// 装着还没清扫的孤儿头、没发的段（[`discard_the_segments_reusable_as_a_whole`]：discard 不当擦除，孤儿头留给实例表清扫抹）。
    pub withheld_holding_unswept_orphan_heads: Vec<(DeviceIdentity, SlotNumber)>,
    /// 判孤儿头时读不出实例表或段里的一个槽、没发的段（读不到不等于没有，按装着算）。
    pub withheld_after_a_failed_read: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 一次整理运行（显式入口 [`MountedSession::run_compaction`]，或后台入口 [`MountedSession::run_compaction_in_the_background`] 起跑的那一趟）做成了什么。
#[derive(Debug)]
pub struct CompactionRun {
    /// 这一趟发成的每一版，按先后：先走空的删除续做、再是整理的各批。最后一个就是会话的 `current`。
    pub versions_published: Vec<TransactionOutput>,
    /// 其中整理批的个数：搬了单元的那几次（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2），签发、
    /// 一个没搬的删意图与丢意图不算。
    pub batches_published: u64,
    pub outcome: CompactionRunOutcome,
    /// 这一趟的发布都返回之后发的 discard（显式入口与后台入口在这里发；可写挂载续做那一趟不发）。
    pub discards: DiscardsAfterThePublishes,
    /// 整理每推一次发布（[`advance_compaction_by_one_publish`] 每进一次门：签发、搬一批、删意图，停在判据上的那一次也算）是后台任务的一批，
    /// 按先后一批一项：请求的后端（这一批开头读到的设置）与实际走的路（D26（后台整理与放置回收） 已定项 13 第 1 条；
    /// `.claude/rules/fs-design.md` 五条硬要求第 4 条）。删除续做那几步不是整理的批，不在这里。
    pub background_compute_of_each_batch: Vec<BackgroundComputeOfOneBatch>,
}

/// 一次整理运行停在哪：照判据停（[`CompactionRunStop`]），或一次发布没发成。
#[derive(Debug)]
pub enum CompactionRunOutcome {
    Stopped(CompactionRunStop),
    Failed(CompactionRunFailure),
}

/// 整理运行里一次发布没发成（被空间拒不在这里：那是暂停，[`CompactionRunStop::PausedShortOfSpace`]）。已发成的那几版留在盘上、`current` 停在最后那一版。
#[derive(Debug)]
pub enum CompactionRunFailure {
    /// 先走空的删除续做有一步没发成。
    DeletionContinuation(PublishError),
    /// 一批没发成（计划对不上上一版，或发布路径的错）。
    Batch(RelocationBatchRefused),
}

/// 整理运行在任何读写之前就拒了。
#[derive(Debug)]
pub enum CompactionRunRefused {
    /// 同 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]：交进来的盘表与这次挂载核过的那一份不同。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 现行那一版树表 0 条：没有可搬的单元。
    NoVersionOfTrees,
}

/// 后台入口一次调用做成了什么（[`MountedSession::run_compaction_in_the_background`]）。
#[derive(Debug)]
pub enum BackgroundCompactionCall {
    /// 没起跑，一个写都没发。
    NotStarted(BackgroundCompactionNotStarted),
    /// 起跑了一趟：为什么起跑、那一趟停在哪、发了哪几版。装箱：不装箱这一个成员比另一个大两百多字节（clippy `large_enum_variant`）。
    Ran {
        trigger: BackgroundCompactionTrigger,
        run: Box<CompactionRun>,
    },
}

/// 后台入口这一次为什么没起跑。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundCompactionNotStarted {
    /// 整理因一批出错停着（D26（后台整理与放置回收） 已定项 13 第 4 条）：「该跑了」留着，解除之后下一次调用再取。
    BackgroundTaskStoppedAfterAComputeError,
    /// 没有「该跑了」，定时兜底关着。
    NothingDueAndTheTimerFallbackOff,
    /// 没有「该跑了」，离定时兜底上一次查还不到一个间隔。
    TimerFallbackIntervalNotElapsed,
    /// 没有「该跑了」，定时兜底这一次查了起跑判据，为假。
    NotDueByTheWatermarksAtTheTimerFallback,
    /// 上一次用户改动给的那一份 `B_gc` 已经扣完（岔路单第 18 行甲：定时兜底与用户改动共用这一份）：等下一次用户改动才再有。
    /// 「该跑了」也取走、不留：下一次用户改动发成之后起跑判据再记。
    PublishBudgetOfTheLastUserChangeSpent,
}

/// 后台入口这一趟为什么起跑。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundCompactionTrigger {
    /// 用户改动发成之后起跑判据记下的「该跑了」（判决 G8 丁：至多一批，预算 `B_gc`）。
    DueAfterAUserChange,
    /// 定时兜底这一次查起跑判据为真（D26 已定项 13 第 5 条）。这一趟没有自己的预算，与用户改动共用上一次用户改动给的那一份 `B_gc`
    /// （岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲，用户 2026-10-26 定）。
    TimerFallback,
}

/// 后台入口这一次起不起跑（[`MountedSession::run_compaction_in_the_background`] 开头那几判的结论）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackgroundCompactionStart {
    NotStarted(BackgroundCompactionNotStarted),
    /// 起跑：为什么起跑、这一趟推到哪、预算按哪一条算。
    Starts {
        trigger: BackgroundCompactionTrigger,
        reach: CompactionRunReach,
        budget: CompactionRunBudget,
    },
}

/// 一趟整理的发布预算按哪一条算。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompactionRunBudget {
    /// 判决 G8 `B_gc`：上一次用户改动之后已经过去的发布数之外还剩的（[`publish_budget_of_a_compaction_run`]，至少 1）。
    /// 显式入口与可写挂载续做照它算。
    SinceTheLastUserChange,
    /// 后台入口的一趟（用户改动之后那一趟与定时兜底那几趟）：上一次用户改动给的那一份 `B_gc` 还剩的，没有至少 1，扣完是 0
    /// （[`publishes_left_of_the_budget_given_by_the_last_user_change`]；岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲）。
    LeftOfTheBudgetGivenByTheLastUserChange,
}

/// 一趟整理每一批之前看不看调用方（D26（后台整理与放置回收） 已定项 13 第 6 条）。封闭集合：显式入口与可写挂载续做不看，后台入口看。
#[derive(Clone, Copy)]
enum PacingOfTheRun<'caller> {
    NotPaced,
    PacedByTheCaller {
        write_bandwidth_limit: CompactionWriteBandwidthLimit,
        caller: &'caller dyn CallerOfTheBackgroundCompaction,
    },
}

/// 一次用户改动里推的抬 F 之外发成的每一版，按先后：先走空的删除步（D8（核心索引结构） 已定项 16 第 6 条）、这次改动本身
/// （unlink 是意图那次）、unlink 之后走空的删除步；与推的抬 F 的空发布按 txg 交错。最后一个是会话的 `current`
/// （之后没再推抬 F 时）。交给调用方逐次比（模型对拍）：会话的 `current` 只是最后那一版。
pub type VersionsPublishedInThisChange = Vec<TransactionOutput>;

/// 一次用户改动没发成。成员按调用方要做的决定分：换一份盘表再发、换回这个池的盘、换一个版本再改、原样看发布路径的错、报空间不够。
#[derive(Debug)]
pub enum UserChangeRefused {
    /// 交进来的盘表的设备身份（按次序）与这次挂载核过的那一份不同（多一块、少一块、换了次序、同一个身份交两次）：
    /// 写入口按盘表逐项写系统配置与单元，拿别的盘表发，多出来的盘收到本池的写、设备数与本盘设备号被改写。
    /// 在任何读写之前返回，盘上逐字节不变、会话不动。推环入口与要出路先核同一判（[`RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount`]、
    /// [`WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount`]）：三个入口收盘表同一判。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 现行那一版没有可改的文件：树表 0 条（这几种改动都要接在有树的一版后面，第一个文件版本走 `transaction::publish_first_file`，
    /// 不经这一层），在任何读写之前返回；或有树而文件集合为空（`PoolVersion::WithTreesAndNoFile`，D8（核心索引结构） 已定项 16 第 5 条：
    /// 删完之后只许建 inode（size 0）、写行与空发布）而这次改动不是建 inode——先走空队列（那一版的 `deleted_inodes` 根指针置零，
    /// 走空一个写都不发；走空把带文件的一版走成文件集合为空的，那几步已落盘），再拒这次改动。
    /// 走空发成的那几版与走空里为空间推的那几串抬 F 已落盘、带在这里（与别的被拒成员一样交回被拒之前发成的，
    /// 调用方逐版比得到）；树表 0 条那一格与队列本来就空的那一格没有走空，三样都是空的。
    NoFileVersionToChange {
        floor_raises: Vec<RaisedFloor>,
        /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）。
        refusals_that_pushed_the_floor_raises: Vec<PublishError>,
        /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）：走空的每一步。
        versions_published: VersionsPublishedInThisChange,
    },
    /// 盘表的身份与这次挂载核过的那一份相同，`devices` 里的这几块（按交进来的次序）却不「可见」：两个系统配置槽里一份本池自证过的
    /// （整槽校验和过、fsid 与本池相同）都没有——换上去的空盘、别的池的盘、两槽都读不出（D18（块里携带什么信息） 已定项 11
    /// 「可写挂载的顺序」；与可写挂载取号之前的逐盘核第一支、挂着之后收盘表的入口同一判，`mount::devices_without_a_self_verified_system_configuration`）。
    /// 只比身份时，发布照样做成、本池的写落到那块盘上（代码三方 m2-closeout-code-r1 Z3-A 乙，用户 2026-09-27 定）。
    /// 读过每块盘的两个系统配置槽，一个写都没发：盘上逐字节不变、会话不动。调用方要做的是停下这块盘上的写（换回这个池的盘），
    /// 不是换一份盘表的次序再发（那是 `DeviceTableOtherThanTheOneOfTheMount`）。
    DevicesWithoutASelfVerifiedSystemConfiguration { devices: Vec<DeviceIdentity> },
    /// 盘表的身份与这次挂载核过的那一份相同、每块盘都「可见」，这几块盘（按交进来的次序）自证过的系统配置槽里记的本盘设备号却不是
    /// 盘表给它的身份——盘体对调了、或插错了位置（与可写挂载、挂着之后收盘表的入口同一判，
    /// `mount::own_device_numbers_on_disk_differing_from_the_identities_handed_in`）。只比身份时，发布照样做成、系统配置轮换把错的本盘设备号
    /// 写进两块盘、根写到不归它的那块盘上，之后这个池可写挂不上（代码三方 m2-closeout-code-r2「盘体对调的盘表」，用户 2026-09-27 定）。
    /// 读过每块盘的两个系统配置槽，一个写都没发：盘上逐字节不变、会话不动。调用方要做的是把盘体放回各自的身份再发。
    OwnDeviceNumbersDifferFromTheDeviceTable {
        disagreements: Vec<OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn>,
    },
    /// 盘表的身份对得上、每块盘都「可见」、本盘设备号也对得上，这几块盘（按交进来的次序）却落后于会话的现行那一版、又缺它的单元或它的根：
    /// 同池一份旧快照被换了上去（与可写挂载取号之前逐盘核的落后支、挂着之后收盘表的入口同一判，`mount::devices_behind_the_current_version`）。
    /// 不核这一判时发布照样做成、旧快照的系统配置被轮换到「跟得上」，之后可写挂载的落后支就认不出它、池级 checker 红 I-2.1
    /// （代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」，用户 2026-09-27 定）。旧快照取在一次发布途中（单元与记录已写、根槽 FUA
    /// 还没写）、那次发布的根又住在这块盘上的，单元都在而根槽里不是现行那一版：不核缺根那一半时发布照样做成、池级 checker 红 I-3.1
    /// （C585（会话发布放进发布途中取下的旧快照），用户定改法 C）。只剩一槽自证、那一槽停在上一次轮换的盘落后而单元都在、根槽里是它，不在这里。
    /// 读过每块盘的两个系统配置槽、落后那几块上现行那一版单元的落点与它的根槽，一个写都没发：盘上逐字节不变、会话不动。
    /// 调用方要做的是换回这个池现行的盘。
    DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
        devices: Vec<DeviceBehindTheCurrentVersion>,
    },
    /// 发布被拒、不是空间不够的那两种（内容装不下、释放判定对不上、块设备错、冻结着一次没重发……）：`cause` 原样，不因它推。
    /// `floor_raises` 是这之前已经因为空间不够推过的那几串（它们已落盘，现行那一版在它们后面）。
    Publish {
        cause: PublishError,
        floor_raises: Vec<RaisedFloor>,
        /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）。
        versions_published: VersionsPublishedInThisChange,
    },
    /// 空间不够（报给用户就是 ENOSPC），推过的与不再推的原因在 [`NoSpaceAfterRaisingTheFloor`] 里。
    /// 装箱：不装箱 `UserChangeRefused` 就大过 clippy `result_large_err` 的 128 字节（`MountError` 那几个成员同一个做法）。
    NoSpaceAfterRaisingTheFloor(Box<NoSpaceAfterRaisingTheFloor>),
    /// 发布被拒是空间不够、为它推一串抬 F 时抬 F 自己报了不是空间不够的错（块设备错、读不出、预演被别的理由拒……）：不是「推满仍不够」，
    /// 不报空间不够（代码审阅第 23 条），错原样在 [`FloorRaiseFailedWhilePushingForSpace`] 里。抬 F 那一串自己也取不到落点、被空间准入拒的
    /// 是推满仍不够，走 `NoSpaceAfterRaisingTheFloor`。装箱同 `NoSpaceAfterRaisingTheFloor`。
    FloorRaiseFailedWhilePushingForSpace(Box<FloorRaiseFailedWhilePushingForSpace>),
}

/// 推满仍不够：最后一次被拒是 `last_refusal`（`SpaceAdmissionRefused` 或 `PlacementRefused`），已经推了 `floor_raises` 那几串
/// （每一串因为 `refusals_that_pushed_the_floor_raises` 里对应的那一次被拒才推），`stop` 是不再推的原因
/// （一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）。
#[derive(Debug)]
pub struct NoSpaceAfterRaisingTheFloor {
    pub last_refusal: PublishError,
    pub floor_raises: Vec<RaisedFloor>,
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    pub stop: FloorRaiseStop,
    /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）。
    pub versions_published: VersionsPublishedInThisChange,
    /// 这一次是不是被宽护卡住（[`WideProtectionAtTheCeiling`]）：F 已在上限、C 那一圈还隔离着槽时点名这一形、带上算出路要的东西
    /// （出路在调用方要的时候算，[`MountedSession::ways_out_of_the_wide_protection`]：推环那一条预演过放得下才列），
    /// 别的形是 [`WideProtectionAtTheCeiling::NotWhatStopsTheAdmission`]，照旧只是空间不够。
    pub wide_protection_at_the_ceiling: WideProtectionAtTheCeiling,
}

/// 推满仍不够时，挡着准入的是不是「这个进程写掉那一槽才放」的那一份隔离（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」：
/// 不自动推环，准入因宽护卡住时报一个点名「被宽护卡住」的错、带上出路；用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」：
/// 推环那一条出路预演过、推完这次改动放得下才列；同日弹窗定「改：预演延到要出路时才做」：报的时候不预演，调用方要出路时另调
/// [`MountedSession::ways_out_of_the_wide_protection`] 算，只要错误码、不看出路的调用方不付这一笔）。
///
/// 为什么不另开一个 [`UserChangeRefused`] 成员：checker 档的装置（`e158_root_choice_repair`、`e168_delete_ring_window_borrow`）对
/// `UserChangeRefused` 与 `FloorRaiseStop` 写的是穷举 `match`，多一个成员它们就编不过，而 checker 档这一批不改（第 10z 批第十六块派发）。
/// 点名这一形的成员住在这里，外层照旧是 `NoSpaceAfterRaisingTheFloor`，交回的已落盘的东西不变。
#[derive(Debug)]
pub enum WideProtectionAtTheCeiling {
    /// 不是这一形：不再推的原因不是「F 已在上限」（预算用完、抬 F 那一串自己被拒），或 C 那一圈这一刻一个槽都没隔离。
    NotWhatStopsTheAdmission,
    /// 被宽护卡住（[`StuckBehindTheWideProtection`]）。装箱同 `NoSpaceAfterRaisingTheFloor`。
    StuckBehindTheWideProtection(Box<StuckBehindTheWideProtection>),
}

/// 被宽护卡住：准入被拒、会话推抬 F 停在上限（`FloorRaiseStop::FloorAlreadyAtTheCeiling`），而 C 那一圈（水位之上没施加的记录点名的落点、
/// 那几次发布的账、比所选那一版新的可读根的账；认不出回退时并进 R 那一槽的「现行账里全部已释放的落点」就是宽护，它是这一圈里最大的一份）
/// 还隔离着槽。这一圈只在这个进程写掉那一槽时解开（`allocator::PoolAllocator::record_root_written_by_this_process`），写被拒就不发布、
/// 那一槽写不掉，不管它就隔离到进程结束（第 10z 批第十四块实现员报告第六节，384 槽盘 22 形写全被拒）。
///
/// 这里分不出 C 那一圈里哪一份是宽护、哪一份是窄的那几份：分得出要在根环表里另记一样（`PoolAllocator` 的 `{:?}` 随之变，checker 档 E171
/// 记它），这一份不记；出路对整圈算。
///
/// **宽护是挡着准入的之一，不是唯一**：同一刻没有 C 那一圈，准入照样可以被拒（第 10z 批代码轮第一轮攻方
/// `research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md` 第一节：同一个池停在 R 之前、C 一槽都没有的对照臂，同一个长度也被拒的 27 格，
/// 这里照样报这一形）。所以报这一形不说明写掉 R 那一槽之后放得下：推完放不放得下由调用方要出路时那一份预演说
/// （[`MountedSession::ways_out_of_the_wide_protection`]），放不下（推完上限退到环里最旧的根、推环自己换下的那几份要等环再转一圈才回收，
/// 同一节「读法」）就不列推环。
///
/// 报这一形只读分配器（内存）：不拷分配器、不预演、一个盘都不读。预演要的「被拒在哪一版上」「这次改动还剩什么」记在这里（私有，
/// 只有会话造得出这一份），这次改动的内容拷一份进来（错误不借调用方的内容）。
#[derive(Debug)]
pub struct StuckBehindTheWideProtection {
    /// 推环要写到的那一槽：C 那一圈还没解开、记着落点的槽里，从现行那一版起按 txg 往后最后一个轮到的（R 那一槽）。
    pub ring_slot_of_r: RootRingSlot,
    /// R 的 txg：根环表里那一槽记着的有效根（挂载那一刻按最新那条根的实例表有效、记成有效根，写行之后才被抛弃的那一条）；
    /// 那一槽记的不是有效根（读坏、读回全零、没有根的记录）时 `None`。
    pub checkpoint_txg_of_r: Option<CheckpointTxg>,
    /// 从现行那一版起要发几次 F 不变的空发布才写掉 R 那一槽（不超过环长 3S）。
    pub empty_publishes_until_the_ring_slot_of_r_is_written: u64,
    /// C 那一圈每块盘上这一刻隔离着的槽数（按分配器里盘的次序，同一个槽只数一次）：宽护挡着的槽数。只数 C 那一圈罩着的，
    /// 被抛弃根罩着、C 不罩的不在其内。
    pub isolated_slots_on_each_device: Vec<(DeviceIdentity, u64)>,
    /// 被拒的那一次要接在后面的那一版（报这一形那一刻会话的现行那一版）的实例代号与 txg：要出路时会话不在这一版上就不预演
    /// （[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。报里不记是哪个会话报的：
    /// 别的会话的报这两个数恰好相同时不拦（同一成员的文档）。
    instance_of_the_version_refused_on: InstanceGeneration,
    checkpoint_txg_of_the_version_refused_on: CheckpointTxg,
    /// 被拒的那一次之后这次用户改动还剩什么要发（内容拷一份）：要出路时推完环照它再预演一次。
    remainder_of_the_change: RemainderOfTheUserChangeKeptByTheRefusal,
}

/// 调用方要出路时预演推环与推完之后这次改动的准入（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」「改：预演延到要出路时才做」）：
/// 在分配器的一份拷贝上、对只进内存的盘走一遍推环入口与推完之后这次改动剩下的发布（`rehearse_the_push_then_the_remainder_of_the_change`），
/// 一个写都没发到盘上，会话不动。
#[derive(Debug)]
pub enum RootRingPushRehearsal {
    /// 推环做成，推完之后报里剩下的那一段放得下（还剩这次改动时是走空再发它；只剩走空时是那一段走空，unlink 的意图已发成）：出路列推环。
    /// `isolated_slots_released_by_the_push_on_each_device` 是推完之后每块盘隔离的槽数
    /// 比推之前少了几个（按分配器里盘的次序；推完反而多的那块盘记 0）：C 那一圈与被抛弃根罩着同一个槽时，被抛弃根还在环里那一槽就不放，
    /// 推环途中写掉被抛弃根那一槽时它独占的也放出来——与 `isolated_slots_on_each_device` 不是同一个数。
    ChangeAdmittedAfterThePush {
        isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)>,
    },
    /// 推环做成，推完之后这次改动照旧被拒（`refusal` 是预演里交回的那一种，例如推完上限退到环里最旧的根、推满仍不够）：出路不列推环。
    ChangeStillRefusedAfterThePush {
        isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)>,
        refusal: Box<UserChangeRefused>,
    },
    /// 推环自己在预演里被拒（`RootRingPushRefused` 的哪一种原样）：出路不列推环。
    PushRefused(Box<RootRingPushRefused>),
}

/// 调用方要出路时算出来的（[`MountedSession::ways_out_of_the_wide_protection`]）：那一刻的预演结局与照它列的出路。
#[derive(Debug)]
pub struct WaysOutOfTheWideProtection {
    /// 在分配器的拷贝与只进内存的盘上预演推环、推完再预演这次改动（连同它会推的抬 F）的结局：推环这条出路列不列按它定。
    pub pushing_the_root_ring: RootRingPushRehearsal,
    /// 出路：卸载重挂一定列；推环（会丢掉最近可退到的状态，调用即确认）只在预演过推完这次改动放得下时列
    /// （[`RootRingPushRehearsal::ChangeAdmittedAfterThePush`]）。按这个次序。报给用户的文字由调用方照每一条带的数写。
    pub ways_out: Vec<WayOutOfTheWideProtection>,
}

/// 被宽护卡住时的一条出路（[`WaysOutOfTheWideProtection::ways_out`]）：说清做什么、做了会怎样要的数，文字由调用方写。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WayOutOfTheWideProtection {
    /// 正常卸载（`mount::unmount`）之后再可写挂载一次。`checkpoint_txg_of_the_root_in_that_ring_slot` 是宽护挡着的那一槽（R 那一槽）记着的有效根的 txg：有它时再挂按新实例的实例表
    /// 判它被抛弃，影子账只护它自己账里引用的落点，宽护不再出现；它的账读不出若是盘上永久坏，卸载之后再挂会被拒（C694 的 B），而会话在卸载时
    /// 已断开。`None`：R 那一槽（`ring_slot_held_by_the_wide_protection`）没有读得出的有效根，再挂时择的是卸载推出来的最后一版，R 那几条记录不在它的水位之上，
    /// C 那一圈照它重算、不再有这一槽，宽护不再出现。
    UnmountThenMountWritableAgain {
        ring_slot_held_by_the_wide_protection: RootRingSlot,
        checkpoint_txg_of_the_root_in_that_ring_slot: Option<CheckpointTxg>,
    },
    /// 调 [`MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`]：从现行那一版起发
    /// `empty_publishes_until_that_ring_slot_is_written` 次 F 不变的空发布，写掉根环里接下来轮到的同样多槽，R 那一槽（`ring_slot_held_by_the_wide_protection`）
    /// 是最后一槽；宽护随之解开，预演里每块盘放出来的隔离槽数是 `isolated_slots_released_on_each_device`。这几槽里原来的根换成同一个用户状态的
    /// 空发布根，回退候选集可以掉到少于 4 个不同状态（D16（发布语义） 已定项 1），调用即确认。推完之后做什么见 `after_the_push`。
    PushTheRootRingUpToTheRingSlotHeldByTheWideProtection {
        ring_slot_held_by_the_wide_protection: RootRingSlot,
        empty_publishes_until_that_ring_slot_is_written: u64,
        isolated_slots_released_on_each_device: Vec<(DeviceIdentity, u64)>,
        after_the_push: WhatIsLeftAfterThePush,
    },
}

/// 推环那一条出路推完之后还剩什么（按报里剩下什么分）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhatIsLeftAfterThePush {
    /// 报里还剩这次改动：预演过推完之后这次改动放得下，调用方推完再发它。
    PublishTheChangeAgain,
    /// 这次 unlink 的意图已发成，报里只剩它之后的走空：预演过推完之后剩下的走空在下一次改动开头走、放得下；不再发这次改动
    /// （照「再发这次改动」再发，开头的走空走完、要 unlink 的文件已不在，被拒）。
    WalkTheQueueAtTheStartOfTheNextChange,
}

/// 要出路时没预演。成员按调用方要做的决定分：再发一次这次改动拿新的报、换回挂载时那一份盘表。
/// 两种都在预演之前返回：一个盘都没读、一个写都没发，会话不动。
#[derive(Debug)]
pub enum WaysOutOfTheWideProtectionRefused {
    /// 会话的现行那一版已不是报「被宽护卡住」时被拒的那一次要接在后面的那一版（之后又发过：推过环、别的改动发成、走空走了几步）：
    /// 报里的 R 那一槽、要推的次数与挡着的槽数说的是那一刻，照这一刻预演会与它们对不上，按哪一刻算条款没写，第一版不支持。
    /// 调用方要做的是再发一次这次改动、拿新的那一份报再要出路。这一道在盘表那一道之前：会话已不在那一版上、交进来的盘表又与挂载时的不同，
    /// 报的是这一种。同一个池卸载或崩了之后再挂取新实例，现行那一版的实例代号随之变，再挂之前的报在这一道拒成这一种
    /// （`instance_of_the_current_version` 是新实例的）。
    ///
    /// 这一道只比实例代号与 txg，报里不记是哪个会话报的。别的会话（同一份 mkfs 起的另一个池、读盘造的另一条会话）
    /// 的报若这两个数恰好与这个会话的现行那一版相同，这里不拦：照这个会话的状态（分配器、盘、现行那一版）预演，预演发的是报里拷下的那一份改动
    /// （不是这个会话自己要发的），卸载重挂那一条带的仍是报里的数
    /// （R 的 txg 或 R 那一槽），列推环那一条时其中的 R 那一槽与要推的次数也是报里的（第 10z 批代码轮攻方「打中一」；用户 2026-10-09 定撤掉会话身份、照实写）。
    /// 调用方要拿报去问报它的那个会话。
    SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
        instance_refused_on: InstanceGeneration,
        checkpoint_txg_refused_on: CheckpointTxg,
        instance_of_the_current_version: InstanceGeneration,
        checkpoint_txg_of_the_current_version: CheckpointTxg,
    },
    /// 会话还在被拒的那一版上（实例代号与 txg 与报里记的相同），交进来的盘表的设备身份（按次序）却与这次挂载核过的那一份不同（多一块、少一块、换了次序、
    /// 同一个身份交两次；与 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`] 同一判，`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`）。
    /// 预演经推环的本体走（`MountedSession::push_the_root_ring_on`，交的是包过的盘），本体里核盘表那一套不看次序：不先核这一判时，
    /// 次序对调的盘表预演放得下、列推环，照出路二推完再发这次改动却拒成 `DeviceTableOtherThanTheOneOfTheMount`（第 10z 批代码轮攻方「打中二」）。
    /// 推环公开入口与 [`MountedSession::publish_user_change`] 先核同一判（[`RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount`]）：
    /// 三个入口收盘表同一判。调用方要做的是换回挂载时那一份盘表再要出路。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
}

impl StuckBehindTheWideProtection {
    /// 照这一份报与要出路时那一份预演列出路：卸载重挂一定列；推环只在预演过推完这次改动放得下时列。按这个次序。
    /// 推环那一条推完之后还剩什么按报里剩下什么分：还剩这次改动时是推完再发它；只剩走空时是意图已发成、剩下的走空在下一次改动开头走、
    /// 不再发这次改动。
    fn ways_out_after(
        &self,
        pushing_the_root_ring: &RootRingPushRehearsal,
    ) -> Vec<WayOutOfTheWideProtection> {
        let mut ways_out = vec![WayOutOfTheWideProtection::UnmountThenMountWritableAgain {
            ring_slot_held_by_the_wide_protection: self.ring_slot_of_r,
            checkpoint_txg_of_the_root_in_that_ring_slot: self.checkpoint_txg_of_r,
        }];
        match pushing_the_root_ring {
            RootRingPushRehearsal::ChangeAdmittedAfterThePush {
                isolated_slots_released_by_the_push_on_each_device,
            } => {
                let after_the_push = match &self.remainder_of_the_change {
                    RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(
                        _,
                    ) => WhatIsLeftAfterThePush::PublishTheChangeAgain,
                    RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue => {
                        WhatIsLeftAfterThePush::WalkTheQueueAtTheStartOfTheNextChange
                    }
                };
                ways_out.push(WayOutOfTheWideProtection::PushTheRootRingUpToTheRingSlotHeldByTheWideProtection {
                    ring_slot_held_by_the_wide_protection: self.ring_slot_of_r,
                    empty_publishes_until_that_ring_slot_is_written: self
                        .empty_publishes_until_the_ring_slot_of_r_is_written,
                    isolated_slots_released_on_each_device:
                        isolated_slots_released_by_the_push_on_each_device.clone(),
                    after_the_push,
                });
            }
            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
            | RootRingPushRehearsal::PushRefused(_) => {}
        }
        ways_out
    }
}

/// 预演用的盘：读穿到底下那块盘（这一刻盘上的字节），写与写零只记进内存、盖在之后的读上，屏障什么都不做——一个写都不发到底下那块盘。
/// 要出路时预演推环与推完之后这次改动的准入用（[`rehearse_the_push_then_the_remainder_of_the_change`]）；可写挂载取号之前预演续做、
/// 续做要推抬 F 时，取号之后那一串连同续做也在它上面走一遍（`mount` 的 `walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`）：
/// 那一处底下那块盘是这次挂载共用的那一份读（[`DeviceReadOnceInThisMount`]），这里读成的块在那里记下，真做再读取的是同一份。
/// 底下那块盘只读借成 `&dyn BlockDevice`：改盘的三个动作要 `&mut`，类型上调不到。请求先过两个后端那一份判据
/// （`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒。
pub(crate) struct DeviceKeepingWritesInMemory<'disk> {
    disk: &'disk dyn BlockDevice,
    physical_block_size: PhysicalBlockSizeInBytes,
    /// 写过的物理块：块号（按 `physical_block_size`）→ 那一块现在的字节。写零也逐块记成全 0（发布路径不写零，只有 mkfs 写零）。
    written_blocks: BTreeMap<u64, Vec<u8>>,
}

impl<'disk> DeviceKeepingWritesInMemory<'disk> {
    pub(crate) fn over_the_disk(disk: &'disk dyn BlockDevice) -> Self {
        DeviceKeepingWritesInMemory {
            disk,
            physical_block_size: disk.probe_physical_block_size(),
            written_blocks: BTreeMap::new(),
        }
    }

    fn block_bytes(&self) -> u64 {
        u64::from(self.physical_block_size.0)
    }

    /// 两个后端那一份判据（`block_device::check_aligned_and_in_range`），设备大小取底下那块盘的。
    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        check_aligned_and_in_range(
            offset,
            length,
            self.physical_block_size,
            self.disk.size_in_bytes(),
        )
    }

    /// 块号 `first_unwritten_block` 起、到 `end_block` 之前那一段没写过的块从底下那块盘读进 `buffer` 里对应的那一截
    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）；`pass` 是当场重读那一遍时走底下那块盘的
    /// [`BlockDevice::reread_at_after_a_bad_read`]。
    fn read_unwritten_blocks_from_the_disk(
        &self,
        first_unwritten_block: u64,
        end_block: u64,
        first_block_of_the_buffer: u64,
        buffer: &mut [u8],
        pass: ReadPassOverThePool,
    ) -> Result<(), BlockDeviceError> {
        let block_bytes = self.block_bytes();
        let start_in_buffer =
            usize::try_from((first_unwritten_block - first_block_of_the_buffer) * block_bytes)
                .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let end_in_buffer = usize::try_from((end_block - first_block_of_the_buffer) * block_bytes)
            .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let offset = DeviceOffsetInBytes(first_unwritten_block * block_bytes);
        let range_of_the_buffer = &mut buffer[start_in_buffer..end_in_buffer];
        match pass {
            ReadPassOverThePool::FirstRead => self.disk.read_at(offset, range_of_the_buffer),
            ReadPassOverThePool::RereadAfterABadRead => self
                .disk
                .reread_at_after_a_bad_read(offset, range_of_the_buffer),
        }
    }

    /// 读一段：写过的块取内存里那一份，没写过的那几段从底下那块盘读（`pass` 定走 [`BlockDevice::read_at`] 还是当场重读那一读）。
    fn read_over_the_blocks_kept_in_memory(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
        pass: ReadPassOverThePool,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length)?;
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length / block_bytes;
        // 迭代上界是这次读的块数；跨轮带着还没读的那一段没写过的块从哪一块起；出口只有底下那块盘读错（原样交回）。
        let mut unwritten_run_start: Option<u64> = None;
        for block_index in first_block..end_block {
            match self.written_blocks.get(&block_index) {
                None => {
                    unwritten_run_start.get_or_insert(block_index);
                }
                Some(written) => {
                    if let Some(run_start) = unwritten_run_start.take() {
                        self.read_unwritten_blocks_from_the_disk(
                            run_start,
                            block_index,
                            first_block,
                            buffer,
                            pass,
                        )?;
                    }
                    let start_in_buffer = usize::try_from(block_index - first_block)
                        .expect("块数装得进 usize")
                        * block_length;
                    buffer[start_in_buffer..start_in_buffer + block_length]
                        .copy_from_slice(written);
                }
            }
        }
        if let Some(run_start) = unwritten_run_start {
            self.read_unwritten_blocks_from_the_disk(
                run_start,
                end_block,
                first_block,
                buffer,
                pass,
            )?;
        }
        Ok(())
    }

    fn keep_in_memory(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
            self.written_blocks
                .insert(first_block + block_offset, block.to_vec());
        }
    }
}

impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_over_the_blocks_kept_in_memory(offset, buffer, ReadPassOverThePool::FirstRead)
    }

    /// 写过的块照旧取内存里那一份（这一趟写的，盘上没有）；没写过的走底下那块盘的当场重读那一读。
    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_over_the_blocks_kept_in_memory(
            offset,
            buffer,
            ReadPassOverThePool::RereadAfterABadRead,
        )
    }

    /// 没写过的那几段交给底下那块盘（它们是从那里重读来的）；写过的块是这一趟写在内存里的，不交。
    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
        for (block_index, block) in (first_block..).zip(bytes.chunks(block_length)) {
            if !self.written_blocks.contains_key(&block_index) {
                self.disk.keep_the_self_verified_reread(
                    DeviceOffsetInBytes(block_index * block_bytes),
                    block,
                );
            }
        }
    }

    /// 没写过的那几段交给底下那块盘（读坏的字节是从那里读来的，那一层记进读坏账、丢掉记下的那几块），一段连着的没写过的块交一次；
    /// 写过的块是这一趟写在内存里的，不交。
    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        let block_bytes = self.block_bytes();
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length_in_bytes / block_bytes;
        let hand_the_unwritten_run_to_the_disk = |run_start: u64, run_end: u64| {
            self.disk
                .note_the_read_failing_self_verification_and_forget_its_bytes(
                    DeviceOffsetInBytes(run_start * block_bytes),
                    (run_end - run_start) * block_bytes,
                );
        };
        // 迭代上界是这一段的块数；跨轮带着还没交的那一段没写过的块从哪一块起；没有提前出口。
        let mut unwritten_run_start: Option<u64> = None;
        for block_index in first_block..end_block {
            if self.written_blocks.contains_key(&block_index) {
                if let Some(run_start) = unwritten_run_start.take() {
                    hand_the_unwritten_run_to_the_disk(run_start, block_index);
                }
            } else {
                unwritten_run_start.get_or_insert(block_index);
            }
        }
        if let Some(run_start) = unwritten_run_start {
            hand_the_unwritten_run_to_the_disk(run_start, end_block);
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        _durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"))?;
        self.keep_in_memory(offset, bytes);
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        let zeros = vec![0u8; usize::try_from(length).expect("写零的长度装得进 usize")];
        self.keep_in_memory(offset, &zeros);
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.disk.size_in_bytes()
    }
}

/// 可写挂载这次挂载里的读，取号之前的预演与取号之后的真做共用一份（岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一
/// 「读一次两边共用」那一行，用户定）：共用的那一段里每一块在这次挂载里第一次读时才穿到底下那块盘，读到的字节记下；之后再读这一块——预演
/// （分配器拷贝上那一趟、只进内存那一趟，[`DeviceKeepingWritesInMemory`] 读没写过的块穿到它这里）还是真做（判号、写行、暖机、抬 F、续做）——
/// 都取记下的那一份，不再读盘。照旧读盘的几段不在共用的那一段里：取号之前逐盘核、真做取号写之前那几次读、实例表清扫
/// （[`ReadRouteOfThisMount::StraightToTheDisk`]），取号写与轮换写算世代号、覆写哪一槽那一读（[`BlockDevice::read_at_straight_from_the_disk`]），
/// 读坏之后当场重读那一读（见下）。这次挂载写成的块记成写下的字节，之后读它的读写下的。
/// 写报错的那几块原来记着的丢掉（盘上那几块现在是什么不知道），之后读它们穿到盘上。
/// 读错（盘报了错）的那一块不记，下一次读照样穿到盘上，读错记进 `reads_failed_on_the_disk`；读坏（读成了、读的一方核出自证不过，
/// [`BlockDevice::note_the_read_failing_self_verification_and_forget_its_bytes`]）的那几块从记下的那一份里丢掉（坏字节不留，下一次读穿到盘上），
/// 读坏记进 `reads_failing_self_verification`（岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路一「两样都做」那一行，用户定）。
/// 两本账几块盘各共用一份：取号之前预演没走到续做的结局时拿它们判（`mount::establish_instance`）。
/// 记下的那一份活到这次挂载交回为止，读过的、写过的块都在里面，没有上限（岔路二「先量再定」归计数实验）。
/// 读走哪一路看几块盘共用的 `read_route`（[`ReadRouteOfThisMount`]，`mount::establish_instance` 按段切）。
/// 读坏之后当场重读那一读（[`BlockDevice::reread_at_after_a_bad_read`]）不取记下的那一份、穿到盘上读；调用方核过自证交回来的那几字节
/// （[`BlockDevice::keep_the_self_verified_reread`]）换掉记下的那一份（[`ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies`]）。
/// 请求先过两个后端那一份判据（`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒、不发到盘上。
pub(crate) struct DeviceReadOnceInThisMount<'disk, 'log, Device> {
    identity: DeviceIdentity,
    disk: &'disk mut Device,
    physical_block_size: PhysicalBlockSizeInBytes,
    /// 这次挂载读成或写成的物理块：块号（按 `physical_block_size`）→ 那一块的字节。
    blocks_read_or_written_in_this_mount: RefCell<BTreeMap<u64, Vec<u8>>>,
    reads_failed_on_the_disk: &'log RefCell<Vec<ReadFailedOnTheDisk>>,
    reads_failing_self_verification: &'log RefCell<Vec<ReadFailingSelfVerification>>,
    read_route: &'log Cell<ReadRouteOfThisMount>,
}

/// [`DeviceReadOnceInThisMount`] 的读此刻走哪一路。写不分路：写成的块一律记成写下的字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReadRouteOfThisMount {
    /// 取记下的那一份，没记下的穿到盘上读、读成记下：预演与真做共用的那一份（岔路一「读一次两边共用」那一行）。
    SharedBetweenTheRehearsalAndTheRealRun,
    /// 穿到盘上读，不取也不改记下的那一份：条款要这一刻再读一次盘的那几段——取号之前逐盘核（读阶段之后的瞬时读错在这一遍报，
    /// D18（块里携带什么信息） 已定项 11）、真做取号（屏障前后各读一次见证值，C554 乙-配置续 Q1）、实例表清扫（抹之前重读每个头，
    /// 用户 2026-10-04 定 K1-b）。它们不在岔路单那一行说的「抬 F 与续做要读的东西」里，预演也不替它们读。
    StraightToTheDisk,
    /// 共用的那一段里读坏（读不出或自证不过）之后当场重读的那一读（[`BlockDevice::reread_at_after_a_bad_read`]；D19（块指针的结构与宽度预算）
    /// 已定项 5、D16（发布语义） 已定项 1「根槽这一次读坏」那一行要它再读一次盘）：绕过记下的那一份、穿到盘上读，读错照样记进读错账；
    /// 读到的字节调用方核过自证才交回来（[`BlockDevice::keep_the_self_verified_reread`]），换掉记下的那一份——预演里换了，真做再读取的就是它。
    /// 第一次读回来的坏字节记下了也挡不住这一读（主 agent 判定：抬 F 与续做里「读坏当场重读」那一读不许被共用的那一份吃掉）。
    /// 这一路只按每一次重读走，不装进几块盘共用的那一格。
    RereadStraightToTheDiskKeepingWhatSelfVerifies,
}

impl ReadRouteOfThisMount {
    /// 这一段里当场重读那一读走哪一路：共用的那一段改走穿到盘上、核过自证再换掉记下的那一份；照旧读盘的那几段本来就穿到盘上，
    /// 也不改记下的那一份。
    fn for_the_reread_after_a_bad_read(self) -> ReadRouteOfThisMount {
        match self {
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies
            }
            ReadRouteOfThisMount::StraightToTheDisk => ReadRouteOfThisMount::StraightToTheDisk,
        }
    }
}

impl<'disk, 'log, Device: BlockDevice> DeviceReadOnceInThisMount<'disk, 'log, Device> {
    pub(crate) fn over_the_disk(
        identity: DeviceIdentity,
        disk: &'disk mut Device,
        reads_failed_on_the_disk: &'log RefCell<Vec<ReadFailedOnTheDisk>>,
        reads_failing_self_verification: &'log RefCell<Vec<ReadFailingSelfVerification>>,
        read_route: &'log Cell<ReadRouteOfThisMount>,
    ) -> Self {
        let physical_block_size = disk.probe_physical_block_size();
        DeviceReadOnceInThisMount {
            identity,
            disk,
            physical_block_size,
            blocks_read_or_written_in_this_mount: RefCell::new(BTreeMap::new()),
            reads_failed_on_the_disk,
            reads_failing_self_verification,
            read_route,
        }
    }

    fn block_bytes(&self) -> u64 {
        u64::from(self.physical_block_size.0)
    }

    /// 两个后端那一份判据（`block_device::check_aligned_and_in_range`），设备大小取底下那块盘的。
    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let device_size = self.disk.size_in_bytes();
        check_aligned_and_in_range(offset, length, self.physical_block_size, device_size)
    }

    /// 块号 `first_unrecorded_block` 起、到 `end_block` 之前那一段没记下的块从底下那块盘读进 `buffer` 里对应的那一截
    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）：读成就逐块记下；读错记进读错账、原样交回。
    fn read_unrecorded_blocks_from_the_disk(
        &self,
        first_unrecorded_block: u64,
        end_block: u64,
        first_block_of_the_buffer: u64,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let start_in_buffer =
            usize::try_from((first_unrecorded_block - first_block_of_the_buffer) * block_bytes)
                .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let end_in_buffer = usize::try_from((end_block - first_block_of_the_buffer) * block_bytes)
            .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let offset = DeviceOffsetInBytes(first_unrecorded_block * block_bytes);
        if let Err(error) = self
            .disk
            .read_at(offset, &mut buffer[start_in_buffer..end_in_buffer])
        {
            self.reads_failed_on_the_disk
                .borrow_mut()
                .push(ReadFailedOnTheDisk {
                    device: self.identity,
                    offset,
                    length_in_bytes: (end_block - first_unrecorded_block) * block_bytes,
                    error: block_device_error_with_the_same_content(&error),
                });
            return Err(error);
        }
        let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
        // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
        for (block_index, block) in (first_unrecorded_block..end_block)
            .zip(buffer[start_in_buffer..end_in_buffer].chunks(block_length))
        {
            recorded.insert(block_index, block.to_vec());
        }
        Ok(())
    }

    /// 穿到盘上读、不取也不改记下的那一份（当场重读那一读）：读错记进读错账、原样交回。
    fn reread_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        if let Err(error) = self.disk.read_at(offset, buffer) {
            self.reads_failed_on_the_disk
                .borrow_mut()
                .push(ReadFailedOnTheDisk {
                    device: self.identity,
                    offset,
                    length_in_bytes,
                    error: block_device_error_with_the_same_content(&error),
                });
            return Err(error);
        }
        Ok(())
    }

    /// 读一段，照 `route` 走（[`ReadRouteOfThisMount`]）。
    fn read_by_the_route(
        &self,
        route: ReadRouteOfThisMount,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length)?;
        match route {
            ReadRouteOfThisMount::StraightToTheDisk => return self.disk.read_at(offset, buffer),
            ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                return self.reread_straight_from_the_disk(offset, buffer)
            }
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun => {}
        }
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length / block_bytes;
        // 迭代上界是这次读的块数；跨轮带着还没读的那一段没记下的块从哪一块起；出口只有底下那块盘读错（原样交回）。
        let mut unrecorded_run_start: Option<u64> = None;
        for block_index in first_block..end_block {
            let is_recorded_in_this_mount = self
                .blocks_read_or_written_in_this_mount
                .borrow()
                .contains_key(&block_index);
            if !is_recorded_in_this_mount {
                unrecorded_run_start.get_or_insert(block_index);
                continue;
            }
            if let Some(run_start) = unrecorded_run_start.take() {
                self.read_unrecorded_blocks_from_the_disk(
                    run_start,
                    block_index,
                    first_block,
                    buffer,
                )?;
            }
            let start_in_buffer = usize::try_from(block_index - first_block)
                .expect("块数装得进 usize")
                * block_length;
            let recorded = self.blocks_read_or_written_in_this_mount.borrow();
            buffer[start_in_buffer..start_in_buffer + block_length].copy_from_slice(
                recorded
                    .get(&block_index)
                    .expect("这一块刚判过记着：判与取之间没有写"),
            );
        }
        if let Some(run_start) = unrecorded_run_start {
            self.read_unrecorded_blocks_from_the_disk(run_start, end_block, first_block, buffer)?;
        }
        Ok(())
    }

    /// 写成的那几块记成写下的字节。
    fn record_the_written_blocks(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        let recorded = self.blocks_read_or_written_in_this_mount.get_mut();
        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
            recorded.insert(first_block + block_offset, block.to_vec());
        }
    }

    /// 写报错的那几块：原来记着的丢掉，之后读它们穿到盘上。
    fn forget_the_blocks_of_a_failed_write(&mut self, offset: DeviceOffsetInBytes, length: u64) {
        let block_bytes = self.block_bytes();
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length / block_bytes;
        let recorded = self.blocks_read_or_written_in_this_mount.get_mut();
        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
        for block_index in first_block..end_block {
            recorded.remove(&block_index);
        }
    }
}

/// 读错记进读错账要一份自己的：盘报的错交回给读的那一方，账里另存一份内容相同的（`BlockDeviceError` 带着 `io::Error`，不能 `Clone`）。
fn block_device_error_with_the_same_content(error: &BlockDeviceError) -> BlockDeviceError {
    match error {
        BlockDeviceError::OutOfRange {
            offset,
            length,
            device_size,
        } => BlockDeviceError::OutOfRange {
            offset: *offset,
            length: *length,
            device_size: *device_size,
        },
        BlockDeviceError::Unaligned {
            offset,
            length,
            physical_block_size,
        } => BlockDeviceError::Unaligned {
            offset: *offset,
            length: *length,
            physical_block_size: *physical_block_size,
        },
        BlockDeviceError::InputOutput(input_output_error) => BlockDeviceError::InputOutput(
            std::io::Error::new(input_output_error.kind(), input_output_error.to_string()),
        ),
        BlockDeviceError::ProbeFailed { attribute } => BlockDeviceError::ProbeFailed { attribute },
        BlockDeviceError::ImageFileAlreadyExists { path } => {
            BlockDeviceError::ImageFileAlreadyExists { path: path.clone() }
        }
        BlockDeviceError::ImageFileMissing { path } => {
            BlockDeviceError::ImageFileMissing { path: path.clone() }
        }
    }
}

impl<Device: BlockDevice> BlockDevice for DeviceReadOnceInThisMount<'_, '_, Device> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_by_the_route(self.read_route.get(), offset, buffer)
    }

    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_by_the_route(
            self.read_route.get().for_the_reread_after_a_bad_read(),
            offset,
            buffer,
        )
    }

    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        match self.read_route.get().for_the_reread_after_a_bad_read() {
            ReadRouteOfThisMount::StraightToTheDisk => {}
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                let length = u64::try_from(bytes.len()).expect("重读的长度装得进 u64");
                assert!(
                    self.check_request(offset, length).is_ok(),
                    "交回来的是一次读成的重读：偏移与长度过了同一份对齐与越界判据"
                );
                let block_bytes = self.block_bytes();
                let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
                let first_block = offset.0 / block_bytes;
                let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
                // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
                for (block_index, block) in (first_block..).zip(bytes.chunks(block_length)) {
                    recorded.insert(block_index, block.to_vec());
                }
            }
        }
    }

    /// 共用的那一段里（当场重读那一读也算）：记进读坏账、丢掉记下的那几块，下一次读它们穿到盘上（坏字节不留）。
    /// 照旧读盘的那几段里读到的字节本来没记下，什么都不做。
    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        match self.read_route.get().for_the_reread_after_a_bad_read() {
            ReadRouteOfThisMount::StraightToTheDisk => {}
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                assert!(
                    self.check_request(offset, length_in_bytes).is_ok(),
                    "报读坏的是一次读成的读：偏移与长度过了同一份对齐与越界判据"
                );
                self.reads_failing_self_verification.borrow_mut().push(
                    ReadFailingSelfVerification {
                        device: self.identity,
                        offset,
                        length_in_bytes,
                    },
                );
                let block_bytes = self.block_bytes();
                let first_block = offset.0 / block_bytes;
                let end_block = first_block + length_in_bytes / block_bytes;
                let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
                // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
                for block_of_the_bad_read in first_block..end_block {
                    recorded.remove(&block_of_the_bad_read);
                }
            }
        }
    }

    /// 不取也不改记下的那一份、直接读底下那块盘；读错不记进读错账（与 [`ReadRouteOfThisMount::StraightToTheDisk`] 那一路同一个读法）。
    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_by_the_route(ReadRouteOfThisMount::StraightToTheDisk, offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        self.check_request(offset, length)?;
        match self.disk.write_at(offset, bytes, durability) {
            Ok(()) => {
                self.record_the_written_blocks(offset, bytes);
                Ok(())
            }
            Err(error) => {
                self.forget_the_blocks_of_a_failed_write(offset, length);
                Err(error)
            }
        }
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        match self.disk.write_zeroes_at(offset, length) {
            Ok(()) => {
                let zeros =
                    vec![0u8; usize::try_from(length).expect("写零的长度装得进 usize")];
                self.record_the_written_blocks(offset, &zeros);
                Ok(())
            }
            Err(error) => {
                self.forget_the_blocks_of_a_failed_write(offset, length);
                Err(error)
            }
        }
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.disk.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.disk.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.disk
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// 被拒的那一次发布之后，这次用户改动还剩什么要发（「被宽护卡住」那一报记一份拷贝，要出路时推完环之后照它再预演一次，
/// [`rehearse_the_push_then_the_remainder_of_the_change`]）。
#[derive(Clone, Copy, Debug)]
enum RemainderOfTheUserChange<'content> {
    /// 先把现行那一版的 `deleted_inodes` 走空、再发这次改动（unlink 连同它之后的走空）：被拒的是走空的一步或这次改动本身。
    WalkTheQueueThenPublishTheChange(UserChange<'content>),
    /// 只剩走空：unlink 的意图已发成，被拒的是它之后走空的一步（别的改动之后队列本来就空）。
    WalkTheQueue,
}

/// [`RemainderOfTheUserChange`] 记进「被宽护卡住」那一份报里的拷贝：错误活得比调用方借给发布的内容长，内容拷一份。
#[derive(Debug)]
enum RemainderOfTheUserChangeKeptByTheRefusal {
    WalkTheQueueThenPublishTheChange(UserChangeKeptByTheRefusal),
    WalkTheQueue,
}

/// [`UserChange`] 的一份拷贝，内容自有。
#[derive(Debug)]
enum UserChangeKeptByTheRefusal {
    Overwrite {
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    SequentialWrite {
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    NewInodes {
        count: u64,
        write_time_seconds: u64,
    },
    Unlink {
        inode: InodeNumber,
        write_time_seconds: u64,
    },
    CreateSnapshot,
    DestroySnapshot {
        snapshot: TreeIdentifier,
    },
    Truncate {
        inode: InodeNumber,
        new_size: u64,
        write_time_seconds: u64,
    },
    OverwriteOfAnInode {
        inode: InodeNumber,
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    SequentialWriteOfAnInode {
        inode: InodeNumber,
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    /// 字节至少一个（从 [`PartialWriteBytes`] 拷来的）。
    WriteAtOffset {
        inode: InodeNumber,
        offset: FileOffsetInBytes,
        bytes: Vec<u8>,
        write_time_seconds: u64,
    },
    /// 整份写的那个文件（号、内容、时间秒）与几次建 inode，内容自有。
    DataChangesPublishedTogether {
        whole_write: Option<(InodeNumber, Vec<u8>, u64)>,
        new_inodes: Vec<NewInodesOfOneChange>,
    },
}

impl UserChangeKeptByTheRefusal {
    fn kept_from(change: UserChange<'_>) -> Self {
        match change {
            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {
                content: file.content.to_vec(),
                write_time_seconds: file.write_time_seconds,
            },
            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {
                content: file.content.to_vec(),
                write_time_seconds: file.write_time_seconds,
            },
            UserChange::NewInodes {
                count,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::NewInodes {
                count,
                write_time_seconds,
            },
            UserChange::Unlink {
                inode,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::Unlink {
                inode,
                write_time_seconds,
            },
            UserChange::CreateSnapshot => UserChangeKeptByTheRefusal::CreateSnapshot,
            UserChange::DestroySnapshot { snapshot } => {
                UserChangeKeptByTheRefusal::DestroySnapshot { snapshot }
            }
            UserChange::Truncate {
                inode,
                new_size,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::Truncate {
                inode,
                new_size,
                write_time_seconds,
            },
            UserChange::OverwriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::OverwriteOfAnInode {
                inode,
                content: content.to_vec(),
                write_time_seconds,
            },
            UserChange::SequentialWriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::SequentialWriteOfAnInode {
                inode,
                content: content.to_vec(),
                write_time_seconds,
            },
            UserChange::WriteAtOffset {
                inode,
                offset,
                bytes,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::WriteAtOffset {
                inode,
                offset,
                bytes: bytes.bytes().to_vec(),
                write_time_seconds,
            },
            UserChange::DataChangesPublishedTogether(batch) => {
                UserChangeKeptByTheRefusal::DataChangesPublishedTogether {
                    whole_write: batch.whole_write.map(|whole_write| {
                        (
                            whole_write.inode,
                            whole_write.content.to_vec(),
                            whole_write.write_time_seconds,
                        )
                    }),
                    new_inodes: batch.new_inodes.to_vec(),
                }
            }
        }
    }

    fn as_user_change(&self) -> UserChange<'_> {
        match self {
            UserChangeKeptByTheRefusal::Overwrite {
                content,
                write_time_seconds,
            } => UserChange::Overwrite(FirstFile {
                content,
                write_time_seconds: *write_time_seconds,
            }),
            UserChangeKeptByTheRefusal::SequentialWrite {
                content,
                write_time_seconds,
            } => UserChange::SequentialWrite(FirstFile {
                content,
                write_time_seconds: *write_time_seconds,
            }),
            UserChangeKeptByTheRefusal::NewInodes {
                count,
                write_time_seconds,
            } => UserChange::NewInodes {
                count: *count,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::Unlink {
                inode,
                write_time_seconds,
            } => UserChange::Unlink {
                inode: *inode,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::CreateSnapshot => UserChange::CreateSnapshot,
            UserChangeKeptByTheRefusal::DestroySnapshot { snapshot } => {
                UserChange::DestroySnapshot {
                    snapshot: *snapshot,
                }
            }
            UserChangeKeptByTheRefusal::Truncate {
                inode,
                new_size,
                write_time_seconds,
            } => UserChange::Truncate {
                inode: *inode,
                new_size: *new_size,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::OverwriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChange::OverwriteOfAnInode {
                inode: *inode,
                content,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::SequentialWriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChange::SequentialWriteOfAnInode {
                inode: *inode,
                content,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::WriteAtOffset {
                inode,
                offset,
                bytes,
                write_time_seconds,
            } => UserChange::WriteAtOffset {
                inode: *inode,
                offset: *offset,
                bytes: PartialWriteBytes::new(bytes)
                    .expect("拷的是一段 PartialWriteBytes：至少一个字节"),
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::DataChangesPublishedTogether {
                whole_write,
                new_inodes,
            } => {
                UserChange::DataChangesPublishedTogether(DataChangesPublishedTogether {
                    whole_write: whole_write.as_ref().map(
                        |(inode, content, write_time_seconds)| WholeWriteOfOneInode {
                            inode: *inode,
                            content,
                            write_time_seconds: *write_time_seconds,
                        },
                    ),
                    new_inodes,
                })
            }
        }
    }
}

impl RemainderOfTheUserChangeKeptByTheRefusal {
    fn kept_from(remainder: RemainderOfTheUserChange<'_>) -> Self {
        match remainder {
            RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(
                    UserChangeKeptByTheRefusal::kept_from(change),
                )
            }
            RemainderOfTheUserChange::WalkTheQueue => {
                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue
            }
        }
    }

    fn as_remainder(&self) -> RemainderOfTheUserChange<'_> {
        match self {
            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(change) => {
                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change.as_user_change())
            }
            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue => {
                RemainderOfTheUserChange::WalkTheQueue
            }
        }
    }
}

/// 每块盘上这一刻隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项那一项），按分配器里盘的次序。只读内存。
fn isolated_slots_of_each_device(allocator: &PoolAllocator) -> Vec<(DeviceIdentity, u64)> {
    allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect()
}

/// 要出路时的预演（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」「改：预演延到要出路时才做」，
/// [`MountedSession::ways_out_of_the_wide_protection`]）：在分配器的一份拷贝上、对只进内存的盘
/// （[`DeviceKeepingWritesInMemory`]）先走一遍推环入口（与真调推环入口同一段，[`MountedSession::push_the_root_ring_on`]：它自己的整串预演、
/// 真发与末尾的断言都在），推完再把这次改动剩下的照会话发布那一段走一遍（`remainder`：准入被拒照样推抬 F，抬 F 的上限按推完的根环算——
/// 读的是盖着推环那几次写的盘）。读盘照读底下那几块盘；写一个都不发到底下那几块盘；会话的分配器与现行那一版不动。
///
/// 预演里的准入再被拒时报不报「被宽护卡住」只读内存（[`wide_protection_at_the_ceiling`] 不预演）；推环做成之后 C 那一圈一槽都不记着
/// （推环入口末尾那条断言），报的是 `NotWhatStopsTheAdmission`。推环在预演里被拒就不往下走。
fn rehearse_the_push_then_the_remainder_of_the_change<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    remainder: RemainderOfTheUserChange<'_>,
) -> RootRingPushRehearsal {
    let mut devices_keeping_writes_in_memory: Vec<(
        DeviceIdentity,
        DeviceKeepingWritesInMemory<'_>,
    )> = devices
        .iter()
        .map(|(identity, device)| {
            let disk: &dyn BlockDevice = device;
            (*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))
        })
        .collect();
    let mut rehearsed_allocator = allocator.clone();
    let mut rehearsed_version = PoolVersion::of_a_version_of_trees(current.clone());
    let isolated_before_the_push = isolated_slots_of_each_device(&rehearsed_allocator);
    if let Err(refusal) = MountedSession::push_the_root_ring_on(
        parameters,
        &mut devices_keeping_writes_in_memory,
        &mut rehearsed_allocator,
        &mut rehearsed_version,
        instance,
    ) {
        return RootRingPushRehearsal::PushRefused(Box::new(refusal));
    }
    let isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)> =
        isolated_before_the_push
            .iter()
            .zip(isolated_slots_of_each_device(&rehearsed_allocator))
            .map(|((device, before), (device_after, after))| {
                assert_eq!(
                    *device, device_after,
                    "推环前后数的是同一个分配器的拷贝：盘的次序不变"
                );
                (*device, before.saturating_sub(after))
            })
            .collect();
    let remainder_after_the_push = match remainder {
        RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
            MountedSession::publish_the_change_raising_the_floor_for_space(
                parameters,
                &mut devices_keeping_writes_in_memory,
                &mut rehearsed_allocator,
                &mut rehearsed_version,
                instance,
                shadow_ledger,
                change,
            )
            .map(|_published| ())
        }
        RemainderOfTheUserChange::WalkTheQueue => {
            let mut version_to_walk = rehearsed_version.into_version_of_trees().expect(
                "推环接在有树的一版后面、发的是带文件那一版的空发布（树表 0 条那一版推环入口先拒了），推完还是有树的一版",
            );
            publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                parameters,
                &mut devices_keeping_writes_in_memory,
                &mut rehearsed_allocator,
                &mut version_to_walk,
                instance,
                shadow_ledger,
                RemainderOfTheUserChange::WalkTheQueue,
                PublishedInThisChange {
                    floor_raises: Vec::new(),
                    refusals_that_pushed_the_floor_raises: Vec::new(),
                    versions_published: Vec::new(),
                },
            )
            .map(|_published_in_this_change| ())
        }
    };
    match remainder_after_the_push {
        Ok(()) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
        },
        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
            refusal: Box::new(refusal),
        },
    }
}

/// C 那一圈还没解开、记着落点的那几槽，与从 `current_txg` 起推环要推到哪儿：推环入口（[`MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`]）
/// 与「被宽护卡住」那一报（[`wide_protection_at_the_ceiling`]）读同一份。
struct RootRingPushTarget {
    /// 这几槽，按槽的次序。
    ring_slots: Vec<RootRingSlot>,
    /// 这几槽记着的全部落点。
    placements: Vec<PlacementOnDevice>,
    /// 最后一个轮到的那一槽（R 那一槽）与写掉它要发几次空发布。
    ring_slot_of_r: RootRingSlot,
    publishes_until_the_ring_slot_of_r_is_written: u64,
}

/// [`RootRingPushTarget`]：分配器没装根环表、或 C 那一圈这一刻没有记着落点的槽时 `None`。只读内存。
///
/// 每一槽要几次发布：txg 逐个加一、每次发布写 `target_for_publish(txg)` 那一槽（`root_ring::target_for_publish`），环长 3S 个连续的 txg
/// 里每一槽恰好轮到一次，所以从 `current_txg + 1` 起数到 3S 之内必找到；txg 加上去越过 u64 的那几个当找不到，取满一环 3S
/// （真发时那一次发布自己就按 txg 到顶拒，`PublishError::NextCheckpointTxgPastTheTopOfItsRange`，在推环的预演里先报出来）。
fn root_ring_push_target(
    allocator: &PoolAllocator,
    current_txg: CheckpointTxg,
) -> Option<RootRingPushTarget> {
    let occupancy = allocator.root_ring_occupancy()?;
    let holding = occupancy.ring_slots_holding_placements_named_by_unapplied_journal_records();
    let slots_per_region = occupancy.slots_per_region();
    let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
    let publishes_until_written = |ring_slot: RootRingSlot| -> u64 {
        (1..=ring_length_in_txgs)
            .find(|publishes_ahead| {
                current_txg
                    .0
                    .checked_add(*publishes_ahead)
                    .is_some_and(|txg| {
                        target_for_publish(CheckpointTxg(txg), slots_per_region) == ring_slot
                    })
            })
            .unwrap_or(ring_length_in_txgs)
    };
    let (publishes_until_the_ring_slot_of_r_is_written, ring_slot_of_r) = holding
        .keys()
        .map(|ring_slot| (publishes_until_written(*ring_slot), *ring_slot))
        .max()?;
    Some(RootRingPushTarget {
        ring_slots: holding.keys().copied().collect(),
        placements: holding.into_values().flatten().collect(),
        ring_slot_of_r,
        publishes_until_the_ring_slot_of_r_is_written,
    })
}

/// 推满仍不够时报不报「被宽护卡住」（[`WideProtectionAtTheCeiling`]）：`stop` 是 F 已在上限、C 那一圈还有记着落点的槽、它们这一刻至少隔离着一个槽，
/// 才报；别的都照旧。判这三样与报的那几个数只读分配器（内存）。报的时候不预演（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：
/// 只记下被拒在哪一版上（`current`）与这次改动还剩什么（`remainder`，内容拷一份），调用方要出路时
/// [`MountedSession::ways_out_of_the_wide_protection`] 照它们预演。
fn wide_protection_at_the_ceiling(
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    remainder: RemainderOfTheUserChange<'_>,
    stop: &FloorRaiseStop,
) -> WideProtectionAtTheCeiling {
    match stop {
        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}
        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
        | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {
            return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
        }
    }
    let Some(target) = root_ring_push_target(allocator, current.root.checkpoint_txg) else {
        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
    };
    let isolated_slots_on_each_device = allocator.isolated_slots_covered_by(&target.placements);
    if isolated_slots_on_each_device
        .iter()
        .all(|(_, isolated)| *isolated == 0)
    {
        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
    }
    let checkpoint_txg_of_r = allocator.root_ring_occupancy().and_then(|occupancy| {
        occupancy.checkpoint_txg_of_the_valid_root_recorded_on(target.ring_slot_of_r)
    });
    WideProtectionAtTheCeiling::StuckBehindTheWideProtection(Box::new(
        StuckBehindTheWideProtection {
            ring_slot_of_r: target.ring_slot_of_r,
            checkpoint_txg_of_r,
            empty_publishes_until_the_ring_slot_of_r_is_written: target
                .publishes_until_the_ring_slot_of_r_is_written,
            isolated_slots_on_each_device,
            instance_of_the_version_refused_on: current.root.instance,
            checkpoint_txg_of_the_version_refused_on: current.root.checkpoint_txg,
            remainder_of_the_change: RemainderOfTheUserChangeKeptByTheRefusal::kept_from(remainder),
        },
    ))
}

/// 推环入口（[`MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`]）做成了：
/// 发成的每一版按先后（最后一版是会话的 `current`），与写掉的那几槽（C 那一圈在这几槽记着的落点都已解开）。
#[derive(Debug)]
pub struct RootRingPushed {
    pub versions_published: Vec<TransactionOutput>,
    pub ring_slots_written: Vec<RootRingSlot>,
}

/// 推环入口没做成。成员按调用方要做的决定分：换回挂载时那一份盘表、等冻结着的那次发布重发、换盘表或换回这个池的盘、这一形第一版不支持、不必推、
/// 空间不够推不过去、发布路径的错。
#[derive(Debug)]
pub enum RootRingPushRefused {
    /// 交进来的盘表的设备身份（按次序）与这次挂载核过的那一份不同（多一块、少一块、换了次序、同一个身份交两次）：入口第一道，
    /// 与 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]、[`WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount`]
    /// 同一判（`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`，只比身份），三个入口收盘表同一判
    /// （用户 2026-10-10 弹窗定「也先比次序」）。不核这一判时次序对调的盘表照样推成（写的先后跟着盘表次序变），推完照它再发被拒成盘表那一种，
    /// 三个入口收不收同一份盘表不一致（判决 `research/prompts/m3-batch10z-xy-code-r1-main-verification.md` 第二节 K1-2）。
    /// 在任何读写之前返回：物理块大小没探、一个盘都没读、一个写都没发，盘上逐字节不变、分配器与会话不动。调用方要做的是换回挂载时那一份盘表再推。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 分配器上冻结着一次落盘途中失败、还没原样重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」；它的 txg 与实例代号）：
    /// 推环的第一次空发布本来就会被拒（`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`），而整串预演走的是发布路径落盘之前那一段、
    /// 不判冻结，会在冻结着的分配器拷贝上照样过。比过盘表次序（只比身份、不读盘）之后就拒，在核盘表那一套、预演与任何读写之前：
    /// 盘上逐字节不变、分配器与会话不动。调用方要做的是先让那次发布原样重发（会话这一层没有重发的入口，等进程重开）。
    PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg,
        instance: InstanceGeneration,
    },
    /// 交进来的盘表没过挂着之后收盘表的那一套核（与抬 F 那一串的空发布同一套：身份交了不止一次、参数或盘表与盘上择到的系统配置不一致、
    /// 有盘不「可见」、有盘落后于现行那一版又缺它的单元或它的根；系统配置择不出来），原样带着那一种 `MountError`。
    /// 身份与次序已由入口第一道比过（`DeviceTableOtherThanTheOneOfTheMount`；要出路时的预演由要出路那一道比过），到这里的盘表身份与挂载时那一份
    /// 逐项相同，身份交了不止一次那一支走不到；
    /// 盘体对调（身份照旧、盘上的本盘设备号对不上）这类读盘才看得出的落在这里。一个写都没发，盘上逐字节不变、会话不动。
    DevicesHandedIn(Box<MountError>),
    /// 现行那一版树表 0 条：推环的空发布在这一版上是零单元发布（D16（发布语义） 已定项 9），会话层的推环在这一形上条款没写，第一版不支持。
    /// 会话在这一版上也不会被宽护卡住（用户改动在这一版上一律 `NoFileVersionToChange`）。在任何读写之前返回，盘上逐字节不变、会话不动。
    VersionWithoutFileNotSupportedInTheFirstVersion,
    /// C 那一圈这一刻没有记着落点的槽，或记着落点而它们罩着的槽这一刻一个都没隔离（全在现行账里仍分配、或已释放还在 defer 窗口里）：
    /// 这个进程写掉哪一槽都放不出东西，不推。一个写都没发，盘上逐字节不变、会话不动。
    NothingHeldUntilThisProcessWritesARingSlot,
    /// 整串空发布在任何写之前的预演里（分配器的一份拷贝上走发布路径落盘之前那一段）第 `refused_publish_in_the_push` 次（从 1 数）被拒：
    /// 推到半路空发布取不到落点（`PublishError::PlacementRefused`）就是这里。一个写都没发，盘上逐字节不变、分配器与会话不动。
    RefusedByTheRehearsalBeforeAnyWrite {
        refused_publish_in_the_push: usize,
        publishes_in_the_push: usize,
        cause: PublishError,
    },
    /// 预演过了、真发时有一次发不出去（块设备错这一类：落盘途中失败的那一次冻结在分配器上，之后再调入口拒成
    /// `PublishFrozenAfterAWriteFailureIsNotResentYet`；冻结着时入口在预演之前就拒，不进预演，所以不在这里。空间上预演与真发只在
    /// 读盘核换下的那一份对不上时分叉）：[`RootRingPushFailedAfterTheRehearsal`]。
    PublishFailedAfterTheRehearsal(Box<RootRingPushFailedAfterTheRehearsal>),
}

/// 推环真发时第 `failed_publish_in_the_push` 次（从 1 数）发不出去：那一次没成立（分配器换回它之前，发布路径的做法），
/// 前面那几次已落盘、在 `versions_published` 里，会话的 `current` 停在最后一次落盘的那一版上。
#[derive(Debug)]
pub struct RootRingPushFailedAfterTheRehearsal {
    pub failed_publish_in_the_push: usize,
    pub publishes_in_the_push: usize,
    pub cause: PublishError,
    pub versions_published: Vec<TransactionOutput>,
}

/// 为空间不够推抬 F 时抬 F 自己报了错：`cause` 是抬 F 的错原样（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`），
/// 这一串是因为 `refusal_that_pushed_the_failed_raise` 那一次被拒才推的；之前推成的几串在 `floor_raises`（已落盘，
/// 每一串因为 `refusals_that_pushed_the_floor_raises` 里对应的那一次被拒才推）。报错那一串落盘途中失败时（`MountError::RaiseFloorSequencePublishFailed`），
/// 它前面已落盘的几次在会话的现行版本里、不在 `floor_raises` 里。
#[derive(Debug)]
pub struct FloorRaiseFailedWhilePushingForSpace {
    pub cause: MountError,
    pub refusal_that_pushed_the_failed_raise: PublishError,
    pub floor_raises: Vec<RaisedFloor>,
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    /// 报错之前发成的那几版（[`VersionsPublishedInThisChange`]）。
    pub versions_published: VersionsPublishedInThisChange,
}

/// 这一次发布被拒是不是空间不够的那两种（D16（发布语义） 已定项 1「准入」那一行：准入不够，或准入放行而落点取不到）——
/// 是就推一串抬 F 再重判，不是就原样交回。
#[must_use]
pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
    match refusal {
        PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,
        // 合取 7（全空聚簇段少于 `N₀`）：全空段数按记录口径数（`allocator::DeviceFreeMap::empty_segments`，defer 里已释放的槽本来就不算占用），
        // 抬 F 把 defer 里的槽回收不改这个数，推了只多发几次空发布；原样交回这个成员，调用方分得出「先整理」与「真没空间」。
        PublishError::EmptyClusterSegmentsBelowTheFloor(_)
        | PublishError::MultiLevelCodeTwoTreeRefused { .. }
        | PublishError::ReleaseNotInMapping { .. }
        | PublishError::ReleaseTargetNotAllocated { .. }
        | PublishError::ReleaseTargetAlreadyReleased { .. }
        | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
        | PublishError::ReleaseSpanMismatch { .. }
        | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
        | PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. }
        | PublishError::MappingEntryLocationsOnTheSameDevice { .. }
        | PublishError::ContentExceedsDataUnit { .. }
        | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
        | PublishError::InodeTreeWriteRefused(_)
        | PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. }
        | PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. }
        | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
        | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
        | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        // 一次发布切出来的记录多于上限：推抬 F 腾不出记录的位置，要把改动切小（代码审阅第 26 条）；txg 到顶是坏镜像（第 36 条）。
        | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
        | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        // 墓碑树、谱系树或退役树最右那片的容器号已到顶（C841）：坏镜像，推抬 F 不会让号变小。
        | PublishError::NextContainerNumberPastTheTopOfItsRange(_)
        // 写删除中的 inode、写已经删完的 inode、unlink 一个不是活记录的号：推抬 F 也不会让它复活。
        | PublishError::WriteToAnInodeBeingDeleted { .. }
        | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
        // 删完第一个文件之后给别的 inode 写数据（K-U5，第一版不支持）：推抬 F 也不会让第一个文件回来。
        | PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. }
        | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
        // `deleted_inodes` 装不下：类型化错误（D8（核心索引结构） 已定项 16 第 6 条），与空间无关。
        | PublishError::DeletedInodesQueueRefused(_)
        // 快照已到第一版的上限、树 ID 水位到顶：推抬 F 回收不出快照的名额，也抬不低水位。
        | PublishError::SnapshotCreationRefused(_)
        // 删快照点名的快照不在、正在删、已有别的在删：推抬 F 改变不了树表。deadlist 条目留存的映射条目没有、落点核不过：盘上的事，与空间无关。
        | PublishError::SnapshotDestructionRefused(_)
        | PublishError::DeadlistEntryNotReleasable { .. }
        // 改 size 的对象不是活记录、目标 size 溢出、条款没写的那一格、尾单元读不出：推抬 F 改变不了。
        | PublishError::TruncationRefused(_)
        | PublishError::BlockDevice(_)
        // 取号之前预演读错、读坏那两拒只在可写挂载取号之前那一判里造，发布路径不报它们。
        | PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
        | PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(_) => false,
    }
}

/// 在一个新开的写入口上把现行那一版的 `deleted_inodes` 走空（D8（核心索引结构） 已定项 16 第 6 条：会话里每一次用户改动先走空），
/// 每发成一步 `current` 就换成那一版——中途失败时 `current` 停在最后一次落盘的那一版上。批大小与墓碑容器容量取分配器上装的开关。
fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    instance: InstanceGeneration,
) -> DeletionQueueWalk {
    let switches: DeletionSwitches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    walk_the_deletion_queue_until_it_is_empty(
        &mut writer,
        allocator,
        current,
        switches.batch_size,
        instance,
    )
}

/// 在一个新开的写入口上发一次这个改动，接在 `previous` 后面。unlink 这里只发意图那一次，全部批由调用方接着走空。
fn publish_the_change_once<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    change: UserChange<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let switches: DeletionSwitches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    match change {
        UserChange::Unlink {
            inode,
            write_time_seconds,
        } => publish_unlink_intent(
            &mut writer,
            allocator,
            previous,
            inode,
            write_time_seconds,
            instance,
        ),
        UserChange::Overwrite(file) => {
            publish_overwrite(&mut writer, allocator, previous, file, instance)
        }
        UserChange::SequentialWrite(file) => {
            publish_sequential_write(&mut writer, allocator, previous, file, instance)
        }
        UserChange::NewInodes {
            count,
            write_time_seconds,
        } => publish_new_inodes(
            &mut writer,
            allocator,
            previous,
            count,
            write_time_seconds,
            instance,
        ),
        UserChange::CreateSnapshot => {
            publish_create_snapshot(&mut writer, allocator, previous, instance)
        }
        UserChange::DestroySnapshot { snapshot } => publish_snapshot_destruction_intent(
            &mut writer,
            allocator,
            previous,
            snapshot,
            instance,
        ),
        UserChange::Truncate {
            inode,
            new_size,
            write_time_seconds,
        } => publish_size_change(
            &mut writer,
            allocator,
            previous,
            inode,
            new_size,
            write_time_seconds,
            instance,
        ),
        UserChange::OverwriteOfAnInode {
            inode,
            content,
            write_time_seconds,
        } => publish_overwrite_of_an_inode(
            &mut writer,
            allocator,
            previous,
            inode,
            content,
            write_time_seconds,
            instance,
        ),
        UserChange::SequentialWriteOfAnInode {
            inode,
            content,
            write_time_seconds,
        } => publish_sequential_write_of_an_inode(
            &mut writer,
            allocator,
            previous,
            inode,
            content,
            write_time_seconds,
            instance,
        ),
        UserChange::WriteAtOffset {
            inode,
            offset,
            bytes,
            write_time_seconds,
        } => publish_partial_write(
            &mut writer,
            allocator,
            previous,
            inode,
            offset,
            bytes,
            write_time_seconds,
            instance,
        ),
        UserChange::DataChangesPublishedTogether(batch) => publish_data_changes_published_together(
            &mut writer,
            allocator,
            previous,
            batch,
            instance,
        ),
    }
}

impl MountedSession {
    /// 一次可写挂载做成之后的会话：交回挂载写出的东西与接着它的会话。
    #[must_use]
    pub fn of_the_mount(mounted: Mounted) -> (MountOutput, MountedSession) {
        let Mounted {
            output,
            allocator,
            current,
        } = mounted;
        let session = MountedSession {
            allocator,
            current,
            instance: output.instance,
            shadow_ledger: output.shadow_ledger,
            parameters_and_device_table: output.parameters_and_device_table.clone(),
            compaction: output.compaction_continuation.state_of_this_mount.clone(),
        };
        (output, session)
    }

    /// 发布一次用户改动：被拒是空间不够的那两种（[`refusal_is_short_of_space`]）就推一串抬 F 的空发布、再重判，
    /// 直到发成或不再推（D16（发布语义） 已定项 1「准入」那一行）。发成时 `current` 换成新的一版；推了的那几串已落盘，
    /// 没发成时 `current` 停在最后一次落盘的空发布上。写入口只按会话持着的那份参数建（`parameters_and_device_table`），
    /// 交进来的盘表先与它逐项比，再读每块盘的两个系统配置槽核它「可见」（Z3-A 乙），再核本盘设备号与落后支
    /// （D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」：「每次发布」按「每次用户改动」读）。
    /// 三样在进循环之前核一次：一次调用里盘表借着、换不了盘；循环里推的每一串抬 F 在它自己的第一个写之前再核一遍三样
    /// （`mount::push_one_floor_raise_within_the_admission_budget` 经 `caller_inputs_agreeing_with_the_disk`），
    /// 所以这次调用发出的每一个写之前都核过。
    ///
    /// 循环：每一轮发一次；被拒是空间不够就推一串（至少一次空发布、推的次数增加）进下一轮，不推了就交回。
    /// 预算把推的次数压在 B − 1 以内 ⇒ 至多 8 轮；跨轮携带的是现行那一版（抬 F 就地推进它）与已推的几串。
    ///
    /// 先把现行那一版的 `deleted_inodes` 走空（每一次用户改动都先走空，D8（核心索引结构） 已定项 16 第 6 条），再发这次改动；
    /// unlink 发成意图之后接着把它的全部批走完。走空里的删除类发布（不判式子那一判、落得下那一判照判，D28（挂载期承诺量） 已定项 1）
    /// 被拒是空间不够时同样推一串抬 F、回收之后接着走（用户 2026-10-02 定案 Q-V「推抬 F 再试」，每一次被拒的删除类发布各算一次准入），
    /// 推满仍不够交回 `NoSpaceAfterRaisingTheFloor`；推的那几串与意图那次推的一起报在 `floor_raises` 里。
    /// 删除那几次发布的两个只供测试的开关取分配器上装的（可写挂载的开关链装上去的，`unlink::DeletionSwitches`）。
    /// 每发成一次（走空的每一步、这次改动）`current` 都按 `PoolVersion::of_a_version_of_trees` 重新归成员；现行那一版是有树而文件集合为空的
    /// 一版时只放建 inode（D8（核心索引结构） 已定项 16 第 5 条），别的改动拒成 `NoFileVersionToChange`。
    /// 发成之后查一次整理的起跑判据，为真只记成「该跑了」（`compaction_plan::CompactionStartJudgement::DueAfterAUserChange`），
    /// 返回之前一批整理都不发（整理挪出了调用线程，用户 2026-10-26 定；后台入口 [`Self::run_compaction_in_the_background`]）。
    ///
    /// # Errors
    /// [`UserChangeRefused`] 的八种。
    pub fn publish_user_change<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        change: UserChange<'_>,
    ) -> Result<UserChangePublished, UserChangeRefused> {
        let MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger,
            parameters_and_device_table,
            compaction,
        } = self;
        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
            return Err(UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        let parameters = parameters_and_device_table.parameters_on_disk();
        let current_version_of_trees = match &*current {
            PoolVersion::WithFile(current_version_of_trees)
            | PoolVersion::WithTreesAndNoFile(current_version_of_trees) => current_version_of_trees,
            PoolVersion::WithoutFile(_) => {
                return Err(no_file_version_to_change(PublishedInThisChange {
                    floor_raises: Vec::new(),
                    refusals_that_pushed_the_floor_raises: Vec::new(),
                    versions_published: Vec::new(),
                }))
            }
        };
        let devices_not_visible =
            devices_without_a_self_verified_system_configuration(devices.as_slice(), parameters);
        if !devices_not_visible.is_empty() {
            return Err(
                UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration {
                    devices: devices_not_visible,
                },
            );
        }
        let own_device_numbers_differing =
            own_device_numbers_on_disk_differing_from_the_identities_handed_in(
                devices.as_slice(),
                parameters,
            );
        if !own_device_numbers_differing.is_empty() {
            return Err(
                UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable {
                    disagreements: own_device_numbers_differing,
                },
            );
        }
        let devices_behind = devices_behind_the_current_version(
            devices.as_slice(),
            parameters,
            current_version_of_trees,
        );
        if !devices_behind.is_empty() {
            return Err(
                UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    devices: devices_behind,
                },
            );
        }
        let mut published = Self::publish_the_change_raising_the_floor_for_space(
            parameters,
            devices,
            allocator,
            current,
            *instance,
            *shadow_ledger,
            change,
        )?;
        // 改动发成：整理「已经过去的发布数」从这里重新数（判决 G8 `B_gc`）。起跑判据照旧在这里查一次（判决 G8 丁），整理却挪出了用户改动的
        // 调用线程（用户 2026-10-26 定「挪出调用线程」；D24（后台重活能不能卸给 GPU） 已定项 2 第 2 条「不许进前台路径」）：这里只把「该跑了」
        // 记在会话上，一批都不发，由后台入口 [`Self::run_compaction_in_the_background`] 在调用方自己的线程上去跑。这一次调用拿着 `&mut self`
        // 从头到尾，为这次改动推的抬 F 与之后的再判之间插不进整理批（第三轮判决第二节 G8 的第三样保证）。
        compaction.publishes_since_the_last_user_change = 0;
        // 改动发成：清掉会话上跨趟留着的上一条完成的意图（用户 2026-10-26 定「每次用户改动发成就清掉」，
        // `records/2026-10-19-里程碑四GPU垃圾回收开工.md`「调查员交回」那一行）。D26（后台整理与放置回收） 已定项 13 第 6 条
        // 「签下一条意图那一判跨趟留着」只留到下一次用户改动发成为止：池子一直被写满时全空段数只降不升，留到回滚或重挂会让整理一直停着；
        // 下一趟从空起照常签新意图，预算照旧是这一次改动给的那一份 `B_gc`（岔路单第 18 行甲）。被拒、没发成的改动在上面 `?` 交回，这一条照旧留着。
        compaction.last_completed_intent = None;
        if compaction_starts_by_the_watermarks(allocator.compaction().switches.run_forcing) {
            compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
        }
        published.discards_after_the_change = discard_the_segments_reusable_as_a_whole(
            devices.as_mut_slice(),
            allocator,
            parameters,
            current.root(),
        );
        Ok(published)
    }

    /// 整理的显式入口（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G8 甲）：先把删除续做走空，再一次一次推整理意图——
    /// 没有活意图就选段、签发前判、发一次把意图写进 logged_ops 的发布（判决 G1）；有就对 R 重扫（不复用上一批的扫描，丁 1 交判第 3 点）、
    /// R 里有隔离槽就发一次删意图的发布丢掉它（攻方 K-c）、计划一批、发这一批，最后一批同一次发布删意图（D8（核心索引结构） 已定项 5）；
    /// 全局差值为假就不签下一条。这一趟至多发 `B_gc` 次整理发布（[`publish_budget_of_a_compaction_run`]，签发、搬一批、删意图各算一次）；
    /// 一次整理发布被空间拒就暂停（撤 R 排他、意图留在盘上）、不推抬 F。发布都返回之后发 discard（判决 G5）。
    /// 批大小条款没写，产品路径在签任何意图之前停下（`CompactionRunStop::BatchSizeNotDecidedByTheClauses`），只供测试的开关给出批大小才推得动。
    ///
    /// # Errors
    /// 盘表与挂载核过的不同、现行那一版树表 0 条 ⇒ [`CompactionRunRefused`]，在任何读写之前。
    pub fn run_compaction<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<CompactionRun, CompactionRunRefused> {
        self.run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region(
            devices,
            RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        )
    }

    /// 同 [`Self::run_compaction`]，只差一处：一批照抄意图核出腾空了 R、照同一批删意图再发时，删意图那一支落盘之前的核照 `region_check` 判
    /// （判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2）。别处删意图（最后一批、R 已空的只删意图）照记录核，不读它。
    /// 产品路径只经 [`Self::run_compaction`] 进来（[`RegionCheckOfTheIntentDeletion::JudgedByTheRecords`]）；只供测试的
    /// [`RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`] 强制进入「删意图那一支核不过、照这一批不删意图再发」
    /// 那一支（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：发出去的那一版 R 按记录全空、意图还在，下一次发布照「R 已按记录全空」只删意图。
    ///
    /// # Errors
    /// 同 [`Self::run_compaction`]。
    pub fn run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        region_check: RegionCheckOfTheIntentDeletion,
    ) -> Result<CompactionRun, CompactionRunRefused> {
        self.run_compaction_with_the_region_check_and_a_hook_in_the_middle_of_each_batch(
            devices,
            region_check,
            InTheMiddleOfEachBackgroundBatch::NothingHappens,
        )
    }

    /// 同 [`Self::run_compaction`]，只差一处：整理的每一批读完后台计算设置、开算之前调用例给的钩子（只供测试，
    /// `.claude/rules/fs-design.md`「五条硬要求」第 2 条）。钩子拿到这次挂载的后台计算状态，在这一刻请求切换，强制进入
    /// 「一批跑到一半时请求切换：这一批照旧、下一批才换」那一格（D26（后台整理与放置回收） 已定项 13 第 1 条）。产品路径只经
    /// [`Self::run_compaction`] 进来，钩子那一处什么都不做。
    ///
    /// # Errors
    /// 同 [`Self::run_compaction`]。
    pub fn run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        hook_in_the_middle_of_each_batch: &mut dyn FnMut(&mut BackgroundComputeOfThisMount),
    ) -> Result<CompactionRun, CompactionRunRefused> {
        self.run_compaction_with_the_region_check_and_a_hook_in_the_middle_of_each_batch(
            devices,
            RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
            InTheMiddleOfEachBackgroundBatch::CallTheTestOnlyHook(hook_in_the_middle_of_each_batch),
        )
    }

    /// 后台计算的设置在这次挂载里的状态：生效的那一组与待生效的切换。
    #[must_use]
    pub fn background_compute(&self) -> &BackgroundComputeOfThisMount {
        self.parameters_and_device_table
            .background_compute_of_this_mount()
    }

    /// 挂着切换后台计算的设置（D26（后台整理与放置回收） 已定项 13 第 1 条：一次切全部后台任务，从下一批开始生效、一批之内不换）：
    /// 与挂载同一套校验（第 4 条，`background_compute::admit_background_compute_settings_with_the_gpu_compute`；`gpu_compute` 是这一次递进来的 GPU 实现，
    /// `None` 是沿用这次挂载已有的那一份，都没有而切到 `gpu` 就是 GPU 不在）；
    /// 过了就记成待生效，下一批开头起用（整理的每一批在 [`advance_compaction_by_one_publish`] 进门读一次）。不读写盘。
    ///
    /// # Errors
    /// 校验不过：拒绝切换，生效的与待生效的设置都留着原样，不降级到 `cpu`。
    pub fn switch_background_compute_from_the_next_batch(
        &mut self,
        background_compute_settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
    ) -> Result<(), BackgroundComputeSettingsRefused> {
        self.parameters_and_device_table
            .background_compute_of_this_mount_mut()
            .request_switch_from_the_next_batch(background_compute_settings, gpu_compute)
    }

    /// 显式入口的三种起法共用的那一段：盘表核过、现行那一版有树，就推一趟、发 discard。
    fn run_compaction_with_the_region_check_and_a_hook_in_the_middle_of_each_batch<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        region_check: RegionCheckOfTheIntentDeletion,
        mut in_the_middle_of_each_batch: InTheMiddleOfEachBackgroundBatch<'_>,
    ) -> Result<CompactionRun, CompactionRunRefused> {
        let MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction,
        } = self;
        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
            return Err(CompactionRunRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        match &*current {
            PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
            PoolVersion::WithoutFile(_) => return Err(CompactionRunRefused::NoVersionOfTrees),
        }
        let (parameters, background_compute) = parameters_and_device_table
            .parameters_on_disk_and_the_background_compute_of_this_mount_mut();
        let mut run = run_compaction_on_with_the_region_check_of_the_intent_deletion(
            parameters,
            devices.as_mut_slice(),
            allocator,
            compaction,
            current,
            *instance,
            CompactionRunReach::UntilAStopOrTheBudget,
            CompactionRunBudget::SinceTheLastUserChange,
            PacingOfTheRun::NotPaced,
            region_check,
            background_compute,
            &mut in_the_middle_of_each_batch,
        );
        run.discards = discard_the_segments_reusable_as_a_whole(
            devices.as_mut_slice(),
            allocator,
            parameters,
            current.root(),
        );
        Ok(run)
    }

    /// 整理的后台入口（D26（后台整理与放置回收） 已定项 13 第 5、6 条；用户 2026-10-26 定 GC「挪出调用线程，补齐三样保证」）：调用方在它自己的线程上
    /// 一次次调，核心层不起线程、不读钟（钟与前台的 fsync 由 `caller` 答）。每次调用先看起不起跑，依次：
    /// 1. 整理因一批出错停着（第 4 条）⇒ 不起跑，「该跑了」留着；
    /// 2. 用户改动之后起跑判据记下了「该跑了」⇒ 取走，照用户改动之后那一趟推（判决 G8 丁：至多一批）；
    /// 3. 没有 ⇒ 看定时兜底（`pacing.timer_fallback`）：关着、或离上一次查还不到一个间隔就不起跑；到了就再查一次起跑判据，为真就推一趟到停；
    /// 4. 用户改动之后那一趟与定时兜底那一趟都从上一次用户改动给的那一份 `B_gc` 里扣（岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲，
    ///    用户 2026-10-26 定「与用户改动共用 `B_gc`」；[`publishes_left_of_the_budget_given_by_the_last_user_change`]）：那一份扣完就不起跑
    ///    （[`BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent`]），等下一次用户改动才再有。闲置时整理发布因此合计不过
    ///    一份 `B_gc`，不随定时兜底查了几次涨。
    ///
    /// 起跑之后每一批之前问调用方：前台有 fsync 在等就不发下一批（[`CompactionRunStop::YieldedToAForegroundFsync`]），这一个写带宽窗口里
    /// 整理写的字节到了 `pacing.write_bandwidth_limit` 也不发（[`CompactionRunStop::WriteBandwidthLimitReached`]）；用户改动之后起跑的那一趟
    /// 停在这两样上时「该跑了」放回去，下一次调用接着推（不发下一批是缓发，不是做完）。签下一条意图那一判
    /// 读会话上跨趟留着的上一条完成的意图（`compaction_plan::CompactionOfThisMount::last_completed_intent`）。发布都返回之后发 discard（判决 G5）。
    ///
    /// # Errors
    /// 盘表与挂载核过的不同、现行那一版树表 0 条 ⇒ [`CompactionRunRefused`]，在任何读写之前（「该跑了」留着）。
    pub fn run_compaction_in_the_background<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
    ) -> Result<BackgroundCompactionCall, CompactionRunRefused> {
        match self.whether_the_background_compaction_starts(devices, pacing, caller)? {
            BackgroundCompactionStart::NotStarted(not_started) => {
                Ok(BackgroundCompactionCall::NotStarted(not_started))
            }
            BackgroundCompactionStart::Starts {
                trigger,
                reach,
                budget,
            } => Ok(self.run_the_background_compaction_that_starts(
                devices, pacing, caller, trigger, reach, budget,
            )),
        }
    }

    /// [`Self::run_compaction_in_the_background`] 开头那几判：盘表、有没有树、整理停没停、有没有「该跑了」、定时兜底、上一次用户改动给的那一份
    /// `B_gc` 还剩不剩。取走「该跑了」与定时兜底记下这一次查的读数都在这里做，一个盘都不读、一个写都不发。攒批的会话
    /// （[`MountedSessionWithPendingChanges::run_compaction_in_the_background`]）在这一判说起跑之后、跑之前先排空待发。
    ///
    /// # Errors
    /// 同 [`Self::run_compaction_in_the_background`]。
    fn whether_the_background_compaction_starts<Device: BlockDevice>(
        &mut self,
        devices: &[(DeviceIdentity, Device)],
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
    ) -> Result<BackgroundCompactionStart, CompactionRunRefused> {
        let MountedSession {
            allocator,
            current,
            instance: _,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction,
        } = self;
        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
            return Err(CompactionRunRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        match &*current {
            PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
            PoolVersion::WithoutFile(_) => return Err(CompactionRunRefused::NoVersionOfTrees),
        }
        let parameters = parameters_and_device_table.parameters_on_disk();
        let background_compute = parameters_and_device_table.background_compute_of_this_mount();
        if background_compute.is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation)
        {
            return Ok(BackgroundCompactionStart::NotStarted(
                BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
            ));
        }
        let (trigger, reach) = match compaction.start_judgement {
            CompactionStartJudgement::DueAfterAUserChange => {
                compaction.start_judgement = CompactionStartJudgement::NotDue;
                (
                    BackgroundCompactionTrigger::DueAfterAUserChange,
                    CompactionRunReach::AtMostOneBatchAfterAUserChange,
                )
            }
            CompactionStartJudgement::NotDue => {
                match compaction
                    .check_the_timer_fallback(pacing.timer_fallback, caller.clock_reading())
                {
                    TimerFallbackCheck::Off => {
                        return Ok(BackgroundCompactionStart::NotStarted(
                            BackgroundCompactionNotStarted::NothingDueAndTheTimerFallbackOff,
                        ))
                    }
                    TimerFallbackCheck::IntervalNotElapsed => {
                        return Ok(BackgroundCompactionStart::NotStarted(
                            BackgroundCompactionNotStarted::TimerFallbackIntervalNotElapsed,
                        ))
                    }
                    TimerFallbackCheck::Due => {}
                }
                if !compaction_starts_by_the_watermarks(allocator.compaction().switches.run_forcing)
                {
                    return Ok(BackgroundCompactionStart::NotStarted(
                        BackgroundCompactionNotStarted::NotDueByTheWatermarksAtTheTimerFallback,
                    ));
                }
                (
                    BackgroundCompactionTrigger::TimerFallback,
                    CompactionRunReach::UntilAStopOrTheBudget,
                )
            }
        };
        // 两种起跑共用上一次用户改动给的那一份 `B_gc`：扣完就不起跑，等下一次用户改动。
        if publishes_left_of_the_budget_given_by_the_last_user_change(
            parameters.geometry.root_ring_slots_per_region.count(),
            compaction.publishes_since_the_last_user_change,
        ) == 0
        {
            return Ok(BackgroundCompactionStart::NotStarted(
                BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent,
            ));
        }
        Ok(BackgroundCompactionStart::Starts {
            trigger,
            reach,
            budget: CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange,
        })
    }

    /// [`Self::run_compaction_in_the_background`] 判过起跑之后那一段：推一趟（每一批之前问调用方）、节奏截住时把「该跑了」放回去、发 discard。
    fn run_the_background_compaction_that_starts<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
        trigger: BackgroundCompactionTrigger,
        reach: CompactionRunReach,
        budget: CompactionRunBudget,
    ) -> BackgroundCompactionCall {
        let MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction,
        } = self;
        let (parameters, background_compute) = parameters_and_device_table
            .parameters_on_disk_and_the_background_compute_of_this_mount_mut();
        let mut run = run_compaction_on_with_the_region_check_of_the_intent_deletion(
            parameters,
            devices.as_mut_slice(),
            allocator,
            compaction,
            current,
            *instance,
            reach,
            budget,
            PacingOfTheRun::PacedByTheCaller {
                write_bandwidth_limit: pacing.write_bandwidth_limit,
                caller,
            },
            RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
            background_compute,
            &mut InTheMiddleOfEachBackgroundBatch::NothingHappens,
        );
        // 节奏让下一批缓发（前台 fsync 在等、写带宽到了上限）不是做完：用户改动之后那一趟被它截住时「该跑了」放回去，下一次调用接着推。
        let is_held_back_by_the_pacing = matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::YieldedToAForegroundFsync
                    | CompactionRunStop::WriteBandwidthLimitReached { .. }
            )
        );
        match trigger {
            BackgroundCompactionTrigger::DueAfterAUserChange if is_held_back_by_the_pacing => {
                compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
            }
            BackgroundCompactionTrigger::DueAfterAUserChange
            | BackgroundCompactionTrigger::TimerFallback => {}
        }
        run.discards = discard_the_segments_reusable_as_a_whole(
            devices.as_mut_slice(),
            allocator,
            parameters,
            current.root(),
        );
        BackgroundCompactionCall::Ran {
            trigger,
            run: Box::new(run),
        }
    }

    /// [`Self::publish_user_change`] 核过盘表之后那一半：先走空、再发这次改动（unlink 连同它之后的走空），被拒是空间不够就推抬 F 再判。
    /// 要出路时的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]），
    /// 所以拆成不收 `self` 的一段。
    fn publish_the_change_raising_the_floor_for_space<Device: BlockDevice>(
        parameters: &MakeFilesystemParameters,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        allocator: &mut PoolAllocator,
        current: &mut PoolVersion,
        instance: InstanceGeneration,
        shadow_ledger: ShadowLedger,
        change: UserChange<'_>,
    ) -> Result<UserChangePublished, UserChangeRefused> {
        let current_version_of_trees = match &*current {
            PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees,
            PoolVersion::WithoutFile(_) => {
                return Err(no_file_version_to_change(PublishedInThisChange {
                    floor_raises: Vec::new(),
                    refusals_that_pushed_the_floor_raises: Vec::new(),
                    versions_published: Vec::new(),
                }))
            }
        };
        let mut published_in_this_change = PublishedInThisChange {
            floor_raises: Vec::new(),
            refusals_that_pushed_the_floor_raises: Vec::new(),
            versions_published: Vec::new(),
        };
        // 队列空（任何一版都是常态）时一个写都不发、不复制；不空时在一份复制上走，走到哪一步（连同推的抬 F）`current` 就是哪一版。
        // 销毁中的快照（判决 S4：删快照照 unlink 的过程做，第七节岔路 4）、截断条目（判决 m4-trunc-r1 K1）与 `deleted_inodes` 一起走空。
        if !current_version_of_trees.queue_of_committed_intents_is_empty() {
            let mut walked = current_version_of_trees.clone();
            let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                parameters,
                devices,
                allocator,
                &mut walked,
                instance,
                shadow_ledger,
                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                published_in_this_change,
            );
            *current = PoolVersion::of_a_version_of_trees(walked);
            published_in_this_change = walk?;
        }
        let mut publishes_pushed = 0;
        // 按成员分流（类型上放什么）：有树而文件集合为空的一版只接建 inode。走空把带文件的一版走成这一形时，走空的那几步与推的抬 F
        // 已落盘，带在这个拒绝里（`NoFileVersionToChange` 的三样）。
        let version_to_change = match current {
            PoolVersion::WithFile(file_version) => file_version,
            PoolVersion::WithTreesAndNoFile(emptied_version) => match change {
                UserChange::NewInodes { .. } | UserChange::DestroySnapshot { .. } => {
                    emptied_version
                }
                // 只建 inode 的那一批同建 inode；带文件写的那一批同写文件的改动。
                UserChange::DataChangesPublishedTogether(batch) if batch.whole_write.is_none() => {
                    emptied_version
                }
                UserChange::DataChangesPublishedTogether(_) => {
                    return Err(no_file_version_to_change(published_in_this_change));
                }
                // 文件集合为空时建快照放行、照常发布，读它读到空（用户 2026-10-16 定「放开」，判决 m4-segment-code-r1 第三节 F5）。
                UserChange::CreateSnapshot => emptied_version,
                // 改 size 同写文件的改动：删完之后只许建 inode、写行与空发布（D8（核心索引结构） 已定项 16 第 5 条）。
                // 按 inode 号写同写第一个文件：删空之后的一版上 inode 树一条记录都没有，没有可写的文件。这一判只罩删空那一版本身；
                // 之后建过 inode、归带文件的那几版上给删完之后才建的 inode 写数据，由写入口按 K-U5 的 β 读法拒
                // （`transaction::PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`）。
                UserChange::Overwrite(_)
                | UserChange::SequentialWrite(_)
                | UserChange::OverwriteOfAnInode { .. }
                | UserChange::SequentialWriteOfAnInode { .. }
                | UserChange::WriteAtOffset { .. }
                | UserChange::Unlink { .. }
                | UserChange::Truncate { .. } => {
                    return Err(no_file_version_to_change(published_in_this_change));
                }
            },
            PoolVersion::WithoutFile(_) => {
                return Err(no_file_version_to_change(published_in_this_change))
            }
        };
        loop {
            let refusal = match publish_the_change_once(
                parameters,
                devices.as_mut_slice(),
                allocator,
                version_to_change,
                change,
                instance,
            ) {
                Ok(published) => {
                    // unlink：意图发成之后接着把全部批走完（一次调用 = 走空 + 意图 + 全部批）；别的改动之后队列本来就空，一个写都不发。
                    let mut published = published;
                    published_in_this_change
                        .versions_published
                        .push(published.clone());
                    let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                        parameters,
                        devices,
                        allocator,
                        &mut published,
                        instance,
                        shadow_ledger,
                        RemainderOfTheUserChange::WalkTheQueue,
                        published_in_this_change,
                    );
                    *current = PoolVersion::of_a_version_of_trees(published);
                    let PublishedInThisChange {
                        floor_raises,
                        refusals_that_pushed_the_floor_raises,
                        versions_published,
                    } = walk?;
                    return Ok(UserChangePublished {
                        floor_raises,
                        refusals_that_pushed_the_floor_raises,
                        versions_published,
                        discards_after_the_change: DiscardsAfterThePublishes::default(),
                    });
                }
                Err(refusal) => refusal,
            };
            let publishes_of_this_raise;
            (published_in_this_change, publishes_of_this_raise) =
                push_one_floor_raise_for_a_refusal_for_space(
                    parameters,
                    devices,
                    allocator,
                    version_to_change,
                    shadow_ledger,
                    RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                    publishes_pushed,
                    published_in_this_change,
                    refusal,
                )?;
            publishes_pushed += publishes_of_this_raise;
        }
    }

    /// 推环到 R 那一槽（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」那一个管理入口）：在现行那一版后面接连发 F 不变的空发布
    /// （与可写挂载暖机同一张计划，`mount::empty_publish_plan_after_file_version`：重写固定点单元，不碰用户树、实例表照抄），
    /// 直到这个进程写掉 C 那一圈还没解开、记着落点的每一槽（R 那一槽最后轮到），宽护随之解开（`allocator::PoolAllocator::record_root_written_by_this_process`
    /// 摘下那一槽记着的落点、清掉别处不再护着的隔离位）。要几次见 [`StuckBehindTheWideProtection::empty_publishes_until_the_ring_slot_of_r_is_written`]，至多环长 3S。
    ///
    /// **调用即确认丢掉最近可退到的状态**：R 那一槽是 C 那一圈里最后轮到的一槽，推到它要写掉根环里从现行那一版起接下来轮到的那几槽（次数就是槽数，
    /// 不超过一环），那几槽里原来的根（回退候选）换成同一个用户状态的空发布根；推完之后回退候选集可以掉到少于 4 个不同状态，
    /// D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」只在调这个入口时破
    /// （第 10z 批调查报告 10z-p 第六节第 1 条：384 槽盘那一形推完只剩 2 个）。不自动调：准入被宽护卡住时只报 [`StuckBehindTheWideProtection`]，
    /// 调用方要出路时（[`Self::ways_out_of_the_wide_protection`]）预演过推完之后那次改动放得下才写明这一条出路。
    ///
    /// 第一道照 [`Self::publish_user_change`] 与 [`Self::ways_out_of_the_wide_protection`] 那样，把交进来的盘表与挂载时核过的那一份逐项比
    /// （次序也比，`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`，只比身份、不读盘），不符就拒
    /// （`DeviceTableOtherThanTheOneOfTheMount`，带两份盘表），在任何读写之前：三个入口收盘表同一判（用户 2026-10-10 弹窗定「也先比次序」）。
    /// 这一判在这个公开入口里、不在本体（`push_the_root_ring_on`）里：要出路时的预演经本体走、交的是包过的盘，要出路那一边先核过同一判。
    /// 第二道看分配器上冻结着没重发的发布，冻结着就拒（`PublishFrozenAfterAWriteFailureIsNotResentYet`），在核盘表那一套与预演之前。
    /// C 那一圈记着落点、而它们罩着的槽这一刻一个都没隔离时不推（写掉那几槽放不出东西，`NothingHeldUntilThisProcessWritesARingSlot`），在任何写之前。
    /// 整串先在分配器的一份拷贝上预演（发布路径落盘之前那一段，与抬 F 那一串的预演同一个做法）：推到半路空发布取不到落点（没挡的槽不够这一串）
    /// 时一个写都不发，盘上逐字节不变、分配器与会话不动。预演过了才真发；真发时有一次发不出去，前面那几次已落盘、`current` 停在最后一次落盘的那一版上。
    /// 这些空发布不过准入两判（没有普通分配，D28（挂载期承诺量） 已定项 1），只受「取得到落点」约束。交进来的盘表照抬 F 那一串的
    /// 空发布同一套核（`mount::parameters_of_the_pool_agreeing_with_the_caller_inputs`），在任何写之前；写入口只按盘上那一份参数建。
    ///
    /// # Errors
    /// [`RootRingPushRefused`] 的七种。
    pub fn push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<RootRingPushed, RootRingPushRefused> {
        let MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction: _,
        } = self;
        let is_the_device_table_handed_in_the_one_of_the_mount =
            parameters_and_device_table.is_the_device_table_of_the_mount(devices);
        if !is_the_device_table_handed_in_the_one_of_the_mount {
            return Err(RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        Self::push_the_root_ring_on(
            parameters_and_device_table.parameters_on_disk(),
            devices,
            allocator,
            current,
            *instance,
        )
    }

    /// 被宽护卡住之后调用方要出路时调（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：照 `stuck` 那一份报记着的「这次改动还剩什么」，
    /// 在分配器的一份拷贝上、对只进内存的盘预演推环与推完之后这次改动剩下的发布（[`rehearse_the_push_then_the_remainder_of_the_change`]：
    /// 与真调推环入口、之后走报里剩下的那一段走同一段：报里还剩这次改动时是再发同一个改动；只剩走空时 unlink 的意图已发成，
    /// 是下一次改动开头的走空，不再发这次改动），照结局列出路（推完放得下才列推环）。报 `stuck` 的那一次不付这一笔。
    ///
    /// 收 `&self`：预演只读交进来的盘、在分配器的拷贝上走，会话的分配器、现行那一版一样都不动——类型上就动不了（`PoolAllocator` 里没有内部可变），
    /// 调用方不必为了要出路把会话交出去。盘表只读借：只进内存的盘只读借底下那块盘，写一个都发不到它上面。
    /// 交进来的盘表先照 [`Self::publish_user_change`] 与推环入口那样与挂载时核过的那一份逐项比（次序也比，三个入口同一判），
    /// 再照推环本体那一套核（在预演的推环里，核不过就是推环被拒、出路只列卸载再挂），与照出路二真推、再走报里剩下的那一段时同一个结局
    /// （报里还剩这次改动时是再发这次改动）。
    ///
    /// 第一道看会话还在不在被拒的那一版上（实例代号与 txg 都与报里记的相同），再比盘表：哪一道不过都不预演，一个盘都不读。
    /// 第一道只比这两个数，认不出 `stuck` 是不是这个会话报的：别的会话的报若这两个数恰好与这个会话的现行那一版相同，照样过，
    /// 照这个会话的状态预演（[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。
    /// 调用方要拿报去问报它的那个会话。
    ///
    /// # Errors
    /// [`WaysOutOfTheWideProtectionRefused`] 的两种。
    pub fn ways_out_of_the_wide_protection<Device: BlockDevice>(
        &self,
        devices: &[(DeviceIdentity, Device)],
        stuck: &StuckBehindTheWideProtection,
    ) -> Result<WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused> {
        let is_the_version_refused_on = |version_of_trees: &TransactionOutput| {
            version_of_trees.root.instance == stuck.instance_of_the_version_refused_on
                && version_of_trees.root.checkpoint_txg
                    == stuck.checkpoint_txg_of_the_version_refused_on
        };
        let version_refused_on = match &self.current {
            PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees)
                if is_the_version_refused_on(version_of_trees) =>
            {
                version_of_trees
            }
            PoolVersion::WithFile(_)
            | PoolVersion::WithTreesAndNoFile(_)
            | PoolVersion::WithoutFile(_) => {
                let current_root = self.current.root();
                return Err(
                    WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                        instance_refused_on: stuck.instance_of_the_version_refused_on,
                        checkpoint_txg_refused_on: stuck.checkpoint_txg_of_the_version_refused_on,
                        instance_of_the_current_version: current_root.instance,
                        checkpoint_txg_of_the_current_version: current_root.checkpoint_txg,
                    },
                );
            }
        };
        if !self
            .parameters_and_device_table
            .is_the_device_table_of_the_mount(devices)
        {
            return Err(
                WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount {
                    device_table_of_the_mount: self
                        .parameters_and_device_table
                        .device_identities_in_table_order()
                        .to_vec(),
                    device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
                },
            );
        }
        let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(
            self.parameters_and_device_table.parameters_on_disk(),
            devices,
            &self.allocator,
            version_refused_on,
            self.instance,
            self.shadow_ledger,
            stuck.remainder_of_the_change.as_remainder(),
        );
        let ways_out = stuck.ways_out_after(&pushing_the_root_ring);
        Ok(WaysOutOfTheWideProtection {
            pushing_the_root_ring,
            ways_out,
        })
    }

    /// [`Self::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`] 的本体，盘上择到的参数由调用方给：
    /// 要出路时的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
    ///
    /// # Errors
    /// 同那个入口。
    fn push_the_root_ring_on<Device: BlockDevice>(
        parameters_on_disk: &MakeFilesystemParameters,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        allocator: &mut PoolAllocator,
        current: &mut PoolVersion,
        instance: InstanceGeneration,
    ) -> Result<RootRingPushed, RootRingPushRefused> {
        // D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：冻结着的那次发布等原样重发。下面的整串预演走的是发布路径落盘之前那一段、
        // 不判冻结，在冻结着的分配器拷贝上照样过，真发第一次才被拒（第 10z 批代码轮第一轮攻方第二节「一处报法不准」）。
        if let Some(frozen) = allocator.frozen_publish() {
            return Err(
                RootRingPushRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: frozen.checkpoint_txg(),
                    instance: frozen.instance(),
                },
            );
        }
        let start = match &*current {
            PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees.clone(),
            PoolVersion::WithoutFile(_) => {
                return Err(RootRingPushRefused::VersionWithoutFileNotSupportedInTheFirstVersion);
            }
        };
        let parameters_of_the_pool = parameters_of_the_pool_agreeing_with_the_caller_inputs(
            parameters_on_disk,
            devices,
            &start,
        )
        .map_err(|refusal| RootRingPushRefused::DevicesHandedIn(Box::new(refusal)))?;
        let parameters = &parameters_of_the_pool;
        let Some(target) = root_ring_push_target(allocator, start.root.checkpoint_txg) else {
            return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);
        };
        // C 那一圈记着落点、它们罩着的槽却一个都没隔离（全在现行账里仍分配、或已释放还在 defer 窗口里）：写掉那几槽放不出东西，
        // 与没有记着落点的槽同一个结局（判决 `research/prompts/m3-batch10z-pqrs-code-r1-main-verification.md` 第二节「K1-1 三个量」那一行，正推注入级那一形）。
        let slots_the_push_would_release_isolation_of =
            allocator.isolated_slots_covered_by(&target.placements);
        if slots_the_push_would_release_isolation_of
            .iter()
            .all(|(_, isolated)| *isolated == 0)
        {
            return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);
        }
        let publishes_in_the_push =
            usize::try_from(target.publishes_until_the_ring_slot_of_r_is_written)
                .expect("至多环长 3S ≤ 48 次");
        // 预演：迭代上界是这一串的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）与分配器的拷贝。
        {
            let rehearsal_writer = PoolWriter::new(parameters, devices.as_mut_slice());
            let mut copy = allocator.clone();
            let mut rehearsed = start.clone();
            for publish_index in 0..publishes_in_the_push {
                let (_, next) = prepare_the_version_publish(
                    &rehearsal_writer,
                    &mut copy,
                    &empty_publish_plan_after_file_version(&rehearsed, instance),
                    Some(&rehearsed),
                    rehearsed.tree_identifiers,
                    ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                    UnmountMarker::NotWrittenByTheUnmountSequence,
                    &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
                )
                .map_err(|cause| {
                    RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
                        refused_publish_in_the_push: publish_index + 1,
                        publishes_in_the_push,
                        cause,
                    }
                })?;
                rehearsed = next;
            }
        }
        // 真发：迭代上界同上；跨轮携带的是落盘了的现行那一版与发成的每一版；出口只有发不出去那一次。
        let mut last_published = start;
        let mut versions_published = Vec::with_capacity(publishes_in_the_push);
        for publish_index in 0..publishes_in_the_push {
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            match publish_version(
                &mut writer,
                allocator,
                empty_publish_plan_after_file_version(&last_published, instance),
                Some(&last_published),
            ) {
                Ok(next) => {
                    versions_published.push(next.clone());
                    last_published = next;
                }
                Err(cause) => {
                    *current = PoolVersion::of_a_version_of_trees(last_published);
                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(
                        Box::new(RootRingPushFailedAfterTheRehearsal {
                            failed_publish_in_the_push: publish_index + 1,
                            publishes_in_the_push,
                            cause,
                            versions_published,
                        }),
                    ));
                }
            }
        }
        *current = PoolVersion::of_a_version_of_trees(last_published);
        let still_holding = allocator
            .root_ring_occupancy()
            .expect("推环之前读到了 C 那一圈，装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        assert!(
            target
                .ring_slots
                .iter()
                .all(|ring_slot| !still_holding.contains_key(ring_slot)),
            "推环写到了 R 那一槽：每次发布 txg 加一、写 target_for_publish(txg) 那一槽并在取完落点之后摘下 C 在那一槽记着的落点，\
             R 那一槽是这几槽里最后轮到的；还记着说明有一次发布没经分配器记根"
        );
        Ok(RootRingPushed {
            versions_published,
            ring_slots_written: target.ring_slots,
        })
    }
}

/// 发布返回之后发 discard（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G5，判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
/// 第二节 S6）：「可再分配」谓词为真（已回收、没被抬 F 扣住、没被隔离、F 已生效）的槽按 64 槽对齐的整段发——只发回收过槽、整段可再分配的段
/// （`allocator::PoolAllocator::take_the_segments_to_discard_on_every_device`），每块盘发它自己那几段。discard 取几态、设备日志怎么判不在这里
/// （归设备侧与崩溃点重放）。长度恒为一段（64 × 16384 字节），不发长度 0 的。
///
/// 装着还没清扫的孤儿头的段不发（主 agent 2026-10-15 定，零轮）：discard 的结局是三态之一（D13（验证路线） 已定项 4），不能拿它当擦除——
/// 抹孤儿头归实例表清扫（`instance_table_sweep`），清扫判过、抹过之前段里的孤儿头要原样留着。孤儿头 = 段里某个槽开头认得出本池的单元头
/// （清扫认头的那一判，`instance_table_sweep::unit_head_read_by_sweep`），它的写序实例在现行那一版的实例表里还有一行、按那一行判未发布
/// （`UnitHeadReadBySweep::is_unpublished_by_the_row`）。判之前读不出实例表、读不出段里的一个槽，这一段同样不发（读不到不等于没有）。
pub fn discard_the_segments_reusable_as_a_whole<Device: BlockDevice>(
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    parameters: &MakeFilesystemParameters,
    current_root: &RootRecord,
) -> DiscardsAfterThePublishes {
    let mut discards = DiscardsAfterThePublishes::default();
    let segments_to_discard = allocator.take_the_segments_to_discard_on_every_device();
    if segments_to_discard
        .iter()
        .all(|(_, segment_starts)| segment_starts.is_empty())
    {
        return discards;
    }
    let rows_of_the_instance_table: Option<BTreeMap<InstanceGeneration, InstanceRow>> =
        instance_table_chain_of_root(&*devices, current_root)
            .ok()
            .map(|chain| {
                chain
                    .records
                    .rows
                    .iter()
                    .map(|row| (row.instance, *row))
                    .collect()
            });
    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    for (device_identity, segment_starts) in segments_to_discard {
        for segment_start in segment_starts {
            let holding = match &rows_of_the_instance_table {
                Some(rows) => segment_holding_unswept_orphan_heads(
                    &*devices,
                    device_identity,
                    segment_start,
                    rows,
                    filesystem_identifier,
                ),
                None => SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed,
            };
            match holding {
                SegmentHoldingUnsweptOrphanHeads::No => {}
                SegmentHoldingUnsweptOrphanHeads::Yes => {
                    discards
                        .withheld_holding_unswept_orphan_heads
                        .push((device_identity, segment_start));
                    continue;
                }
                SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed => {
                    discards
                        .withheld_after_a_failed_read
                        .push((device_identity, segment_start));
                    continue;
                }
            }
            let Some((_, device)) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == device_identity)
            else {
                continue;
            };
            match device.discard_at(
                segment_start.to_device_offset(),
                CLUSTER_SEGMENT_SLOTS * SLOT_BYTES,
            ) {
                Ok(DiscardOutcome::Applied) => {
                    discards.applied.push((device_identity, segment_start));
                }
                Ok(DiscardOutcome::Unsupported) => {
                    discards.unsupported.push((device_identity, segment_start));
                }
                Err(_cause) => discards.failed.push((device_identity, segment_start)),
            }
        }
    }
    discards
}

/// 一段装没装着还没清扫的孤儿头（[`discard_the_segments_reusable_as_a_whole`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SegmentHoldingUnsweptOrphanHeads {
    No,
    Yes,
    /// 段里一个槽读不出：判不了，按装着算（不发）。
    NotJudgedReadFailed,
}

/// `device` 上从 `segment_start` 起的那一段：逐槽读一整槽，开头认得出本池的单元头、写序实例在 `rows` 里有一行、按那一行判未发布的，就是还没清扫的孤儿头。
/// 认出一个头就按它的整单元跨度跳（同清扫的认领规则）。迭代上界是一段 64 槽；提前出口是认出一个孤儿头与读失败。
fn segment_holding_unswept_orphan_heads<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    segment_start: SlotNumber,
    rows: &BTreeMap<InstanceGeneration, InstanceRow>,
    filesystem_identifier: u64,
) -> SegmentHoldingUnsweptOrphanHeads {
    let slot_bytes = usize::try_from(SLOT_BYTES).expect("一槽 16384 字节");
    let mut slot = segment_start.0;
    while slot < segment_start.0 + CLUSTER_SEGMENT_SLOTS {
        let Some(bytes) = reader.read(device, SlotNumber(slot).to_device_offset(), slot_bytes)
        else {
            return SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed;
        };
        let Some(head) = unit_head_read_by_sweep(&bytes) else {
            slot += 1;
            continue;
        };
        slot += head.span_in_slots();
        if head.filesystem_identifier == filesystem_identifier
            && rows
                .get(&head.writer_instance)
                .is_some_and(|row| head.is_unpublished_by_the_row(row))
        {
            return SegmentHoldingUnsweptOrphanHeads::Yes;
        }
    }
    SegmentHoldingUnsweptOrphanHeads::No
}

/// 一次整理运行推到哪为止。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompactionRunReach {
    /// 显式入口与定时兜底起跑的那一趟：推到停机原因或发布预算用完。
    UntilAStopOrTheBudget,
    /// 后台入口取走用户改动之后记下的「该跑了」（判决 G8 丁）：至多一批。一批 = 搬了单元的那一次发布（判决 `research/prompts/m4-segment-code-r1-main-verification.md`
    /// 第三节 F2）；不搬单元的发布不算批：上一条意图的收尾（一个没搬就删意图、R 已按记录全空只删意图、丢意图）与签发。所以一趟可以发
    /// 「收尾 + 签发 + 一批」三次；签发之后计划里只剩这次发布本来就重写的节点时，那一次也是一个没搬就删意图，全局差值为真就再签再搬。
    /// 一趟发几次的上界是这一趟的发布预算 `B_gc`（[`publish_budget_of_a_compaction_run`]），每签一条之前照判签发前判与「上一条无产出就不签」
    /// （判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G4、G8；判决 `research/prompts/m4-segment-code-r3-main-verification.md`
    /// 第三节 J3 核过：与 G8 丁「至多一批」、G8 甲「至多签 `B_gc` 条」一致）。
    AtMostOneBatchAfterAUserChange,
    /// 可写挂载续做（D8（核心索引结构） 已定项 5 第 2 条；判决 G1）：只推盘上那一条未完成的意图，推到它删了、停机原因或预算用完，不签新的。
    ContinueTheLiveIntentAtTheMount,
}

/// 整理的一次发布做的是哪一种（[`CompactionStep::Published`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompactionPublishKind {
    /// 签发那次发布：意图写进 logged_ops。
    IntentIssued,
    /// 搬一批，意图照抄。
    Batch,
    /// 最后一批：搬完 R 按记录全空，同一次发布删意图（D8（核心索引结构） 已定项 5）。
    BatchCompletingTheIntent,
    /// R 已按记录全空（别的发布腾空的），单独一次发布删意图。
    IntentCompletedWithoutABatch,
    /// 丢意图：单独一次发布删它（攻方 K-c 的隔离槽、区域扫描拒了 R）。
    IntentDropped,
    /// 丢意图：R 里一个能搬的都没有、却还有未释放的记录；丢完这一趟停（[`CompactionRunStop::NothingMovableButTheRegionIsNotEmpty`]）。
    IntentDroppedWithNothingMovable,
}

/// 推一步整理的结局：发成一次整理发布，或照判据停。
enum CompactionStep {
    Published {
        version: Box<TransactionOutput>,
        kind: CompactionPublishKind,
    },
    Stopped(CompactionRunStop),
}

/// 整理推一次发布（后台任务的一批）交回的两样：这一步的结局，与这一批的后台计算记账（请求的后端、实际走的路）。
struct CompactionPublishAdvanced {
    step: Result<CompactionStep, RelocationBatchRefused>,
    background_compute_of_the_batch: BackgroundComputeOfOneBatch,
}

/// 整理一趟（显式入口、可写挂载续做共用；后台入口另经 [`run_compaction_on_with_the_region_check_of_the_intent_deletion`]）：先走空删除续做（G8 次序），
/// 再照预算一次一次推（[`advance_compaction_by_one_publish`]）。预算 `B_gc` 数的是整理的每一次发布（签发、搬一批、删意图：每一次都用掉一个根槽）。
/// 每一批之前不看调用方。discard 由调用方在这一趟之后发。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、这次挂载的后台计算状态"
)]
pub(crate) fn run_compaction_on<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    reach: CompactionRunReach,
    background_compute: &mut BackgroundComputeOfThisMount,
) -> CompactionRun {
    run_compaction_on_with_the_region_check_of_the_intent_deletion(
        parameters,
        devices,
        allocator,
        compaction,
        current,
        instance,
        reach,
        CompactionRunBudget::SinceTheLastUserChange,
        PacingOfTheRun::NotPaced,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        background_compute,
        &mut InTheMiddleOfEachBackgroundBatch::NothingHappens,
    )
}

/// 同 [`run_compaction_on`]，另带三样：这一趟的发布预算按 `budget_rule` 算；每一批之前照 `pacing` 问调用方（后台入口：前台有 fsync 在等、
/// 写带宽到了上限就不发下一批，D26（后台整理与放置回收） 已定项 13 第 6 条）；一批腾空了 R、照同一批删意图再发时删意图那一支的核照 `region_check` 判
/// （[`MountedSession::run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region`]），
/// 每一批读完后台计算设置之后照 `in_the_middle_of_each_batch` 做（产品路径什么都不做，
/// [`MountedSession::run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch`]）。整理因一批出错停着时一个写都不发
/// （[`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、预算按哪一条算、每批之前看不看调用方、删意图那一支的核怎么判、这次挂载的后台计算状态、一批读完设置之后做什么"
)]
fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    reach: CompactionRunReach,
    budget_rule: CompactionRunBudget,
    pacing: PacingOfTheRun<'_>,
    region_check: RegionCheckOfTheIntentDeletion,
    background_compute: &mut BackgroundComputeOfThisMount,
    in_the_middle_of_each_batch: &mut InTheMiddleOfEachBackgroundBatch<'_>,
) -> CompactionRun {
    compaction.observations.runs_started += 1;
    let mut versions_published: Vec<TransactionOutput> = Vec::new();
    let mut background_compute_of_each_batch: Vec<BackgroundComputeOfOneBatch> = Vec::new();
    let finish = |compaction_of_the_run: &mut CompactionOfThisMount,
                  versions_published_by_the_run: Vec<TransactionOutput>,
                  background_compute_of_each_batch_of_the_run: Vec<BackgroundComputeOfOneBatch>,
                  batches_published: u64,
                  outcome: CompactionRunOutcome| {
        if let CompactionRunOutcome::Stopped(stop) = &outcome {
            compaction_of_the_run.observations.last_stop = Some(*stop);
        }
        CompactionRun {
            versions_published: versions_published_by_the_run,
            batches_published,
            outcome,
            discards: DiscardsAfterThePublishes::default(),
            background_compute_of_each_batch: background_compute_of_each_batch_of_the_run,
        }
    };
    let switches = allocator.compaction().switches;
    match switches.run_forcing {
        CompactionRunForcing::ForcedToStopForTests => {
            return finish(
                compaction,
                versions_published,
                background_compute_of_each_batch,
                0,
                CompactionRunOutcome::Stopped(CompactionRunStop::ForcedToStopForTests),
            );
        }
        CompactionRunForcing::FollowTheWatermarks | CompactionRunForcing::ForcedToStartForTests => {
        }
    }
    // D26（后台整理与放置回收） 已定项 13 第 4 条：整理因一批出错停着，等一次通过校验的切换请求解除；停着时一个写都不发（删除续做也不走）。
    if background_compute.is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation) {
        return finish(
            compaction,
            versions_published,
            background_compute_of_each_batch,
            0,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError,
            ),
        );
    }
    let batch_units = match switches.batch_size {
        CompactionBatchSize::NotDecidedByTheClauses => {
            return finish(
                compaction,
                versions_published,
                background_compute_of_each_batch,
                0,
                CompactionRunOutcome::Stopped(CompactionRunStop::BatchSizeNotDecidedByTheClauses),
            );
        }
        CompactionBatchSize::ForcedForTests(batch_units) => batch_units,
    };
    // 次序（判决 G8）：删除续做在整理批之前。
    let queue_is_empty = match &*current {
        PoolVersion::WithFile(version_of_trees)
        | PoolVersion::WithTreesAndNoFile(version_of_trees) => {
            version_of_trees.queue_of_committed_intents_is_empty()
        }
        PoolVersion::WithoutFile(_) => true,
    };
    if !queue_is_empty {
        let mut walked = current
            .version_of_trees()
            .expect("队列不空的一版是有树的一版")
            .clone();
        let walk = publish_the_queue_until_it_is_empty(
            parameters,
            devices,
            allocator,
            &mut walked,
            instance,
        );
        *current = PoolVersion::of_a_version_of_trees(walked);
        versions_published.extend(walk.steps_published);
        if let Err(cause) = walk.stopped_by {
            return finish(
                compaction,
                versions_published,
                background_compute_of_each_batch,
                0,
                CompactionRunOutcome::Failed(CompactionRunFailure::DeletionContinuation(cause)),
            );
        }
    }
    let budget = match budget_rule {
        CompactionRunBudget::SinceTheLastUserChange => publish_budget_of_a_compaction_run(
            parameters.geometry.root_ring_slots_per_region.count(),
            compaction.publishes_since_the_last_user_change,
        ),
        CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange => {
            publishes_left_of_the_budget_given_by_the_last_user_change(
                parameters.geometry.root_ring_slots_per_region.count(),
                compaction.publishes_since_the_last_user_change,
            )
        }
    };
    let mut publishes_of_the_run: u64 = 0;
    let mut batches_published: u64 = 0;
    // 循环：每一轮发成一次整理发布或停。迭代上界是预算（每发成一次计一次，到 budget 停）；跨轮携带的是现行那一版、已发的次数与批数。
    // 上一条完成的意图住会话（`CompactionOfThisMount::last_completed_intent`）：跨趟留着，签下一条之前那一判（攻方 K-h）读它。
    loop {
        if publishes_of_the_run >= budget {
            return finish(
                compaction,
                versions_published,
                background_compute_of_each_batch,
                batches_published,
                CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget }),
            );
        }
        if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {
            return finish(
                compaction,
                versions_published,
                background_compute_of_each_batch,
                batches_published,
                CompactionRunOutcome::Stopped(CompactionRunStop::OneBatchAfterTheUserChangeDone),
            );
        }
        // D26（后台整理与放置回收） 已定项 13 第 6 条：后台入口每一批之前问调用方，前台有 fsync 在等、写带宽到了上限就不发下一批。
        let write_bandwidth_limit = match pacing {
            PacingOfTheRun::NotPaced => CompactionWriteBandwidthLimit::Unlimited,
            PacingOfTheRun::PacedByTheCaller {
                write_bandwidth_limit,
                caller,
            } => {
                match caller.foreground_fsync() {
                    ForegroundFsync::Waiting => {
                        return finish(
                            compaction,
                            versions_published,
                            background_compute_of_each_batch,
                            batches_published,
                            CompactionRunOutcome::Stopped(
                                CompactionRunStop::YieldedToAForegroundFsync,
                            ),
                        );
                    }
                    ForegroundFsync::NoneWaiting => {}
                }
                if let Err(stop) = compaction.admit_the_next_batch_under_the_write_bandwidth_limit(
                    write_bandwidth_limit,
                    caller.clock_reading(),
                ) {
                    return finish(
                        compaction,
                        versions_published,
                        background_compute_of_each_batch,
                        batches_published,
                        CompactionRunOutcome::Stopped(stop),
                    );
                }
                write_bandwidth_limit
            }
        };
        let CompactionPublishAdvanced {
            step,
            background_compute_of_the_batch,
        } = advance_compaction_by_one_publish(
            parameters,
            devices,
            allocator,
            compaction,
            current,
            instance,
            batch_units,
            reach,
            region_check,
            background_compute,
            in_the_middle_of_each_batch,
        );
        background_compute_of_each_batch.push(background_compute_of_the_batch);
        match step {
            Ok(CompactionStep::Published { version, kind }) => {
                publishes_of_the_run += 1;
                compaction.publishes_since_the_last_user_change += 1;
                compaction.observations.root_ring_slots_used += 1;
                compaction.note_the_bytes_written_by_a_compaction_publish(
                    write_bandwidth_limit,
                    bytes_written_by_a_compaction_publish(&version, devices.len()),
                );
                match kind {
                    // 一批 = 搬了单元的那一次（判决 m4-segment-code-r1 第三节 F2：没搬单元的发布不记成一批——删意图的那一批计划里的单元
                    // 都是这次发布本来就重写的节点时，它就是一次只删意图的发布）。
                    CompactionPublishKind::Batch
                    | CompactionPublishKind::BatchCompletingTheIntent
                        if !version.units_relocated_by_this_publish.is_empty() =>
                    {
                        batches_published += 1;
                        compaction.observations.batches_published += 1;
                        let relocated =
                            u64::try_from(version.units_relocated_by_this_publish.len())
                                .expect("一批的单元数");
                        compaction.observations.units_relocated += relocated;
                        compaction.observations.units_relocated_by_the_last_batch = relocated;
                    }
                    CompactionPublishKind::Batch
                    | CompactionPublishKind::BatchCompletingTheIntent
                    | CompactionPublishKind::IntentIssued
                    | CompactionPublishKind::IntentCompletedWithoutABatch
                    | CompactionPublishKind::IntentDropped
                    | CompactionPublishKind::IntentDroppedWithNothingMovable => {}
                }
                versions_published.push(*version);
                if kind == CompactionPublishKind::IntentDroppedWithNothingMovable {
                    return finish(
                        compaction,
                        versions_published,
                        background_compute_of_each_batch,
                        batches_published,
                        CompactionRunOutcome::Stopped(
                            CompactionRunStop::NothingMovableButTheRegionIsNotEmpty,
                        ),
                    );
                }
            }
            Ok(CompactionStep::Stopped(stop)) => {
                return finish(
                    compaction,
                    versions_published,
                    background_compute_of_each_batch,
                    batches_published,
                    CompactionRunOutcome::Stopped(stop),
                );
            }
            Err(refused) => {
                return finish(
                    compaction,
                    versions_published,
                    background_compute_of_each_batch,
                    batches_published,
                    CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),
                );
            }
        }
    }
}

/// 一版里整理要读的几样：每个进映射单元的映射条目（按 key 在这一版的映射树里查，连同留存的），钉住的槽（豁免映射的三类：映射树各节点、
/// 树表、实例表链——根记录直接指着它们），码 1 数据单元落着的槽（签发前判「只装提交内生块的段不算非全空」用）。
struct VersionAsTheCompactionReadsIt {
    mapping_entries: Vec<CentralMappingEntry>,
    pinned_slots: BTreeSet<SlotNumber>,
    slots_holding_user_data: BTreeSet<SlotNumber>,
}

/// 整理读一版时取映射条目的那一步：整理选段与搬单元读版本（`version_as_the_compaction_reads_it`）经它取，别处不调。
/// 交回每个进映射单元按 key 在这一版的映射树里查到的映射条目，按 `mapped_units` 的次序（查不到、条目窄于字段表的不收），
/// 后面接这一版留存的映射条目，按留存的次序。
/// 公开给测试：七·6 的用例拿它逐项比逐 key 查（`central_mapping_locations_in_the_version`）的参照，判整理读漏、读错；
/// 它做的查找记在 [`LookupSite::CompactionReadsTheCentralMapping`] 那一处（`transaction::lookup_work`），用例读那一处的
/// 解叶次数与建索引次数判退回逐 key 查、每把 key 现建读者、每片叶重解（七·6 代码轮第二轮判决第三节）。
#[must_use]
pub fn central_mapping_as_the_compaction_reads_it(
    version: &TransactionOutput,
) -> Vec<CentralMappingEntry> {
    let _inside_the_compaction_read =
        lookup_work::entering(LookupSite::CompactionReadsTheCentralMapping);
    // 每把 key 查一次，叶按片解一次（七·6）：答案与逐 key 查 `central_mapping_locations_in_the_version` 逐个相同。
    let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    let mut mapping_entries: Vec<CentralMappingEntry> = version
        .mapped_units
        .iter()
        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {
            MappingLookup::Found(locations) => Some(CentralMappingEntry {
                key: key.clone(),
                locations,
            }),
            MappingLookup::NodeMalformedOrNoEntryWithThisKey
            | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
        })
        .collect();
    mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(
        |(key, locations)| CentralMappingEntry {
            key: key.clone(),
            locations: *locations,
        },
    ));
    mapping_entries
}

fn version_as_the_compaction_reads_it(
    version: &TransactionOutput,
) -> VersionAsTheCompactionReadsIt {
    let mapping_entries = central_mapping_as_the_compaction_reads_it(version);
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
    let mut slots_holding_user_data = BTreeSet::new();
    for entry in &mapping_entries {
        if entry.key.first() == Some(&UNIT_CLASS_DATA) {
            let slot = entry.locations[0].slot;
            slots_holding_user_data.insert(slot);
            slots_holding_user_data.insert(SlotNumber(slot.0 + 1));
        }
    }
    VersionAsTheCompactionReadsIt {
        mapping_entries,
        pinned_slots,
        slots_holding_user_data,
    }
}

/// 在一个新开的写入口上发整理的一次发布（`transaction::publish_compaction_step`），接在 `previous` 后面；删意图那一支照记录核。
fn publish_the_compaction_step_once<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion(
        parameters,
        devices,
        allocator,
        previous,
        planned,
        intent_step,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        instance,
    )
}

/// 同 [`publish_the_compaction_step_once`]，删意图那一支的核照 `region_check_of_the_intent_deletion` 判
/// （[`publish_compaction_step_with_the_region_check_of_the_intent_deletion`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、上一版、计划、对意图做什么、删意图那一支的核怎么判、实例代号"
)]
fn publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    region_check_of_the_intent_deletion: RegionCheckOfTheIntentDeletion,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    let switches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    publish_compaction_step_with_the_region_check_of_the_intent_deletion(
        &mut writer,
        allocator,
        previous,
        planned,
        intent_step,
        region_check_of_the_intent_deletion,
        instance,
    )
}

/// 整理的一次发布写了多少单元字节（写带宽上限数的就是它，D26（后台整理与放置回收） 已定项 13 第 6 条）：这次重写的角色加搬到目的地的单元，
/// 按槽数乘 16384、每块盘各一份（第一版每个单元落池里每一块盘）；journal 记录与根槽那几次写不计。搬走的单元的跨度取这一版里它目的落点那条
/// 分配记录的（目的地那条记录就是这次发布记下的，找不到不计）。
fn bytes_written_by_a_compaction_publish(version: &TransactionOutput, devices: usize) -> u64 {
    let slots_of_the_rewritten_roles: u64 =
        version.rewritten.iter().map(|role| role.span_slots()).sum();
    let slots_of_the_relocated_units: u64 = version
        .units_relocated_by_this_publish
        .iter()
        .filter_map(|relocated| {
            let destination = relocated.destination[0];
            version
                .allocation_records
                .iter()
                .find(|record| {
                    record.device == destination.device && record.slot == destination.slot
                })
                .map(|record| u64::from(record.span_slots))
        })
        .sum();
    (slots_of_the_rewritten_roles + slots_of_the_relocated_units)
        .saturating_mul(SLOT_BYTES)
        .saturating_mul(u64::try_from(devices).expect("盘数装得进 u64"))
}

/// 整理的一次发布被拒是不是空间不够的那几种（[`refusal_is_short_of_space`]）：是就暂停、不推抬 F（判决 G8 g1）。
fn compaction_publish_refused_short_of_space(refused: &RelocationBatchRefused) -> bool {
    match refused {
        RelocationBatchRefused::Publish(cause) => refusal_is_short_of_space(cause),
        RelocationBatchRefused::PlannedUnitNotInTheVersion { .. }
        | RelocationBatchRefused::PinnedUnitInThePlan { .. }
        | RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition { .. }
        | RelocationBatchRefused::UnitBytesNotAvailable { .. }
        | RelocationBatchRefused::CompactionIntentAlreadyLive { .. }
        | RelocationBatchRefused::NoLiveCompactionIntent
        | RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
            ..
        }
        | RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried { .. } => {
            false
        }
    }
}

/// 整理的一批（推一次发布，后台任务种类 [`BackgroundTask::CompactionRelocation`]）：进门读一次这次挂载的后台计算设置
/// （D26（后台整理与放置回收） 已定项 13 第 1 条：挂着请求的切换在这里换上，从这一批起生效；一趟里每一批各读一次，不在一趟开头读一次），
/// 按它定这一批走哪条路，一批之内不再读（`.claude/rules/fs-design.md` 五条硬要求第 5 条，`background_compute::BackgroundComputeOfThisMount::end_the_batch`
/// 断言没换）。选 `gpu` 时这一批的甲、乙、丙三段经设置带着的那一份 GPU 实现算（[`CompactionComputeOfTheBatch::OnTheGpu`]），
/// 三段各自确认、不确定、交 CPU 判的条目数记进这一批的记账；发布那一半照旧在 CPU 上，照发布路径逐条核计划与读盘核（第 8 条）。
/// 读完设置之后、开算之前照 `in_the_middle_of_each_batch` 做（只供测试的钩子在这一刻请求切换、或注入一次出错）。
/// 这一批算着出了错（第 4 条：GPU 实现交回出错，或只供测试的钩子注入）：这一批放弃，一个写都不发、意图留在盘上（撤 R 排他，同暂停），
/// 不换回 CPU；整理这个后台任务停下，出错之前就挂着待生效的切换时当场解除（`background_compute::BackgroundComputeOfThisMount::stop_the_task_after_a_compute_error`）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、批大小、推到哪为止、一批腾空了 R 之后删意图那一支的核怎么判、这次挂载的后台计算状态、一批读完设置之后做什么"
)]
fn advance_compaction_by_one_publish<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    batch_units: CompactionBatchUnits,
    reach: CompactionRunReach,
    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region: RegionCheckOfTheIntentDeletion,
    background_compute: &mut BackgroundComputeOfThisMount,
    in_the_middle_of_each_batch: &mut InTheMiddleOfEachBackgroundBatch<'_>,
) -> CompactionPublishAdvanced {
    let batch_under_way = background_compute.begin_a_batch(BackgroundTask::CompactionRelocation);
    in_the_middle_of_each_batch.in_the_middle_of_the_batch(background_compute);
    let gpu_compute = batch_under_way.gpu_compute_of_the_route().cloned();
    let mut gpu_work = GpuWorkOfTheBatch::default();
    let advanced = match background_compute.take_the_compute_error_of_the_batch_under_way() {
        Some(error) => Err(CompactionBatchNotCompleted::ComputeFailed(error)),
        None => match &gpu_compute {
            None => advance_compaction_by_one_publish_computing_on(
                parameters,
                devices,
                allocator,
                compaction,
                current,
                instance,
                batch_units,
                reach,
                region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
                &mut CompactionComputeOfTheBatch::OnTheCpu,
            ),
            Some(gpu_compute) => {
                let mut implementation = gpu_compute.lock();
                advance_compaction_by_one_publish_computing_on(
                    parameters,
                    devices,
                    allocator,
                    compaction,
                    current,
                    instance,
                    batch_units,
                    reach,
                    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
                    &mut CompactionComputeOfTheBatch::OnTheGpu {
                        gpu_compute: &mut *implementation,
                        gpu_work: &mut gpu_work,
                    },
                )
            }
        },
    };
    let mut compute_error_that_abandoned_the_batch = None;
    let step = match advanced {
        Ok(step) => Ok(step),
        Err(CompactionBatchNotCompleted::Refused(refused)) => Err(refused),
        Err(CompactionBatchNotCompleted::ComputeFailed(error)) => {
            allocator.compaction_mut().pause_the_live_intent();
            let task_after_the_error = background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                error.clone(),
            );
            compute_error_that_abandoned_the_batch = Some(error);
            Ok(CompactionStep::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error,
                },
            ))
        }
    };
    CompactionPublishAdvanced {
        step,
        background_compute_of_the_batch: background_compute.end_the_batch(
            batch_under_way,
            gpu_compute.map(|_| gpu_work),
            compute_error_that_abandoned_the_batch,
        ),
    }
}

/// 一批整理这一次在哪算（[`advance_compaction_by_one_publish`] 进门定下，一批之内不变）。封闭集合。
enum CompactionComputeOfTheBatch<'gpu> {
    /// 区域扫描、选段打分都照 CPU 写法（`scan_region_over_the_identity_index`、`choose_region`）。
    OnTheCpu,
    /// 甲、乙两段经 [`scan_region_on_the_gpu`]，丙段经 [`live_slots_per_cluster_segment_computing_on`]，都交这一份实现；
    /// 各段的条目数记进 `gpu_work`。
    OnTheGpu {
        gpu_compute: &'gpu mut (dyn BackgroundGpuCompute + Send),
        gpu_work: &'gpu mut GpuWorkOfTheBatch,
    },
}

/// 一批整理没推成一步的两种：发布路径拒了（照旧交给一趟去判停还是失败），或这一批算着出了错（第 4 条：放弃这一批、停下整理）。
enum CompactionBatchNotCompleted {
    Refused(RelocationBatchRefused),
    ComputeFailed(BackgroundComputeError),
}

impl From<RelocationBatchRefused> for CompactionBatchNotCompleted {
    fn from(refused: RelocationBatchRefused) -> Self {
        CompactionBatchNotCompleted::Refused(refused)
    }
}

/// 扫一个 R，照这一批在哪算：CPU 那一条交回 `scan_on_the_cpu` 的结果（没有不确定的记录）；GPU 那一条经 [`scan_region_on_the_gpu`]，
/// 读盘、钉住、隔离与 CPU 那一条读同一版、同一份索引与同两张集合。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：R、CPU 那一条的扫法、这一版的记录、按身份索引、钉住与隔离的槽、读的盘、这一批在哪算"
)]
fn scan_the_region_computing_on<Device: BlockDevice>(
    region: CompactionRegion,
    scan_on_the_cpu: impl Fn(
        CompactionRegion,
        &[(DeviceIdentity, Device)],
    ) -> Result<RegionScan, RegionScanRefusal>,
    allocation_records: &[AllocationRecord],
    mapping_index: &CentralMappingIdentityIndex<'_>,
    pinned_slots: &BTreeSet<SlotNumber>,
    quarantined_slots: &BTreeSet<SlotNumber>,
    devices_to_read: &[(DeviceIdentity, Device)],
    compute: &mut CompactionComputeOfTheBatch<'_>,
) -> Result<RegionScanWithTheRecordsUncertainThisRound, BackgroundComputeError> {
    match compute {
        CompactionComputeOfTheBatch::OnTheCpu => Ok(RegionScanWithTheRecordsUncertainThisRound {
            scan: scan_on_the_cpu(region, devices_to_read),
            records_uncertain_this_round: Vec::new(),
        }),
        CompactionComputeOfTheBatch::OnTheGpu {
            gpu_compute,
            gpu_work,
        } => scan_region_on_the_gpu(
            region,
            allocation_records,
            mapping_index,
            pinned_slots,
            quarantined_slots,
            |device, slot, bytes| devices_to_read.read(device, slot.to_device_offset(), bytes),
            &mut **gpu_compute,
            gpu_work,
        ),
    }
}

/// 丙段（选段打分）在 GPU 上数每个整段的活槽数：CPU 那一条交 `None`（由 `choose_region` 自己数）；GPU 那一条一个整段一个，
/// 这一轮不确定的段记成 [`LiveSlotsOfAClusterSegment::UncertainThisRound`]。
///
/// # Errors
/// GPU 实现交回出错 ⇒ [`BackgroundComputeError::GpuSegmentFailed`]。
fn live_slots_per_cluster_segment_computing_on(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    compute: &mut CompactionComputeOfTheBatch<'_>,
) -> Result<Option<Vec<LiveSlotsOfAClusterSegment>>, BackgroundComputeError> {
    match compute {
        CompactionComputeOfTheBatch::OnTheCpu => Ok(None),
        CompactionComputeOfTheBatch::OnTheGpu {
            gpu_compute,
            gpu_work,
        } => {
            let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
            let decisions = decisions_of_one_segment(
                GpuComputedSegment::LiveSlotsPerClusterSegment,
                segments.len(),
                gpu_compute.count_live_slots_per_cluster_segment(
                    allocation_records,
                    unit_area,
                    &segments,
                ),
                &mut gpu_work.live_slots_per_cluster_segment,
            )?;
            Ok(Some(
                decisions
                    .into_iter()
                    .map(|decision| match decision {
                        GpuEntryDecision::ConfirmedByBothKernels(live_slots)
                        | GpuEntryDecision::JudgedByTheCpuReference(live_slots) => {
                            LiveSlotsOfAClusterSegment::Counted(live_slots)
                        }
                        GpuEntryDecision::UncertainThisRound => {
                            LiveSlotsOfAClusterSegment::UncertainThisRound
                        }
                    })
                    .collect(),
            ))
        }
    }
}

/// 推整理一次发布（判决 G1–G4、G8 的整串；意图住 logged_ops，判决 G1）。活意图读现行那一版的 logged_ops（盘上的权威），
/// 分配器这一侧的 R 排他与它一致（会话签发、删意图时装上与撤掉，挂载按 logged_ops 装上）。先把发布路径隔离过的槽并进常驻隔离集。
/// 1. 没有活意图：可写挂载续做停在「没有活意图可续」（只续做、不签新的）；上一条完成的意图全局差值为假就停（攻方 K-h）；选段（强制选的那一段，
///    或 greedy 先扫后选），选不出就停；签发前判 R 外非全空段的空洞 ≥ R 里要搬的单元数（攻方 K-g），不够就停；签发：先在分配器上排他 R
///    （签发那次发布自己的新落点也不进 R），发一次只把意图写进 logged_ops 的发布（`c0` = 现行那一版的 txg，`segs0` = 签发那次发布之前、
///    现行那一版 R 那块盘按记录口径的全空段数，判决 m4-segment-code-r2 第三节 H3，用户 2026-10-16 定「取签发之前」）；
///    被空间拒就撤排他、停（意图没签成）。
/// 2. 有活意图：暂停过的重新排他；R 里出现隔离槽（直接查常驻隔离集，钉住与隔离同时成立也算，丁 1 交判第 4 点）、区域扫描拒了 R、扫出隔离类，
///    都发一次只删意图的发布丢掉它（攻方 K-c）；对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）、计划一批（跳过这次发布本来就重写的节点、补上 R 里
///    真要搬的单元，判决 m4-segment-code-r2 第三节 H4）：要搬就发——这一批罩住 R 里每一条未释放的记录时同一次发布删意图（D8（核心索引结构）
///    已定项 5；R 里只剩这次发布本来就重写的节点时也是这一支；发布在算好的账上核 R 按记录全空，核不过照这一批不删意图再发）；别的批照抄意图、
///    在算好的账上核 R 还有未释放的记录，这一批腾空了 R（计划外的、这次发布本来就重写的节点也离开 R）就照同一批删意图再发（判决 m4-segment-code-r1
///    第三节 F2），R 还不空而计划里有本来就重写、不另搬的单元就跳过它们再计划，所以照抄意图的一批至少真搬一个；一个都没搬的不记成一批；
///    R 已按记录全空就发一次只删意图的发布；
///    一个能搬的都没有而 R 不空就发一次只删意图的发布丢掉它、这一趟停。整理的发布被空间拒就暂停（撤 R 排他、意图留在盘上，攻方 K-k）、停。
///
/// 一次调用至多算 R 里能搬的单元数加三次账、发一次（每跳过一次本来就重写的单元再算一次；照抄那一支核出腾空了 R 换删意图再算，删意图那一支核不过
/// 换照抄再算），选段那一圈的上界是单元区的整段数。
///
/// 区域扫描与选段打分照 `compute` 在哪算（[`CompactionComputeOfTheBatch`]）：走 GPU 时三段交实现，判定都确认（或交 CPU 判）时每一步与 CPU 那一条
/// 逐项相同；这一轮不确定的条目这一批不回收、不搬（D26（后台整理与放置回收） 已定项 13 第 8 条）——不确定的段不进选段候选，不确定的记录不当可搬；
/// 选中要签的那一段里有不确定的记录就这一批不签、停在 [`CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound`]；
/// 有活意图时 R 里除了不确定的记录再没有可搬的就这一批不发、停在
/// [`CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove`]（意图照留，不删也不丢），下一批重扫照判。
/// 实现交回出错 ⇒ [`CompactionBatchNotCompleted::ComputeFailed`]，在任何一次发布之前。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、批大小、推到哪为止、一批腾空了 R 之后删意图那一支的核怎么判、这一批在哪算"
)]
fn advance_compaction_by_one_publish_computing_on<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    batch_units: CompactionBatchUnits,
    reach: CompactionRunReach,
    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region: RegionCheckOfTheIntentDeletion,
    compute: &mut CompactionComputeOfTheBatch<'_>,
) -> Result<CompactionStep, CompactionBatchNotCompleted> {
    compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());
    let version = current
        .version_of_trees()
        .expect("整理只在有树的一版上推：入口判过，整理的发布与删除续做都交回有树的一版")
        .clone();
    let live_on_disk = live_compaction_intent_of_the_entries(&version.logged_operations);
    assert_eq!(
        allocator
            .compaction()
            .live_intent()
            .map(|live| live.intent),
        live_on_disk.map(|(_, intent)| intent),
        "分配器这一侧的活意图就是现行那一版 logged_ops 里那一条：会话签发、删意图时装上与撤掉，挂载按 logged_ops 装上"
    );
    let unit_area = {
        let device_map = allocator
            .devices
            .first()
            .expect("池里至少一块盘：两盘同槽，选段与判类按第一块盘的记录做（判决 G1 攻方 K-n）");
        UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        }
    };
    let read_of_the_version = version_as_the_compaction_reads_it(&version);
    let quarantined_slots = compaction.quarantined_slots().clone();
    // 映射条目的按身份索引每读一版建一次（用户 2026-10-26 定「每读一版建一次」）：这一版里扫几个 R（选段时逐段先扫、续做意图时扫 R）都共用它。
    let mapping_index =
        CentralMappingIdentityIndex::of_the_mapping_entries(&read_of_the_version.mapping_entries);
    let scan_of = |region: CompactionRegion, devices_to_read: &[(DeviceIdentity, Device)]| {
        scan_region_over_the_identity_index(
            region,
            &version.allocation_records,
            &mapping_index,
            &read_of_the_version.pinned_slots,
            &quarantined_slots,
            |device, slot, bytes| devices_to_read.read(device, slot.to_device_offset(), bytes),
        )
    };
    let scan_of_the_batch =
        |region: CompactionRegion,
         devices_to_read: &[(DeviceIdentity, Device)],
         compute_of_the_batch: &mut CompactionComputeOfTheBatch<'_>| {
            scan_the_region_computing_on(
                region,
                scan_of,
                &version.allocation_records,
                &mapping_index,
                &read_of_the_version.pinned_slots,
                &quarantined_slots,
                devices_to_read,
                compute_of_the_batch,
            )
        };
    let Some((_issued_txg, intent)) = live_on_disk else {
        if reach == CompactionRunReach::ContinueTheLiveIntentAtTheMount {
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::NoLiveIntentLeftToContinue,
            ));
        }
        if let Some(completed) = compaction.last_completed_intent {
            let empty_now =
                empty_cluster_segments_by_the_records(&version.allocation_records, unit_area);
            if !the_completed_intent_yielded(empty_now, completed.empty_cluster_segments_at_issue) {
                return Ok(CompactionStep::Stopped(
                    CompactionRunStop::NoYieldFromTheLastIntent {
                        empty_cluster_segments_now: empty_now,
                        empty_cluster_segments_at_issue: completed.empty_cluster_segments_at_issue,
                    },
                ));
            }
        }
        // 钉住的槽与常驻隔离集里的槽都让一段没资格：带隔离槽的段签了也立刻要丢（攻方 K-c）。
        let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = read_of_the_version
            .pinned_slots
            .union(&quarantined_slots)
            .copied()
            .collect();
        // 选中的那一段的扫描与它这一轮不确定的记录（CPU 那一条恒空）。
        let chosen_scan: Option<(RegionScan, Vec<Placement>)> =
            match allocator.compaction().switches.forced_region {
                Some(forced_region) => {
                    let is_disqualified = allocator
                        .cluster_segments()
                        .contains(&forced_region.start_slot())
                        || slots_that_disqualify_a_segment
                            .range(forced_region.start_slot()..forced_region.end_slot())
                            .next()
                            .is_some()
                        || compaction
                            .segments_refused_by_the_region_scan
                            .contains(forced_region.start_slot());
                    let scanned = scan_of_the_batch(forced_region, &*devices, compute)
                        .map_err(CompactionBatchNotCompleted::ComputeFailed)?;
                    match (is_disqualified, scanned.scan) {
                        (false, Ok(scan)) if scan.movable_units().next().is_some() => {
                            Some((scan, scanned.records_uncertain_this_round))
                        }
                        (false, Ok(_)) | (true, _) => None,
                        (false, Err(refusal)) => {
                            compaction
                                .segments_refused_by_the_region_scan
                                .note_refusal(forced_region, refusal);
                            None
                        }
                    }
                }
                None => {
                    let mut region_scans: Vec<RegionScan> =
                        region_scans_of_the_segments_without_an_unreleased_record(
                            &version.allocation_records,
                            unit_area,
                        );
                    // 扫过的段这一轮不确定的记录（按段起点；CPU 那一条恒空）。
                    let mut records_uncertain_in_each_scanned_segment: BTreeMap<
                        SlotNumber,
                        Vec<Placement>,
                    > = BTreeMap::new();
                    // 丙段：GPU 那一条这一批数一次每段的活槽数（这一圈里记录不变）；CPU 那一条是 `None`，照 `choose_region` 自己数。
                    let live_slots_counted_on_the_gpu =
                        live_slots_per_cluster_segment_computing_on(
                            &version.allocation_records,
                            unit_area,
                            compute,
                        )
                        .map_err(CompactionBatchNotCompleted::ComputeFailed)?;
                    // 迭代上界：每答一次「先扫」下一次就多一段有结论（扫成了进 region_scans，被拒了进不合格段表），不超过单元区的整段数。
                    loop {
                        let choice = match &live_slots_counted_on_the_gpu {
                            None => choose_region(
                                &version.allocation_records,
                                unit_area,
                                allocator.cluster_segments(),
                                None,
                                &slots_that_disqualify_a_segment,
                                &region_scans,
                                &compaction.segments_refused_by_the_region_scan,
                                RegionChoiceCriterion::Greedy,
                            ),
                            Some(live_slots_per_segment) => choose_region_over_the_live_slots(
                                live_slots_per_segment,
                                unit_area,
                                allocator.cluster_segments(),
                                None,
                                &slots_that_disqualify_a_segment,
                                &region_scans,
                                &compaction.segments_refused_by_the_region_scan,
                                RegionChoiceCriterion::Greedy,
                            ),
                        };
                        match choice {
                            RegionChoice::Chosen(region) => {
                                break region_scans
                                    .into_iter()
                                    .find(|scan| scan.region == region)
                                    .map(|scan| {
                                        let records_uncertain =
                                            records_uncertain_in_each_scanned_segment
                                                .remove(&region.start_slot())
                                                .unwrap_or_default();
                                        (scan, records_uncertain)
                                    });
                            }
                            RegionChoice::ScanBeforeChoosing(region) => {
                                let scanned = scan_of_the_batch(region, &*devices, compute)
                                    .map_err(CompactionBatchNotCompleted::ComputeFailed)?;
                                match scanned.scan {
                                    Ok(scan) => {
                                        records_uncertain_in_each_scanned_segment.insert(
                                            region.start_slot(),
                                            scanned.records_uncertain_this_round,
                                        );
                                        region_scans.push(scan);
                                    }
                                    Err(refusal) => compaction
                                        .segments_refused_by_the_region_scan
                                        .note_refusal(region, refusal),
                                }
                            }
                            RegionChoice::NoEligibleSegment => break None,
                        }
                    }
                }
            };
        let Some((scan, records_uncertain_in_the_chosen_region)) = chosen_scan else {
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::NoEligibleSegment,
            ));
        };
        // GPU 那一条选中的这一段里有这一轮不确定的记录：R 里要搬几个还数不准，签发前那一判（空洞够不够装下 R 里要搬的）判不了，
        // 这一批不签，下一批重扫照判（第 8 条「这一批不回收、不搬」；CPU 那一条恒空）。
        if !records_uncertain_in_the_chosen_region.is_empty() {
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound {
                    records_uncertain_this_round: u64::try_from(
                        records_uncertain_in_the_chosen_region.len(),
                    )
                    .expect("一段 64 槽"),
                },
            ));
        }
        let units_to_move = u64::try_from(scan.movable_units().count()).expect("一段 64 槽");
        let device_map = allocator
            .devices
            .first()
            .expect("池里至少一块盘（上面取单元区时判过）");
        let holes = holes_outside_the_region_in_segments_holding_user_data(
            &version.allocation_records,
            &read_of_the_version.slots_holding_user_data,
            unit_area,
            scan.region,
            |slot| device_map.is_free(slot),
        );
        if !holes_admit_the_issue(holes, units_to_move) {
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove {
                    holes,
                    units_to_move,
                },
            ));
        }
        let issued = CompactionIntent {
            region: scan.region,
            allocation_generation_at_issue: version.root.checkpoint_txg,
            empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
                &version.allocation_records,
                unit_area,
            ),
        };
        // R 从签发那次发布起就排他：那次发布自己的新落点也不进 R。没发成就撤掉（盘上没有这条意图）。
        allocator.compaction_mut().issue(issued);
        return match publish_the_compaction_step_once(
            parameters,
            devices,
            allocator,
            &version,
            &[],
            CompactionIntentStep::Issue(issued),
            instance,
        ) {
            Ok(published) => {
                compaction.observations.intents_issued += 1;
                *current = PoolVersion::of_a_version_of_trees(published.clone());
                Ok(CompactionStep::Published {
                    version: Box::new(published),
                    kind: CompactionPublishKind::IntentIssued,
                })
            }
            Err(refused) => {
                allocator.compaction_mut().withdraw_the_live_intent();
                if compaction_publish_refused_short_of_space(&refused) {
                    Ok(CompactionStep::Stopped(
                        CompactionRunStop::IssueRefusedShortOfSpace,
                    ))
                } else {
                    Err(refused.into())
                }
            }
        };
    };
    let is_the_region_excluded = allocator
        .compaction()
        .live_intent()
        .is_some_and(|live| live.is_the_region_excluded_from_placement);
    if !is_the_region_excluded {
        allocator.compaction_mut().resume_the_live_intent();
    }
    let region = intent.region;
    // 对 R 重扫的结果与这一轮不确定的记录（CPU 那一条恒空）。
    let scan: Option<(RegionScan, Vec<Placement>)> =
        if region_holds_a_quarantined_slot(region, &quarantined_slots) {
            None
        } else {
            let scanned = scan_of_the_batch(region, &*devices, compute)
                .map_err(CompactionBatchNotCompleted::ComputeFailed)?;
            match scanned.scan {
                Ok(scan)
                    if !scan
                        .allocations
                        .iter()
                        .any(|allocation| allocation.class == AllocationInRegion::Quarantined) =>
                {
                    Some((scan, scanned.records_uncertain_this_round))
                }
                Ok(_scan_with_a_quarantined_slot) => None,
                Err(refusal) => {
                    compaction
                        .segments_refused_by_the_region_scan
                        .note_refusal(region, refusal);
                    None
                }
            }
        };
    // 这一次发布要搬哪几个、对意图做什么、发成了是哪一种：计划跳过这次发布本来就重写的节点、补上 R 里真要搬的单元（判决
    // `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，用户 2026-10-16 定「规划时补足」）。照抄意图的一批在算好的账上
    // 核计划里每个单元都真搬，核出本来就重写的那几把 key 就并进跳过的集合、再计划一次（[`compaction_publish_planned_for_the_region_scan`]）。
    // 迭代上界是 R 里能搬的单元数加一：被拒时点名的 key 都在这一次的计划里，计划不取已跳过的，所以每被拒一次跳过的集合至少多一把 key；
    // 能搬的都跳过之后计划要么罩住 R 里每一条记录（删意图那一支，不再核这一样），要么没有可搬的。跨轮携带的只有跳过的集合。
    let mut units_this_publish_rewrites_anyway: BTreeSet<Vec<u8>> = BTreeSet::new();
    let (published, kind) = loop {
        let Some((planned, intent_step, kind)) =
            compaction_publish_planned_with_the_records_uncertain_this_round(
                scan.as_ref()
                    .map(|(scan, records_uncertain)| (scan, records_uncertain.as_slice())),
                batch_units,
                &units_this_publish_rewrites_anyway,
                &version.allocation_records,
                region,
            )
        else {
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove {
                    records_uncertain_this_round: scan.as_ref().map_or(
                        0,
                        |(_, records_uncertain)| {
                            u64::try_from(records_uncertain.len()).expect("一段 64 槽")
                        },
                    ),
                },
            ));
        };
        match publish_the_planned_compaction_step_falling_back_on_the_region_check(
            parameters,
            devices,
            allocator,
            &version,
            &planned,
            intent_step,
            kind,
            region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
            instance,
        ) {
            Ok(published_and_kind) => break published_and_kind,
            Err(CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish {
                mapping_keys,
            }) => {
                let skipped_before = units_this_publish_rewrites_anyway.len();
                units_this_publish_rewrites_anyway.extend(mapping_keys);
                assert!(
                    units_this_publish_rewrites_anyway.len() > skipped_before,
                    "被拒时点名的 key 都在这一次的计划里，计划不取已跳过的：跳过的集合每被拒一次至少多一把 key"
                );
            }
            Err(CompactionStepRefused::Refused(refused)) => {
                return pause_or_hand_up(allocator, refused)
                    .map_err(CompactionBatchNotCompleted::from)
            }
        }
    };
    match kind {
        CompactionPublishKind::BatchCompletingTheIntent
        | CompactionPublishKind::IntentCompletedWithoutABatch => {
            allocator.compaction_mut().withdraw_the_live_intent();
            compaction.observations.intents_completed += 1;
            compaction.last_completed_intent = Some(intent);
        }
        CompactionPublishKind::IntentDropped
        | CompactionPublishKind::IntentDroppedWithNothingMovable => {
            allocator.compaction_mut().withdraw_the_live_intent();
            compaction.observations.intents_dropped += 1;
        }
        CompactionPublishKind::Batch | CompactionPublishKind::IntentIssued => {}
    }
    *current = PoolVersion::of_a_version_of_trees(published.clone());
    Ok(CompactionStep::Published {
        version: Box::new(published),
        kind,
    })
}

/// 同 [`compaction_publish_planned_for_the_region_scan`]，扫描带着这一轮不确定的记录（GPU 那一条，D26（后台整理与放置回收） 已定项 13 第 8 条）：
/// 没有不确定的记录就是它；有的时候 R 里还留着不确定的未释放记录，这一批只搬判定了可搬的那几个、照抄意图（R 被这一批腾空时照 F2 同一批删意图，
/// 发布路径在算好的账上核），不删意图也不因「一个能搬的都没有」丢意图；判定了可搬的一个都没有时交 `None`：这一批不发。
fn compaction_publish_planned_with_the_records_uncertain_this_round(
    scan: Option<(&RegionScan, &[Placement])>,
    batch_units: CompactionBatchUnits,
    rewritten_anyway: &BTreeSet<Vec<u8>>,
    allocation_records_of_the_version: &[AllocationRecord],
    region: CompactionRegion,
) -> Option<(
    Vec<PlannedRelocation>,
    CompactionIntentStep,
    CompactionPublishKind,
)> {
    match scan {
        Some((scan, records_uncertain)) if !records_uncertain.is_empty() => {
            match plan_batch_skipping_the_units_this_publish_rewrites_anyway(
                scan,
                batch_units,
                rewritten_anyway,
            ) {
                PlannedBatch::Relocate(planned) => {
                    // 照抄意图那一支：R 被这一批腾空时发布路径照 F2 同一批删意图（R 里还留着不确定的记录时腾不空）。
                    let carry_the_intent_unless_the_region_is_emptied =
                        CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion;
                    Some((
                        planned,
                        carry_the_intent_unless_the_region_is_emptied,
                        CompactionPublishKind::Batch,
                    ))
                }
                PlannedBatch::RegionEmptied
                | PlannedBatch::NothingMovableButTheRegionIsNotEmpty => None,
            }
        }
        Some((scan, _no_record_uncertain)) => Some(compaction_publish_planned_for_the_region_scan(
            Some(scan),
            batch_units,
            rewritten_anyway,
            allocation_records_of_the_version,
            region,
        )),
        None => Some(compaction_publish_planned_for_the_region_scan(
            None,
            batch_units,
            rewritten_anyway,
            allocation_records_of_the_version,
            region,
        )),
    }
}

/// 有活意图时这一次整理发布的计划：要搬哪几个、对意图做什么、发成了是哪一种（[`advance_compaction_by_one_publish`]）。`scan` 是对 R 重扫的结果
/// （R 里有隔离槽、区域扫描拒了 R、扫出隔离类时是 `None`），`rewritten_anyway` 是这一次发布上已试算出的、本来就重写而不另搬的映射 key
/// （[`plan_batch_skipping_the_units_this_publish_rewrites_anyway`]）。
fn compaction_publish_planned_for_the_region_scan(
    scan: Option<&RegionScan>,
    batch_units: CompactionBatchUnits,
    rewritten_anyway: &BTreeSet<Vec<u8>>,
    allocation_records_of_the_version: &[AllocationRecord],
    region: CompactionRegion,
) -> (
    Vec<PlannedRelocation>,
    CompactionIntentStep,
    CompactionPublishKind,
) {
    match scan.map(|scan| {
        (
            scan,
            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
                scan,
                batch_units,
                rewritten_anyway,
            ),
        )
    }) {
        // R 里有隔离槽、区域扫描拒了 R、扫出隔离类：丢意图（攻方 K-c）。
        None => (
            Vec::new(),
            CompactionIntentStep::Drop,
            CompactionPublishKind::IntentDropped,
        ),
        Some((scan, PlannedBatch::Relocate(planned))) => {
            // 这一批罩住 R 里每一条未释放的记录（搬走，或 R 里只剩这次发布本来就重写的节点）：最后一批，同一次发布删意图
            // （D8（核心索引结构） 已定项 5；判决 m4-segment-code-r1 第三节 F2）。
            if planned.len() == scan.allocations.len() {
                (
                    planned,
                    CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                    CompactionPublishKind::BatchCompletingTheIntent,
                )
            } else {
                // 照抄意图那一支在算好的账上核两样：R 被这一批腾空了（计划外的、这次发布本来就重写的节点也离开 R）就照同一批删意图再发
                // （判决 m4-segment-code-r1 第三节 F2）；计划里有本来就重写、不另搬的就跳过它们再计划（判决 m4-segment-code-r2 第三节 H4）。
                (
                    planned,
                    CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion,
                    CompactionPublishKind::Batch,
                )
            }
        }
        Some((_scan, PlannedBatch::RegionEmptied)) => {
            assert!(
                region_is_empty_by_the_records(allocation_records_of_the_version, region),
                "扫描里一条未释放记录都没有，R 就按记录全空：扫描与停机谓词读的是同一份记录"
            );
            (
                Vec::new(),
                CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                CompactionPublishKind::IntentCompletedWithoutABatch,
            )
        }
        Some((_scan, PlannedBatch::NothingMovableButTheRegionIsNotEmpty)) => (
            Vec::new(),
            CompactionIntentStep::Drop,
            CompactionPublishKind::IntentDroppedWithNothingMovable,
        ),
    }
}

/// 发这一次计划好的整理发布，删意图那一支或照抄那一支在算好的账上核不过时换个对意图的做法再算（[`advance_compaction_by_one_publish`]）。
/// 照抄意图那一支经 [`publish_compaction_step_in_the_session`] 发，计划里有这次发布本来就重写的单元就不发、交回
/// [`CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish`]（判决 m4-segment-code-r2 第三节 H4）。发成交回这一版与发成的是哪一种；
/// 别的被拒原样交回。一次调用至多算三次账、发一次。
///
/// 两条「核不过就照抄意图」的退路在产品路径上走得到（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H2）：
/// 计划里的单元在这次发布里都离开 R（搬走，或本来就重写），删意图那一核不过只有一种来历——释放之前读盘核把一份隔离、那条记录留在已分配，
/// 也就是一份副本读与重读都读不出（瞬时读错）；下一遍算账再读时它又读得出，照抄意图那一版里 R 就按记录全空了，下一次发布照「R 已按记录全空」
/// 只删意图。另外两处走得到：只供测试的开关关掉 R 排他、这次发布的新落点落进 R；只供测试的 `region_check` 强制删意图那一核不过。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、计划、对意图做什么、发成了是哪一种、一批腾空了 R 之后删意图那一支的核怎么判、实例代号"
)]
fn publish_the_planned_compaction_step_falling_back_on_the_region_check<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    version: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    kind: CompactionPublishKind,
    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region: RegionCheckOfTheIntentDeletion,
    instance: InstanceGeneration,
) -> Result<(TransactionOutput, CompactionPublishKind), CompactionStepRefused> {
    let switches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    let first_try = publish_compaction_step_in_the_session(
        &mut writer,
        allocator,
        version,
        planned,
        intent_step,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        PlannedUnitsRewrittenAnyway::RefusedSoThatTheSessionPlansAgain,
        instance,
    );
    match first_try {
        Ok(published) => Ok((published, kind)),
        // 最后一批在算好的账上 R 还不按记录全空（一份副本读与重读都读不出而被隔离，那条记录留在已分配；或只供测试的开关关掉排他、
        // 这一批的新落点落进了 R）：这一批不删意图，照搬、照抄意图。
        Err(CompactionStepRefused::Refused(
            RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
                ..
            },
        )) if kind == CompactionPublishKind::BatchCompletingTheIntent => {
            match publish_the_compaction_step_once(
                parameters,
                devices,
                allocator,
                version,
                planned,
                CompactionIntentStep::CarryTheIntent,
                instance,
            ) {
                Ok(published) => Ok((published, CompactionPublishKind::Batch)),
                Err(refused) => Err(refused.into()),
            }
        }
        // 这一批在算好的账上腾空了 R（计划外的、这次发布本来就重写的节点也离开 R）：同一批删意图再发（判决 m4-segment-code-r1 第三节 F2）。
        // 删意图那一支再核 R 按记录全空：两次只差 logged_ops 那一条，取落点不读它，核不过的来历同上一支（瞬时读错、关掉排他），
        // 照这一批不删意图再发（用例经只供测试的 `RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`
        // 或故障注入的瞬时读错强制进入）。
        Err(CompactionStepRefused::Refused(
            RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried { .. },
        )) if kind == CompactionPublishKind::Batch => {
            match publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion(
                parameters,
                devices,
                allocator,
                version,
                planned,
                CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
                instance,
            ) {
                Ok(published) => Ok((published, CompactionPublishKind::BatchCompletingTheIntent)),
                Err(
                    RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
                        ..
                    },
                ) => match publish_the_compaction_step_once(
                    parameters,
                    devices,
                    allocator,
                    version,
                    planned,
                    CompactionIntentStep::CarryTheIntent,
                    instance,
                ) {
                    Ok(published) => Ok((published, CompactionPublishKind::Batch)),
                    Err(refused) => Err(refused.into()),
                },
                Err(refused) => Err(refused.into()),
            }
        }
        Err(refused) => Err(refused),
    }
}

/// 整理的一次发布没发成（活意图在盘上）：空间不够就暂停——撤 R 排他、意图留着（攻方 K-k），不推抬 F（判决 G8 g1）；别的原样交回。
fn pause_or_hand_up(
    allocator: &mut PoolAllocator,
    refused: RelocationBatchRefused,
) -> Result<CompactionStep, RelocationBatchRefused> {
    if compaction_publish_refused_short_of_space(&refused) {
        allocator.compaction_mut().pause_the_live_intent();
        Ok(CompactionStep::Stopped(
            CompactionRunStop::PausedShortOfSpace,
        ))
    } else {
        Err(refused)
    }
}

/// 现行那一版没有可改的文件：拒成 [`UserChangeRefused::NoFileVersionToChange`]，带上这次改动里已经落盘的（走空的那几步与推的抬 F）。
fn no_file_version_to_change(published_in_this_change: PublishedInThisChange) -> UserChangeRefused {
    let PublishedInThisChange {
        floor_raises,
        refusals_that_pushed_the_floor_raises,
        versions_published,
    } = published_in_this_change;
    UserChangeRefused::NoFileVersionToChange {
        floor_raises,
        refusals_that_pushed_the_floor_raises,
        versions_published,
    }
}

/// 这次用户改动里已经落盘的：推了的几串抬 F（按先后）与推它们的那几次被拒（一一对应），推的抬 F 之外发成的每一版（按先后）。
struct PublishedInThisChange {
    floor_raises: Vec<RaisedFloor>,
    refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    versions_published: VersionsPublishedInThisChange,
}

/// 一次发布被拒之后会话怎么接（D16（发布语义） 已定项 1「准入」那一行）：不是空间不够的那两种（[`refusal_is_short_of_space`]）原样交回 `Publish`；
/// 是就为被拒的那一次发布推一串抬 F 的空发布（这次准入已推 `publishes_pushed_in_this_admission` 次，预算照
/// `mount::push_one_floor_raise_within_the_admission_budget`），推成交回记上这一串之后的 `published_in_this_change` 与这一串推了几次空发布，
/// 不再推交回 `NoSpaceAfterRaisingTheFloor`（报不报「被宽护卡住」，报的话记下这次改动剩下的 `remainder`，要出路时照它预演，
/// [`wide_protection_at_the_ceiling`]），抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`。
/// 推的那一串就地推进 `current`（被拒的那一次要接在它后面的那一版）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、影子账那一臂、这次改动还剩什么、预算已用的数、已落盘的、这一次被拒，不成一个概念"
)]
fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
    remainder: RemainderOfTheUserChange<'_>,
    publishes_pushed_in_this_admission: usize,
    published_in_this_change: PublishedInThisChange,
    refusal: PublishError,
) -> Result<(PublishedInThisChange, usize), UserChangeRefused> {
    let PublishedInThisChange {
        mut floor_raises,
        mut refusals_that_pushed_the_floor_raises,
        versions_published,
    } = published_in_this_change;
    if !refusal_is_short_of_space(&refusal) {
        return Err(UserChangeRefused::Publish {
            cause: refusal,
            floor_raises,
            versions_published,
        });
    }
    match push_one_floor_raise_within_the_admission_budget(
        parameters,
        devices,
        allocator,
        current,
        shadow_ledger,
        publishes_pushed_in_this_admission,
    ) {
        Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
            let publishes_of_this_raise = raised.publishes.len();
            floor_raises.push(raised);
            refusals_that_pushed_the_floor_raises.push(refusal);
            Ok((
                PublishedInThisChange {
                    floor_raises,
                    refusals_that_pushed_the_floor_raises,
                    versions_published,
                },
                publishes_of_this_raise,
            ))
        }
        Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => Err(
            UserChangeRefused::NoSpaceAfterRaisingTheFloor(Box::new(NoSpaceAfterRaisingTheFloor {
                last_refusal: refusal,
                floor_raises,
                refusals_that_pushed_the_floor_raises,
                // 先借 `stop` 判这一形，再把它挪进下一个字段（字段按写的次序求值）。
                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(
                    allocator, current, remainder, &stop,
                ),
                stop,
                versions_published,
            })),
        ),
        Err(cause) => Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(
            Box::new(FloorRaiseFailedWhilePushingForSpace {
                cause,
                refusal_that_pushed_the_failed_raise: refusal,
                floor_raises,
                refusals_that_pushed_the_floor_raises,
                versions_published,
            }),
        )),
    }
}

/// 把 `current` 的 `deleted_inodes` 走空（[`publish_the_queue_until_it_is_empty`]），走空里的删除类发布被拒是空间不够时
/// 推一串抬 F、回收之后接着走（用户 2026-10-02 定案 Q-V「推抬 F 再试」：删除类发布不判式子那一判，落得下那一判照判，
/// 被它以空间拒时推抬 F 腾得出槽）；推满仍放不下、抬 F 自己报错、被拒不是空间不够，交回会话要报的那一种拒绝。
/// 每一次被拒的删除类发布各算一次准入（与 unlink 意图那次、别的用户改动那次一样，预算数的是为被判的那一次发布推了几次空发布）：
/// 这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，这次准入已推的数从 0 数起。`current` 停在最后一次落盘的那一版上
/// （删除那一步或推的空发布）；发成的每一步记进 `versions_published`。`remainder` 是走空之后这次改动还剩什么（推满仍不够、被宽护卡住时
/// 记进报里，要出路时预演推环之后照它再发一次，[`wide_protection_at_the_ceiling`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、实例代号、影子账那一臂、这次改动还剩什么、已落盘的，不成一个概念"
)]
fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    remainder: RemainderOfTheUserChange<'_>,
    published_in_this_change: PublishedInThisChange,
) -> Result<PublishedInThisChange, UserChangeRefused> {
    let mut published_in_this_change = published_in_this_change;
    let mut publishes_pushed_in_this_admission = 0;
    // 迭代上界：每一轮要么走空交回，要么交回拒绝，要么推成一串（推的数加进这次准入，同一次发布的准入在预算 B − 1 次空发布之内停），
    // 要么这一趟有一步落了盘（队首那个文件少了单元或删掉了，队列与单元数都有限）。跨轮携带的是 `current` 与已落盘的。
    loop {
        let DeletionQueueWalk {
            steps_published,
            stopped_by,
        } = publish_the_queue_until_it_is_empty(
            parameters,
            devices.as_mut_slice(),
            allocator,
            current,
            instance,
        );
        if !steps_published.is_empty() {
            publishes_pushed_in_this_admission = 0;
        }
        published_in_this_change
            .versions_published
            .extend(steps_published);
        let refusal = match stopped_by {
            Ok(()) => return Ok(published_in_this_change),
            Err(refusal) => refusal,
        };
        let publishes_of_this_raise;
        (published_in_this_change, publishes_of_this_raise) =
            push_one_floor_raise_for_a_refusal_for_space(
                parameters,
                devices,
                allocator,
                current,
                shadow_ledger,
                remainder,
                publishes_pushed_in_this_admission,
                published_in_this_change,
                refusal,
            )?;
        publishes_pushed_in_this_admission += publishes_of_this_raise;
    }
}

/// 攒批的会话（D16（发布语义） 已定项 2、5、10，判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节；用户 2026-10-26 定射程
/// 「串行下时间与脏量两种触发」）：一次可写挂载之后的会话（[`MountedSession`]）连同收下、还没发布的改动（待发）。数据写类改动经 [`Self::accept`]
/// 收进待发，不当场发布；触发 ⟺ 待发非空 ∧（now − 上次发布的读数 ≥ `T_time` ∨ 待发改动要写的 journal 字节 ≥ 有效 `T_dirty`），
/// 只在 [`Self::accept`] 收下一个完整改动之后与调用方按周期调的 [`Self::publish_when_due`] 里判，一个请求不被触发切开；触发就把待发整个并成一次发布
/// （[`UserChange::DataChangesPublishedTogether`]）。核心层不起线程、不读钟：`now` 是调用方交进来的单调读数，读数倒退当作没过时间、不回拨。
///
/// 屏障（unlink、截断、建删快照、整理、回退、卸载）进门先排空待发、自己单发：经 [`Self::accept`] 交进来的 unlink、截断、建删快照先排空、再照
/// [`MountedSession::publish_user_change`] 单发；整理只在后台入口里跑（[`Self::run_compaction_in_the_background`]：判过起跑之后、跑之前先排空），
/// [`Self::accept`] 与 [`Self::publish_when_due`] 发成之后只记「该跑了」，不在调用线程上跑整理（用户 2026-10-26 定「挪出调用线程」）；
/// 回退与卸载要的是会话本身，只经 [`Self::into_session_after_publishing_the_pending_changes`] 交出——它先排空待发，所以类型上卸载、回退之前待发一定空了，
/// 不会静默丢掉。原入口 [`MountedSession::publish_user_change`] 语义不变：这里的 [`Self::publish_user_change`] 先排空待发再交给它。
///
/// 待发与会话都是私有字段：只有这几个入口动得了它们，读只经 [`Self::session`] 与几个观测。
#[derive(Debug)]
pub struct MountedSessionWithPendingChanges {
    session: MountedSession,
    pending: PendingDataChanges,
    thresholds: PublishTriggerThresholds,
    /// 上一次发布那次调用交进来的读数（还没发布过时是第一次见到的读数，D16（发布语义） 已定项 5「上次 checkpoint 的时刻」）；一次读数都还没见过是 `None`。
    /// 只往前走：读数倒退时不回拨。
    reading_of_the_last_publish: Option<CallerMonotonicClockReading>,
}

/// 收下、还没发布的数据写类改动：折叠过的那一份整份写（同一 inode 的几次写只留最后一次）、按收下的次序的几次建 inode，
/// 与收下的这几次改动各自单独发布要写的 journal 字节之和（D16（发布语义） 已定项 5：按待发改动要写的 journal 字节算，批内覆盖写 K 次按 K 次的记录照实算、不折叠）。
#[derive(Debug)]
struct PendingDataChanges {
    whole_write: Option<PendingWholeWrite>,
    new_inodes: Vec<NewInodesOfOneChange>,
    journal_bytes_of_the_changes_accepted: JournalBytesToWrite,
    changes_accepted: u64,
}

/// 待发里那一份整份写（内容自有）。
#[derive(Clone, Debug)]
struct PendingWholeWrite {
    inode: InodeNumber,
    content: Vec<u8>,
    write_time_seconds: u64,
}

impl PendingDataChanges {
    fn empty() -> Self {
        Self {
            whole_write: None,
            new_inodes: Vec::new(),
            journal_bytes_of_the_changes_accepted: JournalBytesToWrite::ZERO,
            changes_accepted: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.changes_accepted == 0
    }

    /// 待发并成一次发布的样子。
    fn as_published_together(&self) -> DataChangesPublishedTogether<'_> {
        DataChangesPublishedTogether {
            whole_write: self
                .whole_write
                .as_ref()
                .map(|whole_write| WholeWriteOfOneInode {
                    inode: whole_write.inode,
                    content: &whole_write.content,
                    write_time_seconds: whole_write.write_time_seconds,
                }),
            new_inodes: &self.new_inodes,
        }
    }

    /// 这一批里建的 inode 一共几个（之后建的号从记账水位加上它起发）。
    fn new_inodes_in_total(&self) -> u64 {
        self.as_published_together().new_inodes_in_total()
    }

    /// 收下一次数据写类改动：整份写折叠（后到的盖掉先到的），建 inode 追加在后面；这次改动单独发布要写的 journal 字节加进合计。
    fn take_in(
        &mut self,
        change: DataChangeAccepted<'_>,
        journal_bytes_of_this_change: JournalBytesToWrite,
    ) {
        match change {
            DataChangeAccepted::WholeWrite(whole_write) => {
                self.whole_write = Some(PendingWholeWrite {
                    inode: whole_write.inode,
                    content: whole_write.content.to_vec(),
                    write_time_seconds: whole_write.write_time_seconds,
                });
            }
            DataChangeAccepted::NewInodes(new_inodes) => self.new_inodes.push(new_inodes),
        }
        self.journal_bytes_of_the_changes_accepted = self
            .journal_bytes_of_the_changes_accepted
            .plus(journal_bytes_of_this_change);
        self.changes_accepted += 1;
    }
}

/// 收得进待发的一次数据写类改动（D16（发布语义） 已定项 10「数据写类可并」）。封闭集合。
#[derive(Clone, Copy, Debug)]
enum DataChangeAccepted<'content> {
    WholeWrite(WholeWriteOfOneInode<'content>),
    NewInodes(NewInodesOfOneChange),
}

impl<'content> DataChangeAccepted<'content> {
    /// 这一次改动单独并成一批的样子（单独发布要写多少 journal 字节按它判）。`new_inodes` 是装建 inode 那一项的一格槽（借给返回值）。
    fn alone(
        self,
        new_inodes: &'content mut [NewInodesOfOneChange; 1],
    ) -> DataChangesPublishedTogether<'content> {
        match self {
            DataChangeAccepted::WholeWrite(whole_write) => DataChangesPublishedTogether {
                whole_write: Some(whole_write),
                new_inodes: &[],
            },
            DataChangeAccepted::NewInodes(created) => {
                new_inodes[0] = created;
                DataChangesPublishedTogether {
                    whole_write: None,
                    new_inodes: &new_inodes[..],
                }
            }
        }
    }
}

/// 交给 [`MountedSessionWithPendingChanges::accept`] 的一次改动按什么收。封闭集合。
#[derive(Clone, Copy, Debug)]
enum ChangeAtTheAccept<'content> {
    /// 数据写类：收进待发。覆盖写那两种（一个数据单元的契约）带 `fits_one_data_unit_only`。
    Data {
        change: DataChangeAccepted<'content>,
        contract: WholeWriteContractAtTheAccept,
    },
    /// 屏障：先排空待发、再单发（unlink、截断、建删快照）。
    Barrier,
    /// 按偏移写：进不进待发、算哪一类条款没写（D16（发布语义） 已定项 10 只列了整份写、建 inode 与屏障），第一版不收。
    WriteAtOffset,
    /// 已经并好的一批：它是排空待发时交给会话的，不是用户的一次改动，收进待发条款没写，第一版不收。
    DataChangesPublishedTogether,
}

/// 整份写的入口契约（同会话两条入口：覆盖写一个数据单元以内，顺序写不按单元数拒）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WholeWriteContractAtTheAccept {
    OneDataUnit,
    AnyNumberOfDataUnits,
    /// 不是整份写（建 inode）。
    NotAWholeWrite,
}

impl<'content> ChangeAtTheAccept<'content> {
    fn of(change: UserChange<'content>) -> Self {
        let whole_write = |inode: InodeNumber,
                           content: &'content [u8],
                           write_time_seconds: u64,
                           contract: WholeWriteContractAtTheAccept| {
            ChangeAtTheAccept::Data {
                change: DataChangeAccepted::WholeWrite(WholeWriteOfOneInode {
                    inode,
                    content,
                    write_time_seconds,
                }),
                contract,
            }
        };
        let first_file = InodeNumber(crate::transaction::FIRST_INODE_NUMBER);
        match change {
            UserChange::Overwrite(file) => whole_write(
                first_file,
                file.content,
                file.write_time_seconds,
                WholeWriteContractAtTheAccept::OneDataUnit,
            ),
            UserChange::SequentialWrite(file) => whole_write(
                first_file,
                file.content,
                file.write_time_seconds,
                WholeWriteContractAtTheAccept::AnyNumberOfDataUnits,
            ),
            UserChange::OverwriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => whole_write(
                inode,
                content,
                write_time_seconds,
                WholeWriteContractAtTheAccept::OneDataUnit,
            ),
            UserChange::SequentialWriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => whole_write(
                inode,
                content,
                write_time_seconds,
                WholeWriteContractAtTheAccept::AnyNumberOfDataUnits,
            ),
            UserChange::NewInodes {
                count,
                write_time_seconds,
            } => ChangeAtTheAccept::Data {
                change: DataChangeAccepted::NewInodes(NewInodesOfOneChange {
                    count,
                    write_time_seconds,
                }),
                contract: WholeWriteContractAtTheAccept::NotAWholeWrite,
            },
            UserChange::Unlink { .. }
            | UserChange::Truncate { .. }
            | UserChange::CreateSnapshot
            | UserChange::DestroySnapshot { .. } => ChangeAtTheAccept::Barrier,
            UserChange::WriteAtOffset { .. } => ChangeAtTheAccept::WriteAtOffset,
            UserChange::DataChangesPublishedTogether(_) => {
                ChangeAtTheAccept::DataChangesPublishedTogether
            }
        }
    }
}

/// 一次触发是哪一支先到（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支可观测）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishTriggerCause {
    /// now − 上次发布的读数 ≥ `T_time`，脏量还没到。
    TimeThreshold,
    /// 待发改动要写的 journal 字节 ≥ 有效 `T_dirty`，时间还没到。
    DirtyThreshold,
    /// 两支同时到。
    BothThresholds,
}

/// D16（发布语义） 已定项 2 的触发式：待发非空 ∧（离上次发布过了 `T_time` ∨ 待发改动要写的 journal 字节到了有效 `T_dirty`）。待发空时两支都不看
/// （空闲时不推空发布：照不带这一合取的式子，空闲时每 `T_time` 一次空发布，把回退候选集挤掉——判决第二节「时间那一支」那一格）。
/// `time_since_the_last_publish` 是调用方按读数算好的间隔（读数倒退时是 0：当作没过时间）。纯函数。
#[must_use]
pub fn publish_trigger_cause(
    pending_is_empty: bool,
    time_since_the_last_publish: Duration,
    journal_bytes_of_the_pending_changes: JournalBytesToWrite,
    thresholds: PublishTriggerThresholds,
) -> Option<PublishTriggerCause> {
    if pending_is_empty {
        return None;
    }
    let is_time_reached = time_since_the_last_publish >= thresholds.time_threshold();
    let is_dirty_reached = journal_bytes_of_the_pending_changes >= thresholds.dirty_threshold();
    match (is_time_reached, is_dirty_reached) {
        (true, true) => Some(PublishTriggerCause::BothThresholds),
        (true, false) => Some(PublishTriggerCause::TimeThreshold),
        (false, true) => Some(PublishTriggerCause::DirtyThreshold),
        (false, false) => None,
    }
}

/// 攒批的会话这一次为什么发布（报给调用方，RefFS 翻译按它把每一次发布翻成一个显式发布点，判决第二节「RefFS」那一格）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReasonOfThePublish {
    /// [`MountedSessionWithPendingChanges::accept`] 收下这次改动之后触发，待发整个并成一次发布（这次改动在里面）。
    TriggeredAfterTheAccept(PublishTriggerCause),
    /// [`MountedSessionWithPendingChanges::publish_when_due`] 里触发。
    TriggeredWhenDue(PublishTriggerCause),
    /// 屏障进门之前排空待发（屏障自己不在里面）。
    PendingChangesPublishedBeforeABarrier,
    /// 屏障自己那一次（[`MountedSession::publish_user_change`] 单发：unlink、截断连同它们的全部批）。
    BarrierPublishedAlone,
    /// [`MountedSessionWithPendingChanges::publish_pending_now`]：立即排空。
    PendingChangesPublishedOnRequest,
    /// [`MountedSessionWithPendingChanges::publish_user_change`] 之前排空待发。
    PendingChangesPublishedBeforeTheUserChange,
    /// [`MountedSessionWithPendingChanges::publish_user_change`] 那一次改动自己。
    UserChangePublishedAlone,
    /// 后台整理判过起跑、跑之前排空待发。
    PendingChangesPublishedBeforeTheBackgroundCompaction,
    /// 交出会话（卸载、回退之前）排空待发。
    PendingChangesPublishedBeforeHandingBackTheSession,
}

/// 攒批的会话报的一次发布：为什么发、发成了什么（推的抬 F、发成的每一版、之后发的 discard，同 [`UserChangePublished`]）。
#[derive(Debug)]
pub struct PublishReportedByTheSession {
    pub reason: ReasonOfThePublish,
    pub published: UserChangePublished,
}

/// [`MountedSessionWithPendingChanges::accept`] 收下了：怎么收的、这次调用里报的每一次发布（按先后）。
#[derive(Debug)]
pub struct ChangeAccepted {
    pub taken: HowTheChangeWasTaken,
    pub publishes: Vec<PublishReportedByTheSession>,
}

/// 一次改动在 [`MountedSessionWithPendingChanges::accept`] 里怎么收的。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HowTheChangeWasTaken {
    /// 收进待发；`journal_bytes_of_this_change` 是这次改动单独发布要写的 journal 字节（加进待发的合计）。收下之后触发了，待发连同它已经发成
    /// （报在 [`ChangeAccepted::publishes`] 里）。
    KeptPending {
        journal_bytes_of_this_change: JournalBytesToWrite,
    },
    /// 屏障：先排空待发、再单发，都已发成。
    PublishedAloneAsABarrier,
}

/// [`MountedSessionWithPendingChanges::accept`] 没收下，或收下之后触发的那一次发布没发成。成员按调用方要做的决定分。
#[derive(Debug)]
pub enum AcceptRefused {
    /// 分配器上冻结着一次写失败、没重发的发布（D23（journal 的角色与格式） 已定项 14；判决第二节「串行下不立的」那一格「冻结时 `accept` 拒」）：
    /// 在任何读写之前拒，待发不动、不长。调用方要做的是先原样重发那一次（交出会话，`transaction::resend_the_frozen_publish`）。
    PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg,
        instance: InstanceGeneration,
    },
    /// 同 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]，在任何读写之前。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 现行那一版没有可改的文件（树表 0 条；或有树而文件集合为空而这次是整份写），同 [`UserChangeRefused::NoFileVersionToChange`]，在任何读写之前。
    NoFileVersionToChange,
    /// 现行那一版的承诺类意图队列（`deleted_inodes`、销毁中的快照、截断条目）没走空：会话里每一次用户改动先走空（D8（核心索引结构） 已定项 16 第 6 条），
    /// 走空是写，而这次的预判要在任何读写之前做、要接在走空之后的那一版上判——两样合不到一起，条款没写，第一版不支持。在任何读写之前拒；
    /// 调用方要做的是先经 [`MountedSessionWithPendingChanges::publish_user_change`] 发一次（它先走空）。
    QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion,
    /// 按偏移写进不进待发、算数据写类还是屏障，条款没写（D16（发布语义） 已定项 10 只列整份写、建 inode 与屏障），第一版不支持。在任何读写之前拒；
    /// 调用方要做的是经 [`MountedSessionWithPendingChanges::publish_user_change`] 单发它。
    WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion,
    /// 交进来的是已经并好的一批（[`UserChange::DataChangesPublishedTogether`]）：把一批再收进待发条款没写，第一版不支持。在任何读写之前拒。
    DataChangesPublishedTogetherIntoThePendingChangesNotSupportedInTheFirstVersion,
    /// 待发里已经有另一个文件的整份写：一次发布写一个文件以内是这一版发布计划的形状（`transaction::PublishPlan::file`），不同 inode 的写并不并进一次发布
    /// 条款没写（D16（发布语义） 已定项 10「同一 inode 的写折叠」管不到两个号），第一版不支持。在任何读写之前拒；调用方要做的是先
    /// [`MountedSessionWithPendingChanges::publish_pending_now`]、再交这次的写。
    WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion {
        inode_pending: InodeNumber,
        inode_written: InodeNumber,
    },
    /// 写的号是待发里建 inode 那几次要发的号（还不在现行那一版的树里）：同一次发布里建一个 inode 又写它，条款没写，第一版不支持。在任何读写之前拒。
    WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion { inode: InodeNumber },
    /// 这一次改动单独发布就过不了落盘之前那几判（`transaction::journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write`：
    /// 号不在树里、删除中、内容装不进覆盖写的一个数据单元、准入、落点、记录条数……），`cause` 原样。在任何读写之前拒，待发不动。
    ChangeRefusedByThePublishJudgement(Box<PublishError>),
    /// 这一次改动单独过得了，并进待发合起来这一批过不了（判决第二节「一次发布里攒什么」那一格：`accept` 进门先做这一批合起来的准入与条数预判，
    /// 过不了就在任何读写之前拒，不在返回成功之后拒），`cause` 原样。在任何读写之前拒，待发不动；调用方要做的是先排空待发、再交这一次。
    PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(Box<PublishError>),
    /// 屏障进门之前排空待发没发成（`refusal` 原样；推的抬 F、发成的走空步在里面）：屏障没发。
    PublishOfThePendingChangesBeforeTheBarrierRefused(Box<UserChangeRefused>),
    /// 待发已排空（`publishes_before_the_refusal`），屏障自己没发成（`refusal` 原样）。
    BarrierRefused {
        publishes_before_the_refusal: Vec<PublishReportedByTheSession>,
        refusal: Box<UserChangeRefused>,
    },
    /// 这次改动已收进待发，收下之后触发的那一次发布没发成（`refusal` 原样）：没冻结时待发原样留着（连同这一次），下一次触发再发；
    /// 冻结了时待发并进了冻结的那一次（D16（发布语义） 已定项 17：报错之后这次发布可能已成也可能未成），待发清空。
    AcceptedButTheTriggeredPublishRefused(Box<UserChangeRefused>),
}

/// 排空待发那一次没发成（[`MountedSessionWithPendingChanges::publish_when_due`]、[`MountedSessionWithPendingChanges::publish_pending_now`]）：
/// `refusal` 原样。没冻结时待发原样留着；冻结了时待发并进了冻结的那一次、清空（D16（发布语义） 已定项 17）。
#[derive(Debug)]
pub struct PublishOfThePendingChangesRefused {
    pub refusal: Box<UserChangeRefused>,
}

/// [`MountedSessionWithPendingChanges::publish_user_change`] 没发成：之前排空的那一次（发成了就在 `publishes_before_the_refusal` 里）与这一次的拒绝。
#[derive(Debug)]
pub struct UserChangeAfterThePendingChangesRefused {
    pub publishes_before_the_refusal: Vec<PublishReportedByTheSession>,
    pub refusal: Box<UserChangeRefused>,
}

/// [`MountedSessionWithPendingChanges::run_compaction_in_the_background`] 做成了什么：判过起跑之后先排空的那一次（待发空、没起跑时 `None`）与后台入口这一次。
#[derive(Debug)]
pub struct BackgroundCompactionAfterThePendingChanges {
    pub pending_changes_published_before_the_run: Option<PublishReportedByTheSession>,
    pub call: BackgroundCompactionCall,
}

/// [`MountedSessionWithPendingChanges::run_compaction_in_the_background`] 没跑。成员按调用方要做的决定分。
#[derive(Debug)]
pub enum BackgroundCompactionAfterThePendingChangesRefused {
    /// 后台入口自己在任何读写之前拒（同 [`MountedSession::run_compaction_in_the_background`]）。
    CompactionRefused(CompactionRunRefused),
    /// 判过起跑、跑之前排空待发没发成：这一趟整理没跑，用户改动之后记下的「该跑了」放回去。
    PublishOfThePendingChangesRefused(Box<UserChangeRefused>),
}

/// [`MountedSessionWithPendingChanges::into_session_after_publishing_the_pending_changes`] 交不出会话：排空待发没发成，攒批的会话原样交回。
#[derive(Debug)]
pub struct HandingBackTheSessionRefused {
    pub session_with_pending_changes: MountedSessionWithPendingChanges,
    pub refusal: Box<UserChangeRefused>,
}

impl MountedSessionWithPendingChanges {
    /// 会话接上待发（一开始空的），两个阈值取这次挂载的有效值（[`PublishTriggerThresholds::of_the_mount`]：`T_time` 5000 ms，
    /// `T_dirty` = min(2 GiB, 环长 ÷ F)）。
    #[must_use]
    pub fn of_the_session(session: MountedSession) -> Self {
        let thresholds = PublishTriggerThresholds::of_the_mount(
            session
                .parameters_and_device_table
                .parameters_on_disk()
                .geometry
                .journal_ring_bytes,
        );
        Self::of_the_session_with_thresholds(session, thresholds)
    }

    /// 同 [`Self::of_the_session`]，两个阈值由调用方给：产品路径上只拿得到 [`PublishTriggerThresholds::of_the_mount`] 那一份，只供测试的那一份
    /// 在默认关的特性后面（[`PublishTriggerThresholds`] 文档）。
    #[must_use]
    pub fn of_the_session_with_thresholds(
        session: MountedSession,
        thresholds: PublishTriggerThresholds,
    ) -> Self {
        Self {
            session,
            pending: PendingDataChanges::empty(),
            thresholds,
            reading_of_the_last_publish: None,
        }
    }

    /// 底下那个会话（只读）。
    #[must_use]
    pub fn session(&self) -> &MountedSession {
        &self.session
    }

    /// 这次挂载生效的两个阈值。
    #[must_use]
    pub fn thresholds(&self) -> PublishTriggerThresholds {
        self.thresholds
    }

    /// 待发里收下了几次改动（排空之后归零）。
    #[must_use]
    pub fn pending_changes_accepted(&self) -> u64 {
        self.pending.changes_accepted
    }

    /// 待发里那几次改动各自单独发布要写的 journal 字节之和（脏量那一支比的量）。
    #[must_use]
    pub fn journal_bytes_of_the_pending_changes(&self) -> JournalBytesToWrite {
        self.pending.journal_bytes_of_the_changes_accepted
    }

    /// 上一次发布那次调用交进来的读数（还没发布过时是第一次见到的读数）。
    #[must_use]
    pub fn reading_of_the_last_publish(&self) -> Option<CallerMonotonicClockReading> {
        self.reading_of_the_last_publish
    }

    /// 第一次见到的读数当基线（还没发布过时「上次发布的时刻」取它）。
    fn note_the_reading_seen(&mut self, now: CallerMonotonicClockReading) {
        if self.reading_of_the_last_publish.is_none() {
            self.reading_of_the_last_publish = Some(now);
        }
    }

    /// 发成一次：上次发布的读数取这次调用交进来的那一个，只往前走（读数倒退时不回拨）。
    fn note_a_publish(&mut self, now: CallerMonotonicClockReading) {
        self.reading_of_the_last_publish = Some(match self.reading_of_the_last_publish {
            Some(before) => before.max(now),
            None => now,
        });
    }

    /// 此刻（`now`）触不触发：读数倒退时间隔是 0（当作没过时间）。
    fn trigger_cause_at(&self, now: CallerMonotonicClockReading) -> Option<PublishTriggerCause> {
        let time_since_the_last_publish = match self.reading_of_the_last_publish {
            Some(last) => now.0.saturating_sub(last.0),
            None => Duration::ZERO,
        };
        publish_trigger_cause(
            self.pending.is_empty(),
            time_since_the_last_publish,
            self.pending.journal_bytes_of_the_changes_accepted,
            self.thresholds,
        )
    }

    /// 把待发整个并成一次发布交给会话（[`UserChange::DataChangesPublishedTogether`]，先走空队列、被空间拒照旧推抬 F 再判）。待发空就什么都不发、交回 `None`
    /// （不推空发布）。发成：待发清空、记下这次的读数。没发成：没冻结时待发原样放回；冻结了时待发并进了冻结的那一次，不放回（D16（发布语义） 已定项 17）。
    fn publish_the_pending_changes<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        now: CallerMonotonicClockReading,
        reason: ReasonOfThePublish,
    ) -> Result<Option<PublishReportedByTheSession>, Box<UserChangeRefused>> {
        if self.pending.is_empty() {
            return Ok(None);
        }
        let pending = std::mem::replace(&mut self.pending, PendingDataChanges::empty());
        match self.session.publish_user_change(
            devices,
            UserChange::DataChangesPublishedTogether(pending.as_published_together()),
        ) {
            Ok(published) => {
                self.note_a_publish(now);
                Ok(Some(PublishReportedByTheSession { reason, published }))
            }
            Err(refusal) => {
                if self.session.allocator.frozen_publish().is_none() {
                    self.pending = pending;
                }
                Err(Box::new(refusal))
            }
        }
    }

    /// 收下一次用户改动（D16（发布语义） 已定项 10，判决第二节「一次发布里攒什么」「串行下不立的」两格）：
    /// 1. 冻结着一次没重发的发布、交进来的盘表与挂载核过的不同：在任何读写之前拒。
    /// 2. 屏障（unlink、截断、建删快照）：先排空待发（一次发布）、再照 [`MountedSession::publish_user_change`] 单发，两次都报。
    /// 3. 数据写类（整份写、建 inode）：在任何读写之前先判——现行那一版有可改的文件、队列走空了、并得进待发（至多一个文件、不写待发里建的号）、
    ///    这一次单独发布过得了落盘之前那几判、并进待发合起来的这一批也过得了（准入与条数预判，`transaction::journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write`）；
    ///    有一样不过就拒，一个盘都不读、一个写都不发、待发不动。都过了才收进待发（同一 inode 的写折叠、建 inode 追加），脏量加上这一次单独发布要写的 journal 字节；
    ///    收下之后判一次触发（[`publish_trigger_cause`]），触发就把待发整个并成一次发布。
    /// 4. 按偏移写、已经并好的一批：条款没写，在任何读写之前拒。
    ///
    /// 整理不在这里跑：发成之后起跑判据为真只记「该跑了」（[`MountedSession::publish_user_change`]），由后台入口去跑。
    ///
    /// # Errors
    /// [`AcceptRefused`] 的几种。
    pub fn accept<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        change: UserChange<'_>,
        now: CallerMonotonicClockReading,
    ) -> Result<ChangeAccepted, AcceptRefused> {
        self.note_the_reading_seen(now);
        if let Some(frozen) = self.session.allocator.frozen_publish() {
            return Err(
                AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: frozen.checkpoint_txg(),
                    instance: frozen.instance(),
                },
            );
        }
        if !self
            .session
            .parameters_and_device_table
            .is_the_device_table_of_the_mount(devices)
        {
            return Err(AcceptRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: self
                    .session
                    .parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        let (data_change, contract) = match ChangeAtTheAccept::of(change) {
            ChangeAtTheAccept::Barrier => return self.publish_a_barrier_alone(devices, change, now),
            ChangeAtTheAccept::WriteAtOffset => {
                return Err(
                    AcceptRefused::WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion,
                )
            }
            ChangeAtTheAccept::DataChangesPublishedTogether => {
                return Err(AcceptRefused::DataChangesPublishedTogetherIntoThePendingChangesNotSupportedInTheFirstVersion)
            }
            ChangeAtTheAccept::Data { change, contract } => (change, contract),
        };
        let journal_bytes_of_this_change =
            self.judge_a_data_change_before_any_read_or_write(devices, data_change, contract)?;
        self.pending
            .take_in(data_change, journal_bytes_of_this_change);
        let mut publishes = Vec::new();
        if let Some(cause) = self.trigger_cause_at(now) {
            if let Some(published) = self
                .publish_the_pending_changes(
                    devices,
                    now,
                    ReasonOfThePublish::TriggeredAfterTheAccept(cause),
                )
                .map_err(AcceptRefused::AcceptedButTheTriggeredPublishRefused)?
            {
                publishes.push(published);
            }
        }
        Ok(ChangeAccepted {
            taken: HowTheChangeWasTaken::KeptPending {
                journal_bytes_of_this_change,
            },
            publishes,
        })
    }

    /// [`Self::accept`] 收数据写类改动之前那几判，在任何读写之前：交回这一次单独发布要写的 journal 字节。
    ///
    /// # Errors
    /// [`AcceptRefused::NoFileVersionToChange`]、`QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`、
    /// `WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion`、`WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion`、
    /// `ChangeRefusedByThePublishJudgement`、`PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement`。
    fn judge_a_data_change_before_any_read_or_write<Device: BlockDevice>(
        &self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        data_change: DataChangeAccepted<'_>,
        contract: WholeWriteContractAtTheAccept,
    ) -> Result<JournalBytesToWrite, AcceptRefused> {
        let current_version = match (&self.session.current, data_change) {
            (PoolVersion::WithFile(version), _)
            | (PoolVersion::WithTreesAndNoFile(version), DataChangeAccepted::NewInodes(_)) => {
                version
            }
            (PoolVersion::WithTreesAndNoFile(_), DataChangeAccepted::WholeWrite(_))
            | (PoolVersion::WithoutFile(_), _) => return Err(AcceptRefused::NoFileVersionToChange),
        };
        if !current_version.queue_of_committed_intents_is_empty() {
            return Err(AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion);
        }
        if let DataChangeAccepted::WholeWrite(whole_write) = data_change {
            if let Some(pending_whole_write) = &self.pending.whole_write {
                if pending_whole_write.inode != whole_write.inode {
                    return Err(
                        AcceptRefused::WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion {
                            inode_pending: pending_whole_write.inode,
                            inode_written: whole_write.inode,
                        },
                    );
                }
            }
            let first_number_created_by_the_pending_changes =
                current_version.inode_number_watermark();
            let is_created_by_the_pending_changes = whole_write.inode.0
                >= first_number_created_by_the_pending_changes
                && whole_write.inode.0 - first_number_created_by_the_pending_changes
                    < self.pending.new_inodes_in_total();
            if is_created_by_the_pending_changes {
                return Err(
                    AcceptRefused::WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion {
                        inode: whole_write.inode,
                    },
                );
            }
            if contract == WholeWriteContractAtTheAccept::OneDataUnit
                && whole_write.content.len() > data_unit_payload_capacity()
            {
                return Err(AcceptRefused::ChangeRefusedByThePublishJudgement(Box::new(
                    PublishError::ContentExceedsDataUnit {
                        bytes: whole_write.content.len(),
                        capacity: data_unit_payload_capacity(),
                    },
                )));
            }
        }
        let parameters = self
            .session
            .parameters_and_device_table
            .parameters_on_disk();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        writer.set_container_tree_capacities(
            self.session
                .allocator
                .deletion_switches()
                .container_tree_capacities(),
        );
        let mut slot_of_the_new_inodes = [NewInodesOfOneChange {
            count: 0,
            write_time_seconds: 0,
        }];
        let journal_bytes_of_this_change =
            journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write(
                &writer,
                &self.session.allocator,
                current_version,
                data_change.alone(&mut slot_of_the_new_inodes),
                self.session.instance,
            )
            .map_err(|cause| AcceptRefused::ChangeRefusedByThePublishJudgement(Box::new(cause)))?;
        let mut new_inodes_together = self.pending.new_inodes.clone();
        let whole_write_together = match data_change {
            DataChangeAccepted::WholeWrite(whole_write) => Some(whole_write),
            DataChangeAccepted::NewInodes(created) => {
                new_inodes_together.push(created);
                self.pending
                    .whole_write
                    .as_ref()
                    .map(|pending_whole_write| WholeWriteOfOneInode {
                        inode: pending_whole_write.inode,
                        content: &pending_whole_write.content,
                        write_time_seconds: pending_whole_write.write_time_seconds,
                    })
            }
        };
        journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write(
            &writer,
            &self.session.allocator,
            current_version,
            DataChangesPublishedTogether {
                whole_write: whole_write_together,
                new_inodes: &new_inodes_together,
            },
            self.session.instance,
        )
        .map_err(|cause| {
            AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(
                Box::new(cause),
            )
        })?;
        Ok(journal_bytes_of_this_change)
    }

    /// 屏障进门：先排空待发、再单发。
    fn publish_a_barrier_alone<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        change: UserChange<'_>,
        now: CallerMonotonicClockReading,
    ) -> Result<ChangeAccepted, AcceptRefused> {
        let mut publishes = Vec::new();
        if let Some(drained) = self
            .publish_the_pending_changes(
                devices,
                now,
                ReasonOfThePublish::PendingChangesPublishedBeforeABarrier,
            )
            .map_err(AcceptRefused::PublishOfThePendingChangesBeforeTheBarrierRefused)?
        {
            publishes.push(drained);
        }
        match self.session.publish_user_change(devices, change) {
            Ok(published) => {
                self.note_a_publish(now);
                publishes.push(PublishReportedByTheSession {
                    reason: ReasonOfThePublish::BarrierPublishedAlone,
                    published,
                });
                Ok(ChangeAccepted {
                    taken: HowTheChangeWasTaken::PublishedAloneAsABarrier,
                    publishes,
                })
            }
            Err(refusal) => Err(AcceptRefused::BarrierRefused {
                publishes_before_the_refusal: publishes,
                refusal: Box::new(refusal),
            }),
        }
    }

    /// 调用方按周期调（D16（发布语义） 已定项 2、5：丢失窗口里的 P 是这个周期，归调用方）：判一次触发，触发就把待发整个并成一次发布。
    /// 待发空、或两支都没到，就什么都不发、交回 `None`，一个盘都不读。
    ///
    /// # Errors
    /// 触发的那一次没发成：[`PublishOfThePendingChangesRefused`]。
    pub fn publish_when_due<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        now: CallerMonotonicClockReading,
    ) -> Result<Option<PublishReportedByTheSession>, PublishOfThePendingChangesRefused> {
        self.note_the_reading_seen(now);
        let Some(cause) = self.trigger_cause_at(now) else {
            return Ok(None);
        };
        self.publish_the_pending_changes(devices, now, ReasonOfThePublish::TriggeredWhenDue(cause))
            .map_err(|refusal| PublishOfThePendingChangesRefused { refusal })
    }

    /// 立即排空（D16（发布语义） 已定项 11：只是提前触发一次发布，同一个 journal、同一条发布路径）：待发非空就整个并成一次发布；待发空就什么都不发、交回 `None`
    /// （不推空发布）。
    ///
    /// # Errors
    /// 同 [`Self::publish_when_due`]。
    pub fn publish_pending_now<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        now: CallerMonotonicClockReading,
    ) -> Result<Option<PublishReportedByTheSession>, PublishOfThePendingChangesRefused> {
        self.note_the_reading_seen(now);
        self.publish_the_pending_changes(
            devices,
            now,
            ReasonOfThePublish::PendingChangesPublishedOnRequest,
        )
        .map_err(|refusal| PublishOfThePendingChangesRefused { refusal })
    }

    /// 原入口（[`MountedSession::publish_user_change`]）语义不变，只多一条先排空待发：待发非空就先并成一次发布，再交这一次改动（任何一种）单发。
    ///
    /// # Errors
    /// 排空或这一次没发成：[`UserChangeAfterThePendingChangesRefused`]（排空发成了的那一次在里面）。
    pub fn publish_user_change<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        change: UserChange<'_>,
        now: CallerMonotonicClockReading,
    ) -> Result<Vec<PublishReportedByTheSession>, UserChangeAfterThePendingChangesRefused> {
        self.note_the_reading_seen(now);
        let mut publishes = Vec::new();
        match self.publish_the_pending_changes(
            devices,
            now,
            ReasonOfThePublish::PendingChangesPublishedBeforeTheUserChange,
        ) {
            Ok(Some(drained)) => publishes.push(drained),
            Ok(None) => {}
            Err(refusal) => {
                return Err(UserChangeAfterThePendingChangesRefused {
                    publishes_before_the_refusal: publishes,
                    refusal,
                })
            }
        }
        match self.session.publish_user_change(devices, change) {
            Ok(published) => {
                self.note_a_publish(now);
                publishes.push(PublishReportedByTheSession {
                    reason: ReasonOfThePublish::UserChangePublishedAlone,
                    published,
                });
                Ok(publishes)
            }
            Err(refusal) => Err(UserChangeAfterThePendingChangesRefused {
                publishes_before_the_refusal: publishes,
                refusal: Box::new(refusal),
            }),
        }
    }

    /// 整理的后台入口（[`MountedSession::run_compaction_in_the_background`]）连同待发：整理是屏障——待发里不并进整理的发布，判过起跑之后、
    /// 那一趟起跑之前先把待发排空（判决第二节「一次发布里攒什么」那一格；整理仍只在后台入口里跑）。没起跑就不排空。
    /// 排空那一次是一次用户改动的发布：`B_gc` 从它重新数、起跑判据为真记「该跑了」（同 [`MountedSession::publish_user_change`]）。
    ///
    /// # Errors
    /// [`BackgroundCompactionAfterThePendingChangesRefused`] 的两种。
    pub fn run_compaction_in_the_background<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
    ) -> Result<
        BackgroundCompactionAfterThePendingChanges,
        BackgroundCompactionAfterThePendingChangesRefused,
    > {
        let now = caller.clock_reading();
        self.note_the_reading_seen(now);
        let (trigger, reach, budget) = match self
            .session
            .whether_the_background_compaction_starts(devices, pacing, caller)
            .map_err(BackgroundCompactionAfterThePendingChangesRefused::CompactionRefused)?
        {
            BackgroundCompactionStart::NotStarted(not_started) => {
                return Ok(BackgroundCompactionAfterThePendingChanges {
                    pending_changes_published_before_the_run: None,
                    call: BackgroundCompactionCall::NotStarted(not_started),
                })
            }
            BackgroundCompactionStart::Starts {
                trigger,
                reach,
                budget,
            } => (trigger, reach, budget),
        };
        let pending_changes_published_before_the_run = match self.publish_the_pending_changes(
            devices,
            now,
            ReasonOfThePublish::PendingChangesPublishedBeforeTheBackgroundCompaction,
        ) {
            Ok(drained) => drained,
            Err(refusal) => {
                match trigger {
                    BackgroundCompactionTrigger::DueAfterAUserChange => {
                        self.session.compaction.start_judgement =
                            CompactionStartJudgement::DueAfterAUserChange;
                    }
                    BackgroundCompactionTrigger::TimerFallback => {}
                }
                return Err(
                    BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused(
                        refusal,
                    ),
                );
            }
        };
        let call = self.session.run_the_background_compaction_that_starts(
            devices, pacing, caller, trigger, reach, budget,
        );
        let has_published = match &call {
            BackgroundCompactionCall::Ran { run, .. } => !run.versions_published.is_empty(),
            BackgroundCompactionCall::NotStarted(_) => false,
        };
        if has_published {
            self.note_a_publish(now);
        }
        Ok(BackgroundCompactionAfterThePendingChanges {
            pending_changes_published_before_the_run,
            call,
        })
    }

    /// 交出会话（卸载、回退、重发冻结的那一次要的是会话本身）：先把待发排空，交回排空那一次（待发空时 `None`）与会话。类型上卸载、回退之前待发一定空了
    /// （判决第二节「一次发布里攒什么」那一格：回退、卸载是屏障）。
    ///
    /// # Errors
    /// 排空没发成：[`HandingBackTheSessionRefused`]，攒批的会话原样交回（没冻结时待发还在里面）。
    pub fn into_session_after_publishing_the_pending_changes<Device: BlockDevice>(
        mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        now: CallerMonotonicClockReading,
    ) -> Result<
        (Option<PublishReportedByTheSession>, MountedSession),
        Box<HandingBackTheSessionRefused>,
    > {
        self.note_the_reading_seen(now);
        match self.publish_the_pending_changes(
            devices,
            now,
            ReasonOfThePublish::PendingChangesPublishedBeforeHandingBackTheSession,
        ) {
            Ok(drained) => Ok((drained, self.session)),
            Err(refusal) => Err(Box::new(HandingBackTheSessionRefused {
                session_with_pending_changes: self,
                refusal,
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    //! 只进内存的盘（[`DeviceKeepingWritesInMemory`]）：写与写零只进内存、盖在之后的读上，读没写过的块穿到底下那块盘，底下那块盘一字不动；
    //! 请求判据与两个后端是同一份（`block_device::check_aligned_and_in_range`）。差分对拍照正推腿的做法
    //! （`research/prompts/m3-batch10z-tu-code-r1-sonnet-model/src/core-mounted-session-debug.diff` 的 `sonnet_k5_differential`），
    //! 参照换成一块把每个写直接写进去的字节盘（同一份请求判据）。
    use super::*;

    /// 物理块 512 字节、16 块的盘。
    const PHYSICAL_BLOCK_BYTES: u32 = 512;
    const BLOCKS_OF_THE_DISK: u64 = 16;

    /// 底下那块盘（也是差分对拍的参照）：整块字节放内存，请求判据照两个后端那一份。
    struct DeviceOfBytes {
        bytes: Vec<u8>,
        physical_block_size: PhysicalBlockSizeInBytes,
    }

    impl DeviceOfBytes {
        /// 物理块 [`PHYSICAL_BLOCK_BYTES`]、[`BLOCKS_OF_THE_DISK`] 块的盘，每个字节按位置与 `seed` 填一个图样：读回哪一块看得出来。
        fn filled_with_a_pattern(seed: u8) -> Self {
            Self::filled_with_a_pattern_on_blocks_of(
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                BLOCKS_OF_THE_DISK,
                seed,
            )
        }

        /// 物理块 `physical_block_size`、`blocks_of_the_disk` 块的盘，图样同 [`Self::filled_with_a_pattern`]。
        fn filled_with_a_pattern_on_blocks_of(
            physical_block_size: PhysicalBlockSizeInBytes,
            blocks_of_the_disk: u64,
            seed: u8,
        ) -> Self {
            let device_bytes =
                usize::try_from(u64::from(physical_block_size.0) * blocks_of_the_disk)
                    .expect("盘长装得进 usize");
            DeviceOfBytes {
                bytes: (0..device_bytes)
                    .map(|position| {
                        u8::try_from(position % 251)
                            .expect("小于 256")
                            .wrapping_add(seed)
                    })
                    .collect(),
                physical_block_size,
            }
        }

        fn range_of(
            &self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<std::ops::Range<usize>, BlockDeviceError> {
            check_aligned_and_in_range(
                offset,
                length,
                self.physical_block_size,
                self.size_in_bytes(),
            )?;
            let start = usize::try_from(offset.0).expect("判过不越过盘尾");
            let end = start + usize::try_from(length).expect("判过不越过盘尾");
            Ok(start..end)
        }
    }

    impl BlockDevice for DeviceOfBytes {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, u64::try_from(buffer.len()).expect("读长"))?;
            buffer.copy_from_slice(&self.bytes[range]);
            Ok(())
        }

        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            _durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, u64::try_from(bytes.len()).expect("写长"))?;
            self.bytes[range].copy_from_slice(bytes);
            Ok(())
        }

        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, length)?;
            self.bytes[range].fill(0);
            Ok(())
        }

        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }

        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            self.physical_block_size
        }

        fn size_in_bytes(&self) -> u64 {
            u64::try_from(self.bytes.len()).expect("盘长装得进 u64")
        }
    }

    fn offset_of_block(block: u64) -> DeviceOffsetInBytes {
        DeviceOffsetInBytes(block * u64::from(PHYSICAL_BLOCK_BYTES))
    }

    fn bytes_of_blocks(blocks: u64) -> usize {
        usize::try_from(blocks * u64::from(PHYSICAL_BLOCK_BYTES)).expect("块数装得进 usize")
    }

    /// 从 `first_block` 起读 `blocks` 块；读缓冲先填 0xAA，没读到的那一截露得出来。
    fn read_blocks(device: &dyn BlockDevice, first_block: u64, blocks: u64) -> Vec<u8> {
        let mut buffer = vec![0xAA_u8; bytes_of_blocks(blocks)];
        device
            .read_at(offset_of_block(first_block), &mut buffer)
            .expect("对齐、没越过盘尾的读做得成");
        buffer
    }

    fn bytes_of_the_disk_blocks(disk: &DeviceOfBytes, first_block: u64, blocks: u64) -> Vec<u8> {
        let start = bytes_of_blocks(first_block);
        disk.bytes[start..start + bytes_of_blocks(blocks)].to_vec()
    }

    #[test]
    fn a_write_kept_in_memory_reads_back_and_leaves_the_disk_below_unchanged() {
        let disk = DeviceOfBytes::filled_with_a_pattern(0);
        let bytes_before = disk.bytes.clone();
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let written = vec![0x5A_u8; bytes_of_blocks(1)];
        kept.write_at(
            offset_of_block(3),
            &written,
            WriteDurability::ForceUnitAccess,
        )
        .expect("对齐、没越过盘尾的写做得成");
        assert_eq!(
            read_blocks(&kept, 3, 1),
            written,
            "写过的块读回写进去的字节"
        );
        assert!(
            disk.bytes == bytes_before,
            "写只进内存：底下那块盘一个字节都没变"
        );
    }

    #[test]
    fn a_read_of_blocks_never_written_passes_through_to_the_disk_below() {
        let disk = DeviceOfBytes::filled_with_a_pattern(7);
        let kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        assert_eq!(
            read_blocks(&kept, 5, 3),
            bytes_of_the_disk_blocks(&disk, 5, 3),
            "没写过的块读穿到底下那块盘"
        );
    }

    #[test]
    fn a_read_across_written_and_unwritten_blocks_stitches_the_kept_blocks_with_the_disk_below() {
        let disk = DeviceOfBytes::filled_with_a_pattern(11);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let block_two = vec![0x22_u8; bytes_of_blocks(1)];
        let block_four = vec![0x44_u8; bytes_of_blocks(1)];
        let blocks_seven_and_eight: Vec<u8> = (0..bytes_of_blocks(2))
            .map(|position| u8::try_from(position % 199).expect("小于 256"))
            .collect();
        for (first_block, bytes) in [
            (2, &block_two),
            (4, &block_four),
            (7, &blocks_seven_and_eight),
        ] {
            kept.write_at(offset_of_block(first_block), bytes, WriteDurability::Plain)
                .expect("对齐、没越过盘尾的写做得成");
        }
        let mut expected = bytes_of_the_disk_blocks(&disk, 1, 9);
        for (first_block, bytes) in [
            (2, &block_two),
            (4, &block_four),
            (7, &blocks_seven_and_eight),
        ] {
            let start = bytes_of_blocks(first_block - 1);
            expected[start..start + bytes.len()].copy_from_slice(bytes);
        }
        assert!(
            read_blocks(&kept, 1, 9) == expected,
            "块 1 到 9 一次读：写过的块（2、4、7、8）读回写进去的字节，没写过的（1、3、5、6、9，含末尾那一段）读穿到底下那块盘"
        );
    }

    #[test]
    fn zeroes_written_in_memory_read_back_as_zero_over_the_disk_below() {
        let disk = DeviceOfBytes::filled_with_a_pattern(13);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        kept.write_zeroes_at(offset_of_block(3), u64::from(PHYSICAL_BLOCK_BYTES) * 3)
            .expect("对齐、没越过盘尾的写零做得成");
        let mut expected = bytes_of_the_disk_blocks(&disk, 2, 5);
        expected[bytes_of_blocks(1)..bytes_of_blocks(4)].fill(0);
        assert!(
            read_blocks(&kept, 2, 5) == expected,
            "写零的块 3 到 5 读回全 0，两头的块 2、6 读穿到底下那块盘"
        );
        assert!(
            bytes_of_the_disk_blocks(&disk, 3, 3)
                .iter()
                .any(|byte| *byte != 0),
            "底下那块盘那一段不是全 0（阳性对照：读回的 0 是写零记下的）"
        );
    }

    #[test]
    fn a_request_off_the_physical_block_boundaries_is_refused_before_anything_is_kept() {
        let disk = DeviceOfBytes::filled_with_a_pattern(17);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let unaligned_offset = DeviceOffsetInBytes(u64::from(PHYSICAL_BLOCK_BYTES) + 100);
        let one_block = vec![0x33_u8; bytes_of_blocks(1)];
        let short_of_a_block = vec![0x33_u8; 100];
        let refusals = [
            (
                "偏移不在物理块边界上的写",
                kept.write_at(unaligned_offset, &one_block, WriteDurability::Plain),
                unaligned_offset,
                u64::from(PHYSICAL_BLOCK_BYTES),
            ),
            (
                "长度不是物理块整数倍的写",
                kept.write_at(
                    offset_of_block(2),
                    &short_of_a_block,
                    WriteDurability::Plain,
                ),
                offset_of_block(2),
                100,
            ),
            (
                "长度不是物理块整数倍的写零",
                kept.write_zeroes_at(offset_of_block(4), 100),
                offset_of_block(4),
                100,
            ),
            (
                "偏移不在物理块边界上的读",
                kept.read_at(unaligned_offset, &mut vec![0_u8; bytes_of_blocks(1)]),
                unaligned_offset,
                u64::from(PHYSICAL_BLOCK_BYTES),
            ),
        ];
        for (request, outcome, offset, length) in refusals {
            assert!(
                matches!(
                    outcome,
                    Err(BlockDeviceError::Unaligned {
                        offset: refused_offset,
                        length: refused_length,
                        physical_block_size: PHYSICAL_BLOCK_BYTES,
                    }) if refused_offset == offset && refused_length == length
                ),
                "{request}：拒成 Unaligned，带偏移 {} 与长度 {length}，而交回 {outcome:?}",
                offset.0
            );
        }
        assert!(
            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == disk.bytes,
            "被拒的写一个块都没记：整盘读回底下那块盘的字节"
        );
    }

    #[test]
    fn a_request_past_the_end_of_the_disk_below_is_refused_and_one_ending_at_the_end_is_kept() {
        let disk = DeviceOfBytes::filled_with_a_pattern(19);
        let device_size = disk.size_in_bytes();
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let last_block = vec![0x77_u8; bytes_of_blocks(1)];
        kept.write_at(
            offset_of_block(BLOCKS_OF_THE_DISK - 1),
            &last_block,
            WriteDurability::Plain,
        )
        .expect("最后一块正好写到盘尾：放行");
        assert_eq!(
            read_blocks(&kept, BLOCKS_OF_THE_DISK - 1, 1),
            last_block,
            "最后一块读回写进去的字节"
        );
        let two_blocks = vec![0x66_u8; bytes_of_blocks(2)];
        let offset_near_the_top =
            DeviceOffsetInBytes(u64::MAX - u64::from(PHYSICAL_BLOCK_BYTES) + 1);
        let refusals = [
            (
                "从最后一块起写两块（越过盘尾一块）",
                kept.write_at(
                    offset_of_block(BLOCKS_OF_THE_DISK - 1),
                    &two_blocks,
                    WriteDurability::Plain,
                ),
                offset_of_block(BLOCKS_OF_THE_DISK - 1),
                u64::from(PHYSICAL_BLOCK_BYTES) * 2,
            ),
            (
                "从盘尾起写零一块",
                kept.write_zeroes_at(
                    offset_of_block(BLOCKS_OF_THE_DISK),
                    u64::from(PHYSICAL_BLOCK_BYTES),
                ),
                offset_of_block(BLOCKS_OF_THE_DISK),
                u64::from(PHYSICAL_BLOCK_BYTES),
            ),
            (
                "偏移加长度越过 u64 的读",
                kept.read_at(offset_near_the_top, &mut vec![0_u8; bytes_of_blocks(2)]),
                offset_near_the_top,
                u64::from(PHYSICAL_BLOCK_BYTES) * 2,
            ),
        ];
        for (request, outcome, offset, length) in refusals {
            assert!(
                matches!(
                    outcome,
                    Err(BlockDeviceError::OutOfRange {
                        offset: refused_offset,
                        length: refused_length,
                        device_size: refused_device_size,
                    }) if refused_offset == offset && refused_length == length && refused_device_size == device_size
                ),
                "{request}：拒成 OutOfRange，带偏移 {}、长度 {length} 与盘长 {device_size}，而交回 {outcome:?}",
                offset.0
            );
        }
        let mut expected = disk.bytes.clone();
        expected[bytes_of_blocks(BLOCKS_OF_THE_DISK - 1)..].copy_from_slice(&last_block);
        assert!(
            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == expected,
            "被拒的写一个块都没记：整盘读回只有最后一块是写进去的"
        );
    }

    /// 差分对拍那一串请求的个数。
    const REQUESTS_OF_THE_RANDOM_STREAM: u64 = 20_000;

    /// 线性同余，取高 33 位（只要确定、够散）。
    fn next_random(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *state >> 33
    }

    /// 偏移：多半落在物理块边界上、盘以内，少数不对齐、正好在盘尾、越过盘尾、贴着 u64 的顶。
    fn random_offset(state: &mut u64, device_size: u64, block_bytes: u64) -> u64 {
        match next_random(state) % 8 {
            0 => next_random(state) % (device_size + 2 * block_bytes),
            1 => u64::MAX - next_random(state) % 1024,
            2 => device_size,
            _ => (next_random(state) % (device_size / block_bytes)) * block_bytes,
        }
    }

    /// 长度：多半是一到几个物理块，少数是 0、不对齐、跨过盘尾。
    fn random_length(state: &mut u64, device_size: u64, block_bytes: u64) -> u64 {
        match next_random(state) % 8 {
            0 => 0,
            1 => next_random(state) % (2 * device_size),
            _ => (1 + next_random(state) % 4) * block_bytes,
        }
    }

    /// 一串随机请求（读、写、写零，偶尔一道屏障）同时交给盖在底下那块盘上的只进内存的盘与一块把写直接写进去的同样的盘之后看到的。
    struct RandomRequestStreamAgainstADeviceWrittenThrough {
        /// 两边对不上的请求，逐条写明第几个、什么请求、哪一样不同（读回的字节、交回值）。
        disagreements: Vec<String>,
        refused_requests: u64,
        kept_writes: u64,
        /// 一串请求之后整盘读回的字节与写穿的那块盘相同。
        whole_disk_reads_back_as_written_through: bool,
        /// 一串请求之后底下那块盘一个字节都没变。
        disk_below_is_unchanged: bool,
    }

    /// 物理块 `physical_block_size`、`blocks_of_the_disk` 块、图样种子 `seed` 的盘上跑 [`REQUESTS_OF_THE_RANDOM_STREAM`] 个随机请求。
    fn random_request_stream_against_a_device_written_through(
        physical_block_size: PhysicalBlockSizeInBytes,
        blocks_of_the_disk: u64,
        seed: u8,
    ) -> RandomRequestStreamAgainstADeviceWrittenThrough {
        let disk = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(
            physical_block_size,
            blocks_of_the_disk,
            seed,
        );
        let bytes_before = disk.bytes.clone();
        let device_size = disk.size_in_bytes();
        let block_bytes = u64::from(physical_block_size.0);
        let mut written_through = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(
            physical_block_size,
            blocks_of_the_disk,
            seed,
        );
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let mut state = 1_u64;
        let mut disagreements: Vec<String> = Vec::new();
        let mut refused_requests = 0_u64;
        let mut kept_writes = 0_u64;
        // 迭代上界是请求个数；跨轮带着两块盘、随机数的状态与两个计数；没有提前出口。
        for request_number in 1..=REQUESTS_OF_THE_RANDOM_STREAM {
            let offset = DeviceOffsetInBytes(random_offset(&mut state, device_size, block_bytes));
            let length = random_length(&mut state, device_size, block_bytes);
            let length_in_memory = usize::try_from(length).expect("长度不过两倍盘长");
            let (request, through, in_memory) = match next_random(&mut state) % 3 {
                0 => {
                    let mut through_buffer = vec![0xAA_u8; length_in_memory];
                    let mut in_memory_buffer = vec![0xAA_u8; length_in_memory];
                    let through = written_through.read_at(offset, &mut through_buffer);
                    let in_memory = kept.read_at(offset, &mut in_memory_buffer);
                    if through_buffer != in_memory_buffer {
                        disagreements.push(format!(
                            "第 {request_number} 个请求（读 {} 起 {length} 字节）：读回的字节不同",
                            offset.0
                        ));
                    }
                    ("读", through, in_memory)
                }
                1 => {
                    let bytes: Vec<u8> = (0..length_in_memory)
                        .map(|_| u8::try_from(next_random(&mut state) % 256).expect("小于 256"))
                        .collect();
                    (
                        "写",
                        written_through.write_at(offset, &bytes, WriteDurability::Plain),
                        kept.write_at(offset, &bytes, WriteDurability::ForceUnitAccess),
                    )
                }
                _ => (
                    "写零",
                    written_through.write_zeroes_at(offset, length),
                    kept.write_zeroes_at(offset, length),
                ),
            };
            let (through_returned, in_memory_returned) =
                (format!("{through:?}"), format!("{in_memory:?}"));
            if through_returned != in_memory_returned {
                disagreements.push(format!(
                    "第 {request_number} 个请求（{request} {} 起 {length} 字节）：写穿的盘交回 {through_returned}，只进内存的盘交回 {in_memory_returned}",
                    offset.0
                ));
            }
            match through {
                Ok(()) => kept_writes += u64::from(request != "读"),
                Err(_) => refused_requests += 1,
            }
            if next_random(&mut state).is_multiple_of(50) {
                kept.barrier().expect("屏障什么都不做");
            }
        }
        let mut whole_disk =
            vec![0xAA_u8; usize::try_from(device_size).expect("盘长装得进 usize")];
        kept.read_at(DeviceOffsetInBytes(0), &mut whole_disk)
            .expect("整盘一次读：对齐、没越过盘尾");
        RandomRequestStreamAgainstADeviceWrittenThrough {
            disagreements,
            refused_requests,
            kept_writes,
            whole_disk_reads_back_as_written_through: whole_disk == written_through.bytes,
            disk_below_is_unchanged: disk.bytes == bytes_before,
        }
    }

    /// 物理块 512 字节的盘上：只进内存的盘与写穿的盘逐个请求交回值、读回的字节相同，整盘读回相同，底下那块盘一字不动。
    #[test]
    fn the_device_keeping_writes_in_memory_agrees_with_a_device_written_through_on_a_random_request_stream(
    ) {
        let stream = random_request_stream_against_a_device_written_through(
            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            BLOCKS_OF_THE_DISK,
            23,
        );
        assert!(
            stream.disagreements.is_empty(),
            "每个请求两边交回值与读回的字节相同，而 {} 个对不上，头几个：{:?}",
            stream.disagreements.len(),
            stream.disagreements.iter().take(5).collect::<Vec<_>>()
        );
        assert!(
            stream.refused_requests > 0 && stream.kept_writes > 0,
            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {} 个、写 {} 个",
            stream.refused_requests,
            stream.kept_writes
        );
        assert!(
            stream.whole_disk_reads_back_as_written_through,
            "整盘读回与写穿的那块盘相同"
        );
        assert!(
            stream.disk_below_is_unchanged,
            "一串写之后底下那块盘一个字节都没变"
        );
    }

    /// 物理块 4096 字节的盘（正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md` K2：入库用例原来只在 512 的盘上，
    /// 判据里把物理块写死 512 照绿）：同一串随机请求按 4096 的块取偏移与长度，两边逐个请求相同、整盘读回相同、底下那块盘一字不动。
    #[test]
    fn the_device_keeping_writes_in_memory_on_4096_byte_physical_blocks_agrees_with_a_device_written_through_on_a_random_request_stream(
    ) {
        let stream = random_request_stream_against_a_device_written_through(
            PhysicalBlockSizeInBytes(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES),
            BLOCKS_OF_THE_DISK,
            29,
        );
        assert!(
            stream.disagreements.is_empty(),
            "每个请求两边交回值与读回的字节相同，而 {} 个对不上，头几个：{:?}",
            stream.disagreements.len(),
            stream.disagreements.iter().take(5).collect::<Vec<_>>()
        );
        assert!(
            stream.refused_requests > 0 && stream.kept_writes > 0,
            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {} 个、写 {} 个",
            stream.refused_requests,
            stream.kept_writes
        );
        assert!(
            stream.whole_disk_reads_back_as_written_through,
            "整盘读回与写穿的那块盘相同"
        );
        assert!(
            stream.disk_below_is_unchanged,
            "一串写之后底下那块盘一个字节都没变"
        );
    }

    /// 4096 字节的物理块。
    const FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES: u32 = 4096;

    /// 物理块 4096 字节的盘上，落在 512 字节边界上、不在 4096 边界上的请求（偏移或长度）照两个后端那一份判据拒成 `Unaligned`、
    /// 带 4096 这个物理块，一个块都不记；同一块盘上按 4096 对齐的写读得回来。判据里物理块写死 512 时这几个请求都放行。
    #[test]
    fn requests_on_512_byte_boundaries_off_the_4096_byte_physical_blocks_are_refused_before_anything_is_kept(
    ) {
        let block = PhysicalBlockSizeInBytes(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES);
        let block_bytes = u64::from(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES);
        let disk = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(block, BLOCKS_OF_THE_DISK, 31);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let one_block = vec![0x3C_u8; usize::try_from(block_bytes).expect("一块装得进 usize")];
        let one_sector = vec![0x3C_u8; 512];
        let offset_on_a_sector_inside_block_one = DeviceOffsetInBytes(block_bytes + 512);
        let refusals = [
            (
                "偏移在 512 边界上、不在 4096 边界上的写",
                kept.write_at(
                    offset_on_a_sector_inside_block_one,
                    &one_block,
                    WriteDurability::Plain,
                ),
                offset_on_a_sector_inside_block_one,
                block_bytes,
            ),
            (
                "长度 512 的写",
                kept.write_at(
                    DeviceOffsetInBytes(2 * block_bytes),
                    &one_sector,
                    WriteDurability::Plain,
                ),
                DeviceOffsetInBytes(2 * block_bytes),
                512,
            ),
            (
                "长度 512 的写零",
                kept.write_zeroes_at(DeviceOffsetInBytes(3 * block_bytes), 512),
                DeviceOffsetInBytes(3 * block_bytes),
                512,
            ),
            (
                "偏移在 512 边界上、不在 4096 边界上的读",
                kept.read_at(offset_on_a_sector_inside_block_one, &mut vec![0_u8; 512]),
                offset_on_a_sector_inside_block_one,
                512,
            ),
        ];
        for (request, outcome, offset, length) in refusals {
            assert!(
                matches!(
                    outcome,
                    Err(BlockDeviceError::Unaligned {
                        offset: refused_offset,
                        length: refused_length,
                        physical_block_size: FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES,
                    }) if refused_offset == offset && refused_length == length
                ),
                "{request}：拒成 Unaligned，带偏移 {}、长度 {length} 与物理块 4096，而交回 {outcome:?}",
                offset.0
            );
        }
        kept.write_at(
            DeviceOffsetInBytes(5 * block_bytes),
            &one_block,
            WriteDurability::Plain,
        )
        .expect("按 4096 对齐的一块写做得成（阳性对照）");
        let mut expected = disk.bytes.clone();
        let start_of_block_five = usize::try_from(5 * block_bytes).expect("偏移装得进 usize");
        expected[start_of_block_five..start_of_block_five + one_block.len()]
            .copy_from_slice(&one_block);
        let mut whole_disk = vec![0xAA_u8; disk.bytes.len()];
        kept.read_at(DeviceOffsetInBytes(0), &mut whole_disk)
            .expect("整盘一次读：对齐、没越过盘尾");
        assert!(
            whole_disk == expected,
            "被拒的请求一个块都没记：整盘读回只有按 4096 对齐写的那一块是写进去的"
        );
    }

    /// 「被宽护卡住」那一份报拷下的这次改动（[`UserChangeKeptByTheRefusal`]），要出路时读回成交给预演的改动，与被拒那一次交进来的改动
    /// 逐项相同：种类、内容全长、写入时间、inode、个数（正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md` K3-3：
    /// 记错这几项的 8 种改坏原来照绿，其中读回时把覆盖写当顺序写那一种只在读回这一步，报的 `{:?}` 看不出来）。
    #[test]
    fn each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in() {
        let overwrite_content: Vec<u8> = (0..2999_u32)
            .map(|position| u8::try_from(position % 241).expect("小于 256"))
            .collect();
        let sequential_write_content: Vec<u8> = (0..5003_u32)
            .map(|position| u8::try_from((position * 7) % 239).expect("小于 256"))
            .collect();
        let changes_handed_in = [
            UserChange::Overwrite(FirstFile {
                content: &overwrite_content,
                write_time_seconds: 1_000_003,
            }),
            UserChange::SequentialWrite(FirstFile {
                content: &sequential_write_content,
                write_time_seconds: 1_000_007,
            }),
            UserChange::NewInodes {
                count: 37,
                write_time_seconds: 1_000_011,
            },
            UserChange::Unlink {
                inode: InodeNumber(4_099),
                write_time_seconds: 1_000_013,
            },
            UserChange::CreateSnapshot,
            UserChange::DestroySnapshot {
                snapshot: TreeIdentifier(65_551),
            },
        ];
        for change_handed_in in changes_handed_in {
            let kept = UserChangeKeptByTheRefusal::kept_from(change_handed_in);
            assert_eq!(
                format!("{:?}", kept.as_user_change()),
                format!("{change_handed_in:?}"),
                "拷下的改动读回成被拒那一次交进来的改动（种类、内容全长、写入时间、inode、个数逐项相同）"
            );
        }
    }
}
