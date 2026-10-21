//! 恢复对条目宽宽于字段表的码 2 节点也拒（欠账 C780；用户 2026-10-18 弹窗定「宽于字段表也拒」，I-1.10（码 2 条目宽等于字段表宽） 写「等于」）：
//! 记账树叶的条目宽 34 → 35、记账树内部节点的条目宽 108 → 109，冷走读与可写挂载都拒、不照读；可写挂载拒在任何写之前，盘上逐字节不变。
//! 两份镜像都从两棵树压小的第一个文件版本起（`common_tree_split::TreeSplitPool`，记账树两层四片叶），只改一个节点、补好引用链上的校验和，
//! 镜像上只剩条目宽这一处说话。映射树叶条目宽于 55 不在这里，账在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）。
//! 判别力：`crates/mutations.tsv` 里以「C780」开头的三行（把拒的那一判改回只拒窄于），各让这里点名的用例红。
mod common;
mod common_images_for_the_checker_tier;
mod common_tree_split;
#[path = "common_images_for_the_checker_tier/accounting_and_mapping_tree_split.rs"]
mod scenarios_of_accounting_and_mapping_tree_split;

use common::{disk_snapshot, parameters};
use common_tree_split::{
    capacities, propagate_the_new_checksum, write_unit_to_the_same_slot_on_both_devices,
    TreeSplitPool,
};
use scenarios_of_accounting_and_mapping_tree_split::{
    image_with_the_root_damaged, node_units, RootDamage, SMALL_ACCOUNTING, SMALL_CENTRAL_MAPPING,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::code_two_tree::{
    build_internal_entry, parse_internal_entry, InternalEntryRefusal,
};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryFailure, RecoveryOutcome};
use singlefs_core::transaction::MultiLevelCodeTwoTree;
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 记账条目字段表的宽（D5（快照 / 空间记账机制） 已定项 5：key 22 + value 8 + seq 4）。
const ACCOUNTING_ENTRY_FIELD_TABLE_BYTES: usize = 34;
/// 记账树内部条目字段表的宽：记账 key 22 + 子指针 86（D8（核心索引结构） 已定项 11）。
const ACCOUNTING_INTERNAL_ENTRY_FIELD_TABLE_BYTES: usize = 108;
/// 记账 key 的宽（统计量标签 2 + 树 ID 8 + 设备 4 + 代 8）。
const ACCOUNTING_KEY_BYTES: usize = 22;

/// 两棵树压小写出第一个文件版本的池：记账树两层四片叶，中央映射树也是多层。
fn pool_with_a_multi_level_accounting_tree() -> TreeSplitPool {
    TreeSplitPool::with_the_first_file_version_under(capacities(
        SMALL_ACCOUNTING,
        SMALL_CENTRAL_MAPPING,
    ))
}

/// 这一版的根（所选根）：恢复拒的时候报在它上面。
fn chosen_root_of(pool: &TreeSplitPool) -> (InstanceGeneration, CheckpointTxg) {
    (pool.output.root.instance, pool.output.root.checkpoint_txg)
}

