use super::*;
use singlefs_format::UNIT_AREA_START_SLOT;

#[test]
fn unit_area_of_a_4_gib_image_has_211968_slots_in_3312_segments() {
    let map = DeviceFreeMap::new(DeviceIdentity(0), 4 << 30);
    assert_eq!(map.unit_area_slots(), 211_968);
    assert_eq!(map.empty_segments(), 3312);
    assert_eq!(map.free_runs(), 1);
}

/// [`DeviceFreeMap::slots_counted_both_as_allocated_and_as_isolated`] 的阳性对照
/// 一块盘上分配三段（起点槽起两槽、第 4 槽起一槽、第 8 槽起两槽），
/// 再用不看分配位的整段 [`DeviceFreeMap::isolate`] 罩住第 0 到第 5 槽：第 0、1、4 槽同时置着分配位与隔离位，第 2、3、5 槽只置隔离位，
/// 第 8、9 槽只置分配位。访问器交 3，隔离数 6、已分配数 5 各自对得上。
#[test]
fn isolating_a_whole_span_over_allocated_slots_counts_each_overlapping_slot_both_as_allocated_and_as_isolated(
) {
    let device_bytes = (UNIT_AREA_START_SLOT + 385) * SLOT_BYTES;
    let mut map = DeviceFreeMap::new(DeviceIdentity(0), device_bytes);
    let first_slot = map.unit_area_start().slot();
    let slot_at = |offset_in_slots: u64| SlotNumber(first_slot.0 + offset_in_slots);
    map.mark_allocated(slot_at(0), 2);
    map.mark_allocated(slot_at(4), 1);
    map.mark_allocated(slot_at(8), 2);
    assert_eq!(
        map.slots_counted_both_as_allocated_and_as_isolated(),
        0,
        "隔离之前没有一个槽置着隔离位"
    );
    map.isolate(slot_at(0), 6);
    assert_eq!(
        map.slots_counted_both_as_allocated_and_as_isolated(),
        3,
        "整段隔离罩住的第 0、1、4 槽同时置着分配位与隔离位"
    );
    assert_eq!(
        map.isolated_slots(),
        3 + 3,
        "隔离着的槽：同时置两位的 3 个加只置隔离位的第 2、3、5 槽"
    );
    assert_eq!(
        map.allocated_slots(),
        3 + 2,
        "已分配的槽：同时置两位的 3 个加没被罩住的第 8、9 槽"
    );
}

/// 逐槽扫出来的「没挡的槽数」与「这次挂载开过的聚簇段之外、两槽都没挡的偶数起点槽对数」：只用公开的 `is_free`
/// （与 `lowest_user_data_slot` 同一个谓词）与聚簇段集合，不读增量维护的那几个计数。
fn unblocked_slots_and_pairs_outside_the_cluster_segments_by_scan(
    pool: &PoolAllocator,
    device_map: &DeviceFreeMap,
) -> (u64, SlotPairCount) {
    let first_slot = device_map.unit_area_start().slot().0;
    let past_the_last_slot = first_slot + device_map.unit_area_slots();
    let unblocked_slots = u64::try_from(
        (first_slot..past_the_last_slot)
            .filter(|slot| device_map.is_free(SlotNumber(*slot)))
            .count(),
    )
    .expect("槽数装得进 u64");
    let pairs_outside = u64::try_from(
        (first_slot..past_the_last_slot)
            .step_by(2)
            .filter(|pair_start| {
                pair_start + 1 < past_the_last_slot
                    && device_map.is_free(SlotNumber(*pair_start))
                    && device_map.is_free(SlotNumber(pair_start + 1))
                    && !pool.cluster_segments().iter().any(|segment_start| {
                        *pair_start >= segment_start.0
                            && *pair_start < segment_start.0 + CLUSTER_SEGMENT_SLOTS
                    })
            })
            .count(),
    )
    .map(SlotPairCount)
    .expect("槽对数装得进 u64");
    (unblocked_slots, pairs_outside)
}

/// 发布路径准入先判「这次的单元落得下」读的两个数（C545（空间准入罩不住分裂与聚簇段层））——
/// 提交内生块落得下的槽数、用户数据落得下的段外槽对数——是增量维护的，这里每一步之后拿逐槽扫出来的同一个量对，两块盘都要对得上。
/// 三种挡位的置与清各走一遍：分配（开段、bump、用户数据）、回收、抬 F 扣住与放开、影子账隔离（只在盘 0）与清隔离；
/// 单元区 385 槽（末尾一槽不成对、末尾一段不满 64 槽）。
/// 判别力：放开扣住时只清位图、不跟着维护（如 `fill(false)`），放开之后两个数都少算放开的那几槽。
#[test]
fn unblocked_slot_and_slot_pair_counts_are_maintained_incrementally_and_match_a_scan() {
    let device_bytes = (UNIT_AREA_START_SLOT + 385) * SLOT_BYTES;
    let mut pool = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
        DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
    ]);
    let check = |pool_after_the_step: &PoolAllocator, step: &str| {
        for device_map in &pool_after_the_step.devices {
            assert_eq!(
                (
                    device_map.unblocked_slots(),
                    device_map.unblocked_slot_pairs_outside_the_cluster_segments(
                        pool_after_the_step.cluster_segments()
                    )
                ),
                unblocked_slots_and_pairs_outside_the_cluster_segments_by_scan(
                    pool_after_the_step,
                    device_map
                ),
                "{step}：盘 {:?} 的（没挡的槽数，段外没挡的槽对数）与逐槽扫的相同",
                device_map.device
            );
        }
    };
    check(&pool, "空盘");
    assert_eq!(
        pool.devices[0].unblocked_slot_pairs_outside_the_cluster_segments(&BTreeSet::new()),
        SlotPairCount(192),
        "385 槽：192 对，末尾一槽不成对"
    );
    let mut placements = Vec::new();
    for _ in 0..3 {
        placements.push(
            pool.allocate_user_data(CheckpointTxg(1))
                .expect("空盘上分得到数据单元"),
        );
    }
    for footprint in [
        UnitFootprint::OneSlot,
        UnitFootprint::TwoSlotsAligned,
        UnitFootprint::OneSlot,
    ] {
        placements.push(
            pool.allocate_commit_generated(footprint, CheckpointTxg(1))
                .expect("开段、bump"),
        );
    }
    check(&pool, "分配三个数据单元、开一段 bump 三个提交内生块");
    for placement in &placements[..4] {
        pool.release(*placement, CheckpointTxg(2));
    }
    pool.reclaim_released_up_to(CheckpointTxg(2), ReclaimedReuse::Immediately);
    check(&pool, "释放四个、回收");
    for _ in 0..2 {
        let placement = pool
            .allocate_user_data(CheckpointTxg(3))
            .expect("回收之后分得到");
        pool.release(placement, CheckpointTxg(4));
    }
    pool.reclaim_released_up_to(CheckpointTxg(4), ReclaimedReuse::HeldUntilFloorTakesEffect);
    check(&pool, "回收的两个扣住到 F 生效");
    pool.isolate_abandoned(DeviceIdentity(0), SlotNumber(UNIT_AREA_START_SLOT + 301), 2);
    pool.isolate_abandoned(DeviceIdentity(0), SlotNumber(UNIT_AREA_START_SLOT + 384), 1);
    check(&pool, "盘 0 隔离一对跨两对的槽与末尾不成对的那一槽");
    pool.devices[0].clear_isolation_of_slot(SlotNumber(UNIT_AREA_START_SLOT + 302));
    check(&pool, "盘 0 清一槽的隔离");
    let unblocked_before_releasing_the_holds = pool.devices[1].unblocked_slots();
    pool.release_reclaim_holds();
    check(&pool, "放开扣住");
    assert_eq!(
        pool.devices[1].unblocked_slots(),
        unblocked_before_releasing_the_holds + 4,
        "放开扣住的两个数据单元（4 槽）"
    );
}

