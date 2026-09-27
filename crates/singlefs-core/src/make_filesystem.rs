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
    INSTANCE_ROW_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR,
    NODE_BYTES, NODE_POINTER_BYTES, ROOT_RECORD_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
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
use crate::root_record::{RootRecord, UnmountMarker};
use crate::root_ring::{region_length_in_bytes, region_start, ring_end, slot_offset, RootRingSlot};
use crate::system_configuration::{
    journal_in_flight_record_limit, SystemConfiguration, SystemImmutableConfiguration,
    SystemImmutableSizes, SystemMutableConfiguration, SystemRuntimeConfiguration,
    SystemRuntimeQuantities,
};
use crate::unit::{
    build_index_node, build_packed_unit, PackedIdentity, WriteOrder, PACKED_TYPE_INSTANCE_TABLE,
};

/// mkfs 写实例代号 0（D23（journal 的角色与格式） 已定项 16）。
pub const MKFS_INSTANCE_GENERATION: InstanceGeneration = InstanceGeneration(0);
/// 系统配置槽世代号从 1 起，mkfs 把两个槽都种上 1（D22（单元原子性怎么合成） 已定项 16）。
pub const SYSTEM_CONFIGURATION_GENERATION_AT_MKFS: u64 = 1;
/// mkfs 写的回退下界 F：第 0 代根与每个系统配置槽两处都写 0（`.claude/kb/layout/01-first-txn.md` 一「回退下界 F」、
/// 七「回退下界 F」两行；D16（发布语义） 已定项 1：F 平时不动，准入不够或正常卸载时才抬）。
pub const ROLLBACK_FLOOR_AT_MKFS: CheckpointTxg = CheckpointTxg(0);
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

/// 每块盘开头那几段固定结构（D22（单元原子性怎么合成） 已定项 8 / 已定项 16，D23（journal 的角色与格式） 已定项 2）。
/// 封闭集合：`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixedStructure {
    /// 系统配置槽 `slot_index`（每盘两槽，槽 i 在 i × 固定结构槽距处，宽 4096）。
    SystemConfigurationSlot { slot_index: u64 },
    /// 根环区域 `region`（起点 = 基址 + region × P × chunk，长 = S × 固定结构槽距）。mkfs 在每块盘上把三个区域都清零，
    /// 所以三个区域在每块盘上都占着这一段，不论这块盘背不背它的根。
    RootRingRegion { region: u64 },
    /// journal 环（从槽 1024 起，长 = 系统配置里的环长）。
    JournalRing,
}

/// 一段固定结构在盘上占的字节：`[start, start + length_in_bytes)`，设备内偏移。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedStructureExtent {
    pub structure: FixedStructure,
    pub start: DeviceOffsetInBytes,
    pub length_in_bytes: u64,
}

impl FixedStructureExtent {
    fn end_exclusive(self) -> u64 {
        self.start.0 + self.length_in_bytes
    }

    /// 两段半开区间有没有共同的字节：首尾相接不算。
    fn overlaps(self, other: Self) -> bool {
        self.start.0 < other.end_exclusive() && other.start.0 < self.end_exclusive()
    }
}

