use super::*;
use singlefs_format::UNIT_AREA_START_SLOT;

/// 开段之后才隔离的槽（抬 F 重算影子账落在活分配器上）bump 游标要绕开：先开段发一个单槽单元，再把游标前方的 50180 隔离，
/// 之后连发四个单槽单元一个都不落在 50180 上。
#[test]
fn commit_generated_bump_skips_a_slot_isolated_after_the_segment_was_opened() {
    let mut pool = pool_after_mkfs();
    let first = pool
        .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
        .expect("开段");
    let open = pool.open_segment().expect("开放段");
    let isolated = SlotNumber(open.0 + 4);
    assert!(first.slot < isolated, "隔离的槽在游标前方");
    pool.isolate_abandoned(DeviceIdentity(0), isolated, 1);
    pool.isolate_abandoned(DeviceIdentity(1), isolated, 1);
    let handed_out: Vec<SlotNumber> = (0..4)
        .map(|_| {
            pool.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
                .expect("开放段还有空")
                .slot
        })
        .collect();
    assert!(
        !handed_out.contains(&isolated),
        "被隔离的 {isolated:?} 发了出去：{handed_out:?}"
    );
    assert!(
        handed_out.contains(&SlotNumber(isolated.0 + 1)),
        "游标越过隔离的槽接着发：{handed_out:?}"
    );
}

#[test]
fn new_pool_file_creation_placements_follow_the_byte_table() {
    let mut pool = pool_after_mkfs();
    assert_eq!(
        pool.devices[0].free_runs(),
        1,
        "mkfs 占了单元区开头，空闲仍是一段"
    );
    assert_eq!(pool.devices[0].empty_segments(), 3311);
    let data = pool.allocate_user_data(CheckpointTxg(3)).expect("有空槽");
    assert_eq!(
        data,
        Placement {
            slot: SlotNumber(50180),
            span: 2
        },
        "t1@50180：最低的偶数空槽对"
    );
    assert_eq!(
        pool.records().len(),
        6,
        "mkfs 两个单元 × 2 盘（分配代 0）+ t1 × 2 盘"
    );
    let extent_root = pool
        .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
        .expect("开放段");
    assert_eq!(
        extent_root.slot,
        SlotNumber(50240),
        "开放段 = 最低的 64 槽对齐全空段"
    );
    let inode_leaf = pool
        .allocate_commit_generated(UnitFootprint::TwoSlotsAligned, CheckpointTxg(3))
        .expect("开放段");
    assert_eq!(
        inode_leaf,
        Placement {
            slot: SlotNumber(50242),
            span: 2
        },
        "容器按 32768 对齐，槽 50241 空着"
    );
    for expected in [50244u64, 50245, 50246, 50247, 50248] {
        let node = pool
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
            .expect("开放段");
        assert_eq!(node.slot, SlotNumber(expected));
    }
    assert_eq!(
        pool.records().len(),
        20,
        "m1 m2 t1..t8 十个单元 × 2 盘（字节表五）"
    );
}

#[test]
fn releasing_a_placement_keeps_the_slots_occupied_and_moves_them_into_the_defer_queue() {
    let mut pool = pool_after_mkfs();
    let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
    pool.release(data, CheckpointTxg(4));
    assert_eq!(pool.devices[0].allocated_slots(), 5, "占着的槽数不变");
    assert_eq!(pool.devices[0].deferred_slots(), 2, "两槽进了 defer 队列");
    assert_eq!(pool.devices[0].free_slots(), 211_968 - 5, "空闲不变");
    let released: Vec<&AllocationRecord> = pool
        .records()
        .iter()
        .filter(|record| record.slot == data.slot)
        .collect();
    assert_eq!(released.len(), 2, "每盘一条");
    for record in released {
        assert!(record.is_released);
        assert_eq!(record.generation, CheckpointTxg(4), "释放代");
        assert_eq!(record.span_slots, 2, "跨度值不含标志位");
        let parsed = AllocationRecord::parse(&record.to_bytes());
        assert_eq!(parsed, Ok(*record), "标志位进跨度段最高位再读回来");
        assert_eq!(
            AllocationRecord::parse(&record.to_bytes()[..19]),
            Err(
                AllocationRecordEntryRefusal::EntryWidthDiffersFromTheFieldTable {
                    entry_bytes: 19
                }
            ),
            "窄于字段表的 20 字节一律不解（条目宽是盘上的值）"
        );
        assert_eq!(
            u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]),
            2 | ALLOCATION_RECORD_RELEASED_FLAG
        );
    }
    assert_eq!(
        pool.allocate_user_data(CheckpointTxg(4))
            .expect("有空槽")
            .slot,
        SlotNumber(50182),
        "已释放的 50180 不许再发，下一个偶数空槽对是 50182"
    );
}

