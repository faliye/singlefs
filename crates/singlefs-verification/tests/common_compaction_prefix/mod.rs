//! 整理（自建 GC）两条崩溃流与生命周期闭环流共用的搭建（里程碑四二·3、三·1；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G6）：
//! 公共前缀是 `common::build_pool` 那一段（mkfs → 取号 → 暖机 → 新池新建文件 A，第一个文件一个数据单元），之后按流各接几步：
//! - 整理搬迁流（G6 前缀 a）：A → B、C 两次顺序写（多记录流那两次，内容由调用方给，与 `common_crash_points` 的那两份同一份字节）
//!   → 重挂（可写挂载：取号、写行、暖机）→ 整理签发 → 一批 → 最后一批（同一次发布删意图）；
//! - 活意图期间写类改动流：A → B、C → 实例 1 里建空 inode 填满 [`INODE_LEAF_CONTAINERS_LEFT_IN_THE_REGION_BEHIND_THE_WRITE`] 个叶容器 → 重挂 → 整理签发 → 一批
//!   → 意图活着时覆盖写第一个文件（放掉 R 里第一片叶容器与 inode 根的落点，别的叶容器留在 R 里）→ 一批 → 最后一批（对 R 重扫，搬留下的叶容器）；
//! - 暂停删意图流：前缀到第一批同整理搬迁流 →（在会话的分配器上压准入余量）最后一批被准入拒、暂停，单发一次只删意图的发布
//!   →（解开余量）在同一个 R 上重新签发 → 最后一批（同一次发布删意图）；
//! - 只剩 R 有空槽时暂停删意图流：前缀到第一批同整理搬迁流 →（在会话的分配器上把 R 之外的每一个空槽隔离）最后一批在 R 之外取不到落点、暂停，
//!   单发一次只删意图的发布，它取落点时放开 R、新落点落在 R 里 →（解开隔离）流到这里为止；
//! - 出错停下删意图流：前缀到第一批同整理搬迁流 →（只供测试的钩子在下一批读完设置之后注入一次算着出错）那一批放弃、单发一次只删意图的发布、
//!   整理停下 →（选 `cpu` 的切换解除停下，不写盘）在同一个 R 上重新签发 → 最后一批（同一次发布删意图）；
//! - 搬被快照共享的块流（二·3「搬被共享的块一条」）：A → 建快照 S → 快照下覆盖写 W（A 的四个单元进 deadlist、映射条目留存）→ 重挂 → 整理签发 → 几批 → 最后一批，
//!   R 里装着只被快照引用的那几个单元（留存的映射条目跟着搬到目的地）；
//! - 生命周期闭环流（三·1）：A（分配）→ 覆盖写 → 建快照 → 再覆盖写 → 删文件（unlink 意图、最后一批）→ 重挂 → 整理搬共享块（签发、几批、最后一批）
//!   → 删快照（会话里一次调用：意图、各批）→ 回收（抬 F 到准入上限、建 inode，直到 R 整段回收、会话对它发 discard）
//!   → 在搬出来的空段上再分配（建 inode，直到 R 里有一条未释放的记录）。
//!
//! R 取段 [`REGION_START_SLOT_OF_THE_CRASH_FLOWS`]：第一个文件那次发布的提交内生块开在这一段，重挂之后它不是本次挂载开过的段（判决 G6「选段资格要 R 不在
//! 本次挂载开过的段里」），这几条流都强制选它、一批 [`COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS`] 个（判决 G6：批大小 1–3，单元段写数 ≤ 30）。
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
use singlefs_core::address::InodeNumber;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::admission::{admission_reading_before_a_publish, SpaceAdmission};
use singlefs_core::background_compute::{
    BackgroundComputeSettings, BackgroundTaskAfterTheComputeError,
};
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
use singlefs_core::transaction::{
    publish_data_changes_published_together, publish_overwrite_of_an_inode,
    DataChangesPublishedTogether, NewInodesOfOneChange, PoolWriter, WholeWriteOfOneInode,
    FIRST_INODE_NUMBER,
};
use singlefs_core::transaction::{publish_new_inodes, FirstFile, PoolVersion, TransactionOutput};
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_format::SLOT_BYTES;
use singlefs_verification::memory_pool::{
    writes_and_segments, writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::mount_transparency::WritableMountRefusedOnTheCrashDisks;
use singlefs_verification::scenario::deletion_history_switches;
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RecordedOperation, RecordedOperationKind, RetainedOperation};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

