//! 被抛弃根或有效根只在重建那一遍自证过：抬 F 读回全零、从重读起读不出时上限与隔离怎么定，卸载之后与分配器里一直留到它离开根环。

use super::*;

/// K2-2 那组历史里盘 `device` 那几个区域上每条不被抛弃的自证根住的槽（这组历史里被抛弃的只有 C，[`pool_with_an_abandoned_root`]）。
pub fn ring_slots_of_the_roots_not_abandoned_on(
    history: &PoolWithAnAbandonedRootAndThePlacesItWrote,
    device: DeviceIdentity,
) -> Vec<RootRingSlot> {
    self_verified_roots_in_the_ring(&history.image)
        .into_iter()
        .filter(|(ring_slot, _)| {
            history.parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")]
                == device
                && *ring_slot != history.ring_slot_of_the_abandoned_root
        })
        .map(|(ring_slot, _)| ring_slot)
        .collect()
}

/// 重建那一遍读得出的那一槽之后怎么读坏（`s1_plans`
/// 里 a、c 两形）。
#[derive(Clone, Copy, Debug)]
pub enum ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass {
    /// a：从重读那一遍起每一次都读成这一样。
    FailingFromTheRereadOnward(RootRingSlotReadFault),
    /// c：从重读那一遍起每三次读里前两次读成这一样、第三次读得出（算上限那一遍读得出、抬 F 那一遍读它两次都读坏的那一形）。
    FailingTwoReadsInEveryThreeFromTheRereadOn(RootRingSlotReadFault),
}

/// 交替读坏那一形点名到重建那一遍之后第几次读为止：远多于一次挂载加 [`USER_CHANGES_AFTER_THE_REMOUNT`] 次改动读那一槽的次数。
pub const READS_NAMED_BY_THE_ALTERNATING_FAULT: u64 = 400;

/// 再挂之后接着发几次改动（攻方 K1-1、K1-2、K1-3 同一个数）。
pub const USER_CHANGES_AFTER_THE_REMOUNT: usize = 40;

/// 再挂之后接着发的用户改动。
#[derive(Clone, Copy, Debug)]
pub enum UserChangesAfterTheRemount {
    /// 每次覆盖写一次。
    Overwrites,
    /// 第 `first_sequential_write` 次之前覆盖写，从它起每次顺序写 8 个数据单元（攻方 K1-1 那一档）。
    OverwritesThenSequentialWritesOfEightDataUnitsFrom { first_sequential_write: usize },
}

/// 一次可写挂载交回的盘留着读坏的计划接着用：挂载的结局、挂载之后的盘、读根环的那一份计数。
pub struct RemountKeepingTheFaultedDevices {
    pub mounted: Result<Mounted, MountError>,
    pub devices: Vec<(DeviceIdentity, DeviceFailingTheRebuildPass)>,
    pub root_ring_reads: SharedRootRingReadsWithFaults,
}

/// K2-2 那组历史上再可写挂载（准入判着、影子账开）：重建那一遍盘 `device_failing_the_rebuild_pass` 上每条不被抛弃的根住的槽报读错
/// （那块盘一条有效根都没读出，rr2 整环重读），`slot` 第一遍读得出、之后照 `reads_of_the_slot` 读坏；这个进程写过的槽不再读坏。
pub fn remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
    history: &PoolWithAnAbandonedRootAndThePlacesItWrote,
    device_failing_the_rebuild_pass: DeviceIdentity,
    slot: RootRingSlot,
    reads_of_the_slot: ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass,
) -> RemountKeepingTheFaultedDevices {
    let mut faulted_reads = faulted_reads_of(
        &ring_slots_of_the_roots_not_abandoned_on(history, device_failing_the_rebuild_pass)
            .into_iter()
            .filter(|ring_slot| *ring_slot != slot)
            .collect::<Vec<_>>(),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    let mut slots_failing_from_a_read_onward = BTreeMap::new();
    match reads_of_the_slot {
        ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(fault) => {
            slots_failing_from_a_read_onward
                .insert(slot, (READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD, fault));
        }
        ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingTwoReadsInEveryThreeFromTheRereadOn(
            fault,
        ) => {
            // 迭代上界是点名的读数；没有提前出口。
            for read in READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD
                ..READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD
                    + READS_NAMED_BY_THE_ALTERNATING_FAULT
            {
                if !(read - READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD).is_multiple_of(3) {
                    faulted_reads.insert((slot, read), fault);
                }
            }
        }
    }
    let root_ring_reads = root_ring_reads_failing_with(
        &history.image,
        faulted_reads,
        slots_failing_from_a_read_onward,
        FaultsOnARootRingSlotThisProcessWrote::StopFailing,
    );
    let mut devices: Vec<(DeviceIdentity, DeviceFailingTheRebuildPass)> =
        plain_devices_on(&history.image)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    DeviceFailingTheRebuildPass {
                        identity,
                        inner: device,
                        root_ring_reads: root_ring_reads.clone(),
                    },
                )
            })
            .collect();
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &history.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::RereadImmediately,
    );
    RemountKeepingTheFaultedDevices {
        mounted,
        devices,
        root_ring_reads,
    }
}

