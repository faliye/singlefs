//! 崩溃点重放枚举用例的两段入口（崩溃判定库的 [`CrashLibraryClass::CrashStateEnumeration`] 类）：录入段在 CPU 上把流写进库，判段只从库里读回、
//! 按块判（配了 GPU 上卡判，没配由 CPU 判）、结论按（块键，结论种类，版本）存进库，判过的块下一趟复用（剪枝），崩溃点重放的计数
//! （[`CrashPointReplayTally`]）从库里读回的结论重建。判一个状态的判法在 `crate::crash`（CPU）与 `crate::crash_judge_gpu`（GPU），这里只排两段、
//! 领块、存库、读回。
//!
//! 一条崩溃枚举用例的流按步切成前缀树的节点（L6：写下标区间互相交叠的几段并成一步，一步一个崩溃点，点名 `<项名>.<周期>.<步序>` 只作索引），
//! 加末节点（整张写表全部持久那一个状态，`crash_amplification::END_OF_FLOW_NODE_NAME`）；块键与崩溃放量同一个算法、只由内容定
//! （段写表、起步镜像、计划哈希），几条流共有的前几步只判一次；判器那一半是判法单元指纹（`crate::crash_identity::judging_units_key`），
//! 设点文件、项名都不进键。录入段按录入键入库，命中就不再录（`crate::crash_library::two_stages`）。
//! 跑完打一行 `CRASH_CASE_BLOCKS blocks= reused= fresh= red= refused= states=`（[`CrashCaseBlocks`]）；块一律照键读库复用，缺的当场判；
//! 判器那一半拒算的整条用例照判、不入库，块计入 `refused=`。
//! 两台分片与崩溃放量同一种读法（`SINGLEFS_CRASH_AMPLIFICATION_SHARE`、`…_HOST_WEIGHTS`、`…_IMPORT`）。
//!
//! 行：开跑一行 `CRASH_POINT_REPLAY_PARALLEL_START`、跑完一行 `CRASH_POINT_REPLAY_PARALLEL_FINISHED`（门禁 crash-point-replay 的准入模块按它判线程；
//! `slices=` 是块数、`resumed_slices=` 是库里复用的块、`freshly_run_slices=` 是这一趟判的块），每个新签名一行 `CRASH_POINT_REPLAY_FINDING`、
//! 收尾一行 `CRASH_POINT_REPLAY_FINDINGS`；设了 `SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE` 时发现日志写一节定稿（`crate::crash_point_replay_progress`）。
//! 每一趟开头（不论只录还是只判）照调用方的流打覆盖行 `CRASH_FLOW_COVERAGE`（`crate::crash_amplification::crash_flow_coverage_lines`）。

use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::ops::Range;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::gpu_memory_ledger::StopRequestRememberedInThisRun;
use crate::heavy_threads::{
    default_heavy_threads_of_the_cap_value, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use crate::hexadecimal::hexadecimal_text;
use crate::memory_pool::{
    root_identity_of_write, CrashImage, MemoryPool, PublishedVersion, RecordCheck, RetainedWrite,
};
use crate::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use crate::segments::StepKind;
use crate::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_checker::image::IMPLEMENTED_INVARIANTS;

use crate::crash::{
    crash_state_ranges_by_segment, publish_of_each_segment, CrashPointReplayFindingSignature,
    CrashPointReplayOracleViolationKind, CrashPointReplayPublishOfState, CrashPointReplayRedPass,
    CrashPointReplaySegmentExpansion, CrashPointReplaySegmentOfState, CrashPointReplayTally,
    CrashPointReplayVersionTable, CrashStateJudgement,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE, CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER,
    CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER, RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT,
    RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};
use crate::crash_amplification::{
    changed_units_line_of_the_stale_verdicts, check_blocks_where_until_with_threads,
    count_vector_disagreements_of_the_flow, crash_flow_coverage_lines,
    observer_versions_keeping_the_refused_out, observers_whose_recorded_blocks_are_missing,
    pending_blocks, plan_crash_points, record_the_observers_of_the_flow,
    states_with_the_root_slot_write_persisted, stop_is_requested_by_the_environment,
    unjudged_blocks_of_the_flow, write_the_recorded_blocks, ConclusionVersionsOfThePlan,
    CpuJudgementsOfSampledBlocks, CrashFlow, CrashPlan, CrashPointSpan, ExpandUpToWritesPerSegment,
    FlowOfTheLibrary, JudgedBy, NodeOfAnObserver, ObserveACrashState,
    ObserveACrashStateHandingBackAnImage, ObserverCode, ObserverDeduplication, ObserverOnTheNodes,
    ObserverVersions, ObservingOfTheRun, PoolCheckerOfTheImagesAfterTheObservers, RecordedFlow,
    RecordedFlowReadBack, WeightedShares, END_OF_FLOW_NODE_NAME,
};
use crate::crash_identity::{
    judging_units_key, ConclusionKind, ConclusionVersion, JudgingUnits, ObserverName,
};
use crate::crash_judge_conclusions::{
    red_items_of_the_encoded_verdict, RECOVERY_OUTCOME_CODE_FAILED,
    RECOVERY_OUTCOME_CODE_FILE_READ, RECOVERY_OUTCOME_CODE_NO_FILE,
};
use crate::crash_library::{
    recording_key_of_the_inputs, toolchain_of_this_process, two_stages,
    two_stages_under_the_recording_key_topping_up_a_reused_recording, CrashLibrary,
    CrashLibraryClass, ItemManifest, ItemName, RecordingInputsDigest, RunStage, StageOutcome,
};
use crate::crash_point_replay_progress::{
    findings_log_final_lines, findings_standard_output_new_signature_line,
    findings_standard_output_summary_line, CrashPointReplayFindingsLog,
    CrashPointReplayFindingsLogBegin, CrashPointReplayFindingsLogSection,
};
use crate::crash_point_replay_provenance::{
    observers_on_the_nodes_with_the_ones_attached_to_the_flow, RunOfTheFlow,
};
use crate::verdict_store::{StoredConclusionBlock, VerdictStore};

/// 崩溃点重放判段的工作线程数从这个环境变量取（十进制正整数）；没设取崩溃放量的 `SINGLEFS_CRASH_AMPLIFICATION_THREADS`，再没设取
/// 每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）。门禁 crash-point-replay 显式传进来。
pub const CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_POINT_REPLAY_THREADS";

/// 清单里流那一样的名字。
const MANIFEST_FLOW: &str = "flow";
/// 清单里 GPU 判表那一样的名字（配了 GPU 才录）。
const MANIFEST_JUDGE_TABLES: &str = "judge_tables";
/// 清单的种类。
const MANIFEST_KIND: &str = "crash_state_enumeration";

/// 一条崩溃枚举用例的项：项名（录入段按它登清单、判段凭它找；也是这条流那个崩溃点的名字，进节点身份与路径）与设点文件
/// （仓内路径，`crash_amplification::repository_relative_file(file!())`：节点身份与 CPU 结论版本按它取）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrashPointReplayItem<'item> {
    pub name: &'item str,
    pub setting_file: &'item str,
}

/// 逐状态看一遍、只记计数的观察者（那一种）：名字、另读的输入的摘要（进它的版本；没有另读的给全零）、怎么看一个状态
/// （拿到崩溃镜像、看 journal 那一遍恢复的报告、这个状态的计数；要记的数全经计数记，放在闭包自己变量里的续跑与复用之后就少了）。
/// 两段：录入段在 CPU 上逐状态看、结果写进库，判段只从库里读、拼成结论按（块键，观察者种类，版本）存进库，
/// 看过的块下一趟复用；观察者 panic 的状态记成红项，判段收尾时整条用例停下。它可以读持久集合的结构，不去重。
#[derive(Clone, Copy)]
pub struct CrashPointReplayObserver<'observer> {
    pub name: &'observer str,
    pub extra_inputs_digest: [u8; DIGEST_BYTES],
    pub observe: &'observer ObserveACrashState<'observer>,
}

/// 交回镜像的观察者（可写挂载、续做）：名字、另读的输入的摘要（进它的版本与去重键；有另读的就要真算）、怎么看一个状态（交回它在崩溃镜像上发的写与
/// 停下时的盘，`crash_amplification::ImageAfterTheObserver`）。录入段在 CPU 上看、按恢复镜像的字节去重（同一张镜像只看一次），判段从库里读回、
/// 在观察者交回的盘上跑池级 checker（配了 GPU 在卡上判，没配由 CPU 判）。
#[derive(Clone, Copy)]
pub struct CrashPointReplayObserverHandingBackAnImage<'observer> {
    pub name: &'observer str,
    pub extra_inputs_digest: [u8; DIGEST_BYTES],
    pub observe: &'observer ObserveACrashStateHandingBackAnImage<'observer>,
    pub deduplication: ObserverDeduplication,
}

/// 崩溃枚举用例自己挂的一个观察者挂在哪几个点上（「按功能挂到对应的段」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPointReplayObserverPoints<'segments> {
    /// 整条流的每个点与末节点（用例自己那个节点与末节点）；另读的输入照原样进版本、结论键不动。
    EveryPointOfTheFlow,
    /// 只挂在罩着这几段（流的 `segments` 的下标）的点上，不挂末节点；一个点罩几段的（写区间交叠并成一步的），那一点整点都挂。
    /// 段集合（去重、排序）并进另读的输入、进这个观察者的结论版本，不进判器块键（D13（验证路线） 已定项 17：这份判定自己的内容输入）。
    PointsOverTheSegments(&'segments [usize]),
}

/// 观察者代码文件：它那一组判法单元按（代码文件，观察者名）在 `crate::judging_units_entries::OBSERVER_ENTRIES` 里找入口。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPointReplayObserverCodeFile<'file> {
    /// 用例文件（设点文件）。
    TheCaseFile,
    /// 几条用例共用的那一份（仓内路径）：几条流挂同一个观察者、要同一把结论版本时点名同一份文件，登记表里只有那一行、只收一次闭包。
    SharedFile(&'file str),
}

/// 崩溃枚举用例挂一个观察者的挂法：挂在哪几个点上、代码文件是哪一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrashPointReplayObserverAttachment<'attachment> {
    pub points: CrashPointReplayObserverPoints<'attachment>,
    pub code_file: CrashPointReplayObserverCodeFile<'attachment>,
}

impl CrashPointReplayObserverAttachment<'static> {
    /// 默认的挂法：整条流、代码文件是用例文件。
    pub const EVERY_POINT_WITH_THE_CASE_FILE: Self = Self {
        points: CrashPointReplayObserverPoints::EveryPointOfTheFlow,
        code_file: CrashPointReplayObserverCodeFile::TheCaseFile,
    };
}

/// 只记计数的观察者连同它的挂法。
#[derive(Clone, Copy)]
pub struct AttachedCrashPointReplayObserver<'observer> {
    pub observer: CrashPointReplayObserver<'observer>,
    pub attachment: CrashPointReplayObserverAttachment<'observer>,
}

/// 交回镜像的观察者连同它的挂法。
#[derive(Clone, Copy)]
pub struct AttachedCrashPointReplayObserverHandingBackAnImage<'observer> {
    pub observer: CrashPointReplayObserverHandingBackAnImage<'observer>,
    pub attachment: CrashPointReplayObserverAttachment<'observer>,
}

/// 判段在哪判（有卡上卡、没配 GPU 由 CPU 判；配了 GPU 而这份二进制没编进 `gpu` 特性，判段不跑）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrashPointReplayJudgeRoute {
    Cpu {
        why_not_the_gpu: &'static str,
    },
    Gpu,
    /// 配置开了 GPU、这份二进制没编进 `gpu` 特性（不带特性编的那一趟）：录入段照录、写进库，判段不判、打本次未跑那一行
    /// （[`crash_point_replay_judge_not_run_line`]）、交只录（`StageOutcome::RecordedOnly`），带特性那一趟从库里读回在卡上判
    /// （「配 GPU 必用 GPU、做不到报错不降级」；与 `crate::crash_amplification::run_from_environment` 同一种写法）。
    JudgeNotRunWithoutTheGpuFeature,
}

impl CrashPointReplayJudgeRoute {
    fn name(self) -> &'static str {
        match self {
            Self::Cpu { .. } => "cpu",
            Self::Gpu => "gpu",
            Self::JudgeNotRunWithoutTheGpuFeature => "not_run_without_the_gpu_feature",
        }
    }

    /// 结论按哪一种判器的版本取（观察者那几把版本按它算）：判段不跑的那一趟按配置的那一路（GPU），带特性那一趟照同一把读回录入段录的。
    fn judged_by(self) -> JudgedBy {
        match self {
            Self::Cpu { .. } => JudgedBy::CpuJudge,
            Self::Gpu | Self::JudgeNotRunWithoutTheGpuFeature => JudgedBy::GpuJudge,
        }
    }
}

/// 照本机多机配置选路（[`crash_point_replay_judge_route_of_the_switches`]）：配置按这个进程从哪起找
/// （`crate::crash_amplification::multi_host_configuration_of_this_process`，不收仓根）。
fn crash_point_replay_judge_route() -> Result<CrashPointReplayJudgeRoute, String> {
    let configuration = crate::crash_amplification::multi_host_configuration_of_this_process()?;
    Ok(crash_point_replay_judge_route_of_the_switches(
        configuration.mode.uses_gpu(),
        cfg!(feature = "gpu"),
    ))
}

/// 配置开了 GPU（`ENABLE_GPU=1`）而这份二进制编进了 `gpu` 特性就上卡；配了而没编进来，判段不跑、报本次未跑（不退回 CPU 判
/// 「配 GPU 必用 GPU、做不到报错不降级」）；没配由 CPU 判。
fn crash_point_replay_judge_route_of_the_switches(
    gpu_is_switched_on: bool,
    gpu_feature_is_compiled_in: bool,
) -> CrashPointReplayJudgeRoute {
    match (gpu_is_switched_on, gpu_feature_is_compiled_in) {
        (true, true) => CrashPointReplayJudgeRoute::Gpu,
        (true, false) => CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature,
        (false, _) => CrashPointReplayJudgeRoute::Cpu {
            why_not_the_gpu: "gpu_not_enabled_in_the_configuration",
        },
    }
}

/// 判段不跑那一趟打的本次未跑那一行：门禁认的 GPU 用例那一族记号（`NOT_RUN_<原因> test=<用例> reason=<一句>`，
/// `singlefs_verification::gpu_memory_ledger::gpu_not_run_line`），用例是这条崩溃枚举用例的项名；原因同崩溃放量那一处
/// （`crate::crash_amplification::judge_not_run_without_the_gpu_feature`）。不打 `CRASH_POINT_REPLAY_JUDGE route=cpu`、不打带 `judge=` 的 PARALLEL 行。
#[must_use]
pub fn crash_point_replay_judge_not_run_line(item_name: &str) -> String {
    crate::gpu_memory_ledger::gpu_not_run_line(
        &format!("crash_state_enumeration::{item_name}"),
        &crate::crash_amplification::judge_not_run_without_the_gpu_feature(true, false)
            .expect("配置开了 GPU、没编进特性的那一种原因总在"),
    )
}

/// 判段的工作线程数与它从哪来（`environment_variable`、`crash_amplification_threads_variable`、`heavy_thread_default`）。
///
/// # Panics
/// 变量设了却不是正整数。
#[must_use]
pub fn crash_point_replay_worker_threads() -> (usize, &'static str) {
    worker_threads_of_the_variable_values(
        process_environment_variable(CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE)
            .as_deref(),
        process_environment_variable(
            crate::crash_amplification::CHECKING_THREADS_ENVIRONMENT_VARIABLE,
        )
        .as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// [`crash_point_replay_worker_threads`] 的判法，三个变量的值由调用方给（`None` 是没设）：崩溃点重放的变量设了用它，没设用崩溃放量核对线程数的变量，
/// 都没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP` 的值，设了照它，再没设取 `.claude/heavy-thread-default`
/// 的登记值，判法在 `singlefs_verification::heavy_threads`）。设了而不是正整数（0 也不是）就 panic、消息带变量名与原值（经共用解析
/// [`number_from_the_variable_value_panicking_when_unreadable`]），不悄悄退回 1 个线程。
///
/// # Panics
/// 设了的值不是正整数。
#[must_use]
pub fn worker_threads_of_the_variable_values(
    crash_point_replay_threads_value: Option<&str>,
    checking_threads_value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> (usize, &'static str) {
    if let Some(threads) = number_from_the_variable_value_panicking_when_unreadable::<NonZeroUsize>(
        CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE,
        crash_point_replay_threads_value,
    ) {
        return (threads.get(), "environment_variable");
    }
    if let Some(threads) = number_from_the_variable_value_panicking_when_unreadable::<NonZeroUsize>(
        crate::crash_amplification::CHECKING_THREADS_ENVIRONMENT_VARIABLE,
        checking_threads_value,
    ) {
        return (threads.get(), "crash_amplification_threads_variable");
    }
    (
        default_heavy_threads_of_the_cap_value(thread_cap_value).get(),
        "heavy_thread_default",
    )
}

// ============================== 公开入口（崩溃枚举用例调的几种写法） ==============================

/// 单版本形态、每一段都展开（全量）。
pub fn enumerate_crash_states(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    root_index: usize,
    expected_content: &[u8],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_selecting(
        item,
        base,
        writes,
        segments,
        root_index,
        expected_content,
        &|_segment_index, _segment| true,
    )
}

/// 单版本形态，`expand` 选哪几段展开（展开就是任意真子集）。
pub fn enumerate_crash_states_selecting(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    root_index: usize,
    expected_content: &[u8],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> StageOutcome<CrashPointReplayTally> {
    let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
    let versions = [PublishedVersion {
        instance,
        checkpoint_txg,
        content: expected_content.to_vec(),
    }];
    enumerate_crash_states_selecting_versions(
        item, base, writes, segments, root_index, &versions, expand,
    )
}

/// 多版本、每一段都展开（全量）。
pub fn enumerate_crash_states_versions(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_selecting_versions(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &|_segment_index, _segment| true,
    )
}

/// 多版本，`expand` 选哪几段展开（展开就是任意真子集）。
pub fn enumerate_crash_states_selecting_versions(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_with_expansion(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
        &|segment_index, segment| {
            if expand(segment_index, segment) {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        },
        &[],
    )
}

/// 崩溃点重放枚举的总入口：`expansion` 定每一段怎么展开，`versions` 是 oracle 读的版本表（带「没有文件的那几版」的删除流也经它），
/// `observers` 逐状态看（没有就空）。录入段把流（连同每段的展开）写进库、登清单；判段读回、按块判、存结论、重建计数。
/// 调用方挂的观察者之外，按这条流在来历表里那一行另挂几个（`crate::crash_point_replay_provenance`：登记位上的流挂并集对走读观察者，
/// 来历是 mkfs 返回之后的另挂两槽全废盘面观察者），设点文件里不手挂。
///
/// # Panics
/// 开不了库、写不进、读不回；选路要卡而卡用不上（配了 GPU 不退回 CPU）；结论版本与编译时对不上（整趟不写库）；判段判完还有块没结论
/// （不是两台分片时）；计数向量与计划对不上；某个观察者在某个状态上 panic（它那一句原样带出）；项名撞了；来历表里没有这条流、有几行，
/// 或调用方挂的观察者与按来历挂的同名（录任何东西之前）。
#[allow(
    clippy::too_many_arguments,
    reason = "前六个是同一条流的几样输入（与 crash::judge_crash_state_range_of_the_read_back_flow 同一组），其余是展开、版本表与观察者"
)]
pub fn enumerate_crash_states_with_expansion(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    observers: &[CrashPointReplayObserver<'_>],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_with_every_kind_of_observer(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        observers,
        &[],
    )
}

/// 同 [`enumerate_crash_states_with_expansion`]，另带交回镜像的观察者（[`CrashPointReplayObserverHandingBackAnImage`]：可写挂载之后的盘上的
/// 池级 checker 在判段判，配了 GPU 上卡）。
///
/// # Panics
/// 同 [`enumerate_crash_states_with_expansion`]。
#[allow(
    clippy::too_many_arguments,
    reason = "与 enumerate_crash_states_with_expansion 同一组参数，多一组交回镜像的观察者"
)]
pub fn enumerate_crash_states_with_every_kind_of_observer(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    observers: &[CrashPointReplayObserver<'_>],
    observers_handing_back_images: &[CrashPointReplayObserverHandingBackAnImage<'_>],
) -> StageOutcome<CrashPointReplayTally> {
    let attachment = CrashPointReplayObserverAttachment::EVERY_POINT_WITH_THE_CASE_FILE;
    enumerate_crash_states_with_attached_observers(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        &observers
            .iter()
            .map(|observer| AttachedCrashPointReplayObserver {
                observer: *observer,
                attachment,
            })
            .collect::<Vec<_>>(),
        &observers_handing_back_images
            .iter()
            .map(
                |observer| AttachedCrashPointReplayObserverHandingBackAnImage {
                    observer: *observer,
                    attachment,
                },
            )
            .collect::<Vec<_>>(),
    )
}

/// 同 [`enumerate_crash_states_with_every_kind_of_observer`]，每个观察者各带挂法（[`CrashPointReplayObserverAttachment`]）：只挂在点名的段上的观察者
/// 在别的点上不录、不判，块账在它不挂的点上不要它的结论；代码文件可以点名几条用例共用的那一份。
///
/// # Panics
/// 同 [`enumerate_crash_states_with_expansion`]；挂法点名的段集合是空的、段下标越过流的段数、或点名的段一个点都罩不着（录任何东西之前）。
#[allow(
    clippy::too_many_arguments,
    reason = "与 enumerate_crash_states_with_expansion 同一组参数，观察者两组各带挂法"
)]
pub fn enumerate_crash_states_with_attached_observers(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    observers: &[AttachedCrashPointReplayObserver<'_>],
    observers_handing_back_images: &[AttachedCrashPointReplayObserverHandingBackAnImage<'_>],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_through_the_library(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        observers,
        observers_handing_back_images,
    )
    .unwrap_or_else(|refusal| panic!("崩溃枚举用例「{}」：{refusal}", item.name))
}

/// 实验装置逐状态取样（`src/bin/` 下的 E170、E171 三份与它们那一类）：录入段把流写进库，判段读回、只在 CPU 上按序号走 `ordinals` 这一段，
/// 每个状态把镜像与 CPU 判器的判定交给 `judge_state`。`ordinals` 整块罩住的块，判完照 CPU 判器那一处（`JudgedBy::CpuJudge`）存进库
/// （「能走两段的都走两段」：取样的结论也入库，打一行 `CRASH_POINT_REPLAY_SAMPLED_CONCLUSIONS_STORED`）；不复用、不上卡：
/// 装置要的是每个状态恢复报告的全部字段，库里的结论不装它们，取样照旧逐状态现判。
///
/// # Panics
/// 开不了库、写不进、读不回；`ordinals` 越过整条流的状态数。
#[allow(
    clippy::too_many_arguments,
    reason = "前六个是同一条流的几样输入，其余是展开、序号段与回调"
)]
pub fn sample_crash_states_on_the_cpu(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    ordinals: Range<u64>,
    judge_state: &mut dyn FnMut(u64, &CrashImage<'_>, &CrashStateJudgement),
) -> StageOutcome<u64> {
    sample_crash_states_on_the_cpu_against_the_version_table(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
        expansion,
        ordinals,
        judge_state,
    )
}

/// 同 [`sample_crash_states_on_the_cpu`]，oracle 对一张版本表判（带「没有文件的那几版」的删除流）。
///
/// # Panics
/// 同 [`sample_crash_states_on_the_cpu`]。
#[allow(
    clippy::too_many_arguments,
    reason = "与 sample_crash_states_on_the_cpu 同一组参数，版本表换成带「没有文件的那几版」的那一张"
)]
pub fn sample_crash_states_on_the_cpu_against_the_version_table(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    ordinals: Range<u64>,
    judge_state: &mut dyn FnMut(u64, &CrashImage<'_>, &CrashStateJudgement),
) -> StageOutcome<u64> {
    let flow_to_record = flow_of_the_inputs(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
    );
    let sampled_ordinals = ordinals.clone();
    sample_through_the_library(item, &flow_to_record, &sampled_ordinals, |flow, note| {
        crate::crash::judge_crash_state_range_of_the_read_back_flow(
            flow.base,
            flow.writes,
            flow.segments,
            flow.judged_root_index,
            flow.version_table(),
            flow.expansion,
            ordinals,
            &mut |ordinal, publish, image, judgement| {
                note(ordinal, publish, judgement);
                judge_state(ordinal, image, judgement);
            },
        )
    })
}

/// 一条线程切几段：段切得比线程数细、轮着领，靠后的状态历史长、判得慢，按线程数等分时最后一段拖住整趟。
const SAMPLING_SLICES_PER_THREAD: u64 = 8;

/// 同 [`sample_crash_states_on_the_cpu_against_the_version_table`]，判段分 `threads` 条线程：`ordinals` 切成每线程
/// [`SAMPLING_SLICES_PER_THREAD`] 段轮着领，`judge_state` 跨线程调、按序号各报各的，调用方要按序号排；录入段只录一次，
/// 流只从库里读回一次、各线程共用；判段的撕裂表也只建一份、各线程借用（判完核恰好建了一份，不是一份就 panic）。
/// 交回的与单线程那一支相同：整条流的状态数（不是 `ordinals` 的长度）；各线程判的状态数合起来等于 `ordinals` 的长度，不等就 panic。
///
/// # Panics
/// 同 [`sample_crash_states_on_the_cpu`]；判定线程 panic。
#[allow(
    clippy::too_many_arguments,
    reason = "与 sample_crash_states_on_the_cpu_against_the_version_table 同一组参数，多一个线程数、判法换成跨线程的"
)]
pub fn sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    ordinals: Range<u64>,
    threads: NonZeroUsize,
    judge_state: &(dyn Fn(u64, &CrashImage<'_>, &CrashStateJudgement) + Sync),
) -> StageOutcome<u64> {
    let flow_to_record = flow_of_the_inputs(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
    );
    let sampled_ordinals = ordinals.clone();
    sample_through_the_library(item, &flow_to_record, &sampled_ordinals, |flow, note| {
        let state_count = ordinals.end.saturating_sub(ordinals.start);
        let threads = u64::try_from(threads.get()).expect("线程数装得进 u64");
        let slice = state_count
            .div_ceil(threads * SAMPLING_SLICES_PER_THREAD)
            .max(1);
        let slice_count = state_count.div_ceil(slice);
        let next_slice = AtomicU64::new(0);
        // 撕裂表只随流定，一份可上 GB（用过的盘上重做 mkfs 那条流两块盘各一次清零罩着旧 journal 环，一份留着 1.5 GiB）：
        // 判段只建这一份、各线程经同一份排好次序的状态借用，不在每一段里各建一份（`TornImageTablesOfTheReadBackFlow` 的文档
        // 「一个阶段只建一份」；每段各建时三十一条线程同时留着三十一份，挤掉了 40G 的内存上限）。建了几份数在这里，判完核恰好一份。
        let torn_image_tables_built = AtomicU64::new(0);
        let build_the_torn_image_tables_of_the_flow = || {
            torn_image_tables_built.fetch_add(1, Ordering::SeqCst);
            crate::crash::TornImageTablesOfTheReadBackFlow::of_the_read_back_flow(
                flow.base,
                flow.writes,
                flow.segments,
            )
        };
        let torn_image_tables_of_the_flow = build_the_torn_image_tables_of_the_flow();
        let crash_states_of_the_flow = torn_image_tables_of_the_flow.crash_states_planned_by(
            flow.base,
            flow.writes,
            flow.segments,
            flow.expansion,
            &crate::crash::no_segment_slices(flow.segments.len()),
        );
        if slice_count == 0 {
            // 空序号段：一个状态都不判，照单线程那一支交回整条流的状态数
            return crate::crash::judge_crash_state_range_of_the_read_back_flow_with_slices(
                &crash_states_of_the_flow,
                flow.judged_root_index,
                flow.version_table(),
                ordinals.clone(),
                &mut |_ordinal, _publish, _image, _judgement| {},
            );
        }
        let (judged, states_of_the_whole_flow) = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..threads.min(slice_count))
                .map(|_| {
                    let next_slice = &next_slice;
                    let ordinals = &ordinals;
                    let crash_states_of_the_flow = &crash_states_of_the_flow;
                    scope.spawn(move || {
                        let mut judged_here = 0u64;
                        let mut states_of_the_whole_flow: Option<u64> = None;
                        loop {
                            let index = next_slice.fetch_add(1, Ordering::SeqCst);
                            if index >= slice_count {
                                break;
                            }
                            let range = ordinals.start + index * slice
                                ..(ordinals.start + (index + 1) * slice).min(ordinals.end);
                            // 交回的是整条流的状态数（与单线程那一支同一个契约），不是这一段判了几个：判了几个按段的长度数
                            let states = crate::crash::judge_crash_state_range_of_the_read_back_flow_with_slices(
                                crash_states_of_the_flow,
                                flow.judged_root_index,
                                flow.version_table(),
                                range.clone(),
                                &mut |ordinal, publish, image, judgement| {
                                    note(ordinal, publish, judgement);
                                    judge_state(ordinal, image, judgement);
                                },
                            );
                            assert!(
                                range.end <= states,
                                "序号段 {range:?} 越过整条流的 {states} 个状态"
                            );
                            assert!(
                                states_of_the_whole_flow.is_none_or(|known| known == states),
                                "各段看到的整条流状态数不同：{states_of_the_whole_flow:?} 与 {states}"
                            );
                            states_of_the_whole_flow = Some(states);
                            judged_here += range.end - range.start;
                        }
                        (judged_here, states_of_the_whole_flow)
                    })
                })
                .collect();
            let mut judged = 0u64;
            let mut states_of_the_whole_flow: Option<u64> = None;
            for handle in handles {
                let (judged_here, states_here) = handle.join().expect("判定线程不 panic");
                judged += judged_here;
                if let Some(states_here) = states_here {
                    assert!(
                        states_of_the_whole_flow.is_none_or(|known| known == states_here),
                        "各线程看到的整条流状态数不同：{states_of_the_whole_flow:?} 与 {states_here}"
                    );
                    states_of_the_whole_flow = Some(states_here);
                }
            }
            (
                judged,
                states_of_the_whole_flow.expect("序号段不空，至少一条线程领到了段"),
            )
        });
        assert_eq!(
            torn_image_tables_built.load(Ordering::SeqCst),
            1,
            "判段的撕裂表只建一份、各线程借用，不按段各建一份"
        );
        assert_eq!(judged, state_count, "各线程判的状态数合起来是序号段的长度");
        states_of_the_whole_flow
    })
}

