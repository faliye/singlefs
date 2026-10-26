//! 跨请求攒批与屏障（判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节「一次发布里攒什么」「返回即落盘」「RefFS」三格；
//! D16（发布语义） 已定项 10）：数据写类并进一次发布（同一 inode 的写折叠、建 inode 求和），unlink、截断、建删快照、整理是屏障——进门先排空待发、
//! 自己单发；`accept` 进门先做这一批合起来的准入与条数预判，过不了在任何读写之前拒；整理仍只在后台入口里跑。
//! 攻方 P3 四处（`research/prompts/m4-publish-trigger-r1-opus-output.md`：删除类并进别的改动、攒批里没发布的 unlink 被写活、整理锚在一次调用、
//! 返回成功之后才拒）各一条；`accept` 与 `publish_when_due` 都不在调用线程上跑整理一条；原入口先排空待发一条；
//! RefFS 按会话报告的发布点判（攻方 P5 那十四格：今天「返回即落盘」的接法判红的合法态，新接法判绿；一个请求不被触发切开）两条；
//! 攒批会话的随机历史按会话报告的发布点与模型对拍、与 RefFS 判一条。
//!
//! 池在两块内存稀疏盘上起（`tests/common_admission_pool/mod.rs`：mkfs、取号、暖机、第一个文件），阈值经默认关的特性 `test-only-publish-thresholds`
//! 后面那个只供测试的来源给（这个测试包的 dev 依赖打开它）。
mod common_admission_pool;

use std::cell::Cell;
use std::num::NonZeroU64;
use std::time::Duration;

use common_admission_pool::{
    content_of, parameters_of_the_384_slot_small_devices, plain_device, PoolUnderTest,
    FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::allocator::{PoolAllocator, UnitAreaStart};
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionObservations,
    CompactionPacing, CompactionRunForcing, CompactionStartJudgement, CompactionSwitches,
    CompactionTimerFallback, CompactionWriteBandwidthLimit, ForegroundFsync,
};
use singlefs_core::mounted_session::{
    AcceptRefused, BackgroundCompactionCall, BackgroundCompactionTrigger, HowTheChangeWasTaken,
    MountedSession, MountedSessionWithPendingChanges, PublishTriggerCause, ReasonOfThePublish,
    UserChange,
};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::system_configuration::PublishTriggerThresholds;
use singlefs_core::transaction::{
    journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write,
    DataChangesPublishedTogether, FirstFile, NewInodesOfOneChange, PoolWriter, PublishError,
    TransactionOutput, WholeWriteOfOneInode, FIRST_INODE_NUMBER,
};
use singlefs_format::{INODE_LEAF_RECORDS, JOURNAL_RECORD_BYTES, SLOT_BYTES};
use singlefs_harness::history::{
    random_batching_session_calls, run_batching_session_calls, BatchingSessionCall,
    BatchingSessionCallSeen,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::model::{ModelFilesReadableAfterThePublishes, ModelOfThePendingChanges};
use singlefs_harness::reference_filesystem_translation::judge_a_crash_after_every_batching_session_call_returned;
use singlefs_reference_filesystem::{
    FileContent, Operation, OperationOutcome, ReferenceFileSystem, VisibleState,
};

/// 时间那一支不该先到的用例给的 `T_time`：一小时。
const TIME_THRESHOLD_NEVER_REACHED: Duration = Duration::from_secs(3600);

/// 脏量那一支不该先到的用例给的（夹之前的）`T_dirty`：夹成环长 ÷ F。
const DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP: u64 = 1 << 40;

/// 单元区放大到这么多槽的盘（6 MiB 的环、环窗口补足 1007 槽之外）：要发几十次、要压余量的用例用它，落点离单元区最高处远。
const UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES: u64 = 8192;

/// 小盘那一档为环窗口补足的槽数（抄自 `tests/common_admission_pool/mod.rs` 那一份私有常量，式子的来历写在 `crates/singlefs-harness/src/history.rs`）。
const RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;

fn reading(milliseconds: u64) -> CallerMonotonicClockReading {
    CallerMonotonicClockReading(Duration::from_millis(milliseconds))
}

/// 6 MiB 的环之后单元区 `unit_area_slots` 槽的盘的字节数。
fn device_bytes_with_a_unit_area_of(unit_area_slots: u64) -> u64 {
    let unit_area_start_slot = UnitAreaStart::following_the_journal_ring(
        parameters_of_the_384_slot_small_devices()
            .geometry
            .journal_ring_bytes,
    )
    .expect("6 MiB 的环末尾的下一个槽落在聚簇段边界上")
    .slot()
    .0;
    (unit_area_start_slot + RING_WINDOW_ALLOWANCE_SLOTS + unit_area_slots) * SLOT_BYTES
}

/// 起池（第一个文件之后）、交出会话，盘的单元区 `unit_area_slots` 槽。
fn pool_and_session(unit_area_slots: u64) -> (PoolUnderTest<SparseBlockDevice>, MountedSession) {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_with_a_unit_area_of(unit_area_slots),
        plain_device,
    );
    let session = pool.session.take().expect("起池之后有会话");
    (pool, session)
}

