//! 崩溃注入（里程碑「第二个事务」增补 3 第 3 件）：第 1 件的每段历史跑的过程中开着内容保留的录制流，把流按屏障与 FUA 写切段，
//! 按种子摆若干个崩溃状态（一段 + 那一段里持久了哪几个写），对每个崩溃状态重建镜像、跑恢复、跑池级 checker、跑记录核对器，
//! 并问理想模型「恢复到的这一版允不允许」（`crate::model::crash_recovery_disagreement`）。
//!
//! 枚举域与层 0（`crate::crash`）同一个：更早的段整段持久、当前段任意真子集、更晚的段一个都没持久，撕裂并进「没持久」
//! （D13（验证路线） 已定项 4）。差别只在**怎么取**：层 0 在两条写死的流上全枚举，这里在任意一段随机历史上抽样——
//! 关闭重开、可写挂载、回退、抬 F、零单元发布、冷启动都在流里，而那样的流全枚举不起。
//! 屏障因此进了枚举域：少一道屏障就把两段并成一段，段内子集立刻多出「后发的写先持久」那一类状态
//! （`crates/mutations.tsv` 里「步 3：零单元发布在记录与根之间少一道屏障」那一条靠这个判得出）。
//!
//! 每段历史自己怎么跑由 `HistoryExecution` 定；崩溃状态摆在起点那一段起、最后一个跑完的操作为止的那几段上——
//! 起点（mkfs 与起点那次发布）也算（代码三方 m2-supp3-item3-code-r1 判决 K1-d，用户 2026-09-20 定案第 4 条），
//! 失败那一步的写不摆（模型还没判过它写出的根，目录里没有那一版）。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
};

use crate::crash::{
    check_records_against, newest_persisted_root, some_publish_persisted_without_its_root,
    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, RecordCheck, RetainedWrite,
    SECTOR_BYTES,
};
use crate::history::{
    classify_failure, execute_history_with, generate_history_with_weights,
    newest_ring_root_and_slot_count_of, raised_floor_lands_only_on_abandoned_roots,
    record_check_aspects, FailureObservation, FailureSignature, GeneratedHistory,
    GenerationWeights, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
    HistorySeed, SeededRandomSource, StepPosition, KNOWN_RED_FORMS,
};
use crate::model::{
    crash_recovery_disagreement, ModelDisagreement, ModelRootKey, ObservedReadBack,
};
use crate::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use crate::segments::{FixedGeometry, StepKind};
use crate::SharedStream;

/// 崩溃注入的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。
pub const CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_INJECTION_THREADS";

/// 这一个测试周期用的种子基：随机历史那五段（增补 3 第 1 件）与崩溃注入这三档（第 3 件）的种子区间都从它起。
///
/// **一个测试周期**从两件事里先到的那一件开始：开一个新里程碑，或者用户显式说从零开始测试 / 重建测试。
/// 周期因此可能比里程碑短——里程碑跑到一半被显式重建，那就是新的一个周期。
///
/// 当前这个周期（里程碑「第二个事务」）的基是 2026-09-20 抽的，抽法：
/// `python3 -c "import secrets; print(secrets.randbits(63))"`。
///
/// **开一个新周期要做三样**：
/// 1. 照上面那条命令重抽一次，把下面这个数换掉；
/// 2. 旧的数据盘与虚拟盘镜像全删掉、全部重建（上一个周期的盘面是在旧种子基上攒出来的）；
/// 3. `crates/mutations.tsv` 整表在新种子基上重验一遍——点名随机历史与崩溃注入那些行的判红，都是在旧基上量的。
///
/// 周期之内写死不动：`crates/mutations.tsv` 的「这条变异必须红」与里程碑那几条验收说的都是**这一批**历史上的判红，
/// 种子基一动它们就成了掷骰子，判别力本身没了。周期之间重抽，才不会永远只测同一批历史（用户 2026-09-20 定案第 7 条，
/// 同日定的范围：随机说的是多次实验之间的随机，不是每次跑重抽）。它接着用户已定的另一条：
/// 历史实验数据只在它自己那个周期下有效。
pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 7_463_871_032_432_355_113;

/// 摆崩溃状态的随机源与生成历史那一路岔开：同一个种子，历史怎么生成不受它影响，反过来也一样。
const CRASH_POINT_SEED_SALT: u64 = 0x63_72_61_73_68_70_74_00;

/// 每个工作线程摊到的片数：各段历史的长短差得远（一步就被拒的与连发四十次的），片切得比线程多，先跑完的线程接着领下一片。
const SLICES_PER_WORKER_THREAD: usize = 4;

/// 段内写数不超过这个数时，子集掩码直接用一个 u64 抽；更长的段逐写各抽一次。
const WRITES_A_SUBSET_MASK_HOLDS: usize = 63;

/// 工作线程数是从哪来的：与实际起的线程数一起打进进度行，「机器多于 1 核却只用了 1 个线程」看得出来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashInjectionWorkerThreads {
    /// 环境变量 [`CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    FromTheEnvironmentVariable(usize),
    /// 没设环境变量，取 `available_parallelism`。
    FromAvailableParallelism(usize),
    /// 没设环境变量，`available_parallelism` 也报不出来：只用 1 个线程，照实报出来。
    AvailableParallelismUnknown,
    /// 调用方在代码里直接给的（用例拿不同线程数对拍）。
    GivenByCaller(usize),
}

impl CrashInjectionWorkerThreads {
    /// 线程数取环境变量，没设就取 `available_parallelism`。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程。
    #[must_use]
    pub fn from_the_environment() -> Self {
        Self::from_the_environment_variable_named(
            CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
        )
    }

    /// 线程数取 `variable_name` 这个环境变量，没设就取 `available_parallelism`。
    /// 坏盘输入（增补 3 第 5 件）拿它配自己那个变量名：判定只有「环境变量 → 正整数」这一件事，
    /// 抄一份出来两边会分叉（`code-discipline.md`「重复要生成，不许手抄」）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程。
    #[must_use]
    pub fn from_the_environment_variable_named(variable_name: &str) -> Self {
        Self::from_the_environment_value(
            variable_name,
            std::env::var(variable_name),
            std::thread::available_parallelism,
        )
    }

