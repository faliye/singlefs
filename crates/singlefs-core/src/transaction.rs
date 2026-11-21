//! 事务层（里程碑步 3 / 步 4 / 步 5）：一个共享的提交状态机 + 一个封闭的提交步骤枚举
//! （D17（实现分层与第三方管道） 已定项 2；`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
//! 暖机与发布的每一步都从同一个枚举走；取号只有屏障经它，取号的系统配置槽写走私有的 `PoolWriter::write_system_configuration_slot`：
//! 取号 = 屏障 → 逐盘一次系统配置槽写 → 屏障（D23（journal 的角色与格式） 已定项 16，C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案；
//! 写之前那道是代码审阅第 19 条加的）；
//! 暖机（D16（发布语义） 已定项 8）= 屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障；
//! 新池新建文件（D16（发布语义） 已定项 7）= 单元写 × 12（默认几何）→ 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障。
//! 覆盖写（里程碑「覆盖写、释放、回退与复用」步 1 / 步 2）走同一条骨架：新数据单元 COW 到新落点、十个提交内生块与树表 COW 出新版本，
//! 被换下的十二个单元在同一次发布里释放（分配记录改写成已释放 + 释放代，条目不删，D3（空间分配） 已定项 7）。
//! 每一步都经过块设备接口，录制器挂在那层（D17（实现分层与第三方管道） 已定项 5）。

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, DELETED_INODES_ENTRY_BYTES, EXTENT_KEY_BYTES, INODE_INTERNAL_ENTRY,
    INODE_RECORD_BYTES, INSTANCE_ROW_BYTES, JOURNAL_NAMED_ENTRIES_PER_RECORD, LINEAGE_RECORD_BYTES,
    LOGGED_OPERATIONS_ENTRY_BYTES, MAPPING_ENTRY_BYTES, RETIREMENT_RECORD_BYTES, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, TOMBSTONE_RECORD_BYTES, TREE_IDENTIFIER_ACCOUNTING,
    TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
    TREE_IDENTIFIER_DELETED_INODES, TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE,
    TREE_IDENTIFIER_LINEAGE, TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_LOGGED_OPERATIONS,
    TREE_IDENTIFIER_RETIREMENT, TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_TOMBSTONE,
    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
    TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
};

