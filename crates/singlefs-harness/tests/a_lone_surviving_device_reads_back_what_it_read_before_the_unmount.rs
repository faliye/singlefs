//! 卸载那一串掉电之后只剩一块盘：剩下那块盘只读挂载、整读每个活文件，与卸载之前同一块盘上读出的逐项相同（走到的根 txg 与内容）。
//! 判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 2 条（GC）；用户 2026-10-03 定甲 GA
//! （`mount::RaiseFloorEntry::reuse_of_the_reclaimed_slots`：卸载那一格只把释放代不高于每块盘上最新有效根的那一截当场放开）。
//! 刀摆在卸载那一串某一次空发布的单元写写完之后（先写系统配置那两个写、之前那几次空发布的全部写与这一次的单元写落下，这一次的 journal 记录与根都没落），
//! 丢掉住着现行那一版的根的那块盘。恢复择根不看 F（`recovery::choose_root`），剩下那块盘择到它自己上面最新的根，低于系统配置里的新 F。
//! 两形照攻方（`research/prompts/m3-unlink-code-r3-opus-output.md` 第一节 1.4）：h2 窄盘续做之后卸载，h4 4 GiB 盘卸载—再挂循环之后第二次卸载，刀都在第一次空发布。
//! 第 16 批补两形（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第三节第 3、4 条）：窄盘崩了再挂一次之后卸载、刀在第二次空发布，
//! 剩下那块盘整读的活文件不为空；同一形上这一遍有一块盘一条有效根都读不出，这一串回收的槽整截扣到生效。
//! 池级 checker 在单盘镜像上卸载之前就红六七条（判决第二节 N1「第五节跑前判据的字面」那一行），这里不判 checker，判的是读回。

mod common_admission;

use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    start_plain, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{PoolAllocator, RootRingOccupancy, RootRingOccupant};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, unmount, MountError,
    ShadowLedger, Unmounted,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
    RootRingSlotReading,
};
use singlefs_core::root_ring::{target_for_publish, RootRingSlot};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_core::write_accounting::WrittenStructureKind;
use singlefs_format::ROOT_RING_REGIONS;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, NamedRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 只读挂载、把挂载态里每个活着的文件从头读到尾：走到的根 txg 与每个活文件（inode 号、全部字节）。删除中的 inode（nlink = 0）读者答「没有文件」，不读。
#[derive(Debug, PartialEq, Eq)]
struct WholeReadOfTheLiveFiles {
    root_txg: CheckpointTxg,
    live_files: Vec<(u64, Vec<u8>)>,
}

/// 只读挂载与整读每个活文件；哪一步读不出就交回那一步的错（只读挂载、打开、读）。
fn whole_read_of_the_live_files(image: &MemoryPool) -> Result<WholeReadOfTheLiveFiles, String> {
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
    Ok(WholeReadOfTheLiveFiles {
        root_txg: read_only.effective_root.checkpoint_txg,
        live_files,
    })
}

/// 一份镜像去掉一块盘。
fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
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

/// 卸载那一串掉电、丢掉一块盘之后要比的几样。
struct LoneSurvivorAfterAnUnmountPublish {
    /// 丢掉的盘：住着卸载之前现行那一版的根。
    lost_device: DeviceIdentity,
    current_txg_before_the_unmount: CheckpointTxg,
    /// 掉电在那一串第几次空发布（从 0 数）的单元写写完之后，那一次写了几个单元（不注错地在一份拷贝上卸载一次数出来的）。
    unit_writes_of_the_publish: u64,
    /// 剩下那块盘上恢复择到的根（看 journal）。
    root_chosen_on_the_survivor: Option<CheckpointTxg>,
    before_the_unmount: Result<WholeReadOfTheLiveFiles, String>,
    after_the_crash: Result<WholeReadOfTheLiveFiles, String>,
}

/// 卸载那一串第几次空发布（从 0 数）的单元写写完之后掉电。
#[derive(Clone, Copy, Debug)]
struct PublishOfTheUnmountSequence(usize);