    /// [`Self::from_the_environment_variable_named`] 的判定本身：环境变量读到什么、`available_parallelism` 报什么
    /// 都由调用方给（用例不改进程的环境变量）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数。
    #[must_use]
    fn from_the_environment_value(
        variable_name: &str,
        variable: Result<String, std::env::VarError>,
        available_parallelism: impl Fn() -> std::io::Result<std::num::NonZeroUsize>,
    ) -> Self {
        match variable {
            Ok(text) => {
                let count: usize = text.trim().parse().unwrap_or_else(|error| {
                    panic!("{variable_name}={text} 不是十进制正整数：{error}")
                });
                assert!(count > 0, "{variable_name}={text}：至少要 1 个线程");
                Self::FromTheEnvironmentVariable(count)
            }
            Err(std::env::VarError::NotPresent) => match available_parallelism() {
                Ok(count) => Self::FromAvailableParallelism(count.get()),
                Err(_) => Self::AvailableParallelismUnknown,
            },
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{variable_name} 不是 UTF-8：{raw:?}")
            }
        }
    }

    #[must_use]
    pub fn count(self) -> usize {
        match self {
            Self::FromTheEnvironmentVariable(count)
            | Self::FromAvailableParallelism(count)
            | Self::GivenByCaller(count) => count,
            Self::AvailableParallelismUnknown => 1,
        }
    }

    #[must_use]
    pub fn source_name(self) -> &'static str {
        match self {
            Self::FromTheEnvironmentVariable(_) => "environment_variable",
            Self::FromAvailableParallelism(_) => "available_parallelism",
            Self::AvailableParallelismUnknown => "available_parallelism_unknown",
            Self::GivenByCaller(_) => "given_by_caller",
        }
    }
}

/// 一段历史上摆几个崩溃状态、怎么挑。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPointDraw {
    /// 按种子抽这么多个（每个是「一段 + 那一段的一个真子集」，同一段同一个子集只摆一次）。
    Sampled { crash_points_per_history: usize },
    /// 写数不超过 `segment_writes` 的段整段枚举（全部真子集），更长的段按种子各抽 `sampled_in_longer_segments` 个：
    /// 写死的那几段历史靠它把小段摆全，判别力不靠运气。
    EveryProperSubsetOfShortSegments {
        segment_writes: usize,
        sampled_in_longer_segments: usize,
    },
}

/// 失败时那份崩溃镜像留不留。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureImageRetention {
    /// 留着，路径打进报告（探索档／大档：现场要能重看）。
    KeepTheImageFiles,
    /// 整批跑完就把这一批的镜像目录删掉（门禁档不在临时目录里留垃圾）。
    DeleteTheImageFilesWhenTheRunFinishes,
}

/// 一个崩溃状态：截在录制流的第几段、那一段里哪几个写持久了。
/// 更早的段整段持久、更晚的段一个都没持久；这一段里持久的是一个真子集（全持久等于「没崩在这一段」，那个状态由下一段的空子集摆）。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrashPoint {
    pub segment_index: usize,
    /// 这一段里第 k 个写持久了没有（长度就是这一段的写数）。
    pub persisted_within_the_segment: Vec<bool>,
    /// 这一段里第一个没持久的写落在历史的哪一步里。
    pub step: StepPosition,
    /// 那一步是哪一类操作；落在起点里的是 None。
    pub operation_kind: Option<HistoryOperationKind>,
}

impl CrashPoint {
    /// 段内持久的那几个写不是这一段的前缀：有一个写被扣下了，它后面却有写持久了
    /// （「后发的写先持久」那一类状态，只截前缀摆不出来）。
    #[must_use]
    pub fn withholds_a_write_before_a_persisted_one(&self) -> bool {
        self.persisted_within_the_segment
            .iter()
            .position(|persisted| !persisted)
            .is_some_and(|withheld| {
                self.persisted_within_the_segment[withheld..]
                    .iter()
                    .any(|persisted| *persisted)
            })
    }

    /// 给人看：第几段、段内持久了几个写、扣下的写在段内的下标。
    #[must_use]
    pub fn render(&self) -> String {
        let withheld: Vec<usize> = self
            .persisted_within_the_segment
            .iter()
            .enumerate()
            .filter(|(_, persisted)| !**persisted)
            .map(|(index, _)| index)
            .collect();
        format!(
            "第 {} 段（{} 个写，持久 {} 个，扣下段内第 {:?} 个）",
            self.segment_index,
            self.persisted_within_the_segment.len(),
            self.persisted_within_the_segment
                .iter()
                .filter(|persisted| **persisted)
                .count(),
            withheld
        )
    }
}

/// 一段历史上一类崩溃状态失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointFinding {
    pub signature: FailureSignature,
    pub seed: HistorySeed,
    pub crash_point: CrashPoint,
    pub observation: FailureObservation,
    /// 恢复读回了什么（给人看）。
    pub read_back: String,
    /// 这份崩溃镜像落成文件之后放在哪；没落盘（不留现场，或写文件没成）时是 None。
    pub image_files: Option<PathBuf>,
}

impl CrashPointFinding {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "崩溃状态上的新发现 {:?}：种子 {}，{}，落在 {:?}／{:?}",
            self.signature,
            self.seed.0,
            self.crash_point.render(),
            self.crash_point.step,
            self.crash_point.operation_kind
        );
        let _ = writeln!(text, "  恢复读回：{}", self.read_back);
        for (invariant, detail) in &self.observation.violations {
            let _ = writeln!(text, "  {invariant}：{detail}");
        }
        for aspect in record_check_aspects(&self.observation.record_check) {
            let _ = writeln!(text, "  记录核对器判红：{aspect}");
        }
        if let Some(panic) = &self.observation.panic {
            let _ = writeln!(text, "  panic 在 {}：{}", panic.location, panic.message);
        }
        if let Some(disagreement) = &self.observation.model_disagreement {
            let _ = writeln!(
                text,
                "  模型对不上（{}）：模型答 {}；实现 {}",
                disagreement.aspect.name(),
                disagreement.model_answer,
                disagreement.implementation_answer
            );
        }
        if let Some(directory) = &self.image_files {
            let _ = writeln!(text, "  崩溃镜像留在：{}", directory.display());
        }
        let _ = writeln!(
            text,
            "  复现：SINGLEFS_CRASH_INJECTION_FIRST_SEED={} SINGLEFS_CRASH_INJECTION_SEEDS=1 跑大档那条 #[ignore] 用例",
            self.seed.0
        );
        text
    }
}

/// 崩溃状态上以「已知红」收尾的一次：种子、清单第几条、哪个崩溃状态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownRedAtACrashPoint {
    pub seed: HistorySeed,
    pub form: usize,
    pub crash_point: CrashPoint,
}

