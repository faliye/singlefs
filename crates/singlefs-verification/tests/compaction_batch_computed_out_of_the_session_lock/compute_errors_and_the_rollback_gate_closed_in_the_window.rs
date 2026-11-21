//! 窗口里的计算出错与回退门关上：窗口里有前台发布时计算出错照锁里出错那样停整理，窗口里回退门关上之后回锁什么都不发、回锁时计算出错留着活意图；
//! 整理没停着（出错之前已请求切换、只放弃这一批的出错）时被门压住的删意图在门重开之后取下一批的输入之前删掉，回锁那一刻破例成立时门关着照删，
//! 取下一批之前此刻破例成立时（起跑之后排空待发把全空段推到底额以下）门关着照删，门仍关着、破例不成立时照旧压着；
//! 没有被压住的删意图时，排空待发把全空段推到底额以下不让别的批绕过关着的门。

use super::*;
use singlefs_core::background_compute::{
    BackgroundGpuComputeFailed, BackgroundTaskAfterTheComputeError,
};
use singlefs_core::compaction_plan::{
    foreground_writes_are_below_the_empty_cluster_segments_floor, EmptyClusterSegmentsFloor,
};
use singlefs_format::CLUSTER_SEGMENT_SLOTS;

/// 锁外那一批算着出错（GPU 实现交回出错），同时锁外期间前台发过布：回锁里出错先于一切——照锁里出错那一支停下整理、单发一次只删活意图的发布、
/// 这一次叫起带报警，与锁里一口气推到出错的结局相同（不因为现行那一版变了就先去比记录、改走别的停）；池级 checker 在 `judge` 上判不红。
fn the_compute_error_with_a_foreground_publish_in_the_window_stops_the_compaction_as_an_error_in_the_lock_does_judged_by(
    judge: JudgeOfAPoolCheckerJudgement,
) {
    let mut pool = build_pool("out-of-the-lock-compute-error-first");
    let (mut pending, mut devices, shared) = session_with_pending_changes_on_the_gpu_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH)),
        1,
    );
    let caller = CallerForTests::with_nothing_waiting();
    create_one_inode(&mut pending, &mut devices, &caller);
    let issuing = ticket_taken(&mut pending, &mut devices, &caller);
    let issuing = computed_reading_in_the_session(*issuing, &pending, &devices);
    let issued = run_of(relocked(&mut pending, &mut devices, issuing, &caller));
    assert_eq!(issued.versions_published.len(), 1, "前提：头一批签发");
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = Some((
        GpuComputedSegment::RegionRecordClassification,
        device_lost_on_the_card_with_eight_gibibytes(),
    ));
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    create_one_inode(&mut pending, &mut devices, &caller);
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let after = relocked(&mut pending, &mut devices, computed, &caller);
    assert!(
        after
            .call
            .alarm_of_the_compaction_stopped_after_a_compute_error()
            .is_some(),
        "这一次叫起带着整理停下的报警"
    );
    let run = run_of(after);
    assert!(
        matches!(
            stop_of(&run),
            CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
        ),
        "照锁里出错那一支停：{:?}",
        run.outcome
    );
    assert!(
        pending
            .session()
            .background_compute()
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
        "整理这个后台任务停下"
    );
    assert_eq!(run.versions_published.len(), 1, "单发了一次只删意图的发布");
    assert!(
        live_compaction_intent_of_the_entries(&current_of(pending.session()).logged_operations)
            .is_none()
            && pending
                .session()
                .allocator
                .compaction()
                .live_intent()
                .is_none(),
        "活意图删了、R 排他撤了"
    );
    assert!(
        run.background_compute_of_each_batch
            .last()
            .is_some_and(|batch| batch.compute_error_that_abandoned_the_batch.is_some()),
        "这一批的记账报出放弃它的那个错"
    );
    if let Some(violated) =
        invariants_violated_on(&pool, "out-of-the-lock-compute-error-first", judge)
    {
        assert_eq!(
            violated,
            Vec::<&str>::new(),
            "池级 checker 不红（{judge:?}）"
        );
    }
}

