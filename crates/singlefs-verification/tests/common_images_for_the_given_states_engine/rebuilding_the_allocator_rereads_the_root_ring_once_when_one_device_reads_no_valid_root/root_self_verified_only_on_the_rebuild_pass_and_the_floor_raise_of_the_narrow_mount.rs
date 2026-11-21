//! 只在重建那一遍自证过的根：从重读起读不出、重读之后再读坏一次时，窄挂载的抬 F 不拒、照样定上限。

use super::*;

/// 一份镜像上盘 `device` 那几个区域里最新那条自证过的根住的根环槽。
pub fn ring_slot_of_the_newest_root_on(image: &MemoryPool, device: DeviceIdentity) -> RootRingSlot {
    let region_devices = choose_system_configuration(image)
        .expect("系统配置")
        .immutable
        .region_devices;
    self_verified_roots_in_the_ring(image)
        .into_iter()
        .filter(|(ring_slot, _)| {
            region_devices[usize::try_from(ring_slot.region).expect("区域号")] == device
        })
        .max_by_key(|(_, root)| (root.checkpoint_txg, root.instance))
        .map(|(ring_slot, _)| ring_slot)
        .expect("这块盘上有一条自证过的根")
}

/// 那一槽在重建那一遍（第一遍）之后怎么读坏。
#[derive(Clone, Copy, Debug)]
pub enum ReadsOfTheSlotFailingAfterTheRebuildPass {
    /// 从重读那一遍起每一次都报读错（只在「从重读起一直坏」上出现）。
    FromTheRereadOnward,
    /// 重读那一遍报读错；可写挂载头一次把它读进预演与真做共用的那一份那一次（[`READ_OF_THE_SLOT_INTO_THE_READS_SHARED_IN_THIS_MOUNT`]）
    /// 读成功、读回乱字节（记下的是坏字节），别的读得出：写行与暖机之后推的抬 F 算上限那一遍取记下的那一份读坏，当场重读穿到盘上读得出
    /// （[`READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE`]）。
    OnTheRereadAndBackAsGarbageOnItsFirstReadIntoTheReadsSharedInThisMount,
    /// 只有重读那一遍报读错（对照）。
    OnlyOnTheReread,
    /// 重建两遍都读得出（这个进程知道那里住着一条根）；只有可写挂载头一次把它读进预演与真做共用的那一份那一次
    /// （[`READ_OF_THE_SLOT_INTO_THE_READS_SHARED_IN_THIS_MOUNT`]）读成功、读回乱字节（记下的是坏字节）。
    BackAsGarbageOnlyOnItsFirstReadIntoTheReadsSharedInThisMount,
    /// 重建那一遍之后一次都不读坏（对照）。
    NeverAfterTheRebuildPass,
}

/// `narrow-384-20-2` 上照 [`ReadsOfTheSlotFailingAfterTheRebuildPass`] 再挂时，盘 0 最新那条根住的槽第几次读到盘上是可写挂载头一次把它读进
/// 预演与真做共用的那一份（`mounted_session::DeviceReadOnceInThisMount`；岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一
/// 「读一次两边共用」）：1–4 读阶段与它之后取第一个 txg、取水位、算 F 生效值那几读，5、6 重建分配器两遍，7 是可写挂载这一段里头一次读根环（之后写行、暖机、抬 F 读它都取记下的那一份）。
pub const READ_OF_THE_SLOT_INTO_THE_READS_SHARED_IN_THIS_MOUNT: u64 =
    READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD + 1;

/// 同上，那一槽第几次读到盘上是写行与暖机之后推的第一次抬 F 算上限那一遍（`mount::roots_of_the_ring_for_the_ceiling`）读坏之后的当场重读：
/// 8 是真做取号写之前那一遍（条款要那一刻再读一次盘，照旧读盘）；之后读它都取记下的那一份，抬 F 算上限那一遍读到记下的坏字节、
/// 当场重读那一读绕过记下的那一份穿到盘上（主 agent 判定：抬 F 与续做里「读坏当场重读」那一读不许被共用的那一份吃掉），是第 9 次。
/// 重读读到自证过的换掉记下的那一份，之后再算一次上限、重算影子账那几遍取它，不再到盘上。读到盘上的次序变了这个数跟着变：
/// 瞬时那条用例另断言这一次读前后都紧挨着一段写、这一槽上一次读到盘上在这次挂载第一个写之前
/// （[`read_is_the_floor_raises_reread_alone_on_the_disk_between_two_writes`]），读序一变那一项就红。
pub const READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE: u64 =
    READ_OF_THE_SLOT_INTO_THE_READS_SHARED_IN_THIS_MOUNT + 2;

