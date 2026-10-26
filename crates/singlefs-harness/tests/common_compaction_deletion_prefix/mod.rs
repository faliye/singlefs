//! 删除与整理交错、回退跨过整理意图的崩溃流的搭建（里程碑四二·3，欠账 C621 那一条流；条款：D8（核心索引结构） 已定项 16 第 6 条续做、
//! 判决 `research/prompts/m4-trunc-r1-main-verification.md` K1「回退时整理条目照旧随现行」、D23（journal 的角色与格式） 已定项 14 管理员回退）：
//! 公共前缀是 `common::build_pool` 那一段（mkfs → 取号 → 暖机 → 新池新建文件 A，第一个文件一个数据单元），之后：
//! 同一个进程（实例 1）里建一个空 inode（[`INODE_OF_THE_OTHER_FILE`]）→ 顺序写它（两个数据单元，池里有两个带数据的文件）
//! → 重挂（可写挂载：取号、写行、暖机）→ 整理签发（意图写进 logged_ops 树）→ 一批
//! → 会话里 unlink 那个文件（一次调用：意图、一批、最后一批；意图那次与那一批两版里删除意图与整理意图同时活着）
//! → 管理员回退到整理签发之前那一版（重挂最后那次暖机那一版：inode 树、extent 树与 `deleted_inodes` 取它的，logged_ops 随现行、整理意图跨回退留着）
//! → 整理接着推，一批一批直到最后一批同一次发布删意图。
//!
//! 建 inode 与顺序写放在实例 1 里，它们的提交内生块与第一个文件那次的落在同一个开放段 R 里（重挂之后 R 不是本次挂载开过的段，判决 G6 选段资格）；
//! 放在重挂之后的会话里，R 里能搬的单元都被那两次发布换下、R 没有资格（实测选段停在「没有合格的段」）。
//! 整理开关、R 与批大小同整理搬迁流（`common_compaction_prefix::compaction_switches_of_the_crash_flows`，R 是第一个文件那次发布的提交内生块所在的段）；
//! 删除开关同快照流（批大小 1，`scenario::deletion_history_switches`）。整理一趟只发一次（会话的发布预算压到 1），每一次整理发布记成流里一步。
//!
//! 这一份只录流、交回每一步发出的那段录制流与它的输出，再给观察者用的几样（流里每一版、读两个文件、在崩溃镜像上可写挂载一次续做到底）；
//! 只引 harness 档与实现，不引 checker 档：观察者的计数与判定在 checker 档的用例里。
#![allow(
    dead_code,
    reason = "共用的搭建模块：每个测试文件只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use std::ops::Range;

use super::common::{
    build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use super::common_compaction_prefix::{
    compaction_switches_of_the_crash_flows,
    mount_writable_with_the_compaction_switches_on_the_crash_disks, region_of_the_crash_flows,
    MountedWithTheCompactionSwitchesAfterTheCrash, COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::compaction_plan::{region_is_empty_by_the_records, CompactionRunStop};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    DeletionQueueContinuationOutcome, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession, UserChange};
use singlefs_core::recovery::{walk_to_file, PoolReader};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{
    publish_new_inodes, publish_sequential_write_of_an_inode, PoolWriter,
};
use singlefs_core::transaction::{PoolVersion, TransactionOutput};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite,
};
use singlefs_harness::scenario::deletion_history_switches;
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedOperationKind, RetainedOperation};

/// 这一份的仓内路径（与 `file!()` 归一之后相同）。
pub const THIS_FILE: &str =
    "crates/singlefs-harness/tests/common_compaction_deletion_prefix/mod.rs";

/// 实例 1 里建的那个 inode 的号：第一个文件是 inode 1，建一个空 inode 取下一个号。这条流 unlink 它、回退又把它带回来。
pub const INODE_OF_THE_OTHER_FILE: InodeNumber = InodeNumber(2);

/// 建 inode、顺序写与 unlink 的调用方时间比第一个文件晚这么多秒：与第一个文件的时间不同。
pub const SECONDS_FROM_THE_FIRST_WRITE_TO_THE_LATER_CHANGES: u64 = 120;

/// 会话里整理一趟只发一次：「已经过去的发布数」取得比 `3S − 4` 大，发布预算落到下限 1（`compaction_plan::publish_budget_of_a_compaction_run`）。
const PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE: u64 = 1_000;

/// 回退之后整理至多再发几次（各批与最后一批）：R 里一个段至多 64 个单元，批大小同整理搬迁流；录流时超了就是整理停不下来。
const COMPACTION_PUBLISHES_AFTER_THE_ROLLBACK_AT_MOST: usize =
    64 / COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS;

/// 那个文件的内容：两个数据单元（一个满的、一个半满的），与第一个文件的内容不同。
#[must_use]
pub fn content_of_the_other_file() -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + 5) % 239).expect("小于 256"))
        .collect()
}

