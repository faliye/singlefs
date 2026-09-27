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
//!
//! 每个崩溃状态判三截（[`CrashStateStage`]；代码审阅第 2 条，用户 2026-09-27 定案「崩溃注入层补可写挂载」）：
//! 崩溃镜像本身（只读恢复、模型、checker、记录核对器）；在那份崩溃后镜像上真的起一次可写挂载（取号、写行、暖机）再发一次布之后的池
//! （checker 与记录核对器，挂载或那次发布被拒也判）；挂载途中再崩一次的镜像（二次崩溃，取号、写行、暖机三段各摆一个；只读恢复、模型、
//! checker、记录核对器）。层 0 不跑可写挂载，这两截只在这里有。后两截交给记录核对器的是整条历史录制流接上挂载那一段
//! （与之后那次发布）的录制流（[`HistoryThenWritableMountRecords`]），历史那几次发布在挂载之后的池与二次崩溃镜像上还在不在也核得到。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
};

use crate::crash::{
    check_records_against, newest_persisted_root, root_identity_written_by,
    some_publish_persisted_without_its_root, writes_and_segments_with_stream_indexes, CrashImage,
    MemoryPool, RecordCheck, RecordStreamContinuity, RetainedWrite, SparseBlockDevice,
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
    crash_recovery_disagreement, ModelDisagreement, ModelDisagreementAspect, ModelRefusalReason,
    ModelRootKey, ObservedReadBack, ObservedRefusalReason,
};
use crate::model_comparison::{
    model_root_key, observed_read_back_after_a_crash, refusal_reason_of_mount_error,
    refusal_reason_of_publish_error,
};
use crate::scenario::FIXED_WRITE_TIME_SECONDS;
use crate::segments::{FixedGeometry, StepKind};
use crate::{RecordingBlockDevice, RetainedOperation, SharedStream};

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

/// 摆挂载途中的二次崩溃的随机源，与上面两路再岔开（再按第一次崩溃的段号岔开：同一段历史上各个崩溃状态抽的不是同一串）。
const SECOND_CRASH_POINT_SEED_SALT: u64 = 0x73_65_63_6f_6e_64_63_72;

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

/// 可写挂载的三段（D23（journal 的角色与格式） 已定项 16 取号、D18（块里携带什么信息） 已定项 11 写行、D16（发布语义） 已定项 8 暖机）：
/// 挂载那一段录制流里的一次写落在哪一段。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WritableMountPhase {
    /// 取号：写行那次发布的第一个单元写或 journal 记录写之前的那几次系统配置槽写。
    Acquisition,
    /// 写行那次发布：到它的根槽写、连同紧跟着的系统配置槽轮换为止。
    RowPublish,
    /// 写行那次之后的暖机空发布（与挂载在暖机之后推的抬 F）。
    WarmUp,
}

impl WritableMountPhase {
    pub const ALL: [WritableMountPhase; 3] = [
        WritableMountPhase::Acquisition,
        WritableMountPhase::RowPublish,
        WritableMountPhase::WarmUp,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            WritableMountPhase::Acquisition => "acquisition",
            WritableMountPhase::RowPublish => "row_publish",
            WritableMountPhase::WarmUp => "warm_up",
        }
    }
}

/// 一个崩溃状态上的判定落在哪一截（代码审阅第 2 条，用户 2026-09-27 定案「崩溃注入层补可写挂载」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashStateStage {
    /// 崩溃镜像本身：只读恢复（看 journal）、问模型、池级 checker、记录核对器。
    CrashImage,
    /// 在那份崩溃后镜像上可写挂载（取号、写行、暖机）再发一次布之后的池：池级 checker、记录核对器（崩溃态镜像是那份崩溃镜像，
    /// 实现恢复后的镜像是这个池）；挂载或那次发布被拒（单元区墙之外）也记在这一截。
    AfterTheWritableMountAndOnePublish,
    /// 可写挂载途中再崩一次（二次崩溃），崩在挂载的这一段里：只读恢复、问模型、池级 checker、记录核对器（历史与挂载写出的发布都核）。
    SecondCrashInsideTheWritableMount(WritableMountPhase),
}

impl CrashStateStage {
    /// 报告与镜像目录里的名字。
    #[must_use]
    pub fn name(self) -> String {
        match self {
            CrashStateStage::CrashImage => "crash_image".to_string(),
            CrashStateStage::AfterTheWritableMountAndOnePublish => {
                "after_the_writable_mount_and_one_publish".to_string()
            }
            CrashStateStage::SecondCrashInsideTheWritableMount(phase) => {
                format!("second_crash_inside_{}", phase.name())
            }
        }
    }
}

/// 一段历史上一类崩溃状态失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointFinding {
    pub signature: FailureSignature,
    pub seed: HistorySeed,
    pub crash_point: CrashPoint,
    /// 判红的是哪一截（崩溃镜像本身、之后的可写挂载与一次发布、挂载途中的二次崩溃）。
    pub stage: CrashStateStage,
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
            "崩溃状态上的新发现 {:?}（{}）：种子 {}，{}，落在 {:?}／{:?}",
            self.signature,
            self.stage.name(),
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

