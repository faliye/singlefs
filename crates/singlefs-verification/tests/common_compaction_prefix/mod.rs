//! 整理（自建 GC）两条崩溃流与生命周期闭环流共用的搭建（里程碑四二·3、三·1；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G6）：
//! 公共前缀是 `common::build_pool` 那一段（mkfs → 取号 → 暖机 → 新池新建文件 A，第一个文件一个数据单元），之后按流各接几步：
//! - 整理搬迁流（G6 前缀 a）：A → B、C 两次顺序写（多记录流那两次，内容由调用方给，与 `common_crash_points` 的那两份同一份字节）
//!   → 重挂（可写挂载：取号、写行、暖机）→ 整理签发 → 一批 → 最后一批（同一次发布删意图）；
//! - 搬被快照共享的块流（二·3「搬被共享的块一条」）：A → 建快照 S → 快照下覆盖写 W（A 的四个单元进 deadlist、映射条目留存）→ 重挂 → 整理签发 → 几批 → 最后一批，
//!   R 里装着只被快照引用的那几个单元（留存的映射条目跟着搬到目的地）；
//! - 生命周期闭环流（三·1）：A（分配）→ 覆盖写 → 建快照 → 再覆盖写 → 删文件（unlink 意图、最后一批）→ 重挂 → 整理搬共享块（签发、几批、最后一批）
//!   → 删快照（会话里一次调用：意图、各批）→ 回收（抬 F 到准入上限、建 inode，直到 R 整段回收、会话对它发 discard）
//!   → 在搬出来的空段上再分配（建 inode，直到 R 里有一条未释放的记录）。
//!
//! R 取段 [`REGION_START_SLOT_OF_THE_CRASH_FLOWS`]：第一个文件那次发布的提交内生块开在这一段，重挂之后它不是本次挂载开过的段（判决 G6「选段资格要 R 不在
//! 本次挂载开过的段里」），三条流都强制选它、一批 [`COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS`] 个（判决 G6：批大小 1–3，单元段写数 ≤ 30）。
//! 整理一次一次发（会话的发布预算压到 1，`compaction_plan::publish_budget_of_a_compaction_run`），每一次发布记成流里一步，崩溃放量照它设点。
//!
//! 这一份只录流、交回每一步发出的那段录制流与它的输出，再给观察者用的几样（流里每一版、读快照与读文件、在崩溃镜像上可写挂载一次续做到底）；
//! 崩溃点的设点、观察者的计数与判定都在崩溃点重放的用例里。快照那几步的发布照 `common_snapshot_prefix` 那一份
//! （入口名同快照流，节点与快照流共享）。
#![allow(
    dead_code,
    reason = "共用的搭建模块：每个测试文件只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use std::ops::Range;
use std::sync::Arc;

use super::common::{
    build_pool, file_content, geometry, parameters, publish_overwrite_in_process,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
    FIXED_WRITE_TIME_SECONDS, OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
};
use super::common_snapshot_prefix::{
    create_snapshot, live_snapshot_trees_of, overwrite_content, overwrite_under_the_snapshot,
    unlink_intent, unlink_step, SnapshotFlowStepKind, INSTANCE_OF_THE_FLOWS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{
    region_is_empty_by_the_records, CompactionBatchSize, CompactionBatchUnits,
    CompactionRegionExclusion, CompactionRunForcing, CompactionRunStop, CompactionSwitches,
    EmptyClusterSegmentsFloor, ForcedReleaseChecksumMismatchAfterIssue,
    RelocationDestinationPolicy, RelocationPublishOrder,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mount::{
    mount_writable_with_compaction_switches,
    mount_writable_with_test_only_switches_and_deletion_switches,
    raise_rollback_floor_to_the_admission_ceiling, DeletionQueueContinuationOutcome, Mounted,
    RaiseToTheAdmissionCeiling, ShadowLedger,
};
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession, UserChange};
use singlefs_core::recovery::{
    read_the_first_file_of_a_snapshot, walk_to_file, PoolReader, SnapshotFileReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{PoolVersion, TransactionOutput};
use singlefs_format::SLOT_BYTES;
use singlefs_verification::memory_pool::{
    writes_and_segments, writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::scenario::deletion_history_switches;
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RecordedOperation, RecordedOperationKind, RetainedOperation};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 这一份的仓内路径（与 `file!()` 归一之后相同）。
pub const FILE_OF_THE_COMPACTION_PREFIX: &str =
    "crates/singlefs-verification/tests/common_compaction_prefix/mod.rs";

/// 三条流强制选的整理区域 R 的起始槽：第一个文件那次发布的提交内生块开在这一段（默认几何，D3（空间分配） 已定项 10 ⑤ 的开放段 [50240, 50304)），
/// 重挂之后本次挂载不在它上面开段。两块盘同槽，意图按盘 0 点名。
pub const REGION_START_SLOT_OF_THE_CRASH_FLOWS: u64 = 50240;

/// 一批搬几个单元（判决 G6：批大小 1–3；取 2，一批里有不止一个单元，单元段写数 2 ×（提交内生块 + 2）仍在 30 以内）。
pub const COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS: usize = 2;

/// 会话里整理一趟只发一次：「已经过去的发布数」取得比 `3S − 4` 大，发布预算落到下限 1（`compaction_plan::publish_budget_of_a_compaction_run`）。
pub const PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE: u64 = 1_000;

/// 一条意图至多几次整理发布（签发、各批、最后一批）：R 里一个段至多 64 个单元，批大小 2，再加签发；录流时超了就是整理停不下来。
pub const COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST: usize =
    1 + 64 / COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS;

/// 生命周期闭环流「回收」那一段至多几轮（抬 F 到准入上限、建一次 inode）：R 整段回收、会话对它发 discard 之前超了就是回收不下来。
pub const LIFECYCLE_RECLAIM_ROUNDS_AT_MOST: usize = 8;

/// 生命周期闭环流「再分配」那一段至多建几次 inode：R 里有一条未释放的记录之前超了就是分配不到搬出来的空段上。
pub const LIFECYCLE_REALLOCATION_PUBLISHES_AT_MOST: usize = 12;

/// 生命周期闭环流第一次覆盖写（建快照之前）的内容：一个数据单元装得下、与第一个文件与快照下覆盖写那一份都不同。
#[must_use]
pub fn content_overwritten_before_the_snapshot() -> Vec<u8> {
    (0..2000_usize)
        .map(|index| u8::try_from((index * 7 + 3) % 241).expect("小于 256"))
        .collect()
}

/// R：盘 0 上段 [`REGION_START_SLOT_OF_THE_CRASH_FLOWS`]。
#[must_use]
pub fn region_of_the_crash_flows() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(REGION_START_SLOT_OF_THE_CRASH_FLOWS),
    )
    .expect("64 的倍数、在六字节槽号以内")
}

/// 三条流的整理开关：强制选 R、一批 [`COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS`] 个，别的字段取产品值（源落点在映射换位置的那一次发布里释放）。
/// 录流的会话与观察者在崩溃镜像上的可写挂载用同一份。
#[must_use]
pub fn compaction_switches_of_the_crash_flows() -> CompactionSwitches {
    CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS).expect("至少 1"),
        ),
        forced_region: Some(region_of_the_crash_flows()),
        run_forcing: CompactionRunForcing::FollowTheWatermarks,
        region_exclusion: CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
        destination_policy:
            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
        release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue::NotForced,
        empty_cluster_segments_floor: EmptyClusterSegmentsFloor::OfTheFirstVersion,
        relocation_publish_order:
            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
    }
}

/// 三条流。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionCrashFlow {
    /// A → B → C → 重挂 → 签发 → 一批 → 最后一批。
    CompactionRelocation,
    /// A → S → W → 重挂 → 签发 → 几批 → 最后一批（R 里有只被快照引用的单元）。
    CompactionRelocatingABlockSharedWithASnapshot,
    /// A → 覆盖写 → S → 再覆盖写 → 删文件 → 重挂 → 整理搬共享块 → 删快照 → 回收 → 在搬出来的空段上再分配。
    LifecycleClosure,
}

