use super::*;

fn settings_with_floor(floor_segments: u64) -> PoolSettings {
    PoolSettings {
        floor_segments,
        is_compaction_enabled: true,
        stop_arm: StopArm::LocalRegionPredicate,
        resident_base_multiplier: 3,
    }
}

/// 两块盘、每块 50176 + 64 × segments 槽；叶宽 812 ⇒ 树高 2、b = 3。
fn small_geometry(segments: u64, root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64) -> PoolGeometry {
    PoolGeometry::new(
        UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS * segments,
        ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        root_ring_slots_per_region,
        commit_generated_blocks_per_publish,
    )
}

fn four_gibibyte_geometry(root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64) -> PoolGeometry {
    PoolGeometry::new(
        FOUR_GIBIBYTE_DISK_SLOTS,
        ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        root_ring_slots_per_region,
        commit_generated_blocks_per_publish,
    )
}

/// 在事务外直接摆一个用户数据单元（单测构造状态用）。
fn place_user_unit(pool: &mut Pool, even_offset: u64, birth_txg: u64) -> u32 {
    pool.begin_transaction();
    let unit = pool.create_unit(even_offset, birth_txg);
    pool.allocate_slot(even_offset, SlotOwner::UserDataUnit { unit }, birth_txg);
    pool.allocate_slot(even_offset + 1, SlotOwner::UserDataUnit { unit }, birth_txg);
    pool.commit_transaction();
    unit
}

/// 在事务外直接摆一个占槽的提交内生块记录（只为让段有活槽，不进块位置表）。
fn place_live_commit_generated_slot(pool: &mut Pool, offset: u64) {
    let block = BlockIdentity::FileTreeNode { node: 1_000_000 + offset };
    pool.set_slot_state(offset, SlotState::Live { owner: SlotOwner::CommitGeneratedBlock { block }, allocation_txg: 0 });
}

#[test]
fn formula_values_in_the_nine_cells_match_the_values_registered_before_the_run() {
    assert_eq!(formula_floor_segments(4, 5), 1 + 1);
    assert_eq!(formula_floor_segments(4, 10), 2 + 1);
    assert_eq!(formula_floor_segments(4, 13), 3 + 1);
    assert_eq!(formula_floor_segments(8, 5), 2 + 1);
    assert_eq!(formula_floor_segments(8, 10), 4 + 1);
    assert_eq!(formula_floor_segments(8, 13), 5 + 1);
    assert_eq!(formula_floor_segments(16, 5), 4 + 1);
    assert_eq!(formula_floor_segments(16, 10), 8 + 1);
    assert_eq!(formula_floor_segments(16, 13), 10 + 1);
    for (root_ring_slots_per_region, commit_generated_blocks_per_publish, registered) in FORMULA_VALUES_REGISTERED_BEFORE_THE_RUN {
        assert_eq!(formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish), registered);
    }
    assert!(formula_lines().iter().all(|line| line.ends_with("matches_registered=true")));
}

#[test]
fn ring_slots_minus_one_times_commit_generated_blocks_in_the_nine_cells() {
    let products: Vec<u64> = FORMULA_VALUES_REGISTERED_BEFORE_THE_RUN
        .iter()
        .map(|(root_ring_slots_per_region, commit_generated_blocks_per_publish, _registered)| {
            (ROOT_RING_REGIONS * root_ring_slots_per_region - 1) * commit_generated_blocks_per_publish
        })
        .collect();
    assert_eq!(products, vec![55, 110, 143, 115, 230, 299, 235, 470, 611]);
}

#[test]
fn compaction_call_budget_matches_the_judgement_values() {
    assert_eq!(compaction_call_publish_budget(4, 0), 4 + 4);
    assert_eq!(compaction_call_publish_budget(8, 0), 16 + 4);
    assert_eq!(compaction_call_publish_budget(16, 0), 40 + 4);
    assert_eq!(compaction_call_publish_budget(4, 2), 2 + 4);
    assert_eq!(compaction_call_publish_budget(8, 2), 14 + 4);
    assert_eq!(compaction_call_publish_budget(16, 2), 38 + 4);
    for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
        let publishes = 3 * root_ring_slots_per_region - 4;
        assert_eq!(compaction_call_publish_budget(root_ring_slots_per_region, publishes), 1);
        assert_eq!(compaction_call_publish_budget(root_ring_slots_per_region, publishes + 7), 1);
    }
}

