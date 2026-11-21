//! 各版的中央映射与分配记录对照：I-3.15（未释放记录罩住的单元由映射指在这一槽）、
//! I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数） 与 I-1.14（同一映射 key 在各版里的单元逐字节相同）。

use super::*;

/// 一版自己的那几条根指针：根环里的根取根记录（树表 36、实例表 170、中央映射根 256、树表 0 条那一版的分配记录树根 342），
/// 由记录施加出来的那一版取记录新根段里的同几条（D23（journal 的角色与格式） 已定项 15）。I-3.15、I-5.6、I-4.4、I-1.14 按版本各读一遍
pub(super) struct VersionWithItsRootPointers<'record> {
    pub(super) name: String,
    pub(super) checkpoint_txg: u64,
    pub(super) tree_table_pointer: &'record [u8],
    pub(super) instance_table_pointer: &'record [u8],
    pub(super) mapping_root_pointer: &'record [u8],
    pub(super) allocation_record_tree_root_held_outside_the_tree_table: &'record [u8],
}

/// 一版读不全、按版本判的那几条（I-3.15、I-5.6、I-4.4、I-1.14）判不了那一版：报「射程里有一部分判不了」的理由。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VersionReadingGap {
    TreeTablePointerAllZero,
    TreeTableUnreadable,
    TreeTableEntryNarrowerThanTheFieldTable,
    AllocationRecordTreeUnreadable,
    CentralMappingTreeUnreadable,
    InstanceTableChainUnreadable,
    LoggedOperationsTreeUnreadable,
}

impl VersionReadingGap {
    /// 把这一样记成 `invariant` 射程里判不了的一部分：这一版在射程里、在镜像上，它要的那一样读不出、读不全或配不上，报部分没判、
    /// 逐条列出没判到的是哪一样（[`Judgements::part_of_the_range_left_unjudged`]；C701（别的判法静默跳过读不出的部分）
    /// 「一律报部分没判」，不报不适用）。
    pub(super) fn note_as_a_part_of_the_range_left_unjudged(
        self,
        judgements: &mut Judgements,
        invariant: &'static str,
    ) {
        judgements.part_of_the_range_left_unjudged(invariant, self.reason().to_string());
    }

    pub(super) fn reason(self) -> &'static str {
        match self {
            VersionReadingGap::TreeTablePointerAllZero => {
                "回退候选集里有一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的分配记录、中央映射、logged_ops 判不了"
            }
            VersionReadingGap::TreeTableUnreadable => {
                "回退候选集里有一版树表读不出：那一版的分配记录、中央映射、logged_ops 判不了"
            }
            VersionReadingGap::TreeTableEntryNarrowerThanTheFieldTable => {
                "回退候选集里有一版树表有条目比字段表短（那几条不解字段）：那一版的分配记录、logged_ops 数不全"
            }
            VersionReadingGap::AllocationRecordTreeUnreadable => {
                "回退候选集里有一版的分配记录树读不全：那一版的未释放记录数不全"
            }
            VersionReadingGap::CentralMappingTreeUnreadable => {
                "回退候选集里有一版的中央映射树读不全：那一版哪把 key 落在哪认不全"
            }
            VersionReadingGap::InstanceTableChainUnreadable => {
                "回退候选集里有一版的实例表链读不到末片：那一版被父指针直接指着的实例表各片认不全"
            }
            VersionReadingGap::LoggedOperationsTreeUnreadable => {
                "回退候选集里有一版的 logged_ops 树读不全：那一版的意图条目数不全"
            }
        }
    }
}

/// 一版树表里的条目，只读不判（条目宽是节点头里的一个字段：有比字段表短的就不解，交回缺口，那一格由 I-1.7 / I-1.1 说话）。
pub(super) fn tree_table_entries_of_a_version_without_judging(
    reader: &dyn ImageReader,
    tree_table_pointer: &[u8],
    cache: &mut IndexNodeCache,
) -> Result<Vec<Vec<u8>>, VersionReadingGap> {
    let pointer = parse_node_pointer(tree_table_pointer);
    if pointer.all_zero {
        return Err(VersionReadingGap::TreeTablePointerAllZero);
    }
    let tree_table = read_index_node_without_judging(
        reader,
        &pointer,
        PositionOfASecondaryRead::ThePointerItself,
        cache,
    )
    .ok_or(VersionReadingGap::TreeTableUnreadable)?;
    if tree_table
        .entries
        .iter()
        .any(|entry| entry.len() < tree_table_entry_bytes())
    {
        return Err(VersionReadingGap::TreeTableEntryNarrowerThanTheFieldTable);
    }
    Ok(tree_table.entries)
}

