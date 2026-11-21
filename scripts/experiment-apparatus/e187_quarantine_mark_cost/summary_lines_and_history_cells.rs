//! 比例与份额的格式化、格摘要与各输出行、历史格的判定与汇总行，以及这一路的运行。

use super::*;

/// 有理数按四位小数四舍五入（远离 0）打出来；产物里不用浮点。
const FIXED_POINT_SCALE: i128 = 10_000;

fn scaled_ratio(numerator: i128, denominator: i128) -> i128 {
    assert!(denominator > 0, "分母是正数");
    let scaled = numerator * FIXED_POINT_SCALE;
    let half = denominator / 2;
    if scaled >= 0 {
        (scaled + half) / denominator
    } else {
        -((-scaled + half) / denominator)
    }
}

fn format_scaled(scaled: i128) -> String {
    let sign = if scaled < 0 { "-" } else { "" };
    let magnitude = scaled.unsigned_abs();
    let scale = FIXED_POINT_SCALE.unsigned_abs();
    format!("{sign}{}.{:04}", magnitude / scale, magnitude % scale)
}

fn format_ratio(numerator: i128, denominator: i128) -> String {
    if denominator == 0 {
        "none".to_string()
    } else {
        format_scaled(scaled_ratio(numerator, denominator))
    }
}

/// 份额落哪一档：档一 < 0.1%，档二 [0.1%, 1%)，档三 ≥ 1%；少写（负）的落档一。
pub(super) fn share_tier(extra_bytes: i128, baseline_bytes: i128, bounds: ShareTierBounds) -> u8 {
    let parts_per_billion_times_baseline = extra_bytes * PARTS_PER_BILLION;
    if parts_per_billion_times_baseline < i128::from(bounds.tier_two_from) * baseline_bytes {
        1
    } else if parts_per_billion_times_baseline < i128::from(bounds.tier_three_from) * baseline_bytes {
        2
    } else {
        3
    }
}

/// 一组整数的均值、最小、最大（均值四位小数）；空组写 none。
fn mean_minimum_maximum(values: &[i64]) -> String {
    match (values.iter().min(), values.iter().max()) {
        (Some(minimum), Some(maximum)) => {
            let sum: i128 = values.iter().map(|value| i128::from(*value)).sum();
            format!("{}/{}/{}", format_ratio(sum, i128::from(signed(u64_of(values.len())))), minimum, maximum)
        }
        (None, _) | (_, None) => "none/none/none".to_string(),
    }
}

/// 一组有理数（每个按四位小数取整之后）的均值、最小、最大。
fn mean_minimum_maximum_of_ratios(ratios: &[(i64, u64)]) -> String {
    let scaled: Vec<i128> = ratios.iter().map(|(numerator, denominator)| scaled_ratio(i128::from(*numerator), i128::from(*denominator))).collect();
    match (scaled.iter().min(), scaled.iter().max()) {
        (Some(minimum), Some(maximum)) => {
            let sum: i128 = scaled.iter().sum();
            let count = i128::from(signed(u64_of(scaled.len())));
            format!("{}/{}/{}", format_scaled(scaled_ratio(sum, count * FIXED_POINT_SCALE)), format_scaled(*minimum), format_scaled(*maximum))
        }
        (None, _) | (_, None) => "none/none/none".to_string(),
    }
}

/// 一格在一条丙的臂上份额的两种口径。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ShareMeasurement {
    extra_bytes: i128,
    baseline_bytes: i128,
    tier: u8,
}

fn share_of(arm: WriteTally, baseline: WriteTally) -> ShareMeasurement {
    let extra_bytes = i128::from(difference(arm, baseline).bytes_on_both_devices);
    let baseline_bytes = i128::from(signed(baseline.bytes_on_both_devices));
    ShareMeasurement { extra_bytes, baseline_bytes, tier: share_tier(extra_bytes, baseline_bytes, SHARE_TIER_BOUNDS_PARTS_PER_BILLION) }
}

fn format_share(share: &ShareMeasurement) -> String {
    format_ratio(share.extra_bytes * 1_000_000, share.baseline_bytes)
}

