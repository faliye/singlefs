//! 崩溃注入（里程碑「覆盖写、释放、回退与复用」增补 3 第 3 件）：第 1 件的每段历史跑的过程中开着内容保留的录制流，把流按屏障与 FUA 写切段，
//! 按种子摆若干个崩溃状态（一段 + 那一段里持久了哪几个写），对每个崩溃状态重建镜像、跑恢复、跑池级 checker、跑记录核对器，
//! 并问理想模型「恢复到的这一版允不允许」（`singlefs_harness::model::crash_recovery_disagreement`）。
//!
//! 枚举域与崩溃点重放（`crate::crash`）同一个：更早的段整段持久、当前段任意真子集、更晚的段一个都没持久，撕裂并进「没持久」
//! （D13（验证路线） 已定项 4）。差别只在**怎么取**：崩溃点重放在两条写死的流上全枚举，这里在任意一段随机历史上抽样——
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
//! checker、记录核对器）。崩溃点重放的枚举流上也做可写挂载：挂在节点上的可写挂载续做观察者在每个状态上真挂一次、挂完的盘过池级 checker
//! （`tests/common_crash_point_observers/mod.rs`）；mkfs 返回之后的枚举流上另挂两槽全废盘面的观察者（同一份文件），相对流的基镜像
//! 判新出现的、一个自证过的系统配置槽都没有的盘（D13（验证路线） 已定项 6）。这两截是随机历史上抽样摆出的崩溃状态那一份。后两截交给记录核对器的是整条历史录制流接上挂载那一段
//! （与之后那次发布）的录制流（[`GivenStatesTableAfterTheFirstCrash`]，与 [`HistoryThenWritableMountRecords`] 同一种断法），
//! 历史那几次发布在挂载之后的池与二次崩溃镜像上还在不在也核得到；第二截的记录核对器读两份镜像（第一条判据读第一次崩溃那份、
//! 第二条读挂载之后的池，[`ConclusionsAfterTheWritableMount::record_check`]）。
//! 每个二次崩溃镜像上再起一次可写挂载，判 I-8.4（重放幂等）的 R-换：「恢复加写行那一整段」做两次（第一次崩在半途）与做一次，
//! 读出的内容相同（[`compare_content_read_once_and_twice`]；不比扇区、实例代号、实例表行）。
//!
//! 挂载透明（独立的第四条，判法在 `singlefs_harness::mount_transparency`；r5 判决第二节 U5、U6，用户 2026-10-02 定 Y7「崩溃注入里量」、F6「加二次」）：
//! 挂载做成的每个崩溃状态上，崩溃镜像的只读恢复（R1）与只挂载之后、挂载加一次发布之后的只读恢复（R2，后者比发布之前那一版）逐字段比，
//! 挂载途中每个二次崩溃镜像的只读恢复也与 R1 比；可写挂载被拒的，理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的判拒绝本身合不合法。

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::walk::ContainerCapacityKnobs;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::instance_table_sweep::{
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    SweepPublishPlacements,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_instance_table_sweep_configuration, MountError, Mounted, ShadowLedger,
};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
    RecoveryReport,
};
use singlefs_core::transaction::{
    publish_first_file, publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput,
};
use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};

use crate::crash::check_records_against;
use crate::crash_judge_conclusions::BlockCountVector;
use crate::writable_mount_phases::writable_mount_phase_of_each_write;
/// 可写挂载的分段与崩溃镜像落池住在 `crate::writable_mount_phases`（不拿随机历史当输入的用例经那一份引）；崩溃注入的用例照旧从这里引。
pub use crate::writable_mount_phases::{
    materialized_crash_image, publishes_before_the_instance_table_sweep_of_the_mount_output,
    writes_of_the_instance_table_sweep_phases, WritableMountPhase,
};
use singlefs_harness::history::{
    choice_from_reproduction_environment, classify_failure, execute_history_with,
    generate_history_with_weights,
    history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring,
    newest_ring_root_and_slot_count_of, raised_floor_lands_only_on_abandoned_roots,
    record_check_aspects, render_environment_assignments, ContentChoice, ContentLength,
    FailureObservation, FailureSignature, GeneratedHistory, GenerationWeights, HistoryEnding,
    HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySampling, HistorySeed,
    HistoryStartingPoint, ReproductionVariablePrefix, SeededRandomSource, StepPosition,
    KNOWN_RED_FORMS,
};
use singlefs_harness::memory_pool::{
    newest_persisted_root, root_identity_written_by, some_publish_persisted_without_its_root,
    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, RecordCheck,
    RecordStreamContinuity, RecordStreamSeam, RetainedWrite, SparseBlockDevice, SECTOR_BYTES,
};
use singlefs_harness::model::{
    crash_recovery_disagreement, ModelDisagreement, ModelDisagreementAspect, ModelRefusalReason,
    ModelRootKey, ObservedReadBack, ObservedRefusalReason,
};

/// 恢复读回的那一版问理想模型允不允许（`singlefs_harness::model::crash_recovery_disagreement`）：比的是实现跑出来的结局与只住内存的
/// 理想模型，两边都是 Rust 代码在跑，不是判一张镜像，上不了 GPU（豁免表 `.claude/gate.d/gpu-exemptions.tsv` 那一行，崩溃点重放合并
/// 第三轮判决 J5）。随机历史的池级 checker 与两遍恢复那一截上 GPU（给定状态表），这一截不上。
// gpu-exempt: comparison-with-the-ideal-model
fn disagreement_with_the_ideal_model(
    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    newest_persisted_root: Option<ModelRootKey>,
    read_back: &ObservedReadBack,
) -> Option<ModelDisagreement> {
    crash_recovery_disagreement(committed_versions, newest_persisted_root, read_back)
}
use singlefs_harness::heavy_threads::{
    default_heavy_threads_of_the_cap_value, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::model_comparison::{
    model_root_key, observed_read_back_after_a_crash, refusal_reason_of_mount_error,
    refusal_reason_of_publish_error,
};
use singlefs_harness::mount_transparency::{
    judge_mount_transparency,
    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount,
    read_recovery_fields, ComparedVersion, DiskAfterTheRefusal, FieldOutcome,
    FloorRaisedByTheMount, FloorRaisesInTheMountWrites, MountTransparency, MountTransparencyAction,
    ReadFaultsInTheCrashState, ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage,
    RecoveryFieldsRead, RefusalConditionNotMet, RefusalReasonOnTheD23List,
    UserVisibleTreeRootPointersOfTheContinuation, WritableMountRefusalVerdict,
};
use singlefs_harness::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_harness::reference_filesystem_translation::{
    judge_a_second_crash_after_the_restart, JointVerdictOfTheReferenceFileSystemAndTheModel,
    ReferenceFileSystemJudgementOfACrash, ReferenceFileSystemTranslation, ReferenceFileSystemVote,
    TranslatedHistory,
};
use singlefs_harness::scenario::FIXED_WRITE_TIME_SECONDS;
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

/// 崩溃注入的工作线程数从这个环境变量取（十进制正整数）；没设就取每台默认重活线程数
/// （`singlefs_harness::heavy_threads::default_heavy_threads`：`SINGLEFS_THREAD_CAP`，再没设取登记值）。
pub const CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_INJECTION_THREADS";

/// 摆崩溃状态的随机源与生成历史那一路岔开：同一个种子，历史怎么生成不受它影响，反过来也一样。
const CRASH_POINT_SEED_SALT: u64 = 0x63_72_61_73_68_70_74_00;

/// 摆挂载途中的二次崩溃的随机源，与上面两路再岔开（再按第一次崩溃的段号与段内子集岔开：同一段历史上各个崩溃状态抽的不是同一串，
/// [`draw_second_crash_points`]）。
const SECOND_CRASH_POINT_SEED_SALT: u64 = 0x73_65_63_6f_6e_64_63_72;

/// 第一次崩溃那个段内子集的指纹（[`fingerprint_of_the_persisted_subset`]）取 64 位 FNV-1a：起始值。
const PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// 64 位 FNV-1a 的乘数。
const PERSISTED_SUBSET_FINGERPRINT_PRIME: u64 = 0x0000_0100_0000_01b3;

/// 每个工作线程摊到的片数：各段历史的长短差得远（一步就被拒的与连发四十次的），片切得比线程多，先跑完的线程接着领下一片。
const SLICES_PER_WORKER_THREAD: usize = 4;

/// 段内写数不超过这个数时，子集掩码直接用一个 u64 抽；更长的段逐写各抽一次。
const WRITES_A_SUBSET_MASK_HOLDS: usize = 63;

/// 工作线程数是从哪来的：与实际起的线程数一起打进进度行，「机器多于 1 核却只用了 1 个线程」看得出来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashInjectionWorkerThreads {
    /// 环境变量 [`CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    FromTheEnvironmentVariable(usize),
    /// 没设环境变量，取每台默认重活线程数（`singlefs_harness::heavy_threads::default_heavy_threads`）。
    FromTheHeavyThreadDefault(usize),
    /// 调用方在代码里直接给的（用例拿不同线程数对拍）。
    GivenByCaller(usize),
}

impl CrashInjectionWorkerThreads {
    /// 线程数取环境变量，没设就取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取登记值）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数同样停。
    #[must_use]
    pub fn from_the_environment() -> Self {
        Self::from_the_environment_variable_named(
            CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
        )
    }

    /// 线程数取 `variable_name` 这个环境变量，没设就取每台默认重活线程数。
    /// 坏盘输入（增补 3 第 5 件）拿它配自己那个变量名：判定只有「环境变量 → 正整数」这一件事，
    /// 抄一份出来两边会分叉（`code-discipline.md`「重复要生成，不许手抄」）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数同样停。
    #[must_use]
    pub fn from_the_environment_variable_named(variable_name: &str) -> Self {
        Self::from_the_environment_value(
            variable_name,
            process_environment_variable(variable_name).as_deref(),
            process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        )
    }

    /// [`Self::from_the_environment_variable_named`] 的判定本身：环境变量的值、`SINGLEFS_THREAD_CAP` 的值（`None` 是没设）
    /// 都由调用方给（用例不改进程的环境变量）。解析经数值环境变量共用的那一份，线程数取 `NonZeroUsize`：
    /// 写错、写 0 的口径与仓里别的线程数变量相同。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串）：消息带变量名与原值；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
    #[must_use]
    fn from_the_environment_value(
        variable_name: &str,
        value: Option<&str>,
        thread_cap_value: Option<&str>,
    ) -> Self {
        match number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
            variable_name,
            value,
        ) {
            Some(count) => Self::FromTheEnvironmentVariable(count.get()),
            None => Self::FromTheHeavyThreadDefault(
                default_heavy_threads_of_the_cap_value(thread_cap_value).get(),
            ),
        }
    }

    #[must_use]
    pub fn count(self) -> usize {
        match self {
            Self::FromTheEnvironmentVariable(count)
            | Self::FromTheHeavyThreadDefault(count)
            | Self::GivenByCaller(count) => count,
        }
    }

    #[must_use]
    pub fn source_name(self) -> &'static str {
        match self {
            Self::FromTheEnvironmentVariable(_) => "environment_variable",
            Self::FromTheHeavyThreadDefault(_) => "heavy_thread_default",
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

/// 报告与新发现写成文字时，崩溃镜像落在哪的路径写不写进去。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImagePathsInTheText {
    /// 写路径（给人看、回头找现场）。
    Shown,
    /// 不写路径：镜像目录每次调用各不相同（[`create_the_image_directory_of_a_campaign_call`]），两份报告比对时路径不进比对。
    LeftOut,
}

impl CrashPointFinding {
    #[must_use]
    pub fn render(&self) -> String {
        self.render_given(ImagePathsInTheText::Shown)
    }

    /// 同 [`CrashPointFinding::render`]，崩溃镜像落了盘只写一句、不写路径（两份报告比对时用这一份）。
    #[must_use]
    pub fn render_the_conclusions(&self) -> String {
        self.render_given(ImagePathsInTheText::LeftOut)
    }

    /// 两个新发现除了崩溃镜像落在哪的路径之外逐样相同：镜像只比落没落盘（[`CrashInjectionReport::has_the_same_conclusions_as`] 逐个用它）。
    #[must_use]
    pub fn has_the_same_conclusions_as(&self, other: &Self) -> bool {
        let Self {
            signature,
            seed,
            crash_point,
            stage,
            observation,
            read_back,
            image_files,
        } = self;
        *signature == other.signature
            && *seed == other.seed
            && *crash_point == other.crash_point
            && *stage == other.stage
            && *observation == other.observation
            && *read_back == other.read_back
            && image_files.is_some() == other.image_files.is_some()
    }

    fn render_given(&self, image_paths: ImagePathsInTheText) -> String {
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
        match (&self.image_files, image_paths) {
            (Some(directory), ImagePathsInTheText::Shown) => {
                let _ = writeln!(text, "  崩溃镜像留在：{}", directory.display());
            }
            (Some(_), ImagePathsInTheText::LeftOut) => {
                let _ = writeln!(text, "  崩溃镜像落了盘（路径见 render）");
            }
            (None, ImagePathsInTheText::Shown | ImagePathsInTheText::LeftOut) => {}
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
    /// 第一截各状态上逐条不变量判绿（判到而没违例）的次数，从给定状态表的计数向量并进来。
    pub invariant_holds: BTreeMap<&'static str, u64>,
    /// 第一截各状态上逐条不变量没判到的次数：不适用或部分没判（GPU 判器的判到位不分这两样，两个判器同一个数法），不红，也不算判绿。
    pub invariant_not_judged: BTreeMap<&'static str, u64>,
    /// 记录核对器跑过几次、两类各判红几次（与崩溃点重放那一路同一份判据，`crash::check_records_against`）。
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
    /// 挂载做成之后的那一次发布（第一个文件还活着的一版上覆盖写、树表 0 条的一版上发第一个文件、第一个文件已删完或正在删的一版上建一个 inode）：
    /// 做成的、单元区墙、别的理由（判红）。
    pub publishes_after_the_writable_mount: u64,
    pub publishes_after_the_writable_mount_refused_at_the_unit_area_wall: u64,
    pub publishes_after_the_writable_mount_refused: u64,
    /// 挂载与那次发布之后的池上跑的池级 checker 与记录核对器。
    pub checker_runs_after_the_writable_mount: u64,
    pub record_checks_after_the_writable_mount: u64,
    /// 崩溃后镜像上那次可写挂载末尾的清扫发布删了行的次数，与那几次删行之前一共抹了几个头（清扫那两段摆没摆得到二次崩溃按它们看）。
    pub writable_mounts_after_the_crash_deleting_instance_table_rows: u64,
    pub heads_erased_by_the_instance_table_sweep_after_the_crash: u64,
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
    /// 二次崩溃之后再挂被拒、拒因是「读阶段判完之后系统配置又读不出」的（第三截判改法 G4 那一条，与 C670 同形）：判过的、
    /// 崩溃态没注入读故障而那一拒的判据在二次崩溃镜像上复算为真、拒得不合法的（判红）。
    pub remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage: u64,
    pub remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults:
        u64,
    /// 二次崩溃之后再挂被拒、拒因是「重建分配器读根环整环重读一次仍一条不被抛弃的自证有效根都没有」的（第三截判 Q3 那一条，与 G4 同形）：
    /// 判过的、崩溃态没注入读故障而那一拒的判据在二次崩溃镜像上复算为真、拒得不合法的（判红）。
    pub remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator: u64,
    pub remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults:
        u64,
    /// 二次崩溃之后再挂被拒、拒因是「记录有缺口时拒可写」的（D23（journal 的角色与格式） 已定项 14，F1 乙）：判过的次数。这一拒没有 C670 同形的那一条
    /// （判官不列「没注入读故障时判据复算为真」那一类条件），只差 U6 那几条的照别的拒因不报红。
    pub remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify:
        u64,
    /// 二次崩溃之后再挂被拒、拒因是 C 读一次没施加的发布的账读不出、读得出的记录不齐、映射也现算不成的（`C_MAPPING` 收窄）：判过的次数。
    /// 与 F1 乙同样没有 C670 同形的那一条，只差 U6 那几条的照别的拒因不报红。
    pub remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records:
        u64,
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
    /// 挂载透明判的时候 R2 那一版是清扫发布那一版、把清扫删的行放回去再比的次数（C704（挂载透明不认清扫发布）的处置真走到了）。
    pub mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back: u64,
    /// 崩溃后镜像上的可写挂载被拒、理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的：拒得合法的、不合法的（判红）。
    pub writable_mount_refusals_on_the_d23_list_legal: u64,
    pub writable_mount_refusals_on_the_d23_list_illegal: u64,
    /// 崩溃窗口末尾之前历史里有一步在注入的读故障下发过东西的崩溃状态：第二截判拒可写时不判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）
    /// 与改法 G4 那两条。
    pub crash_points_after_a_step_reading_under_injected_faults: u64,
    /// 第一截与二次崩溃那一截上 RefFS 与 O1 判官两票合起来（G1 用户定案：任一红即红），按哪一边红分开计；
    /// RefFS 弃权的按理由另计（两截并在一起）。
    pub reference_filesystem_and_model_verdicts_on_crash_images:
        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
    pub reference_filesystem_and_model_verdicts_on_second_crashes:
        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
    pub reference_filesystem_abstentions: BTreeMap<&'static str, u64>,
    /// 给定状态表里判器交了占位结论的张数（[`GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost`]：两台分片、这张表归别的机器判、
    /// 库里又没有）：不为 0 时这一批的计数偏少、新发现可能漏，报告里另打一行（[`CrashInjectionReport::render`]），入口照「本次未跑」交回。
    pub given_states_tables_left_to_other_hosts: u64,
    /// 给定状态表里判器只录了没判的张数（[`GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly`]：这一趟只跑录入段，
    /// `SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）：交的是占位结论，这一批的计数与新发现不作数，入口照「本次未跑」交回、等判段那一趟。
    pub given_states_tables_recorded_only: u64,
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
            invariant_not_judged,
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
            writable_mounts_after_the_crash_deleting_instance_table_rows,
            heads_erased_by_the_instance_table_sweep_after_the_crash,
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
            remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
            remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults,
            remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
            remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults,
            remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify,
            remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records,
            crash_states_ending_known_red,
            crash_states_ending_new_finding,
            mount_transparency_judged,
            mount_transparency_judged_on_second_crashes_by_phase,
            mount_transparency_not_judged,
            mount_transparency_violations_by_field,
            mount_transparency_fields_not_judged,
            mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back,
            writable_mount_refusals_on_the_d23_list_legal,
            writable_mount_refusals_on_the_d23_list_illegal,
            crash_points_after_a_step_reading_under_injected_faults,
            reference_filesystem_and_model_verdicts_on_crash_images,
            reference_filesystem_and_model_verdicts_on_second_crashes,
            reference_filesystem_abstentions,
            given_states_tables_left_to_other_hosts,
            given_states_tables_recorded_only,
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
        for (invariant, count) in invariant_not_judged {
            *self.invariant_not_judged.entry(invariant).or_insert(0) += count;
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
        self.writable_mounts_after_the_crash_deleting_instance_table_rows +=
            writable_mounts_after_the_crash_deleting_instance_table_rows;
        self.heads_erased_by_the_instance_table_sweep_after_the_crash +=
            heads_erased_by_the_instance_table_sweep_after_the_crash;
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
        self.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage +=
            remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage;
        self.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults +=
            remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults;
        self.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator +=
            remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator;
        self.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults +=
            remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults;
        self.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify +=
            remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify;
        self.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records +=
            remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records;
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
        self.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back +=
            mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back;
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
        self.given_states_tables_left_to_other_hosts += given_states_tables_left_to_other_hosts;
        self.given_states_tables_recorded_only += given_states_tables_recorded_only;
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
            "崩溃后那次可写挂载末尾的清扫发布删了行 {} 次，删行之前一共抹了 {} 个头",
            self.writable_mounts_after_the_crash_deleting_instance_table_rows,
            self.heads_erased_by_the_instance_table_sweep_after_the_crash
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
            "二次崩溃之后再挂被拒、拒因是读阶段判完之后系统配置又读不出的（第三截判 G4 那一条）：{} 次，崩溃态没注入读故障而判据复算为真、拒得不合法 {}",
            self.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
            self.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults
        );
        let _ = writeln!(
            text,
            "二次崩溃之后再挂被拒、拒因是重建分配器读根环整环重读一次仍一条有效根都没有的（第三截判 Q3 那一条）：{} 次，崩溃态没注入读故障而判据复算为真、拒得不合法 {}",
            self.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
            self.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults
        );
        let _ = writeln!(
            text,
            "二次崩溃之后再挂被拒、拒因是记录有缺口时拒可写的（F1 乙）：{} 次",
            self.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify
        );
        let _ = writeln!(
            text,
            "二次崩溃之后再挂被拒、拒因是没施加的发布的账读不出而读得出的记录不齐的（C_MAPPING）：{} 次",
            self.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records
        );
        let _ = writeln!(
            text,
            "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}；把清扫删的行放回去再比的 {} 次",
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
            self.mount_transparency_fields_not_judged,
            self.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back
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
                "  {invariant}：判绿 {} 次、没判到（不适用或部分没判）{} 次",
                self.invariant_holds.get(invariant).copied().unwrap_or(0),
                self.invariant_not_judged
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

/// 每次掉电之后第一个写在写表里的下标，按次序：录制流下标（[`singlefs_harness::history::HistoryRun::earliest_stream_index_after_each_power_loss`]）
/// 换成写表下标，取录制流下标不小于它的第一个写（写表不算屏障，`stream_indexes` 与写表逐条对应）；掉电之后一个写都没录下的取写表长度。
fn first_write_after_each_power_loss_in_the_write_table(
    earliest_stream_index_after_each_power_loss: &[usize],
    stream_indexes: &[usize],
) -> Vec<usize> {
    earliest_stream_index_after_each_power_loss
        .iter()
        .map(|earliest_stream_index_after_the_power_loss| {
            stream_indexes
                .iter()
                .position(|stream_index| {
                    *stream_index >= *earliest_stream_index_after_the_power_loss
                })
                .unwrap_or(stream_indexes.len())
        })
        .collect()
}

/// 写表下标上的几道掉电接缝（[`RecordStreamSeam::AfterAPowerLoss`]），次序照给的。
fn power_loss_seams(first_write_after_each_power_loss: &[usize]) -> Vec<RecordStreamSeam> {
    first_write_after_each_power_loss
        .iter()
        .map(
            |first_write_after_the_power_loss| RecordStreamSeam::AfterAPowerLoss {
                first_write_after_the_power_loss: *first_write_after_the_power_loss,
            },
        )
        .collect()
}

/// 崩溃窗口末尾那个写在录制流里的下标：窗口从写表第 `first_write_of_the_segment` 个写起数 `writes_from_the_segment_on` 个
/// （靶向用例的窗口可以越过段尾），末尾是写表第 `first_write_of_the_segment + writes_from_the_segment_on - 1` 个写。
/// 读故障的认定（[`StreamMarks::read_faults_in_a_crash_state`]）按录制流下标与每一步跑完时流里的步数比；写表下标不算屏障，
/// 流里每过一道屏障两者就差一，拿写表下标去比会把窗口末尾认早（攻方 K2-c，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节）。
///
/// # Panics
/// `writes_from_the_segment_on` 是 0（崩溃窗口里至少有一个写：段内真子集的长度是段的写数，靶向用例的写数由 [`draw_crash_points`] 拒 0），
/// 或窗口末尾越过写表（`stream_indexes` 与写表逐条对应）。
fn stream_index_of_the_last_write_of_the_crash_window(
    stream_indexes: &[usize],
    first_write_of_the_segment: usize,
    writes_from_the_segment_on: usize,
) -> usize {
    let last_write_within_the_crash_window = writes_from_the_segment_on
        .checked_sub(1)
        .expect("崩溃窗口里至少有一个写：段内真子集的长度是段的写数，靶向用例的写数在摆的时候拒 0");
    stream_indexes[first_write_of_the_segment + last_write_within_the_crash_window]
}

/// 这一类历史步在注入的读故障下读盘（`singlefs_harness::history` 的 `DeviceReadingZerosOverHiddenRanges`）：崩溃恢复抛弃根（那次可写挂载期间
/// 最新那条根的根槽与它点名的单元读回全 0），每块盘最新那一槽系统配置读回全 0 的可写挂载（C696（崩溃注入的读故障一支在随机历史里从没触发）：
/// 随机历史里唯一真的在读故障下往盘上发写的一种步）。别的步读的就是盘上的字节。
fn operation_reads_under_injected_faults(kind: HistoryOperationKind) -> bool {
    match kind {
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => true,
        HistoryOperationKind::PublishFirstFile
        | HistoryOperationKind::PublishOverwrite
        | HistoryOperationKind::PublishWithoutUnits
        | HistoryOperationKind::CloseAndMountWritable
        | HistoryOperationKind::RollBackWhileMounted
        | HistoryOperationKind::RaiseRollbackFloor
        | HistoryOperationKind::ColdStartRecover
        | HistoryOperationKind::Unlink
        | HistoryOperationKind::CreateInode
        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
        // 建快照、删快照经会话发、读的是盘上的字节；读快照只读、不经注入的读故障。
        | HistoryOperationKind::CreateSnapshot
        | HistoryOperationKind::ReadSnapshot
        | HistoryOperationKind::DestroySnapshot
        // 挂着时整理经会话发、读的是盘上的字节（区域扫描读源落点）；批间崩溃之后的可写挂载不藏任何一处。
        | HistoryOperationKind::CompactWhileMounted
        // 截断经会话发，读的是盘上的字节（重写尾单元读它的旧净荷）。
        | HistoryOperationKind::Truncate
        // 建 inode、按 inode 号写经会话发，读的是盘上的字节。
        | HistoryOperationKind::CreateInodes
        | HistoryOperationKind::WriteOfAnInode
        // 按偏移写经会话发，读的是盘上的字节（读—改—写读回旧单元、读新槽残留）。
        | HistoryOperationKind::WriteAtOffset => false,
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
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_crashes_into_history_drawing_second_crashes(
        history,
        execution,
        draw,
        SecondCrashPointDraw::OneInEachMountPhaseBySeed,
        image_directory,
        given_states_judge,
    )
}

/// [`inject_crashes_into_history`]，挂载途中的二次崩溃按 `second_crash_draw` 摆（靶向用例经它摆写死的那一个二次崩溃）。
///
/// # Panics
/// 同 [`inject_crashes_into_history`]；`second_crash_draw` 是 [`SecondCrashPointDraw::ExactlyOneSecondCrashState`] 而写死的段号、
/// 写数或扣下的下标在某一次挂载写出的写表上不成立（见那一支）。
#[must_use]
pub fn inject_crashes_into_history_drawing_second_crashes(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    second_crash_draw: SecondCrashPointDraw,
    image_directory: Option<&Path>,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_crashes_into_history_with_every_switch(
        history,
        execution,
        draw,
        second_crash_draw,
        InstanceTableSweepConfiguration::PRODUCT,
        image_directory,
        given_states_judge,
    )
}

/// [`inject_crashes_into_history`]，第一截与第三截上只读一张镜像的那几样（池级 checker、两遍恢复、记录核对器）交 `given_states_judge` 判
/// （给定状态表，崩溃点重放合并第三轮判决 J3；崩溃注入战役经它把判器换成引擎，[`run_crash_injection_campaign_judging_the_given_states_with`]）。
///
/// # Panics
/// 同 [`inject_crashes_into_history`]；`given_states_judge` 判不了某一张表。
#[must_use]
pub fn inject_crashes_into_history_judging_the_given_states_with(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    image_directory: Option<&Path>,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_crashes_into_history_with_every_switch(
        history,
        execution,
        draw,
        SecondCrashPointDraw::OneInEachMountPhaseBySeed,
        InstanceTableSweepConfiguration::PRODUCT,
        image_directory,
        given_states_judge,
    )
}

/// [`inject_crashes_into_history`]，崩溃后镜像上那次可写挂载与每个二次崩溃镜像上的再挂末尾那一判实例表清扫按 `instance_table_sweep`
/// （[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`] 这一份让短历史摆得到清扫那两段；别的入口取产品路径 `InstanceTableSweepConfiguration::PRODUCT`）。
///
/// # Panics
/// 同 [`inject_crashes_into_history`]。
#[must_use]
pub fn inject_crashes_into_history_with_the_instance_table_sweep_configuration(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    instance_table_sweep: InstanceTableSweepConfiguration,
    image_directory: Option<&Path>,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_crashes_into_history_with_every_switch(
        history,
        execution,
        draw,
        SecondCrashPointDraw::OneInEachMountPhaseBySeed,
        instance_table_sweep,
        image_directory,
        given_states_judge,
    )
}

/// [`history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row`] 里实例 2 覆盖写几次。根环 24 槽
/// （`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 每区 8 槽、三区）里实例 1 的根要全被轮出去：实例 2 的写行、两次暖机与崩溃之前的 24 次
/// 覆盖写一共 27 条根；最后一次是摆崩溃状态的那一步。
const OVERWRITES_OF_THE_HISTORY_WHOSE_MOUNT_DELETES_AN_INSTANCE_TABLE_ROW: u64 = 25;

/// 崩溃后那次可写挂载（有一行就扫，[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`]）删得了行的一段历史：第一个文件之后关掉会话、可写挂载成实例 2
/// （写行把实例 1 那一行写进表），再覆盖写 [`OVERWRITES_OF_THE_HISTORY_WHOSE_MOUNT_DELETES_AN_INSTANCE_TABLE_ROW`] 次——根环里实例 1 的根
/// 全被轮出去，根环每一槽都是自证过的根，实例 1 那一行是候选行（D18（块里携带什么信息） 已定项 11「回收」一条）。每次发布都做完了，
/// 盘上没有按实例 1 那一行判未发布的单元：清扫删行、一个头都不抹。
#[must_use]
pub fn history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row(
) -> GeneratedHistory {
    let mut operations = vec![HistoryOperation::CloseAndMountWritable];
    operations.extend(
        (0..OVERWRITES_OF_THE_HISTORY_WHOSE_MOUNT_DELETES_AN_INSTANCE_TABLE_ROW).map(|overwrite| {
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: overwrite,
                },
                fill_seed: overwrite,
            })
        }),
    );
    GeneratedHistory {
        seed: HistorySeed(3),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    }
}

/// 在 [`history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row`] 最后一步的开头摆一个崩溃状态（更早的写全落、
/// 最后一步一个写都没落），崩溃后那次可写挂载与每个二次崩溃镜像上的再挂有一行就扫（[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`]）。
/// 最后一步的第一个写落在第几段随历史的写数变：先把这段历史跑一遍、按同一条切段规则取出来，不写死段号。
///
/// # Panics
/// 历史没跑到最后一步；含最后一步第一个写的那一段里，段首到它之前有 discard 以外的写（上一步发布返回之前那道屏障把上一步的写与它隔开，
/// 隔不开的只有上一步发布返回之后发的 discard：它们与最后一步之间没有屏障，切段把它们并进同一段；有别的写说明切段变了，
/// 「最后一步一个写都没落」那个状态摆不准）。
#[must_use]
pub fn inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
    execution: HistoryExecution,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_a_crash_at_the_start_of_the_last_step_sweeping_on_any_row(
        &history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row(),
        execution,
        given_states_judge,
    )
}

/// 在 [`history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`] 最后一步的开头摆一个崩溃状态（摆法同
/// [`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]）：那段历史开头那次覆盖写掉电在
/// journal 记录之前、留下实例 1 的孤儿，之后实例 2 把实例 1 的根全轮出根环；崩溃后那次可写挂载（有一行就扫）末尾的清扫先抹孤儿的头、
/// 再发清扫发布删实例 1 那一行，抹头那一段（D18（块里携带什么信息） 已定项 11「回收」一条）的二次崩溃从它来。
///
/// # Panics
/// 同 [`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]。
#[must_use]
pub fn inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
    execution: HistoryExecution,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_a_crash_at_the_start_of_the_last_step_sweeping_on_any_row(
        &history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring(),
        execution,
        given_states_judge,
    )
}