/// S1 那一形再挂之后看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 重读之前的钩子调了几次，每次调的那一刻每个根环槽是不是都恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次。
    pub hook_calls_after_exactly_the_rebuild_pass: Vec<bool>,
    /// 这次挂载写行与暖机之后推了抬 F（取号之前准入不够）。
    pub floor_raises_pushed_by_the_mount: bool,
    /// 那一槽在重读那一遍之后还被读过（抬 F 那一遍读了它）。
    pub the_slot_was_read_after_the_reread: bool,
    /// 交回的盘面与「那一槽只在重读那一遍报读错」的再挂交回的逐字节相同。
    pub same_disk_as_a_remount_failing_the_slot_only_on_the_reread: bool,
    /// 丢盘 0、丢盘 1 之后剩下那块盘只读挂载、整读每个活文件，与挂载之前两块盘都在时读出的相同（挂载之前最新那条根只落在盘 0 上，
    /// 那时丢盘 0 读出的是更旧的一版，拿它比不出这次挂载丢没丢东西）。
    pub lone_survivor_reads_what_both_devices_read_before_the_remount: [bool; 2],
    /// 丢盘 0、丢盘 1 之后，挂载之前环里还在、不低于 F 的每条自证根在剩下那块盘上整读与挂载之前相同。
    pub lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [bool; 2],
    /// 那一槽第 [`READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE`] 次读到盘上的是抬 F 那一段写之前唯一一次读到盘上的根环槽读，
    /// 这一槽上一次读到盘上在这次挂载第一个写之前（[`read_is_the_floor_raises_reread_alone_on_the_disk_between_two_writes`]）。
    pub the_named_read_is_the_floor_raises_reread_alone_on_the_disk_between_two_writes: bool,
    /// 挂载结束时那一槽一共读到盘上几次。
    pub reads_of_the_slot_on_the_disk_by_the_end_of_the_remount: u64,
}

/// 读写次序里（`ring_slot`，第 `read` 次读到盘上）那一次读是不是：① 紧挨在一段写之后（那段写之后头一次读到盘上的根环槽读）；
/// ② 紧挨着它的是下一段写（两段写之间读到盘上的根环槽读只有它这一次）；③ 这一槽上一次读到盘上（第 `read − 1` 次）在这次挂载第一个写之前。
/// 读一次两边共用之后，这次挂载写之后读根环都取预演与真做共用的那一份、不到盘上，只有读坏之后当场重读那一读穿到盘上
/// （主 agent 判定：抬 F 与续做里「读坏当场重读」那一读不许被共用的那一份吃掉）。暖机之后推的抬 F 头一遍读根环是准入算上限那一遍：
/// 它取到记下的坏字节、当场重读（①；③：写行、暖机算生效 F 那几遍读它取的是记下的那一份，没到盘上）；重读读到自证过的换掉记下的那一份，
/// 之后再算一次上限、重算影子账那几遍取它、不再到盘上（②）。别的位置对不上：真做取号写之前那一遍照旧读盘，前面没有写（① 不成立）；
/// 重读读到的不换掉记下的那一份时，再算一次上限那一遍又读坏、又重读一次（② 不成立）。
pub fn read_is_the_floor_raises_reread_alone_on_the_disk_between_two_writes(
    reads_and_writes_in_order: &[Option<(RootRingSlot, u64)>],
    ring_slot: RootRingSlot,
    read: u64,
) -> bool {
    let position_of = |read_of_the_slot: u64| {
        reads_and_writes_in_order
            .iter()
            .position(|event| *event == Some((ring_slot, read_of_the_slot)))
    };
    let Some(position) = position_of(read) else {
        return false;
    };
    let right_after_a_write = position > 0 && reads_and_writes_in_order[position - 1].is_none();
    let right_before_a_write = reads_and_writes_in_order.get(position + 1) == Some(&None);
    let first_write = reads_and_writes_in_order.iter().position(Option::is_none);
    let previous_read_before_the_first_write =
        match (read.checked_sub(1).and_then(position_of), first_write) {
            (Some(previous), Some(first_write)) => previous < first_write,
            (None, _) | (Some(_), None) => false,
        };
    right_after_a_write && right_before_a_write && previous_read_before_the_first_write
}

