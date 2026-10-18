//! 攻方原型（m3-b44-code-r3 攻方腿，只在冻结副本的拷贝上跑）：可写挂载这次挂载里预演与真做共用一份读（W13、W15）、
//! 读坏当场重读穿到盘上（W16）、预演读错在取号之前拒（W13 的读法 ⓑ）在带读故障的盘上。
//! 镜像照 W13 用例 `writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs` 的窄盘（第二轮攻方原型的 `narrow_image` 原样）：
//! 没改坏的那一份续做 16 步、中途推抬 F，取号之前那一趟在只进内存的盘上走；改坏的那一份退役树唯一那片号是 u64::MAX，续做最后一批报 C841。
//! 盘是这里自己写的一层（`ProbeDisk`），垫在挂载交进去的那几块盘的位置，也就是 `DeviceReadOnceInThisMount` 底下：它数到的读都是穿到盘上的读。
//! 故障两种：读报错（`fail`），读报成功而第 0 字节翻掉最低位（`garbage`，读得出而自证不过）。点法两种：整池第 n 次起连着 w 次读
//! （`window`），或某块盘某个偏移起的读的头 k 次（`offset`，一块一阵子读坏、过后读得好的盘面）。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use common::PoolImageGeometry;
use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest,
};
use common_deletion_in_progress::{
    batches_of_one_data_unit, first_container_slot_of_the_tree_of_kind_of_the_newest_root,
    root_slot_of_the_tree_of_kind_of_the_newest_root,
};
use common_pointer_head_corruption::{
    change_the_unit_and_propagate_its_checksum_on, data_unit_bytes, node_bytes,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_session::UserChange;
use singlefs_core::records::TREE_KIND_RETIREMENT;
use singlefs_core::retirement_tree::{
    RetirementContainerCapacity, RetirementRecordsPerContainerCappedForTests,
};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, PoolWriter, PublishError, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::DeletionSwitches;
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;
const CONTAINER_TREE_ROOT_SMALLEST_KEY_OFFSET: usize = 52;
const CONTAINER_TREE_ROOT_LARGEST_KEY_OFFSET: usize = 60;
const CONTAINER_TREE_ROOT_ENTRIES_START: usize = 131;
const CONTAINER_TREE_ENTRY_SEPARATOR_KEY_OFFSET: usize = 0;
const CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_NUMBER_OFFSET: usize = 18;
const UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES: u64 = 304;
const DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES: usize = 16;

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn set_u64_at(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn switches() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: batches_of_one_data_unit(),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity: RetirementContainerCapacity::CappedForTests(
            RetirementRecordsPerContainerCappedForTests::new(1).expect("1..=959"),
        ),
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 退役树唯一那片容器的号与根里指它的那几处都改成 u64::MAX：续做最后一批要开新容器时报 C841。
fn move_the_only_retirement_container_to_the_top_of_its_range(
    image: &mut MemoryPool,
    geometry: &PoolImageGeometry,
    number_before: u64,
) {
    let container_slot =
        first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_RETIREMENT);
    change_the_unit_and_propagate_its_checksum_on(
        image,
        geometry,
        container_slot,
        data_unit_bytes(),
        |container| {
            assert_eq!(u64_at(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET), number_before);
            set_u64_at(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET, u64::MAX);
        },
    );
    let root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_RETIREMENT);
    change_the_unit_and_propagate_its_checksum_on(image, geometry, root_slot, node_bytes(), |root| {
        let entry = CONTAINER_TREE_ROOT_ENTRIES_START;
        for offset in [
            CONTAINER_TREE_ROOT_SMALLEST_KEY_OFFSET,
            CONTAINER_TREE_ROOT_LARGEST_KEY_OFFSET,
            entry + CONTAINER_TREE_ENTRY_SEPARATOR_KEY_OFFSET,
            entry + CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_NUMBER_OFFSET,
        ] {
            assert_eq!(u64_at(root, offset), number_before);
            set_u64_at(root, offset, u64::MAX);
        }
    });
}

