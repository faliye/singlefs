//! 孩子是码 3 容器的码 2 节点头里的 key 区间取子树覆盖区间（D18（块里携带什么信息） 已定项 2「区间取子树覆盖区间」，C478）的搭建：
//! 写路径写出「子树覆盖区间与首末两条条目的 key 不同」的几形——inode 树根（最右那片装到第二条记录、最左那片删掉了打头那一条），
//! 墓碑树、谱系树、退役树层级 2 的根（最后一个孩子那一枝不止一片容器）——与把根头里的区间改回「首末两条条目的 key」的坏镜像
//! （改头、重封，新的整单元校验和沿引用链补到根槽，再把每条根抄进它那几条 journal 记录的新根段）。
//! harness 档的写者与读者用例、池级 checker 自己的单测与 checker 档的 GPU 对拍各经 `mod` / `#[path]` 引这一份；
//! 用这一份的测试文件同时声明 `mod common;`、`mod common_pointer_head_corruption;` 与 `mod common_deletion_in_progress;`。
#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]

use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
};
use singlefs_core::lineage_tree::{
    LineageContainerCapacity, LineageRecordsPerContainerCappedForTests,
};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::records::{
    TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_RETIREMENT, TREE_KIND_TOMBSTONE,
};
use singlefs_core::retirement_tree::{
    RetirementContainerCapacity, RetirementRecordsPerContainerCappedForTests,
};
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty, PoolWriter,
    TransactionUnit,
};
use singlefs_core::unit::{parse_index_node, IndexNodeHeader};
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::memory_pool::MemoryPool;

use crate::common::{build_pool, parameters, BuiltPool};
use crate::common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, pool_with_new_inodes,
    publish_new_inodes_and_advance_the_pool, unlink_to_the_end, INSTANCE_OF_THE_PUBLISHES,
    WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use crate::common_pointer_head_corruption::{
    node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at,
};

/// 码 2 节点头里 key 区间的起点：`[min_key | max_key]` 各一个 key 宽，接在 key 宽那一字节（偏移 51）之后
/// （`crates/singlefs-core/src/unit.rs` 里码 2 头的字段表「52 key 区间 2k」）。
pub const INDEX_NODE_KEY_RANGE_OFFSET: usize = 52;

/// 压小的那几棵容器树：一片容器装 1 条、一个码 2 节点装 2 条条目——第 3 片容器根满长到层级 2，第 4 片挂进 (1, 1)，
/// 层级 2 的根下面是 (1, 0) = [1, 2]、(1, 1) = [3, 4]：根的子树覆盖区间是 [1, 4]，首末两条条目的分隔 key 是 [1, 3]。
pub const CONTAINERS_OF_A_LEVEL_TWO_TREE_WHOSE_LAST_CHILD_HOLDS_TWO: u64 = 4;

/// 一个码 2 节点装 2 条条目（D18（块里携带什么信息） 已定项 24 第 3 条：只供测试的旋钮不落盘）。
fn two_entries_per_node() -> ContainerTreeNodeEntries {
    ContainerTreeNodeEntries::CappedForTests(
        ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
    )
}

/// 墓碑容器一片 1 条、节点 2 条，谱系与退役容器照产品。
pub fn tombstone_capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container: RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: two_entries_per_node(),
    }
}

/// 谱系容器一片 1 条、节点 2 条，墓碑与退役容器照产品。
pub fn lineage_capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(1).expect("1..=1359"),
        ),
        retirement_container: RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: two_entries_per_node(),
    }
}

/// 退役容器一片 1 条、节点 2 条，墓碑与谱系容器照产品。
pub fn retirement_capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container: RetirementContainerCapacity::CappedForTests(
            RetirementRecordsPerContainerCappedForTests::new(1).expect("1..=959"),
        ),
        node_entries: two_entries_per_node(),
    }
}

