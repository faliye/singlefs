//! 里程碑「覆盖写、释放、回退与复用」增补 3 第 4 件：故障注入。通用的设备包装、注入点怎么摆、注入之后怎么判在
//! `singlefs_harness::fault_injection`；这里是快档（普通 `cargo test`）、多线程对拍、写死的那几条用例与大档（`#[ignore]`，规模从环境变量取）。
//!
//! 被测的两条性质：注入之后 `singlefs-core` **返回错误而不是 panic**；出错之后**重开恢复到模型允许的版本**。
//! 种子基是这个测试周期写死的那一个（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，随机历史与崩溃注入那两个二进制用的是同一个）。
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/fault_injection_fast_tier.rs"]
mod scenarios_of_fault_injection_fast_tier;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_fault_injection_fast_tier::{
    ordinal_of_the_root_slot_write_within_the_step, overwrite_with_fill_seed,
    the_fixed_history_with_its_root_slot_write_swallowed,
    the_history_whose_make_filesystem_fails_at_its_first_barrier, FAST_TIER_OPERATIONS_PER_HISTORY,
    HANDING_EVERY_STEP_IMAGE_OUT,
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
use singlefs_harness::fault_injection::{
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault,
    invariants_still_violated_on_the_comparison_image,
    unit_start_slots_quarantined_over_a_swallowed_write, FailureAfterAFault,
    PoolCheckerVerdictsOfTheImagesAfterAFault, SwallowedWriteByteRange, ViolationsNamingTheSites,
};
use singlefs_harness::fault_injection::{
    inject_faults_into_history,
    inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier,
    inject_one_fault_into_the_segment, run_fault_injection_campaign, AcquiredInstanceLeft,
    AcquiredInstanceLeftAfterAFailedMount, DrawnFault, FaultCounting, FaultDeviceSelector,
    FaultInjectionCampaign, FaultInjectionReport, FaultInjectionTally, FaultInjectionWorkerThreads,
    FaultOccurrence, FaultOutcome, FaultPlacement, FaultSchedule, FaultedSegment,
    FaultedSegmentKind, FlippedBit, ImagesAfterAFault, InjectedFault,
    PoolCheckerVerdictsAfterAFault, SharedFaultPlan,
};
use singlefs_harness::history::CopyQuarantinedByTheImplementation;
use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::history::{
    count_from_reproduction_environment, environment_assignments_in_line, execute_history_with,
    execute_history_with_faults, generate_history_with_weights,
    number_from_reproduction_environment, render_environment_assignments, AppliedEffect,
    CompactionChoice, CompactionRunEnding, FailureSignature, FloorTargetChoice, GeneratedHistory,
    GenerationWeights, HarnessJudgement, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryOperationKind, HistoryRun, HistorySampling, HistorySeed,
    HistoryStartingPoint, PerStepChecker, ReproductionEnvironmentError, ReproductionVariablePrefix,
    StartingPointStep, StepOutcome, StepPosition,
};
use singlefs_harness::memory_pool::{root_identity_written_by, MemoryPool};
use singlefs_harness::model::{ModelDisagreementAspect, ModelReportedUnreadablePlace};
use singlefs_harness::numeric_environment_variable::process_environment_variable;
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedOperationKind, SharedStream};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;

/// 快档的规模：段数、每段步数、每段注入几次。每次注入都要把这段历史整个重跑一遍，所以规模按「别显著变慢」定。
const FAST_TIER_SEEDS: u64 = 24;

const FAST_TIER_FAULTS_PER_HISTORY: usize = 4;

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
/// （`singlefs_harness::history::HistorySamplingVariable`，格式只有那一份定义；代码三方 test-script-align-code-r1 判决 C607）。
const FAULT_INJECTION_VARIABLE_PREFIX: ReproductionVariablePrefix =
    ReproductionVariablePrefix("SINGLEFS_FAULT_INJECTION_");

singlefs_harness::enum_listing_every_member! {
/// 重放行与大档入口里故障注入自己的几个变量（接在 [`FAULT_INJECTION_VARIABLE_PREFIX`] 后面；取样参数接在它们后面）。
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

/// 重放这一整批的那一行命令：种子区间、每段注入几次、取样参数，交大档那条入口照抄跑的就是原来那一批。
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
        "{} cargo test --release -p singlefs-harness --test fault_injection_fast_tier -- --include-ignored --exact fault_injection_large_tier_from_the_environment",
        render_environment_assignments(&assignments)
    )
}

/// 大档那条入口读环境：读的变量与 [`fault_injection_replay_line`] 写出的逐个相同，没设的取 `defaults` 的；线程数不在重放行里，照 `defaults` 的。
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

fn fast_tier_campaign(worker_threads: FaultInjectionWorkerThreads) -> FaultInjectionCampaign {
    FaultInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: FAST_TIER_SEEDS,
        operations_per_history: FAST_TIER_OPERATIONS_PER_HISTORY,
        faults_per_history: FAST_TIER_FAULTS_PER_HISTORY,
        weights: GenerationWeights::BROAD,
        execution: HANDING_EVERY_STEP_IMAGE_OUT,
        worker_threads,
    }
}

/// 快档：这个测试周期的种子基起 24 段、每段 20 步、每段摆 4 个注入点。每个注入点重跑一遍这段历史，
/// 判「返回错误而不是 panic」与「重开恢复到模型允许的版本」；「已知红」清单里的形态照记不停，清单外的一条都不许有。
///
/// 读出坏字节（`read_returns_corrupted_bytes`）只注入在一次读上：释放之前读盘核那一读撞上它时重读一次就对得上，不隔离
/// （D19（块指针的结构与宽度预算） 已定项 5：核出对不上也先重读一次）。只重读的是读不出那一半时，这一格在种子 7463871032432355113
/// 第 7 步（覆盖写）把一对好槽隔离掉、池级 checker 判 I-3.11 红。**每一读都给坏字节**（坏读一直在、盘上的字节是好的）时两次都对不上，
/// 照规则隔离那一对好槽——那是「两次都对不上才隔离」认下的形态，不是新发现；随机注入不摆这一形（一次注入只坏一次调用），
/// 它的样子钉在 `release_checksum_quarantine.rs` 的用例 13。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn fault_injection_fast_tier_returns_errors_instead_of_panicking() {
    let started = std::time::Instant::now();
    let report = run_fault_injection_campaign(&fast_tier_campaign(
        FaultInjectionWorkerThreads::from_the_environment(),
    ));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 故障注入快档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert_eq!(
        report.tally.faults_that_panicked,
        0,
        "注入之后 core panic 了：这是被测的性质（返回错误而不是 panic）；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.new_findings.is_empty(),
        "注入之后「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert_every_fault_injection_path_was_exercised(&report);
}

/// 各条路径真的跑到了：六种注入都摆过、起点段与每一类会写盘的操作上都注入过、重开都跑过、四类写落点都注入过。
fn assert_every_fault_injection_path_was_exercised(report: &FaultInjectionReport) {
    let tally = &report.tally;
    let rendered = report.render();
    assert!(
        tally.faults >= FAST_TIER_SEEDS,
        "摆出来的注入点太少（{}）：每段至少该摆出一个\n{rendered}",
        tally.faults
    );
    // 六种全抽：前四种是里程碑逐字写的（写、读、刷盘返回块设备错，加上读回改坏的字节），
    // 后两种是设备说谎（判决第二节改法一把它们抽进样本；`draw_injected_fault` 的 `below(6)` 钉在这一条上）。
    for fault in [
        InjectedFault::WriteFails,
        InjectedFault::ReadFails,
        InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FlippedBit {
                byte_index: 0,
                bit_index: 0,
            },
        },
        InjectedFault::BarrierFails,
        InjectedFault::WriteIsSwallowed,
        InjectedFault::BarrierIsSwallowed,
    ] {
        assert!(
            tally.faults_by_kind.contains_key(fault.name()),
            "一次都没注入过 {}：这一种注入在快档里没跑到\n{rendered}",
            fault.name()
        );
    }
    // 起点段（mkfs、取号、暖机、第一个文件）上注入过：判决第二节改法二的验收那一格，改之前结构性地恒为 0
    // （`draw_faults` 的候选从第 1 步起算）。
    assert!(
        tally.faults_on_segment(FaultedSegmentKind::TheStartingPoint) > 0,
        "起点段上一次都没注入过：`draw_faults` 的候选区间又把第 0 段排掉了\n{rendered}"
    );
    // 说谎的设备真的丢掉过内容（盘面不一致只有池级 checker 看得见）：这一格归 checker 档两段，同一批种子在
    // `crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`
    // 里核说谎的设备留下的不一致至少一次（`mutation-sampling.md`「取样点不敏感」）。
    assert!(
        tally.reopens_reading_a_file > 0 && tally.reopens_without_a_file > 0,
        "重开的两种结局没都见过：读回文件 {}、没有文件 {}\n{rendered}",
        tally.reopens_reading_a_file,
        tally.reopens_without_a_file
    );
    assert!(
        tally
            .faults_by_outcome
            .contains_key(FaultOutcome::SurfacedAsAnError.name()),
        "一次都没有「注入那一步返回了错误」：这一件要的就是这一格\n{rendered}"
    );
    // 会写盘的四类操作：发布（覆盖写）、可写挂载、挂着时回退（一次向前发布）、抬 F。冷启动只读不写，不要求写落点上注入过。
    for operation in [
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::CloseAndMountWritable,
        HistoryOperationKind::RollBackWhileMounted,
        HistoryOperationKind::RaiseRollbackFloor,
    ] {
        assert!(
            tally.faults_on_segment(FaultedSegmentKind::Operation(operation)) > 0,
            "{operation:?} 上一次都没注入过（验收要「每个发布步骤与挂载步骤上各注入过至少一次」）\n{rendered}"
        );
    }
    assert!(
        tally.reopens >= tally.faults,
        "每次注入之后都要重开一次：重开 {} 次、注入 {} 次\n{rendered}",
        tally.reopens,
        tally.faults
    );
    assert_eq!(
        tally.reopens_outside_everything_the_model_allows,
        0,
        "重开走到了模型都不允许的版本：{}\n{rendered}",
        how_to_replay(report)
    );
    assert_eq!(
        tally.images_after_the_fault_handed_out, tally.faults,
        "每次注入之后重开的盘都要交出去一次（池级 checker 那一遍归 checker 档两段）\n{rendered}"
    );
    assert_eq!(
        tally.checker_runs_on_the_image_after_the_fault, 0,
        "harness 档不在注入之后的盘上跑池级 checker\n{rendered}"
    );
}

/// 大档没设环境变量时每段几步、注入几次（大档撞出的单段回归用例照同一组规模重放那一段）。
const LARGE_TIER_OPERATIONS_PER_HISTORY: u64 = 30;

const LARGE_TIER_FAULTS_PER_HISTORY: u64 = 6;

/// 大档注入之后这段历史怎么跑：harness 不跑池级 checker（那一遍归 checker 档两段）、空间准入按公式判，盘宽由调用方给（大档从环境变量取）。
fn large_tier_execution(device_width: HistoryDeviceWidth) -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