/// 会话接上待发，两个阈值取只供测试的值（脏量照环长夹）。
fn batching(
    session: MountedSession,
    journal_ring_bytes: u64,
    time_threshold: Duration,
    dirty_threshold_before_the_clamp: u64,
) -> MountedSessionWithPendingChanges {
    MountedSessionWithPendingChanges::of_the_session_with_thresholds(
        session,
        PublishTriggerThresholds::of_test_only_values_clamped_by_the_journal_ring(
            time_threshold,
            dirty_threshold_before_the_clamp,
            journal_ring_bytes,
        ),
    )
}

fn first_file_read_back(image: &MemoryPool) -> Vec<u8> {
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => content,
        other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
            panic!("第一个文件之后恢复读得出它：{other:?}")
        }
    }
}

fn txg_of(version: &TransactionOutput) -> u64 {
    version.root.checkpoint_txg.0
}

/// 一次发布里第一个文件之外的 `inode` 这一版的 nlink（不在树里是 `None`）。
fn nlink_of(version: &TransactionOutput, inode: u64) -> Option<u32> {
    version
        .inode_record_of(InodeNumber(inode))
        .map(|record| record.nlink)
}

/// 经攒批的会话发一次用户改动（先排空待发，这里待发是空的），交回发成的每一版。
fn publish_through_the_original_entry(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    session: &mut MountedSessionWithPendingChanges,
    change: UserChange<'_>,
    now: CallerMonotonicClockReading,
) -> Vec<TransactionOutput> {
    session
        .publish_user_change(&mut pool.devices, change, now)
        .expect("经原入口发成")
        .into_iter()
        .flat_map(|reported| reported.published.versions_published)
        .collect()
}

/// 建一个 inode、给它整份写 `data_bytes` 字节，都经原入口发成；交回它的号与写的内容。
fn create_and_write_an_inode_other_than_the_first(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    session: &mut MountedSessionWithPendingChanges,
    data_bytes: usize,
) -> (u64, Vec<u8>) {
    let inode = session
        .session()
        .current
        .version_of_trees()
        .expect("有树")
        .inode_number_watermark();
    publish_through_the_original_entry(
        pool,
        session,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1,
        },
        reading(0),
    );
    let content = content_of(data_bytes, 41);
    publish_through_the_original_entry(
        pool,
        session,
        UserChange::SequentialWriteOfAnInode {
            inode: InodeNumber(inode),
            content: &content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 2,
        },
        reading(0),
    );
    (inode, content)
}

/// 删除类的步不并进别的改动（攻方 P3-甲：D8（核心索引结构） 已定项 16 第 2 条「意图那次、每一批与续做的每一批各占一个事务号」）：
/// 收下一次覆盖写第一个文件（待发），再交 unlink 别的 inode：先排空待发成一次发布（第一个文件改了、那个 inode 还活着），unlink 再单发（意图、各批、最后一批），
/// unlink 的每一版都在排空那一版之后、都不重写第一个文件那条记录。
#[test]
fn an_unlink_is_published_alone_after_the_pending_changes_never_in_the_same_publish() {
    let (mut pool, session) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    let mut session = batching(
        session,
        pool.parameters.geometry.journal_ring_bytes,
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let (inode, _) =
        create_and_write_an_inode_other_than_the_first(&mut pool, &mut session, 3 * 30_000);
    let content = content_of(OVERWRITE_BYTES, 7);
    let accepted = session
        .accept(
            &mut pool.devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 3,
            }),
            reading(0),
        )
        .expect("收下覆盖写");
    assert!(accepted.publishes.is_empty(), "两支都没到：收下不发");
    let unlinked = session
        .accept(
            &mut pool.devices,
            UserChange::Unlink {
                inode: InodeNumber(inode),
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 4,
            },
            reading(0),
        )
        .expect("unlink 是屏障：先排空、再单发");
    assert_eq!(
        unlinked.taken,
        HowTheChangeWasTaken::PublishedAloneAsABarrier
    );
    let reasons: Vec<ReasonOfThePublish> = unlinked
        .publishes
        .iter()
        .map(|reported| reported.reason)
        .collect();
    assert_eq!(
        reasons,
        [
            ReasonOfThePublish::PendingChangesPublishedBeforeABarrier,
            ReasonOfThePublish::BarrierPublishedAlone
        ],
        "屏障进门先排空待发，自己再单发"
    );
    let drained = &unlinked.publishes[0].published.versions_published;
    assert_eq!(drained.len(), 1, "待发并成一次发布");
    let drained_version = &drained[0];
    let first_file_record = drained_version
        .inode_record_of(InodeNumber(FIRST_INODE_NUMBER))
        .expect("第一个文件活着");
    assert_eq!(
        first_file_record.change_count,
        txg_of(drained_version),
        "排空那一次写了第一个文件"
    );
    assert_ne!(
        nlink_of(drained_version, inode),
        Some(NLINK_OF_AN_INODE_BEING_DELETED),
        "排空那一次不碰 unlink 的那个 inode"
    );
    let barrier_versions = &unlinked.publishes[1].published.versions_published;
    assert!(
        barrier_versions.len() >= 2,
        "unlink 一次调用 = 意图 + 全部批"
    );
    for version in barrier_versions {
        assert!(
            txg_of(version) > txg_of(drained_version),
            "unlink 的每一版在排空之后"
        );
        assert_eq!(
            version
                .inode_record_of(InodeNumber(FIRST_INODE_NUMBER))
                .expect("第一个文件活着")
                .change_count,
            txg_of(drained_version),
            "unlink 的各版不重写第一个文件那条记录：它的改动不在删除类的发布里"
        );
    }
    assert_eq!(
        nlink_of(&barrier_versions[0], inode),
        Some(NLINK_OF_AN_INODE_BEING_DELETED),
        "意图那一次把那个 inode 记成删除中"
    );
}