/// 一版的账（[`allocation_records_of_a_version_without_judging`] 交回的）：分配记录，与树表 0 条那一版由根记录直接持有的那一棵分配记录树
/// 的每个节点（树 ID 0，同树表单元与实例表单元由根记录独占持有，不进映射：根之下的节点只有这棵树里的父节点指着）。
pub(super) struct AllocationRecordsOfAVersion {
    pub(super) records: Vec<AllocationRecordView>,
    pub(super) nodes_of_the_tree_the_root_record_holds: Vec<PointerView>,
    /// 读到的分配记录树有几棵（根记录持有的那一棵、树表里种类 3 那几条指着的）：0 说明这一版的账一棵树都没有，`records` 空不等于
    /// 「没有未释放的记录」（树表缺种类 3 那一条的坏镜像就是这样，I-9.16 判它）。
    pub(super) allocation_record_trees_read: usize,
}

/// 一版的分配记录（这一版的账）：树表里种类 3 那一条指着的那一棵，加树表 0 条那一版由根记录（或记录新根段）直接持有的那一棵
/// （C512（树表 0 条的一版上被换下的实例表记在哪没有条款））。只读不判；读不全交回缺口。
pub(super) fn allocation_records_of_a_version_without_judging(
    reader: &dyn ImageReader,
    version: &VersionWithItsRootPointers<'_>,
    tree_table_entries: &[Vec<u8>],
    cache: &mut IndexNodeCache,
) -> Result<AllocationRecordsOfAVersion, VersionReadingGap> {
    let mut account = AllocationRecordsOfAVersion {
        records: Vec::new(),
        nodes_of_the_tree_the_root_record_holds: Vec::new(),
        allocation_record_trees_read: 0,
    };
    // 分配记录树的节点不在豁免三类里：按这一版自己的中央映射认位置（树表 0 条那一版由根记录持有的那一棵不进映射，查不到、照指针读）。
    let central_mapping = CentralMappingLocationsOfAVersion::read_without_judging(
        reader,
        &parse_node_pointer(version.mapping_root_pointer),
        cache,
    );
    let root_the_root_record_holds =
        parse_node_pointer(version.allocation_record_tree_root_held_outside_the_tree_table);
    if !root_the_root_record_holds.all_zero {
        let (node_pointers, records) = allocation_record_tree_without_judging(
            reader,
            &root_the_root_record_holds,
            &central_mapping,
            cache,
        )
        .ok_or(VersionReadingGap::AllocationRecordTreeUnreadable)?;
        account.nodes_of_the_tree_the_root_record_holds = node_pointers;
        account.records.extend(records);
        account.allocation_record_trees_read += 1;
    }
    for allocation_root in tree_table_entries
        .iter()
        .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_ALLOCATION)
        .map(|entry| parse_node_pointer(&entry[14..100]))
        .filter(|pointer| !pointer.all_zero)
    {
        let (_, records_of_this_tree) = allocation_record_tree_without_judging(
            reader,
            &allocation_root,
            &central_mapping,
            cache,
        )
        .ok_or(VersionReadingGap::AllocationRecordTreeUnreadable)?;
        account.records.extend(records_of_this_tree);
        account.allocation_record_trees_read += 1;
    }
    Ok(account)
}

/// 一版中央映射树根之下各层的节点指针（只有父条目指着它们，D19（块指针的结构与宽度预算） 已定项 8 的自举豁免）与叶里的全部条目，只读不判。
struct CentralMappingTreeOfAVersion {
    node_pointers_below_the_root: Vec<PointerView>,
    leaf_entries: Vec<Vec<u8>>,
}