/// 大档那条入口读环境：变量与判红时打的重放行同一张表（[`fault_injection_campaign_from_environment`]），没设的取大档的默认
/// （种子基起 512 段、每段 30 步、每段注入 6 次、各类操作都抽、每一步之后跑 checker、两块 4 GiB 的盘、判空间准入），
/// 线程数从 `SINGLEFS_FAULT_INJECTION_WORKER_THREADS` 取。入口与「重放行喂回入口」那两条自证都经这一个函数。
fn large_tier_campaign_from(lookup: &dyn Fn(&str) -> Option<String>) -> FaultInjectionCampaign {
    fault_injection_campaign_from_environment(
        lookup,
        &FaultInjectionCampaign {
            first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            seed_count: 512,
            operations_per_history: usize::try_from(LARGE_TIER_OPERATIONS_PER_HISTORY)
                .expect("步数"),
            faults_per_history: usize::try_from(LARGE_TIER_FAULTS_PER_HISTORY).expect("注入次数"),
            weights: GenerationWeights::BROAD,
            execution: large_tier_execution(HistoryDeviceWidth::FourGibibytes),
            worker_threads: FaultInjectionWorkerThreads::from_the_environment(),
        },
    )
    .unwrap_or_else(|error| panic!("{}", error.render()))
}

/// 大档入口里种子数写 0：停下，消息带变量名与原值，不照收（一段都不跑的一批照「全部返回错误、不 panic」判成绿）。
#[test]
#[should_panic(expected = "SINGLEFS_FAULT_INJECTION_SEEDS=0：只认 正整数")]
fn a_fault_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
    let _campaign = large_tier_campaign_from(&|variable: &str| {
        (variable == "SINGLEFS_FAULT_INJECTION_SEEDS").then(|| "0".to_string())
    });
}

/// 大档入口里每段注入次数写 0：停下，消息带变量名与原值，不照收（一次都不注入的一批照样判成绿）。
#[test]
#[should_panic(expected = "SINGLEFS_FAULT_INJECTION_FAULTS=0：只认 正整数")]
fn a_fault_injection_fault_count_of_zero_panics_naming_the_variable_and_the_value() {
    let _campaign = large_tier_campaign_from(&|variable: &str| {
        (variable == "SINGLEFS_FAULT_INJECTION_FAULTS").then(|| "0".to_string())
    });
}

/// 大档：规模与取样参数照判红时打的重放行从环境变量取，后台跑（`cargo test --release -p singlefs-harness --test
/// fault_injection_fast_tier -- --ignored --nocapture`）。
#[test]
#[ignore = "大档：照判红时打的重放行设环境变量；SINGLEFS_FAULT_INJECTION_FIRST_SEED 起 SINGLEFS_FAULT_INJECTION_SEEDS 段、每段 SINGLEFS_FAULT_INJECTION_OPERATIONS 步、每段注入 SINGLEFS_FAULT_INJECTION_FAULTS 次，比重 SINGLEFS_FAULT_INJECTION_WEIGHTS（broad、reuse、rollback、wall、unit-area-wall、broad-compaction 等，取值同随机历史的比重名），checker SINGLEFS_FAULT_INJECTION_CHECKER（只认 skip），盘宽 SINGLEFS_FAULT_INJECTION_DEVICES（4gib、unit-area-384、unit-area-256、unit-area-240），空间准入 SINGLEFS_FAULT_INJECTION_SPACE_ADMISSION（judged、skipped），线程数 SINGLEFS_FAULT_INJECTION_WORKER_THREADS；没设的取大档默认（种子基、512 段、30 步、6 次、broad、skip、4gib、judged）；按里程碑「增补 3」后台跑"]
fn fault_injection_large_tier_from_the_environment() {
    let started = std::time::Instant::now();
    let report =
        run_fault_injection_campaign(&large_tier_campaign_from(&process_environment_variable));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 故障注入大档 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert_eq!(
        report.tally.faults_that_panicked,
        0,
        "注入之后 core panic 了；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.new_findings.is_empty(),
        "注入之后「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
}

/// 判红时对一批故障注入会打的那一行重放命令（`how_to_replay`；造一份报告，不必真红）原样喂回大档那条入口
/// （`large_tier_campaign_from`，入口用的同一个函数）：种子区间、每段步数、每段注入几次、比重、跑法与原来那一批逐项相同，
/// 第一段生成的历史逐步相同。不跑注入：同一组参数下摆出来的注入点由种子定，这里只核参数一项不漏地回来了
/// （代码三方 test-script-align-code-r1 判决 C607：原先的重放行只带种子区间、步数与注入次数，比重与跑法照抄回不来）。
fn the_replay_line_printed_for_a_batch_runs_the_same_campaign_on_the_large_tier_entry(
    campaign: &FaultInjectionCampaign,
) {
    let report = FaultInjectionReport {
        first_seed: campaign.first_seed,
        seed_count: campaign.seed_count,
        operations_per_history: campaign.operations_per_history,
        faults_per_history: campaign.faults_per_history,
        weights: campaign.weights,
        execution: campaign.execution,
        tally: FaultInjectionTally::default(),
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
    };
    let replay = how_to_replay(&report);
    let assignments = environment_assignments_in_line(&replay);
    let replayed = large_tier_campaign_from(&|variable: &str| assignments.get(variable).cloned());
    assert_eq!(
        (
            replayed.first_seed,
            replayed.seed_count,
            replayed.operations_per_history,
            replayed.faults_per_history,
            replayed.weights,
            replayed.execution,
        ),
        (
            campaign.first_seed,
            campaign.seed_count,
            campaign.operations_per_history,
            campaign.faults_per_history,
            campaign.weights,
            campaign.execution,
        ),
        "照重放行跑大档，跑的不是原来那一批（种子、种子数、步数、注入次数、比重、跑法）：{replay}"
    );
    assert_eq!(
        generate_history_with_weights(
            HistorySeed(replayed.first_seed),
            replayed.operations_per_history,
            &replayed.weights
        ),
        generate_history_with_weights(
            HistorySeed(campaign.first_seed),
            campaign.operations_per_history,
            &campaign.weights
        ),
        "第一段生成的历史逐步相同"
    );
}

#[test]
fn replay_line_of_the_fast_tier_runs_the_same_campaign_on_the_large_tier_entry() {
    the_replay_line_printed_for_a_batch_runs_the_same_campaign_on_the_large_tier_entry(
        &fast_tier_campaign(FaultInjectionWorkerThreads::GivenByCaller(1)),
    );
}

/// 每一项都与大档的默认不同的一批：重放行漏写哪一项，大档入口就退回它的默认，读回的不是这一批。
/// 前提断言先核「每一项都不同」，不然漏写那一项读回的恰好还是它。
#[test]
fn replay_line_of_a_batch_unlike_the_large_tier_defaults_runs_the_same_campaign_on_the_large_tier_entry(
) {
    let campaign = FaultInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7,
        seed_count: 3,
        operations_per_history: 11,
        faults_per_history: 2,
        weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        // 每一步的池级 checker 只剩不跑那一种（那一遍归 checker 档两段），那一项取不出与默认不同的值，前提断言不核它
        execution: HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        worker_threads: FaultInjectionWorkerThreads::GivenByCaller(1),
    };
    let defaults = large_tier_campaign_from(&|_variable: &str| None);
    assert!(
        campaign.first_seed != defaults.first_seed
            && campaign.seed_count != defaults.seed_count
            && campaign.operations_per_history != defaults.operations_per_history
            && campaign.faults_per_history != defaults.faults_per_history
            && campaign.weights != defaults.weights
            && campaign.execution.device_width != defaults.execution.device_width
            && campaign.execution.space_admission != defaults.execution.space_admission,
        "前提：这一批除每一步的池级 checker 之外的每一项都与大档的默认不同：{campaign:?} 对 {defaults:?}"
    );
    the_replay_line_printed_for_a_batch_runs_the_same_campaign_on_the_large_tier_entry(&campaign);
}

/// 故障注入的段数上限：从这个测试周期的种子基起，在这么多段里找一段整理做成过一批的历史。
const SEEDS_SEARCHED_FOR_A_COMPACTION_BATCH: u64 = 64;

/// 瞬时读错撞上整理（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J5）：大档入口按比重变量点名「快档加整理」那一组
/// （`SINGLEFS_FAULT_INJECTION_WEIGHTS=broad-compaction`，`GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED`），从种子基起找头一段
/// 整理做成过一批的历史（那一步读过单元头），在那一步发出的头一次读上摆一次读错（只坏这一次调用，下一次照常读得出），重跑这段历史：
/// 注入落在整理那一步上，返回错误而不是 panic。注入之后重开落不落在模型允许的版本、有没有「已知红」之外的新发现，归大档入口点名这一组比重
/// 跑（`fault_injection_large_tier_from_the_environment`）；这里把计数与新发现打出来，不判。
#[test]
fn a_transient_read_error_lands_on_a_compaction_step_of_the_weights_the_large_tier_names_broad_compaction(
) {
    let campaign = large_tier_campaign_from(&|variable: &str| {
        (variable == "SINGLEFS_FAULT_INJECTION_WEIGHTS").then(|| "broad-compaction".to_string())
    });
    let (history, compaction_step) = (0..SEEDS_SEARCHED_FOR_A_COMPACTION_BATCH)
        .find_map(|offset| {
            let history = generate_history_with_weights(
                HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + offset),
                FAST_TIER_OPERATIONS_PER_HISTORY,
                &campaign.weights,
            );
            let run = execute_history_with(
                &history,
                HANDING_EVERY_STEP_IMAGE_OUT,
                &SharedStream::new(),
                &mut |_| {},
            );
            let compaction_step = run.outcomes.iter().position(|outcome| {
                matches!(
                    outcome,
                    StepOutcome::Applied(AppliedEffect::Compacted {
                        batches_published,
                        ..
                    }) if *batches_published > 0
                )
            })?;
            Some((history, compaction_step))
        })
        .unwrap_or_else(|| {
            panic!(
                "比重 {} 在种子基起 {SEEDS_SEARCHED_FOR_A_COMPACTION_BATCH} 段里没有一段整理做成过一批：这一组比重不抽整理",
                campaign.weights.name
            )
        });
    let compaction_segment = FaultedSegment::Operation {
        step_index: compaction_step,
        operation_kind: HistoryOperationKind::CompactWhileMounted,
    };
    let injection = inject_one_fault_into_the_segment(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        InjectedFault::ReadFails,
        compaction_segment,
        1,
    );
    let rendered = injection.tally.render();
    print_uncaptured(&format!(
        "── 瞬时读错撞上整理（种子 {}、第 {compaction_step} 步）──\n{rendered}新发现 {} 条\n{}",
        history.seed.0,
        injection.new_findings.len(),
        injection
            .new_findings
            .iter()
            .map(|finding| finding.render())
            .collect::<String>()
    ));
    assert_eq!(
        injection
            .drawn_faults
            .iter()
            .map(|drawn| (drawn.fault, drawn.segment))
            .collect::<Vec<_>>(),
        vec![(InjectedFault::ReadFails, compaction_segment)],
        "注入点就是整理那一步的头一次读\n{rendered}"
    );
    assert_eq!(
        injection
            .tally
            .faults_on_segment(FaultedSegmentKind::Operation(
                HistoryOperationKind::CompactWhileMounted
            )),
        1,
        "这一次注入记在整理那一步上\n{rendered}"
    );
    assert_eq!(
        injection.tally.faults_that_panicked, 0,
        "注入之后 core panic 了：被测的性质是返回错误而不是 panic\n{rendered}"
    );
}

