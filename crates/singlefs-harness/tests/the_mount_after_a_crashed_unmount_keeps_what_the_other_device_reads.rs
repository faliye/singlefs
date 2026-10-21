//! 卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后的下一次可写挂载只放开一截（D16（发布语义） 已定项 1「正常卸载」那一行，
//! 用户 2026-10-04 选甲，`research/prompts/m3-batch14-code-r1-forks.md`「甲 再挂也只放开一截」那一行）：重建分配器时当场可用的只到
//! min(F_生效, 每块盘上最新有效根的 txg 取最小)，某块盘一条有效根都读不出时只到一圈窗口里最旧有效根（用户 2026-10-04 选甲，
//! `research/prompts/m3-batch16-code-r1-forks.md`「甲 只放到窗口里最旧有效根」那一行），其余扣到这次挂载的暖机在每块盘上都落了根（`mount::rebuilt_allocator`），
//! 暖机落满之后放开（`mount::establish_instance`）。
//! 形 R（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行，攻方 `research/prompts/m3-batch14-code-r1-opus-output.md`
//! 第一节 1.2）：卸载那一串只落了第一个系统配置写就崩 → 两块盘都在、再可写挂载 → 这次挂载取号、写行的单元写到一半再崩 → 丢掉盘 0。
//! 恢复择根不看 F，剩下那块盘择到它自己上面最新的根，低于系统配置里的新 F；判剩下那块盘只读挂载、整读每个活文件，与卸载之前同一块盘上读出的相同。
//! 造法照攻方原型 `research/prompts/m3-batch14-code-r1-opus-model/probe/main.rs` 的 `narrow_start`、`plain_start`、`run_remount_on`（只抄造法）：
//! 原型按录制流取前缀刀，这里用故障注入让某一次写起每次写都报错，盘停在前 n 个写之后。池级 checker 在单盘镜像上判不了（判决同一行），这里判读回。

mod common_admission;

use common_admission::{
    content_of, plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    start_plain, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::BlockDevice;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_space_admission, mount_writable_with_test_only_switches,
    mount_writable_with_test_only_switches_and_deletion_switches, unmount, MountError, Mounted,
    ShadowLedger,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::root_ring::RootRingSlot;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, FirstFile, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
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

/// 只读挂载、把挂载态里每个活着的文件从头读到尾：走到的根 txg 与每个活文件（inode 号、全部字节）。删除中的 inode（nlink = 0）不读。
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

/// 整池从第 `first_failing_write` 次写起每次写都报错的注错计划（掉电那一形：盘停在之前那几个写之后）。
fn plan_failing_every_write_from(
    device_width: HistoryDeviceWidth,
    first_failing_write: u64,
) -> SharedFaultPlan {
    SharedFaultPlan::armed(
        device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(first_failing_write),
        },
    )
}

/// 两块盘包上同一份注错计划：整池从第 `first_failing_write` 次写起每次写都报错。
fn devices_failing_every_write_from(
    image: &MemoryPool,
    device_width: HistoryDeviceWidth,
    first_failing_write: u64,
) -> Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> {
    let plan = plan_failing_every_write_from(device_width, first_failing_write);
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect()
}

