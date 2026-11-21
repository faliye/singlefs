//! 一趟的汇总与产物行、分片选择、并行跑、格判定与跨取样点的汇总行、预设值行、分片读入与打印。

use super::*;

/// 一条跑交给格一级判定的东西。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RunSummary {
    pub(super) key: RunKey,
    growth: GrowthVerdict,
    pub(super) deletion_failures: u64,
    pub(super) intent_errors: IntentErrorCounts,
}

fn round_line(label: &str, row: &RoundRow) -> String {
    let metrics = &row.metrics;
    let free_at_refusal = metrics.free_slots_at_last_refusal.map_or_else(|| "NA".to_string(), |slots| slots.to_string());
    format!(
        "name=round {label} round={} lowest_record_empty_segments={} lowest_fully_free_segments={} lowest_openable_segments={} deletion_publishes={} deletion_commit_generated_block_peak={} deletion_commit_generated_block_sum={} fallback={} deletion_publish_failures={} empty_publish_failures={} refused_writes_by_conjunct_one={} refused_writes_by_conjunct_seven={} free_slots_at_last_refusal={free_at_refusal} writes={} batches={} moved={} empties={} intents_done={} intents_dropped={} runs={} live_units={} unrecoverable={} record_empty_segments_at_end={}",
        row.round,
        metrics.lowest_record_empty_segments,
        metrics.lowest_fully_free_segments,
        metrics.lowest_openable_segments,
        metrics.deletion_publishes,
        metrics.deletion_commit_generated_block_peak,
        metrics.deletion_commit_generated_block_sum,
        metrics.fallback_blocks,
        metrics.deletion_publish_failures,
        metrics.empty_publish_failures,
        metrics.refused_writes_by_conjunct_one,
        metrics.refused_writes_by_conjunct_seven,
        metrics.foreground_writes,
        metrics.compaction_batches,
        metrics.units_moved,
        metrics.empty_publishes,
        metrics.intents_completed,
        metrics.intents_discarded,
        row.free_runs,
        row.live_user_units,
        row.unrecoverable_slots,
        row.record_empty_segments_at_end
    )
}

/// 一条跑的全部产物行（逐轮行、逐意图列、汇总行）与交给格一级的汇总。
pub(super) fn lines_of_finished_run(run: &FinishedRun) -> (Vec<String>, RunSummary) {
    let label = run.key.label();
    let floor_segments = run.key.arm.floor_segments(run.key.root_ring_slots_per_region, run.key.commit_generated_blocks_per_publish);
    let mut lines: Vec<String> = run.rows.iter().map(|row| round_line(&label, row)).collect();
    let intents = &run.ended_intents;
    lines.push(format!(
        "name=intents {label} count={} completed={} region_is_record_empty={} global_difference_is_positive={} region_became_openable={} openable_count_rose={}",
        intents.len(),
        bit_column(intents, |intent| Some(intent.is_completed)),
        bit_column(intents, |intent| Some(intent.region_is_record_empty)),
        bit_column(intents, |intent| Some(intent.global_difference_is_positive)),
        bit_column(intents, |intent| intent.region_became_openable),
        bit_column(intents, |intent| intent.openable_count_rose)
    ));
    let series = |value_of: fn(&RoundRow) -> u64| -> Vec<u64> { run.rows.iter().map(value_of).collect() };
    let record_empty = trajectory_statistics(&series(|row| row.metrics.lowest_record_empty_segments), floor_segments);
    let fully_free = trajectory_statistics(&series(|row| row.metrics.lowest_fully_free_segments), floor_segments);
    let openable = trajectory_statistics(&series(|row| row.metrics.lowest_openable_segments), floor_segments);
    let fallback = trajectory_statistics(&series(|row| row.metrics.fallback_blocks), 0);
    let deletion_failures = deletion_failures_of(&run.rows);
    let deletion_publish_failures: u64 = run.rows.iter().map(|row| row.metrics.deletion_publish_failures).sum();
    let empty_publish_failures: u64 = run.rows.iter().map(|row| row.metrics.empty_publish_failures).sum();
    let refused_by_one: u64 = run.rows.iter().map(|row| row.metrics.refused_writes_by_conjunct_one).sum();
    let refused_by_seven: u64 = run.rows.iter().map(|row| row.metrics.refused_writes_by_conjunct_seven).sum();
    let intent_errors = IntentErrorCounts::of(intents);
    lines.push(format!(
        "name=run {label} rounds={} segment_floor={floor_segments} deletion_block_growth={} deletion_failures_verdict={} deletion_failures={deletion_failures} deletion_publish_failures={deletion_publish_failures} empty_publish_failures={empty_publish_failures} {} {} {} fallback_peak_from_round_two={} fallback_positive_rounds={} fallback_final={} refused_writes_by_conjunct_one={refused_by_one} refused_writes_by_conjunct_seven={refused_by_seven} {}",
        run.rows.len(),
        run.growth.tag(),
        failure_tag(deletion_failures),
        trajectory_fields("record_empty_segments", &record_empty),
        trajectory_fields("fully_free_segments", &fully_free),
        trajectory_fields("openable_segments", &openable),
        fallback.peak_from_round_two,
        fallback.rounds_positive,
        fallback.final_value,
        intent_errors.fields()
    ));
    let summary = RunSummary { key: run.key, growth: run.growth, deletion_failures, intent_errors };
    (lines, summary)
}

