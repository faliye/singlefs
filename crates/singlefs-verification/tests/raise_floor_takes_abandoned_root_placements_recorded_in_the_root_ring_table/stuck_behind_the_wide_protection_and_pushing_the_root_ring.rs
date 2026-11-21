//! 被宽护卡住的几形与推环：报卡住、推环之后写放得下、卸载重挂；推环取不到落点、一槽都不用推、在没有文件的那一版上推时的拒法。

use super::*;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_the_recorded_publish_and_lets_writes_through`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_the_recorded_publish_and_lets_writes_through(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_the_recorded_publish_and_lets_writes_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way,
    );
}

/// 推满仍不够、F 已在上限而没有 C 那一圈（崩在静止点之后再挂，水位之上没有没施加的记录）：照旧只报空间不够、不报「被宽护卡住」；
/// 这时调推环入口也不推（C 那一圈没有记着落点的槽），两块盘逐字节不变。池照 `admission_raises_the_floor_before_refusing.rs` 里
/// 「截断之后同样大小写回」那一条起（单元区 256 槽两块盘，崩了再挂、覆盖写 3 次、顺序写把文件长到会话报空间不够）。
fn pool_stuck_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written() -> (
    PoolUnderTest<SparseBlockDevice>,
    NoSpaceAfterRaisingTheFloor,
) {
    let device_width = singlefs_verification::history::HistoryDeviceWidth::UnitAreaOf256Slots;
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        device_width.parameters(),
        device_width.device_bytes(),
        plain_device,
    );
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    for overwrite_number in 1..=3 {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_number} 次覆盖写：{refusal:?}"));
    }
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let mut units = 0;
    // 迭代上界 100 个单元；跨轮只带长到的单元数；出口：被拒。
    let refusal = loop {
        match pool.sequential_write((units + 1) * unit_bytes) {
            Ok(_) => units += 1,
            Err(refusal) => break refusal,
        }
        assert!(units < 100, "256 槽的盘长不到 100 个单元");
    };
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("长大被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
        ),
        "推抬 F 停在上限（阳性对照：与被宽护卡住同一种停法）：{:?}",
        no_space.stop
    );
    assert!(
        pool.session()
            .allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .is_empty(),
        "崩在静止点之后再挂：C 那一圈一槽都没有"
    );
    (pool, *no_space)
}

#[test]
fn no_space_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written_is_not_reported_as_the_wide_protection(
) {
    let (_pool, no_space) =
        pool_stuck_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written();
    assert!(
        matches!(
            no_space.wide_protection_at_the_ceiling,
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
        ),
        "没有 C 那一圈：照旧只报空间不够，不报被宽护卡住：{:?}",
        no_space.wide_protection_at_the_ceiling
    );
}

#[test]
fn pushing_the_root_ring_with_nothing_held_until_a_ring_slot_is_written_writes_nothing() {
    let (mut pool, _no_space) =
        pool_stuck_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written();
    let image_before = pool.image();
    let mut session = pool.session.take().expect("挂着的会话");
    let pushed = session
        .push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states(
            &mut pool.devices,
        );
    pool.session = Some(session);
    assert!(
        matches!(
            pushed,
            Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot)
        ),
        "C 那一圈没有记着落点的槽：不推，而交回 {pushed:?}"
    );
    assert!(pool.image() == image_before, "不推：两块盘逐字节不变");
}

/// 现行那一版树表 0 条（mkfs 之后可写挂载、还没写文件）：推环入口在任何读写之前拒成第一版不支持的那一种，两块盘逐字节不变。
#[test]
fn pushing_the_root_ring_on_a_version_without_file_is_refused_as_not_supported_in_the_first_version(
) {
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    singlefs_core::make_filesystem::make_filesystem(&parameters, &mut devices)
        .expect("小盘上 mkfs 做得成");
    let mounted = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("mkfs 之后可写挂载做得成");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    assert!(
        matches!(session.current, PoolVersion::WithoutFile(_)),
        "还没写文件：现行那一版树表 0 条（阳性对照）"
    );
    let image_of = |devices_of_the_pool: &[(DeviceIdentity, SparseBlockDevice)]| MemoryPool {
        devices: devices_of_the_pool
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    };
    let image_before = image_of(&devices);
    let pushed = session
        .push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states(
            &mut devices,
        );
    assert!(
        matches!(
            pushed,
            Err(RootRingPushRefused::VersionWithoutFileNotSupportedInTheFirstVersion)
        ),
        "树表 0 条：拒成第一版不支持，而交回 {pushed:?}"
    );
    assert!(image_of(&devices) == image_before, "两块盘逐字节不变");
}
