//! harness 测试文件 `crates/singlefs-harness/tests/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_root_ring_holes::{
    crash_a_mount_before_its_row_root, devices_of, parameters_of_the_ring_hole_pools, pool_of,
    pool_overwritten_through, SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, instance_table_chain_of_root,
    readable_roots_with_ring_slots, root_is_abandoned_by_the_instance_table,
};
use singlefs_core::root_record::RootRecord;
use singlefs_harness::memory_pool::MemoryPool;

/// 实例 1 覆盖写到的 txg（攻方 `C = 24`）。
pub const LAST_TXG_OF_INSTANCE_ONE: u64 = 24;

/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
pub const INVARIANTS_THE_MOUNT_KEEPS_GREEN: [&str; 5] =
    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];

/// 实例 1 覆盖写到 txg 24，再连着 `crashes` 次可写挂载崩在写行那次根槽写之前。
pub fn pool_after_crashes_before_the_row_root(crashes: usize) -> MemoryPool {
    let mut image = pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE);
    // 迭代上界：`crashes` 次挂载，每次一个洞；跨轮携带的是崩溃之后的盘面。
    for _ in 0..crashes {
        let (crashed, _row_txg) = crash_a_mount_before_its_row_root(&image);
        image = crashed;
    }
    image
}

/// 盘面上读得出、按 `newest_root` 指着的实例表仍然有效的每一条根。
pub fn valid_roots_of_the_ring(image: &MemoryPool, newest_root: &RootRecord) -> Vec<RootRecord> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = &system_configuration.immutable;
    let table = instance_table_chain_of_root(image, newest_root)
        .expect("现行那一版的实例表")
        .records;
    readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|(_, root)| root)
    .filter(|root| !root_is_abandoned_by_the_instance_table(root, &table))
    .collect()
}

/// 环里每条有效根的账里仍分配、而分配器上空着（可再分配）的槽：(根的 txg, 盘, 槽)。
pub fn slots_of_valid_ring_roots_left_free(
    image: &MemoryPool,
    allocator: &PoolAllocator,
    roots: &[RootRecord],
) -> Vec<(u64, u32, u64)> {
    let mut left_free = Vec::new();
    for root in roots {
        let records = allocation_records_under_root(image, root).expect("环里有效根的账读得出");
        for record in records.iter().filter(|record| !record.is_released) {
            let device_map = allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == record.device)
                .expect("账里的盘在分配器里");
            for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
                if device_map.is_free(singlefs_core::address::SlotNumber(slot)) {
                    left_free.push((root.checkpoint_txg.0, record.device.0, slot));
                }
            }
        }
    }
    left_free
}

/// 场景：连崩 11 次之后可写挂载做完。
/// 预期：环里实例 1 留在洞里的根有落在一圈窗口外的（前提），挂载之后的分配器上，环里每一条有效根的账里仍分配的槽一个都不空闲；
/// 挂载之后盘上那几条不变量不红。改之前（回收门槛取一圈窗口）窗口外那几条根引用的单元当场回收、空了出来。
pub fn eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated(
    images: &mut PoolCheckerImages,
) {
    let image = pool_after_crashes_before_the_row_root(11);
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(&image);
    let Mounted {
        allocator, current, ..
    } = mount_writable(&parameters, &mut devices).expect("连崩之后那一次可写挂载");
    let after = pool_of(&devices);
    let roots = valid_roots_of_the_ring(&after, current.root());
    let newest_txg = roots
        .iter()
        .map(|root| root.checkpoint_txg.0)
        .max()
        .expect("环里有根");
    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
    let roots_outside_the_window: Vec<u64> = roots
        .iter()
        .map(|root| root.checkpoint_txg.0)
        .filter(|txg| txg + ring_length <= newest_txg)
        .collect();
    assert!(
        !roots_outside_the_window.is_empty(),
        "前提：洞里留着的实例 1 的根有落在一圈窗口外的（最新根 txg {newest_txg}）"
    );
    assert_eq!(
        slots_of_valid_ring_roots_left_free(&after, &allocator, &roots),
        Vec::<(u64, u32, u64)>::new(),
        "环里有效根（窗口外的 {roots_outside_the_window:?} 也算）引用的单元一个都不回收；每盘已分配 {:?}",
        allocator
            .devices
            .iter()
            .map(|device_map| device_map.allocated_slots())
            .collect::<Vec<u64>>()
    );
    images.hand_out(
        "连崩 11 次之后挂载做完",
        || after.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_MOUNT_KEEPS_GREEN),
    );
}

/// 场景：连崩 1 次之后可写挂载做完（攻方 RGalloc 臂 I-3.1 误红的那一形：`results/b1-RGalloc-pre0.log` 的 `k=1 j=0`）。
/// 预期：挂载之后池级 checker 判那几条不变量不红——分配器的门槛与 checker 的候选过滤都取整个环，窗口外那几条根照当候选走、它们引用的单元照算已分配。
pub fn one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring(
    images: &mut PoolCheckerImages,
) {
    let image = pool_after_crashes_before_the_row_root(1);
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(&image);
    mount_writable(&parameters, &mut devices).expect("崩过一次之后那一次可写挂载");
    images.hand_out(
        "连崩 1 次之后挂载做完",
        || pool_of(&devices),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_MOUNT_KEEPS_GREEN),
    );
}