/// 注错的那几块盘上这会儿的盘面。
fn image_of(
    devices: &[(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)],
    device_size_in_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

/// 再可写挂载那一次挂载期间根环怎么读（只供测试的开关 `RootRingSlotTarget`）：照常读，或点名那块盘住的几个区域里每个根环槽每次读都报块设备错
/// （盘上内容不动；挂载交回之后的恢复与整读不经这一层、读得出：再挂这次挂载期间的瞬时读错那一形）。
#[derive(Clone, Copy, Debug)]
enum RootRingDuringTheRemount {
    ReadAsUsual,
    EverySlotOfTheDeviceFailsEveryRead(DeviceIdentity),
}

/// 再挂那一次的读注错计划：照常读的不注入、只数调用；点名一块盘的，那块盘住的几个区域里每个根环槽每次读都报错。
fn read_plan_of_the_remount(
    pool: &PoolUnderTest<SparseBlockDevice>,
    device_width: HistoryDeviceWidth,
    root_ring: RootRingDuringTheRemount,
) -> SharedFaultPlan {
    match root_ring {
        RootRingDuringTheRemount::ReadAsUsual => {
            SharedFaultPlan::unarmed(device_width.fixed_geometry())
        }
        RootRingDuringTheRemount::EverySlotOfTheDeviceFailsEveryRead(device) => {
            let slots_per_region = pool.parameters.geometry.root_ring_slots_per_region.count();
            let ring_slots_of_the_device: Vec<RootRingSlot> = (0..ROOT_RING_REGIONS)
                .filter(|region| {
                    pool.parameters.region_devices[usize::try_from(*region).expect("区域号")]
                        == device
                })
                .flat_map(|region| {
                    (0..slots_per_region).map(move |slot| RootRingSlot { region, slot })
                })
                .collect();
            SharedFaultPlan::armed(
                device_width.fixed_geometry(),
                FaultSchedule::every_read_of_named_root_ring_slots_fails(RootRingSlotTarget {
                    named_slots: NamedRootRingSlots::naming(&ring_slots_of_the_device),
                    region_devices: pool.parameters.region_devices,
                    fixed_structure_slot_spacing: pool
                        .parameters
                        .geometry
                        .fixed_structure_slot_spacing,
                }),
            )
        }
    }
}

/// 再挂那一次的盘：外层按读注错计划注（根环读不出那一形），里层按写注错计划注（掉电那一形）。
type RemountDevice = FaultInjectingBlockDevice<FaultInjectingBlockDevice<SparseBlockDevice>>;

/// 再可写挂载走哪个入口：窄盘与攻方原型同一种开关（准入判着、影子账开、删除批大小 1），普通盘走 `crash_and_mount_writable` 那一个。
#[derive(Clone, Copy, Debug)]
enum RemountEntry {
    WithDeletionBatchesOfOne,
    WithTheSpaceAdmissionJudged,
}

fn mount_writable_through<Device: BlockDevice>(
    entry: RemountEntry,
    pool: &PoolUnderTest<SparseBlockDevice>,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Result<Mounted, MountError> {
    match entry {
        RemountEntry::WithDeletionBatchesOfOne => {
            mount_writable_with_test_only_switches_and_deletion_switches(
                &pool.parameters,
                devices,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                deletion_switches_with_batches_of_one(),
            )
        }
        RemountEntry::WithTheSpaceAdmissionJudged => mount_writable_with_space_admission(
            &pool.parameters,
            devices,
            SpaceAdmission::JudgedByTheFormula,
        ),
    }
}

/// 再可写挂载取号写的系统配置槽写数：两轮 × 两块盘（`transaction::write_the_witnessed_acquisition`，甲' 取号写两轮）。按「再挂落下几个写」
/// 摆掉电点的用例要落到写行那次的单元写上，落下的写数从这几个写之后数起：取号写的轮数变了，掉电点跟着挪，不停在取号那几个写上。
const ACQUISITION_WRITES_OF_THE_REMOUNT: u64 = 4;

/// 形 R 两次崩溃、丢掉一块盘之后要比的几样。
struct LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
    current_txg_before_the_unmount: CheckpointTxg,
    /// 剩下那块盘上恢复择到的根（看 journal）。
    root_chosen_on_the_survivor: Option<CheckpointTxg>,
    /// 再挂那一次挂载期间读注错计划拦下的读（照常读的恒 0）。
    reads_failed_during_the_remount: usize,
    /// 再挂那一次挂载期间发下去的写调用（含掉电之后报错的那几次）。
    write_calls_of_the_remount: u64,
    before_the_unmount: Result<WholeReadOfTheLiveFiles, String>,
    after_the_crashes: Result<WholeReadOfTheLiveFiles, String>,
}

/// 在 `pool` 的会话上卸载，前 `unmount_writes_landed` 个写落下就崩；交回崩溃之后两块盘的盘面。
fn image_after_an_unmount_crashing_after(
    pool: &PoolUnderTest<SparseBlockDevice>,
    device_width: HistoryDeviceWidth,
    unmount_writes_landed: u64,
) -> MemoryPool {
    let image_before_the_unmount = pool.image();
    let session = pool.session.as_ref().expect("这一刻有可写会话");
    let mut unmount_devices = devices_failing_every_write_from(
        &image_before_the_unmount,
        device_width,
        unmount_writes_landed + 1,
    );
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
        "卸载在第 {} 个写上掉电、报错：{:?}",
        unmount_writes_landed + 1,
        unmounted.as_ref().ok().map(|_| "做成了")
    );
    image_of(
        &unmount_devices,
        image_before_the_unmount.device_size_in_bytes,
    )
}

