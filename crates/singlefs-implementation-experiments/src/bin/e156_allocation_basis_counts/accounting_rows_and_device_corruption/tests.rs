//! 记账三项的读数、镜像字节里的记账条目，与只改盘 0 那一份记账节点的造坏的单测。

use super::new_pool_file_creation_state;
use super::{
    accounting_row_slots, allocated_minus_deferred_matches_referenced,
    allocated_minus_deferred_mismatches_referenced, corrupt_device_zero_accounting,
    memory_pool_snapshot, parameters, referenced_slots,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::transaction::TransactionUnit;
use singlefs_format::SLOT_BYTES;

/// K1-1（E156 第 3 次重跑登记「七」7.2）：分配记录树按位置寻址之后新池新建文件的记账第 1 项是 17
/// （五个树节点占五条记录，不再是登记第一、二版的单节点 13），第 5 项仍是 1。
#[test]
fn accounting_after_new_pool_file_creation_matches_the_registered_anchor() {
    let (allocator, first, _devices) = new_pool_file_creation_state();
    assert_eq!(first.root.checkpoint_txg.0, 3);
    let (allocated, _free, deferred) = accounting_row_slots(&allocator, DeviceIdentity(0));
    assert_eq!(allocated, 17, "K1-1 第 1 项");
    assert_eq!(deferred, 1, "K1-1 第 5 项");
}

/// K1-3：β0 的最新根走读引用 = 16（实例表 2 槽 + 新池新建文件九个角色共 14 槽，五个分配记录树节点
/// 记在这 14 槽里），不再是登记第一、二版的 12。
#[test]
fn referenced_slots_counts_the_carried_instance_table_before_its_first_rewrite() {
    let (allocator, first, _devices) = new_pool_file_creation_state();
    let referenced = referenced_slots(&first);
    assert_eq!(
        referenced, 16,
        "实例表 2 槽（未进 units，靠兜底加回）+ 新池新建文件九个角色共 14 槽（K1-3）"
    );
    assert!(
        allocated_minus_deferred_matches_referenced(&allocator, DeviceIdentity(0), referenced),
        "新池新建文件之后 G27 应当成立：17 − 1 == 16（K1-3）"
    );
}

/// U8：β0 的镜像做 Bd(+1)：G27（R3，读镜像）判红；同一镜像上内存分配器的行没变。
/// `crates/mutations.tsv` 的 M21 把下面这一行的调用换成内存读法的反面，这条测试必须由绿转红。
#[test]
fn red_check_reads_the_corrupted_mirror_not_the_live_allocator() {
    let (allocator, first, devices) = new_pool_file_creation_state();
    let referenced = referenced_slots(&first);
    let accounting_slot = first.unit(TransactionUnit::AccountingTree).slot.0;
    let tree_table_slot = first.unit(TransactionUnit::TreeTable).slot.0;
    let region_devices = parameters().region_devices;
    let spacing = parameters().geometry.fixed_structure_slot_spacing;
    let base_pool = memory_pool_snapshot(&devices);
    let corrupted_pool = corrupt_device_zero_accounting(
        &base_pool,
        accounting_slot,
        tree_table_slot,
        &region_devices,
        spacing,
        &[(5u16, i64::try_from(SLOT_BYTES).expect("16384"))],
    );
    let mirror_red = allocated_minus_deferred_mismatches_referenced(
        &corrupted_pool,
        accounting_slot,
        referenced,
    );
    assert!(mirror_red, "U8：G27（R3，读镜像）在 Bd(+1) 上必须判红");
    let (memory_allocated, _free, memory_deferred) =
        accounting_row_slots(&allocator, DeviceIdentity(0));
    assert_eq!(
        (memory_allocated, memory_deferred),
        (17, 1),
        "U8：同一镜像上内存分配器的行没变（只改了镜像字节，K1-1：17/1）"
    );
}
