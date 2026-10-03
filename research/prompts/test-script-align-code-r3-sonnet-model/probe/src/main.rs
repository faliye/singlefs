//! r3 L2 探针：在冻结副本的真代码上量「写错、写 0、空串、前后空白、非 UTF-8」各入口的结局。只读冻结副本，不改它。
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt as _;
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_checker_tier::bad_disk_input::{
    bad_disk_campaign_from_environment, BadDiskCampaign, BadDiskInputWorkerThreads,
};
use singlefs_checker_tier::crash_injection::{
    crash_injection_campaign_from_environment, CrashInjectionAssertions, CrashInjectionCampaign,
    CrashInjectionWorkerThreads, CrashPointDraw, FailureImageRetention,
};
use singlefs_harness::fault_injection::FaultInjectionWorkerThreads;
use singlefs_harness::history::{
    number_from_reproduction_environment, GenerationWeights, HistoryExecution,
};
use singlefs_harness::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};

fn outcome<T: std::fmt::Debug>(f: impl FnOnce() -> T) -> String {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => format!("OK {value:?}"),
        Err(payload) => {
            let text = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "?".to_string());
            format!("PANIC {}", text.chars().take(110).collect::<String>())
        }
    }
}

fn crash_defaults() -> CrashInjectionCampaign {
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
    let values: [(&str, &str); 8] = [
        ("empty", ""),
        ("blank", " "),
        ("lead-space", " 8"),
        ("trail-newline", "8\n"),
        ("plus", "+8"),
        ("zero", "0"),
        ("garbage", "eight"),
        ("plain", "8"),
    ];
    println!("== A 两种解析函数对同一取值的结局（u64 / NonZeroUsize）");
    for (name, value) in values {
        println!(
            "A {name:14} 共用解析(panic版)::<u64>      = {}",
            outcome(|| number_from_the_variable_value_panicking_when_unreadable::<u64>("V", Some(value)))
        );
        println!(
            "A {name:14} history::number_from_reproduction_environment(Result版) = {}",
            outcome(|| number_from_reproduction_environment(&|_| Some(value.to_string()), "V", 0).map_err(|e| e.render()))
        );
        println!(
            "A {name:14} 共用解析::<NonZeroUsize>      = {}",
            outcome(|| number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>("V", Some(value)))
        );
    }
    println!("== B 同一个入口 crash_injection_campaign_from_environment 里不同变量的空白口径");
    for (variable, value) in [
        ("SINGLEFS_CRASH_INJECTION_FIRST_SEED", " 11"),
        ("SINGLEFS_CRASH_INJECTION_SEEDS", " 12"),
        ("SINGLEFS_CRASH_INJECTION_POINTS", " 4"),
        ("SINGLEFS_CRASH_INJECTION_OPERATIONS", " 24"),
        ("SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES", " none"),
        ("SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES", " 5"),
        ("SINGLEFS_CRASH_INJECTION_FIRST_SEED", ""),
        ("SINGLEFS_CRASH_INJECTION_OPERATIONS", ""),
        ("SINGLEFS_CRASH_INJECTION_ASSERTIONS", ""),
    ] {
        let result = outcome(|| {
            let c = crash_injection_campaign_from_environment(
                &|asked: &str| (asked == variable).then(|| value.to_string()),
                &crash_defaults(),
            );
            (c.first_seed, c.seed_count, c.operations_per_history, c.draw)
        });
        println!("B {variable}={value:?} -> {result}");
    }
    println!("== C 坏盘输入入口（Result 版）同样的变量");
    let bad_defaults = BadDiskCampaign {
        first_seed: 1,
        seed_count: 2,
        operations_per_history: 3,
        weights: GenerationWeights::BROAD,
        execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: BadDiskInputWorkerThreads::GivenByCaller(1),
    };
    for (variable, value) in [
        ("SINGLEFS_BAD_DISK_FIRST_SEED", " 11"),
        ("SINGLEFS_BAD_DISK_FIRST_SEED", ""),
        ("SINGLEFS_BAD_DISK_FIRST_SEED", "eleven"),
        ("SINGLEFS_BAD_DISK_OPERATIONS", " 24"),
    ] {
        let result = outcome(|| {
            bad_disk_campaign_from_environment(
                &|asked: &str| (asked == variable).then(|| value.to_string()),
                &bad_defaults,
            )
            .map(|c| (c.first_seed, c.seed_count, c.operations_per_history))
            .map_err(|e| e.render())
        });
        println!("C {variable}={value:?} -> {result}");
    }
    println!("== D 非 UTF-8 的环境变量，各入口读进程环境");
    let raw = OsString::from_vec(vec![0x38, 0xff]);
    for name in [
        "R3_PROBE_VAR",
        "SINGLEFS_CRASH_INJECTION_THREADS",
        "SINGLEFS_FAULT_INJECTION_WORKER_THREADS",
    ] {
        std::env::set_var(name, &raw);
    }
    println!("D process_environment_variable            = {}", outcome(|| process_environment_variable("R3_PROBE_VAR")));
    println!("D CrashInjectionWorkerThreads::from_the_environment = {}", outcome(CrashInjectionWorkerThreads::from_the_environment));
    println!("D FaultInjectionWorkerThreads::from_the_environment = {}", outcome(FaultInjectionWorkerThreads::from_the_environment));
    println!("== E 写 0 / 空串 / 前后空白的线程数，经三个公开入口");
    for value in ["0", "", " 4", "four"] {
        std::env::set_var("SINGLEFS_CRASH_INJECTION_THREADS", value);
        std::env::set_var("SINGLEFS_FAULT_INJECTION_WORKER_THREADS", value);
        println!("E value={value:?} 崩溃注入 = {}", outcome(|| CrashInjectionWorkerThreads::from_the_environment().count()));
        println!("E value={value:?} 故障注入 = {}", outcome(|| FaultInjectionWorkerThreads::from_the_environment().count()));
    }
}
