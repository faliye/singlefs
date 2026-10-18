//! 冷启动恢复（里程碑步 6）：择系统配置 → 择根 → 全环扫描 journal → 按前缀口径施加记录的新根段 → 沿树走到数据单元。
//! 只通过 [`PoolReader`] 读盘，不沿进程内的指针热读；崩溃点重放（步 7）拿同一条路径去判每一个崩溃状态。
//!
//! 口径：择系统配置 = 校验和过且世代号大的那个槽、各盘 fsid 与设备数要对得上（D22（单元原子性怎么合成） 已定项 16）；
//! 择根 = 三个区域全部槽里自证过、`(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7）；
//! journal 全环扫描、不先信 tail（D23（journal 的角色与格式） 已定项 3），两份镜像任一份自证过即算在（D23（journal 的角色与格式） 已定项 14）；
//! 前缀 = `(实例代号, checkpoint_txg)` 严格大于所选根、jsn 严格连续、反向链等于本实例内逻辑前一条的头算出来的链值
//! （I-8.6（反向链算法），前一条不在盘上不判）、提交标记齐全、在飞上限之内、点名单元逐项验过，
//! 施加一条记录 = 把所选根的四个字段换成记录新根段里的（D23（journal 的角色与格式） 已定项 15）。

use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
    FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR, LINEAGE_RECORD_BYTES,
    MAPPING_ENTRY_BYTES, NODE_BYTES, RETIREMENT_RECORD_BYTES, ROOT_RECORD_BYTES, ROOT_RING_REGIONS,
    SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
    TOMBSTONE_RECORD_BYTES,
};
use singlefs_replay_prefix::{
    choose_root_slot, first_copy_of_each_record_key, replay_core, CoreFailure, CoreRecord,
    JournalCounter, NamedUnitVerification, PrefixKernelCheckpointTxg,
    PrefixKernelInstanceGeneration, PrefixKernelJournalRecordOrdinalWithinPublish,
    PrefixKernelJournalRecordPlaceInPublish, RootChoiceKey, TransactionNumber,
};

use crate::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InodeNumber,
    InstanceGeneration, SlotNumber, TreeIdentifier,
};
use crate::allocation_record_tree::{
    read_allocation_record_tree, AllocationRecordTreeGeometry, AllocationRecordTreeHeaderJudgement,
    AllocationRecordTreeReadFromDisk,
};
use crate::allocator::{
    unit_area_slots_of_device_starting_at, AllocationRecord,
    AllocationRecordTreeOfTheVersionWithoutFile, DeviceEndsBeforeTheUnitAreaStart, UnitAreaStart,
};
use crate::block_device::BlockDevice;
use crate::checksum::crc32_castagnoli;
use crate::code_two_tree::{
    read_code_two_tree, CodeTwoTreeHeaderJudgement, CodeTwoTreeReadFromDisk,
};
use crate::container_tree::{
    ContainerTreeNodePosition, ContainerTreeShape, RecordContainer, IDENTITY_OF_A_CODE_TWO_CHILD,
};
use crate::deleted_inodes::inode_numbers_of_the_deleted_inodes_root_entries;
use crate::deleted_inodes::DELETED_INODES_KEY_WIDTH_IN_BYTES;
use crate::extent_tree::{
    read_extent_tree, ExtentTreeHeaderJudgement, ExtentTreeReadFromDisk, ExtentTreeReading,
    ExtentTreeVersion, ExtentsOfAFileReadFromDisk,
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
use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
use crate::pointer::{
    location_entries_ascend_by_device, DataPointer, LocationEntry, NodePointer,
    PointerHeadFieldGroup, PointerHeadFieldOutsideTheFirstVersion, PointerHeadFieldsJudgement,
};
use crate::records::{
    mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry_or_refusal,
    parse_mapping_entry, AccountingEntry, InodeRecord, LineageRecord, PointerCarryingEntryRefusal,
    RetirementRecord, TombstoneRecord, TreeTableEntry, NLINK_OF_AN_INODE_BEING_DELETED,
    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
    TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE,
    TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
    TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
};
use crate::retirement_tree::RetirementContainerIndexInTree;
use crate::root_record::RootRecord;
use crate::root_ring::{region_start, slot_offset, RootRingSlot, RootRingSlotsPerRegionOutOfRange};
use crate::system_configuration::{
    journal_in_flight_record_limit, journal_in_flight_record_limit_fits_its_four_byte_field,
    journal_ring_geometry_fields_recorded_in_the_slot, unit_area_start_slot_recorded_in_the_slot,
    IncompatBitmap, SystemConfiguration, SystemConfigurationSlotRefusal, SystemImmutableSizes,
    JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION, ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
};
use crate::tombstone_tree::TombstoneContainerIndexInTree;
use crate::tombstone_tree::CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES;
use crate::transaction::{
    role_of_allocation_record_tree_node, role_of_extent_upper_node, ContainerTreeIndexNodesVersion,
    FileVersionTreeIdentifiers, InodeLeafContainerVersion, LineageContainerVersion,
    MultiLevelCodeTwoTree, PublishedUnit, RetirementContainerVersion, TombstoneContainerVersion,
    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use crate::unit::{
    data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
    parse_packed_unit, unit_filesystem_identifier, IndexNodeHeader, PackedIdentity,
    PackedUnitHeader, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE, PACKED_TYPE_LINEAGE,
    PACKED_TYPE_RETIREMENT, PACKED_TYPE_TOMBSTONE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
    UNIT_CLASS_PACKED,
};
use crate::write_accounting::WritesByStructureKind;
use crate::write_request_split::data_unit_count_of_a_sequential_write;

/// 恢复路径读盘的口子。两个实现：文件后端的池（`Vec<(DeviceIdentity, Device)>`）与崩溃点重放枚举出的崩溃镜像（harness）。
pub trait PoolReader {
    fn device_identities(&self) -> Vec<DeviceIdentity>;
    /// 这块盘有多少字节；池里没有这块盘就 `None`。
    /// 恢复拿它判「盘上读来的分配记录的跨度落不落在这块盘的单元区里」——那是分配器位图的长度，
    /// 而位图在 `mount` 里才建出来，判定必须在字节进分配器之前做。
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64>;
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
}

/// 一遍读是第一次读，还是读坏之后当场重读的那一遍（D19（块指针的结构与宽度预算） 已定项 5、D16（发布语义） 已定项 1「根槽这一次读坏」那一行
/// 与「重读一次」：只重读一次，重读要再读一次盘）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadPassOverThePool {
    FirstRead,
    /// 读走 [`PoolReader::reread_after_a_bad_read`]：可写挂载里绕过这次挂载记下的那一份、直接读底下那块盘
    /// （主 agent 判定：抬 F 与续做里「读坏当场重读」那一读不许被预演与真做共用的那一份吃掉）；读到自证过的换掉记下的那一份。
    RereadAfterABadRead,
}

/// 一遍读的读者：第一次读照原样读；读坏之后当场重读的那一遍，读走 [`PoolReader::reread_after_a_bad_read`]，
/// 读的一方核过自证报上来的字节（[`PoolReader::note_that_the_bytes_read_self_verify`]）转成 [`PoolReader::keep_the_self_verified_reread`]。
pub struct PoolReaderOfOneReadPass<'reader, Reader: PoolReader + ?Sized> {
    reader: &'reader Reader,
    pass: ReadPassOverThePool,
}

impl<'reader, Reader: PoolReader + ?Sized> PoolReaderOfOneReadPass<'reader, Reader> {
    #[must_use]
    pub fn reading_for_the_first_time(reader: &'reader Reader) -> Self {
        PoolReaderOfOneReadPass {
            reader,
            pass: ReadPassOverThePool::FirstRead,
        }
    }

    #[must_use]
    pub fn rereading_after_a_bad_read(reader: &'reader Reader) -> Self {
        PoolReaderOfOneReadPass {
            reader,
            pass: ReadPassOverThePool::RereadAfterABadRead,
        }
    }

    #[must_use]
    pub fn in_the_pass(reader: &'reader Reader, pass: ReadPassOverThePool) -> Self {
        PoolReaderOfOneReadPass { reader, pass }
    }
}

impl<Reader: PoolReader + ?Sized> PoolReader for PoolReaderOfOneReadPass<'_, Reader> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.reader.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.reader.device_size_in_bytes(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        match self.pass {
            ReadPassOverThePool::FirstRead => self.reader.read(device, offset, length),
            ReadPassOverThePool::RereadAfterABadRead => {
                self.reader.reread_after_a_bad_read(device, offset, length)
            }
        }
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.reader
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reader.reread_after_a_bad_read(device, offset, length)
    }
    fn keep_the_self_verified_reread(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        self.reader
            .keep_the_self_verified_reread(device, offset, bytes);
    }
    fn note_that_the_bytes_read_self_verify(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        match self.pass {
            ReadPassOverThePool::FirstRead => self
                .reader
                .note_that_the_bytes_read_self_verify(device, offset, bytes),
            ReadPassOverThePool::RereadAfterABadRead => self
                .reader
                .keep_the_self_verified_reread(device, offset, bytes),
        }
    }
    /// 两遍一样转过去：重读那一读同样是共用的那一段里的一次读，读坏同样记账（重读不记下读到的字节，丢掉的是第一次读记下的那一份）。
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    ) {
        self.reader.note_that_the_bytes_read_fail_self_verification(
            device,
            offset,
            length_in_bytes,
        );
    }
}

impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.iter().map(|(identity, _)| *identity).collect()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
        Some(block_device.size_in_bytes())
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
        let mut buffer = vec![0u8; length];
        block_device.read_at(offset, &mut buffer).ok()?;
        Some(buffer)
    }
    /// 盘自己答（[`BlockDevice::journal_record_offsets_hint`]）：真盘答 `None`、全环扫描；只记写过的扇区的内存盘答它写过的记录槽。
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
        block_device.journal_record_offsets_hint(ring_start, ring_bytes)
    }
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let (_, block_device) = self.iter().find(|(identity, _)| *identity == device)?;
        let mut buffer = vec![0u8; length];
        block_device
            .reread_at_after_a_bad_read(offset, &mut buffer)
            .ok()?;
        Some(buffer)
    }
    fn keep_the_self_verified_reread(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        if let Some((_, block_device)) = self.iter().find(|(identity, _)| *identity == device) {
            block_device.keep_the_self_verified_reread(offset, bytes);
        }
    }
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    ) {
        if let Some((_, block_device)) = self.iter().find(|(identity, _)| *identity == device) {
            block_device.note_the_read_failing_self_verification_and_forget_its_bytes(
                offset,
                u64::try_from(length_in_bytes).expect("读长装得进 u64"),
            );
        }
    }
}

/// 文件后端的池就是一串盘：读法与切片那一份相同。
impl<Device: BlockDevice> PoolReader for Vec<(DeviceIdentity, Device)> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.as_slice())
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.as_slice(), device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        PoolReader::read(self.as_slice(), device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        PoolReader::journal_record_offsets_hint(self.as_slice(), device, ring_start, ring_bytes)
    }
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        PoolReader::reread_after_a_bad_read(self.as_slice(), device, offset, length)
    }
    fn keep_the_self_verified_reread(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        PoolReader::keep_the_self_verified_reread(self.as_slice(), device, offset, bytes);
    }
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    ) {
        PoolReader::note_that_the_bytes_read_fail_self_verification(
            self.as_slice(),
            device,
            offset,
            length_in_bytes,
        );
    }
}

/// 这一刻读盘的读者（[`BlockDevice::read_at_straight_from_the_disk`]）：写系统配置槽算世代号、覆写哪一槽那一读用它
/// （`transaction::PoolWriter` 的系统配置槽写）。读的一方报上来的自证结论（[`PoolReader::note_that_the_bytes_read_self_verify`]、
/// [`PoolReader::note_that_the_bytes_read_fail_self_verification`]）不转给盘那一层：读到的字节没进这次挂载记下的那一份。
pub struct PoolReaderStraightFromTheDisk<'devices, Device: BlockDevice> {
    devices: &'devices [(DeviceIdentity, Device)],
}

impl<'devices, Device: BlockDevice> PoolReaderStraightFromTheDisk<'devices, Device> {
    #[must_use]
    pub fn over(devices: &'devices [(DeviceIdentity, Device)]) -> Self {
        PoolReaderStraightFromTheDisk { devices }
    }
}

impl<Device: BlockDevice> PoolReader for PoolReaderStraightFromTheDisk<'_, Device> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.devices)
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.devices, device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let (_, block_device) = self
            .devices
            .iter()
            .find(|(identity, _)| *identity == device)?;
        let mut buffer = vec![0u8; length];
        block_device
            .read_at_straight_from_the_disk(offset, &mut buffer)
            .ok()?;
        Some(buffer)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        PoolReader::journal_record_offsets_hint(self.devices, device, ring_start, ring_bytes)
    }
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
    /// （[`SystemConfigurationSlotRefusal::IncompatBitsNotRecognized`]：退役的位 0 那一版旧镜像，或更新的实现写的，
    /// D15（格式冻结政策） 已定项 4）。这不是数据坏了，是布局不认识：报它、不报 [`Self::NoValidSystemConfiguration`]，
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
    /// 系统配置自述的每区槽数 S 落在格式承诺的区间之外（D22（单元原子性怎么合成） 已定项 1 的字段表：
    /// 下界 k_tol + 2 = 4；上界字段表写的是「由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算」，
    /// 那个数今天算不出来、按 16 收着，见 `singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM` 的占位）。
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
    /// 索引节点头自述的条目宽窄于这棵树的条目字段表：条目宽是**盘上的值**，
    /// `parse_index_node` 只判了它 ≥ key 宽，按字段表的固定偏移切之前要再判一次（panic 面普查 R1 / R3 / R4）。
    EntryNarrowerThanItsFieldTable {
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

fn unit_bytes_for_class(unit_class: u8) -> Option<usize> {
    match unit_class {
        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => {
            Some(usize::try_from(DATA_UNIT_BYTES).expect("32768"))
        }
        UNIT_CLASS_INDEX_NODE => Some(usize::try_from(NODE_BYTES).expect("16384")),
        _ => None,
    }
}

/// 一条记录点名的一个单元占几个槽（单元种类定的字节数 ÷ 16384）：可写挂载隔离水位之上没施加的记录点名的落点按它
/// （`mount::placements_named_by_unapplied_journal_records`）。种类不认识的那条点名项（坏盘上的记录：施加前逐项验点名单元时它本来就验不过）
/// 按最宽的那一档（数据单元的两槽）算，隔离只会多不会少。
pub(crate) fn span_in_slots_of_a_named_unit(named: &crate::journal::NamedUnit) -> u64 {
    let unit_bytes = unit_bytes_for_class(named.unit_class)
        .unwrap_or(usize::try_from(DATA_UNIT_BYTES).expect("32768"));
    u64::try_from(unit_bytes).expect("单元字节数装得进 u64") / SLOT_BYTES
}

/// 按指针里的位置条目读一个单元：逐条试，整单元 CRC-32C 等于条目里的校验和才算读到。
/// 挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
/// 对得上的那一份报给读者（[`PoolReader::note_that_the_bytes_read_self_verify`]），读成而对不上的那一份也报
/// （[`PoolReader::note_that_the_bytes_read_fail_self_verification`]：可写挂载这次挂载共用的那一份记进读坏账、丢掉那几块）。
pub(crate) fn read_unit_via_locations<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    locations: &[LocationEntry; 2],
    unit_bytes: usize,
) -> Result<Vec<u8>, RecoveryFailure> {
    for location in locations {
        let Some(bytes) = reader.read(
            location.device,
            location.slot.to_device_offset(),
            unit_bytes,
        ) else {
            continue;
        };
        if crc32_castagnoli(&bytes) == location.unit_checksum {
            reader.note_that_the_bytes_read_self_verify(
                location.device,
                location.slot.to_device_offset(),
                &bytes,
            );
            return Ok(bytes);
        }
        reader.note_that_the_bytes_read_fail_self_verification(
            location.device,
            location.slot.to_device_offset(),
            unit_bytes,
        );
    }
    Err(RecoveryFailure::UnitUnreadable {
        slot: locations[0].slot,
    })
}

/// 进中央映射的树节点分两类（D19（块指针的结构与宽度预算） 已定项 8：映射装码 1、码 2、码 3）：码 2 索引节点
/// （extent 树根、inode 树根、分配记录树与记账树的根）与码 3 打包记录单元（inode 叶容器）。类定两件事：映射 key 里的
/// 类标签与读几个字节。豁免三类（映射树根、树表、实例表）不进映射，不走这里——它们的父指针里的位置条目是权威。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MappedTreeNodeClass {
    IndexNode,
    PackedRecordUnit,
}

impl MappedTreeNodeClass {
    fn unit_class_tag(self) -> u8 {
        match self {
            MappedTreeNodeClass::IndexNode => UNIT_CLASS_INDEX_NODE,
            MappedTreeNodeClass::PackedRecordUnit => UNIT_CLASS_PACKED,
        }
    }

    fn unit_bytes(self) -> usize {
        match self {
            MappedTreeNodeClass::IndexNode => usize::try_from(NODE_BYTES).expect("16384"),
            MappedTreeNodeClass::PackedRecordUnit => {
                usize::try_from(DATA_UNIT_BYTES).expect("32768")
            }
        }
    }
}

/// 查中央映射的口子：给一个 27 字节的映射 key，交回映射里它的两条位置条目（没有这个 key 是 `Ok(None)`）。
/// 挂载态查打开时整片读进来的条目，冷走读查映射树根——查法不同，回退那一步是同一条。
pub(crate) type CentralMappingLocationsOfKey<'lookup> =
    dyn Fn(&[u8]) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> + 'lookup;

/// 按父指针里的位置提示读一个进映射的树节点；两条提示都读不出（读不回字节，或整单元校验和对不上）时，
/// 按这个节点的映射 key 查中央映射、照映射里的两条位置条目再读一次（D19（块指针的结构与宽度预算） 已定项 8：
/// 豁免三类之外的单元位置提示读不出时一律经映射回退）。挂载态的读（`crate::mounted_read`）与冷启动走读走这同一条。
/// 每回退一次 `stale_location_hint_hops` 加一（已定项 5 硬规则 3：提示过期的多跳要有运行时观测点）。
///
/// # Errors
/// 提示读不出、映射里没有这个 key ⇒ `MappingMiss`（带提示的槽）；映射落点也读不出 ⇒ `MappingStillUnreadable`
/// （带映射落点的槽）——与冷走读里数据单元那一条回退同两个成员。查映射本身判红的（映射树根读不出、条目窄于字段表），原样交回。
pub(crate) fn read_mapped_tree_node_via_hint_then_central_mapping(
    reader: &dyn PoolReader,
    pointer: &NodePointer,
    class: MappedTreeNodeClass,
    central_mapping_locations_of_key: &CentralMappingLocationsOfKey<'_>,
    stale_location_hint_hops: &mut usize,
) -> Result<Vec<u8>, RecoveryFailure> {
    let unit_bytes = class.unit_bytes();
    if let Ok(bytes) = read_unit_via_locations(reader, &pointer.locations, unit_bytes) {
        return Ok(bytes);
    }
    *stale_location_hint_hops += 1;
    let mapping_key = mapping_key_for_node(class.unit_class_tag(), *pointer);
    let Some(mapped_locations) = central_mapping_locations_of_key(&mapping_key)? else {
        return Err(RecoveryFailure::MappingMiss {
            slot: pointer.locations[0].slot,
        });
    };
    read_unit_via_locations(reader, &mapped_locations, unit_bytes).map_err(|_still_unreadable| {
        RecoveryFailure::MappingStillUnreadable {
            slot: mapped_locations[0].slot,
        }
    })
}

/// 在映射树根兼叶的条目里找一个 key 的两条位置条目：逐条解（条目宽是盘上的值，窄于 55 的报
/// `EntryNarrowerThanItsFieldTable`，不按固定偏移切），同 key 多条取最后一条。冷走读查映射只走这一个函数。
fn central_mapping_locations_among_entries(
    mapping_entries: &[Vec<u8>],
    mapping_key: &[u8],
) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
    let mut mapped = None;
    for entry in mapping_entries {
        let (candidate_key, locations) =
            parse_mapping_entry(entry).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "映射条目",
                entry_bytes: entry.len(),
                field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
            })?;
        if candidate_key == mapping_key {
            mapped = Some(locations);
        }
    }
    Ok(mapped)
}

/// 按数据指针里的位置提示读一个码 1 数据单元；两条提示都读不出（读不回字节，或整单元校验和对不上）时，按它的码 1 映射 key
/// 查中央映射、照映射里的两条位置条目再读一次（D19（块指针的结构与宽度预算） 已定项 8：豁免三类之外的单元位置提示读不出时一律经映射回退）。
/// 冷启动走读与从盘上重建上一版走这同一条；每回退一次 `data_unit_stale_location_hint_hops` 加一（已定项 5 硬规则 3 的观测点）。
/// 数据单元读不出不是这里的错：交回 [`DataUnitReadThroughTheCentralMapping`] 的两种读不出，由调用方按各自的条款处置。
///
/// # Errors
/// 查映射本身判红的（映射树根读不出、条目窄于字段表），原样交回。
fn read_data_unit_via_hint_then_central_mapping(
    reader: &dyn PoolReader,
    data_pointer: &DataPointer,
    central_mapping_locations_of_key: &CentralMappingLocationsOfKey<'_>,
    data_unit_stale_location_hint_hops: &mut usize,
) -> Result<DataUnitReadThroughTheCentralMapping, RecoveryFailure> {
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    if let Ok(bytes) = read_unit_via_locations(reader, &data_pointer.locations, data_unit_bytes) {
        return Ok(DataUnitReadThroughTheCentralMapping::Content(bytes));
    }
    *data_unit_stale_location_hint_hops += 1;
    let mapping_key = mapping_key_for_data(data_pointer.head, data_pointer.write_order);
    let Some(mapped_data_unit_locations) = central_mapping_locations_of_key(&mapping_key)? else {
        return Ok(
            DataUnitReadThroughTheCentralMapping::MissingFromTheMapping {
                hint_slot: data_pointer.locations[0].slot,
            },
        );
    };
    Ok(
        match read_unit_via_locations(reader, &mapped_data_unit_locations, data_unit_bytes) {
            Ok(bytes) => DataUnitReadThroughTheCentralMapping::Content(bytes),
            Err(_still_unreadable) => {
                DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation {
                    mapped_slot: mapped_data_unit_locations[0].slot,
                }
            }
        },
    )
}

/// 一个数据单元按位置提示、再经中央映射读下来的结局（[`read_data_unit_via_hint_then_central_mapping`]）。
/// 读不出的两种各自带着冷走读报错要点名的那个槽；冷走读把它们报成错，从盘上重建上一版照抄位置项、不读内容。
enum DataUnitReadThroughTheCentralMapping {
    Content(Vec<u8>),
    /// 提示读不出，映射里没有它的 key。
    MissingFromTheMapping {
        hint_slot: SlotNumber,
    },
    /// 提示读不出，映射落点也读不出。
    UnreadableAtTheMappedLocation {
        mapped_slot: SlotNumber,
    },
}

impl DataUnitReadThroughTheCentralMapping {
    /// 冷走读的读法：读不出就是这一步走不下去。
    ///
    /// # Errors
    /// 映射里没有 ⇒ `MappingMiss`（带提示的槽）；映射落点也读不出 ⇒ `MappingStillUnreadable`（带映射落点的槽）。
    fn into_content(self) -> Result<Vec<u8>, RecoveryFailure> {
        match self {
            DataUnitReadThroughTheCentralMapping::Content(bytes) => Ok(bytes),
            DataUnitReadThroughTheCentralMapping::MissingFromTheMapping { hint_slot } => {
                Err(RecoveryFailure::MappingMiss { slot: hint_slot })
            }
            DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation { mapped_slot } => {
                Err(RecoveryFailure::MappingStillUnreadable { slot: mapped_slot })
            }
        }
    }
}

/// 从盘上重建上一版、读一条根的分配记录时用的中央映射树（自举豁免，只按父指针里的位置条目读，D19（块指针的结构与宽度预算） 已定项 8）：
/// 到第一次有提示读不出、要经映射回退时，或重建走到映射树那一步时才读，读过一次就留着——每个节点的盘上字节连同形状，重建要把字节照抄进上一版。
/// 多层时整棵读回来（D8（核心索引结构） 已定项 11），核到哪一步看 `judgement`：从盘上重建上一版照冷走读判全
/// （`EveryHeaderAgainstItsReference`：树 ID、key 宽、出生身份、fsid、层级、区间与 key 次序，代码审阅第 24 / 33 条），
/// 读别的根的分配记录（影子账、抬 F 的上限）只按「拼得成一棵树」核（`OnlyWhatTheShapeNeeds`）。
/// 不提前读：提示都读得出的镜像上，读序与报错的次序照旧。
struct CentralMappingTreeWithBytesReadOnFirstUse<'reader> {
    reader: &'reader dyn PoolReader,
    root: &'reader RootRecord,
    judgement: CodeTwoTreeHeaderJudgement,
    tree: OnceCell<CodeTwoTreeReadFromDisk>,
}

impl<'reader> CentralMappingTreeWithBytesReadOnFirstUse<'reader> {
    fn new(
        reader: &'reader dyn PoolReader,
        root: &'reader RootRecord,
        judgement: CodeTwoTreeHeaderJudgement,
    ) -> Self {
        Self {
            reader,
            root,
            judgement,
            tree: OnceCell::new(),
        }
    }

    fn tree(&self) -> Result<&CodeTwoTreeReadFromDisk, RecoveryFailure> {
        if let Some(read) = self.tree.get() {
            return Ok(read);
        }
        let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
        let tree = read_code_two_tree(
            &self.root.mapping_root,
            &MultiLevelCodeTwoTree::CentralMapping
                .read_expectation(self.root.mapping_root.head.birth_tree),
            self.judgement,
            self.root,
            unit_filesystem_identifier(&self.root.filesystem_identifier),
            &mut |pointer: &NodePointer| {
                read_unit_via_locations(self.reader, &pointer.locations, node_bytes)
            },
        )?;
        Ok(self.tree.get_or_init(|| tree))
    }

    fn locations_of_key(
        &self,
        mapping_key: &[u8],
    ) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
        central_mapping_locations_among_entries(
            &self.tree()?.leaf_entries_in_key_order,
            mapping_key,
        )
    }

    fn into_tree(self) -> Result<CodeTwoTreeReadFromDisk, RecoveryFailure> {
        self.tree()?;
        Ok(self
            .tree
            .into_inner()
            .expect("上一行刚把映射树读进来，读不出已经返回了"))
    }
}

/// 一个系统配置槽读回来之后可择不可择（[`choose_system_configuration`] 按它分流）。封闭集合，`match` 不写通配臂。
enum SystemConfigurationSlotReading {
    /// 三关都过、S 在区间里：这一槽可择。
    Mountable(SystemConfiguration),
    /// 读不到，或 magic / 整槽校验和不过，或节点大小不是格式常量（当自证不过，C840）：这一槽不可择，换一槽换一盘还可以试。
    NotSelfDescribing,
    /// magic 与整槽校验和都过、incompat 位图不认识：这一槽同样不可择，记下它，池里一份可择的都没有时报布局不认识。
    IncompatBitsNotRecognized(IncompatBitmap),
}

impl SystemConfigurationSlotReading {
    fn into_mountable(self) -> Option<SystemConfiguration> {
        match self {
            Self::Mountable(system_configuration) => Some(system_configuration),
            Self::NotSelfDescribing | Self::IncompatBitsNotRecognized(_) => None,
        }
    }

    fn incompat_bitmap_not_recognized(&self) -> Option<IncompatBitmap> {
        match self {
            Self::IncompatBitsNotRecognized(incompat_bitmap) => Some(*incompat_bitmap),
            Self::Mountable(_) | Self::NotSelfDescribing => None,
        }
    }

    /// 可择的这一槽记的固定结构槽距；自证不过或布局不认识的槽里记的不能信。
    fn recorded_fixed_structure_slot_spacing(&self) -> Option<u64> {
        match self {
            Self::Mountable(system_configuration) => Some(u64::from(
                system_configuration
                    .immutable
                    .sizes
                    .fixed_structure_slot_spacing,
            )),
            Self::NotSelfDescribing | Self::IncompatBitsNotRecognized(_) => None,
        }
    }
}

/// 系统配置里这个读者不收的那一个池级不可变字段与它的值（[`RecoveryFailure::SystemConfigurationValueRefused`]）。封闭集合，`match` 不写通配臂。
/// 上下界取 mkfs 判几何用的同一组条款（`make_filesystem::check_geometry`），读者这一侧的判法条款没有写，实审 A3a 的报告给了要补的原句。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemConfigurationValueOutsideWhatThisReaderAccepts {
    /// 格式版本不是这个读者写的那一版（`system_configuration::FORMAT_VERSION`）。
    FormatVersionNotRecognized { format_version: u16 },
    /// 加密类型不是 0（关）：加密不进第一个可运行版本（D9（加密） 已定项 10），1、2 是登记过的算法、别的码不认识，都不收。
    EncryptionTypeNotOff { encryption_type: u8 },
    /// 固定结构槽距落在格式允许的区间之外：不小于 4096、槽 1 整槽落在同一槽自述的根环起点之前（D22（单元原子性怎么合成） 已定项 16
    /// 第 5 句；根环起点字段 × 16384 − 4096 是上界，[`largest_fixed_structure_slot_spacing_under_the_root_ring_base`]；根环起点不到一个
    /// 系统配置槽宽时一档槽距都不收）。槽距是根槽宽的上界，这一条先于根槽宽判；也先于根环起点等不等第一版常量那一判，
    /// 根环起点落在槽 1 之前的槽报在这里（与 checker 的 I-7.13 报同一格）。
    FixedStructureSlotSpacingOutsideTheFormatRange { fixed_structure_slot_spacing: u32 },
    /// `physical_block_size`（根槽宽，D22（单元原子性怎么合成） 已定项 2）装不下根记录（457 字节，已定项 7），或大于固定结构槽距
    /// （槽 j 在区域起点 + j × 槽距，已定项 16）。读根槽按它开缓冲，不设上界时一个盘上的 4 字节就定得了每一次读的内存。
    PhysicalBlockSizeOutsideTheRootSlotBounds { physical_block_size: u32 },
    /// 同一槽记着的根环起点（偏移 371 的 8 字节，16 KiB 槽号）不是第一版写的那个（64 = 1 MiB，
    /// `system_configuration::ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION`）：core 读写根环按格式常量走（`root_ring::region_start`），
    /// 字段与常量不等时两边读的不是同一个环；条款定的是按字段读、第一版只收常量，不等整池拒（D22（单元原子性怎么合成） 已定项 16 第 1 句，
    /// 用户 2026-09-27 定「读字段，不等于常量就整池拒」；代码三方 m2-closeout-code-r2「core 与 checker 对系统配置几何字段的读法」）。
    RootRingBaseNotTheFirstVersionSlot {
        recorded_root_ring_base_slot: SlotNumber,
        first_version_root_ring_base_slot: SlotNumber,
    },
    /// 同一槽记着的 journal 环起点（偏移 325 的 8 字节，16 KiB 槽号）不是第一版写的那个（1024，
    /// `system_configuration::JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION`）：core 读写 journal 环按格式常量走，处置同
    /// [`Self::RootRingBaseNotTheFirstVersionSlot`]（同一条款、同一次用户定案）。
    JournalRingStartNotTheFirstVersionSlot {
        recorded_journal_ring_start_slot: SlotNumber,
        first_version_journal_ring_start_slot: SlotNumber,
    },
    /// journal 环长装不下 F 条记录（在飞上限 = 环槽数 ÷ F 是 0，D23（journal 的角色与格式） 已定项 18），在飞上限装不进系统配置里它那 4 字节
    /// （mkfs 的 `JournalInFlightRecordLimitWiderThanItsFourByteField`；不拒的话挂载之后轮换系统配置槽时写这 4 字节那一句 panic），
    /// 或环的末端越过同一槽记着的单元区起点（偏移 417 的 8 字节；与 checker 同一个判法，C475（非默认环长下单元区起点取编译期常量）之后
    /// 单元区起点随环长走、不再有编译期的上界）。写记录按它取模，不设界时一个盘上的 8 字节就定得了算术溢不溢出；扫环逐槽列偏移只列到
    /// 那块盘的末尾（[`scan_journal`]）。
    JournalRingBytesOutsideTheSupportedRange { journal_ring_bytes: u64 },
    /// 同一槽记着的单元区起始槽号（偏移 417 的 8 字节）不是环长现算的那个（journal 环末尾的下一个槽，D3（空间分配） 已定项 10 ④
    /// 「第一版 = journal 环末尾的下一个槽」）：挂载建空闲图、恢复判分配记录落点都按环长现算的起点走（`allocator::UnitAreaStart`，
    /// 由环长算只有 `journal::slot_after_the_journal_ring` 一处），两个数不一样时按哪一个条款没写，第一版不收。
    UnitAreaStartNotTheSlotAfterTheJournalRing {
        recorded_unit_area_start_slot: SlotNumber,
        slot_after_the_journal_ring: SlotNumber,
    },
    /// 单元区起点（环长现算、与偏移 417 那 8 字节相等）不落在 64 槽聚簇段边界上：这样的单元区怎么分段条款没写，第一版不支持
    /// （`allocator::UnitAreaStart` 的不变量；mkfs 同一格拒成 `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`）。
    UnitAreaStartOffTheClusterSegmentBoundaryUnsupported { unit_area_start_slot: SlotNumber },
    /// 同一槽记着的安全系数 F（偏移 357 的 4 字节）不是这个读者写的那个（`singlefs_format::JOURNAL_SAFETY_FACTOR`，第一版 3）。
    /// I-8.1（环几何够大） 的三条等式之一，core 与 checker 同判（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4 行）：
    /// 恢复取前缀、发布切记录都按格式常量算在飞上限，盘上的 F 与它不等时两边说的不是同一个环几何。F ≥ 2 那一半落在格式常量上
    /// （`system_configuration` 里的编译期断言）。
    JournalSafetyFactorNotTheFirstVersionValue {
        recorded_safety_factor: u32,
        first_version_safety_factor: u64,
    },
    /// 同一槽记着的在飞记录数上限（偏移 345 的 4 字节）不等于按同一槽的环长现算的 ⌊环长 ÷ 4096 ÷ F⌋
    /// （`system_configuration::journal_in_flight_record_limit`；I-8.1（环几何够大））。
    JournalInFlightRecordLimitNotRecomputedFromTheRingLength {
        recorded_in_flight_record_limit: u32,
        recomputed_in_flight_record_limit: u64,
    },
    /// 同一槽记着的最坏占用（偏移 349 的 8 字节）不等于现算的在飞记录数上限 × 4096（I-8.1（环几何够大））。
    JournalWorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes {
        recorded_worst_case_occupancy_bytes: u64,
        recomputed_worst_case_occupancy_bytes: u64,
    },
}

