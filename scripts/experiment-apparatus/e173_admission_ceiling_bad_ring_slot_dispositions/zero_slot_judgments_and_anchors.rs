//! 全零槽的判定、控制池与锚点行、阳性对照、段的核对与各段的运行。

use super::*;

/// 第二段的判定：Q2-1（越过数与少掉的状态数，两条 Q2 臂比）、Q2-2（全零并的一次挂载全 ENOSPC，z 族与 H5 并排）、Q2-3（回退候选数）。
fn emit_the_second_segment_judgments(emitter: &mut Emitter, tally: &mut RunTally, results: &[GroupResult]) {
    emit_the_zero_slot_judgments(emitter, tally, "second", results);
}

/// Q2 的判定，`segment` 是产物行里的段名（原第二段 `second`；`checkpoint-cost-reserve` 模式里 `ckpt-second`）。
pub(super) fn emit_the_zero_slot_judgments(emitter: &mut Emitter, tally: &mut RunTally, segment: &str, results: &[GroupResult]) {
    let mut totals: BTreeMap<(&'static str, &'static str, u64, &'static str, i64, &'static str), ZeroSlotTotals> = BTreeMap::new();
    for result in results {
        let key = &result.not_merged.key;
        let scope = if key.fault_kind.is_some() { "z_h1_to_h4" } else { "h5_holes" };
        totals
            .entry((scope, key.arm.stuck_ceiling.label(), key.slots_per_region, key.reserve_reading.label(), key.metadata_blocks, key.reserve_reading.pool_size_label()))
            .or_insert_with(ZeroSlotTotals::start)
            .add(result);
    }
    const ZERO_SLOT_JUDGMENT_LABELS: [&str; 4] = ["Q2-1-not-merged-has-more", "Q2-1-merged-has-more", "Q2-2-merged-all-enospc", "Q2-2-merged-all-enospc-beyond-not-merged"];
    let mut hits_by_scope_and_arm: BTreeMap<(&'static str, &'static str), [(usize, usize); 4]> = BTreeMap::new();
    let mut observed_maxima: BTreeMap<(&'static str, &'static str), [i64; 4]> = BTreeMap::new();
    for ((scope, arm, slots_per_region, reserve, metadata_blocks, pool_size), total) in &totals {
        let pool_size_field = if pool_size.is_empty() { String::new() } else { format!(" pool_size={pool_size}") };
        let hit_not_merged_more = total.cells_where_not_merged_has_more >= 1;
        let hit_merged_more = total.cells_where_merged_has_more >= 1;
        // 登记第六节 Q2-2 的字面读法：全零并自己的一次挂载全 ENOSPC（对去掉读故障那一次）≥ 1 个位置；与全零不并并排看。
        let hit_merged_all_enospc = total.all_enospc_positions_merged >= 1;
        // 补一条（修订三）：全零并第一次多拒（全零不并那一次发成）之后一次都没发成的位置，两条 Q2 臂共有的卡死不算进来。
        let hit_merged_all_enospc_beyond_not_merged = total.positions_merged_is_all_enospc_against_not_merged >= 1;
        let maxima = observed_maxima.entry((*scope, *arm)).or_insert([0; 4]);
        for (index, value) in [
            count_as_i64(total.cells_where_not_merged_has_more),
            count_as_i64(total.cells_where_merged_has_more),
            count_as_i64(total.all_enospc_positions_merged),
            count_as_i64(total.positions_merged_is_all_enospc_against_not_merged),
        ]
        .into_iter()
        .enumerate()
        {
            maxima[index] = maxima[index].max(value);
        }
        let counters = hits_by_scope_and_arm.entry((*scope, *arm)).or_insert([(0, 0); 4]);
        for (index, hit) in [hit_not_merged_more, hit_merged_more, hit_merged_all_enospc, hit_merged_all_enospc_beyond_not_merged].into_iter().enumerate() {
            counters[index].0 += usize::from(hit);
            counters[index].1 += 1;
        }
        emit(
            emitter,
            format!(
                "name=q2_comparison segment={segment} scope={scope} arm={arm} S={slots_per_region} reserve={reserve} c_max={metadata_blocks}{pool_size_field} cells={} \
                 cells_where_not_merged_has_more={} cells_where_merged_has_more={} largest_crossed_excess_of_not_merged={} largest_lost_excess_of_not_merged={} \
                 crossed_maximum_not_merged={} crossed_maximum_merged={} lost_maximum_not_merged={} lost_maximum_merged={} \
                 all_enospc_positions_not_merged={} all_enospc_positions_merged={} extra_rejected_maximum_not_merged={} extra_rejected_maximum_merged={} \
                 positions_merged_is_all_enospc_against_not_merged={} positions_merged_rejects_more={} largest_rejected_excess_of_merged={} \
                 lowest_candidates_not_merged={} lowest_candidates_merged={} lowest_final_candidates_not_merged={} lowest_final_candidates_merged={} \
                 hit_not_merged_has_more={} hit_merged_has_more={} hit_merged_all_enospc={} hit_merged_all_enospc_beyond_not_merged={}",
                total.cells,
                total.cells_where_not_merged_has_more,
                total.cells_where_merged_has_more,
                total.largest_crossed_excess_of_not_merged,
                total.largest_lost_excess_of_not_merged,
                total.crossed_maximum_not_merged,
                total.crossed_maximum_merged,
                total.lost_maximum_not_merged,
                total.lost_maximum_merged,
                total.all_enospc_positions_not_merged,
                total.all_enospc_positions_merged,
                total.extra_rejected_maximum_not_merged,
                total.extra_rejected_maximum_merged,
                total.positions_merged_is_all_enospc_against_not_merged,
                total.positions_merged_rejects_more,
                total.largest_rejected_excess_of_merged,
                total.lowest_candidates_not_merged,
                total.lowest_candidates_merged,
                total.lowest_final_candidates_not_merged,
                total.lowest_final_candidates_merged,
                yes_no(hit_not_merged_more),
                yes_no(hit_merged_more),
                yes_no(hit_merged_all_enospc),
                yes_no(hit_merged_all_enospc_beyond_not_merged)
            ),
        );
    }
    // 判别力自证（登记第八节）：门槛从 1 挪到「观测到的最大值 + 1」，打中必须由真转假（按 (判定, 范围) 各一组，臂名当键）。
    for (index, judgment_label) in ZERO_SLOT_JUDGMENT_LABELS.iter().enumerate() {
        for scope in ["z_h1_to_h4", "h5_holes"] {
            let per_arm: BTreeMap<&'static str, (usize, usize, Option<i64>)> = hits_by_scope_and_arm
                .iter()
                .filter(|((hit_scope, _), _)| *hit_scope == scope)
                .map(|((_, arm), counters)| (*arm, (counters[index].0, counters[index].1, Some(observed_maxima[&(scope, *arm)][index]))))
                .collect();
            emit_the_threshold_shift_check(emitter, tally, segment, &format!("{judgment_label}@{scope}"), &per_arm);
        }
    }
    for ((scope, arm), counters) in &hits_by_scope_and_arm {
        for (index, judgment_label) in ZERO_SLOT_JUDGMENT_LABELS.iter().enumerate() {
            let (hit_points, points) = counters[index];
            let stability = if hit_points == points { "stable_hit" } else if hit_points == 0 { "stable_no_hit" } else { "unstable" };
            emit(
                emitter,
                format!("name=stability segment={segment} id={judgment_label} scope={scope} arm={arm} hit_points={hit_points} points={points} stability={stability}"),
            );
        }
    }
}

