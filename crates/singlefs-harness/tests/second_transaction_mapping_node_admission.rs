//! 写侧准入的第三条（中央映射树）：映射树第一版也只有一个节点（key 27、条目 55，装 294 条），
//! 一次发布写的映射条目 = 进映射的五个固定角色（数据单元、extent 根、inode 根、分配记录树、记账树）
//! 加这一版每片 inode 叶容器各一条 ⇒ 叶容器多到 290 片时是 295 条，装不下。
//! 改之前分配记录树与记账树各有一条这样的准入，映射树一条都没有：装不下时直接走到 `build_index_node` 的断言 panic。
//!
//! 今天 panic 不了，靠的是两棵无关的树之间的数字巧合：inode 树根一个节点只装得下 135 片叶容器
//! （`InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds` 在发布路径之前就拒了），
//! 5 + 135 = 140 < 294。两个数各自由各自的字段表定，谁先动谁就把巧合弄没了 ⇒ 这一条准入自己判自己那棵树。
//! 本文件三条用例：把两个数与那五个固定角色钉死、正好装满的一次发布照常过准入、多一片叶容器在取号之前被拒且盘上逐字节不变。
//!
//! 让映射树能分裂不在这一轮里（2026-09-23 用户定案：树的分裂估计里程碑 4 或 5 才做），这一轮只加闸。

mod common;

use common::{build_pool, disk_snapshot};
use singlefs_core::inode_tree::leaf_containers_one_root_node_holds;
use singlefs_core::transaction::{
    publish_admission, publish_sequence_admission, PublishError, PublishShape,
};
use singlefs_core::unit::index_node_entry_capacity;
use singlefs_format::{MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES};

/// 映射树第一版那一个节点装得下几条条目。
fn mapping_node_capacity() -> usize {
    index_node_entry_capacity(
        usize::try_from(MAPPING_KEY_BYTES).expect("27"),
        usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
    )
}

/// 进映射而与 inode 叶容器无关的角色数：数据单元、extent 根、inode 根、分配记录树、记账树。
/// 这里另写一遍，不从核心层取：准入的算术要有一个独立的对照物，共用同一个常量就对照不出东西来。
/// 数据单元那一个是「文件只有一个数据单元」那一档（第一个事务的文件）：一个文件跨多个单元时每个单元各一条
/// （并行线一），那一档由 `second_transaction_parallel_line_one_multi_unit_file.rs` 判。
const MAPPING_ROLES_OUTSIDE_THE_INODE_LEAF_CONTAINERS: usize = 5;

/// 这几条用例的文件都是第一个事务写的那一个：一个数据单元。
const DATA_UNITS_OF_THE_FIRST_FILE: usize = 1;

/// 叶容器数取到这个值时，映射条目正好把一个节点装满。
fn inode_leaf_containers_that_exactly_fill_the_mapping_node() -> usize {
    mapping_node_capacity() - MAPPING_ROLES_OUTSIDE_THE_INODE_LEAF_CONTAINERS
}

#[test]
fn the_mapping_node_holds_two_hundred_ninety_four_entries_and_five_of_them_are_not_inode_leaf_containers(
) {
    assert_eq!(mapping_node_capacity(), 294, "key 27、条目 55 的码 2 节点");
    assert_eq!(
        inode_leaf_containers_that_exactly_fill_the_mapping_node(),
        289
    );
    // 今天走得到的最大条目数：inode 树根装得下几片叶，映射节点就最多这么多条加五条。
    assert_eq!(
        leaf_containers_one_root_node_holds(),
        135,
        "key 8、条目 120 的码 2 根"
    );
    assert_eq!(
        MAPPING_ROLES_OUTSIDE_THE_INODE_LEAF_CONTAINERS + leaf_containers_one_root_node_holds(),
        140,
        "今天的上界 140 < 294：映射树不翻车靠的是这个巧合，不是有人判过"
    );
}

#[test]
fn a_publish_whose_mapping_entries_exactly_fill_the_node_passes_admission() {
    let pool = build_pool("mapping-node-admission-exactly-full");
    let containers = inode_leaf_containers_that_exactly_fill_the_mapping_node();
    let shape = PublishShape::EMPTY_PUBLISH;
    publish_admission(
        &pool.allocator,
        &shape.rewritten_roles(),
        containers,
        DATA_UNITS_OF_THE_FIRST_FILE,
    )
    .expect("289 片叶容器：5 + 289 = 294 条，正好装满");
    publish_sequence_admission(
        &pool.allocator,
        &[shape],
        containers,
        DATA_UNITS_OF_THE_FIRST_FILE,
    )
    .expect("同一条算术，取号之前那一遍");
}

#[test]
fn one_inode_leaf_container_past_the_mapping_node_is_refused_before_the_instance_generation_is_acquired(
) {
    let pool = build_pool("mapping-node-admission-one-past-full");
    let containers = inode_leaf_containers_that_exactly_fill_the_mapping_node() + 1;
    let shape = PublishShape::ROW_PUBLISH;
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let records_before = pool.allocator.records().len();

    let refused = publish_admission(
        &pool.allocator,
        &shape.rewritten_roles(),
        containers,
        DATA_UNITS_OF_THE_FIRST_FILE,
    );
    assert!(
        matches!(
            refused,
            Err(PublishError::MappingEntriesExceedOneNode {
                entries: 295,
                capacity: 294
            })
        ),
        "290 片叶容器要 5 + 290 = 295 条：{refused:?}"
    );

    let refused_before_acquisition = publish_sequence_admission(
        &pool.allocator,
        &[shape],
        containers,
        DATA_UNITS_OF_THE_FIRST_FILE,
    );
    match refused_before_acquisition {
        Err(refusal) => {
            assert_eq!(refusal.publish_index, 0, "这一串里第 0 次就算不过");
            assert!(
                matches!(
                    refusal.cause,
                    PublishError::MappingEntriesExceedOneNode {
                        entries: 295,
                        capacity: 294
                    }
                ),
                "取号之前那一遍报的是同一个成员：{:?}",
                refusal.cause
            );
        }
        Ok(()) => panic!("取号之前那一遍也要算不过"),
    }

    assert_eq!(
        pool.allocator.records().len(),
        records_before,
        "准入只读：分配记录一条都没加"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "两盘系统配置槽逐字节不变、根环没有新根、录制流一步都没多"
    );
    // 拒了之后这个池照旧能发布：准入不留下半新的状态。
    let containers_that_fit = pool.output.inode_leaf_containers.len();
    publish_admission(
        &pool.allocator,
        &PublishShape::ROW_PUBLISH.rewritten_roles(),
        containers_that_fit,
        DATA_UNITS_OF_THE_FIRST_FILE,
    )
    .expect("这一版只有一片叶容器，6 条映射条目");
}

#[test]
fn a_published_version_writes_five_mapping_entries_plus_one_per_inode_leaf_container() {
    let pool = build_pool("mapping-node-admission-entry-count");
    assert_eq!(
        pool.output.mapping_keys.len(),
        MAPPING_ROLES_OUTSIDE_THE_INODE_LEAF_CONTAINERS + pool.output.inode_leaf_containers.len(),
        "准入算的条目数与真装进映射节点的条数是同一个"
    );
    assert_eq!(
        (
            pool.output.mapping_keys.len(),
            pool.output.inode_leaf_containers.len()
        ),
        (6, 1),
        "第一个文件版本：一片叶容器，六条映射条目"
    );
}
