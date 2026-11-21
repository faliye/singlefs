//! 挂着的会话：一次可写挂载之后接着发布的那一层（D16（发布语义） 已定项 1「准入」那一行的发布那一处，
//! C283（准入失败时不先推发布就报 ENOSPC））。它持着这次挂载的分配器、现行那一版、实例代号与影子账那一臂，用户的改动经它发布：
//! 发布路径报空间准入不够（`PublishError::SpaceAdmissionRefused`）、或准入放行而落点取不到（`PublishError::PlacementRefused`）时，
//! 先推一串抬 F 的空发布（`mount::push_one_floor_raise_within_the_admission_budget`：F 抬到准入上限；上限卡在挂载时读坏或读回全零的槽上时
//! 推一次不抬 F 的空发布，D16（发布语义） 已定项 1「抬 F 的上限」那一行；一次准入最多 `mount::PUBLISHES_PER_ADMISSION_AT_MOST` 次发布，
//! 两种推的都算、再算上这次用户发布），再重判；做满仍不够、或抬不动了，才报空间不够。推抬 F 放得出的全放出来也放不下时不推，当场报空间不够
//! （`UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）；推串之前全空聚簇段已少于 `N₀`、被拒的又是前台写入时同样不推、同一个成员。
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
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};

use crate::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes,
    InodeNumber, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use crate::admission::{
    admit_the_empty_cluster_segments_floor_on_every_device, AdmissionRefusedOnSomeDevices,
    AvailableBytesOnOneDevice, BytesOnOneDevice, DeviceShortOfDemand,
    EmptyClusterSegmentsBelowTheFloorOnSomeDevices, JudgementThatFoundTheDeviceShort,
    SpaceAdmission,
};
use crate::allocator::{
    AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PoolAllocator, SlotPairCount,
};
use crate::background_compute::{
    decisions_of_one_segment, AlarmOfABackgroundTaskStoppedAfterAComputeError,
    AllocatedBytesAndCapacityOfADevice, BackgroundBatchUnderWay, BackgroundComputeError,
    BackgroundComputeOfOneBatch, BackgroundComputeOfThisMount, BackgroundComputeSettings,
    BackgroundComputeSettingsRefused, BackgroundGcCpuCompensationThreshold,
    BackgroundGpuComputeHandle, BackgroundTask, GcCpuCompensationReading, GpuComputedSegment,
    GpuEntryDecision, GpuWorkOfTheBatch, InTheMiddleOfEachBackgroundBatch,
    WhetherTheTaskRunsTheNextBatch,
};
use crate::block_device::{
    check_aligned_and_in_range, BlockDevice, BlockDeviceError, DiscardOutcome,
    PhysicalBlockSizeInBytes, WriteDurability,
};
use crate::compaction_plan::{
    choose_region, choose_region_over_the_live_slots, cluster_segment_entry_keys_of_the_unit_area,
    compaction_watermark_verdict_of_the_allocator, empty_cluster_segments_by_the_records,
    foreground_writes_are_below_the_empty_cluster_segments_floor, holes_admit_the_issue,
    holes_outside_the_region_in_segments_holding_user_data,
    plan_batch_skipping_the_units_this_publish_rewrites_anyway, publish_budget_of_a_compaction_run,
    publishes_left_of_the_budget_given_by_the_last_user_change, region_holds_a_quarantined_slot,
    region_is_empty_by_the_records, region_scans_of_the_segments_without_an_unreleased_record,
    the_completed_intent_yielded, AnswerOfTheCallerBeforeTheNextBatch, BackgroundTripUnderWay,
    BatchOutOfTheLock, CallerMonotonicClockReading, CallerOfTheBackgroundCompaction,
    CompactionBatchSize, CompactionBatchUnits, CompactionOfThisMount, CompactionPacing,
    CompactionRunForcing, CompactionRunStop, CompactionStartJudgement, CompactionSwitches,
    CompactionWriteBandwidthLimit, CompactionYieldFloor, ForegroundEntryWaitingForTheSessionLock,
    ForegroundYieldUnderTheFloor, LiveSlotsOfAClusterSegment, PlannedBatch, PlannedRelocation,
    RegionChoice, RegionChoiceCriterion, SegmentsRefusedByTheRegionScan, TimerFallbackCheck,
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
    physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration,
    push_one_floor_raise_within_the_admission_budget, rollback_states_of_the_ring,
    DeviceBehindTheCurrentVersion, FloorRaisePushedWithinTheAdmissionBudget, FloorRaiseStop,
    MountError, MountOutput, Mounted, OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn,
    ParametersAndDeviceTableOfTheMount,
    PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration, RaisedFloor, ShadowLedger,
};
use crate::mounted_read::{
    open_pool_for_read, FileReadFailure, FileReadOutput, OpenFileFailure, OpenPoolForReadFailure,
};
use crate::pointer::LocationEntry;
use crate::recovery::{
    instance_table_chain_of_root, PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
    PoolReaderOfOneReadPass, ReadPassOverThePool, RecoveryFailure,
};
use crate::region_scan::{
    scan_region_on_the_gpu, scan_region_over_the_identity_index, unit_reads_of_the_region_scan,
    AllocationInRegion, CentralMappingEntry, CentralMappingIdentityIndex, RegionScan,
    RegionScanWithTheRecordsUncertainThisRound, UnitBytesReadBeforeTheRegionScan,
    UnitReadsOfARegionScan,
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
    publish_the_next_step_of_the_deletion_queue,
    publish_the_relocations_and_the_repair_of_the_exempt_units, publish_unlink_intent,
    publish_version, relocations_of_old_data_units_kept_for_the_snapshots,
    relocations_of_the_units_read_with_one_copy_failing,
    repair_of_the_exempt_units_at_these_locations, resend_the_frozen_publish,
    units_at_these_locations, walk_the_deletion_queue_until_it_is_empty,
    CentralMappingOfAVersionReadLeafByLeaf, CompactionIntentStep, CompactionStepRefused,
    DataChangesPublishedTogether, DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile,
    KindOfTheFrozenPublish, MappingLookup, NewInodesOfOneChange, PartialWriteBytes,
    PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter, PublishError, ReadFailedOnTheDisk,
    ReadFailingSelfVerification, RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck,
    RelocationBatchRefused, RepairOfTheExemptUnits, TransactionOutput, TransactionUnit,
    WholeWriteOfOneInode,
};
use crate::unit::{data_unit_payload_capacity, unit_filesystem_identifier, UNIT_CLASS_DATA};
use crate::unlink::DeletionSwitches;
use singlefs_format::ROOT_RING_REGIONS;