/// 一格三条臂跑完之后，跨格汇总要的那几样。
pub(super) struct CellSummary {
    cell: HistoryCell,
    pub(super) lines: Vec<String>,
    /// (臂, 口径（0 不含根槽与系统配置槽、1 含）) → 份额。
    shares: BTreeMap<(Arm, usize), ShareMeasurement>,
    /// 臂 → 每次隔离那一次发布的设备写差之和与事件数（算法一）。
    per_event_device_writes: BTreeMap<Arm, (i64, u64)>,
    pub(super) arms_agree_without_events: Option<bool>,
    pub(super) schedule_realized: bool,
    pass_counts: BTreeMap<(Arm, CheckerUnderTest, usize), PassAndFalseRedCounts>,
    /// (臂, 变体) → (完好事件数, 期末为 0 的, 期末为正的)。
    trajectory_finals: BTreeMap<(Arm, CheckerUnderTest), (u64, u64, u64)>,
    span_bit_writes_extra_on_average: bool,
}

fn history_line(cell: &HistoryCell, arm: Arm, outcome: &HistoryOutcome) -> String {
    let without = outcome.totals.without_root_and_system_configuration();
    let with = outcome.totals.with_root_and_system_configuration();
    let quarantined_at_the_end = outcome.events.iter().filter(|event| event.record.revival.is_none()).count();
    format!(
        "name=history {} arm={} events={} rollbacks={} revived={} device_writes={} bytes={} bytes_single_device={} device_writes_with_root={} bytes_with_root={} \
         journal_records={} final_txg={} final_allocated_slots_per_device={} final_deferred_slots_per_device={} normal_units={} quarantined_units={} \
         mapping_tree_height={} placements_fell_back={}",
        cell.label(),
        arm.label(),
        outcome.events.len(),
        outcome.rollbacks.len(),
        outcome.rollbacks.iter().map(|rollback| rollback.revived).sum::<u64>(),
        without.device_writes,
        without.bytes_on_both_devices,
        without.bytes_on_both_devices / DEVICE_COUNT,
        with.device_writes,
        with.bytes_on_both_devices,
        outcome.totals.journal_records,
        outcome.final_txg,
        outcome.final_occupied_slots,
        outcome.final_deferred_slots,
        outcome.normal_units,
        quarantined_at_the_end,
        outcome.mapping_tree_height,
        outcome.placements_that_fell_back
    )
}

