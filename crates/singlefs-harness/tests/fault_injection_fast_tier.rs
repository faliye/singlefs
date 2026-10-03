//! 里程碑「覆盖写、释放、回退与复用」增补 3 第 4 件：故障注入。通用的设备包装、注入点怎么摆、注入之后怎么判在
//! `singlefs_harness::fault_injection`；这里是快档（普通 `cargo test`）、多线程对拍、写死的那几条用例与大档（`#[ignore]`，规模从环境变量取）。
//!
//! 被测的两条性质：注入之后 `singlefs-core` **返回错误而不是 panic**；出错之后**重开恢复到模型允许的版本**。
//! 种子基是这个测试周期写死的那一个（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，随机历史与崩溃注入那两个二进制用的是同一个）。

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, resend_the_frozen_publish, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError,
};
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::fault_injection::{
    inject_faults_into_history, inject_one_fault_into_the_segment, run_fault_injection_campaign,
    AcquiredInstanceLeft, AcquiredInstanceLeftAfterAFailedMount, DrawnFault, FaultCounting,
    FaultDeviceSelector, FaultInjectingBlockDevice, FaultInjectionCampaign, FaultInjectionReport,
    FaultInjectionTally, FaultInjectionWorkerThreads, FaultOccurrence, FaultOutcome,
    FaultPlacement, FaultSchedule, FaultedSegment, FaultedSegmentKind, FlippedBit, InjectedFault,
    KnownRedAtAFault, KnownRedFormAtAFault, KnownRedFormOnlyAfterAFault, SharedFaultPlan,
};
use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::history::{
    count_from_reproduction_environment, environment_assignments_in_line, execute_history_with,
    execute_history_with_faults, generate_history_with_weights,
    number_from_reproduction_environment, render_environment_assignments, ContentChoice,
    ContentLength, FailureSignature, FloorTargetChoice, GeneratedHistory, GenerationWeights,
    HarnessJudgement, HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation,
    HistoryOperationKind, HistoryRun, HistorySampling, HistorySeed, HistoryStartingPoint,
    PerStepChecker, ReproductionEnvironmentError, ReproductionVariablePrefix, StartingPointStep,
    StepPosition,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::model::ModelDisagreementAspect;
use singlefs_harness::numeric_environment_variable::process_environment_variable;
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 写死的那几条用例里两块内存盘各多大（与各步用例、随机历史的 4 GiB 档相同）。
const IMAGE_BYTES: u64 = 4 << 30;

/// 快档的规模：段数、每段步数、每段注入几次。每次注入都要把这段历史整个重跑一遍，所以规模按「别显著变慢」定。
const FAST_TIER_SEEDS: u64 = 24;
const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 20;
const FAST_TIER_FAULTS_PER_HISTORY: usize = 4;

/// 注入之后这段历史怎么跑：每一步之后跑池级 checker、判红就停，两块 4 GiB 的盘。
/// 跑 checker 是这一件的一半——注入之后盘面坏没坏，只有它看得见。
const CHECKED_AFTER_EVERY_STEP: HistoryExecution =
    HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES;

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
        execution: CHECKED_AFTER_EVERY_STEP,
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
    // 说谎的设备真的丢掉过内容：这一格是零的话，说谎那两种注得进去却什么也没碰着，
    // `InjectedFault::the_device_lies` 的那道豁免就成了一张白给的免罪符，判别力要重新量
    //（`mutation-sampling.md`「取样点不敏感」）。
    assert!(
        tally.faults_where_a_lying_device_left_the_image_inconsistent > 0,
        "说谎的设备一次都没把盘面弄脏：说谎那两种抽到了却什么也没碰着\n{rendered}"
    );
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
    assert!(
        tally.checker_runs_on_the_image_after_the_fault >= tally.faults,
        "每次注入之后都要对镜像跑一次池级 checker\n{rendered}"
    );
}

/// 大档没设环境变量时每段几步、注入几次（大档撞出的单段回归用例照同一组规模重放那一段）。
const LARGE_TIER_OPERATIONS_PER_HISTORY: u64 = 30;
const LARGE_TIER_FAULTS_PER_HISTORY: u64 = 6;

