//! 冷走读到文件（[`walk_to_file`]、[`walk_to_file_in_the_unit_area_starting_at`]）与读只读快照的第一个文件。

use super::*;

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
    let mut deadlist_tree: Option<CodeTwoTreeReadFromDisk> = None;
    let mut allocation_tree: Option<AllocationRecordTreeReadFromDisk> = None;
    let mut extent_tree: Option<(TreeIdentifier, ExtentsOfTheFirstFile)> = None;
    for entry in &entries {
        tree_identifier_of_the_entry_is_below_the_watermark(entry, root)?;
        // 只读快照的两种条目（码 14、15）：它们的根由快照的读路径解（判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
        // 第二节 S2；这一批不解），冷走读读的是活头那一版的文件，跳过它们。
        if entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
            || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT
        {
            continue;
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
        // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：多层码 2 树，同记账树整棵读回来；
        // 它的节点与它的条目各占映射条目（条目留存的那几条，判决 S3 的 α），数映射条目时要它们。
        if entry.kind == TREE_KIND_DEADLIST {
            deadlist_tree = Some(read_code_two_tree(
                &entry.root,
                &MultiLevelCodeTwoTree::Deadlist.read_expectation(entry.tree),
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
        + usize::from(by_kind.contains_key(&TREE_KIND_LOGGED_OPERATIONS))
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
            deadlist_tree_nodes: deadlist_tree
                .as_ref()
                .map_or(0, |read| read.version.node_count()),
            mapping_entries_kept_for_the_snapshots: deadlist_tree
                .as_ref()
                .map_or(0, |read| read.leaf_entries_in_key_order.len()),
            inode_leaf_containers: roots.inode.entries.len(),
            data_units: roots.extent.data_units_of_every_file(),
            deletion_tree_nodes,
        },
    )?;

    // inode 树：根（码 2、层级 1）→ 内部条目 → 叶容器（码 3）→ 记录。
    inode_tree_root_level_is_one(&roots.inode)?;
    let mut inode_record: Option<InodeRecord> = None;
    let mut inode_numbers_read_back: Vec<u64> = Vec::new();
    let mut largest_inode_number_of_the_leaves_read = None;
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
        // 冷走读与可写挂载重建同一判：全部叶的记录按叶序严格升序（写者的前提），不收两片交叠的一版；分隔 key 那一句同判。
        largest_inode_number_of_the_leaves_read =
            inode_records_of_the_leaf_ascend_after_the_leaves_before(
                separator_key,
                largest_inode_number_of_the_leaves_read,
                &records,
            )?;
        for record in records {
            inode_numbers_read_back.push(record.inode);
            if record.inode == FIRST_INODE_NUMBER {
                inode_record = Some(record);
            }
        }
    }
    inode_tree_root_key_range_is_the_subtree_coverage(&roots.inode, inode_numbers_read_back)?;
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
    let data_units_within_the_size = data_unit_count_matches_the_inode_size(
        roots.extent.data_pointers.len(),
        roots.extent.tail,
        &inode_record,
    )?;
    let mut content: Vec<u8> =
        Vec::with_capacity(usize::try_from(inode_record.size).expect("文件长度装得进 usize"));
    // 迭代次数的上界是 size 之内的单元数（上面按标签核过）；跨轮携带的只有已经拼出来的内容，每一个提前出口都是交回一个错。
    for (position, pointer) in roots
        .extent
        .data_pointers
        .iter()
        .take(data_units_within_the_size)
        .enumerate()
    {
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
            DataUnitOfAFile {
                position,
                pointer,
                extent_tree: roots.extent_tree,
            },
            &inode_record,
            expected_filesystem_identifier,
        )?;
        content.extend_from_slice(payload);
    }
    Ok(Some(content_padded_with_the_hole_up_to_the_size(
        content,
        &inode_record,
    )))
}

/// 读一个只读快照里的第一个文件（[`read_the_first_file_of_a_snapshot`]）读出了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotFileReading {
    /// 这条根的树表里没有树号是那个号的码 14 条目。
    NoSnapshotWithThisTree,
    /// 快照那一刻没有第一个文件（inode 1 没有记录，或已在删除中、nlink = 0）。
    NoFile,
    /// 第一个文件的内容。
    File(Vec<u8>),
}

