//! 读树的 core 读者对条目宽宽于字段表的节点也拒（欠账 C867；I-1.10（码 2 条目宽等于字段表宽） 写「等于」，用户 2026-10-18 弹窗定
//! 「宽于字段表也拒」）：extent 树内部条目 110、extent 叶记录 112、inode 树内部条目 120、分配记录 20、分配记录树内部条目 96、deadlist 条目 43，
//! 一处一条用例。记账条目与码 2 多层树内部条目那三处是 C780 做的（`recovery_refuses_entries_wider_than_the_field_table.rs`）；
//! 树表条目 200 的读者（`TreeTableEntry::parse`）本来就判「不等于 200」，不在这里；映射树叶条目宽于 55 欠在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）。
//!
//! 坏镜像只改一个节点：在发布路径写出的一版上把那个节点的每条条目末尾补一个 0、条目宽加一，按原来的头重建、重封，
//! 新的整单元校验和沿引用链补到根槽（`common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum`），镜像上只剩条目宽这一处说话。
//! 每条用例先在改之前的同一版上核：读者照常读出（或可写挂载照常挂上）。改之前这几处只拒窄于字段表，宽于的照读。
//! 判别力：`crates/mutations.tsv` 里以「C867」开头的六行（把拒的那一判改回只拒窄于），各让这里点名的用例红。

mod common;
mod common_pointer_head_corruption;

use common::{
    build_pool, parameters, publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
};
use common_pointer_head_corruption::{
    first_byte_of_the_first_file_read, first_entry_of_the_node_at, first_upper_leaf_entry,
    image_after_a_sequential_write_of, node_bytes, open_the_first_file_refused, recovery_failure,
    replace_the_unit_and_propagate_its_checksum, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at, writable_devices_of, writable_mount_refused_before_any_write,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::bytes::ByteReader;
use singlefs_core::extent_tree::ExtentUpperLeafTarget;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::mounted_read::{
    mount_read_only, MountReadOnlyFailure, OpenFileFailure, OpenPoolForReadFailure,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_INODE};
use singlefs_core::recovery::RecoveryFailure;
use singlefs_core::transaction::{publish_create_snapshot, PoolWriter};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_harness::memory_pool::MemoryPool;

/// extent 树内部条目字段表的宽：extent key 24 + 子指针 86（D8（核心索引结构） 已定项 11）。
const EXTENT_TREE_INTERNAL_ENTRY_FIELD_TABLE_BYTES: usize = 110;
/// extent 叶记录字段表的宽：extent key 24 + 数据指针 88。
const EXTENT_LEAF_RECORD_FIELD_TABLE_BYTES: usize = 112;
/// inode 树内部条目字段表的宽：分隔 key 8 + 身份引用 26 + 子指针 86。
const INODE_INTERNAL_ENTRY_FIELD_TABLE_BYTES: usize = 120;
/// 分配记录字段表的宽：设备 4 + 槽号 6 + 跨度与释放位 2 + 代 8。
const ALLOCATION_RECORD_FIELD_TABLE_BYTES: usize = 20;
/// 分配记录树内部条目字段表的宽：分配记录 key 10 + 子指针 86。
const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_FIELD_TABLE_BYTES: usize = 96;
/// 分配记录 key 的宽（设备 4 + 槽号 6）：内部条目里子指针从这里起。
const ALLOCATION_RECORD_KEY_BYTES: usize = 10;
/// deadlist 条目字段表的宽：头的树 ID 8 + 纪元 8 + 中央映射 key 27，value 空（D5（快照 / 空间记账机制） 已定项 12）。
const DEADLIST_ENTRY_FIELD_TABLE_BYTES: usize = 43;
/// 下段两层的文件有几个数据单元：比一片下段叶装得下的 144 个多一个（D8（核心索引结构） 已定项 14「实现取值」）。
const DATA_UNITS_OF_A_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT: usize = 145;
/// 下段只有一片叶（根兼叶）的文件有几个数据单元。
const DATA_UNITS_OF_A_FILE_WITH_A_ONE_LEAF_LOWER_SEGMENT: usize = 2;
/// 建快照与快照下覆盖写都在实例 1 上（`build_pool` 取的号）。
const INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE: InstanceGeneration = InstanceGeneration(1);

/// `pointer` 指的那个码 2 节点的每条条目末尾补一个 0、条目宽加一，按原来的头重建（重建即重封），写回两盘，
/// 新的整单元校验和沿引用链补到根槽。
fn widen_every_entry_of_the_node_by_one_byte(image: &mut MemoryPool, pointer: &NodePointer) {
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
}

/// 码 2 节点的层级与条目宽。
fn level_and_entry_width_of_the_node_at(image: &MemoryPool, pointer: &NodePointer) -> (u8, usize) {
    let node = parse_index_node(&unit_bytes_at(image, the_slot_of(pointer), node_bytes()))
        .expect("节点解得开");
    (node.level, node.entry_width)
}

/// 改之前可写挂载照常挂上（挂在镜像的一份拷贝上，镜像本身不动）。
fn writable_mount_accepts(image: &MemoryPool) -> bool {
    mount_writable(&parameters(), &mut writable_devices_of(image)).is_ok()
}

/// 分配记录树从根沿每层第 0 条内部条目往下，走到的那片叶。
fn allocation_record_tree_leaf_down_the_first_entries(image: &MemoryPool) -> NodePointer {
    let mut pointer = tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION).root;
    let (mut level, _entry_width) = level_and_entry_width_of_the_node_at(image, &pointer);
    // 迭代上界是根的层级（池几何定的那一层）：每一轮往下一层，层级严格变小，到 0 停。
    while level > 0 {
        pointer = NodePointer::read_from(&mut ByteReader::at(
            &first_entry_of_the_node_at(image, &pointer),
            ALLOCATION_RECORD_KEY_BYTES,
        ));
        let (child_level, _child_entry_width) =
            level_and_entry_width_of_the_node_at(image, &pointer);
        assert_eq!(child_level + 1, level, "孩子的层级是父层级减一");
        level = child_level;
    }
    pointer
}

