//! 草稿探针（腿自己的副本，不入库）：抬 F 回收的槽被扣住的那段里，式子读数与落点读数各说什么。
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::admission::{
    AdmissionReading, BytesOnOneDevice, BytesSummedOverAllReplicas, PlacementRoomOnOneDevice, PoolWideCommitments,
};
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, ReclaimedReuse};

fn pool() -> PoolAllocator {
    PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), 4 << 30),
        DeviceFreeMap::new(DeviceIdentity(1), 4 << 30),
    ])
}
fn line(tag: &str, p: &PoolAllocator) {
    let reading = AdmissionReading::of_allocator(
        p,
        BytesOnOneDevice(0),
        PoolWideCommitments::of_the_first_version(BytesSummedOverAllReplicas(0), BytesSummedOverAllReplicas(0)),
        BytesOnOneDevice(0),
    );
    let available = reading.available_on_each_device();
    let room = PlacementRoomOnOneDevice::of_every_device_of_the_allocator(p);
    println!(
        "name=a3-probe stage={tag} d0_allocated_slots={} d0_free_slots={} d0_available_bytes={} d0_unblocked_slots={} d0_pairs_for_data={}",
        p.devices[0].allocated_slots(),
        p.devices[0].free_slots(),
        available[0].1 .0,
        room[0].slots_for_commit_generated_units,
        room[0].slot_pairs_for_data_units
    );
}
#[test]
fn probe() {
    let mut p = pool();
    line("fresh", &p);
    let a = p.allocate_user_data(CheckpointTxg(1)).expect("a");
    let b = p.allocate_user_data(CheckpointTxg(1)).expect("b");
    line("two_data_units_allocated", &p);
    p.release(a, CheckpointTxg(2));
    p.release(b, CheckpointTxg(2));
    line("both_released_in_defer_window", &p);
    let reclaimed = p.reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::HeldUntilFloorTakesEffect);
    println!("reclaimed_placements={}", reclaimed.len());
    line("reclaimed_but_held_until_floor_takes_effect", &p);
    p.release_reclaim_holds();
    line("holds_released", &p);
}

fn gap(tag: &str, p: &PoolAllocator) {
    let reading = AdmissionReading::of_allocator(
        p,
        BytesOnOneDevice(0),
        PoolWideCommitments::of_the_first_version(BytesSummedOverAllReplicas(0), BytesSummedOverAllReplicas(0)),
        BytesOnOneDevice(0),
    );
    let available_slots = reading.available_on_each_device()[0].1 .0 / 16384;
    let unblocked = i128::from(p.devices[0].unblocked_slots());
    println!(
        "name=a2a3-identity stage={tag} reading_available_slots={available_slots} allocator_issuable_unblocked_slots={unblocked} reading_minus_issuable={}",
        available_slots - unblocked
    );
}
#[test]
fn identity_between_the_reading_and_what_the_allocator_can_issue() {
    use singlefs_core::address::SlotNumber;
    let mut p = pool();
    gap("fresh", &p);
    let a = p.allocate_user_data(CheckpointTxg(1)).expect("a");
    let b = p.allocate_user_data(CheckpointTxg(1)).expect("b");
    gap("two_data_units_allocated", &p);
    // 隔离两个空槽（只被被抛弃根引用、现行账里空闲）：第九项扣 2，分配器也发不出这 2 槽
    p.isolate_abandoned(DeviceIdentity(0), SlotNumber(60_000), 2);
    gap("isolate_two_free_slots", &p);
    // 隔离已分配的两槽（被抛弃根引用的槽在现行账里仍分配 ⇒ 已分配与第九项各扣一次）
    p.isolate_abandoned(DeviceIdentity(0), a.slot, a.span);
    gap("isolate_two_allocated_slots_double_deducted", &p);
    // 回收但扣住
    p.release(b, CheckpointTxg(2));
    let _ = p.reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::HeldUntilFloorTakesEffect);
    gap("two_slots_reclaimed_and_held", &p);
    p.release_reclaim_holds();
    gap("holds_released", &p);
}