/// 流里一步是哪一种（一次调用一步；会话里的 unlink 一次调用发意图与全部批，是一步几版）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletionCompactionFlowStepKind {
    /// 实例 1 里建一个空 inode。
    NewInodeInProcess,
    /// 实例 1 里顺序写那个文件（两个数据单元）。
    SequentialWriteOfTheOtherFileInProcess,
    /// 重开镜像、可写挂载（取号、写行、暖机）。
    MountWritable,
    /// 整理签发：一次只把意图写进 logged_ops 树的发布。
    CompactionIssue,
    /// 整理的一批（意图还在）。
    CompactionBatch,
    /// 会话里 unlink 那个文件：意图那次发布与全部批。
    UnlinkOfTheOtherFileThroughTheSession,
    /// 管理员回退到整理签发之前那一版。
    RollbackToTheVersionBeforeTheIssue,
    /// 整理的最后一批：同一次发布删意图。
    CompactionFinalBatch,
}

/// 流里一版是哪一种发布（观察者按恢复落到哪一种的那一版分别计数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishOfTheDeletionCompactionFlow {
    /// 新池新建文件 A。
    FirstFile,
    /// 实例 1 里建空 inode。
    NewInode,
    /// 实例 1 里顺序写那个文件。
    SequentialWriteOfTheOtherFile,
    /// 重挂那一步的写行与暖机。
    RowOrWarmUpOfTheMount,
    /// 整理签发。
    CompactionIssue,
    /// 回退之前整理的一批。
    CompactionBatchBeforeTheRollback,
    /// unlink 的意图那次发布（删除意图与整理意图同时活着）。
    UnlinkIntent,
    /// unlink 不是最后一批的那几批（两条意图同时活着）。
    UnlinkBatch,
    /// unlink 的最后一批（删除意图删了，整理意图还在）。
    UnlinkFinalBatch,
    /// 管理员回退那次发布。
    Rollback,
    /// 回退之后整理的一批（意图还在）。
    CompactionBatchAfterTheRollback,
    /// 回退之后整理的最后一批（同一次发布删意图）。
    CompactionFinalBatchAfterTheRollback,
}

impl PublishOfTheDeletionCompactionFlow {
    /// 计数的名字里这一种那一段。
    #[must_use]
    pub const fn counter_name(self) -> &'static str {
        match self {
            Self::FirstFile => "first_file",
            Self::NewInode => "new_inode",
            Self::SequentialWriteOfTheOtherFile => "sequential_write_of_the_other_file",
            Self::RowOrWarmUpOfTheMount => "row_or_warm_up_of_the_mount",
            Self::CompactionIssue => "compaction_issue",
            Self::CompactionBatchBeforeTheRollback => "compaction_batch_before_the_rollback",
            Self::UnlinkIntent => "unlink_intent",
            Self::UnlinkBatch => "unlink_batch",
            Self::UnlinkFinalBatch => "unlink_final_batch",
            Self::Rollback => "rollback",
            Self::CompactionBatchAfterTheRollback => "compaction_batch_after_the_rollback",
            Self::CompactionFinalBatchAfterTheRollback => {
                "compaction_final_batch_after_the_rollback"
            }
        }
    }
}

/// 流里一步：哪一种、它发出的那段录制流（绝对下标的半开区间）、它发成的每一版（按先后，连同那一版是哪一种发布）。
#[derive(Clone, Debug)]
pub struct DeletionCompactionFlowStep {
    pub kind: DeletionCompactionFlowStepKind,
    pub operations: Range<usize>,
    pub versions: Vec<(PublishOfTheDeletionCompactionFlow, TransactionOutput)>,
}

/// 录好的一条流：池（录制流在 `pool.stream` 上）、A 的输出、A 之后的每一步、回退的目标（整理签发之前那一版的根：重挂最后那次暖机）。
pub struct RecordedDeletionCompactionFlow {
    pub pool: BuiltPool,
    pub first_file: TransactionOutput,
    pub steps: Vec<DeletionCompactionFlowStep>,
    pub rollback_target: RollbackTarget,
}

impl RecordedDeletionCompactionFlow {
    /// 某一种的每一步，按先后。
    #[must_use]
    pub fn steps_of(
        &self,
        kind: DeletionCompactionFlowStepKind,
    ) -> Vec<&DeletionCompactionFlowStep> {
        self.steps.iter().filter(|step| step.kind == kind).collect()
    }

    /// 某一种发布的每一版，按先后。
    #[must_use]
    pub fn versions_of(
        &self,
        publish: PublishOfTheDeletionCompactionFlow,
    ) -> Vec<&TransactionOutput> {
        self.steps
            .iter()
            .flat_map(|step| step.versions.iter())
            .filter(|(publish_of_the_version, _output)| *publish_of_the_version == publish)
            .map(|(_publish_of_the_version, output)| output)
            .collect()
    }
}

/// 有树的一版（会话与挂载交回的现行版本）。
fn version_of_trees(version: &PoolVersion) -> &TransactionOutput {
    version
        .version_of_trees()
        .expect("重挂之前已经发过第一个文件：挂载与会话交回的每一版都有树")
}

/// 这一版的 `deleted_inodes` 里有没有那个文件（删除意图活着）。
fn is_the_other_file_being_deleted_in(version: &TransactionOutput) -> bool {
    version.deleted_inodes.contains(&INODE_OF_THE_OTHER_FILE)
}