/// 新池新建文件之后建一个快照、再在快照下覆盖写一次：换下的数据单元、extent 根、inode 叶容器与根都生在快照之前，进 deadlist。
fn image_with_a_deadlist_after_an_overwrite_under_a_snapshot() -> MemoryPool {
    let mut pool: BuiltPool = build_pool("wider-than-the-field-table-deadlist");
    let publish_parameters = parameters();
    let before_the_snapshot = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_create_snapshot(
        &mut writer,
        &mut pool.allocator,
        &before_the_snapshot,
        INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE,
    )
    .expect("建快照");
    let before_the_overwrite = pool.output.clone();
    let overwrite_content: Vec<u8> = (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + 11) % 239).expect("小于 256"))
        .collect();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &before_the_overwrite,
        &overwrite_content,
        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
        INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE,
    )
    .expect("快照下覆盖写");
    pool.memory_pool()
}

/// 场景：145 个数据单元的文件，下段根在第 1 层；下段根的内部条目宽 110 → 111，每条末尾多一个 0，别处不动。
/// 预期：冷启动恢复报 `EntryWidthDiffersFromItsFieldTable { what: "extent 树内部条目", entry_bytes: 111, field_table_bytes: 110 }`，
/// 打开文件报同一个成员（`ExtentTreeWalk`）；改之前读得出第一个字节。
#[test]
fn an_extent_tree_internal_node_whose_entries_are_wider_than_the_field_table_is_refused_by_recovery_and_by_opening_the_file(
) {
    let mut image = image_after_a_sequential_write_of(
        "wider-than-the-field-table-extent-internal-node",
        DATA_UNITS_OF_A_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT,
    );
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "前提：改之前读得出第一个字节"
    );
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&image).target
    else {
        panic!("145 个数据单元的文件在上段叶里是一条标签 1 的条目");
    };
    assert_eq!(
        level_and_entry_width_of_the_node_at(&image, &lower_root),
        (1, EXTENT_TREE_INTERNAL_ENTRY_FIELD_TABLE_BYTES),
        "前提：145 个单元的下段根在第 1 层，写者写的内部条目宽等于字段表宽"
    );
    widen_every_entry_of_the_node_by_one_byte(&mut image, &lower_root);
    let wider_internal_entries_refused = RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
        what: "extent 树内部条目",
        entry_bytes: EXTENT_TREE_INTERNAL_ENTRY_FIELD_TABLE_BYTES + 1,
        field_table_bytes: EXTENT_TREE_INTERNAL_ENTRY_FIELD_TABLE_BYTES,
    };
    assert_eq!(
        recovery_failure(&image),
        wider_internal_entries_refused,
        "冷启动恢复：extent 树内部条目宽于字段表，拒、不照读"
    );
    assert_eq!(
        open_the_first_file_refused(&image),
        OpenFileFailure::ExtentTreeWalk(wider_internal_entries_refused),
        "打开文件：同一个成员"
    );
}