/// 按记录逐槽扫出来的「defer、影子账隔离、抬 F 扣住三样全放开之后段外成的槽对数」：一槽罩在未释放的分配记录下就放不开，
/// 别的（空闲的、罩在已释放记录下的、隔离或扣住而没有未释放记录罩着的）都放得开；只读记录表、跨度与聚簇段集合，
/// 不读增量维护的逐槽位与计数。
fn slot_pairs_once_everything_is_released_by_the_records(
    pool: &PoolAllocator,
    device_map: &DeviceFreeMap,
) -> SlotPairCount {
    let first_slot = device_map.unit_area_start().slot().0;
    let past_the_last_slot = first_slot + device_map.unit_area_slots();
    let is_under_an_unreleased_record_by_the_records = |slot: u64| {
        pool.records().iter().any(|record| {
            record.device == device_map.device
                && !record.is_released
                && slot >= record.slot.0
                && slot < record.slot.0 + u64::from(record.span_slots)
        })
    };
    u64::try_from(
        (first_slot..past_the_last_slot)
            .step_by(2)
            .filter(|pair_start| {
                pair_start + 1 < past_the_last_slot
                    && !is_under_an_unreleased_record_by_the_records(*pair_start)
                    && !is_under_an_unreleased_record_by_the_records(pair_start + 1)
                    && !pool.cluster_segments().iter().any(|segment_start| {
                        *pair_start >= segment_start.0
                            && *pair_start < segment_start.0 + CLUSTER_SEGMENT_SLOTS
                    })
            })
            .count(),
    )
    .map(SlotPairCount)
    .expect("槽对数装得进 u64")
}

/// 真把三样都放开之后段外没挡的槽对数：在一份拷贝上回收全部已释放的、清掉每一槽的影子账隔离、放开抬 F 扣住，再读没挡的段外槽对。
fn slot_pairs_after_really_releasing_everything(
    pool: &PoolAllocator,
    device: DeviceIdentity,
) -> SlotPairCount {
    let mut released = pool.clone();
    released.reclaim_released_up_to(CheckpointTxg(u64::MAX), ReclaimedReuse::Immediately);
    released.release_reclaim_holds();
    let cluster_segments = released.cluster_segments().clone();
    let device_map = released
        .devices
        .iter_mut()
        .find(|device_map| device_map.device == device)
        .expect("盘在池里");
    let first_slot = device_map.unit_area_start().slot().0;
    for slot in first_slot..first_slot + device_map.unit_area_slots() {
        device_map.clear_isolation_of_slot(SlotNumber(slot));
    }
    device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
}

/// 一串定种子的伪随机分配、开段、释放、回收、抬 F 扣住与放开、影子账隔离与清隔离、管理员回退复活（两块盘，单元区 385 槽：
/// 末尾一槽不成对、末尾一段不满 64 槽），每一步之后把分配器交给 `check_after_the_step`（步号与这一步做了什么）。
fn random_history_of_the_allocator(check_after_the_step: &dyn Fn(&PoolAllocator, usize, &str)) {
    let device_bytes = (UNIT_AREA_START_SLOT + 385) * SLOT_BYTES;
    let mut pool = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
        DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
    ]);
    let mut random_state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next_random = |below: u64| {
        random_state ^= random_state << 13;
        random_state ^= random_state >> 7;
        random_state ^= random_state << 17;
        random_state % below
    };
    let mut generation = 1u64;
    let mut placements_allocated: Vec<Placement> = Vec::new();
    let mut placements_in_the_defer_queue: Vec<(Placement, CheckpointTxg)> = Vec::new();
    check_after_the_step(&pool, 0, "空盘");
    for step_number in 1..=600 {
        let what_the_step_did = match next_random(9) {
            0 | 1 => {
                if let Some(placement) = pool.allocate_user_data(CheckpointTxg(generation)) {
                    placements_allocated.push(placement);
                }
                "分配一个数据单元"
            }
            2 => {
                let footprint = if next_random(2) == 0 {
                    UnitFootprint::OneSlot
                } else {
                    UnitFootprint::TwoSlotsAligned
                };
                if let Some(placement) =
                    pool.allocate_commit_generated(footprint, CheckpointTxg(generation))
                {
                    placements_allocated.push(placement);
                }
                "开段或 bump 一个提交内生块"
            }
            3 | 4 => {
                if !placements_allocated.is_empty() {
                    let position = usize::try_from(next_random(
                        u64::try_from(placements_allocated.len()).expect("个数"),
                    ))
                    .expect("下标");
                    let placement = placements_allocated.swap_remove(position);
                    generation += 1;
                    pool.release(placement, CheckpointTxg(generation));
                    placements_in_the_defer_queue.push((placement, CheckpointTxg(generation)));
                }
                "释放一个落点进 defer"
            }
            5 => {
                let reuse = if next_random(2) == 0 {
                    ReclaimedReuse::Immediately
                } else {
                    ReclaimedReuse::HeldUntilFloorTakesEffect
                };
                pool.reclaim_released_up_to(CheckpointTxg(generation), reuse);
                placements_in_the_defer_queue.clear();
                "回收 defer 里的全部（立即可用或抬 F 扣住）"
            }
            6 => {
                pool.release_reclaim_holds();
                "抬 F 生效、放开扣住"
            }
            7 => {
                let slot = SlotNumber(UNIT_AREA_START_SLOT + next_random(384));
                if next_random(2) == 0 {
                    pool.isolate_abandoned(DeviceIdentity(0), slot, 1 + next_random(2));
                } else {
                    pool.devices[0].clear_isolation_of_slot(slot);
                }
                "盘 0 隔离或清隔离"
            }
            _ => {
                if let Some((placement, _)) = placements_in_the_defer_queue.pop() {
                    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                        pool.resurrect_released_record(
                            device,
                            placement,
                            CheckpointTxg(generation),
                        );
                    }
                    placements_allocated.push(placement);
                }
                "管理员回退复活一个 defer 里的落点"
            }
        };
        check_after_the_step(&pool, step_number, what_the_step_did);
    }
}