/// 卸载时每次读都失败的根环槽（只供测试的开关 `RootRingSlotTarget`，`NamedRootRingSlots::NONE` 是哪个都不点名）。
fn ring_slots_failing_every_read_during_the_unmount(
    pool: &PoolUnderTest<SparseBlockDevice>,
    device_width: HistoryDeviceWidth,
    named_slots: NamedRootRingSlots,
) -> SharedFaultPlan {
    SharedFaultPlan::armed(
        device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::ReadFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::WithinNamedRootRingSlots(RootRingSlotTarget {
                named_slots,
                region_devices: pool.parameters.region_devices,
                fixed_structure_slot_spacing: pool.parameters.geometry.fixed_structure_slot_spacing,
            }),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
        },
    )
}

/// 在 `pool` 的盘面上拿 `allocator_of_the_unmount`（会话的分配器或它的改写本）与会话的现行那一版卸载：先在一份拷贝上不注错地卸载一次，
/// 数清那一串第 `publish` 次空发布之前落下的写与它自己的单元写；再在另一份上从「那几个写 + 这一次的单元写」之后的第一个写起每个写都报错
/// （掉电那一形：盘停在这一次的单元写之后，它的 journal 记录与根都没落）；丢掉住着现行那一版的根的那块盘，卸载之前与掉电之后各在剩下那块盘上整读一遍。
/// `unreadable_ring_slots` 点名的根环槽在两次卸载里每次读都失败（盘上内容不动，整读不经这一层）。
fn lone_survivor_after_the_unit_writes_of_an_unmount_publish(
    pool: &PoolUnderTest<SparseBlockDevice>,
    device_width: HistoryDeviceWidth,
    allocator_of_the_unmount: &PoolAllocator,
    publish: PublishOfTheUnmountSequence,
    unreadable_ring_slots: NamedRootRingSlots,
) -> LoneSurvivorAfterAnUnmountPublish {
    let image_before_the_unmount = pool.image();
    let session = pool.session.as_ref().expect("这一刻有可写会话");
    let current_txg_before_the_unmount = session.current.root().checkpoint_txg;
    let (writes_before_the_publish, unit_writes_of_the_publish) = {
        let mut allocator = allocator_of_the_unmount.clone();
        let mut current = session.current.clone();
        let read_plan = ring_slots_failing_every_read_during_the_unmount(
            pool,
            device_width,
            unreadable_ring_slots,
        );
        let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
            plain_devices_on(&image_before_the_unmount)
                .into_iter()
                .map(|(identity, device)| {
                    (
                        identity,
                        FaultInjectingBlockDevice::new(identity, device, read_plan.clone()),
                    )
                })
                .collect();
        let unmounted = unmount(
            &pool.parameters,
            &mut devices,
            &mut allocator,
            &mut current,
            session.shadow_ledger,
        );
        let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
            panic!("拷贝上卸载做成：{:?}", unmounted.as_ref().err());
        };
        let the_publish = &raised.publishes[publish.0];
        let structure_writes = |kind: WrittenStructureKind| the_publish.writes.of(kind).write_calls;
        let writes_before_the_publish = raised
            .system_configuration_writes_before_the_first_publish
            .total()
            .write_calls
            + raised.publishes[..publish.0]
                .iter()
                .map(|earlier| earlier.writes.total().write_calls)
                .sum::<u64>();
        (
            writes_before_the_publish,
            the_publish.writes.total().write_calls
                - structure_writes(WrittenStructureKind::JournalRecord)
                - structure_writes(WrittenStructureKind::RootSlot)
                - structure_writes(WrittenStructureKind::SystemConfigurationSlot),
        )
    };
    let write_plan = SharedFaultPlan::armed(
        device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(
                writes_before_the_publish + unit_writes_of_the_publish + 1,
            ),
        },
    );
    let read_plan =
        ring_slots_failing_every_read_during_the_unmount(pool, device_width, unreadable_ring_slots);
    let mut failing_devices: Vec<(
        DeviceIdentity,
        FaultInjectingBlockDevice<FaultInjectingBlockDevice<SparseBlockDevice>>,
    )> = plain_devices_on(&image_before_the_unmount)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, write_plan.clone()),
                    read_plan.clone(),
                ),
            )
        })
        .collect();
    let mut allocator = allocator_of_the_unmount.clone();
    let mut current = session.current.clone();
    let unmounted = unmount(
        &pool.parameters,
        &mut failing_devices,
        &mut allocator,
        &mut current,
        session.shadow_ledger,
    );
    assert!(
        matches!(
            unmounted,
            Err(MountError::RaiseFloorSequencePublishFailed(_))
        ),
        "掉电之后那一串第 {} 次空发布写 journal 记录时报错：{:?}",
        publish.0,
        unmounted.as_ref().err()
    );
    let crashed = MemoryPool {
        devices: failing_devices
            .iter()
            .map(|(identity, device)| {
                (
                    *identity,
                    device.wrapped_device().wrapped_device().image.clone(),
                )
            })
            .collect(),
        device_size_in_bytes: image_before_the_unmount.device_size_in_bytes,
    };
    let ring_slot_of_the_current_root = target_for_publish(
        current_txg_before_the_unmount,
        pool.parameters.geometry.root_ring_slots_per_region,
    );
    let lost_device = pool.parameters.region_devices
        [usize::try_from(ring_slot_of_the_current_root.region).expect("区域号")];
    let survivor_after_the_crash = image_without(&crashed, lost_device);
    LoneSurvivorAfterAnUnmountPublish {
        lost_device,
        current_txg_before_the_unmount,
        unit_writes_of_the_publish,
        root_chosen_on_the_survivor: recover(&survivor_after_the_crash, JournalPolicy::Consult)
            .effective_root
            .map(|(_, checkpoint_txg)| checkpoint_txg),
        before_the_unmount: whole_read_of_the_live_files(&image_without(
            &image_before_the_unmount,
            lost_device,
        )),
        after_the_crash: whole_read_of_the_live_files(&survivor_after_the_crash),
    }
}