/// 系统配置槽里格式版本那 2 字节的偏移：紧跟 magic（`system_configuration::SystemConfiguration::to_slot` 的写法）。
const SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET: usize =
    crate::system_configuration::SYSTEM_CONFIGURATION_MAGIC.len();
/// 系统配置槽里加密类型那 1 字节的偏移：整槽校验和 32 之后是 MAC 16、nonce 水位 12、KDF 标识 4，再是它（字段表偏移 219）。
const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET: usize =
    crate::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32 + 16 + 12 + 4;
/// 加密类型登记表里的「关」（未加密，D9（加密） 已定项 10；D22（单元原子性怎么合成） 已定项 17）。
const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF: u8 = 0;

/// 格式允许的最大固定结构槽距：槽 1 整槽落在第一版根环基址（区域 0 的起点）之前。逐档找槽 1 用它：那时一槽可择的都没有，
/// 没有哪一槽自述的根环起点可读。读者判一槽的槽距用那一槽自述的根环起点（[`largest_fixed_structure_slot_spacing_under_the_root_ring_base`]）。
fn largest_fixed_structure_slot_spacing_the_format_allows() -> u64 {
    region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES
}

/// 一槽自述的根环起点下格式允许的最大固定结构槽距：槽 1 整槽落在根环基址（该字段 × 16384）之前（D22（单元原子性怎么合成） 已定项 16
/// 第 1、5 句）。根环起点是盘上读来的 8 字节：乘出来溢出时上界取 `u64::MAX`（基址在任何设备偏移之外，槽 1 落在它之前）；
/// 基址不到一个系统配置槽宽时一档都不收（`None`）。
fn largest_fixed_structure_slot_spacing_under_the_root_ring_base(
    root_ring_base_slot: SlotNumber,
) -> Option<u64> {
    root_ring_base_slot.0.checked_mul(SLOT_BYTES).map_or(
        Some(u64::MAX),
        |root_ring_base_in_bytes| {
            root_ring_base_in_bytes.checked_sub(SYSTEM_CONFIGURATION_SLOT_BYTES)
        },
    )
}

/// 一槽自证得过的系统配置，逐个判这个读者收不收它的池级不可变字段（[`SystemConfigurationValueOutsideWhatThisReaderAccepts`] 各成员的出处）。
/// 次序：格式版本、加密类型、槽距、根槽宽、根环起点、journal 环起点、环长、单元区起点——根槽宽的上界是槽距，所以槽距先判；
/// 槽距的上界取同一槽自述的根环起点，所以根环起点落在槽 1 之前的那一格报成槽距越界、与 checker 的 I-7.13 同一格，之后才判它等不等
/// 第一版常量；环末端对单元区起点那一判先于起点是不是环长现算的那个，与 checker 的环长判法（`journal_ring_bytes_lie_in_the_supported_range`）报同一格。
fn system_configuration_values_this_reader_accepts(
    slot: &[u8],
    system_configuration: &SystemConfiguration,
) -> Result<(), SystemConfigurationValueOutsideWhatThisReaderAccepts> {
    let format_version = u16::from_le_bytes([
        slot[SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET],
        slot[SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET + 1],
    ]);
    if format_version != crate::system_configuration::FORMAT_VERSION {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::FormatVersionNotRecognized {
                format_version,
            },
        );
    }
    let encryption_type = slot[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET];
    if encryption_type != SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::EncryptionTypeNotOff {
                encryption_type,
            },
        );
    }
    let sizes = &system_configuration.immutable.sizes;
    let fixed_structure_slot_spacing = u64::from(sizes.fixed_structure_slot_spacing);
    if fixed_structure_slot_spacing < FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
        || largest_fixed_structure_slot_spacing_under_the_root_ring_base(
            system_configuration.immutable.root_ring_base_slot,
        )
        .is_none_or(|largest_spacing| fixed_structure_slot_spacing > largest_spacing)
    {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange {
                fixed_structure_slot_spacing: sizes.fixed_structure_slot_spacing,
            },
        );
    }
    if u64::from(sizes.physical_block_size) < ROOT_RECORD_BYTES
        || sizes.physical_block_size > sizes.fixed_structure_slot_spacing
    {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::PhysicalBlockSizeOutsideTheRootSlotBounds {
                physical_block_size: sizes.physical_block_size,
            },
        );
    }
    let recorded_root_ring_base_slot = system_configuration.immutable.root_ring_base_slot;
    if recorded_root_ring_base_slot != ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingBaseNotTheFirstVersionSlot {
                recorded_root_ring_base_slot,
                first_version_root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,
            },
        );
    }
    let recorded_journal_ring_start_slot = system_configuration.immutable.journal_ring_start_slot;
    if recorded_journal_ring_start_slot != JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingStartNotTheFirstVersionSlot {
                recorded_journal_ring_start_slot,
                first_version_journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,
            },
        );
    }
    let recorded_unit_area_start_slot = unit_area_start_slot_recorded_in_the_slot(slot);
    // 环末端 = 环起点 + 环长；单元区起点 × 16384。两个都是盘上读来的数：乘加溢出时环末端算越界，起点乘出来溢出时它在任何设备偏移之外。
    let journal_ring_end_in_bytes =
        (JOURNAL_RING_START_SLOT * SLOT_BYTES).checked_add(sizes.journal_ring_bytes);
    let journal_ring_ends_by_the_recorded_unit_area_start =
        journal_ring_end_in_bytes.is_some_and(|journal_ring_end| {
            recorded_unit_area_start_slot
                .0
                .checked_mul(SLOT_BYTES)
                .is_none_or(|unit_area_start| journal_ring_end <= unit_area_start)
        });
    if journal_in_flight_record_limit(sizes.journal_ring_bytes) == 0
        || !journal_in_flight_record_limit_fits_its_four_byte_field(sizes.journal_ring_bytes)
        || !journal_ring_ends_by_the_recorded_unit_area_start
    {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange {
                journal_ring_bytes: sizes.journal_ring_bytes,
            },
        );
    }
    let slot_after_the_ring = slot_after_the_journal_ring(sizes.journal_ring_bytes);
    if recorded_unit_area_start_slot != slot_after_the_ring {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitAreaStartNotTheSlotAfterTheJournalRing {
                recorded_unit_area_start_slot,
                slot_after_the_journal_ring: slot_after_the_ring,
            },
        );
    }
    UnitAreaStart::following_the_journal_ring(sizes.journal_ring_bytes).map_err(|off_the_boundary| {
        SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported {
            unit_area_start_slot: off_the_boundary.slot_after_the_journal_ring,
        }
    })?;
    journal_ring_geometry_fields_recompute_from_the_ring_length(slot, sizes.journal_ring_bytes)
}

/// I-8.1（环几何够大） 的三条等式，挂载读系统配置时复算（core 同判，`research/prompts/m3-inv-a-r1-main-verification.md`
/// 第二节第 4 行、第三节）：F = 格式常量、在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋、最坏占用 = 在飞上限 × 4096，按这个次序判，第一个不等的报出来。
/// 放在 [`system_configuration_values_this_reader_accepts`] 的最后：环长与单元区起点不收的槽先按那几格报，这三个字段是环长的函数，
/// 环长本身不收时比它们没有意义。
fn journal_ring_geometry_fields_recompute_from_the_ring_length(
    slot: &[u8],
    journal_ring_bytes: u64,
) -> Result<(), SystemConfigurationValueOutsideWhatThisReaderAccepts> {
    let recorded = journal_ring_geometry_fields_recorded_in_the_slot(slot);
    if u64::from(recorded.safety_factor) != JOURNAL_SAFETY_FACTOR {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalSafetyFactorNotTheFirstVersionValue {
                recorded_safety_factor: recorded.safety_factor,
                first_version_safety_factor: JOURNAL_SAFETY_FACTOR,
            },
        );
    }
    let recomputed_in_flight_record_limit = journal_in_flight_record_limit(journal_ring_bytes);
    if u64::from(recorded.in_flight_record_limit) != recomputed_in_flight_record_limit {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalInFlightRecordLimitNotRecomputedFromTheRingLength {
                recorded_in_flight_record_limit: recorded.in_flight_record_limit,
                recomputed_in_flight_record_limit,
            },
        );
    }
    let recomputed_worst_case_occupancy_bytes =
        recomputed_in_flight_record_limit * JOURNAL_RECORD_BYTES;
    if recorded.worst_case_occupancy_bytes != recomputed_worst_case_occupancy_bytes {
        return Err(
            SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalWorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes {
                recorded_worst_case_occupancy_bytes: recorded.worst_case_occupancy_bytes,
                recomputed_worst_case_occupancy_bytes,
            },
        );
    }
    Ok(())
}

/// 择到的系统配置下的单元区起点：journal 环末尾的下一个槽（`allocator::UnitAreaStart::following_the_journal_ring`）。
/// 挂载建空闲图、判分配记录与被抛弃根的落点在不在单元区里，都按它（C475（非默认环长下单元区起点取编译期常量），实审 A3b）。
///
/// # Panics
/// 环长算出的起点不在聚簇段边界上：`system_configuration` 必须是 [`choose_system_configuration`] 交回的——择的时候
/// `system_configuration_values_this_reader_accepts` 判过起点在段边界上、且与槽里偏移 417 那 8 字节相等。
pub(crate) fn unit_area_start_of_the_chosen_system_configuration(
    system_configuration: &SystemConfiguration,
) -> UnitAreaStart {
    UnitAreaStart::following_the_journal_ring(system_configuration.immutable.sizes.journal_ring_bytes)
        .expect("择系统配置时 system_configuration_values_this_reader_accepts 判过环长算出的单元区起点在聚簇段边界上")
}

/// 这个池的单元区起点：先择系统配置（[`choose_system_configuration`]），再按它的环长算。给不带单元区起点的那几个公开入口
/// （[`rebuild_version`]、[`walk_to_file`]、[`allocation_records_under_root`]、[`allocation_records_of_version_without_file`]）用：
/// 它们的调用方多是用例与实验装置，手里没有择好的系统配置；挂载与冷走读手里有，走带起点的那一份，不多读系统配置槽。
///
/// # Errors
/// 择不到系统配置（[`choose_system_configuration`] 的各个成员）。
fn unit_area_start_of_the_pool(reader: &dyn PoolReader) -> Result<UnitAreaStart, RecoveryFailure> {
    choose_system_configuration(reader).map(|system_configuration| {
        unit_area_start_of_the_chosen_system_configuration(&system_configuration)
    })
}

/// 一个槽读回来之后分五路：读不到 / 自证不过 ⇒ 不可择（换一槽换一盘还可以试）；自证得过、incompat 位图不认识 ⇒ 同样不可择、
/// 另记一笔；自述的每区槽数 S 越界、或别的池级不可变字段这个读者不收 ⇒ 整池拒绝挂载，把点名的成员交回去；都过 ⇒ 可择。
///
/// 单拎成一个函数是因为 [`choose_system_configuration`] 每块盘要做两次（槽 0、槽 1），
/// 而两次的分流必须一模一样：其中一次把越界悄悄当成「这一槽不可择」，越界的池就会靠另一槽挂上去。
fn mountable_slot_or_refusal(
    device: DeviceIdentity,
    bytes: Option<Vec<u8>>,
) -> Result<SystemConfigurationSlotReading, RecoveryFailure> {
    let Some(bytes) = bytes else {
        return Ok(SystemConfigurationSlotReading::NotSelfDescribing);
    };
    match SystemConfiguration::parse_slot(&bytes) {
        Ok(system_configuration) => {
            system_configuration_values_this_reader_accepts(&bytes, &system_configuration)
                .map_err(|value| RecoveryFailure::SystemConfigurationValueRefused {
                    device,
                    value,
                })?;
            Ok(SystemConfigurationSlotReading::Mountable(
                system_configuration,
            ))
        }
        // 节点大小不是格式常量的槽当自证不过（C840，用户 2026-10-15 定）：与整槽校验和不过同一路，换一槽换一盘还可以试。
        Err(
            SystemConfigurationSlotRefusal::NotSelfDescribing
            | SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant { .. },
        ) => Ok(SystemConfigurationSlotReading::NotSelfDescribing),
        Err(SystemConfigurationSlotRefusal::IncompatBitsNotRecognized { incompat_bitmap }) => Ok(
            SystemConfigurationSlotReading::IncompatBitsNotRecognized(incompat_bitmap),
        ),
        Err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange(out_of_range)) => {
            Err(RecoveryFailure::RootRingSlotsPerRegionOutOfRange {
                device,
                out_of_range,
            })
        }
    }
}

/// 逐档找槽 1 时相邻两档槽距之差：槽距 = 4096 向上取整到 io_min 的整数倍（D2（RAID 条带策略） 已定项 19），
/// io_min 是逻辑块宽的整数倍、逻辑块宽最小 512 字节（Linux 块层的扇区），所以格式允许的槽距都是 512 的整数倍。
const SLOT_SPACING_SEARCH_STEP_BYTES: u64 = 512;

/// 全池没有一个槽 0 可择时找一块盘的槽 1：槽距无处可读，在格式允许的每一档槽距上各读一槽，
/// 只收可择、且自己记的固定结构槽距正好等于它所在偏移的那一槽；几档都收得到时取世代号最大的
/// （D22（单元原子性怎么合成） 已定项 16「择槽取校验和过且世代号最大的」，逐盘计），世代号相同取偏移小的。
/// 各盘择到的之间 fsid 与设备数要对得上，由 [`choose_system_configuration`] 照常判。
///
/// 格式允许的槽距：不小于 4096（[`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`]）、[`SLOT_SPACING_SEARCH_STEP_BYTES`] 的整数倍，
/// 槽 1 整槽落在根环基址（[`region_start`] 的区域 0，1 MiB）之前——mkfs 判固定结构两两不重叠（`make_filesystem::check_geometry`）。
/// 上下界之间 (1 MiB − 4096 − 4096) ÷ 512 + 1 = 2033 档，每档读一次。
///
/// 风险：同一块盘上更早一次 mkfs 用别的槽距写下的槽 1，这一次 mkfs 不清（它只写本次的两槽），它自证得过、记的槽距也等于它所在的偏移，
/// 与本池的槽 1 分不出。它的世代号更大时择到的就是它：fsid 不同时各盘择到的对不上、整池报 [`RecoveryFailure::SystemConfigurationsDisagree`]；
/// fsid 相同（调用方拿同一个 fsid 重做 mkfs）时按旧池的几何挂。只按最小槽距 4096 试一档时，旧池槽距是 4096 那一格上同一个风险也在。
///
/// 一档读回来的分流与槽 0、槽 1 同一个（[`mountable_slot_or_refusal`]）。一槽可择的都没有时，交回试到的第一个 incompat 位图不认识的槽
/// （按偏移升序；它的槽距记在哪一格读不出来，不核偏移），再没有就是自证不过。
///
/// # Errors
/// 试到的某一槽自证得过、每区槽数 S 越界 ⇒ [`RecoveryFailure::RootRingSlotsPerRegionOutOfRange`]，整池拒，与槽 0、槽 1 同一个判法。
fn slot_one_found_by_trying_every_slot_spacing_the_format_allows(
    reader: &dyn PoolReader,
    device: DeviceIdentity,
) -> Result<SystemConfigurationSlotReading, RecoveryFailure> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let largest_spacing = largest_fixed_structure_slot_spacing_the_format_allows();
    let mut highest_generation_found: Option<SystemConfiguration> = None;
    let mut first_slot_with_incompat_bits_not_recognized: Option<IncompatBitmap> = None;
    // 迭代次数的上界是档数（2033）；跨轮带着的是至今世代号最大的那一槽与第一个 incompat 位图，都只换不删。
    // 这一档槽距下槽 1 的偏移就是槽距本身（槽 i 在 i × 槽距）。
    for candidate_spacing in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES..=largest_spacing)
        .step_by(usize::try_from(SLOT_SPACING_SEARCH_STEP_BYTES).expect("512"))
    {
        match mountable_slot_or_refusal(
            device,
            reader.read(device, DeviceOffsetInBytes(candidate_spacing), slot_bytes),
        )? {
            SystemConfigurationSlotReading::Mountable(system_configuration) => {
                let recorded_spacing = u64::from(
                    system_configuration
                        .immutable
                        .sizes
                        .fixed_structure_slot_spacing,
                );
                if recorded_spacing != candidate_spacing {
                    continue;
                }
                if highest_generation_found.is_none_or(|found| {
                    system_configuration.quantities.slot_generation
                        > found.quantities.slot_generation
                }) {
                    highest_generation_found = Some(system_configuration);
                }
            }
            SystemConfigurationSlotReading::NotSelfDescribing => {}
            SystemConfigurationSlotReading::IncompatBitsNotRecognized(incompat_bitmap) => {
                first_slot_with_incompat_bits_not_recognized =
                    first_slot_with_incompat_bits_not_recognized.or(Some(incompat_bitmap));
            }
        }
    }
    Ok(
        match (
            highest_generation_found,
            first_slot_with_incompat_bits_not_recognized,
        ) {
            (Some(system_configuration), _) => {
                SystemConfigurationSlotReading::Mountable(system_configuration)
            }
            (None, Some(incompat_bitmap)) => {
                SystemConfigurationSlotReading::IncompatBitsNotRecognized(incompat_bitmap)
            }
            (None, None) => SystemConfigurationSlotReading::NotSelfDescribing,
        },
    )
}

/// 每盘两槽：槽 0 在偏移 0；槽 1 在固定结构槽距处（D22（单元原子性怎么合成） 已定项 16）。槽距是池级字段（两盘四槽同值）：
/// 先读遍每块盘的槽 0，一块盘的槽 0 可择就按它自己记的槽距找它的槽 1；自己的槽 0 不可择，按池里第一块槽 0 可择的盘记的槽距找；
/// 全池没有一个槽 0 可择时槽距无处可读，在格式允许的每一档槽距上试（[`slot_one_found_by_trying_every_slot_spacing_the_format_allows`]，
/// 那里写着它的风险：更早一次 mkfs 留下的槽 1 分不出来）。
/// 一块盘两个槽都无效时**跳过这块盘**，接着看别的盘：系统配置每盘放一份买的就是这份冗余
/// （D22（单元原子性怎么合成） 已定项 8 第 1 条；E87 的 8 种失效组合里「掉了那块盘」这一格要可挂）。
///
/// # Errors
/// 池里每块盘的两个槽都无效（[`RecoveryFailure::NoValidSystemConfiguration`]）；
/// 池里一槽可择的都没有、而至少一槽自证得过只是 incompat 位图不认识（[`RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized`]，
/// 先于上一条判）；
/// 交得出系统配置的几块盘之间 fsid 或设备数对不上（[`RecoveryFailure::SystemConfigurationsDisagree`]）；
/// 池里一块盘都没有（[`RecoveryFailure::NoDeviceInThePool`]）；
/// 某一槽自证得过、而自述的每区槽数 S 越界（[`RecoveryFailure::RootRingSlotsPerRegionOutOfRange`]）——
/// **这一条是挂载时的区间判**：S 越界的池在这里就拒，后面一步不走、盘上一个字节不动。
pub fn choose_system_configuration(
    reader: &dyn PoolReader,
) -> Result<SystemConfiguration, RecoveryFailure> {
    choose_system_configuration_naming_devices_without_a_mountable_slot(reader)
        .map(|choice| choice.system_configuration)
}

/// 择到的系统配置，连带池里两个槽都不可择（读不出、自证不过、或 incompat 位图不认识）而被跳过的那几块盘
/// （[`choose_system_configuration_naming_devices_without_a_mountable_slot`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfigurationChoice {
    pub system_configuration: SystemConfiguration,
    /// 两个系统配置槽都不可择的盘，按 [`PoolReader::device_identities`] 的次序；冗余完好时为空。择系统配置时跳过它们、照常择别的盘那一份
    /// （D22（单元原子性怎么合成） 已定项 8 第 1 条），这张表把「冗余被吃掉一份」报给只读与恢复路径的调用方（C463（一块盘的系统配置两份全废之后静默成功），
    /// `research/prompts/m3-ring-r1-main-verification.md` 第二节第 11 行「只读与恢复路径报出来」）。只报、不重写：自愈随那一轮第四节第 3 条定。
    pub devices_without_a_mountable_system_configuration_slot: Vec<DeviceIdentity>,
}

/// 与 [`choose_system_configuration`] 同一个择法、同一组错误（它就是调这一个），另交回两个槽都不可择而被跳过的盘
/// （[`SystemConfigurationChoice::devices_without_a_mountable_system_configuration_slot`]）。
/// 单开一个函数、不改 [`choose_system_configuration`] 的返回类型：后者的调用点遍布 core、harness 与 checker 档，
/// 要这张表的只有只读挂载（`mounted_read::MountedReadOnly`）与想看冗余的恢复调用方。
///
/// # Errors
/// 同 [`choose_system_configuration`]。
pub fn choose_system_configuration_naming_devices_without_a_mountable_slot(
    reader: &dyn PoolReader,
) -> Result<SystemConfigurationChoice, RecoveryFailure> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut chosen: Option<SystemConfiguration> = None;
    let mut devices_without_a_mountable_system_configuration_slot: Vec<DeviceIdentity> = Vec::new();
    let mut first_slot_with_incompat_bits_not_recognized: Option<(DeviceIdentity, IncompatBitmap)> =
        None;
    let mut slot_zero_of_each_device: Vec<(DeviceIdentity, SystemConfigurationSlotReading)> =
        Vec::new();
    for device in reader.device_identities() {
        let slot_zero = mountable_slot_or_refusal(
            device,
            reader.read(device, DeviceOffsetInBytes(0), slot_bytes),
        )?;
        slot_zero_of_each_device.push((device, slot_zero));
    }
    let slot_spacing_recorded_by_the_first_mountable_slot_zero = slot_zero_of_each_device
        .iter()
        .find_map(|(_, slot_zero)| slot_zero.recorded_fixed_structure_slot_spacing());
    for (device, slot_zero) in slot_zero_of_each_device {
        // 自己的槽 0 不可择时它记的槽距不能信，用池里别的盘槽 0 记的；全池一个都没有时逐档试——按 D2（RAID 条带策略） 已定项 19
        // 的式子拿挂载时探得的 io_min 算，要 `PoolReader` 交得出探测值，今天交不出。
        let slot_one = match slot_zero
            .recorded_fixed_structure_slot_spacing()
            .or(slot_spacing_recorded_by_the_first_mountable_slot_zero)
        {
            Some(spacing) => mountable_slot_or_refusal(
                device,
                reader.read(device, DeviceOffsetInBytes(spacing), slot_bytes),
            )?,
            None => slot_one_found_by_trying_every_slot_spacing_the_format_allows(reader, device)?,
        };
        if first_slot_with_incompat_bits_not_recognized.is_none() {
            first_slot_with_incompat_bits_not_recognized = slot_zero
                .incompat_bitmap_not_recognized()
                .or_else(|| slot_one.incompat_bitmap_not_recognized())
                .map(|incompat_bitmap| (device, incompat_bitmap));
        }
        let best_on_device = match (slot_zero.into_mountable(), slot_one.into_mountable()) {
            (None, None) => {
                // 这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上；记下来交给调用方。
                devices_without_a_mountable_system_configuration_slot.push(device);
                continue;
            }
            (Some(only), None) | (None, Some(only)) => only,
            (Some(zero), Some(one)) => {
                if one.quantities.slot_generation > zero.quantities.slot_generation {
                    one
                } else {
                    zero
                }
            }
        };
        match &chosen {
            None => chosen = Some(best_on_device),
            Some(previous) => {
                if previous.immutable.filesystem_identifier
                    != best_on_device.immutable.filesystem_identifier
                    || previous.immutable.device_count != best_on_device.immutable.device_count
                {
                    return Err(RecoveryFailure::SystemConfigurationsDisagree);
                }
            }
        }
    }
    if let Some(system_configuration) = chosen {
        return Ok(SystemConfigurationChoice {
            system_configuration,
            devices_without_a_mountable_system_configuration_slot,
        });
    }
    // 一份可择的都没有：有一槽自证得过、只是布局不认识，就报布局不认识（不是数据坏了）；否则才是每块盘两槽都坏。
    match (
        first_slot_with_incompat_bits_not_recognized,
        devices_without_a_mountable_system_configuration_slot
            .first()
            .copied(),
    ) {
        (Some((device, incompat_bitmap)), _) => Err(
            RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
                first_device_carrying_them: device,
                incompat_bitmap,
            },
        ),
        (None, Some(device)) => Err(RecoveryFailure::NoValidSystemConfiguration {
            first_device_with_no_valid_system_configuration_slot: device,
        }),
        // 一块盘交得出系统配置就落在上面的 `chosen`，一块交不出就记进那张表：两样都空只在池里一块盘都没有时。
        (None, None) => Err(RecoveryFailure::NoDeviceInThePool),
    }
}

/// 根环三个区域全部槽里自证过的根，逐个交给 `visit`（择根与取号共用这一段遍历）。
fn visit_valid_roots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    mut visit: impl FnMut(RootRecord),
) {
    visit_valid_roots_with_ring_slots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |_ring_slot, root| visit(root),
    );
}

/// 同 `visit_valid_roots`，连它读出来的那个根环槽一起交出去：分配器要知道每条根住哪个槽，
/// 才认得出之后哪一次发布盖掉了它（`allocator::RootRingOccupancy`）。读不出、自证不过的槽跳过。
fn visit_valid_roots_with_ring_slots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    mut visit: impl FnMut(RootRingSlot, RootRecord),
) {
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        match read_root_ring_slot(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root) => visit(ring_slot, root),
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => continue,
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}
        }
    }
}

/// 根环三个区域的全部槽，区域号升序、区域内槽号升序（读根环的每一处都按这个次序走）。
#[must_use]
pub fn every_root_ring_slot(immutable_sizes: &SystemImmutableSizes) -> Vec<RootRingSlot> {
    (0..ROOT_RING_REGIONS)
        .flat_map(|region| {
            (0..immutable_sizes.root_ring_slots_per_region.count())
                .map(move |slot| RootRingSlot { region, slot })
        })
        .collect()
}

/// 读一个根环槽读坏了是哪一种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadRootRingSlotReading {
    /// 读不出：设备报错、越界、那块盘不在读者里（`PoolReader::read` 交回空）。
    Unreadable,
    /// 读得出、自证不过：校验和不过、fsid 不是本池的、从没写过的全 0 槽（`RootRecord::parse_slot` 交回空）。
    NotSelfVerified,
}

/// 读一个根环槽的结局。
#[allow(
    clippy::large_enum_variant,
    reason = "一次读一个槽、按值交回马上拆开，装箱只多一次分配"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootRingSlotReading {
    SelfVerified(RootRecord),
    Bad(BadRootRingSlotReading),
}

/// 读一个根环槽（区域归属表给出它在哪块盘、槽距给出偏移），验自证校验和与 fsid。只读一次，不重试。
/// 自证过的字节报给读者（[`PoolReader::note_that_the_bytes_read_self_verify`]：当场重读那一遍的读者拿它换掉这次挂载记下的那一份）。
#[must_use]
pub fn read_root_ring_slot<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slot: RootRingSlot,
) -> RootRingSlotReading {
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset = slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
    let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
        return RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable);
    };
    match RootRecord::parse_slot(&bytes, filesystem_identifier) {
        Some(root) => {
            reader.note_that_the_bytes_read_self_verify(device, offset, &bytes);
            RootRingSlotReading::SelfVerified(root)
        }
        None => RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
    }
}

/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7 择新；
/// 已定项 24 先逐个验证全部候选、再在有效者中择新）；最大的不止一个槽时取读根环次序里最前的那个。
/// 取最大那一截在 `singlefs_replay_prefix::choose_root_slot` 里（Verus 验的是那一份）；这里只读槽、验自证校验和与 fsid，
/// 把每个槽摊成核的入参（自证过的给择新键，读不出、自证不过的给 None），再照核交回的下标取回那条根。
#[must_use]
pub fn choose_root(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> Option<RootRecord> {
    let immutable = &system_configuration.immutable;
    let self_verified_roots_in_ring_order: Vec<Option<RootRecord>> =
        every_root_ring_slot(&immutable.sizes)
            .into_iter()
            .map(|ring_slot| {
                match read_root_ring_slot(
                    reader,
                    &immutable.region_devices,
                    &immutable.sizes,
                    &immutable.filesystem_identifier,
                    ring_slot,
                ) {
                    RootRingSlotReading::SelfVerified(root) => Some(root),
                    RootRingSlotReading::Bad(
                        BadRootRingSlotReading::Unreadable
                        | BadRootRingSlotReading::NotSelfVerified,
                    ) => None,
                }
            })
            .collect();
    let choice_keys_in_ring_order: Vec<Option<RootChoiceKey>> = self_verified_roots_in_ring_order
        .iter()
        .map(|self_verified_root| {
            self_verified_root.map(|root| RootChoiceKey {
                checkpoint_txg: PrefixKernelCheckpointTxg(root.checkpoint_txg.0),
                instance: PrefixKernelInstanceGeneration(root.instance.0),
            })
        })
        .collect();
    choose_root_slot(&choice_keys_in_ring_order).map(|chosen_slot| {
        self_verified_roots_in_ring_order[chosen_slot]
            .expect("择根核只交回自证过的槽（它的 ensures：所选槽在入参里是 Some），这里是 None 说明核或摊入参错了")
    })
}

/// 根环全部自证过的根里最大的 checkpoint_txg：新实例的第一次发布取 max(它, 环里全部自证通过的记录的 checkpoint_txg) + 1
/// （D23（journal 的角色与格式） 已定项 14 第 3 条）；一条都没有时 None。
#[must_use]
pub fn highest_root_txg<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Option<CheckpointTxg> {
    let mut highest: Option<CheckpointTxg> = None;
    visit_valid_roots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |root| {
            highest = Some(highest.map_or(root.checkpoint_txg, |current| {
                current.max(root.checkpoint_txg)
            }));
        },
    );
    highest
}

/// 本实例第一次发布要带的树 ID 水位：根环全部自证过的根的「树 ID 水位」取 max（D8（核心索引结构） 已定项 8 ②
/// 「根环里全部根记录该字段的 max」——被抛弃时间线上的根也算，它们仍承载那一代发过的号），
/// 并上 `records`（环里全部自证通过的 journal 记录）新根段里带的水位（D23（journal 的角色与格式） 已定项 15）。
///
/// 记录那一半是根读不出时的最保守读法：根槽撕了、被盖了或那块盘掉了，而它那次发布的记录还在，就按记录带的水位算
/// （C342（树 ID 水位在根读不出时退回去重发） 的载体；条款没写读不出的根怎么算，报告里交主 agent）。
/// 根与它那条记录都读不出的那一格仍然盖不住。与新实例第一次发布的 txg 取 max(环里的根, 环里的记录) 同一个形态
/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。一条根、一条记录都没有时 None。
#[must_use]
pub fn highest_tree_identifier_watermark_in_the_ring<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> Option<u64> {
    let mut highest: Option<u64> = records
        .values()
        .map(|record| record.new_tree_identifier_watermark)
        .max();
    visit_valid_roots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |root| {
            highest = Some(highest.map_or(root.tree_identifier_watermark, |current| {
                current.max(root.tree_identifier_watermark)
            }));
        },
    );
    highest
}

/// 根环里全部自证过的根，每个可读根槽一条（回退候选集与影子账从这里取）。
#[must_use]
pub fn readable_roots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Vec<RootRecord> {
    let mut roots = Vec::new();
    visit_valid_roots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |root| roots.push(root),
    );
    roots
}

/// 读根环，读坏的槽按这个进程知不知道那里住着一条根分两类（D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句，用户 2026-09-26 定；
/// 与 `mount::rollback_floor_ceiling` 读根环同一个分法）：每个槽读一次；读坏（读不出 [`BadRootRingSlotReading::Unreadable`]：设备报错、越界；
/// 或自证不过 [`BadRootRingSlotReading::NotSelfVerified`]）的槽不在 `ring_slots_known_to_hold_a_root` 里（挂载那一刻就读不出或自证不过、
/// 这个进程之后也没写过）的当没有根、不重读；在里面的（挂载那一刻读得出、或这个进程写过且 FUA 返回过，
/// `allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）立即重读一次（C554 乙的形态，R = 1），重读自证得过就照它算，
/// 仍坏（读不出或自证不过）就报错，不按有根或没根猜。交回读得出的每一条根连同它的槽，按根环槽的次序。
/// 挂着时抬 F 重算影子账与回收门槛、管理员回退判候选与算 F 生效值用它（C554 乙报告 Q6：`readable_roots` 把读不出的槽一律当没有根，
/// 那一槽里的被抛弃根引用的槽不隔离、也不计进读不出账的被抛弃根）。
///
/// # Errors
/// 知道住着根的槽重读仍坏 ⇒ [`RootRingSlotKnownToHoldARootStillBadAfterOneReread`]（按 [`every_root_ring_slot`] 的次序第一个）。
/// 只读盘。
pub fn readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
) -> Result<Vec<(RootRingSlot, RootRecord)>, RootRingSlotKnownToHoldARootStillBadAfterOneReread> {
    let read_the_slot = |ring_slot| {
        read_root_ring_slot(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        )
    };
    // 重读那一读绕过这次挂载记下的那一份、再读一次盘（[`ReadPassOverThePool::RereadAfterABadRead`]）。
    let rereading_reader = PoolReaderOfOneReadPass::rereading_after_a_bad_read(reader);
    let reread_the_slot = |ring_slot| {
        read_root_ring_slot(
            &rereading_reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        )
    };
    let mut roots = Vec::new();
    // 迭代次数的上界是根环槽数 R × S；跨轮只带已认出的根。提前出口只有「知道住着根的槽重读仍坏」一个。
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        let first_reading = match read_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => {
                roots.push((ring_slot, root));
                continue;
            }
            RootRingSlotReading::Bad(first_reading) => first_reading,
        };
        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {
            continue;
        }
        match reread_the_slot(ring_slot) {
            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),
            RootRingSlotReading::Bad(reread) => {
                return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {
                    ring_slot,
                    first_reading,
                    reread,
                });
            }
        }
    }
    Ok(roots)
}

