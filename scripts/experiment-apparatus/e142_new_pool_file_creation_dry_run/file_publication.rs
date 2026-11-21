//! 新池里发布第一个文件的整趟：事务内的单元、写序与日志记录。

use e7_index_bench::{u16_from_u64, usize_from_u64};
use super::*;

#[allow(clippy::too_many_arguments, reason = "字段表就是这么多段，收成结构体只会多一层没人验的名字（build_index_node 同一处理由）")]
pub(super) fn publish_first_file(
    pool: &mut RecordingPool,
    parameters: &PoolParameters,
    genesis: &MkfsOutput,
    file_bytes: &[u8],
    instance: InstanceGeneration,
    previous_record_bytes: Option<&[u8]>,
    change_count: u64,
) -> TransactionOutput {
    let txg = CheckpointTxg(NEW_POOL_FILE_CREATION_TXG);
    let transaction = TransactionNumber(1);
    let write_order = WriteOrder { instance, transaction };
    let fsid = &parameters.fsid;
    let mut sequences = BirthSequenceAllocator::default();
    let mut units: Vec<(SlotNumber, TransactionUnit, Vec<u8>)> = Vec::new();
    let mut index_node_header_widths = Vec::new();

    // t1 数据单元
    let data_identity = DataUnitIdentity { tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), object: FIRST_INODE_NUMBER, object_birth: txg, anchor_offset: 0 };
    let data_unit = build_data_unit(data_identity, txg, fsid, write_order, file_bytes);
    let data_pointer = DataPointer {
        head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(SLOT_DATA_UNIT), &data_unit),
        write_order,
    };

    // t3 inode 树叶容器
    let inode_leaf_identity = PackedIdentity { birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE), record_type: PACKED_TYPE_INODE, container: FIRST_INODE_NUMBER, container_birth: txg };
    let inode_leaf_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
    // 改动计数取调用方传入的 change_count（D8 已定项 6 偏移 88 的字段定义 = 最后一次改动所在发布的
    // checkpoint_txg；主口径传 NEW_POOL_FILE_CREATION_TXG，`crates/` 今天同口径——重跑登记 e142-r11-prereg.md 第三节）。
    let inode_record = InodeRecord { inode: FIRST_INODE_NUMBER, object_birth: txg, size: file_bytes.len() as u64, change_count };
    let inode_leaf_unit = build_packed_unit(inode_leaf_identity, u16_from_u64(INODE_RECORD_BYTES), &[inode_record.to_bytes()], txg, fsid, write_order, inode_leaf_sequence);
    let inode_leaf_pointer = NodePointer {
        head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE), birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(SLOT_INODE_LEAF), &inode_leaf_unit),
        instance,
        birth_sequence: inode_leaf_sequence,
    };

    // t2 extent 树上段叶（根即叶，5.2 ζ：只有一个 inode ⇒ 最低的、罩得住最大 inode 号的层级是 0）。
    // key 区间照 D18 已定项 2 射程写这片叶按位置罩的那一段（D8 已定项 14 第 401 行，唯一写法），
    // 不写叶里第一条与最后一条条目的 key：第 k 片（k = inode ÷ 143）罩 `[143k, 143k + 142]`，闭区间，第三分量 2^64 − 1。
    let extent_leaf_index = extent_upper_leaf_index(FIRST_INODE_NUMBER);
    let (extent_smallest_key, extent_largest_key) = extent_upper_leaf_positional_key_range(extent_leaf_index);
    let extent_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
    let extent_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_EXTENT),
        0,
        24,
        &extent_smallest_key,
        &extent_largest_key,
        txg,
        fsid,
        instance,
        extent_sequence,
        u16_from_u64(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES),
        &[build_extent_upper_leaf_entry(FIRST_INODE_NUMBER, data_pointer)],
    );
    index_node_header_widths.push(("extent", index_node_header_bytes(24)));

    // t4 inode 树根
    let inode_root_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
    let inode_key = FIRST_INODE_NUMBER.to_le_bytes();
    let inode_root_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_INODE),
        1,
        8,
        &inode_key,
        &inode_key,
        txg,
        fsid,
        instance,
        inode_root_sequence,
        u16_from_u64(INODE_INTERNAL_ENTRY),
        &[build_inode_internal_entry(FIRST_INODE_NUMBER, inode_leaf_identity, inode_leaf_pointer)],
    );
    index_node_header_widths.push(("inode", index_node_header_bytes(8)));

    // t5..t9 分配记录树（按位置寻址，D8 已定项 14）：mkfs 的 m1 分配代 0；m2（第 0 版树表）被这次发布重写树表换下
    // ⇒ 记录改写成已释放、释放代 = 这次的 txg（D3 已定项 7，里程碑「覆盖写、释放、回退与复用」步 5 逼出：不释放它，txg 0 的根离开候选集
    // 之后这一槽永远占着、I-3.1 红）；其余就是这次发布的 txg（「分配代」）；每盘各一条——含分配记录树自己的 5 个新节点
    // （D8 已定项 14 射程：这几个节点也占落点、也要记分配记录，不是只有旧的八个单元）。
    // deadlist 树 day-1 注册但没有根节点 ⇒ 不占落点、不写分配记录（字节表零那一节的 t5 那一行）。
    // 分配记录树两片叶 / 两个层级 1 节点只在两块盘时都建（build_allocation_tree 只给 `parameters.devices()`
    // 里真实存在的盘各建一份）——一块盘的对照臂（第七节 B10）没有「盘 1」那两个节点，`allocated` 跟着它走，
    // 不能不看 device_count 就把两块盘的槽都算成「有条分配记录」（那样会在没写过的槽上凭空记一条记录）。
    let mut allocated: Vec<(u64, u16, u64, bool)> = vec![
        (SLOT_INSTANCE_TABLE, 2, 0, false),
        (SLOT_TREE_TABLE_GENESIS, 1, NEW_POOL_FILE_CREATION_TXG, true),
        (SLOT_DATA_UNIT, 2, NEW_POOL_FILE_CREATION_TXG, false),
        (SLOT_EXTENT_ROOT, 1, NEW_POOL_FILE_CREATION_TXG, false),
        (SLOT_INODE_LEAF, 2, NEW_POOL_FILE_CREATION_TXG, false),
        (SLOT_INODE_ROOT, 1, NEW_POOL_FILE_CREATION_TXG, false),
        (SLOT_ALLOCATION_LEAF_DEVICE_0, 1, NEW_POOL_FILE_CREATION_TXG, false),
    ];
    if parameters.device_count > 1 {
        allocated.push((SLOT_ALLOCATION_LEAF_DEVICE_1, 1, NEW_POOL_FILE_CREATION_TXG, false));
    }
    allocated.push((SLOT_ALLOCATION_INTERNAL_DEVICE_0, 1, NEW_POOL_FILE_CREATION_TXG, false));
    if parameters.device_count > 1 {
        allocated.push((SLOT_ALLOCATION_INTERNAL_DEVICE_1, 1, NEW_POOL_FILE_CREATION_TXG, false));
    }
    allocated.push((SLOT_ALLOCATION_ROOT, 1, NEW_POOL_FILE_CREATION_TXG, false));
    allocated.push((SLOT_ACCOUNTING_ROOT, 1, NEW_POOL_FILE_CREATION_TXG, false));
    allocated.push((SLOT_MAPPING_ROOT, 1, NEW_POOL_FILE_CREATION_TXG, false));
    allocated.push((SLOT_TREE_TABLE_FIRST_PUBLISH, 1, NEW_POOL_FILE_CREATION_TXG, false));
    let mut allocation_records: Vec<AllocationRecord> = parameters
        .devices()
        .iter()
        .flat_map(|device| allocated.iter().map(move |(slot, span, generation, is_released)| AllocationRecord { device: *device, slot: SlotNumber(*slot), span_slots: *span, generation: CheckpointTxg(*generation), is_released: *is_released }))
        .collect();
    allocation_records.sort_by_key(AllocationRecord::sort_key);
    let allocation_tree = build_allocation_tree(parameters, &allocation_records, txg, instance, &mut sequences);
    // 分配记录树的物理节点数与门槛看齐（第七节 B13「new_pool_file_creation_allocation_nodes」）；变了要先看 build_allocation_tree 的假设。
    assert_eq!(
        allocation_tree.units.len() as u64,
        allocation_record_tree_new_pool_file_creation_node_count(u64::try_from(parameters.device_count).expect("设备数"), 2),
        "分配记录树的物理节点数与形状函数算出来的不一致"
    );
    index_node_header_widths.push(("allocation", index_node_header_bytes(ALLOCATION_KEY_BYTES)));

    // t6 记账树（D5 已定项 8）：两盘时 **15 行**——
    // 带设备维的六项各两行（已分配 1、空闲 2、不可回收 3、defer 待释放 5、碎片度 runs 10、全空聚簇段数 11），
    // 池级三行（待删占用 4、已承诺预留 6、inode 号水位 12）。准入不等式要的四项 day-1 各写一行：不可回收、待删占用两项 0，
    // 已承诺预留照 D3（空间分配） 已定项 2 的式子现算（下面 `committed_reservation`），defer 待释放见下；
    // 「空 ≠ 0」那条防线因此保住：读不到行是坏账，读到 0 是真的 0。seq 一律 1（D8 已定项 10：直落叶）。
    //
    // 已承诺预留要的四个量里，「树表里有没有 `deleted_inodes` 条目」与「墓碑树的根指针」住在树表第 1 版的条目表里。
    // 树表里 day-1 注册、根指针为零的八条（D18 已定项 24 第 6 条，含退役树）不引用任何别的单元，先建出来；带指针的四条要等记账树单元落了位才建得出来
    // （它们引用记账树的根指针），树表单元仍在后面照 D8 已定项 8 的树 ID 升序组装。
    let day_one_zero_root_tree_table_entries = [
        TreeTableEntry { kind: TREE_KIND_LIVELIST, tree: TreeIdentifier(TREE_IDENTIFIER_LIVELIST), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        TreeTableEntry { kind: TREE_KIND_SHARE_COUNT, tree: TreeIdentifier(TREE_IDENTIFIER_SHARE_COUNT), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        TreeTableEntry { kind: TREE_KIND_DEADLIST, tree: TreeIdentifier(TREE_IDENTIFIER_DEADLIST), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        TreeTableEntry { kind: TREE_KIND_DELETED_INODES, tree: TreeIdentifier(TREE_IDENTIFIER_DELETED_INODES), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_IDENTIFIER_INODE },
        TreeTableEntry { kind: TREE_KIND_TOMBSTONE, tree: TreeIdentifier(TREE_IDENTIFIER_TOMBSTONE), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_IDENTIFIER_INODE },
        TreeTableEntry { kind: TREE_KIND_LINEAGE, tree: TreeIdentifier(TREE_IDENTIFIER_LINEAGE), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        TreeTableEntry { kind: TREE_KIND_LOGGED_OPERATIONS, tree: TreeIdentifier(TREE_IDENTIFIER_LOGGED_OPERATIONS), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        // `layout/03`「树表退役树条目」：退役树一条，零根，头 ID 写 inode 树的树 ID（同墓碑树）。
        TreeTableEntry { kind: TREE_KIND_RETIREMENT, tree: TreeIdentifier(TREE_IDENTIFIER_RETIREMENT), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_IDENTIFIER_INODE },
    ];
    let tree_table_has_deleted_inodes_entry = day_one_zero_root_tree_table_entries.iter().any(|entry| entry.tree == TreeIdentifier(TREE_IDENTIFIER_DELETED_INODES));
    let tombstone_tree_root = day_one_zero_root_tree_table_entries
        .iter()
        .find(|entry| entry.tree == TreeIdentifier(TREE_IDENTIFIER_TOMBSTONE))
        .expect("树表第 1 版里墓碑树（65545）day-1 注册")
        .root;
    let retirement_tree_root = day_one_zero_root_tree_table_entries
        .iter()
        .find(|entry| entry.tree == TreeIdentifier(TREE_IDENTIFIER_RETIREMENT))
        .expect("树表第 1 版里退役树 day-1 注册")
        .root;
    let inode_leaf_record_bytes = [inode_record.to_bytes()];
    let committed_reservation = committed_reservation_bytes(
        u64::try_from(parameters.device_count).expect("设备数装得进 u64"),
        tombstone_tree_internal_levels(&tombstone_tree_root),
        retirement_tree_internal_levels(&retirement_tree_root),
        live_object_count_of_inode_records(&inode_leaf_record_bytes),
        record_count_of_inode_records(&inode_leaf_record_bytes),
        tree_table_has_deleted_inodes_entry,
    );
    let allocated_slots: u64 = allocated.iter().map(|(_, span, _, _)| u64::from(*span)).sum();
    let occupied: std::collections::BTreeSet<u64> = allocated.iter().flat_map(|(slot, span, _, _)| (0..u64::from(*span)).map(move |offset| slot + offset)).collect();
    // bump 次序 + 码 3 容器要 32768 对齐 ⇒ 开放段里 t2 与 t3 之间恰好空着一个槽，它自己是一段空闲 run。
    assert_eq!(SLOT_SKIPPED_BY_ALIGNMENT, SLOT_EXTENT_ROOT + 1, "空洞紧跟在 extent 根后面");
    assert_eq!(SLOT_INODE_LEAF, SLOT_SKIPPED_BY_ALIGNMENT + 1, "inode 叶容器跳过空洞落在下一个对齐槽对");
    assert!(!occupied.contains(&SLOT_SKIPPED_BY_ALIGNMENT), "槽 50241 空着（D3 已定项 10 ⑤ 的落点表）");
    let pool_wide = |statistic: u16, tree: u64, value: u64| AccountingEntry {
        statistic,
        tree: TreeIdentifier(tree),
        device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
        generation: txg,
        value,
        sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
    };
    let mut accounting_entries = vec![
        // 第 12 项带树维、不带设备维：树 ID 就是那个可写头的 inode 树（D5 已定项 9）。
        pool_wide(STATISTIC_INODE_WATERMARK, TREE_IDENTIFIER_INODE, FIRST_INODE_NUMBER + 1),
        pool_wide(STATISTIC_PENDING_DELETE_BYTES, TREE_IDENTIFIER_NONE, 0),
        pool_wide(STATISTIC_COMMITTED_RESERVATION_BYTES, TREE_IDENTIFIER_NONE, committed_reservation),
    ];
    for device in parameters.devices() {
        let per_device = |statistic: u16, value: u64| AccountingEntry {
            statistic,
            tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
            device,
            generation: txg,
            value,
            sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
        };
        accounting_entries.push(per_device(STATISTIC_ALLOCATED_BYTES, allocated_slots * SLOT_BYTES));
        // 第 2 项必须独立维护、不许由「容量 − 已分配」现算（D5 已定项 4 的 ⚠️）：这里从空槽数直接数出来。
        accounting_entries.push(per_device(STATISTIC_FREE_BYTES, free_slot_count(&occupied, parameters.unit_area_slots()) * SLOT_BYTES));
        accounting_entries.push(per_device(STATISTIC_UNRECLAIMABLE_BYTES, 0));
        // defer 待释放 = mkfs 那片第 0 版树表单元（1 槽）：A 重写树表把它换下（D3 已定项 7）。
        accounting_entries.push(per_device(STATISTIC_DEFER_QUEUE_BYTES, SLOT_BYTES));
        // 第 10 项带设备维：单元区空闲槽的连续段数，逐设备一行。
        accounting_entries.push(per_device(STATISTIC_FRAGMENTATION_RUNS, free_run_count(&occupied, parameters.unit_area_slots())));
        accounting_entries.push(per_device(STATISTIC_EMPTY_CLUSTER_SEGMENTS, empty_cluster_segment_count(&occupied, parameters.unit_area_slots())));
    }
    accounting_entries.sort_by_key(AccountingEntry::sort_key);
    let accounting_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), txg, instance);
    let accounting_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
        0,
        22,
        &accounting_entries[0].key_bytes(),
        &accounting_entries[accounting_entries.len() - 1].key_bytes(),
        txg,
        fsid,
        instance,
        accounting_sequence,
        u16_from_u64(ACCOUNTING_ENTRY_BYTES),
        &accounting_entries.iter().map(AccountingEntry::to_bytes).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("accounting", index_node_header_bytes(22)));

    // 五棵树的根指针（t11 映射与 t12 树表都要引它们）
    let node_pointer = |tree: TreeIdentifier, slot: u64, unit: &[u8], sequence: BirthSequence| NodePointer {
        head: PointerHead { birth_tree: tree, birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(slot), unit),
        instance,
        birth_sequence: sequence,
    };
    let extent_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_EXTENT), SLOT_EXTENT_ROOT, &extent_unit, extent_sequence);
    let inode_root_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_INODE), SLOT_INODE_ROOT, &inode_root_unit, inode_root_sequence);
    let allocation_pointer = allocation_tree.root_pointer;
    let accounting_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), SLOT_ACCOUNTING_ROOT, &accounting_unit, accounting_sequence);

    // t11 中央映射树：码 1 一条（数据单元）+ 码 2 / 码 3 九条（D19 已定项 8；映射树自己、树表、实例表豁免）——
    // 分配记录树自己的 5 个新节点也要各留一条映射条目，第七节 B7「mapping_entries=10」。
    let mut mapping_entries: Vec<(Vec<u8>, [LocationEntry; 2])> = vec![
        (mapping_key_for_data(data_pointer.head, data_pointer.write_order), data_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer), extent_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer), inode_leaf_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer), inode_root_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer), accounting_pointer.locations),
    ];
    for (_, pointer) in allocation_tree.leaf_pointers.iter().chain(allocation_tree.internal_pointers.iter()) {
        mapping_entries.push((mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer), pointer.locations));
    }
    mapping_entries.push((mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer), allocation_pointer.locations));
    mapping_entries.sort_by_key(|(key, _)| mapping_key_sort_key(key));
    let mapping_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_MAPPING), txg, instance);
    let mapping_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_MAPPING),
        0,
        usize_from_u64(MAPPING_KEY_BYTES),
        &mapping_entries[0].0,
        &mapping_entries[mapping_entries.len() - 1].0,
        txg,
        fsid,
        instance,
        mapping_sequence,
        u16_from_u64(MAPPING_ENTRY_BYTES),
        &mapping_entries.iter().map(|(key, locations)| build_mapping_entry(key, *locations)).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("mapping", index_node_header_bytes(usize_from_u64(MAPPING_KEY_BYTES))));
    let mapping_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_MAPPING), SLOT_MAPPING_ROOT, &mapping_unit, mapping_sequence);

    // t12 树表单元第 1 版：**十二条**条目按树 ID 升序（D8 已定项 8）；中央映射树的根住根记录、不进树表（D19 已定项 11）；
    // livelist、稀疏旁表、deadlist 与 `deleted_inodes`、墓碑树、谱系树、logged_ops、退役树八棵 day-1 注册、根指针为零（D18 已定项 24 第 6 条）。
    // 头 ID（D5 已定项 9；D8 已定项 8「头 ID 8 字节的取值」）：inode 树写自己（65537）、extent 树写它服务的那个头（65537）、
    // `deleted_inodes`、墓碑树与退役树写 inode 树的树 ID（65537），其余写 0。
    // 带指针的四条（extent、inode、分配记录、记账）在这里建，后面接上先建好的八条（树 ID 从 livelist 到退役树）。
    let mut tree_table_entries = vec![
        TreeTableEntry { kind: TREE_KIND_EXTENT, tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), root: extent_pointer, birth_txg: txg, head_identifier: TREE_IDENTIFIER_INODE },
        TreeTableEntry { kind: TREE_KIND_INODE, tree: TreeIdentifier(TREE_IDENTIFIER_INODE), root: inode_root_pointer, birth_txg: txg, head_identifier: TREE_IDENTIFIER_INODE },
        TreeTableEntry { kind: TREE_KIND_ALLOCATION, tree: TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), root: allocation_pointer, birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        TreeTableEntry { kind: TREE_KIND_ACCOUNTING, tree: TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), root: accounting_pointer, birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
    ];
    tree_table_entries.extend(day_one_zero_root_tree_table_entries);
    assert_eq!(
        tree_table_entries.iter().any(|entry| entry.tree == TreeIdentifier(TREE_IDENTIFIER_DELETED_INODES)),
        tree_table_has_deleted_inodes_entry,
        "组装出来的树表第 1 版有没有 deleted_inodes 条目，要与算已承诺预留时看的那一份一致"
    );
    assert!(tree_table_entries.windows(2).all(|pair| pair[0].tree < pair[1].tree), "树表条目按树 ID 严格升序（D8 已定项 8）");
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
    let tree_table_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &TREE_IDENTIFIER_EXTENT.to_le_bytes(),
        &tree_table_entries.last().expect("树表第 1 版不空").tree.0.to_le_bytes(),
        txg,
        fsid,
        instance,
        tree_table_sequence,
        u16_from_u64(TREE_TABLE_ENTRY_BYTES),
        &tree_table_entries.iter().map(TreeTableEntry::to_bytes).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("tree_table", index_node_header_bytes(TREE_TABLE_KEY_WIDTH)));
    let tree_table_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_NONE), SLOT_TREE_TABLE_FIRST_PUBLISH, &tree_table_unit, tree_table_sequence);

    units.push((SlotNumber(SLOT_DATA_UNIT), TransactionUnit::Data, data_unit));
    units.push((SlotNumber(SLOT_EXTENT_ROOT), TransactionUnit::ExtentRoot, extent_unit));
    units.push((SlotNumber(SLOT_INODE_LEAF), TransactionUnit::InodeLeaf, inode_leaf_unit));
    units.push((SlotNumber(SLOT_INODE_ROOT), TransactionUnit::InodeRoot, inode_root_unit));
    // t5..t9：分配记录树的五个物理节点，bump 次序已经是「树内先叶后根、同层按设备升序」（build_allocation_tree）。
    units.extend(allocation_tree.units);
    units.push((SlotNumber(SLOT_ACCOUNTING_ROOT), TransactionUnit::AccountingTree, accounting_unit));
    units.push((SlotNumber(SLOT_MAPPING_ROOT), TransactionUnit::MappingTree, mapping_unit));
    units.push((SlotNumber(SLOT_TREE_TABLE_FIRST_PUBLISH), TransactionUnit::TreeTable, tree_table_unit));

    // 持久顺序第一段：单元与节点。
    for (slot, _, unit) in &units {
        write_unit_to_every_device(pool, parameters, *slot, unit);
    }
    if parameters.barriers == BarrierPolicy::Settled {
        pool.barrier();
    }

    // 第二段：journal 记录，点名 t1..t12 每个两盘；反向链照 D23 已定项 19 ② 罩前一条的整个头（header_csum 按零参与）。
    // 点名项的 key 尾段与 `units` 一一对应，次序就是 t1..t12（第五节 5.2 θ：点名照 `units` 的先后）；
    // unit_class 与 TransactionUnit 自报的那一个对账。
    let named_identities: Vec<(u8, TreeIdentifier, [u8; 10])> = {
        let mut list = vec![
            (UNIT_CLASS_DATA, data_pointer.head.birth_tree, data_key_tail(data_pointer.write_order)),
            (UNIT_CLASS_INDEX_NODE, extent_pointer.head.birth_tree, node_key_tail(extent_pointer.instance, extent_pointer.birth_sequence)),
            (UNIT_CLASS_PACKED, inode_leaf_pointer.head.birth_tree, node_key_tail(inode_leaf_pointer.instance, inode_leaf_pointer.birth_sequence)),
            (UNIT_CLASS_INDEX_NODE, inode_root_pointer.head.birth_tree, node_key_tail(inode_root_pointer.instance, inode_root_pointer.birth_sequence)),
        ];
        for (_, pointer) in allocation_tree.leaf_pointers.iter().chain(allocation_tree.internal_pointers.iter()) {
            list.push((UNIT_CLASS_INDEX_NODE, pointer.head.birth_tree, node_key_tail(pointer.instance, pointer.birth_sequence)));
        }
        list.push((UNIT_CLASS_INDEX_NODE, allocation_pointer.head.birth_tree, node_key_tail(allocation_pointer.instance, allocation_pointer.birth_sequence)));
        list.push((UNIT_CLASS_INDEX_NODE, accounting_pointer.head.birth_tree, node_key_tail(accounting_pointer.instance, accounting_pointer.birth_sequence)));
        list.push((UNIT_CLASS_INDEX_NODE, mapping_pointer.head.birth_tree, node_key_tail(mapping_pointer.instance, mapping_pointer.birth_sequence)));
        list.push((UNIT_CLASS_INDEX_NODE, tree_table_pointer.head.birth_tree, node_key_tail(tree_table_pointer.instance, tree_table_pointer.birth_sequence)));
        list
    };
    assert_eq!(named_identities.len(), units.len(), "点名项与写清单的单元数要一一对应");
    let named: Vec<NamedUnit> = units
        .iter()
        .zip(named_identities)
        .map(|((slot, unit_identity, unit), (unit_class, birth_tree, key_tail))| {
            assert_eq!(unit_class, unit_identity.class_and_key_width().0, "点名项的类标签与 {} 自报的对不上", unit_identity.tag());
            assert_eq!(birth_tree, unit_identity.tree(), "点名项的出生树与 {} 自报的对不上", unit_identity.tag());
            NamedUnit { locations: parameters.location_entries(*slot, unit), unit_class, birth_tree, birth_txg: txg, key_tail }
        })
        .collect();
    // 这次发布要写的根（第三段写它）先在这里构造：记录新根段的后三项取它的同名字段（D23 已定项 15「由记录重建那次发布的根」）。
    let root = RootRecord {
        fsid: parameters.fsid,
        unmount_marker: false, // 新池新建文件不是正常卸载（D22 已定项 7；这条流里没有正常卸载）
        instance,
        checkpoint_txg: txg,
        tree_table: tree_table_pointer,
        tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH,
        rollback_floor: CheckpointTxg(0),
        instance_table: genesis.root.instance_table,
        mapping_root: mapping_pointer,
        // C512：带文件的一版写全零（不是「树表 0 条、写过行」那一格）。
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    let counter = JournalCounter(NEW_POOL_FILE_CREATION_TXG);
    let record = JournalRecord {
        instance,
        counter,
        checkpoint_txg: txg,
        transaction,
        is_commit: true,
        ordinal_within_publish: 1,
        record_flags: parameters.last_of_publish_flag,
        back_chain: previous_record_bytes.map_or(0, journal_back_chain),
        fsid: unit_fsid(fsid),
        new_tree_table: tree_table_pointer,
        new_mapping_root: mapping_pointer,
        new_tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH,
        new_rollback_floor: CheckpointTxg(0),
        new_instance_table: root.instance_table,
        new_allocation_record_tree_root: root.allocation_record_tree_root,
        new_unmount_marker: root.unmount_marker,
        named,
    };
    let record_bytes = record.to_bytes();
    for device in parameters.devices() {
        pool.write(device, journal_record_offset(counter), &record_bytes, StepKind::JournalRecord);
    }
    if parameters.barriers == BarrierPolicy::Settled {
        pool.barrier();
    }

    // 第三段：根槽 FUA，区域 3 mod 3 = 0 的槽 1（暖机之后新池新建文件是 txg 3）。
    // （这次发布的根在上面记录之前已经构造好，记录新根段的后三项取它的同名字段。）
    let (region, ring_slot) = ring_target_for_publish(txg);
    pool.write(parameters.region_devices[usize_from_u64(region)], ring_slot_offset(region, ring_slot), &root.to_slot(), StepKind::RootRecordFua);

    // 根槽之后：系统配置槽轮换，世代号 5、落槽 1（D22 已定项 16），tail 前移到 jsn 3（D16 已定项 7 的系统配置注）。
    let (slot_generation, slot_index) = system_configuration_write_for_publish(txg);
    for device in parameters.devices() {
        let system_configuration = SystemConfiguration {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation,
            region_devices: parameters.region_devices,
            journal_tail: NEW_POOL_FILE_CREATION_TXG,
            journal_instance: instance,
            rollback_floor: CheckpointTxg(0), // 新池新建文件没有抬 F：非空有效根不足 4 个（D16 已定项 1）
        };
        pool.write(device, DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[slot_index]), &system_configuration.to_slot(), StepKind::SystemConfigurationSlot);
    }
    // ⑧ 发布尾那道屏障（D16 已定项 7「根槽（FUA）→ 系统配置槽 → 屏障」，登记 R43）。
    if parameters.issues_trailing_publish_barrier() {
        pool.barrier();
    }

    TransactionOutput {
        root,
        record,
        record_bytes,
        units_by_slot: units,
        data_pointer,
        mapping_keys: mapping_entries.into_iter().map(|(key, _)| key).collect(),
        allocation_records,
        accounting_entries,
        index_node_header_widths,
    }
}

#[cfg(test)]
#[path = "file_publication_tests.rs"]
mod tests;