/// 取样那一趟判过的一个状态交给库的那一份：序号、它归哪次发布、CPU 判器的判定（整块都判到的块照 CPU 判器那一处存进库）。
type NoteASampledState<'note> =
    dyn Fn(u64, CrashPointReplayPublishOfState, &CrashStateJudgement) + Sync + 'note;

/// 取样的两段：录入段把 `flow_to_record` 按摘要写进崩溃点重放枚举那一类的库、项名下登清单；判段按项名读清单、按摘要读回流，
/// 只把读回的那一份交 `judge_the_read_back_flow`，它每判一个状态交给第二个参数：取样序号段整块罩住的块，判完照 CPU 判器那一处
/// （`JudgedBy::CpuJudge`，结论版本照计划各点那一把）存进库（「能走两段的都走两段」：取样的结论也入库）。
/// 只录那一趟交 `RecordedOnly`。
fn sample_through_the_library(
    item: CrashPointReplayItem<'_>,
    flow_to_record: &FlowToRecord<'_>,
    ordinals: &Range<u64>,
    judge_the_read_back_flow: impl FnOnce(&FlowOfTheLibrary<'_>, &NoteASampledState<'_>) -> u64,
) -> StageOutcome<u64> {
    let item_name = item_name_of(item);
    let recorded_bytes = flow_to_record.recorded_bytes();
    two_stages(
        CrashLibraryClass::CrashStateEnumeration,
        &item_name,
        &FlowToRecord::recording_inputs_digest(&recorded_bytes, false),
        |library| flow_to_record.record_into(library, false, &recorded_bytes),
        |library, manifest| {
            let read_back = RecordedFlowReadBack::read_from_the_library(
                &library.lock(),
                &manifest.digest_named(MANIFEST_FLOW)?,
            )?;
            let flow = read_back.flow();
            let plan = plan_crash_points(&flow);
            // CPU 判器那一组拒算：取这个进程的占位、取样的结论照判不入库（4-2a 报告第四节第 7 条后半句）。取样只由 CPU 判器判：
            // GPU 判器那一组不取，它拒算不打拒算行、不挡取样
            let (conclusion_versions, refusals) =
                ConclusionVersionsOfThePlan::of_the_plan_keeping_the_refused_out(
                    &plan,
                    toolchain_of_this_process(),
                    JudgedBy::CpuJudge,
                );
            for refusal in &refusals {
                println!(
                    "CRASH_CASE_BLOCKS_KEY_REFUSED item={} {}（这一组照判、不入库）",
                    item.name,
                    refusal.sentence()
                );
            }
            library
                .lock()
                .keep_the_rows_of_these_versions_out_of_the_library(
                    conclusion_versions.versions_kept_out_of_the_library(),
                );
            let sampled_blocks = CpuJudgementsOfSampledBlocks::for_the_ordinals(
                &library.lock(),
                &plan,
                conclusion_versions.stored_under(JudgedBy::CpuJudge),
                ordinals,
            )
            .map_err(|error| format!("读判定块失败：{error:?}"))?;
            let sampled = std::sync::Mutex::new((sampled_blocks, None::<String>));
            let note = |ordinal: u64,
                        publish: CrashPointReplayPublishOfState,
                        judgement: &CrashStateJudgement| {
                let mut sampled = sampled
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if sampled.1.is_some() {
                    return;
                }
                if let Err(error) =
                    sampled
                        .0
                        .note_a_state(&mut library.lock(), ordinal, publish, judgement)
                {
                    sampled.1 = Some(format!("取样的 CPU 结论写不进库：{error:?}"));
                }
            };
            let states = judge_the_read_back_flow(&flow, &note);
            let (sampled_blocks_after_judging, failure) = sampled
                .into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(failure) = failure {
                return Err(failure);
            }
            println!(
                "CRASH_POINT_REPLAY_SAMPLED_CONCLUSIONS_STORED item={} judged_by=cpu_judge blocks_stored_here={} blocks_already_stored={}",
                item.name,
                sampled_blocks_after_judging.blocks_stored_here,
                sampled_blocks_after_judging.blocks_already_stored
            );
            Ok(StageOutcome::Judged(states))
        },
    )
    .unwrap_or_else(|refusal| panic!("崩溃点重放取样「{}」：{refusal}", item.name))
}

// ============================== 录入段 ==============================

/// 项名：崩溃点重放的项名也是崩溃点的名字（路径段、节点名的一截），所以 `/` 与 `|` 都不许。
fn item_name_of(item: CrashPointReplayItem<'_>) -> ItemName {
    assert!(
        !item.name.contains('/') && !item.name.contains('|'),
        "崩溃点重放项名 {:?} 里不许有 / 与 |：它也是这条流那个崩溃点的名字，是路径段、节点名的一截",
        item.name
    );
    ItemName::new(item.name).unwrap_or_else(|refusal| panic!("崩溃点重放项名：{refusal}"))
}

/// 录入段要抄进库的那条流：调用方给的几样，加每段的展开（这里先逐段问一次 `expansion`，`CrashFlow` 要的闭包就能跨线程），
/// 加一个罩住整张写表的崩溃点（名字是项名、设点文件是用例文件）。
struct FlowToRecord<'flow> {
    base: &'flow MemoryPool,
    writes: &'flow [RetainedWrite],
    segments: &'flow [Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'flow>,
    expansion_by_segment: Vec<CrashPointReplaySegmentExpansion>,
    crash_points: Vec<CrashPointSpan>,
}

fn flow_of_the_inputs<'flow>(
    item: CrashPointReplayItem<'_>,
    base: &'flow MemoryPool,
    writes: &'flow [RetainedWrite],
    segments: &'flow [Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'flow>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> FlowToRecord<'flow> {
    FlowToRecord {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion_by_segment: segments
            .iter()
            .enumerate()
            .map(|(segment_index, segment)| expansion(segment_index, segment))
            .collect(),
        crash_points: crash_points_of_the_steps(item, writes, segments),
    }
}

/// 一条崩溃枚举用例的流按步切成前缀树的节点（L6）：写下标区间互相交叠的几段并成一步，每一步一个崩溃点，点的写区间首尾相接罩住整张写表；
/// 点名 `<项名>.<周期>.<步序>`（周期取这一步最后一次写的，`crash_amplification::period_of_write`）只作索引。块键照崩溃放量的同一个算法
/// （段写表、起步镜像、计划哈希），几条流共有的前几步就是同几个节点，只判一次。切不成首尾相接的几步（某一段的写落在前一步之前）就
/// 整条流一个点。实验装置（E184（常规层与大负载层的代价计数））也调它，按同一套切法数节点，不另抄一份。
pub fn crash_points_of_the_steps(
    item: CrashPointReplayItem<'_>,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) -> Vec<CrashPointSpan> {
    let whole_flow = || {
        vec![CrashPointSpan {
            name: item.name.to_string(),
            code_file_in_repository: item.setting_file.to_string(),
            writes: 0..writes.len(),
            ignored: false,
        }]
    };
    // 每一步：（最低的写下标，最高的写下标），按段的次序并
    let mut steps: Vec<(usize, usize)> = Vec::new();
    for segment in segments {
        let (Some(lowest), Some(highest)) = (segment.iter().min(), segment.iter().max()) else {
            continue;
        };
        match steps.last_mut() {
            Some((step_lowest, step_highest)) if *lowest <= *step_highest => {
                if *lowest < *step_lowest {
                    return whole_flow();
                }
                *step_highest = (*step_highest).max(*highest);
            }
            Some(_) | None => steps.push((*lowest, *highest)),
        }
    }
    if steps.len() < 2 || !steps.windows(2).all(|pair| pair[0].1 < pair[1].0) {
        return whole_flow();
    }
    let starts: Vec<usize> = std::iter::once(0)
        .chain(steps.iter().skip(1).map(|(lowest, _highest)| *lowest))
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(step_index, start)| {
            let end = starts.get(step_index + 1).copied().unwrap_or(writes.len());
            let (_lowest, highest) = steps[step_index];
            CrashPointSpan {
                name: format!(
                    "{}.{}.{step_index}",
                    item.name,
                    crate::crash_amplification::period_of_write(&writes[highest])
                ),
                code_file_in_repository: item.setting_file.to_string(),
                writes: *start..end,
                ignored: false,
            }
        })
        .collect()
}

impl FlowToRecord<'_> {
    fn with_the_flow<Outcome>(
        &self,
        use_the_flow: impl FnOnce(&CrashFlow<'_>) -> Outcome,
    ) -> Outcome {
        let expansion =
            |segment_index: usize, _segment: &[usize]| self.expansion_by_segment[segment_index];
        let flow = CrashFlow {
            base: self.base,
            writes: self.writes,
            segments: self.segments,
            judged_root_index: self.judged_root_index,
            versions: self.versions.versions_with_a_file(),
            versions_without_a_file: self.versions.versions_without_a_file(),
            expansion: &expansion,
            crash_points: self.crash_points.clone(),
        };
        use_the_flow(&flow)
    }

    /// 录进库的那一份流的字节（[`RecordedFlow`]）：录入键的输入照它算，录入段照它写。
    fn recorded_bytes(&self) -> Vec<u8> {
        self.with_the_flow(|flow| RecordedFlow::of(flow).to_bytes())
    }

    /// 录入键的输入那一半：录进库的那一份流的字节与判表开不开（配了 GPU 才打包判表）；项名、设点文件不进。
    fn recording_inputs_digest(recorded_bytes: &[u8], gpu_judges: bool) -> RecordingInputsDigest {
        let mut message = b"singlefs crash state enumeration recording inputs\n".to_vec();
        message.extend_from_slice(&sha256_digest(recorded_bytes));
        message.extend_from_slice(format!("gpu_judges={gpu_judges}\n").as_bytes());
        RecordingInputsDigest(sha256_digest(&message))
    }

    /// 录入段：排计划、写录入表与路径索引、把流写进库（配了 GPU 再把打包判表写进库），交回清单。
    fn record_into(
        &self,
        library: &CrashLibrary,
        gpu_judges: bool,
        recorded_bytes: &[u8],
    ) -> Result<ItemManifest, String> {
        self.with_the_flow(|flow| {
            let mut store = library.lock();
            let plan = plan_crash_points(flow);
            write_the_recorded_blocks(&mut store, &plan)
                .map_err(|error| format!("录入失败：{error:?}"))?;
            let flow_digest = sha256_digest(recorded_bytes);
            store
                .store_recorded_flow(&flow_digest, recorded_bytes)
                .map_err(|error| format!("录下的流程写不进库：{error:?}"))?;
            let mut digests = vec![(MANIFEST_FLOW.to_string(), flow_digest)];
            if gpu_judges {
                digests.push((
                    MANIFEST_JUDGE_TABLES.to_string(),
                    record_the_judge_tables(&mut store, flow, &plan)?,
                ));
            }
            println!(
                "CRASH_POINT_REPLAY_FLOW_RECORDED flow={} blocks={} states={}",
                hexadecimal_text(&flow_digest[..8]),
                plan.points
                    .iter()
                    .map(|point| point.blocks().len())
                    .sum::<usize>(),
                plan.points
                    .last()
                    .map_or(0, |end_of_flow| end_of_flow.ordinals.end)
            );
            Ok(ItemManifest {
                kind: MANIFEST_KIND.to_string(),
                digests,
            })
        })
    }
}

/// 录入段把 GPU 判表写进库（配了 GPU 的那一趟），交回它的摘要。
#[cfg(feature = "gpu")]
fn record_the_judge_tables(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
) -> Result<[u8; DIGEST_BYTES], String> {
    let packed_bytes =
        crate::crash_amplification::packed_judge_tables_of_the_plan(flow, plan)?.to_bytes();
    let digest = sha256_digest(&packed_bytes);
    store
        .store_packed_judge_tables(&digest, &packed_bytes)
        .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
    Ok(digest)
}

/// 没编进 `gpu` 特性选路不会要 GPU 判表（[`crash_point_replay_judge_route`]）。
#[cfg(not(feature = "gpu"))]
fn record_the_judge_tables(
    _store: &mut VerdictStore,
    _flow: &CrashFlow<'_>,
    _plan: &CrashPlan,
) -> Result<[u8; DIGEST_BYTES], String> {
    Err("这份二进制没编进 gpu 特性，建不了 GPU 判表（选路不该要它）".to_string())
}

// ============================== 两段 ==============================

/// 一条崩溃枚举用例跑完打的那一行的头（前五个字段次序照 `scripts/gate/admission.py` 的 `CRASH_CASE_BLOCKS_FIELDS`，它按字段名读、不看后面多出的
/// `states=`）。
pub const CRASH_CASE_BLOCKS_LINE_PREFIX: &str = "CRASH_CASE_BLOCKS";

/// 一条崩溃枚举用例这一趟这台机器那一份的块账（[`CRASH_CASE_BLOCKS_LINE_PREFIX`] 那一行）：`blocks = reused + fresh + refused`。
/// 判段那一趟：`reused` 是键相同、库里有不红判定的块，`fresh` 是这一趟现判的（记着红的块不复用、这一趟现判，算在这里：
/// 「记红、撤绿、下趟现判」）。`refused` 是判器那一半键拒算、照判不入库的，`red` 是判红的块（现判的与库里记着的都算）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrashCaseBlocks {
    pub blocks: u64,
    pub reused: u64,
    pub fresh: u64,
    pub red: u64,
    pub refused: u64,
    /// 这一份的块里一共几个状态（[`states_of_the_blocks_of_this_share`]，出口 10「每项态数现数」）。
    pub states: u64,
    /// 这一趟缩过域时用的展开上限（`crate::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE` 的值）；没缩是 `None`。
    pub expand_up_to: Option<usize>,
}

impl CrashCaseBlocks {
    /// `CRASH_CASE_BLOCKS blocks=<n> reused=<n> fresh=<n> red=<n> refused=<n> states=<n>`，缩过域的那一趟末尾再接 ` expand_up_to=<n>`。
    #[must_use]
    pub fn line(&self) -> String {
        let counted_blocks = format!(
            "{CRASH_CASE_BLOCKS_LINE_PREFIX} blocks={} reused={} fresh={} red={} refused={}",
            self.blocks, self.reused, self.fresh, self.red, self.refused
        );
        let line = format!("{counted_blocks} states={}", self.states);
        match self.expand_up_to {
            Some(expand_up_to) => format!("{line} expand_up_to={expand_up_to}"),
            None => line,
        }
    }
}

/// 这条用例判器那一半的键：判器与挂着的观察者各一把（[`crate::crash_identity::judging_units_key`]）。哪几把拒算：那几组取这个进程的
/// 占位、照判不入库（键里带着占位与从它拼出来的版本的行只留在这个进程里，`VerdictStore::keep_the_rows_of_these_versions_out_of_the_library`），
/// 别的照键入库同一份长期库（4-2a 报告第四节第 7 条后半句；不另建一次性库、不退回整文件或整表指纹）；这条用例的块全部计入 `refused=`。
struct KeysOfTheItem {
    /// 挂着的每个观察者那几把版本（拒算的那几组是占位拼的）。
    observer_versions: Vec<ObserverVersions>,
    /// 拒算的那几组，一组一句（行里逐个点名）。
    refusal_sentences: Vec<String>,
    /// 判器那一组拒算（判器的结论用占位、照判不入库）。
    judge_is_refused: bool,
    /// 照判、不入库的那几把版本：开库之后登记进库。
    versions_kept_out_of_the_library: Vec<[u8; DIGEST_BYTES]>,
}

impl KeysOfTheItem {
    /// 有没有哪一组拒算（这条用例的块全部计入 `refused=`）。
    fn are_refused(&self) -> bool {
        !self.refusal_sentences.is_empty()
    }
}

/// 这条用例这一趟判器那一半的键（判器那一组、挂着的每个观察者那一组）。
fn keys_of_the_item(
    route: CrashPointReplayJudgeRoute,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
) -> KeysOfTheItem {
    keys_of_the_item_with_the_judge_units_keyed_by(route, observers_on_the_nodes, |units| {
        judging_units_key(units, toolchain_of_this_process())
    })
}

/// 同 [`keys_of_the_item`]，判器那一半那几组的键由 `key_of_the_judge_units` 取（单测造它拒算）；观察者那几组照这个进程取。
fn keys_of_the_item_with_the_judge_units_keyed_by(
    route: CrashPointReplayJudgeRoute,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    key_of_the_judge_units: impl Fn(
        &JudgingUnits,
    ) -> Result<
        crate::crash_identity::JudgingUnitsKey,
        crate::crash_identity::JudgingUnitsKeyRefused,
    >,
) -> KeysOfTheItem {
    let toolchain = toolchain_of_this_process();
    // 镜像判定的版本是判定定义那一组与 GPU 判器那一组并成的一把，CPU 判与 GPU 判两条路线都取（「GPU 判器代码进键」）：
    // 哪一组取不出，两条路线都照判、不入库、报拒算（只查 CPU 路线要不要判定定义那一组时，GPU 判器那一组拒算让判段报写不了库、整条用例错）
    let judge_units_needed: &[JudgingUnits] =
        &crate::crash_amplification::JUDGING_UNITS_OF_THE_IMAGE_JUDGEMENT_VERSION;
    let mut keys = KeysOfTheItem {
        observer_versions: Vec::with_capacity(observers_on_the_nodes.len()),
        refusal_sentences: Vec::new(),
        judge_is_refused: false,
        versions_kept_out_of_the_library: Vec::new(),
    };
    for judge_units in judge_units_needed {
        if let Err(refusal) = key_of_the_judge_units(judge_units) {
            keys.refusal_sentences.push(refusal.sentence());
            keys.judge_is_refused = true;
            keys.versions_kept_out_of_the_library.push(
                *crate::crash_amplification::placeholder_judging_units_key_of_the_refused(
                    judge_units,
                )
                .fingerprint(),
            );
        }
    }
    for observer in observers_on_the_nodes {
        let (versions, refusals, versions_kept_out) =
            observer_versions_keeping_the_refused_out(observer, route.judged_by(), toolchain);
        keys.refusal_sentences
            .extend(refusals.iter().map(|refusal| refusal.sentence()));
        keys.versions_kept_out_of_the_library
            .extend(versions_kept_out);
        keys.observer_versions.push(versions);
    }
    keys
}

#[allow(
    clippy::too_many_arguments,
    reason = "与 enumerate_crash_states_with_expansion 同一组参数"
)]
fn enumerate_crash_states_through_the_library(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    observers: &[AttachedCrashPointReplayObserver<'_>],
    observers_handing_back_images: &[AttachedCrashPointReplayObserverHandingBackAnImage<'_>],
) -> Result<StageOutcome<CrashPointReplayTally>, String> {
    // 按来历统一挂：来历表里查不到这条流就拒，在录任何东西之前
    let observers_on_the_nodes = observers_on_the_nodes_with_the_ones_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        item.name,
        &observers_on_the_nodes_of_the_item(
            item,
            writes,
            segments,
            observers,
            observers_handing_back_images,
        )?,
        &nodes_of_the_item(item),
    )
    .map_err(|refusal| refusal.sentence())?;
    let route = crash_point_replay_judge_route()?;
    let keys = keys_of_the_item(route, &observers_on_the_nodes);
    // 缩域（「允许缩域」）：设了展开上限变量就取用例自己的展开与它里小的那个；块键含段展开，缩过域的块各存各的
    let narrowing = ExpandUpToWritesPerSegment::narrowing_from_the_environment();
    let narrowed_expansion = |segment_index: usize, segment: &[usize]| {
        expansion_of_the_case_narrowed_by(narrowing, expansion(segment_index, segment), segment)
    };
    let flow_to_record = flow_of_the_inputs(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &narrowed_expansion,
    );
    // 覆盖行：覆盖表脚本从这一趟的日志里读哪个（入口，提交步骤，段）在哪个域里展开了。崩溃枚举用例没有按段登记；
    // 只录、只判的那几趟同样打（录下的流是同一条）
    flow_to_record.with_the_flow(|flow| {
        for coverage_line in crash_flow_coverage_lines(flow, &[]) {
            println!("{}", coverage_line.text());
        }
    });
    let (outcome, blocks) = enumerate_the_flow_through_the_library_with_the_keys(
        item,
        &flow_to_record,
        &observers_on_the_nodes,
        route,
        keys,
    )?;
    if let Some(mut blocks) = blocks {
        blocks.expand_up_to = narrowing.map(|limit| limit.0);
        println!("{}", blocks.line());
    }
    Ok(outcome)
}

/// 崩溃枚举用例一段的展开：没缩（`narrowing` 是 `None`）照用例自己的，缩了取用例自己的与上限里小的那个
/// （[`ExpandUpToWritesPerSegment::narrowed_expansion`]）。
fn expansion_of_the_case_narrowed_by(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    own_expansion: CrashPointReplaySegmentExpansion,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    narrowing.map_or(own_expansion, |limit| {
        limit.narrowed_expansion(own_expansion, segment)
    })
}

/// 缩过域那一趟的闭式态数断言不判时打的那一行的头（[`ClosedFormStateCountJudgement::line`]）。
pub const CLOSED_FORM_NOT_JUDGED_LINE_PREFIX: &str = "CRASH_CASE_CLOSED_FORM_NOT_JUDGED";

/// 崩溃枚举用例「枚举到的态数等于闭式」那一条断言这一趟判了没有。闭式按用例自己的展开算（全量域；态数、按发布分的态数都算），
/// 缩过域的那一趟数出来的本来就比它少。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClosedFormStateCountJudgement {
    /// 没缩域：判了，枚举到的等于闭式。
    Judged,
    /// 缩过域（[`crate::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE`] 设了）：闭式不判；每个崩溃块照判（在枚举那一趟里判，与这一条无关）。
    NotJudgedBecauseTheDomainWasNarrowed {
        /// 这一趟的展开上限（变量的值）。
        expand_up_to: usize,
        /// 这一趟实际展开、枚举到的（`Debug` 的写法）。
        counted: String,
        /// 全量域的闭式（`Debug` 的写法；这一趟不判）。
        closed_form: String,
    },
}

