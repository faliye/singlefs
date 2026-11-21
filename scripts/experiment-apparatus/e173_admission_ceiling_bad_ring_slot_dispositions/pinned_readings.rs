use super::*;

fn arm_of_the_pair(stuck_ceiling: StuckCeilingDisposition, all_zero_slot: AllZeroSlotDisposition) -> Arm {
    arm_of(stuck_ceiling, all_zero_slot)
}

fn fault_below_the_newest_root(history: &PoolAfterHistory, positions_below: u64, kind: ReadFaultKind, persistence: FaultPersistence) -> ReadFault {
    ReadFault { slot: history.geometry.slot_of_txg(history.highest_txg_used - positions_below), kind, persistence }
}

fn pool_of_the_control_cells(reserve_reading: ReserveReading, fullness: PoolFullness) -> PoolSettings {
    control_pool_settings(reserve_reading, fullness)
}

/// 场景：冻结树那份用例的两份盘面（S = 4）上，装置按每一条臂在锚点上做一次准入抬 F。
/// 预期：有洞读不出 6（K1）；没洞读不出 17 而不并 18（K2、K5）；没洞读回全零 18 越过 1 个、全零并 17（K3）；有洞读回全零或读得出 6（K4）；
/// 根环表里读回全零的槽钉在 5 与 9，读不出的槽能住的最大 txg 是 17 与 21（K6）。
#[test]
fn frozen_tree_use_case_cells_are_reproduced_on_every_arm() {
    let holes = events_of_the_pool_over_the_holes();
    let no_holes = events_of_the_pool_without_holes();
    for stuck_ceiling in StuckCeilingDisposition::every_arm() {
        let over_the_holes = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::DeviceError), ANCHOR_TXG_OVER_THE_HOLES, arm_of_the_pair(stuck_ceiling, AllZeroSlotDisposition::NotMerged));
        assert_eq!(over_the_holes.floor_after, 6, "K1 {stuck_ceiling:?}");
        assert_eq!(over_the_holes.slot_entry_before_the_raise, Some(Occupant::BadAtMount { oldest_txg_it_can_have_held: 9, highest_txg_it_can_hold: 21 }), "K1 的根环表");
        for zero in [AllZeroSlotDisposition::NotMerged, AllZeroSlotDisposition::Merged] {
            for kind in [Some(ReadFaultKind::AllZeroBytes), None] {
                let outcome = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, kind, ANCHOR_TXG_OVER_THE_HOLES, arm_of_the_pair(stuck_ceiling, zero));
                assert_eq!(outcome.floor_after, 6, "K4 {stuck_ceiling:?} {zero:?} {kind:?}");
            }
        }
    }
    for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes] {
        for stuck_ceiling in [StuckCeilingDisposition::AsToday, StuckCeilingDisposition::PushEmptyWithinBudget, StuckCeilingDisposition::PushEmptyBeyondBudget] {
            let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(kind), ANCHOR_TXG_WITHOUT_HOLES, arm_of_the_pair(stuck_ceiling, AllZeroSlotDisposition::NotMerged));
            assert_eq!(outcome.floor_after, 17, "K2 {kind:?} {stuck_ceiling:?}");
            assert_eq!(outcome.slot_entry_before_the_raise, Some(Occupant::BadAtMount { oldest_txg_it_can_have_held: 5, highest_txg_it_can_hold: 17 }), "K2 的根环表");
        }
    }
    for stuck_ceiling in [StuckCeilingDisposition::AsToday, StuckCeilingDisposition::PushEmptyWithinBudget, StuckCeilingDisposition::PushEmptyBeyondBudget] {
        let not_merged = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, arm_of_the_pair(stuck_ceiling, AllZeroSlotDisposition::NotMerged));
        assert_eq!((not_merged.floor_after, not_merged.crossed_non_empty_roots), (18, 1), "K3 全零不并 {stuck_ceiling:?}");
        assert!(matches!(not_merged.slot_entry_before_the_raise, Some(Occupant::AllZeroAtMount { oldest_txg_it_can_have_held: 5, .. })), "K6 没洞读回全零的槽钉在 5");
        let merged = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, arm_of_the_pair(stuck_ceiling, AllZeroSlotDisposition::Merged));
        assert_eq!((merged.floor_after, merged.crossed_non_empty_roots), (17, 0), "K3 全零并 {stuck_ceiling:?}");
    }
    for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes, ReadFaultKind::AllZeroBytes] {
        for zero in [AllZeroSlotDisposition::NotMerged, AllZeroSlotDisposition::Merged] {
            let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(kind), ANCHOR_TXG_WITHOUT_HOLES, arm_of_the_pair(StuckCeilingDisposition::DoNotMergeTheBadSlots, zero));
            assert_eq!(outcome.floor_after, 18, "K5 {kind:?} {zero:?}");
        }
    }
    let zero_over_the_holes = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_OVER_THE_HOLES, arm_of_the_pair(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged));
    assert!(matches!(zero_over_the_holes.slot_entry_before_the_raise, Some(Occupant::AllZeroAtMount { oldest_txg_it_can_have_held: 9, .. })), "K6 有洞读回全零的槽钉在 9");
}