/// 同一个进程（实例 1）里接着池的现行那一版发一次，记成一步，池接到它那一版上。
///
/// # Panics
/// 这一次被拒。
fn publish_in_process_as_instance_one(
    pool: &mut BuiltPool,
    kind: DeletionCompactionFlowStepKind,
    publish_of_the_version: PublishOfTheDeletionCompactionFlow,
    publish: impl FnOnce(
        &mut PoolWriter<'_, super::common::Recorded>,
        &mut singlefs_core::allocator::PoolAllocator,
        &TransactionOutput,
    ) -> Result<TransactionOutput, singlefs_core::transaction::PublishError>,
) -> DeletionCompactionFlowStep {
    let start = pool.stream.operation_count();
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish(&mut writer, &mut pool.allocator, &previous).unwrap_or_else(|refusal| {
        panic!(
            "{kind:?}：接在 txg {} 之后发得成：{refusal:?}",
            previous.root.checkpoint_txg.0
        )
    });
    let end = pool.stream.operation_count();
    pool.output = output.clone();
    DeletionCompactionFlowStep {
        kind,
        operations: start..end,
        versions: vec![(publish_of_the_version, output)],
    }
}

/// 重开镜像、可写挂载（删除开关同快照流：批大小 1），会话装上整理开关。挂载发的写录进同一条流。
fn remount_and_open_the_session(
    pool: &mut BuiltPool,
) -> (DeletionCompactionFlowStep, MountedSession) {
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
    let mut versions = vec![(
        PublishOfTheDeletionCompactionFlow::RowOrWarmUpOfTheMount,
        version_of_trees(&output.row_publish).clone(),
    )];
    versions.extend(output.warm_up_publishes.iter().map(|published| {
        (
            PublishOfTheDeletionCompactionFlow::RowOrWarmUpOfTheMount,
            version_of_trees(published).clone(),
        )
    }));
    (
        DeletionCompactionFlowStep {
            kind: DeletionCompactionFlowStepKind::MountWritable,
            operations: start..end,
            versions,
        },
        session,
    )
}

/// 会话里发一次用户改动，记成一步：这次调用发成的每一版按先后，逐版照 `publish_of_each_version` 归种类（参数是这一版在这次调用里的下标与版数）。
///
/// # Panics
/// 会话拒了这次改动，或这次改动里推了抬 F（这条流的盘空得很，准入一次都不推）。
fn publish_a_user_change_through_the_session(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    kind: DeletionCompactionFlowStepKind,
    change: UserChange<'_>,
    publish_of_each_version: impl Fn(usize, usize) -> PublishOfTheDeletionCompactionFlow,
) -> DeletionCompactionFlowStep {
    let start = pool.stream.operation_count();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let published = session
        .publish_user_change(devices, change)
        .unwrap_or_else(|refusal| panic!("{kind:?}：会话发得成：{refusal:?}"));
    let end = pool.stream.operation_count();
    assert!(
        published.floor_raises.is_empty(),
        "{kind:?}：盘空得很，准入一次抬 F 都不推：{:?}",
        published.floor_raises.len()
    );
    assert!(
        published.discards_after_the_change.applied.is_empty(),
        "{kind:?}：改动之后没有可再分配的整段，不发 discard"
    );
    let version_count = published.versions_published.len();
    let versions = published
        .versions_published
        .iter()
        .enumerate()
        .map(|(version_index, output)| {
            (
                publish_of_each_version(version_index, version_count),
                output.clone(),
            )
        })
        .collect();
    DeletionCompactionFlowStep {
        kind,
        operations: start..end,
        versions,
    }
}

/// 会话里整理发一次（预算压到 1），记成一步：签发、一批或最后一批，按 logged_ops 与搬走的单元认，交给 `publish_of_the_kind` 归发布种类。
///
/// # Panics
/// 这一趟没恰好发一次、停的不是预算用完、发了一次既没写意图也没搬单元，或认出的那一种不在 `expected_kinds` 里。
fn publish_the_compaction_once(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    expected_kinds: &[DeletionCompactionFlowStepKind],
    publish_of_the_kind: impl Fn(DeletionCompactionFlowStepKind) -> PublishOfTheDeletionCompactionFlow,
) -> DeletionCompactionFlowStep {
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
    assert!(
        run.discards.applied.is_empty(),
        "源落点还在 defer、没有可再分配的整段：整理之后不发 discard"
    );
    let [published] = <[TransactionOutput; 1]>::try_from(run.versions_published)
        .expect("预算 1：这一趟恰好发一次");
    let kind = match (
        published.logged_operations.is_empty(),
        published.units_relocated_by_this_publish.is_empty(),
    ) {
        (false, true) => DeletionCompactionFlowStepKind::CompactionIssue,
        (false, false) => DeletionCompactionFlowStepKind::CompactionBatch,
        (true, false) => DeletionCompactionFlowStepKind::CompactionFinalBatch,
        (true, true) => panic!(
            "整理一次发布既没写意图、也没搬单元（丢意图或只删意图）：这条流的 R 里没有隔离槽、每一批都搬得动，不该走到这里"
        ),
    };
    assert!(
        expected_kinds.contains(&kind),
        "这一次整理发布是 {kind:?}，不是这条流此刻该发的 {expected_kinds:?}"
    );
    DeletionCompactionFlowStep {
        kind,
        operations: start..end,
        versions: vec![(publish_of_the_kind(kind), published)],
    }
}

