//! 崩溃注入（里程碑「覆盖写、释放、回退与复用」增补 3 第 3 件）：第 1 件的每段历史跑的过程中开着内容保留的录制流，把流按屏障与 FUA 写切段，
//! 按种子摆若干个崩溃状态（一段 + 那一段里持久了哪几个写），对每个崩溃状态重建镜像、跑恢复、跑池级 checker、跑记录核对器，
//! 并问理想模型「恢复到的这一版允不允许」（`singlefs_harness::model::crash_recovery_disagreement`）。
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
//! 每个二次崩溃镜像上再起一次可写挂载，判 I-8.4（重放幂等）的 R-换：「恢复加写行那一整段」做两次（第一次崩在半途）与做一次，
//! 读出的内容相同（[`compare_content_read_once_and_twice`]；不比扇区、实例代号、实例表行）。
//!
//! 挂载透明（独立的第四条，判法在 `singlefs_harness::mount_transparency`；r5 判决第二节 U5、U6，用户 2026-10-02 定 Y7「崩溃注入里量」、F6「加二次」）：
//! 挂载做成的每个崩溃状态上，崩溃镜像的只读恢复（R1）与只挂载之后、挂载加一次发布之后的只读恢复（R2，后者比发布之前那一版）逐字段比，
//! 挂载途中每个二次崩溃镜像的只读恢复也与 R1 比；可写挂载被拒的，理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的判拒绝本身合不合法。

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
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
    RecoveryReport,
};
use singlefs_core::transaction::{
    publish_first_file, publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
};
use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};

use crate::crash::check_records_against;
use singlefs_harness::history::{
    choice_from_reproduction_environment, classify_failure, execute_history_with,
    generate_history_with_weights, newest_ring_root_and_slot_count_of,
    raised_floor_lands_only_on_abandoned_roots, record_check_aspects,
    render_environment_assignments, FailureObservation, FailureSignature, GeneratedHistory,
    GenerationWeights, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
    HistorySampling, HistorySeed, ReproductionVariablePrefix, SeededRandomSource, StepPosition,
    KNOWN_RED_FORMS,
};
use singlefs_harness::memory_pool::{
    newest_persisted_root, root_identity_written_by, some_publish_persisted_without_its_root,
    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, RecordCheck,
    RecordStreamContinuity, RetainedWrite, SparseBlockDevice, SECTOR_BYTES,
};
use singlefs_harness::model::{
    crash_recovery_disagreement, ModelDisagreement, ModelDisagreementAspect, ModelRefusalReason,
    ModelRootKey, ObservedReadBack, ObservedRefusalReason,
};
use singlefs_harness::model_comparison::{
    model_root_key, observed_read_back_after_a_crash, refusal_reason_of_mount_error,
    refusal_reason_of_publish_error,
};
use singlefs_harness::mount_transparency::{
    judge_mount_transparency, judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields,
    ComparedVersion, DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount,
    FloorRaisesInTheMountWrites, MountTransparency, MountTransparencyAction,
    ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage, RecoveryFieldsRead,
    RefusalConditionNotMet, RefusalReasonOnTheD23List,
    UserVisibleTreeRootPointersOfTheContinuation, WritableMountRefusalVerdict,
};
use singlefs_harness::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_harness::reference_filesystem_translation::{
    judge_a_second_crash_after_the_restart, JointVerdictOfTheReferenceFileSystemAndTheModel,
    ReferenceFileSystemJudgementOfACrash, ReferenceFileSystemTranslation, ReferenceFileSystemVote,
};
use singlefs_harness::scenario::FIXED_WRITE_TIME_SECONDS;
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

/// 崩溃注入的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。
pub const CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_INJECTION_THREADS";

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
            process_environment_variable(variable_name).as_deref(),
            std::thread::available_parallelism,
        )
    }

    /// [`Self::from_the_environment_variable_named`] 的判定本身：环境变量的值（`None` 是没设）、`available_parallelism` 报什么
    /// 都由调用方给（用例不改进程的环境变量）。解析经数值环境变量共用的那一份，线程数取 `NonZeroUsize`：
    /// 写错、写 0 的口径与仓里别的线程数变量相同。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串）：消息带变量名与原值。
    #[must_use]
    fn from_the_environment_value(
        variable_name: &str,
        value: Option<&str>,
        available_parallelism: impl Fn() -> std::io::Result<std::num::NonZeroUsize>,
    ) -> Self {
        match number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
            variable_name,
            value,
        ) {
            Some(count) => Self::FromTheEnvironmentVariable(count.get()),
            None => match available_parallelism() {
                Ok(count) => Self::FromAvailableParallelism(count.get()),
                Err(_) => Self::AvailableParallelismUnknown,
            },
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
    /// 就摆这一个崩溃状态（靶向用例用，代码三方 m2-supp3-item3-code-r2 判决第四节第 1 题，用户 2026-09-28 选「靶向用例 + 变异行」）：
    /// 从第 `segment_index` 段的第一个写起数 `writes_from_the_segment_on` 个写，其中下标（从这一段的第一个写数起）在
    /// `withheld_among_them` 里的扣下、别的持久；更早的段整段持久，这几个写之后的一个都没持久。
    ///
    /// 这几个写可以越过这一段的末尾，越过的那几道屏障当设备没守：记录核对器两条判据防的正是屏障没守住才有的状态
    /// （记录与根之间那道没守：根在而记录不在；单元与根之间的没守：恢复自称的那一版之前的发布缺单元），
    /// 今天的持久顺序下它们在段内真子集里一个都摆不出来，判据接没接上只有这样摆得出来看（K3-④⑨）。
    ///
    /// 只给写死的一段历史经 [`inject_crashes_into_history`] 用，不进战役：写死的段号只对写死的那一段历史成立，重放行也写不出它
    /// （[`run_crash_injection_campaign`] 拿到它就 panic）。段号不是候选段、这几个写越过最后一个跑完的操作（或写表末尾）、
    /// `writes_from_the_segment_on` 是 0、扣下的下标越界或一个都没有时 panic——写死的输入写错了，不悄悄摆别的状态。
    ExactlyOneCrashState {
        segment_index: usize,
        writes_from_the_segment_on: usize,
        withheld_among_them: &'static [usize],
    },
}

/// 靶向用例那一支（[`CrashPointDraw::ExactlyOneCrashState`]）拿去跑战役或写重放行时的 panic 消息。
const A_TARGETED_CRASH_STATE_IS_NOT_A_CAMPAIGN_DRAW: &str =
    "靶向用例那一个崩溃状态不进战役：写死的段号只对写死的那一段历史成立，重放行写不出它（只经 inject_crashes_into_history 摆）";

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
/// 靶向用例（[`CrashPointDraw::ExactlyOneCrashState`]）摆的那一个例外：持久集合从这一段的第一个写起，可以越过这一段的末尾。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrashPoint {
    pub segment_index: usize,
    /// 这一段里第 k 个写持久了没有（长度就是这一段的写数；靶向用例摆的那一个，长度是它从这一段的第一个写起数的写数）。
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

/// 可写挂载的四段（D23（journal 的角色与格式） 已定项 16 取号、D18（块里携带什么信息） 已定项 11 写行、D16（发布语义） 已定项 8 暖机、
/// D8（核心索引结构） 已定项 16 第 6 条挂载续做）：挂载那一段录制流里的一次写落在哪一段。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WritableMountPhase {
    /// 取号：写行那次发布的第一个单元写或 journal 记录写之前的那几次系统配置槽写。
    Acquisition,
    /// 写行那次发布：到它的根槽写、连同紧跟着的系统配置槽轮换为止。
    RowPublish,
    /// 写行那次之后的暖机空发布（与挂载在暖机之后推的抬 F：摆二次崩溃时两者算一段，F 那一样按抬 F 的写有没有持久分开判，
    /// [`FloorRaisesInTheMountWrites::at_a_second_crash`]）。
    WarmUp,
    /// 挂载续做：暖机与推的抬 F 之后、交回之前把所选那一版的 `deleted_inodes` 走空的那几步删除发布（所选那一版有删除中的文件时才有）。
    Continuation,
}

impl WritableMountPhase {
    pub const ALL: [WritableMountPhase; 4] = [
        WritableMountPhase::Acquisition,
        WritableMountPhase::RowPublish,
        WritableMountPhase::WarmUp,
        WritableMountPhase::Continuation,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            WritableMountPhase::Acquisition => "acquisition",
            WritableMountPhase::RowPublish => "row_publish",
            WritableMountPhase::WarmUp => "warm_up",
            WritableMountPhase::Continuation => "continuation",
        }
    }
}

/// 第一次崩溃镜像上挂载透明比的是哪个动作之后的池（`singlefs_harness::mount_transparency::MountTransparencyAction` 的前两种）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MountTransparencyComparison {
    /// 只挂载（取号、写行、暖机与挂载推的抬 F），不接发布：v2a。
    TheMountAlone,
    /// 挂载加一次发布，比发布之前那一版。
    TheMountAndOnePublish,
}

impl MountTransparencyComparison {
    pub const ALL: [MountTransparencyComparison; 2] = [
        MountTransparencyComparison::TheMountAlone,
        MountTransparencyComparison::TheMountAndOnePublish,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            MountTransparencyComparison::TheMountAlone => "the_mount_alone",
            MountTransparencyComparison::TheMountAndOnePublish => "the_mount_and_one_publish",
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
    /// 二次崩溃镜像上再起一次可写挂载之后的池：读出的内容要与没崩过的那一次挂载之后读出的相同（I-8.4 的 R-换）。
    RemountAfterASecondCrashInsideTheWritableMount(WritableMountPhase),
    /// 挂载透明：崩溃镜像上做了这个动作之后的池，只读恢复与崩溃镜像上的只读恢复逐字段比。二次崩溃镜像上的那一判记在
    /// [`CrashStateStage::SecondCrashInsideTheWritableMount`] 那一截的违例里。
    MountTransparencyAfterTheCrash(MountTransparencyComparison),
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
            CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(phase) => {
                format!("remount_after_a_second_crash_inside_{}", phase.name())
            }
            CrashStateStage::MountTransparencyAfterTheCrash(comparison) => {
                format!("mount_transparency_of_{}", comparison.name())
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
        text
    }
}

/// RefFS 判的两截（G1 用户定案的调用点：第一截与二次崩溃那一截；挂载加一次发布之后那一截不判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StageTheReferenceFileSystemJudges {
    CrashImage,
    SecondCrashInsideTheWritableMount,
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
    /// 二次崩溃状态上判 I-8.4 的次数（每个二次崩溃状态一次，在它上面再起一次可写挂载）：判了的按崩在挂载的哪一段分、
    /// 读出的内容与没崩过的那一次相同的、不同的（判红），没判的按为什么没判分。
    pub remounts_after_a_second_crash: u64,
    pub remount_idempotence_judged_by_phase: BTreeMap<WritableMountPhase, u64>,
    pub remounts_after_a_second_crash_reading_the_same_content: u64,
    pub remounts_after_a_second_crash_reading_different_content: u64,
    pub remount_idempotence_not_judged: BTreeMap<RemountIdempotenceNotJudged, u64>,
    /// 二次崩溃之后再挂被拒、拒因是判据 N-配置 的（第三截判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那一条）：判过的、
    /// 崩溃态没注入读故障而判据在二次崩溃镜像上复算仍为真、拒得不合法的（判红）。
    pub remount_refusals_for_a_newer_publish_witnessed_by_system_configuration: u64,
    pub remount_refusals_with_the_witness_criterion_true_without_read_faults: u64,
    /// 每一截（崩溃镜像本身、挂载与一次发布之后、二次崩溃）判到已知红与新发现的状态数之和。
    pub crash_states_ending_known_red: BTreeMap<usize, u64>,
    pub crash_states_ending_new_finding: u64,
    /// 挂载透明：第一次崩溃镜像上按动作判了几次、二次崩溃镜像上按崩在挂载的哪一段判了几次；整个动作没判的按理由分；
    /// 判红的按字段分（名字里带「量过」或「推的」，提交时的快档一眼分得开）；某一样没判的按理由分。
    pub mount_transparency_judged: BTreeMap<MountTransparencyComparison, u64>,
    pub mount_transparency_judged_on_second_crashes_by_phase: BTreeMap<WritableMountPhase, u64>,
    pub mount_transparency_not_judged: BTreeMap<&'static str, u64>,
    pub mount_transparency_violations_by_field: BTreeMap<&'static str, u64>,
    pub mount_transparency_fields_not_judged: BTreeMap<&'static str, u64>,
    /// 崩溃后镜像上的可写挂载被拒、理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的：拒得合法的、不合法的（判红）。
    pub writable_mount_refusals_on_the_d23_list_legal: u64,
    pub writable_mount_refusals_on_the_d23_list_illegal: u64,
    /// 崩溃窗口末尾之前历史里有一步在注入的读故障下发过东西的崩溃状态：第二截判拒可写时不判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）那一条。
    pub crash_points_after_a_step_reading_under_injected_faults: u64,
    /// 第一截与二次崩溃那一截上 RefFS 与 O1 判官两票合起来（G1 用户定案：任一红即红），按哪一边红分开计；
    /// RefFS 弃权的按理由另计（两截并在一起）。
    pub reference_filesystem_and_model_verdicts_on_crash_images:
        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
    pub reference_filesystem_and_model_verdicts_on_second_crashes:
        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
    pub reference_filesystem_abstentions: BTreeMap<&'static str, u64>,
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
            remounts_after_a_second_crash,
            remount_idempotence_judged_by_phase,
            remounts_after_a_second_crash_reading_the_same_content,
            remounts_after_a_second_crash_reading_different_content,
            remount_idempotence_not_judged,
            remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
            remount_refusals_with_the_witness_criterion_true_without_read_faults,
            crash_states_ending_known_red,
            crash_states_ending_new_finding,
            mount_transparency_judged,
            mount_transparency_judged_on_second_crashes_by_phase,
            mount_transparency_not_judged,
            mount_transparency_violations_by_field,
            mount_transparency_fields_not_judged,
            writable_mount_refusals_on_the_d23_list_legal,
            writable_mount_refusals_on_the_d23_list_illegal,
            crash_points_after_a_step_reading_under_injected_faults,
            reference_filesystem_and_model_verdicts_on_crash_images,
            reference_filesystem_and_model_verdicts_on_second_crashes,
            reference_filesystem_abstentions,
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
        self.remounts_after_a_second_crash += remounts_after_a_second_crash;
        for (phase, count) in remount_idempotence_judged_by_phase {
            *self
                .remount_idempotence_judged_by_phase
                .entry(*phase)
                .or_insert(0) += count;
        }
        self.remounts_after_a_second_crash_reading_the_same_content +=
            remounts_after_a_second_crash_reading_the_same_content;
        self.remounts_after_a_second_crash_reading_different_content +=
            remounts_after_a_second_crash_reading_different_content;
        for (reason, count) in remount_idempotence_not_judged {
            *self
                .remount_idempotence_not_judged
                .entry(*reason)
                .or_insert(0) += count;
        }
        self.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration +=
            remount_refusals_for_a_newer_publish_witnessed_by_system_configuration;
        self.remount_refusals_with_the_witness_criterion_true_without_read_faults +=
            remount_refusals_with_the_witness_criterion_true_without_read_faults;
        for (form, count) in crash_states_ending_known_red {
            *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
        }
        self.crash_states_ending_new_finding += crash_states_ending_new_finding;
        for (comparison, count) in mount_transparency_judged {
            *self
                .mount_transparency_judged
                .entry(*comparison)
                .or_insert(0) += count;
        }
        for (phase, count) in mount_transparency_judged_on_second_crashes_by_phase {
            *self
                .mount_transparency_judged_on_second_crashes_by_phase
                .entry(*phase)
                .or_insert(0) += count;
        }
        for (reason, count) in mount_transparency_not_judged {
            *self
                .mount_transparency_not_judged
                .entry(reason)
                .or_insert(0) += count;
        }
        for (field, count) in mount_transparency_violations_by_field {
            *self
                .mount_transparency_violations_by_field
                .entry(field)
                .or_insert(0) += count;
        }
        for (reason, count) in mount_transparency_fields_not_judged {
            *self
                .mount_transparency_fields_not_judged
                .entry(reason)
                .or_insert(0) += count;
        }
        self.writable_mount_refusals_on_the_d23_list_legal +=
            writable_mount_refusals_on_the_d23_list_legal;
        self.writable_mount_refusals_on_the_d23_list_illegal +=
            writable_mount_refusals_on_the_d23_list_illegal;
        self.crash_points_after_a_step_reading_under_injected_faults +=
            crash_points_after_a_step_reading_under_injected_faults;
        for (verdict, count) in reference_filesystem_and_model_verdicts_on_crash_images {
            *self
                .reference_filesystem_and_model_verdicts_on_crash_images
                .entry(*verdict)
                .or_insert(0) += count;
        }
        for (verdict, count) in reference_filesystem_and_model_verdicts_on_second_crashes {
            *self
                .reference_filesystem_and_model_verdicts_on_second_crashes
                .entry(*verdict)
                .or_insert(0) += count;
        }
        for (reason, count) in reference_filesystem_abstentions {
            *self
                .reference_filesystem_abstentions
                .entry(reason)
                .or_insert(0) += count;
        }
    }

    /// 记一票：RefFS 与 O1 判官两票合起来的那一格，RefFS 弃权时连理由。
    fn note_the_reference_filesystem_vote(
        &mut self,
        stage: StageTheReferenceFileSystemJudges,
        vote: &ReferenceFileSystemVote,
        model_disagreement: Option<&ModelDisagreement>,
    ) {
        let joint = JointVerdictOfTheReferenceFileSystemAndTheModel::of(vote, model_disagreement);
        let verdicts = match stage {
            StageTheReferenceFileSystemJudges::CrashImage => {
                &mut self.reference_filesystem_and_model_verdicts_on_crash_images
            }
            StageTheReferenceFileSystemJudges::SecondCrashInsideTheWritableMount => {
                &mut self.reference_filesystem_and_model_verdicts_on_second_crashes
            }
        };
        *verdicts.entry(joint).or_insert(0) += 1;
        if let ReferenceFileSystemVote::Abstained(reason) = vote {
            *self
                .reference_filesystem_abstentions
                .entry(reason.name())
                .or_insert(0) += 1;
        }
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
            "二次崩溃之后再挂 {} 次（I-8.4 的 R-换）：读出的内容与没崩过的那一次相同 {}、不同 {}；判了的按段 {:?}；没判的 {:?}",
            self.remounts_after_a_second_crash,
            self.remounts_after_a_second_crash_reading_the_same_content,
            self.remounts_after_a_second_crash_reading_different_content,
            self.remount_idempotence_judged_by_phase
                .iter()
                .map(|(phase, count)| (phase.name(), *count))
                .collect::<Vec<_>>(),
            self.remount_idempotence_not_judged
        );
        let _ = writeln!(
            text,
            "二次崩溃之后再挂被拒、拒因是判据 N-配置 的（第三截判 C670 那一条）：{} 次，崩溃态没注入读故障而判据复算仍为真、拒得不合法 {}",
            self.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
            self.remount_refusals_with_the_witness_criterion_true_without_read_faults
        );
        let _ = writeln!(
            text,
            "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}",
            self.mount_transparency_judged
                .iter()
                .map(|(comparison, count)| (comparison.name(), *count))
                .collect::<Vec<_>>(),
            self.mount_transparency_judged_on_second_crashes_by_phase
                .iter()
                .map(|(phase, count)| (phase.name(), *count))
                .collect::<Vec<_>>(),
            self.mount_transparency_not_judged,
            self.mount_transparency_violations_by_field,
            self.mount_transparency_fields_not_judged
        );
        let _ = writeln!(
            text,
            "可写挂载被拒、理由在 D23 已定项 14 那张单子上的：拒得合法 {}、不合法 {}；崩溃窗口之前有一步带读故障、不判 C670 那一条的崩溃状态 {}",
            self.writable_mount_refusals_on_the_d23_list_legal,
            self.writable_mount_refusals_on_the_d23_list_illegal,
            self.crash_points_after_a_step_reading_under_injected_faults
        );
        let _ = writeln!(
            text,
            "RefFS 与 O1 判官两票（任一红即红）：第一截 {:?}；二次崩溃 {:?}；RefFS 弃权的理由 {:?}",
            self.reference_filesystem_and_model_verdicts_on_crash_images
                .iter()
                .map(|(verdict, count)| (verdict.name(), *count))
                .collect::<Vec<_>>(),
            self.reference_filesystem_and_model_verdicts_on_second_crashes
                .iter()
                .map(|(verdict, count)| (verdict.name(), *count))
                .collect::<Vec<_>>(),
            self.reference_filesystem_abstentions
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

    /// 崩溃窗口末尾那个写（录制流第 `last_stream_index_of_the_crash_window` 步）之前（含），有没有一步在读故障下往录制流里发过东西
    /// （C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）；D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。
    /// 崩溃注入跑历史不带注入计划（`execute_history_with`），读故障只由 [`operation_reads_under_injected_faults`] 认的那几类步造；
    /// 那样的一步一个都没发（取号之前就拒了：C554 乙之后「崩溃恢复抛弃根」的常态）时盘面与没走这一步的历史一样，不算注入过。
    fn read_faults_in_a_crash_state(
        &self,
        step_kinds: &[HistoryOperationKind],
        last_stream_index_of_the_crash_window: usize,
    ) -> ReadFaultsInTheCrashState {
        let mut operations_before_the_step = 0;
        for (position, operations_after_the_step) in &self.after_each_step {
            let step_reads_under_injected_faults = match position {
                StepPosition::StartingPoint => false,
                StepPosition::Operation(step_index) => step_kinds
                    .get(*step_index)
                    .copied()
                    .is_some_and(operation_reads_under_injected_faults),
            };
            let step_sent_something_by_the_crash = operations_before_the_step
                < *operations_after_the_step
                && operations_before_the_step <= last_stream_index_of_the_crash_window;
            if step_reads_under_injected_faults && step_sent_something_by_the_crash {
                return ReadFaultsInTheCrashState::Injected;
            }
            operations_before_the_step = *operations_after_the_step;
        }
        ReadFaultsInTheCrashState::NoneInjected
    }
}

