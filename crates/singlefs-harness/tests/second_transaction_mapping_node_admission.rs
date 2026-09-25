//! 写侧准入的中央映射树那一条（D19（块指针的结构与宽度预算） 已定项 5：写侧每次发布按映射树现在的条目数与这次发布最多新增的条目数
//! 做映射树容量准入，不靠别的树容量上的数字巧合）。映射树长成多层之后（D8（核心索引结构） 已定项 11），条目装不下一个节点时
//! 照分裂规则长出内部节点、不拒。分配记录树按位置寻址之后（D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，
//! 可写挂载取号之前那一串（写行 + 暖机）的预演走的就是发布路径落盘之前那一段（`transaction::prepare_the_version_publish` 等），
//! 不再另推每次重写几个角色；预演与真发取到的落点逐项相同由 `mount::establish_instance` 在那一串发完之后断言，每条可写挂载的用例都走到它。
//! 长到 257 层（码 2 头的层级是 1 字节，D18（块里携带什么信息） 已定项 18）是多层之后唯一的结构上限，走得到要 256 层，
//! 在规划那一步由 `code_two_tree` 的单测钉住。
//!
//! 本文件两条用例：数字钉死（叶 294 / 内部 143、第一个文件版本 6 条）；295 条的映射树长成两层而不是被拒（规划那一步，产品容量）。

mod common;
mod common_tree_split;

use std::collections::BTreeSet;

use common_tree_split::TreeSplitPool;
use singlefs_core::code_two_tree::{
    plan_the_tree_after_this_publish, CodeTwoKeyFieldWidths, CodeTwoTreeKey,
    CodeTwoTreeNodeContents, CodeTwoTreeShape,
};
use singlefs_core::transaction::{CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree};
use singlefs_format::MAPPING_KEY_BYTES;

/// 进映射而与 inode 叶容器、分配记录树都无关的角色数：数据单元、extent 根（一个单元的文件内联，extent 树只有上段根兼叶）、
/// inode 根、记账树（根兼叶时一个节点）。分配记录树每个节点各一条（D8（核心索引结构） 已定项 14：按位置寻址，节点数随池的形状走）。
/// 这里另写一遍，不从核心层取：条目数的算术要有一个独立的对照物。
const MAPPING_ROLES_OUTSIDE_THE_INODE_LEAF_CONTAINERS_AND_THE_ALLOCATION_RECORD_TREE: usize = 4;

#[test]
fn the_central_mapping_node_holds_two_hundred_ninety_four_entries_and_an_internal_node_one_hundred_forty_three(
) {
    let capacity = MultiLevelCodeTwoTree::CentralMapping.node_capacity_of_the_node_format();
    assert_eq!(
        (capacity.leaf_entries, capacity.internal_entries),
        (294, 143),
        "key 27、叶条目 55、内部条目 27 + 86 = 113 的码 2 节点（D8 已定项 11）"
    );
    let pool = TreeSplitPool::with_the_first_file_version_under(
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    assert_eq!(
        pool.output.mapping_keys.len(),
        MAPPING_ROLES_OUTSIDE_THE_INODE_LEAF_CONTAINERS_AND_THE_ALLOCATION_RECORD_TREE
            + pool.output.allocation_record_tree.nodes.len()
            + pool.output.inode_leaf_containers.len(),
        "条目数与真装进映射树的条数是同一个"
    );
    assert_eq!(
        (
            pool.output.mapping_keys.len(),
            pool.output.allocation_record_tree.nodes.len(),
            pool.output.inode_leaf_containers.len(),
            pool.output.central_mapping_tree.node_count()
        ),
        (10, 5, 1, 1),
        "第一个文件版本：分配记录树五个节点（4 GiB 两块盘上根在第 2 层，每块盘第 1 层一个、单元区起点那片叶一个），\
         一片叶容器，十条映射条目，映射树一个节点（根兼叶）"
    );
}

/// 按产品容量（叶 294 条）：295 把 key 的映射树从空长起来，在第 295 把时根兼叶从中间切，长成两层（148 + 147 条两片叶），
/// 不是被拒——改之前这一格是 `MappingEntriesExceedOneNode`。只走规划那一步（纯函数，不碰盘）：今天发布路径走得到的映射条目数
/// 装不满一个节点（inode 树根一个节点装 135 片叶，5 + 135 < 294），要真发布出多层映射得压小容量（本文件第三条与分裂的那几份用例）。
#[test]
fn two_hundred_ninety_five_mapping_keys_grow_the_tree_to_two_levels_instead_of_being_refused() {
    let key_width = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let mapping_keys: BTreeSet<CodeTwoTreeKey> = (0..295u64)
        .map(|value| {
            let mut key = vec![0u8; key_width];
            key[0] = 1;
            key[9..17].copy_from_slice(&value.to_le_bytes());
            CodeTwoTreeKey::new(&key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING)
        })
        .collect();
    let plan = plan_the_tree_after_this_publish(
        &CodeTwoTreeShape::default(),
        &mapping_keys,
        MultiLevelCodeTwoTree::CentralMapping.node_capacity_of_the_node_format(),
    )
    .expect("两层装得下");
    assert_eq!(
        plan.shape.height(),
        2,
        "根兼叶装不下第 295 条：从中间切、长出新根"
    );
    assert_eq!(plan.shape.nodes().len(), 3, "两片叶加一个根");
    let leaf_sizes: Vec<usize> = plan
        .shape
        .nodes()
        .iter()
        .filter(|node| node.position.level == 0)
        .map(|node| match &node.contents {
            CodeTwoTreeNodeContents::Leaf { keys } => keys.len(),
            CodeTwoTreeNodeContents::Internal { .. } => {
                unreachable!("层级 0 是叶")
            }
        })
        .collect();
    assert_eq!(
        leaf_sizes,
        vec![148, 147],
        "295 条从中间切：左 ⌈295 ÷ 2⌉ = 148"
    );
}
