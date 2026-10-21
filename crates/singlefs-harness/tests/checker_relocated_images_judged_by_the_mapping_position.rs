//! 池级 checker 以映射位置为准（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3、G6 与第七节用户定案「重构」；
//! D19（块指针的结构与宽度预算） 已定项 5「位置条目降为提示，中央映射是唯一入口」、已定项 8「豁免三类之外的位置条目只当提示」）：
//! 走读按映射认单元的位置、先按提示读、两份都对不上再按映射位置读，引用集合记映射位置；「提示 ≠ 映射、提示那一槽已释放」在合法搬迁之后不许红。
//!
//! 合法搬迁过的镜像用字节级改动造（GC 发布入口 `publish_relocation_batch` 落地之前写路径造不出它）：新池新建文件（txg 3）之后建一个空 inode
//! （txg 4：数据单元与 extent 树不动，分配记录叶、记账树、中央映射根照常重写），在最新那一版里把第一个文件唯一的数据单元与 extent 树的根
//! 原样拷到单元区里没人用的槽，映射条目换成新槽、源槽的分配记录记释放（释放代 = 这一版的 txg）、新槽记一条已分配（分配代 = 单元的诞生代号，
//! 判决 G3 的 G-a），记账的已分配、空闲、defer、runs 与全空聚簇段数跟着改；extent 树与树表里的指针一个字节不动（提示过期）。改过的单元重封，新的整单元校验和
//! 沿引用链补到根槽、journal 记录的新根段照抄（`common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum`）。
mod common;
mod common_pointer_head_corruption;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use common_pointer_head_corruption::{
    allocation_record_tree_nodes_as_far_as_readable, newest_root,
    replace_the_unit_and_propagate_its_checksum, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at,
};
use singlefs_checker::image::{
    InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
    UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::ALLOCATION_RECORD_RELEASED_FLAG;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::records::{TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_EXTENT};
use singlefs_core::transaction::{publish_new_inodes, PoolWriter};
use singlefs_core::unit::{build_index_node, parse_index_node, seal_header_checksum};
use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, NODE_BYTES, SLOT_BYTES,
    UNIT_AREA_START_SLOT,
};
use singlefs_harness::memory_pool::MemoryPool;

const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
/// 中央映射叶条目 55 字节：key 27 之后两条位置条目各 14（设备身份 4、槽号 6、整单元校验和 4，D19 已定项 10）。
const MAPPING_LOCATION_ENTRY_BYTES: usize = 14;
/// 映射 key 里出生 txg 那 8 字节的起点：类标签 1 + 出生树 8（D19 已定项 6）。
const MAPPING_KEY_BIRTH_TXG_OFFSET: usize = 1 + 8;
/// 分配记录 20 字节：设备身份 4、起点槽 6、跨度 2（最高位是已释放标志）、分配代或释放代 8（D3（空间分配） 已定项 7）。
const ALLOCATION_RECORD_BYTES: usize = 20;
const ALLOCATION_RECORD_SPAN_OFFSET: usize = 10;
const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;
/// 记账条目 34 字节：key 22（统计量 2 + 树 8 + 设备 4 + 代 8）之后是值 8（D8（核心索引结构） 已定项 11）。
const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 2 + 8;
const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
/// 记账统计量的行号（D5（快照 / 空间记账机制） 已定项 4）：已分配 1、空闲 2、defer 待释放 5，单位字节；runs 10（单元区空闲槽的连续段数）、
/// 全空聚簇段数 11，单位个；都带设备维。
const STATISTIC_ALLOCATED_BYTES: u16 = 1;
const STATISTIC_FREE_BYTES: u16 = 2;
const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
const STATISTIC_FRAGMENTATION_RUNS: u16 = 10;
const STATISTIC_EMPTY_CLUSTER_SEGMENTS: u16 = 11;
/// 目的槽离这一版分配记录里最后一个在册单元的末槽隔开这么多槽：落在没人用过的地方（不碰被更早的根引用过、已回收的槽）。
const DESTINATION_GAP_IN_SLOTS: u64 = 64;

fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

fn mapping_key_bytes() -> usize {
    usize::try_from(MAPPING_KEY_BYTES).expect("27")
}

fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}

fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn read_six_bytes_at(bytes: &[u8], offset: usize) -> u64 {
    let mut wide = [0u8; 8];
    wide[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(wide)
}

/// 码 2 节点改过之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)），k 是住偏移 51 的 key 宽。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 新池新建文件（txg 3）之后建一个空 inode（txg 4）：第一个文件的数据单元与 extent 树留在 txg 3 写的槽里，被 txg 3 与 txg 4 两条根共同引用。
fn pool_with_an_empty_inode_after_the_first_file(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        1,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("建一个空 inode");
    pool
}

/// 搬哪一个单元：它在两块盘上共同的源槽、占几槽（码 1 两槽、码 2 一槽）。
#[derive(Clone, Copy, Debug)]
struct UnitToRelocate {
    source_slot: SlotNumber,
    span_in_slots: u64,
}

/// 搬完之后它在哪：目的槽与它的诞生代号（映射 key 里的出生 txg，目的记录的分配代取它）。
#[derive(Clone, Copy, Debug)]
struct RelocatedUnit {
    unit: UnitToRelocate,
    destination_slot: SlotNumber,
    birth_txg: u64,
}

/// 分配记录与记账跟不跟着搬迁改。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocationRecordsOfTheRelocation {
    /// 合法搬迁：源记录记释放、目的记一条已分配，记账的已分配、空闲、defer 都跟着改。
    SourceReleasedAndDestinationAllocated,
    /// 映射与分配记录对不上：源记录记释放、defer 跟着加，目的槽没有记录、已分配与空闲不动。
    SourceReleasedAndDestinationNotRecorded,
}

/// 最新那一版里第一个文件唯一的数据单元的槽与 extent 树的根节点的槽（两盘同槽）。
fn data_unit_and_extent_root_of_the_newest_version(image: &MemoryPool) -> [UnitToRelocate; 2] {
    let extent_root = tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root;
    let mapping = parse_index_node(&unit_bytes_at(
        image,
        the_slot_of(&newest_root(image).mapping_root),
        node_bytes(),
    ))
    .expect("中央映射根解得开");
    let data_units: Vec<SlotNumber> = mapping
        .entries
        .iter()
        .filter(|entry| entry[0] == 1)
        .map(|entry| SlotNumber(read_six_bytes_at(entry, mapping_key_bytes() + 4)))
        .collect();
    assert_eq!(
        data_units.len(),
        1,
        "第一个文件只有一个数据单元：{data_units:?}"
    );
    [
        UnitToRelocate {
            source_slot: data_units[0],
            span_in_slots: 2,
        },
        UnitToRelocate {
            source_slot: the_slot_of(&extent_root),
            span_in_slots: 1,
        },
    ]
}

/// 最新那一版分配记录树里装着盘 `device` 的记录、按位置罩住 `slot` 的那片叶（槽号、叶的字节）。
fn allocation_leaf_holding(
    image: &MemoryPool,
    device: DeviceIdentity,
    slot: SlotNumber,
) -> (SlotNumber, Vec<u8>) {
    let root = tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION).root;
    allocation_record_tree_nodes_as_far_as_readable(image, &root)
        .into_iter()
        .map(|pointer| {
            let node_slot = the_slot_of(&pointer);
            (node_slot, unit_bytes_at(image, node_slot, node_bytes()))
        })
        .find(|(_, node)| {
            let header = parse_index_node(node).expect("分配记录树节点解得开");
            header.level == 0
                && header.entries.iter().any(|record| {
                    read_u32_at(record, 0) == device.0 && read_six_bytes_at(record, 4) == slot.0
                })
        })
        .expect("最新那一版的分配记录里有这个单元的记录")
}

/// 目的槽的起点：两块盘上装着源记录的那几片叶里在册单元的末槽取最大、隔开 [`DESTINATION_GAP_IN_SLOTS`] 再对齐到 2 槽，
/// 交回之前核它与搬的几个单元依次排开之后仍落在每片叶按位置罩的那一段里。
fn first_destination_slot(image: &MemoryPool, units: &[UnitToRelocate]) -> SlotNumber {
    let mut end_of_the_last_unit = 0u64;
    let mut end_of_the_leaf_ranges = u64::MAX;
    for device in DEVICES {
        for unit in units {
            let (_, leaf) = allocation_leaf_holding(image, device, unit.source_slot);
            let header = parse_index_node(&leaf).expect("分配记录叶解得开");
            for record in &header.entries {
                if read_u32_at(record, 0) == device.0 {
                    let span = u64::from(
                        read_u16_at(record, ALLOCATION_RECORD_SPAN_OFFSET)
                            & !ALLOCATION_RECORD_RELEASED_FLAG,
                    );
                    end_of_the_last_unit =
                        end_of_the_last_unit.max(read_six_bytes_at(record, 4) + span);
                }
            }
            end_of_the_leaf_ranges =
                end_of_the_leaf_ranges.min(read_six_bytes_at(&header.largest_key, 4));
        }
    }
    let first = (end_of_the_last_unit + DESTINATION_GAP_IN_SLOTS).next_multiple_of(2);
    let span_of_every_unit: u64 = units.iter().map(|unit| unit.span_in_slots).sum();
    assert!(
        first + span_of_every_unit <= end_of_the_leaf_ranges,
        "目的槽 {first} 起 {span_of_every_unit} 槽落在叶罩的那一段（末槽 {end_of_the_leaf_ranges}）里"
    );
    SlotNumber(first)
}

