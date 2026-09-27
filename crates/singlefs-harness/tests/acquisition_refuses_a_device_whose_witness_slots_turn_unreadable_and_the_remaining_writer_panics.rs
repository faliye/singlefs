//! 实审 A3c（规格 `/tmp/claude-1000/impl-rev-a3c/spec.md`，写于 2026-09-27 JST）：
//!
//! 一、C554 乙-配置续 Q1（主 agent 定走「拒」）：取号那一刻读见证值时某块盘两槽里一份本池自证过的系统配置都读不出，
//! 在第一个取号写之前拒——可写挂载报 `MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`
//! （`NoSelfVerifiedSystemConfiguration`，与取号之前的逐盘核第一支、Z3-A 乙、D18（块里携带什么信息） 已定项 11「可见」同一判），
//! `acquire_instance` 报 `InstanceAcquisitionFailed::DeviceWithoutASelfVerifiedSystemConfigurationWhenReadingTheWitness`。
//! 改之前这块盘在见证值里不出数、取号写照做：逐盘核那一遍读得出、见证值那一遍读不出（两次读之间一次瞬时读错）就走得到。
//! 这一核在第一道屏障之前做一遍（拒的时候录制流一步不多），屏障之后取见证值时再读一遍，对不上就不写。
//!
//! 二、代码审阅 38 条里落在写者这一侧、今天还开着的几处 panic 面（实审 A2c 报告「停下交主 agent 的」第 4 条、
//! C476 普查「缺口」表 `inode_tree.rs:88` 那一行）：零单元发布判一次记录条数上限；inode 叶容器装满 233 条那道断言靠读者的哪一道判定。
//!
//! 盘：第一节用 `common` 的文件镜像（录制流开着，`DiskSnapshot` 比录制流步数），第二节用内存稀疏盘（改盘上字节之后重封每一道校验和）。
mod common;

use std::cell::RefCell;
use std::rc::Rc;

use common::{
    build_pool, disk_snapshot, with_unreadable_ranges, without_unreadable_ranges, BuiltPool,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{
    mount_writable, DeviceWithoutTheSelectedVersion, MountError, RollbackTarget,
    SelectedVersionLackingOnDevice,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::{parse_inode_internal_entry, TreeTableEntry, TREE_KIND_INODE};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, PoolReader,
    RecoveryFailure, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, journal_record_limit_of_one_publish, publish_first_file,
    publish_without_units, warm_up, FirstFile, InstanceAcquisitionFailed, PoolWriter, PublishError,
    TransactionOutput, ZeroUnitPublishPlan,
};
use singlefs_core::unit::{parse_index_node, seal_header_checksum};
use singlefs_format::{
    DATA_UNIT_BYTES, INODE_LEAF_RECORDS, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES,
    NODE_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::crash::{SparseBlockDevice, SparseDevice};
use singlefs_harness::RecordedOperationKind;

// ─── 一、取号那一刻读见证值时读不出的那块盘拒 ───

const DEVICE_WHOSE_SLOTS_TURN_UNREADABLE: DeviceIdentity = DeviceIdentity(1);

/// 盘 1 两个系统配置槽各一段读故障，两段都按 `failing_reads` 坏（每段各自从 1 数它被读了几次）。
fn both_system_configuration_slots_of_device_one(
    failing_reads_of_each_slot: [FailingReadsOfARange; 2],
) -> Vec<UnreadableRange> {
    let spacing = u64::from(common::parameters().geometry.fixed_structure_slot_spacing);
    [0, spacing]
        .into_iter()
        .zip(failing_reads_of_each_slot)
        .map(|(offset_in_bytes, failing_reads)| UnreadableRange {
            device: DEVICE_WHOSE_SLOTS_TURN_UNREADABLE,
            offset_in_bytes,
            length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
            failing_reads,
        })
        .collect()
}

/// 读、写原样交给里面那块盘；第一次屏障那一刻记下读故障每一段各被读了几次。可写挂载在取号之前一个写、一道屏障都不发，
/// 它的第一道屏障就是取号写之前那一道：记下的就是「取号写之前最后一次读这一段」是第几次读。
struct DeviceCountingReadsUpToTheFirstBarrier<Inner: BlockDevice> {
    inner: Inner,
    ranges: SharedUnreadableRanges,
    range_count: usize,
    reads_of_each_range_at_the_first_barrier: Rc<RefCell<Option<Vec<u64>>>>,
}

impl<Inner: BlockDevice> BlockDevice for DeviceCountingReadsUpToTheFirstBarrier<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        let mut recorded = self.reads_of_each_range_at_the_first_barrier.borrow_mut();
        if recorded.is_none() {
            *recorded = Some(
                (0..self.range_count)
                    .map(|range_index| self.ranges.reads_of_range(range_index))
                    .collect(),
            );
        }
        drop(recorded);
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 在另一份逐字节相同的池上（`build_pool` 的 fsid、写入时间、内容都是固定的）照常可写挂载一次、一个读都不坏，
/// 交回取号写之前那道屏障那一刻，盘 1 两个系统配置槽各被读了几次。
fn reads_of_device_one_slots_before_the_acquisition_barrier(tag: &str) -> [u64; 2] {
    let mut pool = build_pool(tag);
    let ranges = SharedUnreadableRanges::new(
        both_system_configuration_slots_of_device_one([FailingReadsOfARange::Never; 2]),
        UnreadableRangeReadBack::DeviceError,
    );
    let recorded = Rc::new(RefCell::new(None));
    let mut devices: Vec<_> = with_unreadable_ranges(pool.reopen_recorded(), &ranges)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceCountingReadsUpToTheFirstBarrier {
                    inner: device,
                    ranges: ranges.clone(),
                    range_count: 2,
                    reads_of_each_range_at_the_first_barrier: Rc::clone(&recorded),
                },
            )
        })
        .collect();
    let mounted = mount_writable(&common::parameters(), &mut devices);
    assert!(
        mounted.is_ok(),
        "一个读都不坏时这份池照常可写挂载：{:?}",
        mounted.as_ref().err()
    );
    let counts = recorded.borrow().clone().expect("可写挂载取号之前发过屏障");
    assert!(
        counts.iter().all(|reads| *reads >= 2),
        "取号写之前盘 1 两槽各被读过不止一次（择系统配置、逐盘核、算号、见证值那一核）：{counts:?}"
    );
    [counts[0], counts[1]]
}

