use super::*;
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, PlacementRefusal};

fn device_reading(capacity_slots: u64, live_slots: u64, available_slots: i128) -> DeviceReading {
    DeviceReading {
        capacity_slots,
        live_slots,
        live_slots_in_memory_account: live_slots,
        available_bytes: available_slots * i128::from(SLOT_BYTES),
        common_commitments_bytes: 0,
        ring_window_slots: 0,
        unblocked_slots: 0,
        overlap_slots_allocated_and_isolated: 0,
        isolated_slots: 0,
        allocated_slots: live_slots,
        deferred_slots: 0,
        closed_free_slots_in_cluster_segments: 0,
    }
}

fn state_reading(
    devices: Vec<DeviceReading>,
    data_units: u64,
    cost: u64,
    lag_slots: u64,
) -> StateReading {
    StateReading {
        devices,
        data_units,
        checkpoint_cost_blocks: cost,
        lag_slots,
        lag_unattributed_slots: 0,
        oldest_valid_root_txg: 0,
        fourth_newest_nonempty_valid_root_txg: None,
        cluster_segment_count: 1,
        inode_tree_internal_levels: 0,
        records_agree_with_the_disk: true,
        decomposition_holds: true,
        available_is_whole_slots: true,
    }
}

fn arm_bytes(reading: &StateReading, arm: CandidateArm) -> i128 {
    reading.raw_bytes_of(Arm::Candidate(arm))
}

#[test]
fn the_df_literal_arms_subtract_the_reserve_the_residue_and_the_lag_and_differ_by_the_live_metadata(
) {
    // 两盘各容量 1000 槽、活 100 槽，第一个文件 10 个数据单元（每盘 20 槽），c_max = 2，滞后 6。
    let reading = state_reading(
        vec![
            device_reading(1000, 100, 700),
            device_reading(1000, 100, 700),
        ],
        10,
        2,
        6,
    );
    // 没用过的加已释放的 = (900 + 900) ÷ 2 = 900；保留池 10 + 7 × 2 = 24；残留 5 + 6 × 2 = 17。
    let slot_bytes = i128::from(SLOT_BYTES);
    assert_eq!(
        arm_bytes(&reading, CandidateArm::DfLiteralWithoutLiveMetadata),
        slot_bytes * (900 - 24 - 17 - 6)
    );
    // 活元数据 = 每盘 (100 − 20) 槽，两盘之和 ÷ 2 = 80。
    assert_eq!(
        arm_bytes(&reading, CandidateArm::DfLiteralWithAllLiveMetadata),
        slot_bytes * (900 + 80 - 24 - 17 - 6)
    );
}

/// 按需收紧看的「各臂要的单元数」与判定用的是同一个门槛：下界恰等于它判一定不多报，少一个单元判未定（登记第十二节修订 4）。
#[test]
fn the_units_asked_for_by_every_arm_are_the_threshold_the_judgement_uses() {
    let reading = state_reading(
        vec![
            device_reading(1000, 100, 700),
            device_reading(1000, 100, 700),
        ],
        10,
        2,
        6,
    );
    let asks = reading.units_asked_for_by_every_arm();
    assert_eq!(asks.len(), all_arms().len());
    // 钉绝对值：W2 按槽字节 700 × 16384 = 11468800 字节 ⇒ ⌈÷ 32634⌉ = 352；W2 按载荷 350 个单元；W3 两盘 1400 ÷ 2 槽同 W2 按槽；阴性对照 0。
    assert_eq!(asks[2], 352);
    assert_eq!(asks[3], 350);
    assert_eq!(asks[4], 352);
    assert_eq!(asks[7], 0);
    for (arm, asked) in all_arms().into_iter().zip(&asks) {
        let reported = reported_bytes(reading.raw_bytes_of(arm));
        let censored_at = |lower_bound_units: u64| WritableUnits::AtLeastBecauseOfTheJournalCap {
            lower_bound_units,
            lower_bound_was_tightened: true,
        };
        assert_eq!(
            judge(reported, &censored_at(*asked)),
            Judgement::CertainlyDoesNotOverReportBelowTheLowerBound,
            "{}",
            arm.label()
        );
        if *asked > 0 {
            assert_eq!(
                judge(reported, &censored_at(asked - 1)),
                Judgement::Undetermined,
                "{}",
                arm.label()
            );
        }
    }
}