/// 流里一步是哪一种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionFlowStepKind {
    /// 多记录流的 B、C（实例 1）。
    SequentialWrite,
    /// 生命周期闭环流建快照之前那一次覆盖写（实例 1）。
    OverwriteBeforeTheSnapshot,
    /// 快照流那几步（实例 1，发布照 `common_snapshot_prefix`）。
    Snapshot(SnapshotFlowStepKind),
    /// 重开镜像、可写挂载（取号、写行、暖机）。
    MountWritable,
    /// 整理签发：一次只把意图写进 logged_ops 树的发布。
    CompactionIssue,
    /// 整理的一批（意图还在）。
    CompactionBatch,
    /// 整理的最后一批：同一次发布删意图。
    CompactionFinalBatch,
    /// 会话里删快照一次调用：意图那次发布与全部批。
    DestroySnapshotThroughTheSession,
    /// 抬 F 到准入上限那一串（先写系统配置、再推带新 F 的空发布）。
    RaiseFloorToTheAdmissionCeiling,
    /// 会话里建一个空 inode（文件删光之后的一版只许建 inode）。
    NewInodesThroughTheSession,
}

impl CompactionFlowStepKind {
    /// 入口的名字：崩溃放量那一路按它设点（`<流程>.<周期>.<步>` 的流程那一段；设点在 `common_crash_points`）。
    /// 快照那几步与快照流同名（节点共享），B、C 与多记录流同名（`common_crash_points::sequential_write`）。
    #[must_use]
    pub const fn entry_name(self) -> &'static str {
        match self {
            Self::SequentialWrite => "sequential_write",
            Self::OverwriteBeforeTheSnapshot => "publish_overwrite",
            Self::Snapshot(kind) => kind.entry_name(),
            Self::MountWritable => "mount_writable",
            Self::CompactionIssue => "publish_compaction_issue",
            Self::CompactionBatch => "publish_compaction_batch",
            Self::CompactionFinalBatch => "publish_compaction_final_batch",
            Self::DestroySnapshotThroughTheSession => "destroy_snapshot_through_the_session",
            Self::RaiseFloorToTheAdmissionCeiling => {
                "raise_rollback_floor_to_the_admission_ceiling"
            }
            Self::NewInodesThroughTheSession => "publish_new_inodes_through_the_session",
        }
    }

    /// 计数的名字里这一种那一段（观察者按恢复落到哪一种的那一版分别计数）。
    #[must_use]
    pub const fn counter_name(self) -> &'static str {
        match self {
            Self::SequentialWrite => "sequential_write",
            Self::OverwriteBeforeTheSnapshot => "overwrite_before_the_snapshot",
            Self::Snapshot(kind) => kind.counter_name(),
            Self::MountWritable => "mount_writable",
            Self::CompactionIssue => "compaction_issue",
            Self::CompactionBatch => "compaction_batch",
            Self::CompactionFinalBatch => "compaction_final_batch",
            Self::DestroySnapshotThroughTheSession => "destroy_snapshot_through_the_session",
            Self::RaiseFloorToTheAdmissionCeiling => "raise_floor_to_the_admission_ceiling",
            Self::NewInodesThroughTheSession => "new_inodes_through_the_session",
        }
    }
}

/// 流里一步：哪一种、它发出的那段录制流（绝对下标的半开区间）、它发成的每一版（按先后；一步一次发布的只有一版）、它之后会话发的 discard（盘、段起点）。
#[derive(Clone, Debug)]
pub struct CompactionFlowStep {
    pub kind: CompactionFlowStepKind,
    pub operations: Range<usize>,
    pub versions: Vec<TransactionOutput>,
    pub discards: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 录好的一条流：池（录制流在 `pool.stream` 上）、A 的输出、A 之后的每一步、建过的快照（生命周期闭环流与搬共享块流有）。
pub struct RecordedCompactionFlow {
    pub flow: CompactionCrashFlow,
    pub pool: BuiltPool,
    pub first_file: TransactionOutput,
    pub steps: Vec<CompactionFlowStep>,
    pub snapshot: Option<TreeIdentifier>,
}

impl RecordedCompactionFlow {
    /// 某一种的每一步，按先后。
    #[must_use]
    pub fn steps_of(&self, kind: CompactionFlowStepKind) -> Vec<&CompactionFlowStep> {
        self.steps.iter().filter(|step| step.kind == kind).collect()
    }

    /// A 之后每一步的入口名与它发出的那段录制流（崩溃放量照它设点，`common_crash_points::crash_points_of_a_flow_after_the_first_file`）。
    #[must_use]
    pub fn crash_point_entries_of_the_steps(&self) -> Vec<(&'static str, Range<usize>)> {
        self.steps
            .iter()
            .map(|step| (step.kind.entry_name(), step.operations.clone()))
            .collect()
    }

    /// 整理那几次发布搬走的单元：映射 key 的类标签、源落点、目的落点（盘 0 那一条位置项的槽），按先后。
    #[must_use]
    pub fn relocated_units(&self) -> Vec<RelocatedUnitOfTheFlow> {
        self.steps
            .iter()
            .flat_map(|step| step.versions.iter())
            .flat_map(|version| {
                version
                    .units_relocated_by_this_publish
                    .iter()
                    .map(|relocated| RelocatedUnitOfTheFlow {
                        unit_class: relocated.mapping_key[0],
                        is_kept_only_for_the_snapshots: matches!(
                            relocated.holder,
                            singlefs_core::transaction::RelocatedUnitHolder::KeptForTheSnapshots
                        ),
                        source: relocated
                            .source
                            .iter()
                            .map(|location| (location.device, location.slot))
                            .collect(),
                        destination: relocated
                            .destination
                            .iter()
                            .map(|location| (location.device, location.slot))
                            .collect(),
                    })
            })
            .collect()
    }
}

/// 整理搬走的一个单元（录流的形状核与必红世界的说明用）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelocatedUnitOfTheFlow {
    pub unit_class: u8,
    pub is_kept_only_for_the_snapshots: bool,
    pub source: Vec<(DeviceIdentity, SlotNumber)>,
    pub destination: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 发一步实例 1 的发布、记下它发出的那段录制流，池接到它那一版上。
fn record_one_publish_of_instance_one(
    pool: &mut BuiltPool,
    kind: CompactionFlowStepKind,
    publish: impl FnOnce(&mut BuiltPool) -> TransactionOutput,
) -> CompactionFlowStep {
    let start = pool.stream.operation_count();
    let output = publish(pool);
    let end = pool.stream.operation_count();
    pool.output = output.clone();
    CompactionFlowStep {
        kind,
        operations: start..end,
        versions: vec![output],
        discards: Vec::new(),
    }
}

/// 有树的一版（会话与挂载交回的现行版本）。
pub fn version_of_trees(version: &PoolVersion) -> &TransactionOutput {
    version
        .version_of_trees()
        .expect("录流在重挂之前都已经发过第一个文件：挂载与会话交回的每一版都有树")
}

/// 重开镜像、可写挂载（删除开关同快照流：批大小 1，`deletion_history_switches`），会话装上整理开关。挂载发的写录进同一条流。
fn remount_and_open_the_session(pool: &mut BuiltPool) -> (CompactionFlowStep, MountedSession) {
    let start = pool.stream.operation_count();
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_history_switches(),
    )
    .expect("重挂：可写挂载做得成");
    pool.devices = Some(devices);
    let end = pool.stream.operation_count();
    let (output, mut session) = MountedSession::of_the_mount(mounted);
    session
        .allocator
        .set_compaction_switches(compaction_switches_of_the_crash_flows());
    let mut versions = vec![version_of_trees(&output.row_publish).clone()];
    versions.extend(
        output
            .warm_up_publishes
            .iter()
            .map(|published| version_of_trees(published).clone()),
    );
    (
        CompactionFlowStep {
            kind: CompactionFlowStepKind::MountWritable,
            operations: start..end,
            versions,
            discards: Vec::new(),
        },
        session,
    )
}

/// 会话里整理一次一次发，直到最后一批同一次发布删了意图：每一次发布记成一步（签发、一批、最后一批，按 logged_ops 与搬走的单元认）。
/// 一趟只发一次（发布预算压到 1），这一趟发布都返回之后会话发的 discard 记在这一步上。
///
/// # Panics
/// 一趟一次都没发（R 不合格、签发前判不过）、发布被拒、或 [`COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST`] 次之内删不了意图。
fn publish_the_compaction_until_the_intent_is_deleted(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
) -> Vec<CompactionFlowStep> {
    let mut steps = Vec::new();
    for _publish_index in 0..COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST {
        session.compaction.publishes_since_the_last_user_change =
            PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
        let start = pool.stream.operation_count();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let run = session.run_compaction(devices).expect("整理一趟");
        let end = pool.stream.operation_count();
        assert!(
            matches!(
                run.outcome,
                CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget: 1 })
            ),
            "整理一趟只发一次、停在预算用完：{:?}",
            run.outcome
        );
        let [published] = <[TransactionOutput; 1]>::try_from(run.versions_published)
            .expect("预算 1：这一趟恰好发一次");
        let kind = match (
            published.logged_operations.is_empty(),
            published.units_relocated_by_this_publish.is_empty(),
        ) {
            (false, true) => CompactionFlowStepKind::CompactionIssue,
            (false, false) => CompactionFlowStepKind::CompactionBatch,
            (true, false) => CompactionFlowStepKind::CompactionFinalBatch,
            (true, true) => panic!(
                "整理一次发布既没写意图、也没搬单元（丢意图）：三条流的 R 里没有隔离槽、搬得动，不该走到这里"
            ),
        };
        steps.push(CompactionFlowStep {
            kind,
            operations: start..end,
            versions: vec![published],
            discards: run.discards.applied.clone(),
        });
        if kind == CompactionFlowStepKind::CompactionFinalBatch {
            return steps;
        }
    }
    panic!(
        "{COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST} 次整理发布之内最后一批没删意图：{:?}",
        steps.iter().map(|step| step.kind).collect::<Vec<_>>()
    );
}

