//! 384 槽盘上被宽护卡住的几形：报「被宽护卡住」、推环之后写放得下、卸载重挂那一条出路、推到半路取不到落点时在任何写之前拒。

use super::*;

/// 384 槽盘卡死的几形（从 22 形里取的）：
/// R 是哪一种、录在几次覆盖写之后，挂载那一刻读回全零的是哪几段（挂上就撤）。
#[derive(Clone, Copy, Debug)]
pub enum FormStuckBehindTheWideProtectionOnThe384SlotDevices {
    /// 覆盖写 `overwrites_before` 次、回退到倒数第 7 版录 R；R 的账与 R 写的谱系片读回全零（探针 `b6`、`racct,rlin`）。
    RollbackSixVersionsBack { overwrites_before: u64 },
    /// 覆盖写 `overwrites_before` 次、建一个 inode 录 R；R 写的全部单元读回全零（探针 `n`、`rall`）。
    InodeCreation { overwrites_before: u64 },
    /// 覆盖写 `overwrites_before` 次、建一个 inode、删掉它，录删除的第一次发布得 R；R 的账与 R 写的墓碑片读回全零（探针 `+n` `d`、`racct,rtomb`）。
    FirstPublishOfADeletion { overwrites_before: u64 },
}

/// 录下 `form` 的 R，交回录下来的 R 与挂载那一刻读回全零的那几段。
pub fn recorded_publish_and_ranges_reading_back_zero_of(
    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
) -> (RecordedPublish, RangesReadingBackAllZero) {
    match form {
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before,
        } => {
            let recorded = record_one_rollback_after_overwrites(
                overwrites_before,
                VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
            )
            .recorded;
            let lineage_unit_writes = lineage_unit_writes_of(&recorded);
            assert!(
                !lineage_unit_writes.is_empty(),
                "回退那次发布写了谱系片（阳性对照：故障有处可打）"
            );
            let ranges = RangesReadingBackAllZero::default()
                .with_the_writes(&account_writes_of(&recorded))
                .with_the_writes(&lineage_unit_writes);
            (recorded, ranges)
        }
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before,
        } => {
            let recorded = record_one_inode_creation_after_overwrites(overwrites_before);
            let ranges =
                RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
            (recorded, ranges)
        }
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
            overwrites_before,
        } => {
            let recorded =
                record_the_first_publish_of_one_deletion_after_overwrites(overwrites_before);
            let tombstone_unit_writes =
                packed_unit_writes_of_type(&recorded, singlefs_core::unit::PACKED_TYPE_TOMBSTONE);
            assert!(
                !tombstone_unit_writes.is_empty(),
                "删除的第一次发布写了墓碑片（阳性对照：故障有处可打）"
            );
            let ranges = RangesReadingBackAllZero::default()
                .with_the_writes(&account_writes_of(&recorded))
                .with_the_writes(&tombstone_unit_writes);
            (recorded, ranges)
        }
    }
}

/// 卡住的那一刻：挂着的池（读回全零已撤）、录下来的 R、准入第一次被拒时报的「被宽护卡住」。
pub struct PoolStuckBehindTheWideProtection {
    pub recorded: RecordedPublish,
    pub pool: PoolUnderTest<SparseBlockDevice>,
    pub stuck: StuckBehindTheWideProtection,
}