/// 在 `history` 最后一步的开头摆一个崩溃状态（更早的写全落、最后一步一个写都没落），崩溃后那次可写挂载与每个二次崩溃镜像上的再挂
/// 有一行就扫（[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`]）。最后一步的第一个写落在第几段随历史的写数变：先把这段历史跑一遍、
/// 按同一条切段规则取出来，不写死段号。
///
/// 上一步发布返回之后核心层发的 discard（整段可再分配的那几段）与最后一步之间没有屏障（核心层 discard 之后不补屏障；补不补连着
/// D26（后台整理与放置回收） 未定项 9，没定，这里照今天的写流摆），切段把它们与最后一步的写并成一段，最后一步的第一个写就不是段首。这里不要求它是段首：找含它的那一段，
/// 段首到它之前那几个 discard 算更早的写、持久，它扣下，它之后的一个都不持久。没有这样的 discard 时它就是段首，摆法不变。
///
/// # Panics
/// 历史没跑到最后一步；含最后一步第一个写的那一段里，段首到它之前有 discard 以外的写。
fn inject_a_crash_at_the_start_of_the_last_step_sweeping_on_any_row(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    let step_before_the_last = StepPosition::Operation(history.operations.len() - 2);
    let stream = SharedStream::retaining_contents();
    let mut operations_before_the_last_step = None;
    let _run = execute_history_with(history, execution, &stream, &mut |observation| {
        if observation.position == step_before_the_last {
            operations_before_the_last_step = Some(stream.operation_count());
        }
    });
    let operations_before_the_last_step = operations_before_the_last_step.expect(
        "这段历史的每一步都做得成（覆盖写落在 4 GiB 的盘上），倒数第二步之后观察者一定被调过",
    );
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let first_write_of_the_last_step = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= operations_before_the_last_step)
        .expect("最后一步（覆盖写）写过东西");
    let segment_index = segments
        .iter()
        .position(|segment| segment.contains(&first_write_of_the_last_step))
        .expect("切段时录到的每个写都恰好落在一段里");
    let position_of_the_first_write_of_the_last_step = segments[segment_index]
        .iter()
        .position(|write| *write == first_write_of_the_last_step)
        .expect("这一段是按含它挑出来的");
    assert!(
        segments[segment_index][..position_of_the_first_write_of_the_last_step]
            .iter()
            .all(|write| writes[*write].kind == StepKind::Discard),
        "段首到最后一步的第一个写之间只许是上一步发布返回之后发的 discard：上一步发布返回之前那道屏障把上一步别的写与它隔开"
    );
    inject_crashes_into_history_with_the_instance_table_sweep_configuration(
        history,
        execution,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index,
            writes_from_the_segment_on: position_of_the_first_write_of_the_last_step + 1,
            withheld_among_them: vec![position_of_the_first_write_of_the_last_step].leak(),
        },
        INSTANCE_TABLE_SWEPT_ON_ANY_ROW,
        None,
        given_states_judge,
    )
}

/// 崩溃注入的全部开关一处收：第一次崩溃怎么摆、二次崩溃怎么摆、挂载末尾的实例表清扫用哪一份配置、给定状态表交谁判。
fn inject_crashes_into_history_with_every_switch(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    second_crash_draw: SecondCrashPointDraw,
    instance_table_sweep: InstanceTableSweepConfiguration,
    image_directory: Option<&Path>,
    given_states_judge: &dyn GivenStatesJudge,
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
    let first_write_after_each_power_loss = first_write_after_each_power_loss_in_the_write_table(
        &run.earliest_stream_index_after_each_power_loss,
        &stream_indexes,
    );
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
    // 第一截只读一张镜像的那几样交给定状态表（基线是 mkfs 之后的池，用户 2026-10-10 定）：一段历史上的崩溃状态先一起判完，下面逐个取
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        marks.after_make_filesystem,
        execution,
    );
    let first_crash_conclusions = judge_the_first_crashes_on_given_states_tables(
        given_states_judge,
        history.seed,
        &history_after_the_make_filesystem,
        &crash_points,
        &segments,
        writes.len(),
        &first_write_after_each_power_loss,
        &mut tally,
    );

    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    let mut writes_applied_to_the_base = 0usize;
    let mut findings = FindingsOfOneHistory {
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
    };
    for (crash_point, first_crash_conclusion) in crash_points.iter().zip(&first_crash_conclusions) {
        let first_write_of_the_segment = segments[crash_point.segment_index][0];
        let writes_in_the_segment = crash_point.persisted_within_the_segment.len();
        // 段号从小到大，基线只往前叠：更早的段整段持久，按写表整段施加，不为每个崩溃状态从头重建。
        base.apply_writes(&writes[writes_applied_to_the_base..first_write_of_the_segment]);
        writes_applied_to_the_base = first_write_of_the_segment;
        let persisted = persisted_over_the_whole_write_table(
            writes.len(),
            first_write_of_the_segment,
            &crash_point.persisted_within_the_segment,
        );
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
            disagreement_with_the_ideal_model(&committed_versions, newest_persisted, &read_back);
        // RefFS 那一票：崩溃时刻按切开当前段的那道屏障由哪一步发出归属（r6 用户定案 H1-收严，只给 RefFS 用；
        // O1 判官与 `crash_point.step` 照旧按段里第一个被扣下的写）。当前段取持久窗口最后一个写所在那一段：段内的崩溃状态就是
        // `segment_index` 那一段；靶向用例越过段尾的那一个，崩在窗口末尾那一段里（窗口里更晚的写都可能已落）。
        let segment_the_crash_closes = segment_containing_the_write(
            &segments,
            first_write_of_the_segment + writes_in_the_segment - 1,
        );
        let reference_filesystem_judgement = reference_filesystem_judgement_of_a_crash_image(
            &reference_filesystem_history,
            segment_the_crash_closes,
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
        // 记录核对器与池级 checker 在上面那张给定状态表上判过（judge_the_first_crashes_on_given_states_tables）：核的是 mkfs 之后这段历史的
        // 整条记录流——更早的段里发出的单元写与 journal 记录写要算进一次发布里；恢复落到由 journal 记录重建的一版时，那一版的根槽写在更晚的段里，
        // 只给到这一段为止的前缀就找不到它、读不出它的实例表，被它抛弃的发布的豁免整条落空（D23（journal 的角色与格式） 已定项 15）。
        // 这个崩溃状态之前掉过电的，在每道掉电接缝处断开（[`RecordStreamSeam::AfterAPowerLoss`]）；第二、三截照这几道接缝接上挂载。
        let first_write_after_each_power_loss_before_the_crash: Vec<usize> =
            first_write_after_each_power_loss
                .iter()
                .copied()
                .filter(|first_write_after_the_power_loss| {
                    *first_write_after_the_power_loss <= first_write_of_the_segment
                })
                .collect();
        let record_check = first_crash_conclusion.record_check();
        tally.record_checks += 1;
        if record_check.root_without_record {
            tally.record_root_without_record += 1;
        }
        if record_check.claimed_state_missing_unit {
            tally.record_claimed_state_missing_unit += 1;
        }
        tally.checker_runs += 1;
        let mut violations = first_crash_conclusion.pool_checker_violations();
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
            stream_index_of_the_last_write_of_the_crash_window(
                &stream_indexes,
                first_write_of_the_segment,
                writes_in_the_segment,
            ),
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
        let mount_run =
            mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the_crash_image(
                &crash_image_materialized,
                &parameters,
                execution,
                history.seed,
                crash_point,
                instance_table_sweep,
            );
        if let Some(rows_deleted) = &mount_run.rows_deleted_by_the_instance_table_sweep {
            tally.writable_mounts_after_the_crash_deleting_instance_table_rows += 1;
            tally.heads_erased_by_the_instance_table_sweep_after_the_crash +=
                u64::try_from(rows_deleted.heads_erased).expect("抹的头数装得进 u64");
        }
        let (mount_writes, mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(&mount_run.mount_operations, &geometry);
        let (
            writes_of_the_publish_after_the_mount,
            segments_of_the_publish_after_the_mount,
            _publish_stream_indexes,
        ) = writes_and_segments_with_stream_indexes(&mount_run.publish_operations, &geometry);
        let records_across_the_crash = HistoryThenWritableMountRecords::new(
            &writes,
            &persisted,
            &first_write_after_each_power_loss_before_the_crash,
            report.effective_root,
            &mount_writes,
            &writes_of_the_publish_after_the_mount,
        );
        let given_states_of_the_mount = GivenStatesAfterTheFirstCrash::new(
            given_states_judge,
            &history_after_the_make_filesystem,
            &persisted,
            &first_write_after_each_power_loss_before_the_crash,
            report.effective_root,
            &writes_of_the_publish_after_the_mount,
            &segments_of_the_publish_after_the_mount,
        );
        // 第二截经库（用户 2026-10-13 定「能走两段的都走两段」）：挂载与之后那一次发布之后的盘与第一次崩溃那份镜像当一张表的两个给定状态
        // 写进库、从库里读回，池级 checker 与记录核对器交判器判（配了 GPU 走 GPU）；记录核对器两份镜像各判一条（D13（验证路线） 已定项 7，
        // 用户 2026-10-13 定「过渡后补回两份镜像」）。可写挂载与那次发布本身真跑实现、只能在 CPU 上（上面那一次）
        let conclusions_after_the_mount = given_states_of_the_mount
            .judge_the_pool_after_the_writable_mount(
                &mount_writes,
                &mount_segments,
                format!(
                    "seed={}/after-the-mount/segment={}/first-crash-fingerprint={:016x}",
                    history.seed.0,
                    crash_point.segment_index,
                    fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment)
                ),
            );
        count_a_table_left_to_another_host(&mut tally, conclusions_after_the_mount.source);
        tally.record_checks_after_the_writable_mount += 1;
        let record_check_after_the_mount = conclusions_after_the_mount.record_check();
        judge_the_writable_mount_on_the_crash_image(
            &mut tally,
            &mut findings,
            &mount_run,
            record_check_after_the_mount,
            conclusions_after_the_mount.violations_on_the_pool_after_the_mount(),
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
        // 挂载透明的射程只管不带故障的状态（D13（验证路线） 已定项 1）：注入过读故障的崩溃态不判，按理由记一次没判。
        match read_faults_in_the_crash_state {
            ReadFaultsInTheCrashState::NoneInjected => judge_mount_transparency_on_the_crash_image(
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
            ),
            ReadFaultsInTheCrashState::Injected => {
                *tally
                    .mount_transparency_not_judged
                    .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
                    .or_insert(0) += 1;
            }
        }
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
                publishes_before_the_instance_table_sweep: mount_run
                    .publishes_before_the_instance_table_sweep,
                parameters: &parameters,
                space_admission: execution.space_admission,
                instance_table_sweep,
                rows_deleted_by_the_sweep_of_the_uninterrupted_mount: mount_run
                    .rows_deleted_by_the_instance_table_sweep
                    .as_ref(),
                reference_filesystem_at_the_first_crash: &reference_filesystem_judgement,
                read_faults_in_the_first_crash_state: read_faults_in_the_crash_state,
                second_crash_draw,
                given_states: given_states_of_the_mount,
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

// ============================== 给定状态表（崩溃点重放合并第三轮判决 J3、J5、J6；用户 2026-10-10 定的四件） ==============================
/// 给定状态表的形态、结论、判器 trait 与 CPU 判法住在 `crate::crash_given_states`（流水线经那一份引，不经崩溃注入）；产品实现是
/// 崩溃点重放的引擎 `crate::crash_amplification::GivenStatesEngine`（表先写进给定状态表那一类的判定库、读回再判、结论存库复用）；用例照旧从这里引。
pub use crate::crash_given_states::{
    count_vector_of_the_given_states, judge_a_given_state_on_the_cpu,
    judge_the_given_states_on_the_cpu, ClaimedVersionOfAGivenState, ClaimedVersionsOfTheStates,
    GivenStateConclusion, GivenStatesConclusionsOfATable, GivenStatesConclusionsSource,
    GivenStatesJudge, GivenStatesOfATable, GivenStatesOfATableWithItsColumns,
    POOL_CHECKER_VIOLATION_WITHOUT_ITS_DETAIL,
};

/// 判器交的是占位结论就记一张：两台分片、这张表归别的机器判（[`CrashInjectionTally::given_states_tables_left_to_other_hosts`]），
/// 或这一趟只跑录入段（[`CrashInjectionTally::given_states_tables_recorded_only`]）。
fn count_a_table_left_to_another_host(
    tally: &mut CrashInjectionTally,
    source: GivenStatesConclusionsSource,
) {
    match source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost => {
            tally.given_states_tables_left_to_other_hosts += 1;
        }
        GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => {
            tally.given_states_tables_recorded_only += 1;
        }
    }
}

/// 一张表的结论并进崩溃注入的计数：逐条不变量判绿（判到减违例）、没判到（不适用或部分没判）各几次。
fn absorb_the_pool_checker_counts_of_the_table(
    tally: &mut CrashInjectionTally,
    count_vector: &BlockCountVector,
) {
    for (invariant, counts) in &count_vector.per_invariant {
        let invariant = singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .find(|candidate| **candidate == invariant.as_str())
            .expect(
                "计数向量按 IMPLEMENTED_INVARIANTS 逐条一行（StateCountsOfABlock::into_count_vector）；判定库读回的在同一把版本下存，清单相同",
            );
        *tally.invariant_holds.entry(invariant).or_insert(0) += counts.judged - counts.violated;
        *tally.invariant_not_judged.entry(invariant).or_insert(0) += counts.not_applicable;
    }
}

/// mkfs 写了写表的前几个写：录制流下标在 `after_make_filesystem` 之前的那几个（写表与 `stream_indexes` 逐条对应、按录制流次序）。
fn writes_of_the_make_filesystem(stream_indexes: &[usize], after_make_filesystem: usize) -> usize {
    stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= after_make_filesystem)
        .unwrap_or(stream_indexes.len())
}

/// 写表从第 `first_write` 个起的那一截的切段（下标减去 `first_write`）：整段在它之前的不要，跨过它的只留它之后那一部分
/// （跨过它的段不是候选段，上面不摆状态；留下的那一部分只为让每个写都在一段里）。
fn segments_from_the_write(segments: &[Vec<usize>], first_write: usize) -> Vec<Vec<usize>> {
    segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .filter(|write| **write >= first_write)
                .map(|write| write - first_write)
                .collect::<Vec<usize>>()
        })
        .filter(|segment| !segment.is_empty())
        .collect()
}

/// 整条写表上一个崩溃状态的持久集合：段首之前全持久，从段首起照 `persisted_within_the_segment`（靶向用例可以越过段尾），之后都没持久。
fn persisted_over_the_whole_write_table(
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

/// 给定状态表的基线与写表（用户 2026-10-10 定「基线改成 mkfs 之后」；同日弹窗定创世那几次发布照样进记录核对器，「先查 GPU 能不能核，
/// 不能就 CPU 核一遍」，查下来 GPU 核得到）：基线是空盘叠上 mkfs 的全部写；写表从 mkfs 第一个不是清零的写起——mkfs 先清零、再写单元、
/// 根槽与系统配置，清零只在基线里（GPU 的落点模型不收清零写），mkfs 那几个数据写留在写表的头上，每个给定状态里都落了。
/// 它们叠在基线上一个字节都不改（清零都在它们之前，每个字节上最后一次写还是基线里那一次）；留在写表里是为了让创世那几次发布进两侧
/// 记录核对器的发布表：发布表按写表下标记单元副本与根槽写，基线里的写没有下标，GPU 那一侧就核不到创世发布的单元。
pub struct HistoryAfterTheMakeFilesystem<'history> {
    base: MemoryPool,
    writes: &'history [RetainedWrite],
    segments: Vec<Vec<usize>>,
    /// 整条写表的第几个写是这张写表的第 0 个：mkfs 的清零写有几个（只叠进基线）。
    first_write_of_the_table: usize,
    /// 这张写表的头几个写是 mkfs 写的（每个给定状态里都落了）。
    writes_of_the_make_filesystem_in_the_table: usize,
}

impl<'history> HistoryAfterTheMakeFilesystem<'history> {
    /// 空盘叠上 mkfs 的写就是基线；写表从 mkfs 第一个不是清零的写起（`writes` 是整条写表，与 `stream_indexes` 逐条对应，
    /// `after_make_filesystem` 是 mkfs 写完时录制流的下标）。
    ///
    /// # Panics
    /// mkfs 的清零写不都在它的数据写之前（叠回去的数据写会盖掉清零，基线就不再是这张写表在全落时的样子）。
    #[must_use]
    pub fn of(
        writes: &'history [RetainedWrite],
        segments: &[Vec<usize>],
        stream_indexes: &[usize],
        after_make_filesystem: usize,
        execution: HistoryExecution,
    ) -> Self {
        let writes_of_the_make_filesystem =
            writes_of_the_make_filesystem(stream_indexes, after_make_filesystem);
        let first_write_of_the_table = writes[..writes_of_the_make_filesystem]
            .iter()
            .position(|write| write.kind != StepKind::ZeroFill)
            .unwrap_or(writes_of_the_make_filesystem);
        assert!(
            writes[first_write_of_the_table..writes_of_the_make_filesystem]
                .iter()
                .all(|write| write.kind != StepKind::ZeroFill),
            "mkfs 先发清零、再发数据写（make_filesystem 的次序）：清零之后再没有清零"
        );
        let mut base = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        base.apply_writes(&writes[..writes_of_the_make_filesystem]);
        Self {
            base,
            writes: &writes[first_write_of_the_table..],
            segments: segments_from_the_write(segments, first_write_of_the_table),
            first_write_of_the_table,
            writes_of_the_make_filesystem_in_the_table: writes_of_the_make_filesystem
                - first_write_of_the_table,
        }
    }

    #[must_use]
    pub fn base_pool(&self) -> &MemoryPool {
        &self.base
    }

    /// 这张写表：mkfs 的数据写在头上，接着是 mkfs 之后的历史。
    #[must_use]
    pub fn writes(&self) -> &'history [RetainedWrite] {
        self.writes
    }

    /// 这张写表的切段（下标按这张写表）。
    #[must_use]
    pub fn segments(&self) -> &[Vec<usize>] {
        &self.segments
    }

    /// 这张写表头上 mkfs 写的那几个（创世那几次发布的单元写、根槽写与系统配置槽写）。
    #[must_use]
    pub fn writes_of_the_make_filesystem(&self) -> &'history [RetainedWrite] {
        &self.writes[..self.writes_of_the_make_filesystem_in_the_table]
    }

    /// 整条写表的下标换成这张写表的下标。
    ///
    /// # Panics
    /// 那个写是 mkfs 的清零写（不在这张写表里）。
    #[must_use]
    pub fn index_in_the_table(&self, index_over_the_whole_write_table: usize) -> usize {
        index_over_the_whole_write_table
            .checked_sub(self.first_write_of_the_table)
            .expect("清零写只在基线里：问到的写在 mkfs 的清零写之后")
    }

    /// 整条写表上的一个持久集合换成这张写表的。
    ///
    /// # Panics
    /// mkfs 的写在这个持久集合里有一个没落（崩溃状态只摆在整段落在 mkfs 之后的段上，`draw_crash_points` 的候选段）。
    #[must_use]
    pub fn persisted_set_of_the_table(
        &self,
        persisted_over_the_whole_write_table: &[bool],
    ) -> Vec<bool> {
        assert!(
            persisted_over_the_whole_write_table
                [..self.first_write_of_the_table + self.writes_of_the_make_filesystem_in_the_table]
                .iter()
                .all(|is_persisted| *is_persisted),
            "崩溃状态摆在整段落在 mkfs 之后的段上：mkfs 的写都已落"
        );
        persisted_over_the_whole_write_table[self.first_write_of_the_table..].to_vec()
    }

    /// 整条写表下标上的掉电之后第一个写换成这张写表上的接缝。
    ///
    /// # Panics
    /// 那次掉电在 mkfs 写完之前（mkfs 不掉电：随机历史的掉电是历史里一步，在起点之后）。
    fn power_loss_seams_of_the_table(
        &self,
        first_write_after_each_power_loss: &[usize],
    ) -> Vec<RecordStreamSeam> {
        power_loss_seams(
            &first_write_after_each_power_loss
                .iter()
                .map(|first_write_after_the_power_loss| {
                    assert!(
                        *first_write_after_the_power_loss
                            >= self.first_write_of_the_table
                                + self.writes_of_the_make_filesystem_in_the_table,
                        "掉电是历史里的一步，在 mkfs 写完之后"
                    );
                    self.index_in_the_table(*first_write_after_the_power_loss)
                })
                .collect::<Vec<usize>>(),
        )
    }
}

/// 第一截：这段历史上每个第一次崩溃状态的池级 checker、两遍恢复、记录核对器交 `judge` 判，掉电接缝相同的几个状态一张表
/// （记录核对器照每个状态之前掉过的电断开，第一次崩溃之前没掉过电的是一条录制流）。交回与 `crash_points` 同序的结论；
/// 计数（判绿、没判到）并进 `tally`。判不了就 panic：配了 GPU 卡用不上、判定库读写失败都判红，不退回 CPU。
#[allow(
    clippy::too_many_arguments,
    reason = "判器、历史、崩溃状态、整条写表的切段与掉电接缝、计数，各是各的东西"
)]
fn judge_the_first_crashes_on_given_states_tables(
    judge: &dyn GivenStatesJudge,
    seed: HistorySeed,
    history_after_the_make_filesystem: &HistoryAfterTheMakeFilesystem<'_>,
    crash_points: &[CrashPoint],
    segments: &[Vec<usize>],
    write_count: usize,
    first_write_after_each_power_loss: &[usize],
    tally: &mut CrashInjectionTally,
) -> Vec<GivenStateConclusion> {
    let mut crash_points_by_power_losses: BTreeMap<Vec<usize>, Vec<usize>> = BTreeMap::new();
    for (crash_point_index, crash_point) in crash_points.iter().enumerate() {
        let first_write_of_the_segment = segments[crash_point.segment_index][0];
        let power_losses_before_the_crash: Vec<usize> = first_write_after_each_power_loss
            .iter()
            .copied()
            .filter(|first_write_after_the_power_loss| {
                *first_write_after_the_power_loss <= first_write_of_the_segment
            })
            .collect();
        crash_points_by_power_losses
            .entry(power_losses_before_the_crash)
            .or_default()
            .push(crash_point_index);
    }
    let mut conclusions: Vec<Option<GivenStateConclusion>> = vec![None; crash_points.len()];
    for (power_losses, crash_point_indexes) in crash_points_by_power_losses {
        let seams = history_after_the_make_filesystem.power_loss_seams_of_the_table(&power_losses);
        let persisted_sets: Vec<Vec<bool>> = crash_point_indexes
            .iter()
            .map(|crash_point_index| {
                let crash_point = &crash_points[*crash_point_index];
                history_after_the_make_filesystem.persisted_set_of_the_table(
                    &persisted_over_the_whole_write_table(
                        write_count,
                        segments[crash_point.segment_index][0],
                        &crash_point.persisted_within_the_segment,
                    ),
                )
            })
            .collect();
        let table = GivenStatesOfATable {
            table_name: format!("seed={}/first-crash/power-losses={power_losses:?}", seed.0),
            base: &history_after_the_make_filesystem.base,
            writes: history_after_the_make_filesystem.writes,
            segments: &history_after_the_make_filesystem.segments,
            seams: &seams,
            persisted_sets,
        };
        let judged = judge
            .judge_the_given_states(&table)
            .unwrap_or_else(|reason| {
                panic!(
                    "第一截给定状态表 {} 判不了（不退回别处判）：{reason}",
                    table.table_name
                )
            });
        assert_eq!(
            judged.states.len(),
            crash_point_indexes.len(),
            "判器逐状态交回结论，与表里的状态同序"
        );
        absorb_the_pool_checker_counts_of_the_table(tally, &judged.count_vector);
        count_a_table_left_to_another_host(tally, judged.source);
        for (crash_point_index, conclusion) in crash_point_indexes.into_iter().zip(judged.states) {
            conclusions[crash_point_index] = Some(conclusion);
        }
    }
    conclusions
        .into_iter()
        .map(|conclusion| conclusion.expect("每个崩溃状态恰好落在一组掉电接缝里"))
        .collect()
}

/// 第二、三截那几张给定状态表要的东西（一个第一次崩溃状态共一份）：mkfs 之后的历史、它在这个第一次崩溃状态上的持久集合与掉电接缝、
/// 第一次崩溃之后只读恢复落到的那一版、挂载之后那一次发布的写与切段，以及交谁判。表怎么建见 [`GivenStatesTableAfterTheFirstCrash`]。
pub struct GivenStatesAfterTheFirstCrash<'history> {
    judge: &'history dyn GivenStatesJudge,
    history_after_the_make_filesystem: &'history HistoryAfterTheMakeFilesystem<'history>,
    persisted_history_of_the_table: Vec<bool>,
    power_loss_seams_of_the_table: Vec<RecordStreamSeam>,
    version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
    writes_of_the_publish_after_the_mount: &'history [RetainedWrite],
    segments_of_the_publish_after_the_mount: &'history [Vec<usize>],
}

impl<'history> GivenStatesAfterTheFirstCrash<'history> {
    /// `persisted_over_the_whole_history` 是这个第一次崩溃状态的持久集合，与整条历史写表（mkfs 的清零写也算）逐条对应；
    /// `first_write_after_each_power_loss_before_the_crash` 是第一次崩溃之前每次掉电之后第一个写在整条历史写表里的下标，从小到大
    /// （没掉过电的给空表）；`segments_of_the_publish_after_the_mount` 按之后那一次发布自己那段写表的下标。
    ///
    /// # Panics
    /// mkfs 的写在持久集合里有一个没落（[`HistoryAfterTheMakeFilesystem::persisted_set_of_the_table`]）；一道掉电接缝落在 mkfs 写完之前。
    #[must_use]
    pub fn new(
        judge: &'history dyn GivenStatesJudge,
        history_after_the_make_filesystem: &'history HistoryAfterTheMakeFilesystem<'history>,
        persisted_over_the_whole_history: &[bool],
        first_write_after_each_power_loss_before_the_crash: &[usize],
        version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
        writes_of_the_publish_after_the_mount: &'history [RetainedWrite],
        segments_of_the_publish_after_the_mount: &'history [Vec<usize>],
    ) -> Self {
        Self {
            judge,
            history_after_the_make_filesystem,
            persisted_history_of_the_table: history_after_the_make_filesystem
                .persisted_set_of_the_table(persisted_over_the_whole_history),
            power_loss_seams_of_the_table: history_after_the_make_filesystem
                .power_loss_seams_of_the_table(first_write_after_each_power_loss_before_the_crash),
            version_landed_on_after_the_first_crash,
            writes_of_the_publish_after_the_mount,
            segments_of_the_publish_after_the_mount,
        }
    }

    /// 第二截那张表：挂载与之后那一次发布之后的池（实现恢复后的镜像：挂载那一段与之后那次发布全落）与第一次崩溃那份镜像（崩溃态镜像：
    /// 挂载那一段与之后那次发布一个写都没落）是这张表的两个状态，次序照 [`StateOfTheTableAfterTheWritableMount::ALL`]。
    /// 两个状态在历史那一截上的持久集合相同，记录核对器两份镜像各判一条（[`ConclusionsAfterTheWritableMount::record_check`]）。
    /// 挂载之后的池那个状态那一栏恢复自称的那一版写挂载与那次发布写出的最新那条根（[`newest_root_written_after_the_crash`]，D13（验证路线）
    /// 已定项 7；用户 2026-10-14 弹窗定「照字面改：表里记下根的来路」），判段照它判第二条判据、与恢复落到的根对不上判红；第一次崩溃那份镜像那个状态不写。
    #[must_use]
    pub fn table_after_the_writable_mount(
        &self,
        mount_writes: &[RetainedWrite],
        mount_segments: &[Vec<usize>],
        table_name: String,
    ) -> GivenStatesTableAfterTheFirstCrash<'history> {
        let states: Vec<(Vec<bool>, PublishAfterTheMountInTheState)> =
            StateOfTheTableAfterTheWritableMount::ALL
                .iter()
                .map(|state| match state {
                    StateOfTheTableAfterTheWritableMount::PoolAfterTheMount => (
                        vec![true; mount_writes.len()],
                        PublishAfterTheMountInTheState::Landed,
                    ),
                    StateOfTheTableAfterTheWritableMount::FirstCrashImage => (
                        vec![false; mount_writes.len()],
                        PublishAfterTheMountInTheState::NotWrittenYet,
                    ),
                })
                .collect();
        let mut table = self.table_of_the_states(mount_writes, mount_segments, &states, table_name);
        let history_write_count = self.history_after_the_make_filesystem.writes.len();
        table.claimed_versions
            [StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position()] =
            ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(
                newest_root_written_after_the_crash(
                    &table.writes[history_write_count..],
                    self.version_landed_on_after_the_first_crash,
                ),
            );
        table
    }

    /// 第二截：[`Self::table_after_the_writable_mount`] 那张表交判器判，交回两个状态各自的结论。判不了就 panic
    /// （同 [`judge_the_first_crashes_on_given_states_tables`]）。
    #[must_use]
    pub fn judge_the_pool_after_the_writable_mount(
        &self,
        mount_writes: &[RetainedWrite],
        mount_segments: &[Vec<usize>],
        table_name: String,
    ) -> ConclusionsAfterTheWritableMount {
        let table = self.table_after_the_writable_mount(mount_writes, mount_segments, table_name);
        let judged = judge_the_given_states_after_the_mount(self, &table);
        let conclusion_of =
            |state: StateOfTheTableAfterTheWritableMount| judged.states[state.position()].clone();
        ConclusionsAfterTheWritableMount {
            on_the_pool_after_the_mount: conclusion_of(
                StateOfTheTableAfterTheWritableMount::PoolAfterTheMount,
            ),
            on_the_first_crash_image: conclusion_of(
                StateOfTheTableAfterTheWritableMount::FirstCrashImage,
            ),
            source: judged.source,
        }
    }

    /// 第二、三截那几张表的同一种建法：写表是 mkfs 之后的历史接挂载那一段、再接之后那一次发布，接缝 = 第一次崩溃之前的掉电接缝 +
    /// 历史与挂载之间那一道崩溃接缝；`states` 里每个状态给挂载那一段的持久集合（与整条挂载流逐条对应）与之后那一次发布落没落，
    /// 持久集合 = 历史那一截（这个第一次崩溃状态的）接上它们。
    ///
    /// # Panics
    /// 一个状态的挂载那一段持久集合与整条挂载流不一样长。
    fn table_of_the_states(
        &self,
        mount_writes: &[RetainedWrite],
        mount_segments: &[Vec<usize>],
        states: &[(Vec<bool>, PublishAfterTheMountInTheState)],
        table_name: String,
    ) -> GivenStatesTableAfterTheFirstCrash<'history> {
        let history = self.history_after_the_make_filesystem;
        let history_write_count = history.writes.len();
        let writes: Vec<RetainedWrite> = history
            .writes
            .iter()
            .chain(mount_writes)
            .chain(self.writes_of_the_publish_after_the_mount)
            .cloned()
            .collect();
        let shifted = |segments: &[Vec<usize>], by: usize| -> Vec<Vec<usize>> {
            segments
                .iter()
                .map(|segment| segment.iter().map(|write| write + by).collect())
                .collect()
        };
        let segments: Vec<Vec<usize>> = history
            .segments
            .iter()
            .cloned()
            .chain(shifted(mount_segments, history_write_count))
            .chain(shifted(
                self.segments_of_the_publish_after_the_mount,
                history_write_count + mount_writes.len(),
            ))
            .collect();
        let mut seams = self.power_loss_seams_of_the_table.clone();
        seams.push(RecordStreamSeam::AfterACrash {
            first_write_after_the_crash: history_write_count,
            version_landed_on_after_the_crash: self.version_landed_on_after_the_first_crash,
        });
        let persisted_sets: Vec<Vec<bool>> = states
            .iter()
            .map(|(persisted_in_the_mount, publish_after_the_mount)| {
                assert_eq!(
                    persisted_in_the_mount.len(),
                    mount_writes.len(),
                    "挂载那一段的持久集合与整条挂载流逐条对应"
                );
                let publish_after_the_mount_persisted = match publish_after_the_mount {
                    PublishAfterTheMountInTheState::Landed => true,
                    PublishAfterTheMountInTheState::NotWrittenYet => false,
                };
                self.persisted_history_of_the_table
                    .iter()
                    .chain(persisted_in_the_mount)
                    .copied()
                    .chain(std::iter::repeat_n(
                        publish_after_the_mount_persisted,
                        self.writes_of_the_publish_after_the_mount.len(),
                    ))
                    .collect()
            })
            .collect();
        let claimed_versions =
            vec![ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery; persisted_sets.len()];
        GivenStatesTableAfterTheFirstCrash {
            table_name,
            base: &history.base,
            writes,
            segments,
            seams,
            persisted_sets,
            claimed_versions,
        }
    }
}

