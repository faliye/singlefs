use super::*;

/// 根环 R × S = 24 槽，txg 0–23 各一条有效根，另有一个释放代 1 的落点在 defer 队列里。
fn pool_whose_root_ring_holds_txg_0_through_23() -> PoolAllocator {
    let mut pool = rebuilt_pool(records_on_both_devices(&[(50180, 2, 1, true)]));
    let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
    pool.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
        slots_per_region,
        (0..24)
            .map(|txg| {
                (
                    target_for_publish(CheckpointTxg(txg), slots_per_region),
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: CheckpointTxg(txg),
                    },
                )
            })
            .collect(),
        CheckpointTxg(0),
        CheckpointTxg(23),
    ));
    pool
}

/// C518（一次挂载之内环转过一圈之后不回收） 那张根环表：不跳地记下 txg 24（盖掉 txg 0 的根），环里最旧有效根变成 1，
/// 释放代 1 的落点当场回收。
#[test]
fn root_ring_occupancy_that_follows_every_root_reclaims_as_the_ring_turns() {
    let mut following = pool_whose_root_ring_holds_txg_0_through_23();
    following.record_root_written_by_this_process(CheckpointTxg(24));
    for device_map in &following.devices {
        assert!(
            device_map.is_free(SlotNumber(50180)) && device_map.is_free(SlotNumber(50181)),
            "盘 {:?}：txg 24 盖掉 txg 0 之后环里最旧有效根是 1，释放代 1 的落点回收",
            device_map.device
        );
        assert_eq!(device_map.deferred_slots(), 0);
    }
}

/// 抬 F 那一串取的被抛弃根落点按根认、不只按槽认（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句）：
/// 24 槽的环里 txg 8 那一槽记着实例 2 的被抛弃根（读出那一遍记成 `AbandonedRoot`），txg 9 那一槽是两遍里只有一遍自证过、被抛弃的
/// 实例 2 的根。同一条根交回记着的落点；同一槽上 txg 差一整圈的根、或实例代号不同的根交 `None`；有效根那一槽交 `None`。
#[test]
fn placements_recorded_for_an_abandoned_root_answer_only_for_the_same_root_not_for_another_root_on_its_ring_slot(
) {
    let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
    let ring_length = 3 * slots_per_region.count();
    let recorded_on_txg_8 = vec![PlacementOnDevice {
        device: DeviceIdentity(0),
        slot: SlotNumber(50180),
        span: 2,
    }];
    let recorded_on_txg_9 = vec![PlacementOnDevice {
        device: DeviceIdentity(1),
        slot: SlotNumber(50190),
        span: 1,
    }];
    let root_on_txg_9 = RootRecord {
        filesystem_identifier: [5u8; 16],
        unmount_marker: crate::root_record::UnmountMarker::NotWrittenByTheUnmountSequence,
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(9),
        tree_table: crate::pointer::NodePointer::empty_root(),
        tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        rollback_floor: CheckpointTxg(0),
        instance_table: crate::pointer::NodePointer::empty_root(),
        mapping_root: crate::pointer::NodePointer::empty_root(),
        allocation_record_tree_root: crate::pointer::NodePointer::empty_root(),
        reclaim_threshold: crate::address::CheckpointTxg(0),
        pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
    };
    let occupancy = RootRingOccupancy::read_from_the_ring(
        slots_per_region,
        vec![
            (
                target_for_publish(CheckpointTxg(8), slots_per_region),
                RootRingOccupant::AbandonedRoot {
                    instance: InstanceGeneration(2),
                    checkpoint_txg: CheckpointTxg(8),
                    referenced_placements: recorded_on_txg_8.clone(),
                },
            ),
            (
                target_for_publish(CheckpointTxg(9), slots_per_region),
                RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                    root: RootReadOnOneRebuildPass::AbandonedRoot {
                        referenced_placements: recorded_on_txg_9.clone(),
                    },
                    root_record_self_verified_on_one_rebuild_pass: Box::new(root_on_txg_9),
                },
            ),
            (
                target_for_publish(CheckpointTxg(10), slots_per_region),
                RootRingOccupant::ValidRoot {
                    checkpoint_txg: CheckpointTxg(10),
                },
            ),
        ],
        CheckpointTxg(0),
        CheckpointTxg(10),
    );
    assert_eq!(
        occupancy
            .placements_recorded_for_the_abandoned_root(InstanceGeneration(2), CheckpointTxg(8)),
        Some(recorded_on_txg_8),
        "记成被抛弃根的那一条：交回挂载时记着的落点"
    );
    assert_eq!(
        occupancy
            .placements_recorded_for_the_abandoned_root(InstanceGeneration(2), CheckpointTxg(9)),
        Some(recorded_on_txg_9),
        "两遍里只有一遍自证过、被抛弃的那一条：按那一遍读出的整条根记录认，交回记着的落点"
    );
    for (instance, checkpoint_txg) in [
        (InstanceGeneration(2), CheckpointTxg(8 + ring_length)),
        (InstanceGeneration(3), CheckpointTxg(8)),
        (InstanceGeneration(2), CheckpointTxg(9 + ring_length)),
        (InstanceGeneration(3), CheckpointTxg(9)),
        (InstanceGeneration(2), CheckpointTxg(10)),
    ] {
        assert_eq!(
            occupancy.placements_recorded_for_the_abandoned_root(instance, checkpoint_txg),
            None,
            "({instance:?}, {checkpoint_txg:?}) 不是那一槽记着的那条被抛弃根（同槽差一整圈、实例代号不同，或那一槽是有效根）：不交落点"
        );
    }
}