/// `narrow-384-20-2` 上照一份读坏计划再挂一次交回的：挂载的结局、挂载之后的盘面、钩子每次调的那一刻每个根环槽读过几次、
/// 挂载结束时每个根环槽读过几次与读写次序。
pub struct NarrowRemountThroughFaultedReads {
    pub mounted: Result<Mounted, MountError>,
    pub image_after_the_remount: MemoryPool,
    pub reads_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>>,
    pub reads_of_each_slot_by_the_end_of_the_remount: BTreeMap<RootRingSlot, u64>,
    pub reads_and_writes_in_order: Vec<Option<(RootRingSlot, u64)>>,
}

/// `narrow-384-20-2` 上再可写挂载一次（准入判着、影子账开）：重建那一遍盘 1 每个根环槽报读错（盘 1 一条有效根都没读出，rr2 整环重读），
/// 盘 0 上的 `slot` 第一遍读得出、之后照 `reads_of_the_slot` 读坏；按槽并两遍时那一槽只在第一遍自证过。
pub fn remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
    before: &PoolBeforeTheRemount,
    slot: RootRingSlot,
    reads_of_the_slot: ReadsOfTheSlotFailingAfterTheRebuildPass,
) -> NarrowRemountThroughFaultedReads {
    let mut faulted_reads: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
        faulted_slots_of_the_plan(&before.image_to_remount, DEVICE_ONE_REPORTS_READ_ERRORS)
            .into_iter()
            .map(|(ring_slot, fault)| {
                (
                    (
                        ring_slot,
                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
                    ),
                    fault,
                )
            })
            .collect();
    let mut slots_failing_from_a_read_onward = BTreeMap::new();
    match reads_of_the_slot {
        ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward => {
            slots_failing_from_a_read_onward.insert(
                slot,
                (
                    READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
                    RootRingSlotReadFault::ReadFails,
                ),
            );
        }
        ReadsOfTheSlotFailingAfterTheRebuildPass::OnTheRereadAndBackAsGarbageOnItsFirstReadIntoTheReadsSharedInThisMount => {
            faulted_reads.extend(faulted_reads_of(
                &[slot],
                READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
                RootRingSlotReadFault::ReadFails,
            ));
            faulted_reads.extend(faulted_reads_of(
                &[slot],
                READ_OF_THE_SLOT_INTO_THE_READS_SHARED_IN_THIS_MOUNT,
                RootRingSlotReadFault::ReadsBackGarbage,
            ));
        }
        ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread => {
            faulted_reads.extend(faulted_reads_of(
                &[slot],
                READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
                RootRingSlotReadFault::ReadFails,
            ));
        }
        ReadsOfTheSlotFailingAfterTheRebuildPass::BackAsGarbageOnlyOnItsFirstReadIntoTheReadsSharedInThisMount => {
            faulted_reads.extend(faulted_reads_of(
                &[slot],
                READ_OF_THE_SLOT_INTO_THE_READS_SHARED_IN_THIS_MOUNT,
                RootRingSlotReadFault::ReadsBackGarbage,
            ));
        }
        ReadsOfTheSlotFailingAfterTheRebuildPass::NeverAfterTheRebuildPass => {}
    }
    let root_ring_reads = root_ring_reads_failing_also_from_a_read_onward(
        &before.image_to_remount,
        faulted_reads,
        slots_failing_from_a_read_onward,
    );
    let (mounted, image_after_the_remount, reads_at_each_hook_call) =
        remount_through(before, &root_ring_reads);
    NarrowRemountThroughFaultedReads {
        mounted,
        image_after_the_remount,
        reads_at_each_hook_call,
        reads_of_each_slot_by_the_end_of_the_remount: root_ring_reads.reads_of_each_slot(),
        reads_and_writes_in_order: root_ring_reads.reads_and_writes_in_order(),
    }
}

