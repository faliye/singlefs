//! 重挂之前的几种池（崩了的卸载、续做之后崩的、带活文件的、窄池上续做过删除的），与照读错计划重挂一次、读活文件与根环里自证过的根。

use super::*;

/// 窄盘：第一个文件之后覆盖写 `overwrites` 次、顺序写长到 `data_units_of_the_file` 个单元，在这个进程里发 unlink 的意图、
/// 再删 `batches_after_the_intent` 批就崩；重开可写（准入判着、影子账开、批大小 1）续做走完，会话挂回池上。
pub fn narrow_pool_whose_deletion_was_continued(
    unit_area_slots: u64,
    overwrites: u64,
    data_units_of_the_file: usize,
    batches_after_the_intent: u64,
) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            unit_area_slots,
        );
    for overwrite_index in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
        });
    }
    pool.sequential_write((data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
        .unwrap_or_else(|refusal| {
            panic!("顺序写把文件长到 {data_units_of_the_file} 个单元：{refusal:?}")
        });
    {
        let PoolUnderTest {
            session,
            devices,
            parameters,
            write_time_seconds,
            ..
        } = &mut pool;
        let session = session.as_mut().expect("这一刻有可写会话");
        let mut current = session
            .current
            .version_of_trees()
            .expect("第一个文件之后有树")
            .clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            *write_time_seconds,
            session.instance,
        )
        .expect("意图那次");
        // 迭代上界是 `batches_after_the_intent`；跨轮携带的是现行那一版。
        for batch_index in 0..batches_after_the_intent {
            current = publish_unlink_step(
                &mut writer,
                &mut session.allocator,
                &current,
                InodeNumber(FIRST_INODE_NUMBER),
                deletion_switches_with_batches_of_one().batch_size,
                session.instance,
            )
            .unwrap_or_else(|refusal| panic!("第 {batch_index} 批：{refusal:?}"));
        }
    }
    pool.session = None;
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .unwrap_or_else(|error| panic!("崩了之后重开可写、续做：{error:?}"));
    let (_, session_of_the_continued_mount) = MountedSession::of_the_mount(mounted);
    pool.session = Some(session_of_the_continued_mount);
    pool
}

pub fn pool_before_the_remount(starting_point: StartingPoint) -> PoolBeforeTheRemount {
    match starting_point {
        StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites => {
            let unmount_writes_landed = 8;
            let pool = narrow_pool_whose_deletion_was_continued(192, 9, 3, 1);
            let image_before_the_unmount = pool.image();
            let write_plan = plan_failing_every_write_from(
                HistoryDeviceWidth::UnitAreaOf384Slots,
                unmount_writes_landed + 1,
            );
            let mut unmount_devices: Vec<(
                DeviceIdentity,
                FaultInjectingBlockDevice<SparseBlockDevice>,
            )> = plain_devices_on(&image_before_the_unmount)
                .into_iter()
                .map(|(identity, device)| {
                    (
                        identity,
                        FaultInjectingBlockDevice::new(identity, device, write_plan.clone()),
                    )
                })
                .collect();
            let session = pool.session();
            let mut allocator = session.allocator.clone();
            let mut current = session.current.clone();
            let unmounted = unmount(
                &pool.parameters,
                &mut unmount_devices,
                &mut allocator,
                &mut current,
                session.shadow_ledger,
            );
            assert!(
                unmounted.is_err(),
                "前提：卸载在第 {} 个写上掉电、报错",
                unmount_writes_landed + 1
            );
            let image_after_the_crashed_unmount = MemoryPool {
                devices: unmount_devices
                    .iter()
                    .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                    .collect(),
                device_size_in_bytes: image_before_the_unmount.device_size_in_bytes,
            };
            PoolBeforeTheRemount {
                image_before_the_crash: image_before_the_unmount,
                image_to_remount: image_after_the_crashed_unmount,
                parameters: pool.parameters.clone(),
            }
        }
        StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount => {
            let pool = narrow_pool_whose_deletion_was_continued(216, 7, 5, 4);
            let image = pool.image();
            PoolBeforeTheRemount {
                image_before_the_crash: image.clone(),
                image_to_remount: image,
                parameters: pool.parameters.clone(),
            }
        }
        StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount => {
            let mut pool =
                start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                    HistoryDeviceWidth::UnitAreaOf384Slots,
                    264,
                );
            // 迭代上界是 6 次覆盖写；没有提前出口。
            for overwrite_index in 0..6 {
                pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
                    panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
                });
            }
            pool.sequential_write(3 * data_unit_payload_capacity() + 100)
                .unwrap_or_else(|refusal| panic!("顺序写把文件长到 4 个单元：{refusal:?}"));
            pool.session = None;
            mount_writable_with_test_only_switches_and_deletion_switches(
                &pool.parameters,
                &mut pool.devices,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                deletion_switches_with_batches_of_one(),
            )
            .unwrap_or_else(|error| panic!("崩了之后重开可写：{error:?}"));
            let image = pool.image();
            PoolBeforeTheRemount {
                image_before_the_crash: image.clone(),
                image_to_remount: image,
                parameters: pool.parameters.clone(),
            }
        }
        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits => {
            let mut pool =
                start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                    HistoryDeviceWidth::UnitAreaOf384Slots,
                    384,
                );
            // 迭代上界是 20 次覆盖写；没有提前出口。
            for overwrite_index in 0..20 {
                pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
                    panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
                });
            }
            pool.sequential_write(data_unit_payload_capacity() + 100)
                .unwrap_or_else(|refusal| panic!("顺序写把文件长到 2 个单元：{refusal:?}"));
            pool.session = None;
            let image = pool.image();
            PoolBeforeTheRemount {
                image_before_the_crash: image.clone(),
                image_to_remount: image,
                parameters: pool.parameters.clone(),
            }
        }
    }
}