#[test]
fn geometry_of_the_two_sampling_disk_sizes() {
    let four = four_gibibyte_geometry(8, 5);
    assert_eq!((four.unit_area_slots(), four.segment_count()), (211_968, 3_312));
    assert_eq!(four.leaf_of_absolute_slot(UNIT_AREA_START_SLOT), 61);
    assert_eq!(four.leaf_of_absolute_slot(FOUR_GIBIBYTE_DISK_SLOTS - 1), 322);
    assert_eq!(four.leaves_covering_the_unit_area(), 262);
    assert_eq!((four.record_tree_root_level(), four.record_tree_blocks_per_publish_at_one_leaf()), (2, 5));
    let one = PoolGeometry::new(ONE_GIBIBYTE_DISK_SLOTS, ALLOCATION_RECORD_TREE_LEAF_SLOTS, 8, 5);
    assert_eq!((one.unit_area_slots(), one.segment_count()), (15_360, 240));
    assert_eq!(one.leaf_of_absolute_slot(ONE_GIBIBYTE_DISK_SLOTS - 1), 80);
    assert_eq!(one.leaves_covering_the_unit_area(), 20);
    assert_eq!((one.record_tree_root_level(), one.record_tree_blocks_per_publish_at_one_leaf()), (1, 3));
    assert_eq!(four.structural_commit_generated_block_bound(), 2 * 262 + 2 * 2 + 1);
    assert_eq!(one.structural_commit_generated_block_bound(), 2 + 2 * 20 + 1);
}

#[test]
fn dirty_slots_across_the_level_one_boundary_write_four_more_blocks() {
    let geometry = four_gibibyte_geometry(8, 10);
    assert_eq!(geometry.level_one_width_in_slots(), 137_228);
    assert_eq!((geometry.leaf_of_absolute_slot(136_416), geometry.leaf_of_absolute_slot(137_227)), (168, 168));
    assert_eq!(geometry.leaf_of_absolute_slot(137_228), 169);
    assert_eq!(commit_generated_block_count_for_dirty_slots(&geometry, &[136_416, 137_228]), 10 + 4);
    assert_eq!(commit_generated_block_count_for_dirty_slots(&geometry, &[136_416, 136_418]), 10);
}

#[test]
fn released_slot_is_reclaimable_ring_slots_minus_one_publishes_after_its_release() {
    for (root_ring_slots_per_region, last_held, first_reclaimable) in [(4, 10, 11), (8, 22, 23), (16, 46, 47)] {
        let geometry = four_gibibyte_geometry(root_ring_slots_per_region, 5);
        let release_txg = 1_000;
        assert!(release_txg > reclaim_threshold_at_publish(release_txg + last_held, 0, &geometry));
        assert!(release_txg <= reclaim_threshold_at_publish(release_txg + first_reclaimable, 0, &geometry));
    }
}

#[test]
fn new_pool_on_two_four_gibibyte_disks_writes_five_record_tree_blocks_from_slot_50240() {
    let pool = Pool::made_by_make_filesystem(four_gibibyte_geometry(8, 5), settings_with_floor(1));
    assert_eq!(pool.geometry.record_tree_blocks_per_publish_at_one_leaf(), 5);
    let mut block_slots: Vec<u64> = pool
        .leaf_slots
        .iter()
        .chain(pool.level_one_slots.iter())
        .chain(std::iter::once(&pool.root_slot))
        .flatten()
        .map(|offset| offset + UNIT_AREA_START_SLOT)
        .collect();
    block_slots.sort_unstable();
    assert_eq!(block_slots, vec![50_240, 50_241, 50_242, 50_243, 50_244]);
    assert_eq!(pool.slot_state(0).live_owner(), Some(SlotOwner::InstanceTable));
    assert_eq!(pool.slot_state(1).live_owner(), Some(SlotOwner::InstanceTable));
    assert_eq!(pool.segments[1].live_slots, 5);
    let with_file_tree = Pool::made_by_make_filesystem(four_gibibyte_geometry(8, 10), settings_with_floor(1));
    assert_eq!(with_file_tree.segments[1].live_slots, 10);
    assert_eq!(with_file_tree.txg, 0);
    assert!(with_file_tree.open_cursor.is_none(), "mkfs 之后挂载一次，游标作废");
}

/// 段 2–7 各一个用户数据单元：全空段数（记录口径）是 0，空闲槽还多。
fn pool_without_record_empty_segments(floor_segments: u64) -> Pool {
    let mut pool = Pool::made_by_make_filesystem(small_geometry(8, 4, 3), settings_with_floor(floor_segments));
    for segment in 2..8 {
        place_user_unit(&mut pool, segment * CLUSTER_SEGMENT_SLOTS, 0);
    }
    assert_eq!(pool.record_empty_segment_count, 0);
    pool
}

