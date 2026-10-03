//! 择系统配置报得清两件事（判决 `research/prompts/m3-ring-r1-main-verification.md` 第二节第 11 行）：
//! - C462（空池恢复报的错名不副实）：交进来的池里一块盘都没有，恢复这一层报 `RecoveryFailure::NoDeviceInThePool`，
//!   不再报「各盘系统配置对不上」（`SystemConfigurationsDisagree`）；
//! - C463（一块盘的系统配置两份全废之后静默成功）：一块盘两个系统配置槽都废了、别的盘那一份照常择到时，只读挂载
//!   （`MountedReadOnly::devices_without_a_mountable_system_configuration_slot`）、恢复这一层
//!   （`choose_system_configuration_naming_devices_without_a_mountable_slot`）与整条恢复的报告
//!   （`RecoveryReport::devices_without_a_mountable_system_configuration_slot`，判决 `research/prompts/m3-batch4-code-r1-main-verification.md`
//!   第二节第 7 行）点名那块盘；冗余完好时三处都是空表。在用的 `choose_system_configuration` 返回类型不变、择到的还是同一份。
//!
//! 盘：两块 4 GiB 内存稀疏盘上 mkfs、取号、暖机、第一个文件（txg 3）之后的池（只读挂载要沿根打开文件版本）。

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration,
    choose_system_configuration_naming_devices_without_a_mountable_slot, recover, JournalPolicy,
    RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{SparseBlockDevice, SECTOR_BYTES};

const DEVICE_BYTES: u64 = 4 << 30;
const WRITE_TIME_SECONDS: u64 = 1_788_400_000;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-m3-c463";
/// 系统配置槽里整槽校验和罩着的一个字节（magic 之后、fsid 里那一段，在槽的第一个扇区里）：翻转它，那一槽自证不过。
const BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT: u64 = 10;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// 两块内存盘上 mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）。
fn pool_after_the_first_file() -> Devices {
    let parameters = parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
    let mut allocator: PoolAllocator =
        allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content: Vec<u8> = (0..3000u32)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect();
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号 1");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    publish_first_file(
        &mut writer,
        &mut allocator,
        warmed.roots.last().expect("暖机写了两条根"),
        FirstFile {
            content: &content,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("第一个文件");
    devices
}

/// `identity` 那块盘的两个系统配置槽（偏移 0 与槽距）各翻一个校验和罩着的字节：两槽都自证不过。
fn corrupt_both_system_configuration_slots_of(devices: &mut Devices, identity: DeviceIdentity) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let device = &mut devices
        .iter_mut()
        .find(|(candidate, _)| *candidate == identity)
        .expect("池里有这块盘")
        .1;
    for slot_offset in [0, spacing] {
        let sector = DeviceOffsetInBytes(slot_offset);
        let mut bytes = device
            .image
            .read(sector, usize::try_from(SECTOR_BYTES).expect("512"));
        bytes[usize::try_from(BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT).expect("10")] ^= 0xff;
        device.image.write(sector, &bytes);
    }
}

/// 场景：交进来的池一块盘都没有。预期：择系统配置与整条恢复都报 `NoDeviceInThePool`。
/// 改之前报 `SystemConfigurationsDisagree`，调用方会去找哪块盘是外来的。
#[test]
fn a_pool_without_any_device_is_refused_as_having_no_device() {
    let empty_pool: Devices = Vec::new();
    assert_eq!(
        choose_system_configuration(&empty_pool).err(),
        Some(RecoveryFailure::NoDeviceInThePool),
        "择系统配置：池里一块盘都没有"
    );
    assert_eq!(
        recover(&empty_pool, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: None,
            failure: RecoveryFailure::NoDeviceInThePool,
        },
        "整条恢复：同一个成员"
    );
}

/// 场景：第一个文件之后的池，两块盘四个系统配置槽都完好。预期：只读挂载、恢复这一层与整条恢复的报告都报空表（冗余完好），
/// 择到的与在用的入口同一份。
#[test]
fn a_pool_whose_every_device_keeps_a_mountable_system_configuration_names_no_device() {
    let devices = pool_after_the_first_file();
    let choice = choose_system_configuration_naming_devices_without_a_mountable_slot(&devices)
        .expect("系统配置择得到");
    assert_eq!(
        choice.devices_without_a_mountable_system_configuration_slot,
        Vec::<DeviceIdentity>::new(),
        "恢复这一层：冗余完好，一块盘都不点名"
    );
    assert_eq!(
        Ok(choice.system_configuration),
        choose_system_configuration(&devices),
        "择到的与 choose_system_configuration 同一份"
    );
    let mounted = mount_read_only(&devices).expect("只读挂载");
    assert_eq!(
        mounted.devices_without_a_mountable_system_configuration_slot,
        Vec::<DeviceIdentity>::new(),
        "只读挂载：冗余完好，一块盘都不点名"
    );
    let report = recover(&devices, JournalPolicy::Consult);
    assert!(
        matches!(report.outcome, RecoveryOutcome::FileRead { .. }),
        "整条恢复读得出文件"
    );
    assert_eq!(
        report.devices_without_a_mountable_system_configuration_slot,
        Vec::<DeviceIdentity>::new(),
        "整条恢复的报告：冗余完好，一块盘都不点名"
    );
}

/// 场景：第一个文件之后的池，盘 1 两个系统配置槽都自证不过，盘 0 两槽完好。
/// 预期：只读挂载照常挂上（择盘 0 那一份），并点名盘 1；恢复这一层同样点名盘 1，择到的与 `choose_system_configuration` 同一份；
/// 整条恢复照常读出文件，报告里那一格点名盘 1。改之前几处都只报成功，盘 1 的系统配置冗余已经没了却没人知道（C463）。
#[test]
fn a_device_whose_two_system_configuration_slots_are_both_lost_is_named_by_the_read_only_mount_and_recovery(
) {
    let mut devices = pool_after_the_first_file();
    corrupt_both_system_configuration_slots_of(&mut devices, DeviceIdentity(1));
    let choice = choose_system_configuration_naming_devices_without_a_mountable_slot(&devices)
        .expect("盘 0 那一份照常择到");
    assert_eq!(
        choice.devices_without_a_mountable_system_configuration_slot,
        vec![DeviceIdentity(1)],
        "恢复这一层点名盘 1"
    );
    assert_eq!(
        Ok(choice.system_configuration),
        choose_system_configuration(&devices),
        "在用的入口择到同一份"
    );
    assert_eq!(
        choice.system_configuration.immutable.this_device,
        DeviceIdentity(0),
        "择到的是盘 0 那一份"
    );
    let mounted = mount_read_only(&devices).expect("一块盘两槽全废，只读挂载照常挂上");
    assert_eq!(
        mounted.devices_without_a_mountable_system_configuration_slot,
        vec![DeviceIdentity(1)],
        "只读挂载点名盘 1"
    );
    let report = recover(&devices, JournalPolicy::Consult);
    assert!(
        matches!(report.outcome, RecoveryOutcome::FileRead { .. }),
        "一块盘两槽全废，整条恢复照常读出文件"
    );
    assert_eq!(
        report.devices_without_a_mountable_system_configuration_slot,
        vec![DeviceIdentity(1)],
        "整条恢复的报告点名盘 1"
    );
}