/// 照计划点名的根环槽（`but1` 按这份盘面上每块盘最新那条自证过的根认）。
pub fn faulted_slots_of_the_plan(
    image: &MemoryPool,
    plan: RebuildPassPlan,
) -> BTreeMap<RootRingSlot, RootRingSlotReadFault> {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = system_configuration.immutable;
    let device_of = |ring_slot: RootRingSlot| {
        immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")]
    };
    let ring_slots = every_root_ring_slot(&immutable.sizes);
    let mut slot_of_the_newest_root_on_each_device: BTreeMap<
        DeviceIdentity,
        (CheckpointTxg, RootRingSlot),
    > = BTreeMap::new();
    // 迭代上界是根环的槽数；没有提前出口。
    for ring_slot in &ring_slots {
        if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(
            image,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            *ring_slot,
        ) {
            let newest = slot_of_the_newest_root_on_each_device
                .entry(device_of(*ring_slot))
                .or_insert((root.checkpoint_txg, *ring_slot));
            if root.checkpoint_txg > newest.0 {
                *newest = (root.checkpoint_txg, *ring_slot);
            }
        }
    }
    ring_slots
        .into_iter()
        .filter(|ring_slot| {
            let device = device_of(*ring_slot);
            let slots_of_the_device = if device == DeviceIdentity(0) {
                plan.device_zero
            } else {
                plan.device_one
            };
            match slots_of_the_device {
                RootRingSlotsOfOneDevice::NoSlot => false,
                RootRingSlotsOfOneDevice::Every => true,
                RootRingSlotsOfOneDevice::EveryButTheSlotOfTheNewestRoot => {
                    slot_of_the_newest_root_on_each_device
                        .get(&device)
                        .is_none_or(|(_, newest_slot)| newest_slot != ring_slot)
                }
            }
        })
        .map(|ring_slot| (ring_slot, plan.fault))
        .collect()
}

/// 只读挂载、把挂载态里每个活着的文件从头读到尾：每个活文件（inode 号、全部字节）。删除中的 inode（nlink = 0）不读。
pub fn live_files_read_whole(image: &MemoryPool) -> Result<Vec<(u64, Vec<u8>)>, String> {
    let read_only = mount_read_only(image).map_err(|failure| format!("只读挂载：{failure:?}"))?;
    let mut live_files = Vec::new();
    // 迭代上界是这一版 inode 树的记录数；提前出口只有读不出的那三种错。
    for record in read_only.mounted.inode_records() {
        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = read_only
            .mounted
            .open_file(image, InodeNumber(record.inode))
            .map_err(|failure| format!("打开 inode {}：{failure:?}", record.inode))?;
        let read = file
            .read_at(image, FileOffsetInBytes(0), record.size)
            .map_err(|failure| format!("读 inode {}：{failure:?}", record.inode))?;
        live_files.push((record.inode, read.bytes));
    }
    Ok(live_files)
}