/// 跑过的崩溃状态的计数：绝对数，证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrashInjectionTally {
    pub histories: u64,
    /// 这段历史自己怎么收尾（崩溃状态之外）。
    pub histories_completed: u64,
    pub histories_ended_known_red: BTreeMap<usize, u64>,
    pub histories_ended_new_finding: u64,
    /// 一个崩溃状态都摆不出的历史段数（流里一个候选段都没有）。
    pub histories_without_any_crash_point: u64,
    pub crash_points: u64,
    /// 按段内被扣下的写的种类分（四种落点都扣下过才算罩全）；一个崩溃状态扣下几种就各记一次。
    pub crash_points_withholding_write_kind: BTreeMap<&'static str, u64>,
    /// 按崩溃状态落在哪一类操作里分（固定脚本只有一条路，这里证明挂载、回退、抬 F、冷启动的写上都摆过）。
    pub crash_points_by_operation_kind: BTreeMap<HistoryOperationKind, u64>,
    /// 崩溃状态落在起点那一段里（起点段 2026-09-20 才放开，这个数证明真的摆到了）。
    pub crash_points_inside_the_starting_point: u64,
    /// 段内持久的不是前缀：「后发的写先持久」那一类状态摆出来了几个。
    pub crash_points_withholding_a_write_before_a_persisted_one: u64,
    /// 截断之后盘上已持久的最新根槽，比这段历史最后提交的那一版旧：真的把状态截回了过去。
    pub crash_points_that_land_before_the_last_committed_version: u64,
    /// 一次发布已经写过东西而它的根槽还没持久。
    pub crash_points_inside_an_unfinished_publish: u64,
    pub recoveries_reading_a_file: u64,
    pub recoveries_without_a_file: u64,
    pub recoveries_failed: u64,
    /// 恢复真的施加过 journal 记录前缀的崩溃状态数。
    pub recoveries_that_applied_journal_records: u64,
    pub checker_runs: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
    /// 记录核对器跑过几次、两类各判红几次（与层 0 那一路同一份判据，`crash::check_records_against`）。
    pub record_checks: u64,
    pub record_root_without_record: u64,
    pub record_claimed_state_missing_unit: u64,
    /// 问过模型「恢复到的这一版允不允许」的次数（每个崩溃状态一次）。
    pub model_judgements: u64,
    /// 模型认得择到的那条根、并真的逐字节比过内容的次数。
    pub model_contents_compared: u64,
    /// 模型认得择到的那条根、那一版树表 0 条、实现也报没有文件的次数。
    pub model_versions_without_a_file_matched: u64,
    /// 攒到的「模型提交过的每一版」目录里，最大的一段有几版。
    pub most_committed_versions_in_one_history: usize,
    pub crash_states_ending_known_red: BTreeMap<usize, u64>,
    pub crash_states_ending_new_finding: u64,
}

impl CrashInjectionTally {
    /// 把紧跟在后面的那一段的计数并进来：逐项相加，按字段拆开写全——新加一个字段而这里没并，编译不过。
    pub fn absorb(&mut self, following: &CrashInjectionTally) {
        let CrashInjectionTally {
            histories,
            histories_completed,
            histories_ended_known_red,
            histories_ended_new_finding,
            histories_without_any_crash_point,
            crash_points,
            crash_points_withholding_write_kind,
            crash_points_by_operation_kind,
            crash_points_inside_the_starting_point,
            crash_points_withholding_a_write_before_a_persisted_one,
            crash_points_that_land_before_the_last_committed_version,
            crash_points_inside_an_unfinished_publish,
            recoveries_reading_a_file,
            recoveries_without_a_file,
            recoveries_failed,
            recoveries_that_applied_journal_records,
            checker_runs,
            invariant_holds,
            invariant_not_applicable,
            record_checks,
            record_root_without_record,
            record_claimed_state_missing_unit,
            model_judgements,
            model_contents_compared,
            model_versions_without_a_file_matched,
            most_committed_versions_in_one_history,
            crash_states_ending_known_red,
            crash_states_ending_new_finding,
        } = following;
        self.histories += histories;
        self.histories_completed += histories_completed;
        for (form, count) in histories_ended_known_red {
            *self.histories_ended_known_red.entry(*form).or_insert(0) += count;
        }
        self.histories_ended_new_finding += histories_ended_new_finding;
        self.histories_without_any_crash_point += histories_without_any_crash_point;
        self.crash_points += crash_points;
        for (kind, count) in crash_points_withholding_write_kind {
            *self
                .crash_points_withholding_write_kind
                .entry(kind)
                .or_insert(0) += count;
        }
        for (kind, count) in crash_points_by_operation_kind {
            *self
                .crash_points_by_operation_kind
                .entry(*kind)
                .or_insert(0) += count;
        }
        self.crash_points_inside_the_starting_point += crash_points_inside_the_starting_point;
        self.crash_points_withholding_a_write_before_a_persisted_one +=
            crash_points_withholding_a_write_before_a_persisted_one;
        self.crash_points_that_land_before_the_last_committed_version +=
            crash_points_that_land_before_the_last_committed_version;
        self.crash_points_inside_an_unfinished_publish += crash_points_inside_an_unfinished_publish;
        self.recoveries_reading_a_file += recoveries_reading_a_file;
        self.recoveries_without_a_file += recoveries_without_a_file;
        self.recoveries_failed += recoveries_failed;
        self.recoveries_that_applied_journal_records += recoveries_that_applied_journal_records;
        self.checker_runs += checker_runs;
        for (invariant, count) in invariant_holds {
            *self.invariant_holds.entry(invariant).or_insert(0) += count;
        }
        for (invariant, count) in invariant_not_applicable {
            *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
        }
        self.record_checks += record_checks;
        self.record_root_without_record += record_root_without_record;
        self.record_claimed_state_missing_unit += record_claimed_state_missing_unit;
        self.model_judgements += model_judgements;
        self.model_contents_compared += model_contents_compared;
        self.model_versions_without_a_file_matched += model_versions_without_a_file_matched;
        self.most_committed_versions_in_one_history = self
            .most_committed_versions_in_one_history
            .max(*most_committed_versions_in_one_history);
        for (form, count) in crash_states_ending_known_red {
            *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
        }
        self.crash_states_ending_new_finding += crash_states_ending_new_finding;
    }