/// 记账树按 bump 次序第一片叶的条目宽比字段表宽一个字节（34 → 35，每一条末尾补一个 0），节点写回原槽，引用链上的校验和补好。
/// 与 checker 档 `crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 的
/// `image_whose_accounting_leaf_entries_are_wider_than_the_field_table` 同一种改法。
fn image_whose_accounting_leaf_entries_are_wider_than_the_field_table(
    pool: &TreeSplitPool,
) -> MemoryPool {
    let output = &pool.output;
    let leaf_unit = node_units(output, MultiLevelCodeTwoTree::Accounting)
        .into_iter()
        .find(|unit| {
            parse_index_node(&unit.bytes)
                .expect("记账树的节点解得开")
                .level
                == 0
        })
        .expect("压小的记账树长成两层，有叶");
    let leaf = parse_index_node(&leaf_unit.bytes).expect("记账树的叶解得开");
    assert_eq!(
        leaf.entry_width, ACCOUNTING_ENTRY_FIELD_TABLE_BYTES,
        "前提：写者写的记账叶条目宽等于字段表宽"
    );
    let wider_entries: Vec<Vec<u8>> = leaf
        .entries
        .iter()
        .map(|entry| {
            let mut wider = entry.clone();
            wider.push(0);
            wider
        })
        .collect();
    let wider_leaf = build_index_node(
        leaf.tree,
        leaf.level,
        leaf.key_width,
        &leaf.smallest_key,
        &leaf.largest_key,
        leaf.birth_txg,
        &parameters().filesystem_identifier,
        leaf.instance,
        leaf.birth_sequence,
        u16::try_from(leaf.entry_width + 1).expect("条目宽装得进 u16"),
        &wider_entries,
    );
    let mut image = pool.memory_pool();
    write_unit_to_the_same_slot_on_both_devices(&mut image, leaf_unit.slot.0, &wider_leaf);
    let units: Vec<(u64, usize)> = output
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.bytes.len()))
        .collect();
    propagate_the_new_checksum(
        &mut image,
        &units,
        (
            leaf_unit.slot.0,
            crc32_castagnoli(&leaf_unit.bytes),
            crc32_castagnoli(&wider_leaf),
        ),
    );
    image
}

/// 记账树根（内部节点）的条目宽比字段表宽一个字节（108 → 109），引用链上的校验和补好
/// （`scenarios_of_accounting_and_mapping_tree_split::image_with_the_root_damaged` 的 `EntryWiderThanTheInternalEntryFieldTable`）。
fn image_whose_accounting_root_entries_are_wider_than_the_field_table(
    pool: &TreeSplitPool,
) -> MemoryPool {
    image_with_the_root_damaged(
        pool,
        MultiLevelCodeTwoTree::Accounting,
        RootDamage::EntryWiderThanTheInternalEntryFieldTable,
    )
}

/// 把镜像装进录制设备交给可写挂载，交回它的拒绝；之后核两盘系统配置槽逐字节不变、根环没有新根、录制流一步都没多。
fn writable_mount_refusal_with_the_disks_left_unchanged(image: &MemoryPool) -> MountError {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let before = disk_snapshot(image, &stream);
    let refusal = match mount_writable(&parameters(), &mut devices) {
        Ok(_) => panic!("条目宽宽于字段表的镜像，可写挂载必须拒"),
        Err(refusal) => refusal,
    };
    let after_image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: common::IMAGE_BYTES,
    };
    assert_eq!(
        disk_snapshot(&after_image, &stream),
        before,
        "拒在任何写之前：两盘系统配置槽逐字节不变、根环没有新根、录制流一步都没多"
    );
    refusal
}

/// 场景：记账树一片叶的条目宽 35（字段表 34），每条末尾多一个 0，别处不动。
/// 预期：冷走读在所选根上报 `EntryNarrowerThanItsFieldTable { what: "记账条目", entry_bytes: 35, field_table_bytes: 34 }`，不读出文件；
/// 没改的那一份照常读出文件（前提：拒的只是条目宽）。
#[test]
fn recovery_refuses_an_accounting_leaf_whose_entries_are_wider_than_the_field_table() {
    let pool = pool_with_a_multi_level_accounting_tree();
    assert!(
        matches!(
            recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
            RecoveryOutcome::FileRead { .. }
        ),
        "前提：没改的那一份冷走读读出文件"
    );
    let image = image_whose_accounting_leaf_entries_are_wider_than_the_field_table(&pool);
    assert_eq!(
        recover(&image, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some(chosen_root_of(&pool)),
            failure: RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "记账条目",
                entry_bytes: ACCOUNTING_ENTRY_FIELD_TABLE_BYTES + 1,
                field_table_bytes: ACCOUNTING_ENTRY_FIELD_TABLE_BYTES,
            },
        },
        "记账条目宽于字段表：冷走读拒、不照读"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：重建那一版读记账树时同样判条目宽，在取号与任何写之前交回
/// `Recovery(EntryNarrowerThanItsFieldTable { what: "记账条目", entry_bytes: 35, field_table_bytes: 34 })`，盘上逐字节不变。
#[test]
fn writable_mount_refuses_an_accounting_leaf_whose_entries_are_wider_than_the_field_table_before_any_write(
) {
    let pool = pool_with_a_multi_level_accounting_tree();
    let image = image_whose_accounting_leaf_entries_are_wider_than_the_field_table(&pool);
    let refusal = writable_mount_refusal_with_the_disks_left_unchanged(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "记账条目",
                entry_bytes: 35,
                field_table_bytes: 34,
            })
        ),
        "可写挂载报记账条目宽与字段表不符：{refusal:?}"
    );
}