/// 照调用方的多机配置判池级 checker（配了 GPU 而这个进程拿不到卡记「本次未跑」）。
#[test]
fn a_compute_error_with_a_foreground_publish_in_the_window_stops_the_compaction_as_an_error_in_the_lock_does(
) {
    the_compute_error_with_a_foreground_publish_in_the_window_stops_the_compaction_as_an_error_in_the_lock_does_judged_by(
        JudgeOfAPoolCheckerJudgement::JudgeOfTheMultiHostConfigurationOfTheCaller,
    );
}

/// 在只开 CPU 的多机配置下由 CPU 判器判池级 checker（不打「本次未跑」）。
#[test]
fn a_compute_error_with_a_foreground_publish_in_the_window_stops_the_compaction_as_an_error_in_the_lock_does_on_the_cpu_judge(
) {
    the_compute_error_with_a_foreground_publish_in_the_window_stops_the_compaction_as_an_error_in_the_lock_does_judged_by(
        JudgeOfAPoolCheckerJudgement::CpuJudgeUnderACpuOnlyMultiHostConfiguration,
    );
}

/// 回锁那一次发布过回退门（D26（后台整理与放置回收） 已定项 13 第 5 条）：开关照水位判（越线靠只供测试的开关），取输入那一刻门开；
/// 锁外那一段先算完（读盘时盘上还是票里那一版），之后、回锁之前前台推空发布推到门关（[`push_one_empty_publish_sequence`]：先抬 F 到上限，
/// 到了上限推不抬 F 的；根环里同一个用户状态的空发布挤掉旧根；抬 F 回收的槽会被后面的空发布再发出去，算在推之前，选中那一段读到的才是票里那一版的单元），
/// 回锁在现行那一版上再判一次门，门关就不发。
#[test]
fn a_relock_after_the_rollback_gate_closed_in_the_window_publishes_nothing() {
    let mut pool = build_pool("relock-judges-the-rollback-gate-again");
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
    assert!(
        pending
            .session()
            .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
            .is_none(),
        "前提：取输入那一刻门开"
    );
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
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
        "前提：锁外期间推到了门关（推了 {sequences_pushed} 串）"
    );
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert!(
        matches!(
            stop_of(&run),
            CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
        ),
        "回锁照门关停：{:?}",
        run.outcome
    );
    assert!(run.versions_published.is_empty(), "门关就一个写都不发");
}

/// 回锁里出错那一支单发的删意图发布同样过回退门（D26（后台整理与放置回收） 已定项 13 第 5 条：停着时那一次只删意图的发布同样过这道门，
/// 门关就不发，意图与 R 排他留到门开）：选 `gpu`、开关照水位判（越线靠只供测试的开关）、在单元区头一段签着一条意图、根环装满不同用户状态，
/// 取输入那一刻门开；锁外那一批 GPU 实现交回出错，锁外期间前台推空发布推到门关；回锁出错先于一切——整理停下、这一次叫起带报警，
/// 而门关着就不删意图：一个写都不发，盘上与分配器这一侧的活意图都留着。
#[test]
fn a_compute_error_at_the_relock_after_the_rollback_gate_closed_in_the_window_keeps_the_live_intent(
) {
    let mut pool = build_pool("relock-compute-error-keeps-the-intent-behind-the-rollback-gate");
    let (mut pending, mut devices, shared) = session_with_pending_changes_on_the_gpu_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::FollowTheWatermarks,
        None,
        UNITS_OF_A_BATCH_COVERING_A_SEGMENT,
    );
    pending
        .session_for_tests_only()
        .allocator
        .compaction_mut()
        .watermark_crossing_forcing =
        CompactionWatermarkCrossingForcing::ForcedForTests(WatermarkQuantity::EmptyClusterSegments);
    issue_an_intent_on(
        &mut pending,
        &mut devices,
        region_starting_at(SEGMENT_OF_THE_FIRST_DATA_UNIT),
    );
    let caller = CallerForTests::with_nothing_waiting();
    for _ in 0..USER_CHANGES_FILLING_THE_ROOT_RING {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    assert!(
        pending
            .session()
            .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
            .is_none(),
        "前提：取输入那一刻门开"
    );
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = Some((
        GpuComputedSegment::RegionRecordClassification,
        device_lost_on_the_card_with_eight_gibibytes(),
    ));
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
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
        "前提：锁外期间推到了门关（推了 {sequences_pushed} 串）"
    );
    let txg_before_the_relock = current_of(pending.session()).root.checkpoint_txg;
    let after = relocked(&mut pending, &mut devices, computed, &caller);
    assert!(
        after
            .call
            .alarm_of_the_compaction_stopped_after_a_compute_error()
            .is_some(),
        "出错先于一切：这一次叫起带着整理停下的报警"
    );
    let run = run_of(after);
    assert!(
        matches!(
            stop_of(&run),
            CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
        ),
        "照锁里出错那一支停：{:?}",
        run.outcome
    );
    assert!(
        pending
            .session()
            .background_compute()
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
        "整理这个后台任务停下"
    );
    assert!(
        run.versions_published.is_empty(),
        "门关着：删意图那一次不发"
    );
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before_the_relock,
        "一个写都没发"
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(pending.session()).logged_operations)
            .is_some()
            && pending
                .session()
                .allocator
                .compaction()
                .live_intent()
                .is_some(),
        "活意图与 R 排他留到门开"
    );
}