mod compaction_of_snapshot_units_flow;
mod crash_flows_their_steps_and_the_region;
mod mounting_the_crash_images_to_continue_the_compaction;
mod preparing_the_compaction_crash_flow;
mod recording_the_compaction_crash_flows;

#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use compaction_of_snapshot_units_flow::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use crash_flows_their_steps_and_the_region::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use mounting_the_crash_images_to_continue_the_compaction::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use preparing_the_compaction_crash_flow::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use recording_the_compaction_crash_flows::*;

/// 这一份的仓内路径（与 `file!()` 归一之后相同）。
pub const FILE_OF_THE_COMPACTION_PREFIX: &str =
    "crates/singlefs-verification/tests/common_compaction_prefix/mod.rs";

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
    publish_the_compaction_writing_after_the_first_batch(
        pool,
        session,
        WriteWhileTheIntentIsLive::NoWrite,
    )
}

/// 整理一批发成之后，意图还活着时做不做一次写类用户改动。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteWhileTheIntentIsLive {
    NoWrite,
    /// 第一批发成之后覆盖写第一个文件（[`content_overwritten_while_the_intent_is_live`]）。
    OverwriteTheFirstFileAfterTheFirstBatch,
}

/// 同 [`publish_the_compaction_until_the_intent_is_deleted`]，第一批发成之后照 `write` 做一次写类用户改动（记成一步），下一批对 R 重扫。
fn publish_the_compaction_writing_after_the_first_batch(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    write: WriteWhileTheIntentIsLive,
) -> Vec<CompactionFlowStep> {
    let mut steps: Vec<CompactionFlowStep> = Vec::new();
    let mut is_the_write_still_to_do = match write {
        WriteWhileTheIntentIsLive::NoWrite => false,
        WriteWhileTheIntentIsLive::OverwriteTheFirstFileAfterTheFirstBatch => true,
    };
    for _publish_index in 0..COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST {
        let step = publish_the_compaction_once_as_a_step(pool, session, &steps);
        let kind = step.kind;
        steps.push(step);
        if kind == CompactionFlowStepKind::CompactionFinalBatch {
            return steps;
        }
        if kind == CompactionFlowStepKind::CompactionBatch && is_the_write_still_to_do {
            is_the_write_still_to_do = false;
            let content = content_overwritten_while_the_intent_is_live();
            steps.push(publish_a_user_change_through_the_session(
                pool,
                session,
                CompactionFlowStepKind::OverwriteWhileTheIntentIsLive,
                UserChange::Overwrite(FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS
                        + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
                }),
            ));
        }
    }
    panic!(
        "{COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST} 次整理发布之内最后一批没删意图：{:?}",
        steps.iter().map(|step| step.kind).collect::<Vec<_>>()
    );
}

/// 会话里整理一趟只发一次（发布预算压到 1），记成一步：签发、一批、最后一批按 logged_ops 与搬走的单元认，这一趟发布都返回之后会话发的 discard
/// 记在这一步上。`steps_so_far` 只进 panic 的说明。
///
/// # Panics
/// 这一趟没停在预算用完、没恰好发一次，或那一次既没写意图、也没搬单元（丢意图，或 R 已按记录全空只删意图）。
fn publish_the_compaction_once_as_a_step(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    steps_so_far: &[CompactionFlowStep],
) -> CompactionFlowStep {
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
            "整理一次发布既没写意图、也没搬单元（丢意图，或 R 已按记录全空只删意图）：这几条流的 R 里没有隔离槽、最后一批之前都还有要搬的单元，不该走到这里；\
             已录的整理步 {:?}，这一版 R 上的记录（盘、槽、跨度、代、释放没有）{:?}",
            steps_so_far.iter().map(|step| step.kind).collect::<Vec<_>>(),
            allocation_records_over_the_region(&published)
        ),
    };
    CompactionFlowStep {
        kind,
        operations: start..end,
        versions: vec![published],
        discards: run.discards.applied.clone(),
    }
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
