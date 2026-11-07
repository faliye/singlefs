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
mod common_gpu_cards_of_the_caller;

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

/// 种子基是一个测试周期的常量：开一个周期抽一次，抽完在这个周期之内写死。周期从「开一个新里程碑」与
/// 「显式要求从零开始测试 / 重建测试」里先到的那一件开始。
///
/// 这一条钉两样：
/// 1. 常量就是当前这个周期（里程碑「覆盖写、释放、回退与复用」）抽出来的那个数。改成别的数 = 把这个周期的实验数据整批换掉，
///    而 `crates/mutations.tsv` 里名字以「增补 3 第 2 件」起头的那些「这条变异必须红」与里程碑那几条验收，量的都是这一批历史；
/// 2. 从它起跑出来的报告把它带了回来（报告第一行与重放那一行都带着它），判红了才重放得出来。
///
/// 随机历史那个二进制的五段用的是同一个常量——两边同基，那一半由它自己那条用例钉
/// （`random_histories.rs` 的 `the_five_samplings_start_from_the_test_cycle_seed_base`）。
#[test]
fn the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle() {
    assert_eq!(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        7_463_871_032_432_355_113,
        "种子基不是 2026-09-20 抽的那个数：它永久固定，换了就让绑在这批历史上的判红、镜像与过程文件整批作废\
         （为什么固定、要多测历史怎么追加区间，写在 SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE 的文档注释里）"
    );
    let report = run_through_the_engine(&CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 2,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    });
    assert_eq!(
        report.first_seed, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        "报告没把跑的那个种子基带回来"
    );
    let replay = how_to_replay(&report);
    assert!(
        replay.contains(&SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.to_string()),
        "重放那一行里没有种子基，判红了就重放不出来：{replay}"
    );
    assert!(
        report.render().contains("这个测试周期写死的"),
        "报告第一行没说清种子基是这个测试周期写死的：\n{}",
        report.render()
    );
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

/// 照这一批的那一组断言判（[`CrashInjectionAssertions`]）：出红的那条用例与照它的重放行跑的崩溃注入入口都经这一个函数，
/// 所以照抄重放行跑，红在同一条断言上（代码三方 test-script-align-code-r1 判决 S2）。
fn judge_crash_injection_report(report: &CrashInjectionReport) {
    assert!(
        report.new_findings.is_empty(),
        "崩溃状态上「已知红」清单外的失败；{}\n{}",
        how_to_replay(report),
        report.render()
    );
    match report.assertions {
        CrashInjectionAssertions::NewFindingsOnly => {
            assert_at_least_one_crash_state_was_judged(report);
        }
        CrashInjectionAssertions::DefaultBatch => {
            assert_each_stage_ran_on_every_crash_state(report);
            assert_every_crash_injection_path_was_exercised(report);
        }
        CrashInjectionAssertions::RollbackSampling => {
            assert_the_rollback_sampling_placed_crash_states_and_recovered_from_each(report);
        }
    }
}

/// 只判清单外失败的那一组也要判过至少一个崩溃状态：一个都没判过的一批没有清单外失败，只是因为什么都没判
/// （上游 `show-me-test.md`「扫到 0 项也不是通过」）。别的两组各自要的计数都不少于段数，已经含着这一条。
fn assert_at_least_one_crash_state_was_judged(report: &CrashInjectionReport) {
    assert!(
        report.tally.crash_points > 0,
        "这一批一个崩溃状态都没判过，扫到 0 项不算通过；{}",
        how_to_replay(report)
    );
}

/// 一段都不跑的一批照只判清单外失败的那一组判：判红，不因为没有清单外失败就判成绿。
// crash-case-check:not-a-crash-case 一段都不跑的一批判红：造的计数、不跑崩溃状态，核只判清单外失败那一组断言的判别力
#[test]
#[should_panic(expected = "这一批一个崩溃状态都没判过")]
fn a_batch_that_judged_no_crash_state_is_red_under_the_new_findings_only_assertions() {
    let report = run_through_the_engine(&CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 0,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    });
    judge_crash_injection_report(&report);
}

/// 默认批那一组的结构计数：三截每一截在每个崩溃状态上都真的跑了，二次崩溃摆出来过、每个都判过，恢复没失败过。
/// 先于各条路径那一半判：结构计数缺了，红在这里点名的那一截上。
fn assert_each_stage_ran_on_every_crash_state(report: &CrashInjectionReport) {
    let replay = how_to_replay(report);
    let tally = &report.tally;
    // 每段历史都摆出过崩溃状态（总数够放得过「几段一个都没有、别的段多摆」；
    // 为什么每段都摆得出见 [`assert_every_crash_injection_path_was_exercised`]）。
    assert_eq!(
        tally.histories_without_any_crash_point, 0,
        "有历史段一个崩溃状态都没摆出来；{replay}"
    );
    assert!(
        tally.crash_points >= report.seed_count,
        "平均每段连一个崩溃状态都没摆出来：{}；{replay}",
        tally.crash_points
    );
    for (stage, runs) in [
        ("问模型", tally.model_judgements),
        ("池级 checker", tally.checker_runs),
        ("记录核对器", tally.record_checks),
        (
            "崩溃后镜像上的可写挂载",
            tally.writable_mounts_after_the_crash,
        ),
        (
            "可写挂载之后的池上 checker",
            tally.checker_runs_after_the_writable_mount,
        ),
        (
            "可写挂载之后的池上记录核对器",
            tally.record_checks_after_the_writable_mount,
        ),
    ] {
        assert_eq!(
            runs, tally.crash_points,
            "每个崩溃状态都要跑过{stage}；{replay}"
        );
    }
    assert!(
        tally.second_crash_points >= 1,
        "挂载途中一个二次崩溃都没摆出来；{replay}"
    );
    for (stage, runs) in [
        ("问模型", tally.second_crash_model_judgements),
        ("池级 checker", tally.second_crash_checker_runs),
        ("记录核对器", tally.second_crash_record_checks),
    ] {
        assert_eq!(
            runs, tally.second_crash_points,
            "每个二次崩溃都要跑过{stage}；{replay}"
        );
    }
    assert_eq!(tally.recoveries_failed, 0, "崩溃之后恢复失败过；{replay}");
}

/// 默认批：这个测试周期的种子基起 [`DEFAULT_BATCH_SEEDS`] 段、每段 [`DEFAULT_BATCH_OPERATIONS_PER_HISTORY`] 步、[`DEFAULT_BATCH_DRAW`]、
/// 各类操作都抽。跑完把镜像目录删掉：临时目录里不留东西（门禁 77 号判跑完机器干不干净）。
fn default_batch_campaign(worker_threads: CrashInjectionWorkerThreads) -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: DEFAULT_BATCH_SEEDS,
        operations_per_history: DEFAULT_BATCH_OPERATIONS_PER_HISTORY,
        draw: DEFAULT_BATCH_DRAW,
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads,
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::DefaultBatch,
    }
}

/// 默认批那一组的覆盖：各条路径都跑到了（四种落点都扣下过、段内摆出过洞、起点那一段摆到过、截在发布中间过……）。
fn assert_every_crash_injection_path_was_exercised(report: &CrashInjectionReport) {
    let tally = &report.tally;
    let replay = how_to_replay(report);
    // 总数够不说明每段都摆到了：`crash_points >= seed_count` 放得过「几段一个都没有、别的段多摆」。
    // 从第一个文件起的历史，起点那一段（取号、暖机、
    // 第一个文件）就有写、都是候选段；从 mkfs 起的历史要等第一次做成的可写挂载，关着会话时只抽可写挂载与冷启动两种，
    // 一次可写挂载都没做成就收尾的（新发现或已知红收尾在那之前）这一条照判红。推的，没在默认批上跑过。
    assert_eq!(
        tally.histories_without_any_crash_point, 0,
        "有历史段一个崩溃状态都没摆出来；{replay}"
    );
    assert!(
        tally.crash_points >= report.seed_count,
        "平均每段连一个崩溃状态都没摆出来：{}；{replay}",
        tally.crash_points
    );
    assert_eq!(
        tally.model_judgements, tally.crash_points,
        "每个崩溃状态都要问过模型；{replay}"
    );
    assert_eq!(
        tally.checker_runs, tally.crash_points,
        "每个崩溃状态都要跑过池级 checker；{replay}"
    );
    assert_eq!(
        tally.record_checks, tally.crash_points,
        "每个崩溃状态都要跑过记录核对器；{replay}"
    );
    for kind in [
        StepKind::UnitWrite,
        StepKind::JournalRecord,
        StepKind::RootRecordFua,
        StepKind::SystemConfigurationSlot,
    ] {
        assert!(
            tally
                .crash_points_withholding_write_kind
                .get(kind.name())
                .copied()
                .unwrap_or(0)
                >= 1,
            "一次都没扣下 {}：{:?}；{replay}",
            kind.name(),
            tally.crash_points_withholding_write_kind
        );
    }
    for kind in [
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::CloseAndMountWritable,
    ] {
        assert!(
            tally
                .crash_points_by_operation_kind
                .get(&kind)
                .copied()
                .unwrap_or(0)
                >= 1,
            "一个崩溃状态都没落在 {kind:?} 里：{:?}；{replay}",
            tally.crash_points_by_operation_kind
        );
    }
    assert!(
        tally.crash_points_withholding_a_write_before_a_persisted_one >= 1,
        "一个段内有洞的状态都没摆出来（等于还是只截前缀）；{replay}"
    );
    assert!(
        tally.crash_points_inside_the_starting_point >= 1,
        "起点那一段里一个崩溃状态都没摆到（2026-09-20 才放开）；{replay}"
    );
    assert!(
        tally.crash_points_inside_an_unfinished_publish >= 1,
        "一次都没把发布截在它的根落盘之前；{replay}"
    );
    assert!(
        tally.crash_points_that_land_before_the_last_committed_version >= 1,
        "一个崩溃状态都没截回到最后一版之前（全落在流的尾巴上）；{replay}"
    );
    assert_eq!(tally.recoveries_failed, 0, "崩溃之后恢复失败过；{replay}");
    assert!(
        tally.recoveries_reading_a_file >= 1,
        "崩溃之后一次都没读回文件；{replay}"
    );
    assert!(
        tally.recoveries_without_a_file >= 1,
        "崩溃之后一次都没落在树表 0 条的一版上；{replay}"
    );
    assert!(
        tally.recoveries_that_applied_journal_records >= 1,
        "恢复一次都没施加过 journal 记录前缀（journal 在这一段里没承重）；{replay}"
    );
    assert!(
        tally.model_contents_compared >= 1,
        "模型一次都没比过崩溃之后读回的内容；{replay}"
    );
    assert!(
        tally.model_versions_without_a_file_matched >= 1,
        "模型一次都没对上「这一版树表 0 条」；{replay}"
    );
    for invariant in ["I-3.1", "I-5.4"] {
        assert!(
            tally.invariant_holds.get(invariant).copied().unwrap_or(0) >= 1,
            "崩溃状态上 checker 一次都没判过 {invariant}（全是不适用）；{replay}"
        );
    }
}

/// 多线程只影响跑得多快：同一批种子，1 个线程与 4 个线程跑出来的报告逐字相同
/// （`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」的「合并要确定」）。
/// 计数按片的次序相加、新发现按种子留第一个，所以结论与线程数、调度次序无关；进度行按到达次序打，不带判定，不在比对之内；
/// 判红的崩溃镜像每次调用写进自己新建的目录，路径不在比对之内（比的是 `render_the_conclusions`）。
/// 两遍拿的是同一个种子基（这个测试周期写死的那一个），比的就是同一批历史上的同一批崩溃状态。
#[test]
fn one_thread_and_four_threads_render_the_same_report() {
    let first_seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
    let campaign = |worker_threads: usize| {
        run_through_the_engine(&CrashInjectionCampaign {
            first_seed,
            seed_count: 8,
            operations_per_history: 16,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 3,
            },
            weights: GenerationWeights::BROAD,
            execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(worker_threads),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        })
    };
    let report_on_one_thread = campaign(1);
    let report_on_four_threads = campaign(4);
    for report in [&report_on_one_thread, &report_on_four_threads] {
        if printed_not_run_because_the_engine_gave_placeholder_conclusions(
            PlaceholderTablesOfTheRun::of_the_tally(&report.tally),
            "one_thread_and_four_threads_render_the_same_report",
        ) {
            return;
        }
    }
    let on_one_thread = report_on_one_thread.render_the_conclusions();
    let on_four_threads = report_on_four_threads.render_the_conclusions();
    assert_eq!(
        on_one_thread, on_four_threads,
        "线程数不改结论（种子基 {first_seed}）"
    );
    assert!(
        on_one_thread.contains("崩溃状态 "),
        "报告里有崩溃状态的计数：\n{on_one_thread}"
    );
}

/// 写死的一段历史：mkfs → 可写挂载 → 第一个文件 → 覆盖写 → 可写挂载 → 覆盖写；写数不超过 4 的段整段枚举（全部真子集），
/// 更长的段各抽 8 个。今天的代码上每个崩溃状态都恢复到模型提交过的某一版、池级 checker 与记录核对器一条都不红。
///
/// 判别力靠的就是这一条：段内真子集全枚举，所以「记录写持久了而根槽没持久」与「根槽持久了而它那次发布的记录一条都不在」
/// 两类状态都摆得出来——少一道屏障（`crates/mutations.tsv`「步 3：零单元发布在记录与根之间少一道屏障」）就把两段并成一段，
/// 后一类立刻摆得出，记录核对器判红。种子写死不随机：这一条判的是枚举域罩不罩得住，不是抽样运气。
// crash-case-check:not-a-crash-case 写死的一段短历史，写数不超过 4 的段由崩溃注入库自己整段枚举真子集、更长的段各抽 8 个，不调枚举一族函数、不经块判定库；随机崩溃注入的默认批登记成 crash-case:crash-injection-default-batch
#[test]
fn every_crash_state_of_a_written_out_history_recovers_into_a_committed_version() {
    let content = |selector: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector },
            fill_seed: selector,
        })
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 3 },
                fill_seed: 1,
            }),
            content(5),
            HistoryOperation::PublishWithoutUnits,
            HistoryOperation::CloseAndMountWritable,
            content(7),
        ],
    };
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 4,
            sampled_in_longer_segments: 8,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：写死的一段历史，段内真子集全枚举 ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "every_crash_state_of_a_written_out_history_recovers_into_a_committed_version",
    ) {
        return;
    }
    assert!(
        injection.new_findings.is_empty(),
        "写死的这段历史上崩溃状态判红：{:#?}",
        injection.new_findings
    );
    // 代码审阅第 19 条之后取号写之前多一道屏障：第二次可写挂载那一处，上一次发布末尾的轮换与取号那 4 写一段拆成 2、2，
    // 真子集 15 → 3 + 3，这段历史摆得出的崩溃状态 104 → 95（屏障只拆段、不添状态）。
    assert!(
        injection.tally.crash_points >= 95,
        "这段历史摆得出的崩溃状态不少于 95 个：{}",
        injection.tally.crash_points
    );
    assert_eq!(
        injection.tally.recoveries_failed, 0,
        "每个崩溃状态都恢复得出来"
    );
    assert!(
        injection.tally.crash_points_inside_an_unfinished_publish >= 4,
        "每次发布的单元写与记录写都截在它的根落盘之前：{}",
        injection.tally.crash_points_inside_an_unfinished_publish
    );
    assert!(
        injection
            .tally
            .crash_points_withholding_write_kind
            .get(StepKind::JournalRecord.name())
            .copied()
            .unwrap_or(0)
            >= 1,
        "一次都没把 journal 记录写扣下（少一道屏障那条变异就判不出来）：{:?}",
        injection.tally.crash_points_withholding_write_kind
    );
    assert_eq!(
        injection.tally.record_root_without_record, 0,
        "今天的代码上没有「根在而它那次发布的记录一条都不在」的状态"
    );
    assert_eq!(
        injection.tally.crash_states_ending_new_finding, 0,
        "一个崩溃状态都不许是新发现"
    );
}