/// 抬 F 之前先算槽对那一判的上界读的数（D16（发布语义） 已定项 10 射程「抬 F 之前先算」）是增量维护的：
/// 一串随机操作的每一步之后，拿按记录逐槽扫出来的同一个量对，两块盘都要对得上。
/// 判别力：释放时不清那一槽的「罩在未释放记录下」位，defer 里的槽就少算。
/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
#[test]
fn slot_pairs_once_everything_is_released_are_maintained_incrementally_and_match_a_scan_of_the_records(
) {
    random_history_of_the_allocator(&|pool, step_number, what_the_step_did| {
        for device_map in &pool.devices {
            assert_eq!(
                device_map.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
                    pool.cluster_segments()
                ),
                slot_pairs_once_everything_is_released_by_the_records(pool, device_map),
                "第 {step_number} 步（{what_the_step_did}）：盘 {:?} 的增量数与按记录逐槽扫的相同",
                device_map.device
            );
        }
    });
}

/// 上界不许算小（算小了会误拒一个推了本来写得进的写）：一串随机操作的每一步之后，增量维护的数不小于在拷贝上真把 defer、隔离、扣住都放开之后
/// 段外没挡的槽对数。
/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
#[test]
fn slot_pairs_once_everything_is_released_never_fall_below_the_pairs_after_really_releasing_everything(
) {
    random_history_of_the_allocator(&|pool, step_number, what_the_step_did| {
        for device_map in &pool.devices {
            let upper_bound = device_map
                .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
                    pool.cluster_segments(),
                );
            let really_released =
                slot_pairs_after_really_releasing_everything(pool, device_map.device);
            assert!(
                upper_bound >= really_released,
                "第 {step_number} 步（{what_the_step_did}）：盘 {:?} 上界 {upper_bound:?} 小于真放开之后的 {really_released:?}",
                device_map.device
            );
        }
    });
}

/// 重开时从记录重建分配器（[`PoolAllocator::rebuild_from_records`]）之后，抬 F 之前先算槽对那一判读的三样——逐槽「罩在未释放记录下」位、
/// 按段的槽对数、全盘的槽对数——与重建之前逐项相同：三样只随记录未释放与否变，重建照记录走 `mark_allocated` 与 `mark_released`。
/// 一串随机分配、释放、回收、扣住、隔离与复活的每一步之后各重建一次比，两块盘都比。
/// 判别力：重建时已释放的两槽落点不进 defer（少走一处 `mark_released`），那两槽重建之后仍罩在未释放记录下，三样都对不上。
/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
#[test]
fn slot_pairs_once_everything_is_released_are_the_same_after_rebuilding_the_allocator_from_its_records(
) {
    random_history_of_the_allocator(&|pool, step_number, what_the_step_did| {
        let rebuilt = PoolAllocator::rebuild_from_records(
            pool.devices
                .iter()
                .map(|device_map| {
                    DeviceFreeMap::new(device_map.device, device_map.device_size_in_bytes)
                })
                .collect(),
            pool.records().to_vec(),
        );
        for (device_map, rebuilt_device_map) in pool.devices.iter().zip(&rebuilt.devices) {
            assert_eq!(
                (
                    &rebuilt_device_map.is_under_an_unreleased_record,
                    &rebuilt_device_map.slot_pairs_without_an_unreleased_record_per_segment,
                    rebuilt_device_map.slot_pairs_without_an_unreleased_record,
                ),
                (
                    &device_map.is_under_an_unreleased_record,
                    &device_map.slot_pairs_without_an_unreleased_record_per_segment,
                    device_map.slot_pairs_without_an_unreleased_record,
                ),
                "第 {step_number} 步（{what_the_step_did}）：盘 {:?} 重建之后的（逐槽位、按段槽对数、全盘槽对数）与重建之前相同",
                device_map.device
            );
        }
    });
}

#[test]
fn accounting_deltas_are_maintained_incrementally() {
    let mut pool = pool_after_mkfs();
    pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
    assert_eq!(pool.devices[0].allocated_slots(), 5, "mkfs 3 + t1 2");
    assert_eq!(pool.devices[0].free_slots(), 211_968 - 5);
    assert_eq!(pool.devices[0].free_runs(), 2, "[50179] 与 [50182, 末]");
    assert_eq!(pool.devices[0].empty_segments(), 3311);
    for footprint in [
        UnitFootprint::OneSlot,
        UnitFootprint::TwoSlotsAligned,
        UnitFootprint::OneSlot,
        UnitFootprint::OneSlot,
        UnitFootprint::OneSlot,
        UnitFootprint::OneSlot,
        UnitFootprint::OneSlot,
    ] {
        pool.allocate_commit_generated(footprint, CheckpointTxg(3))
            .expect("开放段");
    }
    assert_eq!(pool.devices[0].allocated_slots(), 13, "字节表五：13 个槽");
    assert_eq!(pool.devices[0].free_slots() * SLOT_BYTES, 3_472_670_720);
    assert_eq!(
        pool.devices[0].free_runs(),
        4,
        "[50179]、[50182, 50239]、[50241]、[50249, 末]"
    );
    assert_eq!(pool.devices[0].empty_segments(), 3310);
    assert_eq!(pool.devices[1].free_runs(), 4, "两盘同构");
}

/// 全空聚簇段数按记录口径（D3（空间分配） 已定项 10 ①、D26（后台整理与放置回收） 已定项 1 第 1 条，C372（全空聚簇段数两份定义相反））：
/// 一段里只剩已释放、还在 defer 窗口里的槽时它算全空，而那几槽照旧占着（[`DeviceFreeMap::segment_is_empty`] 不收它）；
/// 复活把它减回去，回收不动这个数。单元区末尾凑不满 64 槽的那一截不是聚簇段：开头不数它，往里分配也不动这个数。
#[test]
fn segment_left_with_only_deferred_slots_counts_as_empty_by_the_records_while_its_slots_stay_occupied(
) {
    let full_segments = 3u64;
    let slots_past_the_last_full_segment = 10u64;
    let device_bytes = (UNIT_AREA_START_SLOT
        + full_segments * CLUSTER_SEGMENT_SLOTS
        + slots_past_the_last_full_segment)
        * SLOT_BYTES;
    let mut map = DeviceFreeMap::new(DeviceIdentity(0), device_bytes);
    assert_eq!(
        map.empty_segments(),
        full_segments,
        "末尾那 10 槽凑不满一段，不数"
    );
    let segment_index = 1usize;
    let slot_in_the_segment =
        SlotNumber(map.unit_area_start().slot().0 + CLUSTER_SEGMENT_SLOTS + 4);
    map.mark_allocated(slot_in_the_segment, 2);
    assert_eq!(
        map.empty_segments(),
        full_segments - 1,
        "分配进第 1 段：那一段罩着未释放的记录"
    );
    map.mark_released(slot_in_the_segment, 2);
    assert_eq!(
        map.empty_segments(),
        full_segments,
        "释放之后第 1 段只剩 defer 槽：按记录口径全空"
    );
    assert!(
        !map.segment_is_empty(segment_index),
        "defer 槽仍占着：按占用第 1 段不空"
    );
    map.mark_resurrected_out_of_the_defer_queue(slot_in_the_segment, 2);
    assert_eq!(
        map.empty_segments(),
        full_segments - 1,
        "复活之后那条记录又是未释放的"
    );
    map.mark_released(slot_in_the_segment, 2);
    map.mark_reclaimed(slot_in_the_segment, 2);
    assert_eq!(
        map.empty_segments(),
        full_segments,
        "回收的是已释放的记录，不动记录口径的数"
    );
    assert!(map.segment_is_empty(segment_index), "回收之后按占用也空了");
    map.mark_allocated(
        SlotNumber(map.unit_area_start().slot().0 + full_segments * CLUSTER_SEGMENT_SLOTS),
        2,
    );
    assert_eq!(
        map.empty_segments(),
        full_segments,
        "分配进末尾那一截不动全空聚簇段数"
    );
}