/// 场景：两个数据单元的文件，下段只有一片叶（根兼叶）；它的叶记录宽 112 → 113，每条末尾多一个 0，别处不动。
/// 预期：冷启动恢复报 `EntryWidthDiffersFromItsFieldTable { what: "extent 叶记录", entry_bytes: 113, field_table_bytes: 112 }`，
/// 打开文件报同一个成员；改之前读得出第一个字节。
#[test]
fn an_extent_leaf_whose_records_are_wider_than_the_field_table_is_refused_by_recovery_and_by_opening_the_file(
) {
    let mut image = image_after_a_sequential_write_of(
        "wider-than-the-field-table-extent-leaf",
        DATA_UNITS_OF_A_FILE_WITH_A_ONE_LEAF_LOWER_SEGMENT,
    );
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "前提：改之前读得出第一个字节"
    );
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&image).target
    else {
        panic!("两个数据单元的文件在上段叶里是一条标签 1 的条目");
    };
    assert_eq!(
        level_and_entry_width_of_the_node_at(&image, &lower_root),
        (0, EXTENT_LEAF_RECORD_FIELD_TABLE_BYTES),
        "前提：两个单元的下段根兼叶，写者写的叶记录宽等于字段表宽"
    );
    widen_every_entry_of_the_node_by_one_byte(&mut image, &lower_root);
    let wider_leaf_records_refused = RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
        what: "extent 叶记录",
        entry_bytes: EXTENT_LEAF_RECORD_FIELD_TABLE_BYTES + 1,
        field_table_bytes: EXTENT_LEAF_RECORD_FIELD_TABLE_BYTES,
    };
    assert_eq!(
        recovery_failure(&image),
        wider_leaf_records_refused,
        "冷启动恢复：extent 叶记录宽于字段表，拒、不照读"
    );
    assert_eq!(
        open_the_first_file_refused(&image),
        OpenFileFailure::ExtentTreeWalk(wider_leaf_records_refused),
        "打开文件：同一个成员"
    );
}

/// 场景：新池新建文件那一版，inode 树根（层级 1）的内部条目宽 120 → 121，每条末尾多一个 0，别处不动。
/// 预期：冷启动恢复报 `EntryWidthDiffersFromItsFieldTable { what: "inode 树内部条目", entry_bytes: 121, field_table_bytes: 120 }`；
/// 只读挂载报 `RecordMalformed { what: "inode 树内部条目" }`（与窄于字段表同一个成员）；改之前读得出第一个字节。
#[test]
fn an_inode_tree_root_whose_internal_entries_are_wider_than_the_field_table_is_refused_by_recovery_and_by_the_read_only_mount(
) {
    let mut image = build_pool("wider-than-the-field-table-inode-internal-entry").memory_pool();
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "前提：改之前读得出第一个字节"
    );
    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    assert_eq!(
        level_and_entry_width_of_the_node_at(&image, &inode_root),
        (1, INODE_INTERNAL_ENTRY_FIELD_TABLE_BYTES),
        "前提：inode 树根在第 1 层，写者写的内部条目宽等于字段表宽"
    );
    widen_every_entry_of_the_node_by_one_byte(&mut image, &inode_root);
    assert_eq!(
        recovery_failure(&image),
        RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
            what: "inode 树内部条目",
            entry_bytes: INODE_INTERNAL_ENTRY_FIELD_TABLE_BYTES + 1,
            field_table_bytes: INODE_INTERNAL_ENTRY_FIELD_TABLE_BYTES,
        },
        "冷启动恢复：inode 树内部条目宽于字段表，拒、不照读"
    );
    let Err(refusal) = mount_read_only(&image) else {
        panic!("inode 树内部条目宽于字段表，只读挂载却挂上了");
    };
    assert!(
        matches!(
            refusal,
            MountReadOnlyFailure::Open(OpenPoolForReadFailure::RecordMalformed {
                what: "inode 树内部条目"
            })
        ),
        "只读挂载：条目宽与字段表不符，报 RecordMalformed，实际 {refusal:?}"
    );
}

