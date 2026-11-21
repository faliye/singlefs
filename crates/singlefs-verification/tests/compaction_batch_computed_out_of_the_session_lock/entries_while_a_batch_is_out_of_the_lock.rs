//! 一批在锁外期间的各个入口：三段串起来、中间没有窗口的写与入口写的块相同（至多一次发布），第二次取输入在期间请求的开关生效之前就拒，显式整理入口什么都不发，锁里的后台入口不起，GPU 实现在锁里读与算完交回时都不借出。

use super::*;

/// 三段首尾相接、中间什么都不发生：与锁里推完的那一个入口（至多一次发布）在同一份输入上写出的块序列逐字节相同、每次调用的结局相同。
/// 两个池走同一串操作（同一份起池、同一次挂载、同一个用户改动），一个每次调用经 `run_compaction_in_the_background_for_at_most_one_publish`，
/// 一个经取输入、锁外算（回会话读盘）、回锁发布；定时兜底每秒一次、钟每次调用走一秒，推到预算用完之后不起跑。
/// 改动前那个入口的写序列由没改的崩溃枚举用例钉着（整理搬迁那几条流的步数与状态数）：这里钉的是放锁的三段与它逐字节相同。
#[test]
fn three_stages_chained_without_a_window_write_the_same_blocks_as_the_entry_for_at_most_one_publish(
) {
    let (pool_in_the_lock, mut pending_in_the_lock, mut devices_in_the_lock) =
        pool_due_after_a_user_change("three-stages-chained-against-the-entry-in-the-lock");
    let (pool_in_three_stages, mut pending_in_three_stages, mut devices_in_three_stages) =
        pool_due_after_a_user_change("three-stages-chained-in-three-stages");
    let caller = CallerForTests::with_nothing_waiting();
    let mut outcomes_in_the_lock = Vec::new();
    let mut outcomes_in_three_stages = Vec::new();
    let mut batches_out_of_the_lock = 0_usize;
    for call in 0..CALLS_COMPARED_AT_MOST {
        caller
            .clock_in_milliseconds
            .set(1000 * u64::try_from(call).expect("调用次数"));
        let in_the_lock = pending_in_the_lock
            .run_compaction_in_the_background_for_at_most_one_publish(
                &mut devices_in_the_lock,
                pacing_with_a_timer_fallback_every_second(),
                &caller,
            )
            .expect("盘表是挂载的那一份、现行那一版有树");
        outcomes_in_the_lock.push(outcome_of_the_call(&in_the_lock.call));
        let in_three_stages = match pending_in_three_stages
            .take_the_input_of_a_background_compaction_batch(
                &mut devices_in_three_stages,
                pacing_with_a_timer_fallback_every_second(),
                CompactionYieldFloor::NO_FLOOR,
                &caller,
            )
            .expect("没有一批在锁外、盘表是挂载的那一份")
        {
            BackgroundCompactionInputTaken::FinishedInTheLock(after) => after,
            BackgroundCompactionInputTaken::BatchOutOfTheLock { ticket, .. } => {
                batches_out_of_the_lock += 1;
                let computed = computed_reading_in_the_session(
                    *ticket,
                    &pending_in_three_stages,
                    &devices_in_three_stages,
                );
                relocked(
                    &mut pending_in_three_stages,
                    &mut devices_in_three_stages,
                    computed,
                    &caller,
                )
            }
        };
        outcomes_in_three_stages.push(outcome_of_the_call(&in_three_stages.call));
    }
    assert_eq!(
        outcomes_in_three_stages, outcomes_in_the_lock,
        "三段首尾相接每次调用的结局与锁里推完的入口相同"
    );
    assert!(
        batches_out_of_the_lock > 0,
        "前提：有交到锁外的批（这一串里整理真推过）"
    );
    let operations_in_the_lock = pool_in_the_lock.retained_operations();
    let operations_in_three_stages = pool_in_three_stages.retained_operations();
    assert_eq!(
        operations_in_three_stages.len(),
        operations_in_the_lock.len(),
        "两条路录下的块操作一样多"
    );
    assert!(
        operations_in_three_stages == operations_in_the_lock,
        "两条路写出的块序列逐字节相同"
    );
}

