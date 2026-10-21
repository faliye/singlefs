//! 正推腿探针（T7）：分配器的 `empty_segments` 增量维护对着「全扫记录重算」。
//! 不经任何池：两块 1 GiB 的盘，随机走分配（用户数据 / 提交内生块两档）、释放、回收、回退复活，每一步之后两块盘各比一次。
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator, ReclaimedReuse, UnitFootprint};
use singlefs_format::CLUSTER_SEGMENT_SLOTS;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(n).unwrap()).unwrap()
    }
}

/// 全扫记录：该盘单元区里凑满 64 槽的整段中，没有任何一条未释放记录与它相交的段数（一趟扫记录，按段标记）。
fn empty_segments_by_scanning_the_records(allocator: &PoolAllocator, device_position: usize) -> u64 {
    let map = &allocator.devices[device_position];
    let start = map.unit_area_start().slot().0;
    let full_segments = usize::try_from(map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS).unwrap();
    let mut touched = vec![false; full_segments];
    for record in allocator.records() {
        if record.device != map.device || record.is_released {
            continue;
        }
        let first = record.slot.0 - start;
        let last = first + u64::from(record.span_slots) - 1;
        for slot_offset in [first, last] {
            let segment = usize::try_from(slot_offset / CLUSTER_SEGMENT_SLOTS).unwrap();
            if segment < full_segments {
                touched[segment] = true;
            }
        }
    }
    u64::try_from(touched.iter().filter(|segment| !**segment).count()).unwrap()
}

#[test]
fn probe_forward_empty_segments_incremental_equals_full_scan() {
    let device_bytes = 1u64 << 30;
    let mut checked = 0u64;
    let mut rebuilds_checked = 0u64;
    let mut max_deferred_only_difference = 0u64;
    for seed in 1..=4u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
            DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
        ]);
        let mut live: Vec<Placement> = Vec::new();
        let mut released: Vec<(Placement, CheckpointTxg)> = Vec::new();
        let mut txg = 10u64;
        for step in 0..4000 {
            txg += 1;
            match rng.below(10) {
                0..=2 => {
                    if let Ok(placement) = allocator.try_allocate_user_data(CheckpointTxg(txg)) {
                        live.push(placement);
                    }
                }
                3..=4 => {
                    let footprint = if rng.below(2) == 0 {
                        UnitFootprint::OneSlot
                    } else {
                        UnitFootprint::TwoSlotsAligned
                    };
                    if let Ok(placement) =
                        allocator.try_allocate_commit_generated(footprint, CheckpointTxg(txg))
                    {
                        live.push(placement);
                    }
                }
                5..=7 => {
                    if !live.is_empty() {
                        let placement = live.swap_remove(rng.below(live.len()));
                        allocator.release(placement, CheckpointTxg(txg));
                        released.push((placement, CheckpointTxg(txg)));
                    }
                }
                8 => {
                    let floor = CheckpointTxg(txg.saturating_sub(u64::try_from(rng.below(40)).unwrap()));
                    allocator.reclaim_released_up_to(floor, ReclaimedReuse::Immediately);
                }
                _ => {
                    // 回退复活：挑一条已释放的（还在 defer 里，或已回收没被复用），两盘一起复活。
                    if !released.is_empty() {
                        let position = rng.below(released.len());
                        let (placement, _) = released[position];
                        let still_released_on_both = [DeviceIdentity(0), DeviceIdentity(1)].iter().all(|device| {
                            allocator
                                .record_for(*device, placement.slot)
                                .is_some_and(|record| {
                                    record.is_released && u64::from(record.span_slots) == placement.span
                                })
                        });
                        if still_released_on_both {
                            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                                allocator.resurrect_released_record(device, placement, CheckpointTxg(txg));
                            }
                            released.swap_remove(position);
                            live.push(placement);
                        } else {
                            released.swap_remove(position);
                        }
                    }
                }
            }
            for device_position in 0..2 {
                let incremental = allocator.devices[device_position].empty_segments();
                let scanned = empty_segments_by_scanning_the_records(&allocator, device_position);
                assert_eq!(
                    incremental, scanned,
                    "seed {seed} 第 {step} 步之后盘 {device_position}：增量 {incremental}，全扫 {scanned}"
                );
                checked += 1;
                let by_occupancy = (0..(allocator.devices[device_position].unit_area_slots() / CLUSTER_SEGMENT_SLOTS))
                    .filter(|segment| allocator.devices[device_position].segment_is_empty(usize::try_from(*segment).unwrap()))
                    .count() as u64;
                max_deferred_only_difference = max_deferred_only_difference.max(incremental.saturating_sub(by_occupancy));
            }
            let _ = SlotNumber(0);
            if step % 500 == 499 {
                // 挂载重建：同一批记录交给 rebuild_from_records，两盘各比一次。
                let rebuilt = PoolAllocator::rebuild_from_records(
                    vec![
                        DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
                        DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
                    ],
                    allocator.records().to_vec(),
                );
                for device_position in 0..2 {
                    assert_eq!(
                        rebuilt.devices[device_position].empty_segments(),
                        empty_segments_by_scanning_the_records(&rebuilt, device_position),
                        "seed {seed} 第 {step} 步：重建出来的分配器盘 {device_position}"
                    );
                    rebuilds_checked += 1;
                }
            }
        }
    }
    println!(
        "name=probe_forward_empty_segments checked_comparisons={checked} rebuilds_checked={rebuilds_checked} max_segments_empty_by_records_but_occupied_by_deferred_slots={max_deferred_only_difference}"
    );
}
