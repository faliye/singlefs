//! 扫描重建与遍历方向读出的索引比对：I-1.13（扫描重建与遍历方向的索引相同）。

use super::*;

/// 中央映射树的全部叶条目，只读不判（D8（核心索引结构） 已定项 11：多层码 2 树，key 宽 27；叶条目 55 = key 27 + 位置条目 14 × 2，
/// 内部条目 = key 27 + 子指针 86）。有一个节点读不出、key 宽不是 27、条目宽窄于字段表、孩子层级不是父层级减一、子指针全零 ⇒ None。
/// 递归的上界：每下一层层级减一，层级 0 停。
pub(super) fn central_mapping_leaf_entries_without_judging(
    reader: &dyn ImageReader,
    node_pointer: &PointerView,
    level_the_parent_names: Option<u8>,
    cache: &mut IndexNodeCache,
    leaf_entries: &mut Vec<Vec<u8>>,
) -> Option<()> {
    let node = read_index_node_without_judging(
        reader,
        node_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        cache,
    )?;
    let key_width = KEY_SCHEMA_MAPPING.width();
    if node.key_width != key_width
        || level_the_parent_names.is_some_and(|level| level != node.level)
    {
        return None;
    }
    let Some(child_level) = node.level.checked_sub(1) else {
        if node.entry_width < mapping_entry_bytes() {
            return None;
        }
        leaf_entries.extend(node.entries.iter().cloned());
        return Some(());
    };
    if node.entry_width < internal_entry_bytes(KEY_SCHEMA_MAPPING) {
        return None;
    }
    for entry in &node.entries {
        let child = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
        if child.all_zero {
            return None;
        }
        central_mapping_leaf_entries_without_judging(
            reader,
            &child,
            Some(child_level),
            cache,
            leaf_entries,
        )?;
    }
    Some(())
}

/// 一个码 1 数据指针（88 字节，D19（块指针的结构与宽度预算） 已定项 6）指的那个单元的中央映射 key：类标签 1 + 出生树 + 出生 txg + 写序 10，
/// 各段取指针头里的（遍历方向自己那一边的读法，扫描重建从单元头现算另一边）。
pub(super) fn central_mapping_key_of_a_data_pointer(pointer: &PointerView) -> CentralMappingKey {
    central_mapping_key_of_a_pointer(pointer, MappingKeyUnitClass::DataUnit)
}

pub(super) fn mapping_key_bytes() -> usize {
    usize::try_from(MAPPING_KEY_BYTES).expect("27")
}

/// 遍历方向从最新根读出的三张表（I-1.13（扫描重建与遍历方向的索引相同） 比对的那一边），只读不判。中央映射只留码 1 / 码 3 的条目：
/// 扫描重建第一版只重建这两类（码 2 节点不进比对，码 0 与未登记码由 I-1.11（映射 key 与单元头相符） 判）。
struct IndexTablesWalkedFromTheNewestRoot {
    pub(super) central_mapping: BTreeMap<CentralMappingKey, [PointerLocation; 2]>,
    data_units_of_each_inode: BTreeMap<DataUnitPosition, CentralMappingKey>,
    inode_records: BTreeMap<u64, Vec<u8>>,
}

/// 遍历方向那三张表读不全的是哪一样：I-1.13 报不适用并带这个理由（读不出、形状不对由主走读的各条判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IndexTableTheWalkDoesNotReadInFull {
    TreeTable,
    ExtentTree,
    InodeTree,
    CentralMappingTree,
    ExtentTreeNamesOnePositionTwice,
    CentralMappingTreeNamesOneKeyTwice,
    DeadlistTree,
}

