//! 冷启动恢复（里程碑步 6）：择系统配置 → 择根 → 全环扫描 journal → 按前缀口径施加记录的新根段 → 沿树走到数据单元。
//! 只通过 [`PoolReader`] 读盘，不沿进程内的指针热读；崩溃点重放（步 7）拿同一条路径去判每一个崩溃状态。
//!
//! 口径：择系统配置 = 校验和过且世代号大的那个槽、各盘 fsid 与设备数要对得上（D22（单元原子性怎么合成） 已定项 16）；
//! 择根 = 三个区域全部槽里自证过、`(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7）；
//! journal 全环扫描、不先信 tail（D23（journal 的角色与格式） 已定项 3），两份镜像任一份自证过即算在（D23（journal 的角色与格式） 已定项 14）；
//! 前缀 = `(实例代号, checkpoint_txg)` 严格大于所选根、jsn 严格连续、反向链等于本实例内逻辑前一条的头算出来的链值
//! （I-8.6（反向链算法），前一条不在盘上不判）、提交标记齐全、在飞上限之内、点名单元逐项验过，
//! 施加一条记录 = 把所选根的四个字段换成记录新根段里的（D23（journal 的角色与格式） 已定项 15）。

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
    FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR, LINEAGE_RECORD_BYTES,
    LOC_ENTRY, MAPPING_ENTRY_BYTES, NODE_BYTES, RETIREMENT_RECORD_BYTES, ROOT_RECORD_BYTES,
    ROOT_RING_CHUNK_BYTES, ROOT_RING_PRIME_STEP, ROOT_RING_REGIONS, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES, TOMBSTONE_RECORD_BYTES,
};
// 取前缀时验不验点名单元是 `replay_journal` 的入参：调用方经这里拿，验证包不直接依赖取前缀核那个包。
pub use singlefs_replay_prefix::NamedUnitVerification;
use singlefs_replay_prefix::{
    choose_root_slot, first_copy_of_each_record_key, replay_core, CoreFailure, CoreRecord,
    JournalCounter, PrefixKernelCheckpointTxg, PrefixKernelInstanceGeneration,
    PrefixKernelJournalRecordOrdinalWithinPublish, PrefixKernelJournalRecordPlaceInPublish,
    RootChoiceKey, TransactionNumber,
};

