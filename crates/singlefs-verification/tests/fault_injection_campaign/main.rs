//! 里程碑「覆盖写、释放、回退与复用」增补 3 第 4 件：故障注入。通用的设备包装、注入点怎么摆、注入之后怎么判在
//! `singlefs_verification::fault_injection`；这里是故障注入这一批（不设变量跑默认批，设了 `SINGLEFS_FAULT_INJECTION_*` 照变量跑）、多线程对拍与写死的那几条用例。
//!
//! 被测的两条性质：注入之后 `singlefs-core` **返回错误而不是 panic**；出错之后**重开恢复到模型允许的版本**。
//! 种子基是这个测试周期写死的那一个（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，随机历史与崩溃注入那两个二进制用的是同一个）。
#[path = "../common/mod.rs"]
mod common;
#[path = "../common_images_for_the_given_states_engine/mod.rs"]
mod common_images_for_the_given_states_engine;
#[path = "../common_shared_by_harness_scenarios/mod.rs"]
mod common_shared_by_harness_scenarios;
#[path = "../common_images_for_the_given_states_engine/fault_injection_campaign.rs"]
mod scenarios_of_fault_injection_campaign;

use common_images_for_the_given_states_engine::PoolCheckerImages;
use scenarios_of_fault_injection_campaign::{
    ordinal_of_the_root_slot_write_within_the_step, overwrite_with_fill_seed,
    the_fixed_history_with_its_root_slot_write_swallowed,
    the_history_whose_make_filesystem_fails_at_its_first_barrier,
    DEFAULT_BATCH_OPERATIONS_PER_HISTORY, HANDING_EVERY_STEP_IMAGE_OUT,
};
use singlefs_checker::image::ViolationSite;
use singlefs_core::address::SlotNumber;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{AllocationRecord, AllocationRecordQuarantineMark, Placement};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration, recover, JournalPolicy,
};
use singlefs_core::transaction::{
    CopyQuarantinedAfterReleaseChecksumMismatch, QuarantinedCopyReading, TransactionUnit,
};
use singlefs_verification::fault_injection::{
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault,
    invariants_still_violated_on_the_comparison_image,
    unit_start_slots_quarantined_over_a_swallowed_write, FailureAfterAFault,
    PoolCheckerVerdictsOfTheImagesAfterAFault, SwallowedWriteByteRange, ViolationsNamingTheSites,
};
use singlefs_verification::fault_injection::{
    inject_faults_into_history,
    inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine,
    inject_one_fault_into_the_segment, run_fault_injection_campaign, AcquiredInstanceLeft,
    AcquiredInstanceLeftAfterAFailedMount, DrawnFault, FaultCounting, FaultDeviceSelector,
    FaultInjectionCampaign, FaultInjectionReport, FaultInjectionTally, FaultInjectionWorkerThreads,
    FaultOccurrence, FaultOutcome, FaultPlacement, FaultSchedule, FaultedSegment,
    FaultedSegmentKind, FlippedBit, ImagesAfterAFault, InjectedFault,
    PoolCheckerVerdictsAfterAFault, SharedFaultPlan,
};
use singlefs_verification::history::CopyQuarantinedByTheImplementation;
use singlefs_verification::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_verification::history::{
    count_from_reproduction_environment, environment_assignments_in_line, execute_history_with,
    execute_history_with_faults, generate_history_with_weights,
    number_from_reproduction_environment, render_environment_assignments, AppliedEffect,
    CompactionChoice, CompactionRunEnding, FailureSignature, FloorTargetChoice, GeneratedHistory,
    GenerationWeights, HarnessJudgement, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryOperationKind, HistoryRun, HistorySampling, HistorySeed,
    HistoryStartingPoint, PerStepChecker, ReproductionEnvironmentError, ReproductionVariablePrefix,
    StartingPointStep, StepOutcome, StepPosition,
};
use singlefs_verification::memory_pool::{root_identity_written_by, MemoryPool};
use singlefs_verification::model::{ModelDisagreementAspect, ModelReportedUnreadablePlace};
use singlefs_verification::numeric_environment_variable::process_environment_variable;
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RecordedOperationKind, SharedStream};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;

mod default_batch_variables_and_replay_lines;
mod pool_checker_verdicts_merged_from_the_given_states_engine;
mod read_faults_on_compaction_and_overwrites;
mod starting_segment_and_make_filesystem_faults;
mod swallowed_writes_left_by_the_lying_device;
mod write_errors_after_the_acquisition;

/// 默认批的规模：段数、每段步数、每段注入几次。每次注入都要把这段历史整个重跑一遍，所以规模按「别显著变慢」定。
const DEFAULT_BATCH_SEEDS: u64 = 24;

const DEFAULT_BATCH_FAULTS_PER_HISTORY: usize = 4;

/// 报告直接写进进程的标准输出，不经 libtest 的捕获：通过时计数照样出现在 `check.sh` 的输出里
/// （`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

/// 判红时先把种子基与规模说清楚：同一个构建加同一个种子基重放得出来。
fn how_to_replay(report: &FaultInjectionReport) -> String {
    format!("重放：{}", fault_injection_replay_line(report))
}

/// 故障注入那个二进制的环境变量前缀；取样参数（每段步数、比重、checker、盘宽、空间准入）的后缀与随机历史共用
/// （`singlefs_verification::history::HistorySamplingVariable`，格式只有那一份定义；代码三方 test-script-align-code-r1 判决 C607）。
const FAULT_INJECTION_VARIABLE_PREFIX: ReproductionVariablePrefix =
    ReproductionVariablePrefix("SINGLEFS_FAULT_INJECTION_");

