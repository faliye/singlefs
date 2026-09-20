//! mkfs（里程碑「第一个事务」步 1）：吃 N 块设备，写出一个能被挂载、但一个文件都没有的池。
//!
//! 写序（字节表零 m1..m4；段序列 `4+1+1+1+4`）：实例表单元第 0 片与树表单元第 0 版（两盘各一份）→ 屏障 →
//! 第 0 代根记录种进三个区域各自槽 0（各一道 FUA 写）→ 每盘两个系统配置槽都写世代号 1 → 屏障。
//! journal 环与根环区域「整段写 0」是镜像准备，不进录制流：这里要求设备是新建的全零镜像。

use singlefs_format::{
    INSTANCE_ROW_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, NODE_POINTER_BYTES,
    ROOT_RING_REGIONS, SLOT_BYTES, SUPERBLOCK_SLOTS_PER_DEVICE, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
    TREE_TABLE_ENTRY_BYTES, UNIT_AREA_START_SLOT,
};

use crate::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
use crate::bytes::ByteWriter;
use crate::checksum::crc32_castagnoli;
use crate::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use crate::root_record::RootRecord;
use crate::root_ring::{ring_end, slot_offset, RootRingSlot};
use crate::superblock::{FormatTimeGeometry, Superblock};
use crate::unit::{
    build_index_node, build_packed_unit, PackedIdentity, WriteOrder, PACKED_TYPE_INSTANCE_TABLE,
};

/// mkfs 写实例代号 0（D23（journal 的角色与格式） 已定项 16）。
pub const MKFS_INSTANCE_GENERATION: InstanceGeneration = InstanceGeneration(0);
/// 系统配置槽世代号从 1 起，mkfs 把两个槽都种上 1（D22（单元原子性怎么合成） 已定项 16）。
pub const SUPERBLOCK_GENERATION_AT_MKFS: u64 = 1;
/// 实例表单元第 0 片落槽 50176（占两槽），树表单元第 0 版落槽 50178（D3（空间分配） 已定项 10 ④）。
pub const INSTANCE_TABLE_SLOT: SlotNumber = SlotNumber(UNIT_AREA_START_SLOT);
pub const TREE_TABLE_GENESIS_SLOT: SlotNumber = SlotNumber(UNIT_AREA_START_SLOT + 2);
/// 树表单元的 key = 树 ID，8 字节。
pub const TREE_TABLE_KEY_WIDTH: usize = 8;
/// 第一版两块盘时根环三个区域的归属：区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0（D2（RAID 条带策略） 已定项 7，写死、不是 mkfs 参数）。
pub const FIRST_VERSION_REGION_DEVICES: [DeviceIdentity; 3] =
    [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)];

/// mkfs 的参数：fsid 与时间戳都是参数，同参数两次 mkfs 逐字节相同（里程碑步 1 验收）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MakeFilesystemParameters {
    pub filesystem_identifier: [u8; 16],
    pub region_devices: [DeviceIdentity; 3],
    pub geometry: FormatTimeGeometry,
}

/// mkfs 能出的错：几何放不下，或底层块设备错。
#[derive(Debug)]
pub enum MakeFilesystemError {
    /// journal 环超过设备容量的四分之一（D23（journal 的角色与格式） 已定项 19）。
    JournalRingTooLargeForDevice {
        ring_bytes: u64,
        device_bytes: u64,
    },
    /// 根环三个区域越过设备末尾。
    RootRingBeyondDevice {
        ring_end: u64,
        device_bytes: u64,
    },
    /// 单元区起点越过设备末尾。
    UnitAreaBeyondDevice {
        unit_area_start: u64,
        device_bytes: u64,
    },
    /// 参数说三个区域住哪块盘，而池里没有那块盘。
    RegionDeviceMissing {
        region: u64,
        device: DeviceIdentity,
    },
    /// 第一版两块盘时根环三个区域的归属写死盘 0 / 盘 1 / 盘 0（D2（RAID 条带策略） 已定项 7，2026-09-14 用户定案：不是 mkfs 参数），
    /// 参数给的不是它：暖机次数与第一个事务的 txg 3 都压在这个归属上（m2-emptypool-nonempty-r1 云端攻方腿 Z3-B：[0, 1, 1] 下暖机要推两次、
    /// 第一个文件版本与暖机撞在同一个 (实例, txg) 上）。
    RegionDevicesNotTheFirstVersionLayout {
        region_devices: [DeviceIdentity; 3],
    },
    BlockDevice(BlockDeviceError),
}

