//! 被抛弃根的根槽只在重读那一遍读坏、读回全零、读回乱字节，或重建那一遍读坏而重读读得出：它的单元一直隔离到它离开根环；整环重读失败时照重建那一遍读出的根挂上。

use super::*;

/// K2-2 那组历史停在进程退出那一刻：盘面、参数，被抛弃的根 C、它住的根环槽，与 C 写出的每个落点（盘、槽、那时盘上的字节）。
pub struct PoolWithAnAbandonedRootAndThePlacesItWrote {
    pub image: MemoryPool,
    pub parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    pub device_bytes: u64,
    pub write_time_seconds: u64,
    pub abandoned_root: RootRecord,
    pub ring_slot_of_the_abandoned_root: RootRingSlot,
    pub places_the_abandoned_root_wrote: Vec<(DeviceIdentity, SlotNumber, Vec<u8>)>,
}

pub fn bytes_on(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length: usize,
) -> Vec<u8> {
    let (_, block_device) = devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    let mut bytes = vec![0u8; length];
    block_device
        .read_at(offset, &mut bytes)
        .expect("内存盘读得出");
    bytes
}

pub fn write_on(
    devices: &mut [(DeviceIdentity, SparseBlockDevice)],
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    bytes: &[u8],
) {
    let (_, block_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    block_device
        .write_at(offset, bytes, WriteDurability::Plain)
        .expect("内存盘写得进");
}

/// 两块盘（盘宽 `device_width`：K2-2 那组历史两档，4 GiB 与单元区 384 槽）：第一个文件、覆盖写 `overwrites_before_the_abandoned_root` 次
/// （实例 1；K2-2 两档，1 次与 3 次）；崩了再挂（实例 2），覆盖写一次得 C；照 `tests/common` 的
/// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 那一形抛弃 C——C 的根槽与数据单元暂存、清零，见证 C 的那一次系统配置轮换
/// 写回成轮换之前的那一代，崩了再挂（实例 3）择根落到 C 之前那一条、写行与暖机，再把暂存的字节写回；进程退出。
pub fn pool_with_an_abandoned_root(
    device_width: HistoryDeviceWidth,
    overwrites_before_the_abandoned_root: usize,
) -> PoolWithAnAbandonedRootAndThePlacesItWrote {
    let mut pool = start_plain(device_width);
    // 迭代上界是 overwrites_before_the_abandoned_root；没有提前出口。
    for overwrite_index in 0..overwrites_before_the_abandoned_root {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("实例 1 第 {overwrite_index} 次覆盖写：{refusal:?}"));
    }
    pool.crash_and_mount_writable().expect("崩了再挂：取号 2");
    pool.overwrite(OVERWRITE_BYTES - 499)
        .expect("实例 2 覆盖写一次得 C");
    let abandoned = pool.current_file_version().clone();
    let geometry = pool.parameters.geometry;
    let ring_slot_of_the_abandoned_root = target_for_publish(
        abandoned.root.checkpoint_txg,
        geometry.root_ring_slots_per_region,
    );
    let root_slot_device = pool.parameters.region_devices
        [usize::try_from(ring_slot_of_the_abandoned_root.region).expect("区域号")];
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    let places_to_hide: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> = std::iter::once((
        root_slot_device,
        slot_offset(
            ring_slot_of_the_abandoned_root,
            geometry.fixed_structure_slot_spacing,
        ),
        usize::try_from(geometry.physical_block_size).expect("根槽宽"),
    ))
    .chain(
        abandoned
            .data_pointers
            .iter()
            .flat_map(|pointer| pointer.locations)
            .map(|location| {
                (
                    location.device,
                    location.slot.to_device_offset(),
                    data_unit_bytes,
                )
            }),
    )
    .collect();
    let saved: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = places_to_hide
        .iter()
        .map(|(device, offset, length)| {
            let bytes = bytes_on(&pool.devices, *device, *offset, *length);
            write_on(&mut pool.devices, *device, *offset, &vec![0u8; *length]);
            (*device, *offset, bytes)
        })
        .collect();
    // 见证 C 的那一次轮换（每块盘世代号最大的那一槽）写回成轮换之前的那一代：另一槽的内容、世代号减 2（与 `tests/common` 那一形同一个造法）。
    let slot_spacing_in_bytes = u64::from(geometry.fixed_structure_slot_spacing);
    let rotations_not_written: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = pool
        .devices
        .iter()
        .map(|(identity, _)| {
            let mut verified_slots = verified_system_configuration_slots(
                pool.devices.as_slice(),
                *identity,
                slot_spacing_in_bytes,
                &pool.parameters.filesystem_identifier,
            );
            verified_slots.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest] =
                <[_; 2]>::try_from(verified_slots).expect("造被抛弃的根之前每块盘两槽都自证得过");
            let newest_generation = newest.quantities.slot_generation;
            let mut rotation_not_written = older;
            rotation_not_written.quantities.slot_generation = newest_generation
                .checked_sub(2)
                .expect("C 之前至少还有两次轮换");
            (
                *identity,
                DeviceOffsetInBytes(
                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing_in_bytes,
                ),
                rotation_not_written.to_slot(),
            )
        })
        .collect();
    // 迭代上界是两块盘；没有提前出口。
    for (device, offset, bytes) in &rotations_not_written {
        write_on(&mut pool.devices, *device, *offset, bytes);
    }
    let abandoning = pool
        .crash_and_mount_writable()
        .expect("C 的根槽与数据单元读不出：择根落到 C 之前那一条，照常可写挂载（实例 3）");
    assert!(
        abandoning.effective_root.checkpoint_txg < abandoned.root.checkpoint_txg,
        "前提：实例 3 落到 C 之前"
    );
    // 迭代上界是暂存的落点数；没有提前出口。
    for (device, offset, bytes) in &saved {
        write_on(&mut pool.devices, *device, *offset, bytes);
    }
    pool.session = None;
    let mut places_the_abandoned_root_wrote: Vec<(DeviceIdentity, SlotNumber, Vec<u8>)> =
        Vec::new();
    // 迭代上界是 C 写出的单元数 × 两块盘；没有提前出口。
    for unit in &abandoned.units {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let on_disk = bytes_on(
                &pool.devices,
                device,
                unit.slot.to_device_offset(),
                unit.bytes.len(),
            );
            if on_disk[..] == unit.bytes[..] {
                places_the_abandoned_root_wrote.push((device, unit.slot, on_disk));
            }
        }
    }
    // 迭代上界是 C 的数据指针数 × 两个位置；没有提前出口。
    for location in abandoned
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
    {
        if !places_the_abandoned_root_wrote
            .iter()
            .any(|(device, slot, _)| *device == location.device && *slot == location.slot)
        {
            let on_disk = bytes_on(
                &pool.devices,
                location.device,
                location.slot.to_device_offset(),
                data_unit_bytes,
            );
            places_the_abandoned_root_wrote.push((location.device, location.slot, on_disk));
        }
    }
    PoolWithAnAbandonedRootAndThePlacesItWrote {
        image: pool.image(),
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        write_time_seconds: pool.write_time_seconds,
        abandoned_root: abandoned.root,
        ring_slot_of_the_abandoned_root,
        places_the_abandoned_root_wrote,
    }
}

