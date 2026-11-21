//! 给等会话锁的前台入口让路：有前台入口在等时压住下一批、行程照旧，连着让路到下限次数或下限时长之后下一批不让，显式入口的整理发布清零连着让路的次数，不让路的一次取输入即使这一批什么都不发也打断连着让路，停着时一趟开头的删意图发成也清零，
//! 那一次删意图写盘报错冻结、重发成了同样清零。

use super::publishes_frozen_and_drains_refused_at_the_relock::{
    create_one_inode_on_devices_failing_writes, issue_an_intent_on_devices_failing_writes,
    session_on_the_gpu_on_devices_failing_writes_while_the_fault_is_set,
};
use super::*;
use singlefs_core::transaction::KindOfTheFrozenPublish;

/// 照让路的下限 `yield_floor` 取一次输入。
fn taken_under_the_floor<Device: BlockDevice>(
    pending: &mut MountedSessionWithPendingChanges,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    caller: &CallerForTests,
    yield_floor: CompactionYieldFloor,
) -> BackgroundCompactionInputTaken {
    pending
        .take_the_input_of_a_background_compaction_batch(
            devices,
            pacing_with_the_timer_fallback_off(),
            yield_floor,
            caller,
        )
        .expect("没有一批在锁外、盘表是挂载的那一份")
}

/// 这一次取输入在锁里为前台在等会话锁的入口让了路。
fn yielded_to_a_foreground_entry_waiting_for_the_session_lock(
    taken: BackgroundCompactionInputTaken,
) -> bool {
    match taken {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            let run = run_of(after);
            run.versions_published.is_empty()
                && stop_of(&run)
                    == CompactionRunStop::YieldedToAForegroundEntryWaitingForTheSessionLock
        }
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => false,
    }
}

fn caller_with_a_foreground_entry_waiting_for_the_session_lock() -> CallerForTests {
    CallerForTests::answering(
        AnswerOfTheCallerBeforeTheNextBatch::ForegroundEntryOtherThanAnFsyncIsWaitingForTheSessionLock,
    )
}

/// 前台有 fsync 之外的入口在等会话锁：不设下限时这一批让路、一个写都不发，停因说清让给谁，这一趟还在途，观测里记一次。
#[test]
fn a_foreground_entry_waiting_for_the_session_lock_holds_back_the_next_batch_and_keeps_the_trip_under_way(
) {
    let (_pool, mut pending, mut devices) =
        pool_with_a_batch_issuing_on_the_segment_of_the_first_publish(
            "yield-to-a-foreground-entry-waiting-for-the-session-lock",
        );
    let caller = caller_with_a_foreground_entry_waiting_for_the_session_lock();
    let taken = taken_under_the_floor(
        &mut pending,
        &mut devices,
        &caller,
        CompactionYieldFloor::NO_FLOOR,
    );
    assert!(
        yielded_to_a_foreground_entry_waiting_for_the_session_lock(taken),
        "为前台在等会话锁的入口让路"
    );
    assert_eq!(
        pending
            .session()
            .compaction
            .observations
            .yields_to_a_foreground_entry_waiting_for_the_session_lock,
        1
    );
    assert!(
        pending
            .session()
            .compaction
            .has_a_background_trip_under_way(),
        "让路不是做完：这一趟还在途"
    );
    assert!(!pending.session().compaction.has_a_batch_out_of_the_lock());
}

/// 下限 N 先到：连着让路 2 次之后，前台还在等，下一批不让、交到锁外。
#[test]
fn after_yielding_as_many_times_in_a_row_as_the_floor_allows_the_next_batch_does_not_yield() {
    let (_pool, mut pending, mut devices) =
        pool_with_a_batch_issuing_on_the_segment_of_the_first_publish(
            "yield-floor-of-two-yields-in-a-row",
        );
    let caller = caller_with_a_foreground_entry_waiting_for_the_session_lock();
    let floor = CompactionYieldFloor {
        yields_in_a_row_at_most: NonZeroU64::new(2),
        yielding_time_at_most: None,
    };
    for yield_in_a_row in 1..=2 {
        assert!(
            yielded_to_a_foreground_entry_waiting_for_the_session_lock(taken_under_the_floor(
                &mut pending,
                &mut devices,
                &caller,
                floor
            )),
            "第 {yield_in_a_row} 次照让"
        );
    }
    assert!(
        matches!(
            taken_under_the_floor(&mut pending, &mut devices, &caller, floor),
            BackgroundCompactionInputTaken::BatchOutOfTheLock { .. }
        ),
        "连着让了 2 次：这一批不让、交到锁外"
    );
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        0,
        "下限到了、取了输入而没让路：连着让路断了，计数清零"
    );
}

