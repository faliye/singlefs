//! 冷启动恢复（里程碑步 6）：择系统配置 → 择根 → 全环扫描 journal → 按前缀口径施加记录的新根段 → 沿树走到数据单元。
//! 只通过 [`PoolReader`] 读盘，不沿进程内的指针热读；崩溃点重放（步 7）拿同一条路径去判每一个崩溃状态。
//!
//! 口径：择系统配置 = 校验和过且世代号大的那个槽、各盘 fsid 与设备数要对得上（D22（单元原子性怎么合成） 已定项 16）；
//! 择根 = 三个区域全部槽里自证过、`(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7）；
//! journal 全环扫描、不先信 tail（D23（journal 的角色与格式） 已定项 3），两份镜像任一份自证过即算在（D23（journal 的角色与格式） 已定项 14）；
//! 前缀 = `(实例代号, checkpoint_txg)` 严格大于所选根、jsn 严格连续、提交标记齐全、在飞上限之内、点名单元逐项验过，
//! 施加一条记录 = 把所选根的四个字段换成记录新根段里的（D23（journal 的角色与格式） 已定项 15）。

use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    journal_in_flight_record_limit, ACCOUNTING_ENTRY_BYTES, DATA_UNIT_BYTES,
    FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, MAPPING_ENTRY_BYTES, NODE_BYTES,
    ROOT_RING_REGIONS, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
};

use crate::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    SlotNumber, TreeIdentifier,
};
use crate::allocation_record_tree::{
    read_allocation_record_tree, AllocationRecordTreeGeometry, AllocationRecordTreeHeaderJudgement,
    AllocationRecordTreeReadFromDisk,
};
use crate::allocator::{
    unit_area_slots_of_device, AllocationRecord, AllocationRecordTreeOfTheVersionWithoutFile,
};
use crate::block_device::BlockDevice;
use crate::checksum::crc32_castagnoli;
use crate::code_two_tree::{
    read_code_two_tree, CodeTwoTreeHeaderJudgement, CodeTwoTreeReadFromDisk,
};
use crate::extent_tree::{
    read_extent_tree, ExtentTreeHeaderJudgement, ExtentTreeReadFromDisk, ExtentTreeReading,
    ExtentTreeVersion, ExtentsOfAFileReadFromDisk,
};
use crate::inode_tree::{InodeLeafContainer, InodeLeafContainerIndexInTree};
use crate::instance_table::{
    InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex, InstanceTableRecords,
};
use crate::journal::{
    JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
};
use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
use crate::pointer::{DataPointer, LocationEntry, NodePointer};
use crate::records::{
    mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry, parse_mapping_entry,
    AccountingEntry, InodeRecord, TreeTableEntry, STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING,
    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_EXTENT, TREE_KIND_INODE,
    TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
};
use crate::rollback_witness::RollbackWitness;
use crate::root_record::RootRecord;
use crate::root_ring::target_for_publish;
use crate::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegionOutOfRange};
use crate::system_configuration::{
    SystemConfiguration, SystemConfigurationSlotRefusal, SystemImmutableSizes,
};
use crate::transaction::{
    role_of_allocation_record_tree_node, role_of_extent_upper_node, FileVersionTreeIdentifiers,
    InodeLeafContainerVersion, MultiLevelCodeTwoTree, PublishedUnit, TransactionOutput,
    TransactionUnit, FIRST_INODE_NUMBER,
};
use crate::unit::{
    data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
    parse_packed_unit, unit_filesystem_identifier, IndexNodeHeader, PACKED_TYPE_INODE,
    PACKED_TYPE_INSTANCE_TABLE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use crate::write_accounting::WritesByStructureKind;
use crate::write_request_split::data_unit_count_of_a_sequential_write;

/// 恢复路径读盘的口子。两个实现：文件后端的池（`Vec<(DeviceIdentity, Device)>`）与层 0 枚举出的崩溃镜像（harness）。
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
    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
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
    /// 各盘的系统配置 fsid 或设备数对不上。
    SystemConfigurationsDisagree,
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
    /// 可写挂载或回退要写的回退见证表装不下：删除规则删过之后（回退时再加上这一次那一条）条目数 `entries` 越过这个池的上限
    /// `capacity`（根环槽数减 1，D23（journal 的角色与格式） 已定项 14「回退见证」）。条款说表写不满，那是按「根环每个槽都读得出」推的；
    /// 删除规则把读不出的槽按「可能住着被抛弃的根」算，有槽持续读不出时条目删不掉、表就写得满——那时怎么办条款没写 ⇒ 第一版不支持：
    /// 挂载在取号之前返回，盘上逐字节不变。
    RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: usize, capacity: usize },
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
    /// （`RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）时是 None。记录核对器拿它判「恢复自称的状态」。
    pub effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    pub journal: JournalScanReport,
    /// 位置提示读不到、转去查映射的次数。
    pub mapping_fallbacks: usize,
}

/// 要不要看 journal：正常恢复看；层 0 里与「不看」逐状态比，问 journal 在这条流里承不承重；
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

/// 按指针里的位置条目读一个单元：逐条试，整单元 CRC-32C 等于条目里的校验和才算读到。
/// 挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
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
            return Ok(bytes);
        }
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
/// 多层时整棵读回来（D8（核心索引结构） 已定项 11），按「拼得成一棵树」核（`code_two_tree::CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds`，
/// 与重建原来读映射树根那一步同一个口径：解得开就收、不核自描述），冷走读那一份（`CentralMappingTreeReadOnFirstUse`）另核树 ID、
/// key 宽、出生身份、fsid、层级与区间。不提前读：提示都读得出的镜像上，读序与报错的次序照旧。
struct CentralMappingTreeWithBytesReadOnFirstUse<'reader> {
    reader: &'reader dyn PoolReader,
    root: &'reader RootRecord,
    tree: OnceCell<CodeTwoTreeReadFromDisk>,
}