mod background_compaction_out_of_the_lock;
mod compaction_batch;
mod compaction_entry_points;
mod compaction_publish;
mod compaction_run;
mod device_keeping_writes_in_memory;
mod device_read_once_in_this_mount;
mod discard_of_reusable_segments;
mod floor_raise_for_space;
mod pending_changes;
mod relocation_of_units_read_with_one_copy_failing;
mod root_ring_push;
mod user_change_kept_by_the_refusal;

pub use background_compaction_out_of_the_lock::*;
use compaction_batch::*;
pub use compaction_entry_points::*;
pub use compaction_publish::*;
pub(crate) use compaction_run::*;
pub(crate) use device_keeping_writes_in_memory::*;
pub(crate) use device_read_once_in_this_mount::*;
pub use discard_of_reusable_segments::*;
pub use floor_raise_for_space::*;
pub use pending_changes::*;
pub use relocation_of_units_read_with_one_copy_failing::*;
pub use root_ring_push::*;
use user_change_kept_by_the_refusal::*;

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
    /// 这次准入里推的那几串抬 F 的空发布，按先后；一次都没推的是空的。另发的那一批搬迁被空间拒时推的不在这里，在
    /// [`Self::relocation_after_the_change_of_the_units_read_with_one_copy_failing`] 里（[`FloorRaisesPushedForTheRelocation`]）。
    pub floor_raises: Vec<RaisedFloor>,
    /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）：`SpaceAdmissionRefused` 或 `PlacementRefused`。
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    /// 这次调用里推的抬 F 之外发成的每一版，按先后（[`VersionsPublishedInThisChange`]）。整理一批都不在这里：整理挪出了用户改动的调用线程
    /// （用户 2026-10-26 定），改动发成之后起跑判据为真只记成「该跑了」（`compaction_plan::CompactionStartJudgement`），
    /// 由后台入口 [`MountedSession::run_compaction_in_the_background`] 去跑。另发的那一批搬迁（不是整理，是修只坏一份的旧单元，
    /// [`Self::relocation_after_the_change_of_the_units_read_with_one_copy_failing`]）发成时排在末尾。
    pub versions_published: VersionsPublishedInThisChange,
    /// 这次调用的发布都返回之后发的 discard（判决 G5，[`discard_the_segments_reusable_as_a_whole`]）。
    pub discards_after_the_change: DiscardsAfterThePublishes,
    /// 这次改动发成之后另发的那一批做成了什么（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]）：管这次改动里读—改—写读回时只坏一份、改动之后仍被活快照钉着的旧单元，
    /// 可写挂载读盘建上一版时读到只坏一份的单元，与可写挂载期间别的读读到只坏一份的单元。
    pub relocation_after_the_change_of_the_units_read_with_one_copy_failing:
        RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing,
}

/// 一次用户改动里各次发布的写入口上读—改—写读回旧数据单元时只坏一份的那几次（`transaction::PoolWriter` 的两份账），按发布的先后并起来：
/// 次数并进这次挂载的累计（[`MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]），映射 key 交给另发的那一批搬迁。
/// 被拒的那几次发布读回的也算：坏的那一份是真读到的。这个数按「一次用户改动」去重：被空间拒、推一串抬 F 之后重发同一个改动，每一次发布都重读同几个旧单元
/// （同一把映射 key），算一次（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 6 条「每次用户改动只计一次」）；跨改动相加，
/// 同一份坏副本下一次改动再读回再算。去重的范围与挂载那个数（一次挂载、同一组位置条目）、只读挂载的 `read_at` 累计（不去重）各是各的，三个数分开报。
struct OldDataUnitReadsBackWithOneCopyFailingInThisChange {
    reads: u64,
    mapping_keys: Vec<Vec<u8>>,
}

impl OldDataUnitReadsBackWithOneCopyFailingInThisChange {
    fn nothing_read_yet() -> Self {
        Self {
            reads: 0,
            mapping_keys: Vec::new(),
        }
    }

    /// 并进一次发布的写入口上那两份账：这次改动里前面的发布已经读回过的旧单元（映射 key 已在账上）不再加次数、不再记 key。
    ///
    /// # Panics
    /// 次数加起来越过 u64：每一次都是一次真读盘，一次改动里读不到 2⁶⁴ 次。写入口上的次数与 key 不是一次一把：两份账在
    /// `transaction::read_back_the_old_payload_counting_a_failing_copy` 里同一处各记一次。
    fn add_the_reads_of_the_writer<Device: BlockDevice>(
        &mut self,
        writer: &PoolWriter<'_, Device>,
    ) {
        let mapping_keys_of_the_writer =
            writer.mapping_keys_of_old_data_units_read_back_with_one_copy_failing();
        assert_eq!(
            u64::try_from(mapping_keys_of_the_writer.len())
                .expect("一次发布读回的单元数装得进 u64"),
            writer.old_data_unit_reads_back_with_one_copy_failing(),
            "写入口上只坏一份的读回次数与映射 key 一次一把：两份账同一处各记一次"
        );
        let reads_of_units_read_back_before_in_this_change = self
            .keep_the_mapping_keys_not_read_back_before_in_this_change(mapping_keys_of_the_writer);
        self.reads = self
            .reads
            .checked_add(writer.old_data_unit_reads_back_with_one_copy_failing())
            .and_then(|reads| reads.checked_sub(reads_of_units_read_back_before_in_this_change))
            .expect("一次改动里只坏一份的读回次数装得进 u64");
    }

    /// 一次发布读回时只坏一份的那几把映射 key 里，这次改动前面的发布没读回过的记上（同一把只记一次）；交回读回过的有几把（这几次不再加次数）。
    fn keep_the_mapping_keys_not_read_back_before_in_this_change(
        &mut self,
        mapping_keys_of_one_publish: Vec<Vec<u8>>,
    ) -> u64 {
        let mut reads_of_units_read_back_before_in_this_change: u64 = 0;
        // 迭代上界是这一次发布读回时只坏一份的单元数；跨轮携带的是已记的 key 与读回过的次数。
        for mapping_key in mapping_keys_of_one_publish {
            if self.mapping_keys.contains(&mapping_key) {
                reads_of_units_read_back_before_in_this_change += 1;
            } else {
                self.mapping_keys.push(mapping_key);
            }
        }
        reads_of_units_read_back_before_in_this_change
    }
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
    /// 分配器上冻结着一次没重发的发布时，一段都不发、待 discard 表原样留着（C928（冻结着时整理照发 discard））：冻结那次的 txg；没冻结是 `None`。
    /// 冻结时分配器退回了发布之前那一份，而冻结那次的单元、记录、根槽可能已经落盘：退回的那一份里整段可再分配的段可能正装着它的单元，
    /// 重发之前断电，恢复择到它的根，那一段收过 discard 就读不回它的单元。
    pub txg_of_the_frozen_publish_withholding_every_discard: Option<CheckpointTxg>,
}