/// 第 7 行：空池上只发空发布 1000 次，记不可开聚簇段数（开放段 + 装着未回收块的段）的峰值。
fn metadata_stream_peak_unopenable_segments(geometry: PoolGeometry) -> u64 {
    let settings = PoolSettings {
        floor_segments: 0,
        is_compaction_enabled: false,
        stop_arm: StopArm::LocalRegionPredicate,
        resident_base_multiplier: 3,
    };
    let mut pool = Pool::made_by_make_filesystem(geometry, settings);
    let mut peak = 0;
    for _empty_publish in 0..METADATA_STREAM_EMPTY_PUBLISHES {
        pool.publish_empty().expect("空池上的空发布放得下");
        let open_segment = pool.open_cursor.map(|cursor| cursor.segment);
        let unopenable = (0..geometry.segment_count())
            .filter(|segment| Some(*segment) == open_segment || pool.segments[usize_of(*segment)].released_slots > 0)
            .count() as u64;
        peak = peak.max(unopenable);
    }
    peak
}

/// 一份分片跑哪些：一个取样点，可再限一个 S（不带参数的全跑是 None）。分片按（取样点, S）的次序拼起来就是全跑的次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ShardSelection<'name> {
    pub(super) point: &'name str,
    pub(super) root_ring_slots_per_region: Option<u64>,
}

fn is_selected(selection: Option<ShardSelection>, point: &SamplingPoint, root_ring_slots_per_region: u64) -> bool {
    selection.is_none_or(|shard| {
        shard.point == point.name && shard.root_ring_slots_per_region.is_none_or(|wanted| wanted == root_ring_slots_per_region)
    })
}

/// 第一段的全部正式跑：7 个取样点 × 9 格 × 4 档 `N₀` × 整理开关 × 5 个种子，按这个次序排（产物行的次序就是它）。
pub(super) fn all_run_keys(selection: Option<ShardSelection>) -> Vec<RunKey> {
    let mut keys = Vec::new();
    for point in SAMPLING_POINTS.iter().copied() {
        for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
            if !is_selected(selection, &point, root_ring_slots_per_region) {
                continue;
            }
            for commit_generated_blocks_per_publish in COMMIT_GENERATED_BLOCKS_CHOICES {
                for arm in FLOOR_ARMS {
                    for is_compaction_enabled in [true, false] {
                        for seed in 1..=SEEDS_PER_CELL {
                            keys.push(RunKey {
                                point,
                                root_ring_slots_per_region,
                                commit_generated_blocks_per_publish,
                                arm,
                                is_compaction_enabled,
                                seed,
                            });
                        }
                    }
                }
            }
        }
    }
    keys
}

/// 线程数：`E10_THREADS`，没设取 `SINGLEFS_THREAD_CAP`（`scripts/hosts/capped.sh` 设的），都没设取 1。输出与线程数无关。
fn worker_thread_count() -> usize {
    ["E10_THREADS", "SINGLEFS_THREAD_CAP"]
        .iter()
        .find_map(|variable| std::env::var(variable).ok())
        .map_or(1, |text| text.parse::<usize>().expect("线程数是正整数").max(1))
}