/// 窄盘镜像：镜像、几何、退役树唯一那片的号。
fn narrow_image() -> (MemoryPool, PoolImageGeometry, u64) {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES,
        );
    pool.sequential_write(
        (DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES - 1) * data_unit_payload_capacity() + 100,
    )
    .expect("顺序写长到 16 个单元");
    let version_with_the_grown_file = pool.current_file_version().clone();
    pool.write_time_seconds += 1;
    let write_time_seconds_of_the_unlink = pool.write_time_seconds;
    pool.publish(
        UserChange::Unlink {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            write_time_seconds: write_time_seconds_of_the_unlink,
        },
        None,
    )
    .expect("unlink 到底");
    let geometry = PoolImageGeometry {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
    };
    let PoolUnderTest {
        session,
        devices,
        parameters,
        write_time_seconds,
        ..
    } = &mut pool;
    let session = session.as_mut().expect("可写会话");
    let mut current = session.current.version_of_trees().expect("有树").clone();
    roll_back_by_a_forward_publish(
        parameters,
        devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: version_with_the_grown_file.root.instance,
            checkpoint_txg: version_with_the_grown_file.root.checkpoint_txg,
        },
    )
    .expect("回退到 16 个单元那一版");
    let intent = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            *write_time_seconds,
            session.instance,
        )
        .expect("意图")
    };
    let number = intent
        .retirement_containers
        .last()
        .expect("退役树一片")
        .contents
        .identity
        .container;
    pool.session = None;
    (pool.image(), geometry, number)
}