/// 崩溃状态上以「已知红」收尾的一次：种子、清单第几条、哪个崩溃状态、哪一截。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownRedAtACrashPoint {
    pub seed: HistorySeed,
    pub form: usize,
    pub crash_point: CrashPoint,
    pub stage: CrashStateStage,
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
    /// 在崩溃后镜像上起的可写挂载（每个崩溃状态一次）：做成的、在单元区墙上被拒的、被别的理由拒的（后者判红）。
    pub writable_mounts_after_the_crash: u64,
    pub writable_mounts_after_the_crash_succeeded: u64,
    pub writable_mounts_after_the_crash_refused_at_the_unit_area_wall: u64,
    pub writable_mounts_after_the_crash_refused: u64,
    /// 挂载做成之后的那一次发布（带文件的一版上覆盖写、树表 0 条的一版上发第一个文件）：做成的、单元区墙、别的理由（判红）。
    pub publishes_after_the_writable_mount: u64,
    pub publishes_after_the_writable_mount_refused_at_the_unit_area_wall: u64,
    pub publishes_after_the_writable_mount_refused: u64,
    /// 挂载与那次发布之后的池上跑的池级 checker 与记录核对器。
    pub checker_runs_after_the_writable_mount: u64,
    pub record_checks_after_the_writable_mount: u64,
    /// 挂载途中的二次崩溃：摆了几个、按崩在挂载的哪一段分。
    pub second_crash_points: u64,
    pub second_crash_points_by_phase: BTreeMap<WritableMountPhase, u64>,
    /// 二次崩溃状态上：恢复的三种结局、问模型的次数、池级 checker 与记录核对器跑的次数。
    pub second_crash_recoveries_reading_a_file: u64,
    pub second_crash_recoveries_without_a_file: u64,
    pub second_crash_recoveries_failed: u64,
    pub second_crash_model_judgements: u64,
    pub second_crash_checker_runs: u64,
    pub second_crash_record_checks: u64,
    /// 每一截（崩溃镜像本身、挂载与一次发布之后、二次崩溃）判到已知红与新发现的状态数之和。
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
            writable_mounts_after_the_crash,
            writable_mounts_after_the_crash_succeeded,
            writable_mounts_after_the_crash_refused_at_the_unit_area_wall,
            writable_mounts_after_the_crash_refused,
            publishes_after_the_writable_mount,
            publishes_after_the_writable_mount_refused_at_the_unit_area_wall,
            publishes_after_the_writable_mount_refused,
            checker_runs_after_the_writable_mount,
            record_checks_after_the_writable_mount,
            second_crash_points,
            second_crash_points_by_phase,
            second_crash_recoveries_reading_a_file,
            second_crash_recoveries_without_a_file,
            second_crash_recoveries_failed,
            second_crash_model_judgements,
            second_crash_checker_runs,
            second_crash_record_checks,
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
        self.writable_mounts_after_the_crash += writable_mounts_after_the_crash;
        self.writable_mounts_after_the_crash_succeeded += writable_mounts_after_the_crash_succeeded;
        self.writable_mounts_after_the_crash_refused_at_the_unit_area_wall +=
            writable_mounts_after_the_crash_refused_at_the_unit_area_wall;
        self.writable_mounts_after_the_crash_refused += writable_mounts_after_the_crash_refused;
        self.publishes_after_the_writable_mount += publishes_after_the_writable_mount;
        self.publishes_after_the_writable_mount_refused_at_the_unit_area_wall +=
            publishes_after_the_writable_mount_refused_at_the_unit_area_wall;
        self.publishes_after_the_writable_mount_refused +=
            publishes_after_the_writable_mount_refused;
        self.checker_runs_after_the_writable_mount += checker_runs_after_the_writable_mount;
        self.record_checks_after_the_writable_mount += record_checks_after_the_writable_mount;
        self.second_crash_points += second_crash_points;
        for (phase, count) in second_crash_points_by_phase {
            *self.second_crash_points_by_phase.entry(*phase).or_insert(0) += count;
        }
        self.second_crash_recoveries_reading_a_file += second_crash_recoveries_reading_a_file;
        self.second_crash_recoveries_without_a_file += second_crash_recoveries_without_a_file;
        self.second_crash_recoveries_failed += second_crash_recoveries_failed;
        self.second_crash_model_judgements += second_crash_model_judgements;
        self.second_crash_checker_runs += second_crash_checker_runs;
        self.second_crash_record_checks += second_crash_record_checks;
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
            "崩溃后镜像上的可写挂载 {} 次：做成 {}、单元区墙上被拒 {}、别的理由被拒 {}；之后的一次发布做成 {}、单元区墙上被拒 {}、别的理由被拒 {}；之后的池上 checker 跑了 {} 次、记录核对器跑了 {} 次",
            self.writable_mounts_after_the_crash,
            self.writable_mounts_after_the_crash_succeeded,
            self.writable_mounts_after_the_crash_refused_at_the_unit_area_wall,
            self.writable_mounts_after_the_crash_refused,
            self.publishes_after_the_writable_mount,
            self.publishes_after_the_writable_mount_refused_at_the_unit_area_wall,
            self.publishes_after_the_writable_mount_refused,
            self.checker_runs_after_the_writable_mount,
            self.record_checks_after_the_writable_mount
        );
        let _ = writeln!(
            text,
            "挂载途中的二次崩溃 {} 个：恢复读回文件 {} 次、没有文件 {} 次、失败 {} 次；问模型 {} 次、checker {} 次、记录核对器 {} 次",
            self.second_crash_points,
            self.second_crash_recoveries_reading_a_file,
            self.second_crash_recoveries_without_a_file,
            self.second_crash_recoveries_failed,
            self.second_crash_model_judgements,
            self.second_crash_checker_runs,
            self.second_crash_record_checks
        );
        for phase in WritableMountPhase::ALL {
            let _ = writeln!(
                text,
                "  二次崩溃崩在挂载的 {}：{} 个",
                phase.name(),
                self.second_crash_points_by_phase
                    .get(&phase)
                    .copied()
                    .unwrap_or(0)
            );
        }
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

