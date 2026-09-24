//! 冷启动恢复（里程碑步 6）：择系统配置 → 择根 → 全环扫描 journal → 按前缀口径施加记录的新根段 → 沿树走到数据单元。
//! 只通过 [`PoolReader`] 读盘，不沿进程内的指针热读；崩溃点重放（步 7）拿同一条路径去判每一个崩溃状态。
//!
//! 口径：择系统配置 = 校验和过且世代号大的那个槽、各盘 fsid 与设备数要对得上（D22（单元原子性怎么合成） 已定项 16）；
//! 择根 = 三个区域全部槽里自证过、`(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7）；
//! journal 全环扫描、不先信 tail（D23（journal 的角色与格式） 已定项 3），两份镜像任一份自证过即算在（D23（journal 的角色与格式） 已定项 14）；
//! 前缀 = `(实例代号, checkpoint_txg)` 严格大于所选根、jsn 严格连续、提交标记齐全、在飞上限之内、点名单元逐项验过，
//! 施加一条记录 = 把所选根的四个字段换成记录新根段里的（D23（journal 的角色与格式） 已定项 15）。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    journal_in_flight_record_limit, ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES,
    DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
    INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT,
    MAPPING_ENTRY_BYTES, NODE_BYTES, ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOT_BYTES, TREE_IDENTIFIER_CENTRAL_MAPPING, UNIT_AREA_START_SLOT,
};

use crate::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use crate::allocator::{unit_area_slots_of_device, AllocationRecord};
use crate::block_device::BlockDevice;
use crate::checksum::crc32_castagnoli;
use crate::inode_tree::{InodeLeafContainer, InodeLeafContainerIndexInTree};
use crate::instance_table::InstanceTableRecords;
use crate::journal::JournalRecord;
use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
use crate::pointer::{DataPointer, LocationEntry, NodePointer};
use crate::records::{
    mapping_key_for_data, mapping_key_for_node, parse_extent_record, parse_inode_internal_entry,
    parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry, STATISTIC_INODE_WATERMARK,
    TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_EXTENT,
    TREE_KIND_INODE, TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
};
use crate::root_record::RootRecord;
use crate::root_ring::target_for_publish;
use crate::root_ring::{slot_offset, RootRingSlot};
use crate::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use crate::transaction::{
    InodeLeafContainerVersion, PublishedUnit, TransactionOutput, TransactionUnit,
    FIRST_INODE_NUMBER,
};
use crate::unit::{
    data_unit_payload, parse_data_unit, parse_index_node, parse_packed_unit,
    unit_filesystem_identifier, IndexNodeHeader, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE,
    UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use crate::write_accounting::WritesByStructureKind;

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
    /// 施加 journal 之后实际走的那条根（所选根，或由记录重建的根）；择不到根时是 None。记录核对器拿它判「恢复自称的状态」。
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
pub(crate) fn read_unit_via_locations(
    reader: &dyn PoolReader,
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

/// 每盘两槽：槽 0 在偏移 0；槽 1 的偏移按槽 0 里记的槽距，槽 0 无效时按最小槽距 4096 试。
/// 一块盘两个槽都无效时**跳过这块盘**，接着看别的盘：系统配置每盘放一份买的就是这份冗余
/// （D22（单元原子性怎么合成） 已定项 8 第 1 条；E87 的 8 种失效组合里「掉了那块盘」这一格要可挂）。
///
/// # Errors
/// 池里每块盘的两个槽都无效（[`RecoveryFailure::NoValidSystemConfiguration`]）；
/// 交得出系统配置的几块盘之间 fsid 或设备数对不上、或者池里一块盘都没有（[`RecoveryFailure::SystemConfigurationsDisagree`]）。
pub fn choose_system_configuration(
    reader: &dyn PoolReader,
) -> Result<SystemConfiguration, RecoveryFailure> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut chosen: Option<SystemConfiguration> = None;
    let mut first_device_with_no_valid_system_configuration_slot: Option<DeviceIdentity> = None;
    for device in reader.device_identities() {
        let slot_zero = reader
            .read(device, DeviceOffsetInBytes(0), slot_bytes)
            .and_then(|bytes| SystemConfiguration::parse_slot(&bytes));
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
        let slot_one = reader
            .read(device, DeviceOffsetInBytes(spacing), slot_bytes)
            .and_then(|bytes| SystemConfiguration::parse_slot(&bytes));
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
    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
    for region in 0..ROOT_RING_REGIONS {
        let device = region_devices[usize::try_from(region).expect("区域号")];
        for slot in 0..ROOT_RING_SLOTS_PER_REGION {
            let offset = slot_offset(
                RootRingSlot { region, slot },
                immutable_sizes.fixed_structure_slot_spacing,
            );
            let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
                continue;
            };
            if let Some(root) = RootRecord::parse_slot(&bytes, filesystem_identifier) {
                visit(root);
            }
        }
    }
}

/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的。
#[must_use]
pub fn choose_root(
    reader: &dyn PoolReader,
    system_configuration: &SystemConfiguration,
) -> Option<RootRecord> {
    let mut best: Option<RootRecord> = None;
    visit_valid_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        |candidate| {
            let candidate_key = (candidate.checkpoint_txg, candidate.instance);
            if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance))
            {
                best = Some(candidate);
            }
        },
    );
    best
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
        let device = region_devices
            [usize::try_from(target_for_publish(root.checkpoint_txg).region).expect("区域号")];
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

