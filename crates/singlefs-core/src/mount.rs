//! 可写挂载（里程碑「覆盖写、释放、回退与复用」步 3）：进程重开镜像之后先走一次恢复，从盘上重建可写态——所选根、施加前缀之后的根、
//! 上一版全部角色的单元与指针、分配器、下一条 jsn——再取实例代号、给上一个实例写行（D18（块里携带什么信息） 已定项 11）、
//! 推空发布直到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 戊），之后本实例的发布才接在后面。
//! 第一版没有干净关闭标记，重开一律走恢复。
//!
//! 挂着的时候：抬 F（准入那一格与正常卸载共用那一串，D16（发布语义） 已定项 1），与管理员回退——挂着时的一次向前发布
//! （D23（journal 的角色与格式） 已定项 14，[`roll_back_by_a_forward_publish`]）。

use crate::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, JournalSequenceNumber,
    SlotNumber,
};
use crate::admission::{
    admission_reading_of_a_writable_mount_with_node_capacities, admit_on_every_device,
    AdmissionRefusedOnSomeDevices, BytesOnOneDevice, DemandOnDevice, SpaceAdmission,
};
use crate::allocation_record_tree::{
    node_pointers_as_far_as_readable, AllocationRecordTreeGeometry,
};
use crate::allocator::{
    oldest_txg_in_the_ring_rereading_slots_bad_at_mount, AllocationRecord, DeviceFreeMap,
    Placement, PlacementOnDevice, PlacementRefusal, PoolAllocator, ReclaimedReuse,
    RereadOfARootRingSlotBadAtMount, RootReadOnOneRebuildPass, RootRingOccupancy, RootRingOccupant,
    TxgsARingSlotBadAtMountCanHold, UnitAreaStart,
};
use crate::background_compute::{
    admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold,
    AdmittedBackgroundComputeSettings, BackgroundComputeOfThisMount, BackgroundComputeSettings,
    BackgroundComputeSettingsRefused, BackgroundGcCpuCompensationThreshold,
    BackgroundGpuComputeHandle, BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
};
use crate::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use crate::checksum::crc32_castagnoli;
use crate::compaction_plan::{
    root_ring_slots_left_by_a_compaction_publish, CompactionBatchSize, CompactionMountParameters,
    CompactionOfThisMount, CompactionSwitches,
};
use crate::instance_table_sweep::{
    sweep_the_instance_table_at_the_end_of_the_mount, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration,
};
use crate::journal::back_chain_of;
use crate::logged_operations::{live_compaction_intent_of_the_entries, LoggedOperationKind};
use crate::make_filesystem::MakeFilesystemParameters;
use crate::mounted_session::{
    refusal_is_short_of_space, run_compaction_on, CompactionRun, CompactionRunReach,
    DeviceKeepingWritesInMemory, DeviceReadOnceInThisMount, ReadRouteOfThisMount,
};
use crate::pointer::LocationEntry;
use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
use crate::records::{
    LineageRecord, TreeTableEntry, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
    TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use crate::recovery::read_unit_via_locations;
use crate::recovery::{
    allocation_records_fit_the_pool_geometry, every_device_reaches_the_unit_area_start,
    placement_lies_in_the_unit_area_of_its_device,
    unit_area_start_of_the_chosen_system_configuration,
};
use crate::recovery::{
    allocation_records_of_version_without_file_in_the_unit_area_starting_at,
    allocation_records_under_root_in_the_unit_area_starting_at, choose_system_configuration,
    choose_system_configuration_of_the_pool,
    effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once,
    effective_rollback_floor_rereading_the_newest_instance_table_once,
    effective_rollback_floor_under_the_newest_roots_table, fall_back_judged_on_the_three_readers,
    highest_root_txg, highest_tree_identifier_watermark_in_the_ring, instance_table_chain_of_root,
    instance_table_of_root, instance_table_page_pointers_as_far_as_readable, readable_roots,
    readable_roots_rereading_ring_slots_known_to_hold_a_root_once,
    readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once,
    rebuild_version_in_the_unit_area_starting_at, replay_journal,
    root_is_abandoned_by_the_instance_table, scan_journal,
    system_configuration_slots_of_one_device_read_once,
    system_configuration_witness_is_undecidable, tree_table_has_no_entries,
    user_visible_tree_root_pointers, verified_system_configuration_slots, BadRootRingSlotReading,
    EffectiveFloorReadingStillUnreadableAfterOneReread, InstanceTableChain,
    InstanceTableOfTheNewestRootStillUnreadableAfterOneReread, JournalScanReport,
    NamedUnitVerification, PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
    RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
    RootRingSlotKnownToHoldARootStillBadAfterOneReread, SystemConfigurationSlotsOfOneDevice,
    UserVisibleTreeRootPointers,
};
use crate::recovery::{
    choose_root_keeping_every_self_verified_root_read, every_root_ring_slot, read_root_ring_slot,
    PoolReaderOfOneReadPass, RootChosenWithEverySelfVerifiedRootRead, RootRingSlotReading,
};
use crate::root_record::{RootRecord, UnmountMarker};
use crate::root_ring::{
    root_sits_in_the_ring_slot_the_formula_gives, target_for_publish, RootRingSlot,
};
use crate::system_configuration::SystemImmutableSizes;
use crate::transaction::{
    central_mapping_locations_in_the_version,
    copies_of_a_released_unit_failing_the_release_checksum_check,
    next_instance_generation_to_acquire, placement_to_release_after_checking_every_device,
    prepare_the_publish_without_units, prepare_the_row_publish_on_a_version_without_file,
    prepare_the_version_publish, publish_instance_table_on_version_without_file, publish_version,
    publish_version_appending_a_lineage_record, publish_version_with_unmount_marker,
    publish_without_units, read_the_witness_before_the_expected_acquisition_writes,
    refuse_locations_that_do_not_name_two_pool_devices,
    rehearse_every_step_of_the_deletion_queue_walk, role_of_allocation_record_tree_node,
    units_at_these_locations, walk_the_deletion_queue_until_it_is_empty,
    write_the_raised_floor_into_every_system_configuration,
    write_the_witnessed_expected_acquisition, AcquisitionFailed, CodeTwoTreeNodeCapacities,
    ContainerTreeRootUnitPositions, CopyQuarantinedAfterReleaseChecksumMismatch,
    DeletionAndLineageOfThePublish, DeletionQueueWalk, ExpectedInstanceAcquisitionFailed,
    InstanceGenerationPastTheTopOfItsRange, InstanceTableOnlyPublishPlan, InstanceTablePlan,
    InstanceTableRewrite, MappingLookup, PoolVersion, PoolWriter, PublishError, PublishPlan,
    PublishedUnit, RaisedFloorSystemConfigurationWriteFailed, ReadFailedOnTheDisk,
    ReadFailingSelfVerification, ReleaseChecksumCheck, TransactionOutput, TransactionUnit,
    VersionWithoutFilePublishOutput, ZeroUnitPublishPlan,
};
use crate::unlink::DeletionSwitches;
use crate::write_accounting::{WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind};
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub use crate::instance_table::{InstanceRow, InstanceTableRecords};

mod allocator_rebuild;
mod caller_inputs_agreeing_with_the_disk;
mod continuation_at_the_mount;
mod devices_checked_against_the_selected_version;
mod floor_raise_on_admission_and_unmount;
mod floor_raise_sequence;
mod instance_establishment;
mod placements_of_unapplied_publishes_and_newer_readable_roots;
mod previous_version_and_instance_start;
mod read_stage;
mod rehearsal_before_acquisition;
mod rollback_by_a_forward_publish;
mod rollback_floor_ceiling;
mod root_ring_reads_for_the_rebuild;
mod row_publish_and_warm_up;
mod shadow_ledger;
mod writable_mount_entry_points;

pub use allocator_rebuild::*;
pub(crate) use caller_inputs_agreeing_with_the_disk::*;
use continuation_at_the_mount::*;
pub use devices_checked_against_the_selected_version::*;
pub use floor_raise_on_admission_and_unmount::*;
use floor_raise_sequence::*;
use instance_establishment::*;
use placements_of_unapplied_publishes_and_newer_readable_roots::*;
use previous_version_and_instance_start::*;
pub(crate) use read_stage::*;
use rehearsal_before_acquisition::*;
pub use rollback_by_a_forward_publish::*;
pub use rollback_floor_ceiling::*;
use root_ring_reads_for_the_rebuild::*;
pub(crate) use row_publish_and_warm_up::*;
use shadow_ledger::*;
pub use writable_mount_entry_points::*;

/// 可写挂载没做成。
#[derive(Debug)]
pub enum MountError {
    Recovery(RecoveryFailure),
    /// 重建分配器读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出（被抛弃按最新那条根指着的实例表判；一条根归哪块盘按读出它的那个槽所在的区域），
    /// 整环重读一次，两遍逐槽并起来（`merged_readings_of_the_two_root_ring_passes`）整个环仍一条都没有：在取号之前拒可写，盘上逐字节不变，
    /// 只读挂载照常（D23（journal 的角色与格式） 已定项 14 那张单子上的一行，用户 2026-10-05 定「另起拒因」，岔路单
    /// `research/prompts/m3-batch10-code-r1-forks.md` Q3；重读的触发照 Q4 rr2；两遍按槽并照用户 2026-10-05 定 R1「按槽并两遍」，
    /// 判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第六节）。第一遍读出过一条有效根的，并出来的那一份里就有它，不拒。
    /// 与读阶段择不到根的 `Recovery(RecoveryFailure::NoValidRoot)` 分开报：那一拒只读挂载同样挂不上，这一拒读阶段择得到根。
    NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
    /// 所选根下面有文件版本，而环里一条自证过的记录都没有：从盘上重建上一版要一条记录顶着，拿不出来
    /// （树表 0 条的根用不到记录，刚 mkfs 的池不走这里）。
    FileVersionWithoutAnyJournalRecord,
    /// 所选根（抬 F 时是现行那一版的根）指着的实例表沿链有一片解不出行与链指针（D18（块里携带什么信息） 已定项 11「表不可读时」）：
    /// 判不了候选集，在任何写之前拒绝。读得出字节、内容不收的那一类（C866 改法甲：与 [`Self::InstanceTableUnreadable`] 拆开，F2 只为字节丢了退）。
    InstanceTableMalformed,
    /// 所选根（抬 F 时是现行那一版的根）指着的实例表沿链有一片两条位置条目都读不到校验和相符的那份：判不了候选集，在任何写之前拒绝。
    /// 字节丢了的那一类（C866 改法甲，与 [`Self::InstanceTableMalformed`] 拆开）。
    InstanceTableUnreadable,
    Acquisition(AcquisitionFailed),
    /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成，或挂载续做里的一步没发成、不是空间不够：错，连同这次挂载交得出的写账
    /// （[`PublishSequenceFailed`]：续做那一格的已落盘各次是写行、暖机、推的抬 F 与续做里已落盘的每一次，按先后）。
    /// 续做里有一步要开的容器号到顶（C841）在取号之前预演出来时同样交这个成员；取号之前预演续做重走到上限仍没走到结局、最后那一趟读盘时
    /// 又有过读错或读坏（`cause` 是 `PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition` 或
    /// `PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition`）也交这个成员。这几种都在取号之前，
    /// 一个写都没发，三份写账都是空的
    /// （`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`）。
    /// 装箱（与 `RaiseFloorSequencePublishFailed` 同）：写账带上抬 F 先写系统配置那一步之后，不装箱 `MountError` 就大过 clippy
    /// `result_large_err` 的 128 字节。
    Publish(Box<PublishSequenceFailed>),
    /// 抬 F 那一串空发布（D16（发布语义） 已定项 1：推到每块盘上都有一条带新 F 的根才生效）预演过了、真发时有一次发不出去：错，连同抬 F 自己开的
    /// 写入口交得出的写账（[`PublishSequenceFailed`]，增补 2 收口表第 58 行，与可写挂载那一条同一个形态）。
    /// 落盘之前就报得出的错在预演里已经报了（`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`），走到这里的只剩落盘途中失败，
    /// 与真发时读盘核出对不上、同预演分叉的那一格（`raise_rollback_floor` 里那条注释）。
    /// `writes_of_persisted_publishes` 有几份，前面就有几次已经落盘——带新 F 的根在盘上、调用方的现行版本已经是最后落盘的那一版，
    /// F 还没在每块盘上生效，抬 F 回收的槽照旧扣着；一份都没有、第一次在任何写之前被拒时，分配器与抬 F 之前逐项相同
    /// （扣住的槽放开、回收的回到 defer，C546（抬 F 被拒时扣住的槽不退回））。`cause` 是下一次（份数 + 1，从 1 数）的错。`cause` 自己说的「在任何写之前」
    /// （例如 `PublishError::PlacementRefused`）只对出错的这一次成立，不对整串成立（C516（抬 F 那一串发布被拒时前面几次已落盘））。
    RaiseFloorSequencePublishFailed(Box<PublishSequenceFailed>),
    /// 抬 F 那一串 `publishes_in_the_sequence` 次空发布在任何写之前、在分配器的一份拷贝上整串预演（与真发同一段落盘之前的代码，
    /// `rehearse_the_publishes_raising_the_floor`），第 `refused_publish_in_the_sequence` 次（从 1 数）报了 `cause`：这一串一次都不发，
    /// 盘上逐字节不变，分配器与抬 F 之前逐项相同（扣住的槽放开、回收的回到 defer、补的隔离撤掉，C546（抬 F 被拒时扣住的槽不退回））。
    /// 改之前这一串逐次发，第 n 次（n ≥ 2）才被拒时前 n − 1 次已经落盘（`RaiseFloorSequencePublishFailed`，份数 ≥ 1）。
    RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
        refused_publish_in_the_sequence: usize,
        publishes_in_the_sequence: usize,
        cause: PublishError,
    },
    /// 抬 F 那一串（准入与卸载共用）预演过了、第一次发布之前先把新 F 写进每块盘的系统配置、过一道屏障那一步报错
    /// （D16（发布语义） 已定项 1「抬 F 那一串」，已定项 7）：这一串的根一条都没发；报错之前已写进新 F 的盘
    /// （`devices_carrying_the_raised_floor`）留着、不回卷——F 的生效值只增不减，下一次挂载按它们算 F_生效。
    /// 分配器与抬 F 之前逐项相同（扣住的槽放开、回收的回到 defer、补的隔离撤掉）；调用方的现行版本不动。
    /// 装箱：不装箱 `MountError` 就大过 clippy `result_large_err` 的 128 字节。
    RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(
        Box<RaisedFloorSystemConfigurationWriteFailed>,
    ),
    /// 要抬到的 F 低于盘上现算的 F 生效值（`recovery::effective_rollback_floor`：根上带的与系统配置里读得出的取最大）：
    /// 往下「抬」条款没写（D16（发布语义） 已定项 1 只写了抬 F 那一串先写新 F、F 的生效值只增不减），第一版不支持。
    /// 走得到的一格：同一个进程里上一次抬 F 先写系统配置那一步只写进了一部分盘（`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`），
    /// 现行版本的根还带着旧 F，而那几块盘的系统配置里已经是新 F。在任何写之前拒绝，盘上逐字节不变、分配器不动。
    RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
        requested: CheckpointTxg,
        effective: CheckpointTxg,
    },
    /// 要抬的 F 超过上限 min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)（D16（发布语义） 已定项 1）。
    RollbackFloorAboveCeiling {
        requested: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
    /// 树表 0 条的那一版，根记录或它的分配记录树指着的一个单元（`unit`），指针是这个实现写不出的样子（`defect`，两种见
    /// [`VersionWithoutFileUnitPointerDefect`]）⇒ 是一份坏镜像，拒绝挂载。指针是**盘上读来的字节**、不是我们的不变量，
    /// 所以这里报错、不断言（panic 面普查 R5）。在动分配器之前拒绝，盘上逐字节不变。详见 `format_time_allocator` 的文档注释。
    VersionWithoutFileUnitPointerOfABadImage {
        unit: TransactionUnit,
        defect: VersionWithoutFileUnitPointerDefect,
    },
    /// 算抬 F 的上限时一条有效根（不是被抛弃的、txg ≥ F）的树表读不出或解不开：它算不算非空判不了，读不出要走修复、不跳过也不猜，
    /// 这次抬 F 拒绝（不回收、不发布）。
    RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
        root: RollbackTarget,
        failure: RecoveryFailure,
    },
    /// 算抬 F 的上限时读根环：`ring_slot` 是这个进程知道住着一条根的槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过，
    /// `allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`），这一次读坏（`first_reading`），重读一次仍坏（`reread`）：
    /// 这条根在不在判不了，不按有根或没根猜，这次抬 F 拒绝（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定；
    /// C562（抬 F 算上限时根槽读坏当没有根））。在回收、影子账与任何写之前返回：盘上逐字节不变、分配器不动、调用方的现行版本不动。
    /// 这个进程的准入抬 F 里重建分配器两遍里只有一遍自证过的槽不走这一条（重读仍坏拿重建那一遍读出、记在根环表里的那条根顶上，用户 2026-10-06 定 T1 甲）。
    RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
        ring_slot: RootRingSlot,
        first_reading: BadRootRingSlotReading,
        reread: BadRootRingSlotReading,
    },
    /// 取号之前判定时算出的号与取号写之前重算的号不同（两次读系统配置之间有瞬时读错）：判定作废，一个字节都没写。
    InstanceGenerationChangedBeforeAcquisition {
        expected: InstanceGeneration,
        recomputed: InstanceGeneration,
    },
    /// 空间准入不够（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md`
    /// 第四节第 2 条把它接进可写挂载）：扣掉这次挂载的实例切换预留（挂载期承诺量，D28（挂载期承诺量） 已定项 3）与别的几项之后，
    /// 至少一块盘的可用(d) < 0——「实例切换的预留拿得到」这条可写挂载准入合取不成立（D2（RAID 条带策略） 已定项 13：任一不成立即只读挂载；
    /// 这里交回错误，要不要只读挂载由调用方定），而且上一版树表 0 条：抬 F 要带文件的现行版本，推不出空间。
    /// 上一版带文件的不报这一条：写行与暖机之后推抬 F 的空发布再判（`MountSpaceAdmission`，D16（发布语义） 已定项 1「准入」那一行）。
    /// 在**取号之前**返回，一个写都没发、盘上逐字节不变、两块盘系统配置里的实例代号不动。
    SpaceAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        refusal: AdmissionRefusedOnSomeDevices,
    },
    /// 写行那次发布在取号之前的预演（发布路径落盘之前那一段：释放核验、这次要换下的那条实例表旧链逐片核、两棵多层码 2 树的形状、
    /// 分配记录树重写集合的固定点）报错：在**取号之前**拒绝，盘上一个字节都不动、
    /// 两块盘系统配置里的实例代号不动（增补 2 第 20a 行，代码三方第一轮打中）。改之前这些只在发布路径里算，
    /// 取号（两次系统配置槽写 + 一道屏障）已经写完才报出来，而取号的回卷只管取号自己那几次写报错、管不到之后的发布失败 ⇒
    /// 那样的池此后每试一次可写挂载就再烧一个实例代号。
    RowPublishAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        cause: PublishError,
    },
    /// 暖机那几次空发布（写行之后推到本实例的根覆盖每块盘，D16（发布语义） 已定项 8 戊）里第几次在取号之前的预演报错：连写行那次一起
    /// 在取号之前预演，报错在任何写之前返回（增补 2 第 20a 行，2026-09-18 用户定案）。分配记录树按位置寻址之后
    /// （D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，暖机这几次还报得出的只剩两棵多层码 2 树长过 256 层、
    /// 分配记录树重写集合迭代不收敛（只在强制复用窗口为 0 的只供测试的开关下可能）这类，落点取不到另报
    /// `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`。
    WarmUpAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        /// 第几次暖机空发布算不过，从 1 数。
        warm_up_publish_index: usize,
        /// 这次挂载要推几次暖机空发布（按根环落点公式现算）。
        warm_up_publishes_planned: usize,
        cause: PublishError,
    },
    /// 取号之前在分配器的一份拷贝上把这次挂载取号之后要发的那一串（写行一次、暖机 `warm_up_publishes_planned` 次）逐次取落点，
    /// 第 `publish_index` 次（从 0 数，0 是写行那次）的 `unit` 取不到，`refusal` 是分配器给的原因
    /// （`dry_run_of_the_publishes_after_acquisition`）。**取号之前的准入怎么把这次挂载要写的落点算进去，条款没定**：
    /// 可写挂载的准入要「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13；D23（journal 的角色与格式） 已定项 14「切换要用的块在挂载准入时预留」），
    /// 预留按 D28（挂载期承诺量） 已定项 3 算，其中暖机那一半的 c_max 与已定项 4 的 checkpoint 保留池从哪读没有条款
    /// （C363（现算保留池时树高从哪读没有条款）），D28（挂载期承诺量） 已定项 1 那条式子另一边的「需求」怎么摊到每块盘也没有
    /// （C370（需求、可用与 df 没有共同单位））。第一版不算那条式子，只把「这一串自己的落点取不到」挪到取号之前：在任何写之前返回，
    /// 盘上逐字节不变、两块盘系统配置里的实例代号不动。改之前取号写完、写行或暖机才被落点拒绝（`Publish`），实例代号一去不回
    /// （里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 39 行那一族：单元区 240 槽的小盘上走得到）。
    PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
        instance_to_acquire: InstanceGeneration,
        publish_index: usize,
        warm_up_publishes_planned: usize,
        unit: TransactionUnit,
        refusal: PlacementRefusal,
    },
    /// 可写挂载准入不成立：`devices` 列出不带所选那一版的每一块盘（施加前缀之后的那一版——
    /// 新实例的第一次发布接在它后面、照抄它的单元），各带一样它缺的（[`SelectedVersionLackingOnDevice`]）。
    /// 依据：D2（RAID 条带策略） 已定项 13 挂载准入「可写设备数 ≥ w 的下限，任一不成立即只读挂载」，已定项 6「`w ≥ 2` 是硬下界」；
    /// D18（块里携带什么信息） 已定项 11「可写挂载的顺序」里「可见」= 独占打开成功且系统配置读得通，「前提」里打开过又掉了、
    /// 缺号期间池里发布过的盘整盘作废、只读到重同步完成。第一版每个单元两块盘各一份，拿这样的盘接着写，照抄过去的单元只剩一份。
    /// 在取号之前返回，一个写都没发、盘上逐字节不变。只读挂载照旧（`mounted_read::mount_read_only`：每个单元按位置条目逐条试，
    /// 读到的是验得过的那一份，也就是带着这一版的那块盘上的）。重同步第一版不做，留给 C120（分叉盘回归的判定与重同步）。
    WritableMountRefusedByDevicesWithoutTheSelectedVersion {
        selected_version: RollbackTarget,
        /// 所选那一版那次发布的末条记录的 jsn（`journal_position_of_the_selected_version`）：判「落后」拿它比。
        selected_version_journal_position: JournalSequenceNumber,
        devices: Vec<DeviceWithoutTheSelectedVersion>,
    },
    /// 可写挂载准入不成立：交进来的盘数 `devices_handed_in` 低于 w 的下限 `stripe_width_lower_bound`（[`STRIPE_WIDTH_LOWER_BOUND`]）。
    /// 依据：D2（RAID 条带策略） 已定项 13 挂载准入的第一条合取「可写设备数 ≥ w 的下限，任一不成立即只读挂载」，
    /// 已定项 6「`w ≥ 2` 是硬下界（零冗余的条带不许发出）」，已定项 9 第一版跑 2 块盘；D18（块里携带什么信息） 已定项 11
    /// 「可写挂载的顺序」先判这一条。可写设备数取交进来的盘表里不同设备身份的个数（同一个身份交两次算一块）：准入在这次挂载的第一个写之前判，
    /// 已定项 13 运行期那一行的探针写不在这里做。
    /// 在选系统配置、恢复、重建上一版、读分配记录树与任何写之前返回：一块盘都没读，盘上逐字节不变。
    /// 要不要只读挂载由调用方定（同 `SpaceAdmissionRefusedBeforeAcquisition`）。
    WritableDeviceCountBelowTheStripeWidthLowerBound {
        devices_handed_in: DeviceCount,
        stripe_width_lower_bound: DeviceCount,
    },
    /// 交进来的盘表里有设备身份出现了不止一次（`[(盘 0, A), (盘 1, B), (盘 0, C)]`）：取号、发布与系统配置轮换按盘表逐项写，
    /// 多出来的那一项（C）不是盘表说的那块盘，却会收到本池的系统配置写与单元写（代码审阅第 18 条）。`repeated` 按设备身份升序列出每个重复的身份与它交了几次。
    /// 判在「可写设备数 ≥ w 的下限」那一条之后（那一条数的是不同身份，同一块盘交两次、别的一块都没交的仍报那一条），
    /// 在选系统配置、恢复与任何读写之前返回：一块盘都没读，盘上逐字节不变。
    DeviceIdentitiesHandedInMoreThanOnce {
        repeated: Vec<RepeatedDeviceIdentity>,
    },
    /// 调用方交进来的 mkfs 参数（`MakeFilesystemParameters`）与盘表，同盘上择到的那份系统配置（`recovery::choose_system_configuration`）的
    /// 系统不可变配置不一致：`disagreeing_fields` 按字段表次序列出参数里不一致的每一项（代码审阅第 17 条）；`disagreeing_device_table`
    /// 列出盘表不一致的那几项——交进来的盘数与系统配置里的设备数 `devs` 不同、某块盘自证过的系统配置里的本盘设备号不是盘表给它的身份
    /// （[`DeviceTableDisagreement`]）；挂着之后收盘表的入口另报交进来的某块盘两槽一份本池自证过的系统配置都没有
    /// （[`DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`]，代码三方 m2-closeout-code-r1 Z3-A 乙）；
    /// 某块盘这一刻探测到的 `physical_block_size` 不是系统配置记着的那个，同样报在盘表那一张里
    /// （[`DeviceTableDisagreement::PhysicalBlockSizeProbedOnTheDeviceDiffersFromTheSystemConfiguration`]，D12（目标介质） 已定项 4，C1040）。
    /// 两张清单至少一张不空。系统不可变配置 mkfs 之后不可改
    /// （D22（单元原子性怎么合成） 已定项 26 第一档），写入口按择到的那一份建（取号、写行、暖机、推抬 F 的系统配置写、
    /// 根槽与 journal 记录的落点都从它来），系统配置槽写又按盘表逐项写设备数与本盘设备号；两边不一致说明调用方拿错了池、参数或盘，
    /// 照调用方的写会把盘上不可变段改写成调用方给的值。可写挂载在选系统配置之后、恢复与任何写之前返回；挂着之后收调用方参数与盘表的入口
    /// （抬 F、准入抬 F、一次准入里再推一串、正常卸载）在任何写之前返回：盘上逐字节不变。
    CallerParametersDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields: Vec<MakeFilesystemParameterField>,
        disagreeing_device_table: Vec<DeviceTableDisagreement>,
    },
    /// 挂着之后收盘表的入口（抬 F、准入抬 F、一次准入里再推一串、正常卸载）在参数、盘表与「可见」都核过之后，交进来的这几块盘落后于
    /// 现行那一版、又缺它的单元或它的根（[`DeviceBehindTheCurrentVersion`]，与可写挂载取号之前逐盘核的落后支同一判，「所选那一版」换成调用方持着的
    /// 现行那一版；D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定；缺根那一半是 C585
    /// （会话发布放进发布途中取下的旧快照） 用户定的改法 C）。不核这一判时，
    /// 同池一份旧快照的盘经这些入口照常收到本池的写、系统配置被轮换到「跟得上」，之后可写挂载的落后支就认不出它
    /// （代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」）。只读过每块盘的两个系统配置槽、落后那几块盘上现行那一版单元的落点
    /// 与（单元都在时）现行那一版的根槽，
    /// 在任何写之前返回（抬 F 与卸载在动分配器之前）：盘上逐字节不变。调用方要做的是换回这个池现行的盘，不是改参数或盘表的次序。
    DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
        devices: Vec<DeviceBehindTheCurrentVersion>,
    },
    /// 盘上读来的一个编号已到它那一格的顶、下一次要用的号（它加一）装不下（代码审阅第 36 条）：可写挂载要的新实例代号
    /// （盘上最大的实例代号 + 1）、新实例第一次发布的 txg（环里最大的 txg + 1）与暖机那几次、抬 F 与正常卸载那一串接在现行那一版后面的 txg。
    /// 这些号是盘上读来的 4 / 8 字节，坏镜像、外来镜像才走得到。在任何写之前返回（抬 F 与卸载在动分配器之前），盘上逐字节不变。
    SequenceNumberPastTheTopOfItsRange(SequenceNumberAtTheTopOfItsRange),
    /// 取号之前空间准入不够、写行与暖机之后推抬 F 时，抬 F 自己报了不是空间不够的错（块设备错、根环槽读坏又重读仍坏、预演被别的理由拒……）：
    /// 那时实例已取、行已写，写行、暖机与之前推成的那几串已落盘（D16（发布语义） 已定项 1「准入」那一行）。
    /// 写入口、分配器与现行那一版随错丢掉，这次挂载已落盘的写账都在 [`FloorRaiseFailedAfterTheMountsPublishes`] 里
    /// （增补 2 收口表第 58 行：与 `Publish` 同一个形态，不交出来设备一层数到的写与程序交得出的账对不上）。
    /// 抬 F 那一串自己也取不到落点、被空间准入拒的不走这里（推满仍不够，挂载照样做成，`MountSpaceAdmission::StillShortAfterTheFloorRaises`）。
    /// 挂载续做里为空间推的抬 F 自己报了不是空间不够的错，同样交这个成员：那时写行、暖机、准入推的、续做发成的各步与续做推成的那几串
    /// 都已落盘，写账口径见 [`FloorRaiseFailedAfterTheMountsPublishes`]。
    /// 装箱：不装箱 `MountError` 就大过 clippy `result_large_err` 的 128 字节。
    FloorRaiseFailedAfterTheMountsPublishes(Box<FloorRaiseFailedAfterTheMountsPublishes>),
    /// C554 乙（用户 2026-09-27 定：重读后再判，R = 1；岔路单 `research/prompts/c554-fix-forks.md` #1「挂载时有一条根（或它引用的账）暂时读不出」）：
    /// 可写挂载读到的样子里有一样更新的东西读不出，重读一次（D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」）仍读不出——
    /// 不按读得出的那一版往下走，不把读不出的那一版当成被抛弃，拒可写挂载。是哪一样读不出见 [`StillUnreadableAfterOneReread`]。
    /// 在取号之前返回：一个写都没发，盘上逐字节不变。只读挂载不走这一判，照常（`mounted_read`、`recovery::recover`）。
    /// 挂着时抬 F 那一串（准入与卸载共用）算 F 生效值、重算影子账读根环重读一次仍读坏，同样报这一条
    /// （[`StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor`]、
    /// [`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`]）：在动分配器与任何写之前拒这一次抬，
    /// 盘上逐字节不变、分配器与调用方的现行版本不动。
    /// 装箱：不装箱 `MountError` 就大过 clippy `result_large_err` 的 128 字节。
    NewerStateStillUnreadableAfterOneReread(Box<StillUnreadableAfterOneReread>),
    /// 读阶段判完、判据 N-配置 为假之后，取号之前的逐盘核或取号那一刻读系统配置又读出判不出（R2）或某块盘一份本池自证过的都没有
    /// （D23（journal 的角色与格式） 已定项 14「取号那一刻按同一个判不出（R2）判：判不出就在第一个取号写之前拒……这一步在某块盘一份自证过的
    /// 系统配置都没有时也拒」）：拒可写。与读阶段判据 N-配置 那一拒（`NewerStateStillUnreadableAfterOneReread`）分开报——两遍读之间的瞬时读错
    /// 在崩溃镜像上复算不出，并成一个拒因时挂载透明在带读故障的档里会把这一种合法拒判成不合法（C664（挂载拒绝合法复算把两种拒因报成一个）；
    /// D13（验证路线） 已定项 1 挂载透明射程那一句）。一个字节都没写，盘上逐字节不变。
    /// 装箱：不装箱 `MountError` 就大过 clippy `result_large_err` 的 128 字节。
    SystemConfigurationUnreadableAfterTheReadStage(
        Box<SystemConfigurationUnreadableAfterTheReadStage>,
    ),
}