/// 跑一段历史，按 `draw` 在它的录制流上摆崩溃状态，逐个重建镜像、跑恢复、池级 checker 与记录核对器、问模型；
/// 再在同一份崩溃后镜像上起可写挂载、发一次布、跑 checker，挂载途中摆二次崩溃（[`CrashStateStage`] 的后两截）。
/// `image_directory` 给了就把判红的那几个崩溃状态（哪一截判红就落哪一截的镜像）落成镜像文件放进去（留现场）。
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
    let mut findings = FindingsOfOneHistory {
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
    };
    for crash_point in &crash_points {
        let segment = &segments[crash_point.segment_index];
        let first_write_of_the_segment = segment[0];
        let writes_in_the_segment = crash_point.persisted_within_the_segment.len();
        // 段号从小到大，基线只往前叠：更早的段整段持久，按写表整段施加，不为每个崩溃状态从头重建。
        base.apply_writes(&writes[writes_applied_to_the_base..first_write_of_the_segment]);
        writes_applied_to_the_base = first_write_of_the_segment;
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
        // 记录核对器（层 0 那一路同一份判据、同一种交法）：两份崩溃后镜像都是这份崩溃镜像（只读恢复不改盘），核的是这段历史的整条记录流——
        // 更早的段里发出的单元写与 journal 记录写要算进一次发布里；恢复落到由 journal 记录重建的一版时，那一版的根槽写在更晚的段里，
        // 只给到这一段为止的前缀就找不到它、读不出它的实例表，被它抛弃的发布的豁免整条落空（D23（journal 的角色与格式） 已定项 15）。
        // 持久集合与整条流逐条对应：更早的段整段持久、当前段按这个崩溃点的子集、更晚的段一个都没持久（D13（验证路线） 已定项 7 的第四样入参）。
        let record_check = check_records_against(
            &image,
            &image,
            &writes,
            &persisted,
            RecordStreamContinuity::OneRecording,
            report.effective_root,
        );
        tally.record_checks += 1;
        if record_check.root_without_record {
            tally.record_root_without_record += 1;
        }
        if record_check.claimed_state_missing_unit {
            tally.record_claimed_state_missing_unit += 1;
        }
        let violations = checker_violations_on(&image, &mut tally);
        if !(violations.is_empty()
            && disagreement.is_none()
            && record_check == RecordCheck::default())
        {
            let observation = crash_state_observation(
                &image,
                crash_point,
                violations,
                disagreement.clone(),
                record_check,
            );
            findings.settle(
                &mut tally,
                FailureAtAStage {
                    seed: history.seed,
                    crash_point,
                    stage: CrashStateStage::CrashImage,
                    observation,
                    read_back: format!("{read_back:?}"),
                    image: &image,
                },
                image_directory,
            );
        }

        // 第二截与第三截（代码审阅第 2 条）：只读恢复只读盘，取号、写行、暖机在崩溃状态上从没跑过；在同一份崩溃后镜像上
        // 真的起一次可写挂载、再发一次布、跑 checker 与记录核对器，挂载途中再崩一次。
        let crash_image_materialized = materialized_crash_image(&image);
        let content_read_back_after_the_crash = match &read_back {
            ObservedReadBack::FileRead { content, .. } => Some(Some(Rc::from(content.as_slice()))),
            ObservedReadBack::NoFile { .. } => Some(None),
            ObservedReadBack::Failed { .. } => None,
        };
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &crash_image_materialized,
            &parameters,
            execution,
            history.seed,
            crash_point,
        );
        let (mount_writes, mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(&mount_run.mount_operations, &geometry);
        let (writes_of_the_publish_after_the_mount, _publish_segments, _publish_stream_indexes) =
            writes_and_segments_with_stream_indexes(&mount_run.publish_operations, &geometry);
        let records_across_the_crash = HistoryThenWritableMountRecords::new(
            &writes,
            &persisted,
            report.effective_root,
            &mount_writes,
            &writes_of_the_publish_after_the_mount,
        );
        tally.record_checks_after_the_writable_mount += 1;
        let record_check_after_the_mount = records_across_the_crash
            .record_check_after_the_writable_mount(&image, &mount_run.pool_after);
        judge_the_writable_mount_on_the_crash_image(
            &mut tally,
            &mut findings,
            &mount_run,
            record_check_after_the_mount,
            history.seed,
            crash_point,
            image_directory,
        );
        judge_second_crashes_inside_the_writable_mount(
            &mut tally,
            &mut findings,
            SecondCrashInputs {
                crash_image_materialized: &crash_image_materialized,
                records_across_the_crash: &records_across_the_crash,
                mount_segments: &mount_segments,
                committed_versions: &committed_versions,
                content_read_back_after_the_crash,
                newest_persisted_before_the_mount: newest_persisted,
            },
            history.seed,
            crash_point,
            image_directory,
        );
    }
    HistoryCrashInjection {
        seed: history.seed,
        crash_points,
        tally,
        known_red_hits: findings.known_red_hits,
        new_findings: findings.new_findings,
    }
}

/// 一段历史上攒下的已知红与新发现。同一段历史里同一截同一个签名的新发现只留第一个（按段号从小到大）。
struct FindingsOfOneHistory {
    known_red_hits: Vec<KnownRedAtACrashPoint>,
    new_findings: Vec<CrashPointFinding>,
}