/// 下限 T 先到：从这一串让路的头一次起累计到 100 毫秒，前台还在等，下一批不让。
#[test]
fn after_yielding_for_as_long_as_the_floor_allows_the_next_batch_does_not_yield() {
    let (_pool, mut pending, mut devices) =
        pool_with_a_batch_issuing_on_the_segment_of_the_first_publish(
            "yield-floor-of-one_hundred-milliseconds",
        );
    let caller = caller_with_a_foreground_entry_waiting_for_the_session_lock();
    let floor = CompactionYieldFloor {
        yields_in_a_row_at_most: None,
        yielding_time_at_most: Some(Duration::from_millis(100)),
    };
    for clock_in_milliseconds in [0, 50] {
        caller.clock_in_milliseconds.set(clock_in_milliseconds);
        assert!(
            yielded_to_a_foreground_entry_waiting_for_the_session_lock(taken_under_the_floor(
                &mut pending,
                &mut devices,
                &caller,
                floor
            )),
            "第 {clock_in_milliseconds} 毫秒照让"
        );
    }
    caller.clock_in_milliseconds.set(100);
    assert!(
        matches!(
            taken_under_the_floor(&mut pending, &mut devices, &caller, floor),
            BackgroundCompactionInputTaken::BatchOutOfTheLock { .. }
        ),
        "从头一次让路起累计 100 毫秒：这一批不让、交到锁外"
    );
}

/// 一批发成两个计数都清零（D26（后台整理与放置回收） 已定项 13 第 6 条，一批是一次整理发布）：下限 2 次，前台在等、让过 1 次；
/// 显式入口（不问调用方，不经取输入那一判，那一判没让路时另清零）发成整理发布之后，连着让路的次数清零。
#[test]
fn a_compaction_publish_of_the_explicit_entry_clears_the_yields_in_a_row() {
    let (_pool, mut pending, mut devices) =
        pool_with_a_batch_issuing_on_the_segment_of_the_first_publish(
            "yield-floor-cleared-by-a-publish-of-the-explicit-entry",
        );
    let caller = caller_with_a_foreground_entry_waiting_for_the_session_lock();
    let floor = CompactionYieldFloor {
        yields_in_a_row_at_most: NonZeroU64::new(2),
        yielding_time_at_most: None,
    };
    assert!(
        yielded_to_a_foreground_entry_waiting_for_the_session_lock(taken_under_the_floor(
            &mut pending,
            &mut devices,
            &caller,
            floor
        )),
        "前提：前台在等，让一次"
    );
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        1,
        "前提：连着让了 1 次"
    );
    let explicit = pending
        .session_for_tests_only()
        .run_compaction(&mut devices)
        .expect("盘表是挂载的那一份、现行那一版有树");
    assert!(
        !explicit.versions_published.is_empty(),
        "前提：显式入口发成了整理发布：{:?}",
        explicit.outcome
    );
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        0,
        "一批发成，连着让路的次数清零"
    );
}

/// 产品默认的让路下限（运行库的两个默认常量：8 次、5 秒）。
const YIELDS_IN_A_ROW_OF_THE_PRODUCT_FLOOR: u64 = 8;

const YIELDING_TIME_OF_THE_PRODUCT_FLOOR: Duration = Duration::from_secs(5);

/// 前台不等、这一批停在回退门上的那一次取输入，钟在第几毫秒。
const CLOCK_OF_THE_TAKE_STOPPING_AT_THE_GATE_IN_MILLISECONDS: u64 = 1000;

/// 前台又在等的那一次取输入，钟在第几毫秒：离头一次让路正好满下限的时长。
const CLOCK_OF_THE_LATER_WAIT_IN_MILLISECONDS: u64 = 5000;