/// 隔离（`release_leaving_the_record_allocated_on` 点名两块盘）：两块盘上那条记录仍分配、分配代不变、带隔离记号，
/// 落盘的跨度段是 0x4002；槽不进 defer、仍占着。
#[test]
fn quarantining_a_placement_marks_the_record_on_every_named_device_and_keeps_its_generation() {
    let mut pool = pool_after_mkfs();
    let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
    pool.release_leaving_the_record_allocated_on(
        data,
        CheckpointTxg(4),
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    let quarantined: Vec<&AllocationRecord> = pool
        .records()
        .iter()
        .filter(|record| record.slot == data.slot)
        .collect();
    assert_eq!(quarantined.len(), 2, "每盘一条");
    for record in quarantined {
        assert!(!record.is_released, "隔离的记录留在已分配");
        assert_eq!(record.generation, CheckpointTxg(3), "分配代不变");
        assert_eq!(
            record.quarantine_mark,
            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
            "隔离那次发布置上记号"
        );
        assert_eq!(
            u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]),
            2 | ALLOCATION_RECORD_QUARANTINE_MARK,
            "落盘的跨度段是 0x4002"
        );
    }
    assert_eq!(pool.devices[0].deferred_slots(), 0, "隔离的槽不进 defer");
}

/// 释放隔离记录时改写成已释放、记号随之清掉（D19 已定项 5）：跨度段落盘是 0x8002。
#[test]
fn releasing_a_quarantined_record_rewrites_it_released_and_clears_the_mark() {
    let mut pool = pool_after_mkfs();
    let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
    pool.release_leaving_the_record_allocated_on(
        data,
        CheckpointTxg(4),
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    pool.release(data, CheckpointTxg(5));
    for record in pool
        .records()
        .iter()
        .filter(|record| record.slot == data.slot)
    {
        assert!(record.is_released, "改写成已释放");
        assert_eq!(record.generation, CheckpointTxg(5), "释放代");
        assert_eq!(
            record.quarantine_mark,
            AllocationRecordQuarantineMark::NotQuarantined,
            "记号随释放清掉"
        );
        assert_eq!(
            u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]),
            2 | ALLOCATION_RECORD_RELEASED_FLAG,
            "落盘的跨度段是 0x8002"
        );
    }
}

/// 回退清记号：记录仍分配、分配代不变，记号去掉，槽照旧占着。
#[test]
fn clearing_the_quarantine_mark_keeps_the_record_allocated_under_its_generation() {
    let mut pool = pool_after_mkfs();
    let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
    pool.release_leaving_the_record_allocated_on(
        data,
        CheckpointTxg(4),
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    let allocated_slots_while_quarantined = pool.devices[0].allocated_slots();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        pool.clear_the_quarantine_mark_of_a_record_the_rollback_target_references(device, data);
    }
    for record in pool
        .records()
        .iter()
        .filter(|record| record.slot == data.slot)
    {
        assert!(!record.is_released, "仍分配");
        assert_eq!(record.generation, CheckpointTxg(3), "分配代不变");
        assert_eq!(
            record.quarantine_mark,
            AllocationRecordQuarantineMark::NotQuarantined,
            "回退那一次清掉记号"
        );
    }
    assert_eq!(
        pool.devices[0].allocated_slots(),
        allocated_slots_while_quarantined,
        "位图不动"
    );
}

/// 重建时认出还没被换下的第 0 版树表单元（分配代 0、跨度 1、未释放）；换下之后（已释放）就不认。
#[test]
fn rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released() {
    let device_bytes = 4_u64 * 1024 * 1024 * 1024;
    let records = |released: bool| {
        vec![
            AllocationRecord {
                device: DeviceIdentity(0),
                slot: SlotNumber(50176),
                span_slots: 2,
                generation: CheckpointTxg(0),
                is_released: false,
                quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
            },
            AllocationRecord {
                device: DeviceIdentity(0),
                slot: SlotNumber(50178),
                span_slots: 1,
                generation: if released {
                    CheckpointTxg(3)
                } else {
                    CheckpointTxg(0)
                },
                is_released: released,
                quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
            },
        ]
    };
    let fresh = PoolAllocator::rebuild_from_records(
        vec![DeviceFreeMap::new(DeviceIdentity(0), device_bytes)],
        records(false),
    );
    assert_eq!(
        fresh.format_time_tree_table(),
        Some(Placement {
            slot: SlotNumber(50178),
            span: 1
        })
    );
    let replaced = PoolAllocator::rebuild_from_records(
        vec![DeviceFreeMap::new(DeviceIdentity(0), device_bytes)],
        records(true),
    );
    assert_eq!(replaced.format_time_tree_table(), None);
}

