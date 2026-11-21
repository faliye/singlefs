//! C 那一圈记着落点的槽：记着空串的不算，两槽 C 推到较晚的那一槽，推环那条出路报的放出槽数，预算用完或抬 F 取不到落点时不报卡住，两槽落点里较晚的那一槽补隔离。

use super::*;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_both_devices_lost_their_newest_rotation_write_and_pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both_where_both_devices_lost_their_newest_rotation_write(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_both_devices_lost_their_newest_rotation_write_and_pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both,
    );
}

/// 每块盘上挑几个这一刻空闲的槽、置隔离位，记成 `ring_slot` 那一槽 C 记着的落点（一槽跨度一个），并进 C 那一圈原来记着的那几槽。
/// 注入级：可写挂载自己造不出这一份，用例拿它摆出 C 那一圈多一槽的形。
fn hold_isolated_free_slots_on_a_further_ring_slot(
    allocator: &mut PoolAllocator,
    ring_slot: RootRingSlot,
    free_slots_on_each_device: u64,
) {
    let placements = isolate_free_slots(allocator, free_slots_on_each_device);
    let mut holding = allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .ring_slots_holding_placements_named_by_unapplied_journal_records();
    assert!(
        !holding.contains_key(&ring_slot),
        "注入的那一槽原来没记着落点（阳性对照）"
    );
    holding.insert(ring_slot, placements);
    allocator.note_placements_named_by_unapplied_journal_records(holding);
}

/// 两槽 C、落点重叠时出路二说放出来的隔离槽数（「K1-1 三个量」正推注入级那一形：重叠时放出来的数偏大）：攻方 K1 那一形挂上之后
/// 长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限），在 R 那一槽之前先轮到的一槽（现行那一版之后第一次发布写的那一槽）注入 C 的第二槽，记着 R 那一槽一半的落点（与 R 那一槽重叠）
/// 加每块盘几个隔离着的空闲槽；另在每块盘隔离一个 C 那一圈不罩的空闲槽。再写同一个长度报「被宽护卡住」、推完放得下：
/// 报的挡着的槽数只数 C 罩着的（攻方 M7：取全部隔离时多数那几个），出路二说放出来的槽数等于它（重叠的不重数、C 不罩的不算），
/// 照推环走完就是推完前后每块盘隔离数之差，同一个写放得下。
#[test]
fn the_slots_the_way_out_by_the_push_releases_with_two_ring_slots_holding_overlapping_placements_are_what_the_push_releases(
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    );
    let slots_per_region = pool.parameters.geometry.root_ring_slots_per_region;
    let ring_slot_written_first = target_for_publish(
        CheckpointTxg(pool.session().current.root().checkpoint_txg.0 + 1),
        slots_per_region,
    );
    {
        let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
        let holding = allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        let (last_ring_slot_the_push_reaches, placements_of_the_recorded_publish) =
            holding.iter().next().expect("C 那一圈只有 R 那一槽");
        assert_ne!(
            *last_ring_slot_the_push_reaches, ring_slot_written_first,
            "R 那一槽不是推环第一次就写的那一槽（阳性对照：注入的那一槽先轮到）"
        );
        let overlapping_with_the_recorded_publish: Vec<PlacementOnDevice> =
            placements_of_the_recorded_publish
                .iter()
                .step_by(2)
                .copied()
                .collect();
        hold_isolated_free_slots_on_a_further_ring_slot(allocator, ring_slot_written_first, 2);
        let mut two_ring_slots = allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        two_ring_slots
            .get_mut(&ring_slot_written_first)
            .expect("刚注入的那一槽")
            .extend(overlapping_with_the_recorded_publish);
        allocator.note_placements_named_by_unapplied_journal_records(two_ring_slots);
        isolate_free_slots(allocator, 1);
    }
    assert!(
        isolated_slots_covered_by_the_ring_slots_holding_placements(&pool.session().allocator)
            < isolated_slots_of(&pool.session().allocator),
        "每块盘另有一个隔离着、C 那一圈不罩的槽（阳性对照：推环放不出它，挡着的槽数与放出来的槽数都不算它）"
    );
    let length = crashed
        .bytes_of_the_file_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
    let refusal = pool.sequential_write(length).expect_err("两槽 C 挡着准入");
    let WriteStuckBehindTheWideProtection { stuck, .. } =
        stuck_behind_the_wide_protection_reported_by(refusal, "两槽 C、落点重叠");
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, "两槽 C、落点重叠");
    let released_in_the_rehearsal =
        released_by_the_push_if_the_change_is_admitted_after_it(&pushing_the_root_ring)
            .unwrap_or_else(|| {
                panic!(
                    "两槽 C：预演推完之后这次改动放得下，而预演的结局是 {pushing_the_root_ring:?}"
                )
            });
    assert_eq!(
        released_in_the_rehearsal, stuck.isolated_slots_on_each_device,
        "根环表里没有被抛弃根：推环放出来的就是 C 那一圈罩着的、隔离着的槽，两槽重叠的那几个只数一次"
    );
    assert_eq!(ways_out.len(), 2, "推完放得下：两条出路都列：{ways_out:?}");
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_the_recorded_publish_and_the_released_slots(
        &stuck,
        &ways_out[1],
        &released_in_the_rehearsal,
        WhatIsLeftAfterThePush::PublishTheChangeAgain,
        "两槽 C、落点重叠",
    );
    let isolated_before_the_push = isolated_slots_on_each_device_of(&pool.session().allocator);
    let pushed = push_the_root_ring_of(&mut pool)
        .unwrap_or_else(|push_refusal| panic!("两槽 C 推环做得成：{push_refusal:?}"));
    assert_eq!(
        pushed.ring_slots_written.len(),
        2,
        "推环写掉 C 那一圈的两槽：{:?}",
        pushed.ring_slots_written
    );
    assert_eq!(
        released_in_the_rehearsal,
        isolated_slots_released_between(
            &isolated_before_the_push,
            &isolated_slots_on_each_device_of(&pool.session().allocator),
        ),
        "出路二说放出来的隔离槽数就是推完前后每块盘隔离数之差"
    );
    pool.sequential_write(length)
        .unwrap_or_else(|refusal_after_the_push| {
            panic!("推完之后同一个写放得下：{refusal_after_the_push:?}")
        });
}

