//! 验证包模块：crash、crash_point_replay_progress、crash_injection、crash_library、crash_amplification、verdict_store、gpu_unit_checks
//! 里程碑「覆盖写、释放、回退与复用」增补 3 第 3 件：崩溃注入。生成器、执行器与理想模型在第 1、2 件（`singlefs_verification::history`、
//! `singlefs_verification::model`），切段、摆崩溃状态、重建与判定在 `singlefs_verification::crash_injection`；
//! 这里是崩溃注入这一批（不设变量跑默认批，设了 `SINGLEFS_CRASH_INJECTION_*` 照变量跑）、多线程对拍与写死的那几段历史（段内真子集全枚举）。
//!
//! 枚举域与崩溃点重放同一个（屏障与 FUA 写切段、段内任意真子集）；差别在怎么取：崩溃点重放在两条写死的流上全枚举（门禁 crash-point-replay），
//! 这里在随机历史上抽样。种子基是这个测试周期写死的那一个（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，随机历史那个二进制用的是同一个），
//! 打在报告第一行与失败信息里（随机说的是一个测试周期与下一个之间，不是每次跑重抽）。
//! 第一截与第三截上只读一张镜像的那几样（池级 checker、两遍恢复、记录核对器）交给定状态表判（J3、J5、J6）：
//! 这一批经崩溃点重放的引擎判（`crash_amplification::GivenStatesEngine`：照本机多机配置只走 GPU 或只走 CPU、
//! 结论存进判定库）；编进 `gpu` 特性另有两条小规模用例：同一小批 GPU 一趟、CPU 一趟分开判，两趟的报告逐样相同，
//! 往 GPU 那一侧塞一个故障两趟就不同。

use std::collections::BTreeMap;
use std::io::Write as _;

use singlefs_checker::walk::InstanceTableOfRootRecord;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader};
use singlefs_format::DATA_UNIT_BYTES;
use singlefs_verification::crash::check_records_against;
use singlefs_verification::crash_injection::{
    crash_injection_campaign_from_environment, inject_crashes_into_history,
    CrashInjectionAssertions, CrashInjectionCampaign, CrashInjectionReport, CrashInjectionTally,
    CrashInjectionWorkerThreads, CrashPoint, CrashPointDraw, CrashPointFinding, CrashStateStage,
    FailureImageRetention, HistoryAfterTheMakeFilesystem, HistoryCrashInjection,
};
use singlefs_verification::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_verification::history::{
    environment_assignments_in_line, execute_history_with, generate_history,
    generate_history_with_weights, ContentChoice, ContentLength, FailureObservation,
    FailureSignature, FloorTargetChoice, GeneratedHistory, GenerationWeights, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySeed,
    HistoryStartingPoint, PerStepChecker, RollbackTargetChoice, StepOutcome, StepPosition,
};
use singlefs_verification::memory_pool::{
    root_identity_written_by, writes_and_segments, writes_and_segments_with_stream_indexes,
    CrashImage, MemoryPool, RecordCheck, RecordStreamContinuity, RetainedWrite,
};
use singlefs_verification::numeric_environment_variable::process_environment_variable;
use singlefs_verification::segments::StepKind;
use singlefs_verification::SharedStream;