/// 把一个索引节点的条目逐条解成分配记录，再按这个池的几何判一遍。
///
/// 两道判分得开：前一道是**字段表**（条目宽够不够装下 20 字节的记录），后一道是**几何**
/// （设备身份、槽号、跨度、两条记录罩不罩同一个槽）。分配器那一侧的下标、位图长度与两条断言
/// 全部按这两道已经判过来写（`allocator::DeviceFreeMap::index` / `mark_allocated` 的消息指着这里）。
///
/// # Errors
/// 条目宽不足 ⇒ [`RecoveryFailure::EntryNarrowerThanItsFieldTable`]；
/// 结构值出了这个池的几何 ⇒ [`RecoveryFailure::AllocationRecordOutsideThePoolGeometry`]。
fn allocation_records_of_node(
    reader: &dyn PoolReader,
    node: &IndexNodeHeader,
) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
    let mut records = Vec::with_capacity(node.entries.len());
    for bytes in &node.entries {
        records.push(AllocationRecord::parse(bytes).ok_or(
            RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "分配记录",
                entry_bytes: bytes.len(),
                field_table_bytes: usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
            },
        )?);
    }
    allocation_records_fit_the_pool_geometry(reader, &records)?;
    Ok(records)
}

/// 盘上读来的分配记录逐条对这个池的几何判一遍：这几个字段可以是任何值，它们不是不变量。
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
///
/// # Errors
/// 树表或分配记录树根读不到、解不开；条目宽或结构值判红（见 [`allocation_records_of_node`]）。
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
    let mut allocation_pointer = None;
    for bytes in &tree_table.entries {
        let entry = TreeTableEntry::parse(bytes).ok_or(RecoveryFailure::UnitMalformed {
            what: "树表条目",
        })?;
        if entry.kind == TREE_KIND_ALLOCATION {
            allocation_pointer = Some(entry.root);
        }
    }
    let Some(allocation_pointer) = allocation_pointer else {
        return Ok(Vec::new());
    };
    let allocation_bytes =
        read_unit_via_locations(reader, &allocation_pointer.locations, node_bytes)?;
    let allocation_node =
        parse_index_node(&allocation_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "分配记录树根",
        })?;
    // 第一版的分配记录树只有一个节点（层 0），多层的树这条路还不会走（里程碑「第二个事务」步 6 的欠账）；
    // 读到层 > 0 的根就报格式错，不把内部节点的指针当分配记录解。
    if allocation_node.level != 0 {
        return Err(RecoveryFailure::UnitMalformed {
            what: "分配记录树根不止一层",
        });
    }
    allocation_records_of_node(reader, &allocation_node)
}

/// 一条根指着的实例表（行与链指针）；单元读不出或解不开都是 `None`（读不出的由走读另报）。
#[must_use]
pub fn instance_table_of_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Option<InstanceTableRecords> {
    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let bytes = read_unit_via_locations(reader, &root.instance_table.locations, data_bytes).ok()?;
    InstanceTableRecords::parse(&bytes)
}