/// 攒批里没发布的 unlink 不会被同一批里接着的写写活（攻方 P3-乙：队列与写入口都读现行那一版，没发布的 unlink 两处都看不见）：unlink 是屏障、
/// 收下就单发到最后一批，接着交给同一个 inode 的覆盖写在任何读写之前被拒（它已不在树里），盘上逐字节不变、待发不动。
#[test]
fn a_write_after_an_unlink_of_the_same_inode_is_refused_because_the_unlink_was_already_published() {
    let (mut pool, session) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    let mut session = batching(
        session,
        pool.parameters.geometry.journal_ring_bytes,
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let (inode, _) =
        create_and_write_an_inode_other_than_the_first(&mut pool, &mut session, 3 * 30_000);
    let unlinked = session
        .accept(
            &mut pool.devices,
            UserChange::Unlink {
                inode: InodeNumber(inode),
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 3,
            },
            reading(0),
        )
        .expect("unlink 单发");
    assert_eq!(
        unlinked.taken,
        HowTheChangeWasTaken::PublishedAloneAsABarrier
    );
    assert_eq!(
        nlink_of(
            session.session().current.version_of_trees().expect("有树"),
            inode
        ),
        None,
        "unlink 走到最后一批：那个 inode 已不在树里"
    );
    let image_before = pool.image();
    let content = content_of(OVERWRITE_BYTES, 9);
    let refused = session.accept(
        &mut pool.devices,
        UserChange::OverwriteOfAnInode {
            inode: InodeNumber(inode),
            content: &content,
            write_time_seconds: FIRST_WRITE_TIME_SECONDS + 4,
        },
        reading(0),
    );
    match refused {
        Err(AcceptRefused::ChangeRefusedByThePublishJudgement(cause)) => assert!(
            matches!(*cause, PublishError::WriteToAnInodeThatIsNotInTheTree { inode: refused_inode } if refused_inode == inode),
            "拒因是写一个不在树里的 inode：{cause:?}"
        ),
        other => panic!("写一个已经 unlink 完的 inode：收下之前就拒，实际 {other:?}"),
    }
    assert_eq!(
        pool.image(),
        image_before,
        "在任何读写之前拒：盘上逐字节不变"
    );
    assert_eq!(session.pending_changes_accepted(), 0, "待发不动");
}

/// 后台入口的调用方：钟读数由用例摆，前台没有 fsync 在等。
struct CallerForTests {
    clock_in_milliseconds: Cell<u64>,
}

impl CallerOfTheBackgroundCompaction for CallerForTests {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        reading(self.clock_in_milliseconds.get())
    }

    fn foreground_fsync(&self) -> ForegroundFsync {
        ForegroundFsync::NoneWaiting
    }
}