    /// 给人看的一整块：绝对数——摆了几个崩溃状态、恢复成了几次、checker 与记录核对器跑了几次、模型判了几次、新发现几条。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；一个崩溃状态都摆不出的 {} 段；模型目录最多 {} 版",
            self.histories,
            self.histories_completed,
            self.histories_ended_known_red,
            self.histories_ended_new_finding,
            self.histories_without_any_crash_point,
            self.most_committed_versions_in_one_history
        );
        let _ = writeln!(
            text,
            "崩溃状态 {} 个：截在发布中间（根还没落盘）的 {} 个、截回到最后一版之前的 {} 个、段内有洞（后发的写先持久）的 {} 个、落在起点那一段里的 {} 个",
            self.crash_points,
            self.crash_points_inside_an_unfinished_publish,
            self.crash_points_that_land_before_the_last_committed_version,
            self.crash_points_withholding_a_write_before_a_persisted_one,
            self.crash_points_inside_the_starting_point
        );
        for (kind, count) in &self.crash_points_withholding_write_kind {
            let _ = writeln!(text, "  段内扣下过 {kind}：{count} 个");
        }
        for kind in HistoryOperationKind::ALL {
            let _ = writeln!(
                text,
                "  落在 {kind:?} 里：{} 个",
                self.crash_points_by_operation_kind
                    .get(&kind)
                    .copied()
                    .unwrap_or(0)
            );
        }
        let _ = writeln!(
            text,
            "恢复：读回文件 {} 次、没有文件 {} 次、失败 {} 次；施加过 journal 记录前缀 {} 次",
            self.recoveries_reading_a_file,
            self.recoveries_without_a_file,
            self.recoveries_failed,
            self.recoveries_that_applied_journal_records
        );
        let _ = writeln!(
            text,
            "崩溃状态上 checker 跑了 {} 次；记录核对器跑了 {} 次：根在而记录一条都不在 {} 次、恢复自称新态而单元缺席 {} 次",
            self.checker_runs,
            self.record_checks,
            self.record_root_without_record,
            self.record_claimed_state_missing_unit
        );
        let _ = writeln!(
            text,
            "问模型 {} 次：比过内容 {} 次、树表 0 条对上 {} 次",
            self.model_judgements,
            self.model_contents_compared,
            self.model_versions_without_a_file_matched
        );
        let _ = writeln!(
            text,
            "崩溃状态：以已知红收尾 {:?}、新发现 {}",
            self.crash_states_ending_known_red, self.crash_states_ending_new_finding
        );
        for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
            let _ = writeln!(
                text,
                "  {invariant}：判绿 {} 次、不适用 {} 次",
                self.invariant_holds.get(invariant).copied().unwrap_or(0),
                self.invariant_not_applicable
                    .get(invariant)
                    .copied()
                    .unwrap_or(0)
            );
        }
        text
    }
}

/// 一段历史注入崩溃之后交回的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryCrashInjection {
    pub seed: HistorySeed,
    /// 摆出来的崩溃状态，按段号从小到大。
    pub crash_points: Vec<CrashPoint>,
    pub tally: CrashInjectionTally,
    pub known_red_hits: Vec<KnownRedAtACrashPoint>,
    /// 按段号从小到大排；同一段历史里同一个签名只留第一个。
    pub new_findings: Vec<CrashPointFinding>,
}

/// 一段历史跑到哪一步、录制流到那时有多长：崩溃状态只摆在这段区间里。
struct StreamMarks {
    /// mkfs 占了流开头的几步（崩溃状态从它之后起）。
    after_make_filesystem: usize,
    /// 最后一个跑完的操作之后流里有几步（崩溃状态到它为止；失败那一步的写不摆）。
    after_the_last_finished_step: usize,
    /// 每一步跑完时流里有几步，按次序（崩溃状态落在哪一步靠它定）。
    after_each_step: Vec<(StepPosition, usize)>,
}

impl StreamMarks {
    /// 录制流里第 `stream_index` 步落在哪一步操作里：第一个「跑完时流里的步数」大于它的那一条登记。
    /// 登记里找不到（在最后一个跑完的操作之后）时是 None。
    fn step_containing(&self, stream_index: usize) -> Option<StepPosition> {
        self.after_each_step
            .iter()
            .find(|(_, operations_so_far)| *operations_so_far > stream_index)
            .map(|(position, _)| *position)
    }
}