#[test]
fn conjunct_seven_refuses_only_foreground_writes_when_no_segment_is_record_empty() {
    let mut writing = pool_without_record_empty_segments(1);
    assert_eq!(writing.publish_new_file(1), Err(PublishRefusal::ConjunctSeven));
    let mut deleting = pool_without_record_empty_segments(1);
    let unit = match deleting.slot_state(2 * CLUSTER_SEGMENT_SLOTS).live_owner() {
        Some(SlotOwner::UserDataUnit { unit }) => unit,
        other => panic!("段 2 起点是用户数据单元：{other:?}"),
    };
    assert!(deleting.publish_head_deletion(&[unit]).is_ok(), "删除类发布不受合取 7 管");
    let mut emptying = pool_without_record_empty_segments(1);
    assert!(emptying.publish_empty().is_ok(), "空发布不受合取 7 管");
    let mut compacting = pool_without_record_empty_segments(1);
    let region = compacting.choose_region_greedy().expect("有可整理的段");
    assert!(
        matches!(compacting.publish_compaction_batch(Some(region)), Ok(BatchOutcome::Published { .. })),
        "整理批不受合取 7 管"
    );
}

#[test]
fn runs_count_released_slots_as_occupied() {
    let mut pool = Pool::blank(small_geometry(2, 4, 3), settings_with_floor(1));
    for offset in (0..2).chain(6..CLUSTER_SEGMENT_SLOTS).chain(CLUSTER_SEGMENT_SLOTS..2 * CLUSTER_SEGMENT_SLOTS) {
        place_live_commit_generated_slot(&mut pool, offset);
    }
    place_live_commit_generated_slot(&mut pool, 4);
    pool.set_slot_state(4, SlotState::Released { release_txg: 1 });
    assert_eq!(pool.free_runs, 2, "段 A 的 2–3 与 5 两段空闲；4 已释放未回收不算空闲");
    pool.verify_incremental_counters_against_a_full_scan();
}

#[test]
fn segment_of_released_slots_is_record_empty_but_neither_fully_free_nor_openable() {
    let mut pool = Pool::blank(small_geometry(2, 4, 3), settings_with_floor(1));
    for offset in 0..2 * CLUSTER_SEGMENT_SLOTS {
        place_live_commit_generated_slot(&mut pool, offset);
    }
    for offset in CLUSTER_SEGMENT_SLOTS..2 * CLUSTER_SEGMENT_SLOTS {
        pool.set_slot_state(offset, SlotState::Released { release_txg: 1 });
    }
    assert_eq!(pool.record_empty_segment_count, 1);
    assert_eq!(pool.fully_free_segments.len(), 0);
    assert_eq!(pool.openable_segment_count(), 0);
    assert_eq!(pool.free_runs, 0);
    assert!(pool.conjunct_seven_admits(), "合取 7 读记录口径：1 ≥ N₀ = 1");
    pool.verify_incremental_counters_against_a_full_scan();
}

#[test]
fn floor_raise_targets_the_fourth_newest_non_empty_root() {
    let ring: VecDeque<u64> = [100, 103, 104, 110, 111].into_iter().collect();
    assert_eq!(floor_raise_target(&ring, 70), 100 + 3);
    let short_ring: VecDeque<u64> = [100, 103, 104].into_iter().collect();
    assert_eq!(floor_raise_target(&short_ring, 70), 70, "非空根不足 4 个取环里最旧有效根");
}

#[test]
fn raised_floor_takes_effect_only_after_the_second_empty_publish() {
    let mut pool = Pool::blank(small_geometry(4, 16, 3), settings_with_floor(0));
    pool.txg = 112;
    pool.non_empty_root_txgs_in_ring = [100, 103, 104, 110, 111].into_iter().collect();
    let held_offset = 3 * CLUSTER_SEGMENT_SLOTS;
    place_live_commit_generated_slot(&mut pool, held_offset);
    pool.set_slot_state(held_offset, SlotState::Released { release_txg: 103 });
    pool.reclaim_queue.push_back((103, held_offset));
    pool.pending_floor_raise = Some((103, 0));
    pool.publish_empty().expect("第 1 次空发布");
    assert_eq!(pool.effective_floor_txg, 0);
    pool.reclaim_for_the_next_publish();
    assert!(pool.slot_state(held_offset).is_released(), "第 1 次之后还不可回收");
    pool.publish_empty().expect("第 2 次空发布");
    assert_eq!(pool.effective_floor_txg, 103);
    pool.reclaim_for_the_next_publish();
    assert!(pool.slot_state(held_offset).is_free(), "第 2 次之后释放代 ≤ 103 的槽可回收");
}