/// 说谎的设备留下的盘面不一致里只判红 `["I-3.1"]` 的那一形的取样点（按注入点认，`fault_injection::left_by_the_lying_device`；
/// `crates/mutations.tsv` 钉着把被吞的写插回去那一步拿掉的变异）。快档那 24 段里说谎的设备留下的不一致全落在另一组
/// （`["I-2.1", "I-4.8", "I-7.4"]`），这一组要单独钉。
///
/// 取样点写死成一段历史与一个注入点，不按种子从比重表里抽：此前钉的是种子 7463871032432355210 在 `GenerationWeights::BROAD` 上抽出的
/// 第 25 步覆盖写那次根槽写，删除批把比重表里可写挂载那一格切出 unlink 之后同一个种子抽出的历史变了、那次被吞的写不在了，用例跟着红
/// （调度记录 `records/2026-10-01-删除与并发事务.md` 第 11 批的慢档红调查问 1；比重表一改，种子钉的取样点就会再丢一次）。
/// 这一段：第一个文件（txg 3）之后覆盖写五次（txg 4–8），txg 7 那次（第 3 步）的根槽 FUA 写被吞；harness 不跑池级 checker，
/// 带着注入跑的那一遍每一步的镜像、重开的盘与被吞的写插回去之后的盘经钩子交出去（快档与大档那一遍归 checker 档两段），
/// checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）在交出去的盘上
/// 经给定状态引擎判池级 checker、交归类（核的是归类那一截）：txg 8 那次覆盖写之后 checker 只判红 I-3.1（被吞的根槽留下的，
/// 被吞的写插回去就消失）。它要按注入点认成说谎的设备留下的，不算新发现——带着注入跑的那一遍里第一张判红的每步镜像、重开的那张盘，各一条。
/// 这里只核注入的结局与交出去的盘。
#[test]
fn swallowed_write_after_which_the_checker_flags_only_the_allocated_statistic_invariant_is_excused_as_what_the_lying_device_may_leave(
) {
    let (injection, handed_out) = the_fixed_history_with_its_root_slot_write_swallowed();
    assert!(
        injection.new_findings.is_empty(),
        "harness 那一截（模型、RefFS、执行器）不许有新发现：{:?}",
        injection.new_findings
    );
    let [images] = handed_out.as_slice() else {
        panic!("一个注入点交出去一份盘：交了 {} 份", handed_out.len());
    };
    assert!(
        images.image_with_the_swallowed_writes_landed.is_some(),
        "被吞了写，要交被吞的写插回去之后的那张盘"
    );
}

/// 写死那一段（[`the_fixed_history_with_its_root_slot_write_swallowed`]）的那一个注入点交出去的盘，与被吞那次写之后的头一张每步镜像在里面的下标。
fn the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write(
) -> (ImagesAfterAFault, usize) {
    let (_injection, handed_out) = the_fixed_history_with_its_root_slot_write_swallowed();
    let [images] = handed_out.as_slice() else {
        panic!("一个注入点交出去一份盘：交了 {} 份", handed_out.len());
    };
    assert_eq!(images.swallowed_writes.len(), 1, "这一段只吞了一次写");
    let first_step_image_after_the_swallowed_write = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .position(|step_image| step_image.swallowed_writes_before_it > 0)
        .expect("被吞那次写之后还有每步镜像");
    (images.clone(), first_step_image_after_the_swallowed_write)
}

/// 合成的一份违例：`invariant` 判红，点名 `slots` 里的每一个（盘, 槽）。
fn violations_naming(invariant: &'static str, slots: &[(u32, u64)]) -> ViolationsNamingTheSites {
    ViolationsNamingTheSites {
        violations: vec![(invariant, "合成的违例".to_string())],
        violation_sites: slots
            .iter()
            .map(|(device, slot)| ViolationSite {
                invariant,
                device: *device,
                slot: *slot,
            })
            .collect(),
    }
}

/// 写死那一段交出去的盘上，第 `observed_step_image` 张每步镜像判红 `observed`、它的按步对照盘判红 `at_the_step`、
/// 被吞的写都插回去那张判红 `with_every_swallowed_write_landed`、重开那张不红：交归类器，交回归好的那几条。
fn classified_with_given_verdicts(
    images: &ImagesAfterAFault,
    observed_step_image: usize,
    observed: ViolationsNamingTheSites,
    at_the_step: ViolationsNamingTheSites,
    with_every_swallowed_write_landed: ViolationsNamingTheSites,
) -> Vec<FailureAfterAFault> {
    let mut violations_of_the_step_images =
        vec![Vec::new(); images.step_images_of_the_run_with_the_fault.len()];
    violations_of_the_step_images[observed_step_image] = observed.violations;
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
        images,
        PoolCheckerVerdictsOfTheImagesAfterAFault {
            violations_of_the_step_images,
            violation_sites_of_the_first_red_step_image: observed.violation_sites,
            after_the_fault: ViolationsNamingTheSites::default(),
            at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed: Some(
                at_the_step,
            ),
            with_the_swallowed_writes_landed: Some(with_every_swallowed_write_landed),
        },
    )
}

/// 按步对照盘（判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3）：被吞那次写之前的每步镜像没有按步对照盘（插回去就是它本身）；
/// 之后的每一张，按步对照盘是那一刻的录制流前缀插回那一次被吞的写——与被吞的写都插回去那张盘的头这么多步逐步相同。
#[test]
fn the_step_comparison_image_lands_only_the_writes_swallowed_before_that_step_image() {
    let (images, first_step_image_after_the_swallowed_write) =
        the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write();
    let landed = images
        .image_with_the_swallowed_writes_landed
        .as_ref()
        .expect("被吞了写，要交被吞的写插回去之后的那张盘");
    for (index, step_image) in images
        .step_images_of_the_run_with_the_fault
        .iter()
        .enumerate()
    {
        let at_the_step =
            images.image_at_the_step_image_with_the_swallowed_writes_before_it_landed(index);
        if index < first_step_image_after_the_swallowed_write {
            assert_eq!(
                at_the_step, None,
                "第 {index} 张每步镜像那一刻之前没吞写，没有按步对照盘"
            );
            continue;
        }
        let at_the_step = at_the_step
            .unwrap_or_else(|| panic!("第 {index} 张每步镜像那一刻之前吞过一次写，要有按步对照盘"));
        assert_eq!(
            at_the_step.operations.as_slice(),
            &landed.operations[..step_image.operations_recorded + 1],
            "第 {index} 张：那一刻的前缀插回被吞的那一次写，与全部插回去那张的头几步逐步相同"
        );
        assert_eq!(
            at_the_step.operations_written_by_make_filesystem,
            landed.operations_written_by_make_filesystem,
            "第 {index} 张：被吞的写落在 mkfs 之后，mkfs 的步数不跟着挪"
        );
    }
}

/// 按步对照，不对照末尾那张（判决第三节 C3：第二轮 C3 原句「对照插回去的最终那张盘」作废）：被吞那次写之后的头一张每步镜像点名
/// （盘 0 与盘 1, 槽 777）判红 I-3.1，它的按步对照盘上同一处照红，被吞的写都插回去那张盘上一条都不红（那一处只在中间那一步的镜像上红，
/// 两腿 `leak_once` 量到的那一类）：仍红、不是被吞的写留下的，报新发现。对照末尾那张盘就把它归成说谎的设备。
#[test]
fn a_violation_still_named_at_its_slot_on_the_step_comparison_image_is_a_new_finding_though_the_last_comparison_image_no_longer_shows_it(
) {
    let (images, step_image) =
        the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write();
    let failures = classified_with_given_verdicts(
        &images,
        step_image,
        violations_naming("I-3.1", &[(0, 777), (1, 777)]),
        violations_naming("I-3.1", &[(0, 777), (1, 777)]),
        ViolationsNamingTheSites::default(),
    );
    assert!(
        matches!(
            failures.as_slice(),
            [FailureAfterAFault::NewFinding { signature, .. }]
                if *signature == FailureSignature::CheckerViolations { invariants: vec!["I-3.1"] }
        ),
        "按步对照盘上同一处照红：报新发现一条，判成 {failures:?}"
    );
}

/// 按（不变量, 盘, 槽）比，不按名比（判决第三节 C3）：被比那一张点名（盘 0, 槽 777）判红 I-3.1，按步对照盘上 I-3.1 也红、点名的却是
/// 另一处（盘 0, 槽 888）：被比那一张点名的那一处插回去就不红了，这一条是被吞的写留下的，归说谎的设备。按名比就照报新发现。
#[test]
fn a_violation_named_at_another_slot_on_the_step_comparison_image_is_left_by_the_lying_device() {
    let (images, step_image) =
        the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write();
    let failures = classified_with_given_verdicts(
        &images,
        step_image,
        violations_naming("I-3.1", &[(0, 777)]),
        violations_naming("I-3.1", &[(0, 888)]),
        violations_naming("I-3.1", &[(0, 888)]),
    );
    assert!(
        matches!(
            failures.as_slice(),
            [FailureAfterAFault::LeftByTheLyingDevice { .. }]
        ),
        "点名的那一处插回去就不红：归说谎的设备一条，判成 {failures:?}"
    );
}

/// 实现交回的隔离（判决第三节 C3「实现的隔离清单收齐四条路」）：被比那一张点名（两块盘, 槽 s）判红 I-3.1，按步对照盘上同一处照红，
/// 而槽 s 正是实现在那一刻之前交回的一份隔离、那一份压在被吞的写上——插回去之后两份都对得上、记录却留在已分配，是实现在说谎的设备上
/// 读盘核出对不上才隔离的：减掉之后不再仍红，归说谎的设备。不减（隔离清单不进比较）就照报新发现。槽 s 取被吞那次写落的槽，
/// 隔离那一份合成（这一段被吞的是根槽写，实现没有隔离）。
#[test]
fn a_violation_named_only_at_a_unit_the_implementation_quarantined_over_a_swallowed_write_is_left_by_the_lying_device(
) {
    let (mut images, step_image) =
        the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write();
    let swallowed = images.swallowed_writes[0].clone();
    let slot_of_the_swallowed_write = swallowed.offset.0 / singlefs_format::SLOT_BYTES;
    images
        .copies_quarantined_by_the_implementation
        .push(CopyQuarantinedByTheImplementation {
            step: images.step_images_of_the_run_with_the_fault[step_image].position,
            copy: CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: TransactionUnit::InodeRoot,
                device: swallowed.device,
                placement: Placement {
                    slot: SlotNumber(slot_of_the_swallowed_write),
                    span: 1,
                },
                reading: QuarantinedCopyReading::ChecksumMismatch,
            },
        });
    images.step_images_of_the_run_with_the_fault[step_image]
        .copies_quarantined_by_the_implementation_before_it = 1;
    assert_eq!(
        unit_start_slots_quarantined_over_a_swallowed_write(
            &images.copies_quarantined_by_the_implementation,
            &images.swallowed_writes,
        ),
        BTreeSet::from([slot_of_the_swallowed_write]),
        "合成的那一份隔离压在被吞的写上"
    );
    let sites = [
        (0, slot_of_the_swallowed_write),
        (1, slot_of_the_swallowed_write),
    ];
    let failures = classified_with_given_verdicts(
        &images,
        step_image,
        violations_naming("I-3.1", &sites),
        violations_naming("I-3.1", &sites),
        ViolationsNamingTheSites::default(),
    );
    assert!(
        matches!(
            failures.as_slice(),
            [FailureAfterAFault::LeftByTheLyingDevice { .. }]
        ),
        "插回去之后仍红的只有压在被吞的写上的那一份隔离：归说谎的设备一条，判成 {failures:?}"
    );
}

