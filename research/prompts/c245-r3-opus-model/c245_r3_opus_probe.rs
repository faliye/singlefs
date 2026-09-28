//! C245 第三轮云端攻方腿的探针（只在副本上跑，不入库装置）。
//! 甲 = 今天的代码（每次发布轮换系统配置、返回前等它持久）。乙、丙不改产品代码，改的是盘面：
//! 丙 = 发布返回之后、轮换还没持久时崩溃（录制流里拿掉那次发布的轮换写）；乙-朴素 = 取号之后的轮换写全都不存在（录制流里拿掉取号之后的全部系统配置写）。
//! 故障一律在内存盘上直接改字节：根槽清零、单元一份翻一个字节、系统配置槽清零。
//! 每条用例打印一行 `probe=<名> …`，并断言打中的结局；打中消失（被修掉）就红。

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    PublishedUnit, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SECTOR_BYTES};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream};

const IMAGE_BYTES: u64 = 4 << 30;
const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);

fn content_of(version: u64) -> Vec<u8> {
    let salt = usize::try_from(version).expect("小");
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + salt * 13 + 3) % 253).expect("小于 256"))
        .collect()
}

/// mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）→ 覆盖写 txg 4、txg 5。txg 5 那次就是被攻的「最后一次发布 P」。
struct History {
    parameters: MakeFilesystemParameters,
    operations: Vec<RetainedOperation>,
    /// 取号那两写落完（取号返回）时录制流的长度：乙-朴素只留这之前的系统配置写。
    acquisition_end: usize,
    /// txg 4 那次发布返回时录制流的长度：txg 5 的全部写都在它之后。
    before_last_publish: usize,
    /// 第一个文件（txg 3）返回时录制流的长度：「盘 1 停在 txg 3」的旧快照取到这里。
    after_the_first_file: usize,
    /// txg 3、4、5。
    versions: Vec<TransactionOutput>,
}