/// 在 `root` 那一版上打开、整读每个活文件（inode 号、全部字节）；删除中的 inode 不读。
pub fn live_files_read_whole_at(
    image: &MemoryPool,
    root: &RootRecord,
) -> Result<Vec<(u64, Vec<u8>)>, String> {
    let mounted =
        open_pool_for_read(image, root).map_err(|failure| format!("打开那一版：{failure:?}"))?;
    let mut live_files = Vec::new();
    // 迭代上界是这一版 inode 树的记录数；提前出口只有读不出的那两种错。
    for record in mounted.inode_records() {
        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = mounted
            .open_file(image, InodeNumber(record.inode))
            .map_err(|failure| format!("打开 inode {}：{failure:?}", record.inode))?;
        let read = file
            .read_at(image, FileOffsetInBytes(0), record.size)
            .map_err(|failure| format!("读 inode {}：{failure:?}", record.inode))?;
        live_files.push((record.inode, read.bytes));
    }
    Ok(live_files)
}

/// 一份镜像的根环里每条自证过的根，带它读出的那个槽。
pub fn self_verified_roots_in_the_ring(image: &MemoryPool) -> Vec<(RootRingSlot, RootRecord)> {
    let immutable = choose_system_configuration(image)
        .expect("系统配置")
        .immutable;
    every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                image,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some((ring_slot, root)),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect()
}

/// 挂载之前环里每条自证过的根，挂载之后还住在同一槽、txg 不低于挂载之后最新那条根带的 F（回退还能落到的），丢 `lost` 之后在剩下那块盘上
/// 整读每个活文件，与挂载之前丢同一块盘读出的相同（挂载之前就读不出的不比）。交回比了几条根、其中有活文件的几条与读得不一样的那几条的 txg。
pub fn roots_still_in_the_ring_read_on_a_lone_survivor(
    image_before_the_remount: &MemoryPool,
    image_after_the_remount: &MemoryPool,
    lost: DeviceIdentity,
) -> (usize, usize, Vec<u64>) {
    let roots_after: Vec<(RootRingSlot, RootRecord)> =
        self_verified_roots_in_the_ring(image_after_the_remount);
    let floor_after = roots_after
        .iter()
        .map(|(_, root)| (root.checkpoint_txg, root.instance, root.rollback_floor))
        .max()
        .map_or(CheckpointTxg(0), |(_, _, rollback_floor)| rollback_floor);
    let before_without = image_without(image_before_the_remount, lost);
    let after_without = image_without(image_after_the_remount, lost);
    let mut compared = 0;
    let mut compared_with_live_files = 0;
    let mut read_differently = Vec::new();
    // 迭代上界是根环的槽数；没有提前出口。
    for (ring_slot, root) in self_verified_roots_in_the_ring(image_before_the_remount) {
        if root.checkpoint_txg < floor_after || !roots_after.contains(&(ring_slot, root)) {
            continue;
        }
        let Ok(before) = live_files_read_whole_at(&before_without, &root) else {
            continue;
        };
        compared += 1;
        if !before.is_empty() {
            compared_with_live_files += 1;
        }
        if live_files_read_whole_at(&after_without, &root) != Ok(before) {
            read_differently.push(root.checkpoint_txg.0);
        }
    }
    (compared, compared_with_live_files, read_differently)
}