/// 会话装上整理只供测试的「起跑判据恒为真」，其余照产品值。
fn force_the_compaction_to_start(session: &mut MountedSession) {
    let CompactionSwitches {
        batch_size,
        forced_region,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size,
            forced_region,
            run_forcing: CompactionRunForcing::ForcedToStartForTests,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
}

/// 整理的保证不再锚在「一次调用发成」上（攻方 P3-丙）：收下改动不发布、不动 `B_gc`、不记「该跑了」；后台入口判过起跑之后、那一趟起跑之前先排空待发
/// （排空那一次报在前面、`B_gc` 从它重新数），整理一趟在它之后跑。
#[test]
fn the_background_compaction_publishes_the_pending_changes_before_its_run_and_an_accept_alone_moves_nothing_of_the_compaction(
) {
    let (mut pool, mut session) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    force_the_compaction_to_start(&mut session);
    let mut session = batching(
        session,
        pool.parameters.geometry.journal_ring_bytes,
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let compaction_before = session.session().compaction.clone();
    let content = content_of(OVERWRITE_BYTES, 11);
    let accepted = session
        .accept(
            &mut pool.devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1,
            }),
            reading(0),
        )
        .expect("收下");
    assert!(accepted.publishes.is_empty(), "收下不发");
    assert_eq!(
        session.session().compaction,
        compaction_before,
        "收下不动整理的状态：`B_gc` 不重数、不记「该跑了」"
    );
    let caller = CallerForTests {
        clock_in_milliseconds: Cell::new(10),
    };
    let ran = session
        .run_compaction_in_the_background(
            &mut pool.devices,
            CompactionPacing {
                timer_fallback: CompactionTimerFallback::EveryMilliseconds(
                    NonZeroU64::new(1).expect("1 不是 0"),
                ),
                write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
            },
            &caller,
        )
        .expect("后台入口");
    let drained = ran
        .pending_changes_published_before_the_run
        .expect("判过起跑之后、跑之前先排空待发");
    assert_eq!(
        drained.reason,
        ReasonOfThePublish::PendingChangesPublishedBeforeTheBackgroundCompaction
    );
    let drained_txg = txg_of(
        drained
            .published
            .versions_published
            .last()
            .expect("排空发了一版"),
    );
    match ran.call {
        BackgroundCompactionCall::Ran { trigger, run } => {
            assert_eq!(trigger, BackgroundCompactionTrigger::TimerFallback);
            for version in &run.versions_published {
                assert!(txg_of(version) > drained_txg, "整理的每一版在排空之后");
            }
        }
        BackgroundCompactionCall::NotStarted(reason) => {
            panic!("定时兜底查到起跑判据为真（强制起跑）：起跑，实际没起跑 {reason:?}")
        }
    }
    assert_eq!(session.pending_changes_accepted(), 0, "待发排空");
    assert_eq!(
        first_file_read_back(&pool.image()),
        content,
        "排空那一次写了收下的内容"
    );
}

/// 覆盖写第一个文件的那一次改动。
fn overwrite_of_the_first_file(content: &[u8], seconds: u64) -> UserChange<'_> {
    UserChange::Overwrite(FirstFile {
        content,
        write_time_seconds: FIRST_WRITE_TIME_SECONDS + seconds,
    })
}

