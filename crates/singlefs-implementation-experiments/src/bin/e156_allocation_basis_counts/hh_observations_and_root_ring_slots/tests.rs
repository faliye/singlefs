//! 岔路 1 的族 Hh：观测点、臂与回收时机、一趟的请求与结局、树表的谱系与根环槽的内容、挂载时读回全零的槽的单测。

use super::new_pool_file_creation_state;
use super::{parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::ReclaimedReuse;
use singlefs_core::transaction::{FirstFile, PoolWriter};

#[test]
fn reclaimed_records_are_not_removed_but_their_slots_become_free() {
    // `PoolAllocator::reclaim_released_up_to` 不删记录（`allocator.rs` I8 字面）：一条记录被真实分配器回收之后，
    // `is_released` 仍是 `true`、条目还留在 `records()` 里，直到下一次 `record()` 落在同一个起点槽上才被改写；
    // 但它的槽此刻已经是空闲的（`DeviceFreeMap::is_free`）。岔路 1 的「已释放未回收」按 `is_free` 现查，
    // 这条测试钉住这份读法赖以成立的前提本身。
    let (mut allocator, first, mut devices) = new_pool_file_creation_state();
    let parameters = parameters();
    let instance = singlefs_core::address::InstanceGeneration(1);
    let overwritten = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        singlefs_core::transaction::publish_overwrite(
            &mut pool,
            &mut allocator,
            &first,
            FirstFile {
                content: &super::overwrite_content(1),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 61,
            },
            instance,
        )
        .expect("一次覆盖写")
    };
    let released_slot = allocator
        .records()
        .iter()
        .find(|record| {
            record.device == DeviceIdentity(0)
                && record.is_released
                && record.generation == overwritten.root.checkpoint_txg
        })
        .map(|record| record.slot)
        .expect("这次覆盖写应当释放至少一条记录");
    allocator.reclaim_released_up_to(overwritten.root.checkpoint_txg, ReclaimedReuse::Immediately);
    let still_marked_released = allocator.records().iter().any(|record| {
        record.device == DeviceIdentity(0) && record.slot == released_slot && record.is_released
    });
    assert!(
        still_marked_released,
        "回收之后记录条目应当仍留着、`is_released` 仍是 true（I8：不删、不点删）"
    );
    let device_free_map_for_device_zero = allocator
        .devices
        .iter()
        .find(|map| map.device == DeviceIdentity(0))
        .expect("D0 在池里");
    assert!(
        device_free_map_for_device_zero.is_free(released_slot),
        "回收之后这个槽的位图应当已经清空（`is_free` 为 true），即使记录条目还标着 `is_released`"
    );
}