impl<'reader> CentralMappingTreeWithBytesReadOnFirstUse<'reader> {
    fn new(reader: &'reader dyn PoolReader, root: &'reader RootRecord) -> Self {
        Self {
            reader,
            root,
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
            CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,
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

/// 一个槽读回来之后分三路：读不到 / 自证不过 ⇒ `None`（这一槽不可择，换一槽换一盘还可以试）；
/// 自述的每区槽数 S 越界 ⇒ 整池拒绝挂载，把点名的成员交回去；三关都过 ⇒ `Some`。
///
/// 单拎成一个函数是因为 [`choose_system_configuration`] 每块盘要做两次（槽 0、槽 1），
/// 而两次的分流必须一模一样：其中一次把越界悄悄当成「这一槽不可择」，越界的池就会靠另一槽挂上去。
fn mountable_slot_or_refusal(
    device: DeviceIdentity,
    bytes: Option<Vec<u8>>,
) -> Result<Option<SystemConfiguration>, RecoveryFailure> {
    let Some(bytes) = bytes else {
        return Ok(None);
    };
    match SystemConfiguration::parse_slot(&bytes) {
        Ok(system_configuration) => Ok(Some(system_configuration)),
        Err(SystemConfigurationSlotRefusal::NotSelfDescribing) => Ok(None),
        Err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange(out_of_range)) => {
            Err(RecoveryFailure::RootRingSlotsPerRegionOutOfRange {
                device,
                out_of_range,
            })
        }
    }
}

/// 每盘两槽：槽 0 在偏移 0；槽 1 的偏移按槽 0 里记的槽距，槽 0 无效时按最小槽距 4096 试。
/// 一块盘两个槽都无效时**跳过这块盘**，接着看别的盘：系统配置每盘放一份买的就是这份冗余
/// （D22（单元原子性怎么合成） 已定项 8 第 1 条；E87 的 8 种失效组合里「掉了那块盘」这一格要可挂）。
///
/// # Errors
/// 池里每块盘的两个槽都无效（[`RecoveryFailure::NoValidSystemConfiguration`]）；
/// 交得出系统配置的几块盘之间 fsid 或设备数对不上、或者池里一块盘都没有（[`RecoveryFailure::SystemConfigurationsDisagree`]）；
/// 某一槽自证得过、而自述的每区槽数 S 越界（[`RecoveryFailure::RootRingSlotsPerRegionOutOfRange`]）——
/// **这一条是挂载时的区间判**：S 越界的池在这里就拒，后面一步不走、盘上一个字节不动。
pub fn choose_system_configuration(
    reader: &dyn PoolReader,
) -> Result<SystemConfiguration, RecoveryFailure> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut chosen: Option<SystemConfiguration> = None;
    let mut first_device_with_no_valid_system_configuration_slot: Option<DeviceIdentity> = None;
    for device in reader.device_identities() {
        let slot_zero = mountable_slot_or_refusal(
            device,
            reader.read(device, DeviceOffsetInBytes(0), slot_bytes),
        )?;
        let spacing = slot_zero.as_ref().map_or(
            FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
            |system_configuration| {
                u64::from(
                    system_configuration
                        .immutable
                        .sizes
                        .fixed_structure_slot_spacing,
                )
            },
        );
        let slot_one = mountable_slot_or_refusal(
            device,
            reader.read(device, DeviceOffsetInBytes(spacing), slot_bytes),
        )?;
        let best_on_device = match (slot_zero, slot_one) {
            (None, None) => {
                // 这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上。
                if first_device_with_no_valid_system_configuration_slot.is_none() {
                    first_device_with_no_valid_system_configuration_slot = Some(device);
                }
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
        return Ok(system_configuration);
    }
    match first_device_with_no_valid_system_configuration_slot {
        Some(device) => Err(RecoveryFailure::NoValidSystemConfiguration {
            first_device_with_no_valid_system_configuration_slot: device,
        }),
        // 池里一块盘都没有：没有哪一块盘点得出名，保持这个函数原先对空池的判定不变（见报告里的设计问题）。
        None => Err(RecoveryFailure::SystemConfigurationsDisagree),
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
/// 才认得出之后哪一次发布盖掉了它（`allocator::RootRingOccupancy`）。
fn visit_valid_roots_with_ring_slots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    mut visit: impl FnMut(RootRingSlot, RootRecord),
) {
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    for region in 0..ROOT_RING_REGIONS {
        let device = region_devices[usize::try_from(region).expect("区域号")];
        for slot in 0..immutable_sizes.root_ring_slots_per_region.count() {
            let ring_slot = RootRingSlot { region, slot };
            let offset = slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
            let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
                continue;
            };
            if let Some(root) = RootRecord::parse_slot(&bytes, filesystem_identifier) {
                visit(ring_slot, root);
            }
        }
    }
}

/// 三个区域全部槽逐个验自证校验和，先跳过被回退见证表抛弃的根（D23（journal 的角色与格式） 已定项 14「回退见证」：
/// 择根先跳过被任一条目抛弃的根，再照 D22（单元原子性怎么合成） 已定项 7 择新），再取 `(checkpoint_txg, 实例代号)` 最大的。
/// 见证表从盘上现读（[`rollback_witness_of_the_pool`]）：各盘择到的那一槽里的表取并集。
#[must_use]
pub fn choose_root(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> Option<RootRecord> {
    let rollback_witness = rollback_witness_of_the_pool(reader, system_configuration);
    let mut best: Option<RootRecord> = None;
    visit_valid_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        |candidate| {
            if rollback_witness.abandons(candidate.instance, candidate.checkpoint_txg) {
                return;
            }
            let candidate_key = (candidate.checkpoint_txg, candidate.instance);
            if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance))
            {
                best = Some(candidate);
            }
        },
    );
    best
}

/// 一个池此刻的回退见证（D23（journal 的角色与格式） 已定项 14「回退见证」）：每块盘两槽里自证过（fsid 与本池相同）、
/// 世代号最大的那一槽里的见证表，各盘取并集（`RollbackWitness`）。一块盘两槽都读不出就不算它；一槽都读不出时是空的——
/// 那时系统配置本身就择不出来，挂载在择系统配置那一步已经报错（见证表读不出就是系统配置槽读不出）。
#[must_use]
pub fn rollback_witness_of_the_pool<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &SystemConfiguration,
) -> RollbackWitness {
    let spacing = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let chosen_on_each_device: Vec<SystemConfiguration> = reader
        .device_identities()
        .into_iter()
        .filter_map(|device| {
            verified_system_configuration_slots(
                reader,
                device,
                spacing,
                &system_configuration.immutable.filesystem_identifier,
            )
            .into_iter()
            .max_by_key(|slot| slot.quantities.slot_generation)
        })
        .collect();
    RollbackWitness::of_tables(
        chosen_on_each_device
            .iter()
            .map(|chosen| &chosen.rollback_witness),
    )
}