/// 一截上判红的一次：哪段历史、哪个崩溃状态、哪一截、失败观察、恢复读回了什么、判红的那份镜像（留现场用）。
struct FailureAtAStage<'judged> {
    seed: HistorySeed,
    crash_point: &'judged CrashPoint,
    stage: CrashStateStage,
    observation: FailureObservation,
    read_back: String,
    image: &'judged CrashImage<'judged>,
}

impl FindingsOfOneHistory {
    /// 对「已知红」清单（与历史那一路同一张）：清单里的照记，清单外的是新发现。
    fn settle(
        &mut self,
        tally: &mut CrashInjectionTally,
        failure: FailureAtAStage<'_>,
        image_directory: Option<&Path>,
    ) {
        let FailureAtAStage {
            seed,
            crash_point,
            stage,
            observation,
            read_back,
            image,
        } = failure;
        match classify_failure(observation) {
            HistoryEnding::Completed => {}
            HistoryEnding::KnownRed { form, .. } => {
                *tally.crash_states_ending_known_red.entry(form).or_insert(0) += 1;
                self.known_red_hits.push(KnownRedAtACrashPoint {
                    seed,
                    form,
                    crash_point: crash_point.clone(),
                    stage,
                });
            }
            HistoryEnding::NewFinding {
                signature,
                observation,
            } => {
                tally.crash_states_ending_new_finding += 1;
                if !self
                    .new_findings
                    .iter()
                    .any(|finding| finding.stage == stage && finding.signature == signature)
                {
                    let image_files = image_directory.and_then(|directory| {
                        write_crash_image_files(image, directory, seed, crash_point, stage)
                    });
                    self.new_findings.push(CrashPointFinding {
                        signature,
                        seed,
                        crash_point: crash_point.clone(),
                        stage,
                        observation,
                        read_back,
                        image_files,
                    });
                }
            }
        }
    }
}

/// 把一份崩溃镜像（基线 + 这一段里持久了的写）落成一个池：之后的可写挂载在它的副本上写。
#[must_use]
pub fn materialized_crash_image(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
        if *is_persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

/// 崩溃后镜像上那次可写挂载与之后那一次发布的结局。
#[derive(Debug)]
pub enum WritableMountOutcome {
    /// 挂载被拒：成员与它说的理由。
    MountRefused {
        member: String,
        reason: ObservedRefusalReason,
    },
    /// 挂载做成、之后那一次发布被拒。
    PublishRefused {
        member: String,
        reason: ObservedRefusalReason,
    },
    /// 挂载与那一次发布都做成了。
    Published,
}

/// 在一份崩溃后镜像上跑一次可写挂载加一次发布：交回结局、挂载那一段录制流（取号、写行、暖机与挂载推的抬 F；被拒时是拒之前写出的）、
/// 之后那一次发布的录制流、挂载与发布之后的池。
pub struct WritableMountOnTheCrashImage {
    pub outcome: WritableMountOutcome,
    pub mount_operations: Vec<RetainedOperation>,
    /// 挂载做成之后那一次发布写出的那一段录制流（挂载被拒时是空的）：`pool_after` 上有它，第二截交给记录核对器的写表也要有它。
    pub publish_operations: Vec<RetainedOperation>,
    pub pool_after: MemoryPool,
}

/// 挂载之后那一次发布写的内容：随种子与崩溃状态变，装得进一个数据单元（模型只罩一个数据单元的文件）。
fn content_of_the_publish_after_the_crash(seed: HistorySeed, crash_point: &CrashPoint) -> Vec<u8> {
    let length = 97 + crash_point.segment_index % 64;
    let fill = seed.0.to_le_bytes()[0];
    (0..length)
        .map(|index| fill ^ u8::try_from(index % 251).expect("小于 251"))
        .collect()
}

/// 在 `crash_image_materialized` 的副本上起可写挂载（建池那一份参数、那一段历史的空间准入开关），做成了就接着发一次布：
/// 挂载交回的现行版本带文件时覆盖写，树表 0 条时发第一个文件（与随机历史里这两个入口同一种调法）。
#[must_use]
pub fn mount_and_publish_once_on_the_crash_image(
    crash_image_materialized: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    execution: HistoryExecution,
    seed: HistorySeed,
    crash_point: &CrashPoint,
) -> WritableMountOnTheCrashImage {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        crash_image_materialized
            .devices
            .iter()
            .map(|(identity, sectors)| {
                let mut device = SparseBlockDevice::new(
                    crash_image_materialized.device_size_in_bytes,
                    PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
                );
                device.image = sectors.clone();
                (
                    *identity,
                    RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
                )
            })
            .collect();
    let mounted =
        mount_writable_with_space_admission(parameters, &mut devices, execution.space_admission);
    let mount_operation_count = stream.operation_count();
    let outcome = match mounted {
        Err(error) => WritableMountOutcome::MountRefused {
            member: format!("{error:?}"),
            reason: refusal_reason_of_mount_error(&error),
        },
        Ok(mounted) => {
            let content = content_of_the_publish_after_the_crash(seed, crash_point);
            let file = FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            };
            let mut allocator = mounted.allocator;
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            let published = match &mounted.current {
                PoolVersion::WithFile(current) => publish_overwrite(
                    &mut writer,
                    &mut allocator,
                    current,
                    file,
                    mounted.output.instance,
                )
                .map(|_| ()),
                PoolVersion::WithoutFile(current) => publish_first_file(
                    &mut writer,
                    &mut allocator,
                    &current.root,
                    file,
                    mounted.output.instance,
                    &current.record_bytes,
                )
                .map(|_| ()),
            };
            match published {
                Ok(()) => WritableMountOutcome::Published,
                Err(error) => WritableMountOutcome::PublishRefused {
                    member: format!("{error:?}"),
                    reason: refusal_reason_of_publish_error(&error),
                },
            }
        }
    };
    let mut mount_operations = stream.retained_operations();
    let publish_operations = mount_operations.split_off(mount_operation_count);
    let pool_after = MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner_and_operations().0.image))
            .collect(),
        device_size_in_bytes: crash_image_materialized.device_size_in_bytes,
    };
    WritableMountOnTheCrashImage {
        outcome,
        mount_operations,
        publish_operations,
        pool_after,
    }
}