/// 在 `pool` 的会话上卸载，前 `unmount_writes_landed` 个写落下就崩；崩溃镜像上两块盘都在、经 `remount_entry` 再可写挂载（这次挂载期间根环照
/// `root_ring_during_the_remount` 读），前 `remount_writes_landed` 个写落下又崩；丢掉 `lost_device`，卸载之前与两次崩溃之后各在剩下那块盘上整读一遍。
fn lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
    pool: &PoolUnderTest<SparseBlockDevice>,
    device_width: HistoryDeviceWidth,
    unmount_writes_landed: u64,
    remount_entry: RemountEntry,
    root_ring_during_the_remount: RootRingDuringTheRemount,
    remount_writes_landed: u64,
    lost_device: DeviceIdentity,
) -> LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
    let image_before_the_unmount = pool.image();
    let current_txg_before_the_unmount = pool
        .session
        .as_ref()
        .expect("这一刻有可写会话")
        .current
        .root()
        .checkpoint_txg;
    let after_the_unmount =
        image_after_an_unmount_crashing_after(pool, device_width, unmount_writes_landed);
    let write_plan = plan_failing_every_write_from(device_width, remount_writes_landed + 1);
    let read_plan = read_plan_of_the_remount(pool, device_width, root_ring_during_the_remount);
    let mut remount_devices: Vec<(DeviceIdentity, RemountDevice)> =
        plain_devices_on(&after_the_unmount)
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
    let remounted = mount_writable_through(remount_entry, pool, &mut remount_devices);
    assert!(
        remounted.is_err(),
        "再可写挂载在第 {} 个写上掉电、报错：{:?}",
        remount_writes_landed + 1,
        remounted.as_ref().ok().map(|_| "做成了")
    );
    drop(remounted);
    let after_the_remount = MemoryPool {
        devices: remount_devices
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
    let survivor_after_the_crashes = image_without(&after_the_remount, lost_device);
    LoneSurvivorAfterACrashedUnmountAndACrashedRemount {
        current_txg_before_the_unmount,
        root_chosen_on_the_survivor: recover(&survivor_after_the_crashes, JournalPolicy::Consult)
            .effective_root
            .map(|(_, checkpoint_txg)| checkpoint_txg),
        reads_failed_during_the_remount: read_plan.fired_count(),
        write_calls_of_the_remount: write_plan.calls().writes,
        before_the_unmount: whole_read_of_the_live_files(&image_without(
            &image_before_the_unmount,
            lost_device,
        )),
        after_the_crashes: whole_read_of_the_live_files(&survivor_after_the_crashes),
    }
}

/// 剩下那块盘择到的是它自己上面低于卸载之前现行那一版的根；卸载之前那块盘读得出；两次崩溃之后读出来的与卸载之前逐项相同。
fn assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
    survivor: &LoneSurvivorAfterACrashedUnmountAndACrashedRemount,
) {
    assert!(
        survivor
            .root_chosen_on_the_survivor
            .is_some_and(|chosen| chosen < survivor.current_txg_before_the_unmount),
        "剩下那块盘择到它自己上面早于卸载之前现行那一版（txg {:?}）的根：{:?}",
        survivor.current_txg_before_the_unmount,
        survivor.root_chosen_on_the_survivor
    );
    assert!(
        survivor.before_the_unmount.is_ok(),
        "卸载之前剩下那块盘只读挂载、整读每个活文件做得成：{:?}",
        survivor.before_the_unmount
    );
    assert_eq!(
        survivor.after_the_crashes, survivor.before_the_unmount,
        "剩下那块盘走到的根 txg 与每个活文件的内容与卸载之前同一块盘上读出的相同：再挂那一次当场放开的槽没有一个被它最新的根引用"
    );
}

/// h2（攻方第一节 1.2）：不补环窗口、准入关着造的窄盘（单元区 176 槽），不先覆盖写，第一个文件顺序写长到 4 个单元，在这个进程里发 unlink 的意图与
/// 3 批（批大小 1）就崩；重开可写（准入判着、影子账开、批大小 1）续做走完。卸载只落下第一个写（盘 0 的系统配置带新 F）就崩；再可写挂载落下
/// 8 个写（取号两轮四个系统配置写、写行那次的前 4 个单元写）又崩；丢盘 0。
/// 判别力：重建分配器照原门槛 max(F_生效, 窗口里最旧有效根) 整截当场放开时，写行那次的单元写进盘 1 上最新的根（txg 13 < F_生效 14）还引用的槽，
/// 盘 1 只读挂载走树读不出（攻方原型上量过 `MappingStillUnreadable { slot: SlotNumber(1411) }`），最后一项红。
#[test]
fn after_a_crashed_unmount_of_a_narrow_pool_whose_deletion_was_continued_the_next_writable_mount_crashing_in_its_row_publish_keeps_what_device_one_reads(
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
        &lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
            &pool,
            device_width,
            1,
            RemountEntry::WithDeletionBatchesOfOne,
            RootRingDuringTheRemount::ReadAsUsual,
            ACQUISITION_WRITES_OF_THE_REMOUNT + 4,
            DeviceIdentity(0),
        ),
    );
}