/// 根环每一个槽都读得出、都自证过（是这个池的一条根）时交回全部根，按区域、槽的次序；有一个槽读不出或自证不过就是 `None`。
/// 回退见证的删除规则要它（`mount`）：读不出的槽按「可能有被抛弃的根」算，与 D18（块里携带什么信息） 已定项 11 行回收的根环条件同一个读法。
#[must_use]
pub fn every_root_ring_slot_holds_a_root<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &SystemConfiguration,
) -> Option<Vec<RootRecord>> {
    let sizes = &system_configuration.immutable.sizes;
    let root_slot_bytes = usize::try_from(sizes.physical_block_size).expect("根槽宽");
    let mut roots = Vec::new();
    for region in 0..ROOT_RING_REGIONS {
        let device =
            system_configuration.immutable.region_devices[usize::try_from(region).expect("区域号")];
        for slot in 0..sizes.root_ring_slots_per_region.count() {
            let offset = slot_offset(
                RootRingSlot { region, slot },
                sizes.fixed_structure_slot_spacing,
            );
            let bytes = reader.read(device, offset, root_slot_bytes)?;
            roots.push(RootRecord::parse_slot(
                &bytes,
                &system_configuration.immutable.filesystem_identifier,
            )?);
        }
    }
    Some(roots)
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

/// 生效的回退下界 F（D16（发布语义） 已定项 1「生效」那一行：恢复后生效值 = 各幸存盘所带 F 最大值的最小值）；
/// 一块盘上一条根都没有就不算它，一条根都没有时 0。根落在哪块盘按它的 txg 算区域（与写者同一条公式）。
#[must_use]
pub fn effective_rollback_floor<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
) -> CheckpointTxg {
    let mut highest_per_device: BTreeMap<DeviceIdentity, CheckpointTxg> = BTreeMap::new();
    for root in readable_roots(
        reader,
        region_devices,
        immutable_sizes,
        filesystem_identifier,
    ) {
        let device = region_devices[usize::try_from(
            target_for_publish(
                root.checkpoint_txg,
                immutable_sizes.root_ring_slots_per_region,
            )
            .region,
        )
        .expect("区域号")];
        let highest = highest_per_device
            .entry(device)
            .or_insert(root.rollback_floor);
        *highest = (*highest).max(root.rollback_floor);
    }
    highest_per_device
        .values()
        .copied()
        .min()
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
/// 单元区槽数只有 [`crate::allocator::unit_area_slots_of_device`] 一处定义，分配器的位图长度读的也是它。
///
/// # Errors
/// 四样里任一样不成立 ⇒ [`RecoveryFailure::AllocationRecordOutsideThePoolGeometry`]，`what` 说清是哪一样。
fn allocation_records_fit_the_pool_geometry(
    reader: &dyn PoolReader,
    records: &[AllocationRecord],
) -> Result<(), RecoveryFailure> {
    let mut covered_slots: BTreeSet<(DeviceIdentity, u64)> = BTreeSet::new();
    for record in records {
        // 「这块盘在不在池里」只这一处判：`PoolReader::device_size_in_bytes` 对池外的盘交 `None`
        // （每个实现都从 `device_identities` 那张表里找），另写一句 `device_identities().contains(…)`
        // 是同一条判定的第二份手抄，两份会分叉（`code-discipline.md`「重复要生成，不许手抄」）。
        let device_bytes = reader.device_size_in_bytes(record.device).ok_or(
            RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "分配记录的设备身份不在池里",
            },
        )?;
        if record.slot.0 < UNIT_AREA_START_SLOT {
            return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "分配记录的槽号落在单元区起点之下",
            });
        }
        let unit_area_end_slot = UNIT_AREA_START_SLOT + unit_area_slots_of_device(device_bytes);
        let span = u64::from(record.span_slots);
        if record.slot.0 + span > unit_area_end_slot {
            return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "分配记录的跨度越过单元区末尾",
            });
        }
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

/// 一条根引用的分配记录（树表 → 分配记录树根节点）：回退的影子账要读每条被抛弃根的账。第 0 代树表（没有分配记录树）给空。
/// 分配记录树根的位置提示读不出时经这条根的中央映射回退（D19（块指针的结构与宽度预算） 已定项 8，与冷走读同一条
/// `read_mapped_tree_node_via_hint_then_central_mapping`）；树表是自举豁免，只按根记录里的位置条目读。
/// 回退的次数这里不交出去：调用方（影子账、重建分配器）今天没有接多跳观测点的口子。
///
/// # Errors
/// 树表读不到、解不开；分配记录树根提示读不出且映射里没有它（`MappingMiss`）、映射落点也读不出（`MappingStillUnreadable`）、
/// 映射树根读不出或解不开、分配记录树根解不开；条目宽或结构值判红（见 [`allocation_records_of_node`]）。
pub fn allocation_records_under_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    let mut allocation_entry = None;
    for bytes in &tree_table.entries {
        let entry = TreeTableEntry::parse(bytes).ok_or(RecoveryFailure::UnitMalformed {
            what: "树表条目",
        })?;
        if entry.kind == TREE_KIND_ALLOCATION {
            allocation_entry = Some(entry);
        }
    }
    let Some(allocation_entry) = allocation_entry else {
        return Ok(Vec::new());
    };
    let central_mapping_root = CentralMappingTreeWithBytesReadOnFirstUse::new(reader, root);
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
    allocation_records_fit_the_pool_geometry(reader, &tree.records)?;
    Ok(tree.records)
}

