//! 崩了再挂抛弃的根 C 的账或根槽读不出：卸载与会话写照根环表里记着的落点抬 F、不拒；公开的抬 F 上限同口径；有效根的根槽读坏照拒，崩了再可写挂载照拒。

use super::*;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements,
    );
}

/// 「崩了再可写挂载」那一道照 C694 的 B 拒，不变：同一份盘面上 C 的账读不出、现行账里没有罩住它树表槽的凭据，可写挂载在取号之前拒、
/// 点名 C，两块盘逐字节不变。
#[test]
fn writable_mount_after_a_crash_still_refuses_while_the_account_of_an_abandoned_root_is_unreadable_without_the_credential(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
    fault.is_active.set(true);
    let mut devices = devices_with_the_read_fault_on(&built.image, &fault);
    let refused = mount_writable_with_space_admission(
        &built.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    );
    let expected_abandoned_root = RollbackTarget {
        instance: built.abandoned_root.instance,
        checkpoint_txg: built.abandoned_root.checkpoint_txg,
    };
    match refused {
        Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) => {
            assert!(
                matches!(
                    *still_unreadable,
                    StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { abandoned_root }
                        if abandoned_root == expected_abandoned_root
                ),
                "拒因点名 C 的账：{still_unreadable:?}"
            );
        }
        Err(other) => panic!("拒因是 C 的账读不出，而是：{other:?}"),
        Ok(_) => panic!("C 的账读不出、没有凭据：崩了再可写挂载照拒，而挂上了"),
    }
    let image_after_the_refusal = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.sparse_image().clone()))
            .collect(),
        device_size_in_bytes: built.device_bytes,
    };
    assert!(
        image_after_the_refusal == built.image,
        "在取号与任何写之前拒：两块盘逐字节不变"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements,
    );
}

/// 公开的抬 F 上限（`rollback_floor_ceiling`），读坏、重读仍坏就拒的槽取 `ring_slots`：系统配置、现行那一版的实例表与 F 都照这一刻的盘与会话现读。
fn public_rollback_floor_ceiling_of(
    pool: &PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
    ring_slots: &std::collections::BTreeSet<RootRingSlot>,
) -> Result<singlefs_core::address::CheckpointTxg, MountError> {
    let system_configuration = choose_system_configuration(&pool.devices).expect("系统配置读得出");
    let table = instance_table_chain_of_root(&pool.devices, &pool.current_file_version().root)
        .expect("现行那一版的实例表读得出")
        .records;
    rollback_floor_ceiling(
        &pool.devices,
        &system_configuration,
        pool.rollback_floor(),
        &table,
        ring_slots,
    )
}

/// K1-1（正推 `research/prompts/m3-batch10z-d-code-r1-sonnet-output.md` K1-1 末段公开上限函数那一形）：挂上之后 C 的根槽读报错。自算上限的调用方
/// （`raise-rollback-floor` 模式、e158 装置）照抬 F 那一串的口径取读坏重读仍坏就拒的槽
/// （`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`：除去根环表记成被抛弃根的槽），公开的 `rollback_floor_ceiling` 不拒，
/// C 那一槽当没有根、不进上限：算出的上限与故障关着时相同。阳性对照：同一刻照旧口径传「知道住着根的槽」整集就拒、点名 C 那一槽
/// （`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）。
#[test]
fn the_public_rollback_floor_ceiling_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_equals_the_ceiling_while_it_reads(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (pool, _recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    let ring_slots_whose_reread_still_bad_refuses_the_ceiling =
        ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(&pool.session().allocator);
    let ceiling_while_the_slot_reads = public_rollback_floor_ceiling_of(
        &pool,
        &ring_slots_whose_reread_still_bad_refuses_the_ceiling,
    )
    .expect("故障关着：上限算得出");
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
    let ring_slot_of_the_abandoned_root = target_for_publish(
        built.abandoned_root.checkpoint_txg,
        built.parameters.geometry.root_ring_slots_per_region,
    );
    match public_rollback_floor_ceiling_of(
        &pool,
        &ring_slots_known_to_hold_a_root_by(&pool.session().allocator),
    ) {
        Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
            ring_slot,
            ..
        }) => assert_eq!(
            ring_slot, ring_slot_of_the_abandoned_root,
            "旧口径（知道住着根的槽整集）拒、点名 C 那一槽"
        ),
        other => panic!("阳性对照：旧口径下 C 的根槽读坏重读仍坏就拒，而是：{other:?}"),
    }
    match public_rollback_floor_ceiling_of(
        &pool,
        &ring_slots_whose_reread_still_bad_refuses_the_ceiling,
    ) {
        Ok(ceiling) => assert_eq!(
            ceiling, ceiling_while_the_slot_reads,
            "被抛弃根的槽读坏当没有根、不进上限：上限与故障关着时相同"
        ),
        Err(refusal) => {
            panic!("被抛弃根的槽读坏不拒（与抬 F 那一串同一口径），而拒了：{refusal:?}")
        }
    }
}

/// H1 的另一半：有效根的槽照「这个进程知道住着根的槽」那一句，读坏重读仍坏就拒。挂上之后这次挂载择到的那条有效根（实例 3 退出时现行那一版，
/// 根环表记成有效根）的根槽读报错：正常卸载拒成 `NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`、点名那一槽，
/// 在任何写之前，两块盘逐字节不变。
#[test]
fn unmount_after_the_root_ring_slot_of_a_valid_root_turns_unreadable_is_still_refused_before_any_write(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault = ReadFaultOnChosenSlots::on_the_root_ring_slot_of(
        &built.newest_valid_root,
        &built.parameters,
    );
    let (mut pool, _recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.newest_valid_root);
    let expected_ring_slot: RootRingSlot = target_for_publish(
        built.newest_valid_root.checkpoint_txg,
        built.parameters.geometry.root_ring_slots_per_region,
    );
    let image_before_the_unmount = pool.image();
    let mut session = pool.session.take().expect("挂着的会话");
    let refused = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match refused {
        Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) => {
            assert!(
                matches!(
                    *still_unreadable,
                    StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { ring_slot, .. }
                        if ring_slot == expected_ring_slot
                ),
                "拒因点名那条有效根的根槽 {expected_ring_slot:?}：{still_unreadable:?}"
            );
        }
        Err(other) => panic!("拒因是有效根的根槽重读仍坏，而是：{other:?}"),
        Ok(_) => panic!("有效根的根槽读坏、重读仍坏：卸载照拒，而做成了"),
    }
    assert!(
        pool.image() == image_before_the_unmount,
        "在任何写之前拒：两块盘逐字节不变"
    );
}