/// 场景：新池新建文件那一版，分配记录树从根沿第 0 条往下走到的那片叶，记录宽 20 → 21，每条末尾多一个 0，别处不动。
/// 预期：可写挂载在任何写之前拒（两盘逐字节不变），报
/// `Recovery(EntryWidthDiffersFromItsFieldTable { what: "分配记录", entry_bytes: 21, field_table_bytes: 20 })`；改之前可写挂载挂得上。
#[test]
fn an_allocation_record_tree_leaf_whose_records_are_wider_than_the_field_table_is_refused_by_the_writable_mount_before_any_write(
) {
    let mut image = build_pool("wider-than-the-field-table-allocation-leaf").memory_pool();
    assert!(writable_mount_accepts(&image), "前提：改之前可写挂载挂得上");
    let leaf = allocation_record_tree_leaf_down_the_first_entries(&image);
    assert_eq!(
        level_and_entry_width_of_the_node_at(&image, &leaf),
        (0, ALLOCATION_RECORD_FIELD_TABLE_BYTES),
        "前提：写者写的分配记录宽等于字段表宽"
    );
    widen_every_entry_of_the_node_by_one_byte(&mut image, &leaf);
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                what: "分配记录",
                entry_bytes: 21,
                field_table_bytes: 20,
            })
        ),
        "可写挂载报分配记录宽与字段表不符：{refusal:?}"
    );
}

/// 场景：新池新建文件那一版，分配记录树根（内部节点）的条目宽 96 → 97，每条末尾多一个 0，别处不动。
/// 预期：可写挂载在任何写之前拒（两盘逐字节不变），报
/// `Recovery(EntryWidthDiffersFromItsFieldTable { what: "分配记录树内部条目", entry_bytes: 97, field_table_bytes: 96 })`；改之前可写挂载挂得上。
#[test]
fn an_allocation_record_tree_root_whose_internal_entries_are_wider_than_the_field_table_is_refused_by_the_writable_mount_before_any_write(
) {
    let mut image = build_pool("wider-than-the-field-table-allocation-internal-node").memory_pool();
    assert!(writable_mount_accepts(&image), "前提：改之前可写挂载挂得上");
    let allocation_root = tree_table_entry_of_kind(&image, TREE_KIND_ALLOCATION).root;
    let (root_level, root_entry_width) =
        level_and_entry_width_of_the_node_at(&image, &allocation_root);
    assert!(
        root_level >= 1 && root_entry_width == ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_FIELD_TABLE_BYTES,
        "前提：分配记录树根是内部节点，写者写的内部条目宽等于字段表宽（层级 {root_level}、条目宽 {root_entry_width}）"
    );
    widen_every_entry_of_the_node_by_one_byte(&mut image, &allocation_root);
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                what: "分配记录树内部条目",
                entry_bytes: 97,
                field_table_bytes: 96,
            })
        ),
        "可写挂载报分配记录树内部条目宽与字段表不符：{refusal:?}"
    );
}

/// 场景：新池新建文件、建快照、快照下覆盖写之后那一版，deadlist 树根（根兼叶，装着换下的那几个单元）的条目宽 43 → 44，每条末尾多一个 0，别处不动。
/// 预期：可写挂载重建那一版读 deadlist 时在任何写之前拒（两盘逐字节不变），报
/// `Recovery(EntryWidthDiffersFromItsFieldTable { what: "deadlist 条目", entry_bytes: 44, field_table_bytes: 43 })`；改之前可写挂载挂得上。
#[test]
fn a_deadlist_leaf_whose_entries_are_wider_than_the_field_table_is_refused_by_the_writable_mount_before_any_write(
) {
    let mut image = image_with_a_deadlist_after_an_overwrite_under_a_snapshot();
    assert!(writable_mount_accepts(&image), "前提：改之前可写挂载挂得上");
    let deadlist_root = tree_table_entry_of_kind(&image, TREE_KIND_DEADLIST).root;
    let deadlist_root_node = parse_index_node(&unit_bytes_at(
        &image,
        the_slot_of(&deadlist_root),
        node_bytes(),
    ))
    .expect("deadlist 树根解得开");
    assert!(
        deadlist_root_node.level == 0
            && !deadlist_root_node.entries.is_empty()
            && deadlist_root_node.entry_width == DEADLIST_ENTRY_FIELD_TABLE_BYTES,
        "前提：deadlist 树根兼叶、装着条目，写者写的条目宽等于字段表宽（层级 {}、{} 条、条目宽 {}）",
        deadlist_root_node.level,
        deadlist_root_node.entries.len(),
        deadlist_root_node.entry_width
    );
    widen_every_entry_of_the_node_by_one_byte(&mut image, &deadlist_root);
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                what: "deadlist 条目",
                entry_bytes: 44,
                field_table_bytes: 43,
            })
        ),
        "可写挂载报 deadlist 条目宽与字段表不符：{refusal:?}"
    );
}