/// 实例 2、txg `checkpoint_txg` 的一条根记录（只有 (实例, txg) 有用，指针都是空的）：两遍里只有一遍自证过的槽记着的那整条根。
fn root_record_of_instance_2_at(checkpoint_txg: CheckpointTxg) -> RootRecord {
    RootRecord {
        filesystem_identifier: [5u8; 16],
        unmount_marker: crate::root_record::UnmountMarker::NotWrittenByTheUnmountSequence,
        instance: InstanceGeneration(2),
        checkpoint_txg,
        tree_table: crate::pointer::NodePointer::empty_root(),
        tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        rollback_floor: CheckpointTxg(0),
        instance_table: crate::pointer::NodePointer::empty_root(),
        mapping_root: crate::pointer::NodePointer::empty_root(),
        allocation_record_tree_root: crate::pointer::NodePointer::empty_root(),
        reclaim_threshold: crate::address::CheckpointTxg(0),
        pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
    }
}

/// 24 槽的环：txg 11 那一槽挂载时读坏，12 读回全零，13 挂载时读坏、进程内重读读出有效根，14 两遍里只有一遍自证过、那一遍读出的是有效根
/// （实例 2），15 是有效根，16 是实例 2 的被抛弃根；C 那一圈在 11 到 16 每一槽都记着一个落点（槽号 50200 + txg）。
fn occupancy_with_every_kind_of_slot_from_txg_11_through_16_and_unapplied_records_on_each(
) -> RootRingOccupancy {
    let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
    let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
    let mut occupancy = RootRingOccupancy::read_from_the_ring(
        slots_per_region,
        vec![
            (
                ring_slot_of(11),
                RootRingOccupant::BadAtMount(TxgsARingSlotBadAtMountCanHold::new(
                    CheckpointTxg(11),
                    CheckpointTxg(11),
                )),
            ),
            (
                ring_slot_of(12),
                RootRingOccupant::AllZeroAtMount {
                    txgs_it_can_hold: TxgsARingSlotBadAtMountCanHold::new(
                        CheckpointTxg(12),
                        CheckpointTxg(12),
                    ),
                },
            ),
            (
                ring_slot_of(13),
                RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                    checkpoint_txg: CheckpointTxg(13),
                },
            ),
            (
                ring_slot_of(14),
                RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                    root: RootReadOnOneRebuildPass::ValidRoot {
                        checkpoint_txg: CheckpointTxg(14),
                    },
                    root_record_self_verified_on_one_rebuild_pass: Box::new(
                        root_record_of_instance_2_at(CheckpointTxg(14)),
                    ),
                },
            ),
            (
                ring_slot_of(15),
                RootRingOccupant::ValidRoot {
                    checkpoint_txg: CheckpointTxg(15),
                },
            ),
            (
                ring_slot_of(16),
                RootRingOccupant::AbandonedRoot {
                    instance: InstanceGeneration(2),
                    checkpoint_txg: CheckpointTxg(16),
                    referenced_placements: Vec::new(),
                },
            ),
        ],
        CheckpointTxg(0),
        CheckpointTxg(16),
    );
    occupancy.placements_named_by_unapplied_journal_records = (11..=16)
        .map(|txg| {
            (
                ring_slot_of(txg),
                vec![PlacementOnDevice {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(50200 + txg),
                    span: 1,
                }],
            )
        })
        .collect();
    occupancy
}