/// 管理员回退到 `target`（会话的分配器与现行那一版，`mount::roll_back_by_a_forward_publish`），记成一步；回退之后照
/// `PoolVersion::of_a_version_of_trees` 把会话的现行那一版重新归成员。
///
/// # Panics
/// 回退被拒，或会话的现行那一版不是带文件的一版。
fn roll_back_through_the_session(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    target: RollbackTarget,
) -> DeletionCompactionFlowStep {
    let start = pool.stream.operation_count();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("回退之前现行那一版带文件（第一个文件一直在）");
    };
    roll_back_by_a_forward_publish(
        &parameters(),
        devices,
        &mut session.allocator,
        current,
        target,
    )
    .unwrap_or_else(|refused| panic!("回退到 {target:?} 做得成：{refused:?}"));
    let rolled_back = current.clone();
    let end = pool.stream.operation_count();
    session.current = PoolVersion::of_a_version_of_trees(rolled_back.clone());
    DeletionCompactionFlowStep {
        kind: DeletionCompactionFlowStepKind::RollbackToTheVersionBeforeTheIssue,
        operations: start..end,
        versions: vec![(PublishOfTheDeletionCompactionFlow::Rollback, rolled_back)],
    }
}

/// 录这条流（`tag` 是镜像文件名里那一段）。
///
/// # Panics
/// 任何一步被拒、整理在签发之后紧接的那一批就删了意图（之后的交错无从谈起）、回退之后整理停不下来：这条流的每一步按条款都做得成。
#[must_use]
pub fn record_the_deletion_compaction_crash_flow(tag: &str) -> RecordedDeletionCompactionFlow {
    let mut pool = build_pool(tag);
    let first_file = pool.output.clone();
    let mut steps = Vec::new();
    let write_time_seconds =
        FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_FIRST_WRITE_TO_THE_LATER_CHANGES;
    let instance_of_the_first_file = first_file.root.instance;
    steps.push(publish_in_process_as_instance_one(
        &mut pool,
        DeletionCompactionFlowStepKind::NewInodeInProcess,
        PublishOfTheDeletionCompactionFlow::NewInode,
        |writer, allocator, previous| {
            publish_new_inodes(
                writer,
                allocator,
                previous,
                1,
                write_time_seconds,
                instance_of_the_first_file,
            )
        },
    ));
    let content = content_of_the_other_file();
    steps.push(publish_in_process_as_instance_one(
        &mut pool,
        DeletionCompactionFlowStepKind::SequentialWriteOfTheOtherFileInProcess,
        PublishOfTheDeletionCompactionFlow::SequentialWriteOfTheOtherFile,
        |writer, allocator, previous| {
            publish_sequential_write_of_an_inode(
                writer,
                allocator,
                previous,
                INODE_OF_THE_OTHER_FILE,
                &content,
                write_time_seconds,
                instance_of_the_first_file,
            )
        },
    ));
    let (mount_step, mut session) = remount_and_open_the_session(&mut pool);
    steps.push(mount_step);
    let version_before_the_issue = version_of_trees(&session.current);
    let rollback_target = RollbackTarget {
        instance: version_before_the_issue.root.instance,
        checkpoint_txg: version_before_the_issue.root.checkpoint_txg,
    };
    steps.push(publish_the_compaction_once(
        &mut pool,
        &mut session,
        &[DeletionCompactionFlowStepKind::CompactionIssue],
        |_kind| PublishOfTheDeletionCompactionFlow::CompactionIssue,
    ));
    steps.push(publish_the_compaction_once(
        &mut pool,
        &mut session,
        &[DeletionCompactionFlowStepKind::CompactionBatch],
        |_kind| PublishOfTheDeletionCompactionFlow::CompactionBatchBeforeTheRollback,
    ));
    steps.push(publish_a_user_change_through_the_session(
        &mut pool,
        &mut session,
        DeletionCompactionFlowStepKind::UnlinkOfTheOtherFileThroughTheSession,
        UserChange::Unlink {
            inode: INODE_OF_THE_OTHER_FILE,
            write_time_seconds,
        },
        |version_index, version_count| {
            if version_index == 0 {
                PublishOfTheDeletionCompactionFlow::UnlinkIntent
            } else if version_index + 1 == version_count {
                PublishOfTheDeletionCompactionFlow::UnlinkFinalBatch
            } else {
                PublishOfTheDeletionCompactionFlow::UnlinkBatch
            }
        },
    ));
    steps.push(roll_back_through_the_session(
        &mut pool,
        &mut session,
        rollback_target,
    ));
    for _publish_index in 0..COMPACTION_PUBLISHES_AFTER_THE_ROLLBACK_AT_MOST {
        let step = publish_the_compaction_once(
            &mut pool,
            &mut session,
            &[
                DeletionCompactionFlowStepKind::CompactionBatch,
                DeletionCompactionFlowStepKind::CompactionFinalBatch,
            ],
            |kind| match kind {
                DeletionCompactionFlowStepKind::CompactionFinalBatch => {
                    PublishOfTheDeletionCompactionFlow::CompactionFinalBatchAfterTheRollback
                }
                DeletionCompactionFlowStepKind::NewInodeInProcess
                | DeletionCompactionFlowStepKind::SequentialWriteOfTheOtherFileInProcess
                | DeletionCompactionFlowStepKind::MountWritable
                | DeletionCompactionFlowStepKind::CompactionIssue
                | DeletionCompactionFlowStepKind::CompactionBatch
                | DeletionCompactionFlowStepKind::UnlinkOfTheOtherFileThroughTheSession
                | DeletionCompactionFlowStepKind::RollbackToTheVersionBeforeTheIssue => {
                    PublishOfTheDeletionCompactionFlow::CompactionBatchAfterTheRollback
                }
            },
        );
        let is_the_final_batch = step.kind == DeletionCompactionFlowStepKind::CompactionFinalBatch;
        steps.push(step);
        if is_the_final_batch {
            pool.output = version_of_trees(&session.current).clone();
            return RecordedDeletionCompactionFlow {
                pool,
                first_file,
                steps,
                rollback_target,
            };
        }
    }
    panic!(
        "回退之后 {COMPACTION_PUBLISHES_AFTER_THE_ROLLBACK_AT_MOST} 次整理发布之内最后一批没删意图：{:?}",
        steps.iter().map(|step| step.kind).collect::<Vec<_>>()
    );
}

