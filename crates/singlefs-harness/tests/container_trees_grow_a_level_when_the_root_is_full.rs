//! 墓碑树与谱系树根满长高（D18（块里携带什么信息） 已定项 24 第 1、5 条「根满时长高，照 D8（核心索引结构） 已定项 6」，用户定案 K-R3-3）：
//! 码 2 节点装满时末尾分裂，根满时长一层——新根与分出的右半各一个码 2，旧根一个字节不动、留在原位置当根之下的节点；
//! 只供测试的旋钮把容器与节点压小（一片 1 条、一个节点 2 条），旋钮不落盘；重建读层级 > 1 的根。池是两块 4 GiB 盘。

mod common;

use common::{build_pool, file_content, parameters, BuiltPool};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::admission::{
    central_mapping_entries_of_one_lineage_container_cow,
    checkpoint_cost_of_one_lineage_container_cow, MetadataBlocks,
};
use singlefs_core::allocator::Placement;
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
    ContainerTreeNodePosition,
};
use singlefs_core::lineage_tree::{
    LineageContainerCapacity, LineageRecordsPerContainerCappedForTests,
};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::records::STATISTIC_COMMITTED_RESERVATION_BYTES;
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
};
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    publish_new_inodes, publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty,
    PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};

/// 删除与回退都在新池新建文件那个实例里发。
const INSTANCE_OF_THE_PUBLISHES: InstanceGeneration = InstanceGeneration(1);

/// 一片容器 1 条、一个码 2 节点 2 条：第 3 片容器根满长一层，第 5 片长到层级 3。
fn capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(1).expect("1..=2039"),
        ),
        node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
        ),
    }
}

fn position(level: u8, index_in_level: u32) -> ContainerTreeNodePosition {
    ContainerTreeNodePosition {
        level,
        index_in_level,
    }
}

/// 建空 inode 那次的写入时间；unlink 它们的意图照传这一个（记录现有的时间，盘上字节与改之前相同）。
const WRITE_TIME_SECONDS_OF_THE_NEW_INODES: u64 = 1_788_000_100;

/// 新池新建文件之后建 `count` 个空 inode（号从 2 起）。
fn pool_with_new_inodes(tag: &str, count: u64) -> BuiltPool {
    let mut pool = build_pool(tag);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        count,
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("建空 inode");
    pool
}

/// unlink 一个空 inode：意图那次（交回它），再走空（空 inode 没有数据单元，直接是最后一批）。写入口装 `capacities`。
fn unlink(
    pool: &mut BuiltPool,
    inode: u64,
    capacities: ContainerTreeCapacities,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities);
    let intent = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    let mut current = intent.clone();
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("最后一批");
    pool.output = current;
    intent
}

fn rebuilt_from_disk(pool: &BuiltPool) -> TransactionOutput {
    match rebuild_version(
        &pool.memory_pool(),
        &pool.output.root,
        Some(pool.output.record.clone()),
    )
    .expect("从盘上重建")
    {
        RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version) => version,
        RebuiltVersion::WithoutFile => panic!("有树的一版"),
    }
}

fn violations(pool: &BuiltPool) -> Vec<String> {
    check_pool_image(&pool.memory_pool())
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn tags_of_the_tombstone_tree(roles: &[TransactionUnit]) -> Vec<String> {
    roles
        .iter()
        .filter(|role| {
            matches!(
                role,
                TransactionUnit::TombstoneLeafContainer(_)
                    | TransactionUnit::TombstoneRoot
                    | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            )
        })
        .map(|role| role.tag())
        .collect()
}

fn committed_reservation_of(version: &TransactionOutput) -> u64 {
    version
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == STATISTIC_COMMITTED_RESERVATION_BYTES)
        .expect("带树的一版记着标签 6")
        .value
}

/// 一片 1 条、一个节点 2 条：第 3 次 unlink 的意图那次墓碑树根满长一层——重写新开的容器 tt@2、右半 (1, 1)、新根，旧根不重写、
/// 不释放（它的落点在这一版里成了 (1, 0) 那个节点、照旧已分配）；墓碑预付按 H = 1 记（副本数 × ((2 + 1 + 1) × 16384 + 41 × 3 个活对象)）；
/// 从盘上重建读回层级 2 的根与它下面的两个节点；冷启动恢复读回第一个文件；池级 checker 全绿。
#[test]
fn the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place() {
    let mut pool = pool_with_new_inodes("tombstone-tree-grows-a-level", 5);
    let capacities = capacities_of_one_record_and_two_entries();
    unlink(&mut pool, 2, capacities);
    unlink(&mut pool, 3, capacities);
    let before_the_growth = pool.output.clone();
    let old_root = before_the_growth
        .unit(TransactionUnit::TombstoneRoot)
        .clone();
    let growth = unlink(&mut pool, 4, capacities);
    assert_eq!(
        tags_of_the_tombstone_tree(&growth.rewritten),
        ["tt@2", "tt^1.1", "tt"],
        "长高那次重写新开的容器、右半与新根，旧根不重写"
    );
    assert_eq!(growth.tombstone_tree_index_nodes.shape.root_level(), 2);
    let carried_down = growth.unit(TransactionUnit::TombstoneNodeBelowTheRoot(position(1, 0)));
    assert_eq!(
        (carried_down.slot, &carried_down.bytes),
        (old_root.slot, &old_root.bytes),
        "旧根一个字节不动、留在原位置当 (1, 0) 那个节点"
    );
    let old_root_placement = Placement {
        slot: old_root.slot,
        span: TransactionUnit::TombstoneRoot.span_slots(),
    };
    assert!(
        !growth.released.contains(&old_root_placement),
        "旧根不被这次换下：{:?}",
        growth.released
    );
    assert_eq!(
        committed_reservation_of(&growth),
        2 * ((2 + 1 + 1) * 16_384 + 41 * 3),
        "墓碑预付按长高之后的 H = 1：inode 1、5、6 三个活对象（2、3 删完，4 删除中）"
    );
    let rebuilt = rebuilt_from_disk(&pool);
    assert_eq!(
        (
            &rebuilt.tombstone_containers,
            &rebuilt.tombstone_tree_index_nodes
        ),
        (
            &pool.output.tombstone_containers,
            &pool.output.tombstone_tree_index_nodes
        ),
        "从盘上重建读回层级 2 的墓碑树：每片容器、形状与根之下节点的指针逐项相等"
    );
    assert_eq!(
        recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (pool.output.root.instance, pool.output.root.checkpoint_txg),
            content: file_content(),
        },
        "冷启动恢复（映射条目数把墓碑树每个节点都算上）读回第一个文件"
    );
    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
}