/// 跑一段历史，按 `draw` 在它的录制流上摆崩溃状态，逐个重建镜像、跑恢复、池级 checker 与记录核对器、问模型。
/// `image_directory` 给了就把判红的那几个崩溃状态落成镜像文件放进去（留现场）。
///
/// # Panics
/// 录制流没开内容保留（重建镜像要字节）——这里自己建流，走不到。
#[must_use]
pub fn inject_crashes_into_history(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    image_directory: Option<&Path>,
) -> HistoryCrashInjection {
    let stream = SharedStream::retaining_contents();
    let mut committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let mut marks = StreamMarks {
        after_make_filesystem: 0,
        after_the_last_finished_step: 0,
        after_each_step: Vec::new(),
    };
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed_versions.insert(key, file);
        }
        let operations_so_far = stream.operation_count();
        marks.after_the_last_finished_step = operations_so_far;
        marks
            .after_each_step
            .push((observation.position, operations_so_far));
    });
    marks.after_make_filesystem = run.operations_written_by_make_filesystem;

    let mut tally = CrashInjectionTally {
        histories: 1,
        most_committed_versions_in_one_history: committed_versions.len(),
        ..CrashInjectionTally::default()
    };
    match &run.ending {
        HistoryEnding::Completed => tally.histories_completed = 1,
        HistoryEnding::KnownRed { form, .. } => {
            tally.histories_ended_known_red.insert(*form, 1);
        }
        HistoryEnding::NewFinding { .. } => tally.histories_ended_new_finding = 1,
    }

    let operations = stream.retained_operations();
    let parameters = execution.device_width.parameters();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &geometry);
    let step_kinds: Vec<HistoryOperationKind> = history
        .operations
        .iter()
        .map(HistoryOperation::kind)
        .collect();
    let crash_points = draw_crash_points(
        history.seed,
        &segments,
        &stream_indexes,
        &marks,
        &step_kinds,
        draw,
    );
    if crash_points.is_empty() {
        tally.histories_without_any_crash_point = 1;
    }
    let last_committed_version = committed_versions.keys().next_back().copied();

    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    let mut writes_applied_to_the_base = 0usize;
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<CrashPointFinding> = Vec::new();
    for crash_point in &crash_points {
        let segment = &segments[crash_point.segment_index];
        let first_write_of_the_segment = segment[0];
        let writes_in_the_segment = crash_point.persisted_within_the_segment.len();
        // 段号从小到大，基线只往前叠：更早的段整段持久，按写表整段施加，不为每个崩溃状态从头重建。
        base.apply_writes(&writes[writes_applied_to_the_base..first_write_of_the_segment]);
        writes_applied_to_the_base = first_write_of_the_segment;
        let writes_up_to_this_segment =
            &writes[..first_write_of_the_segment + writes_in_the_segment];
        let mut persisted = vec![false; writes.len()];
        persisted[..first_write_of_the_segment].fill(true);
        for (within_the_segment, is_persisted) in
            crash_point.persisted_within_the_segment.iter().enumerate()
        {
            persisted[first_write_of_the_segment + within_the_segment] = *is_persisted;
        }
        let image = CrashImage {
            base: &base,
            writes: &writes
                [first_write_of_the_segment..first_write_of_the_segment + writes_in_the_segment],
            persisted: crash_point.persisted_within_the_segment.clone(),
        };

        tally.crash_points += 1;
        for kind in withheld_write_kinds(&writes, segment, crash_point) {
            *tally
                .crash_points_withholding_write_kind
                .entry(kind.name())
                .or_insert(0) += 1;
        }
        match crash_point.step {
            StepPosition::StartingPoint => tally.crash_points_inside_the_starting_point += 1,
            StepPosition::Operation(_) => {}
        }
        if let Some(kind) = crash_point.operation_kind {
            *tally
                .crash_points_by_operation_kind
                .entry(kind)
                .or_insert(0) += 1;
        }
        if crash_point.withholds_a_write_before_a_persisted_one() {
            tally.crash_points_withholding_a_write_before_a_persisted_one += 1;
        }
        let newest_persisted = newest_persisted_root(&writes, &persisted)
            .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
        if newest_persisted < last_committed_version {
            tally.crash_points_that_land_before_the_last_committed_version += 1;
        }
        if some_publish_persisted_without_its_root(&writes, &persisted) {
            tally.crash_points_inside_an_unfinished_publish += 1;
        }

        let report = recover(&image, JournalPolicy::Consult);
        match &report.outcome {
            RecoveryOutcome::FileRead { .. } => tally.recoveries_reading_a_file += 1,
            RecoveryOutcome::NoFile { .. } => tally.recoveries_without_a_file += 1,
            RecoveryOutcome::Failed { .. } => tally.recoveries_failed += 1,
        }
        if report.journal.prefix_applied > 0 {
            tally.recoveries_that_applied_journal_records += 1;
        }
        let read_back = observed_read_back_after_a_crash(&report);
        tally.model_judgements += 1;
        let disagreement =
            crash_recovery_disagreement(&committed_versions, newest_persisted, &read_back);
        if disagreement.is_none() {
            match &read_back {
                ObservedReadBack::FileRead { .. } => tally.model_contents_compared += 1,
                ObservedReadBack::NoFile { .. } => {
                    tally.model_versions_without_a_file_matched += 1;
                }
                ObservedReadBack::Failed { .. } => {}
            }
        }
        // 记录核对器（层 0 那一路同一份判据）：读的是这份崩溃镜像，核的是到这一段为止的整条记录流——
        // 更早的段里发出的单元写与 journal 记录写也要算进一次发布里，只给当前段就核不出跨段的那几次发布。
        let record_check =
            check_records_against(&image, writes_up_to_this_segment, report.effective_root);
        tally.record_checks += 1;
        if record_check.root_without_record {
            tally.record_root_without_record += 1;
        }
        if record_check.claimed_state_missing_unit {
            tally.record_claimed_state_missing_unit += 1;
        }
        let violations = checker_violations_on(&image, &mut tally);
        if violations.is_empty() && disagreement.is_none() && record_check == RecordCheck::default()
        {
            continue;
        }
        let observation = crash_state_observation(
            &image,
            crash_point,
            violations,
            disagreement.clone(),
            record_check,
        );
        match classify_failure(observation) {
            HistoryEnding::Completed => {}
            HistoryEnding::KnownRed { form, .. } => {
                *tally.crash_states_ending_known_red.entry(form).or_insert(0) += 1;
                known_red_hits.push(KnownRedAtACrashPoint {
                    seed: history.seed,
                    form,
                    crash_point: crash_point.clone(),
                });
            }
            HistoryEnding::NewFinding {
                signature,
                observation,
            } => {
                tally.crash_states_ending_new_finding += 1;
                if !new_findings
                    .iter()
                    .any(|finding| finding.signature == signature)
                {
                    let image_files = image_directory.and_then(|directory| {
                        write_crash_image_files(&image, directory, history.seed, crash_point)
                    });
                    new_findings.push(CrashPointFinding {
                        signature,
                        seed: history.seed,
                        crash_point: crash_point.clone(),
                        observation,
                        read_back: format!("{read_back:?}"),
                        image_files,
                    });
                }
            }
        }
    }
    HistoryCrashInjection {
        seed: history.seed,
        crash_points,
        tally,
        known_red_hits,
        new_findings,
    }
}

/// 这个崩溃状态在段内扣下的写各是什么种类（同一种只记一次）。
fn withheld_write_kinds(
    writes: &[RetainedWrite],
    segment: &[usize],
    crash_point: &CrashPoint,
) -> Vec<StepKind> {
    let mut kinds: BTreeSet<StepKind> = BTreeSet::new();
    for (within_the_segment, is_persisted) in
        crash_point.persisted_within_the_segment.iter().enumerate()
    {
        if !is_persisted {
            kinds.insert(writes[segment[within_the_segment]].kind);
        }
    }
    kinds.into_iter().collect()
}

/// 按 `draw` 摆崩溃状态：候选段是「整段的写都落在 mkfs 之后、最后一个跑完的操作之前」的那几段，段内取真子集。
///
/// 起点那一段（取号、暖机、第一个文件）在内——2026-09-20 才放开；mkfs 那几段不在内，与层 0 同一条界：
/// mkfs 不是事务，它写到一半的盘面上还没有池，恢复报不出根是对的，拿事务的 oracle 去判只会判出假红
/// （层 0 也从 `mkfs_operation_count` 之后起枚举）。失败那一步的写也不在内（模型还没判过它写出的根）。
///
/// 抽的那一路同一段同一个子集只摆一次；摆完按段号排好——基线只往前叠。
fn draw_crash_points(
    seed: HistorySeed,
    segments: &[Vec<usize>],
    stream_indexes: &[usize],
    marks: &StreamMarks,
    step_kinds: &[HistoryOperationKind],
    draw: CrashPointDraw,
) -> Vec<CrashPoint> {
    let candidate_segments: Vec<usize> = (0..segments.len())
        .filter(|segment_index| {
            segments[*segment_index].iter().all(|write| {
                (marks.after_make_filesystem..marks.after_the_last_finished_step)
                    .contains(&stream_indexes[*write])
            })
        })
        .collect();
    if candidate_segments.is_empty() {
        return Vec::new();
    }
    let mut source = SeededRandomSource::from_seed(seed.0 ^ CRASH_POINT_SEED_SALT);
    let mut chosen: BTreeSet<(usize, Vec<bool>)> = BTreeSet::new();
    match draw {
        CrashPointDraw::Sampled {
            crash_points_per_history,
        } => {
            for _ in 0..crash_points_per_history {
                let picked = usize::try_from(
                    source.below(u64::try_from(candidate_segments.len()).expect("候选段数")),
                )
                .expect("下标装得进 usize");
                let segment_index = candidate_segments[picked];
                let subset = draw_a_proper_subset(&mut source, segments[segment_index].len());
                chosen.insert((segment_index, subset));
            }
        }
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes,
            sampled_in_longer_segments,
        } => {
            for segment_index in candidate_segments {
                let length = segments[segment_index].len();
                if length <= segment_writes {
                    for mask in 0..(1u64 << length) - 1 {
                        chosen.insert((segment_index, bits_of(mask, length)));
                    }
                } else {
                    for _ in 0..sampled_in_longer_segments {
                        let subset = draw_a_proper_subset(&mut source, length);
                        chosen.insert((segment_index, subset));
                    }
                }
            }
        }
    }
    chosen
        .into_iter()
        .map(|(segment_index, persisted_within_the_segment)| {
            let first_withheld = persisted_within_the_segment
                .iter()
                .position(|persisted| !persisted)
                .expect("段内持久的是真子集，至少有一个写被扣下");
            let step = marks
                .step_containing(stream_indexes[segments[segment_index][first_withheld]])
                .expect("崩溃状态摆在最后一个跑完的操作之前，登记里一定找得到它那一步");
            let operation_kind = match step {
                StepPosition::StartingPoint => None,
                StepPosition::Operation(step_index) => step_kinds.get(step_index).copied(),
            };
            CrashPoint {
                segment_index,
                persisted_within_the_segment,
                step,
                operation_kind,
            }
        })
        .collect()
}