/// 会话里发一次用户改动，记成一步（这一次调用发成的每一版、之后的 discard）。
fn publish_a_user_change_through_the_session(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    kind: CompactionFlowStepKind,
    change: UserChange<'_>,
) -> CompactionFlowStep {
    let start = pool.stream.operation_count();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let published = session
        .publish_user_change(devices, change)
        .unwrap_or_else(|refusal| panic!("{kind:?}：会话发得成：{refusal:?}"));
    let end = pool.stream.operation_count();
    let mut versions: Vec<TransactionOutput> = published
        .floor_raises
        .iter()
        .flat_map(|raised| raised.publishes.iter().cloned())
        .collect();
    versions.extend(
        published
            .versions_published
            .iter()
            .map(Arc::as_ref)
            .cloned(),
    );
    versions.sort_by_key(|version| version.root.checkpoint_txg);
    CompactionFlowStep {
        kind,
        operations: start..end,
        versions,
        discards: published.discards_after_the_change.applied.clone(),
    }
}

/// 抬 F 到准入上限（会话的分配器与现行那一版）：上限不高于 F 时一个字节都不写、交回 `None`。
fn raise_the_floor_to_the_admission_ceiling(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
) -> Option<CompactionFlowStep> {
    let start = pool.stream.operation_count();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut current = version_of_trees(&session.current).clone();
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &parameters(),
        devices,
        &mut session.allocator,
        &mut current,
        session.shadow_ledger,
    )
    .expect("抬 F 到准入上限");
    let end = pool.stream.operation_count();
    session.current = PoolVersion::of_a_version_of_trees(current);
    match raised {
        RaiseToTheAdmissionCeiling::Raised(raised) => Some(CompactionFlowStep {
            kind: CompactionFlowStepKind::RaiseFloorToTheAdmissionCeiling,
            operations: start..end,
            versions: raised.publishes,
            discards: Vec::new(),
        }),
        RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. } => {
            assert_eq!(start, end, "上限不高于 F：一个字节都不写");
            None
        }
    }
}

/// R 那块盘与另一块盘上都有一条罩着 R 的未释放记录（两盘同槽）。
fn region_holds_an_unreleased_record(version: &TransactionOutput) -> bool {
    !region_is_empty_by_the_records(&version.allocation_records, region_of_the_crash_flows())
}

/// 这一步之后会话对 R 那一段发了 discard（两块盘都发）。
fn the_step_discards_the_region_on_both_devices(step: &CompactionFlowStep) -> bool {
    [DeviceIdentity(0), DeviceIdentity(1)].iter().all(|device| {
        step.discards
            .contains(&(*device, SlotNumber(REGION_START_SLOT_OF_THE_CRASH_FLOWS)))
    })
}

/// 录一条流（`tag` 是镜像文件名里那一段）。`contents_of_the_sequential_writes` 只给整理搬迁流用（B、C 的内容，多记录流那两份），别的流给空。
///
/// # Panics
/// 任何一步发布被拒、整理停不下来、回收或再分配在上限之内做不成：三条流的每一步按条款都做得成。
#[must_use]
pub fn record_the_compaction_crash_flow(
    tag: &str,
    flow: CompactionCrashFlow,
    contents_of_the_sequential_writes: &[Vec<u8>],
) -> RecordedCompactionFlow {
    let mut pool = build_pool(tag);
    let first_file = pool.output.clone();
    let mut steps = Vec::new();
    let mut snapshot = None;
    match flow {
        CompactionCrashFlow::CompactionRelocation => {
            assert_eq!(
                contents_of_the_sequential_writes.len(),
                2,
                "整理搬迁流的前缀是多记录流：B、C 两次顺序写"
            );
            for content in contents_of_the_sequential_writes {
                steps.push(record_one_publish_of_instance_one(
                    &mut pool,
                    CompactionFlowStepKind::SequentialWrite,
                    |pool| {
                        sequential_write_in_process_as_instance_one_and_advance_the_pool(
                            pool, content,
                        )
                    },
                ));
            }
        }
        CompactionCrashFlow::CompactionRelocatingABlockSharedWithASnapshot => {
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot),
                create_snapshot,
            ));
            snapshot = live_snapshot_trees_of(&steps[0].versions[0])
                .last()
                .copied();
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
                overwrite_under_the_snapshot,
            ));
        }
        CompactionCrashFlow::LifecycleClosure => {
            // 增量矩阵可在闭环前加入多单元顺序写；旧闭环传空切片，写流不变。
            for content in contents_of_the_sequential_writes {
                steps.push(record_one_publish_of_instance_one(
                    &mut pool,
                    CompactionFlowStepKind::SequentialWrite,
                    |pool| {
                        sequential_write_in_process_as_instance_one_and_advance_the_pool(
                            pool, content,
                        )
                    },
                ));
            }
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::OverwriteBeforeTheSnapshot,
                |pool| {
                    let previous = pool.output.clone();
                    publish_overwrite_in_process(
                        pool,
                        &previous,
                        &content_overwritten_before_the_snapshot(),
                        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
                        INSTANCE_OF_THE_FLOWS,
                    )
                    .expect("建快照之前覆盖写")
                },
            ));
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot),
                create_snapshot,
            ));
            snapshot = live_snapshot_trees_of(&steps.last().expect("刚发布快照").versions[0])
                .last()
                .copied();
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
                overwrite_under_the_snapshot,
            ));
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkIntent),
                unlink_intent,
            ));
            // 一个数据单元的文件、批大小 1：意图之后只有最后一批（`common_snapshot_prefix::SnapshotFlowStepKind::UnlinkFinalBatch`）
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkFinalBatch),
                unlink_step,
            ));
            assert!(
                pool.output.deleted_inodes.is_empty(),
                "一个数据单元的文件：意图之后一批就删完"
            );
        }
    }
    let (mount_step, mut session) = remount_and_open_the_session(&mut pool);
    steps.push(mount_step);
    steps.extend(publish_the_compaction_until_the_intent_is_deleted(
        &mut pool,
        &mut session,
    ));
    if flow == CompactionCrashFlow::LifecycleClosure {
        let snapshot_to_destroy = snapshot.expect("生命周期闭环流建过快照");
        steps.push(publish_a_user_change_through_the_session(
            &mut pool,
            &mut session,
            CompactionFlowStepKind::DestroySnapshotThroughTheSession,
            UserChange::DestroySnapshot {
                snapshot: snapshot_to_destroy,
            },
        ));
        record_the_reclaim_and_the_reallocation(&mut pool, &mut session, &mut steps);
    }
    pool.output = version_of_trees(&session.current).clone();
    RecordedCompactionFlow {
        flow,
        pool,
        first_file,
        steps,
        snapshot,
    }
}