/// 并行跑全部正式跑，按键的次序交回（各片互不共享状态，合并按下标，确定）。
fn run_fill_delete_fill_runs_in_parallel(keys: &[RunKey], threads: usize) -> Vec<(Vec<String>, RunSummary)> {
    let next_index = AtomicUsize::new(0);
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let results: Mutex<Vec<Option<(Vec<String>, RunSummary)>>> = Mutex::new(vec![None; keys.len()]);
    std::thread::scope(|scope| {
        for _worker in 0..threads {
            scope.spawn(|| loop {
                let index = next_index.fetch_add(1, Ordering::Relaxed);
                let Some(key) = keys.get(index) else {
                    break;
                };
                let finished = FillDeleteFillRun::new(*key).run_to_judgement();
                let produced = lines_of_finished_run(&finished);
                results.lock().expect("没有工作线程在持锁时 panic")[index] = Some(produced);
            });
        }
    });
    results
        .into_inner()
        .expect("工作线程都已结束")
        .into_iter()
        .map(|produced| produced.expect("每条跑都交回了"))
        .collect()
}

/// 一格一臂一个整理开关上五个种子合成的判定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CellArmVerdict {
    pub(super) growth: &'static str,
    pub(super) failures: &'static str,
    pub(super) growth_seeds: String,
    pub(super) failure_seeds: String,
}

fn cell_arm_verdict(summaries: &[&RunSummary]) -> CellArmVerdict {
    let growth_tags: Vec<&'static str> = summaries.iter().map(|summary| summary.growth.tag()).collect();
    let failure_tags: Vec<&'static str> = summaries.iter().map(|summary| failure_tag(summary.deletion_failures)).collect();
    CellArmVerdict {
        growth: combine_seed_tags(&growth_tags),
        failures: combine_seed_tags(&failure_tags),
        growth_seeds: growth_tags.join("/"),
        failure_seeds: summaries.iter().map(|summary| summary.deletion_failures.to_string()).collect::<Vec<_>>().join("/"),
    }
}

/// 阳性对照（登记第五节）：同一臂整理关掉要退化——第 3 行「发散」或第 4 行「失败」。
pub(super) fn off_control_degrades(off: &CellArmVerdict) -> bool {
    off.growth == GrowthVerdict::Diverging.tag() || off.failures == "failure"
}

/// 一格一个候选的判定：第 3 行发散或第 4 行失败 ⇒ 不够；第 3 行有界、第 4 行无、对照成立、没打中 F3 ⇒ 过；其余不够判。
pub(super) fn candidate_cell_verdict(on: (&str, &str), control_holds: bool, shared_premise_hit: bool) -> &'static str {
    let (growth, failures) = on;
    if growth == GrowthVerdict::Diverging.tag() || failures == "failure" {
        "fail"
    } else if growth == GrowthVerdict::Bounded.tag() && failures == "none" && control_holds && !shared_premise_hit {
        "pass"
    } else {
        "not_judgeable"
    }
}

