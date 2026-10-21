//! 区域扫描（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G9）：对一个整理区域 R 按槽升序看每条未释放的分配记录，
//! 读单元头反查映射 key、查中央映射核位置，把它分成可搬、钉住、隔离、已死、不一致五类；已释放或没有记录的槽跳过。
//! 「已死」第一版只报告不动（里程碑五的打包容器那一支不写，只保证这一类在）。
//!
//! 它在三层里打头（G2：`scan_region` → `crate::compaction_plan::choose_region` → `crate::compaction_plan::plan_batch`）：
//! 读单元的那一步交给调用方给的闭包，这里只经 `crate::unit` 的三个 parse 解头，不发一个写、不动分配器。
//!
//! 第一版两盘同槽（D2（RAID 条带策略） 已定项 10），每个落点逐盘一条记录、两盘对称（D3（空间分配） 已定项 7；释放前核出对不上时
//! 两盘一起留在已分配，D19（块指针的结构与宽度预算） 已定项 5）：分类按 R 那块盘上的记录做，钉住与隔离按槽号登记
//! （R 罩住每块盘上的同号槽，判决 G1 攻方 K-n）。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};

use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use crate::allocator::{AllocationRecord, Placement};
use crate::logged_operations::CompactionRegion;
use crate::pointer::{LocationEntry, NodePointer, PointerHead};
use crate::records::{mapping_key_for_data, mapping_key_for_node};
use crate::unit::{
    parse_data_unit, parse_index_node, parse_packed_unit, UnitError, UNIT_CLASS_DATA,
    UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};

/// 中央映射里的一条：27 字节 key 加两条位置条目（D19（块指针的结构与宽度预算） 已定项 10）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CentralMappingEntry {
    pub key: Vec<u8>,
    pub locations: [LocationEntry; 2],
}

impl CentralMappingEntry {
    /// 这条映射的位置条目里有没有一条指着 `device` 上的 `slot`。
    #[must_use]
    pub fn points_at(&self, device: DeviceIdentity, slot: SlotNumber) -> bool {
        self.locations
            .iter()
            .any(|location| location.device == device && location.slot == slot)
    }
}

/// 进映射的三类单元（D19（块指针的结构与宽度预算） 已定项 8）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappedUnitClass {
    /// 码 1 数据单元。
    Data,
    /// 码 2 索引节点。
    IndexNode,
    /// 码 3 打包记录单元。
    Packed,
}

impl MappedUnitClass {
    /// 单元类型标签，也是映射 key 的第一个字节。
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            MappedUnitClass::Data => UNIT_CLASS_DATA,
            MappedUnitClass::IndexNode => UNIT_CLASS_INDEX_NODE,
            MappedUnitClass::Packed => UNIT_CLASS_PACKED,
        }
    }
}

/// 一个可搬的单元：头里反查出的映射 key 在映射里、映射的位置就是这个槽（G9「可搬」）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovableUnit {
    pub mapping_key: Vec<u8>,
    pub unit_class: MappedUnitClass,
    /// 单元头里的诞生代号（搬迁不改它，D26（后台整理与放置回收） 已定项 5 第 1 条）。
    pub birth_txg: CheckpointTxg,
}

/// 单元头为什么读不出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitHeaderUnreadable {
    /// 读单元的闭包交回读不到。
    DeviceReadFailed,
    /// 记录的跨度不是 1（码 2）也不是 2（码 1 / 码 3）：三个 parse 都不收这样长的单元，不去读。
    SpanIsNotAUnitFootprint { span_slots: u16 },
    /// 读到了，parse 拒收（校验和、类标签、结构字段）。
    RefusedByTheParser(UnitError),
}

/// 两个来源（单元头、中央映射）对不上的三种（G9「不一致」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionScanInconsistency {
    HeaderUnreadable(UnitHeaderUnreadable),
    /// 指着这个槽的映射条目的 key 打头的类标签与单元头的类不符。
    UnitClassDisagreesWithTheMappingKey {
        header_unit_class: MappedUnitClass,
        mapping_key_class_tag: u8,
    },
    /// 头里反查出的 key 在映射里的位置不是这个槽，或指着这个槽的是别的 key。
    HeaderKeyDisagreesWithTheMappingLocation,
}

/// 一条未释放的分配记录分进哪一类。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllocationInRegion {
    Movable(MovableUnit),
    /// 罩住的槽里有钉住的（豁免三类或被根直接指着）：不搬。
    Pinned,
    /// 罩住的槽里有在隔离集里的（释放前核出对不上，D19 已定项 5 硬规则 1），不论映射里有没有条目指着它：搬不动，
    /// 单列给丢意图那一步（G2 攻方 K-c）。
    Quarantined,
    /// 映射里没有条目指着它、头里的 key 映射里也没有、不在隔离集里：第一版只报告。
    Dead,
    Inconsistent(RegionScanInconsistency),
}

/// R 里一条未释放的分配记录与它的分类。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScannedAllocation {
    /// 记录的落点（两盘同槽）。
    pub placement: Placement,
    pub class: AllocationInRegion,
}