/// 一次用户改动里推的抬 F 之外发成的每一版，按先后：先走空的删除步（D8（核心索引结构） 已定项 16 第 6 条）、这次改动本身
/// （unlink 是意图那次）、unlink 之后走空的删除步；与推的抬 F 的空发布按 txg 交错。最后一个是会话的 `current`
/// （之后没再推抬 F 时）。交给调用方逐次比（模型对拍）：会话的 `current` 只是最后那一版。
/// 每一版按引用计数共享：最后一个与会话的 `current` 指着同一份（[`PoolVersion`] 那一段），放进这里不拷一整版。
pub type VersionsPublishedInThisChange = Vec<Arc<TransactionOutput>>;

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
    /// 盘表的身份与这次挂载核过的那一份相同，这几块盘（按交进来的次序）这一刻探测到的 `physical_block_size` 却不是会话持着的那份参数里
    /// 记的 mkfs 时的值（系统配置偏移 317，根槽宽）：换上了一块物理块大小不同的盘（D12（目标介质） 已定项 4 比对清单第一行，C1040；
    /// 与可写挂载、挂着之后收盘表的入口同一判，`mount::physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration`）。
    /// 不比时这次改动照常发成、写进那块盘：记录 4096 的池换上探测 512 的盘，每个写都对得齐；记录 512 的池换上探测 4096 的盘，
    /// 读 512 宽的根槽报不对齐，写照样落到那块盘上。只调探测，一个盘都没读、一个写都没发：盘上逐字节不变、会话不动。
    /// 调用方要做的是换回物理块大小与池记的相同的盘。
    PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration {
        disagreements: Vec<PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration>,
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
    /// 空间不够（报给用户就是 ENOSPC），抬 F 之前先算判出这一次不推，两形都在 [`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`] 里：
    /// 槽对与式子那两判是推抬 F 放得出的全放出来也放不下，带上界与需求；合取 7 那一形是这次调用还一串没推、被拒的是合取 7 判的前台写入类、
    /// 写之前那一刻全空聚簇段已少于 `N₀`，带 `N₀` 与少于它的盘。这一次判之前已经推成的串、发成的版在里面，头一次被拒就判到这里时它们是空的，盘上逐字节不变。装箱同 `NoSpaceAfterRaisingTheFloor`。
    NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(
        Box<NoSpaceEvenWithEverythingAFloorRaiseCouldRelease>,
    ),
    /// 发布被拒是空间不够、为它推一串抬 F 时抬 F 自己报了不是空间不够的错（块设备错、读不出、预演被别的理由拒……）：不是「推满仍不够」，
    /// 不报空间不够（代码审阅第 23 条），错原样在 [`FloorRaiseFailedWhilePushingForSpace`] 里。抬 F 那一串自己也取不到落点、被空间准入拒的
    /// 是推满仍不够，走 `NoSpaceAfterRaisingTheFloor`。装箱同 `NoSpaceAfterRaisingTheFloor`。
    FloorRaiseFailedWhilePushingForSpace(Box<FloorRaiseFailedWhilePushingForSpace>),
}

/// 在一个新开的写入口上把现行那一版的 `deleted_inodes` 走空（D8（核心索引结构） 已定项 16 第 6 条：会话里每一次用户改动先走空），
/// 每发成一步 `current` 就换成那一版——中途失败时 `current` 停在最后一次落盘的那一版上。批大小与墓碑容器容量取分配器上装的开关。
fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    current: &mut Arc<TransactionOutput>,
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

/// 这一次调用把一次发布冻结了（进门时没冻结着）：记上那一次是哪一种，重发成了照它记账（[`MountedSession::resend_the_frozen_publish`]）。发布路径不知道发起它的是谁，种类只有这一层说得出。
fn note_the_kind_of_a_publish_frozen_by_this_call(
    allocator: &mut PoolAllocator,
    was_frozen_before_this_call: bool,
    kind: KindOfTheFrozenPublish,
) {
    if was_frozen_before_this_call {
        return;
    }
    if let Some(frozen) = allocator.take_frozen_publish() {
        allocator.freeze_publish(frozen.of_the_kind(kind));
    }
}

/// 同 [`publish_the_queue_until_it_is_empty`]，只走现行那一版删除队列的下一步（[`publish_the_next_step_of_the_deletion_queue`] 选）：
/// 放锁的后台入口在步与步之间放锁，队列多长都不整段持锁。队列空时一步都不发。
fn publish_at_most_one_step_of_the_queue<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    current: &mut Arc<TransactionOutput>,
    instance: InstanceGeneration,
) -> DeletionQueueWalk {
    let switches: DeletionSwitches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    let Some(next_step) = publish_the_next_step_of_the_deletion_queue(
        &mut writer,
        allocator,
        current,
        switches.batch_size,
        instance,
    ) else {
        return DeletionQueueWalk {
            steps_published: Vec::new(),
            stopped_by: Ok(()),
        };
    };
    match next_step {
        Ok(next) => {
            *current = Arc::new(next);
            DeletionQueueWalk {
                steps_published: vec![Arc::clone(current)],
                stopped_by: Ok(()),
            }
        }
        Err(refusal) => DeletionQueueWalk {
            steps_published: Vec::new(),
            stopped_by: Err(refusal),
        },
    }
}

/// 这一版的删除队列（`deleted_inodes`、销毁中的快照、截断）空着。没有树的一版没有队列。
fn deletion_queue_of_the_version_is_empty(version: &PoolVersion) -> bool {
    match version {
        PoolVersion::WithFile(version_of_trees)
        | PoolVersion::WithTreesAndNoFile(version_of_trees) => {
            version_of_trees.queue_of_committed_intents_is_empty()
        }
        PoolVersion::WithoutFile(_) => true,
    }
}

/// 开关照水位判、任一块盘全空段低于底额：前台写入类发布已写不进（后台入口的破例、不再为前台 fsync 留着没完的一趟都判它）。
fn foreground_cannot_write_under_the_switches(allocator: &PoolAllocator) -> bool {
    matches!(
        allocator.compaction().switches.run_forcing,
        CompactionRunForcing::FollowTheWatermarks
    ) && foreground_writes_are_below_the_empty_cluster_segments_floor(allocator)
}

