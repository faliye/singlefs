//! 读者对条目宽宽于字段表的节点也拒（I-1.10（码 2 条目宽等于字段表宽） 写「等于」）的两处：extent 上段叶条目 113
//! （`ExtentUpperLeafEntry::parse`；m3-close-w13-code-r1 判决第二节 K4 那一行「真漏」）与树表条目 200（`TreeTableEntry::parse`，
//! 本来就判「不等于 200」，补用例与变异）。别的几处在 `tree_readers_refuse_extent_inode_allocation_and_deadlist_entries_wider_than_the_field_table.rs`
//! 与 `recovery_refuses_entries_wider_than_the_field_table.rs`。
//!
//! 坏镜像只改一个节点：在发布路径写出的一版上把那个节点的每条条目末尾补一个 0、条目宽加一，按原来的头重建、重封，
//! 新的整单元校验和沿引用链补到根槽（`common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum`）。
//! 补的那个 0 落在条目自述的长度与预留区之外的末尾：树表条目里「条目长度」字段仍写 200、预留 76 仍全 0，只有条目宽这一处说话。
//! 判别力：`crates/mutations.tsv` 里以「根环一簇 R6」「根环一簇 R8」开头的两行（把拒的那一判改回只拒窄于），各让这里点名的用例红。

mod common;
mod common_pointer_head_corruption;

use common::parameters;
use common_pointer_head_corruption::{
    first_byte_of_the_first_file_read, image_after_a_sequential_write_of, newest_root, node_bytes,
    recovery_failure, replace_the_unit_and_propagate_its_checksum, the_slot_of,
    tree_table_entry_of_kind, unit_bytes_at,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::TREE_KIND_EXTENT;
use singlefs_core::recovery::RecoveryFailure;
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_harness::memory_pool::MemoryPool;

/// extent 上段叶条目字段表的宽：key 24 + 标签 1 + 载荷 88。
const EXTENT_UPPER_LEAF_ENTRY_FIELD_TABLE_BYTES: usize = 113;
/// 树表条目字段表的宽（D22（单元原子性怎么合成） 已定项 7「树表单元」）。
const TREE_TABLE_ENTRY_FIELD_TABLE_BYTES: usize = 200;
/// 文件有几个数据单元：两个，下段只有一片叶，上段根兼叶装一条标签 1 的条目。
const DATA_UNITS_OF_THE_FILE: usize = 2;

/// `pointer` 指的那个码 2 节点的每条条目末尾补一个 0、条目宽加一，按原来的头重建（重建即重封），写回两盘，
/// 新的整单元校验和沿引用链补到根槽。交回改之前的层级与条目宽。
fn widen_every_entry_of_the_node_by_one_byte(
    image: &mut MemoryPool,
    pointer: &NodePointer,
) -> (u8, usize) {
    let node = parse_index_node(&unit_bytes_at(image, the_slot_of(pointer), node_bytes()))
        .expect("要改的节点解得开");
    let wider_entries: Vec<Vec<u8>> = node
        .entries
        .iter()
        .map(|entry| {
            let mut wider = entry.clone();
            wider.push(0);
            wider
        })
        .collect();
    let wider_node = build_index_node(
        node.tree,
        node.level,
        node.key_width,
        &node.smallest_key,
        &node.largest_key,
        node.birth_txg,
        &parameters().filesystem_identifier,
        node.instance,
        node.birth_sequence,
        u16::try_from(node.entry_width + 1).expect("条目宽装得进两字节"),
        &wider_entries,
    );
    replace_the_unit_and_propagate_its_checksum(image, the_slot_of(pointer), &wider_node);
    (node.level, node.entry_width)
}

/// 场景：两个数据单元的文件，extent 树上段根兼叶（层级 0）的条目宽 113 → 114，每条末尾多一个 0，别处不动。
/// 预期：冷启动恢复报 `EntryWidthDiffersFromItsFieldTable { what: "extent 上段叶条目", entry_bytes: 114, field_table_bytes: 113 }`；
/// 改之前读得出第一个字节。改之前上段叶的读者只拒窄于 113，宽的照按固定偏移切、照读。
#[test]
fn an_extent_upper_leaf_whose_entries_are_wider_than_the_field_table_is_refused_by_recovery() {
    let mut image =
        image_after_a_sequential_write_of("wider-extent-upper-leaf", DATA_UNITS_OF_THE_FILE);
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "前提：改之前读得出第一个字节"
    );
    let upper_root = tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
    let level_and_width_before = widen_every_entry_of_the_node_by_one_byte(&mut image, &upper_root);
    assert_eq!(
        level_and_width_before,
        (0, EXTENT_UPPER_LEAF_ENTRY_FIELD_TABLE_BYTES),
        "前提：一个文件的上段根兼叶，写者写的条目宽等于字段表宽"
    );
    assert_eq!(
        recovery_failure(&image),
        RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
            what: "extent 上段叶条目",
            entry_bytes: EXTENT_UPPER_LEAF_ENTRY_FIELD_TABLE_BYTES + 1,
            field_table_bytes: EXTENT_UPPER_LEAF_ENTRY_FIELD_TABLE_BYTES,
        },
        "冷启动恢复：extent 上段叶条目宽于字段表，拒、不照读"
    );
}

/// 场景：两个数据单元的文件，最新根的树表单元条目宽 200 → 201，每条末尾多一个 0（条目自述的长度字段仍是 200），别处不动。
/// 预期：冷启动恢复报 `UnitMalformed { what: "树表条目" }`（`TreeTableEntry::parse` 判条目宽不等于 200）；改之前读得出第一个字节。
#[test]
fn a_tree_table_whose_entries_are_wider_than_the_field_table_is_refused_by_recovery() {
    let mut image = image_after_a_sequential_write_of("wider-tree-table", DATA_UNITS_OF_THE_FILE);
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "前提：改之前读得出第一个字节"
    );
    let tree_table = newest_root(&image).tree_table;
    let level_and_width_before = widen_every_entry_of_the_node_by_one_byte(&mut image, &tree_table);
    assert_eq!(
        level_and_width_before,
        (0, TREE_TABLE_ENTRY_FIELD_TABLE_BYTES),
        "前提：树表单元一层，写者写的条目宽等于字段表宽"
    );
    assert_eq!(
        recovery_failure(&image),
        RecoveryFailure::UnitMalformed {
            what: "树表条目"
        },
        "冷启动恢复：树表条目宽于字段表，拒、不照读"
    );
}