/// mkfs 能出的错：几何放不下，或底层块设备错。几何那几样都在任何写之前判（[`make_filesystem`] 的第一步）。
#[derive(Debug)]
pub enum MakeFilesystemError {
    /// 根槽宽（= 参数里的 physical_block_size）装不下根记录（457 字节，D22（单元原子性怎么合成） 已定项 7）。
    RootSlotNarrowerThanTheRootRecord {
        physical_block_size: u32,
        root_record_bytes: u64,
    },
    /// 根槽宽（= 参数里的 physical_block_size）大于固定结构槽距：每条根写盖到同一区域的下一个槽
    /// （槽 j 在区域起点 + j × 槽距，D22（单元原子性怎么合成） 已定项 16 第 2 句）。
    RootSlotWiderThanTheFixedStructureSlotSpacing {
        physical_block_size: u32,
        fixed_structure_slot_spacing: u32,
    },
    /// journal 环短于一条记录：环槽数是 0，记录落点（`journal::record_offset`，按环槽数取模）无处可落。
    JournalRingShorterThanOneRecord {
        ring_bytes: u64,
        record_bytes: u64,
    },
    /// journal 环装不下 F 条记录（F = [`JOURNAL_SAFETY_FACTOR`]，I-8.1（环几何够大） 的安全系数）：在飞上限 = 环槽数 ÷ F
    /// （D23（journal 的角色与格式） 已定项 18）是 0，恢复一条记录都不施加（`recovery::replay_journal` 按在飞上限取前缀）。
    /// I-8.1 要环 ≥ F × 任一事务的最坏 journal 占用，最坏占用至少一条记录，这是它最低的那一格。
    /// `minimum_ring_bytes` = F × 一条记录的字节数。
    JournalRingHoldsFewerRecordsThanTheSafetyFactor {
        ring_bytes: u64,
        minimum_ring_bytes: u64,
    },
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
    /// 单元区起点越过设备末尾：`unit_area_start` 是 mkfs 写实例表单元的那个设备内字节偏移（[`INSTANCE_TABLE_SLOT`]），
    /// 实例表单元第 0 片与树表单元第 0 版装不进最小那块盘。
    UnitAreaBeyondDevice {
        unit_area_start: u64,
        device_bytes: u64,
    },
    /// 两段固定结构有共同的字节（系统配置两槽、三个根环区域、journal 环两两比）：写后一段会撕掉前一段。
    /// 两段按它们在格式里的排布次序给（系统配置槽 0、槽 1、区域 0、1、2、journal 环）。
    /// 固定结构槽距的上界就由它定：两槽要落在根环基址之前、区域长 S × 槽距不能越过区域间距 P × chunk。
    FixedStructuresOverlap {
        first_in_layout_order: FixedStructureExtent,
        second_in_layout_order: FixedStructureExtent,
    },
    /// journal 环的末端越过单元区起点：环会盖住实例表单元与分配器发出去的单元。
    /// D23（journal 的角色与格式） 已定项 19 ③ 写单元区起始槽号随环长走，而实例表 / 树表的落点（[`INSTANCE_TABLE_SLOT`]、
    /// [`TREE_TABLE_GENESIS_SLOT`]）、分配器（`allocator::unit_area_slots_of_device` 与空闲图的下标）、
    /// 恢复判分配记录落点（`recovery::allocation_records_fit_the_pool_geometry`）都还按编译期常量 `UNIT_AREA_START_SLOT` 算
    /// （C475（非默认环长下单元区起点取编译期常量））⇒ 第一版不支持末端越过它的环长。末端不越过它的环照收，单元区照旧从那个常量起。
    JournalRingPastTheCompiledUnitAreaStartUnsupported {
        journal_ring_end_in_bytes: u64,
        unit_area_start_in_bytes: u64,
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

/// 每块盘开头那几段固定结构各占哪一段字节，按它们在格式里的排布次序：系统配置槽 0、槽 1、根环区域 0、1、2、journal 环。
/// 与 [`make_filesystem`] 写它们用的是同一组式子（槽 i 在 i × 槽距、[`region_start`] 与 [`region_length_in_bytes`]、
/// journal 环从 [`JOURNAL_RING_START_SLOT`] 起），判定与写读的是同一份参数。
fn fixed_structure_extents(geometry: &SystemImmutableSizes) -> Vec<FixedStructureExtent> {
    let system_configuration_slots =
        (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).map(|slot_index| FixedStructureExtent {
            structure: FixedStructure::SystemConfigurationSlot { slot_index },
            start: DeviceOffsetInBytes(
                slot_index * u64::from(geometry.fixed_structure_slot_spacing),
            ),
            length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
        });
    let root_ring_regions = (0..ROOT_RING_REGIONS).map(|region| FixedStructureExtent {
        structure: FixedStructure::RootRingRegion { region },
        start: region_start(region),
        length_in_bytes: region_length_in_bytes(
            geometry.fixed_structure_slot_spacing,
            geometry.root_ring_slots_per_region,
        ),
    });
    let journal_ring = FixedStructureExtent {
        structure: FixedStructure::JournalRing,
        start: DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES),
        length_in_bytes: geometry.journal_ring_bytes,
    };
    system_configuration_slots
        .chain(root_ring_regions)
        .chain(std::iter::once(journal_ring))
        .collect()
}

/// 几何判定，全在任何写之前（[`make_filesystem`] 的第一步）。次序：根槽宽、环长下界（一条记录、F 条记录）与上界、根环末端与单元区在不在盘内、
/// 根槽宽不超过槽距、固定结构两两不重叠、journal 环末端不越过单元区起点、区域归属。
fn check_geometry(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, u64)],
) -> Result<(), MakeFilesystemError> {
    let geometry = &parameters.geometry;
    // 根槽宽 = physical_block_size（`RootRecord::to_slot` 按它切）：装不下根记录就在这里拒，不走到那里的断言——
    // 那时根环与 journal 环已经清过、两个单元已经写了。
    if u64::from(geometry.physical_block_size) < ROOT_RECORD_BYTES {
        return Err(MakeFilesystemError::RootSlotNarrowerThanTheRootRecord {
            physical_block_size: geometry.physical_block_size,
            root_record_bytes: ROOT_RECORD_BYTES,
        });
    }
    let smallest = devices.iter().map(|(_, bytes)| *bytes).min().unwrap_or(0);
    let ring_bytes = geometry.journal_ring_bytes;
    if ring_bytes < JOURNAL_RECORD_BYTES {
        return Err(MakeFilesystemError::JournalRingShorterThanOneRecord {
            ring_bytes,
            record_bytes: JOURNAL_RECORD_BYTES,
        });
    }
    // 判的是恢复真正用的那个数（在飞上限），不另写一遍「环槽数 < F」：两处同一个式子。
    if journal_in_flight_record_limit(ring_bytes) == 0 {
        return Err(
            MakeFilesystemError::JournalRingHoldsFewerRecordsThanTheSafetyFactor {
                ring_bytes,
                minimum_ring_bytes: JOURNAL_SAFETY_FACTOR * JOURNAL_RECORD_BYTES,
            },
        );
    }
    if ring_bytes > smallest / 4 {
        return Err(MakeFilesystemError::JournalRingTooLargeForDevice {
            ring_bytes,
            device_bytes: smallest,
        });
    }
    let root_ring_end = ring_end(
        geometry.fixed_structure_slot_spacing,
        geometry.root_ring_slots_per_region,
    );
    if root_ring_end > smallest {
        return Err(MakeFilesystemError::RootRingBeyondDevice {
            ring_end: root_ring_end,
            device_bytes: smallest,
        });
    }
    // 单元区起点取 mkfs 真正写实例表单元的那个偏移：它今天是编译期常量，不随环长走（C475），按环长现算出来的数只在默认环长下与它相等。
    let unit_area_start = INSTANCE_TABLE_SLOT.to_device_offset().0;
    let end_of_the_units_written_by_make_filesystem =
        TREE_TABLE_GENESIS_SLOT.to_device_offset().0 + NODE_BYTES;
    if end_of_the_units_written_by_make_filesystem > smallest {
        return Err(MakeFilesystemError::UnitAreaBeyondDevice {
            unit_area_start,
            device_bytes: smallest,
        });
    }
    if geometry.physical_block_size > geometry.fixed_structure_slot_spacing {
        return Err(
            MakeFilesystemError::RootSlotWiderThanTheFixedStructureSlotSpacing {
                physical_block_size: geometry.physical_block_size,
                fixed_structure_slot_spacing: geometry.fixed_structure_slot_spacing,
            },
        );
    }
    let extents = fixed_structure_extents(geometry);
    // 六段两两比，路径数只有「第一对重叠的在哪」一种出口；迭代次数的上界是 6 × 5 ÷ 2。
    for (index, first_in_layout_order) in extents.iter().enumerate() {
        for second_in_layout_order in &extents[index + 1..] {
            if first_in_layout_order.overlaps(*second_in_layout_order) {
                return Err(MakeFilesystemError::FixedStructuresOverlap {
                    first_in_layout_order: *first_in_layout_order,
                    second_in_layout_order: *second_in_layout_order,
                });
            }
        }
    }
    // 上面判过两两不重叠，区域与系统配置槽都落在 journal 环起点（16 MiB）之前；越得过单元区起点的只有 journal 环。
    let journal_ring_end_in_bytes = JOURNAL_RING_START_SLOT * SLOT_BYTES + ring_bytes;
    if journal_ring_end_in_bytes > unit_area_start {
        return Err(
            MakeFilesystemError::JournalRingPastTheCompiledUnitAreaStartUnsupported {
                journal_ring_end_in_bytes,
                unit_area_start_in_bytes: unit_area_start,
            },
        );
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
    // 它们互不重叠，也不与同一段里的单元写重叠（`check_geometry` 判过：固定结构两两不重叠、journal 环末端不越过单元区起点）；真正重叠的是清根环与根槽 FUA 写
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
        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
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
        rollback_floor: ROLLBACK_FLOOR_AT_MKFS,
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
                rollback_floor: ROLLBACK_FLOOR_AT_MKFS,
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