/// `accept` 与 `publish_when_due` 都不在调用线程上跑整理（用户 2026-10-26 定「挪出调用线程」，GPU 垃圾回收会话第 39、45 批）：起跑判据强制为真，
/// 时间那一支在 `publish_when_due` 里触发、脏量那一支在 `accept` 里触发，两次发布各只发待发并成的那一版，整理一趟都没起跑、一个意图都没签，只记「该跑了」。
#[test]
fn neither_the_accept_nor_publish_when_due_runs_a_compaction_batch_on_the_calling_thread() {
    let (mut probe_pool, probe) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    let mut probe = batching(
        probe,
        probe_pool.parameters.geometry.journal_ring_bytes,
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let content = content_of(OVERWRITE_BYTES, 13);
    let HowTheChangeWasTaken::KeptPending {
        journal_bytes_of_this_change,
    } = probe
        .accept(
            &mut probe_pool.devices,
            overwrite_of_the_first_file(&content, 1),
            reading(0),
        )
        .expect("量一次覆盖写的 journal 字节")
        .taken
    else {
        panic!("覆盖写收进待发");
    };
    let (mut pool, mut session) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    force_the_compaction_to_start(&mut session);
    let mut session = batching(
        session,
        pool.parameters.geometry.journal_ring_bytes,
        Duration::from_millis(50),
        2 * journal_bytes_of_this_change.0,
    );
    let assert_no_compaction_ran =
        |session: &MountedSessionWithPendingChanges, versions: &[TransactionOutput]| {
            assert_eq!(versions.len(), 1, "只发待发并成的那一版");
            assert!(
                versions[0].units_relocated_by_this_publish.is_empty()
                    && versions[0].logged_operations.is_empty(),
                "那一版没有搬单元、logged_ops 里没有整理意图"
            );
            assert_eq!(
                session.session().compaction.observations,
                CompactionObservations::default(),
                "整理一趟都没起跑"
            );
            assert!(
                session
                    .session()
                    .allocator
                    .compaction()
                    .live_intent()
                    .is_none(),
                "一个意图都没签"
            );
            assert_eq!(
                session.session().compaction.start_judgement,
                CompactionStartJudgement::DueAfterAUserChange,
                "起跑判据为真只记「该跑了」"
            );
        };
    let accepted = session
        .accept(
            &mut pool.devices,
            overwrite_of_the_first_file(&content, 2),
            reading(0),
        )
        .expect("收下");
    assert!(accepted.publishes.is_empty());
    let when_due = session
        .publish_when_due(&mut pool.devices, reading(50))
        .expect("判一次")
        .expect("时间那一支到了：发");
    assert_eq!(
        when_due.reason,
        ReasonOfThePublish::TriggeredWhenDue(PublishTriggerCause::TimeThreshold)
    );
    assert_no_compaction_ran(&session, &when_due.published.versions_published);
    session
        .accept(
            &mut pool.devices,
            overwrite_of_the_first_file(&content, 3),
            reading(60),
        )
        .expect("收下");
    let accepted = session
        .accept(
            &mut pool.devices,
            overwrite_of_the_first_file(&content, 4),
            reading(60),
        )
        .expect("收下，脏量到了");
    assert_eq!(accepted.publishes.len(), 1);
    assert_eq!(
        accepted.publishes[0].reason,
        ReasonOfThePublish::TriggeredAfterTheAccept(PublishTriggerCause::DirtyThreshold)
    );
    assert_no_compaction_ran(
        &session,
        &accepted.publishes[0].published.versions_published,
    );
}

/// 分配器的一份拷贝，每块盘单元区最高处 `slots` 个槽记成隔离（只动隔离位，不写盘；做法同 `compaction_off_the_calling_thread_and_admission.rs`
/// 压准入余量那一份）：隔离的槽在单元区最高处，离发布要取的落点远。
fn allocator_with_the_highest_slots_isolated(
    allocator: &PoolAllocator,
    slots: u64,
) -> PoolAllocator {
    let mut isolated = allocator.clone();
    if slots == 0 {
        return isolated;
    }
    let ends: Vec<(DeviceIdentity, u64)> = isolated
        .devices
        .iter()
        .map(|device_map| {
            (
                device_map.device,
                device_map.unit_area_start().slot().0 + device_map.unit_area_slots(),
            )
        })
        .collect();
    for (device, unit_area_end) in ends {
        isolated.isolate_abandoned(device, SlotNumber(unit_area_end - slots), slots);
    }
    isolated
}

/// 这一批此刻接在会话现行那一版后面发，过不过得了落盘之前那几判（与 `accept` 进门那一判同一个函数），分配器用 `allocator`。
fn the_publish_judgement_admits(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    session: &MountedSession,
    allocator: &PoolAllocator,
    batch: DataChangesPublishedTogether<'_>,
) -> bool {
    let parameters = session.parameters_and_device_table.parameters_on_disk();
    let writer = PoolWriter::new(parameters, pool.devices.as_mut_slice());
    journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write(
        &writer,
        allocator,
        session.current.version_of_trees().expect("有树"),
        batch,
        session.instance,
    )
    .is_ok()
}

/// 建满一片 inode 叶容器那么多个 inode（树里原有第一个文件那一条，再建这么多个就分裂出第二片）。
const NEW_INODES_SPLITTING_THE_FIRST_LEAF_CONTAINER: u64 = INODE_LEAF_RECORDS;

/// 一批合起来的准入在 `accept` 进门判（攻方 P3-丁：一批的准入与条数上限只在发布时判，超了就是返回成功之后才拒）：在会话的分配器拷贝上从单元区最高处
/// 隔离槽，二分找到「覆盖写第一个文件并上一片叶容器那么多个建 inode」这一批刚好过不了落盘之前那几判的隔离数（两样单独各自过得了：并成一批多一个数据单元、
/// 叶容器多分裂一片），把它装上会话。先收下覆盖写（单独过得了），再交建 inode：它单独过得了，并进待发合起来过不了，在任何读写之前拒，
/// 盘上逐字节不变，待发里只有覆盖写那一次。
#[test]
fn a_change_that_fits_alone_but_not_together_with_the_pending_changes_is_refused_at_the_accept() {
    let content = content_of(OVERWRITE_BYTES, 17);
    let new_inodes = [NewInodesOfOneChange {
        count: NEW_INODES_SPLITTING_THE_FIRST_LEAF_CONTAINER,
        write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1,
    }];
    let whole_write = WholeWriteOfOneInode {
        inode: InodeNumber(FIRST_INODE_NUMBER),
        content: &content,
        write_time_seconds: FIRST_WRITE_TIME_SECONDS + 2,
    };
    let overwrite_alone = DataChangesPublishedTogether {
        whole_write: Some(whole_write),
        new_inodes: &[],
    };
    let new_inodes_alone = DataChangesPublishedTogether {
        whole_write: None,
        new_inodes: &new_inodes,
    };
    let together = DataChangesPublishedTogether {
        whole_write: Some(whole_write),
        new_inodes: &new_inodes,
    };
    let (mut pool, mut session) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    let allocator = session.allocator.clone();
    assert!(
        the_publish_judgement_admits(&mut pool, &session, &allocator, together),
        "不压余量时合起来那一批过得了"
    );
    let unit_area_slots = allocator.devices[0].unit_area_slots();
    // 二分：隔离 `admitted_with` 个槽时合起来那一批过得了，隔离 `refused_with` 个时过不了；可用随隔离单调减，判定随可用单调。
    let (mut admitted_with, mut refused_with) = (0_u64, unit_area_slots);
    // 迭代上界是 log2(单元区槽数) 轮；跨轮携带的是一过一不过的两端（夹紧不放松）。
    while refused_with - admitted_with > 1 {
        let middle = admitted_with + (refused_with - admitted_with) / 2;
        let isolated = allocator_with_the_highest_slots_isolated(&allocator, middle);
        if the_publish_judgement_admits(&mut pool, &session, &isolated, together) {
            admitted_with = middle;
        } else {
            refused_with = middle;
        }
    }
    let isolated = allocator_with_the_highest_slots_isolated(&allocator, refused_with);
    assert!(
        the_publish_judgement_admits(&mut pool, &session, &isolated, overwrite_alone)
            && the_publish_judgement_admits(&mut pool, &session, &isolated, new_inodes_alone),
        "隔离 {refused_with} 个槽：合起来刚好过不了，两样单独各自过得了"
    );
    session.allocator = isolated;
    let mut session = batching(
        session,
        pool.parameters.geometry.journal_ring_bytes,
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let accepted = session
        .accept(
            &mut pool.devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: whole_write.write_time_seconds,
            }),
            reading(0),
        )
        .expect("覆盖写单独过得了：收下");
    assert!(accepted.publishes.is_empty());
    let image_before = pool.image();
    let refused = session.accept(
        &mut pool.devices,
        UserChange::NewInodes {
            count: new_inodes[0].count,
            write_time_seconds: new_inodes[0].write_time_seconds,
        },
        reading(0),
    );
    assert!(
        matches!(
            refused,
            Err(AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(_))
        ),
        "并进待发合起来过不了：在收下这一刻拒，实际 {refused:?}"
    );
    assert_eq!(
        pool.image(),
        image_before,
        "在任何读写之前拒：盘上逐字节不变"
    );
    assert_eq!(
        session.pending_changes_accepted(),
        1,
        "待发里只有覆盖写那一次"
    );
}