/// 读一版的中央映射树，只读不判：根指针全零（树表 0 条的一版）交回空的；节点读不出、key 宽不是 27、层级不是父条目给的那一层、
/// 条目窄于走读要的、子指针全零，都交回缺口（判定归主走读的 I-1.1、I-2.1）。
/// 迭代上界：每个节点只经它父节点的一条条目进栈一次，层级每下一层减一（层级不对的整棵交回缺口），层级 0 不再往下。
fn central_mapping_tree_of_a_version_without_judging(
    reader: &dyn ImageReader,
    mapping_root_pointer: &PointerView,
    cache: &mut IndexNodeCache,
) -> Result<CentralMappingTreeOfAVersion, VersionReadingGap> {
    let mut tree = CentralMappingTreeOfAVersion {
        node_pointers_below_the_root: Vec::new(),
        leaf_entries: Vec::new(),
    };
    if mapping_root_pointer.all_zero {
        return Ok(tree);
    }
    let key_width = KEY_SCHEMA_MAPPING.width();
    let mut pending: Vec<(PointerView, Option<u8>)> = vec![(*mapping_root_pointer, None)];
    while let Some((pointer, level_the_parent_names)) = pending.pop() {
        let node = read_index_node_without_judging(
            reader,
            &pointer,
            PositionOfASecondaryRead::ThePointerItself,
            cache,
        )
        .ok_or(VersionReadingGap::CentralMappingTreeUnreadable)?;
        if node.key_width != key_width
            || level_the_parent_names.is_some_and(|level| level != node.level)
        {
            return Err(VersionReadingGap::CentralMappingTreeUnreadable);
        }
        let Some(child_level) = node.level.checked_sub(1) else {
            if node.entry_width < mapping_entry_bytes() {
                return Err(VersionReadingGap::CentralMappingTreeUnreadable);
            }
            tree.leaf_entries.extend(node.entries.iter().cloned());
            continue;
        };
        if node.entry_width < internal_entry_bytes(KEY_SCHEMA_MAPPING) {
            return Err(VersionReadingGap::CentralMappingTreeUnreadable);
        }
        for entry in &node.entries {
            let child = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
            if child.all_zero {
                return Err(VersionReadingGap::CentralMappingTreeUnreadable);
            }
            tree.node_pointers_below_the_root.push(child);
            pending.push((child, Some(child_level)));
        }
    }
    Ok(tree)
}

/// 一把中央映射 key 与一个单元头相符（I-3.15 反查那一步）：类标签相同；头里的诞生代号、写序实例与尾段（码 1 事务号 6 字节、码 2 / 码 3 出生序号 4 字节）
/// 与 key 里的出生 txg、实例代号、尾段相同；码 1 / 码 3 头里偏移 43 的出生树与 key 里的出生树相同。码 2 的出生树取哪个字段还开着（C289），
/// 不比——与 I-1.11（映射 key 与单元头相符） 那一判同一个读法（[`Walk::walk_central_mapping_entries`]）。
fn mapping_key_matches_the_unit_header(key: &[u8], header: &[u8]) -> bool {
    if key[MAPPING_KEY_UNIT_CLASS_OFFSET] != header[UNIT_HEADER_CLASS_TAG_OFFSET] {
        return false;
    }
    let Some(identity) = birth_identity_in_the_header(header) else {
        return false;
    };
    let key_tail = match identity.tail {
        TailOfTheBirthIdentity::TransactionNumber(_) => {
            read_six_byte_unsigned(key, MAPPING_KEY_TAIL_OFFSET)
        }
        TailOfTheBirthIdentity::BirthSequence(_) => {
            u64::from(read_u32(key, MAPPING_KEY_TAIL_OFFSET))
        }
    };
    let birth_identity_matches = identity.write_order.birth_txg()
        == read_u64(key, MAPPING_KEY_BIRTH_TXG_OFFSET)
        && identity.write_order.instance() == read_u32(key, MAPPING_KEY_INSTANCE_OFFSET)
        && identity.tail.birth_identity_tail_number() == key_tail;
    let birth_tree_matches = match birth_tree_in_the_header_as_the_mapping_key_names_it(header) {
        Some(BirthTreeInTheHeader::AtOffsetFortyThree(header_birth_tree)) => {
            header_birth_tree == read_u64(key, MAPPING_KEY_BIRTH_TREE_OFFSET)
        }
        Some(BirthTreeInTheHeader::FieldStillOpenUnderC289) => true,
        None => false,
    };
    birth_identity_matches && birth_tree_matches
}

/// 一条未释放记录在 I-3.15 那一遍里对不上的样子（判红时写进说明）。
struct AllocationRecordOffTheCentralMapping {
    pub(super) device: u32,
    detail: String,
}