use crate::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes,
    InodeNumber, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use crate::admission::{
    admission_reading_before_a_publish_on_a_ring_of, admit_on_every_device,
    admit_the_empty_cluster_segments_floor_on_every_device,
    admit_the_units_landing_on_every_device,
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities,
    committed_reservation_for_tombstones_and_retirement_records,
    demand_of_the_roles_on_each_device, publish_has_an_ordinary_allocation,
    units_of_a_publish_to_land, worst_case_occupancy_of_a_compaction_batch,
    AdmissionRefusedOnSomeDevices, BytesSummedOverAllReplicas,
    EmptyClusterSegmentsBelowTheFloorOnSomeDevices, EmptyClusterSegmentsConjunctReach,
    InodeRecordCounts, PlacementRoomOnOneDevice, ReplicaCount, SpaceAdmission,
    TombstoneAndRetirementPrepayBasis,
};
use crate::allocation_record_tree::{
    build_allocation_record_tree_node, children_of, nodes_holding_records,
    nodes_whose_contents_changed, records_of_each_leaf, AllocationRecordTreeGeometry,
    AllocationRecordTreeNode, AllocationRecordTreeNodeContents, AllocationRecordTreeNodeOrigin,
    AllocationRecordTreeNodePosition, AllocationRecordTreePlan, AllocationRecordTreeVersion,
};
use crate::allocator::{
    AllocationRecord, AllocationRecordTreeLeafIndex, Placement, PlacementRefusal, PoolAllocator,
    RewriteSetLeaves, UnitFootprint,
};
use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
use crate::checksum::crc32_castagnoli;
use crate::code_two_tree::{
    build_internal_entry, internal_entry_width_in_bytes, leaf_position_routed_to,
    plan_the_tree_after_this_publish, plan_the_tree_after_this_publish_down_to_a_zero_root,
    CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodeCapacity, CodeTwoTreeNodeContents,
    CodeTwoTreeNodeOrigin, CodeTwoTreeNodePosition, CodeTwoTreePlan, CodeTwoTreeReadExpectation,
    CodeTwoTreeRefusal, CodeTwoTreeShape, CodeTwoTreeShapeNode, CodeTwoTreeVersion,
};
use crate::compaction_plan::{
    region_is_empty_by_the_records, PlannedRelocation, RelocationDestinationPolicy,
    RelocationPublishOrder,
};
use crate::container_tree::{
    ContainerTreeAfterThisPublish, ContainerTreeCapacities, ContainerTreeNodeOrigin,
    ContainerTreeNodePosition, ContainerTreeShape, NextContainerNumberPastTheTopOfItsRange,
    RecordContainer, IDENTITY_OF_A_CODE_TWO_CHILD,
};
use crate::deleted_inodes::{
    deleted_inodes_root_entries, DeletedInodesQueueRefusal, DELETED_INODES_KEY_WIDTH_IN_BYTES,
};
use crate::extent_tree::{
    build_lower_internal_node, build_lower_leaf, build_upper_internal_node, build_upper_leaf,
    lower_children_of_a_file_without_holes, lower_segment_nodes_of_a_file_without_holes,
    plan_the_extent_upper_segment, ExtentLowerNodePosition, ExtentTreeNodeIdentity,
    ExtentTreeVersion, ExtentUpperLeafEntry, ExtentUpperLeafTarget, ExtentUpperNodePosition,
    TailOfAFile,
};
use crate::inode_tree::{
    remove_record_from_leaf_containers, write_records_into_leaf_containers, InodeLeafContainer,
    InodeLeafContainerIndexInTree, InodeLeafContainerOrigin, InodeLeafContainersAfterThisPublish,
    InodeTreeWriteRefusal, INODE_KEY_WIDTH_IN_BYTES,
};
use crate::instance_table::{
    instance_table_page_records, instance_table_pages_for_rows, instance_table_rows_of_each_page,
    InstanceRow, InstanceTableChainRecord, InstanceTablePageIndex,
};
use crate::journal::{
    back_chain_of, record_offset, JournalBytesToWrite, JournalRecord,
    JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, NamedUnit,
};
use crate::lineage_tree::{
    append_lineage_record_to_the_open_container, LineageContainer, LineageContainerIndexInTree,
    LineageTreeAfterThisPublish,
};
use crate::logged_operations::{
    live_compaction_intent_of_the_entries, live_truncation_intent_of_the_entries,
    logged_operations_root_entries, CompactionIntent, CompactionRegion, LoggedOperation,
    LoggedOperationEntry, LoggedOperationKind, TruncationIntent,
    LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
};
use crate::make_filesystem::{
    location_entries, MakeFilesystemParameters, MKFS_INSTANCE_GENERATION, TREE_TABLE_KEY_WIDTH,
};
use crate::pointer::{
    slot_shared_by_both_location_entries, BirthSequence, DataPointer,
    LocationEntriesOnDifferentSlots, LocationEntry, NodePointer, PointerHead,
};
use crate::records::{
    birth_txg_of_a_mapping_key, DeadlistEntry, SnapshotDestructionMark, DEADLIST_ENTRY_KEY_BYTES,
};
use crate::records::{
    build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
    mapping_key_for_data, mapping_key_for_node, node_key_tail, parse_mapping_entry,
    AccountingEntry, InodeRecord, InodeRecordTruncationField, LineageRecord, RetirementRecord,
    TombstoneRecord, TreeTableEntry, ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
    NLINK_OF_AN_INODE_BEING_DELETED, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
    SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, STATISTIC_ALLOCATED_BYTES,
    STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
    STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
    STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETE_BYTES,
    STATISTIC_UNRECLAIMABLE_BYTES, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
    TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE,
    TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
    TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT, TREE_KIND_SPARSE_SIDE_TABLE,
    TREE_KIND_TOMBSTONE,
};
use crate::recovery::{
    effective_rollback_floor_rereading_unreadable_reads_once, highest_root_instance,
    instance_table_chain_of_root, snapshot_tree_table_entries_paired_one_to_one,
    system_configuration_slots_of_one_device_read_once,
    system_configuration_witness_is_undecidable, tree_table_entry_count,
    verified_system_configuration_slots, PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
    PoolReaderOfOneReadPass, PoolReaderStraightFromTheDisk, RecoveryFailure,
    SnapshotTreeTableEntries, SystemConfigurationSlotsOfOneDevice,
};
use crate::retirement_tree::{
    append_retirement_records_to_the_open_container, retirement_records_of_this_publish,
    RetirementContainer, RetirementContainerIndexInTree, RetirementTreeAfterThisPublish,
};
use crate::root_record::{RootRecord, UnmountMarker};
use crate::root_ring::{slot_offset, target_for_publish};
use crate::system_configuration::{
    journal_in_flight_record_limit, SystemConfiguration, SystemImmutableConfiguration,
    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
};
use crate::tombstone_tree::{
    append_tombstone_to_the_open_container, TombstoneContainer, TombstoneContainerIndexInTree,
    TombstoneTreeAfterThisPublish, CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES,
};
use crate::truncate::{
    declared_length_within_a_size, next_truncation_step, plan_the_extent_tree_after_a_size_change,
    plan_the_lower_segment_after_a_size_change, tail_data_unit_rewrite_of_a_truncation,
    tail_data_unit_rewrite_of_an_extension, target_size_rounded_up_to_a_whole_data_unit_fits,
    truncation_field_after_the_truncation_intent, DataUnitsRewrittenInOnePublish,
    SizeChangeOfARequest, SizeChangeOfTheFile, TailDataUnitRewrite, TruncationStep,
};
use crate::unit::{
    build_data_unit, build_index_node, build_packed_unit, data_unit_payload,
    data_unit_payload_capacity, parse_data_unit, parse_index_node, unit_filesystem_identifier,
    DataUnitIdentity, PackedIdentity, WriteOrder, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
    UNIT_CLASS_PACKED,
};
use crate::unlink::{
    next_unlink_step, plan_the_extent_tree_after_a_deletion_batch,
    plan_the_extent_tree_after_the_final_batch, plan_the_lower_segment_after_a_deletion_batch,
    UnlinkBatchSize, UnlinkStep,
};
use crate::write_accounting::{WritesByStructureKind, WrittenStructureKind};
use crate::write_request_split::{
    split_sequential_write_into_one_unit_transactions, OneUnitTransaction, PartialWriteOfAFile,
};