use singlefs_verification::crash_amplification::{
    FaultPutOnTheGpuSide, GivenStatesEngine, GivenStatesItem, GivenStatesRun,
};
use singlefs_verification::crash_injection::{
    run_crash_injection_campaign_judging_the_given_states_with, GivenStatesConclusionsOfATable,
    GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
    GivenStatesOfATableWithItsColumns, GIVEN_STATES_TABLES_LEFT_TO_OTHER_HOSTS_NOTE,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;

#[cfg(feature = "gpu")]
#[path = "../common_gpu_cards_of_the_caller/mod.rs"]
mod common_gpu_cards_of_the_caller;

mod default_batch_variables_and_replay_lines;
mod findings_of_the_record_checker_alone;
mod gpu_and_cpu_runs_stop_files_shares_and_other_hosts;
mod kept_publishes_missing_a_unit_and_versions_rebuilt_from_records;
mod pool_after_the_writable_mount_that_follows_a_crash;
mod written_out_histories_recover_into_committed_versions;

use default_batch_variables_and_replay_lines::{
    crash_injection_campaign_from, judge_crash_injection_report,
};

/// 崩溃点重放合表里这一项的名字（`.claude/gate.d/crash-point-replay-items.tsv`）。
const CRASH_POINT_REPLAY_ITEM: &str = "crash-injection-campaign";

/// 一批崩溃注入的种子与规模（进给定状态表的块键，H8「键 = 种子 + 规模参数 + 测试文件出发的整份闭包」）：线程数、镜像留不留
/// 与照哪一组断言判不改哪个状态判成什么，不进。
fn seeds_and_scale_of(campaign: &CrashInjectionCampaign) -> String {
    format!(
        "first_seed={} seed_count={} operations_per_history={} draw={:?} weights={:?} execution={:?}",
        campaign.first_seed,
        campaign.seed_count,
        campaign.operations_per_history,
        campaign.draw,
        campaign.weights,
        campaign.execution
    )
}

/// 一批崩溃注入经崩溃点重放的引擎判给定状态表（`fault` 只给证明 GPU 与 CPU 两趟的报告会不同的那条用例），交回这一批的报告与引擎那一趟。
fn campaign_through_the_given_states_engine(
    campaign: &CrashInjectionCampaign,
    fault: FaultPutOnTheGpuSide,
) -> (CrashInjectionReport, GivenStatesRun) {
    campaign_through_the_given_states_engine_started(
        campaign,
        fault,
        EngineStart::UnderTheCallersEnvironmentGuard,
    )
}

/// 起引擎时拿不拿改环境的锁：拿着 [`CrashAmplificationEnvironment`] 的调用方不许再拿（同一把锁，不可重入），别的调用方要拿
/// （同一个进程里别的用例拿着守卫换叫停文件、分片与这个进程开的那一份库，引擎起步读环境那一刻不许撞上）。
/// 起在调用方给的一次性库上的那一种先把它设成这个进程开的那一份、再拿改环境的锁（全仓用例都是这个次序），引擎起完两把都放：
/// 引擎自己连着那一份的临时 prune-server，一次性库由调用方持着、比引擎活得长。
/// 起在全仓那一份上的那一种（走崩溃点重放入口的这一批）先拿全仓那一份的守卫、再拿改环境的锁，引擎起完两把都放：
/// 不拿守卫的，同一个二进制里设一次性库的用例在引擎开库那一刻设着，引擎就开到那一份、那条用例收尾停服之后连不上；起完之后引擎连着全仓那一份，谁再设都不碍它。
#[derive(Clone, Copy)]
enum EngineStart<'library> {
    UnderTheCallersEnvironmentGuard,
    HoldingTheEnvironmentLockOnTheOneLibrary,
    HoldingTheEnvironmentLockOnTheDisposableLibrary(
        &'library singlefs_verification::crash_library::DisposableLibrary,
    ),
}

