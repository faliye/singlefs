//! 池子超过阈值时，GPU 出错停着的整理在 CPU 上推进意图、后台入口起跑并在 CPU 上跑那一批；不超过时照旧停着。

use super::*;

/// 停着且最满的那一块盘超过阈值（挂载给 0%）：显式入口照常推，每一批走 CPU、记账写明是停着时补跑并带那一判的读数（请求的后端照旧是 `gpu`），
/// GPU 实现一次都没被调；出错停下时删了意图，这一趟在同一个 R 上从头签发、照常推进（R 里未释放的记录变少、意图还在 R 上）；整理照旧停着、报的还是那个错，补跑的批数等于这一趟的批数；
/// 会话读得出此刻最满的那一块盘、它的已分配字节与容量（与用例自己从分配器逐盘计数挑的相同）与阈值 0%。
#[test]
fn a_compaction_stopped_after_a_gpu_failure_moves_its_intent_on_the_cpu_while_the_pool_is_above_the_threshold(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensated-on-the-cpu-above-the-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    let reading_before_the_run = session.gc_cpu_compensation_reading();
    assert_eq!(
        reading_before_the_run,
        GcCpuCompensationReading {
            fullest_device: fullest_device_counted_by_the_test(&session),
            threshold: THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE,
        },
        "此刻的占用读得出、数对：最满的那一块盘与它的已分配、容量就是分配器逐盘计数挑出的那一块，阈值是挂载给的 0%"
    );
    assert!(
        reading_before_the_run.fullest_device.allocated_bytes > 0
            && reading_before_the_run.is_the_allocated_share_above_the_threshold(),
        "最满的那一块盘有已分配的字节：超过 0%（{reading_before_the_run:?}）"
    );
    let unreleased_in_the_region_before = unreleased_records_in(&session, region).len();
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        run.batches_published > 0 && !run.versions_published.is_empty(),
        "停着而超过阈值：照常推、搬了批（{:?}）",
        run.outcome
    );
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget })
                if budget == PUBLISHES_OF_THE_COMPENSATING_RUN
        ),
        "推到预算用完，不停在「此前出错停着」上：{:?}",
        run.outcome
    );
    for batch in &run.background_compute_of_each_batch {
        let BackgroundComputePathTaken::Cpu(
            ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading),
        ) = batch.path_taken
        else {
            panic!("停着时补跑的每一批走 CPU、记成补跑：{batch:?}");
        };
        assert_eq!(
            (
                batch.task,
                batch.requested_backend,
                &batch.compute_error_that_abandoned_the_batch
            ),
            (
                BackgroundTask::CompactionRelocation,
                BackgroundComputeBackend::Gpu,
                &None
            ),
            "设置照旧是停下时生效的 gpu 那一组、这一批没出错：{batch:?}"
        );
        assert!(
            reading.threshold == THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE
                && reading.is_the_allocated_share_above_the_threshold(),
            "每一批的记账带那一判的读数：超过阈值 0%（{reading:?}）"
        );
    }
    let first_batch_reading = match run.background_compute_of_each_batch.first() {
        Some(BackgroundComputeOfOneBatch {
            path_taken:
                BackgroundComputePathTaken::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading),
                ),
            ..
        }) => *reading,
        other => panic!("至少一批、走的是补跑：{other:?}"),
    };
    assert_eq!(
        first_batch_reading, reading_before_the_run,
        "头一批那一判读的就是开跑之前那一刻的占用与阈值（删除队列是空的，其间没有发布）"
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "补跑不经 GPU 实现：一段都没交给它"
    );
    assert!(
        unreleased_records_in(&session, region).len() < unreleased_in_the_region_before,
        "从头签发、照常推进：R 里未释放的记录变少（{unreleased_in_the_region_before} 条起）"
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .is_some_and(|(_, intent)| intent.region == region),
        "预算用完时意图还在 R 上，下一趟接着推"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error,
            batches_compensated_on_the_cpu: u64::try_from(
                run.background_compute_of_each_batch.len()
            )
            .expect("批数"),
        }],
        "整理照旧停着（报警那一态）、报的还是那个错，补跑的批数就是这一趟的批数"
    );
}

