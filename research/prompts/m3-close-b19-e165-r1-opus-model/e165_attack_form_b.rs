//! 攻方腿模型（三方轮 m3-close-b19-e165-r1）：今天的写者在「可写挂载那一刻最高几条根所在的槽读坏、它们那几次发布的记录也读不到、
//! R2 不触发」时，新实例第一次发布的 txg 会不会不大于那条根，之后那几槽读回来，盘上是不是形甲（同 txg、不同实例）或形乙
//! （实例代号更高而 txg 更低的一条根与实例代号低而 txg 高的一条根同在）。
//!
//! 复跑时 tools/rerun.sh 把这份拷进冻结副本的 crates/singlefs-harness/tests/，当一个 harness 档测试目标编（只在副本里，不进仓）。
//! 盘是内存稀疏盘（SparseBlockDevice），起点历史只建一次，每一格从那一份克隆。
//!
//! 起点历史（真写者）：mkfs 两块盘 → 实例 1 取号、暖机（txg 1、2）→ 新池新建文件（txg 3）→ 覆盖写 B（txg 4）、C（txg 5）、
//! D（txg 6）；D 的根槽 FUA 持久、系统配置轮换没持久就崩（把两块盘的系统配置槽退回 D 之前的字节）。C 完整发布、已确认；D 没确认。
//!
//! 每一格：实例 2 的可写挂载在读故障下跑（读故障按格给：读坏几条根、根与记录读坏的种类、最新那一槽系统配置读坏的种类），
//! 写进来的那一段改了的落点故障撤掉（写过之后读回新字节）。录下这次挂载的写流，照崩溃点重放的枚举域（屏障与 FUA 切段、
//! 段内写的真子集逐个、再加全部持久）逐个状态：读全部根槽、判形甲 / 形乙、按基线与三条变体各择哪一槽、跑池级 checker、
//! 再在没有读故障的盘上做一次可写挂载（基线）看择到哪一版。
//!
//! 对照：
//!   控制-低号写者  写者变体：在起点镜像上把 txg 5 那一槽换成实例 2、txg 5 的根（第一次发布取「读得出的最大」而不是 + 1 的写者），
//!                  形乙在、v1826 与基线择得不同——这一格不报形乙，模型的判法就是坏的。
//!   控制-错位同号  把实例 2、txg 6 的根写进 txg 7 那一槽（错位）：读根环的那一道（C743）当它读坏，形甲不出现。
#![allow(clippy::too_many_lines, reason = "模型一份文件，逐格的历史与判法写在一起")]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters,
    INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    PoolReader, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::{
    closed_form_state_count, root_identity_of_write, writes_and_segments_with_stream_indexes,
    MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

const DEVICE_BYTES: u64 = 4 << 30;
const ROOT_SLOT_BYTES: u64 = 512;
const SLOT_SPACING: u64 = 4096;
/// 与 harness 用例共用搭建里那一份 fsid 同值（`tests/common/mod.rs` 的 `E142_FILESYSTEM_IDENTIFIER`）。
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 读得出、不是全零而自证不过那一种读坏：读回的字节里翻这一个（根记录里落在树表指针那一段，记录头与系统配置槽里同样在校验和罩住的范围里）。
const FLIPPED_BYTE_INSIDE_A_RANGE: u64 = 100;
/// 一段历史的崩溃状态数超过它就不跑（报告里写明）。
const STATES_OF_ONE_HISTORY_LIMIT: u64 = 1_000_000;
/// 起点历史里 D 那一版的 txg（根槽持久、轮换没持久）与 C 那一版（完整发布、已确认）。
const TXG_OF_D: u64 = 6;
const TXG_OF_C: u64 = 5;

fn pool_parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn fixed_geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    }
}

fn file_content_of(seed: u64) -> Vec<u8> {
    (0..3000u64)
        .map(|index| u8::try_from((index * 7 + seed) % 251).expect("小于 256"))
        .collect()
}

/// 一次读坏是哪一种（派发提示 Q2.1 的三种）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadFault {
    DeviceError,
    AllZero,
    NotSelfVerified,
}

impl ReadFault {
    fn name(self) -> &'static str {
        match self {
            ReadFault::DeviceError => "device_error",
            ReadFault::AllZero => "all_zero",
            ReadFault::NotSelfVerified => "not_self_verified",
        }
    }
}

