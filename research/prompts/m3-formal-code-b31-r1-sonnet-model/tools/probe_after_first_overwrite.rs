//! 腿自己的探针（只放进草稿副本编）：build_pool 之后做一次覆盖写（txg 4），看 mkfs 树表那个落点（槽 50178，释放代 3）的分配记录是不是被新的分配改写了。
mod common;
use common::{build_pool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::InstanceGeneration;
#[test]
fn probe() {
    let mut pool = build_pool("zz-probe2");
    let previous = pool.output.clone();
    pool.output = common::publish_overwrite_in_process(
        &mut pool,
        &previous,
        &vec![7u8; 3000],
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .unwrap();
    for device_map in &pool.allocator.devices {
        println!("PROBE2 device={:?} allocated={} deferred={} free={}", device_map.device, device_map.allocated_slots(), device_map.deferred_slots(), device_map.free_slots());
    }
    let at_slot: Vec<_> = pool.allocator.records().iter().filter(|r| r.slot.0 == 50178).collect();
    println!("PROBE2 records_at_slot_50178={at_slot:?}");
}
