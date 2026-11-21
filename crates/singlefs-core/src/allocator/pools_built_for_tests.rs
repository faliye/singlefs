//! 几组测试共用的池：mkfs 之后的池、下一个提交内生块要回退的池、两块盘上的分配记录与照记录重建的池。

use super::*;
use singlefs_format::UNIT_AREA_START_SLOT;

pub(super) fn pool_after_mkfs() -> PoolAllocator {
    let device_bytes = 4u64 << 30;
    let mut pool = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
        DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
    ]);
    pool.mark_format_time_units(
        Placement {
            slot: SlotNumber(50176),
            span: 2,
        },
        Placement {
            slot: SlotNumber(50178),
            span: 1,
        },
    );
    pool
}

/// 两块盘上各一份同样的记录。
pub(super) fn records_on_both_devices(
    placements: &[(u64, u16, u64, bool)],
) -> Vec<AllocationRecord> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .iter()
        .flat_map(|device| {
            placements.iter().map(
                |(slot, span_slots, generation, is_released)| AllocationRecord {
                    device: *device,
                    slot: SlotNumber(*slot),
                    span_slots: *span_slots,
                    generation: CheckpointTxg(*generation),
                    is_released: *is_released,
                    quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
                },
            )
        })
        .collect()
}

pub(super) fn rebuilt_pool(records: Vec<AllocationRecord>) -> PoolAllocator {
    let device_bytes = 4u64 << 30;
    PoolAllocator::rebuild_from_records(
        vec![
            DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
            DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
        ],
        records,
    )
}

/// 每块盘上把 `[from, to)` 里还空着的槽都占掉（只动空闲图，不写分配记录：造形态用）。
pub(super) fn fill_free_slots(pool: &mut PoolAllocator, from: u64, to: u64) {
    for device_map in &mut pool.devices {
        for slot in from..to {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 开段 [50240, 50304) 的第一槽已发出、段里其余槽占满、别的全空段各占掉段首一槽 ⇒ 一个全空段都没有、开放段装不下，
/// 下一个提交内生块回落。段首之外各片分配记录树叶全空：叶 62（[50344, 51156)）罩 13 个段首、空 799 槽，
/// 叶 63（[51156, 51968)）罩 12 个段首、空 800 槽，是单元区里空槽最多、叶序号最小的那片。
pub(super) fn pool_after_mkfs_whose_next_commit_generated_block_falls_back() -> PoolAllocator {
    let mut pool = pool_after_mkfs();
    let opening = pool
        .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
        .expect("开段");
    assert_eq!(opening.slot, SlotNumber(50240));
    fill_free_slots(&mut pool, 50241, 50304);
    for device_map in &mut pool.devices {
        let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
        for segment_index in 0..full_segments {
            if device_map.segment_is_empty(usize::try_from(segment_index).expect("段号")) {
                device_map.mark_allocated(
                    SlotNumber(UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS),
                    1,
                );
            }
        }
    }
    pool
}