// ── 对拍锚点：装置与冻结树 `crates/` 逐格对（登记第十一节停机条款 K1–K6） ─────────────

pub(super) const SLOTS_PER_REGION_OF_THE_CONTROL_POOLS: u64 = 4;
pub(super) const METADATA_BLOCKS_OF_THE_CONTROL_POOLS: i64 = 5;

pub(super) fn geometry_of_the_control_pools() -> RingGeometry {
    RingGeometry { slots_per_region: SLOTS_PER_REGION_OF_THE_CONTROL_POOLS }
}

pub(super) fn control_pool_settings(reserve_reading: ReserveReading, fullness: PoolFullness) -> PoolSettings {
    PoolSettings {
        geometry: geometry_of_the_control_pools(),
        metadata_blocks: METADATA_BLOCKS_OF_THE_CONTROL_POOLS,
        reserve_reading,
        fullness,
    }
}

/// 没洞那一份（用例文件头）：实例 1 覆盖写到 txg 17（第一个文件在 3，之后 15 次覆盖写到 17）。
pub(super) fn events_of_the_pool_without_holes() -> Vec<HistoryEvent> {
    let mut events = vec![HistoryEvent::WritableMount];
    events.extend(std::iter::repeat_n(HistoryEvent::Overwrite, 15));
    events
}

/// 有洞那一份：实例 1 覆盖写到 17，崩三次留洞 18、19、20，再挂写 X = 21（写行）与暖机、再空发布两次，正常退出。
pub(super) fn events_of_the_pool_over_the_holes() -> Vec<HistoryEvent> {
    let mut events = events_of_the_pool_without_holes();
    events.extend(std::iter::repeat_n(HistoryEvent::MountCrashedBeforeItsRowRoot, 3));
    events.push(HistoryEvent::WritableMount);
    events.extend(std::iter::repeat_n(HistoryEvent::EmptyPublish, 2));
    events
}

pub(super) const FAULT_TXG_WITHOUT_HOLES: u64 = 17;
pub(super) const ANCHOR_TXG_WITHOUT_HOLES: u64 = 28;
pub(super) const FAULT_TXG_OVER_THE_HOLES: u64 = 21;
pub(super) const ANCHOR_TXG_OVER_THE_HOLES: u64 = 29;

/// 在锚点上做一次准入抬 F 的读数：被测挂载起 X 那一槽按 `fault_kind` 读坏，空发布推到 `anchor_txg`，再调一次准入抬 F（宽松池，只看上限）。
#[derive(Clone, Debug)]
pub(super) struct AnchorRaise {
    unmerged_ceiling: u64,
    merged_ceiling: u64,
    pub(super) floor_after: u64,
    pub(super) crossed_non_empty_roots: u32,
    pushed: bool,
    stop: Option<RejectionReason>,
    pub(super) slot_entry_before_the_raise: Option<Occupant>,
    pub(super) trace: Vec<String>,
}

pub(super) fn raise_at_an_anchor(events: &[HistoryEvent], fault_txg: u64, fault_kind: Option<ReadFaultKind>, anchor_txg: u64, arm: Arm) -> AnchorRaise {
    let geometry = geometry_of_the_control_pools();
    let history = PoolAfterHistory::from_events(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS, events);
    let fault_slot = geometry.slot_of_txg(fault_txg);
    let fault = fault_kind.map(|kind| ReadFault { slot: fault_slot, kind, persistence: FaultPersistence::UntilTheProcessWritesTheSlot });
    let settings = control_pool_settings(ReserveReading::CodeReadingOneEmptyPublish, PoolFullness::Roomy);
    let mut session = start_a_session(&history, fault, settings, arm, Controls::none(), true);
    let slot_entry_before_the_raise = session.pool.table.occupants.get(&fault_slot).copied();
    while session.pool.current_txg < anchor_txg {
        session.publish_an_empty_root();
    }
    let reading = session.pool.ceiling_pass(session.merges_all_zero_slots()).expect("锚点上没有知道住着根的槽读坏");
    session.note_the_cap(reading);
    let mut tally = RequestTally { pushed_within_the_budget: 0, empty_publishes: 0, empty_publish_blocks: 0, raise_publishes: 0, pushed_an_empty_publish: false };
    let outcome = session.push_one_floor_raise_within_the_budget(&mut tally, Some(reading));
    let (pushed, stop) = match outcome {
        StepOutcome::Pushed => (true, None),
        StepOutcome::Stopped(reason) => (false, Some(reason)),
    };
    AnchorRaise {
        unmerged_ceiling: reading.unmerged,
        merged_ceiling: reading.merged,
        floor_after: session.pool.floor_in_the_current_version,
        crossed_non_empty_roots: session.ledger.raises.first().map_or(0, |raise| raise.crossed_non_empty_roots),
        pushed,
        stop,
        slot_entry_before_the_raise,
        trace: session.ledger.trace,
    }
}