/// 掩码的低 `length` 位摊成逐写的持久与否。
fn bits_of(mask: u64, length: usize) -> Vec<bool> {
    (0..length).map(|bit| mask & (1 << bit) != 0).collect()
}

/// 抽一个真子集：段短时直接抽一个掩码（[0, 2^n − 1) 不含全持久），段长时逐写各抽一次、全中了再去掉一个。
/// 真子集这一条要守住：全持久等于「没崩在这一段」，那个状态由下一段的空子集摆。
fn draw_a_proper_subset(source: &mut SeededRandomSource, segment_length: usize) -> Vec<bool> {
    if segment_length <= WRITES_A_SUBSET_MASK_HOLDS {
        let mask = source.below((1u64 << segment_length) - 1);
        return bits_of(mask, segment_length);
    }
    let mut persisted: Vec<bool> = (0..segment_length).map(|_| source.below(2) == 1).collect();
    if persisted.iter().all(|is_persisted| *is_persisted) {
        let withheld = usize::try_from(source.below(u64::try_from(segment_length).expect("段长")))
            .expect("下标装得进 usize");
        persisted[withheld] = false;
    }
    persisted
}

/// 对一份崩溃镜像跑池级 checker，记每条不变量判绿、不适用各几次，交回判红的那几条（与 `history.rs` 的 `violations_on` 同一条口径，
/// 计数进的是崩溃注入自己的那一份）。
fn checker_violations_on(
    image: &CrashImage<'_>,
    tally: &mut CrashInjectionTally,
) -> Vec<(&'static str, String)> {
    tally.checker_runs += 1;
    let mut violations = Vec::new();
    for (invariant, verdict) in check_pool_image(image) {
        match verdict {
            InvariantVerdict::Holds => *tally.invariant_holds.entry(invariant).or_insert(0) += 1,
            InvariantVerdict::NotApplicable(_) => {
                *tally.invariant_not_applicable.entry(invariant).or_insert(0) += 1;
            }
            InvariantVerdict::Violated(detail) => violations.push((invariant, detail)),
        }
    }
    violations
}

/// 这个崩溃镜像上，最新那条根带的回退下界 F 落不落在回退留下的空档里（F 那个 txg 上的根全属于被抛弃的实例）。
/// 与活盘面那一路（抬 F 之前的镜像 + 新 F）是同一个谓词、同一份实现，读的盘面不同：这里读崩溃镜像自己，
/// F 从镜像里最新那条根上取——抬 F 之后的任何一个崩溃状态都摆得出同一机理的盘面，不限于抬 F 那一步
/// （代码三方 m2-supp3-item3-code-r1 判决 K6 的假阳那一半：此前这里写死成 None，收口表第 43 行那一形在崩溃状态上恒不匹配）。
fn raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(
    image: &CrashImage<'_>,
) -> Option<bool> {
    let system_configuration = choose_system_configuration(image).ok()?;
    let newest_root = choose_root(image, &system_configuration)?;
    raised_floor_lands_only_on_abandoned_roots(image, newest_root.rollback_floor)
}

/// 一个崩溃状态的失败观察：拿去对「已知红」清单（`KNOWN_RED_FORMS`，与历史那一路同一张清单）。
fn crash_state_observation(
    image: &CrashImage<'_>,
    crash_point: &CrashPoint,
    violations: Vec<(&'static str, String)>,
    model_disagreement: Option<ModelDisagreement>,
    record_check: RecordCheck,
) -> FailureObservation {
    let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count_of(image);
    FailureObservation {
        position: crash_point.step,
        operation_kind: crash_point.operation_kind,
        violations,
        panic: None,
        newest_ring_root_txg,
        root_ring_slot_count,
        harness_judgement: None,
        model_disagreement,
        raised_floor_lands_only_on_abandoned_roots:
            raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(image),
        record_check,
    }
}

/// 把一个崩溃状态落成镜像文件（一块盘一个稀疏文件）：只写这个状态里有内容的那些扇区，没写过的留成洞、读出全 0，
/// 与内存里那份逐字节相同。写不出来（目录建不出来、盘满）时交回 None：留现场是为了好查，不是判据，不因为它红。
fn write_crash_image_files(
    image: &CrashImage<'_>,
    directory: &Path,
    seed: HistorySeed,
    crash_point: &CrashPoint,
) -> Option<PathBuf> {
    use std::os::unix::fs::FileExt as _;
    let here = directory.join(format!(
        "seed-{}-segment-{}",
        seed.0, crash_point.segment_index
    ));
    std::fs::create_dir_all(&here).ok()?;
    let sector_length = usize::try_from(SECTOR_BYTES).expect("512");
    for device in image.base.devices.keys().copied() {
        let file = std::fs::File::create(here.join(format!("device-{}.img", device.0))).ok()?;
        file.set_len(image.base.device_size_in_bytes).ok()?;
        let mut sectors: BTreeSet<u64> = image
            .base
            .devices
            .get(&device)
            .expect("这块盘在基线里")
            .written_sectors_in(DeviceOffsetInBytes(0), image.base.device_size_in_bytes)
            .into_iter()
            .collect();
        for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            let first_sector = write.offset.0 / SECTOR_BYTES;
            let sector_count = write.length_in_bytes() / SECTOR_BYTES;
            sectors.extend(first_sector..first_sector + sector_count);
        }
        for sector in sectors {
            let offset = DeviceOffsetInBytes(sector * SECTOR_BYTES);
            let bytes = PoolReader::read(image, device, offset, sector_length)?;
            file.write_all_at(&bytes, offset.0).ok()?;
        }
    }
    Some(here)
}

/// 一批种子跑下来的崩溃注入报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashInjectionReport {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub draw: CrashPointDraw,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub tally: CrashInjectionTally,
    pub known_red_hits: Vec<KnownRedAtACrashPoint>,
    /// 按种子从小到大排；同一个签名只留第一个种子。
    pub new_findings: Vec<CrashPointFinding>,
    /// 这一批判红的崩溃镜像留在哪；跑完删掉的那一档是 None。
    pub image_directory: Option<PathBuf>,
    pub image_retention: FailureImageRetention,
}

