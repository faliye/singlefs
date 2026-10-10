//! C694 的 B（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条，用户 2026-10-07 定「C 与 B 都做」）：影子账读一条被抛弃根的账
//! 读不出、从中央映射也现算不成、重读一次仍不成时（`mount::isolate_slots_referenced_only_by_abandoned_roots` 那一支，改之前一律拒），
//! 内存里的现行账有一条分配代大于那条根的 txg、罩住它树表槽（与中央映射树根槽）的记录就不再拒；凭据只读现行账，不读那一槽。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R，崩在 R 的根槽落盘之后、
//! 系统配置轮换之前。
//! - 已被占的那一形：取新实例的那次可写挂载 M1 里 R 的根槽与单元读回全零，M1 关着影子账（只供测试的开关 `ShadowLedger::Off`，连 C 一起关：
//!   造出 C 罩不到的那一形——R 的记录也读回全零时 M1 写行那一条根就盖掉 R 的根槽，造不出被抛弃的 R），M1 与之后的覆盖写把 R 的树表槽与
//!   映射根槽重新分配、写成别的单元（现行账里有那两条记录）；之后 R 的根槽读得出、按实例表被抛弃，账与映射都读不出。三条路都开着影子账。
//! - 瞬时读不出的那一形（同一个崩溃盘面）：M1 里只有 R 的根槽与单元读回全零（`ur`），C 隔离了 R 的落点、谁都没碰它们；卸载那一刻 R 的树表槽与
//!   映射根槽读回全零（一次挂载内两遍都是），现行账里没有罩住它们的记录：照拒。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero, plain_devices_on,
    pool_after_the_mount, record_one_publish_after_overwrites, PublishRecordedAsR,
    RangesReadingBackAllZero, RecordedPublish, OVERWRITE_BYTES,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches,
    raise_rollback_floor_to_the_admission_ceiling, unmount, MountError, RollbackTarget,
    ShadowLedger, StillUnreadableAfterOneReread,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::PoolVersion;
use singlefs_format::SLOT_BYTES;

const OVERWRITES_BEFORE_R: u64 = 5;

/// 已被占那一形里，M1 之后最多再覆盖写几次去等 R 的树表槽与映射根槽都被重新分配（384 槽小盘一圈根环 24 次发布之内）。
const OVERWRITES_AFTER_THE_MOUNT_AT_MOST: u64 = 24;

fn root_of_r(recorded: &RecordedPublish) -> RootRecord {
    RootRecord::parse_slot(
        recorded.root_slot_write().bytes().expect("根槽写带字节"),
        &recorded.pool.parameters.filesystem_identifier,
    )
    .expect("R 的根自证得过")
}

/// R 的树表槽与映射根槽的每一条位置条目。
fn account_slots_of(root: &RootRecord) -> Vec<LocationEntry> {
    root.tree_table
        .locations
        .iter()
        .chain(root.mapping_root.locations.iter())
        .copied()
        .collect()
}

/// 现行账里有一条分配代大于 R 的 txg、罩住这一槽的记录。
fn a_later_record_covers(
    records: &[AllocationRecord],
    root: &RootRecord,
    location: &LocationEntry,
) -> bool {
    records.iter().any(|record| {
        record.device == location.device
            && record.slot.0 <= location.slot.0
            && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
            && record.generation > root.checkpoint_txg
    })
}

fn is_the_refusal_on_the_account_of(error: &MountError, root: &RootRecord) -> bool {
    matches!(
        error,
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
            if matches!(
                still_unreadable.as_ref(),
                StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
                    abandoned_root: RollbackTarget { instance, checkpoint_txg },
                } if (*instance, *checkpoint_txg) == (root.instance, root.checkpoint_txg)
            )
    )
}

