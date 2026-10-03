//! 指针头部里为加密与压缩留的那几段（MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2）第一版写定值，**读指针头部的每一个 core 走读读者**
//! 遇到别的值都判该指针所在的结构损坏（D19（块指针的结构与宽度预算） 已定项 3「加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏」；
//! D9（加密） 已定项 14「读者遇到不认识的码一律判该指针无法解引用（与算法类型那个字段同一条政策）」「码 0 时它恒 0，非 0 判该指针损坏」；
//! `records/2026-09-30-里程碑三开工第一轮.md` 第 8 批 core 簇第 9 条）。多层码 2 树内部条目与 journal 新根段那两类读者早就判
//! （`corrupt_on_disk_content_is_refused_instead_of_panicking.rs`）；根记录这一类还没换（`root_record.rs` 不在这一批）。
//! 这里一类读者一条用例：树表条目的根指针、inode 树内部条目的子指针、extent 上段叶条目里内联的数据指针、extent 叶记录的数据指针、
//! extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针；五种改法轮着用，每条坏镜像只改一处。
//! 影子账认单元的那两个读者照认（多隔离、不漏隔离），在分配记录树与实例表链那两条里一并钉住。
//! 改之前这些读者只判 extent 偏移，加密与压缩那几段跳过、照常读下去。

mod common;
mod common_pointer_head_corruption;

use common::build_pool;
use common_pointer_head_corruption::{
    allocation_record_tree_nodes_as_far_as_readable, change_a_pointer_head_field_inside_the_unit,
    data_pointer_bytes, data_unit_bytes, first_byte_of_the_first_file_read,
    first_entry_of_the_node_at, first_upper_leaf_entry, image_after_a_sequential_write_of,
    image_whose_instance_table_has_two_pages, newest_root, node_bytes, node_pointer_bytes,
    open_the_first_file_refused, recovery_failure, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at, writable_mount_refused_before_any_write, PointerHeadFieldChange,
};
use singlefs_core::allocation_record_tree::ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION;
use singlefs_core::bytes::ByteReader;
use singlefs_core::extent_tree::{
    ExtentUpperLeafTarget,
    EXTENT_LEAF_RECORD_DATA_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
    EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
    EXTENT_UPPER_LEAF_ENTRY_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
};
use singlefs_core::instance_table::{
    InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex,
};
use singlefs_core::mount::MountError;
use singlefs_core::mounted_read::{
    mount_read_only, MountReadOnlyFailure, OpenFileFailure, OpenPoolForReadFailure,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{
    parse_extent_record, parse_inode_internal_entry, TREE_KIND_ALLOCATION, TREE_KIND_EXTENT,
    TREE_KIND_INODE,
};
use singlefs_core::recovery::{
    instance_table_chain_of_root, instance_table_page_pointers_as_far_as_readable, recover,
    JournalPolicy, RecoveryFailure, RecoveryOutcome,
    INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
};
use singlefs_core::unit::parse_index_node;

/// 场景：新池新建文件那一版，树表里 inode 树那条条目的根指针 MAC 第 0 字节改成 1（树表单元重封、根记录补校验和）。
/// 预期：冷启动恢复与可写挂载都报「树表条目」解不开（`UnitMalformed`），可写挂载在任何写之前拒；只读挂载报树表条目解不开。
#[test]
fn a_tree_table_entry_whose_root_pointer_carries_a_non_zero_mac_is_refused() {
    let mut image = build_pool("pointer-mac-tree-table-entry").memory_pool();
    assert!(
        matches!(
            recover(&image, JournalPolicy::Consult).outcome,
            RecoveryOutcome::FileRead { .. }
        ),
        "改之前冷启动读得出文件"
    );
    let tree_table = newest_root(&image).tree_table;
    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    change_a_pointer_head_field_inside_the_unit(
        &mut image,
        the_slot_of(&tree_table),
        node_bytes(),
        &node_pointer_bytes(&inode_root),
        PointerHeadFieldChange::FirstMacByteToOne,
    );
    let tree_table_entry_malformed = RecoveryFailure::UnitMalformed {
        what: "树表条目"
    };
    assert_eq!(
        recovery_failure(&image),
        tree_table_entry_malformed,
        "冷启动恢复：树表条目解不开"
    );
    assert!(
        matches!(
            mount_read_only(&image),
            Err(MountReadOnlyFailure::Open(
                OpenPoolForReadFailure::TreeTableEntryMalformed
            ))
        ),
        "只读挂载：打开挂载态时树表条目解不开"
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == tree_table_entry_malformed),
        "可写挂载：树表条目解不开，实际 {refusal:?}"
    );
}

