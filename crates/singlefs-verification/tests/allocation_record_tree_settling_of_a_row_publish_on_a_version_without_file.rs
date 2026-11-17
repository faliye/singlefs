//! 树表 0 条那一版写行时分配记录树重写集的固定点（`transaction::settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`）：两处并进重写集都顺着释放链走到闭包，释放链不占轮数。
//! 只做 mkfs、反复可写挂载不写文件的盘上走得到这段历史；反复挂载要一万多次才把实例表长到 64 片，这里一次挂载之后接着发写行，表从头一次起就是 64 片（实例表的行编的，只求占槽），
//! 每次发布占的槽跨得过几片叶，根环转一圈之后回收出来的段又把落点带回冷的叶。盘在进程里造，不读活树；轮数读数住写入口上，这几次写行经 `PoolWriter` 直接发，发法与挂载里写行那一次同一个函数。

use std::collections::BTreeSet;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocation_record_tree::{
    nodes_whose_contents_changed, AllocationRecordTreeGeometry, AllocationRecordTreeNode,
    AllocationRecordTreeNodePosition, AllocationRecordTreeVersion,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::instance_table_page_pointers_as_far_as_readable;
use singlefs_core::transaction::{
    publish_instance_table_on_version_without_file, InstanceTableOnlyPublishPlan,
    InstanceTableRewrite, PoolVersion, PoolWriter,
};
use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::scenario::e142_parameters;

const DEVICE_BYTES: u64 = 4 << 30;
const ROWS_PER_INSTANCE_TABLE_PAGE: u64 = 369;
/// 每次写行占这么多片加几个分配记录树节点的槽：根环 24 槽转一圈钉着的落点比两片叶（每片 812 槽）还宽。
const INSTANCE_TABLE_PAGES: u64 = 64;
/// 够根环转几圈、落点回到冷的叶几次。
const ROW_PUBLISHES: u64 = 120;
/// 编的行用的实例号从这里起：盘上没有这些实例的根，抛弃判定碰不到这些行。
const INSTANCE_OF_THE_ROWS_MADE_UP: u32 = 1_000_000;
/// 重写集一开头就罩住释放链与取落点时的轮数：头一轮重写集是空的，释放弄脏的叶罩不住，接着那一轮定下。
const ROUNDS_WITH_THE_CLOSURE_COVERING_THE_ALLOCATIONS: usize = 2;
/// 这一版的节点集合与上一版不同（回收把一片叶清空、或头一次落到一片新叶）时，「装着记录的节点等于这一版的节点」那一判多来回的一轮。
const ROUNDS_FOR_THE_NODE_SET_TO_SETTLE: usize = 1;

fn rows_filling_the_instance_table() -> Vec<InstanceRow> {
    (0..INSTANCE_TABLE_PAGES * ROWS_PER_INSTANCE_TABLE_PAGE)
        .map(|row_index| InstanceRow {
            instance: InstanceGeneration(
                INSTANCE_OF_THE_ROWS_MADE_UP + u32::try_from(row_index).expect("两万多行"),
            ),
            selected_root_txg: CheckpointTxg(0),
            applied_transaction_high_water: 0,
        })
        .collect()
}

/// 一个槽所在的叶连同它到根的每个祖先：这个槽的记录一变，这几个节点的内容都变。
fn leaf_of_the_slot_and_its_ancestors(
    geometry: &AllocationRecordTreeGeometry,
    device: DeviceIdentity,
    slot: SlotNumber,
) -> BTreeSet<AllocationRecordTreeNode> {
    let mut nodes = BTreeSet::new();
    let mut leaf_or_ancestor = Some(geometry.leaf_of_slot(device, slot));
    // 迭代上界是树高。
    while let Some(node) = leaf_or_ancestor {
        nodes.insert(node);
        leaf_or_ancestor = geometry.parent_of(node);
    }
    nodes
}

/// 重写集从 `dirty_nodes` 起顺着释放链走到闭包：重写集里的节点在上一版有两份旧落点，释放它们弄脏旧落点所在的叶与祖先。逐跳从头算，不抄产品代码那一叠待走的节点。
fn nodes_along_the_release_chain(
    dirty_nodes: &BTreeSet<AllocationRecordTreeNode>,
    previous_tree: &AllocationRecordTreeVersion,
    geometry: &AllocationRecordTreeGeometry,
) -> BTreeSet<AllocationRecordTreeNode> {
    let mut walked = dirty_nodes.clone();
    // 迭代上界是树的节点数：不多就停，多一轮至少多一个节点。
    loop {
        let mut grown = walked.clone();
        for node in &walked {
            if let Some(pointer) = previous_tree.pointer_of(*node) {
                for location in pointer.locations {
                    grown.extend(leaf_of_the_slot_and_its_ancestors(
                        geometry,
                        location.device,
                        location.slot,
                    ));
                }
            }
        }
        if grown.len() == walked.len() {
            return walked;
        }
        walked = grown;
    }
}

/// 判别力：`crates/mutations.tsv`「分配记录树固定点（写行那一份）」开头那两行。释放那一处不走闭包，释放链上一跳多一轮，红在「两轮定下」那一判；
/// 取落点那一处不走闭包，进了冷的叶、冷叶的旧落点又在第三片叶上时多一轮，红在轮数上界那一判。两条判别力各要历史走到一次，收尾两个计数钉住走到了。
// 一个场景：这几次写行接在同一段历史上按次序做，后一次的旧落点是前一次取的，轮数本身就是被量的次序效应。
#[test]
fn row_publishes_on_a_version_without_file_take_no_round_for_the_release_chain_and_one_round_for_each_leaf_entered(
) {
    let parameters = e142_parameters(512, 512);
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let PoolVersion::WithoutFile(mut current_version) = current else {
        panic!("只做过 mkfs，现行那一版是树表 0 条的一版");
    };
    let rows = rows_filling_the_instance_table();
    let geometry = AllocationRecordTreeGeometry::of_allocator(&allocator);
    let one_device = devices[0].0;
    let device_identities: Vec<DeviceIdentity> =
        devices.iter().map(|(identity, _)| *identity).collect();
    let mut publishes_with_a_release_chain_and_nothing_else_to_settle = 0u64;
    let mut publishes_entering_one_leaf_with_a_chain_behind_it = 0u64;
    for publish_index in 0..ROW_PUBLISHES {
        let previous_tree = allocator
            .allocation_record_tree_of_the_version_without_file()
            .cloned();
        let replaced_chain =
            instance_table_page_pointers_as_far_as_readable(&devices, &current_version.root);
        // 写行之前已经和上一版那份记录不同的节点，加这次换下的实例表链弄脏的节点：产品代码头一轮并进重写集的就是它们。
        let dirty_before_the_walk = previous_tree.as_ref().map(|tree| {
            let mut dirty =
                nodes_whose_contents_changed(&geometry, &tree.records, allocator.records());
            for pointer in &replaced_chain {
                for location in pointer.locations {
                    dirty.extend(leaf_of_the_slot_and_its_ancestors(
                        &geometry,
                        location.device,
                        location.slot,
                    ));
                }
            }
            dirty
        });
        let rewrite = InstanceTableRewrite {
            rows: rows.clone(),
            replaced_chain,
        };
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let published = publish_instance_table_on_version_without_file(
            &mut writer,
            &mut allocator,
            &current_version.root,
            InstanceTableOnlyPublishPlan {
                txg: CheckpointTxg(current_version.root.checkpoint_txg.0 + 1),
                counter: current_version.record.counter + 1,
                instance: output.instance,
                back_chain: 0,
                rollback_floor: current_version.root.rollback_floor,
                instance_table: &rewrite,
                tree_identifier_watermark: current_version.root.tree_identifier_watermark,
            },
        )
        .expect("4 GiB 的盘离写满还远，写行做得成");
        let rounds = writer.rounds_of_the_last_allocation_record_tree_settling();
        if let (Some(tree), Some(dirty_nodes)) = (&previous_tree, &dirty_before_the_walk) {
            let rewritten_after_the_release_chain =
                nodes_along_the_release_chain(dirty_nodes, &tree.version, &geometry);
            let release_chain_walks_further =
                rewritten_after_the_release_chain.len() > dirty_nodes.len();
            let mut nodes_of_the_allocations: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();
            for unit in &published.units {
                for device in &device_identities {
                    nodes_of_the_allocations.extend(leaf_of_the_slot_and_its_ancestors(
                        &geometry, *device, unit.slot,
                    ));
                }
            }
            let leaves_entered: usize = nodes_of_the_allocations
                .difference(&rewritten_after_the_release_chain)
                .filter(|node| {
                    matches!(
                        node,
                        AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
                            level: 0,
                            device,
                            ..
                        }) if *device == one_device
                    )
                })
                .count();
            let node_set_stays = allocator
                .allocation_record_tree_of_the_version_without_file()
                .expect("刚写过行，分配器记着这一版的树")
                .version
                .node_set()
                == tree.version.node_set();
            println!(
                "SETTLING_OF_A_ROW_PUBLISH index={publish_index} rounds={rounds} leaves_entered={leaves_entered} release_chain_walks_further={release_chain_walks_further} node_set_stays={node_set_stays}"
            );
            if node_set_stays && leaves_entered == 0 {
                assert_eq!(
                    rounds, ROUNDS_WITH_THE_CLOSURE_COVERING_THE_ALLOCATIONS,
                    "第 {publish_index} 次写行：释放链走到闭包之后罩住了取落点碰到的节点，两轮定下"
                );
                if release_chain_walks_further {
                    publishes_with_a_release_chain_and_nothing_else_to_settle += 1;
                }
            } else {
                let rounds_of_the_node_set = if node_set_stays {
                    0
                } else {
                    ROUNDS_FOR_THE_NODE_SET_TO_SETTLE
                };
                let most_rounds = ROUNDS_WITH_THE_CLOSURE_COVERING_THE_ALLOCATIONS
                    + leaves_entered
                    + rounds_of_the_node_set;
                assert!(
                    rounds <= most_rounds,
                    "第 {publish_index} 次写行走了 {rounds} 轮：进了 {leaves_entered} 片叶，至多 {most_rounds} 轮"
                );
                let rewritten_with_the_allocations: BTreeSet<AllocationRecordTreeNode> =
                    rewritten_after_the_release_chain
                        .union(&nodes_of_the_allocations)
                        .copied()
                        .collect();
                if node_set_stays
                    && leaves_entered == 1
                    && nodes_along_the_release_chain(
                        &rewritten_with_the_allocations,
                        &tree.version,
                        &geometry,
                    )
                    .len()
                        > rewritten_with_the_allocations.len()
                {
                    publishes_entering_one_leaf_with_a_chain_behind_it += 1;
                }
            }
        }
        current_version = published;
    }
    println!(
        "SETTLING_OF_THE_ROW_PUBLISHES with_a_release_chain={publishes_with_a_release_chain_and_nothing_else_to_settle} entering_a_leaf_with_a_chain_behind_it={publishes_entering_one_leaf_with_a_chain_behind_it}"
    );
    assert!(
        publishes_with_a_release_chain_and_nothing_else_to_settle >= 1,
        "前提：历史走到过释放链多走一跳、取落点没进新叶的那一次写行"
    );
    assert!(
        publishes_entering_one_leaf_with_a_chain_behind_it >= 1,
        "前提：历史走到过取落点进一片叶、那片叶的旧落点又在重写集之外的那一次写行"
    );
}