/// 一段读坏的落点：哪块盘、从哪个字节起、多长、哪一种读坏；这一段被写过就撤掉（之后读回新字节）。
#[derive(Clone, Copy, Debug)]
struct FaultyRange {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
    fault: ReadFault,
    lifted: bool,
}

impl FaultyRange {
    fn overlaps(&self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
        self.device == device && offset < self.offset + self.length && self.offset < offset + length
    }
}

/// 几块盘共用的一份读故障。
#[derive(Clone)]
struct SharedReadFaults(Rc<RefCell<Vec<FaultyRange>>>);

impl SharedReadFaults {
    fn new(ranges: Vec<FaultyRange>) -> Self {
        Self(Rc::new(RefCell::new(ranges)))
    }

    fn active_ranges_hit(&self, device: DeviceIdentity, offset: u64, length: u64) -> Vec<FaultyRange> {
        self.0
            .borrow()
            .iter()
            .filter(|range| !range.lifted && range.overlaps(device, offset, length))
            .copied()
            .collect()
    }

    fn lift_the_ranges_written(&self, device: DeviceIdentity, offset: u64, length: u64) {
        for range in self.0.borrow_mut().iter_mut() {
            if range.overlaps(device, offset, length) {
                range.lifted = true;
            }
        }
    }
}

/// 读的时候按 [`SharedReadFaults`] 读坏的盘；写、写零、discard、屏障原样交给里面那块盘，写到的故障段撤掉。
struct DeviceWithReadFaults<Inner: BlockDevice> {
    identity: DeviceIdentity,
    inner: Inner,
    faults: SharedReadFaults,
}

impl<Inner: BlockDevice> BlockDevice for DeviceWithReadFaults<Inner> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        let hit = self.faults.active_ranges_hit(self.identity, offset.0, length);
        if hit.iter().any(|range| range.fault == ReadFault::DeviceError) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "攻方模型点名的这一段读坏",
            )));
        }
        self.inner.read_at(offset, buffer)?;
        for range in hit {
            let start = range.offset.max(offset.0) - offset.0;
            let end = (range.offset + range.length).min(offset.0 + length) - offset.0;
            let start_index = usize::try_from(start).expect("落在缓冲区里");
            let end_index = usize::try_from(end).expect("落在缓冲区里");
            match range.fault {
                ReadFault::DeviceError => unreachable!("设备报错的那一种已在上面交回"),
                ReadFault::AllZero => buffer[start_index..end_index].fill(0),
                ReadFault::NotSelfVerified => {
                    let flipped = range.offset + FLIPPED_BYTE_INSIDE_A_RANGE;
                    let flipped_index = if flipped >= offset.0 && flipped < offset.0 + length {
                        usize::try_from(flipped - offset.0).expect("落在缓冲区里")
                    } else {
                        start_index
                    };
                    buffer[flipped_index] ^= 0xff;
                }
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.faults.lift_the_ranges_written(
            self.identity,
            offset.0,
            u64::try_from(bytes.len()).expect("写长装得进 u64"),
        );
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.faults.lift_the_ranges_written(self.identity, offset.0, length);
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.faults.lift_the_ranges_written(self.identity, offset.0, length);
        self.inner.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(ring_start, ring_bytes)
    }

    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        self.inner.keep_the_self_verified_reread(offset, bytes);
    }

    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        self.inner
            .note_the_read_failing_self_verification_and_forget_its_bytes(offset, length_in_bytes);
    }
}

