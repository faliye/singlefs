//! 攻方腿模型，照规矩改跑法的那一份（三方轮 m3-close-b19-e165-r1；承接 research/prompts/m3-close-b19-e165-r1-opus-output.md，
//! 前一份模型 research/prompts/m3-close-b19-e165-r1-opus-model/e165_attack_form_b.rs 不动）。判的东西与前一份相同：
//! 可写挂载那一刻最高几条根所在的槽读坏、它们那几次发布的记录也读不到、两块盘最新那一槽系统配置读回全零或自证不过时，
//! 新实例第一次发布的 txg 会不会不大于那条根，之后读故障消失，盘上是不是形甲（同 txg、不同实例）或形乙
//! （实例代号更高而 txg 更低的一条根与实例代号低而 txg 高的一条根同在），基线与三条改坏择根序的变体各择哪一条。
//!
//! 复跑时把这份拷进副本的 crates/singlefs-harness/tests/，把 scripts/experiment-apparatus/heavy_threads_default/threads.rs
//! 拷成同目录下的 e165_attack_support/threads.rs（子目录里的不成测试目标），副本根上要有 .claude/heavy-thread-default。
//! 盘是内存稀疏盘（SparseBlockDevice），起点历史每条用例只建一次，每一格从那一份克隆。
//!
//! 起点历史（真写者）：mkfs 两块盘 → 实例 1 取号、暖机（txg 1、2）→ 新池新建文件（txg 3）→ 覆盖写 B（txg 4）、C（txg 5）、
//! D（txg 6）；D 的根槽 FUA 持久、系统配置轮换没持久就崩（把两块盘的系统配置槽退回 D 之前的字节）。C 完整发布、已确认；D 没确认。
//!
//! 四条用例：
//!   e165_attack_minimal_history_fixed_crash_point  报告第一节那段最小历史，崩溃点固定在实例 2 暖机写 txg 6 的根之前（不枚举）；
//!                                                  根槽三种读坏 × 系统配置槽两种读坏，逐格一行。
//!   e165_attack_estimate                           对照、二十五格各数一次状态、读坏两条根那一格按等距抽 2 万个崩溃状态多线程判，
//!                                                  按抽样的挂钟推全量。
//!   e165_attack_sliced_matches_single_thread       同一个小域（600 个崩溃状态）单线程与切片多线程各判一遍，逐态读数与合并后的计数逐字相同。
//!   e165_attack_full                               #[ignore]：二十五格每格挂载写流的崩溃状态一个不落，切片多线程，每 5 分钟每片报一行进度。
//!
//! 枚举域：照崩溃点重放的枚举域（屏障与 FUA 切段，第 k 段之前整段持久、第 k 段取一个真子集，最后再加全部持久），
//! 个数等于 `closed_form_state_count`；原地覆写的撕裂第三态不在里面（报告「限度」一节）。
#![allow(clippy::too_many_lines, reason = "模型一份文件，逐格的历史与判法写在一起")]

#[path = "e165_attack_support/threads.rs"]
mod heavy_threads_default;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Range;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
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
use singlefs_core::mounted_read::open_pool_for_read;
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
/// 这一条腿连同前一条攻方腿的原型全量合起来不超过约 10⁷ 个状态。
const STATES_BUDGET_OF_BOTH_LEGS: u64 = 10_000_000;
/// 前一条腿的估时用例在任务 7230、7436 里被撤之前，至多各判完一格 393 241 个状态（`probe-7230-before-cancel.txt` 的 cell_mount 行）。
const STATES_SPENT_BY_THE_EARLIER_LEG: u64 = 2 * 393_241;
/// 估时用例抽判的崩溃状态数（日常验证压在 20 万以内）。
const ESTIMATE_SAMPLE_STATES: u64 = 20_000;
/// 单线程与切片多线程对拍的小域。
const CROSS_CHECK_SAMPLE_STATES: u64 = 600;
/// 最小历史那条用例的格数（根槽三种读坏 × 系统配置槽两种读坏），每格一个崩溃状态。
const MINIMAL_HISTORY_CELLS: u64 = 6;
/// 每片至少每过这么久报一行进度。
const PROGRESS_INTERVAL: Duration = Duration::from_secs(300);
/// 每格最多打几行带形的崩溃状态（按状态下标取最小的那几个）。
const DETAIL_LINES_PER_CELL: usize = 3;
/// 起点历史里 D 那一版的 txg（根槽持久、轮换没持久）与 C 那一版（完整发布、已确认）。
const TXG_OF_D: u64 = 6;
const TXG_OF_C: u64 = 5;
/// 第一个文件的 inode 号。
const FIRST_FILE: InodeNumber = InodeNumber(1);
/// 起点历史里各版文件内容的种子：新池新建文件、B、C、D。
const SEED_OF_THE_FIRST_FILE: u64 = 1;
const SEED_OF_B: u64 = 2;
const SEED_OF_C: u64 = 3;
const SEED_OF_D: u64 = 4;