/// 写入口装 `capacities`，把空 inode `inode` unlink 到底：意图那一次，再走空（空 inode 没有数据单元，直接是最后一批）。
pub fn unlink_an_empty_inode_under_the_capacities(
    pool: &mut BuiltPool,
    inode: u64,
    capacities: ContainerTreeCapacities,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities);
    let mut current = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        singlefs_core::address::InodeNumber(inode),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
    pool.output = current;
}

/// 新池新建文件之后再建 234 个空 inode：叶 [1..=233] 装满，最右那片装 [234, 235] 两条——根的子树覆盖区间 [1, 235]，
/// 首末两条条目的分隔 key 是 [1, 234]（第二片的容器号 = 触发分裂那条记录的号 234）。
pub fn pool_whose_inode_root_covers_a_full_leaf_and_a_leaf_of_two_records(tag: &str) -> BuiltPool {
    pool_with_new_inodes(tag, INODE_LEAF_RECORDS + 1)
}

/// 新池新建文件之后建 2 个空 inode（2、3，与第一个文件同在容器号 1 那片叶里），再把第一个文件 unlink 到底：那片叶只剩 [2, 3]，
/// 它的分隔 key 照旧是 1（诞生时的最小 key，D8（核心索引结构） 已定项 6）——根的子树覆盖区间 [2, 3]，首末两条条目的分隔 key 是 [1, 1]。
pub fn pool_whose_leftmost_inode_leaf_lost_its_first_record(tag: &str) -> BuiltPool {
    let mut pool = pool_with_new_inodes(tag, 2);
    unlink_to_the_end(&mut pool, 1);
    pool
}

/// 墓碑树长到层级 2、最后一个孩子装两片：新池新建文件之后建 5 个空 inode（2..=6），墓碑容器一片 1 条、节点 2 条，unlink 2..=5 各占一片容器。
pub fn pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers(
    tag: &str,
) -> BuiltPool {
    let mut pool = pool_with_new_inodes(tag, 5);
    let capacities = tombstone_capacities_of_one_record_and_two_entries();
    // 迭代上界是常量 4（每次 unlink 一个 inode、开一片墓碑容器）。
    for inode in 2..2 + CONTAINERS_OF_A_LEVEL_TWO_TREE_WHOSE_LAST_CHILD_HOLDS_TWO {
        unlink_an_empty_inode_under_the_capacities(&mut pool, inode, capacities);
    }
    pool
}

/// 墓碑树长到层级 3：同一组旋钮再 unlink 6 号，第 5 片容器时层级 1 与根都满（新开 (1, 2)、(2, 1)、新根）。
/// 层级 2 的 (2, 0) 下面是 (1, 0) = [1, 2]、(1, 1) = [3, 4]：它的子树覆盖区间 [1, 4]，首末两条条目的分隔 key 是 [1, 3]；根是 [1, 5]。
pub fn pool_whose_tombstone_root_is_of_level_three(tag: &str) -> BuiltPool {
    let mut pool = pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers(tag);
    unlink_an_empty_inode_under_the_capacities(
        &mut pool,
        2 + CONTAINERS_OF_A_LEVEL_TWO_TREE_WHOSE_LAST_CHILD_HOLDS_TWO,
        tombstone_capacities_of_one_record_and_two_entries(),
    );
    pool
}

/// `slot` 那个码 2 节点头里的最大 key 改成 `largest`（8 字节小端），重封，新的整单元校验和沿引用链补到根槽。
pub fn rewrite_the_largest_key_of_the_node_at(
    image: &mut MemoryPool,
    slot: singlefs_core::address::SlotNumber,
    largest: u64,
) {
    let key_width = index_node_header_of(&unit_bytes_at(image, slot, node_bytes())).key_width;
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |bytes| {
        let largest_at = INDEX_NODE_KEY_RANGE_OFFSET + key_width;
        bytes[largest_at..largest_at + key_width].copy_from_slice(&largest.to_le_bytes());
    });
}