impl ClosedFormStateCountJudgement {
    /// 缩过域的那一趟打的结果行：`CRASH_CASE_CLOSED_FORM_NOT_JUDGED expand_up_to=<n> counted=<这一趟实际展开、枚举到的> closed_form=<全量域的闭式> what=<哪一条>`；
    /// 判了的那一趟不打（`None`）。
    #[must_use]
    pub fn line(&self, what: &str) -> Option<String> {
        match self {
            Self::Judged => None,
            Self::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to,
                counted,
                closed_form,
            } => Some(format!(
                "{CLOSED_FORM_NOT_JUDGED_LINE_PREFIX} expand_up_to={expand_up_to} counted={counted} closed_form={closed_form} what={what}"
            )),
        }
    }
}

/// 崩溃枚举用例「枚举到的态数等于闭式」那一条断言（态数，或按发布分的态数那一串），各用例都经它判：照这个进程的环境
/// （[`ExpandUpToWritesPerSegment::narrowing_from_the_environment`]），没缩域照判，缩过域不判、往 stdout 打结果行
/// （写明缩过、这一趟的展开上限与实际展开、枚举到的）。
///
/// # Panics
/// 没缩域而 `counted` 不等于 `closed_form`；缩域变量写坏了（同 [`ExpandUpToWritesPerSegment::narrowing_from_the_environment`]）。
#[track_caller]
pub fn assert_the_counted_states_equal_the_closed_form_unless_narrowed<Counted, ClosedForm>(
    counted: Counted,
    closed_form: ClosedForm,
    what: &str,
) -> ClosedFormStateCountJudgement
where
    Counted: PartialEq<ClosedForm> + std::fmt::Debug,
    ClosedForm: std::fmt::Debug,
{
    let judgement = closed_form_state_count_judged_under(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment(),
        counted,
        closed_form,
        what,
    );
    if let Some(line) = judgement.line(what) {
        println!("{line}");
    }
    judgement
}

/// 同 [`assert_the_counted_states_equal_the_closed_form_unless_narrowed`]，缩域上限由调用方给（`None` 是没缩），不读环境、不打行。
///
/// # Panics
/// `narrowing` 是 `None` 而 `counted` 不等于 `closed_form`。
#[track_caller]
pub fn closed_form_state_count_judged_under<Counted, ClosedForm>(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    counted: Counted,
    closed_form: ClosedForm,
    what: &str,
) -> ClosedFormStateCountJudgement
where
    Counted: PartialEq<ClosedForm> + std::fmt::Debug,
    ClosedForm: std::fmt::Debug,
{
    match narrowing {
        None => {
            assert_eq!(counted, closed_form, "{what}：枚举到的要等于闭式");
            ClosedFormStateCountJudgement::Judged
        }
        Some(limit) => ClosedFormStateCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
            expand_up_to: limit.0,
            counted: format!("{counted:?}"),
            closed_form: format!("{closed_form:?}"),
        },
    }
}

/// 缩过域那一趟按全量域钉的计数断言不判时打的那一行的头（[`FullDomainCountJudgement::line`]）。
pub const FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX: &str =
    "CRASH_CASE_FULL_DOMAIN_COUNT_NOT_JUDGED";

/// 崩溃枚举用例里按全量域钉的别的计数——逐条不变量的评估数（`CrashPointReplayTally::checker_evaluated_states`）、观察者计数
/// （`CrashPointReplayTally::observer_counts`）、oracle 与记录核对器那几格（根已持久、读出文件、两遍恢复不同……）、按用例自己的展开现算的数——
/// 那一条断言这一趟判了没有。钉值按用例自己的展开算（全量域）：等于钉的数，或「全量域上多于某个数」（至少一个、多于一个、多于另一格）；
/// 缩过域的那一趟数出来的本来就比它少，同 [`ClosedFormStateCountJudgement`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FullDomainCountJudgement {
    /// 没缩域：判了。
    Judged,
    /// 缩过域：这一条不判；每个崩溃块照判（在枚举那一趟里判，与这一条无关）。
    NotJudgedBecauseTheDomainWasNarrowed {
        /// 这一趟的展开上限（变量的值）。
        expand_up_to: usize,
        /// 这一趟实际展开、数出来的（`Debug` 的写法）。
        counted: String,
        /// 全量域上要的：钉的数（`Debug` 的写法）、[`FULL_DOMAIN_EXPECTS_ABOVE_ZERO`]，或 [`FULL_DOMAIN_EXPECTS_ABOVE_PREFIX`] 接下界。
        full_domain_expects: String,
    },
}

/// 「全量域上至少一个」那一种断言在结果行 `full_domain_expects=` 里的写法。
pub const FULL_DOMAIN_EXPECTS_ABOVE_ZERO: &str = "above_zero";

/// 「全量域上多于下界」那一种断言在结果行 `full_domain_expects=` 里的写法：这个头接下界的十进制（`above_1`）。
pub const FULL_DOMAIN_EXPECTS_ABOVE_PREFIX: &str = "above_";

impl FullDomainCountJudgement {
    /// 缩过域的那一趟打的结果行：`CRASH_CASE_FULL_DOMAIN_COUNT_NOT_JUDGED expand_up_to=<n> counted=<这一趟数出来的> full_domain_expects=<全量域上要的> what=<哪一条>`；
    /// 判了的那一趟不打（`None`）。
    #[must_use]
    pub fn line(&self, what: &str) -> Option<String> {
        match self {
            Self::Judged => None,
            Self::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to,
                counted,
                full_domain_expects,
            } => Some(format!(
                "{FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX} expand_up_to={expand_up_to} counted={counted} full_domain_expects={full_domain_expects} what={what}"
            )),
        }
    }
}

/// 崩溃枚举用例里按全量域钉了数的计数（评估数、观察者计数、oracle 与记录核对器那几格、按全量的计划现算的数；一个数，或几个数拼成的一组）
/// 那一条断言，各用例都经它判：照这个进程的环境
/// （[`ExpandUpToWritesPerSegment::narrowing_from_the_environment`]），没缩域照判，缩过域不判、往 stdout 打结果行。
/// 和同一趟数出来的别的数比的（`== tally.states`、几格之和等于态数）缩域时照样成立，不经它。
///
/// # Panics
/// 没缩域而 `counted` 不等于 `pinned`；缩域变量写坏了（同 [`ExpandUpToWritesPerSegment::narrowing_from_the_environment`]）。
#[track_caller]
pub fn assert_the_count_equals_its_full_domain_pin_unless_narrowed<Counted, Pinned>(
    counted: Counted,
    pinned: Pinned,
    what: &str,
) -> FullDomainCountJudgement
where
    Counted: PartialEq<Pinned> + std::fmt::Debug,
    Pinned: std::fmt::Debug,
{
    let judgement = count_equal_to_its_full_domain_pin_judged_under(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment(),
        counted,
        pinned,
        what,
    );
    if let Some(line) = judgement.line(what) {
        println!("{line}");
    }
    judgement
}

/// 同 [`assert_the_count_equals_its_full_domain_pin_unless_narrowed`]，缩域上限由调用方给（`None` 是没缩），不读环境、不打行。
///
/// # Panics
/// `narrowing` 是 `None` 而 `counted` 不等于 `pinned`。
#[track_caller]
pub fn count_equal_to_its_full_domain_pin_judged_under<Counted, Pinned>(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    counted: Counted,
    pinned: Pinned,
    what: &str,
) -> FullDomainCountJudgement
where
    Counted: PartialEq<Pinned> + std::fmt::Debug,
    Pinned: std::fmt::Debug,
{
    match narrowing {
        None => {
            assert_eq!(counted, pinned, "{what}：数出来的要等于按全量域钉的数");
            FullDomainCountJudgement::Judged
        }
        Some(limit) => FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
            expand_up_to: limit.0,
            counted: format!("{counted:?}"),
            full_domain_expects: format!("{pinned:?}"),
        },
    }
}

/// 崩溃枚举用例里「全量域上至少一个状态评估过这条不变量 / 跑到过这一格观察者计数」那一条断言，各用例都经它判：
/// 照这个进程的环境，没缩域照判，缩过域不判（缩掉的段里的那几格可以一个都不剩）、往 stdout 打结果行。
///
/// # Panics
/// 没缩域而 `counted` 是 0；缩域变量写坏了。
#[track_caller]
pub fn assert_the_count_is_above_zero_unless_narrowed(
    counted: u64,
    what: &str,
) -> FullDomainCountJudgement {
    let judgement = count_above_zero_judged_under(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment(),
        counted,
        what,
    );
    if let Some(line) = judgement.line(what) {
        println!("{line}");
    }
    judgement
}

/// 同 [`assert_the_count_is_above_zero_unless_narrowed`]，缩域上限由调用方给（`None` 是没缩），不读环境、不打行。
///
/// # Panics
/// `narrowing` 是 `None` 而 `counted` 是 0。
#[track_caller]
pub fn count_above_zero_judged_under(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    counted: u64,
    what: &str,
) -> FullDomainCountJudgement {
    match narrowing {
        None => {
            assert!(counted > 0, "{what}：全量域上至少一个，数出来的是 0");
            FullDomainCountJudgement::Judged
        }
        Some(limit) => FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
            expand_up_to: limit.0,
            counted: counted.to_string(),
            full_domain_expects: FULL_DOMAIN_EXPECTS_ABOVE_ZERO.to_string(),
        },
    }
}

/// 崩溃枚举用例里「全量域上数出来的多于下界」那一条断言（多于一个状态、态数多于读得出新号的那一格这类：下界是 1，或同一趟数出来的另一格），
/// 各用例都经它判：照这个进程的环境，没缩域照判，缩过域不判（缩掉的段里可以一个都不剩，或只剩那一格的状态）、往 stdout 打结果行。
///
/// # Panics
/// 没缩域而 `counted` 不多于 `floor`；缩域变量写坏了。
#[track_caller]
pub fn assert_the_count_exceeds_its_full_domain_floor_unless_narrowed(
    counted: u64,
    floor: u64,
    what: &str,
) -> FullDomainCountJudgement {
    let judgement = count_exceeding_its_full_domain_floor_judged_under(
        ExpandUpToWritesPerSegment::narrowing_from_the_environment(),
        counted,
        floor,
        what,
    );
    if let Some(line) = judgement.line(what) {
        println!("{line}");
    }
    judgement
}

/// 同 [`assert_the_count_exceeds_its_full_domain_floor_unless_narrowed`]，缩域上限由调用方给（`None` 是没缩），不读环境、不打行。
///
/// # Panics
/// `narrowing` 是 `None` 而 `counted` 不多于 `floor`。
#[track_caller]
pub fn count_exceeding_its_full_domain_floor_judged_under(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    counted: u64,
    floor: u64,
    what: &str,
) -> FullDomainCountJudgement {
    match narrowing {
        None => {
            assert!(
                counted > floor,
                "{what}：全量域上要多于 {floor}，数出来的是 {counted}"
            );
            FullDomainCountJudgement::Judged
        }
        Some(limit) => FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
            expand_up_to: limit.0,
            counted: counted.to_string(),
            full_domain_expects: format!("{FULL_DOMAIN_EXPECTS_ABOVE_PREFIX}{floor}"),
        },
    }
}

/// 这一份的块里一共几个状态（`CRASH_CASE_BLOCKS` 行的 `states=`；出口 10「每项态数现数」：判段的块账顺带数，录入段本来就排了计划）：
/// 块是点内序号区间，状态数是区间长，块号的数法与 [`blocks_of_this_host`] 相同。分片的段（C790）只数这一趟的片，分片跑的流数出来的是这一趟那一截。
fn states_of_the_blocks_of_this_share(plan: &CrashPlan, is_mine: &dyn Fn(u64) -> bool) -> u64 {
    let mut states = 0u64;
    let mut block_sequence = 0u64;
    for point in &plan.points {
        for local in point.blocks() {
            if is_mine(block_sequence) {
                states += local.end - local.start;
            }
            block_sequence += 1;
        }
    }
    states
}

/// 块账里挂着的一个观察者：计划里每个点它挂没挂（与计划的点同序）、这一趟它那一把结论版本。
struct ObserverInTheBlockAccount {
    holds_each_point: Vec<bool>,
    conclusion: ConclusionVersion,
}

/// 挂着的每个观察者在块账里的样子（[`ObserverInTheBlockAccount`]），与 `observers_on_the_nodes` 同序。
///
/// # Panics
/// 观察者与它们的结论版本不一样多。
fn observers_in_the_block_account(
    plan: &CrashPlan,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    observer_conclusion_versions: &[ConclusionVersion],
) -> Vec<ObserverInTheBlockAccount> {
    assert_eq!(
        observers_on_the_nodes.len(),
        observer_conclusion_versions.len(),
        "每个观察者各一把结论版本、同序"
    );
    observers_on_the_nodes
        .iter()
        .zip(observer_conclusion_versions)
        .map(|(observer, conclusion)| ObserverInTheBlockAccount {
            holds_each_point: plan
                .points
                .iter()
                .map(|point| {
                    observer
                        .nodes
                        .iter()
                        .any(|node| node.holds_the_point(point))
                })
                .collect(),
            conclusion: *conclusion,
        })
        .collect()
}

/// 挂在计划第 `point_index` 个点上的观察者：（在 `observers` 里的位置，观察者结论的种类与这一把版本）。不挂这个点的观察者（只挂点名段的）
/// 在这个点上不要结论：判段收工问缺不缺、重建计数读结论都只问它。
fn observers_holding_the_point(
    observers: &[ObserverInTheBlockAccount],
    point_index: usize,
) -> Vec<(usize, (ConclusionKind, ConclusionVersion))> {
    observers
        .iter()
        .enumerate()
        .filter(|(_position, observer)| observer.holds_each_point[point_index])
        .map(|(position, observer)| (position, (ConclusionKind::Observer, observer.conclusion)))
        .collect()
}

/// 库里点到的那几块的账（[`blocks_in_the_library`]）。红块一块都不在罩住的里头（记着红的不复用、下一趟现判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BlocksInTheLibrary {
    blocks: u64,
    /// 罩住的块：判器结论可复用，挂在这个点上的每个观察者都有这一把版本的结论、且不记着红。
    covered: u64,
    /// 判器结论或任一观察者结论里有违例的块。
    red: u64,
    states: u64,
}

/// 只读库、一块不判：`is_counted` 点到的块里几块罩住、几块红。一块算罩住要两样都在：判器结论照判段「要不要判」同一个谓词在
/// （`ConclusionsStoredUnder::is_stored_for_reuse`，连同判器交回的逐盘自证槽数；记着红的不算）；挂在这个点上的每个观察者在
/// 这一把版本下都有结论、且不记着红（观察者版本换了、结论记着红就不算：判段照 `crate::crash_amplification` 观察者那一半同一个规矩现拼）。
/// 红块是判器结论里有违例、或任一观察者结论里存着违例的块；红块一块都不算罩住。判段的块账照它数。
///
/// # Errors
/// 库读失败。
fn blocks_in_the_library(
    store: &VerdictStore,
    plan: &CrashPlan,
    judged_conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    observers: &[ObserverInTheBlockAccount],
    is_counted: &dyn Fn(u64) -> bool,
) -> Result<BlocksInTheLibrary, String> {
    let mut account = BlocksInTheLibrary {
        blocks: 0,
        covered: 0,
        red: 0,
        states: states_of_the_blocks_of_this_share(plan, is_counted),
    };
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let observer_conclusions_of_the_point: Vec<(ConclusionKind, ConclusionVersion)> = observers
            .iter()
            .filter(|observer| observer.holds_each_point[point_index])
            .map(|observer| (ConclusionKind::Observer, observer.conclusion))
            .collect();
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_counted(this_block) {
                continue;
            }
            account.blocks += 1;
            let key = point.block_key(&local);
            let judgement_is_reusable = judged_conclusions
                .is_stored_for_reuse(store, point_index, &key)
                .map_err(|error| format!("查判器结论失败：{error:?}"))?;
            let mut observer_conclusions_not_reusable = store
                .conclusions_missing_for_block(&key, &observer_conclusions_of_the_point)
                .map_err(|error| format!("查观察者结论失败：{error:?}"))?;
            let mut is_red = !judged_conclusions
                .violations_in_block(store, point_index, &key)
                .map_err(|error| format!("读违例失败：{error:?}"))?
                .is_empty();
            for (kind, version) in &observer_conclusions_of_the_point {
                let observed = store
                    .read_conclusion_block(&key, kind, version)
                    .map_err(|error| format!("读观察者结论失败：{error:?}"))?;
                if observed.is_some_and(|observed| !observed.violations.is_empty()) {
                    is_red = true;
                    // 记着红的观察者结论不复用、下一趟现判（「记红、撤绿、下趟现判」）：与缺的一样不算罩住
                    observer_conclusions_not_reusable.push((kind.clone(), *version));
                }
            }
            if judgement_is_reusable && observer_conclusions_not_reusable.is_empty() {
                account.covered += 1;
            }
            if is_red {
                account.red += 1;
            }
        }
    }
    Ok(account)
}

/// 两段过库，键由调用方给（单测造拒算用）：在长期库里录、判（录入键命中就不再录）；判器那一半哪几组拒算，
/// 那几组照判不入库（开库之后登记它们的占位版本，键里带着它们的行只留在这个进程里），别的照键入库同一份库，块全部计入 `refused=`
/// （4-2a 报告第四节第 7 条后半句）。交回判段的结局与这一趟的块账（只录那一趟是 `None`）。
///
/// # Errors
/// 开不了库、写不进、读不回；判段的错。
fn enumerate_the_flow_through_the_library_with_the_keys(
    item: CrashPointReplayItem<'_>,
    flow_to_record: &FlowToRecord<'_>,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    route: CrashPointReplayJudgeRoute,
    keys: KeysOfTheItem,
) -> Result<(StageOutcome<CrashPointReplayTally>, Option<CrashCaseBlocks>), String> {
    let item_name = item_name_of(item);
    let recorded_bytes = flow_to_record.recorded_bytes();
    let gpu_judges = route == CrashPointReplayJudgeRoute::Gpu;
    let inputs = FlowToRecord::recording_inputs_digest(&recorded_bytes, gpu_judges);
    for sentence in &keys.refusal_sentences {
        println!(
            "CRASH_CASE_BLOCKS_KEY_REFUSED item={} {sentence}（这一组照判、不入库，别的照键入库同一份库）",
            item.name
        );
    }
    let library = CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration)?;
    library
        .lock()
        .keep_the_rows_of_these_versions_out_of_the_library(
            keys.versions_kept_out_of_the_library.iter().copied(),
        );
    let stage = RunStage::of_this_process();
    let recording_key_of_this_run = recording_key_of_the_inputs(&inputs);
    let observer_versions = &keys.observer_versions;
    let blocks_of_the_run: std::cell::Cell<Option<CrashCaseBlocks>> = std::cell::Cell::new(None);
    let outcome = two_stages_under_the_recording_key_topping_up_a_reused_recording(
        &library,
        stage,
        &item_name,
        &recording_key_of_this_run,
        |library| {
            let manifest = flow_to_record.record_into(library, gpu_judges, &recorded_bytes)?;
            if !observers_on_the_nodes.is_empty() {
                // 观察者的录入段（「观察者改走两段」）：这台机器领的块逐状态看、结果写进库，判段从库里读
                flow_to_record.with_the_flow(|flow| {
                    let plan = plan_crash_points(flow);
                    let is_mine = blocks_of_this_host(&plan)?;
                    record_these_observers_and_print_the_line(
                        library,
                        flow,
                        &plan,
                        &|block| is_mine.contains(&block),
                        item,
                        observers_on_the_nodes,
                        observer_versions,
                    )
                })?;
            }
            Ok(manifest)
        },
        |library| {
            record_the_observers_missing_from_a_reused_recording(
                library,
                flow_to_record,
                item,
                observers_on_the_nodes,
                observer_versions,
            )
        },
        |library, manifest| {
            // 配了 GPU 而这份二进制没编进 gpu 特性：判段不判、报本次未跑，交只录（录入段上面照录了），带特性那一趟从库里读回在卡上判
            if route == CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature {
                println!("{}", crash_point_replay_judge_not_run_line(item.name));
                return Ok(StageOutcome::RecordedOnly);
            }
            judge_the_recorded_item(
                library,
                manifest,
                item,
                observers_on_the_nodes,
                observer_versions,
                route,
                &JudgingUnitsOfTheRun {
                    are_refused: keys.are_refused(),
                    judge_is_refused: keys.judge_is_refused,
                    blocks_of_the_run: &blocks_of_the_run,
                },
            )
        },
    )?;
    drop(library);
    Ok((outcome, blocks_of_the_run.get()))
}

/// 判段这一趟判器那一半键取没取得出，与它把块账交回到哪。
struct JudgingUnitsOfTheRun<'run> {
    /// 有一组拒算：那几组照判不入库，块全部计入 `refused=`。
    are_refused: bool,
    /// 判器那一组拒算：判器的结论按这个进程的占位存、照判不入库（开库之后已登记）。
    judge_is_refused: bool,
    blocks_of_the_run: &'run std::cell::Cell<Option<CrashCaseBlocks>>,
}

/// 观察者的录入段（「观察者改走两段」）：给的这几个观察者在这台机器领的块上逐状态看、结果写进库（库里这一把观察者版本
/// 录过的块不再录，[`record_the_observers_of_the_flow`]），打 `CRASH_POINT_REPLAY_OBSERVERS_RECORDED` 一行。
///
/// # Errors
/// 库读写失败。
fn record_these_observers_and_print_the_line(
    library: &CrashLibrary,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    item: CrashPointReplayItem<'_>,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
) -> Result<(), String> {
    let started = Instant::now();
    let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
    let recording = record_the_observers_of_the_flow(
        &mut library.lock(),
        flow,
        plan,
        observers,
        versions,
        is_mine,
        &|| stop_request.is_requested(),
    )?;
    println!(
        "CRASH_POINT_REPLAY_OBSERVERS_RECORDED item={} observers={} blocks_recorded_here={} states_recorded_here={} blocks_already_recorded={} states_counted_here={} images_observed_fresh={} states_reusing_an_observed_image={} buckets={} witness_misses={} stopped_early={} seconds={:.1}",
        item.name,
        observers
            .iter()
            .map(|observer| observer.observer_name.as_str())
            .collect::<Vec<&str>>()
            .join(","),
        recording.blocks_recorded_here,
        recording.states_recorded_here,
        recording.blocks_already_recorded,
        recording.states_counted_here,
        recording.images_observed_here,
        recording.states_reusing_an_observed_image,
        recording.buckets,
        recording.witness_misses,
        recording.stopped_early,
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

/// 录制复用时补录观察者：流的录入键只由流定（[`FlowToRecord::recording_inputs_digest`]），挂着的观察者的版本不进键，命中之后录入那一段
/// 整段不跑；观察者换了版本（代码改了、另读的输入改了、换了一个观察者）而流没变，判段就读不到它录入段的块。这里在录入段里逐个观察者核
/// 它录入段的块都在库里（[`observers_whose_recorded_blocks_are_missing`]），缺块的那几个照录（只录缺的观察者，流与判表已在库里、键只由内容定，
/// 不整段重录），打 `CRASH_POINT_REPLAY_OBSERVERS_CHECKED_ON_A_REUSED_RECORDING` 一行点名缺块的观察者。
///
/// # Errors
/// 读写库失败；分片与权重写法错。
fn record_the_observers_missing_from_a_reused_recording(
    library: &CrashLibrary,
    flow_to_record: &FlowToRecord<'_>,
    item: CrashPointReplayItem<'_>,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
) -> Result<(), String> {
    if observers.is_empty() {
        return Ok(());
    }
    flow_to_record.with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let blocks_this_host_takes = blocks_of_this_host(&plan)?;
        let is_mine = |block: u64| blocks_this_host_takes.contains(&block);
        let missing = observers_whose_recorded_blocks_are_missing(
            &library.lock(),
            &plan,
            observers,
            versions,
            &is_mine,
        )?;
        println!(
            "CRASH_POINT_REPLAY_OBSERVERS_CHECKED_ON_A_REUSED_RECORDING item={} observers={} observers_missing_blocks={}",
            item.name,
            observers.len(),
            missing
                .iter()
                .map(|position| observers[*position].observer_name.as_str())
                .collect::<Vec<&str>>()
                .join(",")
        );
        // 迭代上界：缺块的观察者个数（不多于观察者个数）
        for position in missing {
            record_these_observers_and_print_the_line(
                library,
                flow,
                &plan,
                &is_mine,
                item,
                std::slice::from_ref(&observers[position]),
                std::slice::from_ref(&versions[position]),
            )?;
        }
        Ok(())
    })
}