fn registered_threads() -> usize {
    heavy_threads_default::default_heavy_threads(include_str!("../../../.claude/heavy-thread-default"))
}

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

/// 根环全部槽按读根环次序读出来的根；读坏（读不出、不是全零而自证不过、全零、错位）的是 `None`。
fn ring_roots(image: &MemoryPool, configuration: &SystemConfiguration) -> Vec<Option<RootRecord>> {
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
                RootRingSlotReading::SelfVerified(root) => Some(root),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect()
}

/// 每一槽的 (实例代号, txg)。
fn keys_of(roots: &[Option<RootRecord>]) -> Vec<Option<(u32, u64)>> {
    roots
        .iter()
        .map(|root| root.as_ref().map(|root| (root.instance.0, root.checkpoint_txg.0)))
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


/// 读出来的字节是起点历史里哪一版的内容。
fn name_of_the_content(bytes: &[u8]) -> &'static str {
    [
        (SEED_OF_THE_FIRST_FILE, "first_file"),
        (SEED_OF_B, "version_b"),
        (SEED_OF_C, "version_c"),
        (SEED_OF_D, "version_d"),
    ]
    .into_iter()
    .find(|(seed, _)| file_content_of(*seed) == bytes)
    .map_or("other", |(_, name)| name)
}

/// 按这一条根打开、读第一个文件整份：读出的是哪一版，或读不出（带卡在哪一步与错误名）。
fn content_under(image: &MemoryPool, root: &RootRecord) -> String {
    let mounted = match open_pool_for_read(image, root) {
        Ok(mounted) => mounted,
        Err(error) => return format!("unreadable:open:{}", short_name_of_debug(&format!("{error:?}"))),
    };
    let file = match mounted.open_file(image, FIRST_FILE) {
        Ok(file) => file,
        Err(error) => return format!("unreadable:file:{}", short_name_of_debug(&format!("{error:?}"))),
    };
    let size = file.inode_record().size;
    match file.read_at(image, FileOffsetInBytes(0), size) {
        Ok(output) => name_of_the_content(&output.bytes).to_string(),
        Err(error) => format!("unreadable:read:{}", short_name_of_debug(&format!("{error:?}"))),
    }
}