/// 生命周期闭环流的「回收」与「在搬出来的空段上再分配」：一轮抬 F 到准入上限、建一次 inode（建 inode 发成之后会话发 discard），
/// 直到会话对 R 整段发了 discard；再一次一次建 inode，直到 R 里有一条未释放的记录（分配器在最低的全空段开新段，D3（空间分配） 已定项 10 ①）。
/// 步数按实现的取落点走，用例只核「发了 discard、之后分配到 R 上」这两件事，不核轮数。
fn record_the_reclaim_and_the_reallocation(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    steps: &mut Vec<CompactionFlowStep>,
) {
    let mut new_inodes_published = 0_u64;
    let mut new_inodes_step =
        |pool_of_the_flow: &mut BuiltPool, session_of_the_flow: &mut MountedSession| {
            new_inodes_published += 1;
            publish_a_user_change_through_the_session(
                pool_of_the_flow,
                session_of_the_flow,
                CompactionFlowStepKind::NewInodesThroughTheSession,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS
                        + LIFECYCLE_SECONDS_FROM_THE_FIRST_WRITE_TO_THE_NEW_INODES
                        + new_inodes_published,
                },
            )
        };
    let mut is_the_region_discarded = false;
    for _round_index in 0..LIFECYCLE_RECLAIM_ROUNDS_AT_MOST {
        if let Some(raised) = raise_the_floor_to_the_admission_ceiling(pool, session) {
            steps.push(raised);
        }
        let step = new_inodes_step(pool, session);
        is_the_region_discarded = the_step_discards_the_region_on_both_devices(&step);
        steps.push(step);
        if is_the_region_discarded {
            break;
        }
    }
    assert!(
        is_the_region_discarded,
        "{LIFECYCLE_RECLAIM_ROUNDS_AT_MOST} 轮抬 F 之内 R 整段回收、会话对它发了 discard"
    );
    for _publish_index in 0..LIFECYCLE_REALLOCATION_PUBLISHES_AT_MOST {
        let step = new_inodes_step(pool, session);
        let is_reallocated = step
            .versions
            .last()
            .is_some_and(region_holds_an_unreleased_record);
        steps.push(step);
        if is_reallocated {
            return;
        }
    }
    panic!("建 {LIFECYCLE_REALLOCATION_PUBLISHES_AT_MOST} 次 inode 之内分配到 R 上");
}

/// 生命周期闭环流建 inode 那几次的调用方时间比第一个文件晚这么多秒（再逐次加一）：与之前每一次写的时间都不同。
pub const LIFECYCLE_SECONDS_FROM_THE_FIRST_WRITE_TO_THE_NEW_INODES: u64 = 600;

// ============================== 崩溃点重放要的那几样 ==============================

/// 流里的一版（A 与之后每一步发成的每一版）：根（实例，txg）、哪一种（A 是 `None`）、根记录、这一版里第一个文件的内容（没有文件是 `None`）、
/// 这一版的活快照与它们建的那一刻第一个文件的内容、这一版有没有活的整理意图、这一版与它之前整理搬走的单元的源落点（读的时候先抹掉它们，
/// 核读路径照映射走到目的地）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionOfTheCompactionFlow {
    pub version: (InstanceGeneration, CheckpointTxg),
    pub kind: Option<CompactionFlowStepKind>,
    pub root: RootRecord,
    pub first_file_content: Option<Vec<u8>>,
    pub live_snapshots: Vec<TreeIdentifier>,
    pub content_captured_by_the_snapshots: Option<Vec<u8>>,
    pub has_a_live_compaction_intent: bool,
    pub relocation_sources_up_to_this_version: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 一条录好的流按崩溃点重放要的样子摆好：起点镜像（mkfs 之后）、写表与段、每次写在录制流里的下标、被判的根槽写（最后一条）、
/// 版本表（有文件的与文件从那一版起没有了的）、观察者另读的流里每一版。
pub struct PreparedCompactionCrashFlow {
    pub recorded: RecordedCompactionFlow,
    pub base: MemoryPool,
    pub operations_after_mkfs: Vec<RetainedOperation>,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub stream_indexes: Vec<usize>,
    pub judged_root_index: usize,
    pub versions_with_a_file: Vec<PublishedVersion>,
    pub versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    pub versions_of_the_flow: Vec<VersionOfTheCompactionFlow>,
}

/// 录一条流并摆好。版本表逐版现推：每一步的每一版都进表（有文件的带它那一刻第一个文件的内容），重挂、整理、删快照、抬 F、建 inode 不动文件内容。
/// 这里只摆、不核形状：段形、搬走的单元、discard 这些在用例里判器判完之后再核。
#[must_use]
pub fn prepare_the_compaction_crash_flow(
    tag: &str,
    flow: CompactionCrashFlow,
    contents_of_the_sequential_writes: &[Vec<u8>],
) -> PreparedCompactionCrashFlow {
    let recorded = record_the_compaction_crash_flow(tag, flow, contents_of_the_sequential_writes);
    let pool = &recorded.pool;
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let operations_after_mkfs = operations[pool.mkfs_operation_count..].to_vec();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations_after_mkfs, &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    let mut versions_with_a_file = Vec::new();
    let mut versions_without_a_file = Vec::new();
    let mut versions_of_the_flow = Vec::new();
    let mut first_file_content = Some(file_content());
    let mut content_captured_by_the_snapshots = None;
    let mut relocation_sources = Vec::new();
    let mut sequential_writes_seen = 0usize;
    let mut note_version =
        |kind: Option<CompactionFlowStepKind>,
         output: &TransactionOutput,
         content_of_this_version: &Option<Vec<u8>>,
         captured_content_of_this_version: &Option<Vec<u8>>,
         sources_of_this_version: &[(DeviceIdentity, SlotNumber)]| {
            match content_of_this_version {
                Some(content) => versions_with_a_file.push(PublishedVersion {
                    instance: output.root.instance,
                    checkpoint_txg: output.root.checkpoint_txg,
                    content: content.clone(),
                }),
                None => versions_without_a_file.push(PublishedVersionWithoutAFile {
                    instance: output.root.instance,
                    checkpoint_txg: output.root.checkpoint_txg,
                }),
            }
            let live_snapshots = live_snapshot_trees_of(output);
            versions_of_the_flow.push(VersionOfTheCompactionFlow {
                version: (output.root.instance, output.root.checkpoint_txg),
                kind,
                root: output.root,
                first_file_content: content_of_this_version.clone(),
                content_captured_by_the_snapshots: if live_snapshots.is_empty() {
                    None
                } else {
                    captured_content_of_this_version.clone()
                },
                live_snapshots,
                has_a_live_compaction_intent: !output.logged_operations.is_empty(),
                // 回收之后又分配出去的源落点（生命周期闭环流在搬出来的空段上再分配）这一版有人用，不抹
                relocation_sources_up_to_this_version: sources_of_this_version
                    .iter()
                    .filter(|(device, slot)| {
                        !output.allocation_records.iter().any(|record| {
                            record.device == *device
                                && !record.is_released
                                && record.slot.0 <= slot.0
                                && slot.0 < record.slot.0 + u64::from(record.span_slots)
                        })
                    })
                    .copied()
                    .collect(),
            });
        };
    note_version(
        None,
        &recorded.first_file,
        &first_file_content,
        &content_captured_by_the_snapshots,
        &relocation_sources,
    );
    for step in &recorded.steps {
        match step.kind {
            CompactionFlowStepKind::SequentialWrite => {
                first_file_content = Some(
                    contents_of_the_sequential_writes
                        .get(sequential_writes_seen)
                        .expect("每一次顺序写的内容都由调用方给了")
                        .clone(),
                );
                sequential_writes_seen += 1;
            }
            CompactionFlowStepKind::OverwriteBeforeTheSnapshot => {
                first_file_content = Some(content_overwritten_before_the_snapshot());
            }
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot) => {
                content_captured_by_the_snapshots = first_file_content.clone();
            }
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot) => {
                first_file_content = Some(overwrite_content());
            }
            CompactionFlowStepKind::Snapshot(
                SnapshotFlowStepKind::UnlinkIntent | SnapshotFlowStepKind::UnlinkFinalBatch,
            ) => {
                first_file_content = None;
            }
            CompactionFlowStepKind::Snapshot(
                SnapshotFlowStepKind::SnapshotDestructionIntent
                | SnapshotFlowStepKind::SnapshotDestructionBatch
                | SnapshotFlowStepKind::SnapshotDestructionFinalBatch,
            )
            | CompactionFlowStepKind::MountWritable
            | CompactionFlowStepKind::CompactionIssue
            | CompactionFlowStepKind::CompactionBatch
            | CompactionFlowStepKind::CompactionFinalBatch
            | CompactionFlowStepKind::DestroySnapshotThroughTheSession
            | CompactionFlowStepKind::RaiseFloorToTheAdmissionCeiling
            | CompactionFlowStepKind::NewInodesThroughTheSession => {}
        }
        for output in &step.versions {
            relocation_sources.extend(output.units_relocated_by_this_publish.iter().flat_map(
                |relocated| {
                    relocated
                        .source
                        .iter()
                        .map(|location| (location.device, location.slot))
                },
            ));
            note_version(
                Some(step.kind),
                output,
                &first_file_content,
                &content_captured_by_the_snapshots,
                &relocation_sources,
            );
        }
    }
    PreparedCompactionCrashFlow {
        recorded,
        base,
        operations_after_mkfs,
        writes,
        segments,
        stream_indexes,
        judged_root_index,
        versions_with_a_file,
        versions_without_a_file,
        versions_of_the_flow,
    }
}

