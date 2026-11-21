//! 验证包模块：crash、crash_state_enumeration、crash_library、crash_amplification
//! 活意图期间一批交到会话锁外算、前台在取输入与回锁之间发布一次改动的崩溃流（D26（后台整理与放置回收） 已定项 13 第 5 条：会话锁里取输入、锁外算、
//! 回会话锁里发布；回锁里不作废，先排空待发、重取隔离集，比起点在 R 里的未释放记录的整条记录与集合，变了的单元从计划里剔掉，其余在现行那一版上发布）：
//! 「取号 → 暖机 × 2 → A → B、C 两次顺序写 → 实例 1 里建空 inode 填满几个叶容器 → 重挂（取号、写行、暖机）→ 整理签发 → 一批
//!  → 取下一批的输入（票里是那一批发成的那一版，计划搬 R 里槽号最小的两个单元，其中一个是前台改动要重写的第一片叶容器）→ 前台改动发布 → 锁外算
//!  → 回锁剔掉前台改动放掉的那一个、只搬另一个 → 接着在锁里一批一批搬 → 最后一批（同一次发布删意图）」
//! 前台改动两种，各一条用例：意图活着时覆盖写第一个文件（与活意图期间写类改动流那一次覆盖写同一份内容），删第一个文件（会话里一次调用：意图那次发布与全部批）。
//! 两种都重写 R 里第一片叶容器与 inode 根；第一个文件的 extent 节点第一批已搬出 R。
//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态；每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器，观察者同整理搬迁流
//! （`common_compaction_crash_point_observers` 那两个：恢复落到的那一版照映射读、抹掉源落点再读，可写挂载一次看整理续做推到意图删了）。
//!
//! **比已有的流多罩了什么**：已登记的整理崩溃流（`crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs` 在内）都经会话的显式入口
//! 在锁里推整理，前台改动之后的那一批在现行那一版上现算，罩不到「计划在前台发布之前那一版上算好、回锁剔掉记录变了的单元、在现行那一版上发」这一种交错。
//! 这条流到前台改动为止与锁里现算的那一条逐字节相同（覆盖写那一种锁里现算的就是已登记的活意图期间写类改动流，
//! [`the_in_lock_reference_with_an_overwrite_writes_the_blocks_of_the_registered_flow_with_a_write_while_the_intent_is_live`] 钉着），
//! 前台改动之后的每一次发布都不同：回锁那一批搬的是锁里现算那一批的真子集（[`the_batch_after_an_overwrite_is_pruned_to_a_strict_subset_of_the_recomputation_in_the_lock`]
//! 与删文件那一条钉着），之后各批的边界跟着挪，所以回锁那一批与之后各批的每个崩溃状态、恢复落到这几版上的状态与在它们上面的可写挂载续做，都是已有流里没有的。
//! 多跑的一步：放锁的三段入口（`take_the_input_of_a_background_compaction_batch`、`compute_out_of_the_lock`、`publish_the_batch_computed_out_of_the_lock`）
//! 里回锁那一支剔单元再发，在崩溃枚举下走到。删文件那一种另罩「意图活着时会话里删第一个文件」：已有的流里活意图期间删的是别的 inode
//! （`crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs`）。
//!
//! 登记：崩溃点重放合表那一项与来历表那一行沿用活意图期间写类改动流的（项名第一个 `::` 之前那一截同它），两个观察者的代码文件点名共用的那一份
//! （`common_compaction_crash_point_observers::FILE_OF_THE_COMPACTION_CRASH_POINT_OBSERVERS`），观察者登记表里已有那两行。
//! 多机配置开着 GPU 时那两条用例的判段走 GPU 判器，观察者交回的盘要上卡判池级 checker，任务不拿卡、判不出结局；每种前台改动另有一条在 CPU 判器上
//! 逐状态判同一组崩溃状态的用例（只开 CPU 的多机配置，只挂读的那个观察者），改坏回锁那一支与版本表的变异行点名它。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。
//! 录流照 `common_compaction_prefix` 录活意图期间写类改动流的那几步写在这一份里（那份模块的流里没有放锁的三段）：锁里现算那一条与已登记的流逐字节相同由上面那条用例钉着。