/// 回锁那一刻门关、出错那一支的删意图被压住之后，前台再建满根环（门重开）时至多建几次 inode。
const USER_CHANGES_REOPENING_THE_ROLLBACK_GATE_AT_MOST: usize =
    2 * USER_CHANGES_FILLING_THE_ROOT_RING;

/// 回锁出错那一支门关着的那几格共用的起法（D26（后台整理与放置回收） 已定项 13 第 4、5 条）：同 [`relocked_with_the_rollback_gate_closed_in_the_window`]，
/// GPU 实现在区域扫描里的单元头解析与映射反查那一段交回 `failure`。
fn relocked_after_a_compute_error_with_the_rollback_gate_closed_in_the_window(
    tag: &str,
    failure: BackgroundGpuComputeFailed,
    before_the_computation: impl FnOnce(&mut MountedSessionWithPendingChanges),
    before_the_relock: impl FnOnce(&mut MountedSessionWithPendingChanges),
) -> (
    BuiltPool,
    MountedSessionWithPendingChanges,
    Devices,
    Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
    CompactionRun,
) {
    relocked_with_the_rollback_gate_closed_in_the_window(
        tag,
        Some(failure),
        before_the_computation,
        before_the_relock,
    )
}

/// 回锁那一刻门关的那几格共用的起法（D26（后台整理与放置回收） 已定项 13 第 4、5 条）：选 `gpu`、开关照水位判（越线靠只供测试的开关）、
/// 一批罩得住一整段，在单元区头一段签着一条意图、根环装满不同用户状态，取输入那一刻门开；GPU 实现在区域扫描里的单元头解析与映射反查那一段
/// 交回 `failure`（`None` 是不出错）。取输入之后 `before_the_computation` 在锁外期间做（算之前），锁外那一批算完、前台推空发布推到门关，
/// `before_the_relock` 再做，回锁。交回池、会话、盘、替身与回锁那一趟。
fn relocked_with_the_rollback_gate_closed_in_the_window(
    tag: &str,
    failure: Option<BackgroundGpuComputeFailed>,
    before_the_computation: impl FnOnce(&mut MountedSessionWithPendingChanges),
    before_the_relock: impl FnOnce(&mut MountedSessionWithPendingChanges),
) -> (
    BuiltPool,
    MountedSessionWithPendingChanges,
    Devices,
    Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
    CompactionRun,
) {
    let mut pool = build_pool(tag);
    let (mut pending, mut devices, shared) = session_with_pending_changes_on_the_gpu_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::FollowTheWatermarks,
        None,
        UNITS_OF_A_BATCH_COVERING_A_SEGMENT,
    );
    pending
        .session_for_tests_only()
        .allocator
        .compaction_mut()
        .watermark_crossing_forcing =
        CompactionWatermarkCrossingForcing::ForcedForTests(WatermarkQuantity::EmptyClusterSegments);
    issue_an_intent_on(
        &mut pending,
        &mut devices,
        region_starting_at(SEGMENT_OF_THE_FIRST_DATA_UNIT),
    );
    let caller = CallerForTests::with_nothing_waiting();
    for _ in 0..USER_CHANGES_FILLING_THE_ROOT_RING {
        create_one_inode(&mut pending, &mut devices, &caller);
    }
    assert!(
        pending
            .session()
            .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
            .is_none(),
        "前提：取输入那一刻门开"
    );
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment =
        failure.map(|failed| (GpuComputedSegment::RegionRecordClassification, failed));
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    before_the_computation(&mut pending);
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
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
        "前提：锁外期间推到了门关（推了 {sequences_pushed} 串）"
    );
    before_the_relock(&mut pending);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    (pool, pending, devices, shared, run)
}

