//! m2-step45-code-r3 云端攻方腿的模型：T3 扣住位（X8）与 T1 隔离位（X1）在分配器三条发路径上的行为。
//! 只用 `singlefs-core` 的公开 API，不改被判代码；跑法在报告开头。
//! 每块盘的单元区做成 128 槽（2 个聚簇段）：`50176 + 128 = 50304`，`50304 * 16384 = 824180736` 字节。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{
    AllocationRecord, DeviceFreeMap, PoolAllocator, ReclaimedReuse, UnitFootprint,
};

const UNIT_AREA_START_SLOT: u64 = 50176;
const SEGMENT_SLOTS: u64 = 64;
const TWO_SEGMENT_DEVICE_BYTES: u64 = (UNIT_AREA_START_SLOT + 2 * SEGMENT_SLOTS) * 16384;

fn two_segment_device_maps() -> Vec<DeviceFreeMap> {
    vec![
        DeviceFreeMap::new(DeviceIdentity(0), TWO_SEGMENT_DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), TWO_SEGMENT_DEVICE_BYTES),
    ]
}

/// 每块盘上一条单槽记录。
fn one_slot_record(
    device_number: u32,
    slot_number: u64,
    generation: u64,
    is_released: bool,
) -> AllocationRecord {
    AllocationRecord {
        device: DeviceIdentity(device_number),
        slot: SlotNumber(slot_number),
        span_slots: 1,
        generation: CheckpointTxg(generation),
        is_released,
    }
}

/// 段 0 全部仍分配，段 1 全部已释放、释放代 5：抬 F 到 5 时段 1 整段会被回收。
fn records_with_second_segment_released() -> Vec<AllocationRecord> {
    let mut records = Vec::new();
    for device_number in 0..2 {
        for offset_in_segment in 0..SEGMENT_SLOTS {
            records.push(one_slot_record(
                device_number,
                UNIT_AREA_START_SLOT + offset_in_segment,
                3,
                false,
            ));
            records.push(one_slot_record(
                device_number,
                UNIT_AREA_START_SLOT + SEGMENT_SLOTS + offset_in_segment,
                5,
                true,
            ));
        }
    }
    records
}

/// X8-1：抬 F 回收整段之后扣住，池里唯一的全空段被扣住位挡掉 ⇒ `allocate_commit_generated` 给 None
/// （`publish_version` 会把它变成 `PublishError::NoSpaceFor`，`raise_rollback_floor` 第 493 行的 `?` 直接返回，
/// 第 501 行的 `release_reclaim_holds()` 走不到），而这时空闲计数已经加过 64。
/// 同一段历史换成 `ReclaimedReuse::Immediately` 分配得到 50240：分不出别的因素，只差扣住位。
#[test]
fn holding_the_reclaimed_segment_makes_the_next_commit_generated_allocation_fail_while_the_free_count_says_there_is_room(
) {
    let mut held_pool =
        PoolAllocator::rebuild_from_records(two_segment_device_maps(), records_with_second_segment_released());
    assert_eq!(held_pool.devices[0].free_slots(), 0, "回收之前一个空闲槽都没有");
    let reclaimed = held_pool
        .reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::HeldUntilFloorTakesEffect);
    assert_eq!(reclaimed.len(), 64, "段 1 整段回收（按盘 0 报）");
    assert_eq!(
        held_pool.devices[0].free_slots(),
        64,
        "记账的空闲已经加过：抬 F 的回收在写第一条带新 F 的根之前"
    );
    assert_eq!(
        held_pool.devices[0].lowest_empty_segment(),
        None,
        "段 1 用量为 0 却被扣住位挡掉，池里没有可开的全空段"
    );
    assert_eq!(
        held_pool.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(6)),
        None,
        "带新 F 的空发布分配不到固定点"
    );

    let mut immediate_pool =
        PoolAllocator::rebuild_from_records(two_segment_device_maps(), records_with_second_segment_released());
    immediate_pool.reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::Immediately);
    assert_eq!(
        immediate_pool
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(6))
            .map(|placement| placement.slot),
        Some(SlotNumber(50240)),
        "不扣住就发得出去：两条路只差 ReclaimedReuse"
    );
}