/// 一条根指着的树表是不是 0 条（mkfs 的第 0 代树表，第一次可写挂载的暖机根照抄它）：这条根下面还没有发布过文件版本。
///
/// # Errors
/// 树表单元读不到、解不开。
pub fn tree_table_has_no_entries(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<bool, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    Ok(tree_table.entries.is_empty())
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
/// （照抄没重写的角色、经映射释放被换下的角色都靠它；可写挂载在恢复之后调）。`record_standing_for_root` 是所选根自己那条记录
/// （读不出时由调用方顶一条；树表 0 条时用不到）。
///
/// # Errors
/// 树表不是 0 条而 `record_standing_for_root` 是 `None` ⇒ `NoRecordStandingForFileVersion`；单元读不到或解不开 ⇒ 走读同款的错；
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
    let read_node = |pointer: &NodePointer, what: &'static str| {
        let bytes = read_unit_via_locations(reader, &pointer.locations, node_bytes)?;
        let node =
            parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed { what })?;
        Ok::<(Vec<u8>, IndexNodeHeader), RecoveryFailure>((bytes, node))
    };
    let (extent_bytes, extent_node) = read_node(&extent_pointer, "extent 树根")?;
    let (_, data_pointer) = parse_extent_record(extent_node.entries.first().ok_or(
        RecoveryFailure::UnitMalformed {
            what: "extent 树根没有记录",
        },
    )?)
    .ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
        what: "extent 叶记录",
        entry_bytes: extent_node.entry_width,
        field_table_bytes: usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
    })?;
    let data_unit_bytes = read_unit_via_locations(reader, &data_pointer.locations, data_bytes)?;
    let (inode_root_bytes, inode_root_node) = read_node(&inode_root_pointer, "inode 树根")?;
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
        let leaf_bytes = read_unit_via_locations(reader, &leaf_pointer.locations, data_bytes)?;
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
    let (allocation_bytes, allocation_node) = read_node(&allocation_pointer, "分配记录树根")?;
    let allocation_records: Vec<AllocationRecord> =
        allocation_records_of_node(reader, &allocation_node)?;
    let (accounting_bytes, accounting_node) = read_node(&accounting_pointer, "记账树根")?;
    let mut accounting_entries: Vec<AccountingEntry> =
        Vec::with_capacity(accounting_node.entries.len());
    for bytes in &accounting_node.entries {
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
    let (mapping_bytes, mapping_node) = read_node(&root.mapping_root, "映射树根")?;
    let mut mapping_keys: Vec<Vec<u8>> = Vec::with_capacity(mapping_node.entries.len());
    for entry in &mapping_node.entries {
        let (key, _locations) =
            parse_mapping_entry(entry).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "映射条目",
                entry_bytes: entry.len(),
                field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
            })?;
        mapping_keys.push(key);
    }
    let mut mapped_units = vec![
        (
            TransactionUnit::Data,
            mapping_key_for_data(data_pointer.head, data_pointer.write_order),
        ),
        (
            TransactionUnit::ExtentRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer),
        ),
    ];
    for (position, container) in inode_leaf_containers.iter().enumerate() {
        mapped_units.push((
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
                position,
            )),
            mapping_key_for_node(UNIT_CLASS_PACKED, container.pointer),
        ));
    }
    mapped_units.extend([
        (
            TransactionUnit::InodeRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
        ),
        (
            TransactionUnit::AllocationTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
        ),
        (
            TransactionUnit::AccountingTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer),
        ),
    ]);
    let unit =
        |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
            slot: locations[0].slot,
            identity,
            bytes,
        };
    let mut units = vec![
        unit(
            TransactionUnit::Data,
            &data_pointer.locations,
            data_unit_bytes,
        ),
        unit(
            TransactionUnit::ExtentRoot,
            &extent_pointer.locations,
            extent_bytes,
        ),
    ];
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
    units.extend([
        unit(
            TransactionUnit::InodeRoot,
            &inode_root_pointer.locations,
            inode_root_bytes,
        ),
        unit(
            TransactionUnit::AllocationTree,
            &allocation_pointer.locations,
            allocation_bytes,
        ),
        unit(
            TransactionUnit::AccountingTree,
            &accounting_pointer.locations,
            accounting_bytes,
        ),
        unit(
            TransactionUnit::MappingTree,
            &root.mapping_root.locations,
            mapping_bytes,
        ),
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
        units,
        rewritten: Vec::new(),
        data_pointer,
        mapping_keys,
        allocation_records,
        accounting_entries,
        tree_table_entries,
        inode_record,
        inode_leaf_containers,
        mapped_units,
        released: Vec::new(),
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
        .filter_map(|bytes| SystemConfiguration::parse_slot(&bytes))
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

/// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
pub fn replay_journal(
    reader: &dyn PoolReader,
    root: &RootRecord,
    ring_bytes: u64,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    verify_named_units: bool,
    rollback_high_water: Option<u64>,
) -> (JournalScanReport, RootRecord) {
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
    // 链首接在所选根自己那条记录（同实例、checkpoint_txg 相等）之后；那条记录读不出（两份都撕了）时不知道它的 jsn，
    // 链首只能是 checkpoint_txg = 根的 txg + 1 的那条（第一版一次发布一条记录、txg 每次加一）：水位之上最小的那条若 txg 更大，
    // 中间就少了一条，断号即止照样成立（里程碑「第二个事务」步 3 三方第一轮攻方腿打中：无锚点时无条件接上会跳过撕掉的一条）。
    let root_own_record_counter = records
        .values()
        .find(|record| {
            record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
        })
        .map(|record| record.counter);
    let mut expected_next: Option<(InstanceGeneration, u64)> =
        root_own_record_counter.map(|counter| (root.instance, counter + 1));
    let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);
    for record in above.into_iter().take(in_flight_limit) {
        if let Some(expected_key) = expected_next {
            if (record.instance, record.counter) != expected_key {
                break;
            }
        } else if record.checkpoint_txg != chain_start_txg_without_anchor {
            break;
        }
        // 前缀第五条：所选根的实例有回退行时只施加到回退行的 W 为止——W = 0 就是「之后的一个都不算」，
        // 空发布（事务号 0）也不许把根推过 T_old。
        if let Some(high_water) = rollback_high_water {
            if high_water == 0 || record.transaction > high_water {
                break;
            }
        }
        expected_next = Some((record.instance, record.counter + 1));
        if !record.is_commit {
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
        report.prefix_applied += 1;
        report.maximum_applied_transaction =
            report.maximum_applied_transaction.max(record.transaction);
        rebuilt = RootRecord {
            filesystem_identifier: rebuilt.filesystem_identifier,
            instance: record.instance,
            checkpoint_txg: record.checkpoint_txg,
            tree_table: record.new_tree_table,
            tree_identifier_watermark: record.new_tree_identifier_watermark,
            rollback_floor: record.new_rollback_floor,
            instance_table: rebuilt.instance_table,
            mapping_root: record.new_mapping_root,
        };
    }
    (report, rebuilt)
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

/// 读一棵树的根节点并核它的自描述：树 ID、key 宽、出生身份、fsid、key 区间与条目相符。
/// 挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
pub(crate) fn read_tree_root(
    reader: &dyn PoolReader,
    tree: TreeIdentifier,
    key_width: usize,
    pointer: &NodePointer,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
) -> Result<IndexNodeHeader, RecoveryFailure> {
    let bytes = read_unit_via_locations(
        reader,
        &pointer.locations,
        usize::try_from(NODE_BYTES).expect("16384"),
    )?;
    let node = parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
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

struct TreeRoots {
    extent: IndexNodeHeader,
    inode: IndexNodeHeader,
    allocation: IndexNodeHeader,
    accounting: IndexNodeHeader,
    mapping: IndexNodeHeader,
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
    let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
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
        let key_width = key_width_for_kind(entry.kind).ok_or(RecoveryFailure::UnitMalformed {
            what: "树的种类没登记",
        })?;
        by_kind.insert(
            entry.kind,
            read_tree_root(
                reader,
                entry.tree,
                key_width,
                &entry.root,
                root,
                expected_filesystem_identifier,
            )?,
        );
    }
    let mut take = |kind: u16| {
        by_kind.remove(&kind).ok_or(RecoveryFailure::UnitMalformed {
            what: "树表里缺一棵有根的树",
        })
    };
    let roots = TreeRoots {
        extent: take(TREE_KIND_EXTENT)?,
        inode: take(TREE_KIND_INODE)?,
        allocation: take(TREE_KIND_ALLOCATION)?,
        accounting: take(TREE_KIND_ACCOUNTING)?,
        mapping: read_tree_root(
            reader,
            TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
            27,
            &root.mapping_root,
            root,
            expected_filesystem_identifier,
        )?,
    };
    let device_identities = reader.device_identities();
    let device_count = device_identities.len();
    let allocation_records: Vec<AllocationRecord> =
        allocation_records_of_node(reader, &roots.allocation)?;
    // 每个落点每盘一条（两盘同槽）：第一个事务 10 × 盘数，每次覆盖写再加 8 × 盘数（换下的那些改写、不删）。
    // 只核总数是盘数的整数倍拦不住「一盘多一条、另一盘少一条」——发布 B 三方第一轮正推腿打中，改成逐盘核同一批（槽, 跨度）。
    if !allocation_records_are_one_per_device(&allocation_records, &device_identities) {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于 10 个落点",
        });
    }
    if roots.accounting.entries.len() != 3 + 6 * device_count {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "记账条目数不是 3 + 6 × 盘数",
        });
    }
    // 进映射的单元：数据、extent 根、inode 根、分配记录树、记账树各一条，加上 inode 树的每一片叶容器
    // （映射树自己、树表、实例表豁免，D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
    // inode 树的叶容器数 = 它的根的条目数（根恒是层级 1，下面每条条目一片容器；层级检查在下面）。
    let mapping_entries_expected = 5 + roots.inode.entries.len();
    if roots.mapping.entries.len() != mapping_entries_expected {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "映射条目数不是 5 + inode 叶容器数",
        });
    }
    // 已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
    for record in &allocation_records {
        if record.generation > root.checkpoint_txg || record.span_slots == 0 {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
            });
        }
    }
    for entry_bytes in &roots.accounting.entries {
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
        let leaf_bytes = read_unit_via_locations(reader, &child.locations, data_unit_bytes)?;
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

    // extent 树：按 (locality 0, inode 1, offset 0) 找指针；解引用先按位置提示、读不到再查映射（D19（块指针的结构与宽度预算） 已定项 5）。
    let mut wanted_key = [0u8; 24];
    wanted_key[8..16].copy_from_slice(&FIRST_INODE_NUMBER.to_le_bytes());
    let mut data_unit: Option<(DataPointer, Vec<u8>)> = None;
    for record_bytes in &roots.extent.entries {
        let (key, pointer) = parse_extent_record(record_bytes).ok_or(
            RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "extent 叶记录",
                entry_bytes: record_bytes.len(),
                field_table_bytes: usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
            },
        )?;
        if key != wanted_key {
            continue;
        }
        let bytes = match read_unit_via_locations(reader, &pointer.locations, data_unit_bytes) {
            Ok(bytes) => bytes,
            Err(_hint_error) => {
                *mapping_fallbacks += 1;
                let mapping_key = mapping_key_for_data(pointer.head, pointer.write_order);
                let mut mapped = None;
                for entry in &roots.mapping.entries {
                    let (candidate_key, locations) = parse_mapping_entry(entry).ok_or(
                        RecoveryFailure::EntryNarrowerThanItsFieldTable {
                            what: "映射条目",
                            entry_bytes: entry.len(),
                            field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
                        },
                    )?;
                    if candidate_key == mapping_key {
                        mapped = Some(locations);
                    }
                }
                let Some(locations) = mapped else {
                    return Err(RecoveryFailure::MappingMiss {
                        slot: pointer.locations[0].slot,
                    });
                };
                read_unit_via_locations(reader, &locations, data_unit_bytes).map_err(|_error| {
                    RecoveryFailure::MappingStillUnreadable {
                        slot: locations[0].slot,
                    }
                })?
            }
        };
        data_unit = Some((pointer, bytes));
    }
    let Some((pointer, bytes)) = data_unit else {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "extent 树里没有这个文件的记录",
        });
    };
    let header = parse_data_unit(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
        what: "数据单元",
    })?;
    if header.identity.tree.0 != roots.extent.tree.0
        || header.identity.object != FIRST_INODE_NUMBER
        || header.identity.anchor_offset != 0
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
    if u64::from(header.declared_length) != inode_record.size {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "声明长度与 inode size 不符",
        });
    }
    let payload = data_unit_payload(&bytes, header.declared_length).map_err(|_error| {
        RecoveryFailure::InvariantViolated {
            invariant: "I-2.3",
            detail: "补齐字节非零",
        }
    })?;
    Ok(Some(payload.to_vec()))
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
    let (journal, effective_root) = match policy {
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
        JournalPolicy::Ignore => (JournalScanReport::default(), root),
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