/// 场景：B5 那一格（H1，S = 4，c = 5，P保留 = 45，近满 k = 0，不注读故障）挂载之后的第一次请求，八条臂各跑一次。
/// 预期：N = 50、写行 51、暖机 1 次、环 [41, 52]、待回收 127 块；不够 ⇒ 上限 47、抬 F 那一串 3 次、F 抬到 47、抬 F 期间空闲最低 30、回收 78 块
/// （待回收剩 127 − 78 + 15 + 13 = 77）、这一次请求在一串抬 F 之后发成、发成之后空闲 − P保留 = 50。
#[test]
fn first_request_of_the_crashed_near_full_control_cell_follows_the_independent_arithmetic_on_every_arm() {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
    let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
    for arm in every_arm_pair() {
        let mut session = start_a_session(&history, None, settings, arm, Controls::none(), false);
        assert_eq!((history.highest_txg_used, session.pool.current_txg, session.pool.space.pending_blocks()), (50, 52, 127), "B5 挂载之后 {arm:?}");
        assert_eq!(session.pool.truth_ring.iter().flatten().map(|root| root.checkpoint_txg).min(), Some(41), "环里最旧的根");
        let record = session.run_one_request();
        let raise = session.ledger.raises[0];
        assert!(record.accepted, "B5 发成 {arm:?}");
        assert_eq!((session.ledger.raises.len(), raise.truth_ceiling, raise.new_floor, raise.publishes), (1, 47, 47, 3), "B5 抬 F {arm:?}");
        assert_eq!(session.ledger.lowest_free_blocks_in_the_first_raise, Some(30), "B5 抬 F 期间空闲最低 {arm:?}");
        assert_eq!((record.free_above_the_reserve_after, session.pool.space.pending_blocks(), session.pool.current_txg), (50, 77, 56), "B5 发成之后 {arm:?}");
    }
}

/// 场景：抬 F 那一串与暖机的 txg 只由根环落点公式定。
/// 预期：抬 F 那一串按现行 txg mod 3 = 0 / 1 / 2 是 2 / 3 / 2 次；暖机按写行 txg mod 3 = 0 / 1 / 2 是 1 / 1 / 2 次；保留池条款读法 c_max = 5 / 10 时 45 / 80。
#[test]
fn raise_sequence_and_warm_up_lengths_and_reserve_follow_the_region_ownership_and_the_clause() {
    let raise_lengths: Vec<usize> = (0..3).map(|residue| txgs_of_the_publishes_carrying_the_floor_to_every_device(30 + residue).len()).collect();
    assert_eq!(raise_lengths, vec![2, 3, 2]);
    let warm_up_lengths: Vec<usize> = (0..3).map(|residue| warm_up_publish_txgs(60 + residue).len()).collect();
    assert_eq!(warm_up_lengths, vec![1, 1, 2]);
    assert_eq!(txgs_of_the_publishes_carrying_the_floor_to_every_device(31), vec![32, 33, 34]);
    assert_eq!(warm_up_publish_txgs(62), vec![63, 64]);
    assert_eq!(PUBLISHES_PER_ADMISSION_AT_MOST, 8);
    assert_eq!(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes.reserve_blocks(5), 45);
    assert_eq!(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes.reserve_blocks(10), 80);
    assert_eq!(ReserveReading::CodeReadingOneEmptyPublish.reserve_blocks(10), 10);
}