/// 不再推的原因是一次准入的发布数做满时不报「被宽护卡住」（正推 `budgetAsStuck`）：攻方 K1 那一形长 20 个单元被宽护挡着，推环之后再试，
/// 会话推两串抬 F 就碰到一次准入 8 次发布的预算（攻方 `k1-trace`）；在现行那一版那一槽（满一环才轮到）注入 C 一槽、每块盘隔离两个空闲槽，
/// 再试：照旧停在预算上，C 那一圈这一刻隔离着槽，报的照旧只是空间不够。
#[test]
fn no_space_when_the_publishes_of_one_admission_run_out_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection(
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted();
    let length = crashed.bytes_of_the_file_grown_by(
        DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY,
    );
    pool.sequential_write(length)
        .expect_err("长 20 个单元：宽护挡着准入");
    push_the_root_ring_of(&mut pool).unwrap_or_else(|refusal| panic!("推环做得成：{refusal:?}"));
    let ring_slot_of_the_current_version = target_for_publish(
        pool.session().current.root().checkpoint_txg,
        pool.parameters.geometry.root_ring_slots_per_region,
    );
    hold_isolated_free_slots_on_a_further_ring_slot(
        &mut pool.session.as_mut().expect("挂着的会话").allocator,
        ring_slot_of_the_current_version,
        2,
    );
    let refusal = pool
        .sequential_write(length)
        .expect_err("推完之后长 20 个单元照旧被拒");
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
        ),
        "不再推的原因是一次准入的发布数做满（阳性对照）：{:?}",
        no_space.stop
    );
    assert!(
        isolated_slots_covered_by_the_ring_slots_holding_placements(&pool.session().allocator) > 0,
        "这一刻 C 那一圈还隔离着槽（阳性对照：只差停因）"
    );
    assert!(
        matches!(
            no_space.wide_protection_at_the_ceiling,
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
        ),
        "停因是预算用完：不报被宽护卡住，而报 {:?}",
        no_space.wide_protection_at_the_ceiling
    );
}

/// 挂上之后先覆盖写几次：之后有 4 个以上不同的非空状态，准入上限高过 F，被拒时会话真去推抬 F。
const OVERWRITES_BEFORE_THE_FLOOR_RAISE_FINDS_NO_PLACEMENT: usize = 5;