// ============================== 崩溃点重放要的那几样 ==============================

/// 流里的一版（A 与之后每一步发成的每一版）：根（实例，txg）、哪一种发布、根记录、读者在这一版上读那个文件该读出的内容
/// （还没建、或删除意图活着答没有文件是 `None`；D8（核心索引结构） 已定项 16 第 4 条）、那个文件是不是删除中（续做之后答没有文件）、
/// 这一版有没有活的整理意图、这一版与它之前整理搬走的单元里这一版没人用的源落点（读的时候先抹掉它们，核读路径照映射走到目的地）。
/// 第一个文件每一版都在、内容不变（这条流不动它）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionOfTheDeletionCompactionFlow {
    pub version: (InstanceGeneration, CheckpointTxg),
    pub publish: PublishOfTheDeletionCompactionFlow,
    pub root: RootRecord,
    pub content_of_the_other_file_read_back: Option<Vec<u8>>,
    pub is_the_other_file_being_deleted: bool,
    pub has_a_live_compaction_intent: bool,
    pub relocation_sources_unused_by_this_version: Vec<(DeviceIdentity, SlotNumber)>,
}

impl VersionOfTheDeletionCompactionFlow {
    /// 可写挂载续做走完之后读那个文件该读出的内容：删除中的续做把它删完、答没有文件；别的同这一版。
    #[must_use]
    pub fn content_of_the_other_file_after_the_continuation(&self) -> Option<Vec<u8>> {
        if self.is_the_other_file_being_deleted {
            None
        } else {
            self.content_of_the_other_file_read_back.clone()
        }
    }
}

/// 一条录好的流按崩溃点重放要的样子摆好：起点镜像（mkfs 之后）、写表与段、每次写在录制流里的下标、被判的根槽写（最后一条）、
/// 版本表（每一版都有第一个文件、内容不变）、观察者另读的流里每一版。
pub struct PreparedDeletionCompactionCrashFlow {
    pub recorded: RecordedDeletionCompactionFlow,
    pub base: MemoryPool,
    pub operations_after_mkfs: Vec<RetainedOperation>,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub stream_indexes: Vec<usize>,
    pub judged_root_index: usize,
    pub versions_with_a_file: Vec<PublishedVersion>,
    pub versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    pub versions_of_the_flow: Vec<VersionOfTheDeletionCompactionFlow>,
}

/// 这一版没人用的源落点：已搬走的源落点里，这一版的分配记录里没有一条未释放的记录罩着它的（回退把整理搬过的单元复活在源落点上，
/// 复活的那几槽这一版又有人用，不抹）。
fn sources_unused_by_the_version(
    output: &TransactionOutput,
    sources: &[(DeviceIdentity, SlotNumber)],
) -> Vec<(DeviceIdentity, SlotNumber)> {
    sources
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
        .collect()
}

