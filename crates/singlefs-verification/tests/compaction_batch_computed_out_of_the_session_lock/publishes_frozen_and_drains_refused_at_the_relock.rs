//! 回锁时发布冻结与排空被拒：窗口里冻结的发布之后回锁的批丢掉、整理不失败，取输入时已经冻结的批在回锁时丢掉、整理不失败，回锁排空待发被拒时交回拒绝、丢掉这一批；
//! 窗口里冻结了一次用户改动、回锁出错那一支删意图被冻结拒时，冻结的那一次照旧是用户改动、重发成了照用户改动记账，会话记下没删成的意图、
//! 下一次调用先删它；回锁出错那一支删意图自己写盘报错冻结、重发成了之后，会话记着的那一条已不是现行那一版的活意图，下一次取输入忘掉它、不停在它上面。

use super::*;
use singlefs_core::background_compute::{
    BackgroundGpuComputeFailed, BackgroundTaskAfterTheComputeError,
};
use singlefs_core::transaction::KindOfTheFrozenPublish;

/// 写到这一刻摆了故障就报错（报错时什么都不写）的盘：锁外期间前台那一次发布写盘报错、冻结。
pub(super) struct DeviceFailingWritesWhileTheFaultIsSet {
    device: Recorded,
    fault: Arc<std::sync::atomic::AtomicBool>,
}

impl BlockDevice for DeviceFailingWritesWhileTheFaultIsSet {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.device.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        if self.fault.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                std::io::Error::other("用例摆的写错"),
            ));
        }
        self.device.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.device.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<
        singlefs_core::block_device::DiscardOutcome,
        singlefs_core::block_device::BlockDeviceError,
    > {
        self.device.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.device.barrier()
    }

    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.device.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.device.size_in_bytes()
    }
}

/// 盘套上可摆写错的那一层的盘表。
pub(super) type DevicesFailingWritesWhileTheFaultIsSet =
    Vec<(DeviceIdentity, DeviceFailingWritesWhileTheFaultIsSet)>;

/// 重开、盘套上可摆写错的那一层：交回盘表与开关（起先不摆）。
fn devices_failing_writes_while_the_fault_is_set(
    pool: &mut BuiltPool,
) -> (
    DevicesFailingWritesWhileTheFaultIsSet,
    Arc<std::sync::atomic::AtomicBool>,
) {
    let fault = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let devices: DevicesFailingWritesWhileTheFaultIsSet = pool
        .reopen_recorded()
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceFailingWritesWhileTheFaultIsSet {
                    device,
                    fault: Arc::clone(&fault),
                },
            )
        })
        .collect();
    (devices, fault)
}

/// 重开、盘套上可摆写错的那一层（开关交回，起先不摆）、可写挂载（选 `gpu`，GPU 实现是背后照 CPU 写法算的假实现）、套上攒批的会话：
/// 交回会话、盘、开关与用例留着的那一份实现。整理的开关照产品值，由用例再装。
pub(super) fn session_on_the_gpu_on_devices_failing_writes_while_the_fault_is_set(
    pool: &mut BuiltPool,
) -> (
    MountedSessionWithPendingChanges,
    DevicesFailingWritesWhileTheFaultIsSet,
    Arc<std::sync::atomic::AtomicBool>,
    Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
) {
    let shared = Arc::new(Mutex::new(
        CpuBackedBackgroundGpuComputeForTests::with_nothing_injected(),
    ));
    let handle = BackgroundGpuComputeHandle::of_a_shared_implementation(shared.clone());
    let (mut devices, fault) = devices_failing_writes_while_the_fault_is_set(pool);
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        admissible_gpu_settings_naming_both_cards(),
        Some(handle),
    )
    .expect("选 gpu 的可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (
        MountedSessionWithPendingChanges::of_the_session(session),
        devices,
        fault,
        shared,
    )
}