/// 一条丙的臂比 `no_mark` 多写多少（登记第六节前五行在工作负载格上的量）。
fn extra_writes_line(cell: &HistoryCell, arm: Arm, outcome: &HistoryOutcome, no_mark: &HistoryOutcome, summary: &mut CellSummary) -> String {
    assert_eq!(outcome.events.len(), no_mark.events.len(), "三条臂排的是同一串事件");
    assert_eq!(outcome.rollbacks.len(), no_mark.rollbacks.len(), "三条臂排的是同一串回退");
    let per_event: Vec<WriteDifference> = outcome
        .events
        .iter()
        .zip(no_mark.events.iter())
        .map(|(mine, baseline)| difference(mine.writes.without_root_and_system_configuration(), baseline.writes.without_root_and_system_configuration()))
        .collect();
    let device_writes: Vec<i64> = per_event.iter().map(|value| value.device_writes).collect();
    let bytes: Vec<i64> = per_event.iter().map(|value| value.bytes_on_both_devices).collect();
    let grouped = |leaf_rewritten_anyway: bool| -> String {
        let group: Vec<i64> = per_event
            .iter()
            .zip(no_mark.events.iter())
            .filter(|(_, baseline)| baseline.victim_leaf_rewritten == leaf_rewritten_anyway)
            .map(|(value, _)| value.device_writes)
            .collect();
        let sum: i64 = group.iter().sum();
        format!("{}(n={})", format_ratio(i128::from(sum), i128::from(signed(u64_of(group.len())))), group.len())
    };
    let mut structure_sum_arm = PublishWrites::default();
    let mut structure_sum_baseline = PublishWrites::default();
    for (mine, baseline) in outcome.events.iter().zip(no_mark.events.iter()) {
        structure_sum_arm.add(&mine.writes);
        structure_sum_baseline.add(&baseline.writes);
    }
    let event_count = i128::from(signed(u64_of(outcome.events.len())));
    let whole = difference(outcome.totals.without_root_and_system_configuration(), no_mark.totals.without_root_and_system_configuration());
    let revivals: Vec<(i64, u64)> = outcome
        .rollbacks
        .iter()
        .zip(no_mark.rollbacks.iter())
        .filter(|(_, baseline)| baseline.revived > 0)
        .map(|(mine, baseline)| (difference(mine.writes.without_root_and_system_configuration(), baseline.writes.without_root_and_system_configuration()).device_writes, baseline.revived))
        .collect();
    let revival_bytes: Vec<(i64, u64)> = outcome
        .rollbacks
        .iter()
        .zip(no_mark.rollbacks.iter())
        .filter(|(_, baseline)| baseline.revived > 0)
        .map(|(mine, baseline)| (difference(mine.writes.without_root_and_system_configuration(), baseline.writes.without_root_and_system_configuration()).bytes_on_both_devices, baseline.revived))
        .collect();
    let share_without = share_of(outcome.totals.without_root_and_system_configuration(), no_mark.totals.without_root_and_system_configuration());
    let share_with = share_of(outcome.totals.with_root_and_system_configuration(), no_mark.totals.with_root_and_system_configuration());
    summary.shares.insert((arm, 0), share_without);
    summary.shares.insert((arm, 1), share_with);
    summary.per_event_device_writes.insert(arm, (device_writes.iter().sum(), u64_of(device_writes.len())));
    format!(
        "name=extra_writes {} arm={} events={} per_event_device_writes_mean_min_max={} per_event_bytes_mean_min_max={} \
         per_event_device_writes_mean_when_the_leaf_is_rewritten_anyway={} per_event_device_writes_mean_when_the_leaf_is_not={} \
         per_event_structure_sums {} whole_history_device_writes_per_event={} whole_history_bytes_per_event={} \
         share_without_root_ppm={} tier_without_root={} share_with_root_ppm={} tier_with_root={} \
         revival_device_writes_per_revived_mean_min_max={} revival_bytes_per_revived_mean_min_max={}",
        cell.label(),
        arm.label(),
        outcome.events.len(),
        mean_minimum_maximum(&device_writes),
        mean_minimum_maximum(&bytes),
        grouped(true),
        grouped(false),
        structure_differences(&structure_sum_arm, &structure_sum_baseline),
        format_ratio(i128::from(whole.device_writes), event_count),
        format_ratio(i128::from(whole.bytes_on_both_devices), event_count),
        format_share(&share_without),
        share_without.tier,
        format_share(&share_with),
        share_with.tier,
        mean_minimum_maximum_of_ratios(&revivals),
        mean_minimum_maximum_of_ratios(&revival_bytes)
    )
}

/// 终态上的放过与误红：每个仍隔离的单元按副本与年龄归类、正常单元一类，每类每种记号状态各一张盘面（登记第六节 `pass_and_false_red_counts`）。
fn terminal_pass_and_false_red(cell: &HistoryCell, arm: Arm, outcome: &HistoryOutcome, summary: &mut CellSummary) -> Vec<String> {
    let mut lines = Vec::new();
    for checker in checkers_of(arm) {
        let mut counts = [PassAndFalseRedCounts::default(); 2];
        for mark_state in MARK_STATES {
            let normal_verdict = judge_record(*checker, &constructed_view(arm, RecordClass::Normal, mark_state, JudgedVersion::TheQuarantineVersion));
            counts[universe_of(mark_state)].count(is_a_legal_disk(RecordClass::Normal, mark_state), &normal_verdict, outcome.normal_units);
            for event in outcome.events.iter().filter(|event| event.record.revival.is_none()) {
                let class = match event.record.copies {
                    CopyCondition::Intact => RecordClass::QuarantineIntact,
                    CopyCondition::Garbled => RecordClass::QuarantineGarbled,
                };
                let version = judged_version_at_the_end(&event.record, outcome.final_txg, cell.candidate_window);
                let verdict = judge_record(*checker, &constructed_view(arm, class, mark_state, version));
                counts[universe_of(mark_state)].count(is_a_legal_disk(class, mark_state), &verdict, 1);
            }
        }
        for (universe, universe_counts) in counts.iter().enumerate() {
            summary.pass_counts.insert((arm, *checker, universe), *universe_counts);
            lines.push(format!(
                "name=terminal_pass_and_false_red {} arm={} checker={} universe={} leak_units=0 passed_defects={} false_reds={} judged_disks={}",
                cell.label(),
                arm.label(),
                checker.label(),
                UNIVERSE_LABELS[universe],
                universe_counts.passed_defects,
                universe_counts.false_reds,
                universe_counts.judged_disks
            ));
        }
    }
    lines
}