/// 判一份 [`LoneSurvivorAfterAnUnmountPublish`]：那一次空发布真写了单元；剩下那块盘择到的是它自己上面低于现行那一版的根；
/// 卸载之前那块盘读得出；掉电之后读出来的与卸载之前逐项相同。
fn assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
    survivor: &LoneSurvivorAfterAnUnmountPublish,
) {
    assert!(
        survivor.unit_writes_of_the_publish > 0,
        "卸载那一串那次空发布写了单元（这一刀要判的就是它们先于带新 F 的根写下去）"
    );
    assert!(
        survivor
            .root_chosen_on_the_survivor
            .is_some_and(|chosen| chosen < survivor.current_txg_before_the_unmount),
        "丢掉住着现行那一版（txg {:?}）的根的盘 {:?} 之后，剩下那块盘择到它自己上面更早的根：{:?}",
        survivor.current_txg_before_the_unmount,
        survivor.lost_device,
        survivor.root_chosen_on_the_survivor
    );
    assert!(
        survivor.before_the_unmount.is_ok(),
        "卸载之前剩下那块盘只读挂载、整读每个活文件做得成：{:?}",
        survivor.before_the_unmount
    );
    assert_eq!(
        survivor.after_the_crash, survivor.before_the_unmount,
        "剩下那块盘走到的根 txg 与每个活文件的内容与卸载之前同一块盘上读出的相同：当场放开的槽没有一个被它最新的根引用"
    );
}

/// 窄盘（不补环窗口、准入关着造，单元区 176 槽）：第一个文件之后不覆盖写，崩了再挂可写一次（准入判着、影子账开、批大小 1），在这次挂载的会话上卸载。
/// 盘 1 上最新的根是写行那次（txg 4），盘 0 上是暖机那次（txg 5，现行那一版）；盘 1 上最新的根引用的槽有一个由暖机那次释放（释放代 5）。
fn narrow_pool_remounted_once_after_its_first_file() -> PoolUnderTest<SparseBlockDevice> {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            176,
        );
    pool.session = None;
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .unwrap_or_else(|error| panic!("第一个文件之后崩了再挂可写：{error:?}"));
    let (_, session) = MountedSession::of_the_mount(mounted);
    pool.session = Some(session);
    pool
}

