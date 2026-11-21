//! 崩溃点重放枚举用例的两段入口（崩溃判定库的 [`CrashLibraryClass::CrashStateEnumeration`] 类）：录入段在 CPU 上把流写进库，判段只从库里读回、
//! 按块判（配了 GPU 写成待判行交 GPU 批处理服务判，没配由 CPU 判）、结论按（块键，结论种类，版本）存进库，判过的块下一趟复用（剪枝），崩溃点重放的计数
//! （[`CrashPointReplayTally`]）从库里读回的结论重建。判一个状态的判法在 `crate::crash`（CPU）与 `crate::crash_judge_gpu`（GPU），这里只排两段、
//! 领块、存库、读回。
//!
//! 一条崩溃枚举用例的流按步切成前缀树的节点（L6：写下标区间互相交叠的几段并成一步，一步一个崩溃点，点名 `<项名>.<周期>.<步序>` 只作索引），
//! 加末节点（整张写表全部持久那一个状态，`crash_amplification::END_OF_FLOW_NODE_NAME`）；块键与崩溃放量同一个算法、只由内容定
//! （段写表、起步镜像、计划哈希），几条流共有的前几步只判一次；镜像判定的版本由结论种类与判器引到的被检测产品代码定，判器的代码不进键；同一块上几个观察者由各自的
//! 判定项标识分开（观察者代码文件与观察者名，`crate::crash_amplification::observer_identity_digest`），观察者的代码、设点文件、项名都不进键。
//! 录入段按录入键入库，命中就不再录（`crate::crash_library::two_stages`；录入键只由录下的写流摘要与这条用例的项名组成，
//! `crate::crash_library::recording_key_of_the_scenario_and_the_inputs`，代码一个单元都不进）。
//! 跑完打一行 `CRASH_CASE_BLOCKS blocks= reused= fresh= red= refused= states=`（[`CrashCaseBlocks`]）；块一律照键读库复用，缺的当场判；
//! 挂了没登记的观察者的整条用例照判、没登记的那几个不入库，块计入 `refused=`。
//! 两台分片与崩溃放量同一种读法（`SINGLEFS_CRASH_AMPLIFICATION_SHARE`、`…_HOST_WEIGHTS`、`…_IMPORT`）；按块序号区间选块
//! （`crate::crash_amplification::BLOCK_RANGE_ENVIRONMENT_VARIABLE`）这一路不认，设了就拒（读份那一步交错）。
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
    check_blocks_where_until_with_threads, count_vector_disagreements_of_the_flow,
    crash_flow_coverage_lines, observer_versions_keeping_the_refused_out_with_the_observer_keys_in,
    observers_whose_recorded_blocks_are_missing, pending_blocks, plan_crash_points,
    record_the_observers_of_the_flow, states_with_the_root_slot_write_persisted,
    stop_is_requested_by_the_environment, unjudged_blocks_of_the_flow,
    write_the_recorded_blocks_taking_the_library_lock_row_by_row, CardSourcesOfTheEngineStart,
    ConclusionVersionsOfThePlan, CpuJudgementsOfSampledBlocks, CrashFlow, CrashPlan,
    CrashPointSpan, ExpandUpToWritesPerSegment, FlowOfTheLibrary, JudgedBy, NodeOfAnObserver,
    ObserveACrashState, ObserveACrashStateHandingBackAnImage, ObserverCode, ObserverDeduplication,
    ObserverOnTheNodes, ObserverVersions, ObservingOfTheRun,
    PoolCheckerOfTheImagesAfterTheObservers, RecordedFlow, RecordedFlowAndItsPlan,
    RecordedFlowReadBack, WeightedShares, END_OF_FLOW_NODE_NAME,
};
use crate::crash_identity::{
    ConclusionKind, ConclusionVersion, ObserverJudgingUnitsKeyTable, ObserverName,
};
use crate::crash_judge_conclusions::{
    red_items_of_the_encoded_verdict, RECOVERY_OUTCOME_CODE_FAILED,
    RECOVERY_OUTCOME_CODE_FILE_READ, RECOVERY_OUTCOME_CODE_NO_FILE,
};
use crate::crash_library::{
    assert_this_thread_holds_no_library_lock, recording_key_of_the_scenario_and_the_inputs,
    two_stages_under_the_recording_key,
    two_stages_under_the_recording_key_topping_up_a_reused_recording, CrashLibrary,
    CrashLibraryClass, ItemManifest, ItemName, LibraryTakenAtEachReadOrWrite,
    RecordingInputsDigest, RunStage, StageOutcome,
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

mod block_account;
mod count_assertions;
mod cpu_sampling;
mod enumeration_entries;
mod flow_through_the_library;
mod judge_route_and_worker_threads;
mod judging_stage;
mod keys_and_observer_nodes;
mod recording_stage;
mod tally_from_the_library;

pub use block_account::*;
pub use count_assertions::*;
pub use cpu_sampling::*;
pub use enumeration_entries::*;
use flow_through_the_library::*;
pub use judge_route_and_worker_threads::*;
use judging_stage::*;
use keys_and_observer_nodes::*;
pub use recording_stage::*;
use tally_from_the_library::*;
#[cfg(test)]
use tests::{
    block_keys_of_each_node, blocks_of_the_plan, every_proper_subset, observe_no_crash_state,
    store_a_conclusion_block_of_the_unit_test, with_the_unit_writes_flow_and_an_empty_library,
    writes_and_segments_of, BLOCK_ACCOUNT_PROBE_ITEM,
};

/// 清单里流那一样的名字。
const MANIFEST_FLOW: &str = "flow";
/// 清单里 GPU 判表那一样的名字（配了 GPU 才录）。
const MANIFEST_JUDGE_TABLES: &str = "judge_tables";
/// 清单的种类。
const MANIFEST_KIND: &str = "crash_state_enumeration";

/// 一条崩溃枚举用例的项：项名（行里报的名字；崩溃枚举用例也拿它当这条流那个崩溃点的名字，进节点身份、路径与录下的流的字节，取样那一族不拿）与设点文件
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

    /// 录入键的输入那一半：录进库的那一份流的字节与判表开不开（配了 GPU 才打包判表）。字节里带崩溃点的名字与设点文件（[`RecordedFlow::to_bytes`]）：
    /// 崩溃枚举用例的项名固定，一条用例一把键；取样那一族名字固定（[`CRASH_POINT_NAME_OF_A_SAMPLED_FLOW`]），调用方的项名不进。
    fn recording_inputs_digest(recorded_bytes: &[u8], gpu_judges: bool) -> RecordingInputsDigest {
        let mut message = b"singlefs crash state enumeration recording inputs\n".to_vec();
        message.extend_from_slice(&sha256_digest(recorded_bytes));
        message.extend_from_slice(format!("gpu_judges={gpu_judges}\n").as_bytes());
        RecordingInputsDigest(sha256_digest(&message))
    }

    /// 录入段：排计划、写录入表与路径索引、把流写进库（配了 GPU 再把打包判表写进库），交回清单。排计划、算摘要、打包都在锁外，锁只包写库的往返。
    fn record_into(
        &self,
        library: &CrashLibrary,
        gpu_judges: bool,
        recorded_bytes: &[u8],
    ) -> Result<ItemManifest, String> {
        self.with_the_flow(|flow| {
            assert_this_thread_holds_no_library_lock("录入段排计划");
            let plan = plan_crash_points(flow);
            write_the_recorded_blocks_taking_the_library_lock_row_by_row(library, &plan)
                .map_err(|error| format!("录入失败：{error:?}"))?;
            assert_this_thread_holds_no_library_lock("录入段算流的摘要");
            let flow_digest = sha256_digest(recorded_bytes);
            library
                .lock()
                .store_recorded_flow(&flow_digest, recorded_bytes)
                .map_err(|error| format!("录下的流程写不进库：{error:?}"))?;
            let mut digests = vec![(MANIFEST_FLOW.to_string(), flow_digest)];
            if gpu_judges {
                digests.push((
                    MANIFEST_JUDGE_TABLES.to_string(),
                    record_the_judge_tables(library, flow, &plan)?,
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

/// 这条用例挂着的观察者各自的几把版本（判定项标识、观察者引到的被检测产品代码与另读的输入，[`crate::crash_amplification::observer_identity_digest`]；
/// 判器与观察者自己的代码都不进键，镜像判定的版本由结论种类与判器引到的被检测产品代码定）。哪几个没登记或那一组拒算
/// （[`crate::crash_identity::observer_product_code_key_in`]）：那几个取这个进程的
/// 占位、照判不入库（键里带着占位与从它拼出来的版本的行只留在这个进程里，`VerdictStore::keep_the_rows_of_these_versions_out_of_the_library`），
/// 别的照键入库同一份长期库（不另建一次性库、不退回整文件或整表指纹）；这条用例的块全部计入 `refused=`。
struct KeysOfTheItem {
    /// 挂着的每个观察者那几把版本（没登记的那几个是占位拼的）。
    observer_versions: Vec<ObserverVersions>,
    /// 拒算的那几组，一组一句（行里逐个点名）。
    refusal_sentences: Vec<String>,
    /// 照判、不入库的那几把版本：开库之后登记进库。
    versions_kept_out_of_the_library: Vec<[u8; DIGEST_BYTES]>,
}

impl KeysOfTheItem {
    /// 有没有哪一组拒算（这条用例的块全部计入 `refused=`）。
    fn are_refused(&self) -> bool {
        !self.refusal_sentences.is_empty()
    }
}

/// 块账里挂着的一个观察者：计划里每个点它挂没挂（与计划的点同序）、这一趟它那一把结论版本。
struct ObserverInTheBlockAccount {
    holds_each_point: Vec<bool>,
    conclusion: ConclusionVersion,
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

/// 判段走 GPU 那一路从进程环境读的，判段入口照起步那一刻的环境读一次、往下传，判段里不再读进程环境
/// （同一个测试二进制里别的用例拿着环境守卫改着多机配置、提交方仓根时，判段照的仍是入口那一刻的）。
struct GpuJudgingInputsOfTheEntry {
    /// 观察者交回的盘上取卡照的显卡来路（[`CardSourcesOfTheEngineStart::read_from_the_environment_now`]）。
    card_sources: CardSourcesOfTheEngineStart,
}

impl GpuJudgingInputsOfTheEntry {
    /// 此刻环境里的显卡来路，多机配置是入口读的那一份 `configuration_file`。
    fn read_at_the_entry(configuration_file: std::path::PathBuf) -> Self {
        Self {
            card_sources: CardSourcesOfTheEngineStart::read_from_the_environment_now(
                configuration_file,
            ),
        }
    }
}

/// 判段这一趟判器那一半键取没取得出，与它把块账交回到哪。
struct JudgingUnitsOfTheRun<'run> {
    /// 有一组拒算：那几组照判不入库，块全部计入 `refused=`。
    are_refused: bool,
    blocks_of_the_run: &'run std::cell::Cell<Option<CrashCaseBlocks>>,
}

/// 这条流的块里还缺结论的块数（[`blocks_of_the_flow_without_every_conclusion`]）。
struct BlocksOfTheFlowWithoutEveryConclusion {
    /// 挂着的观察者里至少一个在这一趟那一把版本下没有结论的块。
    missing_an_observer_conclusion: usize,
    /// 判器结论或观察者结论缺一种的块（两种都缺的算一块）：双机的一份收工时留给别的机器的块数。
    missing_the_judgement_or_an_observer_conclusion: u64,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 单测小流（[`with_the_unit_writes_flow_and_an_empty_library`]）的项。
    pub(super) const BLOCK_ACCOUNT_PROBE_ITEM: CrashPointReplayItem<'static> =
        CrashPointReplayItem {
            name: "block_account_probe",
            setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
        };

    /// 单测的小流（三次单元写一段、一次根槽写一段，9 个状态，三个节点各一块：与 [`the_blocks_line_reports_the_states_in_the_blocks_of_this_share`]
    /// 那一条同一条）与一份空库，交给 `check` 看它的流、计划与库。
    pub(super) fn with_the_unit_writes_flow_and_an_empty_library<Outcome>(
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
    pub(super) fn blocks_of_the_plan(
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
    pub(super) fn store_a_conclusion_block_of_the_unit_test(
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
    pub(super) fn writes_and_segments_of(
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

    pub(super) fn every_proper_subset(
        _segment_index: usize,
        _segment: &[usize],
    ) -> CrashPointReplaySegmentExpansion {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    }

    /// 一条流每个节点（计划里的点，连同末节点）各自的块键。
    pub(super) fn block_keys_of_each_node(
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

    pub(super) fn observe_no_crash_state(
        _ordinal: u64,
        _image: &CrashImage<'_>,
        _report: &singlefs_core::recovery::RecoveryReport,
        _counts: &mut crate::crash::CrashPointReplayObserverCounts,
    ) {
    }
}
