//! 池级 checker 按子树覆盖区间判孩子是码 3 容器的码 2 节点头里的 key 区间（I-1.1，D18（块里携带什么信息） 已定项 2「区间取子树覆盖区间」，C478）：
//! inode 树的内部节点按它下面叶容器里的记录号判（`walk::Walk::walk_inode_internal_node`），墓碑树、谱系树、退役树层级 ≥ 2 的节点按第一个、
//! 最后一个孩子头里的 key 判（`walk::Walk::walk_the_internal_node_below_a_container_tree_entry`）。
//! 阳性对照是写者写出的几形（子树覆盖区间与首末两条条目的 key 不同）一条都不红、I-1.1 判到且成立；必红世界是同一张镜像把根头里的区间
//! 改回首末两条条目的 key（C478 之前写者的写法）——只红 I-1.1。搭建在 `common_subtree_coverage_key_range`；写者与 core 读者那一半在
//! `key_range_of_code_two_nodes_over_containers_is_the_subtree_coverage.rs`。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_subtree_coverage_key_range;

use common_subtree_coverage_key_range::{
    image_whose_inode_root_key_range_is_the_entry_key_range,
    image_whose_lineage_root_key_range_is_the_entry_key_range,
    image_whose_retirement_root_key_range_is_the_entry_key_range,
    image_whose_tombstone_node_of_level_one_is_not_tight_over_its_containers,
    image_whose_tombstone_root_key_range_is_the_entry_key_range,
    pool_whose_inode_root_covers_a_full_leaf_and_a_leaf_of_two_records,
    pool_whose_leftmost_inode_leaf_lost_its_first_record,
    pool_whose_lineage_root_of_level_two_has_a_last_child_of_two_containers,
    pool_whose_retirement_root_of_level_two_has_a_last_child_of_two_containers,
    pool_whose_tombstone_root_is_of_level_three,
    pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    check_pool_image, check_pool_image_given_the_container_capacity_knobs, ContainerCapacityKnobs,
    LineageContainerCapacityKnob, RetirementContainerCapacityKnob, TombstoneContainerCapacityKnob,
};
use singlefs_harness::memory_pool::MemoryPool;

/// 压小的那几棵容器树一片装几条（`common_subtree_coverage_key_range` 的旋钮），交给池级 checker 带旋钮值的入口。
const RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;

/// 池级 checker 怎么判这张镜像：产品入口，或带写的时候那一组容器旋钮值。
#[derive(Clone, Copy, Debug)]
enum CheckerEntry {
    Product,
    GivenTheKnobs(ContainerCapacityKnobs),
}

fn verdicts_of(image: &MemoryPool, entry: CheckerEntry) -> Vec<(&'static str, InvariantVerdict)> {
    match entry {
        CheckerEntry::Product => check_pool_image(image),
        CheckerEntry::GivenTheKnobs(knobs) => {
            check_pool_image_given_the_container_capacity_knobs(image, knobs)
        }
    }
}

fn violated_invariants(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<&'static str> {
    verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect()
}

fn key_range_verdict<'verdicts>(
    verdicts: &'verdicts [(&'static str, InvariantVerdict)],
) -> &'verdicts InvariantVerdict {
    &verdicts
        .iter()
        .find(|(name, _)| *name == "I-1.1")
        .expect("清单里有 I-1.1")
        .1
}

/// 写者写出的这一张：一条都不红，I-1.1 判到且成立。
fn written_image_holds_the_key_range_invariant(image: &MemoryPool, entry: CheckerEntry) {
    let verdicts = verdicts_of(image, entry);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "写者写出的子树覆盖区间一条都不红：{verdicts:?}"
    );
    assert!(
        matches!(key_range_verdict(&verdicts), InvariantVerdict::Holds),
        "I-1.1 判到且成立：{:?}",
        key_range_verdict(&verdicts)
    );
}

/// 根头里写成首末两条条目 key 的这一张：只红 I-1.1，违例说明里写着子树覆盖区间。
fn entry_key_range_reddens_only_the_key_range_invariant(image: &MemoryPool, entry: CheckerEntry) {
    let verdicts = verdicts_of(image, entry);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.1"],
        "只红 I-1.1：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = key_range_verdict(&verdicts) else {
        panic!("I-1.1 红");
    };
    assert!(
        detail.contains("子树覆盖区间"),
        "违例说明写的是子树覆盖区间：{detail}"
    );
}