mod accounting_and_ordinary_allocation;
mod allocation_record_tree_settling;
mod compaction_publishes;
mod extent_tree_and_file_plans;
mod file_data_publishes;
mod instance_acquisition;
mod persisting_a_publish;
mod pool_version_and_transaction_units;
mod pool_writer;
mod publish_admitted;
mod publish_error;
mod publish_plan;
mod publish_plan_resolution;
mod release_of_replaced_units;
mod resolved_publish;
mod size_change_and_partial_write_publishes;
mod snapshot_lifetime_and_deadlist;
mod snapshot_publishes;
mod transaction_output;
mod transaction_unit_building;
mod unlink_and_deletion_queue_publishes;
mod version_mapping_reads;
mod version_publish_and_its_judgement;
mod version_without_file_publishes;

use self::accounting_and_ordinary_allocation::*;
pub use self::allocation_record_tree_settling::*;
pub use self::compaction_publishes::*;
pub use self::extent_tree_and_file_plans::*;
pub use self::file_data_publishes::*;
pub use self::instance_acquisition::*;
pub use self::persisting_a_publish::*;
pub use self::pool_version_and_transaction_units::*;
pub use self::pool_writer::*;
use self::publish_admitted::*;
pub use self::publish_error::*;
pub use self::publish_plan::*;
pub use self::release_of_replaced_units::*;
pub use self::resolved_publish::*;
pub use self::size_change_and_partial_write_publishes::*;
pub use self::snapshot_lifetime_and_deadlist::*;
pub use self::snapshot_publishes::*;
pub use self::transaction_output::*;
pub use self::transaction_unit_building::*;
pub use self::unlink_and_deletion_queue_publishes::*;
pub use self::version_mapping_reads::*;
pub use self::version_publish_and_its_judgement::*;
pub use self::version_without_file_publishes::*;

/// 第一个文件的 inode 号（里程碑步 3 预想）。
pub const FIRST_INODE_NUMBER: u64 = 1;
/// 新池新建文件的事务号；0 保留给不承载事务的记录（D23（journal 的角色与格式） 已定项 19 ①）。
pub const NEW_POOL_FILE_CREATION_NUMBER: u64 = 1;
/// 无归属的树 ID（树表单元自己、mkfs 的固定单元）。
pub const TREE_IDENTIFIER_NONE: u64 = 0;

/// 提交步骤的封闭枚举，五个成员，`match` 不写通配臂——少一个臂编译不过。
/// D17（实现分层与第三方管道） 已定项 2 说的六种步骤种类是录制流一侧的 `StepKind`（`crates/singlefs-verification/src/segments.rs`），
/// 不是这五个成员：多出的整段清零（`StepKind::ZeroFill`）在这里没有对应成员，它是 mkfs 清零根环基址之前那一段、根环与 journal 环时直接发的 `write_zeroes_at`，不经提交步骤。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitStep<'publish> {
    WriteUnitToEveryDevice {
        slot: SlotNumber,
        unit: &'publish [u8],
        /// 这个单元的角色：只供按结构种类记账（增补 1），不影响写到哪、怎么写。
        identity: TransactionUnit,
    },
    WriteJournalRecordToEveryDevice {
        counter: u64,
        record: &'publish [u8],
    },
    /// 根槽 FUA 写：落区域 `txg mod 3` 的槽 `(txg div 3) mod 8`，写在那个区域归属的盘上。
    WriteRootRecordForceUnitAccess {
        checkpoint_txg: CheckpointTxg,
        root_slot: &'publish [u8],
    },
    /// 每盘一次系统配置槽原地覆写：世代号 = 这块盘两槽里自证过的最大世代号 + 1，槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16，逐盘计）。
    RotateSystemConfigurationSlots {
        journal_tail: u64,
        journal_instance: InstanceGeneration,
    },
    Barrier,
}