/// 第二、三截交给记录核对器的记录流：整条历史录制流接上那份崩溃后镜像上可写挂载那一段录制流、再接上挂载之后那一次发布的录制流，
/// 在历史与挂载的接缝处断开（[`RecordStreamContinuity::ResumedAfterACrash`]：历史那一段只留第一次崩溃之后那条时间线上的发布）。
/// 历史那一段的持久集合是第一次崩溃那个状态的，与第一截交的同一张。只交挂载那一段时，历史那几次发布在挂载之后的池与二次崩溃镜像上
/// 还在不在没人核，二次崩溃落到由记录重建的写行那一版时它的根槽写也不在表里（B3c-1 报告第六节第 2 条、B3b 报告第二节第 4、5 条）。
/// 挂载之后那一次发布的写也要在表里：它会复用历史发布换下、已可回收的槽，第二截的池上那些单元被它合法盖掉，表里没有它就解释不了。
pub struct HistoryThenWritableMountRecords {
    /// 历史录制流的写在前，挂载那一段的写接在后面，挂载之后那一次发布的写接在最后。
    writes: Vec<RetainedWrite>,
    /// 与历史那一段逐条对应的持久集合；它的长度就是挂载那一段在 `writes` 里起头的下标。
    persisted_in_the_history: Vec<bool>,
    /// 挂载那一段有几个写（之后是那一次发布的写）。
    writes_of_the_mount: usize,
    /// 第一次崩溃之后只读恢复落到的那一版（恢复失败是 None）。
    version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
}

impl HistoryThenWritableMountRecords {
    /// # Panics
    /// `persisted_in_the_history` 与 `history_writes` 不一样长。
    #[must_use]
    pub fn new(
        history_writes: &[RetainedWrite],
        persisted_in_the_history: &[bool],
        version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
        mount_writes: &[RetainedWrite],
        writes_of_the_publish_after_the_mount: &[RetainedWrite],
    ) -> Self {
        assert_eq!(
            persisted_in_the_history.len(),
            history_writes.len(),
            "历史那一段的持久集合与历史录制流逐条对应"
        );
        Self {
            writes: history_writes
                .iter()
                .chain(mount_writes)
                .chain(writes_of_the_publish_after_the_mount)
                .cloned()
                .collect(),
            persisted_in_the_history: persisted_in_the_history.to_vec(),
            writes_of_the_mount: mount_writes.len(),
            version_landed_on_after_the_first_crash,
        }
    }

    /// 挂载那一段的写（二次崩溃按它切段、摆状态）。
    #[must_use]
    pub fn mount_writes(&self) -> &[RetainedWrite] {
        let first_write_of_the_mount = self.persisted_in_the_history.len();
        &self.writes[first_write_of_the_mount..first_write_of_the_mount + self.writes_of_the_mount]
    }

    fn continuity(&self) -> RecordStreamContinuity {
        RecordStreamContinuity::ResumedAfterACrash {
            first_write_after_the_crash: self.persisted_in_the_history.len(),
            version_landed_on_after_the_crash: self.version_landed_on_after_the_first_crash,
        }
    }

    /// 历史那一段的持久集合，接上挂载那一段的（`persisted_in_the_mount`，与挂载那一段逐条对应），
    /// 再接上挂载之后那一次发布的（`publish_after_the_mount`）。
    fn persisted_with_the_mount(
        &self,
        persisted_in_the_mount: &[bool],
        publish_after_the_mount: PublishAfterTheMountInTheState,
    ) -> Vec<bool> {
        assert_eq!(
            persisted_in_the_mount.len(),
            self.writes_of_the_mount,
            "挂载那一段的持久集合与挂载那一段的写逐条对应"
        );
        let writes_of_the_publish_after_the_mount =
            self.writes.len() - self.persisted_in_the_history.len() - self.writes_of_the_mount;
        let publish_after_the_mount_persisted = match publish_after_the_mount {
            PublishAfterTheMountInTheState::Landed => true,
            PublishAfterTheMountInTheState::NotWrittenYet => false,
        };
        self.persisted_in_the_history
            .iter()
            .chain(persisted_in_the_mount)
            .copied()
            .chain(std::iter::repeat_n(
                publish_after_the_mount_persisted,
                writes_of_the_publish_after_the_mount,
            ))
            .collect()
    }