/// 挂上之后那张根环表里 `slot` 是不是记成重建分配器两遍里只有一遍自证过的那一类。
pub fn slot_is_self_verified_on_only_one_rebuild_pass(
    mounted: &Mounted,
    slot: RootRingSlot,
) -> bool {
    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(
        &mounted.allocator,
    )
    .contains_key(&slot)
}

/// 再挂、接着发改动之后看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 挂上之后那一槽记成重建分配器两遍里只有一遍自证过的那一类。
    pub the_slot_is_self_verified_on_only_one_rebuild_pass: bool,
    /// 挂上之后那一槽还读坏过（读坏落在抬 F 那一串上，不只落在挂载里）。
    pub the_slot_read_bad_after_the_mount: bool,
    /// 做成了几次改动。
    pub changes_published: usize,
    /// 被拒的那一次改动报的错（没被拒是 `None`）。
    pub refusal: Option<String>,
    /// C 还在根环里时 F 抬过了 C 的 txg（之后的回收与影子账按新 F 算，C 引用、挂载那一刻被候选根豁免的单元要补隔离）。
    pub floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: bool,
}

/// 照 [`remount_with_a_slot_self_verified_only_on_the_rebuild_pass`] 再挂，盘留着读坏的计划，接着发 [`USER_CHANGES_AFTER_THE_REMOUNT`] 次
/// `user_changes`（被拒就停），每次之后两块盘都在时跑池级 checker。
pub fn changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
    images: &mut PoolCheckerImages,
    history: &PoolWithAnAbandonedRootAndThePlacesItWrote,
    device_failing_the_rebuild_pass: DeviceIdentity,
    slot: RootRingSlot,
    reads_of_the_slot: ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass,
    user_changes: UserChangesAfterTheRemount,
) -> ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
    let RemountKeepingTheFaultedDevices {
        mounted,
        devices,
        root_ring_reads,
    } = remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
        history,
        device_failing_the_rebuild_pass,
        slot,
        reads_of_the_slot,
    );
    let mounted = match mounted {
        Ok(mounted) => mounted,
        Err(error) => {
            return ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
                root_ring_of_the_rebuilt_allocator: Err(format!("{error:?}")),
                the_slot_is_self_verified_on_only_one_rebuild_pass: false,
                the_slot_read_bad_after_the_mount: false,
                changes_published: 0,
                refusal: None,
                floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: false,
            };
        }
    };
    let faulted_reads_by_the_end_of_the_mount = root_ring_reads.faulted_reads();
    let root_ring_of_the_rebuilt_allocator =
        Ok(mounted.output.rereads.root_ring_of_the_rebuilt_allocator);
    let the_slot_is_self_verified_on_only_one_rebuild_pass =
        slot_is_self_verified_on_only_one_rebuild_pass(&mounted, slot);
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool: PoolUnderTest<DeviceFailingTheRebuildPass> = PoolUnderTest {
        parameters: history.parameters.clone(),
        device_bytes: history.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: history.write_time_seconds + 100,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut changes_published = 0;
    let mut refusal = None;
    let mut floor_raised_above_the_abandoned_root_while_it_was_in_the_ring = false;
    // 迭代上界是 USER_CHANGES_AFTER_THE_REMOUNT；跨轮携带做成的次数与第一次违例；提前出口只有改动被拒。
    for change in 1..=USER_CHANGES_AFTER_THE_REMOUNT {
        let published = match user_changes {
            UserChangesAfterTheRemount::OverwritesThenSequentialWritesOfEightDataUnitsFrom {
                first_sequential_write,
            } if change >= first_sequential_write => pool
                .sequential_write(8 * data_unit_payload_capacity() + 100)
                .map(|_| ()),
            UserChangesAfterTheRemount::OverwritesThenSequentialWritesOfEightDataUnitsFrom {
                ..
            }
            | UserChangesAfterTheRemount::Overwrites => pool.overwrite(OVERWRITE_BYTES).map(|_| ()),
        };
        if let Err(refused) = published {
            refusal = Some(format!("{refused:?}"));
            break;
        }
        changes_published = change;
        let image = pool.image();
        if pool.rollback_floor() > history.abandoned_root.checkpoint_txg
            && abandoned_root_still_in_the_ring(&image, history)
        {
            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring = true;
        }
        // 每次改动之后两块盘都在时池级 checker 不判违例
        images.hand_out(
            &format!("再挂之后第 {change} 次改动之后"),
            || image,
            PoolCheckerExpectation::no_invariant_violated(),
        );
    }
    ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
        root_ring_of_the_rebuilt_allocator,
        the_slot_is_self_verified_on_only_one_rebuild_pass,
        the_slot_read_bad_after_the_mount: root_ring_reads.faulted_reads()
            > faulted_reads_by_the_end_of_the_mount,
        changes_published,
        refusal,
        floor_raised_above_the_abandoned_root_while_it_was_in_the_ring,
    }
}

