use super::*;
use singlefs_format::{INODE_LEAF_RECORDS, NEW_POOL_FILE_CREATION_TXG};

#[test]
fn reversing_eight_extent_keys_counts_seven_mismatches_and_one_key_counts_zero() {
    let key = |offset: u64| {
        let mut key = [0u8; 24];
        key[8..16].copy_from_slice(&1u64.to_le_bytes());
        key[16..24].copy_from_slice(&offset.to_le_bytes());
        key
    };
    let ascending: Vec<[u8; 24]> = (0..8u64).map(|index| key(index * 32768)).collect();
    assert_eq!(count_key_order_mismatches(&ascending), 0);
    let mut reversed = ascending.clone();
    reversed.reverse();
    assert_eq!(
        count_key_order_mismatches(&reversed),
        7,
        "7 对相邻 key 全反"
    );
    assert_eq!(
        count_key_order_mismatches(&ascending[..1]),
        0,
        "新池新建文件只有一个数据单元"
    );
    assert_eq!(
        count_key_order_mismatches(&[key(256 << 8), key(1)]),
        1,
        "按字段比不按字节比：小端字节序会把 65536 看成比 1 小"
    );
}

#[test]
fn birth_sequences_count_per_tree_and_per_checkpoint_from_zero() {
    let mut sequences = BirthSequenceAllocator::default();
    let inode = TreeIdentifier(TREE_IDENTIFIER_INODE);
    assert_eq!(
        sequences.next(inode, CheckpointTxg(3), InstanceGeneration(1)),
        BirthSequence(0),
        "叶先发号"
    );
    assert_eq!(
        sequences.next(
            TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            CheckpointTxg(3),
            InstanceGeneration(1)
        ),
        BirthSequence(0)
    );
    assert_eq!(
        sequences.next(inode, CheckpointTxg(3), InstanceGeneration(1)),
        BirthSequence(1),
        "同树同 txg 的根排第二"
    );
    assert_eq!(
        sequences.next(inode, CheckpointTxg(4), InstanceGeneration(1)),
        BirthSequence(0),
        "换 txg 从 0 起"
    );
}

