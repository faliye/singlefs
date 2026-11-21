//! 崩溃注入（里程碑「覆盖写、释放、回退与复用」增补 3 第 3 件）：第 1 件的每段历史跑的过程中开着内容保留的录制流，把流按屏障与 FUA 写切段，
//! 按种子摆若干个崩溃状态（一段 + 那一段里持久了哪几个写），对每个崩溃状态重建镜像、跑恢复、跑池级 checker、跑记录核对器，
//! 并问理想模型「恢复到的这一版允不允许」（`singlefs_verification::model::crash_recovery_disagreement`）。
//!
//! 枚举域与崩溃点重放（`crate::crash`）同一个：更早的段整段持久、当前段任意真子集、更晚的段一个都没持久，撕裂并进「没持久」
//! （D13（验证路线） 已定项 4）。差别只在**怎么取**：崩溃点重放在两条写死的流上全枚举，这里在任意一段随机历史上抽样——
//! 关闭重开、可写挂载、回退、抬 F、零单元发布、冷启动都在流里，而那样的流全枚举不起。
//! 屏障因此进了枚举域：少一道屏障就把两段并成一段，段内子集立刻多出「后发的写先持久」那一类状态
//! （`crates/mutations.tsv` 里「步 3：零单元发布在记录与根之间少一道屏障」那一条靠这个判得出）。
//!
//! 每段历史自己怎么跑由 `HistoryExecution` 定；崩溃状态摆在起点那一段起、最后一个跑完的操作为止的那几段上——
//! 起点（mkfs 与起点那次发布）也算，
//! 失败那一步的写不摆（模型还没判过它写出的根，目录里没有那一版）。
//!
//! 每个崩溃状态判三截（[`CrashStateStage`]；代码审阅第 2 条「崩溃注入层补可写挂载」）：
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
//! 挂载透明（独立的第四条，判法在 `singlefs_verification::mount_transparency` Y7「崩溃注入里量」、F6「加二次」）：
//! 挂载做成的每个崩溃状态上，崩溃镜像的只读恢复（R1）与只挂载之后、挂载加一次发布之后的只读恢复（R2，后者比发布之前那一版）逐字段比，
//! 挂载途中每个二次崩溃镜像的只读恢复也与 R1 比；可写挂载被拒的，理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的判拒绝本身合不合法。

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
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
use crate::history::{
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
use crate::memory_pool::{
    newest_persisted_root, root_identity_written_by, some_publish_persisted_without_its_root,
    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, RecordCheck,
    RecordStreamContinuity, RecordStreamSeam, RetainedWrite, SparseBlockDevice, SECTOR_BYTES,
};
use crate::model::{
    crash_recovery_disagreement, ModelDisagreement, ModelDisagreementAspect, ModelRefusalReason,
    ModelRootKey, ObservedReadBack, ObservedRefusalReason,
};
use crate::writable_mount_phases::writable_mount_phase_of_each_write;
/// 可写挂载的分段与崩溃镜像落池住在 `crate::writable_mount_phases`（不拿随机历史当输入的用例经那一份引）；崩溃注入的用例照旧从这里引。
pub use crate::writable_mount_phases::{
    materialized_crash_image, publishes_before_the_instance_table_sweep_of_the_mount_output,
    writes_of_the_instance_table_sweep_phases, WritableMountPhase,
};

/// 恢复读回的那一版问理想模型允不允许（`singlefs_verification::model::crash_recovery_disagreement`）：比的是实现跑出来的结局与只住内存的
/// 理想模型，两边都是 Rust 代码在跑，不是判一张镜像，上不了 GPU（豁免表 `.claude/gate.d/gpu-exemptions.tsv` 那一行）。
/// 随机历史的池级 checker 与两遍恢复那一截上 GPU（给定状态表），这一截不上。
// gpu-exempt: comparison-with-the-ideal-model
fn disagreement_with_the_ideal_model(
    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    newest_persisted_root: Option<ModelRootKey>,
    read_back: &ObservedReadBack,
) -> Option<ModelDisagreement> {
    crash_recovery_disagreement(committed_versions, newest_persisted_root, read_back)
}
use crate::heavy_threads::{
    default_heavy_threads_of_the_cap_value, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use crate::model_comparison::{
    model_root_key, observed_read_back_after_a_crash, refusal_reason_of_mount_error,
    refusal_reason_of_publish_error,
};
use crate::mount_transparency::{
    judge_mount_transparency, judge_writable_mount_refusal_of_a_crash_state_of_the_pool,
    read_recovery_fields, ComparedVersion, DiskAfterTheRefusal, FieldOutcome,
    FloorRaisedByTheMount, FloorRaisesInTheMountWrites, MountTransparency, MountTransparencyAction,
    ReadFaultsInTheCrashState, ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage,
    RecoveryFieldsRead, RefusalConditionNotMet, RefusalReasonOnTheLegalRefusalList,
    UserVisibleTreeRootPointersOfTheContinuation, WritableMountRefusalVerdict,
};
use crate::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use crate::reference_filesystem_translation::{
    judge_a_second_crash_after_the_restart, JointVerdictOfTheReferenceFileSystemAndTheModel,
    ReferenceFileSystemJudgementOfACrash, ReferenceFileSystemTranslation, ReferenceFileSystemVote,
    TranslatedHistory,
};
use crate::scenario::FIXED_WRITE_TIME_SECONDS;
use crate::segments::{FixedGeometry, StepKind};
use crate::{RecordingBlockDevice, RetainedOperation, SharedStream};

mod campaign;
mod campaign_environment;
mod crash_point_draw;
mod crash_points_and_findings;
mod given_states_after_the_first_crash;
mod history_injection;
mod mount_transparency;
mod remount_idempotence;
mod report;
mod second_crashes;
mod tally;
mod writable_mount_on_the_crash_image;

pub use campaign::*;
pub use campaign_environment::*;
use crash_point_draw::*;
pub use crash_points_and_findings::*;
pub use given_states_after_the_first_crash::*;
pub use history_injection::*;
pub use mount_transparency::*;
pub use remount_idempotence::*;
pub use report::*;
pub use second_crashes::*;
pub use tally::*;
#[cfg(test)]
use tests::{
    assert_the_overwrite_rotates_one_system_configuration_slot_per_device,
    crash_injections_into_histories_reaching_every_mount_phase, given_states_engine_for,
    history_of_the_first_file_and_one_overwrite, whole_write_table_of_the_history,
    writes_segments_and_first_write_of_the_overwrite, FOUR_CRASH_POINTS,
};
pub use writable_mount_on_the_crash_image::*;

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

// ============================== 给定状态表（四件） ==============================
/// 给定状态表的形态、结论、判器 trait 与 CPU 判法住在 `crate::crash_given_states`（流水线经那一份引，不经崩溃注入）；产品实现是
/// 崩溃点重放的引擎 `crate::crash_amplification::GivenStatesEngine`（表先写进给定状态表那一类的判定库、读回再判、结论存库复用）；用例照旧从这里引。
pub use crate::crash_given_states::{
    count_vector_of_the_given_states, judge_a_given_state_on_the_cpu,
    judge_the_given_states_on_the_cpu, ClaimedVersionOfAGivenState, ClaimedVersionsOfTheStates,
    GivenStateConclusion, GivenStatesConclusionsOfATable, GivenStatesConclusionsSource,
    GivenStatesJudge, GivenStatesOfATable, GivenStatesOfATableWithItsColumns,
    POOL_CHECKER_VIOLATION_WITHOUT_ITS_DETAIL,
};

/// 给定状态表的基线与写表（「基线改成 mkfs 之后」；同日弹窗定创世那几次发布照样进记录核对器，「先查 GPU 能不能核，
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

/// 判可写挂载被拒合不合法要的：崩溃后镜像（拒的判据在它上面复算，拒之后的池要与它逐字节相同）、它上面只读恢复落到的那一版、
/// 那一次只读读出在不在模型的允许集合里（问模型的结论）。第二截交第一次崩溃的镜像与第一截的结论；第三截交二次崩溃镜像与那一个二次崩溃上的结论，
/// 读故障的认定沿用第一次崩溃那个态的（第二截那次挂载起在第一次崩溃的镜像上，不注入读故障）。
struct RefusalOnTheCrashImage<'crash> {
    crash_image_materialized: &'crash MemoryPool,
    /// 本池 fsid（建池参数里那一份）：复算判据用的系统配置只从这个 fsid 的槽里择。
    filesystem_identifier_of_the_pool: &'crash [u8; 16],
    version_landed_on_after_the_crash: Option<(InstanceGeneration, CheckpointTxg)>,
    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
    /// 这个崩溃态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：没注入的，判据 N-配置 复算为真即拒得不合法（C670），
    /// 「读阶段判完之后系统配置又读不出」那一拒的判据复算为真也拒得不合法（改法 G4）。
    read_faults: ReadFaultsInTheCrashState,
}

/// 挂载途中的一次二次崩溃：挂载那一段的第几段、段内哪几个写持久了（靶向用例摆的那一个从这一段的第一个写起数，可以越过段尾）、崩在挂载的哪一段。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SecondCrashPoint {
    segment_index: usize,
    persisted_within_the_segment: Vec<bool>,
    phase: WritableMountPhase,
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

/// 二次崩溃镜像上再挂被拒的那一次：成员、错误、拒之后的池、二次崩溃镜像那一份判拒要的、崩在挂载的哪一段。
struct RemountRefusedAfterASecondCrash<'remount> {
    member: &'remount str,
    refusal: &'remount MountError,
    pool_after_the_refusal: &'remount MemoryPool,
    second_crash_image: &'remount RefusalOnTheCrashImage<'remount>,
    phase: WritableMountPhase,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::generate_history;

    pub(super) const FOUR_CRASH_POINTS: CrashPointDraw = CrashPointDraw::Sampled {
        crash_points_per_history: 4,
    };

    /// 起在自己那一份一次性库上的引擎：引擎连着的是这一份的临时 prune-server，一次性库要比引擎活得长。
    /// 字段按声明次序丢：先丢引擎（放掉连接），再丢一次性库（停临时服务、删目录）。
    pub(super) struct GivenStatesEngineOnItsOwnDisposableLibrary {
        engine: crate::crash_amplification::GivenStatesEngine,
        _disposable_library: crate::crash_library::DisposableLibrary,
    }

    impl GivenStatesEngineOnItsOwnDisposableLibrary {
        pub(super) fn engine(&self) -> &crate::crash_amplification::GivenStatesEngine {
            &self.engine
        }
    }

    /// 这份文件的单测起崩溃点重放的引擎判给定状态表：表与结论都进这一次自己起的一次性库（`crate::crash_library::DisposableLibrary`：
    /// 临时 prune-server，不连真服务；同一个进程里并行的别的用例各连各的），配了 GPU 且编进了
    /// `gpu` 特性上卡、否则 CPU。`label_of_the_report` 只进项名（行里报的名字）与一次性库的标签，种子与规模不进键（块键只由表的内容定）。
    pub(super) fn given_states_engine_for(
        label_of_the_report: &str,
    ) -> GivenStatesEngineOnItsOwnDisposableLibrary {
        // 一次性库留着：库单测只对临时服务（`crate::crash_library` 文件头），判的是几个状态的小表，整条按测试函数键复用
        let disposable_library = crate::crash_library::DisposableLibrary::new(label_of_the_report)
            .unwrap_or_else(|refusal| panic!("一次性库建不了：{refusal}"));
        // 先设一次性库、再拿改环境的锁（全仓用例都是这个次序）：引擎起步时开的是这一份，起完就放
        let the_library_of_this_process = disposable_library.make_it_the_library_of_this_process();
        let one_test_at_a_time =
            crate::crash_amplification::one_test_changes_the_environment_at_a_time();
        let engine = crate::crash_amplification::GivenStatesEngine::start_from_environment(
            crate::crash_amplification::GivenStatesItem {
                item_name: format!("crash_injection_unit_tests::{label_of_the_report}"),
            },
            &crate::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"));
        drop(one_test_at_a_time);
        drop(the_library_of_this_process);
        GivenStatesEngineOnItsOwnDisposableLibrary {
            engine,
            _disposable_library: disposable_library,
        }
    }

    /// 挂载每一段都摆得到二次崩溃的两段历史（U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：从第一个文件起的
    /// 随机一段（起点就是攻方 `research/prompts/m3-inv-a-r1-opus-output.md` 一·2 的「新池新建文件做完的镜像」），与第一个文件之后经挂着的会话
    /// unlink 它的一段（意图那次 + 最后一批：崩溃状态有落在意图那一版上的，那一版上的可写挂载续做、写出续做那一段，
    /// 之后那一次发布落在删完那一版上）。两段同一种摆法（短段整段枚举），计数并在一起。再并上一段崩溃后那次可写挂载删得了行的历史
    /// （[`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]，有一行就扫）：
    /// 清扫发布那一段从它来；那段历史上没有孤儿、一个头都不抹。最后并上一段留下实例 1 的孤儿再把实例 1 的根全轮出根环的历史
    /// （[`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans`]，同一种摆法）：
    /// 抹头那一段从它来。
    pub(super) fn crash_injections_into_histories_reaching_every_mount_phase(
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
                    given_states_engine_for("crash_injection_unit_tests").engine(),
                )
            })
            .collect();
        injections.push(
            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                given_states_engine_for("crash_injection_unit_tests").engine(),
            ),
        );
        injections.push(
            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                given_states_engine_for("crash_injection_unit_tests").engine(),
            ),
        );
        let mut tally = CrashInjectionTally::default();
        for injection in &injections {
            tally.absorb(&injection.tally);
        }
        (tally, injections)
    }

    /// 起点（取号、暖机、第一个文件）之后一次覆盖写、不重挂的历史，与跑它用的执行（单元区 384 槽的小盘，不逐步跑 checker）。
    pub(super) fn history_of_the_first_file_and_one_overwrite(
    ) -> (GeneratedHistory, HistoryExecution) {
        use crate::history::{HistoryDeviceWidth, PerStepChecker};
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
    pub(super) fn writes_segments_and_first_write_of_the_overwrite(
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
    pub(super) fn assert_the_overwrite_rotates_one_system_configuration_slot_per_device(
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
    pub(super) fn whole_write_table_of_the_history(
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
}