/// 按 `PROBE_IMAGE`（`unchanged` 或 `corrupted`）取镜像。
fn image_named(name: &str) -> (MemoryPool, PoolImageGeometry) {
    let (mut image, geometry, number) = narrow_image();
    match name {
        "unchanged" => {}
        "corrupted" => {
            move_the_only_retirement_container_to_the_top_of_its_range(&mut image, &geometry, number)
        }
        other => panic!("认不出的镜像名 {other}"),
    }
    (image, geometry)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultKind {
    Fail,
    Garbage,
}

#[derive(Clone, Copy, Debug)]
enum FaultPlan {
    Nothing,
    /// 整池第 `start` 次读起连着 `width` 次。
    Window { start: u64, width: u64, kind: FaultKind },
    /// `device` 上从 `offset` 起的读，头 `first` 次。
    Offset { device: DeviceIdentity, offset: u64, first: u32, kind: FaultKind },
    /// 两块盘上从 `offset` 起的读，整池第 `after` 次读之后各自的头 `first` 次（一个单元的两份；两版二进制在第 `after` 次读之前读的是同一串）。
    BothCopiesAfter { offset: u64, after: u64, first: u32, kind: FaultKind },
}

#[derive(Clone, Copy, Debug)]
struct ReadOnTheDisk {
    ordinal: u64,
    device: DeviceIdentity,
    offset: u64,
    length: u64,
    writes_before: u64,
}

#[derive(Default)]
struct ProbeState {
    reads: u64,
    writes: u64,
    reads_when_the_first_write_came: Option<u64>,
    fired: Vec<ReadOnTheDisk>,
    reads_of_each_offset: BTreeMap<(DeviceIdentity, u64), u32>,
    reads_of_each_offset_after_the_anchor: BTreeMap<(DeviceIdentity, u64), u32>,
    trace: Vec<ReadOnTheDisk>,
    blocks_read: BTreeSet<(DeviceIdentity, u64)>,
    blocks_written: BTreeSet<(DeviceIdentity, u64)>,
    /// 每一次写：盘、偏移、长度、它之前整池读到盘上几次、写的头 8 字节之后那几字节里的世代号（只对系统配置槽有意义，见下面那条用例）。
    writes_log: Vec<(DeviceIdentity, u64, u64, u64)>,
}

/// 挂载交进去的那几块盘：数读、按计划注错、记第一个写之前读了几次。
struct ProbeDisk {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    plan: FaultPlan,
    state: Rc<RefCell<ProbeState>>,
}

impl ProbeDisk {
    fn block_bytes(&self) -> u64 {
        u64::from(self.inner.probe_physical_block_size().0)
    }
}

impl BlockDevice for ProbeDisk {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let length = buffer.len() as u64;
        let block = self.block_bytes();
        let (fault, read) = {
            let mut state = self.state.borrow_mut();
            state.reads += 1;
            let read = ReadOnTheDisk {
                ordinal: state.reads,
                device: self.identity,
                offset: offset.0,
                length,
                writes_before: state.writes,
            };
            state.trace.push(read);
            let seen = state.reads_of_each_offset.entry((self.identity, offset.0)).or_insert(0);
            *seen += 1;
            let seen = *seen;
            let seen_after = match self.plan {
                FaultPlan::BothCopiesAfter { after, .. } if read.ordinal > after => {
                    let count = state
                        .reads_of_each_offset_after_the_anchor
                        .entry((self.identity, offset.0))
                        .or_insert(0);
                    *count += 1;
                    *count
                }
                _ => 0,
            };
            let fault = match self.plan {
                FaultPlan::Nothing => None,
                FaultPlan::Window { start, width, kind } => {
                    (start <= read.ordinal && read.ordinal < start + width).then_some(kind)
                }
                FaultPlan::Offset { device, offset: at, first, kind } => {
                    (device == self.identity && at == offset.0 && seen <= first).then_some(kind)
                }
                FaultPlan::BothCopiesAfter { offset: at, after, first, kind } => {
                    (at == offset.0 && read.ordinal > after && seen_after <= first).then_some(kind)
                }
            };
            if fault.is_some() {
                state.fired.push(read);
            }
            (fault, read)
        };
        if fault == Some(FaultKind::Fail) {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)?;
        {
            let mut state = self.state.borrow_mut();
            for index in 0..length / block {
                state.blocks_read.insert((self.identity, offset.0 / block + index));
            }
        }
        if fault == Some(FaultKind::Garbage) {
            buffer[0] ^= 1;
        }
        let _ = read;
        Ok(())
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.note_a_write(offset.0, bytes.len() as u64);
        {
            let mut state = self.state.borrow_mut();
            let reads = state.reads;
            state.writes_log.push((self.identity, offset.0, bytes.len() as u64, reads));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.note_a_write(offset.0, length);
        self.inner.write_zeroes_at(offset, length)
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
}

impl ProbeDisk {
    fn note_a_write(&self, offset: u64, length: u64) {
        let block = self.block_bytes();
        let mut state = self.state.borrow_mut();
        if state.reads_when_the_first_write_came.is_none() {
            state.reads_when_the_first_write_came = Some(state.reads);
        }
        state.writes += 1;
        for index in 0..length / block {
            state.blocks_written.insert((self.identity, offset / block + index));
        }
    }
}

fn probe_disks(image: &MemoryPool, plan: FaultPlan) -> (Vec<(DeviceIdentity, ProbeDisk)>, Rc<RefCell<ProbeState>>) {
    let state = Rc::new(RefCell::new(ProbeState::default()));
    let disks = plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| (identity, ProbeDisk { identity, inner, plan, state: state.clone() }))
        .collect();
    (disks, state)
}

fn images_of(disks: &[(DeviceIdentity, ProbeDisk)]) -> Vec<(DeviceIdentity, SparseDevice)> {
    disks.iter().map(|(identity, disk)| (*identity, disk.inner.image.clone())).collect()
}

fn pool_of(disks: &[(DeviceIdentity, ProbeDisk)], device_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: disks.iter().map(|(identity, disk)| (*identity, disk.inner.image.clone())).collect(),
        device_size_in_bytes: device_bytes,
    }
}

fn first_word(text: &str) -> String {
    text.split(|character: char| !character.is_alphanumeric() && character != '_')
        .next()
        .unwrap_or("")
        .to_owned()
}

/// 一次挂载的结局，分到几类。
fn outcome_name(result: &Result<Mounted, MountError>) -> String {
    match result {
        Ok(mounted) => format!(
            "Ok(steps={})",
            mounted.output.deletion_queue_continuation.len()
        ),
        Err(MountError::Publish(failed)) => {
            let persisted = failed.writes_of_persisted_publishes.len();
            let ledgers_empty = persisted == 0 && failed.writes_of_failed_publishes.is_empty();
            match (&failed.cause, ledgers_empty) {
                (PublishError::NextContainerNumberPastTheTopOfItsRange(_), true) => {
                    "C841-before-acquisition".to_owned()
                }
                (PublishError::NextContainerNumberPastTheTopOfItsRange(_), false) => {
                    format!("C841-after-writes(persisted={persisted})")
                }
                // 按名字认（W13 之前那一版没有这一成员，同一份原型两版都编得过）。
                (cause, true)
                    if format!("{cause:?}")
                        .starts_with("ReadFailedWhileRehearsingTheContinuationBeforeAcquisition") =>
                {
                    "ReadFailedRefusalBeforeAcquisition".to_owned()
                }
                (other, _) => format!("Publish({},persisted={persisted})", first_word(&format!("{other:?}"))),
            }
        }
        Err(other) => first_word(&format!("{other:?}")),
    }
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().map_or(default, |text| text.parse().expect("十进制整数"))
}