/// [`TransactionOutput::unit`] 与 [`FirstEntryOfEachIdentity::unit`] 查不到时的 panic 消息：两条查法查的是同一张 `units`。
const EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION: &str =
    "八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在";

/// 七·6 按身份查在产品里实际做了多少事（判决 `research/prompts/m4-publish-lookup-code-r2-main-verification.md` 第三节）：
/// 在索引上查了几次、每次看过索引里几项、建了几次索引、解了几次中央映射叶，按点名的几处（[`lookup_work::LookupSite`]）分开记。
/// 数的是产品调用点上的次数，不是测试自己另建的索引；测试读它判增长与「每片叶至多解一次」，不读挂钟与 CPU 时间。
///
/// 只在特性 `lookup-work-counters` 下记（验证包的 dev 依赖打开它）：不开时 [`lookup_work::entering`] 与 [`lookup_work::count`]
/// 什么都不做，发布构建不多出开销。计数记在本线程上：核心层不起线程，一次发布或一趟整理的查找都在调它的那个线程里做；
/// 测试的各条用例各在自己的线程里跑，互不串数。
pub mod lookup_work {
    #[cfg(feature = "lookup-work-counters")]
    use std::cell::RefCell;
    #[cfg(feature = "lookup-work-counters")]
    use std::collections::BTreeMap;

    /// 本包构建脚本这一次交给依赖方的特性：`build.rs` 打的 `cargo::metadata=enabled_features=` 那一行的值，同一个值经
    /// `cargo::rustc-env` 编进本包。依赖方读到的就是它，测试拿它判「cargo 真开着这个特性时构建脚本报开着」
    /// （七·6 后续批代码轮第二轮判决第三节第 7 条：构建脚本里那一格 cfg 写死成假时，依赖方当它没开，别的检查都看不见）。
    pub const ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT: &str =
        env!("SINGLEFS_CORE_ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT");