/// 场景：攻方 K1-1 那一形（`research/prompts/m3-batch10i-code-r1-opus-output.md`「K1-1」一节；原型 `abandoned_start(UnitAreaOf384Slots, 3, 0)`、
/// 计划 `s1-A@0/3(d0)-trig0-c-Z`）：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），再可写挂载时重建那一遍盘 0 每条不被抛弃的根住的槽报读错
/// （rr2 整环重读），被抛弃的根 C 那一槽第一遍读得出、从重读起每三次读里前两次读回全 0：并两遍时 C 记成两遍里只有一遍自证过的那一类。
/// 挂上之后覆盖写 5 次、从第 6 次起每次顺序写 8 个数据单元，共 40 次；准入抬 F 时算上限那一遍读 C 那一槽读得出，抬 F 那一遍读它两次都读坏。
/// 预期：抬 F 那一遍拿根环表记着的 C 顶上（T1 甲），按新 F 重算的影子账照隔离 C 引用、挂载那一刻被候选根豁免的单元：
/// 40 次都做成，每次之后池级 checker 都不判违例；C 还在环里时 F 就抬过了 C 的 txg（这一形的前提）。
/// 判别力：抬 F 那一遍读坏重读仍坏的这一类不拿记着的根顶上（`crates/mutations.tsv`「T1 甲 抬 F 重算影子账那一遍读坏重读仍坏的这一类不拿记着的根顶上」）。
pub fn run_the_scenario_where_an_abandoned_root_self_verified_only_on_the_rebuild_pass_read_back_as_zeros_by_the_floor_raise_keeps_its_units_isolated(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
    assert_eq!(
        changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(images,
            &history,
            DeviceIdentity(0),
            history.ring_slot_of_the_abandoned_root,
            ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingTwoReadsInEveryThreeFromTheRereadOn(
                RootRingSlotReadFault::ReadsBackAllZero,
            ),
            UserChangesAfterTheRemount::OverwritesThenSequentialWritesOfEightDataUnitsFrom {
                first_sequential_write: 6,
            },
        ),
        ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
            the_slot_read_bad_after_the_mount: true,
            changes_published: USER_CHANGES_AFTER_THE_REMOUNT,
            refusal: None,
            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: true,
        },
        "被抛弃的根两遍里只有一遍自证过、抬 F 那一遍读它读坏：拿记着的根顶上，影子账照隔离，checker 不红"
    );
}