/// 隔离判据那三条之外的不变量不减隔离（I-3.10 判的是分配代与诞生代号，隔离不豁免它）；点了槽的一边没有槽可比时按名比。
#[test]
fn only_the_quarantine_criterion_invariants_drop_the_quarantined_slots_and_a_side_without_slots_compares_by_name(
) {
    let quarantined = BTreeSet::from([777]);
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-3.10", &[(1, 777)]),
            Some(&violations_naming("I-3.10", &[(1, 777)])),
            &quarantined,
        ),
        BTreeSet::from(["I-3.10"]),
        "I-3.10 不归隔离判据：同一处照红就仍红"
    );
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-3.15", &[(1, 777)]),
            Some(&violations_naming("I-3.15", &[(1, 777)])),
            &quarantined,
        ),
        BTreeSet::new(),
        "I-3.15 随 I-3.1 那一份隔离判据：压在被吞的写上的隔离减掉"
    );
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-2.1", &[]),
            Some(&violations_naming("I-2.1", &[])),
            &quarantined,
        ),
        BTreeSet::from(["I-2.1"]),
        "没点槽的按名比：对照盘上也红就仍红"
    );
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-3.1", &[(0, 5)]),
            Some(&violations_naming("I-3.1", &[])),
            &quarantined,
        ),
        BTreeSet::from(["I-3.1"]),
        "对照盘上这一条红而一个槽都没点：按名比，仍红"
    );
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-3.1", &[(0, 5)]),
            None,
            &quarantined,
        ),
        BTreeSet::from(["I-3.1"]),
        "没有对照盘（那一刻之前没吞写）：每一条都仍红"
    );
}

/// 镜像上最新根那棵分配记录树里起点是某一槽的一条记录读回来的样子（盘上的隔离记号，D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」）。
#[derive(Debug, PartialEq, Eq)]
enum AllocatedRecordReadBack {
    Released,
    CarryingTheQuarantineMark,
    WithoutTheQuarantineMark,
}

/// 镜像上最新根那棵分配记录树里起点是 `slot` 的那几条记录（按盘号从小到大）各读成什么样。
fn records_of_the_newest_root_starting_at(
    image: &MemoryPool,
    slot: u64,
) -> Vec<AllocatedRecordReadBack> {
    let system_configuration = choose_system_configuration(image).expect("择系统配置");
    let newest = choose_root(image, &system_configuration).expect("最新根");
    let mut records: Vec<AllocationRecord> = allocation_records_under_root(image, &newest)
        .expect("最新根那棵分配记录树读得回来（读者认隔离记号）")
        .into_iter()
        .filter(|record| record.slot.0 == slot)
        .collect();
    records.sort_by_key(|record| record.device.0);
    records
        .iter()
        .map(|record| {
            if record.is_released {
                return AllocatedRecordReadBack::Released;
            }
            match record.quarantine_mark {
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck => {
                    AllocatedRecordReadBack::CarryingTheQuarantineMark
                }
                AllocationRecordQuarantineMark::NotQuarantined => {
                    AllocatedRecordReadBack::WithoutTheQuarantineMark
                }
            }
        })
        .collect()
}

/// 必红世界那一段（场景 [`a_leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces`]）的实现那一半：
/// 实现交回两次隔离、各两份——X 那一次（泄漏的那个单元，不压在被吞的写上）与 Y 那一次（被吞的那一份所在的单元，压在被吞的写上）；
/// X 之后那一张每步镜像那一刻之前吞过写，交出去的盘里它的按步对照盘、被吞的写都插回去那张都在。盘上的记号（D19（块指针的结构与宽度预算）
/// 已定项 5「隔离的盘上记号」）：X 之后那一张每步镜像上泄漏的那个单元两块盘那两条记录仍分配、带记号；重开那张上它们仍分配、两块盘都不带记号
/// （泄漏），Y 隔离的那个单元两块盘那两条仍分配、都带记号（照条款豁免）。池级 checker 那一遍与归类在 checker 档同名用例：重开那张报新发现。
#[test]
fn a_leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces() {
    let images =
        scenarios_of_fault_injection_fast_tier::a_leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces();
    let quarantined = &images.copies_quarantined_by_the_implementation;
    assert_eq!(quarantined.len(), 4, "两次隔离各两份：{quarantined:?}");
    let leaked_slot = quarantined[0].copy.placement.slot.0;
    let slot_quarantined_over_the_swallowed_write = quarantined[2].copy.placement.slot.0;
    let pool_images = scenarios_of_fault_injection_fast_tier::pool_images_after_a_fault(&images);
    let step_image_after_the_swallowed_write = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .position(|step_image| step_image.swallowed_writes_before_it > 0)
        .expect("X 之后那一张每步镜像那一刻之前吞过写");
    assert_eq!(
        records_of_the_newest_root_starting_at(
            &pool_images.step_images[step_image_after_the_swallowed_write],
            leaked_slot
        ),
        vec![
            AllocatedRecordReadBack::CarryingTheQuarantineMark,
            AllocatedRecordReadBack::CarryingTheQuarantineMark
        ],
        "X 之后那一张：泄漏的那个单元两块盘那两条记录仍分配、带着 X 写下的记号（头一张判红的每步镜像上没有泄漏）"
    );
    assert_eq!(
        records_of_the_newest_root_starting_at(&pool_images.after_the_fault, leaked_slot),
        vec![
            AllocatedRecordReadBack::WithoutTheQuarantineMark,
            AllocatedRecordReadBack::WithoutTheQuarantineMark
        ],
        "重开那张：泄漏的那个单元两块盘那两条记录仍分配、都不带记号"
    );
    assert_eq!(
        records_of_the_newest_root_starting_at(
            &pool_images.after_the_fault,
            slot_quarantined_over_the_swallowed_write
        ),
        vec![
            AllocatedRecordReadBack::CarryingTheQuarantineMark,
            AllocatedRecordReadBack::CarryingTheQuarantineMark
        ],
        "重开那张：Y 隔离的那个单元两块盘那两条记录仍分配、都带记号"
    );
    assert_ne!(
        leaked_slot, slot_quarantined_over_the_swallowed_write,
        "泄漏的那个单元与被吞的那一份所在的单元不是同一个"
    );
    assert_eq!(
        unit_start_slots_quarantined_over_a_swallowed_write(quarantined, &images.swallowed_writes),
        BTreeSet::from([slot_quarantined_over_the_swallowed_write]),
        "压在被吞的写上的只有 Y 那一次隔离的那个单元"
    );
    let step_image_of_the_swallowed_write = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .position(|step_image| step_image.swallowed_writes_before_it > 0)
        .expect("X 之后那一张每步镜像那一刻之前吞过写");
    assert!(
        images
            .image_at_the_step_image_with_the_swallowed_writes_before_it_landed(
                step_image_of_the_swallowed_write
            )
            .is_some()
            && images.image_with_the_swallowed_writes_landed.is_some(),
        "按步对照盘与被吞的写都插回去那张盘都交出去"
    );
}

/// 必绿世界 K2-1 那一点（场景 [`the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot`]）的实现那一半：
/// 头一个注入点吞掉盘 1 上一次落在复用过的槽上的 16 KiB 写，实现之后释放那一版时读盘核出盘 1 那一份对不上、隔离两块盘那一份，
/// 清单经交出去的盘带出来（跨重挂累积：之后的挂载不清掉它）。池级 checker 那一遍与归类在 checker 档同名用例：归说谎的设备，不报新发现。
#[test]
fn the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot_hands_out_the_quarantine_of_both_copies(
) {
    let images = scenarios_of_fault_injection_fast_tier::the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot();
    assert_eq!(
        images.drawn.fault,
        InjectedFault::WriteIsSwallowed,
        "头一个注入点是一次被吞的写：{}",
        images.drawn.render()
    );
    let [swallowed] = images.swallowed_writes.as_slice() else {
        panic!("吞了一次写：{:?}", images.swallowed_writes.len());
    };
    let swallowed_range = SwallowedWriteByteRange::of(swallowed);
    let swallowed_slot = swallowed.offset.0 / singlefs_format::SLOT_BYTES;
    assert_eq!(
        (
            swallowed.device.0,
            swallowed_range.end_in_bytes - swallowed_range.start_in_bytes
        ),
        (1, singlefs_format::SLOT_BYTES),
        "被吞的是盘 1 上一槽那么长的一次写"
    );
    assert_eq!(
        unit_start_slots_quarantined_over_a_swallowed_write(
            &images.copies_quarantined_by_the_implementation,
            &images.swallowed_writes,
        ),
        BTreeSet::from([swallowed_slot]),
        "实现隔离了被吞的写落的那一槽：{:?}",
        images.copies_quarantined_by_the_implementation
    );
    let devices_quarantined_at_the_slot: BTreeSet<u32> = images
        .copies_quarantined_by_the_implementation
        .iter()
        .filter(|quarantined| quarantined.copy.placement.slot.0 == swallowed_slot)
        .map(|quarantined| quarantined.copy.device.0)
        .collect();
    assert_eq!(
        devices_quarantined_at_the_slot,
        BTreeSet::from([0, 1]),
        "两块盘那一份一起隔离（账保持对称）"
    );
}

/// 大档（这个测试周期的种子基起 512 段）撞出的一段，单独跑：种子 7463871032432355397，规模与执行方式与大档没设环境变量时逐项相同。
/// 抽出的 6 个注入点里有两次被吞掉的写（`write_is_swallowed`）：整池第 481 次写调用（第 17 步覆盖写那次发布的根槽写）与第 448 次
/// （第 16 步挂着时回退那次向前发布的根槽写）。两个注入点上历史都走到第 18 步挂着时回退：判候选集读根环，这个进程知道住着根的那一槽
/// （被吞那次写的槽）读坏、重读仍坏，拒在任何写之前（`RollbackError::CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`，
/// D16（发布语义） 已定项 1「根槽这一次读坏」那一行的读法）。模型不知道根槽没落、说该成；回退点名的槽正是被吞那次写的槽，各记进说谎那一格一次，
/// 不算新发现。回退路径不把点名的槽交给模型比对时（`crates/mutations.tsv` 钉着的那一格），这一格判不出是说谎的设备留下的，照报新发现。
/// 原先 harness 每一步就地判池级 checker、判红就停：第 448 次那个注入点上第 17 步之后的镜像判红 I-3.1、I-3.9（被吞的根留下的），
/// 历史停在那里、走不到第 18 步，这一格只记一次；池级 checker 那一遍搬进 checker 档两段之后 harness 不停，两个注入点各记一次
/// （照判红就停那一档的口径并进 checker 档结论的是合并入口 `inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rollback_refused_on_the_root_ring_slot_whose_write_the_lying_device_swallowed_is_excused_as_what_the_lying_device_may_leave(
) {
    let report = run_fault_injection_campaign(&FaultInjectionCampaign {
        first_seed: 7_463_871_032_432_355_397,
        seed_count: 1,
        operations_per_history: usize::try_from(LARGE_TIER_OPERATIONS_PER_HISTORY).expect("步数"),
        faults_per_history: usize::try_from(LARGE_TIER_FAULTS_PER_HISTORY).expect("注入次数"),
        weights: GenerationWeights::BROAD,
        execution: large_tier_execution(HistoryDeviceWidth::FourGibibytes),
        worker_threads: FaultInjectionWorkerThreads::from_the_environment(),
    });
    let rendered = report.render();
    assert_eq!(
        report.tally.faults_that_panicked,
        0,
        "注入之后 core panic 了；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.new_findings.is_empty(),
        "回退拒在被吞根槽写的那个槽上要记进说谎那一格，不算新发现；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert_eq!(
        report
            .tally
            .faults_by_injection_point
            .get("PublishOverwrite/root_record_fua"),
        Some(&1),
        "这一段抽出的注入点里要有那一次被吞的覆盖写根槽写：\n{rendered}"
    );
    assert_eq!(
        report
            .tally
            .lying_device_signatures
            .get("ModelDisagreement { aspect: \"模型说该成、实现拒了\" }"),
        Some(&2),
        "回退那两次「模型说该成、实现拒了」要按点名的槽等于被吞根槽写的槽放过、记进说谎那一格，两个注入点各一次：\n{rendered}"
    );
}