/// 录这条流并摆好。版本表逐版现推：每一步的每一版都进表，第一个文件的内容每一版都是 `common::file_content`。
/// 这里只摆、不核形状：段形、两条意图同时活着、回退之后意图还在这些在用例里判器判完之后再核。
#[must_use]
pub fn prepare_the_deletion_compaction_crash_flow(
    tag: &str,
) -> PreparedDeletionCompactionCrashFlow {
    let recorded = record_the_deletion_compaction_crash_flow(tag);
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
    let first_file_content = file_content();
    let content_of_the_other_file = content_of_the_other_file();
    let mut versions_with_a_file = Vec::new();
    let mut versions_of_the_flow = Vec::new();
    let mut relocation_sources: Vec<(DeviceIdentity, SlotNumber)> = Vec::new();
    let every_version = std::iter::once((
        PublishOfTheDeletionCompactionFlow::FirstFile,
        &recorded.first_file,
    ))
    .chain(recorded.steps.iter().flat_map(|step| {
        step.versions
            .iter()
            .map(|(publish, output)| (*publish, output))
    }));
    for (publish, output) in every_version {
        relocation_sources.extend(output.units_relocated_by_this_publish.iter().flat_map(
            |relocated| {
                relocated
                    .source
                    .iter()
                    .map(|location| (location.device, location.slot))
            },
        ));
        versions_with_a_file.push(PublishedVersion {
            instance: output.root.instance,
            checkpoint_txg: output.root.checkpoint_txg,
            content: first_file_content.clone(),
        });
        let is_the_other_file_being_deleted = is_the_other_file_being_deleted_in(output);
        let content_of_the_other_file_read_back = match publish {
            PublishOfTheDeletionCompactionFlow::FirstFile => None,
            PublishOfTheDeletionCompactionFlow::NewInode => Some(Vec::new()),
            PublishOfTheDeletionCompactionFlow::SequentialWriteOfTheOtherFile
            | PublishOfTheDeletionCompactionFlow::RowOrWarmUpOfTheMount
            | PublishOfTheDeletionCompactionFlow::CompactionIssue
            | PublishOfTheDeletionCompactionFlow::CompactionBatchBeforeTheRollback
            | PublishOfTheDeletionCompactionFlow::Rollback
            | PublishOfTheDeletionCompactionFlow::CompactionBatchAfterTheRollback
            | PublishOfTheDeletionCompactionFlow::CompactionFinalBatchAfterTheRollback => {
                Some(content_of_the_other_file.clone())
            }
            PublishOfTheDeletionCompactionFlow::UnlinkIntent
            | PublishOfTheDeletionCompactionFlow::UnlinkBatch
            | PublishOfTheDeletionCompactionFlow::UnlinkFinalBatch => None,
        };
        versions_of_the_flow.push(VersionOfTheDeletionCompactionFlow {
            version: (output.root.instance, output.root.checkpoint_txg),
            publish,
            root: output.root,
            content_of_the_other_file_read_back,
            is_the_other_file_being_deleted,
            has_a_live_compaction_intent: !output.logged_operations.is_empty(),
            relocation_sources_unused_by_this_version: sources_unused_by_the_version(
                output,
                &relocation_sources,
            ),
        });
    }
    PreparedDeletionCompactionCrashFlow {
        recorded,
        base,
        operations_after_mkfs,
        writes,
        segments,
        stream_indexes,
        judged_root_index,
        versions_with_a_file,
        versions_without_a_file: Vec::new(),
        versions_of_the_flow,
    }
}

impl PreparedDeletionCompactionCrashFlow {
    /// 观察者另读的输入的摘要（流里每一版，[`text_of_the_versions_of_the_deletion_compaction_flow`]）：进观察者的版本，录制流一变它就变。
    #[must_use]
    pub fn digest_of_the_versions_of_the_flow(
        &self,
    ) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
        singlefs_harness::sha256::sha256_digest(
            text_of_the_versions_of_the_deletion_compaction_flow(&self.versions_of_the_flow)
                .as_bytes(),
        )
    }

    /// 每段的写数（用例核段形）。
    #[must_use]
    pub fn segment_sizes(&self) -> Vec<usize> {
        self.segments.iter().map(Vec::len).collect()
    }

    /// 写数小于 `writes_below` 的那几段（快档展开的段）。
    #[must_use]
    pub fn segments_shorter_than(&self, writes_below: usize) -> Vec<Vec<usize>> {
        self.segments
            .iter()
            .filter(|segment| segment.len() < writes_below)
            .cloned()
            .collect()
    }

    /// 录制流里的 discard 有几个。
    #[must_use]
    pub fn discards_in_the_stream(&self) -> usize {
        self.operations_after_mkfs
            .iter()
            .filter(|retained| retained.operation.kind == RecordedOperationKind::Discard)
            .count()
    }

    /// 一步里第 `version_index` 版那次发布落完（根槽写与之后的系统配置轮换都落了盘、下一次发布一个写都没发）的那张镜像：
    /// 起点镜像施加录制流到那一刻为止。harness 档的用例拿它当「崩在这一版之后」的盘（只取这一张，不枚举）。
    ///
    /// # Panics
    /// 这一步发成的版数不到 `version_index + 1`。
    #[must_use]
    pub fn image_right_after_the_version_of_the_step(
        &self,
        step: &DeletionCompactionFlowStep,
        version_index: usize,
    ) -> MemoryPool {
        let mkfs_operation_count = self.recorded.pool.mkfs_operation_count;
        let root_writes_of_the_step: Vec<usize> = self
            .writes
            .iter()
            .enumerate()
            .filter(|(write_index, write)| {
                write.kind == StepKind::RootRecordFua
                    && step
                        .operations
                        .contains(&(mkfs_operation_count + self.stream_indexes[*write_index]))
            })
            .map(|(write_index, _write)| write_index)
            .collect();
        assert_eq!(
            root_writes_of_the_step.len(),
            step.versions.len(),
            "{:?}：这一步每发成一版就写一次根槽",
            step.kind
        );
        let root_write_index = root_writes_of_the_step[version_index];
        let mut last_write_of_the_publish = root_write_index;
        while self
            .writes
            .get(last_write_of_the_publish + 1)
            .is_some_and(|write| write.kind == StepKind::SystemConfigurationSlot)
        {
            last_write_of_the_publish += 1;
        }
        let end = self.stream_indexes[last_write_of_the_publish] + 1;
        let mut image = self.base.clone();
        image.apply(&self.operations_after_mkfs[..end]);
        image
    }
}