/// 「已知红」清单那一条（增补 2 收口表第 43 行）在崩溃状态上的复现那一段历史（与随机历史那边
/// `the_history_that_raised_the_floor_into_a_rollback_gap_completes_under_the_forward_rollback` 逐项相同），两次回退换成挂着时的向前回退：
/// 没有回退留下的空档，抬 F 那几步之后的崩溃状态上一条清单外的失败都不许有；回退与抬 F 的写上都摆得到崩溃状态。
/// 清单那一形（F 落在被抛弃实例留下的空档里）今天只剩崩溃恢复抛弃的时间线造得出，随机历史与崩溃注入都没有这一种操作，
/// 它在崩溃状态上认不认得出来（代码三方 m2-supp3-item3-code-r1 判决 K6 的假阳那一半）这一段不再罩，交主 agent（实三报告）。
#[test]
fn crash_states_of_the_history_that_raised_the_floor_into_a_rollback_gap_are_clean_under_the_forward_rollback(
) {
    let empty = ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    };
    let history = GeneratedHistory {
        seed: HistorySeed(80),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 0,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::PublishOverwrite(empty),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 8,
            }),
        ],
    };
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 2,
            sampled_in_longer_segments: 1,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：原先抬 F 落进回退空档那一段历史（向前回退） ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "crash_states_of_the_history_that_raised_the_floor_into_a_rollback_gap_are_clean_under_the_forward_rollback",
    ) {
        return;
    }
    assert!(
        injection.new_findings.is_empty(),
        "一条清单外的失败都不许有：{:#?}",
        injection.new_findings
    );
    // 「回退与抬 F 的写上也摆得到崩溃状态」这一条钉在这里，不钉在随机种子那一段上：
    // 这段历史是写死的，回退两次、抬 F 一次都在里面，段内真子集又是全枚举的，所以这个结论不靠抽样运气。
    for kind in [
        HistoryOperationKind::RollBackWhileMounted,
        HistoryOperationKind::RaiseRollbackFloor,
    ] {
        assert!(
            injection
                .tally
                .crash_points_by_operation_kind
                .get(&kind)
                .copied()
                .unwrap_or(0)
                >= 1,
            "一个崩溃状态都没落在 {kind:?} 里：{:?}",
            injection.tally.crash_points_by_operation_kind
        );
    }
}

/// 各类操作的写上摆到几个崩溃状态（共八类操作各钉一条；
/// 含崩溃恢复抛弃根）。覆盖写与可写挂载由默认批钉、回退与抬 F 由上一条写死历史钉，剩下四类钉在这一段写死的历史上：
/// mkfs → 可写挂载 → 零单元发布（树表 0 条的一版上才调得成）→ 冷启动 → 可写挂载 → 第一个文件 → 崩溃恢复抛弃根（C554 乙拒）→ 可写挂载。
/// 写数不超过 2 的段整段枚举（2^n − 1 个真子集），更长的段各抽 1 个，所以每一类摆到几个是定数，不靠抽样运气：
/// 零单元发布的段是 2、1、2 ⇒ 3 + 1 + 3 = 7；第一个文件的段是 24、2、1、2 ⇒ 1 + 3 + 1 + 3 = 8；
/// 冷启动只读盘、一个写都不发，崩溃恢复抛弃根在取号之前被拒、也不发写 ⇒ 两类都是 0（崩溃状态按第一个被扣下的写落在哪一步归类，
/// 不发写的那一步归不到）。段序列是 harness 侧按同一段历史、同一个执行（`execute_history_with` 加
/// `writes_and_segments_with_stream_indexes`）现量的。
/// 比已有的两段写死历史多罩的：树表 0 条的一版上的零单元发布、冷启动之后与崩溃恢复被拒之后的可写挂载那几段的崩溃状态。
#[test]
fn crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishWithoutUnits,
            HistoryOperation::ColdStartRecover,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 3 },
                fill_seed: 1,
            }),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperation::CloseAndMountWritable,
        ],
    };
    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
    assert_eq!(
        crash_recovery_outcomes,
        vec![StepOutcome::Refused {
            member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
        }],
        "崩溃恢复抛弃根那一步被 C554 乙拒：它归到 0 个崩溃状态说的是「被拒的那一步不发写」"
    );
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 2,
            sampled_in_longer_segments: 1,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：零单元发布、第一个文件、冷启动与崩溃恢复抛弃根各摆几个崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "crash_states_land_in_zero_unit_publishes_and_first_files_but_never_in_cold_starts_or_refused_crash_recoveries",
    ) {
        return;
    }
    let crash_points_in = |kind: HistoryOperationKind| {
        injection
            .tally
            .crash_points_by_operation_kind
            .get(&kind)
            .copied()
            .unwrap_or(0)
    };
    assert_eq!(
        crash_points_in(HistoryOperationKind::ColdStartRecover),
        0,
        "冷启动一个写都不发，崩溃状态归不到它：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    assert_eq!(
        crash_points_in(HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot),
        0,
        "崩溃恢复抛弃根在取号之前被拒、一个写都不发，崩溃状态归不到它：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    assert_eq!(
        crash_points_in(HistoryOperationKind::PublishWithoutUnits),
        7,
        "零单元发布的段 2、1、2：3 + 1 + 3：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    assert_eq!(
        crash_points_in(HistoryOperationKind::PublishFirstFile),
        8,
        "第一个文件的段 24、2、1、2：1 + 3 + 1 + 3：{:?}",
        injection.tally.crash_points_by_operation_kind
    );
    // 计数先钉、判定后钉：计数只看段序列与归步，少一道屏障那类变异先红在上面的计数上（`crates/mutations.tsv`）。
    assert!(
        injection.new_findings.is_empty(),
        "一条清单外的失败都不许有：{:#?}",
        injection.new_findings
    );
}

/// 一段历史上写出的每次发布：它写出的单元写（写表下标）与它的根身份 (实例代号, checkpoint_txg)。
/// 归法与记录核对器的相同：一次根槽写之前、上一次根槽写之后的单元写都归这一次。
fn unit_writes_of_each_publish(writes: &[RetainedWrite]) -> Vec<((u32, u64), Vec<usize>)> {
    let mut publishes = Vec::new();
    let mut units = Vec::new();
    for (index, write) in writes.iter().enumerate() {
        match write.kind {
            StepKind::UnitWrite => units.push(index),
            StepKind::RootRecordFua => {
                let (instance, checkpoint_txg) =
                    root_identity_written_by(write.bytes().expect("根槽写带着字节"));
                publishes.push(((instance.0, checkpoint_txg.0), std::mem::take(&mut units)));
            }
            StepKind::JournalRecord
            | StepKind::ZeroFill
            | StepKind::Discard
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => {}
        }
    }
    publishes
}

/// 这次单元写写下的字节还在不在 `image` 上。
fn unit_write_is_still_on_disk(image: &CrashImage<'_>, write: &RetainedWrite) -> bool {
    let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
    PoolReader::read(image, write.device, write.offset, length).as_deref() == write.bytes()
}

/// 记录核对器不要求被抛弃时间线上那次发布的单元还在（实七报告 (c)，实八；复现的随机种子是崩溃注入种子基 + 2）：
/// 第一个文件（txg 3）之后覆盖写一次（txg 4），崩溃恢复抛弃它（新实例 2 写行 (1, 3, 0)），再覆盖写几次——实例 2 看不见 (1, 4)，
/// 它的数据单元那一对槽被后来的发布复用。整条流都落了盘的那个状态上，恢复落到最新那一版，它的实例表判 (1, 4) 被抛弃
/// （行 (1, 3) 且 4 > 3，D23（journal 的角色与格式） 已定项 14），(1, 4) 写出的单元不算「该在」，记录核对器不判红。
/// 判别力的两半：同一条流上把没被抛弃的发布的一个单元的两份都扣下，照判红——一次是 (1, 3)（行 (1, 3) 的边上，T = Ti 不算被抛弃），
/// 一次是最后那次覆盖写。
///
/// **C554 乙之后「崩溃恢复抛弃根」这一步造不出被抛弃的根**（改钉「这一形不可达」）：(1, 4) 是这个会话发的，
/// 系统配置见证过它；这一步让它的根槽与单元读回全 0 之后，可写挂载读阶段判据 N-配置 为真、重读一次仍为真，按 C554 乙拒
/// （`MountError::NewerStateStillUnreadableAfterOneReread`，`history.rs` 的 `apply_crash_recovery_abandoning_the_newest_root` 文档写着），
/// 不开新实例、不写抛弃行，后面三次覆盖写没有可写会话、都不适用。没有 C554 乙时这一条绿，
/// 今天的底座上把 C577 那道屏障去掉它照样造不出（探针量过，合入后验证一报告）：不可达是 C554 乙定的，不是 C577。
/// 这一条保留原来的构造（同一串步），改钉今天的样子：整条流都落了盘的那个状态上恢复落到 (1, 4)（实例 1），流里没有实例 2 的发布，
/// 记录核对器不判红；判别力那两次扣单元照旧，扣的是 (1, 3) 与流里最后一次发布 (1, 4)。把 C554 乙那一判关掉，挂载照旧做成、落到实例 2，
/// 红在「落到实例 1」那一条（`crates/mutations.tsv`）。三态（全落、扣 (1, 3) 的、扣 (1, 4) 的）一张给定状态表交崩溃点重放的引擎判
/// （两段：表先写进判定库、读回再判，配了 GPU 走 GPU）；「落到哪一版」那几条前提是实现的恢复，照旧在 CPU 上跑。
// crash-case-check:not-a-crash-case 一段写死的短历史上摆三个给定状态（全落、扣两个数据单元）作一张给定状态表交判器判，核记录核对器不要求被抛弃时间线上那次发布的单元，不调枚举一族函数
#[test]
fn units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red(
) {
    print_not_run_when_the_targeted_case_was_not_judged(
        the_whole_stream_is_clean_and_each_kept_publish_missing_a_unit_is_red_after_the_crash_recovery_abandoning_the_newest_root(
            given_states_engine_for("crash_recovery_abandoning_the_newest_root_then_three_overwrites")
                .engine(),
        ),
        "units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red",
    );
}

/// [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`] 那段历史与那三态
/// （全落、扣 (1, 3) 的数据单元、扣流里最后一次发布的数据单元）作一张给定状态表交 `judge` 判，断言照那条用例：
/// 全落那一态记录核对器一条都不红，扣单元的两态第二条判据红。引擎交了占位结论就不判、交回为什么。
fn the_whole_stream_is_clean_and_each_kept_publish_missing_a_unit_is_red_after_the_crash_recovery_abandoning_the_newest_root(
    judge: &dyn GivenStatesJudge,
) -> TargetedCaseOutcome {
    let overwrite = |fill_seed: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed,
        })
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite(1),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            overwrite(2),
            overwrite(3),
            overwrite(4),
        ],
    };
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    let publishes = unit_writes_of_each_publish(&writes);
    let units_of = |identity: (u32, u64)| -> &Vec<usize> {
        &publishes
            .iter()
            .find(|(written, _)| *written == identity)
            .unwrap_or_else(|| panic!("流里有 {identity:?} 那次发布：{publishes:?}"))
            .1
    };
    let every_write_landed = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true; writes.len()],
    };
    let report = recover(&every_write_landed, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "C554 乙之后崩溃恢复抛弃根那一步的可写挂载被拒：落到实例 1 的 (1, 4)，没有实例 2（C554 乙之前落到实例 2、比 (1, 4) 新的一版）"
    );
    assert!(
        publishes.iter().all(|((instance, _), _)| *instance <= 1),
        "流里只有 mkfs（实例 0）与实例 1 的发布、没有实例 2 的：{publishes:?}"
    );

    let last_overwrite = publishes.last().expect("流里至少一次发布").0;
    let kept_publishes = [(1, 3), last_overwrite];
    // 整条流都落了盘的那一态，再接两态：没被抛弃的那两次发布各扣下一个数据单元的两份（两块盘各一份）
    let mut persisted_sets = vec![vec![true; writes.len()]];
    for kept in kept_publishes {
        let data_unit = *units_of(kept)
            .iter()
            .find(|unit| writes[**unit].length_in_bytes() == DATA_UNIT_BYTES)
            .unwrap_or_else(|| panic!("{kept:?} 那次发布写了数据单元"));
        let mut persisted = vec![true; writes.len()];
        for unit in units_of(kept) {
            if writes[*unit].offset == writes[data_unit].offset {
                persisted[*unit] = false;
            }
        }
        let recovery_with_the_unit_withheld = recover(
            &CrashImage {
                base: &base,
                writes: &writes,
                persisted: persisted.clone(),
            },
            JournalPolicy::Consult,
        );
        assert!(
            recovery_with_the_unit_withheld
                .effective_root
                .is_some_and(|(_, txg)| txg.0 >= kept.1),
            "恢复落到的那一版不早于 {kept:?}：{:?}",
            recovery_with_the_unit_withheld.effective_root
        );
        persisted_sets.push(persisted);
    }
    // 三态一张给定状态表交崩溃点重放的引擎判（两段：表先写进判定库、读回再判，配了 GPU 走 GPU；能走两段的都走两段）
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    let conclusions = judge
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: "every-write-landed-and-a-kept-unit-withheld".to_string(),
            base: history_after_the_make_filesystem.base_pool(),
            writes: history_after_the_make_filesystem.writes(),
            segments: history_after_the_make_filesystem.segments(),
            seams: &[],
            persisted_sets: persisted_sets
                .iter()
                .map(|persisted| {
                    history_after_the_make_filesystem.persisted_set_of_the_table(persisted)
                })
                .collect(),
        })
        .unwrap_or_else(|reason| panic!("引擎判得了这张表：{reason}"));
    if let Some(reason) = why_the_placeholder_conclusions_leave_this_run_unjudged(
        PlaceholderTablesOfTheRun::of_one_table(conclusions.source),
    ) {
        return TargetedCaseOutcome::NotRunOnPlaceholderConclusions { reason };
    }
    assert_eq!(
        conclusions.states.len(),
        1 + kept_publishes.len(),
        "判器逐状态交回结论"
    );
    assert_eq!(
        conclusions.states[0].record_check(),
        RecordCheck::default(),
        "整条流都落了盘：记录核对器一条都不判红"
    );
    for (kept, conclusion) in kept_publishes.iter().zip(&conclusions.states[1..]) {
        assert!(
            conclusion.record_check().claimed_state_missing_unit,
            "{kept:?} 没被抛弃，它的数据单元两份都没落，照判红"
        );
    }
    TargetedCaseOutcome::Judged
}

/// 同 [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`]，给定状态表交
/// 起在一份不开 GPU 的配置上的引擎（[`given_states_engine_on_a_configuration_without_the_gpu`]：只由 CPU 判器判），收尾核这一趟是 CPU 判器判的。
/// 判别力（`crates/mutations.tsv`「实审 B3c-1：记录核对器在实例表读得出时把每次发布都当被抛弃」）：恢复落到的 (1, 4) 的实例表读得出，
/// 每次发布都当被抛弃，扣单元的两态不再判红，这条红。
// crash-case-check:not-a-crash-case 与 units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red 那一条同样的三个给定状态，交起在不开 GPU 的配置上的引擎（只由 CPU 判器判），核这一趟是 CPU 判器判的；不调枚举一族函数
#[test]
fn on_a_configuration_without_the_gpu_the_whole_stream_is_clean_and_each_kept_publish_missing_a_unit_is_red_after_the_crash_recovery_abandoning_the_newest_root(
) {
    let engine_on_the_cpu = given_states_engine_on_a_configuration_without_the_gpu(
        "crash_recovery_abandoning_the_newest_root_then_three_overwrites_on_the_cpu",
    );
    let outcome =
        the_whole_stream_is_clean_and_each_kept_publish_missing_a_unit_is_red_after_the_crash_recovery_abandoning_the_newest_root(
            engine_on_the_cpu.engine(),
        );
    finish_the_engine_asserting_it_judged_on_the_cpu(engine_on_the_cpu);
    assert_eq!(
        outcome,
        TargetedCaseOutcome::Judged,
        "引擎起在清掉分片与选段变量的环境里，这张表这一趟就判"
    );
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

/// 调查报告 `research/prompts/m2-investigate-crash-injection-reds-report.md` 第 1 条那一形所在的历史：种子基 + 2，
/// 默认批的步数与比重（按 [`DEFAULT_BATCH_DRAW`] 摆的崩溃状态也与默认批相同）。
fn history_with_a_crash_state_landing_on_a_version_rebuilt_from_records() -> GeneratedHistory {
    generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 2),
        DEFAULT_BATCH_OPERATIONS_PER_HISTORY,
        &GenerationWeights::BROAD,
    )
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

/// 与整条流（`write_count` 个写）逐条对应的持久集合：崩溃点所在段之前整段持久、
/// 这一段按崩溃点的子集、之后的一个都没持久——崩溃注入交给记录核对器的那一种。
fn persisted_over_the_whole_stream(
    write_count: usize,
    first_write_of_the_segment: usize,
    persisted_within_the_segment: &[bool],
) -> Vec<bool> {
    let mut persisted = vec![false; write_count];
    persisted[..first_write_of_the_segment].fill(true);
    persisted[first_write_of_the_segment
        ..first_write_of_the_segment + persisted_within_the_segment.len()]
        .copy_from_slice(persisted_within_the_segment);
    persisted
}

/// 两块空盘上叠 `writes_before_the_segment` 里在持久集合 `persisted`（整条流那一份，逐条对应）里的写。
fn pool_of_the_persisted_writes_before_the_segment(
    writes_before_the_segment: &[RetainedWrite],
    persisted: &[bool],
    device_bytes: u64,
) -> MemoryPool {
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], device_bytes);
    for (write, is_persisted) in writes_before_the_segment.iter().zip(persisted) {
        if *is_persisted {
            base.apply_writes(std::slice::from_ref(write));
        }
    }
    base
}