/// 已被占的那一形：卸载、会话里为空间推的那一串抬 F、再可写挂载三条路都做得成。
/// 判别力：凭据那一支拿掉（变异「B 的凭据恒不成立」）时三条路都拒成 `AccountOfAnAbandonedRoot`，这一条红。
#[test]
fn an_abandoned_root_whose_account_slots_were_taken_by_later_units_no_longer_refuses_unmount_floor_raise_or_remount(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let root = root_of_r(&recorded);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable_with_test_only_switches(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::Off,
    )
    .expect("R 的根槽与单元读回全零：可写挂载落到 P、做得成");
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
    let mut overwrites_after_the_mount = 0;
    // 迭代上界 OVERWRITES_AFTER_THE_MOUNT_AT_MOST；每一轮看一次现行账，不够再覆盖写一次。
    while !account_slots_of(&root).iter().all(|location| {
        a_later_record_covers(
            &pool.current_file_version().allocation_records,
            &root,
            location,
        )
    }) {
        assert!(
            overwrites_after_the_mount < OVERWRITES_AFTER_THE_MOUNT_AT_MOST,
            "{OVERWRITES_AFTER_THE_MOUNT_AT_MOST} 次覆盖写之内 R 的树表槽与映射根槽都被重新分配"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上这一串覆盖写都落得下");
        overwrites_after_the_mount += 1;
    }
    let image = pool.image();
    let root_slot_write = recorded.root_slot_write();
    assert!(
        root_slot_write
            .contents
            .still_on_disk(&image.devices[&root_slot_write.device].read(
                root_slot_write.offset,
                usize::try_from(root_slot_write.length_in_bytes()).expect("根槽写长"),
            )),
        "R 的根槽还没被这个进程盖掉：R 还在根环里，三条路的影子账都要算它"
    );
    let session = pool.session();
    let PoolVersion::WithFile(current_file_version) = &session.current else {
        panic!("覆盖写之后现行那一版带文件");
    };
    let mut devices_of_the_unmount = plain_devices_on(&image);
    let mut allocator_of_the_unmount = session.allocator.clone();
    let mut current_of_the_unmount = session.current.clone();
    let unmounted = unmount(
        &recorded.pool.parameters,
        &mut devices_of_the_unmount,
        &mut allocator_of_the_unmount,
        &mut current_of_the_unmount,
        ShadowLedger::On,
    );
    assert!(
        unmounted.is_ok(),
        "卸载做得成（R 的账读不出，现行账里有那两槽之后的记录）：{:?}",
        unmounted.err()
    );
    let mut devices_of_the_raise = plain_devices_on(&image);
    let mut allocator_of_the_raise = session.allocator.clone();
    let mut current_of_the_raise = current_file_version.clone();
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &recorded.pool.parameters,
        &mut devices_of_the_raise,
        &mut allocator_of_the_raise,
        &mut current_of_the_raise,
        ShadowLedger::On,
    );
    assert!(
        raised.is_ok(),
        "会话为空间推的那一串抬 F 做得成：{:?}",
        raised.err()
    );
    let mut devices_of_the_remount = plain_devices_on(&image);
    let remounted = mount_writable(&recorded.pool.parameters, &mut devices_of_the_remount);
    assert!(remounted.is_ok(), "再可写挂载做得成：{:?}", remounted.err());
}

/// 瞬时读不出的那一形（同一个崩溃盘面）：现行账里没有罩住 R 树表槽与映射根槽的记录（C 隔离了 R 的落点，谁都没重新分配它们），
/// 那两槽这一次读回全零，卸载照拒、点名 R，盘上逐字节不变。判别力：凭据不看现行账、读不出就放（第一轮攻方 A 那一臂）时这一条红。
#[test]
fn an_abandoned_root_whose_account_slots_only_read_back_zero_still_refuses_the_unmount_before_any_write(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let root = root_of_r(&recorded);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("R 的根槽与单元读回全零：可写挂载落到 P、做得成");
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
    let pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
    assert!(
        !account_slots_of(&root).iter().any(|location| {
            a_later_record_covers(
                &pool.current_file_version().allocation_records,
                &root,
                location,
            )
        }),
        "C 隔离了 R 的落点：现行账里没有罩住 R 树表槽或映射根槽的记录"
    );
    let account_ranges = RangesReadingBackAllZero(
        account_slots_of(&root)
            .iter()
            .map(|location| (location.device, location.slot.0 * SLOT_BYTES, SLOT_BYTES))
            .collect(),
    );
    let image = pool.image();
    let session = pool.session();
    let mut devices_of_the_unmount = devices_reading_back_all_zero_on(&image, &account_ranges);
    let mut allocator_of_the_unmount = session.allocator.clone();
    let mut current_of_the_unmount = session.current.clone();
    let Err(refusal) = unmount(
        &recorded.pool.parameters,
        &mut devices_of_the_unmount,
        &mut allocator_of_the_unmount,
        &mut current_of_the_unmount,
        ShadowLedger::On,
    ) else {
        panic!("R 的账与映射这一次都读回全零、现行账里没有凭据：卸载照拒，而这次卸载做成了");
    };
    assert!(
        is_the_refusal_on_the_account_of(&refusal, &root),
        "拒因点名 R 的账：{refusal:?}"
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices_of_the_unmount, recorded.pool.device_bytes)
            == image,
        "在任何写之前拒：两块盘逐字节不变"
    );
}
