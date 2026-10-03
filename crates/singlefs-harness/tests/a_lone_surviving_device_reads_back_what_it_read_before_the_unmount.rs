//! 卸载那一串掉电之后只剩一块盘：剩下那块盘只读挂载、整读每个活文件，与卸载之前同一块盘上读出的逐项相同（走到的根 txg 与内容）。
//! 判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 2 条（GC）；用户 2026-10-03 定甲 GA
//! （`mount::RaiseFloorEntry::reuse_of_the_reclaimed_slots`：卸载那一格只把释放代不高于每块盘上最新有效根的那一截当场放开）。
//! 刀摆在卸载那一串第一次空发布的单元写写完之后（先写系统配置那两个写 + 那几个单元写落下，journal 记录与根都没落），丢掉住着现行那一版的根
//! 的那块盘。恢复择根不看 F（`recovery::choose_root`），剩下那块盘择到它自己上面最新的根，低于系统配置里的新 F。
//! 两形照攻方（`research/prompts/m3-unlink-code-r3-opus-output.md` 第一节 1.4）：h2 窄盘续做之后卸载，h4 4 GiB 盘卸载—再挂循环之后第二次卸载。
//! 池级 checker 在单盘镜像上卸载之前就红六七条（判决第二节 N1「第五节跑前判据的字面」那一行），这里不判 checker，判的是读回。

mod common_admission;

use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    start_plain, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, unmount, MountError,
    ShadowLedger, Unmounted,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_core::write_accounting::WrittenStructureKind;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 抬 F 那一串第一次发布之前先写系统配置那一步的写：每块盘一次，两块盘。
const SYSTEM_CONFIGURATION_WRITES_OF_THE_SEQUENCE: u64 = 2;

/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
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
struct LoneSurvivorAfterTheFirstUnmountPublish {
    /// 丢掉的盘：住着卸载之前现行那一版的根。
    lost_device: DeviceIdentity,
    current_txg_before_the_unmount: CheckpointTxg,
    /// 卸载那一串第一次空发布写了几个单元（不注错地在一份拷贝上卸载一次数出来的）。
    unit_writes_of_the_first_publish: u64,
    /// 剩下那块盘上恢复择到的根（看 journal）。
    root_chosen_on_the_survivor: Option<CheckpointTxg>,
    before_the_unmount: Result<WholeReadOfTheLiveFiles, String>,
    after_the_crash: Result<WholeReadOfTheLiveFiles, String>,
}

/// 在 `pool` 的会话上卸载：先在一份拷贝上不注错地卸载一次，数清那一串第一次空发布的单元写；再在另一份上从「先写系统配置那两个写 + 那几个单元写」
/// 之后的第一个写起每个写都报错（掉电那一形：盘停在那几个单元写之后，journal 记录与根都没落）；丢掉住着现行那一版的根的那块盘，
/// 卸载之前与掉电之后各在剩下那块盘上整读一遍。
fn lone_survivor_after_the_first_unmount_publish(
    pool: &PoolUnderTest<SparseBlockDevice>,
    device_width: HistoryDeviceWidth,
) -> LoneSurvivorAfterTheFirstUnmountPublish {
    let image_before_the_unmount = pool.image();
    let session = pool.session.as_ref().expect("这一刻有可写会话");
    let current_txg_before_the_unmount = session.current.root().checkpoint_txg;
    let unit_writes_of_the_first_publish = {
        let mut allocator = session.allocator.clone();
        let mut current = session.current.clone();
        let mut devices = plain_devices_on(&image_before_the_unmount);
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
        let first = &raised.publishes[0];
        let structure_writes = |kind: WrittenStructureKind| first.writes.of(kind).write_calls;
        first.writes.total().write_calls
            - structure_writes(WrittenStructureKind::JournalRecord)
            - structure_writes(WrittenStructureKind::RootSlot)
            - structure_writes(WrittenStructureKind::SystemConfigurationSlot)
    };
    let plan = SharedFaultPlan::armed(
        device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(
                SYSTEM_CONFIGURATION_WRITES_OF_THE_SEQUENCE + unit_writes_of_the_first_publish + 1,
            ),
        },
    );
    let mut failing_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&image_before_the_unmount)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, plan.clone()),
                )
            })
            .collect();
    let mut allocator = session.allocator.clone();
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
        "掉电之后那一串第一次空发布写 journal 记录时报错：{:?}",
        unmounted.as_ref().err()
    );
    let crashed = MemoryPool {
        devices: failing_devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
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
    LoneSurvivorAfterTheFirstUnmountPublish {
        lost_device,
        current_txg_before_the_unmount,
        unit_writes_of_the_first_publish,
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

/// 判一份 [`LoneSurvivorAfterTheFirstUnmountPublish`]：第一次空发布真写了单元；剩下那块盘择到的是它自己上面低于现行那一版的根；
/// 卸载之前那块盘读得出；掉电之后读出来的与卸载之前逐项相同。
fn assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
    survivor: &LoneSurvivorAfterTheFirstUnmountPublish,
) {
    assert!(
        survivor.unit_writes_of_the_first_publish > 0,
        "卸载那一串第一次空发布写了单元（这一刀要判的就是它们先于带新 F 的根写下去）"
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
        &lone_survivor_after_the_first_unmount_publish(&pool, device_width),
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
        &lone_survivor_after_the_first_unmount_publish(&pool, device_width),
    );
}