/// 这一类历史步在注入的读故障下读盘：崩溃恢复抛弃根（那次可写挂载期间最新那条根的根槽与它点名的单元读回全 0，
/// `singlefs_harness::history` 的 `DeviceReadingZerosOverHiddenRanges`）。别的步读的就是盘上的字节。
fn operation_reads_under_injected_faults(kind: HistoryOperationKind) -> bool {
    match kind {
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => true,
        HistoryOperationKind::PublishFirstFile
        | HistoryOperationKind::PublishOverwrite
        | HistoryOperationKind::PublishWithoutUnits
        | HistoryOperationKind::CloseAndMountWritable
        | HistoryOperationKind::RollBackWhileMounted
        | HistoryOperationKind::RaiseRollbackFloor
        | HistoryOperationKind::ColdStartRecover
        | HistoryOperationKind::Unlink
        | HistoryOperationKind::CreateInode => false,
    }
}

/// 跑一段历史，按 `draw` 在它的录制流上摆崩溃状态，逐个重建镜像、跑恢复、池级 checker 与记录核对器、问模型；
/// 再在同一份崩溃后镜像上起可写挂载、发一次布、跑 checker，挂载途中摆二次崩溃（[`CrashStateStage`] 的后两截）。
/// `image_directory` 给了就把判红的那几个崩溃状态（哪一截判红就落哪一截的镜像）落成镜像文件放进去（留现场）。
///
/// # Panics
/// 录制流没开内容保留（重建镜像要字节）——这里自己建流，走不到；`draw` 是 [`CrashPointDraw::ExactlyOneCrashState`] 而写死的段号、
/// 写数或扣下的下标在这段历史的录制流上不成立（见那一支）。
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
    // RefFS 的输入（G1 用户定案：第一截与二次崩溃那一截的判定 = RefFS ∧ O1 判官全四条）：跑历史时逐步攒。
    let mut reference_filesystem_translation =
        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed_versions.insert(key, file);
        }
        let operations_so_far = stream.operation_count();
        marks.after_the_last_finished_step = operations_so_far;
        marks
            .after_each_step
            .push((observation.position, operations_so_far));
        reference_filesystem_translation.note_an_observed_step(observation, operations_so_far);
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
    // 崩溃状态摆在最后一个跑完的操作之前，停下那一步不进 RefFS（与 O1 那一路同一个范围）。
    let reference_filesystem_history =
        reference_filesystem_translation.translate(&operations, &geometry);
    assert_eq!(
        reference_filesystem_history.segment_count(),
        segments.len(),
        "RefFS 的崩溃时刻按同一条录制流、同一条切段规则切"
    );
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
        let first_write_of_the_segment = segments[crash_point.segment_index][0];
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
        for kind in withheld_write_kinds(&writes, first_write_of_the_segment, crash_point) {
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

        // 只读恢复连同挂载透明要比的字段一起读（R1）：恢复报告就是这一次的。
        let fields_read_after_the_crash =
            read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);
        let report = &fields_read_after_the_crash.report;
        match &report.outcome {
            RecoveryOutcome::FileRead { .. } => tally.recoveries_reading_a_file += 1,
            RecoveryOutcome::NoFile { .. } => tally.recoveries_without_a_file += 1,
            RecoveryOutcome::Failed { .. } => tally.recoveries_failed += 1,
        }
        if report.journal.prefix_applied > 0 {
            tally.recoveries_that_applied_journal_records += 1;
        }
        let read_back = observed_read_back_after_a_crash(report);
        tally.model_judgements += 1;
        let disagreement =
            crash_recovery_disagreement(&committed_versions, newest_persisted, &read_back);
        // RefFS 那一票：崩溃时刻按切开当前段的那道屏障由哪一步发出归属（r6 用户定案 H1-收严，只给 RefFS 用；
        // O1 判官与 `crash_point.step` 照旧按段里第一个被扣下的写）。当前段取持久窗口最后一个写所在那一段：段内的崩溃状态就是
        // `segment_index` 那一段；靶向用例越过段尾的那一个，崩在窗口末尾那一段里（窗口里更晚的写都可能已落）。
        let segment_the_crash_closes = segment_containing_the_write(
            &segments,
            first_write_of_the_segment + writes_in_the_segment - 1,
        );
        let reference_filesystem_judgement = reference_filesystem_history.judge_a_crash(
            reference_filesystem_history.crash_moment_in_segment(segment_the_crash_closes),
            &read_back,
        );
        tally.note_the_reference_filesystem_vote(
            StageTheReferenceFileSystemJudges::CrashImage,
            &reference_filesystem_judgement.vote,
            disagreement.as_ref(),
        );
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
        let mut violations = checker_violations_on(&image, &mut tally);
        violations.extend(reference_filesystem_judgement.vote.violation());
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
        let read_faults_in_the_crash_state = marks.read_faults_in_a_crash_state(
            &step_kinds,
            stream_indexes[first_write_of_the_segment + writes_in_the_segment - 1],
        );
        match read_faults_in_the_crash_state {
            ReadFaultsInTheCrashState::NoneInjected => {}
            ReadFaultsInTheCrashState::Injected => {
                tally.crash_points_after_a_step_reading_under_injected_faults += 1;
            }
        }
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
            RefusalOnTheCrashImage {
                crash_image_materialized: &crash_image_materialized,
                version_landed_on_after_the_crash: report.effective_root,
                read_only_read_back: if disagreement.is_none() {
                    ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
                } else {
                    ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
                },
                read_faults: read_faults_in_the_crash_state,
            },
            history.seed,
            crash_point,
            image_directory,
        );
        // 没崩过的那一次挂载之后的池（挂载那一段的写全落在第一次崩溃之后的镜像上，不接之后那一次发布）：v2a。挂载被拒时没有。
        let fields_after_the_uninterrupted_mount = pool_after_the_uninterrupted_mount(
            &crash_image_materialized,
            &mount_run,
            records_across_the_crash.mount_writes(),
        )
        .map(|pool| {
            let fields = read_recovery_fields(&pool, ComparedVersion::TheRecoveredVersion);
            (pool, fields)
        });
        let content_after_the_uninterrupted_mount = match &fields_after_the_uninterrupted_mount {
            None => Err(RemountIdempotenceNotJudged::TheUninterruptedMountWasRefused),
            Some((_pool, fields)) => Ok(content_read_by_the_recovery_report(&fields.report)),
        };
        judge_mount_transparency_on_the_crash_image(
            &mut tally,
            &mut findings,
            &fields_read_after_the_crash,
            &mount_run,
            fields_after_the_uninterrupted_mount
                .as_ref()
                .map(|(pool, fields)| (pool, fields)),
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
                content_after_the_uninterrupted_mount,
                fields_read_after_the_first_crash: &fields_read_after_the_crash,
                floor_raises_in_the_uninterrupted_mount_writes: mount_run
                    .floor_raises_in_the_mount_writes,
                continuation_of_the_uninterrupted_mount: mount_run
                    .user_visible_tree_root_pointers_of_the_continuation
                    .as_ref(),
                publishes_before_the_deletion_queue_continuation: mount_run
                    .publishes_before_the_deletion_queue_continuation,
                parameters: &parameters,
                space_admission: execution.space_admission,
                reference_filesystem_at_the_first_crash: &reference_filesystem_judgement,
                read_faults_in_the_first_crash_state: read_faults_in_the_crash_state,
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

/// I-8.4（重放幂等）在这里判的那一条的编号：R-换（用户 2026-10-01 定，`research/prompts/m3-inv-a-r1-main-verification.md` 第四节）——
/// 在二次崩溃上判「恢复加写行那一整段」幂等，等价关系是读出的内容相同。池级 checker 不判它（单份镜像上恒真），判红时记在这一截的违例里。
pub const REPLAY_IDEMPOTENCE_INVARIANT: &str = "I-8.4";

/// 只读恢复（看 journal）读出的内容：I-8.4 的等价关系只比这个，不比扇区、实例代号、实例表行、恢复落到的那条根
/// （攻方 `research/prompts/m3-inv-a-r1-opus-output.md` 一·2：挂一次与崩在取号之后再挂一次，这几样都不同，读出的内容相同）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentReadByRecovery {
    File(Vec<u8>),
    NoFile,
    /// 恢复失败，什么都没读出来。
    NothingRead,
}

impl ContentReadByRecovery {
    /// 给人看：几个字节、没有文件、没读出来（不把整份内容打出来）。
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            ContentReadByRecovery::File(content) => format!("文件 {} 字节", content.len()),
            ContentReadByRecovery::NoFile => "没有文件".to_string(),
            ContentReadByRecovery::NothingRead => "恢复失败、没读出内容".to_string(),
        }
    }
}

/// 对 `reader` 跑一次只读恢复（看 journal），交回读出的内容。要恢复报告与挂载透明的字段的，走
/// `singlefs_harness::mount_transparency::read_recovery_fields`，再用 [`content_read_by_the_recovery_report`] 取内容。
#[must_use]
pub fn content_read_by_recovery(reader: &dyn PoolReader) -> ContentReadByRecovery {
    content_read_by_the_recovery_report(&recover(reader, JournalPolicy::Consult))
}

/// 一份只读恢复报告读出的内容。
#[must_use]
pub fn content_read_by_the_recovery_report(report: &RecoveryReport) -> ContentReadByRecovery {
    match &report.outcome {
        RecoveryOutcome::FileRead { content, .. } => ContentReadByRecovery::File(content.clone()),
        RecoveryOutcome::NoFile { .. } => ContentReadByRecovery::NoFile,
        RecoveryOutcome::Failed { .. } => ContentReadByRecovery::NothingRead,
    }
}

/// 二次崩溃之后再挂这一截为什么没判：这几种情形条款没写「做一次」或「做两次」的结果是什么，不替它定红绿，只计数（交主 agent）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RemountIdempotenceNotJudged {
    /// 没崩过的那一次可写挂载被拒：没有「做一次」的结果。
    TheUninterruptedMountWasRefused,
    /// 没崩过的那一次挂载之后只读恢复失败：「做一次」没读出内容，没有可比的。
    NothingWasReadAfterTheUninterruptedMount,
    /// 二次崩溃镜像上再挂被拒：「做两次」那一趟没做完。
    TheRemountWasRefused,
}

/// 「恢复加写行那一整段」做一次与做两次读出的内容比下来。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemountIdempotence {
    SameContent,
    /// 读出的内容不同：判红。
    DifferentContent {
        once: ContentReadByRecovery,
        twice: ContentReadByRecovery,
    },
    NotJudged(RemountIdempotenceNotJudged),
}

/// I-8.4 的等价关系：两边读出的内容逐字节相同（都没有文件也算相同）。做一次那一边恢复失败时没有可比的，不判；
/// 做一次读出了东西、做两次恢复失败，算内容不同。
#[must_use]
pub fn compare_content_read_once_and_twice(
    once: &ContentReadByRecovery,
    twice: &ContentReadByRecovery,
) -> RemountIdempotence {
    match once {
        ContentReadByRecovery::NothingRead => RemountIdempotence::NotJudged(
            RemountIdempotenceNotJudged::NothingWasReadAfterTheUninterruptedMount,
        ),
        ContentReadByRecovery::File(_) | ContentReadByRecovery::NoFile => {
            if once == twice {
                RemountIdempotence::SameContent
            } else {
                RemountIdempotence::DifferentContent {
                    once: once.clone(),
                    twice: twice.clone(),
                }
            }
        }
    }
}

/// 只起一次可写挂载（取号、写行、暖机，不接发布）的结局。
#[derive(Debug)]
pub enum WritableMountAlone {
    Mounted {
        instance: InstanceGeneration,
        pool_after: MemoryPool,
    },
    /// 被拒：成员、它说的理由、那个错误本身（拒得合不合法按它判）与拒之后的池（盘变没变按它比）。
    Refused {
        member: String,
        reason: ObservedRefusalReason,
        refusal: Box<MountError>,
        pool_after: MemoryPool,
    },
}

/// 在 `pool` 的副本上只起一次可写挂载（建池那一份参数、给定的空间准入开关），不接发布；交回挂载之后的池，或拒的理由、错误与拒之后的池。
#[must_use]
pub fn writable_mount_alone_on(
    pool: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    space_admission: SpaceAdmission,
) -> WritableMountAlone {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                pool.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    let mounted_instance =
        match mount_writable_with_space_admission(parameters, &mut devices, space_admission) {
            Err(error) => Err(error),
            Ok(mounted) => Ok(mounted.output.instance),
        };
    let pool_after = MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.image))
            .collect(),
        device_size_in_bytes: pool.device_size_in_bytes,
    };
    match mounted_instance {
        Err(error) => WritableMountAlone::Refused {
            member: format!("{error:?}"),
            reason: refusal_reason_of_mount_error(&error),
            refusal: Box::new(error),
            pool_after,
        },
        Ok(instance) => WritableMountAlone::Mounted {
            instance,
            pool_after,
        },
    }
}