/// 同 [`readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once`]，只交回根（不带槽）。
///
/// # Errors
/// 同那一份。
pub fn readable_roots_rereading_ring_slots_known_to_hold_a_root_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
) -> Result<Vec<RootRecord>, RootRingSlotKnownToHoldARootStillBadAfterOneReread> {
    readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        ring_slots_known_to_hold_a_root,
    )
    .map(|roots_with_ring_slots| {
        roots_with_ring_slots
            .into_iter()
            .map(|(_, root)| root)
            .collect()
    })
}

/// 同 [`readable_roots_with_ring_slots`]，而一个根槽读不出（[`BadRootRingSlotReading::Unreadable`]：设备报错、越界）时立即重读那一槽一次，
/// 重读读得出就照它算，仍读不出当没有根；读得出、自证不过的槽不重读（从没写过的全 0 槽都是这一类）。给手里没有这个进程根环表、
/// 分不出哪几个槽住着根的读者用（`transaction::PoolWriter` 写系统配置之前算 F 生效值，[`effective_rollback_floor_rereading_unreadable_reads_once`]）。
#[must_use]
pub fn readable_roots_with_ring_slots_rereading_unreadable_slots_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Vec<(RootRingSlot, RootRecord)> {
    let read_the_slot = |ring_slot| {
        read_root_ring_slot(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slot,
        )
    };
    let mut roots = Vec::new();
    // 迭代次数的上界是根环槽数 R × S；跨轮只带已认出的根；没有提前出口。
    for ring_slot in every_root_ring_slot(immutable_sizes) {
        let reading = match read_the_slot(ring_slot) {
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {
                read_the_slot(ring_slot)
            }
            first_reading @ (RootRingSlotReading::SelfVerified(_)
            | RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)) => first_reading,
        };
        match reading {
            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),
            RootRingSlotReading::Bad(
                BadRootRingSlotReading::NotSelfVerified | BadRootRingSlotReading::Unreadable,
            ) => {}
        }
    }
    roots
}

/// 同 `readable_roots`，每条根带着它读出来的那个根环槽（挂载与抬 F 给分配器建根环那张表用，`allocator::RootRingOccupancy`）。
#[must_use]
pub fn readable_roots_with_ring_slots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Vec<(RootRingSlot, RootRecord)> {
    let mut roots = Vec::new();
    visit_valid_roots_with_ring_slots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |ring_slot, root| roots.push((ring_slot, root)),
    );
    roots
}

/// 一条根按一张实例表判是不是被抛弃的：表里有它那个实例的行 (i, Ti, Wi) 且它的 txg > Ti（D23（journal 的角色与格式） 已定项 14
/// 候选集那一格「(i, T) 可选 ⟺ 表里无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti」反过来）。回退候选集、影子账、抬 F 的上限与
/// F 生效值里「有效根」那一半都按它判，一处定义（判法在 [`publish_is_abandoned_by_the_instance_table`]）。
#[must_use]
pub fn root_is_abandoned_by_the_instance_table(
    root: &RootRecord,
    table: &InstanceTableRecords,
) -> bool {
    publish_is_abandoned_by_the_instance_table(root.instance, root.checkpoint_txg, table)
}

/// 一次发布 (实例代号 `instance`, `checkpoint_txg`) 按一张实例表判是不是在被抛弃的时间线上：表里有 `instance` 的行 (i, Ti, Wi) 且
/// `checkpoint_txg` > Ti。[`root_is_abandoned_by_the_instance_table`] 拿根记录的这两样判；F2 判「退过去会不会跳过一次根读不出的发布」时
/// 拿那次发布带末条标志的记录的这两样判（那次发布的根读不出，没有根记录可拿）。
#[must_use]
pub fn publish_is_abandoned_by_the_instance_table(
    instance: InstanceGeneration,
    checkpoint_txg: CheckpointTxg,
    table: &InstanceTableRecords,
) -> bool {
    table
        .rows
        .iter()
        .any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)
}

/// 生效的回退下界 F（D16（发布语义） 已定项 1「生效」那一行，SysPre）：F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值,
/// 系统配置里读得出的 F 的最大值)。
///
/// - 根那一半：根环里自证过的根，按根环落点公式归到它那块盘（区域归属表），每块盘取按实例表判仍然有效（[`root_is_abandoned_by_the_instance_table`]
///   判不是被抛弃的）的根里 (txg, 实例代号) 最大的那一条，取它带的 F；各盘取最大。判有效用的实例表是根环里 (txg, 实例代号) 最大那条根指着的
///   那一张（恢复择的就是它，`choose_root`）；那张表读不出、解不开时不按表滤，每块盘取读得出的根里最新的那条。
///   被抛弃时间线上的根带的 F 也算进来（D16（发布语义） 已定项 1，用户 2026-09-28 定「算进」）：根环里按那张表判被抛弃的根，取它们带的 F 的最大值，与各盘那一份取大；那张表读不出时没有「被抛弃」可判，照旧。
/// - 系统配置那一半：池里每块盘两槽里全部自证过（整槽校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）
///   的槽带的 F 的最大值。
///
/// 两处一条都读不出时 0（mkfs 写的就是 0）。
///
/// ⚠️ 读不出的根槽当没有根、那张表读不出时「不按表滤」是无声放过（C554 乙报告 Q6，实审 A3b 报告 Q7）：产品路径都不走这一份——
/// 可写挂载按重读过的那张表算（[`effective_rollback_floor_under_the_newest_roots_table`]），抬 F 走
/// [`effective_rollback_floor_rereading_the_newest_instance_table_once`]，管理员回退走
/// [`effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`]，写系统配置槽走
/// [`effective_rollback_floor_rereading_unreadable_reads_once`]。留给单外的实验装置与用例。
#[must_use]
pub fn effective_rollback_floor<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> CheckpointTxg {
    let roots_with_ring_slots = readable_roots_with_ring_slots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
    );
    let newest_roots_table = newest_root_among(&roots_with_ring_slots)
        .and_then(|newest| instance_table_chain_of_root(reader, &newest).ok())
        .map(|chain| chain.records);
    effective_rollback_floor_of_the_roots_read(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        &roots_with_ring_slots,
        newest_roots_table.as_ref(),
    )
}

/// 生效的回退下界 F（[`effective_rollback_floor`] 同一个算法），判「有效根」用调用方交进来的那张表：调用方已经把根环里最新那条根的实例表
/// 读出来了（可写挂载重建分配器时读过、读不出重读过一次，`mount` 的 `instance_table_of_the_newest_root_read_at_most_twice`），
/// 不再读一遍、也就没有「读不出就不按表滤」那一支。
#[must_use]
pub fn effective_rollback_floor_under_the_newest_roots_table<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    newest_roots_table: &InstanceTableRecords,
) -> CheckpointTxg {
    let roots_with_ring_slots = readable_roots_with_ring_slots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
    );
    effective_rollback_floor_of_the_roots_read(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        &roots_with_ring_slots,
        Some(newest_roots_table),
    )
}

/// 生效的回退下界 F（[`effective_rollback_floor`] 同一个算法），根环里最新那条根的实例表这一次读不出、解不开（或一条自证过的根都择不到）时
/// 立即重读一次根环与那张表（C554 乙的形态，R = 1；D16（发布语义） 已定项 1「重读一次」），仍是这样就报错，不按「不按表滤」往下走。
///
/// # Errors
/// 重读仍读不出 ⇒ [`InstanceTableOfTheNewestRootStillUnreadableAfterOneReread`]。只读盘、不写盘。
pub fn effective_rollback_floor_rereading_the_newest_instance_table_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Result<CheckpointTxg, InstanceTableOfTheNewestRootStillUnreadableAfterOneReread> {
    let read_the_ring = |pass| {
        Ok::<_, std::convert::Infallible>(readable_roots_with_ring_slots(
            &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
            region_devices,
            immutable_sizes,
            filesystem_identifier,
        ))
    };
    let Ok(floor) = effective_rollback_floor_rereading_the_newest_instance_table_once_over(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        read_the_ring,
    );
    match floor {
        EffectiveFloorAfterAtMostOneReread::UnderTheNewestRootsTable(floor) => Ok(floor),
        EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
            newest_root_on_the_reread,
            ..
        } => Err(InstanceTableOfTheNewestRootStillUnreadableAfterOneReread {
            newest_root_on_the_reread,
        }),
    }
}

/// 生效的回退下界 F（[`effective_rollback_floor`] 同一个算法），管理员回退判候选用：读根环照
/// [`readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once`]（这个进程知道住着根的槽读不出就重读一次，
/// 别的读不出的槽当没有根，D16（发布语义） 已定项 1「根槽这一次读坏」那一行的分法），根环里最新那条根的实例表照
/// [`effective_rollback_floor_rereading_the_newest_instance_table_once`] 读不出就连根环一起重读一次。
/// 不按原 [`effective_rollback_floor`] 那样把读不出的槽当没有根、把读不出的表当不按表滤（实审 A3b 报告 Q7：
/// 那样 F 生效值可以算低，低于真 F 的目标被当成候选）。
///
/// # Errors
/// 知道住着根的槽重读仍坏 ⇒ [`EffectiveFloorReadingStillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`]；
/// 最新那条根的实例表重读仍读不出 ⇒ [`EffectiveFloorReadingStillUnreadableAfterOneReread::InstanceTableOfTheNewestRoot`]。只读盘、不写盘。
pub fn effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
) -> Result<CheckpointTxg, EffectiveFloorReadingStillUnreadableAfterOneReread> {
    let read_the_ring = |pass| {
        readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once(
            &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            ring_slots_known_to_hold_a_root,
        )
    };
    match effective_rollback_floor_rereading_the_newest_instance_table_once_over(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        read_the_ring,
    )
    .map_err(EffectiveFloorReadingStillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot)?
    {
        EffectiveFloorAfterAtMostOneReread::UnderTheNewestRootsTable(floor) => Ok(floor),
        EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
            newest_root_on_the_reread,
            ..
        } => Err(
            EffectiveFloorReadingStillUnreadableAfterOneReread::InstanceTableOfTheNewestRoot(
                InstanceTableOfTheNewestRootStillUnreadableAfterOneReread {
                    newest_root_on_the_reread,
                },
            ),
        ),
    }
}

/// 生效的回退下界 F（[`effective_rollback_floor`] 同一个算法），给手里没有这个进程根环表、也没有错误可报的写者用
/// （`transaction::PoolWriter` 每次写系统配置槽之前算要写的 F，实审 A3b 报告 Q7）：读根环照
/// [`readable_roots_with_ring_slots_rereading_unreadable_slots_once`]（读不出的槽重读一次，仍读不出当没有根），根环里最新那条根的实例表
/// 读不出就连根环一起重读一次，仍读不出不按表滤（与原 [`effective_rollback_floor`] 同一个退路）。
/// 为什么这一处不拒：见 `transaction::PoolWriter::write_system_configuration_slot` 里那一段注释。只读盘、不写盘。
#[must_use]
pub fn effective_rollback_floor_rereading_unreadable_reads_once<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> CheckpointTxg {
    let read_the_ring = |pass| {
        Ok::<_, std::convert::Infallible>(
            readable_roots_with_ring_slots_rereading_unreadable_slots_once(
                &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
                region_devices,
                immutable_sizes,
                filesystem_identifier,
            ),
        )
    };
    let Ok(floor) = effective_rollback_floor_rereading_the_newest_instance_table_once_over(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        read_the_ring,
    );
    match floor {
        EffectiveFloorAfterAtMostOneReread::UnderTheNewestRootsTable(floor) => floor,
        EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
            roots_with_ring_slots_on_the_reread,
            ..
        } => effective_rollback_floor_of_the_roots_read(
            reader,
            region_devices,
            immutable_sizes,
            filesystem_identifier,
            &roots_with_ring_slots_on_the_reread,
            None,
        ),
    }
}

/// 算 F 生效值时读根环、择最新那条根、读它的实例表，至多重读一次之后的结局
/// （[`effective_rollback_floor_rereading_the_newest_instance_table_once_over`]）。
enum EffectiveFloorAfterAtMostOneReread {
    /// 第一遍或重读那一遍读出了最新那条根的实例表，按它算出的 F 生效值。
    UnderTheNewestRootsTable(CheckpointTxg),
    /// 重读那一遍仍没有表（表读不出、解不开，或一条自证过的根都择不到）：交回重读那一遍读到的根环与择到的最新那条根，
    /// 调用方定拒还是不按表滤。
    NewestRootsTableStillUnreadable {
        roots_with_ring_slots_on_the_reread: Vec<(RootRingSlot, RootRecord)>,
        /// 重读那一遍择到的最新那条根（实例代号, checkpoint_txg）；一条都没择到时 `None`。
        newest_root_on_the_reread: Option<(InstanceGeneration, CheckpointTxg)>,
    },
}

/// 读一遍根环（`read_the_ring`：怎么读、读坏的槽怎么办由调用方定）、择最新那条根、沿链读它的实例表；表读不出、解不开或一条根都择不到时
/// 立即把根环与表一起重读一次（C554 乙的形态，R = 1；D16（发布语义） 已定项 1「重读一次」），读出表就按它算 F 生效值。
/// 第一次读与重读一次走同一段；重读那一遍根环与表都按 [`ReadPassOverThePool::RereadAfterABadRead`] 读（`read_the_ring` 收到哪一遍）。
///
/// # Errors
/// `read_the_ring` 的错原样交回（那一遍之后不再读）。
fn effective_rollback_floor_rereading_the_newest_instance_table_once_over<
    Reader: PoolReader + ?Sized,
    RingReadingFailure,
>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    read_the_ring: impl Fn(
        ReadPassOverThePool,
    ) -> Result<Vec<(RootRingSlot, RootRecord)>, RingReadingFailure>,
) -> Result<EffectiveFloorAfterAtMostOneReread, RingReadingFailure> {
    let read_the_ring_and_the_newest_roots_table = |pass: ReadPassOverThePool| {
        let roots_with_ring_slots = read_the_ring(pass)?;
        let newest = newest_root_among(&roots_with_ring_slots);
        let newest_roots_table = newest
            .and_then(|newest| {
                instance_table_chain_of_root(
                    &PoolReaderOfOneReadPass::in_the_pass(reader, pass),
                    &newest,
                )
                .ok()
            })
            .map(|chain| chain.records);
        Ok((roots_with_ring_slots, newest, newest_roots_table))
    };
    let floor_under = |roots_with_ring_slots: &[(RootRingSlot, RootRecord)],
                       table: &InstanceTableRecords| {
        EffectiveFloorAfterAtMostOneReread::UnderTheNewestRootsTable(
            effective_rollback_floor_of_the_roots_read(
                reader,
                region_devices,
                immutable_sizes,
                filesystem_identifier,
                roots_with_ring_slots,
                Some(table),
            ),
        )
    };
    if let (roots_with_ring_slots, _, Some(table)) =
        read_the_ring_and_the_newest_roots_table(ReadPassOverThePool::FirstRead)?
    {
        return Ok(floor_under(&roots_with_ring_slots, &table));
    }
    Ok(
        match read_the_ring_and_the_newest_roots_table(ReadPassOverThePool::RereadAfterABadRead)? {
            (roots_with_ring_slots, _, Some(table)) => floor_under(&roots_with_ring_slots, &table),
            (roots_with_ring_slots_on_the_reread, newest_on_the_reread, None) => {
                EffectiveFloorAfterAtMostOneReread::NewestRootsTableStillUnreadable {
                    roots_with_ring_slots_on_the_reread,
                    newest_root_on_the_reread: newest_on_the_reread
                        .map(|root| (root.instance, root.checkpoint_txg)),
                }
            }
        },
    )
}

/// 读出来的根里 (checkpoint_txg, 实例代号) 最大的那一条（与 [`choose_root`] 同一个择法）；一条都没有时 `None`。
/// 择新键完全相同的不止一条时取读根环次序里最前的那条，与 `singlefs_replay_prefix::choose_root_slot` 的平局取法相同
/// （只有后面那条严格更新才换），F 生效值按表判有效根用的那张表才是恢复择的那条根指着的。
/// 入参须按读根环次序排（`every_root_ring_slot`：读根环的每一处都按这个次序走）。
fn newest_root_among(roots_with_ring_slots: &[(RootRingSlot, RootRecord)]) -> Option<RootRecord> {
    roots_with_ring_slots
        .iter()
        .map(|(_, root)| *root)
        .reduce(|newest_so_far, candidate| {
            if (candidate.checkpoint_txg, candidate.instance)
                > (newest_so_far.checkpoint_txg, newest_so_far.instance)
            {
                candidate
            } else {
                newest_so_far
            }
        })
}

/// 生效的回退下界 F 的算法本身（[`effective_rollback_floor`] 的文档），根环已经读过、判「有效根」的那张表已经读过（读不出时 `None`：
/// 不按表滤，只有 [`effective_rollback_floor`] 那一份这样传）。
fn effective_rollback_floor_of_the_roots_read<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    roots_with_ring_slots: &[(RootRingSlot, RootRecord)],
    newest_roots_table: Option<&InstanceTableRecords>,
) -> CheckpointTxg {
    let mut newest_valid_root_on_each_device: BTreeMap<DeviceIdentity, RootRecord> =
        BTreeMap::new();
    for (ring_slot, root) in roots_with_ring_slots {
        if newest_roots_table
            .is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table))
        {
            continue;
        }
        let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let newest_on_this_device = newest_valid_root_on_each_device
            .entry(device)
            .or_insert(*root);
        if (root.checkpoint_txg, root.instance)
            > (
                newest_on_this_device.checkpoint_txg,
                newest_on_this_device.instance,
            )
        {
            *newest_on_this_device = *root;
        }
    }
    // 被抛弃根带的 F 也算进 F_生效（收口表第 ② 行，用户 2026-09-28 定「算进」，被攻过一轮；
    // `research/prompts/abandoned-floor-r1-main-verification.md` M2）：不算时被抛弃根带着比别处高的 F 那一形上 I-7.12 红、
    // 回退能退到那个 F 之下，而被抛弃时间线抬 F 时已按它回收过槽。
    let highest_on_the_abandoned_roots = roots_with_ring_slots
        .iter()
        .filter(|(_, root)| {
            newest_roots_table
                .is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table))
        })
        .map(|(_, root)| root.rollback_floor)
        .max();
    let highest_on_the_roots = newest_valid_root_on_each_device
        .values()
        .map(|root| root.rollback_floor)
        .chain(highest_on_the_abandoned_roots)
        .max();
    let slot_spacing_in_bytes = u64::from(immutable_sizes.fixed_structure_slot_spacing);
    let highest_in_the_system_configurations = reader
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            verified_system_configuration_slots(
                reader,
                device,
                slot_spacing_in_bytes,
                filesystem_identifier,
            )
        })
        .map(|system_configuration| system_configuration.quantities.rollback_floor)
        .max();
    highest_on_the_roots
        .max(highest_in_the_system_configurations)
        .unwrap_or(CheckpointTxg(0))
}

/// 盘上读来的分配记录逐条对这个池的几何判一遍：这几个字段可以是任何值，它们不是不变量。
/// 读分配记录树时两道判分得开：前一道是**字段表**与**位置**（条目宽够不够装下 20 字节的记录、记录落不落在它所在叶里，
/// `crate::allocation_record_tree::read_allocation_record_tree`），这一道是**几何**（设备身份、槽号、跨度、两条记录罩不罩同一个槽）。
/// 分配器那一侧的下标、位图长度与两条断言全部按这两道已经判过来写（`allocator::DeviceFreeMap::index` / `mark_allocated` 的消息指着这里）。
///
/// 四样各对着 panic 面普查里的一条：设备身份不在池里（R9，`PoolAllocator::rebuild_from_records` 的 `expect`）、
/// 槽号在单元区起点之下（R6，`DeviceFreeMap::index` 的减法）、跨度越过单元区末尾与同一块盘上两条记录罩住同一个槽
/// （R6 / R8，`mark_allocated` 的两条断言）。
///
/// 单元区槽数只有 [`crate::allocator::unit_area_slots_of_device_starting_at`] 一处定义，分配器的位图长度读的也是它。
/// 前三样逐条走 [`placement_lies_in_the_unit_area_of_its_device`]；mkfs 那一版的两个单元进分配器之前也走这一道
/// （`mount::format_time_allocator`，代码审阅第 32 条）。
///
/// # Errors
/// 四样里任一样不成立 ⇒ [`RecoveryFailure::AllocationRecordOutsideThePoolGeometry`]，`what` 说清是哪一样；
/// 记录所在那块盘末尾在单元区起点之前 ⇒ [`RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart`]。
pub(crate) fn allocation_records_fit_the_pool_geometry(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    records: &[AllocationRecord],
) -> Result<(), RecoveryFailure> {
    let mut covered_slots: BTreeSet<(DeviceIdentity, u64)> = BTreeSet::new();
    for record in records {
        let span = u64::from(record.span_slots);
        placement_lies_in_the_unit_area_of_its_device(
            reader,
            unit_area_start,
            record.device,
            record.slot,
            span,
        )?;
        for slot in record.slot.0..record.slot.0 + span {
            if !covered_slots.insert((record.device, slot)) {
                return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                    what: "同一块盘上两条分配记录罩住同一个槽",
                });
            }
        }
    }
    Ok(())
}

/// 盘上读来的一个落点 `(device, [slot, slot + span))` 落不落在那块盘的单元区里（单元区从 `unit_area_start` 起：择到的系统配置按环长现算的那个，
/// [`unit_area_start_of_the_chosen_system_configuration`]）：设备身份在池里、槽号不在单元区起点之下、跨度不越过单元区末尾
/// （[`allocation_records_fit_the_pool_geometry`] 逐条记录判的前三样；影子账拿被抛弃根的指针槽号进分配器之前也走这一处，代码审阅第 32 条）。
/// 分配器的位图下标与「跨度越过单元区末尾」那几句断言按这一判已经做过来写（`allocator::DeviceFreeMap::index`、`isolate`、`mark_allocated`）。
///
/// # Errors
/// 设备身份不在池里、槽号在单元区起点之下、跨度越过单元区末尾 ⇒ [`RecoveryFailure::AllocationRecordOutsideThePoolGeometry`]；
/// 那块盘末尾在单元区起点之前 ⇒ [`RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart`]。
pub(crate) fn placement_lies_in_the_unit_area_of_its_device(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    device: DeviceIdentity,
    slot: SlotNumber,
    span_slots: u64,
) -> Result<(), RecoveryFailure> {
    // 「这块盘在不在池里」只这一处判：`PoolReader::device_size_in_bytes` 对池外的盘交 `None`
    // （每个实现都从 `device_identities` 那张表里找），另写一句 `device_identities().contains(…)`
    // 是同一条判定的第二份手抄，两份会分叉（`code-discipline.md`「重复要生成，不许手抄」）。
    let device_bytes = reader.device_size_in_bytes(device).ok_or(
        RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
            what: "分配记录的设备身份不在池里",
        },
    )?;
    if slot.0 < unit_area_start.slot().0 {
        return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
            what: "分配记录的槽号落在单元区起点之下",
        });
    }
    let unit_area_slots = unit_area_slots_of_device_starting_at(device_bytes, unit_area_start)
        .map_err(|DeviceEndsBeforeTheUnitAreaStart { device_bytes, .. }| {
            RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                device,
                device_bytes,
            }
        })?;
    // 槽号是盘上的 6 字节（< 2⁴⁸）、跨度 ≤ 2¹⁶ 由调用方的字段宽给出，两个加法都装得进 u64。
    let unit_area_end_slot = unit_area_start.slot().0 + unit_area_slots;
    if slot.0 + span_slots > unit_area_end_slot {
        return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
            what: "分配记录的跨度越过单元区末尾",
        });
    }
    Ok(())
}

/// 池里每块盘的完整槽数都到得了单元区起点 `unit_area_start`（择到的系统配置按环长现算的那个；代码审阅第 35 条）：
/// 可写挂载建空闲图（`allocator::DeviceFreeMap::with_unit_area_start`）的前置条件，在择系统配置之后、读根环之前逐盘判。
/// 只读盘的字节数，不读盘。
///
/// # Errors
/// 按 [`PoolReader::device_identities`] 次序第一块末尾在单元区起点之前的盘 ⇒ [`RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart`]。
pub fn every_device_reaches_the_unit_area_start(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
) -> Result<(), RecoveryFailure> {
    for device in reader.device_identities() {
        let device_bytes = reader
            .device_size_in_bytes(device)
            .expect("device_identities 列出的盘，device_size_in_bytes 交得出它的字节数（PoolReader 的契约）");
        unit_area_slots_of_device_starting_at(device_bytes, unit_area_start).map_err(
            |_ends_before| RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                device,
                device_bytes,
            },
        )?;
    }
    Ok(())
}

/// 一条根引用的分配记录（树表 → 分配记录树根节点）：回退的影子账要读每条被抛弃根的账。第 0 代树表（没有分配记录树）给空。
/// 分配记录树根的位置提示读不出时经这条根的中央映射回退（D19（块指针的结构与宽度预算） 已定项 8，与冷走读同一条
/// `read_mapped_tree_node_via_hint_then_central_mapping`）；树表是自举豁免，只按根记录里的位置条目读。
/// 回退的次数这里不交出去：调用方（影子账、重建分配器）今天没有接多跳观测点的口子。
///
/// # Errors
/// 树表读不到、解不开，同一种树有两条（[`TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE`]）；
/// 分配记录树根提示读不出且映射里没有它（`MappingMiss`）、映射落点也读不出（`MappingStillUnreadable`）、
/// 映射树根读不出或解不开、分配记录树根解不开；条目宽判红（见 [`read_allocation_record_tree`]）或结构值判红
/// （见 `allocation_records_fit_the_pool_geometry`）。
///
/// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个，[`unit_area_start_of_the_chosen_system_configuration`]）：
/// 分配记录落不落在单元区里按它判。
pub fn allocation_records_under_root_in_the_unit_area_starting_at(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;
    let Some(allocation_entry) = tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_ALLOCATION)
    else {
        return Ok(Vec::new());
    };
    // 读的是别的根（影子账、抬 F 的上限）的账：映射树只按「拼得成一棵树」核，与这里读分配记录树同一个口径。
    let central_mapping_root = CentralMappingTreeWithBytesReadOnFirstUse::new(
        reader,
        root,
        CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,
    );
    let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
    // 分配记录树按绝对槽号按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵读回来，节点进映射、提示读不出经这条根的中央映射回退。
    let tree = read_allocation_record_tree(
        &allocation_entry.root,
        &AllocationRecordTreeGeometry::of_reader(reader),
        allocation_entry.tree,
        AllocationRecordTreeHeaderJudgement::OnlyWhatThePositionsNeed,
        root,
        unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &NodePointer| {
            read_mapped_tree_node_via_hint_then_central_mapping(
                reader,
                pointer,
                MappedTreeNodeClass::IndexNode,
                &|mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key),
                &mut stale_location_hint_hops_not_exposed_by_this_reader,
            )
        },
    )?;
    allocation_records_fit_the_pool_geometry(reader, unit_area_start, &tree.records)?;
    Ok(tree.records)
}

/// 同 [`allocation_records_under_root_in_the_unit_area_starting_at`]，单元区起点先择一次系统配置现算
/// （[`choose_system_configuration`]：多读每块盘的两个系统配置槽）。给手里没有择好的系统配置的调用方（用例、实验装置）。
///
/// # Errors
/// 择不到系统配置（[`choose_system_configuration`] 的各个成员）；其余同带起点的那一份。
pub fn allocation_records_under_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
    allocation_records_under_root_in_the_unit_area_starting_at(
        reader,
        unit_area_start_of_the_pool(reader)?,
        root,
    )
}

/// 树表 0 条的那一版自己那棵分配记录树（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））：整棵读回来
/// （D8（核心索引结构） 已定项 14：挂载时分配记录树整棵读），交回它的节点与指针、全部记录。树号 0、节点豁免映射，只按位置条目读。
/// 指针全零 ⇒ `None`，那是 mkfs 的第 0 代（那一版的账由实例表与树表两条指针直接算）。
///
/// # Errors
/// 分配记录树的节点读不到、解不开、位置对不上、节点头里的自描述与指着它的指针对不上（`crate::allocation_record_tree::read_allocation_record_tree`，
/// `EveryHeaderAgainstItsReference`）；结构值判红（`allocation_records_fit_the_pool_geometry`）。
///
/// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个）：分配记录落不落在单元区里按它判。
pub fn allocation_records_of_version_without_file_in_the_unit_area_starting_at(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
) -> Result<Option<AllocationRecordTreeOfTheVersionWithoutFile>, RecoveryFailure> {
    if root.allocation_record_tree_root == NodePointer::empty_root() {
        return Ok(None);
    }
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    // 这是可写挂载要接着写的那一版自己的账：节点头照冷走读判全（树 ID、key 宽、出生身份、fsid，代码审阅第 24 / 33 条）。
    let tree = read_allocation_record_tree(
        &root.allocation_record_tree_root,
        &AllocationRecordTreeGeometry::of_reader(reader),
        TreeIdentifier(crate::transaction::TREE_IDENTIFIER_NONE),
        AllocationRecordTreeHeaderJudgement::EveryHeaderAgainstItsReference,
        root,
        unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &NodePointer| {
            read_unit_via_locations(reader, &pointer.locations, node_bytes)
        },
    )?;
    allocation_records_fit_the_pool_geometry(reader, unit_area_start, &tree.records)?;
    // 这棵树的节点指针下一次发布照抄或换下时写回内部条目：按 I-2.5 判（`pointers_of_the_rebuilt_version_ascend_by_device` 同一道）。
    for (_, pointer, _) in &tree.nodes {
        pointer
            .location_entries_ascend_by_device()
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    Ok(Some(AllocationRecordTreeOfTheVersionWithoutFile {
        version: tree.version(),
        records: tree.records,
    }))
}

/// 同 [`allocation_records_of_version_without_file_in_the_unit_area_starting_at`]；根记录那一项全零（没有分配记录树）时直接交 `None`、
/// 不读系统配置，否则单元区起点先择一次系统配置现算（[`choose_system_configuration`]）。给手里没有择好的系统配置的调用方。
///
/// # Errors
/// 择不到系统配置（[`choose_system_configuration`] 的各个成员）；其余同带起点的那一份。
pub fn allocation_records_of_version_without_file(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Option<AllocationRecordTreeOfTheVersionWithoutFile>, RecoveryFailure> {
    if root.allocation_record_tree_root == NodePointer::empty_root() {
        return Ok(None);
    }
    allocation_records_of_version_without_file_in_the_unit_area_starting_at(
        reader,
        unit_area_start_of_the_pool(reader)?,
        root,
    )
}

/// 一条根指着的整张实例表（全部行）：沿链读到「无下一片」为止（[`instance_table_chain_of_root`]）；
/// 任一片读不出或解不开都是 `None`（读不出的由走读另报）。
#[must_use]
pub fn instance_table_of_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Option<InstanceTableRecords> {
    instance_table_chain_of_root(reader, root)
        .ok()
        .map(|chain| chain.records)
}

/// 一条根指着的整张实例表：各片的行按链上的次序接起来，连同每一片的指针（D18（块里携带什么信息） 已定项 11）。
pub struct InstanceTableChain {
    pub records: InstanceTableRecords,
    /// 每一片的指针，按片序号：第 0 片是根记录里那一条，第 k 片是第 k − 1 片链指针记录里那一条。至少一片。
    pub page_pointers: Vec<NodePointer>,
}

/// 沿链读的结果：读到哪儿认得出哪几片的指针，与整张表读没读成。影子账要前一样（读不全的链也要把认得出的那几片隔离），
/// 别的读者要后一样。
struct InstanceTableChainRead {
    /// 认得出的每一片的指针，按片序号（第 0 片恒在）：一片读不出、解不开时，它自己的指针也在（上一片的链指针记录给的），
    /// 它后面的认不出。
    page_pointers: Vec<NodePointer>,
    outcome: Result<InstanceTableRecords, RecoveryFailure>,
}

/// 从根记录里那条指针读第 0 片，按每一片的链指针记录读下一片，读到「无下一片」为止。
///
/// 迭代上界：第 k 轮要求读到的单元身份里容器号是 k（[`InstanceTablePage::parse`]），两轮读到同一个单元就要它的容器号
/// 同时等于两个数 ⇒ 每一轮读的是盘上不同的单元，轮数不超过一块盘装得下的 32 KiB 单元数；链指回前面任何一片（成环）
/// 在身份那一判上断掉。跨轮带的是认得出的各片指针与已经接起来的行；提前出口只有读不出、解不开两个。
fn read_instance_table_chain<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
    chain_pointer_head_fields: PointerHeadFieldsJudgement,
) -> InstanceTableChainRead {
    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mut page_pointers = vec![root.instance_table];
    let mut page_index = InstanceTablePageIndex::FIRST;
    let mut rows = Vec::new();
    loop {
        let pointer = *page_pointers.last().expect("第 0 片那一条恒在");
        let bytes = match read_unit_via_locations(reader, &pointer.locations, data_bytes) {
            Ok(bytes) => bytes,
            Err(failure) => {
                return InstanceTableChainRead {
                    page_pointers,
                    outcome: Err(failure),
                }
            }
        };
        let Some(page) = InstanceTablePage::parse_with_chain_pointer_head_fields_judgement(
            &bytes,
            page_index,
            chain_pointer_head_fields,
        ) else {
            return InstanceTableChainRead {
                page_pointers,
                outcome: Err(RecoveryFailure::UnitMalformed {
                    what: "实例表的一片（身份不是 (0, 4, 片序号, 0)、链指针记录或行不合法）",
                }),
            };
        };
        rows.extend(page.rows);
        match page.chain {
            InstanceTableChainRecord::LastPage => {
                return InstanceTableChainRead {
                    page_pointers,
                    outcome: Ok(InstanceTableRecords { rows }),
                }
            }
            InstanceTableChainRecord::NextPage(next_page) => {
                page_pointers.push(next_page);
                page_index = page_index.next();
            }
        }
    }
}

/// 沿链读一条根指着的整张实例表（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片的链指针记录指着第 k + 1 片）。
/// 一片读不出或解不开，整张表就不可读（D18（块里携带什么信息） 已定项 11「表不可读时」：指针指不到或校验不过就是表不可读）。
///
/// # Errors
/// 某一片两条位置条目都读不到校验和相符的那份 ⇒ `UnitUnreadable`；某一片解不开（身份不是 (0, 4, 片序号, 0)、
/// 链指针记录或行不合法）⇒ `UnitMalformed`。
pub fn instance_table_chain_of_root<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
) -> Result<InstanceTableChain, RecoveryFailure> {
    let read = read_instance_table_chain(
        reader,
        root,
        PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion,
    );
    read.outcome.map(|records| InstanceTableChain {
        records,
        page_pointers: read.page_pointers,
    })
}

