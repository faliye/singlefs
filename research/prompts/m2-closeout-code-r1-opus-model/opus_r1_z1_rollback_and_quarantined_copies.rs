//! 代码轮第一轮云端攻方腿（m2-closeout-code-r1）Z1 那一格：管理员回退碰上「释放时读盘核对不上、两份都留在已分配」的单元
//! （D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 那一格，下称隔离单元）。
//! 甲：回退目标引用隔离单元——它在现行账里仍分配、分配代与目标那一版相同，按「两版共用」处理，不进复活集、不逐盘验；
//!     它盘 1 那一份是坏的，回退照样做成。
//! 乙：回退目标不引用隔离单元——D23（journal 的角色与格式） 已定项 14「释放」一格字面是「cur 那一版的账里仍分配、而 new 不引用的落点」，
//!     隔离单元在现行账里仍分配、新根不引用；代码只释放现行那一版经映射引用的用户可见单元，隔离单元留在已分配。
//! 用例钉快照今天的行为，打印两格的读数。
mod common;
mod common_admission;

use common_admission::{start_plain, OVERWRITE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::history::HistoryDeviceWidth;

fn corrupt_first_data_unit_copy_on_device_one(pool: &mut common_admission::PoolUnderTest<singlefs_harness::crash::SparseBlockDevice>) -> (DeviceIdentity, u64) {
    let pointer = pool.current_file_version().data_pointers[0];
    let location = pointer
        .locations
        .iter()
        .find(|location| location.device == DeviceIdentity(1))
        .copied()
        .expect("盘 1 上有一份");
    let (_, device) = pool.devices.iter_mut().find(|(identity, _)| *identity == DeviceIdentity(1)).expect("盘 1");
    device.image.write(location.slot.to_device_offset(), &vec![0xA5_u8; 4096]);
    (location.device, location.slot.0)
}

fn allocation_record_of(current: &singlefs_core::transaction::TransactionOutput, device: DeviceIdentity, slot: u64) -> Option<(bool, u64)> {
    current
        .allocation_records
        .iter()
        .find(|record| record.device == device && record.slot.0 == slot)
        .map(|record| (record.is_released, record.generation.0))
}

#[test]
fn rollback_to_a_version_referencing_a_quarantined_unit_is_accepted_without_checking_its_copies() {
    let mut pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    let target_root = pool.current_file_version().root;
    let (device, slot) = corrupt_first_data_unit_copy_on_device_one(&mut pool);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写：换下的数据单元盘 1 那一份核不上，两份都留在已分配");
    let quarantined = pool.current_file_version().quarantined_after_release_checksum_mismatch.len();
    pool.overwrite(OVERWRITE_BYTES).expect("再覆盖写一次");
    let violations_before = common_admission::checker_violations_on(&pool.image());
    println!("Z1Q-A checker_violations_before_rollback={} first={:?}", violations_before.len(), violations_before.first());
    let session = pool.session.as_mut().expect("会话");
    let PoolVersion::WithFile(current) = &mut session.current else { panic!() };
    println!("Z1Q-A quarantined_copies_at_overwrite={quarantined} record_before_rollback={:?}", allocation_record_of(current, device, slot));
    let outcome = roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        current,
        RollbackTarget { instance: target_root.instance, checkpoint_txg: target_root.checkpoint_txg },
    );
    let accepted = outcome.is_ok();
    println!("Z1Q-A rollback_to_txg{} accepted={accepted} err={:?} resurrected={:?}", target_root.checkpoint_txg.0, outcome.as_ref().err(), outcome.as_ref().map(|done| done.resurrected_user_visible_copies.len()).ok());
    let violations = common_admission::checker_violations_on(&pool.image());
    println!("Z1Q-A checker_violations_after={} first={:?}", violations.len(), violations.first());
    assert!(quarantined >= 1, "那一次覆盖写真的隔离了");
    assert!(accepted, "快照今天：目标引用的隔离单元不进复活集、不逐盘验，回退做成");
    // 回退之前池级 checker 已经红在同一处（环里目标那条根引用着坏的那一份）：这一格不是回退造出来的红，是回退没拒。
    assert_eq!(violations_before.first(), violations.first(), "回退前后 checker 红在同一处（I-2.1，盘 1 那一份）");
}

#[test]
fn rollback_to_a_version_not_referencing_a_quarantined_unit_leaves_it_allocated() {
    let mut pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    let (device, slot) = corrupt_first_data_unit_copy_on_device_one(&mut pool);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写：隔离");
    let target_root = pool.current_file_version().root;
    pool.overwrite(OVERWRITE_BYTES).expect("再覆盖写一次");
    let session = pool.session.as_mut().expect("会话");
    let PoolVersion::WithFile(current) = &mut session.current else { panic!() };
    let before = allocation_record_of(current, device, slot);
    let outcome = roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        current,
        RollbackTarget { instance: target_root.instance, checkpoint_txg: target_root.checkpoint_txg },
    )
    .expect("回退做成");
    let after = allocation_record_of(current, device, slot);
    let rollback_txg = current.root.checkpoint_txg;
    println!(
        "Z1Q-B target_txg={} rollback_txg={} quarantined_slot={slot} record_before={before:?} record_after={after:?} released_user_visible_units={}",
        target_root.checkpoint_txg.0, rollback_txg.0, outcome.released_user_visible_units.len()
    );
    // 字面读法：隔离单元在 cur 的账里仍分配、新根不引用 ⇒ 释放，释放代 = 新根的 txg。快照今天：留在已分配、分配代不变。
    assert_eq!(after, before, "快照今天：隔离单元的记录回退前后逐字段相同（仍分配）");
    assert_eq!(after.map(|(is_released, _)| is_released), Some(false));
    let _ = CheckpointTxg(0);
}