/// 没崩过的那一次可写挂载之后的池：挂载那一段的写全落在第一次崩溃之后的镜像上（不接之后那一次发布）。I-8.4 的「做一次」与
/// 挂载透明的 v2a 都在它上面只读恢复。挂载被拒时没有（None）。
fn pool_after_the_uninterrupted_mount(
    crash_image_materialized: &MemoryPool,
    mount_run: &WritableMountOnTheCrashImage,
    mount_writes: &[RetainedWrite],
) -> Option<MemoryPool> {
    match mount_run.outcome {
        WritableMountOutcome::MountRefused { .. } => None,
        WritableMountOutcome::PublishRefused { .. } | WritableMountOutcome::Published => {
            let mut pool_after_the_mount = crash_image_materialized.clone();
            pool_after_the_mount.apply_writes(mount_writes);
            Some(pool_after_the_mount)
        }
    }
}

/// 崩溃后镜像上那次可写挂载与之后那一次发布的结局。
#[derive(Debug)]
pub enum WritableMountOutcome {
    /// 挂载被拒：成员、它说的理由与那个错误本身（拒的理由在不在 D23 已定项 14 那张单子上按它判）。
    MountRefused {
        member: String,
        reason: ObservedRefusalReason,
        refusal: Box<MountError>,
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
    /// 挂载交回值里推没推抬 F、最后抬到的 F（挂载透明要它）；挂载被拒时 None。
    pub floor_raised_by_the_mount: Option<FloorRaisedByTheMount>,
    /// 挂载续做之前这次挂载发了几次（写行、暖机、推的抬 F 的每一次空发布）：挂载那一段录制流里第几个根槽写之后起是续做；
    /// 挂载交回 `MountError::Publish` 或 `MountError::FloorRaiseFailedAfterTheMountsPublishes` 时按错里交回的已落盘发布写账划
    /// （[`publishes_before_the_continuation_of_a_mount_that_failed`]；划不出续做、别的错都是 None：写行之后都算暖机）。
    pub publishes_before_the_deletion_queue_continuation: Option<usize>,
    /// 挂载推的抬 F 从挂载那一段写表（`mount_operations` 按同一份几何切出来的写表）的哪个写起（二次崩溃上 F 判不判按它分；
    /// 挂载准入推的与续做里推的都认，界按它们在写表里的位置取，取不出的是 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]）；挂载被拒时 None。
    pub floor_raises_in_the_mount_writes: Option<FloorRaisesInTheMountWrites>,
    /// 挂载续做每一步那一版的两棵用户可见树的根指针（挂载透明那一样按它比，用户 2026-10-03 定「续做算挂载的一部分」）；挂载被拒时 None。
    pub user_visible_tree_root_pointers_of_the_continuation:
        Option<UserVisibleTreeRootPointersOfTheContinuation>,
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
/// 挂载交回的现行版本带文件时覆盖写，树表 0 条时发第一个文件（与随机历史里这两个入口同一种调法），有树而文件集合为空（删完那一版）时
/// 建一个 size 0 的 inode（D8（核心索引结构） 已定项 16 第 5 条）。
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
    let floor_raised_by_the_mount = mounted
        .as_ref()
        .ok()
        .map(|mounted| FloorRaisedByTheMount::of_the_mount_output(&mounted.output));
    let publishes_before_the_deletion_queue_continuation = match &mounted {
        Ok(mounted) => Some(
            1 + mounted.output.warm_up_publishes.len()
                + mounted
                    .output
                    .space_admission
                    .floor_raises()
                    .iter()
                    .map(|raised| raised.publishes.len())
                    .sum::<usize>(),
        ),
        Err(error) => publishes_before_the_continuation_of_a_mount_that_failed(error),
    };
    let user_visible_tree_root_pointers_of_the_continuation =
        mounted.as_ref().ok().map(|mounted| {
            UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output)
        });
    // 录制流此刻只有挂载那一段：按建池那一份参数的几何切出写表，与调用方切二次崩溃的写表同一张。抬 F 的界按位置取（挂载准入推的与续做里推的
    // 那几串都认，`FloorRaisesInTheMountWrites::of_the_mount_output`），取不出时不 panic、交 NotLocatedInTheMountWrites，
    // 这一次挂载的二次崩溃整段 F 不判、按理由计进 `mount_transparency_fields_not_judged`。
    let floor_raises_in_the_mount_writes = mounted.as_ref().ok().map(|mounted| {
        let geometry = FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        };
        let (mount_writes, _mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
        FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &mount_writes)
    });
    let outcome = match mounted {
        Err(error) => WritableMountOutcome::MountRefused {
            member: format!("{error:?}"),
            reason: refusal_reason_of_mount_error(&error),
            refusal: Box::new(error),
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
                // 删完那一版（D8（核心索引结构） 已定项 16 第 5 条：之后只许建 inode、写行与空发布）：建一个 size 0 的 inode
                // （随机历史的「建 inode」经会话建的也是一个）；覆盖写在这一版上被拒成写一个不在树里的 inode。
                PoolVersion::WithTreesAndNoFile(current) => publish_new_inodes(
                    &mut writer,
                    &mut allocator,
                    current,
                    1,
                    FIXED_WRITE_TIME_SECONDS,
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
        floor_raised_by_the_mount,
        publishes_before_the_deletion_queue_continuation,
        floor_raises_in_the_mount_writes,
        user_visible_tree_root_pointers_of_the_continuation,
    }
}

/// 一次发布写没写只有续做的删除步才写的那几种结构：文件的 extent 树、inode 树与 `deleted_inodes` 树（数据单元也算，挂载里没有别的发布写它）。
/// 写行、暖机与抬 F 的空发布只重写分配记录树、记账树、中央映射树、树表、实例表与固定结构（2026-10-03 在副本上量的：崩在意图之后重挂，
/// 写行与两次暖机写的是这几种，续做那一步写了 extent 树与 inode 树根）。
/// 五种里 `DataUnit` 与 `DeletedInodesTreeNode` 在那次探针上空转（续做那一步没写它们）、仍留着（代码轮第二轮判决 V6）：这一判认的是
/// 「写行、暖机与抬 F 的空发布都不写的结构」，不是「今天的续做那一步写了的结构」，两种都属于前者——那几种发布不碰文件数据，也不碰
/// `deleted_inodes`（队列只由删除类发布改），所以认它们不会把写行或暖机认成续做；去掉它们，划段就随续做那一步今天的写法变：
/// 第一版只有一个文件、走空那一步删 key 时 `deleted_inodes` 根指针置零（D8（核心索引结构） 已定项 16 第 5 条）而不写它的节点，
/// 队列里有不止一条 key 时删一条要重写那一片（推的，没量）；删除批只释放数据单元、不写（已定项 16 第 2 条），以后某一步续做写了数据单元，
/// 它也只可能是续做。
fn writes_a_structure_only_the_continuation_changes(writes: &WritesByStructureKind) -> bool {
    WrittenStructureKind::IN_REPORT_ORDER.iter().any(|kind| {
        let only_the_continuation_writes_it = match kind {
            WrittenStructureKind::DataUnit
            | WrittenStructureKind::ExtentTreeNode
            | WrittenStructureKind::InodeTreeLeafContainer
            | WrittenStructureKind::InodeTreeRoot
            | WrittenStructureKind::DeletedInodesTreeNode => true,
            WrittenStructureKind::AllocationRecordTreeNode
            | WrittenStructureKind::AccountingTreeNode
            | WrittenStructureKind::TombstoneTreeNode
            | WrittenStructureKind::LineageTreeNode
            | WrittenStructureKind::CentralMappingTreeNode
            | WrittenStructureKind::TreeTableUnit
            | WrittenStructureKind::InstanceTableUnit
            | WrittenStructureKind::JournalRecord
            | WrittenStructureKind::RootSlot
            | WrittenStructureKind::SystemConfigurationSlot => false,
        };
        only_the_continuation_writes_it && writes.of(*kind).write_calls > 0
    })
}

/// 挂载交回错时续做从第几次发布起（U12，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节；V1，
/// `research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）：带着这次挂载已落盘发布写账的两个成员按同一个划法划
/// （[`publishes_before_the_continuation_of_a_mount_that_handed_back_an_error`]），不并进暖机段——`Publish`（写行、暖机或续做里的一步没发成）
/// 连同失败那一次的写账；`FloorRaiseFailedAfterTheMountsPublishes`（准入或续做里推的抬 F 自己报了不是空间不够的错）按它的
/// `writes_of_persisted_publishes`，抬 F 那一串自己的写账在 `cause` 里，都是空发布、不写只有续做才写的结构，不交进来。
/// 别的错在写行之前（取号之前的判、取号自己）就交回，没有续做：None。
fn publishes_before_the_continuation_of_a_mount_that_failed(error: &MountError) -> Option<usize> {
    match error {
        MountError::Publish(failed) => {
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &failed.writes_of_persisted_publishes,
                &failed.writes_of_failed_publishes,
            )
        }
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &failed.writes_of_persisted_publishes,
                &[],
            )
        }
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::Acquisition(_)
        | MountError::RaiseFloorSequencePublishFailed(_)
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        | MountError::RollbackFloorAboveCeiling { .. }
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_) => None,
    }
}

/// 挂载在写行之后交回 `MountError::Publish` 时续做之前发了几次（U12，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：
/// `writes_of_persisted_publishes` 是已落盘的各次发布的写账（按先后，写行那次在最前，`mount::PublishSequenceFailed`），
/// `writes_of_failed_publishes` 是落盘阶段失败的那一次的。写行之后第一个写了只有续做才写的结构
/// （[`writes_a_structure_only_the_continuation_changes`]）的那一次起是续做；已落盘的都没写、失败的那一次写了，续做从失败那一次起
/// （已落盘的那几次之后）；都没写（错出在写行、暖机或准入推的抬 F 里，或续做的第一步落盘之前就被拒、一个写都没发）是 None，写行之后都算暖机。
/// 续做开头先被空间拒、先推的那一串抬 F 归暖机：错里分不出它是准入推的还是续做推的。
fn publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
    writes_of_persisted_publishes: &[WritesByStructureKind],
    writes_of_failed_publishes: &[WritesByStructureKind],
) -> Option<usize> {
    let first_persisted_continuation_step = writes_of_persisted_publishes
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, writes)| writes_a_structure_only_the_continuation_changes(writes))
        .map(|(publish_index, _)| publish_index);
    match first_persisted_continuation_step {
        Some(publish_index) => Some(publish_index),
        None if !writes_of_persisted_publishes.is_empty()
            && writes_of_failed_publishes
                .iter()
                .any(writes_a_structure_only_the_continuation_changes) =>
        {
            Some(writes_of_persisted_publishes.len())
        }
        None => None,
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

/// 判可写挂载被拒合不合法要的：崩溃后镜像（拒的判据在它上面复算，拒之后的池要与它逐字节相同）、它上面只读恢复落到的那一版、
/// 那一次只读读出在不在模型的允许集合里（问模型的结论）。第二截交第一次崩溃的镜像与第一截的结论；第三截交二次崩溃镜像与那一个二次崩溃上的结论，
/// 读故障的认定沿用第一次崩溃那个态的（第二截那次挂载起在第一次崩溃的镜像上，不注入读故障）。
struct RefusalOnTheCrashImage<'crash> {
    crash_image_materialized: &'crash MemoryPool,
    version_landed_on_after_the_crash: Option<(InstanceGeneration, CheckpointTxg)>,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
    /// 这个崩溃态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：没注入的，判据 N-配置 复算为真即拒得不合法（C670）。
    read_faults: ReadFaultsInTheCrashState,
}

/// 第二截：挂载或那次发布被拒（单元区墙只计数，模型在单元区墙上只答允许拒绝的区间、而这里没有崩溃点上的模型状态可判区间），
/// 做没做成都在之后的池上跑池级 checker；`record_check` 是同一个池上记录核对器的结论
/// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）。挂载被拒而理由在 D23（journal 的角色与格式） 已定项 14
/// 那张单子上的，判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`：判据在崩溃镜像上复算为真、
/// 盘上逐字节不变、只读读出在允许集合里；崩溃态没注入读故障而判据 N-配置 复算为真的另判不合法，C670），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
/// 模型对可写挂载与覆盖写、第一个文件从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数与发现、挂载那一趟与它之后的记录核对、判拒绝要的崩溃镜像那一份、这一个崩溃状态落在哪"
)]
fn judge_the_writable_mount_on_the_crash_image(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    mount_run: &WritableMountOnTheCrashImage,
    record_check: RecordCheck,
    refusal_on_the_crash_image: RefusalOnTheCrashImage<'_>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    tally.writable_mounts_after_the_crash += 1;
    let refusal = match &mount_run.outcome {
        WritableMountOutcome::MountRefused {
            member,
            reason,
            refusal: refusal_error,
        } => {
            let disk =
                if mount_run.pool_after == *refusal_on_the_crash_image.crash_image_materialized {
                    DiskAfterTheRefusal::ByteForByteUnchanged
                } else {
                    DiskAfterTheRefusal::Changed
                };
            match judge_writable_mount_refusal_of_a_crash_state(
                refusal_on_the_crash_image.crash_image_materialized,
                refusal_on_the_crash_image.version_landed_on_after_the_crash,
                refusal_error,
                disk,
                refusal_on_the_crash_image.read_only_read_back,
                refusal_on_the_crash_image.read_faults,
            ) {
                WritableMountRefusalVerdict::Legal(_) => {
                    tally.writable_mount_refusals_on_the_d23_list_legal += 1;
                    None
                }
                WritableMountRefusalVerdict::Illegal {
                    reason: reason_on_the_list,
                    conditions_not_met,
                } => {
                    tally.writable_mount_refusals_on_the_d23_list_illegal += 1;
                    Some(format!(
                        "崩溃后镜像上的可写挂载拒了，理由 {reason_on_the_list:?} 在 D23 已定项 14 那张单子上，拒得不合法（{conditions_not_met:?}）：{member}"
                    ))
                }
                WritableMountRefusalVerdict::ReasonNotOnTheD23List => {
                    if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall)
                    {
                        tally.writable_mounts_after_the_crash_refused_at_the_unit_area_wall += 1;
                        None
                    } else {
                        tally.writable_mounts_after_the_crash_refused += 1;
                        Some(format!("崩溃后镜像上的可写挂载拒了：{member}"))
                    }
                }
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

/// 挂载透明整个动作没判的理由（计数名）：崩溃后镜像上的可写挂载被拒，判的是拒绝本身（[`judge_the_writable_mount_on_the_crash_image`]）。
const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_MOUNT_WAS_REFUSED: &str = "可写挂载被拒（判拒绝本身）";
/// 挂载做成、之后那一次发布被拒：没有「挂载加一次发布」那一版可比。
const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_PUBLISH_AFTER_THE_MOUNT_WAS_REFUSED: &str =
    "挂载之后那一次发布被拒";
/// 二次崩溃镜像上：没崩过的那一次可写挂载被拒，没有挂载交回值（推没推抬 F）。
const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED: &str =
    "没崩过的那一次可写挂载被拒";

/// 挂载透明的一判记进计数：整个动作没判的按理由、某一样没判的按理由、判红的按字段（名字里带量过或推的）；交回判红的那几样，
/// 一样一条违例（名字是 [`singlefs_harness::mount_transparency::MountTransparencyField::violation_name`]）。
fn violations_of_the_mount_transparency(
    tally: &mut CrashInjectionTally,
    transparency: &MountTransparency,
) -> Vec<(&'static str, String)> {
    let verdicts = match transparency {
        MountTransparency::NotJudged(reason) => {
            *tally
                .mount_transparency_not_judged
                .entry(reason.name())
                .or_insert(0) += 1;
            return Vec::new();
        }
        MountTransparency::Judged(verdicts) => verdicts,
    };
    let mut violations = Vec::new();
    for verdict in verdicts {
        match &verdict.outcome {
            FieldOutcome::Holds => {}
            FieldOutcome::NotJudged(reason) => {
                *tally
                    .mount_transparency_fields_not_judged
                    .entry(reason.name())
                    .or_insert(0) += 1;
            }
            FieldOutcome::Violated { expected, observed } => {
                let name = verdict.field.violation_name(verdict.basis);
                *tally
                    .mount_transparency_violations_by_field
                    .entry(name)
                    .or_insert(0) += 1;
                violations.push((name, format!("期望 {expected}；读出 {observed}")));
            }
        }
    }
    violations
}

/// 第一次崩溃镜像上一个动作的挂载透明：R1（`before_the_mount`）与动作之后的池（`pool_after_the_action`）上的只读恢复（`after_the_action`）逐字段比；
/// 判红的落成这一截（[`CrashStateStage::MountTransparencyAfterTheCrash`]）的新发现，镜像留的是动作之后的池。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数与发现、哪个动作、两次恢复与动作之后的池、动作与挂载续做那几步的交回值、这一个崩溃状态落在哪"
)]
fn judge_mount_transparency_of_one_comparison(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    comparison: MountTransparencyComparison,
    before_the_mount: &RecoveryFieldsRead,
    pool_after_the_action: &MemoryPool,
    after_the_action: &RecoveryFieldsRead,
    action: MountTransparencyAction,
    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    let transparency =
        judge_mount_transparency(before_the_mount, after_the_action, action, continuation);
    match &transparency {
        MountTransparency::Judged(_) => {
            *tally
                .mount_transparency_judged
                .entry(comparison)
                .or_insert(0) += 1;
        }
        MountTransparency::NotJudged(_) => {}
    }
    let violations = violations_of_the_mount_transparency(tally, &transparency);
    if violations.is_empty() {
        return;
    }
    let image = CrashImage {
        base: pool_after_the_action,
        writes: &[],
        persisted: Vec::new(),
    };
    let detail = violations
        .iter()
        .map(|(name, detail)| format!("{name}：{detail}"))
        .collect::<Vec<_>>()
        .join("；");
    let observation = crash_state_observation(
        &image,
        crash_point,
        violations,
        None,
        RecordCheck::default(),
    );
    findings.settle(
        tally,
        FailureAtAStage {
            seed,
            crash_point,
            stage: CrashStateStage::MountTransparencyAfterTheCrash(comparison),
            observation,
            read_back: detail,
            image: &image,
        },
        image_directory,
    );
}

/// 挂载做成了（`floor_raised_by_the_mount` 是 Some）时一起交回的续做那几步的两根指针。
fn continuation_of_a_mount_that_was_made(
    mount_run: &WritableMountOnTheCrashImage,
) -> &UserVisibleTreeRootPointersOfTheContinuation {
    mount_run
        .user_visible_tree_root_pointers_of_the_continuation
        .as_ref()
        .expect("挂载做成时抬 F 与续做那几步的根指针一起交回（mount_and_publish_once_on_the_crash_image 里同一个 Ok 取的两样）")
}

/// 第一次崩溃镜像上的挂载透明（r5 判决第二节 U5）：只挂载（v2a，`after_the_uninterrupted_mount` 是那个池与它上面的只读恢复）、
/// 挂载加一次发布（比发布之前那一版）各判一次。挂载被拒的不判（判的是拒绝本身），发布被拒的不判「挂载加一次发布」那一个。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数与发现、R1、挂载那一趟与 v2a、这一个崩溃状态落在哪"
)]
fn judge_mount_transparency_on_the_crash_image(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    before_the_mount: &RecoveryFieldsRead,
    mount_run: &WritableMountOnTheCrashImage,
    after_the_uninterrupted_mount: Option<(&MemoryPool, &RecoveryFieldsRead)>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    match (
        after_the_uninterrupted_mount,
        mount_run.floor_raised_by_the_mount,
    ) {
        (Some((pool, after_the_mount)), Some(floor)) => judge_mount_transparency_of_one_comparison(
            tally,
            findings,
            MountTransparencyComparison::TheMountAlone,
            before_the_mount,
            pool,
            after_the_mount,
            MountTransparencyAction::MountAlone(floor),
            continuation_of_a_mount_that_was_made(mount_run),
            seed,
            crash_point,
            image_directory,
        ),
        (None, _) | (_, None) => {
            *tally
                .mount_transparency_not_judged
                .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_MOUNT_WAS_REFUSED)
                .or_insert(0) += 1;
        }
    }
    match (&mount_run.outcome, mount_run.floor_raised_by_the_mount) {
        (WritableMountOutcome::Published, Some(floor)) => {
            let after_the_publish = read_recovery_fields(
                &mount_run.pool_after,
                ComparedVersion::TheVersionBeforeTheRecoveredOne,
            );
            judge_mount_transparency_of_one_comparison(
                tally,
                findings,
                MountTransparencyComparison::TheMountAndOnePublish,
                before_the_mount,
                &mount_run.pool_after,
                &after_the_publish,
                MountTransparencyAction::MountAndOnePublish(floor),
                continuation_of_a_mount_that_was_made(mount_run),
                seed,
                crash_point,
                image_directory,
            );
        }
        (WritableMountOutcome::PublishRefused { .. }, _) => {
            *tally
                .mount_transparency_not_judged
                .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_PUBLISH_AFTER_THE_MOUNT_WAS_REFUSED)
                .or_insert(0) += 1;
        }
        (WritableMountOutcome::MountRefused { .. } | WritableMountOutcome::Published, _) => {
            *tally
                .mount_transparency_not_judged
                .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_MOUNT_WAS_REFUSED)
                .or_insert(0) += 1;
        }
    }
}