use crate::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InodeNumber,
    InstanceGeneration, SlotNumber, TreeIdentifier,
};
use crate::admission::InodeRecordCounts;
use crate::allocation_record_tree::{
    read_allocation_record_tree, AllocationRecordTreeGeometry, AllocationRecordTreeHeaderJudgement,
    AllocationRecordTreeReadFromDisk,
};
use crate::allocator::{
    unit_area_slots_of_device_starting_at, AllocationRecord,
    AllocationRecordTreeOfTheVersionWithoutFile, DeviceEndsBeforeTheUnitAreaStart, UnitAreaStart,
};
use crate::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use crate::checksum::crc32_castagnoli;
use crate::code_two_tree::{
    read_code_two_tree, CodeTwoTreeHeaderJudgement, CodeTwoTreeReadFromDisk, CodeTwoTreeVersion,
};
use crate::container_tree::{
    ContainerTreeNodePosition, ContainerTreeShape, RecordContainer, IDENTITY_OF_A_CODE_TWO_CHILD,
};
use crate::deleted_inodes::inode_numbers_of_the_deleted_inodes_root_entries;
use crate::deleted_inodes::DELETED_INODES_KEY_WIDTH_IN_BYTES;
use crate::extent_tree::TailOfAFile;
use crate::extent_tree::{
    read_extent_tree, ExtentLowerNodePosition, ExtentTreeHeaderJudgement, ExtentTreeReadFromDisk,
    ExtentTreeReading, ExtentTreeVersion, ExtentUpperSegmentReadFromDisk,
    ExtentsOfAFileReadFromDisk,
};
use crate::inode_tree::{
    InodeLeafContainer, InodeLeafContainerIndexInTree, INODE_KEY_WIDTH_IN_BYTES,
};
use crate::instance_table::{
    InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex, InstanceTableRecords,
};
use crate::journal::{
    back_chain_of, record_offset, slot_after_the_journal_ring, JournalRecord,
    JournalRecordPlaceInPublish, FIRST_JOURNAL_COUNTER,
};
use crate::lineage_tree::LineageContainerIndexInTree;
use crate::logged_operations::{
    logged_operation_entries_of_the_root_entries, LoggedOperationEntry,
    LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
};
use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
use crate::pointer::{
    location_entries_ascend_by_device, DataPointer, LocationEntry, NodePointer,
    PointerHeadFieldGroup, PointerHeadFieldOutsideTheFirstVersion, PointerHeadFieldsJudgement,
};
use crate::records::{
    mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry_or_refusal,
    parse_mapping_entry, AccountingEntry, InodeRecord, LineageRecord, PointerCarryingEntryRefusal,
    RetirementRecord, TombstoneRecord, TreeTableEntry, NLINK_OF_AN_INODE_BEING_DELETED,
    SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING,
    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT,
    TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS,
    TREE_KIND_RETIREMENT, TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT,
    TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
};
use crate::records::{DeadlistEntry, SnapshotDestructionMark, DEADLIST_ENTRY_KEY_BYTES};
use crate::retirement_tree::RetirementContainerIndexInTree;
use crate::root_record::RootRecord;
use crate::root_ring::{
    region_start, root_sits_in_the_ring_slot_the_formula_gives, slot_offset, RootRingSlot,
    RootRingSlotsPerRegionOutOfRange,
};
use crate::system_configuration::{
    filesystem_identifier_self_described_by_the_slot, journal_in_flight_record_limit,
    journal_in_flight_record_limit_fits_its_four_byte_field,
    journal_ring_geometry_fields_recorded_in_the_slot, root_ring_region_count_recorded_in_the_slot,
    root_ring_region_stride_fields_recorded_in_the_slot, unit_area_start_slot_recorded_in_the_slot,
    unit_geometry_fields_recorded_in_the_slot, IncompatBitmap, SystemConfiguration,
    SystemConfigurationSlotRefusal, SystemImmutableSizes,
    EXTENSION_POINT_DECLARED_BYTES_OF_THE_FIRST_VERSION,
    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
};
use crate::tombstone_tree::TombstoneContainerIndexInTree;
use crate::tombstone_tree::CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES;
use crate::transaction::{
    role_of_allocation_record_tree_node, role_of_extent_upper_node,
    take_the_mapping_positions_over_the_location_hints, ContainerTreeIndexNodesVersion,
    ContainerTreeRootUnitPositions, FileOfAnInodeOtherThanTheFirst, FileVersionTreeIdentifiers,
    InodeLeafContainerVersion, LineageContainerVersion, MultiLevelCodeTwoTree, PublishedUnit,
    RetirementContainerVersion, TombstoneContainerVersion, TransactionOutput, TransactionUnit,
    FIRST_INODE_NUMBER,
};
use crate::unit::{
    data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
    parse_packed_unit, unit_filesystem_identifier, IndexNodeHeader, PackedIdentity,
    PackedUnitHeader, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE, PACKED_TYPE_LINEAGE,
    PACKED_TYPE_RETIREMENT, PACKED_TYPE_TOMBSTONE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
    UNIT_CLASS_PACKED,
};
use crate::write_accounting::WritesByStructureKind;
use crate::write_request_split::{data_units_read_within_the_file_size, DataUnitsOfTheInode};

mod allocation_records_and_instance_table_chain;
mod allocation_records_one_per_device;
mod checks_shared_by_the_cold_walk_and_the_rebuild;
mod container_tree_reads;
mod effective_rollback_floor;
mod fall_back_on_the_three_readers;
mod journal_scan_and_replay;
mod pool_readers;
mod root_ring_reads_and_root_choice;
mod system_configuration_choice;
mod tree_table_entries;
mod unit_reads_through_the_central_mapping;
mod version_rebuild;
mod walk_to_file;

pub use allocation_records_and_instance_table_chain::*;
pub use allocation_records_one_per_device::*;
pub use checks_shared_by_the_cold_walk_and_the_rebuild::*;
pub use container_tree_reads::*;
pub use effective_rollback_floor::*;
pub use fall_back_on_the_three_readers::*;
pub use journal_scan_and_replay::*;
pub use pool_readers::*;
pub use root_ring_reads_and_root_choice::*;
pub use system_configuration_choice::*;
pub use tree_table_entries::*;
pub(crate) use unit_reads_through_the_central_mapping::*;
pub use version_rebuild::*;
pub use walk_to_file::*;

