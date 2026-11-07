//! 公开的准入抬 F 入口在一条有效根都读不出时交回 `Recovery(NoValidRoot)`、不 panic（C724（准入抬 F 上限里同名拒因不在单子上）；
//! 代码轮 `research/prompts/m3-close-b19-code-r1-main-verification.md` 第二节 T2：core 判决 `research/prompts/m3-close-b19-core-r1-main-verification.md`
//! 第二节 R2 定共享函数保留这个错，有外部调用方）。
//! 外部调用方手里的分配器可以不带这个进程的根环表（`PoolAllocator::new` 拼的）：算上限读根环时读坏的槽一律当没有根，根环全读坏就一条有效根都没有。
//! 「这个进程写出的现行根都经记根进了根环表」那条前提只在本进程准入路径上成立，断言只留在那一处（`mount::push_one_floor_raise_within_the_admission_budget`）。
//!
//! 两条用例各一种读坏：根环每一槽每次读都报设备错；每一槽每次读都读回整块 0。可写挂载做完（读阶段、重建分配器两遍都没有读故障）之后，
//! 拿挂载那一份分配器的盘账另拼一份不带根环表的分配器，走公开入口 `raise_rollback_floor_to_the_admission_ceiling`。
mod common;
mod common_root_ring_holes;

use common::{
    with_unreadable_ranges, without_unreadable_ranges, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use common_root_ring_holes::{
    devices_of, parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, MountError, ShadowLedger,
};
use singlefs_core::recovery::{every_root_ring_slot, RecoveryFailure};
use singlefs_core::root_ring::slot_offset;

/// 可写挂载之前实例 1 覆盖写到的 txg：环洞池（S = 4 的 12 槽环）上根环转过一圈。
const LAST_TXG_OF_THE_FIRST_INSTANCE: u64 = 17;

/// 可写挂载做完之后根环每一槽每次读都照 `read_back` 读坏，拿不带根环表的分配器走公开的准入抬 F 入口：交回 `Recovery(NoValidRoot)`，
/// 根环真被读过、读坏，两块盘逐字节不变。
fn the_public_admission_ceiling_entry_returns_no_valid_root_when_every_root_ring_slot_reads_bad(
    read_back: UnreadableRangeReadBack,
) {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(&pool_overwritten_through(LAST_TXG_OF_THE_FIRST_INSTANCE));
    let mounted = mount_writable(&parameters, &mut devices)
        .expect("可写挂载：读阶段与重建分配器两遍都没有读故障");
    assert!(
        mounted.allocator.root_ring_occupancy().is_some(),
        "前提：挂载那一份分配器带这个进程的根环表"
    );
    let mut allocator_without_the_root_ring_table =
        PoolAllocator::new(mounted.allocator.devices.clone());
    assert!(
        allocator_without_the_root_ring_table
            .root_ring_occupancy()
            .is_none(),
        "前提：另拼的那一份不带根环表（外部调用方手里的那一种）"
    );
    let mut current = mounted
        .current
        .into_file_version()
        .expect("覆盖写过的池，这一版带文件");
    assert!(
        current.root.rollback_floor <= current.root.checkpoint_txg,
        "前提：现行那一版的根带的 F 不高于它自己的 txg（入口那条断言不该先响）"
    );
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
        read_back,
    );
    let mut devices_reading_the_ring_bad =
        with_unreadable_ranges(devices, &every_ring_slot_reads_bad);
    let refused = raise_rollback_floor_to_the_admission_ceiling(
        &parameters,
        &mut devices_reading_the_ring_bad,
        &mut allocator_without_the_root_ring_table,
        &mut current,
        ShadowLedger::On,
    )
    .expect_err("根环全读坏、分配器不知道哪几槽住着根：一条有效根都没有，公开入口交回错误");
    assert!(
        every_ring_slot_reads_bad.failed_reads() > 0,
        "前提：算上限那一遍读了根环、读坏"
    );
    assert!(
        matches!(refused, MountError::Recovery(RecoveryFailure::NoValidRoot)),
        "公开入口交回共享函数保留的那个错 Recovery(NoValidRoot)：{refused:?}"
    );
    assert!(
        pool_of(&without_unreadable_ranges(devices_reading_the_ring_bad)) == image_before_the_raise,
        "拒在任何写之前：两块盘逐字节不变"
    );
}

/// 场景：可写挂载做完之后根环每一槽每次读都报设备错，拿不带根环表的分配器走公开的准入抬 F 入口。
/// 预期：交回 `Recovery(NoValidRoot)`，不 panic；两块盘逐字节不变。
/// 判别力：公开入口改回断言（`crates/mutations.tsv`「C724 公开的准入抬 F 入口在一条有效根都没有时断言失败（改回 panic）」）时这一条 panic、红。
#[test]
fn the_public_admission_ceiling_entry_returns_no_valid_root_when_every_root_ring_slot_reports_a_device_error(
) {
    the_public_admission_ceiling_entry_returns_no_valid_root_when_every_root_ring_slot_reads_bad(
        UnreadableRangeReadBack::DeviceError,
    );
}

/// 场景：可写挂载做完之后根环每一槽每次读都读回整块 0，拿不带根环表的分配器走公开的准入抬 F 入口。
/// 预期：交回 `Recovery(NoValidRoot)`，不 panic；两块盘逐字节不变。
/// 判别力：同上一条（「C724 公开的准入抬 F 入口在一条有效根都没有时断言失败（改回 panic）」）。
#[test]
fn the_public_admission_ceiling_entry_returns_no_valid_root_when_every_root_ring_slot_reads_back_zeros(
) {
    the_public_admission_ceiling_entry_returns_no_valid_root_when_every_root_ring_slot_reads_bad(
        UnreadableRangeReadBack::Zeros,
    );
}