/// 经会话读文件没读成（[`MountedSession::read_the_file`]）。盘表那四样在读文件之前，盘上逐字节不变（读本身从不写盘）；后三样是读者报的。
#[derive(Debug)]
pub enum FileReadInTheSessionRefused {
    /// 同 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 同 [`UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`]。
    DevicesWithoutASelfVerifiedSystemConfiguration { devices: Vec<DeviceIdentity> },
    /// 同 [`UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable`]。
    OwnDeviceNumbersDifferFromTheDeviceTable {
        disagreements: Vec<OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn>,
    },
    /// 同 [`UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`]。
    DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
        devices: Vec<DeviceBehindTheCurrentVersion>,
    },
    /// 现行那一版树表 0 条：还没有文件版本。
    NoFileVersionToRead,
    /// 打开挂载态没做成。装箱：读者的错远大过别的成员（clippy `result_large_err`）。
    OpenPool(Box<OpenPoolForReadFailure>),
    /// 打开文件没做成。装箱同上。
    OpenFile(Box<OpenFileFailure>),
    /// 读字节没做成。
    Read(FileReadFailure),
}

/// [`MountedSession::read_the_file`] 读文件之前核盘表（同 [`MountedSession::publish_user_change`] 进循环之前那几样）：只读每块盘的系统配置槽与落后那几块上现行那一版的落点；
/// 落后支读到只坏一份的那几组经 `allocator` 的计数层进别的读那一份账（[`crate::allocator::PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`]）。
///
/// # Errors
/// [`FileReadInTheSessionRefused`] 的盘表四样与 `NoFileVersionToRead`。
fn refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount<Device: BlockDevice>(
    parameters_and_device_table: &ParametersAndDeviceTableOfTheMount,
    allocator: &PoolAllocator,
    current: &PoolVersion,
    devices: &[(DeviceIdentity, Device)],
) -> Result<(), FileReadInTheSessionRefused> {
    if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
        return Err(
            FileReadInTheSessionRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            },
        );
    }
    let parameters = parameters_and_device_table.parameters_on_disk();
    let Some(current_version_of_trees) = current.version_of_trees() else {
        return Err(FileReadInTheSessionRefused::NoFileVersionToRead);
    };
    let devices_without_a_system_configuration =
        devices_without_a_self_verified_system_configuration(devices, parameters);
    if !devices_without_a_system_configuration.is_empty() {
        return Err(
            FileReadInTheSessionRefused::DevicesWithoutASelfVerifiedSystemConfiguration {
                devices: devices_without_a_system_configuration,
            },
        );
    }
    let disagreements =
        own_device_numbers_on_disk_differing_from_the_identities_handed_in(devices, parameters);
    if !disagreements.is_empty() {
        return Err(
            FileReadInTheSessionRefused::OwnDeviceNumbersDifferFromTheDeviceTable { disagreements },
        );
    }
    // 落后的盘上窗口之外坏了的那几份经计数层记进别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」），
    // 下一次用户改动发成之后另发的那一批换下它们；核不过不读文件，账照留。
    let devices_of_the_table_behind = allocator
        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(devices, |reader| {
            devices_behind_the_current_version(reader, parameters, current_version_of_trees)
        });
    if !devices_of_the_table_behind.is_empty() {
        return Err(
            FileReadInTheSessionRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                devices: devices_of_the_table_behind,
            },
        );
    }
    Ok(())
}

/// 沿 `root` 打开挂载态、打开 `inode` 那个文件、读那一段（[`MountedSession::read_the_file`] 经数只坏一份的那一层交 `reader`）。
///
/// # Errors
/// [`FileReadInTheSessionRefused`] 的读者那三种。
fn read_the_file_of_the_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
    inode: InodeNumber,
    offset: FileOffsetInBytes,
    length_in_bytes: u64,
) -> Result<FileReadOutput, FileReadInTheSessionRefused> {
    let mounted = open_pool_for_read(reader, root)
        .map_err(|failure| FileReadInTheSessionRefused::OpenPool(Box::new(failure)))?;
    let file = mounted
        .open_file(reader, inode)
        .map_err(|failure| FileReadInTheSessionRefused::OpenFile(Box::new(failure)))?;
    file.read_at(reader, offset, length_in_bytes)
        .map_err(FileReadInTheSessionRefused::Read)
}

/// 在一个新开的写入口上发一次这个改动，接在 `previous` 后面。unlink 这里只发意图那一次，全部批由调用方接着走空。
/// 发完（发成或被拒）把这个写入口上读—改—写读回时只坏一份的那两份账并进 `old_data_unit_reads_back`：会话里只有这一处的发布读回旧数据单元
/// （部分写、截断与变长，`transaction::publish_partial_write`、`transaction::publish_size_change`），别的几处写入口上这两份账恒空。
fn publish_the_change_once<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    change: UserChange<'_>,
    instance: InstanceGeneration,
    old_data_unit_reads_back: &mut OldDataUnitReadsBackWithOneCopyFailingInThisChange,
) -> Result<TransactionOutput, PublishError> {
    let switches: DeletionSwitches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    let published =
        publish_the_change_on_the_writer(&mut writer, allocator, previous, change, instance);
    old_data_unit_reads_back.add_the_reads_of_the_writer(&writer);
    published
}