/// 原入口语义不变、多一条先排空待发（判决第二节「返回即落盘」那一格）：收下一次覆盖写，再经原入口建一个 inode：先排空（覆盖写那一版），再单发建 inode，
/// 两次 txg 相连，排空那一版读出收下的内容。
#[test]
fn the_original_entry_publishes_the_pending_changes_before_the_user_change_it_is_handed() {
    let (mut pool, session) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    let mut session = batching(
        session,
        pool.parameters.geometry.journal_ring_bytes,
        TIME_THRESHOLD_NEVER_REACHED,
        DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
    );
    let content = content_of(OVERWRITE_BYTES, 19);
    session
        .accept(
            &mut pool.devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1,
            }),
            reading(0),
        )
        .expect("收下");
    let image_after_the_drain = {
        let reported = session
            .publish_user_change(
                &mut pool.devices,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIRST_WRITE_TIME_SECONDS + 2,
                },
                reading(0),
            )
            .expect("原入口发成");
        let reasons: Vec<ReasonOfThePublish> =
            reported.iter().map(|published| published.reason).collect();
        assert_eq!(
            reasons,
            [
                ReasonOfThePublish::PendingChangesPublishedBeforeTheUserChange,
                ReasonOfThePublish::UserChangePublishedAlone
            ]
        );
        let drained_txg = txg_of(&reported[0].published.versions_published[0]);
        assert_eq!(
            txg_of(&reported[1].published.versions_published[0]),
            drained_txg + 1,
            "先排空、再单发，两次相连"
        );
        pool.image()
    };
    assert_eq!(first_file_read_back(&image_after_the_drain), content);
    assert_eq!(session.pending_changes_accepted(), 0);
}

/// 收下第一个文件的一次覆盖写，交回这一次看到的（RefFS 翻译读它）。
fn accept_an_overwrite_seen(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    session: &mut MountedSessionWithPendingChanges,
    content: &[u8],
    milliseconds: u64,
) -> BatchingSessionCallSeen {
    let accepted = session
        .accept(
            &mut pool.devices,
            UserChange::Overwrite(FirstFile {
                content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + milliseconds,
            }),
            reading(milliseconds),
        )
        .expect("收下");
    BatchingSessionCallSeen {
        call: BatchingSessionCall::AcceptOverwriteOfTheFirstFile {
            length: content.len(),
            clock_advance_in_milliseconds: 0,
        },
        reading_in_milliseconds: milliseconds,
        first_file_content_accepted: Some(content.to_vec()),
        publishes_reported: accepted.publishes.len(),
    }
}

/// 今天的接法（每次改动返回就跟一个显式发布点，「返回即落盘」）判这一次崩溃：交回判没判红。
fn returned_is_durable_translation_judges_red(
    first_file_at_the_start: &[u8],
    contents_accepted: &[Vec<u8>],
    recovered: &[u8],
) -> bool {
    let mut reference_filesystem = ReferenceFileSystem::on_a_freshly_formatted_pool();
    reference_filesystem
        .submit(
            Operation::CreateTheFirstFile(FileContent(first_file_at_the_start.to_vec())),
            OperationOutcome::Succeeded,
        )
        .expect("建第一个文件");
    reference_filesystem
        .submit(Operation::ExplicitPublish, OperationOutcome::Succeeded)
        .expect("发布");
    for content in contents_accepted {
        reference_filesystem
            .submit(
                Operation::OverwriteTheFile(FileContent(content.clone())),
                OperationOutcome::Succeeded,
            )
            .expect("覆盖写");
        reference_filesystem
            .submit(Operation::ExplicitPublish, OperationOutcome::Succeeded)
            .expect("今天的接法：返回就是一个显式发布点");
    }
    let crash_falls_in = reference_filesystem.crash_after_every_operation_returned();
    reference_filesystem
        .crash_and_recover(
            crash_falls_in,
            VisibleState::OneFile(FileContent(recovered.to_vec())),
        )
        .is_err()
}

