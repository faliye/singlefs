//! 代码审阅第 23 条（实审 A1）：准入不够时推的抬 F 半路报块设备错，被归成「推满仍不够」——可写挂载那一处交回
//! `MountSpaceAdmission::StillShortAfterTheFloorRaises`、挂载照样做成，会话那一处交回 `UserChangeRefused::NoSpaceAfterRaisingTheFloor`
//! （报给用户就是 ENOSPC）。C565（挂载处推满仍不够怎么收尾没定） 那一格的例外（D16（发布语义） 已定项 1「准入」那一行）只管空间不够。
//! 改成：抬 F 自己报的错原样往上交——可写挂载返回那个 `MountError`（装在 `MountError::FloorRaiseFailedAfterTheMountsPublishes` 里、
//! 连同写行与暖机已落盘的写账，实审 A1b Q5），会话返回 `UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`；
//! 只有预算用完、F 已在上限才走「推满仍不够」。
//!
//! 场景照 `admission_raises_the_floor_before_refusing.rs` 的
//! `mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount`：
//! 两块单元区 240 槽的小盘，第一个文件之后连着崩了再挂，第 10 次取号之前不够、写行与暖机之后推抬 F；那次挂载的会话里覆盖写一次，
//! 被空间准入拒、推抬 F。注入的是抬 F 那一串先写系统配置那一步里盘 0 的那次系统配置槽写（块设备错）。

mod common_admission;

use common_admission::{PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::mount::{MountError, MountSpaceAdmission};
use singlefs_core::mounted_session::UserChangeRefused;
use singlefs_core::transaction::PublishError;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf240Slots;

type FaultInjectingDevice = FaultInjectingBlockDevice<SparseBlockDevice>;

/// 两块单元区 240 槽的小盘，外面包一层故障注入（计划先不上膛），mkfs、取号、暖机、第一个文件，之后崩了再挂 9 次（都做成）。
fn pool_after_nine_crash_remounts() -> (PoolUnderTest<FaultInjectingDevice>, SharedFaultPlan) {
    let plan = SharedFaultPlan::unarmed(DEVICE_WIDTH.fixed_geometry());
    let mut pool = PoolUnderTest::start_after_the_first_file(DEVICE_WIDTH, |identity, sparse| {
        FaultInjectingBlockDevice::new(identity, sparse, plan.clone())
    });
    for mount_index in 1..=9 {
        pool.crash_and_mount_writable()
            .unwrap_or_else(|error| panic!("第 {mount_index} 次崩了再挂做成：{error:?}"));
    }
    (pool, plan)
}

/// 盘 0 上从上膛起数第 `ordinal` 次系统配置槽写（两槽住在偏移 0 起的两个槽距之内）报块设备错，只这一次。
fn fail_the_system_configuration_write_on_device_zero(
    pool: &PoolUnderTest<FaultInjectingDevice>,
    ordinal: u64,
) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(0)),
        placement: FaultPlacement::OffsetBelow(
            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                * u64::from(pool.parameters.geometry.fixed_structure_slot_spacing),
        ),
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
    }
}