/// 现行那一版里与分配器这一侧都有活意图。
fn has_a_live_intent(pending: &MountedSessionWithPendingChanges) -> bool {
    live_compaction_intent_of_the_entries(&current_of(pending.session()).logged_operations)
        .is_some()
        && pending
            .session()
            .allocator
            .compaction()
            .live_intent()
            .is_some()
}

/// 回锁出错那一支的删意图被门压住、整理没停着（出错之前已请求切换、只放弃这一批的出错）之后（D26（后台整理与放置回收） 已定项 13 第 4 条
/// 「正在算的那一批放弃，意图也删掉」、第 5 条「门关就不发，意图与 `R` 排他留到门开」）：回锁那一趟一个写都不发、意图留着、会话记下它；
/// 替身不再出错、前台建 inode 建到门重开（每次也记下「该跑了」），下一次取输入不交一批到锁外：先单发一次只删意图的发布（不搬单元），
/// 这一趟停在放弃那一批时停的那一条，意图与 R 排他都撤了、会话不再记着它，之后没有这条意图可接着搬。
fn the_held_back_drop_is_published_once_the_gate_reopens_before_any_relocation_under_the_intent(
    tag: &str,
    failure: BackgroundGpuComputeFailed,
    before_the_computation: impl FnOnce(&mut MountedSessionWithPendingChanges),
    task_after_the_error: BackgroundTaskAfterTheComputeError,
) {
    let (_pool, mut pending, mut devices, shared, at_the_relock) =
        relocked_after_a_compute_error_with_the_rollback_gate_closed_in_the_window(
            tag,
            failure,
            before_the_computation,
            |_pending| {},
        );
    let stop_of_the_abandoned_batch =
        CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
            task_after_the_error,
        };
    assert_eq!(
        stop_of(&at_the_relock),
        stop_of_the_abandoned_batch,
        "前提：回锁出错先于一切，这一批放弃"
    );
    assert!(
        !pending
            .session()
            .background_compute()
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
        "前提：整理没停着（{task_after_the_error:?}）"
    );
    assert!(
        at_the_relock.versions_published.is_empty() && has_a_live_intent(&pending),
        "门关着：回锁那一次删意图不发，意图与 R 排他留着"
    );
    assert!(
        pending
            .session()
            .compaction
            .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "会话记下这条被门压住的删意图"
    );
    shared
        .lock()
        .expect("替身没在别处 panic 过")
        .failing_segment = None;
    let caller = CallerForTests::with_nothing_waiting();
    let mut user_changes = 0_usize;
    while pending
        .session()
        .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
        .is_some()
        && user_changes < USER_CHANGES_REOPENING_THE_ROLLBACK_GATE_AT_MOST
    {
        create_one_inode(&mut pending, &mut devices, &caller);
        user_changes += 1;
    }
    assert!(
        pending
            .session()
            .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
            .is_none(),
        "前提：前台建 inode 建到门重开（建了 {user_changes} 次）"
    );
    let after_the_gate_reopened = match pending
        .take_the_input_of_a_background_compaction_batch(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            CompactionYieldFloor::NO_FLOOR,
            &caller,
        )
        .expect("没有一批在锁外、盘表是挂载的那一份")
    {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => run_of(after),
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => panic!(
            "门开之后先删被压住的那一条意图：不在它下面再交一批到锁外接着搬（{task_after_the_error:?}）"
        ),
    };
    assert_eq!(
        after_the_gate_reopened.versions_published.len(),
        1,
        "门开之后单发一次只删意图的发布：{:?}",
        after_the_gate_reopened.outcome
    );
    assert!(
        after_the_gate_reopened.versions_published[0]
            .units_relocated_by_this_publish
            .is_empty(),
        "删意图那一次一个单元都不搬"
    );
    assert_eq!(
        stop_of(&after_the_gate_reopened),
        stop_of_the_abandoned_batch,
        "删它那一趟停在放弃那一批时停的那一条"
    );
    assert!(
        !has_a_live_intent(&pending)
            && live_compaction_intent_of_the_entries(
                &after_the_gate_reopened.versions_published[0].logged_operations
            )
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

/// 出错之前已请求切换（同一组 gpu 设置，沿用这次挂载的那一份实现）：停下那一刻当场解除，整理没停着。
#[test]
fn after_a_compute_error_lifted_by_a_switch_requested_before_it_the_drop_held_back_by_the_gate_is_published_once_the_gate_reopens(
) {
    the_held_back_drop_is_published_once_the_gate_reopens_before_any_relocation_under_the_intent(
        "held-back-drop-after-a-switch-requested-before-the-error",
        device_lost_on_the_card_with_eight_gibibytes(),
        |pending| {
            pending
                .switch_background_compute_from_the_next_batch(
                    admissible_gpu_settings_naming_both_cards(),
                    None,
                )
                .expect("全合法的 gpu 设置：切换收下，记成待生效");
        },
        BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError,
    );
}

/// 一批里同一个条目键出现两次：只放弃这一批，整理不停。
#[test]
fn after_the_same_entry_key_twice_in_one_batch_the_drop_held_back_by_the_gate_is_published_once_the_gate_reopens(
) {
    the_held_back_drop_is_published_once_the_gate_reopens_before_any_relocation_under_the_intent(
        "held-back-drop-after-the-same-entry-key-twice",
        BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
            position_of_the_first_occurrence: 1,
            position_of_the_second_occurrence: 3,
        },
        |_pending| {},
        BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned,
    );
}