fn env_text(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn kind_named(name: &str) -> FaultKind {
    match name {
        "fail" => FaultKind::Fail,
        "garbage" => FaultKind::Garbage,
        other => panic!("认不出的故障名 {other}"),
    }
}

struct OneMount {
    outcome: String,
    after: MemoryPool,
    state: Rc<RefCell<ProbeState>>,
}

fn mount_once(image: &MemoryPool, geometry: &PoolImageGeometry, plan: FaultPlan) -> OneMount {
    let (mut disks, state) = probe_disks(image, plan);
    let result = mount_writable_with_test_only_switches_and_deletion_switches(
        &geometry.parameters,
        &mut disks,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(),
    );
    let outcome = outcome_name(&result);
    drop(result);
    OneMount { outcome, after: pool_of(&disks, image.device_size_in_bytes), state }
}

/// 镜像内容的摘要（两块盘的非零扇区：扇区号与字节，`DefaultHasher` 的固定键）：同一份工具链编出的两版二进制拿它比盘面。
fn image_digest(image: &MemoryPool) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for (identity, device) in &image.devices {
        for (sector, bytes) in device.written_sectors() {
            if bytes.iter().any(|byte| *byte != 0) {
                (identity.0, *sector, bytes.as_slice()).hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

/// 两份镜像逐扇区比（没写过的扇区当全 0），把不同的扇区按盘连成段打出来（至多 40 段）。
fn print_sector_ranges_differing(label: &str, left: &MemoryPool, right: &MemoryPool) {
    let zero = vec![0u8; 512];
    let mut ranges: Vec<(u64, u64, u64)> = Vec::new();
    for (identity, left_device) in &left.devices {
        let right_device = &right.devices[identity];
        let sectors: BTreeSet<u64> = left_device
            .written_sectors()
            .keys()
            .chain(right_device.written_sectors().keys())
            .copied()
            .collect();
        for sector in sectors {
            let left_bytes = left_device.written_sectors().get(&sector).unwrap_or(&zero);
            let right_bytes = right_device.written_sectors().get(&sector).unwrap_or(&zero);
            if left_bytes != right_bytes {
                match ranges.last_mut() {
                    Some((device, _, end)) if *device == u64::from(identity.0) && *end == sector => *end = sector + 1,
                    _ => ranges.push((u64::from(identity.0), sector, sector + 1)),
                }
            }
        }
    }
    println!("{label} differing_ranges={}", ranges.len());
    for (device, start, end) in ranges.iter().take(40) {
        println!("{label} differ dev={device} bytes={}..{}", start * 512, end * 512);
    }
}

/// 盘与改之前、与不带故障那一次比。
fn disk_class(after: &MemoryPool, before: &MemoryPool, fault_free_after: &MemoryPool) -> &'static str {
    if after.devices == before.devices {
        "disk-unchanged"
    } else if after.devices == fault_free_after.devices {
        "disk-as-fault-free"
    } else {
        "disk-other"
    }
}

/// 注进去的故障是不是全在这次挂载第一个写之前。
fn phase_of_the_faults(state: &ProbeState) -> &'static str {
    match (state.fired.is_empty(), state.reads_when_the_first_write_came) {
        (true, _) => "none-fired",
        (false, None) => "all-before-the-first-write(no-write)",
        (false, Some(reads)) if state.fired.iter().all(|read| read.ordinal <= reads) => {
            "all-before-the-first-write"
        }
        (false, Some(reads)) if state.fired.iter().all(|read| read.ordinal > reads) => {
            "all-after-the-first-write"
        }
        _ => "both-sides-of-the-first-write",
    }
}

/// 不带故障挂一次：结局、读到盘上几次、第一个写之前几次、读过与写过的块数（共用的那一份最多记这么多块）。
#[test]
fn fault_free_trace() {
    let name = env_text("PROBE_IMAGE", "unchanged");
    let (image, geometry) = image_named(&name);
    let mount = mount_once(&image, &geometry, FaultPlan::Nothing);
    let state = mount.state.borrow();
    let before_first_write = state.reads_when_the_first_write_came.unwrap_or(state.reads);
    let distinct_before: BTreeSet<(DeviceIdentity, u64)> = state
        .trace
        .iter()
        .filter(|read| read.ordinal <= before_first_write)
        .map(|read| (read.device, read.offset))
        .collect();
    let distinct_all: BTreeSet<(DeviceIdentity, u64)> =
        state.trace.iter().map(|read| (read.device, read.offset)).collect();
    let touched: BTreeSet<(DeviceIdentity, u64)> =
        state.blocks_read.union(&state.blocks_written).copied().collect();
    println!(
        "name=fault_free_trace image={name} outcome={} reads={} reads_before_the_first_write={before_first_write} writes={} distinct_offsets_read_before_the_first_write={} distinct_offsets_read={} blocks_read={} blocks_written={} blocks_touched={} disk_changed={} digest_before={:016x} digest_after={:016x}",
        mount.outcome,
        state.reads,
        state.writes,
        distinct_before.len(),
        distinct_all.len(),
        state.blocks_read.len(),
        state.blocks_written.len(),
        touched.len(),
        mount.after.devices != image.devices,
        image_digest(&image),
        image_digest(&mount.after)
    );
    if env_u64("PROBE_PRINT_TRACE", 0) == 1 {
        for read in &state.trace {
            println!("trace {} dev={} off={} len={} writes_before={}", read.ordinal, read.device.0, read.offset, read.length, read.writes_before);
        }
    }
}

fn report_histogram(label: &str, histogram: &BTreeMap<String, u64>, scanned: u64, elapsed: std::time::Duration) {
    println!("{label} scanned={scanned} in {elapsed:?}");
    for (key, count) in histogram {
        println!("{label} summary {count} × {key}");
    }
}

/// 整池第 n 次读起连着 w 次读坏：n 从 `PROBE_FROM` 到 `PROBE_TO`（缺省到不带故障那一次的读总数），w 取 `PROBE_WIDTHS`。
#[test]
fn scan_windows() {
    let name = env_text("PROBE_IMAGE", "unchanged");
    let kind = kind_named(&env_text("PROBE_KIND", "fail"));
    let (image, geometry) = image_named(&name);
    let fault_free = mount_once(&image, &geometry, FaultPlan::Nothing);
    let total_reads = fault_free.state.borrow().reads;
    let from = env_u64("PROBE_FROM", 1);
    let to = env_u64("PROBE_TO", total_reads).min(total_reads);
    let widths: Vec<u64> = env_text("PROBE_WIDTHS", "2")
        .split(',')
        .map(|text| text.parse().expect("窗口宽"))
        .collect();
    let label = format!("window image={name} kind={kind:?}");
    println!("{label} fault-free outcome={} total_reads={total_reads}", fault_free.outcome);
    let mut histogram = BTreeMap::new();
    let started = std::time::Instant::now();
    let mut scanned = 0;
    for width in &widths {
        for start in from..=to {
            let mount = mount_once(&image, &geometry, FaultPlan::Window { start, width: *width, kind });
            let state = mount.state.borrow();
            let key = format!(
                "width={width} {} {} {} fired={}",
                mount.outcome,
                disk_class(&mount.after, &image, &fault_free.after),
                phase_of_the_faults(&state),
                state.fired.len()
            );
            if env_u64("PROBE_VERBOSE", 1) == 1 {
                println!("{label} start={start} {key}");
            }
            *histogram.entry(key).or_insert(0) += 1;
            scanned += 1;
        }
    }
    report_histogram(&label, &histogram, scanned, started.elapsed());
}

/// 一块盘一个偏移起的读，头 k 次读坏（k 取 `PROBE_FIRST`）：偏移取不带故障那一次读到过的每一个（两块盘），按读到的先后。
#[test]
fn scan_offsets() {
    let name = env_text("PROBE_IMAGE", "unchanged");
    let kind = kind_named(&env_text("PROBE_KIND", "fail"));
    let first = u32::try_from(env_u64("PROBE_FIRST", 2)).expect("小");
    let (image, geometry) = image_named(&name);
    let fault_free = mount_once(&image, &geometry, FaultPlan::Nothing);
    let mut offsets: Vec<(DeviceIdentity, u64)> = Vec::new();
    {
        let state = fault_free.state.borrow();
        let mut seen = BTreeSet::new();
        for read in &state.trace {
            if seen.insert((read.device, read.offset)) {
                offsets.push((read.device, read.offset));
            }
        }
    }
    let from = usize::try_from(env_u64("PROBE_FROM", 0)).expect("小");
    let to = usize::try_from(env_u64("PROBE_TO", offsets.len() as u64)).expect("小").min(offsets.len());
    let label = format!("offset image={name} kind={kind:?} first={first}");
    println!("{label} fault-free outcome={} distinct_offsets={}", fault_free.outcome, offsets.len());
    let mut histogram = BTreeMap::new();
    let started = std::time::Instant::now();
    let mut scanned = 0;
    for (device, offset) in &offsets[from..to] {
        let mount = mount_once(
            &image,
            &geometry,
            FaultPlan::Offset { device: *device, offset: *offset, first, kind },
        );
        let state = mount.state.borrow();
        let key = format!(
            "{} {} {} fired={}",
            mount.outcome,
            disk_class(&mount.after, &image, &fault_free.after),
            phase_of_the_faults(&state),
            state.fired.len()
        );
        if env_u64("PROBE_VERBOSE", 1) == 1 {
            let ordinals: Vec<u64> = state.fired.iter().map(|read| read.ordinal).collect();
            let lengths: Vec<u64> = state.fired.iter().map(|read| read.length).collect();
            println!(
                "{label} dev={} off={offset} fired_at={ordinals:?} lengths={lengths:?} first_write_at={:?} {key}",
                device.0, state.reads_when_the_first_write_came
            );
        }
        *histogram.entry(key).or_insert(0) += 1;
        scanned += 1;
    }
    report_histogram(&label, &histogram, scanned, started.elapsed());
}

/// 一个计划：`w:<起点>:<宽>:<fail|garbage>` 或 `o:<盘>:<偏移>:<头几次>:<fail|garbage>`。
fn plan_named(text: &str) -> FaultPlan {
    let parts: Vec<&str> = text.split(':').collect();
    match parts.as_slice() {
        ["w", start, width, kind] => FaultPlan::Window {
            start: start.parse().expect("起点"),
            width: width.parse().expect("宽"),
            kind: kind_named(kind),
        },
        ["o", device, offset, first, kind] => FaultPlan::Offset {
            device: DeviceIdentity(device.parse().expect("盘")),
            offset: offset.parse().expect("偏移"),
            first: first.parse().expect("头几次"),
            kind: kind_named(kind),
        },
        ["p", offset, after, first, kind] => FaultPlan::BothCopiesAfter {
            offset: offset.parse().expect("偏移"),
            after: after.parse().expect("锚"),
            first: first.parse().expect("头几次"),
            kind: kind_named(kind),
        },
        _ => panic!("认不出的计划 {text}"),
    }
}

/// 点名的几个计划各挂一次，之后不带故障再挂两次：结局、盘变没变、与不带故障那一次比。
#[test]
fn follow_up() {
    let name = env_text("PROBE_IMAGE", "corrupted");
    let (image, geometry) = image_named(&name);
    let fault_free = mount_once(&image, &geometry, FaultPlan::Nothing);
    println!(
        "follow image={name} fault-free outcome={} digest_after={:016x}",
        fault_free.outcome,
        image_digest(&fault_free.after)
    );
    for text in env_text("PROBE_PLANS", "").split(',').filter(|text| !text.is_empty()) {
        let plan = plan_named(text);
        let first = mount_once(&image, &geometry, plan);
        let state = first.state.borrow();
        println!(
            "follow image={name} plan={text} first mount {} {} {} digest_after={:016x} fired_at={:?} first_write_at={:?}",
            first.outcome,
            disk_class(&first.after, &image, &fault_free.after),
            phase_of_the_faults(&state),
            image_digest(&first.after),
            state.fired.iter().map(|read| (read.ordinal, read.device.0, read.offset, read.length)).collect::<Vec<_>>(),
            state.reads_when_the_first_write_came
        );
        if env_u64("PROBE_SECTOR_DIFF", 0) == 1 {
            print_sector_ranges_differing(&format!("follow image={name} plan={text}"), &fault_free.after, &first.after);
        }
        let mut previous = first.after.clone();
        for remount in 1..=env_u64("PROBE_REMOUNTS", 2) {
            let next = mount_once(&previous, &geometry, FaultPlan::Nothing);
            println!(
                "follow image={name} plan={text} remount {remount} without fault {} disk_changed_by_it={}",
                next.outcome,
                next.after.devices != previous.devices
            );
            previous = next.after.clone();
        }
    }
}

/// 不带故障那一次第 `PROBE_AFTER` 次读之后读到过的每一个偏移（两块盘并起来，按先后），各造一个计划：两块盘上这个偏移第 `PROBE_AFTER` 次读之后
/// 各自的头 `PROBE_FIRST` 次读坏。两版二进制在第 `PROBE_AFTER` 次读之前读的是同一串（不带故障的读迹逐行比过），同一个计划落在同一个逻辑读上。
/// 偏移表取 `PROBE_OFFSETS_FROM` 那一版的读迹文件（每行 `trace  dev=… off=… len=…`），两版用同一张表。
#[test]
fn scan_both_copies_after_the_anchor() {
    let name = env_text("PROBE_IMAGE", "unchanged");
    let kind = kind_named(&env_text("PROBE_KIND", "garbage"));
    let after = env_u64("PROBE_AFTER", 306);
    let first = u32::try_from(env_u64("PROBE_FIRST", 1)).expect("小");
    let (image, geometry) = image_named(&name);
    let fault_free = mount_once(&image, &geometry, FaultPlan::Nothing);
    let trace_text = std::fs::read_to_string(env_text("PROBE_OFFSETS_FROM", "")).expect("读迹文件");
    let mut offsets: Vec<u64> = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, line) in trace_text.lines().enumerate() {
        if (index as u64) < after {
            continue;
        }
        let offset: u64 = line
            .split_whitespace()
            .find_map(|field| field.strip_prefix("off="))
            .expect("off= 一栏")
            .parse()
            .expect("偏移");
        if seen.insert(offset) {
            offsets.push(offset);
        }
    }
    let label = format!("pair image={name} kind={kind:?} after={after} first={first}");
    println!("{label} fault-free outcome={} offsets={}", fault_free.outcome, offsets.len());
    let mut histogram = BTreeMap::new();
    let started = std::time::Instant::now();
    for offset in &offsets {
        let mount = mount_once(
            &image,
            &geometry,
            FaultPlan::BothCopiesAfter { offset: *offset, after, first, kind },
        );
        let state = mount.state.borrow();
        let key = format!(
            "{} {} {} fired={}",
            mount.outcome,
            disk_class(&mount.after, &image, &fault_free.after),
            phase_of_the_faults(&state),
            state.fired.len()
        );
        println!("{label} off={offset} fired_at={:?} {key}", state.fired.iter().map(|read| (read.ordinal, read.device.0)).collect::<Vec<_>>());
        *histogram.entry(key).or_insert(0) += 1;
    }
    report_histogram(&label, &histogram, offsets.len() as u64, started.elapsed());
}

/// 每块盘两槽里自证过的系统配置：世代号（槽 = 世代号 mod 2）、实例代号、tail、F。
fn print_system_configurations(label: &str, image: &MemoryPool, geometry: &PoolImageGeometry) {
    let spacing = u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing);
    for identity in image.devices.keys() {
        let slots = singlefs_core::recovery::verified_system_configuration_slots(
            image,
            *identity,
            spacing,
            &geometry.parameters.filesystem_identifier,
        );
        let rendered: Vec<String> = slots
            .iter()
            .map(|configuration| {
                let quantities = &configuration.quantities;
                format!(
                    "slot{}:generation={},instance={},tail={},floor={}",
                    quantities.slot_generation % 2,
                    quantities.slot_generation,
                    quantities.journal_instance.0,
                    quantities.journal_tail,
                    quantities.rollback_floor.0
                )
            })
            .collect();
        println!("{label} system-configuration dev={} {}", identity.0, rendered.join(" "));
    }
}

