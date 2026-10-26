//! 孩子是码 3 容器的码 2 节点头里的 key 区间取子树覆盖区间（D18（块里携带什么信息） 已定项 2「区间取子树覆盖区间」：`[min_key, max_key]`
//! 是这个节点的子树里到得了的最小 key 与最大 key；孩子是码 3 容器的内部节点上，max_key 是最后那片容器里最后一条记录的 key，
//! 不是最后一条条目的分隔 key；C478）：写者与 core 的读者那一半。
//!
//! 写者：inode 树根写 [最左那片的最小记录号, 最右那片的最大记录号]；墓碑树、谱系树、退役树（key 是容器号）层级 ≥ 2 的节点写
//! [第一个孩子那一枝最左那片的号, 最后一个孩子那一枝最右那片的号]。读者（冷走读 `recover`、可写挂载重建上一版、只读挂载）收这几形，
//! 拒根头里写成首末两条条目 key 的坏镜像（C478 之前写者的写法）。池级 checker 那一半在
//! `pool_checker_judges_the_key_range_of_code_two_nodes_over_containers_against_the_subtree.rs`，GPU 判器与 CPU 判法的对拍在 checker 档
//! `crash_judge_gpu_judges_the_subtree_coverage_key_range_like_the_cpu_judge.rs`。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_subtree_coverage_key_range;

use common::{file_content, parameters, BuiltPool};
use common_pointer_head_corruption::{
    recovery_failure, writable_devices_of, writable_mount_refused_before_any_write,
};
use common_subtree_coverage_key_range::{
    first_and_last_entry_keys_of_the_unit, image_whose_inode_root_key_range_is_the_entry_key_range,
    image_whose_lineage_root_key_range_is_the_entry_key_range,
    image_whose_retirement_root_key_range_is_the_entry_key_range,
    image_whose_tombstone_node_of_level_one_is_not_tight_over_its_containers,
    image_whose_tombstone_root_key_range_is_the_entry_key_range, key_range_of_the_unit,
    pool_whose_inode_root_covers_a_full_leaf_and_a_leaf_of_two_records,
    pool_whose_leftmost_inode_leaf_lost_its_first_record,
    pool_whose_lineage_root_of_level_two_has_a_last_child_of_two_containers,
    pool_whose_retirement_root_of_level_two_has_a_last_child_of_two_containers,
    pool_whose_tombstone_root_is_of_level_three,
    pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers,
};
use singlefs_core::container_tree::ContainerTreeNodePosition;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure, OpenPoolForReadFailure};
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RecoveryFailure, RecoveryOutcome,
    CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
    INODE_TREE_ROOT_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
};
use singlefs_core::transaction::TransactionUnit;
use singlefs_harness::memory_pool::MemoryPool;

/// 这一版第一个文件还在时冷走读读回它的内容，删完了答「没有文件」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstFileOfTheVersion {
    Present,
    Deleted,
}

/// 写者写出的这一版，四样读者都收：从盘上重建上一版、冷走读（读回第一个文件，或答没有文件）、只读挂载、可写挂载。
fn every_core_reader_accepts(pool: &BuiltPool, first_file: FirstFileOfTheVersion) {
    let image = pool.memory_pool();
    if let Err(failure) =
        rebuild_version(&image, &pool.output.root, Some(pool.output.record.clone()))
    {
        panic!("从盘上重建这一版被拒：{failure:?}");
    }
    let root = (pool.output.root.instance, pool.output.root.checkpoint_txg);
    let expected_outcome = match first_file {
        FirstFileOfTheVersion::Present => RecoveryOutcome::FileRead {
            root,
            content: file_content(),
        },
        FirstFileOfTheVersion::Deleted => RecoveryOutcome::NoFile { root },
    };
    assert_eq!(
        recover(&image, JournalPolicy::Consult).outcome,
        expected_outcome,
        "冷走读收这一版"
    );
    if let Err(failure) = mount_read_only(&image) {
        panic!("只读挂载被拒：{failure:?}");
    }
    let mut devices = writable_devices_of(&image);
    if let Err(refusal) = mount_writable(&parameters(), &mut devices) {
        panic!("可写挂载被拒：{refusal:?}");
    }
}