/// 墓碑树层级 2 的根之下、层级 1 的 (1, 1)（装容器 3 与 4）头里的最大 key 改成 3：层级 1 的节点不贴紧它的首末两条容器号。
pub fn image_whose_tombstone_node_of_level_one_is_not_tight_over_its_containers(
    pool: &BuiltPool,
) -> MemoryPool {
    let mut image = pool.memory_pool();
    let slot = pool
        .output
        .unit(TransactionUnit::TombstoneNodeBelowTheRoot(
            singlefs_core::container_tree::ContainerTreeNodePosition {
                level: 1,
                index_in_level: 1,
            },
        ))
        .slot;
    rewrite_the_largest_key_of_the_node_at(&mut image, slot, 3);
    image
}

/// 谱系树长到层级 2、最后一个孩子装两片：谱系容器一片 1 条、节点 2 条，挂着回退到第一个文件那一版四次，每次追加一条谱系记录、开一片容器。
pub fn pool_whose_lineage_root_of_level_two_has_a_last_child_of_two_containers(
    tag: &str,
) -> BuiltPool {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    let capacities = lineage_capacities_of_one_record_and_two_entries();
    pool.allocator.set_deletion_switches(DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity: capacities.tombstone_container,
        lineage_container_capacity: capacities.lineage_container,
        retirement_container_capacity: capacities.retirement_container,
        container_tree_node_entries: capacities.node_entries,
    });
    // 迭代上界是常量 4（每次回退开一片谱系容器）。
    for _ in 0..CONTAINERS_OF_A_LEVEL_TWO_TREE_WHOSE_LAST_CHILD_HOLDS_TWO {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &publish_parameters,
            devices,
            &mut pool.allocator,
            &mut pool.output,
            RollbackTarget {
                instance: first_file_version.root.instance,
                checkpoint_txg: first_file_version.root.checkpoint_txg,
            },
        )
        .expect("回退到第一个文件那一版");
    }
    pool
}

/// 退役树长到层级 2、最后一个孩子装两片：退役容器一片 1 条、节点 2 条。新池新建文件之后建 233 个空 inode（叶 [1..=233]、[234]），
/// unlink 234 到底（最右那片删空，身份消失、退役一条）；再三次「建 1 个（最左那片满了，开一片新叶）、unlink 它到底（又退役一条）」。
pub fn pool_whose_retirement_root_of_level_two_has_a_last_child_of_two_containers(
    tag: &str,
) -> BuiltPool {
    let mut pool = pool_with_new_inodes(tag, INODE_LEAF_RECORDS);
    let capacities = retirement_capacities_of_one_record_and_two_entries();
    let mut next_inode = INODE_LEAF_RECORDS + 1;
    unlink_an_empty_inode_under_the_capacities(&mut pool, next_inode, capacities);
    // 迭代上界是常量 3（第一片退役已在上面那一次）。
    for _ in 1..CONTAINERS_OF_A_LEVEL_TWO_TREE_WHOSE_LAST_CHILD_HOLDS_TWO {
        next_inode += 1;
        publish_new_inodes_and_advance_the_pool(&mut pool, 1);
        unlink_an_empty_inode_under_the_capacities(&mut pool, next_inode, capacities);
    }
    pool
}

/// 一个码 2 单元的字节解开的头（解不开就是搭建坏了）。
pub fn index_node_header_of(bytes: &[u8]) -> IndexNodeHeader {
    parse_index_node(bytes).expect("写者写出的码 2 节点解得开")
}

/// 这一版里 `role` 那个码 2 单元头里的 key 区间，按 8 字节小端读成两个数（inode 号、容器号的 key 都是 8 字节）。
pub fn key_range_of_the_unit(
    output: &singlefs_core::transaction::TransactionOutput,
    role: TransactionUnit,
) -> (u64, u64) {
    let header = index_node_header_of(&output.unit(role).bytes);
    (
        u64::from_le_bytes(header.smallest_key[..8].try_into().expect("key 宽 8")),
        u64::from_le_bytes(header.largest_key[..8].try_into().expect("key 宽 8")),
    )
}