mod common;
mod common_compaction_crash_point_observers;
mod common_compaction_prefix;
mod common_crash_point_observers;
mod common_crash_points;
mod common_snapshot_prefix;

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use common::{
    build_pool, file_content, geometry, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
    FIXED_WRITE_TIME_SECONDS, OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
};
use common_compaction_crash_point_observers::{
    assert_every_state_was_observed, landed_counter_of, landed_version_observer,
    mount_continuation_observer, observe_the_landed_version_reading_through_the_mapping,
    observe_the_mount_continuing_the_compaction, FILE_OF_THE_COMPACTION_CRASH_POINT_OBSERVERS,
    LANDED_BEFORE_THE_FIRST_FILE,
};
use common_compaction_prefix::{
    compaction_switches_of_the_crash_flows, content_overwritten_while_the_intent_is_live,
    record_the_compaction_crash_flow, version_of_trees, CompactionCrashFlow, CompactionFlowStep,
    CompactionFlowStepKind, PreparedCompactionCrashFlow, RecordedCompactionFlow,
    VersionOfTheCompactionFlow, COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS,
    COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST,
    INODE_LEAF_CONTAINERS_LEFT_IN_THE_REGION_BEHIND_THE_WRITE,
    PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE,
    SECONDS_FROM_THE_FIRST_WRITE_TO_THE_INODES_LEFT_IN_THE_REGION,
};
use common_snapshot_prefix::{live_snapshot_trees_of, SnapshotFlowStepKind, INSTANCE_OF_THE_FLOWS};
use singlefs_core::address::{DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::compaction_plan::{
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionPacing, CompactionRunForcing, CompactionRunStop,
    CompactionStartJudgement, CompactionSwitches, CompactionTimerFallback,
    CompactionWriteBandwidthLimit, CompactionYieldFloor,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, ShadowLedger,
};
use singlefs_core::mounted_session::{
    BackgroundCompactionCall, BackgroundCompactionInputTaken, CompactionRunOutcome, MountedSession,
    MountedSessionWithPendingChanges, UserChange,
};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::{
    publish_new_inodes, FirstFile, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
use singlefs_verification::crash_library::{
    multi_host_configuration_on_the_cpu_keeping_the_prune_server_of, ExclusiveTemporaryDirectory,
    TheOneLibraryKeptForThisProcess,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_attached_observers, AttachedCrashPointReplayObserver,
    AttachedCrashPointReplayObserverHandingBackAnImage, CrashPointReplayItem,
    CrashPointReplayObserverAttachment, CrashPointReplayObserverCodeFile,
    CrashPointReplayObserverPoints,
};
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, CrashImage, PublishedVersion,
    PublishedVersionWithoutAFile,
};
use singlefs_verification::scenario::deletion_history_switches;
use singlefs_verification::segments::StepKind;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件。
const PATH_OF_THIS_TEST_FILE: &str = "crates/singlefs-verification/tests/crash_enumeration_compaction_batch_computed_before_a_foreground_change_and_pruned_at_the_relock_stream.rs";

/// 多记录流 B、C 的内容（与活意图期间写类改动流、`common_crash_points` 的那两份同一份字节）。
fn contents_of_the_sequential_writes() -> Vec<Vec<u8>> {
    vec![
        common_crash_points::content_needing(2, 1),
        common_crash_points::content_needing(3, 2),
    ]
}

/// 意图活着时前台做的那一次改动（封闭集合）。两种都重写 R 里第一片叶容器与 inode 根。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ForegroundChangeWhileTheIntentIsLive {
    /// 覆盖写第一个文件（[`content_overwritten_while_the_intent_is_live`]），同活意图期间写类改动流那一次。
    OverwriteOfTheFirstFile,
    /// 删第一个文件：会话里一次调用，意图那次发布与全部批（批大小同录流的删除开关，`deletion_history_switches`）。
    UnlinkOfTheFirstFile,
}

/// 签发之后第一批之后的那一批怎么算（封闭集合）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BatchAfterTheForegroundChange {
    /// 放锁的三段：取输入（票里是第一批发成的那一版）→ 前台改动发布 → 锁外算 → 回锁剔掉记录变了的单元、在现行那一版上发。
    ComputedOutOfTheLockAroundTheChange,
    /// 先发前台改动，再经显式入口在锁里推一批（在现行那一版上现算）：覆盖写那一种就是已登记的活意图期间写类改动流。
    RecomputedInTheLockAfterTheChange,
}

/// 录好的一条流，连同回锁那一次剔掉的单元数（会话的观测计数在回锁前后之差；锁里现算那一条是 0）。
struct RecordedFlowAroundTheForegroundChange {
    recorded: RecordedCompactionFlow,
    units_pruned_at_the_relock: u64,
}

/// 后台入口的调用方：钟停在 0，前台没有入口在等会话锁。
struct CallerWithNothingWaiting;

impl CallerOfTheBackgroundCompaction for CallerWithNothingWaiting {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::ZERO)
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

/// 取输入那一次的节奏：定时兜底关、写带宽不设上限（起跑靠记下的「该跑了」）。
fn pacing_without_the_timer_fallback_and_the_write_bandwidth_limit() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::Off,
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

/// 录流的整理开关，只差起跑判据恒为真（只供测试的 [`CompactionRunForcing::ForcedToStartForTests`]：取输入那一次不看水位、不读根环那一份预算，发布数由录流自己摆）。
fn compaction_switches_of_the_crash_flows_forced_to_start() -> CompactionSwitches {
    let CompactionSwitches {
        batch_size,
        forced_region,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = compaction_switches_of_the_crash_flows();
    CompactionSwitches {
        batch_size,
        forced_region,
        run_forcing: CompactionRunForcing::ForcedToStartForTests,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    }
}

/// 发一步实例 1 的发布、记下它发出的那段录制流，池接到它那一版上。
fn record_a_publish_of_instance_one(
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

/// 实例 1 里建空 inode，连同第一个文件恰好填满 [`INODE_LEAF_CONTAINERS_LEFT_IN_THE_REGION_BEHIND_THE_WRITE`] 个叶容器：
/// 这次发布的提交内生块接着 C 落在 R 里（重挂之后 R 不是本次挂载开过的段）。
fn publish_the_inode_leaf_containers_left_in_the_region(pool: &mut BuiltPool) -> TransactionOutput {
    let previous = pool.output.clone();
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        INODE_LEAF_CONTAINERS_LEFT_IN_THE_REGION_BEHIND_THE_WRITE * INODE_LEAF_RECORDS - 1,
        FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_FIRST_WRITE_TO_THE_INODES_LEFT_IN_THE_REGION,
        INSTANCE_OF_THE_FLOWS,
    )
    .expect("重挂之前建空 inode：盘空得很、叶容器一个根装得下")
}

/// 重开镜像、可写挂载（删除开关同快照流：批大小 1，`deletion_history_switches`），会话装上整理开关、套上攒批的会话。挂载发的写录进同一条流。
fn remount_and_open_the_session_with_pending_changes(
    pool: &mut BuiltPool,
) -> (CompactionFlowStep, MountedSessionWithPendingChanges) {
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
        MountedSessionWithPendingChanges::of_the_session(session),
    )
}