/// 这台机器领的块（块序号 = 全部点的块按次序编号）：两台分片与崩溃放量同一种读法，按权重平滑轮转。
///
/// # Errors
/// 分片与权重写法错。
fn blocks_of_this_host(plan: &CrashPlan) -> Result<HashSet<u64>, String> {
    let (share, host_weights) =
        crate::crash_amplification::host_share_and_weights_from_the_environment()?;
    let host_shares = WeightedShares::new(host_weights);
    let total_blocks: u64 = plan
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数装得进 u64"))
        .sum();
    Ok((0..total_blocks)
        .filter(|block| host_shares.owner_of_block(*block) == share.index)
        .collect())
}

/// 一条崩溃枚举用例的两个节点：它自己那个点（`<项名>.<周期>.<步序>` 都是它的）与末节点，设点文件就是用例文件。
fn nodes_of_the_item(item: CrashPointReplayItem<'_>) -> Vec<NodeOfAnObserver> {
    vec![
        NodeOfAnObserver {
            setting_file: item.setting_file.to_string(),
            entry_name: item.name.to_string(),
        },
        NodeOfAnObserver {
            setting_file: item.setting_file.to_string(),
            entry_name: END_OF_FLOW_NODE_NAME.to_string(),
        },
    ]
}

/// 一条崩溃枚举用例自己挂的观察者照各自的挂法挂（[`CrashPointReplayObserverAttachment`]：整条流是它自己那个点与末节点，点名段的是罩着那几段的点；
/// 观察者代码文件是用例文件或点名的共用文件）；按来历挂的另由
/// `crate::crash_point_replay_provenance::observers_on_the_nodes_with_the_ones_attached_to_the_flow` 接在后面（整条流）。
///
/// # Errors
/// 观察者名字不合规矩；挂法点名的段集合是空的、段下标越过流的段数、点名的段一个点都罩不着。
fn observers_on_the_nodes_of_the_item<'observer>(
    item: CrashPointReplayItem<'_>,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    observers: &[AttachedCrashPointReplayObserver<'observer>],
    observers_handing_back_images: &[AttachedCrashPointReplayObserverHandingBackAnImage<
        'observer,
    >],
) -> Result<Vec<ObserverOnTheNodes<'observer>>, String> {
    let observer_name = |name: &str| {
        ObserverName::new(name).map_err(|rejected| format!("观察者名字不合规矩：{rejected:?}"))
    };
    let mut observers_on_the_nodes = Vec::new();
    for attached in observers {
        let observer = attached.observer;
        observers_on_the_nodes.push(ObserverOnTheNodes {
            observer_name: observer_name(observer.name)?,
            observer_code_file: observer_code_file_of_the_attachment(item, attached.attachment)
                .to_string(),
            extra_inputs_digest: extra_inputs_digest_of_the_attachment(
                observer.extra_inputs_digest,
                attached.attachment.points,
            ),
            nodes: nodes_of_the_attachment(item, writes, segments, attached.attachment.points)?,
            observe: ObserverCode::CountsOnly(observer.observe),
        });
    }
    for attached in observers_handing_back_images {
        let observer = attached.observer;
        observers_on_the_nodes.push(ObserverOnTheNodes {
            observer_name: observer_name(observer.name)?,
            observer_code_file: observer_code_file_of_the_attachment(item, attached.attachment)
                .to_string(),
            extra_inputs_digest: extra_inputs_digest_of_the_attachment(
                observer.extra_inputs_digest,
                attached.attachment.points,
            ),
            nodes: nodes_of_the_attachment(item, writes, segments, attached.attachment.points)?,
            observe: ObserverCode::HandsBackAnImage {
                observe: observer.observe,
                deduplication: observer.deduplication,
            },
        });
    }
    Ok(observers_on_the_nodes)
}

/// 观察者代码文件：用例文件，或挂法点名的共用文件。
fn observer_code_file_of_the_attachment<'file>(
    item: CrashPointReplayItem<'file>,
    attachment: CrashPointReplayObserverAttachment<'file>,
) -> &'file str {
    match attachment.code_file {
        CrashPointReplayObserverCodeFile::TheCaseFile => item.setting_file,
        CrashPointReplayObserverCodeFile::SharedFile(shared_file) => shared_file,
    }
}

/// 观察者挂在哪几个节点上：整条流是它自己那个点与末节点（[`nodes_of_the_item`]），点名段的是罩着那几段的点（[`nodes_of_the_points_over_the_segments`]）。
///
/// # Errors
/// 同 [`nodes_of_the_points_over_the_segments`]。
fn nodes_of_the_attachment(
    item: CrashPointReplayItem<'_>,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    points: CrashPointReplayObserverPoints<'_>,
) -> Result<Vec<NodeOfAnObserver>, String> {
    match points {
        CrashPointReplayObserverPoints::EveryPointOfTheFlow => Ok(nodes_of_the_item(item)),
        CrashPointReplayObserverPoints::PointsOverTheSegments(named_segments) => {
            nodes_of_the_points_over_the_segments(item, writes, segments, named_segments)
        }
    }
}

/// 罩着点名那几段的点（照 [`crash_points_of_the_steps`] 同一种切法，点名就是计划里那个点的名字），一点一个节点；不含末节点。
///
/// # Errors
/// 段集合是空的；段下标越过流的段数；点名的段一个写都没有、一个点都罩不着。
fn nodes_of_the_points_over_the_segments(
    item: CrashPointReplayItem<'_>,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    named_segments: &[usize],
) -> Result<Vec<NodeOfAnObserver>, String> {
    if named_segments.is_empty() {
        return Err(
            "观察者挂的段集合是空的：一个点都不挂的观察者什么都不看，不挂它就是".to_string(),
        );
    }
    if let Some(outside) = named_segments
        .iter()
        .find(|segment_index| **segment_index >= segments.len())
    {
        return Err(format!(
            "观察者挂的段 {outside} 不在这条流里（流一共 {} 段）",
            segments.len()
        ));
    }
    let named_writes: HashSet<usize> = named_segments
        .iter()
        .flat_map(|segment_index| segments[*segment_index].iter().copied())
        .collect();
    let nodes: Vec<NodeOfAnObserver> = crash_points_of_the_steps(item, writes, segments)
        .into_iter()
        .filter(|point| {
            point
                .writes
                .clone()
                .any(|write| named_writes.contains(&write))
        })
        .map(|point| NodeOfAnObserver {
            setting_file: point.code_file_in_repository,
            entry_name: point.name,
        })
        .collect();
    if nodes.is_empty() {
        return Err(format!(
            "观察者挂的段 {named_segments:?} 一个写都没有，一个点都罩不着"
        ));
    }
    Ok(nodes)
}

/// 进观察者版本的另读的输入：整条流照原样（结论键不动），点名段的并上段集合（[`extra_inputs_digest_with_the_named_segments`]）。
fn extra_inputs_digest_of_the_attachment(
    extra_inputs_digest: [u8; DIGEST_BYTES],
    points: CrashPointReplayObserverPoints<'_>,
) -> [u8; DIGEST_BYTES] {
    match points {
        CrashPointReplayObserverPoints::EveryPointOfTheFlow => extra_inputs_digest,
        CrashPointReplayObserverPoints::PointsOverTheSegments(named_segments) => {
            extra_inputs_digest_with_the_named_segments(extra_inputs_digest, named_segments)
        }
    }
}

/// 点名的段集合（去重、排序：同一组段换个写法版本不变）并上观察者自己另读的输入，取 SHA-256。
fn extra_inputs_digest_with_the_named_segments(
    extra_inputs_digest: [u8; DIGEST_BYTES],
    named_segments: &[usize],
) -> [u8; DIGEST_BYTES] {
    let named_segments_in_order: std::collections::BTreeSet<usize> =
        named_segments.iter().copied().collect();
    sha256_digest(
        format!(
            "singlefs crash state enumeration observer attached to named segments\nsegments={}\nextra_inputs={}\n",
            named_segments_in_order
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(","),
            hexadecimal_text(&extra_inputs_digest)
        )
        .as_bytes(),
    )
}

/// 判段：凭清单读回流，排计划，这台机器领的、库里还没有这一把版本结论的块上卡判或由 CPU 判、存进库；观察者照样按块看、存；
/// 对账、核计数向量，从库里读回每一块的结论重建崩溃点重放的计数。
#[allow(
    clippy::too_many_lines,
    reason = "读回、选路、版本、分工、判、看、对账、重建计数、打行，是判段的全部步骤，拆开要把十几个量来回传"
)]
#[allow(
    clippy::too_many_arguments,
    reason = "库、清单、项、观察者与它们的版本、选路、仓根、这一趟判器那一半键的结局，各是各的东西"
)]
fn judge_the_recorded_item(
    library: &CrashLibrary,
    manifest: &ItemManifest,
    item: CrashPointReplayItem<'_>,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    observer_versions: &[ObserverVersions],
    route: CrashPointReplayJudgeRoute,
    judging_units_of_the_run: &JudgingUnitsOfTheRun<'_>,
) -> Result<StageOutcome<CrashPointReplayTally>, String> {
    let started = Instant::now();
    let read_back = RecordedFlowReadBack::read_from_the_library(
        &library.lock(),
        &manifest.digest_named(MANIFEST_FLOW)?,
    )?;
    let flow_of_the_library = read_back.flow();
    let flow = &flow_of_the_library;
    let plan = plan_crash_points(flow);
    // 判器那一组拒算：照开库之前那一次判定（keys_of_the_item）取这个进程的占位、照判不入库，不在这里再取一次键——取得出也不改按真键存
    // （判定与写读同一份输入）；没拒算就照判法单元指纹存进长期库，组成清单随键登进库
    let conclusion_versions = if judging_units_of_the_run.judge_is_refused {
        let (conclusion_versions, _refusals) =
            ConclusionVersionsOfThePlan::of_the_plan_keeping_the_refused_out(
                &plan,
                toolchain_of_this_process(),
                route.judged_by(),
            );
        let conclusion_versions =
            conclusion_versions.with_the_judge_refused_by_the_caller(route.judged_by());
        library
            .lock()
            .keep_the_rows_of_these_versions_out_of_the_library(
                conclusion_versions.versions_kept_out_of_the_library(),
            );
        conclusion_versions
    } else {
        let conclusion_versions = ConclusionVersionsOfThePlan::of_the_plan(
            &plan,
            toolchain_of_this_process(),
            route.judged_by(),
        )
        .map_err(|not_writable| {
            let line = not_writable.not_written_line();
            println!("{line}");
            line
        })?;
        let mut store = library.lock();
        for judging_units in conclusion_versions.judging_units_keys() {
            store
                .store_judging_units_composition(judging_units)
                .map_err(|error| format!("组成清单写不进库：{error:?}"))?;
        }
        if let Some(line) = changed_units_line_of_the_stale_verdicts(
            &store,
            &plan,
            &conclusion_versions,
            route.judged_by(),
        )
        .map_err(|error| format!("读库里别的判法单元指纹失败：{error:?}"))?
        {
            println!("{line} item={}", item.name);
        }
        conclusion_versions
    };
    let observer_conclusion_versions: Vec<ConclusionVersion> = observer_versions
        .iter()
        .map(|versions| versions.conclusion)
        .collect();
    let judged_by = route.judged_by();
    let judged_conclusions = conclusion_versions.stored_under(judged_by);
    // 两台分片与崩溃放量同一种读法：这台机器领哪几块按权重平滑轮转
    let (share, host_weights) =
        crate::crash_amplification::host_share_and_weights_from_the_environment()?;
    let host_shares = WeightedShares::new(host_weights);
    let total_blocks: u64 = plan
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数装得进 u64"))
        .sum();
    let selected: HashSet<u64> = (0..total_blocks)
        .filter(|block| host_shares.owner_of_block(*block) == share.index)
        .collect();
    let is_mine = |block: u64| selected.contains(&block);
    let (worker_threads, worker_threads_source) = crash_point_replay_worker_threads();
    let mut store = library.lock();
    // 块账（CRASH_CASE_BLOCKS）点这一份领的块（两台各领一份时，别的机器判的块直接写进服务那一份，不分片的那一趟照库复用、罩整条流）；
    // 判之前照库数一遍罩住的，没罩住的就是这一趟要判的
    let is_counted = |block: u64| is_mine(block);
    let observers_in_the_account = observers_in_the_block_account(
        &plan,
        observers_on_the_nodes,
        &observer_conclusion_versions,
    );
    let blocks_before_judging = blocks_in_the_library(
        &store,
        &plan,
        &judged_conclusions,
        &observers_in_the_account,
        &is_counted,
    )?;
    let pending = pending_blocks(&store, &plan, &judged_conclusions, &is_mine)
        .map_err(|error| format!("读判定块失败：{error:?}"))?;
    let blocks_of_this_share = u64::try_from(selected.len()).expect("块数装得进 u64");
    let blocks_to_judge = u64::try_from(pending.len()).expect("块数装得进 u64");
    let blocks_reused = blocks_of_this_share - blocks_to_judge;
    let state_count = plan
        .points
        .last()
        .map_or(0, |end_of_flow| end_of_flow.ordinals.end);
    let share_fields = if share.count > 1 {
        format!(
            " shard={}/{} all_blocks={total_blocks}",
            share.index, share.count
        )
    } else {
        String::new()
    };
    println!(
        "CRASH_POINT_REPLAY_PARALLEL_START states={state_count} slices={blocks_of_this_share} worker_threads={worker_threads} configured_worker_threads={worker_threads} worker_threads_source={worker_threads_source} resumed_slices={blocks_reused} freshly_run_slices={blocks_to_judge} judge={} library={} item={}{share_fields}",
        route.name(),
        library.library_name(),
        item.name
    );
    let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
    let stop_is_requested = || stop_request.is_requested();
    // 复用哨兵（与崩溃放量那一路同一套：`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」每一趟真跑插哨兵与强制现判）：
    // 开跑前从这一份里键相同、库里有不红结论的块里抽几块当哨兵、再抽一块强制现判，种子打进日志；判完照它判（哨兵被现判、强制现判仍复用、
    // 强制现判出的与库里那一份不同，这条用例红）。块账与 resumed / freshly_run 在这之前数（强制现判那一块按复用算）
    let reuse_sentinel_count = crate::crash_amplification::reuse_sentinel_count_of_this_process();
    let reuse_sentinels = crate::crash_amplification::plan_the_reuse_sentinels(
        &mut store,
        &plan,
        &judged_conclusions,
        &is_mine,
        reuse_sentinel_count,
        crate::crash_amplification::reuse_sentinel_seed_of_this_process(),
        &crate::crash_amplification::observers_for_the_reuse_sentinels(
            &plan,
            observers_on_the_nodes,
            observer_versions,
        ),
    )
    .map_err(|error| format!("抽复用哨兵读库失败：{error:?}"))?;
    if let Some(sentinels) = &reuse_sentinels {
        println!("{}", sentinels.line(item.name, reuse_sentinel_count));
        // 强制现判那一块上观察者要读的录入段那一行不在库里：回录入段只给它重录（判段要现拼它上面的观察者结论）
        if crate::crash_amplification::record_the_observers_of_the_forced_block_again(
            &mut store,
            flow,
            &plan,
            observers_on_the_nodes,
            observer_versions,
            sentinels,
            &stop_is_requested,
        )? {
            println!(
                "CRASH_POINT_REPLAY_REUSE_SENTINEL_FORCED_BLOCK_RECORDED_AGAIN item={}",
                item.name
            );
        }
    }
    match route {
        CrashPointReplayJudgeRoute::Cpu { why_not_the_gpu } => {
            println!(
                "CRASH_POINT_REPLAY_JUDGE route=cpu why_not_the_gpu={why_not_the_gpu} threads={worker_threads}"
            );
            check_blocks_where_until_with_threads(
                &mut store,
                flow,
                &plan,
                &is_mine,
                &judged_conclusions,
                &stop_is_requested,
                worker_threads,
            )
            .map_err(|error| format!("CPU 判块失败：{error:?}"))?;
        }
        CrashPointReplayJudgeRoute::Gpu => judge_the_pending_blocks_on_the_cards(
            &mut store,
            flow,
            &plan,
            manifest,
            &is_mine,
            &judged_conclusions,
            &stop_is_requested,
        )?,
        CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature => unreachable!(
            "判段不跑的那一趟在 enumerate_crash_states_through_the_library 的判段闭包里先交只录，不进判段"
        ),
    }
    // 观察者的判段：只从库里读录入段那几块，观察者交回的盘上的池级 checker 照第 ① 段的选路判（配了 GPU 上卡）。
    // 叫停了不判观察者：录入段可能停在块边上，没录的块判段读不到，交回叫停那一句，重跑同一条用例接着录、接着判
    // （哨兵与强制现判那一块可能没跑到，照「没跑到不判」只清掉强制现判表）
    if !observers_on_the_nodes.is_empty() && stop_is_requested() {
        store.force_these_conclusion_blocks_to_be_judged_again([]);
        return Err(
            "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE）：观察者这一趟没判完，录过、判过的在库里，重跑同一条用例接着录、接着判"
                .to_string(),
        );
    }
    let pool_checker = pool_checker_of_the_images_on_the_route(route)?;
    let observers_judging_started = Instant::now();
    // 判器与观察者两段都判完才判哨兵（观察者结论也要核）：次序在两处判段共用的那一处
    let (observing, reuse_sentinel_problems) =
        crate::crash_amplification::judge_the_observers_then_the_reuse_sentinels(
            &mut store,
            &plan,
            Some((observers_on_the_nodes, observer_versions, &pool_checker)),
            &is_mine,
            reuse_sentinels.as_ref(),
        )?;
    let observing = observing.expect("交了观察者那一格，观察者判段的结果就在");
    if !observers_on_the_nodes.is_empty() {
        // 观察者判段的「现判 N、复用 M」：这一趟在观察者交回的盘上判池级 checker 的张数（其中在卡上的）、库里这一把判器版本早先判过的张数
        println!(
            "CRASH_POINT_REPLAY_OBSERVERS_JUDGED item={} images_judged_fresh={} images_judged_on_the_gpu_here={} image_judgements_reused={} blocks_observed_here={} states_observed_here={} seconds={:.1}",
            item.name,
            observing.images_judged_here,
            observing.images_judged_on_the_gpu_here,
            observing.image_judgements_reused,
            observing.blocks_observed_here,
            observing.states_observed_here,
            observers_judging_started.elapsed().as_secs_f64()
        );
    }
    // 被叫停的那一趟：哨兵与强制现判那一块可能没跑到，照「没跑到不判」只清掉强制现判表（problems_after_judging 已清）
    if let Some(message) =
        reuse_sentinel_red_message(item.name, stop_is_requested(), &reuse_sentinel_problems)
    {
        return Err(message);
    }
    // 这一份自己领的块上的观察者红当场红（排在双机那一份收工之前：分片那一份也报自己那一半；别的机器领的块上存着的红由不分片的那一趟
    // 从整条流的观察者结论里读，见下面重建计数那一步）
    if let Some(message) = red_message_of_the_observers_on_the_blocks_of_this_share(&observing) {
        drop(store);
        panic!("{message}");
    }
    // 对账：这条流的块里还没有这一把版本结论的
    let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &judged_conclusions)
        .map_err(|error| format!("对账失败：{error:?}"))?;
    if !unjudged.is_empty() && stop_is_requested() {
        return Err(format!(
            "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE）：{} 块还没判，判过的在库里，重跑同一条用例接着判",
            unjudged.len()
        ));
    }
    // 观察者判段只判这台机器领的块：别的机器领的块这一把版本的观察者结论由那台机器写进服务那一份；只挂点名段的观察者在它不挂的点上不要结论
    let without_every_conclusion = blocks_of_the_flow_without_every_conclusion(
        &store,
        &plan,
        &judged_conclusions,
        &observers_in_the_account,
    )?;
    if this_share_leaves_the_tally_to_the_whole_flow_run(
        share.count,
        unjudged.len(),
        without_every_conclusion.missing_an_observer_conclusion,
    ) {
        judging_units_of_the_run
            .blocks_of_the_run
            .set(Some(crash_case_blocks_of_the_share(
                &store,
                &plan,
                &judged_conclusions,
                &observers_in_the_account,
                &is_counted,
                blocks_before_judging,
                judging_units_of_the_run.are_refused,
            )?));
        let blocks_left: u64 =
            without_every_conclusion.missing_the_judgement_or_an_observer_conclusion;
        println!(
                "CRASH_POINT_REPLAY_PARALLEL_FINISHED states={state_count} slices={blocks_of_this_share} worker_threads={worker_threads} configured_worker_threads={worker_threads} worker_threads_source={worker_threads_source} resumed_slices={blocks_reused} freshly_run_slices={blocks_to_judge} progress_file_after_completion=none elapsed_seconds={:.1} judge={}{share_fields} blocks_left_to_other_hosts={blocks_left}",
                started.elapsed().as_secs_f64(),
                route.name()
            );
        return Ok(StageOutcome::JudgedMyShareOnly {
            blocks_judged_here: blocks_to_judge,
            blocks_left_to_other_hosts: blocks_left,
        });
    }
    if !unjudged.is_empty() {
        return Err(format!(
            "判段判完库里还有 {} 块没有这一把版本的结论（第一块：{}）：判器那一路没写库",
            unjudged.len(),
            unjudged[0].0.short_text()
        ));
    }
    // 计数向量每一趟都判（J7）
    let disagreements =
        count_vector_disagreements_of_the_flow(&store, flow, &plan, &judged_conclusions, &|_| true)
            .map_err(|error| format!("读计数向量失败：{error:?}"))?;
    if !disagreements.is_empty() {
        return Err(format!(
            "计数向量与计划对不上 {} 处：{}",
            disagreements.len(),
            disagreements.join("；")
        ));
    }
    judging_units_of_the_run
        .blocks_of_the_run
        .set(Some(crash_case_blocks_of_the_share(
            &store,
            &plan,
            &judged_conclusions,
            &observers_in_the_account,
            &is_counted,
            blocks_before_judging,
            judging_units_of_the_run.are_refused,
        )?));
    let (tally, observer_red_of_the_flow) = crash_point_replay_tally_from_the_library(
        &store,
        flow,
        &plan,
        judged_by.conclusion_kind(),
        conclusion_versions.versions_of_each_point_when_judged_by(judged_by),
        &observers_in_the_account,
    )?;
    drop(store);
    // 整条流的块上观察者结论里存着的红（别的机器写进服务的、库里早先存着的、这一趟看的块都读）：别的机器领的块上的观察者红在这里报
    if let Some(message) = red_message_of_the_observers_on_the_whole_flow(
        &observer_red_of_the_flow,
        observers_on_the_nodes,
    ) {
        panic!("{message}");
    }
    report_the_findings(&tally, item, state_count);
    println!(
        "CRASH_POINT_REPLAY_PARALLEL_FINISHED states={state_count} slices={blocks_of_this_share} worker_threads={worker_threads} configured_worker_threads={worker_threads} worker_threads_source={worker_threads_source} resumed_slices={blocks_reused} freshly_run_slices={blocks_to_judge} progress_file_after_completion=none elapsed_seconds={:.1} judge={}{share_fields}",
        started.elapsed().as_secs_f64(),
        route.name()
    );
    Ok(StageOutcome::Judged(tally))
}

/// 判完之后复用哨兵判出的红（[`crate::crash_amplification::ReuseSentinelsOfTheRun::problems_after_judging`]，一句一处）连成这条用例的那一句；
/// 没有红、或这一趟被叫停（哨兵与强制现判那一块可能没跑到，照「没跑到不判」）是 `None`。
fn reuse_sentinel_red_message(
    item_name: &str,
    stop_was_requested: bool,
    problems: &[String],
) -> Option<String> {
    (!stop_was_requested && !problems.is_empty()).then(|| {
        format!(
            "崩溃枚举用例 {item_name} 复用哨兵判红：{}",
            problems.join("；")
        )
    })
}

/// 这一份自己领的块上观察者判红的那一句（这一趟看的与库里复用的块都算，`crate::crash_amplification::judge_the_observers_of_the_flow` 只走这一份的块）；没有红是 `None`。
fn red_message_of_the_observers_on_the_blocks_of_this_share(
    observing: &ObservingOfTheRun,
) -> Option<String> {
    if let Some(red) = observing.red_texts.first() {
        return Some(format!(
            "观察者在 {} 个状态上 panic（第一个：路径 {} 点内第 {} 个状态）：{}",
            observing.red_states_of_each_observer.values().sum::<u64>(),
            red.0,
            red.1,
            red.2
        ));
    }
    observing
        .red_states_of_each_observer
        .iter()
        .find(|(_, red_states)| **red_states > 0)
        .map(|(observer_name, red_states)| {
            format!(
                "观察者 {observer_name} 在 {red_states} 个状态上 panic（这一趟复用的块里存着红项，这一趟没看：改了观察者代码它的版本就换、会重看）"
            )
        })
}