/// 观察者另读的输入写成文字（取摘要进观察者的版本）：只有逐版那几行，不带标签与手写的版本号（D13（验证路线） 已定项 17：
/// 手写的版本常量不进键）；录法变了，变的是逐版的内容，摘要跟着变。
#[must_use]
pub fn text_of_the_versions_of_the_deletion_compaction_flow(
    versions: &[VersionOfTheDeletionCompactionFlow],
) -> String {
    let mut text = String::new();
    for version in versions {
        text.push_str(&format!(
            "version instance={} txg={} publish={:?} other_file_bytes={:?} other_file_being_deleted={} live_intent={} relocation_sources={:?}\n",
            version.version.0 .0,
            version.version.1 .0,
            version.publish,
            version
                .content_of_the_other_file_read_back
                .as_ref()
                .map(|content| singlefs_harness::fnv1a_64(content)),
            version.is_the_other_file_being_deleted,
            version.has_a_live_compaction_intent,
            version.relocation_sources_unused_by_this_version
        ));
    }
    text
}

/// 恢复落到的根在流里是哪一版。
#[derive(Clone, Copy, Debug)]
pub enum LandedVersionOfTheDeletionCompactionFlow<'flow> {
    /// 落到 A 之前（mkfs 或暖机那几条根）：根下面没有树。
    BeforeTheFirstFile,
    Version(&'flow VersionOfTheDeletionCompactionFlow),
}

/// 恢复落到的根（实例，txg）在流里是哪一版：不是流里任何一版的根都在 A 之前（流里每一版的根都在表里）。
///
/// # Panics
/// 恢复落到的根不是流里任何一版、又不在 A 之前。
#[must_use]
pub fn version_of_the_deletion_compaction_flow_landed_on(
    versions: &[VersionOfTheDeletionCompactionFlow],
    effective_root: (InstanceGeneration, CheckpointTxg),
) -> LandedVersionOfTheDeletionCompactionFlow<'_> {
    match versions
        .iter()
        .find(|version| version.version == effective_root)
    {
        Some(version) => LandedVersionOfTheDeletionCompactionFlow::Version(version),
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
            LandedVersionOfTheDeletionCompactionFlow::BeforeTheFirstFile
        }
    }
}

/// 在 `reader` 上冷走读这一版的第一个文件：读出 `common::file_content`（这条流不动第一个文件）。
///
/// # Panics
/// 读不出，或读出的不是它。
pub fn assert_the_first_file_reads_its_content(
    reader: &dyn PoolReader,
    root: &RootRecord,
    what: &str,
) {
    let mut mapping_fallbacks = 0usize;
    let read = walk_to_file(reader, root, &mut mapping_fallbacks);
    assert!(
        read.as_ref()
            .is_ok_and(|content| content.as_deref() == Some(file_content().as_slice())),
        "{what}：第一个文件读出它的内容（{} 字节），实际 {:?}",
        file_content().len(),
        read.map(|content| content.map(|bytes| bytes.len()))
    );
}

/// 只读挂载 `image`（扫环只这一次），打开那个文件、从偏移 0 读整个文件：打不开（没有这个号、或删除中）是 `None`。
/// 交回只读挂载择到的根（实例，txg）与读出的内容。
///
/// # Panics
/// 只读挂载做不成，或打开了却读不出整个文件。
#[must_use]
pub fn content_of_the_other_file_read_by_a_read_only_mount(
    image: &MemoryPool,
    what: &str,
) -> ((InstanceGeneration, CheckpointTxg), Option<Vec<u8>>) {
    let mounted = mount_read_only(image)
        .unwrap_or_else(|failure| panic!("{what}：只读挂载得上：{failure:?}"));
    let effective_root = (
        mounted.effective_root.instance,
        mounted.effective_root.checkpoint_txg,
    );
    let content = mounted
        .mounted
        .open_file(image, INODE_OF_THE_OTHER_FILE)
        .ok()
        .map(|file| {
            let size = file.inode_record().size;
            file.read_at(image, FileOffsetInBytes(0), size)
                .unwrap_or_else(|failure| {
                    panic!("{what}：打开了那个文件，读它的 {size} 字节：{failure:?}")
                })
                .bytes
        });
    (effective_root, content)
}

