//! harness 测试文件 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_admission::{
    content_of, plain_devices_on, start_plain, PoolUnderTest, OVERWRITE_BYTES,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish,
    unmount, MountError, RollbackError, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{FirstFile, PoolVersion, TransactionOutput};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, FlippedBit, InjectedFault, NamedRootRingSlots, RootRingSlotTarget,
    SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

pub const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;

pub type SparseDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

/// 管理员回退那一格的目标：第一个文件那一版（实例 1、txg 3），F = 0 时是候选。
pub const FIRST_FILE_VERSION: RollbackTarget = RollbackTarget {
    instance: InstanceGeneration(1),
    checkpoint_txg: CheckpointTxg(3),
};

/// 镜像里 `identity` 那块盘的一份拷贝（改它不动镜像）。
pub fn copy_of_device(image: &MemoryPool, identity: DeviceIdentity) -> SparseBlockDevice {
    let mut device =
        SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
    device.image = image
        .devices
        .get(&identity)
        .cloned()
        .expect("镜像里有这块盘");
    device
}

/// 盘表每一项的身份与盘体从哪份镜像的哪块盘拷来：`(身份, 镜像, 那份镜像里的哪块盘)`。
pub fn devices_from(bodies: [(DeviceIdentity, &MemoryPool, DeviceIdentity); 2]) -> SparseDevices {
    bodies
        .into_iter()
        .map(|(identity, image, body)| (identity, copy_of_device(image, body)))
        .collect()
}