/// 已确认的 C 丢了：读出来既不是 C、也不是叠在 C 之上的 D（D 没确认，盖过 C 不算丢）。
fn confirmed_c_lost(content: &str) -> bool {
    content != "version_c" && content != "version_d"
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
    let keys = keys_of(&ring_roots(image, &configuration));
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

/// 枚举域里第 `index` 个崩溃状态（持久了哪几条写）与它的名字：第 k 段之前整段持久、第 k 段取掩码 0..2^|段|−1 的真子集（含空集），
/// 段按次序排；最后一个（下标 = 状态数 − 1）是全部持久。个数 = 1 + Σ(2^|段| − 1)（`closed_form_state_count`），次序与前一份模型的
/// `for_each_crash_state` 逐个相同。
fn crash_state_at(index: u64, write_count: usize, segments: &[Vec<usize>]) -> (Vec<bool>, String) {
    let mut persisted = vec![false; write_count];
    let mut remaining = index;
    for (segment_index, segment) in segments.iter().enumerate() {
        let proper_subsets = (1u64 << u32::try_from(segment.len()).expect("段宽")) - 1;
        if remaining < proper_subsets {
            for (bit, write_index) in segment.iter().enumerate() {
                if (remaining >> bit) & 1 == 1 {
                    persisted[*write_index] = true;
                }
            }
            return (persisted, format!("segment={segment_index}/mask={remaining:b}"));
        }
        remaining -= proper_subsets;
        for write_index in segment {
            persisted[*write_index] = true;
        }
    }
    assert_eq!(remaining, 0, "下标越过枚举域");
    (persisted, "all".to_string())
}

/// 判哪几个崩溃状态：下标 0、stride、2·stride……共 `count` 个，都小于 `total`。stride 为 1 就是枚举域全部。
#[derive(Clone, Copy, Debug)]
struct StateSelection {
    total: u64,
    stride: u64,
    count: u64,
}

impl StateSelection {
    fn every_state(total: u64) -> Self {
        Self { total, stride: 1, count: total }
    }

    /// 等距抽至多 `at_most` 个；域本来就不大于它时取全部。
    fn evenly_spaced(total: u64, at_most: u64) -> Self {
        if total <= at_most {
            return Self::every_state(total);
        }
        let stride = total.div_ceil(at_most);
        Self { total, stride, count: total.div_ceil(stride) }
    }

    fn state_index(self, position: u64) -> u64 {
        position * self.stride
    }

    fn label(self) -> String {
        format!("selected={} stride={} domain={}", self.count, self.stride, self.total)
    }
}

/// 一格（或一格的一片）的计数。合并按片的次序相加。
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
    mount_after_outcomes: BTreeMap<String, u64>,
    red_invariants: BTreeMap<&'static str, u64>,
}

impl CellTally {
    fn absorb(&mut self, later: CellTally) {
        self.states += later.states;
        self.unchosen_system_configuration += later.unchosen_system_configuration;
        self.form_a += later.form_a;
        self.form_b += later.form_b;
        self.form_b_with_the_first_publish_complete += later.form_b_with_the_first_publish_complete;
        for (mine, theirs) in self.variant_differs.iter_mut().zip(later.variant_differs) {
            *mine += theirs;
        }
        self.core_disagrees_with_the_local_baseline += later.core_disagrees_with_the_local_baseline;
        self.checker_red_states += later.checker_red_states;
        for (outcome, count) in later.mount_after_outcomes {
            *self.mount_after_outcomes.entry(outcome).or_insert(0) += count;
        }
        for (invariant, count) in later.red_invariants {
            *self.red_invariants.entry(invariant).or_insert(0) += count;
        }
    }

    /// 合并之后的计数（不带挂钟）：对拍逐字比的就是它。
    fn text(&self, label: &str, selection: StateSelection) -> String {
        let mount_after = self
            .mount_after_outcomes
            .iter()
            .map(|(outcome, count)| format!("{outcome}:{count}"))
            .collect::<Vec<_>>()
            .join(",");
        let red = self
            .red_invariants
            .iter()
            .map(|(invariant, count)| format!("{invariant}:{count}"))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "OPUSMODEL name=cell {label} {} states={} unchosen_system_configuration={} form_a={} form_b={} form_b_first_publish_complete={} v1823_differs={} v1825_differs={} v1826_differs={} core_disagrees_with_local_baseline={} checker_red_states={} red_invariants=[{red}] mount_after=[{mount_after}]",
            selection.label(),
            self.states,
            self.unchosen_system_configuration,
            self.form_a,
            self.form_b,
            self.form_b_with_the_first_publish_complete,
            self.variant_differs[0],
            self.variant_differs[1],
            self.variant_differs[2],
            self.core_disagrees_with_the_local_baseline,
            self.checker_red_states
        )
    }
}

/// 一格要判的东西：起点镜像、这次挂载的写表与段、实例 2 第一次发布占哪几条写、判哪几个状态。几个线程共读它，不改。
struct CellJob<'start> {
    label: String,
    base: &'start MemoryPool,
    mount: &'start MountUnderFaults,
    first_publish: Option<(u64, Vec<usize>)>,
    selection: StateSelection,
}

/// 一片（或合并之后）判出来的：计数、按下标最小的几行带形的状态、（对拍时）逐态一行。
#[derive(Default)]
struct SliceResult {
    tally: CellTally,
    detail_lines: Vec<String>,
    every_state_lines: Vec<String>,
}