/// 树表 0 条的那一版上一个单元的指针坏在哪（[`MountError::VersionWithoutFileUnitPointerOfABadImage`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VersionWithoutFileUnitPointerDefect {
    /// 根记录里的分配记录树根指针全零（这一版没有自己的分配记录树），而根记录指着的实例表或树表不是 mkfs 写的那一版
    /// （指针的诞生 txg 不是 0）：这一版的账只剩这两条指针，别的活单元会被当空闲槽发出去。
    BirthTxgNotZeroWithoutAnAllocationRecordTree { birth_txg: CheckpointTxg },
    /// 指针的两条位置条目落在不同的槽：第一版的布局是两盘同槽（一个单元整个落在一列上、两盘各一份）。
    /// 拒的是「这一条指针的两条位置条目不同槽」，与「各盘算出来的落点不同」（`PlacementRefusal` 那两条第一版不支持的池形状）
    /// 不是同一件事：那两条说的是写侧算出来的落点，这一条说的是盘上已经写着的字节。
    LocationEntriesOnDifferentSlots(LocationEntriesOnDifferentSlots),
}

/// 读阶段判完之后系统配置又读不出（[`MountError::SystemConfigurationUnreadableAfterTheReadStage`]）：读阶段往下走用的那一遍读数，与之后读到的是哪一样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemConfigurationUnreadableAfterTheReadStage {
    /// 读阶段往下走用的那一遍（判据 N-配置 在这一遍为假）：它的所选那一版与见证读数。
    pub read_stage: SelectedVersionAgainstTheWitness,
    pub unreadable: SystemConfigurationUnreadableWhen,
}

