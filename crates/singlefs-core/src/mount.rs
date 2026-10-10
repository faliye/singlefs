//! 可写挂载（里程碑「覆盖写、释放、回退与复用」步 3）：进程重开镜像之后先走一次恢复，从盘上重建可写态——所选根、施加前缀之后的根、
//! 上一版全部角色的单元与指针、分配器、下一条 jsn——再取实例代号、给上一个实例写行（D18（块里携带什么信息） 已定项 11）、
//! 推空发布直到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 戊），之后本实例的发布才接在后面。
//! 第一版没有干净关闭标记，重开一律走恢复。
//!
//! 挂着的时候：抬 F（准入那一格与正常卸载共用那一串，D16（发布语义） 已定项 1），与管理员回退——挂着时的一次向前发布
//! （D23（journal 的角色与格式） 已定项 14，[`roll_back_by_a_forward_publish`]）。

use crate::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, JournalSequenceNumber, SlotNumber,
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
use crate::block_device::BlockDevice;
use crate::checksum::crc32_castagnoli;
use crate::instance_table_sweep::{
    sweep_the_instance_table_at_the_end_of_the_mount, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration,
};
use crate::journal::back_chain_of;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::mounted_session::refusal_is_short_of_space;
use crate::pointer::LocationEntry;
use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
use crate::records::{
    LineageRecord, TreeTableEntry, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
};
use crate::recovery::read_unit_via_locations;
use crate::recovery::{
    allocation_records_fit_the_pool_geometry, every_device_reaches_the_unit_area_start,
    placement_lies_in_the_unit_area_of_its_device,
    unit_area_start_of_the_chosen_system_configuration,
};
use crate::recovery::{
    allocation_records_of_version_without_file_in_the_unit_area_starting_at,
    allocation_records_under_root_in_the_unit_area_starting_at, choose_root,
    choose_system_configuration,
    effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once,
    effective_rollback_floor_rereading_the_newest_instance_table_once,
    effective_rollback_floor_under_the_newest_roots_table, highest_root_txg,
    highest_tree_identifier_watermark_in_the_ring, instance_table_chain_of_root,
    instance_table_of_root, instance_table_page_pointers_as_far_as_readable,
    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
    readable_roots, readable_roots_rereading_ring_slots_known_to_hold_a_root_once,
    readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once,
    rebuild_version_in_the_unit_area_starting_at, replay_journal,
    root_is_abandoned_by_the_instance_table, scan_journal,
    system_configuration_slots_of_one_device_read_once,
    system_configuration_witness_is_undecidable, tree_table_has_no_entries,
    user_visible_tree_root_pointers, verified_system_configuration_slots, BadRootRingSlotReading,
    EffectiveFloorReadingStillUnreadableAfterOneReread, InstanceTableChain,
    InstanceTableOfTheNewestRootStillUnreadableAfterOneReread, JournalScanReport, PoolReader,
    RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
    RootRingSlotKnownToHoldARootStillBadAfterOneReread, SystemConfigurationSlotsOfOneDevice,
    UserVisibleTreeRootPointers,
};
use crate::recovery::{every_root_ring_slot, read_root_ring_slot, RootRingSlotReading};
use crate::root_record::{RootRecord, UnmountMarker};
use crate::root_ring::{target_for_publish, RootRingSlot};
use crate::system_configuration::SystemImmutableSizes;
use crate::transaction::{
    acquire_expected_instance, central_mapping_locations_in_the_version,
    copies_of_a_released_unit_failing_the_release_checksum_check,
    next_instance_generation_to_acquire, placement_to_release_after_checking_every_device,
    prepare_the_publish_without_units, prepare_the_row_publish_on_a_version_without_file,
    prepare_the_version_publish, publish_instance_table_on_version_without_file, publish_version,
    publish_version_appending_a_lineage_record, publish_version_with_unmount_marker,
    publish_without_units, refuse_locations_that_do_not_name_two_pool_devices,
    role_of_allocation_record_tree_node, walk_the_deletion_queue_until_it_is_empty,
    write_the_raised_floor_into_every_system_configuration, AcquisitionFailed,
    CodeTwoTreeNodeCapacities, CopyQuarantinedAfterReleaseChecksumMismatch,
    DeletionAndLineageOfThePublish, DeletionQueueWalk, ExpectedInstanceAcquisitionFailed,
    InstanceGenerationPastTheTopOfItsRange, InstanceTableOnlyPublishPlan, InstanceTablePlan,
    InstanceTableRewrite, MappingLookup, PoolVersion, PoolWriter, PublishError, PublishPlan,
    PublishedUnit, RaisedFloorSystemConfigurationWriteFailed, ReleaseChecksumCheck,
    TransactionOutput, TransactionUnit, VersionWithoutFilePublishOutput, ZeroUnitPublishPlan,
};
use crate::unlink::DeletionSwitches;
use crate::write_accounting::{WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind};
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

pub use crate::instance_table::{InstanceRow, InstanceTableRecords};

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
    /// 所选根（抬 F 时是现行那一版的根）指着的实例表沿链有一片读不出，或解不出行与链指针（D18（块里携带什么信息） 已定项 11「表不可读时」）：
    /// 判不了候选集，在任何写之前拒绝。
    InstanceTableMalformed,
    Acquisition(AcquisitionFailed),
    /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成，或挂载续做里的一步没发成、不是空间不够：错，连同这次挂载交得出的写账
    /// （[`PublishSequenceFailed`]：续做那一格的已落盘各次是写行、暖机、推的抬 F 与续做里已落盘的每一次，按先后）。
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
    /// 树表 0 条、而根记录指着的实例表或树表不是 mkfs 写的那一版（指针的诞生 txg 不是 0）：这样一版的分配记录在哪没有条款，
    /// 拒绝挂载。详见 `format_time_allocator` 的文档注释。
    VersionWithoutFileNotWrittenByMakeFilesystem {
        root: RollbackTarget,
        instance_table_birth_txg: CheckpointTxg,
        tree_table_birth_txg: CheckpointTxg,
    },
    /// 树表 0 条、而根记录指着的实例表或树表那条指针的两条位置条目落在不同的槽：这一版的落点就是这两个单元，
    /// 而第一版的布局是两盘同槽（一个单元整个落在一列上、两盘各一份），两条位置条目各指一个槽的版本这个实现写不出来
    /// ⇒ 是一份坏镜像，拒绝挂载。槽号是**盘上读来的 6 字节**、不是我们的不变量，所以这里报错、不断言（panic 面普查 R5）。
    /// 拒的是「这一条指针的两条位置条目不同槽」，与「各盘算出来的落点不同」（`PlacementRefusal` 那两条第一版不支持的池形状）
    /// 不是同一件事：那两条说的是写侧算出来的落点，这一条说的是盘上已经写着的字节。在动分配器之前拒绝，盘上逐字节不变。
    FormatTimeUnitLocationsOnDifferentSlots {
        unit: TransactionUnit,
        disagreement: LocationEntriesOnDifferentSlots,
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
    /// （[`DeviceTableDisagreement`]，实审 A1b Q4）；挂着之后收盘表的入口另报交进来的某块盘两槽一份本池自证过的系统配置都没有
    /// （[`DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`]，代码三方 m2-closeout-code-r1 Z3-A 乙）。
    /// 两张清单至少一张不空。系统不可变配置 mkfs 之后不可改
    /// （D22（单元原子性怎么合成） 已定项 26 第一档），写入口按择到的那一份建（取号、写行、暖机、推抬 F 的系统配置写、
    /// 根槽与 journal 记录的落点都从它来），系统配置槽写又按盘表逐项写设备数与本盘设备号；两边不一致说明调用方拿错了池、参数或盘，
    /// 照调用方的写会把盘上不可变段改写成调用方给的值。可写挂载在选系统配置之后、恢复与任何写之前返回；挂着之后收调用方参数与盘表的入口
    /// （抬 F、准入抬 F、一次准入里再推一串、正常卸载）在任何写之前返回（实审 A1b Q2、Q3）：盘上逐字节不变。
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
    /// 取号之前空间准入不够、写行与暖机之后推抬 F 时，抬 F 自己报了不是空间不够的错（块设备错、根环槽读坏又重读仍坏、预演被别的理由拒……，
    /// 实审 A1b Q5）：那时实例已取、行已写，写行、暖机与之前推成的那几串已落盘（D16（发布语义） 已定项 1「准入」那一行）。
    /// 写入口、分配器与现行那一版随错丢掉，这次挂载已落盘的写账都在 [`FloorRaiseFailedAfterTheMountsPublishes`] 里
    /// （增补 2 收口表第 58 行：与 `Publish` 同一个形态，不交出来设备一层数到的写与程序交得出的账对不上）。
    /// 抬 F 那一串自己也取不到落点、被空间准入拒的不走这里（推满仍不够，挂载照样做成，`MountSpaceAdmission::StillShortAfterTheFloorRaises`）。
    /// 挂载续做里为空间推的抬 F 自己报了不是空间不够的错，同样交这个成员（代码轮 V1）：那时写行、暖机、准入推的、续做发成的各步与续做推成的那几串
    /// 都已落盘，写账口径见 [`FloorRaiseFailedAfterTheMountsPublishes`]。
    /// 装箱：不装箱 `MountError` 就大过 clippy `result_large_err` 的 128 字节。
    FloorRaiseFailedAfterTheMountsPublishes(Box<FloorRaiseFailedAfterTheMountsPublishes>),
    /// C554 乙（用户 2026-09-27 定：重读后再判，R = 1；岔路单 `research/prompts/c554-fix-forks.md` 第 1 行）：
    /// 可写挂载读到的样子里有一样更新的东西读不出，重读一次（D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」）仍读不出——
    /// 不按读得出的那一版往下走，不把读不出的那一版当成被抛弃，拒可写挂载。是哪一样读不出见 [`StillUnreadableAfterOneReread`]。
    /// 在取号之前返回：一个写都没发，盘上逐字节不变。只读挂载不走这一判，照常（`mounted_read`、`recovery::recover`）。
    /// 挂着时抬 F 那一串（准入与卸载共用）算 F 生效值、重算影子账读根环重读一次仍读坏，同样报这一条
    /// （[`StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor`]、
    /// [`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`]，实审 A3b 报告 Q4）：在动分配器与任何写之前拒这一次抬，
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
    /// 取号之前的逐盘核（落后支要读每块盘自证过、世代号最大的那一份）读到 `device` 两槽里一份本池自证过的都没有。
    CheckingEachDeviceBeforeTheAcquisition { device: DeviceIdentity },
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
    /// 判据 N-配置（定义取自 E158 第 3 次跑登记 `research/prompts/e158-r3-prereg.md` 第 347 行）：系统配置见证过一次比所选那一版新的发布——
    /// 系统配置在发布的根槽 FUA 之后才轮换（D16（发布语义） 已定项 7），见证到的发布它的根落过盘——而读阶段（择根、扫 journal、重放）
    /// 交出的所选那一版比它旧；在这次挂载的读缓存上重做读阶段一遍（[`ReadStageCache`]），仍判真。
    PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read: SelectedVersionAgainstTheWitness,
        reread: SelectedVersionAgainstTheWitness,
    },
    /// 代码审阅第 22 条：重建分配器时按最新那条根的实例表判哪几条根被抛弃（影子账、回收的门槛、根环表都按它），那张表读不出，
    /// 重读一次仍读不出。不再按「没有一条根被抛弃」往下走（那样一个槽都不隔离、`abandoned_roots_unreadable` 仍是 0）。
    InstanceTableOfTheNewestRootForTheShadowLedger {
        /// 重读那一遍择到的最新那条根；根环那一遍一条自证过的根都没读到时 `None`。
        newest_root_on_the_reread: Option<RollbackTarget>,
    },
    /// 挂着时抬 F 算 F 生效值（`recovery::effective_rollback_floor_rereading_the_newest_instance_table_once`）或管理员回退判候选算
    /// F_生效（`recovery::effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）：根环里最新那条根
    /// 指着的实例表（按它判哪几条根被抛弃）读不出、解不开，或一条自证过的根都择不到，连根环一起重读一次仍是这样（C554 乙报告 Q6）。
    /// 不按「不按表滤」往下走（那样被抛弃时间线上的根带的 F 也算进生效值）。实审 A3b 报告 Q4：与乙那一族同级，改之前是
    /// `RecoveryFailure::InstanceTableOfTheNewestRootStillUnreadableAfterOneReread`。
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
    /// 实审 A3b 报告 Q4：与乙那一族同级，改之前是 `RecoveryFailure::RootRingSlotStillUnreadableAfterOneReread`（那时不分槽知不知道住着根）。
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
    /// 较新可读根那次发布一条记录都读不出时，只在系统配置里取过号的实例代号不比那条根新时走到这一拒（攻方
    /// `research/prompts/m3-batch37-43-code-r1-opus-output.md` 第一节 Y2，用户 2026-10-08 定「两形都修」）。
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
    /// c_见证 可能缺了那一槽，按判据为真处置（C331（择根倒挂压过已确认的写） 改取 R2，用户 2026-10-03 定，零轮；
    /// D22（单元原子性怎么合成） 已定项 8 射程）。撕裂的系统配置轮换（读得出、自证不过，设备没报错）不落在这里，照常取自证过的槽比。
    SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
}

/// 可写挂载在「重读一次」之前做什么（D16（发布语义） 已定项 1「根槽这一次读坏」那一行：这一次读不出就重读一次，仍坏就拒）。
/// 产品路径立即重读，两次读之间不等、不另设间隔。只供测试的那一臂在重读之前调用例给的钩子
/// （`.claude/rules/fs-design.md` 五条硬要求第 2 条：用例在钩子里撤掉暂时的读故障，强制进入「重读读得出」那一支；
/// 或在钩子里让重读那一遍也读坏，强制进入「重读仍读不出」那一支）。
/// 每处「重读一次」之前各调一次：读阶段那一次（判据 N-配置），重建分配器读最新那条根的实例表那一次（代码审阅第 22 条），
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
    /// 第一遍判真，在读缓存上重做一遍判假。
    OnTheOneReread {
        first_read: SelectedVersionAgainstTheWitness,
        reread: SelectedVersionAgainstTheWitness,
    },
}

/// 重建分配器时最新那条根的实例表在哪一遍读出来（代码审阅第 22 条）。
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
/// 那一串（实审 A1b Q5），与挂载续做里为空间推的那一串（代码轮 V1，`research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）。
/// 这次挂载交给设备的写（取号那两次系统配置槽写除外）= `writes_of_persisted_publishes` 各份 + `floor_raises` 各串先写系统配置那一步
/// + 写行之后那一处另加 `floor_raises` 各串的各次空发布（续做那一处它们已在 `writes_of_persisted_publishes` 里，不另加）+ `cause` 里那一串自己的写账。
#[derive(Debug)]
pub struct FloorRaiseFailedAfterTheMountsPublishes {
    /// 抬 F 自己报的错，原样：它那一串自己的写账（先写系统配置那一步、已落盘的几次空发布、失败那一次已记的写）在它里面
    /// （`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`、`RaiseFloorSequencePublishFailed`）。
    pub cause: MountError,
    /// 这次挂载在推报错那一串之前已经落盘的发布各自的写，按先后（与 `PublishSequenceFailed::writes_of_persisted_publishes` 同一个口径；
    /// 取号那两次系统配置槽写不是发布、不在这里）。写行之后那一处：写行那次在最前，之后暖机每次一份，推成的那几串的空发布不在这里
    /// （在 `floor_raises` 里）。续做那一处：按 txg 先后接着写行、暖机、准入推的那几串的各次空发布、续做发成的各步与续做里推成的那几串的各次空发布——
    /// 与续做那一格交 `Publish` 时的 `writes_of_persisted_publishes` 同一张表，崩溃注入照同一个划法从里面划出续做段。
    pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
    /// 报错那一串之前已经推成的那几串抬 F，按先后（每一串先写系统配置那一步与各次空发布的写都在里面，都已落盘）。续做那一处是准入推的那几串
    /// 接着续做里推成的那几串，它们的各次空发布同时在 `writes_of_persisted_publishes` 里。
    pub floor_raises: Vec<RaisedFloor>,
}

/// 调用方交进来的参数或盘表不对：可写挂载、挂着之后收参数与盘表的入口（抬 F、准入抬 F、一次准入里再推一串、正常卸载）与管理员回退
/// 共用一套核（[`caller_inputs_agreeing_with_the_disk`]；实审 A1b Q2、Q3，代码审阅第 17、18 条）。两个成员与 `MountError` 那两个
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

/// 交进来的盘表与盘上择到的系统配置不一致的一项（`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`，实审 A1b Q4）：
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
    /// 那一版有单元在这块盘上它的落点读不出、或字节与验得过的那一份不同：按角色与落点列出，次序同那一版的单元清单。
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
    /// 而且所选那一版有单元在这块盘上它的落点读不出、或者字节与验得过的那一份不同（`units_missing`，按角色与落点列出，
    /// 次序同所选那一版的单元清单）。只落后、单元都在的不算：发布的根落盘之后、系统配置轮换之前崩了，
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
/// （D16（发布语义） 已定项 8 戊）；抬 F 那一串空发布——推到每块盘上都有一条带新 F 的根（D16（发布语义） 已定项 1）。
/// 写入口是这一串自己开的、随错一起丢掉，所以它交得出的写账都在这里：已经落盘的那几次发布各自的写，
/// 与失败那一次落盘阶段已记的写（增补 2 收口表第 58 行：不交出来，设备一层数到的写与程序交得出的账对不上）。
/// 不是发布的写不在其内：可写挂载取号那两次系统配置槽写（取号的回卷另由 `AcquisitionFailed` 报）。
#[derive(Debug)]
pub struct PublishSequenceFailed {
    pub cause: PublishError,
    /// 这一串里不属于任何一次发布、已经发出去的写：抬 F 那一串是第一次发布之前先把新 F 写进每块盘系统配置那一步
    /// （每块盘一次系统配置槽写，D16（发布语义） 已定项 1「抬 F 那一串」）；可写挂载那一串在写行、暖机那几次失败时是空账（取号那两次写不在这里，见上），
    /// 在挂载续做里失败时是这次挂载推成的每一串抬 F（准入推的与续做推的）先写系统配置那一步逐种相加——那几串的空发布在
    /// `writes_of_persisted_publishes` 里，先写系统配置那一步不是发布，只能放在这里（代码轮 V2，`research/prompts/m3-unlink-code-r2-main-verification.md`
    /// 第三节）。于是这一串交给设备的写 = 这一格 + 已落盘各份 + 失败账。
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
/// C314（回退可以复用被抛弃的根引用的单元） 那两格必红靠它强制进入；产品路径恒 `On`。
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
/// 交进来的盘表与这份逐项比（实审 A1b Q2、Q3）。字段私有，只由可写挂载与 [`ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`]
/// 读盘造出：调用方拼不出一份与盘上不同的参数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParametersAndDeviceTableOfTheMount {
    parameters_on_disk: MakeFilesystemParameters,
    device_identities_in_table_order: Vec<DeviceIdentity>,
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
        }
    }

    /// 盘上择到的那份系统配置里的 mkfs 参数：挂着之后的写入口按它建。
    #[must_use]
    pub fn parameters_on_disk(&self) -> &MakeFilesystemParameters {
        &self.parameters_on_disk
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
    /// 这次挂载认下的池（盘上那份参数、核过的盘表）：挂着的会话只用它（实审 A1b Q2、Q3）。
    pub parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
    /// 这次挂载重读过哪几样（C554 乙）：读阶段在哪一遍判完、最新那条根的实例表在哪一遍读出来、重建分配器读根环在哪一遍读出有效根。
    pub rereads: RereadsOfThisMount,
    /// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）发成的每一步，按先后：接在写行、暖机与推的抬 F 之后，走到了空时最后一个就是
    /// `Mounted::current`；所选那一版的 `deleted_inodes` 是空的（常态）时空。
    pub deletion_queue_continuation: Vec<TransactionOutput>,
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

fn map_rebuild_failure(failure: RebuildVersionFailure) -> MountError {
    match failure {
        RebuildVersionFailure::Walk(recovery_failure) => MountError::Recovery(recovery_failure),
        RebuildVersionFailure::NoRecordStandingForFileVersion => {
            MountError::FileVersionWithoutAnyJournalRecord
        }
    }
}

/// 恢复之后从盘上重建出来的上一版，带文件的连同写行要接在后面的那一版实例表。
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只有一个，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
#[allow(
    clippy::enum_variant_names,
    reason = "三个成员与 PoolVersion、RebuiltVersion 同名（D8 已定项 16 第 5 条：第三个成员三处同形），都以 File 结尾是条款的叫法"
)]
enum PreviousVersion {
    /// 树表 0 条：这一版只有根自己指着的实例表与树表两个单元。写行要在这一版那张实例表后面接行，所以连表一起解出来。
    WithoutFile {
        root: RootRecord,
        table: InstanceTableChain,
    },
    WithFile {
        output: TransactionOutput,
        table: InstanceTableChain,
    },
    /// 有树、文件集合为空（D8（核心索引结构） 已定项 16 第 5 条，`RebuiltVersion::WithTreesAndNoFile`）：写行与暖机照带文件的一版那样做，
    /// 发出来的一版按 `PoolVersion::of_a_version_of_trees` 归成员。
    WithTreesAndNoFile {
        output: TransactionOutput,
        table: InstanceTableChain,
    },
}

impl PreviousVersion {
    /// 这一版的实例表链：全部行（写行时要在它后面接上这次的行，取号之前的准入也按它的行数算）与每一片的指针
    /// （写行那次发布整条链重写，这几片逐片释放）。两臂都有表：树表 0 条那一版的表是根记录直接指着的那条链
    /// （mkfs 种下的，或上一次挂载写行重写的）。
    fn instance_table_chain(&self) -> &InstanceTableChain {
        match self {
            PreviousVersion::WithoutFile { table, .. }
            | PreviousVersion::WithFile { table, .. }
            | PreviousVersion::WithTreesAndNoFile { table, .. } => table,
        }
    }

    /// 有树的那一版（带文件的、有树而文件集合为空的）；树表 0 条的是 `None`（空间准入的 ckpt_cost 按它算，
    /// `admission::checkpoint_cost_of_the_version_to_build_on`）。
    fn version_of_trees(&self) -> Option<&TransactionOutput> {
        match self {
            PreviousVersion::WithoutFile { .. } => None,
            PreviousVersion::WithFile { output, .. }
            | PreviousVersion::WithTreesAndNoFile { output, .. } => Some(output),
        }
    }

    /// 这一版的单元（角色、落点、验得过的那一份字节）：可写挂载在取号之前逐盘核每块盘带不带它们
    /// （`devices_without_the_selected_version`）。落点取第一条位置条目的槽——第一版两盘同槽（D2（RAID 条带策略） 已定项 10），
    /// 发布路径照抄一个单元时也按这一个落点对待它。
    ///
    /// 带文件的一版就是重建出来的 `units`（全部角色；提示与映射都读不出的数据单元字节是空的，逐盘核时跳过它）。
    /// 树表 0 条的一版是根记录直接指着的几个单元：实例表第 0 片、树表、写过行的一版那棵分配记录树的每个节点
    /// （与 `placements_referenced_by_root` 树表 0 条那一臂同一份清单，字节按位置条目读验得过的那一份）。
    /// 两臂都不列实例表链第 1 片起的各片：每次写行整条链 COW 重写（D18（块里携带什么信息） 已定项 11），后面各片与第 0 片同一次发布写出，
    /// 停在那次发布之前的盘缺它们，就也缺第 0 片。
    ///
    /// # Errors
    /// 树表 0 条那一版的实例表、树表或分配记录树这一次读不出、解不开（重建时读得出，两次读之间的瞬时读错）⇒ `Recovery`：
    /// 核不了就不放行，在任何写之前返回。
    #[allow(
        clippy::ptr_arg,
        reason = "allocation_records_of_version_without_file 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
    )]
    fn units<Device: BlockDevice>(
        &self,
        devices: &Vec<(DeviceIdentity, Device)>,
        unit_area_start: UnitAreaStart,
    ) -> Result<Cow<'_, [PublishedUnit]>, MountError> {
        match self {
            PreviousVersion::WithFile { output, .. }
            | PreviousVersion::WithTreesAndNoFile { output, .. } => {
                Ok(Cow::Borrowed(&output.units))
            }
            PreviousVersion::WithoutFile { root, .. } => {
                let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
                let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
                let mut units = vec![
                    PublishedUnit {
                        slot: root.instance_table.locations[0].slot,
                        identity: TransactionUnit::InstanceTable,
                        bytes: read_unit_via_locations(
                            devices,
                            &root.instance_table.locations,
                            data_unit_bytes,
                        )?,
                    },
                    PublishedUnit {
                        slot: root.tree_table.locations[0].slot,
                        identity: TransactionUnit::TreeTable,
                        bytes: read_unit_via_locations(
                            devices,
                            &root.tree_table.locations,
                            node_bytes,
                        )?,
                    },
                ];
                if let Some(tree) =
                    allocation_records_of_version_without_file_in_the_unit_area_starting_at(
                        devices,
                        unit_area_start,
                        root,
                    )?
                {
                    for (node, pointer) in &tree.version.nodes {
                        units.push(PublishedUnit {
                            slot: pointer.locations[0].slot,
                            identity: role_of_allocation_record_tree_node(*node),
                            bytes: read_unit_via_locations(
                                devices,
                                &pointer.locations,
                                node_bytes,
                            )?,
                        });
                    }
                }
                Ok(Cow::Owned(units))
            }
        }
    }
}

/// 写行那次发布的实例表链怎么重写：这一版那张表的行后面接上这次写的行，被换下的是这一版的整条链
/// （D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）。取号之前的准入与写行那次发布读的都是它，两处同一份输入。
fn instance_table_rewrite_of_the_row_publish(
    table: &InstanceTableChain,
    rows_written: &[InstanceRow],
) -> InstanceTableRewrite {
    let mut rows = table.records.rows.clone();
    rows.extend_from_slice(rows_written);
    InstanceTableRewrite {
        rows,
        replaced_chain: table.page_pointers.clone(),
    }
}

/// 重建上一版：树表 0 条就留根加它指着的那张实例表；带文件的连同重建出来的单元。两臂的实例表都从根记录里那条指针沿链读
/// （D18（块里携带什么信息） 已定项 11）——只解 `TransactionUnit::InstanceTable` 那一个单元的字节只有第 0 片，
/// 多于一片的表会被按前半张判准入、接行。
#[allow(
    clippy::ptr_arg,
    reason = "rebuild_version 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn rebuild_previous_version<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    record_standing_for_root: Option<crate::journal::JournalRecord>,
) -> Result<PreviousVersion, MountError> {
    let rebuilt = rebuild_version_in_the_unit_area_starting_at(
        devices,
        unit_area_start,
        root,
        record_standing_for_root,
    )
    .map_err(map_rebuild_failure)?;
    let table = instance_table_chain_of_root(devices, root)
        .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?;
    match rebuilt {
        RebuiltVersion::WithoutFile => Ok(PreviousVersion::WithoutFile { root: *root, table }),
        RebuiltVersion::WithFile(output) => Ok(PreviousVersion::WithFile { output, table }),
        RebuiltVersion::WithTreesAndNoFile(output) => {
            Ok(PreviousVersion::WithTreesAndNoFile { output, table })
        }
    }
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
}

/// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。
///
/// # Errors
/// 那个最大值已是 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条：盘上读来的 txg，不 panic）。
fn first_txg_of_new_instance<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
) -> Result<CheckpointTxg, MountError> {
    let highest_record_txg = records
        .values()
        .map(|record| record.checkpoint_txg)
        .max()
        .unwrap_or(CheckpointTxg(0));
    let highest_ring_txg = highest_root_txg(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .unwrap_or(CheckpointTxg(0));
    checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))
}

/// 接在 txg 为 `version` 的那一版后面的下一个 txg（它加一），挂载一侧（可写挂载、抬 F、正常卸载）用。
///
/// # Errors
/// `version` 已是 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条）。
fn checkpoint_txg_after_the_version(version: CheckpointTxg) -> Result<CheckpointTxg, MountError> {
    version.0.checked_add(1).map(CheckpointTxg).ok_or(
        MountError::SequenceNumberPastTheTopOfItsRange(
            SequenceNumberAtTheTopOfItsRange::CheckpointTxg(version),
        ),
    )
}

/// 本实例第一次发布要带的树 ID 水位（D8（核心索引结构） 已定项 8 ②「根环里全部根记录该字段的 max」）：
/// 根环里全部自证过的根与 `records`（环里全部自证通过的记录）各自带的水位取 max，再与这次要接在后面的那一版自己带的取 max——
/// 那一版是从环里读出来的根或从环里的记录施加出来的，按构造已经在内；再取一次是为了两次读环之间的瞬时读错
/// （择根那一遍读到了它、这一遍没读到）不把水位拉低。读不出的根怎么算见 `highest_tree_identifier_watermark_in_the_ring`。
fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    version_to_build_on: &RootRecord,
) -> u64 {
    highest_tree_identifier_watermark_in_the_ring(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        records,
    )
    .map_or(version_to_build_on.tree_identifier_watermark, |ring| {
        ring.max(version_to_build_on.tree_identifier_watermark)
    })
}

/// 进程内重读一个挂载那一刻读坏的根环槽（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）：只读一次，不重试；读得出、自证过、
/// 按 `table` 仍然有效 ⇒ 照它的 txg；读得出、自证过而按 `table` 被抛弃 ⇒ 当没有根；读回全零 ⇒ 不算确认为空，照旧钉在它可能住过的最旧 txg
/// （用户 2026-10-05 定乙：读回全零的槽在回收门槛里照「自证不过」那一类钉住，只是不并进准入抬 F 的上限）；设备报读错或读得出、不是全零而自证不过
/// ⇒ 照旧钉在它可能住过的最旧 txg（用户 2026-10-04 定改法 A，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1：
/// 这一刻读到「自证不过」可能是一次读坏的有效根，当没有根在这上面让回收门槛越过盘上一条有效根）。
#[must_use]
pub fn reread_of_a_root_ring_slot_bad_at_mount<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters: &MakeFilesystemParameters,
    table: &InstanceTableRecords,
    ring_slot: RootRingSlot,
) -> RereadOfARootRingSlotBadAtMount {
    match read_root_ring_slot_telling_never_written_apart(
        reader,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
        ring_slot,
    ) {
        RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root)
            if !abandoned_by_table(&root, table) =>
        {
            RereadOfARootRingSlotBadAtMount::ValidRoot {
                checkpoint_txg: root.checkpoint_txg,
            }
        }
        RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(_) => {
            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable
        }
        RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => {
            RereadOfARootRingSlotBadAtMount::AllZero
        }
        RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => {
            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
        }
    }
}

/// 这个进程写了一条根：记进分配器（[`PoolAllocator::record_root_written_by_this_process`]），记之前照 E2 重读挂载那一刻读坏、
/// 记了这条根之后会是环里最小的那个槽（[`PoolAllocator::record_root_written_by_this_process_rereading_slots_bad_at_mount`]；
/// D16（发布语义） 已定项 1，用户 2026-10-03 定；读坏的槽钉在它可能住过的最旧 txg，用户 2026-10-04 定）。名字里的「读不出」是读坏的两类
/// （读不出、不是全零而自证不过）合起来说的：`transaction` 里两处调用点按这个名字调，这一批不改那份文件。
/// 判「按实例表仍然有效」用分配器根环表上记着的那一张
/// （`RootRingOccupancy::instance_table_judging_the_rereads`）。产品路径上记根的每一处都走它：带单元的发布在取完落点之后
/// （`transaction::publish_admitted`、`transaction::prepare_the_row_publish_on_a_version_without_file`），零单元发布在根落盘之后。
/// 抬 F 那一串在分配器拷贝上的整串预演（`rehearse_the_publishes_raising_the_floor`）不重读：它按钉着的最旧 txg 算、回收得只会更少，
/// 预演过得去真发就过得去。
pub fn record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    parameters: &MakeFilesystemParameters,
    allocator: &mut PoolAllocator,
    checkpoint_txg: CheckpointTxg,
) {
    let table = allocator.root_ring_occupancy().map_or_else(
        || InstanceTableRecords { rows: Vec::new() },
        |occupancy| occupancy.instance_table_judging_the_rereads().clone(),
    );
    allocator.record_root_written_by_this_process_rereading_slots_bad_at_mount(
        checkpoint_txg,
        &mut |ring_slot| {
            reread_of_a_root_ring_slot_bad_at_mount(reader, parameters, &table, ring_slot)
        },
    );
}

/// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」，用户 2026-10-04 定 RGall：
/// 从一圈窗口退回整个环，`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）：根环 R × S 槽（S 读自系统配置），第 0 代根被盖之前
/// 环里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由环里最旧的有效根接管。调用方交进来的 `oldest_valid_root` 取的是环里全部有效根
/// （挂载时 `rebuilt_allocator`，进程内 `allocator::RootRingOccupancy`，抬 F 时 `raise_rollback_floor`），不按一圈窗口滤。
#[must_use]
pub fn reclaim_floor(
    effective_floor: CheckpointTxg,
    oldest_valid_root: Option<CheckpointTxg>,
) -> CheckpointTxg {
    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
}

/// 一条根按一张实例表判是不是被抛弃的（`recovery::root_is_abandoned_by_the_instance_table`，一处定义）。
fn abandoned_by_table(root: &RootRecord, table: &InstanceTableRecords) -> bool {
    root_is_abandoned_by_the_instance_table(root, table)
}

/// 重建分配器之后要带回去的东西。
struct RebuiltAllocator {
    allocator: PoolAllocator,
    effective_floor: CheckpointTxg,
    abandoned_roots_unreadable: u64,
    instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead,
    root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead,
}

/// 重建分配器判哪几条根被抛弃用的那张表：最新那条根（`choose_root` 现择）指着的实例表（代码审阅第 22 条）。
/// 这一次读不出（择不到根、或那张表沿链有一片读不出、解不开）就重读一次（D16（发布语义） 已定项 1「重读一次」；C554 乙，R = 1），
/// 重读之前调 `before_the_one_reread`。
///
/// # Errors
/// 重读仍读不出 ⇒ `NewerStateStillUnreadableAfterOneReread(InstanceTableOfTheNewestRootForTheShadowLedger)`：
/// 不按「没有一条根被抛弃」往下走（那样一个槽都不隔离）。可写挂载在取号之前调它，拒的时候盘上逐字节不变。
fn instance_table_of_the_newest_root_read_at_most_twice(
    reader: &dyn PoolReader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<(InstanceTableRecords, InstanceTableOfTheNewestRootRead), MountError> {
    if let Some(table) = choose_root(reader, system_configuration)
        .and_then(|newest| instance_table_of_root(reader, &newest))
    {
        return Ok((table, InstanceTableOfTheNewestRootRead::OnTheFirstRead));
    }
    before_the_one_reread.before_rereading();
    let newest_on_the_reread = choose_root(reader, system_configuration);
    match newest_on_the_reread.and_then(|newest| instance_table_of_root(reader, &newest)) {
        Some(table) => Ok((table, InstanceTableOfTheNewestRootRead::OnTheOneReread)),
        None => Err(MountError::NewerStateStillUnreadableAfterOneReread(
            Box::new(
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                    newest_root_on_the_reread: newest_on_the_reread.map(|newest| RollbackTarget {
                        instance: newest.instance,
                        checkpoint_txg: newest.checkpoint_txg,
                    }),
                },
            ),
        )),
    }
}

/// 影子账算完交回的：读不出账的被抛弃根条数，与每条读得出账的被抛弃根引用的全部落点（按根的 (实例, txg) 认；
/// 分配器那张根环表拿它判「这条根离开根环时清哪几个槽」「挂载内回收出来的槽还有没有被抛弃根引用」，`allocator::RootRingOccupant`）。
struct ShadowLedgerComputed {
    abandoned_roots_unreadable: u64,
    placements_referenced_by_abandoned_root:
        BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
}

/// 影子账里一条被抛弃根引用的落点从哪来（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句「抬 F 那一串（准入与卸载）里的被抛弃根」、
/// D23（journal 的角色与格式） 已定项 14 回退段「抬 F 那一串（准入与卸载）不走这一道」，用户 2026-10-07 定）。封闭集合，`match` 不写通配臂。
enum PlacementsOfAbandonedRoots<'recorded> {
    /// 可写挂载重建分配器那一遍：从盘上读它的账，读不出照 C393 从映射现算、重读一次、再看 C694 的 B 凭据，仍不成就拒
    /// （「崩了再可写挂载」那一道照 B 拒，不变）。
    ReadFromTheirAccounts,
    /// 抬 F 那一串（准入与卸载）：按 (实例代号, txg) 认、根环表里记着同一条被抛弃根的（`allocator::RootRingOccupancy::placements_recorded_for_the_abandoned_root`），
    /// 照挂载时算出、记着的那一串落点算，不再从盘上读它的账，读不出也不拒。挂载之后才按现行那一版的实例表判成被抛弃的根（挂载那一刻它按最新那条根的表
    /// 判有效、记成有效根）借 C 那一圈在那一槽记着的落点（那一槽记的是有效根且 txg 相同，
    /// `allocator::RootRingOccupancy::placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot`，用户 2026-10-08 定 H2），
    /// 同样不读账、读不出不拒。两样都没记着的（C 在那一槽也没记的；挂载那一刻读坏、读回全零的槽这一遍读出的被抛弃根；
    /// 进程内重读读出被抛弃根、已从表里拿掉的槽）照可写挂载那一遍读盘（「C 也没记着的照今天读盘」）。
    RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
        &'recorded BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
    ),
}

impl PlacementsOfAbandonedRoots<'_> {
    /// 根环表里记着的这条被抛弃根的落点；读盘那一臂与表里没记着它时 `None`。只读内存。
    fn recorded_in_the_root_ring_table(
        &self,
        root: &RootRecord,
    ) -> Option<Vec<(DeviceIdentity, SlotNumber, u64)>> {
        match self {
            Self::ReadFromTheirAccounts => None,
            Self::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(recorded) => recorded
                .get(&(root.instance, root.checkpoint_txg))
                .map(|placements| {
                    placements
                        .iter()
                        .map(|placement| (placement.device, placement.slot, placement.span))
                        .collect()
                }),
        }
    }
}

/// 影子账（D28（挂载期承诺量） 已定项 1 第九项）：只被被抛弃根引用的槽 = 被抛弃根引用的槽 − 候选集里的根引用的槽 − 当前这一版账里
/// 还分配着的槽；候选 = 可读 ∧ 按实例表有效 ∧ txg ≥ F。用户 2026-09-16 定的窄读法措辞是「仍被有效根引用的槽不在其内」、有效只按实例表判，
/// 豁免只给候选集里的根（多了 txg ≥ F、抬 F 之后按新 F 重算）是对那句措辞的收严（alloc-basis 那一轮的 G5 臂）：按原措辞，抬 F 到 11 之后
/// A 仍豁免 mkfs 实例表那 2 槽，它们被回收、发出去，而被抛弃根 B 还引用着，违反 D23（journal 的角色与格式） 已定项 14 的主句。
/// 预想、偏离用户定案的措辞，交用户。
/// 一条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。
/// 被抛弃根的树表或分配记录树读不出、解不开时不跳过：从中央映射现算它引用的落点照样隔离，计进 `abandoned_roots_unreadable`
/// （C393 取 (b)-从映射现算）；映射也现算不成就重读一次（重读之前调钩子），仍不成时看现行账（`current_records`）：现行账里那条根的树表槽
/// （与中央映射树根槽，映射根是空指针时只看树表槽）每块盘上都有一条分配代大于那条根的 txg、罩住它的记录，就是那几槽已被那条根之后的合法写占了，
/// 那条根引用的单元已经没有要护的，只计数、不隔离、不拒（C694 的 B，判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条：
/// 凭据只读内存里的现行账，不读那一槽）；没有这样的记录（那几槽只是这一次读不出）就在任何写之前拒：挂载在取号之前拒可写、抬 F 拒这一次抬
/// （抬 F 那一串只在根环表里没记着这条根时才读盘、才走到这一拒）
/// （[`StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot`]）。候选根读不出就当它什么都不豁免（隔离只会多不会少）。
/// 同一次也隔离水位之上没施加的 journal 记录点名的落点、那几次发布的账里的落点与比所选那一版新的可读根的账里的落点
/// （`placements_named_by_unapplied_journal_records`，C694 的 C 连同 `C_ACCOUNT`、`C_NEWER_ROOTS`、`C_MAPPING`）：
/// 只豁免现行账里仍分配着的槽，不豁免候选集里的根引用的槽（根环线 K4-1，函数体里那一圈的注释）；落点里分配位置着的槽
/// （已释放、还在 defer 窗口里的也算）不隔离、回收时补（同一圈的注释），D28（挂载期承诺量） 已定项 1 第九项
/// 按分配器实际隔离的槽数取，两部分都在里面。
/// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；
/// 被抛弃根引用的只在它离开根环的那一次发布里清，水位之上的记录点名的只在这个进程写掉那条记录的 txg 对应的根环槽那一次发布里清
/// （`PoolAllocator::record_root_written_by_this_process`，按交回的每条根引用的落点与分配器根环表记着的那一份）。
/// 被抛弃根引用的落点从哪来由 `placements_of_abandoned_roots` 定（[`PlacementsOfAbandonedRoots`]）：可写挂载那一遍读盘，
/// 抬 F 那一串取根环表里记着的那一份，表里没记着这条根的才读盘（上面那一串读不出的处置只走在这一支上）。
/// `placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad` 是抬 F 那一遍根槽读坏、没读出来的被抛弃根挂载时记在根环表里的落点
/// （D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1「被抛弃根的槽不拒」，[`readable_roots_for_raising_the_floor`]）：
/// 与读出来的被抛弃根同一种豁免（候选集里的根引用的、当前账里还分配着的不隔离），不计进 `abandoned_roots_unreadable`（它们的账一个字都没读）；
/// 可写挂载那一遍给空。
#[allow(
    clippy::ptr_arg,
    clippy::too_many_arguments,
    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、水位之上没施加的记录点名的落点、被抛弃根的落点从哪来、重读前的快照、根槽这一遍读坏的被抛弃根记着的落点"
)]
fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    allocator: &mut PoolAllocator,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    floor: CheckpointTxg,
    current_records: &[AllocationRecord],
    placements_named_by_unapplied_journal_records: &[PlacementOnDevice],
    placements_of_abandoned_roots: PlacementsOfAbandonedRoots<'_>,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
    placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[PlacementOnDevice],
) -> Result<ShadowLedgerComputed, MountError> {
    let referenced_by_the_current_account: BTreeSet<(u32, u64)> = current_records
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    let mut referenced_by_candidates: BTreeSet<(u32, u64)> =
        referenced_by_the_current_account.clone();
    // 候选 = 按实例表有效 ∧ txg ≥ F（D16（发布语义） 已定项 1：影子账的候选过滤是原来同取一圈窗口的六处之一，用户 2026-10-04 定 RGall
    // 一起退回整个环：窗口外的有效根照样是候选，它引用的槽豁免）。
    for root in roots
        .iter()
        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)
    {
        if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {
            referenced_by_candidates.extend(
                placements
                    .iter()
                    .map(|(device, slot, _)| (device.0, slot.0)),
            );
        }
    }
    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut unreadable = 0;
    let mut placements_referenced_by_abandoned_root = BTreeMap::new();
    for root in roots.iter().filter(|root| is_abandoned(root)) {
        // C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮；
        // `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）：账（树表或分配记录树）读不出时不跳过，
        // 从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报（显式报告）；
        // 现算也不成（映射树读不出、解不开）才重读一次（重读之前调钩子），账与现算都仍不成就在取号之前拒可写、只读照常。
        // 抬 F 那一串先取根环表里记着的那一份（`or_else` 是惰性的：取到就不读盘，读不出的处置也走不到）。
        let recorded_in_the_root_ring_table =
            placements_of_abandoned_roots.recorded_in_the_root_ring_table(root);
        // 回收时补不补得上隔离（第 10z 批第十四块实现员报告第七节第 2 条，派发第十六块第 2 条）：根环表里记着这条根的落点（表里的被抛弃根、
        // 借 C 那一圈的），或可写挂载那一遍（这一遍读出的落点随后整串记进表里，`rebuilt_allocator` 建根环表那一步），回收出来的槽照表补隔离
        // （`allocator::RootRingOccupancy::placements_the_isolation_still_protects`），分配位置着的槽就不在这里隔离、不与「已分配」双扣；
        // 抬 F 那一串读盘得来、表里没记着的，回收时没有一份补得上，照旧整段隔离。
        let reclaim_isolates_what_this_root_references = match (
            &placements_of_abandoned_roots,
            &recorded_in_the_root_ring_table,
        ) {
            (PlacementsOfAbandonedRoots::ReadFromTheirAccounts, _)
            | (
                PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
                    _,
                ),
                Some(_),
            ) => true,
            (
                PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
                    _,
                ),
                None,
            ) => false,
        };
        let placements = match recorded_in_the_root_ring_table
            .or_else(|| placements_referenced_by_root(devices, unit_area_start, root))
        {
            Some(placements) => placements,
            None => {
                unreadable += 1;
                placements_of_an_abandoned_root_whose_account_is_unreadable(
                    devices,
                    unit_area_start,
                    root,
                    current_records,
                    before_the_one_reread,
                )?
            }
        };
        for (device, slot, span_slots) in placements.iter().copied() {
            let key = (device.0, slot.0);
            if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
                continue;
            }
            if reclaim_isolates_what_this_root_references {
                allocator
                    .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
                        device, slot, span_slots,
                    );
            } else {
                allocator.isolate_abandoned(device, slot, span_slots);
            }
        }
        placements_referenced_by_abandoned_root.insert(
            (root.instance, root.checkpoint_txg),
            placements
                .into_iter()
                .map(|(device, slot, span)| PlacementOnDevice { device, slot, span })
                .collect(),
        );
    }
    // 根槽这一遍读坏的被抛弃根（H1）：读不出它的根记录，照它挂载时记着的落点补隔离，豁免与读出来的被抛弃根同一个集合。
    // 落点取自根环表，回收时照表补隔离：分配位置着的槽不在这里隔离（同上一圈）。
    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {
        let key = (placement.device.0, placement.slot.0);
        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
            continue;
        }
        allocator.isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
            placement.device,
            placement.slot,
            placement.span,
        );
    }
    // C 那一份不豁免候选根引用的槽（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K1 / K2 T1 C 的候选豁免」那一行，
    // 根环线 K4-1）：水位之上没施加的那次发布的根读得出、账读得出时它自己就是候选根（按实例表有效、txg ≥ F），豁免它引用的槽就是不护它的单元——
    // 它比所选那一版新，它新分配的单元不在现行账里、回收门槛也罩不到，挂载之后当空闲槽发出去。只豁免现行账里仍分配着的槽：它们这时发不出去，
    // 这次挂载里被释放、回收出来的时候，分配器按还没解开的这一份补隔离（`PoolAllocator::record_root_written_by_this_process`）；
    // 不豁免它们，第九项与「已分配」就把同一个槽数两遍。
    // 已释放、还在 defer 窗口里的槽同样在「已分配」里（分配位置着，`DeviceFreeMap::allocated_slots` 的读法甲），D28（挂载期承诺量） 已定项 1
    // 第九项「不在这里再算一次」：落点里分配位置着的槽一律不隔离（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md`
    // 第二节 K1-3 H2：宽护并进来的已释放落点被第九项与「已分配」各扣一次，1536 槽盘挂载那一刻隔离 2242 槽、写全被拒），它们回收的那一刻
    // 分配器照还没解开的这一份补隔离：挂载之内记根时（`PoolAllocator::record_root_written_by_this_process`），抬 F 那一串回收时
    // （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`，[`raise_rollback_floor`]）。
    for placement in placements_named_by_unapplied_journal_records {
        let key = (placement.device.0, placement.slot.0);
        if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {
            continue;
        }
        allocator
            .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(
                placement.device,
                placement.slot,
                placement.span,
            );
    }
    Ok(ShadowLedgerComputed {
        abandoned_roots_unreadable: unreadable,
        placements_referenced_by_abandoned_root,
    })
}

/// 一条被抛弃根的账（树表或分配记录树）这一遍读不出之后，它引用的落点从哪来（C393 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮；
/// C694 的 B，用户 2026-10-07 定）：[`placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`]，
/// 那一份交 `None` 就拒。
///
/// # Errors
/// 账与映射重读一次仍都不成、现行账里又没有那道凭据 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`（在任何写之前）。
#[allow(
    clippy::ptr_arg,
    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn placements_of_an_abandoned_root_whose_account_is_unreadable<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<Vec<(DeviceIdentity, SlotNumber, u64)>, MountError> {
    placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread(
        devices,
        unit_area_start,
        root,
        current_records,
        before_the_one_reread,
    )
    .ok_or_else(|| {
        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
            StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
                abandoned_root: RollbackTarget {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                },
            },
        ))
    })
}

/// 一条根的账（树表或分配记录树）这一遍读不出之后，它引用的落点从哪来，被抛弃根（[`placements_of_an_abandoned_root_whose_account_is_unreadable`]）
/// 与可写挂载的 C 读一次没施加的发布（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）共用这一份、交 `None` 时各报各的拒因：
/// 从中央映射现算；现算也不成就重读一次（重读之前调钩子），账或现算有一样成就用它；仍都不成时，现行账里有那条根之后的记录罩住它的树表槽
/// （与映射根槽）就当它什么都不引用（[`the_current_account_records_later_units_over_the_account_slots_of`]），没有就交 `None`。
#[allow(
    clippy::ptr_arg,
    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread<
    Device: BlockDevice,
>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Option<Vec<(DeviceIdentity, SlotNumber, u64)>> {
    if let Some(computed) =
        placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)
    {
        return Some(computed);
    }
    before_the_one_reread.before_rereading();
    if let Some(placements) =
        placements_referenced_by_root(devices, unit_area_start, root).or_else(|| {
            placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)
        })
    {
        return Some(placements);
    }
    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {
        return Some(Vec::new());
    }
    None
}

/// C694 的 B 那道凭据（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条）：被抛弃根 `root` 的树表槽、与它的中央映射树根槽
/// （映射根是空指针时没有这一槽，只看树表槽）的每一条位置条目，在现行账 `current_records` 里那块盘上都有一条分配代大于 `root` 的 txg、
/// 罩住那一槽的记录（已释放的也算：分配代说的是那一槽在 `root` 之后被重新分配过）。只读内存里的现行账，不读那几槽的内容——读回旧内容的
/// 瞬时故障下读那一槽判「诞生身份不同」会把活着的根判成已被占（判决第二节 K3 那一行，W1 出局）。
fn the_current_account_records_later_units_over_the_account_slots_of(
    root: &RootRecord,
    current_records: &[AllocationRecord],
) -> bool {
    let mapping_root_locations: &[crate::pointer::LocationEntry] =
        if root.mapping_root == crate::pointer::NodePointer::empty_root() {
            &[]
        } else {
            &root.mapping_root.locations
        };
    root.tree_table
        .locations
        .iter()
        .chain(mapping_root_locations)
        .all(|location| {
            current_records.iter().any(|record| {
                record.device == location.device
                    && record.slot.0 <= location.slot.0
                    && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
                    && record.generation > root.checkpoint_txg
            })
        })
}

/// 水位之上、没被施加的 journal 记录点名的落点（C694 的 C，判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条，选法 `all`：
/// 不只认带末条标志的记录），按那条记录的 txg 在根环里对应的槽（[`target_for_publish`]）归组。水位是所选那一版 `selected_version`
/// （施加之后的根）的 (实例代号, checkpoint_txg)，按实例代号为主比，与恢复判施加哪一段同一个序（D23（journal 的角色与格式） 已定项 14 注 2）；
/// 水位之下的记录要么施加过、要么比所选那一版旧，不在其内。一条记录交出的落点：它点名的每个单元的两条位置条目（跨度按单元种类），
/// 与它新根段里四条指针（树表、中央映射树根、实例表、分配记录树根；空指针不算）的位置条目——一次发布的每条记录都带整次发布的新根段
/// （已定项 15），末条读不出时前几条的新根段照样说得出那次发布的树表与映射根在哪。落点不在那块盘的单元区里的（坏盘上的记录）不交。
/// 只读内存里的记录、不读盘；读盘只问盘的字节数（落点在不在单元区）。那几次发布的账里的落点另由
/// [`placements_in_the_accounts_of_unapplied_publishes`] 读盘交出，调用方（`rebuilt_allocator`）并进同一份。
#[allow(
    clippy::ptr_arg,
    reason = "placement_lies_in_the_unit_area_of_its_device 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn placements_named_by_unapplied_journal_records<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
    let mut named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> = BTreeMap::new();
    for record in records.values().filter(|record| {
        (record.instance, record.checkpoint_txg)
            > (selected_version.instance, selected_version.checkpoint_txg)
    }) {
        let new_root_segment_pointers = [
            (&record.new_tree_table, TransactionUnit::TreeTable),
            (&record.new_mapping_root, TransactionUnit::MappingTree),
            (&record.new_instance_table, TransactionUnit::InstanceTable),
            (
                &record.new_allocation_record_tree_root,
                TransactionUnit::AllocationTree,
            ),
        ]
        .into_iter()
        .filter(|(pointer, _)| **pointer != crate::pointer::NodePointer::empty_root())
        .flat_map(|(pointer, unit)| {
            pointer
                .locations
                .iter()
                .map(move |location| (*location, unit.span_slots()))
        });
        let named_units = record.named.iter().flat_map(|named| {
            let span_slots = crate::recovery::span_in_slots_of_a_named_unit(named);
            named
                .locations
                .iter()
                .map(move |location| (*location, span_slots))
        });
        let placements: Vec<PlacementOnDevice> = named_units
            .chain(new_root_segment_pointers)
            .filter(|(location, span_slots)| {
                placement_lies_in_the_unit_area_of_its_device(
                    devices,
                    unit_area_start,
                    location.device,
                    location.slot,
                    *span_slots,
                )
                .is_ok()
            })
            .map(|(location, span_slots)| PlacementOnDevice {
                device: location.device,
                slot: location.slot,
                span: span_slots,
            })
            .collect();
        named_by_ring_slot
            .entry(target_for_publish(record.checkpoint_txg, slots_per_region))
            .or_default()
            .extend(placements);
    }
    named_by_ring_slot
}

/// C 的账那一份（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K2 T1 C 的盲区」那一行与第三节第 1 条，攻方 `C_ACCOUNT`）：
/// 水位之上每一次没施加的发布（同一 (实例代号, checkpoint_txg) 的记录），用它读得出的记录里计数器最小的那一条的新根段拼一条根
/// （每条记录都带整次发布的新根段，D23（journal 的角色与格式） 已定项 15；[`crate::recovery::root_assembled_from_the_new_root_segment_of_the_record`]），
/// 按影子账同一份读法（[`placements_referenced_by_root`]）读它的账，账里的落点按那次发布的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，
/// 调用方并进 C 的集合、与记录点名的那一份同一个时机解开。账读不出、解不开的那次发布：读得出的记录齐（序号 1..N 连着、带末条）就不交——
/// 那时它的单元多半也验不过，交 F1 乙；不齐就照被抛弃根那一份从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，
/// `C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H3 那一行与第三节第 1 条）。
/// 它补的是 C 只看读得出的记录点名了什么那一处盲区：那次发布的根槽与它的某一条记录（末条、中间一条或第一条）读回全零、单元读得出时，
/// F1 乙不拒，缺的那条记录单独点名的单元又不在别的记录里；账里是那次发布引用的全部落点。水位与 C 同一个（[`placements_named_by_unapplied_journal_records`]）。
///
/// # Errors
/// 那一支现算也不成、重读一次仍不成、现行账里又没有 C694 的 B 那道凭据 ⇒
/// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`
/// （点名那次发布的 (实例代号, checkpoint_txg)），在任何写之前。
#[allow(
    clippy::ptr_arg,
    clippy::too_many_arguments,
    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
)]
fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>, MountError> {
    // 记录表按 (实例代号, 计数器) 升序：同一次发布里先遇到的就是计数器最小的那一条。
    let mut readable_records_of_each_publish: BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        Vec<&crate::journal::JournalRecord>,
    > = BTreeMap::new();
    for record in records.values().filter(|record| {
        (record.instance, record.checkpoint_txg)
            > (selected_version.instance, selected_version.checkpoint_txg)
    }) {
        readable_records_of_each_publish
            .entry((record.instance, record.checkpoint_txg))
            .or_default()
            .push(record);
    }
    let mut in_the_accounts_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> =
        BTreeMap::new();
    for ((_instance, checkpoint_txg), records_of_the_publish) in readable_records_of_each_publish {
        let first_readable_record = records_of_the_publish
            .first()
            .expect("按发布归组时每一组至少放进了一条记录");
        let root_of_the_publish =
            crate::recovery::root_assembled_from_the_new_root_segment_of_the_record(
                first_readable_record,
                selected_version.filesystem_identifier,
            );
        let placements =
            match placements_referenced_by_root(devices, unit_area_start, &root_of_the_publish) {
                Some(placements) => placements,
                None if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(
                    &records_of_the_publish,
                ) =>
                {
                    placements_of_an_unapplied_publish_whose_account_is_unreadable(
                        devices,
                        unit_area_start,
                        &root_of_the_publish,
                        current_records,
                        before_the_one_reread,
                    )?
                }
                None => continue,
            };
        in_the_accounts_by_ring_slot
            .entry(target_for_publish(checkpoint_txg, slots_per_region))
            .or_default()
            .extend(
                placements
                    .into_iter()
                    .map(|(device, slot, span)| PlacementOnDevice { device, slot, span }),
            );
    }
    Ok(in_the_accounts_by_ring_slot)
}

/// C 并上的较新可读根的账（`C_NEWER_ROOTS`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1 那一行与第三节第 1 条，
/// 用户 2026-10-08 定「四形都修」）：根环里比所选那一版 `selected_version` 新（择根的序：checkpoint_txg 为主、实例代号破平局）、按最新那条根的实例表
/// 有效（`is_abandoned` 判否）、读得出的根——F2 退回之后被退掉的那条根就是这样：它的根槽读得出、它自己的实例表里它有效，影子账只管被抛弃的根、
/// 不管它；它的记录又全读不出时 C 那两份（记录点名的、记录拼根读的账）都是空的。照影子账同一份读法（[`placements_referenced_by_root`]）读它的账，
/// 账里的落点按它的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，调用方并进 C 的集合、同一个时机解开。账读不出时照没施加的发布那一份办：
/// 那次发布读得出的记录不齐就从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，仍不成拒），记录齐就不交账（交 F1 乙）、
/// 改交挂载那一刻现行账（`current_records`）里仍分配着的落点（丙，用户 2026-10-08 定，函数体里那一支的注释）；
/// 一条记录都读不出的，映射现算得出就交；现算不成时，`journal_instance_in_the_system_configuration`（挂载择到的系统配置里取过号的实例代号，
/// `SystemRuntimeQuantities::journal_instance`）不比那条根新就照记录不齐那一支办（仍不成拒），比它新就不交也不拒（函数体里那一支的注释）。
/// `roots` 是重建分配器读根环那一遍读出的根（影子账、根环表用的同一份）。
///
/// # Errors
/// 同 [`placements_in_the_accounts_of_unapplied_publishes`]（点名那条根的 (实例代号, checkpoint_txg)）。
#[allow(
    clippy::ptr_arg,
    clippy::too_many_arguments,
    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、根环里读出的根、判被抛弃的谓词、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、系统配置里取过号的实例代号、重读前的钩子"
)]
fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
    current_records: &[AllocationRecord],
    journal_instance_in_the_system_configuration: InstanceGeneration,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<PlacementsInTheAccountsOfNewerReadableRoots, MountError> {
    let mut in_the_accounts_of_newer_roots_by_ring_slot: BTreeMap<
        RootRingSlot,
        Vec<PlacementOnDevice>,
    > = BTreeMap::new();
    let mut txgs_of_the_roots_taking_the_current_account = Vec::new();
    for root in roots.iter().filter(|root| {
        !is_abandoned(root)
            && (root.checkpoint_txg, root.instance)
                > (selected_version.checkpoint_txg, selected_version.instance)
    }) {
        let placements_in_the_newer_roots_account =
            match placements_referenced_by_root(devices, unit_area_start, root) {
                Some(placements) => placements,
                None => {
                    let readable_records_of_its_publish: Vec<&crate::journal::JournalRecord> =
                        records
                            .values()
                            .filter(|record| {
                                (record.instance, record.checkpoint_txg)
                                    == (root.instance, root.checkpoint_txg)
                            })
                            .collect();
                    if readable_records_of_its_publish.is_empty() {
                        // 那次发布一条记录都读不出：映射现算得出就并进来（攻方 H1 里账读不出的那几格要它）。现算不成时看系统配置里取过号的
                        // 实例代号：不比那条根新，照「记录不齐」那一支办（重读一次、看 C694 的 B 凭据，仍不成拒，攻方
                        // `research/prompts/m3-batch37-43-code-r1-opus-output.md` 第一节 Y2，臂 `OPUS_ARM_REFUSE_UNMAPPED=2`，用户 2026-10-08 定「两形都修」）——
                        // 那条根之后没有别的挂载取过号，它的单元还在，不拒就会在写行那次发布里写掉它们；比那条根新，不拒、不重读——
                        // 那条根之后已有一次挂载取过号、写掉了它的单元、记录槽也被那次挂载的记录占了（入库用例 `urJ` 那一形），映射本来就现算不成，
                        // 拒了就再也做不成可写挂载。
                        match placements_referenced_by_root_computed_from_the_mapping(
                            devices,
                            unit_area_start,
                            root,
                        ) {
                            Some(computed) => computed,
                            None if journal_instance_in_the_system_configuration
                                <= root.instance =>
                            {
                                placements_of_an_unapplied_publish_whose_account_is_unreadable(
                                    devices,
                                    unit_area_start,
                                    root,
                                    current_records,
                                    before_the_one_reread,
                                )?
                            }
                            None => continue,
                        }
                    } else if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(
                        &readable_records_of_its_publish,
                    ) {
                        placements_of_an_unapplied_publish_whose_account_is_unreadable(
                            devices,
                            unit_area_start,
                            root,
                            current_records,
                            before_the_one_reread,
                        )?
                    } else {
                        // 记录齐：账不交（交 F1 乙），C 那一槽另并上挂载那一刻现行账里仍分配着的落点（丙，D16（发布语义） 已定项 1
                        // 「根槽这一次读坏」那一行末句，用户 2026-10-08 定「改成丙」，判决
                        // `research/prompts/m3-batch10z-d-code-r1-main-verification.md` 第三节；攻方改法丙只在攻方模型上量过、被攻过零轮）。
                        // 这条根建在所选那一版上：它引用的单元要么是它自己写的（记录点名了，C 那一份里有），要么在所选那一版的账里，
                        // 也就是现行账里这一截。它们这时分配着、影子账照豁免（第九项不数两遍）；这次挂载里释放之后，抬 F 借 C 那一槽
                        // （写行之后这条根被抛弃，H2）与进程内回收（`allocator::PoolAllocator::record_root_written_by_this_process`）照这一份补隔离。
                        // 不并它时，抬 F 只借到记录点名的那几个，它从所选那一版继承、被会话释放的单元回收出去（攻方 K2-2，I-7.4 红）。
                        // 这一句的前提有两处不成立（判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 两行，
                        // 用户 2026-10-08 定「丙继续补洞」；两处补法的形是第 10z 批第十块实现员定的，被攻过零轮）：
                        // 一、这条根与所选那一版之间还有别的发布（攻方一形）：它从那几次继承的单元只记在 C 里那几次的槽上，
                        //    调用方照交回的 txg 把那几槽并进这条根那一槽（[`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`]）；
                        // 二、这条根是一次管理员回退（正推一形）：它复活的单元在现行账里是已释放，这里另并上
                        //    [`placements_a_rollback_publish_revived_released_in_the_current_account`]；认不出是不是回退、
                        //    或认出回退而复活集算不出时，那里退到现行账里全部已释放的落点（tri，用户 2026-10-08 定）。
                        txgs_of_the_roots_taking_the_current_account.push(root.checkpoint_txg);
                        let mut placements =
                            placements_still_allocated_in_the_account(current_records);
                        placements.extend(
                            placements_a_rollback_publish_revived_released_in_the_current_account(
                                devices,
                                unit_area_start,
                                root,
                                &readable_records_of_its_publish,
                                roots,
                                is_abandoned,
                                current_records,
                            ),
                        );
                        placements
                    }
                }
            };
        in_the_accounts_of_newer_roots_by_ring_slot
            .entry(target_for_publish(root.checkpoint_txg, slots_per_region))
            .or_default()
            .extend(
                placements_in_the_newer_roots_account
                    .into_iter()
                    .map(|(device, slot, span)| PlacementOnDevice { device, slot, span }),
            );
    }
    Ok(PlacementsInTheAccountsOfNewerReadableRoots {
        by_ring_slot: in_the_accounts_of_newer_roots_by_ring_slot,
        txgs_of_the_roots_taking_the_current_account,
    })
}

/// [`placements_in_the_accounts_of_newer_readable_roots`] 交回的两样。
struct PlacementsInTheAccountsOfNewerReadableRoots {
    /// 较新可读根的账里的落点，按那条根的 txg 在根环里对应的槽（[`target_for_publish`]）归组。
    by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
    /// 账读不出而记录齐、那一槽改交现行账里仍分配着的落点（丙）的那几条根的 txg，按根环里读出的次序：调用方把所选那一版与它之间
    /// 那几次发布在 C 里记着的落点并进它那一槽（[`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`]）。
    txgs_of_the_roots_taking_the_current_account: Vec<CheckpointTxg>,
}

/// 丙并中间（攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-output.md` 第一节改法表第一行 `chain`，判决
/// `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 攻方一形；第 10z 批第十块实现员定形，被攻过零轮）：
/// 走了丙那一支的根 R（`txgs_of_the_roots_taking_the_current_account`）与所选那一版之间还有别的发布 Q 时，R 从 Q 继承的单元既不是 R 写的、
/// 也不在所选那一版的账里，只记在 C 里 Q 那一槽；这个进程写掉 Q 那一槽时那一份解开，R 那一槽要晚几个 txg 才写掉，其间它们发得出去。
/// 这里把 (所选那一版, R) 之间每个 txg 在根环里对应的槽在 C 里记着的落点（`named_by_ring_slot`，C 的三份并好之后那一份）并进 R 那一槽，
/// 已在 R 那一槽里的（盘、起点槽、跨度都同）不再并：它们就随 R 那一槽一起解开。中间的 txg 至多取一整圈（环长 3S 个），
/// 两条根相差超过一圈时环里每一槽都已取到。只算内存，不读盘。
fn with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
    named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
    selected_version_txg: CheckpointTxg,
    txgs_of_the_roots_taking_the_current_account: &[CheckpointTxg],
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
    let ring_length_in_txgs =
        usize::try_from(ROOT_RING_REGIONS * slots_per_region.count()).expect("环长 3S ≤ 48");
    let mut chained = named_by_ring_slot.clone();
    for newer_root_txg in txgs_of_the_roots_taking_the_current_account {
        let ring_slot_of_the_newer_root = target_for_publish(*newer_root_txg, slots_per_region);
        let ring_slots_in_between: BTreeSet<RootRingSlot> = (selected_version_txg.0 + 1
            ..newer_root_txg.0)
            .rev()
            .take(ring_length_in_txgs)
            .map(|txg_in_between| {
                target_for_publish(CheckpointTxg(txg_in_between), slots_per_region)
            })
            .filter(|ring_slot| *ring_slot != ring_slot_of_the_newer_root)
            .collect();
        let on_the_ring_slot_of_the_newer_root =
            chained.entry(ring_slot_of_the_newer_root).or_default();
        let mut already_there: BTreeSet<(DeviceIdentity, SlotNumber, u64)> =
            on_the_ring_slot_of_the_newer_root
                .iter()
                .map(|placement| (placement.device, placement.slot, placement.span))
                .collect();
        for placement in ring_slots_in_between
            .iter()
            .filter_map(|ring_slot| named_by_ring_slot.get(ring_slot))
            .flatten()
        {
            if already_there.insert((placement.device, placement.slot, placement.span)) {
                on_the_ring_slot_of_the_newer_root.push(*placement);
            }
        }
    }
    chained
}

/// 回退复活（判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 正推一形，用户 2026-10-08 定「丙继续补洞」；
/// 第 10z 批第十块实现员定形，被攻过零轮）：走了丙那一支的根 `newer_root` 是一次管理员回退（向前发布）时，它从回退目标 R_old 取用户可见树，
/// R_old 引用、现行账记成已释放的单元被它复活；回退发布不写它们，记录不点名，现行账里又是已释放，丙那一截滤掉了它们。
/// 认回退照那次发布读得出的记录点名的打包单元（类标签 3）分三种结局（[`rollback_recognized_among_the_packed_units_named_by_a_publish`]）：
/// - 是回退：解开的谱系片（打包记录类型 5）里有一条 rollback_txg 等于这条根 txg 的谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条：
///   回退那次发布追加一条 (T_rb, T_old)），T_old 就是回退目标的 txg。根环里读出、按最新那条根的实例表有效、txg 等于 T_old 的根就是 R_old，
///   读它的账（影子账同一份读法 [`placements_referenced_by_root`]）。交回 R_old 账里仍分配着、现行账里记成已释放的落点（按盘与起点槽对上）：
///   只护复活的那一截，不并现行账里别的已释放落点（那是攻方乙那一类，第四块判决第二节 K2-2 改法行：384 槽盘上隔离约 370 个槽、600 段里 588 段被拒）。
/// - 肯定不是回退：点名的打包单元全读得出、解得开，里面没有这条谱系记录：交空，一槽不多。
/// - 认不出时宽护（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节；
///   判法照正推 tri 臂 `research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff`，细节第 10z 批第十二块实现员定，被攻过零轮）：
///   没认出回退而点名的打包单元有读不出、解不开的（不知道是不是回退；删除发布的墓碑片读不出也走到这里），或认出回退而根环里没读出 R_old
///   （它的根槽读不出，或它已不在环里）、R_old 的账读不出（是回退、复活集算不出），都交回现行账里全部已释放的落点（正推的对照臂那一截）。
///   宽护那一截与别的 C 落点同一个时机解开（这个进程写掉这条根那一槽时）。
///
/// 只读盘，不改分配器。
#[allow(
    clippy::ptr_arg,
    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn placements_a_rollback_publish_revived_released_in_the_current_account<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    newer_root: &RootRecord,
    readable_records_of_its_publish: &[&crate::journal::JournalRecord],
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    current_records: &[AllocationRecord],
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    let all_released_in_the_current_account = || -> Vec<(DeviceIdentity, SlotNumber, u64)> {
        current_records
            .iter()
            .filter(|record| record.is_released)
            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))
            .collect()
    };
    let rollback_target_txg = match rollback_recognized_among_the_packed_units_named_by_a_publish(
        newer_root.checkpoint_txg,
        &packed_units_named_by_the_records_of_a_publish_as_read(
            devices,
            readable_records_of_its_publish,
        ),
    ) {
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(rollback_target_txg) => {
            rollback_target_txg
        }
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback => return Vec::new(),
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback => {
            return all_released_in_the_current_account();
        }
    };
    let accounts_of_the_rollback_target: Option<Vec<Vec<(DeviceIdentity, SlotNumber, u64)>>> =
        roots
            .iter()
            .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)
            .map(|rollback_target| {
                placements_referenced_by_root(devices, unit_area_start, rollback_target)
            })
            .collect();
    match accounts_of_the_rollback_target {
        Some(accounts) if !accounts.is_empty() => {
            let released_in_the_current_account: BTreeSet<(DeviceIdentity, SlotNumber)> =
                current_records
                    .iter()
                    .filter(|record| record.is_released)
                    .map(|record| (record.device, record.slot))
                    .collect();
            accounts
                .into_iter()
                .flatten()
                .filter(|(device, slot, _)| {
                    released_in_the_current_account.contains(&(*device, *slot))
                })
                .collect()
        }
        // 根环里没读出 R_old（它的根槽读不出，或它已不在环里），或 R_old 的账读不出：是回退、复活集算不出（tri 的第二种退法）。
        Some(_) | None => all_released_in_the_current_account(),
    }
}

/// 一次发布读得出的记录点名的一个打包单元（类标签 3）读成了什么。
enum PackedUnitNamedByAPublishAsRead {
    /// 两条位置条目都读不出，或读出来整单元校验和对不上条目里的。
    UnreadableOrChecksumMismatch,
    /// 读出来校验和对得上，`parse_packed_unit` 解不开。
    ReadButUnparsable,
    /// 解开了：打包记录类型与逐条记录的字节。
    Parsed {
        record_type: u16,
        records: Vec<Vec<u8>>,
    },
}

/// 一次发布读得出的记录点名的每个打包单元（类标签 3），按记录与点名的次序，经位置条目里的校验和读、解。只读盘。
#[allow(
    clippy::ptr_arg,
    reason = "read_unit_via_locations 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn packed_units_named_by_the_records_of_a_publish_as_read<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    readable_records_of_the_publish: &[&crate::journal::JournalRecord],
) -> Vec<PackedUnitNamedByAPublishAsRead> {
    let packed_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    readable_records_of_the_publish
        .iter()
        .flat_map(|record| record.named.iter())
        .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)
        .map(
            |named| match read_unit_via_locations(devices, &named.locations, packed_unit_bytes) {
                Err(_unreadable) => PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,
                Ok(bytes) => match crate::unit::parse_packed_unit(&bytes) {
                    Err(_unparsable) => PackedUnitNamedByAPublishAsRead::ReadButUnparsable,
                    Ok(packed) => PackedUnitNamedByAPublishAsRead::Parsed {
                        record_type: packed.identity.record_type,
                        records: packed.records,
                    },
                },
            },
        )
        .collect()
}

/// 认回退的三种结局（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节）。
#[derive(Debug, PartialEq, Eq)]
enum RollbackRecognizedAmongThePackedUnitsNamedByAPublish {
    /// 解开的谱系片（打包记录类型 5）里有一条 rollback_txg 等于这次发布 txg 的谱系记录：这次发布是一次管理员回退，回到 T_old 那一版。
    RollbackTo(CheckpointTxg),
    /// 点名的打包单元全读得出、解得开，谱系片里的记录也全解得开，没有那一条：肯定不是回退。
    NotARollback,
    /// 没认出回退，而点名的打包单元有读不出、校验和对不上、解不开的，或谱系片里有一条记录解不开：不知道是不是回退。
    UnknownWhetherARollback,
}

/// 照一次发布的记录点名的打包单元读成的样子认回退（D18（块里携带什么信息） 已定项 24 第 5 条：回退那次发布追加一条 (T_rb, T_old)，
/// 装在它写的谱系片里、记录点名）。只算内存。读出来的里面认出了就是回退，不管别的几片读不读得出；没认出时，有一片读不出、解不开就是不知道。
fn rollback_recognized_among_the_packed_units_named_by_a_publish(
    checkpoint_txg_of_the_publish: CheckpointTxg,
    packed_units_as_read: &[PackedUnitNamedByAPublishAsRead],
) -> RollbackRecognizedAmongThePackedUnitsNamedByAPublish {
    let mut some_named_packed_unit_unknown = false;
    for packed_unit in packed_units_as_read {
        match packed_unit {
            PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch
            | PackedUnitNamedByAPublishAsRead::ReadButUnparsable => {
                some_named_packed_unit_unknown = true;
            }
            PackedUnitNamedByAPublishAsRead::Parsed {
                record_type,
                records,
            } => {
                if *record_type != crate::unit::PACKED_TYPE_LINEAGE {
                    continue;
                }
                for record_bytes in records {
                    match LineageRecord::parse(record_bytes) {
                        Some(lineage) if lineage.rollback_txg == checkpoint_txg_of_the_publish => {
                            return RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(
                                lineage.rollback_target_txg,
                            );
                        }
                        Some(_lineage_of_another_publish) => {}
                        None => some_named_packed_unit_unknown = true,
                    }
                }
            }
        }
    }
    if some_named_packed_unit_unknown {
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback
    } else {
        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback
    }
}

/// 一份账里仍分配着的每个落点（盘、起点槽、跨度），按记录的次序。只读内存。
fn placements_still_allocated_in_the_account(
    records: &[AllocationRecord],
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    records
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device, record.slot, u64::from(record.span_slots)))
        .collect()
}

/// 水位之上一次没施加的发布（或比所选那一版新的可读根）的账读不出、那次发布读得出的记录又不齐时，它引用的落点照被抛弃根那一份办
/// （`C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 1 条）：
/// 从中央映射现算、重读一次、看 C694 的 B 那道凭据（[`placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`]，
/// 与被抛弃根那一份共用）。
/// 记录齐的那几次发布不走这里（调用方判）：那时账读不出交 F1 乙，照被抛弃根那一份办会在单元全读不出、映射也现算不成的正常盘面上误拒。
///
/// # Errors
/// 映射现算不成、重读一次仍不成、凭据也没有 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`，
/// `unapplied_publish` 点名 `root_of_the_publish` 的 (实例代号, checkpoint_txg)；与 C393 那一拒分开报（用户 2026-10-08 定「乙：core 另起错误成员」），
/// 在任何写之前。
#[allow(
    clippy::ptr_arg,
    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn placements_of_an_unapplied_publish_whose_account_is_unreadable<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    root_of_the_publish: &RootRecord,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<Vec<(DeviceIdentity, SlotNumber, u64)>, MountError> {
    placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread(
        devices,
        unit_area_start,
        root_of_the_publish,
        current_records,
        before_the_one_reread,
    )
    .ok_or_else(|| {
        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
            StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                unapplied_publish: RollbackTarget {
                    instance: root_of_the_publish.instance,
                    checkpoint_txg: root_of_the_publish.checkpoint_txg,
                },
            },
        ))
    })
}

/// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，
/// 不读分配记录、不读树表，按这条根现算它引用的全部落点：根记录直接指着的豁免单元（实例表链上认得出的每一片、
/// 树表、中央映射树的每个节点）加中央映射树里每一条映射条目的落点（数据单元、打包单元、四棵树的节点，含分配记录树自己的节点）。
/// 树表自己的落点取根记录里那条指针。只在内存里用，不写盘。中央映射树读不出、解不开（映射根是空指针的 mkfs 第 0 代、树表 0 条的一版也在内），
/// 映射条目解不开、落点不同槽或不在单元区：交 `None`。
#[allow(
    clippy::ptr_arg,
    reason = "read_unit_via_locations 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn placements_referenced_by_root_computed_from_the_mapping<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let mapping_tree = crate::code_two_tree::read_code_two_tree(
        &root.mapping_root,
        &crate::transaction::MultiLevelCodeTwoTree::CentralMapping
            .read_expectation(root.mapping_root.head.birth_tree),
        crate::code_two_tree::CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,
        root,
        crate::unit::unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &crate::pointer::NodePointer| {
            read_unit_via_locations(devices, &pointer.locations, node_bytes)
        },
    )
    .ok()?;
    let mut located: Vec<([LocationEntry; 2], u64)> = Vec::new();
    for page in instance_table_page_pointers_as_far_as_readable(devices, root) {
        located.push((page.locations, TransactionUnit::InstanceTable.span_slots()));
    }
    located.push((
        root.tree_table.locations,
        TransactionUnit::TreeTable.span_slots(),
    ));
    for pointer in &mapping_tree.version.pointers {
        located.push((pointer.locations, TransactionUnit::MappingTree.span_slots()));
    }
    for entry in &mapping_tree.leaf_entries_in_key_order {
        let (key, locations) = crate::records::parse_mapping_entry(entry)?;
        let span = match key.first().copied()? {
            crate::unit::UNIT_CLASS_DATA | crate::unit::UNIT_CLASS_PACKED => 2,
            crate::unit::UNIT_CLASS_INDEX_NODE => 1,
            _ => return None,
        };
        located.push((locations, span));
    }
    let mut placements = Vec::new();
    for (locations, span) in located {
        let slot = slot_shared_by_both_location_entries(&locations).ok()?;
        for (identity, _) in devices {
            placement_lies_in_the_unit_area_of_its_device(
                devices,
                unit_area_start,
                *identity,
                slot,
                span,
            )
            .ok()?;
            placements.push((*identity, slot, span));
        }
    }
    Some(placements)
}

/// 一条根引用着的落点，逐盘：带文件的一版从它那一版的分配记录里取（账里已释放的是它换下的上一版的，不算它引用）；
/// **树表 0 条的一版**引用的就是根记录直接指着的那几个单元——实例表（连同链上后面各片）、树表，写过行的一版还有它自己那片
/// 分配记录树节点（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））——两盘同槽，不读分配记录树：
/// 那一片读不出时它自己的落点照样认得出。读不出、解不开交回 `None`，怎么办归调用方：影子账读被抛弃根计一次读不出、从中央映射现算，
/// 仍不成重读一次，再不成看 B 的凭据放行或拒（[`placements_of_an_abandoned_root_whose_account_is_unreadable`]）；候选根读不出就当它什么都不豁免；
/// C 的账那一份读不出就不加（[`placements_in_the_accounts_of_unapplied_publishes`]）；
/// 解不开包括根记录里那几条指针的两条位置条目不同槽、槽号与跨度不在那块盘的单元区里（代码审阅第 32 条）。
///
/// 树表 0 条那一臂不能省：写过行的树表 0 条根指着的是一片新实例表，它既不在任何分配记录树里、也不在 mkfs 那两个单元里，
/// 少了这一臂它在影子账里就成了「账读不出」的被抛弃根，那片实例表不被隔离、也不在重建出来的账里，
/// 挂载之后当空闲槽发出去，把根环里还引用着它的那条根读坏（2026-09-23 崩溃注入打中：那时的挂载时回退里
/// I-2.1 / I-4.8 / I-7.4 三条红在「最新根引用的单元已被复用」；崩溃恢复落到树表 0 条的根时是同一格）。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn placements_referenced_by_root<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {
    match tree_table_has_no_entries(devices, root) {
        Ok(true) => {
            let mut placements = Vec::new();
            // 实例表是一条链（D18（块里携带什么信息） 已定项 11）：认得出的每一片都是这条根引用的，只有一片时就是根记录里那一条。
            // 读不全的链照样把认得出的那几片算上——少算一片，被抛弃根的那一片就不隔离、挂载之后可能被发出去。
            let instance_table_pages =
                instance_table_page_pointers_as_far_as_readable(devices, root);
            // 写过行的一版那棵分配记录树的节点同样是这条根引用的单元（D23（journal 的角色与格式） 已定项 14：被抛弃时间线的根离开根环之前，
            // 它们引用的单元不许重新分配）；漏了它们，被抛弃根的这几片既不隔离、也不在所选那一版的账里，挂载之后是空闲槽。
            // 树可以多层（D8（核心索引结构） 已定项 14）：读得出多少认多少，读不出的节点它自己那一片照样认（父条目里有它的指针）。
            // mkfs 的第 0 代与照抄它的暖机根那一项全零，没有这棵树。
            let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
            let allocation_record_tree_nodes: Vec<crate::pointer::NodePointer> =
                if root.allocation_record_tree_root == crate::pointer::NodePointer::empty_root() {
                    Vec::new()
                } else {
                    node_pointers_as_far_as_readable(
                        &root.allocation_record_tree_root,
                        &AllocationRecordTreeGeometry::of_reader(devices),
                        &mut |pointer: &crate::pointer::NodePointer| {
                            read_unit_via_locations(devices, &pointer.locations, node_bytes).ok()
                        },
                    )
                };
            for (pointer, unit) in instance_table_pages
                .iter()
                .map(|page_pointer| (page_pointer, TransactionUnit::InstanceTable))
                .chain([(&root.tree_table, TransactionUnit::TreeTable)])
                .chain(
                    allocation_record_tree_nodes
                        .iter()
                        .map(|pointer| (pointer, TransactionUnit::AllocationTree)),
                )
            {
                let slot = slot_shared_by_both_location_entries(&pointer.locations).ok()?;
                for (identity, _) in devices {
                    // 槽号是盘上读来的 6 字节：进分配器（`PoolAllocator::isolate_abandoned` 与根环表里清隔离那一步）之前判它在这块盘的单元区里，
                    // 不在就与两条位置条目不同槽同一个结局——这条根的账解不开，调用方只计数（代码审阅第 32 条；改之前是
                    // `DeviceFreeMap::index` 的 expect 与 `isolate` 的跨度断言 panic）。
                    placement_lies_in_the_unit_area_of_its_device(
                        devices,
                        unit_area_start,
                        *identity,
                        slot,
                        unit.span_slots(),
                    )
                    .ok()?;
                    placements.push((*identity, slot, unit.span_slots()));
                }
            }
            Some(placements)
        }
        Ok(false) => Some(
            allocation_records_under_root_in_the_unit_area_starting_at(
                devices,
                unit_area_start,
                root,
            )
            .ok()?
            .into_iter()
            .filter(|record| !record.is_released)
            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))
            .collect(),
        ),
        Err(_failure) => None,
    }
}

/// 挂载重建分配器与进程内重读时读一个根环槽读到了什么：在 [`read_root_ring_slot`] 的分法之上，把读回全零的槽单分出来
/// （用户 2026-10-04 定：盘上全零的根槽不并进准入抬 F 的上限；2026-10-05 定乙：回收门槛里照「自证不过」那一类钉住，
/// 一次读回全零的槽盘上可能住着一条有效根）。
/// 别的读者（择根、算上限那一遍、清扫）照旧用 [`read_root_ring_slot`]：清扫把全零与读不出同待遇（D18（块里携带什么信息） 已定项 11）。
#[allow(
    clippy::large_enum_variant,
    reason = "一次读一个槽、按值交回马上拆开，装箱只多一次分配"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootRingSlotReadingTellingNeverWrittenApart {
    SelfVerified(RootRecord),
    /// 设备报读错，或读得出、不是全零而自证不过（校验和不过、fsid 不是本池的）。
    BadAndNotAllZero,
    /// 读得出、每个字节都是 0：多半从没写过，也可能是一次读坏的有效根（回收门槛照这一种钉住）。
    AllZeroNeverWritten,
}

/// 读一个根环槽，只读一次、不重试：读不出、全零、自证过与否按 [`RootRingSlotReadingTellingNeverWrittenApart`] 分。偏移与盘的取法与
/// [`read_root_ring_slot`] 相同（区域归属表给出盘、槽距给出偏移）；要看字节是不是全零，所以不经它读（经它再读一次，两次之间的瞬时读错
/// 会让两个读数对不上）。
fn read_root_ring_slot_telling_never_written_apart<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slot: RootRingSlot,
) -> RootRingSlotReadingTellingNeverWrittenApart {
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset =
        crate::root_ring::slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
    let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
        return RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero;
    };
    if bytes.iter().all(|byte| *byte == 0) {
        return RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten;
    }
    match RootRecord::parse_slot(&bytes, filesystem_identifier) {
        Some(root) => RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root),
        None => RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero,
    }
}

/// 一个根环槽两遍读数里较好的那一份（用户 2026-10-05 定 R1「按槽并两遍」，判决 `research/prompts/m3-batch10-code-r2-main-verification.md`
/// 第六节）：任一遍自证过的根（两遍都自证过取重读那一条）＞ 任一遍读坏、不是全零 ＞ 全零。
/// 自证过的根不是读错造得出来的（整槽校验和过、fsid 是本池的），挂载期间又没有别的写者，第一遍读出的那一条重读时仍住在那一槽：
/// 只拿重读那一遍、丢掉第一遍读出的根，第一遍读得出、重读读坏的被抛弃根会掉出影子账，它独占的单元当场发得出去
/// （攻方第二次抽样 `research/prompts/m3-batch10-code-r2-opus-s2-output.md` K2-2，违反 D23（journal 的角色与格式） 已定项 14
/// 「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」）。读坏压过全零：读坏那一类回收门槛钉住、准入抬 F 的上限也并，
/// 全零那一类只在回收门槛里钉住（用户 2026-10-05 定乙），两类里读坏的约束多。
fn better_reading_of_a_root_ring_slot_over_the_two_passes(
    first_reading: RootRingSlotReadingTellingNeverWrittenApart,
    reading_of_the_reread: RootRingSlotReadingTellingNeverWrittenApart,
) -> RootRingSlotReadingTellingNeverWrittenApart {
    use RootRingSlotReadingTellingNeverWrittenApart::{
        AllZeroNeverWritten, BadAndNotAllZero, SelfVerified,
    };
    match (first_reading, reading_of_the_reread) {
        (SelfVerified(_), SelfVerified(root_of_the_reread)) => SelfVerified(root_of_the_reread),
        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(root_of_the_reread)) => {
            SelfVerified(root_of_the_reread)
        }
        (SelfVerified(root_of_the_first_pass), BadAndNotAllZero | AllZeroNeverWritten) => {
            SelfVerified(root_of_the_first_pass)
        }
        (BadAndNotAllZero, BadAndNotAllZero) => BadAndNotAllZero,
        (BadAndNotAllZero, AllZeroNeverWritten) | (AllZeroNeverWritten, BadAndNotAllZero) => {
            BadAndNotAllZero
        }
        (AllZeroNeverWritten, AllZeroNeverWritten) => AllZeroNeverWritten,
    }
}

/// 重建分配器读根环的两遍（[`every_root_ring_slot`] 的次序，每槽各读一次）逐槽并成一份
/// （[`better_reading_of_a_root_ring_slot_over_the_two_passes`]）。两遍按同一个次序读同一组槽，槽对不上是这里的 bug、断言。
fn merged_readings_of_the_two_root_ring_passes(
    first_readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
    readings_of_the_reread: Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)>,
) -> Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)> {
    assert_eq!(
        first_readings.len(),
        readings_of_the_reread.len(),
        "两遍都按 every_root_ring_slot 读整个环，槽数相同"
    );
    first_readings
        .iter()
        .zip(readings_of_the_reread)
        .map(
            |((ring_slot_of_the_first_pass, first_reading), (ring_slot, reading_of_the_reread))| {
                assert_eq!(
                    *ring_slot_of_the_first_pass, ring_slot,
                    "两遍都按 every_root_ring_slot 的次序读，第 n 个读数是同一个槽"
                );
                (
                    ring_slot,
                    better_reading_of_a_root_ring_slot_over_the_two_passes(
                        *first_reading,
                        reading_of_the_reread,
                    ),
                )
            },
        )
        .collect()
}

/// 重建分配器读根环的两遍里只有一遍自证过的槽（另一遍读不出、读得出而自证不过或读回全零）：并出来是那一遍读出的根
/// （[`better_reading_of_a_root_ring_slot_over_the_two_passes`]），根环表把它另记一类（用户 2026-10-06 定 S1，
/// `allocator::RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`）。两遍按同一个次序读同一组槽，槽对不上是这里的 bug、断言。
fn ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(
    first_readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
    readings_of_the_reread: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
) -> BTreeSet<RootRingSlot> {
    use RootRingSlotReadingTellingNeverWrittenApart::{
        AllZeroNeverWritten, BadAndNotAllZero, SelfVerified,
    };
    assert_eq!(
        first_readings.len(),
        readings_of_the_reread.len(),
        "两遍都按 every_root_ring_slot 读整个环，槽数相同"
    );
    first_readings
        .iter()
        .zip(readings_of_the_reread)
        .filter_map(
            |((ring_slot_of_the_first_pass, first_reading), (ring_slot, reading_of_the_reread))| {
                assert_eq!(
                    ring_slot_of_the_first_pass, ring_slot,
                    "两遍都按 every_root_ring_slot 的次序读，第 n 个读数是同一个槽"
                );
                match (first_reading, reading_of_the_reread) {
                    (SelfVerified(_), BadAndNotAllZero | AllZeroNeverWritten)
                    | (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(_)) => Some(*ring_slot),
                    (SelfVerified(_), SelfVerified(_))
                    | (
                        BadAndNotAllZero | AllZeroNeverWritten,
                        BadAndNotAllZero | AllZeroNeverWritten,
                    ) => None,
                }
            },
        )
        .collect()
}

/// 挂载那一刻读坏（设备报读错，或读得出、不是全零而自证不过）的根环槽各能住哪几个 txg（区域内槽位只由 txg 定，
/// D22（单元原子性怎么合成） 已定项 2）：
/// - 可能住过的最旧 txg：落到这个槽、小于 `first_txg` 的最小 txg。回收门槛钉在这里（用户 2026-10-04 定：读不出、自证不过的根槽在回收门槛里
///   钉在它可能住过的最旧 txg，不再只在一圈窗口里取——窗口之外、崩在记录之后根槽之前留在洞里的旧根也可能就住在这一槽里）。
/// - 能住的最大 txg：(N − 3S, first_txg − 1] 里落到这个槽的最大 txg（D16（发布语义） 已定项 1「根环表记最大的」，
///   `research/prompts/m3-ring-r3-main-verification.md` 第二节第 4 行）：准入抬 F 的上限照 Q3 按它并。上沿取新实例第一次发布之前的那个 txg：
///   读坏的槽里可能住着比环里读得出的最新根还新的根（被藏住的最新根、由记录施加出来的那一版那次发布的根）。
///
/// 一个槽在 [0, first_txg) 里一个 txg 都住不了（年轻的环里还没轮到的槽）就不交回。挂载时读回全零的槽也经它算（`rebuilt_allocator`），
/// 那一份只用可能住过的最旧 txg（回收门槛照「自证不过」那一类钉住，用户 2026-10-05 定乙；不并进上限）。
///
/// 循环：最大 txg 只走区间尾 min(3S, 区间长) 个 txg（与 first_txg 多大无关：外来记录的 txg 可以贴着 `u64::MAX`），最旧 txg 只走
/// [0, min(3S, first_txg)) ——`target_for_publish` 只看 txg mod 3S，一个槽最小的 txg 落在第一个周期里、区间里最大的 txg 落在区间尾一个周期里；
/// 轮数至多 6S，跨轮只往按槽的表里写；没有提前出口。
fn txgs_ring_slots_bad_at_mount_can_hold(
    bad_ring_slots: &BTreeSet<RootRingSlot>,
    window_anchor: CheckpointTxg,
    first_txg: CheckpointTxg,
    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
) -> Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)> {
    let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
    let lowest_txg_inside_the_window = window_anchor
        .0
        .checked_sub(ring_length_in_txgs)
        .map_or(0, |below_the_window| below_the_window + 1);
    let interval_length_in_txgs = first_txg.0.saturating_sub(lowest_txg_inside_the_window);
    let txgs_walked_at_the_tail_of_the_interval = interval_length_in_txgs.min(ring_length_in_txgs);
    let mut highest_txg_of_each_bad_slot: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
    // 从小到大走，最后一次碰到的是最大值。
    for txg in first_txg.0 - txgs_walked_at_the_tail_of_the_interval..first_txg.0 {
        let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
        if bad_ring_slots.contains(&ring_slot) {
            highest_txg_of_each_bad_slot.insert(ring_slot, CheckpointTxg(txg));
        }
    }
    let mut oldest_txg_of_each_bad_slot: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
    // 从小到大走，第一次碰到的是最小值。
    for txg in 0..first_txg.0.min(ring_length_in_txgs) {
        let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
        if bad_ring_slots.contains(&ring_slot) {
            oldest_txg_of_each_bad_slot
                .entry(ring_slot)
                .or_insert(CheckpointTxg(txg));
        }
    }
    oldest_txg_of_each_bad_slot
        .into_iter()
        .map(|(ring_slot, oldest_txg_it_can_have_held)| {
            let highest_txg_it_can_hold = *highest_txg_of_each_bad_slot.get(&ring_slot).expect(
                "这个槽在 [0, first_txg) 里住得下一个 txg，区间 (N − 3S, first_txg − 1] 要么从 0 起、要么长过一圈，它在区间里也住得下一个",
            );
            (
                ring_slot,
                TxgsARingSlotBadAtMountCanHold::new(
                    oldest_txg_it_can_have_held,
                    highest_txg_it_can_hold,
                ),
            )
        })
        .collect()
}

/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收（当场可用的只到每块盘上最新有效根那一截、某块盘一条有效根都读不出时
/// 只到环里最旧有效根（RGall），其余扣到这次挂载的暖机落满每块盘，甲），再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
/// 实例表判，D23（journal 的角色与格式） 已定项 14 射程「影子账与按实例表判抛弃留着，理由是崩溃恢复」）。
/// 影子账只住内存，所以每次挂载都要重算（步 4 / 步 5 代码三方第一轮云端攻方腿打中：普通重开一次隔离就归零）。
/// 每块盘的空闲图从择到的系统配置按环长现算的单元区起点起（与偏移 417 那 8 字节相等，择的时候判过；C475（非默认环长下单元区起点取编译期常量））。
/// 生效的回退下界 F 判「有效根」用的也是影子账那一张表（最新那条根的实例表，读不出重读一次）：不另读一遍、也就没有「读不出就不按表滤」
/// 那一支（C554 乙报告 Q6）。
///
/// # Errors
/// 最新那条根的实例表读不出、重读一次仍读不出（[`instance_table_of_the_newest_root_read_at_most_twice`]）——在读阶段之后、取号之前；
/// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）；
/// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有 ⇒
/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）；
/// C 读一次没施加的发布或一条较新可读根的账读不出、那次发布读得出的记录不齐（较新可读根那次发布一条记录都读不出、系统配置里取过号的实例代号
/// 又不比它新的，也算这一支）、从中央映射也现算不成、重读一次仍不成
/// （[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）⇒
/// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`。
/// 都在取号之前，盘上逐字节不变。
#[allow(
    clippy::ptr_arg,
    reason = "choose_root 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：设备表、系统配置、上一版、新实例第一次发布的 txg、读阶段扫出的记录、所选那一版、影子账开关、重读前的钩子"
)]
fn rebuilt_allocator<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    previous: &PreviousVersion,
    first_txg: CheckpointTxg,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
    shadow_ledger: ShadowLedger,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<RebuiltAllocator, MountError> {
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(system_configuration);
    // 盘够不够长到单元区起点，可写挂载在读根环之前判过（`every_device_reaches_the_unit_area_start`，同一个起点）。
    let device_maps: Vec<DeviceFreeMap> = devices
        .iter()
        .map(|(identity, device)| {
            DeviceFreeMap::with_unit_area_start(*identity, device.size_in_bytes(), unit_area_start)
        })
        .collect();
    let mut allocator = match previous {
        PreviousVersion::WithFile { output, .. }
        | PreviousVersion::WithTreesAndNoFile { output, .. } => {
            PoolAllocator::rebuild_from_records(device_maps, output.allocation_records.clone())
        }
        PreviousVersion::WithoutFile { root, .. } => {
            allocator_of_version_without_file(devices, unit_area_start, device_maps, root)?
        }
    };
    let current_records = allocator.records().to_vec();
    let (newest_table, instance_table_of_the_newest_root) =
        instance_table_of_the_newest_root_read_at_most_twice(
            devices,
            system_configuration,
            before_the_one_reread,
        )?;
    let effective_floor = effective_rollback_floor_under_the_newest_roots_table(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        &newest_table,
    );
    // 影子账、回收门槛与分配器那张根环表读的是同一遍根环（每槽读一次）：两遍之间的瞬时读错会让几边看见的根不一样。
    let slots_per_region = system_configuration
        .immutable
        .sizes
        .root_ring_slots_per_region;
    let read_every_root_ring_slot_once =
        || -> Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)> {
            every_root_ring_slot(&system_configuration.immutable.sizes)
                .into_iter()
                .map(|ring_slot| {
                    (
                        ring_slot,
                        read_root_ring_slot_telling_never_written_apart(
                            devices,
                            &system_configuration.immutable.region_devices,
                            &system_configuration.immutable.sizes,
                            &system_configuration.immutable.filesystem_identifier,
                            ring_slot,
                        ),
                    )
                })
                .collect()
        };
    // 窗口锚：这一遍读出的不被抛弃的自证有效根的最大 txg（下面算挂载时读坏、读回全零的槽能住哪几个 txg 用它）。
    let newest_valid_root_txg_of_the_readings =
        |readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)]| {
            readings
                .iter()
                .filter_map(|(_, reading)| match reading {
                    RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => {
                        (!abandoned_by_table(root, &newest_table)).then_some(root.checkpoint_txg)
                    }
                    RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
                    | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
                })
                .max()
        };
    // 一条根归哪块盘按读出它的那个槽所在区域的 `region_devices` 定（下面判「每块盘都读出了有效根」与当场放开到哪儿取 m 都用它，同一处定义）。
    // 不按 txg 的落点公式（`target_for_publish`）：mkfs 在三个区域的槽 0 各写一份第 0 代根（`make_filesystem::genesis_root_ring_slots`），
    // 公式把三份都归到盘 0，环里只剩第 0 代根时盘 1 实际读出了有效根、却算成一条都没有，rr2 照样整环重读（判决
    // `research/prompts/m3-batch10-code-r2-main-verification.md` 第二节 K2 第一、二、三格，攻方第一次抽样 H1）。别的根都按公式写进槽，两种归法只在第 0 代根上不同。
    let device_of_the_ring_slot = |ring_slot: RootRingSlot| {
        system_configuration.immutable.region_devices
            [usize::try_from(ring_slot.region).expect("区域号")]
    };
    let roots_on_the_devices_they_were_read_from = |readings: &[(
        RootRingSlot,
        RootRingSlotReadingTellingNeverWrittenApart,
    )]|
     -> Vec<(DeviceIdentity, RootRecord)> {
        readings
            .iter()
            .filter_map(|(ring_slot, reading)| match reading {
                RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => {
                    Some((device_of_the_ring_slot(*ring_slot), *root))
                }
                RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
                | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
            })
            .collect()
    };
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    // 这一遍每块盘都读出了一条不被抛弃的自证有效根（与下面当场放开到哪儿那一步取 m 同一处定义 [`newest_valid_root_txg_on_every_device`]）。
    let a_valid_root_was_read_on_every_device =
        |readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)]| {
            newest_valid_root_txg_on_every_device(
                &roots_on_the_devices_they_were_read_from(readings),
                &newest_table,
                &all_devices,
            )
            .is_some()
        };
    // 这一遍有一块盘一条不被抛弃的自证有效根都没读出（择根那几遍读得出）就把根环整环再读一遍（用户 2026-10-05 定 Q4 rr2，
    // 岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行：改之前只在整个环一条都没读出时重读，只坏这一遍、只有一块盘没读出的那一形
    // m 取不到、当场只放到环里最旧有效根，窄盘上取不到落点被拒；重读之前调 `before_the_one_reread`，与另几处重读一样），
    // 根环表、回收门槛与影子账都用两遍逐槽并起来的那一份（用户 2026-10-05 定 R1「按槽并两遍」，[`merged_readings_of_the_two_root_ring_passes`]：
    // 只拿重读那一遍时，第一遍读出、重读读坏的被抛弃根掉出影子账；并起来那一份仍有一块盘没读出的，照 m 取不到那一支往下走）；
    // 并起来整个环还是一条都没有，就在任何写之前拒这次可写挂载（用户 2026-10-05 定「重读」，岔路单 `research/prompts/m3-batch17-code-r1-forks.md`
    // 第 2 行；判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节「N1 甲二」那一行与第四节；拒因另起成员，Q3；R1 之后
    // 第一遍读出过一条有效根的不拒）。不重读、给空表往下走时，
    // 写行那次记根算回收门槛只看得到它自己，释放代高于 F_生效 的一截当场放开、被暖机写掉，盘上环里更旧的有效根还引用着（D16（发布语义） 已定项 1
    // 「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）。拒在取号之前，盘上逐字节不变。
    let first_readings = read_every_root_ring_slot_once();
    let window_anchor_of_the_first_readings =
        newest_valid_root_txg_of_the_readings(&first_readings)
            .filter(|_| a_valid_root_was_read_on_every_device(&first_readings));
    // 两遍里只有一遍自证过的槽另记一类（用户 2026-10-06 定 S1，`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`）：
    // 并出来照读出来的根算，抬 F 那一串读它仍坏时拿读出来的那条根顶上、不拒（用户 2026-10-06 定 T1 甲）。只读了一遍的没有这一类（保持空集）。
    let mut ring_slots_self_verified_on_only_one_of_the_two_passes: BTreeSet<RootRingSlot> =
        BTreeSet::new();
    let (ring_slot_readings, window_anchor, root_ring_of_the_rebuilt_allocator) =
        match window_anchor_of_the_first_readings {
            Some(window_anchor) => (
                first_readings,
                window_anchor,
                RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,
            ),
            None => {
                before_the_one_reread.before_rereading();
                let readings_of_the_reread = read_every_root_ring_slot_once();
                ring_slots_self_verified_on_only_one_of_the_two_passes =
                    ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(
                        &first_readings,
                        &readings_of_the_reread,
                    );
                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(
                    &first_readings,
                    readings_of_the_reread,
                );
                let Some(window_anchor) =
                    newest_valid_root_txg_of_the_readings(&readings_of_the_two_passes_merged)
                else {
                    return Err(
                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
                    );
                };
                (
                    readings_of_the_two_passes_merged,
                    window_anchor,
                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
                )
            }
        };
    let roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)> = ring_slot_readings
        .iter()
        .filter_map(|(ring_slot, reading)| match reading {
            RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => {
                Some((*ring_slot, *root))
            }
            RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
            | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
        })
        .collect();
    let bad_ring_slots_not_all_zero: BTreeSet<RootRingSlot> = ring_slot_readings
        .iter()
        .filter_map(|(ring_slot, reading)| match reading {
            RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => Some(*ring_slot),
            RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(_)
            | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
        })
        .collect();
    let ring_slots_reading_back_all_zero: BTreeSet<RootRingSlot> = ring_slot_readings
        .iter()
        .filter_map(|(ring_slot, reading)| match reading {
            RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => Some(*ring_slot),
            RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(_)
            | RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => None,
        })
        .collect();
    let roots: Vec<RootRecord> = roots_with_ring_slots
        .iter()
        .map(|(_, root)| *root)
        .collect();
    let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);
    // 挂载时读坏的槽能住哪几个 txg（[`txgs_ring_slots_bad_at_mount_can_hold`]）：N = 环里读得出的最新有效根的 txg（D16（发布语义） 已定项 1
    // 「挂载时读不出的槽」那一句：挂载重建分配器时取它、不取 first_txg − 1）。读回全零的槽按同一个算法另算一份、只取可能住过的最旧 txg：
    // 回收门槛里照「自证不过」那一类钉住（用户 2026-10-05 定乙），准入抬 F 的上限不并它（用户 2026-10-04 定），能住的最大 txg 用不到。
    // 一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回。窗口锚上面取到了（有一块盘没读出就重读、两遍并起来整个环仍取不到就拒），
    // 两类不会因为它是空表。
    let ring_slots_bad_at_mount = txgs_ring_slots_bad_at_mount_can_hold(
        &bad_ring_slots_not_all_zero,
        window_anchor,
        first_txg,
        slots_per_region,
    );
    let ring_slots_reading_back_all_zero_at_mount: Vec<(RootRingSlot, CheckpointTxg)> =
        txgs_ring_slots_bad_at_mount_can_hold(
            &ring_slots_reading_back_all_zero,
            window_anchor,
            first_txg,
            slots_per_region,
        )
        .into_iter()
        .map(|(ring_slot, txgs_it_can_hold)| {
            (ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held())
        })
        .collect();
    // 环里有效根的最小 txg（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall：不按一圈窗口滤）；
    // 挂载时读坏的槽钉在它可能住过的最旧 txg（用户 2026-10-04 定），读回全零的槽同样钉着（用户 2026-10-05 定乙）。
    let oldest_valid_root_in_the_ring = roots
        .iter()
        .filter(|root| !is_abandoned(root))
        .map(|root| root.checkpoint_txg)
        .chain(
            ring_slots_bad_at_mount
                .iter()
                .map(|(_, txgs_it_can_hold)| txgs_it_can_hold.oldest_txg_it_can_have_held()),
        )
        .chain(
            ring_slots_reading_back_all_zero_at_mount
                .iter()
                .map(|(_, oldest_txg_it_can_have_held)| *oldest_txg_it_can_have_held),
        )
        .min();
    // 卸载半途崩溃之后的下一次可写挂载只放开一截（D16（发布语义） 已定项 1「正常卸载」那一行，用户 2026-10-04 选甲，
    // `research/prompts/m3-batch14-code-r1-forks.md` 第 1 行）：卸载那一串先写了系统配置里的新 F、带新 F 的根没落满每块盘就崩时，
    // F_生效 高于某块盘上最新的有效根；恢复择根不看 F，丢掉另一块盘之后择的就是那块盘上最新的根，它引用的单元释放代可以高于它、不高于 F_生效
    // （判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行）。于是当场可用的只到
    // reclaim_floor(min(F_生效, m), 环里最旧有效根)（RGall），m 是每块盘上最新有效根的 txg 取最小（被抛弃的根不算，与卸载那一格同一处定义
    // [`newest_valid_root_txg_on_every_device`]）；其余到原门槛为止的那一截扣住，这次挂载的暖机在每块盘上都落了根之后放开（`establish_instance`）。
    // 某块盘一条有效根都读不出（m 取不到）时当场只放到环里最旧有效根（用户 2026-10-04 选甲，`research/prompts/m3-batch16-code-r1-forks.md`
    // 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行；用户 2026-10-04 定 RGall
    // 之后取整个环里的有效根、不按一圈窗口滤）：那块盘上真正最新的根这一遍看不到，它引用的单元释放代可以高到 F_生效；它与回收门槛同一个读数
    // （挂载时读坏、读回全零的槽钉在它可能住过的最旧 txg 并进）。第一遍有一块盘没读出就整环重读（Q4 rr2），走到这一支的只剩两遍并起来
    // 仍有一块盘一条有效根都没读出的那一形。这一份读数至少有一条有效根（上面取窗口锚那一步：两遍并起来也一条都没有就拒），
    // 环里最旧有效根恒取得到。m 与上面判要不要重读同一个归盘法（按读出它的槽所在的区域）。
    let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_ring);
    let floor_released_immediately = match newest_valid_root_txg_on_every_device(
        &roots_on_the_devices_they_were_read_from(&ring_slot_readings),
        &newest_table,
        &all_devices,
    ) {
        Some(newest_valid_root_on_every_device) => Some(reclaim_floor(
            effective_floor.min(newest_valid_root_on_every_device),
            oldest_valid_root_in_the_ring,
        )),
        None => oldest_valid_root_in_the_ring,
    };
    if let Some(released_immediately_up_to) = floor_released_immediately {
        allocator.reclaim_released_up_to(released_immediately_up_to, ReclaimedReuse::Immediately);
    }
    allocator.reclaim_released_up_to(
        floor_of_the_reclaim,
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );
    // C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条）：水位之上没施加的记录点名的落点与影子账同一次隔离、
    // 豁免同一个集合。影子账关着（只供测试）时它也不隔离：那一臂要的是「什么都不护」的对照。
    // 同一份里再并上那几次没施加的发布的账里的落点（C 的账那一份，[`placements_in_the_accounts_of_unapplied_publishes`]；账读不出、
    // 读得出的记录不齐时从中央映射现算，仍不成就在这里拒），解开时机相同。
    let placements_named_by_unapplied_journal_records = match shadow_ledger {
        ShadowLedger::On => {
            let mut named_by_ring_slot = placements_named_by_unapplied_journal_records(
                devices,
                unit_area_start,
                records,
                selected_version,
                slots_per_region,
            );
            for (ring_slot, placements_in_the_account) in
                placements_in_the_accounts_of_unapplied_publishes(
                    devices,
                    unit_area_start,
                    records,
                    selected_version,
                    slots_per_region,
                    &current_records,
                    before_the_one_reread,
                )?
            {
                named_by_ring_slot
                    .entry(ring_slot)
                    .or_default()
                    .extend(placements_in_the_account);
            }
            // 再并上环里比所选那一版新、按实例表有效、读得出的根自己的账（`C_NEWER_ROOTS`，[`placements_in_the_accounts_of_newer_readable_roots`]）：
            // F2 退回之后被退掉的那条根，影子账不管它，它的记录又读不出时前两份都是空的。同一个时机解开。
            let PlacementsInTheAccountsOfNewerReadableRoots {
                by_ring_slot: in_the_accounts_of_newer_roots_by_ring_slot,
                txgs_of_the_roots_taking_the_current_account,
            } = placements_in_the_accounts_of_newer_readable_roots(
                devices,
                unit_area_start,
                &roots,
                &is_abandoned,
                records,
                selected_version,
                slots_per_region,
                &current_records,
                system_configuration.quantities.journal_instance,
                before_the_one_reread,
            )?;
            for (ring_slot, placements_in_the_newer_roots_account) in
                in_the_accounts_of_newer_roots_by_ring_slot
            {
                named_by_ring_slot
                    .entry(ring_slot)
                    .or_default()
                    .extend(placements_in_the_newer_roots_account);
            }
            // 丙并中间：三份都并好之后，走了丙那一支的根那一槽再并上它与所选那一版之间那几槽（函数的注释）。
            with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
                named_by_ring_slot,
                selected_version.checkpoint_txg,
                &txgs_of_the_roots_taking_the_current_account,
                slots_per_region,
            )
        }
        ShadowLedger::Off => BTreeMap::new(),
    };
    let shadow_ledger_computed = match shadow_ledger {
        ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            unit_area_start,
            &mut allocator,
            &roots,
            &is_abandoned,
            effective_floor,
            &current_records,
            &placements_named_by_unapplied_journal_records
                .values()
                .flatten()
                .copied()
                .collect::<Vec<PlacementOnDevice>>(),
            PlacementsOfAbandonedRoots::ReadFromTheirAccounts,
            before_the_one_reread,
            &[],
        )?,
        ShadowLedger::Off => ShadowLedgerComputed {
            abandoned_roots_unreadable: 0,
            placements_referenced_by_abandoned_root: BTreeMap::new(),
        },
    };
    let ShadowLedgerComputed {
        abandoned_roots_unreadable,
        placements_referenced_by_abandoned_root,
    } = shadow_ledger_computed;
    // 挂载时读坏的槽在根环表里记成「挂载时读坏」，带它能住的 txg：回收门槛钉在它可能住过的最旧 txg、准入抬 F 的上限按它能住的最大 txg 并；
    // 读回全零的槽记成「挂载时读回全零」，只在回收门槛里钉在它可能住过的最旧 txg（用户 2026-10-05 定乙）。两类都钉到这个进程写到那个槽、
    // 或进程内重读读出有效根或被抛弃的根为止（用户 2026-10-04 定）。同一个槽这一遍读出了根的，读出来的那条在后面、盖掉它
    // （这一遍读的是同一个槽，几样不会同时成立，留着次序只为不依赖这一点）。
    // 两遍里只有一遍自证过的槽另记读出来的那整条根记录：抬 F 那一串读它读坏、重读一次仍坏时拿它顶上（用户 2026-10-06 定 T1 甲）。
    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = ring_slots_bad_at_mount
        .iter()
        .map(|(ring_slot, txgs_it_can_hold)| {
            (*ring_slot, RootRingOccupant::BadAtMount(*txgs_it_can_hold))
        })
        .chain(ring_slots_reading_back_all_zero_at_mount.iter().map(
            |(ring_slot, oldest_txg_it_can_have_held)| {
                (
                    *ring_slot,
                    RootRingOccupant::AllZeroAtMount {
                        oldest_txg_it_can_have_held: *oldest_txg_it_can_have_held,
                    },
                )
            },
        ))
        .chain(roots_with_ring_slots.iter().map(|(ring_slot, root)| {
            let root_read = if is_abandoned(root) {
                RootReadOnOneRebuildPass::AbandonedRoot {
                    referenced_placements: placements_referenced_by_abandoned_root
                        .get(&(root.instance, root.checkpoint_txg))
                        .cloned()
                        .unwrap_or_default(),
                }
            } else {
                RootReadOnOneRebuildPass::ValidRoot {
                    checkpoint_txg: root.checkpoint_txg,
                }
            };
            let self_verified_on_only_one_of_the_two_passes =
                ring_slots_self_verified_on_only_one_of_the_two_passes.contains(ring_slot);
            let occupant = match (self_verified_on_only_one_of_the_two_passes, root_read) {
                (true, root_read) => {
                    RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                        root: root_read,
                        root_record_self_verified_on_one_rebuild_pass: Box::new(*root),
                    }
                }
                (false, RootReadOnOneRebuildPass::ValidRoot { checkpoint_txg }) => {
                    RootRingOccupant::ValidRoot { checkpoint_txg }
                }
                (
                    false,
                    RootReadOnOneRebuildPass::AbandonedRoot {
                        referenced_placements,
                    },
                ) => RootRingOccupant::AbandonedRoot {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                    referenced_placements,
                },
            };
            (*ring_slot, occupant)
        }))
        .collect();
    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
        system_configuration
            .immutable
            .sizes
            .root_ring_slots_per_region,
        occupants,
        effective_floor,
        CheckpointTxg(
            first_txg
                .0
                .checked_sub(1)
                .expect("新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1"),
        ),
    ));
    allocator.note_placements_named_by_unapplied_journal_records(
        placements_named_by_unapplied_journal_records,
    );
    Ok(RebuiltAllocator {
        allocator,
        effective_floor,
        abandoned_roots_unreadable,
        instance_table_of_the_newest_root,
        root_ring_of_the_rebuilt_allocator,
    })
}

/// 树表 0 条的那一版的账从哪来，只有两条路：
///
/// - 根记录那一项（分配记录树根指针）不是全零 ⇒ 这一版写过行，写行那次发布把这一版的全部分配记录写进了那棵分配记录树
///   （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案；按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14）：
///   整棵读回来重建（挂载时分配记录树整棵读，用户 K4），被换下的实例表因此带着已释放标志回来，不会被当空闲槽发出去。
///   那棵树的节点与记录记进分配器——下一次发布要照抄或换下它们，而这一版没有上一版的内存态可查。
/// - 全零 ⇒ mkfs 的第 0 代（或照抄它的暖机根）：账由实例表与树表两条指针直接算（`format_time_allocator`）。
///
/// # Errors
/// 分配记录树读不出、解不开、位置对不上、条目宽或结构值判红 ⇒ `Recovery(...)`；某个节点两条位置条目不同槽 ⇒
/// `FormatTimeUnitLocationsOnDifferentSlots`；全零那一条上两条指针任一条不是 mkfs 写的那一版 ⇒
/// `VersionWithoutFileNotWrittenByMakeFilesystem`（见 `format_time_allocator`）。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_of_version_without_file 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn allocator_of_version_without_file<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    let Some(tree) = allocation_records_of_version_without_file_in_the_unit_area_starting_at(
        devices,
        unit_area_start,
        root,
    )
    .map_err(MountError::Recovery)?
    else {
        return format_time_allocator(devices, unit_area_start, device_maps, root);
    };
    // 每个节点两条位置条目同槽（第一版两盘同槽）：下一次发布照抄或换下它们时按一个落点释放。
    for (node, pointer) in &tree.version.nodes {
        slot_shared_by_both_location_entries(&pointer.locations).map_err(|disagreement| {
            MountError::FormatTimeUnitLocationsOnDifferentSlots {
                unit: role_of_allocation_record_tree_node(*node),
                disagreement,
            }
        })?;
    }
    let mut allocator = PoolAllocator::rebuild_from_records(device_maps, tree.records.clone());
    allocator.note_allocation_record_tree_of_the_version_without_file(tree);
    Ok(allocator)
}

/// mkfs 写出的那一版（或照抄它的暖机根）的账：盘上没有分配记录树，这一版的全部落点就是根记录直接指着的实例表与树表两个单元
/// ——mkfs 写在单元区里的那两个（字节表五里它们两盘各一条、分配代 0）。与 mkfs 同一个进程里新池新建文件用的分配器同一个起点
/// （`PoolAllocator::mark_format_time_units`），第一个文件版本换下 mkfs 那片树表时照样把它释放。
///
/// # Errors
/// 根记录指着的实例表或树表不是 mkfs 写的那一版（诞生 txg 不是 0）⇒ `VersionWithoutFileNotWrittenByMakeFilesystem`。
///
/// ⚠️ **这一道今天的依据（2026-09-23 随 C512（树表 0 条的一版上被换下的单元记在哪） 定案重判）**：
/// 它**不再**拦任何一条正当历史。此前拦得住的那一条是「建池 → 可写挂载 → 不写文件 → 退出 → 再可写挂载（写行）→ 退出 → 第三次可写挂载」：
/// 写行那次发布换下上一版那片实例表，而那一版没有地方记这条释放，重开之后那一片就成了空闲槽。
/// 定案之后写行那次发布**建起这一版自己的分配记录树**、根指针住根记录，那条历史走 `allocator_of_version_without_file` 的第一条路、
/// 根本到不了这里（`the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool` 钉住）。
/// 走得到这里的只剩「树表或实例表不是 mkfs 写的那一版、而根记录里的分配记录树根指针又是全零」——**实现一处都写不出这种根**
/// （写行那条路径写指针，带文件的那一版的树表不是 mkfs 那一片时早就不走这条分支），所以它今天只对坏镜像与外来镜像说话。
/// 用例要造它得手抹那一项（`plant_a_root_without_its_allocation_record_tree`）。
/// 这一格仍在动分配器、动盘之前拒绝，盘上逐字节不变。
///
/// 两个指针里任一条的两条位置条目槽号不等 ⇒ `FormatTimeUnitLocationsOnDifferentSlots`；两个落点在每块盘上记成的分配记录
/// 过不了盘上分配记录那一道几何判（`recovery::allocation_records_fit_the_pool_geometry`：槽号在单元区起点之下、跨度越过单元区末尾、
/// 两个单元罩住同一个槽）⇒ `Recovery(AllocationRecordOutsideThePoolGeometry)`（代码审阅第 32 条：槽号是根记录里盘上读来的 6 字节，
/// 改之前直接进 `mark_format_time_units`，在分配器的下标与断言上 panic）。三样都在动分配器之前返回。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_fit_the_pool_geometry 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn format_time_allocator<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    if root.instance_table.head.birth_txg != CheckpointTxg(0)
        || root.tree_table.head.birth_txg != CheckpointTxg(0)
    {
        return Err(MountError::VersionWithoutFileNotWrittenByMakeFilesystem {
            root: RollbackTarget {
                instance: root.instance,
                checkpoint_txg: root.checkpoint_txg,
            },
            instance_table_birth_txg: root.instance_table.head.birth_txg,
            tree_table_birth_txg: root.tree_table.head.birth_txg,
        });
    }
    let placement_of = |pointer: &crate::pointer::NodePointer, identity: TransactionUnit| {
        let slot =
            slot_shared_by_both_location_entries(&pointer.locations).map_err(|disagreement| {
                MountError::FormatTimeUnitLocationsOnDifferentSlots {
                    unit: identity,
                    disagreement,
                }
            })?;
        Ok::<Placement, MountError>(Placement {
            slot,
            span: identity.span_slots(),
        })
    };
    let instance_table_placement =
        placement_of(&root.instance_table, TransactionUnit::InstanceTable)?;
    let tree_table_placement = placement_of(&root.tree_table, TransactionUnit::TreeTable)?;
    // `mark_format_time_units` 在每块盘上各记这两条分配代 0 的记录：进分配器之前按盘上分配记录同一道几何判一遍。
    let records_the_format_time_units_become: Vec<AllocationRecord> = device_maps
        .iter()
        .flat_map(|device_map| {
            [instance_table_placement, tree_table_placement].map(|placement| AllocationRecord {
                device: device_map.device,
                slot: placement.slot,
                span_slots: u16::try_from(placement.span).expect("单元的跨度是 1 或 2 槽"),
                generation: CheckpointTxg(0),
                is_released: false,
            })
        })
        .collect();
    allocation_records_fit_the_pool_geometry(
        devices,
        unit_area_start,
        &records_the_format_time_units_become,
    )?;
    let mut allocator = PoolAllocator::new(device_maps);
    allocator.mark_format_time_units(instance_table_placement, tree_table_placement);
    Ok(allocator)
}

/// 一条有效根的用户可见状态有没有变（D16（发布语义） 已定项 1「非空」从盘上怎么认，2026-09-17 用户定案）：它树表里 inode 树与 extent 树的
/// 根指针，与它前一条有效根树表里的，逐字节比；两棵树任一棵不同就算非空。最旧的有效根没有前一条，拿 `UserVisibleTreeRootPointers::ABSENT` 比。
#[must_use]
pub fn user_visible_trees_changed(
    root_pointers: &UserVisibleTreeRootPointers,
    previous_valid_root_pointers: &UserVisibleTreeRootPointers,
) -> bool {
    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree
}

/// 算抬 F 的上限时读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定）：每个槽读一次；读坏的槽按这个进程知不知道
/// 那里住着一条根分两类——`ring_slots_known_to_hold_a_root` 里没有的（挂载那一刻就读不出或自证不过、之后也没写过）当没有根，
/// 有的（挂载那一刻读得出、或这个进程写过且 FUA 返回过）重读一次，读出来就用重读的那一条，仍坏就拒，不按有根或没根猜。
/// `root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass` 里的槽（重建分配器两遍里只有一遍自证过的，各带重建那一遍读出的整条根记录）
/// 读坏也重读一次、读出来就用重读的那一条，仍坏不拒、拿记着的那条根顶上（用户 2026-10-06 定 T1 甲：有效的照它本身进上限，被抛弃的由调用方按实例表滤掉）。
/// 交回读得出（或顶上）的每一条根（按根环槽的次序）与这一遍读坏、当没有根的那几个槽。
///
/// # Errors
/// 知道住着根的槽（不在 `root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass` 里的）重读仍坏 ⇒
/// `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
    root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass: &BTreeMap<
        RootRingSlot,
        RootRecord,
    >,
) -> Result<RootsOfTheRingForTheCeiling, MountError> {
    let read_once = |ring_slot: RootRingSlot| {
        read_root_ring_slot(
            reader,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
            ring_slot,
        )
    };
    let mut roots = Vec::new();
    let mut ring_slots_bad_on_this_read_taken_as_holding_no_root = BTreeSet::new();
    // 迭代上界是根环的槽数（3 × S）；跨轮携带的只有交回的根与当没有根的槽；提前出口只有「知道住着根的槽重读仍坏」那一条。
    for ring_slot in every_root_ring_slot(&system_configuration.immutable.sizes) {
        let first_reading = match read_once(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => {
                roots.push(root);
                continue;
            }
            RootRingSlotReading::Bad(first_reading) => first_reading,
        };
        if let Some(root_record_self_verified_on_one_rebuild_pass) =
            root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass.get(&ring_slot)
        {
            match read_once(ring_slot) {
                RootRingSlotReading::SelfVerified(root) => roots.push(root),
                RootRingSlotReading::Bad(_still_bad) => {
                    roots.push(*root_record_self_verified_on_one_rebuild_pass);
                }
            }
            continue;
        }
        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {
            ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);
            continue;
        }
        match read_once(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => roots.push(root),
            RootRingSlotReading::Bad(reread) => {
                return Err(
                    MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
                        ring_slot,
                        first_reading,
                        reread,
                    },
                );
            }
        }
    }
    Ok(RootsOfTheRingForTheCeiling {
        roots,
        ring_slots_bad_on_this_read_taken_as_holding_no_root,
    })
}

/// 算上限那一遍读根环读出来的（[`roots_of_the_ring_for_the_ceiling`]）。
struct RootsOfTheRingForTheCeiling {
    /// 读得出、自证过的每一条根，与重建分配器两遍里只有一遍自证过、这一遍重读一次仍坏的槽记着的那条根，按根环槽的次序。
    roots: Vec<RootRecord>,
    /// 这一遍读不出或自证不过、又不是「这个进程知道住着根的槽」的槽：当没有根。
    ring_slots_bad_on_this_read_taken_as_holding_no_root: BTreeSet<RootRingSlot>,
}

/// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。
/// 有效 = 自证合法 ∧ 按传进来的实例表不被抛弃 ∧ txg ≥ 今天的 F；非空 = 树表里 inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排、
/// 比它小的有效根里最大的那条）的不同（`user_visible_trees_changed`，2026-09-17 用户定案）。
/// 上限与「非空」在环里全部有效根上取：原来同取一圈窗口的六处之一（B2，用户 2026-10-03 定），用户 2026-10-04 定 RGall 一起退回整个环
/// （`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。回收门槛同时退回整个环，窗口外的有效根引用的单元不再被回收，
/// 读它的树表读得到它那一版（B2 当初挡的攻方 K2-b2 那一形——窗口许可复用之后读窗口外旧根的树表读到别的内容——随之没有了）。
/// 根环怎么读见 [`roots_of_the_ring_for_the_ceiling`]：`ring_slots_whose_reread_still_bad_refuses_the_ceiling` 是这一遍读坏、重读一次仍坏就拒的槽，
/// 取这个进程分配器上那张根环表里知道住着根、又没记成被抛弃根的槽（[`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`]，
/// 与抬 F 那一串同一个口径：被抛弃根的槽读坏不拒、当没有根、不进上限，D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1）；
/// 分配器上没装根环表的（只有用例自己拼的分配器，可写挂载与 `make_filesystem::allocator_after_make_filesystem` 都装）给空集，读坏的槽一律当没有根。
/// 这一份不知道哪几个槽是挂载那一刻读坏的（给手里没有这个进程的根环表的调用方：实验装置、用例）；这个进程的准入抬 F 走
/// [`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]。
///
/// # Errors
/// 一条有效根一条都没有 ⇒ `RecoveryFailure::NoValidRoot`；要比的一条有效根的树表读不出或解不开 ⇒
/// `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（读不出要走修复，不按空或非空猜）；
/// `ring_slots_whose_reread_still_bad_refuses_the_ceiling` 里的根环槽重读仍坏 ⇒ `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
#[allow(
    clippy::ptr_arg,
    reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
pub fn rollback_floor_ceiling<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
    ring_slots_whose_reread_still_bad_refuses_the_ceiling: &BTreeSet<RootRingSlot>,
) -> Result<CheckpointTxg, MountError> {
    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
        devices,
        system_configuration,
        current_floor,
        table,
        ring_slots_whose_reread_still_bad_refuses_the_ceiling,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
}

/// 同 [`rollback_floor_ceiling`]，再把挂载那一刻读不出或读得出而自证不过、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它
/// （D16（发布语义） 已定项 1「抬 F 的上限」与「根槽这一次读坏」两行；RS3 用户 2026-10-04 定只并读不出的，
/// `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节，同日 Q3 重定成两类都并，
/// `research/prompts/m3-root-ring-code-r3-main-verification.md` 第六节）：`ring_slots_bad_since_the_mount` 是这个进程根环表里还记着的
/// 那几个槽（[`ring_slots_bad_at_mount_by`]）。两类都只认挂载那一刻在 (N − 3S, 新实例第一次发布的 txg − 1] 里能住一个 txg 的槽
/// （N = 挂载时环里读得出的最新有效根，`rebuilt_allocator`），各按它在那里能住的最大 txg 并：读不出的那一类与回收门槛记的同一个数，
/// 自证不过的那一类同一个算法（区域内槽位只由 txg 定）。这一遍读那一槽又读坏（读不出或自证不过）、那个 txg 不低于今天的 F，上限就取不高于它。
/// 这一遍读得出的照读出来的根算，不另封顶。判「非空」、找「前一条」照旧只在读得出的根上取（读不出的那一条判不了空不空，F 不越过它就罩住了它是第 4 新的那一种）。
/// `ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes` 是这个进程根环表里重建分配器两遍里只有一遍自证过的槽，
/// 各带重建那一遍读出的整条根记录（[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]）：这一遍读坏就重读一次，
/// 读出来照读出来的算，仍坏不拒、拿记着的那条根顶上，与读出来的根一样按 `table` 判有效（用户 2026-10-06 定 T1 甲：有效的照它本身进上限，
/// 被抛弃的不进；它们也在 `ring_slots_known_to_hold_a_root` 里，这里先按它们分）。
///
/// # Errors
/// 同 [`rollback_floor_ceiling`]。
#[allow(
    clippy::ptr_arg,
    reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
pub fn rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
    ring_slots_bad_since_the_mount: &BTreeMap<RootRingSlot, CheckpointTxg>,
    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes: &BTreeMap<
        RootRingSlot,
        RootRecord,
    >,
) -> Result<CheckpointTxg, MountError> {
    let RootsOfTheRingForTheCeiling {
        roots: readable,
        ring_slots_bad_on_this_read_taken_as_holding_no_root,
    } = roots_of_the_ring_for_the_ceiling(
        devices,
        system_configuration,
        ring_slots_known_to_hold_a_root,
        ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes,
    )?;
    let valid: Vec<RootRecord> = readable
        .iter()
        .copied()
        .filter(|root| root.checkpoint_txg >= current_floor)
        .filter(|root| !abandoned_by_table(root, table))
        .collect();
    let mut newest_per_device: std::collections::BTreeMap<DeviceIdentity, CheckpointTxg> =
        std::collections::BTreeMap::new();
    for root in &valid {
        let device = system_configuration.immutable.region_devices[usize::try_from(
            target_for_publish(
                root.checkpoint_txg,
                system_configuration
                    .immutable
                    .sizes
                    .root_ring_slots_per_region,
            )
            .region,
        )
        .expect("区域号")];
        let newest = newest_per_device
            .entry(device)
            .or_insert(root.checkpoint_txg);
        *newest = (*newest).max(root.checkpoint_txg);
    }
    let newest_on_every_device = newest_per_device
        .values()
        .copied()
        .min()
        .ok_or(RecoveryFailure::NoValidRoot)?;
    // 读不出或解不开就拒绝：它算不算非空、它后面那条跟谁比都判不了，读不出要走修复（D16（发布语义） 已定项 1「非空」从盘上怎么认只说了比树表里的根指针）。
    let pointers_of = |root: &RootRecord| match user_visible_tree_root_pointers(devices, root) {
        Ok(pointers) => Ok(pointers),
        Err(failure) => Err(
            MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
                root: RollbackTarget {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                },
                failure,
            },
        ),
    };
    // 「前一条」只在有效根里找：被抛弃时间线的根与 F 之下的根不算（D16（发布语义） 已定项 1「非空」从盘上怎么认；RGall 之后窗口外的有效根照算）。
    let previous_candidates: &[RootRecord] = &valid;
    let mut non_empty: Vec<NonEmptyValidRoot> = Vec::new();
    for root in &valid {
        let previous_valid_root = previous_candidates
            .iter()
            .filter(|candidate| {
                (candidate.checkpoint_txg, candidate.instance)
                    < (root.checkpoint_txg, root.instance)
            })
            .max_by_key(|candidate| (candidate.checkpoint_txg, candidate.instance));
        let previous_pointers = match previous_valid_root {
            Some(previous) => pointers_of(previous)?,
            None => UserVisibleTreeRootPointers::ABSENT,
        };
        let root_pointers = pointers_of(root)?;
        if user_visible_trees_changed(&root_pointers, &previous_pointers) {
            non_empty.push(NonEmptyValidRoot {
                checkpoint_txg: root.checkpoint_txg,
                instance: root.instance,
                user_visible_tree_root_pointers: root_pointers,
            });
        }
    }
    let ceiling_from_the_roots_read = ceiling_from_newest_and_non_empty_roots(
        newest_on_every_device,
        newest_first_distinct_states(non_empty),
        valid.iter().map(|root| root.checkpoint_txg).min(),
    )
    .ok_or(RecoveryFailure::NoValidRoot)?;
    let lowest_txg_a_ring_slot_bad_since_the_mount_can_hold =
        ring_slots_bad_on_this_read_taken_as_holding_no_root
            .iter()
            .filter_map(|ring_slot| ring_slots_bad_since_the_mount.get(ring_slot))
            .copied()
            .filter(|highest_txg_it_can_hold| *highest_txg_it_can_hold >= current_floor)
            .min();
    Ok(match lowest_txg_a_ring_slot_bad_since_the_mount_can_hold {
        Some(highest_txg_it_can_hold) => ceiling_from_the_roots_read.min(highest_txg_it_can_hold),
        None => ceiling_from_the_roots_read,
    })
}

/// 一条非空的有效根：它的 (txg, 实例代号) 与它树表里用户可见的两棵树的根指针（去重按后者比）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NonEmptyValidRoot {
    pub checkpoint_txg: CheckpointTxg,
    pub instance: InstanceGeneration,
    pub user_visible_tree_root_pointers: UserVisibleTreeRootPointers,
}

/// 数「第 N 新的非空有效根」与「4 个不同状态」时去重（D16（发布语义） 已定项 1「「非空」从盘上怎么认」末段）：从新到旧数，
/// 一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入——管理员回退发出的新根
/// 与它前一条有效根不同、算非空，而它的状态与更早的某一条相同，不去重就把同一个状态数两次。交回计入的那几条的 txg，从新到旧。
///
/// 根环容量是这一数的边界（同一段定案首段）：两个状态来回交替超过根环一圈时，环里的根全是这两个状态，这里只数得出两条，
/// 上限随之取最旧的有效根（[`ceiling_from_newest_and_non_empty_roots`]）。
///
/// 循环：外层按 (txg, 实例代号) 从新到旧走一遍非空根（轮数 = 非空根的条数），跨轮携带已计入的状态；没有提前出口。
#[must_use]
pub fn newest_first_distinct_states(mut non_empty: Vec<NonEmptyValidRoot>) -> Vec<CheckpointTxg> {
    non_empty.sort_unstable_by(|left, right| {
        (right.checkpoint_txg, right.instance).cmp(&(left.checkpoint_txg, left.instance))
    });
    let mut counted_states: Vec<&UserVisibleTreeRootPointers> = Vec::new();
    let mut distinct = Vec::new();
    for root in &non_empty {
        if counted_states.contains(&&root.user_visible_tree_root_pointers) {
            continue;
        }
        counted_states.push(&root.user_visible_tree_root_pointers);
        distinct.push(root.checkpoint_txg);
    }
    distinct
}

/// 上限 = min(每块盘上最新的有效根, 第 4 新的不同状态的非空有效根)；去重之后不足 4 个时取最旧的有效根；一条有效根都没有时 `None`。
/// `distinct_states_newest_first` 是 [`newest_first_distinct_states`] 交回的那一串，`oldest_valid_root` 是有效根里最小的 txg。
fn ceiling_from_newest_and_non_empty_roots(
    newest_on_every_device: CheckpointTxg,
    distinct_states_newest_first: Vec<CheckpointTxg>,
    oldest_valid_root: Option<CheckpointTxg>,
) -> Option<CheckpointTxg> {
    let fourth_newest = distinct_states_newest_first
        .get(3)
        .copied()
        .or(oldest_valid_root)?;
    Some(newest_on_every_device.min(fourth_newest))
}

/// 抬 F 的空发布做完之后的东西。
#[derive(Clone, Debug)]
pub struct RaisedFloor {
    pub ceiling: CheckpointTxg,
    /// 第一次发布之前先把新 F 写进每块盘系统配置那一步的写（每块盘一次系统配置槽写，D16（发布语义） 已定项 1「抬 F 那一串」）：
    /// 不属于 `publishes` 里任何一次，与它们相加才等于这一串交给设备的写。
    pub system_configuration_writes_before_the_first_publish: WritesByStructureKind,
    /// 带新 F 的空发布，直到每块盘上都有一条带新 F 的持久根（生效条件）。
    pub publishes: Vec<TransactionOutput>,
    /// 生效之后回收的落点（两盘同槽，按盘 0 报）。
    pub reclaimed: Vec<Placement>,
    /// 抬 F 重算影子账时读不出账、被跳过的被抛弃根的条数（与 `MountOutput::abandoned_roots_unreadable` 同一个量；
    /// 代码三方第三轮云端攻方腿打中：抬 F 那一处此前把计数丢掉了）。根环表里记着落点的被抛弃根、借 C 那一圈落点的被抛弃根与根槽这一遍读坏的
    /// 被抛弃根这一遍都不读盘，不计在内。
    pub abandoned_roots_unreadable: u64,
}

/// 正常卸载做完之后的样子（D16（发布语义） 已定项 1「正常卸载」）。
pub enum Unmounted {
    /// 现行那一版带文件：推了一串带卸载记号的空发布，F 抬到它的 txg。
    FloorRaisedToTheCurrentVersion(UnmountRaisedTheFloor),
    /// 现行那一版树表 0 条（还没写过文件）：一个字节都不写，卸载照常做成。回退候选要带文件，而这一版在它的时间线上还没有带文件的版本，
    /// 抬 F 买不到东西（主 agent 2026-09-26 定，实二交回第一问；推的，被攻过零轮）。
    NothingWrittenOnAVersionWithoutFile { current: RollbackTarget },
}

/// 正常卸载推了那一串空发布之后的东西。卸载不判上限，没有 `ceiling`。
pub struct UnmountRaisedTheFloor {
    /// 抬到的 F：卸载开始时现行那一版的 txg。
    pub rollback_floor: CheckpointTxg,
    /// 第一次发布之前先把新 F 写进每块盘系统配置那一步的写（同 [`RaisedFloor`] 那一项）。
    pub system_configuration_writes_before_the_first_publish: WritesByStructureKind,
    /// 带新 F、带卸载记号的空发布，直到每块盘上都有一条（第一版几何上 2 或 3 次）。
    pub publishes: Vec<TransactionOutput>,
    /// 生效之后回收的落点（同 [`RaisedFloor`] 那一项）。
    pub reclaimed: Vec<Placement>,
    /// 同 [`RaisedFloor`] 那一项。
    pub abandoned_roots_unreadable: u64,
}

/// 抬 F 那一串从哪个入口进来（D16（发布语义） 已定项 1「抬 F 那一串」：准入与卸载共用这一串，两个入口只差三样：
/// 判不判上限、根带不带卸载记号、回收出来的槽什么时候能发出去）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RaiseFloorEntry {
    /// 准入抬 F（`raise_rollback_floor`）：新 F 不高于上限（「抬 F 的上限」那一行的准入式子），这一串的根不带卸载记号。
    Admission,
    /// 正常卸载（`unmount`）：新 F 是现行那一版的 txg、不另判上限，这一串的根全带卸载记号（D22（单元原子性怎么合成） 已定项 7）。
    Unmount,
}

impl RaiseFloorEntry {
    const fn unmount_marker(self) -> UnmountMarker {
        match self {
            RaiseFloorEntry::Admission => UnmountMarker::NotWrittenByTheUnmountSequence,
            RaiseFloorEntry::Unmount => UnmountMarker::WrittenByTheUnmountSequence,
        }
    }

    /// 这一串回收出来的槽什么时候能发出去（用户 2026-10-03 定「甲 GA」，`research/prompts/m3-unlink-code-r3-forks.md` 丙1 单盘幸存那一格；
    /// 判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 1 条）：准入那一格整截扣到生效（每块盘上都有带新 F 的根）；
    /// 卸载那一格只把释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场放开，整串预演与真发都能用，其余照准入那一格扣到生效
    /// （[`ReuseOfTheSlotsReclaimedByTheSequence`]）。
    /// 为什么只放那一截：恢复择根不看 F（`recovery::choose_root` 照 `(checkpoint_txg, 实例代号)` 取最大），丢掉一块盘之后择的是剩下那块盘上
    /// 最新的根，它可以低于系统配置里的新 F（判决第二节 N1 那一行）；一个槽的释放代 g 不高于每块盘上最新的有效根，就没有一块盘上最新的根引用它
    /// （引用它的根都早于 g），丢哪块盘恢复都择不到引用它的根。挂载续做推满仍放不下之后卸载靠这一截做成（槽被续做吃光，整截扣着的话这一串在预演里
    /// 取不到落点，`admission_raises_the_floor_before_refusing.rs` 里续做推满之后卸载、再挂续做那一条）。
    const fn reuse_of_the_reclaimed_slots(self) -> ReuseOfTheSlotsReclaimedByTheSequence {
        match self {
            RaiseFloorEntry::Admission => ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect,
            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice,
        }
    }
}

/// 抬 F 那一串回收出来的槽什么时候能发出去（[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReuseOfTheSlotsReclaimedByTheSequence {
    /// 整截扣到生效（带新 F 的根落满每块盘之后 `PoolAllocator::release_reclaim_holds` 才放开）。
    HeldUntilFloorTakesEffect,
    /// 释放代不高于 [`newest_valid_root_txg_on_every_device`] 的那一截当场能发，其余扣到生效。
    ImmediatelyBelowTheNewestValidRootOnEveryDevice,
}

/// 每块盘上最新的有效根的 txg 取最小（GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]；可写挂载重建分配器时当场放开到哪儿、
/// 要不要整环重读也取它，甲、Q4 rr2，`rebuilt_allocator`）：`roots_on_their_devices` 是调用方这一遍从根环读出来的根，各带它归哪块盘，
/// `table` 是判被抛弃用的那张实例表（卸载那一格是现行那一版的，重建那一格是最新那条根指着的）。归盘由调用方定：重建那一格按读出它的那个槽
/// 所在的区域（mkfs 三个区域槽 0 各一份的第 0 代根各归各的盘，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第二节 K2），
/// 卸载那一格按根环落点公式（那一遍的读数不带槽；卸载接在一次可写挂载之后，暖机已在每块盘上落了根，环里不会只剩第 0 代根）。
/// - 被抛弃的根（按 `table`，`abandoned_by_table`）不算。卸载那一格滤不滤结局相同：卸载接在一次可写挂载之后，那次挂载的暖机先在每块盘上
///   落了新实例的根（D16（发布语义） 已定项 8 戊），新实例的 txg 大于环里每一条根（D23（journal 的角色与格式） 已定项 14 第 3 条），
///   卸载时各盘最新的根都不是被抛弃的（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「单盘恢复择中被抛弃的根」那一行，
///   一次抽样 9027 个判定择中 0 次）。
/// - 这一遍读不出、自证不过的槽当没有根（与这一遍回收门槛同一个读数）：恢复同样择不到它；它日后又读得出的话那条根只会更新或更旧——更新的引用的已释放槽更少，
///   更旧的轮不到被择。读不出让最小值只会更小，当场放开的只会更少。
/// - 有一块盘上一条有效根都没读出来：交回 `None`（不按 txg 0 去比，说清是哪一格），两格各自处置：
///   卸载那一格这一串回收的槽整截扣到生效；重建那一格当场只放到环里最旧有效根（RGall；挂载时读坏的槽钉在它可能住过的最旧 txg 并进，
///   与回收门槛同一个读数），其余扣到这次挂载的暖机落满每块盘（`rebuilt_allocator`，用户 2026-10-04 选甲，
///   `research/prompts/m3-batch16-code-r1-forks.md` 第 1 行）。
fn newest_valid_root_txg_on_every_device(
    roots_on_their_devices: &[(DeviceIdentity, RootRecord)],
    table: &InstanceTableRecords,
    all_devices: &[DeviceIdentity],
) -> Option<CheckpointTxg> {
    let newest_on_each_device: Vec<Option<CheckpointTxg>> = all_devices
        .iter()
        .map(|device| {
            roots_on_their_devices
                .iter()
                .filter(|(device_of_the_root, root)| {
                    !abandoned_by_table(root, table) && *device_of_the_root == *device
                })
                .map(|(_, root)| root.checkpoint_txg)
                .max()
        })
        .collect();
    newest_on_each_device
        .into_iter()
        .collect::<Option<Vec<CheckpointTxg>>>()?
        .into_iter()
        .min()
}

/// 抬 F 那一串做完之后两个入口共有的东西；准入那一格另带它判过的上限。
struct FloorRaisedThroughTheSequence {
    ceiling_judged_by_the_admission: Option<CheckpointTxg>,
    system_configuration_writes_before_the_first_publish: WritesByStructureKind,
    publishes: Vec<TransactionOutput>,
    reclaimed: Vec<Placement>,
    abandoned_roots_unreadable: u64,
}

/// 抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）。
///
/// 产品路径上的触发是准入（「准入」那一行：准入不够、或准入放行而落点取不到时先推空发布抬 F），经
/// [`raise_rollback_floor_to_the_admission_ceiling`] 抬到上限：挂着的会话在发布被拒时推（`mounted_session`），可写挂载在写行之后推
/// （`establish_instance`）。直接给一个 `new_floor` 的这一个入口是只供测试强制进入的那一档（`.claude/rules/fs-design.md`
/// 五条硬要求第 2 条）：抬到上限以下的某一格。
/// 推空发布直到每块盘上都有带新 F 的根（生效），然后把释放代 ≤ 新 F 的已释放落点回收；第一次发布之前先把新 F 写进每块盘的系统配置、
/// 过一道屏障（「抬 F 那一串」那一行）。
///
/// 调用方交进来的参数与盘表照可写挂载同一套核（实审 A1b Q2、Q3），写入口只按盘上择到的那一份参数建。
///
/// # Errors
/// 盘表里有身份交了不止一次（`DeviceIdentitiesHandedInMoreThanOnce`）、调用方的参数或盘表（盘数与 `devs`、每块盘的本盘设备号）
/// 与盘上择到的系统配置不一致、或有盘不「可见」（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`）、有盘落后于现行那一版又缺它的单元或它的根
/// （`DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`），都在回收、影子账与任何写之前；
/// 现行那一版的根指着的实例表读不出或解不开（`InstanceTableMalformed`）；`new_floor` 低于盘上现算的 F 生效值
/// （`RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided`）；`new_floor` 超过上限；算上限的错（[`rollback_floor_ceiling`]，
/// 含知道住着根的根环槽重读仍坏 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）；现行那一版的 txg 已到顶、这一串的 txg 装不下
/// （`SequenceNumberPastTheTopOfItsRange`，在回收、影子账与任何写之前）；
/// 那一串空发布在任何写之前的预演里有一次报错（`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`，一次都不发）；
/// 先写系统配置那一步报错（`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`，一条根都不发）；
/// 预演过了、真发时有一次发不出去（`RaiseFloorSequencePublishFailed`，带着先写系统配置那一步、前面已经落盘的那几次与失败那一次的写账）。
pub fn raise_rollback_floor<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
    shadow_ledger: ShadowLedger,
) -> Result<RaisedFloor, MountError> {
    let raised = raise_the_floor_through(
        RaiseFloorEntry::Admission,
        parameters,
        devices,
        allocator,
        current,
        new_floor,
        shadow_ledger,
    )?;
    Ok(RaisedFloor {
        ceiling: raised
            .ceiling_judged_by_the_admission
            .expect("准入那一格判过上限才走到发布：`raise_the_floor_through` 只在准入入口算上限，这里是准入入口"),
        system_configuration_writes_before_the_first_publish: raised
            .system_configuration_writes_before_the_first_publish,
        publishes: raised.publishes,
        reclaimed: raised.reclaimed,
        abandoned_roots_unreadable: raised.abandoned_roots_unreadable,
    })
}

/// 正常卸载（D16（发布语义） 已定项 1「正常卸载」，用户 2026-09-25 定 B1）：推一串空发布把 F 抬到现行那一版的 txg、不另判上限，
/// 走的是准入抬 F 那一串（影子账按新 F 重算、回收、整串预演、先把新 F 写进每块盘的系统配置过一道屏障、推到每块盘上都有一条带新 F 的根），
/// 这一串写的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）。与准入那一格差在回收出来的槽里释放代不高于每块盘上最新有效根的
/// 那一截不扣到生效、预演与真发都能用，其余照样扣（用户 2026-10-03 定甲 GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）：
/// 挂载续做推满仍放不下、槽被吃光的盘上卸载照样做成，靠的就是这一串的预演算上那一截。第一版几何（R = 3、区域归属 0 / 1 / 0）下推 2 或 3 次：
/// 现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次。不写干净关闭标记，下一次挂载照旧走恢复。做完之后这个写入口不再发布。
/// 现行那一版树表 0 条时一个字节都不写，卸载照常做成（[`Unmounted::NothingWrittenOnAVersionWithoutFile`]）。
///
/// # Errors
/// 同 [`raise_rollback_floor`]（含调用方参数与盘表的那一套核），少一条「超过上限」；现行那一版树表 0 条时不报错、不核（一个字节都不写）。
pub fn unmount<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut PoolVersion,
    shadow_ledger: ShadowLedger,
) -> Result<Unmounted, MountError> {
    let current_file_version = match current {
        PoolVersion::WithFile(current_file_version)
        | PoolVersion::WithTreesAndNoFile(current_file_version) => current_file_version,
        PoolVersion::WithoutFile(current_version_without_file) => {
            return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {
                current: RollbackTarget {
                    instance: current_version_without_file.root.instance,
                    checkpoint_txg: current_version_without_file.root.checkpoint_txg,
                },
            });
        }
    };
    let rollback_floor = current_file_version.root.checkpoint_txg;
    let raised = raise_the_floor_through(
        RaiseFloorEntry::Unmount,
        parameters,
        devices,
        allocator,
        current_file_version,
        rollback_floor,
        shadow_ledger,
    )?;
    Ok(Unmounted::FloorRaisedToTheCurrentVersion(
        UnmountRaisedTheFloor {
            rollback_floor,
            system_configuration_writes_before_the_first_publish: raised
                .system_configuration_writes_before_the_first_publish,
            publishes: raised.publishes,
            reclaimed: raised.reclaimed,
            abandoned_roots_unreadable: raised.abandoned_roots_unreadable,
        },
    ))
}

/// 这个进程分配器上那张根环表里挂载那一刻读坏（读不出，或读得出、不是全零而自证不过）、之后没写过也没重读出有效根或被抛弃的根的槽，
/// 各按它能住的最大 txg（`allocator::RootRingOccupancy::ring_slots_bad_at_mount`，Q3 两类都并；读回全零的槽不在其内，
/// 用户 2026-10-04 定，它们只在回收门槛里钉着）；没装根环表的分配器给空表。准入抬 F 的上限按它封顶（[`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]）。
#[must_use]
pub fn ring_slots_bad_at_mount_by(
    allocator: &PoolAllocator,
) -> BTreeMap<RootRingSlot, CheckpointTxg> {
    allocator
        .root_ring_occupancy()
        .map_or_else(BTreeMap::new, RootRingOccupancy::ring_slots_bad_at_mount)
        .into_iter()
        .map(|(ring_slot, txgs_it_can_hold)| {
            (ring_slot, txgs_it_can_hold.highest_txg_it_can_hold())
        })
        .collect()
}

/// 这个进程分配器上那张根环表里知道住着一条根的槽（`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）；
/// 没装根环表的分配器（只有用例自己拼的）给空集。挂着时管理员回退判候选读根环按它分两类；算抬 F 的上限除去记成被抛弃根的槽
/// （[`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`]）。
#[must_use]
pub fn ring_slots_known_to_hold_a_root_by(allocator: &PoolAllocator) -> BTreeSet<RootRingSlot> {
    allocator.root_ring_occupancy().map_or_else(
        BTreeSet::new,
        RootRingOccupancy::ring_slots_known_to_hold_a_root,
    )
}

/// 这个进程分配器上那张根环表里重建分配器两遍里只有一遍自证过、之后没写过的槽，各带重建那一遍读出的整条根记录
/// （`allocator::RootRingOccupancy::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes`）；
/// 没装根环表的分配器给空表。抬 F 那一串（准入算上限、抬 F 重算影子账与回收门槛，卸载同一串）读它们读坏、重读一次仍坏时拿记着的那条根顶上、
/// 不拒（用户 2026-10-06 定 T1 甲，[`raise_rollback_floor`] 一族）。
#[must_use]
pub fn ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(
    allocator: &PoolAllocator,
) -> BTreeMap<RootRingSlot, RootRecord> {
    allocator.root_ring_occupancy().map_or_else(
        BTreeMap::new,
        RootRingOccupancy::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes,
    )
}

/// 这个进程分配器上那张根环表里记成被抛弃根的槽与那条根挂载时记着的落点
/// （`allocator::RootRingOccupancy::ring_slots_recorded_as_holding_an_abandoned_root`）；没装根环表的分配器给空表。
/// 抬 F 那一串（准入算上限、抬 F 重算影子账与回收门槛，卸载同一串）读它们读坏、重读仍坏不拒：上限本来不看被抛弃根，影子账照这里记着的落点算
/// （用户 2026-10-08 定 H1「被抛弃根的槽不拒」）。
#[must_use]
pub fn ring_slots_recorded_as_holding_an_abandoned_root_by(
    allocator: &PoolAllocator,
) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
    allocator.root_ring_occupancy().map_or_else(
        BTreeMap::new,
        RootRingOccupancy::ring_slots_recorded_as_holding_an_abandoned_root,
    )
}

/// 这个进程的准入抬 F 的上限（[`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]），三类槽都取这个进程分配器上那张根环表：
/// 知道住着根的槽、挂载那一刻读坏的槽、重建分配器两遍里只有一遍自证过的槽（后一类读坏重读仍坏时拿记着的那条根顶上，用户 2026-10-06 定 T1 甲）。
/// 知道住着根的槽里记成被抛弃根的那几槽不算「重读仍坏就拒」的槽（用户 2026-10-08 定 H1「被抛弃根的槽不拒」）：这一遍读坏就当没有根，
/// 上限只在有效根上取、本来不看它们；它们也不是挂载那一刻读坏的槽，不并进上限。
///
/// # Errors
/// 同 [`rollback_floor_ceiling`]。
#[allow(
    clippy::ptr_arg,
    reason = "rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount 走 &Vec：要一个有大小的读者"
)]
fn rollback_floor_ceiling_of_this_process<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
    allocator: &PoolAllocator,
) -> Result<CheckpointTxg, MountError> {
    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
        devices,
        system_configuration,
        current_floor,
        table,
        &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(allocator),
        &ring_slots_bad_at_mount_by(allocator),
        &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator),
    )
}

/// 算抬 F 的上限读根环时，这一遍读坏、重读一次仍坏就拒的槽（[`rollback_floor_ceiling`] 的 `ring_slots_whose_reread_still_bad_refuses_the_ceiling`）：
/// 这个进程分配器上那张根环表里知道住着根的槽（[`ring_slots_known_to_hold_a_root_by`]），除去记成被抛弃根的那几槽
/// （[`ring_slots_recorded_as_holding_an_abandoned_root_by`]；D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1
/// 「被抛弃根的槽不拒」：读坏就当没有根，上限只在有效根上取、本来不看它们）。这个进程的准入抬 F（[`rollback_floor_ceiling_of_this_process`]）
/// 与自算上限的调用方（实验装置、checker 档的运行模式）都取这一份。没装根环表的分配器给空集。
#[must_use]
pub fn ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(
    allocator: &PoolAllocator,
) -> BTreeSet<RootRingSlot> {
    let ring_slots_recorded_as_holding_an_abandoned_root =
        ring_slots_recorded_as_holding_an_abandoned_root_by(allocator);
    ring_slots_known_to_hold_a_root_by(allocator)
        .into_iter()
        .filter(|ring_slot| {
            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
        })
        .collect()
}

/// 一次准入最多推几次发布：B = 4 + 2 k_tol，k_tol = 2 ⇒ 8（D16（发布语义） 已定项 1「准入」那一行）。数的是这次准入推的空发布
/// 加上这次准入为之判的那一次发布（发布路径是那次用户发布，可写挂载那一处是写行那次）：`df` 的保留池按 B − 1 次空发布留
/// （同一张表「`df`」那一行：保留池 = 2 × 5 + (B − 1) × c_max）。
pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;

/// 准入抬 F 到上限那一次的结局（[`raise_rollback_floor_to_the_admission_ceiling`]）。
#[derive(Debug)]
pub enum RaiseToTheAdmissionCeiling {
    /// 上限高于现行那一版的 F：推了那一串，F 抬到上限。
    Raised(RaisedFloor),
    /// 上限不高于现行那一版的 F：没有可抬的，一个字节都没写、分配器不动。
    FloorAlreadyAtTheCeiling {
        floor: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
}

/// 准入抬 F（D16（发布语义） 已定项 1「准入」那一行：准入不够、或准入放行而落点取不到时先推空发布抬 F）：F 抬到这一刻算出的上限
/// （「抬 F 的上限」那一行的准入式子，[`rollback_floor_ceiling`]）。上限不高于现行那一版的 F 就不推。
/// 上限在这里算一次、交给 [`raise_rollback_floor`] 之后它在任何写之前按同一份输入再算一次：两次之间没有写，
/// 对不上（两次读之间的瞬时读错让第二次算得更低）时那边在任何写之前拒（`RollbackFloorAboveCeiling`）。
/// 调用方交进来的参数与盘表先照可写挂载同一套核（`caller_inputs_agreeing_with_the_disk`，实审 A1b Q2、Q3），
/// 之后只用盘上那一份参数；上限不高于 F、一个字节都不写的那一格也先核。
///
/// # Errors
/// 盘表里有身份交了不止一次、调用方的参数或盘表与盘上系统配置不一致（`DeviceIdentitiesHandedInMoreThanOnce`、
/// `CallerParametersDisagreeWithTheSelectedSystemConfiguration`）；选系统配置、读现行那一版的实例表、算上限的错（都在任何写之前）；
/// [`raise_rollback_floor`] 的错原样交回。
pub fn raise_rollback_floor_to_the_admission_ceiling<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
) -> Result<RaiseToTheAdmissionCeiling, MountError> {
    let caller_inputs_agreeing_with_the_disk =
        caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;
    raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk(
        &caller_inputs_agreeing_with_the_disk,
        devices,
        allocator,
        current,
        shadow_ledger,
    )
}

/// [`raise_rollback_floor_to_the_admission_ceiling`] 核过调用方的参数与盘表之后那一半：算上限、不高于 F 就不推，高于就抬到它。
fn raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
    caller_inputs_agreeing_with_the_disk: &CallerInputsAgreeingWithTheDisk,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
) -> Result<RaiseToTheAdmissionCeiling, MountError> {
    let current_instance_table = instance_table_chain_of_root(&*devices, &current.root)
        .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?
        .records;
    let ceiling = rollback_floor_ceiling_of_this_process(
        devices,
        &caller_inputs_agreeing_with_the_disk.system_configuration,
        current.root.rollback_floor,
        &current_instance_table,
        allocator,
    )?;
    if ceiling <= current.root.rollback_floor {
        return Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling {
            floor: current.root.rollback_floor,
            ceiling,
        });
    }
    raise_rollback_floor(
        &caller_inputs_agreeing_with_the_disk.parameters_of_the_pool,
        devices,
        allocator,
        current,
        ceiling,
        shadow_ledger,
    )
    .map(RaiseToTheAdmissionCeiling::Raised)
}

/// 一次准入里为什么不再推抬 F 的空发布（推完仍不够时交回的原因）：只有「推满仍不够」的三种——预算用完、F 已在上限、
/// 抬 F 那一串自己也被空间不够拒（D16（发布语义） 已定项 1「准入」那一行：落点取不到也先推抬 F 再判，推不出空间就是推满仍不够；实审 A1b Q1）。
/// 抬 F 自己报的别的错（块设备错、读不出、预演被别的理由拒……）不在这里：它不是空间不够，原样往上交（代码审阅第 23 条，
/// [`push_one_floor_raise_within_the_admission_budget`] 的 `Err`）。
#[derive(Debug)]
pub enum FloorRaiseStop {
    /// 再推一串就超过一次准入最多的发布数（[`PUBLISHES_PER_ADMISSION_AT_MOST`]）：这次准入已经推了 `publishes_pushed` 次空发布，
    /// 下一串要推 `publishes_of_the_next_raise` 次，再加这次准入为之判的那一次发布。
    PublishesPerAdmissionWouldBeExceeded {
        publishes_pushed: usize,
        publishes_of_the_next_raise: usize,
    },
    /// 上限不高于现行那一版的 F：抬不动（[`RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling`]）。
    FloorAlreadyAtTheCeiling {
        floor: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
    /// 抬 F 那一串自己的空发布取不到落点、或被空间准入拒（`mounted_session::refusal_is_short_of_space` 那两种）：这一串推不出空间。
    /// `refusal` 是抬 F 那一串报的错原样，只会是两个成员之一：`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`（预演里被拒，
    /// 一次都没发，盘上逐字节不变、分配器与抬 F 之前逐项相同），或 `RaiseFloorSequencePublishFailed`（预演与真发分叉那一格：真发时第 n 次
    /// 在落盘之前被拒，先写系统配置那一步与前面 n − 1 次已落盘、在调用方的现行版本里），两者的 `cause` 都是空间不够那两种之一。
    /// 装箱：不装箱就把 `MountError` 整个摊进这个枚举。
    FloorRaiseRefusedForSpace { refusal: Box<MountError> },
}

/// 抬 F 那一串报的错是不是空间不够那两种（取不到落点、空间准入不够，`mounted_session::refusal_is_short_of_space`）：
/// 预演里被拒，或真发时在落盘之前被拒。是 ⇒ 这一串推不出空间，算推满仍不够（[`FloorRaiseStop::FloorRaiseRefusedForSpace`]）；
/// 别的成员都不是空间不够（实审 A1b Q1）。只看错，不读写盘。
fn floor_raise_refused_for_space(refusal: &MountError) -> bool {
    match refusal {
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { cause, .. } => {
            refusal_is_short_of_space(cause)
        }
        MountError::RaiseFloorSequencePublishFailed(failed) => {
            refusal_is_short_of_space(&failed.cause)
        }
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        | MountError::RollbackFloorAboveCeiling { .. }
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        // txg 到顶不是空间不够；可写挂载那一处抬 F 的错装进的那个成员只由可写挂载交回、抬 F 自己报不出来。
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::FloorRaiseFailedAfterTheMountsPublishes(_)
        // 重读一次仍读不出只由可写挂载的读阶段与重建分配器交回（C554 乙），读阶段之后系统配置又读不出只由可写挂载取号之前与取号交回：
        // 抬 F 报不出来，也不是空间不够。
        | MountError::NewerStateStillUnreadableAfterOneReread(_)
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
        // 重建分配器读根环重读那一遍仍一条有效根都没有只由可写挂载的重建分配器交回：抬 F 报不出来，也不是空间不够。
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => false,
    }
}

/// 一次准入里再推一串抬 F 做成了什么：推了，或推满了、不再推（[`FloorRaiseStop`]）。抬 F 自己报的错不在这里，是
/// [`push_one_floor_raise_within_the_admission_budget`] 的 `Err`。
#[derive(Debug)]
pub enum FloorRaisePushedWithinTheAdmissionBudget {
    Raised(RaisedFloor),
    Stopped(FloorRaiseStop),
}

/// 一次准入里再推一串抬 F 的空发布：先核调用方交进来的参数与盘表（照可写挂载同一套，实审 A1b Q2、Q3；之后只用盘上那一份参数），
/// 再按预算判（这次准入已经推的 `publishes_pushed` 次，加这一串要推的次数，
/// 加这次准入为之判的那一次发布，不超过 [`PUBLISHES_PER_ADMISSION_AT_MOST`]），再抬到上限。这一串要推几次与抬 F 那一串按同一个
/// 纯算的函数数（`txgs_of_the_publishes_carrying_the_floor_to_every_device`）。推满仍不够（预算用完、F 已在上限、抬 F 那一串自己也被
/// 空间不够拒）交回 [`FloorRaisePushedWithinTheAdmissionBudget::Stopped`]。
///
/// # Errors
/// 盘表里有身份交了不止一次、调用方的参数或盘表与盘上系统配置不一致（在判预算与任何写之前）；
/// 抬 F 自己报的别的错（[`raise_rollback_floor_to_the_admission_ceiling`] 那一半的错，原样；代码审阅第 23 条：块设备错不是空间不够）：
/// 在任何写之前被拒的，盘上与分配器不动；真发时落盘途中失败的（`MountError::RaiseFloorSequencePublishFailed`），
/// 前面已落盘的那几次在调用方的现行版本里。
pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
    publishes_pushed: usize,
) -> Result<FloorRaisePushedWithinTheAdmissionBudget, MountError> {
    let caller_inputs_agreeing_with_the_disk =
        caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;
    let parameters_of_the_pool = &caller_inputs_agreeing_with_the_disk.parameters_of_the_pool;
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(
            txg,
            parameters_of_the_pool.geometry.root_ring_slots_per_region,
        );
        parameters_of_the_pool.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    let publishes_of_the_next_raise = txgs_of_the_publishes_carrying_the_floor_to_every_device(
        current.root.checkpoint_txg,
        &all_devices,
        &device_of_txg,
    )?
    .len();
    let publishes_of_this_admission = publishes_pushed + publishes_of_the_next_raise + 1;
    if publishes_of_this_admission > PUBLISHES_PER_ADMISSION_AT_MOST {
        return Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
            FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded {
                publishes_pushed,
                publishes_of_the_next_raise,
            },
        ));
    }
    match raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk(
        &caller_inputs_agreeing_with_the_disk,
        devices,
        allocator,
        current,
        shadow_ledger,
    ) {
        Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised))
        }
        Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
                FloorRaiseStop::FloorAlreadyAtTheCeiling { floor, ceiling },
            ))
        }
        Err(refusal) if floor_raise_refused_for_space(&refusal) => {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(
                FloorRaiseStop::FloorRaiseRefusedForSpace {
                    refusal: Box::new(refusal),
                },
            ))
        }
        Err(refusal) => Err(refusal),
    }
}

/// 抬 F 那一遍重算影子账与回收门槛读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：每个槽读一次；读坏的槽分三类——
/// 这个进程不知道住着根的当没有根；知道住着根的重读一次，仍坏就拒（C554 乙报告 Q6）；重建分配器两遍里只有一遍自证过的
/// （[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]，它们也在「知道住着根的槽」里，这里先按它们分）重读一次，
/// 读出来照读出来的算，仍坏不拒、拿重建那一遍读出、记在根环表里的那条根顶上（用户 2026-10-06 定 T1 甲：有效的照它本身进回收门槛与候选集，
/// 被抛弃的照它引用的落点进影子账；有效还是被抛弃由调用方按同一张实例表判）。根环表里记成被抛弃根的槽（它们也在「知道住着根的槽」里，
/// 这里同样先按它们分，[`ring_slots_recorded_as_holding_an_abandoned_root_by`]）这一遍读坏不拒、不重读：读不出它的根记录，交回它挂载时
/// 记在根环表里的落点，由影子账照它补隔离、不进回收门槛（用户 2026-10-08 定 H1「被抛弃根的槽不拒」）。重读读得出也是同一条被抛弃根、
/// 影子账同样取表里记着的那一份（[`PlacementsOfAbandonedRoots`]），所以不重读结局相同。交回读得出（或顶上）的每一条根（重读读出来的与
/// 顶上的排在后面）与那几槽记着的落点。只读盘。
///
/// # Errors
/// 知道住着根的槽（两遍里只有一遍自证过的、记成被抛弃根的除外）重读仍坏 ⇒
/// `NewerStateStillUnreadableAfterOneReread(StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot(..))`。
fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    allocator: &PoolAllocator,
) -> Result<RootsReadForRaisingTheFloor, MountError> {
    let ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass =
        ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator);
    let ring_slots_recorded_as_holding_an_abandoned_root =
        ring_slots_recorded_as_holding_an_abandoned_root_by(allocator);
    let ring_slots_whose_reread_still_bad_refuses_the_raise: BTreeSet<RootRingSlot> =
        ring_slots_known_to_hold_a_root_by(allocator)
            .into_iter()
            .filter(|ring_slot| {
                !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass
                    .contains_key(ring_slot)
            })
            .filter(|ring_slot| {
                !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
            })
            .collect();
    let mut roots_with_ring_slots =
        readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once(
            reader,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
            &ring_slots_whose_reread_still_bad_refuses_the_raise,
        )
        .map_err(|still_unreadable| {
            MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
        })?;
    let ring_slots_read: BTreeSet<RootRingSlot> = roots_with_ring_slots
        .iter()
        .map(|(ring_slot, _)| *ring_slot)
        .collect();
    // 迭代上界是两遍里只有一遍自证过的槽数（至多根环的槽数）；跨轮只往交回的根里加；没有提前出口。上面那一遍读坏、当没有根的才在这里重读一次。
    for (ring_slot, root_record_self_verified_on_one_rebuild_pass) in
        &ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass
    {
        if ring_slots_read.contains(ring_slot) {
            continue;
        }
        match read_root_ring_slot(
            reader,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
            *ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root) => {
                roots_with_ring_slots.push((*ring_slot, root))
            }
            RootRingSlotReading::Bad(_still_bad) => roots_with_ring_slots
                .push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),
        }
    }
    let placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: Vec<PlacementOnDevice> =
        ring_slots_recorded_as_holding_an_abandoned_root
            .into_iter()
            .filter(|(ring_slot, _)| !ring_slots_read.contains(ring_slot))
            .flat_map(|(_, recorded_placements)| recorded_placements)
            .collect();
    Ok(RootsReadForRaisingTheFloor {
        roots: roots_with_ring_slots
            .into_iter()
            .map(|(_, root)| root)
            .collect(),
        placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
    })
}

/// 抬 F 那一遍读根环读出来的（[`readable_roots_for_raising_the_floor`]）。
struct RootsReadForRaisingTheFloor {
    /// 读得出、自证过的每一条根，与两遍里只有一遍自证过、这一遍重读仍坏的槽记着的那条根。
    roots: Vec<RootRecord>,
    /// 根环表里记成被抛弃根、这一遍读坏的槽各记着的落点（摊平）：影子账照它补隔离（H1）。
    placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: Vec<PlacementOnDevice>,
}

/// 抬 F 那一遍的回收门槛里「环里最旧有效根」（D16（发布语义） 已定项 1，用户 2026-10-04 定 RGall：不按一圈窗口滤）：这一遍读出来的根里
/// 按 `table` 有效的，并上分配器根环表里挂载那一刻读坏或读回全零、这一遍也没在那一槽读出根的槽（各钉在它可能住过的最旧 txg，用户 2026-10-04 定，
/// 读回全零的 2026-10-05 定乙；
/// 并进来是 RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一，攻方 B1），取最小；最小的是一个读坏的槽时照 E2
/// 先重读那一槽（[`oldest_txg_in_the_ring_rereading_slots_bad_at_mount`]，与进程内记根同一取法）。重建分配器两遍里只有一遍自证过、
/// 这一遍重读一次仍坏的槽不另钉：顶上的那条根已在 `roots_read_from_the_ring` 里（用户 2026-10-06 定 T1 甲）。只读盘，不改分配器。
fn oldest_valid_root_in_the_ring_for_raising_the_floor<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    allocator: &PoolAllocator,
    roots_read_from_the_ring: &[RootRecord],
    table: &InstanceTableRecords,
) -> Option<CheckpointTxg> {
    let slots_per_region = parameters_of_the_pool.geometry.root_ring_slots_per_region;
    let valid_root_txgs: Vec<CheckpointTxg> = roots_read_from_the_ring
        .iter()
        .filter(|root| !abandoned_by_table(root, table))
        .map(|root| root.checkpoint_txg)
        .collect();
    let ring_slots_read_from_the_ring: BTreeSet<RootRingSlot> = roots_read_from_the_ring
        .iter()
        .map(|root| target_for_publish(root.checkpoint_txg, slots_per_region))
        .collect();
    let bad_at_mount: BTreeMap<RootRingSlot, CheckpointTxg> = allocator
        .root_ring_occupancy()
        .map_or_else(
            BTreeMap::new,
            RootRingOccupancy::ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held,
        )
        .into_iter()
        .filter(|(ring_slot, _)| !ring_slots_read_from_the_ring.contains(ring_slot))
        .collect();
    oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
        &valid_root_txgs,
        bad_at_mount,
        &mut |ring_slot| {
            reread_of_a_root_ring_slot_bad_at_mount(
                reader,
                parameters_of_the_pool,
                table,
                ring_slot,
            )
        },
    )
    .0
}

/// 抬 F 那一串（准入与卸载共用，D16（发布语义） 已定项 1「抬 F 那一串」）。两个入口只差 [`RaiseFloorEntry`] 那三样：
/// 判不判上限、根带不带卸载记号、回收出来的槽什么时候能发出去（[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）。
fn raise_the_floor_through<Device: BlockDevice>(
    entry: RaiseFloorEntry,
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
    shadow_ledger: ShadowLedger,
) -> Result<FloorRaisedThroughTheSequence, MountError> {
    // 分配器上冻结着一次没重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：这一串的第一次空发布本来就会被拒，
    // 而下面的影子账与回收在发布之前就动分配器——重发成功时分配器整个换成那次发布之后的一份，这些改动会被一起丢掉。第一道就拒，一样都不动。
    if let Some(frozen) = allocator.frozen_publish() {
        return Err(MountError::RaiseFloorSequencePublishFailed(Box::new(
            PublishSequenceFailed {
                cause: PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: frozen.checkpoint_txg(),
                    instance: frozen.instance(),
                },
                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
                writes_of_persisted_publishes: Vec::new(),
                writes_of_failed_publishes: Vec::new(),
            },
        )));
    }
    // 下面的影子账重算与回收在第一次空发布之前就动分配器：第一次空发布在任何写之前被拒（落点、准入、释放判定），这一串一次都没落盘、
    // F 没有一条根带出去，分配器整个换回这一份——扣住的槽放开、回收的回到 defer、补的隔离撤掉（C546（抬 F 被拒时扣住的槽不退回））。
    let allocator_before_the_raise = allocator.clone();
    // 调用方交进来的参数与盘表照可写挂载同一套核，写入口只按盘上那一份建（实审 A1b Q2、Q3）：对不上在任何写之前拒，分配器还没动。
    let CallerInputsAgreeingWithTheDisk {
        system_configuration,
        parameters_of_the_pool,
    } = caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;
    // 候选集按现行那一版的实例表判：从它的根指着的第 0 片沿链真读出来、解出来（C502（抬 F 时现行版本里没有实例表单元）；
    // D18（块里携带什么信息） 已定项 11：一张表可以不止一片）。不看 `TransactionOutput::units`——那是这个进程内存里的角色列表，
    // 第一个文件版本那一版起就不带实例表单元，mkfs 同一个进程里后面发多少次都一样，而表就在根指针后面、读得出也解得开。
    let table = instance_table_chain_of_root(&*devices, &current.root)
        .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?
        .records;
    // F 的生效值只增不减（「抬 F 那一串」那一行）：新 F 低于盘上现算的生效值，往下「抬」条款没写，在任何写之前拒。
    // 先写系统配置那一步写之前按盘上现状再算一次、取两者的大者（`transaction::RollbackFloorOfASystemConfigurationWrite::RaisedFloor`）。
    // 判「有效根」的那张表（根环里最新那条根的实例表）读不出就重读一次，仍读不出就拒这一次抬，不按「不按表滤」算（C554 乙报告 Q6）：
    // 在动分配器与任何写之前返回，分配器还没动。
    let effective_floor = effective_rollback_floor_rereading_the_newest_instance_table_once(
        &**devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .map_err(|still_unreadable| {
        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
    })?;
    if new_floor < effective_floor {
        return Err(
            MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
                requested: new_floor,
                effective: effective_floor,
            },
        );
    }
    let ceiling_judged_by_the_admission = match entry {
        RaiseFloorEntry::Admission => {
            let ceiling = rollback_floor_ceiling_of_this_process(
                devices,
                &system_configuration,
                current.root.rollback_floor,
                &table,
                allocator,
            )?;
            if new_floor > ceiling {
                return Err(MountError::RollbackFloorAboveCeiling {
                    requested: new_floor,
                    ceiling,
                });
            }
            Some(ceiling)
        }
        // 卸载抬到现行那一版的 txg，不另判上限（「抬 F 的上限」那一行：等价于上限取最新的持久有效根、不按盘取小）。
        RaiseFloorEntry::Unmount => None,
    };
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(
            txg,
            parameters_of_the_pool.geometry.root_ring_slots_per_region,
        );
        parameters_of_the_pool.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    // 这一串的 txg 在动分配器（影子账重算、回收）之前算：越过 u64::MAX 就拒（代码审阅第 36 条），分配器与盘都不动。
    let planned_txgs = txgs_of_the_publishes_carrying_the_floor_to_every_device(
        current.root.checkpoint_txg,
        &all_devices,
        &device_of_txg,
    )?;
    // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。但回收的槽在生效（每块盘都有带新 F 的根）之前
    // 不许发出去：带新 F 的空发布自己就在分配固定点，开放段满了会开到刚回收空的那一段（alloc-basis 第二轮云端攻方腿打中），所以先扣住、
    // 落满每块盘之后再放开——这是准入那一格；卸载那一格只扣释放代高于每块盘上最新有效根的那一截（用户 2026-10-03 定甲 GA，
    // `RaiseFloorEntry::reuse_of_the_reclaimed_slots`）。
    // 记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
    // 回收门槛与影子账读的这一遍根环，照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行分两类（与算上限那一遍同一个分法）：
    // 这个进程知道住着根的槽读坏就重读那一槽一次，仍坏就拒这一次抬（C554 乙报告 Q6：按「没有根」往下走，那一槽里的被抛弃根引用的槽
    // 不隔离、也不计数）；挂载那一刻就读坏、之后没写过的槽当没有根；重建分配器两遍里只有一遍自证过的槽读坏也重读一次，仍坏不拒、
    // 拿重建那一遍读出、记在根环表里的那条根顶上（用户 2026-10-06 定 T1 甲，[`readable_roots_for_raising_the_floor`]）：被抛弃的照它引用的落点
    // 进下面的影子账重算（不顶上时它引用、挂载那一刻被候选根豁免的槽按新 F 回收之后发得出去，判决第二节 K1 攻方 K1-1，I-7.4 红），
    // 有效的照它本身进回收门槛与候选集。在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。
    // 根环表里记成被抛弃根的槽这一遍读坏不拒（用户 2026-10-08 定 H1）：交回它们记着的落点，下面的影子账照它补隔离。
    let RootsReadForRaisingTheFloor {
        roots,
        placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
    } = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;
    // 根环表里挂载那一刻读坏、这一遍也没读出根的槽，钉在它可能住过的最旧 txg 并进环里最旧有效根（RR4，攻方 B1；用户 2026-10-04 定钉在最旧 txg；与进程内记根同一取法，
    // E2 的重读照用：它成了最小的那个时先重读，`oldest_valid_root_in_the_ring_for_raising_the_floor`）。
    let oldest_valid_root_in_the_ring = oldest_valid_root_in_the_ring_for_raising_the_floor(
        &**devices,
        &parameters_of_the_pool,
        allocator,
        &roots,
        &table,
    );
    // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
    // 不然那个槽回收之后就发得出去（步 4 / 步 5 代码三方第二轮辩方腿：窄读法要按每次挂载与每次抬 F 的候选集现算）。
    // 水位之上没施加的记录点名、还没解开的落点（C694 的 C）按新候选集同一次重算：此前靠一条低于新 F 的根豁免的那几个，回收之前补上隔离。
    let placements_named_by_unapplied_journal_records: Vec<PlacementOnDevice> = allocator
        .root_ring_occupancy()
        .map(RootRingOccupancy::placements_named_by_unapplied_journal_records)
        .unwrap_or_default();
    // 抬 F 那一串里的被抛弃根（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，用户 2026-10-07 定，判决
    // `research/prompts/m3-batch10y-code-r1-main-verification.md` 第三节）：这一遍读出的、按现行那一版的实例表被抛弃的根，根环表里那一槽记着的
    // 是同一条（实例代号与 txg 都对上）就照挂载时算出、记着的落点进影子账，不再从盘上读它的账、读不出也不拒（不走 C694 的 B）。
    // 改之前照盘上重读，挂上之后它的账读坏就照 B 拒，正常卸载与会话写都被拒（攻方 `research/prompts/m3-batch10y-code-r1-opus-output.md` 「K4-2」）。
    // 表里没记着这条被抛弃根、挂载那一刻那一槽记成同一条有效根（写行之后才被抛弃）的，借 C 那一圈在那一槽记着的落点
    // （用户 2026-10-08 定 H2「用 C 那一圈记着的落点」，判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第三节）：
    // 同样不读账、读不出不拒。两样都没记着的照旧读盘（[`PlacementsOfAbandonedRoots`] 那一臂的注释）。那条根挂载那一刻账读不出而记录齐时，
    // C 那一槽另带着挂载那一刻现行账里仍分配着的落点（丙，用户 2026-10-08 定，[`placements_in_the_accounts_of_newer_readable_roots`]）。
    // C 那一槽有条目而落点是空的（凭据放行那一形），照空串算、不读账（「照 C 那一圈记着的落点算」，与根环表里记着空串同一处置）。
    let placements_recorded_for_abandoned_roots: BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        Vec<PlacementOnDevice>,
    > = allocator
        .root_ring_occupancy()
        .map(|occupancy| {
            roots
                .iter()
                .filter(|root| abandoned_by_table(root, &table))
                .filter_map(|root| {
                    occupancy
                        .placements_recorded_for_the_abandoned_root(
                            root.instance,
                            root.checkpoint_txg,
                        )
                        .or_else(|| {
                            occupancy
                                .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
                                    root.instance,
                                    root.checkpoint_txg,
                                )
                        })
                        .map(|recorded| ((root.instance, root.checkpoint_txg), recorded))
                })
                .collect()
        })
        .unwrap_or_default();
    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {
        match isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            unit_area_start_of_the_chosen_system_configuration(&system_configuration),
            allocator,
            &roots,
            &|root| abandoned_by_table(root, &table),
            new_floor,
            &current.allocation_records,
            &placements_named_by_unapplied_journal_records,
            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
                &placements_recorded_for_abandoned_roots,
            ),
            &mut BeforeTheOneReread::RereadImmediately,
            &placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
        ) {
            Ok(computed) => computed.abandoned_roots_unreadable,
            Err(error) => {
                *allocator = allocator_before_the_raise;
                return Err(error);
            }
        }
    } else {
        0
    };
    // 卸载那一格当场放开的那一截（`RaiseFloorEntry::reuse_of_the_reclaimed_slots`，GA）：下面的整串预演就在分配器的拷贝上算上它们；
    // 其余那一截与准入那一格一样扣到生效。分配器记着回收过的槽不重复回收，第二次只回收第一次没回收的那几个。
    let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_ring);
    let reclaimed = match entry.reuse_of_the_reclaimed_slots() {
        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => {
            reclaim_released_in_the_sequence_raising_the_floor_up_to(
                allocator,
                shadow_ledger,
                floor_of_the_reclaim,
                ReclaimedReuse::HeldUntilFloorTakesEffect,
            )
        }
        ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice => {
            let roots_on_the_devices_of_their_txgs: Vec<(DeviceIdentity, RootRecord)> = roots
                .iter()
                .map(|root| (device_of_txg(root.checkpoint_txg), *root))
                .collect();
            let mut reclaimed = match newest_valid_root_txg_on_every_device(
                &roots_on_the_devices_of_their_txgs,
                &table,
                &all_devices,
            ) {
                Some(newest_valid_root_on_every_device) => {
                    reclaim_released_in_the_sequence_raising_the_floor_up_to(
                        allocator,
                        shadow_ledger,
                        floor_of_the_reclaim.min(newest_valid_root_on_every_device),
                        ReclaimedReuse::Immediately,
                    )
                }
                None => Vec::new(),
            };
            reclaimed.extend(reclaim_released_in_the_sequence_raising_the_floor_up_to(
                allocator,
                shadow_ledger,
                floor_of_the_reclaim,
                ReclaimedReuse::HeldUntilFloorTakesEffect,
            ));
            reclaimed
        }
    };
    let publishes_in_the_sequence = planned_txgs.len();
    let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
    // 这一串在任何写之前在分配器的一份拷贝上整串预演（与可写挂载取号之前那一串同一个做法，`dry_run_of_the_publishes_after_acquisition`）：
    // 第 n 次在拷贝上报错（取不到落点、别的落盘之前的错），这一串一次都不发——F 不带出去，分配器整个换回抬 F 之前那一份
    // （扣住的槽放开、回收的回到 defer、补的隔离撤掉，C546（抬 F 被拒时扣住的槽不退回））。不预演时前 n − 1 次已经落盘：
    // 盘上带着新 F 而 F 没在每块盘上生效，扣住的槽留在这个进程里、计数上是空闲而发不出去（实二五报告第六节 Q3），
    // 而回退留下空档时那条落了盘的新 F 让 checker 在合法状态上判 I-3.1 红（增补 2 收口表第 43 行那一形，
    // 代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第三节「越格线索」的两个种子）。
    let placements_rehearsed = match rehearse_the_publishes_raising_the_floor(
        &pool,
        allocator,
        current,
        new_floor,
        &planned_txgs,
        entry.unmount_marker(),
    ) {
        Ok(placements_rehearsed) => placements_rehearsed,
        Err(refusal) => {
            *allocator = allocator_before_the_raise;
            return Err(
                MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                    refused_publish_in_the_sequence: refusal.refused_publish_in_the_sequence,
                    publishes_in_the_sequence,
                    cause: refusal.cause,
                },
            );
        }
    };
    // 预演过了、第一次发布之前：先把新 F 写进每块盘的系统配置、过一道屏障，之后才发第一条带新 F 的根（SysPre，D16（发布语义） 已定项 1
    // 「抬 F 那一串」、已定项 7）。于是任何一条带新 F 的根落盘时，它那块盘的系统配置里已经有新 F（I-7.12（系统配置 F 不低于同盘根上的 F））；
    // 带新 F 的根全坏了，生效值照样读得出新 F。这一步报错：这一串的根一条都不发，已写进的新 F 留着、不回卷，分配器换回抬 F 之前那一份。
    let system_configuration_writes_before_the_first_publish =
        match write_the_raised_floor_into_every_system_configuration(
            &mut pool,
            new_floor,
            current.record.counter,
            current.record.instance,
        ) {
            Ok(writes) => writes,
            Err(failed) => {
                *allocator = allocator_before_the_raise;
                return Err(
                    MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(Box::new(
                        failed,
                    )),
                );
            }
        };
    let mut publishes = Vec::new();
    // 迭代上界是预演过的次数（至多根环区域数）；跨轮携带的是现行那一版（下一次的计划接着它）。
    for planned_txg in &planned_txgs {
        let published = publish_version_with_unmount_marker(
            &mut pool,
            allocator,
            empty_publish_plan_raising_the_floor(current, new_floor),
            Some(&*current),
            entry.unmount_marker(),
        );
        let next = match published {
            Ok(next) => next,
            Err(cause) => {
                // 预演过了还在这里报错的只剩两种：落盘途中失败（那一次冻结在分配器上等原样重发，它的字节按回收之后的账装的，不换），
                // 与真发时读盘核出对不上、那一份留在已分配，而那一槽在拷贝上被这一串自己回收又发了出去（两边分叉，
                // 同 `establish_instance` 那一格）。第一次就在任何写之前被拒的，这一串什么都没落盘，分配器换回抬 F 之前的那一份；
                // 第二次起被拒的，前面已落盘的根带着新 F 与回收之后的账，扣住位照旧留在这个进程里，下一次抬 F 做成才放开
                // （C516（抬 F 那一串发布被拒时前面几次已落盘）；这一格扣住位怎么放，条款没写）。
                if publishes.is_empty() && allocator.frozen_publish().is_none() {
                    *allocator = allocator_before_the_raise.clone();
                }
                // 抬 F 的写入口是这里开的、随错一起丢掉：已经落盘的那几次各自的写与失败那一次已记的写都随错交出（增补 2 收口表第 58 行）。
                return Err(MountError::RaiseFloorSequencePublishFailed(
                    publish_sequence_failed(
                        &pool,
                        system_configuration_writes_before_the_first_publish.clone(),
                        publishes
                            .iter()
                            .map(|persisted: &TransactionOutput| persisted.writes.clone())
                            .collect(),
                        cause,
                    ),
                ));
            }
        };
        assert_eq!(
            next.root.checkpoint_txg, *planned_txg,
            "真发的 txg 与这一串计划里的那一个相同：计划从现行那一版的 txg 逐个加一，每一次都接着现行那一版加一"
        );
        publishes.push(next.clone());
        *current = next;
    }
    // 预演取的落点就是真发取的：同一个分配器、同一串分配动作。有一份换下的单元读盘核对不上时不比（那一份真发时留在已分配，
    // 拷贝上照常释放，那一槽若在这一串里被回收两边可以不同，同 `establish_instance` 那一格）。
    let some_copy_was_quarantined = publishes.iter().any(|version: &TransactionOutput| {
        !version
            .quarantined_after_release_checksum_mismatch
            .is_empty()
    });
    if !some_copy_was_quarantined {
        let placements_taken: Vec<PlacementsTakenByOnePublish> = publishes
            .iter()
            .map(placements_taken_by_a_file_version)
            .collect();
        assert_eq!(
            placements_taken, placements_rehearsed,
            "抬 F 之前在分配器的拷贝上预演取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
             不同说明两处的计划、次序或记根分叉了，预演判的不是真发的那一串"
        );
    }
    allocator.release_reclaim_holds();
    Ok(FloorRaisedThroughTheSequence {
        ceiling_judged_by_the_admission,
        system_configuration_writes_before_the_first_publish,
        publishes,
        reclaimed,
        abandoned_roots_unreadable,
    })
}

/// 抬 F 那一串空发布的 txg：从现行那一版的下一个 txg 起逐次加一，直到每块盘上都落过一条（那一次的根落在哪块盘由根环落点公式定，
/// `device_of_txg`），至多根环区域数那么多次（D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」）。预演与真发都按这一串推。
///
/// # Errors
/// 下一个 txg 越过 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条；调用方在动分配器与任何写之前算它）。
fn txgs_of_the_publishes_carrying_the_floor_to_every_device(
    current_txg: CheckpointTxg,
    all_devices: &[DeviceIdentity],
    device_of_txg: &dyn Fn(CheckpointTxg) -> DeviceIdentity,
) -> Result<Vec<CheckpointTxg>, MountError> {
    let mut covered: Vec<DeviceIdentity> = Vec::new();
    let mut publishes: Vec<CheckpointTxg> = Vec::new();
    // 迭代上界是根环区域数；跨轮携带的是已经落到了哪几块盘与上一次的 txg。
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let txg = checkpoint_txg_after_the_version(
            publishes.last().map_or(current_txg, |previous| *previous),
        )?;
        let device = device_of_txg(txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        publishes.push(txg);
    }
    Ok(publishes)
}

/// 计划里的下一次空发布接在现行那一版后面：它的 txg 加一。计划那一串在任何写之前按同一个加一算过
/// （`warm_up_publish_txgs`、`txgs_of_the_publishes_carrying_the_floor_to_every_device`，越过 `u64::MAX` 就在那里拒）。
///
/// # Panics
/// 现行那一版的 txg 已是 `u64::MAX`：那几个计划函数先拒了，走到这里说明调用方没按计划推。
fn checkpoint_txg_of_the_planned_publish_after(current: CheckpointTxg) -> CheckpointTxg {
    CheckpointTxg(current.0.checked_add(1).expect(
        "计划那一串在任何写之前按同一个加一算过、没越过 u64::MAX（warm_up_publish_txgs / txgs_of_the_publishes_carrying_the_floor_to_every_device）",
    ))
}

/// 抬 F 那一串的一次空发布的计划：接在现行那一版之后，带新 F。预演与真发读同一张计划（`raise_rollback_floor`）。
fn empty_publish_plan_raising_the_floor<'plan>(
    current: &TransactionOutput,
    new_floor: CheckpointTxg,
) -> PublishPlan<'plan> {
    PublishPlan {
        txg: checkpoint_txg_of_the_planned_publish_after(current.root.checkpoint_txg),
        counter: current.record.counter + 1,
        transaction: 0,
        highest_transaction_number_before_this_publish: current
            .highest_transaction_number_in_this_instance,
        instance: current.root.instance,
        back_chain: back_chain_of(&current.record_bytes),
        file: None,
        // 抬 F 的这几次空发布不碰 inode 树。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current.root.instance_table),
        tree_birth_txg: current.tree_birth_txg(),
        // 一个树 ID 都不发：现行那一版的水位就是根环里的 max（本会话每次发布只照抄或推高它）。
        tree_identifier_watermark: current.root.tree_identifier_watermark,
        rollback_floor: new_floor,
    }
}

/// 抬 F 那一串的回收（[`raise_rollback_floor`]）：影子账开着时回收出来的槽里隔离还护着的当场补隔离
/// （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）——C 那一圈的落点在抬 F 重算影子账那一刻
/// 分配位置着的不隔离（D28（挂载期承诺量） 已定项 1 第九项，[`isolate_slots_referenced_only_by_abandoned_roots`] 那一圈的注释），
/// 不在这里补，它们回收之后就发得出去。影子账关着（只供测试的对照，什么都不护）时只回收。
fn reclaim_released_in_the_sequence_raising_the_floor_up_to(
    allocator: &mut PoolAllocator,
    shadow_ledger: ShadowLedger,
    floor: CheckpointTxg,
    reuse: ReclaimedReuse,
) -> Vec<Placement> {
    match shadow_ledger {
        ShadowLedger::On => allocator
            .reclaim_released_up_to_isolating_what_the_isolation_still_protects(floor, reuse),
        ShadowLedger::Off => allocator.reclaim_released_up_to(floor, reuse),
    }
}

/// 抬 F 那一串在预演里第几次（从 1 数）报了什么错。
struct RaiseFloorRehearsalRefusal {
    refused_publish_in_the_sequence: usize,
    cause: PublishError,
}

/// 抬 F 那一串空发布（txg 依次是 `planned_txgs`）在分配器的一份拷贝上逐次预演：只动拷贝、不碰盘，走的就是发布路径落盘之前那一段
/// （`transaction::prepare_the_version_publish`，真发的 `publish_version_with_unmount_marker` 走同一段、给同一个 `unmount_marker`），
/// 换下的每一份不读盘核（`ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition`）、照常释放。交回每一次取到的落点。
///
/// # Errors
/// 第几次（从 1 数）的准备报了什么错（[`RaiseFloorRehearsalRefusal`]）。
fn rehearse_the_publishes_raising_the_floor<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    new_floor: CheckpointTxg,
    planned_txgs: &[CheckpointTxg],
    unmount_marker: UnmountMarker,
) -> Result<Vec<PlacementsTakenByOnePublish>, RaiseFloorRehearsalRefusal> {
    let mut copy = allocator.clone();
    let mut rehearsed_current = current.clone();
    let mut taken_by_every_publish = Vec::with_capacity(planned_txgs.len());
    // 迭代上界是这一串的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）。
    for (publish_index, planned_txg) in planned_txgs.iter().enumerate() {
        let (_, rehearsed) = prepare_the_version_publish(
            pool,
            &mut copy,
            &empty_publish_plan_raising_the_floor(&rehearsed_current, new_floor),
            Some(&rehearsed_current),
            rehearsed_current.tree_identifiers,
            ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
            unmount_marker,
            &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
        )
        .map_err(|cause| RaiseFloorRehearsalRefusal {
            refused_publish_in_the_sequence: publish_index + 1,
            cause,
        })?;
        assert_eq!(
            rehearsed.root.checkpoint_txg, *planned_txg,
            "预演的 txg 与这一串计划里的那一个相同：计划从现行那一版的 txg 逐个加一，预演每一次都接着预演出来的现行那一版加一"
        );
        taken_by_every_publish.push(placements_taken_by_a_file_version(&rehearsed));
        rehearsed_current = rehearsed;
    }
    Ok(taken_by_every_publish)
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

/// 写行那次发布、上一版带文件的计划：在上一版那张实例表后面接上这次写的行，重写整条实例表链与固定点单元（D18（块里携带什么信息） 已定项 11；
/// 记账树已经存在 ⇒ 空发布也重写固定点单元，D16（发布语义） 已定项 9）。事务号 0、本实例第一条反向链 0。
/// 取号之后的发布与取号之前的预演读的是同一张计划（`publish_rows_on_file_version`、`dry_run_of_the_publishes_after_acquisition`）。
fn row_publish_plan_on_file_version<'plan>(
    previous: &TransactionOutput,
    instance_table: InstanceTableRewrite,
    identity: &RowPublishIdentity,
) -> PublishPlan<'plan> {
    PublishPlan {
        txg: identity.txg,
        counter: identity.counter,
        transaction: 0,
        // 新实例的第一条记录（反向链恒 0），事务号按实例各算各的，从这里重新从 1 起。
        highest_transaction_number_before_this_publish: 0,
        instance: identity.instance,
        back_chain: 0,
        file: None,
        // 写行那次发布不碰 inode 树。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Rewrite(instance_table),
        tree_birth_txg: previous.tree_birth_txg(),
        tree_identifier_watermark: identity.tree_identifier_watermark,
        rollback_floor: identity.rollback_floor,
    }
}

/// 写行那次发布、上一版带文件：按 [`row_publish_plan_on_file_version`] 发。
fn publish_rows_on_file_version<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    instance_table: InstanceTableRewrite,
    identity: RowPublishIdentity,
) -> Result<TransactionOutput, PublishError> {
    publish_version(
        pool,
        allocator,
        row_publish_plan_on_file_version(previous, instance_table, &identity),
        Some(previous),
    )
}

/// 暖机的一次空发布接在带文件的现行那一版后面时的计划：重写固定点单元（记账树存在，D16（发布语义） 已定项 9），不写文件内容、不碰 inode 树、
/// 实例表照抄。取号之后的发布与取号之前的预演读的是同一张计划；会话的推环（`mounted_session`）发的也是这一张（F 不变）。
pub(crate) fn empty_publish_plan_after_file_version<'plan>(
    current_file_version: &TransactionOutput,
    instance: InstanceGeneration,
) -> PublishPlan<'plan> {
    PublishPlan {
        txg: checkpoint_txg_of_the_planned_publish_after(current_file_version.root.checkpoint_txg),
        counter: current_file_version.record.counter + 1,
        transaction: 0,
        highest_transaction_number_before_this_publish: current_file_version
            .highest_transaction_number_in_this_instance,
        instance,
        back_chain: back_chain_of(&current_file_version.record_bytes),
        file: None,
        // 暖机那几次空发布不碰 inode 树。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current_file_version.root.instance_table),
        tree_birth_txg: current_file_version.tree_birth_txg(),
        // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
        tree_identifier_watermark: current_file_version.root.tree_identifier_watermark,
        rollback_floor: current_file_version.root.rollback_floor,
    }
}

/// 暖机的一次空发布接在树表 0 条的现行那一版后面时的计划（零单元，D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）。
fn empty_publish_plan_after_version_without_file(
    current_version_without_file: &VersionWithoutFilePublishOutput,
    instance: InstanceGeneration,
) -> ZeroUnitPublishPlan {
    ZeroUnitPublishPlan {
        txg: checkpoint_txg_of_the_planned_publish_after(
            current_version_without_file.root.checkpoint_txg,
        ),
        counter: current_version_without_file.record.counter + 1,
        instance,
        back_chain: back_chain_of(&current_version_without_file.record_bytes),
        rollback_floor: current_version_without_file.root.rollback_floor,
        // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
        tree_identifier_watermark: current_version_without_file.root.tree_identifier_watermark,
    }
}

/// 暖机的一次空发布，接在现行那一版后面：带文件的一版重写固定点单元（记账树存在，D16（发布语义） 已定项 9），
/// 树表 0 条的一版写零个单元（同一条已定项「树表 0 条 ⇒ 零单元」）。
fn publish_empty_after<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    current: &PoolVersion,
    instance: InstanceGeneration,
) -> Result<PoolVersion, PublishError> {
    match current {
        PoolVersion::WithFile(current_file_version)
        | PoolVersion::WithTreesAndNoFile(current_file_version) => publish_version(
            pool,
            allocator,
            empty_publish_plan_after_file_version(current_file_version, instance),
            Some(current_file_version),
        )
        .map(PoolVersion::of_a_version_of_trees),
        PoolVersion::WithoutFile(current_version_without_file) => {
            let published = publish_without_units(
                pool,
                &current_version_without_file.root,
                empty_publish_plan_after_version_without_file(
                    current_version_without_file,
                    instance,
                ),
            )?;
            // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`），
            // 记之前照 E2 重读挂载时读不出的那一槽。
            record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
                &*pool.devices,
                pool.parameters,
                allocator,
                published.root.checkpoint_txg,
            );
            Ok(PoolVersion::WithoutFile(published))
        }
    }
}

/// 取号之后那一串里一次发布取到的落点：这次重写的角色，按取落点的次序（bump 次序）各一个槽。零单元的发布一个都不取。
type PlacementsTakenByOnePublish = Vec<(TransactionUnit, SlotNumber)>;

/// 取号之后那一串里第几次发布（从 0 数：0 是写行那次，之后是暖机）在分配器的拷贝上预演时报的错。
struct DryRunRefusal {
    publish_index: usize,
    cause: PublishError,
}

/// 取号之前把这次挂载取号之后要发的那一串——写行一次、暖机 `warm_up_publish_txgs` 那几次——在分配器的一份拷贝上整串预演一遍，
/// 交回每一次取到的落点。只动拷贝、不碰盘；**走的就是发布路径落盘之前那一段**（带文件的一版 `transaction::prepare_the_version_publish`，
/// 树表 0 条的一版上写行 `transaction::prepare_the_row_publish_on_a_version_without_file`，零单元发布
/// `transaction::prepare_the_publish_without_units`），计划也是取号之后那几次发布用的同一张（`row_publish_plan_on_file_version`、
/// `empty_publish_plan_after_file_version`、`empty_publish_plan_after_version_without_file`），不另推一份：
/// 这一串每一次重写哪些角色——分配记录树重写哪几个节点要在分配器上走到固定点才知道（D8（核心索引结构） 已定项 14）——
/// 与真发时是同一段代码算的。写行那次的释放核验、树的形状、落点取不到，都在这里先报出来，在取号之前拒（D18（块里携带什么信息） 已定项 11
/// 「可写挂载的顺序」第五个合取；增补 2 第 20a 行）。
///
/// 发布路径在取落点之前还读盘核换下的每一份的校验和，对不上（读不出也算）的那一份在它那块盘上的分配记录留在已分配、不释放
/// （D19（块指针的结构与宽度预算） 已定项 5），这里不读盘（`ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition`）、照常释放。
/// 两边只在那一槽这一串里被回收时分叉：拷贝上它回收了、可能再发出去，真发时它一直占着；这一串里回收它，得这一串自己的根把根环里
/// 比它旧的有效根全盖掉（崩溃恢复落到环里最旧的那条根、其余全被抛弃时走得到）。所以这一串里没有一份核出对不上时，这里取到的与真发起来取到的逐项相同
/// （`establish_instance` 在这一串发完之后断言）；有一份核出对不上时可能不同，那一格见 `establish_instance` 的注释。
///
/// # Errors
/// 第几次（从 0 数）的准备报了什么错（[`DryRunRefusal`]）。
fn dry_run_of_the_publishes_after_acquisition<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    start: &InstanceStart,
    instance_table_rewrite: &InstanceTableRewrite,
    rows_to_write: usize,
    warm_up_publish_txgs: &[CheckpointTxg],
    instance_to_acquire: InstanceGeneration,
) -> Result<Vec<PlacementsTakenByOnePublish>, DryRunRefusal> {
    let mut copy = allocator.clone();
    let refused_at = |publish_index: usize| {
        move |cause: PublishError| DryRunRefusal {
            publish_index,
            cause,
        }
    };
    let row_publish = match &start.previous {
        PreviousVersion::WithFile { output, .. }
        | PreviousVersion::WithTreesAndNoFile { output, .. } => {
            let plan = row_publish_plan_on_file_version(
                output,
                instance_table_rewrite.clone(),
                &RowPublishIdentity {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance: instance_to_acquire,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            );
            prepare_the_version_publish(
                pool,
                &mut copy,
                &plan,
                Some(output),
                output.tree_identifiers,
                ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                UnmountMarker::NotWrittenByTheUnmountSequence,
                &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
            )
            .map(|(_, rehearsed)| PoolVersion::of_a_version_of_trees(rehearsed))
            .map_err(refused_at(0))?
        }
        PreviousVersion::WithoutFile { root, .. } if rows_to_write == 0 => {
            let (_, rehearsed) = prepare_the_publish_without_units(
                pool,
                root,
                ZeroUnitPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance: instance_to_acquire,
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            );
            copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
            PoolVersion::WithoutFile(rehearsed)
        }
        PreviousVersion::WithoutFile { root, .. } => {
            prepare_the_row_publish_on_a_version_without_file(
                pool,
                &mut copy,
                root,
                InstanceTableOnlyPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance: instance_to_acquire,
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    instance_table: instance_table_rewrite,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            )
            .map(|(_, rehearsed)| PoolVersion::WithoutFile(rehearsed))
            .map_err(refused_at(0))?
        }
    };
    let mut taken_by_every_publish = vec![placements_taken_by(&row_publish)];
    let mut current = row_publish;
    // 迭代上界是暖机的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）。
    for (warm_up_position, planned_txg) in warm_up_publish_txgs.iter().enumerate() {
        let publish_index = warm_up_position + 1;
        let next = match &current {
            PoolVersion::WithFile(current_file_version)
            | PoolVersion::WithTreesAndNoFile(current_file_version) => {
                let plan = empty_publish_plan_after_file_version(
                    current_file_version,
                    instance_to_acquire,
                );
                prepare_the_version_publish(
                    pool,
                    &mut copy,
                    &plan,
                    Some(current_file_version),
                    current_file_version.tree_identifiers,
                    ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                    UnmountMarker::NotWrittenByTheUnmountSequence,
                    &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
                )
                .map(|(_, rehearsed)| PoolVersion::of_a_version_of_trees(rehearsed))
                .map_err(refused_at(publish_index))?
            }
            PoolVersion::WithoutFile(current_version_without_file) => {
                let (_, rehearsed) = prepare_the_publish_without_units(
                    pool,
                    &current_version_without_file.root,
                    empty_publish_plan_after_version_without_file(
                        current_version_without_file,
                        instance_to_acquire,
                    ),
                );
                copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
                PoolVersion::WithoutFile(rehearsed)
            }
        };
        assert_eq!(
            next.root().checkpoint_txg,
            *planned_txg,
            "预演的暖机 txg 与计划里的那一个相同：计划逐个加一，计划接着现行那一版的 txg 加一"
        );
        taken_by_every_publish.push(placements_taken_by(&next));
        current = next;
    }
    Ok(taken_by_every_publish)
}

/// 带文件的一版的一次发布真取到的落点，按取落点的次序：这次重写的每个角色（`TransactionOutput::rewritten`）。
fn placements_taken_by_a_file_version(output: &TransactionOutput) -> PlacementsTakenByOnePublish {
    output
        .rewritten
        .iter()
        .map(|role| (*role, output.unit(*role).slot))
        .collect()
}

/// 一次发布真取到的落点，按取落点的次序：带文件的一版见 [`placements_taken_by_a_file_version`]；
/// 树表 0 条的一版是这次那条记录的点名项：写行那次按取落点的次序点名实例表链各片（尾片先）与分配记录树节点
/// （`transaction::publish_instance_table_on_version_without_file`；第 1 片起的指针不在根记录里，只在记录的点名项与上一片的链指针里），
/// 零单元发布一项都不点名、一个都没有。
fn placements_taken_by(version: &PoolVersion) -> PlacementsTakenByOnePublish {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
            placements_taken_by_a_file_version(output)
        }
        PoolVersion::WithoutFile(version_without_file) => {
            // 点名项按取落点的次序排在这次发布的各条记录里（末条再跨记录时前几条在 `earlier_records_of_this_publish`，
            // D23（journal 的角色与格式） 已定项 17），按记录的次序接起来就是整次发布的点名项。
            let named: Vec<&crate::journal::NamedUnit> = version_without_file
                .earlier_records_of_this_publish
                .iter()
                .map(|written| &written.record)
                .chain(std::iter::once(&version_without_file.record))
                .flat_map(|record| record.named.iter())
                .collect();
            assert_eq!(
                named.len(),
                version_without_file.rewritten.len(),
                "树表 0 条的一版上一次发布的点名项与它写出的角色一一对应（写行那次按同一张角色清单点名，零单元发布两边都空）"
            );
            version_without_file
                .rewritten
                .iter()
                .copied()
                .zip(named)
                .map(|(role, named_unit)| {
                (
                    role,
                    slot_shared_by_both_location_entries(&named_unit.locations).expect(
                        "这个进程这次发布刚写的指针，两条位置条目按一个落点写（两盘同槽，`PoolWriter::location_entries`）",
                    ),
                )
            })
            .collect()
        }
    }
}

/// 暖机要推的那几次空发布，按它们的 checkpoint_txg 列出来（D16（发布语义） 已定项 8 戊）：从写行那次发布的 txg 起逐个加一，
/// 按根环落点公式看落哪块盘，直到本实例的根覆盖每块盘，至多根环的区域数那么多次。
/// 纯算——只读区域归属表与池里的盘，不碰盘、不碰分配器、不看分配记录，所以取号之前算得出来。
/// 取号之前算准入与取号之后推发布共用这一份计划（`establish_instance` 按它推），两处的次数因此必定相同：
/// 各算各的时只要有一处多算一次，准入就罩不住实际推的那几次。
///
/// # Errors
/// 下一个 txg 越过 `u64::MAX` ⇒ `SequenceNumberPastTheTopOfItsRange`（代码审阅第 36 条；取号之前，一个写都没发）。
fn warm_up_publish_txgs(
    parameters: &MakeFilesystemParameters,
    all_devices: &[DeviceIdentity],
    row_publish_txg: CheckpointTxg,
) -> Result<Vec<CheckpointTxg>, MountError> {
    let device_of_txg = |txg: CheckpointTxg| {
        parameters.region_devices[usize::try_from(
            target_for_publish(txg, parameters.geometry.root_ring_slots_per_region).region,
        )
        .expect("区域号")]
    };
    let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish_txg)];
    let mut warm_up_publishes: Vec<CheckpointTxg> = Vec::new();
    let mut latest_txg = row_publish_txg;
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next_txg = checkpoint_txg_after_the_version(latest_txg)?;
        let device = device_of_txg(next_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        warm_up_publishes.push(next_txg);
        latest_txg = next_txg;
    }
    Ok(warm_up_publishes)
}

/// 这次挂载要写的行：给 [max(上一个实例, 1), 要取的号) 里每个实例各一行——上一个实例 (i, T, W)，
/// 中间实例 (i, 0, 0)；实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。纯算，取号之前就列得出来。
fn instance_rows_to_write(
    previous_row: &PreviousInstanceRow,
    instance_to_acquire: InstanceGeneration,
) -> Vec<InstanceRow> {
    let first_row_instance = previous_row.instance.0.max(1);
    (first_row_instance..instance_to_acquire.0)
        .map(|row_instance| {
            if row_instance == previous_row.instance.0 {
                InstanceRow {
                    instance: InstanceGeneration(row_instance),
                    selected_root_txg: previous_row.selected_root_txg,
                    applied_transaction_high_water: previous_row.applied_transaction_high_water,
                }
            } else {
                InstanceRow {
                    instance: InstanceGeneration(row_instance),
                    selected_root_txg: CheckpointTxg(0),
                    applied_transaction_high_water: 0,
                }
            }
        })
        .collect()
}

/// 一串发布里有一次失败了：错，连同这一串的写入口交得出的账——`writes_before_the_first_publish` 是第一次发布之前、不属于任何一次发布的写
/// （抬 F 那一串先写系统配置那一步；可写挂载那一串给空账），`writes_of_persisted_publishes` 是这一串里已经落盘的那几次各自的写
/// （按先后），失败那一次落盘阶段已记的写从写入口的失败账取。可写挂载与抬 F 共用这一处。
fn publish_sequence_failed<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    writes_before_the_first_publish: WritesByStructureKind,
    writes_of_persisted_publishes: Vec<WritesByStructureKind>,
    cause: PublishError,
) -> Box<PublishSequenceFailed> {
    Box::new(PublishSequenceFailed {
        cause,
        writes_before_the_first_publish,
        writes_of_persisted_publishes,
        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
    })
}

/// 取号之后的一次发布失败了：错，连同这次挂载的写入口交得出的账——`persisted` 是这次挂载里已经落盘的那几次（按先后）。
fn publish_failed_after_acquisition<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    persisted: &[&PoolVersion],
    cause: PublishError,
) -> MountError {
    MountError::Publish(publish_sequence_failed(
        pool,
        WritesByStructureKind::NOTHING_WRITTEN,
        persisted
            .iter()
            .map(|version| match version {
                PoolVersion::WithoutFile(version_without_file) => {
                    version_without_file.writes.clone()
                }
                PoolVersion::WithFile(file_version)
                | PoolVersion::WithTreesAndNoFile(file_version) => file_version.writes.clone(),
            })
            .collect(),
        cause,
    ))
}

/// 所选那一版在 journal 里的位置：它那次发布的末条记录的 jsn（调用方从环里认出来的那一条，认不出时它顶上的那一条）；
/// 一条都拿不出来（只做过 mkfs 的池，环里没有记录）时取 (这一版的实例代号, 0)。每块盘在一次发布之后轮换的系统配置写的是
/// 同一对 (实例代号, 末条计数器)（`transaction::CommitStep::RotateSystemConfigurationSlots`），逐盘判「落后」拿它比。
fn journal_position_of_the_selected_version(
    root: &RootRecord,
    record_standing_for_root: Option<&crate::journal::JournalRecord>,
) -> JournalSequenceNumber {
    record_standing_for_root.map_or(
        JournalSequenceNumber {
            instance_generation: root.instance,
            counter: 0,
        },
        |record| JournalSequenceNumber {
            instance_generation: record.instance,
            counter: record.counter,
        },
    )
}

/// 可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13、D18（块里携带什么信息） 已定项 11）：交进来的每块盘带不带所选那一版，
/// 交回不带的那几块与各自缺的（[`SelectedVersionLackingOnDevice`]）。只读盘，不写。
///
/// 每块盘先看它两槽里自证过的系统配置：一份都没有（读阶段的见证读两槽都读得出、到这一遍才读不出：两次读之间的瞬时读错）
/// 就交 `Err(那块盘)`，由调用方按 C331（择根倒挂压过已确认的写） 取甲的拒因报（用户 2026-09-28 定：这一支并进甲的拒因）；世代号最大的那一份的 (实例代号, journal tail) 不落后于
/// `selected_version_journal_position` 就带（跟上了这一版的系统配置轮换，不读它的单元）；落后的按落后支那一判（[`what_a_device_behind_the_version_lacks`]，
/// 与挂着之后收盘表的入口同一判）：逐个读 `units` 在这块盘上的落点，读不出或字节与验得过的那一份不同就记下，记下了至少一个才算不带；
/// 单元都在、所选那一版的根写进了根环（`root_of_the_selected_version`）而住在这块盘上的，再读那个根环槽，读一次重读一次都不是它也算不带。
/// 字节是空的单元（提示与映射都读不出的数据单元，D19（块指针的结构与宽度预算） 已定项 5）没有验得过的一份可比，跳过。
///
/// 循环：外层按交进来的盘（轮数 = 盘数），内层按 `units`（轮数 = 这一版的单元数）；跨轮只往交回的清单里追加。
/// 外层的提前出口两个：没有自证过的系统配置（整个核交 `Err`）、系统配置不落后（这块盘判完了、看下一块，不记）。
///
/// # Errors
/// 某块盘两槽里一份自证过的系统配置都读不出 ⇒ `Err(那块盘)`。
fn devices_without_the_selected_version<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    selected_version_journal_position: JournalSequenceNumber,
    units: &[PublishedUnit],
    root_of_the_selected_version: RootOfTheVersionInTheRing,
) -> Result<Vec<DeviceWithoutTheSelectedVersion>, DeviceIdentity> {
    let parameters_of_the_pool = parameters_of_the_system_configuration(system_configuration);
    let mut devices_without = Vec::new();
    for device in reader.device_identities() {
        let newest_self_verified = self_verified_system_configuration_slots_of_the_pool(
            reader,
            device,
            &parameters_of_the_pool,
        )
        .into_iter()
        .max_by_key(|slot| slot.quantities.slot_generation);
        let Some(newest_self_verified) = newest_self_verified else {
            return Err(device);
        };
        let Some((newest_system_configuration_journal_tail, lacking)) =
            what_a_device_behind_the_version_lacks(
                reader,
                &parameters_of_the_pool,
                device,
                &newest_self_verified,
                selected_version_journal_position,
                units,
                root_of_the_selected_version,
            )
        else {
            continue;
        };
        let lacking = match lacking {
            VersionLackingOnADeviceBehindIt::UnitsMissing(units_missing) => {
                SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                    newest_system_configuration_journal_tail,
                    units_missing,
                }
            }
            VersionLackingOnADeviceBehindIt::RootNotInItsRingSlot(root_not_in_its_ring_slot) => {
                SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndItsRootNotInItsRingSlot {
                    newest_system_configuration_journal_tail,
                    root_not_in_its_ring_slot,
                }
            }
        };
        devices_without.push(DeviceWithoutTheSelectedVersion { device, lacking });
    }
    Ok(devices_without)
}

/// 读阶段的见证读判完之后，逐盘核或取号那一遍又读到判不出（R2）或某块盘一份自证过的系统配置都没有（两次读之间的瞬时读错就够）：
/// 拒可写，报 [`MountError::SystemConfigurationUnreadableAfterTheReadStage`]，带读阶段往下走用的那一遍读数与 `unreadable`。
/// 不并进读阶段判据 N-配置 那一拒（C664（挂载拒绝合法复算把两种拒因报成一个））。一个字节都没写。
fn refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
    read_stage: ReadStageSettled,
    unreadable: SystemConfigurationUnreadableWhen,
) -> MountError {
    let settled = match read_stage {
        ReadStageSettled::OnTheFirstRead { first_read } => first_read,
        ReadStageSettled::OnTheOneReread { reread, .. } => reread,
    };
    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
        SystemConfigurationUnreadableAfterTheReadStage {
            read_stage: settled,
            unreadable,
        },
    ))
}

/// 落后支缺单元那一半（[`what_a_device_behind_the_version_lacks`] 先判它）：`newest_self_verified` 是这块盘两槽里本池自证过、世代号最大的那一份系统配置；它的 (实例代号, journal tail) 不落后于
/// `version_journal_position`（那一版那次发布末条记录的 jsn）就交回 `None`（跟上了这一版的系统配置轮换，不读它的单元）；落后的再逐个读
/// `units` 在这块盘上的落点，读不出或字节与验得过的那一份不同就记下，一个都没记下也交回 `None`（只落后、单元都在：根落盘之后、轮换之前崩，
/// 或只剩一槽自证、那一槽停在上一次轮换）；记下了至少一个才交回 (那份的 tail, 缺的单元)。字节是空的单元（提示与映射都读不出的数据单元，
/// D19（块指针的结构与宽度预算） 已定项 5）没有验得过的一份可比，跳过。只读盘。
///
/// 循环：按 `units`（轮数 = 这一版的单元数），跨轮只往缺的清单里追加，没有提前出口。
fn units_of_the_version_missing_on_a_device_behind_it<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    newest_self_verified: &crate::system_configuration::SystemConfiguration,
    version_journal_position: JournalSequenceNumber,
    units: &[PublishedUnit],
) -> Option<(JournalSequenceNumber, Vec<(TransactionUnit, SlotNumber)>)> {
    let newest_system_configuration_journal_tail = JournalSequenceNumber {
        instance_generation: newest_self_verified.quantities.journal_instance,
        counter: newest_self_verified.quantities.journal_tail,
    };
    if newest_system_configuration_journal_tail >= version_journal_position {
        return None;
    }
    let units_missing: Vec<(TransactionUnit, SlotNumber)> = units
        .iter()
        .filter(|unit| !unit.bytes.is_empty())
        .filter(|unit| {
            !reader
                .read(device, unit.slot.to_device_offset(), unit.bytes.len())
                .is_some_and(|bytes_on_this_device| bytes_on_this_device == unit.bytes)
        })
        .map(|unit| (unit.identity, unit.slot))
        .collect();
    (!units_missing.is_empty()).then_some((newest_system_configuration_journal_tail, units_missing))
}

/// 落后支的那一判（可写挂载取号之前逐盘核 [`devices_without_the_selected_version`] 与挂着之后收盘表的入口 [`devices_behind_the_current_version`]
/// 共用这一份；D18（块里携带什么信息） 已定项 11 落后支，C585（会话发布放进发布途中取下的旧快照） 用户定改法 C）：这块盘最新那份自证系统配置
/// （`newest_self_verified`）的 (实例代号, journal tail) 落后于 `version_journal_position`（那一版那次发布末条记录的 jsn），**并且**
/// 缺那一版的单元（[`units_of_the_version_missing_on_a_device_behind_it`]），**或**单元都在而那一版的根写在这块盘上、那个根环槽里不是它
/// （[`root_of_the_version_not_in_its_ring_slot_on_the_device`]），才交回 (那份的 tail, 缺的是什么)；不落后、或落后而两样都不缺的交回 `None`
/// （只落后、单元都在、根槽里是它：根落盘之后、轮换之前崩；只剩一槽自证、那一槽停在上一次轮换）。只读盘。
fn what_a_device_behind_the_version_lacks<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    device: DeviceIdentity,
    newest_self_verified: &crate::system_configuration::SystemConfiguration,
    version_journal_position: JournalSequenceNumber,
    units: &[PublishedUnit],
    root_of_the_version: RootOfTheVersionInTheRing,
) -> Option<(JournalSequenceNumber, VersionLackingOnADeviceBehindIt)> {
    if let Some((newest_system_configuration_journal_tail, units_missing)) =
        units_of_the_version_missing_on_a_device_behind_it(
            reader,
            device,
            newest_self_verified,
            version_journal_position,
            units,
        )
    {
        return Some((
            newest_system_configuration_journal_tail,
            VersionLackingOnADeviceBehindIt::UnitsMissing(units_missing),
        ));
    }
    let newest_system_configuration_journal_tail = JournalSequenceNumber {
        instance_generation: newest_self_verified.quantities.journal_instance,
        counter: newest_self_verified.quantities.journal_tail,
    };
    let is_behind_the_version = newest_system_configuration_journal_tail < version_journal_position;
    if !is_behind_the_version {
        return None;
    }
    let root_not_in_its_ring_slot = root_of_the_version_not_in_its_ring_slot_on_the_device(
        reader,
        parameters_of_the_pool,
        device,
        root_of_the_version,
    );
    root_not_in_its_ring_slot.map(|root_not_in_its_ring_slot| {
        (
            newest_system_configuration_journal_tail,
            VersionLackingOnADeviceBehindIt::RootNotInItsRingSlot(root_not_in_its_ring_slot),
        )
    })
}

/// 落后支缺根那一半：那一版的根写进了根环（`root_of_the_version`），它该在的根环槽（`root_ring::target_for_publish` 按 txg 算）
/// 住在 `device` 这块盘上，读那一槽一次，不是它（读坏、或自证得过而是另一条根）就重读一次，仍不是它才交回两次读到的
/// （D16（发布语义） 已定项 1「根槽这一次读坏」：知道住着根的槽重读一次仍坏才算；挂着之后现行那一版的根槽是这个进程写过、FUA 返回过的）。
/// 根没写进根环、或那一槽不在这块盘上的交回 `None`，不读盘。只读盘：至多读那一槽两次。
fn root_of_the_version_not_in_its_ring_slot_on_the_device<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    device: DeviceIdentity,
    root_of_the_version: RootOfTheVersionInTheRing,
) -> Option<VersionRootNotInItsRingSlot> {
    let (instance, checkpoint_txg) = match root_of_the_version {
        RootOfTheVersionInTheRing::Written {
            instance,
            checkpoint_txg,
        } => (instance, checkpoint_txg),
        RootOfTheVersionInTheRing::NotWritten => return None,
    };
    let ring_slot = target_for_publish(
        checkpoint_txg,
        parameters_of_the_pool.geometry.root_ring_slots_per_region,
    );
    let device_of_the_ring_slot = parameters_of_the_pool.region_devices
        [usize::try_from(ring_slot.region).expect("区域号是 txg 对区域数取模，小于 3")];
    if device_of_the_ring_slot != device {
        return None;
    }
    let reading_other_than_the_version_root = || match read_root_ring_slot(
        reader,
        &parameters_of_the_pool.region_devices,
        &parameters_of_the_pool.geometry,
        &parameters_of_the_pool.filesystem_identifier,
        ring_slot,
    ) {
        RootRingSlotReading::SelfVerified(root)
            if root.instance == instance && root.checkpoint_txg == checkpoint_txg =>
        {
            None
        }
        RootRingSlotReading::SelfVerified(another_root) => {
            Some(RingSlotReadingOtherThanTheVersionRoot::AnotherRoot {
                instance: another_root.instance,
                checkpoint_txg: another_root.checkpoint_txg,
            })
        }
        RootRingSlotReading::Bad(bad_reading) => {
            Some(RingSlotReadingOtherThanTheVersionRoot::Bad(bad_reading))
        }
    };
    let first_reading = reading_other_than_the_version_root()?;
    let reread = reading_other_than_the_version_root()?;
    Some(VersionRootNotInItsRingSlot {
        ring_slot,
        first_reading,
        reread,
    })
}

/// 挂着之后收盘表的入口在「可见」之后逐盘核的落后支（D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」的第三样，
/// 用户 2026-09-27 定；代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」）：交进来的每块盘，两槽里本池自证过、世代号最大的那一份
/// 系统配置落后于调用方持着的现行那一版（`current` 那次发布的末条记录），又缺它的单元、或它的根写在这块盘上而那个根环槽里不是它，
/// 就记下（[`DeviceBehindTheCurrentVersion`]）；判法与可写挂载取号之前逐盘核的落后支同一份（[`what_a_device_behind_the_version_lacks`]），
/// 「所选那一版」换成现行那一版，它的根恒写进了根环（每次发布在根槽 FUA 返回之后才换 `current`）。
/// 活的会话里每次发布在系统配置轮换落盘之后才返回（`transaction::CommitStep::RotateSystemConfigurationSlots`），现行那一版的盘都跟得上，
/// 只有被换成旧快照的盘落后；只剩一槽自证、那一槽停在上一次轮换的盘落后而单元都在、根槽里是它，不记（不误拒）。
/// 旧快照取在一次发布途中（单元与记录已写、根槽 FUA 还没写）、那次发布的根又写在这块盘上的，单元都在，靠根那一半拒（C585（会话发布放进发布途中取下的旧快照））。
/// 一份自证过的都没有的盘不在这里判（「可见」那一判先拒了它，[`devices_without_a_self_verified_system_configuration`]）。
/// 按交进来的次序交回。只读盘：每块盘读两个系统配置槽，落后的那几块再读现行那一版每个单元的落点，单元都在的再读至多两次它的根槽。
///
/// 循环：按交进来的盘逐块（轮数 = 盘数），跨轮只往交回的清单里追加；外层的提前出口一个：一份自证过的都没有（「可见」那一判的事，看下一块）。
pub(crate) fn devices_behind_the_current_version<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    current: &TransactionOutput,
) -> Vec<DeviceBehindTheCurrentVersion> {
    let current_version_journal_position = JournalSequenceNumber {
        instance_generation: current.record.instance,
        counter: current.record.counter,
    };
    let mut devices_behind = Vec::new();
    for device in reader.device_identities() {
        let Some(newest_self_verified) = self_verified_system_configuration_slots_of_the_pool(
            reader,
            device,
            parameters_of_the_pool,
        )
        .into_iter()
        .max_by_key(|slot| slot.quantities.slot_generation) else {
            continue;
        };
        if let Some((newest_system_configuration_journal_tail, lacking)) =
            what_a_device_behind_the_version_lacks(
                reader,
                parameters_of_the_pool,
                device,
                &newest_self_verified,
                current_version_journal_position,
                &current.units,
                RootOfTheVersionInTheRing::Written {
                    instance: current.root.instance,
                    checkpoint_txg: current.root.checkpoint_txg,
                },
            )
        {
            devices_behind.push(DeviceBehindTheCurrentVersion {
                device,
                newest_system_configuration_journal_tail,
                current_version_journal_position,
                lacking,
            });
        }
    }
    devices_behind
}

/// 交进来的每块盘自证过的系统配置槽（本池的 fsid、整槽校验和过，`recovery::verified_system_configuration_slots`，槽距取本池的固定结构槽距）
/// 里记的本盘设备号不是盘表给它的身份的各一项（一块盘上同一个号只报一次），按交进来的次序（D18（块里携带什么信息） 已定项 11
/// 「挂着之后收盘表的每个入口……核三样」的第二样）。可写挂载与挂着之后收盘表的入口经 [`device_table_disagreeing_with`] 调它，
/// 会话在每次用户改动之前调它（`mounted_session`；代码三方 m2-closeout-code-r2「盘体对调的盘表」：会话此前只比身份次序）。
/// 一份自证过的都没有的盘不在这里报。调用方先判过盘表里没有重复身份（按身份读盘，重复了读的是哪一块说不清）。读每块盘的两个系统配置槽，不写盘。
///
/// 循环：按交进来的盘逐块（轮数 = 盘数），每块读两槽；跨轮只往交回的清单里追加，没有提前出口。
pub(crate) fn own_device_numbers_on_disk_differing_from_the_identities_handed_in<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
) -> Vec<OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn> {
    let mut differing = Vec::new();
    for identity_handed_in in reader.device_identities() {
        let own_device_numbers_on_disk: BTreeSet<DeviceIdentity> =
            self_verified_system_configuration_slots_of_the_pool(
                reader,
                identity_handed_in,
                parameters_of_the_pool,
            )
            .into_iter()
            .map(|slot| slot.immutable.this_device)
            .collect();
        differing.extend(
            own_device_numbers_on_disk
                .into_iter()
                .filter(|own_device_number_on_disk| {
                    *own_device_number_on_disk != identity_handed_in
                })
                .map(|own_device_number_on_disk| {
                    OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn {
                        identity_handed_in,
                        own_device_number_on_disk,
                    }
                }),
        );
    }
    differing
}

/// 一块盘两个系统配置槽里本池自证过的那几份（整槽校验和过、fsid 与本池相同，`recovery::verified_system_configuration_slots`，
/// 槽距取本池的固定结构槽距）。一份都没有，这块盘就不「可见」（D18（块里携带什么信息） 已定项 11「可写挂载的顺序」：
/// 「可见」= 独占打开成功且系统配置读得通）：可写挂载取号之前的逐盘核（[`devices_without_the_selected_version`] 第一支）、
/// 挂着之后收盘表的入口（[`caller_inputs_agreeing_with_the_disk`]）与会话每次发布之前（`mounted_session`）判的是同一件事，都经这一个读法。
/// `parameters_of_the_pool` 是盘上择到的那份系统配置里的参数（[`parameters_of_the_system_configuration`]），不是调用方交进来的。
/// 读这块盘的两个系统配置槽，不写盘。
fn self_verified_system_configuration_slots_of_the_pool<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    parameters_of_the_pool: &MakeFilesystemParameters,
) -> Vec<crate::system_configuration::SystemConfiguration> {
    verified_system_configuration_slots(
        reader,
        device,
        u64::from(parameters_of_the_pool.geometry.fixed_structure_slot_spacing),
        &parameters_of_the_pool.filesystem_identifier,
    )
}

/// 交进来的盘里不「可见」的那几块（两个系统配置槽一份本池自证过的都没有，[`self_verified_system_configuration_slots_of_the_pool`]），
/// 按交进来的次序（代码三方 m2-closeout-code-r1 Z3-A 乙：挂着之后每个收盘表的入口都逐盘核，含会话发布；用户 2026-09-27 定）。
/// 挂着之后收盘表的入口在 [`caller_inputs_agreeing_with_the_disk`] 里调，会话在每次发布之前调（`mounted_session`）。
/// 读每块盘的两个系统配置槽，不写盘。
///
/// 循环：按交进来的盘逐块（轮数 = 盘数），每块读两槽；跨轮只往交回的清单里追加，没有提前出口。
pub(crate) fn devices_without_a_self_verified_system_configuration<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
) -> Vec<DeviceIdentity> {
    reader
        .device_identities()
        .into_iter()
        .filter(|device| {
            self_verified_system_configuration_slots_of_the_pool(
                reader,
                *device,
                parameters_of_the_pool,
            )
            .is_empty()
        })
        .collect()
}

/// 交进来的盘表里有几个不同的设备身份：同一个身份交两次（两份都标成盘 0）只算一块盘。D2（RAID 条带策略） 已定项 13 的
/// 「可写设备数」数的是盘，不是盘表的项数（实二八交回第 2 问；调度记录 2026-09-24 第三节实现批次排法里实四那一项「数不同的设备身份」）。
/// 只看盘表里的身份，不读盘：池外的身份照样算一块，由取号之前的逐盘核拒（`devices_without_the_selected_version`）。
fn distinct_device_identities_handed_in<Device>(
    devices: &[(DeviceIdentity, Device)],
) -> DeviceCount {
    DeviceCount(
        devices
            .iter()
            .map(|(identity, _)| *identity)
            .collect::<BTreeSet<DeviceIdentity>>()
            .len(),
    )
}

/// 交进来的盘表里重复的设备身份有就拒（[`MountError::DeviceIdentitiesHandedInMoreThanOnce`]）：只看盘表里的身份，不读盘、不写盘。
/// 可写挂载在「可写设备数 ≥ w 的下限」那一判之后、读第一块盘之前调它。
///
/// 循环：按盘表逐项数（轮数 = 盘表项数），跨轮只往每个身份的计数上加一，没有提前出口。
fn refuse_device_identities_handed_in_more_than_once<Device>(
    devices: &[(DeviceIdentity, Device)],
) -> Result<(), CallerInputsDisagreeingWithTheDisk> {
    let mut times_handed_in_by_device: BTreeMap<DeviceIdentity, usize> = BTreeMap::new();
    for (identity, _) in devices {
        *times_handed_in_by_device.entry(*identity).or_insert(0) += 1;
    }
    let repeated: Vec<RepeatedDeviceIdentity> = times_handed_in_by_device
        .into_iter()
        .filter(|(_, times_handed_in)| *times_handed_in > 1)
        .map(|(device, times_handed_in)| RepeatedDeviceIdentity {
            device,
            times_handed_in,
        })
        .collect();
    if repeated.is_empty() {
        Ok(())
    } else {
        Err(CallerInputsDisagreeingWithTheDisk::DeviceIdentitiesHandedInMoreThanOnce { repeated })
    }
}

/// 盘上一份系统配置的系统不可变配置里、mkfs 参数那几项：fsid、根环逐区域设备身份与几何（D22（单元原子性怎么合成） 已定项 26 第一档：
/// mkfs 之后不可改）。写入口按它建。
fn parameters_of_the_system_configuration(
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: system_configuration.immutable.filesystem_identifier,
        region_devices: system_configuration.immutable.region_devices,
        geometry: system_configuration.immutable.sizes,
    }
}

/// 交进来的盘表与盘上择到的那份系统配置不一致的几项（[`DeviceTableDisagreement`]，实审 A1b Q4）：盘数不是 `devs` 的一项，
/// 再按盘表次序、每块盘自证过的系统配置槽（本池的 fsid、整槽校验和过，`recovery::verified_system_configuration_slots`）里
/// 记的本盘设备号不是盘表给它的身份的各一项（一块盘上同一个号只报一次）。读每块盘的两个系统配置槽，不写盘。
/// 调用方先判过盘表里没有重复身份（按身份读盘，重复了读的是哪一块说不清）。
///
/// 循环：按盘表逐块（轮数 = 盘表项数），每块读两槽；跨轮只往交回的清单里追加，没有提前出口。
fn device_table_disagreeing_with<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> Vec<DeviceTableDisagreement> {
    let devices_handed_in = DeviceCount(devices.len());
    let device_count_in_the_system_configuration = DeviceCount(
        usize::try_from(system_configuration.immutable.device_count)
            .expect("系统配置里的设备数是 4 字节，装得进 usize"),
    );
    let mut disagreements = Vec::new();
    if devices_handed_in != device_count_in_the_system_configuration {
        disagreements.push(
            DeviceTableDisagreement::DeviceCountDiffersFromTheSystemConfiguration {
                devices_handed_in,
                device_count_in_the_system_configuration,
            },
        );
    }
    disagreements.extend(
        own_device_numbers_on_disk_differing_from_the_identities_handed_in(
            devices,
            &parameters_of_the_system_configuration(system_configuration),
        )
        .into_iter()
        .map(
            |OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn {
                 identity_handed_in,
                 own_device_number_on_disk,
             }| {
                DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn {
                    identity_handed_in,
                    own_device_number_on_disk,
                }
            },
        ),
    );
    disagreements
}

/// 写入口用的参数：盘上择到的那份系统配置里的 mkfs 参数（[`parameters_of_the_system_configuration`]）。调用方交进来的参数与它逐项比，
/// 交进来的盘表与它的设备数、每块盘的本盘设备号比（[`device_table_disagreeing_with`]），有一项不同就拒
/// （[`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`]）。读每块盘的两个系统配置槽，不写盘。
/// 调用方先判过盘表里没有重复身份。
fn parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table<
    Device: BlockDevice,
>(
    caller_parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> Result<MakeFilesystemParameters, CallerInputsDisagreeingWithTheDisk> {
    let parameters_on_disk = parameters_of_the_system_configuration(system_configuration);
    // 两边都整个解构：参数或几何加了字段、这里没比，编译不过。
    let MakeFilesystemParameters {
        filesystem_identifier: caller_filesystem_identifier,
        region_devices: caller_region_devices,
        geometry:
            SystemImmutableSizes {
                physical_block_size: caller_physical_block_size,
                minimum_input_output_bytes: caller_minimum_input_output_bytes,
                fixed_structure_slot_spacing: caller_fixed_structure_slot_spacing,
                journal_ring_bytes: caller_journal_ring_bytes,
                root_ring_slots_per_region: caller_root_ring_slots_per_region,
            },
    } = caller_parameters;
    let MakeFilesystemParameters {
        filesystem_identifier: disk_filesystem_identifier,
        region_devices: disk_region_devices,
        geometry:
            SystemImmutableSizes {
                physical_block_size: disk_physical_block_size,
                minimum_input_output_bytes: disk_minimum_input_output_bytes,
                fixed_structure_slot_spacing: disk_fixed_structure_slot_spacing,
                journal_ring_bytes: disk_journal_ring_bytes,
                root_ring_slots_per_region: disk_root_ring_slots_per_region,
            },
    } = &parameters_on_disk;
    let disagreeing_fields: Vec<MakeFilesystemParameterField> = [
        (
            MakeFilesystemParameterField::FilesystemIdentifier,
            caller_filesystem_identifier == disk_filesystem_identifier,
        ),
        (
            MakeFilesystemParameterField::RegionDevices,
            caller_region_devices == disk_region_devices,
        ),
        (
            MakeFilesystemParameterField::PhysicalBlockSize,
            caller_physical_block_size == disk_physical_block_size,
        ),
        (
            MakeFilesystemParameterField::MinimumInputOutputBytes,
            caller_minimum_input_output_bytes == disk_minimum_input_output_bytes,
        ),
        (
            MakeFilesystemParameterField::FixedStructureSlotSpacing,
            caller_fixed_structure_slot_spacing == disk_fixed_structure_slot_spacing,
        ),
        (
            MakeFilesystemParameterField::JournalRingBytes,
            caller_journal_ring_bytes == disk_journal_ring_bytes,
        ),
        (
            MakeFilesystemParameterField::RootRingSlotsPerRegion,
            caller_root_ring_slots_per_region == disk_root_ring_slots_per_region,
        ),
    ]
    .into_iter()
    .filter(|(_, is_equal_on_both_sides)| !is_equal_on_both_sides)
    .map(|(field, _)| field)
    .collect();
    let disagreeing_device_table = device_table_disagreeing_with(devices, system_configuration);
    if disagreeing_fields.is_empty() && disagreeing_device_table.is_empty() {
        Ok(parameters_on_disk)
    } else {
        Err(
            CallerInputsDisagreeingWithTheDisk::ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields,
                disagreeing_device_table,
            },
        )
    }
}

/// 调用方交进来的参数与盘表照可写挂载那一套核过之后（[`caller_inputs_agreeing_with_the_disk`]）：盘上择到的那份系统配置，
/// 与照它建的参数——写入口只用这一份，不用调用方的。
struct CallerInputsAgreeingWithTheDisk {
    system_configuration: crate::system_configuration::SystemConfiguration,
    parameters_of_the_pool: MakeFilesystemParameters,
}

/// 挂着之后收调用方参数与盘表的入口（抬 F、准入抬 F、一次准入里再推一串、正常卸载）与管理员回退照可写挂载同一套核（实审 A1b Q2、Q3；
/// 代码审阅第 17、18 条）：盘表里有身份交了不止一次就拒（不读盘）；择系统配置；调用方的参数逐项比，盘表比设备数与每块盘的本盘设备号，
/// 有一项不同就拒；这几项都对上之后，交进来的每块盘要「可见」——两个系统配置槽里至少一份本池自证过的，有一块没有就拒
/// （[`DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`]，与可写挂载取号之前的逐盘核第一支同一判；
/// 代码三方 m2-closeout-code-r1 Z3-A 乙）。择系统配置那一步只要求读得出的那几份彼此对得上，空盘上一份都没有、择得出来，
/// 前三项也报不出它（本盘设备号是逐份自证槽比的，一份都没有就一条都不比），所以另判这一项。
/// 「可见」之后再核落后支：每块盘最新那份自证系统配置不落后于调用方持着的现行那一版（`current`）、或落后而现行那一版的单元都在、
/// 它的根住在这块盘上时那个根环槽里也是它（[`devices_behind_the_current_version`]；D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定，
/// 代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」）。三样里的本盘设备号在前面盘表那一判里已比过（[`device_table_disagreeing_with`]）。
/// 只读盘，不写盘：每个入口在它的第一个写之前调它。
///
/// # Errors
/// 交进来的不对（[`CallerInputsDisagreeingWithTheDisk`] 的三种；不「可见」的盘报在第二种的 `disagreeing_device_table` 里，
/// 落后又缺单元的盘报在第三种里）；系统配置择不出来（别的池的盘换进来，两块盘上各有一份 fsid 不同的自证槽，报在这里）。
#[allow(
    clippy::ptr_arg,
    reason = "choose_system_configuration 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
    caller_parameters: &MakeFilesystemParameters,
    devices: &Vec<(DeviceIdentity, Device)>,
    current: &TransactionOutput,
) -> Result<CallerInputsAgreeingWithTheDisk, CallerInputsCheckRefused> {
    refuse_device_identities_handed_in_more_than_once(devices)
        .map_err(CallerInputsCheckRefused::Disagreeing)?;
    let system_configuration = choose_system_configuration(devices)
        .map_err(CallerInputsCheckRefused::SystemConfigurationNotChosen)?;
    let parameters_of_the_pool =
        parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table(
            caller_parameters,
            devices,
            &system_configuration,
        )
        .map_err(CallerInputsCheckRefused::Disagreeing)?;
    let devices_not_visible: Vec<DeviceTableDisagreement> =
        devices_without_a_self_verified_system_configuration(devices, &parameters_of_the_pool)
            .into_iter()
            .map(|identity_handed_in| {
                DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn {
                    identity_handed_in,
                }
            })
            .collect();
    if !devices_not_visible.is_empty() {
        return Err(CallerInputsCheckRefused::Disagreeing(
            CallerInputsDisagreeingWithTheDisk::ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields: Vec::new(),
                disagreeing_device_table: devices_not_visible,
            },
        ));
    }
    let devices_behind =
        devices_behind_the_current_version(devices, &parameters_of_the_pool, current);
    if !devices_behind.is_empty() {
        return Err(CallerInputsCheckRefused::Disagreeing(
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                devices: devices_behind,
            },
        ));
    }
    Ok(CallerInputsAgreeingWithTheDisk {
        system_configuration,
        parameters_of_the_pool,
    })
}

/// 会话的推环入口（`mounted_session::MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`）
/// 收盘表时走的核：与抬 F 那一串的空发布收盘表同一套（[`caller_inputs_agreeing_with_the_disk`]），交回盘上择到的那份系统配置照它建的参数，
/// 写入口只用这一份。只读盘。
///
/// # Errors
/// 同 [`caller_inputs_agreeing_with_the_disk`]，装成 `MountError`（与抬 F 那一串交回的同一种）。
#[allow(
    clippy::ptr_arg,
    reason = "caller_inputs_agreeing_with_the_disk 走 &Vec：choose_system_configuration 要一个有大小的读者"
)]
pub(crate) fn parameters_of_the_pool_agreeing_with_the_caller_inputs<Device: BlockDevice>(
    caller_parameters: &MakeFilesystemParameters,
    devices: &Vec<(DeviceIdentity, Device)>,
    current: &TransactionOutput,
) -> Result<MakeFilesystemParameters, MountError> {
    Ok(
        caller_inputs_agreeing_with_the_disk(caller_parameters, devices, current)?
            .parameters_of_the_pool,
    )
}

/// 可写挂载准入的第一条合取（D2（RAID 条带策略） 已定项 13「挂载准入」：可写设备数 ≥ w 的下限；D18（块里携带什么信息） 已定项 11
/// 「可写挂载的顺序」先判这一条）：交进来的不同设备身份数（[`distinct_device_identities_handed_in`]）低于 [`STRIPE_WIDTH_LOWER_BOUND`] 就拒。
/// 只比两个数，不读盘、不写盘。
/// 可写挂载在读第一块盘之前调它；之后到取号为止交进来的那份盘表只借给重建与写入口、长短不变，取号写的就是这里数过的这几块盘。
fn admit_the_writable_device_count(devices_handed_in: DeviceCount) -> Result<(), MountError> {
    if devices_handed_in < STRIPE_WIDTH_LOWER_BOUND {
        return Err(
            MountError::WritableDeviceCountBelowTheStripeWidthLowerBound {
                devices_handed_in,
                stripe_width_lower_bound: STRIPE_WIDTH_LOWER_BOUND,
            },
        );
    }
    Ok(())
}

/// 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘 → 取号之前准入不够的，推抬 F 的空发布再判：可写挂载的后半段。
fn establish_instance<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    mut allocator: PoolAllocator,
    start: InstanceStart,
) -> Result<Mounted, MountError> {
    let isolated_slots_per_device: Vec<(DeviceIdentity, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect();
    // 所选那一版的单元清单在开写入口之前取好（树表 0 条那一版要按位置条目读盘）：取号之前逐盘核拿它比。
    let units_of_the_selected_version = start.previous.units(
        devices,
        unit_area_start_of_the_chosen_system_configuration(&start.system_configuration),
    )?;
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    // 暖机推哪几次，取号之前先算出来：准入按这一份算，取号之后的循环也按这一份推。
    let warm_up_publishes_planned =
        warm_up_publish_txgs(parameters, &all_devices, start.first_txg)?;
    // 要取的号装不下（盘上最大的实例代号已是 u32::MAX，代码审阅第 36 条）就在取号之前拒，一个写都没发。
    let instance_to_acquire = next_instance_generation_to_acquire(&pool).map_err(
        |InstanceGenerationPastTheTopOfItsRange { highest_on_disk }| {
            MountError::SequenceNumberPastTheTopOfItsRange(
                SequenceNumberAtTheTopOfItsRange::InstanceGeneration(highest_on_disk),
            )
        },
    )?;
    // 要写的行按要取的号先列出来：实例表那一项准入按它算，取号之后写行也用这一份（号在写之前重算、不等就不写）。
    let rows_written = instance_rows_to_write(&start.previous_row, instance_to_acquire);
    let instance_table_rewrite = instance_table_rewrite_of_the_row_publish(
        start.previous.instance_table_chain(),
        &rows_written,
    );
    // 空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决第四节第 2 条接进可写挂载）：可写挂载的准入要
    // 「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13），预留是式子里的挂载期承诺量——每块盘上连它与别的几项都扣完，
    // 可用(d) ≥ 0（这一刻不另要需求：写行与暖机那一串的空间就在切换预留与 checkpoint 保留池里）。rows0 = 挂载时读到的行数加写行那次
    // 要写的行数（D28（挂载期承诺量） 已定项 3），挂载期间常量，记在分配器上给这次挂载之后的每次发布用（发布路径按它加一）。
    // 不够时（D16（发布语义） 已定项 1「准入」那一行，用户 2026-09-26 定）：写行那次发布照走切换预留，写完行（与暖机）之后推抬 F 的空发布、
    // 再判（`push_floor_raises_after_the_row_publish`）。上一版树表 0 条的不推：抬 F 要带文件的现行版本，这一版上没有可退的带文件版本、
    // 推不出空间（卸载在这一版上同样一个字节都不写），照旧在取号之前拒，盘上逐字节不变。
    // 只供测试的开关（`SpaceAdmission::SkippedByTheTestOnlySwitch`）关掉准入时不判，这次挂载之后的发布也不判（开关装在分配器上）。
    allocator.record_instance_rows_after_this_mounts_row_publish(
        u64::try_from(instance_table_rewrite.rows.len()).expect("实例表的行数装得进 u64"),
    );
    // 写行那次发布之后最新那条根带的就是这张表（池级 checker 按它判有效）：进程内重读挂载时读不出的槽（E2）从写行那次记根起按它判。
    allocator.note_the_instance_table_judging_the_rereads_of_root_ring_slots(
        InstanceTableRecords {
            rows: instance_table_rewrite.rows.clone(),
        },
    );
    allocator.set_space_admission(start.space_admission);
    let admission_before_acquisition = match start.space_admission {
        SpaceAdmission::JudgedByTheFormula => match admit_on_every_device(
            &admission_reading_of_a_writable_mount_with_node_capacities(
                &allocator,
                start.previous.version_of_trees(),
                pool.code_two_tree_node_capacities(),
                parameters.geometry.root_ring_slots_per_region,
            ),
            &no_demand_on_any_device_of(&allocator),
        ) {
            Ok(()) => WritableMountAdmissionBeforeAcquisition::Admitted,
            Err(refusal) => match &start.previous {
                PreviousVersion::WithFile { .. } | PreviousVersion::WithTreesAndNoFile { .. } => {
                    WritableMountAdmissionBeforeAcquisition::ShortUntilTheFloorIsRaisedAfterTheRowPublish(
                        refusal,
                    )
                }
                PreviousVersion::WithoutFile { .. } => {
                    return Err(MountError::SpaceAdmissionRefusedBeforeAcquisition {
                        instance_to_acquire,
                        refusal,
                    });
                }
            },
        },
        SpaceAdmission::SkippedByTheTestOnlySwitch => {
            WritableMountAdmissionBeforeAcquisition::NotJudgedByTheTestOnlySwitch
        }
    };
    // 取号之后要发的那一串（写行一次、暖机那几次）在取号之前整串预演一遍（分配器的拷贝上，走发布路径落盘之前那一段，
    // 与后面真发读同一个分配器——取号不碰它）：释放核验、树的形状、落点取不到，都在任何写之前返回。
    // 落点取不到怎么算进取号之前的准入，条款没定（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的文档注释）。
    let placements_planned = match dry_run_of_the_publishes_after_acquisition(
        &pool,
        &allocator,
        &start,
        &instance_table_rewrite,
        rows_written.len(),
        &warm_up_publishes_planned,
        instance_to_acquire,
    ) {
        Ok(taken) => Some(taken),
        Err(DryRunRefusal {
            publish_index,
            cause: PublishError::PlacementRefused { unit, refusal },
        }) => {
            return Err(
                MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
                    instance_to_acquire,
                    publish_index,
                    warm_up_publishes_planned: warm_up_publishes_planned.len(),
                    unit,
                    refusal,
                },
            )
        }
        Err(DryRunRefusal {
            publish_index: 0,
            cause,
        }) => {
            return Err(MountError::RowPublishAdmissionRefusedBeforeAcquisition {
                instance_to_acquire,
                cause,
            })
        }
        Err(DryRunRefusal {
            publish_index: warm_up_publish_index,
            cause,
        }) => {
            return Err(MountError::WarmUpAdmissionRefusedBeforeAcquisition {
                instance_to_acquire,
                warm_up_publish_index,
                warm_up_publishes_planned: warm_up_publishes_planned.len(),
                cause,
            })
        }
    };
    // 可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13、D18（块里携带什么信息） 已定项 11）：取号是这次挂载的第一个写，
    // 核在它之前、读的是紧接着要写的这批盘；有一块不带所选那一版就不取号，盘上逐字节不变。排在别的准入之后，
    // 那几条先判出来的仍报各自的成员。落后支缺根那一半只核写进了根环的所选那一版：择到的那条根上没施加 journal 记录才是它
    // （施加过的那一版是崩在根槽 FUA 之前的那次发布，哪块盘的根环里都没有它，那一形是正常的崩溃状态）。
    let root_of_the_selected_version = if start.effective_root.instance
        == start.chosen_root.instance
        && start.effective_root.checkpoint_txg == start.chosen_root.checkpoint_txg
    {
        RootOfTheVersionInTheRing::Written {
            instance: start.effective_root.instance,
            checkpoint_txg: start.effective_root.checkpoint_txg,
        }
    } else {
        RootOfTheVersionInTheRing::NotWritten
    };
    let devices_without = devices_without_the_selected_version(
        &*pool.devices,
        &start.system_configuration,
        start.selected_version_journal_position,
        &units_of_the_selected_version,
        root_of_the_selected_version,
    )
    .map_err(|device| {
        refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
            start.rereads.read_stage,
            SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition { device },
        )
    })?;
    if !devices_without.is_empty() {
        return Err(
            MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
                selected_version: RollbackTarget {
                    instance: start.effective_root.instance,
                    checkpoint_txg: start.effective_root.checkpoint_txg,
                },
                selected_version_journal_position: start.selected_version_journal_position,
                devices: devices_without,
            },
        );
    }
    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(
        |failure| match failure {
            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
                expected,
                recomputed,
            } => MountError::InstanceGenerationChangedBeforeAcquisition {
                expected,
                recomputed,
            },
            ExpectedInstanceAcquisitionFailed::InstanceGenerationPastTheTopOfItsRange(
                InstanceGenerationPastTheTopOfItsRange { highest_on_disk },
            ) => MountError::SequenceNumberPastTheTopOfItsRange(
                SequenceNumberAtTheTopOfItsRange::InstanceGeneration(highest_on_disk),
            ),
            // 读阶段的见证读判得出、取号那一刻读见证值时判不出（R2）或这块盘一份自证过的系统配置都没有（两次读之间的瞬时读错，
            // C554 乙-配置续 Q1）：拒可写，与读阶段判据 N-配置 那一拒分开报（C664）。一个字节都没写。
            ExpectedInstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
                first_device_reporting_a_read_error,
            } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
                start.rereads.read_stage,
                SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
                    first_device_reporting_a_read_error,
                },
            ),
            ExpectedInstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
                device,
            } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
                start.rereads.read_stage,
                SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
                    device,
                },
            ),
            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {
                MountError::Acquisition(acquisition)
            }
        },
    )?;
    assert_eq!(
        instance, instance_to_acquire,
        "acquire_expected_instance 写之前重算、与判定时的号不等就不写：交回的就是列行与判准入用的那个号"
    );

    // 取号之后的每一次发布失败都带着这次挂载的写入口交得出的账返回（`PublishSequenceFailed`）：写入口随错一起丢掉。
    let row_publish = match &start.previous {
        PreviousVersion::WithFile { output, .. }
        | PreviousVersion::WithTreesAndNoFile { output, .. } => {
            let published = publish_rows_on_file_version(
                &mut pool,
                &mut allocator,
                output,
                instance_table_rewrite,
                RowPublishIdentity {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            );
            published.map(PoolVersion::of_a_version_of_trees)
        }
        // 树表 0 条的一版：要写的行为空（上一个实例是 0，第一次可写挂载）就走零单元发布，新池新建文件的字节不变
        // （D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」；layout/01-first-txn.md 八「第一次之后的可写挂载（写行）」那一行 2026-09-14 的改名注）；
        // 要写的行不为空就只重写实例表那一个单元——这一版没有记账树，已定项 9 的那五样一样都不写，而 D18（块里携带什么信息） 已定项 11
        // 要求每次可写挂载都写行。
        PreviousVersion::WithoutFile { root, .. } if rows_written.is_empty() => {
            publish_without_units(
                &mut pool,
                root,
                ZeroUnitPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            )
            .map(|published| {
                // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`），
                // 记之前照 E2 重读挂载时读不出的那一槽。
                record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
                    &*pool.devices,
                    pool.parameters,
                    &mut allocator,
                    published.root.checkpoint_txg,
                );
                PoolVersion::WithoutFile(published)
            })
        }
        PreviousVersion::WithoutFile { root, .. } => {
            publish_instance_table_on_version_without_file(
                &mut pool,
                &mut allocator,
                root,
                InstanceTableOnlyPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    // 新实例的第一条记录，反向链恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    instance_table: &instance_table_rewrite,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            )
            .map(PoolVersion::WithoutFile)
        }
    }
    .map_err(|cause| publish_failed_after_acquisition(&pool, &[], cause))?;

    // 暖机（D16（发布语义） 已定项 8 戊）：本实例的根覆盖两块盘之前连推空发布，推哪几次按取号之前算好的那一份计划
    // （`warm_up_publishes_planned`），不在这里另算一遍——两处各算各的，只要有一处多算一次，取号之前那道准入就罩不住实际推的那几次。
    assert_eq!(
        row_publish.root().checkpoint_txg,
        start.first_txg,
        "写行那次发布的 txg 就是算暖机计划用的那个：两条路（带文件的写行、树表 0 条的零单元发布）交给发布路径的 txg 都取 start.first_txg"
    );
    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();
    for planned_txg in &warm_up_publishes_planned {
        assert_eq!(
            checkpoint_txg_of_the_planned_publish_after(current.root().checkpoint_txg),
            *planned_txg,
            "这次空发布要用的 txg 与计划里的那一个相同：`publish_empty_after` 取的是现行那一版的 txg 加一，计划也是逐个加一"
        );
        let next = publish_empty_after(&mut pool, &mut allocator, &current, instance).map_err(
            |cause| {
                let persisted: Vec<&PoolVersion> = std::iter::once(&row_publish)
                    .chain(&warm_up_publishes)
                    .collect();
                publish_failed_after_acquisition(&pool, &persisted, cause)
            },
        )?;
        warm_up_publishes.push(next.clone());
        current = next;
    }
    // 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开
    // （`rebuilt_allocator`，用户 2026-10-04 选甲）：此后丢哪块盘，恢复择到的都不低于本实例的根，它们不引用这一截。
    // 放开的只有那一截：扣住位只有两处会置——重建分配器那一处与抬 F 那一串（`raise_rollback_floor`），这次挂载的抬 F 都排在这一句之后
    // （写行之后的准入 `push_floor_raises_after_the_row_publish`、续做里推的那几串），它们置的扣住位归它们自己放（`raise_rollback_floor`）；
    // 走到这里分配器上的扣住位全是重建那一处置的。暖机没走完（取号之前被拒、写行或暖机落盘途中报错）时挂载整次报错，分配器随错丢掉，
    // 扣住的那一截不交给任何会话。
    allocator.release_reclaim_holds();
    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——
    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（崩溃恢复落到环里最旧的那条根、其余全被抛弃），两边可以不同
    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。
    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)
        .chain(&warm_up_publishes)
        .map(placements_taken_by)
        .collect();
    let some_copy_was_quarantined = std::iter::once(&row_publish)
        .chain(&warm_up_publishes)
        .filter_map(PoolVersion::version_of_trees)
        .any(|version| {
            !version
                .quarantined_after_release_checksum_mismatch
                .is_empty()
        });
    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {
        assert_eq!(
            &placements_taken, planned,
            "取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
             不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串"
        );
    }

    let space_admission = match admission_before_acquisition {
        WritableMountAdmissionBeforeAcquisition::Admitted => {
            MountSpaceAdmission::AdmittedBeforeAcquisition
        }
        WritableMountAdmissionBeforeAcquisition::NotJudgedByTheTestOnlySwitch => {
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch
        }
        WritableMountAdmissionBeforeAcquisition::ShortUntilTheFloorIsRaisedAfterTheRowPublish(
            refusal_before_acquisition,
        ) => {
            let current_file_version = match &mut current {
                PoolVersion::WithFile(current_file_version)
                | PoolVersion::WithTreesAndNoFile(current_file_version) => current_file_version,
                PoolVersion::WithoutFile(_) => panic!(
                    "取号之前不够、要写行之后推的只有上一版有树的那一格（树表 0 条的在取号之前拒了）：\
                     有树的一版上写行与暖机交回的都是有树的一版（`publish_rows_on_file_version`、`publish_empty_after`）"
                ),
            };
            // 这次挂载已落盘的发布（写行那次、暖机每次）各自的写：抬 F 报错时随错交出（实审 A1b Q5）。
            let writes_of_persisted_publishes: Vec<WritesByStructureKind> =
                std::iter::once(&row_publish)
                    .chain(&warm_up_publishes)
                    .map(|version| version.writes().clone())
                    .collect();
            // 写行与暖机那几次的写入口装的节点容量：之后推抬 F、再判的读数按同一档算。
            let node_capacities_of_the_writer = pool.code_two_tree_node_capacities();
            push_floor_raises_after_the_row_publish(
                parameters,
                devices,
                &mut allocator,
                current_file_version,
                start.shadow_ledger,
                node_capacities_of_the_writer,
                refusal_before_acquisition,
                writes_of_persisted_publishes,
            )?
        }
    };

    // 续做之前这次挂载已落盘的发布（写行那次、暖机每次、写行之后推的抬 F 每一次空发布）各自的写，按先后：续做里的一步没发成、
    // 不是空间不够时随错交出（`PublishSequenceFailed::writes_of_persisted_publishes` 的契约，代码轮 U9）。
    let writes_of_the_publishes_before_the_continuation: Vec<WritesByStructureKind> =
        std::iter::once(&row_publish)
            .chain(&warm_up_publishes)
            .map(|version| version.writes().clone())
            .chain(
                space_admission
                    .floor_raises()
                    .iter()
                    .flat_map(|raised| raised.publishes.iter())
                    .map(|publish| publish.writes.clone()),
            )
            .collect();
    let DeletionQueueContinuation {
        steps_published: deletion_queue_continuation,
        floor_raises: floor_raises_pushed_by_the_continuation,
        refusals_that_pushed_the_floor_raises: refusals_that_pushed_the_continuation_floor_raises,
        outcome: deletion_queue_continuation_outcome,
    } = continue_the_deletion_queue_until_it_is_empty(
        parameters,
        devices,
        &mut allocator,
        &mut current,
        instance,
        start.shadow_ledger,
        writes_of_the_publishes_before_the_continuation,
        space_admission.floor_raises(),
    )?;
    // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
    // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
    let instance_table_sweep = sweep_the_instance_table_at_the_end_of_the_mount(
        parameters,
        devices,
        &mut allocator,
        &mut current,
        start.instance_table_sweep,
    );

    Ok(Mounted {
        output: MountOutput {
            instance,
            chosen_root: start.chosen_root,
            effective_root: start.effective_root,
            journal: start.journal,
            rows_written,
            row_publish,
            warm_up_publishes,
            shadow_ledger: start.shadow_ledger,
            shadow_ledger_branch: start.shadow_ledger.branch_name(),
            isolated_slots_per_device,
            abandoned_roots_unreadable: start.abandoned_roots_unreadable,
            space_admission,
            // `parameters` 是可写挂载按盘上择到的系统配置建的那一份（`mount_writable_with_test_only_switches` 核过调用方的参数与盘表），
            // 盘表就是取号写过的这几块盘。
            parameters_and_device_table: ParametersAndDeviceTableOfTheMount {
                parameters_on_disk: parameters.clone(),
                device_identities_in_table_order: all_devices,
            },
            rereads: start.rereads,
            deletion_queue_continuation,
            floor_raises_pushed_by_the_continuation,
            refusals_that_pushed_the_continuation_floor_raises,
            deletion_queue_continuation_outcome,
            instance_table_sweep,
        },
        allocator,
        current,
    })
}

/// 挂载续做交回的东西（[`continue_the_deletion_queue_until_it_is_empty`]）：发成的每一步与为空间推的那几串抬 F，两样按 txg 交错；
/// 走到哪儿停（走到了空，或推满仍放不下、队列留着）。
struct DeletionQueueContinuation {
    steps_published: Vec<TransactionOutput>,
    floor_raises: Vec<RaisedFloor>,
    refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    outcome: DeletionQueueContinuationOutcome,
}

impl DeletionQueueContinuation {
    /// 续做里已落盘的每一次发布（发成的每一步、推的每一串的每一次空发布）各自的写，按 txg 的先后。
    /// 推的那几串先写系统配置那一步（`RaisedFloor::system_configuration_writes_before_the_first_publish`）不是发布，不在这里。
    fn writes_of_the_persisted_publishes_in_txg_order(&self) -> Vec<WritesByStructureKind> {
        let mut persisted: Vec<&TransactionOutput> = self
            .steps_published
            .iter()
            .chain(
                self.floor_raises
                    .iter()
                    .flat_map(|raised| raised.publishes.iter()),
            )
            .collect();
        persisted.sort_by_key(|publish| publish.root.checkpoint_txg);
        persisted
            .into_iter()
            .map(|publish| publish.writes.clone())
            .collect()
    }
}

/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）：写行与暖机（连同之后为空间推的抬 F）之后、交回之前，把现行那一版的 `deleted_inodes`
/// 走到空；与会话每次用户改动先走空共用同一个走空函数（`transaction::walk_the_deletion_queue_until_it_is_empty`），不进取号之前的预演。
/// 批大小与墓碑容器容量取分配器上装的开关（`mount_writable_with_test_only_switches_and_deletion_switches`）。队列空（常态）时一个写都不发；
/// 走完 `current` 按 `PoolVersion::of_a_version_of_trees` 重新归成员。交回发成的每一步（按先后）、为空间推的那几串抬 F 与停在哪儿。
///
/// 走空里的删除类发布（不判式子那一判、落得下那一判照判，D28（挂载期承诺量） 已定项 1）被拒是空间不够的那两种
/// （[`refusal_is_short_of_space`]）时，推一串抬 F 的空发布（[`push_one_floor_raise_within_the_admission_budget`]）、回收之后接着走
/// （用户 2026-10-03 定案「挂载续做被空间拒：同样推抬 F 再试」，与会话那一处 `mounted_session` 的走空同一个做法）：每一次被拒的删除类发布
/// 各算一次准入（用户 2026-10-03 定案 Q-V 预算「甲 每次被拒各算一次」：这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，
/// 这次准入已推的数从 0 数起）。推满仍放不下（一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）时
/// 挂载照样做成可写、队列留着（用户 2026-10-03 定「照 D16 做成可写」，D16（发布语义） 已定项 1「准入」那一行「挂载处推满仍不够时挂载照样做成可写」）：
/// 交回 [`DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`]，`current` 是最后落盘的那一版；之后会话里每一次用户改动先走空、
/// 走不动报空间不够；正常卸载照常做成，靠的是卸载那一串的预演算上新 F 回收出来、当场放开的那一截（[`unmount`]，用户 2026-10-03 定甲 GA）。
/// 迭代上界：每一轮要么走空交回，要么推满仍放不下交回，要么交回错，要么推成一串（推的数加进这次准入，同一次发布的准入在预算 B − 1 次空发布之内停），
/// 要么这一趟有一步落了盘（队首那个文件少了单元或删掉了，队列与单元数都有限）。跨轮携带的是走到的那一版、已推的数与已落盘的。
///
/// # Errors
/// 走空里的一步没发成、不是空间不够：错连同这次挂载已落盘各次发布的写（`writes_of_the_publishes_before_the_continuation` 接上续做里已落盘的
/// 每一步与推的每一次空发布，按 txg 的先后）、这次挂载推成的每一串抬 F（`floor_raises_pushed_by_the_admission` 接上续做里推成的）
/// 先写系统配置那一步（代码轮 V2）与这一趟走空的写入口的失败账（`MountError::Publish`，[`PublishSequenceFailed`] 的契约）。
/// 抬 F 自己报了不是空间不够的错：那个错装进 `MountError::FloorRaiseFailedAfterTheMountsPublishes`（代码轮 V1），
/// 连同同一张已落盘发布的写与这次挂载推成的每一串抬 F（准入推的接上续做推的）。
/// 两种都是已经发成的那几步与推成的那几串留在盘上。
#[allow(
    clippy::too_many_arguments,
    reason = "挂载续做要的：参数、盘、分配器、现行那一版、实例、影子账那一臂、续做之前已落盘的写、准入推成的那几串，各自独立"
)]
fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    writes_of_the_publishes_before_the_continuation: Vec<WritesByStructureKind>,
    floor_raises_pushed_by_the_admission: &[RaisedFloor],
) -> Result<DeletionQueueContinuation, MountError> {
    let mut continuation = DeletionQueueContinuation {
        steps_published: Vec::new(),
        floor_raises: Vec::new(),
        refusals_that_pushed_the_floor_raises: Vec::new(),
        outcome: DeletionQueueContinuationOutcome::WalkedToEmpty,
    };
    let current_version_of_trees = match &*current {
        PoolVersion::WithoutFile(_) => return Ok(continuation),
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
    };
    if current_version_of_trees.deleted_inodes.is_empty() {
        return Ok(continuation);
    }
    let switches = allocator.deletion_switches();
    let mut walked = current_version_of_trees.clone();
    let mut publishes_pushed_in_this_admission = 0;
    loop {
        let (walk, writes_of_failed_publishes) = {
            let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
            pool.set_container_tree_capacities(switches.container_tree_capacities());
            let walk = walk_the_deletion_queue_until_it_is_empty(
                &mut pool,
                allocator,
                &mut walked,
                switches.batch_size,
                instance,
            );
            (walk, pool.writes_of_failed_publishes().to_vec())
        };
        let DeletionQueueWalk {
            steps_published,
            stopped_by,
        } = walk;
        if !steps_published.is_empty() {
            publishes_pushed_in_this_admission = 0;
        }
        continuation.steps_published.extend(steps_published);
        let refusal = match stopped_by {
            Ok(()) => {
                *current = PoolVersion::of_a_version_of_trees(walked);
                return Ok(continuation);
            }
            Err(refusal) => refusal,
        };
        if !refusal_is_short_of_space(&refusal) {
            *current = PoolVersion::of_a_version_of_trees(walked);
            let writes_of_persisted_publishes: Vec<WritesByStructureKind> =
                writes_of_the_publishes_before_the_continuation
                    .iter()
                    .cloned()
                    .chain(continuation.writes_of_the_persisted_publishes_in_txg_order())
                    .collect();
            return Err(MountError::Publish(Box::new(PublishSequenceFailed {
                cause: refusal,
                writes_before_the_first_publish: system_configuration_writes_of_the_floor_raises(
                    floor_raises_pushed_by_the_admission
                        .iter()
                        .chain(&continuation.floor_raises),
                ),
                writes_of_persisted_publishes,
                writes_of_failed_publishes,
            })));
        }
        match push_one_floor_raise_within_the_admission_budget(
            parameters,
            devices,
            allocator,
            &mut walked,
            shadow_ledger,
            publishes_pushed_in_this_admission,
        ) {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
                publishes_pushed_in_this_admission += raised.publishes.len();
                continuation.floor_raises.push(raised);
                continuation
                    .refusals_that_pushed_the_floor_raises
                    .push(refusal);
            }
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
                *current = PoolVersion::of_a_version_of_trees(walked);
                continuation.outcome =
                    DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises {
                        last_refusal: refusal,
                        stop,
                    };
                return Ok(continuation);
            }
            Err(cause) => {
                *current = PoolVersion::of_a_version_of_trees(walked);
                return Err(MountError::FloorRaiseFailedAfterTheMountsPublishes(
                    Box::new(FloorRaiseFailedAfterTheMountsPublishes {
                        cause,
                        writes_of_persisted_publishes: [
                            writes_of_the_publishes_before_the_continuation,
                            continuation.writes_of_the_persisted_publishes_in_txg_order(),
                        ]
                        .concat(),
                        floor_raises: floor_raises_pushed_by_the_admission
                            .iter()
                            .cloned()
                            .chain(continuation.floor_raises)
                            .collect(),
                    }),
                ));
            }
        }
    }
}

/// 几串抬 F 各自先写系统配置那一步的写（`RaisedFloor::system_configuration_writes_before_the_first_publish`）逐种相加：挂载续做那一格交
/// `Publish` 时装进 `PublishSequenceFailed::writes_before_the_first_publish`（代码轮 V2）。`WritesByStructureKind` 每一种只记写调用数与字节数之和、
/// 没有加法，这里按种类把两样各自加起来、重放成同样多次写调用（这一种的字节全记在第一次里，其余几次 0 字节）：交出去的每一种的调用数与字节数
/// 与逐串相加相同；合计另核一次，`WrittenStructureKind::IN_REPORT_ORDER` 漏一种就对不上。
fn system_configuration_writes_of_the_floor_raises<'raised>(
    floor_raises: impl IntoIterator<Item = &'raised RaisedFloor>,
) -> WritesByStructureKind {
    let ledgers: Vec<&WritesByStructureKind> = floor_raises
        .into_iter()
        .map(|raised| &raised.system_configuration_writes_before_the_first_publish)
        .collect();
    let mut added_up = WritesByStructureKind::NOTHING_WRITTEN;
    for kind in WrittenStructureKind::IN_REPORT_ORDER {
        let of_this_kind = ledgers
            .iter()
            .fold(WriteCallsAndBytes::NONE, |sum, ledger| {
                sum.plus(ledger.of(kind))
            });
        // 迭代上界是这一种加起来的写调用数；跨轮只多记这一种的一次调用。
        for call_index in 0..of_this_kind.write_calls {
            let bytes_of_this_call = if call_index == 0 {
                usize::try_from(of_this_kind.written_bytes)
                    .expect("几串抬 F 先写系统配置那一步的字节数之和装得进 usize")
            } else {
                0
            };
            added_up.count_write_call(kind, &vec![0_u8; bytes_of_this_call]);
        }
    }
    assert_eq!(
        added_up.total(),
        ledgers
            .iter()
            .fold(WriteCallsAndBytes::NONE, |sum, ledger| sum.plus(ledger.total())),
        "逐种相加的合计与逐串合计之和相同：IN_REPORT_ORDER 罩着每一种，每一种有字节就至少有一次写调用"
    );
    added_up
}

/// 可写挂载取号之前那一判的结局（`establish_instance` 用，交回之前换成 [`MountSpaceAdmission`]）。
enum WritableMountAdmissionBeforeAcquisition {
    Admitted,
    /// 不够，上一版带文件：写行与暖机照做，之后推抬 F 的空发布再判（`push_floor_raises_after_the_row_publish`）。
    ShortUntilTheFloorIsRaisedAfterTheRowPublish(AdmissionRefusedOnSomeDevices),
    NotJudgedByTheTestOnlySwitch,
}

/// 每块盘的需求都是 0：可写挂载判「实例切换的预留拿得到」这一刻不另要需求。
fn no_demand_on_any_device_of(allocator: &PoolAllocator) -> Vec<DemandOnDevice> {
    allocator
        .devices
        .iter()
        .map(|device_map| DemandOnDevice {
            device: device_map.device,
            bytes: BytesOnOneDevice::ZERO,
        })
        .collect()
}

/// 可写挂载那一处推（D16（发布语义） 已定项 1「准入」那一行，用户 2026-09-26 定）：取号之前不够的挂载，写行与暖机之后按
/// 可写挂载的读数（这次挂载的 rows0，`admission::admission_reading_of_a_writable_mount_with_node_capacities`，节点容量取
/// `node_capacities`：写行与暖机那几次的写入口装的那一档）再判，不够就推一串抬 F 的空发布
/// （[`push_one_floor_raise_within_the_admission_budget`]，一次准入最多 [`PUBLISHES_PER_ADMISSION_AT_MOST`] 次发布，写行那次算一次）、
/// 再判，直到够了或不再推。推满仍不够时怎么收尾条款没定（C565（挂载处推满仍不够怎么收尾没定））：这里取实现员提的那一种——
/// 挂载照样交回做成（[`MountSpaceAdmission::StillShortAfterTheFloorRaises`]），实例已取、行已写，之后的发布照发布路径的准入判，
/// 正常卸载照常做成（它抬 F 到现行那一版、不判上限，预演算上新 F 回收出来、当场放开的那一截，[`unmount`]，用户 2026-10-03 定甲 GA）；交主 agent 定。
///
/// 循环：每一轮先判，够了就交回；不够就推一串，推成了（至少一次空发布、`publishes_pushed` 增加）进下一轮，推满了就交回
/// （抬 F 那一串自己也取不到落点、被空间准入拒，同样是推满了：[`FloorRaiseStop::FloorRaiseRefusedForSpace`]，实审 A1b Q1），
/// 抬 F 自己报别的错就把错原样交回（代码审阅第 23 条：块设备错不是「推满仍不够」，不走 C565 那一格，D2（RAID 条带策略） 已定项 13
/// 那一格的例外只管空间不够）。
/// 预算把推的次数压在 B − 1 以内 ⇒ 至多 7 轮；跨轮携带的是现行那一版（抬 F 就地推进它）与已推的几串。
///
/// # Errors
/// 抬 F 自己报的、不是空间不够的错（[`push_one_floor_raise_within_the_admission_budget`] 的 `Err`）：装进
/// `FloorRaiseFailedAfterTheMountsPublishes`，连同 `writes_of_persisted_publishes`（这次挂载写行与暖机那几次已落盘的写）
/// 与报错之前已推成的那几串（实审 A1b Q5）。
#[allow(
    clippy::too_many_arguments,
    reason = "挂载推抬 F 要的：参数、盘、分配器、现行那一版、影子账那一臂、写入口的节点容量、取号之前那一判、已落盘的写，各自独立"
)]
fn push_floor_raises_after_the_row_publish<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
    node_capacities: CodeTwoTreeNodeCapacities,
    refusal_before_acquisition: AdmissionRefusedOnSomeDevices,
    writes_of_persisted_publishes: Vec<WritesByStructureKind>,
) -> Result<MountSpaceAdmission, MountError> {
    let mut floor_raises: Vec<RaisedFloor> = Vec::new();
    let mut publishes_pushed = 0;
    loop {
        let refusal = match admit_on_every_device(
            &admission_reading_of_a_writable_mount_with_node_capacities(
                allocator,
                Some(&*current),
                node_capacities,
                parameters.geometry.root_ring_slots_per_region,
            ),
            &no_demand_on_any_device_of(allocator),
        ) {
            Ok(()) => {
                return Ok(MountSpaceAdmission::AdmittedAfterTheFloorRaises {
                    refusal_before_acquisition,
                    floor_raises,
                })
            }
            Err(refusal) => refusal,
        };
        match push_one_floor_raise_within_the_admission_budget(
            parameters,
            devices,
            allocator,
            current,
            shadow_ledger,
            publishes_pushed,
        ) {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
                publishes_pushed += raised.publishes.len();
                floor_raises.push(raised);
            }
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
                return Ok(MountSpaceAdmission::StillShortAfterTheFloorRaises {
                    refusal_before_acquisition,
                    floor_raises,
                    last_refusal: refusal,
                    stop,
                })
            }
            Err(cause) => {
                return Err(MountError::FloorRaiseFailedAfterTheMountsPublishes(
                    Box::new(FloorRaiseFailedAfterTheMountsPublishes {
                        cause,
                        writes_of_persisted_publishes,
                        floor_raises,
                    }),
                ))
            }
        }
    }
}

/// 可写挂载：恢复 → 重建上一版与分配器 → 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘。只做过 mkfs 的池（所选根的树表 0 条、
/// 环里没有记录）同样走这条路：取号 1、不写行、零单元的发布推到本实例的根覆盖每块盘，第一个文件版本接在 `current` 后面。
///
/// # Errors
/// 交进来的盘数低于 w 的下限（`WritableDeviceCountBelowTheStripeWidthLowerBound`，在读任何一块盘之前）、
/// 恢复失败、所选根下有文件而环里一条记录都没有、实例表沿链读不出或解不开、上一版树表 0 条而取号之前的空间准入不过
/// （`SpaceAdmissionRefusedBeforeAcquisition`；上一版带文件的不在这里报，写行之后推抬 F 再判，结局在 `MountOutput::space_admission`）、
/// 取号之前的预演报错、取号之后那一串在分配器的拷贝上取不到落点
/// （`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`）、有盘不带所选那一版（空盘，或停在旧状态：
/// `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，那样的池只许只读挂载）、取号失败、发布失败。
/// 盘表里有设备身份交了不止一次（`DeviceIdentitiesHandedInMoreThanOnce`，在读任何一块盘之前）、调用方的参数或盘表（盘数与 `devs`、
/// 每块盘的本盘设备号）与盘上择到的系统配置不一致（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`，在任何写之前）。
/// 取号之前准入不够、写行与暖机之后推抬 F 时，抬 F 自己报的、不是空间不够的错装进 `FloorRaiseFailedAfterTheMountsPublishes` 交回
/// （块设备错这一类不是「推满仍不够」；那时实例已取、行已写，写行、暖机与之前推成的那几串已落盘，它们的写账随错交出，实审 A1b Q5）；
/// 盘上读来的实例代号或 txg 已到顶、下一个号装不下（`SequenceNumberPastTheTopOfItsRange`，取号之前）；
/// 抬 F 那一串自己也取不到落点、被空间准入拒的，是推满仍不够，挂载照样做成（`MountSpaceAdmission::StillShortAfterTheFloorRaises`）。
/// 挂载续做里推满仍放不下同样不报错：挂载照样做成、队列留着（`MountOutput::deletion_queue_continuation_outcome`）；续做里的一步没发成、
/// 不是空间不够才报 `Publish`，续做里为空间推的抬 F 自己报了不是空间不够的错报 `FloorRaiseFailedAfterTheMountsPublishes`
/// （[`continue_the_deletion_queue_until_it_is_empty`]）。
pub fn mount_writable<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Result<Mounted, MountError> {
    mount_writable_with_space_admission(parameters, devices, SpaceAdmission::JudgedByTheFormula)
}

/// 同 [`mount_writable`]，判不判空间准入由调用方给（只供测试的开关 `SpaceAdmission`；产品路径走 `mount_writable`，恒判）。
/// 开关装在交回的分配器上，这次挂载之后的发布照它判不判。
///
/// # Errors
/// 同 [`mount_writable`]；`SkippedByTheTestOnlySwitch` 时不报 `SpaceAdmissionRefusedBeforeAcquisition`。
pub fn mount_writable_with_space_admission<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
) -> Result<Mounted, MountError> {
    mount_writable_with_test_only_switches(parameters, devices, space_admission, ShadowLedger::On)
}

/// 同 [`mount_writable_with_space_admission`]，影子账那一臂也由调用方给（只供测试的开关 [`ShadowLedger`]，`.claude/rules/fs-design.md`
/// 五条硬要求第 2 条：C314（回退可以复用被抛弃的根引用的单元） 那一格的必红靠它强制进入——被抛弃的根由崩溃恢复造出）。
/// 产品路径走 [`mount_writable`]，恒 `On`。
///
/// # Errors
/// 同 [`mount_writable_with_space_admission`]。
pub fn mount_writable_with_test_only_switches<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
) -> Result<Mounted, MountError> {
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        parameters,
        devices,
        space_admission,
        shadow_ledger,
        BeforeTheOneReread::RereadImmediately,
    )
}

/// 这次挂载内有效的读缓存（C554 乙：E158 第 3 次跑登记第 363 行「乙-配置」那一格；第 4 次跑登记 5.2 第 1 条与 5.5 第 1 条主 agent 的认定）：
/// 读阶段（择根、扫 journal、重放）两遍都经它读。只收读成、而且不是全零的落点的字节；读失败的与读回全零的不收，重读那一遍照样打到盘上——
/// 于是重读那一遍真正打到盘上的，是第一遍读失败或读回全零的落点，与重做时新走到的落点。读阶段判完之后的读直接读盘、不经它。
/// 代价（推的，没量）：扫 journal 时环里写过的每个记录槽都进缓存，环写满一圈之后是环长乘盘数（默认环两块盘约 1.5 GiB），活到读阶段判完为止。
struct ReadStageCache<'devices, Reader: PoolReader> {
    devices: &'devices Reader,
    bytes_read_by_placement: std::cell::RefCell<
        BTreeMap<(DeviceIdentity, crate::address::DeviceOffsetInBytes, usize), Vec<u8>>,
    >,
}

impl<'devices, Reader: PoolReader> ReadStageCache<'devices, Reader> {
    fn empty_over(devices: &'devices Reader) -> Self {
        ReadStageCache {
            devices,
            bytes_read_by_placement: std::cell::RefCell::new(BTreeMap::new()),
        }
    }
}

impl<Reader: PoolReader> PoolReader for ReadStageCache<'_, Reader> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.devices.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.devices.device_size_in_bytes(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: crate::address::DeviceOffsetInBytes,
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
        ring_start: crate::address::DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<crate::address::DeviceOffsetInBytes>> {
        self.devices
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

/// 读阶段一遍读出来的：所选根、环里全部自证过的记录、重放之后的所选那一版与判据 N-配置 的读数。
struct ReadStage {
    chosen_root: RootRecord,
    records: BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    journal: JournalScanReport,
    effective_root: RootRecord,
    against_the_witness: SelectedVersionAgainstTheWitness,
    /// 水位之上那几次缺记录、读得出的记录点名的单元又验不过的发布（F1 乙，[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]）。
    publishes_missing_records_whose_named_units_do_not_verify: Vec<RollbackTarget>,
}

/// F1 乙（用户 2026-10-07 定「缺记录就拒可写」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条）：水位之上、没被施加的
/// 每一次发布（同一 (实例代号, checkpoint_txg) 的记录），读得出的那几条的「本次发布内序号」不是 1..N 连着（I-8.9：一次发布的序号依次是 1..N），
/// 或其中没有一条带末条标志，而它们点名的单元又有一个验不过——某个点名单元两份都读不出或校验和对不上（`F1_ANY`，判决
/// `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行与第三节第 1 条；`recovery::every_named_unit_of_the_record_has_a_copy_that_verifies`：
/// 一份坏是单故障，按两份都要验过判会让写记录中途断电、根没落的那次发布一份副本坏时每次可写挂载都拒；施加前验点名单元仍两份都要验过，
/// 不是这一份）——交回这样的发布，按 (实例代号, checkpoint_txg) 升序。
/// 缺的那条记录点名了哪几个单元挂载时没有别的东西记着（C 隔离不到，判决第二节 K2「C 的盲区」那一行），而那次发布可能已落过根、
/// 可能被下一次恢复择到：调用方重读一次仍这样就在取号之前拒可写。正常崩溃在写记录中途（单元在记录之前落盘，D16（发布语义） 已定项 7）
/// 读得出的记录点名的单元验得过，不交回。水位与 C 同一个（`placements_named_by_unapplied_journal_records`）。
fn publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
) -> Vec<RollbackTarget> {
    let mut records_of_each_publish: BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        Vec<&crate::journal::JournalRecord>,
    > = BTreeMap::new();
    for record in records.values().filter(|record| {
        (record.instance, record.checkpoint_txg)
            > (selected_version.instance, selected_version.checkpoint_txg)
    }) {
        records_of_each_publish
            .entry((record.instance, record.checkpoint_txg))
            .or_default()
            .push(record);
    }
    records_of_each_publish
        .into_iter()
        .filter(|(_, records_of_the_publish)| {
            let some_named_unit_does_not_verify = records_of_the_publish.iter().any(|record| {
                !crate::recovery::every_named_unit_of_the_record_has_a_copy_that_verifies(
                    reader, record,
                )
            });
            readable_records_of_the_publish_leave_a_gap_or_miss_the_last(records_of_the_publish)
                && some_named_unit_does_not_verify
        })
        .map(|((instance, checkpoint_txg), _)| RollbackTarget {
            instance,
            checkpoint_txg,
        })
        .collect()
}

/// 一次发布读得出的那几条记录（`records_of_the_publish`：同一 (实例代号, checkpoint_txg)、读阶段扫得出的）不齐：「本次发布内序号」不是 1..N 连着
/// （I-8.9：一次发布的序号依次是 1..N），或其中没有一条带末条标志；一条都读不出也算不齐。F1 乙
/// （[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]）与 C 的账读不出时从中央映射现算的那一支
/// （`C_MAPPING` 收窄，[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）用同一份。只读内存里的记录。
fn readable_records_of_the_publish_leave_a_gap_or_miss_the_last(
    records_of_the_publish: &[&crate::journal::JournalRecord],
) -> bool {
    let mut ordinals: Vec<u32> = records_of_the_publish
        .iter()
        .map(|record| record.ordinal_within_publish.0)
        .collect();
    ordinals.sort_unstable();
    let ordinals_run_from_one_without_a_gap = ordinals
        .iter()
        .zip(1..)
        .all(|(ordinal, expected)| *ordinal == expected);
    let carries_a_last_record = records_of_the_publish.iter().any(|record| {
        record.place_in_publish
            == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
    });
    !(ordinals_run_from_one_without_a_gap && carries_a_last_record)
}

/// 判据 N-配置（E158 第 3 次跑登记第 347 行）的读数：系统配置槽直接读盘（`system_configuration_slots_of_one_device_read_once`，每块盘两槽），
/// 不经读缓存（第 4 次跑登记 5.1 第 1 条：重读那一遍也直接从盘上重读这几槽）；记录取读阶段那一遍扫出来的。
fn newer_publish_witness<Reader: PoolReader + ?Sized>(
    devices: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    selected_version: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
) -> NewerPublishWitness {
    let slot_spacing_in_bytes = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    // 按盘读（S-辨）：每块盘两槽各读一次，自证过的进 c_见证，设备报读错的计数（D22（单元原子性怎么合成） 已定项 8 射程）。
    let slots_of_each_device: Vec<SystemConfigurationSlotsOfOneDevice> = devices
        .device_identities()
        .into_iter()
        .map(|device| {
            system_configuration_slots_of_one_device_read_once(
                devices,
                device,
                slot_spacing_in_bytes,
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .collect();
    let witnessed_journal_counter = slots_of_each_device
        .iter()
        .flat_map(|slots_of_this_device| slots_of_this_device.self_verified.iter())
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    let comparison = if system_configuration_witness_is_undecidable(&slots_of_each_device) {
        WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
    } else if witnessed_journal_counter == 0 {
        WitnessedCounterComparison::NothingWitnessed
    } else if let Some(last_record_of_the_selected_version) = records
        .values()
        .filter(|record| {
            record.instance == selected_version.instance
                && record.checkpoint_txg == selected_version.checkpoint_txg
                && record.place_in_publish
                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
    {
        WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
            selected_version_last_record_counter: last_record_of_the_selected_version.counter,
        }
    } else if let Some(record_at_the_witnessed_counter) = records
        .values()
        .filter(|record| record.counter == witnessed_journal_counter)
        .max_by_key(|record| (record.instance, record.checkpoint_txg))
    {
        WitnessedCounterComparison::AgainstTheRecordAtTheWitnessedCounter {
            record: RollbackTarget {
                instance: record_at_the_witnessed_counter.instance,
                checkpoint_txg: record_at_the_witnessed_counter.checkpoint_txg,
            },
        }
    } else {
        WitnessedCounterComparison::Undecidable
    };
    NewerPublishWitness {
        witnessed_journal_counter,
        comparison,
    }
}

/// 读阶段一遍：择根、扫 journal、重放（经 `cache` 读），再判 N-配置（系统配置槽经 `devices` 直接读）。
///
/// # Errors
/// 一条自证过的根都没有（`RecoveryFailure::NoValidRoot`）；重放报的（`replay_journal` 的 `Errors`）。
fn read_stage<Reader: PoolReader>(
    cache: &ReadStageCache<'_, Reader>,
    devices: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> Result<ReadStage, MountError> {
    let chosen_root =
        choose_root(cache, system_configuration).ok_or(RecoveryFailure::NoValidRoot)?;
    let records = scan_journal(cache, system_configuration);
    let (journal, effective_root) = replay_journal(
        cache,
        &chosen_root,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        true,
    )?;
    let witness = newer_publish_witness(devices, system_configuration, &effective_root, &records);
    let publishes_missing_records_whose_named_units_do_not_verify =
        publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
            cache,
            &records,
            &effective_root,
        );
    Ok(ReadStage {
        chosen_root,
        records,
        journal,
        effective_root,
        against_the_witness: SelectedVersionAgainstTheWitness {
            selected_version: RollbackTarget {
                instance: effective_root.instance,
                checkpoint_txg: effective_root.checkpoint_txg,
            },
            witness,
        },
        publishes_missing_records_whose_named_units_do_not_verify,
    })
}

/// 一版 `version` 覆盖的最后一条记录（建上一版时拿它当上一版的记录）：所选根覆盖的最后一条记录读得出就是它——同实例、同 checkpoint_txg
/// 的记录里带「本次发布末条」标志的那一条（D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定：「那条」按末条标志认；
/// 一次发布切成多条记录时它是那次发布的末条）。读不出（两份都撕了，或那次发布读得出的几条都不带标志）就拿环里最大 jsn 那条顶着——
/// 本实例的第一条反向链恒 0、事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。树表 0 条的根用不到记录
/// （只做过 mkfs 的池环里一条都没有）。
fn record_standing_for_the_version(
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    version: &RootRecord,
) -> Option<crate::journal::JournalRecord> {
    records
        .values()
        .filter(|record| {
            record.instance == version.instance
                && record.checkpoint_txg == version.checkpoint_txg
                && record.place_in_publish
                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
}

/// C554 乙（用户 2026-09-27 定，R = 1）：读阶段读一遍，判据 N-配置 为真（系统配置见证过比所选那一版新的发布）就在同一个读缓存上
/// 重做一遍（重做之前调 `before_the_one_reread`）。哪一遍判假，就交回哪一遍：往下走用它的所选根、记录与所选那一版。
/// 在 `replay_journal` 交回之后、`rebuild_previous_version` 之前判（E158 第 3 次跑登记 5.1 末段），取号之前。
/// F1 乙（用户 2026-10-07 定）同一个做法：水位之上有一次缺记录、读得出的记录点名的单元又验不过的发布
/// （[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]）也在同一个读缓存上重做读阶段一遍，仍有就拒。
///
/// # Errors
/// 重读那一遍仍判真 ⇒ `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`；
/// 重读那一遍判假而仍有那样一次发布 ⇒ `NewerStateStillUnreadableAfterOneReread(PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)`；
/// 任一遍 [`read_stage`] 报的错原样交回。
fn read_stage_settled_at_most_on_the_one_reread<Reader: PoolReader>(
    devices: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<(ReadStage, ReadStageSettled), MountError> {
    let cache = ReadStageCache::empty_over(devices);
    let first = read_stage(&cache, devices, system_configuration)?;
    let first_read = first.against_the_witness;
    if !first_read.witnesses_a_publish_newer_than_the_selected_version()
        && first
            .publishes_missing_records_whose_named_units_do_not_verify
            .is_empty()
    {
        return Ok((first, ReadStageSettled::OnTheFirstRead { first_read }));
    }
    before_the_one_reread.before_rereading();
    let reread = read_stage(&cache, devices, system_configuration)?;
    let reread_against_the_witness = reread.against_the_witness;
    if reread_against_the_witness.witnesses_a_publish_newer_than_the_selected_version() {
        return Err(MountError::NewerStateStillUnreadableAfterOneReread(
            Box::new(
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                    first_read,
                    reread: reread_against_the_witness,
                },
            ),
        ));
    }
    if let Some(publish) = reread
        .publishes_missing_records_whose_named_units_do_not_verify
        .first()
    {
        return Err(MountError::NewerStateStillUnreadableAfterOneReread(
            Box::new(
                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
                    publish: *publish,
                },
            ),
        ));
    }
    Ok((
        reread,
        ReadStageSettled::OnTheOneReread {
            first_read,
            reread: reread_against_the_witness,
        },
    ))
}

/// F1 乙按 F2 退到的那一版再判一次（`F1_AFTER_F2`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H2 那一行与
/// 第三节第 1 条）：读阶段判 F1 时的水位是退之前的所选那一版——所选根读得出、走读失败、退回的那一形里，被退掉的那次发布不在那时的水位之上，
/// 一条都不判；退到 `version_fallen_back_to` 之后它在水位之上了。按退到的那一版当水位判
/// （[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]，记录取读阶段扫出的那一份、点名单元直接读盘），
/// 有那样一次发布就重读一次（重读之前调 `before_the_one_reread`，点名单元再直接读一遍盘），仍有就拒，与读阶段那一判同一个拒因。只读盘、不写。
///
/// # Errors
/// 重读一次仍有 ⇒ `NewerStateStillUnreadableAfterOneReread(PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)`（按 (实例代号, checkpoint_txg)
/// 最小的那一次），在取号之前、盘上逐字节不变。
fn refuse_publishes_above_the_version_fallen_back_to_missing_records_whose_named_units_do_not_verify<
    Reader: PoolReader + ?Sized,
>(
    devices: &Reader,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    version_fallen_back_to: &RootRecord,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<(), MountError> {
    if publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
        devices,
        records,
        version_fallen_back_to,
    )
    .is_empty()
    {
        return Ok(());
    }
    before_the_one_reread.before_rereading();
    match publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
        devices,
        records,
        version_fallen_back_to,
    )
    .first()
    {
        Some(publish) => Err(MountError::NewerStateStillUnreadableAfterOneReread(
            Box::new(
                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
                    publish: *publish,
                },
            ),
        )),
        None => Ok(()),
    }
}

/// 同 [`mount_writable_with_test_only_switches`]，「重读一次」之前做什么也由调用方给（[`BeforeTheOneReread`]：只供测试的钩子，
/// 用例在里面撤掉暂时的读故障；产品路径走 [`mount_writable`]，立即重读）。
///
/// # Errors
/// 同 [`mount_writable_with_test_only_switches`]；读到的样子里有更新的东西读不出、重读一次仍读不出 ⇒
/// `NewerStateStillUnreadableAfterOneReread`（取号之前，盘上逐字节不变）。
pub fn mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
    before_the_one_reread: BeforeTheOneReread<'_>,
) -> Result<Mounted, MountError> {
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        space_admission,
        shadow_ledger,
        DeletionSwitches::PRODUCT,
        before_the_one_reread,
        InstanceTableSweepConfiguration::PRODUCT,
    )
}

/// 同 [`mount_writable_with_test_only_switches`]，删除那几次发布的两个只供测试的开关（每批几个数据单元、一片墓碑容器装满几条，
/// `unlink::DeletionSwitches`，`.claude/rules/fs-design.md` 五条硬要求第 2 条）也由调用方给：装在交回的分配器上，挂载续做与这次挂载之后的
/// 会话照它（与空间准入开关同一个做法）。产品路径走 [`mount_writable`]，恒 `DeletionSwitches::PRODUCT`。
///
/// # Errors
/// 同 [`mount_writable_with_test_only_switches`]。
pub fn mount_writable_with_test_only_switches_and_deletion_switches<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
    deletion_switches: DeletionSwitches,
) -> Result<Mounted, MountError> {
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        space_admission,
        shadow_ledger,
        deletion_switches,
        BeforeTheOneReread::RereadImmediately,
        InstanceTableSweepConfiguration::PRODUCT,
    )
}

/// 同 [`mount_writable_with_test_only_switches`]，交回之前那一判实例表清扫的开关与阈值也由调用方给
/// （D18（块里携带什么信息） 已定项 11「回收」一条：开关与阈值是配置项，阈值默认值没定，C690；
/// [`crate::instance_table_sweep::InstanceTableSweepConfiguration`]）。产品路径走 [`mount_writable`]，取
/// `InstanceTableSweepConfiguration::PRODUCT`。
///
/// # Errors
/// 同 [`mount_writable_with_test_only_switches`]：清扫自己不报错，它走到哪儿记在 `MountOutput::instance_table_sweep`。
pub fn mount_writable_with_instance_table_sweep_configuration<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
    instance_table_sweep: InstanceTableSweepConfiguration,
) -> Result<Mounted, MountError> {
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        space_admission,
        shadow_ledger,
        DeletionSwitches::PRODUCT,
        BeforeTheOneReread::RereadImmediately,
        instance_table_sweep,
    )
}

/// 可写挂载的全部只供测试的开关一处收：空间准入、影子账、删除开关、「重读一次」之前的钩子、实例表清扫的配置。
fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
    deletion_switches: DeletionSwitches,
    mut before_the_one_reread: BeforeTheOneReread<'_>,
    instance_table_sweep: InstanceTableSweepConfiguration,
) -> Result<Mounted, MountError> {
    admit_the_writable_device_count(distinct_device_identities_handed_in(devices))?;
    refuse_device_identities_handed_in_more_than_once(devices)?;
    let system_configuration = choose_system_configuration(&*devices)?;
    // 写入口按盘上择到的这一份建，不按调用方的（代码审阅第 17 条）；调用方的参数、盘表的盘数与每块盘的本盘设备号与它不一致
    // 就在任何写之前拒（实审 A1b Q4）。
    let parameters_of_the_pool =
        parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table(
            parameters,
            devices,
            &system_configuration,
        )?;
    // 单元区起点按择到的系统配置的环长现算（与偏移 417 那 8 字节相等，择的时候判过）：建空闲图、判分配记录与被抛弃根的落点都按它。
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
    // 每块盘都到得了单元区起点（代码审阅第 35 条）：重建分配器按盘的字节数建空闲图（`DeviceFreeMap::with_unit_area_start`），盘比单元区起点
    // 还短时那里的减法下溢；在读根环之前拒，盘上逐字节不变。
    every_device_reaches_the_unit_area_start(&*devices, unit_area_start)?;
    // 读阶段：择根、扫 journal、重放；系统配置见证过比所选那一版新的发布就重读一遍，仍判真就在这里拒（C554 乙）。
    let (
        ReadStage {
            chosen_root,
            records,
            journal,
            effective_root,
            against_the_witness: _,
            publishes_missing_records_whose_named_units_do_not_verify: _,
        },
        read_stage_settled,
    ) = read_stage_settled_at_most_on_the_one_reread(
        &*devices,
        &system_configuration,
        &mut before_the_one_reread,
    )?;
    let own_record = record_standing_for_the_version(&records, &effective_root);
    let selected_version_journal_position =
        journal_position_of_the_selected_version(&effective_root, own_record.as_ref());
    // 所选那一版建不出来（走读失败）、系统配置没见证它：退到 F2 交回的那一版建上一版（F2 乙，用户 2026-10-07 定「三处都做」，与恢复、只读挂载
    // 同一份判法 `recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`）。退过去不重放：所选根与所选那一版都换成它，
    // 扫环报告里施加的那一截清零（新实例写的上一个实例那一行 W 取 0，与落在一条根上、一条记录都没施加时相同）。
    // 退过去的那一版也建不出来，照报所选那一版的失败。之后的一切（F1 乙再判一次、取号之前逐盘核、写行、影子账与 C）都按退到的那一版。
    // 退到哪一版照 `F2_NEWEST`：[见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（判法在那个函数里）。
    let (chosen_root, effective_root, journal, selected_version_journal_position, previous) =
        match rebuild_previous_version(devices, unit_area_start, &effective_root, own_record) {
            Ok(previous) => (
                chosen_root,
                effective_root,
                journal,
                selected_version_journal_position,
                previous,
            ),
            Err(failure_of_the_selected_version) => {
                let Some(version_fallen_back_to) =
                    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                        &*devices,
                        &system_configuration,
                        &chosen_root,
                        &effective_root,
                        &records,
                    )
                else {
                    return Err(failure_of_the_selected_version);
                };
                let record_standing_for_the_version_fallen_back_to =
                    record_standing_for_the_version(&records, &version_fallen_back_to);
                let journal_position_of_the_version_fallen_back_to =
                    journal_position_of_the_selected_version(
                        &version_fallen_back_to,
                        record_standing_for_the_version_fallen_back_to.as_ref(),
                    );
                match rebuild_previous_version(
                    devices,
                    unit_area_start,
                    &version_fallen_back_to,
                    record_standing_for_the_version_fallen_back_to,
                ) {
                    Ok(previous) => {
                        // F1 乙按退到的那一版的水位再判一次（`F1_AFTER_F2`）：读阶段判 F1 时水位是退之前的所选那一版。
                        refuse_publishes_above_the_version_fallen_back_to_missing_records_whose_named_units_do_not_verify(
                            &*devices,
                            &records,
                            &version_fallen_back_to,
                            &mut before_the_one_reread,
                        )?;
                        (
                            version_fallen_back_to,
                            version_fallen_back_to,
                            JournalScanReport {
                                valid_records: journal.valid_records,
                                above_water: journal.above_water,
                                prefix_applied: 0,
                                verification_passed: journal.verification_passed,
                                verification_failed: journal.verification_failed,
                                maximum_applied_transaction: 0,
                            },
                            journal_position_of_the_version_fallen_back_to,
                            previous,
                        )
                    }
                    Err(_failure_of_the_version_fallen_back_to) => {
                        return Err(failure_of_the_selected_version)
                    }
                }
            }
        };
    // 环里没有记录时从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条：计数器全池接着走）。
    let next_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0)
        + 1;
    let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records)?;
    let tree_identifier_watermark_of_the_ring = tree_identifier_watermark_of_the_ring(
        devices,
        &system_configuration,
        &records,
        &effective_root,
    );
    let rebuilt = rebuilt_allocator(
        devices,
        &system_configuration,
        &previous,
        first_txg,
        &records,
        &effective_root,
        shadow_ledger,
        &mut before_the_one_reread,
    );
    let RebuiltAllocator {
        mut allocator,
        effective_floor,
        abandoned_roots_unreadable,
        instance_table_of_the_newest_root,
        root_ring_of_the_rebuilt_allocator,
    } = rebuilt?;
    allocator.set_deletion_switches(deletion_switches);
    let previous_row = PreviousInstanceRow {
        instance: effective_root.instance,
        selected_root_txg: effective_root.checkpoint_txg,
        applied_transaction_high_water: journal.maximum_applied_transaction,
    };
    establish_instance(
        &parameters_of_the_pool,
        devices,
        allocator,
        InstanceStart {
            chosen_root,
            effective_root,
            journal,
            previous,
            previous_row,
            first_txg,
            next_counter,
            tree_identifier_watermark_of_the_ring,
            effective_floor,
            abandoned_roots_unreadable,
            shadow_ledger,
            system_configuration,
            space_admission,
            selected_version_journal_position,
            rereads: RereadsOfThisMount {
                read_stage: read_stage_settled,
                instance_table_of_the_newest_root,
                root_ring_of_the_rebuilt_allocator,
            },
            instance_table_sweep,
        },
    )
}

/// 管理员回退没做成（D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格与候选集）。成员按管理员要做的决定分：
/// 换一条目标（不在候选集里）、先修盘或放弃这次回退（两版的账读不出、单元读不出、账对不上）、先原样重发冻结着的那次发布、
/// 回退那次发布本身没发成。除了 `Publish` 落盘途中失败那一格，都在任何写之前返回，盘上逐字节不变、分配器与调用方的现行版本不动。
#[derive(Debug)]
pub enum RollbackError {
    /// 分配器上冻结着一次没重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：重发成功才建下一次发布）：
    /// 先原样重发它（`transaction::resend_the_frozen_publish`）。在任何读写之前拒绝。
    PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg,
        instance: InstanceGeneration,
    },
    /// 系统配置择不出来（与挂载那一步同一个错）。
    Recovery(RecoveryFailure),
    /// 调用方交进来的参数或盘表不对（盘表里有身份交了不止一次；参数、盘数或某块盘的本盘设备号与盘上择到的系统配置不一致），
    /// 照可写挂载同一套核（[`caller_inputs_agreeing_with_the_disk`]，实审 A1b 留下的这一处）：回退那次发布的写入口按盘上那份参数建、
    /// 系统配置轮换按盘表逐项写，拿错的参数或盘表回退会改写盘上不可变段、让多交的那块盘收到本池的写
    /// （D22（单元原子性怎么合成） 已定项 26 第一档）。有盘不「可见」、或落后于现行那一版又缺它的单元或它的根（D18（块里携带什么信息） 已定项 11
    /// 「挂着之后收盘表的每个入口……核三样」）也报在这里。在任何写之前（重复身份在任何读之前）拒，盘上逐字节不变。
    CallerInputsDisagreeWithTheDisk(CallerInputsDisagreeingWithTheDisk),
    /// 现行那一版的 checkpoint_txg 已是 `u64::MAX`，回退那次发布的 txg（它加一）装不下（代码审阅第 36 条）：那个 txg 是盘上读来的，
    /// 坏镜像、外来镜像才走得到。在任何写之前、动分配器之前拒。
    NextCheckpointTxgPastTheTopOfItsRange { current: CheckpointTxg },
    /// 现行那一版的根指着的实例表沿链读不出或解不开：候选集「按现行那一版的实例表判仍然有效」判不了。
    CurrentInstanceTableMalformed,
    /// 目标不在回退候选集里，`exclusion` 说是哪一条。
    TargetNotACandidate {
        target: RollbackTarget,
        exclusion: RollbackCandidateExclusion,
    },
    /// 回退目标那一版从盘上重建不出来或自相矛盾：它的树表、inode 树、extent 树、分配记录树（「R_old 那一版的账有一个节点读不出」）、
    /// 记账树、中央映射树有一个节点读不出或解不开，它引用的用户可见单元在它自己的映射里查不到、映射条目的位置项不指池里两块不同的盘，
    /// 或它自己的账里没有它引用的单元的已分配记录。`failure` 说是哪一样。
    TargetVersionUnreadable {
        target: RollbackTarget,
        failure: RecoveryFailure,
    },
    /// 现行那一版的账（它的根指着的分配记录树）从盘上读，有一个节点读不出或解不开（「cur 那一版的账有一个节点读不出」）。
    CurrentAccountUnreadable { failure: RecoveryFailure },
    /// 回退目标那一版的十二棵树的号与现行那一版的不同：新根要取目标的 inode 树与 extent 树、别的树按现行那一版重算，两边号不同时
    /// 新根的树表怎么排条款没写（D23（journal 的角色与格式） 已定项 14 只写了取哪几棵树的根指针），第一版不支持。
    /// 合法历史里走不到：按现行那一版的实例表判仍然有效的根都在同一条时间线上，那条线上的第一个文件版本只发一次号。
    TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
        target: RollbackTarget,
    },
    /// 回退目标引用的一个用户可见单元，在现行那一版的账里这块盘上没有它的记录（没有起点是这个槽、跨度相同的那一条）。
    UserVisibleUnitWithoutItsRecordInTheCurrentAccount {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 回退目标引用的一个用户可见单元，在现行那一版的账里这块盘上仍分配、而分配代与回退目标那一版账里的不同：那一槽被复用过。
    UserVisibleUnitStillAllocatedUnderAnotherGeneration {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
        current_account_generation: CheckpointTxg,
        target_account_generation: CheckpointTxg,
    },
    /// 复活集里一个单元（现行账记成已释放、这一次要改回已分配）在这块盘上的那一份读不出或整单元 CRC-32C 与映射条目里的对不上。
    ResurrectedUnitCopyUnreadableOrMismatched {
        unit: TransactionUnit,
        device: DeviceIdentity,
        slot: SlotNumber,
    },
    /// 释放现行那一版不再被引用的用户可见单元时，释放判定路径报错（映射里查不到、条目切不动、两条位置条目不同槽、这块盘的记录
    /// 不在册 / 已释放 / 跨度不对、位置项指池外的盘或两条指同一块盘）：`cause` 原样是发布路径的那一种。
    CurrentVersionUnitNotReleasable { cause: PublishError },
    /// 判候选集时读根环或算 F_生效 重读一次仍读坏（[`rollback_candidate`]；D16（发布语义） 已定项 1「根槽这一次读坏」那一行的读法，
    /// 实审 A3b 报告 Q7）：只有 [`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`] 与
    /// [`StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor`] 两个成员走得到。目标在不在候选集里判不了，
    /// 不按有根或没根猜。在任何写之前、动分配器之前拒，盘上逐字节不变。
    CandidateJudgementStillUnreadableAfterOneReread(Box<StillUnreadableAfterOneReread>),
    /// 回退那次发布没发成，连同这次回退开的写入口交得出的写账（[`PublishSequenceFailed`]：之前没有别的写、已落盘的发布一次都没有）。
    /// 落盘之前被拒（`cause` 是落点、释放判定这一类）：分配器换回回退之前那一份，盘上逐字节不变。落盘途中失败（`cause` 是块设备错）：
    /// 这次发布冻结在分配器上等原样重发（`transaction::resend_the_frozen_publish`），重发成功就是回退做成。
    Publish(Box<PublishSequenceFailed>),
}

/// 复活的一份：哪个单元、哪块盘、落点、写回的分配代（回退目标那一版账里的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResurrectedCopy {
    pub unit: TransactionUnit,
    pub device: DeviceIdentity,
    pub placement: Placement,
    pub allocation_generation: CheckpointTxg,
}

/// 管理员回退做成之后交回的东西。回退那次发布之后的现行那一版已经换进调用方的 `current`（它的 `released` 是那次发布换下的固定点单元）。
#[derive(Debug)]
pub struct RolledBack {
    pub target: RollbackTarget,
    /// 释放集：现行那一版引用、新根不引用的用户可见单元，按现行那一版的角色次序（两盘同槽，释放代 = 新根的 txg）。
    pub released_user_visible_units: Vec<(TransactionUnit, Placement)>,
    /// 释放集里释放之前读盘核出对不上（读不出也算）、每块盘上那一份的记录留在已分配的那几份（D19（块指针的结构与宽度预算） 已定项 5
    /// 硬规则 1，与发布路径同一段读法）。
    pub quarantined_user_visible_copies: Vec<CopyQuarantinedAfterReleaseChecksumMismatch>,
    /// 复活集，逐盘。
    pub resurrected_user_visible_copies: Vec<ResurrectedCopy>,
}

/// 一个角色是不是用户可见单元（D23（journal 的角色与格式） 已定项 14 的记号：数据单元与 extent 树、inode 树的单元）。
/// 新根的这几样取回退目标那一版的，别的（固定点单元与实例表）按回退那次发布现算或照抄现行那一版的。
#[must_use]
pub const fn is_a_user_visible_unit(identity: TransactionUnit) -> bool {
    match identity {
        TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot => true,
        TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AllocationTree
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
        // `deleted_inodes` 根不在 D23 那个记号里（新根照样取 R_old 的，见 [`is_a_unit_taken_from_the_rollback_target`]）；
        // 墓碑树、谱系树与退役树随现行那一版（退役树：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）。
        TransactionUnit::DeletedInodesRoot
        | TransactionUnit::TombstoneLeafContainer(_)
        | TransactionUnit::TombstoneRoot
        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
        | TransactionUnit::LineageLeafContainer(_)
        | TransactionUnit::LineageRoot
        | TransactionUnit::LineageNodeBelowTheRoot(_)
        | TransactionUnit::RetirementLeafContainer(_)
        | TransactionUnit::RetirementRoot
        | TransactionUnit::RetirementNodeBelowTheRoot(_) => false,
    }
}

/// 回退那次发布的新根取 R_old 那一版的单元：用户可见单元（[`is_a_user_visible_unit`]）与 `deleted_inodes` 根（D23（journal 的角色与格式）
/// 已定项 14：新根的 inode 树、extent 树与 `deleted_inodes` 的根指针取 R_old 的；墓碑树、谱系树、logged_ops 随现行那一版）。
/// 复活、释放、逐盘验与映射 key 都按这一组走：`deleted_inodes` 根指针取 R_old 的而它的单元不复活，新根就指着一个记成已释放的槽。
/// 回退到带 nlink = 0 记录的一版，队列跟着回来，由下一次挂载续做（或会话的下一次用户改动）走空（D8（核心索引结构） 已定项 16 第 6 条）。
#[must_use]
pub const fn is_a_unit_taken_from_the_rollback_target(identity: TransactionUnit) -> bool {
    match identity {
        TransactionUnit::DeletedInodesRoot => true,
        TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot
        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AllocationTree
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_)
        | TransactionUnit::TombstoneLeafContainer(_)
        | TransactionUnit::TombstoneRoot
        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
        | TransactionUnit::LineageLeafContainer(_)
        | TransactionUnit::LineageRoot
        | TransactionUnit::LineageNodeBelowTheRoot(_)
        | TransactionUnit::RetirementLeafContainer(_)
        | TransactionUnit::RetirementRoot
        | TransactionUnit::RetirementNodeBelowTheRoot(_) => is_a_user_visible_unit(identity),
    }
}

/// 一版里每个用户可见单元与它在这一版中央映射里的那两条位置项（映射是落点的权威，D19（块指针的结构与宽度预算） 已定项 5 第 1 条），
/// 按这一版 `mapped_units` 的次序。
///
/// # Errors
/// 查不到 ⇒ `ReleaseNotInMapping`；映射条目窄于字段表 ⇒ `MappingEntryNarrowerThanItsFieldTable`（与释放判定路径同两种）。
fn user_visible_units_through_the_mapping(
    version: &TransactionOutput,
) -> Result<Vec<(TransactionUnit, [LocationEntry; 2])>, PublishError> {
    version
        .mapped_units
        .iter()
        .filter(|(identity, _)| is_a_unit_taken_from_the_rollback_target(*identity))
        .map(
            |(identity, key)| match central_mapping_locations_in_the_version(version, key) {
                MappingLookup::Found(locations) => Ok((*identity, locations)),
                MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
                    Err(PublishError::ReleaseNotInMapping { unit: *identity })
                }
                MappingLookup::EntryNarrowerThanItsFieldTable { entry_bytes } => {
                    Err(PublishError::MappingEntryNarrowerThanItsFieldTable {
                        unit: *identity,
                        entry_bytes,
                        field_table_bytes: usize::try_from(singlefs_format::MAPPING_ENTRY_BYTES)
                            .expect("55"),
                    })
                }
            },
        )
        .collect()
}

/// 回退候选集那一判（D23（journal 的角色与格式） 已定项 14「候选集」）：根环里有它 ∧ txg ≥ F_生效 ∧ 按现行那一版的实例表判仍然有效
/// ∧ 带文件（D16（发布语义） 已定项 1「回退候选集」；原来另有「txg 落在一圈窗口里」一格，用户 2026-10-04 定 RGall 退回整个环）。
/// 按 [`RollbackCandidateExclusion`] 成员的次序判。交回根环里读出来的那条根。
/// 「根环里有它」与 F_生效 两处读根环都照抬 F 同一个读法（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，实审 A3b 报告 Q7）：
/// `ring_slots_known_to_hold_a_root`（这个进程分配器上那张根环表，[`ring_slots_known_to_hold_a_root_by`]）里没有的槽读坏当没有根；
/// 有的重读一次，仍坏就拒——不按没有根往下走（那样目标可以被判成不在环里，F_生效 可以算低、低于真 F 的目标被当成候选）。
/// F_生效 那一半最新那条根的实例表读不出同样重读一次，仍读不出就拒。
///
/// # Errors
/// `TargetNotACandidate`；目标那一版的树表读不出 ⇒ `TargetVersionUnreadable`；知道住着根的槽或最新那条根的实例表重读仍读坏 ⇒
/// `CandidateJudgementStillUnreadableAfterOneReread`（在任何写之前、动分配器之前）。
fn rollback_candidate<Reader: PoolReader>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_table: &InstanceTableRecords,
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
    target: RollbackTarget,
) -> Result<RootRecord, RollbackError> {
    let not_a_candidate = |exclusion| RollbackError::TargetNotACandidate { target, exclusion };
    let still_unreadable = |still_unreadable: StillUnreadableAfterOneReread| {
        RollbackError::CandidateJudgementStillUnreadableAfterOneReread(Box::new(still_unreadable))
    };
    let target_root = readable_roots_rereading_ring_slots_known_to_hold_a_root_once(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        ring_slots_known_to_hold_a_root,
    )
    .map_err(|reading| still_unreadable(reading.into()))?
    .into_iter()
    .find(|root| root.instance == target.instance && root.checkpoint_txg == target.checkpoint_txg)
    .ok_or_else(|| not_a_candidate(RollbackCandidateExclusion::NotInRing))?;
    let effective_floor =
        effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once(
            reader,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
            ring_slots_known_to_hold_a_root,
        )
        .map_err(|reading| still_unreadable(reading.into()))?;
    if target.checkpoint_txg < effective_floor {
        return Err(not_a_candidate(
            RollbackCandidateExclusion::BelowEffectiveFloor,
        ));
    }
    if abandoned_by_table(&target_root, current_table) {
        return Err(not_a_candidate(
            RollbackCandidateExclusion::OnAbandonedTimeline,
        ));
    }
    match tree_table_has_no_entries(reader, &target_root) {
        Ok(true) => Err(not_a_candidate(
            RollbackCandidateExclusion::VersionWithoutFile,
        )),
        Ok(false) => Ok(target_root),
        Err(failure) => Err(RollbackError::TargetVersionUnreadable { target, failure }),
    }
}

/// 回退目标引用的用户可见单元，逐个逐盘对现行那一版的账（D23（journal 的角色与格式） 已定项 14「在任何写之前拒」前两样与「复活」）：
/// 每块盘上起点是它的槽、跨度相同的那条记录——没有就拒；仍分配而分配代与回退目标那一版账里的不同就拒；已释放的进复活集
/// （分配代写回目标那一版账里的）；仍分配、分配代相同的是新旧两版共用的单元，不动。交回复活集，与复活集里有份的单元（逐盘验要读它们每一份）。
///
/// 循环：外层按目标的用户可见单元（轮数 = 单元数），内层按它的两条位置项；跨轮只往两张清单里追加；提前出口都是拒。
///
/// # Errors
/// `UserVisibleUnitWithoutItsRecordInTheCurrentAccount`、`UserVisibleUnitStillAllocatedUnderAnotherGeneration`；
/// 目标那一版自己的账里没有它引用的单元的已分配记录、映射条目的位置项不指池里两块不同的盘 ⇒ `TargetVersionUnreadable`。
fn resurrection_against_the_current_account(
    target: RollbackTarget,
    target_version: &TransactionOutput,
    target_units: &[(TransactionUnit, [LocationEntry; 2])],
    current_account: &PoolAllocator,
) -> Result<Resurrection, RollbackError> {
    let pool_devices: Vec<DeviceIdentity> = current_account
        .devices
        .iter()
        .map(|device_map| device_map.device)
        .collect();
    let malformed_target = |what: &'static str| RollbackError::TargetVersionUnreadable {
        target,
        failure: RecoveryFailure::UnitMalformed { what },
    };
    let mut resurrected = Vec::new();
    let mut units_to_check_on_every_device = Vec::new();
    for (unit, locations) in target_units {
        refuse_locations_that_do_not_name_two_pool_devices(*unit, locations, &pool_devices)
            .map_err(|_refusal| {
                malformed_target("回退目标那一版的映射条目，位置项不指池里两块不同的盘")
            })?;
        let mut is_resurrected_on_some_device = false;
        for location in locations {
            let target_generation = target_version
                .allocation_records
                .iter()
                .find(|record| {
                    record.device == location.device
                        && record.slot == location.slot
                        && !record.is_released
                        && u64::from(record.span_slots) == unit.span_slots()
                })
                .map(|record| record.generation)
                .ok_or_else(|| {
                    malformed_target("回退目标那一版自己的账里没有它引用的用户可见单元的已分配记录")
                })?;
            let current_record = current_account
                .record_for(location.device, location.slot)
                .filter(|record| u64::from(record.span_slots) == unit.span_slots());
            match current_record {
                None => {
                    return Err(
                        RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount {
                            unit: *unit,
                            device: location.device,
                            slot: location.slot,
                        },
                    );
                }
                Some(record) if record.is_released => {
                    is_resurrected_on_some_device = true;
                    resurrected.push(ResurrectedCopy {
                        unit: *unit,
                        device: location.device,
                        placement: Placement {
                            slot: location.slot,
                            span: unit.span_slots(),
                        },
                        allocation_generation: target_generation,
                    });
                }
                Some(record) if record.generation != target_generation => {
                    return Err(
                        RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration {
                            unit: *unit,
                            device: location.device,
                            slot: location.slot,
                            current_account_generation: record.generation,
                            target_account_generation: target_generation,
                        },
                    );
                }
                Some(_shared_by_both_versions) => {}
            }
        }
        if is_resurrected_on_some_device {
            units_to_check_on_every_device.push((*unit, *locations));
        }
    }
    Ok(Resurrection {
        resurrected,
        units_to_check_on_every_device,
    })
}

/// 复活那一判交回的：复活集（逐盘），与复活集里有份的单元（逐盘验要读它们每一份）。
struct Resurrection {
    resurrected: Vec<ResurrectedCopy>,
    units_to_check_on_every_device: Vec<(TransactionUnit, [LocationEntry; 2])>,
}

/// 复活集里每个单元在每块盘上的那一份都要读得出、整单元 CRC-32C 与映射条目里的对得上（D23（journal 的角色与格式） 已定项 14
/// 「在任何写之前拒」第三样，主 agent 2026-09-26 定）：它们的槽在现行那一版的时间线上释放过，可能被一次没发布成的写盖掉一块盘上那一份。
/// 只读一次，不重读：读错只让这次回退被拒，不留下任何状态。
///
/// # Errors
/// `ResurrectedUnitCopyUnreadableOrMismatched`，按单元次序、位置项次序报第一份。
fn every_copy_of_the_resurrected_units_reads_back<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    units: &[(TransactionUnit, [LocationEntry; 2])],
) -> Result<(), RollbackError> {
    for (unit, locations) in units {
        let unit_bytes = usize::try_from(unit.span_slots() * SLOT_BYTES).expect("一个单元两槽以内");
        for location in locations {
            let is_intact = reader
                .read(
                    location.device,
                    location.slot.to_device_offset(),
                    unit_bytes,
                )
                .is_some_and(|copy| crc32_castagnoli(&copy) == location.unit_checksum);
            if !is_intact {
                return Err(RollbackError::ResurrectedUnitCopyUnreadableOrMismatched {
                    unit: *unit,
                    device: location.device,
                    slot: location.slot,
                });
            }
        }
    }
    Ok(())
}

/// `ring_roots` 里每一条带文件的根，它那一版记账里的 inode 号水位，取最大；从盘上重建那一版读不出的根不算（「环里读得出的根」）。
/// 只有管理员回退读它（D23（journal 的角色与格式） 已定项 14「水位」一格），交进来的只是按 cur 的实例表没被崩溃恢复抛弃的那几条
/// （Opt2，[`ring_roots_not_abandoned_by_crash_recovery`]）。`record_standing_for_every_root` 只是给重建顶着的一条记录
/// （重建出来的版本只拿它的 jsn 与事务号，这里都不用）。
#[allow(
    clippy::ptr_arg,
    reason = "rebuild_version 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn highest_inode_number_watermark_in_the_ring<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    ring_roots: &[RootRecord],
    record_standing_for_every_root: &crate::journal::JournalRecord,
) -> Option<u64> {
    ring_roots
        .iter()
        .filter_map(|root| {
            match rebuild_version_in_the_unit_area_starting_at(
                devices,
                unit_area_start,
                root,
                Some(record_standing_for_every_root.clone()),
            ) {
                Ok(
                    RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version),
                ) => Some(version.inode_number_watermark()),
                Ok(RebuiltVersion::WithoutFile) | Err(_) => None,
            }
        })
        .max()
}

/// 根环里读得出的根里，按 cur 的实例表（`current_table`）没被崩溃恢复抛弃的那几条（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2：
/// 管理员回退取 inode 号水位只看它们）。被抛弃的根在它那条时间线上发过的号，崩溃恢复落到较旧的根之后本来就会重发（D8（核心索引结构） 已定项 6 射程），
/// 算进来会让回退把水位抬过这条时间线没发过的号。
fn ring_roots_not_abandoned_by_crash_recovery(
    ring_roots: &[RootRecord],
    current_table: &InstanceTableRecords,
) -> Vec<RootRecord> {
    ring_roots
        .iter()
        .filter(|root| !abandoned_by_table(root, current_table))
        .cloned()
        .collect()
}

/// 管理员回退那次发布的 inode 号水位（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2）：max(cur 那一版内存里的水位,
/// 根环里按 cur 的实例表没被崩溃恢复抛弃的那几条根各自那一版的水位)。环里那一项由 `highest_inode_number_watermark_of` 去盘上重建
/// （回退路径交 [`highest_inode_number_watermark_in_the_ring`]）；取哪几条根、怎么与 cur 那一项合在这里，构造态单测直接调它
/// （判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F3：环那一项在走得到的状态上不大于 cur，去掉它整套用例照绿）。
/// cur 那一项不能去（只取环，回退那次就重发号，C143）。
fn inode_number_watermark_of_the_rollback(
    ring_roots: &[RootRecord],
    current_table: &InstanceTableRecords,
    inode_number_watermark_of_the_current_version: u64,
    highest_inode_number_watermark_of: impl FnOnce(&[RootRecord]) -> Option<u64>,
) -> u64 {
    highest_inode_number_watermark_of(&ring_roots_not_abandoned_by_crash_recovery(
        ring_roots,
        current_table,
    ))
    .map_or(inode_number_watermark_of_the_current_version, |ring| {
        ring.max(inode_number_watermark_of_the_current_version)
    })
}

/// 管理员回退那次发布的树 ID 水位（D23（journal 的角色与格式） 已定项 14「水位」一格）：max(cur 那一版内存里的水位, 根环里读得出的**全部**根的水位)。
/// 与 inode 号水位（[`inode_number_watermark_of_the_rollback`]）不同，被崩溃恢复抛弃的根照旧算：树 ID 发过就不再发，被抛弃的时间线上发过的号
/// 照样不能再发给别的树。`_current_table_not_consulted` 是 cur 的实例表，这里不读它：收它是为了与 inode 号水位那一段同形，
/// 构造态单测交一条按它被抛弃的根，「改成只取没被抛弃的根」这一错写得出来、会红（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`
/// 第三节 G5：回退用例在走得到的历史上分不出这两种取法）。
fn tree_identifier_watermark_of_the_rollback(
    ring_roots: &[RootRecord],
    _current_table_not_consulted: &InstanceTableRecords,
    tree_identifier_watermark_of_the_current_version: u64,
) -> u64 {
    ring_roots
        .iter()
        .map(|root| root.tree_identifier_watermark)
        .fold(tree_identifier_watermark_of_the_current_version, u64::max)
}

/// 回退那次发布接在后面的「上一版」：用户可见的那一半（数据单元与指针、extent 树、inode 树的叶容器与根、第一个文件那条 inode 记录）
/// 取回退目标那一版的，别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的；
/// 根记录、末条记录、事务号取现行那一版的（这次发布接着它发，反向链、计数器、txg 都接着它）。
/// 发布路径照这一版照抄用户可见单元（inode 树根指针从树表条目取，所以树表条目里 inode 树与 extent 树那两条换成目标那一版的）、
/// 按分配器现算固定点单元、换下现行那一版被重写的固定点单元——正是「新根的用户可见树取 R_old 的，树表单元、分配记录树、中央映射树与
/// 记账统计量按这次发布照调整之后的分配器现算」（D23（journal 的角色与格式） 已定项 14）。
/// 记账行原样取 cur 的：inode 号水位的台阶行照抄 cur 的（D5（快照 / 空间记账机制） 已定项 2 射程「管理员回退那次从 cur 带、不取 R_old 的」），
/// 回退那次发布的水位（「水位」一格的 max）由发布路径从谱系那一步拿（`publish_version_appending_a_lineage_record`），不改这里的行。
fn version_to_publish_the_rollback_after(
    current: &TransactionOutput,
    target_version: &TransactionOutput,
) -> TransactionOutput {
    let user_visible_units_of_the_target = target_version
        .units
        .iter()
        .filter(|unit| is_a_unit_taken_from_the_rollback_target(unit.identity))
        .cloned();
    let other_units_of_the_current_version = current
        .units
        .iter()
        .filter(|unit| !is_a_unit_taken_from_the_rollback_target(unit.identity))
        .cloned();
    let mapped_units = target_version
        .mapped_units
        .iter()
        .filter(|(identity, _)| is_a_unit_taken_from_the_rollback_target(*identity))
        .chain(
            current
                .mapped_units
                .iter()
                .filter(|(identity, _)| !is_a_unit_taken_from_the_rollback_target(*identity)),
        )
        .cloned()
        .collect();
    let target_root_of_the_tree_of_kind = |kind: u16| {
        target_version
            .tree_table_entries
            .iter()
            .find(|target_entry| target_entry.kind == kind)
            .expect("重建带文件的一版时判过树表里 extent 树与 inode 树各有一条（`recovery::rebuild_version`）")
            .root
    };
    let tree_table_entries = current
        .tree_table_entries
        .iter()
        .map(|entry| TreeTableEntry {
            kind: entry.kind,
            tree: entry.tree,
            root: if entry.kind == TREE_KIND_EXTENT
                || entry.kind == TREE_KIND_INODE
                || entry.kind == TREE_KIND_DELETED_INODES
            {
                target_root_of_the_tree_of_kind(entry.kind)
            } else {
                entry.root
            },
            birth_txg: entry.birth_txg,
            head_identifier: entry.head_identifier,
        })
        .collect();
    TransactionOutput {
        root: current.root,
        record: current.record.clone(),
        record_bytes: current.record_bytes.clone(),
        earlier_records_of_this_publish: Vec::new(),
        units: user_visible_units_of_the_target
            .chain(other_units_of_the_current_version)
            .collect(),
        rewritten: Vec::new(),
        data_pointers: target_version.data_pointers.clone(),
        mapping_keys: current.mapping_keys.clone(),
        allocation_records: current.allocation_records.clone(),
        allocation_record_tree: current.allocation_record_tree.clone(),
        extent_tree: target_version.extent_tree.clone(),
        accounting_entries: current.accounting_entries.clone(),
        accounting_tree: current.accounting_tree.clone(),
        central_mapping_tree: current.central_mapping_tree.clone(),
        tree_table_entries,
        tree_identifiers: current.tree_identifiers,
        inode_record: target_version.inode_record,
        // `deleted_inodes` 取 R_old 的（树表那一条上面换了），墓碑树随现行那一版（D23（journal 的角色与格式） 已定项 14）。
        deleted_inodes: target_version.deleted_inodes.clone(),
        tombstone_containers: current.tombstone_containers.clone(),
        tombstone_tree_index_nodes: current.tombstone_tree_index_nodes.clone(),
        // 谱系树同样随现行那一版（D18（块里携带什么信息） 已定项 24 第 5 条「随现行那一版、只增不删」）：回退这次往它上面追加。
        lineage_containers: current.lineage_containers.clone(),
        lineage_tree_index_nodes: current.lineage_tree_index_nodes.clone(),
        // 退役树同样随现行那一版、不取 R_old（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：
        // 回退那次不写退役记录，R_old 之后身份消失、回退之后又活过来的叶容器，它的退役记录死亡 txg 不在新这一版的父链上。
        retirement_containers: current.retirement_containers.clone(),
        retirement_tree_index_nodes: current.retirement_tree_index_nodes.clone(),
        inode_leaf_containers: target_version.inode_leaf_containers.clone(),
        mapped_units,
        released: Vec::new(),
        quarantined_after_release_checksum_mismatch: Vec::new(),
        key_order_mismatches: 0,
        writes: WritesByStructureKind::NOTHING_WRITTEN,
        highest_transaction_number_in_this_instance: current
            .highest_transaction_number_in_this_instance,
    }
}

/// 管理员回退：挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14「管理员回退是挂着时的一次向前发布」）。
/// 管理员从回退候选集里选 R_old（`target`）；文件系统在现行那一版（`current`，cur）后面发一次普通发布（D16（发布语义） 已定项 7 的
/// 持久顺序）：新根的 inode 树、extent 树取 R_old 的；树表单元、分配记录树、中央映射树与记账统计量按这次发布照调整之后的分配器现算；
/// 实例表照 cur 的；checkpoint_txg 加一；不新增实例、不取号、不写实例表行、不施加也不删除任何记录。
///
/// - 候选集：根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ 带文件（[`rollback_candidate`]）。
/// - 释放：cur 引用、新根不引用的用户可见单元（按落点比），释放代 = 新根的 txg；释放之前照发布路径读盘核，对不上的那几份留在已分配。
///   cur 被这次重写换下的固定点单元由发布路径照常释放。
/// - 复活：R_old 引用的用户可见单元里 cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的。
/// - 水位：inode 号水位 = max(cur 那一版内存里的, 环里读得出、按 cur 的实例表没被崩溃恢复抛弃的根的)（Opt2）；
///   树 ID 水位 = max(cur 那一版内存里的, 环里读得出的全部根的)。台阶行照抄 cur 的；谱系记录带 H_cur = cur 的 inode 号水位（从 cur 读）。
/// - 调用方的参数与盘表：照可写挂载同一套核（[`caller_inputs_agreeing_with_the_disk`]），写入口只按盘上择到的系统配置里那份参数建。
/// - 在任何写之前拒：见 [`RollbackError`] 除 `Publish` 落盘途中失败之外的每一个成员。
///
/// 新根持久之后才交回 `Ok`（与 fsync 同一个返回条件）；那时 `current` 换成回退那次发布之后的一版。回退那次发布的事务号写 0
/// （它不写数据单元，与空发布同）。回退的调用方要带文件的现行版本：cur 树表 0 条时它的时间线上还没有带文件的版本，
/// 按实例表判仍然有效、带文件的根一条都没有（候选集空），所以这里不收 `PoolVersion`。
///
/// # Errors
/// 见 [`RollbackError`]。
pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    target: RollbackTarget,
) -> Result<RolledBack, RollbackError> {
    if let Some(frozen) = allocator.frozen_publish() {
        return Err(
            RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                checkpoint_txg: frozen.checkpoint_txg(),
                instance: frozen.instance(),
            },
        );
    }
    // 调用方的参数与盘表照可写挂载同一套核，写入口只按盘上那一份建（实审 A1b 留下的这一处）。
    let CallerInputsAgreeingWithTheDisk {
        system_configuration,
        parameters_of_the_pool,
    } = caller_inputs_agreeing_with_the_disk(parameters, devices, current).map_err(|refused| {
        match refused {
            CallerInputsCheckRefused::Disagreeing(disagreeing) => {
                RollbackError::CallerInputsDisagreeWithTheDisk(disagreeing)
            }
            CallerInputsCheckRefused::SystemConfigurationNotChosen(failure) => {
                RollbackError::Recovery(failure)
            }
        }
    })?;
    let rollback_txg = current
        .root
        .checkpoint_txg
        .0
        .checked_add(1)
        .map(CheckpointTxg)
        .ok_or(RollbackError::NextCheckpointTxgPastTheTopOfItsRange {
            current: current.root.checkpoint_txg,
        })?;
    let current_table = instance_table_chain_of_root(&*devices, &current.root)
        .map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?
        .records;
    let target_root = rollback_candidate(
        &*devices,
        &system_configuration,
        &current_table,
        &ring_slots_known_to_hold_a_root_by(allocator),
        target,
    )?;
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
    // R_old 那一版整个从盘上重建（它的账、它的三棵用户可见树、记账树、中央映射树都读回来）：有一个节点读不出就拒。
    // 顶着的记录只给重建填字段，这里只用那一版的用户可见一半与它的账。
    let target_version = match rebuild_version_in_the_unit_area_starting_at(
        &*devices,
        unit_area_start,
        &target_root,
        Some(current.record.clone()),
    ) {
        // 有树而文件集合为空的一版也是候选：候选那一判按树表有没有条目判（带文件读成「那一版写过文件」），回退到它就是回到删空的那一刻。
        Ok(RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version)) => {
            version
        }
        // 候选那一判刚读过它的树表、不是 0 条：两次读之间的瞬时读错，按候选集那一判同一个结论报。
        Ok(RebuiltVersion::WithoutFile) => {
            return Err(RollbackError::TargetNotACandidate {
                target,
                exclusion: RollbackCandidateExclusion::VersionWithoutFile,
            });
        }
        Err(RebuildVersionFailure::Walk(failure)) => {
            return Err(RollbackError::TargetVersionUnreadable { target, failure });
        }
        Err(RebuildVersionFailure::NoRecordStandingForFileVersion) => {
            panic!(
                "交给重建的是 Some(现行那一版的末条记录)：rebuild_version 只在拿不出记录时报这一格"
            )
        }
    };
    // cur 那一版的账有一个节点读不出就拒：从盘上沿它的根读一遍分配记录树（判定与后面的写读的都是内存里那一份账，这一读只判读不读得出）。
    if let Err(failure) = allocation_records_under_root_in_the_unit_area_starting_at(
        &*devices,
        unit_area_start,
        &current.root,
    ) {
        return Err(RollbackError::CurrentAccountUnreadable { failure });
    }
    if target_version.tree_identifiers != current.tree_identifiers {
        return Err(
            RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
                target,
            },
        );
    }
    let target_units = user_visible_units_through_the_mapping(&target_version).map_err(|_| {
        RollbackError::TargetVersionUnreadable {
            target,
            failure: RecoveryFailure::UnitMalformed {
                what: "回退目标那一版引用的用户可见单元在它自己的映射里查不到或条目切不动",
            },
        }
    })?;
    let Resurrection {
        resurrected,
        units_to_check_on_every_device,
    } = resurrection_against_the_current_account(
        target,
        &target_version,
        &target_units,
        allocator,
    )?;
    every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;
    // 释放集：cur 引用的用户可见单元里，落点不在 R_old 引用的那几个里的（在的那几个上面判过同槽同分配代，是两版共用的）。
    let slots_referenced_by_the_target: BTreeSet<(DeviceIdentity, SlotNumber)> = target_units
        .iter()
        .flat_map(|(_, locations)| {
            locations
                .iter()
                .map(|location| (location.device, location.slot))
        })
        .collect();
    let pool_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let not_releasable = |cause| RollbackError::CurrentVersionUnitNotReleasable { cause };
    let mut released_user_visible_units = Vec::new();
    let mut quarantined_user_visible_copies = Vec::new();
    for (unit, locations) in
        user_visible_units_through_the_mapping(current).map_err(not_releasable)?
    {
        if locations.iter().any(|location| {
            slots_referenced_by_the_target.contains(&(location.device, location.slot))
        }) {
            continue;
        }
        refuse_locations_that_do_not_name_two_pool_devices(unit, &locations, &pool_devices)
            .map_err(not_releasable)?;
        let placement =
            placement_to_release_after_checking_every_device(unit, &locations, allocator)
                .map_err(not_releasable)?;
        quarantined_user_visible_copies.extend(
            copies_of_a_released_unit_failing_the_release_checksum_check(
                unit, &locations, &*devices,
            ),
        );
        released_user_visible_units.push((unit, placement));
    }
    let ring_roots = readable_roots(
        &*devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let tree_identifier_watermark = tree_identifier_watermark_of_the_rollback(
        &ring_roots,
        &current_table,
        current.root.tree_identifier_watermark,
    );
    // inode 号水位（Opt2，D23（journal 的角色与格式） 已定项 14「水位」一格）：环里只取按 cur 的实例表没被崩溃恢复抛弃的根；
    // max 里 cur 那一版内存里的那一项不能去——只取环，回退那次就重发号（C143）。
    let inode_number_watermark = inode_number_watermark_of_the_rollback(
        &ring_roots,
        &current_table,
        current.inode_number_watermark(),
        |ring_roots_not_abandoned| {
            highest_inode_number_watermark_in_the_ring(
                devices,
                unit_area_start,
                ring_roots_not_abandoned,
                &current.record,
            )
        },
    );
    let previous = version_to_publish_the_rollback_after(current, &target_version);
    // 动分配器：先复活、再释放（释放代 = 新根的 txg），之后交给发布路径——它在这份账上照常释放 cur 被换下的固定点单元、取落点、
    // 装记账行。发布在落盘之前被拒时整个换回这一份之前的样子。
    let allocator_before_the_rollback = allocator.clone();
    for copy in &resurrected {
        allocator.resurrect_released_record(
            copy.device,
            copy.placement,
            copy.allocation_generation,
        );
    }
    for (_, placement) in &released_user_visible_units {
        let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> =
            quarantined_user_visible_copies
                .iter()
                .filter(|copy: &&CopyQuarantinedAfterReleaseChecksumMismatch| {
                    copy.placement == *placement
                })
                .map(|copy| copy.device)
                .collect();
        allocator.release_leaving_the_record_allocated_on(
            *placement,
            rollback_txg,
            &devices_whose_copy_failed_the_checksum,
        );
    }
    let plan = PublishPlan {
        txg: rollback_txg,
        counter: current.record.counter + 1,
        transaction: 0,
        highest_transaction_number_before_this_publish: current
            .highest_transaction_number_in_this_instance,
        instance: current.root.instance,
        back_chain: back_chain_of(&current.record_bytes),
        file: None,
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current.root.instance_table),
        tree_birth_txg: current.tree_birth_txg(),
        tree_identifier_watermark,
        rollback_floor: current.root.rollback_floor,
    };
    // 回退那次发布追加一条谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条）：T_rb 是这次的 txg，T_old 是 R_old 的 txg，
    // H_cur 是 cur 那一版的 inode 号水位（从 cur 读，不从交给发布路径的「上一版」现算）。
    let lineage_record = LineageRecord {
        rollback_txg,
        rollback_target_txg: target_root.checkpoint_txg,
        inode_number_watermark_of_the_version_rolled_back_from: current.inode_number_watermark(),
    };
    let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
    // 谱系容器与容器树节点的容量取分配器上装的只供测试的开关（产品路径按格式），与会话里的删除同一个做法。
    pool.set_container_tree_capacities(allocator.deletion_switches().container_tree_capacities());
    match publish_version_appending_a_lineage_record(
        &mut pool,
        allocator,
        plan,
        &previous,
        lineage_record,
        inode_number_watermark,
    ) {
        Ok(published) => {
            *current = published;
            Ok(RolledBack {
                target,
                released_user_visible_units,
                quarantined_user_visible_copies,
                resurrected_user_visible_copies: resurrected,
            })
        }
        Err(cause) => {
            // 落盘之前被拒：发布路径把分配器换回了复活、释放之后那一份，这里再换回回退之前的。落盘途中失败：那次发布冻结在分配器上
            // （连同它成立之后的那一份），分配器留着复活、释放之后那一份，重发成功时整个换成冻结着的那一份。
            if allocator.frozen_publish().is_none() {
                *allocator = allocator_before_the_rollback;
            }
            Err(RollbackError::Publish(publish_sequence_failed(
                &pool,
                WritesByStructureKind::NOTHING_WRITTEN,
                Vec::new(),
                cause,
            )))
        }
    }
}

#[cfg(test)]
mod non_empty_root_tests {
    use super::{
        ceiling_from_newest_and_non_empty_roots, newest_first_distinct_states,
        user_visible_trees_changed, NonEmptyValidRoot,
    };
    use crate::address::{CheckpointTxg, InstanceGeneration};
    use crate::recovery::UserVisibleTreeRootPointers;

    fn non_empty_root(checkpoint_txg: u64, state: u8) -> NonEmptyValidRoot {
        NonEmptyValidRoot {
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            instance: InstanceGeneration(2),
            user_visible_tree_root_pointers: pointers(state, state),
        }
    }

    /// 「4 个不同状态」去重（D16（发布语义） 已定项 1「「非空」从盘上怎么认」末段）：从新到旧数，状态与更新的已计入根相同的不计入。
    /// A(3)、B(4)、C(8) 各一个状态，D(9) 是回退到 A 发出的新根、状态同 A：从新到旧计入 D、C、B，A 不计——三个不同状态，
    /// 第 4 新的没有，上限落到最旧的有效根。不去重时 A 被当成第 4 个，上限是 3。
    #[test]
    fn the_rollback_root_repeating_an_older_state_is_counted_once_and_the_ceiling_falls_to_the_oldest_valid_root(
    ) {
        let distinct = newest_first_distinct_states(vec![
            non_empty_root(3, 0xA),
            non_empty_root(4, 0xB),
            non_empty_root(8, 0xC),
            non_empty_root(9, 0xA),
        ]);
        assert_eq!(
            distinct,
            vec![CheckpointTxg(9), CheckpointTxg(8), CheckpointTxg(4)],
            "从新到旧：D（状态 A）、C、B 计入，A 与 D 同一个状态不再计"
        );
        assert_eq!(
            ceiling_from_newest_and_non_empty_roots(
                CheckpointTxg(9),
                distinct,
                Some(CheckpointTxg(1))
            ),
            Some(CheckpointTxg(1)),
            "不同状态只有 3 个：上限取最旧的有效根（根环容量边界那一格）"
        );
        let four_states = newest_first_distinct_states(vec![
            non_empty_root(3, 0xA),
            non_empty_root(4, 0xB),
            non_empty_root(8, 0xC),
            non_empty_root(9, 0xD),
        ]);
        assert_eq!(
            ceiling_from_newest_and_non_empty_roots(
                CheckpointTxg(9),
                four_states,
                Some(CheckpointTxg(1))
            ),
            Some(CheckpointTxg(3)),
            "四个不同状态：上限取第 4 新的那一条"
        );
    }

    fn pointers(inode_tree: u8, extent_tree: u8) -> UserVisibleTreeRootPointers {
        UserVisibleTreeRootPointers {
            inode_tree: Some(vec![inode_tree; 86]),
            extent_tree: Some(vec![extent_tree; 86]),
        }
    }

    /// 两棵树任一棵的根指针变了就算非空：只换 extent 树（同一个 inode 的内容覆盖写、inode 记录原地不动的那种发布）与只换 inode 树都要认出来，
    /// 两棵都没变的（空发布、回退那次照抄上一版的）不算；最旧的有效根与「树表里没有这两棵树的条目」比。
    #[test]
    fn root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root(
    ) {
        assert!(
            user_visible_trees_changed(&pointers(1, 2), &pointers(1, 3)),
            "只有 extent 树的根指针变了"
        );
        assert!(
            user_visible_trees_changed(&pointers(4, 2), &pointers(1, 2)),
            "只有 inode 树的根指针变了"
        );
        assert!(
            !user_visible_trees_changed(&pointers(1, 2), &pointers(1, 2)),
            "两棵树都照抄：空发布"
        );
        assert!(
            user_visible_trees_changed(&pointers(1, 2), &UserVisibleTreeRootPointers::ABSENT),
            "最旧的有效根下面有文件"
        );
        assert!(
            !user_visible_trees_changed(
                &UserVisibleTreeRootPointers::ABSENT,
                &UserVisibleTreeRootPointers::ABSENT
            ),
            "mkfs 的第 0 代根与暖机根：树表里没有这两棵树"
        );
    }
}

#[cfg(test)]
mod reclaim_floor_tests {
    use super::reclaim_floor;
    use crate::address::CheckpointTxg;

    /// 第 0 代根还在环里时门槛就是 F_生效；被盖之后环里最旧的有效根接管（比 F 大时取它）。
    #[test]
    fn reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root() {
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(0))),
            CheckpointTxg(11)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(0), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(reclaim_floor(CheckpointTxg(11), None), CheckpointTxg(11));
    }
}

#[cfg(test)]
mod txgs_ring_slots_bad_at_mount_can_hold_tests {
    use super::txgs_ring_slots_bad_at_mount_can_hold;
    use crate::address::CheckpointTxg;
    use crate::allocator::TxgsARingSlotBadAtMountCanHold;
    use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
    use singlefs_format::ROOT_RING_REGIONS;
    use std::collections::{BTreeMap, BTreeSet};

    /// 两个槽之外环里每个槽都读坏（两个槽读出了根）。
    fn bad_ring_slots_but_two(slots_per_region: RootRingSlotsPerRegion) -> BTreeSet<RootRingSlot> {
        let ring_slots_read = [
            RootRingSlot { region: 0, slot: 0 },
            RootRingSlot { region: 1, slot: 2 },
        ];
        (0..ROOT_RING_REGIONS)
            .flat_map(|region| {
                (0..slots_per_region.count()).map(move |slot| RootRingSlot { region, slot })
            })
            .filter(|ring_slot| !ring_slots_read.contains(ring_slot))
            .collect()
    }

    /// 参照：[0, first_txg) 与区间 (N − 3S, first_txg − 1] 里的 txg 一个一个走（只给区间装得下的组用）。
    fn walk_every_txg_below_the_first(
        bad_ring_slots: &BTreeSet<RootRingSlot>,
        window_anchor: CheckpointTxg,
        first_txg: CheckpointTxg,
        slots_per_region: RootRingSlotsPerRegion,
    ) -> Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)> {
        let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
        let lowest_txg_inside_the_window = window_anchor
            .0
            .checked_sub(ring_length_in_txgs)
            .map_or(0, |below_the_window| below_the_window + 1);
        let mut oldest_of_each: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
        let mut highest_of_each: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
        for txg in 0..first_txg.0 {
            let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
            if !bad_ring_slots.contains(&ring_slot) {
                continue;
            }
            oldest_of_each
                .entry(ring_slot)
                .or_insert(CheckpointTxg(txg));
            if txg >= lowest_txg_inside_the_window {
                highest_of_each.insert(ring_slot, CheckpointTxg(txg));
            }
        }
        oldest_of_each
            .into_iter()
            .map(|(ring_slot, oldest)| {
                (
                    ring_slot,
                    TxgsARingSlotBadAtMountCanHold::new(
                        oldest,
                        *highest_of_each
                            .get(&ring_slot)
                            .expect("住得下就在区间里也住得下"),
                    ),
                )
            })
            .collect()
    }

    /// 锚 40、first_txg 41（S = 4，区间 [29, 40] 正好一圈）：区域 2 槽 3 能住的最大 txg 是区间里余 11 的 35，
    /// 可能住过的最旧 txg 是第一圈的 11——回收门槛钉在 11，不在窗口里的 35（用户 2026-10-04 定）。
    #[test]
    fn a_bad_slot_is_pinned_at_the_oldest_txg_it_can_have_held_below_the_window() {
        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
            .expect("4 是 S 区间 [4, 16] 的下界");
        let txgs = txgs_ring_slots_bad_at_mount_can_hold(
            &BTreeSet::from([RootRingSlot { region: 2, slot: 3 }]),
            CheckpointTxg(40),
            CheckpointTxg(41),
            slots_per_region,
        );
        assert_eq!(
            txgs,
            vec![(
                RootRingSlot { region: 2, slot: 3 },
                TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(11), CheckpointTxg(35))
            )]
        );
    }

    /// 年轻的环（first_txg 5，S = 4）：txg 5 起才轮到的槽一个 txg 都住不了，不交回。
    #[test]
    fn a_bad_slot_the_young_ring_has_not_reached_holds_no_txg() {
        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
            .expect("4 是 S 区间 [4, 16] 的下界");
        let not_reached = target_for_publish(CheckpointTxg(7), slots_per_region);
        let reached = target_for_publish(CheckpointTxg(4), slots_per_region);
        assert_eq!(
            txgs_ring_slots_bad_at_mount_can_hold(
                &BTreeSet::from([not_reached, reached]),
                CheckpointTxg(4),
                CheckpointTxg(5),
                slots_per_region,
            ),
            vec![(
                reached,
                TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(4), CheckpointTxg(4))
            )]
        );
    }

    /// 区间短于 3S、正好 3S、长于 3S 若干圈、年轻的环与空区间几组逐项比参照；first_txg 是 `u64::MAX` 那一组（外来记录的 txg 是
    /// `u64::MAX − 1`）参照走不完，比的是按 3S 周期手算的期望。
    #[test]
    fn walking_one_period_at_each_end_matches_walking_every_txg_below_the_first() {
        for declared_slots_per_region in [4, 5] {
            let slots_per_region =
                RootRingSlotsPerRegion::from_system_configuration_field(declared_slots_per_region)
                    .expect("4 与 5 落在 S 的区间 [4, 16] 里");
            let ring_length_in_txgs = ROOT_RING_REGIONS * declared_slots_per_region;
            let bad_ring_slots = bad_ring_slots_but_two(slots_per_region);
            let anchor_above_one_revolution = 40;
            let lowest_txg_inside_the_window =
                anchor_above_one_revolution - ring_length_in_txgs + 1;
            let anchor_and_first_txg_of_each_group = [
                (anchor_above_one_revolution, anchor_above_one_revolution + 1),
                (
                    anchor_above_one_revolution,
                    lowest_txg_inside_the_window + ring_length_in_txgs + 5,
                ),
                (
                    anchor_above_one_revolution,
                    lowest_txg_inside_the_window + 2 * ring_length_in_txgs - 1,
                ),
                (
                    anchor_above_one_revolution,
                    lowest_txg_inside_the_window + 4 * ring_length_in_txgs + 7,
                ),
                (5, 6),
                (5, 9),
                (5, 5 * ring_length_in_txgs + 3),
                (0, 0),
            ];
            for (window_anchor, first_txg) in anchor_and_first_txg_of_each_group {
                assert_eq!(
                    txgs_ring_slots_bad_at_mount_can_hold(
                        &bad_ring_slots,
                        CheckpointTxg(window_anchor),
                        CheckpointTxg(first_txg),
                        slots_per_region,
                    ),
                    walk_every_txg_below_the_first(
                        &bad_ring_slots,
                        CheckpointTxg(window_anchor),
                        CheckpointTxg(first_txg),
                        slots_per_region,
                    ),
                    "S = {declared_slots_per_region}，窗口锚 = {window_anchor}，first_txg = {first_txg}"
                );
            }
        }

        // S = 4，3S = 12：txg 模 12 的余数 r 落在区域 r mod 3 的槽 r div 3（D22 已定项 2 的槽位公式在一个周期里展开），
        // 可能住过的最旧 txg 就是 r。2^64 模 12 是 4，所以 u64::MAX 模 12 是 3：区间尾 u64::MAX − 12 ..= u64::MAX − 1 里余数是 r 的那个
        // txg 是 u64::MAX − k，k 取 1..=12 里 (3 − k) 模 12 等于 r 的那一个。
        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
            .expect("4 是 S 区间 [4, 16] 的下界");
        let ring_length_in_txgs = 12;
        assert_eq!(u64::MAX % ring_length_in_txgs, 3);
        let bad_ring_slots = bad_ring_slots_but_two(slots_per_region);
        let mut expected: Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)> = (0
            ..ring_length_in_txgs)
            .map(|residue| {
                let distance_modulo_the_ring =
                    (3 + ring_length_in_txgs - residue) % ring_length_in_txgs;
                let distance_below_u64_maximum = if distance_modulo_the_ring == 0 {
                    ring_length_in_txgs
                } else {
                    distance_modulo_the_ring
                };
                (
                    RootRingSlot {
                        region: residue % 3,
                        slot: residue / 3,
                    },
                    TxgsARingSlotBadAtMountCanHold::new(
                        CheckpointTxg(residue),
                        CheckpointTxg(u64::MAX - distance_below_u64_maximum),
                    ),
                )
            })
            .filter(|(ring_slot, _)| bad_ring_slots.contains(ring_slot))
            .collect();
        expected.sort_by_key(|(ring_slot, _)| *ring_slot);
        assert_eq!(
            txgs_ring_slots_bad_at_mount_can_hold(
                &bad_ring_slots,
                CheckpointTxg(40),
                CheckpointTxg(u64::MAX),
                slots_per_region,
            ),
            expected
        );
    }
}

#[cfg(test)]
mod newest_valid_root_on_every_device_tests {
    use super::newest_valid_root_txg_on_every_device;
    use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
    use crate::instance_table::{InstanceRow, InstanceTableRecords};
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};

    fn root_of(instance: u32, checkpoint_txg: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: [0; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
        }
    }

    /// 这几条用例的落点公式：偶数 txg 落盘 0、奇数落盘 1（只为把每块盘上有哪几条根摆清楚，不是第一版几何）。
    fn device_of_an_even_or_odd_txg(checkpoint_txg: CheckpointTxg) -> DeviceIdentity {
        if checkpoint_txg.0.is_multiple_of(2) {
            DeviceIdentity(0)
        } else {
            DeviceIdentity(1)
        }
    }

    /// 实例 1 择过 txg 8（实例表那一行），它之后的 txg 9 被抛弃；实例 2 从 txg 12 起。盘 1 上最新的根是被抛弃的 txg 9，
    /// 最新的有效根是 txg 7：取最小时按 7 算（滤掉被抛弃的根），不按 9 算。只有被抛弃的根的那块盘算「一条有效根都没有」。
    /// 判别力：不滤被抛弃的根时第一条断言得 9、第二条得 Some(9)。
    #[test]
    fn an_abandoned_root_newest_on_a_device_does_not_stand_for_that_devices_newest_valid_root() {
        let table = InstanceTableRecords {
            rows: vec![InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(8),
                applied_transaction_high_water: 0,
            }],
        };
        let both_devices = [DeviceIdentity(0), DeviceIdentity(1)];
        let on_their_devices = |roots: &[RootRecord]| -> Vec<(DeviceIdentity, RootRecord)> {
            roots
                .iter()
                .map(|root| (device_of_an_even_or_odd_txg(root.checkpoint_txg), *root))
                .collect()
        };
        let roots = [root_of(1, 7), root_of(1, 8), root_of(1, 9), root_of(2, 12)];
        assert_eq!(
            newest_valid_root_txg_on_every_device(&on_their_devices(&roots), &table, &both_devices,),
            Some(CheckpointTxg(7)),
            "盘 0 上最新的有效根 txg 12、盘 1 上 txg 7（txg 9 被抛弃、不算），取最小是 7"
        );
        let only_the_abandoned_root_on_device_one = [root_of(1, 8), root_of(1, 9), root_of(2, 12)];
        assert_eq!(
            newest_valid_root_txg_on_every_device(
                &on_their_devices(&only_the_abandoned_root_on_device_one),
                &table,
                &both_devices,
            ),
            None,
            "盘 1 上只有被抛弃的 txg 9：那块盘一条有效根都没有"
        );
    }
}

#[cfg(test)]
mod merged_readings_of_the_two_root_ring_passes_tests {
    use super::{
        better_reading_of_a_root_ring_slot_over_the_two_passes,
        merged_readings_of_the_two_root_ring_passes, RootRingSlotReadingTellingNeverWrittenApart,
    };
    use crate::address::{CheckpointTxg, InstanceGeneration};
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};
    use crate::root_ring::RootRingSlot;
    use RootRingSlotReadingTellingNeverWrittenApart::{
        AllZeroNeverWritten, BadAndNotAllZero, SelfVerified,
    };

    fn root_of(instance: u32, checkpoint_txg: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: [0; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
        }
    }

    /// 场景：一个根环槽两遍读数的九种组合（第一遍、重读各取自证过、读坏不是全零、全零三种），两遍都自证过时两遍读出的是两条不同的根
    /// （只为看出取的是哪一遍的，挂载里同一槽两遍读出的是同一条）。
    /// 预期：照用户 2026-10-05 定 R1 的次序——任一遍自证过的根（两遍都自证过取重读那一条）＞ 任一遍读坏不是全零 ＞ 全零。
    /// 判别力：`crates/mutations.tsv` 里「第10h批 R1 …」那几行各改一处判别，各红在点名那一组上。
    #[test]
    fn each_pair_of_readings_merges_into_the_self_verified_root_then_the_bad_reading_then_all_zero()
    {
        let root_of_the_first_pass = root_of(1, 7);
        let root_of_the_reread = root_of(1, 31);
        let pairs_and_merged = [
            (
                SelfVerified(root_of_the_first_pass),
                SelfVerified(root_of_the_reread),
                SelfVerified(root_of_the_reread),
                "两遍都自证过：取重读那一条",
            ),
            (
                BadAndNotAllZero,
                SelfVerified(root_of_the_reread),
                SelfVerified(root_of_the_reread),
                "第一遍读坏、重读自证过：取重读那一条",
            ),
            (
                AllZeroNeverWritten,
                SelfVerified(root_of_the_reread),
                SelfVerified(root_of_the_reread),
                "第一遍全零、重读自证过：取重读那一条",
            ),
            (
                SelfVerified(root_of_the_first_pass),
                BadAndNotAllZero,
                SelfVerified(root_of_the_first_pass),
                "第一遍自证过、重读读坏：取第一遍那一条",
            ),
            (
                SelfVerified(root_of_the_first_pass),
                AllZeroNeverWritten,
                SelfVerified(root_of_the_first_pass),
                "第一遍自证过、重读全零：取第一遍那一条",
            ),
            (
                BadAndNotAllZero,
                BadAndNotAllZero,
                BadAndNotAllZero,
                "两遍都读坏：读坏",
            ),
            (
                BadAndNotAllZero,
                AllZeroNeverWritten,
                BadAndNotAllZero,
                "第一遍读坏、重读全零：读坏压过全零",
            ),
            (
                AllZeroNeverWritten,
                BadAndNotAllZero,
                BadAndNotAllZero,
                "第一遍全零、重读读坏：读坏压过全零",
            ),
            (
                AllZeroNeverWritten,
                AllZeroNeverWritten,
                AllZeroNeverWritten,
                "两遍都全零：全零",
            ),
        ];
        // 迭代上界是九种组合；没有提前出口。
        for (first_reading, reading_of_the_reread, expected, case) in pairs_and_merged {
            assert_eq!(
                better_reading_of_a_root_ring_slot_over_the_two_passes(
                    first_reading,
                    reading_of_the_reread
                ),
                expected,
                "{case}"
            );
        }
    }

    /// 场景：三个槽的两遍读数整份并（第一遍：自证过、读坏、全零；重读：读坏、自证过、读坏）。
    /// 预期：逐槽并、次序与槽不变——第一个槽留第一遍的根，第二个槽取重读的根，第三个槽读坏。
    /// 判别力：并的时候槽错位、或只取其中一遍时这一项红（改回整份换成重读那一遍改的是 `rebuilt_allocator` 里的调用点，由 harness 档用例罩）。
    #[test]
    fn the_two_passes_are_merged_slot_by_slot_in_ring_order() {
        let slots = [
            RootRingSlot { region: 0, slot: 0 },
            RootRingSlot { region: 1, slot: 0 },
            RootRingSlot { region: 2, slot: 0 },
        ];
        let root_of_the_first_pass = root_of(2, 8);
        let root_of_the_reread = root_of(3, 13);
        let first_readings = vec![
            (slots[0], SelfVerified(root_of_the_first_pass)),
            (slots[1], BadAndNotAllZero),
            (slots[2], AllZeroNeverWritten),
        ];
        let readings_of_the_reread = vec![
            (slots[0], BadAndNotAllZero),
            (slots[1], SelfVerified(root_of_the_reread)),
            (slots[2], BadAndNotAllZero),
        ];
        assert_eq!(
            merged_readings_of_the_two_root_ring_passes(&first_readings, readings_of_the_reread),
            vec![
                (slots[0], SelfVerified(root_of_the_first_pass)),
                (slots[1], SelfVerified(root_of_the_reread)),
                (slots[2], BadAndNotAllZero),
            ],
            "逐槽取较好的那一份，次序与槽不变"
        );
    }
}

#[cfg(test)]
mod placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests {
    use super::with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots;
    use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
    use crate::allocator::PlacementOnDevice;
    use crate::root_ring::{target_for_publish, RootRingSlotsPerRegion};
    use std::collections::BTreeMap;

    fn placement(slot: u64) -> PlacementOnDevice {
        PlacementOnDevice {
            device: DeviceIdentity(0),
            slot: SlotNumber(slot),
            span: 2,
        }
    }

    /// 所选那一版 txg 6，走了丙那一支的根 R txg 9，其间 Q（txg 7）、txg 8 两槽在 C 里各记着落点，R 之后的 txg 10 那一槽也记着：
    /// R 那一槽并上 txg 7、8 两槽记着的、R 那一槽里还没有的落点（R 那一槽已有的不再并一次），txg 10 那一槽的不并；别的槽原样。
    #[test]
    fn the_ring_slot_of_a_root_taking_the_current_account_takes_what_the_publishes_in_between_named(
    ) {
        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
        let named_by_ring_slot = BTreeMap::from([
            (ring_slot_of(7), vec![placement(100), placement(102)]),
            (ring_slot_of(8), vec![placement(104), placement(200)]),
            (ring_slot_of(9), vec![placement(200)]),
            (ring_slot_of(10), vec![placement(300)]),
        ]);
        let chained = with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
            named_by_ring_slot.clone(),
            CheckpointTxg(6),
            &[CheckpointTxg(9)],
            slots_per_region,
        );
        assert_eq!(
            chained[&ring_slot_of(9)],
            vec![
                placement(200),
                placement(100),
                placement(102),
                placement(104)
            ],
            "R 那一槽：原有的在前，再按槽的次序并上 txg 7、8 两槽的，已有的 200 不再并"
        );
        for txg in [7, 8, 10] {
            assert_eq!(
                chained[&ring_slot_of(txg)],
                named_by_ring_slot[&ring_slot_of(txg)],
                "txg {txg} 那一槽原样（中间那几槽照旧在自己的 txg 上解开）"
            );
        }
        assert_eq!(chained.len(), named_by_ring_slot.len(), "不新开别的槽");
    }

    /// 走了丙那一支的根紧挨着所选那一版（其间没有别的发布）：什么都不并，原样交回。
    #[test]
    fn a_root_taking_the_current_account_right_after_the_selected_version_takes_nothing_more() {
        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
        let named_by_ring_slot = BTreeMap::from([
            (ring_slot_of(6), vec![placement(100)]),
            (ring_slot_of(7), vec![placement(200)]),
        ]);
        assert_eq!(
            with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
                named_by_ring_slot.clone(),
                CheckpointTxg(6),
                &[CheckpointTxg(7)],
                slots_per_region,
            ),
            named_by_ring_slot,
            "(6, 7) 之间没有 txg：原样"
        );
    }

    /// 两条走了丙那一支的根各并各自与所选那一版之间那几槽（正推 K5 的 E7：只并最后一条照绿）。交进来的 txg 按根环里读出的次序，
    /// 那不是 txg 的次序：S = 8 时 txg 24 在区域 0 槽 0、txg 23 在区域 2 槽 7，较新的 24 排在前面。所选那一版 txg 21，其间 Q（txg 22）。
    /// 24 那一槽并 22、23 两槽的，23 那一槽并 22 那一槽的；只并其中一条时另一条那一槽缺它的中间那几槽
    /// （只并 23 时 24 那一槽缺 23 写的 200：23 那一槽先于 24 那一槽写掉，其间 24 还在环里、引用着 200）。
    #[test]
    fn every_root_taking_the_current_account_takes_what_the_publishes_between_it_and_the_selected_version_named(
    ) {
        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
        assert!(
            ring_slot_of(24) < ring_slot_of(23),
            "根环里读出的次序是区域、槽：txg 24 排在 txg 23 前面（这条用例要的那一形）"
        );
        let named_by_ring_slot = BTreeMap::from([
            (ring_slot_of(22), vec![placement(100)]),
            (ring_slot_of(23), vec![placement(200)]),
            (ring_slot_of(24), vec![placement(300)]),
        ]);
        let chained = with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
            named_by_ring_slot.clone(),
            CheckpointTxg(21),
            &[CheckpointTxg(24), CheckpointTxg(23)],
            slots_per_region,
        );
        assert_eq!(
            chained[&ring_slot_of(24)],
            vec![placement(300), placement(100), placement(200)],
            "txg 24 那一槽：原有的在前，再并上 22、23 两槽的"
        );
        assert_eq!(
            chained[&ring_slot_of(23)],
            vec![placement(200), placement(100)],
            "txg 23 那一槽：原有的在前，再并上 22 那一槽的"
        );
        assert_eq!(
            chained[&ring_slot_of(22)],
            named_by_ring_slot[&ring_slot_of(22)],
            "txg 22 那一槽原样"
        );
    }
}

/// 认回退的三种结局（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」）：只算内存的那一半
/// （[`super::rollback_recognized_among_the_packed_units_named_by_a_publish`]）。读盘那一半与落到 C 里的那一截由 harness 档
/// `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 里回退复活那几条用例判。
#[cfg(test)]
mod rollback_recognized_among_the_packed_units_named_by_a_publish_tests {
    use super::{
        rollback_recognized_among_the_packed_units_named_by_a_publish,
        PackedUnitNamedByAPublishAsRead, RollbackRecognizedAmongThePackedUnitsNamedByAPublish,
    };
    use crate::address::CheckpointTxg;
    use crate::records::LineageRecord;
    use crate::unit::{PACKED_TYPE_INODE, PACKED_TYPE_LINEAGE};

    /// 这几条用例里那次发布的 txg。
    const CHECKPOINT_TXG_OF_THE_PUBLISH: CheckpointTxg = CheckpointTxg(9);

    fn lineage_record_bytes(rollback_txg: u64, rollback_target_txg: u64) -> Vec<u8> {
        LineageRecord {
            rollback_txg: CheckpointTxg(rollback_txg),
            rollback_target_txg: CheckpointTxg(rollback_target_txg),
            inode_number_watermark_of_the_version_rolled_back_from: 3,
        }
        .to_bytes()
    }

    fn parsed(record_type: u16, records: Vec<Vec<u8>>) -> PackedUnitNamedByAPublishAsRead {
        PackedUnitNamedByAPublishAsRead::Parsed {
            record_type,
            records,
        }
    }

    /// 谱系片里有 rollback_txg 等于这次发布 txg 的那一条：是回退，回到它的 T_old；同一片里别的发布的谱系记录不碍事。
    #[test]
    fn a_lineage_record_of_this_publish_recognizes_a_rollback_to_its_target() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    parsed(PACKED_TYPE_INODE, vec![vec![0; 16]]),
                    parsed(
                        PACKED_TYPE_LINEAGE,
                        vec![lineage_record_bytes(7, 3), lineage_record_bytes(9, 5)]
                    ),
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(CheckpointTxg(5)),
        );
    }

    /// 点名的打包单元全解得开，谱系片里只有别的发布（txg 7）的谱系记录：肯定不是回退（正推 K5 的 E2：不比 rollback_txg 时认成回到 3）。
    #[test]
    fn lineage_records_of_other_publishes_only_are_not_a_rollback() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    parsed(PACKED_TYPE_INODE, vec![vec![0; 16]]),
                    parsed(PACKED_TYPE_LINEAGE, vec![lineage_record_bytes(7, 3)]),
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback,
        );
    }

    /// 不是谱系片的打包单元里的记录不当谱系记录读：一片 inode 叶里的 24 字节记录恰好拼得出 (9, 5, 3)，也不认成回退。
    #[test]
    fn records_of_a_packed_unit_that_is_not_a_lineage_unit_are_not_read_as_lineage_records() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[parsed(PACKED_TYPE_INODE, vec![lineage_record_bytes(9, 5)])],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback,
        );
    }

    /// 点名的打包单元有一片读不出（或校验和对不上），读得出的那几片里没认出：不知道是不是回退。
    #[test]
    fn an_unreadable_named_packed_unit_without_a_recognized_rollback_is_unknown() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    parsed(PACKED_TYPE_INODE, vec![vec![0; 16]]),
                    PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
        );
    }

    /// 点名的打包单元有一片读得出、解不开：同样不知道。
    #[test]
    fn an_unparsable_named_packed_unit_without_a_recognized_rollback_is_unknown() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    PackedUnitNamedByAPublishAsRead::ReadButUnparsable,
                    parsed(PACKED_TYPE_LINEAGE, vec![lineage_record_bytes(7, 3)]),
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
        );
    }

    /// 谱系片解开了、里面有一条记录解不开（长度不是 24），别的记录都不是这次发布的：同样不知道。
    #[test]
    fn an_unparsable_lineage_record_without_a_recognized_rollback_is_unknown() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[parsed(
                    PACKED_TYPE_LINEAGE,
                    vec![lineage_record_bytes(7, 3), vec![0; 23]]
                )],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
        );
    }

    /// 读得出的那几片里认出了回退，就是回退：另一片读不出不改结局（回退那条谱系记录只这一次发布写）。
    #[test]
    fn a_recognized_rollback_stands_though_another_named_packed_unit_is_unreadable() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[
                    PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,
                    parsed(PACKED_TYPE_LINEAGE, vec![lineage_record_bytes(9, 5)]),
                ],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(CheckpointTxg(5)),
        );
    }

    /// 一片打包单元都没点名（读得出的记录里没有类标签 3 的）：肯定不是回退（回退那次发布必定写谱系片）。
    #[test]
    fn no_named_packed_unit_is_not_a_rollback() {
        assert_eq!(
            rollback_recognized_among_the_packed_units_named_by_a_publish(
                CHECKPOINT_TXG_OF_THE_PUBLISH,
                &[],
            ),
            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback,
        );
    }
}

#[cfg(test)]
mod packed_units_named_by_the_records_of_a_publish_as_read_tests {
    use super::{
        packed_units_named_by_the_records_of_a_publish_as_read, PackedUnitNamedByAPublishAsRead,
    };
    use crate::address::{
        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
        TreeIdentifier,
    };
    use crate::block_device::{
        BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
    };
    use crate::checksum::crc32_castagnoli;
    use crate::journal::{
        JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, NamedUnit,
    };
    use crate::pointer::{BirthSequence, LocationEntry, NodePointer};
    use crate::records::LineageRecord;
    use crate::root_record::UnmountMarker;
    use crate::unit::{
        build_packed_unit, PackedIdentity, WriteOrder, PACKED_TYPE_INODE, PACKED_TYPE_LINEAGE,
        UNIT_CLASS_DATA, UNIT_CLASS_PACKED,
    };
    use singlefs_format::{DATA_UNIT_BYTES, LINEAGE_RECORD_BYTES, SLOT_BYTES};

    /// 这几条用例里那块盘的槽数：四个打包单元各占两槽，再留几槽。
    const SLOTS_OF_THE_DEVICE: u64 = 16;

    /// 这几条用例里点名的单元出生的那一代、那次发布的 txg。
    const CHECKPOINT_TXG_OF_THE_PUBLISH: CheckpointTxg = CheckpointTxg(9);

    /// 一块只在内存里的盘：读按偏移切字节，越过末尾报越界。
    struct DeviceOfBytesInMemory {
        bytes: Vec<u8>,
    }

    impl DeviceOfBytesInMemory {
        fn range_of(
            &self,
            offset: DeviceOffsetInBytes,
            length: usize,
        ) -> Result<std::ops::Range<usize>, BlockDeviceError> {
            let start = usize::try_from(offset.0).expect("用例里的偏移装得进 usize");
            if start + length > self.bytes.len() {
                return Err(BlockDeviceError::OutOfRange {
                    offset,
                    length: u64::try_from(length).expect("长度装得进 u64"),
                    device_size: self.size_in_bytes(),
                });
            }
            Ok(start..start + length)
        }
    }

    impl BlockDevice for DeviceOfBytesInMemory {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, buffer.len())?;
            buffer.copy_from_slice(&self.bytes[range]);
            Ok(())
        }
        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            _durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, bytes.len())?;
            self.bytes[range].copy_from_slice(bytes);
            Ok(())
        }
        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            let range =
                self.range_of(offset, usize::try_from(length).expect("长度装得进 usize"))?;
            self.bytes[range].fill(0);
            Ok(())
        }
        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            PhysicalBlockSizeInBytes(512)
        }
        fn size_in_bytes(&self) -> u64 {
            u64::try_from(self.bytes.len()).expect("盘宽装得进 u64")
        }
    }

    fn packed_unit_of(record_type: u16, record_width: u16, records: &[Vec<u8>]) -> Vec<u8> {
        build_packed_unit(
            PackedIdentity {
                birth_tree: TreeIdentifier(65_536),
                record_type,
                container: 0,
                container_birth: CHECKPOINT_TXG_OF_THE_PUBLISH,
            },
            record_width,
            records,
            CHECKPOINT_TXG_OF_THE_PUBLISH,
            &[1; 16],
            WriteOrder {
                instance: InstanceGeneration(1),
                transaction: 1,
            },
            BirthSequence(0),
        )
    }

    /// 点名 `slot` 上那个单元的点名项，两条位置条目都在盘 0 的这一槽、校验和是 `unit_checksum`。
    fn named_unit_at(slot: u64, unit_class: u8, unit_checksum: u32) -> NamedUnit {
        let location = LocationEntry {
            device: DeviceIdentity(0),
            slot: SlotNumber(slot),
            unit_checksum,
        };
        NamedUnit {
            locations: [location, location],
            unit_class,
            birth_tree: TreeIdentifier(65_536),
            birth_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
            key_tail: [0; 10],
        }
    }

    fn record_naming(ordinal_from_one: u32, named: Vec<NamedUnit>) -> JournalRecord {
        JournalRecord {
            instance: InstanceGeneration(1),
            counter: u64::from(ordinal_from_one),
            checkpoint_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
            transaction: u64::from(ordinal_from_one),
            is_commit: true,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal_from_one),
            place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
            back_chain: 0,
            filesystem_identifier: 0,
            new_tree_table: NodePointer::empty_root(),
            new_mapping_root: NodePointer::empty_root(),
            new_tree_identifier_watermark: 0,
            new_rollback_floor: CheckpointTxg(0),
            new_instance_table: NodePointer::empty_root(),
            new_allocation_record_tree_root: NodePointer::empty_root(),
            new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            named,
        }
    }

    /// 读成的样子写成一行字（用例只比这一行，被测的类型不为用例另加比较）。
    fn description_of(packed_unit: &PackedUnitNamedByAPublishAsRead) -> String {
        match packed_unit {
            PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch => {
                "读不出或校验和对不上".to_string()
            }
            PackedUnitNamedByAPublishAsRead::ReadButUnparsable => "读得出、解不开".to_string(),
            PackedUnitNamedByAPublishAsRead::Parsed {
                record_type,
                records,
            } => format!("解开了：类型 {record_type}，记录 {records:?}"),
        }
    }

    /// 读盘一层（正推 `research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md` 第 2 节 X4、X5、X18）：一次发布两条记录，第一条点名一片 inode 叶容器、
    /// 一个数据单元与一片读得出而解不开的打包单元（整单元校验和对得上条目），第二条点名一片校验和对不上条目的 inode 叶容器与一片谱系片。
    /// 交回逐条记录、逐个点名的每一片打包单元（数据单元不算），次序照记录与点名的次序，四种结局各认得出：只看第一条记录、只看第一片、
    /// 解不开的当解开了（旧写法 `.ok()` 吞掉）、不筛类标签、记录类型或记录读丢了，这一句都红。
    #[test]
    fn every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart(
    ) {
        let unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
        let slots_per_unit = DATA_UNIT_BYTES / SLOT_BYTES;
        let inode_record = vec![7u8; 16];
        let inode_unit = packed_unit_of(PACKED_TYPE_INODE, 16, std::slice::from_ref(&inode_record));
        let lineage_record = LineageRecord {
            rollback_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
            rollback_target_txg: CheckpointTxg(5),
            inode_number_watermark_of_the_version_rolled_back_from: 3,
        }
        .to_bytes();
        let lineage_unit = packed_unit_of(
            PACKED_TYPE_LINEAGE,
            u16::try_from(LINEAGE_RECORD_BYTES).expect("24"),
            std::slice::from_ref(&lineage_record),
        );
        let unparsable_unit = vec![0xA5u8; unit_bytes];
        assert!(
            crate::unit::parse_packed_unit(&unparsable_unit).is_err(),
            "那一片解不开（阳性对照）"
        );
        let slot_of_the_inode_unit = 0;
        let slot_of_the_unparsable_unit = slots_per_unit;
        let slot_of_the_mismatching_unit = 2 * slots_per_unit;
        let slot_of_the_lineage_unit = 3 * slots_per_unit;
        let slot_of_the_data_unit = 4 * slots_per_unit;
        let mut device = DeviceOfBytesInMemory {
            bytes: vec![0; usize::try_from(SLOTS_OF_THE_DEVICE * SLOT_BYTES).expect("盘宽")],
        };
        for (slot, bytes) in [
            (slot_of_the_inode_unit, &inode_unit),
            (slot_of_the_unparsable_unit, &unparsable_unit),
            (slot_of_the_mismatching_unit, &inode_unit),
            (slot_of_the_lineage_unit, &lineage_unit),
        ] {
            device
                .write_at(
                    SlotNumber(slot).to_device_offset(),
                    bytes,
                    WriteDurability::Plain,
                )
                .expect("写在盘宽之内");
        }
        let devices = vec![(DeviceIdentity(0), device)];
        let first_record = record_naming(
            1,
            vec![
                named_unit_at(
                    slot_of_the_inode_unit,
                    UNIT_CLASS_PACKED,
                    crc32_castagnoli(&inode_unit),
                ),
                named_unit_at(slot_of_the_data_unit, UNIT_CLASS_DATA, 0),
                named_unit_at(
                    slot_of_the_unparsable_unit,
                    UNIT_CLASS_PACKED,
                    crc32_castagnoli(&unparsable_unit),
                ),
            ],
        );
        let second_record = record_naming(
            2,
            vec![
                named_unit_at(
                    slot_of_the_mismatching_unit,
                    UNIT_CLASS_PACKED,
                    crc32_castagnoli(&inode_unit) ^ 1,
                ),
                named_unit_at(
                    slot_of_the_lineage_unit,
                    UNIT_CLASS_PACKED,
                    crc32_castagnoli(&lineage_unit),
                ),
            ],
        );
        let descriptions: Vec<String> = packed_units_named_by_the_records_of_a_publish_as_read(
            &devices,
            &[&first_record, &second_record],
        )
        .iter()
        .map(description_of)
        .collect();
        assert_eq!(
            descriptions,
            vec![
                description_of(&PackedUnitNamedByAPublishAsRead::Parsed {
                    record_type: PACKED_TYPE_INODE,
                    records: vec![inode_record],
                }),
                description_of(&PackedUnitNamedByAPublishAsRead::ReadButUnparsable),
                description_of(&PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch),
                description_of(&PackedUnitNamedByAPublishAsRead::Parsed {
                    record_type: PACKED_TYPE_LINEAGE,
                    records: vec![lineage_record],
                }),
            ],
            "两条记录点名的四片打包单元逐个读成各自的样子、次序照记录与点名"
        );
    }
}

/// 管理员回退取 inode 号水位那一段（Opt2，D23（journal 的角色与格式） 已定项 14「水位」一格）的构造态单测：环里那一项在走得到的历史上
/// 不大于 cur 内存里的，去掉它回退用例与随机历史全绿（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第二节 K1、第三节 F3），
/// 这里直接交一份环里那条根的水位高于 cur 的输入。
#[cfg(test)]
mod inode_number_watermark_of_the_rollback_tests {
    use std::collections::BTreeMap;

    use super::inode_number_watermark_of_the_rollback;
    use crate::address::{CheckpointTxg, InstanceGeneration};
    use crate::instance_table::{InstanceRow, InstanceTableRecords};
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};

    /// cur 那一版内存里的 inode 号水位。
    const INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION: u64 = 40;
    /// 环里那条根（实例 1、txg 7）那一版的 inode 号水位：高于 cur 的。
    const INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION: u64 = 90;
    /// 环里另一条根（实例 1、txg 6）那一版的水位：低于 cur 的。
    const INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION: u64 = 25;
    const INSTANCE_OF_THE_RING_ROOTS: InstanceGeneration = InstanceGeneration(1);
    const TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION: CheckpointTxg = CheckpointTxg(7);
    const TXG_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION: CheckpointTxg = CheckpointTxg(6);

    fn ring_root(checkpoint_txg: CheckpointTxg) -> RootRecord {
        RootRecord {
            filesystem_identifier: [7; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: INSTANCE_OF_THE_RING_ROOTS,
            checkpoint_txg,
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 65_549,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
        }
    }

    /// cur 的实例表：实例 1 那一行的所选根 txg 是 `selected_root_txg`（txg 大于它的实例 1 的根被崩溃恢复抛弃）。
    fn current_table_selecting(selected_root_txg: CheckpointTxg) -> InstanceTableRecords {
        InstanceTableRecords {
            rows: vec![InstanceRow {
                instance: INSTANCE_OF_THE_RING_ROOTS,
                selected_root_txg,
                applied_transaction_high_water: 0,
            }],
        }
    }

    /// 替盘上重建：交进来的每条根那一版的水位（按 txg 查），取最大；一条都没交进来是 `None`。
    fn highest_watermark_of_the_roots_handed_in(roots: &[RootRecord]) -> Option<u64> {
        let watermark_of_each_txg = BTreeMap::from([
            (
                TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
                INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
            ),
            (
                TXG_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION,
                INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION,
            ),
        ]);
        roots
            .iter()
            .map(|root| {
                *watermark_of_each_txg
                    .get(&root.checkpoint_txg)
                    .expect("只交这几条构造出来的根")
            })
            .max()
    }

    /// 环里一条没被抛弃的根（实例表里实例 1 选到 txg 9，txg 7 不大于它）那一版的水位 90 高于 cur 的 40：取 90。
    #[test]
    fn a_ring_root_not_abandoned_whose_watermark_is_above_the_current_version_sets_the_watermark() {
        assert_eq!(
            inode_number_watermark_of_the_rollback(
                &[ring_root(TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION)],
                &current_table_selecting(CheckpointTxg(9)),
                INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
                highest_watermark_of_the_roots_handed_in,
            ),
            INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
            "环里那条根没被抛弃，水位取它与 cur 的最大"
        );
    }

    /// 同一条根，实例表里实例 1 只选到 txg 5（txg 7 > 5，被崩溃恢复抛弃）：它那一版的水位不算，取 cur 的 40。
    #[test]
    fn a_ring_root_abandoned_by_the_instance_table_does_not_set_the_watermark() {
        assert_eq!(
            inode_number_watermark_of_the_rollback(
                &[ring_root(TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION)],
                &current_table_selecting(CheckpointTxg(5)),
                INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
                highest_watermark_of_the_roots_handed_in,
            ),
            INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
            "被崩溃恢复抛弃的根那一版发过的号本来就会重发，水位不跳到它"
        );
    }

    /// 环里没被抛弃的根那一版的水位 25 低于 cur 的 40：cur 那一项不能去（只取环，回退那次就重发号，C143），取 40。
    #[test]
    fn a_ring_root_whose_watermark_is_below_the_current_version_leaves_the_current_watermark() {
        assert_eq!(
            inode_number_watermark_of_the_rollback(
                &[ring_root(TXG_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION)],
                &current_table_selecting(CheckpointTxg(9)),
                INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
                highest_watermark_of_the_roots_handed_in,
            ),
            INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
            "水位取环与 cur 的最大，不只取环"
        );
    }
}

/// 管理员回退取树 ID 水位那一段（D23（journal 的角色与格式） 已定项 14「水位」一格：树 ID 水位照旧取环里读得出的全部根）的构造态单测：
/// 回退用例在走得到的历史上分不出「全部根」与「只取没被崩溃恢复抛弃的根」（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`
/// 第二节 K3、第三节 G5），这里直接交一条按 cur 的实例表被抛弃、树 ID 水位高于 cur 的根。
#[cfg(test)]
mod tree_identifier_watermark_of_the_rollback_tests {
    use super::{abandoned_by_table, tree_identifier_watermark_of_the_rollback};
    use crate::address::{CheckpointTxg, InstanceGeneration};
    use crate::instance_table::{InstanceRow, InstanceTableRecords};
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};

    /// cur 那一版内存里的树 ID 水位。
    const TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION: u64 = 65_549;
    /// 环里被抛弃的那条根（实例 1、txg 7）的树 ID 水位：高于 cur 的。
    const TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_ABOVE_THE_CURRENT_VERSION: u64 =
        65_560;
    /// 环里一条没被抛弃的根（实例 1、txg 6）的树 ID 水位：低于 cur 的。
    const TREE_IDENTIFIER_WATERMARK_OF_THE_RING_ROOT_NOT_ABANDONED_BELOW_THE_CURRENT_VERSION: u64 =
        65_540;
    /// 环里被抛弃的另一条根（实例 1、txg 8）的树 ID 水位：低于 cur 的。
    const TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_BELOW_THE_CURRENT_VERSION: u64 =
        65_545;
    const INSTANCE_OF_THE_RING_ROOTS: InstanceGeneration = InstanceGeneration(1);
    /// cur 的实例表里实例 1 那一行的所选根 txg：txg 大于它的实例 1 的根被崩溃恢复抛弃。
    const SELECTED_ROOT_TXG_OF_THE_CURRENT_TABLE: CheckpointTxg = CheckpointTxg(6);

    fn ring_root(checkpoint_txg: CheckpointTxg, tree_identifier_watermark: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: [7; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: INSTANCE_OF_THE_RING_ROOTS,
            checkpoint_txg,
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
        }
    }

    fn current_table() -> InstanceTableRecords {
        InstanceTableRecords {
            rows: vec![InstanceRow {
                instance: INSTANCE_OF_THE_RING_ROOTS,
                selected_root_txg: SELECTED_ROOT_TXG_OF_THE_CURRENT_TABLE,
                applied_transaction_high_water: 0,
            }],
        }
    }

    /// 环里一条没被抛弃的根（txg 6，水位 65540，低于 cur）与一条按 cur 的实例表被抛弃的根（txg 7 > 6，水位 65560，高于 cur 的 65549）：
    /// 取 65560——被抛弃的那条照旧算。
    #[test]
    fn an_abandoned_ring_root_whose_tree_identifier_watermark_is_above_the_current_version_sets_the_watermark(
    ) {
        let abandoned_root = ring_root(
            CheckpointTxg(7),
            TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
        );
        let root_not_abandoned = ring_root(
            CheckpointTxg(6),
            TREE_IDENTIFIER_WATERMARK_OF_THE_RING_ROOT_NOT_ABANDONED_BELOW_THE_CURRENT_VERSION,
        );
        let current_table = current_table();
        assert!(
            abandoned_by_table(&abandoned_root, &current_table)
                && !abandoned_by_table(&root_not_abandoned, &current_table),
            "前提：txg 7 那条按 cur 的实例表被抛弃，txg 6 那条没有"
        );
        assert_eq!(
            tree_identifier_watermark_of_the_rollback(
                &[root_not_abandoned, abandoned_root],
                &current_table,
                TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION,
            ),
            TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
            "树 ID 水位取环里读得出的全部根：被崩溃恢复抛弃的那条照旧算"
        );
    }

    /// 环里两条根（txg 6 没被抛弃、水位 65540；txg 8 被抛弃、水位 65545）都低于 cur 的 65549：取 cur 的。
    #[test]
    fn ring_roots_all_below_the_current_version_leave_the_current_tree_identifier_watermark() {
        let current_table = current_table();
        assert_eq!(
            tree_identifier_watermark_of_the_rollback(
                &[
                    ring_root(
                        CheckpointTxg(6),
                        TREE_IDENTIFIER_WATERMARK_OF_THE_RING_ROOT_NOT_ABANDONED_BELOW_THE_CURRENT_VERSION,
                    ),
                    ring_root(
                        CheckpointTxg(8),
                        TREE_IDENTIFIER_WATERMARK_OF_THE_ABANDONED_RING_ROOT_BELOW_THE_CURRENT_VERSION,
                    ),
                ],
                &current_table,
                TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION,
            ),
            TREE_IDENTIFIER_WATERMARK_OF_THE_CURRENT_VERSION,
            "水位取环与 cur 的最大，不只取环"
        );
    }
}