/// 一段写死的历史上把注入点摆满：同一段历史注入 12 次，逐次判「返回错误而不是 panic」。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn one_fixed_history_injects_twelve_faults_and_none_of_them_panics() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        24,
        &GenerationWeights::BROAD,
    );
    let injection = inject_faults_into_history(&history, HANDING_EVERY_STEP_IMAGE_OUT, 12);
    let rendered = injection.tally.render();
    print_uncaptured(&format!("── 故障注入：一段写死的历史 ──\n{rendered}"));
    assert!(
        injection.drawn_faults.len() >= 8,
        "12 次抽里摆出来的注入点太少（{}）",
        injection.drawn_faults.len()
    );
    assert_eq!(
        injection.tally.faults_that_panicked, 0,
        "注入之后 core panic 了\n{rendered}"
    );
    assert!(
        injection.new_findings.is_empty(),
        "「已知红」清单外的失败\n{rendered}"
    );
}

/// 摆注入点的候选从第 0 段起算，起点段摆得到：`draw_faults` 的 `(0..marks.len())` 改回 `(1..marks.len())`，
/// 起点段的注入点恒为 0，这里必红（判决第二节改法二钉的那一格；`crates/mutations.tsv` 同名的那一条）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_drawn_injection_points_reach_the_starting_segment() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        8,
        &GenerationWeights::BROAD,
    );
    let injection = inject_faults_into_history(&history, HANDING_EVERY_STEP_IMAGE_OUT, 12);
    let rendered_faults: Vec<String> = injection
        .drawn_faults
        .iter()
        .map(DrawnFault::render)
        .collect();
    assert!(
        injection
            .drawn_faults
            .iter()
            .any(|drawn| drawn.segment == FaultedSegment::TheStartingPoint),
        "12 次抽里一个起点段的注入点都没摆出来：{rendered_faults:?}"
    );
    assert_eq!(
        injection.tally.faults_that_panicked,
        0,
        "起点段上注入打出了 panic\n{}",
        injection.tally.render()
    );
}

/// 只有起点段的一段历史：操作 0 步，跑起来只有 mkfs、取号、暖机、第一个文件。
/// 比重挑一律从第一个文件起的那一组，起点段四步才都走得到。
fn history_with_only_the_starting_point() -> GeneratedHistory {
    generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        0,
        &GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
    )
}

/// 在整池第 `ordinal` 次写上注一次写报错，跑那一段历史。`ordinal` 为 `None` 时不注入。
fn run_the_starting_point(
    history: &GeneratedHistory,
    failing_write_ordinal: Option<u64>,
) -> (HistoryRun, SharedFaultPlan) {
    let geometry = HANDING_EVERY_STEP_IMAGE_OUT.device_width.fixed_geometry();
    let plan = match failing_write_ordinal {
        None => SharedFaultPlan::unarmed(geometry),
        Some(ordinal) => SharedFaultPlan::armed(
            geometry,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, ordinal),
        ),
    };
    let run = execute_history_with_faults(
        history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        &SharedStream::new(),
        &plan,
        &mut |_| {},
    );
    (run, plan)
}

/// 起点段失败时那条观察：没 panic，而且停在「起点段的入口返回了错误」上，交回是哪一步。
fn the_starting_point_step_that_returned_an_error(
    run: &HistoryRun,
    ordinal: u64,
) -> Option<StartingPointStep> {
    let observation = match &run.ending {
        HistoryEnding::Completed => return None,
        HistoryEnding::KnownRed { observation, .. }
        | HistoryEnding::NewFinding { observation, .. } => observation,
    };
    assert!(
        observation.panic.is_none(),
        "起点段第 {ordinal} 次写上注入打出了 panic：{:?}",
        observation.panic
    );
    match &observation.harness_judgement {
        Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) => Some(*step),
        Some(_) | None => {
            panic!("起点段第 {ordinal} 次写上注入之后停在别的东西上：{observation:?}")
        }
    }
}

/// 起点段（mkfs、取号、暖机、第一个文件）的四个入口逐个注入：一次 panic 都没有，四步各至少有一次把错交回来。
///
/// 判决第二节改法二钉的就是这一格：改之前这四步在执行器里是 `expect`，注入在起点段只会撞出执行器自己的 panic
/// （判定五：攻方在只改候选区间的探针上实测 49 次，全在 `history.rs` mkfs 那一处）。这条用例不抽样——
/// 起点段每一次写都注入一遍，四步谁少了一处 `expect` 没改，这里必红。
#[test]
fn every_entry_in_the_starting_segment_returns_an_error_instead_of_panicking() {
    let history = history_with_only_the_starting_point();
    // 一、不注入跑一遍，数起点段发了多少次读 / 写 / 刷盘（验收要的那个数，这里现量）。
    let (measured, measurement) = run_the_starting_point(&history, None);
    assert!(
        matches!(measured.ending, HistoryEnding::Completed),
        "不注入时起点段要跑完：{:?}",
        measured.ending
    );
    let calls = measurement.calls();
    print_uncaptured(&format!(
        "── 起点段逐次注入 ──\n起点段的设备调用：读 {}、写 {}、刷盘 {}，合计 {}\n",
        calls.reads,
        calls.writes,
        calls.barriers,
        calls.reads + calls.writes + calls.barriers
    ));
    // 二、起点段的每一次写各注入一遍：记下是哪一步把错交回来的，panic 一次都不许有。
    let mut failures_by_step: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut tolerated = 0u64;
    for ordinal in 1..=calls.writes {
        let (run, plan) = run_the_starting_point(&history, Some(ordinal));
        assert_eq!(plan.fired_count(), 1, "第 {ordinal} 次写没注上");
        match the_starting_point_step_that_returned_an_error(&run, ordinal) {
            None => tolerated += 1,
            Some(step) => *failures_by_step.entry(step.name()).or_insert(0) += 1,
        }
    }
    print_uncaptured(&format!(
        "把错交回来的：{failures_by_step:?}；报了成功、照样跑完的 {tolerated} 次\n"
    ));
    for step in [
        StartingPointStep::MakeFilesystem,
        StartingPointStep::AcquireInstance,
        StartingPointStep::WarmUp,
        StartingPointStep::PublishFirstFile,
    ] {
        assert!(
            failures_by_step.contains_key(step.name()),
            "起点段的{}一次都没把错交回来（这一处的 `expect` 还在？）：{failures_by_step:?}",
            step.name()
        );
    }
}

/// 起点段失败时池级 checker 要判的那张镜像交不交出去（harness 不跑 checker，那一遍归 checker 档两段，镜像经每步镜像那个钩子交出去）：
/// **mkfs 自己就报错那一次不交**（盘上没有一个做完了的文件系统，checker 判的每一条都没有对象），
/// **取号 / 暖机 / 第一个文件报错那几次照交**（mkfs 已经做完，盘上有池）。
///
/// 用户 2026-09-21 定的口径，分界与「重开走读失败算不算失败」同一条：mkfs 做没做完
/// （`history.rs` 的 `StartingPointStep::finished_filesystem_is_on_the_devices`）。
/// 起点段 44 次写逐次注入，两边都数出来——口径被改掉（两边都跑、或者两边都不跑），这里必红。
#[test]
fn the_pool_checker_runs_on_a_starting_point_failure_only_when_make_filesystem_finished() {
    let history = history_with_only_the_starting_point();
    // 不注入时起点段跑完，起点之后交一张镜像：下面那两格拿它当对照。
    let (measured, measurement) = run_the_starting_point(&history, None);
    assert!(
        matches!(measured.ending, HistoryEnding::Completed),
        "不注入时起点段要跑完：{:?}",
        measured.ending
    );
    assert_eq!(
        measured.tally.step_images_handed_out, 1,
        "起点段跑完之后该交一张镜像"
    );
    assert_eq!(measured.tally.checker_runs, 0, "harness 档不跑池级 checker");
    let mut step_images_handed_out_by_step: BTreeMap<&'static str, BTreeSet<u64>> = BTreeMap::new();
    for ordinal in 1..=measurement.calls().writes {
        let (run, _plan) = run_the_starting_point(&history, Some(ordinal));
        let Some(step) = the_starting_point_step_that_returned_an_error(&run, ordinal) else {
            continue;
        };
        step_images_handed_out_by_step
            .entry(step.name())
            .or_default()
            .insert(run.tally.step_images_handed_out);
        let expected_step_images = u64::from(step.finished_filesystem_is_on_the_devices());
        assert_eq!(
            run.tally.step_images_handed_out,
            expected_step_images,
            "起点段第 {ordinal} 次写上注入、停在{}：mkfs 做完了才该交那张镜像",
            step.name()
        );
    }
    print_uncaptured(&format!(
        "── 起点段失败时交出去的镜像 ──\n每一步报错时交了几张：{step_images_handed_out_by_step:?}\n"
    ));
    // 两边都要有样本，不然上面那条断言是空过的。
    assert_eq!(
        step_images_handed_out_by_step.get(StartingPointStep::MakeFilesystem.name()),
        Some(&BTreeSet::from([0])),
        "mkfs 报错那几次要有样本，而且一张镜像都没交：{step_images_handed_out_by_step:?}"
    );
    for step in [
        StartingPointStep::AcquireInstance,
        StartingPointStep::WarmUp,
        StartingPointStep::PublishFirstFile,
    ] {
        assert_eq!(
            step_images_handed_out_by_step.get(step.name()),
            Some(&BTreeSet::from([1])),
            "{}报错那几次要有样本，而且各交过一张：{step_images_handed_out_by_step:?}",
            step.name()
        );
    }
}

/// 用户 2026-10-03 定：mkfs 在写到系统配置之前失败的盘面就是全池皆废，I-8.5（tail 槽有效集非空）照判红，红是对的；
/// 故障注入的「已知红」清单加这一类。快档在种子 7463871032432355120 上摆的那一个注入点（整池第 1 次 barrier 报错，落在 mkfs 第一道屏障上，
/// 那一刻 mkfs 只清过零、写过实例表与树表的创世单元，系统配置槽一次都没写）原样重跑：注入之后的镜像上只红 I-8.5、
/// 重开走读失败，判法记成那一条「已知红」，不记新发现。
///
/// 判别力自证：判法改回不认这一类（`known_red_form_only_after_a_fault` 恒交回 None，`crates/mutations.tsv` 同名那一条），
/// 这一次落成新发现 `CheckerViolations { invariants: ["I-8.5"] }`，checker 档同名用例红（池级 checker 那一遍与归类在那里，
/// `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）；这里只核注入的结局与交出去的盘。
#[test]
fn make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding(
) {
    let (injection, handed_out) = the_history_whose_make_filesystem_fails_at_its_first_barrier();
    let rendered = injection.tally.render();
    print_uncaptured(&format!(
        "── mkfs 第一道屏障报错：全池皆废、只红 I-8.5 ──\n{rendered}"
    ));
    let expected_drawn = DrawnFault {
        fault: InjectedFault::BarrierFails,
        call_ordinal: 1,
        segment: FaultedSegment::TheStartingPoint,
    };
    assert_eq!(
        injection.drawn_faults,
        vec![expected_drawn],
        "注入点是整池第 1 次 barrier、落在起点段\n{rendered}"
    );
    // 停下的是 mkfs 自己，重开走读失败而盘上本来就没有一版立得住：这一形的前两样事实真的在。
    assert_eq!(
        injection.tally.starting_point_failures,
        BTreeMap::from([(StartingPointStep::MakeFilesystem.name(), 1)]),
        "起点段里把错交回来的是 mkfs\n{rendered}"
    );
    assert_eq!(
        injection
            .tally
            .reopens_that_correctly_found_nothing_to_recover,
        1,
        "重开走读失败、盘上本来就没有一版立得住\n{rendered}"
    );
    assert!(
        injection.new_findings.is_empty() && injection.known_red_hits.is_empty(),
        "harness 那一截（模型、RefFS、执行器）既没有新发现、也没有已知红（I-8.5 那一判归池级 checker）：{:?}\n{rendered}",
        injection
            .new_findings
            .iter()
            .map(|finding| &finding.signature)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        injection.tally.checker_runs_on_the_image_after_the_fault, 0,
        "harness 不在注入之后的盘上跑池级 checker（那一遍归 checker 档两段）\n{rendered}"
    );
    let [images] = handed_out.as_slice() else {
        panic!(
            "一个注入点交出去一份盘：交了 {} 份\n{rendered}",
            handed_out.len()
        );
    };
    assert!(
        images.step_images_of_the_run_with_the_fault.is_empty(),
        "mkfs 自己报错，一张每步镜像都不交\n{rendered}"
    );
    assert_eq!(injection.tally.faults_that_panicked, 0, "{rendered}");
}