/// 大档注入之后这段历史怎么跑：每一步之后跑池级 checker、空间准入按公式判，盘宽由调用方给（大档从环境变量取）。
fn large_tier_execution(device_width: HistoryDeviceWidth) -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Run,
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
#[ignore = "大档：照判红时打的重放行设环境变量；SINGLEFS_FAULT_INJECTION_FIRST_SEED 起 SINGLEFS_FAULT_INJECTION_SEEDS 段、每段 SINGLEFS_FAULT_INJECTION_OPERATIONS 步、每段注入 SINGLEFS_FAULT_INJECTION_FAULTS 次，比重 SINGLEFS_FAULT_INJECTION_WEIGHTS（broad、reuse、rollback、wall、unit-area-wall），checker SINGLEFS_FAULT_INJECTION_CHECKER（run、skip），盘宽 SINGLEFS_FAULT_INJECTION_DEVICES（4gib、unit-area-384、unit-area-256、unit-area-240），空间准入 SINGLEFS_FAULT_INJECTION_SPACE_ADMISSION（judged、skipped），线程数 SINGLEFS_FAULT_INJECTION_WORKER_THREADS；没设的取大档默认（种子基、512 段、30 步、6 次、broad、run、4gib、judged）；按里程碑「增补 3」后台跑"]
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
            && campaign.execution.per_step_checker != defaults.execution.per_step_checker
            && campaign.execution.device_width != defaults.execution.device_width
            && campaign.execution.space_admission != defaults.execution.space_admission,
        "前提：这一批的每一项都与大档的默认不同：{campaign:?} 对 {defaults:?}"
    );
    the_replay_line_printed_for_a_batch_runs_the_same_campaign_on_the_large_tier_entry(&campaign);
}

/// 说谎的设备许可留下的盘面不一致里 `["I-3.1"]` 那一组的取样点（`fault_injection::INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE`；
/// `crates/mutations.tsv` 钉着把被吞的写插回去那一步拿掉的变异）。快档那 24 段里说谎的设备留下的不一致全落在另一组
/// （`["I-2.1", "I-4.8", "I-7.4"]`），这一组要单独钉。
///
/// 取样点写死成一段历史与一个注入点，不按种子从比重表里抽：此前钉的是种子 7463871032432355210 在 `GenerationWeights::BROAD` 上抽出的
/// 第 25 步覆盖写那次根槽写，删除批把比重表里可写挂载那一格切出 unlink 之后同一个种子抽出的历史变了、那次被吞的写不在了，用例跟着红
/// （调度记录 `records/2026-10-01-删除与并发事务.md` 第 11 批的慢档红调查问 1；比重表一改，种子钉的取样点就会再丢一次）。
/// 这一段：第一个文件（txg 3）之后覆盖写五次（txg 4–8），txg 7 那次（第 3 步）的根槽 FUA 写被吞；每一步之后照跑池级 checker，
/// txg 8 那次覆盖写之后 checker 只判红 I-3.1（被吞的根槽留下的，被吞的写插回去就消失）。它要按注入点认成说谎的设备留下的、
/// 记进说谎那一格，不算新发现——带着注入跑的那一遍里每一步之后的池级 checker 判红那一处、注入之后的镜像上再跑的那一遍，各记一次。
#[test]
fn swallowed_write_after_which_the_checker_flags_only_the_allocated_statistic_invariant_is_excused_as_what_the_lying_device_may_leave(
) {
    const STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE: usize = 3;
    let history = GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: (1..=5).map(overwrite_with_fill_seed).collect(),
    };
    let injection = inject_one_fault_into_the_segment(
        &history,
        CHECKED_AFTER_EVERY_STEP,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::Operation {
            step_index: STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE,
            operation_kind: HistoryOperationKind::PublishOverwrite,
        },
        ordinal_of_the_root_slot_write_within_the_step(
            &history,
            STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE,
        ),
    );
    assert!(
        injection.new_findings.is_empty(),
        "说谎的设备留下的 I-3.1 要被白名单豁免，不算新发现：{:?}",
        injection.new_findings
    );
    assert_eq!(
        injection
            .tally
            .lying_device_signatures
            .get("CheckerViolations { invariants: [\"I-3.1\"] }"),
        Some(&2),
        "这一段里被吞掉的那一次写之后只判红 I-3.1，记进说谎那一格——带着注入跑的那一遍里每一步之后的池级 checker 判红那一处、\
         注入之后的镜像上再跑的那一遍，各记一次：{:?}",
        injection.tally.lying_device_signatures
    );
}