/// 在 `history` 的整条录制流上，按 `crash_points`（这段历史上崩溃注入摆出来的那几个）逐个重建崩溃镜像、跑恢复，
/// 挑出第一个这样的崩溃状态：恢复施加过 journal 记录、落到的那一版的根槽写不在到这一段为止的前缀里（在更晚的段），
/// 那一版的实例表读得出、抛弃了更早的一次发布，而那次发布写在某个偏移上的单元每一份都已不在盘上（被后来的发布复用盖掉）。
/// 交回落到的那一版与被它的实例表抛弃的那次发布；一个这样的崩溃状态都没有时 `None`（C554 乙之后这一形不可达，见用到它的两条用例）。
fn crash_state_landing_on_a_version_rebuilt_from_records(
    history: &GeneratedHistory,
    crash_points: &[CrashPoint],
) -> Option<((InstanceGeneration, CheckpointTxg), (u32, u64))> {
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    execute_history_with(history, execution, &stream, &mut |_| {});
    let (writes, segments) = writes_and_segments(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let publishes = unit_writes_of_each_publish(&writes);
    let device_bytes = execution.device_width.device_bytes();
    crash_points.iter().find_map(|crash_point| {
        let segment = &segments[crash_point.segment_index];
        assert_eq!(
            segment.len(),
            crash_point.persisted_within_the_segment.len(),
            "这里切出的第 {} 段与崩溃注入那一路切出的写数相同",
            crash_point.segment_index
        );
        let first_write_of_the_segment = segment[0];
        let end_of_the_segment = first_write_of_the_segment + segment.len();
        let persisted = persisted_over_the_whole_stream(
            writes.len(),
            first_write_of_the_segment,
            &crash_point.persisted_within_the_segment,
        );
        let base = pool_of_the_persisted_writes_before_the_segment(
            &writes[..first_write_of_the_segment],
            &persisted,
            device_bytes,
        );
        let image = CrashImage {
            base: &base,
            writes: &writes[first_write_of_the_segment..end_of_the_segment],
            persisted: crash_point.persisted_within_the_segment.clone(),
        };
        let report = recover(&image, JournalPolicy::Consult);
        let landed_version = report.effective_root?;
        if report.journal.prefix_applied == 0
            || writes[..end_of_the_segment]
                .iter()
                .any(|write| root_identity_of_a_root_slot_write(write) == Some(landed_version))
        {
            return None;
        }
        let root_write_of_the_landed_version = writes
            .iter()
            .rposition(|write| root_identity_of_a_root_slot_write(write) == Some(landed_version))?;
        let table = InstanceTableOfRootRecord::read(
            &image,
            writes[root_write_of_the_landed_version]
                .bytes()
                .expect("根槽写带着字节"),
        )?;
        let (abandoned_publish_with_a_unit_overwritten, _) =
            publishes
                .iter()
                .find(|((instance, checkpoint_txg), units)| {
                    let mut copies_by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
                    for unit in units {
                        copies_by_offset
                            .entry(writes[*unit].offset.0)
                            .or_default()
                            .push(*unit);
                    }
                    *checkpoint_txg <= landed_version.1 .0
                        && table.abandons(*instance, *checkpoint_txg)
                        && units.iter().all(|unit| *unit < first_write_of_the_segment)
                        && copies_by_offset.values().any(|copies| {
                            copies
                                .iter()
                                .all(|copy| !unit_write_is_still_on_disk(&image, &writes[*copy]))
                        })
                })?;
        Some((landed_version, *abandoned_publish_with_a_unit_overwritten))
    })
}

/// 调查报告 `research/prompts/m2-investigate-crash-injection-reds-report.md` 第 1 条（装置错）那一形：种子基 + 2 那段历史上摆出来的一个崩溃状态，
/// 扣下一份 journal 记录、留下另一份，恢复施加记录落到由记录重建的那一版，那一版的根槽写在更晚的段里（D23（journal 的角色与格式） 已定项 15）；
/// 那一版的实例表抛弃了更早的一次发布，那次发布的单元已被后来的发布复用盖掉。那时记录核对器按落到的那一版的实例表豁免它（实八），
/// 前提是交给它的写表里找得到那一版的根槽写（崩溃注入交整条流）。
///
/// **C554 乙之后这一形不可达**（改钉「这一形不可达」）：实例表抛弃一次发布要崩溃恢复抛弃根
/// （`CrashRecoveryAbandoningTheNewestRoot`），而那条根是这个会话发的、系统配置见证过它，下一次可写挂载读不出它时按 C554 乙拒
/// （`MountError::NewerStateStillUnreadableAfterOneReread`），没有新实例、没有抛弃行。没有 C554 乙时这一条绿；
/// 今天的底座上把 C577 那道屏障去掉照样摆不出来，不可达是 C554 乙定的。
/// 这一条保留原来的构造（同一段历史、同一批崩溃状态），改钉：这批崩溃状态里一个这样的都没有，记录核对器一条都不判。
/// 把 C554 乙那一判关掉，这一形回来，第一条断言红（`crates/mutations.tsv`）。
#[test]
fn crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons(
) {
    let history = history_with_a_crash_state_landing_on_a_version_rebuilt_from_records();
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        DEFAULT_BATCH_DRAW,
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：种子基 + 2 那段历史，C554 乙之后找「恢复落到由记录重建的一版、它的实例表抛弃了单元被复用的发布」那一形 ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons",
    ) {
        return;
    }
    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
    assert!(
        !crash_recovery_outcomes.is_empty()
            && crash_recovery_outcomes.iter().all(|outcome| *outcome
                == StepOutcome::Refused {
                    member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
                }),
        "这段历史里的崩溃恢复抛弃根每一步都被 C554 乙拒、一个被抛弃的根都没造出来：{crash_recovery_outcomes:?}"
    );
    assert_eq!(
        crash_state_landing_on_a_version_rebuilt_from_records(&history, &injection.crash_points),
        None,
        "C554 乙之后崩溃恢复抛弃根那一步被拒、造不出被抛弃的根：这批崩溃状态里不该有落到由记录重建、实例表抛弃了单元被复用的发布的那一版"
    );
    assert_eq!(
        injection.tally.record_checks,
        u64::try_from(injection.crash_points.len()).expect("崩溃状态数装得进 u64"),
        "每个崩溃状态都跑过记录核对器"
    );
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 0,
        "这段历史上的崩溃状态一条「恢复自称新态而单元缺席」都不判"
    );
    assert_eq!(
        injection.tally.record_root_without_record, 0,
        "这段历史上没有「根在而它那次发布的记录一条都不在」的崩溃状态"
    );
}

/// 判别力那一半（调查报告第 1 条「推翻条件与造它的那一次」第 2 次）原来在上一条那一形上做：落到的那一版在它实例里的前一版（没被抛弃）
/// 写在某个偏移上的单元两份都扣下，照判红。**C554 乙之后那一形不可达**（见上一条），这一条改钉：用默认批的崩溃状态与同一段历史，
/// 一个那样的崩溃状态都摆不出来。「没被抛弃的发布缺单元照样判红」那一半由
/// [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`] 的扣单元那两次守着。
/// 把 C554 乙那一判关掉，这一形回来，这一条红（`crates/mutations.tsv`）。
#[test]
fn on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red(
) {
    let history = history_with_a_crash_state_landing_on_a_version_rebuilt_from_records();
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        DEFAULT_BATCH_DRAW,
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
    assert!(
        !crash_recovery_outcomes.is_empty()
            && crash_recovery_outcomes.iter().all(|outcome| *outcome
                == StepOutcome::Refused {
                    member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
                }),
        "这段历史里的崩溃恢复抛弃根每一步都被 C554 乙拒、一个被抛弃的根都没造出来：{crash_recovery_outcomes:?}"
    );
    assert_eq!(
        crash_state_landing_on_a_version_rebuilt_from_records(&history, &injection.crash_points),
        None,
        "C554 乙之后这一形不可达：判别力那一半没有可做的崩溃状态"
    );
}

/// 偏向抬 F 之后回退那一组比重的取样点：这个测试周期的种子基起 12 段、每段 20 步、每段摆 6 个崩溃状态，一条清单外的失败都不许有。
///
/// 这一段**不**断言「某一类操作里至少摆到一个崩溃状态」：那是抽样摆出来的形态，下一个测试周期重抽种子基就可能一次都没有
/// （前提不成立就不调入口；在随机种子基上实测撞到过：49 个崩溃状态里 `RaiseRollbackFloor` 是 0），
/// 那样的断言判的是抽样运气，不是代码。回退与抬 F 罩得到这一条由上面那条写死历史的用例钉着，那里是全枚举。
/// 各类操作各摆到几个照打进报告，给人看。
// crash-case-check:not-a-crash-case 偏向抬 F 之后回退的随机取样点（种子基起 12 段、每段 6 个崩溃状态），抽样摆点、不枚举；随机崩溃注入的默认批登记成 crash-case:crash-injection-default-batch
#[test]
fn crash_states_land_inside_rollbacks_and_floor_raises() {
    let report = run_through_the_engine(&rollback_sampling_campaign(
        CrashInjectionWorkerThreads::from_the_environment(),
    ));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入：偏向抬 F 之后回退的取样点 ──\n{rendered}"
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&report.tally),
        "crash_states_land_inside_rollbacks_and_floor_raises",
    ) {
        return;
    }
    judge_crash_injection_report(&report);
}

/// 回退取样那一组：摆出过崩溃状态、恢复没失败过（不钉抽样摆出来的形态，见用到它的那条用例的文档注释）。
fn assert_the_rollback_sampling_placed_crash_states_and_recovered_from_each(
    report: &CrashInjectionReport,
) {
    assert!(
        report.tally.crash_points >= 1,
        "一个崩溃状态都没摆出来；{}",
        how_to_replay(report)
    );
    assert_eq!(
        report.tally.recoveries_failed,
        0,
        "崩溃之后恢复失败过；{}",
        how_to_replay(report)
    );
}

/// 偏向抬 F 之后回退的那一批：种子基起 12 段、每段 20 步、每段 6 个崩溃状态。
fn rollback_sampling_campaign(
    worker_threads: CrashInjectionWorkerThreads,
) -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 12,
        operations_per_history: 20,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 6,
        },
        weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads,
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::RollbackSampling,
    }
}

/// 崩溃注入那条入口读环境：变量与判红时打的重放行同一张表（`crash_injection_campaign_from_environment`），没设的取默认批的
/// （[`default_batch_campaign`]，照默认批那一组断言判），线程数从 `SINGLEFS_CRASH_INJECTION_THREADS` 取。入口、「不设变量取默认批」那一格
/// 与「重放行喂回入口」那几条自证都经这一个函数。读回的不是默认批时（设变量照重放行复现、或换一批历史看现场）判红的崩溃镜像留着、
/// 路径打进报告；默认批跑完删掉，临时目录里不留东西。
fn crash_injection_campaign_from(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> CrashInjectionCampaign {
    let default_batch = default_batch_campaign(CrashInjectionWorkerThreads::from_the_environment());
    let campaign = crash_injection_campaign_from_environment(lookup, &default_batch);
    if campaign == default_batch {
        campaign
    } else {
        CrashInjectionCampaign {
            image_retention: FailureImageRetention::KeepTheImageFiles,
            ..campaign
        }
    }
}

/// 一个变量都没设读回默认批；设了的每一项照变量读回、没设的那几项仍取默认批的（设了变量也读回默认批时这一条红）。线程数不比；
/// 读回的不是默认批时判红的崩溃镜像留着。
#[test]
fn the_crash_injection_campaign_is_the_default_batch_without_variables_and_follows_the_variables_that_are_set(
) {
    let default_batch = default_batch_campaign(CrashInjectionWorkerThreads::GivenByCaller(1));
    assert_eq!(
        CrashInjectionCampaign {
            worker_threads: default_batch.worker_threads,
            ..crash_injection_campaign_from(&|_variable: &str| None)
        },
        default_batch,
        "一个 SINGLEFS_CRASH_INJECTION_* 都没设，跑的该是默认批、照默认批那一组断言判、跑完删镜像"
    );
    let given = |variable: &str| {
        match variable {
            "SINGLEFS_CRASH_INJECTION_FIRST_SEED" => Some("11"),
            "SINGLEFS_CRASH_INJECTION_SEEDS" => Some("500"),
            "SINGLEFS_CRASH_INJECTION_OPERATIONS" => Some("40"),
            "SINGLEFS_CRASH_INJECTION_POINTS" => Some("8"),
            "SINGLEFS_CRASH_INJECTION_ASSERTIONS" => Some("new-findings"),
            _ => None,
        }
        .map(str::to_string)
    };
    assert_eq!(
        CrashInjectionCampaign {
            worker_threads: default_batch.worker_threads,
            ..crash_injection_campaign_from(&given)
        },
        CrashInjectionCampaign {
            first_seed: 11,
            seed_count: 500,
            operations_per_history: 40,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 8,
            },
            image_retention: FailureImageRetention::KeepTheImageFiles,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
            ..default_batch
        },
        "设了的种子起点、种子数、步数、崩溃点数、断言组照变量读回，没设的比重与跑法取默认批的，镜像留着"
    );
}