/// 经显式入口在锁里推整理一趟、只发一次（发布预算压到 1），记成一步：签发、一批、最后一批按 logged_ops 与搬走的单元认，这一趟发布都返回之后
/// 会话发的 discard 记在这一步上。`steps_so_far` 只进 panic 的说明。
///
/// # Panics
/// 这一趟没停在预算用完、没恰好发一次，或那一次既没写意图、也没搬单元（这条流的 R 里最后一批之前都还有要搬的单元）。
fn publish_the_compaction_in_the_lock_once_as_a_step(
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
    let [published] = run.versions_published.as_slice() else {
        panic!(
            "预算 1：这一趟恰好发一次，实际 {} 次",
            run.versions_published.len()
        );
    };
    let kind = match (
        published.logged_operations.is_empty(),
        published.units_relocated_by_this_publish.is_empty(),
    ) {
        (false, true) => CompactionFlowStepKind::CompactionIssue,
        (false, false) => CompactionFlowStepKind::CompactionBatch,
        (true, false) => CompactionFlowStepKind::CompactionFinalBatch,
        (true, true) => panic!(
            "整理一次发布既没写意图、也没搬单元：这条流最后一批之前 R 里都还有要搬的单元；已录的步 {:?}",
            steps_so_far.iter().map(|step| step.kind).collect::<Vec<_>>()
        ),
    };
    CompactionFlowStep {
        kind,
        operations: start..end,
        versions: vec![published.clone()],
        discards: run.discards.applied.clone(),
    }
}

/// 前台改动那一步记成哪一种：覆盖写是 [`CompactionFlowStepKind::OverwriteWhileTheIntentIsLive`]；删文件是 `Snapshot(UnlinkIntent)`
/// （共用的步种类里删第一个文件的只有这一种，只用来起计数名：恢复落到这一步任何一版上的状态都记在它名下）。
fn step_kind_of_the_foreground_change(
    change: ForegroundChangeWhileTheIntentIsLive,
) -> CompactionFlowStepKind {
    match change {
        ForegroundChangeWhileTheIntentIsLive::OverwriteOfTheFirstFile => {
            CompactionFlowStepKind::OverwriteWhileTheIntentIsLive
        }
        ForegroundChangeWhileTheIntentIsLive::UnlinkOfTheFirstFile => {
            CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkIntent)
        }
    }
}

/// 经攒批会话的前台入口发那一次改动，记成一步（[`step_kind_of_the_foreground_change`]；这一次调用发成的每一版，含为空间推的抬 F，按 txg；之后的 discard）。
fn publish_the_foreground_change_as_a_step(
    pool: &mut BuiltPool,
    pending: &mut MountedSessionWithPendingChanges,
    change: ForegroundChangeWhileTheIntentIsLive,
) -> CompactionFlowStep {
    let content = content_overwritten_while_the_intent_is_live();
    let write_time_seconds = FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE;
    let kind = step_kind_of_the_foreground_change(change);
    let user_change = match change {
        ForegroundChangeWhileTheIntentIsLive::OverwriteOfTheFirstFile => {
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds,
            })
        }
        ForegroundChangeWhileTheIntentIsLive::UnlinkOfTheFirstFile => UserChange::Unlink {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            write_time_seconds,
        },
    };
    let start = pool.stream.operation_count();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let reports = pending
        .publish_user_change(
            devices,
            user_change,
            CallerWithNothingWaiting.clock_reading(),
        )
        .unwrap_or_else(|refusal| panic!("{change:?}：会话发得成：{refusal:?}"));
    let end = pool.stream.operation_count();
    let mut versions: Vec<TransactionOutput> = Vec::new();
    let mut discards: Vec<(DeviceIdentity, SlotNumber)> = Vec::new();
    for report in &reports {
        versions.extend(
            report
                .published
                .floor_raises
                .iter()
                .flat_map(|raised| raised.publishes.iter().cloned()),
        );
        versions.extend(
            report
                .published
                .versions_published
                .iter()
                .map(Arc::as_ref)
                .cloned(),
        );
        discards.extend(
            report
                .published
                .discards_after_the_change
                .applied
                .iter()
                .copied(),
        );
    }
    versions.sort_by_key(|version| version.root.checkpoint_txg);
    CompactionFlowStep {
        kind,
        operations: start..end,
        versions,
        discards,
    }
}

/// 放锁的三段那一批，连同它前面的前台改动：开关只在取输入那一刻强制起跑（会话上记「该跑了」、上一次用户改动之后的发布数归零），取完就换回录流的那一份，
/// 前台改动与回锁都在录流的开关下；取输入一个写都不发，锁外算时要读盘就回会话里读。交回前台改动那一步、回锁那一批与回锁剔掉的单元数。
///
/// # Panics
/// 取输入没把这一批交到锁外、回锁被拒，或回锁那一次不是恰好发一版照抄意图、真搬了单元的一批。
fn publish_the_batch_computed_out_of_the_lock_around_the_change(
    pool: &mut BuiltPool,
    pending: &mut MountedSessionWithPendingChanges,
    change: ForegroundChangeWhileTheIntentIsLive,
) -> ([CompactionFlowStep; 2], u64) {
    let caller = CallerWithNothingWaiting;
    let units_pruned_before_the_take = {
        let session = pending.session_for_tests_only();
        session
            .allocator
            .set_compaction_switches(compaction_switches_of_the_crash_flows_forced_to_start());
        session.compaction.publishes_since_the_last_user_change = 0;
        session.compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
        session.compaction.observations.units_pruned_at_the_relock
    };
    let devices_at_the_take = pool.devices.as_mut().expect("镜像还开着");
    let ticket = match pending.take_the_input_of_a_background_compaction_batch(
        devices_at_the_take,
        pacing_without_the_timer_fallback_and_the_write_bandwidth_limit(),
        CompactionYieldFloor::NO_FLOOR,
        &caller,
    ) {
        Ok(BackgroundCompactionInputTaken::BatchOutOfTheLock { ticket, .. }) => ticket,
        Ok(BackgroundCompactionInputTaken::FinishedInTheLock(after)) => {
            panic!("这一批应当交到锁外：在锁里就走完了，{:?}", after.call)
        }
        Err(refused) => panic!("这一批应当交到锁外：取输入被拒，{refused:?}"),
    };
    pending
        .session_for_tests_only()
        .allocator
        .set_compaction_switches(compaction_switches_of_the_crash_flows());
    let change_step = publish_the_foreground_change_as_a_step(pool, pending, change);
    let devices_read_out_of_the_lock = pool.devices.as_ref().expect("镜像还开着");
    let computed = (*ticket).compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(
            devices_read_out_of_the_lock.as_slice(),
            reads,
        )
    });
    let start_of_the_relock = pool.stream.operation_count();
    let devices_at_the_relock = pool.devices.as_mut().expect("镜像还开着");
    let after = pending
        .publish_the_batch_computed_out_of_the_lock(devices_at_the_relock, computed, &caller)
        .unwrap_or_else(|refused| panic!("回锁收下这一批：{refused:?}"));
    let end_of_the_relock = pool.stream.operation_count();
    let run = match after.call {
        BackgroundCompactionCall::Ran { run, .. } => run,
        BackgroundCompactionCall::NotStarted { reason, .. } => {
            panic!("回锁那一次应当发这一批：没起跑，{reason:?}")
        }
    };
    let [published] = run.versions_published.as_slice() else {
        panic!(
            "回锁那一次恰好发一版，实际 {} 版，停在 {:?}",
            run.versions_published.len(),
            run.outcome
        );
    };
    assert!(
        !published.logged_operations.is_empty()
            && !published.units_relocated_by_this_publish.is_empty(),
        "回锁发的是照抄意图、真搬了单元的一批（R 里剔完还有要搬的），停在 {:?}",
        run.outcome
    );
    let units_pruned_at_the_relock = pending
        .session()
        .compaction
        .observations
        .units_pruned_at_the_relock
        - units_pruned_before_the_take;
    let batch_step = CompactionFlowStep {
        kind: CompactionFlowStepKind::CompactionBatch,
        operations: start_of_the_relock..end_of_the_relock,
        versions: vec![published.clone()],
        discards: run.discards.applied.clone(),
    };
    ([change_step, batch_step], units_pruned_at_the_relock)
}