/// 普通盘（攻方第一节 1.2 普通盘族 `plain W=4 k=1 m=1 k2=1`）：4 GiB 盘（随机历史默认那一档），第一个文件之后覆盖写 1 次、崩了再挂 1 次、
/// 再覆盖写 1 次。卸载只落下第一个写就崩；再可写挂载（准入判着）落下 35 个写（取号两轮四个系统配置写之后再 31 个写）又崩；丢盘 0。
/// 判别力：重建分配器整截当场放开时，再挂那一次的单元写进盘 1 上最新的根还引用的槽，盘 1 只读挂载做得成、打开活文件时 extent 树走不下去
/// （攻方原型上量过 `ExtentTreeWalk(MappingStillUnreadable { slot: SlotNumber(50253) })`），最后一项红。
#[test]
fn after_a_crashed_unmount_of_a_four_gibibyte_pool_remounted_once_the_next_writable_mount_crashing_thirty_one_writes_after_its_acquisition_keeps_what_device_one_reads(
) {
    let device_width = HistoryDeviceWidth::FourGibibytes;
    let mut pool = start_plain(device_width);
    pool.overwrite(OVERWRITE_BYTES)
        .unwrap_or_else(|refusal| panic!("第一个文件之后覆盖写 1 次：{refusal:?}"));
    pool.crash_and_mount_writable()
        .unwrap_or_else(|error| panic!("崩了再挂：{error:?}"));
    pool.overwrite(OVERWRITE_BYTES)
        .unwrap_or_else(|refusal| panic!("再挂之后覆盖写 1 次：{refusal:?}"));
    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
        &lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
            &pool,
            device_width,
            1,
            RemountEntry::WithTheSpaceAdmissionJudged,
            RootRingDuringTheRemount::ReadAsUsual,
            ACQUISITION_WRITES_OF_THE_REMOUNT + 31,
            DeviceIdentity(0),
        ),
    );
}

/// 攻方 `research/prompts/m3-batch16-code-r1-opus-output.md` 第 1.3 节那一段（判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节
/// 「再挂期间某块盘根环整段读坏」那一行）：不补环窗口、准入关着造的窄盘（单元区 264 槽），第一个文件之后覆盖写 6 次、顺序写长到 4 个单元，
/// 在这个进程里发 unlink 的意图就崩；重开可写（准入判着、影子账开、批大小 1）续做走完。卸载落下 15 个写就崩（两块盘的系统配置都带了新 F、
/// 带新 F 的根一条都没落）；两块盘都在、再可写挂载，这次挂载期间盘 1 的根环每一槽每次读都报错（挂载之后读得出），落下 8 个写又崩
/// （取号两轮四个系统配置写、写行那次的前 4 个单元写）；丢盘 0。造法照攻方原型 `research/prompts/m3-batch16-code-r1-opus-model/probe/main.rs` 的 `narrow_start` 与 `HIDE_DEV`
/// （原型读回全 0，这里报读错；攻方量过两种结局同数）。
/// 判别力：重建分配器时盘 1 一条有效根都读不出（m 取不到），照原门槛 max(F_生效, 窗口里最旧有效根) 整截当场放开时，写行那次的单元写进
/// 盘 1 上最新的根（txg 19 < F_生效 20）还引用的映射树节点，盘 1 只读挂载走树读不出（攻方原型上量过
/// `MappingStillUnreadable { slot: SlotNumber(1410) }`，另一刀 1412），最后一项红；当场只放到窗口里最旧有效根时读得出。
#[test]
fn after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_reading_no_root_on_device_one_and_crashing_in_its_row_publish_keeps_what_device_one_reads(
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let overwrites_before_the_file_grows = 6;
    let data_units_of_the_file = 4;
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            device_width,
            264,
        );
    for overwrite_index in 0..overwrites_before_the_file_grows {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
        });
    }
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
        let current = session
            .current
            .version_of_trees()
            .expect("第一个文件之后有树")
            .clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            *write_time_seconds,
            session.instance,
        )
        .expect("意图那次");
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
    let remount_writes_landed = ACQUISITION_WRITES_OF_THE_REMOUNT + 4;
    let survivor = lone_survivor_after_a_crashed_unmount_and_a_crashed_remount(
        &pool,
        device_width,
        15,
        RemountEntry::WithDeletionBatchesOfOne,
        RootRingDuringTheRemount::EverySlotOfTheDeviceFailsEveryRead(DeviceIdentity(1)),
        remount_writes_landed,
        DeviceIdentity(0),
    );
    assert!(
        survivor.reads_failed_during_the_remount > 0,
        "再挂那一次挂载期间盘 1 的根环读真被拦下过"
    );
    assert!(
        survivor.write_calls_of_the_remount > remount_writes_landed,
        "再挂那一次挂载走到了第 {} 个写（掉电那一个），不是在任何写之前被拒：发下去的写调用 {}",
        remount_writes_landed + 1,
        survivor.write_calls_of_the_remount
    );
    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(&survivor);
}

