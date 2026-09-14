//! 冷启动恢复（里程碑步 6）：择超级块 → 择根 → 全环扫描 journal → 按前缀口径施加记录的新根段 → 沿树走到数据单元。
//! 只通过 [`PoolReader`] 读盘，不沿进程内的指针热读；崩溃点重放（步 7）拿同一条路径去判每一个崩溃状态。
//!
//! 口径：择超级块 = 校验和过且世代号大的那个槽、各盘 fsid 与设备数要对得上（D22（单元原子性怎么合成） 已定项 16）；
//! 择根 = 三个区域全部槽里自证过、`(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7）；
//! journal 全环扫描、不先信 tail（D23（journal 的角色与格式） 已定项 3），两份镜像任一份自证过即算在（D23（journal 的角色与格式） 已定项 14）；
//! 前缀 = `(实例代号, checkpoint_txg)` 严格大于所选根、jsn 严格连续、提交标记齐全、在飞上限之内、点名单元逐项验过，
//! 施加一条记录 = 把所选根的四个字段换成记录新根段里的（D23（journal 的角色与格式） 已定项 15）。

use std::collections::BTreeMap;

use singlefs_format::{
    journal_in_flight_record_limit, DATA_UNIT_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
    INODE_RECORD_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, NODE_BYTES,
    ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION, SLOT_BYTES, SUPERBLOCK_SLOT_BYTES,
    TREE_IDENTIFIER_CENTRAL_MAPPING,
};

use crate::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use crate::allocator::AllocationRecord;
use crate::block_device::BlockDevice;
use crate::checksum::crc32_castagnoli;
use crate::journal::JournalRecord;
use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
use crate::pointer::{DataPointer, LocationEntry, NodePointer};
use crate::records::{
    mapping_key_for_data, parse_extent_record, parse_inode_internal_entry, parse_mapping_entry,
    AccountingEntry, InodeRecord, TreeTableEntry, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION,
    TREE_KIND_DEADLIST, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
};
use crate::root_record::RootRecord;
use crate::root_ring::{slot_offset, RootRingSlot};
use crate::superblock::{FormatTimeGeometry, Superblock};
use crate::transaction::FIRST_INODE_NUMBER;
use crate::unit::{
    data_unit_payload, parse_data_unit, parse_index_node, parse_packed_unit,
    unit_filesystem_identifier, IndexNodeHeader, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE,
    UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};