/// X8-2：扣住位与隔离位都只在 `is_free`、`lowest_user_data_slot`、`lowest_empty_segment` 上起作用，
/// 提交内生块的 bump 路（`allocate_commit_generated` 的开放段内那一段与 `PoolAllocator::record`）一个字都不看。
/// 开放段是在开段那一刻查的三个条件，开段之后再置位的槽照发不误。
#[test]
fn the_bump_path_hands_out_a_slot_that_is_free_map_says_is_not_free_when_it_is_isolated_after_the_segment_was_opened(
) {
    let mut pool = PoolAllocator::new(two_segment_device_maps());
    let first = pool
        .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
        .expect("段 0 全空，开得了");
    assert_eq!(first.slot, SlotNumber(50176), "开放段从单元区开头开");

    let isolated_slot = SlotNumber(50180);
    for device_number in 0..2 {
        pool.isolate_abandoned(DeviceIdentity(device_number), isolated_slot, 1);
    }
    for device_map in &pool.devices {
        assert!(
            !device_map.is_free(isolated_slot),
            "隔离之后 is_free 说它不空闲"
        );
    }

    let mut handed_out = Vec::new();
    for _ in 0..4 {
        handed_out.push(
            pool.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
                .expect("开放段里还有位置")
                .slot,
        );
    }
    assert_eq!(
        handed_out,
        vec![
            SlotNumber(50177),
            SlotNumber(50178),
            SlotNumber(50179),
            SlotNumber(50180)
        ],
        "bump 游标一路推过去，隔离位拦不住它"
    );
    assert!(
        handed_out.contains(&isolated_slot),
        "被隔离的 50180 照样发了出去"
    );
}

/// X1：豁免集与隔离集都按分配记录的**起点槽号**做 key（`mount.rs` 第 221-238 行与第 246-255 行），
/// 而 `isolate` 按整条记录的跨度铺位。一条被抛弃根的双槽记录，只要起点槽号在豁免集里就整条跳过，
/// 跨度里的第二槽没有任何东西罩着。这里把 `mount.rs` 那两段的集合算术原样重写一遍（不改被判代码）。
#[test]
fn a_two_slot_record_of_an_abandoned_root_is_skipped_whole_when_its_start_slot_is_exempt_so_the_second_slot_stays_free(
) {
    // 候选根那一版账：50190 是一个单槽索引节点。
    let candidate_records = vec![one_slot_record(0, 50190, 4, false)];
    // 被抛弃根那一版账：50190 起的双槽单元（同一个起点，跨度不同）。
    let abandoned_records = vec![AllocationRecord {
        device: DeviceIdentity(0),
        slot: SlotNumber(50190),
        span_slots: 2,
        generation: CheckpointTxg(6),
        is_released: false,
    }];

    let exempt: std::collections::BTreeSet<(u32, u64)> = candidate_records
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    let mut isolated_spans = Vec::new();
    for record in &abandoned_records {
        let key = (record.device.0, record.slot.0);
        if record.is_released || exempt.contains(&key) {
            continue;
        }
        isolated_spans.push((record.slot.0, u64::from(record.span_slots)));
    }
    assert!(
        isolated_spans.is_empty(),
        "起点 50190 在豁免集里 ⇒ 整条双槽记录跳过"
    );
    let covered_by_candidate: std::collections::BTreeSet<u64> = candidate_records
        .iter()
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect();
    assert!(
        !covered_by_candidate.contains(&50191),
        "候选根的单槽记录罩不到 50191，而被抛弃根的双槽单元占着它"
    );
}

/// X8-3：扣住位只有 `release_reclaim_holds`（`mount.rs` 第 501 行）会清，而它排在发布循环之后；
/// 循环里第 493 行的 `?` 一走，扣住位就留在这个进程的分配器里。这一格量的是「留着会怎样」：
/// 后续每一次发布都分配不到固定点，而记账仍旧报着 64 个空闲槽；手工放开之后同一条分配立刻成功。
#[test]
fn once_the_holds_are_left_set_every_later_allocation_keeps_failing_until_release_reclaim_holds_is_called() {
    let mut pool =
        PoolAllocator::rebuild_from_records(two_segment_device_maps(), records_with_second_segment_released());
    pool.reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::HeldUntilFloorTakesEffect);
    for attempt in 0..3 {
        assert_eq!(
            pool.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(6 + attempt)),
            None,
            "第 {attempt} 次重试照样发不出去"
        );
        assert_eq!(
            pool.devices[0].free_slots(),
            64,
            "记账一直说有 64 个空闲槽"
        );
    }
    pool.release_reclaim_holds();
    assert_eq!(
        pool.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(9))
            .map(|placement| placement.slot),
        Some(SlotNumber(50240)),
        "放开之后同一条分配立刻成功：拦住它的只有扣住位"
    );
}