/// 录 `form` 的 R、崩在 R 的根槽落盘之后轮换之前，可写挂载那一刻那几段读回全零（挂上就撤），择根退到 P；挂上之后覆盖写一次：
/// 准入被拒、推抬 F 停在上限（`FloorAlreadyAtTheCeiling`），报「被宽护卡住」。挂上之后与被拒之后池级 checker 都不红 I-7.4、I-4.8。
pub fn pool_stuck_behind_the_wide_protection(
    images: &mut PoolCheckerImages,
    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
) -> PoolStuckBehindTheWideProtection {
    let (recorded, ranges) = recorded_publish_and_ranges_reading_back_zero_of(form);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .unwrap_or_else(|refusal| panic!("{form:?}：R 走读不成，可写挂载落到 P、做得成：{refusal:?}"));
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "{form:?}：择根退到 P（阳性对照：故障打到了）"
    );
    lift.lift();
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
    images.hand_out(
        &format!("{form:?}：挂上之后 I-7.4、I-4.8 不红"),
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
    let Err(refusal) = pool.overwrite(OVERWRITE_BYTES) else {
        panic!("{form:?}：挂上之后第一次覆盖写被宽护卡住（第 10z 批第十四块报告第六节：去掉双扣之后照旧全被拒），而它做成了");
    };
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("{form:?}：被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
        ),
        "{form:?}：推抬 F 停在上限（不再推的原因是 F 已在上限）：{:?}",
        no_space.stop
    );
    let NoSpaceAfterRaisingTheFloor {
        wide_protection_at_the_ceiling,
        ..
    } = *no_space;
    let WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) =
        wide_protection_at_the_ceiling
    else {
        panic!("{form:?}：F 在上限、C 那一圈还隔离着槽：报「被宽护卡住」，而报的是 {wide_protection_at_the_ceiling:?}");
    };
    images.hand_out(
        &format!("{form:?}：被拒之后 I-7.4、I-4.8 不红"),
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
    PoolStuckBehindTheWideProtection {
        recorded,
        pool,
        stuck: *stuck,
    }
}

/// 推环之后接着写几次覆盖写、几次顺序写（每次两个数据单元）。
pub const OVERWRITES_AFTER_PUSHING_THE_ROOT_RING: usize = 5;

pub const SEQUENTIAL_WRITES_AFTER_PUSHING_THE_ROOT_RING: usize = 2;