/// 场景：攻方 K1-2 那一形：同一组历史，C 那一槽第一遍读得出、从重读起一直报读错（计划 a），挂上之后覆盖写 40 次。
/// 预期：算上限那一遍拿记着的 C 顶上，按实例表判被抛弃、不进上限（T1 甲）：F 抬得过 C 的 txg，40 次都做成，checker 每次都不判违例。
/// 判别力：算上限那一遍读它重读仍坏照拒（`crates/mutations.tsv`「T1 甲 算上限那一遍读坏重读仍坏的这一类照拒」）时
/// 抬 F 报 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`、改动被拒。
pub fn run_the_scenario_where_an_abandoned_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
    assert_eq!(
        changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
            images,
            &history,
            DeviceIdentity(0),
            history.ring_slot_of_the_abandoned_root,
            ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(
                RootRingSlotReadFault::ReadFails,
            ),
            UserChangesAfterTheRemount::Overwrites,
        ),
        ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(
                RootRingOfTheRebuiltAllocatorRead::OnTheOneReread
            ),
            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
            the_slot_read_bad_after_the_mount: true,
            changes_published: USER_CHANGES_AFTER_THE_REMOUNT,
            refusal: None,
            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: true,
        },
        "被抛弃的根两遍里只有一遍自证过、之后一直读坏：不卡上限，40 次覆盖写都做成"
    );
}

/// 场景：攻方 K1-3 那一形：同一组历史，重建那一遍盘 0 每条不被抛弃的根住的槽报读错（rr2 整环重读），盘 1 上 txg 4 那条有效根住的槽
/// （区域 1、槽 1）第一遍读得出、从重读起一直报读错：它记成两遍里只有一遍自证过、有效的那一类。挂上之后覆盖写 40 次。
/// 预期：算上限、抬 F 那一串读它读坏、重读仍坏时拿记着的那条根顶上、照它本身算（T1 甲）：40 次都做成，checker 每次都不判违例。
/// 同一组历史上别的 10 个住有效根的槽（盘 0 读坏时盘 1 上的、盘 1 读坏时盘 0 上的）
/// 10 个都做满 40 次、checker 不红。
/// 判别力：算上限那一遍读它重读仍坏照拒（`crates/mutations.tsv`「T1 甲 算上限那一遍读坏重读仍坏的这一类照拒（有效根那一槽）」）时改动被拒。
pub fn run_the_scenario_where_a_valid_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
    let slot_of_the_valid_root_of_txg_four = RootRingSlot { region: 1, slot: 1 };
    assert_eq!(
        self_verified_roots_in_the_ring(&history.image)
            .into_iter()
            .find(|(ring_slot, _)| *ring_slot == slot_of_the_valid_root_of_txg_four)
            .map(|(_, root)| root.checkpoint_txg),
        Some(CheckpointTxg(4)),
        "前提：这组历史里区域 1、槽 1 住着 txg 4 那条有效根"
    );
    assert_eq!(
        changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(images,
            &history,
            DeviceIdentity(0),
            slot_of_the_valid_root_of_txg_four,
            ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(
                RootRingSlotReadFault::ReadFails,
            ),
            UserChangesAfterTheRemount::Overwrites,
        ),
        ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
            the_slot_read_bad_after_the_mount: true,
            changes_published: USER_CHANGES_AFTER_THE_REMOUNT,
            refusal: None,
            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: true,
        },
        "有效根两遍里只有一遍自证过、之后一直读坏：照记着的那条根算上限，40 次覆盖写都做成、checker 不红"
    );
}