/// 线程数换了，报告逐字相同：计数按片的次序相加、新发现按种子从小到大留第一个
/// （`implementation-workflow.md`「测试与崩溃检测优先多线程」的「合并要确定」）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_report_is_the_same_text_with_one_worker_thread_and_with_four() {
    let campaign = |threads| FaultInjectionCampaign {
        seed_count: 6,
        operations_per_history: 12,
        faults_per_history: 2,
        ..fast_tier_campaign(FaultInjectionWorkerThreads::GivenByCaller(threads))
    };
    let one = run_fault_injection_campaign(&campaign(1));
    let four = run_fault_injection_campaign(&campaign(4));
    assert_eq!(
        one.render(),
        four.render(),
        "线程数变了报告就变了：并到一起的次序不确定"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_fault_injection_fast_tier::publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root(
) {
    scenarios_of_fault_injection_fast_tier::publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 起点（mkfs、取号 1、暖机、第一个文件）之后只做一步：关掉会话、可写挂载（取号 2 → 写行 → 暖机）。
fn history_that_mounts_once_after_the_first_file() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::CloseAndMountWritable],
    }
}

/// 那一步可写挂载里的第几次写：取号是头四次（取号写两轮，每轮两块盘各一次系统配置槽写），写行那次发布的第一个单元写是第 5 次；
/// 写行那次发布一共 23 次写（实例表两盘 2 次、分配记录树按位置寻址这一次重写的节点两盘 10 次、记账 / 映射 / 树表各两盘 6 次、
/// 记录两盘 2 次、根槽 1 次、系统配置槽轮换 2 次），暖机第一次空发布的第一个单元写是第 4 + 23 + 1 = 28 次
/// （这一步一共 48 次写：暖机一次 21 次）。分配记录树按位置寻址（D8（核心索引结构） 已定项 14）之前那棵树只有一个节点，
/// 写行那次 15 次写、这一格摆在第 18 次；照旧摆在第 18 次就落进写行那次、记成「新号一条根都没有」。
const FIRST_ROW_PUBLISH_WRITE_OF_THE_MOUNT: u64 = 5;

const FIRST_WARM_UP_WRITE_OF_THE_MOUNT: u64 = 28;

/// C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）按用户 2026-09-23 定的那一边（认了，写成已知行为；
/// D23（journal 的角色与格式） 已定项 16 的射程）断言：取号之后写行或暖机那一次写报块设备错，挂载返回错误，
/// 重开的盘上系统配置的实例代号**还是新号 2**（没回卷成 1），这一次记进故障注入的失败账
/// （`FaultInjectionTally::acquired_instances_left_after_a_failed_mount`）、报告里点名。两格各一条：
/// 写行那次的第一个单元写报错 ⇒ 新号一条根都没有（烧掉）；暖机第一次的第一个单元写报错 ⇒ 新号已有写行那次的根。
///
/// 判别力自证：把挂载改成「发布报错之后把系统配置的实例代号写回旧号」（回卷，`crates/mutations.tsv` 里那一条），
/// 重开的盘上系统配置是 1，这一格记不上，这里红。
#[test]
fn write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it(
) {
    let history = history_that_mounts_once_after_the_first_file();
    let mount_step = FaultedSegment::Operation {
        step_index: 0,
        operation_kind: HistoryOperationKind::CloseAndMountWritable,
    };
    for (call_within_the_mount, expected_kind, expected_highest_root_instance) in [
        (
            FIRST_ROW_PUBLISH_WRITE_OF_THE_MOUNT,
            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot,
            InstanceGeneration(1),
        ),
        (
            FIRST_WARM_UP_WRITE_OF_THE_MOUNT,
            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot,
            InstanceGeneration(2),
        ),
    ] {
        let injection = inject_one_fault_into_the_segment(
            &history,
            HANDING_EVERY_STEP_IMAGE_OUT,
            InjectedFault::WriteFails,
            mount_step,
            call_within_the_mount,
        );
        let rendered = injection.tally.render();
        print_uncaptured(&format!(
            "── C378：可写挂载第 {call_within_the_mount} 次写报错 ──\n{rendered}"
        ));
        assert_eq!(
            injection.acquired_instances_left,
            vec![AcquiredInstanceLeft {
                drawn: injection.drawn_faults[0],
                kind: expected_kind,
                system_configuration_instance_before_the_step: InstanceGeneration(1),
                system_configuration_instance_after_the_reopen: InstanceGeneration(2),
                highest_root_instance_after_the_reopen: Some(expected_highest_root_instance),
            }],
            "取号之后第 {call_within_the_mount} 次写报错：重开的盘上系统配置还是新号 2、不回卷成 1\n{rendered}"
        );
        assert_eq!(
            injection.tally.acquired_instances_left_after_a_failed_mount,
            BTreeMap::from([(expected_kind.name(), 1)]),
            "失败账记下这一次\n{rendered}"
        );
        assert!(
            rendered.contains("取号之后挂载报错、新号不回卷（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）")
                && rendered.contains(&format!("  {}：1 次", expected_kind.name())),
            "报告里点名这一格\n{rendered}"
        );
        // 注入真打在那一步可写挂载的单元写上，那一步返回了错误：不是别处先停了。
        assert_eq!(
            injection.tally.faults_by_injection_point,
            BTreeMap::from([("CloseAndMountWritable/unit_write".to_string(), 1)]),
            "注入点\n{rendered}"
        );
        assert_eq!(
            injection
                .tally
                .faults_by_outcome
                .get(FaultOutcome::SurfacedAsAnError.name())
                .copied(),
            Some(1),
            "可写挂载返回了错误\n{rendered}"
        );
        // 记下的错误成员恰好一次、而且是发布那一步的：空表上「每一个都是」恒真，所以先钉条数。
        assert_eq!(
            injection.tally.refusal_members.values().sum::<u64>(),
            1,
            "注入那一步返回的错误成员记了一次\n{rendered}"
        );
        assert!(
            injection
                .tally
                .refusal_members
                .keys()
                .all(|member| member.starts_with("MountError::Publish(")),
            "返回的是发布那一步的错误成员\n{rendered}"
        );
        assert_eq!(injection.tally.faults_that_panicked, 0, "{rendered}");
        assert!(
            injection.new_findings.is_empty(),
            "认下的行为不算新发现，别的也不许有\n{rendered}"
        );
    }
}

/// 写死的整理历史里强制选的那一段在候选段（0 号盘上装着未释放记录、不是挂载开着的聚簇段，按段起点升序）里的位置，同
/// `compaction_while_mounted_in_random_histories.rs` 的同名常量：起点之后可写挂载一次，候选是第一个文件的数据单元那一段与
/// 第一个文件那次发布的提交内生块那一段，取后一段。
const REGION_SELECTOR_OF_THE_SEGMENT_OF_THE_FIRST_PUBLISH: u64 = 1;

/// 起点（mkfs、取号 1、暖机、第一个文件）之后：可写挂载（取号 2）；挂着时整理第一个文件那次发布的那一段，一批一个、推一批就崩，
/// 再可写挂载（取号 3 → 写行 → 暖机）。整理的签发与那一批在号 2 下发。
fn history_whose_compaction_crashes_after_one_batch_then_mounts_writable() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 1,
                region_selector: REGION_SELECTOR_OF_THE_SEGMENT_OF_THE_FIRST_PUBLISH,
                ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            }),
        ],
    }
}

/// 不注入跑一遍这段历史，核第 `step_index` 步是一趟发了批、崩掉之后又可写挂载做成的整理，数这一步发出的写里头一次写下实例代号
/// `acquired_instance` 的根的那次根槽写是这一步的第几次写（1 起）：推批的几次发布写的根带的是这次挂载之前的号，带新号的头一条根是
/// 批间崩溃之后那次挂载写行那次发布的根，取号那两次系统配置槽写在它之前。`inject_one_fault_into_the_segment` 按「这一段发出的第几次写」摆注入点。
fn ordinal_of_the_row_publish_root_slot_write_of_the_mount_after_the_compaction_crash(
    history: &GeneratedHistory,
    step_index: usize,
    acquired_instance: InstanceGeneration,
) -> u64 {
    let stream = SharedStream::retaining_contents();
    let mut stream_length_after: BTreeMap<StepPosition, usize> = BTreeMap::new();
    let run = execute_history_with(
        history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        &stream,
        &mut |observation| {
            stream_length_after.insert(observation.position, stream.operation_count());
        },
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "不注入时这段历史跑完：{:?}",
        run.outcomes
    );
    assert!(
        matches!(
            run.outcomes[step_index],
            StepOutcome::Applied(AppliedEffect::Compacted {
                batches_published: 1..,
                crashed_then_mounted: true,
                ..
            })
        ),
        "前提：第 {step_index} 步整理发了批、崩掉之后可写挂载做成：{:?}",
        run.outcomes[step_index]
    );
    let step_start = match step_index.checked_sub(1) {
        None => stream_length_after[&StepPosition::StartingPoint],
        Some(previous) => stream_length_after[&StepPosition::Operation(previous)],
    };
    let step_end = stream_length_after[&StepPosition::Operation(step_index)];
    let geometry = HANDING_EVERY_STEP_IMAGE_OUT.device_width.fixed_geometry();
    let writes_of_the_step: Vec<_> = stream.retained_operations()[step_start..step_end]
        .iter()
        .filter(|retained| retained.operation.kind != RecordedOperationKind::Barrier)
        .cloned()
        .collect();
    let index = writes_of_the_step
        .iter()
        .position(|retained| {
            geometry.classify(&retained.operation) == StepKind::RootRecordFua
                && root_identity_written_by(
                    retained
                        .contents
                        .as_deref()
                        .expect("开了内容保留的流，根槽写带着字节"),
                )
                .0 == acquired_instance
        })
        .unwrap_or_else(|| panic!("第 {step_index} 步没写过带实例代号 {acquired_instance:?} 的根"));
    u64::try_from(index + 1).expect("一步至多几百次写")
}