/// 出生序号的作用域是一次 checkpoint（D19（块指针的结构与宽度预算） 已定项 9）：同一个 checkpoint 里连着装两个文件对象
/// 与一棵两片叶的 inode 树，后装的接着先装的序号数，写进单元头里的也是接着数的号，映射 key 不撞；
/// 发号器挪回「每次装对象都重建」时第二个对象从 0 重数，这条用例红（里程碑「覆盖写、释放、回退与复用」增补 2 第 19 行）。
#[test]
fn second_file_object_in_the_same_checkpoint_continues_birth_sequences_instead_of_restarting_at_zero(
) {
    let txg = CheckpointTxg(NEW_POOL_FILE_CREATION_TXG);
    let instance = InstanceGeneration(1);
    let filesystem_identifier = [0x5a; 16];
    let device_identities = [DeviceIdentity(0), DeviceIdentity(1)];
    let checkpoint = FileVersionCheckpoint {
        txg,
        write_order: WriteOrder {
            instance,
            transaction: NEW_POOL_FILE_CREATION_NUMBER,
        },
        trees: FileVersionTreeIdentifiers::issued_from_watermark(TREE_IDENTIFIER_WATERMARK_AT_MKFS)
            .expect("mkfs 水位 65536 加十二个号装得下")
            .0,
        filesystem_identifier: &filesystem_identifier,
        device_identities: &device_identities,
    };
    let content = [7u8; 3000];
    let file = FileVersionPlan {
        inode: InodeNumber(FIRST_INODE_NUMBER),
        content: &content,
        write_time_seconds: 1_788_000_000,
        inode_object_birth: txg,
        // 新池新建文件那次发布的取值：改动计数 = 这次发布的 checkpoint_txg（增补 2 第 11 行）。
        change_count: NEW_POOL_FILE_CREATION_TXG,
    };
    let mut checkpoint_sequences = BirthSequenceAllocator::default();
    let one_unit_transactions = split_sequential_write_into_one_unit_transactions(
        u64::try_from(content.len()).expect("内容长度"),
        NEW_POOL_FILE_CREATION_NUMBER,
    );
    let context = MultiLevelTreeBuildContext {
        txg,
        filesystem_identifier: &filesystem_identifier,
        instance,
        device_identities: &device_identities,
    };
    let extent_plan = plan_the_extent_tree_after_this_publish(None, Some(1));
    let extent_tree_of =
        |data_slot: u64, extent_root_slot: u64, sequences: &mut BirthSequenceAllocator| {
            let units = build_file_version_units(
                &checkpoint,
                &file,
                &one_unit_transactions,
                &FileVersionSlots {
                    data: vec![SlotNumber(data_slot)],
                },
            );
            let mut extent_slots = BTreeMap::new();
            extent_slots.insert(TransactionUnit::ExtentRoot, SlotNumber(extent_root_slot));
            build_extent_tree(
                &extent_plan,
                &FilesOtherThanTheFirstPlan {
                    files: Vec::new(),
                    replaced_previous_data_roles: Vec::new(),
                    replaced_previous_lower_node_roles: Vec::new(),
                    previous_roles_without_a_successor: Vec::new(),
                },
                None,
                None,
                None,
                &units.data_pointers,
                Some(u64::try_from(content.len()).expect("内容长度")),
                &[],
                checkpoint.trees.extent,
                &context,
                &extent_slots,
                sequences,
            )
        };
    let first = extent_tree_of(50176, 50240, &mut checkpoint_sequences);
    let second = extent_tree_of(50178, 50241, &mut checkpoint_sequences);
    assert_eq!(
        (
            first.birth_sequences_of_rewritten_nodes[&TransactionUnit::ExtentRoot],
            second.birth_sequences_of_rewritten_nodes[&TransactionUnit::ExtentRoot]
        ),
        (BirthSequence(0), BirthSequence(1)),
        "同一个 checkpoint 的第二个对象接着数：extent 树 0、1"
    );
    assert_eq!(
        parse_index_node(&second.units[0].bytes)
            .expect("刚装的 extent 根解得开")
            .birth_sequence,
        BirthSequence(1),
        "单元头里写的就是接着数的号"
    );
    // inode 树这一半：两片叶容器加一个根，按叶序先叶后根发号，接着上面两个对象已经用掉的号数
    // （inode 树那个计数器上面一个都没用过，所以从 0 起）。
    let mut records = Vec::new();
    // 这里不写 `change_count: txg.0,`：`crates/mutations.tsv` 按那串字面锚在 `publish_overwrite` 上，
    // 同一份文件里出现两次它就腐化（门禁 crates-mutation-replay 判锚点恰好命中一次）。
    let change_count = txg.0;
    for inode in 1..=(INODE_LEAF_RECORDS + 1) {
        records.push(InodeRecord {
            inode,
            object_birth: txg,
            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size: 0,
            change_count,
            write_time_seconds: 1_788_000_000,
            truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
        });
    }
    let inode_tree = write_records_into_leaf_containers(
        &[],
        &records,
        txg,
        TreeIdentifier(TREE_IDENTIFIER_INODE),
    )
    .expect("234 条记录分两片容器");
    let mut slots = BTreeMap::new();
    slots.insert(
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(0)),
        SlotNumber(50242),
    );
    slots.insert(
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(1)),
        SlotNumber(50250),
    );
    let inode_tree_units = build_inode_tree_units(
        &checkpoint,
        &inode_tree,
        &[],
        &slots,
        &mut checkpoint_sequences,
    );
    assert_eq!(
        inode_tree_units
            .leaf_containers
            .iter()
            .map(|container| container.pointer.birth_sequence)
            .collect::<Vec<_>>(),
        vec![BirthSequence(0), BirthSequence(1)],
        "两片叶按叶序发号"
    );
    assert_eq!(
        inode_tree_units
            .rewritten_root
            .as_ref()
            .expect("这次重写了根")
            .birth_sequence,
        BirthSequence(2),
        "树内先叶后根：根排在两片叶之后"
    );
    assert_ne!(
        mapping_key_for_node(
            UNIT_CLASS_PACKED,
            inode_tree_units.leaf_containers[0].pointer
        ),
        mapping_key_for_node(
            UNIT_CLASS_PACKED,
            inode_tree_units.leaf_containers[1].pointer
        ),
        "两片叶容器的映射 key 不撞"
    );
    assert_eq!(
        checkpoint_sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance),
        BirthSequence(3),
        "装完两片叶与一个根之后，同一个 checkpoint 里 inode 树的下一个号是 3"
    );
    assert_eq!(
        checkpoint_sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance),
        BirthSequence(2),
        "extent 树那个计数器各数各的：两个对象用掉 0、1，下一个是 2"
    );
}