fn tombstone_container_knobs() -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: Some(
            TombstoneContainerCapacityKnob::new(RECORDS_PER_CONTAINER_UNDER_THE_KNOB)
                .expect("1..=815"),
        ),
        lineage: None,
        retirement: None,
    }
}

fn tombstone_knobs() -> CheckerEntry {
    CheckerEntry::GivenTheKnobs(tombstone_container_knobs())
}

fn lineage_knobs() -> CheckerEntry {
    CheckerEntry::GivenTheKnobs(ContainerCapacityKnobs {
        tombstone: None,
        lineage: Some(
            LineageContainerCapacityKnob::new(RECORDS_PER_CONTAINER_UNDER_THE_KNOB)
                .expect("1..=1359"),
        ),
        retirement: None,
    })
}

fn retirement_knobs() -> CheckerEntry {
    CheckerEntry::GivenTheKnobs(ContainerCapacityKnobs {
        tombstone: None,
        lineage: None,
        retirement: Some(
            RetirementContainerCapacityKnob::new(RECORDS_PER_CONTAINER_UNDER_THE_KNOB)
                .expect("1..=959"),
        ),
    })
}

/// 阳性对照：inode 树根头里 [1, 235]（最右那片 [234, 235]）判成立。判别力：checker 照旧按首末两条条目判层级 1 的根，这一张红。
#[test]
fn an_inode_root_declaring_the_largest_record_of_its_rightmost_leaf_holds_the_key_range_invariant()
{
    let pool = pool_whose_inode_root_covers_a_full_leaf_and_a_leaf_of_two_records(
        "pool-checker-subtree-coverage-inode-root-holds",
    );
    written_image_holds_the_key_range_invariant(&pool.memory_pool(), CheckerEntry::Product);
}

/// 阳性对照：最左那片丢了打头那一条、根头里 [2, 3] 判成立。
#[test]
fn an_inode_root_declaring_the_smallest_record_of_its_leftmost_leaf_holds_the_key_range_invariant()
{
    let pool = pool_whose_leftmost_inode_leaf_lost_its_first_record(
        "pool-checker-subtree-coverage-inode-root-smallest-holds",
    );
    written_image_holds_the_key_range_invariant(&pool.memory_pool(), CheckerEntry::Product);
}

/// 必红世界：inode 树根头里写成 [1, 234]（首末两条分隔 key），子树覆盖 [1, 235]：只红 I-1.1。
/// 判别力：`walk_inode_internal_node` 不按叶序里的记录号判区间，这一张不红。
#[test]
fn an_inode_root_declaring_the_entry_key_range_reddens_only_the_key_range_invariant() {
    let pool = pool_whose_inode_root_covers_a_full_leaf_and_a_leaf_of_two_records(
        "pool-checker-subtree-coverage-inode-root-red",
    );
    entry_key_range_reddens_only_the_key_range_invariant(
        &image_whose_inode_root_key_range_is_the_entry_key_range(&pool),
        CheckerEntry::Product,
    );
}

/// 必红世界：最左那片丢了打头那一条，根头里写成 [1, 1]（分隔 key），子树覆盖 [2, 3]：只红 I-1.1。
#[test]
fn an_inode_root_declaring_its_separator_as_the_smallest_key_reddens_only_the_key_range_invariant()
{
    let pool = pool_whose_leftmost_inode_leaf_lost_its_first_record(
        "pool-checker-subtree-coverage-inode-root-smallest-red",
    );
    entry_key_range_reddens_only_the_key_range_invariant(
        &image_whose_inode_root_key_range_is_the_entry_key_range(&pool),
        CheckerEntry::Product,
    );
}

/// 阳性对照：墓碑树层级 2 的根头里 [1, 4] 判成立（带写的时候的墓碑容器旋钮值判）。
#[test]
fn a_tombstone_root_of_level_two_declaring_its_last_container_holds_the_key_range_invariant() {
    let pool = pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers(
        "pool-checker-subtree-coverage-tombstone-root-holds",
    );
    written_image_holds_the_key_range_invariant(&pool.memory_pool(), tombstone_knobs());
}