/// 读阶段判完之后在哪一处、读到系统配置的哪一种读不出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemConfigurationUnreadableWhen {
    /// 取号之前的逐盘核（落后支要读每块盘自证过、世代号最大的那一份）读到 `device` 两槽里一份本池自证过的都没有，又没被取号补写：
    /// 为什么没补写在 `not_rewritten_because`（[`DeviceNotRewrittenByTheAcquisition`]，C874（补写那一支进不了崩溃测试、运行时看不出））。
    /// 只报盘表次序第一块这样的盘、它第一件没过的（逐盘那一圈遇到第一块就整个返回，补写闸按闸一、闸五、闸三、闸四、闸二的次序判、
    /// 第一件不过就返回）：管理员按报的那一件修好再挂，可能撞上同一块盘的下一件或下一块盘。
    CheckingEachDeviceBeforeTheAcquisition {
        device: DeviceIdentity,
        not_rewritten_because: DeviceNotRewrittenByTheAcquisition,
    },
    /// 取号那一刻读见证值时判不出（R2：有槽设备报读错，且没有一块盘两槽都自证过）；`first_device_reporting_a_read_error` 是盘表次序第一块报读错的盘。
    AcquisitionReadsAnUndecidableWitness {
        first_device_reporting_a_read_error: DeviceIdentity,
    },
    /// 取号那一刻读见证值时 `device` 两槽里一份本池自证过的都没有。
    AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot { device: DeviceIdentity },
}

/// 可写挂载重读一次之后仍读不出的是哪一样（[`MountError::NewerStateStillUnreadableAfterOneReread`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StillUnreadableAfterOneReread {
    /// 判据 N-配置（定义取自 E158 第 3 次跑登记 `research/prompts/e158-r3-prereg.md`「`| **N-配置** |`」那一行）：系统配置见证过一次比所选那一版新的发布——
    /// 系统配置在发布的根槽 FUA 之后才轮换（D16（发布语义） 已定项 7），见证到的发布它的根落过盘——而读阶段（择根、扫 journal、重放）
    /// 交出的所选那一版比它旧；见证读判不出也按真，判不出有两形：R2 本身（有槽设备报读错，且没有一块盘两槽都自证过，
    /// [`WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots`]），与乙入判不出（读得出而自证不过的槽当读错计，
    /// [`WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots`]）。在这次挂载的读缓存上重做整个读阶段一遍
    /// （[`ReadStageCache`]；系统配置槽不经读缓存、直接从盘上重读），按重读那一遍的读数仍判真。`first_read`、`reread` 是两遍各自的读数。
    PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read: SelectedVersionAgainstTheWitness,
        reread: SelectedVersionAgainstTheWitness,
    },
    /// 代码审阅第 22 条：重建分配器时按读阶段择到的那条根（根环里最新的那条）的实例表判哪几条根被抛弃（影子账、回收的门槛、根环表都按它），
    /// 那张表读不出，重读一次仍读不出。不按「没有一条根被抛弃」往下走（那样一个槽都不隔离、`abandoned_roots_unreadable` 仍是 0）。
    InstanceTableOfTheNewestRootForTheShadowLedger {
        /// 读阶段择到的那条根（判退那一形也是它），按它的实例表判抛弃；两遍读的都是这一条根的表。
        root_chosen_by_the_read_stage: RollbackTarget,
    },
    /// 挂着时抬 F 算 F 生效值（`recovery::effective_rollback_floor_rereading_the_newest_instance_table_once`）或管理员回退判候选算
    /// F_生效（`recovery::effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）：根环里最新那条根
    /// 指着的实例表（按它判哪几条根被抛弃）读不出、解不开，或一条自证过的根都择不到，连根环一起重读一次仍是这样（C554 乙报告 Q6）。
    /// 不按「不按表滤」往下走（那样被抛弃时间线上的根带的 F 也算进生效值）。与乙那一族同级。
    InstanceTableOfTheNewestRootForTheEffectiveFloor {
        /// 重读那一遍择到的最新那条根；一条自证过的根都没读到时 `None`。
        newest_root_on_the_reread: Option<RollbackTarget>,
    },
    /// 挂着时抬 F 重算影子账与回收门槛读根环（`recovery::readable_roots_rereading_ring_slots_known_to_hold_a_root_once`）或管理员回退
    /// 判候选、算 F_生效 读根环：`ring_slot` 是这个进程知道住着一条根的槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过），这一次读坏
    /// （`first_reading`：读不出或自证不过），重读一次仍坏（`reread`）——那一槽里的根在不在判不了，不按有根或没根猜（D16（发布语义） 已定项 1
    /// 「根槽这一次读坏」那一行全句，用户 2026-09-26 定）。挂载那一刻就读坏、之后没写过的槽不走这一条，当没有根；重建分配器两遍里只有一遍
    /// 自证过的槽在抬 F 那一遍也不走这一条（拿重建那一遍读出、记在根环表里的那条根顶上，用户 2026-10-06 定 T1 甲），管理员回退那一遍照走（同日定 T2：照今天拒）；
    /// 根环表里记成被抛弃根的槽在抬 F 那一串（准入算上限、抬 F 重算影子账与回收门槛，卸载同一串）也不走这一条（照表里记着的落点进影子账、
    /// 不进上限，用户 2026-10-08 定 H1「被抛弃根的槽不拒」），管理员回退那一遍照走。
    /// 与乙那一族同级。
    RootRingSlotKnownToHoldARoot {
        ring_slot: RootRingSlot,
        first_reading: BadRootRingSlotReading,
        reread: BadRootRingSlotReading,
    },
    /// 影子账读一条被抛弃根的账（树表或分配记录树）读不出、从中央映射也现算不成，重读一次仍是这样：拒可写
    /// （C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算那一条的最后一支，用户 2026-09-28 定）。
    /// 可写挂载的 C 读一次没施加的发布那一拒不报这一个，报 [`Self::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`]。
    AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget },
    /// 可写挂载的 C 读一次没施加的发布——水位之上那几条记录拼的根，或比所选那一版新、按最新那条根的实例表有效、读得出的根（F2 退回之后
    /// 被退掉的那条）——的账（树表或分配记录树）读不出、那次发布读得出的记录又不齐（本次发布内序号缺一条，或没有带末条标志的那条），
    /// 照被抛弃根那一份从中央映射现算也不成，重读一次仍不成，现行账里又没有 C694 的 B 那道凭据（`C_MAPPING` 收窄，判决
    /// `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K2 W1 / W2 H1、K2 W2 H3 两行与第三节第 1 条）。
    /// 较新可读根那次发布一条记录都读不出时，只在那条根的树表与中央映射树根两个指针都没有「两份都是本池更新实例的单元」时走到这一拒
    /// （C834，用户 2026-10-15 定「同一指针两份都更新」，`research/prompts/c834-r3-main-verification.md`「四、用户定案」；
    /// 单元头里的 fsid 与本池的相同才算本池的，用户定「比 fsid」，零轮）。
    /// `unapplied_publish` 点名那次发布（那条较新可读根就是它自己那次发布）。与 C393 那一拒（[`Self::AccountOfAnAbandonedRoot`]）分开报：
    /// 这一拒列在 D23（journal 的角色与格式） 已定项 14 那张单子上，C393 那一拒不在（用户 2026-10-08 定「乙：core 另起错误成员」）。
    /// 在取号之前拒，盘上逐字节不变，只读挂载照常。
    AccountOfAnUnappliedPublishWithIncompleteReadableRecords { unapplied_publish: RollbackTarget },
    /// 可写挂载读阶段：水位之上有一次没施加的发布 `publish`，读得出的那几条记录的「本次发布内序号」不是 1..N 连着或没有一条带末条标志，
    /// 而它们点名的单元又有一个验不过；在这次挂载的读缓存上重做读阶段一遍仍是这样（F1 乙，用户 2026-10-07 定「缺记录就拒可写」，
    /// 判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条）。缺的那条记录点名的单元挂载时谁都不知道是哪几个，
    /// 隔离不到；在取号之前拒，盘上逐字节不变，只读挂载照常。
    PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { publish: RollbackTarget },
}

