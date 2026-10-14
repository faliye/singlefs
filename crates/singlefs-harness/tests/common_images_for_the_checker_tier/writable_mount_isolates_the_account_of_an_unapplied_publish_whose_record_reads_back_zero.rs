//! harness 测试文件 `crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero, plain_devices_on,
    pool_after_the_mount, record_one_publish_after_overwrites, units_of_r_overwritten_on,
    PublishRecordedAsR, RangesReadingBackAllZero, OVERWRITE_BYTES,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger};
use singlefs_harness::memory_pool::MemoryPool;

/// 覆盖写几次之后录 R：攻方那一形量过 k = 1 与 4（两者逐格同数），取 1。
pub const OVERWRITES_BEFORE_R: u64 = 1;

/// M1 里 R 的根槽与 R 的第 `hidden_record` 条记录读回全零；M1 之后覆盖写一次、正常卸载，逐步核 R 的单元没被写掉、卸载之后 I-7.4 不红。
/// 交回 M1 之后每块盘上分配器隔离的槽数（报告里写的「M1 隔离槽数」）。
pub fn assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(
    images: &mut PoolCheckerImages,
    hidden_record: usize,
) -> Vec<(DeviceIdentity, u64)> {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.record_writes_of_the_record(hidden_record));
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("R 缺一条记录、读得出的记录点名的单元验得过：F1 乙不拒，可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽读回全零、缺一条记录：R 整次不施加，所选那一版是 P"
    );
    let isolated_slots_after_the_mount: Vec<(DeviceIdentity, u64)> = mounted
        .allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect();
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
    pool.overwrite(OVERWRITE_BYTES)
        .expect("384 槽小盘上 M1 之后覆盖写一次落得下");
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &pool.image()),
        Vec::<(DeviceIdentity, u64)>::new(),
        "M1 之后第一次覆盖写没取到 R 的单元（C 隔离了 R 的账里的落点）"
    );
    let session = pool.session();
    let mut devices_of_the_unmount = plain_devices_on(&pool.image());
    let mut allocator_of_the_unmount = session.allocator.clone();
    let mut current_of_the_unmount = session.current.clone();
    unmount(
        &recorded.pool.parameters,
        &mut devices_of_the_unmount,
        &mut allocator_of_the_unmount,
        &mut current_of_the_unmount,
        ShadowLedger::On,
    )
    .expect("M1 之后覆盖写一次的池正常卸载做得成");
    let image_after_the_unmount = MemoryPool {
        devices: devices_of_the_unmount
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: recorded.pool.device_bytes,
    };
    assert_eq!(
        units_of_r_overwritten_on(&recorded, &image_after_the_unmount),
        Vec::<(DeviceIdentity, u64)>::new(),
        "卸载那一串空发布也没取到 R 的单元"
    );
    // 卸载之后池级 checker 不红 I-7.4：被抛弃的 R 引用的单元没被重新分配
    images.hand_out(
        "卸载之后",
        || image_after_the_unmount,
        PoolCheckerExpectation::only_the_named_invariants().and_not_violated("I-7.4"),
    );
    isolated_slots_after_the_mount
}

/// `rL`：末条读回全零，末条单独点名的第三个数据单元与那几个元数据单元只在 R 的账里。
/// 判别力：C 不并 R 的账（变异「C 不并没施加的发布的账（末条读回全零）」）时 M1 之后第一次覆盖写就写掉 R 的单元（攻方量到 2 份）、这一条红。
pub fn a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
    images: &mut PoolCheckerImages,
) {
    let isolated_slots =
        assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(images, 3);
    println!("M1 隔离槽数（rL）：{isolated_slots:?}");
}

/// `rM`：中间一条读回全零，它单独点名的第二个数据单元只在 R 的账里。判别力：C 不并 R 的账（变异「C 不并没施加的发布的账（中间一条读回全零）」）时这一条红。
pub fn a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
    images: &mut PoolCheckerImages,
) {
    let isolated_slots =
        assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(images, 2);
    println!("M1 隔离槽数（rM）：{isolated_slots:?}");
}

/// `rF`：第一条读回全零，它单独点名的第一个数据单元只在 R 的账里。判别力：C 不并 R 的账（变异「C 不并没施加的发布的账（第一条读回全零）」）时这一条红。
pub fn a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
    images: &mut PoolCheckerImages,
) {
    let isolated_slots =
        assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(images, 1);
    println!("M1 隔离槽数（rF）：{isolated_slots:?}");
}