/// 点名的计划挂一次，打出改之前、不带故障那一次之后、这一次之后每块盘的系统配置，再不带故障挂 `PROBE_REMOUNTS` 次、每次之后再打一遍。
#[test]
fn system_configurations_after_the_plans() {
    let name = env_text("PROBE_IMAGE", "unchanged");
    let (image, geometry) = image_named(&name);
    print_system_configurations(&format!("sysconf image={name} before"), &image, &geometry);
    let fault_free = mount_once(&image, &geometry, FaultPlan::Nothing);
    print_system_configurations(&format!("sysconf image={name} fault-free-after"), &fault_free.after, &geometry);
    for text in env_text("PROBE_PLANS", "").split(',').filter(|text| !text.is_empty()) {
        let first = mount_once(&image, &geometry, plan_named(text));
        println!("sysconf image={name} plan={text} outcome={}", first.outcome);
        print_system_configurations(&format!("sysconf image={name} plan={text} after"), &first.after, &geometry);
        let mut previous = first.after.clone();
        for remount in 1..=env_u64("PROBE_REMOUNTS", 1) {
            let next = mount_once(&previous, &geometry, FaultPlan::Nothing);
            println!("sysconf image={name} plan={text} remount {remount} outcome={}", next.outcome);
            print_system_configurations(&format!("sysconf image={name} plan={text} remount {remount} after"), &next.after, &geometry);
            previous = next.after.clone();
        }
    }
}

