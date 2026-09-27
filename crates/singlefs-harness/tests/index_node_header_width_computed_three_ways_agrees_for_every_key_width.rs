//! 码 2 索引节点含 key 区间与预留位的头宽 `86 + 2 × key 宽 + 29`，三方各算一份（代码审阅第 11 条；2026-09-27 用户定案「三方各算一份 + 交叉断言」，
//! 记在 `records/2026-09-27-代码审阅38条去向.md`）：core 的 `singlefs_core::unit::index_node_header_bytes`、checker 的
//! `singlefs_checker::index_node_header_bytes`、理想模型的 `singlefs_harness::model::index_node_header_bytes`，
//! `singlefs-format` 只给标量（D13（验证路线） 已定项 5：发射物里不许有算术）。
//!
//! 两条测试各管一半：
//! - 连起来比：三份在 key 宽的全部取值上逐个相等。哪一份自己的式子写歪了（少算一个 key、漏了预留位、只在某一段 key 宽上歪），这一条红。
//! - 钉绝对值：kb 里给每棵树登记过的头宽（`format-const` 标记，`.claude/kb/layout/01-first-txn.md`）逐个与三份在那棵树的 key 宽上比。
//!   三份共用的标量写错时三份一起错、连起来比看不出，要靠这一半。
//!
//! key 宽的取值域是 0..=255，上界从格式约束推：
//! ① key 宽字段在码 2 头偏移 51、宽 1 字节（D18（块里携带什么信息） 已定项 18 的偏移表），装得下的最大值是 `u8::MAX` = 255；
//! ② 节点装得下至少一个条目：条目以 key 打头、条目宽不小于 key 宽（core 与 checker 的解析都拒条目宽小于 key 宽），
//!    所以要 头宽 + key 宽 ≤ 16384，即 115 + 3 × key 宽 ≤ 16384、key 宽 ≤ 5423。②比①松，上界取①的 255；
//!    第一条测试在每个 key 宽上顺带核②，节点宽或头宽的标量改到②比①紧时它先红，取值域要跟着重推。
//! 下界 0：字段里写得下 0，core 与 checker 的解析器都照读、不拒，三份在 0 上同样要对得上。

use singlefs_format::{
    ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES, ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES,
    CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES, EXTENT_TREE_INDEX_NODE_HEADER_BYTES,
    INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES, NODE_BYTES, TREE_TABLE_UNIT_INDEX_NODE_HEADER_BYTES,
};

/// 三份在同一个 key 宽上算出来的头宽，都换成 u64 再比（core 与 checker 按 usize 交回，模型按 u64）。
struct HeaderBytesComputedThreeWays {
    by_core: u64,
    by_checker: u64,
    by_model: u64,
}

fn header_bytes_computed_three_ways(key_width_byte: u8) -> HeaderBytesComputedThreeWays {
    HeaderBytesComputedThreeWays {
        by_core: u64::try_from(singlefs_core::unit::index_node_header_bytes(usize::from(
            key_width_byte,
        )))
        .expect("头宽至多几百字节，装得进 u64"),
        by_checker: u64::try_from(singlefs_checker::index_node_header_bytes(key_width_byte))
            .expect("头宽至多几百字节，装得进 u64"),
        by_model: singlefs_harness::model::index_node_header_bytes(u64::from(key_width_byte)),
    }
}

#[test]
fn core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte() {
    let mut key_widths_checked: u32 = 0;
    for key_width_byte in u8::MIN..=u8::MAX {
        let computed = header_bytes_computed_three_ways(key_width_byte);
        assert_eq!(
            computed.by_core, computed.by_checker,
            "key 宽 {key_width_byte}：core 算的头宽与 checker 算的对不上"
        );
        assert_eq!(
            computed.by_checker, computed.by_model,
            "key 宽 {key_width_byte}：checker 算的头宽与理想模型算的对不上"
        );
        assert!(
            computed.by_core + u64::from(key_width_byte) <= NODE_BYTES,
            "key 宽 {key_width_byte}：头宽 {} 加一个只有 key 的条目就装不进 {NODE_BYTES} 字节的节点，取值域上界要从 255 往下重推",
            computed.by_core
        );
        key_widths_checked += 1;
    }
    assert_eq!(
        key_widths_checked, 256,
        "key 宽字段 1 字节，0..=255 一个不落"
    );
}

/// kb 给一棵树登记的码 2 头宽：`format-const` 标记的名字、那棵树的 key 宽、标记里写的值。
struct IndexNodeHeaderWidthRegisteredInTheKnowledgeBase {
    tree: &'static str,
    key_width_byte: u8,
    format_constant: u64,
    value_in_the_format_const_marker: u64,
}

#[test]
fn each_tree_registered_index_node_header_width_equals_all_three_formulas_at_that_tree_key_width() {
    let registered = [
        IndexNodeHeaderWidthRegisteredInTheKnowledgeBase {
            tree: "inode 树根（INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES）",
            key_width_byte: 8,
            format_constant: INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES,
            value_in_the_format_const_marker: 131,
        },
        IndexNodeHeaderWidthRegisteredInTheKnowledgeBase {
            tree: "树表单元（TREE_TABLE_UNIT_INDEX_NODE_HEADER_BYTES）",
            key_width_byte: 8,
            format_constant: TREE_TABLE_UNIT_INDEX_NODE_HEADER_BYTES,
            value_in_the_format_const_marker: 131,
        },
        IndexNodeHeaderWidthRegisteredInTheKnowledgeBase {
            tree: "分配记录树（ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES）",
            key_width_byte: 10,
            format_constant: ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES,
            value_in_the_format_const_marker: 135,
        },
        IndexNodeHeaderWidthRegisteredInTheKnowledgeBase {
            tree: "记账树（ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES）",
            key_width_byte: 22,
            format_constant: ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES,
            value_in_the_format_const_marker: 159,
        },
        IndexNodeHeaderWidthRegisteredInTheKnowledgeBase {
            tree: "extent 树（EXTENT_TREE_INDEX_NODE_HEADER_BYTES）",
            key_width_byte: 24,
            format_constant: EXTENT_TREE_INDEX_NODE_HEADER_BYTES,
            value_in_the_format_const_marker: 163,
        },
        IndexNodeHeaderWidthRegisteredInTheKnowledgeBase {
            tree: "中央映射树（CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES）",
            key_width_byte: 27,
            format_constant: CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES,
            value_in_the_format_const_marker: 169,
        },
    ];
    for registered_width in &registered {
        let tree = registered_width.tree;
        let key_width_byte = registered_width.key_width_byte;
        let expected = registered_width.value_in_the_format_const_marker;
        assert_eq!(
            registered_width.format_constant, expected,
            "{tree}：格式常量与 kb 标记里的值对不上"
        );
        let computed = header_bytes_computed_three_ways(key_width_byte);
        assert_eq!(
            computed.by_core, expected,
            "{tree}：core 在 key 宽 {key_width_byte} 上算的头宽不是登记的 {expected}"
        );
        assert_eq!(
            computed.by_checker, expected,
            "{tree}：checker 在 key 宽 {key_width_byte} 上算的头宽不是登记的 {expected}"
        );
        assert_eq!(
            computed.by_model, expected,
            "{tree}：理想模型在 key 宽 {key_width_byte} 上算的头宽不是登记的 {expected}"
        );
    }
}