/// 会话分配器的一份改写本：根环表里不记 `device` 上的根环槽（「挂载那一刻读不出、这个进程之后没写过」，这个进程不知道那里住着根），
/// 别的槽照盘上读出来的记成有效根（这个池里没有被抛弃的根）。经可写挂载造不出这一份：暖机在每块盘上都写了这个进程的根。
fn allocator_not_knowing_the_root_ring_slots_of(
    pool: &PoolUnderTest<SparseBlockDevice>,
    device: DeviceIdentity,
) -> PoolAllocator {
    let image = pool.image();
    let system_configuration =
        choose_system_configuration(&image).expect("两块盘都在的盘面上系统配置择得出");
    let immutable = &system_configuration.immutable;
    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter(|ring_slot| {
            immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")] != device
        })
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                &image,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some((
                    ring_slot,
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: root.checkpoint_txg,
                    },
                )),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect();
    let session = pool.session();
    let mut allocator = session.allocator.clone();
    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
        immutable.sizes.root_ring_slots_per_region,
        occupants,
        session.current.root().rollback_floor,
        session.current.root().checkpoint_txg,
    ));
    allocator
}

/// `device` 那几个区域的每个根环槽。
fn every_root_ring_slot_of(
    pool: &PoolUnderTest<SparseBlockDevice>,
    device: DeviceIdentity,
) -> NamedRootRingSlots {
    let slots_per_region = pool.parameters.geometry.root_ring_slots_per_region.count();
    let ring_slots: Vec<RootRingSlot> = (0..ROOT_RING_REGIONS)
        .filter(|region| {
            pool.parameters.region_devices[usize::try_from(*region).expect("区域号")] == device
        })
        .flat_map(|region| (0..slots_per_region).map(move |slot| RootRingSlot { region, slot }))
        .collect();
    NamedRootRingSlots::naming(&ring_slots)
}

/// h2（攻方第一节 1.4）：不补环窗口、准入关着造的窄盘（单元区 176 槽），不先覆盖写，第一个文件顺序写长到 4 个单元，
/// 在这个进程里发 unlink 的意图与 3 批（批大小 1）就崩；重开可写（准入判着、影子账开、批大小 1）续做走完，在这次挂载的会话上卸载。
/// 判别力：卸载那一格回收的槽整截当场放开（丙1，`ReclaimedReuse::Immediately`）时，第一次空发布把单元写进剩下那块盘上最新的根还引用的槽，
/// 那块盘只读挂载走树读不出（攻方原型上量过 `MappingStillUnreadable`），最后一项红。
#[test]
fn after_the_first_unmount_publish_of_a_narrow_pool_whose_deletion_was_continued_a_lone_surviving_device_reads_back_what_it_read_before(
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let data_units_of_the_file = 4;
    let batches_published_after_the_intent = 3;
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            device_width,
            176,
        );
    pool.sequential_write((data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 4 个单元：{refusal:?}"));
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
        for batch_index in 0..batches_published_after_the_intent {
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
    let (_, session) = MountedSession::of_the_mount(mounted);
    pool.session = Some(session);
    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
        &lone_survivor_after_the_unit_writes_of_an_unmount_publish(
            &pool,
            device_width,
            &pool.session().allocator,
            PublishOfTheUnmountSequence(0),
            NamedRootRingSlots::NONE,
        ),
    );
}

/// h4（攻方第一节 1.4）：4 GiB 盘（随机历史默认那一档），第一个文件之后崩了再挂 1 次、正常卸载、再挂、覆盖写 2 次，在这次挂载的会话上第二次卸载。
/// 判别力：卸载那一格回收的槽整截当场放开（丙1）时，第一次空发布把单元写进剩下那块盘上最新的根还引用的槽（攻方原型上量过：活文件的 extent 叶
/// 与 inode 叶被写掉、只读挂载之后打开文件失败），最后一项红。
#[test]
fn after_the_first_publish_of_a_second_unmount_of_a_four_gibibyte_pool_a_lone_surviving_device_reads_back_what_it_read_before(
) {
    let device_width = HistoryDeviceWidth::FourGibibytes;
    let mut pool = start_plain(device_width);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    {
        let PoolUnderTest {
            session,
            devices,
            parameters,
            ..
        } = &mut pool;
        let session = session.as_mut().expect("这一刻有可写会话");
        let unmounted = unmount(
            parameters,
            devices,
            &mut session.allocator,
            &mut session.current,
            session.shadow_ledger,
        );
        assert!(
            matches!(unmounted, Ok(Unmounted::FloorRaisedToTheCurrentVersion(_))),
            "第一次正常卸载做成：{:?}",
            unmounted.as_ref().err()
        );
    }
    pool.crash_and_mount_writable().expect("卸载之后再挂可写");
    for overwrite_index in 1..=2 {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
    }
    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
        &lone_survivor_after_the_unit_writes_of_an_unmount_publish(
            &pool,
            device_width,
            &pool.session().allocator,
            PublishOfTheUnmountSequence(0),
            NamedRootRingSlots::NONE,
        ),
    );
}