/// 第 10 次崩了再挂：取号之前不够，写行与暖机之后推抬 F，那一串先写系统配置那一步里盘 0 的写报块设备错。
/// 盘 0 在这次挂载里的系统配置槽写依次是取号 1 次、写行那次发布的轮换 1 次、暖机每次轮换 1 次，第一串抬 F 先写系统配置是再下一次——
/// 暖机几次先在盘面拷贝上不注入地挂一遍数出来（那一遍推满仍不够、推了两串）。可写挂载返回抬 F 的错原样
/// （`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`，一块盘都还没带上新 F），不再交回「推满仍不够」、不当挂载做成。
#[test]
fn a_block_device_error_in_the_floor_raise_pushed_by_the_mount_fails_the_mount_instead_of_reporting_still_short(
) {
    let (mut pool, plan) = pool_after_nine_crash_remounts();
    let mut copy_devices = common_admission::plain_devices_on(&pool.image());
    let rehearsal = singlefs_core::mount::mount_writable_with_space_admission(
        &pool.parameters,
        &mut copy_devices,
        singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
    )
    .expect("拷贝上不注入：第 10 次挂载做成");
    assert!(
        matches!(
            &rehearsal.output.space_admission,
            MountSpaceAdmission::StillShortAfterTheFloorRaises { floor_raises, .. }
                if floor_raises.len() == 2
        ),
        "拷贝上不注入：第 10 次推了两串、推满仍不够：{:?}",
        rehearsal.output.space_admission
    );
    let system_configuration_writes_on_device_zero_before_the_first_raise =
        1 + 1 + u64::try_from(rehearsal.output.warm_up_publishes.len()).expect("暖机次数");
    plan.arm(fail_the_system_configuration_write_on_device_zero(
        &pool,
        system_configuration_writes_on_device_zero_before_the_first_raise + 1,
    ));
    let refusal = pool
        .crash_and_mount_writable()
        .expect_err("抬 F 先写系统配置那一步报块设备错：可写挂载交回那个错，不当推满仍不够做成");
    plan.disarm();
    // 抬 F 的错装在 `FloorRaiseFailedAfterTheMountsPublishes` 里、连同写行与暖机已落盘的写账交回（实审 A1b Q5，账怎么核见
    // `core_review_unit_area_start_and_publish_limits.rs`）；这里只核里面那个错是原样的块设备错、不是「推满仍不够」。
    let MountError::FloorRaiseFailedAfterTheMountsPublishes(failed_after_the_mounts_publishes) =
        refusal
    else {
        panic!("该交回 FloorRaiseFailedAfterTheMountsPublishes，实际 {refusal:?}")
    };
    let MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed) =
        failed_after_the_mounts_publishes.cause
    else {
        panic!(
            "里面该是抬 F 的错原样 RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot，实际 {:?}",
            failed_after_the_mounts_publishes.cause
        )
    };
    assert_eq!(
        failed.devices_carrying_the_raised_floor,
        Vec::<DeviceIdentity>::new(),
        "盘 0 是这一步的第一个写：一块盘都还没带上新 F"
    );
}

/// 第 10 次挂载做成（推满仍不够）之后，会话里覆盖写一次：被空间准入拒，推第一串抬 F，那一串先写系统配置那一步里盘 0 的写报块设备错。
/// 会话交回 `FloorRaiseFailedWhilePushingForSpace`：`cause` 是抬 F 的错原样，推它的是那次空间准入拒绝，之前没有推成的串；
/// 不交回「空间不够」（`NoSpaceAfterRaisingTheFloor`）。
#[test]
fn a_block_device_error_in_the_floor_raise_pushed_by_the_session_is_not_reported_as_no_space() {
    let (mut pool, plan) = pool_after_nine_crash_remounts();
    let output = pool
        .crash_and_mount_writable()
        .expect("第 10 次：推满仍不够，挂载照样做成");
    assert!(
        matches!(
            output.space_admission,
            MountSpaceAdmission::StillShortAfterTheFloorRaises { .. }
        ),
        "第 10 次推满仍不够：{:?}",
        output.space_admission
    );
    plan.arm(fail_the_system_configuration_write_on_device_zero(&pool, 1));
    let write = pool.overwrite(OVERWRITE_BYTES);
    plan.disarm();
    let Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed)) = write else {
        panic!("该报 FloorRaiseFailedWhilePushingForSpace（抬 F 报块设备错，不是空间不够），实际 {write:?}")
    };
    assert!(
        matches!(
            failed.cause,
            MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        ),
        "cause 是抬 F 的错原样：{:?}",
        failed.cause
    );
    assert!(
        matches!(
            failed.refusal_that_pushed_the_failed_raise,
            PublishError::SpaceAdmissionRefused(_)
        ),
        "推这一串的是那次空间准入拒绝：{:?}",
        failed.refusal_that_pushed_the_failed_raise
    );
    assert!(
        failed.floor_raises.is_empty() && failed.refusals_that_pushed_the_floor_raises.is_empty(),
        "报错的是第一串，之前没有推成的：{} 串",
        failed.floor_raises.len()
    );
}