fn closed(tag: &str, p: &PoolAllocator) {
    let d = &p.devices[0];
    let outside = d.unblocked_slot_pairs_outside_the_cluster_segments(p.cluster_segments());
    let room = PlacementRoomOnOneDevice::of_every_device_of_the_allocator(p);
    let total_pairs_by_scan_of_unblocked_slots = d.unblocked_slots() / 2;
    println!(
        "name=a4-closed-pairs stage={tag} cluster_segments={} unblocked_slots={} pairs_for_data_outside_segments={} pairs_for_data_per_room={} slots_unblocked_inside_cluster_segments_upper_bound={}",
        p.cluster_segments().len(),
        d.unblocked_slots(),
        outside,
        room[0].slot_pairs_for_data_units,
        d.unblocked_slots() - 2 * outside - (d.unblocked_slots() % 2) * 0
    );
    let _ = total_pairs_by_scan_of_unblocked_slots;
}
#[test]
fn slots_closed_to_user_data_inside_the_mounts_cluster_segments_grow_with_churn() {
    let mut p = pool();
    closed("fresh", &p);
    let mut placements = Vec::new();
    for _ in 0..300 {
        placements.push(p.allocate_commit_generated(singlefs_core::allocator::UnitFootprint::OneSlot, CheckpointTxg(1)).expect("slot"));
    }
    closed("300_one_slot_commit_generated_blocks", &p);
    for pl in placements.iter().step_by(1) {
        p.release(*pl, CheckpointTxg(2));
    }
    let n = p.reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::Immediately).len();
    println!("reclaimed={n}");
    closed("all_300_released_and_reclaimed_inside_closed_segments", &p);
    // 用户数据落点此刻仍不进这些段：下一个数据单元落在哪
    let user = p.allocate_user_data(CheckpointTxg(3)).expect("user");
    println!("name=a4-closed-pairs next_user_data_slot={} (单元区起点 50176)", user.slot.0);
}

#[test]
fn the_landing_check_counts_single_slots_while_a_two_slot_container_needs_an_aligned_pair() {
    use singlefs_core::address::SlotNumber;
    use singlefs_core::admission::{admit_the_units_landing_on_every_device, UnitsOfAPublishToLand};
    use singlefs_core::allocator::UnitFootprint;
    let mut p = pool();
    // 每块盘上把单元区里每个奇数槽标成已分配：没挡的槽剩一半，但一对相邻的空槽都没有。
    for device_map in &mut p.devices {
        let start = device_map.unit_area_start().slot().0;
        let slots = device_map.unit_area_slots();
        for offset in (1..slots).step_by(2) {
            device_map.mark_allocated(SlotNumber(start + offset), 1);
        }
    }
    let room = PlacementRoomOnOneDevice::of_every_device_of_the_allocator(&p);
    println!(
        "name=a5-c574-first-half d0_unblocked_slots={} d0_pairs_for_data={}",
        room[0].slots_for_commit_generated_units, room[0].slot_pairs_for_data_units
    );
    let verdict = admit_the_units_landing_on_every_device(
        &room,
        UnitsOfAPublishToLand { data_units: 0, commit_generated_slots: 2 },
    );
    println!("name=a5-c574-first-half pre_check_for_one_two_slot_container={:?}", verdict.is_ok());
    let landing_one = p.try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(1));
    let landing_two = p.try_allocate_commit_generated(UnitFootprint::TwoSlotsAligned, CheckpointTxg(1));
    println!("name=a5-c574-first-half landing_one_slot_ok={} landing_two_slots_aligned={:?}", landing_one.is_ok(), landing_two);
}

#[test]
fn the_byte_formula_admits_a_data_unit_while_no_slot_pair_outside_the_cluster_segments_is_left() {
    use singlefs_core::address::SlotNumber;
    use singlefs_core::admission::{
        admit_on_every_device, admit_the_units_landing_on_every_device, DemandOnDevice, UnitsOfAPublishToLand,
    };
    use singlefs_core::allocator::UnitFootprint;
    let mut p = pool();
    let mut placed = Vec::new();
    for _ in 0..300 {
        placed.push(p.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(1)).expect("slot"));
    }
    for pl in &placed {
        p.release(*pl, CheckpointTxg(2));
    }
    let _ = p.reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::Immediately);
    let segments: Vec<u64> = p.cluster_segments().iter().map(|s| s.0).collect();
    for device_map in &mut p.devices {
        let start = device_map.unit_area_start().slot().0;
        let slots = device_map.unit_area_slots();
        for slot in start..start + slots {
            let inside = segments.iter().any(|s| slot >= *s && slot < *s + 64);
            if !inside && device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    let reading = AdmissionReading::of_allocator(
        &p,
        BytesOnOneDevice(0),
        PoolWideCommitments::of_the_first_version(BytesSummedOverAllReplicas(0), BytesSummedOverAllReplicas(0)),
        BytesOnOneDevice(0),
    );
    let available_slots = reading.available_on_each_device()[0].1 .0 / 16384;
    let demand: Vec<DemandOnDevice> = p
        .devices
        .iter()
        .map(|d| DemandOnDevice { device: d.device, bytes: BytesOnOneDevice(2 * 16384) })
        .collect();
    let formula = admit_on_every_device(&reading, &demand);
    let room = PlacementRoomOnOneDevice::of_every_device_of_the_allocator(&p);
    let landing = admit_the_units_landing_on_every_device(&room, UnitsOfAPublishToLand { data_units: 1, commit_generated_slots: 0 });
    println!(
        "name=a4-witness d0_formula_available_slots={available_slots} formula_admits_one_data_unit={} d0_pairs_for_data_outside_segments={} landing_check_admits_one_data_unit={} user_data_alloc={:?}",
        formula.is_ok(), room[0].slot_pairs_for_data_units, landing.is_ok(), p.allocate_user_data(CheckpointTxg(3)).map(|x| x.slot.0)
    );
}