/// C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）落在挂着时整理那一步里（判决 `research/prompts/m4-segment-code-r3-main-verification.md`
/// 第三节 J5 的后续：故障注入的比重 `broad-compaction` 抽得到整理）：整理推一批、崩掉之后可写挂载一次（取号 3），写行那次发布的根槽写报块设备错，
/// 挂载返回错误；重开的盘上系统配置的实例代号还是新号 3（这一步之前是 2，整理那几次发布不换号），根环里没有带号 3 的根。
/// 这一次认成取号之后挂载报错、新号没回卷那一格（`acquired_instances_left`、失败账），不算新发现。
///
/// 判别力自证：故障注入把整理那一步当成不取号的操作（`system_configuration_instance_before_a_mount_step` 那一臂交 `None`，
/// `crates/mutations.tsv` 里那一条），这一格记不上，这里红。
#[test]
fn write_error_after_the_acquisition_of_the_mount_after_a_compaction_crash_counts_as_the_acquired_instance_left_and_not_a_new_finding(
) {
    const COMPACTION_STEP: usize = 1;
    let instance_before_the_compaction_step = InstanceGeneration(2);
    let instance_acquired_after_the_crash = InstanceGeneration(3);
    let history = history_whose_compaction_crashes_after_one_batch_then_mounts_writable();
    let call_within_the_step =
        ordinal_of_the_row_publish_root_slot_write_of_the_mount_after_the_compaction_crash(
            &history,
            COMPACTION_STEP,
            instance_acquired_after_the_crash,
        );
    let injection = inject_one_fault_into_the_segment(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        InjectedFault::WriteFails,
        FaultedSegment::Operation {
            step_index: COMPACTION_STEP,
            operation_kind: HistoryOperationKind::CompactWhileMounted,
        },
        call_within_the_step,
    );
    let rendered = injection.tally.render();
    print_uncaptured(&format!(
        "── C378 落在整理那一步：批间崩溃之后的可写挂载、这一步第 {call_within_the_step} 次写报错 ──\n{rendered}"
    ));
    // 注入真打在整理那一步的根槽写上，那一步返回了挂载发布的错误：不是别处先停了。
    assert_eq!(
        injection.tally.faults_by_injection_point,
        BTreeMap::from([("CompactWhileMounted/root_record_fua".to_string(), 1)]),
        "注入点\n{rendered}"
    );
    assert_eq!(
        injection
            .tally
            .faults_by_outcome
            .get(FaultOutcome::SurfacedAsAnError.name())
            .copied(),
        Some(1),
        "整理那一步返回了错误\n{rendered}"
    );
    assert!(
        injection.tally.refusal_members.values().sum::<u64>() == 1
            && injection
                .tally
                .refusal_members
                .keys()
                .all(|member| member.starts_with("MountError::Publish(")),
        "返回的是挂载里发布那一步的错误成员，恰好一次\n{rendered}"
    );
    assert_eq!(
        injection.acquired_instances_left,
        vec![AcquiredInstanceLeft {
            drawn: injection.drawn_faults[0],
            kind: AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot,
            system_configuration_instance_before_the_step: instance_before_the_compaction_step,
            system_configuration_instance_after_the_reopen: instance_acquired_after_the_crash,
            highest_root_instance_after_the_reopen: Some(instance_before_the_compaction_step),
        }],
        "取号之后写行那次的根槽写报错：重开的盘上系统配置还是新号 3、根环里最大的号是 2，记进这一格\n{rendered}"
    );
    assert_eq!(
        injection.tally.acquired_instances_left_after_a_failed_mount,
        BTreeMap::from([(
            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot.name(),
            1
        )]),
        "失败账记下这一次\n{rendered}"
    );
    assert_eq!(injection.tally.faults_that_panicked, 0, "{rendered}");
    assert!(
        injection.new_findings.is_empty(),
        "认下的行为不算新发现，别的也不许有：{:?}\n{rendered}",
        injection.new_findings
    );
}

/// 按注入点认说谎的设备留下的那两格（主 agent 2026-09-26 定，实七；实四丙交回 ③ offset 339 与 ④ offset 284 那两形）：
/// 第一个文件（txg 3）之后覆盖写五次（4–8），txg 7 那次的根槽 FUA 写被吞；再可写挂载（实例 2，择到 txg 8、不施加记录，
/// 与模型对得上），接着抬 F 到 1。这个进程挂载时 txg 7 那个根槽就读不出（从没写对过），实现照旧当没有根
/// （445 的定案：挂载时就读不出、这个进程之后也没写过的槽照旧当没有根），报的上限按看不见 (1, 7) 算，模型按确认过的根算、多一个非空状态——
/// 历史停在这一步；按模型去掉 (1, 7) 之后重算的上限与实现报的相等，认成说谎的设备留下的。抬 F 那一步照实现的上限做成、写出的那几条
/// 空发布根不在模型认过的版本里，重开走到最新那一条、读回的是 txg 8 那一版的内容：「放行集并进历史停下那一步实现已写出的根」认下它。
/// 两格任一格不认，这一次注入就成了新发现。
/// （同一个进程里吞了根槽再抬 F 走的是另一格，见
/// `floor_raise_in_the_same_process_after_a_swallowed_root_slot_write_is_refused_for_that_slot_and_recognised_as_left_by_the_lying_device`。）
#[test]
fn a_swallowed_root_slot_write_is_recognised_by_the_ceiling_recomputed_without_its_root_and_the_floor_raise_it_left_behind_is_allowed_on_reopen(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite_with_fill_seed(1),
            overwrite_with_fill_seed(2),
            overwrite_with_fill_seed(3),
            overwrite_with_fill_seed(4),
            overwrite_with_fill_seed(5),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 1,
            }),
        ],
    };
    // 每一步之后不跑池级 checker：txg 8 那次覆盖写之后 checker 就判红 I-3.1（被吞的根槽留下的，被吞的写插回去就消失），
    // 跑着的话历史停在那一步、走不到抬 F；重开之后那一次 checker 照跑、照判。
    let execution_without_the_per_step_checker = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let injection = inject_one_fault_into_the_segment(
        &history,
        execution_without_the_per_step_checker,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::Operation {
            step_index: 3,
            operation_kind: HistoryOperationKind::PublishOverwrite,
        },
        ordinal_of_the_root_slot_write_within_the_step(&history, 3),
    );
    let tally = &injection.tally;
    assert!(
        injection.new_findings.is_empty(),
        "按注入点认得出的两格不许成新发现：{:?}",
        injection.new_findings
    );
    assert_eq!(
        tally
            .lying_device_signatures
            .get("ModelDisagreement { aspect: \"抬 F 的上限\" }"),
        Some(&1),
        "抬 F 的上限那一格按去掉被吞的根之后重算的上限认下：{:?}",
        tally.lying_device_signatures
    );
    assert_eq!(
        tally.reopened_into_the_version_the_faulted_step_was_writing, 1,
        "重开走到停下那一步写出的根上，放行集认下它"
    );
}

/// 同一个进程里根槽写被吞之后再抬 F（实七报告 (b)，实八）：第一个文件（txg 3）之后覆盖写五次（4–8），txg 7 那次的根槽 FUA 写被吞，
/// 不重新挂载，接着抬 F 到 1。这个进程写过 txg 7 那个槽、FUA 返回过，算上限时读它读坏、重读仍坏，实现照 D16（发布语义） 已定项 1
/// 「根槽这一次读坏」那一行拒这次抬 F、报 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，点名的就是 txg 7 那个槽。
/// 模型不知道根槽没落，要它成或按上限拒，对拍报对不上（前半截钉住这一形）；按注入点认：读坏的槽正是被吞那次写的，认成说谎的设备留下的，
/// 不成新发现（后半截）。复现的随机种子是故障注入大档种子基 + 116。
#[test]
fn floor_raise_in_the_same_process_after_a_swallowed_root_slot_write_is_refused_for_that_slot_and_recognised_as_left_by_the_lying_device(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite_with_fill_seed(1),
            overwrite_with_fill_seed(2),
            overwrite_with_fill_seed(3),
            overwrite_with_fill_seed(4),
            overwrite_with_fill_seed(5),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 1,
            }),
        ],
    };
    // 每一步之后不跑池级 checker：txg 8 那次覆盖写之后 checker 就判红 I-3.1（被吞的根槽留下的），跑着的话历史停在那一步、走不到抬 F。
    let execution_without_the_per_step_checker = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let parameters = execution_without_the_per_step_checker
        .device_width
        .parameters();
    let root_slot_of_the_swallowed_publish = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(7),
        parameters.geometry.root_ring_slots_per_region,
    );
    let plan = SharedFaultPlan::armed(
        execution_without_the_per_step_checker
            .device_width
            .fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteIsSwallowed,
            device: FaultDeviceSelector::OnlyDevice(
                parameters.region_devices[usize::try_from(
                    root_slot_of_the_swallowed_publish.region,
                )
                .expect("区域号小于 3")],
            ),
            placement: FaultPlacement::OffsetExactly(singlefs_core::root_ring::slot_offset(
                root_slot_of_the_swallowed_publish,
                parameters.geometry.fixed_structure_slot_spacing,
            )),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(1),
        },
    );
    let run = execute_history_with_faults(
        &history,
        execution_without_the_per_step_checker,
        &SharedStream::new(),
        &plan,
        &mut |_| {},
    );
    assert_eq!(plan.fired().len(), 1, "只吞掉 txg 7 那一次根槽写");
    let HistoryEnding::NewFinding { observation, .. } = &run.ending else {
        panic!("不按注入点认时，抬 F 那一步对拍对不上：{:?}", run.ending);
    };
    assert_eq!(observation.position, StepPosition::Operation(5));
    let disagreement = observation
        .model_disagreement
        .as_ref()
        .expect("停在模型对不上");
    assert!(
        matches!(
            disagreement.aspect,
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
                | ModelDisagreementAspect::RefusalReason
        ),
        "模型要它成或按上限拒，实现拒了：{disagreement:?}"
    );
    assert!(
        disagreement
            .implementation_answer
            .contains("RollbackFloorCeilingRootRingSlotStillBadAfterOneReread"),
        "实现报的是「根环槽读坏、重读仍坏」：{}",
        disagreement.implementation_answer
    );
    assert_eq!(
        disagreement.implementation_reported_unreadable_place,
        Some(
            ModelReportedUnreadablePlace::RootRingSlotStillBadAfterOneReread(
                singlefs_harness::model_comparison::model_ring_position(
                    root_slot_of_the_swallowed_publish
                )
            )
        ),
        "实现点名的读坏的槽是 txg 7 那一次被吞的根槽写的槽"
    );

    let injection = inject_one_fault_into_the_segment(
        &history,
        execution_without_the_per_step_checker,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::Operation {
            step_index: 3,
            operation_kind: HistoryOperationKind::PublishOverwrite,
        },
        ordinal_of_the_root_slot_write_within_the_step(&history, 3),
    );
    assert!(
        injection.new_findings.is_empty(),
        "读坏的槽正是被吞那次写的，不许成新发现：{:?}",
        injection.new_findings
    );
    let signature = format!(
        "{:?}",
        FailureSignature::ModelDisagreement {
            aspect: disagreement.aspect.name()
        }
    );
    assert_eq!(
        injection.tally.lying_device_signatures.get(&signature),
        Some(&1),
        "按注入点认下的是抬 F 那一格：{:?}",
        injection.tally.lying_device_signatures
    );
}