impl PreparedCompactionCrashFlow {
    /// 观察者另读的输入的摘要（流里每一版，[`text_of_the_versions_of_the_compaction_flow`]）：进观察者的版本，录制流一变它就变。
    #[must_use]
    pub fn digest_of_the_versions_of_the_flow(
        &self,
    ) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
        singlefs_verification::sha256::sha256_digest(
            text_of_the_versions_of_the_compaction_flow(&self.versions_of_the_flow).as_bytes(),
        )
    }

    /// 每段的写数（用例核段形）。
    #[must_use]
    pub fn segment_sizes(&self) -> Vec<usize> {
        self.segments.iter().map(Vec::len).collect()
    }

    /// 写数小于 `writes_below` 的那几段。
    #[must_use]
    pub fn segments_shorter_than(&self, writes_below: usize) -> Vec<Vec<usize>> {
        self.segments
            .iter()
            .filter(|segment| segment.len() < writes_below)
            .cloned()
            .collect()
    }

    /// 录制流里的 discard（mkfs 之后的下标、盘、段起点）。
    #[must_use]
    pub fn discards_in_the_stream(&self) -> Vec<(usize, DeviceIdentity, SlotNumber)> {
        self.operations_after_mkfs
            .iter()
            .enumerate()
            .filter(|(_index, retained)| retained.operation.kind == RecordedOperationKind::Discard)
            .map(|(index, retained)| {
                (
                    index,
                    retained.operation.device,
                    SlotNumber(retained.operation.offset.0 / SLOT_BYTES),
                )
            })
            .collect()
    }
}

/// 观察者另读的输入写成文字（取摘要进观察者的版本）：只有逐版那几行，不带标签与手写的版本号（D13（验证路线） 已定项 17：
/// 手写的版本常量不进键）；录法变了，变的是逐版的内容，摘要跟着变。
#[must_use]
pub fn text_of_the_versions_of_the_compaction_flow(
    versions: &[VersionOfTheCompactionFlow],
) -> String {
    let mut text = String::new();
    for version in versions {
        text.push_str(&format!(
            "version instance={} txg={} kind={:?} first_file_bytes={:?} live_snapshots={:?} captured_bytes={:?} live_intent={} relocation_sources={:?}\n",
            version.version.0 .0,
            version.version.1 .0,
            version.kind,
            version.first_file_content.as_ref().map(|content| singlefs_verification::fnv1a_64(content)),
            version.live_snapshots,
            version
                .content_captured_by_the_snapshots
                .as_ref()
                .map(|content| singlefs_verification::fnv1a_64(content)),
            version.has_a_live_compaction_intent,
            version.relocation_sources_up_to_this_version
        ));
    }
    text
}

/// 恢复落到的根在流里是哪一版。
#[derive(Clone, Copy, Debug)]
pub enum LandedVersionOfTheCompactionFlow<'flow> {
    /// 落到 A 之前（mkfs 或暖机那几条根）：根下面没有树。
    BeforeTheFirstFile,
    Version(&'flow VersionOfTheCompactionFlow),
}

/// 恢复落到的根（实例，txg）在流里是哪一版：不是流里任何一版的根都在 A 之前（流里每一版的根都在表里）。
#[must_use]
pub fn version_of_the_compaction_flow_landed_on(
    versions: &[VersionOfTheCompactionFlow],
    effective_root: (InstanceGeneration, CheckpointTxg),
) -> LandedVersionOfTheCompactionFlow<'_> {
    match versions
        .iter()
        .find(|version| version.version == effective_root)
    {
        Some(version) => LandedVersionOfTheCompactionFlow::Version(version),
        None => {
            let version_of_the_first_file = versions.first().expect("流里至少 A 一版");
            assert!(
                (effective_root.1, effective_root.0)
                    < (
                        version_of_the_first_file.version.1,
                        version_of_the_first_file.version.0,
                    ),
                "恢复落到的根 {effective_root:?} 不是流里任何一版，又不在 A 之前"
            );
            LandedVersionOfTheCompactionFlow::BeforeTheFirstFile
        }
    }
}

/// 一张镜像的副本上把这些源落点（两块盘各自那一槽）整槽抹成 0：整理搬走之后源落点进了 defer、不再被任何一版引用，抹掉它不该改任何读出来的内容。
#[must_use]
pub fn image_with_the_relocation_sources_zeroed(
    image: &MemoryPool,
    sources: &[(DeviceIdentity, SlotNumber)],
) -> MemoryPool {
    let mut zeroed = image.clone();
    let operations: Vec<RetainedOperation> = sources
        .iter()
        .map(|(device, slot)| RetainedOperation {
            operation: RecordedOperation {
                device: *device,
                kind: RecordedOperationKind::WriteZeroes,
                offset: singlefs_core::address::DeviceOffsetInBytes(slot.0 * SLOT_BYTES),
                length: SLOT_BYTES,
                content_hash: singlefs_verification::fnv1a_64_of_zeros(SLOT_BYTES),
            },
            contents: None,
        })
        .collect();
    zeroed.apply(&operations);
    zeroed
}

/// 在 `reader` 上读这一版的每个活快照：读出的要是建它那一刻第一个文件的内容。交回读了几个。
///
/// # Panics
/// 有一个活快照读不出、读成没有文件或读出别的内容。
pub fn assert_every_live_snapshot_of_the_version_reads_its_creation_content(
    reader: &dyn PoolReader,
    version: &VersionOfTheCompactionFlow,
    what: &str,
) -> usize {
    for snapshot in &version.live_snapshots {
        let expected = version
            .content_captured_by_the_snapshots
            .as_ref()
            .expect("有活快照的一版记着快照建的那一刻的内容");
        match read_the_first_file_of_a_snapshot(reader, &version.root, *snapshot) {
            Ok(SnapshotFileReading::File(content)) => assert!(
                &content == expected,
                "落到 {:?}（{:?}），{what}：活快照 {snapshot:?} 读出建它那一刻的内容（{} 字节），实际 {} 字节",
                version.version,
                version.kind,
                expected.len(),
                content.len()
            ),
            other => panic!(
                "落到 {:?}（{:?}），{what}：活快照 {snapshot:?} 要读出建它那一刻的文件，实际 {other:?}",
                version.version, version.kind
            ),
        }
    }
    version.live_snapshots.len()
}

/// 在 `reader` 上冷走读这一版的第一个文件：有文件的一版读出它那一刻的内容，没有文件的一版答没有文件。
///
/// # Panics
/// 读不出，或读出的与这一版不同。
pub fn assert_the_first_file_of_the_version_reads_its_content(
    reader: &dyn PoolReader,
    version: &VersionOfTheCompactionFlow,
    what: &str,
) {
    let mut mapping_fallbacks = 0usize;
    let read = walk_to_file(reader, &version.root, &mut mapping_fallbacks);
    assert!(
        read.as_ref()
            .is_ok_and(|content| content == &version.first_file_content),
        "落到 {:?}（{:?}），{what}：第一个文件读出这一版的内容（{:?} 字节），实际 {:?}",
        version.version,
        version.kind,
        version.first_file_content.as_ref().map(Vec::len),
        read.map(|content| content.map(|bytes| bytes.len()))
    );
}

// ============================== 在崩溃镜像上可写挂载一次、续做到底 ==============================

/// 在一张崩溃镜像落成的盘上可写挂载一次之后：挂载交回的东西、挂载发的每一次写（落在崩溃镜像上，按发的次序）、挂完的盘。
pub struct MountedWithTheCompactionSwitchesAfterTheCrash {
    pub mounted: Mounted,
    pub writes_of_the_mount: Vec<RetainedWrite>,
    pub disks_after_the_mount: MemoryPool,
    /// 挂载读过的每一段（连同 journal 记录槽提示问到的段，按发出次序）：交回镜像的观察者按它算读集见证。
    pub read_ranges_of_the_mount: Vec<singlefs_verification::RecordedReadRange>,
}