/// 映射叶里位置条目落在 `slot`（两条都在这一槽）的那一条的下标。
fn mapping_entry_of_the_unit_at(mapping: &[Vec<u8>], slot: SlotNumber) -> usize {
    let at_the_slot: Vec<usize> = mapping
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            (0..2).all(|location| {
                read_six_bytes_at(
                    entry,
                    mapping_key_bytes() + location * MAPPING_LOCATION_ENTRY_BYTES + 4,
                ) == slot.0
            })
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(at_the_slot.len(), 1, "映射里恰好一条指着槽 {slot:?}");
    at_the_slot[0]
}

/// 一个码 2 节点换成同一个头、`entries` 这些条目（key 区间沿用原头的：分配记录叶按位置罩那一段，与条目无关），重封。
fn rebuilt_with_entries(node: &[u8], entries: &[Vec<u8>]) -> Vec<u8> {
    let header = parse_index_node(node).expect("节点解得开");
    let mut rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽装得进 u16"),
        entries,
    );
    reseal_index_node(&mut rebuilt);
    rebuilt
}

/// 在最新那一版里搬 `units`（见文件头）。交回每个单元搬到了哪。
fn relocate_in_the_newest_version(
    image: &mut MemoryPool,
    units: &[UnitToRelocate],
    allocation_records: AllocationRecordsOfTheRelocation,
) -> Vec<RelocatedUnit> {
    let newest_txg = newest_root(image).checkpoint_txg.0;
    let mut destination = first_destination_slot(image, units);
    let mapping_entries = parse_index_node(&unit_bytes_at(
        image,
        the_slot_of(&newest_root(image).mapping_root),
        node_bytes(),
    ))
    .expect("中央映射根解得开")
    .entries;
    let mut relocated = Vec::new();
    for unit in units {
        let entry =
            &mapping_entries[mapping_entry_of_the_unit_at(&mapping_entries, unit.source_slot)];
        relocated.push(RelocatedUnit {
            unit: *unit,
            destination_slot: destination,
            birth_txg: read_u64_at(entry, MAPPING_KEY_BIRTH_TXG_OFFSET),
        });
        destination = SlotNumber(destination.0 + unit.span_in_slots);
    }
    // 单元原样拷过去：两盘同槽，逐字节不变（头不盖戳、birth 不变，判决 G3）。
    for relocation in &relocated {
        let length = usize::try_from(relocation.unit.span_in_slots * SLOT_BYTES).expect("单元宽");
        for device in DEVICES {
            let sparse = image.devices.get_mut(&device).expect("池里有这块盘");
            let bytes = sparse.read(relocation.unit.source_slot.to_device_offset(), length);
            sparse.write(relocation.destination_slot.to_device_offset(), &bytes);
        }
    }
    // 分配记录：每块盘上装着源记录的那片叶各改一次（搬的几个单元落在同一片叶里时一起改）。
    for device in DEVICES {
        let mut leaves_rewritten: Vec<SlotNumber> = Vec::new();
        for relocation in &relocated {
            let (leaf_slot, leaf) =
                allocation_leaf_holding(image, device, relocation.unit.source_slot);
            if leaves_rewritten.contains(&leaf_slot) {
                continue;
            }
            leaves_rewritten.push(leaf_slot);
            let mut records = parse_index_node(&leaf).expect("分配记录叶解得开").entries;
            let relocated_in_this_leaf: Vec<RelocatedUnit> = relocated
                .iter()
                .filter(|candidate| {
                    records.iter().any(|record| {
                        read_u32_at(record, 0) == device.0
                            && read_six_bytes_at(record, 4) == candidate.unit.source_slot.0
                    })
                })
                .copied()
                .collect();
            for in_this_leaf in &relocated_in_this_leaf {
                let source = records
                    .iter_mut()
                    .find(|record| {
                        read_u32_at(record, 0) == device.0
                            && read_six_bytes_at(record, 4) == in_this_leaf.unit.source_slot.0
                    })
                    .expect("源记录在这片叶里");
                let span = read_u16_at(source, ALLOCATION_RECORD_SPAN_OFFSET);
                source[ALLOCATION_RECORD_SPAN_OFFSET..ALLOCATION_RECORD_SPAN_OFFSET + 2]
                    .copy_from_slice(&(span | ALLOCATION_RECORD_RELEASED_FLAG).to_le_bytes());
                source
                    [ALLOCATION_RECORD_GENERATION_OFFSET..ALLOCATION_RECORD_GENERATION_OFFSET + 8]
                    .copy_from_slice(&newest_txg.to_le_bytes());
                match allocation_records {
                    AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated => {
                        let mut destination_record = vec![0u8; ALLOCATION_RECORD_BYTES];
                        destination_record[..4].copy_from_slice(&device.0.to_le_bytes());
                        destination_record[4..10]
                            .copy_from_slice(&in_this_leaf.destination_slot.0.to_le_bytes()[..6]);
                        destination_record
                            [ALLOCATION_RECORD_SPAN_OFFSET..ALLOCATION_RECORD_SPAN_OFFSET + 2]
                            .copy_from_slice(&span.to_le_bytes());
                        destination_record[ALLOCATION_RECORD_GENERATION_OFFSET
                            ..ALLOCATION_RECORD_GENERATION_OFFSET + 8]
                            .copy_from_slice(&in_this_leaf.birth_txg.to_le_bytes());
                        records.push(destination_record);
                    }
                    AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationNotRecorded => {}
                }
            }
            records.sort_by_key(|record| (read_u32_at(record, 0), read_six_bytes_at(record, 4)));
            let rebuilt = rebuilt_with_entries(&leaf, &records);
            replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &rebuilt);
        }
    }
    // 记账：每块盘上已分配加、空闲减目的那几槽，defer 加源那几槽。目的槽离最后一个在册单元隔开至少 64 个空槽、依次排开，
    // 后面没有在册的单元（[`first_destination_slot`]）：落在一段空闲槽中间、成一个新的占用块，runs 加一；它罩到的聚簇段原来一槽都不占
    // （段起点不早于最后一个在册单元的末槽），全空聚簇段数减它罩到的段数（源槽还在 defer、源记录所在的段里还有别的未释放记录，两种口径同减）。
    let segments_the_destinations_reach: std::collections::BTreeSet<u64> = relocated
        .iter()
        .flat_map(|relocation| {
            relocation.destination_slot.0
                ..relocation.destination_slot.0 + relocation.unit.span_in_slots
        })
        .map(|slot| (slot - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS)
        .collect();
    let segments_newly_occupied =
        u64::try_from(segments_the_destinations_reach.len()).expect("段数");
    let accounting_slot = the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_ACCOUNTING).root);
    let accounting = unit_bytes_at(image, accounting_slot, node_bytes());
    let mut rows = parse_index_node(&accounting).expect("记账根解得开").entries;
    let relocated_bytes: u64 = relocated
        .iter()
        .map(|relocation| relocation.unit.span_in_slots * SLOT_BYTES)
        .sum();
    for row in &mut rows {
        let statistic = read_u16_at(row, 0);
        if !DEVICES
            .iter()
            .any(|device| device.0 == read_u32_at(row, ACCOUNTING_ROW_DEVICE_OFFSET))
        {
            continue;
        }
        let value = read_u64_at(row, ACCOUNTING_ROW_VALUE_OFFSET);
        let changed = match (statistic, allocation_records) {
            (
                STATISTIC_ALLOCATED_BYTES,
                AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
            ) => value + relocated_bytes,
            (
                STATISTIC_FREE_BYTES,
                AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
            ) => value - relocated_bytes,
            (STATISTIC_DEFER_QUEUE_BYTES, _) => value + relocated_bytes,
            (
                STATISTIC_FRAGMENTATION_RUNS,
                AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
            ) => value + 1,
            (
                STATISTIC_EMPTY_CLUSTER_SEGMENTS,
                AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
            ) => value - segments_newly_occupied,
            (_, _) => value,
        };
        row[ACCOUNTING_ROW_VALUE_OFFSET..ACCOUNTING_ROW_VALUE_OFFSET + 8]
            .copy_from_slice(&changed.to_le_bytes());
    }
    let rebuilt_accounting = rebuilt_with_entries(&accounting, &rows);
    replace_the_unit_and_propagate_its_checksum(image, accounting_slot, &rebuilt_accounting);
    // 映射条目换位置：上面几步补校验和时映射叶已经改过，这里重读。
    let mapping_slot = the_slot_of(&newest_root(image).mapping_root);
    let mapping = unit_bytes_at(image, mapping_slot, node_bytes());
    let mut entries = parse_index_node(&mapping)
        .expect("中央映射根解得开")
        .entries;
    for relocation in &relocated {
        let index = mapping_entry_of_the_unit_at(&entries, relocation.unit.source_slot);
        for location in 0..2 {
            let slot_at = mapping_key_bytes() + location * MAPPING_LOCATION_ENTRY_BYTES + 4;
            entries[index][slot_at..slot_at + 6]
                .copy_from_slice(&relocation.destination_slot.0.to_le_bytes()[..6]);
        }
        assert_eq!(
            entries[index].len(),
            usize::try_from(MAPPING_ENTRY_BYTES).expect("55")
        );
    }
    let rebuilt_mapping = rebuilt_with_entries(&mapping, &entries);
    replace_the_unit_and_propagate_its_checksum(image, mapping_slot, &rebuilt_mapping);
    relocated
}