    /// 第二截：崩溃态镜像是第一次崩溃的那份崩溃镜像（`crash_image`，判择根与前缀），实现恢复后的镜像是挂载与之后那一次发布之后的池
    /// （`pool_after_the_mount`，判在不在，D13（验证路线） 已定项 7「崩溃后镜像是两份」）；挂载那一段与之后那次发布全落了。
    /// 恢复自称的那一版取挂载与之后那次发布写出的最新那条根（写行、暖机、之后那次发布都是这个池上的版本）；一条根都没写出
    /// （取号之前就被拒）时取第一次崩溃之后恢复落到的那一版。
    #[must_use]
    pub fn record_check_after_the_writable_mount(
        &self,
        crash_image: &CrashImage<'_>,
        pool_after_the_mount: &MemoryPool,
    ) -> RecordCheck {
        let newest_root_written_after_the_crash = self.writes
            [self.persisted_in_the_history.len()..]
            .iter()
            .rev()
            .find(|write| write.kind == StepKind::RootRecordFua)
            .map(|write| {
                root_identity_written_by(write.bytes().expect("根槽 FUA 写是普通写，带着字节"))
            });
        check_records_against(
            crash_image,
            pool_after_the_mount,
            &self.writes,
            &self.persisted_with_the_mount(
                &vec![true; self.writes_of_the_mount],
                PublishAfterTheMountInTheState::Landed,
            ),
            self.continuity(),
            newest_root_written_after_the_crash.or(self.version_landed_on_after_the_first_crash),
        )
    }

    /// 第三截：两份崩溃后镜像都是二次崩溃镜像（`second_crash_image`，只读恢复不改盘）；挂载那一段的持久集合按二次崩溃
    /// （`persisted_in_the_mount`，与整条挂载流逐条对应：更早的段整段持久、当前段按子集、更晚的段一个都没持久），
    /// 挂载之后那一次发布一个都没写到。
    #[must_use]
    pub fn record_check_after_a_second_crash(
        &self,
        persisted_in_the_mount: &[bool],
        second_crash_image: &CrashImage<'_>,
        effective_root_after_the_second_crash: Option<(InstanceGeneration, CheckpointTxg)>,
    ) -> RecordCheck {
        check_records_against(
            second_crash_image,
            second_crash_image,
            &self.writes,
            &self.persisted_with_the_mount(
                persisted_in_the_mount,
                PublishAfterTheMountInTheState::NotWrittenYet,
            ),
            self.continuity(),
            effective_root_after_the_second_crash,
        )
    }
}

/// 挂载之后那一次发布的写在一个状态里落没落：第二截的池上整次落了，第三截崩在挂载途中、它一个写都还没发。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishAfterTheMountInTheState {
    Landed,
    NotWrittenYet,
}

/// 第二截：挂载或那次发布被拒（单元区墙只计数，模型在单元区墙上只答允许拒绝的区间、而这里没有崩溃点上的模型状态可判区间），
/// 做没做成都在之后的池上跑池级 checker；`record_check` 是同一个池上记录核对器的结论
/// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）。别的理由被拒按「模型说该成、实现拒了」报：
/// 模型对可写挂载与覆盖写、第一个文件从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
fn judge_the_writable_mount_on_the_crash_image(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    mount_run: &WritableMountOnTheCrashImage,
    record_check: RecordCheck,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    tally.writable_mounts_after_the_crash += 1;
    let refusal = match &mount_run.outcome {
        WritableMountOutcome::MountRefused { member, reason } => {
            if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall) {
                tally.writable_mounts_after_the_crash_refused_at_the_unit_area_wall += 1;
                None
            } else {
                tally.writable_mounts_after_the_crash_refused += 1;
                Some(format!("崩溃后镜像上的可写挂载拒了：{member}"))
            }
        }
        WritableMountOutcome::PublishRefused { member, reason } => {
            tally.writable_mounts_after_the_crash_succeeded += 1;
            if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall) {
                tally.publishes_after_the_writable_mount_refused_at_the_unit_area_wall += 1;
                None
            } else {
                tally.publishes_after_the_writable_mount_refused += 1;
                Some(format!("崩溃后可写挂载之后的那一次发布拒了：{member}"))
            }
        }
        WritableMountOutcome::Published => {
            tally.writable_mounts_after_the_crash_succeeded += 1;
            tally.publishes_after_the_writable_mount += 1;
            None
        }
    };
    tally.checker_runs_after_the_writable_mount += 1;
    let violations = checker_violations_of(&mount_run.pool_after);
    if violations.is_empty() && refusal.is_none() && record_check == RecordCheck::default() {
        return;
    }
    let model_disagreement = refusal.map(|implementation_answer| {
        ModelDisagreement::new(
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
            "崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）与之后的覆盖写或第一个文件都该成（单元区墙之外模型不许拒）"
                .to_string(),
            implementation_answer,
        )
    });
    let image = CrashImage {
        base: &mount_run.pool_after,
        writes: &[],
        persisted: Vec::new(),
    };
    let observation = crash_state_observation(
        &image,
        crash_point,
        violations,
        model_disagreement,
        record_check,
    );
    findings.settle(
        tally,
        FailureAtAStage {
            seed,
            crash_point,
            stage: CrashStateStage::AfterTheWritableMountAndOnePublish,
            observation,
            read_back: format!("{:?}", mount_run.outcome),
            image: &image,
        },
        image_directory,
    );
}