/// 只供测试的底额开关抬到 `u64::MAX`：每块盘的全空聚簇段数都低于它，开关照水位判时前台写不进（破例成立），别的开关照旧。
fn raise_the_empty_cluster_segments_floor_above_every_device(
    pending: &mut MountedSessionWithPendingChanges,
) {
    force_the_empty_cluster_segments_floor(pending, u64::MAX);
}

/// 只供测试的底额开关定成 `floor_in_segments`（`EmptyClusterSegmentsFloor::ForcedForTests`），别的开关照旧。
fn force_the_empty_cluster_segments_floor(
    pending: &mut MountedSessionWithPendingChanges,
    floor_in_segments: u64,
) {
    let CompactionSwitches {
        batch_size,
        forced_region,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: _,
        relocation_publish_order,
    } = pending.session().allocator.compaction().switches;
    pending
        .session_for_tests_only()
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size,
            forced_region,
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor: EmptyClusterSegmentsFloor::ForcedForTests(
                floor_in_segments,
            ),
            relocation_publish_order,
        });
}

/// 回锁出错那一支在回锁那一刻判破例（D26（后台整理与放置回收） 已定项 13 第 5 条「破例的条件（全空聚簇段数低于底额、前台已写不进）成立时，
/// 这一次也照发」）：取输入那一刻破例不成立、门开；锁外期间推空发布推到门关，再把全空段降到底额以下（只供测试的底额开关抬过每块盘的全空段数）；
/// 回锁出错先于一切——整理停下，删意图那一次不看取输入时定的回锁判不判门，照发：发了一次只删意图的发布、意图与 R 排他撤了、会话没记下被压住的删意图。
#[test]
fn a_compute_error_at_the_relock_after_the_floor_was_crossed_in_the_window_drops_the_intent_behind_the_closed_gate(
) {
    let (_pool, pending, _devices, _shared, run) =
        relocked_after_a_compute_error_with_the_rollback_gate_closed_in_the_window(
            "relock-compute-error-drops-the-intent-once-the-floor-is-crossed",
            device_lost_on_the_card_with_eight_gibibytes(),
            |pending| {
                assert!(
                    !foreground_writes_are_below_the_empty_cluster_segments_floor(
                        &pending.session().allocator
                    ),
                    "前提：取输入那一刻全空段不低于底额，破例不成立"
                );
            },
            |pending| {
                raise_the_empty_cluster_segments_floor_above_every_device(pending);
                assert!(
                    foreground_writes_are_below_the_empty_cluster_segments_floor(
                        &pending.session().allocator
                    ),
                    "前提：锁外期间全空段降到底额以下"
                );
            },
        );
    assert!(
        matches!(
            stop_of(&run),
            CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
            }
        ),
        "回锁出错先于一切，整理停下：{:?}",
        run.outcome
    );
    assert_eq!(
        run.versions_published.len(),
        1,
        "破例成立：门关着照发那一次只删意图的发布"
    );
    assert!(!has_a_live_intent(&pending), "意图删了、R 排他撤了");
    assert!(
        !pending
            .session()
            .compaction
            .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "照发了：会话不记被压住的删意图"
    );
}