/// 二次崩溃镜像上的挂载透明（r5 判决第二节 U6，用户定 F6「加二次」）：只读恢复读出的与 R1 比。挂载推了抬 F 而二次崩溃镜像上
/// 抬 F 的写有持久的（从第一串把新 F 写进系统配置的那个写起，`persisted_in_the_mount` 与挂载那一段写表逐条对应）时，F 那一样不判
/// （条款没写落在哪一处）；暖机那一次发布的写崩出来的态照 r5 判决 U5 前半句判 F 不变（代码轮第二轮判决 K2-1）。抬 F 的界在挂载写表里取不出的
/// （代码轮第三轮判决第三节第 3 条），这一次挂载的二次崩溃整段 F 不判、计数。没崩过的那一次挂载被拒的不判。
fn mount_transparency_violations_of_a_second_crash(
    tally: &mut CrashInjectionTally,
    fields_read_after_the_first_crash: &RecoveryFieldsRead,
    fields_read_after_the_second_crash: &RecoveryFieldsRead,
    floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
    continuation_of_the_uninterrupted_mount: Option<&UserVisibleTreeRootPointersOfTheContinuation>,
    persisted_in_the_mount: &[bool],
    phase: WritableMountPhase,
) -> Vec<(&'static str, String)> {
    let Some(floor_raises_in_the_mount_writes) = floor_raises_in_the_uninterrupted_mount_writes
    else {
        *tally
            .mount_transparency_not_judged
            .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED)
            .or_insert(0) += 1;
        return Vec::new();
    };
    let continuation = continuation_of_the_uninterrupted_mount.expect(
        "没崩过的那一次挂载做成时抬 F 的界与续做那几步的根指针一起交回（mount_and_publish_once_on_the_crash_image 里同一个 Ok 取的两样）",
    );
    let floor_raises = floor_raises_in_the_mount_writes.at_a_second_crash(persisted_in_the_mount);
    let transparency = judge_mount_transparency(
        fields_read_after_the_first_crash,
        fields_read_after_the_second_crash,
        MountTransparencyAction::SecondCrashInsideTheMount(floor_raises),
        continuation,
    );
    match &transparency {
        MountTransparency::Judged(_) => {
            *tally
                .mount_transparency_judged_on_second_crashes_by_phase
                .entry(phase)
                .or_insert(0) += 1;
        }
        MountTransparency::NotJudged(_) => {}
    }
    violations_of_the_mount_transparency(tally, &transparency)
}

/// 挂载那一段录制流里每一次写落在挂载的哪一段：先是取号的系统配置槽写，第一个别的写起是写行那次发布；
/// 一次发布的根槽写与紧跟着的系统配置槽写（轮换、抬 F 那一串先写的新 F）之后的第一个别的写起是下一次发布：
/// 写行之后、第 `publishes_before_the_deletion_queue_continuation` 个根槽写之前的那几次是暖机（连同挂载推的抬 F），
/// 从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）。
fn writable_mount_phase_of_each_write(
    writes: &[RetainedWrite],
    publishes_before_the_deletion_queue_continuation: Option<usize>,
) -> Vec<WritableMountPhase> {
    let phase_of_the_publish_after = |root_records_written: usize| match (
        root_records_written,
        publishes_before_the_deletion_queue_continuation,
    ) {
        (0, _) => WritableMountPhase::RowPublish,
        (publishes_landed, Some(publishes_before_the_continuation))
            if publishes_landed >= publishes_before_the_continuation =>
        {
            WritableMountPhase::Continuation
        }
        (_, Some(_) | None) => WritableMountPhase::WarmUp,
    };
    let mut phase = WritableMountPhase::Acquisition;
    let mut root_records_written = 0;
    // 迭代上界是挂载那一段的写数；跨轮带的是这一次写落在哪一段与已写的根槽数；没有提前出口。
    writes
        .iter()
        .map(|write| {
            phase = match (phase, write.kind) {
                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => {
                    WritableMountPhase::Acquisition
                }
                (
                    WritableMountPhase::RowPublish
                    | WritableMountPhase::WarmUp
                    | WritableMountPhase::Continuation,
                    StepKind::SystemConfigurationSlot,
                ) => phase,
                (
                    WritableMountPhase::Acquisition
                    | WritableMountPhase::RowPublish
                    | WritableMountPhase::WarmUp
                    | WritableMountPhase::Continuation,
                    StepKind::UnitWrite
                    | StepKind::JournalRecord
                    | StepKind::RootRecordFua
                    | StepKind::ZeroFill
                    | StepKind::Barrier,
                ) => phase_of_the_publish_after(root_records_written),
            };
            if write.kind == StepKind::RootRecordFua {
                root_records_written += 1;
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
    /// 没崩过的那一次可写挂载之后读出的内容（I-8.4 的「做一次」）；挂载被拒时是没判的理由。
    content_after_the_uninterrupted_mount:
        Result<ContentReadByRecovery, RemountIdempotenceNotJudged>,
    /// 没崩过的那一次挂载在续做之前发了几次（[`WritableMountOnTheCrashImage::publishes_before_the_deletion_queue_continuation`]）。
    publishes_before_the_deletion_queue_continuation: Option<usize>,
    /// 第一次崩溃镜像上的只读恢复与挂载透明要比的字段（R1）。
    fields_read_after_the_first_crash: &'history RecoveryFieldsRead,
    /// 没崩过的那一次挂载推的抬 F 从挂载那一段写表的哪个写起（没推是 [`FloorRaisesInTheMountWrites::NotPushed`]）；挂载被拒时 None。
    floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
    /// 没崩过的那一次挂载续做每一步那一版的两根指针（二次崩溃落到挂载写的那一版时挂载透明按它比）；挂载被拒时 None。
    continuation_of_the_uninterrupted_mount:
        Option<&'history UserVisibleTreeRootPointersOfTheContinuation>,
    /// 二次崩溃之后再挂用的建池参数与空间准入开关（与没崩过的那一次挂载同一份）。
    parameters: &'history MakeFilesystemParameters,
    space_admission: SpaceAdmission,
    /// RefFS 在第一次崩溃上的判定：判绿时带着以读出的内容重新起头的 RefFS，二次崩溃接着它判。
    reference_filesystem_at_the_first_crash: &'history ReferenceFileSystemJudgementOfACrash,
    /// 第一次崩溃那个态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：二次崩溃之后再挂被拒时判 C670 那一条沿用它。
    read_faults_in_the_first_crash_state: ReadFaultsInTheCrashState,
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
        content_after_the_uninterrupted_mount,
        publishes_before_the_deletion_queue_continuation,
        fields_read_after_the_first_crash,
        floor_raises_in_the_uninterrupted_mount_writes,
        continuation_of_the_uninterrupted_mount,
        parameters,
        space_admission,
        reference_filesystem_at_the_first_crash,
        read_faults_in_the_first_crash_state,
    } = inputs;
    let mount_writes = records_across_the_crash.mount_writes();
    let phase_of_each_write = writable_mount_phase_of_each_write(
        mount_writes,
        publishes_before_the_deletion_queue_continuation,
    );
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
        let fields_read_after_the_second_crash =
            read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);
        let report = &fields_read_after_the_second_crash.report;
        match &report.outcome {
            RecoveryOutcome::FileRead { .. } => tally.second_crash_recoveries_reading_a_file += 1,
            RecoveryOutcome::NoFile { .. } => tally.second_crash_recoveries_without_a_file += 1,
            RecoveryOutcome::Failed { .. } => tally.second_crash_recoveries_failed += 1,
        }
        let read_back_after_the_second_crash = observed_read_back_after_a_crash(report);
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
        let reference_filesystem_vote = judge_a_second_crash_after_the_restart(
            reference_filesystem_at_the_first_crash,
            &read_back_after_the_second_crash,
        );
        tally.note_the_reference_filesystem_vote(
            StageTheReferenceFileSystemJudges::SecondCrashInsideTheWritableMount,
            &reference_filesystem_vote,
            disagreement.as_ref(),
        );
        tally.second_crash_record_checks += 1;
        let record_check = records_across_the_crash.record_check_after_a_second_crash(
            &persisted_in_the_mount,
            &image,
            report.effective_root,
        );
        let second_crash_image_materialized = materialized_crash_image(&image);
        judge_the_remount_after_a_second_crash(
            tally,
            findings,
            &content_after_the_uninterrupted_mount,
            &RefusalOnTheCrashImage {
                crash_image_materialized: &second_crash_image_materialized,
                version_landed_on_after_the_crash: report.effective_root,
                read_only_read_back: if disagreement.is_none() {
                    ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
                } else {
                    ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
                },
                read_faults: read_faults_in_the_first_crash_state,
            },
            parameters,
            space_admission,
            second.phase,
            seed,
            crash_point,
            image_directory,
        );
        tally.second_crash_checker_runs += 1;
        let mut violations = checker_violations_of(&image);
        violations.extend(reference_filesystem_vote.violation());
        violations.extend(mount_transparency_violations_of_a_second_crash(
            tally,
            fields_read_after_the_first_crash,
            &fields_read_after_the_second_crash,
            floor_raises_in_the_uninterrupted_mount_writes,
            continuation_of_the_uninterrupted_mount,
            &persisted_in_the_mount,
            second.phase,
        ));
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

/// I-8.4 的 R-换：二次崩溃镜像（`second_crash_image.crash_image_materialized`）上再起一次可写挂载，「恢复加写行那一整段」于是做了两次；
/// 再挂之后只读恢复读出的内容要与没崩过的那一次挂载之后读出的（`once`）相同。不同判红（违例记在 [`REPLAY_IDEMPOTENCE_INVARIANT`] 名下）；
/// 条款没写的情形（[`RemountIdempotenceNotJudged`]）只计数。再挂被拒时 I-8.4 照旧不判，另判 C670 那一条
/// （[`judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion`]）；没崩过的那一次挂载被拒（`once` 是没判的理由）时不再挂，两样都不判。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：两次挂载的输入、计数与发现、这一个二次崩溃落在哪"
)]
fn judge_the_remount_after_a_second_crash(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    once: &Result<ContentReadByRecovery, RemountIdempotenceNotJudged>,
    second_crash_image: &RefusalOnTheCrashImage<'_>,
    parameters: &MakeFilesystemParameters,
    space_admission: SpaceAdmission,
    phase: WritableMountPhase,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    tally.remounts_after_a_second_crash += 1;
    let once_content = match once {
        Err(reason) => {
            *tally
                .remount_idempotence_not_judged
                .entry(*reason)
                .or_insert(0) += 1;
            return;
        }
        Ok(content) => content,
    };
    let remount = writable_mount_alone_on(
        second_crash_image.crash_image_materialized,
        parameters,
        space_admission,
    );
    let pool_after_the_remount = match &remount {
        WritableMountAlone::Refused {
            member,
            refusal,
            pool_after: pool_after_the_refusal,
            ..
        } => {
            *tally
                .remount_idempotence_not_judged
                .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)
                .or_insert(0) += 1;
            judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(
                tally,
                findings,
                RemountRefusedAfterASecondCrash {
                    member,
                    refusal,
                    pool_after_the_refusal,
                    second_crash_image,
                    phase,
                },
                seed,
                crash_point,
                image_directory,
            );
            return;
        }
        WritableMountAlone::Mounted { pool_after, .. } => pool_after,
    };
    let (differing_content_read_once, differing_content_read_twice) =
        match compare_content_read_once_and_twice(
            once_content,
            &content_read_by_recovery(pool_after_the_remount),
        ) {
            RemountIdempotence::SameContent => {
                *tally
                    .remount_idempotence_judged_by_phase
                    .entry(phase)
                    .or_insert(0) += 1;
                tally.remounts_after_a_second_crash_reading_the_same_content += 1;
                return;
            }
            RemountIdempotence::NotJudged(reason) => {
                *tally
                    .remount_idempotence_not_judged
                    .entry(reason)
                    .or_insert(0) += 1;
                return;
            }
            RemountIdempotence::DifferentContent {
                once: read_once,
                twice: read_twice,
            } => (read_once, read_twice),
        };
    *tally
        .remount_idempotence_judged_by_phase
        .entry(phase)
        .or_insert(0) += 1;
    tally.remounts_after_a_second_crash_reading_different_content += 1;
    let image = CrashImage {
        base: pool_after_the_remount,
        writes: &[],
        persisted: Vec::new(),
    };
    let detail = format!(
        "恢复加写行那一整段做一次读出 {}，崩在挂载的 {} 里再做一次读出 {}",
        differing_content_read_once.render(),
        phase.name(),
        differing_content_read_twice.render()
    );
    let observation = crash_state_observation(
        &image,
        crash_point,
        vec![(REPLAY_IDEMPOTENCE_INVARIANT, detail.clone())],
        None,
        RecordCheck::default(),
    );
    findings.settle(
        tally,
        FailureAtAStage {
            seed,
            crash_point,
            stage: CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(phase),
            observation,
            read_back: detail,
            image: &image,
        },
        image_directory,
    );
}

/// 二次崩溃镜像上再挂被拒的那一次：成员、错误、拒之后的池、二次崩溃镜像那一份判拒要的、崩在挂载的哪一段。
struct RemountRefusedAfterASecondCrash<'remount> {
    member: &'remount str,
    refusal: &'remount MountError,
    pool_after_the_refusal: &'remount MemoryPool,
    second_crash_image: &'remount RefusalOnTheCrashImage<'remount>,
    phase: WritableMountPhase,
}