/// 一组盘的拷贝（拿去可写挂载，原来那组不动）。
pub fn copies_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> SparseDevices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy =
                SparseBlockDevice::new(device.size_in_bytes(), PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

/// 这组盘（拷贝）上可写挂载的结局，错误按 `Debug` 交回（对照与入口之后的那一次逐字比）。
pub fn writable_mount_outcome_on_a_copy_of(
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Result<InstanceGeneration, String> {
    let mut copies = copies_of(devices);
    mount_writable(parameters, &mut copies)
        .map(|mounted| mounted.output.instance)
        .map_err(|error| format!("{error:?}"))
}

/// 第一个文件之后留一份镜像（旧快照），会话里再覆盖写两次（txg 4、5）：交回池、旧快照与此刻的镜像。
pub fn pool_with_a_snapshot_after_the_first_file(
) -> (PoolUnderTest<SparseBlockDevice>, MemoryPool, MemoryPool) {
    let mut pool = start_plain(DEVICE_WIDTH);
    let snapshot_after_the_first_file = pool.image();
    pool.overwrite(OVERWRITE_BYTES)
        .expect("会话里第一次覆盖写（txg 4）");
    pool.overwrite(OVERWRITE_BYTES)
        .expect("会话里第二次覆盖写（txg 5）");
    let now = pool.image();
    (pool, snapshot_after_the_first_file, now)
}

/// 会话此刻的现行那一版（第一个文件之后都带文件）：抬 F 与管理员回退要一份 `&mut TransactionOutput`。
pub fn file_version_of(current: &mut PoolVersion) -> &mut TransactionOutput {
    match current {
        PoolVersion::WithFile(current) | PoolVersion::WithTreesAndNoFile(current) => current,
        PoolVersion::WithoutFile(_) => panic!("第一个文件之后现行那一版带文件"),
    }
}

/// 一次覆盖写的改动（写入时刻接着池里的往后数）。
pub fn overwrite_after(
    pool: &PoolUnderTest<SparseBlockDevice>,
    steps_after: u64,
) -> (Vec<u8>, u64) {
    let write_time_seconds = pool.write_time_seconds + 1 + steps_after;
    (
        content_of(OVERWRITE_BYTES, write_time_seconds),
        write_time_seconds,
    )
}

/// 挂着之后收盘表的四个入口（D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口」）。
#[derive(Clone, Copy, Debug)]
pub enum EntryAfterTheMount {
    SessionPublish,
    NormalUnmount,
    FloorRaiseToTheAdmissionCeiling,
    AdministratorRollback,
}

pub const EVERY_ENTRY: [EntryAfterTheMount; 4] = [
    EntryAfterTheMount::SessionPublish,
    EntryAfterTheMount::NormalUnmount,
    EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
    EntryAfterTheMount::AdministratorRollback,
];

/// 四个入口各自被拒时的错误（[`go_through`] 做成交回 `None`）。
pub enum EntryRefusal {
    Session(UserChangeRefused),
    Mount(MountError),
    Rollback(RollbackError),
}

/// 管理员回退那个入口回退到哪一版（另三个入口不看它）。
#[derive(Clone, Copy, Debug)]
pub enum RollbackTargetOfTheEntry {
    /// 第一个文件那一版（[`FIRST_FILE_VERSION`]）。
    FirstFileVersion,
    /// 现行那一版的前一版（同一实例、txg 小一）：根环转过一圈之后第一个文件那一版的根槽已被覆盖、不再是候选，
    /// 前一版的根一直在根环里。
    VersionBeforeTheCurrent,
}

/// 拿池里那条会话，经 `entry` 把这组盘交进去一次（管理员回退回到第一个文件那一版）。
pub fn go_through<Device: BlockDevice>(
    entry: EntryAfterTheMount,
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Option<EntryRefusal> {
    go_through_rolling_back_to(
        entry,
        RollbackTargetOfTheEntry::FirstFileVersion,
        pool,
        devices,
    )
}

/// 拿池里那条会话，经 `entry` 把这组盘交进去一次；`entry` 是管理员回退时回退到 `rollback_target`。
pub fn go_through_rolling_back_to<Device: BlockDevice>(
    entry: EntryAfterTheMount,
    rollback_target: RollbackTargetOfTheEntry,
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Option<EntryRefusal> {
    let parameters = pool.parameters.clone();
    let (content, write_time_seconds) = overwrite_after(pool, 0);
    let session = pool.session.as_mut().expect("这一刻有可写会话");
    match entry {
        EntryAfterTheMount::SessionPublish => session
            .publish_user_change(
                devices,
                UserChange::Overwrite(FirstFile {
                    content: &content,
                    write_time_seconds,
                }),
            )
            .err()
            .map(EntryRefusal::Session),
        EntryAfterTheMount::NormalUnmount => unmount(
            &parameters,
            devices,
            &mut session.allocator,
            &mut session.current,
            ShadowLedger::On,
        )
        .map(|unmounted| match unmounted {
            Unmounted::FloorRaisedToTheCurrentVersion(_) => (),
            Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => {
                panic!("现行那一版带文件，卸载要推那一串")
            }
        })
        .err()
        .map(EntryRefusal::Mount),
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling => {
            let MountedSession {
                allocator,
                current,
                shadow_ledger,
                ..
            } = session;
            raise_rollback_floor_to_the_admission_ceiling(
                &parameters,
                devices,
                allocator,
                file_version_of(current),
                *shadow_ledger,
            )
            .err()
            .map(EntryRefusal::Mount)
        }
        EntryAfterTheMount::AdministratorRollback => {
            let MountedSession {
                allocator, current, ..
            } = session;
            let target = match rollback_target {
                RollbackTargetOfTheEntry::FirstFileVersion => FIRST_FILE_VERSION,
                RollbackTargetOfTheEntry::VersionBeforeTheCurrent => {
                    let current_root = file_version_of(current).root;
                    RollbackTarget {
                        instance: current_root.instance,
                        checkpoint_txg: CheckpointTxg(
                            current_root
                                .checkpoint_txg
                                .0
                                .checked_sub(1)
                                .expect("现行那一版在第一个文件之后，txg 大于 0"),
                        ),
                    }
                }
            };
            roll_back_by_a_forward_publish(
                &parameters,
                devices,
                allocator,
                file_version_of(current),
                target,
            )
            .err()
            .map(EntryRefusal::Rollback)
        }
    }
}

pub fn describe(refusal: &Option<EntryRefusal>) -> String {
    match refusal {
        None => "做成了".to_string(),
        Some(EntryRefusal::Session(refused)) => format!("{refused:?}"),
        Some(EntryRefusal::Mount(refused)) => format!("{refused:?}"),
        Some(EntryRefusal::Rollback(refused)) => format!("{refused:?}"),
    }
}

/// Y2-c：盘 1 两个系统配置槽里较新（世代号大）那一槽写坏一个字节，只剩上一次轮换那一槽自证得过——它落后于现行那一版，而现行那一版的单元
/// 都在盘 1 上。四个入口都照常做成（不误拒），做完之后池级 checker 不红；入口写过系统配置轮换的，之后照常可写挂载，
/// 一次轮换都没写的，那一槽还坏着，之后的可写挂载按 C331（择根倒挂压过已确认的写） 取甲拒。
pub fn a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry(
    images: &mut PoolCheckerImages,
) {
    for entry in EVERY_ENTRY {
        let (mut pool, _, now) = pool_with_a_snapshot_after_the_first_file();
        let mut devices = devices_from([
            (DeviceIdentity(0), &now, DeviceIdentity(0)),
            (DeviceIdentity(1), &now, DeviceIdentity(1)),
        ]);
        let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
        let newer_generation = verified_system_configuration_slots(
            &devices,
            DeviceIdentity(1),
            spacing,
            &pool.parameters.filesystem_identifier,
        )
        .into_iter()
        .map(|slot| slot.quantities.slot_generation)
        .max()
        .expect("盘 1 两槽自证得过");
        // 下一次写的槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16）：世代号 g 那一份住槽 g mod 2。
        let newer_slot_offset = DeviceOffsetInBytes((newer_generation % 2) * spacing);
        let device_one = &mut devices[1].1;
        let mut newer_slot = vec![0u8; 512];
        device_one
            .read_at(newer_slot_offset, &mut newer_slot)
            .expect("读较新那一槽");
        newer_slot[100] ^= 0xff;
        device_one
            .write_at(newer_slot_offset, &newer_slot, WriteDurability::Plain)
            .expect("写坏较新那一槽");
        assert_eq!(
            verified_system_configuration_slots(
                &devices,
                DeviceIdentity(1),
                spacing,
                &pool.parameters.filesystem_identifier,
            )
            .len(),
            1,
            "{entry:?}：盘 1 只剩一槽自证"
        );
        let refusal = go_through(entry, &mut pool, &mut devices);
        assert!(
            refusal.is_none(),
            "{entry:?}：只剩一槽自证的盘不误拒：{}",
            describe(&refusal)
        );
        let after = MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: now.device_size_in_bytes,
        };
        images.hand_out(
            &format!("{entry:?}：做完之后"),
            || after,
            PoolCheckerExpectation::no_invariant_violated(),
        );
        // 入口的写里有一次系统配置轮换就把写坏的那一槽重写了；入口一次轮换都没写（抬 F 到准入上限而没有可抬的），那一槽还自证不过——
        // 写坏的一槽读得出、自证不过，设备没报错：C331（择根倒挂压过已确认的写） 改取 R2 之后不判不出（用户 2026-10-03 定），
        // 两种都照常可写挂载（取甲时后一种拒）。
        let outcome = writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices);
        assert!(
            outcome.is_ok(),
            "{entry:?}：写坏的那一槽读得出、只是自证不过，R2 判得出，之后照常可写挂载：{outcome:?}"
        );
    }
}

/// 会话里那次覆盖写 P 的录制流切在哪：P 的根槽 FUA 写之前（单元与记录已写、根槽没写：C585 那一形），或之后（根已落、系统配置轮换还没写）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhereThePublishIsCut {
    BeforeItsRootSlotWrite,
    AfterItsRootSlotWrite,
}

/// 会话做完 P 之后的池（会话的现行那一版是 P），盘表里 P 的根住的那块盘换成「P 之前的盘面 + P 的录制流切下的那一截」，另一块是 P 做完的样子。
pub struct RootDeviceOfThePublishTakenMidPublish {
    pub pool: PoolUnderTest<SparseBlockDevice>,
    pub devices: SparseDevices,
    pub root_device_of_the_publish: DeviceIdentity,
    pub ring_slot_of_the_publish_root: RootRingSlot,
}

/// P 的 txg：起点（第一个文件之后会话里覆盖写两次，txg 5）之后的下一次，根环槽在区域 0（盘 0）、槽 2，那一槽 P 之前没写过。
pub const PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO: CheckpointTxg = CheckpointTxg(6);

/// 起点同 Y2-a（第一个文件之后会话里覆盖写两次，txg 5），会话里再覆盖写到 `publish_txg` 的前一版，P 是下一次覆盖写（txg `publish_txg`），
/// 录下它的写流。切下的那块盘：落后于 P（它的系统配置轮换在根槽之后），而 P 的单元都在它上面（单元与记录都在根槽之前写）——
/// 落后支缺单元那一半放它过去。这两样在这里先核，钉的是攻方那一形，不是别的形。
pub fn root_device_of_the_publish_taken_mid_publish(
    cut: WhereThePublishIsCut,
    publish_txg: CheckpointTxg,
) -> RootDeviceOfThePublishTakenMidPublish {
    let (mut pool, _, _) = pool_with_a_snapshot_after_the_first_file();
    let txg_before_the_extra_overwrites = pool.current_file_version().root.checkpoint_txg;
    for _ in txg_before_the_extra_overwrites.0 + 1..publish_txg.0 {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("会话里覆盖写到 P 的前一版");
    }
    let before_the_publish = pool.image();
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&before_the_publish)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                )
            })
            .collect();
    let (content, write_time_seconds) = overwrite_after(&pool, 0);
    pool.session
        .as_mut()
        .expect("这一刻有可写会话")
        .publish_user_change(
            &mut recorded,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds,
            }),
        )
        .expect("P：会话里第三次覆盖写");
    pool.write_time_seconds = write_time_seconds;
    let after_the_publish = MemoryPool {
        devices: recorded
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: before_the_publish.device_size_in_bytes,
    };
    pool.devices = plain_devices_on(&after_the_publish);
    let publish = pool.current_file_version().clone();
    assert_eq!(
        publish.root.checkpoint_txg, publish_txg,
        "P 是 txg {} 那一版",
        publish_txg.0
    );
    let geometry = pool.parameters.geometry;
    let ring_slot_of_the_publish_root = target_for_publish(
        publish.root.checkpoint_txg,
        geometry.root_ring_slots_per_region,
    );
    let root_device_of_the_publish = pool.parameters.region_devices
        [usize::try_from(ring_slot_of_the_publish_root.region).expect("区域号小于 3")];
    let root_slot_offset = slot_offset(
        ring_slot_of_the_publish_root,
        geometry.fixed_structure_slot_spacing,
    );
    let operations = stream.retained_operations();
    let root_slot_write = operations
        .iter()
        .position(|retained| {
            retained.operation.device == root_device_of_the_publish
                && retained.operation.offset == root_slot_offset
                && retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess
        })
        .expect("P 的录制流里有它的根槽 FUA 写");
    let operations_through_the_cut = match cut {
        WhereThePublishIsCut::BeforeItsRootSlotWrite => root_slot_write,
        WhereThePublishIsCut::AfterItsRootSlotWrite => root_slot_write + 1,
    };
    let mut cut_image = before_the_publish.clone();
    cut_image.apply(&operations[..operations_through_the_cut]);
    let devices: SparseDevices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            let image = if identity == root_device_of_the_publish {
                &cut_image
            } else {
                &after_the_publish
            };
            (identity, copy_of_device(image, identity))
        })
        .collect();
    let cut_device = &devices
        .iter()
        .find(|(identity, _)| *identity == root_device_of_the_publish)
        .expect("盘表里有 P 的根住的那块盘")
        .1;
    assert_ne!(
        cut_device.image, after_the_publish.devices[&root_device_of_the_publish],
        "{cut:?}：切下的那块盘与 P 做完的样子不同（系统配置轮换还没写）"
    );
    let spacing = u64::from(geometry.fixed_structure_slot_spacing);
    let newest_self_verified = verified_system_configuration_slots(
        &devices,
        root_device_of_the_publish,
        spacing,
        &pool.parameters.filesystem_identifier,
    )
    .into_iter()
    .max_by_key(|slot| slot.quantities.slot_generation)
    .expect("切下的那块盘上有自证过的系统配置");
    assert!(
        (
            newest_self_verified.quantities.journal_instance,
            newest_self_verified.quantities.journal_tail
        ) < (publish.record.instance, publish.record.counter),
        "{cut:?}：切下的那块盘落后于 P（系统配置停在 P 之前那次发布）"
    );
    for unit in publish.units.iter().filter(|unit| !unit.bytes.is_empty()) {
        let mut bytes_on_the_cut_device = vec![0u8; unit.bytes.len()];
        cut_device
            .read_at(unit.slot.to_device_offset(), &mut bytes_on_the_cut_device)
            .expect("读 P 的单元在切下的那块盘上的落点");
        assert_eq!(
            bytes_on_the_cut_device, unit.bytes,
            "{cut:?}：P 的单元 {:?} 在切下的那块盘上",
            unit.identity
        );
    }
    RootDeviceOfThePublishTakenMidPublish {
        pool,
        devices,
        root_device_of_the_publish,
        ring_slot_of_the_publish_root,
    }
}