/// 第一块盘单元区里不是整段空闲的那几个整段此刻的空闲槽数（两盘同槽，第一块盘代表）：新写的数据单元把这些槽都落满之前，罩不住一个整段空闲的段。
fn free_slots_in_the_segments_not_wholly_free(pending: &MountedSessionWithPendingChanges) -> u64 {
    let device_map = pending
        .session()
        .allocator
        .devices
        .first()
        .expect("池里有盘");
    let unit_area_start = device_map.unit_area_start().slot().0;
    (0..device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS)
        .map(|segment| {
            let segment_start = unit_area_start + segment * CLUSTER_SEGMENT_SLOTS;
            u64::try_from(
                (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
                    .filter(|slot| device_map.is_free(SlotNumber(*slot)))
                    .count(),
            )
            .expect("一段 64 槽")
        })
        .filter(|free_slots| *free_slots < CLUSTER_SEGMENT_SLOTS)
        .sum()
}

/// 第 1 条补发那一次的破例（D26（后台整理与放置回收） 已定项 13 第 5 条「破例的条件（全空聚簇段数低于底额、前台已写不进）成立时，这一次也照发」）
/// 在取下一批之前此刻判，不只看这一趟起跑时定的预算档：一批里同一个条目键出现两次（整理不停），回锁那一刻门关、删意图被压住、会话记下它；
/// 前台建一个 inode（「该跑了」，门仍关着），只供测试的底额开关定成此刻每块盘全空段数里最小的那一个（起跑那一判前台写得进，预算档不是破例那一档），
/// 收下一次落不进现有非空段的整份写进待发；下一次取输入：判过起跑之后排空待发、那一次写落进全空段，全空段降到底额以下、前台写不进，
/// 取下一批之前破例成立，门关着照发那一次只删意图的发布，这一趟停在放弃那一批时停的那一条，意图与 R 排他撤了、会话不再记着它。
#[test]
fn the_drop_held_back_by_the_gate_is_published_behind_the_closed_gate_when_the_drain_before_the_next_batch_crosses_the_floor(
) {
    let (_pool, mut pending, mut devices, _shared, at_the_relock) =
        relocked_after_a_compute_error_with_the_rollback_gate_closed_in_the_window(
            "held-back-drop-behind-the-closed-gate-after-the-drain-crosses-the-floor",
            BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
                position_of_the_first_occurrence: 1,
                position_of_the_second_occurrence: 3,
            },
            |_pending| {},
            |_pending| {},
        );
    let stop_of_the_abandoned_batch =
        CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
            task_after_the_error:
                BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned,
        };
    assert_eq!(
        stop_of(&at_the_relock),
        stop_of_the_abandoned_batch,
        "前提：回锁出错先于一切，这一批放弃、整理不停"
    );
    assert!(
        at_the_relock.versions_published.is_empty()
            && has_a_live_intent(&pending)
            && pending
                .session()
                .compaction
                .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "前提：门关着，回锁那一次删意图不发，意图留着、会话记下它"
    );
    let caller = CallerForTests::with_nothing_waiting();
    create_one_inode(&mut pending, &mut devices, &caller);
    assert!(
        pending
            .session()
            .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
            .is_some(),
        "前提：建一个 inode 之后门仍关着"
    );
    let empty_segments_of_the_emptiest_device = pending
        .session()
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.empty_segments())
        .min()
        .expect("池里有盘");
    force_the_empty_cluster_segments_floor(&mut pending, empty_segments_of_the_emptiest_device);
    assert!(
        !foreground_writes_are_below_the_empty_cluster_segments_floor(&pending.session().allocator),
        "前提：起跑那一判时全空段不低于底额，前台写得进"
    );
    let data_units_of_the_write =
        free_slots_in_the_segments_not_wholly_free(&pending) / 2 + CLUSTER_SEGMENT_SLOTS + 1;
    let content_of_the_write = vec![
        0x6Bu8;
        usize::try_from(data_units_of_the_write).expect("数据单元数")
            * data_unit_payload_capacity()
    ];
    pending
        .accept(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER + 1),
                content: &content_of_the_write,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            caller.clock_reading(),
        )
        .expect("全空段不低于底额：这一次整份写收下");
    assert_eq!(
        pending.pending_changes_accepted(),
        1,
        "前提：这一次整份写留在待发里，没当场发"
    );
    let after = match pending
        .take_the_input_of_a_background_compaction_batch(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            CompactionYieldFloor::NO_FLOOR,
            &caller,
        )
        .expect("没有一批在锁外、盘表是挂载的那一份")
    {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => after,
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => {
            panic!("会话记着被门压住的那一条：先删它，不在它下面再交一批到锁外")
        }
    };
    assert!(
        after.pending_changes_published_before_the_run.is_some(),
        "前提：判过起跑之后排空了待发"
    );
    assert!(
        foreground_writes_are_below_the_empty_cluster_segments_floor(&pending.session().allocator),
        "前提：排空待发那一次写落进全空段，全空段降到底额以下、前台写不进"
    );
    let run = run_of(after);
    assert_eq!(
        run.versions_published.len(),
        1,
        "取下一批之前此刻破例成立：门关着照发那一次只删意图的发布，{:?}",
        run.outcome
    );
    assert!(
        run.versions_published[0]
            .units_relocated_by_this_publish
            .is_empty(),
        "删意图那一次一个单元都不搬"
    );
    assert_eq!(
        stop_of(&run),
        stop_of_the_abandoned_batch,
        "删它那一趟停在放弃那一批时停的那一条"
    );
    assert!(
        !has_a_live_intent(&pending)
            && !pending
                .session()
                .compaction
                .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "意图删了、R 排他撤了，会话不再记着它"
    );
}