/// 生成器那一路没被这一件碰过：同一个种子生成的历史与随机历史那一段逐项相同（摆崩溃状态的随机源与它岔开）。
#[test]
fn drawing_crash_points_does_not_change_the_generated_history() {
    let history = generate_history(HistorySeed(5), 18);
    let without_any = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::Sampled {
            crash_points_per_history: 0,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    let with_five = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::Sampled {
            crash_points_per_history: 5,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    assert_eq!(with_five.seed, HistorySeed(5));
    assert_eq!(
        history,
        generate_history(HistorySeed(5), 18),
        "生成的历史逐项相同"
    );
    // 名副其实的那一半：摆崩溃状态不改变这段历史自己怎么跑。
    assert_eq!(
        without_any.tally.histories_completed, with_five.tally.histories_completed,
        "摆了崩溃状态之后这段历史的收尾变了"
    );
    assert_eq!(
        without_any.tally.histories_ended_known_red, with_five.tally.histories_ended_known_red,
        "摆了崩溃状态之后这段历史落到的已知红变了"
    );
    assert_eq!(
        without_any.tally.most_committed_versions_in_one_history,
        with_five.tally.most_committed_versions_in_one_history,
        "摆了崩溃状态之后模型提交过的版本数变了"
    );
    // 判别力：不摆就一个崩溃状态都没有，摆了就真摆出来了；两句都不成立时上面那三条恒真。
    assert_eq!(
        without_any.tally.crash_points, 0,
        "要 0 个崩溃状态却摆出了 {}",
        without_any.tally.crash_points
    );
    assert!(
        with_five.crash_points.len() <= 5 && !with_five.crash_points.is_empty(),
        "摆出来的崩溃状态要落在 1..=5：{}",
        with_five.crash_points.len()
    );
}

/// 判红时对一批崩溃注入会打的两行重放命令（`CrashInjectionReport::render` 里新发现之后的「复现这一段：」与末尾的「重放这一整批：」；
/// 造一个假发现，不必真红），原样喂回崩溃注入那条入口（`crash_injection_campaign_from`，入口用的同一个函数）：
/// 种子区间是那一段（种子数 1）或那一整批，崩溃状态怎么摆、每段步数、比重、跑法与原来那一批逐项相同，那一段生成的历史逐步相同；
/// 「重放这一整批」照原来那一批的那一组断言判，「复现这一段」只判清单外的失败（一段的新发现与整批的结构计数无关）。
/// 不跑崩溃状态：同一组参数下摆出来的崩溃状态由种子定（`crash_points_are_reproducible`），这里只核参数一项不漏地回来了。
fn the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_campaign_entry(
    campaign: &CrashInjectionCampaign,
) {
    let finding_seed = HistorySeed(campaign.first_seed + 5);
    let report = CrashInjectionReport {
        first_seed: campaign.first_seed,
        seed_count: campaign.seed_count,
        operations_per_history: campaign.operations_per_history,
        draw: campaign.draw,
        weights: campaign.weights,
        execution: campaign.execution,
        tally: CrashInjectionTally::default(),
        known_red_hits: Vec::new(),
        new_findings: vec![CrashPointFinding {
            signature: FailureSignature::HarnessJudgement {
                judgement: "自证用的假发现",
            },
            seed: finding_seed,
            crash_point: CrashPoint {
                segment_index: 3,
                persisted_within_the_segment: vec![true, false],
                step: StepPosition::Operation(2),
                operation_kind: None,
            },
            stage: CrashStateStage::CrashImage,
            observation: FailureObservation {
                position: StepPosition::Operation(2),
                operation_kind: None,
                violations: Vec::new(),
                panic: None,
                newest_ring_root_txg: None,
                root_ring_slot_count: None,
                harness_judgement: None,
                model_disagreement: None,
                raised_floor_lands_only_on_abandoned_roots: None,
                record_check: RecordCheck::default(),
            },
            read_back: "自证不读回".to_string(),
            image_files: None,
        }],
        image_directory: None,
        image_retention: campaign.image_retention,
        assertions: campaign.assertions,
        histories_left_by_the_stop_request: 0,
    };
    let rendered = report.render();
    let replay_lines: Vec<&str> = rendered
        .lines()
        .filter(|line| line.contains("SINGLEFS_CRASH_INJECTION_FIRST_SEED="))
        .collect();
    assert_eq!(
        replay_lines.len(),
        2,
        "判红的报告里有两行重放命令（那一段、那一整批）：\n{rendered}"
    );
    let replayed_parameters = |line: &str| {
        let assignments = environment_assignments_in_line(line);
        let replayed =
            crash_injection_campaign_from(&|variable: &str| assignments.get(variable).cloned());
        (
            replayed.first_seed,
            replayed.seed_count,
            replayed.operations_per_history,
            replayed.draw,
            replayed.weights,
            replayed.execution,
            replayed.assertions,
        )
    };
    let original_batch = (
        campaign.first_seed,
        campaign.seed_count,
        campaign.operations_per_history,
        campaign.draw,
        campaign.weights,
        campaign.execution,
        campaign.assertions,
    );
    let original_finding = (
        finding_seed.0,
        1,
        campaign.operations_per_history,
        campaign.draw,
        campaign.weights,
        campaign.execution,
        CrashInjectionAssertions::NewFindingsOnly,
    );
    let replayed_finding = replayed_parameters(replay_lines[0]);
    assert_eq!(
        replayed_finding, original_finding,
        "照「复现这一段」那一行跑崩溃注入入口，跑的不是原来那一段（种子、种子数、步数、摆法、比重、跑法、照哪一组断言判）：{}",
        replay_lines[0]
    );
    assert_eq!(
        replayed_parameters(replay_lines[1]),
        original_batch,
        "照「重放这一整批」那一行跑崩溃注入入口，跑的不是原来那一批、或不照原来那一组断言判：{}",
        replay_lines[1]
    );
    assert_eq!(
        generate_history_with_weights(
            HistorySeed(replayed_finding.0),
            replayed_finding.2,
            &replayed_finding.4
        ),
        generate_history_with_weights(
            finding_seed,
            campaign.operations_per_history,
            &campaign.weights
        ),
        "那一段生成的历史逐步相同"
    );
}

/// 照默认批的重放行跑的崩溃注入入口按默认批那一组断言判：一批二次崩溃一个都没摆出来的报告（造的计数，不跑崩溃状态；
/// 攻方的改坏就是这一形：`draw_second_crash_points` 一个都不摆），判红在默认批那一条「挂载途中一个二次崩溃都没摆出来」上，
/// 不是只判新发现而判绿（代码三方 test-script-align-code-r1 判决 S2）。
// crash-case-check:not-a-crash-case 造的计数（一批二次崩溃一个都没摆出来的报告）判红在默认批断言上，不跑崩溃状态，核默认批断言的判别力
#[test]
#[should_panic(expected = "挂载途中一个二次崩溃都没摆出来")]
fn a_default_batch_replay_without_second_crashes_is_judged_red_by_the_default_batch_assertions() {
    let campaign = default_batch_campaign(CrashInjectionWorkerThreads::GivenByCaller(1));
    let every_crash_state = DEFAULT_BATCH_SEEDS;
    let tally = CrashInjectionTally {
        crash_points: every_crash_state,
        model_judgements: every_crash_state,
        checker_runs: every_crash_state,
        record_checks: every_crash_state,
        writable_mounts_after_the_crash: every_crash_state,
        checker_runs_after_the_writable_mount: every_crash_state,
        record_checks_after_the_writable_mount: every_crash_state,
        second_crash_points: 0,
        // 计数袋没有不变量，其余计数从零起（与 `crash_injection.rs` 里汇总一段历史的计数同一种写法）。
        ..CrashInjectionTally::default()
    };
    let replay_line = CrashInjectionReport {
        first_seed: campaign.first_seed,
        seed_count: campaign.seed_count,
        operations_per_history: campaign.operations_per_history,
        draw: campaign.draw,
        weights: campaign.weights,
        execution: campaign.execution,
        tally: CrashInjectionTally::default(),
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
        image_directory: None,
        image_retention: campaign.image_retention,
        assertions: campaign.assertions,
        histories_left_by_the_stop_request: 0,
    }
    .replay_line();
    let assignments = environment_assignments_in_line(&replay_line);
    let replayed =
        crash_injection_campaign_from(&|variable: &str| assignments.get(variable).cloned());
    judge_crash_injection_report(&CrashInjectionReport {
        first_seed: replayed.first_seed,
        seed_count: replayed.seed_count,
        operations_per_history: replayed.operations_per_history,
        draw: replayed.draw,
        weights: replayed.weights,
        execution: replayed.execution,
        tally,
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
        image_directory: None,
        image_retention: replayed.image_retention,
        assertions: replayed.assertions,
        histories_left_by_the_stop_request: 0,
    });
}

#[test]
fn replay_lines_of_the_default_batch_run_the_same_campaign_on_the_campaign_entry() {
    the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_campaign_entry(
        &default_batch_campaign(CrashInjectionWorkerThreads::GivenByCaller(1)),
    );
}

#[test]
fn replay_lines_of_the_rollback_sampling_run_the_same_campaign_on_the_campaign_entry() {
    the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_campaign_entry(
        &rollback_sampling_campaign(CrashInjectionWorkerThreads::GivenByCaller(1)),
    );
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

/// 崩溃镜像那一截（只读恢复、模型、池级 checker、记录核对器）判红记下的那一条新发现；没有时 panic，带上全部新发现。
fn finding_on_the_crash_image(injection: &HistoryCrashInjection) -> &CrashPointFinding {
    injection
        .new_findings
        .iter()
        .find(|finding| finding.stage == CrashStateStage::CrashImage)
        .unwrap_or_else(|| panic!("崩溃镜像那一截没有判红：{:#?}", injection.new_findings))
}

/// 记录核对器第一条判据（根在而那次发布的记录一条都不在）接进崩溃注入之后真的判得出东西（K3-④）：
/// 第 22 段起数 3 个写（(1, 5) 的两份 journal 记录与它的根槽写），两份记录扣下、根槽写持久——越过了记录与根之间那道屏障，
/// 屏障守住时摆不出来。恢复落到 (1, 5)、读回它的内容，模型认、池级 checker 不红（在 harness 侧照同一个持久集合现量的），
/// 这个崩溃状态只有记录核对器判得出。把崩溃镜像那一截通过条件里的 `record_check == RecordCheck::default()` 丢掉、
/// 计数照记（攻方变异 A，`crates/mutations.tsv`），这一截就没有新发现，这一条红。
#[test]
fn a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone()
{
    print_not_run_when_the_targeted_case_was_not_judged(
        the_root_without_either_of_its_records_is_found_by_the_record_checker_alone(
            given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
        ),
        "a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone",
    );
}

/// 同 [`a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone`]，那一个靶向状态交起在一份不开 GPU 的配置上的引擎
/// （[`given_states_engine_on_a_configuration_without_the_gpu`]：只由 CPU 判器判），收尾核这一趟是 CPU 判器判的。
/// 判别力（`crates/mutations.tsv`「实审 B3c-1：崩溃注入第一截交给记录核对器的写表少了崩溃窗口末尾那个写」）：CPU 判器交记录核对器的写表截掉落了盘的根槽写，
/// 「根在而记录一条都不在」判不出，这条红。
#[test]
fn on_a_configuration_without_the_gpu_a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone(
) {
    let engine_on_the_cpu = given_states_engine_on_a_configuration_without_the_gpu(
        "crash_injection_campaign_targeted_history_on_the_cpu",
    );
    let outcome = the_root_without_either_of_its_records_is_found_by_the_record_checker_alone(
        engine_on_the_cpu.engine(),
    );
    finish_the_engine_asserting_it_judged_on_the_cpu(engine_on_the_cpu);
    assert_eq!(
        outcome,
        TargetedCaseOutcome::Judged,
        "引擎起在清掉分片与选段变量的环境里，这个靶向状态这一趟就判"
    );
}

/// [`a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone`] 那一个靶向状态交 `judge` 判：
/// 引擎交了占位结论就不判、交回为什么（[`why_the_placeholder_conclusions_leave_this_run_unjudged`]）。
fn the_root_without_either_of_its_records_is_found_by_the_record_checker_alone(
    judge: &dyn GivenStatesJudge,
) -> TargetedCaseOutcome {
    let history = history_of_a_first_file_and_two_overwrites();
    let (writes, segments) = writes_and_segments_of(&history);
    let first_write = segments[23][0];
    assert_eq!(
        writes[first_write..first_write + 3]
            .iter()
            .map(|write| write.kind)
            .collect::<Vec<_>>(),
        vec![
            StepKind::JournalRecord,
            StepKind::JournalRecord,
            StepKind::RootRecordFua
        ],
        "第 22 段起的 3 个写是一次发布的两份 journal 记录与它的根槽写"
    );
    assert_eq!(
        root_identity_of_a_root_slot_write(&writes[first_write + 2]),
        Some((InstanceGeneration(1), CheckpointTxg(5))),
        "那个根槽写是第二次覆盖写 (1, 5) 的"
    );

    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: 23,
            writes_from_the_segment_on: 3,
            withheld_among_them: &[0, 1],
        },
        None,
        judge,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：记录核对器第一条判据的靶向崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    if let Some(reason) = why_the_placeholder_conclusions_leave_this_run_unjudged(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
    ) {
        return TargetedCaseOutcome::NotRunOnPlaceholderConclusions { reason };
    }
    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
    assert_eq!(
        injection.tally.record_root_without_record, 1,
        "记录核对器在这个崩溃状态上判出「根在而它那次发布的记录一条都不在」"
    );
    let finding = finding_on_the_crash_image(&injection);
    assert_eq!(
        finding.signature,
        FailureSignature::RecordCheck {
            aspects: vec!["root_without_record"]
        },
        "崩溃镜像那一截的新发现记在记录核对器头上：{finding:#?}"
    );
    assert!(
        finding.observation.violations.is_empty()
            && finding.observation.model_disagreement.is_none(),
        "池级 checker、RefFS 与模型都不红，这一条新发现只来自记录核对器：{finding:#?}"
    );
    TargetedCaseOutcome::Judged
}

/// 记录核对器第二条判据（恢复自称的那一版不早于某次发布、而那次发布写出的某个单元两份都不在）接进崩溃注入之后真的判得出东西（K3-⑨）：
/// 第 13 段起数 56 个写（(1, 3) 的单元写起、到 (1, 4) 的根槽写为止），扣下 (1, 3) 的数据单元两份、别的都持久——
/// 越过了 (1, 3) 自己单元与记录之间、记录与根之间两道屏障。恢复落到 (1, 4)（它的根槽写持久了），读回它的内容，模型认。
/// (1, 3) 的数据单元已被 (1, 4) 换下、还没被复用，盘上那两处一直是空的，记录核对器判「恢复自称新态而单元缺席」。
/// 照常点名验证的恢复下，只扣单元不越过根的状态落回上一版，这一条判据判不到（K3-⑨ 调查），所以这里让根也持久。
/// 池级 checker 也红（根环里还留着 (1, 3) 的根，从它出发的遍历读不到那个数据单元），签名照「记录核对器先于 checker」取记录核对器；
/// 只有记录核对器红的那一个状态是下一条用例。
/// 把崩溃镜像那一截交给记录核对器的「恢复落到的那一版」换成 None（`crates/mutations.tsv`），第二条判据的前置不成立，这一条红。
#[test]
fn a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker(
) {
    print_not_run_when_the_targeted_case_was_not_judged(
        the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker(
            given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
        ),
        "a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker",
    );
}

/// 同 [`a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker`]，那一个靶向状态交起在一份不开 GPU 的配置上的引擎
/// （[`given_states_engine_on_a_configuration_without_the_gpu`]：只由 CPU 判器判），收尾核这一趟是 CPU 判器判的。
/// 判别力（`crates/mutations.tsv`「增补 3 第 3 件第 1 题：崩溃镜像那一截交给记录核对器的恢复落到的那一版换成 None」）：CPU 判器交记录核对器的那一版是 None，
/// 第二条判据的前置恒不成立，「恢复自称新态而单元缺席」判不出，这条红。
#[test]
fn on_a_configuration_without_the_gpu_a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker(
) {
    let engine_on_the_cpu = given_states_engine_on_a_configuration_without_the_gpu(
        "crash_injection_campaign_targeted_history_on_the_cpu",
    );
    let outcome = the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker(
        engine_on_the_cpu.engine(),
    );
    finish_the_engine_asserting_it_judged_on_the_cpu(engine_on_the_cpu);
    assert_eq!(
        outcome,
        TargetedCaseOutcome::Judged,
        "引擎起在清掉分片与选段变量的环境里，这个靶向状态这一趟就判"
    );
}

/// [`a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker`] 那一个靶向状态交 `judge` 判：
/// 引擎交了占位结论就不判、交回为什么（[`why_the_placeholder_conclusions_leave_this_run_unjudged`]）。
fn the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker(
    judge: &dyn GivenStatesJudge,
) -> TargetedCaseOutcome {
    let history = history_of_a_first_file_and_two_overwrites();
    let (writes, segments) = writes_and_segments_of(&history);
    let first_write = segments[14][0];
    let withheld_copies = &writes[first_write..first_write + 2];
    assert!(
        withheld_copies
            .iter()
            .all(|write| write.kind == StepKind::UnitWrite
                && write.length_in_bytes() == DATA_UNIT_BYTES
                && write.offset == withheld_copies[0].offset)
            && withheld_copies[0].device != withheld_copies[1].device,
        "第 13 段的头两个写是同一个数据单元在两块盘上的两份：{withheld_copies:?}"
    );
    assert_eq!(
        writes[first_write + 2..first_write + 56]
            .iter()
            .filter_map(root_identity_of_a_root_slot_write)
            .collect::<Vec<_>>(),
        vec![
            (InstanceGeneration(1), CheckpointTxg(3)),
            (InstanceGeneration(1), CheckpointTxg(4))
        ],
        "从第 13 段起数的 56 个写里有 (1, 3) 与 (1, 4) 的根槽写，最后一个是 (1, 4) 的"
    );
    assert_eq!(
        root_identity_of_a_root_slot_write(&writes[first_write + 55]),
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "从第 13 段起数的第 56 个写是 (1, 4) 的根槽写"
    );

    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: 14,
            writes_from_the_segment_on: 56,
            withheld_among_them: &[0, 1],
        },
        None,
        judge,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：记录核对器第二条判据的靶向崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    if let Some(reason) = why_the_placeholder_conclusions_leave_this_run_unjudged(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
    ) {
        return TargetedCaseOutcome::NotRunOnPlaceholderConclusions { reason };
    }
    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 1,
        "记录核对器在这个崩溃状态上判出「恢复自称新态而单元缺席」"
    );
    let finding = finding_on_the_crash_image(&injection);
    assert_eq!(
        finding.signature,
        FailureSignature::RecordCheck {
            aspects: vec!["claimed_state_missing_unit"]
        },
        "崩溃镜像那一截的新发现记在记录核对器第二条判据头上：{finding:#?}"
    );
    assert!(
        finding.observation.model_disagreement.is_none(),
        "恢复落到 (1, 4) 并读回它的内容，模型认：{finding:#?}"
    );
    TargetedCaseOutcome::Judged
}

