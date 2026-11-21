use super::*;

fn window_of_wall(wall: Duration) -> CallWindow {
    CallWindow {
        wall,
        process_cpu: Duration::ZERO,
        gc_thread_cpu: Duration::ZERO,
        foreground_thread_cpu: Duration::ZERO,
        main_thread_cpu: Duration::ZERO,
    }
}

/// 第九节变异「只计发了批的 GC 调用」在这里红：不起跑的那一次调用的 1 秒照算进量 W。
#[test]
fn the_gc_wall_clock_counts_the_calls_that_published_no_batch() {
    let windows = [
        window_of_wall(Duration::from_millis(250)),
        window_of_wall(Duration::from_secs(1)),
    ];
    let records = [
        call_record_with(3, Vec::new()),
        call_record_with(0, Vec::new()),
    ];
    assert_eq!(
        gc_wall_clock_of_the_run(&windows, &records),
        Duration::from_millis(1250)
    );
}

#[test]
fn the_most_compaction_publishes_between_two_user_changes_are_summed_per_change() {
    let mut after_the_first_change = call_record_with(12, Vec::new());
    after_the_first_change.after_the_user_change = 0;
    let mut again_after_the_first_change = call_record_with(8, Vec::new());
    again_after_the_first_change.after_the_user_change = 0;
    let mut after_the_next_change = call_record_with(5, Vec::new());
    after_the_next_change.after_the_user_change = 1;
    assert_eq!(
        most_compaction_publishes_between_two_user_changes(&[
            after_the_first_change,
            again_after_the_first_change,
            after_the_next_change
        ]),
        12 + 8
    );
}

fn gpu_record_giving_work_to(cards: Vec<GpuDeviceIdentifier>) -> BackgroundComputeOfOneBatch {
    let mut work = singlefs_core::background_compute::GpuWorkOfTheBatch::default();
    work.submissions_on_the_gpus.unit_checksum_verification = GpuSubmissionsOfOneSegment {
        submissions_completed: 1,
        pieces_submitted: 1,
        resubmissions_after_timeouts: 0,
        recuts_after_running_out_of_memory: 0,
        devices_given_work: cards,
    };
    BackgroundComputeOfOneBatch {
        task: singlefs_core::background_compute::BackgroundTask::CompactionRelocation,
        requested_backend: BackgroundComputeBackend::Gpu,
        path_taken: BackgroundComputePathTaken::Gpu(work),
        compute_error_that_abandoned_the_batch: None,
    }
}

/// 第九节变异「甲类锚点『分到活的卡』改成不核」在这里红。
#[test]
fn the_batch_giving_work_to_a_second_card_breaks_the_anchor() {
    let lowest = gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0");
    let higher = gpu_device_identifier_of_the_pci_bus_address("0000:02:00.0");
    let on_the_lowest = [call_record_with(
        1,
        vec![gpu_record_giving_work_to(vec![lowest.clone()])],
    )];
    assert!(gpu_work_anchors(&on_the_lowest, &lowest).hold());
    let on_both = [call_record_with(
        1,
        vec![gpu_record_giving_work_to(vec![lowest.clone(), higher])],
    )];
    let anchors = gpu_work_anchors(&on_both, &lowest);
    assert_eq!(anchors.segments_given_to_a_card_other_than_the_lowest, 1);
    assert!(!anchors.hold());
}

#[test]
fn the_path_anchor_counts_cpu_batches_on_a_cpu_arm_and_gpu_batches_on_a_gpu_arm() {
    let lowest = gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0");
    let records = [call_record_with(
        1,
        vec![gpu_record_giving_work_to(vec![lowest])],
    )];
    let on_a_gpu_arm = path_anchor(
        &records,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
    );
    assert_eq!(on_a_gpu_arm.compute_records_on_the_path_of_the_settings, 1);
    assert!(on_a_gpu_arm.holds());
    let on_a_cpu_arm = path_anchor(&records, ComputeOfTheArm::Cpu);
    assert_eq!(on_a_cpu_arm.compute_records_on_the_path_of_the_settings, 0);
    assert!(!on_a_cpu_arm.holds());
}

fn cpu_record_requested_by_the_settings() -> BackgroundComputeOfOneBatch {
    BackgroundComputeOfOneBatch {
        task: singlefs_core::background_compute::BackgroundTask::CompactionRelocation,
        requested_backend: BackgroundComputeBackend::Cpu,
        path_taken: BackgroundComputePathTaken::Cpu(
            ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings,
        ),
        compute_error_that_abandoned_the_batch: None,
    }
}