/// 一条根指着的实例表链上认得出的每一片的指针（第 0 片恒在）：读得出的就沿链往下认，读不出、解不开的那一片自己的指针也在，
/// 它后面的认不出。影子账拿它隔离一条被抛弃根的整条链——读不全的链少隔离一片，被抛弃根在环里被覆写之前那一片就可能被发出去。
/// 所以链指针头部第一版之外的取值（加密、压缩那几段与 extent 偏移）这里不判（[`PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger`]）：
/// 指针里的位置条目照样认下一片；沿链读整张表（[`instance_table_chain_of_root`]）照判。
#[must_use]
pub fn instance_table_page_pointers_as_far_as_readable<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
) -> Vec<NodePointer> {
    read_instance_table_chain(
        reader,
        root,
        PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger,
    )
    .page_pointers
}

/// 一条根指着的树表是不是 0 条（mkfs 的第 0 代树表，第一次可写挂载的暖机根照抄它）：这条根下面还没有发布过文件版本。
///
/// # Errors
/// 树表单元读不到、解不开。
pub fn tree_table_has_no_entries(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<bool, RecoveryFailure> {
    Ok(tree_table_entry_count(reader, root)? == 0)
}

/// 一条根指着的树表有几条条目：0 条就是 mkfs 的第 0 代树表（这条根下面还没有发布过文件版本）。
/// 发布路径（`transaction::publish_first_file`）拿写者手里那串盘直接读，所以读者按泛型收、不要求有大小。
///
/// # Errors
/// 树表单元读不到、解不开。
pub fn tree_table_entry_count<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
) -> Result<usize, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    Ok(tree_table.entries.len())
}

/// 树表里同一种树出现两条时，读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
pub const TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE: &str = "树表里同一种树有两条";

/// 树表的条目读不下去是哪一种（[`tree_table_entries_each_kind_at_most_once`] 交回）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TreeTableEntriesRefusal {
    /// 一条条目解不开（条目长度、flags 未知位、预留 76 非零、根指针头部 extent 偏移非 0，`TreeTableEntry::parse`）。
    EntryMalformed,
    /// 同一种树有两条。
    OneKindOfTreeAppearsTwice,
}

impl From<TreeTableEntriesRefusal> for RecoveryFailure {
    fn from(refusal: TreeTableEntriesRefusal) -> Self {
        match refusal {
            TreeTableEntriesRefusal::EntryMalformed => RecoveryFailure::UnitMalformed {
                what: "树表条目",
            },
            TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice => RecoveryFailure::UnitMalformed {
                what: TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE,
            },
        }
    }
}

/// 树表单元的条目逐条解开，并判每一种树至多一条（审阅第 27 条）。写者每种树只写一条；同一种出现两条时取哪一条条款没有写，
/// 取首条、取末条、取首条非空各有一个读者照着读过，同一张树表在几处读出不同的树 ⇒ 判损坏，一条都不取。
/// 读树表取条目的每一处都经这一处：[`rebuild_version`]、[`allocation_records_under_root`]、[`walk_to_file`]、
/// [`user_visible_tree_root_pointers`]、`mounted_read::open_pool_for_read`。
///
/// # Errors
/// 按条目次序第一处读不下去的：解不开 ⇒ `EntryMalformed`；它的种类前面已经有一条 ⇒ `OneKindOfTreeAppearsTwice`。
pub(crate) fn tree_table_entries_each_kind_at_most_once(
    entry_bytes_of_the_tree_table: &[Vec<u8>],
) -> Result<Vec<TreeTableEntry>, TreeTableEntriesRefusal> {
    let mut kinds_seen: BTreeSet<u16> = BTreeSet::new();
    let mut entries = Vec::with_capacity(entry_bytes_of_the_tree_table.len());
    for entry_bytes in entry_bytes_of_the_tree_table {
        let entry =
            TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;
        if !kinds_seen.insert(entry.kind) {
            return Err(TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice);
        }
        entries.push(entry);
    }
    Ok(entries)
}

/// 树表条目不按排序时报的不变量名：I-9.16（树表条目按树 ID 严格升序且合发号次序）。两道合成这一条，任一道不成立即判红：
/// ① 盘上次序树 ID 严格升序（D8（核心索引结构） 已定项 8 排序契约「条目按树 ID 升序排」）；② 按发号次序相邻两棵的树 ID 严格升序
/// （D8 已定项 8 ②「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」）。立号：实审 A3b Q2 交上来，用户 2026-09-27 定
/// 「立不变量并同步」；`invariants.md` 那一行由 kb 第八批写。
pub const TREE_TABLE_ENTRIES_ORDERING_CONTRACT: &str = "I-9.16";

/// 十二棵进树表的树的种类，按发号次序（D8（核心索引结构） 已定项 8 ②：extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist，
/// 再接 D18（块里携带什么信息） 已定项 24 第 6 条的 `deleted_inodes`、墓碑树、谱系树、logged_ops，最后是退役树（D8 已定项 9 码 13）；
/// 中央映射树不进树表，这里没有它）。
const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 12] = [
    TREE_KIND_EXTENT,
    TREE_KIND_INODE,
    TREE_KIND_ALLOCATION,
    TREE_KIND_ACCOUNTING,
    TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
    TREE_KIND_DEADLIST,
    TREE_KIND_DELETED_INODES,
    TREE_KIND_TOMBSTONE,
    TREE_KIND_LINEAGE,
    TREE_KIND_LOGGED_OPERATIONS,
    TREE_KIND_RETIREMENT,
];

/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：先判 ①（盘上次序），再判 ②（发号次序）。读树表取条目、要往下走的两处都在
/// [`tree_table_entries_each_kind_at_most_once`] 之后、读任何一棵树之前调它：[`rebuild_version`]（可写挂载接着这一版发布，
/// 写者按种类装树表条目、断言树 ID 升序，走得到那条断言的镜像在任何写之前拒）与 [`walk_to_file`]。
///
/// # Errors
/// ① 不成立 ⇒ `InvariantViolated { invariant: "I-9.16", detail: "树表条目不按树 ID 严格升序" }`；
/// ① 成立、② 不成立 ⇒ 同一个不变量名，detail 是发号次序那一句。
pub(crate) fn tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(
    tree_table_entries: &[TreeTableEntry],
) -> Result<(), RecoveryFailure> {
    tree_table_entries_ascend_strictly_by_tree_identifier(tree_table_entries)?;
    tree_table_entries_ascend_strictly_in_the_issuing_order(tree_table_entries)
}

/// I-9.16 ①：树表条目按盘上的次序树 ID 严格升序（两条同号也拒：同一个号认成两棵树）。
///
/// # Errors
/// 相邻两条的树 ID 不是严格升序 ⇒ `InvariantViolated { invariant: "I-9.16", .. }`。
fn tree_table_entries_ascend_strictly_by_tree_identifier(
    tree_table_entries: &[TreeTableEntry],
) -> Result<(), RecoveryFailure> {
    if tree_table_entries
        .windows(2)
        .all(|pair| pair[0].tree < pair[1].tree)
    {
        return Ok(());
    }
    Err(RecoveryFailure::InvariantViolated {
        invariant: TREE_TABLE_ENTRIES_ORDERING_CONTRACT,
        detail: "树表条目不按树 ID 严格升序",
    })
}

/// I-9.16 ②：树表里的树按发号次序（[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]）相邻两棵的树 ID 严格升序。
/// 写者每次发布按这个次序装树表条目、断言它们升序（`transaction` 装树表那一步）：照抄上一版的号，这个次序就是上一版盘上的样子，
/// 那条断言靠的就是这一判（实审 A3c Q-A：两条条目的种类互换之后盘上次序仍按树 ID 升序、写者按种类装出来却降序）。
/// 只比树表里有的种类：缺哪一棵、有没登记的种类，由读那一棵的那一步报（[`rebuild_version`] 报「树表里没有那棵树」，
/// [`walk_to_file`] 报「树表里缺一棵有根的树」或「树的种类没登记」）；同一种至多一条已由 [`tree_table_entries_each_kind_at_most_once`] 判过。
///
/// # Errors
/// 发号次序上相邻两棵的树 ID 不是严格升序 ⇒ `InvariantViolated { invariant: "I-9.16", .. }`。
fn tree_table_entries_ascend_strictly_in_the_issuing_order(
    tree_table_entries: &[TreeTableEntry],
) -> Result<(), RecoveryFailure> {
    let tree_identifiers_in_the_issuing_order: Vec<TreeIdentifier> =
        TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER
            .iter()
            .filter_map(|kind| {
                tree_table_entries
                    .iter()
                    .find(|entry| entry.kind == *kind)
                    .map(|entry| entry.tree)
            })
            .collect();
    if tree_identifiers_in_the_issuing_order
        .windows(2)
        .all(|pair| pair[0] < pair[1])
    {
        return Ok(());
    }
    Err(RecoveryFailure::InvariantViolated {
        invariant: TREE_TABLE_ENTRIES_ORDERING_CONTRACT,
        detail: "树表里各棵树的树 ID 不按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、deleted_inodes、墓碑树、谱系树、logged_ops）严格升序",
    })
}

/// 从盘上按所选根重建出来的上一版。
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只重建一个，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
pub enum RebuiltVersion {
    /// 树表 0 条：所选根下面还没有发布过文件版本（mkfs 的第 0 代根、第一次可写挂载的暖机根），这一版只有根自己指着的两个单元。
    WithoutFile,
    WithFile(TransactionOutput),
    /// 有树、文件集合为空（D8（核心索引结构） 已定项 16 第 5 条）：判定与 [`crate::transaction::PoolVersion::of_a_version_of_trees`] 同一个
    /// （[`TransactionOutput::is_a_version_with_trees_and_no_file`]），不看 logged_ops、墓碑树、谱系树。
    WithTreesAndNoFile(TransactionOutput),
}

impl RebuiltVersion {
    /// 重建出来的一版有树时归哪个成员（判定同 `PoolVersion::of_a_version_of_trees`）。
    #[must_use]
    pub fn of_a_version_of_trees(version: TransactionOutput) -> RebuiltVersion {
        if version.is_a_version_with_trees_and_no_file() {
            RebuiltVersion::WithTreesAndNoFile(version)
        } else {
            RebuiltVersion::WithFile(version)
        }
    }
}

/// 重建上一版没做成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RebuildVersionFailure {
    /// 单元读不到、解不开，或走读同款的判定判红。
    Walk(RecoveryFailure),
    /// 所选根下面有文件版本，而调用方拿不出一条记录顶着这一版（环里一条自证过的记录都没有）。
    NoRecordStandingForFileVersion,
}

impl From<RecoveryFailure> for RebuildVersionFailure {
    fn from(failure: RecoveryFailure) -> Self {
        RebuildVersionFailure::Walk(failure)
    }
}

