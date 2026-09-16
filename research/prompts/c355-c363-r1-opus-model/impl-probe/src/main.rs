//! C355 / C363 第一轮攻方腿：拿 `crates/singlefs-core` 今天的分配器直接跑三格（只读调用公开接口，不改 crates/）。
//! 格一：两盘等量、只剩 F 个槽时，按臂放行一个数据单元，接着按第一个事务的形状分配 7 个提交内生块，数分到几个。
//! 格二：两盘不等大（D2（RAID 条带策略） 已定项 2「各盘不必等大」），池级求和的臂放行之后提交内生块分配在小盘上 panic。
//! 格三：同一不等大池，用户数据落点分配在小盘上 panic。

use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator, UnitFootprint};
use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};

const FIRST_TRANSACTION_SHAPE: [UnitFootprint; 7] = [
    UnitFootprint::OneSlot,
    UnitFootprint::TwoSlotsAligned,
    UnitFootprint::OneSlot,
    UnitFootprint::OneSlot,
    UnitFootprint::OneSlot,
    UnitFootprint::OneSlot,
    UnitFootprint::OneSlot,
];

fn pool(unit_area_slots: [u64; 2]) -> PoolAllocator {
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), (UNIT_AREA_START_SLOT + unit_area_slots[0]) * SLOT_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), (UNIT_AREA_START_SLOT + unit_area_slots[1]) * SLOT_BYTES),
    ]);
    allocator.mark_format_time_units(&[
        Placement { slot: SlotNumber(UNIT_AREA_START_SLOT), span: 2 },
        Placement { slot: SlotNumber(UNIT_AREA_START_SLOT + 2), span: 1 },
    ]);
    allocator
}

fn free_slots(allocator: &PoolAllocator) -> [u64; 2] {
    [allocator.devices[0].free_slots(), allocator.devices[1].free_slots()]
}

fn main() {
    // 格一：单元区 256 槽（4 个聚簇段）两盘等量；提交内生块 bump 吃掉段 1..3，剩段 0 里 mkfs 之后的 61 个空槽、没有全空段。
    // 任何一条臂按槽数都放行（61 ≫ c + 2）；接着按第一个事务的形状分配 7 个提交内生块，数分到几个。
    let mut allocator = pool([256, 256]);
    while allocator.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3)).is_some() {}
    let before = free_slots(&allocator);
    let empty_segments_before = allocator.devices[0].empty_segments();
    let data = allocator.allocate_user_data(CheckpointTxg(4));
    let mut commit_generated_done = 0;
    for footprint in FIRST_TRANSACTION_SHAPE {
        if allocator.allocate_commit_generated(footprint, CheckpointTxg(4)).is_some() {
            commit_generated_done += 1;
        } else {
            break;
        }
    }
    println!(
        "IMPLRESULT name=equal_devices_no_empty_segment free_before={}/{} empty_segments_before={empty_segments_before} data_unit_placed={} commit_generated_placed={commit_generated_done}/7 free_after={}/{}",
        before[0], before[1], data.is_some(), allocator.devices[0].free_slots(), allocator.devices[1].free_slots()
    );

    // 格二：盘 0 单元区 256 槽、盘 1 192 槽。提交内生块从盘 0 的最低全空段取，落点两盘同号。
    let mut allocator = pool([256, 192]);
    let mut placed = 0u64;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        loop {
            let free = free_slots(&allocator);
            let summed = free[0] + free[1];
            if placed % 64 == 0 || free[1] < 8 {
                println!("IMPLRESULT name=unequal_commit_generated_step placed={placed} free={}/{} summed={summed} empty_segments_device0={}", free[0], free[1], allocator.devices[0].empty_segments());
            }
            if allocator.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3)).is_none() {
                return "returned_none";
            }
            placed += 1;
        }
    }));
    let free = free_slots(&allocator);
    println!(
        "IMPLRESULT name=unequal_commit_generated outcome={} placed={placed} free_at_stop={}/{}",
        match outcome { Ok(text) => text.to_string(), Err(_) => "panicked".to_string() },
        free[0], free[1]
    );

    // 格三：同一几何，用户数据落点。
    let mut allocator = pool([256, 192]);
    let mut placed_units = 0u64;
    let outcome = catch_unwind(AssertUnwindSafe(|| loop {
        if allocator.allocate_user_data(CheckpointTxg(3)).is_none() {
            return "returned_none";
        }
        placed_units += 1;
    }));
    let free = free_slots(&allocator);
    println!(
        "IMPLRESULT name=unequal_user_data outcome={} placed_units={placed_units} free_at_stop={}/{}",
        match outcome { Ok(text) => text.to_string(), Err(_) => "panicked".to_string() },
        free[0], free[1]
    );
    println!("IMPLRESULT name=done");
}