/// 这一版里 `role` 那个码 2 单元首末两条条目的 key（分隔 key，8 字节小端）。
pub fn first_and_last_entry_keys_of_the_unit(
    output: &singlefs_core::transaction::TransactionOutput,
    role: TransactionUnit,
) -> (u64, u64) {
    let header = index_node_header_of(&output.unit(role).bytes);
    let key_of = |entry: &Vec<u8>| u64::from_le_bytes(entry[..8].try_into().expect("key 宽 8"));
    (
        key_of(header.entries.first().expect("根至少一条条目")),
        key_of(header.entries.last().expect("根至少一条条目")),
    )
}

/// 最新那条根的树表里种类为 `kind` 的那棵树的根：头里的 key 区间改成首末两条条目的 key（C478 之前写者的写法），重封，
/// 新的整单元校验和沿引用链补到根槽。
pub fn rewrite_the_root_key_range_as_the_entry_key_range(image: &mut MemoryPool, kind: u16) {
    let root_slot = the_slot_of(&tree_table_entry_of_kind(image, kind).root);
    let before = index_node_header_of(&unit_bytes_at(image, root_slot, node_bytes()));
    let key_width = before.key_width;
    let first_key = before.entries.first().expect("根至少一条条目")[..key_width].to_vec();
    let last_key = before.entries.last().expect("根至少一条条目")[..key_width].to_vec();
    assert!(
        (first_key.as_slice(), last_key.as_slice())
            != (
                before.smallest_key.as_slice(),
                before.largest_key.as_slice()
            ),
        "种类 {kind} 的根上子树覆盖区间与首末两条条目的 key 本来就相同，改了也看不出判别力"
    );
    change_the_unit_and_propagate_its_checksum(image, root_slot, node_bytes(), |bytes| {
        let smallest_at = INDEX_NODE_KEY_RANGE_OFFSET;
        let largest_at = INDEX_NODE_KEY_RANGE_OFFSET + key_width;
        bytes[smallest_at..smallest_at + key_width].copy_from_slice(&first_key);
        bytes[largest_at..largest_at + key_width].copy_from_slice(&last_key);
    });
}

/// inode 树根改成首末两条条目的 key 那一张坏镜像。
pub fn image_whose_inode_root_key_range_is_the_entry_key_range(pool: &BuiltPool) -> MemoryPool {
    let mut image = pool.memory_pool();
    rewrite_the_root_key_range_as_the_entry_key_range(&mut image, TREE_KIND_INODE);
    image
}

/// 墓碑树根（层级 2）改成首末两条条目的 key 那一张坏镜像。
pub fn image_whose_tombstone_root_key_range_is_the_entry_key_range(pool: &BuiltPool) -> MemoryPool {
    let mut image = pool.memory_pool();
    rewrite_the_root_key_range_as_the_entry_key_range(&mut image, TREE_KIND_TOMBSTONE);
    image
}

/// 谱系树根（层级 2）改成首末两条条目的 key 那一张坏镜像。
pub fn image_whose_lineage_root_key_range_is_the_entry_key_range(pool: &BuiltPool) -> MemoryPool {
    let mut image = pool.memory_pool();
    rewrite_the_root_key_range_as_the_entry_key_range(&mut image, TREE_KIND_LINEAGE);
    image
}

/// 退役树根（层级 2）改成首末两条条目的 key 那一张坏镜像。
pub fn image_whose_retirement_root_key_range_is_the_entry_key_range(
    pool: &BuiltPool,
) -> MemoryPool {
    let mut image = pool.memory_pool();
    rewrite_the_root_key_range_as_the_entry_key_range(&mut image, TREE_KIND_RETIREMENT);
    image
}
