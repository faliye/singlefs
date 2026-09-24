//! mkfs（里程碑「第一个事务」步 1）：吃 N 块设备，写出一个能被挂载、但一个文件都没有的池。
//!
//! 写序（字节表零 m1..m4；段序列 `12+1+1+1+4`）：根环三个区域与 journal 环整环清零（每块盘 4 次写零调用）→
//! 实例表单元第 0 片与树表单元第 0 版（两盘各一份）→ 屏障 →
//! 第 0 代根记录种进三个区域各自槽 0（各一道 FUA 写）→ 每盘两个系统配置槽都写世代号 1 → 屏障。
//!
//! 清零 2026-09-22 起由 mkfs 自己做（用户 2026-09-19 定案「块设备抽象加写零」，
//! `records/2026-09-19-里程碑二遗留收拢.md` 第五之二节第 4 行；里程碑「第二个事务」并行线四）：
//! 此前它是「镜像准备」、不进录制流，等于要求设备是新建的全零镜像——真设备上没有这个前提。
//! 根环连着一起清是同一天的续作（C484（mkfs 不清根环，同 fsid 重来旧根还择得中），用户 2026-09-22 定案）。

use singlefs_format::{
    INSTANCE_ROW_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, NODE_POINTER_BYTES,
    ROOT_RING_REGIONS, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES, UNIT_AREA_START_SLOT,
};

use crate::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use crate::allocator::{
    DeviceFreeMap, Placement, PoolAllocator, RootRingOccupancy, RootRingOccupant,
};
use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
use crate::bytes::ByteWriter;
use crate::checksum::crc32_castagnoli;
use crate::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use crate::root_record::RootRecord;
use crate::root_ring::{region_length_in_bytes, region_start, ring_end, slot_offset, RootRingSlot};
use crate::system_configuration::{
    SystemConfiguration, SystemImmutableConfiguration, SystemImmutableSizes,
    SystemMutableConfiguration, SystemRuntimeConfiguration, SystemRuntimeQuantities,
};
use crate::unit::{
    build_index_node, build_packed_unit, PackedIdentity, WriteOrder, PACKED_TYPE_INSTANCE_TABLE,
};

/// mkfs 写实例代号 0（D23（journal 的角色与格式） 已定项 16）。
pub const MKFS_INSTANCE_GENERATION: InstanceGeneration = InstanceGeneration(0);
/// 系统配置槽世代号从 1 起，mkfs 把两个槽都种上 1（D22（单元原子性怎么合成） 已定项 16）。
pub const SYSTEM_CONFIGURATION_GENERATION_AT_MKFS: u64 = 1;
/// 实例表单元第 0 片落槽 50176（占两槽），树表单元第 0 版落槽 50178（D3（空间分配） 已定项 10 ④）。
pub const INSTANCE_TABLE_SLOT: SlotNumber = SlotNumber(UNIT_AREA_START_SLOT);
pub const TREE_TABLE_GENESIS_SLOT: SlotNumber = SlotNumber(UNIT_AREA_START_SLOT + 2);
/// 树表单元的 key = 树 ID，8 字节。
pub const TREE_TABLE_KEY_WIDTH: usize = 8;
/// 第一版两块盘时根环三个区域的归属：区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0（D2（RAID 条带策略） 已定项 7，写死、不是 mkfs 参数）。
pub const FIRST_VERSION_REGION_DEVICES: [DeviceIdentity; 3] =
    [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)];