#[test]
fn raise_floor_publishes_two_empty_publishes_up_to_the_fourth_newest_non_empty_root() {
    let mut pool = Pool::blank(small_geometry(4, 16, 3), settings_with_floor(0));
    pool.txg = 112;
    pool.non_empty_root_txgs_in_ring = [100, 103, 104, 110, 111].into_iter().collect();
    pool.raise_floor();
    assert_eq!((pool.txg, pool.effective_floor_txg), (112 + 2, 103));
    pool.raise_floor();
    assert_eq!(pool.txg, 114, "F 已不低于目标就跳过");
}

#[test]
fn greedy_breaks_ties_by_the_lowest_segment() {
    let mut pool = Pool::blank(small_geometry(6, 4, 3), settings_with_floor(1));
    for (segment, live_slots) in [(1, 5), (2, 3), (3, 3), (4, 7)] {
        for slot_in_segment in 0..live_slots {
            place_live_commit_generated_slot(&mut pool, segment * CLUSTER_SEGMENT_SLOTS + slot_in_segment);
        }
    }
    assert_eq!(pool.choose_region_greedy(), Some(2));
}

#[test]
fn greedy_skips_segments_opened_in_this_mount() {
    let mut pool = Pool::blank(small_geometry(4, 4, 3), settings_with_floor(1));
    place_live_commit_generated_slot(&mut pool, CLUSTER_SEGMENT_SLOTS);
    for slot_in_segment in 0..5 {
        place_live_commit_generated_slot(&mut pool, 2 * CLUSTER_SEGMENT_SLOTS + slot_in_segment);
    }
    pool.open_segment(1);
    assert_eq!(pool.choose_region_greedy(), Some(2), "段 1 活槽最少，但这次挂载开过");
}

/// R = 段 0 有 3 个用户数据单元；段 1 有用户数据单元与 `holes_in_destination` 个空闲两槽；段 2 只装提交内生块、有 10 个空闲两槽。
fn pool_for_the_pre_issue_check(holes_in_destination: u64) -> Pool {
    let mut pool = Pool::blank(small_geometry(4, 4, 3), settings_with_floor(1));
    for unit_index in 0..3 {
        place_user_unit(&mut pool, 2 * unit_index, 0);
    }
    for offset in 6..CLUSTER_SEGMENT_SLOTS {
        place_live_commit_generated_slot(&mut pool, offset);
    }
    let destination_start = CLUSTER_SEGMENT_SLOTS;
    place_user_unit(&mut pool, destination_start, 0);
    for offset in destination_start + 2..2 * CLUSTER_SEGMENT_SLOTS - 2 * holes_in_destination {
        place_live_commit_generated_slot(&mut pool, offset);
    }
    for offset in 2 * CLUSTER_SEGMENT_SLOTS..3 * CLUSTER_SEGMENT_SLOTS - 2 * 10 {
        place_live_commit_generated_slot(&mut pool, offset);
    }
    for offset in 3 * CLUSTER_SEGMENT_SLOTS..4 * CLUSTER_SEGMENT_SLOTS {
        place_live_commit_generated_slot(&mut pool, offset);
    }
    assert_eq!(pool.segments[0].live_user_units, 3);
    assert_eq!((pool.segments[1].free_pairs, pool.segments[2].free_pairs), (holes_in_destination, 10));
    pool
}

#[test]
fn pre_issue_check_counts_only_holes_in_destination_segments() {
    assert!(!pool_for_the_pre_issue_check(2).pre_issue_check_passes(0), "合格段 2 个两槽 < 3 个单元");
    assert!(pool_for_the_pre_issue_check(3).pre_issue_check_passes(0), "合格段 3 个两槽 ≥ 3 个单元");
}

#[test]
fn the_fixpoint_rewrites_the_leaf_that_the_new_blocks_land_in() {
    let mut pool = Pool::blank(four_gibibyte_geometry(4, 5), settings_with_floor(0));
    // 叶 62 从绝对槽 50344（偏移 168）起，在段 2（偏移 128–191）里；游标停在 50341，这次的 5 块落在 50341–50345。
    pool.open_segment(2);
    pool.open_cursor = Some(OpenSegmentCursor { segment: 2, next_offset_in_segment: 50_341 - UNIT_AREA_START_SLOT - 128 });
    pool.publish_new_file(1).expect("空池上写一个单元");
    assert_eq!(pool.units[0].first_slot_offset, 0, "用户数据落在单元区起点（叶 61）");
    let written: Vec<u64> = (128..CLUSTER_SEGMENT_SLOTS * 3).filter(|offset| pool.slot_state(*offset).is_live()).collect();
    assert_eq!(written.len(), 5 + 2, "叶 61、层级 1 各两块与根落到 50341–50345，其中两块进了叶 62 ⇒ 叶 62 两块");
    assert!(pool.leaf_slots.iter().flatten().count() == 4, "叶 61 与叶 62 各两块盘一份");
}