/// 恢复路径读盘的口子。两个实现：文件后端的池（`Vec<(DeviceIdentity, Device)>`）与崩溃点重放枚举出的崩溃镜像（harness）。
pub trait PoolReader {
    fn device_identities(&self) -> Vec<DeviceIdentity>;
    /// 这块盘有多少字节；池里没有这块盘就 `None`。
    /// 恢复拿它判「盘上读来的分配记录的跨度落不落在这块盘的单元区里」——那是分配器位图的长度，
    /// 而位图在 `mount` 里才建出来，判定必须在字节进分配器之前做。
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64>;
    /// 这块盘这一刻的 `physical_block_size`：一池真盘问盘（[`BlockDevice::probe_physical_block_size`]），包装读者转给里面那一份，
    /// 内存镜像答它记下的、在它上面做 mkfs 的那块盘的块宽。池里没有这块盘就 `None`。只读挂载择完系统配置之后逐盘拿它比系统配置记着的
    /// mkfs 时的值（D12（目标介质） 已定项 4 比对清单第一行「拒绝挂载（可写与只读都拒）」，`mounted_read::mount_read_only`）。
    /// 没有默认体：每个读者都要答出自己的盘是多宽，不许静默跳过这一比。
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes>;
    /// 读不到（没有那块盘、越界）返回 None：恢复把它当「这一份不在」。
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>>;
    /// journal 环里可能有字节的记录槽偏移；不知道就返回 None，恢复全环扫描。
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>>;
    /// 读坏之后当场重读的那一读（[`BlockDevice::reread_at_after_a_bad_read`]）。默认就是 [`PoolReader::read`]。
    /// 调用方不直接调它：当场重读的那几处经 [`PoolReaderOfOneReadPass::rereading_after_a_bad_read`] 读。
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.read(device, offset, length)
    }
    /// 当场重读那一读读到、核过自证的字节交给盘那一层（[`BlockDevice::keep_the_self_verified_reread`]）。默认什么都不做。
    /// 调用方不直接调它：经 [`PoolReader::note_that_the_bytes_read_self_verify`] 由重读那一遍的读者转过来。
    fn keep_the_self_verified_reread(
        &self,
        _device: DeviceIdentity,
        _offset: DeviceOffsetInBytes,
        _bytes: &[u8],
    ) {
    }
    /// 读的一方核过自证（根槽自证、单元校验和与位置项相符）之后报一声：这几字节是从 `device` 的 `offset` 起读到的、自证过。
    /// 默认什么都不做；当场重读那一遍的读者（[`PoolReaderOfOneReadPass`] 的重读那一种）把它转成
    /// [`PoolReader::keep_the_self_verified_reread`]：重读读到自证过的就换掉这次挂载记下的那一份。
    fn note_that_the_bytes_read_self_verify(
        &self,
        _device: DeviceIdentity,
        _offset: DeviceOffsetInBytes,
        _bytes: &[u8],
    ) {
    }
    /// 读的一方核出读成的字节自证不过（系统配置槽的 magic 与整槽校验和、节点大小，单元校验和与位置项不符）之后报一声：从 `device` 的 `offset` 起
    /// 读的那 `length_in_bytes` 字节读坏了。默认什么都不做；盘切片那一份交给盘那一层
    /// （[`BlockDevice::note_the_read_failing_self_verification_and_forget_its_bytes`]：可写挂载这次挂载共用的那一份记进读坏账、丢掉那几块）。
    /// 与 [`PoolReader::note_that_the_bytes_read_self_verify`] 对称，报的是另一半。根环槽读成而自证不过不报：从没写过的全 0 槽同样自证不过，
    /// 读的一方分不出那是读坏还是本来就空。
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        _device: DeviceIdentity,
        _offset: DeviceOffsetInBytes,
        _length_in_bytes: usize,
    ) {
    }
    /// 按一组位置条目读一个单元时只坏一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：同一组里前一条读不回字节或整单元校验和
    /// 对不上，后一条对上、交回了它那一份。`locations` 是那一组两条位置条目（哪个单元由它们认）。默认什么都不做；要取数的读者
    /// 套 [`PoolReaderCountingUnitReadsWithOneCopyFailing`]。两条位置提示都没对上、转去查中央映射的那一回不算（提示过期与两份都坏分不开），
    /// 映射那一组里只坏一份的照算；按提示读到只坏一份而映射里查到的落点不是提示也不报
    /// （`PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition`）。
    fn note_that_a_unit_was_read_with_one_copy_failing(&self, _locations: &[LocationEntry; 2]) {}
}