/// 停着且超过阈值（挂载给 0%）时，后台入口照 D26（后台整理与放置回收） 已定项 1 的三水位照常判起跑（起跑强制开着，一次用户改动记下「该跑了」）：起跑、推至多一批，
/// 那一批在 CPU 上补跑；GPU 实现一次都没被调，补跑的批数加一。
#[test]
fn the_background_entry_starts_a_compaction_stopped_after_a_gpu_failure_and_runs_its_batch_on_the_cpu_while_the_pool_is_above_the_threshold(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensated-on-the-cpu-through-the-background-entry",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("建一个 inode");
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "用户改动之后起跑判据为真，记下「该跑了」"
    );
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { trigger, run } = call else {
        panic!("停着而超过阈值：后台入口照常判起跑、起跑了：{call:?}");
    };
    assert_eq!(trigger, BackgroundCompactionTrigger::DueAfterAUserChange);
    assert_eq!(
        run.batches_published, 1,
        "用户改动之后那一趟推至多一批：{:?}",
        run.outcome
    );
    assert!(
        !run.background_compute_of_each_batch.is_empty()
            && run
                .background_compute_of_each_batch
                .iter()
                .all(|batch| matches!(
                    batch.path_taken,
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                    )
                )),
        "那一趟每一批在 CPU 上补跑：{:?}",
        run.background_compute_of_each_batch
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "补跑不经 GPU 实现"
    );
    assert_eq!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .iter()
            .map(|stopped| stopped.batches_compensated_on_the_cpu)
            .collect::<Vec<u64>>(),
        vec![u64::try_from(run.background_compute_of_each_batch.len()).expect("批数")],
        "整理照旧停着，补跑的批数就是这一趟的批数"
    );
}

/// 停着且不超过阈值（不给阈值，取默认 85%）：显式入口一个写都不发、一批都不开（没有一批的记账），停在「出错停着」上，盘上逐字节不变；
/// 一次用户改动记下「该跑了」之后后台入口也不起跑、「该跑了」留着；GPU 实现一次都没被调，补跑的批数是 0，出错停下时删掉的意图没再签；
/// 会话读得出阈值是默认的 85%、占用不超过它。
#[test]
fn a_compaction_stopped_after_a_gpu_failure_stays_stopped_while_the_pool_is_not_above_the_threshold(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-stays-stopped-at-or-below-the-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
        );
    let reading = session.gc_cpu_compensation_reading();
    assert_eq!(
        reading,
        GcCpuCompensationReading {
            fullest_device: fullest_device_counted_by_the_test(&session),
            threshold: BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
        },
        "不给阈值：取默认"
    );
    assert_eq!(
        BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT.allocated_percent_of_the_capacity, 85,
        "默认 85%（用户 2026-10-28 定案）"
    );
    assert!(
        !reading.is_the_allocated_share_above_the_threshold(),
        "这个池子最满的那一块盘的占用也不超过 85%：{reading:?}"
    );
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    let before_the_run = disk_snapshot(&pool.memory_pool(), &pool.stream);
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.versions_published.is_empty()
            && run.background_compute_of_each_batch.is_empty(),
        "不超过阈值：照旧停着、一个写都不发、一批都不开：{:?}，{:?}",
        run.outcome,
        run.background_compute_of_each_batch
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_run,
        "盘上逐字节不变：系统配置槽、根环、录制流的步数"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("建一个 inode");
    let after_the_user_change = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    assert!(
        matches!(
            call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ),
        "后台入口也不起跑：{call:?}"
    );
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "「该跑了」留着，解除之后下一次调用再取"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        after_the_user_change,
        "后台入口没起跑：盘上逐字节不变"
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "停着不换回 GPU 也不补跑：一段都没交给实现"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error,
            batches_compensated_on_the_cpu: 0,
        }],
        "整理照旧停着，一批都没补跑"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_none(),
        "出错停下时删了意图，停着不补跑就不再签：盘上与分配器这一侧都没有活意图（R 原来是 {region:?}）"
    );
}