/// 挂载那一段录制流里每一次写落在挂载的哪一段：先是取号的系统配置槽写，第一个别的写起是写行那次发布，
/// 它的根槽写与紧跟着的系统配置槽轮换之后的第一个别的写起是暖机。
fn writable_mount_phase_of_each_write(writes: &[RetainedWrite]) -> Vec<WritableMountPhase> {
    let mut phase = WritableMountPhase::Acquisition;
    let mut row_publish_root_written = false;
    writes
        .iter()
        .map(|write| {
            phase = match (phase, write.kind) {
                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => {
                    WritableMountPhase::Acquisition
                }
                (
                    WritableMountPhase::Acquisition,
                    StepKind::UnitWrite
                    | StepKind::JournalRecord
                    | StepKind::RootRecordFua
                    | StepKind::ZeroFill
                    | StepKind::Barrier,
                ) => WritableMountPhase::RowPublish,
                (WritableMountPhase::RowPublish, StepKind::SystemConfigurationSlot) => {
                    WritableMountPhase::RowPublish
                }
                (
                    WritableMountPhase::RowPublish,
                    StepKind::UnitWrite
                    | StepKind::JournalRecord
                    | StepKind::RootRecordFua
                    | StepKind::ZeroFill
                    | StepKind::Barrier,
                ) => {
                    if row_publish_root_written {
                        WritableMountPhase::WarmUp
                    } else {
                        WritableMountPhase::RowPublish
                    }
                }
                (
                    WritableMountPhase::WarmUp,
                    StepKind::SystemConfigurationSlot
                    | StepKind::UnitWrite
                    | StepKind::JournalRecord
                    | StepKind::RootRecordFua
                    | StepKind::ZeroFill
                    | StepKind::Barrier,
                ) => WritableMountPhase::WarmUp,
            };
            if phase == WritableMountPhase::RowPublish && write.kind == StepKind::RootRecordFua {
                row_publish_root_written = true;
            }
            phase
        })
        .collect()
}

/// 挂载途中的一次二次崩溃：挂载那一段的第几段、段内哪几个写持久了、崩在挂载的哪一段。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SecondCrashPoint {
    segment_index: usize,
    persisted_within_the_segment: Vec<bool>,
    phase: WritableMountPhase,
}

/// 挂载的每一段各摆一个二次崩溃：在那一段的写里按种子挑一个 w，崩在 w 上——w 所在那一段里 w 之前的写都落了、w 没落、
/// w 之后的写各按种子抽；更早的段整段持久，更晚的段一个都没持久（与第一次崩溃同一个枚举域）。挂载没写出某一段的写就不摆那一段。
fn draw_second_crash_points(
    seed: HistorySeed,
    crash_point: &CrashPoint,
    segments: &[Vec<usize>],
    phase_of_each_write: &[WritableMountPhase],
) -> Vec<SecondCrashPoint> {
    let mut source = SeededRandomSource::from_seed(
        seed.0
            ^ SECOND_CRASH_POINT_SEED_SALT
            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64"),
    );
    let mut chosen: BTreeSet<SecondCrashPoint> = BTreeSet::new();
    for phase in WritableMountPhase::ALL {
        let writes_of_the_phase: Vec<(usize, usize)> = segments
            .iter()
            .enumerate()
            .flat_map(|(segment_index, segment)| {
                segment
                    .iter()
                    .enumerate()
                    .filter(|(_, write_index)| phase_of_each_write[**write_index] == phase)
                    .map(move |(within_the_segment, _)| (segment_index, within_the_segment))
            })
            .collect();
        if writes_of_the_phase.is_empty() {
            continue;
        }
        let picked = usize::try_from(
            source.below(u64::try_from(writes_of_the_phase.len()).expect("写数装得进 u64")),
        )
        .expect("下标装得进 usize");
        let (segment_index, withheld) = writes_of_the_phase[picked];
        let persisted_within_the_segment: Vec<bool> = (0..segments[segment_index].len())
            .map(
                |within_the_segment| match within_the_segment.cmp(&withheld) {
                    std::cmp::Ordering::Less => true,
                    std::cmp::Ordering::Equal => false,
                    std::cmp::Ordering::Greater => source.below(2) == 1,
                },
            )
            .collect();
        chosen.insert(SecondCrashPoint {
            segment_index,
            persisted_within_the_segment,
            phase,
        });
    }
    chosen.into_iter().collect()
}

/// 摆二次崩溃要的东西：第一次崩溃之后那份镜像、整条历史录制流接上挂载那一段录制流（记录核对器核的就是它，挂载那一段的写表也从它取）
/// 与挂载那一段的切段、模型提交过的每一版、第一次崩溃之后读回的内容（恢复失败时 None）、第一次崩溃的盘上已持久的最新根槽。
struct SecondCrashInputs<'history> {
    crash_image_materialized: &'history MemoryPool,
    records_across_the_crash: &'history HistoryThenWritableMountRecords,
    mount_segments: &'history [Vec<usize>],
    committed_versions: &'history BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    content_read_back_after_the_crash: Option<Option<Rc<[u8]>>>,
    newest_persisted_before_the_mount: Option<ModelRootKey>,
}