/// 每个 `quarantine_intact` 事件的轨迹，按臂与变体汇总（登记第六节 `no_mark_false_red_trajectory`）。
fn trajectory_lines(cell: &HistoryCell, arm: Arm, outcome: &HistoryOutcome, summary: &mut CellSummary) -> Vec<String> {
    let mut lines = Vec::new();
    for checker in checkers_of(arm) {
        let intact: Vec<EventTrajectory> = outcome
            .events
            .iter()
            .filter(|event| event.record.copies == CopyCondition::Intact)
            .map(|event| trajectory_of_an_event(*checker, &event.record, outcome.final_txg, cell.candidate_window))
            .collect();
        let final_zero = u64_of(intact.iter().filter(|trajectory| trajectory.final_value == 0).count());
        let final_positive = u64_of(intact.len()) - final_zero;
        let positive_versions: Vec<i64> = intact.iter().map(|trajectory| signed(trajectory.positive_versions)).collect();
        let peaks: Vec<i64> = intact.iter().map(|trajectory| signed(trajectory.peak)).collect();
        let first_red_offsets: Vec<i64> = intact.iter().filter_map(|trajectory| trajectory.first_red_offset_of_the_allocated_statistic).map(signed).collect();
        summary.trajectory_finals.insert((arm, *checker), (u64_of(intact.len()), final_zero, final_positive));
        lines.push(format!(
            "name=trajectory {} arm={} checker={} intact_events={} final_value_zero={} final_value_positive={} peak_mean_min_max={} \
             positive_versions_mean_min_max={} allocated_statistic_first_red_offset_mean_min_max={} mark_agreement_red_versions={}",
            cell.label(),
            arm.label(),
            checker.label(),
            intact.len(),
            final_zero,
            final_positive,
            mean_minimum_maximum(&peaks),
            mean_minimum_maximum(&positive_versions),
            mean_minimum_maximum(&first_red_offsets),
            intact.iter().map(|trajectory| trajectory.mark_agreement_red_versions).sum::<u64>()
        ));
    }
    lines
}