singlefs_verification::enum_listing_every_member! {
/// 重放行与故障注入入口里故障注入自己的几个变量（接在 [`FAULT_INJECTION_VARIABLE_PREFIX`] 后面；取样参数接在它们后面）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultInjectionVariable {
    /// 种子区间的起点。
    FirstSeed,
    /// 种子数。
    Seeds,
    /// 每段注入几次。
    Faults,
}
}

impl FaultInjectionVariable {
    fn variable_name(self) -> String {
        FAULT_INJECTION_VARIABLE_PREFIX.variable_named(match self {
            FaultInjectionVariable::FirstSeed => "FIRST_SEED",
            FaultInjectionVariable::Seeds => "SEEDS",
            FaultInjectionVariable::Faults => "FAULTS",
        })
    }
}

/// 重放这一整批的那一行命令：种子区间、每段注入几次、取样参数，交故障注入那条入口照抄跑的就是原来那一批。
/// 线程数不在里面：线程数不改结论（`the_report_is_the_same_text_with_one_worker_thread_and_with_four`）。
fn fault_injection_replay_line(report: &FaultInjectionReport) -> String {
    let mut assignments: Vec<(String, String)> = FaultInjectionVariable::ALL
        .into_iter()
        .map(|variable| {
            let value = match variable {
                FaultInjectionVariable::FirstSeed => report.first_seed.to_string(),
                FaultInjectionVariable::Seeds => report.seed_count.to_string(),
                FaultInjectionVariable::Faults => report.faults_per_history.to_string(),
            };
            (variable.variable_name(), value)
        })
        .collect();
    let sampling = HistorySampling {
        operations_per_history: report.operations_per_history,
        weights: report.weights,
        execution: report.execution,
    };
    assignments.extend(sampling.environment_assignments(FAULT_INJECTION_VARIABLE_PREFIX));
    format!(
        "{} cargo test --release -p singlefs-verification --test fault_injection_campaign -- --exact fault_injection_returns_errors_instead_of_panicking",
        render_environment_assignments(&assignments)
    )
}

/// 故障注入那条入口读环境：读的变量与 [`fault_injection_replay_line`] 写出的逐个相同，没设的取 `defaults` 的；线程数不在重放行里，照 `defaults` 的。
/// 种子数与每段注入次数是计数：写 0 与写错一样认不得（0 段、或每段一次都不注入的一批照样判成绿）。
fn fault_injection_campaign_from_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    defaults: &FaultInjectionCampaign,
) -> Result<FaultInjectionCampaign, ReproductionEnvironmentError> {
    let first_seed = number_from_reproduction_environment(
        lookup,
        &FaultInjectionVariable::FirstSeed.variable_name(),
        defaults.first_seed,
    )?;
    let seed_count = count_from_reproduction_environment(
        lookup,
        &FaultInjectionVariable::Seeds.variable_name(),
        defaults.seed_count,
    )?;
    let faults_variable = FaultInjectionVariable::Faults.variable_name();
    let faults_given = count_from_reproduction_environment(
        lookup,
        &faults_variable,
        u64::try_from(defaults.faults_per_history).expect("注入次数装得进 u64"),
    )?;
    let faults_per_history = usize::try_from(faults_given).map_err(|_too_many| {
        ReproductionEnvironmentError::Unrecognized {
            variable: faults_variable.clone(),
            value: faults_given.to_string(),
            recognized: vec!["装得进 usize 的注入次数".to_string()],
        }
    })?;
    let sampling = HistorySampling::from_environment(
        FAULT_INJECTION_VARIABLE_PREFIX,
        lookup,
        &HistorySampling {
            operations_per_history: defaults.operations_per_history,
            weights: defaults.weights,
            execution: defaults.execution,
        },
    )?;
    Ok(FaultInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history: sampling.operations_per_history,
        faults_per_history,
        weights: sampling.weights,
        execution: sampling.execution,
        worker_threads: defaults.worker_threads,
    })
}

/// 默认批：这个测试周期的种子基起 24 段、每段 20 步、每段摆 4 个注入点，broad 比重，每一步的镜像交出去。
fn default_batch_campaign(worker_threads: FaultInjectionWorkerThreads) -> FaultInjectionCampaign {
    FaultInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: DEFAULT_BATCH_SEEDS,
        operations_per_history: DEFAULT_BATCH_OPERATIONS_PER_HISTORY,
        faults_per_history: DEFAULT_BATCH_FAULTS_PER_HISTORY,
        weights: GenerationWeights::BROAD,
        execution: HANDING_EVERY_STEP_IMAGE_OUT,
        worker_threads,
    }
}

/// 故障注入那条入口读环境：变量与判红时打的重放行同一张表（[`fault_injection_campaign_from_environment`]），没设的取默认批的
/// （[`default_batch_campaign`]），线程数从 `SINGLEFS_FAULT_INJECTION_WORKER_THREADS` 取。入口、「不设变量取默认批」那一格与「重放行喂回入口」那两条自证都经这一个函数。
///
/// # Panics
/// 某个变量的取值认不得（[`fault_injection_campaign_from_environment`] 报的那几种），消息带变量名与原值。
fn fault_injection_campaign_from(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> FaultInjectionCampaign {
    fault_injection_campaign_from_environment(
        lookup,
        &default_batch_campaign(FaultInjectionWorkerThreads::from_the_environment()),
    )
    .unwrap_or_else(|error| panic!("{}", error.render()))
}