fn campaign_through_the_given_states_engine_started(
    campaign: &CrashInjectionCampaign,
    fault: FaultPutOnTheGpuSide,
    start: EngineStart<'_>,
) -> (CrashInjectionReport, GivenStatesRun) {
    let start_the_engine = || {
        GivenStatesEngine::start_from_environment_putting_a_fault_on_the_gpu_side(
            GivenStatesItem {
                item_name: format!("{}::{}", CRASH_POINT_REPLAY_ITEM, seeds_and_scale_of(campaign)),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
            fault,
        )
        .expect("崩溃点重放的引擎照本机配置起得来（配了 GPU 卡用不上、结论版本与编译时对不上都在这里红，不退回 CPU）")
    };
    let engine = match start {
        EngineStart::UnderTheCallersEnvironmentGuard => start_the_engine(),
        EngineStart::HoldingTheEnvironmentLockOnTheOneLibrary => {
            let _the_one_library = singlefs_verification::crash_library::TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
            let _one_test_at_a_time = singlefs_verification::crash_amplification::one_test_changes_the_environment_at_a_time();
            start_the_engine()
        }
        EngineStart::HoldingTheEnvironmentLockOnTheDisposableLibrary(disposable_library) => {
            let _the_library_of_this_process =
                disposable_library.make_it_the_library_of_this_process();
            let _one_test_at_a_time = singlefs_verification::crash_amplification::one_test_changes_the_environment_at_a_time();
            start_the_engine()
        }
    };
    let report = run_crash_injection_campaign_judging_the_given_states_with(campaign, &engine);
    (report, engine.finish())
}

/// 崩溃注入这一批：给定状态表经崩溃点重放的引擎判（GPU 判不了在这里红）。引擎那一趟被叫停、或两台分片而别的机器的表
/// 服务那一份里还没有时，这一批的报告不作数：打「本次未跑」那一行（`test_name` 是用例名）、交 `None`，调用方不判。
fn campaign_judging_the_given_states_through_the_crash_point_replay_entry(
    campaign: &CrashInjectionCampaign,
    test_name: &str,
) -> Option<CrashInjectionReport> {
    let (report, run) = campaign_through_the_given_states_engine_started(
        campaign,
        FaultPutOnTheGpuSide::NoFault,
        EngineStart::HoldingTheEnvironmentLockOnTheOneLibrary,
    );
    assert_eq!(
        run.red_items(),
        Vec::<String>::new(),
        "给定状态表那一路判红（GPU 判不了）；{}",
        how_to_replay(&report)
    );
    report_of_a_whole_run_or_print_why_not(report, &run, test_name)
}

/// 崩溃注入这一批：不设变量是默认批（[`default_batch_campaign`]：这个测试周期的种子基起 24 段、每段 24 步、每段摆 4 个崩溃状态，
/// 照默认批那一组断言判）；设了 `SINGLEFS_CRASH_INJECTION_*` 的照变量跑（[`crash_injection_campaign_from`]：变量与判红时打的重放行同一张表，
/// 没设的那几项取默认批的，照哪一组断言判也是变量之一）。每个崩溃状态重建镜像、跑恢复、跑池级 checker、
/// 跑记录核对器，并问模型「恢复到的这一版允不允许」；「已知红」清单里的形态照记不停，清单外的一条都不许有。
/// 默认批那一组核三截每一截在每个崩溃状态上都真的跑了（问模型、池级 checker、记录核对器，可写挂载与之后的池上 checker、记录核对器；
/// 二次崩溃摆出来过、每个都判过），再核各条路径真的跑到了：四种落点都扣下过、段内摆出过洞、起点那一段摆到过、截在发布中间过、
/// 截回到最后一版之前过、恢复三种结局见过两种（失败一次都不许有）、journal 记录真的施加过、模型真的比过内容。
#[test]
fn crash_injection_recovers_only_into_versions_the_model_committed() {
    let started = std::time::Instant::now();
    let Some(report) = campaign_judging_the_given_states_through_the_crash_point_replay_entry(
        &crash_injection_campaign_from(&process_environment_variable),
        "crash_injection_recovers_only_into_versions_the_model_committed",
    ) else {
        return;
    };
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    judge_crash_injection_report(&report);
}

/// 一批崩溃注入经引擎判（表与结论进这一次自己建的一次性库，[`disposable_library_of_the_campaign`]：临时 prune-server，不连真服务，
/// 同一个进程里并行的别的用例各连各的；配了 GPU 且编进了 `gpu` 特性上卡、否则 CPU），只要报告、不看引擎那一趟的账。
fn run_through_the_engine(campaign: &CrashInjectionCampaign) -> CrashInjectionReport {
    let disposable_library = disposable_library_of_the_campaign("run-through-the-engine");
    campaign_through_the_given_states_engine_started(
        campaign,
        FaultPutOnTheGpuSide::NoFault,
        EngineStart::HoldingTheEnvironmentLockOnTheDisposableLibrary(&disposable_library),
    )
    .0
}

/// 起在自己那一份一次性库上的引擎：引擎连着的是这一份的临时 prune-server，一次性库要比引擎活得长。
/// 字段按声明次序丢：先丢引擎（放掉连接），再丢一次性库（停临时服务、删目录）。
struct GivenStatesEngineOnItsOwnDisposableLibrary {
    engine: GivenStatesEngine,
    _disposable_library: singlefs_verification::crash_library::DisposableLibrary,
}

impl GivenStatesEngineOnItsOwnDisposableLibrary {
    fn engine(&self) -> &GivenStatesEngine {
        &self.engine
    }
}

/// 靶向用例（手摆的一段历史、一个崩溃状态）的给定状态表交引擎判：项名与设点文件同这份文件，种子与规模写用例给的那一句。
/// 表与结论进这一次自己建的一次性库（[`disposable_library_of_the_campaign`]），同一个进程里并行的别的用例各连各的。
fn given_states_engine_for(seeds_and_scale: &str) -> GivenStatesEngineOnItsOwnDisposableLibrary {
    let disposable_library = disposable_library_of_the_campaign(seeds_and_scale);
    // 先设一次性库、再拿改环境的锁（全仓用例都是这个次序；调这一函数的用例都不拿守卫）：引擎起步时开的是这一份，起完两把都放
    let the_library_of_this_process = disposable_library.make_it_the_library_of_this_process();
    let one_test_at_a_time =
        singlefs_verification::crash_amplification::one_test_changes_the_environment_at_a_time();
    let engine = given_states_engine_under_the_callers_environment_guard(seeds_and_scale);
    drop(one_test_at_a_time);
    drop(the_library_of_this_process);
    GivenStatesEngineOnItsOwnDisposableLibrary {
        engine,
        _disposable_library: disposable_library,
    }
}

/// 同 [`given_states_engine_for`]，调用方自己拿着环境守卫（[`CrashAmplificationEnvironment`]，同一把锁、不可重入）：照守卫里设的环境起。
fn given_states_engine_under_the_callers_environment_guard(
    seeds_and_scale: &str,
) -> GivenStatesEngine {
    GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!("{}::{}", CRASH_POINT_REPLAY_ITEM, seeds_and_scale),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
}

/// 经引擎跑一批的那几条用例用的一次性库（`crash_library::DisposableLibrary`：临时 prune-server 里这条用例独占的一段名字空间，起跑是空的，
/// 全仓那一份一行不写；临时 prune-server 由它自己在这个进程里起，起不来当场失败），它那份一次性临时目录里放一份 CPU 配置。这几条钉的是空库起跑的账。
fn disposable_library_with_a_configuration_on_the_cpu(
    tag: &str,
) -> (
    singlefs_verification::crash_library::DisposableLibrary,
    std::path::PathBuf,
) {
    let directory = disposable_library_of_the_campaign(tag);
    let configuration = directory.join("multi-host-on-the-cpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=0\n").expect("写得了配置");
    (directory, configuration)
}

/// 一份一次性库（临时 prune-server 里以 `tag` 命名的一段名字空间，起跑是空的）；临时 prune-server 由它自己在这个进程里起，起不来当场失败。
fn disposable_library_of_the_campaign(
    tag: &str,
) -> singlefs_verification::crash_library::DisposableLibrary {
    // 一次性库留着：崩溃注入的小批与靶向历史（几个到几十个崩溃状态）钉空库起跑的账、两份领表、叫停与 GPU 一侧塞故障的对拍；整条按测试函数键复用
    singlefs_verification::crash_library::DisposableLibrary::new(&format!(
        "crash-injection-given-states-{tag}"
    ))
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// 同 [`given_states_engine_for`]，引擎起在一份不开 GPU 的配置上（[`disposable_library_with_a_configuration_on_the_cpu`] 写的 `ENABLE_GPU=0`，
/// 环境守卫里清掉一趟读的变量、只设多机配置：不分片、不分段，起完还原），给定状态表只由 CPU 判器判
/// （`crash_given_states::judge_a_given_state_on_the_cpu`：两遍恢复、池级 checker、记录核对器）。
/// 为什么另起这一种：靶向用例经 [`given_states_engine_for`] 照本机配置判，配了 GPU 只由 GPU 判器判，CPU 判器一行都不跑，
/// `crates/mutations.tsv` 里改坏它的那几行在开 GPU 的门禁上空转。
// gpu-exempt: cpu-judge-mutation-rows
fn given_states_engine_on_a_configuration_without_the_gpu(
    seeds_and_scale: &str,
) -> GivenStatesEngineOnItsOwnDisposableLibrary {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    let (disposable_library, configuration) =
        disposable_library_with_a_configuration_on_the_cpu(seeds_and_scale);
    // 先设一次性库、再拿改环境的锁（全仓用例都是这个次序）：引擎起步时开的是这一份、读的是这一份配置，起完先还原环境、再放库
    let the_library_of_this_process = disposable_library.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let engine = given_states_engine_under_the_callers_environment_guard(seeds_and_scale);
    drop(environment);
    drop(the_library_of_this_process);
    GivenStatesEngineOnItsOwnDisposableLibrary {
        engine,
        _disposable_library: disposable_library,
    }
}

/// 收尾：关库、核这一趟是 CPU 判器判的、至少现判了一张表；先丢引擎、再丢一次性库。
fn finish_the_engine_asserting_it_judged_on_the_cpu(
    engine_on_its_own_library: GivenStatesEngineOnItsOwnDisposableLibrary,
) {
    use singlefs_verification::crash_amplification::JudgedBy;
    let GivenStatesEngineOnItsOwnDisposableLibrary {
        engine,
        _disposable_library,
    } = engine_on_its_own_library;
    let run = engine.finish();
    assert_eq!(
        (run.judged_by, run.tally.tables_judged_here > 0),
        (JudgedBy::CpuJudge, true),
        "不开 GPU 的配置上只由 CPU 判器判，这一趟至少现判了一张表：{:?}",
        run.tally
    );
}

/// 引擎这一趟作不作数：被叫停（崩溃注入不再领新的一片）、两台分片而有表归别的机器判、服务那一份里又还没有，都打「本次未跑」那一行
/// （`singlefs_verification::gpu_memory_ledger::gpu_not_run_line` 的记号，入口照它记本次未跑）、交 `None`；整批都判了交回报告。
fn report_of_a_whole_run_or_print_why_not(
    report: CrashInjectionReport,
    run: &GivenStatesRun,
    test_name: &str,
) -> Option<CrashInjectionReport> {
    use singlefs_verification::gpu_memory_ledger::gpu_not_run_line;
    if run.stopped_early || report.histories_left_by_the_stop_request > 0 {
        println!(
            "{}",
            gpu_not_run_line(
                test_name,
                &format!(
                    "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE 指的文件在）：{} 段历史没领，判过的表在库里，下一趟接着判",
                    report.histories_left_by_the_stop_request
                )
            )
        );
        return None;
    }
    if run.recorded_only() {
        println!(
            "{}",
            gpu_not_run_line(
                test_name,
                &format!(
                    "只跑了录入段（SINGLEFS_CRASH_AMPLIFICATION_STAGE=record）：{} 张给定状态表写进库、没判，判段那一趟从库里读回判",
                    run.tally.tables_recorded_only
                )
            )
        );
        return None;
    }
    if run.left_tables_to_the_other_hosts() {
        println!(
            "{}",
            gpu_not_run_line(
                test_name,
                &format!(
                    "两台分片：这一份是 {}/{}，{} 张表归别的机器判、服务那一份里还没有（那一台判完、写进服务之后再跑一趟才作数）",
                    run.share.index, run.share.count, run.tally.tables_left_to_the_other_hosts
                )
            )
        );
        return None;
    }
    Some(report)
}

/// 一趟交引擎判的给定状态表里交回占位结论的张数（[`GivenStatesConclusionsSource`] 的两种占位）：两台分片、表归别的机器判而服务那一份里又还没有的，
/// 与这一趟只跑录入段、表写进库没判的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlaceholderTablesOfTheRun {
    left_to_other_hosts: u64,
    recorded_only: u64,
}

impl PlaceholderTablesOfTheRun {
    /// 崩溃注入一段历史或一批交回的计数里记下的那几张（崩溃注入照每张表结论的来源数）。
    fn of_the_tally(tally: &CrashInjectionTally) -> Self {
        Self {
            left_to_other_hosts: tally.given_states_tables_left_to_other_hosts,
            recorded_only: tally.given_states_tables_recorded_only,
        }
    }

    /// 用例自己交引擎判的一张表。
    fn of_one_table(source: GivenStatesConclusionsSource) -> Self {
        match source {
            GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => Self {
                left_to_other_hosts: 0,
                recorded_only: 0,
            },
            GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost => Self {
                left_to_other_hosts: 1,
                recorded_only: 0,
            },
            GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => Self {
                left_to_other_hosts: 0,
                recorded_only: 1,
            },
        }
    }
}

/// 靶向用例经 [`given_states_engine_for`]、小批经 [`run_through_the_engine`] 照调用方的分片与选段起引擎：崩溃点重放入口两台分片跑时
/// （本机 `SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2`、第二台 `1/2`，两台都直接写服务那一份）归别的机器判、服务那一份里还没有的表、只跑录入段时的每张表，引擎交的是占位结论
/// （判到位全 0、记录核对器两位都不红），阳性对照在它上面红，计数为 0 与「没有新发现」的断言在它上面空过。有一张占位就交这一趟为什么不作数
/// （[`report_of_a_whole_run_or_print_why_not`] 那一类「本次未跑」的原因），调用方不判，判归两份都写进服务之后不分片再跑的那一趟（或判段那一趟）；一张都没有交 None，调用方照判。
/// 靶向用例与小批的引擎起在各自那一份一次性库上：别的机器判的表、录入段那一趟写的表都不在这一份里，分片或分段跑时这几条总是报本次未跑，不分片、不分段跑才判。
fn why_the_placeholder_conclusions_leave_this_run_unjudged(
    placeholders: PlaceholderTablesOfTheRun,
) -> Option<String> {
    if placeholders.left_to_other_hosts > 0 {
        return Some(format!(
            "两台分片：{} 张给定状态表归别的机器判、服务那一份里还没有，引擎交的是占位结论（那一台判完、写进服务之后再跑一趟才作数）",
            placeholders.left_to_other_hosts
        ));
    }
    if placeholders.recorded_only > 0 {
        return Some(format!(
            "只跑了录入段（SINGLEFS_CRASH_AMPLIFICATION_STAGE=record）：{} 张给定状态表写进库、没判，引擎交的是占位结论（判段那一趟从库里读回判）",
            placeholders.recorded_only
        ));
    }
    None
}

/// 同 [`why_the_placeholder_conclusions_leave_this_run_unjudged`]，有原因时照 [`report_of_a_whole_run_or_print_why_not`] 那一类打「本次未跑」那一行
/// （`test_name` 是用例名）、交 true，调用方直接返回；没有交 false，调用方照判。
fn printed_not_run_because_the_engine_gave_placeholder_conclusions(
    placeholders: PlaceholderTablesOfTheRun,
    test_name: &str,
) -> bool {
    use singlefs_verification::gpu_memory_ledger::gpu_not_run_line;
    match why_the_placeholder_conclusions_leave_this_run_unjudged(placeholders) {
        Some(reason) => {
            println!("{}", gpu_not_run_line(test_name, &reason));
            true
        }
        None => false,
    }
}

/// 靶向状态这一趟判没判：判了（断言都过了），或引擎交了占位结论、没判（带上为什么，打不打「本次未跑」那一行由调用方定：
/// 用例自己打；两台分片那条用例把两份都跑一遍，不打，免得整项被记成本次未跑）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum TargetedCaseOutcome {
    Judged,
    NotRunOnPlaceholderConclusions { reason: String },
}