/// 抬 F 那一串取被抛弃根记着的落点（[`RootRingOccupancy::placements_recorded_for_the_abandoned_root`]）只认记成被抛弃根的槽：
/// 挂载时读坏、读回全零、读坏后进程内重读读出有效根、两遍里只有一遍自证过而那一遍是有效根这四类槽，这一遍读出一条被抛弃根时交 `None`
/// （照今天读盘），不交一串空落点——交空落点就是「这条根什么都不引用」，不隔离也不读盘，它的单元掉出影子账。
#[test]
fn placements_recorded_for_an_abandoned_root_answer_none_on_slots_recorded_bad_all_zero_or_valid_at_mount(
) {
    let occupancy =
        occupancy_with_every_kind_of_slot_from_txg_11_through_16_and_unapplied_records_on_each();
    for (txg, kind_of_slot) in [
        (11, "挂载时读坏"),
        (12, "挂载时读回全零"),
        (13, "挂载时读坏、进程内重读读出有效根"),
        (14, "两遍里只有一遍自证过、那一遍是有效根"),
        (15, "有效根"),
    ] {
        assert_eq!(
            occupancy.placements_recorded_for_the_abandoned_root(
                InstanceGeneration(2),
                CheckpointTxg(txg)
            ),
            None,
            "txg {txg} 那一槽记的是「{kind_of_slot}」：没记着这条被抛弃根的落点，交 None 照今天读盘，不交空落点"
        );
    }
}

/// 准入抬 F 的上限要并的槽（[`RootRingOccupancy::ring_slots_bad_or_reading_back_all_zero_at_mount`]）是挂载时读坏的与读回全零的两类
/// （D16（发布语义） 已定项 1「抬 F 的上限」那一行，读回全零那一半），各带它能住的 txg；进程内重读读出有效根的、
/// 两遍里只有一遍自证过的、有效根、被抛弃根都不在其内。只取读坏那一类的 [`RootRingOccupancy::ring_slots_bad_at_mount`] 照旧没有读回全零的那一槽。
#[test]
fn the_ring_slots_merged_into_the_admission_ceiling_are_the_slots_bad_at_mount_and_the_slots_reading_back_all_zero_at_mount(
) {
    let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
    let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
    let occupancy =
        occupancy_with_every_kind_of_slot_from_txg_11_through_16_and_unapplied_records_on_each();
    assert_eq!(
        occupancy.ring_slots_bad_or_reading_back_all_zero_at_mount(),
        BTreeMap::from([
            (
                ring_slot_of(11),
                TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(11), CheckpointTxg(11))
            ),
            (
                ring_slot_of(12),
                TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(12), CheckpointTxg(12))
            ),
        ]),
        "并进上限的是挂载时读坏的 txg 11 那一槽与读回全零的 txg 12 那一槽，各带能住的 txg"
    );
    assert_eq!(
        occupancy
            .ring_slots_bad_at_mount()
            .keys()
            .copied()
            .collect::<Vec<RootRingSlot>>(),
        vec![ring_slot_of(11)],
        "只取读坏那一类的那一份没有读回全零的槽"
    );
}

