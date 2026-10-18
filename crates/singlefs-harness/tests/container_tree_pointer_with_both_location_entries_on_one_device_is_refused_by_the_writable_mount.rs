//! I-2.5（位置条目按设备身份严格升序，`crates/singlefs-checker/invariants/04-校验和.md`）在三棵容器树（墓碑树、谱系树、退役树）上的读侧那一判
//! （代码轮 `research/prompts/m3-b44-code-r1-main-verification.md` 第二节 K4）：容器树根条目里第一条子指针（一层的树指向那片容器，
//! 两层的树指向根之下的码 2 节点），两条位置条目改成同一块盘（槽、校验和不动，两块盘同槽的两份内容相同，读得通），整单元校验和沿引用链
//! 补到根槽，读者核得过。可写挂载重建上一版时（`recovery::pointers_of_the_rebuilt_version_ascend_by_device`）按 I-2.5 拒、两块盘逐字节不变。
//! 挂载照收的话，下一次重写那棵树的节点时这条指针原样写回，在写者 `NodePointer::write_to` 的升序断言上 panic（正推腿探针
//! `research/prompts/m3-b44-code-r1-sonnet-model/probe/probe_equal_device_pointer_in_a_copied_container_tree_entry.rs` 实测 `mount=ok`、`publish=panic`）。

mod common;
mod common_deletion_in_progress;
mod common_images_for_the_checker_tier;
mod common_pointer_head_corruption;
#[path = "common_images_for_the_checker_tier/container_trees_grow_a_level_when_the_root_is_full.rs"]
mod scenarios_of_container_trees_grow_a_level_when_the_root_is_full;

use common::build_pool;
use common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, pool_with_new_inodes,
    publish_new_inodes_and_advance_the_pool, roll_back_to,
    root_slot_of_the_tree_of_kind_of_the_newest_root, unlink_to_the_end,
};
use common_pointer_head_corruption::{node_bytes, writable_mount_refused_before_any_write};
use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
    capacities_of_one_record_and_two_entries, unlink,
};
use singlefs_core::mount::MountError;
use singlefs_core::records::{TREE_KIND_LINEAGE, TREE_KIND_RETIREMENT, TREE_KIND_TOMBSTONE};
use singlefs_core::recovery::RecoveryFailure;
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_harness::memory_pool::MemoryPool;

/// 容器树根（码 2，key 宽 8）的条目区从 115 + 2 × 8 = 131 起（D18（块里携带什么信息） 已定项 18）；条目 120 字节，子指针在条目偏移 34
/// （分隔 key 8、身份引用 26 之后，`records::parse_inode_internal_entry_or_refusal`）。
const CONTAINER_TREE_ROOT_ENTRIES_START: usize = 131;
const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: usize = 34;
/// 一条指针里位置条目从偏移 50 起（头部 50 字节之后），每条 14 字节，设备身份是开头 4 字节（`pointer::LocationEntry::write_to`）。
const POINTER_LOCATION_ENTRIES_OFFSET: usize = 50;
const LOCATION_ENTRY_BYTES: usize = 14;
const LOCATION_ENTRY_DEVICE_BYTES: usize = 4;

/// 最新的根那一版种类 `kind` 的容器树根第一条条目的子指针（根是层级 1 时孩子是容器，层级 2 时孩子是根之下的码 2 节点；两种条目同一形态）：
/// 第二条位置条目的设备身份改成第一条的，别的不动；根重封、新的整单元校验和补进树表与根槽。
fn put_both_location_entries_of_the_first_child_pointer_of_the_root_on_one_device(
    image: &mut MemoryPool,
    kind: u16,
) {
    let root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(image, kind);
    change_the_unit_and_propagate_its_checksum(image, root_slot, node_bytes(), |root| {
        let first_location_device = CONTAINER_TREE_ROOT_ENTRIES_START
            + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
            + POINTER_LOCATION_ENTRIES_OFFSET;
        let second_location_device = first_location_device + LOCATION_ENTRY_BYTES;
        let first_device = root
            [first_location_device..first_location_device + LOCATION_ENTRY_DEVICE_BYTES]
            .to_vec();
        assert_ne!(
            first_device,
            root[second_location_device..second_location_device + LOCATION_ENTRY_DEVICE_BYTES],
            "前提：改坏之前那条子指针的两条位置条目在两块盘上"
        );
        root[second_location_device..second_location_device + LOCATION_ENTRY_DEVICE_BYTES]
            .copy_from_slice(&first_device);
    });
}

/// 可写挂载在任何写之前拒、报的是 I-2.5。
fn assert_the_writable_mount_refuses_it_by_the_location_entry_order(
    image: &MemoryPool,
    tree: &str,
) {
    let refusal = writable_mount_refused_before_any_write(image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-2.5",
                ..
            })
        ),
        "{tree}根条目里第一条子指针两条位置条目同盘：可写挂载按 I-2.5 拒：{refusal:?}"
    );
}