impl IndexTableTheWalkDoesNotReadInFull {
    pub(super) fn reason(self) -> &'static str {
        match self {
            IndexTableTheWalkDoesNotReadInFull::TreeTable => {
                "最新根的树表读不出、解不开、条目窄于字段表或同一种树出现两条：遍历方向那三张表读不全"
            }
            IndexTableTheWalkDoesNotReadInFull::ExtentTree => {
                "最新根的 extent 树读不全：遍历方向的每个 inode 的数据单元集合读不全"
            }
            IndexTableTheWalkDoesNotReadInFull::InodeTree => {
                "最新根的 inode 树读不全：遍历方向的 inode 记录集合读不全"
            }
            IndexTableTheWalkDoesNotReadInFull::CentralMappingTree => {
                "最新根的中央映射树读不全：遍历方向的中央映射读不全"
            }
            IndexTableTheWalkDoesNotReadInFull::ExtentTreeNamesOnePositionTwice => {
                "最新根的 extent 树里同一个 (inode, 偏移) 有两条：遍历方向的数据单元集合不是一张表"
            }
            IndexTableTheWalkDoesNotReadInFull::CentralMappingTreeNamesOneKeyTwice => {
                "最新根的中央映射树里同一个 key 有两条：遍历方向的中央映射不是一张表"
            }
            IndexTableTheWalkDoesNotReadInFull::DeadlistTree => {
                "最新根的 deadlist 树读不全：被活快照钉住、映射条目留存的那几个单元认不全，遍历方向的中央映射里哪些是当前版本分不出"
            }
        }
    }
}

/// 最新根下面的三张表：树表、extent 树、inode 树与中央映射树各读一遍（与删除态扫描、I-9.6 那几遍共用节点缓存）。
/// size 之外的数据单元两张表都不收：inode 树里有它那个号的记录、
/// 偏移 ÷ 净荷容量不小于 ⌈size ÷ 净荷⌉.max(1) 的单元（截断中间态里 size 之外还挂着的）不进每个 inode 的数据单元集合，它的映射 key 不进中央映射。
/// 扫描重建那一侧照读法 Z 加写序不认领它们（`rebuild::rebuild_index_tables`），两侧各写各的、只共享格式解析。
fn index_tables_walked_from_the_newest_root(
    reader: &dyn ImageReader,
    newest_root_record: &[u8],
    index_cache: &mut IndexNodeCache,
) -> Result<IndexTablesWalkedFromTheNewestRoot, IndexTableTheWalkDoesNotReadInFull> {
    // inode 树、extent 树与 deadlist 树里豁免三类之外的单元按最新根自己的中央映射认位置（读不出时是空表，照提示）。
    let central_mapping_of_the_newest_root =
        central_mapping_of_a_root_record_without_judging(reader, newest_root_record, index_cache);
    let user_visible_tree_roots =
        user_visible_tree_root_pointers(reader, newest_root_record, index_cache)
            .ok_or(IndexTableTheWalkDoesNotReadInFull::TreeTable)?;
    let inode_records = match user_visible_tree_roots
        .inode_tree
        .as_deref()
        .map(parse_node_pointer)
    {
        Some(inode_root) => inode_record_bytes_without_judging(
            reader,
            &inode_root,
            &central_mapping_of_the_newest_root,
            index_cache,
            &mut PackedUnitCache::new(),
        )
        .map_err(|_gap| IndexTableTheWalkDoesNotReadInFull::InodeTree)?,
        None => BTreeMap::new(),
    };
    let payload_capacity_in_bytes = data_unit_payload_capacity_in_bytes();
    let mut mapping_keys_of_the_data_units_beyond_the_size: BTreeSet<CentralMappingKey> =
        BTreeSet::new();
    let mut data_units_of_each_inode = BTreeMap::new();
    if let Some(extent_root) = user_visible_tree_roots
        .extent_tree
        .as_deref()
        .map(parse_node_pointer)
        .filter(|pointer| !pointer.all_zero)
    {
        let extent_tree = extent_tree_with_the_inode_of_each_data_pointer_without_judging(
            reader,
            &extent_root,
            &central_mapping_of_the_newest_root,
            index_cache,
        )
        .ok_or(IndexTableTheWalkDoesNotReadInFull::ExtentTree)?;
        for data_pointer_of_an_inode in extent_tree
            .data_pointers_of_inodes
            .iter()
            .filter(|data_pointer_of_an_inode| !data_pointer_of_an_inode.pointer.all_zero)
        {
            let position = DataUnitPosition {
                inode: data_pointer_of_an_inode.inode,
                offset_in_bytes: data_pointer_of_an_inode.offset_in_bytes,
            };
            let key = central_mapping_key_of_a_data_pointer(&data_pointer_of_an_inode.pointer);
            let lies_beyond_the_size_of_its_record =
                inode_records.get(&position.inode).is_some_and(|record| {
                    position.offset_in_bytes / payload_capacity_in_bytes
                        >= data_units_the_size_asks_for(read_u64(record, INODE_RECORD_SIZE_OFFSET))
                });
            if lies_beyond_the_size_of_its_record {
                mapping_keys_of_the_data_units_beyond_the_size.insert(key);
                continue;
            }
            if data_units_of_each_inode.insert(position, key).is_some() {
                return Err(IndexTableTheWalkDoesNotReadInFull::ExtentTreeNamesOnePositionTwice);
            }
        }
    }
    let mapping_root = parse_node_pointer(&newest_root_record[ROOT_RECORD_MAPPING_ROOT_POINTER]);
    let mut leaf_entries = Vec::new();
    if !mapping_root.all_zero {
        central_mapping_leaf_entries_without_judging(
            reader,
            &mapping_root,
            None,
            index_cache,
            &mut leaf_entries,
        )
        .ok_or(IndexTableTheWalkDoesNotReadInFull::CentralMappingTree)?;
    }
    let keys_pinned_by_the_deadlist = mapping_keys_named_by_the_deadlist_of_the_newest_root(
        reader,
        newest_root_record,
        &central_mapping_of_the_newest_root,
        index_cache,
    )?;
    let mut central_mapping = BTreeMap::new();
    for entry in leaf_entries
        .iter()
        .filter(|entry| ContentUnitClass::of(entry[MAPPING_KEY_UNIT_CLASS_OFFSET]).is_some())
        .filter(|entry| {
            let key = CentralMappingKey(entry[..mapping_key_bytes()].to_vec());
            !keys_pinned_by_the_deadlist.contains(&key)
                && !mapping_keys_of_the_data_units_beyond_the_size.contains(&key)
        })
    {
        let locations_pointer: Vec<u8> = [
            vec![0u8; 50],
            entry[mapping_key_bytes()..mapping_entry_bytes()].to_vec(),
            vec![0u8; 8],
        ]
        .concat();
        let key = CentralMappingKey(entry[..mapping_key_bytes()].to_vec());
        if central_mapping
            .insert(key, parse_node_pointer(&locations_pointer).locations)
            .is_some()
        {
            return Err(IndexTableTheWalkDoesNotReadInFull::CentralMappingTreeNamesOneKeyTwice);
        }
    }
    Ok(IndexTablesWalkedFromTheNewestRoot {
        central_mapping,
        data_units_of_each_inode,
        inode_records,
    })
}