/// 根环表里没记着落点的被抛弃根借 C 那一圈记着的落点：认同条件不只按槽号——那一槽记的是有效根且 txg 相同
/// （两遍里只有一遍自证过的那一类还比实例代号）。同一条根交回 C 在那一槽记着的落点；同一槽上 txg 差一整圈的另一次发布、实例代号不同、
/// 那一槽记的是被抛弃根或挂载时读坏、读回全零的槽，交 `None`；C 在那一槽解开之后（这个进程写掉了那一槽）也交 `None`。
#[test]
fn placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_answer_only_for_the_same_root_on_its_ring_slot(
) {
    let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
    let ring_length = 3 * slots_per_region.count();
    let mut occupancy =
        occupancy_with_every_kind_of_slot_from_txg_11_through_16_and_unapplied_records_on_each();
    for txg in [13, 14, 15] {
        assert_eq!(
            occupancy.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
                InstanceGeneration(2),
                CheckpointTxg(txg)
            ),
            Some(vec![PlacementOnDevice {
                device: DeviceIdentity(0),
                slot: SlotNumber(50200 + txg),
                span: 1,
            }]),
            "txg {txg} 那一槽记的是同一条有效根：交回 C 在那一槽记着的落点"
        );
    }
    for (instance, checkpoint_txg, why) in [
        (
            InstanceGeneration(2),
            CheckpointTxg(15 + ring_length),
            "同一槽差一整圈的另一次发布",
        ),
        (
            InstanceGeneration(2),
            CheckpointTxg(13 + ring_length),
            "同一槽差一整圈的另一次发布（进程内重读读出的有效根那一槽）",
        ),
        (
            InstanceGeneration(2),
            CheckpointTxg(14 + ring_length),
            "同一槽差一整圈的另一次发布（两遍里只有一遍自证过那一槽）",
        ),
        (
            InstanceGeneration(3),
            CheckpointTxg(14),
            "实例代号与那一遍读出的整条根记录不同",
        ),
        (InstanceGeneration(2), CheckpointTxg(11), "那一槽挂载时读坏"),
        (
            InstanceGeneration(2),
            CheckpointTxg(12),
            "那一槽挂载时读回全零",
        ),
        (
            InstanceGeneration(2),
            CheckpointTxg(16),
            "那一槽记的是被抛弃根（它记着的落点归另一个方法）",
        ),
        (
            InstanceGeneration(2),
            CheckpointTxg(17),
            "那一槽表里什么都没记",
        ),
    ] {
        assert_eq!(
            occupancy.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
                instance,
                checkpoint_txg
            ),
            None,
            "({instance:?}, {checkpoint_txg:?})：{why}，不借 C 的落点"
        );
    }
    occupancy
        .placements_named_by_unapplied_journal_records
        .remove(&target_for_publish(CheckpointTxg(15), slots_per_region));
    assert_eq!(
        occupancy.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            InstanceGeneration(2),
            CheckpointTxg(15)
        ),
        None,
        "C 在那一槽解开之后没有可借的落点：交 None，调用方照今天读盘"
    );
}

/// C 那一圈在那一槽有条目、落点是空的（较新可读根或没施加的发布的账
/// 读不出、记录不齐、映射也现算不成，而现行账里有 C694 的 B 那道凭据时，`mount::placements_of_an_unapplied_publish_whose_account_is_unreadable`
/// 交空串、照样记进 C 那一槽）：借到的是 `Some(空)`，不是 `None`。D16（发布语义） 已定项 1「根槽这一次读坏」那一行：C 那一圈在那一槽记着
/// 同一条根时「照 C 那一圈记着的落点算、不读账、读不出不拒」，「C 也没记着的照今天读盘」——条目在就是记着了，空串照空串算，
/// 与根环表里记着空串（挂载时凭据放行）同一处置；凭据说的是那条根引用的单元已经没有要护的。
#[test]
fn placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_lend_an_empty_entry_as_some_empty(
) {
    let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
    let mut occupancy =
        occupancy_with_every_kind_of_slot_from_txg_11_through_16_and_unapplied_records_on_each();
    occupancy
        .placements_named_by_unapplied_journal_records
        .insert(
            target_for_publish(CheckpointTxg(15), slots_per_region),
            Vec::new(),
        );
    assert_eq!(
        occupancy.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            InstanceGeneration(2),
            CheckpointTxg(15)
        ),
        Some(Vec::new()),
        "那一槽记的是同一条有效根、C 在那一槽有条目而落点是空的：借到空串（照它算、不读账），不交 None（照今天读盘）"
    );
}