fn memory_pool_of_devices(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

fn sparse_devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

/// 起点历史：实例 1 到 D 为止的镜像，与造读故障要的几样（各版根与记录计数器）。
struct StartingHistory {
    image: MemoryPool,
    /// (txg, 那次发布全部记录的计数器)，txg 3 起。
    record_counters_by_txg: Vec<(u64, Vec<u64>)>,
    root_of_c: RootRecord,
}

fn record_counters_of(output: &TransactionOutput) -> Vec<u64> {
    output
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain(std::iter::once(output.record.counter))
        .collect()
}

fn build_starting_history() -> StartingHistory {
    let parameters = pool_parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), DEVICE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    let first_content = file_content_of(1);
    let (instance, first, version_b, version_c) = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile { content: &first_content, write_time_seconds: FIRST_WRITE_TIME_SECONDS },
            instance,
            &warm.last_record_bytes,
        )
        .expect("新池新建文件");
        let content_b = file_content_of(2);
        let version_b = publish_overwrite(
            &mut writer,
            &mut allocator,
            &first,
            FirstFile { content: &content_b, write_time_seconds: FIRST_WRITE_TIME_SECONDS + 60 },
            instance,
        )
        .expect("覆盖写 B");
        let content_c = file_content_of(3);
        let version_c = publish_overwrite(
            &mut writer,
            &mut allocator,
            &version_b,
            FirstFile { content: &content_c, write_time_seconds: FIRST_WRITE_TIME_SECONDS + 120 },
            instance,
        )
        .expect("覆盖写 C");
        (instance, first, version_b, version_c)
    };
    assert_eq!(version_c.root.checkpoint_txg, CheckpointTxg(TXG_OF_C), "C 是 txg 5");
    let before_d = memory_pool_of_devices(&devices);
    let content_d = file_content_of(4);
    let version_d = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &version_c,
            FirstFile { content: &content_d, write_time_seconds: FIRST_WRITE_TIME_SECONDS + 180 },
            instance,
        )
        .expect("覆盖写 D")
    };
    assert_eq!(version_d.root.checkpoint_txg, CheckpointTxg(TXG_OF_D), "D 是 txg 6");
    let mut image = memory_pool_of_devices(&devices);
    // D 的根槽 FUA 持久、系统配置轮换没持久就崩：两块盘的两个系统配置槽退回 D 之前的字节。
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in [0, SLOT_SPACING] {
            let bytes = PoolReader::read(&before_d, device, DeviceOffsetInBytes(offset), slot_bytes)
                .expect("D 之前的系统配置槽读得到");
            image
                .devices
                .get_mut(&device)
                .expect("有这块盘")
                .write(DeviceOffsetInBytes(offset), &bytes);
        }
    }
    StartingHistory {
        image,
        record_counters_by_txg: vec![
            (first.root.checkpoint_txg.0, record_counters_of(&first)),
            (version_b.root.checkpoint_txg.0, record_counters_of(&version_b)),
            (version_c.root.checkpoint_txg.0, record_counters_of(&version_c)),
            (version_d.root.checkpoint_txg.0, record_counters_of(&version_d)),
        ],
        root_of_c: version_c.root,
    }
}

/// txg 那次发布的根按落点公式住的那一槽：哪块盘、偏移。
fn root_slot_place(txg: u64) -> (RootRingSlot, DeviceIdentity, u64) {
    let parameters = pool_parameters();
    let target = target_for_publish(CheckpointTxg(txg), parameters.geometry.root_ring_slots_per_region);
    let device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    (target, device, slot_offset(target, 4096).0)
}

/// 每块盘两槽里自证过、世代号最大的那一槽的偏移（D 的轮换退回之后，它装的是 C 那次发布的 tail）。
fn newest_system_configuration_slot(image: &MemoryPool, device: DeviceIdentity) -> u64 {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut newest: Option<(u64, u64)> = None;
    for offset in [0, SLOT_SPACING] {
        let bytes = PoolReader::read(image, device, DeviceOffsetInBytes(offset), slot_bytes)
            .expect("系统配置槽读得到");
        if let Ok(configuration) = SystemConfiguration::parse_slot(&bytes) {
            let generation = configuration.quantities.slot_generation;
            if newest.is_none_or(|(best, _)| generation > best) {
                newest = Some((generation, offset));
            }
        }
    }
    newest.expect("起点镜像每块盘至少一槽自证过").1
}

/// 一格的读故障怎么给。
#[derive(Clone, Copy, Debug)]
struct FaultCell {
    /// 读坏最高的几条根（1：只有 D；2：C 与 D）连同它们那几次发布的记录（每块盘一份都读坏）。
    roots_hidden: u64,
    root_fault: ReadFault,
    /// 每块盘最新那一槽系统配置读坏成哪一种；`None` 是不读坏。
    system_configuration_fault: Option<ReadFault>,
}

impl FaultCell {
    fn label(&self) -> String {
        format!(
            "roots_hidden={} root_and_record_fault={} newest_system_configuration_fault={}",
            self.roots_hidden,
            self.root_fault.name(),
            self.system_configuration_fault.map_or("none", ReadFault::name)
        )
    }
}