/// 最新根的 deadlist 里每一条点名的中央映射 key（条目偏移 16 起 27 字节，D5（快照 / 空间记账机制） 已定项 12）：被活快照钉住、活头已不引用的单元，
/// 它们的映射条目照上一版两条位置项留在中央映射树里，
/// 不是任何一个当前版本——扫描重建只重建当前版本，I-1.13 比的遍历方向那一张中央映射不收它们（判决 S5「输入取不到报不适用」同一个口径：
/// 留存的条目由 I-3.5 ③ ④ 与逆向、I-3.6 判）。树表里没有 deadlist 树条目、根指针全零 ⇒ 空集；树表读不出、deadlist 树条目不止一条、
/// deadlist 树读不全 ⇒ 交回读不全的是哪一样，I-1.13 报不适用。
fn mapping_keys_named_by_the_deadlist_of_the_newest_root(
    reader: &dyn ImageReader,
    newest_root_record: &[u8],
    central_mapping: &CentralMappingLocationsOfAVersion,
    index_cache: &mut IndexNodeCache,
) -> Result<BTreeSet<CentralMappingKey>, IndexTableTheWalkDoesNotReadInFull> {
    let tree_table_pointer =
        parse_node_pointer(&newest_root_record[ROOT_RECORD_TREE_TABLE_POINTER]);
    if tree_table_pointer.all_zero {
        return Ok(BTreeSet::new());
    }
    let tree_table = read_index_node_without_judging(
        reader,
        &tree_table_pointer,
        PositionOfASecondaryRead::ThePointerItself,
        index_cache,
    )
    .ok_or(IndexTableTheWalkDoesNotReadInFull::TreeTable)?;
    let mut deadlist_roots = Vec::new();
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() {
            return Err(IndexTableTheWalkDoesNotReadInFull::TreeTable);
        }
        if read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_DEADLIST {
            deadlist_roots.push(parse_node_pointer(&entry[14..100]));
        }
    }
    let deadlist_root = match deadlist_roots.as_slice() {
        [] => return Ok(BTreeSet::new()),
        [deadlist_root] => *deadlist_root,
        [_, _, ..] => return Err(IndexTableTheWalkDoesNotReadInFull::TreeTable),
    };
    if deadlist_root.all_zero {
        return Ok(BTreeSet::new());
    }
    let mut deadlist_entries = Vec::new();
    leaf_entries_of_a_multi_level_tree_without_judging(
        reader,
        &deadlist_root,
        KEY_SCHEMA_DEADLIST,
        deadlist_entry_bytes(),
        None,
        central_mapping,
        index_cache,
        &mut deadlist_entries,
    )
    .ok_or(IndexTableTheWalkDoesNotReadInFull::DeadlistTree)?;
    Ok(deadlist_entries
        .iter()
        .map(|entry| {
            CentralMappingKey(
                entry[DEADLIST_ENTRY_MAPPING_KEY_OFFSET
                    ..DEADLIST_ENTRY_MAPPING_KEY_OFFSET + mapping_key_bytes()]
                    .to_vec(),
            )
        })
        .collect())
}