/// I-3.15（未释放记录罩住的单元由映射指在这一槽，N2 反查；「新判：N2 反查」，
/// G9 的「已死」「不一致」两类判红、「钉住」「隔离」两类不红）：每一版各判（[`VersionWithItsRootPointers`]），几版照抄同一组根指针的只判一次。
/// 一版里：① 根记录直接持有的那几样（树表、实例表第 0 片、映射根、树表 0 条那一版的分配记录树根）与映射树根之下各层的节点、实例表第 1 片起
/// 各片——它们的落点只有根记录或父指针指着（D19 已定项 8 的自举豁免）——罩住它们的记录成立；② 别的未释放记录，读它起点槽上的单元头，
/// 头用不了的没有对象（读不出本身归 I-1.1 与 I-2.1）；头里的出生身份与这一版中央映射里某把 key 相符（[`mapping_key_matches_the_unit_header`]）、
/// 且那把 key 的位置条目指着这一槽，成立；③ 对不上的按单元（起点槽、跨度）归组，这一版这一单元每块盘上的未释放记录都带隔离记号的是隔离的单元，
/// 不判（D19（块指针的结构与宽度预算） 已定项 5「池级 checker 只认记号」，变体 `mark_only`；不看副本，
/// [`every_copy_of_the_unit_carries_the_quarantine_mark`]），其余每条判红。
pub(super) fn judge_unreleased_allocation_records_against_the_central_mapping(
    reader: &dyn ImageReader,
    versions: &[VersionWithItsRootPointers<'_>],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut judged_records = 0u64;
    let mut root_pointers_already_judged: BTreeSet<[&[u8]; 4]> = BTreeSet::new();
    // 同一张镜像上同一个 (盘, 槽) 读出的单元头每一版都一样：几版的记录罩住同一个单元时只读一次（崩溃点重放每个崩溃状态都跑这一遍）。
    let mut usable_unit_headers: BTreeMap<(u32, u64), Option<Vec<u8>>> = BTreeMap::new();
    for version in versions {
        if !root_pointers_already_judged.insert([
            version.tree_table_pointer,
            version.instance_table_pointer,
            version.mapping_root_pointer,
            version.allocation_record_tree_root_held_outside_the_tree_table,
        ]) {
            continue;
        }
        let tree_table_entries = match tree_table_entries_of_a_version_without_judging(
            reader,
            version.tree_table_pointer,
            cache,
        ) {
            Ok(entries) => entries,
            Err(gap) => {
                gap.note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
                );
                continue;
            }
        };
        let mapping_root = parse_node_pointer(version.mapping_root_pointer);
        let instance_table_first_page = parse_node_pointer(version.instance_table_pointer);
        let readings = allocation_records_of_a_version_without_judging(
            reader,
            version,
            &tree_table_entries,
            cache,
        )
        .and_then(|account| {
            central_mapping_tree_of_a_version_without_judging(reader, &mapping_root, cache)
                .map(|mapping| (account, mapping))
        });
        let (account, mapping) = match readings {
            Ok(readings) => readings,
            Err(gap) => {
                gap.note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
                );
                continue;
            }
        };
        let instance_table_pages = if instance_table_first_page.all_zero {
            InstanceTablePagesAfterTheFirst {
                page_pointers: Vec::new(),
                is_complete: true,
            }
        } else {
            instance_table_pages_after_the_first_without_judging(reader, &instance_table_first_page)
        };
        if !instance_table_pages.is_complete {
            VersionReadingGap::InstanceTableChainUnreadable
                .note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
                );
            continue;
        }
        let pointed_directly_by_the_root_or_a_parent: BTreeSet<(u32, u64)> = [
            parse_node_pointer(version.tree_table_pointer),
            instance_table_first_page,
            mapping_root,
            parse_node_pointer(version.allocation_record_tree_root_held_outside_the_tree_table),
        ]
        .iter()
        .chain(mapping.node_pointers_below_the_root.iter())
        .chain(instance_table_pages.page_pointers.iter())
        .chain(account.nodes_of_the_tree_the_root_record_holds.iter())
        .filter(|pointer| !pointer.all_zero)
        .flat_map(|pointer| {
            pointer
                .locations
                .iter()
                .map(|location| (location.device, location.slot))
        })
        .collect();
        let mut keys_at_each_placement: BTreeMap<(u32, u64), Vec<&[u8]>> = BTreeMap::new();
        for entry in &mapping.leaf_entries {
            for location in locations_of_a_mapping_entry(entry).locations {
                keys_at_each_placement
                    .entry((location.device, location.slot))
                    .or_default()
                    .push(&entry[..mapping_key_bytes()]);
            }
        }
        let mut records_off_the_mapping_by_unit: BTreeMap<
            (u64, u64),
            Vec<AllocationRecordOffTheCentralMapping>,
        > = BTreeMap::new();
        for record in account.records.iter().filter(|record| !record.is_released) {
            let placement = (record.device, record.slot);
            if pointed_directly_by_the_root_or_a_parent.contains(&placement) {
                judged_records += 1;
                judgements.judge(
                    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
                    true,
                    String::new,
                );
                continue;
            }
            let Some(header) = usable_unit_headers
                .entry(placement)
                .or_insert_with(|| {
                    reader
                        .read(
                            record.device,
                            record.slot * SLOT_BYTES,
                            UNIT_HEADER_SCAN_BYTES,
                        )
                        .filter(|header| birth_txg_in_a_usable_unit_header(header).is_some())
                })
                .clone()
            else {
                continue;
            };
            let keys_here = keys_at_each_placement
                .get(&placement)
                .map_or(&[][..], Vec::as_slice);
            if keys_here
                .iter()
                .any(|key| mapping_key_matches_the_unit_header(key, &header))
            {
                judged_records += 1;
                judgements.judge(
                    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
                    true,
                    String::new,
                );
                continue;
            }
            let keys_named_here: Vec<String> = keys_here
                .iter()
                .map(|key| CentralMappingKey(key.to_vec()).describe())
                .collect();
            let matching_keys_placed_elsewhere: Vec<String> = mapping
                .leaf_entries
                .iter()
                .filter(|entry| {
                    mapping_key_matches_the_unit_header(&entry[..mapping_key_bytes()], &header)
                })
                .map(|entry| {
                    let placements: Vec<(u32, u64)> = locations_of_a_mapping_entry(entry)
                        .locations
                        .iter()
                        .map(|location| (location.device, location.slot))
                        .collect();
                    format!(
                        "{} 在 {placements:?}",
                        CentralMappingKey(entry[..mapping_key_bytes()].to_vec()).describe()
                    )
                })
                .collect();
            let header_identity = birth_identity_in_the_header(&header).map_or_else(
                || {
                    format!(
                        "码 {}（出生身份解不出）",
                        header[UNIT_HEADER_CLASS_TAG_OFFSET]
                    )
                },
                |identity| {
                    format!(
                        "码 {}、诞生代号 {}、写序实例 {}、{} {}",
                        header[UNIT_HEADER_CLASS_TAG_OFFSET],
                        identity.write_order.birth_txg(),
                        identity.write_order.instance(),
                        identity.tail.name(),
                        identity.tail.birth_identity_tail_number()
                    )
                },
            );
            records_off_the_mapping_by_unit
                .entry((record.slot, record.span_slots))
                .or_default()
                .push(AllocationRecordOffTheCentralMapping {
                    device: record.device,
                    detail: format!(
                        "{}：盘 {} 槽 {} 的分配记录（未释放、跨 {} 槽）罩住的单元（{header_identity}）在这一版的中央映射里没有一把相符的 key 指着这一槽（指着这一槽的 key：{}；与它相符的 key 落在：{}）",
                        version.name,
                        record.device,
                        record.slot,
                        record.span_slots,
                        if keys_named_here.is_empty() {
                            "没有".to_string()
                        } else {
                            keys_named_here.join("、")
                        },
                        if matching_keys_placed_elsewhere.is_empty() {
                            "没有".to_string()
                        } else {
                            matching_keys_placed_elsewhere.join("、")
                        }
                    ),
                });
        }
        for ((slot, span_slots), records_of_the_unit) in records_off_the_mapping_by_unit {
            // 隔离的单元只认记号（D19（块指针的结构与宽度预算） 已定项 5「池级 checker 只认记号」）：这一版这一单元
            // （起点槽、跨度）每块盘上的未释放记录都带隔离记号就不判，不看副本、不看这一版映射里有没有 key 指着这一槽；有一条不带的照判。
            let copies_of_the_unit: Vec<AllocationRecordView> = account
                .records
                .iter()
                .filter(|record| {
                    !record.is_released && record.slot == slot && record.span_slots == span_slots
                })
                .copied()
                .collect();
            if every_copy_of_the_unit_carries_the_quarantine_mark(&copies_of_the_unit) {
                continue;
            }
            for record in records_of_the_unit {
                judged_records += 1;
                judgements.judge_naming_the_slots(
                    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
                    false,
                    || vec![(record.device, slot)],
                    || record.detail,
                );
            }
        }
    }
    if judged_records == 0 {
        judgements.not_applicable(
            UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
            "回退候选集里没有一版的分配记录树走得到未释放的记录（或那几条记录罩住的槽上单元头都用不了、都是隔离的单元）",
        );
    }
}