/// 一次区域扫描的结果：R 里每条未释放的记录一项，按槽升序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionScan {
    pub region: CompactionRegion,
    pub allocations: Vec<ScannedAllocation>,
}

impl RegionScan {
    /// 可搬的那几项，按槽升序：落点与单元。
    pub fn movable_units(&self) -> impl Iterator<Item = (Placement, &MovableUnit)> {
        self.allocations
            .iter()
            .filter_map(|allocation| match &allocation.class {
                AllocationInRegion::Movable(unit) => Some((allocation.placement, unit)),
                AllocationInRegion::Pinned
                | AllocationInRegion::Quarantined
                | AllocationInRegion::Dead
                | AllocationInRegion::Inconsistent(_) => None,
            })
    }

    /// 按槽升序头一对判成可搬、映射 key 相同的记录的起点（先遇到的在前）；可搬的 key 两两不同时交 `None`。
    #[must_use]
    pub fn first_two_movable_units_sharing_a_mapping_key(&self) -> Option<[SlotNumber; 2]> {
        let mut slot_of_each_movable_key: BTreeMap<&[u8], SlotNumber> = BTreeMap::new();
        for (placement, unit) in self.movable_units() {
            if let Some(earlier_slot) =
                slot_of_each_movable_key.insert(unit.mapping_key.as_slice(), placement.slot)
            {
                return Some([earlier_slot, placement.slot]);
            }
        }
        None
    }

    /// 能搬的那几项映射 key 两两不同（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7 的 P2「key 互异」的前提，
    /// `singlefs_replay_prefix::compaction_plan_proofs::movable_mapping_keys_are_distinct`）。[`scan_region`] 交回 `Ok` 的扫描结果上恒真。
    #[must_use]
    pub fn movable_mapping_keys_are_pairwise_distinct(&self) -> bool {
        self.first_two_movable_units_sharing_a_mapping_key()
            .is_none()
    }
}

/// 扫不了这个 R 的情形。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionScanRefusal {
    /// R 那块盘上有一条未释放的记录跨着 R 的边界（起点在 R 外而罩进 R，或起点在 R 里而罩出 R）。
    /// 判决 G9 没定这样的记录归哪一类；分配器自己发的落点不跨段（单元区起点在段边界上、两槽单元起点偶数），
    /// 只有盘上读来的坏记录才有（`recovery::allocation_records_fit_the_pool_geometry` 不判跨度与单元类别，C411）：第一版不支持。
    RecordStraddlesTheRegionBoundaryUnsupported {
        device: DeviceIdentity,
        slot: SlotNumber,
        span_slots: u16,
    },
    /// R 里两条起点不同的记录都判成可搬、单元头反查出的映射 key 相同（`slots` 是头一对的起点，按槽升序）。判「可搬」要指着这一槽的映射条目
    /// 都带头里那把 key，而一把 key 在映射里只有一条条目：走到这里，那条条目在 R 那块盘上指着两个槽——两条位置项落在同一块盘上，
    /// D19（块指针的结构与宽度预算） 已定项 5（用户 2026-09-25 定）把它当映射条目损坏、在任何写之前拒绝（发布路径的同一判是
    /// `transaction::refuse_mapping_entries_that_do_not_name_two_pool_devices`）。读盘重建时只有一部分映射条目经它覆盖的指针按 I-2.5 核过
    /// 位置项设备升序（`recovery::pointers_of_the_rebuilt_version_ascend_by_device` 列的那几类），留存给快照的那几条
    /// （`mapping_entries_kept_for_the_snapshots`）不经任何指针、没有经它核，盘上这样一条条目走得到这里（推的，没造过这样的盘）。
    /// 照那一句拒这个 R，一条都不交：不拒的话同一把 key 进同一批两次，判决 G7 的 P2「一批里的 key 互异」不成立。
    TwoRecordsMovableUnderOneMappingKey {
        device: DeviceIdentity,
        slots: [SlotNumber; 2],
    },
}

/// 单元头里反查出来的东西。
struct UnitHeaderIdentity {
    unit_class: MappedUnitClass,
    mapping_key: Vec<u8>,
    birth_txg: CheckpointTxg,
}