#[test]
fn user_data_never_lands_in_the_open_segment() {
    let mut pool = Pool::blank(small_geometry(4, 4, 3), settings_with_floor(0));
    pool.open_segment(0);
    let units = pool.publish_new_file(1).expect("写一个单元");
    assert_eq!(pool.units[usize_of(u64::from(units[0]))].first_slot_offset, CLUSTER_SEGMENT_SLOTS);
}

#[test]
fn user_data_never_lands_in_the_live_intent_region() {
    let mut pool = Pool::blank(small_geometry(4, 4, 3), settings_with_floor(0));
    pool.active_intent = Some(CompactionIntent {
        region_segment: 0,
        allocation_txg_at_issue: 0,
        record_empty_segments_at_issue: 0,
        openable_segments_at_issue: 0,
    });
    let units = pool.publish_new_file(1).expect("写一个单元");
    assert_eq!(pool.units[usize_of(u64::from(units[0]))].first_slot_offset, CLUSTER_SEGMENT_SLOTS);
    assert!((0..CLUSTER_SEGMENT_SLOTS).all(|offset| pool.slot_state(offset).is_free()), "R 里一块提交内生块都没落");
}

#[test]
fn unit_pinned_by_some_snapshot_keeps_its_slots_when_the_head_deletes_it() {
    let mut pool = Pool::made_by_make_filesystem(small_geometry(8, 4, 5), settings_with_floor(0));
    let units = pool.publish_new_file(1).expect("写一个单元");
    let (pinned_units, pinned_file_tree_offsets) = pool.publish_snapshot_creation().expect("建快照");
    assert_eq!((pinned_units.clone(), pinned_file_tree_offsets.len()), (units.clone(), 2), "c − b = 2 个文件树节点被钉");
    pool.publish_head_deletion(&units).expect("头一侧删掉");
    let first_slot_offset = pool.units[usize_of(u64::from(units[0]))].first_slot_offset;
    assert!(pool.slot_state(first_slot_offset).is_live(), "快照钉着，不释放");
    assert_eq!(pool.snapshot_only_unit_total, 1);
    pool.publish_snapshot_destruction_batch(&pinned_units, &pinned_file_tree_offsets).expect("销毁快照");
    assert!(pool.slot_state(first_slot_offset).is_released(), "最后一个快照销毁时释放");
    assert!(pinned_file_tree_offsets.iter().all(|offset| pool.slot_state(*offset).is_released()));
    assert_eq!(pool.snapshot_only_unit_total, 0);
    pool.verify_incremental_counters_against_a_full_scan();
}

/// 段 0 实例表、段 1 mkfs 的块；R 候选段 2 有两个用户数据单元；段 3 有一个用户数据单元（目的地）；段 4–9 全空。
fn pool_with_a_two_unit_region(stop_arm: StopArm) -> Pool {
    let settings = PoolSettings { floor_segments: 0, is_compaction_enabled: true, stop_arm, resident_base_multiplier: 3 };
    let mut pool = Pool::made_by_make_filesystem(small_geometry(10, 4, 3), settings);
    place_user_unit(&mut pool, 2 * CLUSTER_SEGMENT_SLOTS, 0);
    place_user_unit(&mut pool, 2 * CLUSTER_SEGMENT_SLOTS + 2, 0);
    place_user_unit(&mut pool, 3 * CLUSTER_SEGMENT_SLOTS, 0);
    pool
}

fn published_without_ending(outcome: Result<BatchOutcome, PublishRefusal>) {
    assert_eq!(outcome, Ok(BatchOutcome::Published { ended_intent_index: None }), "第一批搬一个、意图还在");
}