/// 一格：同一串事件在三条臂上各跑一遍，出这一格的全部产物行与跨格汇总要的数。
pub(super) fn evaluate_cell(cell: &HistoryCell) -> CellSummary {
    let schedule = schedule_of(cell);
    let outcomes: Vec<(Arm, HistoryOutcome)> = FORK_ARMS.iter().map(|arm| (*arm, run_history(cell, &schedule, *arm))).collect();
    let mut summary = CellSummary {
        cell: *cell,
        lines: Vec::new(),
        shares: BTreeMap::new(),
        per_event_device_writes: BTreeMap::new(),
        arms_agree_without_events: None,
        schedule_realized: true,
        pass_counts: BTreeMap::new(),
        trajectory_finals: BTreeMap::new(),
        span_bit_writes_extra_on_average: false,
    };
    let no_mark = &outcomes[0].1;
    for (arm, outcome) in &outcomes {
        summary.lines.push(history_line(cell, *arm, outcome));
        let expected_events = cell.density * (NORMAL_PUBLISHES_PER_HISTORY / 1000);
        let expected_rollbacks = cell.rollback_rate * ROLLBACKS_AT_RATE_ONE;
        let rollbacks_revive = cell.density == 0 || outcome.rollbacks.iter().all(|rollback| rollback.revived >= 1);
        summary.schedule_realized &= u64_of(outcome.events.len()) == expected_events && u64_of(outcome.rollbacks.len()) == expected_rollbacks && rollbacks_revive;
    }
    for (arm, outcome) in outcomes.iter().skip(1) {
        let line = extra_writes_line(cell, *arm, outcome, no_mark, &mut summary);
        summary.lines.push(line);
    }
    if let Some((sum, count)) = summary.per_event_device_writes.get(&Arm::SpanBit) {
        summary.span_bit_writes_extra_on_average = *count > 0 && *sum > 0;
    }
    if cell.density == 0 && cell.rollback_rate == 0 {
        summary.arms_agree_without_events = Some(outcomes.iter().all(|(_, outcome)| {
            outcome.totals == no_mark.totals && outcome.final_occupied_slots == no_mark.final_occupied_slots && outcome.final_deferred_slots == no_mark.final_deferred_slots
        }));
    }
    for (arm, outcome) in &outcomes {
        let lines = trajectory_lines(cell, *arm, outcome, &mut summary);
        summary.lines.extend(lines);
        let terminal_lines = terminal_pass_and_false_red(cell, *arm, outcome, &mut summary);
        summary.lines.extend(terminal_lines);
    }
    summary.lines.push(format!(
        "name=cell_checks {} schedule_realized={} arms_agree_without_events={}",
        cell.label(),
        summary.schedule_realized,
        summary.arms_agree_without_events.map_or("not_this_cell".to_string(), |agree| agree.to_string())
    ));
    summary
}

/// 第二段的格按次序切片并行：每个线程从共享的下一个号取格，结果放回格的位置，合并按格序（与线程数无关）。
pub(super) fn evaluate_cells(cells: &[HistoryCell], threads: usize) -> Vec<CellSummary> {
    let next_cell = std::sync::atomic::AtomicUsize::new(0);
    let slots: Vec<std::sync::Mutex<Option<CellSummary>>> = cells.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let cell_index = next_cell.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if cell_index >= cells.len() {
                    break;
                }
                let summary = evaluate_cell(&cells[cell_index]);
                // 进度只进标准错误，不进产物。
                eprintln!("e187 第二段：格 {} / {} 跑完（{}）", cell_index + 1, cells.len(), cells[cell_index].label());
                *slots[cell_index].lock().expect("别的线程没在持锁时 panic") = Some(summary);
            });
        }
    });
    slots.into_iter().map(|slot| slot.into_inner().expect("锁没中毒").expect("每一格都评估过")).collect()
}

const SCOPE_LABELS: [&str; 2] = ["without_root_and_system_configuration", "with_root_and_system_configuration"];

/// 两个份额谁大（交叉相乘比，分母都是正数）。
fn share_is_larger(left: &ShareMeasurement, right: &ShareMeasurement) -> bool {
    left.extra_bytes * right.baseline_bytes > right.extra_bytes * left.baseline_bytes
}

fn parts_per_billion_of(share: &ShareMeasurement) -> i128 {
    share.extra_bytes * PARTS_PER_BILLION / share.baseline_bytes
}

/// 第二段跨格汇总的判定（登记第六节、第十节、第十一节）。
struct SegmentTwoJudgements {
    quarantine_cost_negligible_holds: bool,
    span_bit_writes_nothing_extra_holds: bool,
    no_mark_false_red_persists_holds: bool,
    arms_agree_without_events_holds: bool,
    scheduled_events_realized_holds: bool,
}

