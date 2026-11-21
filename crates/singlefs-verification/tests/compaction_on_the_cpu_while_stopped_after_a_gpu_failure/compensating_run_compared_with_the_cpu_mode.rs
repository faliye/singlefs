//! GPU 出错停着时在 CPU 上补跑的那一趟，结局与一开始就选 `cpu` 的那一趟逐项相同。

use super::*;

/// D26（后台整理与放置回收） 已定项 13 第 4 条「占用超过阈值在 CPU 上补跑」与七·3 验收「结果与 CPU 模式相同」在替身上：
/// 同样建、同样写的两个池，一个选 `gpu` 在 GPU 上签发意图、注入甲段出错停下（那一批不搬单元、单发一次删意图的发布）、阈值 0% 下在 CPU 上
/// 补跑一趟（`B_gc` 压到 [`PUBLISHES_OF_THE_COMPENSATING_RUN`]，从头签发再推）；另一个选 `cpu`、同一组整理开关签发意图，经只供测试的钩子
/// 在下一批注入一次出错（同样单发一次删意图的发布、整理停下），选 `cpu` 的切换解除停下，再推同样预算的一趟。补跑那一趟每一批都记成停着时补跑，
/// 与 `cpu` 那一趟停在同一条上、发的每一版逐项相同、搬了一样多批，两条录制流逐步相同（盘上逐字节相同）。
#[test]
fn a_compensating_run_on_the_cpu_while_stopped_after_a_gpu_failure_ends_exactly_as_the_cpu_mode() {
    let (mut gpu_pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensating-run-against-the-cpu-mode-gpu",
    );
    let (mut cpu_pool, region_of_the_cpu_pool) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensating-run-against-the-cpu-mode-cpu",
    );
    assert_eq!(
        (gpu_pool.retained_operations(), region),
        (cpu_pool.retained_operations(), region_of_the_cpu_pool),
        "两个池走同一串操作：整理之前字节相同、数据单元最多的是同一段"
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut gpu_session, mut gpu_devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut gpu_pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    gpu_session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let compensating = gpu_session
        .run_compaction(&mut gpu_devices)
        .expect("整理一趟");
    gpu_pool.devices = Some(gpu_devices);
    let mut cpu_devices = cpu_pool.reopen_recorded();
    let cpu_mounted = mount_writable(&parameters(), &mut cpu_devices).expect("可写挂载，选 cpu");
    let (_, cpu_session) = MountedSession::of_the_mount(cpu_mounted);
    let (mut cpu_session, mut cpu_devices) =
        session_with_the_intent_issued_under_the_compaction_switches_of_the_test(
            cpu_session,
            cpu_devices,
            region,
            "在 CPU 上",
        );
    cpu_session.compaction.publishes_since_the_last_user_change = BUDGET_GIVEN_BY_A_USER_CHANGE - 1;
    let abandoning_on_the_cpu = cpu_session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut cpu_devices,
            &mut |background_compute| {
                background_compute.report_a_compute_error_in_the_batch_under_way_for_tests();
            },
        )
        .expect("整理一趟");
    assert!(
        matches!(
            abandoning_on_the_cpu.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ) && abandoning_on_the_cpu.versions_published.len() == 1,
        "选 cpu 那一边注入的出错同样放弃那一批、单发一次删意图的发布：{:?}",
        abandoning_on_the_cpu.outcome
    );
    cpu_session
        .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
        .expect("选 cpu：切换收下，解除停下");
    cpu_session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let cpu_run = cpu_session
        .run_compaction(&mut cpu_devices)
        .expect("整理一趟");
    cpu_pool.devices = Some(cpu_devices);
    assert!(
        compensating.batches_published > 0
            && compensating
                .background_compute_of_each_batch
                .iter()
                .all(|batch| matches!(
                    batch.path_taken,
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                    )
                )),
        "停着而超过阈值 0%：搬了批，每一批都记成停着时在 CPU 上补跑（{:?}）",
        compensating.outcome
    );
    assert_eq!(
        format!("{:?}", compensating.outcome),
        format!("{:?}", cpu_run.outcome),
        "停在同一条上"
    );
    assert_eq!(
        compensating.versions_published, cpu_run.versions_published,
        "发的每一版逐项相同"
    );
    assert_eq!(
        compensating.batches_published, cpu_run.batches_published,
        "搬了一样多批"
    );
    assert_eq!(
        gpu_pool.retained_operations(),
        cpu_pool.retained_operations(),
        "两条录制流逐步相同：盘上逐字节相同"
    );
}