#[test]
fn the_available_arms_take_the_minimum_over_devices_not_the_maximum() {
    let reading = state_reading(
        vec![
            device_reading(1000, 100, 700),
            device_reading(800, 100, 500),
        ],
        10,
        2,
        0,
    );
    let slot_bytes = i128::from(SLOT_BYTES);
    let payload_bytes = i128::from(DATA_UNIT_PAYLOAD_BYTES);
    assert_eq!(
        arm_bytes(
            &reading,
            CandidateArm::AvailableMinimumOverDevicesInSlotBytes
        ),
        slot_bytes * 500
    );
    assert_eq!(
        arm_bytes(
            &reading,
            CandidateArm::AvailableMinimumOverDevicesInPayloadBytes
        ),
        payload_bytes * 250
    );
    // 两盘相加 ÷ 副本数 = (700 + 500) ÷ 2。
    assert_eq!(
        arm_bytes(
            &reading,
            CandidateArm::AvailableSummedOverDevicesDividedByReplicas
        ),
        slot_bytes * 600
    );
}

#[test]
fn the_payload_arm_counts_whole_units_of_the_registered_payload_size() {
    let reading = state_reading(
        vec![device_reading(1000, 0, 7), device_reading(1000, 0, 9)],
        0,
        0,
        0,
    );
    // 最小 7 槽 ⇒ 3 个整单元，每个 32634 字节。
    assert_eq!(
        arm_bytes(
            &reading,
            CandidateArm::AvailableMinimumOverDevicesInPayloadBytes
        ),
        3 * 32634
    );
}

#[test]
fn the_write_demand_arm_leaves_room_for_the_non_data_demand_of_the_write() {
    let reading = state_reading(
        vec![device_reading(1000, 0, 1000), device_reading(1000, 0, 1000)],
        0,
        10,
        0,
    );
    let units = arm_bytes(
        &reading,
        CandidateArm::AvailableMinimumLessTheNonDataDemandOfTheWrite,
    ) / i128::from(DATA_UNIT_PAYLOAD_BYTES);
    let units = u64::try_from(units).expect("单元数");
    assert!(units < 500, "比不扣非数据需求的 500 个单元少");
    assert_eq!(
        units,
        largest_units_satisfying_the_write_demand(1000, 0, 10)
    );
}

#[test]
fn the_always_over_reporting_control_reports_the_whole_free_pool_without_dividing_by_replicas() {
    let reading = state_reading(
        vec![device_reading(1000, 100, 0), device_reading(1000, 100, 0)],
        0,
        0,
        0,
    );
    assert_eq!(
        reading.raw_bytes_of(Arm::PositiveControlAlwaysOverReports),
        i128::from(SLOT_BYTES) * 1800
    );
    assert_eq!(reading.raw_bytes_of(Arm::NegativeControlReportsZero), 0);
}

#[test]
fn negative_arm_value_is_reported_as_zero_bytes() {
    assert_eq!(reported_bytes(-5), 0);
    assert_eq!(reported_bytes(7), 7);
}

#[test]
fn the_residuals_use_the_overlap_and_held_terms_as_registered() {
    // 可用 + Cm = 100，没挡的 = 90，扣住账 = 0，重叠 = 0 ⇒ 三个残差都是 10（扣住的槽没人记账）。
    let plain = residuals_of(100, 90, 0, 0);
    assert_eq!(
        plain.rewritten_check_residual_with_the_held_and_overlap_terms,
        10
    );
    assert_eq!(plain.original_formula_residual, 10);
    // 重叠 3 槽：原式残差照旧 10，改写检查残差的两项式 13，去掉重叠项的对照 10。
    let overlapping = residuals_of(100, 90, 3, 0);
    assert_eq!(
        overlapping.rewritten_check_residual_with_the_held_and_overlap_terms,
        13
    );
    assert_eq!(
        overlapping.rewritten_check_residual_without_the_overlap_term,
        10
    );
    assert_eq!(overlapping.original_formula_residual, 10);
    // 扣住账 10 槽、没有重叠：改写检查残差恰为 0，原式残差仍是 10。
    let held = residuals_of(100, 90, 0, 10);
    assert_eq!(
        held.rewritten_check_residual_with_the_held_and_overlap_terms,
        0
    );
    assert_eq!(held.original_formula_residual, 10);
}

fn record(
    device: u32,
    slot: u64,
    span_slots: u16,
    generation: u64,
    is_released: bool,
) -> AllocationRecord {
    AllocationRecord {
        device: DeviceIdentity(device),
        slot: SlotNumber(slot),
        span_slots,
        generation: singlefs_core::address::CheckpointTxg(generation),
        is_released,
        quarantine_mark: singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
    }
}