/// 冷走读拒这张坏镜像，报的是 I-1.1 那一条与 `detail` 那一句。
fn cold_walk_refuses_with(image: &MemoryPool, detail: &'static str) {
    assert_eq!(
        recovery_failure(image),
        RecoveryFailure::InvariantViolated {
            invariant: "I-1.1",
            detail,
        },
        "冷走读拒：根头里的 key 区间不是子树覆盖区间"
    );
}

/// 可写挂载在任何写之前拒这张坏镜像（重建上一版那一步），拒的说明里带 `detail` 那一句。
fn writable_mount_refuses_with(image: &MemoryPool, detail: &'static str) {
    let refusal = writable_mount_refused_before_any_write(image);
    assert!(
        format!("{refusal:?}").contains(detail),
        "可写挂载拒在根头里的 key 区间上：{refusal:?}"
    );
}

/// 最右那片装到第二条记录：根头里的最大 key 是 235（最右那片最后一条记录），不是末条条目的分隔 key 234；最小 key 1。四样读者都收。
/// 判别力：写者照旧取末条条目的分隔 key（[1, 234]），这里的区间断言红；读者对层级 1 的根照旧按首末条目判，读者那一判红。
#[test]
fn an_inode_root_over_a_full_leaf_and_a_leaf_of_two_records_declares_the_largest_record_number_and_every_reader_accepts_it(
) {
    let pool = pool_whose_inode_root_covers_a_full_leaf_and_a_leaf_of_two_records(
        "subtree-coverage-inode-root-largest-record",
    );
    assert_eq!(
        first_and_last_entry_keys_of_the_unit(&pool.output, TransactionUnit::InodeRoot),
        (1, 234),
        "两片叶的分隔 key：容器号 1 与触发分裂那条记录的号 234"
    );
    assert_eq!(
        key_range_of_the_unit(&pool.output, TransactionUnit::InodeRoot),
        (1, 235),
        "inode 树根头里的 key 区间是子树覆盖区间 [最左那片的最小记录号, 最右那片的最大记录号]"
    );
    every_core_reader_accepts(&pool, FirstFileOfTheVersion::Present);
}

/// 第一个文件 unlink 到底之后最左那片只剩 [2, 3]、分隔 key 照旧是 1：根头里的最小 key 是 2（子树里到得了的最小记录号），不是分隔 key 1。
/// 四样读者都收（冷走读答没有文件）。判别力：写者照旧取首条条目的分隔 key，这里的区间断言红。
#[test]
fn an_inode_root_whose_leftmost_leaf_lost_its_first_record_declares_its_smallest_record_number_and_every_reader_accepts_it(
) {
    let pool = pool_whose_leftmost_inode_leaf_lost_its_first_record(
        "subtree-coverage-inode-root-smallest-record",
    );
    assert_eq!(
        first_and_last_entry_keys_of_the_unit(&pool.output, TransactionUnit::InodeRoot),
        (1, 1),
        "一片叶，分隔 key 是诞生时的最小 key 1"
    );
    assert_eq!(
        key_range_of_the_unit(&pool.output, TransactionUnit::InodeRoot),
        (2, 3),
        "inode 树根头里的 key 区间是子树里到得了的最小、最大记录号"
    );
    every_core_reader_accepts(&pool, FirstFileOfTheVersion::Deleted);
}

/// 墓碑树长到层级 2、(1, 1) 装容器 3 与 4：根头里的最大 key 是 4（最后一个孩子那一枝最右那片的号），不是末条条目的分隔 key 3；
/// 层级 1 的 (1, 1) 头里是 [3, 4]（贴紧它的首末两条容器号）。四样读者都收。
#[test]
fn a_tombstone_root_of_level_two_declares_the_last_container_of_its_last_child_and_every_reader_accepts_it(
) {
    let pool = pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers(
        "subtree-coverage-tombstone-root",
    );
    assert_eq!(pool.output.tombstone_tree_index_nodes.shape.root_level(), 2);
    assert_eq!(
        first_and_last_entry_keys_of_the_unit(&pool.output, TransactionUnit::TombstoneRoot),
        (1, 3),
        "根的两条条目：(1, 0) 那一枝最左那片的号 1、(1, 1) 那一枝最左那片的号 3"
    );
    assert_eq!(
        key_range_of_the_unit(&pool.output, TransactionUnit::TombstoneRoot),
        (1, 4),
        "层级 2 的根头里的 key 区间是子树覆盖区间 [1, 4]"
    );
    assert_eq!(
        key_range_of_the_unit(
            &pool.output,
            TransactionUnit::TombstoneNodeBelowTheRoot(ContainerTreeNodePosition {
                level: 1,
                index_in_level: 1,
            })
        ),
        (3, 4),
        "层级 1 的节点贴紧首末两条容器号"
    );
    every_core_reader_accepts(&pool, FirstFileOfTheVersion::Present);
}