impl CrashInjectionReport {
    /// 第一行就是这次用的种子基与规模：种子基是这个测试周期写死的那一个（[`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`]），
    /// 报告与失败信息里不带它就重放不了，下一个周期重抽之后这一批数也就换了（用户 2026-09-20 定案第 7 条）。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "种子基 {}（这个测试周期写死的；同一个构建加同一个种子基重放得出来），种子 [{}, {})，每段 {} 步，崩溃状态怎么摆 {:?}，比重：{}；{}",
            self.first_seed,
            self.first_seed,
            self.first_seed + self.seed_count,
            self.operations_per_history,
            self.draw,
            self.weights.name,
            self.execution.name()
        );
        match (&self.image_directory, self.image_retention) {
            (Some(directory), FailureImageRetention::KeepTheImageFiles) => {
                let _ = writeln!(text, "判红的崩溃镜像留在 {}", directory.display());
            }
            (Some(directory), FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes) => {
                let _ = writeln!(
                    text,
                    "判红的崩溃镜像曾写在 {}，跑完已删",
                    directory.display()
                );
            }
            (None, FailureImageRetention::KeepTheImageFiles)
            | (None, FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes) => {}
        }
        text.push_str(&self.tally.render());
        for (form_index, form) in KNOWN_RED_FORMS.iter().enumerate() {
            let hits: Vec<(u64, usize)> = self
                .known_red_hits
                .iter()
                .filter(|hit| hit.form == form_index)
                .map(|hit| (hit.seed.0, hit.crash_point.segment_index))
                .collect();
            let _ = writeln!(
                text,
                "崩溃状态上的已知红第 {form_index} 条（{}）：{} 个崩溃状态；前几个 (种子, 段号) {:?}",
                form.closeout_table_row,
                hits.len(),
                hits.iter().take(8).collect::<Vec<_>>()
            );
        }
        for finding in &self.new_findings {
            text.push_str(&finding.render());
        }
        text
    }
}

/// 一次崩溃注入跑什么：种子区间、每段几步、怎么摆崩溃状态、比重、历史怎么跑、线程数、判红时镜像留不留。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrashInjectionCampaign {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub draw: CrashPointDraw,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub worker_threads: CrashInjectionWorkerThreads,
    pub image_retention: FailureImageRetention,
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，
/// 线程按片号从小到大领片，调用线程收到一片就打一行 `CRASH_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），
/// 再按片号从小到大并计数。计数按片的次序相加、新发现按种子从小到大留第一个，所以
/// [`CrashInjectionReport::render`] 与线程数、调度次序无关（进度行是按到达次序打的，只报跑到哪了，不带判定）。
///
/// # Panics
/// 某个工作线程 panic（历史里的 panic 在执行器里接住，走到这里的是崩溃状态上恢复或 checker 的断言）；有一片领了却没交回。
#[must_use]
pub fn run_crash_injection_campaign(campaign: &CrashInjectionCampaign) -> CrashInjectionReport {
    let CrashInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        draw,
        weights,
        execution,
        worker_threads,
        image_retention,
    } = *campaign;
    let slices = seed_slices(seed_count, worker_threads.count());
    let spawned_worker_threads = worker_threads.count().min(slices.len().max(1));
    let started = Instant::now();
    let image_directory = std::env::temp_dir().join(format!(
        "singlefs-crash-injection-{}-seedbase-{first_seed}",
        std::process::id()
    ));
    println!(
        "CRASH_INJECTION_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} images={}",
        first_seed + seed_count,
        slices.len(),
        worker_threads.count(),
        worker_threads.source_name(),
        image_directory.display()
    );
    let next_slice_index = AtomicUsize::new(0);
    let (finished_slices, merged_slice_count) = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<HistoryCrashInjection>)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            let image_directory = image_directory.as_path();
            scope.spawn(move || loop {
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let injections: Vec<HistoryCrashInjection> = slice
                    .clone()
                    .map(|offset| {
                        let history = generate_history_with_weights(
                            HistorySeed(first_seed.wrapping_add(offset)),
                            operations_per_history,
                            &weights,
                        );
                        inject_crashes_into_history(
                            &history,
                            execution,
                            draw,
                            Some(image_directory),
                        )
                    })
                    .collect();
                if sender.send((slice_index, injections)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut waiting_for_earlier_slices: BTreeMap<usize, Vec<HistoryCrashInjection>> =
            BTreeMap::new();
        let mut in_order: Vec<HistoryCrashInjection> = Vec::new();
        let mut next_slice_to_merge = 0usize;
        let mut finished_histories = 0u64;
        for (finished_slice_count, (slice_index, injections)) in receiver.iter().enumerate() {
            let slice = &slices[slice_index];
            finished_histories += slice.end - slice.start;
            println!(
                "CRASH_INJECTION_PROGRESS slice={}/{} seeds=[{},{}) finished_slices={}/{} finished_histories={finished_histories}/{seed_count} elapsed_seconds={:.1}",
                slice_index + 1,
                slices.len(),
                first_seed.wrapping_add(slice.start),
                first_seed.wrapping_add(slice.end),
                finished_slice_count + 1,
                slices.len(),
                started.elapsed().as_secs_f64()
            );
            waiting_for_earlier_slices.insert(slice_index, injections);
            while let Some(ready) = waiting_for_earlier_slices.remove(&next_slice_to_merge) {
                in_order.extend(ready);
                next_slice_to_merge += 1;
            }
        }
        (in_order, next_slice_to_merge)
    });
    assert_eq!(
        merged_slice_count,
        slices.len(),
        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "CRASH_INJECTION_FINISHED seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} elapsed_seconds={:.1}",
        first_seed + seed_count,
        slices.len(),
        started.elapsed().as_secs_f64()
    );
    let mut tally = CrashInjectionTally::default();
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<CrashPointFinding> = Vec::new();
    let mut signatures_seen: BTreeSet<FailureSignature> = BTreeSet::new();
    for injection in &finished_slices {
        tally.absorb(&injection.tally);
        known_red_hits.extend(injection.known_red_hits.iter().cloned());
        for finding in &injection.new_findings {
            if signatures_seen.insert(finding.signature.clone()) {
                new_findings.push(finding.clone());
            }
        }
    }
    let image_directory_left_behind = match image_retention {
        FailureImageRetention::KeepTheImageFiles => image_directory.exists(),
        FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes => {
            let existed = image_directory.exists();
            let _ = std::fs::remove_dir_all(&image_directory);
            existed
        }
    };
    CrashInjectionReport {
        first_seed,
        seed_count,
        operations_per_history,
        draw,
        weights,
        execution,
        tally,
        known_red_hits,
        new_findings,
        image_directory: image_directory_left_behind.then_some(image_directory),
        image_retention,
    }
}