/// 两台分片的那一趟（崩溃点重放入口双机跑时给本机设 `SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2`、给第二台设 `1/2`），照别的机器还没判完的样子造：
/// 环境守卫里照一份单机 CPU 配置、两份分片各起一个引擎（各一份起跑是空的一次性库，库里没有可复用的结论），各判记录核对器那两条阳性对照的
/// 靶向状态。每张给定状态表恰好归一台，所以每条阳性对照在两份里至少有一份拿到占位结论：那一份不判、交回为什么（用例自己跑时打「本次未跑」，
/// 这一条不打）；判了的那一份阳性照判（过了断言才交回「判了」）。
/// 判别力：断言阳性之前不核归别的机器判的占位（`crates/mutations.tsv` 那一行），首截表归另一台的那一份红在阳性对照的计数断言上（`left: 0 right: 1`）。
#[test]
fn the_record_checker_positive_controls_are_not_run_on_the_share_given_placeholder_conclusions_of_the_other_host(
) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    /// 一条记录核对器阳性对照：把它那一个靶向状态交一个判器判。
    type RecordCheckerPositiveControl = fn(&dyn GivenStatesJudge) -> TargetedCaseOutcome;
    let (_directory, configuration) = disposable_library_with_a_configuration_on_the_cpu(
        "record-checker-positive-controls-under-two-shares",
    );
    let positive_controls: [(&str, RecordCheckerPositiveControl); 2] = [
        (
            "the_root_without_either_of_its_records_is_found_by_the_record_checker_alone",
            the_root_without_either_of_its_records_is_found_by_the_record_checker_alone,
        ),
        (
            "the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker",
            the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker,
        ),
    ];
    for (positive_control_name, positive_control) in positive_controls {
        let mut outcome_of_each_share = Vec::new();
        for share in ["0/2", "1/2"] {
            let library = disposable_library_of_the_campaign(&format!(
                "{positive_control_name}-share-{}",
                share.replace('/', "-of-")
            ));
            let _the_library_of_this_process = library.make_it_the_library_of_this_process();
            let environment = CrashAmplificationEnvironment::cleared_then_set(&[
                ("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str()),
                (
                    "SINGLEFS_CRASH_AMPLIFICATION_SHARE",
                    std::ffi::OsStr::new(share),
                ),
            ]);
            let engine = given_states_engine_under_the_callers_environment_guard(
                "crash_injection_campaign_targeted_history",
            );
            drop(environment);
            outcome_of_each_share.push((share, positive_control(&engine)));
        }
        // 不打「本次未跑」那一行：占位那一份是这一条要的样子，打了整项会被入口记成本次未跑
        assert!(
            outcome_of_each_share.iter().any(|(_, outcome)| matches!(
                outcome,
                TargetedCaseOutcome::NotRunOnPlaceholderConclusions { .. }
            )),
            "{positive_control_name}：每张表恰好归一台，两份里至少有一份拿到占位结论、不判：{outcome_of_each_share:?}"
        );
    }
}

/// 崩溃注入第一截交给记录核对器的写表取整条历史录制流（`crash_injection.rs` 第一截那一次 `check_records_against`），
/// 与只取到崩溃点所在段为止的前缀，判出来不同只在两种崩溃状态上（推的：前缀之后的写一个都没持久，缺席判定与回收谓词只用得到
/// 持久的后写与更早的写；整条流多出来的只有根槽写在前缀之后的那几次发布，与恢复落到的那一版的根槽写）：
/// - 恢复落到由记录重建的一版、它的实例表抛弃了某次单元已被复用的发布：前缀里没有那一版的根槽写，实例表读不出，豁免落空。
///   C554 乙之后这一形摆不出来（[`crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons`]、
///   [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`] 钉着）。
/// - 恢复落到由记录重建的一版、那一版自己的某个单元两份都不在：前缀里没有它的根槽写，那次发布归不出来，第二条判据罩不到它。
///   这一条造的就是它：第二次覆盖写 (1, 5) 的第一个数据单元两份（第 21 段头两个写）扣下，第 21 段别的单元写与第 22 段两份 journal 记录都持久，
///   第 23 段 (1, 5) 的根槽写在窗口之后、没落。验点名单元的恢复（`JournalPolicy::Consult`，崩溃注入用的就是它）不施加那条记录、落回 (1, 4)，
///   两种交法同判；只有不验点名单元的恢复（只供测试强制进入的 `JournalPolicy::ConsultWithoutNamedVerification`）落到 (1, 5)，整条流判红、前缀不判。
///
/// 所以把第一截的写表退回前缀那一条变异不是等价变异：这一条自己就造出了两种交法不同值的输入（不验点名单元的恢复落到 (1, 5)，整条流判红、前缀不判）。
/// 它是第三类「取样点不敏感」（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」）：崩溃注入的取值域里摆不出能分辨两种交法的盘面——
/// 崩溃注入恢复只用验点名单元的那一种（`JournalPolicy::Consult`），C554 乙那一判又让另一形摆不出来；补不上敏感的取样点，是因为要给崩溃注入
/// 加换恢复策略的开关（改被测代码，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第五节「崩溃注入换恢复策略的开关」那一行），
/// 不是因为两种交法在所有输入上同值。这一条钉住取值域外那一支的两种交法确实分开、取值域里那一形两种交法同判；C554 乙那一判或点名验证哪天不在了，
/// 崩溃注入里两种交法也会分开。
/// 第二条判据不罩恰好是恢复落到的那一版的那次发布（`crates/mutations.tsv`），整条流那一判不红，这一条红。
#[test]
fn a_version_rebuilt_from_records_missing_a_unit_is_judged_only_on_the_whole_stream_and_the_verifying_recovery_never_lands_on_it(
) {
    let history = history_of_a_first_file_and_two_overwrites();
    let (writes, segments) = writes_and_segments_of(&history);
    let root_of_the_second_overwrite = segments[24][0];
    assert_eq!(
        root_identity_of_a_root_slot_write(&writes[root_of_the_second_overwrite]),
        Some((InstanceGeneration(1), CheckpointTxg(5))),
        "第 23 段的头一个写是第二次覆盖写 (1, 5) 的根槽写"
    );
    assert_eq!(
        segments[23]
            .iter()
            .map(|index| writes[*index].kind)
            .collect::<Vec<_>>(),
        vec![StepKind::JournalRecord, StepKind::JournalRecord],
        "第 22 段是 (1, 5) 的两份 journal 记录"
    );
    let first_write = segments[22][0];
    let withheld_copies = &writes[first_write..first_write + 2];
    assert!(
        withheld_copies
            .iter()
            .all(|write| write.kind == StepKind::UnitWrite
                && write.length_in_bytes() == DATA_UNIT_BYTES
                && write.offset == withheld_copies[0].offset)
            && withheld_copies[0].device != withheld_copies[1].device,
        "第 21 段的头两个写是 (1, 5) 一个数据单元在两块盘上的两份：{withheld_copies:?}"
    );
    let end_of_the_window = segments[23][0] + segments[23].len();
    assert_eq!(
        end_of_the_window, root_of_the_second_overwrite,
        "窗口（第 21、22 段）截在 (1, 5) 的根槽写之前"
    );

    let persisted_within_the_window: Vec<bool> = (first_write..end_of_the_window)
        .map(|index| index >= first_write + 2)
        .collect();
    let persisted =
        persisted_over_the_whole_stream(writes.len(), first_write, &persisted_within_the_window);
    let base = pool_of_the_persisted_writes_before_the_segment(
        &writes[..first_write],
        &persisted,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
            .device_width
            .device_bytes(),
    );
    let image = CrashImage {
        base: &base,
        writes: &writes[first_write..end_of_the_window],
        persisted: persisted_within_the_window,
    };
    // 第一截的交法：两份镜像同一份、一条录制流（这段历史没掉过电），写表与持久集合取到 `end_of_table` 为止。
    let record_check_on_the_table_up_to =
        |end_of_table: usize, effective_root: Option<(InstanceGeneration, CheckpointTxg)>| {
            check_records_against(
                &image,
                &image,
                &writes[..end_of_table],
                &persisted[..end_of_table],
                RecordStreamContinuity::OneRecording,
                effective_root,
            )
        };

    let verifying = recover(&image, JournalPolicy::Consult).effective_root;
    assert_eq!(
        verifying,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "验点名单元的恢复不施加 (1, 5) 的记录（它点名的数据单元两份都不在），落回 (1, 4)"
    );
    assert_eq!(
        record_check_on_the_table_up_to(writes.len(), verifying),
        RecordCheck::default(),
        "落回 (1, 4)：整条流上记录核对器一条都不判"
    );
    assert_eq!(
        record_check_on_the_table_up_to(end_of_the_window, verifying),
        RecordCheck::default(),
        "落回 (1, 4)：前缀上也一条都不判，两种交法同值"
    );

    let unverifying =
        recover(&image, JournalPolicy::ConsultWithoutNamedVerification).effective_root;
    assert_eq!(
        unverifying,
        Some((InstanceGeneration(1), CheckpointTxg(5))),
        "不验点名单元的恢复施加 (1, 5) 的记录，落到由记录重建的 (1, 5)"
    );
    assert!(
        record_check_on_the_table_up_to(writes.len(), unverifying).claimed_state_missing_unit,
        "整条流里有 (1, 5) 的根槽写，那次发布归得出来，它的数据单元两份都不在：判「恢复自称新态而单元缺席」"
    );
    assert_eq!(
        record_check_on_the_table_up_to(end_of_the_window, unverifying),
        RecordCheck::default(),
        "前缀里没有 (1, 5) 的根槽写，那次发布归不出来，第二条判据罩不到它"
    );
}

/// 第一个文件之后接着覆盖写的次数：根环三个区域各 8 槽，第 24 次覆盖写 (1, 27) 的根槽写盖掉 (1, 3) 的那一槽。
const OVERWRITES_THAT_TURN_THE_ROOT_RING_PAST_THE_FIRST_FILE: u64 = 24;

/// 第一个文件 (1, 3) 之后覆盖写 24 次：根环转过一圈，(1, 3) 的根槽已被 (1, 27) 盖掉。
fn history_of_a_first_file_and_overwrites_that_turn_the_root_ring() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: (0..OVERWRITES_THAT_TURN_THE_ROOT_RING_PAST_THE_FIRST_FILE)
            .map(|overwrite_index| overwrite_choosing(5 + 2 * overwrite_index))
            .collect(),
    }
}

/// 记录核对器第二条判据只有它自己判得出的那一个状态（第二批报告 `research/prompts/m2-rest-batch2/report.md`「停下交主 agent 的设计问题」
/// 第 2 条说找不到：缺单元的发布只要它的根还在环里，从那条根出发的 checker 遍历就会红）。根环转过一圈之后 (1, 3) 的根不在环里了：
/// 第 13 段起到流尾的每一个写都持久，只扣下第 13 段段内第 4、5 个写——(1, 3) 那一片 inode 树叶容器在两块盘上的两份（它已被 (1, 4)
/// 换下，之后没有一个写落在那两处）。恢复落到 (1, 27)、读回它的内容，模型认，池级 checker 不红（在 harness 侧照同一个持久集合现量：
/// 覆盖写 16 次时 (1, 3) 的根还在环里，checker 红 I-2.1、I-4.8、I-7.4；24 次时不红）；记录核对器判 (1, 3) 那个单元两份都缺席。
/// RefFS 的崩溃时刻取持久窗口最后一个写所在那一段（流尾那一次覆盖写），读回最后一版的内容合它。
/// 把崩溃镜像那一截通过条件里的 `record_check == RecordCheck::default()` 丢掉（攻方变异 A，`crates/mutations.tsv`），这一截就没有新发现，这一条红。
#[test]
fn a_crash_state_missing_a_unit_of_a_publish_whose_root_left_the_ring_is_a_finding_of_the_record_checker_alone(
) {
    let history = history_of_a_first_file_and_overwrites_that_turn_the_root_ring();
    let (writes, segments) = writes_and_segments_of(&history);
    let first_write = segments[14][0];
    let withheld_copies = [&writes[first_write + 4], &writes[first_write + 5]];
    assert!(
        withheld_copies
            .iter()
            .all(|write| write.kind == StepKind::UnitWrite
                && write.offset == withheld_copies[0].offset)
            && withheld_copies[0].device != withheld_copies[1].device,
        "第 13 段段内第 4、5 个写是同一个单元在两块盘上的两份：{withheld_copies:?}"
    );
    let first_file_root = writes
        .iter()
        .position(|write| {
            root_identity_of_a_root_slot_write(write)
                == Some((InstanceGeneration(1), CheckpointTxg(3)))
        })
        .expect("流里有第一个文件 (1, 3) 的根槽写");
    assert!(
        first_write + 5 < first_file_root,
        "扣下的两份是 (1, 3) 那次发布的单元写（在它的根槽写之前）"
    );
    assert!(
        writes[first_file_root + 1..].iter().any(|write| {
            write.kind == StepKind::RootRecordFua
                && write.device == writes[first_file_root].device
                && write.offset == writes[first_file_root].offset
        }),
        "(1, 3) 的根槽之后被别的根盖掉了：根环转过一圈"
    );
    assert!(
        writes[first_write + 6..].iter().all(|later| withheld_copies
            .iter()
            .all(|withheld| later.device != withheld.device || later.offset != withheld.offset)),
        "扣下的那两处之后没有一个写落上去：缺席没有更晚的写来解释"
    );
    let writes_from_the_segment_to_the_end = writes.len() - first_write;
    assert_eq!(
        writes_from_the_segment_to_the_end, 793,
        "从第 13 段的第一个写数到流尾共 793 个写"
    );

    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: 14,
            writes_from_the_segment_on: 793,
            withheld_among_them: &[4, 5],
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：只有记录核对器判得出的第二条判据靶向崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "a_crash_state_missing_a_unit_of_a_publish_whose_root_left_the_ring_is_a_finding_of_the_record_checker_alone",
    ) {
        return;
    }
    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 1,
        "记录核对器在这个崩溃状态上判出「恢复自称新态而单元缺席」"
    );
    let finding = finding_on_the_crash_image(&injection);
    assert_eq!(
        finding.signature,
        FailureSignature::RecordCheck {
            aspects: vec!["claimed_state_missing_unit"]
        },
        "崩溃镜像那一截的新发现记在记录核对器第二条判据头上：{finding:#?}"
    );
    assert!(
        finding.observation.violations.is_empty()
            && finding.observation.model_disagreement.is_none(),
        "池级 checker、RefFS 与模型都不红，这一条新发现只来自记录核对器：{finding:#?}"
    );
}

/// 写一份开 GPU 的多机配置，只写开关：显卡先后照调用方给这个进程的那一份、经环境变量交给引擎（[`settings_of_an_engine_on_the_gpu`] 的
/// `cards`，`common_gpu_cards_of_the_caller` 文件头写了为什么不抄进配置）。
#[cfg(feature = "gpu")]
fn write_a_configuration_with_the_gpu_enabled(configuration: &std::path::Path) {
    std::fs::write(configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
}

/// 调用方给这个进程的显卡先后（清场之下是清场之后那一份；不读此刻的环境），交 [`settings_of_an_engine_on_the_gpu`] 设成环境变量。
#[cfg(feature = "gpu")]
fn cards_of_the_caller() -> String {
    common_gpu_cards_of_the_caller::card_priority_value_of_the_caller()
}

/// 用例在环境守卫里起开 GPU 的引擎时设的变量：多机配置指到用例写的那一份（[`write_a_configuration_with_the_gpu_enabled`]），
/// 显卡先后写 `cards`（用例传 [`cards_of_the_caller`]）：配置只写开关，清场之下引擎要清场之后那一份显卡先后，环境变量设了（空串也算）就照它。
/// 这一台是 HOSTS 里哪一项不经环境交：引擎与卡池都照调用方那份多机配置上判法认出的那一项
/// （`gpu_card_configuration::multi_host_addresses_of_the_caller`），派到哪一台都认得出自己。
#[cfg(feature = "gpu")]
fn settings_of_an_engine_on_the_gpu<'settings>(
    configuration: &'settings std::path::Path,
    cards: &'settings str,
) -> [(&'static str, &'settings std::ffi::OsStr); 2] {
    use singlefs_verification::gpu_card_configuration::{
        CARDS_BY_PRIORITY_KEY, MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
    };
    [
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        (CARDS_BY_PRIORITY_KEY, std::ffi::OsStr::new(cards)),
    ]
}

/// 开 GPU 的小规模一批（两段、每段 8 步、每段 2 个崩溃状态）：这一趟自己写一份开 GPU 的多机配置、在环境守卫里跑（清掉、跑完还原一趟读的变量）。
/// 库是临时 prune-server 里以用例标签命名的一段一次性名字空间、起跑是空的（全仓那一份里判过同一批表的话，这一趟就全复用、一张表都不判，
/// 塞在 GPU 那一侧的故障碰不到任何状态）。给定状态表只交 GPU 判器判；`fault` 塞在 GPU 那一侧。
#[cfg(feature = "gpu")]
fn small_campaign_judged_on_the_gpu(
    tag: &str,
    fault: FaultPutOnTheGpuSide,
) -> (CrashInjectionReport, GivenStatesRun) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    let directory = disposable_library_of_the_campaign(&format!("on-the-gpu-{tag}"));
    let configuration = directory.join("multi-host-with-gpu.env");
    write_a_configuration_with_the_gpu_enabled(&configuration);
    let cards = cards_of_the_caller();
    let settings = settings_of_an_engine_on_the_gpu(&configuration, &cards).to_vec();
    let _the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
    let judged = campaign_through_the_given_states_engine(
        &CrashInjectionCampaign {
            first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            seed_count: 2,
            operations_per_history: 8,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 2,
            },
            weights: GenerationWeights::BROAD,
            execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(2),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        },
        fault,
    );
    drop(environment);
    judged
}