/// 第一个事务那一版可写挂载被拒时报的：所选那一版就是它，不带它的只有盘 1、缺的是自证过的系统配置。
fn refused_for_device_one_without_a_self_verified_system_configuration(
    refusal: &Result<singlefs_core::mount::Mounted, MountError>,
    first: &TransactionOutput,
) -> bool {
    matches!(
        refusal,
        Err(MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
            selected_version,
            devices,
            ..
        }) if *selected_version == RollbackTarget {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
        } && *devices == vec![DeviceWithoutTheSelectedVersion {
            device: DEVICE_WHOSE_SLOTS_TURN_UNREADABLE,
            lacking: SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration,
        }]
    )
}

fn describe(refusal: &Result<singlefs_core::mount::Mounted, MountError>) -> String {
    match refusal {
        Ok(mounted) => format!(
            "挂载做成：取到实例 {:?}，所选那一版 ({:?}, {:?})",
            mounted.output.instance,
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        Err(error) => format!("{error:?}"),
    }
}

/// 场景：第一个事务之后重开、可写挂载。盘 1 两个系统配置槽从「取号写之前最后一次读」起每次都读坏
/// （次数取另一份相同的池上照常挂载时数到的：择系统配置、取号之前的逐盘核、算号那几遍都读得出）。
/// 预期：取号拒在第一道屏障之前，报 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`（盘 1、`NoSelfVerifiedSystemConfiguration`），
/// `DiskSnapshot` 不变（系统配置槽、根环、录制流步数：一个写、一道屏障都没发）。
/// 改之前：见证值只取盘 0 的、取号写照做，挂载做成。
#[test]
fn a_device_whose_slots_turn_unreadable_before_the_acquisition_barrier_refuses_the_writable_mount_without_a_step(
) {
    let [slot_zero_reads, slot_one_reads] =
        reads_of_device_one_slots_before_the_acquisition_barrier("a3c-probe-before-barrier");
    let mut pool = build_pool("a3c-before-barrier");
    let first = pool.output.clone();
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let ranges = SharedUnreadableRanges::new(
        both_system_configuration_slots_of_device_one([
            FailingReadsOfARange::FromTheNthOnward(slot_zero_reads),
            FailingReadsOfARange::FromTheNthOnward(slot_one_reads),
        ]),
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &ranges);
    let refusal = mount_writable(&common::parameters(), &mut devices);
    assert!(
        refused_for_device_one_without_a_self_verified_system_configuration(&refusal, &first),
        "取号那一刻盘 1 两槽都读不出：拒可写，报盘 1 不带所选那一版；得到 {}",
        describe(&refusal)
    );
    pool.devices = Some(without_unreadable_ranges(devices));
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在第一道屏障之前：一个写、一道屏障都没发"
    );
}