/// 扫一个 R：`allocation_records` 里 R 那块盘上起点在 R 里、未释放的每条记录分一类，按槽升序交回。
/// `read_unit_bytes(盘, 槽, 字节数)` 读那个落点上的单元，读不到交 `None`。
///
/// 交回 `Ok` 时能搬的那几项映射 key 两两不同（[`RegionScan::movable_mapping_keys_are_pairwise_distinct`]）。
///
/// # Errors
/// R 那块盘上有未释放的记录跨着 R 的边界 ⇒ [`RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported`]，一条都不分类；
/// 两条记录判成可搬而映射 key 相同 ⇒ [`RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey`]，一条都不交。
pub fn scan_region<ReadUnitBytes>(
    region: CompactionRegion,
    allocation_records: &[AllocationRecord],
    mapping_entries: &[CentralMappingEntry],
    pinned_slots: &BTreeSet<SlotNumber>,
    quarantined_slots: &BTreeSet<SlotNumber>,
    mut read_unit_bytes: ReadUnitBytes,
) -> Result<RegionScan, RegionScanRefusal>
where
    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
{
    let device = region.device();
    let mut records_in_region: Vec<&AllocationRecord> = Vec::new();
    for record in allocation_records
        .iter()
        .filter(|record| record.device == device && !record.is_released)
    {
        let record_end = record.slot.0 + u64::from(record.span_slots);
        let starts_inside = region.contains_slot(record.slot);
        let reaches_in_from_below =
            record.slot.0 < region.start_slot().0 && record_end > region.start_slot().0;
        let reaches_out_above = starts_inside && record_end > region.end_slot().0;
        if reaches_in_from_below || reaches_out_above {
            return Err(
                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
                    device,
                    slot: record.slot,
                    span_slots: record.span_slots,
                },
            );
        }
        if starts_inside {
            records_in_region.push(record);
        }
    }
    records_in_region.sort_by_key(|record| record.slot);
    let allocations = records_in_region
        .into_iter()
        .map(|record| ScannedAllocation {
            placement: Placement {
                slot: record.slot,
                span: u64::from(record.span_slots),
            },
            class: classify_allocation(
                record,
                mapping_entries,
                pinned_slots,
                quarantined_slots,
                &mut read_unit_bytes,
            ),
        })
        .collect();
    let scan = RegionScan {
        region,
        allocations,
    };
    if let Some(slots) = scan.first_two_movable_units_sharing_a_mapping_key() {
        return Err(RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey { device, slots });
    }
    Ok(scan)
}

/// 一条未释放记录的分类，次序：钉住 → 隔离 → 读头 → 类标签 → key 与位置（判决 m4-batch2-code-r1 S3）。
/// 钉住与隔离都看记录罩住的每一个槽，不只看起点槽；隔离不看映射有没有条目指着它。
fn classify_allocation<ReadUnitBytes>(
    record: &AllocationRecord,
    mapping_entries: &[CentralMappingEntry],
    pinned_slots: &BTreeSet<SlotNumber>,
    quarantined_slots: &BTreeSet<SlotNumber>,
    read_unit_bytes: &mut ReadUnitBytes,
) -> AllocationInRegion
where
    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
{
    let covered_slots = record.slot..SlotNumber(record.slot.0 + u64::from(record.span_slots));
    if pinned_slots.range(covered_slots.clone()).next().is_some() {
        return AllocationInRegion::Pinned;
    }
    // 隔离不看映射：隔离 = 释放前核出校验和对不上，映射照旧指着它时搬它就是把坏副本搬走（主 agent 2026-10-13 定，判决 m4-batch2-code-r1 S3）。
    if quarantined_slots.range(covered_slots).next().is_some() {
        return AllocationInRegion::Quarantined;
    }
    let entries_pointing_here: Vec<&CentralMappingEntry> = mapping_entries
        .iter()
        .filter(|entry| entry.points_at(record.device, record.slot))
        .collect();
    let header = match read_unit_header(record, read_unit_bytes) {
        Ok(header) => header,
        Err(unreadable) => {
            return AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
                unreadable,
            ))
        }
    };
    if let Some(entry) = entries_pointing_here
        .iter()
        .find(|entry| entry.key.first() != Some(&header.unit_class.tag()))
    {
        return AllocationInRegion::Inconsistent(
            RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
                header_unit_class: header.unit_class,
                mapping_key_class_tag: entry.key.first().copied().unwrap_or_default(),
            },
        );
    }
    let header_key_is_mapped = mapping_entries
        .iter()
        .any(|entry| entry.key == header.mapping_key);
    if entries_pointing_here.is_empty() && !header_key_is_mapped {
        return AllocationInRegion::Dead;
    }
    let every_entry_pointing_here_carries_the_header_key = entries_pointing_here
        .iter()
        .all(|entry| entry.key == header.mapping_key);
    if !entries_pointing_here.is_empty() && every_entry_pointing_here_carries_the_header_key {
        return AllocationInRegion::Movable(MovableUnit {
            mapping_key: header.mapping_key,
            unit_class: header.unit_class,
            birth_txg: header.birth_txg,
        });
    }
    AllocationInRegion::Inconsistent(
        RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
    )
}