/// 仓里几份测试池的 fsid 记进单元头的那 8 字节（`unit_filesystem_identifier`，系统配置 fsid 的低 8 字节）：16 字节不同、低 8 字节相同的
/// 两个池，单元头里分不出来。
#[test]
fn unit_header_filesystem_identifiers_of_fixture_pools() {
    let fixtures: [(&str, [u8; 16]); 5] = [
        ("core_review_tree_table_duplicates_and_slot_one_search FILESYSTEM_IDENTIFIER", *b"singlefs-rev-a2b"),
        ("core_review_tree_table_duplicates_and_slot_one_search ANOTHER_FILESYSTEM_IDENTIFIER", *b"singlefs-earlier"),
        ("crash_judge_tables", *b"singlefs-tables!"),
        ("E142 pool", common::E142_FILESYSTEM_IDENTIFIER),
        ("E142 pool, last 8 bytes changed", {
            let mut other = common::E142_FILESYSTEM_IDENTIFIER;
            other[8..].copy_from_slice(b"-another");
            other
        }),
    ];
    for (label, fsid) in &fixtures {
        println!(
            "fsid {label}: full={:02x?} in_unit_headers={:016x}",
            fsid,
            singlefs_core::unit::unit_filesystem_identifier(fsid)
        );
    }
}