/// 暖机落满之后放开重建时扣住的那一截（`mount::establish_instance` 暖机之后那一句 `release_reclaim_holds`，判决
/// `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「扣住的那一截放开得对不对」那一行）：4 GiB 盘，第一个文件之后覆盖写 1 次、
/// 崩了再挂 1 次、再覆盖写 1 次；卸载只落下第一个写就崩（盘 0 的系统配置带新 F = 现行那一版的 txg，盘 1 上最新的根比它低），两块盘都在、
/// 再可写挂载（影子账关：没有隔离位）做成。挂载交回时分配器每块盘上的空闲槽数与三种位都没置的槽数相同：一个扣住位都没留下。
/// 判别力：删掉暖机之后那一句放开，重建时释放代高于 m、不高于 F_生效 的那一截（上一次覆盖写换下的槽）留着扣住位交给会话，最后一项红。
/// 用户看得到的结局在窄盘上造出来了，见
/// [`after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk_the_first_write_of_the_next_writable_mount_regrows_it_without_raising_the_floor`]。
#[test]
fn after_a_crashed_unmount_of_a_four_gibibyte_pool_the_next_writable_mount_hands_over_no_held_slot_once_its_warm_up_landed_on_every_device(
) {
    let device_width = HistoryDeviceWidth::FourGibibytes;
    let mut pool = start_plain(device_width);
    pool.overwrite(OVERWRITE_BYTES)
        .unwrap_or_else(|refusal| panic!("第一个文件之后覆盖写 1 次：{refusal:?}"));
    pool.crash_and_mount_writable()
        .unwrap_or_else(|error| panic!("崩了再挂：{error:?}"));
    pool.overwrite(OVERWRITE_BYTES)
        .unwrap_or_else(|refusal| panic!("再挂之后覆盖写 1 次：{refusal:?}"));
    let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, 1);
    let mut remount_devices = plain_devices_on(&after_the_unmount);
    let mounted = mount_writable_with_test_only_switches(
        &pool.parameters,
        &mut remount_devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::Off,
    )
    .unwrap_or_else(|error| panic!("卸载半途崩溃之后两块盘都在、再可写挂载：{error:?}"));
    let (_, session) = MountedSession::of_the_mount(mounted);
    for device_map in &session.allocator.devices {
        assert_eq!(
            (device_map.isolated_slots(), device_map.free_slots()),
            (0, device_map.unblocked_slots()),
            "盘 {:?}：影子账关着没有隔离位；空闲的槽每一个都发得出去，重建时扣住的那一截在暖机落满之后全放开了",
            device_map.device
        );
    }
}

/// 现行那一版换下的槽（释放代就是那一版的 txg）。卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后 F_生效 是卸载之前现行那一版的
/// txg；盘 1 单独恢复择到的根可以就是那一版，也可以早它一个或两个 txg：攻方在单元区 176 槽的窄盘上覆盖写 K = 0..6 次量过，卸载只落下第一个写时
/// K = 1、4 就是现行那一版，K = 0、2、5 早一个，K = 3、6 早两个；覆盖写 3 次、卸载落下 15 个写那一形也早两个（`research/prompts/m3-batch21-code-r1-opus-output.md`
/// 第二节「放开扫」与「改法 G5」两段）。早它的时候盘 1 上那条根还引用这几个槽，重建分配器时回收、扣到这次挂载的暖机落满每块盘
/// （`mount::rebuilt_allocator`）；早两个 txg 时中间那一版换下的槽同样扣着，只看现行那一版换下的判不出「只扣现行那一版换下的」那一刀，见
/// [`after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to`]。
fn slots_released_by_the_current_version(pool: &PoolUnderTest<SparseBlockDevice>) -> Vec<u64> {
    pool.current_file_version()
        .released
        .iter()
        .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
        .collect()
}

