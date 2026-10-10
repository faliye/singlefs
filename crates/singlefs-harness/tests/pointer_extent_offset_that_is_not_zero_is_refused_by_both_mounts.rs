//! 指针头部 extent 偏移那 2 字节（指针头部偏移 32）第一版恒 0，读者遇到非 0 判该指针所在的结构损坏，同 MAC / nonce
//! （D19（块指针的结构与宽度预算） 已定项 3；判决 `research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行，C581（extent 偏移非 0 没有判据））。
//!
//! 坏镜像照攻方那一份造（`research/prompts/m3-inv-a-r1-opus-output.md` 五、A6）：两块内存盘上 mkfs、取号、暖机、第一个文件（txg 3），
//! 把根环里每一条自证过的根的实例表指针 extent 偏移改成 1、整槽自证校验和重封。改之前 `recover` 读出文件、`mount_writable` 挂上
//! （读者 `reader.skip(2)`）；改之后两个入口都择不到根、拒，盘上逐字节不变。
//! 另一条在指针一级钉住判头部的那两个读法（多层码 2 树内部条目、journal 新根段与根记录四条指针用的；走读的读者经判不判由读者定的那个入口用的）
//! 报的是 extent 偏移。

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::{ByteReader, ByteWriter};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::pointer::{
    BirthSequence, LocationEntry, NodePointer, PointerHead, PointerHeadExtentOffsetNotZero,
    PointerHeadFieldOutsideTheFirstVersion, PointerHeadFieldsJudgement,
};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
    RecoveryFailure, RecoveryOutcome, RootRingSlotReading,
};
use singlefs_core::root_record::{RootRecord, ROOT_CHECKSUM_OFFSET};
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, NODE_POINTER_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};

const DEVICE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-m3-extn";
const WRITE_TIME_SECONDS: u64 = 1_788_300_000;
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512，根槽宽也是 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
/// 指针头部里 extent 偏移那 2 字节的偏移：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2 之后（D19（块指针的结构与宽度预算） 已定项 7 的字段表）。
const EXTENT_OFFSET_IN_THE_POINTER_HEAD: usize = 32;
/// 根槽里实例表指针的偏移：自证校验和 32 字节紧后面（`RootRecord::parse_slot` 的读法；用例里拿解出来的根核一遍）。
const INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT: usize = ROOT_CHECKSUM_OFFSET + 32;

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
                SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
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

fn images_of(devices: &Devices) -> Vec<(DeviceIdentity, SparseDevice)> {
    devices
        .iter()
        .map(|(identity, device)| (*identity, device.image.clone()))
        .collect()
}