/// [`publish_the_change_once`] 在它新开的写入口上按改动分发的那一段。
#[allow(
    clippy::needless_borrow,
    reason = "各臂照写 `&mut writer`（借一层再自动解一层）：`crates/mutations.tsv` 里有一行按这一段的原文逐字锚着"
)]
fn publish_the_change_on_the_writer<Device: BlockDevice>(
    mut writer: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    change: UserChange<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
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

    /// 这次可写挂载里读—改—写读回旧数据单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：每次用户改动里各次发布的
    /// 写入口上的数之和（`transaction::PoolWriter::old_data_unit_reads_back_with_one_copy_failing`，改动被拒时那几次读也算）。只在内存里，
    /// 不落盘；新挂载从 0 起。只读挂载那一份在挂载态上（`mounted_read::ReadPathObservation::data_unit_reads_with_one_copy_failing`）。
    #[must_use]
    pub fn old_data_unit_reads_back_with_one_copy_failing_in_this_mount(&self) -> u64 {
        self.compaction
            .old_data_unit_reads_back_with_one_copy_failing()
    }

    /// 这次可写挂载读盘建上一版时（`mount::mount_writable` 那一串里建所选或退到的那一版）按位置条目读一个单元只坏一份的次数
    /// （D4（校验和位置） 已定项 4「读到只坏一份时计数」），各类单元都算。与 [`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]
    /// 按读的地方分开报：那一个是会话里读—改—写读回旧数据单元的。只在内存里，不落盘；没经过可写挂载的会话是 0。
    #[must_use]
    pub fn unit_reads_with_one_copy_failing_while_mounting(&self) -> u64 {
        self.compaction
            .unit_reads_with_one_copy_failing_while_mounting()
    }

    /// 这次可写挂载里别的读读到只坏一份的次数（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬，包括……可写挂载期间别的读」）：[`Self::read_the_file`]、
    /// [`crate::allocator::PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`] 文档串里那几类（建上一版读到的那几组也在等下一次用户改动，再读不进这个数）。一组位置条目等着下一次用户改动的这一程里只算一次，处置过之后再读到再算。
    /// 与 [`Self::unit_reads_with_one_copy_failing_while_mounting`]、[`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`] 按读的地方分开报。
    /// 只在内存里，不落盘；新挂载从 0 起。
    #[must_use]
    pub fn other_unit_reads_with_one_copy_failing_in_this_mount(&self) -> u64 {
        self.allocator.other_unit_reads_with_one_copy_failing()
    }

    /// 可写挂载期间的用户读文件：读现行那一版（`current`）里 `inode` 那个文件 `[offset, offset + length_in_bytes)` 的字节（读者同只读挂载，
    /// `mounted_read::open_pool_for_read`、`open_file`、`read_at`，每次调用从根读起），经数只坏一份的那一层读。读到只坏一份的单元进
    /// [`Self::other_unit_reads_with_one_copy_failing_in_this_mount`]，下一次用户改动发成之后另发的那一批把它换到新槽、两份新写
    /// （[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]，D4（校验和位置） 已定项 4）。读本身不写盘。报错的那一次报错之前读到的照样进账。
    /// 盘表照 [`Self::publish_user_change`] 核三样（D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」），核不过不读文件。
    /// `open_pool_for_read` 那条不挂在会话上的路是只读语义，只计数、不搬。
    ///
    /// # Errors
    /// [`FileReadInTheSessionRefused`]。
    pub fn read_the_file<Device: BlockDevice>(
        &self,
        devices: &[(DeviceIdentity, Device)],
        inode: InodeNumber,
        offset: FileOffsetInBytes,
        length_in_bytes: u64,
    ) -> Result<FileReadOutput, FileReadInTheSessionRefused> {
        refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount(
            &self.parameters_and_device_table,
            &self.allocator,
            &self.current,
            devices,
        )?;
        let devices_counting_unit_reads_with_one_copy_failing =
            PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(devices);
        let read = read_the_file_of_the_root(
            &devices_counting_unit_reads_with_one_copy_failing,
            self.current.root(),
            inode,
            offset,
            length_in_bytes,
        );
        self.allocator
            .count_the_other_unit_reads_with_one_copy_failing(
                devices_counting_unit_reads_with_one_copy_failing
                    .locations_of_the_units_read_with_one_copy_failing(),
            );
        read
    }

    /// 这次可写挂载读盘建上一版时读到只坏一份、还没交给另发那一批的那几个单元的映射 key（挂载交回之前按挂载之后那一版认的，
    /// `transaction::units_at_these_locations`）：下一次用户改动发成之后另发的那一批取走它们、照快照下另发那批同一条路径搬
    /// （[`Self::publish_user_change`]，用户 2026-11-05 定「可写挂载时搬」），取走之后是空的。
    #[must_use]
    pub fn mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting_not_yet_relocated(
        &self,
    ) -> &[Vec<u8>] {
        self.compaction
            .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting()
    }

    /// 发布一次用户改动：被拒是空间不够的那两种（[`refusal_is_short_of_space`]）就推一串抬 F 的空发布、再重判，
    /// 直到发成或不再推（D16（发布语义） 已定项 1「准入」那一行）。发成时 `current` 换成新的一版；推了的那几串已落盘，
    /// 没发成时 `current` 停在最后一次落盘的空发布上。写入口只按会话持着的那份参数建（`parameters_and_device_table`），
    /// 交进来的盘表先与它逐项比身份，再逐盘比这一刻探测到的 `physical_block_size` 与参数里记的值（D12（目标介质） 已定项 4 比对清单第一行，
    /// 与可写挂载同一判，`mount::physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration`；只调探测、不读盘），
    /// 再读每块盘的两个系统配置槽核它「可见」（Z3-A 乙），再核本盘设备号与落后支
    /// （D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」：「每次发布」按「每次用户改动」读）。
    /// 探测值与三样在进循环之前核一次：一次调用里盘表借着、换不了盘；循环里推的每一串抬 F 在它自己的第一个写之前再核一遍探测值与三样
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
    /// 抬 F 之前先算判出不推时交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`（`push_one_floor_raise_for_a_refusal_for_space`）：
    /// 槽对与式子那两判是推抬 F 放得出的全放出来也放不下；合取 7 那一形是这次调用还一串没推、被拒的是合取 7 判的前台写入类、写之前那一刻全空聚簇段已少于 `N₀`。
    /// 删除那几次发布的两个只供测试的开关取分配器上装的（可写挂载的开关链装上去的，`unlink::DeletionSwitches`）。
    /// 每发成一次（走空的每一步、这次改动）`current` 都按 `PoolVersion::of_a_version_of_trees` 重新归成员；现行那一版是有树而文件集合为空的
    /// 一版时只放建 inode（D8（核心索引结构） 已定项 16 第 5 条），别的改动拒成 `NoFileVersionToChange`。
    /// 发成之后查一次整理的起跑判据，为真只记成「该跑了」（`compaction_plan::CompactionStartJudgement::DueAfterAUserChange`），
    /// 返回之前一批整理都不发（整理挪出了调用线程，用户 2026-10-26 定；后台入口 [`Self::run_compaction_in_the_background`]）。
    /// 这次改动读—改—写读回旧数据单元时只坏一份的次数并进这次挂载的累计（[`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]），
    /// 发成之后其中仍被活快照钉着的那几个、挂载记下的、别的读记下的另发一批换落点（[`RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`]），都在 discard 之前。
    ///
    /// # Errors
    /// [`UserChangeRefused`] 的十种。
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
        let physical_block_sizes_differing =
            physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration(
                devices.as_slice(),
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
        if !physical_block_sizes_differing.is_empty() {
            return Err(
                UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration {
                    disagreements: physical_block_sizes_differing,
                },
            );
        }
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
        // 落后的盘上窗口之外坏了的那几份经这一层记进别的读那一份账（`mount::devices_behind_the_current_version`），这次改动发成之后另发的那一批换下它们。
        let devices_behind = allocator
            .reading_and_counting_the_other_unit_reads_with_one_copy_failing(
                devices.as_slice(),
                |reader| {
                    devices_behind_the_current_version(reader, parameters, current_version_of_trees)
                },
            );
        if !devices_behind.is_empty() {
            return Err(
                UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    devices: devices_behind,
                },
            );
        }
        let mut old_data_unit_reads_back =
            OldDataUnitReadsBackWithOneCopyFailingInThisChange::nothing_read_yet();
        let published = Self::publish_the_change_raising_the_floor_for_space(
            parameters,
            devices,
            allocator,
            current,
            *instance,
            *shadow_ledger,
            change,
            &mut old_data_unit_reads_back,
        );
        // 只坏一份的读回按一次挂载累计（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：改动被拒时那几次读也并进来。
        compaction
            .count_old_data_unit_reads_back_with_one_copy_failing(old_data_unit_reads_back.reads);
        let mut published = published?;
        // 读回时只坏一份、改动之后仍被活快照钉着的旧单元，照已有的搬迁路径另发一批（用户 2026-11-04 定「另发一批搬迁修好」，
        // 被空间拒「推抬 F 再试一遍」）；没有活快照钉着的那几个在改动那次发布里已经重写、旧落点照释放之前读盘核隔离，不进这一批。
        // 这一批没发成也不让改动算没发成：改动那一版已落盘。
        // 可写挂载读盘建上一版时读到只坏一份的那几个单元（挂载交回之前记下的映射 key）并进同一批（用户 2026-11-05 定「可写挂载时搬」）：
        // 挂载之后的头一次用户改动发成时取走，搬没搬成都只试这一次；这次改动已经把它整个重写了的不进计划。
        let mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting = compaction
            .take_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting();
        published.relocation_after_the_change_of_the_units_read_with_one_copy_failing =
            relocate_the_units_read_with_one_copy_failing_after_the_change(
                parameters,
                devices,
                allocator,
                current,
                *instance,
                *shadow_ledger,
                compaction,
                &old_data_unit_reads_back.mapping_keys,
                &mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting,
                &mut published.versions_published,
            );
        // 改动发成：整理「已经过去的发布数」从这里重新数（判决 G8 `B_gc`）。起跑判据照旧在这里查一次（判决 G8 丁），整理却挪出了用户改动的
        // 调用线程（用户 2026-10-26 定「挪出调用线程」；D24（后台重活能不能卸给 GPU） 已定项 2 第 2 条「不许进前台路径」）：这里只把「该跑了」
        // 记在会话上，一批都不发，由后台入口 [`Self::run_compaction_in_the_background`] 在调用方自己的线程上去跑。这一次调用拿着 `&mut self`
        // 从头到尾，为这次改动推的抬 F 与之后的再判之间插不进整理批（第三轮判决第二节 G8 的第三样保证）。
        compaction.note_a_user_change_published();
        // 改动发成：清掉会话上跨趟留着的上一条完成的意图（用户 2026-10-26 定「每次用户改动发成就清掉」，
        // `records/2026-10-19-里程碑四GPU垃圾回收开工.md`「调查员交回」那一行）。D26（后台整理与放置回收） 已定项 13 第 6 条
        // 「签下一条意图那一判跨趟留着」只留到下一次用户改动发成为止：池子一直被写满时全空段数只降不升，留到回滚或重挂会让整理一直停着；
        // 下一趟从空起照常签新意图，预算照旧是这一次改动给的那一份 `B_gc`（岔路单第 18 行甲）。被拒、没发成的改动在上面 `?` 交回，这一条照旧留着。
        compaction.last_completed_intent = None;
        let watermark_verdict = compaction_watermark_verdict_of_the_allocator(allocator);
        compaction.last_watermark_verdict = Some(watermark_verdict);
        if watermark_verdict.starts() {
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

    /// 后台计算的设置在这次挂载里的状态：生效的那一组与待生效的切换。
    #[must_use]
    pub fn background_compute(&self) -> &BackgroundComputeOfThisMount {
        self.parameters_and_device_table
            .background_compute_of_this_mount()
    }

    /// GC 停着时在 CPU 上补跑那一判此刻的读数（用户 2026-10-28 定案，同日弹窗定「任一块盘超过就补跑」）：池子此刻最满的那一块盘、它的已分配字节与容量
    /// （分配器增量维护的计数，[`AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator`]），与下一批读的那一组的补跑阈值。只读。
    #[must_use]
    pub fn gc_cpu_compensation_reading(&self) -> GcCpuCompensationReading {
        self.background_compute().gc_cpu_compensation_reading(
            AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator(
                &self.allocator,
            ),
        )
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

    /// 同 [`Self::switch_background_compute_from_the_next_batch`]，连同 GC 停着时在 CPU 上补跑的阈值一起换（挂载参数
    /// `background_gc_cpu_compensation_threshold`，用户 2026-10-28 定案；不给阈值的那一个切换沿用原来的阈值）。阈值超过 100% 也拒，停着的照旧停着。
    ///
    /// # Errors
    /// 校验不过：拒绝切换，生效的与待生效的设置都留着原样。
    pub fn switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
        &mut self,
        background_compute_settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
        gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
    ) -> Result<(), BackgroundComputeSettingsRefused> {
        self.parameters_and_device_table
            .background_compute_of_this_mount_mut()
            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                background_compute_settings,
                gpu_compute,
                gc_cpu_compensation_threshold,
            )
    }

    /// 把分配器上冻结着的那一次发布原样重发（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」，[`resend_the_frozen_publish`]）：一批整理或一次用户改动写盘报错之后
    /// 发布路径冻结，前台的写都被拒，靠这一个入口解。重发成了：`current` 换成那一版、分配器换成那次发布成立之后的那一份，活意图的 R 排他按那一版的 logged_ops 重装
    /// （冻结的那一批可能正是删意图的那一批），再照那一次的种类记账（[`KindOfTheFrozenPublish`]；D26（后台整理与放置回收） 已定项 13 第 6 条）：
    /// 用户改动那种与一次发成同一份账（重给预算、清上一条完成的意图、起跑判据为真记「该跑了」），整理那种扣预算。没有冻结着的发布交回 `false`，什么都不写。
    ///
    /// # Errors
    /// 重发又有一步报错（`PublishError::BlockDevice`）：照旧冻结着。
    pub fn resend_the_frozen_publish<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<bool, PublishError> {
        let Some(kind_of_the_frozen_publish) =
            self.allocator.frozen_publish().map(|frozen| frozen.kind())
        else {
            return Ok(false);
        };
        let parameters = self.parameters_and_device_table.parameters_on_disk();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let Some(version) = resend_the_frozen_publish(&mut writer, &mut self.allocator)? else {
            return Ok(false);
        };
        self.current = version;
        let live_intent_of_the_version =
            self.current
                .version_of_trees()
                .and_then(|version_of_trees| {
                    live_compaction_intent_of_the_entries(&version_of_trees.logged_operations)
                });
        self.allocator.compaction_mut().withdraw_the_live_intent();
        if let Some((_, intent)) = live_intent_of_the_version {
            self.allocator.compaction_mut().issue(intent);
        }
        match kind_of_the_frozen_publish {
            KindOfTheFrozenPublish::UserChange => {
                self.compaction.note_a_user_change_published();
                self.compaction.last_completed_intent = None;
                let watermark_verdict =
                    compaction_watermark_verdict_of_the_allocator(&self.allocator);
                self.compaction.last_watermark_verdict = Some(watermark_verdict);
                if watermark_verdict.starts() {
                    self.compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
                }
            }
            KindOfTheFrozenPublish::CompactionPublish => {
                self.compaction.publishes_since_the_last_user_change += 1;
                self.compaction.observations.root_ring_slots_used += 1;
                // 冻结的是后台批，照后台批记：这一次整理发布在重发成了这一刻发成，一批发成，让路的两个计数都清零
                // （D26（后台整理与放置回收） 已定项 13 第 6 条）。
                self.compaction.clear_the_yields_in_a_row();
            }
            KindOfTheFrozenPublish::NeitherOfTheCountedKinds => {}
        }
        Ok(true)
    }
}