/// 管理员回退的复活（D23（journal 的角色与格式） 已定项 14「复活」）：还在 defer 里的已释放记录改回已分配、分配代写回给的那一个，
/// defer 槽数减掉它、已分配槽数不动（槽本来就占着）；已回收、还没被复用的那一条改回已分配时位图重新标占、空闲槽数减掉它。
#[test]
fn resurrecting_a_released_record_moves_it_out_of_the_defer_queue_or_back_onto_the_bitmap() {
    let mut pool = rebuilt_pool(records_on_both_devices(&[
        (50_176, 2, 5, true),
        (50_178, 2, 6, true),
    ]));
    pool.reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::Immediately);
    let device = DeviceIdentity(0);
    let map_of = |allocator: &PoolAllocator| {
        let map = &allocator.devices[0];
        (
            map.allocated_slots(),
            map.deferred_slots(),
            map.free_slots(),
        )
    };
    let (allocated_before, deferred_before, free_before) = map_of(&pool);
    assert_eq!(
        deferred_before, 2,
        "释放代 6 的那一条还在 defer 里，释放代 5 的回收了"
    );
    pool.resurrect_released_record(
        device,
        Placement {
            slot: SlotNumber(50_178),
            span: 2,
        },
        CheckpointTxg(3),
    );
    assert_eq!(
        map_of(&pool),
        (allocated_before, 0, free_before),
        "defer 里那一条复活：defer 减 2，已分配与空闲不动"
    );
    pool.resurrect_released_record(
        device,
        Placement {
            slot: SlotNumber(50_176),
            span: 2,
        },
        CheckpointTxg(2),
    );
    assert_eq!(
        map_of(&pool),
        (allocated_before + 2, 0, free_before - 2),
        "回收过的那一条复活：位图重新标占"
    );
    assert!(!pool.devices[0].is_free(SlotNumber(50_176)));
    assert_eq!(
        records_on_device(&pool, device),
        vec![(50_176, 2, 2, false), (50_178, 2, 3, false)],
        "两条都改回已分配、分配代写回给的那一个"
    );
    assert_eq!(
        records_on_device(&pool, DeviceIdentity(1)),
        vec![(50_176, 2, 5, true), (50_178, 2, 6, true)],
        "按盘改：另一块盘上的记录不动"
    );
}

/// 某块盘上的记录按槽号排好：(槽, 跨度, 代, 已释放)。
fn records_on_device(pool: &PoolAllocator, device: DeviceIdentity) -> Vec<(u64, u16, u64, bool)> {
    let mut on_device: Vec<(u64, u16, u64, bool)> = pool
        .records()
        .iter()
        .filter(|record| record.device == device)
        .map(|record| {
            (
                record.slot.0,
                record.span_slots,
                record.generation.0,
                record.is_released,
            )
        })
        .collect();
    on_device.sort_unstable();
    on_device
}

/// 两槽数据单元复用已回收记录的形状：用户数据（两槽）落在一对都已回收的单槽记录上——50178 那条改写成这次的分配、跨度 2，
/// 50179 那条罩住的槽被新记录罩住，随这次分配删掉；之后按这份记录重建分配器（下一次可写挂载做的事）不 panic、占着的槽数对得上。
#[test]
fn reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_rewrites_the_first_and_deletes_the_second(
) {
    let mut pool = rebuilt_pool(records_on_both_devices(&[
        (50176, 2, 0, false),
        (50178, 1, 3, true),
        (50179, 1, 4, true),
    ]));
    assert_eq!(
        pool.reclaim_released_up_to(CheckpointTxg(4), ReclaimedReuse::Immediately),
        vec![
            Placement {
                slot: SlotNumber(50178),
                span: 1
            },
            Placement {
                slot: SlotNumber(50179),
                span: 1
            },
        ]
    );
    let data = pool
        .try_allocate_user_data(CheckpointTxg(5))
        .expect("有空槽");
    assert_eq!(
        data,
        Placement {
            slot: SlotNumber(50178),
            span: 2
        },
        "最低的偶数空槽对是两条已回收记录罩住的 50178–50179"
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            records_on_device(&pool, device),
            vec![(50176, 2, 0, false), (50178, 2, 5, false)],
            "盘 {device:?}：50178 那条改写成跨 2、代 5，50179 那条删掉"
        );
        assert!(
            !pool.reclaimed.contains(&(device, SlotNumber(50179))),
            "删掉的记录也不再算已回收"
        );
    }
    let rebuilt = rebuilt_pool(pool.records().to_vec());
    assert_eq!(
        rebuilt.devices[0].allocated_slots(),
        4,
        "50176–50179 四个槽"
    );
    assert_eq!(rebuilt.devices[0].deferred_slots(), 0);
}

/// 反过来的形状：一个单槽落点落在一条已回收的两槽记录的第二个槽上——那条记录起点不是这个槽，罩住的槽被新落点罩住，删掉；
/// 它罩住的另一个槽（50180）之后没有记录，与从没分配过的槽一样空着。
#[test]
fn one_slot_placement_inside_reclaimed_two_slot_record_deletes_that_record() {
    let mut pool = rebuilt_pool(records_on_both_devices(&[
        (50176, 2, 0, false),
        (50178, 1, 0, false),
        (50180, 2, 3, true),
    ]));
    pool.reclaim_released_up_to(CheckpointTxg(3), ReclaimedReuse::Immediately);
    pool.record(
        Placement {
            slot: SlotNumber(50181),
            span: 1,
        },
        CheckpointTxg(4),
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            records_on_device(&pool, device),
            vec![
                (50176, 2, 0, false),
                (50178, 1, 0, false),
                (50181, 1, 4, false)
            ],
            "盘 {device:?}：50180 那条删掉、50181 追加一条"
        );
    }
    assert!(pool.devices[0].is_free(SlotNumber(50180)), "50180 仍空着");
    let rebuilt = rebuilt_pool(pool.records().to_vec());
    assert_eq!(
        rebuilt.devices[0].allocated_slots(),
        4,
        "50176–50178 与 50181"
    );
}