/// mkfs 的参数：fsid 与时间戳都是参数，同参数两次 mkfs 逐字节相同（里程碑步 1 验收）。
///
/// 三样都进系统不可变配置那一档（D22（单元原子性怎么合成） 已定项 26 第一档：改了要重建文件系统）；
/// 那一档里剩下的本盘设备号与设备数不是参数——mkfs 逐盘现填、按池里的盘数现数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MakeFilesystemParameters {
    pub filesystem_identifier: [u8; 16],
    pub region_devices: [DeviceIdentity; 3],
    pub geometry: SystemImmutableSizes,
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
    let root_ring_end = ring_end(
        parameters.geometry.fixed_structure_slot_spacing,
        parameters.geometry.root_ring_slots_per_region,
    );
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
    // 先把根环三个区域与 journal 环整段清零（每块盘每段一次调用、录制流各一步），再写元数据。
    // 为什么 mkfs 自己做：一条 journal 记录、一条根记录合不合法只看它自己的魔数与校验和，盘上的旧字节里
    // 凑出一条合法记录的概率不是 0（上一次 mkfs 留下的就是真合法的记录），而「这块盘是新建的全零镜像」
    // 是装置的假设、真设备上不成立。
    // 根环非清不可（C484（mkfs 不清根环，同 fsid 重来旧根还择得中））：挡住旧池那几条根的只有 fsid，
    // 而 fsid 是调用方给的（`MakeFilesystemParameters::filesystem_identifier`，mkfs 自己不生成）⇒
    // 拿同一个 fsid 在用过的盘上重做 mkfs，旧池留在槽 1..7 里的根照样自证得过、解得开，
    // `choose_root` 按 (checkpoint_txg, 实例代号) 取最大还会择中它——而它指着的树表、实例表、分配记录都是旧池的账。
    // 三个区域在每块盘上都清，不按 `region_devices` 只清本盘要读的那几段：区域归属是参数、设备身份是调用方给的，
    // 按参数清要多查一次表，还会在别的盘上留下「本池 fsid、自证得过、这一版读不到」的根记录，
    // 它今天读不到全靠择根与 checker 两处各自按归属读（`recovery::visit_valid_roots`、
    // `singlefs-checker` 的 `image.rs`），任一处以后改成扫全部盘，漏就回来了。
    // 按几何清之后 mkfs 的后置条件只有一句：根环三段在每块盘上只剩这次写下的三条第 0 代根，别处全 0。
    // 次序与屏障：清零按设备内偏移升序发（根环 1 / 4 / 7 MiB，journal 环 16 MiB），几段之间不另加屏障——
    // 它们互不重叠，也不与同一段里的单元写重叠（单元区从 784 MiB 起）；真正重叠的是清根环与根槽 FUA 写
    //（区域 r 的槽 0），那一对由原有的那道屏障（单元写之后、根 FUA 之前）隔开，不靠发出次序。
    // mkfs 中途崩溃第一版没有条款（层 0 从 mkfs 之后的池起枚举，`.claude/kb/layout/01-first-txn.md` 八
    // mkfs 那一行的「层 0 枚举」格写「不在」），不为一个没有条款的语义加屏障。
    // 清的是 S 个槽那么长的一段，S 取 mkfs 参数里的那个——这一次写进系统配置的也是它，
    // 于是「mkfs 清了多长」与「挂载时按多长读」出自同一个数。
    let root_ring_region_bytes = region_length_in_bytes(
        parameters.geometry.fixed_structure_slot_spacing,
        parameters.geometry.root_ring_slots_per_region,
    );
    let journal_ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    for (_, device) in devices.iter_mut() {
        for region_to_clear in 0..ROOT_RING_REGIONS {
            device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;
        }
        device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;
    }
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
        // mkfs 的第 0 代不写分配记录树：这一版的账由实例表与树表两条指针直接算得出
        // （`mount::format_time_allocator`，字节表五里 m1 / m2 两条分配代 0 的记录）。
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let root_slot = root.to_slot(root_slot_bytes);
    for ring_slot in genesis_root_ring_slots() {
        let region_device =
            parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let (_, device) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == region_device)
            .expect("check_geometry 核过区域归属");
        let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
        device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;
    }

    for (identity, device) in devices.iter_mut() {
        let system_configuration = SystemConfiguration {
            immutable: SystemImmutableConfiguration {
                filesystem_identifier: parameters.filesystem_identifier,
                this_device: *identity,
                device_count: u32::try_from(identities.len()).expect("设备数"),
                region_devices: parameters.region_devices,
                sizes: parameters.geometry,
            },
            mutable: SystemMutableConfiguration,
            runtime: SystemRuntimeConfiguration,
            quantities: SystemRuntimeQuantities {
                slot_generation: SYSTEM_CONFIGURATION_GENERATION_AT_MKFS,
                journal_tail: 0,
                journal_instance: instance,
            },
        };
        let slot = system_configuration.to_slot();
        for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
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

/// mkfs 在根环上种第 0 代根的那几个槽：三个区域各自的槽 0。`make_filesystem` 按它写，
/// `root_ring_occupancy_after_make_filesystem` 按它记——两处出自同一份，各写一份会分叉。
fn genesis_root_ring_slots() -> impl Iterator<Item = RootRingSlot> {
    (0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })
}

/// mkfs 刚写完时分配器眼里的根环（`allocator::RootRingOccupancy`），给 mkfs 同一个进程里接着写、一次挂载都没做的那条会话装：
/// 三个区域的槽 0 各一条第 0 代根（`make_filesystem` 写的就是这三条；根环别的槽清成全 0、自证不过，不在表里），
/// F 取第 0 代根带的，这个进程下一条根的 txg 是第 0 代根的加一。装了它，这条会话里根环转过一圈之后与做过挂载的会话一样按可再分配谓词
/// 回收（D16（发布语义） 已定项 1；C518（一次挂载之内环转过一圈之后不回收） 那张表）。暖机那两次零单元发布不经分配器，
/// 由接在它们后面的第一个文件版本补记（`PoolAllocator::record_zero_unit_roots_leading_to`）。
#[must_use]
pub fn root_ring_occupancy_after_make_filesystem(
    parameters: &MakeFilesystemParameters,
    genesis: &MakeFilesystemOutput,
) -> RootRingOccupancy {
    RootRingOccupancy::read_from_the_ring(
        parameters.geometry.root_ring_slots_per_region,
        genesis_root_ring_slots()
            .map(|ring_slot| {
                (
                    ring_slot,
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: genesis.root.checkpoint_txg,
                    },
                )
            })
            .collect(),
        genesis.root.rollback_floor,
        genesis.root.checkpoint_txg,
    )
}

/// mkfs 同一个进程里接着写（取号、暖机、第一个文件版本、之后的发布）、一次挂载都没做的那条会话用的分配器：每块盘一张空闲图，
/// mkfs 写在单元区里的两个单元记成分配代 0（`PoolAllocator::mark_format_time_units`），装上 mkfs 刚写下的根环
/// （`root_ring_occupancy_after_make_filesystem`）。
#[must_use]
pub fn allocator_after_make_filesystem<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    genesis: &MakeFilesystemOutput,
) -> PoolAllocator {
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        parameters, genesis,
    ));
    allocator
}