pub(super) fn arm_of(stuck_ceiling: StuckCeilingDisposition, all_zero_slot: AllZeroSlotDisposition) -> Arm {
    Arm { stuck_ceiling, all_zero_slot }
}

fn render_an_optional_occupant(occupant: Option<Occupant>) -> String {
    match occupant {
        None => "none".to_string(),
        Some(Occupant::ValidRoot { checkpoint_txg }) => format!("valid_root_{checkpoint_txg}"),
        Some(Occupant::ValidRootReadWhenRereading { checkpoint_txg }) => format!("valid_root_reread_{checkpoint_txg}"),
        Some(Occupant::BadAtMount { oldest_txg_it_can_have_held, highest_txg_it_can_hold }) => {
            format!("bad_at_mount_oldest_{oldest_txg_it_can_have_held}_highest_{highest_txg_it_can_hold}")
        }
        Some(Occupant::AllZeroAtMount { oldest_txg_it_can_have_held, highest_txg_it_can_hold }) => {
            format!("all_zero_oldest_{oldest_txg_it_can_have_held}_highest_{highest_txg_it_can_hold}")
        }
    }
}

/// 一格对拍锚点：期望值是冻结树上那份用例钉着的值（登记第十一节表），`expected_ceiling` 与 `expected_crossed` 为 `None` 时只报不判。
#[allow(clippy::too_many_arguments, reason = "这些参数是这一次计算彼此独立的输入，合成只用一次的结构体只多一层包装")]
fn emit_an_anchor_cell(
    emitter: &mut Emitter,
    tally: &mut RunTally,
    anchor_label: &str,
    description: &str,
    outcome: &AnchorRaise,
    arm: Arm,
    expected_floor: Option<u64>,
    expected_crossed: Option<u32>,
) {
    let ceiling_of_this_arm = match arm.stuck_ceiling {
        StuckCeilingDisposition::DoNotMergeTheBadSlots => outcome.unmerged_ceiling,
        StuckCeilingDisposition::AsToday | StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget => {
            outcome.merged_ceiling
        }
    };
    let floor_matches = expected_floor.is_none_or(|expected| outcome.floor_after == expected && ceiling_of_this_arm == expected);
    let crossed_matches = expected_crossed.is_none_or(|expected| outcome.crossed_non_empty_roots == expected);
    tally.halt_check(floor_matches && crossed_matches && outcome.pushed, &format!("{anchor_label}_{}_{}", arm.stuck_ceiling.label(), arm.all_zero_slot.label()));
    emit(
        emitter,
        format!(
            "name=anchor id={anchor_label} description={description} q1={} q2={} c0={} c_merged={} floor_after={} crossed={} pushed={} stop={} expected_floor={} expected_crossed={} slot_entry={} matches={}",
            arm.stuck_ceiling.label(),
            arm.all_zero_slot.label(),
            outcome.unmerged_ceiling,
            outcome.merged_ceiling,
            outcome.floor_after,
            outcome.crossed_non_empty_roots,
            yes_no(outcome.pushed),
            outcome.stop.map_or("none", RejectionReason::label),
            expected_floor.map_or("none".to_string(), |value| value.to_string()),
            expected_crossed.map_or("none".to_string(), |value| value.to_string()),
            render_an_optional_occupant(outcome.slot_entry_before_the_raise),
            yes_no(floor_matches && crossed_matches && outcome.pushed),
        ),
    );
    for (step, line) in outcome.trace.iter().enumerate() {
        emit(emitter, format!("name=trace id={anchor_label} q1={} q2={} step={step} {line}", arm.stuck_ceiling.label(), arm.all_zero_slot.label()));
    }
}

fn emit_a_table_entry_anchor(emitter: &mut Emitter, tally: &mut RunTally, anchor_label: &str, entry: Option<Occupant>, expected_oldest: u64, expected_highest: Option<u64>) {
    let matches = match entry {
        Some(Occupant::BadAtMount { oldest_txg_it_can_have_held, highest_txg_it_can_hold }) => {
            oldest_txg_it_can_have_held == expected_oldest && Some(highest_txg_it_can_hold) == expected_highest
        }
        Some(Occupant::AllZeroAtMount { oldest_txg_it_can_have_held, .. }) => oldest_txg_it_can_have_held == expected_oldest && expected_highest.is_none(),
        Some(Occupant::ValidRoot { .. } | Occupant::ValidRootReadWhenRereading { .. }) | None => false,
    };
    tally.halt_check(matches, anchor_label);
    emit(
        emitter,
        format!(
            "name=anchor id={anchor_label} slot_entry={} expected_oldest={expected_oldest} expected_highest={} matches={}",
            render_an_optional_occupant(entry),
            expected_highest.map_or("none".to_string(), |value| value.to_string()),
            yes_no(matches)
        ),
    );
}