/// 几何敏感性（登记第八节、F7）：主取样点与六个反向点判定都一样才取那个值，否则「不稳定」。
pub(super) fn across_sampling_points(tags: &[&'static str]) -> &'static str {
    if tags.iter().all(|tag| *tag == tags[0]) {
        tags[0]
    } else {
        "unstable"
    }
}

/// 格一级与跨取样点的判定行（登记第六节第 1、2 组，第八节，第十节 F1–F3、F7）。`has_all_points` 为假（只跑了一个取样点）时不出跨点的行。
pub(super) fn cell_and_verdict_lines(summaries: &[RunSummary], has_all_points: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let pick = |point: &str, root_ring_slots_per_region: u64, commit_generated_blocks_per_publish: u64, arm: FloorArm, is_compaction_enabled: bool| {
        summaries
            .iter()
            .filter(|summary| {
                let key = &summary.key;
                key.point.name == point
                    && key.root_ring_slots_per_region == root_ring_slots_per_region
                    && key.commit_generated_blocks_per_publish == commit_generated_blocks_per_publish
                    && key.arm == arm
                    && key.is_compaction_enabled == is_compaction_enabled
            })
            .collect::<Vec<&RunSummary>>()
    };
    let present_points: Vec<SamplingPoint> = SAMPLING_POINTS
        .iter()
        .copied()
        .filter(|point| summaries.iter().any(|summary| summary.key.point.name == point.name))
        .collect();
    // (S, c, 候选下标) → 每个取样点一项 (第 3 行, 第 4 行, 对照成立, F3)，按取样点次序。
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let mut across_points: Vec<((u64, u64, usize), Vec<(&'static str, &'static str, bool, bool)>)> = Vec::new();
    let mut flagged_intent_cells = 0;
    let mut unreadable_intent_cells = 0;
    let mut pooled_all_points = IntentErrorCounts::default();
    for point in &present_points {
        for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
            for commit_generated_blocks_per_publish in COMMIT_GENERATED_BLOCKS_CHOICES {
                let cell = format!(
                    "point={} root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish}",
                    point.name
                );
                let mut on_verdicts = Vec::new();
                let mut off_verdicts = Vec::new();
                for arm in FLOOR_ARMS {
                    for is_compaction_enabled in [true, false] {
                        let verdict = cell_arm_verdict(&pick(
                            point.name,
                            root_ring_slots_per_region,
                            commit_generated_blocks_per_publish,
                            arm,
                            is_compaction_enabled,
                        ));
                        lines.push(format!(
                            "name=cell_arm {cell} arm={} compaction={} deletion_block_growth={} deletion_failures_verdict={} deletion_block_growth_seeds={} deletion_failures_seeds={}",
                            arm.tag(),
                            if is_compaction_enabled { "on" } else { "off" },
                            verdict.growth,
                            verdict.failures,
                            verdict.growth_seeds,
                            verdict.failure_seeds
                        ));
                        if is_compaction_enabled {
                            on_verdicts.push(verdict);
                        } else {
                            off_verdicts.push(verdict);
                        }
                    }
                }
                let diverging = GrowthVerdict::Diverging.tag();
                let shared_premise_hit = on_verdicts.iter().chain(off_verdicts.iter()).all(|verdict| verdict.growth == diverging);
                lines.push(format!("name=shared_premise {cell} all_eight_deletion_block_growth_diverging={shared_premise_hit}"));
                for (candidate_index, arm) in CANDIDATE_FLOOR_ARMS.iter().enumerate() {
                    let control_holds = off_control_degrades(&off_verdicts[candidate_index]);
                    let on = &on_verdicts[candidate_index];
                    let verdict = candidate_cell_verdict((on.growth, on.failures), control_holds, shared_premise_hit);
                    lines.push(format!(
                        "name=control {cell} arm={} off_deletion_block_growth={} off_deletion_failures_verdict={} off_control_degrades={control_holds}",
                        arm.tag(),
                        off_verdicts[candidate_index].growth,
                        off_verdicts[candidate_index].failures
                    ));
                    lines.push(format!(
                        "name=cell_candidate {cell} arm={} deletion_block_growth={} deletion_failures_verdict={} verdict={verdict}",
                        arm.tag(),
                        on.growth,
                        on.failures
                    ));
                    let entry_key = (root_ring_slots_per_region, commit_generated_blocks_per_publish, candidate_index);
                    let entry = (on.growth, on.failures, control_holds, shared_premise_hit);
                    match across_points.iter_mut().find(|(key, _entries)| *key == entry_key) {
                        Some((_key, entries)) => entries.push(entry),
                        None => across_points.push((entry_key, vec![entry])),
                    }
                }
                let mut pooled = IntentErrorCounts::default();
                for arm in FLOOR_ARMS {
                    for summary in pick(point.name, root_ring_slots_per_region, commit_generated_blocks_per_publish, arm, true) {
                        pooled.add(&summary.intent_errors);
                    }
                }
                pooled_all_points.add(&pooled);
                let share_flag = match pooled.region_predicate_false_positive_share_reaches_threshold() {
                    Some(true) => {
                        flagged_intent_cells += 1;
                        "true"
                    }
                    Some(false) => "false",
                    None => {
                        unreadable_intent_cells += 1;
                        "unreadable"
                    }
                };
                lines.push(format!(
                    "name=intent_errors {cell} pooled=compaction_on_all_segment_floor_arms_all_seeds {} region_is_record_empty_true_region_became_openable_false_share_at_least_5_percent={share_flag}",
                    pooled.fields()
                ));
            }
        }
    }
    lines.push(format!(
        "name=verdict_row2_segment1 points={} cells_with_region_is_record_empty_true_region_became_openable_false_share_at_least_5_percent={flagged_intent_cells} cells_unreadable={unreadable_intent_cells} pooled_{} stop_unable_to_stop_starting_points=not_run_in_segment1",
        present_points.len(),
        pooled_all_points.fields()
    ));
    if has_all_points {
        lines.extend(across_point_lines(&across_points));
    }
    lines
}

type AcrossPointEntries = Vec<((u64, u64, usize), Vec<(&'static str, &'static str, bool, bool)>)>;

/// 跨取样点：每格每个候选第 3、4 行在七个取样点上翻不翻面（F7），合成九格的判定，再按登记第六节第 1 组末段取第 4 行模型那一半的候选。
fn across_point_lines(across_points: &AcrossPointEntries) -> Vec<String> {
    let mut lines = Vec::new();
    let mut verdicts_by_candidate: Vec<Vec<&'static str>> = vec![Vec::new(); CANDIDATE_FLOOR_ARMS.len()];
    let mut cells_all_three_fail = 0;
    for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
        for commit_generated_blocks_per_publish in COMMIT_GENERATED_BLOCKS_CHOICES {
            let mut cell_verdicts = Vec::new();
            for (candidate_index, arm) in CANDIDATE_FLOOR_ARMS.iter().enumerate() {
                let entries = &across_points
                    .iter()
                    .find(|(key, _entries)| *key == (root_ring_slots_per_region, commit_generated_blocks_per_publish, candidate_index))
                    .expect("每格每个候选都有")
                    .1;
                assert_eq!(entries.len(), SAMPLING_POINTS.len(), "七个取样点都有");
                let growth_tags: Vec<&'static str> = entries.iter().map(|entry| entry.0).collect();
                let failure_tags: Vec<&'static str> = entries.iter().map(|entry| entry.1).collect();
                let final_growth = across_sampling_points(&growth_tags);
                let final_failures = across_sampling_points(&failure_tags);
                let controls_hold_everywhere = entries.iter().all(|entry| entry.2);
                let shared_premise_hit_somewhere = entries.iter().any(|entry| entry.3);
                for (quantity, tags, final_tag) in
                    [("deletion_block_growth", &growth_tags, final_growth), ("deletion_failures_verdict", &failure_tags, final_failures)]
                {
                    let per_point: Vec<String> = SAMPLING_POINTS
                        .iter()
                        .zip(tags.iter())
                        .map(|(point, tag)| format!("{}={tag}", point.name))
                        .collect();
                    lines.push(format!(
                        "name=sensitivity root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish} arm={} quantity={quantity} {} final={final_tag}",
                        arm.tag(),
                        per_point.join(" ")
                    ));
                }
                let verdict = candidate_cell_verdict(
                    (final_growth, final_failures),
                    controls_hold_everywhere,
                    shared_premise_hit_somewhere,
                );
                verdicts_by_candidate[candidate_index].push(verdict);
                cell_verdicts.push(verdict);
            }
            let all_three_fail = cell_verdicts.iter().all(|verdict| *verdict == "fail");
            cells_all_three_fail += u64::from(all_three_fail);
            lines.push(format!(
                "name=fork_row4_cell root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish} segment_floor_one={} segment_floor_two={} segment_floor_formula={} segment_floor_formula_value={} segment_floor_formula_is_the_same_arm_as_segment_floor_two={} all_three_candidates_insufficient={all_three_fail}",
                cell_verdicts[0],
                cell_verdicts[1],
                cell_verdicts[2],
                formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish),
                formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish) == 2
            ));
        }
    }
    let count_of = |verdicts: &[&'static str], wanted: &str| verdicts.iter().filter(|verdict| **verdict == wanted).count();
    let mut fields = Vec::new();
    for (candidate_index, arm) in CANDIDATE_FLOOR_ARMS.iter().enumerate() {
        let verdicts = &verdicts_by_candidate[candidate_index];
        fields.push(format!(
            "{}_cells_pass={} {}_cells_fail={} {}_cells_not_judgeable={}",
            arm.tag(),
            count_of(verdicts, "pass"),
            arm.tag(),
            count_of(verdicts, "fail"),
            arm.tag(),
            count_of(verdicts, "not_judgeable")
        ));
    }
    let candidate_names = ["1", "2", "formula"];
    let smallest_passing_all_nine = (0..CANDIDATE_FLOOR_ARMS.len())
        .find(|candidate_index| verdicts_by_candidate[*candidate_index].iter().all(|verdict| *verdict == "pass"))
        .map_or("none", |candidate_index| candidate_names[candidate_index]);
    let smallest_without_failing_cell = (0..CANDIDATE_FLOOR_ARMS.len())
        .find(|candidate_index| verdicts_by_candidate[*candidate_index].iter().all(|verdict| *verdict != "fail"))
        .map_or("none", |candidate_index| candidate_names[candidate_index]);
    lines.push(format!(
        "name=verdict_row4_model_half {} cells_all_three_insufficient={cells_all_three_fail} smallest_candidate_passing_all_nine_cells={smallest_passing_all_nine} smallest_candidate_without_a_failing_cell={smallest_without_failing_cell} implementation_arm=not_run_segment3",
        fields.join(" ")
    ));
    lines
}