/// 读一条记录落点上的单元、解头、反查映射 key：跨度 1 当码 2 读 16 KiB，跨度 2 读 32 KiB 先当码 1、类标签是 3 再当码 3。
/// key 的各段照写者的取法（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 8）：码 1 = 树 ID + 诞生代号 + 写序，
/// 码 2 = 树 ID + 诞生代号 + 实例代号 + 出生序号，码 3 = 出生树 + 诞生代号 + 写序里的实例代号 + 出生序号。
fn read_unit_header<ReadUnitBytes>(
    record: &AllocationRecord,
    read_unit_bytes: &mut ReadUnitBytes,
) -> Result<UnitHeaderIdentity, UnitHeaderUnreadable>
where
    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
{
    let node_pointer_of = |birth_tree, birth_txg, instance, birth_sequence| NodePointer {
        head: PointerHead {
            birth_tree,
            birth_txg,
        },
        locations: NodePointer::empty_root().locations,
        instance,
        birth_sequence,
    };
    match record.span_slots {
        1 => {
            let bytes = read_unit_bytes(
                record.device,
                record.slot,
                usize::try_from(NODE_BYTES).expect("16384"),
            )
            .ok_or(UnitHeaderUnreadable::DeviceReadFailed)?;
            let header =
                parse_index_node(&bytes).map_err(UnitHeaderUnreadable::RefusedByTheParser)?;
            Ok(UnitHeaderIdentity {
                unit_class: MappedUnitClass::IndexNode,
                mapping_key: mapping_key_for_node(
                    UNIT_CLASS_INDEX_NODE,
                    node_pointer_of(
                        header.tree,
                        header.birth_txg,
                        header.instance,
                        header.birth_sequence,
                    ),
                ),
                birth_txg: header.birth_txg,
            })
        }
        2 => {
            let bytes = read_unit_bytes(
                record.device,
                record.slot,
                usize::try_from(DATA_UNIT_BYTES).expect("32768"),
            )
            .ok_or(UnitHeaderUnreadable::DeviceReadFailed)?;
            match parse_data_unit(&bytes) {
                Ok(header) => Ok(UnitHeaderIdentity {
                    unit_class: MappedUnitClass::Data,
                    mapping_key: mapping_key_for_data(
                        PointerHead {
                            birth_tree: header.identity.tree,
                            birth_txg: header.birth_txg,
                        },
                        header.write_order,
                    ),
                    birth_txg: header.birth_txg,
                }),
                Err(UnitError::WrongUnitClass {
                    found: UNIT_CLASS_PACKED,
                    ..
                }) => {
                    let header = parse_packed_unit(&bytes)
                        .map_err(UnitHeaderUnreadable::RefusedByTheParser)?;
                    Ok(UnitHeaderIdentity {
                        unit_class: MappedUnitClass::Packed,
                        mapping_key: mapping_key_for_node(
                            UNIT_CLASS_PACKED,
                            node_pointer_of(
                                header.identity.birth_tree,
                                header.birth_txg,
                                header.write_order.instance,
                                header.birth_sequence,
                            ),
                        ),
                        birth_txg: header.birth_txg,
                    })
                }
                Err(refusal) => Err(UnitHeaderUnreadable::RefusedByTheParser(refusal)),
            }
        }
        span_slots => Err(UnitHeaderUnreadable::SpanIsNotAUnitFootprint { span_slots }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use singlefs_format::{TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, UNIT_AREA_START_SLOT};

    use crate::address::{InstanceGeneration, TreeIdentifier};
    use crate::pointer::BirthSequence;
    use crate::unit::{
        build_data_unit, build_index_node, build_packed_unit, DataUnitIdentity, PackedIdentity,
        WriteOrder, PACKED_TYPE_INODE,
    };

    const FILESYSTEM_IDENTIFIER: [u8; 16] = [7u8; 16];
    const BIRTH_TXG: CheckpointTxg = CheckpointTxg(7);
    const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
    const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
    const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);

    /// 单元区里起点 50240 的那个聚簇段 [50240, 50304)。
    fn region_under_test() -> CompactionRegion {
        CompactionRegion::of_one_cluster_segment(DEVICE_ZERO, SlotNumber(UNIT_AREA_START_SLOT + 64))
            .expect("50240 = 785 × 64")
    }

    fn slot_in_region(offset_in_slots: u64) -> SlotNumber {
        SlotNumber(region_under_test().start_slot().0 + offset_in_slots)
    }

    /// 一块镜像：两块盘上每个落点各一份字节、各一条记录（两盘同槽），加中央映射的条目。
    #[derive(Default)]
    struct PoolUnderTest {
        units: BTreeMap<(DeviceIdentity, SlotNumber), Vec<u8>>,
        records: Vec<AllocationRecord>,
        mapping_entries: Vec<CentralMappingEntry>,
    }

    impl PoolUnderTest {
        fn place(&mut self, slot: SlotNumber, span_slots: u16, unit: Option<&[u8]>) {
            for device in [DEVICE_ZERO, DEVICE_ONE] {
                self.records.push(AllocationRecord {
                    device,
                    slot,
                    span_slots,
                    generation: BIRTH_TXG,
                    is_released: false,
                });
                if let Some(bytes) = unit {
                    self.units.insert((device, slot), bytes.to_vec());
                }
            }
        }

        fn release(&mut self, slot: SlotNumber) {
            for record in self.records.iter_mut().filter(|record| record.slot == slot) {
                record.is_released = true;
            }
        }

        fn map(&mut self, key: Vec<u8>, slot: SlotNumber) {
            self.mapping_entries.push(CentralMappingEntry {
                key,
                locations: locations_at(slot),
            });
        }

        fn scan(
            &self,
            pinned_slots: &BTreeSet<SlotNumber>,
            quarantined_slots: &BTreeSet<SlotNumber>,
        ) -> Result<RegionScan, RegionScanRefusal> {
            scan_region(
                region_under_test(),
                &self.records,
                &self.mapping_entries,
                pinned_slots,
                quarantined_slots,
                |device, slot, unit_bytes| {
                    self.units
                        .get(&(device, slot))
                        .filter(|bytes| bytes.len() == unit_bytes)
                        .cloned()
                },
            )
        }
    }

    fn locations_at(slot: SlotNumber) -> [LocationEntry; 2] {
        [DEVICE_ZERO, DEVICE_ONE].map(|device| LocationEntry {
            device,
            slot,
            unit_checksum: 0,
        })
    }

    /// 一个码 1 数据单元与写者给它的映射 key（`transaction` 里 `mapping_key_for_data` 那一处的取法：出生树 = extent 树、
    /// 出生 txg = 这次发布、写序 = 实例代号与事务号）。
    fn data_unit(transaction: u64) -> (Vec<u8>, Vec<u8>) {
        let write_order = WriteOrder {
            instance: WRITING_INSTANCE,
            transaction,
        };
        let bytes = build_data_unit(
            DataUnitIdentity {
                tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                object: 1,
                object_birth: CheckpointTxg(3),
                anchor_offset: 0,
            },
            BIRTH_TXG,
            &FILESYSTEM_IDENTIFIER,
            write_order,
            &[0x5A; 100],
        );
        let key = mapping_key_for_data(
            PointerHead {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
                birth_txg: BIRTH_TXG,
            },
            write_order,
        );
        (bytes, key)
    }

    fn node_pointer(tree: u64, birth_sequence: u32) -> NodePointer {
        NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(tree),
                birth_txg: BIRTH_TXG,
            },
            locations: NodePointer::empty_root().locations,
            instance: WRITING_INSTANCE,
            birth_sequence: BirthSequence(birth_sequence),
        }
    }

    /// 一个码 2 节点与写者给它的映射 key（类标签 2、出生树 = 节点的树、出生 txg、实例代号、出生序号）。
    fn index_node(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
        let bytes = build_index_node(
            TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            0,
            24,
            &[0u8; 24],
            &[0u8; 24],
            BIRTH_TXG,
            &FILESYSTEM_IDENTIFIER,
            WRITING_INSTANCE,
            BirthSequence(birth_sequence),
            112,
            &[],
        );
        let key = mapping_key_for_node(
            UNIT_CLASS_INDEX_NODE,
            node_pointer(TREE_IDENTIFIER_EXTENT, birth_sequence),
        );
        (bytes, key)
    }

    /// 一个码 3 inode 叶容器与写者给它的映射 key（类标签 3、出生树 = inode 树、出生 txg、实例代号、出生序号）。
    fn packed_unit(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
        let bytes = build_packed_unit(
            PackedIdentity {
                birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
                record_type: PACKED_TYPE_INODE,
                container: 1,
                container_birth: BIRTH_TXG,
            },
            140,
            &[vec![1u8; 140]],
            BIRTH_TXG,
            &FILESYSTEM_IDENTIFIER,
            WriteOrder {
                instance: WRITING_INSTANCE,
                transaction: 9,
            },
            BirthSequence(birth_sequence),
        );
        let key = mapping_key_for_node(
            UNIT_CLASS_PACKED,
            node_pointer(TREE_IDENTIFIER_INODE, birth_sequence),
        );
        (bytes, key)
    }

    fn scanned(offset_in_slots: u64, span: u64, class: AllocationInRegion) -> ScannedAllocation {
        ScannedAllocation {
            placement: Placement {
                slot: slot_in_region(offset_in_slots),
                span,
            },
            class,
        }
    }

    fn inconsistent_by_key() -> AllocationInRegion {
        AllocationInRegion::Inconsistent(
            RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
        )
    }

    /// 判决 G9 的五类在一个 R 里各放至少一个，逐项钉分类、落点与可搬单元的 key / 类 / 诞生代；已释放的记录、R 外的记录、
    /// 另一块盘上的镜像记录都不出现。
    #[test]
    fn every_class_of_the_region_scan_is_pinned_down_slot_by_slot() {
        let mut pool = PoolUnderTest::default();
        let (movable_data, movable_data_key) = data_unit(1);
        pool.place(slot_in_region(0), 2, Some(&movable_data));
        pool.map(movable_data_key.clone(), slot_in_region(0));
        let (pinned_node, _pinned_node_key) = index_node(0);
        pool.place(slot_in_region(2), 1, Some(&pinned_node));
        let (movable_node, movable_node_key) = index_node(1);
        pool.place(slot_in_region(3), 1, Some(&movable_node));
        pool.map(movable_node_key.clone(), slot_in_region(3));
        let (movable_packed, movable_packed_key) = packed_unit(0);
        pool.place(slot_in_region(4), 2, Some(&movable_packed));
        pool.map(movable_packed_key.clone(), slot_in_region(4));
        let (quarantined_data, _quarantined_key) = data_unit(2);
        pool.place(slot_in_region(6), 2, Some(&quarantined_data));
        let (dead_data, _dead_key) = data_unit(3);
        pool.place(slot_in_region(8), 2, Some(&dead_data));
        let (mismatched_data, _mismatched_key) = data_unit(4);
        let (_other_data, other_key) = data_unit(5);
        pool.place(slot_in_region(10), 2, Some(&mismatched_data));
        pool.map(other_key, slot_in_region(10));
        let (released_data, released_key) = data_unit(6);
        pool.place(slot_in_region(12), 2, Some(&released_data));
        pool.map(released_key, slot_in_region(12));
        pool.release(slot_in_region(12));
        let (outside_data, outside_key) = data_unit(7);
        pool.place(slot_in_region(64), 2, Some(&outside_data));
        pool.map(outside_key, slot_in_region(64));

        let scan = pool
            .scan(
                &BTreeSet::from([slot_in_region(2)]),
                &BTreeSet::from([slot_in_region(6)]),
            )
            .expect("没有跨 R 边界的记录");

        assert_eq!(scan.region, region_under_test());
        assert_eq!(
            scan.allocations,
            vec![
                scanned(
                    0,
                    2,
                    AllocationInRegion::Movable(MovableUnit {
                        mapping_key: movable_data_key,
                        unit_class: MappedUnitClass::Data,
                        birth_txg: BIRTH_TXG,
                    })
                ),
                scanned(2, 1, AllocationInRegion::Pinned),
                scanned(
                    3,
                    1,
                    AllocationInRegion::Movable(MovableUnit {
                        mapping_key: movable_node_key,
                        unit_class: MappedUnitClass::IndexNode,
                        birth_txg: BIRTH_TXG,
                    })
                ),
                scanned(
                    4,
                    2,
                    AllocationInRegion::Movable(MovableUnit {
                        mapping_key: movable_packed_key,
                        unit_class: MappedUnitClass::Packed,
                        birth_txg: BIRTH_TXG,
                    })
                ),
                scanned(6, 2, AllocationInRegion::Quarantined),
                scanned(8, 2, AllocationInRegion::Dead),
                scanned(10, 2, inconsistent_by_key()),
            ],
            "槽 +0 / +3 / +4 可搬（码 1 / 码 2 / 码 3），+2 钉住，+6 隔离，+8 已死，+10 头的 key 与指着它的映射条目不同；\
             +12 已释放、+64 在 R 外，都不出现"
        );
        assert_eq!(scan.movable_units().count(), 3, "可搬三个");
    }

    /// 映射里这个 key 的位置挪到了 R 里另一个槽，原来那个槽上的单元头照旧：没有条目指着它，但头里的 key 在映射里，
    /// 两个来源对不上，判「不一致」，不判「已死」（G9「头里的 key 与映射里的位置对不上」）。
    #[test]
    fn a_unit_whose_mapping_entry_points_at_another_slot_is_inconsistent_not_dead() {
        let mut pool = PoolUnderTest::default();
        let (data, key) = data_unit(1);
        pool.place(slot_in_region(0), 2, Some(&data));
        pool.map(key, slot_in_region(20));

        let scan = pool
            .scan(&BTreeSet::new(), &BTreeSet::new())
            .expect("没有跨 R 边界的记录");

        assert_eq!(scan.allocations, vec![scanned(0, 2, inconsistent_by_key())]);
    }

    /// 映射条目指着这个槽，而单元头里反查出的 key 是另一把（头的写序与映射 key 的不同）：判「不一致」，不判可搬。
    #[test]
    fn a_unit_whose_header_key_differs_from_the_mapping_entry_pointing_at_it_is_inconsistent() {
        let mut pool = PoolUnderTest::default();
        let (_mapped_data, mapped_key) = data_unit(1);
        let (data_with_another_write_order, _its_own_key) = data_unit(2);
        pool.place(slot_in_region(0), 2, Some(&data_with_another_write_order));
        pool.map(mapped_key, slot_in_region(0));

        let scan = pool
            .scan(&BTreeSet::new(), &BTreeSet::new())
            .expect("没有跨 R 边界的记录");

        assert_eq!(scan.allocations, vec![scanned(0, 2, inconsistent_by_key())]);
    }

    /// 「不一致」的另外两种：头读不出（读不到、头校验和对不上、跨度不是单元的大小）与类标签和映射 key 不符。
    #[test]
    fn an_unreadable_header_or_a_class_tag_unlike_the_mapping_key_is_inconsistent() {
        let mut pool = PoolUnderTest::default();
        let (_never_written, unwritten_key) = data_unit(1);
        pool.place(slot_in_region(0), 2, None);
        pool.map(unwritten_key, slot_in_region(0));
        let (mut damaged, damaged_key) = data_unit(2);
        damaged[60] ^= 1;
        pool.place(slot_in_region(2), 2, Some(&damaged));
        pool.map(damaged_key, slot_in_region(2));
        let (data, _data_key) = data_unit(3);
        let (_node, node_key) = index_node(0);
        pool.place(slot_in_region(4), 2, Some(&data));
        pool.map(node_key, slot_in_region(4));
        let (_three_slot_unit, three_slot_key) = data_unit(4);
        pool.place(slot_in_region(6), 3, None);
        pool.map(three_slot_key, slot_in_region(6));

        let scan = pool
            .scan(&BTreeSet::new(), &BTreeSet::new())
            .expect("没有跨 R 边界的记录");

        assert_eq!(
            scan.allocations,
            vec![
                scanned(
                    0,
                    2,
                    AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
                        UnitHeaderUnreadable::DeviceReadFailed
                    ))
                ),
                scanned(
                    2,
                    2,
                    AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
                        UnitHeaderUnreadable::RefusedByTheParser(UnitError::HeaderChecksumMismatch)
                    ))
                ),
                scanned(
                    4,
                    2,
                    AllocationInRegion::Inconsistent(
                        RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
                            header_unit_class: MappedUnitClass::Data,
                            mapping_key_class_tag: UNIT_CLASS_INDEX_NODE,
                        }
                    )
                ),
                scanned(
                    6,
                    3,
                    AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
                        UnitHeaderUnreadable::SpanIsNotAUnitFootprint { span_slots: 3 }
                    ))
                ),
            ]
        );
    }

    /// 隔离槽上的单元头完好、映射条目照旧指着它、key 也对得上：照样判「隔离」，不判可搬（判决 m4-batch2-code-r1 S3：
    /// 隔离 = 释放前核出校验和对不上，搬它就是把坏副本搬走；隔离排在读头与核映射之前）。
    #[test]
    fn a_quarantined_slot_the_mapping_still_points_at_is_quarantined_not_movable() {
        let mut pool = PoolUnderTest::default();
        let (data, key) = data_unit(1);
        pool.place(slot_in_region(0), 2, Some(&data));
        pool.map(key, slot_in_region(0));

        let scan = pool
            .scan(&BTreeSet::new(), &BTreeSet::from([slot_in_region(0)]))
            .expect("没有跨 R 边界的记录");

        assert_eq!(
            scan.allocations,
            vec![scanned(0, 2, AllocationInRegion::Quarantined)],
            "映射指着隔离槽也判隔离"
        );
        assert_eq!(scan.movable_units().count(), 0, "隔离槽不进可搬");
    }

    /// 两条映射条目指着同一个槽：一条是单元头里反查出的 key，另一条是别的 key（类标签相同）。要每一条都是头里那把 key 才可搬，
    /// 这里判「不一致」（判决 m4-batch2-code-r1 S3，攻方翻过的 `all` → `any`）。
    #[test]
    fn a_slot_two_mapping_entries_point_at_with_only_one_carrying_the_header_key_is_inconsistent() {
        let mut pool = PoolUnderTest::default();
        let (data, own_key) = data_unit(1);
        let (_other_data, other_key) = data_unit(2);
        pool.place(slot_in_region(0), 2, Some(&data));
        pool.map(own_key, slot_in_region(0));
        pool.map(other_key, slot_in_region(0));

        let scan = pool
            .scan(&BTreeSet::new(), &BTreeSet::new())
            .expect("没有跨 R 边界的记录");

        assert_eq!(
            scan.allocations,
            vec![scanned(0, 2, inconsistent_by_key())],
            "另一把 key 也指着这个槽：两个来源对不上"
        );
    }

    /// 单元头里的 key 在映射里，映射的两条位置条目槽号与这个槽相同、却在另两块盘上（不是 R 那块盘）：没有条目指着 R 那块盘上的这个槽，
    /// 判「不一致」，不判可搬（判决 m4-batch2-code-r1 S3，攻方翻过的「位置条目不比盘」）。
    #[test]
    fn a_mapping_entry_at_the_same_slot_number_on_other_devices_does_not_point_at_the_region_device(
    ) {
        let mut pool = PoolUnderTest::default();
        let (data, key) = data_unit(1);
        pool.place(slot_in_region(0), 2, Some(&data));
        pool.mapping_entries.push(CentralMappingEntry {
            key,
            locations: [DeviceIdentity(2), DeviceIdentity(3)].map(|device| LocationEntry {
                device,
                slot: slot_in_region(0),
                unit_checksum: 0,
            }),
        });

        let scan = pool
            .scan(&BTreeSet::new(), &BTreeSet::new())
            .expect("没有跨 R 边界的记录");

        assert_eq!(
            scan.allocations,
            vec![scanned(0, 2, inconsistent_by_key())],
            "同槽号、别的盘上的位置条目不算指着这里"
        );
    }

    /// 两槽单元的第二个槽钉住（起点槽没钉住）：整条记录判「钉住」，不判可搬（判决 m4-batch2-code-r1 S3，攻方翻过的「钉住只看起点槽」）。
    #[test]
    fn a_two_slot_unit_whose_second_slot_is_pinned_is_pinned() {
        let mut pool = PoolUnderTest::default();
        let (data, key) = data_unit(1);
        pool.place(slot_in_region(0), 2, Some(&data));
        pool.map(key, slot_in_region(0));

        let scan = pool
            .scan(&BTreeSet::from([slot_in_region(1)]), &BTreeSet::new())
            .expect("没有跨 R 边界的记录");

        assert_eq!(
            scan.allocations,
            vec![scanned(0, 2, AllocationInRegion::Pinned)],
            "罩住的第二个槽钉住，整条记录钉住"
        );
    }

    /// R 那块盘上一条未释放的记录跨着 R 的边界（从下面罩进来、从里面罩出去）：判决 G9 没定它归哪一类，整次拒、一条都不分类。
    #[test]
    fn a_record_straddling_the_region_boundary_is_refused_before_any_classification() {
        let (inside, inside_key) = data_unit(1);
        let slot_below_the_region = SlotNumber(region_under_test().start_slot().0 - 1);

        let mut reaching_in = PoolUnderTest::default();
        reaching_in.place(slot_in_region(0), 2, Some(&inside));
        reaching_in.map(inside_key.clone(), slot_in_region(0));
        reaching_in.place(slot_below_the_region, 2, None);
        assert_eq!(
            reaching_in.scan(&BTreeSet::new(), &BTreeSet::new()),
            Err(
                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
                    device: DEVICE_ZERO,
                    slot: slot_below_the_region,
                    span_slots: 2,
                }
            ),
            "起点在 R 之下一槽、跨 2：罩进 R"
        );

        let mut reaching_out = PoolUnderTest::default();
        reaching_out.place(slot_in_region(0), 2, Some(&inside));
        reaching_out.map(inside_key, slot_in_region(0));
        reaching_out.place(slot_in_region(63), 2, None);
        assert_eq!(
            reaching_out.scan(&BTreeSet::new(), &BTreeSet::new()),
            Err(
                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
                    device: DEVICE_ZERO,
                    slot: slot_in_region(63),
                    span_slots: 2,
                }
            ),
            "起点在 R 末槽、跨 2：罩出 R"
        );
    }

    /// 同一个单元的两份副本（搬迁原样拷，头里反查出的 key 相同）落在 R 的两个槽上。映射条目健康（两条位置项分在两块盘、指同一个槽）时
    /// 只有它指着的那一份可搬，另一份不一致；映射条目两条位置项都落在 R 那块盘上、各指一份（D19（块指针的结构与宽度预算） 已定项 5 当它损坏）时
    /// 两份都会判成可搬、key 相同：整次拒、一条都不交（判决 G7 的 P2「一批里的 key 互异」靠它）。
    #[test]
    fn two_records_movable_under_one_mapping_key_refuse_the_region() {
        let (copy_of_the_unit, key) = data_unit(1);

        let mut healthy_mapping = PoolUnderTest::default();
        healthy_mapping.place(slot_in_region(0), 2, Some(&copy_of_the_unit));
        healthy_mapping.place(slot_in_region(2), 2, Some(&copy_of_the_unit));
        healthy_mapping.map(key.clone(), slot_in_region(2));
        let scan = healthy_mapping
            .scan(&BTreeSet::new(), &BTreeSet::new())
            .expect("映射条目健康：一把 key 只判一份可搬");
        assert!(
            scan.movable_mapping_keys_are_pairwise_distinct(),
            "能搬的 key 两两不同"
        );
        assert_eq!(
            scan.allocations
                .iter()
                .map(|allocation| allocation.class.clone())
                .collect::<Vec<_>>(),
            vec![
                inconsistent_by_key(),
                AllocationInRegion::Movable(MovableUnit {
                    mapping_key: key.clone(),
                    unit_class: MappedUnitClass::Data,
                    birth_txg: BIRTH_TXG,
                }),
            ],
            "映射指着的那一份可搬，另一份不一致"
        );

        let mut damaged_mapping = PoolUnderTest::default();
        damaged_mapping.place(slot_in_region(0), 2, Some(&copy_of_the_unit));
        damaged_mapping.place(slot_in_region(2), 2, Some(&copy_of_the_unit));
        damaged_mapping.mapping_entries.push(CentralMappingEntry {
            key,
            locations: [slot_in_region(0), slot_in_region(2)].map(|slot| LocationEntry {
                device: DEVICE_ZERO,
                slot,
                unit_checksum: 0,
            }),
        });
        assert_eq!(
            damaged_mapping.scan(&BTreeSet::new(), &BTreeSet::new()),
            Err(RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey {
                device: DEVICE_ZERO,
                slots: [slot_in_region(0), slot_in_region(2)],
            }),
            "两条位置项都在盘 0、各指一份：两份都会判可搬、key 相同，整次拒"
        );
    }
}
