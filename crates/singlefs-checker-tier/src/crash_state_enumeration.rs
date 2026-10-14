//! 崩溃点重放枚举用例的两段入口（崩溃判定库的 [`CrashLibraryClass::CrashStateEnumeration`] 类）：录入段在 CPU 上把流写进库，判段只从库里读回、
//! 按块判（配了 GPU 上卡判，没配由 CPU 判）、结论按（块键，结论种类，版本）存进库，判过的块下一趟复用（剪枝），崩溃点重放的计数
//! （[`CrashPointReplayTally`]）从库里读回的结论重建。判一个状态的判法在 `crate::crash`（CPU）与 `crate::crash_judge_gpu`（GPU），这里只排两段、
//! 领块、存库、读回。
//!
//! 一条崩溃枚举用例的流当一个崩溃点（名字是项名、设点文件是用例文件），加末节点（整张写表全部持久那一个状态，
//! `crash_amplification::END_OF_FLOW_NODE_NAME`）；块键照崩溃放量的身份与复用键（`crate::crash_identity`），结论版本照三种结论各一把
//! （`crate::crash_amplification::ConclusionVersionsOfThePlan`）：改了判器只换判器那把、改了用例文件只换它那把 CPU 版本。
//! 两台分片与崩溃放量同一种读法（`SINGLEFS_CRASH_AMPLIFICATION_SHARE`、`…_HOST_WEIGHTS`、`…_IMPORT`）。
//!
//! 行：开跑一行 `CRASH_POINT_REPLAY_PARALLEL_START`、跑完一行 `CRASH_POINT_REPLAY_PARALLEL_FINISHED`（门禁 checker-tier-crash-point-replay 的准入模块按它判线程；
//! `slices=` 是块数、`resumed_slices=` 是库里复用的块、`freshly_run_slices=` 是这一趟判的块），每个新签名一行 `CRASH_POINT_REPLAY_FINDING`、
//! 收尾一行 `CRASH_POINT_REPLAY_FINDINGS`；设了 `SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE` 时发现日志写一节定稿（`crate::crash_point_replay_progress`）。