/// 恢复走不下去的原因，按调用方能据以行动的粒度分。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryFailure {
    /// 池里没有一块盘交得出有效的系统配置：每块盘的两个槽都无效。
    /// 只有一块盘两个槽全废、别的盘还交得出一份时**不报这个**——系统配置每盘放一份买的就是这份冗余
    /// （D22（单元原子性怎么合成） 已定项 8 第 1 条）。
    /// 带的是 [`PoolReader::device_identities`] 次序里第一块两个槽都无效的盘；这个成员报出来时池里每块盘都是这样，
    /// 所以它就是次序里的第一块盘。
    NoValidSystemConfiguration {
        first_device_with_no_valid_system_configuration_slot: DeviceIdentity,
    },
    /// 池里没有一份可择的系统配置，而至少一槽 magic 与整槽校验和都过、只是 incompat 位图这个读者不认识
    /// （[`SystemConfigurationSlotRefusal::IncompatBitsNotRecognized`]：不是这一版布局写的，D15（格式冻结政策） 已定项 4）。
    /// 这不是数据坏了，是布局不认识：报它、不报 [`Self::NoValidSystemConfiguration`]，
    /// 调用方要做的决定不同（换一个认得这一版布局的实现，不是修盘）。别的盘上的槽同时自证不过也报它：
    /// 认得出的那一槽说明这个池是一个完整的、这个读者不认识的布局。带的是 [`PoolReader::device_identities`] 次序里
    /// 第一块带着这种槽的盘与它先读到的那一槽的位图。在读系统配置那一步就返回，盘上逐字节不变。
    SystemConfigurationIncompatBitsNotRecognized {
        first_device_carrying_them: DeviceIdentity,
        incompat_bitmap: IncompatBitmap,
    },
    /// 各盘的系统配置 fsid 或设备数对不上。
    SystemConfigurationsDisagree,
    /// 交进来的池里一块盘都没有（[`PoolReader::device_identities`] 为空）：调用方要查的是自己交进来的盘表，
    /// 不是哪块盘是外来的，所以不报 [`Self::SystemConfigurationsDisagree`]（C462（空池恢复报的错名不副实），
    /// `research/prompts/m3-ring-r1-main-verification.md` 第二节第 11 行「恢复这一层单列错误成员」）。一个槽都没读，盘上不变。
    NoDeviceInThePool,
    /// 系统配置自述的每区槽数 S 落在格式承诺的区间之外（D22（单元原子性怎么合成） 已定项 2 参数表「每区槽数 S」那一格：
    /// 下界 k_tol + 2 = 4，上界是格式常量 16，`singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM` 与 `_MAXIMUM`）。
    /// S 在区间里而 S × 固定结构槽距越过区域间距 P × chunk 的不在这里，归
    /// [`SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingRegionLongerThanTheRegionStride`]（那一判要槽距）。
    /// S 是**盘上读来的一字节**、不是我们的不变量，所以报错、不断言；它又是池级字段（两盘四槽同值），
    /// 换一槽换一盘读到的还是它 ⇒ 整池拒绝挂载，不退化成「这一槽不可择」。
    /// 在读系统配置那一步就返回，盘上逐字节不变。
    RootRingSlotsPerRegionOutOfRange {
        device: DeviceIdentity,
        out_of_range: RootRingSlotsPerRegionOutOfRange,
    },
    /// 系统配置里一个池级不可变字段的值这个读者不收（[`SystemConfigurationValueOutsideWhatThisReaderAccepts`]：格式版本、加密类型、
    /// 固定结构槽距、`physical_block_size`、journal 环长，代码审阅第 29 条与第 38 条）。这几样都是**盘上读来的值**、不是不变量；
    /// 与每区槽数 S 越界同一个处置（[`Self::RootRingSlotsPerRegionOutOfRange`]）：池级字段两盘四槽同值，换一槽换一盘读到的还是它 ⇒ 整池拒绝挂载，
    /// 不退化成「这一槽不可择」。在读系统配置那一步就返回，盘上逐字节不变。`device` 是先读到这一槽的那块盘。
    SystemConfigurationValueRefused {
        device: DeviceIdentity,
        value: SystemConfigurationValueOutsideWhatThisReaderAccepts,
    },
    /// 一块盘的完整槽数不到单元区起点（代码审阅第 35 条）：盘的字节数是块设备报的、或崩溃镜像自述的，不是不变量。
    /// 可写挂载在择系统配置之后、读根环之前逐盘判（[`every_device_reaches_the_unit_area_start`]），盘上逐字节不变；
    /// 恢复判分配记录落点那一道（`allocation_records_fit_the_pool_geometry`）同样报它，不拿它去减。
    DeviceEndsBeforeTheUnitAreaStart {
        device: DeviceIdentity,
        device_bytes: u64,
    },
    /// 根环里一条自证过的根都没有。
    NoValidRoot,
    /// 一个单元两条位置条目都读不到校验和相符的那份。
    UnitUnreadable { slot: SlotNumber },
    /// 单元读到了、解不出来（哪一个、哪一条）。
    UnitMalformed { what: &'static str },
    /// 某条不变量在走读时判红。
    InvariantViolated {
        invariant: &'static str,
        detail: &'static str,
    },
    /// 索引节点头自述的条目宽不等于这棵树的条目字段表宽（窄于、宽于都算；中央映射条目只拒窄于，欠账 C307）：条目宽是**盘上的值**，
    /// `parse_index_node` 只判了它 ≥ key 宽，按字段表的固定偏移切之前要再判一次（panic 面普查 R1 / R3 / R4）。
    EntryWidthDiffersFromItsFieldTable {
        what: &'static str,
        entry_bytes: usize,
        field_table_bytes: usize,
    },
    /// 重建出来的这一版的记账行里没有 inode 号水位那一行（统计量标签 12，D5（快照 / 空间记账机制） 已定项 8 的池级三行之一）：
    /// 标签是记账条目里**盘上读来的 2 字节**、可以是任何值，不是不变量。发布路径按这一行发下一个 inode 号
    /// （`TransactionOutput::inode_number_watermark`），所以在把这一版交出去之前判一次（panic 面普查 R11：
    /// 不判的话，挂载之后第一次发布在那句 `expect` 上 panic）。
    InodeNumberWatermarkRowMissingFromTheAccountingTree,
    /// 分配记录里的结构值落在这个池的几何之外：设备身份不在池里、槽号在单元区起点之下、跨度越过单元区末尾，
    /// 或者同一块盘上两条记录罩住同一个槽（I-5.4（分配记录罩住的槽互不相交） 的读路径形态；panic 面普查 R6 / R8 / R9）。
    /// 这几样都是**盘上读来的值**，不是不变量——分配器的位图、下标与断言都按它们已经判过来写。
    AllocationRecordOutsideThePoolGeometry { what: &'static str },
    /// 位置提示读不到、映射里也没有这个 key。
    MappingMiss { slot: SlotNumber },
    /// 位置提示读不到、经映射仍读不到。
    MappingStillUnreadable { slot: SlotNumber },
    /// 所选根那次发布（与所选根同实例、同 checkpoint_txg 的记录）读得出的几条里，带「本次发布末条」标志的多于一条：
    /// 链首的锚点「所选根覆盖的最后一条」按末条标志认（D23（journal 的角色与格式） 已定项 14 注 1，读法乙），
    /// 两条以上都带时认哪一条**条款没有写** ⇒ 第一版不支持：恢复在施加任何一条记录之前停下（挂载因此在任何落盘动作之前返回）。
    /// 写者每次发布只给真正的最后一条带标志（已定项 17），走到这里要一条记录坏了而校验和恰好仍对得上，或者镜像是改出来的。
    /// `counters` 是带标志的那几条的 jsn 计数器，按升序。
    RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
        instance: InstanceGeneration,
        checkpoint_txg: CheckpointTxg,
        counters: Vec<u64>,
    },
}