/// 被宽护卡住之后的报与推环（显式推环命令加带出路的错）：报的点名 R 的 txg、宽护挡着的槽数、两条出路；
/// 调推环入口之后 R 那一槽写掉、C 那一圈一槽不剩、隔离回到不含宽护的数（这几形根环表里没有被抛弃根，就是 0），之后普通写放得下；
/// 推之前、推之后与之后每一次写，池级 checker 不红 I-7.4、I-4.8。
pub fn wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
    images: &mut PoolCheckerImages,
    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
) {
    let PoolStuckBehindTheWideProtection {
        recorded,
        mut pool,
        stuck,
    } = pool_stuck_behind_the_wide_protection(images, form);
    assert_eq!(
        stuck.checkpoint_txg_of_the_last_ring_slot_the_push_reaches,
        Some(recorded.key.1),
        "{form:?}：报的 R 是录下来的那一条（R 那一槽挂载那一刻记成有效根）"
    );
    assert_eq!(
        stuck.last_ring_slot_the_push_reaches,
        target_for_publish(
            recorded.key.1,
            recorded.pool.parameters.geometry.root_ring_slots_per_region
        ),
        "{form:?}：推环要写到的是 R 那一槽"
    );
    let isolated_by_the_wide_protection: u64 = stuck
        .isolated_slots_on_each_device
        .iter()
        .map(|(_, isolated)| *isolated)
        .sum();
    assert!(
        isolated_by_the_wide_protection > 0
            && isolated_by_the_wide_protection <= isolated_slots_of(&pool.session().allocator),
        "{form:?}：宽护挡着的槽数大于 0、不多于两块盘隔离的槽数：{isolated_by_the_wide_protection}"
    );
    let why = format!("{form:?}");
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, &why);
    assert_eq!(
        ways_out.len(),
        2,
        "{form:?}：预演过推完之后这次覆盖写放得下，两条出路都列：{ways_out:?}"
    );
    assert_the_way_out_by_unmounting_names_the_recorded_publish_abandoned_and_warns_of_a_permanently_unreadable_account(
        &ways_out[0],
        &stuck,
        recorded.key.1,
        &why,
    );
    let released_in_the_rehearsal = released_by_the_push_if_the_change_is_admitted_after_it(
        &pushing_the_root_ring,
    )
    .unwrap_or_else(|| {
        panic!("{form:?}：预演推完之后这次覆盖写放得下，而预演的结局是 {pushing_the_root_ring:?}")
    });
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_the_recorded_publish_and_the_released_slots(
        &stuck,
        &ways_out[1],
        &released_in_the_rehearsal,
        WhatIsLeftAfterThePush::PublishTheChangeAgain,
        &why,
    );
    let isolated_before_the_push = isolated_slots_on_each_device_of(&pool.session().allocator);
    assert!(
        ring_slots_recorded_as_holding_an_abandoned_root_by(&pool.session().allocator).is_empty(),
        "{form:?}：这几形根环表里没有被抛弃根（隔离全是 C 那一圈的）"
    );
    let mut session = pool.session.take().expect("挂着的会话");
    let pushed = session
        .push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states(
            &mut pool.devices,
        );
    pool.session = Some(session);
    let pushed = pushed.unwrap_or_else(|refusal| panic!("{form:?}：推环做得成：{refusal:?}"));
    // 推环发成的最后一版与会话的现行那一版指着同一份（判决 m4-gpugc-publish-growth-r1 第二节 Q2.2：版本列表不再拷一整版）。
    let last_version_of_the_push = pushed
        .versions_published
        .last()
        .expect("推环至少发一次空发布");
    let current_version_of_the_session = pool
        .session()
        .current
        .shared_version_of_trees()
        .expect("推环接在有树的一版后面，推完还是有树的一版");
    assert!(
        Arc::ptr_eq(last_version_of_the_push, current_version_of_the_session),
        "{form:?}：推环发成的最后一版与会话的现行那一版指着同一份，不是两份内容相同的拷贝"
    );
    for version in &pushed.versions_published {
        pool.note_root_of_the_current_content(&version.root);
    }
    assert_eq!(
        u64::try_from(pushed.versions_published.len()).expect("次数"),
        stuck.empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written,
        "{form:?}：推环发的空发布次数就是报里写的那个数"
    );
    let image_after_the_push = pool.image();
    assert!(
        !root_is_in_the_ring(&image_after_the_push, recorded.key),
        "{form:?}：推环之后 R 那一槽被这个进程写掉、R 离开根环"
    );
    let allocator = &pool.session().allocator;
    assert!(
        allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .is_empty(),
        "{form:?}：推环之后 C 那一圈一槽不剩（宽护解开）"
    );
    assert_eq!(
        isolated_slots_of(allocator),
        0,
        "{form:?}：推环之后隔离回到不含宽护的数（根环表里没有被抛弃根）"
    );
    assert_eq!(
        released_in_the_rehearsal,
        isolated_slots_released_between(
            &isolated_before_the_push,
            &isolated_slots_on_each_device_of(allocator)
        ),
        "{form:?}：出路二说放出来的隔离槽数就是推完前后每块盘隔离数之差"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(allocator),
        0,
        "{form:?}：推环之后没有一个槽同时算进「已分配」与第九项"
    );
    images.hand_out(
        &format!("{form:?}：推环之后 I-7.4、I-4.8 不红"),
        || image_after_the_push.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
    for overwrite_number in 1..=OVERWRITES_AFTER_PUSHING_THE_ROOT_RING {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("{form:?}：推环之后第 {overwrite_number} 次覆盖写放得下：{refusal:?}")
        });
        images.hand_out(
            &format!("{form:?}：推环之后第 {overwrite_number} 次覆盖写之后 I-7.4、I-4.8 不红"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_PUSHING_THE_ROOT_RING {
        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            .unwrap_or_else(|refusal| {
                panic!("{form:?}：推环之后第 {sequential_write_number} 次顺序写放得下：{refusal:?}")
            });
        images.hand_out(
            &format!(
                "{form:?}：推环之后第 {sequential_write_number} 次顺序写之后 I-7.4、I-4.8 不红"
            ),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
}

/// 前缀覆盖写 20 次、回退到倒数第 7 版（第十四块报告第六节 `o×20 | b6` 那一行）。
pub fn run_the_scenario_where_a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
    images: &mut PoolCheckerImages,
) {
    wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: 20,
        },
    );
}