/// 攻方 P5 那十四格（覆盖写 1..=4 次、自动发布落在第 0..=k 次之后；0 是一次都没自动发布）：时间那一支在第 j 次收下之后触发（钟在那一次跳到
/// `T_time`），之后再收下的不发；全部返回之后崩溃、恢复读出第 j 次的内容（j = 0 读出第一个文件原来的内容）。按会话报告的发布点翻（每次发布一个
/// 显式发布点，收下不跟发布点）每一格判绿；今天「返回即落盘」的接法判红的恰好是自动发布落在最后一次之前的那十格。
// harness-test-granularity:one-scenario 攻方 P5 一张 4 × (k + 1) 的格表是一个场景：每格只差覆盖写次数与自动发布落点，各格新起一个池是为了互不影响，合起来核「十格今天红、新接法全绿」
#[test]
fn every_cell_the_returned_is_durable_translation_judges_red_is_green_with_the_publish_points_the_session_reports(
) {
    let mut red_under_the_returned_is_durable_translation = 0;
    let mut cells = 0;
    for overwrites in 1..=4_usize {
        for automatic_publish_after in 0..=overwrites {
            let (mut pool, session) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
            let first_file_at_the_start = first_file_read_back(&pool.image());
            let mut session = batching(
                session,
                pool.parameters.geometry.journal_ring_bytes,
                Duration::from_millis(1000),
                DIRTY_THRESHOLD_NEVER_REACHED_BEFORE_THE_CLAMP,
            );
            // 基线：第一次见到的读数 0 毫秒当上次发布的时刻。
            assert!(session
                .publish_when_due(&mut pool.devices, reading(0))
                .expect("判一次")
                .is_none());
            let contents: Vec<Vec<u8>> = (1..=overwrites)
                .map(|overwrite| {
                    content_of(
                        OVERWRITE_BYTES,
                        100 + u64::try_from(overwrite).expect("次数"),
                    )
                })
                .collect();
            let mut calls_seen = Vec::new();
            for (position, content) in contents.iter().enumerate() {
                let overwrite = position + 1;
                let milliseconds =
                    if automatic_publish_after == 0 || overwrite < automatic_publish_after {
                        0
                    } else {
                        1000
                    };
                let seen = accept_an_overwrite_seen(&mut pool, &mut session, content, milliseconds);
                assert_eq!(
                    seen.publishes_reported,
                    usize::from(overwrite == automatic_publish_after),
                    "覆盖写 {overwrites} 次、自动发布落在第 {automatic_publish_after} 次之后：第 {overwrite} 次"
                );
                calls_seen.push(seen);
            }
            let recovered = first_file_read_back(&pool.image());
            let expected = if automatic_publish_after == 0 {
                first_file_at_the_start.clone()
            } else {
                contents[automatic_publish_after - 1].clone()
            };
            assert_eq!(recovered, expected, "恢复读出最近一次发布那一版");
            judge_a_crash_after_every_batching_session_call_returned(
                &first_file_at_the_start,
                &calls_seen,
                VisibleState::OneFile(FileContent(recovered.clone())),
            )
            .unwrap_or_else(|red| {
                panic!("覆盖写 {overwrites} 次、自动发布落在第 {automatic_publish_after} 次之后：按会话报告的发布点判绿，实际 {red}")
            });
            if returned_is_durable_translation_judges_red(
                &first_file_at_the_start,
                &contents,
                &recovered,
            ) {
                red_under_the_returned_is_durable_translation += 1;
            }
            cells += 1;
        }
    }
    assert_eq!(cells, 14, "攻方 P5 那十四格");
    assert_eq!(
        red_under_the_returned_is_durable_translation, 10,
        "今天「返回即落盘」的接法在自动发布落在最后一次之前的十格判红（合法态）"
    );
}

