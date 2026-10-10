//! 腿自己的探针（只放进草稿副本编）：build_pool（第一个文件那次发布之后，txg 3）时每块盘的已分配 / defer / 空闲槽数与已释放的分配记录。
//! 变异 d（环转之前把回收门槛 0 提前拿 3 当）下第一个文件那次发布就把 mkfs 树表（释放代 3）回收了。
mod common;
use common::build_pool;
#[test]
fn probe() {
    let pool = build_pool("zz-probe");
    for device_map in &pool.allocator.devices {
        println!("PROBE device={:?} allocated={} deferred={} free={}", device_map.device, device_map.allocated_slots(), device_map.deferred_slots(), device_map.free_slots());
    }
    let released: Vec<_> = pool.allocator.records().iter().filter(|r| r.is_released).collect();
    println!("PROBE released_records={released:?}");
}