/// K2-2 那组历史上再可写挂载一次、之后接着覆盖写到 C 离开根环，看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct RemountOfThePoolWithAnAbandonedRoot {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 重读之前的钩子调了几次，每次调的那一刻每个根环槽是不是都恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次。
    pub hook_calls_after_exactly_the_rebuild_pass: Vec<bool>,
    /// 读坏的恰是点名的那几次、各一次。
    pub faulted_reads_are_the_named_ones: bool,
    /// 交回的盘面与同一份盘面上照对照的读坏计划再挂交回的逐字节相同。
    pub same_disk_as_the_reference_remount: bool,
    /// 挂上那一刻两块盘都隔离了只被被抛弃根引用的槽（影子账不为 0）。
    pub both_devices_isolate_slots_after_the_mount: bool,
    /// 挂上那一刻 C 写出的落点没有一个在分配器里是空闲的。
    pub no_place_of_the_abandoned_root_is_free_after_the_mount: bool,
    /// 接着覆盖写到 C 的根槽被盖掉（C 离开根环）为止，每次发布之后 C 写出的每个落点在盘上仍是 C 写的字节。
    pub places_of_the_abandoned_root_kept_until_it_leaves_the_ring: bool,
}

/// 照一份读坏计划再挂一次交回的：挂载的结局、挂载之后的盘面、钩子每次调的那一刻每个根环槽读过几次、这一次读坏了几次。
pub struct RemountThroughFaultedReads {
    pub mounted: Result<Mounted, MountError>,
    pub image_after_the_remount: MemoryPool,
    pub reads_of_each_slot_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>>,
    pub faulted_reads: u64,
}