/// 读根环时，这个进程知道住着一条根的一个槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过，
/// `allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）这一次读坏（`first_reading`：读不出或自证不过），
/// 重读一次仍坏（`reread`；[`readable_roots_rereading_ring_slots_known_to_hold_a_root_once`]；D16（发布语义） 已定项 1「根槽这一次读坏」那一行
/// 全句，用户 2026-09-26 定）：那一槽里的根在不在判不了，不按有根或没根猜。挂着时抬 F 与管理员回退拒这一次，经
/// `mount::StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot` 交出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRingSlotKnownToHoldARootStillBadAfterOneReread {
    pub ring_slot: RootRingSlot,
    pub first_reading: BadRootRingSlotReading,
    pub reread: BadRootRingSlotReading,
}

/// 算生效的回退下界 F 时（[`effective_rollback_floor_rereading_the_newest_instance_table_once`]），根环里最新那条根指着的实例表
/// （按它判哪几条根被抛弃）读不出、解不开，或一条自证过的根都择不到；重读一次（C554 乙的形态，R = 1：立即重读根环与那张表）仍是这样。
/// 不按「不按表滤」往下走（那样被抛弃时间线上的根带的 F 也算进生效值，C554 乙报告 Q6）。挂着时抬 F 与管理员回退拒这一次，经
/// `mount::StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor` 交出。
/// `newest_root_on_the_reread` 是重读那一遍择到的最新那条根（实例代号, checkpoint_txg），一条都没择到时 `None`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceTableOfTheNewestRootStillUnreadableAfterOneReread {
    pub newest_root_on_the_reread: Option<(InstanceGeneration, CheckpointTxg)>,
}

/// 管理员回退算 F 生效值（[`effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`]）时
/// 重读一次仍读坏的是哪一样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectiveFloorReadingStillUnreadableAfterOneReread {
    RootRingSlotKnownToHoldARoot(RootRingSlotKnownToHoldARootStillBadAfterOneReread),
    InstanceTableOfTheNewestRoot(InstanceTableOfTheNewestRootStillUnreadableAfterOneReread),
}