/// 建一个 inode（一次用户改动，强制起跑时记下「该跑了」；盘套上可摆写错的那一层，这一刻没摆写错）。
pub(super) fn create_one_inode_on_devices_failing_writes(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut DevicesFailingWritesWhileTheFaultIsSet,
    caller: &CallerForTests,
) {
    pending
        .publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            caller.clock_reading(),
        )
        .expect("建一个 inode");
}

/// 在 `region` 上签一条意图（同 [`issue_an_intent_on`] 的取法，盘套上可摆写错的那一层），现行那一版换成签发那一版；交回那条意图。
pub(super) fn issue_an_intent_on_devices_failing_writes(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut DevicesFailingWritesWhileTheFaultIsSet,
    region: CompactionRegion,
) -> CompactionIntent {
    let session = pending.session_for_tests_only();
    let previous = current_of(session).clone();
    let intent = CompactionIntent {
        region,
        allocation_generation_at_issue: previous.root.checkpoint_txg,
        empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
            &previous.allocation_records,
            unit_area_of_the_first_device(session),
        ),
    };
    session.allocator.compaction_mut().issue(intent);
    let publish_parameters = parameters();
    let issued = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_compaction_step(
            &mut writer,
            &mut session.allocator,
            &previous,
            &[],
            CompactionIntentStep::Issue(intent),
            session.instance,
        )
        .expect("签发")
    };
    session.current = PoolVersion::of_a_version_of_trees(issued);
    intent
}

/// 重开、盘套上可摆写错的那一层（开关交回，起先不摆）、可写挂载（`cpu`）、套上攒批的会话；强制起跑、强制选第一个文件那次发布的段、一批一个，
/// 建一个 inode（「该跑了」）。
fn session_on_devices_failing_writes_while_the_fault_is_set(
    pool: &mut BuiltPool,
    caller: &CallerForTests,
) -> (
    MountedSessionWithPendingChanges,
    DevicesFailingWritesWhileTheFaultIsSet,
    Arc<std::sync::atomic::AtomicBool>,
) {
    let (mut devices, fault) = devices_failing_writes_while_the_fault_is_set(pool);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pending = MountedSessionWithPendingChanges::of_the_session(session);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH)),
        1,
    );
    pending
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            caller.clock_reading(),
        )
        .expect("建一个 inode");
    (pending, devices, fault)
}

/// 摆上写错、建一个 inode（那一次发布写盘报错、冻结），再撤掉写错。交回那一次被拒、冻结着没有。
fn froze_a_publish_of_a_user_change(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut DevicesFailingWritesWhileTheFaultIsSet,
    fault: &std::sync::atomic::AtomicBool,
    caller: &CallerForTests,
) -> bool {
    fault.store(true, std::sync::atomic::Ordering::SeqCst);
    let refused = pending.publish_user_change(
        devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
        },
        caller.clock_reading(),
    );
    fault.store(false, std::sync::atomic::Ordering::SeqCst);
    refused.is_err() && pending.session().allocator.frozen_publish().is_some()
}

/// 取输入：这一批交到锁外，交回票（盘套上可摆写错的那一层；在锁里就走完了、被拒都 panic）。
fn ticket_taken_on_devices_failing_writes(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut DevicesFailingWritesWhileTheFaultIsSet,
    caller: &CallerForTests,
) -> Box<BackgroundCompactionBatchTicket> {
    match pending
        .take_the_input_of_a_background_compaction_batch(
            devices,
            pacing_with_the_timer_fallback_off(),
            CompactionYieldFloor::NO_FLOOR,
            caller,
        )
        .expect("取输入")
    {
        BackgroundCompactionInputTaken::BatchOutOfTheLock { ticket, .. } => ticket,
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            panic!("这一批应当交到锁外：{:?}", after.call)
        }
    }
}