    /// 七·6 按身份查的几处里，测试要分开判的那几处。封闭集合；不在其中任何一处里做的查找记在「不在点名的几处里」那一格（`None`）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum LookupSite {
        /// 快照判定那一圈：换下的用户可见角色各取一次映射 key，进 deadlist 的再取映射条目（`deadlist_changes_of_this_publish`）。
        SnapshotJudgement,
        /// 经映射释放的角色取映射条目，释放判定路径那一遍（`placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots`）。
        /// 释放的三处各记各的（七·6 代码轮第三轮判决第三节第 2 条）：一处改回线性，另两处补不上它的数。
        MappingEntriesOfTheReleaseJudgement,
        /// 经映射释放的角色取映射条目，核两条位置项各指池里一块不同的盘那一遍（`refuse_mapping_entries_that_do_not_name_two_pool_devices`）。
        MappingEntriesCheckedForTwoPoolDevices,
        /// 经映射释放的角色取映射条目，释放之前读盘核校验和那一遍（`copies_failing_the_release_checksum_check`）。
        MappingEntriesCheckedAgainstTheReleaseChecksums,
        /// 删快照的批放掉的 deadlist 条目按映射 key 取它留存的映射条目（`deadlist_changes_of_this_publish`）。
        KeptMappingEntriesReleasedByTheSnapshotDestruction,
        /// 释放判定路径那一遍，换下的第一个文件之外那一族角色在上一版里有没有落点：按 inode 号找上一版的那个文件
        /// （`has_a_placement_in_the_previous_version`）。每问一个角色进一次。
        FilesOfTheReplacedRolesInThePreviousVersion,
        /// 经映射释放的角色清单（`mapped_roles_released_by_this_publish`：核两条位置项、读盘核校验和、释放那一段与快照判定对账各调它）
        /// 问换下的角色在上一版里有没有落点时按 inode 号找文件。每调一次进一次（七·6 后续批代码轮第一轮判决第三节第 3 条：建索引次数恰好等于走了几遍）。
        FilesOfTheMappedRolesReleasedByThisPublish,
        /// 快照判定那一圈问换下的用户可见角色在上一版里有没有落点时按 inode 号找文件（`deadlist_changes_of_this_publish`）。每走一遍进一次。
        FilesOfTheUserVisibleRolesOfTheSnapshotJudgement,
        /// 释放判定路径那一遍，换下的角色在不在留给快照的角色清单里：清单建一次按角色的索引、每个角色查一次
        /// （`placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots`；七·6 后续批，用户 2026-10-22 定并进）。每走一遍进一次。
        RolesKeptForTheSnapshotsInTheReleaseJudgement,
        /// 经映射释放的角色清单那一处，换下的角色在不在留给快照的角色清单里（`mapped_roles_released_by_this_publish`）。每调一次进一次。
        RolesKeptForTheSnapshotsAmongTheMappedRolesReleased,
        /// 记录点名的角色清单：这次重写的角色在不在最后一条之前那几条已经点名的数据角色里（`roles_named_by_each_record_of_the_publish`）。
        /// 每调一次进一次。
        DataRolesNamedBeforeTheLastRecord,
        /// 一版里进映射的角色换成映射位置时，按角色找它在 `units` 里的那一项（`take_the_mapping_positions_over_the_location_hints`：
        /// 整理发批与恢复重建一版都调它；七·6 后续批，用户 2026-10-22 定并进）。每调一次至多进一次（一把有位置的 key 都没有的不进、不建）。
        UnitsOfTheMappedRolesTakingTheirMappingPositions,
        /// 装一次发布要写的单元：这次重写的每个角色按角色取它在这一版 `units` 里的单元（`publish_admitted` 的 `written_units`）。
        /// 这一版 `units` 按角色的索引在这里建，每次装进一次。
        UnitsOfTheRolesWrittenByThisPublish,
        /// 装一次发布的记录：点名的每个角色按角色取它在这一版 `units` 里的单元（`publish_admitted` 的 `named_unit_of`），
        /// 查的是 [`LookupSite::UnitsOfTheRolesWrittenByThisPublish`] 那里建的同一份索引。每点名一个角色进一次。
        UnitsOfTheRolesNamedByTheRecords,
        /// 照抄第一个文件的发布装这一版 `units` 时，第一个文件的每个数据单元问它在不在这次重写的角色里（`publish_admitted`）：
        /// 这次重写的角色清单建一次按角色的索引、每个数据单元查一次（七·6 后续批代码轮第二轮判决第五节，用户 2026-10-23 定并进）。
        /// 每装一次照抄第一个文件的进一次；第一个文件有新版本的发布不进、不建。
        RewrittenRolesAmongTheDataUnitsOfTheCarriedFirstFile,
        /// 恢复重建一版时数据单元照 inode 记录核，按 inode 号找记录（`recovery::data_units_judged_against_the_inode_records`）：
        /// 全部 inode 叶容器的记录建一次按 inode 号的索引，第一个文件与第一个文件之外每个有数据单元的文件各查一次
        /// （七·6 后续批代码轮第二轮判决第五节，用户 2026-10-23 定并进）。每调一次进一次。
        InodeRecordsOfTheFilesJudgedByTheRecovery,
        /// 发布照抄 extent 树的节点：按角色取上一版那个节点的单元（`build_extent_tree` 的 `carried_node`）。
        /// 整棵 extent 树装的时候都在这一处里（七·6 后续批代码轮第一轮判决第三节第 3 条：照抄那一处的作用域包住建），
        /// 照抄每个节点时再进一次；里面按 inode 号找文件的记在 [`LookupSite::FilesOfTheCarriedExtentLowerNodes`]。
        CarriedExtentTreeNodes,
        /// 发布照抄第一个文件之外一个文件的下段节点：按 inode 号找上一版的那个文件、取那个节点的指针（`build_extent_tree`）。
        FilesOfTheCarriedExtentLowerNodes,
        /// 发布照抄记账树的节点：按角色取上一版那个节点的单元（`build_multi_level_tree`）。三棵多层码 2 树按树分开记
        /// （七·6 后续批代码轮第一轮判决第三节第 3 条）：一棵退回线性找，另两棵补不上它的数。
        CarriedNodesOfTheAccountingTree,
        /// 发布照抄 deadlist 树的节点（同 [`LookupSite::CarriedNodesOfTheAccountingTree`]）。
        CarriedNodesOfTheDeadlistTree,
        /// 发布照抄中央映射树的节点（同 [`LookupSite::CarriedNodesOfTheAccountingTree`]）。
        CarriedNodesOfTheCentralMappingTree,
        /// 整理读一版取全部进映射单元的映射条目（`mounted_session::central_mapping_as_the_compaction_reads_it`）。
        CompactionReadsTheCentralMapping,
        /// 整理发批核计划里每一项的源落点：按映射 key 找角色、取映射条目，找不到的按 key 找留存的映射条目，
        /// 角色的单元按角色取字节（`relocation_checked_against_the_version`）。
        RelocationSources,
    }

    /// 一处（或几处合起来）做的查找。
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct LookupWork {
        /// 建了几次按身份的索引（[`super::FirstEntryOfEachIdentity::of_the_table`]）。
        pub index_builds: u64,
        /// 在索引上按身份查了几次。
        pub index_lookups: u64,
        /// 查与走索引一共看过索引里几项：有序查一次至多看「索引项数的二进制位数」那么多项；在索引上逐项走，走过一项算一项。
        pub index_entries_examined: u64,
        /// 查过的索引里最大那一份有几项（判「每次查至多看几项」用）。
        pub largest_index_looked_up: u64,
        /// 解了几次中央映射叶：批量读者每解一片算一次，逐 key 查（`central_mapping_locations_in_the_version`）每查到一片叶算一次。
        pub mapping_leaf_decodes: u64,
        /// 进了这一处几次（[`entering`] 每调一次算一次）：每一处在哪一层进写在 [`LookupSite`] 那一成员上，
        /// 按「走一遍进一次」进的那几处，测试拿它判「建索引次数恰好等于走了几遍」。
        pub times_entered: u64,
    }

    impl LookupWork {
        /// 两份合起来：次数相加，最大的索引取两份里大的那一份。
        #[must_use]
        pub fn plus(self, other: Self) -> Self {
            Self {
                index_builds: self.index_builds + other.index_builds,
                index_lookups: self.index_lookups + other.index_lookups,
                index_entries_examined: self.index_entries_examined + other.index_entries_examined,
                largest_index_looked_up: self
                    .largest_index_looked_up
                    .max(other.largest_index_looked_up),
                mapping_leaf_decodes: self.mapping_leaf_decodes + other.mapping_leaf_decodes,
                times_entered: self.times_entered + other.times_entered,
            }
        }
    }

    /// 一段代码在点名的每一处做的查找；不在任何一处里做的记在 `None` 下。
    #[cfg(feature = "lookup-work-counters")]
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct LookupWorkOfEachSite {
        pub work_of_each_site: BTreeMap<Option<LookupSite>, LookupWork>,
    }

    #[cfg(feature = "lookup-work-counters")]
    impl LookupWorkOfEachSite {
        /// `site` 那一处做的；一次都没进过那一处就是全 0。
        #[must_use]
        pub fn of_the_site(&self, site: LookupSite) -> LookupWork {
            self.work_of_each_site
                .get(&Some(site))
                .copied()
                .unwrap_or_default()
        }

        /// 每一处（连同不在点名的几处里的）合起来。
        #[must_use]
        pub fn of_every_site(&self) -> LookupWork {
            self.work_of_each_site
                .values()
                .fold(LookupWork::default(), |sum, work| sum.plus(*work))
        }

        fn add(&mut self, other: &Self) {
            // 迭代上界是 `other` 里的格数（至多点名的几处加一）。
            for (site, work) in &other.work_of_each_site {
                let sum = self.work_of_each_site.entry(*site).or_default();
                *sum = sum.plus(*work);
            }
        }
    }

    /// 本线程上进了哪几处（里层的在后）与到此刻各处的计数。
    #[cfg(feature = "lookup-work-counters")]
    struct LookupWorkOnThisThread {
        sites_entered: Vec<LookupSite>,
        work_of_each_site: LookupWorkOfEachSite,
    }

    #[cfg(feature = "lookup-work-counters")]
    thread_local! {
        static LOOKUP_WORK_ON_THIS_THREAD: RefCell<LookupWorkOnThisThread> = const {
            RefCell::new(LookupWorkOnThisThread {
                sites_entered: Vec::new(),
                work_of_each_site: LookupWorkOfEachSite {
                    work_of_each_site: BTreeMap::new(),
                },
            })
        };
    }

    /// 跑 `action`，交回它的结果与它在本线程上做的查找。量的时候先把此前的计数挪开、量完加回去：套着量也各量各的。
    #[cfg(feature = "lookup-work-counters")]
    pub fn lookup_work_while<Output>(
        action: impl FnOnce() -> Output,
    ) -> (Output, LookupWorkOfEachSite) {
        let counted_before = LOOKUP_WORK_ON_THIS_THREAD
            .with(|state| std::mem::take(&mut state.borrow_mut().work_of_each_site));
        let output = action();
        let counted_while = LOOKUP_WORK_ON_THIS_THREAD.with(|state| {
            let mut state = state.borrow_mut();
            let counted_while = std::mem::replace(&mut state.work_of_each_site, counted_before);
            state.work_of_each_site.add(&counted_while);
            counted_while
        });
        (output, counted_while)
    }

    /// 进了点名的一处：活着的时候本线程的查找记在这一处，丢掉时退出（里层的先退）。
    #[must_use = "进了一处要把它留到那一处的查找做完：丢掉就退出了"]
    pub(crate) struct InsideALookupSite {
        _constructed_only_by_entering: (),
    }

    /// 进 `site` 那一处，直到交回的值被丢掉；那一处进的次数加一（[`LookupWork::times_entered`]）。
    pub(crate) fn entering(site: LookupSite) -> InsideALookupSite {
        #[cfg(feature = "lookup-work-counters")]
        LOOKUP_WORK_ON_THIS_THREAD.with(|state| state.borrow_mut().sites_entered.push(site));
        count(|work| work.times_entered += 1);
        #[cfg(not(feature = "lookup-work-counters"))]
        let _ = site;
        InsideALookupSite {
            _constructed_only_by_entering: (),
        }
    }

    // 不开特性时也实现 `Drop`（什么都不做）：调用方在一处的查找做完时 `drop` 它退出，两种编法写法相同。
    impl Drop for InsideALookupSite {
        fn drop(&mut self) {
            #[cfg(feature = "lookup-work-counters")]
            LOOKUP_WORK_ON_THIS_THREAD.with(|state| {
                state.borrow_mut().sites_entered.pop().expect(
                    "每个 InsideALookupSite 都由 entering 进过一处：退出时栈里至少有它那一处",
                );
            });
        }
    }

    /// 把 `update` 记到本线程此刻所在的那一处（最里层的；不在任何一处里记在 `None` 下）。
    pub(crate) fn count(update: impl FnOnce(&mut LookupWork)) {
        #[cfg(feature = "lookup-work-counters")]
        LOOKUP_WORK_ON_THIS_THREAD.with(|state| {
            let mut state = state.borrow_mut();
            let site = state.sites_entered.last().copied();
            update(
                state
                    .work_of_each_site
                    .work_of_each_site
                    .entry(site)
                    .or_default(),
            );
        });
        #[cfg(not(feature = "lookup-work-counters"))]
        let _ = update;
    }
}