/// 登记第十二节修订第 2 条：条数比批数多（签发、删意图、停在判据上的那几项也记账）而每一项都照设置走，判过；有一项走错，
/// 或条数比批数少，各判不过。
#[test]
fn the_path_anchor_holds_with_more_records_than_batches_and_breaks_on_a_wrong_path_or_fewer_records(
) {
    let lowest = gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0");
    let three_on_the_gpu = [call_record_with(
        1,
        vec![
            gpu_record_giving_work_to(vec![lowest.clone()]),
            gpu_record_giving_work_to(vec![lowest.clone()]),
            gpu_record_giving_work_to(vec![lowest.clone()]),
        ],
    )];
    let more_records_than_batches = path_anchor(
        &three_on_the_gpu,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
    );
    assert_eq!(more_records_than_batches.compute_records, 3);
    assert_eq!(more_records_than_batches.batches, 1);
    assert!(more_records_than_batches.holds());
    let one_on_the_cpu = [call_record_with(
        1,
        vec![
            gpu_record_giving_work_to(vec![lowest.clone()]),
            cpu_record_requested_by_the_settings(),
            gpu_record_giving_work_to(vec![lowest.clone()]),
        ],
    )];
    let one_on_the_wrong_path = path_anchor(
        &one_on_the_cpu,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
    );
    assert_eq!(
        one_on_the_wrong_path.compute_records_on_the_path_of_the_settings,
        2
    );
    assert!(!one_on_the_wrong_path.holds());
    let one_record_for_two_batches = [call_record_with(
        2,
        vec![gpu_record_giving_work_to(vec![lowest])],
    )];
    let fewer_records_than_batches = path_anchor(
        &one_record_for_two_batches,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
    );
    assert_eq!(fewer_records_than_batches.compute_records, 1);
    assert_eq!(fewer_records_than_batches.batches, 2);
    assert!(!fewer_records_than_batches.holds());
}

fn gpu_record_with_unit_checksum_entries(
    confirmed_by_both_kernels: u64,
    uncertain_this_round: u64,
    judged_by_the_cpu_reference: u64,
) -> BackgroundComputeOfOneBatch {
    let work = singlefs_core::background_compute::GpuWorkOfTheBatch {
        unit_checksum_verification: singlefs_core::background_compute::GpuEntryCounts {
            confirmed_by_both_kernels,
            uncertain_this_round,
            judged_by_the_cpu_reference,
        },
        region_record_classification: singlefs_core::background_compute::GpuEntryCounts::default(),
        live_slots_per_cluster_segment: singlefs_core::background_compute::GpuEntryCounts::default(
        ),
        units_whose_checksum_mismatched_on_the_gpu: 0,
        submissions_on_the_gpus: GpuSubmissionsOfTheBatch::default(),
    };
    BackgroundComputeOfOneBatch {
        task: singlefs_core::background_compute::BackgroundTask::CompactionRelocation,
        requested_backend: BackgroundComputeBackend::Gpu,
        path_taken: BackgroundComputePathTaken::Gpu(work),
        compute_error_that_abandoned_the_batch: None,
    }
}

/// 甲类第 3 行：确认 + 不确定 + 交 CPU 判 = 交进去的条目数；少一条、或包装记下的批数与记账里走 GPU 的批数不等，各记一批不成立。
#[test]
fn the_decisions_of_a_rejudged_batch_must_add_up_to_the_entries_handed_in() {
    let records = [call_record_with(
        1,
        vec![gpu_record_with_unit_checksum_entries(2, 1, 1)],
    )];
    let four_checksums = EntriesHandedIn {
        unit_checksums: 4,
        region_records: 0,
        cluster_segments: 0,
    };
    assert_eq!(
        batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(&records, &[four_checksums]),
        0
    );
    let five_checksums = EntriesHandedIn {
        unit_checksums: 5,
        ..four_checksums
    };
    assert_eq!(
        batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(&records, &[five_checksums]),
        1
    );
    assert_eq!(
        batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(
            &records,
            &[four_checksums, four_checksums]
        ),
        1
    );
}

/// 乙类第 5 行：CPU 臂 GC 线程 CPU ÷ GC 挂钟 ≥ 0.9。
#[test]
fn the_cpu_arm_thread_cpu_covers_the_wall_at_ninety_percent_and_not_below() {
    assert!(cpu_arm_thread_cpu_covers_the_wall(
        Duration::from_millis(900),
        Duration::from_millis(1000)
    ));
    assert!(!cpu_arm_thread_cpu_covers_the_wall(
        Duration::from_millis(899),
        Duration::from_millis(1000)
    ));
}