fn build_history() -> History {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|number| {
            let identity = DeviceIdentity(number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DEVICE_ZERO, IMAGE_BYTES),
        DeviceFreeMap::new(DEVICE_ONE, IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let acquisition_end = stream.operation_count();
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first = publish_first_file(
        &mut writer,
        &mut allocator,
        warmed.roots.last().expect("暖机两代根"),
        FirstFile { content: &first_file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("第一个文件");
    let mut versions = vec![first];
    let after_the_first_file = stream.operation_count();
    let mut before_last_publish = 0;
    for version in [2u64, 3] {
        before_last_publish = stream.operation_count();
        let previous = versions.last().expect("有上一版").clone();
        let next = publish_overwrite(
            &mut writer,
            &mut allocator,
            &previous,
            FirstFile {
                content: &content_of(version),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + version,
            },
            instance,
        )
        .expect("覆盖写");
        versions.push(next);
    }
    assert_eq!(versions[2].root.checkpoint_txg, CheckpointTxg(5));
    History {
        parameters,
        operations: stream.retained_operations(),
        acquisition_end,
        before_last_publish,
        after_the_first_file,
        versions,
    }
}

fn is_system_configuration_write(parameters: &MakeFilesystemParameters, retained: &RetainedOperation) -> bool {
    let slots_end =
        SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE * u64::from(parameters.geometry.fixed_structure_slot_spacing);
    matches!(retained.operation.kind, RecordedOperationKind::Write)
        && retained.operation.offset.0 < slots_end
}

/// 录制流按 `keep` 过滤之后施加到两块空内存盘上。
fn image_keeping(history: &History, keep: impl Fn(usize, &RetainedOperation) -> bool) -> MemoryPool {
    let kept: Vec<RetainedOperation> = history
        .operations
        .iter()
        .enumerate()
        .filter(|(index, retained)| keep(*index, retained))
        .map(|(_, retained)| retained.clone())
        .collect();
    let mut image = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    image.apply(&kept);
    image
}

/// 盘面的几种来路。
#[derive(Clone, Copy, Debug)]
enum DiskOrigin {
    /// 甲：整条流都持久。
    Full,
    /// 甲的崩溃窗口 / 丙返回之后崩溃：最后一次发布（txg 5）在 `devices` 上的轮换写没持久。
    LastRotationLostOn(&'static [DeviceIdentity]),
    /// 最后一次发布的根槽 FUA 之前崩：txg 5 的根槽写与轮换写都没持久（单元、记录已过屏障）。
    CrashBeforeTheLastRoot,
    /// 乙-朴素：取号之后的系统配置写一次都没有（系统配置只在取号、抬 F、卸载时写）。
    YiNaive,
    /// 乙-朴素之下最后一次发布的根槽 FUA 之前崩。
    YiNaiveCrashBeforeTheLastRoot,
}

fn image_of(history: &History, origin: DiskOrigin) -> MemoryPool {
    let parameters = &history.parameters;
    match origin {
        DiskOrigin::Full => image_keeping(history, |_, _| true),
        DiskOrigin::LastRotationLostOn(devices) => image_keeping(history, |index, retained| {
            !(index >= history.before_last_publish
                && is_system_configuration_write(parameters, retained)
                && devices.contains(&retained.operation.device))
        }),
        DiskOrigin::CrashBeforeTheLastRoot => image_keeping(history, |index, retained| {
            !(index >= history.before_last_publish
                && (is_system_configuration_write(parameters, retained)
                    || matches!(retained.operation.kind, RecordedOperationKind::WriteForceUnitAccess)))
        }),
        DiskOrigin::YiNaive => image_keeping(history, |index, retained| {
            !(index >= history.acquisition_end && is_system_configuration_write(parameters, retained))
        }),
        DiskOrigin::YiNaiveCrashBeforeTheLastRoot => image_keeping(history, |index, retained| {
            !((index >= history.acquisition_end && is_system_configuration_write(parameters, retained))
                || (index >= history.before_last_publish
                    && matches!(retained.operation.kind, RecordedOperationKind::WriteForceUnitAccess)))
        }),
    }
}

/// 一处故障（在内存盘上直接改字节）。
#[derive(Clone, Copy, Debug)]
enum Fault {
    /// txg 这次发布的根槽读不出（清零：自证不过）。
    RootSlotZeroed(u64),
    /// `version`（下标进 `History::versions`）的第一个数据单元在 `device` 上那一份翻一个字节。
    DataUnitCopyCorrupted { version: usize, device: DeviceIdentity },
    /// `version` 的末条记录在 `device` 上那一份清零。
    LastRecordCopyZeroed { version: usize, device: DeviceIdentity },
    /// `device` 上世代号最大的那一份系统配置槽清零。
    NewestSystemConfigurationSlotZeroed(DeviceIdentity),
}

fn apply_fault(history: &History, image: &mut MemoryPool, fault: Fault) {
    let parameters = &history.parameters;
    match fault {
        Fault::RootSlotZeroed(txg) => {
            let target = target_for_publish(CheckpointTxg(txg), parameters.geometry.root_ring_slots_per_region);
            let device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
            let offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
            let bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
            image.devices.get_mut(&device).expect("盘").write(offset, &vec![0u8; bytes]);
        }
        Fault::DataUnitCopyCorrupted { version, device } => {
            let location = history.versions[version].data_pointers[0]
                .locations
                .iter()
                .find(|location| location.device == device)
                .expect("每块盘一份");
            let offset = location.slot.to_device_offset();
            let length = usize::try_from(DATA_UNIT_BYTES).expect("32768");
            let mut bytes = image.devices[&device].read(offset, length);
            bytes[100] ^= 0xFF;
            image.devices.get_mut(&device).expect("盘").write(offset, &bytes);
        }
        Fault::LastRecordCopyZeroed { version, device } => {
            let counter = history.versions[version].record.counter;
            let offset = record_offset(counter, parameters.geometry.journal_ring_bytes);
            let length = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
            image.devices.get_mut(&device).expect("盘").write(offset, &vec![0u8; length]);
        }
        Fault::NewestSystemConfigurationSlotZeroed(device) => {
            let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
            let newest = verified_system_configuration_slots(&*image, device, spacing, &parameters.filesystem_identifier)
                .into_iter()
                .map(|slot| slot.quantities.slot_generation)
                .max()
                .expect("这块盘有自证过的系统配置");
            let offset = DeviceOffsetInBytes((newest % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing);
            image.devices.get_mut(&device).expect("盘").write(offset, &vec![0u8; 4096]);
        }
    }
}

fn state(history: &History, origin: DiskOrigin, faults: &[Fault]) -> MemoryPool {
    let mut image = image_of(history, origin);
    for fault in faults {
        apply_fault(history, &mut image, *fault);
    }
    image
}

/// 一次可写挂载的结局，压成一行能比的字样。
#[derive(Debug, PartialEq, Eq)]
enum WritableMount {
    /// 挂上了：施加前缀之后那一版的 (实例, txg)。
    Mounted { effective_instance: u32, effective_txg: u64, new_instance: u32 },
    /// 被「有盘不带所选那一版」拒：点名的盘。
    RefusedByDevicesWithoutTheSelectedVersion { devices: Vec<u32>, selected_txg: u64 },
    /// 被「系统配置见证过更新的发布、重读一次仍读不出」拒。
    RefusedByTheWitnessOfANewerPublish,
    Other(String),
}

fn devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    [DEVICE_ZERO, DEVICE_ONE]
        .into_iter()
        .map(|identity| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = image.devices[&identity].clone();
            (identity, device)
        })
        .collect()
}

fn image_of_devices(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    for (identity, device) in devices {
        image.devices.insert(*identity, device.image.clone());
    }
    image
}

/// 在 `image` 的一份拷贝上可写挂载；交回结局与挂载之后的盘面（拒了就是原样）。
fn mount_writable_on(history: &History, image: &MemoryPool) -> (WritableMount, MemoryPool) {
    let mut devices = devices_of(image);
    let outcome = match mount_writable(&history.parameters, &mut devices) {
        Ok(mounted) => WritableMount::Mounted {
            effective_instance: mounted.output.effective_root.instance.0,
            effective_txg: mounted.output.effective_root.checkpoint_txg.0,
            new_instance: mounted.output.instance.0,
        },
        Err(MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
            selected_version,
            devices: without,
            ..
        }) => WritableMount::RefusedByDevicesWithoutTheSelectedVersion {
            devices: without.iter().map(|device| device.device.0).collect(),
            selected_txg: selected_version.checkpoint_txg.0,
        },
        Err(MountError::NewerStateStillUnreadableAfterOneReread(_)) => {
            WritableMount::RefusedByTheWitnessOfANewerPublish
        }
        Err(other) => WritableMount::Other(format!("{other:?}")),
    };
    (outcome, image_of_devices(&devices))
}

/// 只读挂载读回第一个文件的全部字节。
fn file_bytes_read_only(image: &MemoryPool, length: usize) -> Result<(u64, Vec<u8>), String> {
    let mounted = mount_read_only(image).map_err(|failure| format!("{failure:?}"))?;
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(FIRST_INODE_NUMBER))
        .map_err(|failure| format!("{failure:?}"))?;
    let read = file
        .read_at(image, FileOffsetInBytes(0), u64::try_from(length).expect("长"))
        .map_err(|failure| format!("{failure:?}"))?;
    Ok((mounted.effective_root.checkpoint_txg.0, read.bytes))
}

/// 两份盘面按读出来的字节是否相同（没写过的扇区读出 0，与写了 0 的扇区不分）。
fn read_equal(left: &MemoryPool, right: &MemoryPool) -> bool {
    [DEVICE_ZERO, DEVICE_ONE].into_iter().all(|device| {
        let sectors: std::collections::BTreeSet<u64> = left.devices[&device]
            .written_sectors()
            .keys()
            .chain(right.devices[&device].written_sectors().keys())
            .copied()
            .collect();
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        sectors.into_iter().all(|sector| {
            let offset = DeviceOffsetInBytes(sector * SECTOR_BYTES);
            left.devices[&device].read(offset, sector_bytes) == right.devices[&device].read(offset, sector_bytes)
        })
    })
}

/// 乙-J 的落后支（攻方腿给乙提的最强一种，推的、被攻过零轮）：这块盘上「那一版末条记录」所在的环槽里读得出、解得开、
/// (实例代号, 计数器) 相同的记录，就算跟得上；否则落后，再照今天的判法逐个读那一版的单元。
fn yi_j_devices_without(
    history: &History,
    image: &MemoryPool,
    last_record: &JournalRecord,
    units: &[PublishedUnit],
) -> Vec<u32> {
    let parameters = &history.parameters;
    let offset = record_offset(last_record.counter, parameters.geometry.journal_ring_bytes);
    let length = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let expected_filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    [DEVICE_ZERO, DEVICE_ONE]
        .into_iter()
        .filter(|device| {
            let bytes = image.devices[device].read(offset, length);
            let holds_the_last_record = JournalRecord::parse(&bytes, expected_filesystem_identifier)
                .is_some_and(|record| {
                    record.instance == last_record.instance && record.counter == last_record.counter
                });
            if holds_the_last_record {
                return false;
            }
            units.iter().filter(|unit| !unit.bytes.is_empty()).any(|unit| {
                image.devices[device].read(unit.slot.to_device_offset(), unit.bytes.len()) != unit.bytes
            })
        })
        .map(|device| device.0)
        .collect()
}

fn corrupt_unit_copy(image: &mut MemoryPool, unit: &PublishedUnit, device: DeviceIdentity) {
    let offset = unit.slot.to_device_offset();
    let mut bytes = image.devices[&device].read(offset, unit.bytes.len());
    bytes[100] ^= 0xFF;
    image.devices.get_mut(&device).expect("盘").write(offset, &bytes);
}

/// Q2 落后支（可写挂载取号之前逐盘核）：甲自己判错的一格。txg 5 发布的根槽 FUA 已落、盘 1 上那次轮换没持久（正常崩溃点），
/// 盘 1 上这一版一个单元的那一份坏了一个字节（单份坏）。两样单独出现条款都放行，叠在一起被判「不带所选那一版」、拒可写，
/// 盘上不变、下一次照样拒。乙-J（落后看这块盘上有没有那一版末条记录）在同一盘面上放行，旧快照照样拒。
#[test]
fn probe_a_missed_rotation_plus_one_corrupted_copy_refuses_the_writable_mount_under_jia() {
    let history = build_history();
    let last = &history.versions[2];
    let corrupted = [Fault::DataUnitCopyCorrupted { version: 2, device: DEVICE_ONE }];
    let image = state(&history, DiskOrigin::LastRotationLostOn(&[DEVICE_ONE]), &corrupted);
    let (first_attempt, after_first_attempt) = mount_writable_on(&history, &image);
    let (second_attempt, _) = mount_writable_on(&history, &after_first_attempt);
    let read_back = file_bytes_read_only(&image, 4100);
    let rotation_only = mount_writable_on(&history, &state(&history, DiskOrigin::LastRotationLostOn(&[DEVICE_ONE]), &[])).0;
    let corruption_only = mount_writable_on(&history, &state(&history, DiskOrigin::Full, &corrupted)).0;
    let yi_j = yi_j_devices_without(&history, &image, &last.record, &last.units);
    println!("probe=A1 state=rotation-lost-on-dev1+dev1-data-copy-corrupted jia_first={first_attempt:?} jia_second={second_attempt:?} disk_unchanged={} read_only_txg_and_content_ok={:?} yi_j_devices_without={yi_j:?}",
        read_equal(&image, &after_first_attempt),
        read_back.as_ref().map(|(txg, bytes)| (*txg, *bytes == content_of(3))));
    println!("probe=A1-control rotation_lost_only={rotation_only:?} corrupted_copy_only={corruption_only:?}");
    assert_eq!(first_attempt, WritableMount::RefusedByDevicesWithoutTheSelectedVersion { devices: vec![1], selected_txg: 5 });
    assert_eq!(second_attempt, first_attempt, "盘上一个字节没动，下一次照样拒");
    assert!(read_equal(&image, &after_first_attempt));
    assert_eq!(read_back.map(|(txg, bytes)| (txg, bytes == content_of(3))), Ok((5, true)), "数据完好：只读挂载读回 txg 5");
    assert!(matches!(rotation_only, WritableMount::Mounted { effective_txg: 5, .. }));
    assert!(matches!(corruption_only, WritableMount::Mounted { effective_txg: 5, .. }));
    assert_eq!(yi_j, Vec::<u32>::new(), "乙-J：盘 1 带着 txg 5 的末条记录，不算落后");

    // 旧快照（盘 1 停在 txg 3）：甲与乙-J 都拒。
    let full = image_of(&history, DiskOrigin::Full);
    let stale_device_one = image_keeping(&history, |index, _| index < history.after_the_first_file);
    let mut stale = full.clone();
    stale.devices.insert(DEVICE_ONE, stale_device_one.devices[&DEVICE_ONE].clone());
    let jia_on_stale = mount_writable_on(&history, &stale).0;
    let yi_j_on_stale = yi_j_devices_without(&history, &stale, &last.record, &last.units);
    println!("probe=A1-stale jia={jia_on_stale:?} yi_j_devices_without={yi_j_on_stale:?}");
    assert_eq!(yi_j_on_stale, vec![1]);

    // 潜伏坏：坏的是 txg 5 那一版里、txg 3 那次就写下、之后一直没重写的单元（若有）。
    let shared: Vec<&PublishedUnit> = last
        .units
        .iter()
        .filter(|unit| !unit.bytes.is_empty() && history.versions[0].units.contains(unit))
        .collect();
    for unit in shared {
        let mut latent = image_of(&history, DiskOrigin::LastRotationLostOn(&[DEVICE_ONE]));
        corrupt_unit_copy(&mut latent, unit, DEVICE_ONE);
        let jia = mount_writable_on(&history, &latent).0;
        let yi_j_latent = yi_j_devices_without(&history, &latent, &last.record, &last.units);
        println!("probe=A1-latent unit={:?}@{} written_at_txg3=true jia={jia:?} yi_j_devices_without={yi_j_latent:?}", unit.identity, unit.slot.0);
        assert_eq!(jia, WritableMount::RefusedByDevicesWithoutTheSelectedVersion { devices: vec![1], selected_txg: 5 });
        assert_eq!(yi_j_latent, Vec::<u32>::new());
    }
}

/// 同一盘面在丙之下：返回之后、下一次发布的第一道屏障之前任何时刻崩溃都落到它（盘 1 或两块盘的轮换没持久）。
/// 两块盘的轮换都没持久时两块盘都落后，单份坏在哪块盘上都拒。
#[test]
fn probe_a2_both_rotations_lost_plus_one_corrupted_copy_on_either_device() {
    let history = build_history();
    for device in [DEVICE_ZERO, DEVICE_ONE] {
        let image = state(
            &history,
            DiskOrigin::LastRotationLostOn(&[DEVICE_ZERO, DEVICE_ONE]),
            &[Fault::DataUnitCopyCorrupted { version: 2, device }],
        );
        let outcome = mount_writable_on(&history, &image).0;
        println!("probe=A2 state=both-rotations-lost+data-copy-corrupted-on-dev{} jia={outcome:?}", device.0);
        assert_eq!(outcome, WritableMount::RefusedByDevicesWithoutTheSelectedVersion { devices: vec![device.0], selected_txg: 5 });
    }
}

/// 乙-J2 的见证（攻方腿给乙提的最强一种，推的、被攻过零轮）：环里读得出一条带「本次发布末条」标志、(实例, txg) 大于所选那一版的记录，
/// 且 `zero_slot_means_never_written` 为真时它那次发布的根槽不是全 0（全 0 当「那次发布的根从没写过」）。为真就按今天的乙那样重读一次、仍真拒可写。
fn yi_j2_witnesses_a_newer_publish(history: &History, image: &MemoryPool, zero_slot_means_never_written: bool) -> bool {
    use singlefs_core::recovery::{choose_system_configuration, scan_journal};
    let system_configuration = choose_system_configuration(image).expect("择系统配置");
    let records = scan_journal(image, &system_configuration);
    let effective = mount_read_only(image).expect("只读挂载").effective_root;
    let parameters = &history.parameters;
    records.values().any(|record| {
        record.place_in_publish == singlefs_core::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
            && (record.instance, record.checkpoint_txg) > (effective.instance, effective.checkpoint_txg)
            && {
                let target = target_for_publish(record.checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
                let device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
                let bytes = image.devices[&device].read(
                    slot_offset(target, parameters.geometry.fixed_structure_slot_spacing),
                    usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽"),
                );
                !(zero_slot_means_never_written && bytes.iter().all(|byte| *byte == 0))
            }
    })
}

/// Q2 判据 N-配置：X = txg 5 已返回（fsync 已确认），它的根槽一时读不出、它的数据单元盘 1 那一份坏（两处故障，C577 那一格）；
/// Y = txg 5 的根槽 FUA 之前崩（没确认），它的数据单元盘 1 那一份坏（崩溃 + 一处故障）。
/// 该有的结局：X 不许把 txg 5 当被抛弃（甲拒可写、重读后读得出就接 txg 5），Y 放行、接 txg 4。
/// 在丙与乙之下 X 与 Y 的盘面逐字节相同（没有一个写在根槽 FUA 之后），甲之下不同（轮换写的 tail）。
#[test]
fn probe_b_the_witness_after_the_root_is_what_separates_a_confirmed_publish_from_an_unconfirmed_one() {
    let history = build_history();
    let x_faults = [Fault::RootSlotZeroed(5), Fault::DataUnitCopyCorrupted { version: 2, device: DEVICE_ONE }];
    let y_faults = [Fault::DataUnitCopyCorrupted { version: 2, device: DEVICE_ONE }];
    let x_jia = state(&history, DiskOrigin::Full, &x_faults);
    let x_bing = state(&history, DiskOrigin::LastRotationLostOn(&[DEVICE_ZERO, DEVICE_ONE]), &x_faults);
    let x_yi = state(&history, DiskOrigin::YiNaive, &x_faults);
    let y_jia_bing = state(&history, DiskOrigin::CrashBeforeTheLastRoot, &y_faults);
    let y_yi = state(&history, DiskOrigin::YiNaiveCrashBeforeTheLastRoot, &y_faults);
    let identical = (read_equal(&x_jia, &y_jia_bing), read_equal(&x_bing, &y_jia_bing), read_equal(&x_yi, &y_yi));
    println!("probe=B-identity x_equals_y_under_jia={} under_bing={} under_yi_naive={}", identical.0, identical.1, identical.2);
    assert_eq!(identical, (false, true, true));

    let outcomes = [
        ("X-jia", mount_writable_on(&history, &x_jia).0),
        ("X-bing", mount_writable_on(&history, &x_bing).0),
        ("X-yi-naive", mount_writable_on(&history, &x_yi).0),
        ("Y-jia-or-bing", mount_writable_on(&history, &y_jia_bing).0),
        ("Y-yi-naive", mount_writable_on(&history, &y_yi).0),
    ];
    for (name, outcome) in &outcomes {
        println!("probe=B state={name} writable_mount={outcome:?}");
    }
    assert_eq!(outcomes[0].1, WritableMount::RefusedByTheWitnessOfANewerPublish, "甲在 X 上拒：已确认的 txg 5 不被抛弃");
    for index in [1, 2, 3, 4] {
        assert!(matches!(outcomes[index].1, WritableMount::Mounted { effective_txg: 4, .. }), "{} 落到 txg 4", outcomes[index].0);
    }
    // 乙-J2 两种读法在同一对盘面上：盘面相同，判法给同一个答案，X 与 Y 必有一个判错。
    for zero_slot_means_never_written in [false, true] {
        let on_x = yi_j2_witnesses_a_newer_publish(&history, &x_yi, zero_slot_means_never_written);
        let on_y = yi_j2_witnesses_a_newer_publish(&history, &y_yi, zero_slot_means_never_written);
        println!("probe=B-yi-j2 zero_slot_means_never_written={zero_slot_means_never_written} witness_on_x={on_x} witness_on_y={on_y}");
        assert_eq!(on_x, on_y);
    }
}

/// 丙之下 X 的后果：可写挂载落到 txg 4、取号 2、写行与暖机；故障撤掉（根槽与那一份单元原样写回，没被新实例盖掉的才写回）之后，
/// 只读挂载读回的是 txg 4 的内容，fsync 已返回的 txg 5 那一版找不回来。甲之下同一盘面是「根槽 FUA 之后、轮换持久之前崩」，txg 5 没确认，
/// 丢它合条款（C554 Q1 的已知残留），这一格只在丙之下是丢已确认的数据。
#[test]
fn probe_b2_under_bing_the_confirmed_publish_is_gone_after_the_faults_lift() {
    let history = build_history();
    let full = image_of(&history, DiskOrigin::Full);
    let x_faults = [Fault::RootSlotZeroed(5), Fault::DataUnitCopyCorrupted { version: 2, device: DEVICE_ONE }];
    let x_bing = state(&history, DiskOrigin::LastRotationLostOn(&[DEVICE_ZERO, DEVICE_ONE]), &x_faults);
    let (outcome, mut after) = mount_writable_on(&history, &x_bing);
    let parameters = &history.parameters;
    let target = target_for_publish(CheckpointTxg(5), parameters.geometry.root_ring_slots_per_region);
    let root_device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
    let data_location = history.versions[2].data_pointers[0].locations.iter().find(|location| location.device == DEVICE_ONE).expect("盘 1 一份");
    let places = [
        (root_device, root_offset, usize::try_from(parameters.geometry.physical_block_size).expect("512")),
        (DEVICE_ONE, data_location.slot.to_device_offset(), usize::try_from(DATA_UNIT_BYTES).expect("32768")),
    ];
    let mut restored = Vec::new();
    for (device, offset, length) in places {
        let untouched = after.devices[&device].read(offset, length) == x_bing.devices[&device].read(offset, length);
        if untouched {
            let original = full.devices[&device].read(offset, length);
            after.devices.get_mut(&device).expect("盘").write(offset, &original);
        }
        restored.push(untouched);
    }
    let read_back = file_bytes_read_only(&after, 4100);
    println!("probe=B2 bing_writable_mount={outcome:?} faults_lifted={restored:?} read_only_after_lift={:?}",
        read_back.as_ref().map(|(txg, bytes)| (*txg, *bytes == content_of(3), *bytes == content_of(2))));
    assert!(matches!(outcome, WritableMount::Mounted { effective_txg: 4, new_instance: 2, .. }));
    let (_, bytes) = read_back.expect("只读挂载读得出");
    assert_eq!(bytes, content_of(2), "读回 txg 4 的内容，txg 5（丙之下已确认）丢了");
}

/// 甲自己在 N-配置 上判错要几处故障：X 的两处 + 两块盘各自世代号最大的那一槽系统配置。只坏一块盘的那一槽时仍拒。
#[test]
fn probe_d_jia_abandons_the_confirmed_publish_only_with_both_newest_system_configuration_slots_gone() {
    let history = build_history();
    let base = [Fault::RootSlotZeroed(5), Fault::DataUnitCopyCorrupted { version: 2, device: DEVICE_ONE }];
    let cells: [(&str, Vec<Fault>); 3] = [
        ("X+sc0", [base.as_slice(), &[Fault::NewestSystemConfigurationSlotZeroed(DEVICE_ZERO)]].concat()),
        ("X+sc1", [base.as_slice(), &[Fault::NewestSystemConfigurationSlotZeroed(DEVICE_ONE)]].concat()),
        ("X+sc0+sc1", [base.as_slice(), &[Fault::NewestSystemConfigurationSlotZeroed(DEVICE_ZERO), Fault::NewestSystemConfigurationSlotZeroed(DEVICE_ONE)]].concat()),
    ];
    let outcomes: Vec<WritableMount> = cells
        .iter()
        .map(|(name, faults)| {
            let outcome = mount_writable_on(&history, &state(&history, DiskOrigin::Full, faults)).0;
            println!("probe=D state={name} faults={} jia={outcome:?}", faults.len());
            outcome
        })
        .collect();
    assert_eq!(outcomes[0], WritableMount::RefusedByTheWitnessOfANewerPublish);
    assert_eq!(outcomes[1], WritableMount::RefusedByTheWitnessOfANewerPublish);
    assert!(matches!(outcomes[2], WritableMount::Mounted { effective_txg: 4, .. }));
}

/// 乙-J2 丢已确认发布的最少故障：根槽读不出 + 那次发布末条记录两份都读不出（三处，不要单元坏：记录读不出重放本来就接不上）。
/// 同一格在甲之下拒（系统配置见证 txg 5），在乙-朴素之下放行。
#[test]
fn probe_e_yi_j2_abandons_the_confirmed_publish_with_three_faults_where_jia_refuses() {
    let history = build_history();
    let faults = [
        Fault::RootSlotZeroed(5),
        Fault::LastRecordCopyZeroed { version: 2, device: DEVICE_ZERO },
        Fault::LastRecordCopyZeroed { version: 2, device: DEVICE_ONE },
    ];
    let on_jia = state(&history, DiskOrigin::Full, &faults);
    let on_yi = state(&history, DiskOrigin::YiNaive, &faults);
    let jia = mount_writable_on(&history, &on_jia).0;
    let yi_naive = mount_writable_on(&history, &on_yi).0;
    let yi_j2 = [false, true].map(|zero_means_never| yi_j2_witnesses_a_newer_publish(&history, &on_yi, zero_means_never));
    println!("probe=E faults=3 jia={jia:?} yi_naive={yi_naive:?} yi_j2_witness(zero=ambiguous,zero=never)={yi_j2:?}");
    assert_eq!(jia, WritableMount::RefusedByTheWitnessOfANewerPublish);
    assert!(matches!(yi_naive, WritableMount::Mounted { effective_txg: 4, .. }));
    assert_eq!(yi_j2, [false, false], "乙-J2 看不见 txg 5：可写挂载照常，已确认的 txg 5 被抛弃");
}

/// 乙-朴素在落后支上的单故障：没有崩溃，只有盘 1 一份单元坏；乙-朴素之下每块盘的 tail 都停在取号那一刻，两块盘都「落后」，
/// 单份坏就被判「不带」、拒可写。甲在同一格放行（`current_device_one_with_one_corrupted_copy_still_mounts_writable`）。
#[test]
fn probe_c_yi_naive_refuses_the_writable_mount_on_a_single_corrupted_copy() {
    let history = build_history();
    let corrupted = [Fault::DataUnitCopyCorrupted { version: 2, device: DEVICE_ONE }];
    let yi_naive = mount_writable_on(&history, &state(&history, DiskOrigin::YiNaive, &corrupted)).0;
    let jia = mount_writable_on(&history, &state(&history, DiskOrigin::Full, &corrupted)).0;
    println!("probe=C faults=1 yi_naive={yi_naive:?} jia={jia:?}");
    assert_eq!(yi_naive, WritableMount::RefusedByDevicesWithoutTheSelectedVersion { devices: vec![1], selected_txg: 5 });
    assert!(matches!(jia, WritableMount::Mounted { effective_txg: 5, .. }));
}