/// 按身份排好序、每个身份只留一项的索引存放：查与走都经这里，各自记看过几项（[`lookup_work`]）。
/// 放在自己的模块里，外面拿不到那张排好序的表，只能经 `value_stored_for_identity` 与 `iter`：按身份查与逐项走都逃不过计数。
mod sorted_identity_index {
    use super::lookup_work::count;

    pub(super) struct SortedIdentityIndex<Identity, Value> {
        /// 按身份严格升序，没有两项身份相同。
        sorted: Vec<(Identity, Value)>,
    }

    impl<Identity: Ord, Value> SortedIdentityIndex<Identity, Value> {
        /// `pairs` 按身份稳定排序（同一身份的几项保持进来的次序），同一身份只留进来最先的那一项。
        pub(super) fn keeping_the_first_value_of_each_identity(
            mut pairs: Vec<(Identity, Value)>,
        ) -> Self {
            pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
            // `dedup_by` 拿后一项与前一项比，判相同就丢后一项：留下的是稳定排序后每个身份的最先那一项。
            pairs.dedup_by(|(later, _), (earlier, _)| later == earlier);
            count(|work| work.index_builds += 1);
            Self { sorted: pairs }
        }

        /// 身份是 `identity` 的那一项。折半查：一次至多看 `sorted` 项数的二进制位数那么多项。
        pub(super) fn value_stored_for_identity(&self, identity: &Identity) -> Option<&Value> {
            let mut entries_examined: u64 = 0;
            let mut low = 0;
            let mut high = self.sorted.len();
            // 迭代上界是 `sorted` 项数的二进制位数；跨轮携带的是还没排除的区间 [low, high)，要找的若在表里就在其中。
            let found = loop {
                if low >= high {
                    break None;
                }
                let middle = low + (high - low) / 2;
                entries_examined += 1;
                let (candidate, value) = &self.sorted[middle];
                match candidate.cmp(identity) {
                    std::cmp::Ordering::Less => low = middle + 1,
                    std::cmp::Ordering::Greater => high = middle,
                    std::cmp::Ordering::Equal => break Some(value),
                }
            };
            let index_entries = u64::try_from(self.sorted.len()).expect("索引项数装得进 u64");
            count(|work| {
                work.index_lookups += 1;
                work.index_entries_examined += entries_examined;
                work.largest_index_looked_up = work.largest_index_looked_up.max(index_entries);
            });
            found
        }

