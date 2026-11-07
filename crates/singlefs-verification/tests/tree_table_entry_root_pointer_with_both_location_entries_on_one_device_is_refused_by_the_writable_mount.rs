//! I-2.5（位置条目按设备身份严格升序，`crates/singlefs-checker/invariants/04-校验和.md`）在树表条目根指针上的读侧那一判
//! （代码轮 `research/prompts/m4-close-code-r1-main-verification.md`「三、交给下一批的实现员（按文件聚簇）」E 簇）：
//! 树表条目的读者 `TreeTableEntry::parse` 不判位置条目次序，写者 `NodePointer::write_to` 断言严格升序。
//! 把盘上读来的树表条目照抄写回的，只有可写挂载接着重建出来的那一版发布；重建在交出那一版之前对每一条树表条目的根指针按 I-2.5 拒
//! （`recovery::pointers_of_the_rebuilt_version_ascend_by_device`，判法 `NodePointer::location_entries_ascend_by_device`）。
//! 墓碑树的根指针只住树表条目（根之下的指针另在 `tombstone_tree_index_nodes` 里判），它只经树表条目那一段判。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, pool_with_new_inodes,
    tree_table_slot_of_the_newest_root, unlink_to_the_end,
};
use common_pointer_head_corruption::{
    node_bytes, tree_table_entry_of_kind, writable_mount_refused_before_any_write,
};
use singlefs_core::mount::MountError;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{TreeTableEntry, TREE_KIND_TOMBSTONE};
use singlefs_core::recovery::RecoveryFailure;
use singlefs_core::unit::parse_index_node;
use singlefs_verification::memory_pool::MemoryPool;

/// 树表条目里根指针从偏移 14 起（树 ID 8、条目长度 2、树的种类 2、flags 2 之后，D8（核心索引结构） 已定项 8 的字段表）。
const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;
/// 一条指针里位置条目从偏移 50 起（头部 50 字节之后），每条 14 字节，设备身份是开头 4 字节（`pointer::LocationEntry::write_to`）。
const POINTER_LOCATION_ENTRIES_OFFSET: usize = 50;
const LOCATION_ENTRY_BYTES: usize = 14;
const LOCATION_ENTRY_DEVICE_BYTES: usize = 4;

/// 最新的根那一版树表里种类 `kind` 那一条条目的根指针：第二条位置条目的设备身份改成第一条的，别的不动；
/// 树表重封、新的整单元校验和补进根槽。
fn put_both_location_entries_of_the_tree_table_root_pointer_of_kind_on_one_device(
    image: &mut MemoryPool,
    kind: u16,
) {
    let tree_table_slot = tree_table_slot_of_the_newest_root(image);
    change_the_unit_and_propagate_its_checksum(
        image,
        tree_table_slot,
        node_bytes(),
        |tree_table| {
            let entry_bytes = parse_index_node(tree_table.as_slice())
                .expect("树表单元解得开")
                .entries
                .into_iter()
                .find(|entry| {
                    TreeTableEntry::parse(entry).is_some_and(|parsed| parsed.kind == kind)
                })
                .expect("树表里有这种树");
            let occurrences: Vec<usize> = tree_table
                .windows(entry_bytes.len())
                .enumerate()
                .filter(|(_, window)| *window == entry_bytes.as_slice())
                .map(|(start, _)| start)
                .collect();
            let [entry_start] = occurrences.as_slice() else {
                panic!("这条条目在树表单元里恰好出现一次：{occurrences:?}")
            };
            let first_location_device = entry_start
                + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
                + POINTER_LOCATION_ENTRIES_OFFSET;
            let second_location_device = first_location_device + LOCATION_ENTRY_BYTES;
            let first_device = tree_table
                [first_location_device..first_location_device + LOCATION_ENTRY_DEVICE_BYTES]
                .to_vec();
            tree_table
                [second_location_device..second_location_device + LOCATION_ENTRY_DEVICE_BYTES]
                .copy_from_slice(&first_device);
        },
    );
}

/// 场景：新池新建文件之后建两个空 inode（2、3 号），把 2 号 unlink 到底：墓碑树一片容器，根指针住树表里墓碑树那一条条目。
/// 那条根指针的两条位置条目改成同一块盘（槽、校验和不动，两块盘同槽的两份内容相同，读得通），树表重封、新的整单元校验和补到根槽。
/// 预期：读者 `TreeTableEntry::parse` 照收这条条目，而它的根指针写者不肯写（`location_entries_ascend_by_device` 报错）；
/// 可写挂载在任何写之前按 I-2.5 拒，两块盘逐字节不变。挂载照收的话，下一次发布照抄这条条目写新树表，在 `NodePointer::write_to` 的升序断言上 panic。
#[test]
fn a_tombstone_tree_root_pointer_in_the_tree_table_with_both_location_entries_on_one_device_is_refused_by_the_writable_mount(
) {
    let mut pool = pool_with_new_inodes(
        "tombstone-tree-root-pointer-in-the-tree-table-on-one-device",
        2,
    );
    unlink_to_the_end(&mut pool, 2);
    let mut image = pool.memory_pool();
    let root_before = tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).root;
    assert_ne!(
        root_before,
        NodePointer::empty_root(),
        "前提：墓碑树有根，根指针住树表条目"
    );
    assert_ne!(
        root_before.locations[0].device, root_before.locations[1].device,
        "前提：改坏之前那条根指针的两条位置条目在两块盘上"
    );
    put_both_location_entries_of_the_tree_table_root_pointer_of_kind_on_one_device(
        &mut image,
        TREE_KIND_TOMBSTONE,
    );
    // `tree_table_entry_of_kind` 经 `TreeTableEntry::parse` 取条目：读者照收改过的这一条。
    let root_after = tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).root;
    assert_eq!(
        root_after.locations[0].device, root_after.locations[1].device,
        "读者照收的这条根指针两条位置条目同盘"
    );
    assert!(
        root_after.location_entries_ascend_by_device().is_err(),
        "读者照收的这条根指针写者不肯写"
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-2.5",
                detail: "位置条目不按设备身份严格升序",
            })
        ),
        "树表里墓碑树根指针两条位置条目同盘：可写挂载按 I-2.5 拒：{refusal:?}"
    );
}