/// 登记第七节甲 1 的九格算式值 (S, c, 值)：跑前由设计员算出、执行员另算一遍对上（报告里有命令）。
pub(super) const FORMULA_VALUES_REGISTERED_BEFORE_THE_RUN: [(u64, u64, u64); 9] = [
    (4, 5, 2),
    (4, 10, 3),
    (4, 13, 4),
    (8, 5, 3),
    (8, 10, 5),
    (8, 13, 6),
    (16, 5, 5),
    (16, 10, 9),
    (16, 13, 11),
];

/// 第 1 行：九格算式值与跑前登记的值比。
pub(super) fn formula_lines() -> Vec<String> {
    FORMULA_VALUES_REGISTERED_BEFORE_THE_RUN
        .iter()
        .map(|(root_ring_slots_per_region, commit_generated_blocks_per_publish, registered)| {
            let value = formula_floor_segments(*root_ring_slots_per_region, *commit_generated_blocks_per_publish);
            format!(
                "name=formula root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish} ring_slots_minus_one_times_commit_generated_blocks={} segment_floor_formula={value} registered={registered} matches_registered={}",
                (ROOT_RING_REGIONS * root_ring_slots_per_region - 1) * commit_generated_blocks_per_publish,
                value == *registered
            )
        })
        .collect()
}