/// 靶向用例照 [`TargetedCaseOutcome`] 收尾：没判的打「本次未跑」那一行。
fn print_not_run_when_the_targeted_case_was_not_judged(
    outcome: TargetedCaseOutcome,
    test_name: &str,
) {
    use singlefs_verification::gpu_memory_ledger::gpu_not_run_line;
    match outcome {
        TargetedCaseOutcome::Judged => {}
        TargetedCaseOutcome::NotRunOnPlaceholderConclusions { reason } => {
            println!("{}", gpu_not_run_line(test_name, &reason));
        }
    }
}

/// 默认批的规模：段数、每段步数与每段摆几个崩溃状态。规模与种子基都写死（种子基是这个测试周期的常量）。
/// 每个崩溃状态上起可写挂载、发一次布、摆三个二次崩溃之后（代码审阅第 2 条），debug 下单跑一百来秒；门禁 crash-point-replay
/// 照 `crash-case:crash-injection-campaign` 跑它。头几段里有种子基 + 2 那一段：调查报告
/// `research/prompts/m2-investigate-crash-injection-reds-report.md` 第 1 条那一形（恢复落到由 journal 记录重建的一版、它的根槽写在更晚的段）就摆在那一段上。
const DEFAULT_BATCH_SEEDS: u64 = 24;