/// 第二、三截的一张给定状态表（[`GivenStatesAfterTheFirstCrash`] 建）：基线是 mkfs 之后的池，写表是 mkfs 之后的历史接挂载那一段、
/// 再接之后那一次发布，接缝 = 第一次崩溃之前的掉电接缝 + 历史与挂载之间那一道崩溃接缝（用户 2026-10-10 定第三截照候选丁，
/// 与 [`HistoryThenWritableMountRecords`] 交 CPU 记录核对器的同一种断法；GPU 的发布表照它分），每个状态的持久集合与写表逐条对应，
/// 每个状态那一栏恢复自称的那一版（`claimed_versions`，与持久集合逐个对应）：第二截挂载之后的池那个状态写挂载与那次发布写出的最新那条根，
/// 别的状态不写（[`ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery`]）。
pub struct GivenStatesTableAfterTheFirstCrash<'history> {
    pub table_name: String,
    pub base: &'history MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub seams: Vec<RecordStreamSeam>,
    pub persisted_sets: Vec<Vec<bool>>,
    pub claimed_versions: Vec<ClaimedVersionOfAGivenState>,
}

impl GivenStatesTableAfterTheFirstCrash<'_> {
    /// 当成一张给定状态表交判器（借这一份的写表、切段与接缝；恢复自称的那一版那一栏另取 [`Self::claimed_versions_of_the_states`]）。
    #[must_use]
    pub fn given_states_table(&self) -> GivenStatesOfATable<'_> {
        GivenStatesOfATable {
            table_name: self.table_name.clone(),
            base: self.base,
            writes: &self.writes,
            segments: &self.segments,
            seams: &self.seams,
            persisted_sets: self.persisted_sets.clone(),
        }
    }

    /// 这张表每个状态那一栏恢复自称的那一版（与 [`Self::given_states_table`] 一起交判器：[`GivenStatesOfATableWithItsColumns`]）。
    #[must_use]
    pub fn claimed_versions_of_the_states(&self) -> ClaimedVersionsOfTheStates<'_> {
        ClaimedVersionsOfTheStates::OfEachState(&self.claimed_versions)
    }
}

/// 挂载与之后那一次发布写出的最新那条根（D13（验证路线） 已定项 7 第二截「恢复自称的那一版取挂载与那次发布写出的最新那条根，
/// 挂载一条根都没写出时退回第一次崩溃落到的那一版」）：`writes_after_the_crash`（第一次崩溃那道接缝之后的写表：挂载那一段接之后那次发布）里
/// 按写的次序最后一个根槽写写出的根；一个根槽写都没有时是 `version_landed_on_after_the_first_crash`（第一次崩溃之后看 journal 那一遍恢复落到的那一版，
/// 那一遍恢复失败是 None）。第二截给定状态表那一栏（[`GivenStatesAfterTheFirstCrash::table_after_the_writable_mount`]）与两份镜像那一路
/// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）都照它取。
///
/// # Panics
/// 根槽写不带字节（根槽 FUA 写是普通写，录制器照字节录）。
#[must_use]
pub fn newest_root_written_after_the_crash(
    writes_after_the_crash: &[RetainedWrite],
    version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
) -> Option<(InstanceGeneration, CheckpointTxg)> {
    writes_after_the_crash
        .iter()
        .rev()
        .find(|write| write.kind == StepKind::RootRecordFua)
        .map(|write| {
            root_identity_written_by(write.bytes().expect("根槽 FUA 写是普通写，带着字节"))
        })
        .or(version_landed_on_after_the_first_crash)
}

/// 第二截那张表里的两个状态（[`GivenStatesAfterTheFirstCrash::table_after_the_writable_mount`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateOfTheTableAfterTheWritableMount {
    /// 挂载与之后那一次发布之后的池：D13（验证路线） 已定项 7 的实现恢复后的镜像（可写挂载会改盘）。
    PoolAfterTheMount,
    /// 第一次崩溃那份镜像：D13（验证路线） 已定项 7 的崩溃态镜像。
    FirstCrashImage,
}

impl StateOfTheTableAfterTheWritableMount {
    /// 表里状态的次序。
    pub const ALL: [StateOfTheTableAfterTheWritableMount; 2] = [
        StateOfTheTableAfterTheWritableMount::PoolAfterTheMount,
        StateOfTheTableAfterTheWritableMount::FirstCrashImage,
    ];

    /// 这个状态在表里的下标（[`Self::ALL`] 里的位置）。
    #[must_use]
    pub fn position(self) -> usize {
        Self::ALL
            .iter()
            .position(|state| *state == self)
            .expect("ALL 里每个成员恰好出现一次")
    }
}

/// 第二截那张表的结论：挂载与之后那一次发布之后的池上那个状态的、第一次崩溃那份镜像上那个状态的，连同这张表的结论从哪来。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionsAfterTheWritableMount {
    /// 池级 checker 判的就是这一份（挂完的盘）。
    pub on_the_pool_after_the_mount: GivenStateConclusion,
    /// 只取它的记录核对器第一条判据（[`Self::record_check`]）；它的池级 checker 与两遍恢复第一截在同一份镜像上已判过。
    pub on_the_first_crash_image: GivenStateConclusion,
    pub source: GivenStatesConclusionsSource,
}

impl ConclusionsAfterTheWritableMount {
    /// 第二截记录核对器的两条判据（D13（验证路线） 已定项 7「崩溃后镜像是两份」）：第一条（根在而记录一条都不在）读崩溃态镜像，
    /// 取第一次崩溃那个状态的；第二条（恢复自称新态而单元缺席）读实现恢复后的镜像，取挂载之后的池那个状态的。两个状态写表与接缝相同、
    /// 历史那一截的持久集合相同，分出的发布也相同（`publishes_in` 在崩溃接缝之前只看那几次发布的根槽写落没落，接缝之后整截都分），
    /// 所以第一次崩溃那个状态上的第一条判据就是两份镜像那一判的第一条（`crate::crash::check_records_against` 的第一条判据只读崩溃态镜像）。
    #[must_use]
    pub fn record_check(&self) -> RecordCheck {
        RecordCheck {
            root_without_record: self
                .on_the_first_crash_image
                .record_check()
                .root_without_record,
            claimed_state_missing_unit: self
                .on_the_pool_after_the_mount
                .record_check()
                .claimed_state_missing_unit,
        }
    }

    /// 第二截挂载之后的池上判红的那几样（池级 checker 判的就是这一份）：池级 checker 违例逐条，再加一条
    /// [`RECOVERY_AFTER_THE_MOUNT_LANDED_ON_ANOTHER_VERSION_THAN_THE_NEWEST_ROOT_WRITTEN`]——这个状态那一栏写的是挂载与那次发布写出的
    /// 最新那条根（D13（验证路线） 已定项 7），看 journal 那一遍恢复在这一份上落到的根却不是它（恢复落旧了、或落到更新的一版）。
    #[must_use]
    pub fn violations_on_the_pool_after_the_mount(&self) -> Vec<(&'static str, String)> {
        let mut violations = self.on_the_pool_after_the_mount.pool_checker_violations();
        if self
            .on_the_pool_after_the_mount
            .claimed_version_is_not_landed_on()
        {
            let landed_on = self
                .on_the_pool_after_the_mount
                .words
                .consulted_pass_landed_root;
            let landed_on_text = landed_on.landed_root().map_or_else(
                || format!("没落到根（结论里落到的根那几个字是 {landed_on:?}）"),
                |(instance, checkpoint_txg)| {
                    format!("实例 {}、checkpoint_txg {}", instance.0, checkpoint_txg.0)
                },
            );
            violations.push((
                RECOVERY_AFTER_THE_MOUNT_LANDED_ON_ANOTHER_VERSION_THAN_THE_NEWEST_ROOT_WRITTEN,
                format!(
                    "挂载与那次发布之后的池上看 journal 那一遍恢复落到的根（{landed_on_text}）不是挂载与那次发布写出的最新那条根"
                ),
            ));
        }
        violations
    }
}

/// 第二截挂载之后的池上那一条判红的名字（[`ConclusionsAfterTheWritableMount::violations_on_the_pool_after_the_mount`]）。
pub const RECOVERY_AFTER_THE_MOUNT_LANDED_ON_ANOTHER_VERSION_THAN_THE_NEWEST_ROOT_WRITTEN: &str =
    "恢复落到的根不是挂载与那次发布写出的最新那条根";

/// 第三截：这个第一次崩溃状态上每个二次崩溃的池级 checker、两遍恢复、记录核对器交判器判（一张表，用户 2026-10-10 定第三截照候选丁；
/// 两份崩溃后镜像都是二次崩溃镜像，只读恢复不改盘）。`persisted_in_the_mount_of_each_second_crash` 里每个与整条挂载流逐条对应；
/// 之后那一次发布一个写都还没发。交回与它同序的结论。判不了就 panic（同 [`judge_the_first_crashes_on_given_states_tables`]）。
fn judge_the_second_crashes_on_a_given_states_table(
    given: &GivenStatesAfterTheFirstCrash<'_>,
    mount_writes: &[RetainedWrite],
    mount_segments: &[Vec<usize>],
    persisted_in_the_mount_of_each_second_crash: &[Vec<bool>],
    table_name: String,
) -> GivenStatesConclusionsOfATable {
    let states: Vec<(Vec<bool>, PublishAfterTheMountInTheState)> =
        persisted_in_the_mount_of_each_second_crash
            .iter()
            .map(|persisted_in_the_mount| {
                (
                    persisted_in_the_mount.clone(),
                    PublishAfterTheMountInTheState::NotWrittenYet,
                )
            })
            .collect();
    let table = given.table_of_the_states(mount_writes, mount_segments, &states, table_name);
    judge_the_given_states_after_the_mount(given, &table)
}

/// 第二、三截的一张表交判器判，交回与表里的状态同序的结论；判不了就 panic（不退回别处判）。
fn judge_the_given_states_after_the_mount(
    given: &GivenStatesAfterTheFirstCrash<'_>,
    table_after_the_first_crash: &GivenStatesTableAfterTheFirstCrash<'_>,
) -> GivenStatesConclusionsOfATable {
    let table = table_after_the_first_crash.given_states_table();
    let judged = given
        .judge
        .judge_the_given_states_with_its_columns(GivenStatesOfATableWithItsColumns {
            table: &table,
            container_capacity_knobs: ContainerCapacityKnobs::PRODUCT,
            claimed_versions: table_after_the_first_crash.claimed_versions_of_the_states(),
        })
        .unwrap_or_else(|reason| {
            panic!(
                "挂载之后的给定状态表 {} 判不了（不退回别处判）：{reason}",
                table.table_name
            )
        });
    assert_eq!(
        judged.states.len(),
        table.persisted_sets.len(),
        "判器逐状态交回结论，与表里的状态同序"
    );
    judged
}

/// RefFS 那一票（第一截）：照崩溃时刻交事件、判恢复读回的东西。比的是 RefFS 翻译的历史与实现读回的内容，不是一张镜像，上不了 GPU
/// （豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 `reference-filesystem` 那一行；用户 2026-10-10 点头）。
// gpu-exempt: reference-filesystem
fn reference_filesystem_judgement_of_a_crash_image(
    reference_filesystem_history: &TranslatedHistory,
    segment_the_crash_closes: usize,
    read_back: &ObservedReadBack,
) -> ReferenceFileSystemJudgementOfACrash {
    reference_filesystem_history.judge_a_crash(
        reference_filesystem_history.crash_moment_in_segment(segment_the_crash_closes),
        read_back,
    )
}

/// RefFS 那一票（二次崩溃那一截）：接着第一次崩溃判绿时重新起头的 RefFS 判（理由同 [`reference_filesystem_judgement_of_a_crash_image`]）。
// gpu-exempt: reference-filesystem
fn reference_filesystem_vote_on_a_second_crash(
    reference_filesystem_at_the_first_crash: &ReferenceFileSystemJudgementOfACrash,
    read_back_after_the_second_crash: &ObservedReadBack,
) -> ReferenceFileSystemVote {
    judge_a_second_crash_after_the_restart(
        reference_filesystem_at_the_first_crash,
        read_back_after_the_second_crash,
    )
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
/// 挂载末尾的实例表清扫取产品路径的配置（`InstanceTableSweepConfiguration::PRODUCT`）。
#[must_use]
pub fn writable_mount_alone_on(
    pool: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    space_admission: SpaceAdmission,
) -> WritableMountAlone {
    writable_mount_alone_with_the_instance_table_sweep_configuration_on(
        pool,
        parameters,
        space_admission,
        InstanceTableSweepConfiguration::PRODUCT,
    )
}

/// 同 [`writable_mount_alone_on`]，挂载末尾那一判实例表清扫的开关与阈值由调用方给（`mount::mount_writable_with_instance_table_sweep_configuration`；
/// 影子账开着，与产品路径同一臂）。
#[must_use]
pub fn writable_mount_alone_with_the_instance_table_sweep_configuration_on(
    pool: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    space_admission: SpaceAdmission,
    instance_table_sweep: InstanceTableSweepConfiguration,
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
    let mounted_instance = match mount_writable_with_instance_table_sweep_configuration(
        parameters,
        &mut devices,
        space_admission,
        ShadowLedger::On,
        instance_table_sweep,
    ) {
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
    /// 挂载末尾清扫写了东西（抹了头或发了清扫发布）时，清扫之前这次挂载发了几次（写行、暖机、推的抬 F、续做每一步与续做里推的抬 F）：
    /// 挂载那一段录制流里第几个根槽写之后起是清扫抹头；没清扫、或清扫一个字节都没写、或挂载被拒时 None。
    pub publishes_before_the_instance_table_sweep: Option<usize>,
    /// 挂载推的抬 F 从挂载那一段写表（`mount_operations` 按同一份几何切出来的写表）的哪个写起（二次崩溃上 F 判不判按它分；
    /// 挂载准入推的与续做里推的都认，界按它们在写表里的位置取，取不出的是 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]）；挂载被拒时 None。
    pub floor_raises_in_the_mount_writes: Option<FloorRaisesInTheMountWrites>,
    /// 挂载续做每一步那一版的两棵用户可见树的根指针（挂载透明那一样按它比，用户 2026-10-03 定「续做算挂载的一部分」）；挂载被拒时 None。
    pub user_visible_tree_root_pointers_of_the_continuation:
        Option<UserVisibleTreeRootPointersOfTheContinuation>,
    /// 挂载末尾的清扫发布删了行时：清扫发布那一版是哪条根、删了哪几行、抹了几个头（挂载透明的实例表行那一样按它放回被删的行再比，
    /// C704（挂载透明不认清扫发布））；没清扫、清扫没删行、或挂载被拒时 None。
    pub rows_deleted_by_the_instance_table_sweep: Option<RowsDeletedByTheInstanceTableSweepPublish>,
}

/// 崩溃后镜像上那次可写挂载末尾的清扫发布删了行（`InstanceTableSweepOutcome::RowsDeleted`）：清扫发布那一版的根身份、删掉的行、
/// 删行之前抹了几个头。清扫是挂载交回之前最后一件事（`mount::mount_writable` 文档「可写挂载末尾」），清扫发布那一版就是挂载交回的现行版本。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowsDeletedByTheInstanceTableSweepPublish {
    pub sweep_publish: (InstanceGeneration, CheckpointTxg),
    pub rows_deleted: Vec<InstanceRow>,
    pub heads_erased: usize,
}

impl RowsDeletedByTheInstanceTableSweepPublish {
    /// 挂载交回值里清扫删了行时交回删了什么；没删行（没扫、扫了没删、清扫发布没发成）时 None。
    #[must_use]
    pub fn of_the_mount(mounted: &Mounted) -> Option<Self> {
        match &mounted.output.instance_table_sweep {
            InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::RowsDeleted {
                    rows_deleted,
                    heads_erased,
                },
            ) => Some(Self {
                sweep_publish: (
                    mounted.current.root().instance,
                    mounted.current.root().checkpoint_txg,
                ),
                rows_deleted: rows_deleted.clone(),
                heads_erased: heads_erased.len(),
            }),
            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
            | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
            | InstanceTableSweepAtTheEndOfTheMount::Swept(
                InstanceTableSweepOutcome::NoRowDeleted(_)
                | InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. }
                | InstanceTableSweepOutcome::SweepPublishFailed { .. }
                | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing { .. },
            ) => None,
        }
    }
}

/// 崩溃注入里那次可写挂载（与每个二次崩溃镜像上的再挂）末尾的实例表清扫：有一行就扫（开关关着、阈值
/// [`InstanceTableRowCountThreshold::ANY_ROW`]、清扫发布照挂载的分配器取落点）。产品路径的阈值占位 369 行（C690（清扫触发阈值的默认值没定）），
/// 短历史到不了；要摆到清扫那两段（[`WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep`]、[`WritableMountPhase::InstanceTableSweepPublish`]）
/// 的用例用这一份（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支都能被测试强制进入）。
pub const INSTANCE_TABLE_SWEPT_ON_ANY_ROW: InstanceTableSweepConfiguration =
    InstanceTableSweepConfiguration {
        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
        row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    };

/// 挂载之后那一次发布写的内容：随种子与崩溃状态变，装得进一个数据单元（模型只罩一个数据单元的文件）。
fn content_of_the_publish_after_the_crash(seed: HistorySeed, crash_point: &CrashPoint) -> Vec<u8> {
    let length = 97 + crash_point.segment_index % 64;
    let fill = seed.0.to_le_bytes()[0];
    (0..length)
        .map(|index| fill ^ u8::try_from(index % 251).expect("小于 251"))
        .collect()
}

/// 在 `crash_image_materialized` 的副本上起可写挂载（建池那一份参数、那一段历史的空间准入开关），做成了就接着发一次布：
/// 挂载交回的现行版本带文件、第一个文件那条记录还活着时覆盖写，树表 0 条时发第一个文件（与随机历史里这两个入口同一种调法），
/// 有树而文件集合为空（删完那一版）时、或带文件而第一个文件已删完或正在删（unlink 之后再建过 inode）时建一个 size 0 的 inode
/// （D8（核心索引结构） 已定项 16 第 4、5 条：写入口拒不在树里与正在删的 inode）。挂载末尾的实例表清扫取产品路径的配置
/// （`InstanceTableSweepConfiguration::PRODUCT`）。
#[must_use]
pub fn mount_and_publish_once_on_the_crash_image(
    crash_image_materialized: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    execution: HistoryExecution,
    seed: HistorySeed,
    crash_point: &CrashPoint,
) -> WritableMountOnTheCrashImage {
    mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the_crash_image(
        crash_image_materialized,
        parameters,
        execution,
        seed,
        crash_point,
        InstanceTableSweepConfiguration::PRODUCT,
    )
}

/// 同 [`mount_and_publish_once_on_the_crash_image`]，挂载末尾那一判实例表清扫的开关与阈值由调用方给
/// （`mount::mount_writable_with_instance_table_sweep_configuration`，影子账开着、与产品路径同一臂）。
/// 可写挂载与之后那一次发布真跑实现，只能在 CPU 上（豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount 那一行）；
/// 挂完的盘上的池级 checker 与记录核对器不在这里，经给定状态表交判器判（第二截，[`GivenStatesAfterTheFirstCrash::judge_the_pool_after_the_writable_mount`]）。
// gpu-exempt: writable-mount
#[must_use]
pub fn mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the_crash_image(
    crash_image_materialized: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    execution: HistoryExecution,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    instance_table_sweep: InstanceTableSweepConfiguration,
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
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        parameters,
        &mut devices,
        execution.space_admission,
        ShadowLedger::On,
        instance_table_sweep,
    );
    let mount_operation_count = stream.operation_count();
    let rows_deleted_by_the_instance_table_sweep = mounted
        .as_ref()
        .ok()
        .and_then(RowsDeletedByTheInstanceTableSweepPublish::of_the_mount);
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
    let publishes_before_the_instance_table_sweep = mounted.as_ref().ok().and_then(|mounted| {
        publishes_before_the_instance_table_sweep_of_the_mount_output(&mounted.output)
    });
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
                PoolVersion::WithFile(current) if first_file_is_a_live_record(current) => {
                    publish_overwrite(
                        &mut writer,
                        &mut allocator,
                        current,
                        file,
                        mounted.output.instance,
                    )
                    .map(|_| ())
                }
                // 带文件的一版而第一个文件已删完（unlink 之后再建过 inode：树里只剩别的 inode）或正在删（nlink = 0）：覆盖写只写第一个文件，
                // 在这一版上被拒成写一个不在树里、或正在删的 inode（D8（核心索引结构） 已定项 16 第 4 条）；与删完那一版一样建一个 size 0 的 inode。
                PoolVersion::WithFile(current) => publish_new_inodes(
                    &mut writer,
                    &mut allocator,
                    current,
                    1,
                    FIXED_WRITE_TIME_SECONDS,
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
        publishes_before_the_instance_table_sweep,
        floor_raises_in_the_mount_writes,
        user_visible_tree_root_pointers_of_the_continuation,
        rows_deleted_by_the_instance_table_sweep,
    }
}

/// 带文件的一版里第一个文件那条记录还活着（在树里、nlink 不是 0）：覆盖写只写第一个文件，写入口拒删完与正在删的
/// （`transaction::live_record_of_the_first_file`，D8（核心索引结构） 已定项 16 第 4 条）。unlink 之后再建过 inode 的一版仍归带文件的一版
/// （`TransactionOutput::is_a_version_with_trees_and_no_file` 为假），它的 `inode_record` 是 None。
fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {
    version
        .inode_record
        .as_ref()
        .is_some_and(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
}

/// 一次发布写没写只有续做的删除步才写的那几种结构：文件的 extent 树、inode 树与 `deleted_inodes` 树（数据单元也算，挂载里没有别的发布写它）。
/// 写行、暖机与抬 F 的空发布只重写分配记录树、记账树、中央映射树、树表、实例表与固定结构（2026-10-03 在副本上量的：崩在意图之后重挂，
/// 写行与两次暖机写的是这几种，续做那一步写了 extent 树与 inode 树根）。
/// 退役树（第 50 批）只在身份消失的那次最后一批写，挂载里只可能是续做那一步写的，同样认。
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
            | WrittenStructureKind::DeletedInodesTreeNode
            // logged_ops 根只在整理签发、删意图的发布里写（判决 m4-gc-r1 第三节 G1）：挂载里只可能是续做整理那一步写的。
            | WrittenStructureKind::LoggedOperationsTreeNode
            | WrittenStructureKind::RetirementTreeNode
            // deadlist 树只在杀了被活快照引用的块的发布与删快照的批里写（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
            // 第三节 S3、S4）：挂载里只可能是续做删快照那一步写的。
            | WrittenStructureKind::DeadlistTreeNode => true,
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
        | MountError::InstanceTableUnreadable
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
        | MountError::NewerStateStillUnreadableAfterOneReread(_)
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,
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
/// 在历史与挂载的接缝处断开（[`RecordStreamSeam::AfterACrash`]：历史那一段只留第一次崩溃之后那条时间线上的发布）；
/// 历史在第一次崩溃之前掉过电的，也在每道掉电接缝处断开（[`RecordStreamSeam::AfterAPowerLoss`]）。
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
    /// 交给记录核对器的接缝：第一次崩溃之前的几道掉电接缝，再加历史与挂载之间那一道崩溃接缝。
    seams: Vec<RecordStreamSeam>,
}

impl HistoryThenWritableMountRecords {
    /// `first_write_after_each_power_loss_before_the_crash` 是第一次崩溃之前每次掉电之后第一个写在历史写表里的下标，从小到大
    /// （历史在那次崩溃之前没掉过电的给空表）。
    ///
    /// # Panics
    /// `persisted_in_the_history` 与 `history_writes` 不一样长；有一道掉电接缝越过历史写表。
    #[must_use]
    pub fn new(
        history_writes: &[RetainedWrite],
        persisted_in_the_history: &[bool],
        first_write_after_each_power_loss_before_the_crash: &[usize],
        version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
        mount_writes: &[RetainedWrite],
        writes_of_the_publish_after_the_mount: &[RetainedWrite],
    ) -> Self {
        assert_eq!(
            persisted_in_the_history.len(),
            history_writes.len(),
            "历史那一段的持久集合与历史录制流逐条对应"
        );
        assert!(
            first_write_after_each_power_loss_before_the_crash
                .iter()
                .all(|first_write_after_the_power_loss| {
                    *first_write_after_the_power_loss <= history_writes.len()
                }),
            "掉电接缝落在历史写表里：{first_write_after_each_power_loss_before_the_crash:?}，历史写表 {} 项",
            history_writes.len()
        );
        let mut seams = power_loss_seams(first_write_after_each_power_loss_before_the_crash);
        seams.push(RecordStreamSeam::AfterACrash {
            first_write_after_the_crash: history_writes.len(),
            version_landed_on_after_the_crash: version_landed_on_after_the_first_crash,
        });
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
            seams,
        }
    }

    /// 挂载那一段的写（二次崩溃按它切段、摆状态）。
    #[must_use]
    pub fn mount_writes(&self) -> &[RetainedWrite] {
        let first_write_of_the_mount = self.persisted_in_the_history.len();
        &self.writes[first_write_of_the_mount..first_write_of_the_mount + self.writes_of_the_mount]
    }

    fn continuity(&self) -> RecordStreamContinuity<'_> {
        RecordStreamContinuity::BrokenAtSeams(&self.seams)
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
        check_records_against(
            crash_image,
            pool_after_the_mount,
            &self.writes,
            &self.persisted_with_the_mount(
                &vec![true; self.writes_of_the_mount],
                PublishAfterTheMountInTheState::Landed,
            ),
            self.continuity(),
            newest_root_written_after_the_crash(
                &self.writes[self.persisted_in_the_history.len()..],
                self.version_landed_on_after_the_first_crash,
            ),
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

/// 挂载之后那一次发布的写在一个状态里落没落：第二截的池上整次落了；第二截那张表里第一次崩溃那份镜像上、第三截崩在挂载途中时，它一个写都还没发。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishAfterTheMountInTheState {
    Landed,
    NotWrittenYet,
}

/// 第二截、第三截被拒的那次挂载都起在崩溃镜像（第一次崩溃的、二次崩溃的）上，盘是不带故障层的内存盘：读到的就是镜像上的字节，
/// 瞬时读错不会发生——崩溃态注入过读故障也一样（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
/// C3 / J2-a 那一行，改法 G3：判官恒交「否」）。
const READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE: ReadFaultsUnderTheRefusedMount =
    ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults;

/// 判可写挂载被拒合不合法要的：崩溃后镜像（拒的判据在它上面复算，拒之后的池要与它逐字节相同）、它上面只读恢复落到的那一版、
/// 那一次只读读出在不在模型的允许集合里（问模型的结论）。第二截交第一次崩溃的镜像与第一截的结论；第三截交二次崩溃镜像与那一个二次崩溃上的结论，
/// 读故障的认定沿用第一次崩溃那个态的（第二截那次挂载起在第一次崩溃的镜像上，不注入读故障）。
struct RefusalOnTheCrashImage<'crash> {
    crash_image_materialized: &'crash MemoryPool,
    version_landed_on_after_the_crash: Option<(InstanceGeneration, CheckpointTxg)>,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
    /// 这个崩溃态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：没注入的，判据 N-配置 复算为真即拒得不合法（C670），
    /// 「读阶段判完之后系统配置又读不出」那一拒的判据复算为真也拒得不合法（改法 G4）。
    read_faults: ReadFaultsInTheCrashState,
}

