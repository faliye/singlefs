//! 攻方腿 m2-supp3-item3-code-r2（K1、K3）的量具：只在副本上跑，不入库。
//! 快档的同一组参数（种子基起 24 段、每段 24 步、Sampled{4}、BROAD、不跑每步 checker、4 GiB），
//! 把每个崩溃状态的段长、段内扣下几个写、同段镜像对被摆成什么形态、记录核对器判了什么，逐个数出来。

use std::collections::BTreeMap;

use singlefs_harness::crash::{writes_and_segments_with_stream_indexes, RetainedWrite};
use singlefs_harness::crash_injection::{
    inject_crashes_into_history, CrashPointDraw, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistorySeed, PerStepChecker,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::SharedStream;

const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
};

const FAST_TIER_SEEDS: u64 = 24;
const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 24;

/// 同段镜像对：同一段里两次写，盘不同、写下的字节逐字节相同（journal 记录与单元写都是双写）。
fn mirror_pairs_in(writes: &[RetainedWrite], segment: &[usize]) -> Vec<(usize, usize, StepKind)> {
    let mut pairs = Vec::new();
    for (a, first) in segment.iter().enumerate() {
        for (b, second) in segment.iter().enumerate().skip(a + 1) {
            let one = &writes[*first];
            let other = &writes[*second];
            if one.device != other.device && one.bytes == other.bytes && one.kind == other.kind {
                pairs.push((a, b, one.kind));
            }
        }
    }
    pairs
}

