//! m3-retire-r1 攻方腿（opus）在冻结树副本上量 `crates/` 今天的纯函数：不建盘、不发写、不碰分配器的落点。
//! 三件：① 删空一片叶时右边各片左移、全部重写（`inode_tree::remove_record_from_leaf_containers`），一次删除类发布的
//! inode 叶角色数对 W 里分配记录树之外那 24 槽；② 容器树（墓碑、谱系共用，退役树若照搬也用它）长高那次发布写的块数，
//! 对「容器 2 + 根 1 + 内部层数」按发布之前、之后的层数各算一次；③ 两块 4 GiB 盘的池上 W（环窗口 ÷ 环里根数 + 1）。
//! 每行以 `R ` 打头，便于 compare 取。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, TreeIdentifier};
use singlefs_core::admission::{
    checkpoint_cost_of_one_lineage_container_cow, ring_window_on_one_device,
    DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST,
};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
use singlefs_core::container_tree::{
    append_record_to_the_open_container, ContainerTreeAfterThisPublish, ContainerTreeNodeEntries,
    ContainerTreeNodeEntriesCappedForTests, ContainerTreeShape,
};
use singlefs_core::inode_tree::{
    leaf_containers_one_root_node_holds, remove_record_from_leaf_containers, InodeLeafContainer,
};
use singlefs_core::records::{InodeRecord, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::unit::{PackedIdentity, PACKED_TYPE_INODE};
use singlefs_format::ROOT_RING_REGIONS;

const INODE_TREE: TreeIdentifier = TreeIdentifier(65537);
const LEAF_SLOTS: u64 = 2;

fn record(inode: u64) -> InodeRecord {
    InodeRecord {
        inode,
        object_birth: CheckpointTxg(3),
        nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
        size: 0,
        change_count: 3,
        write_time_seconds: 0,
    }
}

/// n 片叶，第 `lone` 片只装 1 条、别的各装 2 条，号严格递增；容器号 = 每片第一条的号。
fn leaves(n: usize, lone: usize) -> Vec<InodeLeafContainer> {
    let mut next = 1;
    (0..n)
        .map(|position| {
            let count = if position == lone { 1 } else { 2 };
            let records: Vec<InodeRecord> = (0..count).map(|k| record(next + k)).collect();
            let first = next;
            next += count;
            InodeLeafContainer {
                identity: PackedIdentity {
                    birth_tree: INODE_TREE,
                    record_type: PACKED_TYPE_INODE,
                    container: first,
                    container_birth: CheckpointTxg(3),
                },
                records,
            }
        })
        .collect()
}

#[test]
fn opus_retire_r1_left_shift_on_a_dropped_leaf() {
    let outside = DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST;
    let most = leaf_containers_one_root_node_holds();
    for n in [2usize, 3, 9, 10, 30, most] {
        for lone in [0usize, n / 2, n - 1] {
            let before = leaves(n, lone);
            let inode = before[lone].records[0].inode;
            let after = remove_record_from_leaf_containers(&before, InodeNumber(inode))
                .expect("删的那条在树里");
            let leaf_slots = LEAF_SLOTS * u64::try_from(after.rewritten.len()).expect("片数");
            println!(
                "R left_shift leaves={n} dropped_position={lone} dropped={} rewritten_leaves={} inode_leaf_slots={leaf_slots} \
                 plus_inode_root=1 plus_retire_container_and_root=3 sum={} w_outside_allocation_record_tree={outside} exceeds={}",
                after.containers_dropped,
                after.rewritten.len(),
                leaf_slots + 1 + 3,
                leaf_slots + 1 + 3 > outside
            );
        }
    }
}

fn capped(entries: usize) -> ContainerTreeNodeEntries {
    ContainerTreeNodeEntries::CappedForTests(
        ContainerTreeNodeEntriesCappedForTests::new(entries).expect("2..=135"),
    )
}

/// 一片容器装 1 条（每条记录开一片新容器）、一个节点装 `entries` 条：逐条追加，记下每次发布写的块数与两种算法的项。
fn growth_rows(label: &str, entries: ContainerTreeNodeEntries, appends: u64) {
    let mut after: ContainerTreeAfterThisPublish<u64> =
        ContainerTreeAfterThisPublish::carried(Vec::new(), ContainerTreeShape::zero_root());
    let mut short = 0u64;
    for k in 1..=appends {
        let levels_before = after.shape.internal_levels();
        after = append_record_to_the_open_container(
            &after.containers,
            &after.shape,
            k,
            CheckpointTxg(k + 3),
            TreeIdentifier(65546),
            5,
            1,
            entries,
        );
        let written = LEAF_SLOTS * u64::try_from(after.rewritten_containers.len()).expect("片数")
            + u64::try_from(after.rewritten_nodes().len()).expect("节点数");
        let term_before = checkpoint_cost_of_one_lineage_container_cow(levels_before).0;
        let term_after = checkpoint_cost_of_one_lineage_container_cow(after.shape.internal_levels()).0;
        if written > term_before {
            short += 1;
            println!(
                "R growth {label} append={k} grew={} written_blocks={written} term_with_levels_before={term_before} term_with_levels_after={term_after}",
                after.grew_a_level_this_publish()
            );
        }
        assert!(written <= term_after, "按发布之后的层数算总罩得住");
    }
    println!("R growth_summary {label} appends={appends} publishes_written_more_than_term_before={short}");
}

#[test]
fn opus_retire_r1_container_tree_growth_publish_against_the_term() {
    growth_rows("fanout2", capped(2), 40);
    growth_rows("fanout135", ContainerTreeNodeEntries::FromTheNodeFormat, 140);
}

#[test]
fn opus_retire_r1_w_of_a_two_disk_four_gib_pool() {
    let bytes = 4u64 << 30;
    let allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), bytes),
        DeviceFreeMap::new(DeviceIdentity(1), bytes),
    ]);
    let per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
    let roots_plus_one = ROOT_RING_REGIONS * per_region.count() + 1;
    for k in [1u64, 16, 144] {
        let window = ring_window_on_one_device(&allocator, per_region, k);
        let slots = window.0 / 16384;
        println!(
            "R w pool=2x4GiB batch_data_units={k} ring_window_slots={slots} roots_plus_one={roots_plus_one} w={}",
            slots / roots_plus_one
        );
    }
}