impl From<RootRingSlotKnownToHoldARootStillBadAfterOneReread> for StillUnreadableAfterOneReread {
    fn from(still_bad: RootRingSlotKnownToHoldARootStillBadAfterOneReread) -> Self {
        Self::RootRingSlotKnownToHoldARoot {
            ring_slot: still_bad.ring_slot,
            first_reading: still_bad.first_reading,
            reread: still_bad.reread,
        }
    }
}

impl From<InstanceTableOfTheNewestRootStillUnreadableAfterOneReread>
    for StillUnreadableAfterOneReread
{
    fn from(still_unreadable: InstanceTableOfTheNewestRootStillUnreadableAfterOneReread) -> Self {
        Self::InstanceTableOfTheNewestRootForTheEffectiveFloor {
            newest_root_on_the_reread: still_unreadable.newest_root_on_the_reread.map(
                |(instance, checkpoint_txg)| RollbackTarget {
                    instance,
                    checkpoint_txg,
                },
            ),
        }
    }
}

impl From<EffectiveFloorReadingStillUnreadableAfterOneReread> for StillUnreadableAfterOneReread {
    fn from(still_unreadable: EffectiveFloorReadingStillUnreadableAfterOneReread) -> Self {
        match still_unreadable {
            EffectiveFloorReadingStillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot(
                slot,
            ) => slot.into(),
            EffectiveFloorReadingStillUnreadableAfterOneReread::InstanceTableOfTheNewestRoot(
                table,
            ) => table.into(),
        }
    }
}

/// 读阶段交出的所选那一版（`effective_root`）与判据 N-配置 的读数：「系统配置见证过比它新的发布」判不判得真。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedVersionAgainstTheWitness {
    pub selected_version: RollbackTarget,
    pub witness: NewerPublishWitness,
}

impl SelectedVersionAgainstTheWitness {
    /// 判据 N-配置 为真：系统配置见证过一次比所选那一版新的发布（判不出的也按真）。
    #[must_use]
    pub fn witnesses_a_publish_newer_than_the_selected_version(&self) -> bool {
        match self.witness.comparison {
            WitnessedCounterComparison::NothingWitnessed => false,
            WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter,
            } => self.witness.witnessed_journal_counter > selected_version_last_record_counter,
            WitnessedCounterComparison::AgainstTheRecordAtTheWitnessedCounter { record } => {
                (record.instance, record.checkpoint_txg)
                    > (
                        self.selected_version.instance,
                        self.selected_version.checkpoint_txg,
                    )
            }
            WitnessedCounterComparison::Undecidable
            | WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                true
            }
            // 乙入判不出：读得出而自证不过的槽当读错计时判不出，与 R2 本身判不出同一个处置（按真）。
            WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots => {
                true
            }
        }
    }
}

/// 判据 N-配置 的一次读数（E158 第 3 次跑登记第 347 行，逐字）：c_见证 与拿什么跟它比。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewerPublishWitness {
    /// c_见证：池里每块盘两槽里全部自证过（整槽校验和过、fsid 与本池相同）的系统配置槽的 journal tail 的最大值；一份都没有时 0。
    /// 只取计数器、不取实例代号：jsn 计数器全池接着走（D23（journal 的角色与格式） 已定项 14 注 3），取号那一写的实例代号比它写的 tail 所属的那次发布新。
    pub witnessed_journal_counter: u64,
    pub comparison: WitnessedCounterComparison,
}

/// 判据 N-配置 拿 c_见证 跟什么比，按次序判（E158 第 3 次跑登记第 347 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessedCounterComparison {
    /// c_见证 = 0（没有自证过的槽，或读得出的槽都是 mkfs 与 mkfs 之后第一次取号写的 0）：判据为假。
    NothingWitnessed,
    /// 所选那一版那次发布带「本次发布末条」标志的记录读得出（任一份），计数器 c_E：判据 = c_见证 > c_E。
    AgainstTheSelectedVersionsLastRecord {
        selected_version_last_record_counter: u64,
    },
    /// 所选那一版的末条读不出、计数器等于 c_见证 的记录读得出：判据 = 它的 (实例代号, checkpoint_txg) > 所选那一版的。
    /// 读得出几条（不同实例的记录落回过同一个计数器）时取键最大的那一条。
    AgainstTheRecordAtTheWitnessedCounter { record: RollbackTarget },
    /// 两条都读不出：判不出，按判据为真处置（登记里的标签 `undecidable`）。
    Undecidable,
    /// 见证读判不出（R2，`recovery::system_configuration_witness_is_undecidable`）：有槽设备报读错，且没有一块盘两槽都自证过——
    /// c_见证 可能缺了那一槽，按判据为真处置（C331（择根倒挂压过已确认的写），D22（单元原子性怎么合成） 已定项 8 射程）。
    /// 设备没报读错、有槽读得出而自证不过的落在下一个成员（没有一块盘两槽都自证过时）或照常取自证过的槽比（有一块盘两槽都自证过时）。
    SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
    /// 乙入判不出（C971（多处读坏时可写挂载写出倒挂的新实例根），读法逐字在 `research/prompts/e205-preregistration.md`「乙入判不出」那一行）：
    /// 设备没报读错，但有槽读得出而自证不过（撕裂、读回全零、整槽校验和过而 fsid 不是本池的；布局位不认识、每区槽数越界的槽不算），
    /// 且没有一块盘两槽都自证过——
    /// 那几槽并进 R2「有槽设备报读错」那一半当读错计，按判据为真处置：读阶段重读一次（系统配置槽直接从盘上重读），
    /// 重读那一遍仍是这一形就拒可写（[`StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion`]）。
    /// 只在可写挂载读阶段的见证读里判；取号那一刻读见证值与择根退回那一处照 R2 本身判（[`system_configuration_witness_is_undecidable`]）。
    SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots,
}

/// 可写挂载在「重读一次」之前做什么（D16（发布语义） 已定项 1「根槽这一次读坏」那一行：这一次读不出就重读一次，仍坏就拒）。
/// 产品路径立即重读，两次读之间不等、不另设间隔。只供测试的那一臂在重读之前调用例给的钩子
/// （`.claude/rules/fs-design.md` 五条硬要求第 2 条：用例在钩子里撤掉暂时的读故障，强制进入「重读读得出」那一支；
/// 或在钩子里让重读那一遍也读坏，强制进入「重读仍读不出」那一支）。
/// 每处「重读一次」之前各调一次：读阶段那一次（判据 N-配置），重建分配器读读阶段择到的那条根的实例表那一次（代码审阅第 22 条），
/// 重建分配器读根环有一块盘一条不被抛弃的自证有效根都没读出、整环重读那一次（用户 2026-10-05 定「重读」与 Q4 rr2），影子账里一条被抛弃根的账与从映射现算都不成、
/// 重读它那一次（C393，每条这样的根各一次）。
pub enum BeforeTheOneReread<'hook> {
    RereadImmediately,
    CallTheTestOnlyHookFirst(&'hook mut dyn FnMut()),
}

impl BeforeTheOneReread<'_> {
    fn before_rereading(&mut self) {
        match self {
            BeforeTheOneReread::RereadImmediately => {}
            BeforeTheOneReread::CallTheTestOnlyHookFirst(hook) => hook(),
        }
    }
}

/// 这次挂载重读过哪几样（C554 乙，R = 1）：可观测，挂载做成之后在 [`MountOutput::rereads`] 里报（五条硬要求第 4 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RereadsOfThisMount {
    pub read_stage: ReadStageSettled,
    pub instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead,
    pub root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead,
}

/// 读阶段在哪一遍判完（判据 N-配置 为假的那一遍就是往下走用的那一遍：它的所选根、记录与所选那一版）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadStageSettled {
    OnTheFirstRead {
        first_read: SelectedVersionAgainstTheWitness,
    },
    /// 第一遍判据 N-配置 为真、或水位之上有缺记录而点名单元验不过的发布（F1 乙），在读缓存上重做整个读阶段一遍，重读那一遍两样都判假：
    /// 往下走用重读那一遍。
    OnTheOneReread {
        first_read: SelectedVersionAgainstTheWitness,
        reread: SelectedVersionAgainstTheWitness,
    },
}

/// 重建分配器时读阶段择到的那条根的实例表在哪一遍读出来（代码审阅第 22 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstanceTableOfTheNewestRootRead {
    OnTheFirstRead,
    OnTheOneReread,
}

/// 重建分配器读根环（根环表、回收门槛与影子账用的那一份读数）在哪一遍定下：第一遍有一块盘一条不被抛弃的自证有效根都没读出就整环重读一次
/// （用户 2026-10-05 定「重读」，Q4 定 rr2：不只两块盘都没读出时；一条根归哪块盘按读出它的那个槽所在的区域），用的是两遍逐槽并起来的那一份
/// （用户 2026-10-05 定 R1「按槽并两遍」）；并起来整个环一条都没有就拒
/// （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，不交回这个值）。
/// 只做过 mkfs、环里只有第 0 代根的池第一遍就够：mkfs 在三个区域的槽 0 各写一份第 0 代根，每块盘都读得出一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootRingOfTheRebuiltAllocatorRead {
    OnTheFirstRead,
    OnTheOneReread,
}

/// 到了顶的是哪一个编号、它现在的值（`MountError::SequenceNumberPastTheTopOfItsRange`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequenceNumberAtTheTopOfItsRange {
    /// 盘上最大的实例代号（系统配置槽与根记录里的）已是 `u32::MAX`：取不出新号。
    InstanceGeneration(InstanceGeneration),
    /// 要接在它后面发布的那个 txg 已是 `u64::MAX`。
    CheckpointTxg(CheckpointTxg),
}

/// 可写挂载推的抬 F 自己报错时（`MountError::FloorRaiseFailedAfterTheMountsPublishes`）交得出的东西。两处交它：写行与暖机之后为准入推的
/// 那一串，与挂载续做里为空间推的那一串。
/// 这次挂载交给设备的写（取号两轮那几次系统配置槽写除外）= `writes_of_persisted_publishes` 各份 + `floor_raises` 各串先写系统配置那一步
/// + 写行之后那一处另加 `floor_raises` 各串的各次空发布（续做那一处它们已在 `writes_of_persisted_publishes` 里，不另加）+ `cause` 里那一串自己的写账。
#[derive(Debug)]
pub struct FloorRaiseFailedAfterTheMountsPublishes {
    /// 抬 F 自己报的错，原样：它那一串自己的写账（先写系统配置那一步、已落盘的几次空发布、失败那一次已记的写）在它里面
    /// （`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`、`RaiseFloorSequencePublishFailed`）。
    pub cause: MountError,
    /// 这次挂载在推报错那一串之前已经落盘的发布各自的写，按先后（与 `PublishSequenceFailed::writes_of_persisted_publishes` 同一个口径；
    /// 取号两轮那几次系统配置槽写不是发布、不在这里）。写行之后那一处：写行那次在最前，之后暖机每次一份，推成的那几串的空发布不在这里
    /// （在 `floor_raises` 里）。续做那一处：按 txg 先后接着写行、暖机、准入推的那几串的各次空发布、续做发成的各步与续做里推成的那几串的各次空发布——
    /// 与续做那一格交 `Publish` 时的 `writes_of_persisted_publishes` 同一张表，崩溃注入照同一个划法从里面划出续做段。
    pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
    /// 报错那一串之前已经推成的那几串抬 F，按先后（每一串先写系统配置那一步与各次空发布的写都在里面，都已落盘）。续做那一处是准入推的那几串
    /// 接着续做里推成的那几串，它们的各次空发布同时在 `writes_of_persisted_publishes` 里。
    pub floor_raises: Vec<RaisedFloor>,
}

/// 调用方交进来的参数或盘表不对：可写挂载、挂着之后收参数与盘表的入口（抬 F、准入抬 F、一次准入里再推一串、正常卸载）与管理员回退
/// 共用一套核（[`caller_inputs_agreeing_with_the_disk`]）。两个成员与 `MountError` 那两个
/// 一一对应（`From`），管理员回退报成 `RollbackError::CallerInputsDisagreeWithTheDisk`。
#[derive(Debug)]
pub enum CallerInputsDisagreeingWithTheDisk {
    /// 同 `MountError::DeviceIdentitiesHandedInMoreThanOnce`。
    DeviceIdentitiesHandedInMoreThanOnce {
        repeated: Vec<RepeatedDeviceIdentity>,
    },
    /// 同 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`：两张清单至少一张不空。
    ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields: Vec<MakeFilesystemParameterField>,
        disagreeing_device_table: Vec<DeviceTableDisagreement>,
    },
    /// 同 `MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`：清单不空。
    DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
        devices: Vec<DeviceBehindTheCurrentVersion>,
    },
}