#[test]
fn the_lag_counts_releases_by_user_changes_above_the_fourth_newest_nonempty_root_only() {
    let mut log = PublishLog::default();
    log.user_change_txgs.extend([10, 11, 12, 13, 14, 15]);
    log.empty_publish_txgs.extend([16, 17]);
    // 环里有效根 10..=17；非空的 10..=15，从新到旧第 4 个是 12；最旧有效根 10；门槛 = 12。
    let valid_roots: Vec<u64> = (10..=17).collect();
    let records = vec![
        record(0, 100, 2, 11, true), // 释放代 11 ≤ 门槛：不滞后
        record(1, 100, 2, 11, true),
        record(0, 200, 2, 13, true), // 用户改动放的，高于门槛：滞后
        record(1, 200, 2, 13, true),
        record(0, 300, 1, 16, true), // 空发布放的：不算
        record(1, 300, 1, 16, true),
        record(0, 400, 2, 14, false), // 仍分配：不算
        record(0, 500, 4, 99, true),  // 释放代不在任何日志里：记未归属
    ];
    let reading = lag_reading_of(&records, &valid_roots, &log, 2);
    assert_eq!(reading.fourth_newest_nonempty_valid_root_txg, Some(12));
    assert_eq!(reading.oldest_valid_root_txg, 10);
    assert_eq!(reading.lag_slots, 2, "两盘各 2 槽，除以盘数");
    assert_eq!(reading.unattributed_slots, 4);
}

#[test]
fn the_lag_threshold_falls_back_to_the_oldest_valid_root_when_fewer_than_four_roots_are_nonempty() {
    let mut log = PublishLog::default();
    log.user_change_txgs.extend([5, 6]);
    let reading = lag_reading_of(&[record(0, 1, 2, 5, true)], &[3, 4, 5, 6], &log, 2);
    assert_eq!(reading.fourth_newest_nonempty_valid_root_txg, None);
    assert_eq!(reading.lag_slots, 1, "门槛取最旧有效根 3，释放代 5 高于它");
}

#[test]
fn the_closed_free_slots_count_only_the_segments_this_mount_opened() {
    let mut map = DeviceFreeMap::new(DeviceIdentity(0), FOUR_GIBIBYTES);
    let opened_by_this_mount: BTreeSet<SlotNumber> = [SlotNumber(50240)].into_iter().collect();
    assert_eq!(
        closed_free_slots_in_segments(&map, &opened_by_this_mount),
        64
    );
    map.mark_allocated(SlotNumber(50241), 2);
    assert_eq!(
        closed_free_slots_in_segments(&map, &opened_by_this_mount),
        62
    );
    // 另一段里有分配、但不是这次挂载开的：不算。
    map.mark_allocated(SlotNumber(50304), 2);
    assert_eq!(
        closed_free_slots_in_segments(&map, &opened_by_this_mount),
        62
    );
    let two_segments: BTreeSet<SlotNumber> =
        [SlotNumber(50240), SlotNumber(50304)].into_iter().collect();
    assert_eq!(closed_free_slots_in_segments(&map, &two_segments), 62 + 62);
}

#[test]
fn the_probe_result_is_classified_by_what_reached_the_device_and_why_it_was_refused() {
    let space_admission = PublishError::SpaceAdmissionRefused(
        singlefs_core::admission::AdmissionRefusedOnSomeDevices {
            short_devices: Vec::new(),
        },
    );
    let refused_for_placement = PublishError::PlacementRefused {
        unit: TransactionUnit::InodeRoot,
        refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
    };
    let publish_refused = |cause| UserChangeRefused::Publish {
        cause,
        floor_raises: Vec::new(),
        versions_published: Vec::new(),
    };
    assert_eq!(
        classify_probe_result(Err(publish_refused(space_admission)), false),
        ProbeVerdict::RefusedForSpace
    );
    assert_eq!(
        classify_probe_result(Err(publish_refused(refused_for_placement)), false),
        ProbeVerdict::RefusedForSpace
    );
    assert_eq!(
        classify_probe_result(
            Err(publish_refused(
                PublishError::JournalRecordsOfThePublishExceedTheLimit {
                    records_of_the_publish: 70_000,
                    record_limit: 65_536,
                }
            )),
            false
        ),
        ProbeVerdict::JournalLimit {
            records_of_the_publish: 70_000,
            record_limit: 65_536
        }
    );
    // 数据单元的写到达了设备：不论会话把它当成什么错，都是放行。
    let device_table_error = UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec::new(),
        device_table_handed_in: Vec::new(),
    };
    assert_eq!(
        classify_probe_result(Err(device_table_error), true),
        ProbeVerdict::Admitted
    );
    let device_table_error_without_a_data_write =
        UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount {
            device_table_of_the_mount: Vec::new(),
            device_table_handed_in: Vec::new(),
        };
    assert!(matches!(
        classify_probe_result(Err(device_table_error_without_a_data_write), false),
        ProbeVerdict::Other(_)
    ));
}