use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use singlefs_checker::image::IMPLEMENTED_INVARIANTS;
use singlefs_harness::gpu_memory_ledger::StopRequestRememberedInThisRun;
use singlefs_harness::heavy_threads::{
    default_heavy_threads_of_the_cap_value, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::{
    root_identity_of_write, CrashImage, MemoryPool, PublishedVersion, RecordCheck, RetainedWrite,
};
use singlefs_harness::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash::{
    crash_state_ranges_by_segment, publish_of_each_segment, quick_tier_expansion,
    CrashPointReplayFindingSignature, CrashPointReplayOracleViolationKind,
    CrashPointReplayPublishOfState, CrashPointReplayRedPass, CrashPointReplaySegmentExpansion,
    CrashPointReplaySegmentOfState, CrashPointReplayTally, CrashPointReplayVersionTable,
    CrashStateJudgement, CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE, CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER,
    CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER, RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT,
    RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};
use crate::crash_amplification::{
    check_blocks_where_until_with_threads, count_vector_disagreements_of_the_flow,
    import_the_planned_blocks_of_the_other_hosts, judge_the_observers_of_the_flow,
    observer_versions_to_write_under, pending_blocks, plan_crash_points,
    record_the_observers_of_the_flow, states_with_the_root_slot_write_persisted,
    stop_is_requested_by_the_environment, unjudged_blocks_of_the_flow, write_the_recorded_blocks,
    ConclusionVersionsOfThePlan, CpuJudgementsOfSampledBlocks, CrashAmplificationRunMode,
    CrashFlow, CrashPlan, CrashPointSpan, FlowOfTheLibrary, JudgedBy, NodeOfAnObserver,
    ObserveACrashState, ObserveACrashStateHandingBackAnImage, ObserverCode, ObserverDeduplication,
    ObserverOnTheNodes, ObserverVersions, PoolCheckerOfTheImagesAfterTheObservers, RecordedFlow,
    RecordedFlowReadBack, WeightedShares, END_OF_FLOW_NODE_NAME,
};
use crate::crash_identity::{ConclusionKind, ConclusionVersion, ObserverName};
use crate::crash_judge_conclusions::{
    red_items_of_the_encoded_verdict, RECOVERY_OUTCOME_CODE_FAILED,
    RECOVERY_OUTCOME_CODE_FILE_READ, RECOVERY_OUTCOME_CODE_NO_FILE,
};
use crate::crash_library::{
    repository_root_of_this_crate, two_stages, CrashLibrary, CrashLibraryClass, ItemManifest,
    ItemName, StageOutcome,
};
use crate::crash_point_replay_progress::{
    findings_log_final_lines, findings_standard_output_new_signature_line,
    findings_standard_output_summary_line, CrashPointReplayFindingsLog,
    CrashPointReplayFindingsLogBegin, CrashPointReplayFindingsLogSection,
    CrashPointReplayToolchainIdentity,
};
use crate::gpu_card_configuration::multi_host_configuration_file;
use crate::verdict_store::{StoredConclusionBlock, VerdictStore};

/// 崩溃点重放判段的工作线程数从这个环境变量取（十进制正整数）；没设取崩溃放量的 `SINGLEFS_CRASH_AMPLIFICATION_THREADS`，再没设取
/// 每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）。门禁 checker-tier-crash-point-replay 显式传进来。
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

/// 逐状态看一遍、只记计数的观察者（崩溃点重放合并第三轮判决 J4 那一种）：名字、另读的输入的摘要（进它的版本；没有另读的给全零）、怎么看一个状态
/// （拿到崩溃镜像、看 journal 那一遍恢复的报告、这个状态的计数；要记的数全经计数记，放在闭包自己变量里的续跑与复用之后就少了）。
/// 两段（用户 2026-10-13 定）：录入段在 CPU 上逐状态看、结果写进库，判段只从库里读、拼成结论按（块键，观察者种类，版本）存进库，
/// 看过的块下一趟复用；观察者 panic 的状态记成红项，判段收尾时整条用例停下。它可以读持久集合的结构，不去重。
pub struct CrashPointReplayObserver<'observer> {
    pub name: &'observer str,
    pub extra_inputs_digest: [u8; DIGEST_BYTES],
    pub observe: &'observer ObserveACrashState<'observer>,
}

/// 交回镜像的观察者（可写挂载、续做）：名字、另读的输入的摘要（进它的版本与去重键；有另读的就要真算）、怎么看一个状态（交回它在崩溃镜像上发的写与
/// 停下时的盘，`crash_amplification::ImageAfterTheObserver`）。录入段在 CPU 上看、按恢复镜像的字节去重（同一张镜像只看一次），判段从库里读回、
/// 在观察者交回的盘上跑池级 checker（配了 GPU 在卡上判，没配由 CPU 判）。
pub struct CrashPointReplayObserverHandingBackAnImage<'observer> {
    pub name: &'observer str,
    pub extra_inputs_digest: [u8; DIGEST_BYTES],
    pub observe: &'observer ObserveACrashStateHandingBackAnImage<'observer>,
    pub deduplication: ObserverDeduplication,
}

/// 判段在哪判（有卡上卡、没配 GPU 由 CPU 判；配了 GPU 而这份二进制没编进 `gpu` 特性，判段不跑）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrashPointReplayJudgeRoute {
    Cpu {
        why_not_the_gpu: &'static str,
    },
    Gpu,
    /// 配置开了 GPU、这份二进制没编进 `gpu` 特性（提交时快档不带特性那一趟）：录入段照录、写进库，判段不判、打本次未跑那一行
    /// （[`crash_point_replay_judge_not_run_line`]）、交只录（`StageOutcome::RecordedOnly`），带特性那一趟从库里读回在卡上判
    /// （用户 2026-10-13 定「配 GPU 必用 GPU、做不到报错不降级」；与 `crate::crash_amplification::run_from_environment` 同一种写法）。
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