impl From<CallerInputsDisagreeingWithTheDisk> for MountError {
    fn from(disagreeing: CallerInputsDisagreeingWithTheDisk) -> Self {
        match disagreeing {
            CallerInputsDisagreeingWithTheDisk::DeviceIdentitiesHandedInMoreThanOnce { repeated } => {
                MountError::DeviceIdentitiesHandedInMoreThanOnce { repeated }
            }
            CallerInputsDisagreeingWithTheDisk::ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields,
                disagreeing_device_table,
            } => MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields,
                disagreeing_device_table,
            },
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                devices,
            } => MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        }
    }
}

/// [`caller_inputs_agreeing_with_the_disk`] 核不过：交进来的不对，或系统配置择不出来（与可写挂载那一步同一个错）。
enum CallerInputsCheckRefused {
    Disagreeing(CallerInputsDisagreeingWithTheDisk),
    SystemConfigurationNotChosen(RecoveryFailure),
}

impl From<CallerInputsCheckRefused> for MountError {
    fn from(refused: CallerInputsCheckRefused) -> Self {
        match refused {
            CallerInputsCheckRefused::Disagreeing(disagreeing) => MountError::from(disagreeing),
            CallerInputsCheckRefused::SystemConfigurationNotChosen(failure) => {
                MountError::Recovery(failure)
            }
        }
    }
}

/// 盘表里重复的一个设备身份与它交了几次（`MountError::DeviceIdentitiesHandedInMoreThanOnce`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepeatedDeviceIdentity {
    pub device: DeviceIdentity,
    pub times_handed_in: usize,
}

/// `MakeFilesystemParameters` 里的一项，按系统配置字段表的次序（`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`）。
/// 封闭集合：`MakeFilesystemParameters` 与 `SystemImmutableSizes` 的每个字段各一个成员，比对时两边都整个解构，加字段漏比编译不过。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MakeFilesystemParameterField {
    FilesystemIdentifier,
    RegionDevices,
    PhysicalBlockSize,
    MinimumInputOutputBytes,
    FixedStructureSlotSpacing,
    JournalRingBytes,
    RootRingSlotsPerRegion,
}

/// 交进来的盘表与盘上择到的系统配置不一致的一项（`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`）：
/// 系统配置里只有设备数与本盘设备号这两样是按盘表写的（`transaction::PoolWriter::write_system_configuration_slot` 把设备数写成盘表的项数、
/// 本盘设备号写成盘表给这块盘的身份），别的不可变字段都从参数来；挂着之后收盘表的入口另核每块盘「可见」（第三个成员）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceTableDisagreement {
    /// 交进来的盘数不是系统配置里的设备数 `devs`（交进来之前已判过没有重复身份：盘数就是不同身份的个数）。
    DeviceCountDiffersFromTheSystemConfiguration {
        devices_handed_in: DeviceCount,
        device_count_in_the_system_configuration: DeviceCount,
    },
    /// 盘表把这块盘标成 `identity_handed_in`，它自证过的系统配置槽（本池的 fsid、整槽校验和过）里记的本盘设备号是
    /// `own_device_number_on_disk`。一块盘两槽记的号不同时各报一项；一份自证过的槽都没有的盘不在这一项里报
    /// （挂着之后的入口报成下一个成员；可写挂载在读阶段的见证读里就判不出、拒可写，C331（择根倒挂压过已确认的写） 取甲）。
    OwnDeviceNumberDiffersFromTheIdentityHandedIn {
        identity_handed_in: DeviceIdentity,
        own_device_number_on_disk: DeviceIdentity,
    },
    /// 盘表把这块盘标成 `identity_handed_in`，它两个系统配置槽里一份自证过的（整槽校验和过、fsid 与本池相同）都没有：空盘、别的池的盘、
    /// 两槽都读不出——这块盘不「可见」（D18（块里携带什么信息） 已定项 11「可写挂载的顺序」），同一判（[`self_verified_system_configuration_slots_of_the_pool`]）。
    /// 只由挂着之后收盘表的入口报（[`caller_inputs_agreeing_with_the_disk`]：抬 F、准入抬 F、一次准入里再推一串、正常卸载、管理员回退；
    /// 代码三方 m2-closeout-code-r1 Z3-A 乙，用户 2026-09-27 定）：不核这一判时，一块换上去的空盘让这些入口照样做成、把本池的写发到空盘上，
    /// 真盘从此缺这一串（那一轮攻方用例：卸载报成功、空盘收到 28 次写，之后真盘再挂可写被拒、池级 checker 红 4 条）。
    /// 可写挂载不报这一项：它在见证读里判不出、拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`，C331 取甲）。
    NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn { identity_handed_in: DeviceIdentity },
    /// 盘表把这块盘标成 `identity_handed_in`，这一刻探测到的 `physical_block_size`（[`BlockDevice::probe_physical_block_size`]）不是择到的那份
    /// 系统配置记着的 mkfs 时的 `physical_block_size`（偏移 317，根槽宽，D22（单元原子性怎么合成） 已定项 2「挂载时逐设备复算比对」）：
    /// D12（目标介质） 已定项 4 比对清单第一行「拒绝挂载（可写与只读都拒）」。这里罩的是可写挂载与挂着之后收盘表的入口；
    /// 只读挂载经读者问同一对值、报成 `mounted_read::MountReadOnlyFailure::DevicesWhosePhysicalBlockSizeDisagreesWithTheSystemConfiguration`。
    /// 每块盘各比一次、不等的各报一项；每块盘都等于同一个记录值，
    /// 两盘之间也就相等（D22（单元原子性怎么合成） 已定项 15「第一版退化成各盘互比」那一半由此落下）。只调探测、不读盘、不写盘。
    /// 会话发布用户改动的入口同一判报成 `mounted_session::UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration`。
    PhysicalBlockSizeProbedOnTheDeviceDiffersFromTheSystemConfiguration {
        identity_handed_in: DeviceIdentity,
        probed_physical_block_size: PhysicalBlockSizeInBytes,
        physical_block_size_in_the_system_configuration: PhysicalBlockSizeInBytes,
    },
}

/// 盘表把一块盘标成 `identity_handed_in`，它这一刻探测到的 `physical_block_size` 是 `probed_physical_block_size`，
/// 择到的那份系统配置记着的 mkfs 时的值是 `physical_block_size_in_the_system_configuration`，两者不等
/// （[`physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration`]）。可写挂载与挂着之后收盘表的入口
/// 报成 [`DeviceTableDisagreement::PhysicalBlockSizeProbedOnTheDeviceDiffersFromTheSystemConfiguration`]，会话发布用户改动报成
/// `mounted_session::UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration {
    pub identity_handed_in: DeviceIdentity,
    pub probed_physical_block_size: PhysicalBlockSizeInBytes,
    pub physical_block_size_in_the_system_configuration: PhysicalBlockSizeInBytes,
}

/// 盘表把一块盘标成 `identity_handed_in`，它一份自证过的系统配置槽（本池的 fsid、整槽校验和过）里记的本盘设备号是
/// `own_device_number_on_disk`（[`own_device_numbers_on_disk_differing_from_the_identities_handed_in`]）。可写挂载与挂着之后收盘表的入口
/// 报成 [`DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn`]，会话报成
/// `mounted_session::UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable`（代码三方 m2-closeout-code-r2「盘体对调的盘表」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn {
    pub identity_handed_in: DeviceIdentity,
    pub own_device_number_on_disk: DeviceIdentity,
}

/// 挂着之后收盘表的入口在「可见」之后逐盘核出的一块落后于现行那一版、又缺它的一样东西的盘（[`devices_behind_the_current_version`]）：
/// 它两槽里自证过的系统配置世代号最大那一份的 (实例代号, journal tail) 是 `newest_system_configuration_journal_tail`，小于调用方持着的
/// 现行那一版那次发布末条记录的 jsn（`current_version_journal_position`），而这块盘缺现行那一版的单元、或现行那一版的根写在这块盘上
/// 而那个根环槽里不是它（`lacking`）。与可写挂载取号之前逐盘核的落后支同一判（[`what_a_device_behind_the_version_lacks`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceBehindTheCurrentVersion {
    pub device: DeviceIdentity,
    pub newest_system_configuration_journal_tail: JournalSequenceNumber,
    pub current_version_journal_position: JournalSequenceNumber,
    pub lacking: VersionLackingOnADeviceBehindIt,
}

/// 一块落后于某一版的盘（最新那份自证系统配置的 (实例代号, journal tail) 小于那一版那次发布末条记录的 jsn）缺那一版的什么
/// （[`what_a_device_behind_the_version_lacks`]；C585（会话发布放进发布途中取下的旧快照），用户定改法 C）。按次序判：先看单元，单元都在再看根。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VersionLackingOnADeviceBehindIt {
    /// 那一版有单元在这块盘上它的落点读不出、或字节与验得过的那一份不同，落点被 tail 之后的记录点名或那个窗口取不到
    /// （[`units_of_the_version_missing_on_a_device_behind_it`]）：按角色与落点列出，次序同那一版的单元清单。
    UnitsMissing(Vec<(TransactionUnit, SlotNumber)>),
    /// 单元都在，而那一版的根写在这块盘上、那个根环槽读一次重读一次都不是它：一次发布途中（单元与记录已写、根槽 FUA 还没写）
    /// 取下的这块盘的旧快照就是这个样子。单元都在、根槽里也是它的（根落盘之后、轮换之前崩；只剩一槽自证、那一槽停在上一次轮换）不在这里。
    RootNotInItsRingSlot(VersionRootNotInItsRingSlot),
}

/// 那一版的根该在的根环槽（`root_ring::target_for_publish` 按它的 txg 算）与这块盘上那一槽两次读到的（读一次，不是它就重读一次，
/// D16（发布语义） 已定项 1「根槽这一次读坏」：知道住着根的槽重读一次仍坏才拒）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VersionRootNotInItsRingSlot {
    pub ring_slot: RootRingSlot,
    pub first_reading: RingSlotReadingOtherThanTheVersionRoot,
    pub reread: RingSlotReadingOtherThanTheVersionRoot,
}

/// 那一版的根该在的根环槽读出来不是它：读坏（读不出或自证不过），或自证得过而是另一条根（那一槽上一圈留下的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingSlotReadingOtherThanTheVersionRoot {
    Bad(BadRootRingSlotReading),
    AnotherRoot {
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    },
}

/// 那一版的根写没写进根环：落后支根槽那一半（[`what_a_device_behind_the_version_lacks`]）只核写进了的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RootOfTheVersionInTheRing {
    /// 写进了根环：挂着之后调用方持着的现行那一版（每次发布在根槽 FUA 返回之后才换它）；可写挂载所选那一版就是择到的那条根
    /// （没在它上面施加 journal 记录）。
    Written {
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
    },
    /// 没写进根环：可写挂载所选那一版是在择到的根上施加 journal 记录得来的（那次发布崩在根槽 FUA 之前），哪块盘的根环里都没有它。
    NotWritten,
}

/// 盘的块数：可写挂载准入拿交进来的不同设备身份数与 w 的下限比（`MountError::WritableDeviceCountBelowTheStripeWidthLowerBound`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceCount(pub usize);

/// w 的下限：一条条带至少落几块盘（D2（RAID 条带策略） 已定项 6「`w ≥ 2` 是硬下界（零冗余的条带不许发出）」）。
pub const STRIPE_WIDTH_LOWER_BOUND: DeviceCount = DeviceCount(2);

/// 一块不带所选那一版的盘，与它缺的是什么（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceWithoutTheSelectedVersion {
    pub device: DeviceIdentity,
    pub lacking: SelectedVersionLackingOnDevice,
}

/// 一块盘不带所选那一版，缺在哪。按次序判：先看系统配置，再看单元，单元都在再看根。
///
/// 系统配置不落后、只是缺所选那一版几个单元的盘（单份坏）**不在这里**：它自证过的系统配置跟得上这一版，不是空盘、
/// 也没有停在旧状态，条款没把它归进作废的那两类；它坏的那几份，读的时候按位置条目改读另一份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedVersionLackingOnDevice {
    /// 停在旧状态：这块盘自证过的系统配置里世代号最大的那一份，(实例代号, journal tail) 落后于所选那一版那次发布的末条记录的 jsn
    /// （每块盘在每次发布之后轮换的系统配置写的就是这一对，`transaction::CommitStep::RotateSystemConfigurationSlots`），
    /// 而且所选那一版有单元在这块盘上它的落点读不出、或者字节与验得过的那一份不同，落点被 tail 之后的记录点名或那个窗口取不到
    /// （`units_missing`，按角色与落点列出，次序同所选那一版的单元清单；[`units_of_the_version_missing_on_a_device_behind_it`]）。只落后、单元都在的不算：发布的根落盘之后、系统配置轮换之前崩了，
    /// 或取号失败回卷时写回了取号那一刻的见证值（`transaction::acquire_instance`，C554 乙-配置续），盘都是这个样子。
    BehindTheSelectedVersionAndMissingItsUnits {
        newest_system_configuration_journal_tail: JournalSequenceNumber,
        units_missing: Vec<(TransactionUnit, SlotNumber)>,
    },
    /// 停在旧状态（同上一个成员的前一半），单元都在，而所选那一版的根写进了根环、住在这块盘上，那个根环槽读一次重读一次都不是它
    /// （与挂着之后收盘表的入口同一判，[`VersionLackingOnADeviceBehindIt::RootNotInItsRingSlot`]）。所选那一版是施加 journal 记录得来的
    /// （根环里没有它）时不核这一半。所选那一版就是择到的那条根时，这一槽读阶段刚读得出它：只有两次读都坏才走得到这里。
    BehindTheSelectedVersionAndItsRootNotInItsRingSlot {
        newest_system_configuration_journal_tail: JournalSequenceNumber,
        root_not_in_its_ring_slot: VersionRootNotInItsRingSlot,
    },
}