/// 大档（这个测试周期的种子基起 512 段）撞出的一段，单独跑：种子 7463871032432355397，规模与执行方式与大档没设环境变量时逐项相同。
/// 抽出的 6 个注入点里有一次被吞掉的写（`write_is_swallowed`，整池第 471 次写调用，第 17 步覆盖写那次发布的根槽写，落在根环区域 0 槽 6）；
/// 第 18 步挂着时回退判候选集读根环，这个进程知道住着根的区域 0 槽 6 读坏、重读仍坏，拒在任何写之前
/// （`RollbackError::CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`，D16（发布语义） 已定项 1
/// 「根槽这一次读坏」那一行的读法）。模型不知道根槽没落、说该成；回退点名的槽正是被吞那次写的槽，要记进说谎那一格，不算新发现。
/// 回退路径不把点名的槽交给模型比对时（`crates/mutations.tsv` 钉着的那一格），这一格判不出是说谎的设备留下的，照报新发现。
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
        Some(&1),
        "回退那一次「模型说该成、实现拒了」要按点名的槽等于被吞根槽写的槽放过、记进说谎那一格，只这一次：\n{rendered}"
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
    let injection = inject_faults_into_history(&history, CHECKED_AFTER_EVERY_STEP, 12);
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
    let injection = inject_faults_into_history(&history, CHECKED_AFTER_EVERY_STEP, 12);
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
    let geometry = CHECKED_AFTER_EVERY_STEP.device_width.fixed_geometry();
    let plan = match failing_write_ordinal {
        None => SharedFaultPlan::unarmed(geometry),
        Some(ordinal) => SharedFaultPlan::armed(
            geometry,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, ordinal),
        ),
    };
    let run = execute_history_with_faults(
        history,
        CHECKED_AFTER_EVERY_STEP,
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

/// 起点段失败时跑不跑池级 checker：**mkfs 自己就报错那一次不跑**（盘上没有一个做完了的文件系统，
/// checker 判的每一条都没有对象），**取号 / 暖机 / 第一个文件报错那几次照跑**（mkfs 已经做完，盘上有池）。
///
/// 用户 2026-09-21 定的口径，分界与「重开走读失败算不算失败」同一条：mkfs 做没做完
/// （`history.rs` 的 `StartingPointStep::finished_filesystem_is_on_the_devices`）。
/// 起点段 44 次写逐次注入，两边都数出来——口径被改掉（两边都跑、或者两边都不跑），这里必红。
#[test]
fn the_pool_checker_runs_on_a_starting_point_failure_only_when_make_filesystem_finished() {
    let history = history_with_only_the_starting_point();
    // 不注入时起点段跑完，起点之后跑一次 checker：下面那两格拿它当对照。
    let (measured, measurement) = run_the_starting_point(&history, None);
    assert!(
        matches!(measured.ending, HistoryEnding::Completed),
        "不注入时起点段要跑完：{:?}",
        measured.ending
    );
    assert_eq!(
        measured.tally.checker_runs, 1,
        "起点段跑完之后该跑一次池级 checker"
    );
    let mut checker_runs_by_step: BTreeMap<&'static str, BTreeSet<u64>> = BTreeMap::new();
    for ordinal in 1..=measurement.calls().writes {
        let (run, _plan) = run_the_starting_point(&history, Some(ordinal));
        let Some(step) = the_starting_point_step_that_returned_an_error(&run, ordinal) else {
            continue;
        };
        checker_runs_by_step
            .entry(step.name())
            .or_default()
            .insert(run.tally.checker_runs);
        let expected_checker_runs = u64::from(step.finished_filesystem_is_on_the_devices());
        assert_eq!(
            run.tally.checker_runs,
            expected_checker_runs,
            "起点段第 {ordinal} 次写上注入、停在{}：mkfs 做完了才该跑 checker",
            step.name()
        );
    }
    print_uncaptured(&format!(
        "── 起点段失败时的 checker ──\n每一步报错时 checker 跑过几次：{checker_runs_by_step:?}\n"
    ));
    // 两边都要有样本，不然上面那条断言是空过的。
    assert_eq!(
        checker_runs_by_step.get(StartingPointStep::MakeFilesystem.name()),
        Some(&BTreeSet::from([0])),
        "mkfs 报错那几次要有样本，而且一次 checker 都没跑：{checker_runs_by_step:?}"
    );
    for step in [
        StartingPointStep::AcquireInstance,
        StartingPointStep::WarmUp,
        StartingPointStep::PublishFirstFile,
    ] {
        assert_eq!(
            checker_runs_by_step.get(step.name()),
            Some(&BTreeSet::from([1])),
            "{}报错那几次要有样本，而且各跑过一次 checker：{checker_runs_by_step:?}",
            step.name()
        );
    }
}

/// 快档在 2026-10-03 落进这一形的那一段：种子基 + 7（7463871032432355120），快档的规模与比重。
const SEED_WHERE_MAKE_FILESYSTEM_FAILS_BEFORE_ANY_SYSTEM_CONFIGURATION_SLOT_LANDS: u64 =
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7;

/// 用户 2026-10-03 定：mkfs 在写到系统配置之前失败的盘面就是全池皆废，I-8.5（tail 槽有效集非空）照判红，红是对的；
/// 故障注入的「已知红」清单加这一类。快档在种子 7463871032432355120 上摆的那一个注入点（整池第 1 次 barrier 报错，落在 mkfs 第一道屏障上，
/// 那一刻 mkfs 只清过零、写过实例表与树表的创世单元，系统配置槽一次都没写）原样重跑：注入之后的镜像上只红 I-8.5、
/// 重开走读失败，判法记成那一条「已知红」，不记新发现。
///
/// 判别力自证：判法改回不认这一类（`known_red_form_only_after_a_fault` 恒交回 None，`crates/mutations.tsv` 同名那一条），
/// 这一次落成新发现 `CheckerViolations { invariants: ["I-8.5"] }`，这里红。
#[test]
fn make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding(
) {
    let history = generate_history_with_weights(
        HistorySeed(SEED_WHERE_MAKE_FILESYSTEM_FAILS_BEFORE_ANY_SYSTEM_CONFIGURATION_SLOT_LANDS),
        FAST_TIER_OPERATIONS_PER_HISTORY,
        &GenerationWeights::BROAD,
    );
    let injection = inject_one_fault_into_the_segment(
        &history,
        CHECKED_AFTER_EVERY_STEP,
        InjectedFault::BarrierFails,
        FaultedSegment::TheStartingPoint,
        1,
    );
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
        injection.new_findings.is_empty(),
        "mkfs 在系统配置槽落盘之前报错、只红 I-8.5 是认下的「已知红」，不是新发现：{:?}\n{rendered}",
        injection
            .new_findings
            .iter()
            .map(|finding| &finding.signature)
            .collect::<Vec<_>>()
    );
    let expected_form = KnownRedFormAtAFault::OnlyAfterAFault(
        KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
    );
    assert_eq!(
        injection.known_red_hits,
        vec![KnownRedAtAFault {
            seed: HistorySeed(
                SEED_WHERE_MAKE_FILESYSTEM_FAILS_BEFORE_ANY_SYSTEM_CONFIGURATION_SLOT_LANDS
            ),
            form: expected_form,
            drawn: expected_drawn,
        }],
        "这一次记成「mkfs 没写下系统配置槽」那一条已知红\n{rendered}"
    );
    assert_eq!(
        injection.tally.faults_ending_known_red,
        BTreeMap::from([(expected_form, 1)]),
        "计数里以这一条已知红收尾一次\n{rendered}"
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

/// 增补 2 收口表第 40 行（C381（根已落盘之后发布失败，分配器仍退回））那一格，按 D23（journal 的角色与格式） 已定项 14
/// 「这一版的失败处置」（用户 2026-09-24 定：发布不接受失败，失败的那次冻结，下一次发布之前逐字节原样重发它）：
/// 发布 B 在最后一步（系统配置槽）失败时根已 FUA 落盘、分配器退回——改之前同一个写入口拿同一个上一版再发一次 C，就把那条根指着的单元
/// 原地盖掉，那一次再断电，盘上留下「最新的那条根指着一份内容已经换掉的单元」（池级 checker 判 I-7.4 / I-7.2 红、冷启动走不到那条根）。
/// 现在 B 冻结在分配器上：再发 C 在任何写之前被拒；原样重发 B 之后那条根指着的单元就是它自己写的那一份，checker 不红、冷启动读回 B；
/// 之后 C 接在 B 上照常发布。
#[test]
fn publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root(
) {
    let parameters = e142_parameters(512, 512);
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let stream = SharedStream::retaining_contents();
    let plan = SharedFaultPlan::unarmed(geometry);
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                    plan.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    // mkfs 同一个进程里接着写的分配器照 mkfs 按环长现算的单元区起点建（实审 A3b 报告 Q8：环境变量选小盘那一档时环不是默认的 768 MiB，
    // 起点不是编译期常量 50176），并装上 mkfs 刚写下的根环。
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first = publish_first_file(
        &mut writer,
        &mut allocator,
        warmed.roots.last().expect("暖机两代根"),
        FirstFile {
            content: &first_file_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("新池新建文件");
    assert_eq!(first.root.checkpoint_txg, CheckpointTxg(3));

    // 一、发布 B：系统配置槽那两次写全报错。根槽 FUA 写在它们之前，已经落盘。
    plan.arm(FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::OffsetBelow(
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                * u64::from(parameters.geometry.fixed_structure_slot_spacing),
        ),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let refused = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first,
        FirstFile {
            content: &second_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect_err("系统配置槽写报错，这次发布必须失败");
    assert!(
        matches!(refused, PublishError::BlockDevice(_)),
        "报的应当是块设备错：{refused:?}"
    );
    let fired = plan.fired();
    assert_eq!(fired.len(), 1, "系统配置槽第一次写就报错、发布到此为止");
    assert_eq!(
        fired[0].written_structure,
        Some(StepKind::SystemConfigurationSlot),
        "注入的那一次要落在系统配置槽上"
    );
    let image_after_the_failed_publish = image_from(&stream);
    let roots_after: Vec<u64> = readable_root_txgs(&image_after_the_failed_publish);
    assert!(
        roots_after.contains(&4),
        "C381 的前提：这次发布报了失败，txg 4 那条根却已经 FUA 落盘、在环里读得出来；读到的是 {roots_after:?}"
    );

    // 二、同一个写入口、同一个上一版再发一次：B 冻结着，第一道就拒，一个写都不发。
    let fired_before_the_refused_publish = plan.fired_count();
    let operations_before_the_refused_publish = stream.operations().len();
    let refused_again = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first,
        FirstFile {
            content: &third_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        instance,
    )
    .expect_err("B 冻结着，另建一次发布必须被拒");
    assert!(
        matches!(
            refused_again,
            PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                checkpoint_txg: CheckpointTxg(4),
                ..
            }
        ),
        "报的应当是「B 冻结着、还没重发」：{refused_again:?}"
    );
    assert_eq!(
        (plan.fired_count(), stream.operations().len()),
        (
            fired_before_the_refused_publish,
            operations_before_the_refused_publish
        ),
        "被拒的那一次一个写、一道屏障都没发"
    );

    // 三、盘好了，原样重发 B；之后 C 接在 B 上照常发布。
    plan.disarm();
    let second = resend_the_frozen_publish(&mut writer, &mut allocator)
        .expect("盘好了，重发这一遍不再报错")
        .and_then(PoolVersion::into_file_version)
        .expect("冻结着的是带文件的发布 B");
    assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
    let image_after_the_resend = image_from(&stream);
    let violations_after_the_resend: Vec<&'static str> = check_pool_image(&image_after_the_resend)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    assert!(
        violations_after_the_resend.is_empty(),
        "重发之后最新那条根（B 的 txg 4）指着的单元就是它自己写的那一份：checker 不红，判红的是 {violations_after_the_resend:?}"
    );
    let recovery = recover(&image_after_the_resend, JournalPolicy::Consult);
    assert!(
        matches!(
            &recovery.outcome,
            RecoveryOutcome::FileRead { root, content }
                if *root == (instance, CheckpointTxg(4)) && *content == second_content()
        ),
        "冷启动读回 B：{:?}",
        recovery.outcome
    );
    let third = publish_overwrite(
        &mut writer,
        &mut allocator,
        &second,
        FirstFile {
            content: &third_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        instance,
    )
    .expect("重发之后 C 照常发布");
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(5));
}

/// 起点（mkfs、取号 1、暖机、第一个文件）之后只做一步：关掉会话、可写挂载（取号 2 → 写行 → 暖机）。
fn history_that_mounts_once_after_the_first_file() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::CloseAndMountWritable],
    }
}

/// 那一步可写挂载里的第几次写：取号是头两次（两块盘各一次系统配置槽写），写行那次发布的第一个单元写是第 3 次；
/// 写行那次发布一共 23 次写（实例表两盘 2 次、分配记录树按位置寻址这一次重写的节点两盘 10 次、记账 / 映射 / 树表各两盘 6 次、
/// 记录两盘 2 次、根槽 1 次、系统配置槽轮换 2 次），暖机第一次空发布的第一个单元写是第 2 + 23 + 1 = 26 次
/// （这一步一共 46 次写：暖机一次 21 次）。分配记录树按位置寻址（D8（核心索引结构） 已定项 14）之前那棵树只有一个节点，
/// 写行那次 15 次写、这一格摆在第 18 次；照旧摆在第 18 次就落进写行那次、记成「新号一条根都没有」。
const FIRST_ROW_PUBLISH_WRITE_OF_THE_MOUNT: u64 = 3;
const FIRST_WARM_UP_WRITE_OF_THE_MOUNT: u64 = 26;

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
            CHECKED_AFTER_EVERY_STEP,
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

/// 发布 B 的内容（4100 字节，与虚机二进制 `file-overwrite` 模式相同）。
fn second_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 发布 C 的内容：与发布 B 不同，单元字节才会真的变。
fn third_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 11 + 5) % 251).expect("小于 256"))
        .collect()
}