/// 场景：新池新建文件那一版，inode 树根第 0 条内部条目的子指针 nonce 第 0 字节改成 1。
/// 预期：冷启动恢复报 `UnitMalformed`（[`INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）。
/// 只读挂载那一判在 `mounted_read_tells_a_narrow_inode_internal_entry_from_a_child_pointer_it_cannot_dereference.rs`。
#[test]
fn an_inode_internal_entry_whose_child_pointer_carries_a_non_zero_nonce_is_refused() {
    let mut image = build_pool("pointer-nonce-inode-internal-entry").memory_pool();
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "改之前读得出第一个字节"
    );
    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    let (_separator_key, _identity, child) =
        parse_inode_internal_entry(&first_entry_of_the_node_at(&image, &inode_root))
            .expect("改之前 inode 内部条目解得开");
    change_a_pointer_head_field_inside_the_unit(
        &mut image,
        the_slot_of(&inode_root),
        node_bytes(),
        &node_pointer_bytes(&child),
        PointerHeadFieldChange::FirstNonceByteToOne,
    );
    assert_eq!(
        recovery_failure(&image),
        RecoveryFailure::UnitMalformed {
            what: INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
        },
        "冷启动恢复：inode 内部条目的子指针 nonce 非 0"
    );
}

/// 场景：新池新建文件那一版（3000 字节，一个内联的数据单元），extent 树根里那条上段叶条目内联的数据指针算法类型改成 1。
/// 预期：只读挂载照常挂上，打开文件报 extent 树走读失败（`UnitMalformed`，
/// [`EXTENT_UPPER_LEAF_ENTRY_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）；冷启动恢复报同一个成员。
#[test]
fn an_extent_upper_leaf_entry_whose_inline_data_pointer_names_an_encryption_algorithm_is_refused() {
    let mut image = build_pool("pointer-algorithm-inline-data-pointer").memory_pool();
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "改之前读得出第一个字节"
    );
    let ExtentUpperLeafTarget::InlineDataUnit(data_pointer) = first_upper_leaf_entry(&image).target
    else {
        panic!("3000 字节的文件是一个内联的数据单元（标签 2）");
    };
    let extent_root = tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
    change_a_pointer_head_field_inside_the_unit(
        &mut image,
        the_slot_of(&extent_root),
        node_bytes(),
        &data_pointer_bytes(&data_pointer),
        PointerHeadFieldChange::AlgorithmTypeToOne,
    );
    let upper_leaf_entry_refused = RecoveryFailure::UnitMalformed {
        what: EXTENT_UPPER_LEAF_ENTRY_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
    };
    assert_eq!(
        open_the_first_file_refused(&image),
        OpenFileFailure::ExtentTreeWalk(upper_leaf_entry_refused.clone()),
        "打开文件：上段叶条目内联的数据指针算法类型不是未加密"
    );
    assert_eq!(
        recovery_failure(&image),
        upper_leaf_entry_refused,
        "冷启动恢复：同一个成员"
    );
}

/// 场景：新池新建文件之后顺序写成两个数据单元的文件，下段只有一片叶（根兼叶），它第 0 条叶记录的数据指针压缩算法码改成 1。
/// 预期：打开文件与冷启动恢复都报 `UnitMalformed`（[`EXTENT_LEAF_RECORD_DATA_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）。
#[test]
fn an_extent_leaf_record_whose_data_pointer_names_an_unregistered_compression_algorithm_is_refused()
{
    let mut image = image_after_a_sequential_write_of("pointer-compression-leaf-record", 2);
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "改之前读得出第一个字节"
    );
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&image).target
    else {
        panic!("两个数据单元的文件在上段叶里是一条标签 1 的条目");
    };
    let (_key, data_pointer) =
        parse_extent_record(&first_entry_of_the_node_at(&image, &lower_root))
            .expect("改之前叶记录解得开");
    change_a_pointer_head_field_inside_the_unit(
        &mut image,
        the_slot_of(&lower_root),
        node_bytes(),
        &data_pointer_bytes(&data_pointer),
        PointerHeadFieldChange::CompressionAlgorithmCodeToOne,
    );
    let leaf_record_refused = RecoveryFailure::UnitMalformed {
        what: EXTENT_LEAF_RECORD_DATA_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
    };
    assert_eq!(
        open_the_first_file_refused(&image),
        OpenFileFailure::ExtentTreeWalk(leaf_record_refused.clone()),
        "打开文件：叶记录的数据指针压缩算法码不认识"
    );
    assert_eq!(
        recovery_failure(&image),
        leaf_record_refused,
        "冷启动恢复：同一个成员"
    );
}

/// 下段两层的文件有几个数据单元：比一片下段叶装得下的 144 个多一个（D8（核心索引结构） 已定项 14「实现取值」）。
const DATA_UNITS_OF_A_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT: usize = 145;