/// 场景：新池新建文件之后建两个空 inode（2、3 号），把 2 号 unlink 到底：墓碑树一片容器、一条墓碑。
/// 墓碑树根条目里那条容器指针两条位置条目改成同一块盘。
/// 预期：可写挂载按 I-2.5 拒，两块盘逐字节不变。
#[test]
fn a_tombstone_container_pointer_with_both_location_entries_on_one_device_is_refused_by_the_writable_mount(
) {
    let mut pool = pool_with_new_inodes("tombstone-container-pointer-on-one-device", 2);
    unlink_to_the_end(&mut pool, 2);
    assert_eq!(pool.output.tombstone_containers.len(), 1, "墓碑树一片容器");
    let mut image = pool.memory_pool();
    put_both_location_entries_of_the_first_child_pointer_of_the_root_on_one_device(
        &mut image,
        TREE_KIND_TOMBSTONE,
    );
    assert_the_writable_mount_refuses_it_by_the_location_entry_order(&image, "墓碑树");
}

/// 场景：新池新建文件之后建一个空 inode，再挂着回退到第一个文件那一版：谱系树一片容器、一条谱系记录。
/// 谱系树根条目里那条容器指针两条位置条目改成同一块盘。
/// 预期：可写挂载按 I-2.5 拒，两块盘逐字节不变。
#[test]
fn a_lineage_container_pointer_with_both_location_entries_on_one_device_is_refused_by_the_writable_mount(
) {
    let mut pool = build_pool("lineage-container-pointer-on-one-device");
    let first_file_version = pool.output.clone();
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    roll_back_to(&mut pool, &first_file_version);
    assert_eq!(pool.output.lineage_containers.len(), 1, "谱系树一片容器");
    let mut image = pool.memory_pool();
    put_both_location_entries_of_the_first_child_pointer_of_the_root_on_one_device(
        &mut image,
        TREE_KIND_LINEAGE,
    );
    assert_the_writable_mount_refuses_it_by_the_location_entry_order(&image, "谱系树");
}

/// 场景：新池新建文件之后把第一个文件 unlink 到底：最后一批删掉它的记录，inode 树唯一那片叶容器空了、身份消失，退役树一片容器、一条退役记录。
/// 退役树根条目里那条容器指针两条位置条目改成同一块盘。
/// 预期：可写挂载按 I-2.5 拒，两块盘逐字节不变。
#[test]
fn a_retirement_container_pointer_with_both_location_entries_on_one_device_is_refused_by_the_writable_mount(
) {
    let mut pool = build_pool("retirement-container-pointer-on-one-device");
    unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER);
    assert_eq!(pool.output.retirement_containers.len(), 1, "退役树一片容器");
    let mut image = pool.memory_pool();
    put_both_location_entries_of_the_first_child_pointer_of_the_root_on_one_device(
        &mut image,
        TREE_KIND_RETIREMENT,
    );
    assert_the_writable_mount_refuses_it_by_the_location_entry_order(&image, "退役树");
}

/// 场景：新池新建文件之后建五个空 inode（2–6 号），墓碑容器压到一片 1 条、一个码 2 节点 2 条，把 2、3、4 号各 unlink 到底：
/// 第 3 片墓碑容器让根满长一层，墓碑树两层（层级 2 的根下面两个码 2 节点，它们各指容器）。
/// 墓碑树根第一条条目里指向根之下那个码 2 节点的指针，两条位置条目改成同一块盘。
/// 预期：可写挂载按 I-2.5 拒，两块盘逐字节不变。
#[test]
fn a_tombstone_tree_pointer_below_the_root_with_both_location_entries_on_one_device_is_refused_by_the_writable_mount(
) {
    let mut pool = pool_with_new_inodes("tombstone-tree-pointer-below-the-root-on-one-device", 5);
    let capacities = capacities_of_one_record_and_two_entries();
    for inode in 2..=4 {
        unlink(&mut pool, inode, capacities);
    }
    assert_eq!(
        pool.output.tombstone_tree_index_nodes.shape.root_level(),
        2,
        "前提：墓碑树根满长了一层，根的孩子是码 2 节点"
    );
    assert!(
        !pool
            .output
            .tombstone_tree_index_nodes
            .pointers_below_the_root
            .is_empty(),
        "前提：墓碑树有根之下的节点指针"
    );
    let mut image = pool.memory_pool();
    put_both_location_entries_of_the_first_child_pointer_of_the_root_on_one_device(
        &mut image,
        TREE_KIND_TOMBSTONE,
    );
    assert_the_writable_mount_refuses_it_by_the_location_entry_order(&image, "墓碑树根之下");
}