/// 第三截：挂载途中的二次崩溃。每个二次崩溃状态上：只读恢复（看 journal）、问模型——允许的版本是模型提交过的每一版，加上这次挂载写出的根
/// （写行那次与暖机照抄第一次崩溃之后恢复到的那一版的文件，所以内容取那一次读回的；那一次恢复失败时一条都不加）；
/// 盘上已持久的最新根槽取第一次崩溃与挂载写表两边里更新的那条——、池级 checker、记录核对器。
/// 记录核对器核的是整条历史录制流接上整条挂载流（[`HistoryThenWritableMountRecords::record_check_after_a_second_crash`]）：
/// 历史那几次发布在二次崩溃镜像上还在不在也核；恢复落到由记录重建的写行那一版时，它的根槽写在更晚的段里，也在表里。
fn judge_second_crashes_inside_the_writable_mount(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    inputs: SecondCrashInputs<'_>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    let SecondCrashInputs {
        crash_image_materialized,
        records_across_the_crash,
        mount_segments,
        committed_versions,
        content_read_back_after_the_crash,
        newest_persisted_before_the_mount,
    } = inputs;
    let mount_writes = records_across_the_crash.mount_writes();
    let phase_of_each_write = writable_mount_phase_of_each_write(mount_writes);
    let second_crash_points =
        draw_second_crash_points(seed, crash_point, mount_segments, &phase_of_each_write);
    let mut versions_allowed = committed_versions.clone();
    if let Some(content) = &content_read_back_after_the_crash {
        for write in mount_writes {
            if write.kind != StepKind::RootRecordFua {
                continue;
            }
            let (instance, checkpoint_txg) =
                root_identity_written_by(write.bytes().expect("根槽 FUA 写是普通写，带着字节"));
            versions_allowed.insert(model_root_key(instance, checkpoint_txg), content.clone());
        }
    }
    let mut second_base = crash_image_materialized.clone();
    let mut writes_applied_to_the_second_base = 0usize;
    let mut ordered = second_crash_points;
    ordered.sort_by_key(|second| second.segment_index);
    for second in &ordered {
        let segment = &mount_segments[second.segment_index];
        let first_write_of_the_segment = segment[0];
        let writes_in_the_segment = second.persisted_within_the_segment.len();
        second_base.apply_writes(
            &mount_writes[writes_applied_to_the_second_base..first_write_of_the_segment],
        );
        writes_applied_to_the_second_base = first_write_of_the_segment;
        // 与整条挂载流逐条对应：更早的段整段持久、当前段按二次崩溃的子集、更晚的段一个都没持久。
        let mut persisted_in_the_mount = vec![false; mount_writes.len()];
        persisted_in_the_mount[..first_write_of_the_segment].fill(true);
        persisted_in_the_mount
            [first_write_of_the_segment..first_write_of_the_segment + writes_in_the_segment]
            .copy_from_slice(&second.persisted_within_the_segment);
        let image = CrashImage {
            base: &second_base,
            writes: &mount_writes
                [first_write_of_the_segment..first_write_of_the_segment + writes_in_the_segment],
            persisted: second.persisted_within_the_segment.clone(),
        };
        tally.second_crash_points += 1;
        *tally
            .second_crash_points_by_phase
            .entry(second.phase)
            .or_insert(0) += 1;
        let report = recover(&image, JournalPolicy::Consult);
        match &report.outcome {
            RecoveryOutcome::FileRead { .. } => tally.second_crash_recoveries_reading_a_file += 1,
            RecoveryOutcome::NoFile { .. } => tally.second_crash_recoveries_without_a_file += 1,
            RecoveryOutcome::Failed { .. } => tally.second_crash_recoveries_failed += 1,
        }
        let read_back_after_the_second_crash = observed_read_back_after_a_crash(&report);
        let newest_persisted_by_the_mount =
            newest_persisted_root(mount_writes, &persisted_in_the_mount)
                .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
        let newest_persisted = newest_persisted_before_the_mount.max(newest_persisted_by_the_mount);
        tally.second_crash_model_judgements += 1;
        let disagreement = crash_recovery_disagreement(
            &versions_allowed,
            newest_persisted,
            &read_back_after_the_second_crash,
        );
        tally.second_crash_record_checks += 1;
        let record_check = records_across_the_crash.record_check_after_a_second_crash(
            &persisted_in_the_mount,
            &image,
            report.effective_root,
        );
        tally.second_crash_checker_runs += 1;
        let violations = checker_violations_of(&image);
        if violations.is_empty() && disagreement.is_none() && record_check == RecordCheck::default()
        {
            continue;
        }
        let observation =
            crash_state_observation(&image, crash_point, violations, disagreement, record_check);
        findings.settle(
            tally,
            FailureAtAStage {
                seed,
                crash_point,
                stage: CrashStateStage::SecondCrashInsideTheWritableMount(second.phase),
                observation,
                read_back: format!("{read_back_after_the_second_crash:?}"),
                image: &image,
            },
            image_directory,
        );
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

/// 对挂载之后的池、二次崩溃的镜像跑池级 checker，交回判红的那几条（判绿、不适用的计数只记第一截那一份：
/// 那两截的 checker 次数另记在 `checker_runs_after_the_writable_mount` 与 `second_crash_checker_runs`）。
fn checker_violations_of(image: &dyn ImageReader) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
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
    stage: CrashStateStage,
) -> Option<PathBuf> {
    use std::os::unix::fs::FileExt as _;
    let here = directory.join(format!(
        "seed-{}-segment-{}-{}",
        seed.0,
        crash_point.segment_index,
        stage.name()
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
    /// 按种子从小到大排；同一截同一个签名只留第一个种子。
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
            let hits: Vec<(u64, usize, String)> = self
                .known_red_hits
                .iter()
                .filter(|hit| hit.form == form_index)
                .map(|hit| (hit.seed.0, hit.crash_point.segment_index, hit.stage.name()))
                .collect();
            let _ = writeln!(
                text,
                "崩溃状态上的已知红第 {form_index} 条（{}）：{} 个崩溃状态；前几个 (种子, 段号, 哪一截) {:?}",
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
    let mut signatures_seen: BTreeSet<(CrashStateStage, FailureSignature)> = BTreeSet::new();
    for injection in &finished_slices {
        tally.absorb(&injection.tally);
        known_red_hits.extend(injection.known_red_hits.iter().cloned());
        for finding in &injection.new_findings {
            if signatures_seen.insert((finding.stage, finding.signature.clone())) {
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