/// 回锁时有冻结着的发布（锁外期间前台那一次发布写盘报错）：这一批丢掉、停在 `BatchDroppedWhileAPublishIsFrozenAtTheRelock`，
/// 一个写都不发，不记成整理失败，这一趟还在途。
#[test]
fn a_batch_relocked_while_a_publish_frozen_in_the_window_is_dropped_without_failing_the_compaction()
{
    let mut pool = build_pool("relock-drops-the-batch-while-a-publish-is-frozen");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, fault) =
        session_on_devices_failing_writes_while_the_fault_is_set(&mut pool, &caller);
    let ticket = ticket_taken_on_devices_failing_writes(&mut pending, &mut devices, &caller);
    assert!(
        froze_a_publish_of_a_user_change(&mut pending, &mut devices, &fault, &caller),
        "前提：锁外期间那一次发布写盘报错、冻结着"
    );
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let computed = ticket.compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    });
    let after = pending
        .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
        .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份");
    let run = run_of(after);
    assert_eq!(
        stop_of(&run),
        CompactionRunStop::BatchDroppedWhileAPublishIsFrozenAtTheRelock,
        "冻结着就丢这一批，不记成整理失败"
    );
    assert!(run.versions_published.is_empty(), "一个写都没发");
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock
    );
    assert_eq!(
        pending
            .session()
            .compaction
            .observations
            .batches_dropped_while_a_publish_is_frozen_at_the_relock,
        1
    );
    assert!(
        pending
            .session()
            .compaction
            .has_a_background_trip_under_way(),
        "这一趟还在途：重发成了之后接着推"
    );
}

/// 取输入之前就冻结着一次发布（同一拍里 `publish_when_due` 写盘报错冻结之后接着取输入的那一格），锁外那一段期间一直冻着同一次：
/// 回锁时有冻结的发布就丢掉这一批（D26（后台整理与放置回收） 已定项 13 第 5 条，不问冻结的是不是取输入之后新冻的）——停在
/// `BatchDroppedWhileAPublishIsFrozenAtTheRelock`、记一次观测，一个写都不发，不记成整理失败，这一趟还在途。
#[test]
fn a_batch_taken_while_a_publish_is_already_frozen_is_dropped_at_the_relock_without_failing_the_compaction(
) {
    let mut pool = build_pool("relock-drops-the-batch-taken-while-a-publish-is-frozen");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, fault) =
        session_on_devices_failing_writes_while_the_fault_is_set(&mut pool, &caller);
    assert!(
        froze_a_publish_of_a_user_change(&mut pending, &mut devices, &fault, &caller),
        "前提：取输入之前那一次发布写盘报错、冻结着"
    );
    let txg_of_the_frozen_publish_at_the_input = pending
        .session()
        .allocator
        .frozen_publish()
        .map(|frozen| frozen.checkpoint_txg());
    let ticket = ticket_taken_on_devices_failing_writes(&mut pending, &mut devices, &caller);
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let computed = ticket.compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    });
    assert_eq!(
        pending
            .session()
            .allocator
            .frozen_publish()
            .map(|frozen| frozen.checkpoint_txg()),
        txg_of_the_frozen_publish_at_the_input,
        "前提：回锁时冻着的就是取输入之前那一次"
    );
    let after = pending
        .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
        .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份");
    let run = run_of(after);
    assert_eq!(
        stop_of(&run),
        CompactionRunStop::BatchDroppedWhileAPublishIsFrozenAtTheRelock,
        "冻结着就丢这一批，不记成整理失败"
    );
    assert!(run.versions_published.is_empty(), "一个写都没发");
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock
    );
    assert_eq!(
        pending
            .session()
            .compaction
            .observations
            .batches_dropped_while_a_publish_is_frozen_at_the_relock,
        1
    );
    assert!(
        pending
            .session()
            .compaction
            .has_a_background_trip_under_way(),
        "这一趟还在途：重发成了之后接着推"
    );
}