/// 录一条流（`tag` 是镜像文件名里那一段）：A → B、C → 建空 inode → 重挂 → 签发 → 一批 → 前台改动与它之后那一批（照 `batch_after_the_change`）
/// → 在锁里一批一批搬，直到最后一批同一次发布删意图。
///
/// # Panics
/// 任何一步发布被拒，签发、第一批不是这两步，或 [`COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST`] 次之内删不了意图。
fn record_the_flow_around_the_foreground_change(
    tag: &str,
    change: ForegroundChangeWhileTheIntentIsLive,
    batch_after_the_change: BatchAfterTheForegroundChange,
) -> RecordedFlowAroundTheForegroundChange {
    let mut pool = build_pool(tag);
    let first_file = pool.output.clone();
    let mut steps: Vec<CompactionFlowStep> = Vec::new();
    for content in contents_of_the_sequential_writes() {
        steps.push(record_a_publish_of_instance_one(
            &mut pool,
            CompactionFlowStepKind::SequentialWrite,
            |built_pool| {
                sequential_write_in_process_as_instance_one_and_advance_the_pool(
                    built_pool, &content,
                )
            },
        ));
    }
    steps.push(record_a_publish_of_instance_one(
        &mut pool,
        CompactionFlowStepKind::NewInodesThroughTheSession,
        publish_the_inode_leaf_containers_left_in_the_region,
    ));
    let (mount_step, mut pending) = remount_and_open_the_session_with_pending_changes(&mut pool);
    steps.push(mount_step);
    for expected_kind in [
        CompactionFlowStepKind::CompactionIssue,
        CompactionFlowStepKind::CompactionBatch,
    ] {
        let step = publish_the_compaction_in_the_lock_once_as_a_step(
            &mut pool,
            pending.session_for_tests_only(),
            &steps,
        );
        assert_eq!(step.kind, expected_kind, "前台改动之前的整理步：签发、一批");
        steps.push(step);
    }
    let units_pruned_at_the_relock = match batch_after_the_change {
        BatchAfterTheForegroundChange::ComputedOutOfTheLockAroundTheChange => {
            let (change_and_batch, units_pruned) =
                publish_the_batch_computed_out_of_the_lock_around_the_change(
                    &mut pool,
                    &mut pending,
                    change,
                );
            steps.extend(change_and_batch);
            units_pruned
        }
        BatchAfterTheForegroundChange::RecomputedInTheLockAfterTheChange => {
            let change_step =
                publish_the_foreground_change_as_a_step(&mut pool, &mut pending, change);
            steps.push(change_step);
            0
        }
    };
    let mut is_the_intent_deleted = false;
    for _publish_index in 0..COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST {
        let step = publish_the_compaction_in_the_lock_once_as_a_step(
            &mut pool,
            pending.session_for_tests_only(),
            &steps,
        );
        is_the_intent_deleted = step.kind == CompactionFlowStepKind::CompactionFinalBatch;
        steps.push(step);
        if is_the_intent_deleted {
            break;
        }
    }
    assert!(
        is_the_intent_deleted,
        "{COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST} 次整理发布之内最后一批没删意图：{:?}",
        steps.iter().map(|step| step.kind).collect::<Vec<_>>()
    );
    pool.output = version_of_trees(&pending.session().current).clone();
    RecordedFlowAroundTheForegroundChange {
        recorded: RecordedCompactionFlow {
            flow: CompactionCrashFlow::CompactionWithAWriteWhileTheIntentIsLive,
            pool,
            first_file,
            steps,
            snapshot: None,
        },
        units_pruned_at_the_relock,
    }
}