/// 整条流的块上观察者结论里存着红的那一句（[`ObserverRedOfTheFlow`]，`observers` 与重建计数时给的观察者同序）；没有红是 `None`。
///
/// # Panics
/// 有观察者的红状态数不为 0 而没记下头一块存着红的（数的那一步漏记，是这里的 bug）。
fn red_message_of_the_observers_on_the_whole_flow(
    observer_red: &ObserverRedOfTheFlow,
    observers: &[ObserverOnTheNodes<'_>],
) -> Option<String> {
    let (observer_position, red_states) = observer_red
        .red_states_of_each_observer
        .iter()
        .enumerate()
        .find(|(_, red_states)| **red_states > 0)?;
    let first_red_block = observer_red
        .first_red_block
        .as_ref()
        .expect("有观察者的红状态数不为 0，数的时候就记下了头一块存着红的");
    Some(format!(
        "观察者 {} 在整条流的块上存着 {red_states} 个红状态（头一块存着红的：观察者 {} 点 {} 块 {}；别的机器写进服务的、库里早先存着的与这一趟看的块都读）",
        observers[observer_position].observer_name.as_str(),
        observers[first_red_block.observer_position]
            .observer_name
            .as_str(),
        first_red_block.point_name,
        first_red_block.block_start
    ))
}

/// 双机的一份（`share_count` 大于 1）：判器结论或观察者结论有一种对这条流不齐（别的机器领的块那台还没判完、没写进服务那一份），
/// 就只交自己那一份（`StageOutcome::JudgedMyShareOnly`），整条流的计数留给两份都判完之后不分片的那一趟（它照服务里那一份复用两台判过的块）。
/// 判器结论全复用而观察者版本这一趟变了时，缺的只有观察者结论，也在这里收工（双机四条用例两份都 panic 在重建计数读不到对方那几块的
/// 观察者结论）。整份自己跑的不收工：缺结论照旧交错。
fn this_share_leaves_the_tally_to_the_whole_flow_run(
    share_count: usize,
    blocks_without_the_judgement: usize,
    blocks_missing_an_observer_conclusion: usize,
) -> bool {
    share_count > 1
        && (blocks_without_the_judgement > 0 || blocks_missing_an_observer_conclusion > 0)
}

/// 这条流的块里还缺结论的块数（[`blocks_of_the_flow_without_every_conclusion`]）。
struct BlocksOfTheFlowWithoutEveryConclusion {
    /// 挂着的观察者里至少一个在这一趟那一把版本下没有结论的块。
    missing_an_observer_conclusion: usize,
    /// 判器结论或观察者结论缺一种的块（两种都缺的算一块）：双机的一份收工时留给别的机器的块数。
    missing_the_judgement_or_an_observer_conclusion: u64,
}

/// 逐块问判器结论（`judged_conclusions` 那一处）与挂在这一块那个点上的每个观察者结论（[`observers_holding_the_point`]）在不在库里，
/// 只问在不在、不读内容。
///
/// # Errors
/// 库读失败。
fn blocks_of_the_flow_without_every_conclusion(
    store: &VerdictStore,
    plan: &CrashPlan,
    judged_conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    observers: &[ObserverInTheBlockAccount],
) -> Result<BlocksOfTheFlowWithoutEveryConclusion, String> {
    let mut missing_an_observer_conclusion = 0usize;
    let mut missing_the_judgement_or_an_observer_conclusion = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let observer_kinds_and_versions: Vec<(ConclusionKind, ConclusionVersion)> =
            observers_holding_the_point(observers, point_index)
                .into_iter()
                .map(|(_position, kind_and_version)| kind_and_version)
                .collect();
        for local in point.blocks() {
            let key = point.block_key(&local);
            let is_missing_an_observer_conclusion = !store
                .conclusions_missing_for_block(&key, &observer_kinds_and_versions)
                .map_err(|error| format!("读观察者的结论在不在失败：{error:?}"))?
                .is_empty();
            let is_missing_the_judgement = !judged_conclusions
                .is_stored(store, point_index, &key)
                .map_err(|error| format!("读判器的结论在不在失败：{error:?}"))?;
            if is_missing_an_observer_conclusion {
                missing_an_observer_conclusion += 1;
            }
            if is_missing_an_observer_conclusion || is_missing_the_judgement {
                missing_the_judgement_or_an_observer_conclusion += 1;
            }
        }
    }
    Ok(BlocksOfTheFlowWithoutEveryConclusion {
        missing_an_observer_conclusion,
        missing_the_judgement_or_an_observer_conclusion,
    })
}

/// 这一趟的块账：点到的块（`is_counted`：这一份领的块，不分片的那一趟是整条流）、复用（判之前库里罩住的，
/// `blocks_before_judging`）、现判（判之前没罩住的）、判红（判完之后判器结论或观察者结论里有违例的块，[`blocks_in_the_library`]）；
/// 判器那一半哪一组拒算时点到的块全部计入拒算（判了，拒算那几组照判不入库，这条用例的块在库里罩不全）。
///
/// # Errors
/// 库读失败。
fn crash_case_blocks_of_the_share(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    observers: &[ObserverInTheBlockAccount],
    is_counted: &dyn Fn(u64) -> bool,
    blocks_before_judging: BlocksInTheLibrary,
    judging_units_are_refused: bool,
) -> Result<CrashCaseBlocks, String> {
    let blocks_after_judging =
        blocks_in_the_library(store, plan, conclusions, observers, is_counted)?;
    // 点到的块（这一份领的块；不分片的那一趟是整条流）与判之前没罩住的块
    let blocks_of_this_share = blocks_before_judging.blocks;
    let blocks_judged_here = blocks_before_judging.blocks - blocks_before_judging.covered;
    let (reused, fresh, refused) = if judging_units_are_refused {
        (0, 0, blocks_of_this_share)
    } else {
        (
            blocks_of_this_share - blocks_judged_here,
            blocks_judged_here,
            0,
        )
    };
    Ok(CrashCaseBlocks {
        blocks: blocks_of_this_share,
        reused,
        fresh,
        red: blocks_after_judging.red,
        refused,
        states: blocks_before_judging.states,
        expand_up_to: None,
    })
}

/// 观察者交回的盘上的池级 checker 照第 ① 段的选路：CPU 那一路由 CPU 判；GPU 那一路取这一趟的卡（同 [`judge_the_pending_blocks_on_the_cards`]）。
///
/// # Errors
/// 选路走 GPU 而取不到卡。
fn pool_checker_of_the_images_on_the_route(
    route: CrashPointReplayJudgeRoute,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    match route {
        CrashPointReplayJudgeRoute::Cpu { .. } => {
            Ok(PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu)
        }
        CrashPointReplayJudgeRoute::Gpu => pool_checker_of_the_images_on_the_cards(),
        CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature => unreachable!(
            "判段不跑的那一趟在 enumerate_crash_states_through_the_library 的判段闭包里先交只录，不判观察者交回的盘"
        ),
    }
}

#[cfg(feature = "gpu")]
fn pool_checker_of_the_images_on_the_cards(
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    use crate::crash_amplification::gpu_cards_in_use;
    use crate::gpu_card_configuration::cards_by_priority_from_configuration_or_environment;
    let configuration = crate::crash_amplification::multi_host_configuration_of_this_process()?;
    let cards_by_priority = cards_by_priority_from_configuration_or_environment(Some(
        configuration.configuration_file.as_path(),
    ))?;
    let cards_of_this_run = gpu_cards_in_use(cards_by_priority.as_ref())?;
    Ok(PoolCheckerOfTheImagesAfterTheObservers::OnTheGpu {
        cards: cards_of_this_run.cards,
        taken_from: cards_of_this_run.taken_from,
    })
}

/// 没编进 `gpu` 特性选路不会走 GPU（[`crash_point_replay_judge_route`]）。
#[cfg(not(feature = "gpu"))]
fn pool_checker_of_the_images_on_the_cards(
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    Err("这份二进制没编进 gpu 特性，观察者交回的盘上不了卡（选路不该要它）".to_string())
}

/// 配了 GPU：读回清单里的判表、取卡、这台机器领的块上卡判。卡用不上、显存不够都报错，不退回 CPU。
#[cfg(feature = "gpu")]
#[allow(
    clippy::too_many_arguments,
    reason = "库、流、计划、清单、分工、判法单元指纹、叫停，各是各的东西"
)]
fn judge_the_pending_blocks_on_the_cards(
    store: &mut VerdictStore,
    flow: &crate::crash_amplification::FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    manifest: &ItemManifest,
    is_mine: &dyn Fn(u64) -> bool,
    judged_conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(), String> {
    use crate::crash_amplification::{
        gpu_cards_in_use, judge_my_blocks_on_gpu, packed_judge_tables_read_back,
        GpuJudgingOfMyBlocks,
    };
    use crate::gpu_card_configuration::cards_by_priority_from_configuration_or_environment;
    let tables =
        packed_judge_tables_read_back(store, &manifest.digest_named(MANIFEST_JUDGE_TABLES)?)?;
    let configuration = crate::crash_amplification::multi_host_configuration_of_this_process()?;
    let cards_by_priority = cards_by_priority_from_configuration_or_environment(Some(
        configuration.configuration_file.as_path(),
    ))?;
    let cards_of_this_run = gpu_cards_in_use(cards_by_priority.as_ref())?;
    match judge_my_blocks_on_gpu(
        store,
        flow,
        plan,
        &tables,
        is_mine,
        judged_conclusions,
        &cards_of_this_run.cards,
        cards_of_this_run.taken_from,
        stop_is_requested,
    )? {
        GpuJudgingOfMyBlocks::Judged { tally, outcome } => {
            println!(
                "CRASH_POINT_REPLAY_JUDGE route=gpu cards_used={} blocks={} states={} red_states={} compile_seconds={:.1} judge_seconds={:.1} states_per_second={:.0}",
                outcome.cards_used,
                outcome.blocks_judged,
                outcome.states_judged,
                tally.red_states,
                outcome.compile_seconds,
                outcome.judge_seconds,
                outcome.states_per_second()
            );
            Ok(())
        }
        GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason } => Err(format!(
            "配了 GPU 而显存不够（判了 {} 块就停）：{reason}",
            tally.blocks_checked
        )),
    }
}

/// 没编进 `gpu` 特性选路不会要卡（[`crash_point_replay_judge_route`]）。
#[cfg(not(feature = "gpu"))]
#[allow(clippy::too_many_arguments, reason = "与编进 gpu 特性那一份同一组参数")]
fn judge_the_pending_blocks_on_the_cards(
    _store: &mut VerdictStore,
    _flow: &crate::crash_amplification::FlowOfTheLibrary<'_>,
    _plan: &CrashPlan,
    _manifest: &ItemManifest,
    _is_mine: &dyn Fn(u64) -> bool,
    _judged_conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    _stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(), String> {
    Err("这份二进制没编进 gpu 特性，上不了卡（选路不该要它）".to_string())
}

// ============================== 从库里重建崩溃点重放的计数 ==============================

/// 每个状态归哪一段、哪次发布：按段的序号区间现算，不逐状态造镜像。
struct PositionsOfTheStates {
    ranges_by_segment: Vec<Range<u64>>,
    publish_of_segment: Vec<CrashPointReplayPublishOfState>,
    state_count: u64,
}

impl PositionsOfTheStates {
    fn of_the_plan(flow: &CrashFlow<'_>, plan: &CrashPlan) -> Self {
        let expansion = |segment_index: usize, segment: &[usize]| {
            plan.segment_expansion(flow, segment_index, segment)
        };
        let (ranges_by_segment, state_count) =
            crash_state_ranges_by_segment(flow.base, flow.writes, flow.segments, &expansion);
        Self {
            ranges_by_segment,
            publish_of_segment: publish_of_each_segment(flow.writes, flow.segments),
            state_count,
        }
    }

    /// 序号 `ordinal` 的状态在哪一段、归哪次发布。
    fn position_of(
        &self,
        ordinal: u64,
    ) -> (
        CrashPointReplaySegmentOfState,
        CrashPointReplayPublishOfState,
    ) {
        if ordinal + 1 == self.state_count {
            return (
                CrashPointReplaySegmentOfState::AllPersisted,
                CrashPointReplayPublishOfState::EveryWritePersisted,
            );
        }
        let segment_index = self
            .ranges_by_segment
            .iter()
            .position(|range| range.contains(&ordinal))
            .unwrap_or_else(|| {
                panic!(
                    "序号 {ordinal} 不在任何一段的区间里（状态数 {}）",
                    self.state_count
                )
            });
        (
            CrashPointReplaySegmentOfState::Segment(segment_index),
            self.publish_of_segment[segment_index],
        )
    }
}

/// 库里存的一条红项拆成它说的事。红项的写法与 `crate::crash::CrashStateJudgement::red_items`、`crate::crash_judge_gpu::gpu_verdict_items`
/// 同一份：`<pass>:<名字>`，观察者的是 `observer:<观察者名>:<panic 的那一句>`。
enum RedItem {
    ConsultedOracle(CrashPointReplayOracleViolationKind),
    IgnoredOracle(CrashPointReplayOracleViolationKind),
    PoolChecker(&'static str),
    RecordRootWithoutRecord,
    RecordClaimedStateMissingUnit,
    Observer,
}

fn red_item_of(text: &str) -> Result<RedItem, String> {
    let (pass, rest) = text
        .split_once(':')
        .ok_or_else(|| format!("库里的红项 {text:?} 不是 <pass>:<名字> 的写法"))?;
    match pass {
        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE => {
            CrashPointReplayOracleViolationKind::from_name(rest)
                .map(RedItem::ConsultedOracle)
                .ok_or_else(|| format!("oracle 违例种类认不得：{text:?}"))
        }
        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE => {
            CrashPointReplayOracleViolationKind::from_name(rest)
                .map(RedItem::IgnoredOracle)
                .ok_or_else(|| format!("oracle 违例种类认不得：{text:?}"))
        }
        CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER => IMPLEMENTED_INVARIANTS
            .iter()
            .find(|invariant| **invariant == rest)
            .map(|invariant| RedItem::PoolChecker(invariant))
            .ok_or_else(|| format!("池级 checker 的不变量不在清单里：{text:?}")),
        CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER => match rest {
            RECORD_CHECKER_ROOT_WITHOUT_RECORD => Ok(RedItem::RecordRootWithoutRecord),
            RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT => Ok(RedItem::RecordClaimedStateMissingUnit),
            _ => Err(format!("记录核对器的判据认不得：{text:?}")),
        },
        "observer" => Ok(RedItem::Observer),
        _ => Err(format!("库里的红项 {text:?} 的 pass 认不得")),
    }
}

/// 从库里读回这条流每一块这一种结论（`kind`，每个点那一把 `version_of_each_point`）与各观察者的结论，重建崩溃点重放的计数：
/// 逐状态的红项给违例数、记录核对器两条、第一处与发现表（签名的样本原文是红项的名字：GPU 判的状态没有 CPU 那一句原文）；
/// 恢复字给两遍的结局（没有文件、读到文件、失败）与两遍不一致的状态数（按落到的根、结局码与失败点比）；计数向量给逐条不变量判到、
/// 违例、不适用与按发布分的态数；「根槽已持久」按被判根的位置事后数；观察者计数从挂在那个点上的观察者的结论块读（只挂点名段的观察者在它不挂的点上
/// 不读，[`observers_holding_the_point`]），看过的状态数是至少一个观察者挂着的点上的状态。整条流的块上观察者结论里存着的违例
/// 另交回（[`ObserverRedOfTheFlow`]）：别的机器写进服务的、库里早先存着的、这一趟看的块都读，别的机器领的块上的观察者红靠它报。
///
/// # Errors
/// 库读失败；某一块没有这一种结论（判段没写）；红项解不出；判定向量编号没登记。
fn crash_point_replay_tally_from_the_library(
    store: &VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    kind: ConclusionKind,
    version_of_each_point: &[ConclusionVersion],
    observers: &[ObserverInTheBlockAccount],
) -> Result<(CrashPointReplayTally, ObserverRedOfTheFlow), String> {
    let positions = PositionsOfTheStates::of_the_plan(flow, plan);
    let mut tally = CrashPointReplayTally::default();
    let mut observer_red = ObserverRedOfTheFlow {
        red_states_of_each_observer: vec![0; observers.len()],
        first_red_block: None,
    };
    let invariant_named = |name: &str| -> Result<&'static str, String> {
        IMPLEMENTED_INVARIANTS
            .iter()
            .find(|invariant| **invariant == name)
            .copied()
            .ok_or_else(|| format!("计数向量里的不变量不在清单里：{name:?}"))
    };
    for (point_index, point) in plan.points.iter().enumerate() {
        let observers_of_the_point = observers_holding_the_point(observers, point_index);
        for local in point.blocks() {
            let key = point.block_key(&local);
            let stored: StoredConclusionBlock = store
                .read_conclusion_block(&key, &kind, &version_of_each_point[point_index])
                .map_err(|error| format!("读结论块失败：{error:?}"))?
                .ok_or_else(|| {
                    format!(
                        "点 {} 块 {} 在库里没有 {} 这一种结论（对账说有：库在判段里被改了）",
                        point.name,
                        local.start,
                        kind.version_text()
                    )
                })?;
            let numbers = stored.block.verdict_vector_number_of_each_state();
            let block_states = u64::try_from(numbers.len()).expect("一块的状态数装得进 u64");
            if block_states != local.end - local.start {
                return Err(format!(
                    "点 {} 块 {} 存着 {block_states} 个状态，计划里是 {}",
                    point.name,
                    local.start,
                    local.end - local.start
                ));
            }
            let recovery_words = stored.recovery_words.as_ref().ok_or_else(|| {
                format!(
                    "点 {} 块 {} 的结论没带恢复字（判器那一路存块时漏了）",
                    point.name, local.start
                )
            })?;
            if recovery_words.len() != numbers.len() {
                return Err(format!(
                    "点 {} 块 {} 的恢复字 {} 个、状态 {} 个",
                    point.name,
                    local.start,
                    recovery_words.len(),
                    numbers.len()
                ));
            }
            for (offset, (number, words)) in numbers.iter().zip(recovery_words).enumerate() {
                let ordinal =
                    point.ordinals.start + local.start + u64::try_from(offset).expect("块内序号");
                let vector = store.verdict_vector_of_number(*number).ok_or_else(|| {
                    format!("判定块里的编号 {} 没在这个库里登记过", number.as_u32())
                })?;
                let items = red_items_of_the_encoded_verdict(vector);
                tally.states += 1;
                let (segment, publish) = positions.position_of(ordinal);
                *tally.states_by_publish.entry(publish).or_insert(0) += 1;
                // 两遍各自的恢复字（落到的根连同完整的 checkpoint_txg、结局、失败点）整份比：与现判的
                // `crash_judge_conclusions::recovery_passes_differ` 同一组字
                let consulted_pass = words.consulted_pass();
                let ignored_pass = words.ignored_pass();
                match consulted_pass.outcome {
                    RECOVERY_OUTCOME_CODE_NO_FILE => tally.no_file_states += 1,
                    RECOVERY_OUTCOME_CODE_FILE_READ => tally.file_read_states += 1,
                    RECOVERY_OUTCOME_CODE_FAILED => tally.failed_states += 1,
                    other => return Err(format!("恢复字里的结局码认不得：{other}")),
                }
                if consulted_pass != ignored_pass {
                    tally.journal_differing_states += 1;
                }
                let mut is_red_state = false;
                let mut violated_invariants: Vec<&'static str> = Vec::new();
                let mut record_check = RecordCheck {
                    root_without_record: false,
                    claimed_state_missing_unit: false,
                };
                for item_text in &items {
                    match red_item_of(item_text)? {
                        RedItem::ConsultedOracle(kind_of_violation) => {
                            tally.violations += 1;
                            is_red_state = true;
                            if tally.first_violation.is_none() {
                                tally.first_violation = Some(item_text.clone());
                            }
                            tally.findings.record_red_state(
                                CrashPointReplayFindingSignature {
                                    red_pass: CrashPointReplayRedPass::JournalConsultedOracle(
                                        kind_of_violation,
                                    ),
                                    segment,
                                    publish,
                                },
                                ordinal,
                                || item_text.clone(),
                            );
                        }
                        RedItem::IgnoredOracle(kind_of_violation) => {
                            tally.ignored_violations += 1;
                            is_red_state = true;
                            if tally.first_ignored_violation.is_none() {
                                tally.first_ignored_violation = Some(item_text.clone());
                            }
                            tally.findings.record_red_state(
                                CrashPointReplayFindingSignature {
                                    red_pass: CrashPointReplayRedPass::JournalIgnoredOracle(
                                        kind_of_violation,
                                    ),
                                    segment,
                                    publish,
                                },
                                ordinal,
                                || item_text.clone(),
                            );
                        }
                        RedItem::PoolChecker(invariant) => {
                            violated_invariants.push(invariant);
                            tally
                                .checker_first_violation
                                .entry(invariant)
                                .or_insert_with(|| item_text.clone());
                        }
                        RedItem::RecordRootWithoutRecord => record_check.root_without_record = true,
                        RedItem::RecordClaimedStateMissingUnit => {
                            record_check.claimed_state_missing_unit = true;
                        }
                        RedItem::Observer => {
                            return Err(format!(
                                "判器那一种结论里出现了观察者的红项 {item_text:?}：观察者的结论另存，不该混进判器的块"
                            ));
                        }
                    }
                }
                if !violated_invariants.is_empty() {
                    is_red_state = true;
                    let text = format!(
                        "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{}",
                        violated_invariants.join(",")
                    );
                    tally.findings.record_red_state(
                        CrashPointReplayFindingSignature {
                            red_pass: CrashPointReplayRedPass::PoolChecker(violated_invariants),
                            segment,
                            publish,
                        },
                        ordinal,
                        || text,
                    );
                }
                if record_check.root_without_record {
                    tally.record_root_without_record += 1;
                }
                if record_check.claimed_state_missing_unit {
                    tally.record_claimed_state_missing_unit += 1;
                }
                if record_check.root_without_record || record_check.claimed_state_missing_unit {
                    is_red_state = true;
                    let text = format!(
                        "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{}",
                        CrashPointReplayRedPass::RecordChecker(record_check)
                            .violated_names()
                            .join(",")
                    );
                    tally.findings.record_red_state(
                        CrashPointReplayFindingSignature {
                            red_pass: CrashPointReplayRedPass::RecordChecker(record_check),
                            segment,
                            publish,
                        },
                        ordinal,
                        || text,
                    );
                }
                if is_red_state {
                    tally.findings.red_states += 1;
                }
            }
            for (invariant_name, counts) in &stored.count_vector.per_invariant {
                let invariant = invariant_named(invariant_name)?;
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += counts.judged;
                *tally.checker_violated_states.entry(invariant).or_insert(0) += counts.violated;
                *tally
                    .checker_not_applicable_states
                    .entry(invariant)
                    .or_insert(0) += counts.not_applicable;
            }
            if !observers_of_the_point.is_empty() {
                tally.observed_states += block_states;
            }
            for (observer_position, (observer_kind, observer_version)) in &observers_of_the_point {
                let observer_position = *observer_position;
                let observed = store
                    .read_conclusion_block(&key, observer_kind, observer_version)
                    .map_err(|error| format!("读观察者的结论失败：{error:?}"))?
                    .ok_or_else(|| {
                        format!(
                            "点 {} 块 {} 在库里没有 {} 的结论（观察者那一路没写库）",
                            point.name,
                            local.start,
                            observer_kind.version_text()
                        )
                    })?;
                for (counter, count) in &observed.count_vector.per_observer {
                    tally.observer_counts.add(counter, *count);
                }
                let red_states_in_the_block =
                    u64::try_from(observed.violations.len()).expect("一块的违例数装得进 u64");
                observer_red.red_states_of_each_observer[observer_position] +=
                    red_states_in_the_block;
                if red_states_in_the_block > 0 && observer_red.first_red_block.is_none() {
                    observer_red.first_red_block = Some(FirstObserverRedBlock {
                        observer_position,
                        point_name: point.name.clone(),
                        block_start: local.start,
                    });
                }
            }
        }
    }
    tally.root_persisted_states = if flow
        .writes
        .get(flow.judged_root_index)
        .is_some_and(|write| write.kind == StepKind::RootRecordFua)
    {
        states_with_the_root_slot_write_persisted(flow, plan, flow.judged_root_index)
    } else {
        return Err(format!(
            "被判的第 {} 次写不是根槽写：崩溃点重放的「根槽已持久」只按根槽写两态数",
            flow.judged_root_index
        ));
    };
    if tally.states != positions.state_count {
        return Err(format!(
            "从库里读回 {} 个状态的结论，计划里整条流 {} 个",
            tally.states, positions.state_count
        ));
    }
    Ok((tally, observer_red))
}

/// 整条流的块上观察者结论里存着的红（[`crash_point_replay_tally_from_the_library`] 重建计数时顺带数）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ObserverRedOfTheFlow {
    /// 每个观察者（与重建计数时给的观察者同序）在整条流的块上存着的红状态数。
    red_states_of_each_observer: Vec<u64>,
    /// 按块的次序头一块存着红的。
    first_red_block: Option<FirstObserverRedBlock>,
}

/// 头一块存着观察者红的：哪一个观察者（重建计数时给的次序里的下标）、点名、块起点（点内序号）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FirstObserverRedBlock {
    observer_position: usize,
    point_name: String,
    block_start: u64,
}