/// I-1.13 比的三张表（判红时写进说明的表名）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IndexTableOfTheScanRebuild {
    CentralMapping,
    DataUnitsOfEachInode,
    InodeRecords,
}

impl IndexTableOfTheScanRebuild {
    pub(super) fn name(self) -> &'static str {
        match self {
            IndexTableOfTheScanRebuild::CentralMapping => "中央映射",
            IndexTableOfTheScanRebuild::DataUnitsOfEachInode => "每个 inode 的数据单元集合",
            IndexTableOfTheScanRebuild::InodeRecords => "inode 记录集合",
        }
    }
}

/// 一张表上逐 key 比出来的差异：每一处写成「key …：扫描重建 …、遍历方向 …」。
struct DifferencesInOneTable {
    table: IndexTableOfTheScanRebuild,
    differences: Vec<String>,
}

fn describe_placements(placements: &BTreeSet<UnitPlacement>) -> String {
    let named: Vec<String> = placements
        .iter()
        .map(|placement| format!("盘 {} 槽 {}", placement.device, placement.slot))
        .collect();
    format!("落点 {{{}}}", named.join("、"))
}

fn describe_inode_record(record: &[u8]) -> String {
    format!(
        "记录（对象出生代 {}、nlink {}、size {}）",
        read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET),
        read_u32(record, INODE_RECORD_NLINK_OFFSET),
        read_u64(record, INODE_RECORD_SIZE_OFFSET)
    )
}

/// 一份码 1 / 码 3 副本在盘上读得出来、载荷与它自己头里的载荷校验和对得上：「可读」的后半截（与 I-1.8 那一遍同一个判法）。
pub(super) fn copy_payload_checksum_holds_on_disk(
    reader: &dyn ImageReader,
    unit_class: ContentUnitClass,
    device: u32,
    slot: u64,
) -> bool {
    let offsets = unit_class.header_offsets();
    let Some(bytes) = reader.read(device, slot * SLOT_BYTES, data_unit_bytes()) else {
        return false;
    };
    bytes.len() > offsets.plain_header_end
        && read_u32(&bytes, offsets.payload_checksum)
            == crc32_castagnoli_table(&bytes[offsets.plain_header_end..])
}