/// 一个崩溃状态的读数写成一行；对拍逐字比，带形的那几行另打。
fn state_line(kind: &str, label: &str, state_label: &str, reading: &StateReading, first_publish_complete: bool) -> String {
    format!(
        "OPUSMODEL name={kind} {label} state={state_label} form_a={} form_b={} first_publish_complete={first_publish_complete} roots=[{}] baseline={} {} core_choose_root={} checker_red=[{}] mount_after={}",
        reading.form_a,
        reading.form_b,
        reading.readable_roots,
        key_text(reading.baseline),
        reading.variants.iter().map(|(name, chosen)| format!("{name}={}", key_text(*chosen))).collect::<Vec<_>>().join(" "),
        key_text(reading.choose_root_of_the_core),
        reading.violated.join(","),
        reading.mount_after
    )
}

/// 判一片：位置 `positions` 里每一个崩溃状态都恢复出镜像、读全部根槽、跑池级 checker 与一次没有读故障的可写挂载。
fn judge_a_slice(
    job: &CellJob<'_>,
    positions: Range<u64>,
    judged: &AtomicU64,
    keep_every_state_line: bool,
    detail_limit: usize,
) -> SliceResult {
    let mut result = SliceResult::default();
    for position in positions {
        let index = job.selection.state_index(position);
        let (persisted, state_label) = crash_state_at(index, job.mount.writes.len(), &job.mount.segments);
        let image = image_of_the_state(job.base, &job.mount.writes, &persisted);
        result.tally.states += 1;
        match read_a_state(&image, true) {
            Err(why) => {
                result.tally.unchosen_system_configuration += 1;
                let line = format!("OPUSMODEL name=state_unread {} state={state_label} why={why}", job.label);
                if result.detail_lines.len() < detail_limit {
                    result.detail_lines.push(line.clone());
                }
                if keep_every_state_line {
                    result.every_state_lines.push(line);
                }
            }
            Ok(reading) => {
                let first_publish_complete = job
                    .first_publish
                    .as_ref()
                    .is_some_and(|(_, writes_of_it)| writes_of_it.iter().all(|write_index| persisted[*write_index]));
                let tally = &mut result.tally;
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
                if (reading.form_a || reading.form_b) && result.detail_lines.len() < detail_limit {
                    result.detail_lines.push(state_line("form_state", &job.label, &state_label, &reading, first_publish_complete));
                }
                if keep_every_state_line {
                    result.every_state_lines.push(state_line("state", &job.label, &state_label, &reading, first_publish_complete));
                }
            }
        }
        judged.fetch_add(1, Ordering::Relaxed);
    }
    result
}