/// 录好的流按崩溃点重放要的样子摆好（同 `common_compaction_prefix::prepare_the_compaction_crash_flow` 的摆法）：起点镜像（mkfs 之后）、写表与段、
/// 被判的根槽写（最后一条）、版本表逐版现推（顺序写换成 B、C 的内容，覆盖写换成覆盖写那一份，删第一个文件之后没有文件，别的步不动文件内容）、
/// 观察者另读的流里每一版（带这一版与它之前整理搬走的单元里这一版没人用的源落点）。
fn prepared_flow_of_the_recording(recorded: RecordedCompactionFlow) -> PreparedCompactionCrashFlow {
    let contents = contents_of_the_sequential_writes();
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
    let mut relocation_sources: Vec<(DeviceIdentity, SlotNumber)> = Vec::new();
    let mut sequential_writes_seen = 0_usize;
    let mut note_version =
        |kind: Option<CompactionFlowStepKind>,
         output: &TransactionOutput,
         content_of_this_version: &Option<Vec<u8>>,
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
            versions_of_the_flow.push(VersionOfTheCompactionFlow {
                version: (output.root.instance, output.root.checkpoint_txg),
                kind,
                root: output.root,
                first_file_content: content_of_this_version.clone(),
                live_snapshots: live_snapshot_trees_of(output),
                content_captured_by_the_snapshots: None,
                has_a_live_compaction_intent: !output.logged_operations.is_empty(),
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
        &relocation_sources,
    );
    for step in &recorded.steps {
        match step.kind {
            CompactionFlowStepKind::SequentialWrite => {
                first_file_content = Some(
                    contents
                        .get(sequential_writes_seen)
                        .expect("流里两次顺序写，内容都在 contents_of_the_sequential_writes 里")
                        .clone(),
                );
                sequential_writes_seen += 1;
            }
            CompactionFlowStepKind::OverwriteWhileTheIntentIsLive => {
                first_file_content = Some(content_overwritten_while_the_intent_is_live());
            }
            CompactionFlowStepKind::Snapshot(
                SnapshotFlowStepKind::UnlinkIntent | SnapshotFlowStepKind::UnlinkFinalBatch,
            ) => {
                first_file_content = None;
            }
            CompactionFlowStepKind::OverwriteBeforeTheSnapshot
            | CompactionFlowStepKind::Snapshot(
                SnapshotFlowStepKind::CreateSnapshot
                | SnapshotFlowStepKind::OverwriteUnderTheSnapshot
                | SnapshotFlowStepKind::SnapshotDestructionIntent
                | SnapshotFlowStepKind::SnapshotDestructionBatch
                | SnapshotFlowStepKind::SnapshotDestructionFinalBatch,
            )
            | CompactionFlowStepKind::DestroySnapshotThroughTheSession
            | CompactionFlowStepKind::RaiseFloorToTheAdmissionCeiling
            | CompactionFlowStepKind::CompactionIntentDroppedOnAStop => panic!(
                "这条流不录 {:?} 这一种步：录流只录顺序写、建 inode、重挂、整理、前台改动",
                step.kind
            ),
            CompactionFlowStepKind::MountWritable
            | CompactionFlowStepKind::CompactionIssue
            | CompactionFlowStepKind::CompactionBatch
            | CompactionFlowStepKind::CompactionFinalBatch
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

/// 流里每一种那一版的计数名，每一种都要有状态。
fn landed_counters(prepared: &PreparedCompactionCrashFlow) -> Vec<String> {
    let mut counters: Vec<String> = Vec::new();
    for version in &prepared.versions_of_the_flow {
        let counter = landed_counter_of(version);
        if !counters.contains(&counter) {
            counters.push(counter);
        }
    }
    counters
}

/// 前台改动那一步在流里的位置。
fn position_of_the_foreground_change(
    recorded: &RecordedCompactionFlow,
    change: ForegroundChangeWhileTheIntentIsLive,
) -> usize {
    let kind_of_the_change = step_kind_of_the_foreground_change(change);
    recorded
        .steps
        .iter()
        .position(|step| step.kind == kind_of_the_change)
        .expect("流里有那一次前台改动")
}

/// 一步搬走的单元的映射 key。
fn mapping_keys_relocated_by(step: &CompactionFlowStep) -> BTreeSet<Vec<u8>> {
    step.versions
        .iter()
        .flat_map(|version| version.units_relocated_by_this_publish.iter())
        .map(|relocated| relocated.mapping_key.clone())
        .collect()
}

/// 同一个前台改动两条流：一条前台改动之后那一批放锁算（计划在改动之前那一版上算好、回锁剔掉），一条在锁里现算。
/// 预期：放锁那一条里签发、第一批、前台改动、回锁那一批依次相接；回锁剔掉的单元数加回锁那一批搬的单元数是一整批；
/// 到前台改动为止两条流写出的块序列逐字节相同（取输入一个写都不发）；回锁那一批搬的是锁里现算那一批的非空真子集。
fn the_batch_after_the_change_is_pruned_to_a_strict_subset_of_the_recomputation_in_the_lock(
    change: ForegroundChangeWhileTheIntentIsLive,
    tag: &str,
) {
    let out_of_the_lock = record_the_flow_around_the_foreground_change(
        &format!("{tag}-computed-out-of-the-lock"),
        change,
        BatchAfterTheForegroundChange::ComputedOutOfTheLockAroundTheChange,
    );
    let in_the_lock = record_the_flow_around_the_foreground_change(
        &format!("{tag}-recomputed-in-the-lock"),
        change,
        BatchAfterTheForegroundChange::RecomputedInTheLockAfterTheChange,
    );
    let recorded_out_of_the_lock = &out_of_the_lock.recorded;
    let recorded_in_the_lock = &in_the_lock.recorded;
    let kinds: Vec<CompactionFlowStepKind> = recorded_out_of_the_lock
        .steps
        .iter()
        .map(|step| step.kind)
        .collect();
    let position_out_of_the_lock =
        position_of_the_foreground_change(recorded_out_of_the_lock, change);
    assert_eq!(
        kinds[position_out_of_the_lock - 2..position_out_of_the_lock + 2],
        [
            CompactionFlowStepKind::CompactionIssue,
            CompactionFlowStepKind::CompactionBatch,
            step_kind_of_the_foreground_change(change),
            CompactionFlowStepKind::CompactionBatch,
        ],
        "{change:?}：签发、第一批、前台改动、回锁那一批依次相接：{kinds:?}"
    );
    let batch_out_of_the_lock = &recorded_out_of_the_lock.steps[position_out_of_the_lock + 1];
    let moved_out_of_the_lock = mapping_keys_relocated_by(batch_out_of_the_lock);
    assert_eq!(
        out_of_the_lock.units_pruned_at_the_relock
            + u64::try_from(moved_out_of_the_lock.len()).expect("一批的单元数"),
        u64::try_from(COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS).expect("一批的单元数"),
        "{change:?}：计划是一整批，回锁剔掉的加上搬的就是那一批"
    );
    let position_in_the_lock = position_of_the_foreground_change(recorded_in_the_lock, change);
    let operations_out_of_the_lock = recorded_out_of_the_lock.pool.retained_operations();
    let operations_in_the_lock = recorded_in_the_lock.pool.retained_operations();
    let change_out_of_the_lock = &recorded_out_of_the_lock.steps[position_out_of_the_lock];
    let change_in_the_lock = &recorded_in_the_lock.steps[position_in_the_lock];
    assert!(
        change_out_of_the_lock.operations == change_in_the_lock.operations
            && operations_out_of_the_lock[..change_out_of_the_lock.operations.end]
                == operations_in_the_lock[..change_in_the_lock.operations.end],
        "{change:?}：到前台改动为止两条流写出的块序列逐字节相同（取输入一个写都不发）：前台改动在录制流里 {:?} 对 {:?}",
        change_out_of_the_lock.operations,
        change_in_the_lock.operations
    );
    let moved_in_the_lock =
        mapping_keys_relocated_by(&recorded_in_the_lock.steps[position_in_the_lock + 1]);
    assert!(
        !moved_out_of_the_lock.is_empty()
            && moved_out_of_the_lock.is_subset(&moved_in_the_lock)
            && moved_out_of_the_lock.len() < moved_in_the_lock.len(),
        "{change:?}：回锁那一批搬的是锁里现算那一批的非空真子集：回锁 {moved_out_of_the_lock:?}，锁里 {moved_in_the_lock:?}"
    );
}

#[test]
fn the_batch_after_an_overwrite_is_pruned_to_a_strict_subset_of_the_recomputation_in_the_lock() {
    the_batch_after_the_change_is_pruned_to_a_strict_subset_of_the_recomputation_in_the_lock(
        ForegroundChangeWhileTheIntentIsLive::OverwriteOfTheFirstFile,
        "compaction-batch-out-of-the-lock-around-an-overwrite-shape",
    );
}

#[test]
fn the_batch_after_an_unlink_is_pruned_to_a_strict_subset_of_the_recomputation_in_the_lock() {
    the_batch_after_the_change_is_pruned_to_a_strict_subset_of_the_recomputation_in_the_lock(
        ForegroundChangeWhileTheIntentIsLive::UnlinkOfTheFirstFile,
        "compaction-batch-out-of-the-lock-around-an-unlink-shape",
    );
}

/// 场景：覆盖写那一种、前台改动之后那一批在锁里现算（这份文件录流的那几步），与已登记的活意图期间写类改动流（`common_compaction_prefix` 录的那一条）。
/// 预期：两条写出的块序列逐字节相同：这份文件的录流到前台改动为止就是已登记的那一条，放锁那一条与它比出的不同就是与已登记的流比出的不同。
#[test]
fn the_in_lock_reference_with_an_overwrite_writes_the_blocks_of_the_registered_flow_with_a_write_while_the_intent_is_live(
) {
    let in_the_lock = record_the_flow_around_the_foreground_change(
        "compaction-in-the-lock-reference-of-the-registered-flow",
        ForegroundChangeWhileTheIntentIsLive::OverwriteOfTheFirstFile,
        BatchAfterTheForegroundChange::RecomputedInTheLockAfterTheChange,
    );
    let registered = record_the_compaction_crash_flow(
        "compaction-registered-flow-with-a-write-while-the-intent-is-live",
        CompactionCrashFlow::CompactionWithAWriteWhileTheIntentIsLive,
        &contents_of_the_sequential_writes(),
    );
    let operations_of_this_file = in_the_lock.recorded.pool.retained_operations();
    let operations_of_the_registered_flow = registered.pool.retained_operations();
    assert!(
        operations_of_this_file.len() == operations_of_the_registered_flow.len()
            && operations_of_this_file == operations_of_the_registered_flow,
        "两条流写出的块序列逐字节相同：{} 步对 {} 步",
        operations_of_this_file.len(),
        operations_of_the_registered_flow.len()
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker + 抹掉源落点再读，并可写挂载一次核整理续做。登记成崩溃枚举用例。
fn every_crash_state_with_the_batch_computed_out_of_the_lock_around_the_change_is_clean(
    change: ForegroundChangeWhileTheIntentIsLive,
    tag: &str,
    item_name: &str,
    count_line_name: &str,
) {
    let prepared = prepared_flow_of_the_recording(
        record_the_flow_around_the_foreground_change(
            tag,
            change,
            BatchAfterTheForegroundChange::ComputedOutOfTheLockAroundTheChange,
        )
        .recorded,
    );
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version_reading_through_the_mapping(
            versions_of_the_flow,
            image,
            report,
            counts,
        );
    };
    let observe_the_mount =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuing_the_compaction(versions_of_the_flow, image, report, counts)
        };
    let Some(tally) = enumerate_crash_states_with_attached_observers(
        CrashPointReplayItem {
            name: item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        CrashPointReplayVersionTable::new(
            &prepared.versions_with_a_file,
            &prepared.versions_without_a_file,
        ),
        &full_expansion,
        &[AttachedCrashPointReplayObserver {
            observer: landed_version_observer(&prepared, &observe),
            attachment: CrashPointReplayObserverAttachment {
                points: CrashPointReplayObserverPoints::EveryPointOfTheFlow,
                code_file: CrashPointReplayObserverCodeFile::SharedFile(
                    FILE_OF_THE_COMPACTION_CRASH_POINT_OBSERVERS,
                ),
            },
        }],
        &[AttachedCrashPointReplayObserverHandingBackAnImage {
            observer: mount_continuation_observer(&prepared, &observe_the_mount),
            attachment: CrashPointReplayObserverAttachment {
                points: CrashPointReplayObserverPoints::EveryPointOfTheFlow,
                code_file: CrashPointReplayObserverCodeFile::SharedFile(
                    FILE_OF_THE_COMPACTION_CRASH_POINT_OBSERVERS,
                ),
            },
        }],
    )
    .judged() else {
        return;
    };
    println!(
        "{count_line_name} states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        &format!("{item_name}：用例自己的展开下的闭式"),
    );
    assert_every_state_was_observed(&prepared, &tally, &landed_counters(&prepared));
}

#[test]
fn full_enumeration_with_an_overwrite_between_the_input_and_the_relock_is_exhaustive_and_clean() {
    every_crash_state_with_the_batch_computed_out_of_the_lock_around_the_change_is_clean(
        ForegroundChangeWhileTheIntentIsLive::OverwriteOfTheFirstFile,
        "compaction-batch-out-of-the-lock-around-an-overwrite-full",
        "compaction_with_a_write_while_the_intent_is_live::batch_computed_out_of_the_lock_around_an_overwrite",
        "CRASH_POINT_REPLAY_COMPACTION_BATCH_COMPUTED_OUT_OF_THE_LOCK_AROUND_AN_OVERWRITE",
    );
}

#[test]
fn full_enumeration_with_an_unlink_between_the_input_and_the_relock_is_exhaustive_and_clean() {
    every_crash_state_with_the_batch_computed_out_of_the_lock_around_the_change_is_clean(
        ForegroundChangeWhileTheIntentIsLive::UnlinkOfTheFirstFile,
        "compaction-batch-out-of-the-lock-around-an-unlink-full",
        "compaction_with_a_write_while_the_intent_is_live::batch_computed_out_of_the_lock_around_an_unlink",
        "CRASH_POINT_REPLAY_COMPACTION_BATCH_COMPUTED_OUT_OF_THE_LOCK_AROUND_AN_UNLINK",
    );
}

/// 读的那个观察者看过每个状态（[`assert_every_state_was_observed`] 里读的那一半；可写挂载续做那一半归挂了它的那两条用例）：
/// 每个状态都落到流里某一版或 A 之前，流里每一种那一版都有状态，落到整理搬过单元之后那几版的状态抹掉源落点读过。
///
/// # Panics
/// 上面任何一样对不上。
fn assert_every_state_landed_on_the_flow_and_was_read_through_the_mapping(
    prepared: &PreparedCompactionCrashFlow,
    tally: &CrashPointReplayTally,
    landed_counters: &[String],
) {
    assert_eq!(tally.observed_states, tally.states, "每个状态都看过");
    let counts = &tally.observer_counts;
    assert_eq!(
        counts.count_named(LANDED_BEFORE_THE_FIRST_FILE)
            + landed_counters
                .iter()
                .map(|name| counts.count_named(name))
                .sum::<u64>(),
        tally.states,
        "每个状态都落到流里的某一版或 A 之前：{counts:?}"
    );
    for name in landed_counters {
        singlefs_verification::crash_state_enumeration::assert_the_count_is_above_zero_unless_narrowed(
            counts.count_named(name),
            &format!("{name} 的状态真跑到了：{counts:?}"),
        );
    }
    let landed_on_a_version_after_a_relocation: u64 = landed_counters
        .iter()
        .filter(|name| {
            prepared.versions_of_the_flow.iter().any(|version| {
                &landed_counter_of(version) == *name
                    && !version.relocation_sources_up_to_this_version.is_empty()
            })
        })
        .map(|name| counts.count_named(name))
        .sum();
    assert!(
        counts.count_named("read_through_the_mapping_after_the_relocation_sources_are_zeroed")
            >= landed_on_a_version_after_a_relocation.min(1),
        "落到整理搬过单元之后那几版的状态抹掉源落点读过：{counts:?}"
    );
}

/// 同一条流、同一组崩溃状态（每段都展开，同上面那两条登记用例）在 CPU 判器上逐状态判：照调用方的多机配置写一份只开 CPU 的
/// （`crash_library::multi_host_configuration_on_the_cpu_keeping_the_prune_server_of`：不开 GPU，判段交 CPU 判器；照抄 prune-server 的地址），
/// 环境守卫里把多机配置指过去；调用方设了 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`（指定范围的那几趟可缩域）照它缩，守卫清掉之前记下、清完设回。
/// 每个状态两遍恢复 + 多版本 oracle、池级 checker、记录核对器，挂读的那个观察者（恢复落到的那一版照映射读、抹掉源落点再读）；
/// 不挂可写挂载续做那个观察者：它交回挂完的盘，判段要在那张盘上判池级 checker，这一部分归上面那两条登记用例。块进全仓那一份库、照块键与判器组的单元键复用。
/// 状态数等于闭式（缩过域照缩域的判），判器零红，每个状态落到流里某一版或 A 之前。
// gpu-exempt: cpu-judge-mutation-rows
fn every_crash_state_with_the_batch_computed_out_of_the_lock_around_the_change_judged_on_the_cpu_judge_is_clean(
    change: ForegroundChangeWhileTheIntentIsLive,
    tag: &str,
    item_name: &str,
    count_line_name: &str,
) {
    let prepared = prepared_flow_of_the_recording(
        record_the_flow_around_the_foreground_change(
            tag,
            change,
            BatchAfterTheForegroundChange::ComputedOutOfTheLockAroundTheChange,
        )
        .recorded,
    );
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_landed_version_reading_through_the_mapping(
            versions_of_the_flow,
            image,
            report,
            counts,
        );
    };
    // 调用方的多机配置在守卫清掉 SINGLEFS_MULTI_HOST_CONFIG 之前取（仓副本里只经这个变量交进来）
    let configuration_of_the_caller =
        singlefs_verification::crash_amplification::multi_host_configuration_of_this_process()
            .unwrap_or_else(|refusal| panic!("{refusal}"))
            .configuration_file;
    let configuration_directory =
        ExclusiveTemporaryDirectory::create(tag).unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration_on_the_cpu = configuration_directory.join("multi-host-on-the-cpu.env");
    std::fs::write(
        &configuration_on_the_cpu,
        multi_host_configuration_on_the_cpu_keeping_the_prune_server_of(
            &configuration_of_the_caller,
        )
        .unwrap_or_else(|refusal| panic!("{refusal}")),
    )
    .expect("测试自己的临时目录里写得了只开 CPU 的多机配置");
    // 调用方缩过域的那一趟照它缩：守卫清掉一趟读的全部变量之前记下，清完设回（别的变量照清，用例只换多机配置）
    let narrowing_of_the_caller = std::env::var_os(
        singlefs_verification::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
    );
    // 先拿全仓那一份的守卫、再拿改环境的锁（`crash_library::TheOneLibraryKeptForThisProcess` 文档里的次序）；丢的时候倒过来
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration_on_the_cpu.as_os_str(),
    )]);
    if let Some(expand_up_to) = &narrowing_of_the_caller {
        environment.set_variable(
            singlefs_verification::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
            expand_up_to,
        );
    }
    let Some(tally) = enumerate_crash_states_with_attached_observers(
        CrashPointReplayItem {
            name: item_name,
            setting_file: PATH_OF_THIS_TEST_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        CrashPointReplayVersionTable::new(
            &prepared.versions_with_a_file,
            &prepared.versions_without_a_file,
        ),
        &full_expansion,
        &[AttachedCrashPointReplayObserver {
            observer: landed_version_observer(&prepared, &observe),
            attachment: CrashPointReplayObserverAttachment {
                points: CrashPointReplayObserverPoints::EveryPointOfTheFlow,
                code_file: CrashPointReplayObserverCodeFile::SharedFile(
                    FILE_OF_THE_COMPACTION_CRASH_POINT_OBSERVERS,
                ),
            },
        }],
        &[],
    )
    .judged() else {
        return;
    };
    println!(
        "{count_line_name} states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.observer_counts,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(
        tally.violations, 0,
        "oracle（CPU 判器）：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle（CPU 判器）：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败（CPU 判器）");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据（CPU 判器）"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例（CPU 判器）：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        &format!("{item_name}：用例自己的展开下的闭式"),
    );
    assert_every_state_landed_on_the_flow_and_was_read_through_the_mapping(
        &prepared,
        &tally,
        &landed_counters(&prepared),
    );
}

/// 覆盖写那一种：同一组崩溃状态在 CPU 判器上逐状态判。判完打计数行 `CRASH_POINT_REPLAY_COMPACTION_BATCH_COMPUTED_OUT_OF_THE_LOCK_AROUND_AN_OVERWRITE_ON_THE_CPU_JUDGE`，
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:compaction-batch-computed-out-of-the-lock-around-an-overwrite-on-the-cpu-judge`）。
/// 判别力（`crates/mutations.tsv`）：回锁不剔记录变了的单元时录流 panic；版本表里覆盖写之后几版留着覆盖写之前的内容时 oracle 与读的观察者红。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn full_enumeration_with_an_overwrite_between_the_input_and_the_relock_judged_on_the_cpu_judge_is_exhaustive_and_clean(
) {
    every_crash_state_with_the_batch_computed_out_of_the_lock_around_the_change_judged_on_the_cpu_judge_is_clean(
        ForegroundChangeWhileTheIntentIsLive::OverwriteOfTheFirstFile,
        "compaction-batch-out-of-the-lock-around-an-overwrite-on-the-cpu-judge",
        "compaction_with_a_write_while_the_intent_is_live::batch_computed_out_of_the_lock_around_an_overwrite_on_the_cpu_judge",
        "CRASH_POINT_REPLAY_COMPACTION_BATCH_COMPUTED_OUT_OF_THE_LOCK_AROUND_AN_OVERWRITE_ON_THE_CPU_JUDGE",
    );
}

/// 删文件那一种：同一组崩溃状态在 CPU 判器上逐状态判。判完打计数行 `CRASH_POINT_REPLAY_COMPACTION_BATCH_COMPUTED_OUT_OF_THE_LOCK_AROUND_AN_UNLINK_ON_THE_CPU_JUDGE`，
/// 登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:compaction-batch-computed-out-of-the-lock-around-an-unlink-on-the-cpu-judge`）。
/// 判别力（`crates/mutations.tsv`）：回锁把起点在 R 里的每个槽都当记录变了时录流 panic；版本表里删第一个文件之后几版照旧有文件时 oracle 与读的观察者红。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn full_enumeration_with_an_unlink_between_the_input_and_the_relock_judged_on_the_cpu_judge_is_exhaustive_and_clean(
) {
    every_crash_state_with_the_batch_computed_out_of_the_lock_around_the_change_judged_on_the_cpu_judge_is_clean(
        ForegroundChangeWhileTheIntentIsLive::UnlinkOfTheFirstFile,
        "compaction-batch-out-of-the-lock-around-an-unlink-on-the-cpu-judge",
        "compaction_with_a_write_while_the_intent_is_live::batch_computed_out_of_the_lock_around_an_unlink_on_the_cpu_judge",
        "CRASH_POINT_REPLAY_COMPACTION_BATCH_COMPUTED_OUT_OF_THE_LOCK_AROUND_AN_UNLINK_ON_THE_CPU_JUDGE",
    );
}