/// 环上有洞留下的旧根压住进程内的回收门槛（D16（发布语义） 已定项 1「环里最旧有效根」）：24 槽的环里
/// txg 1 那一槽留着 txg 1 的根（txg 25、49 那两次的根槽都没落），其余 23 槽住着 txg 26–48；这个进程记到 txg 49（没有根的那一个）
/// 之后再记 txg 50，盖掉的是 txg 26。txg 1 离最新根已过两圈，仍是环里最旧的有效根：释放代 2 的落点不回收、仍在 defer 里。
/// 照一圈窗口取（锚 50，窗口 txg > 26）时 txg 1 被滤掉、门槛跳到 27，那两槽当场回收。
#[test]
fn root_ring_occupancy_keeps_the_reclaim_floor_at_the_oldest_valid_root_however_far_below_the_newest_it_is(
) {
    let mut pool = rebuilt_pool(records_on_both_devices(&[(50180, 2, 2, true)]));
    let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
    let ring_length = 3 * slots_per_region.count();
    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = std::iter::once(1)
        .chain(ring_length + 2..=2 * ring_length)
        .map(|txg| {
            (
                target_for_publish(CheckpointTxg(txg), slots_per_region),
                RootRingOccupant::ValidRoot {
                    checkpoint_txg: CheckpointTxg(txg),
                },
            )
        })
        .collect();
    assert_eq!(
        occupants.len(),
        usize::try_from(ring_length).expect("环长"),
        "每个槽上一条根"
    );
    pool.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
        slots_per_region,
        occupants,
        CheckpointTxg(0),
        CheckpointTxg(2 * ring_length + 1),
    ));
    pool.record_root_written_by_this_process(CheckpointTxg(2 * ring_length + 2));
    for device_map in &pool.devices {
        assert!(
            !device_map.is_free(SlotNumber(50180)) && !device_map.is_free(SlotNumber(50181)),
            "盘 {:?}：txg 1 的根离最新根 txg {} 已过两圈，仍是环里最旧的有效根，释放代 2 的落点不回收",
            device_map.device,
            2 * ring_length + 2
        );
        assert_eq!(
            device_map.deferred_slots(),
            2,
            "释放代 2 的那两槽仍在 defer 里"
        );
    }
}

/// E2 那一判的取法（D16（发布语义） 已定项 1；读坏的槽钉在它可能住过的最旧 txg）：
/// 12 槽的环（S = 4）；txg 13–23 住着有效根，区域 0 槽 0 挂载那一刻读坏、钉在它可能住过的最旧 txg 0（它能住的最大 txg 是 12），
/// 是环里最小的那个 ⇒ 重读那一槽一次。
fn oldest_in_the_ring_when_the_slot_bad_at_mount_rereads_as(
    reread_of_the_slot: RereadOfARootRingSlotBadAtMount,
) -> (
    Option<CheckpointTxg>,
    Vec<(RootRingSlot, RereadOfARootRingSlotBadAtMount)>,
) {
    let slots_per_region =
        RootRingSlotsPerRegion::from_system_configuration_field(4).expect("4 在区间里");
    let hidden_slot = RootRingSlot { region: 0, slot: 0 };
    assert_eq!(
        (
            target_for_publish(CheckpointTxg(0), slots_per_region),
            target_for_publish(CheckpointTxg(12), slots_per_region)
        ),
        (hidden_slot, hidden_slot)
    );
    let valid_root_txgs: Vec<CheckpointTxg> = (13..=23).map(CheckpointTxg).collect();
    oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
        &valid_root_txgs,
        BTreeMap::from([(hidden_slot, CheckpointTxg(0))]),
        &mut |ring_slot| {
            assert_eq!(ring_slot, hidden_slot, "只重读环里最小的那一槽");
            reread_of_the_slot
        },
    )
}