/// 放开挪到暖机之前那一刀（出处：第 17 批正文 `research/prompts/_m3-batch17-code-r1-body.md`「「放开挪到暖机之前」那一形没有用例钉」）：
/// 不补环窗口、准入关着造的窄盘（单元区 176 槽），第一个文件之后不再写；卸载只落下第一个写就崩，两块盘都在、再可写挂载（准入判着、影子账开、
/// 批大小 1）做成。写行那次与暖机每一次写下的单元一个都不落在卸载之前现行那一版换下的槽上：那几个槽扣到暖机落满每块盘才放开。
/// 判别力：`mount::establish_instance` 里那一句 `release_reclaim_holds` 挪到暖机之前时，暖机那次的单元写进换下的槽 1410（盘 1 上最新的根还引用它）：
/// 取号之前在分配器拷贝上预演的落点与真发的不同，挂载在预演对真发那一判上 panic，这一条红；那一判不在时，最后一项红
/// （在副本里关掉那一判量过：暖机落在换下的槽 [1410] 上）。只读挂载丢盘的那几条都在写行那次就崩，暖机没跑到，判不出这一刀。
/// 卸载只落下第一个写，这次挂载的暖机只推一次：「第一次暖机之后就放开」与「暖机落满之后放开」在这里是同一刻，判不出，那一刀见
/// [`after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to`]。
#[test]
fn after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_until_its_warm_up_landed_on_every_device(
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            device_width,
            176,
        );
    let released_before_the_unmount = slots_released_by_the_current_version(&pool);
    assert!(
        !released_before_the_unmount.is_empty(),
        "卸载之前现行那一版换下过槽（第一个文件那次换下上一版的单元）"
    );
    let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, 1);
    let mut remount_devices = plain_devices_on(&after_the_unmount);
    let mounted = mount_writable_through(
        RemountEntry::WithDeletionBatchesOfOne,
        &pool,
        &mut remount_devices,
    )
    .unwrap_or_else(|error| panic!("卸载半途崩溃之后两块盘都在、再可写挂载：{error:?}"));
    assert!(
        !mounted.output.warm_up_publishes.is_empty(),
        "这次挂载在写行之后推了暖机"
    );
    let slots_taken_into_the_released_ones: Vec<u64> = std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .filter_map(|version| version.version_of_trees())
        .flat_map(|version| {
            version
                .units
                .iter()
                .flat_map(|unit| unit.slot.0..unit.slot.0 + unit.identity.span_slots())
        })
        .filter(|slot| released_before_the_unmount.contains(slot))
        .collect();
    assert_eq!(
        slots_taken_into_the_released_ones,
        Vec::<u64>::new(),
        "写行那次与暖机写下的单元不落在卸载之前现行那一版换下的槽 {released_before_the_unmount:?} 上"
    );
}

/// 第一个文件之后、卸载之前覆盖写几次（攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`「N3」一节 K = 3 那份历史）。
const OVERWRITES_BEFORE_AN_UNMOUNT_WHOSE_NEXT_MOUNT_WARMS_UP_TWICE: usize = 3;

/// 卸载落下这么多个写就崩：两块盘的系统配置都带了新 F，第一次抬 F 空发布的 journal 记录已落、它的根没落；这时下一次可写挂载的暖机要推两次
/// （攻方同一节：卸载崩在第 15、16 个写之后暖机推两次）。
const UNMOUNT_WRITES_LANDED_BEFORE_A_MOUNT_THAT_WARMS_UP_TWICE: u64 = 15;

