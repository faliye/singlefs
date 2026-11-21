//! C 那一圈记着落点的槽：记着空串的槽不算，两槽 C 推到较晚的那一槽、两槽都放出来。

use super::*;

/// 记着空串的槽不算 C 的槽（正推 `emptyHeld`、`slotsWrittenEmpty`）：主形（建 inode、前缀 40 次）卡住之后，在现行那一版那一槽
/// （从现行那一版起最后轮到的一槽，满一环）记一个空串。再写一次照旧报「被宽护卡住」，R 那一槽与要推的次数都照旧是录下来的 R 那一槽；
/// 推环推到 R 那一槽为止，交回写掉的那几槽就是 C 那一圈原来那一槽。
pub fn run_the_scenario_where_a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        recorded,
        mut pool,
        stuck,
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
    let ring_slot_of_the_current_version = target_for_publish(
        pool.session().current.root().checkpoint_txg,
        pool.parameters.geometry.root_ring_slots_per_region,
    );
    {
        let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
        let mut holding = allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        assert_eq!(
            holding.keys().copied().collect::<Vec<RootRingSlot>>(),
            vec![stuck.last_ring_slot_the_push_reaches],
            "主形 C 那一圈只有 R 那一槽（阳性对照）"
        );
        holding.insert(ring_slot_of_the_current_version, Vec::new());
        allocator.note_placements_named_by_unapplied_journal_records(holding);
    }
    let refusal = pool
        .overwrite(OVERWRITE_BYTES)
        .expect_err("宽护照旧挡着：覆盖写被拒");
    let WriteStuckBehindTheWideProtection {
        stuck: stuck_again, ..
    } = stuck_behind_the_wide_protection_reported_by(refusal, "多记一个空串之后再写");
    assert_eq!(
        (
            stuck_again.last_ring_slot_the_push_reaches,
            stuck_again.empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written
        ),
        (
            target_for_publish(
                recorded.key.1,
                recorded.pool.parameters.geometry.root_ring_slots_per_region
            ),
            stuck.empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written
        ),
        "记着空串的那一槽什么都不罩：R 那一槽与要推的次数照旧"
    );
    let pushed = push_the_root_ring_of(&mut pool)
        .unwrap_or_else(|push_refusal| panic!("推环做得成：{push_refusal:?}"));
    assert_eq!(
        u64::try_from(pushed.versions_published.len()).expect("次数"),
        stuck.empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written,
        "推到 R 那一槽为止，不推到记着空串的那一槽"
    );
    assert_eq!(
        pushed.ring_slots_written,
        vec![stuck.last_ring_slot_the_push_reaches],
        "交回写掉的那几槽：C 那一圈原来那一槽（记着空串的那一槽不算）"
    );
}

/// 两槽 C（攻方 M12，照攻方 `research/prompts/m3-batch10z-pqrs-code-r1-opus-model/atk_part3.rs` 的 `atk_k2_two_c_slots_push` 造法：丙并中间那一形，
/// Q 顺序写、R 建 inode，挂载那一刻 Q 的根与末条记录、R 的账读回全零，C 那一圈有 Q、R 两槽）：挂上就推环，推的次数是到较晚轮到的 R 那一槽的距离，
/// 交回写掉的两槽，推完 C 那一圈一槽不剩、隔离回 0，池级 checker 不红 I-7.4、I-4.8。R 那一槽取最先轮到的那一槽时推环停在 Q 那一槽、R 还记着。
pub fn run_the_scenario_where_both_devices_lost_their_newest_rotation_write_and_pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both(
    images: &mut PoolCheckerImages,
) {
    let recorded = record_a_sequential_write_as_the_intermediate_publish_and_an_inode_creation_as_the_recorded_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_INTERMEDIATE_AND_THE_RECORDED_PUBLISH,
    );
    let crash_image = image_where_both_devices_lost_their_newest_rotation_write(
        &recorded
            .crash_image_with_the_root_of_the_recorded_publish_persisted_and_its_rotation_lost(),
    );
    let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
    let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut faulted_devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("Q 的根与末条记录、R 的账读回全零：可写挂载退到 P、做得成");
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&faulted_devices, recorded.pool.device_bytes);
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices: plain_devices_on(&image_after_the_mount),
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let slots_per_region = recorded.pool.parameters.geometry.root_ring_slots_per_region;
    let ring_slot_of_the_intermediate_publish =
        target_for_publish(recorded.key_of_the_intermediate_publish.1, slots_per_region);
    let last_ring_slot_the_push_reaches =
        target_for_publish(recorded.key_of_the_recorded_publish.1, slots_per_region);
    let holding_before_the_push: Vec<RootRingSlot> = pool
        .session()
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .ring_slots_holding_placements_named_by_unapplied_journal_records()
        .into_keys()
        .collect();
    let mut intermediate_and_later_publishes = vec![
        ring_slot_of_the_intermediate_publish,
        last_ring_slot_the_push_reaches,
    ];
    intermediate_and_later_publishes.sort();
    assert_eq!(
        holding_before_the_push, intermediate_and_later_publishes,
        "C 那一圈有 Q、R 两槽（阳性对照）"
    );
    let current_txg = pool.session().current.root().checkpoint_txg;
    let publishes_until_the_ring_slot_of = |ring_slot: RootRingSlot| -> usize {
        (1..=singlefs_format::ROOT_RING_REGIONS * slots_per_region.count())
            .position(|publishes_ahead| {
                target_for_publish(
                    CheckpointTxg(current_txg.0 + publishes_ahead),
                    slots_per_region,
                ) == ring_slot
            })
            .expect("一环之内每一槽都轮到一次")
            + 1
    };
    assert!(
        publishes_until_the_ring_slot_of(ring_slot_of_the_intermediate_publish)
            < publishes_until_the_ring_slot_of(last_ring_slot_the_push_reaches),
        "Q 那一槽比 R 那一槽先轮到（阳性对照：取最先轮到的与最后轮到的推的次数不同）"
    );
    let pushed = push_the_root_ring_of(&mut pool)
        .unwrap_or_else(|refusal| panic!("两槽 C 推环做得成：{refusal:?}"));
    assert_eq!(
        pushed.versions_published.len(),
        publishes_until_the_ring_slot_of(last_ring_slot_the_push_reaches),
        "推的次数是到较晚轮到的 R 那一槽的距离"
    );
    assert_eq!(
        pushed.ring_slots_written, intermediate_and_later_publishes,
        "交回写掉的那几槽：Q、R 两槽"
    );
    let allocator = &pool.session().allocator;
    assert!(
        allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .is_empty(),
        "推完 C 那一圈一槽不剩"
    );
    assert_eq!(
        isolated_slots_of(allocator),
        0,
        "推完隔离回 0（这一形根环表里没有被抛弃根）"
    );
    images.hand_out(
        "推完之后 I-7.4、I-4.8 不红",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}