/// 中央映射那张表逐 key 比。一个 key 两边都有时比落点：遍历方向的落点集合是扫描重建那一份的子集就对得上（搬迁留下的源落点，见下）；
/// 不是子集时两边各只留读得出来的那几份再比——
/// 遍历方向那一份按位置条目里的整单元校验和读回、扫描重建那一份按头里的载荷校验和读回（可读的后半截只在两边不同时才现算：
/// 整单元 CRC 是这里最贵的一步，崩溃点重放每个状态都跑这一遍）。少了一块盘、一份副本读不出的降级镜像因此不判红；
/// 落点条目改成别的槽（读回对不上）、扫到的那一份不在映射里，判红。
fn differences_in_the_central_mapping(
    reader: &dyn ImageReader,
    rebuilt: &BTreeMap<CentralMappingKey, BTreeSet<UnitPlacement>>,
    walked: &BTreeMap<CentralMappingKey, [PointerLocation; 2]>,
) -> Vec<String> {
    let keys: BTreeSet<&CentralMappingKey> = rebuilt.keys().chain(walked.keys()).collect();
    let mut differences = Vec::new();
    for key in keys {
        match (rebuilt.get(key), walked.get(key)) {
            (Some(rebuilt_placements), Some(walked_locations)) => {
                let walked_placements: BTreeSet<UnitPlacement> = walked_locations
                    .iter()
                    .map(|location| UnitPlacement {
                        device: location.device,
                        slot: location.slot,
                    })
                    .collect();
                // 遍历方向的落点都是扫描重建认领的这一版的副本，扫描重建多出来的也是这一版逐字节相同的副本：搬迁把单元原样写到目的落点、
                // 映射条目换位置，源落点进 defer、回收之前那两份还在、头里的写序与目的那两份相同
                // 只从单元头扫不出哪一对是现行落点。映射说的是哪一对由映射与分配记录那几条判
                // （I-1.11、I-2.1、I-3.1、I-3.11），这里对得上。
                if walked_placements.is_subset(rebuilt_placements) {
                    continue;
                }
                let unit_class = ContentUnitClass::of(key.0[MAPPING_KEY_UNIT_CLASS_OFFSET])
                    .expect("两边都只留码 1 / 码 3 的 key");
                let readable_rebuilt: BTreeSet<UnitPlacement> = rebuilt_placements
                    .iter()
                    .copied()
                    .filter(|placement| {
                        copy_payload_checksum_holds_on_disk(
                            reader,
                            unit_class,
                            placement.device,
                            placement.slot,
                        )
                    })
                    .collect();
                let readable_walked: BTreeSet<UnitPlacement> = walked_locations
                    .iter()
                    .filter(|location| {
                        reader
                            .read(
                                location.device,
                                location.slot * SLOT_BYTES,
                                data_unit_bytes(),
                            )
                            .is_some_and(|bytes| {
                                crc32_castagnoli_table(&bytes) == location.checksum
                            })
                    })
                    .map(|location| UnitPlacement {
                        device: location.device,
                        slot: location.slot,
                    })
                    .collect();
                if readable_rebuilt != readable_walked {
                    differences.push(format!(
                        "key {}：扫描重建 {}（读得出的 {}）、遍历方向 {}（读得出的 {}）",
                        key.describe(),
                        describe_placements(rebuilt_placements),
                        describe_placements(&readable_rebuilt),
                        describe_placements(&walked_placements),
                        describe_placements(&readable_walked)
                    ));
                }
            }
            (Some(rebuilt_placements), None) => differences.push(format!(
                "key {}：扫描重建 {}、遍历方向没有这一条",
                key.describe(),
                describe_placements(rebuilt_placements)
            )),
            (None, Some(walked_locations)) => differences.push(format!(
                "key {}：扫描重建没有这一条、遍历方向 {}",
                key.describe(),
                describe_placements(
                    &walked_locations
                        .iter()
                        .map(|location| UnitPlacement {
                            device: location.device,
                            slot: location.slot,
                        })
                        .collect()
                )
            )),
            (None, None) => unreachable!("key 取自两边的并集"),
        }
    }
    differences
}

/// 值不带落点的两张表（每个 inode 的数据单元集合、inode 记录集合）逐 key 比：一边有一边没有、或两边的值不同，各记一处。
fn differences_in_a_table_of_values<Key: Ord, Value: PartialEq>(
    rebuilt: &BTreeMap<Key, Value>,
    walked: &BTreeMap<Key, Value>,
    describe_key: impl Fn(&Key) -> String,
    describe_value: impl Fn(&Value) -> String,
) -> Vec<String> {
    let keys: BTreeSet<&Key> = rebuilt.keys().chain(walked.keys()).collect();
    keys.into_iter()
        .filter_map(|key| match (rebuilt.get(key), walked.get(key)) {
            (Some(rebuilt_value), Some(walked_value)) if rebuilt_value == walked_value => None,
            (rebuilt_value, walked_value) => Some(format!(
                "key {}：扫描重建 {}、遍历方向 {}",
                describe_key(key),
                rebuilt_value.map_or_else(|| "没有这一条".to_string(), &describe_value),
                walked_value.map_or_else(|| "没有这一条".to_string(), &describe_value)
            )),
        })
        .collect()
}