/// 录制流里真正落到盘上的那些写重建出来的镜像。
fn image_from(stream: &SharedStream) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&stream.retained_operations());
    image
}

/// 第一个文件（txg 3）之后覆盖写，每次换一份内容。
fn overwrite_with_fill_seed(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed,
    })
}

/// 不注入跑一遍这段历史，数第 `step_index` 步发出的写里落在根槽上的那一次是这一步的第几次写（1 起）：
/// `inject_one_fault_into_the_segment` 按「这一段发出的第几次写」摆注入点。
fn ordinal_of_the_root_slot_write_within_the_step(
    history: &GeneratedHistory,
    step_index: usize,
) -> u64 {
    let stream = SharedStream::new();
    let mut stream_length_after: BTreeMap<StepPosition, usize> = BTreeMap::new();
    let run = execute_history_with(
        history,
        CHECKED_AFTER_EVERY_STEP,
        &stream,
        &mut |observation| {
            stream_length_after.insert(observation.position, stream.operation_count());
        },
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "不注入时这段历史跑完");
    let step_start = match step_index.checked_sub(1) {
        None => stream_length_after[&StepPosition::StartingPoint],
        Some(previous) => stream_length_after[&StepPosition::Operation(previous)],
    };
    let step_end = stream_length_after[&StepPosition::Operation(step_index)];
    let geometry = CHECKED_AFTER_EVERY_STEP.device_width.fixed_geometry();
    let writes_of_the_step: Vec<_> = stream.operations()[step_start..step_end]
        .iter()
        .filter(|operation| operation.kind != singlefs_harness::RecordedOperationKind::Barrier)
        .cloned()
        .collect();
    let index = writes_of_the_step
        .iter()
        .position(|operation| geometry.classify(operation) == StepKind::RootRecordFua)
        .expect("一次发布恰好写一次根槽");
    u64::try_from(index + 1).expect("一步至多几十次写")
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
        disagreement.implementation_reported_root_ring_slot_still_bad_after_one_reread,
        Some(singlefs_harness::model_comparison::model_ring_position(
            root_slot_of_the_swallowed_publish
        )),
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
    let execution = CHECKED_AFTER_EVERY_STEP;
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
        CHECKED_AFTER_EVERY_STEP,
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

/// 环里自证过的根的 txg，从小到大。
fn readable_root_txgs(image: &MemoryPool) -> Vec<u64> {
    let system_configuration = choose_system_configuration(image).expect("择系统配置");
    let mut txgs: Vec<u64> = readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root.checkpoint_txg.0)
    .collect();
    txgs.sort_unstable();
    txgs
}