/// 第 7 行：只在几何不同的三个取样点上量（另外四个反向点的旋钮不碰空发布流）。
pub(super) fn metadata_stream_lines(selection: Option<ShardSelection>) -> Vec<String> {
    let mut lines = Vec::new();
    for point in SAMPLING_POINTS.iter().filter(|point| ["main", "disk_1gib", "leaf_203"].contains(&point.name)) {
        for root_ring_slots_per_region in ROOT_RING_SLOTS_PER_REGION_CHOICES {
            if !is_selected(selection, point, root_ring_slots_per_region) {
                continue;
            }
            for commit_generated_blocks_per_publish in COMMIT_GENERATED_BLOCKS_CHOICES {
                let geometry = PoolGeometry::new(
                    point.disk_slots,
                    point.leaf_width_in_slots,
                    root_ring_slots_per_region,
                    commit_generated_blocks_per_publish,
                );
                let peak = metadata_stream_peak_unopenable_segments(geometry);
                let formula = formula_floor_segments(root_ring_slots_per_region, commit_generated_blocks_per_publish);
                lines.push(format!(
                    "name=metadata_stream point={} root_ring_slots_per_region={root_ring_slots_per_region} commit_generated_blocks_per_publish={commit_generated_blocks_per_publish} empty_publishes={METADATA_STREAM_EMPTY_PUBLISHES} peak_unopenable_segments={peak} segment_floor_formula={formula} peak_minus_formula={} formula_undercounts={}",
                    point.name,
                    i128::from(peak) - i128::from(formula),
                    peak > formula
                ));
            }
        }
    }
    lines
}