/// 实例表沿最新根的链读不全时 I-1.13 的不适用理由：已发布谓词的输入不全，扫描方向滤不出哪些单元已发布。
pub const SCAN_REBUILD_INSTANCE_TABLE_NOT_READ_IN_FULL: &str =
    "最新根的实例表沿链读不全：已发布谓词的输入不全，扫描重建滤不出哪些单元已发布";

/// I-1.13 比对之前查的那几条：这个镜像上它们里有一条已判违例，就是遍历方向的索引本身已被判损坏——
/// 映射 key 与单元头不符、extent 指针的出生树与单元头不符、引用了判不成已发布的单元、同一版里一个单元被引用两次——两个方向对不上
/// 是那一处损坏带出来的，I-1.13 报不适用、理由写明是哪一条红在哪，不另报一条违例。按表的次序取第一条违例了的。
/// 只收在必红世界 ①（映射条目落点挪槽：另红 I-2.1、I-3.1、I-3.11、I-4.8、I-7.4）与 ②（数据单元两份副本的对象 key 清零：另红 I-2.1、
/// I-4.8、I-7.2、I-7.4）上都不红的几条（在 `scan_rebuild_agrees_with_the_walked_index.rs` 那两条用例的镜像上现查），
/// 免得把两个必红世界一起判成不适用。I-9.34 是加进来的（那两个必红世界里没有退役记录，它不红）。
/// I-9.39 是截断字段那一批照 I-9.34 的先例加进来的（那两个必红世界里没有截断字段，它不红）。
pub const INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED: [(&str, &str); 7] = [
    (
        MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
        "遍历方向的中央映射已判损坏：I-1.11（映射 key 与单元头相符） 违例，两个方向对不上是那一条映射 key 带出来的，扫描重建不另比",
    ),
    (
        POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
        "遍历方向的指针已判损坏：I-1.12（指针出生树等于被指单元头） 违例，两个方向对不上是那一条指针带出来的，扫描重建不另比",
    ),
    (
        "I-4.2",
        "遍历方向引用了判不成已发布的单元：I-4.2（无被引用未提交块） 违例，扫描方向按已发布谓词不认领它，两个方向对不上是那一处带出来的，扫描重建不另比",
    ),
    (
        "I-5.1",
        "遍历方向的 extent 指针已判损坏：I-5.1（物理范围不重叠） 违例，同一版里一个单元被两处引用，两个方向对不上是那一处带出来的，扫描重建不另比",
    ),
    // 退役树与这一版 inode 树点名同一片叶容器、扫描方向看不见的那一形（叶死后没再写过一版：扫描重建按退役记录不认领它，遍历方向照样读得到）：
    // I-9.34 红时 I-1.13 报不适用并点名 I-9.34。
    (
        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
        "遍历方向的 inode 树与退役树已判对不上：I-9.34（谱系上的退役记录不指本版活叶） 违例，扫描重建按退役记录不认领那片叶、遍历方向照样读得到，两个方向对不上是那一处带出来的，扫描重建不另比",
    ),
    // 退役记录不合法（被退役身份的类型不是 2、容器出生代不小于死亡 txg 这类）：扫描重建按退役记录的身份跳过已退役的叶容器，记录坏了它就不跳、
    // 把那片死叶的旧版认成现行，两个方向对不上是那一条退役记录带出来的（照 I-9.34 那一行的先例；删除中的对象只对它自己不比之后——判决
    // `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M5——这一形不再被「删除中」整张报不适用盖住，交主 agent 认）。
    (
        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
        "退役树里有一条退役记录已判不合法：I-9.32（退役记录合法） 违例，扫描重建按退役记录跳过已退役的叶容器，记录坏了就把死叶的旧版认成现行，两个方向对不上是那一条退役记录带出来的，扫描重建不另比",
    ),
    // 截断字段之外还有出生早于 T 的活单元：扫描重建按读法 Z 第四条判死它、
    // 遍历方向照样读得到，两个方向对不上是那一条记录的截断字段带出来的（照 I-9.34 那一行的先例，交主 agent 认）。
    (
        LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
        "遍历方向的 extent 树与 inode 记录的截断字段已判对不上：I-9.39（截断字段之外的活单元不早于它） 违例，扫描重建按读法 Z 第四条不认领那个单元、遍历方向照样读得到，两个方向对不上是那一处带出来的，扫描重建不另比",
    ),
];