/// K2-2 那组历史上再挂、覆盖写几次、再正常卸载之后看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct UnmountAfterARemountWithTheAbandonedRootSelfVerifiedOnlyOnTheRebuildPass {
    /// 挂上之后 C 那一槽记成重建分配器两遍里只有一遍自证过的那一类。
    pub the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: bool,
    /// 卸载之前的覆盖写做成了几次。
    pub overwrites_published: usize,
    /// 卸载的结局（错只记成员名之前那一段）。
    pub unmounted: Result<(), String>,
    /// 卸载之后 F 高于 C 的 txg（卸载那一串把 F 抬过了 C 那一条时间线上还有效的根）。
    pub floor_above_the_abandoned_root_after_the_unmount: bool,
    /// C 还在根环里。
    pub the_abandoned_root_still_in_the_ring_after_the_unmount: bool,
    /// 卸载之后分配器里空闲的 C 引用的实例表单元那几份（盘，槽）。
    pub instance_table_slots_of_the_abandoned_root_free_after_the_unmount:
        Vec<(DeviceIdentity, SlotNumber)>,
}

/// 卸载之前覆盖写几次：抬 F 之前（攻方 K1-1 那一档准入第一次抬 F 在第 8 次改动上）。
pub const OVERWRITES_BEFORE_THE_UNMOUNT: usize = 3;

/// 场景：攻方 K1-1 末段那一形（「卸载那一串不判上限」那一句，判决 `research/prompts/m3-batch10i-code-r1-main-verification.md` 第五节第一行）：
/// K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），重建那一遍盘 0 每条不被抛弃的根住的槽报读错，C 那一槽第一遍读得出、从重读起一直报读错；
/// 挂上之后覆盖写 3 次，再正常卸载，盘留着读坏。
/// 预期：卸载那一串重算影子账时拿记着的 C 顶上（T1 甲），C 引用的实例表单元（挂载那一刻被实例 2 的有效根豁免，
/// 卸载把 F 抬过那几条根之后只剩 C 引用）照隔离：卸载做成，之后分配器里那两份都不空闲，C 还在环里，checker 不判违例。
/// 判别力：抬 F 重算影子账那一遍读坏重读仍坏的这一类不拿记着的根顶上（`crates/mutations.tsv`「T1 甲 卸载那一串读坏重读仍坏的这一类不拿记着的根顶上」）
/// 时那两份卸载之后空闲，最后一项红。
pub fn run_the_scenario_where_unmounting_after_a_remount_with_an_abandoned_root_self_verified_only_on_the_rebuild_pass_keeps_its_units_isolated(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
    let RemountKeepingTheFaultedDevices {
        mounted, devices, ..
    } = remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
        &history,
        DeviceIdentity(0),
        history.ring_slot_of_the_abandoned_root,
        ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(
            RootRingSlotReadFault::ReadFails,
        ),
    );
    let mounted = mounted.unwrap_or_else(|error| panic!("前提：再可写挂载做得成：{error:?}"));
    let the_abandoned_root_is_self_verified_on_only_one_rebuild_pass =
        slot_is_self_verified_on_only_one_rebuild_pass(
            &mounted,
            history.ring_slot_of_the_abandoned_root,
        );
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool: PoolUnderTest<DeviceFailingTheRebuildPass> = PoolUnderTest {
        parameters: history.parameters.clone(),
        device_bytes: history.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: history.write_time_seconds + 100,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut overwrites_published = 0;
    // 迭代上界是 OVERWRITES_BEFORE_THE_UNMOUNT；提前出口只有被拒。
    for _ in 0..OVERWRITES_BEFORE_THE_UNMOUNT {
        if pool.overwrite(OVERWRITE_BYTES).is_err() {
            break;
        }
        overwrites_published += 1;
    }
    let mut session_being_unmounted = pool.session.take().expect("覆盖写之后会话还在");
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session_being_unmounted.allocator,
        &mut session_being_unmounted.current,
        ShadowLedger::On,
    );
    let image = pool.image();
    let outcome = UnmountAfterARemountWithTheAbandonedRootSelfVerifiedOnlyOnTheRebuildPass {
        the_abandoned_root_is_self_verified_on_only_one_rebuild_pass,
        overwrites_published,
        unmounted: unmounted.map(|_| ()).map_err(|error| format!("{error:?}")),
        floor_above_the_abandoned_root_after_the_unmount: session_being_unmounted
            .current
            .root()
            .rollback_floor
            > history.abandoned_root.checkpoint_txg,
        the_abandoned_root_still_in_the_ring_after_the_unmount: abandoned_root_still_in_the_ring(
            &image, &history,
        ),
        instance_table_slots_of_the_abandoned_root_free_after_the_unmount: history
            .abandoned_root
            .instance_table
            .locations
            .iter()
            .filter(|location| {
                session_being_unmounted.allocator.devices
                    [usize::try_from(location.device.0).expect("盘号")]
                .is_free(location.slot)
            })
            .map(|location| (location.device, location.slot))
            .collect(),
    };
    images.hand_out(
        "卸载之后",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    assert_eq!(
        outcome,
        UnmountAfterARemountWithTheAbandonedRootSelfVerifiedOnlyOnTheRebuildPass {
            the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: true,
            overwrites_published: OVERWRITES_BEFORE_THE_UNMOUNT,
            unmounted: Ok(()),
            floor_above_the_abandoned_root_after_the_unmount: true,
            the_abandoned_root_still_in_the_ring_after_the_unmount: true,
            instance_table_slots_of_the_abandoned_root_free_after_the_unmount: Vec::new(),
        },
        "卸载那一串读 C 读坏、重读仍坏：拿记着的 C 顶上，C 引用的实例表单元照隔离"
    );
}