const DEFAULT_BATCH_OPERATIONS_PER_HISTORY: usize = 24;

const DEFAULT_BATCH_DRAW: CrashPointDraw = CrashPointDraw::Sampled {
    crash_points_per_history: 4,
};

/// 崩溃注入这一段历史本身怎么跑：不跑每一步的池级 checker，两块 4 GiB 的盘。
/// 活盘面上每一步的 checker 由随机历史那五段（门禁 74 号）罩着，这一段的预算全给崩溃状态。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 报告直接写进进程的标准输出，不经 libtest 的捕获：通过时计数照样出现在 `check.sh` 的输出里
/// （`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

/// 判红时先把种子基说清楚：同一个构建加同一个种子基重放得出来，报告里不带它就没法复现。
/// 重放行由报告自己拼（`CrashInjectionReport::replay_line`：种子区间、崩溃状态怎么摆、照哪一组断言判、步数、比重、checker、盘宽、空间准入），
/// 崩溃注入那条入口照同一张变量表读回（`crash_injection_campaign_from_environment`），照同一组断言判（[`judge_crash_injection_report`]）。
fn how_to_replay(report: &CrashInjectionReport) -> String {
    format!("重放：{}", report.replay_line())
}

/// 一次根槽写写出的根身份 (实例代号, checkpoint_txg)；别的写是 None。
fn root_identity_of_a_root_slot_write(
    write: &RetainedWrite,
) -> Option<(InstanceGeneration, CheckpointTxg)> {
    match write.kind {
        StepKind::RootRecordFua => Some(root_identity_written_by(
            write.bytes().expect("根槽写带着字节"),
        )),
        StepKind::UnitWrite
        | StepKind::JournalRecord
        | StepKind::ZeroFill
        | StepKind::Discard
        | StepKind::SystemConfigurationSlot
        | StepKind::Barrier => None,
    }
}