/// 第二截：挂载或那次发布被拒（单元区墙只计数，模型在单元区墙上只答允许拒绝的区间、而这里没有崩溃点上的模型状态可判区间），
/// 做没做成都在之后的池上跑池级 checker；`record_check` 是第二截记录核对器的结论（第一条判据读第一次崩溃那份镜像、第二条读之后的池，
/// [`ConclusionsAfterTheWritableMount::record_check`]）。挂载被拒而理由在 D23（journal 的角色与格式） 已定项 14
/// 那张单子上的，判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount`，被拒的那次挂载恒交不带故障层：判据在崩溃镜像上复算为真、
/// 盘上逐字节不变、只读读出在允许集合里；崩溃态没注入读故障而判据 N-配置 复算为真的另判不合法，C670，「读阶段判完之后系统配置又读不出」
/// 那一拒的判据复算为真的同样另判不合法，改法 G4），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
/// 模型对可写挂载与之后那一次发布（覆盖写活着的第一个文件、第一个文件、建 inode）从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数与发现、挂载那一趟与它之后的记录核对、判拒绝要的崩溃镜像那一份、这一个崩溃状态落在哪"
)]
// gpu-exempt: writable-mount
fn judge_the_writable_mount_on_the_crash_image(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    mount_run: &WritableMountOnTheCrashImage,
    record_check: RecordCheck,
    pool_checker_violations: Vec<(&'static str, String)>,
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
            match judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
                refusal_on_the_crash_image.crash_image_materialized,
                refusal_on_the_crash_image.version_landed_on_after_the_crash,
                refusal_error,
                disk,
                refusal_on_the_crash_image.read_only_read_back,
                refusal_on_the_crash_image.read_faults,
                READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,
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
    let violations = pool_checker_violations;
    if violations.is_empty() && refusal.is_none() && record_check == RecordCheck::default() {
        return;
    }
    let model_disagreement = refusal.map(|implementation_answer| {
        ModelDisagreement::new(
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
            "崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）与之后那一次发布（第一个文件还活着时覆盖写、树表 0 条时第一个文件、第一个文件已删完或正在删时建一个 inode）都该成（单元区墙之外模型不许拒）"
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
/// 第一次崩溃那个态注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：挂载透明的射程只管不带故障的状态
/// （D13（验证路线） 已定项 1），崩溃镜像上那一次挂载整个不判，它途中的每个二次崩溃也不判（各记一次）。
const MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE: &str =
    "崩溃态注入过读故障（挂载透明只管不带故障的状态）";

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

/// 挂载透明判红时记进违例的名字：R2 那一版是清扫发布那一版，它的实例表里却仍有清扫说删掉了的行（C704（挂载透明不认清扫发布））。
pub const MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH: &str =
    "挂载透明·实例表行（清扫发布那一版里仍有它删掉的行）";

/// R2 那一版的实例表行对清扫发布删的行是什么样（C704（挂载透明不认清扫发布）的处置：判法先跟上，再让战役带清扫配置）。
#[derive(Debug)]
enum InstanceTableRowsAgainstTheSweepPublish {
    /// 挂载没删行、R2 那一版不是清扫发布那一版，或它的实例表读不出（读不出照原样交给逐字段判，实例表行那一样照判红）：照原样比。
    ComparedAsRead,
    /// R2 那一版是清扫发布那一版、被删的行一条都不在：把它们放回去之后的 R2（其余的行照 D18 已定项 11「恢复写的行」判）。
    DeletedRowsPutBack(Box<RecoveryFieldsRead>),
    /// R2 那一版是清扫发布那一版，被删的行还在：判红。
    DeletedRowsStillPresent(Vec<InstanceRow>),
}

/// 按清扫发布那一版的根身份认 R2（只挂载与二次崩溃比恢复落到的那一版、挂载加一次发布比发布之前那一版——清扫是挂载交回之前最后一件事，
/// 两种比法拿到的都可能是它），判被删的行在不在。
fn instance_table_rows_against_the_sweep_publish(
    after_the_action: &RecoveryFieldsRead,
    rows_deleted_by_the_sweep: Option<&RowsDeletedByTheInstanceTableSweepPublish>,
) -> InstanceTableRowsAgainstTheSweepPublish {
    let Some(rows_deleted_by_the_sweep) = rows_deleted_by_the_sweep else {
        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
    };
    let Some(compared) = &after_the_action.compared_version else {
        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
    };
    if (compared.root.instance, compared.root.checkpoint_txg)
        != rows_deleted_by_the_sweep.sweep_publish
    {
        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
    }
    let Some(rows_read) = &compared.instance_table_rows else {
        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
    };
    let deleted_instances: BTreeSet<InstanceGeneration> = rows_deleted_by_the_sweep
        .rows_deleted
        .iter()
        .map(|row| row.instance)
        .collect();
    let still_present: Vec<InstanceRow> = rows_read
        .iter()
        .filter(|row| deleted_instances.contains(&row.instance))
        .copied()
        .collect();
    if !still_present.is_empty() {
        return InstanceTableRowsAgainstTheSweepPublish::DeletedRowsStillPresent(still_present);
    }
    let rows_with_the_deleted_put_back: Vec<InstanceRow> = rows_read
        .iter()
        .chain(&rows_deleted_by_the_sweep.rows_deleted)
        .copied()
        .collect();
    let mut put_back = after_the_action.clone();
    put_back
        .compared_version
        .as_mut()
        .expect("上面判过 R2 那一版在")
        .instance_table_rows = Some(rows_with_the_deleted_put_back);
    InstanceTableRowsAgainstTheSweepPublish::DeletedRowsPutBack(Box::new(put_back))
}

/// 挂载透明逐字段判（`mount_transparency::judge_mount_transparency`）之前，挂载末尾的清扫发布删了行时按 C704（挂载透明不认清扫发布）先处置 R2：
/// 那里的逐字段判法按「恢复写的行」比实例表行，不认清扫发布删行。R2 那一版是清扫发布那一版而被删的行一条都不在 ⇒ 交回放回之后的 R2；
/// 被删的行还在 ⇒ R2 照原样交回、另交一条违例（[`MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH`]，
/// 计进按字段分的判红计数）；别的照原样交回。被删的是哪几行取挂载交回值（与 F 取挂载交回的抬 F、续做取挂载交回的根指针同一个做法），
/// 判法不另算哪几行该删。
fn after_the_action_with_the_rows_deleted_by_the_sweep_put_back<'read>(
    tally: &mut CrashInjectionTally,
    after_the_action: &'read RecoveryFieldsRead,
    rows_deleted_by_the_sweep: Option<&RowsDeletedByTheInstanceTableSweepPublish>,
) -> (Cow<'read, RecoveryFieldsRead>, Vec<(&'static str, String)>) {
    match instance_table_rows_against_the_sweep_publish(after_the_action, rows_deleted_by_the_sweep)
    {
        InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead => {
            (Cow::Borrowed(after_the_action), Vec::new())
        }
        InstanceTableRowsAgainstTheSweepPublish::DeletedRowsPutBack(put_back) => {
            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back += 1;
            (Cow::Owned(*put_back), Vec::new())
        }
        InstanceTableRowsAgainstTheSweepPublish::DeletedRowsStillPresent(still_present) => {
            *tally
                .mount_transparency_violations_by_field
                .entry(MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH)
                .or_insert(0) += 1;
            (
                Cow::Borrowed(after_the_action),
                vec![(
                    MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH,
                    format!(
                        "清扫发布 {:?} 删了 {:?}；那一版的实例表里仍有 {still_present:?}",
                        rows_deleted_by_the_sweep.map(|deleted| deleted.sweep_publish),
                        rows_deleted_by_the_sweep.map(|deleted| &deleted.rows_deleted)
                    ),
                )],
            )
        }
    }
}

/// 第一次崩溃镜像上一个动作的挂载透明：R1（`before_the_mount`）与动作之后的池（`pool_after_the_action`）上的只读恢复（`after_the_action_as_read`；挂载末尾的清扫发布删了行时先按 C704 处置）逐字段比；
/// 判红的落成这一截（[`CrashStateStage::MountTransparencyAfterTheCrash`]）的新发现，镜像留的是动作之后的池。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数与发现、哪个动作、两次恢复与动作之后的池、动作与挂载续做那几步的交回值、清扫发布删的行、这一个崩溃状态落在哪"
)]
fn judge_mount_transparency_of_one_comparison(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    comparison: MountTransparencyComparison,
    before_the_mount: &RecoveryFieldsRead,
    pool_after_the_action: &MemoryPool,
    after_the_action_as_read: &RecoveryFieldsRead,
    action: MountTransparencyAction,
    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
    rows_deleted_by_the_sweep: Option<&RowsDeletedByTheInstanceTableSweepPublish>,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    image_directory: Option<&Path>,
) {
    let (compared_after_the_action, rows_deleted_by_the_sweep_still_present) =
        after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
            tally,
            after_the_action_as_read,
            rows_deleted_by_the_sweep,
        );
    let after_the_action: &RecoveryFieldsRead = &compared_after_the_action;
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
    let mut violations = violations_of_the_mount_transparency(tally, &transparency);
    violations.extend(rows_deleted_by_the_sweep_still_present);
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
            mount_run.rows_deleted_by_the_instance_table_sweep.as_ref(),
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
                mount_run.rows_deleted_by_the_instance_table_sweep.as_ref(),
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
/// 没崩过的那一次挂载末尾的清扫发布删了行时，实例表行那一样照 C704（挂载透明不认清扫发布）的处置判
/// （[`after_the_action_with_the_rows_deleted_by_the_sweep_put_back`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：计数、R1 与二次崩溃镜像上的恢复、没崩过的那一次挂载的抬 F 界与续做根指针与清扫删的行、二次崩溃的持久集合与段"
)]
fn mount_transparency_violations_of_a_second_crash(
    tally: &mut CrashInjectionTally,
    fields_read_after_the_first_crash: &RecoveryFieldsRead,
    fields_read_after_the_second_crash: &RecoveryFieldsRead,
    floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
    continuation_of_the_uninterrupted_mount: Option<&UserVisibleTreeRootPointersOfTheContinuation>,
    rows_deleted_by_the_sweep_of_the_uninterrupted_mount: Option<
        &RowsDeletedByTheInstanceTableSweepPublish,
    >,
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
    let (fields_read_after_the_second_crash, rows_deleted_by_the_sweep_still_present) =
        after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
            tally,
            fields_read_after_the_second_crash,
            rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
        );
    let transparency = judge_mount_transparency(
        fields_read_after_the_first_crash,
        &fields_read_after_the_second_crash,
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
    let mut violations = violations_of_the_mount_transparency(tally, &transparency);
    violations.extend(rows_deleted_by_the_sweep_still_present);
    violations
}

/// 挂载途中的一次二次崩溃：挂载那一段的第几段、段内哪几个写持久了（靶向用例摆的那一个从这一段的第一个写起数，可以越过段尾）、崩在挂载的哪一段。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SecondCrashPoint {
    segment_index: usize,
    persisted_within_the_segment: Vec<bool>,
    phase: WritableMountPhase,
}

/// 挂载途中的二次崩溃怎么摆（[`judge_second_crashes_inside_the_writable_mount`]）。
#[derive(Clone, Copy)]
pub enum SecondCrashPointDraw {
    /// 挂载的每一段各按种子摆一个（[`draw_second_crash_points`]）：[`inject_crashes_into_history`] 与战役都走这一支。
    OneInEachMountPhaseBySeed,
    /// 每个第一次崩溃态上都只摆这一个二次崩溃（靶向用例用，攻方改法 F2，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：
    /// 从挂载那一段写表切出的第 `segment_index` 段的第一个写起数 `writes_from_the_segment_on` 个写，其中下标（从这一段的第一个写数起）
    /// 在 `withheld_among_them` 里的扣下、别的持久；更早的段整段持久，这几个写之后的一个都没持久。这几个写可以越过这一段的末尾，
    /// 越过的那几道屏障当设备没守（同 [`CrashPointDraw::ExactlyOneCrashState`]：今天的持久顺序下根槽 FUA 先于轮换、各成一段，
    /// 「轮换落了、根没落」在段内真子集里摆不出来）。崩在挂载的哪一段按第一个被扣下的写取。
    /// 段号不在挂载写出的段里、这几个写越过挂载写表末尾、`writes_from_the_segment_on` 是 0、扣下的下标越界或一个都没有时 panic——
    /// 写死的输入写错了，不悄悄摆别的状态（[`exactly_one_second_crash_point`]）。
    ExactlyOneSecondCrashState {
        segment_index: usize,
        writes_from_the_segment_on: usize,
        withheld_among_them: &'static [usize],
    },
}

/// 靶向用例那一个二次崩溃（[`SecondCrashPointDraw::ExactlyOneSecondCrashState`]）：持久集合从挂载第 `segment_index` 段的第一个写起数，
/// 可以越过段尾；崩在挂载的哪一段取第一个被扣下的写的（`phase_of_each_write` 与挂载写表逐条对应）。
///
/// # Panics
/// 段号不在 `mount_segments` 里、`writes_from_the_segment_on` 是 0 或越过挂载写表末尾、扣下的下标一个都没有或越界。
fn exactly_one_second_crash_point(
    mount_segments: &[Vec<usize>],
    phase_of_each_write: &[WritableMountPhase],
    segment_index: usize,
    writes_from_the_segment_on: usize,
    withheld_among_them: &[usize],
) -> SecondCrashPoint {
    let first_write_of_the_segment = mount_segments
        .get(segment_index)
        .and_then(|segment| segment.first().copied())
        .unwrap_or_else(|| {
            panic!(
                "靶向用例写死的挂载第 {segment_index} 段不在挂载写出的 {} 段里",
                mount_segments.len()
            )
        });
    assert!(
        writes_from_the_segment_on > 0
            && first_write_of_the_segment + writes_from_the_segment_on
                <= phase_of_each_write.len(),
        "靶向用例从挂载第 {segment_index} 段起数 {writes_from_the_segment_on} 个写：要至少一个、不越过挂载写表末尾（{} 个写）",
        phase_of_each_write.len()
    );
    let first_withheld = withheld_among_them
        .iter()
        .min()
        .copied()
        .filter(|_| {
            withheld_among_them
                .iter()
                .all(|withheld| *withheld < writes_from_the_segment_on)
        })
        .unwrap_or_else(|| {
            panic!(
                "靶向用例扣下的写 {withheld_among_them:?} 要至少一个、都落在从挂载第 {segment_index} 段起数的 {writes_from_the_segment_on} 个写里"
            )
        });
    SecondCrashPoint {
        segment_index,
        persisted_within_the_segment: (0..writes_from_the_segment_on)
            .map(|within_the_state| !withheld_among_them.contains(&within_the_state))
            .collect(),
        phase: phase_of_each_write[first_write_of_the_segment + first_withheld],
    }
}

/// 第一次崩溃那个段内子集的指纹：段内第 k 个写持久了折 1、没持久折 0，逐个按 64 位 FNV-1a 折进去。只用来岔开二次崩溃的随机源，
/// 不进任何判定。
fn fingerprint_of_the_persisted_subset(persisted_within_the_segment: &[bool]) -> u64 {
    persisted_within_the_segment.iter().fold(
        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,
        |fingerprint, is_persisted| {
            (fingerprint ^ u64::from(*is_persisted))
                .wrapping_mul(PERSISTED_SUBSET_FINGERPRINT_PRIME)
        },
    )
}

/// 一个第一次崩溃态上按种子摆二次崩溃（[`draw_second_crash_points`]）用的随机源种子。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SecondCrashPointSeed(u64);

/// 历史种子、盐、第一次崩溃的段号与它段内子集的指纹（[`fingerprint_of_the_persisted_subset`]）四样异或。
fn seed_of_the_second_crash_points(
    seed: HistorySeed,
    crash_point: &CrashPoint,
) -> SecondCrashPointSeed {
    SecondCrashPointSeed(
        seed.0
            ^ SECOND_CRASH_POINT_SEED_SALT
            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")
            ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),
    )
}

/// 挂载的每一段各摆一个二次崩溃：在那一段的写里按种子挑一个 w，崩在 w 上——w 所在那一段里 w 之前的写都落了、w 没落、
/// w 之后的写各按种子抽；更早的段整段持久，更晚的段一个都没持久（与第一次崩溃同一个枚举域）。挂载没写出某一段的写就不摆那一段。
/// 种子是历史种子、盐、第一次崩溃的段号与它段内子集的指纹四样异或（[`seed_of_the_second_crash_points`]）：同一段里不同子集的
/// 第一次崩溃态各抽各的（攻方改法 F3，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第五节第 1 行；
/// 只混段号时同一段里的各个第一次崩溃态用同一串随机数，挂载写表相同就摆到同一组二次崩溃）。
fn draw_second_crash_points(
    seed: HistorySeed,
    crash_point: &CrashPoint,
    segments: &[Vec<usize>],
    phase_of_each_write: &[WritableMountPhase],
) -> Vec<SecondCrashPoint> {
    let mut source =
        SeededRandomSource::from_seed(seed_of_the_second_crash_points(seed, crash_point).0);
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
    /// 没崩过的那一次挂载在清扫之前发了几次（[`WritableMountOnTheCrashImage::publishes_before_the_instance_table_sweep`]）。
    publishes_before_the_instance_table_sweep: Option<usize>,
    /// 第一次崩溃镜像上的只读恢复与挂载透明要比的字段（R1）。
    fields_read_after_the_first_crash: &'history RecoveryFieldsRead,
    /// 没崩过的那一次挂载推的抬 F 从挂载那一段写表的哪个写起（没推是 [`FloorRaisesInTheMountWrites::NotPushed`]）；挂载被拒时 None。
    floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
    /// 没崩过的那一次挂载续做每一步那一版的两根指针（二次崩溃落到挂载写的那一版时挂载透明按它比）；挂载被拒时 None。
    continuation_of_the_uninterrupted_mount:
        Option<&'history UserVisibleTreeRootPointersOfTheContinuation>,
    /// 二次崩溃之后再挂用的建池参数、空间准入开关与实例表清扫的配置（与没崩过的那一次挂载同一份）。
    parameters: &'history MakeFilesystemParameters,
    space_admission: SpaceAdmission,
    instance_table_sweep: InstanceTableSweepConfiguration,
    /// 没崩过的那一次挂载末尾的清扫发布删了哪几行（二次崩溃落到清扫发布那一版时挂载透明按它放回再比，C704）；没删行时 None。
    rows_deleted_by_the_sweep_of_the_uninterrupted_mount:
        Option<&'history RowsDeletedByTheInstanceTableSweepPublish>,
    /// RefFS 在第一次崩溃上的判定：判绿时带着以读出的内容重新起头的 RefFS，二次崩溃接着它判。
    reference_filesystem_at_the_first_crash: &'history ReferenceFileSystemJudgementOfACrash,
    /// 第一次崩溃那个态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：二次崩溃之后再挂被拒时判 C670 那一条沿用它；
    /// 注入过的，二次崩溃镜像上的挂载透明照第二截跳过（改法 G5）。
    read_faults_in_the_first_crash_state: ReadFaultsInTheCrashState,
    /// 二次崩溃怎么摆：挂载每一段按种子各摆一个，或靶向用例写死的那一个。
    second_crash_draw: SecondCrashPointDraw,
    /// 二次崩溃镜像上只读一张镜像的那几样交给定状态表判（[`judge_the_second_crashes_on_a_given_states_table`]）。
    given_states: GivenStatesAfterTheFirstCrash<'history>,
}

/// 第三截：挂载途中的二次崩溃。每个二次崩溃状态上：只读恢复（看 journal）、问模型——允许的版本是模型提交过的每一版，加上这次挂载写出的根
/// （写行那次与暖机照抄第一次崩溃之后恢复到的那一版的文件，所以内容取那一次读回的；那一次恢复失败时一条都不加）；
/// 盘上已持久的最新根槽取第一次崩溃与挂载写表两边里更新的那条——、池级 checker、记录核对器。
/// 池级 checker、两遍恢复与记录核对器在一张给定状态表上判（[`judge_the_second_crashes_on_a_given_states_table`]，配了 GPU 走 GPU）：
/// 记录核对器核的是 mkfs 之后的整条历史录制流接上整条挂载流，历史那几次发布在二次崩溃镜像上还在不在也核；恢复落到由记录重建的
/// 写行那一版时，它的根槽写在更晚的段里，也在表里。要录下可写挂载那一趟 Rust 实现发出的写、二次崩溃之后再挂，上不了 GPU。
// gpu-exempt: second-crash-during-mount
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
        publishes_before_the_instance_table_sweep,
        fields_read_after_the_first_crash,
        floor_raises_in_the_uninterrupted_mount_writes,
        continuation_of_the_uninterrupted_mount,
        parameters,
        space_admission,
        instance_table_sweep,
        rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
        reference_filesystem_at_the_first_crash,
        read_faults_in_the_first_crash_state,
        second_crash_draw,
        given_states,
    } = inputs;
    let mount_writes = records_across_the_crash.mount_writes();
    let phase_of_each_write = writable_mount_phase_of_each_write(
        mount_writes,
        publishes_before_the_deletion_queue_continuation,
        publishes_before_the_instance_table_sweep,
    );
    let second_crash_points = match second_crash_draw {
        SecondCrashPointDraw::OneInEachMountPhaseBySeed => {
            draw_second_crash_points(seed, crash_point, mount_segments, &phase_of_each_write)
        }
        SecondCrashPointDraw::ExactlyOneSecondCrashState {
            segment_index,
            writes_from_the_segment_on,
            withheld_among_them,
        } => vec![exactly_one_second_crash_point(
            mount_segments,
            &phase_of_each_write,
            segment_index,
            writes_from_the_segment_on,
            withheld_among_them,
        )],
    };
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
    // 与整条挂载流逐条对应：更早的段整段持久、当前段按二次崩溃的子集、更晚的段一个都没持久。
    let persisted_in_the_mount_of_each_second_crash: Vec<Vec<bool>> = ordered
        .iter()
        .map(|second| {
            persisted_over_the_whole_write_table(
                mount_writes.len(),
                mount_segments[second.segment_index][0],
                &second.persisted_within_the_segment,
            )
        })
        .collect();
    let second_crash_conclusions = judge_the_second_crashes_on_a_given_states_table(
        &given_states,
        mount_writes,
        mount_segments,
        &persisted_in_the_mount_of_each_second_crash,
        format!(
            "seed={}/second-crash/segment={}/first-crash-fingerprint={:016x}",
            seed.0,
            crash_point.segment_index,
            fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment)
        ),
    );
    count_a_table_left_to_another_host(tally, second_crash_conclusions.source);
    for ((second, persisted_in_the_mount), second_crash_conclusion) in ordered
        .iter()
        .zip(&persisted_in_the_mount_of_each_second_crash)
        .zip(&second_crash_conclusions.states)
    {
        let segment = &mount_segments[second.segment_index];
        let first_write_of_the_segment = segment[0];
        let writes_in_the_segment = second.persisted_within_the_segment.len();
        second_base.apply_writes(
            &mount_writes[writes_applied_to_the_second_base..first_write_of_the_segment],
        );
        writes_applied_to_the_second_base = first_write_of_the_segment;
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
            newest_persisted_root(mount_writes, persisted_in_the_mount)
                .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
        let newest_persisted = newest_persisted_before_the_mount.max(newest_persisted_by_the_mount);
        tally.second_crash_model_judgements += 1;
        let disagreement = disagreement_with_the_ideal_model(
            &versions_allowed,
            newest_persisted,
            &read_back_after_the_second_crash,
        );
        let reference_filesystem_vote = reference_filesystem_vote_on_a_second_crash(
            reference_filesystem_at_the_first_crash,
            &read_back_after_the_second_crash,
        );
        tally.note_the_reference_filesystem_vote(
            StageTheReferenceFileSystemJudges::SecondCrashInsideTheWritableMount,
            &reference_filesystem_vote,
            disagreement.as_ref(),
        );
        tally.second_crash_record_checks += 1;
        let record_check = second_crash_conclusion.record_check();
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
            instance_table_sweep,
            second.phase,
            seed,
            crash_point,
            image_directory,
        );
        tally.second_crash_checker_runs += 1;
        let mut violations = second_crash_conclusion.pool_checker_violations();
        violations.extend(reference_filesystem_vote.violation());
        // 挂载透明的射程只管不带故障的状态（D13（验证路线） 已定项 1）：第一次崩溃那个态注入过读故障的，它上面那次挂载途中的二次崩溃
        // 也不判，与第二截同一个理由记一次没判（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
        // 「二次崩溃镜像在注入态下照判（J2-d）」那一行，改法 G5）。
        violations.extend(match read_faults_in_the_first_crash_state {
            ReadFaultsInTheCrashState::NoneInjected => {
                mount_transparency_violations_of_a_second_crash(
                    tally,
                    fields_read_after_the_first_crash,
                    &fields_read_after_the_second_crash,
                    floor_raises_in_the_uninterrupted_mount_writes,
                    continuation_of_the_uninterrupted_mount,
                    rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
                    persisted_in_the_mount,
                    second.phase,
                )
            }
            ReadFaultsInTheCrashState::Injected => {
                *tally
                    .mount_transparency_not_judged
                    .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
                    .or_insert(0) += 1;
                Vec::new()
            }
        });
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
/// 条款没写的情形（[`RemountIdempotenceNotJudged`]）只计数。再挂被拒时 I-8.4 照旧不判，另判 C670 与改法 G4 那两条
/// （[`judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults`]）；没崩过的那一次挂载被拒（`once` 是没判的理由）时不再挂，两样都不判。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：两次挂载的输入（连同清扫的配置）、计数与发现、这一个二次崩溃落在哪"
)]
fn judge_the_remount_after_a_second_crash(
    tally: &mut CrashInjectionTally,
    findings: &mut FindingsOfOneHistory,
    once: &Result<ContentReadByRecovery, RemountIdempotenceNotJudged>,
    second_crash_image: &RefusalOnTheCrashImage<'_>,
    parameters: &MakeFilesystemParameters,
    space_admission: SpaceAdmission,
    instance_table_sweep: InstanceTableSweepConfiguration,
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
    let remount = writable_mount_alone_with_the_instance_table_sweep_configuration_on(
        second_crash_image.crash_image_materialized,
        parameters,
        space_admission,
        instance_table_sweep,
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
            judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults(
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
/// 改法 G4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「新拒因少一条 C670 同形的判法（J2-b）」那一行）同一个接法：
/// 再挂的拒因是「读阶段判完之后系统配置又读不出」、崩溃态没注入读故障、那一拒的判据在二次崩溃镜像上复算为真 ⇒ 拒得不合法，报红
/// （判官的 [`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）。
/// Q3（用户 2026-10-05 定，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）同一个接法：再挂的拒因是「重建分配器读根环
/// 整环重读一次仍一条不被抛弃的自证有效根都没有」、崩溃态没注入读故障、那一拒的判据在二次崩溃镜像上复算为真 ⇒ 拒得不合法，报红
/// （判官的 [`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）。
/// 拒因是实例表读不出的不判。
// gpu-exempt: witness-criterion-w
fn judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults(
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
    let verdict =
        judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
            second_crash_image.crash_image_materialized,
            second_crash_image.version_landed_on_after_the_crash,
            refusal,
            disk,
            second_crash_image.read_only_read_back,
            second_crash_image.read_faults,
            READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE,
        );
    let Some((reason, conditions_not_met)) =
        reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
            tally, verdict,
        )
    else {
        return;
    };
    let detail = format!(
        "崩在挂载的 {} 里、二次崩溃镜像上再挂被拒，理由 {reason:?}；崩溃态没注入读故障，判据在二次崩溃镜像上复算仍为真，拒得不合法（C670 / G4；判官列出的没满足的条件 {conditions_not_met:?}）：{member}",
        phase.name()
    );
    let model_disagreement = ModelDisagreement::new(
        ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
        "没注入读故障的崩溃态上，见证判据与「读阶段判完之后系统配置又读不出」那一拒的判据都不许为真：二次崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）都该成"
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

/// 第三截那一判的计数与取舍：拒因是判据 N-配置、「读阶段判完之后系统配置又读不出」或「重建分配器读根环整环重读一次仍一条有效根都没有」的，
/// 各记一次判过；判官列出的没满足的条件里有
/// 没注入读故障时判据复算为真的那一条（N-配置 的 C670：[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]；
/// 另一拒的改法 G4：[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]；
/// Q3：[`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）时
/// 再各记一次不合法，交回拒因与全部没满足的条件（报红用）。拒因是「记录有缺口时拒可写」（F1 乙）或 C 读一次没施加的发布的账读不出而记录不齐
/// （`C_MAPPING`）的只记一次判过：判官对它们不列那一类条件。
/// 拒因是实例表读不出的、那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
    tally: &mut CrashInjectionTally,
    verdict: WritableMountRefusalVerdict,
) -> Option<(RefusalReasonOnTheD23List, Vec<RefusalConditionNotMet>)> {
    let (reason, conditions_not_met) = match verdict {
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
            (
                RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
                conditions_not_met,
            )
        }
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
        ) => {
            tally.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
            conditions_not_met,
        } => {
            tally.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage += 1;
            (
                RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
                conditions_not_met,
            )
        }
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
        ) => {
            tally.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
            conditions_not_met,
        } => {
            tally.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator += 1;
            (
                RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
                conditions_not_met,
            )
        }
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
        ) => {
            tally.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason:
                RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
            conditions_not_met,
        } => {
            tally.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify += 1;
            (
                RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
                conditions_not_met,
            )
        }
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                ..
            },
        ) => {
            tally.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records += 1;
            return None;
        }
        WritableMountRefusalVerdict::Illegal {
            reason:
                reason @ RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                    ..
                },
            conditions_not_met,
        } => {
            tally.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records += 1;
            (reason, conditions_not_met)
        }
    };
    let mut criterion_true_without_read_faults = false;
    // 迭代上界是判官列出的条件数（至多五条）；跨轮携带的只有上面那一个布尔；没有提前出口。
    for condition in &conditions_not_met {
        match condition {
            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_) => {
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults += 1;
                criterion_true_without_read_faults = true;
            }
            RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {
                tally.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults += 1;
                criterion_true_without_read_faults = true;
            }
            RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults => {
                tally.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults += 1;
                criterion_true_without_read_faults = true;
            }
            RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            | RefusalConditionNotMet::DiskChangedByTheRefusedMount
            | RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet => {}
        }
    }
    criterion_true_without_read_faults.then_some((reason, conditions_not_met))
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
/// 起点那一段（取号、暖机、第一个文件）在内——2026-09-20 才放开；mkfs 那几段不在内，与崩溃点重放同一条界：
/// mkfs 不是事务，它写到一半的盘面上还没有池，恢复报不出根是对的，拿事务的 oracle 去判只会判出假红
/// （崩溃点重放也从 `mkfs_operation_count` 之后起枚举）。失败那一步的写也不在内（模型还没判过它写出的根）。
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

/// 报告里注明「有给定状态表交的是占位结论」那一行的开头（[`CrashInjectionReport::render`]）。
pub const GIVEN_STATES_TABLES_LEFT_TO_OTHER_HOSTS_NOTE: &str = "两台分片留给别的机器";

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
    /// 这一批判红的崩溃镜像写在哪（这一次调用自己新建的那一份目录，跑完删掉的那一档也照写）；一个都没落盘时是 None。
    pub image_directory: Option<PathBuf>,
    pub image_retention: FailureImageRetention,
    /// 这一批照哪一组断言判（交「重放这一整批」那一行写出来；跑的时候不读它）。
    pub assertions: CrashInjectionAssertions,
    /// 给定状态表的判器叫停了（[`GivenStatesJudge::stop_is_requested`]），没领的那几片里有几段历史：0 是整批都跑了；
    /// 不为 0 时计数与新发现只是领了的那几片的，调用方照「本次未跑」交回、不照它判。
    pub histories_left_by_the_stop_request: u64,
}

impl CrashInjectionReport {
    /// 第一行就是这次用的种子基与规模：种子基是这个测试周期写死的那一个（[`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`]），
    /// 报告与失败信息里不带它就重放不了，下一个周期重抽之后这一批数也就换了（用户 2026-09-20 定案第 7 条）。
    /// 判红的崩溃镜像落在哪写路径（给人看）；比对两份报告用 [`CrashInjectionReport::render_the_conclusions`]。
    #[must_use]
    pub fn render(&self) -> String {
        self.render_given(ImagePathsInTheText::Shown)
    }

    /// 同 [`CrashInjectionReport::render`]，判红的崩溃镜像落在哪不写路径、只写落没落盘与留没留：镜像目录每次调用各不相同
    /// （[`create_the_image_directory_of_a_campaign_call`]），同一批种子跑两遍（线程数不同、判器不同、导入与整份）比的是这一份。
    #[must_use]
    pub fn render_the_conclusions(&self) -> String {
        self.render_given(ImagePathsInTheText::LeftOut)
    }

    /// 两份报告除了判红的崩溃镜像落在哪的路径之外逐样相同：这一批的镜像目录与每个新发现的镜像都只比落没落盘、不比路径
    /// （镜像目录每次调用各不相同，[`create_the_image_directory_of_a_campaign_call`]），别的字段与 `==` 一样逐个比（已知红比全部，不只比文字里那前几个）。
    /// 同一批种子跑两遍、各自落了镜像（GPU 判器那一趟与 CPU 判器那一趟）比的是这一份；结构体的 `==` 连路径一起比，两趟各落了镜像、路径不同就判不等。
    #[must_use]
    pub fn has_the_same_conclusions_as(&self, other: &Self) -> bool {
        let Self {
            first_seed,
            seed_count,
            operations_per_history,
            draw,
            weights,
            execution,
            tally,
            known_red_hits,
            new_findings,
            image_directory,
            image_retention,
            assertions,
            histories_left_by_the_stop_request,
        } = self;
        *first_seed == other.first_seed
            && *seed_count == other.seed_count
            && *operations_per_history == other.operations_per_history
            && *draw == other.draw
            && *weights == other.weights
            && *execution == other.execution
            && *tally == other.tally
            && *known_red_hits == other.known_red_hits
            && new_findings.len() == other.new_findings.len()
            && new_findings
                .iter()
                .zip(&other.new_findings)
                .all(|(finding, other_finding)| finding.has_the_same_conclusions_as(other_finding))
            && image_directory.is_some() == other.image_directory.is_some()
            && *image_retention == other.image_retention
            && *assertions == other.assertions
            && *histories_left_by_the_stop_request == other.histories_left_by_the_stop_request
    }

    fn render_given(&self, image_paths: ImagePathsInTheText) -> String {
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
        match (&self.image_directory, self.image_retention, image_paths) {
            (
                Some(directory),
                FailureImageRetention::KeepTheImageFiles,
                ImagePathsInTheText::Shown,
            ) => {
                let _ = writeln!(text, "判红的崩溃镜像留在 {}", directory.display());
            }
            (
                Some(directory),
                FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
                ImagePathsInTheText::Shown,
            ) => {
                let _ = writeln!(
                    text,
                    "判红的崩溃镜像曾写在 {}，跑完已删",
                    directory.display()
                );
            }
            (Some(_), FailureImageRetention::KeepTheImageFiles, ImagePathsInTheText::LeftOut) => {
                let _ = writeln!(
                    text,
                    "判红的崩溃镜像留在这一次调用自己的镜像目录里（路径见 render）"
                );
            }
            (
                Some(_),
                FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
                ImagePathsInTheText::LeftOut,
            ) => {
                let _ = writeln!(
                    text,
                    "判红的崩溃镜像曾写在这一次调用自己的镜像目录里，跑完已删"
                );
            }
            (
                None,
                FailureImageRetention::KeepTheImageFiles
                | FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
                ImagePathsInTheText::Shown | ImagePathsInTheText::LeftOut,
            ) => {}
        }
        if self.histories_left_by_the_stop_request > 0 {
            let _ = writeln!(
                text,
                "叫停了：{} 段历史没领（下面的计数与新发现只是领了的那几片的，这一批不作数）",
                self.histories_left_by_the_stop_request
            );
        }
        if self.tally.given_states_tables_recorded_only > 0 {
            let _ = writeln!(
                text,
                "只跑了录入段：{} 张给定状态表写进库、没判，交的是占位结论（下面的计数与新发现不作数，判段那一趟从库里读回判）",
                self.tally.given_states_tables_recorded_only
            );
        }
        if self.tally.given_states_tables_left_to_other_hosts > 0 {
            let _ = writeln!(
                text,
                "{GIVEN_STATES_TABLES_LEFT_TO_OTHER_HOSTS_NOTE}：{} 张给定状态表归别的机器判、交的是占位结论（下面的计数偏少、新发现可能漏，这一批不作数；导入那一台的库再跑一趟才作数）",
                self.tally.given_states_tables_left_to_other_hosts
            );
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
            text.push_str(&finding.render_given(image_paths));
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
            segment_writes: {
                let segment_writes = positive_count_from_the_variable_text(
                    &short_segment_writes_variable,
                    &short_segment_writes,
                );
                // 整段枚举按 u64 掩码摆子集（`1u64 << 写数`），写数到 64 位移就溢出：上限与抽样那一支的掩码宽同一个数
                assert!(
                    segment_writes <= WRITES_A_SUBSET_MASK_HOLDS,
                    "{short_segment_writes_variable}={short_segment_writes:?} 认不出：短段整段枚举按 u64 掩码摆子集，写数上限最多 {WRITES_A_SUBSET_MASK_HOLDS}"
                );
                segment_writes
            },
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

/// 这个进程里建过几次战役的镜像目录（[`create_the_image_directory_of_a_campaign_call`] 每次取一个号，名字里带它）。
static IMAGE_DIRECTORIES_CREATED_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 一次战役调用试几个名字建镜像目录：名字撞上已有的目录（同进程号的旧进程留下的）就换下一个号，试满还建不出就不留现场。
const MOST_IMAGE_DIRECTORY_NAMES_TRIED: u32 = 64;

/// 给一次战役调用新建一个它独占的空镜像目录（临时目录下，名字带进程号、种子基与这个进程里的调用号；`create_dir` 排他新建，撞上已有的换号）。
/// 同一个进程里同一个种子基的几条用例并行跑时各写各的、各删各的：只由进程号与种子基定名时，一条用例收尾删掉的是另一条正在写的目录，
/// 报告里「判红的崩溃镜像曾写在……」那一行也跟着别的用例变。建不出（临时目录写不了、试满 [`MOST_IMAGE_DIRECTORY_NAMES_TRIED`] 个名字）交回
/// `None`：这一次不留现场（留现场是为了好查，不是判据，同 `write_crash_image_files`）。
fn create_the_image_directory_of_a_campaign_call(first_seed: u64) -> Option<PathBuf> {
    // 迭代上界：MOST_IMAGE_DIRECTORY_NAMES_TRIED 次；只有名字撞上已有目录时才试下一个
    for _name_tried in 0..MOST_IMAGE_DIRECTORY_NAMES_TRIED {
        let call_number = IMAGE_DIRECTORIES_CREATED_IN_THIS_PROCESS.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "singlefs-crash-injection-{}-seedbase-{first_seed}-call-{call_number}",
            std::process::id()
        ));
        match std::fs::create_dir(&directory) {
            Ok(()) => return Some(directory),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_unwritable) => return None,
        }
    }
    None
}