/// 登记第十一节 K1–K6：装置在冻结树那份用例的两份盘面上给的值与用例钉着的逐格对。K7（结构）由执行员读冻结树拷贝里的行逐项写进报告。
fn emit_the_frozen_tree_anchors(emitter: &mut Emitter, tally: &mut RunTally) {
    let holes = events_of_the_pool_over_the_holes();
    let no_holes = events_of_the_pool_without_holes();
    let not_merged = AllZeroSlotDisposition::NotMerged;
    let merged = AllZeroSlotDisposition::Merged;
    for stuck_ceiling in StuckCeilingDisposition::every_arm() {
        let arm = arm_of(stuck_ceiling, not_merged);
        let outcome = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::DeviceError), ANCHOR_TXG_OVER_THE_HOLES, arm);
        emit_an_anchor_cell(emitter, tally, "K1", "有洞那一份 X 读不出：上限 6、F 抬到 6", &outcome, arm, Some(6), None);
    }
    for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes] {
        for stuck_ceiling in [StuckCeilingDisposition::AsToday, StuckCeilingDisposition::PushEmptyWithinBudget, StuckCeilingDisposition::PushEmptyBeyondBudget] {
            let arm = arm_of(stuck_ceiling, not_merged);
            let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(kind), ANCHOR_TXG_WITHOUT_HOLES, arm);
            emit_an_anchor_cell(emitter, tally, &format!("K2_{}", kind.label()), "没洞那一份 X 读不出：上限 17", &outcome, arm, Some(17), None);
        }
    }
    for stuck_ceiling in [StuckCeilingDisposition::AsToday, StuckCeilingDisposition::PushEmptyWithinBudget, StuckCeilingDisposition::PushEmptyBeyondBudget] {
        let arm = arm_of(stuck_ceiling, not_merged);
        let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, arm);
        emit_an_anchor_cell(emitter, tally, "K3", "没洞那一份 X 读回全零、全零不并：上限 18、越过 X（越过数 1）", &outcome, arm, Some(18), Some(1));
        let merged_arm = arm_of(stuck_ceiling, merged);
        let merged_outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, merged_arm);
        emit_an_anchor_cell(emitter, tally, "K3", "没洞那一份 X 读回全零、全零并：上限 17", &merged_outcome, merged_arm, Some(17), Some(0));
    }
    for stuck_ceiling in StuckCeilingDisposition::every_arm() {
        for zero in [not_merged, merged] {
            let arm = arm_of(stuck_ceiling, zero);
            for kind in [Some(ReadFaultKind::AllZeroBytes), None] {
                let outcome = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, kind, ANCHOR_TXG_OVER_THE_HOLES, arm);
                emit_an_anchor_cell(emitter, tally, "K4", "有洞那一份 X 读回全零或读得出：上限 6", &outcome, arm, Some(6), None);
            }
        }
    }
    for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes, ReadFaultKind::AllZeroBytes] {
        for zero in [not_merged, merged] {
            let arm = arm_of(StuckCeilingDisposition::DoNotMergeTheBadSlots, zero);
            let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(kind), ANCHOR_TXG_WITHOUT_HOLES, arm);
            emit_an_anchor_cell(emitter, tally, &format!("K5_{}", kind.label()), "没洞那一份不并：上限 18", &outcome, arm, Some(18), None);
        }
    }
    let arm = arm_of(StuckCeilingDisposition::AsToday, not_merged);
    let zero_no_holes = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, arm);
    emit_a_table_entry_anchor(emitter, tally, "K6_no_holes", zero_no_holes.slot_entry_before_the_raise, 5, None);
    let zero_holes = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_OVER_THE_HOLES, arm);
    emit_a_table_entry_anchor(emitter, tally, "K6_holes", zero_holes.slot_entry_before_the_raise, 9, None);
    let bad_no_holes = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::DeviceError), ANCHOR_TXG_WITHOUT_HOLES, arm);
    emit_a_table_entry_anchor(emitter, tally, "K2_table_no_holes", bad_no_holes.slot_entry_before_the_raise, 5, Some(17));
    let bad_holes = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::DeviceError), ANCHOR_TXG_OVER_THE_HOLES, arm);
    emit_a_table_entry_anchor(emitter, tally, "K1_table_holes", bad_holes.slot_entry_before_the_raise, 9, Some(21));
}

// ── 第七节的锚点：条款本身（甲）与独立算出的（乙） ─────────────────────────────────────

pub(super) fn emit_a_check(emitter: &mut Emitter, tally: &mut RunTally, is_clause_check: bool, check_label: &str, observed: String, expected: String) {
    let matches = observed == expected;
    if is_clause_check {
        tally.halt_check(matches, check_label);
    } else {
        tally.self_check(matches, check_label);
    }
    emit(emitter, format!("name=anchor id={check_label} observed={observed} expected={expected} matches={}", yes_no(matches)));
}