/// 在 `disks`（一张崩溃镜像落成的池）的副本上可写挂载一次，整理开关同录流（[`compaction_switches_of_the_crash_flows`]）：删除续做把队列走到空，
/// 整理续做把盘上那一条未完成的意图推到删了为止。挂载发的写录在一条新的录制流上，连同挂完的盘交回。
///
/// # Panics
/// 可写挂载做不成：三条流的每个崩溃状态按条款都挂得上。
#[must_use]
pub fn mount_writable_with_the_compaction_switches_on_the_crash_disks(
    disks: &MemoryPool,
) -> MountedWithTheCompactionSwitchesAfterTheCrash {
    let mount_parameters = parameters();
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = disks
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                disks.device_size_in_bytes,
                PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let mounted = mount_writable_with_compaction_switches(
        &mount_parameters,
        &mut devices,
        compaction_switches_of_the_crash_flows(),
    )
    .unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));
    let disks_after_the_mount = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: disks.device_size_in_bytes,
    };
    let (writes_of_the_mount, _segments_of_the_mount) =
        writes_and_segments(&stream.retained_operations(), &geometry());
    MountedWithTheCompactionSwitchesAfterTheCrash {
        mounted,
        writes_of_the_mount,
        disks_after_the_mount,
        read_ranges_of_the_mount: stream.read_ranges(),
    }
}

/// 可写挂载之后核续做（D8（核心索引结构） 已定项 5「续做是每次恢复都跑的常规动作」）：删除续做把队列走到空；恢复落到的那一版有活的整理意图时，
/// 整理续做推到意图删了为止（停在「没有活意图可续」，最后那一版 logged_ops 空、R 按记录全空），没有时一次都不发；挂完的那一版 logged_ops 空、
/// 删除队列与销毁中的快照都空。挂完的盘上第一个文件读出恢复落到的那一版的内容（续做不改文件内容），活快照读出建它那一刻的内容。
/// 交回整理续做发了几次。
///
/// # Panics
/// 上面任何一样对不上。
pub fn assert_the_mount_continued_the_compaction_and_the_deletions(
    landed: LandedVersionOfTheCompactionFlow<'_>,
    mounted: &MountedWithTheCompactionSwitchesAfterTheCrash,
) -> usize {
    let output = &mounted.mounted.output;
    assert!(
        matches!(
            output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "删除续做把队列走到空：{:?}",
        output.deletion_queue_continuation_outcome
    );
    let compaction_publishes = output
        .compaction_continuation
        .run
        .as_ref()
        .map_or(0, |run| run.versions_published.len());
    let LandedVersionOfTheCompactionFlow::Version(version) = landed else {
        assert_eq!(compaction_publishes, 0, "落到 A 之前：没有要续做的整理");
        assert!(
            mounted.mounted.current.version_of_trees().is_none(),
            "落到 A 之前：挂完照旧没有树"
        );
        return 0;
    };
    let current = mounted
        .mounted
        .current
        .version_of_trees()
        .expect("落到 A 起的一版：挂完有树");
    if version.has_a_live_compaction_intent {
        let run = output
            .compaction_continuation
            .run
            .as_ref()
            .expect("落到的一版有活的整理意图：挂载续做它");
        assert!(
            matches!(
                run.outcome,
                CompactionRunOutcome::Stopped(CompactionRunStop::NoLiveIntentLeftToContinue)
            ),
            "落到 {:?}（{:?}）：整理续做推到意图删了：{:?}",
            version.version,
            version.kind,
            run.outcome
        );
        assert!(
            run.versions_published
                .last()
                .is_some_and(|last| last.logged_operations.is_empty()
                    && !last.units_relocated_by_this_publish.is_empty()),
            "落到 {:?}（{:?}）：整理续做的最后一次是最后一批、同一次发布删意图",
            version.version,
            version.kind
        );
        assert!(
            region_is_empty_by_the_records(
                &current.allocation_records,
                region_of_the_crash_flows()
            ),
            "落到 {:?}（{:?}）：续做之后 R 按记录全空（停机谓词，判决 G4）",
            version.version,
            version.kind
        );
    } else {
        assert_eq!(
            compaction_publishes, 0,
            "落到 {:?}（{:?}）：没有活的整理意图，挂载一次整理都不发（不签新的）",
            version.version, version.kind
        );
    }
    assert!(
        current.logged_operations.is_empty(),
        "落到 {:?}（{:?}）：挂完 logged_ops 空",
        version.version,
        version.kind
    );
    assert!(
        current.deleted_inodes.is_empty() && current.snapshots_being_destroyed.is_empty(),
        "落到 {:?}（{:?}）：挂完删除队列与销毁中的快照都空",
        version.version,
        version.kind
    );
    let after_the_mount = VersionOfTheCompactionFlow {
        version: (current.root.instance, current.root.checkpoint_txg),
        kind: version.kind,
        root: current.root,
        first_file_content: version.first_file_content.clone(),
        live_snapshots: live_snapshot_trees_of(current),
        content_captured_by_the_snapshots: version.content_captured_by_the_snapshots.clone(),
        has_a_live_compaction_intent: false,
        relocation_sources_up_to_this_version: Vec::new(),
    };
    assert_the_first_file_of_the_version_reads_its_content(
        &mounted.disks_after_the_mount,
        &after_the_mount,
        "可写挂载续做之后",
    );
    assert_every_live_snapshot_of_the_version_reads_its_creation_content(
        &mounted.disks_after_the_mount,
        &after_the_mount,
        "可写挂载续做之后",
    );
    compaction_publishes
}

// ============================== 整理搬只被快照引用的单元、deadlist 跨两代实例记与清、回收的流 ==============================
//
// 生命周期闭环流那一串，多两样：建快照之前那次覆盖写连带建空 inode，把 inode 树填满几个叶容器（这几个容器、inode 根与 extent 根都落在 R 里）；
// 快照下除了覆盖写第一个文件，再逐个容器改一条 inode 记录（给那个 inode 写一个数据单元的内容），把快照抄下的每个容器都换下来。
// 换下来的这些单元只被快照引用，记进 deadlist（实例 1），整理把它们搬出 R（实例 2，几批加最后一批），删快照由实例 2 清空 deadlist。
// 流头（mkfs 到建快照）与整理、deadlist 无关，崩溃点重放的基镜像取在它之后、不枚举；崩溃放量那一路的基镜像照旧取在 mkfs 之后
// （标准前缀上的观察者要有状态可看）。

use singlefs_core::address::InodeNumber;
use singlefs_core::transaction::{
    publish_data_changes_published_together, publish_overwrite_of_an_inode,
    DataChangesPublishedTogether, NewInodesOfOneChange, PoolWriter, WholeWriteOfOneInode,
    FIRST_INODE_NUMBER,
};
use singlefs_format::INODE_LEAF_RECORDS;

/// 快照抄下的 inode 树有几个叶容器（一个容器装 [`INODE_LEAF_RECORDS`] 条记录）。
pub const INODE_LEAF_CONTAINERS_CAPTURED_BY_THE_SNAPSHOT: u64 = 6;

/// 建快照之前那次覆盖写连带建几个空 inode：连同第一个文件恰好填满 [`INODE_LEAF_CONTAINERS_CAPTURED_BY_THE_SNAPSHOT`] 个容器。
pub const INODES_CREATED_WITH_THE_OVERWRITE_BEFORE_THE_SNAPSHOT: u64 =
    INODE_LEAF_CONTAINERS_CAPTURED_BY_THE_SNAPSHOT * INODE_LEAF_RECORDS - 1;

/// 快照下改别的 inode 那几次的调用方时间比第一个文件晚这么多秒（再逐次加一）：与这条流里别的写都不同。
pub const SECONDS_FROM_THE_FIRST_WRITE_TO_THE_WRITES_OF_THE_OTHER_INODES: u64 = 300;

/// 快照下改的那个 inode：第 `container_index` 个叶容器里的第一个号（inode 号从第一个文件起连号，一个容器装满再开下一个）。
#[must_use]
pub fn inode_in_the_leaf_container(container_index: u64) -> InodeNumber {
    InodeNumber(FIRST_INODE_NUMBER + container_index * INODE_LEAF_RECORDS)
}

/// 快照下写给第 `container_index` 个叶容器里那个 inode 的内容：一个数据单元装得下、各不相同。
#[must_use]
pub fn content_written_under_the_snapshot_to_the_inode_in_the_leaf_container(
    container_index: u64,
) -> Vec<u8> {
    let seed = usize::try_from(container_index).expect("容器序号装得进 usize");
    (0..1500_usize)
        .map(|index| u8::try_from((index * 5 + seed * 17 + 1) % 233).expect("小于 256"))
        .collect()
}

/// 录好的这条流与流头有几步（建快照之前那次覆盖写、建快照；崩溃点重放的基镜像取在它们之后）。
pub struct RecordedCompactionOfSnapshotUnitsFlow {
    pub recorded: RecordedCompactionFlow,
    pub steps_of_the_stream_head: usize,
}

/// 录这条流（`tag` 是镜像文件名里那一段）：
/// A → 覆盖写连带建空 inode → 建快照（以上是流头）→ 快照下覆盖写第一个文件 → 快照下逐个容器给一个 inode 写一个单元 → unlink 意图、最后一批
/// → 重挂 → 整理签发、几批、最后一批 → 会话里删快照 → 回收（抬 F、建 inode，直到 R 整段回收）→ 在搬出来的空段上再分配。
/// `flow` 记成生命周期闭环流（同一串，只多了建 inode 与快照下逐个容器那几次写）。
///
/// # Panics
/// 任何一步发布被拒、整理停不下来、回收或再分配在上限之内做不成。
#[must_use]
pub fn record_the_compaction_of_snapshot_units_flow(
    tag: &str,
) -> RecordedCompactionOfSnapshotUnitsFlow {
    let mut pool = build_pool(tag);
    let first_file = pool.output.clone();
    let mut steps = Vec::new();
    let write_time_of_the_overwrite =
        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE;
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::OverwriteBeforeTheSnapshot,
        |pool| {
            let previous = pool.output.clone();
            let content = content_overwritten_before_the_snapshot();
            let publish_parameters = parameters();
            let devices = pool.devices.as_mut().expect("镜像还开着");
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_data_changes_published_together(
                &mut writer,
                &mut pool.allocator,
                &previous,
                DataChangesPublishedTogether {
                    whole_write: Some(WholeWriteOfOneInode {
                        inode: InodeNumber(FIRST_INODE_NUMBER),
                        content: &content,
                        write_time_seconds: write_time_of_the_overwrite,
                    }),
                    new_inodes: &[NewInodesOfOneChange {
                        count: INODES_CREATED_WITH_THE_OVERWRITE_BEFORE_THE_SNAPSHOT,
                        write_time_seconds: write_time_of_the_overwrite,
                    }],
                },
                INSTANCE_OF_THE_FLOWS,
            )
            .expect("建快照之前覆盖写第一个文件、连带建空 inode")
        },
    ));
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot),
        create_snapshot,
    ));
    let snapshot = live_snapshot_trees_of(&steps.last().expect("刚发布快照").versions[0])
        .last()
        .copied();
    let steps_of_the_stream_head = steps.len();
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
        overwrite_under_the_snapshot,
    ));
    for container_index in 1..INODE_LEAF_CONTAINERS_CAPTURED_BY_THE_SNAPSHOT {
        steps.push(record_one_publish_of_instance_one(
            &mut pool,
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
            |pool| {
                let previous = pool.output.clone();
                let content = content_written_under_the_snapshot_to_the_inode_in_the_leaf_container(
                    container_index,
                );
                let publish_parameters = parameters();
                let devices = pool.devices.as_mut().expect("镜像还开着");
                let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
                publish_overwrite_of_an_inode(
                    &mut writer,
                    &mut pool.allocator,
                    &previous,
                    inode_in_the_leaf_container(container_index),
                    &content,
                    FIXED_WRITE_TIME_SECONDS
                        + SECONDS_FROM_THE_FIRST_WRITE_TO_THE_WRITES_OF_THE_OTHER_INODES
                        + container_index,
                    INSTANCE_OF_THE_FLOWS,
                )
                .unwrap_or_else(|refusal| {
                    panic!("快照下给第 {container_index} 个容器里的 inode 写一个单元：{refusal:?}")
                })
            },
        ));
    }
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkIntent),
        unlink_intent,
    ));
    steps.push(record_one_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkFinalBatch),
        unlink_step,
    ));
    assert!(
        pool.output.deleted_inodes.is_empty(),
        "一个数据单元的文件：意图之后一批就删完"
    );
    let (mount_step, mut session) = remount_and_open_the_session(&mut pool);
    steps.push(mount_step);
    steps.extend(publish_the_compaction_until_the_intent_is_deleted(
        &mut pool,
        &mut session,
    ));
    steps.push(publish_a_user_change_through_the_session(
        &mut pool,
        &mut session,
        CompactionFlowStepKind::DestroySnapshotThroughTheSession,
        UserChange::DestroySnapshot {
            snapshot: snapshot.expect("流头建过快照"),
        },
    ));
    record_the_reclaim_and_the_reallocation_of_the_snapshot_units_flow(
        &mut pool,
        &mut session,
        &mut steps,
    );
    pool.output = version_of_trees(&session.current).clone();
    RecordedCompactionOfSnapshotUnitsFlow {
        recorded: RecordedCompactionFlow {
            flow: CompactionCrashFlow::LifecycleClosure,
            pool,
            first_file,
            steps,
            snapshot,
        },
        steps_of_the_stream_head,
    }
}