/// 同一小批在 CPU 上经引擎判（这一趟自己写一份 `ENABLE_GPU=0` 的多机配置，库同 [`small_campaign_judged_on_the_gpu`]：用例标签命名的一次性库、起跑是空的）：
/// GPU 那一趟的对拍对象，两趟各判各的、各写各的库。
#[cfg(feature = "gpu")]
fn small_campaign_judged_on_the_cpu(tag: &str) -> (CrashInjectionReport, GivenStatesRun) {
    let (directory, configuration) =
        disposable_library_with_a_configuration_on_the_cpu(&format!("on-the-cpu-{tag}"));
    campaign_through_the_engine_with(
        &small_campaign_judged_on_the_cpu_through_the_engine(),
        &directory,
        &[("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str())],
    )
}

/// 验收第 1 条（第一截与第三截都上 GPU；GPU 与 CPU 的对拍是独立的两趟，一段里 CPU 不既跑数据又对拍）：
/// 同一小批一趟配了 GPU、第一截与第三截的给定状态表由 GPU 判器判，一趟没开 GPU、由 CPU 判器判，各写各的库；两趟的崩溃注入报告整份
/// 逐样相同（摆出的状态数、已知红、新发现，逐条不变量的判绿与没判到次数连同 I-1.13：GPU 扫描重建那一截补上之后内核不置位的表是空的，C821；
/// 判红的镜像只比落没落盘、不比路径，
/// `CrashInjectionReport::has_the_same_conclusions_as`），两趟都没有判不了的。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
#[cfg(feature = "gpu")]
#[test]
fn a_small_campaign_judged_in_a_gpu_run_and_in_a_cpu_run_reports_the_same() {
    use singlefs_verification::crash_amplification::JudgedBy;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    if gpu_card_pool_or_report_not_run(
        "a_small_campaign_judged_in_a_gpu_run_and_in_a_cpu_run_reports_the_same",
    )
    .is_none()
    {
        return;
    }
    let (gpu_report, gpu_run) =
        small_campaign_judged_on_the_gpu("agrees", FaultPutOnTheGpuSide::NoFault);
    let (cpu_report, cpu_run) = small_campaign_judged_on_the_cpu("agrees");
    assert_eq!(
        gpu_run.judged_by,
        JudgedBy::GpuJudge,
        "配了 GPU 的那一趟：GPU 判器判"
    );
    assert_eq!(
        cpu_run.judged_by,
        JudgedBy::CpuJudge,
        "没开 GPU 的那一趟：CPU 判器判"
    );
    assert!(
        gpu_report.tally.crash_points > 0 && gpu_report.tally.second_crash_points > 0,
        "两截都摆出了状态"
    );
    assert!(
        gpu_run.tally.states_judged_by_the_gpu_here > 0 && gpu_run.tally.states_reused == 0,
        "GPU 那一趟从空库起、每个状态都是这一趟在卡上判的，不是复用的：{:?}",
        gpu_run.tally
    );
    assert!(
        cpu_run.tally.states_judged_by_the_cpu_here > 0 && cpu_run.tally.states_reused == 0,
        "CPU 那一趟从空库起、每个状态都是这一趟判的，不是复用的：{:?}",
        cpu_run.tally
    );
    assert_eq!(
        gpu_run.red_items(),
        Vec::<String>::new(),
        "GPU 那一趟都判得了"
    );
    assert_eq!(cpu_run.red_items(), Vec::<String>::new());
    assert!(
        gpu_report.has_the_same_conclusions_as(&cpu_report),
        "GPU 那一趟与 CPU 那一趟的报告整份逐样相同（连同 I-1.13；镜像只比落没落盘、不比路径：两趟的镜像目录各是自己新建的那一份）：\nGPU 那一趟：\n{}\nCPU 那一趟：\n{}",
        gpu_report.render_the_conclusions(),
        cpu_report.render_the_conclusions()
    );
}

/// 上一条的判别力：往 GPU 那一侧塞一个已知故障（每张表第一个状态的第一条不变量违例位翻过来），GPU 那一趟的报告与 CPU 那一趟不同。
/// 两份报告同上一条一样整份比：两趟不塞故障时整份相同（上一条），塞了这条不绿，证得出上一条的比较抓得到 GPU 那一侧的错。
#[cfg(feature = "gpu")]
#[test]
fn a_fault_put_on_the_gpu_side_of_a_small_campaign_makes_the_gpu_run_differ_from_the_cpu_run() {
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    if gpu_card_pool_or_report_not_run(
        "a_fault_put_on_the_gpu_side_of_a_small_campaign_makes_the_gpu_run_differ_from_the_cpu_run",
    )
    .is_none()
    {
        return;
    }
    let (gpu_report, _gpu_run) = small_campaign_judged_on_the_gpu(
        "fault",
        FaultPutOnTheGpuSide::FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable,
    );
    let (cpu_report, _cpu_run) = small_campaign_judged_on_the_cpu("fault");
    assert!(
        !gpu_report.has_the_same_conclusions_as(&cpu_report),
        "GPU 那一侧塞了故障，两趟的报告却整份相同（不比镜像路径：路径不同不算不同）：\n{}",
        gpu_report.render_the_conclusions()
    );
}

/// 走引擎、照本机配置在 CPU 上判（这一趟自己写一份 `ENABLE_GPU=0` 的多机配置）的小规模一批：两段、每段 8 步、每段 2 个崩溃状态，两条线程。
fn small_campaign_judged_on_the_cpu_through_the_engine() -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 2,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(2),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    }
}

/// 在 `library` 那一份一次性库上（设成这个进程开的那一份）、环境守卫里（清掉一趟读的变量、设 `settings`、跑完还原）经引擎跑一批。
fn campaign_through_the_engine_with(
    campaign: &CrashInjectionCampaign,
    library: &singlefs_verification::crash_library::DisposableLibrary,
    settings: &[(&str, &std::ffi::OsStr)],
) -> (CrashInjectionReport, GivenStatesRun) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    let _the_library_of_this_process = library.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(settings);
    let judged = campaign_through_the_given_states_engine(campaign, FaultPutOnTheGpuSide::NoFault);
    drop(environment);
    judged
}

/// 叫停（给定状态引擎补的第 ② 样，照流水线那一侧的叫停层）：叫停文件在，崩溃注入一片都不领，报告记两段历史都没领、引擎那一趟报叫停了；
/// 叫停文件删掉再跑同一批，两段都领了、没叫停。
/// 判别力：崩溃注入领片之前不问叫停（`crates/mutations.tsv` 那一行），叫停文件在也两段都跑了。
#[test]
fn a_campaign_whose_stop_file_is_there_takes_no_slice_and_reports_every_history_left() {
    let campaign = small_campaign_judged_on_the_cpu_through_the_engine();
    let (directory, configuration) = disposable_library_with_a_configuration_on_the_cpu("stop");
    let stop_file = directory.join("stop");
    std::fs::write(&stop_file, "").expect("写得了叫停文件");
    let settings = [
        ("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str()),
        (
            "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE",
            stop_file.as_os_str(),
        ),
    ];
    let (stopped_report, stopped_run) =
        campaign_through_the_engine_with(&campaign, &directory, &settings);
    assert_eq!(
        stopped_run.library_name,
        directory
            .open_store(
                singlefs_verification::verdict_store::PruneServerFailureHandling::ReturnTheError
            )
            .expect("开得了一次性库")
            .library_name(),
        "引擎开的是这条用例的一次性库、不碰全仓那一份"
    );
    assert!(stopped_run.stopped_early, "叫停文件在：引擎那一趟报叫停了");
    assert_eq!(
        (
            stopped_report.histories_left_by_the_stop_request,
            stopped_report.tally.histories
        ),
        (campaign.seed_count, 0),
        "叫停文件在：一片都不领，两段历史都没领"
    );
    std::fs::remove_file(&stop_file).expect("删得了叫停文件");
    let (whole_report, whole_run) =
        campaign_through_the_engine_with(&campaign, &directory, &settings);
    assert!(!whole_run.stopped_early, "叫停文件不在：没叫停");
    assert_eq!(
        (
            whole_report.histories_left_by_the_stop_request,
            whole_report.tally.histories
        ),
        (0, campaign.seed_count),
        "叫停文件不在：两段都领了"
    );
    drop(directory);
}

/// 两台分片（给定状态引擎补的第 ① 样，H5「配了双机就两台分片」，照流水线那一侧的双机做法）：同一批先整份在一台上判一遍；
/// 再分两份各在一份空库上判（`SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2`、`1/2`），两份判的表加起来恰好是整份判的那么多，每一份判的加留给另一台的
/// 也是那么多；最后两份先后判进同一份库（两台都直接写服务那一份），第 0 份再跑一趟：一张都不现判、一张都不留，报告与整份那一趟逐字相同。
/// 判别力：每张表都归这台机器判（`crates/mutations.tsv` 那一行），两份各判整份，加起来是两倍。
#[test]
fn two_shares_judged_into_one_library_judge_every_table_once_and_render_the_report_of_one_host() {
    let campaign = small_campaign_judged_on_the_cpu_through_the_engine();
    let (_directory, configuration) = disposable_library_with_a_configuration_on_the_cpu("shares");
    let run_on = |library: &singlefs_verification::crash_library::DisposableLibrary,
                  share: Option<&str>| {
        let mut settings: Vec<(&str, &std::ffi::OsStr)> =
            vec![("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str())];
        if let Some(share) = share {
            settings.push((
                "SINGLEFS_CRASH_AMPLIFICATION_SHARE",
                std::ffi::OsStr::new(share),
            ));
        }
        campaign_through_the_engine_with(&campaign, library, &settings)
    };
    let (whole_report, whole_run) =
        run_on(&disposable_library_of_the_campaign("shares-whole"), None);
    // 整份：现判的加同一趟里读回的（同一批里内容相同的表，第二张照库里的复用）
    let every_table = whole_run.tally.tables_judged_here + whole_run.tally.tables_reused;
    assert!(
        whole_run.tally.tables_judged_here > 0,
        "前提：整份那一趟判了表"
    );
    let (_, first_share) = run_on(&disposable_library_of_the_campaign("shares-0"), Some("0/2"));
    let (_, second_share) = run_on(&disposable_library_of_the_campaign("shares-1"), Some("1/2"));
    assert_eq!(
        first_share.tally.tables_judged_here
            + first_share.tally.tables_reused
            + second_share.tally.tables_judged_here
            + second_share.tally.tables_reused,
        every_table,
        "两份判的（连同各自趟里读回的）加起来恰好是整份的：一张表只归一台"
    );
    for run in [&first_share, &second_share] {
        assert_eq!(
            run.tally.tables_judged_here
                + run.tally.tables_reused
                + run.tally.tables_left_to_the_other_hosts,
            every_table,
            "每一份判的加留给另一台的是整份的（分片 {}/{}）",
            run.share.index,
            run.share.count
        );
    }
    let one_library = disposable_library_of_the_campaign("shares-into-one-library");
    let _ = run_on(&one_library, Some("0/2"));
    let _ = run_on(&one_library, Some("1/2"));
    let (report_after_both_shares, run_after_both_shares) = run_on(&one_library, Some("0/2"));
    assert_eq!(
        (
            run_after_both_shares.tally.tables_judged_here,
            run_after_both_shares.tally.tables_left_to_the_other_hosts,
            run_after_both_shares.tally.tables_reused
        ),
        (0, 0, every_table),
        "两份都判进同一份库之后：一张都不现判、一张都不留，全照库里的"
    );
    assert_eq!(
        report_after_both_shares.render_the_conclusions(),
        whole_report.render_the_conclusions(),
        "两份都判进同一份库之后那一趟的报告与整份在一台上判的逐字相同"
    );
}

/// 创世发布上 GPU（先查 GPU 能不能核，不能就 CPU 核一遍；GPU 核得到）：一段历史跑完、每个写都落了的那一态，
/// 与同一张写表末尾再种两个盖掉创世树表单元两份副本的单元写、也都落了的那一态，交配了 GPU 的引擎判（只由 GPU 判器判）。
/// GPU 判器的记录核对器在种了的那一态上判「恢复自称的状态缺了一次发布的单元」，没种的那一态不红；两态 GPU 都判得了。
/// 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
/// 判别力：给定状态表的写表把 mkfs 的写留在表外，创世发布不进 GPU 的发布表，种了也不红（`crates/mutations.tsv` 那一行）。
#[cfg(feature = "gpu")]
#[test]
fn a_genesis_unit_overwritten_by_an_illegal_reuse_is_red_on_the_gpu_record_checker() {
    use singlefs_verification::crash_amplification::{CrashAmplificationEnvironment, JudgedBy};
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    use singlefs_verification::memory_pool::WrittenContents;
    const TEST_NAME: &str =
        "a_genesis_unit_overwritten_by_an_illegal_reuse_is_red_on_the_gpu_record_checker";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let history = history_of_a_first_file_and_two_overwrites();
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    // 创世树表单元的两份副本：mkfs 单元写里偏移最大的那一组（实例表在单元区起点、树表紧跟它）
    let genesis_units: Vec<&RetainedWrite> = history_after_the_make_filesystem
        .writes_of_the_make_filesystem()
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .collect();
    let tree_table_offset = genesis_units
        .iter()
        .map(|write| write.offset)
        .max()
        .expect("mkfs 写了实例表与树表两个单元");
    let overwrites: Vec<RetainedWrite> = genesis_units
        .iter()
        .filter(|write| write.offset == tree_table_offset)
        .map(|copy| RetainedWrite {
            device: copy.device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: copy.offset,
            contents: WrittenContents::Bytes(vec![
                0xA5;
                usize::try_from(copy.length_in_bytes())
                    .expect("单元长装得进 usize")
            ]),
        })
        .collect();
    assert_eq!(overwrites.len(), 2, "前提：创世树表单元两块盘各一份副本");
    let healthy_write_count = history_after_the_make_filesystem.writes().len();
    let table_writes: Vec<RetainedWrite> = history_after_the_make_filesystem
        .writes()
        .iter()
        .chain(&overwrites)
        .cloned()
        .collect();
    let mut table_segments = history_after_the_make_filesystem.segments().to_vec();
    table_segments.push((healthy_write_count..table_writes.len()).collect());
    let mut healthy = vec![true; table_writes.len()];
    healthy[healthy_write_count..].fill(false);
    let planted = vec![true; table_writes.len()];
    let table = GivenStatesOfATable {
        table_name: "first-file-and-two-overwrites/genesis-tree-table-overwritten".to_string(),
        base: history_after_the_make_filesystem.base_pool(),
        writes: &table_writes,
        segments: &table_segments,
        seams: &[],
        persisted_sets: vec![healthy, planted],
    };
    // 库是这条用例自己在进程里起的临时 prune-server 里的一段一次性名字空间（设成这个进程开的那一份）：不连真服务
    let directory = disposable_library_of_the_campaign("genesis-publish-on-the-gpu");
    let configuration = directory.join("multi-host-with-gpu.env");
    write_a_configuration_with_the_gpu_enabled(&configuration);
    let cards = cards_of_the_caller();
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(
        &settings_of_an_engine_on_the_gpu(&configuration, &cards),
    );
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!(
                "{}::history_of_a_first_file_and_two_overwrites genesis-tree-table-overwritten",
                CRASH_POINT_REPLAY_ITEM
            ),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("配了 GPU 的引擎起得来（卡用不上在这里红，不退回 CPU）");
    let conclusions = engine
        .judge_the_given_states(&table)
        .expect("配了 GPU 的引擎判得了这张表");
    let engine_run = engine.finish();
    drop(environment);
    drop(the_library_of_this_process);
    drop(directory);
    assert_eq!(
        engine_run.judged_by,
        JudgedBy::GpuJudge,
        "配了 GPU：GPU 判器判"
    );
    assert_eq!(
        engine_run.red_items(),
        Vec::<String>::new(),
        "两态 GPU 都判得了"
    );
    assert_eq!(
        conclusions.states[0].record_check(),
        RecordCheck::default(),
        "没种的那一态：GPU 的记录核对器不红"
    );
    assert!(
        conclusions.states[1]
            .record_check()
            .claimed_state_missing_unit,
        "种了的那一态：GPU 的记录核对器判创世发布的单元缺了"
    );
}

/// 给定状态表的替身判器：照引擎判（表与结论进给定状态表那一类的判定库），每张表的结论都标成「两台分片里归别的机器判、交的占位结论」，数它判了几张表。
struct EveryTableLeftToAnotherHost {
    tables: std::sync::atomic::AtomicU64,
    engine: GivenStatesEngine,
}

impl EveryTableLeftToAnotherHost {
    fn judging_with(engine: GivenStatesEngine) -> Self {
        Self {
            tables: std::sync::atomic::AtomicU64::new(0),
            engine,
        }
    }
}

impl GivenStatesJudge for EveryTableLeftToAnotherHost {
    fn stop_is_requested(&self) -> bool {
        false
    }

    fn judge_the_given_states(
        &self,
        table: &GivenStatesOfATable<'_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        self.tables
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut conclusions = self.engine.judge_the_given_states(table)?;
        conclusions.source = GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost;
        Ok(conclusions)
    }

    /// 带两栏的表（第二截那张）同样照引擎判、标成占位的：引擎照两栏判，不走没重写的那一份。
    fn judge_the_given_states_with_its_columns(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        self.tables
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut conclusions = self
            .engine
            .judge_the_given_states_with_its_columns(table_with_its_columns)?;
        conclusions.source = GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost;
        Ok(conclusions)
    }
}

/// 占位结论在报告里标出来（E4 报告第三节第 5 条）：判器交了占位结论的表（两台分片、归别的机器判、库里又没有），崩溃注入数进报告的计数，
/// `render()` 多一行注明「这一批不作数」；就地 CPU 判的同一批一张不记、没有那一行。不要特性：判器是替身（照 CPU 就地判、每张都标成占位的）。
/// 两个引擎都在环境守卫里照一份单机 CPU 配置起，不跟调用方的分片走：崩溃点重放入口两台分片跑时调用方给了 `SINGLEFS_CRASH_AMPLIFICATION_SHARE`，
/// 照它起的引擎把归别的机器判的表也交成占位结论，就地那一趟就不是「一张都不是占位的」。
/// 判别力：崩溃注入不数占位结论的表（`crates/mutations.tsv` 那一行），记 0、`render()` 没那一行。
#[test]
fn tables_left_to_another_host_with_placeholder_conclusions_are_counted_in_the_report_and_noted_in_its_render(
) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    let (directory, configuration) =
        disposable_library_with_a_configuration_on_the_cpu("placeholder-note");
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let engine_judging_here =
        given_states_engine_under_the_callers_environment_guard("placeholder_note_judged_here");
    let judge = EveryTableLeftToAnotherHost::judging_with(
        given_states_engine_under_the_callers_environment_guard("placeholder_probe"),
    );
    drop(environment);
    drop(the_library_of_this_process);
    let campaign = CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 2,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(2),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    };
    let judged_here =
        run_crash_injection_campaign_judging_the_given_states_with(&campaign, &engine_judging_here);
    assert_eq!(
        judged_here.tally.given_states_tables_left_to_other_hosts, 0,
        "就地 CPU 判的：一张都不是占位的"
    );
    assert!(
        !judged_here
            .render()
            .contains(GIVEN_STATES_TABLES_LEFT_TO_OTHER_HOSTS_NOTE),
        "就地 CPU 判的：render 里没有占位那一行"
    );
    let left = run_crash_injection_campaign_judging_the_given_states_with(&campaign, &judge);
    let tables = judge.tables.load(std::sync::atomic::Ordering::Relaxed);
    assert!(tables > 0, "前提：这一批建了给定状态表");
    assert_eq!(
        left.tally.given_states_tables_left_to_other_hosts, tables,
        "判器交了占位结论的每一张表都数进报告（第一截与第三截都算）"
    );
    assert!(
        left.render().contains(&format!(
            "{GIVEN_STATES_TABLES_LEFT_TO_OTHER_HOSTS_NOTE}：{tables} 张"
        )),
        "render 注明有几张表交的是占位结论：{}",
        left.render()
    );
    let library_of_the_engine_judging_here = engine_judging_here.finish().library_name;
    assert_eq!(
        library_of_the_engine_judging_here,
        directory
            .open_store(
                singlefs_verification::verdict_store::PruneServerFailureHandling::ReturnTheError
            )
            .expect("开得了一次性库")
            .library_name(),
        "就地判的那个引擎开的是这条用例的一次性库、不碰全仓那一份"
    );
}

/// 同一个进程里同时起两个引擎（两条用例并行跑撞同一份库）：两个都开这个进程的那一份（同一份一次性库，共用这个进程里开着的那一份连接），
/// 两个都起得来，先判的那个把库里还没有的表现判、后判的那个同一批全照库里的复用，两份报告相同。CPU 配置，不要卡。
/// 判别力：存表的结论不写进服务、只记在先判那个引擎那一端（后判的那个读不到），这条红在复用那一句。
#[test]
fn two_engines_started_at_once_in_one_process_both_judge_on_the_one_library_of_the_process() {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    let campaign = small_campaign_judged_on_the_cpu_through_the_engine();
    let (directory, configuration) =
        disposable_library_with_a_configuration_on_the_cpu("two-engines");
    let start = || {
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: format!(
                    "{}::{}",
                    CRASH_POINT_REPLAY_ITEM,
                    seeds_and_scale_of(&campaign)
                ),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
    };
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let first = start().unwrap_or_else(|refusal| panic!("第一个引擎起得来：{refusal}"));
    let second = start().unwrap_or_else(|refusal| {
        panic!("第一个还开着时第二个也起得来（共用这个进程里开着的那一份库）：{refusal}")
    });
    let first_report =
        run_crash_injection_campaign_judging_the_given_states_with(&campaign, &first);
    let second_report =
        run_crash_injection_campaign_judging_the_given_states_with(&campaign, &second);
    let first_run = first.finish();
    let second_run = second.finish();
    drop(environment);
    drop(the_library_of_this_process);
    assert_eq!(
        first_run.library_name, second_run.library_name,
        "两个引擎开的是同一份库"
    );
    let every_table = first_run.tally.tables_judged_here + first_run.tally.tables_reused;
    assert!(every_table > 0, "前提：这一批有表");
    assert_eq!(
        (
            second_run.tally.tables_judged_here,
            second_run.tally.tables_reused
        ),
        (0, every_table),
        "后判的那个全照同一份库里的复用"
    );
    assert_eq!(
        first_report.render_the_conclusions(),
        second_report.render_the_conclusions(),
        "复用的那一趟报告与现判的相同"
    );
}

/// 第二截那张表的一种摆法（[`the_gpu_record_checker_after_the_writable_mount_agrees_with_the_cpu_on_the_crash_state_image_and_the_recovered_image`]）：
/// 第一次崩溃状态（整条历史写表上的持久集合）、挂载那一段的写与切段、之后那一次发布的写与切段，与它在表名里的名字。
#[cfg(feature = "gpu")]
struct FirstCrashAndTheWritableMountAfterIt {
    name: &'static str,
    persisted_over_the_whole_history: Vec<bool>,
    mount_writes: Vec<RetainedWrite>,
    mount_segments: Vec<Vec<usize>>,
    publish_writes: Vec<RetainedWrite>,
    publish_segments: Vec<Vec<usize>>,
}

/// 手摆的那一种：最后一次带 journal 记录的那次发布，根槽写落了、记录写一个都没落；挂载那一段只有一个写，把那次发布的根槽盖成同一块盘上
/// 更早写的一条根（历史那次发布的根槽被挂载的写覆盖），之后没有发布。两份镜像判第一条判据红、只读挂载之后的池时放过
/// （`tests/crash_injection_writable_mount_after_the_crash.rs` 里同一种摆法）。
#[cfg(feature = "gpu")]
fn the_root_of_a_history_publish_overwritten_by_the_mount(
    writes: &[RetainedWrite],
) -> FirstCrashAndTheWritableMountAfterIt {
    let every_write = vec![true; writes.len()];
    let last_publish_with_records = singlefs_verification::memory_pool::publishes_in(
        writes,
        &every_write,
        RecordStreamContinuity::OneRecording,
    )
    .into_iter()
    .rev()
    .find(|publish| !publish.records.is_empty())
    .expect("前提：覆盖写那几次发布写了 journal 记录");
    let overwritten_root = &writes[last_publish_with_records.root];
    let earlier_root_on_the_same_device = writes[..last_publish_with_records.root]
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::RootRecordFua
                && write.device == overwritten_root.device
                && write.length_in_bytes() == overwritten_root.length_in_bytes()
        })
        .expect("前提：那次发布之前同一块盘上写过别的根");
    let mut persisted_over_the_whole_history = every_write;
    for record in &last_publish_with_records.records {
        persisted_over_the_whole_history[*record] = false;
    }
    FirstCrashAndTheWritableMountAfterIt {
        name: "root-of-the-last-publish-with-records-overwritten-by-the-mount",
        persisted_over_the_whole_history,
        mount_writes: vec![RetainedWrite {
            device: overwritten_root.device,
            kind: StepKind::RootRecordFua,
            is_force_unit_access: true,
            offset: overwritten_root.offset,
            contents: earlier_root_on_the_same_device.contents.clone(),
        }],
        mount_segments: vec![vec![0]],
        publish_writes: Vec::new(),
        publish_segments: Vec::new(),
    }
}