#[test]
fn probe_fast_tier_subset_coverage_and_record_checks() {
    let mut crash_points_total = 0u64;
    let mut holes_total = 0u64;
    let mut record_checks = 0u64;
    let mut record_root_without_record = 0u64;
    let mut record_claimed_state_missing_unit = 0u64;
    let mut withheld_histogram: BTreeMap<usize, u64> = BTreeMap::new();
    let mut segment_length_histogram: BTreeMap<usize, u64> = BTreeMap::new();
    let mut withheld_fraction_buckets: BTreeMap<&'static str, u64> = BTreeMap::new();
    // 同段镜像对：全表有几对、被某个崩溃状态摆成「只持久后一份」的有几对（按对去重）、
    // 摆成「只持久前一份」的有几对、两份都持久 / 都扣下的有几对。
    let mut mirror_pairs_in_candidate_segments = 0u64;
    let mut mirror_pairs_in_all_segments = 0u64;
    let mut mirror_pair_only_later_persisted: std::collections::BTreeSet<(u64, usize, usize, usize)> =
        std::collections::BTreeSet::new();
    let mut mirror_pair_only_earlier_persisted: std::collections::BTreeSet<(u64, usize, usize, usize)> =
        std::collections::BTreeSet::new();
    let mut mirror_pair_observations: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut mirror_pairs_by_kind: BTreeMap<&'static str, u64> = BTreeMap::new();
    // 「一次发布的某个单元两份副本都被扣下、这一段里别的写全持久」这种形态摆出来几个。
    let mut exactly_one_mirror_pair_withheld_rest_persisted = 0u64;
    let mut segments_touched: std::collections::BTreeSet<(u64, usize)> = std::collections::BTreeSet::new();
    let mut candidate_segments_total = 0u64;
    let mut candidate_segment_lengths: BTreeMap<usize, u64> = BTreeMap::new();
    let mut mirror_pairs_in_candidate_segments_all = 0u64;
    let mut proper_subsets_in_candidate_segments = 0u128;
    let mut withheld_by_segment_length: BTreeMap<usize, BTreeMap<usize, u64>> = BTreeMap::new();
    let mut one_pair_withheld_rest_persisted_by_length: BTreeMap<usize, u64> = BTreeMap::new();

    for offset in 0..FAST_TIER_SEEDS {
        let seed = HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(offset));
        let history = generate_history_with_weights(
            seed,
            FAST_TIER_OPERATIONS_PER_HISTORY,
            &GenerationWeights::BROAD,
        );
        let injection = inject_crashes_into_history(
            &history,
            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            CrashPointDraw::Sampled {
                crash_points_per_history: 4,
            },
            None,
        );
        crash_points_total += injection.tally.crash_points;
        holes_total += injection
            .tally
            .crash_points_withholding_a_write_before_a_persisted_one;
        record_checks += injection.tally.record_checks;
        record_root_without_record += injection.tally.record_root_without_record;
        record_claimed_state_missing_unit += injection.tally.record_claimed_state_missing_unit;

        // 同一段历史再跑一遍，拿写表与段表（与注入里那一份逐项相同：同一个种子、同一个跑法）。
        let stream = SharedStream::retaining_contents();
        let mut after_the_last_finished_step = 0usize;
        let run = execute_history_with(
            &history,
            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            &stream,
            &mut |_observation| {
                after_the_last_finished_step = stream.operation_count();
            },
        );
        let operations = stream.retained_operations();
        let parameters = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES.device_width.parameters();
        let geometry = FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        };
        let (writes, segments, stream_indexes) =
            writes_and_segments_with_stream_indexes(&operations, &geometry);
        let after_make_filesystem = run.operations_written_by_make_filesystem;
        for segment in &segments {
            mirror_pairs_in_all_segments += mirror_pairs_in(&writes, segment).len() as u64;
        }
        // 候选段：与 draw_crash_points 同一条判定（整段的写都落在 mkfs 之后、最后一个跑完的操作之前）。
        for (segment_index, segment) in segments.iter().enumerate() {
            let is_candidate = segment.iter().all(|write| {
                (after_make_filesystem..after_the_last_finished_step).contains(&stream_indexes[*write])
            });
            if !is_candidate {
                continue;
            }
            candidate_segments_total += 1;
            *candidate_segment_lengths.entry(segment.len()).or_insert(0) += 1;
            let pairs = mirror_pairs_in(&writes, segment);
            mirror_pairs_in_candidate_segments_all += pairs.len() as u64;
            // 这一段里「只持久后一份」那一类状态占全部真子集的比例（段长 n 时恒是 1/4，给人看的是绝对数）：
            proper_subsets_in_candidate_segments +=
                (1u128 << segment.len()) - 1;
            let _ = segment_index;
        }

        for crash_point in &injection.crash_points {
            let segment = &segments[crash_point.segment_index];
            assert_eq!(
                segment.len(),
                crash_point.persisted_within_the_segment.len(),
                "段长对不上：重放出来的段表与注入里那一份不是同一张"
            );
            segments_touched.insert((seed.0, crash_point.segment_index));
            let length = segment.len();
            let withheld = crash_point
                .persisted_within_the_segment
                .iter()
                .filter(|persisted| !**persisted)
                .count();
            *withheld_histogram.entry(withheld).or_insert(0) += 1;
            *withheld_by_segment_length
                .entry(length)
                .or_default()
                .entry(withheld)
                .or_insert(0) += 1;
            *segment_length_histogram.entry(length).or_insert(0) += 1;
            let fraction = (withheld as f64) / (length as f64);
            let bucket = if withheld == 1 {
                "扣下恰好 1 个"
            } else if withheld == 2 {
                "扣下恰好 2 个"
            } else if fraction <= 0.25 {
                "扣下不到四分之一"
            } else if fraction <= 0.75 {
                "扣下四分之一到四分之三"
            } else {
                "扣下超过四分之三"
            };
            *withheld_fraction_buckets.entry(bucket).or_insert(0) += 1;

            let pairs = mirror_pairs_in(&writes, segment);
            let mut withheld_pairs = 0usize;
            let mut withheld_outside_pairs = 0usize;
            let in_a_pair: std::collections::BTreeSet<usize> =
                pairs.iter().flat_map(|(a, b, _)| [*a, *b]).collect();
            for (a, b, kind) in &pairs {
                let first = crash_point.persisted_within_the_segment[*a];
                let second = crash_point.persisted_within_the_segment[*b];
                let shape = match (first, second) {
                    (true, true) => "两份都持久",
                    (false, true) => "只持久后一份",
                    (true, false) => "只持久前一份",
                    (false, false) => "两份都扣下",
                };
                *mirror_pair_observations.entry(shape).or_insert(0) += 1;
                *mirror_pairs_by_kind.entry(kind.name()).or_insert(0) += 1;
                if !first && second {
                    mirror_pair_only_later_persisted.insert((
                        seed.0,
                        crash_point.segment_index,
                        *a,
                        *b,
                    ));
                }
                if first && !second {
                    mirror_pair_only_earlier_persisted.insert((
                        seed.0,
                        crash_point.segment_index,
                        *a,
                        *b,
                    ));
                }
                if !first && !second {
                    withheld_pairs += 1;
                }
            }
            for (within, persisted) in crash_point.persisted_within_the_segment.iter().enumerate() {
                if !persisted && !in_a_pair.contains(&within) {
                    withheld_outside_pairs += 1;
                }
            }
            if withheld_pairs == 1 && withheld_outside_pairs == 0 && withheld == 2 {
                exactly_one_mirror_pair_withheld_rest_persisted += 1;
                *one_pair_withheld_rest_persisted_by_length
                    .entry(length)
                    .or_insert(0) += 1;
            }
        }
        // 候选段里的镜像对数：候选段判定与注入里那一份同一条（mkfs 之后、最后一个跑完的操作之前），
        // 这里只按「被摆到过的段」算下界，别的段这一批种子上没被抽到。
        for (_, segment_index) in segments_touched.iter().filter(|(s, _)| *s == seed.0) {
            mirror_pairs_in_candidate_segments +=
                mirror_pairs_in(&writes, &segments[*segment_index]).len() as u64;
        }
    }

    println!("── 攻方副本量具：快档 24 段 × 24 步 × Sampled{{4}} ──");
    println!("崩溃状态 {crash_points_total} 个；段内有洞（后发的写先持久）{holes_total} 个");
    println!(
        "记录核对器：跑了 {record_checks} 次；root_without_record 判红 {record_root_without_record} 次；claimed_state_missing_unit 判红 {record_claimed_state_missing_unit} 次"
    );
    println!("被摆到过的（种子, 段号）{} 个", segments_touched.len());
    println!("段长直方图（段长 → 崩溃状态数）：{segment_length_histogram:?}");
    println!("段内扣下几个写的直方图（扣下数 → 崩溃状态数）：{withheld_histogram:?}");
    println!("扣下比例分桶：{withheld_fraction_buckets:?}");
    println!(
        "同段镜像对：全部段里 {mirror_pairs_in_all_segments} 对；被摆到过的段里合计 {mirror_pairs_in_candidate_segments} 对（按崩溃状态累计，未去重）"
    );
    println!("镜像对在崩溃状态上的形态（按崩溃状态累计）：{mirror_pair_observations:?}");
    println!("镜像对按种类（按崩溃状态累计）：{mirror_pairs_by_kind:?}");
    println!(
        "被摆出「只持久后一份」的镜像对（按 (种子,段,对) 去重）：{} 对；「只持久前一份」：{} 对",
        mirror_pair_only_later_persisted.len(),
        mirror_pair_only_earlier_persisted.len()
    );
    println!(
        "「恰好一对镜像两份都扣下、这一段别的写全持久」的崩溃状态：{exactly_one_mirror_pair_withheld_rest_persisted} 个，按段长分 {one_pair_withheld_rest_persisted_by_length:?}"
    );
    println!("候选段合计 {candidate_segments_total} 段；候选段段长直方图 {candidate_segment_lengths:?}");
    println!(
        "候选段里的同段镜像对合计 {mirror_pairs_in_candidate_segments_all} 对；候选段的真子集总数 {proper_subsets_in_candidate_segments}"
    );
    println!("按段长分的「扣下几个写」直方图：{withheld_by_segment_length:?}");
}