/// 照本机多机配置选路（[`crash_point_replay_judge_route_of_the_switches`]）。
fn crash_point_replay_judge_route(
    repository_root: &Path,
) -> Result<CrashPointReplayJudgeRoute, String> {
    let configuration_file: Option<PathBuf> = multi_host_configuration_file(repository_root)?;
    let mode = match &configuration_file {
        Some(configuration) => CrashAmplificationRunMode::from_configuration_file(configuration),
        None => CrashAmplificationRunMode::from_configuration_file_or_environment(
            &repository_root.join("multi-host.env"),
        ),
    };
    Ok(crash_point_replay_judge_route_of_the_switches(
        mode.uses_gpu(),
        cfg!(feature = "gpu"),
    ))
}

/// 配置开了 GPU（`ENABLE_GPU=1`）而这份二进制编进了 `gpu` 特性就上卡；配了而没编进来，判段不跑、报本次未跑（不退回 CPU 判，用户 2026-10-13 定
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

/// 判段不跑那一趟打的本次未跑那一行：门禁快档认的 GPU 用例那一族记号（`NOT_RUN_<原因> test=<用例> reason=<一句>`，
/// `singlefs_harness::gpu_memory_ledger::gpu_not_run_line`），用例是这条崩溃枚举用例的项名；原因同崩溃放量那一处
/// （`crate::crash_amplification::judge_not_run_without_the_gpu_feature`）。不打 `CRASH_POINT_REPLAY_JUDGE route=cpu`、不打带 `judge=` 的 PARALLEL 行。
#[must_use]
pub fn crash_point_replay_judge_not_run_line(item_name: &str) -> String {
    singlefs_harness::gpu_memory_ledger::gpu_not_run_line(
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
/// 的登记值，判法在 `singlefs_harness::heavy_threads`）。设了而不是正整数（0 也不是）就 panic、消息带变量名与原值（经共用解析
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

/// 平时快档（甲二，[`CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll`]）：每一段都按甲二展开、多版本。
pub fn enumerate_crash_states_quick_tier_versions(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_with_expansion(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
        &quick_tier_expansion,
        &[],
    )
}

/// 崩溃点重放枚举的总入口：`expansion` 定每一段怎么展开，`versions` 是 oracle 读的版本表（带「没有文件的那几版」的删除流也经它），
/// `observers` 逐状态看（没有就空）。录入段把流（连同每段的展开）写进库、登清单；判段读回、按块判、存结论、重建计数。
///
/// # Panics
/// 开不了库、写不进、读不回；选路要卡而卡用不上（配了 GPU 不退回 CPU）；结论版本与编译时对不上（整趟不写库）；判段判完还有块没结论
/// （不是两台分片时）；计数向量与计划对不上；某个观察者在某个状态上 panic（它那一句原样带出）；项名撞了。
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
/// （用户 2026-10-13 定「能走两段的都走两段」：取样的结论也入库，打一行 `CRASH_POINT_REPLAY_SAMPLED_CONCLUSIONS_STORED`）；不复用、不上卡：
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
/// 流只从库里读回一次、各线程共用。
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
        if slice_count == 0 {
            // 空序号段：一个状态都不判，照单线程那一支交回整条流的状态数
            return crate::crash::judge_crash_state_range_of_the_read_back_flow(
                flow.base,
                flow.writes,
                flow.segments,
                flow.judged_root_index,
                flow.version_table(),
                flow.expansion,
                ordinals.clone(),
                &mut |_ordinal, _publish, _image, _judgement| {},
            );
        }
        let (judged, states_of_the_whole_flow) = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..threads.min(slice_count))
                .map(|_| {
                    let next_slice = &next_slice;
                    let ordinals = &ordinals;
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
                            let states = crate::crash::judge_crash_state_range_of_the_read_back_flow(
                                flow.base,
                                flow.writes,
                                flow.segments,
                                flow.judged_root_index,
                                flow.version_table(),
                                flow.expansion,
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
        assert_eq!(judged, state_count, "各线程判的状态数合起来是序号段的长度");
        states_of_the_whole_flow
    })
}

/// 取样那一趟判过的一个状态交给库的那一份：序号、它归哪次发布、CPU 判器的判定（整块都判到的块照 CPU 判器那一处存进库）。
type NoteASampledState<'note> =
    dyn Fn(u64, CrashPointReplayPublishOfState, &CrashStateJudgement) + Sync + 'note;

/// 取样的两段：录入段把 `flow_to_record` 按摘要写进崩溃点重放枚举那一类的库、项名下登清单；判段按项名读清单、按摘要读回流，
/// 只把读回的那一份交 `judge_the_read_back_flow`，它每判一个状态交给第二个参数：取样序号段整块罩住的块，判完照 CPU 判器那一处
/// （`JudgedBy::CpuJudge`，结论版本照计划各点那一把）存进库（用户 2026-10-13 定「能走两段的都走两段」：取样的结论也入库）。
/// 只录那一趟交 `RecordedOnly`。
fn sample_through_the_library(
    item: CrashPointReplayItem<'_>,
    flow_to_record: &FlowToRecord<'_>,
    ordinals: &Range<u64>,
    judge_the_read_back_flow: impl FnOnce(&FlowOfTheLibrary<'_>, &NoteASampledState<'_>) -> u64,
) -> StageOutcome<u64> {
    let repository_root = repository_root_of_this_crate();
    let item_name = item_name_of(item);
    two_stages(
        CrashLibraryClass::CrashStateEnumeration,
        &repository_root,
        &item_name,
        |library| flow_to_record.record_into(library, false),
        |library, manifest| {
            let read_back = RecordedFlowReadBack::read_from_the_library(
                &library.lock(),
                &manifest.digest_named(MANIFEST_FLOW)?,
            )?;
            let flow = read_back.flow();
            let plan = plan_crash_points(&flow);
            let toolchain = CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test();
            let conclusion_versions =
                ConclusionVersionsOfThePlan::of_the_plan(&repository_root, &plan, &toolchain)
                    .map_err(|not_writable| {
                        let line = not_writable.not_written_line();
                        println!("{line}");
                        line
                    })?;
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
        crash_points: vec![CrashPointSpan {
            name: item.name.to_string(),
            code_file_in_repository: item.setting_file.to_string(),
            writes: 0..writes.len(),
            ignored: false,
        }],
    }
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

    /// 录入段：排计划、写录入表与路径索引、把流写进库（配了 GPU 再把打包判表写进库），交回清单。
    fn record_into(
        &self,
        library: &CrashLibrary,
        gpu_judges: bool,
    ) -> Result<ItemManifest, String> {
        self.with_the_flow(|flow| {
            let mut store = library.lock();
            let plan = plan_crash_points(flow);
            write_the_recorded_blocks(&mut store, &plan)
                .map_err(|error| format!("录入失败：{error:?}"))?;
            let recorded_bytes = RecordedFlow::of(flow).to_bytes();
            let flow_digest = sha256_digest(&recorded_bytes);
            store
                .store_recorded_flow(&flow_digest, &recorded_bytes)
                .map_err(|error| format!("录下的流程写不进库：{error:?}"))?;
            drop(recorded_bytes);
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
    observers: &[CrashPointReplayObserver<'_>],
    observers_handing_back_images: &[CrashPointReplayObserverHandingBackAnImage<'_>],
) -> Result<StageOutcome<CrashPointReplayTally>, String> {
    let repository_root = repository_root_of_this_crate();
    let route = crash_point_replay_judge_route(&repository_root)?;
    let item_name = item_name_of(item);
    let flow_to_record = flow_of_the_inputs(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
    );
    let observers_on_the_nodes =
        observers_on_the_nodes_of_the_item(item, observers, observers_handing_back_images)?;
    let toolchain = CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test();
    let observer_versions: Vec<ObserverVersions> = observers_on_the_nodes
        .iter()
        .map(|observer| {
            observer_versions_to_write_under(
                &repository_root,
                observer,
                route.judged_by(),
                &toolchain,
            )
            .map_err(|not_writable| {
                let line = not_writable.not_written_line();
                println!("{line}");
                line
            })
        })
        .collect::<Result<_, _>>()?;
    two_stages(
        CrashLibraryClass::CrashStateEnumeration,
        &repository_root,
        &item_name,
        |library| {
            let manifest =
                flow_to_record.record_into(library, route == CrashPointReplayJudgeRoute::Gpu)?;
            if !observers_on_the_nodes.is_empty() {
                // 观察者的录入段（用户 2026-10-13 定「观察者改走两段」）：这台机器领的块逐状态看、结果写进库，判段从库里读
                flow_to_record.with_the_flow(|flow| {
                    let plan = plan_crash_points(flow);
                    let is_mine = blocks_of_this_host(&plan)?;
                    let started = Instant::now();
                    let stop_request =
                        StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
                    let recording = record_the_observers_of_the_flow(
                        &mut library.lock(),
                        flow,
                        &plan,
                        &observers_on_the_nodes,
                        &observer_versions,
                        &|block| is_mine.contains(&block),
                        &|| stop_request.is_requested(),
                    )?;
                    println!(
                        "CRASH_POINT_REPLAY_OBSERVERS_RECORDED item={} blocks_recorded_here={} states_recorded_here={} blocks_already_recorded={} states_counted_here={} images_observed_fresh={} states_reusing_an_observed_image={} stopped_early={} seconds={:.1}",
                        item.name,
                        recording.blocks_recorded_here,
                        recording.states_recorded_here,
                        recording.blocks_already_recorded,
                        recording.states_counted_here,
                        recording.images_observed_here,
                        recording.states_reusing_an_observed_image,
                        recording.stopped_early,
                        started.elapsed().as_secs_f64()
                    );
                    Ok::<(), String>(())
                })?;
            }
            Ok(manifest)
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
                &observers_on_the_nodes,
                &observer_versions,
                route,
                &repository_root,
            )
        },
    )
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

/// 一条崩溃枚举用例的观察者挂在它自己那个点与末节点上（设点文件就是用例文件，观察者代码文件也是它）。
///
/// # Errors
/// 观察者名字不合规矩。
fn observers_on_the_nodes_of_the_item<'observer>(
    item: CrashPointReplayItem<'_>,
    observers: &[CrashPointReplayObserver<'observer>],
    observers_handing_back_images: &[CrashPointReplayObserverHandingBackAnImage<'observer>],
) -> Result<Vec<ObserverOnTheNodes<'observer>>, String> {
    let nodes = || {
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
    };
    let observer_name = |name: &str| {
        ObserverName::new(name).map_err(|rejected| format!("观察者名字不合规矩：{rejected:?}"))
    };
    let mut observers_on_the_nodes = Vec::new();
    for observer in observers {
        observers_on_the_nodes.push(ObserverOnTheNodes {
            observer_name: observer_name(observer.name)?,
            observer_code_file: item.setting_file.to_string(),
            extra_inputs_digest: observer.extra_inputs_digest,
            nodes: nodes(),
            observe: ObserverCode::CountsOnly(observer.observe),
        });
    }
    for observer in observers_handing_back_images {
        observers_on_the_nodes.push(ObserverOnTheNodes {
            observer_name: observer_name(observer.name)?,
            observer_code_file: item.setting_file.to_string(),
            extra_inputs_digest: observer.extra_inputs_digest,
            nodes: nodes(),
            observe: ObserverCode::HandsBackAnImage {
                observe: observer.observe,
                deduplication: observer.deduplication,
            },
        });
    }
    Ok(observers_on_the_nodes)
}

/// 判段：凭清单读回流，排计划，这台机器领的、库里还没有这一把版本结论的块上卡判或由 CPU 判、存进库；观察者照样按块看、存；
/// 对账、核计数向量，从库里读回每一块的结论重建崩溃点重放的计数。
#[allow(
    clippy::too_many_lines,
    reason = "读回、选路、版本、分工、导入、判、看、对账、重建计数、打行，是判段的全部步骤，拆开要把十几个量来回传"
)]
fn judge_the_recorded_item(
    library: &CrashLibrary,
    manifest: &ItemManifest,
    item: CrashPointReplayItem<'_>,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    observer_versions: &[ObserverVersions],
    route: CrashPointReplayJudgeRoute,
    repository_root: &Path,
) -> Result<StageOutcome<CrashPointReplayTally>, String> {
    let started = Instant::now();
    let toolchain = CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test();
    let read_back = RecordedFlowReadBack::read_from_the_library(
        &library.lock(),
        &manifest.digest_named(MANIFEST_FLOW)?,
    )?;
    let flow_of_the_library = read_back.flow();
    let flow = &flow_of_the_library;
    let plan = plan_crash_points(flow);
    let conclusion_versions =
        ConclusionVersionsOfThePlan::of_the_plan(repository_root, &plan, &toolchain).map_err(
            |not_writable| {
                let line = not_writable.not_written_line();
                println!("{line}");
                line
            },
        )?;
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
    // 别的机器判完取回的库：只点名这条流里不归本机的块导进来
    let imported_blocks = match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_IMPORT") {
        Ok(list) => import_the_planned_blocks_of_the_other_hosts(
            &mut store,
            repository_root,
            &plan,
            &is_mine,
            &conclusion_versions,
            observers_on_the_nodes,
            &observer_conclusion_versions,
            &[],
            &list,
        )?,
        Err(_) => 0,
    };
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
        "CRASH_POINT_REPLAY_PARALLEL_START states={state_count} slices={blocks_of_this_share} worker_threads={worker_threads} configured_worker_threads={worker_threads} worker_threads_source={worker_threads_source} resumed_slices={blocks_reused} freshly_run_slices={blocks_to_judge} judge={} library={} item={} imported_blocks={imported_blocks}{share_fields}",
        route.name(),
        library.directory().display(),
        item.name
    );
    let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
    let stop_is_requested = || stop_request.is_requested();
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
            repository_root,
            &stop_is_requested,
        )?,
        CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature => unreachable!(
            "判段不跑的那一趟在 enumerate_crash_states_through_the_library 的判段闭包里先交只录，不进判段"
        ),
    }
    // 观察者的判段（用户 2026-10-13 定）：只从库里读录入段那几块，观察者交回的盘上的池级 checker 照第 ① 段的选路判（配了 GPU 上卡）。
    // 叫停了不判观察者：录入段可能停在块边上，没录的块判段读不到，交回叫停那一句，重跑同一条用例接着录、接着判
    if !observers_on_the_nodes.is_empty() && stop_is_requested() {
        return Err(
            "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE）：观察者这一趟没判完，录过、判过的在库里，重跑同一条用例接着录、接着判"
                .to_string(),
        );
    }
    let pool_checker = pool_checker_of_the_images_on_the_route(route, repository_root)?;
    let observers_judging_started = Instant::now();
    let observing = judge_the_observers_of_the_flow(
        &mut store,
        &plan,
        observers_on_the_nodes,
        observer_versions,
        &pool_checker,
        &is_mine,
    )?;
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
    // 对账：这条流的块里还没有这一把版本结论的
    let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &judged_conclusions)
        .map_err(|error| format!("对账失败：{error:?}"))?;
    if !unjudged.is_empty() {
        if stop_is_requested() {
            return Err(format!(
                "叫停了（SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE）：{} 块还没判，判过的在库里，重跑同一条用例接着判",
                unjudged.len()
            ));
        }
        if share.count > 1 {
            let blocks_left: u64 = u64::try_from(unjudged.len()).expect("块数装得进 u64");
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
        return Err(format!(
            "判段判完库里还有 {} 块没有这一把版本的结论（第一块：点 {}，起 {}）：判器那一路没写库",
            unjudged.len(),
            unjudged[0].0.stream_or_node_name.as_str(),
            unjudged[0].0.block_start.0
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
    let observer_kinds_and_versions: Vec<(ConclusionKind, ConclusionVersion)> =
        observers_on_the_nodes
            .iter()
            .zip(&observer_conclusion_versions)
            .map(|(observer, version)| {
                (
                    ConclusionKind::Observer {
                        observer_name: observer.observer_name.clone(),
                    },
                    *version,
                )
            })
            .collect();
    let tally = crash_point_replay_tally_from_the_library(
        &store,
        flow,
        &plan,
        judged_by.conclusion_kind(),
        match judged_by {
            JudgedBy::GpuJudge => &conclusion_versions.gpu_judge_of_each_point,
            JudgedBy::CpuJudge => &conclusion_versions.cpu_judge_of_each_point,
        },
        &observer_kinds_and_versions,
    )?;
    drop(store);
    if let Some(red) = observing.red_texts.first() {
        panic!(
            "观察者在 {} 个状态上 panic（第一个：路径 {} 点内第 {} 个状态）：{}",
            observing.red_states_of_each_observer.values().sum::<u64>(),
            red.0,
            red.1,
            red.2
        );
    }
    if let Some((observer_name, red_states)) = observing
        .red_states_of_each_observer
        .iter()
        .find(|(_, red_states)| **red_states > 0)
    {
        panic!(
            "观察者 {observer_name} 在 {red_states} 个状态上 panic（这一趟复用的块里存着红项，这一趟没看：改了观察者代码它的版本就换、会重看）"
        );
    }
    report_the_findings(&tally, item, state_count);
    println!(
        "CRASH_POINT_REPLAY_PARALLEL_FINISHED states={state_count} slices={blocks_of_this_share} worker_threads={worker_threads} configured_worker_threads={worker_threads} worker_threads_source={worker_threads_source} resumed_slices={blocks_reused} freshly_run_slices={blocks_to_judge} progress_file_after_completion=none elapsed_seconds={:.1} judge={}{share_fields}",
        started.elapsed().as_secs_f64(),
        route.name()
    );
    Ok(StageOutcome::Judged(tally))
}

/// 观察者交回的盘上的池级 checker 照第 ① 段的选路：CPU 那一路由 CPU 判；GPU 那一路取这一趟的卡（同 [`judge_the_pending_blocks_on_the_cards`]）。
///
/// # Errors
/// 选路走 GPU 而取不到卡。
fn pool_checker_of_the_images_on_the_route(
    route: CrashPointReplayJudgeRoute,
    repository_root: &Path,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    match route {
        CrashPointReplayJudgeRoute::Cpu { .. } => {
            Ok(PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu)
        }
        CrashPointReplayJudgeRoute::Gpu => pool_checker_of_the_images_on_the_cards(repository_root),
        CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature => unreachable!(
            "判段不跑的那一趟在 enumerate_crash_states_through_the_library 的判段闭包里先交只录，不判观察者交回的盘"
        ),
    }
}

#[cfg(feature = "gpu")]
fn pool_checker_of_the_images_on_the_cards(
    repository_root: &Path,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    use crate::crash_amplification::gpu_cards_in_use;
    use crate::gpu_card_configuration::{
        cards_by_priority_from_configuration_or_environment, CardHost,
    };
    let configuration_file = multi_host_configuration_file(repository_root)?;
    let cards_by_priority =
        cards_by_priority_from_configuration_or_environment(configuration_file.as_deref())?;
    let host = CardHost::of_this_process()?;
    let cards_of_this_run = gpu_cards_in_use(cards_by_priority.as_deref(), host)?;
    Ok(PoolCheckerOfTheImagesAfterTheObservers::OnTheGpu {
        cards: cards_of_this_run.cards,
        taken_from: cards_of_this_run.taken_from,
    })
}

/// 没编进 `gpu` 特性选路不会走 GPU（[`crash_point_replay_judge_route`]）。
#[cfg(not(feature = "gpu"))]
fn pool_checker_of_the_images_on_the_cards(
    _repository_root: &Path,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    Err("这份二进制没编进 gpu 特性，观察者交回的盘上不了卡（选路不该要它）".to_string())
}

/// 配了 GPU：读回清单里的判表、取卡、这台机器领的块上卡判。卡用不上、显存不够都报错，不退回 CPU。
#[cfg(feature = "gpu")]
#[allow(
    clippy::too_many_arguments,
    reason = "库、流、计划、清单、分工、判法版本、仓根、叫停，各是各的东西"
)]
fn judge_the_pending_blocks_on_the_cards(
    store: &mut VerdictStore,
    flow: &crate::crash_amplification::FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    manifest: &ItemManifest,
    is_mine: &dyn Fn(u64) -> bool,
    judged_conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    repository_root: &Path,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(), String> {
    use crate::crash_amplification::{
        gpu_cards_in_use, judge_my_blocks_on_gpu, packed_judge_tables_read_back,
        GpuJudgingOfMyBlocks,
    };
    use crate::gpu_card_configuration::{
        cards_by_priority_from_configuration_or_environment, CardHost,
    };
    let tables =
        packed_judge_tables_read_back(store, &manifest.digest_named(MANIFEST_JUDGE_TABLES)?)?;
    let configuration_file = multi_host_configuration_file(repository_root)?;
    let cards_by_priority =
        cards_by_priority_from_configuration_or_environment(configuration_file.as_deref())?;
    let host = CardHost::of_this_process()?;
    let cards_of_this_run = gpu_cards_in_use(cards_by_priority.as_deref(), host)?;
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
    _repository_root: &Path,
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
/// 违例、不适用与按发布分的态数；「根槽已持久」按被判根的位置事后数；观察者计数从它们的结论块读。
///
/// # Errors
/// 库读失败；某一块没有这一种结论（判段没写）；红项解不出；判定向量编号没登记。
fn crash_point_replay_tally_from_the_library(
    store: &VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    kind: ConclusionKind,
    version_of_each_point: &[ConclusionVersion],
    observers: &[(ConclusionKind, ConclusionVersion)],
) -> Result<CrashPointReplayTally, String> {
    let positions = PositionsOfTheStates::of_the_plan(flow, plan);
    let mut tally = CrashPointReplayTally::default();
    let invariant_named = |name: &str| -> Result<&'static str, String> {
        IMPLEMENTED_INVARIANTS
            .iter()
            .find(|invariant| **invariant == name)
            .copied()
            .ok_or_else(|| format!("计数向量里的不变量不在清单里：{name:?}"))
    };
    for (point_index, point) in plan.points.iter().enumerate() {
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
                    format!("判定块里的编号 {} 没在这个库里登记过", number.as_u8())
                })?;
                let items = red_items_of_the_encoded_verdict(vector);
                tally.states += 1;
                let (segment, publish) = positions.position_of(ordinal);
                *tally.states_by_publish.entry(publish).or_insert(0) += 1;
                let [consulted_instance, consulted_txg, consulted_outcome, consulted_failure_point, ignored_instance, ignored_txg, ignored_outcome, ignored_failure_point] =
                    words.0;
                match consulted_outcome {
                    RECOVERY_OUTCOME_CODE_NO_FILE => tally.no_file_states += 1,
                    RECOVERY_OUTCOME_CODE_FILE_READ => tally.file_read_states += 1,
                    RECOVERY_OUTCOME_CODE_FAILED => tally.failed_states += 1,
                    other => return Err(format!("恢复字里的结局码认不得：{other}")),
                }
                if (
                    consulted_instance,
                    consulted_txg,
                    consulted_outcome,
                    consulted_failure_point,
                ) != (
                    ignored_instance,
                    ignored_txg,
                    ignored_outcome,
                    ignored_failure_point,
                ) {
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
            for (observer_kind, observer_version) in observers {
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
                tally.observed_states += block_states;
                for (counter, count) in &observed.count_vector.per_observer {
                    tally.observer_counts.add(counter, *count);
                }
            }
        }
    }
    if !observers.is_empty() {
        // 几个观察者各看了每个状态一遍：看过的状态数按一个观察者算
        tally.observed_states /= u64::try_from(observers.len()).expect("观察者数装得进 u64");
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
    Ok(tally)
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

    /// 用户 2026-10-13 定「配 GPU 必用 GPU、做不到报错不降级」：配置开了 GPU 而这份二进制没编进 gpu 特性，判段不跑（不退回 CPU 判），
    /// 结论版本按配置的那一路（GPU）；打的本次未跑那一行是门禁 checker-tier-crash-point-replay 的 `test_not_run_lines` 认的那一族记号、带着项名，
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
                singlefs_harness::heavy_threads::registered_heavy_threads().get(),
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

    #[test]
    fn a_crash_point_replay_item_name_cannot_be_a_path_segment_with_a_slash() {
        let item = CrashPointReplayItem {
            name: "fine_name",
            setting_file: "crates/singlefs-checker-tier/tests/x.rs",
        };
        assert_eq!(item_name_of(item).as_str(), "fine_name");
        let caught = std::panic::catch_unwind(|| {
            item_name_of(CrashPointReplayItem {
                name: "a/b",
                setting_file: "crates/singlefs-checker-tier/tests/x.rs",
            })
        });
        assert!(caught.is_err(), "带 / 的项名要停下");
    }
}