/// 照 `faulted_reads_of_each_slot` 读坏地再可写挂载一次（准入判着、影子账开），重读之前调钩子记下那一刻每个根环槽读过几次。
pub fn remount_the_pool_with_an_abandoned_root(
    history: &PoolWithAnAbandonedRootAndThePlacesItWrote,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
) -> RemountThroughFaultedReads {
    let root_ring_reads = root_ring_reads_failing(&history.image, faulted_reads_of_each_slot);
    let before = PoolBeforeTheRemount {
        image_before_the_crash: history.image.clone(),
        image_to_remount: history.image.clone(),
        parameters: history.parameters.clone(),
    };
    let (mounted, image_after_the_remount, reads_of_each_slot_at_each_hook_call) =
        remount_through(&before, &root_ring_reads);
    RemountThroughFaultedReads {
        mounted,
        image_after_the_remount,
        reads_of_each_slot_at_each_hook_call,
        faulted_reads: root_ring_reads.faulted_reads(),
    }
}

/// 根环里这一槽这一刻是不是还住着 C。
pub fn abandoned_root_still_in_the_ring(
    image: &MemoryPool,
    history: &PoolWithAnAbandonedRootAndThePlacesItWrote,
) -> bool {
    self_verified_roots_in_the_ring(image).contains(&(
        history.ring_slot_of_the_abandoned_root,
        history.abandoned_root,
    ))
}