/// 让路下限的「连着」被一次没让路的取输入打断（D26（后台整理与放置回收） 已定项 13 第 6 条：取了输入而没让路的那一批也把两个计数清零）：
/// 产品默认的下限（8 次、5 秒），前台在等、钟 0 毫秒让一次；钟 1000 毫秒前台不等，这一批停在回退门上没发（之前前台推空发布推到门关）——
/// 两个计数都清零；之后门重开（再建满根环）、钟走到 5000 毫秒前台又在等：这一批照样让。只在一批发成时清零的话，这一次判
/// 「从头一次让路起已满 5 秒」、不让。
#[test]
fn a_take_of_the_input_without_yielding_breaks_the_yields_in_a_row_even_when_its_batch_publishes_nothing(
) {
    let mut pool = build_pool("yield-floor-broken-by-a-take-without-yielding");
    let (mut pending, mut devices) = session_with_pending_changes_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::FollowTheWatermarks,
        Some(region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH)),
        1,
    );
    pending
        .session_for_tests_only()
        .allocator
        .compaction_mut()
        .watermark_crossing_forcing =
        CompactionWatermarkCrossingForcing::ForcedForTests(WatermarkQuantity::EmptyClusterSegments);
    let caller = CallerForTests::with_nothing_waiting();
    for _ in 0..USER_CHANGES_FILLING_THE_ROOT_RING {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    let floor = CompactionYieldFloor {
        yields_in_a_row_at_most: NonZeroU64::new(YIELDS_IN_A_ROW_OF_THE_PRODUCT_FLOOR),
        yielding_time_at_most: Some(YIELDING_TIME_OF_THE_PRODUCT_FLOOR),
    };
    caller.answer_before_the_next_batch.set(
        AnswerOfTheCallerBeforeTheNextBatch::ForegroundEntryOtherThanAnFsyncIsWaitingForTheSessionLock,
    );
    assert!(
        yielded_to_a_foreground_entry_waiting_for_the_session_lock(taken_under_the_floor(
            &mut pending,
            &mut devices,
            &caller,
            floor
        )),
        "前提：前台在等，钟 0 毫秒让一次"
    );
    assert_eq!(pending.session().compaction.yields_in_a_row(), 1);
    caller
        .clock_in_milliseconds
        .set(CLOCK_OF_THE_TAKE_STOPPING_AT_THE_GATE_IN_MILLISECONDS);
    caller
        .answer_before_the_next_batch
        .set(AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting);
    let mut sequences_pushed = 0_usize;
    while pending
        .session()
        .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
        .is_none()
        && sequences_pushed < EMPTY_PUBLISH_SEQUENCES_PUSHED_IN_THE_WINDOW_AT_MOST
    {
        push_one_empty_publish_sequence(pending.session_for_tests_only(), &mut devices);
        sequences_pushed += 1;
    }
    assert!(
        pending
            .session()
            .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
            .is_some(),
        "前提：推到了门关（推了 {sequences_pushed} 串）"
    );
    match taken_under_the_floor(&mut pending, &mut devices, &caller, floor) {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            let run = run_of(after);
            assert!(
                matches!(
                    stop_of(&run),
                    CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
                ),
                "前提：前台不等，这一批停在回退门上：{:?}",
                run.outcome
            );
            assert!(run.versions_published.is_empty(), "前提：一个写都没发");
        }
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => {
            panic!("前提：门关着，这一批停在回退门上、不交到锁外")
        }
    }
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        0,
        "这一次取输入没让路：连着让路断了，计数清零"
    );
    for _ in 0..USER_CHANGES_FILLING_THE_ROOT_RING {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    caller
        .clock_in_milliseconds
        .set(CLOCK_OF_THE_LATER_WAIT_IN_MILLISECONDS);
    caller.answer_before_the_next_batch.set(
        AnswerOfTheCallerBeforeTheNextBatch::ForegroundEntryOtherThanAnFsyncIsWaitingForTheSessionLock,
    );
    assert!(
        yielded_to_a_foreground_entry_waiting_for_the_session_lock(taken_under_the_floor(
            &mut pending,
            &mut devices,
            &caller,
            floor
        )),
        "连着让路断过：离头一次让路满 5 秒、前台又在等，这一批照样让"
    );
}