/// 树表 0 条的那一版自己那棵分配记录树（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））：整棵读回来
/// （D8（核心索引结构） 已定项 14：挂载时分配记录树整棵读），交回它的节点与指针、全部记录。树号 0、节点豁免映射，只按位置条目读。
/// 指针全零 ⇒ `None`，那是 mkfs 的第 0 代（那一版的账由实例表与树表两条指针直接算）。
///
/// # Errors
/// 分配记录树的节点读不到、解不开、位置对不上（`crate::allocation_record_tree::read_allocation_record_tree`）；结构值判红
/// （`allocation_records_fit_the_pool_geometry`）。
pub fn allocation_records_of_version_without_file(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Option<AllocationRecordTreeOfTheVersionWithoutFile>, RecoveryFailure> {
    if root.allocation_record_tree_root == NodePointer::empty_root() {
        return Ok(None);
    }
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree = read_allocation_record_tree(
        &root.allocation_record_tree_root,
        &AllocationRecordTreeGeometry::of_reader(reader),
        TreeIdentifier(crate::transaction::TREE_IDENTIFIER_NONE),
        AllocationRecordTreeHeaderJudgement::OnlyWhatThePositionsNeed,
        root,
        unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &NodePointer| {
            read_unit_via_locations(reader, &pointer.locations, node_bytes)
        },
    )?;
    allocation_records_fit_the_pool_geometry(reader, &tree.records)?;
    Ok(Some(AllocationRecordTreeOfTheVersionWithoutFile {
        version: tree.version(),
        records: tree.records,
    }))
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
        let Some(page) = InstanceTablePage::parse(&bytes, page_index) else {
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
    let read = read_instance_table_chain(reader, root);
    read.outcome.map(|records| InstanceTableChain {
        records,
        page_pointers: read.page_pointers,
    })
}

/// 一条根指着的实例表链上认得出的每一片的指针（第 0 片恒在）：读得出的就沿链往下认，读不出、解不开的那一片自己的指针也在，
/// 它后面的认不出。影子账拿它隔离一条被抛弃根的整条链——读不全的链少隔离一片，被抛弃根在环里被覆写之前那一片就可能被发出去。
#[must_use]
pub fn instance_table_page_pointers_as_far_as_readable<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
) -> Vec<NodePointer> {
    read_instance_table_chain(reader, root).page_pointers
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

/// 所选根的实例在它自己指着的实例表里有回退行时，回退行的 W（D23（journal 的角色与格式） 已定项 14 前缀第五条：
/// 该实例的记录只施加到 W 为止）；没有回退行、实例表读不出或解不开都是 `None`。
#[must_use]
pub fn rollback_high_water_of_root(reader: &dyn PoolReader, root: &RootRecord) -> Option<u64> {
    instance_table_of_root(reader, root)?
        .rows
        .iter()
        .find(|row| row.instance == root.instance && row.is_rollback)
        .map(|row| row.applied_transaction_high_water)
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
/// # Errors
/// 树表不是 0 条而 `record_standing_for_root` 是 `None` ⇒ `NoRecordStandingForFileVersion`；实例表、树表、映射树根读不到，
/// 树根或 inode 叶容器提示读不出且经映射也读不回（`MappingMiss` / `MappingStillUnreadable`），单元解不开 ⇒ 走读同款的错；
/// 记账行里没有 inode 号水位那一行 ⇒ `InodeNumberWatermarkRowMissingFromTheAccountingTree`。
pub fn rebuild_version(
    reader: &dyn PoolReader,
    root: &RootRecord,
    record_standing_for_root: Option<JournalRecord>,
) -> Result<RebuiltVersion, RebuildVersionFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let instance_table_bytes =
        read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?;
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    if tree_table.entries.is_empty() {
        return Ok(RebuiltVersion::WithoutFile);
    }
    let record =
        record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
    let record_bytes = record.to_bytes();
    let mut tree_table_entries = Vec::with_capacity(tree_table.entries.len());
    for bytes in &tree_table.entries {
        tree_table_entries.push(TreeTableEntry::parse(bytes).ok_or(
            RecoveryFailure::UnitMalformed {
                what: "树表条目"
            },
        )?);
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
    // 这一版那八棵树各自的号照盘上读回来：七棵按树表条目的种类取条目里的树 ID，中央映射树不进树表、取根记录里
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
    };
    // 豁免三类之外的单元（数据单元、四棵树的根、inode 叶容器）位置提示读不出时经这一版的中央映射回退，与冷走读同一条
    // （D19（块指针的结构与宽度预算） 已定项 8）；映射树根、树表、实例表是自举豁免，只按根记录里的位置条目读。
    // 映射树根到第一次要回退、或走到它那一步时才读：提示都读得出的镜像上读序与报错次序照旧。
    let central_mapping_root = CentralMappingTreeWithBytesReadOnFirstUse::new(reader, root);
    let central_mapping_locations_of_key =
        |mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key);
    // 回退的次数这里不交出去：`RebuiltVersion` 与可写挂载今天没有接多跳观测点的口子（挂载态的读有，`MountedPoolForRead`）。
    let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
    let read_node =
        |pointer: &NodePointer, what: &'static str, stale_location_hint_hops: &mut usize| {
            let bytes = read_mapped_tree_node_via_hint_then_central_mapping(
                reader,
                pointer,
                MappedTreeNodeClass::IndexNode,
                &central_mapping_locations_of_key,
                stale_location_hint_hops,
            )?;
            let node = parse_index_node(&bytes)
                .map_err(|_error| RecoveryFailure::UnitMalformed { what })?;
            Ok::<(Vec<u8>, IndexNodeHeader), RecoveryFailure>((bytes, node))
        };
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
            judgement: ExtentTreeHeaderJudgement::OnlyWhatThePositionsNeed,
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
    let (inode_root_bytes, inode_root_node) = read_node(
        &inode_root_pointer,
        "inode 树根",
        &mut stale_location_hint_hops_not_exposed_by_this_reader,
    )?;
    if inode_root_node.entries.is_empty() {
        return Err(RecoveryFailure::UnitMalformed {
            what: "inode 树根没有条目",
        }
        .into());
    }
    // inode 树的每一片叶容器：根的条目按分隔 key 升序，逐条读回容器与它装的记录（D8（核心索引结构） 已定项 6）。
    let mut inode_leaf_containers: Vec<InodeLeafContainerVersion> = Vec::new();
    let mut inode_leaf_container_bytes: Vec<Vec<u8>> = Vec::new();
    for entry in &inode_root_node.entries {
        let (_separator_key, _identity_in_the_entry, leaf_pointer) = parse_inode_internal_entry(
            entry,
        )
        .ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
            what: "inode 树内部条目",
            entry_bytes: entry.len(),
            field_table_bytes: usize::try_from(INODE_INTERNAL_ENTRY).expect("120"),
        })?;
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
        let mut records = Vec::with_capacity(leaf.records.len());
        for inode_record_bytes in &leaf.records {
            records.push(InodeRecord::parse(inode_record_bytes).ok_or(
                RecoveryFailure::UnitMalformed {
                    what: "inode 记录"
                },
            )?);
        }
        if records.is_empty() {
            return Err(RecoveryFailure::UnitMalformed {
                what: "inode 叶容器没有记录",
            }
            .into());
        }
        inode_leaf_containers.push(InodeLeafContainerVersion {
            // 身份取容器头那一份：条目里的身份引用与它逐字相等是 I-9.2（条目身份与子头相符），走读同款在 `walk_to_file` 判。
            contents: InodeLeafContainer {
                identity: leaf.identity,
                records,
            },
            pointer: leaf_pointer,
        });
        inode_leaf_container_bytes.push(leaf_bytes);
    }
    let inode_record = inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|inode_record| inode_record.inode == FIRST_INODE_NUMBER)
        .copied()
        .ok_or(RecoveryFailure::UnitMalformed {
            what: "inode 树里没有第一个文件那条记录",
        })?;
    // 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14）：整棵读回来，每个节点按位置核，节点进映射、提示读不出经映射回退。
    let allocation_tree_read = read_allocation_record_tree(
        &allocation_pointer,
        &AllocationRecordTreeGeometry::of_reader(reader),
        tree_identifiers.allocation_records,
        AllocationRecordTreeHeaderJudgement::OnlyWhatThePositionsNeed,
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
    allocation_records_fit_the_pool_geometry(reader, &allocation_tree_read.records)?;
    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();
    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退
    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树按「拼得成一棵树」核（`code_two_tree::read_code_two_tree`）。
    let accounting_tree = read_code_two_tree(
        &accounting_pointer,
        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),
        CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,
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
    let mut accounting_entries: Vec<AccountingEntry> =
        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());
    for bytes in &accounting_tree.leaf_entries_in_key_order {
        accounting_entries.push(AccountingEntry::parse(bytes).ok_or(
            RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "记账条目",
                entry_bytes: bytes.len(),
                field_table_bytes: usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
            },
        )?);
    }
    // 记账行的统计量标签是盘上读来的 2 字节：这一版交出去之前判一次「inode 号水位那一行在」，往里就信
    // （`TransactionOutput::inode_number_watermark` 那句 expect 依赖的就是这一判；panic 面普查 R11）。
    if !accounting_entries
        .iter()
        .any(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
    {
        return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());
    }
    // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。
    let mapping_tree = central_mapping_root.into_tree()?;
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
    // 记账树与中央映射树的每个节点，按 bump 次序（树内先叶后根），与发布路径装出来的 `units` 同序。
    for (tree, read) in [
        (MultiLevelCodeTwoTree::Accounting, &accounting_tree),
        (MultiLevelCodeTwoTree::CentralMapping, &mapping_tree),
    ] {
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
    Ok(RebuiltVersion::WithFile(TransactionOutput {
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
    }))
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
/// 树表单元读不到、解不开；树表条目解不开、种类没登记，或同一种树出现两条。
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
    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
    for entry_bytes in &tree_table.entries {
        let entry = TreeTableEntry::parse(entry_bytes).ok_or(RecoveryFailure::UnitMalformed {
            what: "树表条目",
        })?;
        let slot_for_this_tree = match entry.kind {
            TREE_KIND_INODE => &mut pointers.inode_tree,
            TREE_KIND_EXTENT => &mut pointers.extent_tree,
            TREE_KIND_ALLOCATION
            | TREE_KIND_ACCOUNTING
            | TREE_KIND_LIVELIST
            | TREE_KIND_SPARSE_SIDE_TABLE
            | TREE_KIND_DEADLIST => continue,
            _ => {
                return Err(RecoveryFailure::UnitMalformed {
                    what: "树的种类没登记",
                })
            }
        };
        if slot_for_this_tree
            .replace(TreeTableEntry::root_pointer_bytes(entry_bytes).to_vec())
            .is_some()
        {
            return Err(RecoveryFailure::UnitMalformed {
                what: "树表里同一种树有两条",
            });
        }
    }
    Ok(pointers)
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
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    [0, slot_spacing_in_bytes]
        .into_iter()
        .filter_map(|offset| reader.read(device, DeviceOffsetInBytes(offset), slot_bytes))
        // 这里把每区槽数 S 越界的槽与自证不过的槽一样丢掉：走到这一步之前
        // `choose_system_configuration` 已经在同一个池上判过区间、越界的池根本挂不上，
        // 这个函数再报一次越界也没有第二种处置。
        .filter_map(|bytes| SystemConfiguration::parse_slot(&bytes).ok())
        .filter(|system_configuration| {
            system_configuration.immutable.filesystem_identifier == *filesystem_identifier
        })
        .collect()
}