/// 不再推的原因是抬 F 那一串自己也取不到落点时不报「被宽护卡住」（正推 `spaceAsStuck`）：攻方 K1 那一形挂上之后覆盖写几次，
/// 再把每块盘没挡的槽全隔离掉（注入级），覆盖写一次：准入拒、推抬 F，抬 F 那一串在预演里取不到落点；C 那一圈这一刻隔离着槽，
/// 报的照旧只是空间不够。
#[test]
fn no_space_when_the_floor_raise_finds_no_placement_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection(
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted();
    for overwrite_number in 1..=OVERWRITES_BEFORE_THE_FLOOR_RAISE_FINDS_NO_PLACEMENT {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("挂上之后第 {overwrite_number} 次覆盖写落得下：{refusal:?}")
        });
    }
    for device_map in &mut pool.session.as_mut().expect("挂着的会话").allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                device_map.isolate(SlotNumber(slot), 1);
            }
        }
        assert_eq!(
            device_map.unblocked_slots(),
            0,
            "盘 {} 上没挡的槽全隔离掉了（阳性对照）",
            device_map.device.0
        );
    }
    let refusal = pool
        .overwrite(OVERWRITE_BYTES)
        .expect_err("没挡的槽全隔离掉：覆盖写被拒");
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorRaiseRefusedForSpace { .. }
        ),
        "不再推的原因是抬 F 那一串自己也取不到落点（阳性对照）：{:?}",
        no_space.stop
    );
    assert!(
        isolated_slots_covered_by_the_ring_slots_holding_placements(&pool.session().allocator) > 0,
        "这一刻 C 那一圈还隔离着槽（阳性对照：只差停因）"
    );
    assert!(
        matches!(
            no_space.wide_protection_at_the_ceiling,
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
        ),
        "停因是抬 F 那一串取不到落点：不报被宽护卡住，而报 {:?}",
        no_space.wide_protection_at_the_ceiling
    );
}

/// 根槽读坏的被抛弃根那一圈照记着的整段跨度补隔离（正推 K4-1：`crates/mutations.tsv` 里 K3 loop-span1 那一行改了不红——
/// 挂载那一刻 C 记着的落点两槽都已隔离着，抬 F 那一圈补的那一遍什么都不改）：挂上之后，C 记着的一个跨两槽、两槽都隔离着的落点，
/// 把后一槽的隔离位清掉（注入级：后一槽分配位没置着、空闲），再让 C 的根槽读报错、正常卸载：抬 F 那一串照根环表里记着的落点补隔离，
/// 后一槽也隔离着。只隔起点那一槽时后一槽照旧空闲、发得出去。
#[test]
fn unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_isolates_the_later_slot_of_its_recorded_two_slot_placement_left_unisolated(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    let (device, later_slot) = {
        let allocator = &pool.session().allocator;
        recorded
            .iter()
            .filter(|placement| placement.span >= 2)
            .find_map(|placement| {
                let device_map = allocator
                    .devices
                    .iter()
                    .find(|device_map| device_map.device == placement.device)
                    .expect("池里有这块盘");
                (placement.slot.0..placement.slot.0 + placement.span)
                    .all(|slot| device_map.is_isolated(SlotNumber(slot)))
                    .then_some((
                        placement.device,
                        SlotNumber(placement.slot.0 + placement.span - 1),
                    ))
            })
            .expect("C 记着的落点里有一个跨两槽以上、每一槽都隔离着的（阳性对照）")
    };
    {
        let device_map = pool
            .session
            .as_mut()
            .expect("挂着的会话")
            .allocator
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == device)
            .expect("池里有这块盘");
        device_map.clear_isolation_of_slot(later_slot);
        assert!(
            device_map.is_free(later_slot),
            "后一槽清掉隔离位之后空闲：分配位没置着（阳性对照）"
        );
    }
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
    let (_, allocator_after_the_unmount) =
        unmount_expecting_the_floor_raised(&mut pool, "C 的根槽读不出");
    let device_map = allocator_after_the_unmount
        .devices
        .iter()
        .find(|device_map| device_map.device == device)
        .expect("池里有这块盘");
    assert!(
        device_map.is_isolated(later_slot),
        "抬 F 那一串照根环表里记着的落点补隔离、补满整段跨度：后一槽（盘 {}、槽 {}）隔离着",
        device.0,
        later_slot.0
    );
}