/// 起点在 (盘, 槽) 的单元头用得了（[`birth_txg_in_a_usable_unit_header`]：头校验和对、类标签是码 1 / 2 / 3）时，那一类与它占的槽数
/// （[`slots_spanned_by_a_unit_of_the_class`]）；读不出、头用不了交回 `None`。I-3.18 比跨度段、I-3.1 认 I-3.18 判红的记录用的同一个数。
pub(super) fn class_and_slots_of_the_unit_whose_usable_header_starts_at(
    reader: &dyn ImageReader,
    device: u32,
    slot: u64,
) -> Option<(MappingKeyUnitClass, u64)> {
    let header = reader
        .read(device, slot * SLOT_BYTES, UNIT_HEADER_SCAN_BYTES)
        .filter(|header| birth_txg_in_a_usable_unit_header(header).is_some())?;
    let class = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);
    slots_spanned_by_a_unit_of_the_class(class)
        .map(|slots_of_the_class| (class, slots_of_the_class))
}

/// I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数，C411（分配记录的跨度段与单元类别不符没检查））：每一版各判（[`VersionWithItsRootPointers`]，
/// 与 I-3.15 同一份版本清单），几版照抄同一棵分配记录树（树表、映射根、根记录持有的那一棵三样相同）的只判一次。一版里每条未释放的记录，
/// 读它起点槽上的单元头：头用得了（[`birth_txg_in_a_usable_unit_header`]：头校验和对、类标签是码 1 / 2 / 3）时，跨度段要等于那一类占的槽数
/// （[`slots_spanned_by_a_unit_of_the_class`]，D18（块里携带什么信息） 已定项 11 的登记表）；头用不了的没有对象。跨度段比那一类少时，
/// 单元后面那一槽在盘上没有归属、可被重新分配（静默覆盖）——按跨度段求和的 I-3.1 与 I-5.4 都看不见。
/// 已释放的记录不判：落点可以已被别的类的单元合法复用，罩住的就不是它记的那个单元了。
pub(super) fn judge_allocation_record_spans_against_the_classes_of_the_units_they_cover(
    reader: &dyn ImageReader,
    versions: &[VersionWithItsRootPointers<'_>],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    let mut judged_records = 0u64;
    let mut allocation_record_trees_already_judged: BTreeSet<[&[u8]; 3]> = BTreeSet::new();
    // 同一张镜像上同一个 (盘, 槽) 读出的单元头每一版都一样：几版的记录罩住同一个单元时只读一次。
    let mut classes_of_the_units: BTreeMap<(u32, u64), Option<(MappingKeyUnitClass, u64)>> =
        BTreeMap::new();
    for version in versions {
        if !allocation_record_trees_already_judged.insert([
            version.tree_table_pointer,
            version.mapping_root_pointer,
            version.allocation_record_tree_root_held_outside_the_tree_table,
        ]) {
            continue;
        }
        let account = match tree_table_entries_of_a_version_without_judging(
            reader,
            version.tree_table_pointer,
            cache,
        )
        .and_then(|tree_table_entries| {
            allocation_records_of_a_version_without_judging(
                reader,
                version,
                &tree_table_entries,
                cache,
            )
        }) {
            Ok(account) => account,
            Err(gap) => {
                gap.note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
                );
                continue;
            }
        };
        for record in account.records.iter().filter(|record| !record.is_released) {
            let Some((class, slots_of_the_class)) = *classes_of_the_units
                .entry((record.device, record.slot))
                .or_insert_with(|| {
                    class_and_slots_of_the_unit_whose_usable_header_starts_at(
                        reader,
                        record.device,
                        record.slot,
                    )
                })
            else {
                continue;
            };
            judged_records += 1;
            judgements.judge(
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
                record.span_slots == slots_of_the_class,
                || {
                    format!(
                        "{}：盘 {} 槽 {} 的分配记录（未释放）跨 {} 槽，而它罩住的单元头里是码 {}，那一类占 {slots_of_the_class} 槽",
                        version.name,
                        record.device,
                        record.slot,
                        record.span_slots,
                        class.tag()
                    )
                },
            );
        }
    }
    if judged_records == 0 {
        judgements.not_applicable(
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            NO_UNRELEASED_ALLOCATION_RECORD_COVERING_A_USABLE_UNIT_HEADER,
        );
    }
}