/// 在 K2-2 那组历史上照 `faulted_reads_of_each_slot` 再挂、接着覆盖写到 C 离开根环；盘面拿 `reference_faulted_reads_of_each_slot` 那一次再挂比。
pub fn remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
    history: &PoolWithAnAbandonedRootAndThePlacesItWrote,
    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
    reference_faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
) -> RemountOfThePoolWithAnAbandonedRoot {
    let named_faulted_reads = u64::try_from(faulted_reads_of_each_slot.len()).expect("次数");
    let RemountThroughFaultedReads {
        mounted,
        image_after_the_remount,
        reads_of_each_slot_at_each_hook_call: reads_at_each_hook_call,
        faulted_reads,
    } = remount_the_pool_with_an_abandoned_root(history, faulted_reads_of_each_slot);
    let reference =
        remount_the_pool_with_an_abandoned_root(history, reference_faulted_reads_of_each_slot);
    let image_after_the_reference_remount = reference.image_after_the_remount;
    reference
        .mounted
        .unwrap_or_else(|error| panic!("前提：照对照的读坏计划再可写挂载做得成：{error:?}"));
    let hook_calls_after_exactly_the_rebuild_pass: Vec<bool> = reads_at_each_hook_call
        .iter()
        .map(|reads_of_each_slot| {
            !reads_of_each_slot.is_empty()
                && reads_of_each_slot
                    .values()
                    .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)
        })
        .collect();
    let faulted_reads_are_the_named_ones =
        named_faulted_reads > 0 && faulted_reads == named_faulted_reads;
    let same_disk_as_the_reference_remount =
        image_after_the_remount == image_after_the_reference_remount;
    let mounted = match mounted {
        Ok(mounted) => mounted,
        Err(error) => {
            return RemountOfThePoolWithAnAbandonedRoot {
                root_ring_of_the_rebuilt_allocator: Err(format!("{error:?}")),
                hook_calls_after_exactly_the_rebuild_pass,
                faulted_reads_are_the_named_ones,
                same_disk_as_the_reference_remount,
                both_devices_isolate_slots_after_the_mount: false,
                no_place_of_the_abandoned_root_is_free_after_the_mount: false,
                places_of_the_abandoned_root_kept_until_it_leaves_the_ring: false,
            };
        }
    };
    let root_ring_of_the_rebuilt_allocator =
        Ok(mounted.output.rereads.root_ring_of_the_rebuilt_allocator);
    let both_devices_isolate_slots_after_the_mount = mounted.output.isolated_slots_per_device.len()
        == 2
        && mounted
            .output
            .isolated_slots_per_device
            .iter()
            .all(|(_, isolated_slots)| *isolated_slots > 0);
    let no_place_of_the_abandoned_root_is_free_after_the_mount = history
        .places_the_abandoned_root_wrote
        .iter()
        .all(|(device, slot, _)| {
            !mounted.allocator.devices[usize::try_from(device.0).expect("盘号")].is_free(*slot)
        });
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
    let mut places_of_the_abandoned_root_kept_until_it_leaves_the_ring = true;
    let mut overwrites = 0usize;
    // 迭代上界是根环的槽数（再挂写了几条根，C 的根槽在一圈之内被盖掉）；提前出口：C 的哪个落点被改写了就停。
    while abandoned_root_still_in_the_ring(&pool.image(), history) {
        assert!(
            overwrites < root_ring_slot_count,
            "C 的根槽在一圈（{root_ring_slot_count} 次发布）之内被盖掉"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("再挂之后第 {overwrites} 次覆盖写：{refusal:?}"));
        overwrites += 1;
        if history
            .places_the_abandoned_root_wrote
            .iter()
            .any(|(device, slot, bytes)| {
                bytes_on(&pool.devices, *device, slot.to_device_offset(), bytes.len()) != *bytes
            })
        {
            places_of_the_abandoned_root_kept_until_it_leaves_the_ring = false;
            break;
        }
    }
    // C 离开根环那一刻两块盘都在时池级 checker 不判违例
    images.hand_out(
        "接着覆盖写到 C 离开根环之后",
        || pool.image(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    RemountOfThePoolWithAnAbandonedRoot {
        root_ring_of_the_rebuilt_allocator,
        hook_calls_after_exactly_the_rebuild_pass,
        faulted_reads_are_the_named_ones,
        same_disk_as_the_reference_remount,
        both_devices_isolate_slots_after_the_mount,
        no_place_of_the_abandoned_root_is_free_after_the_mount,
        places_of_the_abandoned_root_kept_until_it_leaves_the_ring,
    }
}

/// 盘 1 那几个区域的每个根环槽。
pub fn root_ring_slots_of_device_one(
    history: &PoolWithAnAbandonedRootAndThePlacesItWrote,
) -> Vec<RootRingSlot> {
    every_root_ring_slot(&history.parameters.geometry)
        .into_iter()
        .filter(|ring_slot| {
            history.parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")]
                == DeviceIdentity(1)
        })
        .collect()
}

/// 每个点名的槽在第 `read_of_each_slot` 次读时读成 `fault`。
pub fn faulted_reads_of(
    ring_slots: &[RootRingSlot],
    read_of_each_slot: u64,
    fault: RootRingSlotReadFault,
) -> BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> {
    ring_slots
        .iter()
        .map(|ring_slot| ((*ring_slot, read_of_each_slot), fault))
        .collect()
}

/// 并两遍之后的预期：重读过一次（钩子恰调一次，在重建那一遍之后）、照常挂上，与对照那一次再挂同一份盘面；两块盘都隔离、C 的落点一个都不空闲，
/// 接着覆盖写到 C 离开根环之前 C 写出的落点一个都没被改写，那一刻池级 checker 不判违例。
pub fn merged_and_keeps_the_abandoned_root_isolated() -> RemountOfThePoolWithAnAbandonedRoot {
    RemountOfThePoolWithAnAbandonedRoot {
        root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
        hook_calls_after_exactly_the_rebuild_pass: vec![true],
        faulted_reads_are_the_named_ones: true,
        same_disk_as_the_reference_remount: true,
        both_devices_isolate_slots_after_the_mount: true,
        no_place_of_the_abandoned_root_is_free_after_the_mount: true,
        places_of_the_abandoned_root_kept_until_it_leaves_the_ring: true,
    }
}

/// 第一遍盘 1 整段报读错、重读只坏 C 的槽（读成 `fault`）的那一格（攻方 `pass1:dev1-all-e reread:c-*`）。
pub fn first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
    images: &mut PoolCheckerImages,
    device_width: HistoryDeviceWidth,
    overwrites_before_the_abandoned_root: usize,
    fault: RootRingSlotReadFault,
) -> RemountOfThePoolWithAnAbandonedRoot {
    let history = pool_with_an_abandoned_root(device_width, overwrites_before_the_abandoned_root);
    let mut faulted_reads = faulted_reads_of(
        &root_ring_slots_of_device_one(&history),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads.extend(faulted_reads_of(
        &[history.ring_slot_of_the_abandoned_root],
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
        fault,
    ));
    remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
        images,
        &history,
        faulted_reads,
        BTreeMap::new(),
    )
}

