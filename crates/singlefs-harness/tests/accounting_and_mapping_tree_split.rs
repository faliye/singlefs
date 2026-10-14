//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 28 行的记账树与中央映射树那一半：两棵树装不下一个节点时照 D8（核心索引结构） 已定项 11
//! 分裂——同段照常 COW、叶从中间切、只摘空节点、根只剩一个孩子时降高、分隔 key 跟着维护、树高 = 根节点头里的层级 + 1——
//! 两棵树共用一份分裂代码（已定项 14，`singlefs_core::code_two_tree`）。内部条目 = 本树 key + 子指针 86（记账 108、映射 113）。
//!
//! 产品容量下（记账叶 477、映射叶 294）记账树要 80 块盘才分裂（`accounting_tree_node_full.rs`），
//! 中央映射树今天走得到的条目数装不满一个节点；这里用只供测试的开关把节点容量压小（`CodeTwoTreeNodeCapacities::CappedForTests`，
//! `.claude/rules/fs-design.md` 五条硬要求第 2 条），在两块盘的池上把两棵树长成多层，判：
//! ① 写出来的每个节点只信它自己头里写的——层级、key 区间（子树覆盖区间）、树 ID、出生序号按 bump 次序发——冷走读、挂载态、
//!    从盘上重建、池级 checker 都认它；
//! ② 一次发布只重写变了的那条路径：没变的叶照抄（字节、落点不动），被换下的节点正好释放掉，别的一个不动；
//! ③ 多层之后可写挂载照常：从盘上重建两棵树、写行与暖机按产品容量接着发；
//! ④ checker 逐节点核「它管的区间就是父条目给的那一段」、分隔 key 的两条不等式、层级逐层减一（I-1.1），改坏一处就红。
//! 分裂那一次发布的崩溃点在 `crash_enumeration_tree_split_streams.rs`。
mod common;
mod common_images_for_the_checker_tier;
mod common_tree_split;
#[path = "common_images_for_the_checker_tier/accounting_and_mapping_tree_split.rs"]
mod scenarios_of_accounting_and_mapping_tree_split;