/// 一次战役调用跑完收尾它自己那一份镜像目录：里面一个文件都没有（没有判红的崩溃状态落盘）就删掉、交回 `None`；有的照 `retention`
/// 留着或删掉，交回它的路径（报告里那一行「留在」「曾写在」照它写）。只碰 `directory` 这一份，不碰别的调用的目录。
fn settle_the_image_directory_of_the_call(
    directory: &Path,
    retention: FailureImageRetention,
) -> Option<PathBuf> {
    let images_were_written =
        std::fs::read_dir(directory).is_ok_and(|mut entries| entries.next().is_some());
    match (images_were_written, retention) {
        (
            false,
            FailureImageRetention::KeepTheImageFiles
            | FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        ) => {
            let _ = std::fs::remove_dir(directory);
            None
        }
        (true, FailureImageRetention::KeepTheImageFiles) => Some(directory.to_path_buf()),
        (true, FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes) => {
            let _ = std::fs::remove_dir_all(directory);
            Some(directory.to_path_buf())
        }
    }
}

/// 一次战役调用的镜像目录（[`create_the_image_directory_of_a_campaign_call`] 建的那一份）的守卫：照常跑完由
/// [`Self::settle_into_the_report`] 收尾（照 `retention` 留或删、交回报告里点名的路径）；没走到那一步就丢掉（工作线程 panic、
/// 并片的断言失败）时 Drop 照同一份收尾：没写镜像的空目录删掉，写了镜像的照 `retention` 留或删，留下时把路径打到标准错误
/// （`.claude/singlefs-ai-sop/rules/command-safety.md`「测试镜像一律放临时目录」：成功、失败、panic 都删，留现场走显式开关）。
struct ImageDirectoryOfACampaignCall {
    /// 收尾过一次之后是 `None`，Drop 不再碰它。
    directory: Option<PathBuf>,
    retention: FailureImageRetention,
}

impl ImageDirectoryOfACampaignCall {
    fn of(directory: Option<PathBuf>, retention: FailureImageRetention) -> Self {
        Self {
            directory,
            retention,
        }
    }

    fn path(&self) -> Option<&Path> {
        self.directory.as_deref()
    }

    /// 照常跑完的收尾：交回报告里点名的镜像目录（[`settle_the_image_directory_of_the_call`]）。
    fn settle_into_the_report(mut self) -> Option<PathBuf> {
        self.directory.take().and_then(|directory| {
            settle_the_image_directory_of_the_call(&directory, self.retention)
        })
    }
}

impl Drop for ImageDirectoryOfACampaignCall {
    fn drop(&mut self) {
        let Some(directory) = self.directory.take() else {
            return;
        };
        let left_behind = settle_the_image_directory_of_the_call(&directory, self.retention);
        match (left_behind, self.retention) {
            (Some(kept), FailureImageRetention::KeepTheImageFiles) => eprintln!(
                "崩溃注入这一次调用没跑完就停了，判红的崩溃镜像留在 {}",
                kept.display()
            ),
            (Some(_), FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes)
            | (
                None,
                FailureImageRetention::KeepTheImageFiles
                | FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            ) => {}
        }
    }
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，
/// 线程按片号从小到大领片，调用线程收到一片就打一行 `CRASH_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），
/// 再按片号从小到大并计数。计数按片的次序相加、新发现按种子从小到大留第一个，所以
/// [`CrashInjectionReport::render_the_conclusions`] 与线程数、调度次序无关（进度行是按到达次序打的，只报跑到哪了，不带判定；
/// 判红的崩溃镜像每次调用写进自己新建的那一份目录，[`create_the_image_directory_of_a_campaign_call`]，路径不进那一份）。
///
/// # Panics
/// 某个工作线程 panic（历史里的 panic 在执行器里接住，走到这里的是崩溃状态上恢复或 checker 的断言）；有一片领了却没交回；
/// `draw` 是靶向用例那一支 [`CrashPointDraw::ExactlyOneCrashState`]（只经 [`inject_crashes_into_history`] 摆，不进战役）。
#[must_use]
pub fn run_crash_injection_campaign(
    campaign: &CrashInjectionCampaign,
    given_states_judge: &dyn GivenStatesJudge,
) -> CrashInjectionReport {
    run_crash_injection_campaign_judging_the_given_states_with(campaign, given_states_judge)
}

/// [`run_crash_injection_campaign`]，第一截与第三截的给定状态表交 `given_states_judge` 判（崩溃点重放的引擎经它接进来，
/// `crate::crash_amplification::GivenStatesEngine`；各工作线程共用这一个判器）。
///
/// # Panics
/// 同 [`run_crash_injection_campaign`]；`given_states_judge` 判不了某一张表。
#[must_use]
pub fn run_crash_injection_campaign_judging_the_given_states_with(
    campaign: &CrashInjectionCampaign,
    given_states_judge: &dyn GivenStatesJudge,
) -> CrashInjectionReport {
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
    let image_directory = ImageDirectoryOfACampaignCall::of(
        create_the_image_directory_of_a_campaign_call(first_seed),
        image_retention,
    );
    println!(
        "CRASH_INJECTION_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} images={}",
        first_seed + seed_count,
        slices.len(),
        worker_threads.count(),
        worker_threads.source_name(),
        image_directory
            .path()
            .map_or_else(|| "none".to_string(), |directory| directory.display().to_string())
    );
    let next_slice_index = AtomicUsize::new(0);
    let (finished_slices, merged_slice_count) = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<HistoryCrashInjection>)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            let image_directory = image_directory.path();
            scope.spawn(move || loop {
                // 领一片之前问叫停（同流水线那一侧「每领一块之前先问」）：叫停了就不再领，领了的片照样判完交回
                if given_states_judge.stop_is_requested() {
                    break;
                }
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
                        inject_crashes_into_history_judging_the_given_states_with(
                            &history,
                            execution,
                            draw,
                            image_directory,
                            given_states_judge,
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
    // 领片的下标从 0 起一个接一个发，叫停只让后面的片没人领：领了的片都交回了，就是并进来的那几片
    let slices_taken = next_slice_index.load(Ordering::Relaxed).min(slices.len());
    assert_eq!(
        merged_slice_count, slices_taken,
        "领了的每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    let histories_left_by_the_stop_request: u64 = slices[slices_taken..]
        .iter()
        .map(|slice| slice.end - slice.start)
        .sum();
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
    let image_directory_of_the_report = image_directory.settle_into_the_report();
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
        image_directory: image_directory_of_the_report,
        image_retention,
        assertions,
        histories_left_by_the_stop_request,
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

    /// 同一个进程里同一个种子基的两次战役调用并发建镜像目录（两条用例并行跑）：各得一份不同的、新建的空目录；没写镜像的那一次收尾
    /// 只删自己那一份、报告不点名镜像目录，另一次写下的镜像还在；写了镜像的那一次收尾删掉自己那一份、报告点名它。
    /// 判别力（`crates/mutations.tsv`）：目录名退回只由进程号与种子基定，第二次调用的名字恒撞上第一次的，建不出自己那一份。
    #[test]
    fn two_campaign_calls_with_the_same_seed_base_in_one_process_each_get_and_settle_their_own_image_directory(
    ) {
        use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
        let (directory_of_one_call, directory_of_another_call) = std::thread::scope(|scope| {
            let one_call = scope.spawn(|| {
                create_the_image_directory_of_a_campaign_call(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE)
            });
            let another_call = scope.spawn(|| {
                create_the_image_directory_of_a_campaign_call(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE)
            });
            (
                one_call.join().expect("建镜像目录的线程不 panic"),
                another_call.join().expect("建镜像目录的线程不 panic"),
            )
        });
        let one_call = directory_of_one_call.expect("这一次调用建得出自己的镜像目录");
        let another_call = directory_of_another_call.expect("另一次调用也建得出自己的镜像目录");
        assert_ne!(
            one_call, another_call,
            "同进程、同种子基的两次调用各一份目录"
        );
        for directory in [&one_call, &another_call] {
            assert!(
                std::fs::read_dir(directory)
                    .expect("新建的目录读得了")
                    .next()
                    .is_none(),
                "新建的 {} 是空的",
                directory.display()
            );
        }
        let image_of_one_call = one_call.join("device-0.img");
        std::fs::write(
            &image_of_one_call,
            b"crash image of the call that judged red",
        )
        .expect("写得进这一次调用的镜像目录");
        assert_eq!(
            settle_the_image_directory_of_the_call(
                &another_call,
                FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes
            ),
            None,
            "没写镜像的那一次：报告不点名镜像目录"
        );
        assert!(
            !another_call.exists(),
            "没写镜像的那一次删了自己那一份空目录"
        );
        assert!(
            image_of_one_call.exists(),
            "另一次调用收尾不删这一次写下的镜像"
        );
        assert_eq!(
            settle_the_image_directory_of_the_call(
                &one_call,
                FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes
            ),
            Some(one_call.clone()),
            "写了镜像的那一次：报告点名它自己那一份目录"
        );
        assert!(!one_call.exists(), "写了镜像的那一次跑完删掉自己那一份");
    }

    /// 只为看镜像目录收没收尾的判器替身：`stop_is_requested` 照 `stop` 答（`Panic` 是问它就 panic，造工作线程 panic），一张表都不判。
    struct JudgeOnlyAnsweringTheStopRequest {
        stop: StopRequestAnswer,
    }

    /// 判器替身怎么答叫停。
    enum StopRequestAnswer {
        StopBeforeAnySlice,
        Panic,
    }

    impl GivenStatesJudge for JudgeOnlyAnsweringTheStopRequest {
        fn stop_is_requested(&self) -> bool {
            match self.stop {
                StopRequestAnswer::StopBeforeAnySlice => true,
                StopRequestAnswer::Panic => panic!("判器替身问叫停就 panic：造一次工作线程 panic"),
            }
        }

        fn judge_the_given_states(
            &self,
            table: &GivenStatesOfATable<'_>,
        ) -> Result<GivenStatesConclusionsOfATable, String> {
            Err(format!(
                "判器替身只答叫停、不判表 {}：领片之前就停了，走不到这里",
                table.table_name
            ))
        }
    }

    /// 一次战役：种子基 `first_seed` 起 4 段、每段 1 步，两个工作线程，判红的镜像跑完就删。
    fn campaign_of_four_histories_from(first_seed: u64) -> CrashInjectionCampaign {
        CrashInjectionCampaign {
            first_seed,
            seed_count: 4,
            operations_per_history: 1,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 1,
            },
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(2),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        }
    }

    /// 临时目录里这个进程以种子基 `first_seed` 建的战役镜像目录（名字照 [`create_the_image_directory_of_a_campaign_call`]）。
    fn image_directories_of_this_process_with_the_seed_base(first_seed: u64) -> Vec<String> {
        let prefix = format!(
            "singlefs-crash-injection-{}-seedbase-{first_seed}-call-",
            std::process::id()
        );
        std::fs::read_dir(std::env::temp_dir())
            .expect("临时目录读得了")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(&prefix))
            .collect()
    }

    /// 只给「照常跑完收尾」那一条用的种子基：别的用例不用它，临时目录里带它的镜像目录只会是那一条建的。
    const SEED_BASE_OF_THE_CAMPAIGN_THAT_ENDS_NORMALLY: u64 = 0x7e3a_0d1e_c7a5_e001;

    /// 只给「工作线程 panic」那一条用的种子基。
    const SEED_BASE_OF_THE_CAMPAIGN_WHOSE_WORKER_PANICS: u64 = 0x7e3a_0d1e_c7a5_e002;

    /// 场景：一次战役照常跑完（判器在领第一片之前就叫停，一段历史都不跑）。
    /// 预期：报告不点名镜像目录，这次调用建的那一份镜像目录不在临时目录里了。
    /// 判别力（`crates/mutations.tsv`）：照常跑完那一步收尾不删目录，目录还在，这条红。
    #[test]
    fn a_campaign_call_that_ends_normally_leaves_no_image_directory_behind() {
        let first_seed = SEED_BASE_OF_THE_CAMPAIGN_THAT_ENDS_NORMALLY;
        let report = run_crash_injection_campaign(
            &campaign_of_four_histories_from(first_seed),
            &JudgeOnlyAnsweringTheStopRequest {
                stop: StopRequestAnswer::StopBeforeAnySlice,
            },
        );
        assert_eq!(
            report.histories_left_by_the_stop_request, 4,
            "四段都被叫停留下"
        );
        assert_eq!(report.image_directory, None, "没写镜像：报告不点名镜像目录");
        assert_eq!(
            image_directories_of_this_process_with_the_seed_base(first_seed),
            Vec::<String>::new(),
            "照常跑完之后这次调用的镜像目录不在了"
        );
    }

    /// 场景：一次战役的工作线程 panic（判器替身被问叫停就 panic），战役没走到照常收尾那一步就 panic 着出去。
    /// 预期：panic 接住之后，这次调用建的那一份镜像目录不在临时目录里了。
    /// 判别力（`crates/mutations.tsv`）：镜像目录守卫的 Drop 不收尾，目录还在，这条红。
    #[test]
    fn a_campaign_call_whose_worker_thread_panics_leaves_no_image_directory_behind() {
        let first_seed = SEED_BASE_OF_THE_CAMPAIGN_WHOSE_WORKER_PANICS;
        let panicked = std::panic::catch_unwind(|| {
            run_crash_injection_campaign(
                &campaign_of_four_histories_from(first_seed),
                &JudgeOnlyAnsweringTheStopRequest {
                    stop: StopRequestAnswer::Panic,
                },
            )
        });
        assert!(panicked.is_err(), "工作线程 panic 之后战役 panic 着出去");
        assert_eq!(
            image_directories_of_this_process_with_the_seed_base(first_seed),
            Vec::<String>::new(),
            "panic 之后这次调用的镜像目录不在了"
        );
    }

    /// 一份落了镜像的报告：一个新发现，这一批的镜像目录与那个新发现的镜像都落在 `image_directory` 底下。
    fn report_whose_images_landed_under(image_directory: &Path) -> CrashInjectionReport {
        use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
        let crash_point = CrashPoint {
            segment_index: 1,
            persisted_within_the_segment: vec![true, false],
            step: StepPosition::Operation(3),
            operation_kind: None,
        };
        let finding = CrashPointFinding {
            signature: FailureSignature::CheckerViolations {
                invariants: vec!["I-3.10"],
            },
            seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
            crash_point,
            stage: CrashStateStage::CrashImage,
            observation: FailureObservation {
                position: StepPosition::Operation(3),
                operation_kind: None,
                violations: vec![("I-3.10", "盘 0 槽 40 的分配代 4，诞生代号 5".to_string())],
                panic: None,
                newest_ring_root_txg: Some(5),
                root_ring_slot_count: Some(64),
                harness_judgement: None,
                model_disagreement: None,
                raised_floor_lands_only_on_abandoned_roots: None,
                record_check: RecordCheck::default(),
            },
            read_back: "第 5 版的文件".to_string(),
            image_files: Some(image_directory.join("seed-crash_image")),
        };
        CrashInjectionReport {
            first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            seed_count: 2,
            operations_per_history: 8,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 2,
            },
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            tally: CrashInjectionTally::default(),
            known_red_hits: Vec::new(),
            new_findings: vec![finding],
            image_directory: Some(image_directory.to_path_buf()),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
            histories_left_by_the_stop_request: 0,
        }
    }

    /// 同一批种子跑两遍、两遍都落了镜像（GPU 判器那一趟与 CPU 判器那一趟，镜像目录各是自己新建的那一份）：路径不同而结论相同的两份报告
    /// 判相同，结构体的 `==` 判不同；结论差一处（新发现读回的不同）的两份判不同，只差有没有落镜像的两份也判不同。
    /// 判别力（`crates/mutations.tsv`）：退回按结构体比，路径不同的那两份判成不同，这条红。
    #[test]
    fn two_reports_whose_images_landed_in_different_directories_have_the_same_conclusions_when_only_the_paths_differ(
    ) {
        let gpu_run = report_whose_images_landed_under(Path::new(
            "/nonexistent/singlefs-crash-injection-gpu-run-call-0",
        ));
        let cpu_run = report_whose_images_landed_under(Path::new(
            "/nonexistent/singlefs-crash-injection-cpu-run-call-1",
        ));
        assert_ne!(
            gpu_run, cpu_run,
            "前提：两份的镜像路径不同，结构体的 == 连路径一起比、判不同"
        );
        assert!(
            gpu_run.has_the_same_conclusions_as(&cpu_run),
            "路径不同而结论相同的两份判相同：\n{}\n{}",
            gpu_run.render_the_conclusions(),
            cpu_run.render_the_conclusions()
        );
        let mut differs_in_what_was_read_back = cpu_run.clone();
        differs_in_what_was_read_back.new_findings[0].read_back = "第 4 版的文件".to_string();
        assert!(
            !gpu_run.has_the_same_conclusions_as(&differs_in_what_was_read_back),
            "新发现读回的不同，判不同"
        );
        let mut no_image_landed = cpu_run.clone();
        no_image_landed.image_directory = None;
        no_image_landed.new_findings[0].image_files = None;
        assert!(
            !gpu_run.has_the_same_conclusions_as(&no_image_landed),
            "一边落了镜像、一边没落，判不同"
        );
    }
    use singlefs_harness::history::generate_history;
    use singlefs_harness::mount_transparency::WitnessedCounterBranchThatHolds;

    const FOUR_CRASH_POINTS: CrashPointDraw = CrashPointDraw::Sampled {
        crash_points_per_history: 4,
    };

    /// 这份文件的单测起崩溃点重放的引擎判给定状态表：表与结论都进家目录下那一份判定库（判过的复用），配了 GPU 且编进了
    /// `gpu` 特性上卡、否则 CPU。`label_of_the_report` 只进项名（行里报的名字），种子与规模不进键（块键只由表的内容定）。
    fn given_states_engine_for(
        label_of_the_report: &str,
    ) -> crate::crash_amplification::GivenStatesEngine {
        // 起引擎时拿着改环境的锁：同一个进程里别的单测拿着环境守卫改叫停文件、分片与库目录
        let _one_test_at_a_time =
            crate::crash_amplification::one_test_changes_the_environment_at_a_time();
        crate::crash_amplification::GivenStatesEngine::start_from_environment(
            crate::crash_amplification::GivenStatesItem {
                item_name: format!("crash_injection_unit_tests::{label_of_the_report}"),
            },
            &crate::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    }

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
        let phases = writable_mount_phase_of_each_write(&writes, Some(2), None);
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

    /// 二次崩溃的种子混进第一次崩溃那个段内子集（攻方改法 F3，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第五节第 1 行）：
    /// 同一历史种子（7）、同一段号（5）、同一份挂载写表，段内子集不同（第 0 个写落了 / 第 1 个写落了，各三个写）的两个第一次崩溃态，
    /// 摆出的二次崩溃不同。挂载写表造成两段：写行 4 个写、暖机 8 个写。怎么挑的：在 harness 档草稿探针里照
    /// [`draw_second_crash_points`] 原样抄一份算这组输入，混子集时两份摆法在写行与暖机两段的持久集合都不同，不混时逐个相同。
    /// 判别力：种子不混子集（`crates/mutations.tsv` 一行，指纹换成空子集的）时两份摆法逐个相同，这一条红。
    #[test]
    fn two_first_crash_states_differing_only_in_the_persisted_subset_draw_different_second_crashes()
    {
        let phase_of_each_write: Vec<WritableMountPhase> = (0..12)
            .map(|write_index| {
                if write_index < 4 {
                    WritableMountPhase::RowPublish
                } else {
                    WritableMountPhase::WarmUp
                }
            })
            .collect();
        let segments: Vec<Vec<usize>> = vec![(0..4).collect(), (4..12).collect()];
        let first_crash_with = |persisted_within_the_segment: Vec<bool>| CrashPoint {
            segment_index: 5,
            persisted_within_the_segment,
            step: StepPosition::Operation(0),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
        };
        let first_write_landed = first_crash_with(vec![true, false, false]);
        let second_write_landed = first_crash_with(vec![false, true, false]);
        let drawn_after_the_first_write_landed = draw_second_crash_points(
            HistorySeed(7),
            &first_write_landed,
            &segments,
            &phase_of_each_write,
        );
        let drawn_after_the_second_write_landed = draw_second_crash_points(
            HistorySeed(7),
            &second_write_landed,
            &segments,
            &phase_of_each_write,
        );
        assert_eq!(
            drawn_after_the_first_write_landed,
            draw_second_crash_points(
                HistorySeed(7),
                &first_write_landed,
                &segments,
                &phase_of_each_write
            ),
            "前提：同一个第一次崩溃态摆两次，摆法相同"
        );
        assert_ne!(
            drawn_after_the_first_write_landed, drawn_after_the_second_write_landed,
            "同一历史种子、同一段号、段内子集不同的两个第一次崩溃态要摆出不同的二次崩溃"
        );
    }

    /// 二次崩溃的种子把段内子集整个折进去（攻方改法 F3-c，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节
    /// 「K2 F3 内联用例判别力」与第三节第 2 条）：同一历史种子（0）、同一段号（16）、长 12 的段，段内扣下集合非空的全部 4095 个第一次崩溃态
    /// （12 个写的 4096 个持久子集去掉全持久的那一个：全持久不是崩在这一段里的态）各算一次种子，两两不同。上一条只比两个子集，
    /// 两个子集的第一个写一个落了一个没落，只折第一个写的弱指纹也分得开。攻方在副本上量过：入库指纹 4095 个种子各不相同，
    /// 只折第一个写落没落时剩 2 个，只取第一个被扣下的写的下标时剩 12 个（`research/prompts/m3-formal-code-b21-r1-opus-model/out/f3.txt`）。
    /// 判别力：指纹只折段内第一个写落没落（`crates/mutations.tsv` 一行）时种子只剩 2 个，这一条红。
    #[test]
    fn all_first_crash_states_of_a_segment_of_twelve_writes_seed_their_second_crashes_differently()
    {
        const WRITES_OF_THE_SEGMENT: usize = 12;
        let all_persisted_mask = (1_u32 << WRITES_OF_THE_SEGMENT) - 1;
        let seeds: BTreeSet<SecondCrashPointSeed> = (0..all_persisted_mask)
            .map(|persisted_mask| {
                seed_of_the_second_crash_points(
                    HistorySeed(0),
                    &CrashPoint {
                        segment_index: 16,
                        persisted_within_the_segment: (0..WRITES_OF_THE_SEGMENT)
                            .map(|write_within_the_segment| {
                                (persisted_mask >> write_within_the_segment) & 1 == 1
                            })
                            .collect(),
                        step: StepPosition::Operation(0),
                        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
                    },
                )
            })
            .collect();
        assert_eq!(
            seeds.len(),
            4095,
            "长 12 的段上 4095 个第一次崩溃态（扣下集合非空）算出的二次崩溃种子两两不同"
        );
    }

    /// F3-d 三条用例变的是段末这么多个写的持久子集。
    const WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT: usize = 12;

    /// 段末 [`WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT`] 个写的持久子集里扣下集合非空的那几种（全持久不是崩在这一段里的态）：4095 种。
    const FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT: usize =
        (1 << WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT) - 1;

    /// 一个段上的第一次崩溃态逐个算二次崩溃的种子：段号 `segment_index`，段内前 `writes_landed_before_the_varied_end` 个写都落了，
    /// 段末 [`WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT`] 个写取扣下集合非空的全部 4095 种（段末第 k 个写落没落取掩码第 k 位），
    /// 次序照掩码从 0 起。F3-d 三条用例共用这一份搭建，各自数种子、各自断言。
    fn seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
        seed: HistorySeed,
        segment_index: usize,
        writes_landed_before_the_varied_end: usize,
    ) -> Vec<SecondCrashPointSeed> {
        (0..FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT)
            .map(|persisted_mask_of_the_varied_end| {
                seed_of_the_second_crash_points(
                    seed,
                    &CrashPoint {
                        segment_index,
                        persisted_within_the_segment: std::iter::repeat_n(
                            true,
                            writes_landed_before_the_varied_end,
                        )
                        .chain((0..WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT).map(
                            |write_within_the_varied_end| {
                                (persisted_mask_of_the_varied_end >> write_within_the_varied_end)
                                    & 1
                                    == 1
                            },
                        ))
                        .collect(),
                        step: StepPosition::Operation(0),
                        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
                    },
                )
            })
            .collect()
    }

    /// 二次崩溃的种子跨段号也两两不同（攻方改法 F3-d，判决 `research/prompts/m3-formal-code-b23-r1-main-verification.md` 第二节
    /// 「K1 F3-c 4095 断言的判别力」与第三节第 2 条）：历史种子 0，段号 0..=63 各取长 12 的段上扣下集合非空的 4095 个第一次崩溃态，
    /// 64 × 4095 = 262080 个种子合成一个集合数，两两不同——不只是同一段号里各自不同，段号 a 上的一个子集与段号 b 上的另一个子集也不撞
    /// （攻方 `f3_fix` 的 A 那一格就是这样数的：`research/prompts/m3-formal-code-b23-r1-opus-model/probe/opus_probe.rs`）。
    /// 上一条固定段号 16，种子里去掉段号、或四样改成或，它照绿。
    /// 判别力：种子不异或段号、指纹改成按位拼接（段号与子集掩码在低位撞）、四样改成或（`crates/mutations.tsv` 各一行）时这一条红；
    /// 攻方在模型上量过三种各剩 4095、4096、13254 个（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 A 那一列）。
    #[test]
    fn first_crash_states_of_segments_zero_to_sixty_three_seed_their_second_crashes_differently_across_segments(
    ) {
        const SEGMENT_INDEXES: std::ops::RangeInclusive<usize> = 0..=63;
        let seeds: Vec<SecondCrashPointSeed> = SEGMENT_INDEXES
            .flat_map(|segment_index| {
                seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                    HistorySeed(0),
                    segment_index,
                    0,
                )
            })
            .collect();
        assert_eq!(
            seeds.len(),
            SEGMENT_INDEXES.count() * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
            "前提：段号 0..=63 各 4095 个第一次崩溃态，共 262080 个"
        );
        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
        assert_eq!(
            distinct_seeds.len(),
            seeds.len(),
            "历史种子 0、段号 0..=63、长 12 的段上 262080 个第一次崩溃态算出的二次崩溃种子跨段号也两两不同"
        );
    }