/// 回锁里发这一批之前排空待发被拒（锁外期间前台收下一次改动，回锁排空时那一次写盘报错）：回锁交回 `PublishOfThePendingChangesRefused`，
/// 这一批不发、丢掉，这个会话不再记有一批在锁外，现行那一版没动（照取输入之前排空被拒那一支办）。
#[test]
fn a_relock_whose_drain_of_the_pending_changes_is_refused_hands_back_the_refusal_and_drops_the_batch(
) {
    let mut pool = build_pool("relock-drain-of-the-pending-changes-refused");
    let caller = CallerForTests::with_nothing_waiting();
    let (mut pending, mut devices, fault) =
        session_on_devices_failing_writes_while_the_fault_is_set(&mut pool, &caller);
    let ticket = ticket_taken_on_devices_failing_writes(&mut pending, &mut devices, &caller);
    pending
        .accept(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            caller.clock_reading(),
        )
        .expect("锁外期间前台收下一次改动、进待发");
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let computed = ticket.compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    });
    fault.store(true, std::sync::atomic::Ordering::SeqCst);
    let relocked_with_the_drain_refused =
        pending.publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller);
    fault.store(false, std::sync::atomic::Ordering::SeqCst);
    assert!(
        matches!(
            relocked_with_the_drain_refused,
            Err(BackgroundCompactionRelockRefused::PublishOfThePendingChangesRefused(_))
        ),
        "回锁排空待发被拒：交回排空被拒：{:?}",
        relocked_with_the_drain_refused
            .as_ref()
            .map(|_after| "收下了这一批")
    );
    assert!(
        !pending.session().compaction.has_a_batch_out_of_the_lock(),
        "这一批丢掉：会话上不再记有一批在锁外"
    );
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock,
        "这一批一个写都没发"
    );
}

/// 分配器上冻结着的那一次发布记的种类；没有冻结着的发布是 `None`。
fn kind_of_the_frozen_publish(
    pending: &MountedSessionWithPendingChanges,
) -> Option<KindOfTheFrozenPublish> {
    pending
        .session()
        .allocator
        .frozen_publish()
        .map(|frozen| frozen.kind())
}

/// 现行那一版里的活意图；没有是 `None`。
fn live_intent_of(pending: &MountedSessionWithPendingChanges) -> Option<CompactionIntent> {
    live_compaction_intent_of_the_entries(&current_of(pending.session()).logged_operations)
        .map(|(_issued_txg, intent)| intent)
}

/// 锁外那一批算着出错的那几格共用的起法（[`batch_failing_on_the_gpu_under_a_live_intent_out_of_the_lock`]）交回的：池、会话、盘、写错的开关、替身、
/// 那条活意图与交到锁外的那一批的票。
struct BatchFailingOnTheGpuUnderALiveIntentOutOfTheLock {
    pool: BuiltPool,
    pending: MountedSessionWithPendingChanges,
    devices: DevicesFailingWritesWhileTheFaultIsSet,
    fault: Arc<std::sync::atomic::AtomicBool>,
    shared: Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
    intent: CompactionIntent,
    ticket: Box<BackgroundCompactionBatchTicket>,
}

/// 锁外那一批算着出错的那几格共用的起法：选 `gpu`、盘套上可摆写错的那一层、强制起跑（回退门不判）、一批罩得住一整段，在单元区头一段签一条意图、
/// 建一个 inode（「该跑了」）；GPU 实现在区域扫描里的单元头解析与映射反查那一段交回 `failure`，取输入（这一批交到锁外，算之前停在这里）。
fn batch_failing_on_the_gpu_under_a_live_intent_out_of_the_lock(
    tag: &str,
    failure: BackgroundGpuComputeFailed,
    caller: &CallerForTests,
) -> BatchFailingOnTheGpuUnderALiveIntentOutOfTheLock {
    let mut pool = build_pool(tag);
    let (mut pending, mut devices, fault, shared) =
        session_on_the_gpu_on_devices_failing_writes_while_the_fault_is_set(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        None,
        UNITS_OF_A_BATCH_COVERING_A_SEGMENT,
    );
    let intent = issue_an_intent_on_devices_failing_writes(
        &mut pending,
        &mut devices,
        region_starting_at(SEGMENT_OF_THE_FIRST_DATA_UNIT),
    );
    create_one_inode_on_devices_failing_writes(&mut pending, &mut devices, caller);
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = Some((GpuComputedSegment::RegionRecordClassification, failure));
    let ticket = ticket_taken_on_devices_failing_writes(&mut pending, &mut devices, caller);
    BatchFailingOnTheGpuUnderALiveIntentOutOfTheLock {
        pool,
        pending,
        devices,
        fault,
        shared,
        intent,
        ticket,
    }
}