use common::{disk_snapshot, parameters};
use common_images_for_the_checker_tier::PoolCheckerImages;
use common_tree_split::{
    capacities, propagate_the_new_checksum, shape_text,
    write_unit_to_the_same_slot_on_both_devices, TreeSplitPool,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::ByteReader;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::RecoveryFailure;
use singlefs_core::transaction::TransactionUnit;
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_accounting_and_mapping_tree_split::under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts(
) {
    scenarios_of_accounting_and_mapping_tree_split::under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_accounting_and_mapping_tree_split::an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes(
) {
    scenarios_of_accounting_and_mapping_tree_split::an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_accounting_and_mapping_tree_split::pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on() {
    scenarios_of_accounting_and_mapping_tree_split::pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 盘上中央映射树根的一条分隔 key 被抬到它那个孩子的最小 key 之上（藏住了那片叶的最小 key：按分隔 key 走会走到左边那片叶）。
/// 可写挂载从盘上重建这一版时照冷走读判全（代码审阅第 24 / 33 条，用户定案「可写挂载判全树头与 key 次序，坏了报错拒挂载，不多读盘」，
/// `code_two_tree::CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference`）：分隔 key 的两条不等式在重建那一步就判，
/// 挂载在读盘阶段报 `Recovery(InvariantViolated { invariant: "I-1.1", detail: "分隔 key 大于孩子头里的最小 key" })`，
/// 还没取号、没预演写行：两盘系统配置槽逐字节不变、根环没有新根、录制流一步都没多。
/// 规划那一步自己的拒绝（上一版的形状把一把 key 引到别的叶、删不掉，`CodeTwoTreeRefusal::PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt`）
/// 从盘上因此走不到了，由 `singlefs-core` 的单测 `code_two_tree::tests::a_previous_shape_whose_separator_hides_a_key_is_refused_by_the_planner` 钉。
/// 判别力：重建读中央映射树时不判节点头（`recovery.rs` 的 `rebuild_version` 那一处改回 `OnlyWhatTheShapeNeeds`），挂载改由规划那一步拒、
/// 报的成员不再是 I-1.1——`crates/mutations.tsv` 里以「实审 A3d：重建读中央映射树不判节点头（分隔 key」开头的那一行。
#[test]
fn a_central_mapping_root_whose_separator_hides_a_key_is_refused_by_the_rebuild_before_the_instance_generation_is_acquired(
) {
    let pool = TreeSplitPool::with_the_first_file_version_under(capacities(
        common_tree_split::ACCOUNTING_OF_THE_NODE_FORMAT,
        (5, 3),
    ));
    let output = &pool.output;
    // 十条映射条目（分配记录树按位置寻址，D8（核心索引结构） 已定项 14：五个节点各一条）按叶容量 5 长成三片叶：
    // [数据单元、extent 根、inode 根]、[分配记录树的前三个节点]、[分配记录树的后两个节点、记账树根、inode 叶]。
    assert_eq!(shape_text(&output.central_mapping_tree), "L3 L3 L4 I3");
    let root_unit = output.unit(TransactionUnit::MappingTree);
    let root = parse_index_node(&root_unit.bytes).expect("映射树根解得开");
    let mapping_key_width = root.key_width;
    let middle_leaf_pointer =
        NodePointer::read_from(&mut ByteReader::at(&root.entries[1], mapping_key_width));
    let middle_leaf = output
        .units
        .iter()
        .find(|unit| unit.slot == middle_leaf_pointer.locations[0].slot)
        .expect("中间那片叶是这一版的单元");
    let middle_leaf_entries = parse_index_node(&middle_leaf.bytes)
        .expect("中间那片叶解得开")
        .entries;
    let (hidden_key, second_key) = (
        &middle_leaf_entries[0][..mapping_key_width],
        &middle_leaf_entries[1][..mapping_key_width],
    );
    let hidden_role = output
        .mapped_units
        .iter()
        .find(|(_, key)| key.as_slice() == hidden_key)
        .expect("中间那片叶的最小 key 是这一版一个进映射的单元的")
        .0;
    assert!(
        matches!(
            hidden_role,
            TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        ),
        "被藏起来的那把是分配记录树一个节点的 key（写行那次发布换下它、要删它）：{hidden_role:?}"
    );
    // 第 1 条分隔 key 原是中间那片叶的最小 key，改成它的第二把：最小那把落到它之下、按分隔 key 走到左边那片叶。
    let mut entries = root.entries.clone();
    entries[1][..mapping_key_width].copy_from_slice(second_key);
    let damaged_root = build_index_node(
        root.tree,
        root.level,
        root.key_width,
        &root.smallest_key,
        &root.largest_key,
        root.birth_txg,
        &parameters().filesystem_identifier,
        root.instance,
        root.birth_sequence,
        u16::try_from(root.entry_width).expect("条目宽"),
        &entries,
    );
    let mut image = pool.memory_pool();
    write_unit_to_the_same_slot_on_both_devices(&mut image, root_unit.slot.0, &damaged_root);
    let units: Vec<(u64, usize)> = output
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.bytes.len()))
        .collect();
    propagate_the_new_checksum(
        &mut image,
        &units,
        (
            root_unit.slot.0,
            crc32_castagnoli(&root_unit.bytes),
            crc32_castagnoli(&damaged_root),
        ),
    );
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let before = disk_snapshot(&image, &stream);
    let refused = mount_writable(&parameters(), &mut devices);
    assert!(
        matches!(
            refused,
            Err(MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-1.1",
                detail: "分隔 key 大于孩子头里的最小 key",
            }))
        ),
        "重建判全那一道在取号之前就拒：{:?}",
        refused.err()
    );
    let after_image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: common::IMAGE_BYTES,
    };
    assert_eq!(
        disk_snapshot(&after_image, &stream),
        before,
        "两盘系统配置槽逐字节不变、根环没有新根、录制流一步都没多"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_accounting_and_mapping_tree_split::the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root() {
    scenarios_of_accounting_and_mapping_tree_split::the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root(&mut PoolCheckerImages::left_to_the_checker_tier());
}