/// 自己开写入口、接连推的一串发布里有一次没做成：可写挂载取号之后那一串——写行一次、暖机推到本实例的根覆盖每块盘
/// （D16（发布语义） 已定项 8 戊）——连同之后的挂载续做；抬 F 那一串空发布——推到每块盘上都有一条带新 F 的根（D16（发布语义） 已定项 1）。
/// 可写挂载取号之前预演续做拒（C841，或预演没走到结局而读盘时有过读错）也交它：那时这一串还没开写入口、一个写都没发，三份写账都是空的。
/// 写入口是这一串自己开的、随错一起丢掉，所以它交得出的写账都在这里：已经落盘的那几次发布各自的写，
/// 与失败那一次落盘阶段已记的写（增补 2 收口表第 58 行：不交出来，设备一层数到的写与程序交得出的账对不上）。
/// 不是发布的写不在其内：可写挂载取号两轮那几次系统配置槽写（取号的回卷另由 `AcquisitionFailed` 报）。
#[derive(Debug)]
pub struct PublishSequenceFailed {
    pub cause: PublishError,
    /// 这一串里不属于任何一次发布、已经发出去的写：抬 F 那一串是第一次发布之前先把新 F 写进每块盘系统配置那一步
    /// （每块盘一次系统配置槽写，D16（发布语义） 已定项 1「抬 F 那一串」）；可写挂载那一串在写行、暖机那几次失败时是空账（取号两轮那几次写不在这里，见上），
    /// 在挂载续做里失败时是这次挂载推成的每一串抬 F（准入推的与续做推的）先写系统配置那一步逐种相加——那几串的空发布在
    /// `writes_of_persisted_publishes` 里，先写系统配置那一步不是发布，只能放在这里。于是这一串交给设备的写 = 这一格 + 已落盘各份 + 失败账。
    pub writes_before_the_first_publish: WritesByStructureKind,
    /// 失败之前这一串里已经落盘的发布各自的写，按先后（可写挂载写行那次在最前）；第一次就失败时是空的。
    pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
    /// 这一串的写入口的失败账（`PoolWriter::writes_of_failed_publishes` 原样）：落盘阶段失败的那一次一份；
    /// 落盘之前就被拒的（准入、释放判定、落点）一个写都没发，不记，这里是空的。
    pub writes_of_failed_publishes: Vec<WritesByStructureKind>,
}

/// 回退目标不在回退候选集里的是哪一条（D23（journal 的角色与格式） 已定项 14「候选集」：根环里按现行那一版的实例表判仍然有效
/// ∧ txg ≥ F_生效（D16（发布语义） 已定项 1「回退候选集」；一圈窗口那一格用户 2026-10-04 定 RGall 拿掉）∧ 带文件的根）。[`roll_back_by_a_forward_publish`] 按成员的次序判，一条目标同时中几条时只报最先判到的那一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RollbackCandidateExclusion {
    /// 根环里没有那个 (实例, txg) 的可读根槽：候选集是根环里的根。
    NotInRing,
    /// txg 低于生效的回退下界 F（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效）。
    BelowEffectiveFloor,
    /// 现行那一版的实例表里有那个实例的行 (i, Ti, Wi) 且目标的 txg > Ti：被崩溃恢复抛弃的时间线上的根
    /// （「按现行那一版的实例表判仍然有效」反过来，`recovery::root_is_abandoned_by_the_instance_table`）。
    /// 原来还有一格「txg 落在现行那一版的一圈窗口外」：回退候选集是原来同取一圈窗口的六处之一，用户 2026-10-04 定 RGall 一起退回整个环
    /// （`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1），那一格拿掉——回收门槛同时退回整个环，窗口外的有效根引用的单元不再被回收。
    OnAbandonedTimeline,
    /// 目标那一版树表 0 条（还没发布过文件版本）：候选集只收带文件的根。
    VersionWithoutFile,
}

/// 一条根的 (实例代号, txg)：回退的目标（管理员从回退候选集里选的那条根，D23（journal 的角色与格式） 已定项 14），
/// 与各个错误成员里报的那一版。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollbackTarget {
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
}

/// 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：关掉影子账，只被被抛弃根（崩溃恢复抛弃的时间线）引用的槽照常可分配——
/// C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 那两格必红靠它强制进入；产品路径恒 `On`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowLedger {
    On,
    Off,
}

impl ShadowLedger {
    /// 这次挂载走的是哪一臂，运行时报得出（`.claude/rules/fs-design.md` 五条硬要求第 4 条：
    /// **分支必须可观测**）。⚠️ 光看 `MountOutput::isolated_slots_per_device` 分不开两臂：
    /// `Off` 恒 0，而 `On` 在「没有被抛弃根、或它们引用的槽都还被候选集引用着」时也是 0。
    #[must_use]
    pub const fn branch_name(self) -> &'static str {
        match self {
            ShadowLedger::On => "shadow_ledger=on",
            ShadowLedger::Off => "shadow_ledger=off",
        }
    }
}

impl From<RecoveryFailure> for MountError {
    fn from(failure: RecoveryFailure) -> Self {
        MountError::Recovery(failure)
    }
}

/// 一次可写挂载认下的池：盘上择到的那份系统配置里的 mkfs 参数（系统不可变配置里的 fsid、根环逐区域设备身份、几何），与挂载时交进来、
/// 核过的盘表的设备身份（按交进来的次序）。挂着的会话（`mounted_session::MountedSession`）只用它：写入口按这份参数建、不收调用方的参数，
/// 交进来的盘表与这份逐项比。字段私有，只由可写挂载与 [`ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`]
/// 读盘造出：调用方拼不出一份与盘上不同的参数。
/// 另带这次挂载给的后台计算设置与它挂着时的切换（`background_compute::BackgroundComputeOfThisMount`，D26（后台整理与放置回收） 已定项 13
/// 第 1–4 条）：它是挂载参数、不是盘上的参数，只住内存；可写挂载按挂载给的那一组建（[`mount_writable_with_background_compute_settings`]，
/// 别的入口都是产品默认 `cpu`），读盘造的那一份是产品默认。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParametersAndDeviceTableOfTheMount {
    parameters_on_disk: MakeFilesystemParameters,
    device_identities_in_table_order: Vec<DeviceIdentity>,
    background_compute_of_this_mount: BackgroundComputeOfThisMount,
}

impl ParametersAndDeviceTableOfTheMount {
    /// 没经过可写挂载的会话（mkfs 同一个进程里接着发布的那一条）用：照可写挂载同一套核读这一刻的盘——盘表里有身份交了不止一次就拒，
    /// 择系统配置，盘数与 `devs`、每块盘的本盘设备号与盘表给的身份有一项不同就拒——交回盘上那份参数与这份盘表。只读盘，不写盘。
    ///
    /// # Errors
    /// `DeviceIdentitiesHandedInMoreThanOnce`（不读盘）；系统配置择不出来（`Recovery`）；
    /// `CallerParametersDisagreeWithTheSelectedSystemConfiguration`（只有 `disagreeing_device_table` 不空）。
    #[allow(
        clippy::ptr_arg,
        reason = "choose_system_configuration 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
    )]
    pub fn of_a_pool_on_disk<Device: BlockDevice>(
        devices: &Vec<(DeviceIdentity, Device)>,
    ) -> Result<Self, MountError> {
        refuse_device_identities_handed_in_more_than_once(devices)?;
        let system_configuration = choose_system_configuration(devices)?;
        let disagreeing_device_table =
            device_table_disagreeing_with(devices, &system_configuration);
        if !disagreeing_device_table.is_empty() {
            return Err(
                MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
                    disagreeing_fields: Vec::new(),
                    disagreeing_device_table,
                },
            );
        }
        Ok(Self::of_the_checked_pool(
            parameters_of_the_system_configuration(&system_configuration),
            devices,
        ))
    }

    /// 核过之后（重复身份、参数、盘数与 `devs`、本盘设备号）的那份参数与盘表。
    fn of_the_checked_pool<Device>(
        parameters_on_disk: MakeFilesystemParameters,
        devices: &[(DeviceIdentity, Device)],
    ) -> Self {
        Self {
            parameters_on_disk,
            device_identities_in_table_order: devices
                .iter()
                .map(|(identity, _)| *identity)
                .collect(),
            background_compute_of_this_mount: BackgroundComputeOfThisMount::of_a_mount(
                AdmittedBackgroundComputeSettings::PRODUCT,
            ),
        }
    }

    /// 盘上择到的那份系统配置里的 mkfs 参数：挂着之后的写入口按它建。
    #[must_use]
    pub fn parameters_on_disk(&self) -> &MakeFilesystemParameters {
        &self.parameters_on_disk
    }

    /// 这次挂载的后台计算设置：生效的那一组与待生效的切换。
    #[must_use]
    pub fn background_compute_of_this_mount(&self) -> &BackgroundComputeOfThisMount {
        &self.background_compute_of_this_mount
    }

    /// 挂着的会话请求切换时改它（`mounted_session::MountedSession::switch_background_compute_from_the_next_batch`）。
    pub(crate) fn background_compute_of_this_mount_mut(
        &mut self,
    ) -> &mut BackgroundComputeOfThisMount {
        &mut self.background_compute_of_this_mount
    }

    /// 整理一趟同时要读盘上的参数、每批开头改后台计算的状态：两样一次借出。
    pub(crate) fn parameters_on_disk_and_the_background_compute_of_this_mount_mut(
        &mut self,
    ) -> (&MakeFilesystemParameters, &mut BackgroundComputeOfThisMount) {
        (
            &self.parameters_on_disk,
            &mut self.background_compute_of_this_mount,
        )
    }

    /// 挂载时核过的盘表的设备身份，按交进来的次序。
    #[must_use]
    pub fn device_identities_in_table_order(&self) -> &[DeviceIdentity] {
        &self.device_identities_in_table_order
    }

    /// 交进来的盘表的设备身份（按次序）与挂载时核过的那一份逐项相同：多一块、少一块、换了次序、同一个身份交两次都不同。只比身份，不读盘。
    #[must_use]
    pub fn is_the_device_table_of_the_mount<Device>(
        &self,
        devices: &[(DeviceIdentity, Device)],
    ) -> bool {
        devices
            .iter()
            .map(|(identity, _)| *identity)
            .eq(self.device_identities_in_table_order.iter().copied())
    }
}

/// 一次可写挂载写出的东西。
#[derive(Debug)]
pub struct MountOutput {
    pub instance: InstanceGeneration,
    /// 恢复择到的根（施加前缀之前）。
    pub chosen_root: RootRecord,
    /// 施加前缀之后的根：写行与照抄都以它为准。
    pub effective_root: RootRecord,
    pub journal: JournalScanReport,
    /// 这次挂载写进实例表的行（上一个实例那一行，中间实例各一行）。
    pub rows_written: Vec<InstanceRow>,
    /// 写行那次发布（本实例的第一次发布）；所选根的树表 0 条、要写的行为空（上一个实例是 0）时是一次零单元发布。
    pub row_publish: PoolVersion,
    /// 之后的暖机空发布，直到本实例的根覆盖每块盘。
    pub warm_up_publishes: Vec<PoolVersion>,
    /// 这次挂载走的影子账那一臂：挂着的会话抬 F 时照它重算影子账（`mounted_session`）。
    pub shadow_ledger: ShadowLedger,
    /// 这次挂载走的影子账分支名（`ShadowLedger::branch_name`）：两臂在这里永远不同，
    /// 而 `isolated_slots_per_device` 两臂都可能是 0，光看它分不开（五条硬要求第 4 条）。
    pub shadow_ledger_branch: &'static str,
    /// 分配器隔离的槽数，逐盘（D28（挂载期承诺量） 已定项 1 第九项，只住内存）：只被被抛弃根引用的槽，加水位之上没施加的 journal 记录点名、
    /// 不被候选集里的根与现行账引用的槽（C694 的 C）；仍被候选集里的根引用的不在其内（对用户 2026-09-16 定的窄读法措辞的收严，
    /// 见 `isolate_slots_referenced_only_by_abandoned_roots`；预想）。
    pub isolated_slots_per_device: Vec<(DeviceIdentity, u64)>,
    /// 被抛弃根里树表或分配记录树读不出、解不开的条数：这样的根从中央映射现算它引用的落点照样隔离（C393）；映射也现算不成、重读一次仍不成时，
    /// 现行账里有那条根之后的记录罩住它的树表槽（与映射根槽）的照样计数、不隔离（C694 的 B），没有的拒可写（`isolate_slots_referenced_only_by_abandoned_roots`）。
    pub abandoned_roots_unreadable: u64,
    /// 这次挂载的空间准入怎么判的、推了几串抬 F 的空发布（[`MountSpaceAdmission`]）。推的那几串写的根不在 `warm_up_publishes` 里：
    /// 它们接在暖机之后，`Mounted::current` 是最后落盘的那一次。
    pub space_admission: MountSpaceAdmission,
    /// 这次挂载认下的池（盘上那份参数、核过的盘表）：挂着的会话只用它。
    pub parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
    /// 这次挂载重读过哪几样（C554 乙）：读阶段在哪一遍判完、读阶段择到的那条根的实例表在哪一遍读出来、重建分配器读根环在哪一遍读出有效根。
    pub rereads: RereadsOfThisMount,
    /// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）发成的每一步，按先后：接在写行、暖机与推的抬 F 之后，走到了空时最后一个就是
    /// `Mounted::current`（两处指着同一份，不拷一整版；写行那一版与暖机各版里只有最后发成的那一版与 `Mounted::current` 共享，前面几版各是一份）；
    /// 所选那一版的 `deleted_inodes` 是空的（常态）时空。
    pub deletion_queue_continuation: Vec<Arc<TransactionOutput>>,
    /// 挂载续做里为空间推的那几串抬 F，按先后（与 `deletion_queue_continuation` 的各步按 txg 交错）：走空里一次删除类发布被拒是空间不够
    /// （不判式子那一判、落得下那一判照判，D28（挂载期承诺量） 已定项 1）时推一串、回收之后接着走（用户 2026-10-03 定案
    /// 「挂载续做被空间拒：同样推抬 F 再试」，与会话那一处同一个做法）。一次都没推的是空的。
    pub floor_raises_pushed_by_the_continuation: Vec<RaisedFloor>,
    /// 每一串是因为哪一次被拒才推的（与 `floor_raises_pushed_by_the_continuation` 一一对应）：`SpaceAdmissionRefused` 或 `PlacementRefused`。
    pub refusals_that_pushed_the_continuation_floor_raises: Vec<PublishError>,
    /// 挂载续做停在哪儿（[`DeletionQueueContinuationOutcome`]）：走到了空，或推满仍放不下、队列留着而挂载照样做成可写。
    pub deletion_queue_continuation_outcome: DeletionQueueContinuationOutcome,
    /// 交回之前那一判实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」）的结局：不扫、交给后台，
    /// 或这次挂载里判、抹、发到了哪儿（[`crate::instance_table_sweep::sweep_the_instance_table_at_the_end_of_the_mount`]）。
    /// 发成了清扫发布时 `Mounted::current` 是清扫发布那一版。
    pub instance_table_sweep: InstanceTableSweepAtTheEndOfTheMount,
    /// 盘上那一条未完成的整理意图的续做（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1；D8（核心索引结构） 已定项 5
    /// 第 2 条），连同这次挂载会话这一侧的整理状态（会话接着用，`mounted_session::MountedSession::of_the_mount`）。
    pub compaction_continuation: CompactionContinuationAtTheMount,
    /// 取号写两轮顺带补写的盘（丙'：两槽一份本池自证过的系统配置都没有、过了补写闸的那几块），按盘表次序；没有补写是空的。
    /// 补写这一支走没走从这里看得出（`.claude/rules/fs-design.md`「五条硬要求」第 4 条；C874（补写那一支进不了崩溃测试、运行时看不出），
    /// 判决 `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R4）；没补写的原因在拒因的载荷里
    /// （[`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`]）。
    pub devices_rewritten_by_the_acquisition: Vec<DeviceIdentity>,
}