/// 恢复的结果：择到的根下面没有文件（第 0 代）、读回文件、或走不下去。`root` 恒是**所选**的那条根。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryOutcome {
    NoFile {
        root: (InstanceGeneration, CheckpointTxg),
    },
    FileRead {
        root: (InstanceGeneration, CheckpointTxg),
        content: Vec<u8>,
    },
    Failed {
        root: Option<(InstanceGeneration, CheckpointTxg)>,
        failure: RecoveryFailure,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JournalScanReport {
    pub valid_records: usize,
    pub above_water: usize,
    pub prefix_applied: usize,
    /// 点名单元**真的逐个读过、校验和逐个比对过**的提交记录条数。
    /// 关掉点名验证那一臂（`JournalPolicy::ConsultWithoutNamedVerification`）恒为 0——
    /// 它一个单元都不读，这个数就是两臂在健康镜像上唯一分得开的读数。
    pub verification_passed: usize,
    pub verification_failed: usize,
    /// 这次恢复施加的记录里最大的事务号（空发布的事务号 0 不进 max，D23（journal 的角色与格式） 已定项 19 ①）；
    /// 可写挂载给上一个实例写行时 W 取它（D18（块里携带什么信息） 已定项 11 的行记录）。
    pub maximum_applied_transaction: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryReport {
    pub outcome: RecoveryOutcome,
    /// 施加 journal 之后实际走的那条根（所选根，或由记录重建的根）；择不到根、或恢复在施加任何记录之前停下
    /// （`RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`、所选根的 txg 加一溢出）时是 None。记录核对器拿它判「恢复自称的状态」。
    pub effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    pub journal: JournalScanReport,
    /// 位置提示读不到、转去查映射的次数。
    pub mapping_fallbacks: usize,
    /// 择系统配置时两个槽都不可择而被跳过的盘（[`SystemConfigurationChoice::devices_without_a_mountable_system_configuration_slot`]，
    /// C463（一块盘的系统配置两份全废之后静默成功）「只读与恢复路径报出来」）；冗余完好时为空。系统配置择不出来（`outcome` 是
    /// 择系统配置那一步的 `Failed`）时也是空的：那时没有「跳过它、照常择别的盘」这回事。
    pub devices_without_a_mountable_system_configuration_slot: Vec<DeviceIdentity>,
}

/// 要不要看 journal：正常恢复看；崩溃点重放里与「不看」逐状态比，问 journal 在这条流里承不承重；
/// 「看但不验点名单元」只供测试强制进入（`.claude/rules/fs-design.md` 五条硬要求第 2 条），证明验证那一步承重。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalPolicy {
    Consult,
    ConsultWithoutNamedVerification,
    Ignore,
}

impl JournalPolicy {
    /// 看 journal 的两臂各自交给取前缀的点名验证；不看 journal 那一臂不重放，交回 `None`。
    #[must_use]
    pub const fn named_unit_verification_of_the_replay(self) -> Option<NamedUnitVerification> {
        match self {
            Self::Consult => Some(NamedUnitVerification::Verify),
            Self::ConsultWithoutNamedVerification => Some(NamedUnitVerification::Skip),
            Self::Ignore => None,
        }
    }
}

/// 这一版中央映射树的叶条目解开之后的三样：每条的映射 key（叶条目本来的次序）、被活快照钉住的单元留存的映射条目
/// （判决 S3 的 α：每条 deadlist 条目的映射 key 一条，按映射 key 的序），与每条映射条目的位置（判决
/// `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「重建时映射条目的位置覆盖提示」：这一版装完之后，指向进映射单元的每一处指针换成它，
/// `transaction::take_the_mapping_positions_over_the_location_hints`）。
struct MappingEntriesOfTheVersion {
    mapping_keys: Vec<Vec<u8>>,
    mapping_entries_kept_for_the_snapshots: Vec<(Vec<u8>, [LocationEntry; 2])>,
    mapping_position_of_key: BTreeMap<Vec<u8>, [LocationEntry; 2]>,
}

/// 读一棵容器树时按树分的几样：树号、种类码（定 key 宽）、打包记录类型、记录宽与记录的读法、报错时说的是哪一样。
struct ContainerTreeToRead<Record> {
    tree: TreeIdentifier,
    kind: u16,
    record_type: u16,
    record_width_in_bytes: u64,
    parse_record: fn(&[u8]) -> Option<Record>,
    what_index_node: &'static str,
    what_entry: &'static str,
    what_container: &'static str,
    what_container_against_its_entry: &'static str,
    what_record: &'static str,
}

/// 从盘上读回来的一棵容器树的码 2 节点：形状，每个节点（bump 次序：层级升序、同层从左到右，根最末）的位置、指针与字节，
/// 与层级 1 那几个节点的条目（按叶序：分隔 key、身份引用、指容器的指针）。
struct ContainerTreeIndexNodesRead {
    shape: ContainerTreeShape,
    nodes: Vec<(ContainerTreeNodePosition, NodePointer, Vec<u8>)>,
    container_entries: Vec<(u64, PackedIdentity, NodePointer)>,
}

/// 从盘上读回来的一棵容器树：码 2 节点那一半与每片容器（叶序：内容、指针、字节）。
struct ContainerTreeRead<Record> {
    index_nodes: ContainerTreeIndexNodesRead,
    containers: Vec<(RecordContainer<Record>, NodePointer, Vec<u8>)>,
}

/// 冷走读里的中央映射树（自举豁免，只按父指针里的位置条目读）：到第一次有树根的提示读不出、要经映射回退时，
/// 或走到 `TreeRoots` 那一步时才读，读过一次就留着。不提前读：提示都读得出的镜像上，走读的读序与判红次序照旧。
/// 映射树多层时整棵读回来（D19（块指针的结构与宽度预算） 已定项 5），每个节点按父条目核（`code_two_tree::read_code_two_tree`）。
struct CentralMappingTreeReadOnFirstUse<'walk> {
    reader: &'walk dyn PoolReader,
    root: &'walk RootRecord,
    expected_filesystem_identifier: u64,
    tree: OnceCell<CodeTwoTreeReadFromDisk>,
}