/// 第一遍盘 1 每个根环槽与 C 的根槽都报读错、重读那一遍都读得出的那一格（K2-2 的反方向，攻方 `pass1:dev1-all-e+c-e reread:-`）。
pub fn first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(
    images: &mut PoolCheckerImages,
    device_width: HistoryDeviceWidth,
    overwrites_before_the_abandoned_root: usize,
) -> RemountOfThePoolWithAnAbandonedRoot {
    let history = pool_with_an_abandoned_root(device_width, overwrites_before_the_abandoned_root);
    let mut faulted_reads = faulted_reads_of(
        &root_ring_slots_of_device_one(&history),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads.extend(faulted_reads_of(
        &[history.ring_slot_of_the_abandoned_root],
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    ));
    remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
        images,
        &history,
        faulted_reads,
        BTreeMap::new(),
    )
}

/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读那一遍只有 C 的根槽报读错。
/// 预期：按槽并两遍，C 那一槽留第一遍读出的 C——影子账照常隔离 C 独占的单元，盘面与不注错的再挂相同；接着覆盖写到 C 离开根环，
/// C 写出的落点一个都没被改写，池级 checker 不红（D23（journal 的角色与格式） 已定项 14「被崩溃恢复抛弃的时间线的根离开根环之前，
/// 它们引用的单元不许重新分配也不许抹头」）。
/// 判别力：改回整份换成重读的读数（`crates/mutations.tsv`「R1 改回整份换成重读的读数」）时 C 掉出影子账、两块盘隔离 0，
/// 第一次覆盖写就复用 C 的单元（I-7.4 红），隔离那一项起红；「R1 第一遍自证过压不过重读读坏」同样红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            1,
            RootRingSlotReadFault::ReadFails
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 1 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：同上，重读那一遍 C 的根槽读回全 0。
/// 预期与判别力同上一条（第一遍自证过压过重读全零）。
pub fn run_the_scenario_where_the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            1,
            RootRingSlotReadFault::ReadsBackAllZero
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：同上，重读那一遍 C 的根槽读回乱字节。
/// 预期与判别力同上一条。
pub fn run_the_scenario_where_the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            1,
            RootRingSlotReadFault::ReadsBackGarbage
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 的反方向（攻方 `pass1:dev1-all-e+c-e reread:-`；单元区 384 槽、实例 1 覆盖写 1 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出。
/// 预期：按槽并两遍，C 那一槽取重读读出的 C，与不注错的再挂同一份盘面，影子账照隔离，C 离开根环之前它的落点不被改写，池级 checker 不红。
/// 判别力：并的时候第一遍读坏压过重读自证过的根（`crates/mutations.tsv`「R1 第一遍读坏压过重读自证过的根」）时 C 掉出影子账，
/// 隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            1,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 1 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽报读错（R1 补的盘宽与前置覆盖写两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            3,
            RootRingSlotReadFault::ReadFails
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 3 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回全 0（R1 补的盘宽与前置覆盖写两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            3,
            RootRingSlotReadFault::ReadsBackAllZero
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回乱字节（R1 补的盘宽与前置覆盖写两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            3,
            RootRingSlotReadFault::ReadsBackGarbage
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 的反方向（单元区 384 槽、实例 1 覆盖写 3 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出（R1 补的两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力同上一条（同一行变异的另一个名字：「R1 两维：重读之后的读数只剩最新那条根…」）。
pub fn run_the_scenario_where_the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(images,
            HistoryDeviceWidth::UnitAreaOf384Slots,
            3,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "单元区 384 槽、实例 1 覆盖写 3 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽报读错（R1 补的盘宽与前置覆盖写两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            1,
            RootRingSlotReadFault::ReadFails
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 1 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回全 0（R1 补的盘宽与前置覆盖写两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            1,
            RootRingSlotReadFault::ReadsBackAllZero
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回乱字节（R1 补的盘宽与前置覆盖写两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            1,
            RootRingSlotReadFault::ReadsBackGarbage
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 的反方向（4 GiB 盘、实例 1 覆盖写 1 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出（R1 补的两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力同上一条（同一行变异的另一个名字：「R1 两维：重读之后的读数只剩最新那条根…」）。
pub fn run_the_scenario_where_the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(images,
            HistoryDeviceWidth::FourGibibytes,
            1,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 1 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽报读错（R1 补的盘宽与前置覆盖写两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            3,
            RootRingSlotReadFault::ReadFails
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 3 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回全 0（R1 补的盘宽与前置覆盖写两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            3,
            RootRingSlotReadFault::ReadsBackAllZero
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回乱字节（R1 补的盘宽与前置覆盖写两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
pub fn run_the_scenario_where_the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(images,
            HistoryDeviceWidth::FourGibibytes,
            3,
            RootRingSlotReadFault::ReadsBackGarbage
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
    );
}

/// 场景：K2-2 的反方向（4 GiB 盘、实例 1 覆盖写 3 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出（R1 补的两维）。
/// 预期同 `run_the_scenario_where_the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
/// 判别力同上一条（同一行变异的另一个名字：「R1 两维：重读之后的读数只剩最新那条根…」）。
pub fn run_the_scenario_where_the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(images,
            HistoryDeviceWidth::FourGibibytes,
            3,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "4 GiB 盘、实例 1 覆盖写 3 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
    );
}