/// 重读读出的是上一圈的 txg 0（离锚 23 已过一圈）：回收门槛取整个环（RGall），照读出来的根算，最旧有效根是 0。
#[test]
fn the_slot_bad_at_mount_rereading_as_a_root_of_the_previous_revolution_counts_by_the_txg_read() {
    assert_eq!(
        oldest_in_the_ring_when_the_slot_bad_at_mount_rereads_as(
            RereadOfARootRingSlotBadAtMount::ValidRoot {
                checkpoint_txg: CheckpointTxg(0),
            }
        ),
        (
            Some(CheckpointTxg(0)),
            vec![(
                RootRingSlot { region: 0, slot: 0 },
                RereadOfARootRingSlotBadAtMount::ValidRoot {
                    checkpoint_txg: CheckpointTxg(0),
                }
            )]
        )
    );
}

/// 重读仍读不出：照旧钉在它可能住过的最旧 txg 0，不按它能住的最大 txg 12 算。
#[test]
fn the_slot_bad_at_mount_still_unreadable_on_the_reread_stays_pinned_at_the_oldest_txg_it_can_have_held(
) {
    assert_eq!(
        oldest_in_the_ring_when_the_slot_bad_at_mount_rereads_as(
            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
        ),
        (
            Some(CheckpointTxg(0)),
            vec![(
                RootRingSlot { region: 0, slot: 0 },
                RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
            )]
        )
    );
}

/// 重读读回全零：不算确认为空，照旧钉在它可能住过的最旧 txg 0。
#[test]
fn the_slot_bad_at_mount_reading_back_all_zero_on_the_reread_stays_pinned_at_the_oldest_txg_it_can_have_held(
) {
    assert_eq!(
        oldest_in_the_ring_when_the_slot_bad_at_mount_rereads_as(
            RereadOfARootRingSlotBadAtMount::AllZero
        ),
        (
            Some(CheckpointTxg(0)),
            vec![(
                RootRingSlot { region: 0, slot: 0 },
                RereadOfARootRingSlotBadAtMount::AllZero
            )]
        )
    );
}

/// 环里还有比那一槽钉着的 txg 更旧的有效根：不重读（重读的口子被调到就断言失败）。
#[test]
fn the_slot_bad_at_mount_is_not_reread_while_an_older_valid_root_is_in_the_ring() {
    // 区域 0 槽 1 可能住过的最旧 txg 是 3，环里还有 txg 2 的有效根。
    let valid_root_txgs: Vec<CheckpointTxg> = std::iter::once(2)
        .chain(4..=13)
        .map(CheckpointTxg)
        .collect();
    assert_eq!(
        oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
            &valid_root_txgs,
            BTreeMap::from([(RootRingSlot { region: 0, slot: 1 }, CheckpointTxg(3))]),
            &mut |ring_slot| panic!("环里 txg 2 比那一槽钉着的 3 旧，不该重读 {ring_slot:?}"),
        ),
        (Some(CheckpointTxg(2)), Vec::new())
    );
}

/// 记到的根 txg 跳了一格（txg 24 那条根没经分配器写出去）：这张表说的环已经不是盘上的环，断言失败，不按猜的环接着回收。
#[test]
#[should_panic(expected = "装了根环表的进程写的根 txg 逐个加一")]
fn root_ring_occupancy_that_missed_the_root_panics_instead_of_reclaiming_by_the_guessed_ring() {
    let mut stopped = pool_whose_root_ring_holds_txg_0_through_23();
    stopped.record_root_written_by_this_process(CheckpointTxg(25));
}

/// 分配器没装根环表（只 `new` 加 mkfs 那两个单元，C551 那一形）而记了一条根：断言失败，不静默跳过回收
#[test]
#[should_panic(expected = "分配器没装根环表却记了一条根")]
fn recording_a_root_on_an_allocator_without_a_root_ring_occupancy_panics_instead_of_never_reclaiming(
) {
    let mut without_a_root_ring = pool_after_mkfs();
    assert!(
        without_a_root_ring.root_ring_occupancy().is_none(),
        "前提：new 不装根环表"
    );
    without_a_root_ring.record_root_written_by_this_process(CheckpointTxg(1));
}