/// 新落点罩住一条没回收的记录（已释放、还在 defer 窗口里）：位图与记录分叉了，在记录这一层就断言，不等到位图的断言。
/// 用 `catch_unwind` 接住、再按消息断言：位图那一层的断言同样会 panic，要钉住拦下它的是记录这一层那一条。
#[test]
fn placement_over_record_that_was_not_reclaimed_is_asserted_on_the_records() {
    let mut pool = rebuilt_pool(records_on_both_devices(&[
        (50176, 2, 0, false),
        (50178, 1, 0, false),
        (50180, 2, 3, true),
    ]));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pool.record(
            Placement {
                slot: SlotNumber(50181),
                span: 1,
            },
            CheckpointTxg(4),
        );
    }));
    let payload = outcome.expect_err("新落点罩住一条没回收的记录，要断言");
    let message = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_default();
    assert!(
        message.contains("不是已回收的"),
        "断言在记录这一层（不是位图那一条）：{message}"
    );
}

/// 每片分配记录树叶的没挡槽数随三种挡位增量维护，与逐槽数出来的相等：分配、隔离、扣住、回收各走一遍，单元区首尾那两片叶只数单元区里的一截。
#[test]
fn unblocked_slots_per_allocation_record_tree_leaf_are_maintained_incrementally_and_match_a_scan() {
    fn unblocked_slots_in_the_leaf_by_scan(
        device_map: &DeviceFreeMap,
        leaf_index: AllocationRecordTreeLeafIndex,
    ) -> SlotCountInALeaf {
        let first = leaf_index.first_slot().0.max(UNIT_AREA_START_SLOT);
        let past = leaf_index
            .past_the_last_slot()
            .0
            .min(UNIT_AREA_START_SLOT + device_map.unit_area_slots());
        let unblocked = (first..past)
            .filter(|slot| !device_map.is_blocked_for_commit_generated(SlotNumber(*slot)))
            .count();
        SlotCountInALeaf(
            u64::try_from(unblocked).expect("数的是一片叶里的槽，至多 812 个，装得进 u64"),
        )
    }
    let mut pool = pool_after_mkfs_whose_next_commit_generated_block_falls_back();
    pool.isolate_abandoned(DeviceIdentity(1), SlotNumber(51_160), 3);
    let fell_back = pool
        .allocate_commit_generated(UnitFootprint::TwoSlotsAligned, CheckpointTxg(4))
        .expect("回落");
    pool.devices[0].hold_until_floor_takes_effect(SlotNumber(51_170), 2);
    pool.release(fell_back, CheckpointTxg(5));
    assert_eq!(
        pool.reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::Immediately),
        vec![fell_back]
    );
    for device_map in &pool.devices {
        let (first_leaf, last_leaf) = device_map
            .allocation_record_tree_leaves_of_the_unit_area()
            .expect("单元区不空");
        assert_eq!(
            (first_leaf, last_leaf),
            (
                AllocationRecordTreeLeafIndex(61),
                AllocationRecordTreeLeafIndex(322)
            )
        );
        for leaf_number in first_leaf.0..=last_leaf.0 {
            let leaf_index = AllocationRecordTreeLeafIndex(leaf_number);
            assert_eq!(
                device_map.unblocked_slots_in_allocation_record_tree_leaf(leaf_index),
                unblocked_slots_in_the_leaf_by_scan(device_map, leaf_index),
                "盘 {:?} 叶 {leaf_number}",
                device_map.device
            );
        }
        assert_eq!(
            device_map.unblocked_slots_in_allocation_record_tree_leaf(
                AllocationRecordTreeLeafIndex(first_leaf.0 - 1)
            ),
            SlotCountInALeaf(0)
        );
        assert_eq!(
            device_map.unblocked_slots_in_allocation_record_tree_leaf(
                AllocationRecordTreeLeafIndex(last_leaf.0 + 1)
            ),
            SlotCountInALeaf(0)
        );
    }
}

/// 对拍用户数据落点用的一串带种子的伪随机操作（两块盘，单元区各 `unit_area_slots_per_device` 槽，可以不等大）：分配数据单元、
/// 开段 / bump / 回落一个提交内生块、释放、回收（立即可用或抬 F 扣住）、放开扣住、直接扣住空槽、影子账隔离与清隔离（两块盘各自）、
/// 签发与撤掉活整理意图的 `R`、克隆分配器接着在拷贝上走，各走到。每一步之后把分配器交给 `check_after_the_step`（步号与这一步做了什么）。
fn random_history_for_the_user_data_placement(
    seed: u64,
    unit_area_slots_per_device: [u64; 2],
    steps: usize,
    check_after_the_step: &dyn Fn(&PoolAllocator, usize, &str),
) {
    let mut pool = PoolAllocator::new(vec![
        DeviceFreeMap::new(
            DeviceIdentity(0),
            (UNIT_AREA_START_SLOT + unit_area_slots_per_device[0]) * SLOT_BYTES,
        ),
        DeviceFreeMap::new(
            DeviceIdentity(1),
            (UNIT_AREA_START_SLOT + unit_area_slots_per_device[1]) * SLOT_BYTES,
        ),
    ]);
    let mut random_state = seed;
    let mut next_random = |below: u64| {
        random_state ^= random_state << 13;
        random_state ^= random_state >> 7;
        random_state ^= random_state << 17;
        random_state % below
    };
    let mut generation = 1u64;
    let mut placements_allocated: Vec<Placement> = Vec::new();
    check_after_the_step(&pool, 0, "空盘");
    for step_number in 1..=steps {
        let what_the_step_did = match next_random(12) {
            0 | 1 => {
                if let Some(placement) = pool.allocate_user_data(CheckpointTxg(generation)) {
                    placements_allocated.push(placement);
                }
                "分配一个数据单元"
            }
            2 | 3 => {
                let footprint = if next_random(2) == 0 {
                    UnitFootprint::OneSlot
                } else {
                    UnitFootprint::TwoSlotsAligned
                };
                if let Some(placement) =
                    pool.allocate_commit_generated(footprint, CheckpointTxg(generation))
                {
                    placements_allocated.push(placement);
                }
                "开段、bump 或回落一个提交内生块"
            }
            4 | 5 => {
                if !placements_allocated.is_empty() {
                    let position = usize::try_from(next_random(
                        u64::try_from(placements_allocated.len()).expect("个数"),
                    ))
                    .expect("下标");
                    let placement = placements_allocated.swap_remove(position);
                    generation += 1;
                    pool.release(placement, CheckpointTxg(generation));
                }
                "释放一个落点进 defer"
            }
            6 => {
                let reuse = if next_random(2) == 0 {
                    ReclaimedReuse::Immediately
                } else {
                    ReclaimedReuse::HeldUntilFloorTakesEffect
                };
                pool.reclaim_released_up_to(CheckpointTxg(generation), reuse);
                "回收 defer 里的全部（立即可用或抬 F 扣住）"
            }
            7 => {
                pool.release_reclaim_holds();
                "抬 F 生效、放开扣住"
            }
            8 => {
                let device_position = usize::try_from(next_random(2)).expect("盘的下标");
                let device_map = &mut pool.devices[device_position];
                let slot = SlotNumber(
                    UNIT_AREA_START_SLOT + next_random(device_map.unit_area_slots() - 1),
                );
                for slot_of_the_span in [slot, SlotNumber(slot.0 + 1)] {
                    if device_map.is_free(slot_of_the_span) {
                        device_map.hold_until_floor_takes_effect(slot_of_the_span, 1);
                    }
                }
                "一块盘上直接扣住两个空槽"
            }
            9 => {
                let device = DeviceIdentity(u32::try_from(next_random(2)).expect("盘号"));
                let device_position = usize::try_from(device.0).expect("盘的下标");
                let unit_area_slots = pool.devices[device_position].unit_area_slots();
                let slot = SlotNumber(UNIT_AREA_START_SLOT + next_random(unit_area_slots - 1));
                if next_random(2) == 0 {
                    pool.isolate_abandoned(device, slot, 1 + next_random(2));
                } else {
                    pool.devices[device_position].clear_isolation_of_slot(slot);
                }
                "一块盘上隔离或清隔离"
            }
            10 => {
                if pool.compaction().live_intent().is_some() {
                    pool.compaction_mut().withdraw_the_live_intent();
                    "撤掉活整理意图的 R"
                } else {
                    let smaller_unit_area =
                        unit_area_slots_per_device[0].min(unit_area_slots_per_device[1]);
                    let segment_index = next_random(smaller_unit_area / CLUSTER_SEGMENT_SLOTS);
                    pool.compaction_mut()
                        .issue(crate::logged_operations::CompactionIntent {
                            region: CompactionRegion::of_one_cluster_segment(
                                DeviceIdentity(0),
                                SlotNumber(
                                    UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS,
                                ),
                            )
                            .expect("单元区起点在段边界上，段起点是 64 的倍数"),
                            allocation_generation_at_issue: CheckpointTxg(generation),
                            empty_cluster_segments_at_issue: 0,
                        });
                    "签发活整理意图的 R"
                }
            }
            _ => {
                let cloned_allocator = pool.clone();
                pool = cloned_allocator;
                "克隆分配器，接着在拷贝上走"
            }
        };
        check_after_the_step(&pool, step_number, what_the_step_did);
    }
}

