//! m3-batch10-code-r1 攻方腿（Opus）K4：孤儿历史与随机历史上，崩溃后那次可写挂载的每一段（写行、暖机、抹头……）各摆出了几个二次崩溃。
//! 调崩溃注入的库函数（不跑 checker 档包的测试）：写死的孤儿历史一段，加偏向留下孤儿那组比重的随机历史若干段（每段抽样摆几个第一次崩溃）。
use singlefs_checker_tier::crash_injection::{
    inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans, inject_crashes_into_history,
    CrashPointDraw,
};
use singlefs_harness::history::{generate_history_with_weights, GenerationWeights, HistoryDeviceWidth, HistoryExecution, HistorySeed, PerStepChecker};

fn main() {
    let execution = HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::FourGibibytes, space_admission: singlefs_core::admission::SpaceAdmission::JudgedByTheFormula };
    let fixed = inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(execution);
    println!("PHASES fixed-orphans crash_points={} second_by_phase={:?} new_findings={}", fixed.crash_points.len(), fixed.tally.second_crash_points_by_phase, fixed.new_findings.len());
    let seeds: u64 = std::env::var("SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(4);
    let steps: usize = std::env::var("STEPS").ok().and_then(|s| s.parse().ok()).unwrap_or(30);
    let per: usize = std::env::var("PER").ok().and_then(|s| s.parse().ok()).unwrap_or(4);
    for seed in 0..seeds {
        let history = generate_history_with_weights(HistorySeed(seed), steps, &GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP);
        let run = inject_crashes_into_history(&history, execution, CrashPointDraw::Sampled { crash_points_per_history: per }, None);
        println!("PHASES orphans-seed{seed} crash_points={} second_by_phase={:?} new_findings={}", run.crash_points.len(), run.tally.second_crash_points_by_phase, run.new_findings.len());
    }
}
