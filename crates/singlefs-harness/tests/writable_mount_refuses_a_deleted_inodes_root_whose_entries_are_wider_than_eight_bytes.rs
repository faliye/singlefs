//! 外部审核 ②：重建上一版读 `deleted_inodes` 根时，条目宽不等于 8 就拒（D8（核心索引结构） 已定项 16：key = inode 号 8 字节、无 value）。
//! 坏镜像：三个单元的文件发完 unlink 意图那一版（队列 {1}），`deleted_inodes` 根的条目宽 8 → 16、声明长度 8 → 16，那一条在号之后补 8 字节，
//! 重封、新的整单元校验和沿引用链补到根槽（校验和自洽，`common_deletion_in_progress::change_the_unit_and_propagate_its_checksum`）。
//! 可写挂载在任何写之前拒、两块盘逐字节不变；条目宽只判「小于 8」时它被收下、号取那一条的前 8 字节。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, image_after_the_intent_of_a_three_unit_file,
};
use common_pointer_head_corruption::{
    node_bytes, the_slot_of, tree_table_entry_of_kind, writable_mount_refused_before_any_write,
};
use singlefs_core::mount::MountError;
use singlefs_core::records::TREE_KIND_DELETED_INODES;
use singlefs_core::recovery::RecoveryFailure;

/// `deleted_inodes` 根的 key 宽 8（D8（核心索引结构） 已定项 16）。
const DELETED_INODES_KEY_BYTES: usize = 8;
/// 码 2 节点头里声明长度（2 字节）在共同前缀偏移 8。
const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
/// 坏镜像把条目宽改成的宽：号 8 字节之后多 8 字节。
const WIDENED_ENTRY_BYTES: usize = 16;
/// 补在号之后的那 8 字节（不是 0，截前 8 字节读号时它们被丢掉）。
const BYTES_AFTER_THE_INODE_NUMBER: [u8; 8] = [0xA5; 8];

/// 码 2 节点头（key 宽 k）：条目数在 82 + 2k、条目宽在 84 + 2k、条目区从 115 + 2k 起（D18（块里携带什么信息） 已定项 7、18）。
fn index_node_entry_count_offset(key_width: usize) -> usize {
    82 + 2 * key_width
}

fn index_node_entry_width_offset(key_width: usize) -> usize {
    84 + 2 * key_width
}

fn index_node_entries_start(key_width: usize) -> usize {
    115 + 2 * key_width
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn set_u16_at(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn a_deleted_inodes_root_whose_entry_is_sixteen_bytes_wide_is_refused_by_the_writable_mount_before_any_write(
) {
    let mut image =
        image_after_the_intent_of_a_three_unit_file("deleted-inodes-root-entry-sixteen-bytes-wide");
    let deleted_inodes_root = tree_table_entry_of_kind(&image, TREE_KIND_DELETED_INODES).root;
    change_the_unit_and_propagate_its_checksum(
        &mut image,
        the_slot_of(&deleted_inodes_root),
        node_bytes(),
        |root| {
            let entries_start = index_node_entries_start(DELETED_INODES_KEY_BYTES);
            assert_eq!(
                u16_at(
                    root,
                    index_node_entry_count_offset(DELETED_INODES_KEY_BYTES)
                ),
                1,
                "意图之后队列里一个号"
            );
            assert_eq!(
                usize::from(u16_at(
                    root,
                    index_node_entry_width_offset(DELETED_INODES_KEY_BYTES)
                )),
                DELETED_INODES_KEY_BYTES,
                "改之前条目宽 8"
            );
            let widened = u16::try_from(WIDENED_ENTRY_BYTES).expect("16 装得进两字节");
            set_u16_at(
                root,
                index_node_entry_width_offset(DELETED_INODES_KEY_BYTES),
                widened,
            );
            set_u16_at(root, UNIT_DECLARED_LENGTH_OFFSET, widened);
            root[entries_start + DELETED_INODES_KEY_BYTES..entries_start + WIDENED_ENTRY_BYTES]
                .copy_from_slice(&BYTES_AFTER_THE_INODE_NUMBER);
        },
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::UnitMalformed {
                what: "deleted_inodes 根不是层级 0、条目宽 8 的非空节点"
            })
        ),
        "条目宽 16 的 deleted_inodes 根在重建上一版时报 UnitMalformed：{refusal:?}"
    );
}
