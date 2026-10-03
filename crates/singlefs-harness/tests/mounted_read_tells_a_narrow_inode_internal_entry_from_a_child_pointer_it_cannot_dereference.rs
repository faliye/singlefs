//! 只读挂载打开池时 inode 树内部条目的两种拒法分成两个错误成员（`records/2026-09-30-里程碑三开工第一轮.md` 第 8 批 core 簇第 8 条）：
//! 条目宽窄于字段表报 `OpenPoolForReadFailure::RecordMalformed`（这一版的读者不认这一版写的条目布局），子指针头部有一段不是第一版的取值报
//! `OpenPoolForReadFailure::PointerInTheRecordCannotBeDereferenced`（条目布局认得、指针指不过去，判该指针所在的结构损坏：
//! D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据））。改之前两种都报 `RecordMalformed`。
//!
//! 坏镜像只改 inode 树根那一个节点：在新池新建文件那一版上重写它（条目宽、或第 0 条条目子指针的 MAC），重封、沿引用链补校验和。

mod common;
mod common_pointer_head_corruption;

use common::{build_pool, parameters};
use common_pointer_head_corruption::{
    change_a_pointer_head_field_inside_the_unit, first_entry_of_the_node_at, node_bytes,
    node_pointer_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of,
    tree_table_entry_of_kind, unit_bytes_at, PointerHeadFieldChange,
};
use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure, OpenPoolForReadFailure};
use singlefs_core::pointer::PointerHeadFieldOutsideTheFirstVersion;
use singlefs_core::records::{parse_inode_internal_entry, TREE_KIND_INODE};
use singlefs_core::unit::{build_index_node, parse_index_node};

/// 场景：新池新建文件那一版，inode 树根按条目宽 119 重写（每条条目截掉最后一个字节：比字段表的 120 窄一个字节）。
/// 预期：只读挂载报 `RecordMalformed { what: "inode 树内部条目" }`。
#[test]
fn a_narrow_inode_internal_entry_is_reported_as_a_malformed_record_by_the_read_only_mount() {
    let mut image = build_pool("mounted-read-narrow-inode-internal-entry").memory_pool();
    assert!(mount_read_only(&image).is_ok(), "改之前只读挂载挂得上");
    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    let node = parse_index_node(&unit_bytes_at(
        &image,
        the_slot_of(&inode_root),
        node_bytes(),
    ))
    .expect("inode 树根解得开");
    let narrower_entries: Vec<Vec<u8>> = node
        .entries
        .iter()
        .map(|entry| entry[..entry.len() - 1].to_vec())
        .collect();
    let narrower_root = build_index_node(
        node.tree,
        node.level,
        node.key_width,
        &node.smallest_key,
        &node.largest_key,
        node.birth_txg,
        &parameters().filesystem_identifier,
        node.instance,
        node.birth_sequence,
        u16::try_from(node.entry_width - 1).expect("条目宽装得进两字节"),
        &narrower_entries,
    );
    replace_the_unit_and_propagate_its_checksum(
        &mut image,
        the_slot_of(&inode_root),
        &narrower_root,
    );
    let Err(refusal) = mount_read_only(&image) else {
        panic!("条目宽窄于字段表，只读挂载却挂上了");
    };
    assert!(
        matches!(
            refusal,
            MountReadOnlyFailure::Open(OpenPoolForReadFailure::RecordMalformed {
                what: "inode 树内部条目"
            })
        ),
        "只读挂载：条目布局不认，报 RecordMalformed，实际 {refusal:?}"
    );
}

/// 场景：新池新建文件那一版，inode 树根第 0 条内部条目的子指针 MAC 第 0 字节改成 1。
/// 预期：只读挂载报 `PointerInTheRecordCannotBeDereferenced { what: "inode 树内部条目的子指针", field: MacOrNonceNotZero }`。
#[test]
fn an_inode_internal_entry_whose_child_pointer_cannot_be_dereferenced_is_reported_apart_by_the_read_only_mount(
) {
    let mut image = build_pool("mounted-read-inode-internal-entry-child-pointer").memory_pool();
    assert!(mount_read_only(&image).is_ok(), "改之前只读挂载挂得上");
    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    let (_separator_key, _identity, child) =
        parse_inode_internal_entry(&first_entry_of_the_node_at(&image, &inode_root))
            .expect("改之前 inode 内部条目解得开");
    change_a_pointer_head_field_inside_the_unit(
        &mut image,
        the_slot_of(&inode_root),
        node_bytes(),
        &node_pointer_bytes(&child),
        PointerHeadFieldChange::FirstMacByteToOne,
    );
    let Err(refusal) = mount_read_only(&image) else {
        panic!("子指针 MAC 非 0，只读挂载却挂上了");
    };
    assert!(
        matches!(
            refusal,
            MountReadOnlyFailure::Open(
                OpenPoolForReadFailure::PointerInTheRecordCannotBeDereferenced {
                    what: "inode 树内部条目的子指针",
                    field: PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero,
                }
            )
        ),
        "只读挂载：条目布局认得、指针指不过去，实际 {refusal:?}"
    );
}
