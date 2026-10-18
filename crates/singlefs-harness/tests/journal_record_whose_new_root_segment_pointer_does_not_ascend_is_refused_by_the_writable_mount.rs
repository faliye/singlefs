//! I-2.5（位置条目按设备身份升序）罩顶着所选那一版的那条 journal 记录新根段里的四条指针（代码轮
//! `research/prompts/m3-b44-code-r2-main-verification.md` 第二节 J1）：重建上一版一拿到那条记录就重新编码（`JournalRecord::to_bytes`），
//! 写者在那里断言升序；那一步之前按 I-2.5 判，坏的交 `Recovery(InvariantViolated { invariant: "I-2.5", .. })`，可写挂载在任何写之前拒、
//! 两块盘逐字节不变，只读挂载照旧挂上（它不重新编码那条记录）。
//! 坏镜像照攻方原型那一形（`research/prompts/m3-b44-code-r2-opus-model/attack_mount_rehearsal_probe.rs` 的
//! `swap_the_location_entries_of_the_tree_table_pointer_in_the_record_of_the_newest_root`）：新池新建文件之后，两块盘的 journal 环里
//! 与最新那条根同实例、同 txg、带本次发布末条标志的那条记录，新根段里一条指针的两条位置条目对调（设备 1 在前），重算记录头的宽校验和；
//! 根槽与单元一字不改。四条指针各一份。
//! 分配记录树根那一条：带文件的一版，根与末条记录新根段里这一项一律是空根（分配记录树挪进了树表，
//! `rebuilt_root_takes_every_new_root_segment_field_from_the_record.rs`），空根两条位置条目全零、对调了什么都没改；
//! 这一份先把记录里这一条换成一条不空、升序的指针（最新那条根的树表指针）重新编码，再对调它的两条位置条目。

mod common;
mod common_pointer_head_corruption;

use std::collections::BTreeSet;

use common::{build_pool, parameters};
use common_pointer_head_corruption::{
    newest_root, node_pointer_bytes, writable_mount_refused_before_any_write,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::{
    record_offset, JournalRecord, JournalRecordPlaceInPublish, JOURNAL_HEADER_CHECKSUM_OFFSET,
};
use singlefs_core::mount::MountError;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{PoolReader, RecoveryFailure};
use singlefs_core::root_record::RootRecord;
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{
    JOURNAL_HEADER_BYTES, JOURNAL_RECORD_BYTES, LOC_ENTRY, NODE_POINTER_BYTES, POINTER_HEAD_BYTES,
};
use singlefs_harness::memory_pool::MemoryPool;

/// 记录新根段里的哪一条指针（与根记录里同名的那一条字节相同：末条记录的新根段就是这次发布写的根上的那几条）。
#[derive(Clone, Copy, Debug)]
enum NewRootSegmentPointer {
    TreeTable,
    MappingRoot,
    InstanceTable,
    AllocationRecordTreeRoot,
}

fn pointer_of_the_root(root: &RootRecord, which: NewRootSegmentPointer) -> &NodePointer {
    match which {
        NewRootSegmentPointer::TreeTable => &root.tree_table,
        NewRootSegmentPointer::MappingRoot => &root.mapping_root,
        NewRootSegmentPointer::InstanceTable => &root.instance_table,
        NewRootSegmentPointer::AllocationRecordTreeRoot => &root.allocation_record_tree_root,
    }
}

fn pointer_of_the_record(record: &JournalRecord, which: NewRootSegmentPointer) -> &NodePointer {
    match which {
        NewRootSegmentPointer::TreeTable => &record.new_tree_table,
        NewRootSegmentPointer::MappingRoot => &record.new_mapping_root,
        NewRootSegmentPointer::InstanceTable => &record.new_instance_table,
        NewRootSegmentPointer::AllocationRecordTreeRoot => &record.new_allocation_record_tree_root,
    }
}

fn set_the_pointer_of_the_record(
    record: &mut JournalRecord,
    which: NewRootSegmentPointer,
    pointer: NodePointer,
) {
    match which {
        NewRootSegmentPointer::TreeTable => record.new_tree_table = pointer,
        NewRootSegmentPointer::MappingRoot => record.new_mapping_root = pointer,
        NewRootSegmentPointer::InstanceTable => record.new_instance_table = pointer,
        NewRootSegmentPointer::AllocationRecordTreeRoot => {
            record.new_allocation_record_tree_root = pointer;
        }
    }
}

/// 记录头里 `pointer` 那 86 字节出现在哪几个偏移上。
fn offsets_of_the_pointer_in_the_header(record_bytes: &[u8], pointer: &[u8]) -> BTreeSet<usize> {
    let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("487");
    (0..=header_bytes - pointer.len())
        .filter(|&start| record_bytes[start..start + pointer.len()] == *pointer)
        .collect()
}

/// 两块盘上与最新那条根同实例、同 txg、带本次发布末条标志的那条记录，新根段里 `which` 那一条的两条位置条目对调、重算记录头的宽校验和；
/// 交回改了几条记录（两块盘各一份）。
fn swap_the_location_entries_in_the_record_of_the_newest_root(
    image: &mut MemoryPool,
    which: NewRootSegmentPointer,
) -> usize {
    let parameters = parameters();
    let root = newest_root(image);
    let pointer = node_pointer_bytes(pointer_of_the_root(&root, which));
    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    let ring_bytes = parameters.geometry.journal_ring_bytes;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("487");
    let pointer_bytes = usize::try_from(NODE_POINTER_BYTES).expect("86");
    let location_entry_bytes = usize::try_from(LOC_ENTRY).expect("14");
    let pointer_head_bytes = usize::try_from(POINTER_HEAD_BYTES).expect("50");
    let mut changed = 0;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        // 迭代上界是环里的记录槽数；读不出一条自证过的记录就停（新池新建文件之后环里只有开头那几条）。
        for counter in 1..=ring_bytes / JOURNAL_RECORD_BYTES {
            let offset = record_offset(counter, ring_bytes);
            let bytes =
                PoolReader::read(&*image, device, offset, record_bytes).expect("环里读得到");
            let Some(record) = JournalRecord::parse(&bytes, filesystem_identifier) else {
                break;
            };
            if record.instance != root.instance
                || record.checkpoint_txg != root.checkpoint_txg
                || record.place_in_publish != JournalRecordPlaceInPublish::LastRecordOfThePublish
            {
                continue;
            }
            assert_eq!(
                node_pointer_bytes(pointer_of_the_record(&record, which)),
                pointer,
                "末条记录新根段里的 {which:?} 与根记录里的同一条字节相同"
            );
            let (bytes_to_swap_in, pointer_start) =
                if *pointer_of_the_root(&root, which) == NodePointer::empty_root() {
                    let mut planted = record.clone();
                    set_the_pointer_of_the_record(&mut planted, which, root.tree_table);
                    let planted_bytes = planted.to_bytes();
                    assert_eq!(
                        planted_bytes.len(),
                        record_bytes,
                        "重新编码的记录照样一条宽"
                    );
                    let tree_table_pointer = node_pointer_bytes(&root.tree_table);
                    let offsets_only_after_planting: Vec<usize> =
                        offsets_of_the_pointer_in_the_header(&planted_bytes, &tree_table_pointer)
                            .difference(&offsets_of_the_pointer_in_the_header(
                                &bytes,
                                &tree_table_pointer,
                            ))
                            .copied()
                            .collect();
                    assert_eq!(
                        offsets_only_after_planting.len(),
                        1,
                        "换进去的那一条在记录头里恰好多出一处"
                    );
                    (planted_bytes, offsets_only_after_planting[0])
                } else {
                    let pointer_start = (0..header_bytes - pointer_bytes)
                        .find(|&start| bytes[start..start + pointer_bytes] == pointer[..])
                        .expect("记录头里有这条指针的 86 字节");
                    (bytes.clone(), pointer_start)
                };
            let first_location = pointer_start + pointer_head_bytes;
            let second_location = first_location + location_entry_bytes;
            let mut changed_bytes = bytes_to_swap_in.clone();
            changed_bytes[first_location..second_location].copy_from_slice(
                &bytes_to_swap_in[second_location..second_location + location_entry_bytes],
            );
            changed_bytes[second_location..second_location + location_entry_bytes]
                .copy_from_slice(&bytes_to_swap_in[first_location..second_location]);
            let header_checksum = wide_checksum_with_field_zeroed(
                &changed_bytes,
                record_bytes,
                JOURNAL_HEADER_CHECKSUM_OFFSET,
            );
            changed_bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&header_checksum);
            let reparsed = JournalRecord::parse(&changed_bytes, filesystem_identifier)
                .expect("改过的记录头校验和自洽，照样解得开");
            let devices_in_order = pointer_of_the_record(&reparsed, which)
                .locations
                .map(|location| location.device);
            assert_eq!(
                devices_in_order,
                [DeviceIdentity(1), DeviceIdentity(0)],
                "两条位置条目对调之后设备 1 在前"
            );
            image
                .devices
                .get_mut(&device)
                .expect("池里有这块盘")
                .write(offset, &changed_bytes);
            changed += 1;
        }
    }
    changed
}