/// 把 [0, `seed_count`) 切成首尾相接的种子区间：片数取 min(种子数, 4 × 线程数)，各片长度相差至多 1。
/// 坏盘输入（增补 3 第 5 件）共用这一份：切法抄一份出来两边会分叉（`code-discipline.md`「重复要生成，不许手抄」）。
#[must_use]
pub fn seed_slices(seed_count: u64, worker_threads: usize) -> Vec<std::ops::Range<u64>> {
    if seed_count == 0 {
        return Vec::new();
    }
    let wanted = u64::try_from(worker_threads.max(1) * SLICES_PER_WORKER_THREAD).expect("片数");
    let slice_count = wanted.min(seed_count);
    (0..slice_count)
        .map(|slice_index| {
            let start = seed_count * slice_index / slice_count;
            let end = seed_count * (slice_index + 1) / slice_count;
            start..end
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{generate_history, HistoryStartingPoint};

    const FOUR_CRASH_POINTS: CrashPointDraw = CrashPointDraw::Sampled {
        crash_points_per_history: 4,
    };

    /// 线程数从环境变量取，没设取 `available_parallelism`；设了不是正整数就停，不悄悄退回单线程。
    #[test]
    fn worker_thread_count_comes_from_the_environment_variable_or_available_parallelism() {
        let four = || Ok(std::num::NonZeroUsize::new(4).expect("4"));
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                Ok("3".to_string()),
                four,
            ),
            CrashInjectionWorkerThreads::FromTheEnvironmentVariable(3)
        );
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                Err(std::env::VarError::NotPresent),
                four
            ),
            CrashInjectionWorkerThreads::FromAvailableParallelism(4)
        );
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                Err(std::env::VarError::NotPresent),
                || Err(std::io::Error::other("说不出几个核"))
            ),
            CrashInjectionWorkerThreads::AvailableParallelismUnknown
        );
        assert_eq!(
            CrashInjectionWorkerThreads::FromAvailableParallelism(4).count(),
            4
        );
        assert_eq!(
            CrashInjectionWorkerThreads::AvailableParallelismUnknown.count(),
            1
        );
    }

    /// 片是首尾相接、不重不漏的：并起来正好是 [0, 种子数)。
    #[test]
    fn seed_slices_tile_the_whole_range() {
        for (seed_count, worker_threads) in [(0u64, 4usize), (1, 4), (7, 3), (96, 16), (5, 32)] {
            let slices = seed_slices(seed_count, worker_threads);
            let covered: Vec<u64> = slices.iter().flat_map(|slice| slice.clone()).collect();
            assert_eq!(
                covered,
                (0..seed_count).collect::<Vec<u64>>(),
                "种子 {seed_count} 个、{worker_threads} 个线程"
            );
            assert!(
                slices.iter().all(|slice| slice.start < slice.end),
                "没有空片：{slices:?}"
            );
        }
    }

    /// 同一个种子摆出来的崩溃状态逐项相同；段内持久的是真子集、按段号排好。
    #[test]
    fn crash_points_are_reproducible_proper_subsets_sorted_by_segment() {
        let history = generate_history(HistorySeed(11), 12);
        let first = inject_crashes_into_history(
            &history,
            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            FOUR_CRASH_POINTS,
            None,
        );
        let second = inject_crashes_into_history(
            &history,
            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            FOUR_CRASH_POINTS,
            None,
        );
        assert_eq!(first, second, "同一个种子、同一个跑法，逐项相同");
        assert!(
            first.tally.crash_points >= 1,
            "种子 11 的 12 步里摆得出崩溃状态：{:?}",
            first.tally
        );
        assert_eq!(
            first.tally.model_judgements, first.tally.crash_points,
            "每个崩溃状态都问过模型"
        );
        assert_eq!(
            first.tally.checker_runs, first.tally.crash_points,
            "每个崩溃状态都跑过 checker"
        );
        assert_eq!(
            first.tally.record_checks, first.tally.crash_points,
            "每个崩溃状态都跑过记录核对器"
        );
        assert!(
            first.crash_points.iter().all(|crash_point| crash_point
                .persisted_within_the_segment
                .iter()
                .any(|persisted| !persisted)),
            "段内持久的是真子集：{:?}",
            first.crash_points
        );
        assert!(
            first
                .crash_points
                .windows(2)
                .all(|pair| pair[0].segment_index <= pair[1].segment_index),
            "崩溃状态按段号排好"
        );
    }

    /// 段内扣下的写四种落点都见得到，段内有洞（后发的写先持久）的状态真的摆出来了，起点那一段也摆得到。
    #[test]
    fn crash_points_withhold_every_kind_of_write_and_leave_holes_inside_a_segment() {
        let mut tally = CrashInjectionTally::default();
        for seed in 0..8u64 {
            let history = generate_history(HistorySeed(seed), 16);
            let injection = inject_crashes_into_history(
                &history,
                HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
                CrashPointDraw::Sampled {
                    crash_points_per_history: 6,
                },
                None,
            );
            tally.absorb(&injection.tally);
        }
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
                "八个种子里一次都没扣下 {}：{:?}",
                kind.name(),
                tally.crash_points_withholding_write_kind
            );
        }
        assert!(
            tally.crash_points_that_land_before_the_last_committed_version >= 1,
            "一个崩溃状态都没截回到最后一版之前"
        );
        assert!(
            tally.crash_points_withholding_a_write_before_a_persisted_one >= 1,
            "一个段内有洞的状态都没摆出来（等于还是只截前缀）：{tally:?}"
        );
        assert!(
            tally.crash_points_inside_the_starting_point >= 1,
            "起点那一段里一个崩溃状态都没摆到（起点段 2026-09-20 才放开）：{tally:?}"
        );
    }

    /// 起点那一段也在候选里：从第一个文件起的历史，崩溃状态落得到起点那一步
    /// （代码三方 m2-supp3-item3-code-r1 判决 K1-d：快档 7219 次写里 627 次此前永远抽不到）。
    #[test]
    fn crash_points_fall_inside_the_starting_point_too() {
        let history = GeneratedHistory {
            seed: HistorySeed(3),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: generate_history(HistorySeed(3), 10).operations,
        };
        let injection = inject_crashes_into_history(
            &history,
            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 2,
                sampled_in_longer_segments: 2,
            },
            None,
        );
        assert!(injection.tally.crash_points >= 1);
        assert!(
            injection
                .crash_points
                .iter()
                .any(|crash_point| crash_point.step == StepPosition::StartingPoint),
            "起点那一段里一个崩溃状态都没摆到：{:?}",
            injection
                .crash_points
                .iter()
                .map(|crash_point| crash_point.step)
                .collect::<Vec<StepPosition>>()
        );
    }
}