/// 再 unlink 两个：第 5 片时层级 1 与根都满，长到层级 3（新开 (1, 2)、(2, 1)、新根 (3, 0)）；重建读回层级 3 的根，池级 checker 全绿。
#[test]
fn the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it() {
    let mut pool = pool_with_new_inodes("tombstone-tree-grows-to-three-levels", 5);
    let capacities = capacities_of_one_record_and_two_entries();
    for inode in 2..=5 {
        unlink(&mut pool, inode, capacities);
    }
    let growth = unlink(&mut pool, 6, capacities);
    assert_eq!(
        tags_of_the_tombstone_tree(&growth.rewritten),
        ["tt@4", "tt^1.2", "tt^2.1", "tt"],
        "层级 1 与根都满：新开的容器、(1, 2)、(2, 1)、新根"
    );
    assert_eq!(growth.tombstone_tree_index_nodes.shape.root_level(), 3);
    let rebuilt = rebuilt_from_disk(&pool);
    assert_eq!(
        (
            &rebuilt.tombstone_containers,
            &rebuilt.tombstone_tree_index_nodes
        ),
        (
            &pool.output.tombstone_containers,
            &pool.output.tombstone_tree_index_nodes
        ),
        "从盘上重建读回层级 3 的墓碑树"
    );
    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
}

/// 旋钮不落盘：压小扇出写出来的两层树，换回产品扇出（135）接着写，照盘上的形状往最右那个层级 1 节点里挂（它只有 1 个孩子），不再长高。
#[test]
fn a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout() {
    let mut pool = pool_with_new_inodes("tombstone-tree-capped-then-product-fanout", 5);
    let capped = capacities_of_one_record_and_two_entries();
    for inode in 2..=4 {
        unlink(&mut pool, inode, capped);
    }
    let product_fanout = ContainerTreeCapacities {
        tombstone_container: capped.tombstone_container,
        lineage_container: capped.lineage_container,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    let next = unlink(&mut pool, 5, product_fanout);
    assert_eq!(
        tags_of_the_tombstone_tree(&next.rewritten),
        ["tt@3", "tt^1.1", "tt"],
        "产品扇出下 (1, 1) 还装得下：挂进去、重写它与根，根层级仍 2"
    );
    assert_eq!(next.tombstone_tree_index_nodes.shape.root_level(), 2);
    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
}

/// 谱系树同一条（第 5 条「谱系树根满同样长高」）：一片 1 条、一个节点 2 条，回退三次之后谱系树两层；从盘上重建读回层级 2 的根；
/// ckpt_cost 谱系那一项按内部 1 层算 4 块、中央映射树那一项的删插多算 3 把（容器、根、内部 1 层）；池级 checker 全绿。
#[test]
fn the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level() {
    let mut pool = build_pool("lineage-tree-grows-a-level");
    let first_file_version = pool.output.clone();
    let capacities = capacities_of_one_record_and_two_entries();
    pool.allocator.set_deletion_switches(DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity: capacities.tombstone_container,
        lineage_container_capacity: capacities.lineage_container,
        container_tree_node_entries: capacities.node_entries,
    });
    for _ in 0..3 {
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
    assert_eq!(
        pool.output
            .rewritten
            .iter()
            .filter(|role| matches!(
                role,
                TransactionUnit::LineageLeafContainer(_)
                    | TransactionUnit::LineageRoot
                    | TransactionUnit::LineageNodeBelowTheRoot(_)
            ))
            .map(|role| role.tag())
            .collect::<Vec<_>>(),
        ["tl@2", "tl^1.1", "tl"],
        "第三次回退：谱系树根满长一层"
    );
    assert_eq!(pool.output.lineage_tree_internal_levels(), 1);
    assert_eq!(
        (
            checkpoint_cost_of_one_lineage_container_cow(
                pool.output.lineage_tree_internal_levels()
            ),
            central_mapping_entries_of_one_lineage_container_cow(
                pool.output.lineage_tree_internal_levels()
            ),
        ),
        (MetadataBlocks(4), 3),
        "谱系那一项：容器 2 + 根 1 + 内部 1 层；映射里容器、根、内部节点各一把"
    );
    let rebuilt = rebuilt_from_disk(&pool);
    assert_eq!(
        (
            &rebuilt.lineage_containers,
            &rebuilt.lineage_tree_index_nodes
        ),
        (
            &pool.output.lineage_containers,
            &pool.output.lineage_tree_index_nodes
        ),
        "从盘上重建读回层级 2 的谱系树"
    );
    assert_eq!(
        recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (pool.output.root.instance, pool.output.root.checkpoint_txg),
            content: file_content(),
        },
        "冷启动恢复读回第一个文件"
    );
    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
}