/// 场景：阳性对照 PC1–PC4（登记第五节），每条臂各跑一次。
/// 预期：PC1 把第一次抬 F 的目标抬到上限 + 2，越过 2 个非空根；PC3 把目标改成现行 txg，少掉 3 个状态；PC2 不回收，近满 k = 2、P保留 = c 时被拒 J − 2；
/// PC4 推空两臂推 5 次 25 块（读坏槽 j = 5，F 先抬到 N − 5），照今天与不并在种下「FloorAlreadyAtTheCeiling 先推一次」之后推 1 次 5 块。
#[test]
fn positive_controls_read_their_planted_effect_on_every_arm() {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
    let requests = i64::try_from(requests_per_mount(geometry)).expect("请求数");
    for arm in every_arm_pair() {
        let no_spare = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
        let pc1 = run_a_mount(&history, None, no_spare, arm, Controls { first_raise_target_above_the_ceiling_by: Some(2), ..Controls::none() }, false);
        assert_eq!(pc1.raises[0].crossed_non_empty_roots, 2, "PC1 {arm:?}");
        let pc3 = run_a_mount(&history, None, no_spare, arm, Controls { first_raise_target_is_the_current_txg: true, ..Controls::none() }, false);
        assert_eq!(pc3.raises[0].states_lost, 3, "PC3 {arm:?}");
        let two_spare = pool_of_the_control_cells(ReserveReading::CodeReadingOneEmptyPublish, PoolFullness::NearlyFullWithTwoSpareOverwrites);
        let pc2 = run_a_mount(&history, None, two_spare, arm, Controls { reclaim_disabled: true, ..Controls::none() }, false);
        assert_eq!(count_as_i64(rejected_count(&pc2)), requests - 2, "PC2 {arm:?}");
        let pushing = matches!(arm.stuck_ceiling, StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget);
        let (history_pc4, fault_pc4) = pc4_history_and_fault_for_the_empty_pushing_arms(if pushing { 45 } else { 47 }, pushing);
        let roomy = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::Roomy);
        let controls = Controls {
            first_request_is_judged_short_throughout: true,
            push_one_empty_publish_on_the_first_floor_already_at_the_ceiling: !pushing,
            ..Controls::none()
        };
        let record = start_a_session(&history_pc4, fault_pc4, roomy, arm, controls, false).run_one_request();
        let expected = if pushing { (5, 25) } else { (1, 5) };
        assert_eq!((record.empty_publishes, record.empty_publish_blocks), expected, "PC4 {arm:?}");
    }
}

/// 场景：H1，宽松池，F 先抬到 N − j，读坏槽 j（能住的最大 txg N − j），第一次请求每一次判够不够都当「不够」。
/// 预期：推空W 推的次数 = max(3S − j − 2, 0)（暖机 1 次）；推空B 受 B 限，= min(那个数, 7)；S = 4、8、16 的每个位置 j = 4..3S − 1 逐位置对上。
#[test]
fn empty_publish_counts_match_the_closed_form_at_every_position() {
    for slots_per_region in SLOTS_PER_REGION_CHOICES {
        let geometry = RingGeometry { slots_per_region };
        let ring_length = i64::try_from(geometry.ring_length_in_txgs()).expect("环长");
        let newest_txg = 2 + 4 * geometry.ring_length_in_txgs();
        for stuck_position in 4..ring_length {
            let mut events = HistoryFamily::CrashedAfterOverwrites.events(geometry);
            events.push(HistoryEvent::FloorRaisedWithoutBuildingTheSequence { new_floor: newest_txg - u64::try_from(stuck_position).expect("j") });
            let history = PoolAfterHistory::from_events(geometry, 5, &events);
            let fault = fault_below_the_newest_root(&history, u64::try_from(stuck_position).expect("j"), ReadFaultKind::DeviceError, FaultPersistence::UntilTheProcessWritesTheSlot);
            let roomy = PoolSettings { geometry, metadata_blocks: 5, reserve_reading: ReserveReading::CodeReadingOneEmptyPublish, fullness: PoolFullness::Roomy };
            let controls = Controls { first_request_is_judged_short_throughout: true, ..Controls::none() };
            let needed = u32::try_from((ring_length - stuck_position - 2).max(0)).expect("次数");
            let beyond = start_a_session(&history, Some(fault), roomy, arm_of(StuckCeilingDisposition::PushEmptyBeyondBudget, AllZeroSlotDisposition::NotMerged), controls, false).run_one_request();
            let within = start_a_session(&history, Some(fault), roomy, arm_of(StuckCeilingDisposition::PushEmptyWithinBudget, AllZeroSlotDisposition::NotMerged), controls, false).run_one_request();
            assert_eq!(beyond.empty_publishes, needed, "推空W S={slots_per_region} j={stuck_position}");
            assert_eq!(within.empty_publishes, needed.min(7), "推空B S={slots_per_region} j={stuck_position}");
        }
    }
}