/// 谱系树长到层级 2（回退四次）：根头里的最大 key 是 4，不是末条条目的分隔 key 3。四样读者都收。
#[test]
fn a_lineage_root_of_level_two_declares_the_last_container_of_its_last_child_and_every_reader_accepts_it(
) {
    let pool = pool_whose_lineage_root_of_level_two_has_a_last_child_of_two_containers(
        "subtree-coverage-lineage-root",
    );
    assert_eq!(pool.output.lineage_tree_index_nodes.shape.root_level(), 2);
    assert_eq!(
        first_and_last_entry_keys_of_the_unit(&pool.output, TransactionUnit::LineageRoot),
        (1, 3),
        "根的两条条目的分隔 key"
    );
    assert_eq!(
        key_range_of_the_unit(&pool.output, TransactionUnit::LineageRoot),
        (1, 4),
        "层级 2 的谱系树根头里的 key 区间是子树覆盖区间 [1, 4]"
    );
    every_core_reader_accepts(&pool, FirstFileOfTheVersion::Present);
}

/// 退役树长到层级 2（四片叶各退役一次）：根头里的最大 key 是 4，不是末条条目的分隔 key 3。四样读者都收。
#[test]
fn a_retirement_root_of_level_two_declares_the_last_container_of_its_last_child_and_every_reader_accepts_it(
) {
    let pool = pool_whose_retirement_root_of_level_two_has_a_last_child_of_two_containers(
        "subtree-coverage-retirement-root",
    );
    assert_eq!(
        pool.output.retirement_tree_index_nodes.shape.root_level(),
        2
    );
    assert_eq!(
        first_and_last_entry_keys_of_the_unit(&pool.output, TransactionUnit::RetirementRoot),
        (1, 3),
        "根的两条条目的分隔 key"
    );
    assert_eq!(
        key_range_of_the_unit(&pool.output, TransactionUnit::RetirementRoot),
        (1, 4),
        "层级 2 的退役树根头里的 key 区间是子树覆盖区间 [1, 4]"
    );
    every_core_reader_accepts(&pool, FirstFileOfTheVersion::Present);
}

/// inode 树根头里写成首末两条条目的 key [1, 234]（子树覆盖 [1, 235]）：冷走读、只读挂载、可写挂载都拒在 I-1.1 上。
/// 判别力：读完叶容器那一判（`recovery::inode_tree_root_key_range_is_the_subtree_coverage`）不调，三样读者都收下这张镜像。
#[test]
fn an_inode_root_declaring_the_entry_key_range_is_refused_by_every_core_reader() {
    let pool = pool_whose_inode_root_covers_a_full_leaf_and_a_leaf_of_two_records(
        "subtree-coverage-inode-root-refused",
    );
    let image = image_whose_inode_root_key_range_is_the_entry_key_range(&pool);
    cold_walk_refuses_with(
        &image,
        INODE_TREE_ROOT_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
    );
    match mount_read_only(&image) {
        Err(MountReadOnlyFailure::Open(OpenPoolForReadFailure::Walk(failure))) => assert_eq!(
            failure,
            RecoveryFailure::InvariantViolated {
                invariant: "I-1.1",
                detail: INODE_TREE_ROOT_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
            },
            "只读挂载打开挂载态时拒在 inode 树根的 key 区间上"
        ),
        Err(other) => panic!("只读挂载拒错了地方：{other:?}"),
        Ok(_) => panic!("只读挂载收下了根头里写成条目 key 区间的 inode 树"),
    }
    writable_mount_refuses_with(
        &image,
        INODE_TREE_ROOT_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
    );
}

/// 墓碑树层级 2 的根头里写成首末两条条目的 key [1, 3]（子树覆盖 [1, 4]）：冷走读与可写挂载拒在 I-1.1 上。
/// 判别力：读容器树码 2 节点时不按最后一个孩子头里的最大 key 判，两样读者都收下。
#[test]
fn a_tombstone_root_declaring_the_entry_key_range_is_refused_by_the_cold_walk_and_the_writable_mount(
) {
    let pool = pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers(
        "subtree-coverage-tombstone-root-refused",
    );
    let image = image_whose_tombstone_root_key_range_is_the_entry_key_range(&pool);
    cold_walk_refuses_with(
        &image,
        CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
    );
    writable_mount_refuses_with(
        &image,
        CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
    );
}