/// 真跑的那一种：第一次崩溃摆在最后一条根槽写所在的段上、只扣下那一条（更早的段整段落、更晚的段一个都没落），在那份崩溃后镜像上
/// 真起一次可写挂载、再发一次布（崩溃注入第二截同一个入口）。
#[cfg(feature = "gpu")]
fn the_last_root_withheld_then_the_writable_mount(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) -> FirstCrashAndTheWritableMountAfterIt {
    use singlefs_verification::crash_injection::{
        materialized_crash_image, mount_and_publish_once_on_the_crash_image, WritableMountOutcome,
    };
    let last_root = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("历史写过根槽");
    let segment_index = segments
        .iter()
        .position(|segment| segment.contains(&last_root))
        .expect("每个写都在某一段里");
    let segment = &segments[segment_index];
    let first_write_of_the_segment = segment[0];
    let persisted_within_the_segment: Vec<bool> =
        segment.iter().map(|write| *write != last_root).collect();
    let mut persisted_over_the_whole_history = vec![false; writes.len()];
    persisted_over_the_whole_history[..first_write_of_the_segment].fill(true);
    persisted_over_the_whole_history
        [first_write_of_the_segment..first_write_of_the_segment + segment.len()]
        .copy_from_slice(&persisted_within_the_segment);
    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    base.apply_writes(&writes[..first_write_of_the_segment]);
    let crash_point = CrashPoint {
        segment_index,
        persisted_within_the_segment: persisted_within_the_segment.clone(),
        step: StepPosition::StartingPoint,
        operation_kind: None,
    };
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &materialized_crash_image(&CrashImage {
            base: &base,
            writes: &writes[first_write_of_the_segment..first_write_of_the_segment + segment.len()],
            persisted: persisted_within_the_segment,
        }),
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &crash_point,
    );
    assert!(
        matches!(mount_run.outcome, WritableMountOutcome::Published),
        "前提：崩溃之后的可写挂载与之后那一次发布都做成：{:?}",
        mount_run.outcome
    );
    let geometry = execution.device_width.fixed_geometry();
    let (mount_writes, mount_segments) =
        writes_and_segments(&mount_run.mount_operations, &geometry);
    let (publish_writes, publish_segments) =
        writes_and_segments(&mount_run.publish_operations, &geometry);
    FirstCrashAndTheWritableMountAfterIt {
        name: "last-root-withheld-then-the-writable-mount",
        persisted_over_the_whole_history,
        mount_writes,
        mount_segments,
        publish_writes,
        publish_segments,
    }
}

/// GPU 对拍的 CPU 参照：第二截那张表经对拍那一类的库读回（`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`），
/// 第一次崩溃那份镜像（崩溃态镜像）与挂载之后的池（实现恢复后的镜像）各从读回的那一份叠出来，记录核对器两份镜像一次判
/// （`crash::check_records_against`：第一条判据读崩溃态镜像、第二条读恢复后的镜像）。恢复自称的那一版照 D13（验证路线） 已定项 7 的字面从读回的
/// 那一份现取：崩溃那道接缝之后（挂载与那次发布）写出的最新那条根，一条都没有时退回接缝上记的第一次崩溃落到的那一版
/// （`crash_injection::newest_root_written_after_the_crash`，判器那一侧照表里那一栏取的同一个）；它与恢复后的镜像上看 journal 那一遍落到的根
/// 不同另记一位。交回（记录核对器两条判据，那一位）；只录那一趟交 None。
// cpu-reference-for-gpu-comparison
#[cfg(feature = "gpu")]
fn cpu_record_check_on_the_two_images_read_back_from_the_pairing_library(
    table: &singlefs_verification::crash_injection::GivenStatesTableAfterTheFirstCrash<'_>,
) -> Option<(RecordCheck, bool)> {
    use singlefs_verification::crash_amplification::RecordedGivenStatesTable;
    use singlefs_verification::crash_injection::{
        materialized_crash_image, newest_root_written_after_the_crash,
        StateOfTheTableAfterTheWritableMount,
    };
    use singlefs_verification::gpu_unit_checks::gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library;
    use singlefs_verification::memory_pool::RecordStreamSeam;
    let read_back = cpu_reference_table_read_back_from_the_pairing_library(
        &RecordedGivenStatesTable::of(&table.given_states_table()),
    )?;
    Some(read_back.with_the_table(|read_back_table| {
        let image_of = |state: StateOfTheTableAfterTheWritableMount| {
            materialized_crash_image(&CrashImage {
                base: read_back_table.base,
                writes: read_back_table.writes,
                persisted: read_back_table.persisted_sets[state.position()].clone(),
            })
        };
        let crash_state_image = image_of(StateOfTheTableAfterTheWritableMount::FirstCrashImage);
        let recovered_image = image_of(StateOfTheTableAfterTheWritableMount::PoolAfterTheMount);
        let (first_write_after_the_crash, version_landed_on_after_the_crash) = read_back_table
            .seams
            .iter()
            .find_map(|seam| match seam {
                RecordStreamSeam::AfterACrash {
                    first_write_after_the_crash,
                    version_landed_on_after_the_crash,
                } => Some((
                    *first_write_after_the_crash,
                    *version_landed_on_after_the_crash,
                )),
                RecordStreamSeam::AfterAPowerLoss { .. } => None,
            })
            .expect("第二截那张表在历史与挂载之间有一道崩溃接缝");
        let claimed_version = newest_root_written_after_the_crash(
            &read_back_table.writes[first_write_after_the_crash..],
            version_landed_on_after_the_crash,
        );
        let landed_on = recover(&recovered_image, JournalPolicy::Consult).effective_root;
        (
            check_records_against(
                &crash_state_image,
                &recovered_image,
                read_back_table.writes,
                &read_back_table.persisted_sets
                    [StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position()],
                read_back_table.continuity(),
                claimed_version,
            ),
            claimed_version != landed_on,
        )
    }))
}

/// 第二截记录核对器两份镜像的输入上 GPU 与 CPU 对拍（欠账 C832，D13（验证路线） 已定项 7「崩溃后镜像是两份」）：第一个文件之后覆盖写两次的历史上
/// 摆两种第一次崩溃状态（手摆的「历史那次发布的根槽被挂载的写覆盖」、真跑的「扣下最后一条根槽写再可写挂载」），各建第二截那张两个状态的表，
/// 交配了 GPU 的引擎判（只由 GPU 判器判），两个状态的结论合成第二截记录核对器的两条判据；CPU 一侧从对拍那一类的库读回同一张表，
/// 叠出两份镜像、记录核对器两份镜像一次判。两侧逐条相同；手摆那一态两份镜像判第一条判据红（对拍的输入里真有一态分得开两种读法）；
/// GPU 判器两态都判得了。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
// crash-case-check:not-a-crash-case 只判写死的两个第一次崩溃状态（手摆一个、扣下最后一条根槽写一个），不枚举崩溃状态
#[cfg(feature = "gpu")]
#[test]
fn the_gpu_record_checker_after_the_writable_mount_agrees_with_the_cpu_on_the_crash_state_image_and_the_recovered_image(
) {
    use singlefs_verification::crash_amplification::{CrashAmplificationEnvironment, JudgedBy};
    use singlefs_verification::crash_injection::GivenStatesAfterTheFirstCrash;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "the_gpu_record_checker_after_the_writable_mount_agrees_with_the_cpu_on_the_crash_state_image_and_the_recovered_image";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let history = history_of_a_first_file_and_two_overwrites();
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    let first_crashes = [
        the_root_of_a_history_publish_overwritten_by_the_mount(&writes),
        the_last_root_withheld_then_the_writable_mount(&history, execution, &writes, &segments),
    ];
    // 库是这条用例自己在进程里起的临时 prune-server 里的一段一次性名字空间（设成这个进程开的那一份，CPU 参照经对拍那一类读回的也是它）：
    // 不连真服务
    let directory = disposable_library_of_the_campaign("record-checker-after-the-mount-on-the-gpu");
    let configuration = directory.join("multi-host-with-gpu.env");
    write_a_configuration_with_the_gpu_enabled(&configuration);
    let cards = cards_of_the_caller();
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(
        &settings_of_an_engine_on_the_gpu(&configuration, &cards),
    );
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!("{}::history_of_a_first_file_and_two_overwrites record-checker-after-the-writable-mount", CRASH_POINT_REPLAY_ITEM),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("配了 GPU 的引擎起得来（卡用不上在这里红，不退回 CPU）");
    let mut compared = Vec::new();
    for first_crash in &first_crashes {
        let version_landed_on_after_the_first_crash = recover(
            &CrashImage {
                base: history_after_the_make_filesystem.base_pool(),
                writes: history_after_the_make_filesystem.writes(),
                persisted: history_after_the_make_filesystem
                    .persisted_set_of_the_table(&first_crash.persisted_over_the_whole_history),
            },
            JournalPolicy::Consult,
        )
        .effective_root;
        let given = GivenStatesAfterTheFirstCrash::new(
            &engine,
            &history_after_the_make_filesystem,
            &first_crash.persisted_over_the_whole_history,
            &[],
            version_landed_on_after_the_first_crash,
            &first_crash.publish_writes,
            &first_crash.publish_segments,
        );
        let table_name = format!("first-file-and-two-overwrites/{}", first_crash.name);
        let conclusions_on_the_gpu = given.judge_the_pool_after_the_writable_mount(
            &first_crash.mount_writes,
            &first_crash.mount_segments,
            table_name.clone(),
        );
        let gpu = (
            conclusions_on_the_gpu.record_check(),
            conclusions_on_the_gpu
                .on_the_pool_after_the_mount
                .claimed_version_is_not_landed_on(),
        );
        let Some(cpu) = cpu_record_check_on_the_two_images_read_back_from_the_pairing_library(
            &given.table_after_the_writable_mount(
                &first_crash.mount_writes,
                &first_crash.mount_segments,
                table_name,
            ),
        ) else {
            return;
        };
        assert_eq!(
            gpu, cpu,
            "{}：GPU 判器两个状态合成的第二截记录核对器（连同恢复落到的根是不是挂载与那次发布写出的最新那条根）与 CPU 两份镜像一次判的逐条相同",
            first_crash.name
        );
        compared.push((first_crash.name, cpu.0));
    }
    let engine_run = engine.finish();
    drop(environment);
    drop(the_library_of_this_process);
    drop(directory);
    assert_eq!(
        engine_run.judged_by,
        JudgedBy::GpuJudge,
        "配了 GPU：GPU 判器判"
    );
    assert_eq!(
        engine_run.red_items(),
        Vec::<String>::new(),
        "两张表的四个状态 GPU 都判得了"
    );
    assert!(
        compared
            .iter()
            .any(|(name, record_check)| *name == first_crashes[0].name
                && record_check.root_without_record),
        "手摆那一态两份镜像判第一条判据红：{compared:?}"
    );
}