/// 恢复路径读盘的口子。两个实现：文件后端的池（`Vec<(DeviceIdentity, Device)>`）与层 0 枚举出的崩溃镜像（harness）。
pub trait PoolReader {
    fn device_identities(&self) -> Vec<DeviceIdentity>;
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
    /// 某块盘两个超级块槽都无效。
    NoValidSuperblock { device: DeviceIdentity },
    /// 各盘的超级块 fsid 或设备数对不上。
    SuperblocksDisagree,
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
    pub verification_passed: usize,
    pub verification_failed: usize,
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
fn read_unit_via_locations(
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
pub fn choose_superblock(reader: &dyn PoolReader) -> Result<Superblock, RecoveryFailure> {
    let slot_bytes = usize::try_from(SUPERBLOCK_SLOT_BYTES).expect("4096");
    let mut chosen: Option<Superblock> = None;
    for device in reader.device_identities() {
        let slot_zero = reader
            .read(device, DeviceOffsetInBytes(0), slot_bytes)
            .and_then(|bytes| Superblock::parse_slot(&bytes));
        let spacing = slot_zero
            .as_ref()
            .map_or(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, |superblock| {
                u64::from(superblock.geometry.fixed_structure_slot_spacing)
            });
        let slot_one = reader
            .read(device, DeviceOffsetInBytes(spacing), slot_bytes)
            .and_then(|bytes| Superblock::parse_slot(&bytes));
        let best_on_device = match (slot_zero, slot_one) {
            (None, None) => return Err(RecoveryFailure::NoValidSuperblock { device }),
            (Some(only), None) | (None, Some(only)) => only,
            (Some(zero), Some(one)) => {
                if one.slot_generation > zero.slot_generation {
                    one
                } else {
                    zero
                }
            }
        };
        match &chosen {
            None => chosen = Some(best_on_device),
            Some(previous) => {
                if previous.filesystem_identifier != best_on_device.filesystem_identifier
                    || previous.device_count != best_on_device.device_count
                {
                    return Err(RecoveryFailure::SuperblocksDisagree);
                }
            }
        }
    }
    chosen.ok_or(RecoveryFailure::SuperblocksDisagree)
}

/// 根环三个区域全部槽里自证过的根，逐个交给 `visit`（择根与取号共用这一段遍历）。
fn visit_valid_roots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    geometry: &FormatTimeGeometry,
    filesystem_identifier: &[u8; 16],
    mut visit: impl FnMut(RootRecord),
) {
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    for region in 0..ROOT_RING_REGIONS {
        let device = region_devices[usize::try_from(region).expect("区域号")];
        for slot in 0..ROOT_RING_SLOTS_PER_REGION {
            let offset = slot_offset(
                RootRingSlot { region, slot },
                geometry.fixed_structure_slot_spacing,
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
pub fn choose_root(reader: &dyn PoolReader, superblock: &Superblock) -> Option<RootRecord> {
    let mut best: Option<RootRecord> = None;
    visit_valid_roots(
        reader,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
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

/// 根环全部自证过的根里最大的实例代号（取号的 max 里「根环里全部根记录的实例代号」那一半）；一条都没有时 None。
#[must_use]
pub fn highest_root_instance<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    geometry: &FormatTimeGeometry,
    filesystem_identifier: &[u8; 16],
) -> Option<InstanceGeneration> {
    let mut highest: Option<InstanceGeneration> = None;
    visit_valid_roots(
        reader,
        region_devices,
        geometry,
        filesystem_identifier,
        |root| {
            highest = Some(highest.map_or(root.instance, |current| current.max(root.instance)));
        },
    );
    highest
}

/// 一块盘两槽里自证过、fsid 与本池相同的超级块。取号的 max（D18（块里携带什么信息） 已定项 11）与超级块槽写的世代号
/// （D22（单元原子性怎么合成） 已定项 16，逐盘计）都按这个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）。
#[must_use]
pub fn verified_superblock_slots<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    slot_spacing_in_bytes: u64,
    filesystem_identifier: &[u8; 16],
) -> Vec<Superblock> {
    let slot_bytes = usize::try_from(SUPERBLOCK_SLOT_BYTES).expect("4096");
    [0, slot_spacing_in_bytes]
        .into_iter()
        .filter_map(|offset| reader.read(device, DeviceOffsetInBytes(offset), slot_bytes))
        .filter_map(|bytes| Superblock::parse_slot(&bytes))
        .filter(|superblock| superblock.filesystem_identifier == *filesystem_identifier)
        .collect()
}

/// 全环扫描：每盘每个记录槽都读（有提示时只读提示的那些），两份镜像任一份自证过即算在；fsid 不符的不算数。
#[must_use]
pub fn scan_journal(
    reader: &dyn PoolReader,
    superblock: &Superblock,
) -> BTreeMap<(InstanceGeneration, u64), JournalRecord> {
    let expected_filesystem_identifier =
        unit_filesystem_identifier(&superblock.filesystem_identifier);
    let ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    let ring_bytes = superblock.geometry.journal_ring_bytes;
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
) -> (JournalScanReport, RootRecord) {
    let mut report = JournalScanReport {
        valid_records: records.len(),
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
    };
    let water = (root.instance, root.checkpoint_txg);
    let mut rebuilt = *root;
    let mut above: Vec<&JournalRecord> = records
        .values()
        .filter(|record| (record.instance, record.checkpoint_txg) > water)
        .collect();
    above.sort_by_key(|record| (record.instance, record.counter));
    report.above_water = above.len();
    let in_flight_limit =
        usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
    let mut expected_next: Option<(InstanceGeneration, u64)> = None;
    for record in above.into_iter().take(in_flight_limit) {
        if let Some(expected_key) = expected_next {
            if (record.instance, record.counter) != expected_key {
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
        report.verification_passed += 1;
        report.prefix_applied += 1;
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

fn read_tree_root(
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
    let device_count = reader.device_identities().len();
    if roots.allocation.entries.len() != 10 * device_count {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "分配记录数不是 10 × 盘数",
        });
    }
    if roots.accounting.entries.len() != 3 + 6 * device_count {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "记账条目数不是 3 + 6 × 盘数",
        });
    }
    if roots.mapping.entries.len() != 6 {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: "映射条目数不是 6",
        });
    }
    for record_bytes in &roots.allocation.entries {
        let record = AllocationRecord::parse(record_bytes);
        if record.span_slots & 0x8000 != 0
            || record.generation > root.checkpoint_txg
            || record.span_slots == 0
        {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录带已释放标志、跨度为 0 或分配代晚于根",
            });
        }
    }
    for entry_bytes in &roots.accounting.entries {
        let entry = AccountingEntry::parse(entry_bytes);
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
        let (separator_key, identity, child) = parse_inode_internal_entry(entry);
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
        let (key, pointer) = parse_extent_record(record_bytes);
        if key != wanted_key {
            continue;
        }
        let bytes = match read_unit_via_locations(reader, &pointer.locations, data_unit_bytes) {
            Ok(bytes) => bytes,
            Err(_hint_error) => {
                *mapping_fallbacks += 1;
                let mapping_key = mapping_key_for_data(pointer.head, pointer.write_order);
                let mapped = roots
                    .mapping
                    .entries
                    .iter()
                    .map(|entry| parse_mapping_entry(entry))
                    .find(|(candidate_key, _)| *candidate_key == mapping_key);
                let Some((_, locations)) = mapped else {
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
    let superblock = match choose_superblock(reader) {
        Ok(superblock) => superblock,
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
    let Some(root) = choose_root(reader, &superblock) else {
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
            let records = scan_journal(reader, &superblock);
            replay_journal(
                reader,
                &root,
                superblock.geometry.journal_ring_bytes,
                &records,
                policy == JournalPolicy::Consult,
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
