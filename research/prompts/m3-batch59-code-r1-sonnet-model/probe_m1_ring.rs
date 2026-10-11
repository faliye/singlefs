//! 正推腿探针（只放自己的拷贝里，不进仓）：M1 的版本级对照。
//! 照 checker 档甲补丁 `pool_written_under_these_knobs` 的三棵树造法（用 harness 的同名工具），
//! 写完之后每转一次根环（建 1 个空 inode）就取一次镜像，用带旋钮值 2 的 CPU 池级 checker 判，打印红的不变量与那一条的违例说明。
//! 同一份探针换三种 walk.rs（正确 / 只判倒数第二片 / 只判最右之前两片）各跑一次，逐行比。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{build_pool, parameters, BuiltPool};
use common_deletion_in_progress::{
    pool_with_new_inodes, publish_new_inodes_and_advance_the_pool, roll_back_to, FIRST_FILE,
    INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    check_pool_image_given_the_container_capacity_knobs, ContainerCapacityKnobs,
    LineageContainerCapacityKnob, RetirementContainerCapacityKnob, TombstoneContainerCapacityKnob,
};
use singlefs_core::container_tree::{ContainerTreeCapacities, ContainerTreeNodeEntries};
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty, PoolWriter,
};
use singlefs_core::unlink::UnlinkBatchSize;

#[derive(Clone, Copy, Debug)]
enum Tree {
    Tombstone,
    Lineage,
    Retirement,
}

fn capacities(tree: Tree, n: u64) -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: match tree {
            Tree::Tombstone => TombstoneContainerCapacity::CappedForTests(
                TombstoneRecordsPerContainerCappedForTests::new(n).expect("1..=815"),
            ),
            _ => TombstoneContainerCapacity::FromTheContainerFormat,
        },
        lineage_container: match tree {
            Tree::Lineage => LineageContainerCapacity::CappedForTests(
                singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(n)
                    .expect("1..=1359"),
            ),
            _ => LineageContainerCapacity::FromTheContainerFormat,
        },
        retirement_container: match tree {
            Tree::Retirement => {
                singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                    singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(n)
                        .expect("1..=959"),
                )
            }
            _ => singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        },
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

fn knobs(tree: Tree, n: u64) -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: matches!(tree, Tree::Tombstone)
            .then(|| TombstoneContainerCapacityKnob::new(n).expect("ok")),
        lineage: matches!(tree, Tree::Lineage)
            .then(|| LineageContainerCapacityKnob::new(n).expect("ok")),
        retirement: matches!(tree, Tree::Retirement)
            .then(|| RetirementContainerCapacityKnob::new(n).expect("ok")),
    }
}

fn unlink(pool: &mut BuiltPool, inode: u64, caps: ContainerTreeCapacities) {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("open");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(caps);
    let mut current = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        singlefs_core::address::InodeNumber(inode),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("intent");
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("steps");
    pool.output = current;
}

fn counts(tree: Tree, pool: &BuiltPool) -> Vec<usize> {
    match tree {
        Tree::Tombstone => pool.output.tombstone_containers.iter().map(|c| c.contents.records.len()).collect(),
        Tree::Lineage => pool.output.lineage_containers.iter().map(|c| c.contents.records.len()).collect(),
        Tree::Retirement => pool.output.retirement_containers.iter().map(|c| c.contents.records.len()).collect(),
    }
}

fn report(tree: Tree, pool: &BuiltPool, label: &str) {
    let image = pool.memory_pool();
    let mut out = String::new();
    for variant in ["base", "second_to_last", "k2", "k3"] {
        std::env::set_var("PROBE_VARIANT", variant);
        let verdicts = check_pool_image_given_the_container_capacity_knobs(&image, knobs(tree, 2));
        let red: Vec<String> = verdicts
            .iter()
            .filter_map(|(name, v)| match v {
                InvariantVerdict::Violated(d) => Some(format!("{name}@{}", d.chars().take(24).collect::<String>())),
                _ => None,
            })
            .collect();
        out.push_str(&format!(" {variant}={}", if red.is_empty() { "green".to_string() } else { red.join("+") }));
    }
    let released = pool.output.allocation_records.iter().filter(|r| r.is_released).count();
    eprintln!(
        "PROBE {tree:?} {label} txg={} counts={:?} alloc_records={} released={}{out}",
        pool.output.root.checkpoint_txg.0,
        counts(tree, pool),
        pool.output.allocation_records.len(),
        released,
    );
}

fn build(tree: Tree, tag: &str, layout: &[u64]) -> BuiltPool {
    match tree {
        Tree::Tombstone => {
            let mut pool = pool_with_new_inodes(tag, u64::try_from(layout.len()).unwrap());
            for (i, (inode, n)) in (FIRST_FILE.0 + 1..).zip(layout).enumerate() {
                unlink(&mut pool, inode, capacities(tree, *n));
                if i + 1 >= 4 { report(tree, &pool, &format!("write{}", i + 1)); }
            }
            pool
        }
        Tree::Lineage => {
            let mut pool = build_pool(tag);
            let first = pool.output.clone();
            publish_new_inodes_and_advance_the_pool(&mut pool, 1);
            for (i, n) in layout.iter().enumerate() {
                let c = capacities(tree, *n);
                pool.allocator.set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
                    batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
                    tombstone_container_capacity: c.tombstone_container,
                    lineage_container_capacity: c.lineage_container,
                    retirement_container_capacity: c.retirement_container,
                    container_tree_node_entries: c.node_entries,
                });
                roll_back_to(&mut pool, &first);
                if i + 1 >= 4 { report(tree, &pool, &format!("write{}", i + 1)); }
            }
            pool
        }
        Tree::Retirement => {
            let mut pool = pool_with_new_inodes(tag, 233);
            for (i, n) in layout.iter().enumerate() {
                if i > 0 {
                    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
                }
                let inode = pool.output.inode_number_watermark() - 1;
                unlink(&mut pool, inode, capacities(tree, *n));
                if i + 1 >= 4 { report(tree, &pool, &format!("write{}", i + 1)); }
            }
            pool
        }
    }
}

fn run(tree: Tree, layout_name: &str, layout: &[u64]) {
    let mut pool = build(tree, &format!("probe-m1-{tree:?}-{layout_name}"), layout);
    for j in 1..=30 {
        publish_new_inodes_and_advance_the_pool(&mut pool, 1);
        if [1, 12, 19, 20, 21, 22, 23, 24, 25, 30].contains(&j) {
            report(tree, &pool, &format!("{layout_name}-turn{j}"));
        }
    }
}

#[test]
fn probe_m1_a_short_container_before_a_full_one() {
    for tree in [Tree::Tombstone, Tree::Lineage, Tree::Retirement] {
        run(tree, "L1", &[2, 2, 1, 1, 2, 1]);
    }
}

#[test]
fn probe_m1_three_containers_left_of_the_rightmost() {
    run(Tree::Tombstone, "L2", &[2, 2, 1, 1, 2, 2, 2, 1]);
}