/// 会话记着被门压住的删意图、门仍关着、全空段不低于底额（破例不成立）时，取下一批之前那一次删意图照样过回退门（D26（后台整理与放置回收） 已定项 13
/// 第 5 条「门开或破例成立就只发这一次删意图，门关就这一趟不发、也不在这条意图下接着搬」）：一批里同一个条目键出现两次（整理不停），回锁那一刻门关、
/// 删意图被压住、会话记下它；前台建一个 inode（「该跑了」，门仍关着、全空段不低于底额），下一次取输入停在回退门上、一个写都不发，
/// 意图与 R 排他留着、会话仍记着它。
#[test]
fn the_drop_held_back_by_the_gate_stays_held_back_while_the_gate_is_closed_and_the_floor_is_not_crossed(
) {
    let (_pool, mut pending, mut devices, _shared, at_the_relock) =
        relocked_after_a_compute_error_with_the_rollback_gate_closed_in_the_window(
            "held-back-drop-stays-behind-the-closed-gate-above-the-floor",
            BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
                position_of_the_first_occurrence: 1,
                position_of_the_second_occurrence: 3,
            },
            |_pending| {},
            |_pending| {},
        );
    assert_eq!(
        stop_of(&at_the_relock),
        CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
            task_after_the_error:
                BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned,
        },
        "前提：回锁出错先于一切，这一批放弃、整理不停"
    );
    assert!(
        at_the_relock.versions_published.is_empty()
            && has_a_live_intent(&pending)
            && pending
                .session()
                .compaction
                .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "前提：门关着，回锁那一次删意图不发，意图留着、会话记下它"
    );
    let caller = CallerForTests::with_nothing_waiting();
    create_one_inode(&mut pending, &mut devices, &caller);
    assert!(
        pending
            .session()
            .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
            .is_some(),
        "前提：建一个 inode 之后门仍关着"
    );
    assert!(
        !foreground_writes_are_below_the_empty_cluster_segments_floor(&pending.session().allocator),
        "前提：全空段不低于底额，破例不成立"
    );
    let run = match pending
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
            panic!("会话记着被门压住的那一条：不在它下面再交一批到锁外")
        }
    };
    assert!(
        matches!(
            stop_of(&run),
            CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
        ),
        "门关着、破例不成立：删意图那一次停在回退门上，{:?}",
        run.outcome
    );
    assert!(run.versions_published.is_empty(), "门关就这一趟不发");
    assert!(
        has_a_live_intent(&pending)
            && pending
                .session()
                .compaction
                .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "意图与 R 排他留到门开或破例成立，会话仍记着它"
    );
}