    /// 长于 12 的段末 12 个写也折进种子（F3-d，同上判决）：历史种子 0、段号 16、长 28 的段，前 16 个写都落了、末 12 个写取扣下集合非空的
    /// 4095 种，4095 个种子两两不同。真历史里的候选段长到 26（384 槽）/ 30（4 GiB）个写（攻方 `research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-segments.txt`），
    /// 长 12 的段只看得见前 12 个写：只折前 12 / 16 个写的指纹在上一条照绿，在这里末 12 个写全被丢掉。
    /// 判别力：指纹只折段内前 12 个写、只折前 16 个写（`crates/mutations.tsv` 各一行）时 4095 个第一次崩溃态只剩 1 个种子，这一条红
    /// （攻方量过，`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 B 那一列）。
    #[test]
    fn first_crash_states_of_a_segment_of_twenty_eight_writes_differing_only_in_the_last_twelve_seed_their_second_crashes_differently(
    ) {
        const WRITES_LANDED_BEFORE_THE_VARIED_END: usize = 16;
        let seeds = seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
            HistorySeed(0),
            16,
            WRITES_LANDED_BEFORE_THE_VARIED_END,
        );
        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
        assert_eq!(
            distinct_seeds.len(),
            FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
            "长 28 的段、前 16 个写都落了、末 12 个写取 4095 种：二次崩溃种子两两不同"
        );
    }

    /// 二次崩溃的种子随历史种子变（F3-d，同上判决）：段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 1 上各算一遍，
    /// 8190 个种子合成一个集合数，两两不同。
    /// 判别力：种子不异或历史种子（`crates/mutations.tsv` 一行）时两个历史种子上同一子集撞成一个，这一条红；
    /// 攻方在模型上量过它剩 4095 个，按位拼接剩 4096 个、四样改成或剩 5998 个（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 C 那一列）。
    #[test]
    fn first_crash_states_under_history_seeds_zero_and_one_seed_their_second_crashes_differently() {
        const HISTORY_SEEDS: [HistorySeed; 2] = [HistorySeed(0), HistorySeed(1)];
        let seeds: Vec<SecondCrashPointSeed> = HISTORY_SEEDS
            .into_iter()
            .flat_map(|seed| {
                seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                    seed, 16, 0,
                )
            })
            .collect();
        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
        assert_eq!(
            distinct_seeds.len(),
            HISTORY_SEEDS.len() * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
            "段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 1 上算出的 8190 个二次崩溃种子两两不同"
        );
    }

    /// 二次崩溃的种子在段号过 63 之后也两两不同（攻方改法 F3-e 的 A2，判决 `research/prompts/m3-formal-code-b24-r1-main-verification.md`
    /// 第二节「K1 改法 F3-e」与第三节第 2 条；原型 `research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-probe/src/main.rs` 的
    /// `f3_proposed`）：历史种子 0，段号 0..=63 再加 2^k、2^k + 1（k = 6..=20）共 94 个段号，各取长 12 的段上扣下集合非空的 4095 个第一次崩溃态，
    /// 384930 个种子合成一个集合数，两两不同。真历史里的段号过得了 63（2026-10-10 写这一句时量的战役大档到 210；战役的段从整条流起点数、含 mkfs，
    /// mkfs 作废屏障之后同一段历史各段号加 1，今天的数没量），段号 0..=63 那一条看不见只取段号低 6 位的种子。
    /// 被攻过零轮：这一组取样（段号 2^k、2^k + 1）对更长的历史、别的种子基罩不罩得住没人攻过。
    /// 判别力：段号只取低 6 位（`crates/mutations.tsv` 一行，攻方 M7d）时段号 64 与 0 撞，这一条红。
    #[test]
    fn first_crash_states_of_segments_past_sixty_three_seed_their_second_crashes_differently_across_segments(
    ) {
        let segment_indexes: Vec<usize> = (0..=63)
            .chain((6..=20).flat_map(|exponent| [1_usize << exponent, (1_usize << exponent) + 1]))
            .collect();
        let seeds: Vec<SecondCrashPointSeed> = segment_indexes
            .iter()
            .flat_map(|segment_index| {
                seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                    HistorySeed(0),
                    *segment_index,
                    0,
                )
            })
            .collect();
        assert_eq!(
            seeds.len(),
            94 * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
            "前提：94 个段号各 4095 个第一次崩溃态，共 384930 个"
        );
        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
        assert_eq!(
            distinct_seeds.len(),
            seeds.len(),
            "历史种子 0、段号 0..=63 与 2^k、2^k + 1（k = 6..=20）、长 12 的段上 384930 个第一次崩溃态算出的二次崩溃种子两两不同"
        );
    }

    /// 长 32 的段里扣下的写落在任何位置都折进种子（攻方改法 F3-e 的 B3，同上判决与原型）：历史种子 0、段号 16、长 32 的段，扣下其中一个写
    /// （32 种）或两个写（496 种），528 个第一次崩溃态的种子两两不同。上面长 28 的那一条只变段末 12 个写，只折段末 12 个写的指纹在它上面照绿。
    /// 被攻过零轮：段长 32、只扣一两个写的取样对更长的段、扣得更多的子集罩不罩得住没人攻过。
    /// 判别力：指纹只折段内末 12 个写（`crates/mutations.tsv` 一行，攻方 M7a）时扣下前 20 个写里哪一个都撞成一个种子，这一条红。
    #[test]
    fn first_crash_states_of_a_segment_of_thirty_two_writes_withholding_one_or_two_writes_anywhere_seed_their_second_crashes_differently(
    ) {
        const WRITES_OF_THE_SEGMENT: usize = 32;
        let first_crash_with_withheld = |withheld: &[usize]| CrashPoint {
            segment_index: 16,
            persisted_within_the_segment: (0..WRITES_OF_THE_SEGMENT)
                .map(|write_within_the_segment| !withheld.contains(&write_within_the_segment))
                .collect(),
            step: StepPosition::Operation(0),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
        };
        let seeds: Vec<SecondCrashPointSeed> = (0..WRITES_OF_THE_SEGMENT)
            .flat_map(|first_withheld| {
                std::iter::once(vec![first_withheld]).chain(
                    (first_withheld + 1..WRITES_OF_THE_SEGMENT)
                        .map(move |second_withheld| vec![first_withheld, second_withheld]),
                )
            })
            .map(|withheld| {
                seed_of_the_second_crash_points(
                    HistorySeed(0),
                    &first_crash_with_withheld(&withheld),
                )
            })
            .collect();
        assert_eq!(
            seeds.len(),
            528,
            "前提：长 32 的段扣下一个写 32 种、两个写 496 种"
        );
        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
        assert_eq!(
            distinct_seeds.len(),
            seeds.len(),
            "历史种子 0、段号 16、长 32 的段上扣下一两个写的 528 个第一次崩溃态算出的二次崩溃种子两两不同"
        );
    }

    /// 二次崩溃的种子随历史种子的高位也变（攻方改法 F3-e 的 C2，同上判决与原型）：段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0、1、2^63
    /// 上各算一遍，12285 个种子两两不同。上面历史种子 0 与 1 那一条只差最低位，只取历史种子最低位的种子在它上面照绿。
    /// 被攻过零轮：历史种子 0、1、2^63 这三个取样对别的种子基罩不罩得住没人攻过。
    /// 判别力：历史种子只取最低位（`crates/mutations.tsv` 一行，攻方 M7f）时 0 与 2^63 撞，这一条红。
    #[test]
    fn first_crash_states_under_history_seeds_zero_one_and_two_to_the_sixty_third_seed_their_second_crashes_differently(
    ) {
        const HISTORY_SEEDS: [HistorySeed; 3] =
            [HistorySeed(0), HistorySeed(1), HistorySeed(1 << 63)];
        let seeds: Vec<SecondCrashPointSeed> = HISTORY_SEEDS
            .into_iter()
            .flat_map(|seed| {
                seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                    seed, 16, 0,
                )
            })
            .collect();
        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
        assert_eq!(
            distinct_seeds.len(),
            HISTORY_SEEDS.len() * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
            "段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0、1、2^63 上算出的 12285 个二次崩溃种子两两不同"
        );
    }

    /// 二次崩溃的种子随历史种子的每一位都变（攻方取样 C3，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
    /// 「F3-e 取样（C4 / J2-f）」那一行与第三节第 1 条、改法 G6；原型 `research/prompts/m3-formal-code-b26-r1-opus-model/probe/src/f3.rs` 的
    /// `F3PROPOSED`）：段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 2^k（k = 0..=63）共 65 个种子上各算一遍，266175 个种子两两不同。
    /// 上面 0、1、2^63 那一条只看得见最低位与最高位，历史种子对 3 取余、只留低 8 位与最高位的种子在它上面照绿（攻方量过，`research/prompts/m3-formal-code-b26-r1-opus-model/out/f3.txt`）。
    /// b24 已是最后一轮，只加宽取样：这一组取样对别的种子形状罩不罩得住没人攻过，剩下的漏形记在 b24 那笔欠账里。
    /// 判别力：历史种子对 3 取余（`crates/mutations.tsv` 一行）时 2^k 与 0、1、2 撞，这一条红。
    #[test]
    fn first_crash_states_under_history_seeds_zero_and_every_power_of_two_seed_their_second_crashes_differently(
    ) {
        let history_seeds: Vec<HistorySeed> = std::iter::once(HistorySeed(0))
            .chain((0..64).map(|bit| HistorySeed(1_u64 << bit)))
            .collect();
        let seeds: Vec<SecondCrashPointSeed> = history_seeds
            .iter()
            .flat_map(|seed| {
                seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                    *seed, 16, 0,
                )
            })
            .collect();
        assert_eq!(
            seeds.len(),
            65 * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
            "前提：历史种子 0 与 2^0..=2^63 共 65 个，各 4095 个第一次崩溃态"
        );
        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
        assert_eq!(
            distinct_seeds.len(),
            seeds.len(),
            "段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 2^k（k = 0..=63）上算出的 266175 个二次崩溃种子两两不同"
        );
    }

    /// 长 1024 的段里扣下的那一个写落在任何位置都折进种子（攻方取样 B4，同上判决、改法 G6 与原型）：历史种子 0、段号 16、长 1024 的段，
    /// 扣下其中一个写（1024 种），1024 个第一次崩溃态的种子两两不同。上面长 32 的那一条看不见第 32 个写之后：指纹只折段内前 32 个或末 32 个写的
    /// 在它上面照绿（攻方量过，`research/prompts/m3-formal-code-b26-r1-opus-model/out/f3.txt`）。
    /// b24 已是最后一轮，只加宽取样：段长 1024、只扣一个写的取样对更长的段、扣得更多的子集罩不罩得住没人攻过，剩下的漏形记在 b24 那笔欠账里。
    /// 判别力：指纹只折段内前 32 个写（`crates/mutations.tsv` 一行）时扣下第 32 个写之后哪一个都撞成一个种子，这一条红。
    #[test]
    fn first_crash_states_of_a_segment_of_one_thousand_twenty_four_writes_withholding_one_write_anywhere_seed_their_second_crashes_differently(
    ) {
        const WRITES_OF_THE_SEGMENT: usize = 1024;
        let seeds: Vec<SecondCrashPointSeed> = (0..WRITES_OF_THE_SEGMENT)
            .map(|withheld| {
                seed_of_the_second_crash_points(
                    HistorySeed(0),
                    &CrashPoint {
                        segment_index: 16,
                        persisted_within_the_segment: (0..WRITES_OF_THE_SEGMENT)
                            .map(|write_within_the_segment| write_within_the_segment != withheld)
                            .collect(),
                        step: StepPosition::Operation(0),
                        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
                    },
                )
            })
            .collect();
        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
        assert_eq!(
            distinct_seeds.len(),
            WRITES_OF_THE_SEGMENT,
            "历史种子 0、段号 16、长 1024 的段上扣下一个写的 1024 个第一次崩溃态算出的二次崩溃种子两两不同"
        );
    }

    /// 实例表清扫两段（D18（块里携带什么信息） 已定项 11「回收」一条）：挂载那一段的写里，第「清扫之前发了几次」个根槽写之后起是清扫抹头
    /// （FUA 写零的单元头、每块盘一道屏障），之后第一个不带 FUA 的单元写起是清扫发布（单元、屏障、记录、根槽、轮换），两段摆二次崩溃时各摆到一个。
    /// 造的写表：取号；写行那次（单元、记录、根槽、轮换）；一次暖机（单元、根槽）；抹两个头、两道屏障；清扫发布——清扫之前发了 2 次。
    /// 判别力：`WritableMountPhase::ALL` 里没有这两段时一个二次崩溃都摆不到；不按 FUA 分开时清扫发布的写归抹头。
    #[test]
    fn the_writes_of_the_instance_table_sweep_split_into_erasing_heads_and_the_sweep_publish_and_each_gets_a_second_crash_point(
    ) {
        let write_of = |kind: StepKind, is_force_unit_access: bool| RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access,
            offset: DeviceOffsetInBytes(0),
            contents: singlefs_harness::memory_pool::WrittenContents::Bytes(vec![0; 512]),
        };
        let writes = vec![
            write_of(StepKind::SystemConfigurationSlot, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::JournalRecord, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::SystemConfigurationSlot, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::UnitWrite, true),
            write_of(StepKind::UnitWrite, true),
            write_of(StepKind::Barrier, false),
            write_of(StepKind::Barrier, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::Barrier, false),
            write_of(StepKind::JournalRecord, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::SystemConfigurationSlot, false),
        ];
        let phases = writable_mount_phase_of_each_write(&writes, None, Some(2));
        assert_eq!(
            phases[7..],
            [
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
            ],
            "抹两个头与两道屏障归抹头，清扫发布从它第一个不带 FUA 的单元写起"
        );
        assert_eq!(
            phases[..7],
            [
                WritableMountPhase::Acquisition,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::WarmUp,
                WritableMountPhase::WarmUp,
            ],
            "清扫之前那几段照旧"
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
        for phase in [
            WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
            WritableMountPhase::InstanceTableSweepPublish,
        ] {
            assert_eq!(
                second_crash_points
                    .iter()
                    .filter(|point| point.phase == phase)
                    .count(),
                1,
                "{} 那一段摆一个二次崩溃：{second_crash_points:?}",
                phase.name()
            );
        }
    }

    /// 清扫一个头都没抹（只删行）：清扫之前最后一次发布的轮换之后，第一个写就是清扫发布的单元写，抹头那一段一个写都没有。
    /// 把清扫发布的第一个写记进抹头那一段的，二次崩溃会说崩在了没写过一个字节的抹头里（假绿）。
    #[test]
    fn a_sweep_that_erased_no_head_puts_every_write_after_the_last_publish_before_it_into_the_sweep_publish(
    ) {
        let write_of = |kind: StepKind, is_force_unit_access: bool| RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access,
            offset: DeviceOffsetInBytes(0),
            contents: singlefs_harness::memory_pool::WrittenContents::Bytes(vec![0; 512]),
        };
        let writes = vec![
            write_of(StepKind::SystemConfigurationSlot, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::JournalRecord, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::SystemConfigurationSlot, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::UnitWrite, false),
            write_of(StepKind::JournalRecord, false),
            write_of(StepKind::RootRecordFua, true),
            write_of(StepKind::SystemConfigurationSlot, false),
        ];
        let phases = writable_mount_phase_of_each_write(&writes, None, Some(1));
        assert_eq!(
            phases,
            [
                WritableMountPhase::Acquisition,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::RowPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
                WritableMountPhase::InstanceTableSweepPublish,
            ],
            "写行那次的轮换之后没有 FUA 写零：清扫发布从它第一个单元写起，抹头那一段是空的"
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
                None,
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

    /// unlink 之后再建过 inode 的一版归带文件的一版、第一个文件那条记录不在（删除与并发那条线读码诊断；攻方随机历史种子 32768、245760 两段那一形，
    /// 判决 `research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节「越出攻击面」那一行）：第一个文件之后 unlink 它、再建一个 inode，
    /// 崩在建 inode 那次发布里——根槽落了、紧跟着的系统配置轮换没落，只读恢复落在建 inode 那一版（这段历史最后提交的一版）。
    /// 崩溃后镜像上的可写挂载做成，之后那一次发布建一个 inode、做成，第二截没有新发现；发布之后那份盘只读挂载读回的 inode 记录是崩溃那一版的
    /// 每一条原样加新建的那一条，新建那一条整条等于第二截那一支的调法该写出的（改法 a，`research/prompts/m3-formal-code-b20-r1-main-verification.md`
    /// 第二节 K2 H1；比整条是改法 A1，`research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节「K2 改法 a 判别力」与第三节第 2 条）。
    /// 判别力：带文件那一臂改回「一律覆盖写」（`crates/mutations.tsv` 一行），覆盖写被拒成写一个不在树里的 inode 1，计一次发布被拒、
    /// 记一条第二截的新发现，这一条红；那一臂建 0 个 inode、新建那一条 nlink = 0、出生代取上一版、size 4096、写入时间 0
    /// （`crates/mutations.tsv` 各一行；攻方量过后四种池级 checker 照样绿，`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2a.txt`），
    /// 红在比整条记录那一条。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个崩溃态（ExactlyOneCrashState），不枚举
    #[test]
    fn the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published(
    ) {
        use singlefs_core::records::{InodeRecord, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES};
        use singlefs_core::transaction::FIRST_INODE_NUMBER;
        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
        let execution = HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        };
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![HistoryOperation::Unlink, HistoryOperation::CreateInode],
        };
        let stream = SharedStream::retaining_contents();
        let mut operations_after_the_unlink = None;
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            if observation.position == StepPosition::Operation(0) {
                operations_after_the_unlink = Some(stream.operation_count());
            }
        });
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "前提：unlink 与建 inode 都跑完"
        );
        let operations_after_the_unlink =
            operations_after_the_unlink.expect("unlink 那一步跑完时登记过一次");
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        let root_of_the_inode_creation = (0..writes.len())
            .rev()
            .find(|write| {
                stream_indexes[*write] >= operations_after_the_unlink
                    && writes[*write].kind == StepKind::RootRecordFua
            })
            .expect("前提：建 inode 那次发布写了根槽");
        assert_eq!(
            writes
                .get(root_of_the_inode_creation + 1)
                .map(|write| write.kind),
            Some(StepKind::SystemConfigurationSlot),
            "前提：建 inode 那次发布的根槽 FUA 之后紧跟着轮换系统配置"
        );
        let segment_of_the_root =
            segment_containing_the_write(&segments, root_of_the_inode_creation);
        let first_write_of_the_segment = segments[segment_of_the_root][0];
        let injection = inject_crashes_into_history(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index: segment_of_the_root,
                writes_from_the_segment_on: root_of_the_inode_creation + 2
                    - first_write_of_the_segment,
                withheld_among_them: vec![
                    root_of_the_inode_creation + 1 - first_write_of_the_segment,
                ]
                .leak(),
            },
            None,
            &given_states_engine_for("crash_injection_unit_tests"),
        );
        let tally = &injection.tally;
        assert_eq!(
            injection
                .crash_points
                .iter()
                .map(|crash_point| crash_point.operation_kind)
                .collect::<Vec<_>>(),
            [Some(HistoryOperationKind::CreateInode)],
            "前提：只摆写死的那一个崩溃态，崩在建 inode 那次发布里"
        );
        assert_eq!(
            tally.crash_points_that_land_before_the_last_committed_version,
            0,
            "前提：建 inode 那次发布的根落了，崩溃态落在这段历史最后提交的那一版上：{}",
            tally.render()
        );
        assert_eq!(
            tally.writable_mounts_after_the_crash_succeeded,
            1,
            "前提：崩溃后镜像上的可写挂载做成：{}",
            tally.render()
        );
        let findings_after_the_writable_mount: Vec<String> = injection
            .new_findings
            .iter()
            .filter(|finding| finding.stage == CrashStateStage::AfterTheWritableMountAndOnePublish)
            .map(CrashPointFinding::render)
            .collect();
        assert_eq!(
            (
                tally.publishes_after_the_writable_mount,
                tally.publishes_after_the_writable_mount_refused
            ),
            (1, 0),
            "第一个文件已删完的带文件一版上，挂载之后那一次发布建一个 inode、做成：{findings_after_the_writable_mount:?}"
        );
        assert!(
            findings_after_the_writable_mount.is_empty(),
            "第二截没有新发现：{findings_after_the_writable_mount:?}"
        );
        // 崩溃态是写表前缀到建 inode 那次的根槽为止（段内摆到 root + 1、扣下 root + 1，之后一个都没落），在同一份盘面上照第二截那一种调法
        // 再挂一次加一次发布，比发布之后那份池与崩溃那一份上只读挂载读回的 inode 号。
        let mut crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        crash_image.apply_writes(&writes[..=root_of_the_inode_creation]);
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &crash_image,
            &execution.device_width.parameters(),
            execution,
            history.seed,
            &injection.crash_points[0],
        );
        assert!(
            matches!(mount_run.outcome, WritableMountOutcome::Published),
            "前提：同一份盘面上再挂一次加一次发布，照样做成：{:?}",
            mount_run.outcome
        );
        let read_only_mount = |pool: &MemoryPool| {
            singlefs_core::mounted_read::mount_read_only(pool)
                .expect("崩溃那一版与发布之后那一版都是整次落盘的版本，只读挂载做得成")
        };
        let crashed_version = read_only_mount(&crash_image);
        let version_after_the_publish = read_only_mount(&mount_run.pool_after);
        // 第一个文件取 1 号（已删完），unlink 之后历史里建的那一个取 2 号：崩溃那一版树里只剩 2 号，inode 号水位 3。
        assert_eq!(
            crashed_version
                .mounted
                .inode_records()
                .iter()
                .map(|record| record.inode)
                .collect::<Vec<_>>(),
            [FIRST_INODE_NUMBER + 1],
            "前提：崩溃那一版树里只剩历史里建的那一个 inode"
        );
        // 照第二截那一支的调法 `publish_new_inodes(…, 1, FIXED_WRITE_TIME_SECONDS, 挂载的实例)` 该写出的那一条：号从水位 3 起发，
        // 对象出生代与改动计数都是这次发布的 checkpoint_txg（发布之后那一版的根），活着（nlink = 1），长度 0（blocks 由 size 现算，跟着是 0）。
        let txg_of_the_publish = version_after_the_publish.effective_root.checkpoint_txg;
        let mut expected_records_after_the_publish =
            crashed_version.mounted.inode_records().to_vec();
        expected_records_after_the_publish.push(InodeRecord {
            inode: FIRST_INODE_NUMBER + 2,
            object_birth: txg_of_the_publish,
            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size: 0,
            change_count: txg_of_the_publish.0,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            // 新建的 inode 没被截断过：截断字段是 0（判决 m4-trunc-r2 R2）。
            truncation_field: singlefs_core::records::InodeRecordTruncationField::NOT_WRITTEN,
        });
        assert_eq!(
            version_after_the_publish.mounted.inode_records(),
            expected_records_after_the_publish.as_slice(),
            "发布之后只读挂载读回的 inode 记录 = 崩溃那一版的每一条原样 + 新建的那一条（号、出生代、nlink、size、改动计数、写入时间逐个对上）"
        );
    }

    /// 挂载每一段都摆得到二次崩溃的两段历史（U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：从第一个文件起的
    /// 随机一段（起点就是攻方 `research/prompts/m3-inv-a-r1-opus-output.md` 一·2 的「新池新建文件做完的镜像」），与第一个文件之后经挂着的会话
    /// unlink 它的一段（意图那次 + 最后一批：崩溃状态有落在意图那一版上的，那一版上的可写挂载续做、写出续做那一段，
    /// 之后那一次发布落在删完那一版上）。两段同一种摆法（短段整段枚举），计数并在一起。再并上一段崩溃后那次可写挂载删得了行的历史
    /// （[`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]，有一行就扫）：
    /// 清扫发布那一段从它来；那段历史上没有孤儿、一个头都不抹。最后并上一段留下实例 1 的孤儿再把实例 1 的根全轮出根环的历史
    /// （[`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans`]，同一种摆法）：
    /// 抹头那一段从它来。
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
        let mut injections: Vec<HistoryCrashInjection> = histories
            .iter()
            .map(|history| {
                inject_crashes_into_history(
                    history,
                    HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                    CrashPointDraw::EveryProperSubsetOfShortSegments {
                        segment_writes: 2,
                        sampled_in_longer_segments: 2,
                    },
                    None,
                    &given_states_engine_for("crash_injection_unit_tests"),
                )
            })
            .collect();
        injections.push(
            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                &given_states_engine_for("crash_injection_unit_tests"),
            ),
        );
        injections.push(
            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                &given_states_engine_for("crash_injection_unit_tests"),
            ),
        );
        let mut tally = CrashInjectionTally::default();
        for injection in &injections {
            tally.absorb(&injection.tally);
        }
        (tally, injections)
    }

    /// 崩溃后那次可写挂载末尾的清扫发布删了行（C704（挂载透明不认清扫发布） 的验收）：挂载做成、删了实例 1 那一行、一个头都没抹；
    /// 只挂载与挂载加一次发布两个比法拿到的 R2 都是清扫发布那一版，实例表行那一样把删的行放回去再比，挂载透明一样都没红；
    /// 清扫发布那一段摆了二次崩溃，再挂判了 I-8.4、挂载透明判了；抹头那一段一个二次崩溃都没摆（没抹头时那一段一个写都没有）；
    /// 一条新发现都没有。判别力：挂载透明不放回删的行时只挂载那一个比法判红（`crates/mutations.tsv` 里点名这条的行）。
    #[test]
    fn a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field(
    ) {
        let injection =
            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                &given_states_engine_for("crash_injection_unit_tests"),
            );
        let tally = &injection.tally;
        let rendered = tally.render();
        assert_eq!(
            (
                tally.crash_points,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.publishes_after_the_writable_mount
            ),
            (1, 1, 1),
            "前提：摆了一个崩溃状态，那次可写挂载与之后那一次发布都做成：{rendered}"
        );
        assert_eq!(
            (
                tally.writable_mounts_after_the_crash_deleting_instance_table_rows,
                tally.heads_erased_by_the_instance_table_sweep_after_the_crash
            ),
            (1, 0),
            "前提：那次挂载末尾的清扫发布删了行、一个头都没抹（这段历史每次发布都做完了，没有孤儿）：{rendered}"
        );
        assert_eq!(
            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back, 2,
            "只挂载与挂载加一次发布两个比法拿到的 R2 都是清扫发布那一版，各放回一次删的行再比：{rendered}"
        );
        let phase_count = |counts: &BTreeMap<WritableMountPhase, u64>, phase| {
            counts.get(&phase).copied().unwrap_or(0)
        };
        assert_eq!(
            phase_count(
                &tally.second_crash_points_by_phase,
                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
            ),
            0,
            "一个头都没抹时抹头那一段一个写都没有，不摆二次崩溃：{rendered}"
        );
        for (what, counts) in [
            ("摆了二次崩溃", &tally.second_crash_points_by_phase),
            ("再挂判了 I-8.4", &tally.remount_idempotence_judged_by_phase),
            (
                "判了挂载透明",
                &tally.mount_transparency_judged_on_second_crashes_by_phase,
            ),
        ] {
            assert!(
                phase_count(counts, WritableMountPhase::InstanceTableSweepPublish) >= 1,
                "清扫发布那一段{what}：{rendered}"
            );
        }
        assert!(
            tally.mount_transparency_violations_by_field.is_empty(),
            "挂载透明判红的字段 {:?}：{}",
            tally.mount_transparency_violations_by_field,
            injection
                .new_findings
                .iter()
                .map(CrashPointFinding::render)
                .collect::<String>()
        );
        assert_eq!(
            tally.remounts_after_a_second_crash_reading_different_content, 0,
            "二次崩溃之后再挂读出的内容与没崩过的那一次不同：{rendered}"
        );
        assert!(
            injection.new_findings.is_empty(),
            "新发现：{}",
            injection
                .new_findings
                .iter()
                .map(CrashPointFinding::render)
                .collect::<String>()
        );
    }

    /// 崩溃后那次可写挂载末尾的清扫先抹孤儿的头、再发清扫发布删行（D18（块里携带什么信息） 已定项 11「回收」一条，抹头那一段）：
    /// 挂载做成、删了行、抹了至少一个头；抹头那一段摆了二次崩溃、再挂判了 I-8.4、判了挂载透明；挂载透明一样都没红、二次崩溃之后再挂
    /// 读出的内容与没崩过的那一次相同、一条新发现都没有。判别力：历史开头那次覆盖写掉电不截写（`singlefs_harness::history` 里
    /// 掉电那一截的判法）时盘上没有孤儿，「抹了至少一个头」那条先红。
    #[test]
    fn a_writable_mount_after_the_crash_whose_sweep_erases_heads_of_orphans_is_judged_in_the_erasing_phase_without_a_red_field(
    ) {
        let injection =
            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                &given_states_engine_for("crash_injection_unit_tests"),
            );
        let tally = &injection.tally;
        let rendered = tally.render();
        assert_eq!(
            (
                tally.crash_points,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.writable_mounts_after_the_crash_deleting_instance_table_rows
            ),
            (1, 1, 1),
            "前提：摆了一个崩溃状态，那次可写挂载做成、末尾的清扫发布删了行：{rendered}"
        );
        assert!(
            tally.heads_erased_by_the_instance_table_sweep_after_the_crash >= 1,
            "前提：那次挂载末尾的清扫抹了孤儿的头：{rendered}"
        );
        let phase_count = |counts: &BTreeMap<WritableMountPhase, u64>, phase| {
            counts.get(&phase).copied().unwrap_or(0)
        };
        for (what, counts) in [
            ("摆了二次崩溃", &tally.second_crash_points_by_phase),
            ("再挂判了 I-8.4", &tally.remount_idempotence_judged_by_phase),
            (
                "判了挂载透明",
                &tally.mount_transparency_judged_on_second_crashes_by_phase,
            ),
        ] {
            assert!(
                phase_count(
                    counts,
                    WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
                ) >= 1,
                "抹头那一段{what}：{rendered}"
            );
        }
        assert!(
            tally.mount_transparency_violations_by_field.is_empty(),
            "挂载透明判红的字段 {:?}：{rendered}",
            tally.mount_transparency_violations_by_field
        );
        assert_eq!(
            tally.remounts_after_a_second_crash_reading_different_content, 0,
            "二次崩溃之后再挂读出的内容与没崩过的那一次不同：{rendered}"
        );
        assert!(
            injection.new_findings.is_empty(),
            "新发现：{}",
            injection
                .new_findings
                .iter()
                .map(CrashPointFinding::render)
                .collect::<String>()
        );
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
        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
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
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
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
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions,
        };
        let sampling = HistorySampling {
            operations_per_history: 3,
            weights: GenerationWeights::BROAD,
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
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
                execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
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

    /// 短段写数超过 u64 掩码的宽（63）：整段枚举 `1u64 << 写数` 会溢出，解析时就停下，消息带变量名、原值与上限。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES=\"64\" 认不出")]
    fn a_crash_injection_short_segment_writes_value_past_the_mask_width_panics_naming_the_limit() {
        let _campaign = campaign_read_back_with_only_this_variable_set(
            "SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES",
            "64",
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
            execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
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

    /// 线程数从环境变量取，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取登记值，不看本机核数）；
    /// 设了不是正整数就停，不悄悄退回单线程。
    #[test]
    fn worker_thread_count_comes_from_the_environment_variable_or_the_heavy_thread_default() {
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                Some("3"),
                Some("5"),
            ),
            CrashInjectionWorkerThreads::FromTheEnvironmentVariable(3),
            "自己的变量先读"
        );
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                None,
                Some("5")
            ),
            CrashInjectionWorkerThreads::FromTheHeavyThreadDefault(5),
            "没设自己的变量读 SINGLEFS_THREAD_CAP"
        );
        let registered = singlefs_harness::heavy_threads::registered_heavy_threads().get();
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
                None,
                None
            ),
            CrashInjectionWorkerThreads::FromTheHeavyThreadDefault(registered),
            "都没设取 .claude/heavy-thread-default 的登记值"
        );
        assert_eq!(
            CrashInjectionWorkerThreads::FromTheHeavyThreadDefault(4).count(),
            4
        );
    }

    /// 线程数设成 0：经数值环境变量共用的那一份解析停下（线程数取 `NonZeroUsize`，0 认不出），消息带变量名与原值，
    /// 不悄悄退回单线程或默认；口径与仓里别的线程数变量相同。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_INJECTION_THREADS=\"0\" 认不出")]
    fn a_crash_injection_worker_thread_count_of_zero_panics_naming_the_variable_and_the_value() {
        let _threads = CrashInjectionWorkerThreads::from_the_environment_value(
            CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
            Some("0"),
            None,
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
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            FOUR_CRASH_POINTS,
            None,
            &given_states_engine_for("crash_injection_unit_tests"),
        );
        let second = inject_crashes_into_history(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            FOUR_CRASH_POINTS,
            None,
            &given_states_engine_for("crash_injection_unit_tests"),
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
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                CrashPointDraw::Sampled {
                    crash_points_per_history: 6,
                },
                None,
                &given_states_engine_for("crash_injection_unit_tests"),
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
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 2,
                sampled_in_longer_segments: 2,
            },
            None,
            &given_states_engine_for("crash_injection_unit_tests"),
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
    fn a_crash_state_counts_as_read_faulted_from_the_earliest_stream_step_of_an_abandoning_recovery_on(
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

    /// 读故障认定拿的崩溃窗口末尾（攻方改法 F5，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：
    /// 写表 6 个写落在录制流第 0、1、3、4、6、9 步（第 2、5、7、8 步是屏障）。从写表第 2 个写起数 3 个，窗口末尾是写表第 4 个写、
    /// 录制流第 6 步——交出去的是 6，不是写表下标 4；窗口越到写表末尾、只有一个写时同样按录制流下标交。
    #[test]
    fn the_end_of_a_crash_window_is_named_by_its_stream_index_and_not_by_its_write_index() {
        let stream_indexes = [0, 1, 3, 4, 6, 9];
        assert_eq!(
            stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 2, 3),
            6,
            "写表第 2、3、4 个写，末尾那个在录制流第 6 步"
        );
        assert_eq!(
            stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 4, 2),
            9,
            "窗口越到写表末尾：写表第 5 个写在录制流第 9 步"
        );
        assert_eq!(
            stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 3, 1),
            4,
            "窗口只有写表第 3 个写：录制流第 4 步"
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
            &given_states_engine_for("crash_injection_unit_tests"),
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

    /// 每块盘最新那一槽系统配置读回全 0 时的可写挂载（C696（崩溃注入的读故障一支在随机历史里从没触发））。
    const NEWEST_SYSTEM_CONFIGURATION_SLOT_ZEROS_MOUNT: HistoryOperationKind =
        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros;

    /// 单元区 384 槽的小盘、不逐步跑 checker（设计调查量读故障那一支用的那一档）。
    fn execution_on_384_slot_devices_without_the_per_step_checker() -> HistoryExecution {
        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        }
    }

    /// C696 判别力（用户定「步表加一条靶向用例」之下的步表那一半；设计调查报告第三节：候选「丁」之下 2676 个注入段里 614 段按写表下标会被认成没注入）：
    /// 比重取 `GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`，从这个测试周期的种子基起取前 4 段「每块盘最新那一槽
    /// 系统配置读回全 0 的可写挂载」往录制流里发过东西的历史（每段 24 步），每段按种子抽 16 个崩溃状态。用例自己再跑一遍同一段历史，照录制流下标
    /// 独立数「崩溃窗口末尾那个写（录制流下标）之前那一步发过东西」的崩溃状态，崩溃注入的计数（`crash_points_after_a_step_reading_under_injected_faults`）
    /// 要与它相等；另核这批崩溃状态里按录制流下标算注入过、按写表下标算没注入的至少一个（F5 那一种接错在这批上改得了计数），注入过的至少一个。
    /// 判别力：读故障认定的崩溃窗口末尾交成写表下标（F5，`crates/mutations.tsv` 一行）时计数少了按写表下标认成没注入的那几个；认定函数恒交
    /// NoneInjected（一行）时计数是 0——两种都与独立数的不等，这一条红。
    #[test]
    fn crash_states_after_a_newest_system_configuration_slot_zeros_mount_count_as_read_faulted_by_the_stream_index_of_the_crash_window(
    ) {
        use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
        const HISTORIES: usize = 4;
        const OPERATIONS_PER_HISTORY: usize = 24;
        const SEEDS_SEARCHED: u64 = 64;
        let execution = execution_on_384_slot_devices_without_the_per_step_checker();
        let mut histories_taken = 0usize;
        let mut counted_by_the_injection = 0u64;
        let mut read_faulted_by_the_stream_index = 0u64;
        let mut read_faulted_by_the_stream_index_but_not_by_the_write_index = 0u64;
        // 迭代上界是 SEEDS_SEARCHED 个种子；跨轮携带的是四样计数；取够 HISTORIES 段就停。
        for seed_offset in 0..SEEDS_SEARCHED {
            if histories_taken == HISTORIES {
                break;
            }
            let history = generate_history_with_weights(
                HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset),
                OPERATIONS_PER_HISTORY,
                &GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS,
            );
            let stream = SharedStream::retaining_contents();
            let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
            execute_history_with(&history, execution, &stream, &mut |observation| {
                operations_after_each_step.push((observation.position, stream.operation_count()));
            });
            // 读故障下发过东西的那几步各自的第一个录制流下标。
            let mut operations_before_the_step = 0usize;
            let mut earliest_stream_index_of_each_read_faulted_step: Vec<usize> = Vec::new();
            for (position, operations_after_the_step) in &operations_after_each_step {
                if let StepPosition::Operation(step_index) = position {
                    if history.operations[*step_index].kind()
                        == NEWEST_SYSTEM_CONFIGURATION_SLOT_ZEROS_MOUNT
                        && operations_before_the_step < *operations_after_the_step
                    {
                        earliest_stream_index_of_each_read_faulted_step
                            .push(operations_before_the_step);
                    }
                }
                operations_before_the_step = *operations_after_the_step;
            }
            if earliest_stream_index_of_each_read_faulted_step.is_empty() {
                continue;
            }
            let (_writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
                &stream.retained_operations(),
                &execution.device_width.fixed_geometry(),
            );
            let read_faulted_by_the_end_of_the_window = |last_index_of_the_window: usize| {
                earliest_stream_index_of_each_read_faulted_step
                    .iter()
                    .any(|earliest_stream_index| *earliest_stream_index <= last_index_of_the_window)
            };
            let injection = inject_crashes_into_history(
                &history,
                execution,
                CrashPointDraw::Sampled {
                    crash_points_per_history: 16,
                },
                None,
                &given_states_engine_for("crash_injection_unit_tests"),
            );
            for crash_point in &injection.crash_points {
                let last_write_of_the_window = segments[crash_point.segment_index][0]
                    + crash_point.persisted_within_the_segment.len()
                    - 1;
                if read_faulted_by_the_end_of_the_window(stream_indexes[last_write_of_the_window]) {
                    read_faulted_by_the_stream_index += 1;
                    if !read_faulted_by_the_end_of_the_window(last_write_of_the_window) {
                        read_faulted_by_the_stream_index_but_not_by_the_write_index += 1;
                    }
                }
            }
            counted_by_the_injection += injection
                .tally
                .crash_points_after_a_step_reading_under_injected_faults;
            histories_taken += 1;
        }
        assert_eq!(
            histories_taken, HISTORIES,
            "前提：种子基起 {SEEDS_SEARCHED} 段里有 {HISTORIES} 段的那一步往录制流里发过东西"
        );
        assert!(
            read_faulted_by_the_stream_index >= 1,
            "前提：抽到的崩溃状态里至少一个在那一步发过东西之后"
        );
        assert!(
            read_faulted_by_the_stream_index_but_not_by_the_write_index >= 1,
            "前提：至少一个崩溃状态按录制流下标算注入过、按写表下标算没注入（F5 那一种接错在这批上改得了计数）"
        );
        assert_eq!(
            counted_by_the_injection, read_faulted_by_the_stream_index,
            "崩溃注入认定的注入过读故障的崩溃状态数，与按录制流下标独立数的相等（其中 {read_faulted_by_the_stream_index_but_not_by_the_write_index} 个按写表下标会认成没注入）"
        );
    }

    /// 起点（取号、暖机、第一个文件）之后：一次「每块盘最新那一槽系统配置读回全 0 的可写挂载」（读故障下发写，C696），再一次覆盖写。
    fn history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite(
    ) -> GeneratedHistory {
        use singlefs_harness::history::{ContentChoice, ContentLength};
        GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![
                HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
                HistoryOperation::PublishOverwrite(ContentChoice {
                    length: ContentLength::InsideOneDataUnit { selector: 0 },
                    fill_seed: 1,
                }),
            ],
        }
    }

    /// 那段历史的写表与切段（与 [`inject_crashes_into_history`] 自己切的同一份），覆盖写那次发布的第一个写在写表里的下标；
    /// 另核读回全 0 的那次可写挂载往录制流里发过东西、两步都做成。
    fn writes_segments_and_first_write_of_the_overwrite_after_the_zeros_mount(
        history: &GeneratedHistory,
        execution: HistoryExecution,
    ) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, usize) {
        let stream = SharedStream::retaining_contents();
        let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
        let run = execute_history_with(history, execution, &stream, &mut |observation| {
            operations_after_each_step.push((observation.position, stream.operation_count()));
        });
        assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
        let [(StepPosition::StartingPoint, after_the_starting_point), (StepPosition::Operation(0), after_the_zeros_mount), (StepPosition::Operation(1), _)] =
            operations_after_each_step[..]
        else {
            panic!("前提：起点与两步各登记一次：{operations_after_each_step:?}");
        };
        assert!(
            after_the_starting_point < after_the_zeros_mount,
            "前提：读回全 0 的那次可写挂载往录制流里发过东西（取号、写行、暖机）"
        );
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        let first_write_of_the_overwrite = stream_indexes
            .iter()
            .position(|stream_index| *stream_index >= after_the_zeros_mount)
            .expect("覆盖写那次发布写过东西");
        (writes, segments, first_write_of_the_overwrite)
    }

    /// C696 与 D13（验证路线） 已定项 1 挂载透明的射程（只管不带故障的状态）：第一次崩溃摆在覆盖写那次发布的第一个写上（它一个写都没落，
    /// 盘面停在读回全 0 的那次可写挂载做成之后），这个态注入过读故障，崩溃镜像上那一次可写挂载做成，挂载透明整个不判，它途中按段摆的
    /// 三个二次崩溃也不判（第 28 批 G5），同一理由共记四次没判。
    /// 判别力（`crates/mutations.tsv` 各一行）：第一次崩溃镜像上照判挂载透明（「第26批 C696 射程」）时没判的那一条理由只剩二次崩溃那 3 次；
    /// 二次崩溃那一截不看标记、照判（「第28批 G5」）时它是 1；二次崩溃那一截跳过却不记没判（「第33批 G5」）时它也是 1。三种这一条都红。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态（ExactlyOneCrashState），不枚举
    #[test]
    fn mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount(
    ) {
        let history = history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite();
        let execution = execution_on_384_slot_devices_without_the_per_step_checker();
        let (_writes, segments, first_write_of_the_overwrite) =
            writes_segments_and_first_write_of_the_overwrite_after_the_zeros_mount(
                &history, execution,
            );
        let overwrite_segment_index = segments
            .iter()
            .position(|segment| segment[0] == first_write_of_the_overwrite)
            .expect("覆盖写那次发布从一段的第一个写起（它前面是挂载最后那道屏障）");
        let injection = inject_crashes_into_history(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index: overwrite_segment_index,
                writes_from_the_segment_on: 1,
                withheld_among_them: &[0],
            },
            None,
            &given_states_engine_for("crash_injection_unit_tests"),
        );
        let tally = &injection.tally;
        assert_eq!(
            (
                tally.crash_points,
                tally.crash_points_after_a_step_reading_under_injected_faults,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.second_crash_points
            ),
            (1, 1, 1, 3),
            "前提：只摆写死的那一个第一次崩溃态，它注入过读故障，它上面的可写挂载做成，挂载途中按段摆了三个二次崩溃：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.mount_transparency_judged.clone(),
                tally
                    .mount_transparency_judged_on_second_crashes_by_phase
                    .clone(),
                tally
                    .mount_transparency_not_judged
                    .get(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
                    .copied()
            ),
            (BTreeMap::new(), BTreeMap::new(), Some(1 + 3)),
            "注入过读故障的崩溃态上挂载透明一个动作都不判：崩溃镜像上那次挂载记一次，它途中的三个二次崩溃各记一次：{}",
            tally.render()
        );
    }

    /// 「每块盘最新那一槽系统配置读回全 0 的可写挂载」之后那次覆盖写的第一个写上摆第一次崩溃（注入过读故障的态），第二截那次可写挂载的
    /// 写行那次发布上摆一个二次崩溃——根槽 FUA 没落、紧跟着的两槽系统配置轮换落了；交回这一趟崩溃注入。两条靶向用例共用这份搭建，各自断言。
    fn injection_with_a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount(
    ) -> HistoryCrashInjection {
        let history = history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite();
        let execution = execution_on_384_slot_devices_without_the_per_step_checker();
        let (writes, segments, first_write_of_the_overwrite) =
            writes_segments_and_first_write_of_the_overwrite_after_the_zeros_mount(
                &history, execution,
            );
        let overwrite_segment_index = segments
            .iter()
            .position(|segment| segment[0] == first_write_of_the_overwrite)
            .expect("覆盖写那次发布从一段的第一个写起（它前面是挂载最后那道屏障）");
        let mut first_crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        first_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &first_crash_image,
            &execution.device_width.parameters(),
            execution,
            history.seed,
            &CrashPoint {
                segment_index: overwrite_segment_index,
                persisted_within_the_segment: vec![false],
                step: StepPosition::Operation(1),
                operation_kind: Some(HistoryOperationKind::PublishOverwrite),
            },
        );
        assert!(
            matches!(mount_run.outcome, WritableMountOutcome::Published),
            "前提：读回全 0 的那次挂载之后那一版上的可写挂载与之后那一次覆盖写都做成：{:?}",
            mount_run.outcome
        );
        let (mount_writes, mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(
                &mount_run.mount_operations,
                &execution.device_width.fixed_geometry(),
            );
        let phase_of_each_write = writable_mount_phase_of_each_write(
            &mount_writes,
            mount_run.publishes_before_the_deletion_queue_continuation,
            mount_run.publishes_before_the_instance_table_sweep,
        );
        let root_of_the_row_publish = (0..mount_writes.len())
            .find(|write| {
                phase_of_each_write[*write] == WritableMountPhase::RowPublish
                    && mount_writes[*write].kind == StepKind::RootRecordFua
            })
            .expect("前提：写行那次发布写了根槽");
        let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
            .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
            .collect();
        assert_eq!(
            rotation_after_the_root.len(),
            2,
            "前提：写行那次发布的根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
        );
        let segment_of_the_root =
            segment_containing_the_write(&mount_segments, root_of_the_row_publish);
        let first_write_of_the_segment_of_the_root = mount_segments[segment_of_the_root][0];
        let last_rotation = rotation_after_the_root[rotation_after_the_root.len() - 1];
        inject_crashes_into_history_drawing_second_crashes(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index: overwrite_segment_index,
                writes_from_the_segment_on: 1,
                withheld_among_them: &[0],
            },
            SecondCrashPointDraw::ExactlyOneSecondCrashState {
                segment_index: segment_of_the_root,
                writes_from_the_segment_on: last_rotation + 1
                    - first_write_of_the_segment_of_the_root,
                withheld_among_them: vec![
                    root_of_the_row_publish - first_write_of_the_segment_of_the_root,
                ]
                .leak(),
            },
            None,
            &given_states_engine_for("crash_injection_unit_tests"),
        )
    }

    /// C696「第三截调用点写死成 NoneInjected」那一种接错的靶向用例（用户定「步表加一条靶向用例」；W 为真的阳性对照，造法同
    /// [`a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site`]）：第一次崩溃摆在
    /// 「每块盘最新那一槽系统配置读回全 0 的可写挂载」之后那次覆盖写的第一个写上（注入过读故障的态），第二截那次可写挂载的写行那次发布上摆一个
    /// 二次崩溃——根槽 FUA 没落、紧跟着的两槽系统配置轮换落了。二次崩溃镜像上再挂以判据 N-配置 拒、判据复算为真；崩溃态注入过读故障，
    /// 第三截判过一次、不判 C670 那一条（不合法 0 次、没有新发现）。
    /// 判别力：第三截调用点把读故障的认定写死成 NoneInjected（`crates/mutations.tsv` 一行）时不合法变 1、报一条新发现，这一条红。
    /// 随机历史上那一种接错是等价变异（设计调查报告第三节：注入态与第三截二次崩溃态上判据从没为真），等价性留档在
    /// `crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs`。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
    #[test]
    fn a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_by_c670(
    ) {
        let injection =
            injection_with_a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount();
        let tally = &injection.tally;
        assert_eq!(
            (
                tally.crash_points,
                tally.crash_points_after_a_step_reading_under_injected_faults,
                tally.writable_mounts_after_the_crash_succeeded
            ),
            (1, 1, 1),
            "前提：只摆写死的那一个第一次崩溃态，它注入过读故障，它上面的可写挂载做成：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.second_crash_points,
                tally
                    .second_crash_points_by_phase
                    .get(&WritableMountPhase::RowPublish)
                    .copied()
            ),
            (1, Some(1)),
            "前提：只摆写死的那一个二次崩溃，崩在写行里：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
            ),
            (1, 0),
            "二次崩溃镜像上再挂以判据 N-配置 拒、判过一次；第一次崩溃那个态注入过读故障，不判 C670 那一条：{:?}",
            injection
                .new_findings
                .iter()
                .map(CrashPointFinding::render)
                .collect::<Vec<_>>()
        );
    }

    /// 改法 G5（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「二次崩溃镜像在注入态下照判（J2-d）」那一行，
    /// D13（验证路线） 已定项 1「带故障的状态不在射程」）：同上那一份搭建——第一次崩溃那个态注入过读故障，它上面那次可写挂载做成，写行里摆一个
    /// 二次崩溃。第二截那一次挂载透明不判；那个二次崩溃镜像上的挂载透明也不判，与第二截同一个理由再记一次（共 2 次），按段判过的一次都没有。
    /// 判别力：二次崩溃那一截不看读故障标记、照判挂载透明（`crates/mutations.tsv` 一行）时按段判过的有写行 1 次、没判的只记 1 次，这一条红。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
    #[test]
    fn mount_transparency_is_not_judged_on_a_second_crash_inside_the_mount_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount(
    ) {
        let injection =
            injection_with_a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount();
        let tally = &injection.tally;
        assert_eq!(
            (
                tally.crash_points_after_a_step_reading_under_injected_faults,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.second_crash_points
            ),
            (1, 1, 1),
            "前提：只摆写死的那一个第一次崩溃态（注入过读故障，它上面的可写挂载做成）与那一个二次崩溃：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally
                    .mount_transparency_judged_on_second_crashes_by_phase
                    .clone(),
                tally
                    .mount_transparency_not_judged
                    .get(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
                    .copied()
            ),
            (BTreeMap::new(), Some(2)),
            "注入过读故障的崩溃态上，那次挂载与它途中的二次崩溃都不判挂载透明，按理由各记一次：{}",
            tally.render()
        );
    }

    /// 起点（取号、暖机、第一个文件）之后一次覆盖写、不重挂的历史，与跑它用的执行（单元区 384 槽的小盘，不逐步跑 checker）。
    fn history_of_the_first_file_and_one_overwrite() -> (GeneratedHistory, HistoryExecution) {
        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
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

    /// 那段历史跑完之后的整条写表、切段、录制流下标与 mkfs 写完时的录制流下标（给定状态表照它们建，[`HistoryAfterTheMakeFilesystem::of`]）。
    fn whole_write_table_of_the_history(
        history: &GeneratedHistory,
        execution: HistoryExecution,
    ) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, Vec<usize>, usize) {
        let stream = SharedStream::retaining_contents();
        let run = execute_history_with(history, execution, &stream, &mut |_observation| {});
        assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        (
            writes,
            segments,
            stream_indexes,
            run.operations_written_by_make_filesystem,
        )
    }

    /// 盖掉创世树表单元两份副本的两个单元写（与 mkfs 写的那两份同盘、同偏移、同长，内容换成别的字节），交回时连同 mkfs 那份副本在写表里的下标。
    /// 树表那一份是 mkfs 单元写里偏移最大的那一组（实例表在单元区起点、树表紧跟它，`make_filesystem` 的 `tree_table_genesis_placement_at`）。
    fn writes_overwriting_the_genesis_tree_table(
        history_after_the_make_filesystem: &HistoryAfterTheMakeFilesystem<'_>,
    ) -> (Vec<RetainedWrite>, usize) {
        let genesis_units: Vec<(usize, &RetainedWrite)> = history_after_the_make_filesystem
            .writes_of_the_make_filesystem()
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::UnitWrite)
            .collect();
        let tree_table_offset = genesis_units
            .iter()
            .map(|(_, write)| write.offset)
            .max()
            .expect("mkfs 写了实例表与树表两个单元");
        let tree_table_copies: Vec<(usize, &RetainedWrite)> = genesis_units
            .into_iter()
            .filter(|(_, write)| write.offset == tree_table_offset)
            .collect();
        assert_eq!(
            tree_table_copies.len(),
            2,
            "前提：创世树表单元两块盘各一份副本"
        );
        let overwrites = tree_table_copies
            .iter()
            .map(|(_, copy)| RetainedWrite {
                device: copy.device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset: copy.offset,
                contents: singlefs_harness::memory_pool::WrittenContents::Bytes(vec![
                    0xA5;
                    usize::try_from(copy.length_in_bytes())
                        .expect("单元长装得进 usize")
                ]),
            })
            .collect();
        (overwrites, tree_table_copies[0].0)
    }

    /// 创世发布进记录核对器（用户 2026-10-10 弹窗定「先查 GPU 能不能核，不能就 CPU 核一遍」，查下来 GPU 核得到、走 GPU 的发布表）：
    /// 那段历史跑完、每个写都落了，末尾再种两个盖掉创世树表单元两份副本的单元写（根环里还有创世那三条根，回收谓词证得出这次复用不合法），
    /// 给定状态表交崩溃点重放的引擎判（两段：表先写进判定库、读回再判，配了 GPU 走 GPU，用户 2026-10-13 定「能走两段的都走两段」），
    /// 记录核对器判「恢复自称的状态缺了一次发布的单元」；不种那两个写时同一张表不红。
    /// 判别力：给定状态表的写表把 mkfs 的写整截留在表外（基线照旧是 mkfs 之后），创世发布不进发布表，种了也不红（`crates/mutations.tsv` 那一行）。
    // crash-case-check:not-a-crash-case 只判写死的两个给定状态（种与不种），不枚举崩溃状态
    #[test]
    fn a_genesis_unit_overwritten_by_a_reuse_the_reclaim_predicate_refuses_is_red_on_the_record_checker_of_the_given_states_judge(
    ) {
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, segments, stream_indexes, after_make_filesystem) =
            whole_write_table_of_the_history(&history, execution);
        let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
            &writes,
            &segments,
            &stream_indexes,
            after_make_filesystem,
            execution,
        );
        let (overwrites, genesis_copy) =
            writes_overwriting_the_genesis_tree_table(&history_after_the_make_filesystem);
        let healthy_writes = history_after_the_make_filesystem.writes();
        let planted_writes: Vec<RetainedWrite> =
            healthy_writes.iter().chain(&overwrites).cloned().collect();
        assert!(
            !crate::crash::reuse_is_not_proven_illegal_by_the_reclaim_predicate(
                &planted_writes,
                crate::crash::checkpoint_txg_of_the_publish_that_made_the_write(
                    &planted_writes,
                    genesis_copy
                ),
                planted_writes.len() - 1,
            ),
            "前提：创世那几条根还在根环里，回收谓词证得出盖掉创世树表单元不合法"
        );
        let engine = given_states_engine_for("crash_injection_unit_tests");
        let record_check_with_every_write_persisted = |table_writes: &[RetainedWrite]| {
            let mut segments_of_the_table = history_after_the_make_filesystem.segments().to_vec();
            if table_writes.len() > healthy_writes.len() {
                segments_of_the_table.push((healthy_writes.len()..table_writes.len()).collect());
            }
            let conclusions = engine
                .judge_the_given_states(&GivenStatesOfATable {
                    table_name: format!(
                        "first-file-and-one-overwrite/every-write-persisted/writes={}",
                        table_writes.len()
                    ),
                    base: history_after_the_make_filesystem.base_pool(),
                    writes: table_writes,
                    segments: &segments_of_the_table,
                    seams: &[],
                    persisted_sets: vec![vec![true; table_writes.len()]],
                })
                .unwrap_or_else(|reason| panic!("引擎判得了这张表：{reason}"));
            assert_eq!(conclusions.states.len(), 1, "这张表一个状态");
            (conclusions.source, conclusions.states[0].record_check())
        };
        let (healthy_source, healthy_record_check) =
            record_check_with_every_write_persisted(healthy_writes);
        if healthy_source == GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly {
            // 配置开了 GPU 而这份二进制没编进 gpu 特性：判段不跑、交占位结论，判归带特性那一趟（引擎开跑时已打本次未跑那一行）
            return;
        }
        assert_eq!(
            healthy_record_check,
            RecordCheck::default(),
            "不种：跑完的历史每个写都落了，记录核对器不红"
        );
        assert!(
            record_check_with_every_write_persisted(&planted_writes).1.claimed_state_missing_unit,
            "种了：创世树表单元两份副本都被不合法地盖掉，记录核对器判恢复自称的状态缺了创世发布的单元"
        );
    }

    /// GPU 判器的发布表照 CPU 记录核对器同一个分法分（给定状态表）：创世那几次发布在表里、带 mkfs 的单元写；枚举写表接在录制流后面的撕裂根槽写
    /// （mkfs 的根槽写叠在 mkfs 之后的基线上可撕，撕出来与原字节相同）不分出另一次发布。不要卡：只建表、不判。
    /// 判别力：发布表照整张枚举写表分（`PublishesSplitFrom::TheEnumeratedWrites`），多出几次根在录制流之后的发布；写表把 mkfs 的写留在表外，
    /// 创世发布不在表里（`crates/mutations.tsv` 两行）。
    #[test]
    fn the_gpu_publishes_of_a_given_states_table_are_split_as_the_cpu_record_checker_splits_them_genesis_included(
    ) {
        use crate::crash_judge_tables::{
            build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams, JudgeFlow,
        };
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, segments, stream_indexes, after_make_filesystem) =
            whole_write_table_of_the_history(&history, execution);
        let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
            &writes,
            &segments,
            &stream_indexes,
            after_make_filesystem,
            execution,
        );
        let table_writes = history_after_the_make_filesystem.writes();
        let persisted = vec![true; table_writes.len()];
        let not_expanded = |_segment_index: usize, _segment: &[usize]| {
            crate::crash::CrashPointReplaySegmentExpansion::NotExpanded
        };
        let enumerated_write_count = crate::crash::crash_state_enumeration_tables(
            history_after_the_make_filesystem.base_pool(),
            table_writes,
            history_after_the_make_filesystem.segments(),
            &not_expanded,
        )
        .enumerated_writes
        .len();
        assert!(
            enumerated_write_count > table_writes.len(),
            "前提：枚举写表接了撕裂镜像（mkfs 的根槽写与系统配置槽写叠在基线上可撕）"
        );
        let mut mask = persisted.clone();
        mask.resize(enumerated_write_count, false);
        let tables = build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams(
            &JudgeFlow {
                base: history_after_the_make_filesystem.base_pool(),
                writes: table_writes,
                segments: history_after_the_make_filesystem.segments(),
                judged_root_index: 0,
                versions: &[],
                expansion: &not_expanded,
            },
            &[],
            vec![mask],
            &[],
        )
        .expect("建得出这张表");
        let sorted = |mut units: Vec<usize>| {
            units.sort_unstable();
            units
        };
        let gpu_publishes: Vec<(usize, Vec<usize>)> = tables
            .publishes
            .iter()
            .map(|publish| {
                (
                    usize::try_from(publish.root_write).expect("下标装得进 usize"),
                    sorted(
                        publish
                            .unit_copy_groups
                            .iter()
                            .flatten()
                            .map(|copy| usize::try_from(*copy).expect("下标装得进 usize"))
                            .collect(),
                    ),
                )
            })
            .collect();
        let cpu_publishes: Vec<(usize, Vec<usize>)> = singlefs_harness::memory_pool::publishes_in(
            table_writes,
            &persisted,
            RecordStreamContinuity::OneRecording,
        )
        .into_iter()
        .map(|publish| (publish.root, sorted(publish.units)))
        .collect();
        assert_eq!(
            gpu_publishes, cpu_publishes,
            "GPU 的发布表与 CPU 记录核对器分出的发布逐次相同（根槽写、单元写）"
        );
        let genesis_writes = history_after_the_make_filesystem
            .writes_of_the_make_filesystem()
            .len();
        let genesis_unit_writes: Vec<usize> = (0..genesis_writes)
            .filter(|write| table_writes[*write].kind == StepKind::UnitWrite)
            .collect();
        assert!(
            !genesis_unit_writes.is_empty()
                && gpu_publishes
                    .iter()
                    .any(|(root, units)| *root < genesis_writes && *units == genesis_unit_writes),
            "创世那次发布在 GPU 的发布表里、带 mkfs 的单元写 {genesis_unit_writes:?}：{gpu_publishes:?}"
        );
    }

    /// C670 的接线（攻方改法 P2，`research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K2-e）：经崩溃注入的入口摆一个靶向崩溃态——
    /// 第一个文件之后一次覆盖写，那次发布只落了系统配置轮换，单元、记录与根都没落（盘面照 harness 档
    /// `writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 的造法；越过了根槽 FUA 与轮换的先后，今天的持久顺序下段内真子集摆不出来）。
    /// 历史里没有带读故障的步；第二截的可写挂载以判据 N-配置 拒，盘没变、只读读出在允许集合里，U6 四条都满足，C670 那一条判不合法，计 1。
    /// 判别力：调用点把读故障的认定恒交 `Injected`、第二截换回不判 C670 的旧判官 `judge_writable_mount_refusal`（`crates/mutations.tsv` 两行），
    /// 这一拒都判成合法，这一条红；调用点把所选那一版交成 None 时计数照样 (0, 1)，不合法改由「判据复算为假」给出，红在条件那两条断言上
    /// （攻方改法 F1）。harness 档不许依赖 checker 档，接线在 checker 档，用例只能写在这里。
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
            &given_states_engine_for("crash_injection_unit_tests"),
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
        // 不合法是哪一条给的（攻方改法 F1，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：调用点把所选那一版交成 None 时，
        // C670 那一条不成立、U6「判据复算为假」那一条顶上来，计数照样是 (0, 1)；只有看判官列出的没满足的条件才分得开。
        let conditions_not_met = conditions_not_met_named_by_the_one_illegal_writable_mount_refusal(
            &injection.new_findings,
        );
        let witness_criterion_true_without_read_faults = format!(
            "{:?}",
            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(
                WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion
            )
        )
        .split('(')
        .next()
        .expect("split 至少交回一段")
        .to_string();
        let criterion_false_when_recomputed = format!(
            "{:?}",
            RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
        );
        assert!(
            conditions_not_met.contains(&witness_criterion_true_without_read_faults),
            "拒得不合法要由 C670 那一条（{witness_criterion_true_without_read_faults}）给出：判官列出的没满足的条件 {conditions_not_met}"
        );
        assert!(
            !conditions_not_met.contains(&criterion_false_when_recomputed),
            "判据在崩溃镜像上复算为真（U6 那一条满足），不许列出 {criterion_false_when_recomputed}：判官列出的没满足的条件 {conditions_not_met}"
        );
    }

    /// 第二截那一条拒得不合法的发现里，判官列出的没满足的条件（[`judge_the_writable_mount_on_the_crash_image`] 写进实现那一半的
    /// 「拒得不合法（…）：」那一段，`Vec<RefusalConditionNotMet>` 的 Debug 文本）。
    ///
    /// # Panics
    /// 第二截的新发现不是恰好一条、它没有模型对不上那一半、或那一半里没有「拒得不合法（…）：」那一段。
    fn conditions_not_met_named_by_the_one_illegal_writable_mount_refusal(
        new_findings: &[CrashPointFinding],
    ) -> String {
        let findings_after_the_writable_mount: Vec<&CrashPointFinding> = new_findings
            .iter()
            .filter(|finding| finding.stage == CrashStateStage::AfterTheWritableMountAndOnePublish)
            .collect();
        let [finding] = findings_after_the_writable_mount.as_slice() else {
            panic!(
                "第二截要恰好一条新发现（那一次拒得不合法）：{:?}",
                findings_after_the_writable_mount
                    .iter()
                    .map(|finding| finding.render())
                    .collect::<Vec<_>>()
            );
        };
        let implementation_answer = &finding
            .observation
            .model_disagreement
            .as_ref()
            .unwrap_or_else(|| panic!("拒得不合法记成模型对不上：{}", finding.render()))
            .implementation_answer;
        implementation_answer
            .split_once("拒得不合法（")
            .and_then(|(_, after)| after.split_once("）："))
            .map(|(conditions, _)| conditions.to_string())
            .unwrap_or_else(|| {
                panic!("实现那一半里没有「拒得不合法（…）：」那一段：{implementation_answer}")
            })
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
            InstanceTableSweepConfiguration::PRODUCT,
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

    /// Q3 在第三截的计数（m3-close-g4-rewrite-r1 判决第二节 V4 那一行：Q3 第三截的计数没有测试断言、变异表也没有行点名）：一份不合法的 Q3 拒
    /// （崩溃态没注入读故障、判据复算为真）与一份合法的 Q3 拒各交一次给第三截那一判，再把这一份计数并进另一份已记过一次判过、一次不合法的计数。
    /// 预期：这一份判过 1 + 1 次、不合法 1 次，不合法那一份交回拒因与那一条条件、合法那一份交回 None；并起来判过 1 + 2、不合法 1 + 1，
    /// 渲染里 Q3 那一行带这两个数。判别力：`crates/mutations.tsv`「根环一簇 R12」那一行（第三截不记 Q3 那一条不合法），这一条红。
    #[test]
    fn the_third_cut_counts_each_q3_refusal_and_each_one_whose_criterion_is_true_without_read_faults(
    ) {
        let mut tally = CrashInjectionTally::default();
        let criterion_true_without_read_faults = vec![
            RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
        ];
        assert_eq!(
            reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
                &mut tally,
                WritableMountRefusalVerdict::Illegal {
                    reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
                    conditions_not_met: criterion_true_without_read_faults.clone(),
                },
            ),
            Some((
                RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
                criterion_true_without_read_faults
            )),
            "不合法的 Q3 拒：交回拒因与那一条条件（报红用）"
        );
        assert_eq!(
            reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
                &mut tally,
                WritableMountRefusalVerdict::Legal(
                    RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator
                ),
            ),
            None,
            "合法的 Q3 拒：只记判过、不报红"
        );
        assert_eq!(
            (
                tally.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
                tally.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults
            ),
            (1 + 1, 1),
            "这一份：判过两次（不合法、合法各一），不合法一次：{}",
            tally.render()
        );
        let mut earlier = CrashInjectionTally::default();
        assert!(
            reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
                &mut earlier,
                WritableMountRefusalVerdict::Illegal {
                    reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
                    conditions_not_met: vec![
                        RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
                    ],
                },
            )
            .is_some(),
            "另一份先记过一次不合法的 Q3 拒"
        );
        earlier.absorb(&tally);
        assert_eq!(
            (
                earlier.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
                earlier.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults
            ),
            (1 + 2, 1 + 1),
            "并起来：判过、不合法各按两份相加：{}",
            earlier.render()
        );
        assert!(
            earlier.render().contains(
                "（第三截判 Q3 那一条）：3 次，崩溃态没注入读故障而判据复算为真、拒得不合法 2"
            ),
            "渲染里 Q3 那一行带判过与不合法两个数：{}",
            earlier.render()
        );
    }

    /// 改法 G4 在第三截的判法（与 C670 那一条同一个接法）：第一个文件之后的盘面上，盘 0 两个系统配置槽各改一个字节（读得出、自证不过，设备不报错），
    /// 另把盘 0 上所选那一版的一个单元（两盘同槽的那一个，盘 1 那一份不动）改一个字节，当成一个二次崩溃镜像交给第三截，没崩过的那一次挂载做成过
    /// （`once` 有内容）。盘 0 两槽全废，可写挂载照丙' 过补写闸；那个单元在盘 0 上读出的字节与验得过的那一份不同，补写闸闸一
    /// （`RewriteGateNotMet::UnitsOfTheSelectedVersionNotAllOnTheDevice`，`singlefs_core::mount` 的补写闸）不过，不补写、照旧拒。
    /// 只改两槽、单元都在的盘面过得了补写闸、挂上了，拒不出来（三方判决 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md` 第三节，测试走甲）。
    /// 再挂以「读阶段判完之后系统配置又读不出」拒；崩溃态没注入读故障、那一拒的判据在这份盘面上复算为真（盘 0 一份自证过的系统配置都没有）：
    /// I-8.4 照旧记「再挂被拒」不判，G4 那一条判过一次、不合法一次，记一条第三截的发现；判据 N-配置 那两样计数不动。
    /// 判别力：第三截不认 G4 那一条（`crates/mutations.tsv` 一行），这一条红。
    #[test]
    fn a_remount_refused_on_a_second_crash_image_where_device_zero_has_both_system_configuration_slots_torn_is_an_illegal_refusal_after_the_read_stage(
    ) {
        /// 撕裂一个系统配置槽时改的那个字节在槽里的位置：槽中间，落在整槽校验和罩着的范围里。
        const TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT: usize = 2048;
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, _segments, first_write_of_the_overwrite) =
            writes_segments_and_first_write_of_the_overwrite(&history, execution);
        let mut second_crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        second_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
        let slot_spacing = u64::from(
            execution
                .device_width
                .parameters()
                .geometry
                .fixed_structure_slot_spacing,
        );
        let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES)
            .expect("系统配置槽 4096 字节装得进 usize");
        // 迭代上界是一块盘的两个系统配置槽；不跨轮携带状态；没有提前出口。
        for slot_index in 0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = DeviceOffsetInBytes(slot_index * slot_spacing);
            let mut slot =
                PoolReader::read(&second_crash_image, DeviceIdentity(0), offset, slot_bytes)
                    .expect("内存盘上系统配置槽读得出");
            slot[TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT] ^= 0xFF;
            second_crash_image
                .devices
                .get_mut(&DeviceIdentity(0))
                .expect("池里有盘 0")
                .write(offset, &slot);
        }
        // 补写闸闸一不过：盘 0 上所选那一版的一个单元改一个字节。挑第一个文件那次发布在盘 0 上写的最后一个单元、而盘 1 同一落点也写过
        // （两盘同槽，盘 1 那一份验得过，择到的那一版照旧是它）
        let unit_write_on_both_devices = writes[..first_write_of_the_overwrite]
            .iter()
            .rev()
            .find(|write| {
                write.kind == StepKind::UnitWrite
                    && write.device == DeviceIdentity(0)
                    && writes[..first_write_of_the_overwrite].iter().any(|mirror| {
                        mirror.kind == StepKind::UnitWrite
                            && mirror.device == DeviceIdentity(1)
                            && mirror.offset == write.offset
                    })
            })
            .expect("前提：第一个文件那次发布在两块盘的同一落点上各写了一份单元");
        let unit_length = usize::try_from(unit_write_on_both_devices.length_in_bytes())
            .expect("一个单元装得进 usize");
        let mut unit_on_device_zero = PoolReader::read(
            &second_crash_image,
            DeviceIdentity(0),
            unit_write_on_both_devices.offset,
            unit_length,
        )
        .expect("内存盘上单元读得出");
        unit_on_device_zero[unit_length / 2] ^= 0xFF;
        second_crash_image
            .devices
            .get_mut(&DeviceIdentity(0))
            .expect("池里有盘 0")
            .write(unit_write_on_both_devices.offset, &unit_on_device_zero);
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
            InstanceTableSweepConfiguration::PRODUCT,
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
            (0, 0),
            "前提：拒因不是判据 N-配置：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
                tally.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults
            ),
            (1, 1),
            "拒因是读阶段判完之后系统配置又读不出、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
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
            &given_states_engine_for("crash_injection_unit_tests"),
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

    /// C670 在第三截经调用点的判法（攻方改法 F2，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：第一个文件之后一次覆盖写的历史，
    /// 第一次崩溃摆在覆盖写那次发布的第一个写上（它一个写都没落，盘面就是起点那一版）；第二截那次可写挂载的写行那次发布上摆一个二次崩溃——
    /// 根槽 FUA 没落、紧跟着的两槽系统配置轮换落了（越过根槽那一段的段尾：今天的持久顺序下根槽 FUA 先于轮换、各成一段，段内真子集摆不出来，
    /// 与 [`a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash`] 越过段尾同一个理由）。
    /// 新实例的记录不跨实例边界施加，二次崩溃镜像上只读恢复落在起点那一版，见证值比它新：再挂以判据 N-配置 拒；崩溃态没注入读故障、判据复算为真，
    /// 第三截判过一次、不合法一次。判别力：调用点（[`judge_second_crashes_inside_the_writable_mount`] 拼 [`RefusalOnTheCrashImage`] 那几行）
    /// 把读故障恒交 `Injected`、再挂的盘换成第一次崩溃的镜像、换成没叠这一段的基线（`crates/mutations.tsv` 三行），不合法都变 0。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
    #[test]
    fn a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site(
    ) {
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, segments, first_write_of_the_overwrite) =
            writes_segments_and_first_write_of_the_overwrite(&history, execution);
        let overwrite_segment_index = segments
            .iter()
            .position(|segment| segment[0] == first_write_of_the_overwrite)
            .expect("覆盖写那次发布从一段的第一个写起（它前面是起点末尾那道屏障）");
        // 第二截那次挂载写出的写表：与注入里那一次同一份盘面（覆盖写一个写都没落）、同一份参数；挂载之后那一次发布写的内容随崩溃点变，不在挂载写表里。
        let mut first_crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        first_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &first_crash_image,
            &execution.device_width.parameters(),
            execution,
            history.seed,
            &CrashPoint {
                segment_index: overwrite_segment_index,
                persisted_within_the_segment: vec![false],
                step: StepPosition::Operation(0),
                operation_kind: Some(HistoryOperationKind::PublishOverwrite),
            },
        );
        assert!(
            matches!(mount_run.outcome, WritableMountOutcome::Published),
            "前提：起点那一版上的可写挂载与之后那一次覆盖写都做成：{:?}",
            mount_run.outcome
        );
        let (mount_writes, mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(
                &mount_run.mount_operations,
                &execution.device_width.fixed_geometry(),
            );
        let phase_of_each_write = writable_mount_phase_of_each_write(
            &mount_writes,
            mount_run.publishes_before_the_deletion_queue_continuation,
            mount_run.publishes_before_the_instance_table_sweep,
        );
        let root_of_the_row_publish = (0..mount_writes.len())
            .find(|write| {
                phase_of_each_write[*write] == WritableMountPhase::RowPublish
                    && mount_writes[*write].kind == StepKind::RootRecordFua
            })
            .expect("前提：写行那次发布写了根槽");
        let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
            .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
            .collect();
        assert_eq!(
            rotation_after_the_root.len(),
            2,
            "前提：写行那次发布的根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
        );
        let segment_of_the_root =
            segment_containing_the_write(&mount_segments, root_of_the_row_publish);
        let first_write_of_the_segment_of_the_root = mount_segments[segment_of_the_root][0];
        let last_rotation = rotation_after_the_root[rotation_after_the_root.len() - 1];
        let injection = inject_crashes_into_history_drawing_second_crashes(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index: overwrite_segment_index,
                writes_from_the_segment_on: 1,
                withheld_among_them: &[0],
            },
            SecondCrashPointDraw::ExactlyOneSecondCrashState {
                segment_index: segment_of_the_root,
                writes_from_the_segment_on: last_rotation + 1
                    - first_write_of_the_segment_of_the_root,
                withheld_among_them: vec![
                    root_of_the_row_publish - first_write_of_the_segment_of_the_root,
                ]
                .leak(),
            },
            None,
            &given_states_engine_for("crash_injection_unit_tests"),
        );
        let tally = &injection.tally;
        assert_eq!(
            (tally.crash_points, tally.writable_mounts_after_the_crash_succeeded),
            (1, 1),
            "前提：只摆写死的那一个第一次崩溃态，它上面的可写挂载做成（没崩过的那一次读得出内容）：{}",
            tally.render()
        );
        assert_eq!(
            tally.crash_points_after_a_step_reading_under_injected_faults,
            0,
            "这段历史没有带读故障的步：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.second_crash_points,
                tally
                    .second_crash_points_by_phase
                    .get(&WritableMountPhase::RowPublish)
                    .copied()
            ),
            (1, Some(1)),
            "前提：只摆写死的那一个二次崩溃，崩在写行里：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
            ),
            (1, 1),
            "二次崩溃镜像上再挂以判据 N-配置 拒、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
            tally.render()
        );
    }

    /// 第三截调用点交给判官的「所选那一版」是二次崩溃镜像上只读恢复落到的那一版（判决 `research/prompts/m3-formal-code-b20-r1-main-verification.md`
    /// 第二节 K1「第四种接错」）：第一次崩溃与上一条同一个（覆盖写一个写都没落，只读恢复落在起点那一版）；第二截那次可写挂载的写行那次发布上
    /// 摆一个二次崩溃——写行那次的记录（每块盘一份）没落，根槽 FUA 与紧跟着的两槽系统配置轮换落了（越过记录与根槽之间那道屏障，与上一条同一个理由）。
    /// 二次崩溃镜像上只读恢复落在写行那一版（新实例的根），与第一次崩溃不同；它那次发布的末条记录读不出，计数器等于 c_见证 的记录也读不出：
    /// 再挂以判据 N-配置 拒，C670 那一条成立在「两条都读不出」那一支。判别力：调用点把所选那一版交成第一次崩溃那一版（`crates/mutations.tsv` 一行）时
    /// 计数照样 (1, 1)，判官列出的那一支换成「c_见证 大于所选那一版末条记录的计数器」，红在那一支的两条断言上。计数分不开：在 harness 档草稿探针里
    /// 量过，三段短历史 131 个第一次崩溃态上挂载写表每个前缀与每个前缀扣一个写（177198 个二次崩溃态），以及这一个第一次崩溃态上每个前缀扣至多两个写，
    /// 两种所选那一版的 (判过, 不合法) 没有一处不同；判官结论不同的只有扣下写行（或第一次暖机）那次记录、根槽落了的这一类，都只差这一支。
    /// 那几处没量到的越屏障靶向态里计数分得开（攻方量过，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节 K3），
    /// 钉在 [`a_second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal`]。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
    #[test]
    fn a_second_crash_landing_on_the_row_publish_whose_records_were_lost_names_the_selected_version_of_the_second_crash_image(
    ) {
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, segments, first_write_of_the_overwrite) =
            writes_segments_and_first_write_of_the_overwrite(&history, execution);
        let overwrite_segment_index = segments
            .iter()
            .position(|segment| segment[0] == first_write_of_the_overwrite)
            .expect("覆盖写那次发布从一段的第一个写起（它前面是起点末尾那道屏障）");
        let mut first_crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        first_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &first_crash_image,
            &execution.device_width.parameters(),
            execution,
            history.seed,
            &CrashPoint {
                segment_index: overwrite_segment_index,
                persisted_within_the_segment: vec![false],
                step: StepPosition::Operation(0),
                operation_kind: Some(HistoryOperationKind::PublishOverwrite),
            },
        );
        assert!(
            matches!(mount_run.outcome, WritableMountOutcome::Published),
            "前提：起点那一版上的可写挂载与之后那一次覆盖写都做成：{:?}",
            mount_run.outcome
        );
        let (mount_writes, mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(
                &mount_run.mount_operations,
                &execution.device_width.fixed_geometry(),
            );
        let phase_of_each_write = writable_mount_phase_of_each_write(
            &mount_writes,
            mount_run.publishes_before_the_deletion_queue_continuation,
            mount_run.publishes_before_the_instance_table_sweep,
        );
        let root_of_the_row_publish = (0..mount_writes.len())
            .find(|write| {
                phase_of_each_write[*write] == WritableMountPhase::RowPublish
                    && mount_writes[*write].kind == StepKind::RootRecordFua
            })
            .expect("前提：写行那次发布写了根槽");
        let records_of_the_row_publish: Vec<usize> = (0..root_of_the_row_publish)
            .filter(|write| {
                phase_of_each_write[*write] == WritableMountPhase::RowPublish
                    && mount_writes[*write].kind == StepKind::JournalRecord
            })
            .collect();
        let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
            .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
            .collect();
        assert_eq!(
            (
                records_of_the_row_publish.len(),
                rotation_after_the_root.len()
            ),
            (2, 2),
            "前提：写行那次发布在根槽之前每块盘写一份记录，根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
        );
        let segment_of_the_first_record =
            segment_containing_the_write(&mount_segments, records_of_the_row_publish[0]);
        let first_write_of_the_segment = mount_segments[segment_of_the_first_record][0];
        let last_rotation = rotation_after_the_root[rotation_after_the_root.len() - 1];
        let withheld_among_them: Vec<usize> = records_of_the_row_publish
            .iter()
            .map(|record| record - first_write_of_the_segment)
            .collect();
        // 前提：两次崩溃之后只读恢复落的版本不同（同一份盘面在这里照摆法叠一遍）。
        let mut second_crash_image = first_crash_image.clone();
        second_crash_image.apply_writes(&mount_writes[..first_write_of_the_segment]);
        let persisted_writes_of_the_second_crash: Vec<RetainedWrite> = (first_write_of_the_segment
            ..=last_rotation)
            .filter(|write| !records_of_the_row_publish.contains(write))
            .map(|write| mount_writes[write].clone())
            .collect();
        second_crash_image.apply_writes(&persisted_writes_of_the_second_crash);
        let version_after_the_first_crash =
            recover(&first_crash_image, JournalPolicy::Consult).effective_root;
        let version_after_the_second_crash =
            recover(&second_crash_image, JournalPolicy::Consult).effective_root;
        assert_ne!(
            version_after_the_first_crash, version_after_the_second_crash,
            "前提：第一次崩溃落在起点那一版、二次崩溃落在写行那一版，两次只读恢复落的版本不同"
        );
        let injection = inject_crashes_into_history_drawing_second_crashes(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index: overwrite_segment_index,
                writes_from_the_segment_on: 1,
                withheld_among_them: &[0],
            },
            SecondCrashPointDraw::ExactlyOneSecondCrashState {
                segment_index: segment_of_the_first_record,
                writes_from_the_segment_on: last_rotation + 1 - first_write_of_the_segment,
                withheld_among_them: withheld_among_them.leak(),
            },
            None,
            &given_states_engine_for("crash_injection_unit_tests"),
        );
        let tally = &injection.tally;
        assert_eq!(
            (
                tally.crash_points,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.crash_points_after_a_step_reading_under_injected_faults
            ),
            (1, 1, 0),
            "前提：只摆写死的那一个第一次崩溃态、它上面的可写挂载做成、这段历史没有带读故障的步：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.second_crash_points,
                tally
                    .second_crash_points_by_phase
                    .get(&WritableMountPhase::RowPublish)
                    .copied()
            ),
            (1, Some(1)),
            "前提：只摆写死的那一个二次崩溃，崩在写行里：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
            ),
            (1, 1),
            "二次崩溃镜像上再挂以判据 N-配置 拒、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
            tally.render()
        );
        let remount_findings: Vec<String> = injection
            .new_findings
            .iter()
            .filter(|finding| {
                matches!(
                    finding.stage,
                    CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
                )
            })
            .map(|finding| {
                finding
                    .observation
                    .model_disagreement
                    .as_ref()
                    .unwrap_or_else(|| panic!("拒得不合法记成模型对不上：{}", finding.render()))
                    .implementation_answer
                    .clone()
            })
            .collect();
        let [remount_finding] = remount_findings.as_slice() else {
            panic!("第三截再挂那一判要恰好一条新发现：{remount_findings:?}");
        };
        let branch_of_the_second_crash_version = format!(
            "{:?}",
            WitnessedCounterBranchThatHolds::NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable
        );
        let branch_of_the_first_crash_version = format!(
            "{:?}",
            WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion
        );
        assert!(
            remount_finding.contains(&branch_of_the_second_crash_version),
            "所选那一版是写行那一版：它的末条记录与计数器等于 c_见证 的记录都读不出，判官要列 {branch_of_the_second_crash_version}：{remount_finding}"
        );
        assert!(
            !remount_finding.contains(&branch_of_the_first_crash_version),
            "所选那一版不是第一次崩溃落的起点那一版（那一版的末条记录读得出），不许列 {branch_of_the_first_crash_version}：{remount_finding}"
        );
    }

    /// 第三截「所选那一版」的接线在计数层分得开的靶向态（攻方改法 K3-a，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md`
    /// 第二节 K3 与第三节第 2 条；攻方探针 `research/prompts/m3-formal-code-b21-r1-opus-model/probe/opus_probe.rs` 的 `k3_sweep`，
    /// 产物 `…/out/k3-sweep.txt` 那一行 `first(last=80,withheld=[0, 1, 4]) second(last=19,withheld=[0, 1, 3])`）。与上两条同一段历史。
    /// 第一次崩溃从第一个文件那次发布的第一份记录起，摆到覆盖写那次的根槽为止：第一个文件那次的两份记录与它根槽之后第二槽系统配置轮换扣下，
    /// 别的（第一个文件那次的根槽、第一槽轮换，覆盖写那次的单元、记录与根槽）都落了——越过记录与根槽之间、根槽与轮换之间的屏障。
    /// 第二截那次可写挂载的写行那次发布上摆一个二次崩溃：写行那次的两份记录与根槽之后第一槽轮换扣下，根槽落了，第二槽轮换没摆到。
    /// 第三截再挂以判据 N-配置 拒；所选那一版取二次崩溃镜像上只读恢复落到的那一版（写行那一版），它的末条记录与计数器等于 c_见证 的记录都读不出，
    /// 判据为真：判过一次、不合法一次。判别力：调用点把所选那一版交成第一次崩溃那一版（`crates/mutations.tsv` 一行，原文与上一条那一行相同）时
    /// 判据复算为假、计数变 (1, 0)，红在最后一条断言（攻方在副本上量过，`research/prompts/m3-formal-code-b21-r1-opus-model/out/k3-sweep.txt`）。
    /// 上一条那一态在计数层分不开这两种接线，只在判官列出的那一支上分得开；这一态在计数层就分得开。
    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
    #[test]
    fn a_second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal(
    ) {
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, segments, first_write_of_the_overwrite) =
            writes_segments_and_first_write_of_the_overwrite(&history, execution);
        let root_of_the_overwrite = (first_write_of_the_overwrite..writes.len())
            .find(|write| writes[*write].kind == StepKind::RootRecordFua)
            .expect("前提：覆盖写那次发布写了根槽");
        let root_of_the_first_file = (0..first_write_of_the_overwrite)
            .rev()
            .find(|write| writes[*write].kind == StepKind::RootRecordFua)
            .expect("前提：第一个文件那次发布写了根槽");
        let root_before_the_first_file = (0..root_of_the_first_file)
            .rev()
            .find(|write| writes[*write].kind == StepKind::RootRecordFua)
            .expect("前提：第一个文件之前那次发布写了根槽");
        let records_of_the_first_file: Vec<usize> = (root_before_the_first_file + 1
            ..root_of_the_first_file)
            .filter(|write| writes[*write].kind == StepKind::JournalRecord)
            .collect();
        let rotation_after_the_first_file: Vec<usize> = (root_of_the_first_file + 1
            ..first_write_of_the_overwrite)
            .filter(|write| writes[*write].kind == StepKind::SystemConfigurationSlot)
            .collect();
        assert_eq!(
            (
                records_of_the_first_file.len(),
                rotation_after_the_first_file.len()
            ),
            (2, 2),
            "前提：第一个文件那次发布在根槽之前每块盘写一份记录，根槽之后每块盘轮换一槽系统配置"
        );
        let first_crash_segment_index =
            segment_containing_the_write(&segments, records_of_the_first_file[0]);
        let first_write_of_the_first_crash_segment = segments[first_crash_segment_index][0];
        let withheld_in_the_first_crash: Vec<usize> = records_of_the_first_file
            .iter()
            .chain(std::iter::once(&rotation_after_the_first_file[1]))
            .map(|write| write - first_write_of_the_first_crash_segment)
            .collect();
        let writes_of_the_first_crash =
            root_of_the_overwrite + 1 - first_write_of_the_first_crash_segment;
        let mut first_crash_image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        first_crash_image.apply_writes(&writes[..first_write_of_the_first_crash_segment]);
        let persisted_writes_of_the_first_crash: Vec<RetainedWrite> = (0
            ..writes_of_the_first_crash)
            .filter(|within_the_state| !withheld_in_the_first_crash.contains(within_the_state))
            .map(|within_the_state| {
                writes[first_write_of_the_first_crash_segment + within_the_state].clone()
            })
            .collect();
        first_crash_image.apply_writes(&persisted_writes_of_the_first_crash);
        // 第二截那次挂载写出的写表：与注入里那一次同一份盘面、同一份参数；挂载之后那一次发布写的内容随崩溃点变，不在挂载写表里。
        let mount_run = mount_and_publish_once_on_the_crash_image(
            &first_crash_image,
            &execution.device_width.parameters(),
            execution,
            history.seed,
            &CrashPoint {
                segment_index: first_crash_segment_index,
                persisted_within_the_segment: (0..writes_of_the_first_crash)
                    .map(|within_the_state| {
                        !withheld_in_the_first_crash.contains(&within_the_state)
                    })
                    .collect(),
                step: StepPosition::Operation(0),
                operation_kind: Some(HistoryOperationKind::PublishOverwrite),
            },
        );
        assert!(
            matches!(mount_run.outcome, WritableMountOutcome::Published),
            "前提：第一次崩溃镜像上的可写挂载与之后那一次发布都做成：{:?}",
            mount_run.outcome
        );
        let (mount_writes, mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(
                &mount_run.mount_operations,
                &execution.device_width.fixed_geometry(),
            );
        let phase_of_each_write = writable_mount_phase_of_each_write(
            &mount_writes,
            mount_run.publishes_before_the_deletion_queue_continuation,
            mount_run.publishes_before_the_instance_table_sweep,
        );
        let root_of_the_row_publish = (0..mount_writes.len())
            .find(|write| {
                phase_of_each_write[*write] == WritableMountPhase::RowPublish
                    && mount_writes[*write].kind == StepKind::RootRecordFua
            })
            .expect("前提：写行那次发布写了根槽");
        let records_of_the_row_publish: Vec<usize> = (0..root_of_the_row_publish)
            .filter(|write| {
                phase_of_each_write[*write] == WritableMountPhase::RowPublish
                    && mount_writes[*write].kind == StepKind::JournalRecord
            })
            .collect();
        let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
            .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
            .collect();
        assert_eq!(
            (
                records_of_the_row_publish.len(),
                rotation_after_the_root.len()
            ),
            (2, 2),
            "前提：写行那次发布在根槽之前每块盘写一份记录，根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
        );
        let second_crash_segment_index =
            segment_containing_the_write(&mount_segments, records_of_the_row_publish[0]);
        let first_write_of_the_second_crash_segment = mount_segments[second_crash_segment_index][0];
        let withheld_in_the_second_crash: Vec<usize> = records_of_the_row_publish
            .iter()
            .chain(std::iter::once(&rotation_after_the_root[0]))
            .map(|write| write - first_write_of_the_second_crash_segment)
            .collect();
        // 前提：两次崩溃之后只读恢复落的版本不同（同一份盘面在这里照摆法叠一遍）。
        let mut second_crash_image = first_crash_image.clone();
        second_crash_image.apply_writes(&mount_writes[..first_write_of_the_second_crash_segment]);
        let persisted_writes_of_the_second_crash: Vec<RetainedWrite> =
            (first_write_of_the_second_crash_segment..=rotation_after_the_root[0])
                .filter(|write| {
                    !withheld_in_the_second_crash
                        .contains(&(write - first_write_of_the_second_crash_segment))
                })
                .map(|write| mount_writes[write].clone())
                .collect();
        second_crash_image.apply_writes(&persisted_writes_of_the_second_crash);
        assert_ne!(
            recover(&first_crash_image, JournalPolicy::Consult).effective_root,
            recover(&second_crash_image, JournalPolicy::Consult).effective_root,
            "前提：第一次崩溃落在覆盖写那一版、二次崩溃落在写行那一版，两次只读恢复落的版本不同"
        );
        let injection = inject_crashes_into_history_drawing_second_crashes(
            &history,
            execution,
            CrashPointDraw::ExactlyOneCrashState {
                segment_index: first_crash_segment_index,
                writes_from_the_segment_on: writes_of_the_first_crash,
                withheld_among_them: withheld_in_the_first_crash.leak(),
            },
            SecondCrashPointDraw::ExactlyOneSecondCrashState {
                segment_index: second_crash_segment_index,
                writes_from_the_segment_on: rotation_after_the_root[0] + 1
                    - first_write_of_the_second_crash_segment,
                withheld_among_them: withheld_in_the_second_crash.leak(),
            },
            None,
            &given_states_engine_for("crash_injection_unit_tests"),
        );
        let tally = &injection.tally;
        assert_eq!(
            (
                tally.crash_points,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.crash_points_after_a_step_reading_under_injected_faults
            ),
            (1, 1, 0),
            "前提：只摆写死的那一个第一次崩溃态、它上面的可写挂载做成、这段历史没有带读故障的步：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.second_crash_points,
                tally
                    .second_crash_points_by_phase
                    .get(&WritableMountPhase::RowPublish)
                    .copied()
            ),
            (1, Some(1)),
            "前提：只摆写死的那一个二次崩溃，崩在写行里：{}",
            tally.render()
        );
        assert_eq!(
            (
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
            ),
            (1, 1),
            "二次崩溃镜像上再挂以判据 N-配置 拒，对着二次崩溃镜像上只读恢复落到的那一版复算判据为真：判过一次、不合法一次：{}",
            tally.render()
        );
    }

    /// 清扫发布那一版的根身份（实例, checkpoint_txg），两条用例共用：R2 那一版就是它。
    const SWEEP_PUBLISH_OF_THE_C704_CASES: (InstanceGeneration, CheckpointTxg) =
        (InstanceGeneration(3), CheckpointTxg(9));
    /// 清扫删掉的那一行与留下的那一行。
    const ROW_DELETED_BY_THE_SWEEP: InstanceRow = InstanceRow {
        instance: InstanceGeneration(1),
        selected_root_txg: CheckpointTxg(4),
        applied_transaction_high_water: 2,
    };
    const ROW_KEPT_BY_THE_SWEEP: InstanceRow = InstanceRow {
        instance: InstanceGeneration(2),
        selected_root_txg: CheckpointTxg(7),
        applied_transaction_high_water: 5,
    };

    /// 一次只读恢复的字段，拿来比的那一版（R2）的根身份是 `root_identity`、实例表行是 `instance_table_rows`。恢复报告与其余字段取自一对空盘上的
    /// 只读恢复（择不到系统配置，那几样都是 None）：C704 那一臂只看 R2 的根身份与实例表行。
    fn recovery_fields_read_comparing_a_version_with_these_instance_table_rows(
        root_identity: (InstanceGeneration, CheckpointTxg),
        instance_table_rows: Vec<InstanceRow>,
    ) -> RecoveryFieldsRead {
        let blank_pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 4 << 30);
        let mut read = read_recovery_fields(&blank_pool, ComparedVersion::TheRecoveredVersion);
        let empty_pointer = singlefs_core::pointer::NodePointer::empty_root();
        read.compared_version = Some(singlefs_harness::mount_transparency::VersionFieldsRead {
            root: singlefs_core::root_record::RootRecord {
                filesystem_identifier: [7u8; 16],
                unmount_marker:
                    singlefs_core::root_record::UnmountMarker::NotWrittenByTheUnmountSequence,
                instance: root_identity.0,
                checkpoint_txg: root_identity.1,
                tree_table: empty_pointer,
                tree_identifier_watermark: 0,
                rollback_floor: CheckpointTxg(0),
                instance_table: empty_pointer,
                mapping_root: empty_pointer,
                allocation_record_tree_root: empty_pointer,
                reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
                pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
            },
            root_record_bytes: Vec::new(),
            content: singlefs_harness::mount_transparency::PosixContentRead::NoFile,
            instance_table_rows: Some(instance_table_rows),
            tree_table_entries: None,
        });
        read
    }

    fn rows_deleted_by_the_sweep_of_the_c704_cases() -> RowsDeletedByTheInstanceTableSweepPublish {
        RowsDeletedByTheInstanceTableSweepPublish {
            sweep_publish: SWEEP_PUBLISH_OF_THE_C704_CASES,
            rows_deleted: vec![ROW_DELETED_BY_THE_SWEEP],
            heads_erased: 0,
        }
    }

    /// C704（挂载透明不认清扫发布）处置那一臂：R2 那一版是清扫发布那一版，它的实例表里却仍有清扫说删掉了的行 ⇒ 交回一条违例，
    /// 名字是 [`MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH`]，按字段分的判红计数里它加 1；
    /// R2 照原样交回、不记放回。判别力（`crates/mutations.tsv` 里点名这条的行）：被删的行还在也当成都不在、放回去再比时一条违例都没有；
    /// 判红不进计数时那一格是 None。
    #[test]
    fn a_sweep_publish_still_holding_the_rows_it_deleted_is_a_mount_transparency_violation_counted_once(
    ) {
        let read = recovery_fields_read_comparing_a_version_with_these_instance_table_rows(
            SWEEP_PUBLISH_OF_THE_C704_CASES,
            vec![ROW_KEPT_BY_THE_SWEEP, ROW_DELETED_BY_THE_SWEEP],
        );
        let rows_deleted = rows_deleted_by_the_sweep_of_the_c704_cases();
        let mut tally = CrashInjectionTally::default();
        let (compared, violations) = after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
            &mut tally,
            &read,
            Some(&rows_deleted),
        );
        assert_eq!(
            violations.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
            [MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH],
            "清扫发布那一版里仍有它删掉的那一行：判红一条：{violations:?}"
        );
        assert_eq!(
            tally
                .mount_transparency_violations_by_field
                .get(MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH)
                .copied(),
            Some(1),
            "按字段分的判红计数里这一样加 1：{}",
            tally.render()
        );
        assert_eq!(
            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back, 0,
            "被删的行还在：不放回"
        );
        assert_eq!(*compared, read, "R2 照原样交回给逐字段判");
    }

    /// 对照：R2 那一版是清扫发布那一版，被删的行一条都不在 ⇒ 不判红，把它们放回去之后的 R2 交给逐字段判，放回计数加 1。
    /// 判别力（`crates/mutations.tsv` 里点名这条的行）：放回时不接上被删的行，交回的 R2 里只有留下的那一行。
    #[test]
    fn a_sweep_publish_without_the_rows_it_deleted_is_compared_with_them_put_back() {
        let read = recovery_fields_read_comparing_a_version_with_these_instance_table_rows(
            SWEEP_PUBLISH_OF_THE_C704_CASES,
            vec![ROW_KEPT_BY_THE_SWEEP],
        );
        let rows_deleted = rows_deleted_by_the_sweep_of_the_c704_cases();
        let mut tally = CrashInjectionTally::default();
        let (compared, violations) = after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
            &mut tally,
            &read,
            Some(&rows_deleted),
        );
        assert!(
            violations.is_empty(),
            "被删的行都不在：不判红：{violations:?}"
        );
        assert!(
            tally.mount_transparency_violations_by_field.is_empty(),
            "按字段分的判红计数一样都没有：{}",
            tally.render()
        );
        assert_eq!(
            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back, 1,
            "放回一次"
        );
        assert_eq!(
            compared
                .compared_version
                .as_ref()
                .expect("R2 那一版在")
                .instance_table_rows,
            Some(vec![ROW_KEPT_BY_THE_SWEEP, ROW_DELETED_BY_THE_SWEEP]),
            "放回之后的 R2：留下的那一行在前，被删的接在后面"
        );
    }
}