/// 一份镜像去掉一块盘。
pub fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
    MemoryPool {
        devices: image
            .devices
            .iter()
            .filter(|(identity, _)| **identity != lost)
            .map(|(identity, device)| (*identity, device.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

/// 照计划只坏重建那一遍的再挂，看到的那几样（各用例自己断言）。
#[derive(Debug, PartialEq, Eq)]
pub struct RemountUnderThePlan {
    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
    pub root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
    /// 重读之前的钩子调了几次，每次调的那一刻每个根环槽是不是都恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次。
    pub hook_calls_after_exactly_the_rebuild_pass: Vec<bool>,
    /// 这一次读坏的恰是计划点名的那几个槽、各一次。
    pub faulted_reads_are_the_named_slots_once: bool,
    /// 交回的盘面与同一份盘面上不注错的再挂交回的逐字节相同。
    pub same_disk_as_a_remount_without_faults: bool,
    /// 丢盘 0、丢盘 1 之后剩下那块盘只读挂载、整读每个活文件，与崩溃之前同一块盘上读出的相同。
    pub lone_survivor_reads_what_it_read_before_the_crash: [bool; 2],
    /// 丢盘 0、丢盘 1 之后，挂载之前环里还在、不低于 F 的每条自证根在剩下那块盘上整读与挂载之前相同
    /// （[`roots_still_in_the_ring_read_on_a_lone_survivor`]）。
    pub lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [bool; 2],
}

/// 再挂一次（准入判着、影子账开），读盘经 `root_ring_reads`；重读之前调钩子记下那一刻每个根环槽读过几次。
pub fn remount_through(
    before: &PoolBeforeTheRemount,
    root_ring_reads: &SharedRootRingReadsWithFaults,
) -> (
    Result<Mounted, MountError>,
    MemoryPool,
    Vec<BTreeMap<RootRingSlot, u64>>,
) {
    let mut devices: Vec<(DeviceIdentity, DeviceFailingTheRebuildPass)> =
        plain_devices_on(&before.image_to_remount)
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
    let reads_seen_by_the_hook = root_ring_reads.clone();
    let mut reads_of_each_slot_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>> = Vec::new();
    let mut note_the_reads = || {
        reads_of_each_slot_at_each_hook_call.push(reads_seen_by_the_hook.reads_of_each_slot());
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &before.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut note_the_reads),
    );
    let image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect(),
        device_size_in_bytes: before.image_to_remount.device_size_in_bytes,
    };
    (mounted, image, reads_of_each_slot_at_each_hook_call)
}

pub fn remount_under_the_plan(
    images: &mut PoolCheckerImages,
    starting_point: StartingPoint,
    plan: RebuildPassPlan,
) -> RemountUnderThePlan {
    let before = pool_before_the_remount(starting_point);
    let faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
        faulted_slots_of_the_plan(&before.image_to_remount, plan)
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
    let faulted_slot_count = u64::try_from(faulted_reads_of_each_slot.len()).expect("槽数");
    let root_ring_reads =
        root_ring_reads_failing(&before.image_to_remount, faulted_reads_of_each_slot);
    let (mounted, image_after_the_remount, reads_at_each_hook_call) =
        remount_through(&before, &root_ring_reads);
    let (mounted_without_faults, image_after_a_remount_without_faults, _) = remount_through(
        &before,
        &root_ring_reads_failing(&before.image_to_remount, BTreeMap::new()),
    );
    mounted_without_faults
        .unwrap_or_else(|error| panic!("前提：同一份盘面不注错地再可写挂载做得成：{error:?}"));
    let lone_survivor_reads_what_it_read_before_the_crash = [DeviceIdentity(0), DeviceIdentity(1)]
        .map(|lost| {
            let before_the_crash =
                live_files_read_whole(&image_without(&before.image_before_the_crash, lost));
            before_the_crash.is_ok()
                && live_files_read_whole(&image_without(&image_after_the_remount, lost))
                    == before_the_crash
        });
    let lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount =
        [DeviceIdentity(0), DeviceIdentity(1)].map(|lost| {
            let (_, _, read_differently) = roots_still_in_the_ring_read_on_a_lone_survivor(
                &before.image_to_remount,
                &image_after_the_remount,
                lost,
            );
            read_differently.is_empty()
        });
    // 两块盘都在时池级 checker 不判违例
    images.hand_out(
        "照计划只坏重建那一遍的再挂之后",
        || image_after_the_remount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    RemountUnderThePlan {
        root_ring_of_the_rebuilt_allocator: mounted
            .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
            .map_err(|error| format!("{error:?}")),
        hook_calls_after_exactly_the_rebuild_pass: reads_at_each_hook_call
            .iter()
            .map(|reads_of_each_slot| {
                !reads_of_each_slot.is_empty()
                    && reads_of_each_slot
                        .values()
                        .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)
            })
            .collect(),
        faulted_reads_are_the_named_slots_once: faulted_slot_count > 0
            && root_ring_reads.faulted_reads() == faulted_slot_count,
        same_disk_as_a_remount_without_faults: image_after_the_remount
            == image_after_a_remount_without_faults,
        lone_survivor_reads_what_it_read_before_the_crash,
        lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount,
    }
}