/// 不误拒：P 的根住的那块盘切在 P 的根槽 FUA 之后（根已落、系统配置轮换没写）。它落后、单元都在、根槽里就是 P 的根——
/// 与「根落盘之后、轮换之前崩」同一个盘面，会话照常发布，做完之后池级 checker 不红。
pub fn a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused(
    images: &mut PoolCheckerImages,
) {
    let RootDeviceOfThePublishTakenMidPublish {
        mut pool,
        mut devices,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let refusal = go_through(EntryAfterTheMount::SessionPublish, &mut pool, &mut devices);
    assert!(
        refusal.is_none(),
        "根槽里是 P 的根：不误拒，实际 {}",
        describe(&refusal)
    );
    let after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: pool.device_bytes,
    };
    images.hand_out(
        "做完之后",
        || after,
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 缺根那一半第一次读 P 的根槽，在正常卸载、准入抬 F、管理员回退里是那一槽被读的第几次（从 1 数）：这三个入口在读根环之前先核盘表
/// （落后支在里面），第 1 次就是它（攻方 `research/prompts/m3-code-r2-opus-model/logs/` 里 txg 6 那一扫：原样代码第 1 次读坏照常做成、
/// 三个入口不重读时第 1 次读坏就拒成缺根）。
pub const ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_THE_OTHER_THREE_ENTRIES: u64 = 1;

/// 根槽这一次读回的字节坏的那一位：根记录里 checkpoint_txg 字段的头一个字节（魔数 4、文件系统标识 16、标志 4、实例代号 4 字节之后，
/// `crates/singlefs-core/src/root_record.rs` 的 `RootRecord::parse_slot`）的最低位。自证校验和罩着整个根槽，翻哪一位都自证不过；
/// 取 txg 字段是让「自证不过」不靠魔数那一判先拒。
pub const BIT_FLIPPED_IN_THE_CHECKPOINT_TXG_OF_THE_ROOT_SLOT: FlippedBit = FlippedBit {
    byte_index: 28,
    bit_index: 0,
};

/// 根槽这一次读报成功、读回的字节坏一位（L1：自证不过的那一种读坏）。
pub const ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT: InjectedFault =
    InjectedFault::ReadReturnsCorruptedBytes {
        flipped_bit: BIT_FLIPPED_IN_THE_CHECKPOINT_TXG_OF_THE_ROOT_SLOT,
    };

/// 盘表里每块盘包上同一份注入计划：P 的根环槽 `ring_slot` 上的读（整池数、只数落在这一槽里的读）按 `occurrence` 注入 `fault`
/// （报块设备错 [`InjectedFault::ReadFails`]，或读报成功而字节坏一位 [`ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT`]）。
pub fn with_a_fault_on_reads_of_the_publish_root_slot(
    devices: SparseDevices,
    pool: &PoolUnderTest<SparseBlockDevice>,
    ring_slot: RootRingSlot,
    fault: InjectedFault,
    occurrence: FaultOccurrence,
) -> (
    Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)>,
    SharedFaultPlan,
) {
    let plan = SharedFaultPlan::armed(
        DEVICE_WIDTH.fixed_geometry(),
        FaultSchedule {
            fault,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::WithinNamedRootRingSlots(RootRingSlotTarget {
                named_slots: NamedRootRingSlots::NONE.with(ring_slot),
                region_devices: pool.parameters.region_devices,
                fixed_structure_slot_spacing: pool.parameters.geometry.fixed_structure_slot_spacing,
            }),
            counting: FaultCounting::AcrossThePool,
            occurrence,
        },
    );
    let wrapped = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    (wrapped, plan)
}

/// J2 的一格（攻方改错 RP「正常卸载、准入抬 F、管理员回退不重读」）：P 的根住的那块盘切在 P 的根槽 FUA 之后（落后、单元都在、
/// 根槽里是 P），经 `entry` 交进去时缺根那一半第一次读那一槽读坏（按 `fault`：读不出，或读回的字节坏一位、自证不过）、
/// 重读读得出 P：照常做成，那一次注入真发生了，做完之后池级 checker 不红。
/// 这三个入口不重读（第一次读不是它就拒）时两种读坏都拒成落后支缺根那一半；只在第一次读不出时才重读（攻方 RU）时字节坏一位的那一种拒。
pub fn one_bad_read_of_the_publish_root_slot_through(
    images: &mut PoolCheckerImages,
    entry: EntryAfterTheMount,
    fault: InjectedFault,
) {
    let RootDeviceOfThePublishTakenMidPublish {
        mut pool,
        devices,
        ring_slot_of_the_publish_root,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let (mut wrapped, plan) = with_a_fault_on_reads_of_the_publish_root_slot(
        devices,
        &pool,
        ring_slot_of_the_publish_root,
        fault,
        FaultOccurrence::TheNthMatchingCall(
            ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_THE_OTHER_THREE_ENTRIES,
        ),
    );
    let refusal = go_through(entry, &mut pool, &mut wrapped);
    assert_eq!(
        plan.fired_count(),
        1,
        "{entry:?}，{fault:?}：P 的根槽上那一次读坏真发生了：{:?}",
        plan.fired()
    );
    assert!(
        refusal.is_none(),
        "{entry:?}，{fault:?}：一次读坏、重读读得出 P：照常做成，实际 {}",
        describe(&refusal)
    );
    let after = MemoryPool {
        devices: wrapped
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: pool.device_bytes,
    };
    images.hand_out(
        &format!("{entry:?}，{fault:?}：做完之后"),
        || after,
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

pub fn a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
    images: &mut PoolCheckerImages,
) {
    one_bad_read_of_the_publish_root_slot_through(
        images,
        EntryAfterTheMount::NormalUnmount,
        InjectedFault::ReadFails,
    );
}

pub fn a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
    images: &mut PoolCheckerImages,
) {
    one_bad_read_of_the_publish_root_slot_through(
        images,
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        InjectedFault::ReadFails,
    );
}

pub fn an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
    images: &mut PoolCheckerImages,
) {
    one_bad_read_of_the_publish_root_slot_through(
        images,
        EntryAfterTheMount::AdministratorRollback,
        InjectedFault::ReadFails,
    );
}

/// L1 的一格：J2 正常卸载那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
pub fn a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
    images: &mut PoolCheckerImages,
) {
    one_bad_read_of_the_publish_root_slot_through(
        images,
        EntryAfterTheMount::NormalUnmount,
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}

/// L1 的一格：J2 准入抬 F 那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
pub fn a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
    images: &mut PoolCheckerImages,
) {
    one_bad_read_of_the_publish_root_slot_through(
        images,
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}

/// L1 的一格：J2 管理员回退那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
pub fn an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
    images: &mut PoolCheckerImages,
) {
    one_bad_read_of_the_publish_root_slot_through(
        images,
        EntryAfterTheMount::AdministratorRollback,
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}