/// 场景：145 个数据单元的文件，下段根第 0 条内部条目的子指针压缩算法码仍 0、压后长度改成 1。
/// 预期：打开文件与冷启动恢复都报 `UnitMalformed`（[`EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）。
#[test]
fn an_extent_internal_entry_whose_child_pointer_carries_a_compressed_length_without_compression_is_refused(
) {
    let mut image = image_after_a_sequential_write_of(
        "pointer-compressed-length-extent-internal-entry",
        DATA_UNITS_OF_A_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT,
    );
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "改之前读得出第一个字节"
    );
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&image).target
    else {
        panic!("145 个数据单元的文件在上段叶里是一条标签 1 的条目");
    };
    assert_eq!(
        parse_index_node(&unit_bytes_at(
            &image,
            the_slot_of(&lower_root),
            node_bytes()
        ))
        .expect("下段根解得开")
        .level,
        1,
        "145 个单元：下段根在第 1 层"
    );
    let first_child = NodePointer::read_from(&mut ByteReader::at(
        &first_entry_of_the_node_at(&image, &lower_root),
        24,
    ));
    change_a_pointer_head_field_inside_the_unit(
        &mut image,
        the_slot_of(&lower_root),
        node_bytes(),
        &node_pointer_bytes(&first_child),
        PointerHeadFieldChange::CompressedLengthToOneWithoutCompression,
    );
    let internal_entry_refused = RecoveryFailure::UnitMalformed {
        what: EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
    };
    assert_eq!(
        open_the_first_file_refused(&image),
        OpenFileFailure::ExtentTreeWalk(internal_entry_refused.clone()),
        "打开文件：下段内部条目的子指针码 0 而压后长度非 0"
    );
    assert_eq!(
        recovery_failure(&image),
        internal_entry_refused,
        "冷启动恢复：同一个成员"
    );
}

/// 场景：新池新建文件那一版，分配记录树根第 0 条内部条目的子指针 MAC 第 0 字节改成 1。
/// 预期：可写挂载在任何写之前拒，报 `UnitMalformed`
/// （[`ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）；
/// 影子账认单元的那个读者照认那条条目指的孩子（多隔离、不漏隔离）。
#[test]
fn an_allocation_record_tree_internal_entry_whose_child_pointer_carries_a_non_zero_mac_is_refused_by_the_walk(
) {
    let mut image = build_pool("pointer-mac-allocation-internal-entry").memory_pool();
    let allocation_root = tree_table_entry_of_kind(&image, TREE_KIND_ALLOCATION).root;
    let nodes_before = allocation_record_tree_nodes_as_far_as_readable(&image, &allocation_root);
    let first_child = NodePointer::read_from(&mut ByteReader::at(
        &first_entry_of_the_node_at(&image, &allocation_root),
        10,
    ));
    assert!(
        nodes_before.contains(&first_child),
        "改之前影子账那个读者认得出第 0 条条目指的孩子"
    );
    change_a_pointer_head_field_inside_the_unit(
        &mut image,
        the_slot_of(&allocation_root),
        node_bytes(),
        &node_pointer_bytes(&first_child),
        PointerHeadFieldChange::FirstMacByteToOne,
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            &refusal,
            MountError::Recovery(RecoveryFailure::UnitMalformed { what })
                if *what == ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION
        ),
        "可写挂载：分配记录树内部条目的子指针 MAC 非 0，实际 {refusal:?}"
    );
    let allocation_root_after = tree_table_entry_of_kind(&image, TREE_KIND_ALLOCATION).root;
    let nodes_after =
        allocation_record_tree_nodes_as_far_as_readable(&image, &allocation_root_after);
    assert_eq!(
        nodes_after.len(),
        nodes_before.len(),
        "影子账那个读者照认那条条目指的孩子：改前改后认得出的节点一样多"
    );
}

/// 场景：实例表链两片的那一版，第 0 片末尾那条链指针记录里下一片指针的算法类型改成 1。
/// 预期：沿链读整张实例表报这一片解不开（`UnitMalformed`）；可写挂载在任何写之前拒（`InstanceTableMalformed`）；
/// 影子账认链上各片的那个读者照认两片。
#[test]
fn an_instance_table_chain_pointer_naming_an_encryption_algorithm_is_refused_by_the_walk() {
    let mut image = image_whose_instance_table_has_two_pages();
    let root = newest_root(&image);
    let first_page = unit_bytes_at(&image, the_slot_of(&root.instance_table), data_unit_bytes());
    let InstanceTableChainRecord::NextPage(second_page) =
        InstanceTablePage::parse(&first_page, InstanceTablePageIndex::FIRST)
            .expect("改之前第 0 片解得开")
            .chain
    else {
        panic!("第 0 片的链指针记录是「有下一片」");
    };
    change_a_pointer_head_field_inside_the_unit(
        &mut image,
        the_slot_of(&root.instance_table),
        data_unit_bytes(),
        &node_pointer_bytes(&second_page),
        PointerHeadFieldChange::AlgorithmTypeToOne,
    );
    let root_after = newest_root(&image);
    assert!(
        matches!(
            instance_table_chain_of_root(&image, &root_after),
            Err(RecoveryFailure::UnitMalformed { .. })
        ),
        "沿链读：第 0 片的链指针记录解不开"
    );
    assert_eq!(
        instance_table_page_pointers_as_far_as_readable(&image, &root_after).len(),
        2,
        "影子账那个读者照认两片"
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(&refusal, MountError::InstanceTableMalformed),
        "可写挂载：所选根的实例表沿链有一片解不开，实际 {refusal:?}"
    );
}