/// 活文件不为空的一形（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第三节第 4 条：h2 整读的活文件集合为空）：
/// 窄盘第一个文件之后崩了再挂一次（[`narrow_pool_remounted_once_after_its_first_file`]），在这次挂载的会话上卸载；掉电在那一串第二次空发布的单元写写完之后
/// （第一次空发布连根落在盘 0，盘 1 上最新的根还是写行那次 txg 4），丢掉住着现行那一版（txg 5）的根的盘 0。剩下的盘 1 整读第一个文件（3000 字节）。
/// 判别力：卸载那一格回收的槽整截当场放开（丙1）时，第二次空发布把单元写进盘 1 上最新的根还引用、由暖机那次释放的槽，
/// 盘 1 只读挂载走树读不出（改回丙1 的拷贝上量过 `UnitUnreadable { slot: SlotNumber(1419) }`），最后一项红。
#[test]
fn after_the_second_unmount_publish_of_a_narrow_pool_remounted_once_a_lone_surviving_device_reads_back_its_live_file(
) {
    let pool = narrow_pool_remounted_once_after_its_first_file();
    let survivor = lone_survivor_after_the_unit_writes_of_an_unmount_publish(
        &pool,
        HistoryDeviceWidth::UnitAreaOf384Slots,
        &pool.session().allocator,
        PublishOfTheUnmountSequence(1),
        NamedRootRingSlots::NONE,
    );
    assert!(
        survivor
            .before_the_unmount
            .as_ref()
            .is_ok_and(|whole_read| !whole_read.live_files.is_empty()),
        "卸载之前剩下那块盘读得出活文件（这一形要判的就是活文件不为空）：{:?}",
        survivor.before_the_unmount
    );
    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(&survivor);
}

/// 卸载这一遍有一块盘一条有效根都读不出时，这一串回收的槽整截扣到生效（D16（发布语义） 已定项 1「正常卸载」那一行「有一块盘一条有效根都读不出时这一截为空」）。
/// 同上一形的池与刀；卸载用的分配器是改写本（[`allocator_not_knowing_the_root_ring_slots_of`]：这个进程不知道盘 1 的根环槽住着根），
/// 两次卸载里盘 1 那几个根环槽每次读都失败，于是这一遍盘 1 上一条有效根都读不出。盘上的内容不动：掉电之后剩下的盘 1 照旧择到 txg 4。
/// 经可写挂载走不到这一格（暖机在每块盘上都写了这个进程的根，那几个槽读坏就重读、仍坏就拒），这里用改写本强制进入（`.claude/rules/fs-design.md`
/// 五条硬要求第 2 条）。
/// 判别力：这一格改成照原门槛当场放开时，第二次空发布把单元写进盘 1 上最新的根还引用的槽，盘 1 只读挂载读不出，最后一项红。
#[test]
fn when_no_valid_root_on_device_one_is_read_during_the_unmount_every_reclaimed_slot_is_held_and_device_one_reads_back_its_live_file(
) {
    let pool = narrow_pool_remounted_once_after_its_first_file();
    let survivor = lone_survivor_after_the_unit_writes_of_an_unmount_publish(
        &pool,
        HistoryDeviceWidth::UnitAreaOf384Slots,
        &allocator_not_knowing_the_root_ring_slots_of(&pool, DeviceIdentity(1)),
        PublishOfTheUnmountSequence(1),
        every_root_ring_slot_of(&pool, DeviceIdentity(1)),
    );
    assert_eq!(
        survivor.lost_device,
        DeviceIdentity(0),
        "丢掉的是住着现行那一版的根的盘 0，剩下的是这一遍读不出根的盘 1"
    );
    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(&survivor);
}