/// 进映射的单元各有几个（[`mapping_entry_count_is_one_per_mapped_unit`] 的输入）。
struct MappedUnitCounts {
    extent_tree_nodes: usize,
    allocation_record_tree_nodes: usize,
    accounting_tree_nodes: usize,
    /// deadlist 树的节点数（零根时 0，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）。
    deadlist_tree_nodes: usize,
    /// 被活快照钉住、留存的映射条目数 = deadlist 条目数（判决 S3 的 α：每条 deadlist 条目的映射 key 一条）。
    mapping_entries_kept_for_the_snapshots: usize,
    /// inode 树的叶容器数 = 它的根的条目数（根恒是层级 1，下面每条条目一片容器；层级另判）。
    inode_leaf_containers: usize,
    /// 数据单元数 = extent 树里第一个文件的数据指针数（下段叶记录条数，或内联的那一个）。
    data_units: usize,
    /// 删除那两棵树的节点数：`deleted_inodes` 根（队列非空时 1）、墓碑树的每片容器与根。
    deletion_tree_nodes: usize,
}

/// inode 树根的一条内部条目解开之后的三样：分隔 key、身份引用、指向叶容器的子指针。
struct InodeInternalEntry {
    separator_key: u64,
    identity: PackedIdentity,
    child: NodePointer,
}

/// 一个文件（第一个文件，或第一个文件之外的文件）的第 `position` 个数据单元在查找路径上的样子：它的数据指针与 extent 树的号
/// （数据单元头里的出生树要与它相同）。它是哪个对象的由核它时交的那条 inode 记录说（五元组的对象要等于那条记录的号）。
struct DataUnitOfAFile<'pointer> {
    position: usize,
    pointer: &'pointer DataPointer,
    extent_tree: TreeIdentifier,
}

/// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。所选那一版三样读者的并集里有字节丢了、没有内容拒收，
/// 系统配置又没见证它时退到 [见证到的那一版, 所选根) 里读得出、不被抛弃的最新一条根（[`fall_back_judged_on_the_three_readers`]）：
/// 那一版走得通，`root` 与 `effective_root` 都报退到的那一条，`journal` 仍是对所选根扫环、施加那一趟的报告；走不通留在所选那一版、
/// 照它自己的走读报。不看 journal 的那一臂（`JournalPolicy::Ignore`）不退。系统配置不筛 fsid；调用方知道本池 fsid 的用 [`recover_of_the_pool`]。
#[must_use]
pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {
    recover_from_the_system_configuration_choice(
        reader,
        policy,
        choose_system_configuration_naming_devices_without_a_mountable_slot(reader),
    )
}

/// 与 [`recover`] 同一条恢复路径，系统配置只从自述 fsid 是 `filesystem_identifier_of_the_pool` 的槽里择
/// （[`choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot`]）：别的池的槽世代号再大也择不到。
/// 盘上没有一槽是这个 fsid 的池的，报告的失败是 [`RecoveryFailure::NoValidSystemConfiguration`]，不读别的池。
#[must_use]
pub fn recover_of_the_pool(
    reader: &dyn PoolReader,
    policy: JournalPolicy,
    filesystem_identifier_of_the_pool: &[u8; 16],
) -> RecoveryReport {
    recover_from_the_system_configuration_choice(
        reader,
        policy,
        choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot(
            reader,
            filesystem_identifier_of_the_pool,
        ),
    )
}