/// 乙类第 6 行与作废第 2 条：挂钟 < GC 线程 CPU、进程口径 < GC 线程口径的调用各记一次。
#[test]
fn the_inclusion_checks_count_a_call_whose_thread_cpu_exceeds_its_wall_or_its_process_cpu() {
    let window =
        |wall_milliseconds: u64, process_milliseconds: u64, gc_thread_milliseconds: u64| {
            CallWindow {
                wall: Duration::from_millis(wall_milliseconds),
                process_cpu: Duration::from_millis(process_milliseconds),
                gc_thread_cpu: Duration::from_millis(gc_thread_milliseconds),
                foreground_thread_cpu: Duration::ZERO,
                main_thread_cpu: Duration::ZERO,
            }
        };
    assert_eq!(calls_failing_the_inclusion_checks(&[window(10, 9, 8)]), 0);
    assert_eq!(
        calls_failing_the_inclusion_checks(&[window(10, 9, 8), window(7, 9, 8), window(10, 7, 8)]),
        2
    );
}

/// 作废第 4 条：出错放弃的一批记进锚点；作废的重复第 3 次（出错放弃的也算一次）这一档停下。
#[test]
fn the_gpu_work_anchors_count_a_batch_abandoned_after_a_compute_error_and_the_third_void_stops_the_tier(
) {
    let mut abandoned = gpu_record_with_unit_checksum_entries(1, 0, 0);
    abandoned.compute_error_that_abandoned_the_batch =
        Some(singlefs_core::background_compute::BackgroundComputeError::InjectedByTheTestOnlyHook);
    let lowest = gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0");
    let anchors = gpu_work_anchors(&[call_record_with(1, vec![abandoned])], &lowest);
    assert_eq!(anchors.batches_abandoned_after_a_compute_error, 1);
    assert!(!the_tier_stops_after_the_voided_repeats(2, 0));
    assert!(the_tier_stops_after_the_voided_repeats(3, 0));
    assert!(the_tier_stops_after_the_voided_repeats(3, 3));
}

/// 第八节轨迹「每块盘的全空聚簇段数」（登记第十二节修订第 5 条）：每次调用开始前、返回后各一个读数，按盘各报峰值、
/// 返回后比开始前多的调用数（相等不算）、期末值（最后一次调用返回后那一个）。
#[test]
fn the_empty_segment_trajectory_counts_calls_ending_with_more_empty_segments_per_device() {
    let with_readings = |before: Vec<u64>, after: Vec<u64>| {
        let mut record = call_record_with(1, Vec::new());
        record.empty_segments_before_the_call = before;
        record.empty_segments_after_the_call = after;
        record
    };
    let records = [
        with_readings(vec![10, 20], vec![9, 20]),
        with_readings(vec![9, 20], vec![12, 21]),
        with_readings(vec![12, 21], vec![12, 19]),
    ];
    let trajectories = empty_segments_trajectory_of_each_device(&records);
    assert_eq!(
        trajectories,
        vec![
            EmptySegmentsTrajectoryOfADevice {
                peak: 12,
                calls_gaining_empty_segments: 1,
                last: 12,
            },
            EmptySegmentsTrajectoryOfADevice {
                peak: 21,
                calls_gaining_empty_segments: 1,
                last: 19,
            },
        ]
    );
    assert_eq!(
        empty_segments_fields(&trajectories),
        "empty_segments_peak=12,21 empty_segments_calls_gaining=1,1 empty_segments_last=12,19"
    );
    assert!(empty_segments_trajectory_of_each_device(&[]).is_empty());
}

/// 第八节轨迹：预算读数照「B_gc 减这次用户改动之后的整理发布数」推，每次调用前后各一个；不确定条目每一项 GPU 记账一个；峰值、大于 0 的次数、期末值。
#[test]
fn the_trajectories_report_the_peak_the_positive_readings_and_the_last_reading() {
    let mut first_call = call_record_with(12, Vec::new());
    first_call.after_the_user_change = 0;
    let mut second_call = call_record_with(8, Vec::new());
    second_call.after_the_user_change = 0;
    let mut after_the_next_change = call_record_with(0, Vec::new());
    after_the_next_change.after_the_user_change = 1;
    let records = [first_call, second_call, after_the_next_change];
    assert_eq!(
        budget_left_before_and_after_each_call(&records),
        [20, 8, 8, 0, 20, 20]
    );
    assert_eq!(
        trajectory_of_the_readings(&budget_left_before_and_after_each_call(&records)),
        TrajectoryOfTheReadings {
            peak: 20,
            positive_readings: 5,
            last: 20,
        }
    );
    let gpu_records = [call_record_with(
        2,
        vec![
            gpu_record_with_unit_checksum_entries(3, 2, 0),
            gpu_record_with_unit_checksum_entries(5, 0, 0),
        ],
    )];
    assert_eq!(uncertain_entries_of_each_gpu_record(&gpu_records), [2, 0]);
    assert_eq!(
        trajectory_of_the_readings(&[]),
        TrajectoryOfTheReadings::default()
    );
}