fn segment_two_summary_lines(summaries: &[CellSummary]) -> (Vec<String>, SegmentTwoJudgements) {
    let mut lines = Vec::new();
    // 份额按密度：那一密度全部格里的最高档与最低档；另按 (U, 几何, K) 分开看档换不换。
    let mut quarantine_cost_negligible_holds = true;
    for arm in MARKING_ARMS {
        for density in DENSITIES_PER_THOUSAND_PUBLISHES {
            for (scope, scope_label) in SCOPE_LABELS.iter().enumerate() {
                let shares: Vec<(&HistoryCell, ShareMeasurement)> = summaries
                    .iter()
                    .filter(|summary| summary.cell.density == density)
                    .map(|summary| (&summary.cell, *summary.shares.get(&(arm, scope)).expect("每一格都算了两种口径")))
                    .collect();
                let mut tier_counts = [0u64; 3];
                for (_, share) in &shares {
                    tier_counts[usize::from(share.tier - 1)] += 1;
                }
                let largest = shares.iter().fold(None, |largest: Option<ShareMeasurement>, (_, share)| match largest {
                    Some(current) if !share_is_larger(share, &current) => Some(current),
                    Some(_) | None => Some(*share),
                });
                let highest_tier = shares.iter().map(|(_, share)| share.tier).max().expect("每个密度都有格");
                let lowest_tier = shares.iter().map(|(_, share)| share.tier).min().expect("每个密度都有格");
                if density == 10 && highest_tier > 1 {
                    quarantine_cost_negligible_holds = false;
                }
                lines.push(format!(
                    "name=share_tier_by_density arm={} density={} scope={} cells={} highest_tier={} lowest_tier={} tier_counts={}/{}/{} largest_share_ppm={}",
                    arm.label(),
                    density,
                    scope_label,
                    shares.len(),
                    highest_tier,
                    lowest_tier,
                    tier_counts[0],
                    tier_counts[1],
                    tier_counts[2],
                    largest.map_or("none".to_string(), |share| format_share(&share))
                ));
                for file_size in FILE_SIZES_IN_DATA_UNITS {
                    for geometry in GEOMETRIES {
                        for candidate_window in CANDIDATE_WINDOWS {
                            let tiers: Vec<u8> = shares
                                .iter()
                                .filter(|(cell, _)| cell.file_size == file_size && cell.geometry == geometry && cell.candidate_window == candidate_window)
                                .map(|(_, share)| share.tier)
                                .collect();
                            lines.push(format!(
                                "name=share_tier_by_point arm={} density={} scope={} file_size={} geometry={} window={} cells={} highest_tier={} lowest_tier={}",
                                arm.label(),
                                density,
                                scope_label,
                                file_size,
                                geometry.label(),
                                candidate_window,
                                tiers.len(),
                                tiers.iter().max().expect("每个点都有格"),
                                tiers.iter().min().expect("每个点都有格")
                            ));
                        }
                    }
                }
            }
        }
    }
    // 甲与乙谁的每次隔离额外写小（算法一，按格的均值比；同号记那一个便宜，换号记依赖取样点）。
    let mut span_bit_cheaper_cells = 0u64;
    let mut quarantine_tree_cheaper_cells = 0u64;
    let mut equal_cells = 0u64;
    for density in DENSITIES_PER_THOUSAND_PUBLISHES.iter().filter(|density| **density > 0) {
        for rollback_rate in ROLLBACK_RATES_PER_THOUSAND_PUBLISHES {
            for file_size in FILE_SIZES_IN_DATA_UNITS {
                for candidate_window in CANDIDATE_WINDOWS {
                    for geometry in GEOMETRIES {
                        let mut seeds_by_reading: [Vec<u64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
                        for summary in summaries.iter().filter(|summary| {
                            summary.cell.density == *density
                                && summary.cell.rollback_rate == rollback_rate
                                && summary.cell.file_size == file_size
                                && summary.cell.candidate_window == candidate_window
                                && summary.cell.geometry == geometry
                        }) {
                            let (span_bit_sum, _) = summary.per_event_device_writes[&Arm::SpanBit];
                            let (quarantine_tree_sum, _) = summary.per_event_device_writes[&Arm::QuarantineTree];
                            let reading = match span_bit_sum.cmp(&quarantine_tree_sum) {
                                std::cmp::Ordering::Less => 0,
                                std::cmp::Ordering::Greater => 1,
                                std::cmp::Ordering::Equal => 2,
                            };
                            seeds_by_reading[reading].push(summary.cell.seed);
                        }
                        span_bit_cheaper_cells += u64_of(seeds_by_reading[0].len());
                        quarantine_tree_cheaper_cells += u64_of(seeds_by_reading[1].len());
                        equal_cells += u64_of(seeds_by_reading[2].len());
                        let seeds = |reading: usize| seeds_by_reading[reading].iter().map(u64::to_string).collect::<Vec<String>>().join(",");
                        lines.push(format!(
                            "name=cheaper_placement_by_point density={density} rollback_rate={rollback_rate} file_size={file_size} window={candidate_window} geometry={} \
                             span_bit_cheaper_seeds=[{}] quarantine_tree_cheaper_seeds=[{}] equal_seeds=[{}]",
                            geometry.label(),
                            seeds(0),
                            seeds(1),
                            seeds(2)
                        ));
                    }
                }
            }
        }
    }
    let cheaper_reading = match (span_bit_cheaper_cells > 0, quarantine_tree_cheaper_cells > 0) {
        (true, true) => "depends_on_sampling_point",
        (true, false) => "span_bit_cheaper_everywhere_or_equal",
        (false, true) => "quarantine_tree_cheaper_everywhere_or_equal",
        (false, false) => "equal_everywhere",
    };
    lines.push(format!(
        "name=cheaper_placement_summary span_bit_cheaper_cells={span_bit_cheaper_cells} quarantine_tree_cheaper_cells={quarantine_tree_cheaper_cells} equal_cells={equal_cells} reading={cheaper_reading}"
    ));
    // 终态放过与误红：全部格相加，按臂、变体、宇宙；丙比 no_mark 怎么记。
    let mut totals: BTreeMap<(Arm, CheckerUnderTest, usize), PassAndFalseRedCounts> = BTreeMap::new();
    for summary in summaries {
        for (key, counts) in &summary.pass_counts {
            let total = totals.entry(*key).or_default();
            total.passed_defects += counts.passed_defects;
            total.false_reds += counts.false_reds;
            total.judged_disks += counts.judged_disks;
        }
    }
    for ((arm, checker, universe), counts) in &totals {
        let reading = match arm {
            Arm::SpanBit | Arm::QuarantineTree => pass_and_false_red_reading(*counts, totals[&(Arm::NoMark, CheckerUnderTest::Today, *universe)]),
            Arm::NoMark | Arm::PlainRelease => "baseline",
        };
        lines.push(format!(
            "name=terminal_pass_and_false_red_total arm={} checker={} universe={} passed_defects={} false_reds={} judged_disks={} reading={reading}",
            arm.label(),
            checker.label(),
            UNIVERSE_LABELS[*universe],
            counts.passed_defects,
            counts.false_reds,
            counts.judged_disks
        ));
    }
    // 轨迹：期末为 0（误红自己消掉）与期末仍为正的事件数，全部格相加。
    let mut no_mark_false_red_persists_holds = true;
    let mut trajectory_totals: BTreeMap<(Arm, CheckerUnderTest), (u64, u64, u64)> = BTreeMap::new();
    for summary in summaries {
        for (key, (events, final_zero, final_positive)) in &summary.trajectory_finals {
            let total = trajectory_totals.entry(*key).or_default();
            total.0 += events;
            total.1 += final_zero;
            total.2 += final_positive;
        }
    }
    for ((arm, checker), (events, final_zero, final_positive)) in &trajectory_totals {
        if *arm == Arm::NoMark && *final_zero > 0 {
            no_mark_false_red_persists_holds = false;
        }
        lines.push(format!(
            "name=trajectory_total arm={} checker={} intact_events={events} final_value_zero={final_zero} final_value_positive={final_positive}",
            arm.label(),
            checker.label()
        ));
    }
    // 判别力自证（登记第八节末段）：把档界挪到 U = 200 与 U = 20000 两点的甲份额之间，两点必须落进不同档；挪回原档界照原样判。
    let probe = |file_size: u32| -> ShareMeasurement {
        let summary = summaries
            .iter()
            .find(|summary| {
                summary.cell.density == 10
                    && summary.cell.rollback_rate == 0
                    && summary.cell.file_size == file_size
                    && summary.cell.candidate_window == CANDIDATE_WINDOWS[0]
                    && summary.cell.geometry == PoolGeometry::DefaultFourGibibyteDevices
                    && summary.cell.seed == SEEDS[0]
            })
            .expect("判别力自证那两格在格表里");
        summary.shares[&(Arm::SpanBit, 0)]
    };
    let small_file = probe(FILE_SIZES_IN_DATA_UNITS[0]);
    let large_file = probe(FILE_SIZES_IN_DATA_UNITS[1]);
    let (small_parts, large_parts) = (parts_per_billion_of(&small_file), parts_per_billion_of(&large_file));
    let moved_bound = u64::try_from((small_parts + large_parts) / 2 + 1).unwrap_or(1);
    let moved = ShareTierBounds { tier_two_from: moved_bound.max(1), tier_three_from: u64::MAX / 2 };
    let moved_tiers = (share_tier(small_file.extra_bytes, small_file.baseline_bytes, moved), share_tier(large_file.extra_bytes, large_file.baseline_bytes, moved));
    let restored_tiers = (
        share_tier(small_file.extra_bytes, small_file.baseline_bytes, SHARE_TIER_BOUNDS_PARTS_PER_BILLION),
        share_tier(large_file.extra_bytes, large_file.baseline_bytes, SHARE_TIER_BOUNDS_PARTS_PER_BILLION),
    );
    lines.push(format!(
        "name=share_tier_discrimination arm=span_bit density=10 rollback_rate=0 window={} geometry=geometry_default seed={} scope=without_root_and_system_configuration \
         small_file_share_ppb={small_parts} large_file_share_ppb={large_parts} moved_tier_two_from_ppb={moved_bound} moved_tiers={}/{} restored_tiers={}/{} \
         separates_when_moved={} restored_equals_original={}",
        CANDIDATE_WINDOWS[0],
        SEEDS[0],
        moved_tiers.0,
        moved_tiers.1,
        restored_tiers.0,
        restored_tiers.1,
        moved_tiers.0 != moved_tiers.1,
        restored_tiers.0 == small_file.tier && restored_tiers.1 == large_file.tier
    ));
    let judgements = SegmentTwoJudgements {
        quarantine_cost_negligible_holds,
        span_bit_writes_nothing_extra_holds: summaries.iter().all(|summary| !summary.span_bit_writes_extra_on_average),
        no_mark_false_red_persists_holds,
        arms_agree_without_events_holds: summaries.iter().filter_map(|summary| summary.arms_agree_without_events).all(|agree| agree),
        scheduled_events_realized_holds: summaries.iter().all(|summary| summary.schedule_realized),
    };
    (lines, judgements)
}

pub(super) fn run_segment_two(threads: usize) -> Vec<String> {
    let cells = all_history_cells();
    let summaries = evaluate_cells(&cells, threads);
    let mut lines = vec![format!(
        "name=segment_two_config cells={} arms={} normal_publishes_per_history={NORMAL_PUBLISHES_PER_HISTORY} overwrites_per_publish={OVERWRITES_PER_PUBLISH} \
         densities_per_thousand={:?} rollback_rates_per_thousand={:?} file_sizes={:?} windows={:?} seeds={:?} threads={threads} \
         counted_writes=every_publish_after_the_file_creation normal_units=unreleased_records_of_live_units_without_the_two_make_filesystem_records leak_units=0",
        cells.len(),
        FORK_ARMS.iter().map(|arm| arm.label()).collect::<Vec<&str>>().join(","),
        DENSITIES_PER_THOUSAND_PUBLISHES,
        ROLLBACK_RATES_PER_THOUSAND_PUBLISHES,
        FILE_SIZES_IN_DATA_UNITS,
        CANDIDATE_WINDOWS,
        SEEDS
    )];
    for summary in &summaries {
        lines.extend(summary.lines.iter().cloned());
    }
    let (summary_lines, judgements) = segment_two_summary_lines(&summaries);
    lines.extend(summary_lines);
    lines.push(format!(
        "name=verdict segment=two quarantine_cost_negligible_holds={} span_bit_writes_nothing_extra_holds={} no_mark_false_red_persists_holds={} \
         arms_agree_without_events={} scheduled_events_realized={} unreadable_counts=zero",
        judgements.quarantine_cost_negligible_holds,
        judgements.span_bit_writes_nothing_extra_holds,
        judgements.no_mark_false_red_persists_holds,
        judgements.arms_agree_without_events_holds,
        judgements.scheduled_events_realized_holds
    ));
    lines
}