/// 两块盘上都在重写集里的叶。
fn rewrite_set_leaves_of_both_devices(leaf_numbers: &[u64]) -> RewriteSetLeaves {
    let on_each_device: BTreeSet<AllocationRecordTreeLeafIndex> = leaf_numbers
        .iter()
        .copied()
        .map(AllocationRecordTreeLeafIndex)
        .collect();
    RewriteSetLeaves::present_on_every_device(&[on_each_device.clone(), on_each_device])
}

/// 每块盘上把单元区里还空着的槽都占掉，只留 `kept_free_per_device` 里点名这块盘的那几个（只动空闲图，不写分配记录：造形态用）。
fn fill_every_free_slot_except(pool: &mut PoolAllocator, kept_free_per_device: &[&[u64]]) {
    for (device_map, kept_free) in pool.devices.iter_mut().zip(kept_free_per_device) {
        for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + device_map.unit_area_slots() {
            if device_map.is_free(SlotNumber(slot)) && !kept_free.contains(&slot) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 两块盘留同一组空槽：叶 70（[56840, 57652)）里 3 个、叶 90（[73080, 73892)）里 5 个、叶 100（[81200, 82012)）里 5 个，别的全占掉。
/// 没有全空段、没有开放段 ⇒ 提交内生块回落；槽号最小的空槽在叶 70，空槽最多的叶是 90 与 100（一样多，90 的叶序号小）。
fn pool_with_free_slots_left_in_three_leaves() -> PoolAllocator {
    let kept_free: Vec<u64> = (56_850..56_853)
        .chain(73_180..73_185)
        .chain(81_300..81_305)
        .collect();
    let mut pool = pool_after_mkfs();
    fill_every_free_slot_except(&mut pool, &[kept_free.as_slice(), kept_free.as_slice()]);
    pool
}

/// 回落先取重写集的叶：叶 62 在两块盘上都重写，取它里面最低的空槽 50344，不取空槽最多的叶 63。
/// 没有重写集取空槽最多的叶 63 里最低的空槽 51156（槽号最小的空槽 50179 在叶 61，不取）；只在一块盘上重写的叶不算（交集），同样取 51156。
#[test]
fn fallback_takes_a_free_slot_in_the_rewrite_set_leaf_before_the_leaf_with_the_most_free_slots() {
    let mut preferring = pool_after_mkfs_whose_next_commit_generated_block_falls_back();
    let placement = preferring
        .try_allocate_commit_generated_preferring_the_rewrite_set(
            UnitFootprint::OneSlot,
            CheckpointTxg(4),
            &rewrite_set_leaves_of_both_devices(&[62]),
        )
        .expect("回落");
    assert_eq!(
        placement.slot,
        SlotNumber(50344),
        "重写集的叶 62 里最低的空槽"
    );
    assert_eq!(preferring.open_segment(), None, "回落不开段");
    assert_eq!(preferring.commit_generated_placements_that_fell_back(), 1);

    let mut without_a_rewrite_set = pool_after_mkfs_whose_next_commit_generated_block_falls_back();
    for device_map in &without_a_rewrite_set.devices {
        assert_eq!(
            device_map.lowest_commit_generated_fallback_slot(UnitFootprint::OneSlot),
            Some(SlotNumber(50179)),
            "这块盘自己答的回落落点是槽号最小的空槽"
        );
    }
    let placement_without_a_rewrite_set = without_a_rewrite_set
        .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
        .expect("回落");
    assert_eq!(
        placement_without_a_rewrite_set.slot,
        SlotNumber(51156),
        "空槽最多的叶 63 里最低的空槽"
    );

    let only_on_one_device = RewriteSetLeaves::present_on_every_device(&[
        BTreeSet::from([AllocationRecordTreeLeafIndex(62)]),
        BTreeSet::new(),
    ]);
    assert!(
        only_on_one_device.leaf_indices().is_empty(),
        "只在一块盘上重写的叶不算"
    );
    let mut preferring_a_leaf_of_one_device =
        pool_after_mkfs_whose_next_commit_generated_block_falls_back();
    let placement_preferring_a_leaf_of_one_device = preferring_a_leaf_of_one_device
        .try_allocate_commit_generated_preferring_the_rewrite_set(
            UnitFootprint::OneSlot,
            CheckpointTxg(4),
            &only_on_one_device,
        )
        .expect("回落");
    assert_eq!(
        placement_preferring_a_leaf_of_one_device.slot,
        SlotNumber(51156)
    );
}

/// [`pool_with_free_slots_left_in_three_leaves`] 造出来的形态：没有全空段；每块盘上叶 90 没挡 5 槽、叶 70 没挡 3 槽。
fn assert_the_pool_with_free_slots_left_in_three_leaves_has_no_empty_segment(pool: &PoolAllocator) {
    for device_map in &pool.devices {
        assert_eq!(
            device_map.lowest_empty_segment(),
            None,
            "造的形态没有全空段"
        );
        assert_eq!(
            device_map
                .unblocked_slots_in_allocation_record_tree_leaf(AllocationRecordTreeLeafIndex(90)),
            SlotCountInALeaf(5)
        );
        assert_eq!(
            device_map
                .unblocked_slots_in_allocation_record_tree_leaf(AllocationRecordTreeLeafIndex(70)),
            SlotCountInALeaf(3)
        );
    }
}

/// 重写集的叶里没有空槽（叶 75 全占着）就取空槽最多的叶：叶 90 与叶 100 各 5 个空槽、叶 70 只有 3 个 ⇒ 单槽的单元取叶 90 里最低的 73180，
/// 不取槽号最小的 56850（叶 70），一样多时不取叶序号大的叶 100。
#[test]
fn fallback_of_a_one_slot_unit_without_room_in_the_rewrite_set_leaves_takes_the_lowest_slot_of_the_leaf_with_the_most_free_slots(
) {
    let mut pool = pool_with_free_slots_left_in_three_leaves();
    assert_the_pool_with_free_slots_left_in_three_leaves_has_no_empty_segment(&pool);
    let placement = pool
        .try_allocate_commit_generated_preferring_the_rewrite_set(
            UnitFootprint::OneSlot,
            CheckpointTxg(4),
            &rewrite_set_leaves_of_both_devices(&[75]),
        )
        .expect("回落");
    assert_eq!(
        placement,
        Placement {
            slot: SlotNumber(73_180),
            span: 1,
        }
    );
}

/// 同一形态上两槽的单元：取空槽最多、叶序号小的叶 90 里最低的一对偶数起点空槽 73180–73181。
#[test]
fn fallback_of_a_two_slot_unit_without_room_in_the_rewrite_set_leaves_takes_the_lowest_even_pair_of_the_leaf_with_the_most_free_slots(
) {
    let mut pool = pool_with_free_slots_left_in_three_leaves();
    assert_the_pool_with_free_slots_left_in_three_leaves_has_no_empty_segment(&pool);
    let placement = pool
        .try_allocate_commit_generated_preferring_the_rewrite_set(
            UnitFootprint::TwoSlotsAligned,
            CheckpointTxg(4),
            &rewrite_set_leaves_of_both_devices(&[75]),
        )
        .expect("回落");
    assert_eq!(
        placement,
        Placement {
            slot: SlotNumber(73_180),
            span: 2,
        }
    );
}

/// 同一形态上重写集的叶是只有 3 个空槽的叶 70：重写集的叶里有空槽就先取它（56850），哪怕它空槽比叶 90、叶 100 少。
#[test]
fn fallback_takes_a_free_slot_of_a_rewrite_set_leaf_poorer_than_the_leaf_with_the_most_free_slots()
{
    let mut preferring_the_poorer_leaf = pool_with_free_slots_left_in_three_leaves();
    let placement = preferring_the_poorer_leaf
        .try_allocate_commit_generated_preferring_the_rewrite_set(
            UnitFootprint::OneSlot,
            CheckpointTxg(4),
            &rewrite_set_leaves_of_both_devices(&[70]),
        )
        .expect("回落");
    assert_eq!(
        placement.slot,
        SlotNumber(56_850),
        "重写集的叶里有空槽就先取它，哪怕它空槽少"
    );
}

/// 活整理意图的 `R`（[73152, 73216)，罩着叶 90 那 5 个空槽）两步都排除：重写集的叶 90 里的空槽全在 R 里 ⇒ 让给空槽最多的叶；
/// 叶 90 刨掉 R 之后一个空槽都不剩 ⇒ 空槽最多的是叶 100，取 81300。
#[test]
fn fallback_excludes_the_live_compaction_region_from_the_rewrite_set_leaves_and_from_the_leaf_with_the_most_free_slots(
) {
    let mut pool = pool_with_free_slots_left_in_three_leaves();
    pool.compaction_mut()
        .issue(crate::logged_operations::CompactionIntent {
            region: CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(73_152))
                .expect("73152 是 64 的倍数"),
            allocation_generation_at_issue: CheckpointTxg(3),
            empty_cluster_segments_at_issue: 0,
        });
    assert!(pool.region_excluded_from_placement().is_some(), "R 排他");
    let placement = pool
        .try_allocate_commit_generated_preferring_the_rewrite_set(
            UnitFootprint::OneSlot,
            CheckpointTxg(4),
            &rewrite_set_leaves_of_both_devices(&[90]),
        )
        .expect("回落");
    assert_eq!(placement.slot, SlotNumber(81_300));
}

/// 两块盘的空闲图不同：盘 0 留叶 70 的 56850–56851、叶 90 的 73180–73184、叶 100 的 81300–81303；盘 1 留 56851、73182–73186、81300–81303。
/// 各盘自己答的回落落点不同（56850 对 56851）；空槽最多的叶按两块盘都空的槽数：叶 90 每块盘各 5 个、两块盘都空的只有 73182–73184 三个，
/// 叶 100 四个 ⇒ 取叶 100 的 81300，两块盘上各记一条同槽的记录。
#[test]
fn fallback_on_two_devices_counts_the_slots_free_on_every_device_and_records_one_slot_on_both() {
    let mut pool = pool_after_mkfs();
    let kept_free_on_device_zero: Vec<u64> = (56_850..56_852)
        .chain(73_180..73_185)
        .chain(81_300..81_304)
        .collect();
    let kept_free_on_device_one: Vec<u64> = std::iter::once(56_851)
        .chain(73_182..73_187)
        .chain(81_300..81_304)
        .collect();
    fill_every_free_slot_except(
        &mut pool,
        &[
            kept_free_on_device_zero.as_slice(),
            kept_free_on_device_one.as_slice(),
        ],
    );
    assert_eq!(
        pool.devices
            .iter()
            .map(|device_map| device_map
                .lowest_commit_generated_fallback_slot(UnitFootprint::OneSlot))
            .collect::<Vec<_>>(),
        vec![Some(SlotNumber(56_850)), Some(SlotNumber(56_851))],
        "各盘自己答的回落落点不同"
    );
    let placement = pool
        .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
        .expect("两块盘都空的槽还有：回落各盘同槽");
    assert_eq!(placement.slot, SlotNumber(81_300));
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let record = pool
            .record_for(device, SlotNumber(81_300))
            .unwrap_or_else(|| panic!("盘 {device:?} 上没有 81300 的记录"));
        assert_eq!(record.generation, CheckpointTxg(4));
    }
}

/// 回落把开放段置空之后，那一段仍是聚簇段、仍不给用户数据
/// （D3（空间分配） 已定项 8 第 2 条「聚簇段只给提交内生块」；已定项 10 ② 的量词是「任何开放的聚簇段」）。
/// 形态：开段 [50240, 50304) 发七个提交内生块（新池新建文件的前七个，字节表零 t2..t8）⇒ 游标停在 50249；段里游标前方的槽占满、单元区里一个全空段都不剩
/// ⇒ 下一个提交内生块回落、`open_segment` 置空；再把段里那个两槽容器释放、回收（游标不回头，bump 够不着它）——
/// 段里于是有一对空的偶数槽 50242–50243，而且它是全池最低的一对，段里 50244–50248 还躺着活着的内生块。
/// 这时要一个用户数据落点：它必须落在段外面。
#[test]
fn user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed() {
    let mut pool = pool_after_mkfs();
    let extent_root = pool
        .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
        .expect("开段");
    assert_eq!(extent_root.slot, SlotNumber(50240));
    let inode_leaf = pool
        .allocate_commit_generated(UnitFootprint::TwoSlotsAligned, CheckpointTxg(3))
        .expect("开放段");
    assert_eq!(
        inode_leaf,
        Placement {
            slot: SlotNumber(50242),
            span: 2
        }
    );
    for expected in [50244u64, 50245, 50246, 50247, 50248] {
        assert_eq!(
            pool.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
                .expect("开放段")
                .slot,
            SlotNumber(expected)
        );
    }
    assert_eq!(
        pool.cluster_segments(),
        &BTreeSet::from([SlotNumber(50240)])
    );
    // 段里游标前方占满、别的段各占掉段首一槽 ⇒ 没有全空段；段外面（第三段起）还有大片空槽。
    fill_free_slots(&mut pool, 50249, 50304);
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
    let fell_back = pool
        .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
        .expect("没有全空段、段外面还有空槽：回落");
    assert_eq!(
        pool.open_segment(),
        None,
        "回落之后没有段可 bump（回落落点 {:?}）",
        fell_back.slot
    );
    assert_eq!(
        pool.cluster_segments(),
        &BTreeSet::from([SlotNumber(50240)]),
        "回落不把那一段从聚簇段里去掉：它还装着这次提交的内生块"
    );
    // 段里那个两槽容器被换下、释放、回收：50242–50243 空出来，游标（50249）不回头，bump 够不着它们。
    pool.release(inode_leaf, CheckpointTxg(4));
    assert_eq!(
        pool.reclaim_released_up_to(CheckpointTxg(4), ReclaimedReuse::Immediately),
        vec![inode_leaf]
    );
    // 段外面最低的偶数空槽对：50176–50239 与 [50240, 50304) 之外，第三段段首 50304 占着 ⇒ 50306–50307。
    fill_free_slots(&mut pool, UNIT_AREA_START_SLOT, 50240);
    for device_map in &pool.devices {
        assert!(
            device_map.is_free(SlotNumber(50242)) && device_map.is_free(SlotNumber(50243)),
            "盘 {:?}：段里那一对空着，而且是全池最低的一对",
            device_map.device
        );
    }
    let user_data = pool
        .try_allocate_user_data(CheckpointTxg(5))
        .expect("段外面有空槽对");
    assert_eq!(
        user_data,
        Placement {
            slot: SlotNumber(50306),
            span: 2
        },
        "用户数据要落在聚簇段外面：段里空着的 50242–50243 与它旁边活着的内生块 50244–50248 同一段"
    );
}

/// 每块盘都答回落、两块盘的空槽错开（盘 0 只剩 56850、盘 1 只剩 56851）：池里取不到每块盘都空的落点，
/// 报「回落取不到每块盘都空的落点」并带上各盘自己答的回落落点，不报段对齐那一条；分配器一样都没动（记录、回落计数、开放段）。
#[test]
fn fallback_without_a_slot_free_on_every_device_is_refused_naming_each_devices_own_fallback_slot() {
    let mut pool = pool_after_mkfs();
    fill_every_free_slot_except(&mut pool, &[[56_850].as_slice(), [56_851].as_slice()]);
    let records_before = pool.records().to_vec();
    let refusal = pool
        .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
        .expect_err("没有一个槽在两块盘上都空");
    assert_eq!(
        refusal,
        PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice {
            fallback_slot_per_device: vec![
                (DeviceIdentity(0), SlotNumber(56_850)),
                (DeviceIdentity(1), SlotNumber(56_851)),
            ],
        }
    );
    assert_eq!(pool.records(), records_before.as_slice(), "记录一条没动");
    assert_eq!(pool.commit_generated_placements_that_fell_back(), 0);
    assert_eq!(pool.open_segment(), None);
}

/// newtype 的序关系与核的 u64 比较一致，覆盖相等边界及最高位，防止接核时窄化或误改成严格小于。
#[test]
fn verified_reclaim_predicate_preserves_txg_order_at_unsigned_boundaries() {
    let generations = [
        0,
        1,
        u64::from(u32::MAX),
        u64::from(u32::MAX) + 1,
        u64::MAX - 1,
        u64::MAX,
    ];
    for released in [false, true] {
        for generation in generations {
            for floor in generations {
                let generation = CheckpointTxg(generation);
                let floor = CheckpointTxg(floor);
                assert_eq!(
                    singlefs_replay_prefix::gc_crash_proofs::released_record_can_be_reclaimed(
                        released,
                        generation.0,
                        floor.0
                    ),
                    released && generation <= floor
                );
            }
        }
    }
}