/// 沿 `root` 打开池（`mounted_read::open_pool_for_read`，不扫环），打开那个文件、从偏移 0 读整个文件：打不开（没有这个号、或删除中）是 `None`。
///
/// # Panics
/// 沿这条根打不开池，或打开了那个文件却读不出整个文件。
#[must_use]
pub fn content_of_the_other_file_read_at_the_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
    what: &str,
) -> Option<Vec<u8>> {
    let opened = open_pool_for_read(reader, root)
        .unwrap_or_else(|failure| panic!("{what}：沿这条根打得开池：{failure:?}"));
    opened
        .open_file(reader, INODE_OF_THE_OTHER_FILE)
        .ok()
        .map(|file| {
            let size = file.inode_record().size;
            file.read_at(reader, FileOffsetInBytes(0), size)
                .unwrap_or_else(|failure| {
                    panic!("{what}：打开了那个文件，读它的 {size} 字节：{failure:?}")
                })
                .bytes
        })
}

/// 读的那个观察者在一张崩溃镜像上核的几样（落到流里一版的）：沿这一版的根，第一个文件读出它的内容、那个文件读出这一版该读出的
/// （删除中的答没有文件）。观察者先把这一版没人用的源落点整槽抹成 0 再交进来（`common_compaction_prefix::image_with_the_relocation_sources_zeroed`），
/// 读路径照映射走到目的地才读得出。
///
/// # Panics
/// 上面任何一样对不上。
pub fn assert_the_version_reads_both_files(
    reader: &dyn PoolReader,
    version: &VersionOfTheDeletionCompactionFlow,
    what: &str,
) {
    let what = format!(
        "落到 {:?}（{:?}），{what}",
        version.version, version.publish
    );
    assert_the_first_file_reads_its_content(reader, &version.root, &what);
    let content = content_of_the_other_file_read_at_the_root(reader, &version.root, &what);
    assert!(
        content == version.content_of_the_other_file_read_back,
        "{what}：那个文件读出这一版该读出的（{:?} 字节），实际 {:?} 字节",
        version
            .content_of_the_other_file_read_back
            .as_ref()
            .map(Vec::len),
        content.as_ref().map(Vec::len)
    );
}

/// 在 `disks`（一张崩溃镜像落成的池）的副本上可写挂载一次，整理开关同录流：删除续做把队列走到空，整理续做把盘上那一条未完成的意图推到删了为止。
/// 同 `common_compaction_prefix::mount_writable_with_the_compaction_switches_on_the_crash_disks`。
#[must_use]
pub fn mount_writable_on_the_crash_disks(
    disks: &MemoryPool,
) -> MountedWithTheCompactionSwitchesAfterTheCrash {
    mount_writable_with_the_compaction_switches_on_the_crash_disks(disks)
}

/// 可写挂载之后核续做（D8（核心索引结构） 已定项 16 第 6 条「续做」、已定项 5「续做是每次恢复都跑的常规动作」；走空次序 `deleted_inodes` 在整理之前，
/// 判决 G8）：删除续做把队列走到空；恢复落到的那一版有活的整理意图时，整理续做推到意图删了为止（停在「没有活意图可续」）、续做之后 R 按记录全空
/// （停机谓词，判决 G4；续做的新落点一个都不进 R：挂载重建时恢复 R 排他，D26（后台整理与放置回收） 已定项 1 第 3 条），没有时一次都不发；
/// 挂完的那一版 logged_ops 空、删除队列空。挂完的盘上第一个文件读出它的内容，那个文件读出那一版续做走完之后该读出的（删除中的删完、答没有文件）。
/// 交回整理续做发了几次。
///
/// # Panics
/// 上面任何一样对不上。
pub fn assert_the_mount_continued_the_deletion_then_the_compaction(
    landed: LandedVersionOfTheDeletionCompactionFlow<'_>,
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
    let LandedVersionOfTheDeletionCompactionFlow::Version(version) = landed else {
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
            version.publish,
            run.outcome
        );
        assert!(
            region_is_empty_by_the_records(
                &current.allocation_records,
                region_of_the_crash_flows()
            ),
            "落到 {:?}（{:?}）：续做之后 R 按记录全空（停机谓词；续做的新落点不进 R）",
            version.version,
            version.publish
        );
    } else {
        assert_eq!(
            compaction_publishes, 0,
            "落到 {:?}（{:?}）：没有活的整理意图，挂载一次整理都不发（不签新的）",
            version.version, version.publish
        );
    }
    assert!(
        current.logged_operations.is_empty() && current.deleted_inodes.is_empty(),
        "落到 {:?}（{:?}）：挂完 logged_ops 与删除队列都空",
        version.version,
        version.publish
    );
    let what = format!(
        "落到 {:?}（{:?}），可写挂载续做之后",
        version.version, version.publish
    );
    assert_the_first_file_reads_its_content(&mounted.disks_after_the_mount, &current.root, &what);
    let content = content_of_the_other_file_read_at_the_root(
        &mounted.disks_after_the_mount,
        &current.root,
        &what,
    );
    let expected = version.content_of_the_other_file_after_the_continuation();
    assert!(
        content == expected,
        "{what}：那个文件读出续做走完之后该读出的（{:?} 字节），实际 {:?} 字节",
        expected.as_ref().map(Vec::len),
        content.as_ref().map(Vec::len)
    );
    compaction_publishes
}