/// 全环扫描：每盘每个记录槽都读（有提示时只读提示的那些），两份镜像任一份自证过即算在；fsid 不符的不算数。
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
    let mut records = BTreeMap::new();
    for device in reader.device_identities() {
        let offsets = reader
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
            .unwrap_or_else(|| {
                (0..ring_bytes / JOURNAL_RECORD_BYTES)
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
                records
                    .entry((record.instance, record.counter))
                    .or_insert(record);
            }
        }
    }
    records
}

/// 一条记录是不是它那次发布的末条：记录标志位 0（D23（journal 的角色与格式） 已定项 14 第六条「发布边界按记录标志位 0 认」、
/// 已定项 17：只有真正的最后一条带「本次发布末条」标志，每次只有一条记录的发布——含空发布记录——那一条也带）。
fn record_ends_its_publish(record: &JournalRecord) -> bool {
    match record.place_in_publish {
        JournalRecordPlaceInPublish::LastRecordOfThePublish => true,
        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,
    }
}

/// 同一 (实例代号, checkpoint_txg) 里 `record` 之后（计数器更大）还有没有读得出的记录。同一实例里计数器与 checkpoint_txg 一起往上走，
/// 所以从下一个计数器起按计数器升序看到 txg 越过这一条的就停。
fn a_readable_record_of_the_same_publish_follows(
    record: &JournalRecord,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> bool {
    let Some(next_counter) = record.counter.checked_add(1) else {
        return false;
    };
    records
        .range((record.instance, next_counter)..=(record.instance, u64::MAX))
        .map(|(_, later)| later)
        .take_while(|later| later.checkpoint_txg <= record.checkpoint_txg)
        .any(|later| later.checkpoint_txg == record.checkpoint_txg)
}

/// 所选根覆盖的最后一条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定）：
/// 「那条」按末条标志认——所选根那次发布（与所选根同实例、同 checkpoint_txg）读得出的几条里带「本次发布末条」标志的那一条。
/// 一条都不带 ⇒ `Ok(None)`，就算「那条读不出」，链首走「序号为 1 的第一条可读记录」那一支；**不取**读得出的同 txg 记录里
/// jsn 最大的那条（读法甲：末条读不出而前几条读得出时，它锚在前几条上，下一次发布一条都接不上）。
///
/// # Errors
/// 带标志的多于一条 ⇒ `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`：认哪一条条款没有写。
fn counter_of_the_last_record_the_root_covers(
    root: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> Result<Option<u64>, RecoveryFailure> {
    // `records` 按 (实例代号, 计数器) 排序 ⇒ 同一实例里按计数器升序。
    let counters_carrying_the_last_record_flag: Vec<u64> = records
        .values()
        .filter(|record| {
            record.instance == root.instance
                && record.checkpoint_txg == root.checkpoint_txg
                && record_ends_its_publish(record)
        })
        .map(|record| record.counter)
        .collect();
    if counters_carrying_the_last_record_flag.len() > 1 {
        return Err(
            RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                instance: root.instance,
                checkpoint_txg: root.checkpoint_txg,
                counters: counters_carrying_the_last_record_flag,
            },
        );
    }
    // 至多一条：带标志的那一条就是锚点，一条都没有就是「那条读不出」。
    Ok(counters_carrying_the_last_record_flag.first().copied())
}

/// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
///
/// 链首锚在所选根覆盖的最后一条：所选根那次发布里带「本次发布末条」标志的那一条（已定项 14 注 1，读法乙，
/// [`counter_of_the_last_record_the_root_covers`]）。
/// 施加的单位是一次发布（第六条）：前五条判出来的前缀里，一次发布的记录要一直走到带末条标志的那一条（[`record_ends_its_publish`]）
/// 才整体施加；前缀停在一次发布中间（末条没到、断号、校验不过、回退行的 W 截在中间、下一条换了 txg、一次发布之内跳号、
/// 一个事务的提交标记没出现）⇒ 那次发布整体不施加。读者规则另外两格（D23（journal 的角色与格式） 已定项 4）同样断链、那次发布不施加：
/// 一次发布的首条序号不是 1（断在这一条）；带末条标志的那一条之后同一 (实例代号, checkpoint_txg) 里还有读得出的记录（断在带标志的那一条）。
///
/// # Errors
/// 所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写）⇒
/// `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`，一条记录都没施加。
pub fn replay_journal(
    reader: &dyn PoolReader,
    root: &RootRecord,
    ring_bytes: u64,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    verify_named_units: bool,
    rollback_high_water: Option<u64>,
) -> Result<(JournalScanReport, RootRecord), RecoveryFailure> {
    // 被回退见证表抛弃的记录不施加（D23（journal 的角色与格式） 已定项 14「回退见证」随实现：见证表同时管择根与重放；
    // 前缀第五条读见证的这一读法交代码三方）。见证表住系统配置槽里，读不出就是系统配置读不出，恢复报错、不按空表施加。
    let system_configuration = choose_system_configuration(reader)?;
    let rollback_witness = rollback_witness_of_the_pool(reader, &system_configuration);
    let mut report = JournalScanReport {
        valid_records: records.len(),
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
        maximum_applied_transaction: 0,
    };
    let water = (root.instance, root.checkpoint_txg);
    let mut rebuilt = *root;
    // 前缀规则不跨实例边界（D23（journal 的角色与格式） 已定项 14 第 1 条）：链从所选根覆盖的最后一条记录之后接，
    // 下一条的实例代号与所选根不同即停——所以只有所选根自己那个实例的记录是候选；所选根是 mkfs 的第 0 代根时一条都不施加。
    let mut above: Vec<&JournalRecord> = records
        .values()
        .filter(|record| {
            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water
        })
        .collect();
    above.sort_by_key(|record| (record.instance, record.counter));
    report.above_water = above.len();
    let in_flight_limit =
        usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
    // 链首锚点 = 所选根覆盖的最后一条：那次发布里带末条标志的那一条（已定项 14 注 1，读法乙）。
    // 取那次发布 jsn 最小那条时，一次发布切成多条记录的那一版上链首落在那次发布自己的第二条（它不在水位之上），
    // 水位之上的第一条对不上号、一条都不施加（三方第一轮 K3：零故障少施加）。
    // 带标志的那条读不出（两份都撕了）时不知道它的 jsn，链首只能是水位之上第一条可读记录，前提是它的 checkpoint_txg = 根的 txg + 1、
    // 本次发布内序号为 1（已定项 14 注 1 / 已定项 4）。txg 更大 ⇒ 中间少了一次发布，断号即止（里程碑「第二个事务」步 3
    // 三方第一轮攻方腿打中：无锚点时无条件接上会跳过撕掉的一条）；序号不是 1 ⇒ 下一次发布的开头缺了，同样断号即止——
    // 只看 txg 时，下一次发布的第一条也读不出、第二条读得出，链首就接在第二条上，缺了第一条的那次发布照样整体施加
    // （三方第一轮 K4-b：多接）。
    let root_own_record_counter = counter_of_the_last_record_the_root_covers(root, records)?;
    let mut expected_next: Option<(InstanceGeneration, u64)> =
        root_own_record_counter.map(|counter| (root.instance, counter + 1));
    let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);
    // 这次发布已经过了前五条、还没走到末条的记录（第六条：末条到了才整体施加）。
    let mut records_of_the_open_publish: Vec<&JournalRecord> = Vec::new();
    for record in above.into_iter().take(in_flight_limit) {
        if let Some(expected_key) = expected_next {
            if (record.instance, record.counter) != expected_key {
                break;
            }
        } else if record.checkpoint_txg != chain_start_txg_without_anchor
            || record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST
        {
            break;
        }
        // 被回退见证表抛弃的记录（所选根那个实例里 txg 越过回退目标的那一段）：它之后同一实例的记录只会更靠后，断在这里。
        // 所选根被见证表抛弃的不会被择中（`choose_root`），所以这一判拦的是「所选根是 R_old 或更早、它之后的被抛弃记录还在环里」
        // 那一格（C332（回退实例两个根都读不出时回退被撤销） 里落到 R_old 的那一支）。
        if rollback_witness.abandons(record.instance, record.checkpoint_txg) {
            break;
        }
        // 一次发布的第一条序号是 1（D23（journal 的角色与格式） 已定项 4 读者规则：锚点读得出时，下一次发布的首条序号不是 1，
        // 当那条记录损坏、断在这一条，那次发布整体不施加；C539（锚点读得出时下一次发布的首条序号不是 1））。锚点读不出那一支上面已经判过；
        // 这一判管锚点读得出时接上的那一次，与之后每一次新开的发布。
        if records_of_the_open_publish.is_empty()
            && record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST
        {
            break;
        }
        // 前缀第五条：所选根的实例有回退行时只施加到回退行的 W 为止——W = 0 就是「之后的一个都不算」，
        // 空发布（事务号 0）也不许把根推过 T_old。
        if let Some(high_water) = rollback_high_water {
            if high_water == 0 || record.transaction > high_water {
                break;
            }
        }
        if let Some(previous_record_of_the_open_publish) = records_of_the_open_publish.last() {
            // 一次发布的记录共享一个 checkpoint_txg（D16（发布语义） 已定项 6）：末条还没到、下一条已经换了 txg
            // ⇒ 这次发布缺了末条，断在这里，它整体不施加。
            if record.checkpoint_txg != previous_record_of_the_open_publish.checkpoint_txg {
                break;
            }
            // 一次发布的 N 条记录序号依次 1..N、与 jsn 同步（D23（journal 的角色与格式） 已定项 4）：一次发布之内跳号，
            // 当这条记录损坏、断链即止——这次发布走不到末条，整体不施加。
            if u64::from(record.ordinal_within_publish.0)
                != u64::from(previous_record_of_the_open_publish.ordinal_within_publish.0) + 1
            {
                break;
            }
            // 提交标记（D23（journal 的角色与格式） 已定项 7）：一个事务可以跨多条记录，只有它的最后一条带提交标记
            // （最后一个事务装不下一条记录时末条再跨记录，已定项 17）。上一条没带提交标记而这一条换了事务号 ⇒
            // 那个事务的提交标记没出现，它被丢掉（已定项 7「丢掉提交标记还没出现的那个事务的全部记录」），
            // 它所在的这次发布因此不完整、整体不施加（第六条）。
            if !previous_record_of_the_open_publish.is_commit
                && record.transaction != previous_record_of_the_open_publish.transaction
            {
                break;
            }
        }
        expected_next = Some((record.instance, record.counter + 1));
        // 末条不带提交标记 ⇒ 这次发布最后一个事务没提交，这次发布整体不施加。末条之外不带提交标记的，是一个跨多条记录的事务
        // 还没写到它的最后一条，接着往下走（它换了事务号还没等到提交标记，上面那一判断链）。
        if !record.is_commit && record_ends_its_publish(record) {
            break;
        }
        // 带末条标志的这一条之后，同一 (实例代号, checkpoint_txg) 里还有读得出的记录（D23（journal 的角色与格式） 已定项 4 读者规则：
        // 当这条记录损坏、断在带标志的这一条，那次发布整体不施加；C540（末条标志坏在一次发布中间，读者切出两次发布））。
        // 合法历史里一次发布只有真正的最后一条带标志（已定项 17），失败的那次原样重发（已定项 14「这一版的失败处置」），走到这里要一条记录坏了而
        // 校验和恰好仍对得上，或者镜像是改出来的。
        if record_ends_its_publish(record)
            && a_readable_record_of_the_same_publish_follows(record, records)
        {
            break;
        }
        let all_verified = !verify_named_units
            || record.named.iter().all(|named| {
                let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
                    return false;
                };
                named.locations.iter().all(|location| {
                    reader
                        .read(
                            location.device,
                            location.slot.to_device_offset(),
                            unit_bytes,
                        )
                        .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
                })
            });
        if !all_verified {
            report.verification_failed += 1;
            break;
        }
        // 只数**真验过**的：`verify_named_units` 关掉时上面那句把 `all_verified` 短路成真、一个单元都没读，
        // 这一格再加一就等于宣称验过了（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测——
        // 两臂在健康镜像上报出逐字相同的数，运行时就看不出走了哪一条）。
        if verify_named_units {
            report.verification_passed += 1;
        }
        records_of_the_open_publish.push(record);
        // 第六条：末条没到就接着往下走，这次发布先不施加。
        if !record_ends_its_publish(record) {
            continue;
        }
        report.prefix_applied += records_of_the_open_publish.len();
        report.maximum_applied_transaction = records_of_the_open_publish
            .iter()
            .map(|applied| applied.transaction)
            .fold(report.maximum_applied_transaction, u64::max);
        records_of_the_open_publish.clear();
        // 每条记录都带整次发布的新根段（已定项 15），施加末条那一份。
        rebuilt = RootRecord {
            filesystem_identifier: rebuilt.filesystem_identifier,
            instance: record.instance,
            checkpoint_txg: record.checkpoint_txg,
            tree_table: record.new_tree_table,
            tree_identifier_watermark: record.new_tree_identifier_watermark,
            rollback_floor: record.new_rollback_floor,
            instance_table: rebuilt.instance_table,
            mapping_root: record.new_mapping_root,
            // 与实例表指针同一条理由：新根段（D23（journal 的角色与格式） 已定项 15）里没有这一项，施加记录只能照抄被施加的那条根的。
            // 施加一条写行记录而它的根槽没落盘时，重建出来的这一版仍指着上一版的分配记录树——那正是「这次写行没有成立」该有的账。
            allocation_record_tree_root: rebuilt.allocation_record_tree_root,
        };
    }
    Ok((report, rebuilt))
}