/// 从盘上按所选根重建「上一版」：全部角色的单元字节、指针、树表、分配记录、记账行与 inode 记录，交给发布路径当上一版
/// （照抄没重写的角色、经映射释放被换下的角色都靠它；可写挂载在恢复之后调）。`record_standing_for_root` 是所选根覆盖的最后一条记录
/// （同实例、同 checkpoint_txg 里带「本次发布末条」标志的那一条，D23（journal 的角色与格式） 已定项 14 注 1 读法乙；读不出时由调用方顶一条；树表 0 条时用不到）。
/// extent 根兼叶里的每条记录指的数据单元都读回来：一个文件跨多个单元时（并行线一）每个单元一个角色。
/// 豁免三类之外的单元位置提示读不出时经这一版的中央映射回退（D19（块指针的结构与宽度预算） 已定项 8）；
/// 提示与映射都读不出的**数据单元**不算失败：照抄它的位置项、不读内容，那一项的字节是空的（D19 已定项 5，用户 2026-09-24 定 N2）。
///
/// 读到手的每个节点与单元照冷走读判全（代码审阅第 24 / 33 条，用户定案「重建时把已经读到的节点照冷走读那套判全，坏了报错拒挂载，不多读盘」）：
/// 四棵树与中央映射树的节点头按指着它的指针与父条目核（`EveryHeaderAgainstItsReference`，含 key 严格递增），inode 树根按
/// `tree_root_checked_against_its_pointer` 核、层级 1，树表 key 宽与 I-7.8，inode 叶容器的 I-9.2 / I-1.2 / I-9.4，分配记录每盘一条、代与跨度，
/// 记账条目数与代、seq，映射条目数，读到手的数据单元按查找路径与 inode 记录核——与 [`walk_to_file`] 调同一组判定函数。不多读一个单元。
///
/// # Errors
/// 树表不是 0 条而 `record_standing_for_root` 是 `None` ⇒ `NoRecordStandingForFileVersion`；树表里同一种树有两条
/// （[`TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE`]）；树表条目不合 I-9.16（[`TREE_TABLE_ENTRIES_ORDERING_CONTRACT`]）；实例表、树表、映射树根读不到，
/// 树根或 inode 叶容器提示读不出且经映射也读不回（`MappingMiss` / `MappingStillUnreadable`），单元解不开、上面那几道判定判红 ⇒ 走读同款的错；
/// 记账行里没有 inode 号水位那一行 ⇒ `InodeNumberWatermarkRowMissingFromTheAccountingTree`。
///
/// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个）：这一版的分配记录落不落在单元区里按它判。
pub fn rebuild_version_in_the_unit_area_starting_at(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    record_standing_for_root: Option<JournalRecord>,
) -> Result<RebuiltVersion, RebuildVersionFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    // 树表 0 条的一版也要判：可写挂载接着它发的零单元发布照抄根记录里的实例表与树表指针。
    root_record_pointers_ascend_by_device(root)?;
    let instance_table_bytes =
        read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?;
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    tree_table_key_width_is_the_tree_tables(&tree_table)?;
    if tree_table.entries.is_empty() {
        return Ok(RebuiltVersion::WithoutFile);
    }
    let record =
        record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
    location_entries_of_the_record_ascend_by_device(&record)?;
    let record_bytes = record.to_bytes();
    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)
        .map_err(RecoveryFailure::from)?;
    // 可写挂载接着这一版发布，写者按种类装树表条目、断言树 ID 升序：盘上的次序与种类在这里先判（I-9.16），
    // 走得到那条断言的镜像在任何写之前拒。
    tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(&tree_table_entries)?;
    for entry in &tree_table_entries {
        tree_identifier_of_the_entry_is_below_the_watermark(entry, root)?;
    }
    let pointer_of = |kind: u16| {
        tree_table_entries
            .iter()
            .find(|entry| entry.kind == kind)
            .map(|entry| entry.root)
            .ok_or(RecoveryFailure::UnitMalformed {
                what: "树表里没有那棵树",
            })
    };
    let extent_pointer = pointer_of(TREE_KIND_EXTENT)?;
    let inode_root_pointer = pointer_of(TREE_KIND_INODE)?;
    let allocation_pointer = pointer_of(TREE_KIND_ALLOCATION)?;
    let accounting_pointer = pointer_of(TREE_KIND_ACCOUNTING)?;
    // 这一版那十三棵树各自的号照盘上读回来：十二棵按树表条目的种类取条目里的树 ID，中央映射树不进树表、取根记录里
    // 它那条根指针的出生树（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11）。接着这一版的发布照抄它们，不另发。
    let tree_of = |kind: u16| {
        tree_table_entries
            .iter()
            .find(|entry| entry.kind == kind)
            .map(|entry| entry.tree)
            .ok_or(RecoveryFailure::UnitMalformed {
                what: "树表里没有那棵树",
            })
    };
    let tree_identifiers = FileVersionTreeIdentifiers {
        extent: tree_of(TREE_KIND_EXTENT)?,
        inode: tree_of(TREE_KIND_INODE)?,
        allocation_records: tree_of(TREE_KIND_ALLOCATION)?,
        accounting: tree_of(TREE_KIND_ACCOUNTING)?,
        central_mapping: root.mapping_root.head.birth_tree,
        livelist: tree_of(TREE_KIND_LIVELIST)?,
        sparse_side_table: tree_of(TREE_KIND_SPARSE_SIDE_TABLE)?,
        deadlist: tree_of(TREE_KIND_DEADLIST)?,
        deleted_inodes: tree_of(TREE_KIND_DELETED_INODES)?,
        tombstone: tree_of(TREE_KIND_TOMBSTONE)?,
        lineage: tree_of(TREE_KIND_LINEAGE)?,
        logged_operations: tree_of(TREE_KIND_LOGGED_OPERATIONS)?,
        retirement: tree_of(TREE_KIND_RETIREMENT)?,
    };
    // 豁免三类之外的单元（数据单元、四棵树的根、inode 叶容器）位置提示读不出时经这一版的中央映射回退，与冷走读同一条
    // （D19（块指针的结构与宽度预算） 已定项 8）；映射树根、树表、实例表是自举豁免，只按根记录里的位置条目读。
    // 映射树根到第一次要回退、或走到它那一步时才读：提示都读得出的镜像上读序与报错次序照旧。
    let central_mapping_root = CentralMappingTreeWithBytesReadOnFirstUse::new(
        reader,
        root,
        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
    );
    let central_mapping_locations_of_key =
        |mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key);
    // 回退的次数这里不交出去：`RebuiltVersion` 与可写挂载今天没有接多跳观测点的口子（挂载态的读有，`MountedPoolForRead`）。
    let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
    // extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14 的两段）：整棵读回来，每个节点按位置核，节点进映射、提示读不出经映射回退。
    // 第一个文件的数据指针按单元号排，第 i 个就是文件第 i 个数据单元（一个文件跨多个单元，并行线一）；
    // 每个指的数据单元都读回来，照抄进这一版的角色（覆盖写经映射释放它们要这几个 key，写行与暖机照抄它们的字节）。
    // 提示与映射都读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错（D19（块指针的结构与宽度预算）
    // 已定项 5，用户 2026-09-24 定 N2）：它在这一版 `units` 里那一项的字节是空的。照抄它的发布只搬这一项、不写它的字节，
    // 重写它的发布按新内容装；释放它的发布照映射条目读盘核（已定项 5 硬规则 1），不读这里的字节。
    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
    let extent_tree_read = read_extent_tree(
        &ExtentTreeReading {
            tree: tree_identifiers.extent,
            judgement: ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference,
            root,
            expected_filesystem_identifier,
        },
        &extent_pointer,
        &mut |pointer: &NodePointer| {
            read_mapped_tree_node_via_hint_then_central_mapping(
                reader,
                pointer,
                MappedTreeNodeClass::IndexNode,
                &central_mapping_locations_of_key,
                &mut stale_location_hint_hops_not_exposed_by_this_reader,
            )
        },
    )?;
    let first_file_extents = extents_of_the_first_file(extent_tree_read)?;
    let mut data_pointers: Vec<DataPointer> =
        Vec::with_capacity(first_file_extents.data_pointers.len());
    let mut data_unit_contents_in_file_order: Vec<Vec<u8>> =
        Vec::with_capacity(first_file_extents.data_pointers.len());
    for data_pointer in first_file_extents.data_pointers.iter().copied() {
        let content_or_nothing_when_unreadable = match read_data_unit_via_hint_then_central_mapping(
            reader,
            &data_pointer,
            &central_mapping_locations_of_key,
            &mut stale_location_hint_hops_not_exposed_by_this_reader,
        )? {
            DataUnitReadThroughTheCentralMapping::Content(bytes) => bytes,
            DataUnitReadThroughTheCentralMapping::MissingFromTheMapping { .. }
            | DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation { .. } => {
                Vec::new()
            }
        };
        data_unit_contents_in_file_order.push(content_or_nothing_when_unreadable);
        data_pointers.push(data_pointer);
    }
    // inode 树根照冷走读核它的自描述（树 ID、key 宽、出生身份、fsid、key 区间，与 `read_mapped_tree_root` 同一道）与层级 1。
    let inode_root_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
        reader,
        &inode_root_pointer,
        MappedTreeNodeClass::IndexNode,
        &central_mapping_locations_of_key,
        &mut stale_location_hint_hops_not_exposed_by_this_reader,
    )?;
    let inode_root_node = tree_root_checked_against_its_pointer(
        &inode_root_bytes,
        tree_identifiers.inode,
        key_width_for_kind(TREE_KIND_INODE).expect("inode 树的 key 宽登记过"),
        &inode_root_pointer,
        root,
        expected_filesystem_identifier,
    )?;
    inode_tree_root_level_is_one(&inode_root_node)?;
    // 零条目的 inode 树根是 unlink 删掉树里最后一条记录之后的那一版（D8（核心索引结构） 已定项 16 第 5 条），不拒。
    // inode 树的每一片叶容器：根的条目按分隔 key 升序，逐条读回容器与它装的记录（D8（核心索引结构） 已定项 6）。
    let mut inode_leaf_containers: Vec<InodeLeafContainerVersion> = Vec::new();
    let mut inode_leaf_container_bytes: Vec<Vec<u8>> = Vec::new();
    for entry in &inode_root_node.entries {
        let (separator_key, identity_in_the_entry, leaf_pointer) =
            inode_internal_entry_parsed_and_judged(entry)?;
        let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
            reader,
            &leaf_pointer,
            MappedTreeNodeClass::PackedRecordUnit,
            &central_mapping_locations_of_key,
            &mut stale_location_hint_hops_not_exposed_by_this_reader,
        )?;
        let leaf =
            parse_packed_unit(&leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
                what: "inode 叶容器",
            })?;
        let records = inode_leaf_container_judged_against_its_entry(
            &leaf,
            InodeInternalEntry {
                separator_key,
                identity: identity_in_the_entry,
                child: leaf_pointer,
            },
            root,
            expected_filesystem_identifier,
        )?;
        if records.is_empty() {
            return Err(RecoveryFailure::UnitMalformed {
                what: "inode 叶容器没有记录",
            }
            .into());
        }
        inode_leaf_containers.push(InodeLeafContainerVersion {
            // 身份取容器头那一份：条目里的身份引用与它逐字相等是 I-9.2（条目身份与子头相符），上面刚判过。
            contents: InodeLeafContainer {
                identity: leaf.identity,
                // 分隔 key 取条目里那一份：诞生时是容器号，被左邻借过之后是被借那一版的最小 key（D8（核心索引结构） 已定项 6，N1）。
                separator_key_in_the_parent_entry: separator_key,
                records,
            },
            pointer: leaf_pointer,
        });
        inode_leaf_container_bytes.push(leaf_bytes);
    }
    // 第一个文件那条记录：unlink 最后一批删掉它之后的版本没有它，那时 extent 树里也一个数据单元都没有。
    let inode_record = inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|inode_record| inode_record.inode == FIRST_INODE_NUMBER)
        .copied();
    // 数据单元照冷走读核：个数对得上 inode size，读到手的每一个按查找路径与 inode 记录核；提示与映射都读不出的那几个（字节是空的）
    // 不读、不核（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定 N2）。
    match (&inode_record, data_pointers.is_empty()) {
        (None, true) => {}
        (None, false) => {
            return Err(RecoveryFailure::UnitMalformed {
                what: "inode 树里没有第一个文件那条记录，extent 树里却有它的数据单元",
            }
            .into());
        }
        (Some(inode_record_of_the_first_file), _) => {
            data_unit_count_matches_the_inode_size(
                data_pointers.len(),
                inode_record_of_the_first_file,
            )?;
            for (position, (data_pointer, content)) in data_pointers
                .iter()
                .zip(&data_unit_contents_in_file_order)
                .enumerate()
            {
                if content.is_empty() {
                    continue;
                }
                data_unit_judged_against_its_pointer_and_the_inode_record(
                    content,
                    DataUnitOfTheFirstFile {
                        position,
                        pointer: data_pointer,
                        extent_tree: tree_identifiers.extent,
                    },
                    inode_record_of_the_first_file,
                    expected_filesystem_identifier,
                )?;
            }
        }
    }
    // 删除那两棵树（D8（核心索引结构） 已定项 16、D18（块里携带什么信息） 已定项 24）：树表条目的根指针全零就是零根。
    // `deleted_inodes` 根是码 2 层级 0（条目 = 号 8 字节）；墓碑树根是码 2 层级 1（条目照 inode 树内部节点的 120 字节形态），孩子是码 3 类型 1 容器。
    let deleted_inodes_root_pointer = pointer_of(TREE_KIND_DELETED_INODES)?;
    let deleted_inodes_root_bytes = if deleted_inodes_root_pointer == NodePointer::empty_root() {
        None
    } else {
        Some(read_mapped_tree_node_via_hint_then_central_mapping(
            reader,
            &deleted_inodes_root_pointer,
            MappedTreeNodeClass::IndexNode,
            &central_mapping_locations_of_key,
            &mut stale_location_hint_hops_not_exposed_by_this_reader,
        )?)
    };
    let deleted_inodes: Vec<InodeNumber> = match &deleted_inodes_root_bytes {
        None => Vec::new(),
        Some(bytes) => {
            let node = tree_root_checked_against_its_pointer(
                bytes,
                tree_identifiers.deleted_inodes,
                key_width_for_kind(TREE_KIND_DELETED_INODES)
                    .expect("deleted_inodes 的 key 宽登记过"),
                &deleted_inodes_root_pointer,
                root,
                expected_filesystem_identifier,
            )?;
            if node.level != 0
                || node.entries.is_empty()
                || node
                    .entries
                    .iter()
                    .any(|entry| entry.len() != DELETED_INODES_KEY_WIDTH_IN_BYTES)
            {
                return Err(RecoveryFailure::UnitMalformed {
                    what: "deleted_inodes 根不是层级 0、条目宽 8 的非空节点",
                }
                .into());
            }
            inode_numbers_of_the_deleted_inodes_root_entries(&node.entries)
        }
    };
    // 墓碑树与谱系树（D18（块里携带什么信息） 已定项 24 第 1、5 条）：根恒为码 2，条目照 inode 树内部节点的 120 字节形态；根满长高之后
    // 根之下是类型段 0 的码 2 节点，层级 1 的节点的孩子是打包记录容器（墓碑类型 1、谱系类型 5）。两棵树读法相同，只差下面这几样。
    let mut read_container_tree_node = |pointer: &NodePointer, class: MappedTreeNodeClass| {
        read_mapped_tree_node_via_hint_then_central_mapping(
            reader,
            pointer,
            class,
            &central_mapping_locations_of_key,
            &mut stale_location_hint_hops_not_exposed_by_this_reader,
        )
    };
    let tombstone_root_pointer = pointer_of(TREE_KIND_TOMBSTONE)?;
    let tombstone_tree_read = read_container_tree(
        &ContainerTreeToRead {
            tree: tree_identifiers.tombstone,
            kind: TREE_KIND_TOMBSTONE,
            record_type: PACKED_TYPE_TOMBSTONE,
            record_width_in_bytes: TOMBSTONE_RECORD_BYTES,
            parse_record: TombstoneRecord::parse,
            what_index_node: "墓碑树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
            what_entry: "墓碑树内部条目",
            what_container: "墓碑容器",
            what_container_against_its_entry: "墓碑容器与父节点里指它的条目对不上（身份、类型 1、出生树、分隔 key = 容器号、记录宽 40、非空）",
            what_record: "墓碑记录",
        },
        &tombstone_root_pointer,
        root,
        expected_filesystem_identifier,
        &mut read_container_tree_node,
    )?;
    let lineage_root_pointer = pointer_of(TREE_KIND_LINEAGE)?;
    let lineage_tree_read = read_container_tree(
        &ContainerTreeToRead {
            tree: tree_identifiers.lineage,
            kind: TREE_KIND_LINEAGE,
            record_type: PACKED_TYPE_LINEAGE,
            record_width_in_bytes: LINEAGE_RECORD_BYTES,
            parse_record: LineageRecord::parse,
            what_index_node: "谱系树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
            what_entry: "谱系树内部条目",
            what_container: "谱系容器",
            what_container_against_its_entry: "谱系容器与父节点里指它的条目对不上（身份、类型 5、出生树、分隔 key = 容器号、记录宽 24、非空）",
            what_record: "谱系记录",
        },
        &lineage_root_pointer,
        root,
        expected_filesystem_identifier,
        &mut read_container_tree_node,
    )?;
    // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条：形态照墓碑树，容器打包记录类型 3、记录宽 34）：
    // 接着这一版发布要往开放容器追加、算内部层数，整棵读回来，同墓碑树、谱系树。
    let retirement_root_pointer = pointer_of(TREE_KIND_RETIREMENT)?;
    let retirement_tree_read = read_container_tree(
        &ContainerTreeToRead {
            tree: tree_identifiers.retirement,
            kind: TREE_KIND_RETIREMENT,
            record_type: PACKED_TYPE_RETIREMENT,
            record_width_in_bytes: RETIREMENT_RECORD_BYTES,
            parse_record: RetirementRecord::parse,
            what_index_node: "退役树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
            what_entry: "退役树内部条目",
            what_container: "退役容器",
            what_container_against_its_entry: "退役容器与父节点里指它的条目对不上（身份、类型 3、出生树、分隔 key = 容器号、记录宽 34、非空）",
            what_record: "退役记录",
        },
        &retirement_root_pointer,
        root,
        expected_filesystem_identifier,
        &mut read_container_tree_node,
    )?;
    // 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14）：整棵读回来，每个节点按位置核，节点进映射、提示读不出经映射回退。
    let allocation_tree_read = read_allocation_record_tree(
        &allocation_pointer,
        &AllocationRecordTreeGeometry::of_reader(reader),
        tree_identifiers.allocation_records,
        AllocationRecordTreeHeaderJudgement::EveryHeaderAgainstItsReference,
        root,
        expected_filesystem_identifier,
        &mut |pointer: &NodePointer| {
            read_mapped_tree_node_via_hint_then_central_mapping(
                reader,
                pointer,
                MappedTreeNodeClass::IndexNode,
                &central_mapping_locations_of_key,
                &mut stale_location_hint_hops_not_exposed_by_this_reader,
            )
        },
    )?;
    allocation_records_fit_the_pool_geometry(
        reader,
        unit_area_start,
        &allocation_tree_read.records,
    )?;
    let device_identities = reader.device_identities();
    allocation_records_are_one_placement_per_device_on_every_device(
        reader,
        unit_area_start,
        &allocation_tree_read.records,
        &device_identities,
    )?;
    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;
    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();
    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退
    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的
    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时
    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。
    let accounting_tree = read_code_two_tree(
        &accounting_pointer,
        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),
        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
        root,
        expected_filesystem_identifier,
        &mut |pointer: &NodePointer| {
            read_mapped_tree_node_via_hint_then_central_mapping(
                reader,
                pointer,
                MappedTreeNodeClass::IndexNode,
                &central_mapping_locations_of_key,
                &mut stale_location_hint_hops_not_exposed_by_this_reader,
            )
        },
    )?;
    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`
    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。
    every_accounting_entry_is_as_wide_as_its_field_table(
        &accounting_tree.leaf_entries_in_key_order,
    )?;
    let mut accounting_entries: Vec<AccountingEntry> =
        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());
    for bytes in &accounting_tree.leaf_entries_in_key_order {
        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(
            bytes, root,
        )?);
    }
    accounting_entry_count_is_three_plus_six_per_device(
        &accounting_tree.leaf_entries_in_key_order,
        root.checkpoint_txg,
        device_identities.len(),
    )?;
    // 记账行的统计量标签是盘上读来的 2 字节：这一版交出去之前判一次「inode 号水位那一行在」，往里就信
    // （`TransactionOutput::inode_number_watermark` 那句 expect 依赖的就是这一判；panic 面普查 R11）。
    // 要的是这一版自己写的那一行（代段 = 根的 txg）：那一统计量另有代段更早的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程），
    // 只剩台阶行时现行值取到的是一级台阶，不是这一版的水位。
    if !accounting_entries.iter().any(|entry| {
        entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg
    }) {
        return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());
    }
    // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。
    let mapping_tree = central_mapping_root.into_tree()?;
    mapping_entry_count_is_one_per_mapped_unit(
        mapping_tree.leaf_entries_in_key_order.len(),
        MappedUnitCounts {
            extent_tree_nodes: first_file_extents.nodes_in_bump_order.len(),
            allocation_record_tree_nodes: allocation_tree_read.nodes.len(),
            accounting_tree_nodes: accounting_tree.version.node_count(),
            inode_leaf_containers: inode_root_node.entries.len(),
            data_units: data_pointers.len(),
            deletion_tree_nodes: usize::from(deleted_inodes_root_bytes.is_some())
                + tombstone_tree_read
                    .as_ref()
                    .map_or(0, ContainerTreeRead::mapped_units)
                + lineage_tree_read
                    .as_ref()
                    .map_or(0, ContainerTreeRead::mapped_units)
                + retirement_tree_read
                    .as_ref()
                    .map_or(0, ContainerTreeRead::mapped_units),
        },
    )?;
    let mut mapping_keys: Vec<Vec<u8>> =
        Vec::with_capacity(mapping_tree.leaf_entries_in_key_order.len());
    for entry in &mapping_tree.leaf_entries_in_key_order {
        let (key, _locations) =
            parse_mapping_entry(entry).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "映射条目",
                entry_bytes: entry.len(),
                field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
            })?;
        mapping_keys.push(key);
    }
    let data_role_of_position = |position: usize| {
        TransactionUnit::Data(DataUnitIndexInFile(
            u64::try_from(position).expect("单元序号"),
        ))
    };
    let mut mapped_units: Vec<(TransactionUnit, Vec<u8>)> = data_pointers
        .iter()
        .enumerate()
        .map(|(position, data_pointer)| {
            (
                data_role_of_position(position),
                mapping_key_for_data(data_pointer.head, data_pointer.write_order),
            )
        })
        .collect();
    // extent 树与分配记录树的每个节点都进映射（码 2 树节点，D19（块指针的结构与宽度预算） 已定项 8），按 bump 次序。
    for (role, pointer, _) in &first_file_extents.nodes_in_bump_order {
        mapped_units.push((*role, mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer)));
    }
    for (position, container) in inode_leaf_containers.iter().enumerate() {
        mapped_units.push((
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
                position,
            )),
            mapping_key_for_node(UNIT_CLASS_PACKED, container.pointer),
        ));
    }
    mapped_units.push((
        TransactionUnit::InodeRoot,
        mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
    ));
    for (node, pointer, _) in &allocation_tree_read.nodes {
        mapped_units.push((
            role_of_allocation_record_tree_node(*node),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
        ));
    }
    // 记账树的每个节点都进映射（码 2 树节点，D19（块指针的结构与宽度预算） 已定项 8），按 bump 次序、根在最末。
    let accounting_shape = &accounting_tree.version.shape;
    for (node, pointer) in accounting_shape
        .nodes()
        .iter()
        .zip(&accounting_tree.version.pointers)
    {
        mapped_units.push((
            MultiLevelCodeTwoTree::Accounting.role_of_node(node.position, accounting_shape),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
        ));
    }
    // 删除那两棵树接在记账树后面（按树 ID），与发布路径的 `mapped_units` 同序。
    if deleted_inodes_root_bytes.is_some() {
        mapped_units.push((
            TransactionUnit::DeletedInodesRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, deleted_inodes_root_pointer),
        ));
    }
    // 墓碑树、谱系树：每片容器、根之下的码 2 节点（bump 次序）、根，与发布路径的 `mapped_units` 同序。
    for (tree_read, roles) in [
        (
            tombstone_tree_read
                .as_ref()
                .map(ContainerTreeRead::units_in_bump_order),
            CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE,
        ),
        (
            lineage_tree_read
                .as_ref()
                .map(ContainerTreeRead::units_in_bump_order),
            CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
        ),
        (
            retirement_tree_read
                .as_ref()
                .map(ContainerTreeRead::units_in_bump_order),
            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
        ),
    ] {
        for (unit_of_the_tree, pointer, _) in tree_read.unwrap_or_default() {
            mapped_units.push((
                roles.of(unit_of_the_tree),
                mapping_key_for_node(unit_of_the_tree.unit_class(), pointer),
            ));
        }
    }
    let unit =
        |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
            slot: locations[0].slot,
            identity,
            bytes,
        };
    let mut units: Vec<PublishedUnit> = data_pointers
        .iter()
        .zip(data_unit_contents_in_file_order)
        .enumerate()
        .map(|(position, (data_pointer, bytes))| {
            unit(
                data_role_of_position(position),
                &data_pointer.locations,
                bytes,
            )
        })
        .collect();
    for (role, pointer, bytes) in first_file_extents.nodes_in_bump_order {
        units.push(unit(role, &pointer.locations, bytes));
    }
    for ((position, container), bytes) in inode_leaf_containers
        .iter()
        .enumerate()
        .zip(inode_leaf_container_bytes)
    {
        units.push(unit(
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
                position,
            )),
            &container.pointer.locations,
            bytes,
        ));
    }
    units.push(unit(
        TransactionUnit::InodeRoot,
        &inode_root_pointer.locations,
        inode_root_bytes,
    ));
    let allocation_record_tree = allocation_tree_read.version();
    for (node, pointer, bytes) in allocation_tree_read.nodes {
        units.push(unit(
            role_of_allocation_record_tree_node(node),
            &pointer.locations,
            bytes,
        ));
    }
    // 记账树、删除那两棵树与中央映射树的每个节点，按 bump 次序（树内先叶后根），与发布路径装出来的 `units` 同序。
    let mut deletion_tree_units: Vec<PublishedUnit> = Vec::new();
    if let Some(bytes) = deleted_inodes_root_bytes {
        deletion_tree_units.push(unit(
            TransactionUnit::DeletedInodesRoot,
            &deleted_inodes_root_pointer.locations,
            bytes,
        ));
    }
    for (tree_read, roles) in [
        (
            tombstone_tree_read
                .as_ref()
                .map(ContainerTreeRead::units_in_bump_order),
            CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE,
        ),
        (
            lineage_tree_read
                .as_ref()
                .map(ContainerTreeRead::units_in_bump_order),
            CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
        ),
        (
            retirement_tree_read
                .as_ref()
                .map(ContainerTreeRead::units_in_bump_order),
            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
        ),
    ] {
        for (unit_of_the_tree, pointer, bytes) in tree_read.unwrap_or_default() {
            deletion_tree_units.push(unit(roles.of(unit_of_the_tree), &pointer.locations, bytes));
        }
    }
    let mut deletion_tree_units_not_yet_placed = Some(deletion_tree_units);
    for (tree, read) in [
        (MultiLevelCodeTwoTree::Accounting, &accounting_tree),
        (MultiLevelCodeTwoTree::CentralMapping, &mapping_tree),
    ] {
        if tree == MultiLevelCodeTwoTree::CentralMapping {
            units.extend(
                deletion_tree_units_not_yet_placed
                    .take()
                    .expect("删除那两棵树的单元只在中央映射树之前放一次"),
            );
        }
        let shape = &read.version.shape;
        for ((node, pointer), bytes) in shape
            .nodes()
            .iter()
            .zip(&read.version.pointers)
            .zip(&read.node_bytes)
        {
            units.push(unit(
                tree.role_of_node(node.position, shape),
                &pointer.locations,
                bytes.clone(),
            ));
        }
    }
    units.extend([
        unit(
            TransactionUnit::TreeTable,
            &root.tree_table.locations,
            tree_table_bytes,
        ),
        unit(
            TransactionUnit::InstanceTable,
            &root.instance_table.locations,
            instance_table_bytes,
        ),
    ]);
    let transaction_number_on_the_record_standing_for_this_root = record.transaction;
    let rebuilt = TransactionOutput {
        root: *root,
        record,
        record_bytes,
        earlier_records_of_this_publish: Vec::new(),
        units,
        rewritten: Vec::new(),
        data_pointers,
        mapping_keys,
        allocation_records,
        allocation_record_tree,
        extent_tree: first_file_extents.version,
        accounting_entries,
        accounting_tree: accounting_tree.version,
        central_mapping_tree: mapping_tree.version,
        tree_table_entries,
        tree_identifiers,
        inode_record,
        deleted_inodes,
        tombstone_containers: tombstone_tree_read.as_ref().map_or_else(Vec::new, |read| {
            read.containers
                .iter()
                .map(|(contents, pointer, _)| TombstoneContainerVersion {
                    contents: contents.clone(),
                    pointer: *pointer,
                })
                .collect()
        }),
        tombstone_tree_index_nodes: tombstone_tree_read.as_ref().map_or_else(
            ContainerTreeIndexNodesVersion::zero_root,
            ContainerTreeRead::index_nodes_version,
        ),
        lineage_containers: lineage_tree_read.as_ref().map_or_else(Vec::new, |read| {
            read.containers
                .iter()
                .map(|(contents, pointer, _)| LineageContainerVersion {
                    contents: contents.clone(),
                    pointer: *pointer,
                })
                .collect()
        }),
        lineage_tree_index_nodes: lineage_tree_read.as_ref().map_or_else(
            ContainerTreeIndexNodesVersion::zero_root,
            ContainerTreeRead::index_nodes_version,
        ),
        retirement_containers: retirement_tree_read.as_ref().map_or_else(Vec::new, |read| {
            read.containers
                .iter()
                .map(|(contents, pointer, _)| RetirementContainerVersion {
                    contents: contents.clone(),
                    pointer: *pointer,
                })
                .collect()
        }),
        retirement_tree_index_nodes: retirement_tree_read.as_ref().map_or_else(
            ContainerTreeIndexNodesVersion::zero_root,
            ContainerTreeRead::index_nodes_version,
        ),
        inode_leaf_containers,
        mapped_units,
        released: Vec::new(),
        quarantined_after_release_checksum_mismatch: Vec::new(),
        key_order_mismatches: 0,
        writes: WritesByStructureKind::NOTHING_WRITTEN,
        // 重建出来的这一版属于**旧**实例。今天每条恢复路径（普通挂载、回退、切换）之后都要取新实例代号，
        // 而事务号按实例各算各的、从 1 重新起（D23（journal 的角色与格式） 已定项 7），所以这个值不会被拿去接着发布——
        // 写行那次发布显式传 0。将来真要在同一个实例上续发，得由调用方扫环算出这个实例的最大非 0 事务号传进来，
        // 这里这条记录上的事务号在它是空发布时是 0，单独拿它续号会重号。
        // 钉住这个前提的是池级 checker 的 I-8.7（实例内事务号不重号）：同一实例按计数器相邻的两条非 0 记录，
        // 后一条事务号要严格大于前一条。前提一旦失效，那条不变量会在盘上判红——2026-09-21 实测把发布路径改回
        // 「上一条记录的事务号 + 1」，33 条里只有 I-8.7 judged 出来。
        highest_transaction_number_in_this_instance:
            transaction_number_on_the_record_standing_for_this_root,
    };
    pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;
    Ok(RebuiltVersion::of_a_version_of_trees(rebuilt))
}

/// 一条根的树表里用户可见的两棵树（inode 树、extent 树）的根指针，取树表条目里那 86 字节的盘上原样；树表里没有那棵树的条目时 `None`。
/// 抬 F 的上限要数「非空」的有效根：一条有效根算非空 ⟺ 这两样与它前一条有效根的不同（D16（发布语义） 已定项 1「非空」从盘上怎么认，
/// 2026-09-17 用户定案）；树表单元自己的落点每次发布都变，不拿它比。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserVisibleTreeRootPointers {
    pub inode_tree: Option<Vec<u8>>,
    pub extent_tree: Option<Vec<u8>>,
}

impl UserVisibleTreeRootPointers {
    /// 树表里没有这两棵树的条目（mkfs 的第 0 代树表就是这样）：最旧的有效根没有前一条时拿它比。
    pub const ABSENT: Self = Self {
        inode_tree: None,
        extent_tree: None,
    };
}

/// 读一条根的树表，取 inode 树与 extent 树的根指针盘上字节。
///
/// # Errors
/// 树表单元读不到、解不开；树表条目解不开、同一种树出现两条（[`TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE`]，先于下一条判），或种类没登记。
pub fn user_visible_tree_root_pointers(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<UserVisibleTreeRootPointers, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;
    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
    // 两串逐条对应：`tree_table_entries` 是 `tree_table.entries` 按次序逐条解出来的，条数相同。
    for (entry, entry_bytes) in tree_table_entries.iter().zip(&tree_table.entries) {
        let slot_for_this_tree = match entry.kind {
            TREE_KIND_INODE => &mut pointers.inode_tree,
            TREE_KIND_EXTENT => &mut pointers.extent_tree,
            TREE_KIND_ALLOCATION
            | TREE_KIND_ACCOUNTING
            | TREE_KIND_LIVELIST
            | TREE_KIND_SPARSE_SIDE_TABLE
            | TREE_KIND_DEADLIST
            | TREE_KIND_DELETED_INODES
            | TREE_KIND_TOMBSTONE
            | TREE_KIND_LINEAGE
            | TREE_KIND_LOGGED_OPERATIONS
            | TREE_KIND_RETIREMENT => continue,
            _ => {
                return Err(RecoveryFailure::UnitMalformed {
                    what: "树的种类没登记",
                })
            }
        };
        // 同一种树至多一条，上面判过：这一格在这一趟里只填一次。
        *slot_for_this_tree = Some(TreeTableEntry::root_pointer_bytes(entry_bytes).to_vec());
    }
    Ok(pointers)
}

/// 一条根的树表单元里每一条条目的盘上原样（每条是树表单元自述的条目宽那么多字节，D8（核心索引结构） 已定项 8 的字段表是 200），
/// 按盘上次序；树表 0 条的一版交空表。只读、不判条目内容（种类登没登记、同一种几条都不判）：挂载透明的判官
/// （`singlefs_harness::mount_transparency`）按字节区间比 R1 与 R2 的条目，它要的是原字节，不是解出来的几样。
///
/// # Errors
/// 树表单元读不到（两份位置都读不出或校验和对不上）、解不开。
pub fn tree_table_entry_bytes_of_root<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
) -> Result<Vec<Vec<u8>>, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    Ok(tree_table.entries)
}

/// 一版的根记录 [0, `ROOT_RECORD_BYTES`) 的字节：根环里有一槽自证过、解出来与 `root` 逐字段相同的，交那一槽的原字节
/// （按根环槽的次序第一个）；没有的（由记录施加出来、根槽从没落盘的那一版）交 `root` 按字段表编出来的那几字节
/// （[`RootRecord::to_slot`]：算法类型、nonce、MAC 那 29 字节与 flags 其余位写 0）。只读。
/// 挂载透明的判官按字节区间比 R1 与 R2 的根记录用它（根记录里 `parse_slot` 不进内存的那几样——留位、flags 其余位——只有原字节里有）。
#[must_use]
pub fn root_record_bytes_of_the_version<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    root: &RootRecord,
) -> Vec<u8> {
    let record_bytes = usize::try_from(ROOT_RECORD_BYTES).expect("457");
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    every_root_ring_slot(immutable_sizes)
        .into_iter()
        .find_map(|ring_slot| {
            let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
            let offset = slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
            let bytes = reader.read(device, offset, root_slot_bytes)?;
            (RootRecord::parse_slot(&bytes, &root.filesystem_identifier).as_ref() == Some(root))
                .then(|| bytes[..record_bytes].to_vec())
        })
        .unwrap_or_else(|| root.to_slot(root_slot_bytes)[..record_bytes].to_vec())
}

/// 择到的系统配置（[`choose_system_configuration`]）那一槽的盘上原样（整槽 `SYSTEM_CONFIGURATION_SLOT_BYTES`）：在择到的那份自述的本盘
/// （`immutable.this_device`）两个槽里找解出来与它相同的那一槽；择不到系统配置、或两槽都对不上时 None。只读。
/// 挂载透明的判官按字节区间比 R1 与 R2 的系统配置用它（系统运行配置那 36 字节 `parse_slot` 不进内存，只有原字节里有）。
#[must_use]
pub fn chosen_system_configuration_slot_bytes(reader: &dyn PoolReader) -> Option<Vec<u8>> {
    let chosen = choose_system_configuration(reader).ok()?;
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let slot_spacing = u64::from(chosen.immutable.sizes.fixed_structure_slot_spacing);
    (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).find_map(|slot_index| {
        let bytes = reader.read(
            chosen.immutable.this_device,
            DeviceOffsetInBytes(slot_index * slot_spacing),
            slot_bytes,
        )?;
        (SystemConfiguration::parse_slot(&bytes).as_ref() == Ok(&chosen)).then_some(bytes)
    })
}

/// 根环全部自证过的根里最大的实例代号（取号的 max 里「根环里全部根记录的实例代号」那一半）；一条都没有时 None。
#[must_use]
pub fn highest_root_instance<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> Option<InstanceGeneration> {
    let mut highest: Option<InstanceGeneration> = None;
    visit_valid_roots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
        |root| {
            highest = Some(highest.map_or(root.instance, |current| current.max(root.instance)));
        },
    );
    highest
}

/// 一块盘两槽里自证过、fsid 与本池相同的系统配置。取号的 max（D18（块里携带什么信息） 已定项 11）与系统配置槽写的世代号
/// （D22（单元原子性怎么合成） 已定项 16，逐盘计）都按这个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）。
#[must_use]
pub fn verified_system_configuration_slots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    slot_spacing_in_bytes: u64,
    filesystem_identifier: &[u8; 16],
) -> Vec<SystemConfiguration> {
    system_configuration_slots_of_one_device_read_once(
        reader,
        device,
        slot_spacing_in_bytes,
        filesystem_identifier,
    )
    .self_verified
}

/// 一块盘两个系统配置槽各读一次的结局（D22（单元原子性怎么合成） 已定项 8 射程「可写挂载读系统配置的见证按盘读」，S-辨：
/// 一槽撕裂、另一槽自证过，与设备读不出分开）：自证过、fsid 与本池相同的那几份，与设备报读错（`PoolReader::read` 交空）的槽数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemConfigurationSlotsOfOneDevice {
    pub device: DeviceIdentity,
    pub self_verified: Vec<SystemConfiguration>,
    pub slots_reporting_a_read_error: u64,
}

impl SystemConfigurationSlotsOfOneDevice {
    /// 两槽都自证过（R2 里「有一块盘两槽都自证过」那一半）。
    #[must_use]
    pub fn both_slots_self_verified(&self) -> bool {
        u64::try_from(self.self_verified.len()).expect("一块盘两个槽，装得进 u64")
            == SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
    }
}

/// 一块盘两槽各读一次：读得出的按整槽校验和与 fsid 判（每区槽数 S 越界的槽与自证不过的槽一样丢掉：走到这一步之前
/// `choose_system_configuration` 已经在同一个池上判过区间、越界的池根本挂不上，这里再报一次越界也没有第二种处置），读不出的计数。
#[must_use]
pub fn system_configuration_slots_of_one_device_read_once<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    slot_spacing_in_bytes: u64,
    filesystem_identifier: &[u8; 16],
) -> SystemConfigurationSlotsOfOneDevice {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut self_verified = Vec::new();
    let mut slots_reporting_a_read_error = 0;
    for offset in [0, slot_spacing_in_bytes] {
        match reader.read(device, DeviceOffsetInBytes(offset), slot_bytes) {
            None => slots_reporting_a_read_error += 1,
            Some(bytes) => match SystemConfiguration::parse_slot(&bytes) {
                Ok(system_configuration) => {
                    if system_configuration.immutable.filesystem_identifier
                        == *filesystem_identifier
                    {
                        self_verified.push(system_configuration);
                    }
                }
                // 读成而自证不过（magic 与整槽校验和、节点大小，C840 当自证不过）：报给读者（可写挂载这次挂载共用的那一份记进读坏账、
                // 丢掉这一槽，下一次读它穿到盘上）。mkfs 把每块盘两槽都写过，这两样不过的槽不是「本来就空」。
                Err(
                    SystemConfigurationSlotRefusal::NotSelfDescribing
                    | SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant { .. },
                ) => reader.note_that_the_bytes_read_fail_self_verification(
                    device,
                    DeviceOffsetInBytes(offset),
                    slot_bytes,
                ),
                // 布局不认识、每区槽数越界：是一份完整的系统配置、不是坏槽（择系统配置那一步已按它们判过整池），不算读坏。
                Err(
                    SystemConfigurationSlotRefusal::IncompatBitsNotRecognized { .. }
                    | SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange(_),
                ) => {}
            },
        }
    }
    SystemConfigurationSlotsOfOneDevice {
        device,
        self_verified,
        slots_reporting_a_read_error,
    }
}

/// 系统配置见证判不出（R2，D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」，零轮）：
/// 判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。可写挂载读阶段的见证读（`mount::newer_publish_witness`）与
/// 取号那一刻读见证值（`transaction::self_verified_system_configurations_of_every_device`）都按它判，一处定义。
/// 撕裂的系统配置轮换（读得出、自证不过，设备没报错）不算判不出；一块盘报读错而另一块盘两槽都自证过也不算。
#[must_use]
pub fn system_configuration_witness_is_undecidable(
    slots_of_every_device: &[SystemConfigurationSlotsOfOneDevice],
) -> bool {
    slots_of_every_device
        .iter()
        .any(|slots| slots.slots_reporting_a_read_error > 0)
        && !slots_of_every_device
            .iter()
            .any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)
}

/// 全环扫描：每盘每个记录槽都读（有提示时只读提示的那些），两份镜像任一份自证过即算在；fsid 不符的不算数。
/// 没有提示时逐槽列偏移只列到那块盘的末尾：环长是盘上读来的 8 字节，读者只按「环末端不越过同一槽记着的单元区起点、在飞上限装得进 4 字节」
/// 判它（`system_configuration_values_this_reader_accepts`），不按盘的字节数判；盘末尾之后的槽本来就读不出（`PoolReader::read` 交空），
/// 不列它们，扫环的内存就不由那 8 字节定。
#[must_use]
pub fn scan_journal(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> BTreeMap<(InstanceGeneration, u64), JournalRecord> {
    let expected_filesystem_identifier =
        unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier);
    let ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let mut scanned_records_in_read_order: Vec<JournalRecord> = Vec::new();
    for device in reader.device_identities() {
        let ring_bytes_inside_the_device = reader
            .device_size_in_bytes(device)
            .map_or(0, |device_bytes| {
                ring_bytes.min(device_bytes.saturating_sub(ring_start.0))
            });
        let offsets = reader
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
            .unwrap_or_else(|| {
                (0..ring_bytes_inside_the_device / JOURNAL_RECORD_BYTES)
                    .map(|record_index| {
                        DeviceOffsetInBytes(ring_start.0 + record_index * JOURNAL_RECORD_BYTES)
                    })
                    .collect()
            });
        for offset in offsets {
            let Some(bytes) = reader.read(device, offset, record_bytes) else {
                continue;
            };
            if let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) {
                scanned_records_in_read_order.push(record);
            }
        }
    }
    // 同一 (实例代号, 计数器) 先到先得（按上面的盘次序、环内偏移次序，第一份读到的留下）：留哪一份在
    // `singlefs_replay_prefix::first_copy_of_each_record_key` 里（Verus 验的是那一份），这里照它交回的位置取。
    let scanned_record_keys: Vec<(u32, u64)> = scanned_records_in_read_order
        .iter()
        .map(|record| (record.instance.0, record.counter))
        .collect();
    let first_copies = first_copy_of_each_record_key(&scanned_record_keys);
    let mut records = BTreeMap::new();
    for (position, record) in scanned_records_in_read_order.into_iter().enumerate() {
        if first_copies.get(&(record.instance.0, record.counter)) == Some(&position) {
            records.insert((record.instance, record.counter), record);
        }
    }
    records
}

/// 一条记录的反向链对不对得上本实例内逻辑前一条（I-8.6（反向链算法））。封闭集合，`match` 不写通配臂。
enum BackChainJudgement {
    /// 前一条在盘上，它的头算出来的链值等于这一条的反向链。
    Holds,
    /// 前一条在盘上，链值不等：这一条不进重放前缀（I-8.6「不等的记录不进重放前缀」）。
    Broken,
    /// 本实例内逻辑前一条不在盘上（扫环时没读到，或重读那一槽时每一份都解不出它）：判不了，不拿它断前缀。
    PreviousRecordNotOnDisk,
}

/// 判一条记录的反向链（I-8.6（反向链算法）；D23（journal 的角色与格式） 已定项 8 / 已定项 10 / 已定项 19 ②）：
/// 本实例内逻辑前一条 = 同一实例、计数器小 1 的那一条；链值 = CRC-32C(它在盘上那一份的 487 字节头，`header_csum` 按 0 参与)。
/// 按盘上的原样字节算、不按解出来的字段重写（重写会把头里读者不看的字节抹成 0），与池级 checker 判 I-8.6 同一个输入；
/// 取的是 [`PoolReader::device_identities`] 次序里第一份解得出、且与扫环时收下的那一条相同的镜像——扫环按同一次序收记录。
/// 前一条不在盘上不判：链是按实例的，跨实例边界本来不比，前缀里的记录都接在所选根覆盖的那一条之后，
/// 读不出那一条时链首另有「本次发布内序号为 1」一关（D23（journal 的角色与格式） 已定项 14 注 1）。
fn judge_back_chain_against_the_previous_record_on_disk(
    reader: &dyn PoolReader,
    record: &JournalRecord,
    ring_bytes: u64,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> BackChainJudgement {
    let Some(previous_counter) = record
        .counter
        .checked_sub(1)
        .filter(|previous_counter| *previous_counter >= FIRST_JOURNAL_COUNTER)
    else {
        return BackChainJudgement::PreviousRecordNotOnDisk;
    };
    let Some(previous) = records.get(&(record.instance, previous_counter)) else {
        return BackChainJudgement::PreviousRecordNotOnDisk;
    };
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let offset = record_offset(previous_counter, ring_bytes);
    // 迭代次数的上界是池里的盘数；第一份解得出、与扫到的那一条相同的镜像就是出口。
    for device in reader.device_identities() {
        let Some(bytes) = reader.read(device, offset, record_bytes) else {
            continue;
        };
        if JournalRecord::parse(&bytes, previous.filesystem_identifier).as_ref() == Some(previous) {
            return if back_chain_of(&bytes) == record.back_chain {
                BackChainJudgement::Holds
            } else {
                BackChainJudgement::Broken
            };
        }
    }
    BackChainJudgement::PreviousRecordNotOnDisk
}

/// [`replay_journal`] 交给取前缀核的「新根从哪来」：核不看它，施加了发布就交回最后施加的那次发布末条带的那一项，
/// 一次都没施加就交回调用方给的那一项。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NewRootSource {
    ChosenRootItself,
    NewRootSegmentOfTheRecordAtKeyOrderIndex(usize),
}

/// 所选根的 checkpoint_txg 是 u64 的最大值时 [`replay_journal`] 报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
pub const CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR: &str =
    "所选根的 checkpoint_txg 是 u64 的最大值：接在它后面的那次发布取不到 txg";

/// 一条记录点名的每个单元的两份（两条位置条目）都读得出、整单元 CRC-32C 都等于条目里的（前缀第四条「两份都验过才施加」，
/// D23（journal 的角色与格式） 已定项 14）；点名项的单元种类不认识就算验不过。只给施加前验点名单元（[`replay_journal`]）用；
/// 可写挂载判「水位之上一次缺记录的发布点名的单元验不过」用的是另一份（[`every_named_unit_of_the_record_has_a_copy_that_verifies`]）。
pub(crate) fn every_named_unit_of_the_record_verifies<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    record: &JournalRecord,
) -> bool {
    record.named.iter().all(|named| {
        let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
            return false;
        };
        named
            .locations
            .iter()
            .all(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))
    })
}

/// 一条记录点名的每个单元都至少有一份（一条位置条目）读得出、整单元 CRC-32C 等于条目里的；点名项的单元种类不认识就算验不过。
/// 可写挂载判「水位之上一次缺记录的发布点名的单元验不过」（F1 乙，`mount::publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`）
/// 只用这一份：某个点名单元两份都验不过才算验不过（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行与第三节第 1 条，
/// 攻方 `F1_ANY`）。一份副本坏是单故障（D22（单元原子性怎么合成） 已定项 8 射程），按「两份都验过」判会让写记录中途断电、根没落的那次发布
/// 一份副本坏时每次可写挂载都拒、没有出口。施加前的点名单元验证照旧两份都要验过（[`every_named_unit_of_the_record_verifies`]），两处不共用。
pub(crate) fn every_named_unit_of_the_record_has_a_copy_that_verifies<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    record: &JournalRecord,
) -> bool {
    record.named.iter().all(|named| {
        let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
            return false;
        };
        named
            .locations
            .iter()
            .any(|location| copy_named_by_the_location_entry_verifies(reader, location, unit_bytes))
    })
}

/// 点名项一条位置条目指的那一份读得出、整单元 CRC-32C 等于条目里的。
fn copy_named_by_the_location_entry_verifies<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    location: &LocationEntry,
    unit_bytes: usize,
) -> bool {
    reader
        .read(
            location.device,
            location.slot.to_device_offset(),
            unit_bytes,
        )
        .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
}

/// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
///
/// 链首锚在所选根覆盖的最后一条：所选根那次发布里带「本次发布末条」标志的那一条（已定项 14 注 1，读法乙；
/// 不取读得出的同 txg 记录里 jsn 最大的那条）。
/// 施加的单位是一次发布（第五条）：前四条判出来的前缀里，一次发布的记录要一直走到带末条标志（记录标志位 0）的那一条
/// 才整体施加；前缀停在一次发布中间（末条没到、断号、校验不过、下一条换了 txg、一次发布之内跳号、
/// 一个事务的提交标记没出现）⇒ 那次发布整体不施加。读者规则另外两格（D23（journal 的角色与格式） 已定项 4）同样断链、那次发布不施加：
/// 一次发布的首条序号不是 1（断在这一条）；带末条标志的那一条之后同一 (实例代号, checkpoint_txg) 里还有读得出的记录（断在带标志的那一条）。
///
/// # Errors
/// 所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写）⇒
/// `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`，一条记录都没施加；
/// 所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布没有 txg 可取）⇒ `UnitMalformed`
/// （[`CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`]），一条记录都没施加。
///
/// # Panics
/// `records` 的键不等于记录自己的 (实例代号, 计数器)，或有记录的计数器是 u64 的最大值：这两条是核的前置条件，调用方违反就是 bug。
pub fn replay_journal(
    reader: &dyn PoolReader,
    root: &RootRecord,
    ring_bytes: u64,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    verify_named_units: bool,
) -> Result<(JournalScanReport, RootRecord), RecoveryFailure> {
    let mut report = JournalScanReport {
        valid_records: records.len(),
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
        maximum_applied_transaction: 0,
    };
    // 取前缀那一截在 `singlefs_replay_prefix::replay_core` 里（Verus 验的是那一份）；这里只做三件：把记录表摊成核的入参、
    // 把两处读盘（反向链重读、点名单元校验）包成回调交给核、照核交回的结果拼报告与新根。
    let in_flight_limit =
        usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
    // 核的两条前置条件 Verus 只在核里假定、不在调用点查，在这里断言：
    for (key, record) in records {
        assert_eq!(
            *key,
            (record.instance, record.counter),
            "记录表的键要等于记录自己的 (实例代号, 计数器)：scan_journal 与各调用方都按它建键，键序因此就是核要的「按 (实例代号, 计数器) 严格升序」；\
             不等说明调用方建错了表，核的前置条件不成立"
        );
        assert!(
            record.counter < u64::MAX,
            "计数器是盘上 6 字节读来的（D23（journal 的角色与格式） 已定项 9 的 48 位），到不了 u64 的最大值；核要给链上下一号加一"
        );
    }
    let records_in_key_order: Vec<&JournalRecord> = records.values().collect();
    let core_records: Vec<CoreRecord<NewRootSource>> = records_in_key_order
        .iter()
        .enumerate()
        .map(|(index_in_key_order, record)| CoreRecord {
            instance: PrefixKernelInstanceGeneration(record.instance.0),
            counter: JournalCounter(record.counter),
            checkpoint_txg: PrefixKernelCheckpointTxg(record.checkpoint_txg.0),
            transaction: TransactionNumber(record.transaction),
            is_commit: record.is_commit,
            ordinal_within_publish: PrefixKernelJournalRecordOrdinalWithinPublish(
                record.ordinal_within_publish.0,
            ),
            place_in_publish: match record.place_in_publish {
                JournalRecordPlaceInPublish::LastRecordOfThePublish => {
                    PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish
                }
                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => {
                    PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                }
            },
            new_root_source: NewRootSource::NewRootSegmentOfTheRecordAtKeyOrderIndex(
                index_in_key_order,
            ),
        })
        .collect();
    // 反向链（D23（journal 的角色与格式） 已定项 7「jsn 严格连续 + 校验和 + 反向链」；I-8.6（反向链算法））：
    // 校验和自洽、jsn 恰好接得上而链值不等的记录（上一条时间线的残留、改过的镜像）断在这一条，它所在的这次发布整体不施加。
    let back_chain_broken =
        |index: usize| match judge_back_chain_against_the_previous_record_on_disk(
            reader,
            records_in_key_order[index],
            ring_bytes,
            records,
        ) {
            BackChainJudgement::Holds | BackChainJudgement::PreviousRecordNotOnDisk => false,
            BackChainJudgement::Broken => true,
        };
    let named_units_verified =
        |index: usize| every_named_unit_of_the_record_verifies(reader, records_in_key_order[index]);
    let named_unit_verification = if verify_named_units {
        NamedUnitVerification::Verify
    } else {
        NamedUnitVerification::Skip
    };
    let outcome = match replay_core(
        PrefixKernelInstanceGeneration(root.instance.0),
        PrefixKernelCheckpointTxg(root.checkpoint_txg.0),
        NewRootSource::ChosenRootItself,
        &core_records,
        in_flight_limit,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
    ) {
        Ok(outcome) => outcome,
        // 核带出它看到的那几条计数器（按升序），直接拼报错。
        Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
            counters,
        }) => {
            return Err(
                RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                    counters: counters.iter().map(|counter| counter.0).collect(),
                },
            );
        }
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {
            return Err(RecoveryFailure::UnitMalformed {
                what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
            });
        }
    };
    report.above_water = outcome.report.above_water;
    report.prefix_applied = outcome.report.prefix_applied;
    report.verification_passed = outcome.report.verification_passed;
    report.verification_failed = outcome.report.verification_failed;
    report.maximum_applied_transaction = outcome.report.maximum_applied_transaction.0;
    let rebuilt = match outcome.root_source_after_replay {
        NewRootSource::ChosenRootItself => *root,
        NewRootSource::NewRootSegmentOfTheRecordAtKeyOrderIndex(index_in_key_order) => {
            let record = records_in_key_order[index_in_key_order];
            // 每条记录都带整次发布的新根段（已定项 15），施加末条那一份：把所选根的七个字段换成记录里的——实例表指针、卸载记号、
            // 分配记录树根从新根段来，不再照抄所选根（用户 2026-10-02「回到甲」；`research/prompts/m3-ring-r3-main-verification.md`
            // 第二节第 1、2 行：照抄时首个文件、卸载两种发布施加出错根，专用清扫发布施加出旧实例表链）。fsid 新根段里没有，照所选根。
            RootRecord {
                filesystem_identifier: root.filesystem_identifier,
                unmount_marker: record.new_unmount_marker,
                instance: record.instance,
                checkpoint_txg: record.checkpoint_txg,
                tree_table: record.new_tree_table,
                tree_identifier_watermark: record.new_tree_identifier_watermark,
                rollback_floor: record.new_rollback_floor,
                instance_table: record.new_instance_table,
                mapping_root: record.new_mapping_root,
                allocation_record_tree_root: record.new_allocation_record_tree_root,
            }
        }
    };
    Ok((report, rebuilt))
}

/// 一条记录的新根段拼出的那次发布的根：每条记录都带整次发布的新根段（D23（journal 的角色与格式） 已定项 15），七个字段取记录里的，
/// fsid 新根段里没有、取调用方给的。可写挂载读水位之上没施加的那次发布的账用它（`mount::placements_in_the_accounts_of_unapplied_publishes`）。
/// [`replay_journal`] 施加末条之后拼根是同一个拼法、在那里就地写着：那一段被变异表多行逐字锚着，这一批不挪它。
pub(crate) fn root_assembled_from_the_new_root_segment_of_the_record(
    record: &JournalRecord,
    filesystem_identifier: [u8; 16],
) -> RootRecord {
    RootRecord {
        filesystem_identifier,
        unmount_marker: record.new_unmount_marker,
        instance: record.instance,
        checkpoint_txg: record.checkpoint_txg,
        tree_table: record.new_tree_table,
        tree_identifier_watermark: record.new_tree_identifier_watermark,
        rollback_floor: record.new_rollback_floor,
        instance_table: record.new_instance_table,
        mapping_root: record.new_mapping_root,
        allocation_record_tree_root: record.new_allocation_record_tree_root,
    }
}

/// 每棵树的 key 宽（码 2 头里的自述 key 宽要与它相符）；day-1 只注册的三棵没有节点要解。
fn key_width_for_kind(kind: u16) -> Option<usize> {
    match kind {
        TREE_KIND_EXTENT => Some(24),
        TREE_KIND_INODE => Some(INODE_KEY_WIDTH_IN_BYTES),
        TREE_KIND_ALLOCATION => Some(10),
        TREE_KIND_ACCOUNTING => Some(22),
        // `deleted_inodes` 的 key 是 inode 号 8 字节（D8（核心索引结构） 已定项 16 第 1 条）；墓碑树与谱系树的根条目照 inode 树内部节点的
        // 120 字节形态，key 是分隔 key = 容器号 8 字节（D18（块里携带什么信息） 已定项 24 第 1、5 条）。
        TREE_KIND_DELETED_INODES => Some(DELETED_INODES_KEY_WIDTH_IN_BYTES),
        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
            Some(CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES)
        }
        // logged_ops 的字段表留到第一个写者落地时定（D8（核心索引结构） 已定项 5、C364（整理意图记录没有字段表））。
        TREE_KIND_LIVELIST
        | TREE_KIND_SPARSE_SIDE_TABLE
        | TREE_KIND_DEADLIST
        | TREE_KIND_LOGGED_OPERATIONS => None,
        _ => None,
    }
}

/// 读一棵进映射的树的根节点（extent、inode、分配记录；记账树多层，走 `code_two_tree::read_code_two_tree`）：位置提示读不出时经中央映射回退
/// （[`read_mapped_tree_node_via_hint_then_central_mapping`]），读到之后核它的自描述：树 ID、key 宽、出生身份、fsid、
/// key 区间与条目相符。挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
#[allow(
    clippy::too_many_arguments,
    reason = "核根要的六样加回退要的两样（查映射的口子、多跳计数），各自独立，收成结构体只会多一层没人验的名字"
)]
pub(crate) fn read_mapped_tree_root(
    reader: &dyn PoolReader,
    tree: TreeIdentifier,
    key_width: usize,
    pointer: &NodePointer,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
    central_mapping_locations_of_key: &CentralMappingLocationsOfKey<'_>,
    stale_location_hint_hops: &mut usize,
) -> Result<IndexNodeHeader, RecoveryFailure> {
    let bytes = read_mapped_tree_node_via_hint_then_central_mapping(
        reader,
        pointer,
        MappedTreeNodeClass::IndexNode,
        central_mapping_locations_of_key,
        stale_location_hint_hops,
    )?;
    tree_root_checked_against_its_pointer(
        &bytes,
        tree,
        key_width,
        pointer,
        root,
        expected_filesystem_identifier,
    )
}

/// 一棵容器树（墓碑树、谱系树）里的一个单元：一片容器（叶序）、根之下的一个码 2 节点（按位置）、根。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitOfAContainerTree {
    Container(usize),
    NodeBelowTheRoot(ContainerTreeNodePosition),
    Root,
}

impl UnitOfAContainerTree {
    /// 映射 key 里的单元类：容器是码 3 打包记录单元，节点是码 2。
    fn unit_class(self) -> u8 {
        match self {
            UnitOfAContainerTree::Container(_) => UNIT_CLASS_PACKED,
            UnitOfAContainerTree::NodeBelowTheRoot(_) | UnitOfAContainerTree::Root => {
                UNIT_CLASS_INDEX_NODE
            }
        }
    }
}

/// 一棵容器树的三种角色（与发布路径同一组：`transaction::TransactionUnit` 里那棵树的叶容器、根、根之下的节点）。
struct ContainerTreeRolesOfOneTree {
    container: fn(usize) -> TransactionUnit,
    root: TransactionUnit,
    node_below_the_root: fn(ContainerTreeNodePosition) -> TransactionUnit,
}

impl ContainerTreeRolesOfOneTree {
    fn of(&self, unit: UnitOfAContainerTree) -> TransactionUnit {
        match unit {
            UnitOfAContainerTree::Container(position) => (self.container)(position),
            UnitOfAContainerTree::NodeBelowTheRoot(position) => {
                (self.node_below_the_root)(position)
            }
            UnitOfAContainerTree::Root => self.root,
        }
    }
}

fn tombstone_container_role_of_position(position: usize) -> TransactionUnit {
    TransactionUnit::TombstoneLeafContainer(TombstoneContainerIndexInTree::of_position(position))
}

fn lineage_container_role_of_position(position: usize) -> TransactionUnit {
    TransactionUnit::LineageLeafContainer(LineageContainerIndexInTree::of_position(position))
}

const CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE: ContainerTreeRolesOfOneTree =
    ContainerTreeRolesOfOneTree {
        container: tombstone_container_role_of_position,
        root: TransactionUnit::TombstoneRoot,
        node_below_the_root: TransactionUnit::TombstoneNodeBelowTheRoot,
    };

const CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE: ContainerTreeRolesOfOneTree =
    ContainerTreeRolesOfOneTree {
        container: lineage_container_role_of_position,
        root: TransactionUnit::LineageRoot,
        node_below_the_root: TransactionUnit::LineageNodeBelowTheRoot,
    };

fn retirement_container_role_of_position(position: usize) -> TransactionUnit {
    TransactionUnit::RetirementLeafContainer(RetirementContainerIndexInTree::of_position(position))
}

const CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE: ContainerTreeRolesOfOneTree =
    ContainerTreeRolesOfOneTree {
        container: retirement_container_role_of_position,
        root: TransactionUnit::RetirementRoot,
        node_below_the_root: TransactionUnit::RetirementNodeBelowTheRoot,
    };

/// 墓碑树、谱系树码 2 节点条目里的子指针头部 extent 偏移非 0 时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句
/// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构损坏，与 inode 树内部条目同一条）。
pub const CONTAINER_TREE_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
    "墓碑树或谱系树内部条目的子指针头部 extent 偏移不是 0";
/// 墓碑树、谱系树码 2 节点条目里的子指针头部加密、压缩那几段（MAC / nonce、算法类型、压缩算法码与压后长度）不是第一版的取值时报
/// [`RecoveryFailure::UnitMalformed`]，`what` 取这一句（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14，与 inode 树内部条目同一条）。
pub const CONTAINER_TREE_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
    &str = "墓碑树或谱系树内部条目的子指针头部的加密、压缩那几段不是第一版的取值";

/// 一条带指针的条目里那条指针头部第一版之外的取值报哪一句：按落在哪一组字段上，在读者给的两句里挑。
fn what_of_a_pointer_head_field_outside_the_first_version(
    field: PointerHeadFieldOutsideTheFirstVersion,
    what_of_the_encryption_or_compression_fields: &'static str,
    what_of_the_extent_offset: &'static str,
) -> &'static str {
    match field.field_group() {
        PointerHeadFieldGroup::EncryptionOrCompression => {
            what_of_the_encryption_or_compression_fields
        }
        PointerHeadFieldGroup::ExtentOffset => what_of_the_extent_offset,
    }
}

/// 读一个进映射的树节点（码 2 或码 3，提示读不出经映射回退）：重建那一路收进来的读盘办法。
type ReadOneMappedTreeNode<'reading> =
    dyn FnMut(&NodePointer, MappedTreeNodeClass) -> Result<Vec<u8>, RecoveryFailure> + 'reading;

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

impl<Record> ContainerTreeRead<Record> {
    /// 进映射的单元数：每片容器、每个码 2 节点（根在内）各一把 key。
    fn mapped_units(&self) -> usize {
        self.containers.len() + self.index_nodes.nodes.len()
    }

    /// 全部单元按 bump 次序（容器按叶序，再是码 2 节点、根最末），与发布路径装 `units` 与 `mapped_units` 的次序相同。
    fn units_in_bump_order(&self) -> Vec<(UnitOfAContainerTree, NodePointer, Vec<u8>)> {
        let containers =
            self.containers
                .iter()
                .enumerate()
                .map(|(position, (_, pointer, bytes))| {
                    (
                        UnitOfAContainerTree::Container(position),
                        *pointer,
                        bytes.clone(),
                    )
                });
        let nodes = self
            .index_nodes
            .nodes
            .iter()
            .map(|(position, pointer, bytes)| {
                let unit = if self.index_nodes.shape.is_the_root(*position) {
                    UnitOfAContainerTree::Root
                } else {
                    UnitOfAContainerTree::NodeBelowTheRoot(*position)
                };
                (unit, *pointer, bytes.clone())
            });
        containers.chain(nodes).collect()
    }

    /// 这一版的形状与根之下每个节点的指针（根的指针住树表条目）。
    fn index_nodes_version(&self) -> ContainerTreeIndexNodesVersion {
        ContainerTreeIndexNodesVersion {
            shape: self.index_nodes.shape.clone(),
            pointers_below_the_root: self
                .index_nodes
                .nodes
                .iter()
                .filter(|(position, _, _)| !self.index_nodes.shape.is_the_root(*position))
                .map(|(position, pointer, _)| (*position, *pointer))
                .collect(),
        }
    }
}

/// 从根往下读一棵容器树的码 2 节点（不读容器）：每个节点照树根同一组核它的自描述（`tree_root_checked_against_its_pointer`：树 ID、
/// key 宽、出生身份、fsid、key 区间与条目相符）；根的层级至少 1、每个节点非空；层级 ℓ > 1 的节点的条目指码 2 孩子——身份引用全 0
/// （类型段 0，D8（核心索引结构） 已定项 6）、孩子的层级 = ℓ − 1、孩子的首 key = 条目的分隔 key；层级 1 的条目指容器，原样交回由读容器那一步判。
/// 递归的上界：根的层级（一字节），每往下一层层级减一。`read_index_node` 读一个码 2 节点（提示读不出经映射回退）。
///
/// # Errors
/// 读不出（经映射回退之后仍读不出）、自描述不符、条目窄于字段表、形状不合 ⇒ 对应的 `RecoveryFailure`。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：树号、种类码、两种报错名、根指针、根记录、fsid、读节点的办法"
)]
fn read_container_tree_index_nodes(
    tree: TreeIdentifier,
    kind: u16,
    what_index_node: &'static str,
    what_entry: &'static str,
    root_pointer: &NodePointer,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
    read_index_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
) -> Result<ContainerTreeIndexNodesRead, RecoveryFailure> {
    let key_width = key_width_for_kind(kind).expect("墓碑树与谱系树的 key 宽登记过");
    let malformed = || RecoveryFailure::UnitMalformed {
        what: what_index_node,
    };
    let root_bytes = read_index_node(root_pointer)?;
    let root_header = tree_root_checked_against_its_pointer(
        &root_bytes,
        tree,
        key_width,
        root_pointer,
        root,
        expected_filesystem_identifier,
    )?;
    if root_header.level == 0 || root_header.entries.is_empty() {
        return Err(malformed());
    }
    let mut level = root_header.level;
    let mut nodes_of_this_level: Vec<(NodePointer, Vec<u8>, IndexNodeHeader)> =
        vec![(*root_pointer, root_bytes, root_header)];
    // 自根往下每一层：(指针, 字节, 孩子数)。
    let mut levels_from_the_root_down: Vec<Vec<(NodePointer, Vec<u8>, usize)>> = Vec::new();
    let mut container_entries = Vec::new();
    loop {
        let mut nodes_of_the_level_below = Vec::new();
        let mut this_level = Vec::new();
        for (pointer, bytes, header) in nodes_of_this_level {
            for entry in &header.entries {
                let (separator_key, identity_in_the_entry, child_pointer) =
                    parse_inode_internal_entry_or_refusal(entry).map_err(
                        |refusal| match refusal {
                            PointerCarryingEntryRefusal::NarrowerThanItsFieldTable => {
                                RecoveryFailure::EntryNarrowerThanItsFieldTable {
                                    what: what_entry,
                                    entry_bytes: entry.len(),
                                    field_table_bytes: usize::try_from(INODE_INTERNAL_ENTRY)
                                        .expect("120"),
                                }
                            }
                            PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(
                                field,
                            ) => RecoveryFailure::UnitMalformed {
                                what: what_of_a_pointer_head_field_outside_the_first_version(
                                    field,
                                    CONTAINER_TREE_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
                                    CONTAINER_TREE_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
                                ),
                            },
                        },
                    )?;
                if level == 1 {
                    container_entries.push((separator_key, identity_in_the_entry, child_pointer));
                    continue;
                }
                if identity_in_the_entry != IDENTITY_OF_A_CODE_TWO_CHILD {
                    return Err(malformed());
                }
                let child_bytes = read_index_node(&child_pointer)?;
                let child_header = tree_root_checked_against_its_pointer(
                    &child_bytes,
                    tree,
                    key_width,
                    &child_pointer,
                    root,
                    expected_filesystem_identifier,
                )?;
                if child_header.level.checked_add(1) != Some(level)
                    || child_header.entries.is_empty()
                    || child_header.smallest_key[..] != separator_key.to_le_bytes()[..]
                {
                    return Err(malformed());
                }
                nodes_of_the_level_below.push((child_pointer, child_bytes, child_header));
            }
            this_level.push((pointer, bytes, header.entries.len()));
        }
        levels_from_the_root_down.push(this_level);
        if level == 1 {
            break;
        }
        level -= 1;
        nodes_of_this_level = nodes_of_the_level_below;
    }
    levels_from_the_root_down.reverse();
    let shape = ContainerTreeShape::from_children_of_each_node_at_each_level(
        levels_from_the_root_down
            .iter()
            .map(|nodes| nodes.iter().map(|(_, _, children)| *children).collect())
            .collect(),
    )
    .ok_or_else(malformed)?;
    let nodes = levels_from_the_root_down
        .into_iter()
        .enumerate()
        .flat_map(|(level_index, nodes)| {
            nodes
                .into_iter()
                .enumerate()
                .map(move |(index_in_level, (pointer, bytes, _))| {
                    (
                        ContainerTreeNodePosition {
                            level: u8::try_from(level_index + 1)
                                .expect("层数不超过根头里的一字节层级"),
                            index_in_level: u32::try_from(index_in_level)
                                .expect("同层节点数装得进 u32"),
                        },
                        pointer,
                        bytes,
                    )
                })
        })
        .collect();
    Ok(ContainerTreeIndexNodesRead {
        shape,
        nodes,
        container_entries,
    })
}

/// 读一棵容器树：零根（树表条目的根指针置零）交回 `None`；否则读码 2 节点（[`read_container_tree_index_nodes`]），再按叶序读每片容器，
/// 核容器头的身份与父条目里的身份引用逐字相等、类型 = 这棵树的打包记录类型、出生树 = 这棵树、分隔 key = 容器号、记录宽、非空，逐条解记录。
/// `read_node` 读一个单元（码 2 或码 3，提示读不出经映射回退）。
///
/// # Errors
/// 同 [`read_container_tree_index_nodes`]；容器读不出、解不开、与条目对不上、记录解不开 ⇒ 对应的 `RecoveryFailure`。
fn read_container_tree<Record>(
    to_read: &ContainerTreeToRead<Record>,
    root_pointer: &NodePointer,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
    read_node: &mut ReadOneMappedTreeNode<'_>,
) -> Result<Option<ContainerTreeRead<Record>>, RecoveryFailure> {
    if *root_pointer == NodePointer::empty_root() {
        return Ok(None);
    }
    let index_nodes = read_container_tree_index_nodes(
        to_read.tree,
        to_read.kind,
        to_read.what_index_node,
        to_read.what_entry,
        root_pointer,
        root,
        expected_filesystem_identifier,
        &mut |pointer: &NodePointer| read_node(pointer, MappedTreeNodeClass::IndexNode),
    )?;
    let mut containers = Vec::with_capacity(index_nodes.container_entries.len());
    for (separator_key, identity_in_the_entry, container_pointer) in &index_nodes.container_entries
    {
        let container_bytes = read_node(container_pointer, MappedTreeNodeClass::PackedRecordUnit)?;
        let container = parse_packed_unit(&container_bytes).map_err(|_error| {
            RecoveryFailure::UnitMalformed {
                what: to_read.what_container,
            }
        })?;
        if container.identity != *identity_in_the_entry
            || container.identity.record_type != to_read.record_type
            || container.identity.birth_tree != to_read.tree
            || container.identity.container != *separator_key
            || container.record_width
                != usize::try_from(to_read.record_width_in_bytes).expect("记录宽是 40 或 16")
            || container.records.is_empty()
        {
            return Err(RecoveryFailure::UnitMalformed {
                what: to_read.what_container_against_its_entry,
            });
        }
        let records = container
            .records
            .iter()
            .map(|record_bytes| {
                (to_read.parse_record)(record_bytes).ok_or(RecoveryFailure::UnitMalformed {
                    what: to_read.what_record,
                })
            })
            .collect::<Result<Vec<Record>, RecoveryFailure>>()?;
        containers.push((
            RecordContainer {
                identity: container.identity,
                records,
            },
            *container_pointer,
            container_bytes,
        ));
    }
    Ok(Some(ContainerTreeRead {
        index_nodes,
        containers,
    }))
}

/// 一个树根节点读回来之后核的那几样：树 ID、key 宽、出生身份、fsid、key 区间与条目相符。
/// 经映射回退读到的节点照样核这几样——搬迁只改映射一条条目，被搬的单元逐字节不变，指针里的出生身份照样对得上。
fn tree_root_checked_against_its_pointer(
    bytes: &[u8],
    tree: TreeIdentifier,
    key_width: usize,
    pointer: &NodePointer,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
) -> Result<IndexNodeHeader, RecoveryFailure> {
    let node = parse_index_node(bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
        what: "树根节点",
    })?;
    if node.tree != tree {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.3",
            detail: "根头里的树 ID 与树表不符",
        });
    }
    if node.key_width != key_width {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "根自述 key 宽与树的种类不符",
        });
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "树根诞生于根之后",
        });
    }
    if node.filesystem_identifier != expected_filesystem_identifier {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.4",
            detail: "树根 fsid 不符",
        });
    }
    if node.birth_sequence != pointer.birth_sequence {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "树根出生序号与指针不符",
        });
    }
    if let (Some(first), Some(last)) = (node.entries.first(), node.entries.last()) {
        if first[..key_width] != node.smallest_key[..] || last[..key_width] != node.largest_key[..]
        {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-1.1",
                detail: "根 key 区间与条目不符",
            });
        }
    }
    Ok(node)
}

/// 从盘上读回来的 extent 树里第一个文件的样子：它的数据指针（第 i 个是单元 i）、这一版 extent 树的节点与指针、
/// 每个节点的角色、指针与字节（bump 次序：下段先叶后根，再上段先叶后根）。
pub(crate) struct ExtentsOfTheFirstFile {
    pub data_pointers: Vec<DataPointer>,
    pub version: ExtentTreeVersion,
    pub nodes_in_bump_order: Vec<(TransactionUnit, NodePointer, Vec<u8>)>,
}

/// 从整棵读回来的 extent 树里取第一个文件（第一版只有第一个文件有内容，`transaction` 只按它建这一版在内存里的样子）：
/// 上段里要恰好只有它那一条叶条目，它的单元从 0 起连号、没有洞。
///
/// # Errors
/// 上段里没有第一个文件的条目、或还有别的 inode 的条目、第一个文件一个单元都没有、单元有洞 ⇒ `UnitMalformed`：
/// 这几样今天的写路径都写不出来（只有它写 extent 树、每个文件版本从偏移 0 顺序写、长度 0 的内容也写一个单元），
/// 盘上读来的却可以是任何样子——第一版不支持，在任何落盘动作之前交回。
pub(crate) fn extents_of_the_first_file(
    tree: ExtentTreeReadFromDisk,
) -> Result<ExtentsOfTheFirstFile, RecoveryFailure> {
    let ExtentTreeReadFromDisk { upper, files } = tree;
    // 上段一条叶条目都没有：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），没有数据单元、没有下段。
    if files.is_empty() {
        let upper_root_level = upper
            .nodes
            .last()
            .map(|(position, _, _)| position.level)
            .expect("读回来的上段至少有根");
        return Ok(ExtentsOfTheFirstFile {
            data_pointers: Vec::new(),
            version: ExtentTreeVersion {
                upper_nodes: upper
                    .nodes
                    .iter()
                    .map(|(position, pointer, _)| (*position, *pointer))
                    .collect(),
                lower_nodes: Vec::new(),
            },
            nodes_in_bump_order: upper
                .nodes
                .into_iter()
                .map(|(position, pointer, bytes)| {
                    (
                        role_of_extent_upper_node(position, upper_root_level),
                        pointer,
                        bytes,
                    )
                })
                .collect(),
        });
    }
    let [(inode, extents)] =
        <[(u64, ExtentsOfAFileReadFromDisk); 1]>::try_from(files).map_err(|_files| {
            RecoveryFailure::UnitMalformed {
                what: "extent 树上段里不是恰好一条叶条目（第一版只有第一个文件有内容）",
            }
        })?;
    if inode != FIRST_INODE_NUMBER {
        return Err(RecoveryFailure::UnitMalformed {
            what: "extent 树上段里那一条叶条目不是第一个文件的",
        });
    }
    let data_pointers_with_units = extents.data_pointers();
    if data_pointers_with_units.is_empty() {
        return Err(RecoveryFailure::UnitMalformed {
            what: "extent 树里第一个文件一个数据单元都没有",
        });
    }
    if data_pointers_with_units
        .iter()
        .enumerate()
        .any(|(position, (unit, _))| u64::try_from(position).expect("单元序号") != *unit)
    {
        return Err(RecoveryFailure::UnitMalformed {
            what: "extent 树里第一个文件的单元有洞（第一版不支持）",
        });
    }
    let lower_nodes = match extents {
        ExtentsOfAFileReadFromDisk::LowerSegment(segment) => segment.nodes,
        ExtentsOfAFileReadFromDisk::NoDataUnit | ExtentsOfAFileReadFromDisk::Inline(_) => {
            Vec::new()
        }
    };
    let upper_root_level = upper
        .nodes
        .last()
        .map(|(position, _, _)| position.level)
        .expect("读回来的上段至少有根");
    let version = ExtentTreeVersion {
        upper_nodes: upper
            .nodes
            .iter()
            .map(|(position, pointer, _)| (*position, *pointer))
            .collect(),
        lower_nodes: lower_nodes
            .iter()
            .map(|(position, pointer, _)| (*position, *pointer))
            .collect(),
    };
    let nodes_in_bump_order = lower_nodes
        .into_iter()
        .map(|(position, pointer, bytes)| {
            (TransactionUnit::ExtentLowerNode(position), pointer, bytes)
        })
        .chain(upper.nodes.into_iter().map(|(position, pointer, bytes)| {
            (
                role_of_extent_upper_node(position, upper_root_level),
                pointer,
                bytes,
            )
        }))
        .collect();
    Ok(ExtentsOfTheFirstFile {
        data_pointers: data_pointers_with_units
            .into_iter()
            .map(|(_, pointer)| pointer)
            .collect(),
        version,
        nodes_in_bump_order,
    })
}

/// 分配记录「每个落点每盘各一条」（两盘同槽、同一批字段）：同一块盘上一个槽只许一条记录，每块盘各自的（槽, 跨度, 代, 已释放）集合相同，
/// 每盘不少于 `placements_on_each_device_at_least` 个落点（走读传的是这套几何下新池新建文件之后每盘的落点数，
/// [`placements_on_each_device_after_the_new_pool_file_creation`]）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
/// 走读自己不判 key 严格递增，这里逐盘核槽号不重复。
///
/// # Panics
/// `device_identities` 为空：调用方要交一张非空的盘表。产品调用方交的是读者的盘表（`PoolReader::device_identities`），
/// 零块盘的池走到这里之前在读实例表单元那一步已经报错返回；空盘表在这里没有「每盘一条」可判，不交回 `true` 也不交回 `false`。
#[must_use]
pub fn allocation_records_are_one_per_device(
    records: &[AllocationRecord],
    device_identities: &[DeviceIdentity],
    placements_on_each_device_at_least: PlacementsOnEachDevice,
) -> bool {
    assert!(
        !device_identities.is_empty(),
        "盘表不许为空：产品调用方交读者的盘表，零块盘的池在读实例表单元那一步已经报错返回"
    );
    let mut placements_per_device: BTreeMap<DeviceIdentity, BTreeSet<(u64, u16, u64, bool)>> =
        BTreeMap::new();
    let mut slots_per_device: BTreeMap<DeviceIdentity, BTreeSet<u64>> = BTreeMap::new();
    for record in records {
        if !slots_per_device
            .entry(record.device)
            .or_default()
            .insert(record.slot.0)
        {
            return false;
        }
        placements_per_device
            .entry(record.device)
            .or_default()
            .insert((
                record.slot.0,
                record.span_slots,
                record.generation.0,
                record.is_released,
            ));
    }
    if placements_per_device.len() != device_identities.len()
        || device_identities
            .iter()
            .any(|identity| !placements_per_device.contains_key(identity))
    {
        return false;
    }
    let mut placement_sets = placements_per_device.values();
    let first_device_placements = placement_sets
        .next()
        .expect("开头断言过盘表非空，上面又核过盘表里每块盘都有记录：至少有一块盘的落点集合");
    first_device_placements.len() >= placements_on_each_device_at_least.0
        && placement_sets.all(|placements| placements == first_device_placements)
}

/// 每块盘上的落点数（分配记录条数按盘数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementsOnEachDevice(pub usize);

/// 新池新建文件之后每块盘上的落点数，按这个池的几何现算（C692 取 G2「按几何现算」，用户 2026-10-07 定）：分配记录每盘落点数的下限
/// （[`allocation_records_are_one_per_device`]），之后每次发布只多不少。几何是每块盘的字节数（`device_sizes_in_bytes`，分配记录树的根层级
/// 由它定，[`AllocationRecordTreeGeometry::of_device_sizes_in_bytes`]）与单元区起点（系统配置里的环长现算的，`unit_area_start`）。
/// 算法是在一个空分配器上照新池新建文件那一版分一遍落点（D3（空间分配） 已定项 8 / 已定项 10 的分配器本身，不另写一份落点规则）：
/// mkfs 写在单元区里的两个单元（实例表第 0 片在单元区起点、两槽，第 0 版树表紧跟其后、一槽，与 `make_filesystem` 同一个摆法），
/// 一个数据单元（第一个文件至少一个数据单元，`write_request_split::data_unit_count_of_a_sequential_write`），再照提交内生块的 bump 次序
/// （D3（空间分配） 已定项 10 ⑤：按树 ID 升序、树内先叶后根、中央映射树倒数第二、树表单元最末）分 extent 树根、inode 叶容器、inode 树根、
/// 分配记录树的节点、记账树根、中央映射树根、树表单元。分配记录树要几个节点取决于这些记录落在几片叶里（按位置寻址，D8（核心索引结构） 已定项 14），
/// 而节点自己的落点也有记录：节点数从「每块盘一条根到叶的路径加根」起，按分完之后的记录现算的节点数调，到两者相等为止。
/// 默认几何（4 GiB 盘、默认环）每盘 14（字节表五：28 条分配记录）；1024 MiB 环那套几何（单元区起点 66560）提交内生块多两个、每盘 16。
///
/// 盘的字节数是块设备报的、或崩溃镜像自述的，不是不变量（正推「发现 6」，判决 `research/prompts/m3-batch33t-code-r1-main-verification.md`
/// 第三节第 1 条「排演函数小盘不 panic」）：建排演用的空闲图之前逐盘判够不够长，不够就交错误。
///
/// # Errors
/// 按 `device_sizes_in_bytes` 次序第一块完整槽数不到单元区起点的盘 ⇒ [`NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart`]；
/// 空分配器上分不出这几个落点（单元区装不下新池新建文件那十几个单元，这套几何上写不出带文件的一版）⇒
/// [`NewPoolFileCreationRehearsalRefused::UnitAreaCannotHoldTheNewPoolFileCreation`]。
///
/// # Panics
/// 节点数调了 [`ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS`] 轮还不相等。
pub fn placements_on_each_device_after_the_new_pool_file_creation(
    device_sizes_in_bytes: &[(DeviceIdentity, u64)],
    unit_area_start: UnitAreaStart,
) -> Result<PlacementsOnEachDevice, NewPoolFileCreationRehearsalRefused> {
    if let Some((device, device_bytes)) = device_sizes_in_bytes.iter().find(|(_, device_bytes)| {
        unit_area_slots_of_device_starting_at(*device_bytes, unit_area_start).is_err()
    }) {
        return Err(
            NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart {
                device: *device,
                device_bytes: *device_bytes,
            },
        );
    }
    let geometry = AllocationRecordTreeGeometry::of_device_sizes_in_bytes(device_sizes_in_bytes);
    let new_pool_file_creation_txg = CheckpointTxg(1);
    // 新池新建文件的落点都在单元区开头的两段里（mkfs 与数据单元在第一段，提交内生块开在最低的全空段、即第二段）：空闲图只建到那两段为止，
    // 走读每次都调它，不为整块盘建一张图（4 GiB 盘是二十多万槽）。分配记录树的几何仍按整块盘的字节数（上面那一行）。
    let rehearsal_bytes_on_each_device =
        (unit_area_start.slot().0 + 2 * CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES;
    let mut allocator = crate::allocator::PoolAllocator::new(
        device_sizes_in_bytes
            .iter()
            .map(|(device, device_bytes)| {
                crate::allocator::DeviceFreeMap::with_unit_area_start(
                    *device,
                    (*device_bytes).min(rehearsal_bytes_on_each_device),
                    unit_area_start,
                )
            })
            .collect(),
    );
    let instance_table_span = TransactionUnit::InstanceTable.span_slots();
    allocator.mark_format_time_units(
        crate::allocator::Placement {
            slot: unit_area_start.slot(),
            span: instance_table_span,
        },
        crate::allocator::Placement {
            slot: SlotNumber(unit_area_start.slot().0 + instance_table_span),
            span: TransactionUnit::TreeTable.span_slots(),
        },
    );
    let allocate = |rehearsing: &mut crate::allocator::PoolAllocator, unit: TransactionUnit| {
        let placement = match unit.placement() {
            crate::transaction::PlacementRule::UserData => {
                rehearsing.allocate_user_data(new_pool_file_creation_txg)
            }
            crate::transaction::PlacementRule::CommitGenerated(footprint) => {
                rehearsing.allocate_commit_generated(footprint, new_pool_file_creation_txg)
            }
        };
        placement
            .map(|_placement| ())
            .ok_or(NewPoolFileCreationRehearsalRefused::UnitAreaCannotHoldTheNewPoolFileCreation)
    };
    for unit in [
        TransactionUnit::Data(DataUnitIndexInFile(0)),
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
        TransactionUnit::InodeRoot,
    ] {
        allocate(&mut allocator, unit)?;
    }
    let units_before_the_allocation_record_tree = allocator.clone();
    let device_count = device_sizes_in_bytes.len();
    // 迭代上界 ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS；跨轮只带猜的节点数，每一轮从同一个分配器拷贝重分。
    let mut allocation_record_tree_nodes = device_count * usize::from(geometry.root_level()) + 1;
    for _round in 0..ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS {
        let mut rehearsal = units_before_the_allocation_record_tree.clone();
        for _node in 0..allocation_record_tree_nodes {
            allocate(&mut rehearsal, TransactionUnit::AllocationTree)?;
        }
        for unit in [
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ] {
            allocate(&mut rehearsal, unit)?;
        }
        let nodes_holding_the_records =
            crate::allocation_record_tree::nodes_holding_records(&geometry, rehearsal.records())
                .len();
        if nodes_holding_the_records == allocation_record_tree_nodes {
            return Ok(PlacementsOnEachDevice(
                rehearsal.records().len() / device_count,
            ));
        }
        allocation_record_tree_nodes = nodes_holding_the_records;
    }
    panic!(
        "分配记录树的节点数调了 {ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS} 轮还不相等：每一轮多分的节点只会让记录多落进一两片叶，几轮就该停"
    );
}

/// [`placements_on_each_device_after_the_new_pool_file_creation`] 排演不出来的两种情形（调用方据此报恢复失败，不 panic）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewPoolFileCreationRehearsalRefused {
    /// 这块盘的完整槽数不到单元区起点（盘的字节数是块设备报的、或崩溃镜像自述的）。
    DeviceEndsBeforeTheUnitAreaStart {
        device: DeviceIdentity,
        device_bytes: u64,
    },
    /// 空分配器上分不出新池新建文件那十几个单元：这套几何上写不出带文件的一版。
    UnitAreaCannotHoldTheNewPoolFileCreation,
}

/// [`placements_on_each_device_after_the_new_pool_file_creation`] 调分配记录树节点数的轮数上界：猜的节点数每一轮只会因为节点自己的落点
/// 多落进一片叶而加几个，推的，没量过会不会到第三轮。
const ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS: usize = 8;

/// 冷走读里的中央映射树（自举豁免，只按父指针里的位置条目读）：到第一次有树根的提示读不出、要经映射回退时，
/// 或走到 `TreeRoots` 那一步时才读，读过一次就留着。不提前读：提示都读得出的镜像上，走读的读序与判红次序照旧。
/// 映射树多层时整棵读回来（D19（块指针的结构与宽度预算） 已定项 5），每个节点按父条目核（`code_two_tree::read_code_two_tree`）。
struct CentralMappingTreeReadOnFirstUse<'walk> {
    reader: &'walk dyn PoolReader,
    root: &'walk RootRecord,
    expected_filesystem_identifier: u64,
    tree: OnceCell<CodeTwoTreeReadFromDisk>,
}

impl CentralMappingTreeReadOnFirstUse<'_> {
    fn tree(&self) -> Result<&CodeTwoTreeReadFromDisk, RecoveryFailure> {
        if let Some(tree) = self.tree.get() {
            return Ok(tree);
        }
        let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
        let tree = read_code_two_tree(
            &self.root.mapping_root,
            &MultiLevelCodeTwoTree::CentralMapping
                .read_expectation(self.root.mapping_root.head.birth_tree),
            CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
            self.root,
            self.expected_filesystem_identifier,
            &mut |pointer: &NodePointer| {
                read_unit_via_locations(self.reader, &pointer.locations, node_bytes)
            },
        )?;
        Ok(self.tree.get_or_init(|| tree))
    }

    fn locations_of_key(
        &self,
        mapping_key: &[u8],
    ) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
        central_mapping_locations_among_entries(
            &self.tree()?.leaf_entries_in_key_order,
            mapping_key,
        )
    }

    fn into_tree(self) -> Result<CodeTwoTreeReadFromDisk, RecoveryFailure> {
        self.tree()?;
        Ok(self
            .tree
            .into_inner()
            .expect("上一行刚把映射树读进来，读不出已经返回了"))
    }
}

// ─── 冷走读与从盘上重建上一版共用的判定 ───
// 代码审阅第 24 / 33 条，用户定案（`records/2026-09-27-代码审阅38条去向.md`「可写挂载判全树头与 key 次序」那一行）：「重建时把已经读到的节点照冷走读那套判全，
// 坏了报错拒挂载，不多读盘」。下面每一道只判两边都已经读到手的字节，[`walk_to_file`] 与 [`rebuild_version`] 各在读到那一样的地方调，
// 报的成员两边相同；一道判定一处定义，不各写一份（`code-discipline.md`「重复要生成，不许手抄」）。

/// 树表单元自述的 key 宽是树表的（[`TREE_TABLE_KEY_WIDTH`]）。
fn tree_table_key_width_is_the_tree_tables(
    tree_table: &IndexNodeHeader,
) -> Result<(), RecoveryFailure> {
    if tree_table.key_width != TREE_TABLE_KEY_WIDTH {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "树表单元自述 key 宽不是 8",
        });
    }
    Ok(())
}

/// 树表条目里的树 ID 低于根记录的树 ID 水位（I-7.8（根记录树 ID 水位不低于全池最大树 ID） 在读路径上的那一半）。
fn tree_identifier_of_the_entry_is_below_the_watermark(
    entry: &TreeTableEntry,
    root: &RootRecord,
) -> Result<(), RecoveryFailure> {
    if entry.tree.0 >= root.tree_identifier_watermark {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-7.8",
            detail: "树 ID 不低于水位",
        });
    }
    Ok(())
}

/// 每个落点每盘一条（两盘同槽）：新池新建文件之后每盘的落点数按这个池的几何现算（默认几何 14 × 盘数，
/// [`placements_on_each_device_after_the_new_pool_file_creation`]），之后每次发布只多不少（换下的那些改写、不删）。
/// 只核总数是盘数的整数倍拦不住「一盘多一条、另一盘少一条」——发布 B 三方第一轮正推腿打中，改成逐盘核同一批（槽, 跨度）
/// （[`allocation_records_are_one_per_device`]）。
///
/// # Panics
/// 读者列出的盘说不出自己的字节数：每个读者都从同一张表里取（`PoolReader::device_size_in_bytes`）。
fn allocation_records_are_one_placement_per_device_on_every_device(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    records: &[AllocationRecord],
    device_identities: &[DeviceIdentity],
) -> Result<(), RecoveryFailure> {
    let device_sizes_in_bytes: Vec<(DeviceIdentity, u64)> = device_identities
        .iter()
        .map(|device| {
            (
                *device,
                reader
                    .device_size_in_bytes(*device)
                    .expect("读者列出的盘说得出自己的字节数"),
            )
        })
        .collect();
    let not_one_placement_per_device = RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数",
    };
    // 先判记录本身（下限取 0），再排演下限：排演要按盘的字节数建空闲图，记录判空、各盘不同这些不用它就判得出（正推「发现 6」）。
    if !allocation_records_are_one_per_device(records, device_identities, PlacementsOnEachDevice(0))
    {
        return Err(not_one_placement_per_device);
    }
    let placements_on_each_device_at_least =
        match placements_on_each_device_after_the_new_pool_file_creation(
            &device_sizes_in_bytes,
            unit_area_start,
        ) {
            Ok(placements_on_each_device_at_least) => placements_on_each_device_at_least,
            Err(NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart {
                device,
                device_bytes,
            }) => {
                return Err(RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                    device,
                    device_bytes,
                })
            }
            // 这套几何上写不出带文件的一版，分配记录就不够这一版的下限。
            Err(NewPoolFileCreationRehearsalRefused::UnitAreaCannotHoldTheNewPoolFileCreation) => {
                return Err(not_one_placement_per_device)
            }
        };
    if !allocation_records_are_one_per_device(
        records,
        device_identities,
        placements_on_each_device_at_least,
    ) {
        return Err(not_one_placement_per_device);
    }
    Ok(())
}

/// 每条记账条目都有字段表那么宽（条目宽是盘上的值，窄于 34 报 `EntryNarrowerThanItsFieldTable`）。排在
/// [`accounting_entry_count_is_three_plus_six_per_device`] 之前：那一判要把条目解开才认得出 inode 号水位的台阶行（统计量标签与代段都在条目里），
/// 条目宽坏了时台阶行解不开、被算进 3 + 6 × 盘数，条目宽这一坏法就报成了条目数不对，不是它自己的成员。
fn every_accounting_entry_is_as_wide_as_its_field_table(
    accounting_entry_bytes: &[Vec<u8>],
) -> Result<(), RecoveryFailure> {
    for entry_bytes in accounting_entry_bytes {
        accounting_entry_parsed_at_its_field_table_width(entry_bytes)?;
    }
    Ok(())
}

/// 记账条目数是 3 + 6 × 盘数（D5（快照 / 空间记账机制） 已定项 8：池级三行加每盘六行），inode 号水位的台阶行不算在里面：
/// 它们是那一统计量代段早于这一版 txg（`root_txg`）的行，随根一直留着、条数随建过 inode 的发布数涨（D5 已定项 2 射程「inode 号水位的台阶行」）。
/// 解不开的条目算进 3 + 6 × 盘数；调用方先过 [`every_accounting_entry_is_as_wide_as_its_field_table`] 与逐条的
/// [`accounting_entry_parsed_and_judged_against_the_root`]，走到这里的每条都解得开、代不晚于根：代写到根之后的台阶行在那一判先红，不落到这里被算进条数。
fn accounting_entry_count_is_three_plus_six_per_device(
    accounting_entry_bytes: &[Vec<u8>],
    root_txg: CheckpointTxg,
    device_count: usize,
) -> Result<(), RecoveryFailure> {
    let entries_other_than_the_inode_number_watermark_steps = accounting_entry_bytes
        .iter()
        .filter(|bytes| {
            !AccountingEntry::parse(bytes).is_some_and(|entry| {
                entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg
            })
        })
        .count();
    if entries_other_than_the_inode_number_watermark_steps != 3 + 6 * device_count {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数",
        });
    }
    Ok(())
}

/// 进映射的单元各有几个（[`mapping_entry_count_is_one_per_mapped_unit`] 的输入）。
struct MappedUnitCounts {
    extent_tree_nodes: usize,
    allocation_record_tree_nodes: usize,
    accounting_tree_nodes: usize,
    /// inode 树的叶容器数 = 它的根的条目数（根恒是层级 1，下面每条条目一片容器；层级另判）。
    inode_leaf_containers: usize,
    /// 数据单元数 = extent 树里第一个文件的数据指针数（下段叶记录条数，或内联的那一个）。
    data_units: usize,
    /// 删除那两棵树的节点数：`deleted_inodes` 根（队列非空时 1）、墓碑树的每片容器与根。
    deletion_tree_nodes: usize,
}

/// 映射条目数 = 进映射的单元数：inode 根一条，extent 树、分配记录树、记账树每个节点一条，加上 inode 树的每一片叶容器与文件的每一个数据单元
/// （映射树自己的节点、树表、实例表豁免，D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
fn mapping_entry_count_is_one_per_mapped_unit(
    mapping_entries: usize,
    counts: MappedUnitCounts,
) -> Result<(), RecoveryFailure> {
    let expected = 1
        + counts.extent_tree_nodes
        + counts.allocation_record_tree_nodes
        + counts.accounting_tree_nodes
        + counts.inode_leaf_containers
        + counts.data_units
        + counts.deletion_tree_nodes;
    if mapping_entries != expected {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数",
        });
    }
    Ok(())
}

/// 分配记录的跨度不是 0、代不晚于根。已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
fn allocation_record_generations_and_spans_are_judged(
    records: &[AllocationRecord],
    root: &RootRecord,
) -> Result<(), RecoveryFailure> {
    for record in records {
        if record.generation > root.checkpoint_txg || record.span_slots == 0 {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
            });
        }
    }
    Ok(())
}

/// 解一条记账条目（条目宽是盘上的值，窄于 34 报 `EntryNarrowerThanItsFieldTable`）。
fn accounting_entry_parsed_at_its_field_table_width(
    entry_bytes: &[u8],
) -> Result<AccountingEntry, RecoveryFailure> {
    AccountingEntry::parse(entry_bytes).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
        what: "记账条目",
        entry_bytes: entry_bytes.len(),
        field_table_bytes: usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
    })
}

/// 解一条记账条目（条目宽是盘上的值，窄于 34 报 `EntryNarrowerThanItsFieldTable`），判它的代不晚于根、seq 不是 0。
fn accounting_entry_parsed_and_judged_against_the_root(
    entry_bytes: &[u8],
    root: &RootRecord,
) -> Result<AccountingEntry, RecoveryFailure> {
    let entry = accounting_entry_parsed_at_its_field_table_width(entry_bytes)?;
    if entry.generation > root.checkpoint_txg || entry.sequence == 0 {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "记账条目的代晚于根或 seq 为 0",
        });
    }
    Ok(entry)
}

/// inode 树根恒是层级 1（I-9.1）。
fn inode_tree_root_level_is_one(inode_root: &IndexNodeHeader) -> Result<(), RecoveryFailure> {
    if inode_root.level != 1 {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.1",
            detail: "inode 树根层级不是 1",
        });
    }
    Ok(())
}

/// inode 树根的一条内部条目解开之后的三样：分隔 key、身份引用、指向叶容器的子指针。
struct InodeInternalEntry {
    separator_key: u64,
    identity: PackedIdentity,
    child: NodePointer,
}

/// inode 树内部条目里的子指针头部 extent 偏移非 0 时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句
/// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构损坏）。
pub const INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
    "inode 树内部条目的子指针头部 extent 偏移不是 0";
/// inode 树内部条目里的子指针头部加密、压缩那几段（MAC / nonce、算法类型、压缩算法码与压后长度）不是第一版的取值时报
/// [`RecoveryFailure::UnitMalformed`]，`what` 取这一句（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14：判该指针所在的结构损坏）。
pub const INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
    &str = "inode 树内部条目的子指针头部的加密、压缩那几段不是第一版的取值";

/// 解一条 inode 树内部条目（条目宽是盘上的值，窄于 120 报 `EntryNarrowerThanItsFieldTable`；子指针头部 extent 偏移非 0 报
/// `UnitMalformed`，`what` 取 [`INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]，加密、压缩那几段不是第一版的取值取
/// [`INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]），判类型段是 2（I-9.2（条目身份与子头相符）
/// 的第一句；第一版 inode 树根下面只有叶容器）。
fn inode_internal_entry_parsed_and_judged(
    entry: &[u8],
) -> Result<(u64, PackedIdentity, NodePointer), RecoveryFailure> {
    let (separator_key, identity, child) =
        parse_inode_internal_entry_or_refusal(entry).map_err(|refusal| match refusal {
            PointerCarryingEntryRefusal::NarrowerThanItsFieldTable => {
                RecoveryFailure::EntryNarrowerThanItsFieldTable {
                    what: "inode 树内部条目",
                    entry_bytes: entry.len(),
                    field_table_bytes: usize::try_from(INODE_INTERNAL_ENTRY).expect("120"),
                }
            }
            PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(field) => {
                RecoveryFailure::UnitMalformed {
                    what: what_of_a_pointer_head_field_outside_the_first_version(
                        field,
                        INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
                        INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
                    ),
                }
            }
        })?;
    if identity.record_type != PACKED_TYPE_INODE {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.2",
            detail: "inode 内部条目类型段不是 2",
        });
    }
    Ok((separator_key, identity, child))
}