/// I-1.14（同一映射 key 在各版里的单元逐字节相同），N6、G7：
/// 回退候选集里每一版（连同由记录施加出来的那几版）的中央映射里每一把 key 落在哪两条位置条目上，按 (盘, 槽, 整单元校验和) 认；
/// 同一把 key 落在不止一组位置条目上时，每一组读它第一份按自己的校验和对得上的副本，与第一组读出来的那一份逐字节比。
/// 位置条目相同的那几版不读盘（同一个落点上就是同一份字节）；一组的两份都读不出的不比（读不出归 I-2.1）。
/// 几版照抄同一棵映射树时只读一次。一对都没比上时报不适用并带理由。
pub(super) fn judge_unit_bytes_of_each_mapping_key_across_the_versions(
    reader: &dyn ImageReader,
    versions: &[VersionWithItsRootPointers<'_>],
    cache: &mut IndexNodeCache,
    judgements: &mut Judgements,
) {
    type LocationsOfAMappingEntry = [(u32, u64, u32); 2];
    let mut versions_of_each_key_and_locations: BTreeMap<
        Vec<u8>,
        BTreeMap<LocationsOfAMappingEntry, String>,
    > = BTreeMap::new();
    let mut mapping_roots_already_read: BTreeSet<&[u8]> = BTreeSet::new();
    for version in versions {
        if !mapping_roots_already_read.insert(version.mapping_root_pointer) {
            continue;
        }
        let mapping = match central_mapping_tree_of_a_version_without_judging(
            reader,
            &parse_node_pointer(version.mapping_root_pointer),
            cache,
        ) {
            Ok(mapping) => mapping,
            Err(gap) => {
                gap.note_as_a_part_of_the_range_left_unjudged(
                    judgements,
                    UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
                );
                continue;
            }
        };
        for entry in &mapping.leaf_entries {
            let locations = locations_of_a_mapping_entry(entry)
                .locations
                .map(|location| (location.device, location.slot, location.checksum));
            versions_of_each_key_and_locations
                .entry(entry[..mapping_key_bytes()].to_vec())
                .or_default()
                .entry(locations)
                .or_insert_with(|| version.name.clone());
        }
    }
    let mut compared_pairs = 0u64;
    for (key, versions_of_each_locations) in &versions_of_each_key_and_locations {
        if versions_of_each_locations.len() < 2 {
            continue;
        }
        let Some(unit_bytes) =
            unit_bytes_of_the_class(MappingKeyUnitClass::of(key[MAPPING_KEY_UNIT_CLASS_OFFSET]))
        else {
            continue;
        };
        let mut first_copy: Option<(Vec<u8>, &String)> = None;
        for (locations, version_name) in versions_of_each_locations {
            let Some(content) = locations.iter().find_map(|(device, slot, checksum)| {
                reader
                    .read(*device, slot * SLOT_BYTES, unit_bytes)
                    .filter(|content| crc32_castagnoli_table(content) == *checksum)
            }) else {
                continue;
            };
            match &first_copy {
                None => first_copy = Some((content, version_name)),
                Some((first_content, first_version_name)) => {
                    compared_pairs += 1;
                    judgements.judge(
                        UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
                        content == *first_content,
                        || {
                            let first_differing_offset = content
                                .iter()
                                .zip(first_content.iter())
                                .position(|(byte, first_byte)| byte != first_byte)
                                .unwrap_or(0);
                            format!(
                                "中央映射 key {} 在{first_version_name}里指的那一份（{:?}）与在{version_name}里指的那一份（{:?}）字节不同，第一处不同在偏移 {first_differing_offset}",
                                CentralMappingKey(key.clone()).describe(),
                                versions_of_each_locations
                                    .iter()
                                    .find(|(_, name)| *name == *first_version_name)
                                    .map(|(first_locations, _)| first_locations),
                                locations
                            )
                        },
                    );
                }
            }
        }
    }
    if compared_pairs == 0 {
        judgements.not_applicable(
            UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
            NO_MAPPING_KEY_PLACED_DIFFERENTLY_IN_TWO_WALKED_VERSIONS,
        );
    }
}