/// 有一批在锁外时再取输入：在换设置之前就拒（`ABatchOfThisSessionIsOutOfTheLock`），这期间请求的切换照旧待生效、生效的那一组没换；
/// 锁外那一批回锁照常结束（`end_the_batch` 的「设置一批之内不换」那一判没被第二个调用者撞到）。
#[test]
fn a_second_take_of_the_input_while_a_batch_is_out_of_the_lock_is_refused_before_the_switch_requested_meanwhile_is_put_in_effect(
) {
    let (_pool, mut pending, mut devices) =
        pool_with_a_batch_issuing_on_the_segment_of_the_first_publish(
            "out-of-the-lock-second-take-refused-before-the-switch",
        );
    let caller = CallerForTests::with_nothing_waiting();
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let in_effect_when_the_batch_began = pending.session().background_compute().in_effect().clone();
    pending
        .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
            BackgroundComputeSettings::PRODUCT,
            None,
            BackgroundGcCpuCompensationThreshold {
                allocated_percent_of_the_capacity: 50,
            },
        )
        .expect("切到 cpu、补跑阈值 50% 照收");
    let take_while_a_batch_is_out_of_the_lock = pending
        .take_the_input_of_a_background_compaction_batch(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            CompactionYieldFloor::NO_FLOOR,
            &caller,
        );
    assert!(
        matches!(
            take_while_a_batch_is_out_of_the_lock,
            Err(BackgroundCompactionInputRefused::ABatchOfThisSessionIsOutOfTheLock)
        ),
        "第二个取输入的拒成有一批在锁外：{take_while_a_batch_is_out_of_the_lock:?}"
    );
    assert_eq!(
        pending.session().background_compute().in_effect(),
        &in_effect_when_the_batch_began,
        "拒在换设置之前：生效的那一组还是锁外那一批开头读的那一组"
    );
    assert!(
        pending
            .session()
            .background_compute()
            .requested_from_the_next_batch()
            .is_some(),
        "切换照旧待生效，下一批开头才换上"
    );
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(stop_of(&run), CompactionRunStop::OnePublishOfTheTripDone);
    assert_eq!(
        run.background_compute_of_each_batch.len(),
        1,
        "锁外那一批照它开头读的设置结束、记了账"
    );
    assert!(
        !pending.session().compaction.has_a_batch_out_of_the_lock(),
        "回锁之后这个会话不再有一批在锁外"
    );
}

/// 有一批在锁外时显式整理入口一个写都不发（`NotRunWhileABatchOfThisSessionIsOutOfTheLock`），锁外那一批回锁照常发。
#[test]
fn the_explicit_compaction_entry_while_a_batch_is_out_of_the_lock_publishes_nothing() {
    let (_pool, mut pending, mut devices) =
        pool_with_a_batch_issuing_on_the_segment_of_the_first_publish(
            "out-of-the-lock-explicit-entry-refused",
        );
    let caller = CallerForTests::with_nothing_waiting();
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let txg_before = current_of(pending.session()).root.checkpoint_txg;
    let explicit = pending
        .session_for_tests_only()
        .run_compaction(&mut devices)
        .expect("盘表是挂载的那一份、现行那一版有树");
    assert!(
        matches!(
            explicit.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::NotRunWhileABatchOfThisSessionIsOutOfTheLock
            )
        ) && explicit.versions_published.is_empty()
            && explicit.background_compute_of_each_batch.is_empty(),
        "显式入口一个写都不发、一批都不开：{:?}",
        explicit.outcome
    );
    assert_eq!(
        current_of(pending.session()).root.checkpoint_txg,
        txg_before,
        "现行那一版没动"
    );
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(run.versions_published.len(), 1, "锁外那一批回锁照常签发");
}