#[test]
fn opening_an_empty_segment_during_the_intent_makes_the_global_difference_false_while_the_region_truly_empties() {
    for stop_arm in [StopArm::LocalRegionPredicate, StopArm::GlobalDifference] {
        let mut pool = pool_with_a_two_unit_region(stop_arm);
        published_without_ending(pool.publish_compaction_batch(Some(2)));
        // 第一批的块落在段 4（开放段）；摆一条不会被重写的记录让它不因块搬走而变回全空，再把游标用尽，下一次写只能开段 5。
        place_live_commit_generated_slot(&mut pool, 4 * CLUSTER_SEGMENT_SLOTS + 40);
        pool.open_cursor = Some(OpenSegmentCursor { segment: 4, next_offset_in_segment: CLUSTER_SEGMENT_SLOTS });
        let record_empty_before_the_write = pool.record_empty_segment_count;
        pool.publish_new_file(1).expect("前台写一个单元");
        assert_eq!(pool.record_empty_segment_count + 1, record_empty_before_the_write, "这次写开掉了一个全空段");
        let outcome = pool.publish_compaction_batch(None).expect("第二批");
        assert_eq!(outcome, BatchOutcome::Published { ended_intent_index: Some(0) });
        let ended = pool.ended_intents[0];
        assert!(ended.is_completed && ended.region_is_record_empty, "{stop_arm:?}：P_R 真");
        assert!(!ended.global_difference_is_positive, "{stop_arm:?}：P_G 假（腾出 R 与开掉一段相抵）");
        for _empty_publish in 0..pool.geometry.retention_publishes() {
            pool.publish_empty().expect("空发布");
        }
        assert_eq!(pool.ended_intents[0].region_became_openable, Some(true), "{stop_arm:?}：Y1 真");
    }
}

#[test]
fn pinned_slot_put_into_the_region_after_issue_makes_the_intent_discarded_with_the_region_not_record_empty() {
    for stop_arm in [StopArm::LocalRegionPredicate, StopArm::GlobalDifference] {
        let mut pool = pool_with_a_two_unit_region(stop_arm);
        published_without_ending(pool.publish_compaction_batch(Some(2)));
        let pinned_offset = 2 * CLUSTER_SEGMENT_SLOTS + 12;
        pool.set_slot_state(pinned_offset, SlotState::Live { owner: SlotOwner::InstanceTable, allocation_txg: 0 });
        let outcome = pool.publish_compaction_batch(None).expect("第二批");
        assert_eq!(outcome, BatchOutcome::Published { ended_intent_index: Some(0) });
        let ended = pool.ended_intents[0];
        assert!(!ended.is_completed, "{stop_arm:?}：只剩钉住的 ⇒ 丢意图");
        assert!(!ended.region_is_record_empty, "{stop_arm:?}：P_R 假");
    }
}

#[test]
fn local_region_predicate_signs_no_next_intent_after_one_without_global_output() {
    let mut pool = pool_without_record_empty_segments(0);
    assert_eq!(pool.run_compaction(4), 1, "第一条意图在签发那一批里就收，P_G 假 ⇒ 不签下一条");
    assert_eq!(pool.ended_intents.len(), 1);
    assert!(!pool.ended_intents[0].global_difference_is_positive);
}

fn rows_with_deletion_blocks(per_round_publish_blocks: &[Vec<u64>]) -> Vec<RoundRow> {
    per_round_publish_blocks
        .iter()
        .enumerate()
        .map(|(round_index, blocks)| {
            let mut metrics = RoundMetrics::starting_at(0, 0, 0);
            metrics.deletion_publishes = blocks.len() as u64;
            metrics.deletion_commit_generated_block_sum = blocks.iter().sum();
            metrics.deletion_commit_generated_block_peak = blocks.iter().copied().max().unwrap_or(0);
            RoundRow {
                round: round_index as u64 + 1,
                metrics,
                free_runs: 0,
                live_user_units: 0,
                unrecoverable_slots: 0,
                record_empty_segments_at_end: 0,
            }
        })
        .collect()
}

#[test]
fn growth_judgement_compares_rounds_41_to_60_with_rounds_21_to_40() {
    let low_then_flat: Vec<Vec<u64>> = (1..=60).map(|round| vec![if round <= 20 { 10 } else { 20 }]).collect();
    assert_eq!(
        judge_deletion_block_growth(&rows_with_deletion_blocks(&low_then_flat), 1_000, &GROWTH_THRESHOLDS),
        GrowthVerdict::Bounded
    );
    let rising: Vec<Vec<u64>> = (1..=60).map(|round| vec![round]).collect();
    assert_eq!(
        judge_deletion_block_growth(&rows_with_deletion_blocks(&rising), 1_000, &GROWTH_THRESHOLDS),
        GrowthVerdict::Diverging
    );
    let silent_late: Vec<Vec<u64>> = (1..=60).map(|round| if round > 40 { Vec::new() } else { vec![5] }).collect();
    assert_eq!(
        judge_deletion_block_growth(&rows_with_deletion_blocks(&silent_late), 1_000, &GROWTH_THRESHOLDS),
        GrowthVerdict::Unreadable
    );
}