/// 这条流「回收」那一段至多几轮（抬 F 到准入上限、建一次 inode）。
pub const SNAPSHOT_UNITS_FLOW_RECLAIM_ROUNDS_AT_MOST: usize = 24;

/// 这条流「再分配」那一段至多建几次 inode。
pub const SNAPSHOT_UNITS_FLOW_REALLOCATION_PUBLISHES_AT_MOST: usize = 24;

/// R 两块盘上罩着它的每条分配记录（盘、槽、跨度、代、释放没有）：录流做不成时报出来。
fn allocation_records_over_the_region(
    version: &TransactionOutput,
) -> Vec<(DeviceIdentity, u64, u16, u64, bool)> {
    let region_end = REGION_START_SLOT_OF_THE_CRASH_FLOWS + 64;
    version
        .allocation_records
        .iter()
        .filter(|record| {
            record.slot.0 < region_end
                && REGION_START_SLOT_OF_THE_CRASH_FLOWS
                    < record.slot.0 + u64::from(record.span_slots)
        })
        .map(|record| {
            (
                record.device,
                record.slot.0,
                record.span_slots,
                record.generation.0,
                record.is_released,
            )
        })
        .collect()
}

/// 这条流的「回收」与「在搬出来的空段上再分配」：同 [`record_the_reclaim_and_the_reallocation`]，上限按这条流取
/// （[`SNAPSHOT_UNITS_FLOW_RECLAIM_ROUNDS_AT_MOST`]、[`SNAPSHOT_UNITS_FLOW_REALLOCATION_PUBLISHES_AT_MOST`]），做不成时报出 R 上的记录。
fn record_the_reclaim_and_the_reallocation_of_the_snapshot_units_flow(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    steps: &mut Vec<CompactionFlowStep>,
) {
    let mut new_inodes_published = 0_u64;
    let mut new_inodes_step =
        |pool_of_the_flow: &mut BuiltPool, session_of_the_flow: &mut MountedSession| {
            new_inodes_published += 1;
            publish_a_user_change_through_the_session(
                pool_of_the_flow,
                session_of_the_flow,
                CompactionFlowStepKind::NewInodesThroughTheSession,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS
                        + LIFECYCLE_SECONDS_FROM_THE_FIRST_WRITE_TO_THE_NEW_INODES
                        + new_inodes_published,
                },
            )
        };
    let mut is_the_region_discarded = false;
    let mut discards_of_each_round = Vec::new();
    for _round_index in 0..SNAPSHOT_UNITS_FLOW_RECLAIM_ROUNDS_AT_MOST {
        if let Some(raised) = raise_the_floor_to_the_admission_ceiling(pool, session) {
            steps.push(raised);
        }
        let step = new_inodes_step(pool, session);
        is_the_region_discarded = the_step_discards_the_region_on_both_devices(&step);
        discards_of_each_round.push(step.discards.clone());
        steps.push(step);
        if is_the_region_discarded {
            break;
        }
    }
    let current = version_of_trees(&session.current).clone();
    assert!(
        is_the_region_discarded,
        "{SNAPSHOT_UNITS_FLOW_RECLAIM_ROUNDS_AT_MOST} 轮抬 F 之内 R 整段回收、会话对它发了 discard：F {:?}，R 上的记录 {:?}，每轮的 discard {:?}",
        current.root.rollback_floor,
        allocation_records_over_the_region(&current),
        discards_of_each_round
    );
    for _publish_index in 0..SNAPSHOT_UNITS_FLOW_REALLOCATION_PUBLISHES_AT_MOST {
        let step = new_inodes_step(pool, session);
        let is_reallocated = step
            .versions
            .last()
            .is_some_and(region_holds_an_unreleased_record);
        steps.push(step);
        if is_reallocated {
            return;
        }
    }
    panic!("建 {SNAPSHOT_UNITS_FLOW_REALLOCATION_PUBLISHES_AT_MOST} 次 inode 之内分配到 R 上");
}