/// [`recover`] 与 [`recover_of_the_pool`] 共用的恢复路径：系统配置已经择过，择不到的把失败报成报告。
fn recover_from_the_system_configuration_choice(
    reader: &dyn PoolReader,
    policy: JournalPolicy,
    system_configuration_choice: Result<SystemConfigurationChoice, RecoveryFailure>,
) -> RecoveryReport {
    let mut mapping_fallbacks = 0;
    let SystemConfigurationChoice {
        system_configuration,
        devices_without_a_mountable_system_configuration_slot,
    } = match system_configuration_choice {
        Ok(choice) => choice,
        Err(failure) => {
            return RecoveryReport {
                outcome: RecoveryOutcome::Failed {
                    root: None,
                    failure,
                },
                effective_root: None,
                journal: JournalScanReport::default(),
                mapping_fallbacks,
                devices_without_a_mountable_system_configuration_slot: Vec::new(),
            }
        }
    };
    let Some(root) = choose_root(reader, &system_configuration) else {
        return RecoveryReport {
            outcome: RecoveryOutcome::Failed {
                root: None,
                failure: RecoveryFailure::NoValidRoot,
            },
            effective_root: None,
            journal: JournalScanReport::default(),
            mapping_fallbacks,
            devices_without_a_mountable_system_configuration_slot,
        };
    };
    let root_key = (root.instance, root.checkpoint_txg);
    // 看 journal 的两臂扫一遍环：重放用它，判退那一步先拿它挡一下（`fall_back_judged_on_the_three_readers` ①）；不看 journal 那一臂不扫、不退。
    let records = match policy {
        JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {
            scan_journal(reader, &system_configuration)
        }
        JournalPolicy::Ignore => BTreeMap::new(),
    };
    let replayed = match policy.named_unit_verification_of_the_replay() {
        Some(named_unit_verification) => replay_journal(
            reader,
            &root,
            system_configuration.immutable.sizes.journal_ring_bytes,
            &records,
            named_unit_verification,
        ),
        None => Ok((JournalScanReport::default(), root)),
    };
    // 恢复在施加任何一条记录之前停下（锚点认哪一条条款没写、所选根的 txg 加一溢出）：没有「实际走的那条根」可报。
    let (journal, effective_root) = match replayed {
        Ok(replayed) => replayed,
        Err(failure) => {
            return RecoveryReport {
                outcome: RecoveryOutcome::Failed {
                    root: Some(root_key),
                    failure,
                },
                effective_root: None,
                journal: JournalScanReport::default(),
                mapping_fallbacks,
                devices_without_a_mountable_system_configuration_slot,
            }
        }
    };
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
    // 判退不看这一处自己的读者（走读）通没通，看三样读者的并集（C755，用户 2026-10-20 定「三句都补」；C866 改法甲：并集里有内容拒收不退）：
    // 退到的那一版走读走得通就落在那一版；走不通（或不退）留在所选的那一版，成不成照它自己的走读。
    let version_fallen_back_to_walked = match policy {
        JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {
            fall_back_judged_on_the_three_readers(
                reader,
                &system_configuration,
                &root,
                &effective_root,
                &records,
            )
        }
        JournalPolicy::Ignore => None,
    }
    .and_then(|fall_back| {
        walk_to_file_in_the_unit_area_starting_at(
            reader,
            unit_area_start,
            &fall_back.version_fallen_back_to,
            &mut mapping_fallbacks,
        )
        .ok()
        .map(|content| (fall_back.version_fallen_back_to, content))
    });
    let (outcome, effective_root) = match version_fallen_back_to_walked {
        Some((version_fallen_back_to, content)) => {
            let key_fallen_back_to = (
                version_fallen_back_to.instance,
                version_fallen_back_to.checkpoint_txg,
            );
            let outcome = match content {
                Some(content) => RecoveryOutcome::FileRead {
                    root: key_fallen_back_to,
                    content,
                },
                None => RecoveryOutcome::NoFile {
                    root: key_fallen_back_to,
                },
            };
            (outcome, version_fallen_back_to)
        }
        None => match walk_to_file_in_the_unit_area_starting_at(
            reader,
            unit_area_start,
            &effective_root,
            &mut mapping_fallbacks,
        ) {
            Ok(Some(content)) => (
                RecoveryOutcome::FileRead {
                    root: root_key,
                    content,
                },
                effective_root,
            ),
            Ok(None) => (RecoveryOutcome::NoFile { root: root_key }, effective_root),
            Err(failure) => (
                RecoveryOutcome::Failed {
                    root: Some(root_key),
                    failure,
                },
                effective_root,
            ),
        },
    };
    RecoveryReport {
        outcome,
        effective_root: Some((effective_root.instance, effective_root.checkpoint_txg)),
        journal,
        mapping_fallbacks,
        devices_without_a_mountable_system_configuration_slot,
    }
}