/// 一片 inode 叶容器按指着它的那条内部条目核（I-9.2（条目身份与子头相符）：叶头的四元组身份等于条目的身份引用、记录宽 140、fsid 是本池；
/// I-1.2：出生树与出生序号等于子指针的、诞生不晚于根、写序实例等于子指针的），再逐条解它的记录、判记录号不小于分隔 key 与容器号（I-9.4）。
/// 交回解开的记录，按容器里的次序。
fn inode_leaf_container_judged_against_its_entry(
    leaf: &PackedUnitHeader,
    entry: InodeInternalEntry,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
) -> Result<Vec<InodeRecord>, RecoveryFailure> {
    let InodeInternalEntry {
        separator_key,
        identity,
        child,
    } = entry;
    if leaf.identity != identity
        || u64::try_from(leaf.record_width).expect("记录宽") != INODE_RECORD_BYTES
        || leaf.filesystem_identifier != expected_filesystem_identifier
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.2",
            detail: "inode 叶头与条目身份引用不符",
        });
    }
    if child.head.birth_tree != identity.birth_tree || leaf.birth_sequence != child.birth_sequence {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "inode 叶的出生身份与子指针不符",
        });
    }
    if leaf.birth_txg > root.checkpoint_txg || leaf.write_order.instance != child.instance {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "inode 叶诞生于根之后或写序实例与子指针不符",
        });
    }
    let mut records = Vec::with_capacity(leaf.records.len());
    for record_bytes in &leaf.records {
        let record = InodeRecord::parse(record_bytes).ok_or(RecoveryFailure::UnitMalformed {
            what: "inode 记录",
        })?;
        if record.inode < separator_key || record.inode < identity.container {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-9.4",
                detail: "inode 记录号小于分隔 key 或容器号",
            });
        }
        records.push(record);
    }
    Ok(records)
}