/// 每棵树的 key 宽（码 2 头里的自述 key 宽要与它相符）；day-1 只注册的三棵没有节点要解。
fn key_width_for_kind(kind: u16) -> Option<usize> {
    match kind {
        TREE_KIND_EXTENT => Some(24),
        TREE_KIND_INODE => Some(8),
        TREE_KIND_ALLOCATION => Some(10),
        TREE_KIND_ACCOUNTING => Some(22),
        TREE_KIND_LIVELIST | TREE_KIND_SPARSE_SIDE_TABLE | TREE_KIND_DEADLIST => None,
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
/// 每盘不少于 10 个落点（mkfs 2 + 第一个事务 8）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
/// 走读自己不判 key 严格递增，这里逐盘核槽号不重复。
#[must_use]
pub fn allocation_records_are_one_per_device(
    records: &[AllocationRecord],
    device_identities: &[DeviceIdentity],
) -> bool {
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
    let first_device_placements = placement_sets.next().expect("上面核过每块盘都有记录");
    first_device_placements.len() >= FIRST_TRANSACTION_PLACEMENTS_PER_DEVICE
        && placement_sets.all(|placements| placements == first_device_placements)
}

/// mkfs 写在单元区里的 2 个落点加第一个事务的 8 个落点（字节表五：20 条记录，每盘 10 条），之后每次发布只多不少。
const FIRST_TRANSACTION_PLACEMENTS_PER_DEVICE: usize = 10;

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
pub fn walk_to_file(
    reader: &dyn PoolReader,
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
    if tree_table.key_width != TREE_TABLE_KEY_WIDTH {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "树表单元自述 key 宽不是 8",
        });
    }
    if tree_table.entries.is_empty() {
        return Ok(None);
    }
    let mut entries = Vec::with_capacity(tree_table.entries.len());
    for bytes in &tree_table.entries {
        entries.push(
            TreeTableEntry::parse(bytes).ok_or(RecoveryFailure::UnitMalformed {
                what: "树表条目",
            })?,
        );
    }
    let central_mapping_tree = CentralMappingTreeReadOnFirstUse {
        reader,
        root,
        expected_filesystem_identifier,
        tree: OnceCell::new(),
    };
    let central_mapping_locations_of_key =
        |mapping_key: &[u8]| central_mapping_tree.locations_of_key(mapping_key);
    let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
    let mut accounting_tree: Option<CodeTwoTreeReadFromDisk> = None;
    let mut allocation_tree: Option<AllocationRecordTreeReadFromDisk> = None;
    let mut extent_tree: Option<(TreeIdentifier, ExtentsOfTheFirstFile)> = None;
    for entry in &entries {
        if entry.tree.0 >= root.tree_identifier_watermark {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-7.8",
                detail: "树 ID 不低于水位",
            });
        }
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
    allocation_records_fit_the_pool_geometry(reader, &roots.allocation.records)?;
    let allocation_records: &[AllocationRecord] = &roots.allocation.records;
    // 每个落点每盘一条（两盘同槽）：第一个事务 10 × 盘数，每次覆盖写再加 8 × 盘数（换下的那些改写、不删）。
    // 只核总数是盘数的整数倍拦不住「一盘多一条、另一盘少一条」——发布 B 三方第一轮正推腿打中，改成逐盘核同一批（槽, 跨度）。
    if !allocation_records_are_one_per_device(allocation_records, &device_identities) {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于 10 个落点",
        });
    }
    if roots.accounting.leaf_entries_in_key_order.len() != 3 + 6 * device_count {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "记账条目数不是 3 + 6 × 盘数",
        });
    }
    // 进映射的单元：inode 根一条，extent 树、分配记录树、记账树每个节点一条，加上 inode 树的每一片叶容器与文件的每一个数据单元
    // （映射树自己的节点、树表、实例表豁免，D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
    // inode 树的叶容器数 = 它的根的条目数（根恒是层级 1，下面每条条目一片容器；层级检查在下面）；
    // 数据单元数 = extent 树里第一个文件的数据指针数（下段叶记录条数，或内联的那一个）。
    let mapping_entries_expected = 1
        + roots.extent.nodes_in_bump_order.len()
        + roots.allocation.nodes.len()
        + roots.accounting.version.node_count()
        + roots.inode.entries.len()
        + roots.extent.data_pointers.len();
    if roots.mapping.leaf_entries_in_key_order.len() != mapping_entries_expected {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数",
        });
    }
    // 已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
    for record in allocation_records {
        if record.generation > root.checkpoint_txg || record.span_slots == 0 {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
            });
        }
    }
    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {
        let entry = AccountingEntry::parse(entry_bytes).ok_or(
            RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "记账条目",
                entry_bytes: entry_bytes.len(),
                field_table_bytes: usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
            },
        )?;
        if entry.generation > root.checkpoint_txg || entry.sequence == 0 {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "记账条目的代晚于根或 seq 为 0",
            });
        }
    }

    // inode 树：根（码 2、层级 1）→ 内部条目 → 叶容器（码 3）→ 记录。
    if roots.inode.level != 1 {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.1",
            detail: "inode 树根层级不是 1",
        });
    }
    let mut inode_record: Option<InodeRecord> = None;
    for entry in &roots.inode.entries {
        let (separator_key, identity, child) = parse_inode_internal_entry(entry).ok_or(
            RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "inode 树内部条目",
                entry_bytes: entry.len(),
                field_table_bytes: usize::try_from(INODE_INTERNAL_ENTRY).expect("120"),
            },
        )?;
        if identity.record_type != PACKED_TYPE_INODE {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-9.2",
                detail: "inode 内部条目类型段不是 2",
            });
        }
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
        if leaf.identity != identity
            || u64::try_from(leaf.record_width).expect("记录宽") != INODE_RECORD_BYTES
            || leaf.filesystem_identifier != expected_filesystem_identifier
        {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-9.2",
                detail: "inode 叶头与条目身份引用不符",
            });
        }
        if child.head.birth_tree != identity.birth_tree
            || leaf.birth_sequence != child.birth_sequence
        {
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
        for record_bytes in &leaf.records {
            let record =
                InodeRecord::parse(record_bytes).ok_or(RecoveryFailure::UnitMalformed {
                    what: "inode 记录",
                })?;
            if record.inode < separator_key || record.inode < identity.container {
                return Err(RecoveryFailure::InvariantViolated {
                    invariant: "I-9.4",
                    detail: "inode 记录号小于分隔 key 或容器号",
                });
            }
            if record.inode == FIRST_INODE_NUMBER {
                inode_record = Some(record);
            }
        }
    }
    let Some(inode_record) = inode_record else {
        return Ok(None);
    };

    // extent 树：这个文件的数据指针按单元号排，第 i 个是文件第 i 个数据单元（读 extent 树时已按位置核过：下段叶记录的 key 是
    // (locality 0, inode 1, 第 i 个单元第一个字节的文件偏移)、单元从 0 起连号，D8（核心索引结构） 已定项 3 / 已定项 14）。
    // 个数要等于 inode size 按净荷容量除出来的单元数（D4（校验和位置） 已定项 5；与写侧切分同一条除法）——文件没有洞，第一版不写稀疏文件。
    // 解引用先按位置提示、读不到再查映射（D19（块指针的结构与宽度预算） 已定项 5）。
    let payload_capacity_in_bytes = u64::try_from(data_unit_payload_capacity()).expect("32634");
    if u64::try_from(roots.extent.data_pointers.len()).expect("单元数")
        != data_unit_count_of_a_sequential_write(inode_record.size)
    {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符",
        });
    }
    let mut content: Vec<u8> =
        Vec::with_capacity(usize::try_from(inode_record.size).expect("文件长度装得进 usize"));
    // 迭代次数的上界是这个文件的单元数（上面核过等于 inode size 除出来的单元数）；跨轮携带的只有已经拼出来的内容，每一个提前出口都是交回一个错。
    for (position, pointer) in roots.extent.data_pointers.iter().enumerate() {
        let unit_index_in_file = DataUnitIndexInFile(u64::try_from(position).expect("单元序号"));
        let first_file_byte = unit_index_in_file.first_file_byte(payload_capacity_in_bytes);
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
        let header = parse_data_unit(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "数据单元",
        })?;
        if header.identity.tree != roots.extent_tree
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
        if header.birth_txg != pointer.head.birth_txg
            || header.write_order != pointer.write_order
            || header.filesystem_identifier != expected_filesystem_identifier
        {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-1.2",
                detail: "数据单元头与指针的出生身份不符",
            });
        }
        // 最后一个单元之外恒装满一个净荷（与写侧切分同一条口径）。
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
        let payload = data_unit_payload(&bytes, header.declared_length).map_err(|_error| {
            RecoveryFailure::InvariantViolated {
                invariant: "I-2.3",
                detail: "补齐字节非零",
            }
        })?;
        content.extend_from_slice(payload);
    }
    Ok(Some(content))
}

/// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。
#[must_use]
pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {
    let mut mapping_fallbacks = 0;
    let system_configuration = match choose_system_configuration(reader) {
        Ok(system_configuration) => system_configuration,
        Err(failure) => {
            return RecoveryReport {
                outcome: RecoveryOutcome::Failed {
                    root: None,
                    failure,
                },
                effective_root: None,
                journal: JournalScanReport::default(),
                mapping_fallbacks,
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
                rollback_high_water_of_root(reader, &root),
            )
        }
        JournalPolicy::Ignore => Ok((JournalScanReport::default(), root)),
    };
    // 恢复在施加任何一条记录之前停下（锚点认哪一条条款没写）：没有「实际走的那条根」可报。
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
            }
        }
    };
    let outcome = match walk_to_file(reader, &effective_root, &mut mapping_fallbacks) {
        Ok(Some(content)) => RecoveryOutcome::FileRead {
            root: root_key,
            content,
        },
        Ok(None) => RecoveryOutcome::NoFile { root: root_key },
        Err(failure) => RecoveryOutcome::Failed {
            root: Some(root_key),
            failure,
        },
    };
    RecoveryReport {
        outcome,
        effective_root: Some((effective_root.instance, effective_root.checkpoint_txg)),
        journal,
        mapping_fallbacks,
    }
}