fn fault_ranges_of(start: &StartingHistory, cell: FaultCell) -> Vec<FaultyRange> {
    let mut ranges = Vec::new();
    let hidden_txgs: Vec<u64> = (0..cell.roots_hidden).map(|back| TXG_OF_D - back).collect();
    for txg in &hidden_txgs {
        let (_, device, offset) = root_slot_place(*txg);
        ranges.push(FaultyRange { device, offset, length: ROOT_SLOT_BYTES, fault: cell.root_fault, lifted: false });
        let counters = &start
            .record_counters_by_txg
            .iter()
            .find(|(record_txg, _)| record_txg == txg)
            .expect("起点历史里有这一版")
            .1;
        for counter in counters {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                ranges.push(FaultyRange {
                    device,
                    offset: record_offset(*counter, JOURNAL_RING_DEFAULT_BYTES).0,
                    length: JOURNAL_RECORD_BYTES,
                    fault: cell.root_fault,
                    lifted: false,
                });
            }
        }
    }
    if let Some(fault) = cell.system_configuration_fault {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            ranges.push(FaultyRange {
                device,
                offset: newest_system_configuration_slot(&start.image, device),
                length: SYSTEM_CONFIGURATION_SLOT_BYTES,
                fault,
                lifted: false,
            });
        }
    }
    ranges
}

fn short_name_of_debug(text: &str) -> String {
    let cut = text.find(['(', '{', ' ']).unwrap_or(text.len());
    text[..cut].to_string()
}

/// 实例 2 在读故障下的那一次可写挂载：结局、录下的写表与段、写表每一项在录制流里的下标、流里屏障的下标。
struct MountUnderFaults {
    outcome: String,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    stream_indexes: Vec<usize>,
    barrier_stream_indexes: Vec<usize>,
}

fn mount_under_faults(start: &MemoryPool, ranges: Vec<FaultyRange>) -> MountUnderFaults {
    let stream = SharedStream::retaining_contents();
    let faults = SharedReadFaults::new(ranges);
    let mut devices: Vec<(DeviceIdentity, DeviceWithReadFaults<RecordingBlockDevice<SparseBlockDevice>>)> =
        sparse_devices_of(start)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    DeviceWithReadFaults {
                        identity,
                        inner: RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                        faults: faults.clone(),
                    },
                )
            })
            .collect();
    let outcome = match mount_writable(&pool_parameters(), &mut devices) {
        Ok(mounted) => format!(
            "ok instance={} chosen=({},{}) effective=({},{}) row_txg={}",
            mounted.output.instance.0,
            mounted.output.chosen_root.instance.0,
            mounted.output.chosen_root.checkpoint_txg.0,
            mounted.output.effective_root.instance.0,
            mounted.output.effective_root.checkpoint_txg.0,
            mounted.output.row_publish.root().checkpoint_txg.0
        ),
        Err(error) => format!("refused:{}", short_name_of_debug(&format!("{error:?}"))),
    };
    let operations = stream.retained_operations();
    let barrier_stream_indexes = operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind == RecordedOperationKind::Barrier)
        .map(|(index, _)| index)
        .collect();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &fixed_geometry());
    MountUnderFaults { outcome, writes, segments, stream_indexes, barrier_stream_indexes }
}

/// 择根序：基线与 `crates/mutations.tsv` 里三条改坏择根序的变体（逐槽往后，比已择的「新」才换）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootChoiceOrder {
    Baseline,
    /// v1823 择根核比较不看实例代号：txg 平局时停在读根环次序里前面那条。
    IgnoringTheInstance,
    /// v1825 择根核 txg 平局时实例代号低者赢。
    LowerInstanceWinsATie,
    /// v1826 择根核以实例代号为主、txg 只破平局。
    InstanceFirst,
}

const VARIANT_ORDERS: [(RootChoiceOrder, &str); 3] = [
    (RootChoiceOrder::IgnoringTheInstance, "v1823"),
    (RootChoiceOrder::LowerInstanceWinsATie, "v1825"),
    (RootChoiceOrder::InstanceFirst, "v1826"),
];