/// 第一个文件的数据指针个数等于 inode size 按净荷容量除出来的单元数（D4（校验和位置） 已定项 5；与写侧切分同一条除法）——
/// 文件没有洞，第一版不写稀疏文件。
fn data_unit_count_matches_the_inode_size(
    data_units: usize,
    inode_record: &InodeRecord,
) -> Result<(), RecoveryFailure> {
    if u64::try_from(data_units).expect("单元数")
        != data_unit_count_of_a_sequential_write(inode_record.size)
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符",
        });
    }
    Ok(())
}

/// 第一个文件的第 `position` 个数据单元在查找路径上的样子：它的数据指针与 extent 树的号（数据单元头里的出生树要与它相同）。
struct DataUnitOfTheFirstFile<'pointer> {
    position: usize,
    pointer: &'pointer DataPointer,
    extent_tree: TreeIdentifier,
}

/// 一个读到手的数据单元按查找路径核（I-1.1：五元组与查找路径相符；I-9.10：对象出生代等于 inode 记录的；I-1.2：诞生代号、写序、fsid
/// 与指针相符；声明长度等于 inode size 按净荷容量切出来的这一段——最后一个单元之外恒装满一个净荷；I-2.3：补齐字节为 0），交回它的净荷。
fn data_unit_judged_against_its_pointer_and_the_inode_record<'unit>(
    bytes: &'unit [u8],
    unit: DataUnitOfTheFirstFile<'_>,
    inode_record: &InodeRecord,
    expected_filesystem_identifier: u64,
) -> Result<&'unit [u8], RecoveryFailure> {
    let payload_capacity_in_bytes = u64::try_from(data_unit_payload_capacity()).expect("32634");
    let unit_index_in_file = DataUnitIndexInFile(u64::try_from(unit.position).expect("单元序号"));
    let first_file_byte = unit_index_in_file.first_file_byte(payload_capacity_in_bytes);
    let header = parse_data_unit(bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
        what: "数据单元",
    })?;
    if header.identity.tree != unit.extent_tree
        || header.identity.object != FIRST_INODE_NUMBER
        || header.identity.anchor_offset != first_file_byte.0
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.1",
            detail: "数据单元五元组与查找路径不符",
        });
    }
    if header.identity.object_birth != inode_record.object_birth {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.10",
            detail: "对象出生代与 inode 记录不符",
        });
    }
    if header.birth_txg != unit.pointer.head.birth_txg
        || header.write_order != unit.pointer.write_order
        || header.filesystem_identifier != expected_filesystem_identifier
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "数据单元头与指针的出生身份不符",
        });
    }
    let expected_payload_length = inode_record
        .size
        .saturating_sub(first_file_byte.0)
        .min(payload_capacity_in_bytes);
    if u64::from(header.declared_length) != expected_payload_length {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "声明长度与 inode size 按净荷容量切出来的这一段不符",
        });
    }
    data_unit_payload(bytes, header.declared_length).map_err(|_error| {
        RecoveryFailure::InvariantViolated {
            invariant: "I-2.3",
            detail: "补齐字节非零",
        }
    })
}

/// 盘上一条指针的位置条目不按设备身份严格升序时报的成员（I-2.5（位置条目按设备身份升序））。
fn location_entries_not_ascending() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "I-2.5",
        detail: "位置条目不按设备身份严格升序",
    }
}

/// 根记录里的四条指针按 I-2.5 判（全零的豁免）：可写挂载要照抄写回的那几条（零单元发布、暖机照抄实例表与树表指针）从这里进来。
fn root_record_pointers_ascend_by_device(root: &RootRecord) -> Result<(), RecoveryFailure> {
    for pointer in [
        &root.tree_table,
        &root.instance_table,
        &root.mapping_root,
        &root.allocation_record_tree_root,
    ] {
        pointer
            .location_entries_ascend_by_device()
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    Ok(())
}

/// 顶着这一版的那条 journal 记录里盘上读来的位置条目按 I-2.5 判，坏的报 [`location_entries_not_ascending`]：
/// 新根段里的四条指针（树表、映射树根、实例表、分配记录树根；全零的豁免），判法与根记录那四条（[`root_record_pointers_ascend_by_device`]）、
/// 重建出来的这一版（[`pointers_of_the_rebuilt_version_ascend_by_device`]）同一个（逐条 `NodePointer::location_entries_ascend_by_device`）；
/// 每个点名项的两条位置条目（I-2.5 射程里的「journal 点名项里的位置条目数组」，点名项不是指针、没有全零豁免，
/// 与数据指针同一个判法 `pointer::location_entries_ascend_by_device`）。
/// 重建一拿到这条记录就重新编码它（`JournalRecord::to_bytes`），写者 `NodePointer::write_to` 在那里断言新根段四条升序：
/// 那一步之前不判，坏镜像就在可写挂载的那条断言上 panic（I-2.5 罩「任一块指针」，新根段这四条也是盘上读来的块指针）。
/// 点名项的写者 `NamedUnit::write_to` 不断言升序，重新编码不 panic；恢复这一侧别处没有按 I-2.5 判点名项的位置条目
/// （代码轮 `research/prompts/m3-b44-code-r3-main-verification.md` 第二节 L4 那一句），并在这一判里。
fn location_entries_of_the_record_ascend_by_device(
    record: &JournalRecord,
) -> Result<(), RecoveryFailure> {
    for pointer in [
        &record.new_tree_table,
        &record.new_mapping_root,
        &record.new_instance_table,
        &record.new_allocation_record_tree_root,
    ] {
        pointer
            .location_entries_ascend_by_device()
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    for named_unit in &record.named {
        location_entries_ascend_by_device(&named_unit.locations)
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    Ok(())
}

/// 一棵容器树（墓碑树、谱系树、退役树）这一版里盘上读来的指针：每片容器的指针与根之下每个码 2 节点的指针（根的指针住树表条目，
/// 随树表条目判）。下一次重写这棵树的某个节点时，没换的条目连同它们的指针照抄写回。
fn container_tree_pointers_read_from_disk<'version>(
    container_pointers: impl Iterator<Item = &'version NodePointer> + 'version,
    index_nodes: &'version ContainerTreeIndexNodesVersion,
) -> impl Iterator<Item = &'version NodePointer> + 'version {
    container_pointers.chain(
        index_nodes
            .pointers_below_the_root
            .iter()
            .map(|(_, pointer)| pointer),
    )
}

/// 重建出来的这一版里每一条盘上读来的指针按 I-2.5 判（C476 普查「缺口」那张表 `pointer.rs` 那一行，实审 A3a 实测坐实走得到）：
/// 下一次发布照抄没重写的角色时把它们原样写回（树表条目、多层码 2 树与分配记录树的内部条目、inode 树根的条目、extent 树、
/// 三棵容器树的内部条目），写者 `NodePointer::write_to` / `DataPointer::write_to` 在那里断言升序——不在读进来的地方判，
/// 坏镜像就在写者的断言上 panic（三棵容器树那一份：代码轮 `research/prompts/m3-b44-code-r1-main-verification.md` 第二节 K4）。
fn pointers_of_the_rebuilt_version_ascend_by_device(
    output: &TransactionOutput,
) -> Result<(), RecoveryFailure> {
    root_record_pointers_ascend_by_device(&output.root)?;
    let tombstone_tree_pointers = container_tree_pointers_read_from_disk(
        output
            .tombstone_containers
            .iter()
            .map(|container| &container.pointer),
        &output.tombstone_tree_index_nodes,
    );
    let lineage_tree_pointers = container_tree_pointers_read_from_disk(
        output
            .lineage_containers
            .iter()
            .map(|container| &container.pointer),
        &output.lineage_tree_index_nodes,
    );
    let retirement_tree_pointers = container_tree_pointers_read_from_disk(
        output
            .retirement_containers
            .iter()
            .map(|container| &container.pointer),
        &output.retirement_tree_index_nodes,
    );
    let node_pointers = output
        .tree_table_entries
        .iter()
        .map(|entry| &entry.root)
        .chain(
            output
                .extent_tree
                .upper_nodes
                .iter()
                .map(|(_, pointer)| pointer),
        )
        .chain(
            output
                .extent_tree
                .lower_nodes
                .iter()
                .map(|(_, pointer)| pointer),
        )
        .chain(
            output
                .inode_leaf_containers
                .iter()
                .map(|container| &container.pointer),
        )
        .chain(
            output
                .allocation_record_tree
                .nodes
                .iter()
                .map(|(_, pointer)| pointer),
        )
        .chain(output.accounting_tree.pointers.iter())
        .chain(output.central_mapping_tree.pointers.iter())
        .chain(tombstone_tree_pointers)
        .chain(lineage_tree_pointers)
        .chain(retirement_tree_pointers);
    for pointer in node_pointers {
        pointer
            .location_entries_ascend_by_device()
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    for data_pointer in &output.data_pointers {
        location_entries_ascend_by_device(&data_pointer.locations)
            .map_err(|_not_ascending| location_entries_not_ascending())?;
    }
    Ok(())
}

struct TreeRoots {
    extent: ExtentsOfTheFirstFile,
    /// extent 树的号（数据单元头里的出生树要与它相同）。
    extent_tree: TreeIdentifier,
    inode: IndexNodeHeader,
    allocation: AllocationRecordTreeReadFromDisk,
    accounting: CodeTwoTreeReadFromDisk,
    mapping: CodeTwoTreeReadFromDisk,
}

/// 沿树走到第一个文件：根记录 → 实例表 / 树表 → inode 树 → inode 记录 → extent 树 → 指针 → 数据单元。
#[allow(
    clippy::too_many_lines,
    reason = "一次走读就是一条挂载关键链，每一步的不变量检查都写在它发生的地方"
)]
///
/// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个）：这一版的分配记录落不落在单元区里按它判。
pub fn walk_to_file_in_the_unit_area_starting_at(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    mapping_fallbacks: &mut usize,
) -> Result<Option<Vec<u8>>, RecoveryFailure> {
    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    // 实例表单元由根记录直接持有（D22（单元原子性怎么合成） 已定项 7）。
    let instance_table_bytes =
        read_unit_via_locations(reader, &root.instance_table.locations, data_unit_bytes)?;
    let instance_table = parse_packed_unit(&instance_table_bytes).map_err(|_error| {
        RecoveryFailure::UnitMalformed {
            what: "实例表单元"
        }
    })?;
    if instance_table.identity.record_type != PACKED_TYPE_INSTANCE_TABLE
        || instance_table.records.is_empty()
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "实例表单元不是类型 4 或没有链指针记录",
        });
    }
    // 实例表是一条链（D18（块里携带什么信息） 已定项 11）：第 0 片的链指针记录说还有下一片，就沿链读到最后一片，
    // 任一片读不出、解不开，整张表就不可读，与第 0 片读不出同一个结局。只有一片的表（行数不超过 369）不多读一次盘。
    let first_page = InstanceTablePage::parse(&instance_table_bytes, InstanceTablePageIndex::FIRST)
        .ok_or(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "实例表第 0 片的身份不是 (0, 4, 0, 0)，或链指针记录、行不合法",
        })?;
    match first_page.chain {
        InstanceTableChainRecord::LastPage => {}
        InstanceTableChainRecord::NextPage(_) => {
            instance_table_chain_of_root(reader, root)?;
        }
    }
    let tree_table_bytes = read_unit_via_locations(
        reader,
        &root.tree_table.locations,
        usize::try_from(NODE_BYTES).expect("16384"),
    )?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    tree_table_key_width_is_the_tree_tables(&tree_table)?;
    if tree_table.entries.is_empty() {
        return Ok(None);
    }
    // 同一种树至多一条（审阅第 27 条）：下面按种类各读一棵，不会有第二条把前一条覆盖掉。
    let entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;
    // I-9.16（树表条目按树 ID 严格升序且合发号次序）：与重建上一版同一组判定，在读任何一棵树之前判。
    tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(&entries)?;
    let central_mapping_tree = CentralMappingTreeReadOnFirstUse {
        reader,
        root,
        expected_filesystem_identifier,
        tree: OnceCell::new(),
    };
    let central_mapping_locations_of_key =
        |mapping_key: &[u8]| central_mapping_tree.locations_of_key(mapping_key);
    let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
    let mut mapped_units_of_each_container_tree: BTreeMap<u16, usize> = BTreeMap::new();
    let mut accounting_tree: Option<CodeTwoTreeReadFromDisk> = None;
    let mut allocation_tree: Option<AllocationRecordTreeReadFromDisk> = None;
    let mut extent_tree: Option<(TreeIdentifier, ExtentsOfTheFirstFile)> = None;
    for entry in &entries {
        tree_identifier_of_the_entry_is_below_the_watermark(entry, root)?;
        if entry.root == NodePointer::empty_root() {
            continue;
        }
        // 分配记录树与 extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14）：整棵读回来，每个节点按位置与父条目核；
        // 节点进映射，提示读不出经映射回退。
        if entry.kind == TREE_KIND_ALLOCATION {
            allocation_tree = Some(read_allocation_record_tree(
                &entry.root,
                &AllocationRecordTreeGeometry::of_reader(reader),
                entry.tree,
                AllocationRecordTreeHeaderJudgement::EveryHeaderAgainstItsReference,
                root,
                expected_filesystem_identifier,
                &mut |pointer: &NodePointer| {
                    read_mapped_tree_node_via_hint_then_central_mapping(
                        reader,
                        pointer,
                        MappedTreeNodeClass::IndexNode,
                        &central_mapping_locations_of_key,
                        mapping_fallbacks,
                    )
                },
            )?);
            continue;
        }
        if entry.kind == TREE_KIND_EXTENT {
            let read = read_extent_tree(
                &ExtentTreeReading {
                    tree: entry.tree,
                    judgement: ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference,
                    root,
                    expected_filesystem_identifier,
                },
                &entry.root,
                &mut |pointer: &NodePointer| {
                    read_mapped_tree_node_via_hint_then_central_mapping(
                        reader,
                        pointer,
                        MappedTreeNodeClass::IndexNode,
                        &central_mapping_locations_of_key,
                        mapping_fallbacks,
                    )
                },
            )?;
            extent_tree = Some((entry.tree, extents_of_the_first_file(read)?));
            continue;
        }
        // 记账树可以是多层（D8（核心索引结构） 已定项 11）：整棵读回来，每个节点按父条目核；节点进映射，提示读不出经映射回退。
        if entry.kind == TREE_KIND_ACCOUNTING {
            accounting_tree = Some(read_code_two_tree(
                &entry.root,
                &MultiLevelCodeTwoTree::Accounting.read_expectation(entry.tree),
                CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
                root,
                expected_filesystem_identifier,
                &mut |pointer: &NodePointer| {
                    read_mapped_tree_node_via_hint_then_central_mapping(
                        reader,
                        pointer,
                        MappedTreeNodeClass::IndexNode,
                        &central_mapping_locations_of_key,
                        mapping_fallbacks,
                    )
                },
            )?);
            continue;
        }
        // 墓碑树与谱系树：从根往下读码 2 节点（根满长高之后不止一层）、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）。
        if entry.kind == TREE_KIND_TOMBSTONE
            || entry.kind == TREE_KIND_LINEAGE
            || entry.kind == TREE_KIND_RETIREMENT
        {
            let index_nodes = read_container_tree_index_nodes(
                entry.tree,
                entry.kind,
                "墓碑树、谱系树或退役树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
                "墓碑树、谱系树或退役树内部条目",
                &entry.root,
                root,
                expected_filesystem_identifier,
                &mut |pointer: &NodePointer| {
                    read_mapped_tree_node_via_hint_then_central_mapping(
                        reader,
                        pointer,
                        MappedTreeNodeClass::IndexNode,
                        &central_mapping_locations_of_key,
                        mapping_fallbacks,
                    )
                },
            )?;
            mapped_units_of_each_container_tree.insert(
                entry.kind,
                index_nodes.nodes.len() + index_nodes.container_entries.len(),
            );
            continue;
        }
        let key_width = key_width_for_kind(entry.kind).ok_or(RecoveryFailure::UnitMalformed {
            what: "树的种类没登记",
        })?;
        by_kind.insert(
            entry.kind,
            read_mapped_tree_root(
                reader,
                entry.tree,
                key_width,
                &entry.root,
                root,
                expected_filesystem_identifier,
                &central_mapping_locations_of_key,
                mapping_fallbacks,
            )?,
        );
    }
    // 删除那两棵树与谱系树进映射的节点数（D8（核心索引结构） 已定项 16、D18（块里携带什么信息） 已定项 24）：`deleted_inodes` 根一个，
    // 墓碑树、谱系树各自的每个码 2 节点（根在内）加每片容器（上面读码 2 节点时数下的）。
    let container_tree_nodes = |kind: u16| {
        mapped_units_of_each_container_tree
            .get(&kind)
            .copied()
            .unwrap_or(0)
    };
    let deletion_tree_nodes = usize::from(by_kind.contains_key(&TREE_KIND_DELETED_INODES))
        + container_tree_nodes(TREE_KIND_TOMBSTONE)
        + container_tree_nodes(TREE_KIND_LINEAGE)
        + container_tree_nodes(TREE_KIND_RETIREMENT);
    let mut take = |kind: u16| {
        by_kind.remove(&kind).ok_or(RecoveryFailure::UnitMalformed {
            what: "树表里缺一棵有根的树",
        })
    };
    let missing_tree = || RecoveryFailure::UnitMalformed {
        what: "树表里缺一棵有根的树",
    };
    let (extent_tree_identifier, extent) = extent_tree.ok_or_else(missing_tree)?;
    let roots = TreeRoots {
        extent,
        extent_tree: extent_tree_identifier,
        inode: take(TREE_KIND_INODE)?,
        allocation: allocation_tree.ok_or_else(missing_tree)?,
        accounting: accounting_tree.ok_or_else(missing_tree)?,
        // 中央映射树不进树表：它是哪棵树由根记录里它那条根指针的出生树说（第一个文件版本那次从水位发的号）。
        // 上面有树根的提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份，不再读一次。
        mapping: central_mapping_tree.into_tree()?,
    };
    let device_identities = reader.device_identities();
    let device_count = device_identities.len();
    allocation_records_fit_the_pool_geometry(reader, unit_area_start, &roots.allocation.records)?;
    let allocation_records: &[AllocationRecord] = &roots.allocation.records;
    allocation_records_are_one_placement_per_device_on_every_device(
        reader,
        unit_area_start,
        allocation_records,
        &device_identities,
    )?;
    // 次序与重建（`rebuild_version_in_the_unit_area_starting_at`）相同：分配记录逐条判代与跨度，记账条目宽、逐条判代与 seq，
    // 之后才数记账条目与映射条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G1）。
    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;
    every_accounting_entry_is_as_wide_as_its_field_table(
        &roots.accounting.leaf_entries_in_key_order,
    )?;
    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {
        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;
    }
    accounting_entry_count_is_three_plus_six_per_device(
        &roots.accounting.leaf_entries_in_key_order,
        root.checkpoint_txg,
        device_count,
    )?;
    mapping_entry_count_is_one_per_mapped_unit(
        roots.mapping.leaf_entries_in_key_order.len(),
        MappedUnitCounts {
            extent_tree_nodes: roots.extent.nodes_in_bump_order.len(),
            allocation_record_tree_nodes: roots.allocation.nodes.len(),
            accounting_tree_nodes: roots.accounting.version.node_count(),
            inode_leaf_containers: roots.inode.entries.len(),
            data_units: roots.extent.data_pointers.len(),
            deletion_tree_nodes,
        },
    )?;

    // inode 树：根（码 2、层级 1）→ 内部条目 → 叶容器（码 3）→ 记录。
    inode_tree_root_level_is_one(&roots.inode)?;
    let mut inode_record: Option<InodeRecord> = None;
    for entry in &roots.inode.entries {
        let (separator_key, identity, child) = inode_internal_entry_parsed_and_judged(entry)?;
        let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
            reader,
            &child,
            MappedTreeNodeClass::PackedRecordUnit,
            &|mapping_key: &[u8]| {
                central_mapping_locations_among_entries(
                    &roots.mapping.leaf_entries_in_key_order,
                    mapping_key,
                )
            },
            mapping_fallbacks,
        )?;
        let leaf =
            parse_packed_unit(&leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
                what: "inode 叶容器",
            })?;
        let records = inode_leaf_container_judged_against_its_entry(
            &leaf,
            InodeInternalEntry {
                separator_key,
                identity,
                child,
            },
            root,
            expected_filesystem_identifier,
        )?;
        for record in records {
            if record.inode == FIRST_INODE_NUMBER {
                inode_record = Some(record);
            }
        }
    }
    let Some(inode_record) = inode_record else {
        return Ok(None);
    };
    // 删除中的 inode（nlink = 0）：从意图那次发布起冷走读答「没有文件」（D8（核心索引结构） 已定项 16 第 4 条）。
    if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
        return Ok(None);
    }

    // extent 树：这个文件的数据指针按单元号排，第 i 个是文件第 i 个数据单元（读 extent 树时已按位置核过：下段叶记录的 key 是
    // (locality 0, inode 1, 第 i 个单元第一个字节的文件偏移)、单元从 0 起连号，D8（核心索引结构） 已定项 3 / 已定项 14）。
    // 个数要等于 inode size 按净荷容量除出来的单元数（D4（校验和位置） 已定项 5；与写侧切分同一条除法）——文件没有洞，第一版不写稀疏文件。
    // 解引用先按位置提示、读不到再查映射（D19（块指针的结构与宽度预算） 已定项 5）。
    data_unit_count_matches_the_inode_size(roots.extent.data_pointers.len(), &inode_record)?;
    let mut content: Vec<u8> =
        Vec::with_capacity(usize::try_from(inode_record.size).expect("文件长度装得进 usize"));
    // 迭代次数的上界是这个文件的单元数（上面核过等于 inode size 除出来的单元数）；跨轮携带的只有已经拼出来的内容，每一个提前出口都是交回一个错。
    for (position, pointer) in roots.extent.data_pointers.iter().enumerate() {
        let bytes = read_data_unit_via_hint_then_central_mapping(
            reader,
            pointer,
            &|mapping_key: &[u8]| {
                central_mapping_locations_among_entries(
                    &roots.mapping.leaf_entries_in_key_order,
                    mapping_key,
                )
            },
            mapping_fallbacks,
        )?
        .into_content()?;
        let payload = data_unit_judged_against_its_pointer_and_the_inode_record(
            &bytes,
            DataUnitOfTheFirstFile {
                position,
                pointer,
                extent_tree: roots.extent_tree,
            },
            &inode_record,
            expected_filesystem_identifier,
        )?;
        content.extend_from_slice(payload);
    }
    Ok(Some(content))
}

