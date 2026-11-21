//! 补跑阈值超过 100% 挂载与切换都拒；带新阈值的切换解除停下，之后的批照新设置走。

use super::*;

/// 阈值超过 100%（101%）：选 `cpu` 的挂载也在读任何一块盘之前拒、报出给的阈值，盘上逐字节不变；100% 照收。
/// 挂着时带 101% 的切换同样被拒，生效的阈值照旧是 100%、没有待生效的切换。
#[test]
fn a_gc_cpu_compensation_threshold_above_one_hundred_percent_is_refused_at_the_mount_and_at_a_switch(
) {
    let mut pool = build_pool("gc-cpu-compensation-threshold-above-the-whole-capacity");
    let above_the_whole_capacity = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 101,
    };
    let the_whole_capacity = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 100,
    };
    let mut devices = pool.reopen_recorded();
    let before_the_mount = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused =
        mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold(
            &parameters(),
            &mut devices,
            BackgroundComputeSettings::PRODUCT,
            None,
            above_the_whole_capacity,
        );
    assert!(
        matches!(
            refused,
            Err(WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings(
                BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
                    threshold
                }
            )) if threshold == above_the_whole_capacity
        ),
        "101% 拒绝挂载、报出给的阈值：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_mount,
        "在读任何一块盘之前拒：盘上逐字节不变"
    );
    let mounted =
        mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold(
            &parameters(),
            &mut devices,
            BackgroundComputeSettings::PRODUCT,
            None,
            the_whole_capacity,
        )
        .expect("100% 照收");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    assert_eq!(
        session
            .background_compute()
            .in_effect()
            .gc_cpu_compensation_threshold(),
        the_whole_capacity,
        "挂载给的阈值生效"
    );
    let switch = session
        .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
            BackgroundComputeSettings::PRODUCT,
            None,
            above_the_whole_capacity,
        );
    pool.devices = Some(devices);
    assert_eq!(
        switch,
        Err(
            BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
                threshold: above_the_whole_capacity
            }
        ),
        "101% 的切换被拒"
    );
    assert!(
        session
            .background_compute()
            .requested_from_the_next_batch()
            .is_none()
            && session
                .background_compute()
                .in_effect()
                .gc_cpu_compensation_threshold()
                == the_whole_capacity,
        "被拒的切换不留待生效的、生效的阈值照旧"
    );
}

/// 停着、不超过阈值（取默认 85%）时收到带新阈值的切换（选 `cpu`、阈值 90%）：当场解除停下；下一趟每一批照新设置走——请求的后端是 `cpu`、
/// 实际走的是「设置选的就是 cpu」（不是补跑），GPU 实现一次都没被调，生效的设置换成切换那一组、阈值 90%。
/// 之后不给阈值的切换沿用 90%。
#[test]
fn a_switch_with_a_new_gc_cpu_compensation_threshold_lifts_the_stop_and_the_next_batches_follow_the_new_settings(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-stop-lifted-by-a-switch-with-a-new-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
        );
    let switched_threshold = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 90,
    };
    session
        .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
            BackgroundComputeSettings::PRODUCT,
            None,
            switched_threshold,
        )
        .expect("选 cpu、阈值 90%：切换收下");
    assert!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .is_empty(),
        "停着时收到通过校验的切换：当场解除停下"
    );
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    let unreleased_in_the_region_before = unreleased_records_in(&session, region).len();
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        run.batches_published > 0
            && unreleased_records_in(&session, region).len() < unreleased_in_the_region_before,
        "解除之后照常推、意图推进：{:?}",
        run.outcome
    );
    assert!(
        !run.background_compute_of_each_batch.is_empty()
            && run
                .background_compute_of_each_batch
                .iter()
                .all(|batch| *batch
                    == BackgroundComputeOfOneBatch {
                        task: BackgroundTask::CompactionRelocation,
                        requested_backend: BackgroundComputeBackend::Cpu,
                        path_taken: BackgroundComputePathTaken::Cpu(
                            ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
                        ),
                        compute_error_that_abandoned_the_batch: None,
                    }),
        "从下一批起照切换那一组走：请求 cpu、照设置走 CPU，不是补跑：{:?}",
        run.background_compute_of_each_batch
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "切到 cpu 之后不经 GPU 实现"
    );
    let in_effect = session.background_compute().in_effect();
    assert_eq!(
        (
            in_effect.settings().clone(),
            in_effect.gc_cpu_compensation_threshold()
        ),
        (BackgroundComputeSettings::PRODUCT, switched_threshold),
        "生效的换成切换那一组：cpu、阈值 90%"
    );
    session
        .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
        .expect("选 cpu：切换收下");
    assert_eq!(
        session
            .background_compute()
            .requested_from_the_next_batch()
            .map(|requested| requested.gc_cpu_compensation_threshold()),
        Some(switched_threshold),
        "不给阈值的切换沿用 90%，不回到默认"
    );
}