/// `narrow-384-20-2` 上再可写挂载（准入判着、影子账开）：重建那一遍盘 1 每个根环槽报读错（盘 1 一条有效根都没读出，rr2 整环重读），
/// 盘 0 最新那条根住的槽第一遍读得出、之后照 `reads_of_the_slot` 读坏；按槽并两遍时那一槽只在第一遍自证过。
pub fn remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(
    images: &mut PoolCheckerImages,
    reads_of_the_slot: ReadsOfTheSlotFailingAfterTheRebuildPass,
) -> RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
    let before = pool_before_the_remount(
        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
    );
    let slot_of_the_newest_root_on_device_zero =
        ring_slot_of_the_newest_root_on(&before.image_to_remount, DeviceIdentity(0));
    let remount_with =
        |reads_of_the_slot_in_this_remount: ReadsOfTheSlotFailingAfterTheRebuildPass| {
            let NarrowRemountThroughFaultedReads {
                mounted,
                image_after_the_remount,
                reads_at_each_hook_call,
                reads_of_each_slot_by_the_end_of_the_remount,
                reads_and_writes_in_order,
            } = remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
                &before,
                slot_of_the_newest_root_on_device_zero,
                reads_of_the_slot_in_this_remount,
            );
            (
                mounted,
                image_after_the_remount,
                reads_at_each_hook_call,
                reads_of_each_slot_by_the_end_of_the_remount,
                reads_and_writes_in_order,
            )
        };
    let (
        mounted,
        image_after_the_remount,
        reads_at_each_hook_call,
        reads_of_each_slot_by_the_end_of_the_remount,
        reads_and_writes_in_order,
    ) = remount_with(reads_of_the_slot);
    let (mounted_for_reference, image_after_the_reference_remount, _, _, _) =
        remount_with(ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread);
    mounted_for_reference
        .unwrap_or_else(|error| panic!("前提：那一槽只在重读那一遍报读错的再挂做得成：{error:?}"));
    // 两块盘都在时池级 checker 不判违例
    images.hand_out(
        "盘 0 最新那条根两遍里只有一遍自证过的再挂之后",
        || image_after_the_remount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
        hook_calls_after_exactly_the_rebuild_pass: reads_at_each_hook_call
            .iter()
            .map(|reads_of_each_slot| {
                !reads_of_each_slot.is_empty()
                    && reads_of_each_slot
                        .values()
                        .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)
            })
            .collect(),
        floor_raises_pushed_by_the_mount: mounted
            .as_ref()
            .is_ok_and(|mounted| !mounted.output.space_admission.floor_raises().is_empty()),
        root_ring_of_the_rebuilt_allocator: mounted
            .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
            .map_err(|error| format!("{error:?}")),
        the_slot_was_read_after_the_reread: reads_of_each_slot_by_the_end_of_the_remount
            .get(&slot_of_the_newest_root_on_device_zero)
            .is_some_and(|reads| *reads > READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD),
        same_disk_as_a_remount_failing_the_slot_only_on_the_reread: image_after_the_remount
            == image_after_the_reference_remount,
        lone_survivor_reads_what_both_devices_read_before_the_remount: [
            DeviceIdentity(0),
            DeviceIdentity(1),
        ]
        .map(|lost| {
            live_files_read_whole(&image_without(&image_after_the_remount, lost))
                == live_files_read_whole(&before.image_to_remount)
        }),
        lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [
            DeviceIdentity(0),
            DeviceIdentity(1),
        ]
        .map(|lost| {
            let (_, _, read_differently) = roots_still_in_the_ring_read_on_a_lone_survivor(
                &before.image_to_remount,
                &image_after_the_remount,
                lost,
            );
            read_differently.is_empty()
        }),
        the_named_read_is_the_floor_raises_reread_alone_on_the_disk_between_two_writes:
            read_is_the_floor_raises_reread_alone_on_the_disk_between_two_writes(
                &reads_and_writes_in_order,
                slot_of_the_newest_root_on_device_zero,
                READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
            ),
        reads_of_the_slot_on_the_disk_by_the_end_of_the_remount:
            reads_of_each_slot_by_the_end_of_the_remount
                .get(&slot_of_the_newest_root_on_device_zero)
                .copied()
                .unwrap_or(0),
    }
}