#[test]
fn moving_the_growth_thresholds_between_two_measured_values_flips_bounded_to_diverging() {
    // 21–40 轮每轮均值 150、41–60 轮 154.5（涨 3%）；峰值都是 200。
    let rounds: Vec<Vec<u64>> = (1..=60).map(|round| if round <= 40 { vec![200, 100] } else { vec![200, 109] }).collect();
    let rows = rows_with_deletion_blocks(&rounds);
    let loosened = GrowthThresholds { mean_growth_percent: 105, structural_bound_tenths: 9 };
    let between = GrowthThresholds { mean_growth_percent: 102, structural_bound_tenths: 9 };
    assert_eq!(judge_deletion_block_growth(&rows, 1_000, &loosened), GrowthVerdict::Bounded);
    assert_eq!(judge_deletion_block_growth(&rows, 1_000, &between), GrowthVerdict::Diverging);
    // 峰值 200：10 × 200 < 9 × 223 成立、< 9 × 222 不成立。
    assert_eq!(judge_deletion_block_growth(&rows, 223, &loosened), GrowthVerdict::Bounded);
    assert_eq!(judge_deletion_block_growth(&rows, 222, &loosened), GrowthVerdict::Diverging);
}

#[test]
fn the_doubled_check_turns_a_late_rise_into_diverging() {
    let late_rise: Vec<Vec<u64>> = (1..=120).map(|round| vec![if round > 100 { 30 } else { 20 }]).collect();
    assert_eq!(judge_doubled_growth_check(&rows_with_deletion_blocks(&late_rise)), GrowthVerdict::Diverging);
    let flat: Vec<Vec<u64>> = (1..=120).map(|_round| vec![20]).collect();
    assert_eq!(judge_doubled_growth_check(&rows_with_deletion_blocks(&flat)), GrowthVerdict::Bounded);
}

#[test]
fn five_seeds_combine_to_their_common_value_or_unstable() {
    assert_eq!(combine_seed_tags(&["bounded"; 5]), "bounded");
    assert_eq!(combine_seed_tags(&["bounded", "bounded", "diverging", "bounded", "bounded"]), "unstable");
    assert_eq!(across_sampling_points(&["none"; 7]), "none");
    assert_eq!(across_sampling_points(&["none", "none", "none", "failure", "none", "none", "none"]), "unstable");
}

#[test]
fn candidate_passes_one_cell_only_when_bounded_without_failures_and_the_control_degrades() {
    assert_eq!(candidate_cell_verdict(("bounded", "none"), true, false), "pass");
    assert_eq!(candidate_cell_verdict(("bounded", "none"), false, false), "not_judgeable", "F1：对照不退化");
    assert_eq!(candidate_cell_verdict(("bounded", "none"), true, true), "not_judgeable", "F3：打中不分辨臂");
    assert_eq!(candidate_cell_verdict(("diverging", "none"), true, false), "fail");
    assert_eq!(candidate_cell_verdict(("bounded", "failure"), false, false), "fail");
    assert_eq!(candidate_cell_verdict(("unstable", "none"), true, false), "not_judgeable");
    assert_eq!(candidate_cell_verdict(("unreadable", "none"), true, false), "not_judgeable");
    let degrading = CellArmVerdict { growth: "bounded", failures: "failure", growth_seeds: String::new(), failure_seeds: String::new() };
    let steady = CellArmVerdict { growth: "bounded", failures: "none", growth_seeds: String::new(), failure_seeds: String::new() };
    assert!(off_control_degrades(&degrading));
    assert!(!off_control_degrades(&steady));
}

#[test]
fn every_reverse_point_changes_exactly_one_knob_of_the_main_point() {
    for point in &SAMPLING_POINTS[1..] {
        let differing_knobs = [
            point.disk_slots != MAIN_SAMPLING_POINT.disk_slots,
            point.leaf_width_in_slots != MAIN_SAMPLING_POINT.leaf_width_in_slots,
            point.remounts_every_round != MAIN_SAMPLING_POINT.remounts_every_round,
            point.deletion_mode != MAIN_SAMPLING_POINT.deletion_mode,
            point.snapshot_limit != MAIN_SAMPLING_POINT.snapshot_limit,
            point.resident_base_multiplier != MAIN_SAMPLING_POINT.resident_base_multiplier,
        ];
        assert_eq!(differing_knobs.iter().filter(|differs| **differs).count(), 1, "{}", point.name);
    }
    assert_eq!(all_run_keys(None).len(), 7 * 9 * 4 * 2 * 5);
}