/// 甲类 A1–A5（出自条款，不符走 F3 停机）与乙类 B1、B2、B4（`anchors.py` 现算的数，不符作废 V4）。
fn emit_the_structural_anchors(emitter: &mut Emitter, tally: &mut RunTally) {
    emit_a_check(emitter, tally, true, "A1", PUBLISHES_PER_ADMISSION_AT_MOST.to_string(), "8".to_string());
    for (metadata_blocks, expected) in [(5, 45), (10, 80)] {
        emit_a_check(
            emitter,
            tally,
            true,
            &format!("A2_c{metadata_blocks}"),
            ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes.reserve_blocks(metadata_blocks).to_string(),
            expected.to_string(),
        );
    }
    let observed_raise_lengths: Vec<usize> = (0..3).map(|residue| txgs_of_the_publishes_carrying_the_floor_to_every_device(30 + residue).len()).collect();
    emit_a_check(emitter, tally, true, "A3_raise_sequence_by_txg_mod_3", format!("{observed_raise_lengths:?}"), "[2, 3, 2]".to_string());
    emit_a_check(emitter, tally, true, "B1_matches_A3", format!("{observed_raise_lengths:?}"), "[2, 3, 2]".to_string());
    let slots_are_in_the_format_range = SLOTS_PER_REGION_CHOICES.iter().all(|slots| (TOLERATED_ROOT_SLOT_WRITE_FAILURES as u64 + 2..=16).contains(slots));
    emit_a_check(emitter, tally, true, "A4_slots_per_region_in_4_to_16", yes_no(slots_are_in_the_format_range).to_string(), "yes".to_string());
    let warm_up_lengths: Vec<usize> = (0..3).map(|residue| warm_up_publish_txgs(60 + residue).len()).collect();
    let warm_up_is_at_most_the_regions = warm_up_lengths.iter().all(|length| *length <= 3);
    emit_a_check(emitter, tally, true, "A5_warm_up_at_most_three", yes_no(warm_up_is_at_most_the_regions).to_string(), "yes".to_string());
    emit_a_check(emitter, tally, false, "B2_warm_up_by_row_mod_3", format!("{warm_up_lengths:?}"), "[1, 1, 2]".to_string());
    for (metadata_blocks, expected_empties, expected_reserve) in [(5i64, 35i64, 45i64), (10, 70, 80)] {
        let empties = i64::try_from(PUBLISHES_PER_ADMISSION_AT_MOST - 1).expect("B") * metadata_blocks;
        emit_a_check(emitter, tally, false, &format!("B4_c{metadata_blocks}_seven_empties"), empties.to_string(), expected_empties.to_string());
        emit_a_check(
            emitter,
            tally,
            false,
            &format!("B4_c{metadata_blocks}_reserve"),
            ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes.reserve_blocks(metadata_blocks).to_string(),
            expected_reserve.to_string(),
        );
    }
}

pub(super) fn every_arm_pair() -> Vec<Arm> {
    let mut arms = Vec::new();
    for stuck_ceiling in StuckCeilingDisposition::every_arm() {
        for all_zero_slot in [AllZeroSlotDisposition::NotMerged, AllZeroSlotDisposition::Merged] {
            arms.push(arm_of(stuck_ceiling, all_zero_slot));
        }
    }
    arms
}

pub(super) fn label_of_the_arm(arm: Arm) -> String {
    format!("{}_{}", arm.stuck_ceiling.label(), arm.all_zero_slot.label())
}