/// 场景：同上，盘 1 两槽从「取号写之前那道屏障之后的第一次读」起读坏：屏障之前那一核读得出，屏障之后取见证值那一遍读不出。
/// 预期：不写、报同一个成员；系统配置槽与根环不变，录制流只多那一道屏障。改之前：见证值只取盘 0 的、取号写照做，挂载做成。
#[test]
fn a_device_whose_slots_turn_unreadable_after_the_acquisition_barrier_refuses_before_any_write() {
    let [slot_zero_reads, slot_one_reads] =
        reads_of_device_one_slots_before_the_acquisition_barrier("a3c-probe-after-barrier");
    let mut pool = build_pool("a3c-after-barrier");
    let first = pool.output.clone();
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let ranges = SharedUnreadableRanges::new(
        both_system_configuration_slots_of_device_one([
            FailingReadsOfARange::FromTheNthOnward(slot_zero_reads + 1),
            FailingReadsOfARange::FromTheNthOnward(slot_one_reads + 1),
        ]),
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &ranges);
    let refusal = mount_writable(&common::parameters(), &mut devices);
    assert!(
        refused_for_device_one_without_a_self_verified_system_configuration(&refusal, &first),
        "屏障之后取见证值时盘 1 两槽都读不出：不写、拒可写；得到 {}",
        describe(&refusal)
    );
    pool.devices = Some(without_unreadable_ranges(devices));
    let after = disk_snapshot(&pool.memory_pool(), &pool.stream);
    assert_eq!(
        (&after.system_configuration_slots, &after.readable_roots),
        (&before.system_configuration_slots, &before.readable_roots),
        "屏障之后读不出就不写：系统配置槽、根环逐字节不变"
    );
    let operations_since = &pool.stream.operations()[before.recorded_operations..];
    assert!(
        !operations_since.is_empty()
            && operations_since
                .iter()
                .all(|operation| operation.kind == RecordedOperationKind::Barrier),
        "录制流只多取号写之前那一道屏障（每块盘一条屏障记录），没有写：{operations_since:?}"
    );
}

/// 场景：第一个事务之后重开，不经可写挂载、直接 `acquire_instance`（前面没有逐盘核）；盘 1 两个系统配置槽每次都读坏。
/// 预期：`InstanceAcquisitionFailed::DeviceWithoutASelfVerifiedSystemConfigurationWhenReadingTheWitness`（盘 1），`DiskSnapshot` 不变。
/// 改之前：见证值只取盘 0 的，两块盘都写了取号写，交出新号 2。
#[test]
fn acquire_instance_refuses_a_device_whose_slots_are_unreadable_before_any_step() {
    let mut pool: BuiltPool = build_pool("a3c-acquire-instance");
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let ranges = SharedUnreadableRanges::new(
        both_system_configuration_slots_of_device_one([FailingReadsOfARange::Every; 2]),
        UnreadableRangeReadBack::DeviceError,
    );
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &ranges);
    let acquisition = {
        let pool_parameters = common::parameters();
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        acquire_instance(&mut writer)
    };
    assert!(
        matches!(
            acquisition,
            Err(InstanceAcquisitionFailed::DeviceWithoutASelfVerifiedSystemConfigurationWhenReadingTheWitness {
                device: DEVICE_WHOSE_SLOTS_TURN_UNREADABLE,
            })
        ),
        "盘 1 两槽都读不出：取号拒、报盘 1；得到 {acquisition:?}"
    );
    pool.devices = Some(without_unreadable_ranges(devices));
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在第一道屏障之前：一个写、一道屏障都没发"
    );
}

// ─── 二、写者这一侧还开着的 panic 面（内存稀疏盘上的池） ───

const SPARSE_DEVICE_BYTES: u64 = 4 << 30;
const SPARSE_FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-rev-a3c";
const SPARSE_WRITE_TIME_SECONDS: u64 = 1_788_300_000;
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);

type SparseDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn sparse_pool_parameters_with_journal_ring(journal_ring_bytes: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: SPARSE_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn sparse_pool_parameters() -> MakeFilesystemParameters {
    sparse_pool_parameters_with_journal_ring(JOURNAL_RING_DEFAULT_BYTES)
}

fn two_sparse_devices() -> SparseDevices {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(SPARSE_DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
            )
        })
        .collect()
}