/// 改坏 `which` 那一条之后：可写挂载在任何写之前交 I-2.5 那一拒（两块盘逐字节不变由 `writable_mount_refused_before_any_write` 核），
/// 只读挂载照旧挂上。
fn writable_mount_refuses_and_read_only_mount_still_mounts(
    tag: &str,
    which: NewRootSegmentPointer,
) {
    let mut image = build_pool(tag).memory_pool();
    let changed = swap_the_location_entries_in_the_record_of_the_newest_root(&mut image, which);
    assert_eq!(changed, 2, "两块盘上各改到一份那条记录");
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-2.5",
                ..
            })
        ),
        "{which:?} 两条位置条目逆序的记录在重新编码之前按 I-2.5 拒：{refusal:?}"
    );
    assert!(
        mount_read_only(&image).is_ok(),
        "只读挂载不重新编码那条记录，照旧挂上"
    );
}

#[test]
fn a_record_whose_new_tree_table_pointer_does_not_ascend_is_refused_by_the_writable_mount_before_any_write(
) {
    writable_mount_refuses_and_read_only_mount_still_mounts(
        "record-new-tree-table-pointer-not-ascending",
        NewRootSegmentPointer::TreeTable,
    );
}

#[test]
fn a_record_whose_new_mapping_root_pointer_does_not_ascend_is_refused_by_the_writable_mount_before_any_write(
) {
    writable_mount_refuses_and_read_only_mount_still_mounts(
        "record-new-mapping-root-pointer-not-ascending",
        NewRootSegmentPointer::MappingRoot,
    );
}

#[test]
fn a_record_whose_new_instance_table_pointer_does_not_ascend_is_refused_by_the_writable_mount_before_any_write(
) {
    writable_mount_refuses_and_read_only_mount_still_mounts(
        "record-new-instance-table-pointer-not-ascending",
        NewRootSegmentPointer::InstanceTable,
    );
}

#[test]
fn a_record_whose_new_allocation_record_tree_root_pointer_does_not_ascend_is_refused_by_the_writable_mount_before_any_write(
) {
    writable_mount_refuses_and_read_only_mount_still_mounts(
        "record-new-allocation-record-tree-root-pointer-not-ascending",
        NewRootSegmentPointer::AllocationRecordTreeRoot,
    );
}