/// 根环里自证过的每一条根，连同它住的槽（改之前读）。
fn self_verified_roots_in_the_ring(devices: &Devices) -> Vec<(RootRingSlot, RootRecord)> {
    let system_configuration = choose_system_configuration(devices).expect("系统配置择得到");
    every_root_ring_slot(&system_configuration.immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                devices,
                &system_configuration.immutable.region_devices,
                &system_configuration.immutable.sizes,
                &system_configuration.immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some((ring_slot, root)),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect()
}

/// 根环里每一条自证过的根：实例表指针的 extent 偏移写成 1、整槽自证校验和重封（攻方 A6 的改法）。交回改了几条。
fn set_the_instance_table_pointer_extent_offset_to_one_in_every_root(
    devices: &mut Devices,
) -> usize {
    let parameters = parameters();
    let root_slot_bytes =
        usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽装得进 usize");
    let roots = self_verified_roots_in_the_ring(devices);
    for (ring_slot, root) in &roots {
        let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let offset = slot_offset(*ring_slot, parameters.geometry.fixed_structure_slot_spacing);
        let device_under_change = &mut devices
            .iter_mut()
            .find(|(identity, _)| *identity == device)
            .expect("池里有这块盘")
            .1;
        let mut slot = device_under_change.image.read(offset, root_slot_bytes);
        assert_eq!(
            NodePointer::read_from(&mut ByteReader::at(
                &slot,
                INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT
            )),
            root.instance_table,
            "根槽偏移 {INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT} 起的 {NODE_POINTER_BYTES} 字节就是实例表指针"
        );
        let extent_offset_at =
            INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT + EXTENT_OFFSET_IN_THE_POINTER_HEAD;
        assert_eq!(
            &slot[extent_offset_at..extent_offset_at + 2],
            &[0u8, 0][..],
            "第一版写者写的 extent 偏移是 0"
        );
        slot[extent_offset_at..extent_offset_at + 2].copy_from_slice(&1u16.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&slot, root_slot_bytes, ROOT_CHECKSUM_OFFSET);
        slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        device_under_change.image.write(offset, &slot);
    }
    roots.len()
}

/// 场景：攻方 A6 那份镜像——第一个文件之后，根环里每条自证过的根的实例表指针 extent 偏移改成 1、整槽重封。
/// 预期：只读恢复与可写挂载两个入口都择不到根（这样的根记录判损坏、不可择），报 `NoValidRoot`；可写挂载在任何写之前拒，两块盘逐字节不变。
/// 改之前读者跳过这 2 字节：`recover` 读出文件、`mount_writable` 挂上。
#[test]
fn every_root_carrying_a_non_zero_instance_table_extent_offset_is_refused_by_recovery_and_the_writable_mount(
) {
    let mut devices = pool_after_the_first_file();
    let roots_changed =
        set_the_instance_table_pointer_extent_offset_to_one_in_every_root(&mut devices);
    assert!(
        roots_changed >= 4,
        "第 0 代根三条、暖机两条、第一个文件一条，根环里自证过的根至少 4 条，实际改了 {roots_changed} 条"
    );
    let images_before = images_of(&devices);
    let recovered = recover(&devices, JournalPolicy::Consult).outcome;
    assert_eq!(
        recovered,
        RecoveryOutcome::Failed {
            root: None,
            failure: RecoveryFailure::NoValidRoot,
        },
        "只读恢复：每条根的实例表指针 extent 偏移都非 0，一条可择的根都没有"
    );
    let writable = mount_writable(&parameters(), &mut devices);
    assert!(
        matches!(
            &writable,
            Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
        ),
        "可写挂载：同样择不到根，实际 {:?}",
        writable.as_ref().err()
    );
    assert_eq!(
        images_of(&devices),
        images_before,
        "可写挂载在任何写之前拒：两块盘逐字节不变"
    );
}

/// 一条节点指针（86 字节），头部 extent 偏移那 2 字节写成 `extent_offset`。
fn node_pointer_bytes_with_extent_offset(extent_offset: u16) -> (NodePointer, Vec<u8>) {
    let pointer = NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_ACCOUNTING),
            birth_txg: CheckpointTxg(3),
        },
        locations: [
            LocationEntry {
                device: DeviceIdentity(0),
                slot: SlotNumber(UNIT_AREA_START_SLOT + 8),
                unit_checksum: 0x1234,
            },
            LocationEntry {
                device: DeviceIdentity(1),
                slot: SlotNumber(UNIT_AREA_START_SLOT + 8),
                unit_checksum: 0x1234,
            },
        ],
        instance: InstanceGeneration(1),
        birth_sequence: BirthSequence(2),
    };
    let mut writer = ByteWriter::new(usize::try_from(NODE_POINTER_BYTES).expect("86"));
    pointer.write_to(&mut writer);
    let mut bytes = writer.into_bytes();
    bytes[EXTENT_OFFSET_IN_THE_POINTER_HEAD..EXTENT_OFFSET_IN_THE_POINTER_HEAD + 2]
        .copy_from_slice(&extent_offset.to_le_bytes());
    (pointer, bytes)
}

/// 场景：一条节点指针头部 extent 偏移写成 7，MAC、nonce、算法类型、压缩那几段都是第一版的取值。
/// 预期：判头部的两个读法都报 extent 偏移非 0、带着读到的 7——多层码 2 树内部条目、journal 新根段与根记录四条指针用的
/// `read_judging_the_encryption_compression_and_extent_offset_fields_from`，走读的读者用的 `read_with_head_fields_judgement_from`
/// 取判的那一种；extent 偏移是 0 的同一条指针两个读法都照常读出。改之前两个读法都跳过这 2 字节、照常读出。
#[test]
fn a_node_pointer_head_with_a_non_zero_extent_offset_is_refused_by_both_judging_readers() {
    let judged = PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion;
    let (pointer, clean) = node_pointer_bytes_with_extent_offset(0);
    assert_eq!(
        (
            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut ByteReader::at(&clean, 0)
            ),
            NodePointer::read_with_head_fields_judgement_from(
                &mut ByteReader::at(&clean, 0),
                judged
            ),
        ),
        (Ok(pointer), Ok(pointer)),
        "extent 偏移 0：两个读法都照常读出"
    );
    let (_, damaged) = node_pointer_bytes_with_extent_offset(7);
    let not_zero = PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(
        PointerHeadExtentOffsetNotZero { extent_offset: 7 },
    );
    assert_eq!(
        NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
            &mut ByteReader::at(&damaged, 0)
        ),
        Err(not_zero),
        "判加密与压缩那几段的读法也判 extent 偏移"
    );
    assert_eq!(
        NodePointer::read_with_head_fields_judgement_from(&mut ByteReader::at(&damaged, 0), judged),
        Err(not_zero),
        "走读的读者取判的那一种：报 extent 偏移"
    );
}