/// 一趟整理推到这一刻的账：[`run_compaction_on_with_the_region_check_of_the_intent_deletion`] 跨批携带的那几样收成一份。
/// 全部自有、不借会话与盘表：放锁的三段入口把它连同一批的输入放进票里（[`BackgroundCompactionBatchTicket`]），回锁之后照它接着记账。
struct CompactionRunUnderWay {
    versions_published: Vec<TransactionOutput>,
    background_compute_of_each_batch: Vec<BackgroundComputeOfOneBatch>,
    deletion_steps_published: u64,
    publishes_of_the_run: u64,
    batches_published: u64,
    budget: u64,
    budget_rule: CompactionRunBudget,
    reach: CompactionRunReach,
    span_of_the_call: SpanOfTheBackgroundCall,
    batches_published_earlier_in_the_trip: u64,
    /// 这一趟开头读的开关：一趟停下时记进不发布的停的备忘。
    switches_of_the_run: CompactionSwitches,
    /// 推这一趟的这一次调用进门时冻没冻着一次发布：锁里推完的入口是开一趟那一刻，放锁的入口回锁那一次是回锁那一刻
    /// （[`CompactionRunUnderWay::note_whether_a_publish_is_frozen_as_the_relock_comes_in`]）。会话锁里只有这一次调用在发，
    /// 进门时没冻着而之后冻着的，冻的就是这一次调用发的那一次；进门时冻着的不是它发的（冻着时发布路径第一道就拒），种类不改。
    was_frozen_when_this_call_came_in: bool,
    batch_units: CompactionBatchUnits,
    is_the_rollback_gate_judged: bool,
    region_check: RegionCheckOfTheIntentDeletion,
    /// 放这一批时照的写带宽上限（[`judge_before_the_next_compaction_batch`] 放行时记下）：这一批发成之后照它记写了多少。
    write_bandwidth_limit_of_the_batch: CompactionWriteBandwidthLimit,
    /// 这一批是全空段低于底额、前台写不进时这一趟的第一次发布（破例：不判回退门、不让路）。
    is_the_batch_the_exception: bool,
}