/// I-1.13 比对时算删除中的号：扫描重建那一侧
/// 墓碑压着、记录还在的号（`records_of_the_tombstoned_objects`）里，遍历方向那一侧也有它的记录、两条记录的对象出生代相同、nlink 都是
/// [`NLINK_OF_AN_INODE_RECORD_BEING_DELETED`] 的那几个（D8（核心索引结构） 已定项 16 第 1 条意图把记录写成 nlink = 0，第 3 条删除中的号集合是
/// inode 树里 nlink = 0 的号集合）。只看两侧的记录原样。
fn inodes_being_deleted_on_both_sides(
    records_of_the_tombstoned_objects: &BTreeMap<u64, Vec<u8>>,
    walked_inode_records: &BTreeMap<u64, Vec<u8>>,
) -> BTreeSet<u64> {
    records_of_the_tombstoned_objects
        .iter()
        .filter(|(inode, rebuilt_record)| {
            walked_inode_records
                .get(inode)
                .is_some_and(|walked_record| {
                    read_u64(walked_record, INODE_RECORD_OBJECT_BIRTH_OFFSET)
                        == read_u64(rebuilt_record, INODE_RECORD_OBJECT_BIRTH_OFFSET)
                        && read_u32(rebuilt_record, INODE_RECORD_NLINK_OFFSET)
                            == NLINK_OF_AN_INODE_RECORD_BEING_DELETED
                        && read_u32(walked_record, INODE_RECORD_NLINK_OFFSET)
                            == NLINK_OF_AN_INODE_RECORD_BEING_DELETED
                })
        })
        .map(|(inode, _)| *inode)
        .collect()
}