/// 锁外那一段里前台一次用户改动写盘报错冻结、锁外那一批算着出错（卡当场报出错）：回锁出错那一支单发的删意图被冻结拒（D26（后台整理与放置回收）
/// 已定项 13 第 4 条）。冻着的是锁外期间前台那一次用户改动，不是回锁这一次调用发的：种类照旧是用户改动（第 6 条「冻结的是用户改动，照一次用户改动发成办」）；
/// 撤写错、重发成了照一次用户改动发成记账——上一次用户改动之后的整理发布数归零（预算重给），用例摆的上一条完成的意图清掉。
#[test]
fn a_user_change_frozen_in_the_window_keeps_its_kind_when_the_drop_after_a_compute_error_at_the_relock_is_refused_by_the_freeze(
) {
    let caller = CallerForTests::with_nothing_waiting();
    let BatchFailingOnTheGpuUnderALiveIntentOutOfTheLock {
        pool: _pool,
        mut pending,
        mut devices,
        fault,
        shared,
        intent: abandoned_intent,
        ticket,
    } = batch_failing_on_the_gpu_under_a_live_intent_out_of_the_lock(
        "relock-drop-refused-by-a-user-change-frozen-in-the-window-keeps-its-kind",
        device_lost_on_the_card_with_eight_gibibytes(),
        &caller,
    );
    assert!(
        froze_a_publish_of_a_user_change(&mut pending, &mut devices, &fault, &caller),
        "前提：锁外期间前台那一次用户改动写盘报错、冻结着"
    );
    assert_eq!(
        kind_of_the_frozen_publish(&pending),
        Some(KindOfTheFrozenPublish::UserChange),
        "前提：冻结着的是那一次用户改动"
    );
    let computed = ticket.compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    });
    let run = run_of(
        pending
            .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
            .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份"),
    );
    assert!(
        matches!(run.outcome, CompactionRunOutcome::Failed(_)) && run.versions_published.is_empty(),
        "前提：回锁出错那一支删意图被冻结拒、一个写都没发：{:?}",
        run.outcome
    );
    assert_eq!(
        kind_of_the_frozen_publish(&pending),
        Some(KindOfTheFrozenPublish::UserChange),
        "冻着的是锁外期间前台那一次用户改动、不是回锁这一次调用发的：种类照旧是用户改动"
    );
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = None;
    // 上一条完成的意图：用例摆一条（另一段上的），看重发成了之后清没清。
    pending
        .session_for_tests_only()
        .compaction
        .last_completed_intent = Some(CompactionIntent {
        region: region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH),
        allocation_generation_at_issue: abandoned_intent.allocation_generation_at_issue,
        empty_cluster_segments_at_issue: abandoned_intent.empty_cluster_segments_at_issue,
    });
    assert!(
        pending
            .resend_the_frozen_publish(&mut devices)
            .expect("撤了写错：重发成了"),
        "前提：有冻结着的发布、重发了"
    );
    assert_eq!(
        pending
            .session()
            .compaction
            .publishes_since_the_last_user_change,
        0,
        "照一次用户改动发成记账：预算重给"
    );
    assert_eq!(
        pending.session().compaction.last_completed_intent,
        None,
        "照一次用户改动发成记账：上一条完成的意图清掉"
    );
}