/// 两块盘此刻的整份镜像：拒之前、拒之后逐字节比。
fn images_of(devices: &SparseDevices) -> Vec<(DeviceIdentity, SparseDevice)> {
    devices
        .iter()
        .map(|(identity, device)| (*identity, device.image.clone()))
        .collect()
}

fn device_mut(devices: &mut SparseDevices, identity: DeviceIdentity) -> &mut SparseBlockDevice {
    &mut devices
        .iter_mut()
        .find(|(candidate, _)| *candidate == identity)
        .expect("池里有这块盘")
        .1
}

fn read_bytes(devices: &SparseDevices, location: &LocationEntry, length: u64) -> Vec<u8> {
    PoolReader::read(
        devices,
        location.device,
        location.slot.to_device_offset(),
        usize::try_from(length).expect("单元宽装得进 usize"),
    )
    .expect("读得到")
}

/// 按位置条目把一个单元的字节写到它的两份落点上（两块盘各一份），交回整单元 CRC-32C（位置条目里记的那个校验和）。
fn write_unit_at_both_locations(
    devices: &mut SparseDevices,
    locations: &[LocationEntry; 2],
    unit: &[u8],
) -> u32 {
    for location in locations {
        device_mut(devices, location.device)
            .image
            .write(location.slot.to_device_offset(), unit);
    }
    crc32_castagnoli(unit)
}

/// 码 2 节点改过之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)）。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 码 3 单元改过之后重封：载荷 CRC（罩 [107, 32768)，住 89）与头校验和（罩 [0, 107)）。
fn reseal_packed_unit(unit: &mut [u8]) {
    let payload_checksum = crc32_castagnoli(&unit[107..]);
    unit[89..93].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(unit, 107);
}

/// 码 2 节点第 `index` 条条目在节点里的字节范围（条目区从 86 + 2k + 29 起，条目宽住 84 + 2k）。
fn entry_range_in_node(node: &[u8], index: usize) -> std::ops::Range<usize> {
    let key_width = usize::from(node[51]);
    let entry_width = usize::from(u16::from_le_bytes([
        node[84 + 2 * key_width],
        node[85 + 2 * key_width],
    ]));
    let entries_start = 86 + 2 * key_width + 29;
    entries_start + index * entry_width..entries_start + (index + 1) * entry_width
}

/// 一条节点指针里两条位置条目的整单元校验和换成 `unit_checksum`（指针在字节里从 `pointer_offset` 起：头部 50，位置条目 14 × 2，
/// 每条的校验和住条目偏移 4 + 6）。
fn repoint_the_checksums_of_a_pointer(bytes: &mut [u8], pointer_offset: usize, unit_checksum: u32) {
    for location_index in 0..2 {
        let checksum_offset = pointer_offset + 50 + location_index * 14 + 10;
        bytes[checksum_offset..checksum_offset + 4].copy_from_slice(&unit_checksum.to_le_bytes());
    }
}

/// 根环里 (txg, 实例代号) 最大的那条根与它住的槽（择根择到的就是它）。
fn newest_root_in_the_ring(devices: &SparseDevices) -> (RootRingSlot, RootRecord) {
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
        .max_by_key(|(_, root)| (root.checkpoint_txg, root.instance))
        .expect("根环里有自证过的根")
}