/// 有一批在锁外时锁里推完的两个后台入口都不起跑（`ABatchOfThisSessionIsOutOfTheLock`），不算一次叫起、不取报警。
#[test]
fn the_background_entries_in_the_lock_while_a_batch_is_out_of_the_lock_do_not_start() {
    let (_pool, mut pending, mut devices) =
        pool_with_a_batch_issuing_on_the_segment_of_the_first_publish(
            "out-of-the-lock-background-entries-do-not-start",
        );
    let caller = CallerForTests::with_nothing_waiting();
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let after = pending
        .run_compaction_in_the_background_for_at_most_one_publish(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &caller,
        )
        .expect("不起跑不是拒");
    assert!(
        matches!(
            after.call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::ABatchOfThisSessionIsOutOfTheLock,
                alarm_of_the_compaction_stopped_after_a_compute_error: None,
            }
        ) && after.pending_changes_published_before_the_run.is_none(),
        "攒批会话的后台入口不起跑：{:?}",
        after.call
    );
    let call_of_the_session = pending
        .session_for_tests_only()
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &caller,
        )
        .expect("不起跑不是拒");
    assert!(
        matches!(
            call_of_the_session,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::ABatchOfThisSessionIsOutOfTheLock,
                ..
            }
        ),
        "会话的后台入口不起跑：{call_of_the_session:?}"
    );
    let computed = computed_reading_in_the_session(*ticket, &pending, &devices);
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(run.versions_published.len(), 1, "锁外那一批回锁照常签发");
}

/// 走 GPU 的一批：锁外那一段回会话锁里读盘时、交回时，GPU 实现那把锁都没人持着（每一段调用借一次、调完就放）。
#[test]
fn the_gpu_implementation_is_not_lent_while_the_batch_reads_in_the_session_lock_nor_once_the_computation_hands_back(
) {
    let mut pool = build_pool("out-of-the-lock-gpu-lock-free-at-the-hand-back");
    let (mut pending, mut devices, shared) = session_with_pending_changes_on_the_gpu_of(&mut pool);
    force_the_compaction_switches(
        &mut pending,
        CompactionRunForcing::ForcedToStartForTests,
        Some(region_starting_at(SEGMENT_OF_THE_FIRST_PUBLISH)),
        1,
    );
    let caller = CallerForTests::with_nothing_waiting();
    create_one_inode(&mut pending, &mut devices, &caller);
    let ticket = ticket_taken(&mut pending, &mut devices, &caller);
    let mut reads_in_the_session_lock = 0_usize;
    let mut was_the_gpu_implementation_free_at_every_read = true;
    let computed = ticket.compute_out_of_the_lock(&mut |reads| {
        reads_in_the_session_lock += 1;
        was_the_gpu_implementation_free_at_every_read &= shared.try_lock().is_ok();
        pending.read_the_units_for_the_batch_out_of_the_lock(devices.as_slice(), reads)
    });
    assert!(
        reads_in_the_session_lock >= 1,
        "前提：强制选的那一段要回会话锁里读一次"
    );
    assert!(
        was_the_gpu_implementation_free_at_every_read,
        "回会话锁读盘时 GPU 实现那把锁没人持着"
    );
    assert!(
        shared.try_lock().is_ok(),
        "交回时 GPU 实现那把锁没人持着（三段怎么交的已取完、锁已放）"
    );
    assert!(
        shared
            .lock()
            .expect("替身没在别处 panic 过")
            .entries_handed_in_to(GpuComputedSegment::RegionRecordClassification)
            > 0,
        "前提：这一批经 GPU 实现分类了"
    );
    let run = run_of(relocked(&mut pending, &mut devices, computed, &caller));
    assert_eq!(run.versions_published.len(), 1, "回锁照常签发");
}