fn violated(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<(&'static str, String)> {
    verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 判定只留种类（成立、违例、不适用、部分没判）：搬迁前后逐条比它。
fn verdict_kinds(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> Vec<(&'static str, &'static str)> {
    verdicts
        .iter()
        .map(|(invariant, verdict)| {
            let kind = match verdict {
                InvariantVerdict::Holds => "成立",
                InvariantVerdict::Violated(_) => "违例",
                InvariantVerdict::NotApplicable(_) => "不适用",
                InvariantVerdict::PartiallyJudged(_) => "部分没判",
            };
            (*invariant, kind)
        })
        .collect()
}

/// 合法搬迁：数据单元与 extent 树的根都搬了，extent 树与树表里的指针还指着源槽（提示过期、源槽已释放、两份旧副本原样还在）。
/// 每一条不变量的判定种类与搬迁之前那份镜像逐条相同、一条违例都没有——I-3.1 / I-3.11 / I-3.9 按映射位置数引用（源槽在最新那一版不再被引用、
/// 释放代落在搬迁那一格），I-2.1 只在映射位置判，I-3.5 把搬前、搬后的两个落点按映射 key 折成同一个单元，
/// I-1.13 认扫描重建在源槽多扫到的同一版副本（判决 G3、G6）。I-1.14（同一映射 key 在各版里的单元逐字节相同） 例外：搬迁之前没有一把 key
/// 在两版里落在不同的位置、报不适用，搬迁之后两个 key 在上一版与最新那一版各落一处，逐字节比上了、成立。
#[test]
fn units_relocated_by_the_central_mapping_with_their_sources_released_keep_every_verdict_of_the_image_before(
) {
    let pool = pool_with_an_empty_inode_after_the_first_file("relocated-by-the-mapping-legal");
    let before = pool.memory_pool();
    let verdicts_before = check_pool_image(&before);
    assert_eq!(
        violated(&verdicts_before),
        [],
        "搬迁之前的镜像是写路径写出的合法镜像"
    );
    let mut image = before.clone();
    let units = data_unit_and_extent_root_of_the_newest_version(&image);
    let relocated = relocate_in_the_newest_version(
        &mut image,
        &units,
        AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
    );
    assert!(
        relocated
            .iter()
            .all(|relocation| relocation.destination_slot != relocation.unit.source_slot),
        "都搬到了别的槽：{relocated:?}"
    );
    let verdicts_after = check_pool_image(&image);
    assert_eq!(
        violated(&verdicts_after),
        [],
        "合法搬迁之后一条违例都没有（搬迁 {relocated:?}）"
    );
    let kinds_other_than_the_same_bytes_invariant =
        |verdicts: &[(&'static str, InvariantVerdict)]| {
            verdict_kinds(verdicts)
                .into_iter()
                .filter(|(invariant, _)| {
                    *invariant != UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS
                })
                .collect::<Vec<_>>()
        };
    assert_eq!(
        kinds_other_than_the_same_bytes_invariant(&verdicts_after),
        kinds_other_than_the_same_bytes_invariant(&verdicts_before),
        "除 I-1.14 之外每一条的判定种类与搬迁之前相同"
    );
    for invariant in [
        "I-3.1",
        "I-3.11",
        "I-3.9",
        "I-2.1",
        SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
        UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
    ] {
        assert_eq!(
            verdicts_after
                .iter()
                .find(|(candidate, _)| *candidate == invariant)
                .map(|(_, verdict)| verdict.clone()),
            Some(InvariantVerdict::Holds),
            "{invariant} 在合法搬迁过的镜像上判到且成立"
        );
    }
}

/// 必红世界：映射条目指的目的槽两份内容都不对（两块盘上各翻一个字节、不重封）。被引用的块在映射位置上，I-2.1 在那一槽判红；
/// 映射那一遍两份都读不出，最新的根走不完（I-7.2 红、点名映射条目指的单元）。树里那条指针先按提示读（判决 G3「走读先按提示读、
/// 两份都对不上再按映射位置读」），源槽那两份原样还在、读得出，extent 树那一支不断：I-7.2 的说明里没有数据单元读不出那一处。
#[test]
fn destination_copies_that_do_not_match_their_mapping_entry_redden_the_checksum_invariant_while_the_walk_reads_the_hints(
) {
    let pool = pool_with_an_empty_inode_after_the_first_file("relocated-by-the-mapping-bad-copy");
    let mut image = pool.memory_pool();
    let units = data_unit_and_extent_root_of_the_newest_version(&image);
    let relocated = relocate_in_the_newest_version(
        &mut image,
        &units[..1],
        AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
    );
    let destination = relocated[0].destination_slot;
    for device in DEVICES {
        let sparse = image.devices.get_mut(&device).expect("池里有这块盘");
        let byte_at = destination.to_device_offset();
        let mut sector = sparse.read(byte_at, 512);
        sector[200] ^= 0xff;
        sparse.write(byte_at, &sector);
    }
    let verdicts = check_pool_image(&image);
    match verdicts
        .iter()
        .find(|(invariant, _)| *invariant == "I-7.2")
        .map(|(_, verdict)| verdict)
    {
        Some(InvariantVerdict::Violated(detail)) => {
            assert!(
                detail.contains("映射条目指的单元两份都读不到对得上的"),
                "映射那一遍两份都读不出：{detail}"
            );
            assert!(
                !detail.contains("数据单元两份都读不到对得上的"),
                "树里那条指针按提示读得出，extent 树那一支不断：{detail}"
            );
        }
        other @ (Some(
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_),
        )
        | None) => panic!("映射条目指的那一个单元两份都读不出，I-7.2 要红：{other:?}"),
    }
    let red = violated(&verdicts);
    let checksum_violations: Vec<&String> = red
        .iter()
        .filter(|(invariant, _)| *invariant == "I-2.1")
        .map(|(_, detail)| detail)
        .collect();
    assert!(
        checksum_violations
            .iter()
            .any(|detail| detail.contains(&format!("槽 {} 的那一份", destination.0))),
        "I-2.1 在映射位置（槽 {}）判红：{red:?}",
        destination.0
    );
}

/// 必红世界：映射与分配记录对不上——映射条目换到目的槽、源记录记释放（defer 跟着加），目的槽没有一条分配记录、已分配与空闲不动。
/// 最新那一版按映射位置引用了一个没人分配的槽：I-3.1（全部有效根的引用并集比已分配多出目的那两槽）与 I-3.11（已分配减 defer 比最新根走读少）都红。
#[test]
fn a_destination_slot_that_no_allocation_record_covers_reddens_the_allocated_bytes_invariants() {
    let pool = pool_with_an_empty_inode_after_the_first_file("relocated-by-the-mapping-unrecorded");
    let mut image = pool.memory_pool();
    let units = data_unit_and_extent_root_of_the_newest_version(&image);
    relocate_in_the_newest_version(
        &mut image,
        &units[..1],
        AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationNotRecorded,
    );
    let red = violated(&check_pool_image(&image));
    let red_names: Vec<&str> = red.iter().map(|(invariant, _)| *invariant).collect();
    for invariant in ["I-3.1", "I-3.11"] {
        assert!(
            red_names.contains(&invariant),
            "映射指着一个没有分配记录的槽：{invariant} 要红：{red:?}"
        );
    }
}