/// 停着时一趟开头单发的删意图也是一次整理发布，发成之后让路的两个计数清零（D26（后台整理与放置回收） 已定项 13 第 6 条「一批发成两个都清零」，
/// 一批是一次整理发布）：选 `gpu`、强制起跑、强制选第一个文件那次发布的段、一批一个；前台在等会话锁、让一次路（连着让了 1 次）；
/// 显式入口推签发那一批时 GPU 实现交回出错——手里没有活意图，一个写都不发，整理停下，计数照旧是 1；之后在单元区头一段签一条意图，
/// 后台入口判出停着而有活意图，起跑那一趟开头单发一次只删意图的发布（不问调用方、不经取输入那一判）：发成之后连着让路的次数是 0。
#[test]
fn a_drop_published_at_the_start_of_a_trip_while_the_compaction_stays_stopped_clears_the_yields_in_a_row(
) {
    let mut pool = build_pool("stopped-drop-at-the-trip-start-clears-the-yields-in-a-row");
    let (mut pending, mut devices, shared) = session_with_pending_changes_on_the_gpu_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH)),
        1,
    );
    let caller_with_an_entry_waiting =
        caller_with_a_foreground_entry_waiting_for_the_session_lock();
    create_one_inode(&mut pending, &mut devices, &caller_with_an_entry_waiting);
    assert!(
        yielded_to_a_foreground_entry_waiting_for_the_session_lock(taken_under_the_floor(
            &mut pending,
            &mut devices,
            &caller_with_an_entry_waiting,
            CompactionYieldFloor::NO_FLOOR
        )),
        "前提：前台在等会话锁，让一次"
    );
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        1,
        "前提：连着让了 1 次"
    );
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = Some((
        GpuComputedSegment::RegionRecordClassification,
        device_lost_on_the_card_with_eight_gibibytes(),
    ));
    let erroring = pending
        .session_for_tests_only()
        .run_compaction(&mut devices)
        .expect("盘表是挂载的那一份、现行那一版有树");
    assert!(
        matches!(
            stop_of(&erroring),
            CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
        ) && erroring.versions_published.is_empty(),
        "前提：签发那一批出错放弃，手里没有活意图，一个写都不发：{:?}",
        erroring.outcome
    );
    assert!(
        pending
            .session()
            .background_compute()
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
        "前提：整理停下"
    );
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        1,
        "前提：出错那一趟没发布，连着让路的次数照旧"
    );
    issue_an_intent_on(
        &mut pending,
        &mut devices,
        region_starting_at(SEGMENT_OF_THE_FIRST_DATA_UNIT),
    );
    match taken_under_the_floor(
        &mut pending,
        &mut devices,
        &CallerForTests::with_nothing_waiting(),
        CompactionYieldFloor::NO_FLOOR,
    ) {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            let run = run_of(after);
            assert_eq!(
                stop_of(&run),
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError,
                "前提：停着而有活意图，起跑那一趟只删意图"
            );
            assert_eq!(
                run.versions_published.len(),
                1,
                "前提：那一趟开头单发了一次只删意图的发布"
            );
        }
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => {
            panic!("前提：停着而不补跑，这一趟不交一批到锁外")
        }
    }
    assert!(
        live_compaction_intent_of_the_entries(&current_of(pending.session()).logged_operations)
            .is_none(),
        "前提：意图删了"
    );
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        0,
        "停着时一趟开头的删意图发成：连着让路的次数清零"
    );
}

/// 只设下限 T 的那一判里 T 是多少毫秒：重发成了之后钟走到这一刻再取输入。
const YIELDING_TIME_AFTER_THE_RESEND_IN_MILLISECONDS: u64 = 100;