/// 根环全部槽按读根环次序的 (实例代号, txg)；读坏（读不出、不是全零而自证不过、全零、错位）的是 `None`。
fn ring_keys(image: &MemoryPool, configuration: &SystemConfiguration) -> Vec<Option<(u32, u64)>> {
    every_root_ring_slot(&configuration.immutable.sizes)
        .into_iter()
        .map(|ring_slot| {
            match read_root_ring_slot(
                image,
                &configuration.immutable.region_devices,
                &configuration.immutable.sizes,
                &configuration.immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some((root.instance.0, root.checkpoint_txg.0)),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect()
}

fn choose_under(keys: &[Option<(u32, u64)>], order: RootChoiceOrder) -> Option<(u32, u64)> {
    let mut chosen: Option<(u32, u64)> = None;
    for (instance, txg) in keys.iter().flatten().copied() {
        let newer = match chosen {
            None => true,
            Some((chosen_instance, chosen_txg)) => match order {
                RootChoiceOrder::Baseline => {
                    txg > chosen_txg || (txg == chosen_txg && instance > chosen_instance)
                }
                RootChoiceOrder::IgnoringTheInstance => txg > chosen_txg,
                RootChoiceOrder::LowerInstanceWinsATie => {
                    txg > chosen_txg || (txg == chosen_txg && instance < chosen_instance)
                }
                RootChoiceOrder::InstanceFirst => {
                    instance > chosen_instance || (instance == chosen_instance && txg > chosen_txg)
                }
            },
        };
        if newer {
            chosen = Some((instance, txg));
        }
    }
    chosen
}

/// (形甲在不在, 形乙在不在)，按同时自证过的根判（择根不按实例表滤，`recovery::choose_root`）。
fn forms_in(keys: &[Option<(u32, u64)>]) -> (bool, bool) {
    let roots: Vec<(u32, u64)> = keys.iter().flatten().copied().collect();
    let mut form_a = false;
    let mut form_b = false;
    for left in &roots {
        for right in &roots {
            if left.1 == right.1 && left.0 != right.0 {
                form_a = true;
            }
            if left.0 > right.0 && left.1 < right.1 {
                form_b = true;
            }
        }
    }
    (form_a, form_b)
}

fn key_text(key: Option<(u32, u64)>) -> String {
    key.map_or_else(|| "none".to_string(), |(instance, txg)| format!("({instance},{txg})"))
}

fn violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 一个镜像上看到的：形、各序择哪条、真 `choose_root` 择哪条、checker 红哪几条、没有读故障时基线可写挂载落到哪一版。
struct StateReading {
    form_a: bool,
    form_b: bool,
    baseline: Option<(u32, u64)>,
    variants: Vec<(&'static str, Option<(u32, u64)>)>,
    choose_root_of_the_core: Option<(u32, u64)>,
    violated: Vec<&'static str>,
    mount_after: String,
    readable_roots: String,
}

fn read_a_state(image: &MemoryPool, run_the_checker_and_the_mount: bool) -> Result<StateReading, String> {
    let configuration = choose_system_configuration(image).map_err(|failure| {
        format!("system_configuration_unchosen:{}", short_name_of_debug(&format!("{failure:?}")))
    })?;
    let keys = ring_keys(image, &configuration);
    let (form_a, form_b) = forms_in(&keys);
    let baseline = choose_under(&keys, RootChoiceOrder::Baseline);
    let variants = VARIANT_ORDERS
        .iter()
        .map(|(order, name)| (*name, choose_under(&keys, *order)))
        .collect();
    let choose_root_of_the_core = choose_root(image, &configuration)
        .map(|root| (root.instance.0, root.checkpoint_txg.0));
    let (violated, mount_after) = if run_the_checker_and_the_mount {
        let violated = violated_invariants(image);
        let mut devices = sparse_devices_of(image);
        let mount_after = match mount_writable(&pool_parameters(), &mut devices) {
            Ok(mounted) => format!(
                "ok:chosen=({},{}),effective=({},{})",
                mounted.output.chosen_root.instance.0,
                mounted.output.chosen_root.checkpoint_txg.0,
                mounted.output.effective_root.instance.0,
                mounted.output.effective_root.checkpoint_txg.0
            ),
            Err(error) => format!("refused:{}", short_name_of_debug(&format!("{error:?}"))),
        };
        (violated, mount_after)
    } else {
        (Vec::new(), "not_run".to_string())
    };
    let readable_roots = keys.iter().flatten().map(|key| key_text(Some(*key))).collect::<Vec<_>>().join(",");
    Ok(StateReading { form_a, form_b, baseline, variants, choose_root_of_the_core, violated, mount_after, readable_roots })
}

/// 照崩溃点重放的枚举域逐个交出崩溃状态（持久了哪几条写）：第 k 段之前整段持久、第 k 段取一个真子集（含空集），最后再加全部持久；
/// 个数 = 1 + Σ(2^|段| − 1)（`closed_form_state_count`）。原地覆写的撕裂第三态不在里面（报告「限度」一节）。
fn for_each_crash_state(write_count: usize, segments: &[Vec<usize>], mut visit: impl FnMut(&[bool], &str)) {
    let mut persisted = vec![false; write_count];
    for (segment_index, segment) in segments.iter().enumerate() {
        let width = u32::try_from(segment.len()).expect("段宽");
        for mask in 0..((1u64 << width) - 1) {
            let mut state = persisted.clone();
            for (bit, write_index) in segment.iter().enumerate() {
                if (mask >> bit) & 1 == 1 {
                    state[*write_index] = true;
                }
            }
            visit(&state, &format!("segment={segment_index}/mask={mask:b}"));
        }
        for write_index in segment {
            persisted[*write_index] = true;
        }
    }
    visit(&persisted, "all");
}

fn image_of_the_state(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut image = base.clone();
    for (write, is_persisted) in writes.iter().zip(persisted) {
        if *is_persisted {
            image.apply_writes(std::slice::from_ref(write));
        }
    }
    image
}

/// 实例 2 第一次发布（写行那一次）在写表里占哪几条：从它的根槽 FUA 写往后到下一道屏障之前的全部写，连同根槽写之前的全部写。
/// 交回 (第一次发布的 txg, 那几条写的下标)；这次挂载没写根就 `None`。
fn first_publish_of_the_new_instance(mount: &MountUnderFaults) -> Option<(u64, Vec<usize>)> {
    let (root_write_index, txg) = mount.writes.iter().enumerate().find_map(|(index, write)| {
        (write.kind == StepKind::RootRecordFua).then(|| (index, root_identity_of_write(write).1 .0))
    })?;
    let root_stream_index = mount.stream_indexes[root_write_index];
    let closing_barrier = mount
        .barrier_stream_indexes
        .iter()
        .copied()
        .find(|index| *index > root_stream_index)
        .unwrap_or(usize::MAX);
    let writes_of_it = (0..mount.writes.len())
        .filter(|index| mount.stream_indexes[*index] < closing_barrier)
        .collect();
    Some((txg, writes_of_it))
}

/// 一格的计数。
#[derive(Default)]
struct CellTally {
    states: u64,
    unchosen_system_configuration: u64,
    form_a: u64,
    form_b: u64,
    form_b_with_the_first_publish_complete: u64,
    variant_differs: [u64; 3],
    core_disagrees_with_the_local_baseline: u64,
    checker_red_states: u64,
    mount_after_outcomes: std::collections::BTreeMap<String, u64>,
    red_invariants: std::collections::BTreeMap<&'static str, u64>,
}

fn run_a_cell(start: &StartingHistory, cell: FaultCell, detail_lines_left: &mut u64) -> (CellTally, f64) {
    let began = Instant::now();
    let mount = mount_under_faults(&start.image, fault_ranges_of(start, cell));
    let first_publish = first_publish_of_the_new_instance(&mount);
    let state_count = closed_form_state_count(&mount.segments);
    let widest_segment = mount.segments.iter().map(Vec::len).max().unwrap_or(0);
    println!(
        "OPUSMODEL name=cell_mount {} h2_mount={} writes={} segments={} widest_segment={} states={} first_publish_txg={}",
        cell.label(),
        mount.outcome,
        mount.writes.len(),
        mount.segments.len(),
        widest_segment,
        state_count,
        first_publish.as_ref().map_or_else(|| "none".to_string(), |(txg, _)| txg.to_string())
    );
    let mut tally = CellTally::default();
    if state_count > STATES_OF_ONE_HISTORY_LIMIT {
        println!("OPUSMODEL name=cell_skipped {} states={state_count} limit={STATES_OF_ONE_HISTORY_LIMIT}", cell.label());
        return (tally, began.elapsed().as_secs_f64());
    }
    for_each_crash_state(mount.writes.len(), &mount.segments, |persisted, state_label| {
        tally.states += 1;
        let image = image_of_the_state(&start.image, &mount.writes, persisted);
        let reading = match read_a_state(&image, true) {
            Ok(reading) => reading,
            Err(why) => {
                tally.unchosen_system_configuration += 1;
                println!("OPUSMODEL name=state_unread {} state={state_label} why={why}", cell.label());
                return;
            }
        };
        let first_publish_complete = first_publish
            .as_ref()
            .is_some_and(|(_, writes_of_it)| writes_of_it.iter().all(|index| persisted[*index]));
        tally.form_a += u64::from(reading.form_a);
        tally.form_b += u64::from(reading.form_b);
        tally.form_b_with_the_first_publish_complete += u64::from(reading.form_b && first_publish_complete);
        for (slot, (_, chosen)) in reading.variants.iter().enumerate() {
            tally.variant_differs[slot] += u64::from(*chosen != reading.baseline);
        }
        tally.core_disagrees_with_the_local_baseline +=
            u64::from(reading.choose_root_of_the_core != reading.baseline);
        tally.checker_red_states += u64::from(!reading.violated.is_empty());
        for invariant in &reading.violated {
            *tally.red_invariants.entry(*invariant).or_insert(0) += 1;
        }
        *tally.mount_after_outcomes.entry(reading.mount_after.clone()).or_insert(0) += 1;
        if (reading.form_a || reading.form_b) && *detail_lines_left > 0 {
            *detail_lines_left -= 1;
            println!(
                "OPUSMODEL name=form_state {} state={state_label} form_a={} form_b={} first_publish_complete={first_publish_complete} roots=[{}] baseline={} {} core_choose_root={} checker_red=[{}] mount_after={}",
                cell.label(),
                reading.form_a,
                reading.form_b,
                reading.readable_roots,
                key_text(reading.baseline),
                reading.variants.iter().map(|(name, chosen)| format!("{name}={}", key_text(*chosen))).collect::<Vec<_>>().join(" "),
                key_text(reading.choose_root_of_the_core),
                reading.violated.join(","),
                reading.mount_after
            );
        }
    });
    (tally, began.elapsed().as_secs_f64())
}

fn print_the_cell(cell: FaultCell, tally: &CellTally, seconds: f64) {
    let joined = |map: &std::collections::BTreeMap<String, u64>| {
        map.iter().map(|(key, count)| format!("{key}:{count}")).collect::<Vec<_>>().join(",")
    };
    let red = tally
        .red_invariants
        .iter()
        .map(|(invariant, count)| format!("{invariant}:{count}"))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "OPUSMODEL name=cell {} states={} unchosen_system_configuration={} form_a={} form_b={} form_b_first_publish_complete={} v1823_differs={} v1825_differs={} v1826_differs={} core_disagrees_with_local_baseline={} checker_red_states={} red_invariants=[{red}] mount_after=[{}] seconds={seconds:.1}",
        cell.label(),
        tally.states,
        tally.unchosen_system_configuration,
        tally.form_a,
        tally.form_b,
        tally.form_b_with_the_first_publish_complete,
        tally.variant_differs[0],
        tally.variant_differs[1],
        tally.variant_differs[2],
        tally.core_disagrees_with_the_local_baseline,
        tally.checker_red_states,
        joined(&tally.mount_after_outcomes)
    );
}

/// 会红的对照：喂一个已知造得出形乙的写者变体（第一次发布落在读得出的最大 txg 上、不 + 1：在起点镜像上把 txg 5 那一槽换成实例 2 的根），
/// 模型要报形乙、v1826 与基线择得不同；再喂一个把同号根写错位的写者，读根环那一道（C743）挡掉、形甲不出现。
fn run_the_controls(start: &StartingHistory) -> bool {
    let mut forged = start.image.clone();
    let mut low_root = start.root_of_c;
    low_root.instance = InstanceGeneration(2);
    let (_, device, offset) = root_slot_place(TXG_OF_C);
    forged
        .devices
        .get_mut(&device)
        .expect("有这块盘")
        .write(DeviceOffsetInBytes(offset), &low_root.to_slot(usize::try_from(ROOT_SLOT_BYTES).expect("512")));
    let low = read_a_state(&forged, true).expect("对照镜像择得到系统配置");
    let v1826 = low.variants.iter().find(|(name, _)| *name == "v1826").expect("有 v1826").1;
    let low_red = low.form_b && v1826 != low.baseline;
    println!(
        "OPUSMODEL name=control kind=writer_variant_first_txg_not_above_the_unreadable_root form_a={} form_b={} roots=[{}] baseline={} v1826={} checker_red=[{}] mount_after={} verdict={}",
        low.form_a,
        low.form_b,
        low.readable_roots,
        key_text(low.baseline),
        key_text(v1826),
        low.violated.join(","),
        low.mount_after,
        if low_red { "red_as_expected" } else { "MODEL_BROKEN" }
    );
    let mut misplaced = start.image.clone();
    let mut same_txg_root = start.root_of_c;
    same_txg_root.instance = InstanceGeneration(2);
    same_txg_root.checkpoint_txg = CheckpointTxg(TXG_OF_D);
    let (_, device, offset) = root_slot_place(TXG_OF_D + 1);
    misplaced
        .devices
        .get_mut(&device)
        .expect("有这块盘")
        .write(DeviceOffsetInBytes(offset), &same_txg_root.to_slot(usize::try_from(ROOT_SLOT_BYTES).expect("512")));
    let same = read_a_state(&misplaced, false).expect("对照镜像择得到系统配置");
    println!(
        "OPUSMODEL name=control kind=writer_variant_same_txg_in_another_slot form_a={} form_b={} roots=[{}] verdict={}",
        same.form_a,
        same.form_b,
        same.readable_roots,
        if same.form_a { "form_a_reaches_the_choice" } else { "misplaced_root_read_as_bad_form_a_absent" }
    );
    low_red
}

fn every_cell() -> Vec<FaultCell> {
    let kinds = [ReadFault::DeviceError, ReadFault::AllZero, ReadFault::NotSelfVerified];
    let mut cells = vec![FaultCell { roots_hidden: 0, root_fault: ReadFault::DeviceError, system_configuration_fault: None }];
    for roots_hidden in [1, 2] {
        for root_fault in kinds {
            for system_configuration_fault in [None, Some(ReadFault::DeviceError), Some(ReadFault::AllZero), Some(ReadFault::NotSelfVerified)] {
                cells.push(FaultCell { roots_hidden, root_fault, system_configuration_fault });
            }
        }
    }
    cells
}

/// 先跑一小段估全量挂钟：对照、一格全量（读坏两条根、全零、最新那一槽系统配置全零），其余各格只做那一次挂载、数状态。
#[test]
fn e165_attack_probe() {
    let began = Instant::now();
    let start = build_starting_history();
    println!("OPUSMODEL name=probe starting_history_seconds={:.1}", began.elapsed().as_secs_f64());
    assert!(run_the_controls(&start), "对照没红：模型判形乙的那一段坏了");
    let probe_cell = FaultCell { roots_hidden: 2, root_fault: ReadFault::AllZero, system_configuration_fault: Some(ReadFault::AllZero) };
    let mut detail_lines_left = 3;
    let (tally, seconds) = run_a_cell(&start, probe_cell, &mut detail_lines_left);
    print_the_cell(probe_cell, &tally, seconds);
    let seconds_per_state = seconds / f64::from(u32::try_from(tally.states.max(1)).expect("状态数"));
    let mut total_states = 0u64;
    for cell in every_cell() {
        let mount = mount_under_faults(&start.image, fault_ranges_of(&start, cell));
        let states = closed_form_state_count(&mount.segments);
        total_states += states.min(STATES_OF_ONE_HISTORY_LIMIT);
        println!("OPUSMODEL name=probe_count {} h2_mount={} states={states}", cell.label(), mount.outcome);
    }
    println!(
        "OPUSMODEL name=estimate cells={} total_states={total_states} seconds_per_state={seconds_per_state:.3} projected_full_seconds={:.0} probe_seconds={:.1}",
        every_cell().len(),
        seconds_per_state * f64::from(u32::try_from(total_states).expect("状态数")),
        began.elapsed().as_secs_f64()
    );
}

/// 全量：对照与全部格，每一格这次挂载的崩溃状态一个不落。
#[test]
#[ignore = "全量：先跑 e165_attack_probe 估时，估出来不超过 40 分钟再跑"]
fn e165_attack_full() {
    let began = Instant::now();
    let start = build_starting_history();
    assert!(run_the_controls(&start), "对照没红：模型判形乙的那一段坏了");
    let mut detail_lines_left = 40;
    let mut cells_with_form_b = 0u64;
    for cell in every_cell() {
        let (tally, seconds) = run_a_cell(&start, cell, &mut detail_lines_left);
        print_the_cell(cell, &tally, seconds);
        cells_with_form_b += u64::from(tally.form_b > 0);
    }
    println!(
        "OPUSMODEL name=done cells={} cells_with_form_b={cells_with_form_b} seconds={:.1}",
        every_cell().len(),
        began.elapsed().as_secs_f64()
    );
}