/// 一批里同一个条目键出现两次（只放弃这一批、整理不停）、锁外那一段里前台一次用户改动写盘报错冻结：回锁出错那一支删意图被冻结拒，
/// 会话照门压住那一支记下这条活意图（D26（后台整理与放置回收） 已定项 13 第 4 条「正在算的那一批放弃，意图也删掉」、第 8 条重复键「那一批的意图照第 4 条
/// 一批出错放弃的规矩删」）；撤写错、重发成了、前台再建一个 inode（「该跑了」）之后，下一次取输入不交一批到锁外：先单发一次只删意图的发布（不搬单元），
/// 这一趟停在放弃那一批时停的那一条，意图与 R 排他撤了、会话不再记着它，不在那条放弃的意图下接着搬。
#[test]
fn after_the_same_entry_key_twice_the_drop_refused_by_a_user_change_frozen_in_the_window_is_published_before_any_relocation_under_the_intent(
) {
    let caller = CallerForTests::with_nothing_waiting();
    let BatchFailingOnTheGpuUnderALiveIntentOutOfTheLock {
        pool: _pool,
        mut pending,
        mut devices,
        fault,
        shared,
        intent: abandoned_intent,
        ticket,
    } = batch_failing_on_the_gpu_under_a_live_intent_out_of_the_lock(
        "relock-drop-refused-by-a-user-change-frozen-in-the-window-is-noted",
        BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
            position_of_the_first_occurrence: 1,
            position_of_the_second_occurrence: 3,
        },
        &caller,
    );
    assert!(
        froze_a_publish_of_a_user_change(&mut pending, &mut devices, &fault, &caller),
        "前提：锁外期间前台那一次用户改动写盘报错、冻结着"
    );
    let computed = ticket.compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    });
    let at_the_relock = run_of(
        pending
            .publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller)
            .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份"),
    );
    assert!(
        matches!(at_the_relock.outcome, CompactionRunOutcome::Failed(_))
            && at_the_relock.versions_published.is_empty(),
        "前提：回锁出错那一支删意图被冻结拒、一个写都没发：{:?}",
        at_the_relock.outcome
    );
    assert_eq!(
        live_intent_of(&pending),
        Some(abandoned_intent),
        "前提：删意图没发成，意图留在现行那一版里"
    );
    assert!(
        pending
            .session()
            .compaction
            .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "删意图没发成：会话照门压住那一支记下这条活意图"
    );
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = None;
    assert!(
        pending
            .resend_the_frozen_publish(&mut devices)
            .expect("撤了写错：重发成了"),
        "前提：有冻结着的发布、重发了"
    );
    create_one_inode_on_devices_failing_writes(&mut pending, &mut devices, &caller);
    let after_the_resend = match pending
        .take_the_input_of_a_background_compaction_batch(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            CompactionYieldFloor::NO_FLOOR,
            &caller,
        )
        .expect("没有一批在锁外、盘表是挂载的那一份")
    {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => run_of(after),
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => {
            panic!("会话记着没删成的那一条：先删它，不在它下面再交一批到锁外接着搬")
        }
    };
    assert_eq!(
        after_the_resend.versions_published.len(),
        1,
        "单发一次只删意图的发布：{:?}",
        after_the_resend.outcome
    );
    assert!(
        after_the_resend.versions_published[0]
            .units_relocated_by_this_publish
            .is_empty(),
        "删意图那一次一个单元都不搬"
    );
    assert_eq!(
        stop_of(&after_the_resend),
        CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
            task_after_the_error:
                BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned,
        },
        "删它那一趟停在放弃那一批时停的那一条"
    );
    assert!(
        live_intent_of(&pending).is_none()
            && pending
                .session()
                .allocator
                .compaction()
                .live_intent()
                .is_none(),
        "意图删了、R 排他撤了：之后没有这条意图可接着搬"
    );
    assert!(
        !pending
            .session()
            .compaction
            .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "删成之后会话不再记着它"
    );
}