/// 场景：第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读那一遍整个环都报读错（攻方 `pass1:dev1-all-e reread:all-e`，判决 K2「多拒」那一格）。
/// 预期：按槽并两遍，第一遍盘 0 读出的根都留着，照常挂上（不是 Q3 那一拒），与「两遍都只坏盘 1」的再挂同一份盘面（两份并出来的读数相同：
/// 盘 0 读出、盘 1 读坏，照 m 取不到那一支往下走）；影子账照隔离 C，C 离开根环之前它的落点不被改写，池级 checker 不红。
/// 判别力：改回整份换成重读的读数时重读那一遍一条有效根都没有，拒成 `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，第一项红。
pub fn run_the_scenario_where_a_reread_failing_on_the_whole_ring_after_a_first_pass_that_read_device_zero_mounts_on_the_first_pass_roots(
    images: &mut PoolCheckerImages,
) {
    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 1);
    let ring_slots_of_device_one = root_ring_slots_of_device_one(&history);
    let mut faulted_reads = faulted_reads_of(
        &ring_slots_of_device_one,
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads.extend(faulted_reads_of(
        &every_root_ring_slot(&history.parameters.geometry),
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
        RootRingSlotReadFault::ReadFails,
    ));
    let mut faulted_reads_only_on_device_one = faulted_reads_of(
        &ring_slots_of_device_one,
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
        RootRingSlotReadFault::ReadFails,
    );
    faulted_reads_only_on_device_one.extend(faulted_reads_of(
        &ring_slots_of_device_one,
        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
        RootRingSlotReadFault::ReadFails,
    ));
    assert_eq!(
        remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
            images,
            &history,
            faulted_reads,
            faulted_reads_only_on_device_one,
        ),
        merged_and_keeps_the_abandoned_root_isolated(),
        "重读整环都坏、第一遍读出了盘 0 的根：并两遍照常挂上"
    );
}
