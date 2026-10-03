//! L2：数值环境变量解析收成一份之后，同一个取值在几处入口的结局（接受成什么 / panic 的消息 / 交回 Err）。
//! 一处一行 `L2 site=… value=… outcome=…`；panic 用 catch_unwind 收，钩子换成不打印。最后一段：崩溃注入大档入口种子数写 0 照收之后，
//! 那一批真跑起来（0 段）判出什么（`L2ZERO`）。
use std::num::NonZeroUsize;
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_checker_tier::crash_amplification::{checking_threads_from_the_variable_value, ExpandUpToWritesPerSegment};
use singlefs_checker_tier::crash_injection::{
    crash_injection_campaign_from_environment, run_crash_injection_campaign, CrashInjectionAssertions, CrashInjectionCampaign,
    CrashInjectionWorkerThreads, CrashPointDraw, FailureImageRetention,
};
use singlefs_harness::history::{GenerationWeights, HistoryExecution};
use singlefs_harness::numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable as parse;

fn outcome<T: std::fmt::Debug>(body: impl FnOnce() -> T) -> String {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => format!("accepted:{value:?}"),
        Err(payload) => {
            let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string())).unwrap_or_default();
            format!("panic:{}", text.chars().take(90).collect::<String>().replace('\n', "\\n"))
        }
    }
}

fn defaults() -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: 1,
        seed_count: 2,
        operations_per_history: 3,
        draw: CrashPointDraw::Sampled { crash_points_per_history: 4 },
        weights: GenerationWeights::BROAD,
        execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let numbers = ["8", " 8", "8\n", "+8", "08", "", " ", "0", "eight", "8\u{3000}"];
    for value in numbers {
        println!("L2 site=common::<usize> value={value:?} outcome={}", outcome(|| parse::<usize>("V", Some(value))));
        println!("L2 site=common::<NonZeroUsize> value={value:?} outcome={}", outcome(|| parse::<NonZeroUsize>("V", Some(value))));
        println!("L2 site=crash_amplification::checking_threads value={value:?} outcome={}", outcome(|| checking_threads_from_the_variable_value(Some(value))));
        println!("L2 site=crash_amplification::expand_up_to value={value:?} outcome={}", outcome(|| ExpandUpToWritesPerSegment::from_value_of_the_variable(Some(value), 6).0));
    }
    // 崩溃注入重放入口：一次只设一个变量
    let variables_and_values: [(&str, &[&str]); 10] = [
        ("SINGLEFS_CRASH_INJECTION_FIRST_SEED", &["7", " 7", "7\n", "0", ""]),
        ("SINGLEFS_CRASH_INJECTION_SEEDS", &["5", " 5", "0", ""]),
        ("SINGLEFS_CRASH_INJECTION_POINTS", &["4", " 4", "0", ""]),
        ("SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES", &["3", " 3", "none", " none", "none\n", "0", ""]),
        ("SINGLEFS_CRASH_INJECTION_OPERATIONS", &["40", " 40", "40\n", "0", ""]),
        ("SINGLEFS_CRASH_INJECTION_ASSERTIONS", &["fast-tier", " fast-tier", "fast-tier\n", ""]),
        ("SINGLEFS_CRASH_INJECTION_DEVICES", &["4gib", " 4gib", ""]),
        ("SINGLEFS_CRASH_INJECTION_WEIGHTS", &["broad", "broad\n"]),
        ("SINGLEFS_CRASH_INJECTION_CHECKER", &["run", " run"]),
        ("SINGLEFS_CRASH_INJECTION_SPACE_ADMISSION", &["judged", "judged "]),
    ];
    for (variable, values) in variables_and_values {
        for value in values {
            let lookup = |name: &str| (name == variable).then(|| (*value).to_string());
            let result = outcome(|| {
                let campaign = crash_injection_campaign_from_environment(&lookup, &defaults());
                format!("first_seed={} seeds={} operations={} draw={:?} assertions={:?}", campaign.first_seed, campaign.seed_count, campaign.operations_per_history, campaign.draw, campaign.assertions)
            });
            println!("L2 site=crash_injection_campaign_from_environment variable={variable} value={value:?} outcome={result}");
        }
    }
    // 种子数写 0：入口照收，那一批真跑（0 段），只判清单外失败的那一组照样判绿
    let zero = crash_injection_campaign_from_environment(&|name: &str| (name == "SINGLEFS_CRASH_INJECTION_SEEDS").then(|| "0".to_string()), &defaults());
    let report = run_crash_injection_campaign(&zero);
    println!(
        "L2ZERO seeds={} histories_run_crash_points={} new_findings={} assertions={:?} judged_by_new_findings_only_would_be_green={}",
        report.seed_count, report.tally.crash_points, report.new_findings.len(), report.assertions, report.new_findings.is_empty()
    );
}
