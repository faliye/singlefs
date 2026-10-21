//! m3-close-c693-r3 辩方腿 T3 兼查（仓副本里编、跑）：`crates/` 的节点容量函数在记账 key 宽 7 / 14 / 19 / 21 / 22 字节（条目宽 = key 宽 + 12，
//! 22 字节对应 `of_the_node_format(22, 34)`，与 transaction.rs 的 477 / 150 同一调用）下的叶容量与内部容量，
//! 供第二轮正推 V3.2 ①④ 两条改后条文里「叶容量 491 掉到 477」「79 到 141 块盘上树高不同」两个数在 `crates/` 自己的函数上复算。
use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;

#[test]
fn leaf_and_internal_capacity_by_accounting_key_width() {
    for key_width in [7usize, 14, 19, 21, 22] {
        let capacity = CodeTwoTreeNodeCapacity::of_the_node_format(key_width, key_width + 12);
        eprintln!("R3CAPACITY key_width={key_width} entry_width={} leaf={} internal={}", key_width + 12, capacity.leaf_entries, capacity.internal_entries);
    }
}