/// 这条流的基镜像取在哪。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseImageOfTheCompactionOfSnapshotUnitsFlow {
    /// mkfs 之后：崩溃放量那一路（标准前缀上的观察者要有状态可看）。
    AfterTheMakeFilesystem,
    /// 流头（mkfs 到建快照）之后：崩溃点重放那一路，流头不枚举。
    AfterTheStreamHead,
}

/// 录这条流并摆好（同 [`prepare_the_compaction_crash_flow`]，基镜像按 `base_image` 取）。写表、段与被判的根槽写从基镜像之后数；
/// `stream_indexes` 照旧是 mkfs 之后录制流里的下标，`operations_after_mkfs` 照旧是 mkfs 之后整条录制流。
/// 版本表与流里每一版收全（流头那几版也在：基镜像里的版本恢复落得到的只有最后一版，收全不改判法）。
#[must_use]
pub fn prepare_the_compaction_of_snapshot_units_flow(
    tag: &str,
    base_image: BaseImageOfTheCompactionOfSnapshotUnitsFlow,
) -> PreparedCompactionCrashFlow {
    let RecordedCompactionOfSnapshotUnitsFlow {
        recorded,
        steps_of_the_stream_head,
    } = record_the_compaction_of_snapshot_units_flow(tag);
    let pool = &recorded.pool;
    let operations = pool.retained_operations();
    let mkfs_operations = pool.mkfs_operation_count;
    let operations_in_the_base_image = match base_image {
        BaseImageOfTheCompactionOfSnapshotUnitsFlow::AfterTheMakeFilesystem => mkfs_operations,
        BaseImageOfTheCompactionOfSnapshotUnitsFlow::AfterTheStreamHead => {
            recorded.steps[steps_of_the_stream_head - 1].operations.end
        }
    };
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&operations[mkfs_operations..operations_in_the_base_image]);
    let operations_after_mkfs = operations[mkfs_operations..].to_vec();
    let (writes, segments, stream_indexes_after_the_base_image) =
        writes_and_segments_with_stream_indexes(
            &operations[operations_in_the_base_image..],
            &geometry(),
        );
    let stream_indexes = stream_indexes_after_the_base_image
        .iter()
        .map(|index| index + (operations_in_the_base_image - mkfs_operations))
        .collect();
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    let mut versions_with_a_file = Vec::new();
    let mut versions_without_a_file = Vec::new();
    let mut versions_of_the_flow = Vec::new();
    let mut first_file_content = Some(file_content());
    let mut content_captured_by_the_snapshots = None;
    let mut relocation_sources = Vec::new();
    let mut note_version =
        |kind: Option<CompactionFlowStepKind>,
         output: &TransactionOutput,
         content_of_this_version: &Option<Vec<u8>>,
         captured_content_of_this_version: &Option<Vec<u8>>,
         sources_of_this_version: &[(DeviceIdentity, SlotNumber)]| {
            match content_of_this_version {
                Some(content) => versions_with_a_file.push(PublishedVersion {
                    instance: output.root.instance,
                    checkpoint_txg: output.root.checkpoint_txg,
                    content: content.clone(),
                }),
                None => versions_without_a_file.push(PublishedVersionWithoutAFile {
                    instance: output.root.instance,
                    checkpoint_txg: output.root.checkpoint_txg,
                }),
            }
            let live_snapshots = live_snapshot_trees_of(output);
            versions_of_the_flow.push(VersionOfTheCompactionFlow {
                version: (output.root.instance, output.root.checkpoint_txg),
                kind,
                root: output.root,
                first_file_content: content_of_this_version.clone(),
                content_captured_by_the_snapshots: if live_snapshots.is_empty() {
                    None
                } else {
                    captured_content_of_this_version.clone()
                },
                live_snapshots,
                has_a_live_compaction_intent: !output.logged_operations.is_empty(),
                // 回收之后又分配出去的源落点这一版有人用，不抹（同 `prepare_the_compaction_crash_flow`）
                relocation_sources_up_to_this_version: sources_of_this_version
                    .iter()
                    .filter(|(device, slot)| {
                        !output.allocation_records.iter().any(|record| {
                            record.device == *device
                                && !record.is_released
                                && record.slot.0 <= slot.0
                                && slot.0 < record.slot.0 + u64::from(record.span_slots)
                        })
                    })
                    .copied()
                    .collect(),
            });
        };
    note_version(
        None,
        &recorded.first_file,
        &first_file_content,
        &content_captured_by_the_snapshots,
        &relocation_sources,
    );
    for step in &recorded.steps {
        match step.kind {
            CompactionFlowStepKind::OverwriteBeforeTheSnapshot => {
                first_file_content = Some(content_overwritten_before_the_snapshot());
            }
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot) => {
                content_captured_by_the_snapshots = first_file_content.clone();
            }
            // 快照下给别的 inode 写的那几次也记成这一种：第一个文件照旧是快照下覆盖写那一份内容
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot) => {
                first_file_content = Some(overwrite_content());
            }
            CompactionFlowStepKind::Snapshot(
                SnapshotFlowStepKind::UnlinkIntent | SnapshotFlowStepKind::UnlinkFinalBatch,
            ) => {
                first_file_content = None;
            }
            CompactionFlowStepKind::SequentialWrite => {
                panic!("这条流没有顺序写：{:?}", step.kind)
            }
            CompactionFlowStepKind::Snapshot(
                SnapshotFlowStepKind::SnapshotDestructionIntent
                | SnapshotFlowStepKind::SnapshotDestructionBatch
                | SnapshotFlowStepKind::SnapshotDestructionFinalBatch,
            )
            | CompactionFlowStepKind::MountWritable
            | CompactionFlowStepKind::CompactionIssue
            | CompactionFlowStepKind::CompactionBatch
            | CompactionFlowStepKind::CompactionFinalBatch
            | CompactionFlowStepKind::DestroySnapshotThroughTheSession
            | CompactionFlowStepKind::RaiseFloorToTheAdmissionCeiling
            | CompactionFlowStepKind::NewInodesThroughTheSession => {}
        }
        for output in &step.versions {
            relocation_sources.extend(output.units_relocated_by_this_publish.iter().flat_map(
                |relocated| {
                    relocated
                        .source
                        .iter()
                        .map(|location| (location.device, location.slot))
                },
            ));
            note_version(
                Some(step.kind),
                output,
                &first_file_content,
                &content_captured_by_the_snapshots,
                &relocation_sources,
            );
        }
    }
    PreparedCompactionCrashFlow {
        recorded,
        base,
        operations_after_mkfs,
        writes,
        segments,
        stream_indexes,
        judged_root_index,
        versions_with_a_file,
        versions_without_a_file,
        versions_of_the_flow,
    }
}

impl PreparedCompactionCrashFlow {
    /// 每一段归流里哪一步（段第一个写在录制流里的下标落在那一步发出的那段录制流里）：`None` 是 A 与 A 之前那几段。
    ///
    /// # Panics
    /// 有一段一个写都没有。
    #[must_use]
    pub fn step_position_of_each_segment(&self) -> Vec<Option<usize>> {
        let mkfs_operations = self.recorded.pool.mkfs_operation_count;
        self.segments
            .iter()
            .map(|segment| {
                let write_opening_the_segment = *segment.first().expect("段至少一个写");
                let operation_in_the_stream =
                    mkfs_operations + self.stream_indexes[write_opening_the_segment];
                self.recorded
                    .steps
                    .iter()
                    .position(|step| step.operations.contains(&operation_in_the_stream))
            })
            .collect()
    }

    /// 崩溃状态落得到的流里那几版：根槽写在写表里的那几版，连同排在它们头一版之前的那一版（基镜像里最后那一版；基镜像取在 mkfs 之后时
    /// 写表里头一版就是 A，前面没有）。再早的几版在基镜像里，崩溃状态择不到。
    ///
    /// # Panics
    /// 写表里一条流里那几版的根槽写都没有。
    #[must_use]
    pub fn versions_landed_on_from_the_base_image(&self) -> Vec<&VersionOfTheCompactionFlow> {
        let roots_written_after_the_base_image: Vec<(InstanceGeneration, CheckpointTxg)> = self
            .writes
            .iter()
            .filter(|write| write.kind == StepKind::RootRecordFua)
            .map(singlefs_verification::memory_pool::root_identity_of_write)
            .collect();
        let earliest_version_written_after_the_base_image = self
            .versions_of_the_flow
            .iter()
            .position(|version| roots_written_after_the_base_image.contains(&version.version))
            .expect("写表里至少有流里一版的根槽写");
        self.versions_of_the_flow[earliest_version_written_after_the_base_image.saturating_sub(1)..]
            .iter()
            .collect()
    }
}