/// C554 乙之后崩溃恢复抛弃根那一步的结局：可写挂载读阶段判据 N-配置 为真、重读一次仍为真，拒可写（`history.rs` 的
/// `apply_crash_recovery_abandoning_the_newest_root`）。
const CRASH_RECOVERY_REFUSED_BY_THE_WITNESS: &str =
    "MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)";

/// `history` 在崩溃注入那一种执行下，每一步「崩溃恢复抛弃根」的结局，按次序。
fn outcomes_of_the_crash_recovery_steps(history: &GeneratedHistory) -> Vec<StepOutcome> {
    let run = execute_history_with(
        history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    history
        .operations
        .iter()
        .zip(run.outcomes)
        .filter(|(operation, _)| {
            matches!(
                operation,
                HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
            )
        })
        .map(|(_, outcome)| outcome)
        .collect()
}

/// 记录核对器两条判据的靶向用例（靶向用例 + 变异行；
/// 改法甲、丙）用的那段历史：起点发完第一个文件 (1, 3)，再覆盖写两次 (1, 4)、(1, 5)。
/// 两块 4 GiB 的盘上这段历史的录制流切成 26 段（mkfs 作废那道屏障（C656）把 mkfs 那一段切成 2 + 12、之后的段号再加 1；取号写两轮（D22（单元原子性怎么合成） 已定项 8）
/// 第一次可写挂载的取号多一个两写的系统配置槽段，之后的段号都加 1）：第 14 段起是 (1, 3) 的单元写
/// （段内第 0、1 个是它的数据单元在两块盘上的两份），第 23 段是 (1, 5) 的两份 journal 记录，第 24 段是 (1, 5) 的根槽写；
/// 从第 14 段的第一个写数到 (1, 4) 的根槽写共 56 个写。两条用例都先核这几处，段形变了先在那里红，不悄悄判别的状态。
fn history_of_a_first_file_and_two_overwrites() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![overwrite_choosing(5), overwrite_choosing(7)],
    }
}