/// 会话记着的待删意图已不是现行那一版里的活意图时，取下一批之前忘掉它、不照它删（D26（后台整理与放置回收） 已定项 13 第 5 条：取下一批的输入之前先删的
/// 是回锁里出错那一支要删、还在盘上的那一条意图）：一批里同一个条目键出现两次（整理不停），回锁出错那一支单发的删意图写盘报错冻结、会话记下这条活意图；
/// 撤写错、重发成了——删意图那一版落盘，现行那一版里没有活意图，会话照旧记着那一条。前台建一个 inode（「该跑了」）之后下一次取输入：
/// 不停在放弃那一批时停的那一条上、会话不再记着它（照它删的话现行那一版里没有活意图可删，这一趟一个写都不发、停在那一条上，会话照旧记着，
/// 之后每一次取输入都一样，整理再也取不到下一批）。
#[test]
fn a_drop_noted_at_the_relock_and_published_by_the_resend_is_forgotten_before_the_next_batch_instead_of_stopping_on_it(
) {
    let caller = CallerForTests::with_nothing_waiting();
    let BatchFailingOnTheGpuUnderALiveIntentOutOfTheLock {
        pool: _pool,
        mut pending,
        mut devices,
        fault,
        shared,
        intent: _abandoned_intent,
        ticket,
    } = batch_failing_on_the_gpu_under_a_live_intent_out_of_the_lock(
        "relock-drop-frozen-then-resent-is-forgotten-before-the-next-batch",
        BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
            position_of_the_first_occurrence: 1,
            position_of_the_second_occurrence: 3,
        },
        &caller,
    );
    let computed = ticket.compute_out_of_the_lock(&mut |reads| {
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    });
    fault.store(true, std::sync::atomic::Ordering::SeqCst);
    let relocked_with_the_drop_failing =
        pending.publish_the_batch_computed_out_of_the_lock(&mut devices, computed, &caller);
    fault.store(false, std::sync::atomic::Ordering::SeqCst);
    let at_the_relock = run_of(
        relocked_with_the_drop_failing
            .expect("这一批是这个会话在锁外的那一批、盘表是挂载的那一份（出错那一支不排空待发）"),
    );
    assert!(
        matches!(at_the_relock.outcome, CompactionRunOutcome::Failed(_))
            && at_the_relock.versions_published.is_empty(),
        "前提：回锁出错那一支的删意图写盘报错、没发成：{:?}",
        at_the_relock.outcome
    );
    assert_eq!(
        kind_of_the_frozen_publish(&pending),
        Some(KindOfTheFrozenPublish::CompactionPublish),
        "前提：冻着的是回锁这一次调用发的那一次删意图"
    );
    assert!(
        pending
            .session()
            .compaction
            .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "前提：删意图没发成，会话记下这条活意图"
    );
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = None;
    assert!(
        pending
            .resend_the_frozen_publish(&mut devices)
            .expect("撤了写错：重发成了"),
        "前提：有冻结着的发布、重发了"
    );
    assert!(
        live_intent_of(&pending).is_none()
            && pending
                .session()
                .compaction
                .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "前提：重发成了，删意图那一版落盘、现行那一版里没有活意图，会话照旧记着那一条"
    );
    create_one_inode_on_devices_failing_writes(&mut pending, &mut devices, &caller);
    let stopped_on_the_noted_drop = match pending
        .take_the_input_of_a_background_compaction_batch(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            CompactionYieldFloor::NO_FLOOR,
            &caller,
        )
        .expect("没有一批在锁外、盘表是挂载的那一份")
    {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            let run = run_of(after);
            run.versions_published.is_empty()
                && matches!(
                    run.outcome,
                    CompactionRunOutcome::Stopped(
                        CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
                    )
                )
        }
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => false,
    };
    assert!(
        !stopped_on_the_noted_drop,
        "现行那一版里已没有那条意图：不照它删、不停在放弃那一批时停的那一条上"
    );
    assert!(
        !pending
            .session()
            .compaction
            .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "现行那一版里已不是它：取下一批之前忘掉"
    );
}