/// 系统配置槽那两段（每块盘偏移 0 与一个槽距）上的写，按先后：不带故障那一次与点名计划各打出头 `PROBE_FIRST_WRITES` 个，
/// 带「写在第几槽」（偏移 ÷ 槽距）与写之前那一刻盘上两槽各自的世代号（直接读写之前那份镜像拷贝，不经挂载）。
#[test]
fn system_configuration_writes_in_order() {
    let name = env_text("PROBE_IMAGE", "unchanged");
    let (image, geometry) = image_named(&name);
    let spacing = u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing);
    let shown = usize::try_from(env_u64("PROBE_FIRST_WRITES", 4)).expect("小");
    let mut plans = vec!["none".to_owned()];
    plans.extend(env_text("PROBE_PLANS", "").split(',').filter(|text| !text.is_empty()).map(str::to_owned));
    for text in &plans {
        let plan = if text == "none" { FaultPlan::Nothing } else { plan_named(text) };
        let mount = mount_once(&image, &geometry, plan);
        let state = mount.state.borrow();
        let writes: Vec<String> = state
            .writes_log
            .iter()
            .filter(|(_, offset, _, _)| *offset < 2 * spacing)
            .take(shown)
            .map(|(device, offset, length, reads)| format!("dev={} slot={} len={length} after_reads={reads}", device.0, offset / spacing))
            .collect();
        println!("sysconf-writes image={name} plan={text} outcome={} first_system_configuration_writes=[{}]", mount.outcome, writes.join("; "));
    }
    print_system_configurations(&format!("sysconf-writes image={name} before"), &image, &geometry);
}
