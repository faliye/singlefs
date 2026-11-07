//! C724（准入抬 F 上限里同名拒因不在单子上）：准入抬 F 算上限那一遍（`mount::rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`）
//! 里有两处 `.ok_or(RecoveryFailure::NoValidRoot)`，报的是读阶段择不到根的同名拒因 `Recovery(NoValidRoot)`，不在 D23（journal 的角色与格式）
//! 已定项 14「在任何写之前拒」那张单子上。欠账行要造的盘面：读阶段与重建分配器那两遍都读得出，算上限那一遍（第三遍）读根环全坏。
//!
//! 这一条造的就是它：可写挂载做完（读阶段、重建分配器两遍都没有读故障），之后根环每一槽每次读都报设备错，再走准入抬 F。
//! 这个进程写过的根（写行、暖机）所在的槽在根环表里记成「这个进程知道住着根的槽」，算上限那一遍读坏就重读一次、仍坏就拒
//! `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`（D16（发布语义） 已定项 1「根槽这一次读坏」那一行），走不到 `NoValidRoot`；
//! 拒在任何写之前，盘上逐字节不变。走不到的理由（推的）见报告：这个进程最新写的那条根 txg 不低于 F、按它自己的实例表有效、它的槽是知道住着根的槽，
//! 读得出就进有效根，读不出就先拒成上面那个拒因。
mod common;
mod common_root_ring_holes;

use common::{
    with_unreadable_ranges, without_unreadable_ranges, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use common_root_ring_holes::{
    devices_of, parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, MountError, ShadowLedger,
};
use singlefs_core::recovery::{every_root_ring_slot, RecoveryFailure};
use singlefs_core::root_ring::slot_offset;

/// 可写挂载之前实例 1 覆盖写到的 txg：环洞池（S = 4 的 12 槽环）上根环转过一圈。
const LAST_TXG_OF_THE_FIRST_INSTANCE: u64 = 17;

#[test]
fn the_admission_ceiling_refuses_with_the_reread_refusal_when_every_root_ring_slot_reads_bad_only_on_its_own_read(
) {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(&pool_overwritten_through(LAST_TXG_OF_THE_FIRST_INSTANCE));
    let mounted = mount_writable(&parameters, &mut devices)
        .expect("可写挂载：读阶段与重建分配器两遍都没有读故障");
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted
        .current
        .into_file_version()
        .expect("覆盖写过的池，这一版带文件");
    let image_before_the_raise = pool_of(&devices);
    let every_ring_slot_reads_bad = SharedUnreadableRanges::new(
        every_root_ring_slot(&parameters.geometry)
            .into_iter()
            .map(|ring_slot| UnreadableRange {
                device: parameters.region_devices
                    [usize::try_from(ring_slot.region).expect("区域号")],
                offset_in_bytes: slot_offset(
                    ring_slot,
                    parameters.geometry.fixed_structure_slot_spacing,
                )
                .0,
                length_in_bytes: u64::from(parameters.geometry.physical_block_size),
                failing_reads: FailingReadsOfARange::Every,
            })
            .collect(),
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices_reading_the_ring_bad =
        with_unreadable_ranges(devices, &every_ring_slot_reads_bad);
    let refused = raise_rollback_floor_to_the_admission_ceiling(
        &parameters,
        &mut devices_reading_the_ring_bad,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect_err("算上限那一遍根环全读坏：准入抬 F 拒");
    assert!(
        every_ring_slot_reads_bad.failed_reads() > 0,
        "前提：算上限那一遍读了根环、读坏"
    );
    assert!(
        !matches!(refused, MountError::Recovery(RecoveryFailure::NoValidRoot)),
        "走不到单子外的同名拒因 Recovery(NoValidRoot)：{refused:?}"
    );
    assert!(
        matches!(
            refused,
            MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        ),
        "这个进程知道住着根的槽读坏、重读仍坏：拒成「根槽这一次读坏」那一行的拒因：{refused:?}"
    );
    assert!(
        pool_of(&without_unreadable_ranges(devices_reading_the_ring_bad)) == image_before_the_raise,
        "拒在任何写之前：两块盘逐字节不变"
    );
}