// ─── 第二截恢复自称的那一版照 D13（验证路线） 已定项 7 的字面取（照字面改：表里记下根的来路） ───

/// 只建表、不判的判器：[`second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written`] 拿 `GivenStatesAfterTheFirstCrash`
/// 建第二截那张表，表不交它判。
struct GivenStatesTableBuiltOnly;

impl GivenStatesJudge for GivenStatesTableBuiltOnly {
    fn stop_is_requested(&self) -> bool {
        false
    }

    fn judge_the_given_states(
        &self,
        table: &GivenStatesOfATable<'_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        Err(format!(
            "这个判器只用来建第二截那张表，表 {} 不该交它判",
            table.table_name
        ))
    }
}

/// 手摆一张第二截的表，挂载之后的池那个状态上恢复落到挂载与那次发布写出的最新那条根之下：第一个文件之后覆盖写两次的历史整条落了、
/// 不崩，在那份镜像上真起一次可写挂载、再发一次布（崩溃注入第二截同一个入口），建第二截那张表（挂载之后的池那个状态那一栏写那次发布的根）；
/// 再把挂载之后的池那个状态里之后那次发布的 journal 记录、它的根槽写与之后的写、它的一个单元（那个偏移上的每一份副本）都改成没落
/// （判段之前把那条根与它的记录从镜像上拿掉：恢复落到挂载最后写的那条根）。交回读回用的那一份（表连同两栏）与那一栏写的那条根。
fn second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written() -> (
    singlefs_verification::crash_amplification::RecordedGivenStatesTableWithItsColumns,
    (InstanceGeneration, CheckpointTxg),
) {
    use singlefs_checker::walk::ContainerCapacityKnobs;
    use singlefs_verification::crash_amplification::{
        RecordedGivenStatesTable, RecordedGivenStatesTableWithItsColumns,
    };
    use singlefs_verification::crash_injection::{
        materialized_crash_image, mount_and_publish_once_on_the_crash_image,
        ClaimedVersionOfAGivenState, GivenStatesAfterTheFirstCrash,
        StateOfTheTableAfterTheWritableMount, WritableMountOutcome,
    };
    let history = history_of_a_first_file_and_two_overwrites();
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    let every_history_write = vec![true; writes.len()];
    let version_landed_on_after_the_first_crash = recover(
        &CrashImage {
            base: history_after_the_make_filesystem.base_pool(),
            writes: history_after_the_make_filesystem.writes(),
            persisted: history_after_the_make_filesystem
                .persisted_set_of_the_table(&every_history_write),
        },
        JournalPolicy::Consult,
    )
    .effective_root;
    let last_segment_index = segments.len() - 1;
    let last_segment = &segments[last_segment_index];
    let first_write_of_the_last_segment = last_segment[0];
    let mut base_before_the_last_segment = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    base_before_the_last_segment.apply_writes(&writes[..first_write_of_the_last_segment]);
    let crash_point = CrashPoint {
        segment_index: last_segment_index,
        persisted_within_the_segment: vec![true; last_segment.len()],
        step: StepPosition::StartingPoint,
        operation_kind: None,
    };
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &materialized_crash_image(&CrashImage {
            base: &base_before_the_last_segment,
            writes: &writes[first_write_of_the_last_segment
                ..first_write_of_the_last_segment + last_segment.len()],
            persisted: vec![true; last_segment.len()],
        }),
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &crash_point,
    );
    assert!(
        matches!(mount_run.outcome, WritableMountOutcome::Published),
        "前提：可写挂载与之后那一次发布都做成：{:?}",
        mount_run.outcome
    );
    let geometry = execution.device_width.fixed_geometry();
    let (mount_writes, mount_segments) =
        writes_and_segments(&mount_run.mount_operations, &geometry);
    let (publish_writes, publish_segments) =
        writes_and_segments(&mount_run.publish_operations, &geometry);
    let given = GivenStatesAfterTheFirstCrash::new(
        &GivenStatesTableBuiltOnly,
        &history_after_the_make_filesystem,
        &every_history_write,
        &[],
        version_landed_on_after_the_first_crash,
        &publish_writes,
        &publish_segments,
    );
    let mut table = given.table_after_the_writable_mount(
        &mount_writes,
        &mount_segments,
        "first-file-and-two-overwrites/whole-history-landed/pool-after-the-mount-lands-below-the-newest-root-written"
            .to_string(),
    );
    let pool_after_the_mount = StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position();
    let ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some(claimed_root)) =
        table.claimed_versions[pool_after_the_mount]
    else {
        panic!(
            "挂载与那次发布写了根：挂载之后的池那个状态那一栏写那条根：{:?}",
            table.claimed_versions
        );
    };
    let publish_after_the_mount = singlefs_verification::memory_pool::publishes_in(
        &table.writes,
        &table.persisted_sets[pool_after_the_mount],
        RecordStreamContinuity::BrokenAtSeams(&table.seams),
    )
    .pop()
    .expect("写表里有发布");
    assert_eq!(
        (
            InstanceGeneration(publish_after_the_mount.instance),
            CheckpointTxg(publish_after_the_mount.checkpoint_txg)
        ),
        claimed_root,
        "前提：写表里最后那次发布就是挂载之后那一次，它的根就是那一栏写的那一条"
    );
    let first_unit_offset = publish_after_the_mount
        .units
        .iter()
        .map(|unit| table.writes[*unit].offset.0)
        .min()
        .expect("前提：挂载之后那一次发布写了单元");
    let persisted = &mut table.persisted_sets[pool_after_the_mount];
    for record in &publish_after_the_mount.records {
        persisted[*record] = false;
    }
    persisted[publish_after_the_mount.root..].fill(false);
    for unit in &publish_after_the_mount.units {
        if table.writes[*unit].offset.0 == first_unit_offset {
            persisted[*unit] = false;
        }
    }
    let recorded = RecordedGivenStatesTableWithItsColumns {
        table: RecordedGivenStatesTable::of(&table.given_states_table()),
        container_capacity_knobs: ContainerCapacityKnobs::PRODUCT,
        claimed_versions: table.claimed_versions.clone(),
    };
    (recorded, claimed_root)
}

/// 第二截恢复自称的那一版照字面取（D13（验证路线） 已定项 7「恢复自称的那一版取挂载与那次发布写出的最新那条根」）：手摆的那张表
/// （[`second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written`]）上，挂载之后的池那个状态恢复落到那条根之下。
/// 旧取法（第二条判据取恢复落到的根）放过：那次发布比恢复落到的那一版新，它缺的那个单元没人核，那一位也不置；照那一栏取（新取法）判红：
/// 第二条判据核到那次发布缺单元，「恢复落到的根不是挂载与那次发布写出的最新那条根」那一位也红。CPU 判器就地判一次，再经崩溃点重放的引擎
/// （两段：表连同两栏写进判定库、读回再判，照一份单机 CPU 配置）判一次，两次逐字相同。
/// 判别力（`crates/mutations.tsv`）：CPU 判器第二条判据改回取恢复落到的根、那一位恒不置、第二截建表时那一栏不写，各红这一条。
// crash-case-check:not-a-crash-case 一段两次覆盖写的历史上手摆一张两个状态的表，不枚举崩溃状态
#[test]
fn a_pool_after_the_mount_whose_recovery_lands_below_the_newest_root_written_is_red_only_under_the_claimed_version(
) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    use singlefs_verification::crash_injection::{
        judge_the_given_states_on_the_cpu, StateOfTheTableAfterTheWritableMount,
    };
    let (recorded, claimed_root) =
        second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written();
    let pool_after_the_mount = StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position();
    let (under_the_landed_root, under_the_claimed_version) =
        recorded.with_the_table(|table_with_its_columns| {
            (
                judge_the_given_states_on_the_cpu(table_with_its_columns.table),
                judge_the_given_states_on_the_cpu(table_with_its_columns),
            )
        });
    let landed_on = under_the_claimed_version.states[pool_after_the_mount]
        .words
        .consulted_pass_landed_root;
    assert_ne!(
        landed_on.landed_root(),
        Some(claimed_root),
        "前提：挂载之后的池那个状态上看 journal 那一遍恢复落到那一栏写的根 {claimed_root:?} 之下"
    );
    let old_reading = &under_the_landed_root.states[pool_after_the_mount];
    assert!(
        !old_reading.record_check().claimed_state_missing_unit
            && !old_reading.claimed_version_is_not_landed_on(),
        "旧取法（取恢复落到的根）放过：{:?}",
        old_reading.words
    );
    let literal_reading = &under_the_claimed_version.states[pool_after_the_mount];
    assert!(
        literal_reading.record_check().claimed_state_missing_unit,
        "照那一栏取的那一版判第二条判据：挂载之后那一次发布缺了一个单元，判红：{:?}",
        literal_reading.words
    );
    assert!(
        literal_reading.claimed_version_is_not_landed_on(),
        "恢复落到的根不是挂载与那次发布写出的最新那条根，判红：{:?}",
        literal_reading.words
    );
    let (directory, configuration) =
        disposable_library_with_a_configuration_on_the_cpu("claimed-version");
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let engine = given_states_engine_under_the_callers_environment_guard(
        "claimed_version_of_the_pool_after_the_mount",
    );
    let through_the_library = recorded
        .with_the_table(|table_with_its_columns| {
            engine.judge_the_given_states_with_its_columns(table_with_its_columns)
        })
        .unwrap_or_else(|reason| panic!("引擎判不了（不退回别处判）：{reason}"));
    let engine_run = engine.finish();
    drop(environment);
    drop(the_library_of_this_process);
    assert_eq!(
        engine_run.tally.tables_judged_here, 1,
        "这一趟从空库起、这张表现判：{:?}",
        engine_run.tally
    );
    assert_eq!(
        through_the_library
            .states
            .iter()
            .map(|state| state.words)
            .collect::<Vec<_>>(),
        under_the_claimed_version
            .states
            .iter()
            .map(|state| state.words)
            .collect::<Vec<_>>(),
        "经判定库两段判的与就地照那一栏判的逐字相同（两栏跟着表录进库、读回照它判）"
    );
}

/// 上一条那张表上 GPU 判器与 CPU 判器对拍（两趟独立、各从自己那一份库读回）：一趟配了 GPU（只由 GPU 判器判，恢复内核照判表里那一栏取那一版），
/// 一趟没开 GPU（CPU 判器），两个状态的对拍字经 `cross_check_words_compared_by_the_cross_check` 去掉内核不置位、对拍不比的那几位
/// （`.claude/kb/checks-owed.md` C821（GPU 判器没有扫描重建内核） 的 I-1.13）之后逐字相同，挂载之后的池那个状态两边都判第二条判据红、那一位红。要本机一张 NVIDIA 独显：
/// 一张适配器都没有时报「本次未跑」。判别力（`crates/mutations.tsv`）：恢复内核第二条判据改回取看 journal 那一遍落到的根，GPU 那一趟放过，这条红。
// crash-case-check:not-a-crash-case 一段两次覆盖写的历史上手摆一张两个状态的表，不枚举崩溃状态
#[cfg(feature = "gpu")]
#[test]
fn the_gpu_judges_the_claimed_version_of_the_pool_after_the_mount_like_the_cpu_when_the_recovery_lands_below_it(
) {
    use singlefs_verification::crash_amplification::{CrashAmplificationEnvironment, JudgedBy};
    use singlefs_verification::crash_injection::StateOfTheTableAfterTheWritableMount;
    use singlefs_verification::crash_judge_conclusions::cross_check_words_compared_by_the_cross_check;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "the_gpu_judges_the_claimed_version_of_the_pool_after_the_mount_like_the_cpu_when_the_recovery_lands_below_it";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let (recorded, _claimed_root) =
        second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written();
    let (directory, cpu_configuration) =
        disposable_library_with_a_configuration_on_the_cpu("claimed-version-paired");
    let gpu_configuration = directory.join("multi-host-with-gpu.env");
    write_a_configuration_with_the_gpu_enabled(&gpu_configuration);
    let cards = cards_of_the_caller();
    let judged_in_a_run = |run: &str, gpu_enabled: bool| {
        let library = disposable_library_of_the_campaign(&format!("claimed-version-paired-{run}"));
        let settings: Vec<(&str, &std::ffi::OsStr)> = if gpu_enabled {
            settings_of_an_engine_on_the_gpu(&gpu_configuration, &cards).to_vec()
        } else {
            vec![("SINGLEFS_MULTI_HOST_CONFIG", cpu_configuration.as_os_str())]
        };
        let _the_library_of_this_process = library.make_it_the_library_of_this_process();
        let environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
        let engine = given_states_engine_under_the_callers_environment_guard(&format!(
            "claimed_version_of_the_pool_after_the_mount_paired {run}"
        ));
        let judged = recorded
            .with_the_table(|table_with_its_columns| {
                engine.judge_the_given_states_with_its_columns(table_with_its_columns)
            })
            .unwrap_or_else(|reason| panic!("{run} 那一趟判不了（不退回别处判）：{reason}"));
        let engine_run = engine.finish();
        drop(environment);
        (judged, engine_run)
    };
    let (on_the_gpu, gpu_run) = judged_in_a_run("on-the-gpu", true);
    let (on_the_cpu, cpu_run) = judged_in_a_run("on-the-cpu", false);
    assert_eq!(
        (gpu_run.judged_by, cpu_run.judged_by),
        (JudgedBy::GpuJudge, JudgedBy::CpuJudge),
        "一趟由 GPU 判器判、一趟由 CPU 判器判"
    );
    assert_eq!(
        gpu_run.red_items(),
        Vec::<String>::new(),
        "GPU 判器判得了这张表的两个状态"
    );
    assert_eq!(
        on_the_gpu
            .states
            .iter()
            .map(|state| cross_check_words_compared_by_the_cross_check(state.words))
            .collect::<Vec<_>>(),
        on_the_cpu
            .states
            .iter()
            .map(|state| cross_check_words_compared_by_the_cross_check(state.words))
            .collect::<Vec<_>>(),
        "两个状态 GPU 与 CPU 的对拍字去掉内核不置位、对拍不比的那几位（C821 的 I-1.13）之后逐字相同"
    );
    let pool_after_the_mount =
        &on_the_gpu.states[StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position()];
    assert!(
        pool_after_the_mount
            .record_check()
            .claimed_state_missing_unit
            && pool_after_the_mount.claimed_version_is_not_landed_on(),
        "GPU 判器照那一栏取那一版：第二条判据红、恢复落到的根不是那一栏写的那一条也红：{:?}",
        pool_after_the_mount.words
    );
}