/// 前缀覆盖写 40 次、建一个 inode（第十四块报告第六节 `o×40 | n | rall` 那一行）。
pub fn run_the_scenario_where_an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
    images: &mut PoolCheckerImages,
) {
    wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
}

/// 前缀覆盖写 160 次、建一个 inode 再删掉它（第十四块报告第六节 `o×160+n | d` 那一行）。
pub fn run_the_scenario_where_a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
    images: &mut PoolCheckerImages,
) {
    wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
            overwrites_before: 160,
        },
    );
}

/// 卸载重挂之后接着覆盖写几次。
pub const OVERWRITES_AFTER_UNMOUNTING_AND_MOUNTING_AGAIN: usize = 5;

/// 出路一（卸载重挂）走得通：主形（覆盖写 40 次、回退到倒数第 7 版，调查报告第一节复现那一形）被宽护卡住之后正常卸载做成，
/// 再可写挂载一次（读回全零早已撤：R 的账读得出）：R 按新实例的实例表判成被抛弃、根环表记着它自己账里的落点，宽护不再出现（C 那一圈一槽不剩），
/// 之后覆盖写一次都不拒，池级 checker 不红 I-7.4、I-4.8。
pub fn run_the_scenario_where_unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_the_recorded_publish_and_lets_writes_through(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        recorded, mut pool, ..
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
        },
    );
    let mut session = pool.session.take().expect("挂着的会话");
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            for version in &raised.publishes {
                pool.note_root_of_the_current_content(&version.root);
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("被宽护卡住时正常卸载做得成：{refusal:?}"),
    }
    pool.crash_and_mount_writable()
        .expect("卸载之后再可写挂载做得成（R 的账读得出）");
    let occupancy = pool
        .session()
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表");
    let recorded_for_the_recorded_publish = occupancy
        .placements_recorded_for_the_abandoned_root(recorded.key.0, recorded.key.1)
        .expect("再挂时根环表把 R 记成被抛弃根（按新实例的实例表）");
    assert!(
        !recorded_for_the_recorded_publish.is_empty(),
        "再挂时 R 的账读得出：根环表记着它自己账里的落点"
    );
    assert!(
        occupancy
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .is_empty(),
        "再挂时宽护不再出现：C 那一圈一槽不剩"
    );
    for overwrite_number in 1..=OVERWRITES_AFTER_UNMOUNTING_AND_MOUNTING_AGAIN {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("卸载重挂之后第 {overwrite_number} 次覆盖写放得下：{refusal:?}")
        });
        images.hand_out(
            &format!("卸载重挂之后第 {overwrite_number} 次覆盖写之后 I-7.4、I-4.8 不红"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
}

/// 推到半路取不到落点那一形人为压到每块盘没挡的槽几个（调查报告第七节第 1 条 `PROBE_FILL_ISO_LEAVE=40`）。
pub const UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH: u64 = 40;

/// 主形卡住之后把每块盘没挡的槽隔离到只剩 [`UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH`] 个（注入级）。
pub fn pool_stuck_with_too_few_unblocked_slots_for_the_whole_push(
    images: &mut PoolCheckerImages,
) -> PoolStuckBehindTheWideProtection {
    let mut stuck_pool = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
        },
    );
    let session = stuck_pool.pool.session.as_mut().expect("挂着的会话");
    for device_map in &mut session.allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if device_map.unblocked_slots() <= UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH {
                break;
            }
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                device_map.isolate(SlotNumber(slot), 1);
            }
        }
    }
    stuck_pool
}