/// 必红世界：墓碑树层级 2 的根头里写成 [1, 3]（首末两条分隔 key），子树覆盖 [1, 4]：只红 I-1.1。
/// 判别力：读第二个孩子时不比父头里的最大 key，这一张不红；checker 照旧按首末两条条目判层级 2 的根，阳性对照那一张红。
#[test]
fn a_tombstone_root_declaring_the_entry_key_range_reddens_only_the_key_range_invariant() {
    let pool = pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers(
        "pool-checker-subtree-coverage-tombstone-root-red",
    );
    entry_key_range_reddens_only_the_key_range_invariant(
        &image_whose_tombstone_root_key_range_is_the_entry_key_range(&pool),
        tombstone_knobs(),
    );
}

/// 阳性对照：谱系树层级 2 的根头里 [1, 4] 判成立（带谱系容器旋钮值判）。
#[test]
fn a_lineage_root_of_level_two_declaring_its_last_container_holds_the_key_range_invariant() {
    let pool = pool_whose_lineage_root_of_level_two_has_a_last_child_of_two_containers(
        "pool-checker-subtree-coverage-lineage-root-holds",
    );
    written_image_holds_the_key_range_invariant(&pool.memory_pool(), lineage_knobs());
}

/// 必红世界：谱系树层级 2 的根头里写成 [1, 3]：只红 I-1.1。
#[test]
fn a_lineage_root_declaring_the_entry_key_range_reddens_only_the_key_range_invariant() {
    let pool = pool_whose_lineage_root_of_level_two_has_a_last_child_of_two_containers(
        "pool-checker-subtree-coverage-lineage-root-red",
    );
    entry_key_range_reddens_only_the_key_range_invariant(
        &image_whose_lineage_root_key_range_is_the_entry_key_range(&pool),
        lineage_knobs(),
    );
}

/// 阳性对照：退役树层级 2 的根头里 [1, 4] 判成立（带退役容器旋钮值判）。
#[test]
fn a_retirement_root_of_level_two_declaring_its_last_container_holds_the_key_range_invariant() {
    let pool = pool_whose_retirement_root_of_level_two_has_a_last_child_of_two_containers(
        "pool-checker-subtree-coverage-retirement-root-holds",
    );
    written_image_holds_the_key_range_invariant(&pool.memory_pool(), retirement_knobs());
}

/// 必红世界：退役树层级 2 的根头里写成 [1, 3]：只红 I-1.1。
#[test]
fn a_retirement_root_declaring_the_entry_key_range_reddens_only_the_key_range_invariant() {
    let pool = pool_whose_retirement_root_of_level_two_has_a_last_child_of_two_containers(
        "pool-checker-subtree-coverage-retirement-root-red",
    );
    entry_key_range_reddens_only_the_key_range_invariant(
        &image_whose_retirement_root_key_range_is_the_entry_key_range(&pool),
        retirement_knobs(),
    );
}

/// 阳性对照：墓碑树长到层级 3，层级 2 的 (2, 0) 头里 [1, 4]、根 [1, 5] 判成立。判别力：读根之下的码 2 节点时照旧按首末两条条目判
/// （`walk_the_internal_node_below_a_container_tree_entry` 读孩子不按层级 ≥ 2 只判分隔 key），(2, 0) 那一判红。
#[test]
fn a_tombstone_node_of_level_two_below_a_root_of_level_three_holds_the_key_range_invariant() {
    let pool = pool_whose_tombstone_root_is_of_level_three(
        "pool-checker-subtree-coverage-tombstone-level-three-holds",
    );
    written_image_holds_the_key_range_invariant(&pool.memory_pool(), tombstone_knobs());
}

/// 必红世界：层级 1 的 (1, 1) 头里写成 [3, 3]（它装容器 3 与 4）：只红 I-1.1（它自己不贴紧首末两条容器号，父头 [1, 4] 也不再是子树覆盖区间）。
#[test]
fn a_tombstone_node_of_level_one_not_tight_over_its_containers_reddens_only_the_key_range_invariant(
) {
    let pool = pool_whose_tombstone_root_of_level_two_has_a_last_child_of_two_containers(
        "pool-checker-subtree-coverage-tombstone-level-one-red",
    );
    let verdicts = check_pool_image_given_the_container_capacity_knobs(
        &image_whose_tombstone_node_of_level_one_is_not_tight_over_its_containers(&pool),
        tombstone_container_knobs(),
    );
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.1"],
        "只红 I-1.1：{verdicts:?}"
    );
}