/// 从一行 `name=run …` 里取回格一级判定要的那几样（`--merge` 用；单测钉住 格式化 → 取回 逐项相同）。
pub(super) fn run_summary_of_line(line: &str) -> Option<RunSummary> {
    let body = line.strip_prefix("E7RESULT ").unwrap_or(line);
    if !body.starts_with("name=run ") {
        return None;
    }
    let field = |wanted: &str| -> &str {
        body.split(' ')
            .find_map(|token| token.strip_prefix(wanted).and_then(|rest| rest.strip_prefix('=')))
            .unwrap_or_else(|| panic!("run 行缺字段 {wanted}：{body}"))
    };
    let number = |wanted: &str| -> u64 { field(wanted).parse().unwrap_or_else(|_| panic!("{wanted} 不是数：{body}")) };
    let point = *SAMPLING_POINTS.iter().find(|point| point.name == field("point")).expect("认得的取样点");
    let arm = *FLOOR_ARMS.iter().find(|arm| arm.tag() == field("arm")).expect("认得的臂");
    let growth = [GrowthVerdict::Bounded, GrowthVerdict::Diverging, GrowthVerdict::Unreadable]
        .into_iter()
        .find(|verdict| verdict.tag() == field("deletion_block_growth"))
        .expect("认得的第 3 行判定");
    let key = RunKey {
        point,
        root_ring_slots_per_region: number("root_ring_slots_per_region"),
        commit_generated_blocks_per_publish: number("commit_generated_blocks_per_publish"),
        arm,
        is_compaction_enabled: field("compaction") == "on",
        seed: number("seed"),
    };
    let intent_errors = IntentErrorCounts {
        ended: number("intents_ended"),
        discarded: number("intents_discarded"),
        judged: number("intents_judged"),
        region_predicate_true_truth_false: number("region_is_record_empty_true_region_became_openable_false"),
        region_predicate_false_truth_true: number("region_is_record_empty_false_region_became_openable_true"),
        global_difference_true_truth_false: number("global_difference_is_positive_true_region_became_openable_false"),
        global_difference_false_truth_true: number("global_difference_is_positive_false_region_became_openable_true"),
        global_difference_true_openable_rise_false: number("global_difference_is_positive_true_openable_count_rose_false"),
        global_difference_false_openable_rise_true: number("global_difference_is_positive_false_openable_count_rose_true"),
    };
    Some(RunSummary { key, growth, deletion_failures: number("deletion_failures"), intent_errors })
}

pub(super) fn header_lines() -> Vec<String> {
    let mut lines = vec![format!(
        "name=config segment=1 points={} root_ring_slots_per_region_choices=4/8/16 commit_generated_blocks_per_publish_choices=5/10/13 arms=segment_floor_one/segment_floor_two/segment_floor_formula/segment_floor_zero compaction=on/off seeds={SEEDS_PER_CELL} rounds={ROUNDS_FIRST_PASS}/{ROUNDS_DOUBLED_CHECK} units_per_deletion_publish={UNITS_PER_DELETION_PUBLISH} file_units=1..{FILE_UNITS_UPPER_BOUND} compaction_batch_units={COMPACTION_BATCH_UNITS} stop_arm=local_region_predicate selection=greedy unit_area_start={UNIT_AREA_START_SLOT} segment_slots={CLUSTER_SEGMENT_SLOTS} leaf_slots={ALLOCATION_RECORD_TREE_LEAF_SLOTS} fanout={ALLOCATION_RECORD_TREE_INTERNAL_FANOUT}",
        SAMPLING_POINTS.iter().map(|point| point.name).collect::<Vec<_>>().join("/")
    )];
    lines.extend(formula_lines());
    lines
}

/// 一个取样点（或全部）的第 7 行与全部正式跑的行，连同交给格一级的汇总。
pub(super) fn measured_lines(selection: Option<ShardSelection>) -> (Vec<String>, Vec<RunSummary>) {
    let mut lines = metadata_stream_lines(selection);
    let keys = all_run_keys(selection);
    let mut summaries = Vec::new();
    for (run_lines, summary) in run_fill_delete_fill_runs_in_parallel(&keys, worker_thread_count()) {
        lines.extend(run_lines);
        summaries.push(summary);
    }
    (lines, summaries)
}

/// 读一份分片产物：末行必须是 `name=done emitted=N` 且 N 等于行数（作废条款 V3），交回去掉收尾行的那些行。
pub(super) fn read_shard(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("读不了分片 {path}：{error}"));
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let done_line = lines.pop().expect("分片不空");
    assert_eq!(done_line, format!("E7RESULT name=done emitted={}", lines.len() + 1), "分片 {path} 的收尾行（V3）");
    lines
        .into_iter()
        .map(|line| line.strip_prefix("E7RESULT ").map(str::to_string).expect("分片的每一行都是 E7RESULT 行"))
        .collect()
}

/// 先打这一次准入的产物行（强制跑时一行 `name=preflight forced=…`，满足时不打），再打 `lines`，最后是收尾行。
pub(super) fn print_with_emitter(preflight_lines: &[String], lines: &[String]) {
    let mut emitter = Emitter::new();
    for line in preflight_lines.iter().chain(lines) {
        println!("{}", emitter.emit_raw(line));
    }
    println!("{}", emitter.finish());
}