/// 第三截判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那一条（攻方改法 P1，`research/prompts/m3-formal-code-b16-r1-main-verification.md`
/// 第二节 K2-b）：二次崩溃镜像同样是一个崩溃态，再挂的拒因是判据 N-配置、崩溃态没注入读故障、判据在二次崩溃镜像上（撕裂的系统配置槽跳过）
/// 复算仍为真 ⇒ 拒得不合法，报红（记在 [`CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount`] 那一截）。判官一并判的 U6 那四条
/// （r5 判决第二节 U6）条款只管第二截，这一截不拿它们定红绿：只差那几条的不报红、不计不合法，判官列出的没满足的条件只随 C670 那一条的红带出来。
/// 拒因不是判据 N-配置 的不判。
fn judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    remount: RemountRefusedAfterASecondCrash<'_>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    let RemountRefusedAfterASecondCrash {
        member,
        refusal,
        pool_after_the_refusal,
        second_crash_image,
        phase,
    } = remount;
    let disk = if *pool_after_the_refusal == *second_crash_image.crash_image_materialized {
        DiskAfterTheRefusal::ByteForByteUnchanged
    } else {
        DiskAfterTheRefusal::Changed
    };
    let verdict = judge_writable_mount_refusal_of_a_crash_state(
        second_crash_image.crash_image_materialized,
        second_crash_image.version_landed_on_after_the_crash,
        refusal,
        disk,
        second_crash_image.read_only_read_back,
        second_crash_image.read_faults,
    );
    let Some(conditions_not_met) =
        conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(tally, verdict)
    else {
        return;
    };
    let detail = format!(
        "崩在挂载的 {} 里、二次崩溃镜像上再挂被拒，理由 {:?}；崩溃态没注入读故障，判据在二次崩溃镜像上复算仍为真，拒得不合法（C670；判官列出的没满足的条件 {conditions_not_met:?}）：{member}",
        phase.name(),
        RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
    );
    let model_disagreement = ModelDisagreement::new(
        ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
        "没注入读故障的崩溃态上，见证判据不许为真：二次崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）都该成"
            .to_string(),
        detail.clone(),
    );
    let image = CrashImage {
        base: second_crash_image.crash_image_materialized,
        writes: &[],
        persisted: Vec::new(),
    };
    let observation = crash_state_observation(
        &image,
        crash_point,
        Vec::new(),
        Some(model_disagreement),
        RecordCheck::default(),
    );
    findings.settle(
        tally,
        FailureAtAStage {
            seed,
            crash_point,
            stage: CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(phase),
            observation,
            read_back: detail,
            image: &image,
        },
        image_directory,
    );
}

/// 第三截那一判的计数与取舍：拒因是判据 N-配置 的记一次判过；判官列出的没满足的条件里有 C670 那一条
/// （[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）时再记一次不合法，交回全部没满足的条件（报红用）。
/// 拒因不是它、C670 那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
fn conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(
    tally: &mut CrashInjectionTally,
    verdict: WritableMountRefusalVerdict,
) -> Option<Vec<RefusalConditionNotMet>> {
    let conditions_not_met = match verdict {
        WritableMountRefusalVerdict::ReasonNotOnTheD23List
        | WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
        )
        | WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
            ..
        } => return None,
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
        ) => {
            tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
            conditions_not_met,
        } => {
            tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration += 1;
            conditions_not_met
        }
    };
    let witness_criterion_true_without_read_faults = conditions_not_met.iter().any(|condition| {
        matches!(
            condition,
            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_)
        )
    });
    if !witness_criterion_true_without_read_faults {
        return None;
    }
    tally.remount_refusals_with_the_witness_criterion_true_without_read_faults += 1;
    Some(conditions_not_met)
}

/// 这个崩溃状态在段内扣下的写各是什么种类（同一种只记一次）。段是写表里连续的一截
/// （`writes_and_segments_with_stream_indexes` 按次序切），第 k 个写的下标是段首加 k——靶向用例摆的持久集合越过段尾时也按这个数。
fn withheld_write_kinds(
    writes: &[RetainedWrite],
    first_write_of_the_segment: usize,
    crash_point: &CrashPoint,
) -> Vec<StepKind> {
    let mut kinds: BTreeSet<StepKind> = BTreeSet::new();
    for (within_the_segment, is_persisted) in
        crash_point.persisted_within_the_segment.iter().enumerate()
    {
        if !is_persisted {
            kinds.insert(writes[first_write_of_the_segment + within_the_segment].kind);
        }
    }
    kinds.into_iter().collect()
}

/// 写表下标 `write` 落在第几段。
///
/// # Panics
/// 这个写不在任何一段里：段与写表不是同一条流切的（每个写都恰好落在一个段里，`split_into_segments`）。
fn segment_containing_the_write(segments: &[Vec<usize>], write: usize) -> usize {
    segments
        .iter()
        .position(|segment| segment.contains(&write))
        .expect("每个写都恰好落在一个段里：段与写表是同一条流按同一条规则切的")
}

/// 按 `draw` 摆崩溃状态：候选段是「整段的写都落在 mkfs 之后、最后一个跑完的操作之前」的那几段，段内取真子集。
///
/// 起点那一段（取号、暖机、第一个文件）在内——2026-09-20 才放开；mkfs 那几段不在内，与层 0 同一条界：
/// mkfs 不是事务，它写到一半的盘面上还没有池，恢复报不出根是对的，拿事务的 oracle 去判只会判出假红
/// （层 0 也从 `mkfs_operation_count` 之后起枚举）。失败那一步的写也不在内（模型还没判过它写出的根）。
///
/// 抽的那一路同一段同一个子集只摆一次；摆完按段号排好——基线只往前叠。
/// 靶向用例那一支（[`CrashPointDraw::ExactlyOneCrashState`]）只摆写死的那一个，持久集合可以越过段尾；写死的输入不对就 panic（见那一支）。
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
    let mut source = SeededRandomSource::from_seed(seed.0 ^ CRASH_POINT_SEED_SALT);
    let mut chosen: BTreeSet<(usize, Vec<bool>)> = BTreeSet::new();
    match draw {
        CrashPointDraw::Sampled {
            crash_points_per_history,
        } => {
            // 一个候选段都没有时一个都不摆（下面按候选段数取随机下标，取不出）。
            if candidate_segments.is_empty() {
                return Vec::new();
            }
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
        CrashPointDraw::ExactlyOneCrashState {
            segment_index,
            writes_from_the_segment_on,
            withheld_among_them,
        } => {
            assert!(
                candidate_segments.contains(&segment_index),
                "靶向用例写死的第 {segment_index} 段不是候选段（整段落在 mkfs 之后、最后一个跑完的操作之前的那几段）：{candidate_segments:?}"
            );
            let first_write_of_the_segment = segments[segment_index][0];
            let writes_of_the_state =
                first_write_of_the_segment..first_write_of_the_segment + writes_from_the_segment_on;
            assert!(
                writes_from_the_segment_on > 0 && writes_of_the_state.end <= stream_indexes.len(),
                "靶向用例从第 {segment_index} 段起数 {writes_from_the_segment_on} 个写：要至少一个、不越过写表末尾（{} 个写）",
                stream_indexes.len()
            );
            // 段首已经在候选段里（落在 mkfs 之后）；这几个写的末尾不越过最后一个跑完的操作。
            assert!(
                stream_indexes[writes_of_the_state.end - 1] < marks.after_the_last_finished_step,
                "靶向用例从第 {segment_index} 段起数的 {writes_from_the_segment_on} 个写越过了最后一个跑完的操作"
            );
            assert!(
                !withheld_among_them.is_empty()
                    && withheld_among_them
                        .iter()
                        .all(|withheld| *withheld < writes_from_the_segment_on),
                "靶向用例扣下的写 {withheld_among_them:?} 要至少一个、都落在从这一段起数的 {writes_from_the_segment_on} 个写里"
            );
            chosen.insert((
                segment_index,
                (0..writes_from_the_segment_on)
                    .map(|within_the_state| !withheld_among_them.contains(&within_the_state))
                    .collect(),
            ));
        }
    }
    chosen
        .into_iter()
        .map(|(segment_index, persisted_within_the_segment)| {
            let first_withheld = persisted_within_the_segment
                .iter()
                .position(|persisted| !persisted)
                .expect("段内持久的是真子集，至少有一个写被扣下");
            // 段是写表里连续的一截：段内第 k 个写是段首加 k（靶向用例的持久集合越过段尾时也这样数）。
            let step = marks
                .step_containing(stream_indexes[segments[segment_index][0] + first_withheld])
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
    /// 这一批照哪一组断言判（交「重放这一整批」那一行写出来；跑的时候不读它）。
    pub assertions: CrashInjectionAssertions,
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
            let _ = writeln!(
                text,
                "  复现这一段：{}",
                crash_injection_replay_line(
                    finding.seed.0,
                    1,
                    &self.history_sampling(),
                    self.draw,
                    CrashInjectionAssertions::NewFindingsOnly
                )
            );
        }
        let _ = writeln!(text, "重放这一整批：{}", self.replay_line());
        text
    }

    /// 这一批的取样参数（每段步数、比重、跑法）。
    #[must_use]
    pub fn history_sampling(&self) -> HistorySampling {
        HistorySampling {
            operations_per_history: self.operations_per_history,
            weights: self.weights,
            execution: self.execution,
        }
    }

    /// 重放这一整批的那一行命令（[`crash_injection_replay_line`]）：带着这一批照哪一组断言判，
    /// 照抄跑大档那条入口，红在结构或覆盖断言上的也红在同一条上。
    #[must_use]
    pub fn replay_line(&self) -> String {
        crash_injection_replay_line(
            self.first_seed,
            self.seed_count,
            &self.history_sampling(),
            self.draw,
            self.assertions,
        )
    }
}

/// 崩溃注入那个二进制的环境变量前缀；取样参数的后缀与随机历史共用（`singlefs_harness::history::HistorySamplingVariable`）。
pub const CRASH_INJECTION_VARIABLE_PREFIX: ReproductionVariablePrefix =
    ReproductionVariablePrefix("SINGLEFS_CRASH_INJECTION_");

/// 重放行与重放入口里崩溃注入自己的几个变量（接在 [`CRASH_INJECTION_VARIABLE_PREFIX`] 后面）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashInjectionVariable {
    /// 种子区间的起点。
    FirstSeed,
    /// 种子数。
    Seeds,
    /// 每段摆几个崩溃状态（`CrashPointDraw::Sampled`）；整段枚举短段那一种是长段上各抽几个。
    Points,
    /// 写数不超过几的段整段枚举（`CrashPointDraw::EveryProperSubsetOfShortSegments`）；只抽样的那一种写 `none`。
    ShortSegmentWrites,
    /// 照哪一组断言判（[`CrashInjectionAssertions`]）。
    Assertions,
}

impl CrashInjectionVariable {
    /// 重放行按这个次序写出崩溃注入自己的几个变量（取样参数接在后面）。
    pub const ALL: [CrashInjectionVariable; 5] = [
        CrashInjectionVariable::FirstSeed,
        CrashInjectionVariable::Seeds,
        CrashInjectionVariable::Points,
        CrashInjectionVariable::ShortSegmentWrites,
        CrashInjectionVariable::Assertions,
    ];

    #[must_use]
    pub const fn suffix(self) -> &'static str {
        match self {
            CrashInjectionVariable::FirstSeed => "FIRST_SEED",
            CrashInjectionVariable::Seeds => "SEEDS",
            CrashInjectionVariable::Points => "POINTS",
            CrashInjectionVariable::ShortSegmentWrites => "SHORT_SEGMENT_WRITES",
            CrashInjectionVariable::Assertions => "ASSERTIONS",
        }
    }

    #[must_use]
    pub fn variable_name(self) -> String {
        CRASH_INJECTION_VARIABLE_PREFIX.variable_named(self.suffix())
    }
}

singlefs_harness::enum_listing_every_member! {
/// 照重放行重跑的那一趟按哪一组断言判：快档、小快档、回退取样那几条用例除了清单外的失败，各有自己的结构与覆盖断言
/// （每个崩溃状态与每个二次崩溃都跑过什么、各条路径跑到没有），大档只判清单外的失败。重放行带上它，大档那条入口照它挑断言，
/// 红在结构或覆盖断言上的照抄重放行也红在同一条上（代码三方 test-script-align-code-r1 判决 S2：原先重放行一律交只判新发现的大档入口，
/// 那一类红照抄复现不出）。各组断言写在 `tests/crash_injection_campaign.rs` 的 `judge_crash_injection_report`，这里只定名字与取值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashInjectionAssertions {
    /// 只判清单外的失败：大档那一组；新发现之后「复现这一段」那一行也照它判（一段的新发现与整批的结构计数无关）。
    NewFindingsOnly,
    /// 小快档那一组（`crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed`）。
    SmallFastTier,
    /// 快档那一组（`crash_injection_fast_tier_recovers_only_into_versions_the_model_committed`）。
    FastTier,
    /// 偏向抬 F 之后回退的取样点那一组（`crash_states_land_inside_rollbacks_and_floor_raises`）。
    RollbackSampling,
}
}

impl CrashInjectionAssertions {
    /// 重放行与大档入口的环境变量里这一组的取值（[`CrashInjectionVariable::Assertions`]）。
    #[must_use]
    pub const fn environment_value(self) -> &'static str {
        match self {
            CrashInjectionAssertions::NewFindingsOnly => "new-findings",
            CrashInjectionAssertions::SmallFastTier => "small-fast-tier",
            CrashInjectionAssertions::FastTier => "fast-tier",
            CrashInjectionAssertions::RollbackSampling => "rollback-sampling",
        }
    }
}

/// 只抽样（`CrashPointDraw::Sampled`）时 [`CrashInjectionVariable::ShortSegmentWrites`] 的取值。
const SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE: &str = "none";

/// 重放行里这一种摆法的两个变量的取值：(每段几个 / 长段上各抽几个, 短段的写数上限或 `none`)。
fn draw_environment_values(draw: CrashPointDraw) -> (String, String) {
    match draw {
        CrashPointDraw::Sampled {
            crash_points_per_history,
        } => (
            crash_points_per_history.to_string(),
            SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE.to_string(),
        ),
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes,
            sampled_in_longer_segments,
        } => (
            sampled_in_longer_segments.to_string(),
            segment_writes.to_string(),
        ),
        CrashPointDraw::ExactlyOneCrashState { .. } => {
            panic!("{A_TARGETED_CRASH_STATE_IS_NOT_A_CAMPAIGN_DRAW}")
        }
    }
}

/// 崩溃注入判红时打的重放命令：种子区间、崩溃状态怎么摆、照哪一组断言判、取样参数（每段步数、比重、checker、盘宽、空间准入），
/// 交大档那条入口（`crash_injection_campaign` 的 `crash_injection_large_tier_from_the_environment`）照抄就是原来那一批、照原来那一组断言判。
/// 线程数不在里面：线程数不改结论（`one_thread_and_four_threads_render_the_same_report`）。
#[must_use]
pub fn crash_injection_replay_line(
    first_seed: u64,
    seed_count: u64,
    sampling: &HistorySampling,
    draw: CrashPointDraw,
    assertions: CrashInjectionAssertions,
) -> String {
    let (points, short_segment_writes) = draw_environment_values(draw);
    let mut assignments: Vec<(String, String)> = CrashInjectionVariable::ALL
        .iter()
        .map(|variable| {
            let value = match variable {
                CrashInjectionVariable::FirstSeed => first_seed.to_string(),
                CrashInjectionVariable::Seeds => seed_count.to_string(),
                CrashInjectionVariable::Points => points.clone(),
                CrashInjectionVariable::ShortSegmentWrites => short_segment_writes.clone(),
                CrashInjectionVariable::Assertions => assertions.environment_value().to_string(),
            };
            (variable.variable_name(), value)
        })
        .collect();
    assignments.extend(sampling.environment_assignments(CRASH_INJECTION_VARIABLE_PREFIX));
    format!(
        "{} cargo test --release -p singlefs-checker-tier --test crash_injection_campaign -- --include-ignored --exact crash_injection_large_tier_from_the_environment",
        render_environment_assignments(&assignments)
    )
}

/// 大档那条入口读环境：读的变量与 [`crash_injection_replay_line`] 写出的逐个相同（照哪一组断言判也在里面），没设的取 `defaults` 的；
/// 线程数与镜像留不留不在重放行里，照 `defaults` 的。
/// 这是读环境变量的测试入口，哪个变量写错都停下（不交回错误让调用方各自处置）：种子起点、种子数、崩溃点数、短段写数
/// 经数值环境变量共用的那一份解析；断言组与取样参数（每段步数、比重、checker、盘宽、空间准入）经 `history` 的复现入口解析
/// （随机历史、坏盘输入、故障注入的入口也读这几个），认不得时照它报的那一句 panic。消息都带变量名与原值。
///
/// # Panics
/// 某个变量的取值认不得：数值变量不是装得进的非负整数（短段写数也不是 `none`），或取值是一组名字的变量不在认得的那几个里；
/// 「跑几个」的计数写 0（种子数、每段步数、只抽样时每段几个崩溃状态、短段写数上限）也算认不得。
#[must_use]
pub fn crash_injection_campaign_from_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    defaults: &CrashInjectionCampaign,
) -> CrashInjectionCampaign {
    let first_seed = u64_from_reproduction_environment(
        lookup,
        &CrashInjectionVariable::FirstSeed.variable_name(),
    )
    .unwrap_or(defaults.first_seed);
    let seed_count = count_from_reproduction_environment_panicking_when_unreadable(
        lookup,
        &CrashInjectionVariable::Seeds.variable_name(),
    )
    .unwrap_or(defaults.seed_count);
    let (default_points, default_short_segment_writes) = draw_environment_values(defaults.draw);
    let points_variable = CrashInjectionVariable::Points.variable_name();
    let points_text = lookup(&points_variable).unwrap_or(default_points);
    let points = usize_from_the_variable_text(&points_variable, &points_text);
    let short_segment_writes_variable = CrashInjectionVariable::ShortSegmentWrites.variable_name();
    let short_segment_writes =
        lookup(&short_segment_writes_variable).unwrap_or(default_short_segment_writes);
    // 只抽样时崩溃点数是每段摆几个，写 0 就一个崩溃状态都不摆；短段整段枚举时它是长段上另抽几个，0 是「只枚举短段」。
    // 短段写数是写数上限，写 0 没有一段够得上，与 none 同义却让重放行多一种写法，按认不出停下。
    let draw = if short_segment_writes == SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE {
        CrashPointDraw::Sampled {
            crash_points_per_history: std::num::NonZeroUsize::new(points)
                .unwrap_or_else(|| {
                    panic!(
                        "{points_variable}={points_text:?} 认不出：只抽样（短段写数是 {SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE}）时每段摆几个崩溃状态要写正整数，\
                         0 个就是这一批一个崩溃状态都不摆"
                    )
                })
                .get(),
        }
    } else {
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: positive_count_from_the_variable_text(
                &short_segment_writes_variable,
                &short_segment_writes,
            ),
            sampled_in_longer_segments: points,
        }
    };
    let assertions = choice_from_reproduction_environment(
        lookup,
        &CrashInjectionVariable::Assertions.variable_name(),
        &CrashInjectionAssertions::ALL,
        CrashInjectionAssertions::environment_value,
        defaults.assertions,
    )
    .unwrap_or_else(|unrecognized| panic!("{}", unrecognized.render()));
    let sampling = HistorySampling::from_environment(
        CRASH_INJECTION_VARIABLE_PREFIX,
        lookup,
        &HistorySampling {
            operations_per_history: defaults.operations_per_history,
            weights: defaults.weights,
            execution: defaults.execution,
        },
    )
    .unwrap_or_else(|unrecognized| panic!("{}", unrecognized.render()));
    CrashInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history: sampling.operations_per_history,
        draw,
        weights: sampling.weights,
        execution: sampling.execution,
        worker_threads: defaults.worker_threads,
        image_retention: defaults.image_retention,
        assertions,
    }
}