/// 推到半路空发布取不到落点（调查报告第七节第 1 条造的那一次）：主形被宽护卡住之后，把每块盘没挡的槽隔离到只剩 40 个（人为的），
/// 推环要发的那一串在第 7 次左右取不到落点。推环入口在任何写之前拒（预演里被拒，`PlacementRefused`）：两块盘逐字节不变、
/// 分配器（`{:?}` 整份）与会话的现行那一版都不变——不留半截不一致的状态。
pub fn run_the_scenario_where_pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_with_too_few_unblocked_slots_for_the_whole_push(images);
    let image_before = pool.image();
    let allocator_before = format!("{:?}", pool.session().allocator);
    let current_root_before = *pool.session().current.root();
    let mut session = pool.session.take().expect("挂着的会话");
    let pushed = session
        .push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states(
            &mut pool.devices,
        );
    pool.session = Some(session);
    match pushed {
        Err(RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
            refused_publish_in_the_push,
            publishes_in_the_push,
            cause: PublishError::PlacementRefused { .. },
        }) => {
            assert!(
                refused_publish_in_the_push > 1
                    && refused_publish_in_the_push <= publishes_in_the_push,
                "推到半路才取不到落点（前面几次放得下，阳性对照）：第 {refused_publish_in_the_push} 次，共 {publishes_in_the_push} 次"
            );
            assert_eq!(
                u64::try_from(publishes_in_the_push).expect("次数"),
                stuck.empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written,
                "这一串的次数就是报里写的那个数"
            );
        }
        other => panic!(
            "推到半路取不到落点：在任何写之前拒成预演被拒、拒因是取不到落点，而交回 {other:?}"
        ),
    }
    assert!(pool.image() == image_before, "推环被拒：两块盘逐字节不变");
    assert_eq!(
        format!("{:?}", pool.session().allocator),
        allocator_before,
        "推环被拒：分配器整份不变"
    );
    assert_eq!(
        *pool.session().current.root(),
        current_root_before,
        "推环被拒：会话的现行那一版不变"
    );
}

/// 要出路时预演里推环自己被拒（攻方 `research/prompts/m3-batch10z-tu-code-r1-opus-output.md` 第五节变异 K：自然历史里没走到这一支，注入级）：
/// 主形卡住之后把每块盘没挡的槽隔离到只剩 40 个，要出路：预演交回推环被拒（预演里那一串取不到落点）、出路只列卸载再挂，
/// 要出路那一次两块盘逐字节不变；照出路二真调推环入口，拒成同一种，`Debug` 逐字相同。
pub fn run_the_scenario_where_ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_with_too_few_unblocked_slots_for_the_whole_push(images);
    let image_before = pool.image();
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, "没挡的槽只剩 40 个");
    assert!(pool.image() == image_before, "要出路：两块盘逐字节不变");
    let RootRingPushRehearsal::PushRefused(rehearsed_push_refusal) = pushing_the_root_ring else {
        panic!("预演里推环那一串取不到落点：预演交回推环被拒，而交回 {pushing_the_root_ring:?}");
    };
    assert!(
        matches!(
            *rehearsed_push_refusal,
            RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
                cause: PublishError::PlacementRefused { .. },
                ..
            }
        ),
        "预演里推环拒成推环自己那一串预演取不到落点（阳性对照：注入打到了）：{rehearsed_push_refusal:?}"
    );
    assert_eq!(
        ways_out.len(),
        1,
        "推环在预演里被拒：只列卸载再挂：{ways_out:?}"
    );
    let real_push_refusal =
        push_the_root_ring_of(&mut pool).expect_err("照出路二真调推环入口：同样被拒");
    assert_eq!(
        format!("{real_push_refusal:?}"),
        format!("{rehearsed_push_refusal:?}"),
        "预演里推环的拒绝与真调推环入口的拒绝逐字相同"
    );
}