/// 可写挂载续做整理意图（[`MountOutput::compaction_continuation`]）：次序在删除续做之后（判决 G8「删除续做 → 为用户请求推的抬 F → GC 批」，
/// 挂载里没有用户请求）、实例表清扫之前；只推盘上那一条，不签新的。
#[derive(Debug)]
pub struct CompactionContinuationAtTheMount {
    /// 推了的那一趟（停在哪、发了哪几版：发了的那几版接在删除续做之后，清扫没发布时最后一个就是 `Mounted::current`）。
    /// 所选那一版的 logged_ops 没有活意图（常态）、或删除续做没走空（整理排在它后面）时 `None`，一个写都不发。
    pub run: Option<CompactionRun>,
    /// 这次挂载会话这一侧的整理状态（不合格段表、常驻隔离集、`B_gc` 数的发布数、观测）：续做时建的，会话接着用。
    pub state_of_this_mount: CompactionOfThisMount,
}

/// 挂载续做停在哪儿。成员按调用方要做的决定分：照常写；队列还留着——之后会话里每一次用户改动先走空、走不动报空间不够
/// （`mounted_session::UserChangeRefused::NoSpaceAfterRaisingTheFloor`），正常卸载照常做成：靠的是卸载那一串的预演算上新 F 回收出来的槽
/// 里当场放开的那一截（[`unmount`]，用户 2026-10-03 定甲 GA），之后再挂可写、续做接着走。
#[derive(Debug)]
pub enum DeletionQueueContinuationOutcome {
    /// 所选那一版的 `deleted_inodes` 是空的（常态），或续做把它走到了空。
    WalkedToEmpty,
    /// 续做里一次删除类发布被拒是空间不够、推满仍放不下（用户 2026-10-03 定「照 D16 做成可写」，D16（发布语义） 已定项 1「准入」那一行
    /// 「挂载处推满仍不够时挂载照样做成可写」）：挂载照样交回可写，`Mounted::current` 是续做最后落盘的那一版（删除那一步或推的空发布），
    /// 它的 `deleted_inodes` 不空。`last_refusal` 是最后那一次被拒（`SpaceAdmissionRefused` 或 `PlacementRefused`），`stop` 是不再推的原因。
    StillShortAfterTheFloorRaises {
        last_refusal: PublishError,
        stop: FloorRaiseStop,
    },
}

/// 一次可写挂载的空间准入怎么判的（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；D16（发布语义） 已定项 1「准入」那一行：
/// 可写挂载准入不够时写行那次发布照走切换预留，写完行之后推抬 F 的空发布、再判）。成员按调用方要做的决定分：照常写；
/// 推过、够了（照常写，推的根要记进账）；推满仍不够（之后的发布会报空间不够，要腾空间就正常卸载再挂）；没判（只供测试的开关）。
#[derive(Debug)]
pub enum MountSpaceAdmission {
    /// 取号之前判过：实例切换的预留拿得到，一次都没推。
    AdmittedBeforeAcquisition,
    /// 取号之前不够（`refusal_before_acquisition`）；写行与暖机之后推了 `floor_raises` 那几串抬 F 的空发布（按推的先后），再判够了。
    /// `floor_raises` 为空：写行与暖机之后再判就够了（这次挂载自己的根转过根环、按可再分配谓词回收了），一次都没推。
    AdmittedAfterTheFloorRaises {
        refusal_before_acquisition: AdmissionRefusedOnSomeDevices,
        floor_raises: Vec<RaisedFloor>,
    },
    /// 推满仍不够（C565（挂载处推满仍不够怎么收尾没定），实现员提的收尾、交主 agent 定）：挂载照样做成——实例已取、行已写、
    /// 暖机与推的那几串都已落盘；`last_refusal` 是最后一次判的结局，`stop` 是不再推的原因。之后的发布照发布路径的准入判，
    /// 不够就报空间不够；正常卸载（抬 F 到现行那一版、不判上限）照常做成，它的预演算上新 F 回收出来、当场放开的那一截（[`unmount`]，用户 2026-10-03 定甲 GA）。
    StillShortAfterTheFloorRaises {
        refusal_before_acquisition: AdmissionRefusedOnSomeDevices,
        floor_raises: Vec<RaisedFloor>,
        last_refusal: AdmissionRefusedOnSomeDevices,
        stop: FloorRaiseStop,
    },
    /// 只供测试的开关关掉准入（`SpaceAdmission::SkippedByTheTestOnlySwitch`）：没判，一次都没推。
    NotJudgedByTheTestOnlySwitch,
}

impl MountSpaceAdmission {
    /// 这次挂载在写行之后推的那几串抬 F 的空发布，按推的先后；没推过的是空的。
    #[must_use]
    pub fn floor_raises(&self) -> &[RaisedFloor] {
        match self {
            MountSpaceAdmission::AdmittedAfterTheFloorRaises { floor_raises, .. }
            | MountSpaceAdmission::StillShortAfterTheFloorRaises { floor_raises, .. } => {
                floor_raises
            }
            MountSpaceAdmission::AdmittedBeforeAcquisition
            | MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => &[],
        }
    }
}

/// 可写挂载的结果：写出的东西、重建并推进过的分配器、接下来的发布要接在后面的那一版。
#[derive(Debug)]
pub struct Mounted {
    pub output: MountOutput,
    pub allocator: PoolAllocator,
    pub current: PoolVersion,
}

/// 新实例写行那一行的来历：所选根那个实例那一行；中间实例的行由 `establish_instance` 补 (i, 0, 0)。
struct PreviousInstanceRow {
    instance: InstanceGeneration,
    selected_root_txg: CheckpointTxg,
    applied_transaction_high_water: u64,
}

/// 恢复之后建立新实例要带的东西。
struct InstanceStart {
    chosen_root: RootRecord,
    effective_root: RootRecord,
    journal: JournalScanReport,
    previous: PreviousVersion,
    previous_row: PreviousInstanceRow,
    first_txg: CheckpointTxg,
    next_counter: u64,
    /// 本实例第一次发布（写行那一次）要带的树 ID 水位：根环里全部自证过的根与环里全部自证通过的记录各自带的水位取 max
    /// （D8（核心索引结构） 已定项 8 ②；`recovery::highest_tree_identifier_watermark_in_the_ring`）。它可以高于
    /// 所选那一版自己带的——被崩溃恢复抛弃的时间线上的根仍承载它们那一代发过的号。
    tree_identifier_watermark_of_the_ring: u64,
    /// 新实例的根带的回退下界 = 恢复后生效的 F（`recovery::effective_rollback_floor`）：与重建分配器时回收用的同一个值，
    /// 一条根带的 F 与它的记账行才说同一件事。
    effective_floor: CheckpointTxg,
    abandoned_roots_unreadable: u64,
    /// 这次挂载走的影子账那一臂：产品路径恒 `On`，只供测试的入口 [`mount_writable_with_test_only_switches`] 由调用方给。
    /// 带着它只为一件事——让 `MountOutput::shadow_ledger_branch` 报得出走了哪一条（五条硬要求第 4 条）。
    shadow_ledger: ShadowLedger,
    /// 恢复择到的系统配置：取号之前逐盘核（`devices_without_the_selected_version`）读槽距与 fsid 从它取。
    system_configuration: crate::system_configuration::SystemConfiguration,
    /// 判不判空间准入（只供测试的开关，`admission::SpaceAdmission`）：产品路径恒判。
    space_admission: SpaceAdmission,
    /// 所选那一版（`effective_root` 那一版）那次发布的末条记录的 jsn（`journal_position_of_the_selected_version`）：
    /// 取号之前逐盘核「这块盘落没落后于这一版」拿它比。
    selected_version_journal_position: JournalSequenceNumber,
    /// 这次挂载重读过哪几样（C554 乙），原样交进 `MountOutput::rereads`。
    rereads: RereadsOfThisMount,
    /// 交回之前那一判实例表清扫的开关、阈值（产品路径 `InstanceTableSweepConfiguration::PRODUCT`）。
    instance_table_sweep: InstanceTableSweepConfiguration,
    /// 这次挂载给的后台计算设置（校验过；除 [`mount_writable_with_background_compute_settings`] 之外的入口都是产品默认 `cpu`）：
    /// 挂载续做整理意图那几批照它记账，交回的会话从它起。
    background_compute: AdmittedBackgroundComputeSettings,
    /// 建上一版按位置条目读单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：交回之前记进这次挂载的状态。
    unit_reads_with_one_copy_failing_while_building_on_the_version: u64,
}

/// 重建分配器之后要带回去的东西。
struct RebuiltAllocator {
    allocator: PoolAllocator,
    effective_floor: CheckpointTxg,
    abandoned_roots_unreadable: u64,
    instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead,
    root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead,
}

/// 影子账算完交回的：读不出账的被抛弃根条数，与每条读得出账的被抛弃根引用的全部落点（按根的 (实例, txg) 认；
/// 分配器那张根环表拿它判「这条根离开根环时清哪几个槽」「挂载内回收出来的槽还有没有被抛弃根引用」，`allocator::RootRingOccupant`）。
struct ShadowLedgerComputed {
    abandoned_roots_unreadable: u64,
    placements_referenced_by_abandoned_root:
        BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
}

/// 重建分配器读根环那一遍（或两遍逐槽并起来）的读数，连同窗口锚、读了几遍、两遍里只有一遍自证过的槽（只读了一遍时是空集）。
struct RootRingReadForTheRebuild {
    ring_slot_readings: Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)>,
    window_anchor: CheckpointTxg,
    read: RootRingOfTheRebuiltAllocatorRead,
    ring_slots_self_verified_on_only_one_of_the_two_passes: BTreeSet<RootRingSlot>,
}

/// 重建分配器那一份根环读数分好的几类：读出来的根与它的槽、挂载时读坏的槽与读回全零的槽各能住哪几个 txg。
struct RootRingReadingsSortedForTheRebuild {
    roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)>,
    roots: Vec<RootRecord>,
    ring_slots_bad_at_mount: Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)>,
    ring_slots_reading_back_all_zero_at_mount: Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)>,
}

/// 准入抬 F 的上限算两遍的结果（[`admission_ceilings_not_passing_ring_slots_bad_since_the_mount`]）：判上限卡没卡在挂载时读坏或读回全零的槽上
/// （D16（发布语义） 已定项 1「抬 F 的上限」那一行：并进它们之后上限不高于现行 F、不并它们时上限高于现行 F）要两个数都有。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AdmissionCeilings {
    /// 只照这一遍读得出（或顶上）的根算的上限，不并挂载时读坏、读回全零的槽。
    from_the_roots_read: CheckpointTxg,
    /// 再并上挂载时读坏、读回全零、之后这个进程没写过而这一遍又读坏的槽（能住的最大 txg 不低于今天的 F 的那几个）：准入抬 F 抬到的是它。
    not_passing_the_ring_slots_bad_since_the_mount: CheckpointTxg,
}