/// 谱系树同形：冷走读与可写挂载拒在 I-1.1 上。
#[test]
fn a_lineage_root_declaring_the_entry_key_range_is_refused_by_the_cold_walk_and_the_writable_mount()
{
    let pool = pool_whose_lineage_root_of_level_two_has_a_last_child_of_two_containers(
        "subtree-coverage-lineage-root-refused",
    );
    let image = image_whose_lineage_root_key_range_is_the_entry_key_range(&pool);
    cold_walk_refuses_with(
        &image,
        CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
    );
    writable_mount_refuses_with(
        &image,
        CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
    );
}

/// 退役树同形：冷走读与可写挂载拒在 I-1.1 上。
#[test]
fn a_retirement_root_declaring_the_entry_key_range_is_refused_by_the_cold_walk_and_the_writable_mount(
) {
    let pool = pool_whose_retirement_root_of_level_two_has_a_last_child_of_two_containers(
        "subtree-coverage-retirement-root-refused",
    );
    let image = image_whose_retirement_root_key_range_is_the_entry_key_range(&pool);
    cold_walk_refuses_with(
        &image,
        CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
    );
    writable_mount_refuses_with(
        &image,
        CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE,
    );
}

/// 墓碑树长到层级 3：层级 2 的 (2, 0) 头里是 [1, 4]（它下面 (1, 1) 那一枝最右那片的号），不是它首末两条条目的分隔 key [1, 3]；根是 [1, 5]。
/// 四样读者都收。判别力：写者对层级 ≥ 2 的节点照旧取末条条目的分隔 key，(2, 0) 那一断言红。
#[test]
fn a_tombstone_node_of_level_two_below_a_root_of_level_three_declares_the_last_container_of_its_subtree_and_every_reader_accepts_it(
) {
    let pool =
        pool_whose_tombstone_root_is_of_level_three("subtree-coverage-tombstone-level-three");
    assert_eq!(pool.output.tombstone_tree_index_nodes.shape.root_level(), 3);
    let node_of_level_two = TransactionUnit::TombstoneNodeBelowTheRoot(ContainerTreeNodePosition {
        level: 2,
        index_in_level: 0,
    });
    assert_eq!(
        first_and_last_entry_keys_of_the_unit(&pool.output, node_of_level_two),
        (1, 3),
        "(2, 0) 的两条条目的分隔 key"
    );
    assert_eq!(
        key_range_of_the_unit(&pool.output, node_of_level_two),
        (1, 4),
        "层级 2 的节点头里的 key 区间是子树覆盖区间 [1, 4]"
    );
    assert_eq!(
        key_range_of_the_unit(&pool.output, TransactionUnit::TombstoneRoot),
        (1, 5),
        "层级 3 的根头里的 key 区间是子树覆盖区间 [1, 5]"
    );
    every_core_reader_accepts(&pool, FirstFileOfTheVersion::Present);
}

/// 墓碑树层级 1 的 (1, 1) 头里写成 [3, 3]（它装容器 3 与 4）：层级 1 的节点不贴紧首末两条容器号，冷走读与可写挂载拒在
/// 「根 key 区间与条目不符」上（读到这个孩子时先判它自己，再判父头的子树覆盖区间）。
/// 判别力：读容器树码 2 节点时不判层级 1 的节点贴紧首末条目（`recovery::container_tree_node_of_level_one_is_tight_over_its_entries`），
/// 冷走读改拒在父头的子树覆盖区间上，报的那一句不同。
#[test]
fn a_tombstone_node_of_level_one_not_tight_over_its_containers_is_refused_by_the_cold_walk_and_the_writable_mount(
) {
    let pool = pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers(
        "subtree-coverage-tombstone-level-one-not-tight",
    );
    let image = image_whose_tombstone_node_of_level_one_is_not_tight_over_its_containers(&pool);
    cold_walk_refuses_with(&image, "根 key 区间与条目不符");
    writable_mount_refuses_with(&image, "根 key 区间与条目不符");
}