#[test]
fn publish_refused_after_rehearsal_leaves_every_counter_as_it_was() {
    let settings = PoolSettings { floor_segments: 0, is_compaction_enabled: true, stop_arm: StopArm::LocalRegionPredicate, resident_base_multiplier: 1_000 };
    let mut pool = Pool::made_by_make_filesystem(small_geometry(8, 4, 3), settings);
    pool.reclaim_for_the_next_publish();
    let slot_states_before = pool.slot_states.clone();
    let segments_before = pool.segments.clone();
    let sets_before = (pool.user_placement_segments.clone(), pool.fully_free_segments.clone());
    let counts_before = (pool.txg, pool.free_runs, pool.record_empty_segment_count, pool.units.len(), pool.reclaim_queue.len());
    let positions_before = (pool.leaf_slots.clone(), pool.root_slot, pool.open_cursor);
    assert_eq!(pool.publish_new_file(3), Err(PublishRefusal::ConjunctOne));
    assert_eq!(pool.slot_states, slot_states_before);
    assert_eq!(pool.segments, segments_before);
    assert_eq!((pool.user_placement_segments.clone(), pool.fully_free_segments.clone()), sets_before);
    assert_eq!((pool.txg, pool.free_runs, pool.record_empty_segment_count, pool.units.len(), pool.reclaim_queue.len()), counts_before);
    assert_eq!((pool.leaf_slots.clone(), pool.root_slot, pool.open_cursor), positions_before);
    pool.verify_incremental_counters_against_a_full_scan();
}

fn short_run_key(point: SamplingPoint, is_compaction_enabled: bool) -> RunKey {
    RunKey {
        point,
        root_ring_slots_per_region: 4,
        commit_generated_blocks_per_publish: 5,
        arm: FloorArm::One,
        is_compaction_enabled,
        seed: 3,
    }
}

/// 1 GiB 上把快照、按单元删、不重挂一起开着跑几轮：每轮末的全扫自检（V4）都过，同一种子两遍逐行相同（V1 的单测形态）。
#[test]
fn short_runs_keep_the_counters_consistent_and_repeat_exactly() {
    let exercising_point = SamplingPoint {
        name: "test_one_gibibyte_snapshots_units_no_remount",
        disk_slots: ONE_GIBIBYTE_DISK_SLOTS,
        leaf_width_in_slots: ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        remounts_every_round: false,
        deletion_mode: DeletionMode::RandomUnits,
        snapshot_limit: 2,
        resident_base_multiplier: 3,
    };
    let one_gibibyte_main = SAMPLING_POINTS[1];
    assert_eq!(one_gibibyte_main.name, "disk_1gib");
    for (point, is_compaction_enabled) in [(exercising_point, true), (exercising_point, false), (one_gibibyte_main, true)] {
        let mut rows_of_two_runs = Vec::new();
        for _repetition in 0..2 {
            let mut run = FillDeleteFillRun::new(short_run_key(point, is_compaction_enabled));
            for round in 1..=6 {
                run.run_round(round);
            }
            assert!(run.rows.iter().any(|row| row.metrics.deletion_publishes > 0), "删段真的发了删除类发布");
            assert!(run.rows[0].metrics.foreground_writes > 0, "第一轮的填满段写了");
            rows_of_two_runs.push((run.rows, run.pool.ended_intents));
        }
        assert_eq!(rows_of_two_runs[0], rows_of_two_runs[1], "{} 同一种子两遍逐行相同", point.name);
    }
}

/// `--merge` 从分片的 run 行取回的汇总，与跑的时候交给格一级的那一份逐项相同。
#[test]
fn run_summary_read_back_from_its_line_equals_the_one_handed_to_the_cells() {
    let key = RunKey {
        point: SAMPLING_POINTS[4],
        root_ring_slots_per_region: 8,
        commit_generated_blocks_per_publish: 10,
        arm: FloorArm::Formula,
        is_compaction_enabled: true,
        seed: 2,
    };
    let mut run = FillDeleteFillRun::new(key);
    for round in 1..=3 {
        run.run_round(round);
    }
    let finished = FinishedRun { key, rows: run.rows, ended_intents: run.pool.ended_intents, growth: GrowthVerdict::Unreadable };
    let (lines, summary) = lines_of_finished_run(&finished);
    let read_back: Vec<RunSummary> = lines.iter().filter_map(|line| run_summary_of_line(line)).collect();
    assert_eq!(read_back, vec![summary]);
    assert!(summary.intent_errors.ended > 0 || summary.deletion_failures == 0);
}
