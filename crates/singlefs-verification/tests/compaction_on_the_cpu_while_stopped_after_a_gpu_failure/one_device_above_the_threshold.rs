//! 只有一块盘越过阈值而全池合计不越时照样补跑（越线的是哪一块盘都一样）：在分配器里把一块盘单元区末尾的空槽标成已分配，把它的占用抬过阈值。

use super::*;

/// 「任一块盘超过就补跑」：停着、阈值 3%，第一块盘（R 那块）的占用抬过 3%，第二块盘不越线，两块盘的已分配与容量各自相加也不越线
/// （逐盘比与全池相加判得不一样）：会话读得出最满的是第一块盘、它的已分配字节与容量；显式入口照常推，每一批在 CPU 上补跑，记账里的读数报的也是第一块盘；
/// 补跑的批数等于这一趟的批数。
#[test]
fn a_compaction_stopped_after_a_gpu_failure_runs_on_the_cpu_when_one_device_is_above_the_threshold_though_the_pool_as_a_whole_is_not(
) {
    let threshold = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 3,
    };
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensated-when-one-device-is-above-the-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(threshold),
        );
    allocate_the_tail_of_a_device_just_above_percent(
        &mut session,
        DeviceIdentity(0),
        threshold.allocated_percent_of_the_capacity,
    );
    let every_device = every_device_counted_by_the_test(&session);
    let [region_device, other_device] = every_device.as_slice() else {
        panic!("池是两块盘：{every_device:?}");
    };
    assert!(
        region_device.device == DeviceIdentity(0)
            && share_is_above_percent(
                region_device.allocated_bytes,
                region_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            )
            && !share_is_above_percent(
                other_device.allocated_bytes,
                other_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            )
            && !share_is_above_percent(
                region_device.allocated_bytes + other_device.allocated_bytes,
                region_device.capacity_bytes + other_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            ),
        "第一块盘越过 3%、第二块盘不越、两块相加也不越：{every_device:?}"
    );
    let reading_before_the_run = session.gc_cpu_compensation_reading();
    assert_eq!(
        reading_before_the_run,
        GcCpuCompensationReading {
            fullest_device: *region_device,
            threshold,
        },
        "会话读得出最满的是第一块盘、它的已分配字节与容量，阈值是挂载给的 3%"
    );
    assert!(
        reading_before_the_run.is_the_allocated_share_above_the_threshold(),
        "第一块盘越线：补跑那一判为真（{reading_before_the_run:?}）"
    );
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget })
                if budget == PUBLISHES_OF_THE_COMPENSATING_RUN
        ) && run.batches_published > 0,
        "一块盘越线就补跑：照常推、搬了批、推到预算用完（{:?}）",
        run.outcome
    );
    assert!(
        !run.background_compute_of_each_batch.is_empty()
            && run.background_compute_of_each_batch.iter().all(|batch| matches!(
                batch.path_taken,
                BackgroundComputePathTaken::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading)
                ) if reading.fullest_device.device == DeviceIdentity(0)
                    && reading.threshold == threshold
                    && reading.is_the_allocated_share_above_the_threshold()
            )),
        "每一批在 CPU 上补跑，读数报的是越线的第一块盘：{:?}",
        run.background_compute_of_each_batch
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "补跑不经 GPU 实现"
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
        "整理照旧停着，补跑的批数就是这一趟的批数"
    );
}

/// 「任一块盘超过就补跑」，最满的是第二块盘：停着、阈值 3%，把第二块盘的占用抬过 3%、第一块盘（R 那块）不越线——会话读得出最满的是
/// 第二块盘；显式入口照常推，每一批在 CPU 上补跑，记账里的读数报的是第二块盘。只看第一块盘（「永远取第一块」那一种改坏）就照旧停着。
#[test]
fn a_compaction_stopped_after_a_gpu_failure_runs_on_the_cpu_when_the_second_device_is_the_one_above_the_threshold(
) {
    let threshold = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 3,
    };
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensated-when-the-second-device-is-above-the-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(threshold),
        );
    allocate_the_tail_of_a_device_just_above_percent(
        &mut session,
        DeviceIdentity(1),
        threshold.allocated_percent_of_the_capacity,
    );
    let every_device = every_device_counted_by_the_test(&session);
    let [region_device, second_device] = every_device.as_slice() else {
        panic!("池是两块盘：{every_device:?}");
    };
    assert!(
        second_device.device == DeviceIdentity(1)
            && share_is_above_percent(
                second_device.allocated_bytes,
                second_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            )
            && !share_is_above_percent(
                region_device.allocated_bytes,
                region_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            ),
        "第二块盘越过 3%、第一块盘不越：{every_device:?}"
    );
    let reading_before_the_run = session.gc_cpu_compensation_reading();
    assert_eq!(
        reading_before_the_run,
        GcCpuCompensationReading {
            fullest_device: *second_device,
            threshold,
        },
        "会话读得出最满的是第二块盘、它的已分配字节与容量"
    );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget })
                if budget == PUBLISHES_OF_THE_COMPENSATING_RUN
        ) && run.batches_published > 0,
        "第二块盘越线就补跑：照常推、搬了批、推到预算用完（{:?}）",
        run.outcome
    );
    assert!(
        !run.background_compute_of_each_batch.is_empty()
            && run.background_compute_of_each_batch.iter().all(|batch| matches!(
                batch.path_taken,
                BackgroundComputePathTaken::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading)
                ) if reading.fullest_device.device == DeviceIdentity(1)
                    && reading.is_the_allocated_share_above_the_threshold()
            )),
        "每一批在 CPU 上补跑，读数报的是越线的第二块盘：{:?}",
        run.background_compute_of_each_batch
    );
}