/// 场景：记账树根（层级 1 的内部节点）的条目宽 109（字段表 key 22 + 子指针 86 = 108），每条末尾多一个 0，别处不动。
/// 预期：冷走读在所选根上报 `EntryNarrowerThanItsFieldTable { what: "记账树内部条目", entry_bytes: 109, field_table_bytes: 108 }`，
/// 不往下读孩子、不读出文件；没改的那一份照常读出文件（前提：拒的只是条目宽）。
#[test]
fn recovery_refuses_an_accounting_tree_internal_node_whose_entries_are_wider_than_the_field_table()
{
    let pool = pool_with_a_multi_level_accounting_tree();
    assert!(
        matches!(
            recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
            RecoveryOutcome::FileRead { .. }
        ),
        "前提：没改的那一份冷走读读出文件"
    );
    let image = image_whose_accounting_root_entries_are_wider_than_the_field_table(&pool);
    assert_eq!(
        recover(&image, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some(chosen_root_of(&pool)),
            failure: RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "记账树内部条目",
                entry_bytes: ACCOUNTING_INTERNAL_ENTRY_FIELD_TABLE_BYTES + 1,
                field_table_bytes: ACCOUNTING_INTERNAL_ENTRY_FIELD_TABLE_BYTES,
            },
        },
        "记账树内部条目宽于字段表：冷走读拒、不照读"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：重建那一版读记账树时同样判内部条目宽，在取号与任何写之前交回
/// `Recovery(EntryNarrowerThanItsFieldTable { what: "记账树内部条目", entry_bytes: 109, field_table_bytes: 108 })`，盘上逐字节不变。
#[test]
fn writable_mount_refuses_an_accounting_tree_internal_node_whose_entries_are_wider_than_the_field_table_before_any_write(
) {
    let pool = pool_with_a_multi_level_accounting_tree();
    let image = image_whose_accounting_root_entries_are_wider_than_the_field_table(&pool);
    let refusal = writable_mount_refusal_with_the_disks_left_unchanged(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "记账树内部条目",
                entry_bytes: 109,
                field_table_bytes: 108,
            })
        ),
        "可写挂载报记账树内部条目宽与字段表不符：{refusal:?}"
    );
}

/// 场景：一条记账树内部条目（key 22 + 子指针 86）末尾多一个 0，交给解内部条目的那一个函数。
/// 预期：`parse_internal_entry` 报 `WidthOtherThanTheKeyAndTheChildPointer`；刚好 108 的那一条照常解出分隔 key 与子指针。
#[test]
fn a_code_two_internal_entry_wider_than_the_key_and_the_child_pointer_is_refused_by_the_entry_parser(
) {
    let pool = pool_with_a_multi_level_accounting_tree();
    let child = pool
        .output
        .multi_level_tree(MultiLevelCodeTwoTree::Accounting)
        .pointers[0];
    let separator_key = [7u8; ACCOUNTING_KEY_BYTES];
    let entry = build_internal_entry(&separator_key, child);
    assert_eq!(entry.len(), ACCOUNTING_INTERNAL_ENTRY_FIELD_TABLE_BYTES);
    assert_eq!(
        parse_internal_entry(&entry, ACCOUNTING_KEY_BYTES),
        Ok((separator_key.to_vec(), child)),
        "前提：刚好 108 的内部条目解得开"
    );
    let mut wider_entry = entry;
    wider_entry.push(0);
    assert_eq!(
        parse_internal_entry(&wider_entry, ACCOUNTING_KEY_BYTES),
        Err(InternalEntryRefusal::WidthOtherThanTheKeyAndTheChildPointer),
        "宽于 key 宽 + 86 的内部条目不解"
    );
}