fn write_root_into_the_ring_slot(
    devices: &mut SparseDevices,
    ring_slot: RootRingSlot,
    root: &RootRecord,
) {
    let parameters = sparse_pool_parameters();
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
    device_mut(devices, device).image.write(
        offset,
        &root.to_slot(usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽")),
    );
}

/// 两块内存盘上 mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）：inode 树根下一片叶容器，映射树是单个叶根。
fn sparse_pool_after_the_first_file() -> SparseDevices {
    let parameters = sparse_pool_parameters();
    let mut devices = two_sparse_devices();
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
            write_time_seconds: SPARSE_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("第一个文件");
    devices
}

/// 一次发布至多几条记录的上限是 0 的环：一槽 4096 字节 ÷ F（3）向下取整是 0（D23（journal 的角色与格式） 已定项 18）。
const JOURNAL_RING_WITH_NO_RECORD_IN_FLIGHT: u64 = JOURNAL_RECORD_BYTES;

/// 场景：只做过 mkfs 的池（默认环），另开一个写入口、参数里的环是一槽（在飞上限 0），在 mkfs 的第 0 代根上发一次零单元发布。
/// mkfs 与可写挂载都拒这种环（从盘上走不到），这里直接喂：写入口的参数由调用方给，`PoolWriter::new` 不判参数。
/// 预期：`PublishError::JournalRecordsOfThePublishExceedTheLimit { records_of_the_publish: 1, record_limit: 0 }`，两块盘逐字节不变、
/// 写入口的失败账是空的（没进落盘阶段）。改之前：零单元发布不判条数、照写（实审 A2c 报告「停下交主 agent 的」第 4 条）。
#[test]
fn a_zero_unit_publish_on_a_ring_with_no_record_in_flight_is_refused_before_any_write() {
    assert_eq!(
        journal_record_limit_of_one_publish(JOURNAL_RING_WITH_NO_RECORD_IN_FLIGHT),
        0,
        "一槽的环在飞上限是 0"
    );
    let mut devices = two_sparse_devices();
    let genesis =
        make_filesystem(&sparse_pool_parameters(), &mut devices).expect("默认几何上 mkfs");
    let before = images_of(&devices);
    let writer_parameters =
        sparse_pool_parameters_with_journal_ring(JOURNAL_RING_WITH_NO_RECORD_IN_FLIGHT);
    let mut writer = PoolWriter::new(&writer_parameters, devices.as_mut_slice());
    let published = publish_without_units(
        &mut writer,
        &genesis.root,
        ZeroUnitPublishPlan {
            txg: CheckpointTxg(1),
            counter: 1,
            instance: InstanceGeneration(1),
            back_chain: 0,
            rollback_floor: genesis.root.rollback_floor,
            tree_identifier_watermark: genesis.root.tree_identifier_watermark,
        },
    );
    assert!(
        matches!(
            published,
            Err(PublishError::JournalRecordsOfThePublishExceedTheLimit {
                records_of_the_publish: 1,
                record_limit: 0,
            })
        ),
        "零单元发布恒一条记录，在飞上限 0 的环上拒：{:?}",
        published.as_ref().map(|output| output.root.checkpoint_txg)
    );
    assert!(
        writer.writes_of_failed_publishes().is_empty(),
        "拒在落盘阶段之前：失败账一份都没有"
    );
    drop(writer);
    assert_eq!(
        images_of(&devices),
        before,
        "拒在任何写之前：两块盘逐字节不变"
    );
}

/// 场景：第一个文件那一版 inode 树唯一那片叶容器的头改成装 234 条、每条 140 字节（记录数住 69、声明长度住 8，载荷与头校验和重封），
/// inode 树根里指着它的那条指针、树表、映射树与根记录的校验和都重算：读者按指针读得到它、校验和对得上。
/// 预期：可写挂载在任何写之前报 `RecoveryFailure::UnitMalformed { what: "inode 叶容器" }`（记录区越过单元末尾），两块盘逐字节不变。
/// 这是 C476 普查「缺口」表 `inode_tree.rs:88` 那一行（`InodeLeafContainer::free_record_slots` 的 233 条上限）从盘上走不到的依据：
/// 读者判记录宽 140（`recovery::inode_leaf_container_judged_against_its_entry`）、记录区不越过单元末尾（`unit::parse_packed_unit`），
/// 于是盘上读来的一片至多 ⌊(32768 − 107 − 29) ÷ 140⌋ = 233 条。
#[test]
fn an_inode_leaf_container_claiming_more_records_than_a_unit_holds_is_refused_by_the_writable_mount(
) {
    let mut devices = sparse_pool_after_the_first_file();
    let (ring_slot, mut root) = newest_root_in_the_ring(&devices);
    let mut tree_table = read_bytes(&devices, &root.tree_table.locations[0], NODE_BYTES);
    let tree_table_node = parse_index_node(&tree_table).expect("树表单元解得开");
    let inode_entry_index = tree_table_node
        .entries
        .iter()
        .position(|entry| {
            TreeTableEntry::parse(entry).is_some_and(|parsed| parsed.kind == TREE_KIND_INODE)
        })
        .expect("树表里有 inode 树");
    let inode_root_pointer = TreeTableEntry::parse(&tree_table_node.entries[inode_entry_index])
        .expect("树表条目解得开")
        .root;
    let mut inode_root = read_bytes(&devices, &inode_root_pointer.locations[0], NODE_BYTES);
    let inode_root_node = parse_index_node(&inode_root).expect("inode 树根解得开");
    assert_eq!(
        inode_root_node.entries.len(),
        1,
        "第一个文件那一版只有一片叶容器"
    );
    let (_, _, leaf_pointer) =
        parse_inode_internal_entry(&inode_root_node.entries[0]).expect("inode 内部条目解得开");

    let records_past_the_limit = INODE_LEAF_RECORDS + 1;
    let inode_record_width: u64 = 140;
    let mut leaf = read_bytes(&devices, &leaf_pointer.locations[0], DATA_UNIT_BYTES);
    leaf[69..71].copy_from_slice(
        &u16::try_from(records_past_the_limit)
            .expect("234 装得进 2 字节")
            .to_le_bytes(),
    );
    leaf[8..10].copy_from_slice(
        &u16::try_from(records_past_the_limit * inode_record_width)
            .expect("32760 装得进 2 字节")
            .to_le_bytes(),
    );
    reseal_packed_unit(&mut leaf);
    let leaf_checksum = write_unit_at_both_locations(&mut devices, &leaf_pointer.locations, &leaf);

    // inode 内部条目：分隔 key 8 + 身份引用 26 + 子指针 86，子指针从条目偏移 34 起。
    let leaf_entry_range = entry_range_in_node(&inode_root, 0);
    repoint_the_checksums_of_a_pointer(&mut inode_root[leaf_entry_range], 34, leaf_checksum);
    reseal_index_node(&mut inode_root);
    let inode_root_checksum =
        write_unit_at_both_locations(&mut devices, &inode_root_pointer.locations, &inode_root);

    // 树表条目里根指针从条目偏移 14 起（树 ID 8 + 条目长度 2 + 种类 2 + flags 2）。
    let inode_entry_range = entry_range_in_node(&tree_table, inode_entry_index);
    repoint_the_checksums_of_a_pointer(&mut tree_table[inode_entry_range], 14, inode_root_checksum);
    reseal_index_node(&mut tree_table);
    let tree_table_checksum =
        write_unit_at_both_locations(&mut devices, &root.tree_table.locations, &tree_table);

    // 映射树（单个叶根）里 inode 树根与叶容器那两条的校验和也换上：提示读得出时读者不查映射，换上它只为让这份镜像处处自洽。
    let mut mapping_root = read_bytes(&devices, &root.mapping_root.locations[0], NODE_BYTES);
    let mapping_node = parse_index_node(&mapping_root).expect("映射树根解得开");
    assert_eq!(mapping_node.level, 0, "第一个文件那一版的映射树是单个叶根");
    for (mapped_pointer, unit_class, new_checksum) in [
        (
            inode_root_pointer,
            singlefs_core::unit::UNIT_CLASS_INDEX_NODE,
            inode_root_checksum,
        ),
        (
            leaf_pointer,
            singlefs_core::unit::UNIT_CLASS_PACKED,
            leaf_checksum,
        ),
    ] {
        let mapping_key = singlefs_core::records::mapping_key_for_node(unit_class, mapped_pointer);
        let mapping_entry_index = mapping_node
            .entries
            .iter()
            .position(|entry| {
                singlefs_core::records::parse_mapping_entry(entry)
                    .is_some_and(|(key, _)| key == mapping_key)
            })
            .expect("映射里有这个单元");
        let mapping_entry_range = entry_range_in_node(&mapping_root, mapping_entry_index);
        for location_index in 0..2 {
            // 映射条目：key 27 + 位置条目 14 × 2，每条的校验和住条目偏移 4 + 6。
            let checksum_offset = mapping_entry_range.start + 27 + location_index * 14 + 10;
            mapping_root[checksum_offset..checksum_offset + 4]
                .copy_from_slice(&new_checksum.to_le_bytes());
        }
    }
    reseal_index_node(&mut mapping_root);
    let mapping_root_checksum =
        write_unit_at_both_locations(&mut devices, &root.mapping_root.locations, &mapping_root);

    for location in &mut root.tree_table.locations {
        location.unit_checksum = tree_table_checksum;
    }
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = mapping_root_checksum;
    }
    write_root_into_the_ring_slot(&mut devices, ring_slot, &root);

    let before = images_of(&devices);
    let refusal = mount_writable(&sparse_pool_parameters(), &mut devices);
    assert!(
        matches!(
            refusal,
            Err(MountError::Recovery(RecoveryFailure::UnitMalformed {
                what: "inode 叶容器"
            }))
        ),
        "一片叶容器自述 234 条、每条 140 字节，装不进一个单元：可写挂载拒；得到 {:?}",
        refusal.as_ref().err()
    );
    assert_eq!(
        images_of(&devices),
        before,
        "拒在任何写之前：两块盘逐字节不变"
    );
}