impl CompactionRunUnderWay {
    /// 放锁的入口回锁那一次调用进门：重读此刻冻没冻着（取输入之后、回锁之前前台的发布可以冻结，也可以重发成了），
    /// 这一次调用里发布被拒时照它判冻的是不是这一次调用发的那一次（[`note_the_kind_of_a_publish_frozen_by_this_call`]）。
    fn note_whether_a_publish_is_frozen_as_the_relock_comes_in(
        &mut self,
        allocator: &PoolAllocator,
    ) {
        self.was_frozen_when_this_call_came_in = allocator.frozen_publish().is_some();
    }

    /// 一趟停下或出错：照 `outcome` 收尾。停下的那一判是在 `txg_the_outcome_was_judged_on` 那一版上判的（锁外那一段在票里那一版上判出的停，
    /// 记的是票里那一版；回锁之后在现行那一版上判出的，记的是现行那一版），不发布的停照它记进备忘。
    /// 交回装箱的那一趟：它是一趟各块（开一趟、每批之前那几判、一批之后记账）`Result` 的错那一侧，比接着推的账大得多。
    fn finish(
        self,
        compaction: &mut CompactionOfThisMount,
        txg_the_outcome_was_judged_on: CheckpointTxg,
        outcome: CompactionRunOutcome,
    ) -> Box<CompactionRun> {
        if let CompactionRunOutcome::Stopped(stop) = &outcome {
            compaction.observations.last_stop = Some(*stop);
            compaction.note_the_stop_of_a_run(
                *stop,
                txg_the_outcome_was_judged_on,
                self.switches_of_the_run,
            );
        }
        Box::new(CompactionRun {
            versions_published: self.versions_published,
            deletion_steps_published: self.deletion_steps_published,
            batches_published: self.batches_published,
            outcome,
            discards: DiscardsAfterThePublishes::default(),
            background_compute_of_each_batch: self.background_compute_of_each_batch,
            alarm_of_the_compaction_stopped_after_a_compute_error: None,
        })
    }
}

/// 一版里整理要读的几样：每个进映射单元的映射条目（按 key 在这一版的映射树里查，连同留存的），钉住的槽（豁免映射的三类：映射树各节点、
/// 树表、实例表链——根记录直接指着它们），码 1 数据单元落着的槽（签发前判「只装提交内生块的段不算非全空」用）。
struct VersionAsTheCompactionReadsIt {
    mapping_entries: Vec<CentralMappingEntry>,
    pinned_slots: BTreeSet<SlotNumber>,
    slots_holding_user_data: BTreeSet<SlotNumber>,
}

/// 整理推一次发布（后台任务的一批）交回的三样：这一步的结局、这一批的后台计算记账（请求的后端、实际走的路），与停在判据上时那一判读的是哪一版。
/// 整理停着、占用不超过补跑阈值而这一批没开成时没有记账（`None`），结局是 [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]。
struct CompactionPublishAdvanced {
    step: Result<CompactionStep, RelocationBatchRefused>,
    background_compute_of_the_batch: Option<BackgroundComputeOfOneBatch>,
    /// 停在判据上时，那一判读的那一版：锁外那一段在票里那一版上判出的停是票里那一版，回锁里在现行那一版上判出的是现行那一版；
    /// 一趟照它记不发布的停的备忘（[`CompactionRunUnderWay::finish`]）。发成、没发成时是那一刻的现行那一版。
    txg_the_step_was_judged_on: CheckpointTxg,
}