/// 「写的实例表行」那一形（实四丙交回第五节第 6 条没判的那一格；实七造、判）：第一个文件（txg 3）之后覆盖写一次（txg 4），
/// 它的根槽 FUA 写被吞——记录与单元落了盘、根槽没落；接着可写挂载。挂载里的恢复择到 txg 3、施加 txg 4 的记录（走到 (1, 4)），
/// 给实例 1 写的行是 (1, 4, W)，W 取这次恢复施加的记录里最大的事务号（D23（journal 的角色与格式） 已定项 14 第 4 条），
/// 实现照条款写。模型不知道根槽没落，按「不建崩溃、一条记录都不施加」答 (1, 4, 0)，对拍报「写的实例表行」。
/// 这里钉住不按注入点认时对拍看到的样子：行里所选根的 txg 两边一样，差的只有 W，而实现写的 W 正是那条记录的事务号。
/// 按注入点认的那一半见 `the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied`。
#[test]
fn mount_after_a_swallowed_root_slot_write_writes_the_transaction_of_the_record_it_applied_while_the_model_expects_none_applied(
) {
    let execution = HANDING_EVERY_STEP_IMAGE_OUT;
    let parameters = execution.device_width.parameters();
    let root_slot_of_the_overwrite = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(4),
        parameters.geometry.root_ring_slots_per_region,
    );
    let plan = SharedFaultPlan::armed(
        execution.device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteIsSwallowed,
            device: FaultDeviceSelector::OnlyDevice(
                parameters.region_devices
                    [usize::try_from(root_slot_of_the_overwrite.region).expect("区域号小于 3")],
            ),
            placement: FaultPlacement::OffsetExactly(singlefs_core::root_ring::slot_offset(
                root_slot_of_the_overwrite,
                parameters.geometry.fixed_structure_slot_spacing,
            )),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(1),
        },
    );
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite_with_fill_seed(1),
            HistoryOperation::CloseAndMountWritable,
        ],
    };
    let mut image_before_the_mount: Option<MemoryPool> = None;
    let run = execute_history_with_faults(
        &history,
        execution,
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            if observation.position == StepPosition::Operation(0) {
                image_before_the_mount = Some(observation.image.clone());
            }
        },
    );
    assert_eq!(plan.fired().len(), 1, "只吞掉覆盖写那一次根槽写");
    let recovery_the_mount_runs = recover(
        &image_before_the_mount.expect("观察者见过覆盖写之后的镜像"),
        JournalPolicy::Consult,
    );
    assert_eq!(
        recovery_the_mount_runs.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "恢复择到 txg 3、施加 txg 4 的记录"
    );
    let applied_transaction = recovery_the_mount_runs.journal.maximum_applied_transaction;
    assert!(applied_transaction > 0, "施加的是一次带单元的覆盖写的记录");
    let HistoryEnding::NewFinding { observation, .. } = &run.ending else {
        panic!("今天这一格报成新发现：{:?}", run.ending);
    };
    assert_eq!(observation.position, StepPosition::Operation(1));
    let disagreement = observation
        .model_disagreement
        .as_ref()
        .expect("停在模型对不上");
    assert_eq!(disagreement.aspect, ModelDisagreementAspect::InstanceRows);
    for (who, rows, high_water) in [
        ("模型", &disagreement.model_answer, 0),
        (
            "实现",
            &disagreement.implementation_answer,
            applied_transaction,
        ),
    ] {
        assert!(
            rows.contains("selected_root_txg: ModelCheckpointTxg(4)")
                && rows.contains(&format!("applied_transaction_high_water: {high_water}")),
            "{who}写的行是 (1, 4, {high_water})：{rows}"
        );
    }
}

/// 「写的实例表行」那一形按注入点认（实七报告 (d)，实八）：同一段历史（第一个文件之后覆盖写一次、它的根槽写被吞、接着可写挂载），
/// 注入经 `inject_one_fault_into_the_segment` 走。模型拿历史停下那一步之前的自己、按「txg 4 的根槽没落、它的记录与单元落了」重算
/// 可写挂载写的行：恢复择到 txg 3、施加 txg 4 的记录，行是 (1, 4, W)，W 取那条记录的事务号（D23（journal 的角色与格式） 已定项 14 第 4 条）。
/// 实现写的正是这一行，认成说谎的设备留下的，不成新发现。
#[test]
fn the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite_with_fill_seed(1),
            HistoryOperation::CloseAndMountWritable,
        ],
    };
    let injection = inject_one_fault_into_the_segment(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::Operation {
            step_index: 0,
            operation_kind: HistoryOperationKind::PublishOverwrite,
        },
        ordinal_of_the_root_slot_write_within_the_step(&history, 0),
    );
    assert!(
        injection.new_findings.is_empty(),
        "实现写的行等于模型重算的行，不许成新发现：{:?}",
        injection.new_findings
    );
    let signature = format!(
        "{:?}",
        FailureSignature::ModelDisagreement {
            aspect: ModelDisagreementAspect::InstanceRows.name()
        }
    );
    assert_eq!(
        injection.tally.lying_device_signatures.get(&signature),
        Some(&1),
        "按注入点认下的是「写的实例表行」那一格：{:?}",
        injection.tally.lying_device_signatures
    );
}

/// 合并入口那两条用例用的那一段历史与注入次数：快档那组比重、种子基那一段 8 步，每段抽 6 次。
const MERGED_HISTORY_OPERATIONS: usize = 8;

const MERGED_FAULTS_PER_HISTORY: usize = 6;

/// 合并入口那一条用例里，用例替 checker 档判红的那一条不变量（真名，归类只看名字）与它的细节。
const INVARIANT_THE_TEST_JUDGES_RED: &str = "I-2.1";

/// 用例替 checker 档判的结论：起点之后的每一步（`StepPosition::Operation`）的镜像一律判红 [`INVARIANT_THE_TEST_JUDGES_RED`]，
/// 起点那一张不红；重开那张盘就是录制流整条，历史走到过起点之后写过盘的一步就红（同一个判法：那张盘是那一步镜像的延续）。
/// 被吞的写插回去那张盘不红。
fn verdicts_red_after_the_starting_point(
    images: &ImagesAfterAFault,
) -> PoolCheckerVerdictsAfterAFault {
    let red = || {
        vec![(
            INVARIANT_THE_TEST_JUDGES_RED,
            "用例替 checker 档判红".to_string(),
        )]
    };
    let violations_of_the_step_images: Vec<Vec<(&'static str, String)>> = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .map(|step_image| match step_image.position {
            StepPosition::StartingPoint => Vec::new(),
            StepPosition::Operation(_) => red(),
        })
        .collect();
    let reached_a_step_after_the_starting_point = violations_of_the_step_images
        .iter()
        .any(|violations| !violations.is_empty());
    PoolCheckerVerdictsAfterAFault {
        violations_of_the_step_images,
        violations_after_the_fault: if reached_a_step_after_the_starting_point {
            red()
        } else {
            Vec::new()
        },
        invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet::new(),
    }
}

/// 合并入口收到的结论一条都不红：交回的与 harness 不判池级 checker、把盘交出去的那一路（[`inject_faults_into_history`]）逐项相同，
/// 只差「交出去几次」那一格（合并入口交给判的不记在那一格）。每个注入点只交一次判。
/// 判别力：合并入口在不红的结论上也截短重跑、或把结论并错地方，交回的与那一路不同；交判的次数不是注入点数。
#[test]
fn merging_pool_checker_verdicts_that_never_turn_red_returns_what_the_harness_alone_judges() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        MERGED_HISTORY_OPERATIONS,
        &GenerationWeights::BROAD,
    );
    let mut hand_outs = 0usize;
    let mut merged =
        inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier(
            &history,
            HANDING_EVERY_STEP_IMAGE_OUT,
            MERGED_FAULTS_PER_HISTORY,
            &mut |images| {
                hand_outs += 1;
                PoolCheckerVerdictsAfterAFault {
                    violations_of_the_step_images: vec![
                        Vec::new();
                        images
                            .step_images_of_the_run_with_the_fault
                            .len()
                    ],
                    violations_after_the_fault: Vec::new(),
                    invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet::new(),
                }
            },
        );
    let mut handed_out = inject_faults_into_history(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        MERGED_FAULTS_PER_HISTORY,
    );
    assert!(!merged.drawn_faults.is_empty(), "前提：这一段摆出了注入点");
    assert_eq!(
        hand_outs,
        merged.drawn_faults.len(),
        "结论都不红时每个注入点只交一次判"
    );
    assert_eq!(
        (
            merged.tally.images_after_the_fault_handed_out,
            handed_out.tally.images_after_the_fault_handed_out
        ),
        (
            0,
            u64::try_from(handed_out.drawn_faults.len()).expect("装得进 u64")
        ),
        "合并入口交给判的不记在「交出去几次」那一格，交出去的那一路每个注入点记一次"
    );
    merged.tally.images_after_the_fault_handed_out = 0;
    handed_out.tally.images_after_the_fault_handed_out = 0;
    assert_eq!(
        merged, handed_out,
        "结论一条都不红时合并入口交回的与 harness 自己判的那一路逐项相同"
    );
}

/// 合并入口收到某个注入点上一张每步镜像判红：照判红就停那一档的口径，历史停在那一步——这个注入点截短重跑一遍，重跑那一遍交出去的每步镜像
/// 就是整段那一遍到判红那一张为止（含）的那几张、重开的正是那一张的盘，再交一次判；新发现里的那一条停在判红那一步、违例是判红的那一条。
/// 用例替 checker 档判（[`verdicts_red_after_the_starting_point`]：起点之后的每一步都红），逐次记它被交了什么。
/// 判别力：不截短（`crates/mutations.tsv` 同名那一行把找第一张判红的那一处改成永远找不到），第二次交判没了、新发现落在注入点那一段而不是判红那一步。
#[test]
fn a_step_image_the_checker_tier_judges_red_stops_the_merged_history_at_that_step() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        MERGED_HISTORY_OPERATIONS,
        &GenerationWeights::BROAD,
    );
    let mut asked: Vec<ImagesAfterAFault> = Vec::new();
    let merged = inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        MERGED_FAULTS_PER_HISTORY,
        &mut |images| {
            asked.push(images.clone());
            verdicts_red_after_the_starting_point(images)
        },
    );
    let mut asked_in_order = asked.iter();
    let mut stopped_positions: Vec<StepPosition> = Vec::new();
    for drawn in &merged.drawn_faults {
        let run_through = asked_in_order
            .next()
            .unwrap_or_else(|| panic!("{} 没交判", drawn.render()));
        assert_eq!(run_through.drawn, *drawn, "交判的次序就是注入点的次序");
        let Some(first_red) = run_through
            .step_images_of_the_run_with_the_fault
            .iter()
            .position(|step_image| matches!(step_image.position, StepPosition::Operation(_)))
        else {
            continue;
        };
        let stopped = asked_in_order.next().unwrap_or_else(|| {
            panic!(
                "{} 的第 {first_red} 张每步镜像判红，要截短重跑、再交一次判，没交",
                drawn.render()
            )
        });
        assert_eq!(stopped.drawn, *drawn, "截短重跑的是同一个注入点");
        assert_eq!(
            stopped.step_images_of_the_run_with_the_fault.as_slice(),
            &run_through.step_images_of_the_run_with_the_fault[..=first_red],
            "{}：截短重跑那一遍停在判红那一张",
            drawn.render()
        );
        assert_eq!(
            stopped.image_after_the_fault.operations.len(),
            run_through.step_images_of_the_run_with_the_fault[first_red].operations_recorded,
            "{}：截短重跑那一遍重开的就是判红那一步的盘",
            drawn.render()
        );
        stopped_positions
            .push(run_through.step_images_of_the_run_with_the_fault[first_red].position);
    }
    assert!(
        asked_in_order.next().is_none(),
        "交判的只有每个注入点整段那一遍与判红时截短那一遍"
    );
    assert!(
        !stopped_positions.is_empty(),
        "前提：至少一个注入点上历史走到了起点之后写过盘的一步（有判红的每步镜像）"
    );
    let red_signature = FailureSignature::CheckerViolations {
        invariants: vec![INVARIANT_THE_TEST_JUDGES_RED],
    };
    let red_finding = merged
        .new_findings
        .iter()
        .find(|finding| finding.signature == red_signature)
        .unwrap_or_else(|| panic!("判红那一条要成新发现：{:?}", merged.new_findings));
    assert!(
        stopped_positions.contains(&red_finding.observation.position),
        "新发现停在判红的那一步（{:?} 之一），不在注入点那一段：{:?}",
        stopped_positions,
        red_finding.observation.position
    );
    assert_eq!(
        red_finding
            .observation
            .violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect::<Vec<_>>(),
        vec![INVARIANT_THE_TEST_JUDGES_RED],
        "停下那一步的违例就是判红的那一条"
    );
}