/// 一个请求不被触发切开（判决第二节「RefFS」那一格：触发只在收下一个完整改动之后判）：脏量压到一条记录，收下一次跨三个数据单元的顺序写，
/// 收下之后在 `accept` 里按脏量发；那一次发布写出整份内容（读出的就是它、文件三个数据单元），RefFS 收到的是一个内容改动跟一个发布点，判绿。
#[test]
fn a_sequential_write_across_three_units_is_published_whole_by_the_trigger_it_sets_off() {
    let (mut pool, session) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
    let first_file_at_the_start = first_file_read_back(&pool.image());
    let mut session = batching(
        session,
        pool.parameters.geometry.journal_ring_bytes,
        TIME_THRESHOLD_NEVER_REACHED,
        JOURNAL_RECORD_BYTES,
    );
    let payload = usize::try_from(singlefs_harness::model::data_unit_payload_capacity_in_bytes())
        .expect("净荷装得进 usize");
    let content = content_of(2 * payload + 5, 23);
    let accepted = session
        .accept(
            &mut pool.devices,
            UserChange::SequentialWrite(FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1,
            }),
            reading(0),
        )
        .expect("收下");
    let HowTheChangeWasTaken::KeptPending {
        journal_bytes_of_this_change,
    } = accepted.taken
    else {
        panic!("顺序写收进待发");
    };
    assert!(
        journal_bytes_of_this_change.0 >= 3 * JOURNAL_RECORD_BYTES,
        "三个一单元事务至少三条记录：{journal_bytes_of_this_change:?}"
    );
    assert_eq!(accepted.publishes.len(), 1, "收下这一整个请求之后才判：发");
    assert_eq!(
        accepted.publishes[0].reason,
        ReasonOfThePublish::TriggeredAfterTheAccept(PublishTriggerCause::DirtyThreshold)
    );
    let version = &accepted.publishes[0].published.versions_published[0];
    assert_eq!(
        version.data_pointers.len(),
        3,
        "那一次发布写出了整份内容的三个数据单元"
    );
    let recovered = first_file_read_back(&pool.image());
    assert_eq!(recovered, content, "读出整份内容");
    judge_a_crash_after_every_batching_session_call_returned(
        &first_file_at_the_start,
        &[BatchingSessionCallSeen {
            call: BatchingSessionCall::AcceptSequentialWriteOfTheFirstFile {
                length: content.len(),
                clock_advance_in_milliseconds: 0,
            },
            reading_in_milliseconds: 0,
            first_file_content_accepted: Some(content.clone()),
            publishes_reported: accepted.publishes.len(),
        }],
        VisibleState::OneFile(FileContent(recovered)),
    )
    .expect("RefFS：一个内容改动跟一个发布点，读出它，判绿");
}

/// 攒批会话的随机历史（判决第二节「返回即落盘」那一格：随机历史、模型对拍、RefFS 翻译按会话报告的发布点判）：几颗种子各跑一段（收下覆盖写、顺序写、建 inode，
/// 按周期判、立即排空，钟随机往前走），每次调用之后会话报的发布与模型判的触发一致、报了发布就与模型比、池级 checker 不红；全部返回之后崩溃，
/// 恢复读出的第一个文件在 RefFS 按会话报告的发布点算出的允许集合里。几颗种子合起来时间那一支、脏量那一支、立即排空都走到过。
// harness-test-granularity:one-scenario 同一个随机历史场景的几颗种子：各新起一个池互不影响，合起来核三种发布理由都走到
#[test]
fn random_batching_histories_agree_with_the_model_at_every_reported_publish_and_with_the_reference_filesystem(
) {
    const SEEDS: [u64; 3] = [1, 2, 3];
    const CALLS_OF_ONE_HISTORY: usize = 36;
    let mut by_time = 0;
    let mut by_dirty = 0;
    let mut on_request = 0;
    for seed in SEEDS {
        let (mut pool, session) = pool_and_session(UNIT_AREA_SLOTS_OF_THE_LARGER_DEVICES);
        let first_file_at_the_start = first_file_read_back(&pool.image());
        let version = session.current.version_of_trees().expect("有树");
        let published = ModelFilesReadableAfterThePublishes {
            contents_by_inode: [(FIRST_INODE_NUMBER, first_file_at_the_start.clone())]
                .into_iter()
                .collect(),
            inode_number_watermark: version.inode_number_watermark(),
        };
        let time_threshold_in_milliseconds = 40;
        let dirty_threshold_in_journal_bytes = 4 * JOURNAL_RECORD_BYTES;
        let mut session = batching(
            session,
            pool.parameters.geometry.journal_ring_bytes,
            Duration::from_millis(time_threshold_in_milliseconds),
            dirty_threshold_in_journal_bytes,
        );
        let mut model = ModelOfThePendingChanges::after_a_publish(
            published,
            time_threshold_in_milliseconds,
            dirty_threshold_in_journal_bytes,
        );
        let calls = random_batching_session_calls(seed, CALLS_OF_ONE_HISTORY, 25);
        let run = run_batching_session_calls(
            &mut session,
            &mut pool.devices,
            &|devices| MemoryPool {
                devices: devices
                    .iter()
                    .map(|(identity, device)| (*identity, device.image.clone()))
                    .collect(),
                device_size_in_bytes: pool.device_bytes,
            },
            &calls,
            &mut model,
            FIRST_WRITE_TIME_SECONDS + 1,
        )
        .unwrap_or_else(|disagreement| panic!("种子 {seed}：{disagreement}"));
        by_time += run.publishes_triggered_by_time + run.publishes_triggered_by_both;
        by_dirty += run.publishes_triggered_by_the_dirty_journal_bytes;
        on_request += run.publishes_on_request;
        judge_a_crash_after_every_batching_session_call_returned(
            &first_file_at_the_start,
            &run.calls_seen,
            VisibleState::OneFile(FileContent(first_file_read_back(&pool.image()))),
        )
        .unwrap_or_else(|red| panic!("种子 {seed}：{red}"));
    }
    assert!(
        by_time > 0 && by_dirty > 0 && on_request > 0,
        "几颗种子合起来三种理由都走到过：时间 {by_time}、脏量 {by_dirty}、立即排空 {on_request}"
    );
}