        /// 按身份升序逐项走；走过一项算看过一项。
        pub(super) fn walk_in_ascending_identity(
            &self,
        ) -> WalkOfTheSortedIdentityIndex<'_, Identity, Value> {
            WalkOfTheSortedIdentityIndex {
                remaining: self.sorted.iter(),
            }
        }
    }

    /// [`SortedIdentityIndex::iter`]：每交出一项记看过一项（从哪一头走都算）。
    pub(super) struct WalkOfTheSortedIdentityIndex<'index, Identity, Value> {
        remaining: std::slice::Iter<'index, (Identity, Value)>,
    }

    impl<'index, Identity, Value> Iterator for WalkOfTheSortedIdentityIndex<'index, Identity, Value> {
        type Item = (&'index Identity, &'index Value);

        fn next(&mut self) -> Option<Self::Item> {
            let (identity, value) = self.remaining.next()?;
            count(|work| work.index_entries_examined += 1);
            Some((identity, value))
        }
    }

    impl<Identity, Value> DoubleEndedIterator for WalkOfTheSortedIdentityIndex<'_, Identity, Value> {
        fn next_back(&mut self) -> Option<Self::Item> {
            let (identity, value) = self.remaining.next_back()?;
            count(|work| work.index_entries_examined += 1);
            Some((identity, value))
        }
    }
}