/// 每过 [`PROGRESS_INTERVAL`] 每片报一行：区间（位置）与已判的数。全部片的发送端都丢了（片都判完）就收。
fn report_progress_until_every_slice_finishes(
    label: &str,
    selection: StateSelection,
    bounds: &[(u64, u64)],
    judged: &[AtomicU64],
    finished: &mpsc::Receiver<()>,
) {
    let began = Instant::now();
    loop {
        match finished.recv_timeout(PROGRESS_INTERVAL) {
            Err(mpsc::RecvTimeoutError::Timeout) => {
                for (slice, ((low, high), judged_of_the_slice)) in bounds.iter().zip(judged).enumerate() {
                    println!(
                        "OPUSMODEL name=progress {label} {} slice={slice} positions=[{low},{high}) judged={} elapsed_seconds={:.0}",
                        selection.label(),
                        judged_of_the_slice.load(Ordering::Relaxed),
                        began.elapsed().as_secs_f64()
                    );
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Ok(()) => {}
        }
    }
}

/// 按区间把选中的位置切成至多 `threads` 片，`std::thread::scope` 里每片一个线程判，另起一个线程报进度；
/// 合并按片的次序：计数相加，带形的行与逐态行按片的次序接起来（片按位置升序，接起来就是按下标升序），带形的行再截到前 `detail_limit` 行。
fn judge_the_selection(job: &CellJob<'_>, threads: usize, keep_every_state_line: bool, detail_limit: usize) -> SliceResult {
    let count = job.selection.count;
    let slice_count = u64::try_from(threads.max(1)).expect("线程数装得进 u64").min(count.max(1));
    let bounds: Vec<(u64, u64)> = (0..slice_count)
        .map(|slice| (count * slice / slice_count, count * (slice + 1) / slice_count))
        .collect();
    let judged: Vec<AtomicU64> = bounds.iter().map(|_| AtomicU64::new(0)).collect();
    let results: Vec<SliceResult> = std::thread::scope(|scope| {
        let (finished_sender, finished_receiver) = mpsc::channel::<()>();
        let label = job.label.as_str();
        let bounds_of_the_slices = bounds.as_slice();
        let judged_of_the_slices = judged.as_slice();
        let selection = job.selection;
        let reporter = scope.spawn(move || {
            report_progress_until_every_slice_finishes(
                label,
                selection,
                bounds_of_the_slices,
                judged_of_the_slices,
                &finished_receiver,
            );
        });
        let workers: Vec<_> = bounds
            .iter()
            .zip(&judged)
            .map(|(&(low, high), judged_of_the_slice)| {
                let sender = finished_sender.clone();
                scope.spawn(move || {
                    let result = judge_a_slice(job, low..high, judged_of_the_slice, keep_every_state_line, detail_limit);
                    drop(sender);
                    result
                })
            })
            .collect();
        drop(finished_sender);
        let results: Vec<SliceResult> = workers
            .into_iter()
            .map(|worker| worker.join().expect("判一片的线程 panic 了"))
            .collect();
        reporter.join().expect("报进度的线程 panic 了");
        results
    });
    let mut merged = SliceResult::default();
    for result in results {
        merged.tally.absorb(result.tally);
        merged.detail_lines.extend(result.detail_lines);
        merged.every_state_lines.extend(result.every_state_lines);
    }
    merged.detail_lines.truncate(detail_limit);
    merged
}

/// 一格这次挂载：在读故障下做实例 2 的可写挂载、录写流、数崩溃状态，打一行 cell_mount。
fn prepare_a_cell(start: &StartingHistory, cell: FaultCell) -> (MountUnderFaults, Option<(u64, Vec<usize>)>, u64) {
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
    (mount, first_publish, state_count)
}

fn seconds_factor(count: u64) -> f64 {
    f64::from(u32::try_from(count).expect("状态数装得进 u32"))
}

/// 读坏两条根（C、D）连同记录、根与记录读回全零、两块盘最新那一槽系统配置读回全零：前一份模型估时那一格。
const ESTIMATE_CELL: FaultCell =
    FaultCell { roots_hidden: 2, root_fault: ReadFault::AllZero, system_configuration_fault: Some(ReadFault::AllZero) };
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


/// 全量那一条还能判多少个状态：两条腿合起来的预算，减去前一条腿已判的、估时、对拍（单线程与多线程各一遍）、最小历史那几条判的。
fn full_run_budget() -> u64 {
    STATES_BUDGET_OF_BOTH_LEGS
        - STATES_SPENT_BY_THE_EARLIER_LEG
        - ESTIMATE_SAMPLE_STATES
        - 2 * CROSS_CHECK_SAMPLE_STATES
        - MINIMAL_HISTORY_CELLS
}

/// 报告第一节「形乙：N-配置 被系统配置那一槽的「全零 / 自证不过」读坏绕过」那段最小历史的一格。第 3 步那次可写挂载在读故障下跑，
/// 第 4 步写行那一次发布（txg 5）完整持久，崩在第 5 步实例 2 暖机写 txg 6 的根槽之前：录制流里那一笔根槽写之前的写全部持久，
/// 它与它之后的都没持久（一个崩溃点，不枚举）。读故障消失之后读全部根槽，基线与三条变体各择哪条、那一版的第一个文件读出来是哪一版、
/// 已确认的 C 丢没丢（读出来既不是 C 也不是叠在 C 上的 D），池级 checker 红哪几条，再做一次没有读故障的可写挂载、读它写行那一版的文件。
fn minimal_history_line(start: &StartingHistory, cell: FaultCell) -> String {
    let label = format!(
        "root_fault={} newest_system_configuration_fault={}",
        cell.root_fault.name(),
        cell.system_configuration_fault.map_or("none", ReadFault::name)
    );
    let mount = mount_under_faults(&start.image, fault_ranges_of(start, cell));
    // root_identity_of_write 只认根槽 FUA 写（别的种类它 assert 拒），先判种类。
    let Some(crash_write) = mount.writes.iter().position(|write| {
        write.kind == StepKind::RootRecordFua && {
            let (instance, txg) = root_identity_of_write(write);
            instance.0 == 2 && txg.0 == TXG_OF_D
        }
    }) else {
        return format!(
            "OPUSMODEL name=minimal_history {label} h2_mount={} crash_point=absent_no_root_write_of_instance_2_txg_{TXG_OF_D} writes={}",
            mount.outcome,
            mount.writes.len()
        );
    };
    let crash_stream_index = mount.stream_indexes[crash_write];
    let persisted: Vec<bool> = mount.stream_indexes.iter().map(|index| *index < crash_stream_index).collect();
    let persisted_count = persisted.iter().filter(|is_persisted| **is_persisted).count();
    let image = image_of_the_state(&start.image, &mount.writes, &persisted);
    let crash_point = format!(
        "crash_point=before_root_of_instance_2_txg_{TXG_OF_D} persisted_writes={persisted_count}/{}",
        mount.writes.len()
    );
    let configuration = match choose_system_configuration(&image) {
        Ok(configuration) => configuration,
        Err(failure) => {
            return format!(
                "OPUSMODEL name=minimal_history {label} h2_mount={} {crash_point} system_configuration_unchosen:{}",
                mount.outcome,
                short_name_of_debug(&format!("{failure:?}"))
            );
        }
    };
    let roots = ring_roots(&image, &configuration);
    let keys = keys_of(&roots);
    let (form_a, form_b) = forms_in(&keys);
    let root_of = |key: (u32, u64)| {
        roots
            .iter()
            .flatten()
            .find(|root| (root.instance.0, root.checkpoint_txg.0) == key)
            .copied()
            .expect("择出来的键在根环里")
    };
    let orders = std::iter::once((RootChoiceOrder::Baseline, "baseline")).chain(VARIANT_ORDERS);
    let per_order: Vec<String> = orders
        .map(|(order, name)| match choose_under(&keys, order) {
            None => format!("{name}=none {name}_content=none {name}_c_lost=true"),
            Some(key) => {
                let content = content_under(&image, &root_of(key));
                format!(
                    "{name}={} {name}_content={content} {name}_c_lost={}",
                    key_text(Some(key)),
                    confirmed_c_lost(&content)
                )
            }
        })
        .collect();
    let core = choose_root(&image, &configuration).map(|root| (root.instance.0, root.checkpoint_txg.0));
    let violated = violated_invariants(&image);
    let mut devices = sparse_devices_of(&image);
    let (mount_after, mount_after_content) = match mount_writable(&pool_parameters(), &mut devices) {
        Ok(mounted) => {
            let row_root = *mounted.output.row_publish.root();
            let after = memory_pool_of_devices(&devices);
            (
                format!(
                    "ok:chosen=({},{}),effective=({},{}),row=({},{})",
                    mounted.output.chosen_root.instance.0,
                    mounted.output.chosen_root.checkpoint_txg.0,
                    mounted.output.effective_root.instance.0,
                    mounted.output.effective_root.checkpoint_txg.0,
                    row_root.instance.0,
                    row_root.checkpoint_txg.0
                ),
                content_under(&after, &row_root),
            )
        }
        Err(error) => (format!("refused:{}", short_name_of_debug(&format!("{error:?}"))), "not_mounted".to_string()),
    };
    format!(
        "OPUSMODEL name=minimal_history {label} h2_mount={} {crash_point} roots=[{}] form_a={form_a} form_b={form_b} {} core_choose_root={} checker_red=[{}] mount_after={mount_after} mount_after_content={mount_after_content} mount_after_c_lost={}",
        mount.outcome,
        keys.iter().flatten().map(|key| key_text(Some(*key))).collect::<Vec<_>>().join(","),
        per_order.join(" "),
        key_text(core),
        violated.join(","),
        confirmed_c_lost(&mount_after_content)
    )
}

/// 最小历史：根槽三种读坏 × 两块盘最新那一槽系统配置两种读坏（读回全零、读得出而自证不过），每格一个固定崩溃点、一行。
#[test]
fn e165_attack_minimal_history_fixed_crash_point() {
    let began = Instant::now();
    let start = build_starting_history();
    let mut cells = 0u64;
    for root_fault in [ReadFault::DeviceError, ReadFault::NotSelfVerified, ReadFault::AllZero] {
        for system_configuration_fault in [ReadFault::AllZero, ReadFault::NotSelfVerified] {
            let cell = FaultCell { roots_hidden: 2, root_fault, system_configuration_fault: Some(system_configuration_fault) };
            println!("{}", minimal_history_line(&start, cell));
            cells += 1;
        }
    }
    assert_eq!(cells, MINIMAL_HISTORY_CELLS, "根槽三种 × 系统配置槽两种");
    println!("OPUSMODEL name=minimal_history_done cells={cells} seconds={:.1}", began.elapsed().as_secs_f64());
}

/// 估时：对照；二十五格各做一次那次挂载、数状态；估时那一格按等距抽至多 2 万个崩溃状态，用登记的线程数切片判，
/// 按抽样的挂钟乘全量（预算之内）的状态数推全量挂钟。
#[test]
fn e165_attack_estimate() {
    let began = Instant::now();
    let threads = registered_threads();
    let start = build_starting_history();
    println!(
        "OPUSMODEL name=estimate_start threads={threads} starting_history_seconds={:.1}",
        began.elapsed().as_secs_f64()
    );
    assert!(run_the_controls(&start), "对照没红：模型判形乙的那一段坏了");
    let mut total_states = 0u64;
    let mut cells_over_the_one_history_limit = 0u64;
    let mut estimate_cell_prepared = None;
    for cell in every_cell() {
        let prepared = prepare_a_cell(&start, cell);
        if prepared.2 > STATES_OF_ONE_HISTORY_LIMIT {
            cells_over_the_one_history_limit += 1;
        } else {
            total_states += prepared.2;
        }
        if cell.label() == ESTIMATE_CELL.label() {
            estimate_cell_prepared = Some(prepared);
        }
    }
    let (mount, first_publish, states) = estimate_cell_prepared.expect("估时那一格在二十五格里");
    let job = CellJob {
        label: format!("{} sample=evenly_spaced", ESTIMATE_CELL.label()),
        base: &start.image,
        mount: &mount,
        first_publish,
        selection: StateSelection::evenly_spaced(states, ESTIMATE_SAMPLE_STATES),
    };
    let sample_began = Instant::now();
    let result = judge_the_selection(&job, threads, false, DETAIL_LINES_PER_CELL);
    let sample_seconds = sample_began.elapsed().as_secs_f64();
    for line in &result.detail_lines {
        println!("{line}");
    }
    println!("{} seconds={sample_seconds:.1} threads={threads}", result.tally.text(&job.label, job.selection));
    let full_states = total_states.min(full_run_budget());
    let projected_full_seconds = sample_seconds / seconds_factor(job.selection.count) * seconds_factor(full_states);
    println!(
        "OPUSMODEL name=estimate cells={} total_states={total_states} cells_over_one_history_limit={cells_over_the_one_history_limit} full_budget={} full_states={full_states} threads={threads} sample_states={} sample_seconds={sample_seconds:.1} projected_full_seconds={projected_full_seconds:.0} estimate_seconds={:.1}",
        every_cell().len(),
        full_run_budget(),
        job.selection.count,
        began.elapsed().as_secs_f64()
    );
}

/// 多线程与单线程的对拍：估时那一格等距抽 600 个崩溃状态，单线程（一片）与登记的线程数切片各判一遍，
/// 逐态一行、带形的那几行、合并后的计数逐字相同。
#[test]
fn e165_attack_sliced_matches_single_thread() {
    let threads = registered_threads().max(2);
    let start = build_starting_history();
    let (mount, first_publish, states) = prepare_a_cell(&start, ESTIMATE_CELL);
    let job = CellJob {
        label: format!("{} sample=cross_check", ESTIMATE_CELL.label()),
        base: &start.image,
        mount: &mount,
        first_publish,
        selection: StateSelection::evenly_spaced(states, CROSS_CHECK_SAMPLE_STATES),
    };
    let single_began = Instant::now();
    let single = judge_the_selection(&job, 1, true, DETAIL_LINES_PER_CELL);
    let single_seconds = single_began.elapsed().as_secs_f64();
    let sliced_began = Instant::now();
    let sliced = judge_the_selection(&job, threads, true, DETAIL_LINES_PER_CELL);
    let sliced_seconds = sliced_began.elapsed().as_secs_f64();
    let lines_of = |result: &SliceResult| {
        let mut lines = result.every_state_lines.clone();
        lines.extend(result.detail_lines.iter().cloned());
        lines.push(result.tally.text(&job.label, job.selection));
        lines
    };
    let single_lines = lines_of(&single);
    let sliced_lines = lines_of(&sliced);
    let first_difference = single_lines.iter().zip(&sliced_lines).position(|(left, right)| left != right);
    let identical = single_lines == sliced_lines;
    println!("{}", single.tally.text(&job.label, job.selection));
    println!(
        "OPUSMODEL name=cross_check threads_sliced={threads} selected_states={} state_lines_single={} state_lines_sliced={} single_seconds={single_seconds:.1} sliced_seconds={sliced_seconds:.1} identical={identical} first_difference={}",
        job.selection.count,
        single.every_state_lines.len(),
        sliced.every_state_lines.len(),
        first_difference.map_or_else(|| "none".to_string(), |line| line.to_string())
    );
    if let Some(line) = first_difference {
        println!("OPUSMODEL name=cross_check_single {}", single_lines[line]);
        println!("OPUSMODEL name=cross_check_sliced {}", sliced_lines[line]);
    }
    assert_eq!(
        single.every_state_lines.len(),
        usize::try_from(job.selection.count).expect("态数"),
        "单线程逐态一行"
    );
    assert!(identical, "切片多线程与单线程的读数不逐字相同");
}

/// 全量：对照与二十五格，每一格这次挂载的崩溃状态一个不落（一段历史超过 10⁶ 个、或越过两条腿合起来约 10⁷ 的预算的格不跑、打一行），
/// 用登记的线程数切片判，每 5 分钟每片报一行进度。
#[test]
#[ignore = "全量：日常验证的状态数压在 20 万以内，这一条照派发提示留 #[ignore]；先跑 e165_attack_estimate 看 projected_full_seconds"]
fn e165_attack_full() {
    let began = Instant::now();
    let threads = registered_threads();
    let start = build_starting_history();
    assert!(run_the_controls(&start), "对照没红：模型判形乙的那一段坏了");
    let mut budget_left = full_run_budget();
    let mut cells_with_form_b = 0u64;
    let mut cells_skipped = 0u64;
    let mut judged_states = 0u64;
    for cell in every_cell() {
        let (mount, first_publish, states) = prepare_a_cell(&start, cell);
        if states > STATES_OF_ONE_HISTORY_LIMIT {
            println!(
                "OPUSMODEL name=cell_skipped {} states={states} reason=one_history_over_limit limit={STATES_OF_ONE_HISTORY_LIMIT}",
                cell.label()
            );
            cells_skipped += 1;
            continue;
        }
        if states > budget_left {
            println!(
                "OPUSMODEL name=cell_skipped {} states={states} reason=over_the_budget_of_both_legs budget_left={budget_left}",
                cell.label()
            );
            cells_skipped += 1;
            continue;
        }
        budget_left -= states;
        let job = CellJob {
            label: cell.label(),
            base: &start.image,
            mount: &mount,
            first_publish,
            selection: StateSelection::every_state(states),
        };
        let cell_began = Instant::now();
        let result = judge_the_selection(&job, threads, false, DETAIL_LINES_PER_CELL);
        for line in &result.detail_lines {
            println!("{line}");
        }
        println!(
            "{} seconds={:.1} threads={threads}",
            result.tally.text(&job.label, job.selection),
            cell_began.elapsed().as_secs_f64()
        );
        cells_with_form_b += u64::from(result.tally.form_b > 0);
        judged_states += result.tally.states;
    }
    println!(
        "OPUSMODEL name=done cells={} cells_skipped={cells_skipped} cells_with_form_b={cells_with_form_b} judged_states={judged_states} budget_left={budget_left} threads={threads} seconds={:.1}",
        every_cell().len(),
        began.elapsed().as_secs_f64()
    );
}
