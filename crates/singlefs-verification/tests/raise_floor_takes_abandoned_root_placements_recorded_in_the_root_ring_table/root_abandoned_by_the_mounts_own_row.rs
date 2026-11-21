//! 可写挂载自己写行之后才判成被抛弃的根 R：照 C 那一圈在 R 那一槽记着的落点抬 F，挂载那一刻凭据放行的被抛弃根记着空串，R 共用、继承的单元留到它那一槽写掉。

use super::*;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot,
    );
}

/// 凭据放行那一形里，挂载之后至多再覆盖写几次去等 R 的树表槽与映射根槽都被重新分配（384 槽小盘一圈根环 24 次发布之内）。
const OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST: u64 = 24;

/// 现行账里有一条分配代大于 `root` 的 txg、罩住这一槽的记录（C694 的 B 那道凭据的一条位置条目）。
fn a_later_record_covers(
    records: &[AllocationRecord],
    root: &RootRecord,
    location: &LocationEntry,
) -> bool {
    records.iter().any(|record| {
        record.device == location.device
            && record.slot.0 <= location.slot.0
            && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
            && record.generation > root.checkpoint_txg
    })
}

/// 攻方 M7（`research/prompts/m3-batch10z-b-code-r1-opus-output.md` 第三节）：挂载那一刻凭据放行的被抛弃根，根环表里记着的是空串，
/// 抬 F 那一串照空串算、不再读它的账。造法同 `abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs`
/// 「已被占的那一形」：M1 里 R 的根槽与单元读回全零、影子账关着，之后覆盖写到 R 的树表槽与映射根槽都被重新分配；再可写挂载一次（M2）：
/// R 的根槽读得出、按实例表被抛弃，账与映射都读不出、现行账里有凭据——放行，根环表把 R 记成被抛弃根、记着空串，挂载计数 1。
/// M2 正常卸载：照记着的空串算，不读 R 的账（计数 0），做成。照读盘时这一遍又数到 R（计数 1）。
#[test]
fn unmount_takes_the_empty_placements_recorded_for_an_abandoned_root_the_credential_passed_at_mount_without_reading_its_account(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let root_of_the_recorded_publish = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let mut devices_of_the_first_mount = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let first_mount = mount_writable_with_test_only_switches(
        &recorded.pool.parameters,
        &mut devices_of_the_first_mount,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::Off,
    )
    .expect("R 的根槽与单元读回全零：可写挂载落到 P、做得成");
    let image_after_the_first_mount = image_of_devices_reading_back_all_zero(
        &devices_of_the_first_mount,
        recorded.pool.device_bytes,
    );
    let mut pool = pool_after_the_mount(&recorded, &image_after_the_first_mount, first_mount);
    let account_slots_of_the_recorded_publish: Vec<LocationEntry> = root_of_the_recorded_publish
        .tree_table
        .locations
        .iter()
        .chain(root_of_the_recorded_publish.mapping_root.locations.iter())
        .copied()
        .collect();
    let mut overwrites_after_the_first_mount = 0;
    // 迭代上界 OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST；每一轮看一次现行账，不够再覆盖写一次。
    while !account_slots_of_the_recorded_publish
        .iter()
        .all(|location| {
            a_later_record_covers(
                &pool.current_file_version().allocation_records,
                &root_of_the_recorded_publish,
                location,
            )
        })
    {
        assert!(
            overwrites_after_the_first_mount < OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST,
            "{OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST} 次覆盖写之内 R 的树表槽与映射根槽都被重新分配"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上这一串覆盖写都落得下");
        overwrites_after_the_first_mount += 1;
    }
    let image = pool.image();
    let mut devices_of_the_second_mount = plain_devices_on(&image);
    let second_mount = mount_writable(&recorded.pool.parameters, &mut devices_of_the_second_mount)
        .expect("R 的账读不出、现行账里有凭据：再可写挂载做得成");
    assert_eq!(
        second_mount.output.abandoned_roots_unreadable, 1,
        "再可写挂载那一遍读不出 R 的账、凭据放行，计数往上报"
    );
    assert_eq!(
        second_mount
            .allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .placements_recorded_for_the_abandoned_root(
                root_of_the_recorded_publish.instance,
                root_of_the_recorded_publish.checkpoint_txg
            ),
        Some(Vec::new()),
        "凭据放行：根环表把 R 记成被抛弃根，记着的落点是空串"
    );
    let (_, mut session) = MountedSession::of_the_mount(second_mount);
    let PoolVersion::WithFile(_) = &session.current else {
        panic!("覆盖写之后现行那一版带文件");
    };
    let unmounted = unmount(
        &recorded.pool.parameters,
        &mut devices_of_the_second_mount,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => assert_eq!(
            raised.abandoned_roots_unreadable, 0,
            "R 记着的是空串：照它算，这一遍不再读 R 的账"
        ),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("R 记着空串：卸载做得成，而被拒：{refusal:?}"),
    }
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_both_devices_lost_their_newest_rotation_write_and_units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
// harness-test-granularity:one-scenario 一个池、一次挂载，循环是同一个会话里接着覆盖写，不是每轮新建一个池
#[test]
fn units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written_where_both_devices_lost_their_newest_rotation_write(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_both_devices_lost_their_newest_rotation_write_and_units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written,
    );
}