/// 读一个 u64 的非负整数变量（种子起点）：没设是 `None`，由调用方取默认。解析经数值环境变量共用的那一份。
///
/// # Panics
/// 设了而不是装得进 u64 的非负整数：消息带变量名与原值。
fn u64_from_reproduction_environment(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
) -> Option<u64> {
    number_from_the_variable_value_panicking_when_unreadable::<u64>(
        variable,
        lookup(variable).as_deref(),
    )
}

/// 读一个「跑几个」的计数变量（种子数）：没设是 `None`，由调用方取默认。解析经数值环境变量共用的那一份，取 `NonZeroU64`：
/// 写 0 就是这一批一段都不跑，照样跑完、按只判清单外失败的那一组判成绿（上游 `show-me-test.md`「扫到 0 项也不是通过」），与写错一样停下。
///
/// # Panics
/// 设了而不是装得进 u64 的正整数：消息带变量名与原值。
fn count_from_reproduction_environment_panicking_when_unreadable(
    lookup: &dyn Fn(&str) -> Option<String>,
    variable: &str,
) -> Option<u64> {
    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(
        variable,
        lookup(variable).as_deref(),
    )
    .map(std::num::NonZeroU64::get)
}

/// 一个装得进 usize 的非负整数变量的原文换成数：解析经数值环境变量共用的那一份，写错就 panic、消息带变量名与原值。
///
/// # Panics
/// `text` 不是装得进 usize 的非负整数。
fn usize_from_the_variable_text(variable: &str, text: &str) -> usize {
    number_from_the_variable_value_panicking_when_unreadable::<usize>(variable, Some(text)).expect(
        "交进去的值是 Some（没设时调用方已取了默认原文），共用解析只在值是 None 时交回 None",
    )
}

/// 一个计数变量（短段的写数上限）的原文换成数：同 [`usize_from_the_variable_text`]，取 `NonZeroUsize`，写 0 同样停下。
///
/// # Panics
/// `text` 不是装得进 usize 的正整数。
fn positive_count_from_the_variable_text(variable: &str, text: &str) -> usize {
    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
        variable,
        Some(text),
    )
    .expect("交进去的值是 Some（没设时调用方已取了默认原文），共用解析只在值是 None 时交回 None")
    .get()
}