#[cfg(test)]
mod allocation_records_per_device_tests {
    use super::{allocation_records_are_one_per_device, FIRST_TRANSACTION_PLACEMENTS_PER_DEVICE};
    use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
    use crate::allocator::AllocationRecord;

    const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

    fn record(device: DeviceIdentity, slot: u64, is_released: bool) -> AllocationRecord {
        AllocationRecord {
            device,
            slot: SlotNumber(slot),
            span_slots: 2,
            generation: CheckpointTxg(3),
            is_released,
        }
    }

    /// 第一个事务的形状：每盘 10 个落点、同一批槽号；已释放的记录照样算一个落点。
    fn first_transaction_shape() -> Vec<AllocationRecord> {
        let mut records = Vec::new();
        for placement_index in 0..FIRST_TRANSACTION_PLACEMENTS_PER_DEVICE {
            let slot = 50176 + 2 * u64::try_from(placement_index).expect("落点序号");
            for device in BOTH_DEVICES {
                records.push(record(device, slot, placement_index == 0));
            }
        }
        records
    }

    #[test]
    fn first_transaction_shape_is_one_record_per_placement_per_device() {
        assert!(allocation_records_are_one_per_device(
            &first_transaction_shape(),
            &BOTH_DEVICES
        ));
    }

    #[test]
    fn placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even() {
        // 盘 0 多一条、盘 1 少一条：总数仍是 20，旧的「总数是盘数的整数倍」判定放过它。
        let mut records = first_transaction_shape();
        let moved = records
            .iter()
            .position(|record| record.device == DeviceIdentity(1))
            .expect("有盘 1 的记录");
        records[moved].device = DeviceIdentity(0);
        records[moved].slot = SlotNumber(50300);
        assert_eq!(records.len(), 2 * FIRST_TRANSACTION_PLACEMENTS_PER_DEVICE);
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES
        ));
    }

    #[test]
    fn unknown_device_or_missing_device_is_rejected() {
        let mut records = first_transaction_shape();
        records.push(record(DeviceIdentity(2), 50176, false));
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES
        ));
        let only_device_zero: Vec<AllocationRecord> = first_transaction_shape()
            .into_iter()
            .filter(|record| record.device == DeviceIdentity(0))
            .collect();
        assert!(!allocation_records_are_one_per_device(
            &only_device_zero,
            &BOTH_DEVICES
        ));
    }

    /// 两盘同一个落点、一盘改写成已释放而另一盘没有：槽号集合相同，字段不同，同样拒。
    #[test]
    fn release_rewritten_on_one_device_only_is_rejected() {
        let mut records = first_transaction_shape();
        let released_on_device_one = records
            .iter()
            .position(|record| record.device == DeviceIdentity(1) && record.is_released)
            .expect("盘 1 有一条已释放");
        records[released_on_device_one].is_released = false;
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES
        ));
    }

    /// 同一块盘上同一个槽两条记录（代不同、两盘对称）：槽号集合相同、字段集合也相同，靠「同盘槽号唯一」拦。
    #[test]
    fn two_records_for_the_same_slot_on_the_same_device_are_rejected() {
        let mut records = first_transaction_shape();
        for device in BOTH_DEVICES {
            let mut second_record_for_first_slot = record(device, 50176, false);
            second_record_for_first_slot.generation = CheckpointTxg(9);
            records.push(second_record_for_first_slot);
        }
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES
        ));
    }

    #[test]
    fn fewer_than_the_first_transaction_placements_is_rejected() {
        let records: Vec<AllocationRecord> = first_transaction_shape()
            .into_iter()
            .filter(|record| record.slot != SlotNumber(50176))
            .collect();
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES
        ));
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
    use crate::allocator::AllocationRecord;
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

    fn judge(records: &[AllocationRecord]) -> Result<(), RecoveryFailure> {
        allocation_records_fit_the_pool_geometry(&PoolOfTwoDevicesOfFourGibibytes, records)
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
    fn a_record_on_a_device_outside_the_pool_is_refused() {
        assert_eq!(
            judge(&[record(7, UNIT_AREA_START_SLOT, 2)]),
            outside_the_geometry("分配记录的设备身份不在池里"),
            "池里只有盘 0 与盘 1（R9：不判就喂进 rebuild_from_records 的 expect）"
        );
    }

    #[test]
    fn a_record_whose_slot_is_below_the_unit_area_is_refused() {
        assert_eq!(
            judge(&[record(0, UNIT_AREA_START_SLOT - 1, 2)]),
            outside_the_geometry("分配记录的槽号落在单元区起点之下"),
            "R6：不判就在 DeviceFreeMap::index 的减法上下溢"
        );
    }

    #[test]
    fn a_record_whose_span_runs_past_the_end_of_the_unit_area_is_refused() {
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