/// I-1.13（扫描重建与遍历方向的索引相同）：扫描方向扫出的码 1 / 码 3 已发布单元（与 I-1.8 那一遍同一份扫描结果，已发布谓词同取最新根
/// 指着的实例表）交给 `rebuild` 模块重建三张表，与遍历方向从最新根读出的三张表逐项比。判一次：三张表都逐项相同成立，否则违例，
/// 说明里每张对不上的表写差几处与第一处（表名、key、两边的值）。实例表读不全、遍历方向的索引已被别的不变量判损坏
/// （[`INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`]）、遍历方向那三张表读不全、扫描重建停在条款没写之处
/// （`rebuild::ScanRebuildNotDecided`）报不适用并带理由。只读：这一遍不写盘。
pub(super) fn judge_scan_rebuild_agrees_with_the_walked_index(
    reader: &dyn ImageReader,
    newest_root_record: &[u8],
    content_units: &[ScannedContentUnit],
    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
    index_cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    match abandoned_roots_filtering {
        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {}
        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
            judgements.not_applicable(
                SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
                SCAN_REBUILD_INSTANCE_TABLE_NOT_READ_IN_FULL,
            );
            return;
        }
    }
    if let Some((_, reason)) = INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED
        .iter()
        .find(|(invariant, _)| judgements.violation_count(invariant) > 0)
    {
        judgements.not_applicable(SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX, reason);
        return;
    }
    let mut walked =
        match index_tables_walked_from_the_newest_root(reader, newest_root_record, index_cache) {
            Ok(walked) => walked,
            Err(table_not_read_in_full) => {
                judgements.not_applicable(
                    SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
                    table_not_read_in_full.reason(),
                );
                return;
            }
        };
    let published_copies: Vec<PublishedUnitCopy> = content_units
        .iter()
        .map(|unit| PublishedUnitCopy {
            placement: UnitPlacement {
                device: unit.device,
                slot: unit.slot,
            },
            plain_header: unit.plain_header.clone(),
        })
        .collect();
    let rebuilt = choose_current_versions(&published_copies).and_then(|current| {
        let container_copies: BTreeMap<ObjectIdentity, Vec<Vec<u8>>> = current
            .packed_containers_whose_records_are_read()
            .into_iter()
            .map(|(identity, placements)| {
                let copies: Vec<Vec<u8>> = placements
                    .iter()
                    .filter_map(|placement| {
                        reader.read(
                            placement.device,
                            placement.slot * SLOT_BYTES,
                            data_unit_bytes(),
                        )
                    })
                    .collect();
                (identity, copies)
            })
            .collect();
        rebuild_index_tables(&current, &container_copies)
    });
    let mut rebuilt = match rebuilt {
        Ok(rebuilt) => rebuilt,
        Err(not_decided) => {
            judgements.not_applicable(
                SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
                not_decided.reason(),
            );
            return;
        }
    };
    // 删除中的对象只对它自己两侧都不比，删除中由两侧一起认
    // 算的，两侧都去掉它的记录、它在每个 inode 的数据单元集合里的每一项与那几项的映射条目。不算的，扫描重建那条记录放回它那一侧的
    // inode 记录集合、两侧照比：它的数据单元扫描重建那一侧按墓碑不认领，遍历方向照收，对不上就红。别的对象照比。
    let inodes_being_deleted = inodes_being_deleted_on_both_sides(
        &rebuilt.records_of_the_tombstoned_objects,
        &walked.inode_records,
    );
    for (inode, record) in &rebuilt.records_of_the_tombstoned_objects {
        if inodes_being_deleted.contains(inode) {
            continue;
        }
        let record_already_there = rebuilt.inode_records.insert(*inode, record.clone());
        assert!(
            record_already_there.is_none(),
            "墓碑压着的对象的记录是从扫描重建的 inode 记录集合里挪出来的，那张表里不会还有同号的一条（inode {inode}）"
        );
    }
    for inode in &inodes_being_deleted {
        walked.inode_records.remove(inode);
    }
    let mapping_keys_of_the_data_units_being_deleted: Vec<CentralMappingKey> = walked
        .data_units_of_each_inode
        .iter()
        .filter(|(position, _)| inodes_being_deleted.contains(&position.inode))
        .map(|(_, key)| key.clone())
        .collect();
    walked
        .data_units_of_each_inode
        .retain(|position, _| !inodes_being_deleted.contains(&position.inode));
    for key in &mapping_keys_of_the_data_units_being_deleted {
        walked.central_mapping.remove(key);
    }
    let tables = [
        DifferencesInOneTable {
            table: IndexTableOfTheScanRebuild::CentralMapping,
            differences: differences_in_the_central_mapping(
                reader,
                &rebuilt.central_mapping,
                &walked.central_mapping,
            ),
        },
        DifferencesInOneTable {
            table: IndexTableOfTheScanRebuild::DataUnitsOfEachInode,
            differences: differences_in_a_table_of_values(
                &rebuilt.data_units_of_each_inode,
                &walked.data_units_of_each_inode,
                |position| {
                    format!(
                        "（inode {}、偏移 {}）",
                        position.inode, position.offset_in_bytes
                    )
                },
                CentralMappingKey::describe,
            ),
        },
        DifferencesInOneTable {
            table: IndexTableOfTheScanRebuild::InodeRecords,
            differences: differences_in_a_table_of_values(
                &rebuilt.inode_records,
                &walked.inode_records,
                |inode| format!("inode {inode}"),
                |record| describe_inode_record(record),
            ),
        },
    ];
    let every_table_agrees = tables.iter().all(|table| table.differences.is_empty());
    judgements.judge(
        SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
        every_table_agrees,
        || {
            let tables_that_differ: Vec<String> = tables
                .iter()
                .filter(|table| !table.differences.is_empty())
                .map(|table| {
                    format!(
                        "「{}」差 {} 处，第一处 {}",
                        table.table.name(),
                        table.differences.len(),
                        table.differences[0]
                    )
                })
                .collect();
            format!(
                "扫描重建的三张表与遍历方向从最新根读出的不同：{}",
                tables_that_differ.join("；")
            )
        },
    );
}