/// B5：H1，S = 4，c = 5，P保留 = 45，近满 k = 0，不注读故障，第一次请求（八条臂都等于这几个数，逐步打印轨迹）。
fn emit_the_first_request_anchor(emitter: &mut Emitter, tally: &mut RunTally) {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
    let settings = control_pool_settings(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
    for arm in every_arm_pair() {
        let mut session = start_a_session(&history, None, settings, arm, Controls::none(), true);
        let ring_oldest = session.pool.truth_ring.iter().flatten().map(|root| root.checkpoint_txg).min().expect("环里有根");
        let facts = format!(
            "N={} row={} warmup={} ring=[{ring_oldest},{}] pending_total={}",
            history.highest_txg_used,
            history.highest_txg_used + 1,
            session.pool.current_txg - (history.highest_txg_used + 1),
            session.pool.current_txg,
            session.pool.space.pending_blocks()
        );
        emit_a_check(emitter, tally, false, &format!("B5_mount_facts_{}", label_of_the_arm(arm)), facts, "N=50 row=51 warmup=1 ring=[41,52] pending_total=127".to_string());
        let record = session.run_one_request();
        let raise = session.ledger.raises.first().copied();
        let after = format!(
            "accepted={} raises={} ceiling={} sequence_publishes={} floor={} lowest_free_in_the_raise={:?} free_above_reserve_after={} pending_after={} txg_after={}",
            record.accepted,
            session.ledger.raises.len(),
            raise.map_or(0, |raise| raise.truth_ceiling),
            raise.map_or(0, |raise| raise.publishes),
            record.floor_after,
            session.ledger.lowest_free_blocks_in_the_first_raise,
            record.free_above_the_reserve_after,
            session.pool.space.pending_blocks(),
            session.pool.current_txg
        );
        // 回收 78 块：127 − 78 + 抬 F 那一串 3 次 × 5 + 用户发布放掉的 13 = 77。
        emit_a_check(
            emitter,
            tally,
            false,
            &format!("B5_first_request_{}", label_of_the_arm(arm)),
            after,
            "accepted=true raises=1 ceiling=47 sequence_publishes=3 floor=47 lowest_free_in_the_raise=Some(30) free_above_reserve_after=50 pending_after=77 txg_after=56".to_string(),
        );
        for (step, line) in session.ledger.trace.iter().enumerate() {
            emit(emitter, format!("name=trace id=B5 arm={} step={step} {line}", label_of_the_arm(arm)));
        }
    }
}

/// B6：宽松池、不注读故障，每条臂被拒 0 次（H1–H4 × S × c_max × P保留的每一格）。
fn emit_the_roomy_pool_anchor(emitter: &mut Emitter, tally: &mut RunTally) {
    let mut cells_checked = 0usize;
    let mut cells_with_a_rejection = 0usize;
    for pool in pool_settings_list().into_iter().filter(|pool| pool.fullness == PoolFullness::Roomy) {
        for family in FAMILIES_WITH_A_FAULT_TO_INJECT {
            let history = family.build_pool_after_history(pool.geometry, pool.metadata_blocks);
            for arm in every_arm_pair() {
                let run = run_a_mount(&history, None, pool, arm, Controls::none(), false);
                cells_checked += 1;
                cells_with_a_rejection += usize::from(rejected_count(&run) > 0);
            }
        }
    }
    emit_a_check(emitter, tally, false, "B6_roomy_pool_no_fault_never_rejects", format!("cells={cells_checked} cells_with_a_rejection={cells_with_a_rejection}"), format!("cells={cells_checked} cells_with_a_rejection=0"));
}

// ── 阳性对照：对每一条臂各跑一次，期望值写死（登记第五节「阳性对照」，失败条款 F1） ────────────

/// PC4 推空两臂的盘面：H1，S = 4（N = 50，暖机 1 次），挂载之前 F 已抬到 N − 5，e，P1，j = 5（读坏槽里住着 txg 45），宽松池。
pub(super) fn pc4_history_and_fault_for_the_empty_pushing_arms(floor_before_the_mount: u64, with_fault: bool) -> (PoolAfterHistory, Option<ReadFault>) {
    let geometry = geometry_of_the_control_pools();
    let mut events = HistoryFamily::CrashedAfterOverwrites.events(geometry);
    events.push(HistoryEvent::FloorRaisedWithoutBuildingTheSequence { new_floor: floor_before_the_mount });
    let history = PoolAfterHistory::from_events(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS, &events);
    let fault = with_fault.then(|| ReadFault {
        slot: geometry.slot_of_txg(history.highest_txg_used - 5),
        kind: ReadFaultKind::DeviceError,
        persistence: FaultPersistence::UntilTheProcessWritesTheSlot,
    });
    (history, fault)
}

fn emit_the_positive_controls(emitter: &mut Emitter, tally: &mut RunTally) {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
    let requests = i64::try_from(requests_per_mount(geometry)).expect("请求数");
    for arm in every_arm_pair() {
        let label = label_of_the_arm(arm);
        // 登记写的是近满 k = 8；单测读数：k = 8 的池在 J 次请求里一次抬 F 都不走（回收随环转动补回每次请求花的块），对照打不出效应，
        // 所以 PC1、PC3 取第一次请求就走抬 F 的近满 k = 0（修订一），k = 8 那一格只报它走了几次抬 F。
        let eight_spare = control_pool_settings(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithEightSpareOverwritesForControlsOnly);
        let registered_pool_run = run_a_mount(&history, None, eight_spare, arm, Controls::none(), false);
        emit(emitter, format!("name=anchor id=PC1_PC3_registered_k8_pool_floor_raises arm={label} observed={} note=registered_pool_does_not_reach_the_raise_path", registered_pool_run.raises.len()));
        let no_spare = control_pool_settings(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
        let controls_pc1 = Controls { first_raise_target_above_the_ceiling_by: Some(2), ..Controls::none() };
        let pc1 = run_a_mount(&history, None, no_spare, arm, controls_pc1, false);
        emit_a_check(emitter, tally, false, &format!("PC1_crossed_by_two_{label}"), pc1.raises.first().map_or(-1, |raise| i64::from(raise.crossed_non_empty_roots)).to_string(), "2".to_string());
        let controls_pc3 = Controls { first_raise_target_is_the_current_txg: true, ..Controls::none() };
        let pc3 = run_a_mount(&history, None, no_spare, arm, controls_pc3, false);
        emit_a_check(emitter, tally, false, &format!("PC3_states_lost_{label}"), pc3.raises.first().map_or(-1, |raise| count_as_i64(raise.states_lost)).to_string(), "3".to_string());
        let two_spare_code_reserve = control_pool_settings(ReserveReading::CodeReadingOneEmptyPublish, PoolFullness::NearlyFullWithTwoSpareOverwrites);
        let controls_pc2 = Controls { reclaim_disabled: true, ..Controls::none() };
        let pc2 = run_a_mount(&history, None, two_spare_code_reserve, arm, controls_pc2, false);
        emit_a_check(emitter, tally, false, &format!("PC2_rejected_{label}"), count_as_i64(rejected_count(&pc2)).to_string(), (requests - 2).to_string());
        let pushing = matches!(arm.stuck_ceiling, StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget);
        let (history_pc4, fault_pc4) = if pushing {
            pc4_history_and_fault_for_the_empty_pushing_arms(45, true)
        } else {
            pc4_history_and_fault_for_the_empty_pushing_arms(47, false)
        };
        let roomy = control_pool_settings(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::Roomy);
        let controls_pc4 = Controls {
            first_request_is_judged_short_throughout: true,
            push_one_empty_publish_on_the_first_floor_already_at_the_ceiling: !pushing,
            ..Controls::none()
        };
        let mut session = start_a_session(&history_pc4, fault_pc4, roomy, arm, controls_pc4, true);
        let record = session.run_one_request();
        let (expected_empties, expected_blocks) = if pushing { (5, 25) } else { (1, 5) };
        emit_a_check(
            emitter,
            tally,
            false,
            &format!("PC4_empty_publishes_{label}"),
            format!("{} {}", record.empty_publishes, record.empty_publish_blocks),
            format!("{expected_empties} {expected_blocks}"),
        );
        for (step, line) in session.ledger.trace.iter().enumerate() {
            emit(emitter, format!("name=trace id=PC4 arm={label} step={step} {line}"));
        }
    }
}

/// B3 与 F4：单个读坏槽 j（能住的最大 txg N − j）暖机之后还要推 D = 3S − j − 1 − w 次才被写掉；推空W 在 PC4 那一形（F 先抬到 N − j）上推的次数 = max(D, 0)，
/// 推空B 受 B 限，推的次数 = min(max(D, 0), B − 1)。H1 的 w = 1（N = 2 + 12S，写行 txg ≡ 0 mod 3）。
fn emit_the_empty_push_count_anchor(emitter: &mut Emitter, tally: &mut RunTally) {
    for slots_per_region in SLOTS_PER_REGION_CHOICES {
        let geometry = RingGeometry { slots_per_region };
        let ring_length = i64::try_from(geometry.ring_length_in_txgs()).expect("环长");
        let base_events = HistoryFamily::CrashedAfterOverwrites.events(geometry);
        let mut mismatches_beyond_budget = Vec::new();
        let mut mismatches_within_budget = Vec::new();
        let mut positions = 0;
        for stuck_position in 4..ring_length {
            let probe_history = {
                let mut events = base_events.clone();
                let newest_txg = 2 + 4 * geometry.ring_length_in_txgs();
                events.push(HistoryEvent::FloorRaisedWithoutBuildingTheSequence { new_floor: newest_txg - u64::try_from(stuck_position).expect("j") });
                PoolAfterHistory::from_events(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS, &events)
            };
            let fault = ReadFault {
                slot: geometry.slot_of_txg(probe_history.highest_txg_used - u64::try_from(stuck_position).expect("j")),
                kind: ReadFaultKind::DeviceError,
                persistence: FaultPersistence::UntilTheProcessWritesTheSlot,
            };
            let roomy = PoolSettings {
                geometry,
                metadata_blocks: METADATA_BLOCKS_OF_THE_CONTROL_POOLS,
                reserve_reading: ReserveReading::CodeReadingOneEmptyPublish,
                fullness: PoolFullness::Roomy,
            };
            let controls = Controls { first_request_is_judged_short_throughout: true, ..Controls::none() };
            let pushes_still_needed = (ring_length - stuck_position - 1 - 1).max(0);
            for (stuck_ceiling, expected, mismatches) in [
                (StuckCeilingDisposition::PushEmptyBeyondBudget, pushes_still_needed, &mut mismatches_beyond_budget),
                (StuckCeilingDisposition::PushEmptyWithinBudget, pushes_still_needed.min(i64::try_from(PUBLISHES_PER_ADMISSION_AT_MOST - 1).expect("B")), &mut mismatches_within_budget),
            ] {
                let mut session = start_a_session(&probe_history, Some(fault), roomy, arm_of(stuck_ceiling, AllZeroSlotDisposition::NotMerged), controls, false);
                let observed = i64::from(session.run_one_request().empty_publishes);
                if observed != expected {
                    mismatches.push(format!("j{stuck_position}:{observed}vs{expected}"));
                }
            }
            positions += 1;
        }
        emit_a_check(
            emitter,
            tally,
            false,
            &format!("B3_F4_empty_pushes_S{slots_per_region}"),
            format!("positions={positions} beyond_budget_mismatches={mismatches_beyond_budget:?} within_budget_mismatches={mismatches_within_budget:?}"),
            format!("positions={positions} beyond_budget_mismatches=[] within_budget_mismatches=[]"),
        );
    }
}

// ── 每段的自检、判决行与入口 ─────────────────────────────────────────────────────────────

fn bool_text(condition: bool) -> &'static str {
    if condition {
        "true"
    } else {
        "false"
    }
}

/// 一段扫完之后的自检（登记第十一节 V2、V5，第七节 A1、互比旁边的钉死）。
#[derive(Clone, Copy, Debug)]
pub(super) struct SegmentChecks {
    pub(super) cells: usize,
    pub(super) positions: usize,
    runs: usize,
    positions_match_the_expected_count: bool,
    publishes_within_the_budget_where_the_arm_keeps_it: bool,
    the_two_zero_slot_arms_agree_where_they_must: bool,
    baselines_agree_across_arms: bool,
}

pub(super) fn check_the_segment(results: &[GroupResult], zero_slot_arms_must_agree_for: &dyn Fn(StuckCeilingDisposition) -> bool) -> SegmentChecks {
    let mut checks = SegmentChecks {
        cells: 0,
        positions: 0,
        runs: 0,
        positions_match_the_expected_count: true,
        publishes_within_the_budget_where_the_arm_keeps_it: true,
        the_two_zero_slot_arms_agree_where_they_must: true,
        baselines_agree_across_arms: true,
    };
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let mut baselines: BTreeMap<(HistoryFamily, u64, i64, &'static str, &'static str, &'static str), Vec<(bool, u64, i64, usize)>> = BTreeMap::new();
    for result in results {
        for cell in [&result.not_merged, &result.merged] {
            checks.cells += 1;
            checks.positions += cell.positions;
            checks.positions_match_the_expected_count &= cell.positions == cell.expected_positions;
            let keeps_the_budget = match cell.key.arm.stuck_ceiling {
                StuckCeilingDisposition::AsToday | StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::DoNotMergeTheBadSlots => true,
                StuckCeilingDisposition::PushEmptyBeyondBudget => false,
            };
            if keeps_the_budget {
                checks.publishes_within_the_budget_where_the_arm_keeps_it &=
                    cell.quantity("publishes_max_in_admission").maximum <= i64::try_from(PUBLISHES_PER_ADMISSION_AT_MOST).expect("B");
            }
        }
        checks.runs += result.runs;
        if zero_slot_arms_must_agree_for(result.not_merged.key.arm.stuck_ceiling) {
            checks.the_two_zero_slot_arms_agree_where_they_must &= result.pair.identical_positions == result.pair.positions;
        }
        let key = result.not_merged.key;
        if key.family != HistoryFamily::YoungRingWithCrashHoles {
            checks.the_two_zero_slot_arms_agree_where_they_must &= result.baselines_of_the_two_zero_slot_arms_are_identical;
            let baseline_key = (key.family, key.slots_per_region, key.metadata_blocks, key.reserve_reading.label(), key.fullness.label(), key.reserve_reading.pool_size_label());
            match baselines.get(&baseline_key) {
                None => {
                    baselines.insert(baseline_key, result.baseline_signature.clone());
                }
                Some(first_signature) => checks.baselines_agree_across_arms &= *first_signature == result.baseline_signature,
            }
        }
    }
    checks
}

pub(super) fn emit_the_segment_verdict(emitter: &mut Emitter, tally: &mut RunTally, segment: &str, checks: SegmentChecks) {
    tally.self_check(checks.positions_match_the_expected_count, "V2_positions_do_not_match_the_expected_count");
    tally.self_check(checks.publishes_within_the_budget_where_the_arm_keeps_it, "A1_publishes_over_the_budget");
    tally.self_check(checks.the_two_zero_slot_arms_agree_where_they_must, "F5_q2_arms_differ_where_they_must_agree");
    tally.self_check(checks.baselines_agree_across_arms, "baseline_differs_across_arms");
    emit(
        emitter,
        format!(
            "name=verdict scope={segment} cells={} positions={} runs={} positions_match_expected={} publishes_within_budget_where_the_arm_keeps_it={} q2_arms_agree_where_they_must={} baselines_agree_across_arms={} void_conditions={} void_list={} halt_conditions={}",
            checks.cells,
            checks.positions,
            checks.runs,
            bool_text(checks.positions_match_the_expected_count),
            bool_text(checks.publishes_within_the_budget_where_the_arm_keeps_it),
            bool_text(checks.the_two_zero_slot_arms_agree_where_they_must),
            bool_text(checks.baselines_agree_across_arms),
            tally.void_conditions.len(),
            if tally.void_conditions.is_empty() { "none".to_string() } else { tally.void_conditions.join(",") },
            tally.halt_conditions.len(),
        ),
    );
}

fn emit_the_configuration_line(emitter: &mut Emitter, segment: &str) {
    emit(
        emitter,
        format!(
            "name=config segment={segment} slots_per_region={SLOTS_PER_REGION_CHOICES:?} metadata_blocks={METADATA_BLOCKS_CHOICES:?} data_blocks_per_overwrite={DATA_BLOCKS_PER_OVERWRITE} admission_budget={PUBLISHES_PER_ADMISSION_AT_MOST} requests_per_mount=2*3S+{REQUESTS_PER_MOUNT_BASE} reserve_readings=code:c,clause:10+7c pools=near0,near2,roomy"
        ),
    );
}

pub(super) fn run_the_checks(emitter: &mut Emitter, tally: &mut RunTally) {
    emit_the_configuration_line(emitter, "checks");
    emit_the_frozen_tree_anchors(emitter, tally);
    emit_the_structural_anchors(emitter, tally);
    emit_the_first_request_anchor(emitter, tally);
    emit_the_roomy_pool_anchor(emitter, tally);
    emit_the_positive_controls(emitter, tally);
    emit_the_empty_push_count_anchor(emitter, tally);
    emit(
        emitter,
        format!(
            "name=verdict scope=checks self_checks_passed={} void_conditions={} void_list={} halt_conditions={} halt_list={}",
            tally.self_checks_passed,
            tally.void_conditions.len(),
            if tally.void_conditions.is_empty() { "none".to_string() } else { tally.void_conditions.join(",") },
            tally.halt_conditions.len(),
            if tally.halt_conditions.is_empty() { "none".to_string() } else { tally.halt_conditions.join(",") },
        ),
    );
}

pub(super) fn run_the_first_segment(emitter: &mut Emitter, tally: &mut RunTally) {
    emit_the_configuration_line(emitter, "first");
    let groups = groups_of_the_first_segment();
    let results = run_groups_in_parallel(&groups);
    for result in &results {
        emit(emitter, render_a_cell("first", &result.not_merged));
    }
    // 第一段的盘面里没有读回全零的槽（e、g 两种读坏），两条 Q2 臂在每一条 Q1 臂上都必须逐项相同。
    let checks = check_the_segment(&results, &|_| true);
    emit_the_first_segment_judgments(emitter, tally, &results);
    emit_the_segment_verdict(emitter, tally, "first", checks);
}

pub(super) fn run_the_second_segment(emitter: &mut Emitter, tally: &mut RunTally) {
    emit_the_configuration_line(emitter, "second");
    let groups = groups_of_the_second_segment();
    let results = run_groups_in_parallel(&groups);
    emit_the_zero_slot_cells(emitter, "second", &results);
    // 不并什么都不并（登记第十节 F5）：Q1 不并臂上两条 Q2 臂在每个位置上逐项相同。
    let checks = check_the_segment(&results, &|stuck_ceiling| stuck_ceiling == StuckCeilingDisposition::DoNotMergeTheBadSlots);
    emit_the_second_segment_judgments(emitter, tally, &results);
    emit_the_segment_verdict(emitter, tally, "second", checks);
}

/// 第二段那一组格的格行与两条 Q2 臂的并排行。
pub(super) fn emit_the_zero_slot_cells(emitter: &mut Emitter, segment: &str, results: &[GroupResult]) {
    for result in results {
        emit(emitter, render_a_cell(segment, &result.not_merged));
        emit(emitter, render_a_cell(segment, &result.merged));
        let pair = &result.pair;
        emit(
            emitter,
            format!(
                "name=q2_pair segment={segment} {} positions={} identical_positions={} not_merged_crossed_more={} merged_crossed_more={} not_merged_lost_more={} merged_lost_more={} largest_crossed_excess_of_not_merged={} largest_lost_excess_of_not_merged={} positions_merged_rejects_more={} largest_rejected_excess_of_merged={} positions_merged_is_all_enospc_against_not_merged={}",
                render_the_pair_key(&result.not_merged.key),
                pair.positions,
                pair.identical_positions,
                pair.not_merged_crossed_more,
                pair.merged_crossed_more,
                pair.not_merged_lost_more,
                pair.merged_lost_more,
                pair.largest_crossed_excess_of_not_merged,
                pair.largest_lost_excess_of_not_merged,
                pair.positions_merged_rejects_more,
                pair.largest_rejected_excess_of_merged,
                pair.positions_merged_is_all_enospc_against_not_merged
            ),
        );
    }
}

fn render_the_pair_key(key: &CellKey) -> String {
    format!(
        "family={} kind={} persistence={} S={} c_max={} reserve={} pool={} q1={}{}",
        key.family.label(),
        label_of_the_fault_kind(key.fault_kind),
        label_of_the_persistence(key.persistence),
        key.slots_per_region,
        key.metadata_blocks,
        key.reserve_reading.label(),
        key.fullness.label(),
        key.arm.stuck_ceiling.label(),
        key.reserve_reading.extra_key_fields()
    )
}