impl From<BlockDeviceError> for MakeFilesystemError {
    fn from(error: BlockDeviceError) -> Self {
        MakeFilesystemError::BlockDevice(error)
    }
}

/// mkfs 写出的三样东西，给验收与步 5 用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MakeFilesystemOutput {
    pub root: RootRecord,
    pub instance_table_unit: Vec<u8>,
    pub tree_table_genesis_unit: Vec<u8>,
}

/// 实例表 kind 1 链指针记录（88 字节）：kind 1 + flags 0 + 指向下一片的指针 86，第一片「无下一片」指针全 0。
#[must_use]
pub fn instance_table_chain_record() -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
    writer.put_u8(1);
    writer.put_u8(0);
    writer.skip(usize::try_from(NODE_POINTER_BYTES).expect("86"));
    writer.assert_position(INSTANCE_ROW_BYTES, "实例表链指针记录");
    writer.into_bytes()
}

/// 一个单元在每块盘上的位置条目，按设备身份升序（I-2.5）；校验和 = 整单元 CRC-32C。
#[must_use]
pub fn location_entries(
    devices: &[DeviceIdentity],
    slot: SlotNumber,
    unit: &[u8],
) -> [LocationEntry; 2] {
    let checksum = crc32_castagnoli(unit);
    let mut sorted: Vec<DeviceIdentity> = devices.to_vec();
    sorted.sort();
    [
        LocationEntry {
            device: sorted[0],
            slot,
            unit_checksum: checksum,
        },
        LocationEntry {
            device: sorted[1],
            slot,
            unit_checksum: checksum,
        },
    ]
}

fn check_geometry(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, u64)],
) -> Result<(), MakeFilesystemError> {
    let smallest = devices.iter().map(|(_, bytes)| *bytes).min().unwrap_or(0);
    let ring_bytes = parameters.geometry.journal_ring_bytes;
    if ring_bytes > smallest / 4 {
        return Err(MakeFilesystemError::JournalRingTooLargeForDevice {
            ring_bytes,
            device_bytes: smallest,
        });
    }
    let root_ring_end = ring_end(parameters.geometry.fixed_structure_slot_spacing);
    if root_ring_end > smallest {
        return Err(MakeFilesystemError::RootRingBeyondDevice {
            ring_end: root_ring_end,
            device_bytes: smallest,
        });
    }
    let unit_area_start = (JOURNAL_RING_START_SLOT + ring_bytes / SLOT_BYTES) * SLOT_BYTES;
    if unit_area_start + 3 * SLOT_BYTES > smallest {
        return Err(MakeFilesystemError::UnitAreaBeyondDevice {
            unit_area_start,
            device_bytes: smallest,
        });
    }
    for (region, device) in parameters.region_devices.iter().enumerate() {
        if !devices.iter().any(|(identity, _)| identity == device) {
            return Err(MakeFilesystemError::RegionDeviceMissing {
                region: u64::try_from(region).expect("区域号"),
                device: *device,
            });
        }
    }
    if devices.len() == 2 && parameters.region_devices != FIRST_VERSION_REGION_DEVICES {
        return Err(MakeFilesystemError::RegionDevicesNotTheFirstVersionLayout {
            region_devices: parameters.region_devices,
        });
    }
    Ok(())
}