/// 放开挪到第一次暖机之后那一刀（判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节「K2 N3 / N4 错实现漏红」那一行，
/// 攻方同一份报告改法表 G1）：不补环窗口、准入关着造的窄盘（单元区 176 槽），第一个文件之后覆盖写 3 次；卸载落下 15 个写就崩，两块盘都在、
/// 再可写挂载（准入判着、影子账开、批大小 1）做成，这次挂载的暖机推两次。卸载崩溃镜像上盘 1 单独恢复择到的根早卸载之前现行那一版两个 txg
/// （判决 `research/prompts/m3-batch21-code-r1-main-verification.md` 第二节「K2 N5 卸载崩点与覆盖写次数放开扫」那一行），它之后的两版
/// （最后两次覆盖写）换下的槽释放代都高于盘 1 上最新的根、不高于 F_生效：写行那次与两次暖机每一次写下的单元一个都不落在这两版换下的槽上，
/// 它们扣到暖机在每块盘上都落了根才放开（D16（发布语义） 已定项 1「正常卸载」那一行当场可用的只到 min(F_生效, 每块盘上最新有效根的 txg 最小者)，
/// `mount::rebuilt_allocator` 扣、`mount::establish_instance` 暖机循环之后放开）。
/// 判别力：第一次暖机之后就放开、取号之前的预演同样在那里放开（预演对真发那一判不分叉）时，第二次暖机的单元写进换下的槽，最后一项红
/// （攻方原型上量过：这份历史写进换下的槽 2 个，`research/prompts/m3-batch19-code-r1-opus-model/out/n34-176-3-1-a036.txt` 里
/// `PREFIX S=176 K=3 U=1 n=15 state=4104 arm=6` 那一行的 `inR=2`；丢盘 0 之后剩下那块盘读文件报 `DataUnitChecksumMismatchEverywhere`）。只在真发那一处放开、预演不放开时，挂载在预演对真发那一判上
/// panic，这一条同样红。卸载只落下第一个写的那几条暖机只推一次，「第一次暖机之后」与「暖机落满之后」是同一刻，判不出这一刀。
/// 重建时当场放开到 min(F_生效, max(每块盘上最新有效根的 txg 最小者, F_生效 − 1))、只扣现行那一版换下的（攻方
/// `research/prompts/m3-batch21-code-r1-opus-model/patches/w2.patch`）时，写行那次的单元写进倒数第二次覆盖写换下的槽，最后一项红；
/// 只比现行那一版换下的槽时这一刀照绿（攻方量过：丢盘 0 之后盘 1 读文件报 `DataUnitChecksumMismatchEverywhere`）。
#[test]
fn after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to(
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            device_width,
            176,
        );
    let mut slots_released_by_every_version: Vec<(CheckpointTxg, Vec<u64>)> = vec![(
        pool.session().current.root().checkpoint_txg,
        slots_released_by_the_current_version(&pool),
    )];
    for overwrite_index in 0..OVERWRITES_BEFORE_AN_UNMOUNT_WHOSE_NEXT_MOUNT_WARMS_UP_TWICE {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
        });
        slots_released_by_every_version.push((
            pool.session().current.root().checkpoint_txg,
            slots_released_by_the_current_version(&pool),
        ));
    }
    let current_txg_before_the_unmount = pool.session().current.root().checkpoint_txg;
    let after_the_unmount = image_after_an_unmount_crashing_after(
        &pool,
        device_width,
        UNMOUNT_WRITES_LANDED_BEFORE_A_MOUNT_THAT_WARMS_UP_TWICE,
    );
    let newest_root_on_device_one = recover(
        &image_without(&after_the_unmount, DeviceIdentity(0)),
        JournalPolicy::Consult,
    )
    .effective_root
    .map(|(_, checkpoint_txg)| checkpoint_txg)
    .expect("卸载崩溃镜像上盘 1 单独恢复择得到根");
    let slots_released_by_every_version_after_the_newest_root_on_device_one: Vec<(
        CheckpointTxg,
        Vec<u64>,
    )> = slots_released_by_every_version
        .into_iter()
        .filter(|(released_by, _)| *released_by > newest_root_on_device_one)
        .collect();
    assert_eq!(
        slots_released_by_every_version_after_the_newest_root_on_device_one
            .iter()
            .map(|(released_by, slots)| (*released_by, slots.is_empty()))
            .collect::<Vec<_>>(),
        vec![
            (CheckpointTxg(current_txg_before_the_unmount.0 - 1), false),
            (current_txg_before_the_unmount, false),
        ],
        "盘 1 上最新的根（txg {newest_root_on_device_one:?}）早卸载之前现行那一版两个 txg，它之后的两版是最后两次覆盖写、各自换下过槽"
    );
    let released_after_the_newest_root_on_device_one: Vec<u64> =
        slots_released_by_every_version_after_the_newest_root_on_device_one
            .into_iter()
            .flat_map(|(_, slots)| slots)
            .collect();
    let mut remount_devices = plain_devices_on(&after_the_unmount);
    let mounted = mount_writable_through(
        RemountEntry::WithDeletionBatchesOfOne,
        &pool,
        &mut remount_devices,
    )
    .unwrap_or_else(|error| panic!("卸载半途崩溃之后两块盘都在、再可写挂载：{error:?}"));
    assert_eq!(
        mounted.output.warm_up_publishes.len(),
        2,
        "这次挂载在写行之后推了两次暖机"
    );
    let slots_taken_into_the_released_ones_by_every_publish: Vec<Vec<u64>> =
        std::iter::once(&mounted.output.row_publish)
            .chain(&mounted.output.warm_up_publishes)
            .map(|version| {
                version
                    .version_of_trees()
                    .map(|version| {
                        version
                            .units
                            .iter()
                            .flat_map(|unit| unit.slot.0..unit.slot.0 + unit.identity.span_slots())
                            .filter(|slot| {
                                released_after_the_newest_root_on_device_one.contains(slot)
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .collect();
    assert_eq!(
        slots_taken_into_the_released_ones_by_every_publish,
        vec![Vec::<u64>::new(); 3],
        "写行那次、第一次暖机、第二次暖机写下的单元（按这个次序一次一项）都不落在盘 1 上最新的根之后每一版换下的槽 \
         {released_after_the_newest_root_on_device_one:?} 上"
    );
}

/// 第一个文件先长到这么多个数据单元、再缩成一个单元：缩的那次换下的槽够多，扣着它们第一次写就长不回去（单元区 176 槽）。
const DATA_UNITS_OF_THE_FILE_BEFORE_IT_IS_SHRUNK: usize = 36;

/// 暖机之后放开那一句用户看得到的结局（判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第三节第 4 条：`admission.rs`
/// `PlacementRoomOnOneDevice` 读落点空间时扣住位算挡着）：不补环窗口、准入关着造的窄盘（单元区 176 槽），第一个文件顺序写长到 36 个单元、
/// 再覆盖写缩成一个单元（换下 36 个数据单元，释放代是这一版的 txg）；卸载只落下第一个写就崩，两块盘都在、再可写挂载（准入关着、影子账开）做成。
/// 挂载交回之后第一次写把文件顺序写回 36 个单元：一串抬 F 都不推、只发成这一版，F 停在挂载那一刻的值。
/// 判别力：删掉 `mount::establish_instance` 暖机之后那一句 `release_reclaim_holds` 时，缩的那次换下的槽留着扣住位交给会话，落点取不到，
/// 这次写拒成空间不够（`NoSpaceAfterRaisingTheFloor`，副本里量过：单元区 176 槽上文件长到 32–40 个单元的每一点都是这个结局，不删时都写成、不推抬 F），这一条红。
/// 这个结局只在准入关着时走得到：这块 176 槽的窄盘照式子判可用本来就是负的，准入开着时第一次写在准入处就被拒（`SpaceAdmissionRefused`），
/// 走不到落点那一判（正推 `research/prompts/m3-batch19-code-r1-sonnet-output.md` 第 4 节「范围」那一条在基线上量过）；所以挂载与第一次写都关着准入。
/// 判别力的边在文件长到多少个单元（攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`「N4」一节，单元区 176、192、240 槽上扫过 8–52 个单元）：
/// 长到 24 个单元以内、或单元区 240 槽时删掉那一句照样写成（不扣也放得下）；30 个单元只在暖机推两次的 3 个崩溃态上红（这一条卸载只落下第一个写，
/// 暖机推一次，30 判不出）；这里取的 36 在会红的区里，下沿在 30 与 36 之间（31–35 攻方没量）。哪一次改动让每版元数据少占十几个槽，
/// 这一条在删掉那一句时会悄悄变绿；4 GiB 那条直接数扣住位，不靠落点取不取得到，照样红。
/// 最后那一项（推了几串抬 F、发成几版、F 动没动）另有一刀：顺序写那一版的根顺手把 F 带成上一版的 txg（不推抬 F 那一串就挪 F）时这次写照样写成，
/// 只有 F 那一样不同，那一项红。
#[test]
fn after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk_the_first_write_of_the_next_writable_mount_regrows_it_without_raising_the_floor(
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let length_of_the_grown_file =
        (DATA_UNITS_OF_THE_FILE_BEFORE_IT_IS_SHRUNK - 1) * data_unit_payload_capacity() + 100;
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            device_width,
            176,
        );
    pool.sequential_write(length_of_the_grown_file)
        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 36 个单元：{refusal:?}"));
    pool.overwrite(OVERWRITE_BYTES)
        .unwrap_or_else(|refusal| panic!("覆盖写把文件缩成一个单元：{refusal:?}"));
    let released_by_the_shrink = slots_released_by_the_current_version(&pool);
    assert!(
        released_by_the_shrink.len() >= 2 * DATA_UNITS_OF_THE_FILE_BEFORE_IT_IS_SHRUNK,
        "缩的那次换下了 36 个数据单元（每个两槽）：换下 {} 槽",
        released_by_the_shrink.len()
    );
    let after_the_unmount = image_after_an_unmount_crashing_after(&pool, device_width, 1);
    let mut remount_devices = plain_devices_on(&after_the_unmount);
    let mounted = mount_writable_with_test_only_switches(
        &pool.parameters,
        &mut remount_devices,
        SpaceAdmission::SkippedByTheTestOnlySwitch,
        ShadowLedger::On,
    )
    .unwrap_or_else(|error| panic!("卸载半途崩溃之后两块盘都在、再可写挂载：{error:?}"));
    let floor_of_the_mount = mounted.current.root().rollback_floor;
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let write_time_seconds = pool.write_time_seconds + 1;
    let regrown_content = content_of(length_of_the_grown_file, write_time_seconds);
    let published = session
        .publish_user_change(
            &mut remount_devices,
            UserChange::SequentialWrite(FirstFile {
                content: &regrown_content,
                write_time_seconds,
            }),
        )
        .unwrap_or_else(|refusal| {
            panic!("挂载交回之后第一次写把文件顺序写回 36 个单元：{refusal:?}")
        });
    assert_eq!(
        (
            published.floor_raises.len(),
            published.versions_published.len(),
            session.current.root().rollback_floor,
        ),
        (0, 1, floor_of_the_mount),
        "第一次写一串抬 F 都不推、只发成这一版，F 停在挂载那一刻的值"
    );
}