use lookup_work::LookupSite;
use sorted_identity_index::SortedIdentityIndex;

/// 一版里一张按身份查的表（`units`、`mapped_units`、`files_other_than_the_first`）的索引：建的时候走一遍整张表、按身份排一次序，
/// 之后每次查是一次折半查找，不再走整张表。发布与整理在一次调用里要对约 P 个身份各查一次，在表上 `iter().find` 是 P 的平方
/// （里程碑四七·6；量的出处 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 2026-10-20 调查员交回那一条）。
///
/// 同一身份在表里出现几次时取最先那一项，与在表上 `iter().find` 逐项相同：换成索引行为一个字节不变（七·6 的验收）。
/// 索引借着那张表（`'table`）：索引活着时那一版改不了，「索引与表一致」由借用检查守住；表换了就要重建，不跨版本留着。
/// 建、查、走各记进 [`lookup_work`]（只在特性 `lookup-work-counters` 下）。
pub struct FirstEntryOfEachIdentity<'table, Identity, Entry> {
    first_entry_of_each_identity: SortedIdentityIndex<Identity, &'table Entry>,
}

impl<'table, Identity: Ord, Entry> FirstEntryOfEachIdentity<'table, Identity, Entry> {
    /// 走一遍 `table`，每个身份记下最先那一项。
    #[must_use]
    pub fn of_the_table(
        table: &'table [Entry],
        identity_of: impl Fn(&'table Entry) -> Identity,
    ) -> Self {
        let mut identities_with_their_entries = Vec::with_capacity(table.len());
        // 迭代上界是表长；按表的次序收：同一身份留最先那一项靠的就是这个次序（`keeping_the_first_value_of_each_identity`）。
        for entry in table {
            identities_with_their_entries.push((identity_of(entry), entry));
        }
        Self {
            first_entry_of_each_identity:
                SortedIdentityIndex::keeping_the_first_value_of_each_identity(
                    identities_with_their_entries,
                ),
        }
    }

    /// 表里身份是 `identity` 的最先那一项；没有就是 `None`（同 `iter().find`）。
    #[must_use]
    pub fn first_entry_of(&self, identity: &Identity) -> Option<&'table Entry> {
        self.first_entry_of_each_identity
            .value_stored_for_identity(identity)
            .copied()
    }

    /// 索引里的每个身份与它在表里的最先那一项，按身份升序：表里每个不同的身份恰好一项。走一项记看过一项。
    pub fn first_entries(&self) -> impl DoubleEndedIterator<Item = (&Identity, &'table Entry)> {
        self.first_entry_of_each_identity
            .walk_in_ascending_identity()
            .map(|(identity, entry)| (identity, *entry))
    }
}

impl<'table> FirstEntryOfEachIdentity<'table, TransactionUnit, PublishedUnit> {
    /// 同 [`TransactionOutput::unit`]：这一版里角色是 `identity` 的那个单元。
    ///
    /// # Panics
    /// 这一版里没有这个角色的单元（同 [`TransactionOutput::unit`]）。
    #[must_use]
    pub fn unit(&self, identity: TransactionUnit) -> &'table PublishedUnit {
        self.first_entry_of(&identity)
            .expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)
    }
}

impl<'table> FirstEntryOfEachIdentity<'table, TransactionUnit, (TransactionUnit, Vec<u8>)> {
    /// 这一版里进映射的角色 `identity` 的映射 key（同在 `mapped_units` 上按角色 `iter().find`）。
    ///
    /// # Panics
    /// 这一版里这个角色不进映射：调用方只拿上一版里进映射的角色来查——照抄进下一版的角色、快照判定那一圈换下的有落点的角色
    /// （上一版装映射节点时逐个记下了）。
    #[must_use]
    pub fn mapping_key_of(&self, identity: TransactionUnit) -> &'table [u8] {
        self.first_entry_of(&identity)
            .map(|(_, key)| key.as_slice())
            .expect("照抄进来的角色在上一版里进映射、有一把 key")
    }
}
