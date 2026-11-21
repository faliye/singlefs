//! 回退复活的单元在 R 的各段读回全零时照样留着；挂载那一刻认得出回退时只多回退目标账里仍分配的那一截，认不出、不是回退时一槽不多。

use super::*;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount,
    );
}

/// 认得出回退时只多那一截（正推 K5 的 E3、E1，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第二节 K5 那一行）：
/// 回退复活那一形、只有 R 的账读回全零（谱系片与 R_old 都读得出）。挂载那一刻 C 里 R 那一槽里、在挂载那一刻现行账里记成已释放的落点，
/// 逐个就是 R_old 账里仍分配着、现行账里已释放的那一截（按盘与起点槽），一个不多、一个不少。认出回退就并现行账里全部已释放的（E3，
/// 正推的对照臂）、回退目标按所有有效根找（E1）都会多出别的已释放落点。期望从录下来的会话算（R_old 那一刻的会话分配器），不经被测的方法。
#[test]
fn a_rollback_recognized_at_the_mount_takes_among_the_released_only_what_its_target_still_allocated(
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    let recorded = &recorded_rollback.recorded;
    let root_of_the_recorded_publish = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(recorded)),
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
    let borrowed_for_the_recorded_publish = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    let released_borrowed_for_the_recorded_publish: std::collections::BTreeSet<(
        DeviceIdentity,
        SlotNumber,
    )> = borrowed_for_the_recorded_publish
        .iter()
        .map(|placement| (placement.device, placement.slot))
        .filter(|placement| released_in_the_current_account.contains(placement))
        .collect();
    let still_allocated_by_the_target_and_released: std::collections::BTreeSet<(
        DeviceIdentity,
        SlotNumber,
    )> = recorded_rollback
        .allocated_in_the_account_of_the_rollback_target
        .iter()
        .copied()
        .filter(|placement| released_in_the_current_account.contains(placement))
        .collect();
    assert!(
        recorded_rollback.revived.iter().all(|revived| {
            still_allocated_by_the_target_and_released.contains(&(revived.device, revived.slot))
        }),
        "复活的落点都在 R_old 账里仍分配、现行账里已释放的那一截里（阳性对照：期望那一截罩着复活集）"
    );
    assert_eq!(
        released_borrowed_for_the_recorded_publish, still_allocated_by_the_target_and_released,
        "C 里 R 那一槽里现行账已释放的落点，就是 R_old 账里仍分配、现行账里已释放的那一截（窄取法：不多并现行账里别的已释放落点）"
    );
}

/// tri 不碰的那一支（正推 K2-3 第 1 条「读得出、确实没有这条根的谱系记录（肯定不是回退）」）：R 建一个 inode、挂载那一刻只有 R 的账读回全零，
/// R 的记录点名的打包单元（inode 叶容器）都读得出、里面没有谱系记录。C 里 R 那一槽里一个现行账已释放的落点都没有（丙只并仍分配着的），
/// 挂载那一刻两块盘隔离的槽数与 tri 之前逐格相同：18（R 写的单元，两块盘各 9 槽；正推对照臂这一形 116，
/// 量法副本 `SINGLEFS_PROBE_ARM=narrow`（tri 之前）与 tri 两臂挂载那一刻同为 18）。
#[test]
fn a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount(
) {
    let recorded = record_one_inode_creation_after_overwrites(
        OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    let root_of_the_recorded_publish = recorded.root();
    assert!(
        lineage_unit_writes_of(&recorded).is_empty(),
        "建 inode 不写谱系片（阳性对照：R 不是回退）"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded)),
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
    assert!(
        !released_in_the_current_account.is_empty(),
        "挂载那一刻现行账里有已释放的落点（阳性对照：并多了看得出来）"
    );
    let released_borrowed_for_the_recorded_publish: Vec<PlacementOnDevice> = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）")
        .into_iter()
        .filter(|placement| {
            released_in_the_current_account.contains(&(placement.device, placement.slot))
        })
        .collect();
    assert!(
        released_borrowed_for_the_recorded_publish.is_empty(),
        "R 不是回退：C 里 R 那一槽一个现行账里已释放的落点都不并，多并的 {released_borrowed_for_the_recorded_publish:?}"
    );
    let isolated_slots_at_the_mount: u64 = mounted
        .allocator
        .devices
        .iter()
        .map(singlefs_core::allocator::DeviceFreeMap::isolated_slots)
        .sum();
    assert_eq!(
        isolated_slots_at_the_mount,
        ISOLATED_SLOTS_AT_THE_MOUNT_WHEN_A_ROOT_TAKING_THE_CURRENT_ACCOUNT_IS_NOT_A_ROLLBACK,
        "挂载那一刻两块盘隔离的槽数与 tri 之前相同（D28（挂载期承诺量） 已定项 1 第九项）"
    );
}

/// [`a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount`] 那一形挂载那一刻两块盘隔离的槽数（tri 之前量的）。
const ISOLATED_SLOTS_AT_THE_MOUNT_WHEN_A_ROOT_TAKING_THE_CURRENT_ACCOUNT_IS_NOT_A_ROLLBACK: u64 =
    18;
