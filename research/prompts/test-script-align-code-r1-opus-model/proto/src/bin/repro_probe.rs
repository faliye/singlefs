//! K2-S1：随机历史六段取样各取种子基起的几个种子：取样那一路（run_history_campaign 一段一个种子）与复现那一路
//! （判红时打的复现行原样解析 → random_history_to_reproduce_from_environment → generate → execute_history_with）
//! 生成的历史与跑出来的结局、计数逐项比。
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    environment_assignments_in_line, execute_history_with, random_history_reproduction_line,
    random_history_to_reproduce_from_environment, run_history_campaign, FindingShrinking, GenerationWeights,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistorySampling, HistorySeed, PerStepChecker,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

fn main() {
    let small = |space_admission| HistoryExecution {
        per_step_checker: PerStepChecker::Run,
        device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
        space_admission,
    };
    let four = HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let samplings: Vec<(&str, HistorySampling)> = vec![
        ("fast", HistorySampling { operations_per_history: 30, weights: GenerationWeights::BROAD, execution: four }),
        ("reuse", HistorySampling { operations_per_history: 30, weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR, execution: four }),
        ("rollback", HistorySampling { operations_per_history: 30, weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR, execution: four }),
        ("wall", HistorySampling { operations_per_history: 150, weights: GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL, execution: four }),
        ("unit_area_wall_skipped", HistorySampling { operations_per_history: 150, weights: GenerationWeights::TOWARD_THE_UNIT_AREA_WALL, execution: small(SpaceAdmission::SkippedByTheTestOnlySwitch) }),
        ("unit_area_wall_judged", HistorySampling { operations_per_history: 150, weights: GenerationWeights::TOWARD_THE_UNIT_AREA_WALL, execution: small(SpaceAdmission::JudgedByTheFormula) }),
    ];
    let seeds_per_sampling: u64 = std::env::var("K2_SEEDS_PER_SAMPLING").ok().and_then(|text| text.parse().ok()).unwrap_or(4);
    let fast_defaults = samplings[0].1;
    let mut total = 0usize;
    let mut differing = 0usize;
    for (name, sampling) in &samplings {
        for offset in 0..seeds_per_sampling {
            let seed = HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + offset);
            let campaign = run_history_campaign(
                seed.0, 1, sampling.operations_per_history, &sampling.weights, sampling.execution, 1,
                FindingShrinking::ReportSeedsOnly,
            );
            let line = random_history_reproduction_line(seed, sampling);
            let assignments = environment_assignments_in_line(&line);
            let (read_seed, read_sampling) =
                random_history_to_reproduce_from_environment(&|variable: &str| assignments.get(variable).cloned(), &fast_defaults)
                    .expect("复现行读得回");
            let history = read_sampling.generate(read_seed);
            let run = execute_history_with(&history, read_sampling.execution, &SharedStream::new(), &mut |_| {});
            let campaign_ending = if !campaign.new_findings.is_empty() {
                "new_finding"
            } else if !campaign.known_red_hits.is_empty() {
                "known_red"
            } else {
                "completed"
            };
            let repro_ending = match &run.ending {
                HistoryEnding::NewFinding { .. } => "new_finding",
                HistoryEnding::KnownRed { .. } => "known_red",
                HistoryEnding::Completed => "completed",
            };
            let same = read_seed == seed && read_sampling == *sampling && campaign.tally == run.tally && campaign_ending == repro_ending;
            total += 1;
            if !same {
                differing += 1;
            }
            println!(
                "K2S1 name=seed sampling={name} seed_offset={offset} campaign_ending={campaign_ending} repro_ending={repro_ending} tally_equal={} sampling_equal={} same={same}",
                campaign.tally == run.tally, read_sampling == *sampling
            );
        }
    }
    println!("K2S1 name=total histories={total} differing={differing}");
}