/// 每块盘上按段取的用户数据落点与从单元区起点逐个候选看的参照取法答得一样：带着这次挂载开过的聚簇段与此刻的 `R` 问一次，
/// 不带 `R` 再问一次。
fn assert_user_data_slot_by_segment_matches_stepping_through_every_candidate(
    seed: u64,
    pool: &PoolAllocator,
    step_number: usize,
    what_the_step_did: &str,
) {
    for device_map in &pool.devices {
        for excluded_region in [pool.region_excluded_from_placement(), None] {
            assert_eq!(
                device_map.lowest_user_data_slot_outside_the_region(
                    pool.cluster_segments(),
                    excluded_region
                ),
                device_map.lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate(
                    pool.cluster_segments(),
                    excluded_region
                ),
                "种子 {seed:#x} 第 {step_number} 步（{what_the_step_did}）：盘 {:?}、R {excluded_region:?}、聚簇段 {:?}",
                device_map.device,
                pool.cluster_segments()
            );
        }
    }
}

/// 两块等大的盘、单元区 385 槽（末尾一段只有一槽、不成对）：一串 1500 步的随机操作，每一步之后按段取与逐个候选看答得一样。
/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
#[test]
fn user_data_slot_found_by_segment_matches_stepping_through_every_candidate_on_two_equal_devices_ending_in_an_unpaired_slot(
) {
    let seed = 0x5EED_0000_0000_0385;
    println!("种子 {seed:#x}");
    random_history_for_the_user_data_placement(
        seed,
        [385, 385],
        1500,
        &|pool, step_number, what_the_step_did| {
            assert_user_data_slot_by_segment_matches_stepping_through_every_candidate(
                seed,
                pool,
                step_number,
                what_the_step_did,
            );
        },
    );
}

/// 两块不等大的盘（单元区 385 槽与 1000 槽）：大盘上小盘够不着的那几段只在大盘的集合里，按段取照样与逐个候选看答得一样。
/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
#[test]
fn user_data_slot_found_by_segment_matches_stepping_through_every_candidate_on_two_unequal_devices()
{
    let seed = 0x5EED_0000_0385_1000;
    println!("种子 {seed:#x}");
    random_history_for_the_user_data_placement(
        seed,
        [385, 1000],
        1500,
        &|pool, step_number, what_the_step_did| {
            assert_user_data_slot_by_segment_matches_stepping_through_every_candidate(
                seed,
                pool,
                step_number,
                what_the_step_did,
            );
        },
    );
}

/// 两块等大的盘、单元区 2000 槽（31 个整段加 16 槽）：段多、开过的聚簇段攒得多，一串 3000 步的随机操作，每一步之后按段取与逐个候选看答得一样。
/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
#[test]
fn user_data_slot_found_by_segment_matches_stepping_through_every_candidate_on_two_equal_devices_of_thirty_one_full_segments(
) {
    let seed = 0x5EED_0000_0000_2000;
    println!("种子 {seed:#x}");
    random_history_for_the_user_data_placement(
        seed,
        [2000, 2000],
        3000,
        &|pool, step_number, what_the_step_did| {
            assert_user_data_slot_by_segment_matches_stepping_through_every_candidate(
                seed,
                pool,
                step_number,
                what_the_step_did,
            );
        },
    );
}

