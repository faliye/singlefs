//! K2-S2：崩溃注入小快档判红时打的「重放这一整批」那一行，照抄喂回大档入口（crash_injection_campaign_from_environment，
//! 默认值照 tests/crash_injection_campaign.rs 的 large_tier_campaign_from），同一批历史、同一组参数，大档入口判不判红。
//! 副本 tree2 里改坏一处：draw_second_crash_points 一个二次崩溃都不摆（小快档「挂载途中一个二次崩溃都没摆出来」那一条该红）。
use singlefs_checker_tier::crash_injection::{
    crash_injection_campaign_from_environment, run_crash_injection_campaign, CrashInjectionCampaign,
    CrashInjectionReport, CrashInjectionWorkerThreads, CrashPointDraw, FailureImageRetention,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    environment_assignments_in_line, GenerationWeights, HistoryDeviceWidth, HistoryExecution, PerStepChecker,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};

const UNCHECKED: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 小快档那条用例在报告上的全部断言（tests/crash_injection_campaign.rs 的 crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed）。
fn small_tier_reds(report: &CrashInjectionReport, seeds: u64) -> Vec<String> {
    let tally = &report.tally;
    let mut reds = Vec::new();
    if !report.new_findings.is_empty() {
        reds.push("new_findings".to_string());
    }
    if tally.crash_points < seeds {
        reds.push("crash_points_below_seeds".to_string());
    }
    for (stage, runs) in [
        ("model", tally.model_judgements),
        ("checker", tally.checker_runs),
        ("record_check", tally.record_checks),
        ("writable_mount", tally.writable_mounts_after_the_crash),
        ("checker_after_mount", tally.checker_runs_after_the_writable_mount),
        ("record_check_after_mount", tally.record_checks_after_the_writable_mount),
    ] {
        if runs != tally.crash_points {
            reds.push(format!("every_crash_point_ran_{stage}"));
        }
    }
    if tally.second_crash_points < 1 {
        reds.push("no_second_crash_point".to_string());
    }
    for (stage, runs) in [
        ("model", tally.second_crash_model_judgements),
        ("checker", tally.second_crash_checker_runs),
        ("record_check", tally.second_crash_record_checks),
    ] {
        if runs != tally.second_crash_points {
            reds.push(format!("every_second_crash_ran_{stage}"));
        }
    }
    if tally.recoveries_failed != 0 {
        reds.push("recoveries_failed".to_string());
    }
    reds
}

fn main() {
    let seeds = 4;
    let campaign = CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: seeds,
        operations_per_history: 24,
        draw: CrashPointDraw::Sampled { crash_points_per_history: 4 },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(4),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
    };
    let report = run_crash_injection_campaign(&campaign);
    let line = report.replay_line();
    println!(
        "K2S2 name=small_fast_tier crash_points={} second_crash_points={} new_findings={} small_tier_reds={:?}",
        report.tally.crash_points, report.tally.second_crash_points, report.new_findings.len(), small_tier_reds(&report, seeds)
    );
    println!("K2S2 name=replay_line {line}");
    let assignments = environment_assignments_in_line(&line);
    let replayed_campaign = crash_injection_campaign_from_environment(
        &|variable: &str| assignments.get(variable).cloned(),
        &CrashInjectionCampaign {
            first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            seed_count: 500,
            operations_per_history: 40,
            draw: CrashPointDraw::Sampled { crash_points_per_history: 8 },
            weights: GenerationWeights::BROAD,
            execution: UNCHECKED,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(4),
            image_retention: FailureImageRetention::KeepTheImageFiles,
        },
    )
    .expect("重放行读得回");
    let replayed = run_crash_injection_campaign(&replayed_campaign);
    println!(
        "K2S2 name=large_tier_entry_from_the_replay_line same_parameters={} crash_points={} second_crash_points={} new_findings={} large_tier_red={} tally_equal={}",
        replayed_campaign.first_seed == campaign.first_seed
            && replayed_campaign.seed_count == campaign.seed_count
            && replayed_campaign.operations_per_history == campaign.operations_per_history
            && replayed_campaign.draw == campaign.draw
            && replayed_campaign.weights == campaign.weights
            && replayed_campaign.execution == campaign.execution,
        replayed.tally.crash_points, replayed.tally.second_crash_points, replayed.new_findings.len(),
        !replayed.new_findings.is_empty(), replayed.tally == report.tally
    );
}