/// 停着时一趟开头单发的删意图写盘报错冻结、重发成了（D26（后台整理与放置回收） 已定项 13 第 6 条「冻结的是后台批，照后台批记」、「一批发成两个都清零」）：
/// 同 [`a_drop_published_at_the_start_of_a_trip_while_the_compaction_stays_stopped_clears_the_yields_in_a_row`] 的起法（盘套上可摆写错的那一层），
/// 只是起跑那一趟开头的删意图写盘报错、冻结（种类是整理发布）；撤写错、重发成了之后连着让路的次数是 0。之后切换解除停下、建一个 inode，
/// 前台又在等、只设下限 T、钟走到第 T 毫秒：这一串让路从这一刻起重新数，这一批照样让（头一次让路那一刻的读数没清掉的话，从钟 0 起已满 T、不让）。
#[test]
fn a_drop_frozen_at_the_start_of_a_trip_while_the_compaction_stays_stopped_clears_the_yields_in_a_row_once_resent(
) {
    let mut pool =
        build_pool("stopped-drop-frozen-at-the-trip-start-clears-the-yields-once-resent");
    let (mut pending, mut devices, fault, shared) =
        session_on_the_gpu_on_devices_failing_writes_while_the_fault_is_set(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH)),
        1,
    );
    let caller_with_an_entry_waiting =
        caller_with_a_foreground_entry_waiting_for_the_session_lock();
    create_one_inode_on_devices_failing_writes(
        &mut pending,
        &mut devices,
        &caller_with_an_entry_waiting,
    );
    assert!(
        yielded_to_a_foreground_entry_waiting_for_the_session_lock(taken_under_the_floor(
            &mut pending,
            &mut devices,
            &caller_with_an_entry_waiting,
            CompactionYieldFloor::NO_FLOOR
        )),
        "前提：前台在等会话锁，钟 0 毫秒让一次"
    );
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        1,
        "前提：连着让了 1 次"
    );
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = Some((
        GpuComputedSegment::RegionRecordClassification,
        device_lost_on_the_card_with_eight_gibibytes(),
    ));
    let erroring = pending
        .session_for_tests_only()
        .run_compaction(&mut devices)
        .expect("盘表是挂载的那一份、现行那一版有树");
    assert!(
        matches!(
            stop_of(&erroring),
            CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
        ) && erroring.versions_published.is_empty(),
        "前提：签发那一批出错放弃，手里没有活意图，一个写都不发：{:?}",
        erroring.outcome
    );
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = None;
    issue_an_intent_on_devices_failing_writes(
        &mut pending,
        &mut devices,
        region_starting_at(SEGMENT_OF_THE_FIRST_DATA_UNIT),
    );
    fault.store(true, std::sync::atomic::Ordering::SeqCst);
    let taken_with_the_drop_failing = taken_under_the_floor(
        &mut pending,
        &mut devices,
        &CallerForTests::with_nothing_waiting(),
        CompactionYieldFloor::NO_FLOOR,
    );
    fault.store(false, std::sync::atomic::Ordering::SeqCst);
    match taken_with_the_drop_failing {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => {
            let run = run_of(after);
            assert!(
                matches!(run.outcome, CompactionRunOutcome::Failed(_))
                    && run.versions_published.is_empty(),
                "前提：停着而有活意图，起跑那一趟开头单发的删意图写盘报错、没发成：{:?}",
                run.outcome
            );
        }
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => {
            panic!("前提：停着而不补跑，这一趟不交一批到锁外")
        }
    }
    assert_eq!(
        pending
            .session()
            .allocator
            .frozen_publish()
            .map(|frozen| frozen.kind()),
        Some(KindOfTheFrozenPublish::CompactionPublish),
        "前提：冻着的是那一次删意图（后台批）"
    );
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        1,
        "前提：没发成的那一趟没清连着让路的次数"
    );
    assert!(
        pending
            .resend_the_frozen_publish(&mut devices)
            .expect("撤了写错：重发成了"),
        "前提：有冻结着的发布、重发了"
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(pending.session()).logged_operations)
            .is_none(),
        "前提：重发成了，意图删了"
    );
    assert_eq!(
        pending.session().compaction.yields_in_a_row(),
        0,
        "冻结的后台批重发成了：一批发成，连着让路的次数清零"
    );
    pending
        .switch_background_compute_from_the_next_batch(
            admissible_gpu_settings_naming_both_cards(),
            None,
        )
        .expect("全合法的 gpu 设置：切换收下，停下从下一批起解除");
    create_one_inode_on_devices_failing_writes(
        &mut pending,
        &mut devices,
        &CallerForTests::with_nothing_waiting(),
    );
    caller_with_an_entry_waiting
        .clock_in_milliseconds
        .set(YIELDING_TIME_AFTER_THE_RESEND_IN_MILLISECONDS);
    assert!(
        yielded_to_a_foreground_entry_waiting_for_the_session_lock(taken_under_the_floor(
            &mut pending,
            &mut devices,
            &caller_with_an_entry_waiting,
            CompactionYieldFloor {
                yields_in_a_row_at_most: None,
                yielding_time_at_most: Some(Duration::from_millis(
                    YIELDING_TIME_AFTER_THE_RESEND_IN_MILLISECONDS
                )),
            }
        )),
        "重发成了也清掉这一串让路头一次的读数：钟走到第 {YIELDING_TIME_AFTER_THE_RESEND_IN_MILLISECONDS} 毫秒、前台又在等，这一批照样让"
    );
}