/// 对拍的定形取样点（随机历史未必走到这一形）：一块盘三个整段全空，这次挂载开过的聚簇段是第 0、1 两段，
/// 段外最低的一对是第 2 段段首。按段取要把两段都跳过、与逐个候选看答得一样；只跳过集合里最后那一段的取法会落进第 0 段。
#[test]
fn user_data_slot_found_by_segment_skips_every_cluster_segment_of_this_mount_not_only_the_last() {
    let device_map = DeviceFreeMap::new(
        DeviceIdentity(0),
        (UNIT_AREA_START_SLOT + 3 * CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES,
    );
    let cluster_segments = BTreeSet::from([
        SlotNumber(UNIT_AREA_START_SLOT),
        SlotNumber(UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS),
    ]);
    let by_stepping_through_every_candidate = device_map
        .lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate(
            &cluster_segments,
            None,
        );
    assert_eq!(
        by_stepping_through_every_candidate,
        Some(SlotNumber(UNIT_AREA_START_SLOT + 2 * CLUSTER_SEGMENT_SLOTS)),
        "参照取法：两段聚簇段都跳过，落在第 2 段段首"
    );
    assert_eq!(
        device_map.lowest_user_data_slot_outside_the_region(&cluster_segments, None),
        by_stepping_through_every_candidate,
        "按段取与逐个候选看答得一样：这次挂载开过的两段聚簇段都要跳过，聚簇段 {cluster_segments:?}"
    );
}

/// 逐对扫出来的「两槽都空（`is_free`）的偶数起点槽对所在的段」：增量维护的 `segments_with_an_unblocked_slot_pair` 拿它对。
fn segments_with_an_unblocked_slot_pair_by_scan(device_map: &DeviceFreeMap) -> BTreeSet<usize> {
    let first_slot = device_map.unit_area_start().slot().0;
    let past_the_last_slot = first_slot + device_map.unit_area_slots();
    (first_slot..past_the_last_slot)
        .step_by(2)
        .filter(|pair_start| {
            pair_start + 1 < past_the_last_slot
                && device_map.is_free(SlotNumber(*pair_start))
                && device_map.is_free(SlotNumber(pair_start + 1))
        })
        .map(|pair_start| {
            usize::try_from((pair_start - first_slot) / CLUSTER_SEGMENT_SLOTS).expect("段号")
        })
        .collect()
}

/// 有没挡槽对的段号集合（`segments_with_an_unblocked_slot_pair`）是增量维护的，在一段的槽对数翻过 0 的两个方向上拿逐对扫出来的段集合对：
/// 一块盘三个整段全空，第 0 段前 31 对分配掉、最后一对扣住 ⇒ 第 0 段一对不剩，要离开集合；
/// 放开扣住 ⇒ 第 0 段的槽对数从 0 变回 1，要回到集合，用户数据落点落回第 0 段最后那一对。
/// 随机历史在 385 槽的盘上未必把哪一段占满，两个方向的转移靠这条定形取样点钉住。
/// 判别力：一段最后一对被挡上时不把那一段拿掉，挡上之后那一段多出来；槽对放开时不把那一段加回，放开之后那一段少了、落点跳到第 1 段。
/// 不并行：挡上、对扫、放开、对扫是同一块盘上按次序的四步，次序就是被测对象。
#[test]
fn segments_with_an_unblocked_slot_pair_leave_the_set_when_the_last_pair_is_blocked_and_return_when_it_is_released(
) {
    let mut device_map = DeviceFreeMap::new(
        DeviceIdentity(0),
        (UNIT_AREA_START_SLOT + 3 * CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES,
    );
    let last_pair_of_the_first_segment =
        SlotNumber(UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS - 2);
    device_map.mark_allocated(SlotNumber(UNIT_AREA_START_SLOT), CLUSTER_SEGMENT_SLOTS - 2);
    device_map.hold_until_floor_takes_effect(last_pair_of_the_first_segment, 2);
    assert_eq!(
        segments_with_an_unblocked_slot_pair_by_scan(&device_map),
        BTreeSet::from([1, 2]),
        "定形取样点：第 0 段 31 对分配、最后一对扣住，逐对扫只剩第 1、2 段"
    );
    assert_eq!(
        device_map.segments_with_an_unblocked_slot_pair,
        segments_with_an_unblocked_slot_pair_by_scan(&device_map),
        "第 0 段最后一对被挡上：那一段离开集合"
    );
    device_map.release_holds();
    assert_eq!(
        segments_with_an_unblocked_slot_pair_by_scan(&device_map),
        BTreeSet::from([0, 1, 2]),
        "定形取样点：放开扣住之后第 0 段又有一对"
    );
    assert_eq!(
        device_map.segments_with_an_unblocked_slot_pair,
        segments_with_an_unblocked_slot_pair_by_scan(&device_map),
        "第 0 段的槽对数从 0 变回 1：那一段回到集合"
    );
    assert_eq!(
        device_map.lowest_user_data_slot(&BTreeSet::new()),
        Some(last_pair_of_the_first_segment),
        "用户数据落点落回第 0 段放开的那一对"
    );
}

/// 有没挡槽对的段号集合（`segments_with_an_unblocked_slot_pair`）是增量维护的：两块 385 槽的盘上走一串随机操作，
/// 每一步之后两块盘的段集合都要与逐对扫出来的相同。
/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
#[test]
fn segments_with_an_unblocked_slot_pair_are_maintained_incrementally_and_match_a_scan_over_a_random_history(
) {
    let seed = 0x5EED_0000_0000_0385;
    println!("种子 {seed:#x}");
    random_history_for_the_user_data_placement(
        seed,
        [385, 385],
        1500,
        &|pool, step_number, what_the_step_did| {
            for device_map_in_the_pool in &pool.devices {
                assert_eq!(
                    device_map_in_the_pool.segments_with_an_unblocked_slot_pair,
                    segments_with_an_unblocked_slot_pair_by_scan(device_map_in_the_pool),
                    "种子 {seed:#x} 第 {step_number} 步（{what_the_step_did}）：盘 {:?}",
                    device_map_in_the_pool.device
                );
            }
        },
    );
}

/// 一块盘上按段取与逐个候选看，在「64 槽格子上任何一组聚簇段起点、任何一个 R」上都答得一样（产品路径交得出的只是这组输入的子集），
/// 增量维护的有槽对段集合每一步都与逐对扫的一样。单元区从 `unit_area_start` 起、`full_segments` 个整段加 `tail_slots` 槽。
/// 历史是一串定种子伪随机操作，直接改这块盘的三种挡位：每 500 步换一次相，填的那一相多分配、隔离、扣住（把段占满、槽对数落到 0），
/// 放的那一相多释放回收、清隔离、放开扣住（槽对数从 0 翻回正）。每一步之后，候选段起点（单元区下面那一段、单元区里每一段含末尾不满的那一段、
/// 单元区上面那一段）的每一个子集当聚簇段，没有 R 或任一候选段起点当 R，各问一次两种取法。
/// 交回「有槽对的段离开集合之后又回到集合」的次数，调用方断言它大于 0：取样点走到了集合翻过 0 的那一步。
fn placement_by_segment_against_stepping_through_every_candidate_over_every_aligned_cluster_set_and_region(
    seed: u64,
    unit_area_start: UnitAreaStart,
    full_segments: u64,
    tail_slots: u64,
    steps: usize,
) -> usize {
    let unit_area_slots = full_segments * CLUSTER_SEGMENT_SLOTS + tail_slots;
    let first_slot = unit_area_start.slot().0;
    let mut device_map = DeviceFreeMap::with_unit_area_start(
        DeviceIdentity(0),
        (first_slot + unit_area_slots) * SLOT_BYTES,
        unit_area_start,
    );
    let segments_touched = unit_area_slots.div_ceil(CLUSTER_SEGMENT_SLOTS);
    let candidate_segment_starts: Vec<SlotNumber> = (0..segments_touched + 2)
        .map(|position| {
            SlotNumber(first_slot - CLUSTER_SEGMENT_SLOTS + position * CLUSTER_SEGMENT_SLOTS)
        })
        .collect();
    let candidate_regions: Vec<Option<CompactionRegion>> = std::iter::once(None)
        .chain(candidate_segment_starts.iter().map(|segment_start| {
            Some(
                CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), *segment_start)
                    .expect("候选段起点是单元区起点加减 64 的倍数，单元区起点在段边界上"),
            )
        }))
        .collect();
    let mut random_state = seed;
    let mut next_random = |below: u64| {
        random_state ^= random_state << 13;
        random_state ^= random_state >> 7;
        random_state ^= random_state << 17;
        random_state % below
    };
    let mut slots_allocated: Vec<SlotNumber> = Vec::new();
    let mut segments_that_left_the_set: BTreeSet<usize> = BTreeSet::new();
    let mut returns_to_the_set = 0_usize;
    let mut set_before_the_step = device_map.segments_with_an_unblocked_slot_pair.clone();
    for step_number in 1..=steps {
        let is_filling = (step_number / 500) % 2 == 0;
        let slot = SlotNumber(first_slot + next_random(unit_area_slots));
        let operation = next_random(10);
        let what_the_step_did = match (is_filling, operation) {
            (true, 0..=6) | (false, 0) => {
                if device_map.is_free(slot) {
                    device_map.mark_allocated(slot, 1);
                    slots_allocated.push(slot);
                }
                "分配一槽"
            }
            (true, 7) | (false, 1..=5) => {
                if !slots_allocated.is_empty() {
                    let position = usize::try_from(next_random(
                        u64::try_from(slots_allocated.len()).expect("个数装得进 u64"),
                    ))
                    .expect("下标小于个数，装得进 usize");
                    let released = slots_allocated.swap_remove(position);
                    device_map.mark_released(released, 1);
                    device_map.mark_reclaimed(released, 1);
                }
                "释放并回收一槽"
            }
            (true, 8) => {
                device_map.isolate(slot, 1);
                "隔离一槽"
            }
            (false, 6..=7) => {
                device_map.clear_isolation_of_slot(slot);
                "清一槽的隔离"
            }
            (true, _) => {
                device_map.hold_until_floor_takes_effect(slot, 1);
                "扣住一槽"
            }
            (false, _) => {
                device_map.release_holds();
                "放开全部扣住"
            }
        };
        assert_eq!(
            device_map.segments_with_an_unblocked_slot_pair,
            segments_with_an_unblocked_slot_pair_by_scan(&device_map),
            "种子 {seed:#x} 第 {step_number} 步（{what_the_step_did}）：增量维护的段集合与逐对扫的不同"
        );
        for segment in
            set_before_the_step.difference(&device_map.segments_with_an_unblocked_slot_pair)
        {
            segments_that_left_the_set.insert(*segment);
        }
        returns_to_the_set += device_map
            .segments_with_an_unblocked_slot_pair
            .difference(&set_before_the_step)
            .filter(|segment| segments_that_left_the_set.contains(*segment))
            .count();
        set_before_the_step = device_map.segments_with_an_unblocked_slot_pair.clone();
        for subset in 0_u32..(1_u32 << candidate_segment_starts.len()) {
            let cluster_segments: BTreeSet<SlotNumber> = candidate_segment_starts
                .iter()
                .enumerate()
                .filter(|(position, _)| (subset & (1_u32 << position)) != 0)
                .map(|(_, segment_start)| *segment_start)
                .collect();
            for excluded_region in &candidate_regions {
                assert_eq!(
                    device_map
                        .lowest_user_data_slot_outside_the_region(&cluster_segments, *excluded_region),
                    device_map.lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate(
                        &cluster_segments,
                        *excluded_region
                    ),
                    "种子 {seed:#x} 第 {step_number} 步（{what_the_step_did}）：聚簇段 {cluster_segments:?}、R {excluded_region:?}"
                );
            }
        }
    }
    returns_to_the_set
}