/// 同 [`walk_to_file_in_the_unit_area_starting_at`]，单元区起点先择一次系统配置现算（[`choose_system_configuration`]：
/// 多读每块盘的两个系统配置槽）。给手里没有择好的系统配置的调用方（用例）；[`recover`] 走带起点的那一份。
///
/// # Errors
/// 择不到系统配置（[`choose_system_configuration`] 的各个成员）；其余同带起点的那一份。
pub fn walk_to_file(
    reader: &dyn PoolReader,
    root: &RootRecord,
    mapping_fallbacks: &mut usize,
) -> Result<Option<Vec<u8>>, RecoveryFailure> {
    walk_to_file_in_the_unit_area_starting_at(
        reader,
        unit_area_start_of_the_pool(reader)?,
        root,
        mapping_fallbacks,
    )
}

/// 同 [`rebuild_version_in_the_unit_area_starting_at`]，单元区起点先择一次系统配置现算（[`choose_system_configuration`]：
/// 多读每块盘的两个系统配置槽）。给手里没有择好的系统配置的调用方（用例）；可写挂载与管理员回退走带起点的那一份。
///
/// # Errors
/// 择不到系统配置 ⇒ `Walk(`[`choose_system_configuration`] 的各个成员`)`；其余同带起点的那一份。
pub fn rebuild_version(
    reader: &dyn PoolReader,
    root: &RootRecord,
    record_standing_for_root: Option<JournalRecord>,
) -> Result<RebuiltVersion, RebuildVersionFailure> {
    rebuild_version_in_the_unit_area_starting_at(
        reader,
        unit_area_start_of_the_pool(reader)?,
        root,
        record_standing_for_root,
    )
}

/// 所选那一版走读失败之后退回哪一版（F2 乙，用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md`
/// 第三节第 7 条）：系统配置没见证过所选那一版时，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根——交回的不一定是
/// 见证到的那一版本身，是这一段里最新的那条（`F2_NEWEST`，
/// 判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H4 那一行与第三节第 1 条，用户 2026-10-08 定「四形都修」）——
/// 见证到的那一版 = 根环里 (checkpoint_txg, 实例代号) 与计数器 = c_见证 的那条记录相等的那一版；不退到比它更旧的根（`F2_EXACT`，
/// 判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与第三节第 1 条：那样已确认的那一版根槽读回全零时会退过它、
/// 记录读不出时会退到被新实例抛弃了的根），也不停在它而跳过它之后一条已确认、读得出的根（P 那次轮换写的较新系统配置槽读回全零时 c_见证 落回 P 之前）。
/// 所选根的实例表读不出时只认与见证到的那一版恰好相等的那条；c_见证 落在所选那次发布自己的某条记录上时那一段是空的、不退（所选那一版算见证过）。
/// 退到的那一条与所选根之间有一次发布带末条标志的记录读得出、根读不出、不被所选根的实例表抛弃时也不退（攻方
/// `research/prompts/m3-batch37-43-code-r1-opus-output.md` 第二节 Y4，用户 2026-10-08 定「两形都修」）：退过去会越过那次写齐了记录的发布。
/// 见证过、见证判不出、c_见证 = 0（环里读得出计数器 0 的记录也一样）、那条记录读不出、那一段里没有读得出、不被抛弃的根、或退过去会越过上面那样一次发布时
/// `None`（照报原来的失败）。
/// 恢复（[`recover`]）、可写挂载读阶段之后建上一版那一步（`mount`）、只读挂载择根
/// （`mounted_read::mount_read_only`）三处都调这一份，退过去之后不重放、直接走那一版。所选那一版是施加过记录、由记录重建的
/// （`selected_version` 不是 `chosen_root`）时不退：条款写的是「所选根走读失败」，施加过记录的那一版走不通条款没写（函数体第一句）。
///
/// 「见证过」的判法取攻方 `strict`（判决同一条）：c_见证 = 每块盘两槽里自证过的系统配置 journal tail 的最大值（与可写挂载判据 N-配置
/// 同一个读数，`mount::newer_publish_witness`）。所选那一版那次发布带末条标志的记录读得出（任一份）⇒ 它的计数器 ≤ c_见证 即见证过；
/// 读不出而计数器 = c_见证 的那条记录读得出 ⇒ 它的 (checkpoint_txg, 实例代号) 不比所选那一版旧即见证过；两条都读不出当见证过、不退。
/// 退到哪一条：上面那一段里最新的那条。根环里的根按择根同一个序（checkpoint_txg 为主、实例代号破平局）比。
///
/// 系统配置见证判不出（R2：有槽设备报读错、又没有一块盘两槽都自证过，[`system_configuration_witness_is_undecidable`]）时不退：
/// 条款只写了「系统配置没见证它时退」，判不出算不算没见证没写；这一支照报原来的失败（与今天相同），交主 agent 定。
///
/// 代价（判决同一条，用户认下）：已确认的一版在较新的系统配置槽与它的单元同时读回全零时，c_见证 落回它之前，读回的是上一版。
#[must_use]
pub fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    system_configuration: &SystemConfiguration,
    chosen_root: &RootRecord,
    selected_version: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> Option<RootRecord> {
    // 条款写的是「所选根走读失败」：只在走读失败的就是所选根本身（一条记录都没施加）时退。施加过记录、由记录重建的那一版走读失败不退：
    // 施加之前它点名的单元两份都验过（前缀第四条），走不通说明记录或单元的内容坏了，不是被后来的写盖掉；那一支条款没写，照报原来的失败（与今天相同），交主 agent 定。
    if (selected_version.instance, selected_version.checkpoint_txg)
        != (chosen_root.instance, chosen_root.checkpoint_txg)
    {
        return None;
    }
    let slot_spacing_in_bytes = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let slots_of_each_device: Vec<SystemConfigurationSlotsOfOneDevice> = reader
        .device_identities()
        .into_iter()
        .map(|device| {
            system_configuration_slots_of_one_device_read_once(
                reader,
                device,
                slot_spacing_in_bytes,
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .collect();
    if system_configuration_witness_is_undecidable(&slots_of_each_device) {
        return None;
    }
    let witnessed_journal_counter = slots_of_each_device
        .iter()
        .flat_map(|slots_of_this_device| slots_of_this_device.self_verified.iter())
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    // c_见证 = 0 不退（D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」）：写者的计数器从 1 起
    // （`journal::FIRST_JOURNAL_COUNTER`），环里读到计数器 0 的记录只能是坏盘或伪造的，不拿它当见证到的那一版。
    if witnessed_journal_counter == 0 {
        return None;
    }
    let order_of_the_root_choice = |root: &RootRecord| (root.checkpoint_txg, root.instance);
    let last_record_counter_of_the_selected_version = records
        .values()
        .filter(|record| {
            record.instance == selected_version.instance
                && record.checkpoint_txg == selected_version.checkpoint_txg
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .map(|record| record.counter)
        .max();
    let record_at_the_witnessed_counter = records
        .values()
        .filter(|record| record.counter == witnessed_journal_counter)
        .map(|record| (record.checkpoint_txg, record.instance))
        .max();
    let is_witnessed = match (
        last_record_counter_of_the_selected_version,
        record_at_the_witnessed_counter,
    ) {
        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter,
        (None, Some(witnessed_publish)) => {
            witnessed_publish >= order_of_the_root_choice(selected_version)
        }
        (None, None) => true,
    };
    if is_witnessed {
        return None;
    }
    // 计数器 = c_见证 的那条记录读不出就不退。
    let witnessed_publish = record_at_the_witnessed_counter?;
    let chosen = order_of_the_root_choice(chosen_root);
    // 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`）。c_见证 落在所选那次发布自己的某条记录上时
    // 这一段是空的：所选那一版算见证过、不退。所选根的实例表读不出时滤不了，只认与见证到的那一版恰好相等的那条（`F2_EXACT` 那一取法）。
    let instance_table_of_the_chosen_root = instance_table_chain_of_root(reader, chosen_root)
        .ok()
        .map(|chain| chain.records);
    let readable_roots_in_the_ring = readable_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    // 不跳过一次末条读得出、根读不出、不被所选根的实例表抛弃的发布（攻方 `research/prompts/m3-batch37-43-code-r1-opus-output.md` 第二节 Y4，
    // 臂 `OPUS_ARM_F2_NO_SKIP=1`，用户 2026-10-08 定「两形都修」）：退到的那一版与所选根之间有这样一次发布，说明那一版之后还有一次写齐了记录的
    // 发布（多半已确认），只是这一刻它的根读不出——退过去就悄悄越过它（较新的系统配置槽也读回全零时 c_见证 落回它之前，下界护不住它）。
    // 照报原来的失败，等故障撤掉再读。只看内存里扫出的记录与读出的根，不另读盘。所选根的实例表读不出时滤不了，那次发布算不被抛弃
    // （条款没写这一格；照攻方那条臂取，交主 agent）。那次发布的根读得出不算（它在区间里读得出又不被抛弃就会被选中；读得出而被抛弃，跳过它本来就对）。
    let newest_root_in_the_range = readable_roots_in_the_ring
        .iter()
        .copied()
        .filter(|root| {
            let order = order_of_the_root_choice(root);
            let kept_by_the_instance_table_of_the_chosen_root =
                match &instance_table_of_the_chosen_root {
                    Some(table) => !root_is_abandoned_by_the_instance_table(root, table),
                    None => order == witnessed_publish,
                };
            witnessed_publish <= order
                && order < chosen
                && kept_by_the_instance_table_of_the_chosen_root
        })
        .max_by_key(order_of_the_root_choice)?;
    let skips_a_publish_whose_last_record_reads_while_its_root_does_not =
        records.values().any(|record| {
            let order = (record.checkpoint_txg, record.instance);
            let abandoned_by_the_instance_table_of_the_chosen_root =
                instance_table_of_the_chosen_root
                    .as_ref()
                    .is_some_and(|table| {
                        publish_is_abandoned_by_the_instance_table(
                            record.instance,
                            record.checkpoint_txg,
                            table,
                        )
                    });
            record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
                && order_of_the_root_choice(&newest_root_in_the_range) < order
                && order < chosen
                && !abandoned_by_the_instance_table_of_the_chosen_root
                && !readable_roots_in_the_ring
                    .iter()
                    .any(|root| order_of_the_root_choice(root) == order)
        });
    if skips_a_publish_whose_last_record_reads_while_its_root_does_not {
        return None;
    }
    Some(newest_root_in_the_range)
}

/// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。所选那一版走读失败、系统配置没见证它时退到
/// [见证到的那一版, 所选根) 里读得出、不被抛弃的最新一条根（[`newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`]）：那一版走得通，`root` 与 `effective_root` 都报退到的那一条，
/// `journal` 仍是对所选根扫环、施加那一趟的报告；走不通照报所选那一版的失败。不看 journal 的那一臂（`JournalPolicy::Ignore`）不退。
#[must_use]
pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {
    let mut mapping_fallbacks = 0;
    let SystemConfigurationChoice {
        system_configuration,
        devices_without_a_mountable_system_configuration_slot,
    } = match choose_system_configuration_naming_devices_without_a_mountable_slot(reader) {
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
    let replayed = match policy {
        JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {
            let records = scan_journal(reader, &system_configuration);
            replay_journal(
                reader,
                &root,
                system_configuration.immutable.sizes.journal_ring_bytes,
                &records,
                policy == JournalPolicy::Consult,
            )
        }
        JournalPolicy::Ignore => Ok((JournalScanReport::default(), root)),
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
    let (outcome, effective_root) = match walk_to_file_in_the_unit_area_starting_at(
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
        // 走读失败才再扫一遍环拿记录给退回的判法（只读、不写盘；走读做成的恢复不多扫）。
        Err(failure) => {
            let version_fallen_back_to_walked = match policy {
                JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {
                    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                        reader,
                        &system_configuration,
                        &root,
                        &effective_root,
                        &scan_journal(reader, &system_configuration),
                    )
                }
                JournalPolicy::Ignore => None,
            }
            .and_then(|version_fallen_back_to| {
                walk_to_file_in_the_unit_area_starting_at(
                    reader,
                    unit_area_start,
                    &version_fallen_back_to,
                    &mut mapping_fallbacks,
                )
                .ok()
                .map(|content| (version_fallen_back_to, content))
            });
            match version_fallen_back_to_walked {
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
                None => (
                    RecoveryOutcome::Failed {
                        root: Some(root_key),
                        failure,
                    },
                    effective_root,
                ),
            }
        }
    };
    RecoveryReport {
        outcome,
        effective_root: Some((effective_root.instance, effective_root.checkpoint_txg)),
        journal,
        mapping_fallbacks,
        devices_without_a_mountable_system_configuration_slot,
    }
}

#[cfg(test)]
mod allocation_records_per_device_tests {
    use super::{
        allocation_records_are_one_per_device,
        allocation_records_are_one_placement_per_device_on_every_device,
        placements_on_each_device_after_the_new_pool_file_creation,
        NewPoolFileCreationRehearsalRefused, PlacementsOnEachDevice, PoolReader, RecoveryFailure,
    };
    use crate::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
    use crate::allocator::{AllocationRecord, UnitAreaStart};

    const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

    /// 默认几何与 1024 MiB 环那套几何的盘都是 4 GiB（`.claude/kb/layout/01-first-txn.md` 字节表五；
    /// `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的一 GiB 环那一条）。
    const FOUR_GIBIBYTES: u64 = 4 << 30;
    const ONE_GIBIBYTE_JOURNAL_RING_BYTES: u64 = 1 << 30;

    fn both_devices_of_four_gibibytes() -> Vec<(DeviceIdentity, u64)> {
        BOTH_DEVICES
            .iter()
            .map(|device| (*device, FOUR_GIBIBYTES))
            .collect()
    }

    fn placements_on_each_device_of_the_default_geometry() -> PlacementsOnEachDevice {
        placements_on_each_device_after_the_new_pool_file_creation(
            &both_devices_of_four_gibibytes(),
            UnitAreaStart::of_the_default_journal_ring(),
        )
        .expect("4 GiB 盘够长到默认环的单元区起点、装得下新池新建文件")
    }

    fn unit_area_start_after_a_one_gibibyte_journal_ring() -> UnitAreaStart {
        UnitAreaStart::following_the_journal_ring(ONE_GIBIBYTE_JOURNAL_RING_BYTES)
            .expect("1024 MiB 环末尾的下一个槽（66560）落在聚簇段边界上")
    }

    fn record(device: DeviceIdentity, slot: u64, is_released: bool) -> AllocationRecord {
        AllocationRecord {
            device,
            slot: SlotNumber(slot),
            span_slots: 2,
            generation: CheckpointTxg(3),
            is_released,
        }
    }

    /// 按下限造的形状：每盘恰好 `placements_on_each_device` 个落点、同一批槽号，从 `first_slot` 起；已释放的记录照样算一个落点。
    fn shape_with_placements_on_each_device(
        first_slot: u64,
        placements_on_each_device: PlacementsOnEachDevice,
    ) -> Vec<AllocationRecord> {
        let mut records = Vec::new();
        for placement_index in 0..placements_on_each_device.0 {
            let slot = first_slot + 2 * u64::try_from(placement_index).expect("落点序号");
            for device in BOTH_DEVICES {
                records.push(record(device, slot, placement_index == 0));
            }
        }
        records
    }

    fn new_pool_file_creation_shape() -> Vec<AllocationRecord> {
        shape_with_placements_on_each_device(
            50176,
            placements_on_each_device_of_the_default_geometry(),
        )
    }

    #[test]
    fn the_default_geometry_has_fourteen_placements_on_each_device_after_the_new_pool_file_creation(
    ) {
        assert_eq!(
            placements_on_each_device_of_the_default_geometry(),
            PlacementsOnEachDevice(14),
            "字节表五：28 条分配记录 = m1、m2、t1..t12 十四个单元 × 2 盘"
        );
    }

    #[test]
    fn the_one_gibibyte_journal_ring_geometry_has_sixteen_placements_on_each_device_after_the_new_pool_file_creation(
    ) {
        assert_eq!(
            placements_on_each_device_after_the_new_pool_file_creation(
                &both_devices_of_four_gibibytes(),
                unit_area_start_after_a_one_gibibyte_journal_ring(),
            ),
            Ok(PlacementsOnEachDevice(16)),
            "单元区起点 66560：mkfs 与数据单元在槽 81 那片叶（W = 812），提交内生块在槽 82 那片叶，分配记录树每盘多一片叶、多两个节点，提交内生块 13 个"
        );
    }

    /// 记录与盘表都空：盘表非空是这个函数的前提（文档注释的 panic 条件），在开头那一句断言上 panic、消息说的就是这个前提。
    /// 改之前走到取第一块盘的落点集合那一句 `expect`，消息写的「上面核过每块盘都有记录」在空输入上不成立（外部代码审查第一条）。
    #[test]
    #[should_panic(expected = "盘表不许为空")]
    fn an_empty_device_table_with_no_records_panics_on_the_non_empty_device_table_precondition() {
        let _ = allocation_records_are_one_per_device(&[], &[], PlacementsOnEachDevice(0));
    }

    #[test]
    fn exactly_the_placements_of_the_default_geometry_are_accepted_and_one_fewer_is_rejected() {
        let placements_on_each_device = placements_on_each_device_of_the_default_geometry();
        assert!(allocation_records_are_one_per_device(
            &new_pool_file_creation_shape(),
            &BOTH_DEVICES,
            placements_on_each_device,
        ));
        let one_fewer = shape_with_placements_on_each_device(
            50176,
            PlacementsOnEachDevice(placements_on_each_device.0 - 1),
        );
        assert!(
            !allocation_records_are_one_per_device(
                &one_fewer,
                &BOTH_DEVICES,
                placements_on_each_device
            ),
            "每盘比新池新建文件少一个落点：判红"
        );
    }

    #[test]
    fn exactly_the_placements_of_the_one_gibibyte_journal_ring_geometry_are_accepted_and_one_fewer_is_rejected(
    ) {
        let unit_area_start = unit_area_start_after_a_one_gibibyte_journal_ring();
        let placements_on_each_device = placements_on_each_device_after_the_new_pool_file_creation(
            &both_devices_of_four_gibibytes(),
            unit_area_start,
        )
        .expect("4 GiB 盘够长到 1024 MiB 环的单元区起点、装得下新池新建文件");
        assert!(allocation_records_are_one_per_device(
            &shape_with_placements_on_each_device(
                unit_area_start.slot().0,
                placements_on_each_device
            ),
            &BOTH_DEVICES,
            placements_on_each_device,
        ));
        let one_fewer = shape_with_placements_on_each_device(
            unit_area_start.slot().0,
            PlacementsOnEachDevice(placements_on_each_device.0 - 1),
        );
        assert!(
            !allocation_records_are_one_per_device(
                &one_fewer,
                &BOTH_DEVICES,
                placements_on_each_device
            ),
            "每盘比这套几何下的新池新建文件少一个落点：判红（默认几何的 14 放得过它）"
        );
    }

    #[test]
    fn placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even() {
        // 盘 0 多一条、盘 1 少一条：总数仍是盘数的整数倍，旧的「总数是盘数的整数倍」判定放过它。
        let mut records = new_pool_file_creation_shape();
        let moved = records
            .iter()
            .position(|record| record.device == DeviceIdentity(1))
            .expect("有盘 1 的记录");
        records[moved].device = DeviceIdentity(0);
        records[moved].slot = SlotNumber(50300);
        assert_eq!(
            records.len(),
            2 * placements_on_each_device_of_the_default_geometry().0
        );
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
    }

    #[test]
    fn unknown_device_or_missing_device_is_rejected() {
        let mut records = new_pool_file_creation_shape();
        records.push(record(DeviceIdentity(2), 50176, false));
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
        let only_device_zero: Vec<AllocationRecord> = new_pool_file_creation_shape()
            .into_iter()
            .filter(|record| record.device == DeviceIdentity(0))
            .collect();
        assert!(!allocation_records_are_one_per_device(
            &only_device_zero,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
    }

    /// 两盘同一个落点、一盘改写成已释放而另一盘没有：槽号集合相同，字段不同，同样拒。
    #[test]
    fn release_rewritten_on_one_device_only_is_rejected() {
        let mut records = new_pool_file_creation_shape();
        let released_on_device_one = records
            .iter()
            .position(|record| record.device == DeviceIdentity(1) && record.is_released)
            .expect("盘 1 有一条已释放");
        records[released_on_device_one].is_released = false;
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
    }

    /// 同一块盘上同一个槽两条记录（代不同、两盘对称）：槽号集合相同、字段集合也相同，靠「同盘槽号唯一」拦。
    #[test]
    fn two_records_for_the_same_slot_on_the_same_device_are_rejected() {
        let mut records = new_pool_file_creation_shape();
        for device in BOTH_DEVICES {
            let mut second_record_for_first_slot = record(device, 50176, false);
            second_record_for_first_slot.generation = CheckpointTxg(9);
            records.push(second_record_for_first_slot);
        }
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
    }

    /// 1 MiB 的盘：默认环的单元区起点（槽 50176）在它末尾之后。
    const ONE_MEBIBYTE: u64 = 1 << 20;

    /// 只交得出「池里有哪几块盘、每块多大」的读者：盘 0 是 4 GiB，盘 1 只有 1 MiB。
    struct PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart;

    impl PoolReader for PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            BOTH_DEVICES.to_vec()
        }
        fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
            match device {
                DeviceIdentity(0) => Some(FOUR_GIBIBYTES),
                DeviceIdentity(1) => Some(ONE_MEBIBYTE),
                DeviceIdentity(_) => None,
            }
        }
        fn read(
            &self,
            _device: DeviceIdentity,
            _offset: DeviceOffsetInBytes,
            _length: usize,
        ) -> Option<Vec<u8>> {
            unreachable!("分配记录每盘一条那一道判只读盘的身份与大小，不读盘上的字节")
        }
        fn journal_record_offsets_hint(
            &self,
            _device: DeviceIdentity,
            _ring_start: DeviceOffsetInBytes,
            _ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            unreachable!("分配记录每盘一条那一道判不扫 journal 环")
        }
    }

    /// 正推「发现 6」：一块盘比单元区起点短时，排演新池新建文件交回那块盘，不在建空闲图那一步 panic。
    /// 判别力：排演不先判盘长（变异「排演不判盘短于单元区起点」）时这一条 panic、红。
    #[test]
    fn rehearsal_on_a_device_shorter_than_the_unit_area_start_returns_that_device_instead_of_panicking(
    ) {
        assert_eq!(
            placements_on_each_device_after_the_new_pool_file_creation(
                &[
                    (DeviceIdentity(0), FOUR_GIBIBYTES),
                    (DeviceIdentity(1), ONE_MEBIBYTE)
                ],
                UnitAreaStart::of_the_default_journal_ring(),
            ),
            Err(
                NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart {
                    device: DeviceIdentity(1),
                    device_bytes: ONE_MEBIBYTE,
                }
            ),
            "盘 1 只有 1 MiB，到不了默认环的单元区起点"
        );
    }

    /// 同一块短盘上走读判分配记录每盘一条：记录是空的先判记录（不必排演就判得出），每盘一条而盘短报那块盘到不了单元区起点；两样都不 panic。
    /// 判别力：同上一条的变异，第二个断言 panic、红；排演挪回判记录之前（变异「排演先于判记录」）时第一个断言报的是盘短、红——
    /// 「不作为实参先求值」由第一个断言钉着。
    #[test]
    fn walk_judging_allocation_records_on_a_device_shorter_than_the_unit_area_start_returns_an_error_instead_of_panicking(
    ) {
        assert_eq!(
            allocation_records_are_one_placement_per_device_on_every_device(
                &PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart,
                UnitAreaStart::of_the_default_journal_ring(),
                &[],
                &BOTH_DEVICES,
            ),
            Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数",
            }),
            "一条分配记录都没有：先判记录，报每盘一条不成立"
        );
        assert_eq!(
            allocation_records_are_one_placement_per_device_on_every_device(
                &PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart,
                UnitAreaStart::of_the_default_journal_ring(),
                &[
                    record(DeviceIdentity(0), 50176, false),
                    record(DeviceIdentity(1), 50176, false)
                ],
                &BOTH_DEVICES,
            ),
            Err(RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                device: DeviceIdentity(1),
                device_bytes: ONE_MEBIBYTE,
            }),
            "每盘一条而盘 1 短于单元区起点：报那块盘，不 panic"
        );
    }
}

/// 盘上读来的分配记录对这个池的几何判的那四样（panic 面普查 R6 / R8 / R9）：设备身份、槽号下界、跨度上界、
/// 同一块盘上两条记录罩不罩同一个槽。四样各钉一格，加一格合法的。
///
/// 为什么在这一层钉而不是只钉在坏镜像上：跨度上界那一条，坏盘输入那一档的盘是 4 GiB（单元区到槽 262144），
/// 而那条坏法能写进跨度字段的最大值是 0x7FFF = 32767 槽——从单元区起点起也够不到末尾，
/// 它实际打中的是「两条记录罩住同一个槽」。跨度上界那一条因此在镜像那一层零覆盖，只能在这里钉。
#[cfg(test)]
mod allocation_record_pool_geometry_tests {
    use super::{allocation_records_fit_the_pool_geometry, PoolReader, RecoveryFailure};
    use crate::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
    use crate::allocator::{AllocationRecord, UnitAreaStart};
    use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};

    /// 每块盘 4 GiB（与坏盘输入那一档的盘同宽）：单元区从 [`UNIT_AREA_START_SLOT`] 起到槽
    /// 4 GiB ÷ 16 KiB = 262144。
    const DEVICE_BYTES: u64 = 4 << 30;
    const UNIT_AREA_END_SLOT: u64 = DEVICE_BYTES / SLOT_BYTES;

    /// 只交得出「池里有哪几块盘、每块多大」的读者。
    struct PoolOfTwoDevicesOfFourGibibytes;

    impl PoolReader for PoolOfTwoDevicesOfFourGibibytes {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            vec![DeviceIdentity(0), DeviceIdentity(1)]
        }
        fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
            self.device_identities()
                .contains(&device)
                .then_some(DEVICE_BYTES)
        }
        fn read(
            &self,
            _device: DeviceIdentity,
            _offset: DeviceOffsetInBytes,
            _length: usize,
        ) -> Option<Vec<u8>> {
            // 这一道判只读盘的身份与大小：它拿到的是已经解好的记录，一个字节都不再读。
            unreachable!("allocation_records_fit_the_pool_geometry 不读盘上的字节")
        }
        fn journal_record_offsets_hint(
            &self,
            _device: DeviceIdentity,
            _ring_start: DeviceOffsetInBytes,
            _ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            // 同上：这一道判不碰 journal 环。
            unreachable!("allocation_records_fit_the_pool_geometry 不扫 journal 环")
        }
    }

    fn record(device: u32, slot: u64, span_slots: u16) -> AllocationRecord {
        AllocationRecord {
            device: DeviceIdentity(device),
            slot: SlotNumber(slot),
            span_slots,
            generation: CheckpointTxg(3),
            is_released: false,
        }
    }

    /// 默认环长的池：单元区从 [`UNIT_AREA_START_SLOT`] 起（这组用例按那个常量摆记录）。
    fn judge(records: &[AllocationRecord]) -> Result<(), RecoveryFailure> {
        allocation_records_fit_the_pool_geometry(
            &PoolOfTwoDevicesOfFourGibibytes,
            UnitAreaStart::of_the_default_journal_ring(),
            records,
        )
    }

    fn outside_the_geometry(what: &'static str) -> Result<(), RecoveryFailure> {
        Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry { what })
    }

    #[test]
    fn records_inside_the_geometry_are_accepted() {
        assert_eq!(
            judge(&[
                record(0, UNIT_AREA_START_SLOT, 2),
                record(1, UNIT_AREA_START_SLOT, 2),
                record(0, UNIT_AREA_START_SLOT + 2, 2),
                record(0, UNIT_AREA_END_SLOT - 2, 2),
            ]),
            Ok(()),
            "槽号在单元区内、跨度贴着末尾也不越过、两块盘各算各的：这几条都合法"
        );
    }

    #[test]
    fn record_on_the_device_outside_the_pool_is_refused() {
        assert_eq!(
            judge(&[record(7, UNIT_AREA_START_SLOT, 2)]),
            outside_the_geometry("分配记录的设备身份不在池里"),
            "池里只有盘 0 与盘 1（R9：不判就喂进 rebuild_from_records 的 expect）"
        );
    }

    #[test]
    fn record_whose_slot_is_below_the_unit_area_is_refused() {
        assert_eq!(
            judge(&[record(0, UNIT_AREA_START_SLOT - 1, 2)]),
            outside_the_geometry("分配记录的槽号落在单元区起点之下"),
            "R6：不判就在 DeviceFreeMap::index 的减法上下溢"
        );
    }

    #[test]
    fn record_whose_span_runs_past_the_end_of_the_unit_area_is_refused() {
        assert_eq!(
            judge(&[record(0, UNIT_AREA_END_SLOT - 1, 2)]),
            outside_the_geometry("分配记录的跨度越过单元区末尾"),
            "R6：不判就在 mark_allocated 的「跨度越过单元区末尾」那条断言上"
        );
    }

    #[test]
    fn two_records_covering_the_same_slot_on_one_device_are_refused() {
        assert_eq!(
            judge(&[
                record(0, UNIT_AREA_START_SLOT, 4),
                record(0, UNIT_AREA_START_SLOT + 2, 2),
            ]),
            outside_the_geometry("同一块盘上两条分配记录罩住同一个槽"),
            "R8：不判就在 mark_allocated 的「跨度里有已分配的槽」那条断言上（I-5.4 的读路径形态）"
        );
        assert_eq!(
            judge(&[
                record(0, UNIT_AREA_START_SLOT, 2),
                record(1, UNIT_AREA_START_SLOT, 2),
            ]),
            Ok(()),
            "两盘同槽是发布路径自己的写法（D2 已定项 10），不是重叠"
        );
    }
}

/// 读侧认得删除新写出的三棵树的 key 宽（里程碑三·2 第 7 条「读侧补新种类的 key 宽」）：根指针不全零时冷走读按它读根、核根的自述 key 宽；
/// 不认得的种类在那一步拒成「树的种类没登记」。logged_ops 还没有字段表，照旧不认。
#[cfg(test)]
mod tree_kind_key_width_tests {
    use super::key_width_for_kind;
    use crate::inode_tree::INODE_KEY_WIDTH_IN_BYTES;
    use crate::records::{
        TREE_KIND_DELETED_INODES, TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LOGGED_OPERATIONS,
        TREE_KIND_RETIREMENT, TREE_KIND_TOMBSTONE,
    };

    #[test]
    fn deleted_inodes_tombstone_and_lineage_trees_have_eight_byte_keys_and_logged_operations_none()
    {
        assert_eq!(
            key_width_for_kind(TREE_KIND_DELETED_INODES),
            Some(8),
            "`deleted_inodes` 的 key = inode 号 8 字节"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_TOMBSTONE),
            Some(8),
            "墓碑树根条目的分隔 key = 容器号 8 字节"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_LINEAGE),
            Some(8),
            "谱系树根条目的分隔 key = 容器号 8 字节"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_RETIREMENT),
            Some(8),
            "退役树根条目的分隔 key = 容器号 8 字节"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_LOGGED_OPERATIONS),
            None,
            "logged_ops 没有字段表"
        );
        assert_eq!(
            key_width_for_kind(TREE_KIND_INODE),
            Some(INODE_KEY_WIDTH_IN_BYTES)
        );
        assert_eq!(
            INODE_KEY_WIDTH_IN_BYTES, 8,
            "inode 树的 key = inode 号 8 字节（D8 已定项 6）"
        );
    }
}