/// K2-2 那组历史上再挂（读坏只落在挂载里）、接着覆盖写到 C 离开根环，看分配器里 C 的落点（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct AbandonedRootWatchedInTheAllocatorUntilItLeavesTheRing {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 挂上之后 C 那一槽记成重建分配器两遍里只有一遍自证过的那一类。
    pub the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: bool,
    /// C 还在根环里的每次覆盖写之后，C 引用的实例表单元在分配器里空闲的那几份（第几次覆盖写，盘，槽）。
    pub instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring:
        Vec<(usize, DeviceIdentity, SlotNumber)>,
    /// 这一趟里环里最旧有效根越过了 C 的 txg、C 还在环里（进程内记根那一次的回收门槛越过 C，C 引用的、挂载那一刻被候选根豁免的单元被回收）。
    pub oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring: bool,
    /// C 离开根环那一次覆盖写之后，C 写出的落点在分配器里不空闲的那几个（盘，槽）：只被 C 引用、挂载时隔离的，C 离环时清掉。
    pub places_the_abandoned_root_wrote_not_free_once_it_left_the_ring:
        Vec<(DeviceIdentity, SlotNumber)>,
}

/// 场景：K2-2 那组历史（4 GiB、实例 1 覆盖写 1 次：盘宽，进程内不推抬 F），第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读只坏 C 的根槽（报读错）：
/// C 记成两遍里只有一遍自证过、被抛弃的那一类。挂上之后照常读盘，覆盖写到 C 离开根环。
/// 预期：进程内记根时这一类被抛弃根照它记着的落点算进「环里还在的被抛弃根引用着的落点」——环里最旧有效根越过 C 的 txg 那一次（C 还在环里）回收出来的
/// C 的实例表单元照补隔离，C 在环里时那两份一次都不空闲；C 离开根环那一次按它记着的落点清隔离，C 写出的落点那一刻全空闲；checker 不红。
/// 判别力：这一类被抛弃根不算进还被引用的落点（`crates/mutations.tsv`「新类被抛弃那一半不算进环里还被引用的落点」）时第三项红；
/// 被换下时不清隔离（「新类被抛弃那一半被换下时不清隔离」）时第五项红。
pub fn run_the_scenario_where_an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::FourGibibytes, 1);
    let mut faulted_reads = faulted_reads_of(
        &root_ring_slots_of_device_one(&history),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads.extend(faulted_reads_of(
        &[history.ring_slot_of_the_abandoned_root],
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
        RootRingSlotReadFault::ReadFails,
    ));
    let RemountThroughFaultedReads {
        mounted,
        image_after_the_remount,
        ..
    } = remount_the_pool_with_an_abandoned_root(&history, faulted_reads);
    let mounted = mounted.unwrap_or_else(|error| panic!("前提：再可写挂载做得成：{error:?}"));
    let root_ring_of_the_rebuilt_allocator =
        Ok(mounted.output.rereads.root_ring_of_the_rebuilt_allocator);
    let the_abandoned_root_is_self_verified_on_only_one_rebuild_pass =
        slot_is_self_verified_on_only_one_rebuild_pass(
            &mounted,
            history.ring_slot_of_the_abandoned_root,
        );
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool: PoolUnderTest<SparseBlockDevice> = PoolUnderTest {
        parameters: history.parameters.clone(),
        device_bytes: history.device_bytes,
        devices: plain_devices_on(&image_after_the_remount),
        session: Some(session),
        write_time_seconds: history.write_time_seconds + 100,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let root_ring_slot_count = every_root_ring_slot(&history.parameters.geometry).len();
    let instance_table_slots_of_the_abandoned_root_free =
        |pool_watched: &PoolUnderTest<SparseBlockDevice>| {
            history
                .abandoned_root
                .instance_table
                .locations
                .iter()
                .filter(|location| {
                    pool_watched.session().allocator.devices
                        [usize::try_from(location.device.0).expect("盘号")]
                    .is_free(location.slot)
                })
                .map(|location| (location.device, location.slot))
                .collect::<Vec<_>>()
        };
    let mut instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring = Vec::new();
    let mut oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring = false;
    let mut overwrites = 0usize;
    // 迭代上界是根环的槽数（C 的根槽在一圈之内被盖掉）；没有提前出口。
    while abandoned_root_still_in_the_ring(&pool.image(), &history) {
        assert!(
            overwrites < root_ring_slot_count,
            "C 的根槽在一圈（{root_ring_slot_count} 次发布）之内被盖掉"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("再挂之后第 {overwrites} 次覆盖写：{refusal:?}"));
        overwrites += 1;
        let image = pool.image();
        if !abandoned_root_still_in_the_ring(&image, &history) {
            break;
        }
        let oldest_valid_root_txg = self_verified_roots_in_the_ring(&image)
            .into_iter()
            .filter(|(ring_slot, _)| *ring_slot != history.ring_slot_of_the_abandoned_root)
            .map(|(_, root)| root.checkpoint_txg)
            .min();
        if oldest_valid_root_txg
            .is_some_and(|oldest| oldest > history.abandoned_root.checkpoint_txg)
        {
            oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring = true;
        }
        instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring.extend(
            instance_table_slots_of_the_abandoned_root_free(&pool)
                .into_iter()
                .map(|(device, slot)| (overwrites, device, slot)),
        );
    }
    let places_the_abandoned_root_wrote_not_free_once_it_left_the_ring = history
        .places_the_abandoned_root_wrote
        .iter()
        .filter(|(device, slot, _)| {
            !pool.session().allocator.devices[usize::try_from(device.0).expect("盘号")]
                .is_free(*slot)
        })
        .map(|(device, slot, _)| (*device, *slot))
        .collect();
    images.hand_out(
        "C 离开根环那一刻",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    assert_eq!(
        AbandonedRootWatchedInTheAllocatorUntilItLeavesTheRing {
            root_ring_of_the_rebuilt_allocator,
            the_abandoned_root_is_self_verified_on_only_one_rebuild_pass,
            instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring,
            oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring,
            places_the_abandoned_root_wrote_not_free_once_it_left_the_ring,
        },
        AbandonedRootWatchedInTheAllocatorUntilItLeavesTheRing {
            root_ring_of_the_rebuilt_allocator: Ok(
                RootRingOfTheRebuiltAllocatorRead::OnTheOneReread
            ),
            the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: true,
            instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring: Vec::new(),
            oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring: true,
            places_the_abandoned_root_wrote_not_free_once_it_left_the_ring: Vec::new(),
        },
        "两遍里只有一遍自证过的被抛弃根：在环里时它引用的单元一直扣着，离环时按它记着的落点清"
    );
}