/// 场景（M1）：H3（正常卸载，F = V = 50），读坏槽里住着 txg 50（j = 2，恰好是 V）。
/// 预期：读坏槽能住的最大 txg 50 不低于 F 50，照今天封在 50；读得出的根算出的 C0 是 51。
#[test]
fn bad_slot_that_can_hold_exactly_the_floor_still_caps_the_ceiling() {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::UnmountedAfterOverwrites.build_pool_after_history(geometry, 5);
    let fault = fault_below_the_newest_root(&history, 2, ReadFaultKind::DeviceError, FaultPersistence::UntilTheProcessWritesTheSlot);
    let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
    let session = start_a_session(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
    assert_eq!((history.highest_txg_used, history.effective_floor), (52, 50));
    let reading = session.pool.ceiling_pass(false).expect("没有知道住着根的槽读坏");
    assert_eq!((reading.unmerged, reading.merged), (51, 50));
}

/// 场景（M5）：H2，S = 4，e，P1，条款读法的近满 k = 0，推空B，j = 3（读坏槽能住 47 = F）。
/// 预期：B = 8 时第一次请求推 7 次空发布后被预算拒、第二次请求 F 抬到 50；B = 9 时第二次请求 F 抬到 53。
#[test]
fn the_admission_budget_of_eight_decides_how_far_the_floor_moves_in_the_second_request() {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::FloorRaisedAfterOverwrites.build_pool_after_history(geometry, 5);
    let fault = fault_below_the_newest_root(&history, 3, ReadFaultKind::DeviceError, FaultPersistence::UntilTheProcessWritesTheSlot);
    let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
    let run = run_a_mount(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::PushEmptyWithinBudget, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
    assert_eq!((run.requests[0].empty_publishes, run.requests[0].rejection, run.requests[0].floor_after), (7, Some(RejectionReason::AdmissionBudgetUsedUp), 47));
    assert_eq!((run.requests[1].rejection, run.requests[1].floor_after, run.requests[1].publishes_in_the_admission), (Some(RejectionReason::AdmissionBudgetUsedUp), 50, 5));
    let at_nine = run_a_mount(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::PushEmptyWithinBudget, AllZeroSlotDisposition::NotMerged), Controls { publishes_per_admission_at_most: 9, ..Controls::none() }, false);
    assert_eq!(at_nine.requests[1].floor_after, 53, "B = 9 时第二次请求 F 抬到 53，与 B = 8 不同");
}

/// 场景（M7）：推空B，H1 先抬 F 到 N − 5，读坏槽 j = 5（PC4 那一形），条款读法的近满 k = 0（空闲 = P保留 = 45）。
/// 预期：推空的空发布只要求空闲 ≥ c，不判空间准入式子（空闲 − P保留 ≥ c），所以第一次请求推 5 次空发布（25 块），再被预算拒。
#[test]
fn empty_publishes_for_a_stuck_ceiling_are_not_judged_by_the_admission_formula() {
    let (history, fault) = pc4_history_and_fault_for_the_empty_pushing_arms(45, true);
    let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
    let run = run_a_mount(&history, fault, settings, arm_of(StuckCeilingDisposition::PushEmptyWithinBudget, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
    let first = &run.requests[0];
    assert_eq!((first.empty_publishes, first.empty_publish_blocks, first.rejection), (5, 25, Some(RejectionReason::AdmissionBudgetUsedUp)));
    assert_eq!(run.empty_publish_refusals, 0);
}

/// 场景（M10）：H1，读坏槽 j = 10，持续读坏、写了也坏（P2），照今天，条款读法的近满 k = 0。
/// 预期：这个进程在暖机之前就写过 X 那一槽（它成了「知道住着根的槽」），算上限那一遍重读仍坏，抬 F 报自己的错：每一次请求都被这个拒因拒、F 一直是 0。
#[test]
fn slot_this_process_wrote_that_still_reads_bad_stops_every_floor_raise_with_its_own_error() {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, 5);
    let fault = fault_below_the_newest_root(&history, 10, ReadFaultKind::DeviceError, FaultPersistence::EvenAfterTheProcessWritesTheSlot);
    let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
    let run = run_a_mount(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
    for request in run.requests.iter().take(3) {
        assert_eq!((request.accepted, request.rejection, request.floor_after), (false, Some(RejectionReason::FloorRaiseReportedItsOwnError), 0));
    }
    assert_eq!(run.raises.len(), 0);
}

/// 场景（M8）：H1，S = 4，e，读坏槽 j = 5 只在挂载里读坏（T1），条款读法的近满 k = 2，照今天。
/// 预期：挂载做完时那一槽还钉着（可能住过的最旧 txg 9、能住的最大 txg 45），门槛压在 9、待回收 543 块；第一次发布记根时进程内重读读出有效根，
/// 换成它的 txg 45，门槛升到环里最旧有效根，待回收收到 127 块、空闲 487 块。
#[test]
fn rereading_a_pinned_slot_after_a_transient_fault_lifts_the_reclaim_floor() {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, 5);
    let fault = fault_below_the_newest_root(&history, 5, ReadFaultKind::DeviceError, FaultPersistence::DuringTheMountOnly);
    let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithTwoSpareOverwrites);
    let mut session = start_a_session(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
    assert_eq!(session.pool.table.occupants.get(&fault.slot), Some(&Occupant::BadAtMount { oldest_txg_it_can_have_held: 9, highest_txg_it_can_hold: 45 }));
    assert_eq!((session.pool.space.pending_blocks(), session.pool.space.free_blocks), (543, 71));
    let record = session.run_one_request();
    assert!(record.accepted);
    assert_eq!(session.pool.table.occupants.get(&fault.slot), Some(&Occupant::ValidRootReadWhenRereading { checkpoint_txg: 45 }));
    assert_eq!((session.pool.space.pending_blocks(), session.pool.space.free_blocks), (127, 487));
}

/// 场景：第二段 z 族的一组（H1，S = 4，c = 5，条款读法，近满 k = 0，P1，照今天）与 H5 的一组，两条 Q2 臂各跑。
/// 预期：H1–H4 每格 3S − 1 = 11 个位置；H5 一个位置；Q1 不并臂上两条 Q2 臂逐位置相同；同一组算两遍结果逐项相同（确定性模型，没有隐藏状态）。
#[test]
fn positions_per_cell_and_the_two_merge_arms_agree_where_the_no_merge_arm_never_merges() {
    let pool = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
    let all_zero_fault_group = GroupSettings {
        family: HistoryFamily::CrashedAfterOverwrites,
        fault_kind: Some(ReadFaultKind::AllZeroBytes),
        persistence: Some(FaultPersistence::UntilTheProcessWritesTheSlot),
        pool,
        stuck_ceiling: StuckCeilingDisposition::DoNotMergeTheBadSlots,
    };
    let first = run_a_group(&all_zero_fault_group);
    let second = run_a_group(&all_zero_fault_group);
    assert_eq!((first.not_merged.positions, first.not_merged.expected_positions), (11, 11));
    assert_eq!(first.pair.identical_positions, first.pair.positions, "不并臂上两条 Q2 臂相同");
    assert_eq!(
        (first.pair.not_merged_crossed_more, first.pair.merged_crossed_more, first.pair.not_merged_lost_more, first.pair.merged_lost_more),
        (0, 0, 0, 0),
        "逐位置相同时谁都不比谁多"
    );
    assert_eq!(first.not_merged.quantities, second.not_merged.quantities);
    let holes_group = GroupSettings { family: HistoryFamily::YoungRingWithCrashHoles, fault_kind: None, persistence: None, pool, stuck_ceiling: StuckCeilingDisposition::DoNotMergeTheBadSlots };
    let holes = run_a_group(&holes_group);
    assert_eq!((holes.not_merged.positions, holes.not_merged.expected_positions), (1, 1));
    assert_eq!(holes.pair.identical_positions, holes.pair.positions, "H5 上不并臂也不并任何槽");
}