/// 取下一批之前那一判的破例只在这一趟头一次发布的预算档、或补发被门压住的那一次删意图时成立（D26（后台整理与放置回收） 已定项 13 第 5 条），
/// 别的批照判回退门：同 [`the_drop_held_back_by_the_gate_is_published_behind_the_closed_gate_when_the_drain_before_the_next_batch_crosses_the_floor`]
/// 的起法，只是锁外那一批不出错——回锁那一刻门关、这一批不发，会话不记任何待删的意图；之后同样建一个 inode、底额定成此刻最少的全空段数、
/// 收下一次落不进现有非空段的整份写；下一次取输入判过起跑之后排空待发、全空段降到底额以下，这一趟的预算档不是破例那一档、又没有要补发的删意图：
/// 照判回退门，停在门上、一个写都不发，不交一批到锁外。
#[test]
fn without_a_held_back_drop_a_floor_crossed_by_the_drain_before_the_next_batch_leaves_the_closed_gate_shut(
) {
    let (_pool, mut pending, mut devices, _shared, at_the_relock) =
        relocked_with_the_rollback_gate_closed_in_the_window(
            "closed-gate-stays-shut-after-the-drain-crosses-the-floor-without-a-held-back-drop",
            None,
            |_pending| {},
            |_pending| {},
        );
    assert!(
        matches!(
            stop_of(&at_the_relock),
            CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
        ) && at_the_relock.versions_published.is_empty(),
        "前提：锁外那一批不出错，回锁那一刻门关、这一批不发：{:?}",
        at_the_relock.outcome
    );
    assert!(
        !pending
            .session()
            .compaction
            .has_an_intent_of_an_abandoned_batch_still_to_drop(),
        "前提：会话不记任何待删的意图"
    );
    let caller = CallerForTests::with_nothing_waiting();
    create_one_inode(&mut pending, &mut devices, &caller);
    assert!(
        pending
            .session()
            .stop_of_the_rollback_gate_now_for_tests(devices.as_slice())
            .is_some(),
        "前提：建一个 inode 之后门仍关着"
    );
    let empty_segments_of_the_emptiest_device = pending
        .session()
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.empty_segments())
        .min()
        .expect("池里有盘");
    force_the_empty_cluster_segments_floor(&mut pending, empty_segments_of_the_emptiest_device);
    assert!(
        !foreground_writes_are_below_the_empty_cluster_segments_floor(&pending.session().allocator),
        "前提：起跑那一判时全空段不低于底额，前台写得进"
    );
    let data_units_of_the_write =
        free_slots_in_the_segments_not_wholly_free(&pending) / 2 + CLUSTER_SEGMENT_SLOTS + 1;
    let content_of_the_write = vec![
        0x6Bu8;
        usize::try_from(data_units_of_the_write).expect("数据单元数")
            * data_unit_payload_capacity()
    ];
    pending
        .accept(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER + 1),
                content: &content_of_the_write,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            caller.clock_reading(),
        )
        .expect("全空段不低于底额：这一次整份写收下");
    assert_eq!(
        pending.pending_changes_accepted(),
        1,
        "前提：这一次整份写留在待发里，没当场发"
    );
    let after = match pending
        .take_the_input_of_a_background_compaction_batch(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            CompactionYieldFloor::NO_FLOOR,
            &caller,
        )
        .expect("没有一批在锁外、盘表是挂载的那一份")
    {
        BackgroundCompactionInputTaken::FinishedInTheLock(after) => after,
        BackgroundCompactionInputTaken::BatchOutOfTheLock { .. } => {
            panic!("没有要补发的删意图、预算档不是破例那一档：门关着，这一批不交到锁外")
        }
    };
    assert!(
        after.pending_changes_published_before_the_run.is_some(),
        "前提：判过起跑之后排空了待发"
    );
    assert!(
        foreground_writes_are_below_the_empty_cluster_segments_floor(&pending.session().allocator),
        "前提：排空待发那一次写落进全空段，全空段降到底额以下、前台写不进"
    );
    let run = run_of(after);
    assert!(
        matches!(
            stop_of(&run),
            CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
        ),
        "照判回退门：停在门上，{:?}",
        run.outcome
    );
    assert!(run.versions_published.is_empty(), "门关着：一个写都不发");
}