/// 一次崩溃注入跑什么：种子区间、每段几步、怎么摆崩溃状态、比重、历史怎么跑、线程数、判红时镜像留不留，
/// 以及跑完照哪一组断言判（跑的时候不读它，只交给报告的重放行）。
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
    pub assertions: CrashInjectionAssertions,
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，
/// 线程按片号从小到大领片，调用线程收到一片就打一行 `CRASH_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），
/// 再按片号从小到大并计数。计数按片的次序相加、新发现按种子从小到大留第一个，所以
/// [`CrashInjectionReport::render`] 与线程数、调度次序无关（进度行是按到达次序打的，只报跑到哪了，不带判定）。
///
/// # Panics
/// 某个工作线程 panic（历史里的 panic 在执行器里接住，走到这里的是崩溃状态上恢复或 checker 的断言）；有一片领了却没交回；
/// `draw` 是靶向用例那一支 [`CrashPointDraw::ExactlyOneCrashState`]（只经 [`inject_crashes_into_history`] 摆，不进战役）。
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
        assertions,
    } = *campaign;
    assert!(
        !matches!(draw, CrashPointDraw::ExactlyOneCrashState { .. }),
        "{A_TARGETED_CRASH_STATE_IS_NOT_A_CAMPAIGN_DRAW}"
    );
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
        assertions,
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
    use singlefs_core::mount::FloorRaiseFailedAfterTheMountsPublishes;
    use singlefs_harness::history::{generate_history, HistoryStartingPoint};

    const FOUR_CRASH_POINTS: CrashPointDraw = CrashPointDraw::Sampled {
        crash_points_per_history: 4,
    };

    /// I-8.4 的等价关系：读出的内容逐字节相同才算相同；一边读出 3000 字节、另一边差一个字节、少一截、没有文件或恢复失败，都判内容不同；
    /// 做一次那一边恢复失败时没有可比的，不判。
    #[test]
    fn content_read_twice_that_differs_from_content_read_once_is_red_and_the_same_content_is_not() {
        let once = ContentReadByRecovery::File(vec![7u8; 3000]);
        let mut one_byte_changed = vec![7u8; 3000];
        one_byte_changed[1234] = 8;
        assert_eq!(
            compare_content_read_once_and_twice(&once, &once.clone()),
            RemountIdempotence::SameContent,
            "逐字节相同的内容判相同"
        );
        assert_eq!(
            compare_content_read_once_and_twice(
                &ContentReadByRecovery::NoFile,
                &ContentReadByRecovery::NoFile
            ),
            RemountIdempotence::SameContent,
            "两边都没有文件判相同"
        );
        for twice in [
            ContentReadByRecovery::File(one_byte_changed),
            ContentReadByRecovery::File(vec![7u8; 2999]),
            ContentReadByRecovery::NoFile,
            ContentReadByRecovery::NothingRead,
        ] {
            assert_eq!(
                compare_content_read_once_and_twice(&once, &twice),
                RemountIdempotence::DifferentContent {
                    once: once.clone(),
                    twice: twice.clone()
                },
                "做两次读出 {} 与做一次读出的 3000 字节不同，要判红",
                twice.render()
            );
        }
        assert_eq!(
            compare_content_read_once_and_twice(
                &ContentReadByRecovery::NothingRead,
                &ContentReadByRecovery::NothingRead
            ),
            RemountIdempotence::NotJudged(
                RemountIdempotenceNotJudged::NothingWasReadAfterTheUninterruptedMount
            ),
            "做一次那一边没读出内容，没有可比的，不判"
        );
    }

    /// 挂载续做（Q-J，D8（核心索引结构） 已定项 16 第 6 条）：挂载那一段的写里，写行之后、第「续做之前发了几次」个根槽写之后起的写
    /// 归续做那一段，而且摆二次崩溃时那一段也摆到一个。造的写表：取号的系统配置槽写；写行那次（单元、记录、根槽、轮换的系统配置槽）；
    /// 一次暖机（单元、根槽）；续做那一步（单元、记录、根槽、轮换）——续做之前发了 2 次。
    /// 判别力：`WritableMountPhase::ALL` 里没有 `Continuation` 时这一段一个二次崩溃都摆不到；按写的种类（不按根槽数）分段时续做的写归暖机。
    #[test]
    fn the_writes_after_the_publishes_before_the_continuation_belong_to_the_continuation_phase_and_get_a_second_crash_point(
    ) {
        let write_of = |kind: StepKind| RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: DeviceOffsetInBytes(0),
            contents: singlefs_harness::memory_pool::WrittenContents::Bytes(vec![0; 512]),
        };
        let kinds = [
            StepKind::SystemConfigurationSlot,
            StepKind::UnitWrite,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::SystemConfigurationSlot,
            StepKind::UnitWrite,
            StepKind::RootRecordFua,
            StepKind::UnitWrite,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::SystemConfigurationSlot,
        ];
        let writes: Vec<RetainedWrite> = kinds.iter().map(|kind| write_of(*kind)).collect();
        let phases = writable_mount_phase_of_each_write(&writes, Some(2));
        assert_eq!(
            phases,
            [
                WritableMountPhase::Acquisition,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::WarmUp,
                WritableMountPhase::WarmUp,
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation,
            ],
            "取号、写行（到它的根槽与轮换）、暖机一次、续做一步"
        );
        let crash_point = CrashPoint {
            segment_index: 0,
            persisted_within_the_segment: vec![false],
            step: StepPosition::StartingPoint,
            operation_kind: None,
        };
        let one_write_per_segment: Vec<Vec<usize>> = (0..writes.len())
            .map(|write_index| vec![write_index])
            .collect();
        let second_crash_points = draw_second_crash_points(
            HistorySeed(7),
            &crash_point,
            &one_write_per_segment,
            &phases,
        );
        assert_eq!(
            second_crash_points
                .iter()
                .filter(|point| point.phase == WritableMountPhase::Continuation)
                .count(),
            1,
            "续做那一段摆一个二次崩溃：{second_crash_points:?}"
        );
    }

    /// 造写表用：一个 512 字节的写；根槽写的字节里在偏移 24、28 写上根身份（`root_identity_written_by` 的读法）。
    fn synthetic_write(kind: StepKind, root: Option<(u32, u64)>) -> RetainedWrite {
        let mut bytes = vec![0u8; 512];
        if let Some((instance, checkpoint_txg)) = root {
            bytes[24..28].copy_from_slice(&instance.to_le_bytes());
            bytes[28..36].copy_from_slice(&checkpoint_txg.to_le_bytes());
        }
        RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: DeviceOffsetInBytes(0),
            contents: singlefs_harness::memory_pool::WrittenContents::Bytes(bytes),
        }
    }

    fn writes_of_kinds(kinds: &[WrittenStructureKind]) -> WritesByStructureKind {
        let mut writes = WritesByStructureKind::NOTHING_WRITTEN;
        for kind in kinds {
            writes.count_write_call(*kind, &[0u8; 512]);
        }
        writes
    }

    /// U12（`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：挂载在续做里交回 `MountError::Publish` 时，
    /// 按错里交回的已落盘发布写账划出续做段。写账照 2026-10-03 在副本上量到的种类造：写行（实例表、树表、分配记录树……）、
    /// 暖机（分配记录树、记账树……）、续做那一步（extent 树、inode 树根……）。
    /// 判别力：错的那一支交 None 时续做的写全归暖机段（攻方 A4）；不认续做的种类时同样交 None。
    #[test]
    fn a_mount_that_handed_back_an_error_inside_the_continuation_marks_the_continuation_by_the_persisted_publishes(
    ) {
        let row_publish = writes_of_kinds(&[
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::TreeTableUnit,
            WrittenStructureKind::InstanceTableUnit,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let warm_up = writes_of_kinds(&[
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::AccountingTreeNode,
            WrittenStructureKind::TreeTableUnit,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let continuation_step = writes_of_kinds(&[
            WrittenStructureKind::ExtentTreeNode,
            WrittenStructureKind::InodeTreeRoot,
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &[
                    row_publish.clone(),
                    warm_up.clone(),
                    warm_up.clone(),
                    continuation_step.clone(),
                    continuation_step.clone()
                ],
                &[],
            ),
            Some(3),
            "写行、两次暖机之后的第一步续做起是续做"
        );
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &[row_publish.clone(), warm_up.clone()],
                std::slice::from_ref(&continuation_step),
            ),
            Some(2),
            "续做第一步落盘阶段失败：续做从失败那一次起"
        );
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&warm_up),
            ),
            None,
            "错出在暖机里：没有续做，写行之后都算暖机"
        );
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                &[],
                std::slice::from_ref(&row_publish),
            ),
            None,
            "写行那次就失败：没有续做"
        );
        let phases_of_a_mount_that_failed_in_the_second_continuation_step =
            writable_mount_phase_of_each_write(
                &[
                    synthetic_write(StepKind::SystemConfigurationSlot, None),
                    synthetic_write(StepKind::UnitWrite, None),
                    synthetic_write(StepKind::RootRecordFua, Some((2, 10))),
                    synthetic_write(StepKind::UnitWrite, None),
                    synthetic_write(StepKind::RootRecordFua, Some((2, 11))),
                    synthetic_write(StepKind::UnitWrite, None),
                    synthetic_write(StepKind::RootRecordFua, Some((2, 12))),
                    synthetic_write(StepKind::UnitWrite, None),
                ],
                publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
                    &[row_publish, warm_up, continuation_step.clone()],
                    std::slice::from_ref(&continuation_step),
                ),
            );
        assert_eq!(
            phases_of_a_mount_that_failed_in_the_second_continuation_step[5..],
            [
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation,
                WritableMountPhase::Continuation
            ],
            "续做那两步（落盘的一步与失败那一步写出的单元）的写落在续做段，不并进暖机段"
        );
    }

    /// V1（`research/prompts/m3-unlink-code-r2-main-verification.md` 第三节）：挂载交回 `MountError::FloorRaiseFailedAfterTheMountsPublishes`
    /// （续做里推的抬 F 自己报了不是空间不够的错）时，按它的 `writes_of_persisted_publishes` 划续做段，与 `MountError::Publish` 那一臂同一个划法：
    /// 写账照 U12 那一条的种类造（写行、暖机、续做那一步），续做从第 2 次发布起；只有写行与暖机的（准入推的抬 F 报错，今天就交这个成员）没有续做。
    /// 判别力：这个成员归「别的错」交 None 时续做的写全归暖机段。
    #[test]
    fn a_mount_whose_floor_raise_failed_after_a_continuation_step_marks_the_continuation_by_the_persisted_publishes(
    ) {
        let row_publish = writes_of_kinds(&[
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::TreeTableUnit,
            WrittenStructureKind::InstanceTableUnit,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let warm_up = writes_of_kinds(&[
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::AccountingTreeNode,
            WrittenStructureKind::TreeTableUnit,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let continuation_step = writes_of_kinds(&[
            WrittenStructureKind::ExtentTreeNode,
            WrittenStructureKind::InodeTreeRoot,
            WrittenStructureKind::AllocationRecordTreeNode,
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SystemConfigurationSlot,
        ]);
        let floor_raise_failed_after =
            |writes_of_persisted_publishes: Vec<WritesByStructureKind>| {
                MountError::FloorRaiseFailedAfterTheMountsPublishes(Box::new(
                    FloorRaiseFailedAfterTheMountsPublishes {
                        cause: MountError::InstanceTableMalformed,
                        writes_of_persisted_publishes,
                        floor_raises: Vec::new(),
                    },
                ))
            };
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_failed(&floor_raise_failed_after(
                vec![row_publish.clone(), warm_up.clone(), continuation_step]
            )),
            Some(2),
            "写行、一次暖机之后的那一步续做起是续做"
        );
        assert_eq!(
            publishes_before_the_continuation_of_a_mount_that_failed(&floor_raise_failed_after(
                vec![row_publish, warm_up]
            )),
            None,
            "准入推的抬 F 报错（只落了写行与暖机）：没有续做，写行之后都算暖机"
        );
    }

    /// D8（核心索引结构） 已定项 16 第 5 条：删完那一版上只许建 inode、写行与空发布。第一个文件之后经挂着的会话 unlink 它跑完（意图那次 + 最后一批），
    /// 整份盘面当崩溃镜像：可写挂载交回的现行那一版是有树而文件集合为空的一版，之后那一次发布建一个 size 0 的 inode、做成。
    /// 判别力：那一版上照旧调覆盖写时被拒成 `WriteToAnInodeThatIsNotInTheTree`、结局是「发布拒了」。
    #[test]
    fn the_publish_after_a_writable_mount_on_the_emptied_version_creates_an_inode_and_is_published()
    {
        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
        let width = HistoryDeviceWidth::UnitAreaOf384Slots;
        let execution = HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        };
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::Unlink],
        };
        let stream = SharedStream::retaining_contents();
        let run = execute_history_with(&history, execution, &stream, &mut |_observation| {});
        assert_eq!(
            run.outcomes.len(),
            history.operations.len(),
            "unlink 那一步要跑完：{:?}",
            run.ending
        );
        let mut crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            width.device_bytes(),
        );
        crash_image.apply(&stream.retained_operations());
        assert!(
            matches!(
                recover(&crash_image, JournalPolicy::Consult).outcome,
                RecoveryOutcome::NoFile { .. }
            ),
            "前提：unlink 跑完之后的盘面上只读恢复读出「没有文件」"
        );
        let crash_point = CrashPoint {
            segment_index: 0,
            persisted_within_the_segment: Vec::new(),
            step: StepPosition::StartingPoint,
            operation_kind: None,
        };
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &crash_image,
            &width.parameters(),
            execution,
            HistorySeed(0),
            &crash_point,
        );
        assert!(
            matches!(mount_run.outcome, WritableMountOutcome::Published),
            "删完那一版上挂载之后那一次发布建 inode、做成：{:?}",
            mount_run.outcome
        );
    }

    /// 挂载每一段都摆得到二次崩溃的两段历史（U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：从第一个文件起的
    /// 随机一段（起点就是攻方 `research/prompts/m3-inv-a-r1-opus-output.md` 一·2 的「新池新建文件做完的镜像」），与第一个文件之后经挂着的会话
    /// unlink 它的一段（意图那次 + 最后一批：崩溃状态有落在意图那一版上的，那一版上的可写挂载续做、写出续做那一段，
    /// 之后那一次发布落在删完那一版上）。两段同一种摆法（短段整段枚举），计数并在一起。
    fn crash_injections_into_histories_reaching_every_mount_phase(
    ) -> (CrashInjectionTally, Vec<HistoryCrashInjection>) {
        let histories = [
            GeneratedHistory {
                seed: HistorySeed(3),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations: generate_history(HistorySeed(3), 10).operations,
            },
            GeneratedHistory {
                seed: HistorySeed(3),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations: vec![HistoryOperation::Unlink],
            },
        ];
        let injections: Vec<HistoryCrashInjection> = histories
            .iter()
            .map(|history| {
                inject_crashes_into_history(
                    history,
                    HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
                    CrashPointDraw::EveryProperSubsetOfShortSegments {
                        segment_writes: 2,
                        sampled_in_longer_segments: 2,
                    },
                    None,
                )
            })
            .collect();
        let mut tally = CrashInjectionTally::default();
        for injection in &injections {
            tally.absorb(&injection.tally);
        }
        (tally, injections)
    }

    /// I-8.4 的 R-换 在真的二次崩溃上不误红：两段历史（[`crash_injections_into_histories_reaching_every_mount_phase`]），
    /// 起点那一段也摆崩溃状态；每个二次崩溃镜像上都再挂了一次，取号、写行、暖机、续做四段各有判过的，
    /// 读出的内容都与没崩过的那一次挂载之后读出的相同（攻方量到实例代号、实例表行、扇区都不同的那种也算相同）。
    #[test]
    fn the_remount_after_each_second_crash_reads_the_same_content_as_the_uninterrupted_mount_in_every_mount_phase(
    ) {
        let (tally, injections) = crash_injections_into_histories_reaching_every_mount_phase();
        let tally = &tally;
        assert!(
            injections
                .iter()
                .flat_map(|injection| injection.crash_points.iter())
                .any(|crash_point| crash_point.step == StepPosition::StartingPoint),
            "起点那一段（新池新建文件做完）里要有崩溃状态"
        );
        assert_eq!(
            tally.remounts_after_a_second_crash, tally.second_crash_points,
            "每个二次崩溃镜像上都再挂过一次：{tally:?}"
        );
        for phase in WritableMountPhase::ALL {
            assert!(
                tally
                    .remount_idempotence_judged_by_phase
                    .get(&phase)
                    .copied()
                    .unwrap_or(0)
                    >= 1,
                "崩在挂载的 {} 里再挂，一次都没判过 I-8.4：判了的 {:?}，没判的 {:?}",
                phase.name(),
                tally.remount_idempotence_judged_by_phase,
                tally.remount_idempotence_not_judged
            );
        }
        assert_eq!(
            tally.remounts_after_a_second_crash_reading_different_content,
            0,
            "二次崩溃之后再挂读出的内容与没崩过的那一次不同（I-8.4 的 R-换 判红）：{:?}",
            injections
                .iter()
                .flat_map(|injection| injection.new_findings.iter())
                .filter(|finding| matches!(
                    finding.stage,
                    CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
                ))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            tally.remounts_after_a_second_crash_reading_the_same_content,
            tally
                .remount_idempotence_judged_by_phase
                .values()
                .sum::<u64>(),
            "判过的都读出相同的内容"
        );
    }

    /// 挂载透明在崩溃注入里真的判了（r5 判决第二节 U5、U6）：与上一条同样两段历史、同一种摆法，每个挂载做成的崩溃状态都判了只挂载那一个，
    /// 挂载与之后那次发布都做成的都判了挂载加一次发布那一个（删完那一版上那次发布建 inode），二次崩溃取号、写行、暖机、续做四段各有判过的；
    /// 一样都没红（挂载续做过时用户可见树根指针按续做那几步比、F 认续做推的抬 F：用户 2026-10-03 定「续做算挂载的一部分」）。
    #[test]
    fn mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field(
    ) {
        let (tally, injections) = crash_injections_into_histories_reaching_every_mount_phase();
        let tally = &tally;
        assert_eq!(
            tally.recoveries_failed, 0,
            "前提：崩溃镜像上的只读恢复都落到了一版：{tally:?}"
        );
        assert!(
            tally.writable_mounts_after_the_crash_succeeded >= 1,
            "前提：崩溃后镜像上的可写挂载做成过：{tally:?}"
        );
        assert_eq!(
            tally
                .mount_transparency_judged
                .get(&MountTransparencyComparison::TheMountAlone)
                .copied()
                .unwrap_or(0),
            tally.writable_mounts_after_the_crash_succeeded,
            "每个挂载做成的崩溃状态都判了只挂载那一个：{:?}",
            tally.mount_transparency_not_judged
        );
        assert_eq!(
            tally
                .mount_transparency_judged
                .get(&MountTransparencyComparison::TheMountAndOnePublish)
                .copied()
                .unwrap_or(0),
            tally.publishes_after_the_writable_mount,
            "每个挂载与发布都做成的崩溃状态都判了挂载加一次发布那一个：{:?}",
            tally.mount_transparency_not_judged
        );
        for phase in WritableMountPhase::ALL {
            assert!(
                tally
                    .mount_transparency_judged_on_second_crashes_by_phase
                    .get(&phase)
                    .copied()
                    .unwrap_or(0)
                    >= 1,
                "崩在挂载的 {} 里的二次崩溃镜像一次都没判过挂载透明：{:?}",
                phase.name(),
                tally.mount_transparency_judged_on_second_crashes_by_phase
            );
        }
        assert!(
            tally.mount_transparency_violations_by_field.is_empty(),
            "挂载透明判红的字段 {:?}：{}",
            tally.mount_transparency_violations_by_field,
            injections
                .iter()
                .flat_map(|injection| injection.new_findings.iter())
                .map(CrashPointFinding::render)
                .collect::<String>()
        );
    }

    /// 代码轮第二轮判决 K2-1：崩溃后镜像上那次可写挂载推了抬 F 时，交回抬 F 从挂载写表的哪个写起——一个系统配置槽写，
    /// 前面紧挨着暖机末尾的系统配置轮换；二次崩溃上 F 判不判按它分（分法两侧的判定由 harness 档
    /// `mount_transparency_judged_field_by_field_after_a_writable_mount` 钉）。盘面照攻方 fms12 的造法：单元区 256 槽的小盘、
    /// 关掉空间准入跑第一个文件之后重挂与 12 次覆盖写，再判准入挂载一次。
    #[test]
    fn the_writable_mount_on_a_crash_image_names_the_first_floor_raise_write_after_the_warm_up() {
        use singlefs_harness::history::{
            ContentChoice, ContentLength, HistoryDeviceWidth, PerStepChecker,
        };
        let width = HistoryDeviceWidth::UnitAreaOf256Slots;
        let mut operations = vec![HistoryOperation::CloseAndMountWritable];
        operations.extend(std::iter::repeat_n(
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 200 },
                fill_seed: 11,
            }),
            12,
        ));
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations,
        };
        let stream = SharedStream::retaining_contents();
        let run = execute_history_with(
            &history,
            HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: width,
                space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
            },
            &stream,
            &mut |_observation| {},
        );
        assert_eq!(
            run.outcomes.len(),
            history.operations.len(),
            "写死的那段历史每一步都要跑完：{:?}",
            run.ending
        );
        let mut crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            width.device_bytes(),
        );
        crash_image.apply(&stream.retained_operations());
        let crash_point = CrashPoint {
            segment_index: 0,
            persisted_within_the_segment: Vec::new(),
            step: StepPosition::StartingPoint,
            operation_kind: None,
        };
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &crash_image,
            &width.parameters(),
            HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: width,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            },
            HistorySeed(0),
            &crash_point,
        );
        assert!(
            matches!(
                mount_run.floor_raised_by_the_mount,
                Some(FloorRaisedByTheMount::RaisedTo(_))
            ),
            "前提：这次挂载在写行与暖机之后推了抬 F：{:?}",
            mount_run.floor_raised_by_the_mount
        );
        let Some(FloorRaisesInTheMountWrites::FirstWriteAt(first_floor_raise_write)) =
            mount_run.floor_raises_in_the_mount_writes
        else {
            panic!(
                "挂载推了抬 F，交回的却不是抬 F 从哪个写起：{:?}",
                mount_run.floor_raises_in_the_mount_writes
            );
        };
        let (mount_writes, _mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(
                &mount_run.mount_operations,
                &width.fixed_geometry(),
            );
        assert_eq!(
            (
                mount_writes[first_floor_raise_write - 1].kind,
                mount_writes[first_floor_raise_write].kind
            ),
            (
                StepKind::SystemConfigurationSlot,
                StepKind::SystemConfigurationSlot
            ),
            "抬 F 从把新 F 写进系统配置的那个写起，前面紧挨着暖机末尾的系统配置轮换"
        );
    }

    /// 重放行写出的两种摆法（只抽样、短段整段枚举）都经重放入口的解析原样读回，连同种子区间与取样参数；
    /// 默认值取的是另一组，读回的不是写出的那一组就红。
    #[test]
    fn every_crash_point_draw_reads_back_from_its_own_replay_line() {
        let defaults = CrashInjectionCampaign {
            first_seed: 1,
            seed_count: 2,
            operations_per_history: 3,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 9,
            },
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        };
        for draw in [
            CrashPointDraw::Sampled {
                crash_points_per_history: 4,
            },
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 5,
                sampled_in_longer_segments: 7,
            },
        ] {
            let sampling = HistorySampling {
                operations_per_history: 24,
                weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
                execution: HistoryExecution {
                    per_step_checker: singlefs_harness::history::PerStepChecker::Skipped,
                    device_width: singlefs_harness::history::HistoryDeviceWidth::UnitAreaOf256Slots,
                    space_admission:
                        singlefs_core::admission::SpaceAdmission::SkippedByTheTestOnlySwitch,
                },
            };
            let line = crash_injection_replay_line(
                11,
                12,
                &sampling,
                draw,
                CrashInjectionAssertions::RollbackSampling,
            );
            let assignments = singlefs_harness::history::environment_assignments_in_line(&line);
            let read_back = crash_injection_campaign_from_environment(
                &|variable: &str| assignments.get(variable).cloned(),
                &defaults,
            );
            assert_eq!(
                read_back,
                CrashInjectionCampaign {
                    first_seed: 11,
                    seed_count: 12,
                    operations_per_history: 24,
                    draw,
                    weights: sampling.weights,
                    execution: sampling.execution,
                    worker_threads: defaults.worker_threads,
                    image_retention: defaults.image_retention,
                    assertions: CrashInjectionAssertions::RollbackSampling,
                },
                "重放行：{line}"
            );
        }
    }

    /// 重放行写出的每一组断言（大档、小快档、快档、回退取样）都经重放入口的解析原样读回；默认值取的是另一组，读回的不是写出的那一组就红。
    #[test]
    fn every_assertion_set_reads_back_from_its_own_replay_line() {
        let defaults_judged_by = |assertions: CrashInjectionAssertions| CrashInjectionCampaign {
            first_seed: 1,
            seed_count: 2,
            operations_per_history: 3,
            draw: FOUR_CRASH_POINTS,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions,
        };
        let sampling = HistorySampling {
            operations_per_history: 3,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        };
        for (position, assertions) in CrashInjectionAssertions::ALL.into_iter().enumerate() {
            let another =
                CrashInjectionAssertions::ALL[(position + 1) % CrashInjectionAssertions::ALL.len()];
            let line = crash_injection_replay_line(1, 2, &sampling, FOUR_CRASH_POINTS, assertions);
            let assignments = singlefs_harness::history::environment_assignments_in_line(&line);
            assert_eq!(
                crash_injection_campaign_from_environment(
                    &|variable: &str| assignments.get(variable).cloned(),
                    &defaults_judged_by(another),
                ),
                defaults_judged_by(assertions),
                "重放行：{line}"
            );
        }
    }

    /// 重放入口只设了 `variable` 这一个变量（取 `value`）时读回的那一批；其余取一组写死的默认。
    fn campaign_read_back_with_only_this_variable_set(
        variable: &'static str,
        value: &'static str,
    ) -> CrashInjectionCampaign {
        campaign_read_back_with_these_variables_set(&[(variable, value)])
    }

    /// 重放入口只设了 `assignments` 里这几个变量（变量名、取值）时读回的那一批；其余取一组写死的默认
    /// （种子起点 1、2 段、每段 3 步、只抽样每段 4 个）。
    fn campaign_read_back_with_these_variables_set(
        assignments: &[(&str, &str)],
    ) -> CrashInjectionCampaign {
        crash_injection_campaign_from_environment(
            &|asked: &str| {
                assignments
                    .iter()
                    .find(|(variable, _value)| *variable == asked)
                    .map(|(_variable, value)| (*value).to_string())
            },
            &CrashInjectionCampaign {
                first_seed: 1,
                seed_count: 2,
                operations_per_history: 3,
                draw: FOUR_CRASH_POINTS,
                weights: GenerationWeights::BROAD,
                execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
                worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
                image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
                assertions: CrashInjectionAssertions::NewFindingsOnly,
            },
        )
    }

    /// 断言组写成认不得的名字：重放入口停下，消息带变量名、原值与认得的那几个，不悄悄退回默认的那一组
    /// （代码三方 test-script-align-code-r1 判决 S2；重放入口写错一律停下）。
    #[test]
    #[should_panic(
        expected = "SINGLEFS_CRASH_INJECTION_ASSERTIONS=structure：只认 new-findings、small-fast-tier、fast-tier、rollback-sampling"
    )]
    fn an_unknown_crash_injection_assertion_set_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_ASSERTIONS",
            "structure",
        );
    }

    /// 取样参数里的盘宽写成认不得的名字：重放入口停下，消息带变量名与原值，不悄悄退回默认的盘宽。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_DEVICES=small：只认")]
    fn an_unknown_crash_injection_device_width_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_DEVICES",
            "small",
        );
    }

    /// 种子起点写错：经数值环境变量共用的那一份解析停下，消息带变量名与原值，不悄悄退回默认的种子起点。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_FIRST_SEED=\"eleven\" 认不出")]
    fn an_unreadable_crash_injection_first_seed_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_FIRST_SEED",
            "eleven",
        );
    }

    /// 短段写数既不是 `none` 也不是数：经数值环境变量共用的那一份解析停下，消息带变量名与原值，不悄悄当成只抽样。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES=\"five\" 认不出")]
    fn an_unreadable_crash_injection_short_segment_writes_value_panics_naming_the_variable_and_the_value(
    ) {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES",
            "five",
        );
    }

    /// 重放行里崩溃点数写错：经数值环境变量共用的那一份解析 panic，消息带变量名与原值。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_POINTS=\"four\" 认不出")]
    fn an_unreadable_crash_injection_points_value_panics_naming_the_variable_and_the_value() {
        let defaults = CrashInjectionCampaign {
            first_seed: 1,
            seed_count: 2,
            operations_per_history: 3,
            draw: FOUR_CRASH_POINTS,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        };
        let _campaign = crash_injection_campaign_from_environment(
            &|variable: &str| {
                (variable == "SINGLEFS_CRASH_INJECTION_POINTS").then(|| "four".to_string())
            },
            &defaults,
        );
    }

    /// 同一个重放入口里种子起点（经数值环境变量共用的那一份解析）与每段步数（经 `history` 的复现入口解析）前后都带空白：
    /// 两个结局相同，都去掉空白读回（原先一个照收、一个停下）。
    #[test]
    fn blanks_around_the_first_seed_and_the_step_count_of_the_replay_entry_are_dropped_alike() {
        let read_back = campaign_read_back_with_these_variables_set(&[
            ("SINGLEFS_CRASH_INJECTION_FIRST_SEED", " 11"),
            ("SINGLEFS_CRASH_INJECTION_OPERATIONS", " 24"),
        ]);
        assert_eq!(
            (read_back.first_seed, read_back.operations_per_history),
            (11, 24),
            "种子起点与每段步数前后的空白都去掉读回"
        );
    }

    /// 种子数写 0：重放入口停下，消息带变量名与原值，不照收（一段都不跑的一批照只判清单外失败的那一组判成绿）。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SEEDS=\"0\" 认不出")]
    fn a_crash_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
        let _campaign =
            campaign_read_back_with_only_this_variable_set("SINGLEFS_CRASH_INJECTION_SEEDS", "0");
    }

    /// 每段步数写 0：重放入口照 `history` 的复现入口报的那一句停下，消息带变量名与原值。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_OPERATIONS=0：只认 正整数")]
    fn a_crash_injection_step_count_of_zero_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_OPERATIONS",
            "0",
        );
    }

    /// 只抽样时每段崩溃点数写 0：重放入口停下，消息带变量名与原值（每段一个崩溃状态都不摆）。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_POINTS=\"0\" 认不出")]
    fn zero_crash_points_with_sampling_only_panics_naming_the_variable_and_the_value() {
        let _campaign =
            campaign_read_back_with_only_this_variable_set("SINGLEFS_CRASH_INJECTION_POINTS", "0");
    }

    /// 短段整段枚举时长段上另抽 0 个：照收，是「只枚举短段」那一种摆法（0 只在只抽样时停下）。
    #[test]
    fn zero_crash_points_beside_whole_short_segments_reads_back_as_short_segments_only() {
        assert_eq!(
            campaign_read_back_with_these_variables_set(&[
                ("SINGLEFS_CRASH_INJECTION_POINTS", "0"),
                ("SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES", "3"),
            ])
            .draw,
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 3,
                sampled_in_longer_segments: 0,
            },
            "长段上另抽 0 个、写数不超过 3 的段整段枚举"
        );
    }

    /// 短段写数上限写 0：重放入口停下，消息带变量名与原值（没有一段够得上，与 `none` 同义却多一种写法）。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES=\"0\" 认不出")]
    fn a_short_segment_write_bound_of_zero_panics_naming_the_variable_and_the_value() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES",
            "0",
        );
    }

    /// 线程数从环境变量取，没设取 `available_parallelism`；设了不是正整数就停，不悄悄退回单线程。
    #[test]
    fn worker_thread_count_comes_from_the_environment_variable_or_available_parallelism() {
        let four = || Ok(std::num::NonZeroUsize::new(4).expect("4"));
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                Some("3"),
                four,
            ),
            CrashInjectionWorkerThreads::FromTheEnvironmentVariable(3)
        );
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                None,
                four
            ),
            CrashInjectionWorkerThreads::FromAvailableParallelism(4)
        );
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                None,
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

    /// 线程数设成 0：经数值环境变量共用的那一份解析停下（线程数取 `NonZeroUsize`，0 认不出），消息带变量名与原值，
    /// 不悄悄退回单线程或本机并行度；口径与仓里别的线程数变量相同。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_THREADS=\"0\" 认不出")]
    fn a_crash_injection_worker_thread_count_of_zero_panics_naming_the_variable_and_the_value() {
        let _threads = CrashInjectionWorkerThreads::from_the_environment_value(
            CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
            Some("0"),
            || Ok(std::num::NonZeroUsize::new(4).expect("4")),
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

    /// 带读故障的步与别的步交错时登记的那几段流：起点发到第 10 步，关掉重挂发到 20，崩溃恢复抛弃根（带读故障）发到 25，零单元发布发到 40。
    fn marks_with_an_abandoning_recovery_sending_stream_steps_15_through_24() -> StreamMarks {
        StreamMarks {
            after_make_filesystem: 4,
            after_the_last_finished_step: 40,
            after_each_step: vec![
                (StepPosition::StartingPoint, 10),
                (StepPosition::Operation(0), 20),
                (StepPosition::Operation(1), 25),
                (StepPosition::Operation(2), 40),
            ],
        }
    }

    /// C670：崩溃窗口末尾那个写之前（含）历史里有一步「崩溃恢复抛弃根」往录制流里发过东西，这个崩溃态才算注入过读故障；
    /// 那一步落在窗口末尾之后的不算。
    #[test]
    fn a_crash_state_counts_as_read_faulted_from_the_first_stream_step_of_an_abandoning_recovery_on(
    ) {
        let step_kinds = [
            HistoryOperationKind::CloseAndMountWritable,
            HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperationKind::PublishWithoutUnits,
        ];
        let marks = marks_with_an_abandoning_recovery_sending_stream_steps_15_through_24();
        assert_eq!(
            marks.read_faults_in_a_crash_state(&step_kinds, 19),
            ReadFaultsInTheCrashState::NoneInjected,
            "窗口末尾落在带读故障那一步发的第一步之前：没注入"
        );
        assert_eq!(
            marks.read_faults_in_a_crash_state(&step_kinds, 20),
            ReadFaultsInTheCrashState::Injected,
            "窗口末尾就是带读故障那一步发的第一步：注入过"
        );
        assert_eq!(
            marks.read_faults_in_a_crash_state(&step_kinds, 35),
            ReadFaultsInTheCrashState::Injected,
            "窗口末尾落在带读故障那一步之后的零单元发布里：注入过"
        );
    }

    /// C670：「崩溃恢复抛弃根」那一步一个都没往录制流里发（取号之前就拒了），盘面与没走这一步的历史一样：之后的崩溃态都不算注入过读故障。
    #[test]
    fn an_abandoning_recovery_that_sent_nothing_leaves_every_crash_state_without_read_faults() {
        let step_kinds = [
            HistoryOperationKind::CloseAndMountWritable,
            HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperationKind::PublishWithoutUnits,
        ];
        let marks = StreamMarks {
            after_make_filesystem: 4,
            after_the_last_finished_step: 40,
            after_each_step: vec![
                (StepPosition::StartingPoint, 10),
                (StepPosition::Operation(0), 20),
                (StepPosition::Operation(1), 20),
                (StepPosition::Operation(2), 40),
            ],
        };
        for last_stream_index_of_the_crash_window in [12, 19, 20, 35] {
            assert_eq!(
                marks.read_faults_in_a_crash_state(&step_kinds, last_stream_index_of_the_crash_window),
                ReadFaultsInTheCrashState::NoneInjected,
                "带读故障那一步一个都没发，窗口末尾在第 {last_stream_index_of_the_crash_window} 步：没注入"
            );
        }
    }

    /// C670 判别力（E171（三oracle合谋测试） 第 242、269 行）：起点（取号、暖机、第一个文件）加一次关掉重挂的历史，短段整段枚举；
    /// 历史里没有带读故障的步，每个崩溃态都判 C670 那一条，第二截一个拒得不合法的都没有、没有新发现。
    /// 系统配置轮换挪到根槽 FUA 之前时（`crates/mutations.tsv` 里「发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前」那条改法），暖机那一段摆得出
    /// 「轮换落了、根没落」：记录不跨实例边界，所选那一版停在上一个实例那一版，见证值比它新，正确实现在这份盘面上永远拒可写——这一条红；
    /// 取号写的见证值取成槽世代号时（「见证值取成槽世代号」那条），重挂取号写落了、暖机的根没落的那几态同样红。
    // crash-case-check:not-a-crash-case 起点加一步的合成历史、短段整段枚举，不调全量枚举，崩溃态数以百计（推的，没量过）
    #[test]
    fn no_crash_state_without_read_faults_after_a_remount_is_refused_writable_for_a_witness_newer_than_the_selected_version(
    ) {
        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::CloseAndMountWritable],
        };
        let injection = inject_crashes_into_history(
            &history,
            HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            },
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 4,
                sampled_in_longer_segments: 2,
            },
            None,
        );
        let tally = &injection.tally;
        assert!(tally.crash_points > 0, "一个崩溃态都没摆出来：{tally:?}");
        assert_eq!(
            tally.writable_mounts_after_the_crash, tally.crash_points,
            "每个崩溃态都起了一次可写挂载：{tally:?}"
        );
        assert_eq!(
            tally.crash_points_after_a_step_reading_under_injected_faults, 0,
            "这段历史没有带读故障的步，每个崩溃态都判 C670 那一条：{tally:?}"
        );
        assert!(
            injection
                .crash_points
                .iter()
                .any(|crash_point| crash_point.step == StepPosition::StartingPoint)
                && injection.crash_points.iter().any(|crash_point| {
                    crash_point.operation_kind == Some(HistoryOperationKind::CloseAndMountWritable)
                }),
            "起点与关掉重挂那一步都要摆到崩溃态：{:?}",
            injection
                .crash_points
                .iter()
                .map(|crash_point| (crash_point.step, crash_point.operation_kind))
                .collect::<Vec<_>>()
        );
        let new_findings: Vec<String> = injection
            .new_findings
            .iter()
            .map(CrashPointFinding::render)
            .collect();
        assert_eq!(
            tally.writable_mount_refusals_on_the_d23_list_illegal, 0,
            "没注入读故障的崩溃态上可写挂载以见证判据为由拒、拒得不合法：{new_findings:?}"
        );
        assert!(
            new_findings.is_empty(),
            "没注入读故障的崩溃态上有新发现：{new_findings:?}"
        );
    }

    /// 起点（取号、暖机、第一个文件）之后一次覆盖写、不重挂的历史，与跑它用的执行（单元区 384 槽的小盘，不逐步跑 checker）。
    fn history_of_the_first_file_and_one_overwrite() -> (GeneratedHistory, HistoryExecution) {
        use singlefs_harness::history::{
            ContentChoice, ContentLength, HistoryDeviceWidth, PerStepChecker,
        };
        (
            GeneratedHistory {
                seed: HistorySeed(0),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations: vec![HistoryOperation::PublishOverwrite(ContentChoice {
                    length: ContentLength::InsideOneDataUnit { selector: 0 },
                    fill_seed: 1,
                })],
            },
            HistoryExecution {
                per_step_checker: PerStepChecker::Skipped,
                device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            },
        )
    }

    /// 那段历史的写表与切段（与 [`inject_crashes_into_history`] 自己切的同一份：同一条执行、同一份几何），和覆盖写那次发布的第一个写在写表里的下标。
    fn writes_segments_and_first_write_of_the_overwrite(
        history: &GeneratedHistory,
        execution: HistoryExecution,
    ) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, usize) {
        let stream = SharedStream::retaining_contents();
        let mut operations_after_the_starting_point = None;
        let run = execute_history_with(history, execution, &stream, &mut |observation| {
            if observation.position == StepPosition::StartingPoint {
                operations_after_the_starting_point = Some(stream.operation_count());
            }
        });
        assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
        let operations_after_the_starting_point =
            operations_after_the_starting_point.expect("起点跑完时登记过一次");
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        let first_write_of_the_overwrite = stream_indexes
            .iter()
            .position(|stream_index| *stream_index >= operations_after_the_starting_point)
            .expect("覆盖写那次发布写过东西");
        (writes, segments, first_write_of_the_overwrite)
    }

    /// 覆盖写那次发布的写里只有系统配置轮换（每块盘一槽）；它还写了记录与根槽（都要扣下）。
    fn assert_the_overwrite_rotates_one_system_configuration_slot_per_device(
        writes_of_the_overwrite: &[RetainedWrite],
    ) {
        assert_eq!(
            writes_of_the_overwrite
                .iter()
                .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
                .count(),
            2,
            "前提：覆盖写那次发布每块盘轮换一个系统配置槽"
        );
        assert!(
            writes_of_the_overwrite
                .iter()
                .any(|write| write.kind == StepKind::RootRecordFua)
                && writes_of_the_overwrite
                    .iter()
                    .any(|write| write.kind == StepKind::JournalRecord),
            "前提：覆盖写那次发布写了记录与根（这个崩溃态里都不落）"
        );
    }

    /// C670 的接线（攻方改法 P2，`research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K2-e）：经崩溃注入的入口摆一个靶向崩溃态——
    /// 第一个文件之后一次覆盖写，那次发布只落了系统配置轮换，单元、记录与根都没落（盘面照 harness 档
    /// `writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 的造法；越过了根槽 FUA 与轮换的先后，今天的持久顺序下段内真子集摆不出来）。
    /// 历史里没有带读故障的步；第二截的可写挂载以判据 N-配置 拒，盘没变、只读读出在允许集合里，U6 四条都满足，C670 那一条判不合法，计 1。
    /// 判别力：调用点把读故障的认定恒交 `Injected`、第二截换回不判 C670 的旧判官 `judge_writable_mount_refusal`（`crates/mutations.tsv` 两行），
    /// 这一拒都判成合法，这一条红。harness 档不许依赖 checker 档，接线在 checker 档，用例只能写在这里。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个崩溃态（ExactlyOneCrashState），不枚举
    #[test]
    fn a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash(
    ) {
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, segments, first_write_of_the_overwrite) =
            writes_segments_and_first_write_of_the_overwrite(&history, execution);
        let writes_of_the_overwrite = &writes[first_write_of_the_overwrite..];
        assert_the_overwrite_rotates_one_system_configuration_slot_per_device(
            writes_of_the_overwrite,
        );
        let segment_index = segments
            .iter()
            .position(|segment| segment[0] == first_write_of_the_overwrite)
            .expect("覆盖写那次发布从一段的第一个写起（它前面是起点末尾那道屏障）");
        let withheld_among_them: Vec<usize> = writes_of_the_overwrite
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind != StepKind::SystemConfigurationSlot)
            .map(|(within_the_overwrite, _)| within_the_overwrite)
            .collect();
        let injection = inject_crashes_into_history(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index,
                writes_from_the_segment_on: writes_of_the_overwrite.len(),
                withheld_among_them: withheld_among_them.leak(),
            },
            None,
        );
        let tally = &injection.tally;
        assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃态");
        assert_eq!(
            tally.crash_points_after_a_step_reading_under_injected_faults,
            0,
            "这段历史没有带读故障的步：{}",
            tally.render()
        );
        assert_eq!(
            tally.model_contents_compared,
            1,
            "前提：只读恢复读出第一个文件那一版，在模型的允许集合里（U6 只读读出那一条满足）：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.writable_mount_refusals_on_the_d23_list_legal,
                tally.writable_mount_refusals_on_the_d23_list_illegal
            ),
            (0, 1),
            "第二截的可写挂载以判据 N-配置 拒、崩溃态没注入读故障而判据复算为真：拒得不合法，计 1：{}",
            tally.render()
        );
    }

    /// C670 在第三截的判法（攻方改法 P1）：把「只落了覆盖写那次轮换」的盘面当成一个二次崩溃镜像交给第三截，没崩过的那一次挂载做成过（`once` 有内容）。
    /// 再挂以判据 N-配置 拒；崩溃态没注入读故障、判据在这份盘面上复算为真：I-8.4 照旧记「再挂被拒」不判，C670 那一条判过一次、不合法一次，
    /// 记一条第三截的发现。判别力：再挂被拒那一支不判 C670、或换回不判 C670 的旧判官（`crates/mutations.tsv` 两行），这一条红。
    #[test]
    fn a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal(
    ) {
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, _segments, first_write_of_the_overwrite) =
            writes_segments_and_first_write_of_the_overwrite(&history, execution);
        let writes_of_the_overwrite = &writes[first_write_of_the_overwrite..];
        assert_the_overwrite_rotates_one_system_configuration_slot_per_device(
            writes_of_the_overwrite,
        );
        let mut second_crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        second_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
        let rotation: Vec<RetainedWrite> = writes_of_the_overwrite
            .iter()
            .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
            .cloned()
            .collect();
        second_crash_image.apply_writes(&rotation);
        let recovered =
            read_recovery_fields(&second_crash_image, ComparedVersion::TheRecoveredVersion);
        let crash_point = CrashPoint {
            segment_index: 0,
            persisted_within_the_segment: vec![false],
            step: StepPosition::StartingPoint,
            operation_kind: None,
        };
        let mut tally = CrashInjectionTally::default();
        let mut findings = FindingsOfOneHistory {
            known_red_hits: Vec::new(),
            new_findings: Vec::new(),
        };
        judge_the_remount_after_a_second_crash(
            &mut tally,
            &mut findings,
            &Ok(content_read_by_the_recovery_report(&recovered.report)),
            &RefusalOnTheCrashImage {
                crash_image_materialized: &second_crash_image,
                version_landed_on_after_the_crash: recovered.report.effective_root,
                read_only_read_back: ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
                read_faults: ReadFaultsInTheCrashState::NoneInjected,
            },
            &execution.device_width.parameters(),
            execution.space_admission,
            WritableMountPhase::RowPublish,
            history.seed,
            &crash_point,
            None,
        );
        assert_eq!(
            tally
                .remount_idempotence_not_judged
                .get(&RemountIdempotenceNotJudged::TheRemountWasRefused)
                .copied(),
            Some(1),
            "前提：再挂被拒，I-8.4 不判：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
            ),
            (1, 1),
            "拒因是判据 N-配置、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
            tally.render()
        );
        assert_eq!(
            findings.known_red_hits.len() + findings.new_findings.len(),
            1,
            "第三截记一条：已知红 {:?}，新发现 {:?}",
            findings.known_red_hits,
            findings
                .new_findings
                .iter()
                .map(CrashPointFinding::render)
                .collect::<Vec<_>>()
        );
    }

    /// C670 在第三截的判别力（攻方改法 P1，`research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K2-b）：起点加一次覆盖写、不重挂，
    /// 短段整段枚举；历史里没有带读故障的步。二次崩溃之后再挂以判据 N-配置 被拒、判据在二次崩溃镜像上复算为真的，一个都没有。
    /// 攻方自造的坏实现 V242u（只在带单元的发布上先轮换后根，`crates/mutations.tsv` 里那一行）在这段历史上第二截看不见（第一次崩溃镜像上 0 个：
    /// 同一实例的记录照重放），第二截那次挂载的写行是新实例第一次带单元的发布，「轮换落了、根没落」只经二次崩溃摆得出来——攻方原型逐个枚举短段量到 108 态；
    /// 这里每个崩溃态每段只抽一个二次崩溃，能不能抽中是推的（报告里估了抽中的概率）。
    // crash-case-check:not-a-crash-case 起点加一步的合成历史、短段整段枚举，不调全量枚举，崩溃态数以百计（推的，没量过）
    #[test]
    fn no_remount_after_a_second_crash_without_read_faults_is_refused_writable_for_a_witness_newer_than_the_selected_version(
    ) {
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let injection = inject_crashes_into_history(
            &history,
            execution,
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 4,
                sampled_in_longer_segments: 2,
            },
            None,
        );
        let tally = &injection.tally;
        assert_eq!(
            tally.crash_points_after_a_step_reading_under_injected_faults,
            0,
            "这段历史没有带读故障的步，每个崩溃态都判 C670 那一条：{}",
            tally.render()
        );
        assert!(
            injection.crash_points.iter().any(|crash_point| {
                matches!(
                    crash_point.operation_kind,
                    Some(HistoryOperationKind::PublishOverwrite)
                )
            }),
            "覆盖写那一步要摆到崩溃态：{}",
            tally.render()
        );
        assert!(
            tally
                .second_crash_points_by_phase
                .get(&WritableMountPhase::RowPublish)
                .copied()
                .unwrap_or(0)
                >= 1,
            "前提：写行那一段摆到了二次崩溃：{}",
            tally.render()
        );
        assert_eq!(
            tally.remount_refusals_with_the_witness_criterion_true_without_read_faults,
            0,
            "二次崩溃之后再挂以见证判据为由拒、崩溃态没注入读故障而判据复算为真（C670）：{:?}",
            injection
                .new_findings
                .iter()
                .filter(|finding| matches!(
                    finding.stage,
                    CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
                ))
                .map(CrashPointFinding::render)
                .collect::<Vec<_>>()
        );
    }
}