/// 场景：`narrow-384-20-2`，重建那一遍盘 1 每个根环槽报读错（rr2 整环重读），盘 0 最新那条根住的槽第一遍读得出、从重读那一遍起每次都报读错
/// ：按槽并两遍，那一槽并成第一遍读出的根，根环表另记「两遍里只有一遍自证过」那一类。
/// 预期：照常挂上（重读过一次），写行与暖机之后推的抬 F 读那一槽仍坏、重读一次仍坏，照挂载那一刻读坏处置、不拒（S1）；
/// 两块盘都在时池级 checker 不判违例，丢任一块盘剩下那块盘整读与挂载之前两块盘读出的相同，挂载之前环里还在、不低于 F 的根丢盘整读也相同；
/// 交回的盘面与「那一槽只在重读那一遍报读错」的再挂相同（这一形里抬 F 抬到的上限不受那一槽封顶，回收门槛按新 F 取、钉住的最旧 txg 在它之下）。
/// 判别力：改回记成「这个进程知道住着根的槽」（`crates/mutations.tsv`「S1 两遍里只有一遍自证过的根改回记成知道住着根的槽」）时
/// 抬 F 重读那一槽仍坏就拒，挂载在写行、暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
pub fn run_the_scenario_where_a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount(
    images: &mut PoolCheckerImages,
) {
    let remount = remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(images,
        ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward,
    );
    assert_eq!(
        (
            &remount.root_ring_of_the_rebuilt_allocator,
            &remount.hook_calls_after_exactly_the_rebuild_pass,
            remount.floor_raises_pushed_by_the_mount,
            remount.the_slot_was_read_after_the_reread,
            remount.same_disk_as_a_remount_failing_the_slot_only_on_the_reread,
            remount.lone_survivor_reads_what_both_devices_read_before_the_remount,
            remount.lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount,
        ),
        (
            &Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            &vec![true],
            true,
            true,
            true,
            [true, true],
            [true, true],
        ),
        "两遍里只有一遍自证过的根、从重读起一直读坏：照常挂上，抬 F 读它仍坏不拒，checker 不红、丢盘整读不变（{remount:?}）"
    );
}

/// 场景：同上，那一槽只在重读那一遍报读错、在可写挂载头一次把它读进预演与真做共用的那一份那一次读回乱字节（读成功、自证不过，记下的是坏字节），
/// 之后读得出：写行、暖机之后推的抬 F 算上限那一遍取到记下的坏字节（这一遍读坏），当场重读穿到盘上（盘上是好的）。
/// 预期：抬 F 那一遍读它读坏就重读一次、重读读出有效根就照读出来的算，与「那一槽只在重读那一遍报读错」的再挂同一份盘面；
/// checker 不红，丢盘整读不变；那一槽第 [`READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE`] 次读到盘上的是抬 F 那一段写之前
/// 唯一一次读到盘上的根环槽读、它上一次读到盘上在第一个写之前，挂载结束时一共读到盘上这么多次（K3-e：只靠常量时读序一变照样绿；
/// 重读读到自证过的换掉记下的那一份，之后再算一次上限、重算影子账不再读坏、不再到盘上）。
/// 判别力：抬 F 算上限那一遍读它读坏一次就拒、不重读（`crates/mutations.tsv`「S1 抬 F 算上限读它读坏一次就拒、不重读」）时
/// 挂载在写行、暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`，第一项红；重读也取记下的那一份（`crates/mutations.tsv`
/// 「4-4 修红 W16 读坏当场重读：重读也取共用的那一份」）时那一槽在抬 F 那一段一次都不到盘上，点名的那一项红；读到盘上的次序变了
/// （「4-4 修红 W16 读序：真做取号写之前多读一遍根环」）时点名的那一次成了取号写之前那一遍，点名的那一项与读到盘上的次数红。
/// 抬 F 算上限那一遍与重算影子账那一遍（`mount::readable_roots_for_raising_the_floor` 里两遍里只有一遍自证过的槽那一段）都读坏当场重读，
/// 哪一遍先重读、换掉记下的那一份，从盘上看不出来：只改一处的变异这条用例照绿，「知道住着根的槽」那一类另有
/// `writable_mount_reread_after_a_bad_read_reaches_the_disk_past_the_reads_shared_with_the_rehearsal.rs` 那条用例。
pub fn run_the_scenario_where_a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(images,
            ReadsOfTheSlotFailingAfterTheRebuildPass::OnTheRereadAndBackAsGarbageOnItsFirstReadIntoTheReadsSharedInThisMount,
        ),
        RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            hook_calls_after_exactly_the_rebuild_pass: vec![true],
            floor_raises_pushed_by_the_mount: true,
            the_slot_was_read_after_the_reread: true,
            same_disk_as_a_remount_failing_the_slot_only_on_the_reread: true,
            lone_survivor_reads_what_both_devices_read_before_the_remount: [true, true],
            lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [true, true],
            the_named_read_is_the_floor_raises_reread_alone_on_the_disk_between_two_writes: true,
            reads_of_the_slot_on_the_disk_by_the_end_of_the_remount:
                READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
        },
        "两遍里只有一遍自证过的根、之后读进共用的那一份时读坏一次：抬 F 当场重读穿到盘上，读出来就照读出来的算"
    );
}