/// 读这条根的树表里只读快照 `snapshot`（码 14 的树号，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 5）指着的
/// 第一个文件：从它那一对码 14 / 15 条目的两条根指针往下读 inode 树与 extent 树，节点按根指针头部的出生树核（判决 S1「读快照指着的根按根指针头部
/// 的出生树比、不按条目自己的树号」），单元的位置提示读不出时经**这条根**的中央映射回退——快照引用、活头已换下的单元靠的就是留存在这里的
/// 映射条目（判决 S3 的 α）。判定与冷走读那几道相同（inode 树根层级 1、叶容器与条目对得上、全部叶的记录按叶序严格升序、单元个数对得上 size、
/// 数据单元按查找路径核）。
///
/// # Errors
/// 树表读不出、解不开、条目不合 I-9.16 或配不成对；快照的两棵树、叶容器或数据单元读不出（提示与映射都读不出）、解不开或判红
/// （两片 inode 叶交叠拒在 I-9.12、一片里不升序拒在 I-9.4，[`inode_records_of_the_leaf_ascend_after_the_leaves_before`]）。
pub fn read_the_first_file_of_a_snapshot(
    reader: &dyn PoolReader,
    root: &RootRecord,
    snapshot: TreeIdentifier,
) -> Result<SnapshotFileReading, RecoveryFailure> {
    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
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
    let entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;
    let Some(pair) = snapshot_tree_table_entries_paired_one_to_one(&entries)?
        .into_iter()
        .find(|pair| pair.inode_root.tree == snapshot)
    else {
        return Ok(SnapshotFileReading::NoSnapshotWithThisTree);
    };
    let central_mapping_tree = CentralMappingTreeReadOnFirstUse {
        reader,
        root,
        expected_filesystem_identifier,
        tree: OnceCell::new(),
    };
    let central_mapping_locations_of_key =
        |mapping_key: &[u8]| central_mapping_tree.locations_of_key(mapping_key);
    let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
    let extent_root_pointer = pair.extent_root.root;
    let extents = extents_of_the_first_file(read_extent_tree(
        &ExtentTreeReading {
            tree: extent_root_pointer.head.birth_tree,
            judgement: ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference,
            root,
            expected_filesystem_identifier,
        },
        &extent_root_pointer,
        &mut |pointer: &NodePointer| {
            read_mapped_tree_node_via_hint_then_central_mapping(
                reader,
                pointer,
                MappedTreeNodeClass::IndexNode,
                &central_mapping_locations_of_key,
                &mut stale_location_hint_hops_not_exposed_by_this_reader,
            )
        },
    )?)?;
    let inode_root_pointer = pair.inode_root.root;
    let inode_root_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
        reader,
        &inode_root_pointer,
        MappedTreeNodeClass::IndexNode,
        &central_mapping_locations_of_key,
        &mut stale_location_hint_hops_not_exposed_by_this_reader,
    )?;
    let inode_root = tree_root_checked_against_its_pointer(
        &inode_root_bytes,
        inode_root_pointer.head.birth_tree,
        key_width_for_kind(TREE_KIND_INODE).expect("inode 树的 key 宽登记过"),
        &inode_root_pointer,
        root,
        expected_filesystem_identifier,
    )?;
    inode_tree_root_level_is_one(&inode_root)?;
    let mut inode_record: Option<InodeRecord> = None;
    let mut inode_numbers_read_back: Vec<u64> = Vec::new();
    let mut largest_inode_number_of_the_leaves_of_the_snapshot = None;
    for entry in &inode_root.entries {
        let (separator_key, identity, child) = inode_internal_entry_parsed_and_judged(entry)?;
        let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
            reader,
            &child,
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
                identity,
                child,
            },
            root,
            expected_filesystem_identifier,
        )?;
        // 快照的冷读与冷走读、可写挂载重建、只读挂载同一判：全部叶的记录按叶序严格升序，同一张两片交叠的镜像四个读者一个结局。
        largest_inode_number_of_the_leaves_of_the_snapshot =
            inode_records_of_the_leaf_ascend_after_the_leaves_before(
                separator_key,
                largest_inode_number_of_the_leaves_of_the_snapshot,
                &records,
            )?;
        inode_numbers_read_back.extend(records.iter().map(|record| record.inode));
        if let Some(record) = records
            .into_iter()
            .find(|record| record.inode == FIRST_INODE_NUMBER)
        {
            inode_record = Some(record);
        }
    }
    inode_tree_root_key_range_is_the_subtree_coverage(&inode_root, inode_numbers_read_back)?;
    let Some(inode_record) = inode_record else {
        return Ok(SnapshotFileReading::NoFile);
    };
    if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
        return Ok(SnapshotFileReading::NoFile);
    }
    let data_units_within_the_size = data_unit_count_matches_the_inode_size(
        extents.data_pointers.len(),
        extents.tail,
        &inode_record,
    )?;
    let mut content: Vec<u8> =
        Vec::with_capacity(usize::try_from(inode_record.size).expect("文件长度装得进 usize"));
    // 迭代次数的上界是 size 之内的单元数（上面按标签核过）；跨轮携带的只有已经拼出来的内容。
    for (position, pointer) in extents
        .data_pointers
        .iter()
        .take(data_units_within_the_size)
        .enumerate()
    {
        let bytes = read_data_unit_via_hint_then_central_mapping(
            reader,
            pointer,
            &central_mapping_locations_of_key,
            &mut stale_location_hint_hops_not_exposed_by_this_reader,
        )?
        .into_content()?;
        let payload = data_unit_judged_against_its_pointer_and_the_inode_record(
            &bytes,
            DataUnitOfAFile {
                position,
                pointer,
                extent_tree: extent_root_pointer.head.birth_tree,
            },
            &inode_record,
            expected_filesystem_identifier,
        )?;
        content.extend_from_slice(payload);
    }
    Ok(SnapshotFileReading::File(
        content_padded_with_the_hole_up_to_the_size(content, &inode_record),
    ))
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
    let system_configuration = choose_system_configuration(reader)?;
    walk_to_file_in_the_unit_area_starting_at(
        reader,
        unit_area_start_of_the_chosen_system_configuration(&system_configuration),
        root,
        mapping_fallbacks,
    )
}