/// 一批整理在会话锁里取的输入（[`take_the_input_of_one_compaction_batch`]）：全部自有、不借会话与盘表，现行那一版只拷 `Arc` 指针。
/// 算的那一段只读它，不碰会话（D26（后台整理与放置回收） 已定项 13 第 5 条）。
struct CompactionBatchInput {
    batch_under_way: BackgroundBatchUnderWay,
    /// 只供测试的钩子在这一批里注入的错（取走了的）：有就不算，回锁照出错那一支办。
    compute_error_of_the_batch: Option<BackgroundComputeError>,
    version: Arc<TransactionOutput>,
    live_intent: Option<CompactionIntent>,
    quarantined_slots: BTreeSet<SlotNumber>,
    segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
    cluster_segments: BTreeSet<SlotNumber>,
    unit_area: UnitAreaOfOneDevice,
    forced_region: Option<CompactionRegion>,
    batch_units: CompactionBatchUnits,
    reach: CompactionRunReach,
    last_completed_intent: Option<CompactionIntent>,
    region_check: RegionCheckOfTheIntentDeletion,
    /// 有活意图、R 里没有隔离槽时，R 里的单元在取输入时读好的字节（区域扫描要读的那几个，[`unit_reads_of_the_region_scan`]）。
    unit_bytes_of_the_live_region: Option<UnitBytesReadBeforeTheRegionScan>,
}

/// 锁外算出的一批（[`compute_the_compaction_batch`]）：取输入时拿到的那几样里回锁还要的、算着记下被扫描拒的段、这一批在 GPU 上算了什么，
/// 与提出的发布（或出的错）。全部自有。
struct CompactionBatchComputed {
    batch_under_way: BackgroundBatchUnderWay,
    version: Arc<TransactionOutput>,
    region_check: RegionCheckOfTheIntentDeletion,
    batch_units: CompactionBatchUnits,
    segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
    gpu_work: Option<GpuWorkOfTheBatch>,
    proposal: Result<CompactionBatchProposal, BackgroundComputeError>,
}

/// 要在现行那一版上发的那一批：开这一批时的记账、票里那一版、删意图那一支的核怎么判、批大小、这一批在 GPU 上算了什么、提议。
struct CompactionBatchToPublish {
    batch_under_way: BackgroundBatchUnderWay,
    version: Arc<TransactionOutput>,
    region_check: RegionCheckOfTheIntentDeletion,
    batch_units: CompactionBatchUnits,
    gpu_work: Option<GpuWorkOfTheBatch>,
    proposal: CompactionBatchProposalToPublish,
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

/// 放锁的三段入口交到锁外的那一批的票（D26（后台整理与放置回收） 已定项 13 第 5 条）：会话锁里取的输入（现行那一版的 `Arc` 指针、整理要的几张集合与开关、
/// 有活意图时 R 里读好的单元字节、这一批开头读的后台计算设置与走法、GPU 实现的把手）连同这一趟到此的账。全部自有，`Send + 'static`：不借会话与盘表，
/// 可以交给别的线程算。
pub struct BackgroundCompactionBatchTicket {
    batch: BatchOutOfTheLock,
    trip: TripOfTheBatchOutOfTheLock,
    run: CompactionRunUnderWay,
    input: Box<CompactionBatchInput>,
}

/// 交到锁外那一批所在的这一趟：为什么起跑、推到哪、预算按哪一条算、这一趟前几次调用发的批数、这次调用之前整理发了几次、节奏的写带宽上限与让路的下限、
/// 回锁那一次判不判回退门。回锁之后照它收这一次调用（[`MountedSession::after_the_background_run`]）。
struct TripOfTheBatchOutOfTheLock {
    trigger: BackgroundCompactionTrigger,
    reach: CompactionRunReach,
    budget: CompactionRunBudget,
    batches_published_earlier_in_the_trip: u64,
    compaction_publishes_before_this_call: u64,
    write_bandwidth_limit: CompactionWriteBandwidthLimit,
    yield_floor: CompactionYieldFloor,
    rollback_gate_at_the_relock: RollbackGateAtTheRelock,
}

/// 排空待发被拒的那一趟（[`MountedSessionWithPendingChanges::refusal_of_the_drain_before_the_background_run`]）：为什么起跑、推到哪、预算按哪一条算、
/// 这次调用推多少、这一趟前几次调用发的批数。
struct TripOfTheDrainRefused {
    trigger: BackgroundCompactionTrigger,
    reach: CompactionRunReach,
    budget: CompactionRunBudget,
    span: SpanOfTheBackgroundCall,
    batches_published_earlier_in_the_trip: u64,
}

#[cfg(test)]
mod old_data_unit_reads_back_with_one_copy_failing_in_this_change_tests {
    use super::OldDataUnitReadsBackWithOneCopyFailingInThisChange;

    /// 被空间拒、推一串抬 F 之后重发同一个改动那一形缩小（判决 m4-close-code-r3 第三节第 6 条）：头一次发布读回单元 A、第二次发布又读回 A 与 B，
    /// 第二次里 A 是这次改动读回过的、交回 1；账上 A、B 各一把，按读到的先后。
    #[test]
    fn unit_read_back_again_by_a_later_publish_of_the_same_change_is_kept_once() {
        let mut reads_back = OldDataUnitReadsBackWithOneCopyFailingInThisChange::nothing_read_yet();
        let unit_read_back_by_both_publishes = vec![1u8; 27];
        let unit_read_back_only_by_the_later_publish = vec![2u8; 27];
        assert_eq!(
            reads_back.keep_the_mapping_keys_not_read_back_before_in_this_change(vec![
                unit_read_back_by_both_publishes.clone()
            ]),
            0,
            "头一次发布读回 A：这次改动里没读回过"
        );
        assert_eq!(
            reads_back.keep_the_mapping_keys_not_read_back_before_in_this_change(vec![
                unit_read_back_by_both_publishes.clone(),
                unit_read_back_only_by_the_later_publish.clone()
            ]),
            1,
            "重发那一次又读回 A：A 读回过、只算一次"
        );
        assert_eq!(
            reads_back.mapping_keys,
            vec![
                unit_read_back_by_both_publishes,
                unit_read_back_only_by_the_later_publish
            ],
            "账上 A、B 各一把、按读到的先后"
        );
    }
}