/// `narrow-384-20-2` 上第 4 新的那条根住的槽两遍里只有一遍自证过、之后一直读坏时再挂看到的那几样（用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct RemountWithTheFourthNewestRootSelfVerifiedOnlyOnTheRebuildPass {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 挂上之后那一槽记成重建分配器两遍里只有一遍自证过的那一类。
    pub the_slot_is_self_verified_on_only_one_rebuild_pass: bool,
    /// 那一槽在重读那一遍之后还被读过（挂载推的抬 F 读了它，读坏）。
    pub the_slot_was_read_after_the_reread: bool,
    /// 这次挂载写行与暖机之后推的每一串抬 F 的上限，按推的先后。
    pub ceilings_of_the_floor_raises_pushed_by_the_mount: Vec<CheckpointTxg>,
    /// 挂载之后现行那一版带的 F。
    pub rollback_floor_after_the_mount: Option<CheckpointTxg>,
    /// 交回的盘面与「那一槽只在重读那一遍报读错」（抬 F 读得出它）的再挂交回的逐字节相同。
    pub same_disk_as_a_remount_failing_the_slot_only_on_the_reread: bool,
}

/// 场景：`narrow-384-20-2`（再挂写行、暖机之后推抬 F），重建那一遍盘 1 每个根环槽报读错（rr2 整环重读），环里第 4 新的那条根（txg 21，
/// 住盘 0）住的槽第一遍读得出、从重读那一遍起每次都报读错：它记成两遍里只有一遍自证过、有效的那一类。环里最新的 5 条根各带一版不同的
/// 文件内容（前提里现核），挂载自己写的根不改文件，所以这条根就是抬 F 那一刻「第 4 新的非空持久有效根」。
/// 预期：挂载推的抬 F 算上限那一遍读它仍坏、重读一次仍坏，拿根环表记着的那条根顶上、照它本身算（T1 甲；
/// D16（发布语义） 已定项 1「抬 F 的上限」：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)）：上限恰是这条根的 txg，F 抬到它；
/// 与「那一槽只在重读那一遍报读错」的再挂同一份盘面；两块盘都在时池级 checker 不判违例。
/// 判别力：算上限那一遍读坏重读仍坏的有效根当没有根（`crates/mutations.tsv`「T1 甲 算上限那一遍读坏重读仍坏的有效根当没有根」）时
/// 第 4 新的换成 txg 20，上限与 F 都落到 20、盘面跟着不同。K1-3 那条根（txg 4）最旧、上一条 S1 用例那条（txg 24）最新，都罩不住这一处：
/// 少了 txg 24，挂载写的第一条根就与 txg 23 比成非空、顶上它的位置（推的；在副本上逐槽量过这份盘面：
/// 盘 1 读坏时盘 0 上只有 txg 21 这一槽、盘 0 读坏时盘 1 上只有 txg 22 那一槽，在这一变异下上限变）。
pub fn run_the_scenario_where_the_fourth_newest_non_empty_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_still_sets_the_ceiling_of_the_narrow_mounts_floor_raise(
    images: &mut PoolCheckerImages,
) {
    let before = pool_before_the_remount(
        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
    );
    let region_devices = choose_system_configuration(&before.image_to_remount)
        .expect("系统配置")
        .immutable
        .region_devices;
    let mut roots_newest_first = self_verified_roots_in_the_ring(&before.image_to_remount);
    roots_newest_first
        .sort_by_key(|(_, root)| std::cmp::Reverse((root.checkpoint_txg, root.instance)));
    let distinct_contents_of_the_five_newest_roots: std::collections::BTreeSet<
        Vec<(u64, Vec<u8>)>,
    > = roots_newest_first
        .iter()
        .take(5)
        .map(|(_, root)| {
            live_files_read_whole_at(&before.image_to_remount, root)
                .unwrap_or_else(|failure| panic!("前提：最新的 5 条根整读得出：{failure}"))
        })
        .collect();
    let (slot_of_the_fourth_newest_root, fourth_newest_root) = roots_newest_first[3];
    assert_eq!(
        (
            fourth_newest_root.checkpoint_txg,
            region_devices[usize::try_from(slot_of_the_fourth_newest_root.region).expect("区域号")],
            distinct_contents_of_the_five_newest_roots.len(),
        ),
        (CheckpointTxg(21), DeviceIdentity(0), 5),
        "前提：第 4 新的根是 txg 21、住盘 0，最新的 5 条根各带一版不同的文件内容"
    );
    let reference =
        remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
            &before,
            slot_of_the_fourth_newest_root,
            ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread,
        );
    let mounted_for_reference = reference
        .mounted
        .unwrap_or_else(|error| panic!("前提：那一槽只在重读那一遍报读错的再挂做得成：{error:?}"));
    assert_eq!(
        mounted_for_reference
            .output
            .space_admission
            .floor_raises()
            .iter()
            .map(|raised| raised.ceiling)
            .collect::<Vec<_>>(),
        vec![fourth_newest_root.checkpoint_txg],
        "前提：抬 F 读得出那一槽的对照再挂只推一串抬 F，上限恰是第 4 新的那条根"
    );
    let remount =
        remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
            &before,
            slot_of_the_fourth_newest_root,
            ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward,
        );
    // 两块盘都在时池级 checker 不判违例
    images.hand_out(
        "第 4 新的那条根从重读起一直读坏的再挂之后",
        || remount.image_after_the_remount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let mounted = remount.mounted.as_ref();
    assert_eq!(
        RemountWithTheFourthNewestRootSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: mounted
                .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
                .map_err(|error| format!("{error:?}")),
            the_slot_is_self_verified_on_only_one_rebuild_pass: mounted.is_ok_and(|mounted| {
                slot_is_self_verified_on_only_one_rebuild_pass(mounted, slot_of_the_fourth_newest_root)
            }),
            the_slot_was_read_after_the_reread: remount
                .reads_of_each_slot_by_the_end_of_the_remount
                .get(&slot_of_the_fourth_newest_root)
                .is_some_and(|reads| *reads > READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD),
            ceilings_of_the_floor_raises_pushed_by_the_mount: mounted.map_or_else(
                |_| Vec::new(),
                |mounted| {
                    mounted
                        .output
                        .space_admission
                        .floor_raises()
                        .iter()
                        .map(|raised| raised.ceiling)
                        .collect()
                }
            ),
            rollback_floor_after_the_mount: mounted
                .ok()
                .map(|mounted| mounted.current.root().rollback_floor),
            same_disk_as_a_remount_failing_the_slot_only_on_the_reread: remount
                .image_after_the_remount
                == reference.image_after_the_remount,
        },
        RemountWithTheFourthNewestRootSelfVerifiedOnlyOnTheRebuildPass {
            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
            the_slot_was_read_after_the_reread: true,
            ceilings_of_the_floor_raises_pushed_by_the_mount: vec![fourth_newest_root.checkpoint_txg],
            rollback_floor_after_the_mount: Some(fourth_newest_root.checkpoint_txg),
            same_disk_as_a_remount_failing_the_slot_only_on_the_reread: true,
        },
        "第 4 新的非空有效根两遍里只有一遍自证过、之后一直读坏：抬 F 拿记着的那条根顶上，上限与 F 恰是它的 txg"
    );
}