/// 抬 F 那一串做完之后两个入口共有的东西；准入那一格另带它判过的上限。
struct FloorRaisedThroughTheSequence {
    ceiling_judged_by_the_admission: Option<CheckpointTxg>,
    system_configuration_writes_before_the_first_publish: WritesByStructureKind,
    publishes: Vec<TransactionOutput>,
    reclaimed: Vec<Placement>,
    abandoned_roots_unreadable: u64,
}

/// 写行那次发布的身份字段：新实例的第一次发布。
struct RowPublishIdentity {
    txg: CheckpointTxg,
    counter: u64,
    instance: InstanceGeneration,
    rollback_floor: CheckpointTxg,
    /// 根环里全部根记录的树 ID 水位取 max（`InstanceStart::tree_identifier_watermark_of_the_ring`）：写行那次发布一个号都不发，
    /// 新根的水位就是它（D8（核心索引结构） 已定项 8 ②）。
    tree_identifier_watermark: u64,
}

/// 取号之后那一串里第几次发布（从 0 数：0 是写行那次，之后是暖机）在分配器的拷贝上预演时报的错。
struct DryRunRefusal {
    publish_index: usize,
    cause: PublishError,
}

/// 取号之前整串预演（[`dry_run_of_the_publishes_after_acquisition`]）交回的：每一次取到的落点，预演到最后的那一版
/// （写行与暖机都预演过之后的现行那一版：挂载续做在真发时从写行、暖机之后的那一版接着走），与预演到最后的那一份分配器拷贝
/// （取号之前预演整个续做接着它取落点）。
struct PublishesRehearsedBeforeAcquisition {
    placements_taken_by_every_publish: Vec<PlacementsTakenByOnePublish>,
    version_after_the_last_publish: PoolVersion,
    allocator_after_the_last_publish: PoolAllocator,
}

/// 所选那一版在取号之前逐盘核里要用的几样（[`devices_without_the_selected_version`]）。
#[derive(Clone, Copy)]
struct SelectedVersionCheckedOnEachDevice<'version> {
    /// 所选那一版的根（择到的根施加过 journal 之后那一条）：按它的 (实例代号, checkpoint_txg) 认那一版的末条记录（乙'）、
    /// 比根环与 journal 环里有没有更新的（丙' 闸三），按它的实例代号比单元区里的写序（丙' 闸四）。
    root: &'version RootRecord,
    /// 那一版那次发布末条记录的 jsn（[`journal_position_of_the_selected_version`]）。
    journal_position: JournalSequenceNumber,
    units: &'version [PublishedUnit],
    root_in_the_ring: RootOfTheVersionInTheRing,
}

/// 取号之前逐盘核的结局（[`devices_without_the_selected_version`]）。
struct DevicesCheckedBeforeTheAcquisition {
    /// 不带所选那一版的盘与各自缺的；非空就拒可写。
    devices_without: Vec<DeviceWithoutTheSelectedVersion>,
    /// 两槽一份本池自证过的系统配置都没有、过了补写闸的盘（丙'），按交进来的次序：取号写两轮顺带补写它们。
    devices_rewritten_by_the_acquisition: Vec<DeviceIdentity>,
    /// 逐盘核读过的每块盘每个系统配置槽在读的那一刻的结论，按读的次序（改法丙：续做预演判两本账时剔掉落在已读坏、已报读错的槽上的那几条）。
    system_configuration_slots_as_checked: Vec<SystemConfigurationSlotAsChecked>,
}

/// 逐盘核读过的一个系统配置槽：哪块盘、从哪个偏移起读多长（与只进内存那一趟取号读它、记进两本账的那一条同一个落点）、读的那一刻的结论。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SystemConfigurationSlotAsChecked {
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length_in_bytes: u64,
    conclusion: SystemConfigurationSlotConclusionWhenChecked,
}

/// 读一块盘两个系统配置槽时套在读者外面的那一层：读与报读坏都原样交给里面那一份，另把每一读的落点与结论记下来——
/// 读回 `None` 记设备报读错，读得出记没报自证不过，读的一方接着报这一读自证不过就把那一条改记读坏。
/// 只包 `recovery::system_configuration_slots_of_one_device_read_once` 那一次调用：记下的每一读都是系统配置槽。
/// 两处套它：取号之前的逐盘核（改法丙，[`devices_without_the_selected_version`]），与读阶段的见证读（乙入判不出，[`newer_publish_witness`]）。
struct SystemConfigurationSlotReadsRecordedByTheCheck<'reader, Reader: PoolReader + ?Sized> {
    reader: &'reader Reader,
    slots_read: RefCell<Vec<SystemConfigurationSlotAsChecked>>,
}

/// 调用方交进来的参数与盘表照可写挂载那一套核过之后（[`caller_inputs_agreeing_with_the_disk`]）：盘上择到的那份系统配置，
/// 与照它建的参数——写入口只用这一份，不用调用方的。
struct CallerInputsAgreeingWithTheDisk {
    system_configuration: crate::system_configuration::SystemConfiguration,
    parameters_of_the_pool: MakeFilesystemParameters,
}

/// 挂载续做交回的东西（[`continue_the_deletion_queue_until_it_is_empty`]）：发成的每一步与为空间推的那几串抬 F，两样按 txg 交错；
/// 走到哪儿停（走到了空，或推满仍放不下、队列留着）。
struct DeletionQueueContinuation {
    /// 最后一步与挂载交回的现行那一版指着同一份（`transaction::DeletionQueueWalk::steps_published` 那样共享）。
    steps_published: Vec<Arc<TransactionOutput>>,
    floor_raises: Vec<RaisedFloor>,
    refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    outcome: DeletionQueueContinuationOutcome,
}

/// 取号之后、续做之前那一串要的、取号之前定下来的那几样（`establish_instance` 算出来）：要取的号、写行那次重写的实例表、
/// 暖机推哪几次、取号之前在分配器拷贝上预演取到的落点（真发之后拿它比）、取号之前那一判的结局。
/// 真发（[`acquire_then_publish_until_the_continuation`]）与取号之前在只进内存的盘上预演续做
/// （[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]）读同一份。
#[derive(Clone)]
struct PublishesPlannedAfterAcquisition {
    instance_to_acquire: InstanceGeneration,
    instance_table_rewrite: InstanceTableRewrite,
    warm_up_publishes_planned: Vec<CheckpointTxg>,
    placements_planned: Option<Vec<PlacementsTakenByOnePublish>>,
    admission_before_acquisition: WritableMountAdmissionBeforeAcquisition,
    /// 取号写两轮顺带补写的盘（丙'）：两槽一份本池自证过的系统配置都没有、过了补写闸的那几块（[`devices_without_the_selected_version`]）。
    /// 取号之前的逐盘核排在续做预演之前判，预演与真做都按它交回的这一份走。
    devices_rewritten_by_the_acquisition: Vec<DeviceIdentity>,
}

/// 取号之后、续做之前那一串发成的（[`acquire_then_publish_until_the_continuation`]）：取到的号、写行那次、暖机每次、
/// 写行之后那一判（连同推的抬 F）、现行那一版与发完之后的分配器。
struct PublishedBetweenAcquisitionAndTheContinuation {
    instance: InstanceGeneration,
    row_publish: PoolVersion,
    warm_up_publishes: Vec<PoolVersion>,
    space_admission: MountSpaceAdmission,
    current: PoolVersion,
    allocator: PoolAllocator,
}

/// 读阶段一遍读出来的：所选根与择根那一遍读出、自证过的全部根，环里全部自证过的记录、重放之后的所选那一版与判据 N-配置 的读数。
struct ReadStage {
    chosen_root: RootRecord,
    /// 择根那一遍读出、自证过的全部根（根环次序，[`choose_root_keeping_every_self_verified_root_read`]），所选根在其中。
    self_verified_roots_read: Vec<RootRecord>,
    records: BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    journal: JournalScanReport,
    effective_root: RootRecord,
    against_the_witness: SelectedVersionAgainstTheWitness,
    /// 水位之上那几次缺记录、读得出的记录点名的单元又验不过的发布（F1 乙，[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]）。
    publishes_missing_records_whose_named_units_do_not_verify: Vec<RollbackTarget>,
}

/// 经真发布搭的池（mkfs、取号、暖机、第一个文件，之后同一个进程里覆盖写）：管理员回退那一道与「每块盘上最新的有效根」两种算法的对拍
/// 两组用例共用。两块 4 GiB 的内存稀疏盘（[`super::shadow_ledger_exempts_and_isolates_slot_by_slot_tests`] 里那一种），几何同那一组的 mkfs 参数。
#[cfg(test)]
mod pool_in_memory_after_the_first_file {
    use super::shadow_ledger_exempts_and_isolates_slot_by_slot_tests::SparseDeviceInMemory;
    use super::{roll_back_by_a_forward_publish, RollbackError, RollbackTarget, RolledBack};
    use crate::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
    use crate::allocator::PoolAllocator;
    use crate::block_device::{BlockDevice, WriteDurability};
    use crate::make_filesystem::{
        allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
    };
    use crate::root_ring::RootRingSlotsPerRegion;
    use crate::system_configuration::SystemImmutableSizes;
    use crate::transaction::{
        acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
        TransactionOutput,
    };

    /// 第一个文件的写入时刻；之后每次覆盖写晚一秒。
    const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
    /// 第一个文件的字节数；之后每次覆盖写少一个字节（都在一个数据单元以内）。
    const FIRST_FILE_BYTES: usize = 3000;

    /// 池：mkfs 参数、两块盘、这个进程的分配器、取到的实例代号、现行那一版。
    pub(super) struct PoolInMemory {
        pub(super) parameters: MakeFilesystemParameters,
        pub(super) devices: Vec<(DeviceIdentity, SparseDeviceInMemory)>,
        pub(super) allocator: PoolAllocator,
        pub(super) instance: InstanceGeneration,
        pub(super) current: TransactionOutput,
        write_time_seconds: u64,
    }

    /// `length` 个字节的文件内容，`seed` 不同内容就不同。
    fn content_of(length: usize, seed: u64) -> Vec<u8> {
        (0..length)
            .map(|index| {
                let index = u64::try_from(index).expect("长度装得进 u64");
                u8::try_from((index * 131 + seed * 7 + 1) % 251).expect("小于 256")
            })
            .collect()
    }

    /// mkfs、取号、暖机、第一个文件，都在同一个进程里。
    pub(super) fn pool_after_the_first_file() -> PoolInMemory {
        let parameters = MakeFilesystemParameters {
            filesystem_identifier: [7; 16],
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            geometry: SystemImmutableSizes {
                physical_block_size: 512,
                minimum_input_output_bytes: 512,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
        };
        let mut devices: Vec<(DeviceIdentity, SparseDeviceInMemory)> =
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|identity| (identity, SparseDeviceInMemory::blank()))
                .collect();
        let genesis =
            make_filesystem(&parameters, &mut devices).expect("4 GiB 两盘照默认几何建得成");
        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
        let first_content = content_of(FIRST_FILE_BYTES, 0);
        let (instance, current) = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let instance = acquire_instance(&mut writer).expect("mkfs 刚建的池上取号做得成");
            let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机做得成");
            let current = publish_first_file(
                &mut writer,
                &mut allocator,
                warmed.roots.last().expect("暖机写了两条根"),
                FirstFile {
                    content: &first_content,
                    write_time_seconds: FIRST_WRITE_TIME_SECONDS,
                },
                instance,
                &warmed.last_record_bytes,
            )
            .expect("第一个文件落得下");
            (instance, current)
        };
        PoolInMemory {
            parameters,
            devices,
            allocator,
            instance,
            current,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS,
        }
    }

    impl PoolInMemory {
        /// 同一个进程里覆盖写一次，新的一版记成现行那一版、交回它。
        pub(super) fn overwrite(&mut self) -> TransactionOutput {
            self.write_time_seconds += 1;
            let content = content_of(FIRST_FILE_BYTES - 1, self.write_time_seconds);
            let mut writer = PoolWriter::new(&self.parameters, self.devices.as_mut_slice());
            let output = publish_overwrite(
                &mut writer,
                &mut self.allocator,
                &self.current,
                FirstFile {
                    content: &content,
                    write_time_seconds: self.write_time_seconds,
                },
                self.instance,
            )
            .expect("4 GiB 两盘上覆盖写落得下");
            self.current = output.clone();
            output
        }

        /// 挂着时回退到 `target`（[`roll_back_by_a_forward_publish`]），做成时现行那一版换成回退那次发布。
        pub(super) fn roll_back(
            &mut self,
            target: RollbackTarget,
        ) -> Result<RolledBack, RollbackError> {
            roll_back_by_a_forward_publish(
                &self.parameters,
                &mut self.devices,
                &mut self.allocator,
                &mut self.current,
                target,
            )
        }

        /// `device` 上 `offset` 起 `length` 个字节，不经任何发布。
        pub(super) fn bytes_on(
            &self,
            device: DeviceIdentity,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> Vec<u8> {
            let (_, block_device) = self
                .devices
                .iter()
                .find(|(identity, _)| *identity == device)
                .expect("池里有这块盘");
            let mut bytes = vec![0; length];
            block_device
                .read_at(offset, &mut bytes)
                .expect("读的那一段在盘里");
            bytes
        }

        /// 把 `bytes` 直接写到 `device` 上 `offset` 起那一段，不经任何发布（用例拿它摆坏份与读不出的根槽）。
        pub(super) fn write_bytes_directly_on(
            &mut self,
            device: DeviceIdentity,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
        ) {
            let (_, block_device) = self
                .devices
                .iter_mut()
                .find(|(identity, _)| *identity == device)
                .expect("池里有这块盘");
            block_device
                .write_at(offset, bytes, WriteDurability::Plain)
                .expect("写的那一段在盘里");
        }
    }
}