/// 对两块设备做 mkfs。`devices` 里每块盘的身份由调用方给（位置条目按设备身份升序）。
pub fn make_filesystem<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
) -> Result<MakeFilesystemOutput, MakeFilesystemError> {
    assert_eq!(
        devices.len(),
        2,
        "第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"
    );
    let sizes: Vec<(DeviceIdentity, u64)> = devices
        .iter()
        .map(|(identity, device)| (*identity, device.size_in_bytes()))
        .collect();
    check_geometry(parameters, &sizes)?;
    let identities: Vec<DeviceIdentity> = sizes.iter().map(|(identity, _)| *identity).collect();
    let instance = MKFS_INSTANCE_GENERATION;
    let genesis = CheckpointTxg(0);
    let genesis_write_order = WriteOrder {
        instance,
        transaction: 0,
    };
    // 出生序号：同一棵树（这里都是「无归属」树 0）在 checkpoint 0 里依次 0、1（D19（块指针的结构与宽度预算） 已定项 9）。
    let instance_table_sequence = BirthSequence(0);
    let tree_table_sequence = BirthSequence(1);

    let instance_table_identity = PackedIdentity {
        birth_tree: TreeIdentifier(0),
        record_type: PACKED_TYPE_INSTANCE_TABLE,
        container: 0,
        container_birth: genesis,
    };
    let instance_table_unit = build_packed_unit(
        instance_table_identity,
        u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
        &[instance_table_chain_record()],
        genesis,
        &parameters.filesystem_identifier,
        genesis_write_order,
        instance_table_sequence,
    );
    let tree_table_genesis_unit = build_index_node(
        TreeIdentifier(0),
        0,
        TREE_TABLE_KEY_WIDTH,
        &[0u8; 8],
        &[0u8; 8],
        genesis,
        &parameters.filesystem_identifier,
        instance,
        tree_table_sequence,
        u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
        &[],
    );
    for (_, device) in devices.iter_mut() {
        device.write_at(
            INSTANCE_TABLE_SLOT.to_device_offset(),
            &instance_table_unit,
            WriteDurability::Plain,
        )?;
        device.write_at(
            TREE_TABLE_GENESIS_SLOT.to_device_offset(),
            &tree_table_genesis_unit,
            WriteDurability::Plain,
        )?;
    }
    for (_, device) in devices.iter_mut() {
        device.barrier()?;
    }

    let root = RootRecord {
        filesystem_identifier: parameters.filesystem_identifier,
        instance,
        checkpoint_txg: genesis,
        tree_table: NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(0),
                birth_txg: genesis,
            },
            locations: location_entries(
                &identities,
                TREE_TABLE_GENESIS_SLOT,
                &tree_table_genesis_unit,
            ),
            instance,
            birth_sequence: tree_table_sequence,
        },
        tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(0),
                birth_txg: genesis,
            },
            locations: location_entries(&identities, INSTANCE_TABLE_SLOT, &instance_table_unit),
            instance,
            birth_sequence: instance_table_sequence,
        },
        mapping_root: NodePointer::empty_root(),
    };
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let root_slot = root.to_slot(root_slot_bytes);
    for region in 0..ROOT_RING_REGIONS {
        let region_device = parameters.region_devices[usize::try_from(region).expect("区域号")];
        let (_, device) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == region_device)
            .expect("check_geometry 核过区域归属");
        let offset = slot_offset(
            RootRingSlot { region, slot: 0 },
            parameters.geometry.fixed_structure_slot_spacing,
        );
        device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;
    }

    for (identity, device) in devices.iter_mut() {
        let superblock = Superblock {
            filesystem_identifier: parameters.filesystem_identifier,
            this_device: *identity,
            device_count: u32::try_from(identities.len()).expect("设备数"),
            slot_generation: SUPERBLOCK_GENERATION_AT_MKFS,
            region_devices: parameters.region_devices,
            geometry: parameters.geometry,
            journal_tail: 0,
            journal_instance: instance,
        };
        let slot = superblock.to_slot();
        for slot_index in 0..SUPERBLOCK_SLOTS_PER_DEVICE {
            let offset = DeviceOffsetInBytes(
                slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing),
            );
            device.write_at(offset, &slot, WriteDurability::Plain)?;
        }
    }
    for (_, device) in devices.iter_mut() {
        device.barrier()?;
    }
    let _ = JOURNAL_RECORD_BYTES;
    Ok(MakeFilesystemOutput {
        root,
        instance_table_unit,
        tree_table_genesis_unit,
    })
}