/// 发现表：每个签名一行 `CRASH_POINT_REPLAY_FINDING event=new …`（按最先那个状态排号）、一行 `CRASH_POINT_REPLAY_FINDINGS`；设了发现日志就写一节定稿。
fn report_the_findings(
    tally: &CrashPointReplayTally,
    item: CrashPointReplayItem<'_>,
    state_count: u64,
) {
    for (position, (signature, finding)) in tally
        .findings
        .signatures_in_first_state_order()
        .iter()
        .enumerate()
    {
        println!(
            "{}",
            findings_standard_output_new_signature_line(
                position + 1,
                signature,
                finding.first_sample()
            )
        );
    }
    println!(
        "{}",
        findings_standard_output_summary_line(&tally.findings, state_count)
    );
    let begin = CrashPointReplayFindingsLogBegin {
        stream_name: Some(item.name.to_string()),
        whole_stream_states: state_count,
    };
    if let CrashPointReplayFindingsLog::AppendedTo(path) =
        CrashPointReplayFindingsLog::from_environment()
    {
        let section = CrashPointReplayFindingsLogSection::begin(&path, &begin);
        section.replace_with_final_lines(&findings_log_final_lines(
            &begin,
            &tally.findings,
            state_count,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crash_amplification::CrashAmplificationEnvironment;
    #[cfg(feature = "gpu")]
    use crate::verdict_store::PruneServerFailureHandling;

    #[test]
    fn red_items_stored_by_either_judge_are_read_back_into_their_passes() {
        assert!(matches!(
            red_item_of("journal_consulted_oracle:wrong_content"),
            Ok(RedItem::ConsultedOracle(_))
        ));
        assert!(matches!(
            red_item_of("journal_ignored_oracle:wrong_content"),
            Ok(RedItem::IgnoredOracle(_))
        ));
        assert!(matches!(
            red_item_of(&format!("pool_checker:{}", IMPLEMENTED_INVARIANTS[0])),
            Ok(RedItem::PoolChecker(invariant)) if invariant == IMPLEMENTED_INVARIANTS[0]
        ));
        assert!(matches!(
            red_item_of("record_checker:root_without_record"),
            Ok(RedItem::RecordRootWithoutRecord)
        ));
        assert!(matches!(
            red_item_of("record_checker:claimed_state_missing_unit"),
            Ok(RedItem::RecordClaimedStateMissingUnit)
        ));
        assert!(matches!(
            red_item_of("observer:f5:炸了"),
            Ok(RedItem::Observer)
        ));
        assert!(
            red_item_of("pool_checker:I-0.0").is_err(),
            "清单里没有的不变量要报错"
        );
        assert!(red_item_of("no_colon").is_err());
        assert!(red_item_of("unknown_pass:x").is_err());
    }

    #[test]
    fn the_judge_route_is_named_and_mapped_to_the_conclusion_kind() {
        assert_eq!(CrashPointReplayJudgeRoute::Gpu.name(), "gpu");
        assert_eq!(
            CrashPointReplayJudgeRoute::Gpu.judged_by(),
            JudgedBy::GpuJudge
        );
        let cpu = CrashPointReplayJudgeRoute::Cpu {
            why_not_the_gpu: "gpu_not_enabled_in_the_configuration",
        };
        assert_eq!(cpu.name(), "cpu");
        assert_eq!(cpu.judged_by(), JudgedBy::CpuJudge);
    }

    /// 双机的一份判器结论全复用、别的机器领的块缺这一趟那一把版本的观察者结论（双机四条用例两份都 panic 的那一格）：
    /// 这一份照样只交自己那一份、整条流的计数留给不分片的那一趟；判器结论缺的照旧收工；两种都齐的照常重建计数；整份自己跑的缺观察者结论不收工
    /// （照旧交错）。判别力（`crates/mutations.tsv`）：收工的条件去掉观察者那一半（改回只看判器结论），这条红。
    #[test]
    fn a_share_with_every_judge_conclusion_reused_still_leaves_the_tally_when_an_observer_conclusion_is_missing(
    ) {
        assert!(
            this_share_leaves_the_tally_to_the_whole_flow_run(2, 0, 1),
            "判器结论全齐、别的机器那几块缺观察者结论：只交自己那一份"
        );
        assert!(
            this_share_leaves_the_tally_to_the_whole_flow_run(2, 1, 0),
            "判器结论缺：只交自己那一份（改前就是这样）"
        );
        assert!(
            !this_share_leaves_the_tally_to_the_whole_flow_run(2, 0, 0),
            "两种结论都齐：照常重建计数"
        );
        assert!(
            !this_share_leaves_the_tally_to_the_whole_flow_run(1, 0, 1),
            "整份自己跑：缺观察者结论照旧交错，不收工"
        );
        assert!(
            !this_share_leaves_the_tally_to_the_whole_flow_run(1, 1, 0),
            "整份自己跑：缺判器结论照旧交错，不收工"
        );
    }

    /// 单测小流（[`with_the_unit_writes_flow_and_an_empty_library`]）的项。
    const BLOCK_ACCOUNT_PROBE_ITEM: CrashPointReplayItem<'static> = CrashPointReplayItem {
        name: "block_account_probe",
        setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
    };

    /// 单测的小流（三次单元写一段、一次根槽写一段，9 个状态，三个节点各一块：与 [`the_blocks_line_reports_the_states_in_the_blocks_of_this_share`]
    /// 那一条同一条）与一份空库，交给 `check` 看它的流、计划与库。
    fn with_the_unit_writes_flow_and_an_empty_library<Outcome>(
        label: &str,
        check: impl FnOnce(&CrashFlow<'_>, &CrashPlan, &mut VerdictStore) -> Outcome,
    ) -> Outcome {
        let (writes, segments) = writes_and_segments_of(&[
            &[
                (StepKind::UnitWrite, 1, 0),
                (StepKind::UnitWrite, 2, 0),
                (StepKind::UnitWrite, 3, 0),
            ],
            &[(StepKind::RootRecordFua, 0, 1)],
        ]);
        let base = MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let item = BLOCK_ACCOUNT_PROBE_ITEM;
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        // 一份空库：临时 prune-server 里的一段一次性名字空间（临时 prune-server 由它自己在这个进程里起，起不来当场失败）
        // 一次性库留着：块账的自证（造的结论与版本），要空库
        let disposable = crate::crash_library::DisposableLibrary::new(label)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let mut store = disposable
            .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
            .expect("开得了一次性库");
        flow_of_the_inputs(
            item,
            &base,
            &writes,
            &segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        )
        .with_the_flow(|flow| {
            let plan = plan_crash_points(flow);
            check(flow, &plan, &mut store)
        })
    }

    /// 计划里的每一块，按块的次序：点名、点内序号区间、块键。
    fn blocks_of_the_plan(
        plan: &CrashPlan,
    ) -> Vec<(String, Range<u64>, crate::verdict_store::VerdictBlockKey)> {
        plan.points
            .iter()
            .flat_map(|point| {
                point.blocks().into_iter().map(|local| {
                    let key = point.block_key(&local);
                    (point.name.clone(), local, key)
                })
            })
            .collect()
    }

    /// 单测里存一块一种结论：`states` 个状态，`red_offsets` 那几个状态记一个观察者红项、别的干净；给了 `same_recovery_words_for_every_state`
    /// 就每个状态带这一份恢复字（判器结论重建计数要它）。
    fn store_a_conclusion_block_of_the_unit_test(
        store: &mut VerdictStore,
        key: &crate::verdict_store::VerdictBlockKey,
        kind: &ConclusionKind,
        version: &ConclusionVersion,
        states: u64,
        red_offsets: &[u64],
        same_recovery_words_for_every_state: Option<crate::verdict_store::RecoveryWordsOfState>,
    ) {
        let clean = store
            .register_verdict_vector(
                &crate::crash_judge_conclusions::encoded_verdict_of_the_red_items(&[]),
            )
            .expect("登记得了");
        let red = store
            .register_verdict_vector(
                &crate::crash_judge_conclusions::encoded_verdict_of_the_red_items(&[
                    "observer:unit_test_observer:单测造的红".to_string(),
                ]),
            )
            .expect("登记得了");
        let state_count = usize::try_from(states).expect("一块的状态数装得进 usize");
        store
            .store_conclusion_block(
                key,
                kind,
                version,
                &crate::verdict_store::ConclusionBlockRecord {
                    block: crate::verdict_store::VerdictBlock::new(
                        (0..states)
                            .map(|offset| {
                                if red_offsets.contains(&offset) {
                                    red
                                } else {
                                    clean
                                }
                            })
                            .collect(),
                    )
                    .expect("一块至少一个状态"),
                    violating_states: red_offsets
                        .iter()
                        .map(|offset| crate::verdict_store::StateOffsetInBlock(*offset))
                        .collect(),
                    count_vector: crate::verdict_store::BlockCountVector::default(),
                    judged_on: crate::verdict_store::JudgingDevice::Cpu,
                    recovery_words: same_recovery_words_for_every_state
                        .map(|words| vec![words; state_count]),
                },
            )
            .expect("存得下");
    }

    /// 两台分活的权重（本机、第二台），与双机四条用例那一趟的多机配置相同。
    const HOST_WEIGHTS_OF_THE_UNIT_TEST: [u64; 2] = [2, 3];

    /// 场景：三块的小流，每一块判器结论都存（干净、带恢复字）、观察者结论都存，其中按权重 2,3 归第二台（不归 0/2 那一份）的头一块的观察者结论
    /// 存一个违例状态（第二台那一份判出、写进服务那一份的那种块）。
    /// 预期：从库重建整条流的计数照常读回 9 个状态；整条流观察者红状态数 = 1，头一块存着红的就是那一块。
    /// 判别力（`crates/mutations.tsv`）：累加观察者红状态数那一步加 0（不分片的那一趟读不到第二台领的块上存着的观察者红），这条红。
    #[test]
    fn the_whole_flow_tally_counts_the_observer_red_stored_on_a_block_the_other_host_owns() {
        with_the_unit_writes_flow_and_an_empty_library(
            "observer-red-of-the-whole-flow",
            |flow, plan, store| {
                let judgement_version = ConclusionVersion([1; DIGEST_BYTES]);
                let observer_version = ConclusionVersion([2; DIGEST_BYTES]);
                let host_shares = WeightedShares::new(HOST_WEIGHTS_OF_THE_UNIT_TEST.to_vec());
                let blocks = blocks_of_the_plan(plan);
                let (red_point_name, red_local, red_key) = (0u64..)
                    .zip(&blocks)
                    .find(|(block_sequence, _)| host_shares.owner_of_block(*block_sequence) == 1)
                    .map(|(_, block)| block.clone())
                    .expect("前提：有一块按权重 2,3 归第二台");
                for (_point_name, local, key) in &blocks {
                    let states = local.end - local.start;
                    store_a_conclusion_block_of_the_unit_test(
                        store,
                        key,
                        &ConclusionKind::ImageJudgement,
                        &judgement_version,
                        states,
                        &[],
                        Some(crate::verdict_store::RecoveryWordsOfState(
                            [0; crate::crash_judge_conclusions::RECOVERY_WORDS_PER_STATE],
                        )),
                    );
                    let red_offsets: &[u64] = if *key == red_key { &[0] } else { &[] };
                    store_a_conclusion_block_of_the_unit_test(
                        store,
                        key,
                        &ConclusionKind::Observer,
                        &observer_version,
                        states,
                        red_offsets,
                        None,
                    );
                }
                let (tally, observer_red) = crash_point_replay_tally_from_the_library(
                    store,
                    flow,
                    plan,
                    ConclusionKind::ImageJudgement,
                    &vec![judgement_version; plan.points.len()],
                    &[ObserverInTheBlockAccount {
                        holds_each_point: vec![true; plan.points.len()],
                        conclusion: observer_version,
                    }],
                )
                .expect("库里每一块两种结论都在");
                assert_eq!(tally.states, 9, "前提：整条流 9 个状态都读回");
                assert_eq!(
                    observer_red.red_states_of_each_observer,
                    vec![1],
                    "整条流观察者红状态数：第二台领的那一块上存着的那一个"
                );
                assert_eq!(
                    observer_red.first_red_block,
                    Some(FirstObserverRedBlock {
                        observer_position: 0,
                        point_name: red_point_name,
                        block_start: red_local.start,
                    }),
                    "头一块存着红的就是第二台领的那一块"
                );
            },
        );
    }

    /// 场景：三块的小流，一个观察者挂在头一个点与末节点上（中间那个点不挂）。
    /// ① 判器结论与逐盘自证槽数每一块都存、观察者结论一块都不存；② 再给观察者挂着的块补上观察者结论，其中一块存一个违例状态；
    /// ③ 另一份库：判器结论每一块都存、逐盘自证槽数只缺头一块、不挂观察者。
    /// 预期：① 罩住的只有不挂观察者的那一块（没罩住的数 = 挂观察者的块数 2）；② 罩住 2 块、红 1 块（观察者结论记着红的那一块不罩住：
    /// 下一趟现判「记红、撤绿、下趟现判」），三块都有判定；③ 罩住 2 块（缺逐盘自证槽数的那一块不算）。
    /// 判别力（`crates/mutations.tsv`）：块账不问观察者结论、红块不数观察者结论里的违例、问判器结论不要逐盘自证槽数，各让这一条红。
    #[test]
    fn the_block_account_needs_every_observer_conclusion_and_the_self_verified_slots_and_counts_observer_red(
    ) {
        let _environment =
            crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
        with_the_unit_writes_flow_and_an_empty_library(
            "block-account-with-observers",
            |_flow, plan, store| {
                let judgement_version = ConclusionVersion([1; DIGEST_BYTES]);
                let judgement_version_of_each_point = vec![judgement_version; plan.points.len()];
                let judged_conclusions =
                    crate::crash_amplification::ConclusionsStoredUnder::TheConclusionKind {
                        kind: ConclusionKind::ImageJudgement,
                        version_of_each_point: &judgement_version_of_each_point,
                    };
                let observer_version = ConclusionVersion([2; DIGEST_BYTES]);
                assert_eq!(
                    plan.points.len(),
                    3,
                    "前提：单元写那一步、根槽写那一步、末节点"
                );
                let observers = [ObserverInTheBlockAccount {
                    holds_each_point: vec![true, false, true],
                    conclusion: observer_version,
                }];
                let blocks = blocks_of_the_plan(plan);
                assert_eq!(blocks.len(), 3, "前提：每个节点一块");
                let store_the_judgement_of_every_block =
                    |library_to_fill: &mut VerdictStore, blocks_without_the_slots: &[usize]| {
                        for (block_position, (_point_name, local, key)) in blocks.iter().enumerate()
                        {
                            store_a_conclusion_block_of_the_unit_test(
                                library_to_fill,
                                key,
                                &ConclusionKind::ImageJudgement,
                                &judgement_version,
                                local.end - local.start,
                                &[],
                                None,
                            );
                            if !blocks_without_the_slots.contains(&block_position) {
                                library_to_fill
                                    .store_observer_block(
                                        key,
                                        &ConclusionKind::ImageJudgement,
                                        &judgement_version,
                                        b"self verified slots of the unit test",
                                    )
                                    .expect("存得下");
                            }
                        }
                    };
                store_the_judgement_of_every_block(store, &[]);
                let every_block = |_block: u64| true;
                let without_observer_conclusions = blocks_in_the_library(
                    store,
                    plan,
                    &judged_conclusions,
                    &observers,
                    &every_block,
                )
                .expect("读得了库");
                assert_eq!(
                    (
                        without_observer_conclusions.blocks,
                        without_observer_conclusions.blocks - without_observer_conclusions.covered,
                        without_observer_conclusions.red
                    ),
                    (3, 2, 0),
                    "① 观察者结论全缺：没罩住的数 = 挂观察者的块数"
                );
                for (block_position, (_point_name, local, key)) in blocks.iter().enumerate() {
                    if observers[0].holds_each_point[block_position] {
                        let red_offsets: &[u64] = if block_position == 0 { &[0] } else { &[] };
                        store_a_conclusion_block_of_the_unit_test(
                            store,
                            key,
                            &ConclusionKind::Observer,
                            &observer_version,
                            local.end - local.start,
                            red_offsets,
                            None,
                        );
                    }
                }
                let with_observer_conclusions = blocks_in_the_library(
                    store,
                    plan,
                    &judged_conclusions,
                    &observers,
                    &every_block,
                )
                .expect("读得了库");
                assert_eq!(
                    (
                        with_observer_conclusions.covered,
                        with_observer_conclusions.red
                    ),
                    (2, 1),
                    "② 观察者结论补齐、一块带违例：记着红的那一块不罩住、红 1 块（观察者结论里的违例照数）"
                );
                // 一次性库留着：块账的自证（造的结论与版本），要另一份空库
                let other_library =
                    crate::crash_library::DisposableLibrary::new("block-account-without-the-slots")
                        .unwrap_or_else(|refusal| panic!("{refusal}"));
                let mut other_store = other_library
                    .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
                    .expect("开得了一次性库");
                store_the_judgement_of_every_block(&mut other_store, &[0]);
                let without_the_slots_of_one_block = blocks_in_the_library(
                    &other_store,
                    plan,
                    &judged_conclusions,
                    &[],
                    &every_block,
                )
                .expect("读得了库");
                assert_eq!(
                    without_the_slots_of_one_block.covered,
                    2,
                    "③ 判器结论在而逐盘自证槽数不在的那一块不算罩住（与判段「要不要判」同一个谓词）"
                );
            },
        );
    }

    /// 场景：CPU 路线（没配 GPU），判器那一半的键里 GPU 判器那一组拒算（单测造的取法：只有它拒算）。
    /// 预期：这条用例的判器那一组算拒算（照判、不入库：判段取这个进程的占位、开库之后登记它不入库），拒算一句、点名的就是 GPU 判器那一组，
    /// 照判不入库的版本里有 GPU 判器那一组的占位；整条用例的块全部计入 `refused=`（[`KeysOfTheItem::are_refused`]）。镜像判定的版本由判定定义那一组与
    /// GPU 判器那一组并成，两条路线都取（「GPU 判器代码进键」）：只查 CPU 路线要不要判定定义那一组时，GPU 判器那一组拒算让判段
    /// 报写不了库、整条用例错。
    /// 判别力（`crates/mutations.tsv`）：「崩溃枚举用例：CPU 路线不取 GPU 判器那一组的键」这一行让这一条红。
    #[test]
    fn the_cpu_route_with_the_gpu_judge_group_refused_judges_the_case_without_storing_and_counts_it_refused(
    ) {
        let route = CrashPointReplayJudgeRoute::Cpu {
            why_not_the_gpu: "unit_test",
        };
        let keys = keys_of_the_item_with_the_judge_units_keyed_by(route, &[], |units| {
            if *units == JudgingUnits::GpuJudge {
                Err(
                    crate::crash_identity::JudgingUnitsKeyRefused::NotComputedAtTheBuild {
                        units: JudgingUnits::GpuJudge,
                        reason: "单测造的拒算".to_string(),
                    },
                )
            } else {
                judging_units_key(units, toolchain_of_this_process())
            }
        });
        assert!(
            keys.judge_is_refused,
            "GPU 判器那一组拒算：CPU 路线的判器那一组同样照判、不入库"
        );
        assert!(keys.are_refused(), "这条用例的块全部计入 refused");
        assert_eq!(
            keys.refusal_sentences.len(),
            1,
            "拒算一句（只有 GPU 判器那一组）：{:?}",
            keys.refusal_sentences
        );
        assert!(
            keys.versions_kept_out_of_the_library.contains(
                crate::crash_amplification::placeholder_judging_units_key_of_the_refused(
                    &JudgingUnits::GpuJudge
                )
                .fingerprint()
            ),
            "照判不入库的版本里有 GPU 判器那一组的占位"
        );
    }

    /// 「配 GPU 必用 GPU、做不到报错不降级」：配置开了 GPU 而这份二进制没编进 gpu 特性，判段不跑（不退回 CPU 判），
    /// 结论版本按配置的那一路（GPU）；打的本次未跑那一行是门禁 crash-point-replay 的 `test_not_run_lines` 认的那一族记号、带着项名，
    /// 不是 `cpu_judging_lines` 认的 `route=cpu`。
    #[test]
    fn the_gpu_switched_on_without_the_gpu_feature_routes_the_judge_to_not_run_and_never_to_the_cpu(
    ) {
        assert_eq!(
            crash_point_replay_judge_route_of_the_switches(true, false),
            CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature
        );
        assert_eq!(
            crash_point_replay_judge_route_of_the_switches(true, true),
            CrashPointReplayJudgeRoute::Gpu
        );
        for gpu_feature_is_compiled_in in [false, true] {
            assert!(matches!(
                crash_point_replay_judge_route_of_the_switches(false, gpu_feature_is_compiled_in),
                CrashPointReplayJudgeRoute::Cpu { .. }
            ));
        }
        assert_eq!(
            CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature.judged_by(),
            JudgedBy::GpuJudge
        );
        let line = crash_point_replay_judge_not_run_line("some_crash_enumeration_item");
        assert!(
            line.starts_with(
                "NOT_RUN_NO_GPU_ADAPTER test=crash_state_enumeration::some_crash_enumeration_item reason="
            ) && line.contains("ENABLE_GPU=1"),
            "{line}"
        );
        assert!(
            !line.contains("route=cpu") && !line.contains("judge=cpu"),
            "{line}"
        );
    }

    #[test]
    fn worker_threads_come_from_the_crash_point_replay_variable_then_the_checking_threads_variable_then_the_heavy_thread_default(
    ) {
        assert_eq!(
            worker_threads_of_the_variable_values(Some("3"), Some("5"), Some("7")),
            (3, "environment_variable")
        );
        assert_eq!(
            worker_threads_of_the_variable_values(None, Some("5"), Some("7")),
            (5, "crash_amplification_threads_variable")
        );
        assert_eq!(
            worker_threads_of_the_variable_values(None, None, Some("7")),
            (7, "heavy_thread_default"),
            "两个自己的变量都没设读 SINGLEFS_THREAD_CAP"
        );
        assert_eq!(
            worker_threads_of_the_variable_values(None, None, None),
            (
                crate::heavy_threads::registered_heavy_threads().get(),
                "heavy_thread_default"
            ),
            "都没设取 .claude/heavy-thread-default 的登记值，不取本机核数"
        );
    }

    #[test]
    fn zero_or_unreadable_worker_threads_stop_instead_of_falling_back() {
        for value in ["0", "x", "-1"] {
            let caught_with_the_crash_point_replay_threads_value = std::panic::catch_unwind(|| {
                worker_threads_of_the_variable_values(Some(value), None, None)
            });
            let payload = caught_with_the_crash_point_replay_threads_value
                .err()
                .unwrap_or_else(|| panic!("{value:?} 要停下，不退回 1 个线程"));
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|text| (*text).to_string())
                })
                .unwrap_or_default();
            assert!(
                message.contains("认不出")
                    && message.contains(CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE),
                "消息经共用解析，带「变量=原值 认不出」：{message}"
            );
            let caught_with_the_checking_threads_value = std::panic::catch_unwind(|| {
                worker_threads_of_the_variable_values(None, Some(value), None)
            });
            assert!(
                caught_with_the_checking_threads_value.is_err(),
                "崩溃放量那个变量 {value:?} 同样要停下"
            );
        }
    }

    /// 一次写：一个扇区（512 字节）、各写各的 4 KiB 块；根槽写的根身份写在偏移 24（实例 1）与 28（checkpoint_txg）。
    fn one_write_of(
        write_index: usize,
        kind: StepKind,
        byte: u8,
        checkpoint_txg: u64,
    ) -> RetainedWrite {
        let mut bytes = vec![byte; 512];
        if kind == StepKind::RootRecordFua {
            bytes = vec![0; 512];
            bytes[24..28].copy_from_slice(&1u32.to_le_bytes());
            bytes[28..36].copy_from_slice(&checkpoint_txg.to_le_bytes());
        }
        RetainedWrite {
            device: singlefs_core::address::DeviceIdentity(1),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: singlefs_core::address::DeviceOffsetInBytes(
                u64::try_from(write_index).expect("写表下标装得进 u64") * 4096,
            ),
            contents: crate::memory_pool::WrittenContents::Bytes(bytes),
        }
    }

    /// 一段一组写（种类、内容字节、根槽写的 txg），交回写表与切段。
    fn writes_and_segments_of(
        segments_of_writes: &[&[(StepKind, u8, u64)]],
    ) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
        let mut writes = Vec::new();
        let mut segments = Vec::new();
        for segment_writes in segments_of_writes {
            let mut segment = Vec::new();
            for (kind, byte, checkpoint_txg) in *segment_writes {
                segment.push(writes.len());
                writes.push(one_write_of(writes.len(), *kind, *byte, *checkpoint_txg));
            }
            segments.push(segment);
        }
        (writes, segments)
    }

    fn every_proper_subset(
        _segment_index: usize,
        _segment: &[usize],
    ) -> CrashPointReplaySegmentExpansion {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    }

    /// 一条流每个节点（计划里的点，连同末节点）各自的块键。
    fn block_keys_of_each_node(
        item: CrashPointReplayItem<'_>,
        base: &MemoryPool,
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
    ) -> Vec<Vec<crate::verdict_store::VerdictBlockKey>> {
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .unwrap_or(0);
        flow_of_the_inputs(
            item,
            base,
            writes,
            segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        )
        .with_the_flow(|flow| {
            plan_crash_points(flow)
                .points
                .iter()
                .map(|point| {
                    point
                        .blocks()
                        .into_iter()
                        .map(|local| point.block_key(&local))
                        .collect()
                })
                .collect()
        })
    }

    /// L6：崩溃枚举用例按步切成前缀树的节点，节点键与崩溃放量同一个算法：两条流共前三步（单元写与 journal 记录、根槽写、系统配置槽轮换），
    /// 前三个节点的块键逐个相同，第四步（单元写的内容不同）起不同。判别力（`crates/mutations.tsv`）：把前缀树退成整条流一个点，这条红。
    #[test]
    fn two_flows_sharing_their_first_steps_share_the_keys_of_those_nodes_and_differ_at_the_next() {
        let shared: [&[(StepKind, u8, u64)]; 3] = [
            &[
                (StepKind::UnitWrite, 1, 0),
                (StepKind::UnitWrite, 1, 0),
                (StepKind::JournalRecord, 1, 0),
            ],
            &[(StepKind::RootRecordFua, 0, 1)],
            &[
                (StepKind::SystemConfigurationSlot, 1, 0),
                (StepKind::SystemConfigurationSlot, 1, 0),
            ],
        ];
        let tail_of = |byte: u8| -> Vec<Vec<(StepKind, u8, u64)>> {
            vec![
                vec![
                    (StepKind::UnitWrite, byte, 0),
                    (StepKind::JournalRecord, 2, 0),
                ],
                vec![(StepKind::RootRecordFua, 0, 2)],
            ]
        };
        let flow_with_the_tail = |byte: u8| {
            let tail = tail_of(byte);
            let mut segments_of_writes: Vec<&[(StepKind, u8, u64)]> = shared.to_vec();
            segments_of_writes.extend(tail.iter().map(Vec::as_slice));
            writes_and_segments_of(&segments_of_writes)
        };
        let base = MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let item = CrashPointReplayItem {
            name: "prefix_probe",
            setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
        };
        let (writes_of_a, segments_of_a) = flow_with_the_tail(2);
        let (writes_of_b, segments_of_b) = flow_with_the_tail(3);
        let nodes_of_a = block_keys_of_each_node(item, &base, &writes_of_a, &segments_of_a);
        let nodes_of_b = block_keys_of_each_node(item, &base, &writes_of_b, &segments_of_b);
        assert_eq!(nodes_of_a.len(), 6, "五步五个节点加末节点");
        assert_eq!(nodes_of_b.len(), 6);
        assert!(
            nodes_of_a.iter().all(|keys| !keys.is_empty()),
            "每个节点都有块"
        );
        assert_eq!(
            nodes_of_a[..3],
            nodes_of_b[..3],
            "共有的前三步是同三个节点：块键逐个相同"
        );
        assert_ne!(nodes_of_a[3], nodes_of_b[3], "第四步内容不同，节点键不同");
    }

    /// 出口 10「每项态数现数」的办法 (a)：`CRASH_CASE_BLOCKS` 行带 `states=`，数的是这一份的块里的状态。三次单元写一段、一次根槽写一段的流
    /// （盘上原本没有内容，没有原地覆写，每次写两态），全量展开：每段 2^|段| − 1 个、末节点全部持久 1 个，(2³ − 1) + (2¹ − 1) + 1 = 9 个，
    /// 与崩溃点重放排的状态计划（`crash_state_ranges_by_segment`）数的一样；只领一部分块时只数领到的。
    /// 判别力（`crates/mutations.tsv`）：一块里的状态只记一个，这条红。
    #[test]
    fn the_blocks_line_reports_the_states_in_the_blocks_of_this_share() {
        let (writes, segments) = writes_and_segments_of(&[
            &[
                (StepKind::UnitWrite, 1, 0),
                (StepKind::UnitWrite, 2, 0),
                (StepKind::UnitWrite, 3, 0),
            ],
            &[(StepKind::RootRecordFua, 0, 1)],
        ]);
        let base = MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let item = CrashPointReplayItem {
            name: "states_probe",
            setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
        };
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        let (states_of_every_block, states_of_no_block) = flow_of_the_inputs(
            item,
            &base,
            &writes,
            &segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        )
        .with_the_flow(|flow| {
            let plan = plan_crash_points(flow);
            (
                states_of_the_blocks_of_this_share(&plan, &|_block| true),
                states_of_the_blocks_of_this_share(&plan, &|_block| false),
            )
        });
        assert_eq!(states_of_every_block, 9, "(2³ − 1) + (2¹ − 1) + 1");
        assert_eq!(
            states_of_every_block,
            crash_state_ranges_by_segment(&base, &writes, &segments, &every_proper_subset).1,
            "与状态计划数的一样"
        );
        assert_eq!(states_of_no_block, 0, "一块都没领就是 0 个");
        let line = CrashCaseBlocks {
            blocks: 0,
            reused: 0,
            fresh: 0,
            red: 0,
            refused: 0,
            states: states_of_every_block,
            expand_up_to: None,
        }
        .line();
        assert!(line.ends_with(" states=9"), "{line}");
    }

    /// 不缩域（守卫清掉缩域变量）：各用例经它判的闭式态数断言照判——等于闭式交回判了、不等当场 panic。
    /// 判别力（`crates/mutations.tsv`）：没设变量也当缩过域、闭式不判，这条红在不等那一条闭式断言上。
    #[test]
    fn the_closed_form_assertion_is_judged_when_the_domain_is_not_narrowed() {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        assert_eq!(
            assert_the_counted_states_equal_the_closed_form_unless_narrowed(4u64, 4u64, "等于闭式"),
            ClosedFormStateCountJudgement::Judged,
            "没缩：判了"
        );
        assert!(
            std::panic::catch_unwind(|| {
                assert_the_counted_states_equal_the_closed_form_unless_narrowed(
                    3u64,
                    4u64,
                    "比闭式少一个",
                )
            })
            .is_err(),
            "没缩而枚举到的比闭式少：闭式断言当场 panic"
        );
        assert_eq!(
            ClosedFormStateCountJudgement::Judged.line("等于闭式"),
            None,
            "判了的那一趟不打结果行"
        );
    }

    /// 缩过域（上限由调用方给）：闭式不判，交回上限、实际展开的与全量域的闭式；按发布分的那一串同样经它（`Debug` 的写法）。
    /// 判别力（`crates/mutations.tsv`）：缩过域也照判闭式，这条红（集成测试那一条同红）。
    #[test]
    fn the_closed_form_assertion_is_left_unjudged_on_a_narrowed_domain() {
        assert_eq!(
            closed_form_state_count_judged_under(
                Some(ExpandUpToWritesPerSegment(4)),
                3u64,
                4u64,
                "比闭式少一个"
            ),
            ClosedFormStateCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to: 4,
                counted: "3".to_string(),
                closed_form: "4".to_string(),
            }
        );
        let by_publish = closed_form_state_count_judged_under(
            Some(ExpandUpToWritesPerSegment(4)),
            "instance1_txg1=20 every_write_persisted=1".to_string(),
            "instance1_txg1=26 every_write_persisted=1",
            "按发布分",
        );
        assert_eq!(
            by_publish.line("按发布分"),
            Some(format!(
                "{CLOSED_FORM_NOT_JUDGED_LINE_PREFIX} expand_up_to=4 counted=\"instance1_txg1=20 every_write_persisted=1\" closed_form=\"instance1_txg1=26 every_write_persisted=1\" what=按发布分"
            ))
        );
    }

    /// 按全量域钉的评估数、观察者计数两种断言：没缩域（守卫清掉缩域变量）照判——等于钉的数、至少一个都交回判了，比钉的少、是 0 当场 panic；
    /// 缩过域（守卫设缩域变量 4）两种都不判，交回上限、这一趟数出来的与全量域上要的，结果行逐字。
    /// 判别力（`crates/mutations.tsv`）：钉数那一种不读缩域变量（缩过域也照判）、至少一个那一种不读缩域变量，各让这条红。
    #[test]
    fn the_full_domain_count_assertions_are_judged_unless_the_environment_narrows_the_domain() {
        {
            let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
            assert_eq!(
                assert_the_count_equals_its_full_domain_pin_unless_narrowed(9u64, 9u64, "钉 9"),
                FullDomainCountJudgement::Judged,
                "没缩：钉数那一种判了"
            );
            assert!(
                std::panic::catch_unwind(|| {
                    assert_the_count_equals_its_full_domain_pin_unless_narrowed(
                        8u64,
                        9u64,
                        "比钉的少一个",
                    )
                })
                .is_err(),
                "没缩而数出来的比钉的少：当场 panic"
            );
            assert_eq!(
                assert_the_count_is_above_zero_unless_narrowed(1, "一个"),
                FullDomainCountJudgement::Judged,
                "没缩：至少一个那一种判了"
            );
            assert!(
                std::panic::catch_unwind(|| assert_the_count_is_above_zero_unless_narrowed(
                    0,
                    "一个都没有"
                ))
                .is_err(),
                "没缩而数出来的是 0：当场 panic"
            );
        }
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            crate::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("4"),
        )]);
        let pinned = assert_the_count_equals_its_full_domain_pin_unless_narrowed(
            [3u64, 12],
            [3u64, 13],
            "两格",
        );
        assert_eq!(
            pinned,
            FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to: 4,
                counted: "[3, 12]".to_string(),
                full_domain_expects: "[3, 13]".to_string(),
            },
            "缩过域：钉数那一种不判"
        );
        assert_eq!(
            pinned.line("两格"),
            Some(format!(
                "{FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX} expand_up_to=4 counted=[3, 12] full_domain_expects=[3, 13] what=两格"
            ))
        );
        let above_zero = assert_the_count_is_above_zero_unless_narrowed(0, "缩掉的那一格");
        assert_eq!(
            above_zero.line("缩掉的那一格"),
            Some(format!(
                "{FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX} expand_up_to=4 counted=0 full_domain_expects={FULL_DOMAIN_EXPECTS_ABOVE_ZERO} what=缩掉的那一格"
            )),
            "缩过域：至少一个那一种不判"
        );
        assert_eq!(FullDomainCountJudgement::Judged.line("钉 9"), None);
    }

    /// 「全量域上多于下界」那一种断言：没缩域（守卫清掉缩域变量）照判——多于下界交回判了，等于下界当场 panic；
    /// 缩过域（守卫设缩域变量 4）不判，交回上限、这一趟数出来的与 `above_<下界>`，结果行逐字。
    /// 判别力（`crates/mutations.tsv`「崩溃枚举「全量域上多于下界」的计数断言不读缩域变量（缩过域也照判）」）：不读缩域变量，缩过域那一半当场 panic，这条红。
    #[test]
    fn the_count_exceeding_its_full_domain_floor_is_judged_unless_the_environment_narrows_the_domain(
    ) {
        {
            let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
            assert_eq!(
                assert_the_count_exceeds_its_full_domain_floor_unless_narrowed(2, 1, "多于一个"),
                FullDomainCountJudgement::Judged,
                "没缩：多于下界那一种判了"
            );
            assert!(
                std::panic::catch_unwind(|| {
                    assert_the_count_exceeds_its_full_domain_floor_unless_narrowed(1, 1, "只有一个")
                })
                .is_err(),
                "没缩而数出来的等于下界：当场 panic"
            );
        }
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            crate::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("4"),
        )]);
        let exceeding =
            assert_the_count_exceeds_its_full_domain_floor_unless_narrowed(1, 1, "缩掉的那几段");
        assert_eq!(
            exceeding,
            FullDomainCountJudgement::NotJudgedBecauseTheDomainWasNarrowed {
                expand_up_to: 4,
                counted: "1".to_string(),
                full_domain_expects: "above_1".to_string(),
            },
            "缩过域：多于下界那一种不判"
        );
        assert_eq!(
            exceeding.line("缩掉的那几段"),
            Some(format!(
                "{FULL_DOMAIN_COUNT_NOT_JUDGED_LINE_PREFIX} expand_up_to=4 counted=1 full_domain_expects=above_1 what=缩掉的那几段"
            ))
        );
    }

    /// 缩域：没设上限照用例自己的展开；上限 4 时 5 次写的段不展开、4 次写的段照用例自己的；缩过域那一趟的块账行末尾接 ` expand_up_to=4`，
    /// 没缩的不接。判别力（`crates/mutations.tsv`）：缩了也照用例自己的展开（不取小的那个），这条红。
    #[test]
    fn a_narrowed_case_takes_the_smaller_expansion_and_its_blocks_line_names_the_limit() {
        let limit = Some(ExpandUpToWritesPerSegment(4));
        assert_eq!(
            expansion_of_the_case_narrowed_by(
                None,
                CrashPointReplaySegmentExpansion::EveryProperSubset,
                &[0; 5]
            ),
            CrashPointReplaySegmentExpansion::EveryProperSubset,
            "没缩：照用例自己的"
        );
        assert_eq!(
            expansion_of_the_case_narrowed_by(
                limit,
                CrashPointReplaySegmentExpansion::EveryProperSubset,
                &[0; 5]
            ),
            CrashPointReplaySegmentExpansion::NotExpanded,
            "5 次写超过上限 4：不展开"
        );
        assert_eq!(
            expansion_of_the_case_narrowed_by(
                limit,
                CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll,
                &[0; 4]
            ),
            CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll,
            "4 次写不超过上限：照用例自己的甲二"
        );
        let blocks = CrashCaseBlocks {
            blocks: 3,
            reused: 1,
            fresh: 2,
            red: 0,
            refused: 0,
            states: 9,
            expand_up_to: Some(4),
        };
        assert_eq!(
            blocks.line(),
            "CRASH_CASE_BLOCKS blocks=3 reused=1 fresh=2 red=0 refused=0 states=9 expand_up_to=4"
        );
        assert_eq!(
            CrashCaseBlocks {
                blocks: 3,
                reused: 1,
                fresh: 2,
                red: 0,
                refused: 0,
                states: 9,
                expand_up_to: None,
            }
            .line(),
            "CRASH_CASE_BLOCKS blocks=3 reused=1 fresh=2 red=0 refused=0 states=9",
            "没缩不接"
        );
    }

    /// 这几条库单测的判路照这台机器的多机配置取（配 GPU 必用 GPU、做不到报错不降级；门禁 crash-point-replay 的
    /// `cpu_judging_lines` 判配了 GPU 的判段不许落在 CPU）：配了 GPU 的机器只有编进 gpu 特性的那一趟判得了，所以这几条只编进带特性的那一趟，
    /// 「没编进特性不跑判段」那一路在这里到不了。
    #[cfg(feature = "gpu")]
    fn configured_judge_route() -> CrashPointReplayJudgeRoute {
        match crash_point_replay_judge_route().expect("多机配置读得出") {
            route @ (CrashPointReplayJudgeRoute::Gpu | CrashPointReplayJudgeRoute::Cpu { .. }) => {
                route
            }
            CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature => {
                unreachable!("这几条编进了 gpu 特性，判路不会是「没编进特性不跑判段」")
            }
        }
    }

    /// 两块盘 mkfs 一次的池（io_min 512、槽距 4096，没有文件）：GPU 判表要底座读得出系统配置（`crash_judge_tables::JudgeTablesRefusal::BaseSystemConfigurationUnreadable`），
    /// 没格式化的空池它建不了表。与 `crash_judge_tables` 单测里的造法同一种，两处各留一份（测试模块之间不互相导入）。
    #[cfg(feature = "gpu")]
    fn formatted_pool_of_two_devices_for_the_probe() -> MemoryPool {
        use crate::memory_pool::SparseBlockDevice;
        use singlefs_core::block_device::PhysicalBlockSizeInBytes;
        use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
        use singlefs_core::root_ring::RootRingSlotsPerRegion;
        use singlefs_core::system_configuration::SystemImmutableSizes;
        let device_bytes: u64 = 4 << 30;
        let mut devices: Vec<(singlefs_core::address::DeviceIdentity, SparseBlockDevice)> = [
            singlefs_core::address::DeviceIdentity(0),
            singlefs_core::address::DeviceIdentity(1),
        ]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
        make_filesystem(
            &MakeFilesystemParameters {
                filesystem_identifier: *b"singlefs-probe!!",
                region_devices: [
                    singlefs_core::address::DeviceIdentity(0),
                    singlefs_core::address::DeviceIdentity(1),
                    singlefs_core::address::DeviceIdentity(0),
                ],
                geometry: SystemImmutableSizes {
                    physical_block_size: 512,
                    minimum_input_output_bytes: 512,
                    fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
                    journal_ring_bytes: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                    root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
                },
            },
            &mut devices,
        )
        .expect("两块空盘上 mkfs 做得成");
        MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: device_bytes,
        }
    }

    /// 探针流两次单元写写什么。两次都落在盘 1 单元区头两个槽的起点，那里坐着 mkfs 在单元区起点写下、底座的根引用的单元
    /// （`make_filesystem` 的 `instance_table_placement_at`），写什么决定单元写落下之后的块红不红。
    #[cfg(feature = "gpu")]
    enum ProbeUnitWriteContents {
        /// 两槽各写满一个物理块的填充字节（7 与 8）：盖掉被引用的单元，池级 checker 判 I-2.1、I-4.8、I-7.4，单元写落下之后的块都红。
        FillerBytesOverReferencedUnits,
        /// 两槽各照底座在那里原有的字节重写一个物理块：单元写落下之后的镜像与底座逐字节相同，那几块与什么都没落下的那一块一样判不红。
        TheBaseBytesWrittenAgain,
    }

    /// 探针流的三次写，每次都落在 GPU 判表认的一个落点的起点上（`crash_judge_tables::JudgeTablesRefusal::WriteNotAlignedToALocation` 拒起点不在落点上的写）：
    /// 两次单元写各在单元区头两个槽的起点（盘 1，内容照 `unit_write_contents`，取底座字节的从 `base` 读），一次根槽写在根环区域 0 的 0 号槽（盘 0，长度一个物理块），
    /// 根身份写在偏移 24（实例 1）与 28（checkpoint_txg 1）。
    #[cfg(feature = "gpu")]
    fn probe_writes_on_the_locations_of(
        base: &MemoryPool,
        unit_write_contents: ProbeUnitWriteContents,
    ) -> Vec<RetainedWrite> {
        use crate::memory_pool::WrittenContents;
        use singlefs_core::journal::slot_after_the_journal_ring;
        /// 每次写的字节数：一个物理块（[`formatted_pool_of_two_devices_for_the_probe`] 的池物理块宽 512）。
        const PROBE_WRITE_BYTES: usize = 512;
        let unit_device = singlefs_core::address::DeviceIdentity(1);
        let unit_area_start =
            slot_after_the_journal_ring(singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0
                * singlefs_format::SLOT_BYTES;
        let unit_write = |slot_in_the_unit_area: u64, filler_byte: u8| {
            let offset = singlefs_core::address::DeviceOffsetInBytes(
                unit_area_start + slot_in_the_unit_area * singlefs_format::SLOT_BYTES,
            );
            let bytes = match unit_write_contents {
                ProbeUnitWriteContents::FillerBytesOverReferencedUnits => {
                    vec![filler_byte; PROBE_WRITE_BYTES]
                }
                ProbeUnitWriteContents::TheBaseBytesWrittenAgain => base
                    .devices
                    .get(&unit_device)
                    .expect("底座是两块盘 mkfs 出的池，盘 1 在里面")
                    .read(offset, PROBE_WRITE_BYTES),
            };
            RetainedWrite {
                device: unit_device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset,
                contents: WrittenContents::Bytes(bytes),
            }
        };
        let mut root_record = vec![0u8; PROBE_WRITE_BYTES];
        root_record[24..28].copy_from_slice(&1u32.to_le_bytes());
        root_record[28..36].copy_from_slice(&1u64.to_le_bytes());
        vec![
            unit_write(0, 7),
            unit_write(1, 8),
            RetainedWrite {
                device: singlefs_core::address::DeviceIdentity(0),
                kind: StepKind::RootRecordFua,
                is_force_unit_access: true,
                offset: singlefs_core::root_ring::region_start(0),
                contents: WrittenContents::Bytes(root_record),
            },
        ]
    }

    /// 一条用例在给定的库上过两段（判路由调用方给，取 [`configured_judge_route`]），交回结局与块账：库是这个进程开的那一份（用例先把一次性库设成它），用例在
    /// [`CrashAmplificationEnvironment`] 里起。
    /// 流是两段单元写加一段根槽写（[`probe_writes_on_the_locations_of`]，单元写的内容照 `unit_write_contents`），被判的那一次写是那条根槽写
    /// （「根槽已持久」只按根槽写两态数，被判的写不是根槽写时判段交错）。
    #[cfg(feature = "gpu")]
    fn run_the_unit_writes_case(
        route: CrashPointReplayJudgeRoute,
        keys: KeysOfTheItem,
        unit_write_contents: ProbeUnitWriteContents,
    ) -> (StageOutcome<CrashPointReplayTally>, Option<CrashCaseBlocks>) {
        run_the_unit_writes_case_with_the_observers(route, keys, unit_write_contents, &[])
    }

    /// 单元写那条用例的项（项名与设点文件）：它自己那个点与末节点都归它（[`nodes_of_the_item`]）。
    #[cfg(feature = "gpu")]
    fn the_unit_writes_case_item() -> CrashPointReplayItem<'static> {
        CrashPointReplayItem {
            name: "crash_case_blocks_probe",
            setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
        }
    }

    /// 同 [`run_the_unit_writes_case`]，另挂 `observers_on_the_nodes`（它们那几把版本在 `keys` 里，同序）。
    #[cfg(feature = "gpu")]
    fn run_the_unit_writes_case_with_the_observers(
        route: CrashPointReplayJudgeRoute,
        keys: KeysOfTheItem,
        unit_write_contents: ProbeUnitWriteContents,
        observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    ) -> (StageOutcome<CrashPointReplayTally>, Option<CrashCaseBlocks>) {
        let base = formatted_pool_of_two_devices_for_the_probe();
        let writes = probe_writes_on_the_locations_of(&base, unit_write_contents);
        let segments: Vec<Vec<usize>> = (0..writes.len()).map(|index| vec![index]).collect();
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        let item = the_unit_writes_case_item();
        let flow_to_record = flow_of_the_inputs(
            item,
            &base,
            &writes,
            &segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        );
        enumerate_the_flow_through_the_library_with_the_keys(
            item,
            &flow_to_record,
            observers_on_the_nodes,
            route,
            keys,
        )
        .expect("两段过得了库")
    }

    #[cfg(feature = "gpu")]
    fn computed_keys(route: CrashPointReplayJudgeRoute) -> KeysOfTheItem {
        keys_of_the_item(route, &[])
    }

    /// 第二节第 9 条与 `CRASH_CASE_BLOCKS` 接口：头一趟现判、写进库，`blocks = reused + fresh + refused`，那一行五个字段次序照 admission.py。
    /// 判别力（`crates/mutations.tsv`）：那一行少打一个字段，这条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_crash_case_blocks_line_counts_the_blocks_of_a_fresh_run() {
        // 一次性库留着：钉头一趟现判的块账行，要从空库起跑
        let disposable = crate::crash_library::DisposableLibrary::new("crash-case-blocks")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
        environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
        let route = configured_judge_route();
        let (judged, fresh_run) = run_the_unit_writes_case(
            route,
            computed_keys(route),
            ProbeUnitWriteContents::FillerBytesOverReferencedUnits,
        );
        assert!(judged.judged().is_some(), "头一趟两段先后跑、交判的结果");
        let fresh_run = fresh_run.expect("判段跑了就有块账");
        assert!(fresh_run.blocks > 0);
        assert_eq!(
            (fresh_run.reused, fresh_run.fresh, fresh_run.refused),
            (0, fresh_run.blocks, 0),
            "空库头一趟全部现判"
        );
        assert!(fresh_run.states > 0, "判段那一趟数了块里的状态");
        assert_eq!(
            fresh_run.blocks,
            fresh_run.reused + fresh_run.fresh + fresh_run.refused
        );
        assert_eq!(
            fresh_run.line(),
            format!(
                "CRASH_CASE_BLOCKS blocks={} reused={} fresh={} red={} refused={} states={}",
                fresh_run.blocks,
                fresh_run.reused,
                fresh_run.fresh,
                fresh_run.red,
                fresh_run.refused,
                fresh_run.states
            )
        );
    }

    /// 第 10 条与 4-2a 报告第四节第 7 条后半句：判器那一组拒算的块照判、不入库、计入 `refused=`：判段照常交判的结果；同一份库
    /// （设成这个进程那一份的一次性库）里录入段照录（流与清单都在），CPU 判器的结论一块都没有（照判不入库的占位版本只留在这个进程里）。
    /// 判别力（`crates/mutations.tsv`）：照判不入库的行照旧写进库，这条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_case_whose_judging_units_are_refused_is_judged_stored_nowhere_and_counted_as_refused() {
        // 一次性库留着：钉拒算的块一块不入库，要数库里的结论
        let disposable = crate::crash_library::DisposableLibrary::new("refused-judging-units")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
        environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
        let judge_units = JudgingUnits::JudgementOfTheCrashImages;
        let (judged, blocks) = run_the_unit_writes_case(
            configured_judge_route(),
            KeysOfTheItem {
                observer_versions: Vec::new(),
                refusal_sentences: vec![
                    "判法单元 judgement_of_the_crash_images 拒算（单测造的）".to_string()
                ],
                judge_is_refused: true,
                versions_kept_out_of_the_library: vec![
                    *crate::crash_amplification::placeholder_judging_units_key_of_the_refused(
                        &judge_units,
                    )
                    .fingerprint(),
                ],
            },
            ProbeUnitWriteContents::FillerBytesOverReferencedUnits,
        );
        let tally = judged.judged().expect("拒算照判：判段交判的结果");
        assert!(tally.states > 0, "状态都判了");
        let blocks = blocks.expect("判段跑了就有块账");
        assert!(blocks.blocks > 0);
        assert_eq!(
            (blocks.reused, blocks.fresh, blocks.refused),
            (0, 0, blocks.blocks),
            "拒算的块全部计入 refused"
        );
        let store = disposable
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .expect("录入段照录进同一份库");
        assert_eq!(
            store.recorded_run_manifest_count().expect("读得了清单"),
            1,
            "录入段照录：这一项的清单在长期库里"
        );
        assert_eq!(
            store
                .stored_conclusion_keys()
                .expect("列得出库里的结论")
                .into_iter()
                .filter(|(_key, kind, _version)| *kind == ConclusionKind::ImageJudgement)
                .count(),
            0,
            "CPU 判器拒算：它的结论一块都没进长期库"
        );
    }

    /// 红批第 5 条与 crates 第 3 条的另一半：拒算的只是别的组（这里造一个观察者那一组拒算，判器那一组取得出），判器的结论照键入库
    /// 同一份长期库、一块不少；这条用例有一组拒算，块照旧全部计入 `refused=`。与上一条合起来钉「拒算的组一块不入、别的组照入」。
    /// 判别力（`crates/mutations.tsv`）：判段把「有一组拒算」当成「判器那一组拒算」，判器的结论也照判不入库，这条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_case_whose_other_judging_units_are_refused_stores_every_judge_block_in_the_library() {
        // 一次性库留着：钉别的组拒算时判器结论照入库，要数库里的结论
        let disposable =
            crate::crash_library::DisposableLibrary::new("refused-observer-judging-units")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
        environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
        let refused_observer_units = JudgingUnits::ObserverCode {
            observer_code_file: "crates/singlefs-verification/src/crash_state_enumeration.rs"
                .to_string(),
            observer_name: "refused_in_the_unit_test".to_string(),
        };
        let (judged, blocks) = run_the_unit_writes_case(
            configured_judge_route(),
            KeysOfTheItem {
                observer_versions: Vec::new(),
                refusal_sentences: vec![
                    "判法单元 observer_code:crates/singlefs-verification/src/crash_state_enumeration.rs:refused_in_the_unit_test 拒算（单测造的）"
                        .to_string(),
                ],
                judge_is_refused: false,
                versions_kept_out_of_the_library: vec![
                    *crate::crash_amplification::placeholder_judging_units_key_of_the_refused(
                        &refused_observer_units,
                    )
                    .fingerprint(),
                ],
            },
            ProbeUnitWriteContents::FillerBytesOverReferencedUnits,
        );
        assert!(
            judged.judged().expect("判段交判的结果").states > 0,
            "状态都判了"
        );
        let blocks = blocks.expect("判段跑了就有块账");
        assert!(blocks.blocks > 0);
        assert_eq!(
            (blocks.reused, blocks.fresh, blocks.refused),
            (0, 0, blocks.blocks),
            "有一组拒算，这条用例的块全部计入 refused"
        );
        let store = disposable
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .expect("录入段照录进同一份库");
        assert_eq!(
            u64::try_from(
                store
                    .stored_conclusion_keys()
                    .expect("列得出库里的结论")
                    .into_iter()
                    .filter(|(_key, kind, _version)| *kind == ConclusionKind::ImageJudgement)
                    .count()
            )
            .expect("块数装得进 u64"),
            blocks.blocks,
            "判器那一组没拒算：它的结论每一块都照键进了长期库"
        );
    }

    /// 场景：同一份库（测试拿着一份开着的库，几趟共用这个进程里同一份 `VerdictStore`）上那条单元写的用例照键复用跑两趟；第二趟设复用哨兵的种子、
    /// 抽 1 块当哨兵。两次单元写照底座原有的字节重写（[`ProbeUnitWriteContents::TheBaseBytesWrittenAgain`]）：哨兵与强制现判只从不红的块里抽，
    /// 根槽写落下之前的几块要判不红；写填充字节的那一种盖掉被引用的单元，只有什么都没落下的那一块不红，凑不出一块哨兵加一块强制现判。
    /// 预期：第二趟照常交判的结果、块账除记着红的块之外全部复用（强制现判那一块按复用算）；这一趟交来存过的块是记着红的那几块加恰好一块
    /// （抽中强制现判的那一块现判、判完照键写回，哨兵与别的不红的块都复用）：崩溃枚举用例那一路接上块级复用哨兵与强制现判（与崩溃放量那一路同一套，`.claude/rules/test-script-discipline.md`
    /// 「判定可复用：按输入存，按前缀剪」每一趟真跑插哨兵）。
    /// 判别力（`crates/mutations.tsv`）：「崩溃枚举用例：判段不抽复用哨兵」这一行让这一条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_case_judged_again_on_the_same_library_draws_reuse_sentinels_and_judges_only_the_forced_block_again(
    ) {
        // 一次性库留着：钉头一趟现判、第二趟抽哨兵与强制现判，要从空库起跑
        let disposable = crate::crash_library::DisposableLibrary::new("crash-case-reuse-sentinels")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
        environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
        let held_library = CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration)
            .expect("开得了设成这个进程那一份的一次性库");
        let route = configured_judge_route();
        let (judged, fresh_run) = run_the_unit_writes_case(
            route,
            computed_keys(route),
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
        );
        assert!(judged.judged().is_some(), "头一趟交判的结果");
        let fresh_run = fresh_run.expect("判段跑了就有块账");
        assert!(
            fresh_run.blocks - fresh_run.red >= 2,
            "前提：至少两块不红（红块不当哨兵的候选；一块当哨兵、一块强制现判）"
        );
        environment.set_variable(
            crate::crash_amplification::REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("7"),
        );
        environment.set_variable(
            crate::crash_amplification::REUSE_SENTINEL_COUNT_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("1"),
        );
        let offered_before_the_rerun = held_library.lock().conclusion_blocks_offered_so_far();
        let (rerun, rerun_blocks) = run_the_unit_writes_case(
            route,
            computed_keys(route),
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
        );
        assert!(
            rerun.judged().is_some(),
            "第二趟照常交判的结果（哨兵一块都没被现判、强制现判那一块现判出同一个结论）"
        );
        let rerun_blocks = rerun_blocks.expect("判段跑了就有块账");
        assert_eq!(
            (rerun_blocks.reused, rerun_blocks.fresh),
            (rerun_blocks.blocks - fresh_run.red, fresh_run.red),
            "强制现判那一块按复用算：除了记着红的块（下一趟现判），块账全部复用"
        );
        assert_eq!(
            u64::try_from(
                held_library
                    .lock()
                    .conclusion_blocks_offered_since(offered_before_the_rerun)
                    .len()
            )
            .expect("块数装得进 u64"),
            fresh_run.red + 1,
            "第二趟交来存过的块：记着红的那几块，加抽中强制现判的那一块"
        );
        drop(held_library);
    }

    /// 单元写那条用例挂的观察者：只记计数、每个状态都不 panic（[`observe_no_crash_state`]），挂在整条流上（它自己那个点与末节点）。
    #[cfg(feature = "gpu")]
    fn the_unit_writes_case_observer() -> ObserverOnTheNodes<'static> {
        ObserverOnTheNodes {
            observer_name: ObserverName::new("crash_case_reuse_sentinel_probe_observer")
                .expect("名字合规矩"),
            observer_code_file: the_unit_writes_case_item().setting_file.to_string(),
            extra_inputs_digest: [5; DIGEST_BYTES],
            nodes: nodes_of_the_item(the_unit_writes_case_item()),
            observe: ObserverCode::CountsOnly(&observe_no_crash_state),
        }
    }

    /// [`the_unit_writes_case_observer`] 那几把版本：没登记入口的观察者取键拒算（照判、不入库），单测照同形另造摘要；判观察者交回的盘的
    /// 那一路照这一趟的判路（判段断言两边同一路）。
    #[cfg(feature = "gpu")]
    fn versions_of_the_unit_writes_case_observer(
        route: CrashPointReplayJudgeRoute,
    ) -> ObserverVersions {
        let observer = ConclusionVersion(sha256_digest(
            b"observer version of the crash case reuse sentinel probe\n",
        ));
        let pool_checker_version = ConclusionVersion(sha256_digest(
            b"pool checker version of the crash case reuse sentinel probe\n",
        ));
        ObserverVersions {
            observer,
            observer_code: ConclusionVersion(sha256_digest(
                b"observer code version of the crash case reuse sentinel probe\n",
            )),
            pool_checker: route.judged_by(),
            pool_checker_version,
            conclusion: ConclusionVersion(sha256_digest(
                &[observer.0, pool_checker_version.0].concat(),
            )),
        }
    }

    /// 场景：同一份库上那条单元写的用例挂一个只记计数的观察者（整条流），照键复用跑两趟；第二趟设复用哨兵的种子、抽 1 块当哨兵（同上一条）。
    /// 预期：第二趟照常交判的结果——判哨兵在观察者判段之后（两处判段共用的
    /// `crate::crash_amplification::judge_the_observers_then_the_reuse_sentinels`），强制现判那一块上的观察者结论这一趟现拼、交来存过，
    /// 哨兵一处红都不报；第二趟交来存过的观察者结论恰好一份（强制现判那一块上的那一份，别的块上的都复用）。
    /// 判别力（`crates/mutations.tsv`）：「复用哨兵：判哨兵挪到观察者判段之前」这一行让这一条红（判哨兵时强制现判那一块上的观察者结论还没现拼，
    /// 强制现判表又被它清掉，报「仍复用」，第二趟交不回判的结果）。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_case_with_an_observer_judged_again_judges_the_reuse_sentinels_after_the_observers() {
        // 一次性库留着：钉头一趟现判、第二趟抽哨兵与强制现判，要从空库起跑
        let disposable =
            crate::crash_library::DisposableLibrary::new("crash-case-reuse-sentinels-observer")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        // 照调用方的多机配置判（副本里只经 SINGLEFS_MULTI_HOST_CONFIG 交进来），走 GPU 那一路要卡：照服务交的卡取（守卫清掉了这两个变量）
        environment.set_the_multi_host_configuration_and_the_cards_of_the_caller();
        let held_library = CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration)
            .expect("开得了设成这个进程那一份的一次性库");
        let route = configured_judge_route();
        let observer = the_unit_writes_case_observer();
        let keys_with_the_observer = || {
            let mut keys = computed_keys(route);
            keys.observer_versions
                .push(versions_of_the_unit_writes_case_observer(route));
            keys
        };
        let (judged, fresh_run) = run_the_unit_writes_case_with_the_observers(
            route,
            keys_with_the_observer(),
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
            std::slice::from_ref(&observer),
        );
        assert!(judged.judged().is_some(), "头一趟交判的结果");
        let fresh_run = fresh_run.expect("判段跑了就有块账");
        assert!(
            fresh_run.blocks - fresh_run.red >= 2,
            "前提：至少两块不红（一块当哨兵、一块强制现判）"
        );
        environment.set_variable(
            crate::crash_amplification::REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("7"),
        );
        environment.set_variable(
            crate::crash_amplification::REUSE_SENTINEL_COUNT_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("1"),
        );
        let offered_before_the_rerun = held_library.lock().conclusion_blocks_offered_so_far();
        let (rerun, _rerun_blocks) = run_the_unit_writes_case_with_the_observers(
            route,
            keys_with_the_observer(),
            ProbeUnitWriteContents::TheBaseBytesWrittenAgain,
            std::slice::from_ref(&observer),
        );
        assert!(
            rerun.judged().is_some(),
            "第二趟照常交判的结果（哨兵一处红都没报）"
        );
        assert_eq!(
            held_library
                .lock()
                .conclusion_blocks_offered_since(offered_before_the_rerun)
                .iter()
                .filter(|(_block_key, kind, _version)| *kind == ConclusionKind::Observer)
                .count(),
            1,
            "第二趟交来存过的观察者结论恰好一份：强制现判那一块上的那一份"
        );
        drop(held_library);
    }

    #[test]
    fn a_crash_point_replay_item_name_cannot_be_a_path_segment_with_a_slash() {
        let item = CrashPointReplayItem {
            name: "fine_name",
            setting_file: "crates/singlefs-verification/tests/x.rs",
        };
        assert_eq!(item_name_of(item).as_str(), "fine_name");
        let caught = std::panic::catch_unwind(|| {
            item_name_of(CrashPointReplayItem {
                name: "a/b",
                setting_file: "crates/singlefs-verification/tests/x.rs",
            })
        });
        assert!(caught.is_err(), "带 / 的项名要停下");
    }

    /// 观察者挂法单测的小流：两次单元写一段、一次单元写一段、一次根槽写一段，写区间首尾相接，一段一步、一步一个点，加末节点共四个点。
    fn writes_and_segments_of_three_steps() -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
        writes_and_segments_of(&[
            &[(StepKind::UnitWrite, 1, 0), (StepKind::UnitWrite, 2, 0)],
            &[(StepKind::UnitWrite, 3, 0)],
            &[(StepKind::RootRecordFua, 0, 1)],
        ])
    }

    /// 挂法单测里的观察者：什么都不看，另读的输入是一份写死的非零摘要（看的是挂法怎么改它）。
    const EXTRA_INPUTS_OF_THE_ATTACHED_PROBE: [u8; DIGEST_BYTES] = [7; DIGEST_BYTES];

    fn observe_no_crash_state(
        _ordinal: u64,
        _image: &CrashImage<'_>,
        _report: &singlefs_core::recovery::RecoveryReport,
        _counts: &mut crate::crash::CrashPointReplayObserverCounts,
    ) {
    }

    /// 挂法 `points`、代码文件取用例文件的那个观察者，在 `writes` / `segments` 那条流上挂成的样子（只挂调用方这一个，不接按来历挂的）。
    fn attached_probe_on_the_nodes<'points>(
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
        points: CrashPointReplayObserverPoints<'points>,
    ) -> Result<ObserverOnTheNodes<'points>, String> {
        let observe: &'static ObserveACrashState<'static> = &observe_no_crash_state;
        observers_on_the_nodes_of_the_item(
            BLOCK_ACCOUNT_PROBE_ITEM,
            writes,
            segments,
            &[AttachedCrashPointReplayObserver {
                observer: CrashPointReplayObserver {
                    name: "attached_probe",
                    extra_inputs_digest: EXTRA_INPUTS_OF_THE_ATTACHED_PROBE,
                    observe,
                },
                attachment: CrashPointReplayObserverAttachment {
                    points,
                    code_file: CrashPointReplayObserverCodeFile::TheCaseFile,
                },
            }],
            &[],
        )
        .map(|mut observers| observers.remove(0))
    }

    /// 场景：三步的小流，一个观察者只挂记录前那一次单元写与根槽写两段（段下标 1、2），另一个挂整条流。
    /// 预期：只挂两段的那一个只挂那两步的点，不挂头一步（前缀）的点，也不挂末节点；挂整条流的每个点与末节点都挂。
    /// 判别力（`crates/mutations.tsv`）：点名段的挂法照旧挂整条流、罩着点名段的点不筛，各让这一条红。
    #[test]
    fn an_observer_attached_to_named_segments_holds_only_the_points_over_those_segments_and_not_the_end_of_the_flow(
    ) {
        let (writes, segments) = writes_and_segments_of_three_steps();
        let base = MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        let plan = flow_of_the_inputs(
            BLOCK_ACCOUNT_PROBE_ITEM,
            &base,
            &writes,
            &segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        )
        .with_the_flow(plan_crash_points);
        assert_eq!(plan.points.len(), 4, "前提：三步各一个点，加末节点");
        let on_two_segments = attached_probe_on_the_nodes(
            &writes,
            &segments,
            CrashPointReplayObserverPoints::PointsOverTheSegments(&[1, 2]),
        )
        .expect("点名的两段在流里");
        let on_the_whole_flow = attached_probe_on_the_nodes(
            &writes,
            &segments,
            CrashPointReplayObserverPoints::EveryPointOfTheFlow,
        )
        .expect("整条流挂得上");
        let account = observers_in_the_block_account(
            &plan,
            &[on_two_segments, on_the_whole_flow],
            &[
                ConclusionVersion([1; DIGEST_BYTES]),
                ConclusionVersion([2; DIGEST_BYTES]),
            ],
        );
        assert_eq!(
            account[0].holds_each_point,
            vec![false, true, true, false],
            "只挂点名的两段：那两步的点挂，前缀那一步的点与末节点都不挂（不录、不判、块账不要它的结论）"
        );
        assert_eq!(
            account[1].holds_each_point,
            vec![true; 4],
            "整条流：每个点与末节点都挂"
        );
    }

    /// 场景：挂法点名一份共用的代码文件，与不点名（用例文件）各取一次观察者代码文件。
    /// 预期：点名的取点名的那一份（几条用例挂同一个观察者时找登记表里同一行），不点名的取用例文件。
    /// 判别力（`crates/mutations.tsv`）：点名共用文件也取用例文件，这一条红。
    #[test]
    fn an_observer_attached_with_a_shared_code_file_takes_that_file_and_otherwise_the_case_file() {
        let shared_file = "crates/singlefs-verification/tests/common_crash_point_observers/mod.rs";
        assert_eq!(
            observer_code_file_of_the_attachment(
                BLOCK_ACCOUNT_PROBE_ITEM,
                CrashPointReplayObserverAttachment {
                    points: CrashPointReplayObserverPoints::EveryPointOfTheFlow,
                    code_file: CrashPointReplayObserverCodeFile::SharedFile(shared_file),
                },
            ),
            shared_file,
            "点名共用文件：观察者代码文件就是它"
        );
        assert_eq!(
            observer_code_file_of_the_attachment(
                BLOCK_ACCOUNT_PROBE_ITEM,
                CrashPointReplayObserverAttachment::EVERY_POINT_WITH_THE_CASE_FILE,
            ),
            BLOCK_ACCOUNT_PROBE_ITEM.setting_file,
            "不点名：观察者代码文件是用例文件"
        );
    }

    /// 场景：三块的小流（单元写那一步、根槽写那一步、末节点），一个观察者挂整条流、一个只挂根槽写那一段；判器结论（带恢复字）与逐盘自证槽数
    /// 每一块都存，挂整条流的那个观察者结论每一块都存，只挂一段的那个只在它挂的那一块上存。
    /// 预期：块账三块全罩住（只挂一段的观察者在它不挂的两块上不要结论，前缀那一块不因它缺而现判）；判段收工问缺不缺：一块都不缺；
    /// 从库重建计数照常读回 9 个状态，看过的状态数 9（至少一个观察者挂着的点上的状态）。
    /// 判别力（`crates/mutations.tsv`）：挂没挂这个点不筛（收工问缺、重建计数都问它在不挂的点上的结论）、看过的状态数只数每个观察者都挂着的点，各让这一条红。
    #[test]
    fn the_block_account_the_finishing_question_and_the_tally_leave_out_the_points_an_attached_observer_does_not_hold(
    ) {
        let _environment =
            crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
        with_the_unit_writes_flow_and_an_empty_library(
            "observer-attached-to-named-segments",
            |flow, plan, store| {
                assert_eq!(
                    plan.points.len(),
                    3,
                    "前提：单元写那一步、根槽写那一步、末节点"
                );
                let observers_on_the_nodes = [
                    attached_probe_on_the_nodes(
                        flow.writes,
                        flow.segments,
                        CrashPointReplayObserverPoints::EveryPointOfTheFlow,
                    )
                    .expect("整条流挂得上"),
                    attached_probe_on_the_nodes(
                        flow.writes,
                        flow.segments,
                        CrashPointReplayObserverPoints::PointsOverTheSegments(&[1]),
                    )
                    .expect("根槽写那一段在流里"),
                ];
                let whole_flow_version = ConclusionVersion([2; DIGEST_BYTES]);
                let one_segment_version = ConclusionVersion([3; DIGEST_BYTES]);
                let observers = observers_in_the_block_account(
                    plan,
                    &observers_on_the_nodes,
                    &[whole_flow_version, one_segment_version],
                );
                assert_eq!(
                    observers[1].holds_each_point,
                    vec![false, true, false],
                    "前提：只挂一段的那个只挂根槽写那一步"
                );
                let judgement_version = ConclusionVersion([1; DIGEST_BYTES]);
                let judgement_version_of_each_point = vec![judgement_version; plan.points.len()];
                let judged_conclusions =
                    crate::crash_amplification::ConclusionsStoredUnder::TheConclusionKind {
                        kind: ConclusionKind::ImageJudgement,
                        version_of_each_point: &judgement_version_of_each_point,
                    };
                for (block_position, (_point_name, local, key)) in
                    blocks_of_the_plan(plan).iter().enumerate()
                {
                    let states = local.end - local.start;
                    store_a_conclusion_block_of_the_unit_test(
                        store,
                        key,
                        &ConclusionKind::ImageJudgement,
                        &judgement_version,
                        states,
                        &[],
                        Some(crate::verdict_store::RecoveryWordsOfState(
                            [0; crate::crash_judge_conclusions::RECOVERY_WORDS_PER_STATE],
                        )),
                    );
                    store
                        .store_observer_block(
                            key,
                            &ConclusionKind::ImageJudgement,
                            &judgement_version,
                            b"self verified slots of the unit test",
                        )
                        .expect("存得下");
                    store_a_conclusion_block_of_the_unit_test(
                        store,
                        key,
                        &ConclusionKind::Observer,
                        &whole_flow_version,
                        states,
                        &[],
                        None,
                    );
                    if observers[1].holds_each_point[block_position] {
                        store_a_conclusion_block_of_the_unit_test(
                            store,
                            key,
                            &ConclusionKind::Observer,
                            &one_segment_version,
                            states,
                            &[],
                            None,
                        );
                    }
                }
                let every_block = |_block: u64| true;
                let account = blocks_in_the_library(
                    store,
                    plan,
                    &judged_conclusions,
                    &observers,
                    &every_block,
                )
                .expect("读得了库");
                assert_eq!(
                    (account.blocks, account.covered, account.red),
                    (3, 3, 0),
                    "块账：三块全罩住，前缀那一块不因只挂一段的观察者缺结论而现判"
                );
                let without_every_conclusion = blocks_of_the_flow_without_every_conclusion(
                    store,
                    plan,
                    &judged_conclusions,
                    &observers,
                )
                .expect("读得了库");
                assert_eq!(
                    (
                        without_every_conclusion.missing_an_observer_conclusion,
                        without_every_conclusion.missing_the_judgement_or_an_observer_conclusion
                    ),
                    (0, 0),
                    "判段收工问缺不缺：只问挂着的观察者，一块都不缺"
                );
                let (tally, observer_red) = crash_point_replay_tally_from_the_library(
                    store,
                    flow,
                    plan,
                    ConclusionKind::ImageJudgement,
                    &judgement_version_of_each_point,
                    &observers,
                )
                .expect("挂着的观察者的结论都在");
                assert_eq!(
                    (tally.states, tally.observed_states),
                    (9, 9),
                    "重建计数读回整条流 9 个状态；每个状态都有观察者挂着，看过的状态数 9"
                );
                assert_eq!(
                    observer_red.red_states_of_each_observer,
                    vec![0, 0],
                    "两个观察者都没有红"
                );
            },
        );
    }

    /// 场景：同一个观察者在三步的小流上换几种挂法：整条流；只挂根槽写那一段；挂记录前那一次单元写与根槽写两段；同两段换个写法（次序倒过来、重复一个）。
    /// 预期：整条流的另读的输入照原样（挂整条流的观察者结论键不动）；点名段的把段集合并进另读的输入；换了段集合，观察者结论版本跟着换，
    /// 只看代码的那一把与判它交回的盘的池级 checker 那一把不动；同一组段换个写法版本不变；判器块键从流算，挂法不进它。
    /// 判别力（`crates/mutations.tsv`）：点名段时段集合不并进另读的输入、整条流也并一份空段集合、段集合不去重排序，各让这一条红。
    #[test]
    fn changing_the_named_segments_of_an_observer_moves_only_its_conclusion_version_and_not_the_block_keys(
    ) {
        let (writes, segments) = writes_and_segments_of_three_steps();
        let on = |points| {
            attached_probe_on_the_nodes(&writes, &segments, points).expect("点名的段在流里")
        };
        let on_the_whole_flow = on(CrashPointReplayObserverPoints::EveryPointOfTheFlow);
        let on_the_root = on(CrashPointReplayObserverPoints::PointsOverTheSegments(&[2]));
        let on_two_segments = on(CrashPointReplayObserverPoints::PointsOverTheSegments(&[
            1, 2,
        ]));
        let on_two_segments_written_otherwise =
            on(CrashPointReplayObserverPoints::PointsOverTheSegments(&[
                2, 1, 2,
            ]));
        assert_eq!(
            on_the_whole_flow.extra_inputs_digest, EXTRA_INPUTS_OF_THE_ATTACHED_PROBE,
            "整条流：另读的输入照原样"
        );
        assert_ne!(
            on_the_root.extra_inputs_digest, EXTRA_INPUTS_OF_THE_ATTACHED_PROBE,
            "点名段：段集合并进另读的输入"
        );
        let toolchain = toolchain_of_this_process();
        let versions_of = |observer: &ObserverOnTheNodes<'_>| {
            observer_versions_keeping_the_refused_out(observer, JudgedBy::CpuJudge, toolchain).0
        };
        let (root_versions, two_segment_versions) =
            (versions_of(&on_the_root), versions_of(&on_two_segments));
        assert_ne!(
            root_versions.conclusion, two_segment_versions.conclusion,
            "换了段集合：观察者结论版本跟着换"
        );
        assert_eq!(
            (
                root_versions.observer_code,
                root_versions.pool_checker_version
            ),
            (
                two_segment_versions.observer_code,
                two_segment_versions.pool_checker_version
            ),
            "只看代码的那一把与池级 checker 那一把不随段集合变"
        );
        assert_eq!(
            versions_of(&on_two_segments_written_otherwise).conclusion,
            two_segment_versions.conclusion,
            "同一组段换个写法（次序、重复）版本不变"
        );
        let base = MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let block_keys =
            block_keys_of_each_node(BLOCK_ACCOUNT_PROBE_ITEM, &base, &writes, &segments);
        assert_eq!(
            block_keys.len(),
            4,
            "判器块键从流排的计划取（三步加末节点），观察者挂法不是计划的输入，换挂法块键不变"
        );
    }

    /// 场景：观察者挂法点名一个空的段集合。
    /// 预期：录任何东西之前停下，说明点名「段集合是空的」。
    /// 判别力（`crates/mutations.tsv`）：空段集合不拦，这一条红（落到「一个点都罩不着」那一句）。
    #[test]
    fn an_observer_attached_to_an_empty_set_of_segments_is_refused() {
        let (writes, segments) = writes_and_segments_of_three_steps();
        let Err(refusal) = attached_probe_on_the_nodes(
            &writes,
            &segments,
            CrashPointReplayObserverPoints::PointsOverTheSegments(&[]),
        ) else {
            panic!("空段集合要停下");
        };
        assert!(
            refusal.contains("段集合是空的"),
            "停下的说明点名空段集合：{refusal}"
        );
    }

    /// 场景：观察者挂法点名的段下标等于流的段数（越过最后一段）。
    /// 预期：录任何东西之前停下，说明点名那一段不在流里，不越界 panic。
    /// 判别力（`crates/mutations.tsv`）：越界只拦大于段数的，这一条红（下标等于段数时越界 panic）。
    #[test]
    fn an_observer_attached_to_a_segment_past_the_end_of_the_flow_is_refused() {
        let (writes, segments) = writes_and_segments_of_three_steps();
        let Err(refusal) = attached_probe_on_the_nodes(
            &writes,
            &segments,
            CrashPointReplayObserverPoints::PointsOverTheSegments(&[segments.len()]),
        ) else {
            panic!("越过最后一段要停下");
        };
        assert!(
            refusal.contains("不在这条流里"),
            "停下的说明点名那一段不在流里：{refusal}"
        );
    }
}