/// 默认环的单元区起点、三个整段加末尾一槽（末尾那一段一对都没有）：2000 步，每一步在每一组格子上的聚簇段与每一个 R 上两种取法答得一样。
/// 不并行：每一步读上一步留下的空闲图，操作的次序就是被测对象。
#[test]
fn user_data_slot_found_by_segment_matches_on_every_aligned_cluster_set_and_region_ending_in_one_unpaired_slot(
) {
    let seed = 0x5EED_0000_00C0_0001;
    println!("种子 {seed:#x}");
    let returns_to_the_set =
        placement_by_segment_against_stepping_through_every_candidate_over_every_aligned_cluster_set_and_region(
            seed,
            UnitAreaStart::of_the_default_journal_ring(),
            3,
            1,
            2000,
        );
    assert!(
        returns_to_the_set > 0,
        "种子 {seed:#x}：没有一段离开集合又回来，取样点没走到集合翻过 0"
    );
}

/// 默认环的单元区起点、三个整段加末尾 63 槽（末尾那一段 31 对、最后一槽不成对）。
/// 不并行：每一步读上一步留下的空闲图，操作的次序就是被测对象。
#[test]
fn user_data_slot_found_by_segment_matches_on_every_aligned_cluster_set_and_region_ending_in_sixty_three_slots(
) {
    let seed = 0x5EED_0000_00C0_0063;
    println!("种子 {seed:#x}");
    let returns_to_the_set =
        placement_by_segment_against_stepping_through_every_candidate_over_every_aligned_cluster_set_and_region(
            seed,
            UnitAreaStart::of_the_default_journal_ring(),
            3,
            63,
            2000,
        );
    assert!(
        returns_to_the_set > 0,
        "种子 {seed:#x}：没有一段离开集合又回来，取样点没走到集合翻过 0"
    );
}

/// 128 MiB 环的单元区起点（槽 9216）、三个整段、没有不满的尾段。
/// 不并行：每一步读上一步留下的空闲图，操作的次序就是被测对象。
#[test]
fn user_data_slot_found_by_segment_matches_on_every_aligned_cluster_set_and_region_after_a_128_mebibyte_ring(
) {
    let seed = 0x5EED_0000_00C0_0128;
    println!("种子 {seed:#x}");
    let returns_to_the_set =
        placement_by_segment_against_stepping_through_every_candidate_over_every_aligned_cluster_set_and_region(
            seed,
            UnitAreaStart::following_the_journal_ring(128 * 1024 * 1024)
                .expect("128 MiB 环从槽 1024 起占 8192 槽，下一个槽 9216 = 144 × 64 在段边界上"),
            3,
            0,
            2000,
        );
    assert!(
        returns_to_the_set > 0,
        "种子 {seed:#x}：没有一段离开集合又回来，取样点没走到集合翻过 0"
    );
}

/// 128 MiB 环的单元区起点、三个整段加末尾两槽（末尾那一段恰好一对）。
/// 不并行：每一步读上一步留下的空闲图，操作的次序就是被测对象。
#[test]
fn user_data_slot_found_by_segment_matches_on_every_aligned_cluster_set_and_region_after_a_128_mebibyte_ring_ending_in_one_pair(
) {
    let seed = 0x5EED_0000_00C0_0002;
    println!("种子 {seed:#x}");
    let returns_to_the_set =
        placement_by_segment_against_stepping_through_every_candidate_over_every_aligned_cluster_set_and_region(
            seed,
            UnitAreaStart::following_the_journal_ring(128 * 1024 * 1024)
                .expect("128 MiB 环从槽 1024 起占 8192 槽，下一个槽 9216 = 144 × 64 在段边界上"),
            3,
            2,
            2000,
        );
    assert!(
        returns_to_the_set > 0,
        "种子 {seed:#x}：没有一段离开集合又回来，取样点没走到集合翻过 0"
    );
}