/// 一次覆盖写：内容落在一个数据单元里，长度与字节都随 `selector` 变（每一次都换下上一版的数据单元）。
fn overwrite_choosing(selector: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector },
        fill_seed: selector,
    })
}

/// 这段历史在崩溃注入那一种执行下的写表与切段（与 `inject_crashes_into_history` 自己切的同一份：同一条执行、同一份几何）。
fn writes_and_segments_of(history: &GeneratedHistory) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史跑完");
    writes_and_segments(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    )
}

/// 写一份开 GPU 的多机配置，只写开关：卡是调用方给这个进程的那一份、在环境守卫里交给引擎
/// （`common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run`，那份文件头写了为什么不抄进配置）。
#[cfg(feature = "gpu")]
fn write_a_configuration_with_the_gpu_enabled(configuration: &std::path::Path) {
    std::fs::write(configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
}

/// 用例在环境守卫里起开 GPU 的引擎时设的变量：多机配置指到用例写的那一份（[`write_a_configuration_with_the_gpu_enabled`]）。
/// 卡不经这里交：用例在环境守卫里接着调 `common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run`（服务交了卡串的设卡串，
/// 清场之下是清场之后那一份；没交卡串的不设、不借卡），先于守卫丢。
/// 这一台是 HOSTS 里哪一项不经环境交：引擎与卡池都照调用方那份多机配置上判法认出的那一项
/// （`gpu_card_configuration::multi_host_addresses_of_the_caller`），派到哪一台都认得出自己。
#[cfg(feature = "gpu")]
fn settings_of_an_engine_on_the_gpu(
    configuration: &std::path::Path,
) -> [(&'static str, &std::ffi::OsStr); 1] {
    use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
    [(
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration.as_os_str(),
    )]
}
