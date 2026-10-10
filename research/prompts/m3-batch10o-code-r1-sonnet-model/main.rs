//! 攻方原型（m3-batch10i-code-r1 云端攻方，从 m3-batch10-code-r3 攻方原型改来，只在冻结副本的拷贝上跑；不是入库用例）。
//! 新子命令 s1grid：造「两遍里只有一遍自证过」的槽（S1 新类），挂上之后按用户动作序列发布，每步核 checker 与丢盘整读。
//! 臂由 OPUS_I1_ARM 选（副本 mount.rs 的开关，mount-arms-i1.patch）：空 = 冻结副本今天；refuse = 第 10i 批之前（S1 槽照知道住着根的槽，抬 F 读坏重读仍坏就拒）；
//! ledger = 零轮改法（抬 F 那一遍读坏的 S1 被抛弃根按根环表记着的落点补隔离）。下面旧子命令留着没删，臂名已不适用。
//! 子命令：grid（按槽并两遍的随机两遍读坏网格）、cells（靶向几格，含故障模型之外的错位读）、l0（选定历史的层 0 崩溃态）、
//! k2young（mkfs 之后第一次可写挂载每个崩溃态上的按槽归盘）。臂由环境变量 OPUS_R3_ARM 选（副本 mount.rs 的开关，mount-arms.patch）：
//! 空 = 冻结副本今天的按槽并；reread_only = R1 之前（整份换成重读）；first_only = 只用第一遍；first_wins = 两遍都自证过取第一遍；
//! zero_over_bad = 读坏与全零并成全零。输出每行以 `OPUS3-` 开头。
#![allow(dead_code, unused_imports, unused_variables, clippy::all)]

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use common_admission::{
    checker_violations_on, plain_devices_on, start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    start_plain, DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread, MountError, Mounted,
    RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
};
use singlefs_core::mounted_read::open_pool_for_read;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, instance_table_of_root, read_root_ring_slot,
    root_is_abandoned_by_the_instance_table, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::TransactionOutput;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

fn arm() -> String {
    std::env::var("OPUS_I1_ARM").unwrap_or_default()
}

fn member(error: &MountError) -> String {
    let text = format!("{error:?}");
    let end = text.find(|c: char| c == '(' || c == '{' || c == ' ').unwrap_or(text.len());
    text[..end].to_string()
}

// ---------------------------------------------------------------- 读故障设备

#[derive(Clone, Debug, PartialEq, Eq)]
enum Fault {
    Error,
    Zeros,
    Garbage,
    /// 故障模型之外：这一次读回另一份字节（错位读、读到旧副本），长度一个槽距。
    Bytes(Vec<u8>),
}

impl Fault {
    fn short(&self) -> &'static str {
        match self {
            Fault::Error => "E",
            Fault::Zeros => "Z",
            Fault::Garbage => "G",
            Fault::Bytes(_) => "B",
        }
    }
}

struct State {
    spacing: u64,
    slots: Vec<(RootRingSlot, DeviceIdentity, u64)>,
    /// (槽, 这个槽第几次读) → 读成什么。
    rules: BTreeMap<(RootRingSlot, u64), Fault>,
    /// 槽 → 从第几次读起每一次都读成什么（坏道从这一刻起一直坏）。
    from_onward: BTreeMap<RootRingSlot, (u64, Fault)>,
    reads: BTreeMap<RootRingSlot, u64>,
    fired: u64,
    /// 这个进程写过的槽：写过之后不再注读故障（OPUS_FAULT_KEEPS_AFTER_WRITE=1 时照注）。
    written: BTreeSet<RootRingSlot>,
}

#[derive(Clone)]
struct Shared(Rc<RefCell<State>>);

impl Shared {
    fn on(image: &MemoryPool) -> Shared {
        let sc = choose_system_configuration(image).expect("系统配置");
        let im = sc.immutable;
        let slots = every_root_ring_slot(&im.sizes)
            .into_iter()
            .map(|s| (s, im.region_devices[usize::try_from(s.region).unwrap()], slot_offset(s, im.sizes.fixed_structure_slot_spacing).0))
            .collect();
        Shared(Rc::new(RefCell::new(State {
            spacing: u64::from(im.sizes.fixed_structure_slot_spacing),
            slots,
            rules: BTreeMap::new(),
            from_onward: BTreeMap::new(),
            reads: BTreeMap::new(),
            fired: 0,
            written: BTreeSet::new(),
        })))
    }
    fn note(&self, device: DeviceIdentity, offset: u64) -> Option<(Fault, u64)> {
        let mut state = self.0.borrow_mut();
        let spacing = state.spacing;
        let (slot, start) = state
            .slots
            .iter()
            .find(|(_, d, s)| *d == device && offset >= *s && offset < *s + spacing)
            .map(|(slot, _, s)| (*slot, *s))?;
        let n = {
            let r = state.reads.entry(slot).or_insert(0);
            *r += 1;
            *r
        };
        if state.written.contains(&slot) && std::env::var_os("OPUS_FAULT_KEEPS_AFTER_WRITE").is_none() {
            return None;
        }
        let fault = match state.rules.get(&(slot, n)) {
            Some(f) => f.clone(),
            None => match state.from_onward.get(&slot) {
                Some((from, f)) if n >= *from => f.clone(),
                _ => return None,
            },
        };
        state.fired += 1;
        Some((fault, start))
    }
    fn reads(&self) -> BTreeMap<RootRingSlot, u64> {
        self.0.borrow().reads.clone()
    }
}

struct FaultDevice {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    shared: Shared,
}

impl BlockDevice for FaultDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let Some((fault, start)) = self.shared.note(self.identity, offset.0) else {
            return self.inner.read_at(offset, buffer);
        };
        if fault == Fault::Error {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("攻方点名的这一次读坏")));
        }
        self.inner.read_at(offset, buffer)?;
        let spacing = self.shared.0.borrow().spacing;
        let lo = usize::try_from(start.saturating_sub(offset.0)).unwrap();
        let hi = usize::try_from((start + spacing - offset.0).min(buffer.len() as u64)).unwrap();
        match fault {
            Fault::Zeros => buffer[lo..hi].fill(0),
            Fault::Garbage => buffer[lo..hi].fill(0xA5),
            Fault::Bytes(bytes) => {
                let base = usize::try_from(offset.0 + lo as u64 - start).unwrap();
                buffer[lo..hi].copy_from_slice(&bytes[base..base + (hi - lo)]);
            }
            Fault::Error => unreachable!(),
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], d: WriteDurability) -> Result<(), BlockDeviceError> {
        {
            let mut state = self.shared.0.borrow_mut();
            let spacing = state.spacing;
            let hit: Vec<RootRingSlot> = state
                .slots
                .iter()
                .filter(|(_, dev, start)| *dev == self.identity && offset.0 < *start + spacing && offset.0 + bytes.len() as u64 > *start)
                .map(|(slot, _, _)| *slot)
                .collect();
            state.written.extend(hit);
        }
        self.inner.write_at(offset, bytes, d)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
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
}

impl DeviceOverASparseImage for FaultDevice {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

impl DeviceOverASparseImage for RecordingBlockDevice<FaultDevice> {
    fn sparse_image(&self) -> &SparseDevice {
        &self.wrapped_device().inner.image
    }
}

fn fault_devices_on(image: &MemoryPool, shared: &Shared) -> Vec<(DeviceIdentity, FaultDevice)> {
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| (identity, FaultDevice { identity, inner, shared: shared.clone() }))
        .collect()
}

// ---------------------------------------------------------------- 起点

use singlefs_core::address::InstanceGeneration;

#[derive(Clone)]
struct StartPoint {
    name: String,
    width: HistoryDeviceWidth,
    parameters: MakeFilesystemParameters,
    device_bytes: u64,
    image: MemoryPool,
    narrow: bool,
    write_time_seconds: u64,
    contents: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    /// 被抛弃的那条根 C（有的起点才有）。
    abandoned: Option<(u32, u64)>,
}

fn read_bytes<D: DeviceOverASparseImage>(pool: &PoolUnderTest<D>, device: DeviceIdentity, offset: DeviceOffsetInBytes, len: usize) -> Vec<u8> {
    let index = pool.devices.iter().position(|(i, _)| *i == device).unwrap();
    let mut bytes = vec![0u8; len];
    pool.devices[index].1.read_at(offset, &mut bytes).unwrap();
    bytes
}

fn write_bytes<D: DeviceOverASparseImage>(pool: &mut PoolUnderTest<D>, device: DeviceIdentity, offset: DeviceOffsetInBytes, bytes: &[u8]) {
    let index = pool.devices.iter().position(|(i, _)| *i == device).unwrap();
    pool.devices[index].1.write_at(offset, bytes, WriteDurability::Plain).unwrap();
}

fn finish(name: &str, width: HistoryDeviceWidth, mut pool: PoolUnderTest<SparseBlockDevice>, narrow: bool, abandoned: Option<(u32, u64)>) -> StartPoint {
    pool.session = None;
    StartPoint {
        name: name.to_string(),
        width,
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        image: pool.image(),
        narrow,
        write_time_seconds: pool.write_time_seconds,
        contents: pool.content_of_each_root.clone(),
        abandoned,
    }
}

/// 照第二轮攻方第二次抽样的起点（`m3-batch10-code-r2-opus-s2-model/k2_attack_reread_replaces_the_first_pass.rs` 的
/// `pool_with_an_abandoned_root`，在这里重写一份）：实例 1 第一个文件、覆盖写 `overwrites_before` 次；崩了再挂（实例 2）覆盖写一次得 C；
/// 藏 C 的根槽与数据单元、系统配置轮换停在 C 之前，可写挂载落到 C 之前那一条（实例 3 写行与暖机），再把藏的字节写回，进程退出。
/// `after` 是实例 3 之后再覆盖写几次（C 仍在环里）。
fn abandoned_start(width: HistoryDeviceWidth, overwrites_before: usize, after: usize) -> StartPoint {
    let mut pool = start_plain(width);
    for _ in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写");
    }
    pool.crash_and_mount_writable().expect("取号 2");
    pool.overwrite(2500).expect("C");
    let c = pool.current_file_version().clone();
    let params = pool.parameters.clone();
    let target = target_for_publish(c.root.checkpoint_txg, params.geometry.root_ring_slots_per_region);
    let root_device = params.region_devices[usize::try_from(target.region).unwrap()];
    let root_offset = slot_offset(target, params.geometry.fixed_structure_slot_spacing);
    let root_len = usize::try_from(params.geometry.physical_block_size).unwrap();
    let unit_len = usize::try_from(singlefs_format::DATA_UNIT_BYTES).unwrap();
    let mut places: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> = vec![(root_device, root_offset, root_len)];
    for pointer in &c.data_pointers {
        for location in pointer.locations {
            places.push((location.device, location.slot.to_device_offset(), unit_len));
        }
    }
    let mut saved = Vec::new();
    for (device, offset, len) in &places {
        let bytes = read_bytes(&pool, *device, *offset, *len);
        write_bytes(&mut pool, *device, *offset, &vec![0u8; *len]);
        saved.push((*device, *offset, bytes));
    }
    let spacing = u64::from(params.geometry.fixed_structure_slot_spacing);
    let rotations: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = pool
        .devices
        .iter()
        .map(|(identity, _)| {
            let mut verified = singlefs_core::recovery::verified_system_configuration_slots(pool.devices.as_slice(), *identity, spacing, &params.filesystem_identifier);
            verified.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest] = <[_; 2]>::try_from(verified).expect("两槽都自证得过");
            let generation = newest.quantities.slot_generation;
            let mut not_written = older;
            not_written.quantities.slot_generation = generation.checked_sub(2).unwrap();
            (*identity, DeviceOffsetInBytes((generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing), not_written.to_slot())
        })
        .collect();
    for (device, offset, bytes) in &rotations {
        write_bytes(&mut pool, *device, *offset, bytes);
    }
    let abandoning = pool.crash_and_mount_writable().expect("择根落到 C 之前那一条");
    assert!(abandoning.effective_root.checkpoint_txg < c.root.checkpoint_txg);
    for (device, offset, bytes) in &saved {
        write_bytes(&mut pool, *device, *offset, bytes);
    }
    if after > 0 {
        // C 的字节写回之后再挂一次（实例 4，影子账看得见 C），再覆盖写：不然实例 3 那条会话没隔离 C，起点自己就红 I-7.4。
        pool.crash_and_mount_writable().expect("实例 4");
        for _ in 0..after {
            pool.overwrite(OVERWRITE_BYTES).expect("实例 4 覆盖写");
        }
    }
    let name = format!("ab-{}-{overwrites_before}-{after}", width_name(width));
    finish(&name, width, pool, false, Some((c.root.instance.0, c.root.checkpoint_txg.0)))
}

fn width_name(width: HistoryDeviceWidth) -> &'static str {
    match width {
        HistoryDeviceWidth::FourGibibytes => "4g",
        HistoryDeviceWidth::UnitAreaOf384Slots => "384",
        HistoryDeviceWidth::UnitAreaOf256Slots => "256",
        HistoryDeviceWidth::UnitAreaOf240Slots => "240",
    }
}

/// 成熟的环：实例 1 覆盖写 `first` 次，崩了再挂（实例 2）再覆盖写 `second` 次，进程退出。
fn mature_start(width: HistoryDeviceWidth, first: usize, second: usize) -> StartPoint {
    let mut pool = start_plain(width);
    for _ in 0..first {
        pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写");
    }
    pool.crash_and_mount_writable().expect("取号 2");
    for _ in 0..second {
        pool.overwrite(OVERWRITE_BYTES).expect("实例 2 覆盖写");
    }
    finish(&format!("mat-{}-{first}-{second}", width_name(width)), width, pool, false, None)
}

/// 窄盘（准入关着造的，第二轮攻方第一次抽样 `mature_starting_points` 那一形）：单元区 `slots` 槽、覆盖写 `overwrites` 次、顺序写 `units` 个单元，进程退出。
fn narrow_start(slots: u64, overwrites: usize, units: usize) -> StartPoint {
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let mut pool = start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(width, slots);
    for _ in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
    }
    pool.sequential_write((units - 1) * singlefs_core::unit::data_unit_payload_capacity() + 100).expect("顺序写");
    finish(&format!("narrow-{slots}-{overwrites}-{units}"), width, pool, true, None)
}

fn start_point_named(name: &str) -> StartPoint {
    match name {
        "ab-4g-1-0" => abandoned_start(HistoryDeviceWidth::FourGibibytes, 1, 0),
        "ab-4g-3-0" => abandoned_start(HistoryDeviceWidth::FourGibibytes, 3, 0),
        "ab-4g-3-5" => abandoned_start(HistoryDeviceWidth::FourGibibytes, 3, 5),
        "ab-384-1-0" => abandoned_start(HistoryDeviceWidth::UnitAreaOf384Slots, 1, 0),
        "ab-384-3-0" => abandoned_start(HistoryDeviceWidth::UnitAreaOf384Slots, 3, 0),
        "ab-384-3-5" => abandoned_start(HistoryDeviceWidth::UnitAreaOf384Slots, 3, 5),
        "mat-4g-30-8" => mature_start(HistoryDeviceWidth::FourGibibytes, 30, 8),
        "mat-384-30-8" => mature_start(HistoryDeviceWidth::UnitAreaOf384Slots, 30, 8),
        "narrow-264-6-4" => narrow_start(264, 6, 4),
        "narrow-384-20-2" => narrow_start(384, 20, 2),
        other => panic!("没有这个起点：{other}"),
    }
}

const ALL_START_POINTS: &[&str] = &[
    "ab-4g-1-0", "ab-4g-3-0", "ab-4g-3-5", "ab-384-1-0", "ab-384-3-0", "ab-384-3-5", "mat-4g-30-8", "mat-384-30-8", "narrow-264-6-4", "narrow-384-20-2",
];

// ---------------------------------------------------------------- 盘面真值

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotTruth {
    Valid(u64),
    Abandoned(u64),
    Zero,
    Bad,
}

struct Truth {
    slots: Vec<(RootRingSlot, DeviceIdentity, SlotTruth)>,
}

fn truth_of(image: &MemoryPool) -> Truth {
    let sc = choose_system_configuration(image).expect("系统配置");
    let im = sc.immutable.clone();
    let newest = choose_root(image, &sc).expect("择得到根");
    let table = instance_table_of_root(image, &newest).expect("最新那条根的实例表");
    let slots = every_root_ring_slot(&im.sizes)
        .into_iter()
        .map(|slot| {
            let device = im.region_devices[usize::try_from(slot.region).unwrap()];
            let offset = slot_offset(slot, im.sizes.fixed_structure_slot_spacing);
            let bytes = singlefs_core::recovery::PoolReader::read(image, device, offset, usize::try_from(im.sizes.physical_block_size).unwrap());
            let class = match read_root_ring_slot(image, &im.region_devices, &im.sizes, &im.filesystem_identifier, slot) {
                RootRingSlotReading::SelfVerified(root) => {
                    if root_is_abandoned_by_the_instance_table(&root, &table) {
                        SlotTruth::Abandoned(root.checkpoint_txg.0)
                    } else {
                        SlotTruth::Valid(root.checkpoint_txg.0)
                    }
                }
                RootRingSlotReading::Bad(_) => {
                    if bytes.map_or(false, |b| b.iter().all(|x| *x == 0)) {
                        SlotTruth::Zero
                    } else {
                        SlotTruth::Bad
                    }
                }
            };
            (slot, device, class)
        })
        .collect();
    Truth { slots }
}

fn ring_roots(image: &MemoryPool) -> Vec<(RootRingSlot, RootRecord)> {
    let sc = choose_system_configuration(image).expect("系统配置");
    let im = sc.immutable;
    every_root_ring_slot(&im.sizes)
        .into_iter()
        .filter_map(|s| match read_root_ring_slot(image, &im.region_devices, &im.sizes, &im.filesystem_identifier, s) {
            RootRingSlotReading::SelfVerified(root) => Some((s, root)),
            RootRingSlotReading::Bad(_) => None,
        })
        .collect()
}

fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
    MemoryPool {
        devices: image.devices.iter().filter(|(i, _)| **i != lost).map(|(i, d)| (*i, d.clone())).collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

fn files_at(image: &MemoryPool, root: &RootRecord) -> Result<Vec<(u64, Vec<u8>)>, String> {
    let mounted = open_pool_for_read(image, root).map_err(|f| format!("{f:?}"))?;
    let mut files = Vec::new();
    for record in mounted.inode_records() {
        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = mounted.open_file(image, InodeNumber(record.inode)).map_err(|f| format!("open {}: {f:?}", record.inode))?;
        let read = file.read_at(image, FileOffsetInBytes(0), record.size).map_err(|f| format!("read {}: {f:?}", record.inode))?;
        files.push((record.inode, read.bytes));
    }
    Ok(files)
}

type Baseline = BTreeMap<(RootRingSlot, u32, u64, u32), Result<Vec<(u64, Vec<u8>)>, String>>;

/// 挂载之前：环里每条自证根、丢掉每一块盘之后读回什么。
fn baseline_of(image: &MemoryPool) -> Baseline {
    let mut baseline = BTreeMap::new();
    for (slot, root) in ring_roots(image) {
        for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
            baseline.insert((slot, root.instance.0, root.checkpoint_txg.0, lost.0), files_at(&image_without(image, lost), &root));
        }
    }
    baseline
}

/// 还在环里、挂载之前丢盘读得回的根，现在丢同一块盘读回的不一样：(txg, 丢哪块, txg ≥ F)。
fn broken_roots(image: &MemoryPool, baseline: &Baseline, floor: u64) -> Vec<(u64, u32, bool)> {
    let mut broken = Vec::new();
    for (slot, root) in ring_roots(image) {
        for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
            let Some(before) = baseline.get(&(slot, root.instance.0, root.checkpoint_txg.0, lost.0)) else { continue };
            if before.is_err() {
                continue;
            }
            if files_at(&image_without(image, lost), &root) != *before {
                broken.push((root.checkpoint_txg.0, lost.0, root.checkpoint_txg.0 >= floor));
            }
        }
    }
    broken
}

/// 池级 checker 的违例按类：I-3.1 红在记账多的一边记 `I-3.1+`（用户认的代价），少的一边记 `I-3.1-`。
fn violation_classes(image: &MemoryPool) -> (Vec<String>, Vec<String>) {
    let violations = checker_violations_on(image);
    let classes = violations
        .iter()
        .map(|v| {
            let name = v.split(':').next().unwrap_or("").to_string();
            if name == "I-3.1" {
                match singlefs_harness::history::allocated_and_walked_bytes(v) {
                    Some((a, w)) if a > w => "I-3.1+".to_string(),
                    Some(_) => "I-3.1-".to_string(),
                    None => "I-3.1?".to_string(),
                }
            } else {
                name
            }
        })
        .collect();
    (classes, violations)
}

// ---------------------------------------------------------------- 挂载与之后的发布

struct MountRun {
    result: Result<Mounted, MountError>,
    devices: Vec<(DeviceIdentity, FaultDevice)>,
    hooks: Vec<BTreeMap<RootRingSlot, u64>>,
    fired: u64,
}

fn mount_with(sp: &StartPoint, shared: &Shared) -> MountRun {
    let mut devices = fault_devices_on(&sp.image, shared);
    let mut hooks = Vec::new();
    let result = {
        let seen = shared.clone();
        let mut hook = || hooks.push(seen.reads());
        mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
            &sp.parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
        )
    };
    let fired = shared.0.borrow().fired;
    MountRun { result, devices, hooks, fired }
}

fn outcome_of(result: &Result<Mounted, MountError>) -> String {
    match result {
        Ok(m) => format!(
            "Ok({})",
            match m.output.rereads.root_ring_of_the_rebuilt_allocator {
                RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead => "first",
                RootRingOfTheRebuiltAllocatorRead::OnTheOneReread => "reread",
            }
        ),
        Err(e) => format!("Err({})", member(e)),
    }
}

/// 重建分配器读根环那一遍是每个槽的第几次读：整环在第 k 次读报错，钩子恰调一次、调的那一刻每个槽恰读过 k 次、读阶段与实例表都第一遍判完。
fn rebuild_pass_read_number(sp: &StartPoint) -> u64 {
    let every = every_root_ring_slot(&sp.parameters.geometry);
    for k in 1..=20u64 {
        let shared = Shared::on(&sp.image);
        for s in &every {
            shared.0.borrow_mut().rules.insert((*s, k), Fault::Error);
        }
        let run = mount_with(sp, &shared);
        let exact = run.hooks.len() == 1 && run.hooks[0].len() == every.len() && run.hooks[0].values().all(|n| *n == k);
        if let Ok(m) = &run.result {
            let r = format!("{:?}", m.output.rereads);
            if exact
                && r.contains("read_stage: OnTheFirstRead")
                && r.contains("instance_table_of_the_newest_root: OnTheFirstRead")
                && m.output.rereads.root_ring_of_the_rebuilt_allocator == RootRingOfTheRebuiltAllocatorRead::OnTheOneReread
            {
                return k;
            }
        }
    }
    panic!("{}：认不出重建那一遍", sp.name)
}

#[derive(Clone, Debug, Default)]
struct Plan {
    id: String,
    rules: BTreeMap<(RootRingSlot, u64), Fault>,
    from_onward: BTreeMap<RootRingSlot, (u64, Fault)>,
    desc: String,
    /// 有一条被抛弃根的槽两遍都读坏（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形，已在欠账里）。
    c554: bool,
}

fn mark_c554(truth: &Truth, plan: &mut Plan, k: u64) {
    plan.c554 = truth.slots.iter().any(|(slot, _, t)| {
        matches!(t, SlotTruth::Abandoned(_))
            && plan.rules.contains_key(&(*slot, k))
            && (plan.rules.contains_key(&(*slot, k + 1)) || plan.from_onward.contains_key(slot))
    });
}

fn shared_with(sp: &StartPoint, plan: &Plan) -> Shared {
    let shared = Shared::on(&sp.image);
    shared.0.borrow_mut().rules = plan.rules.clone();
    shared.0.borrow_mut().from_onward = plan.from_onward.clone();
    shared
}

/// 挂上之后用户发 `steps` 次改动（有活文件就覆盖写、没有就建一个 inode；故障设备留着，从某次读起一直坏的槽在进程内照坏），
/// 每一步之后核：池级 checker 违例的类、环里 txg ≥ F 的根丢盘整读读回的变了没有。交回（第一处发现，最后一步的摘要）。
fn continue_and_check(sp: &StartPoint, mounted: Mounted, devices: Vec<(DeviceIdentity, FaultDevice)>, baseline: &Baseline, steps: usize) -> (Vec<String>, String) {
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    if sp.narrow {
        session.allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    }
    let mut pool: PoolUnderTest<FaultDevice> = PoolUnderTest {
        parameters: sp.parameters.clone(),
        device_bytes: sp.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: sp.write_time_seconds + 1000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut findings = Vec::new();
    let mut classes_seen: BTreeSet<String> = BTreeSet::new();
    let mut check = |pool: &PoolUnderTest<FaultDevice>, step: usize, findings: &mut Vec<String>, classes_seen: &mut BTreeSet<String>| {
        let image = pool.image();
        let floor = pool.session().current.root().rollback_floor.0;
        let (classes, violations) = violation_classes(&image);
        let broken = broken_roots(&image, baseline, floor);
        let protected: Vec<_> = broken.iter().filter(|(_, _, p)| *p).collect();
        for c in &classes {
            classes_seen.insert(c.clone());
        }
        if !protected.is_empty() {
            classes_seen.insert("broken>=F".to_string());
        }
        let red = classes.iter().any(|c| c != "I-3.1+") || !protected.is_empty();
        if red && findings.len() < 3 {
            let short: Vec<String> = violations.iter().map(|v| v.chars().take(220).collect()).collect();
            findings.push(format!("step={step} F={floor} txg={} classes={classes:?} broken>=F={protected:?} v={short:?}", pool.session().current.root().checkpoint_txg.0));
        }
    };
    check(&pool, 0, &mut findings, &mut classes_seen);
    let mut use_inodes = false;
    let mut refused = String::new();
    for step in 1..=steps {
        let outcome = if use_inodes {
            pool.create_one_inode().map(|_| ())
        } else {
            match pool.overwrite(OVERWRITE_BYTES) {
                Err(singlefs_core::mounted_session::UserChangeRefused::NoFileVersionToChange { .. }) => {
                    use_inodes = true;
                    pool.create_one_inode().map(|_| ())
                }
                other => other.map(|_| ()),
            }
        };
        if let Err(refusal) = outcome {
            refused = format!("refused_at={step}:{}", format!("{refusal:?}").chars().take(60).collect::<String>());
            break;
        }
        check(&pool, step, &mut findings, &mut classes_seen);
    }
    let summary = format!("classes={classes_seen:?} {refused}");
    (findings, summary)
}

/// 一段历史：起点 + 读故障计划 → 挂载 → 发 `steps` 次 → 每步核。交回（输出行、汇总键）。
fn run_plan(sp: &StartPoint, k: u64, plan: &Plan, baseline: &Baseline, steps: usize) -> (String, String) {
    let shared = shared_with(sp, plan);
    let run = mount_with(sp, &shared);
    let outcome = outcome_of(&run.result);
    let hook_ok = run.hooks.first().map_or(false, |h| h.values().all(|n| *n == k));
    let k_note = if plan.rules.keys().any(|(_, n)| *n == k) && !hook_ok { " k_mismatch" } else { "" };
    let (findings, summary, red) = match run.result {
        Ok(mounted) => {
            let isolated = format!("{:?}", mounted.output.isolated_slots_per_device);
            let (findings, summary) = continue_and_check(sp, mounted, run.devices, baseline, steps);
            let red = !findings.is_empty();
            (findings, format!("isolated={isolated} {summary}"), red)
        }
        Err(_) => {
            let after = MemoryPool {
                devices: run.devices.iter().map(|(i, d)| (*i, d.inner.image.clone())).collect(),
                device_size_in_bytes: sp.image.device_size_in_bytes,
            };
            let unchanged = after == sp.image;
            if unchanged {
                (Vec::new(), "image_unchanged=true".to_string(), false)
            } else {
                // 落了盘之后才报错的：盘面照池级 checker 与丢盘整读判。
                let (classes, violations) = violation_classes(&after);
                let broken = broken_roots(&after, baseline, 0);
                let red = classes.iter().any(|c| c != "I-3.1+") || !broken.is_empty();
                let short: Vec<String> = violations.iter().map(|v| v.chars().take(220).collect()).collect();
                (if red { vec![format!("after_err classes={classes:?} broken={broken:?} v={short:?}")] } else { Vec::new() }, format!("image_unchanged=false after_err_classes={classes:?}"), red)
            }
        }
    };
    let line = format!(
        "OPUS3-RUN sp={} arm={:?} plan={} c554={} desc={} outcome={outcome} hooks={} fired={}{k_note} red={red} {summary} findings={findings:?}",
        sp.name,
        arm(),
        plan.id,
        plan.c554,
        plan.desc,
        run.hooks.len(),
        run.fired
    );
    let key = format!("sp={} arm={:?} outcome={outcome} red={red} c554={}{k_note}", sp.name, arm(), plan.c554);
    (line, key)
}

// ---------------------------------------------------------------- 计划

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, p: f64) -> bool {
        ((self.next() >> 11) as f64) / ((1u64 << 53) as f64) < p
    }
    fn fault(&mut self) -> Fault {
        match self.below(3) {
            0 => Fault::Error,
            1 => Fault::Zeros,
            _ => Fault::Garbage,
        }
    }
}

fn class_letter(t: SlotTruth) -> &'static str {
    match t {
        SlotTruth::Valid(_) => "V",
        SlotTruth::Abandoned(_) => "A",
        SlotTruth::Zero => "Z",
        SlotTruth::Bad => "B",
    }
}

fn describe(truth: &Truth, plan: &Plan, k: u64) -> String {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for (slot, _, t) in &truth.slots {
        let p1 = plan.rules.get(&(*slot, k)).map_or("-", |f| f.short());
        let p2 = plan
            .rules
            .get(&(*slot, k + 1))
            .map(|f| f.short().to_string())
            .or_else(|| plan.from_onward.get(slot).map(|(_, f)| format!("{}*", f.short())))
            .unwrap_or_else(|| "-".to_string());
        *counts.entry(format!("{}:{p1}{p2}", class_letter(*t))).or_insert(0) += 1;
    }
    let abandoned: Vec<String> = truth
        .slots
        .iter()
        .filter(|(_, _, t)| matches!(t, SlotTruth::Abandoned(_)))
        .map(|(slot, _, t)| {
            let p1 = plan.rules.get(&(*slot, k)).map_or("-", |f| f.short());
            let p2 = plan.rules.get(&(*slot, k + 1)).map(|f| f.short().to_string()).or_else(|| plan.from_onward.get(slot).map(|(_, f)| format!("{}*", f.short()))).unwrap_or_else(|| "-".to_string());
            format!("{t:?}@{}/{}:{p1}{p2}", slot.region, slot.slot)
        })
        .collect();
    let c: Vec<String> = counts.iter().map(|(k, v)| format!("{k}={v}")).collect();
    format!("[{}] abandoned={abandoned:?}", c.join(","))
}

/// 随机计划：第一遍把盘 d 上每个住着有效根的槽读坏（重读一定触发），盘 d 上别的槽一半读坏，另一块盘按 p1 读坏；
/// 重读那一遍每个槽按 p2 读坏；p 取自几档。读坏的种类在 报错 / 全零 / 乱字节里均匀取。有两成计划把重读那一遍报错的槽改成「从这次读起一直坏」。
fn random_plan(truth: &Truth, k: u64, seed: u64, index: u64) -> Plan {
    let mut rng = Rng(seed.wrapping_mul(1_000_003).wrapping_add(index));
    let d = DeviceIdentity(u32::try_from(rng.below(2)).unwrap());
    let p1 = [0.0, 0.15, 0.4][usize::try_from(rng.below(3)).unwrap()];
    let p2 = [0.0, 0.25, 0.5, 0.85, 1.0][usize::try_from(rng.below(5)).unwrap()];
    let persist = rng.chance(0.2);
    let mut plan = Plan { id: format!("r{seed}-{index}"), ..Plan::default() };
    for (slot, device, class) in &truth.slots {
        let on_d = *device == d;
        let fault_first = if on_d { matches!(class, SlotTruth::Valid(_)) || rng.chance(0.5) } else { rng.chance(p1) };
        if fault_first {
            plan.rules.insert((*slot, k), rng.fault());
        }
        if rng.chance(p2) {
            let f = rng.fault();
            if persist && f == Fault::Error {
                plan.from_onward.insert(*slot, (k + 1, Fault::Error));
            } else {
                plan.rules.insert((*slot, k + 1), f);
            }
        }
    }
    plan.desc = format!("d={} p1={p1} p2={p2} persist={persist} {}", d.0, describe(truth, &plan, k));
    mark_c554(truth, &mut plan, k);
    plan
}

fn fixed_plans(truth: &Truth, k: u64) -> Vec<Plan> {
    let mut plans = vec![Plan { id: "f-nofault".into(), ..Plan::default() }];
    let abandoned: Vec<RootRingSlot> = truth.slots.iter().filter(|(_, _, t)| matches!(t, SlotTruth::Abandoned(_))).map(|(s, _, _)| *s).collect();
    for d in [DeviceIdentity(0), DeviceIdentity(1)] {
        let on_d: Vec<RootRingSlot> = truth.slots.iter().filter(|(_, dev, _)| *dev == d).map(|(s, _, _)| *s).collect();
        let mut base = Plan::default();
        for s in &on_d {
            base.rules.insert((*s, k), Fault::Error);
        }
        let mut p = base.clone();
        p.id = format!("f-d{}-allE", d.0);
        plans.push(p);
        let mut p = base.clone();
        p.id = format!("f-d{}-allE-reread-allE", d.0);
        for (s, _, _) in &truth.slots {
            p.rules.insert((*s, k + 1), Fault::Error);
        }
        plans.push(p);
        let mut p = base.clone();
        p.id = format!("f-d{}-allE-reread-allE-persist", d.0);
        for (s, _, _) in &truth.slots {
            p.from_onward.insert(*s, (k + 1, Fault::Error));
        }
        plans.push(p);
        for f in [Fault::Error, Fault::Zeros, Fault::Garbage] {
            for a in &abandoned {
                let mut p = base.clone();
                p.id = format!("f-d{}-allE-reread-A{}/{}-{}", d.0, a.region, a.slot, f.short());
                p.rules.insert((*a, k + 1), f.clone());
                plans.push(p);
            }
        }
        // 第一遍坏与全零交错：盘 d 每个槽第一遍读坏，重读那一遍把它们读回全零（读坏压过全零那一格）。
        let mut p = base.clone();
        p.id = format!("f-d{}-allE-reread-d{}-allZ", d.0, d.0);
        for s in &on_d {
            p.rules.insert((*s, k + 1), Fault::Zeros);
        }
        plans.push(p);
        let mut p = Plan::default();
        p.id = format!("f-d{}-allZ-reread-d{}-allE", d.0, d.0);
        for s in &on_d {
            p.rules.insert((*s, k), Fault::Zeros);
            p.rules.insert((*s, k + 1), Fault::Error);
        }
        plans.push(p);
    }
    for p in &mut plans {
        p.desc = describe(truth, p, k);
        mark_c554(truth, p, k);
    }
    plans
}

fn run_plans_threaded(sp: &StartPoint, k: u64, baseline: &Baseline, plans: &[Plan], steps: usize, workers: usize) -> Vec<(String, String)> {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: std::sync::Mutex<Vec<(usize, String, String)>> = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..workers.max(1) {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(plan) = plans.get(i) else { break };
                let (line, key) = run_plan(sp, k, plan, baseline, steps);
                results.lock().unwrap().push((i, line, key));
            });
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|(i, _, _)| *i);
    results.into_iter().map(|(_, l, k)| (l, k)).collect()
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn truth_summary(truth: &Truth) -> String {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for (_, d, t) in &truth.slots {
        *counts.entry(format!("d{}:{}", d.0, class_letter(*t))).or_insert(0) += 1;
    }
    let abandoned: Vec<String> = truth.slots.iter().filter(|(_, _, t)| matches!(t, SlotTruth::Abandoned(_))).map(|(s, d, t)| format!("{t:?}@{}/{}(d{})", s.region, s.slot, d.0)).collect();
    format!("{counts:?} abandoned={abandoned:?}")
}

/// grid：每个起点先跑固定的几格，再跑 OPUS_PLANS 个随机计划（种子 OPUS_SEED），每段挂上之后发 OPUS_STEPS 次改动。
fn grid(start_points: &[&str]) {
    let seed = env_usize("OPUS_SEED", 1) as u64;
    let n = env_usize("OPUS_PLANS", 50) as u64;
    let steps = env_usize("OPUS_STEPS", 30);
    let workers = env_usize("OPUS_WORKERS", 1);
    let fixed_only = std::env::var_os("OPUS_FIXED_ONLY").is_some();
    for name in start_points {
        let t0 = std::time::Instant::now();
        let sp = start_point_named(name);
        let k = rebuild_pass_read_number(&sp);
        let truth = truth_of(&sp.image);
        let baseline = baseline_of(&sp.image);
        println!("OPUS3-SP sp={} arm={:?} k={k} abandoned={:?} truth={} build_s={:.1}", sp.name, arm(), sp.abandoned, truth_summary(&truth), t0.elapsed().as_secs_f64());
        let mut plans = fixed_plans(&truth, k);
        if !fixed_only {
            plans.extend((0..n).map(|i| random_plan(&truth, k, seed, i)));
        }
        let t1 = std::time::Instant::now();
        let results = run_plans_threaded(&sp, k, &baseline, &plans, steps, workers);
        let mut summary: BTreeMap<String, u64> = BTreeMap::new();
        for (line, key) in &results {
            println!("{line}");
            *summary.entry(key.clone()).or_insert(0) += 1;
        }
        for (key, count) in &summary {
            println!("OPUS3-SUM {key} n={count}");
        }
        println!("OPUS3-TIME sp={} plans={} steps={steps} workers={workers} run_s={:.1}", sp.name, plans.len(), t1.elapsed().as_secs_f64());
    }
}

// ---------------------------------------------------------------- 故障模型之外：错位读（K1 第一格「同一槽两遍读到两个 txg」）

fn slot_bytes(image: &MemoryPool, slot: RootRingSlot) -> Vec<u8> {
    let im = choose_system_configuration(image).expect("系统配置").immutable;
    let device = im.region_devices[usize::try_from(slot.region).unwrap()];
    let offset = slot_offset(slot, im.sizes.fixed_structure_slot_spacing);
    singlefs_core::recovery::PoolReader::read(image, device, offset, usize::try_from(im.sizes.fixed_structure_slot_spacing).unwrap()).expect("读得出")
}

fn cells(start_points: &[&str]) {
    let steps = env_usize("OPUS_STEPS", 30);
    for name in start_points {
        let sp = start_point_named(name);
        let k = rebuild_pass_read_number(&sp);
        let truth = truth_of(&sp.image);
        let baseline = baseline_of(&sp.image);
        let Some((c_slot, c_device)) = truth.slots.iter().find(|(_, _, t)| matches!(t, SlotTruth::Abandoned(_))).map(|(s, d, _)| (*s, *d)) else { continue };
        let other = DeviceIdentity(1 - c_device.0);
        // 同一块盘上另一条有效根的槽：错位读把它的字节当成 C 那一槽读回来（自证得过：根记录里没有槽号）。
        let Some(substitute) = truth.slots.iter().find(|(s, d, t)| *d == c_device && *s != c_slot && matches!(t, SlotTruth::Valid(_))).map(|(s, _, t)| (*s, *t)) else { continue };
        let bytes = slot_bytes(&sp.image, substitute.0);
        let mut base = Plan::default();
        for (s, d, _) in &truth.slots {
            if *d == other {
                base.rules.insert((*s, k), Fault::Error);
            }
        }
        let mut m1 = base.clone();
        m1.id = format!("m1-reread-C-reads-{:?}@{}/{}", substitute.1, substitute.0.region, substitute.0.slot);
        m1.rules.insert((c_slot, k + 1), Fault::Bytes(bytes.clone()));
        let mut m2 = base.clone();
        m2.id = format!("m2-first-C-reads-{:?}@{}/{}", substitute.1, substitute.0.region, substitute.0.slot);
        m2.rules.insert((c_slot, k), Fault::Bytes(bytes.clone()));
        let mut plans = vec![base.clone(), m1, m2];
        plans[0].id = format!("m0-d{}-allE", other.0);
        for p in &mut plans {
            p.desc = format!("C@{}/{}(d{})", c_slot.region, c_slot.slot, c_device.0);
        }
        println!("OPUS3-SP sp={} arm={:?} k={k} truth={}", sp.name, arm(), truth_summary(&truth));
        for plan in &plans {
            let (line, _) = run_plan(&sp, k, plan, &baseline, steps);
            println!("{line}");
        }
    }
}

// ---------------------------------------------------------------- 层 0

use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments_with_stream_indexes_and_entries, CrashImage, PublishedVersion, RetainedWrite};
use singlefs_harness::segments::StepKind;

fn materialize(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    let persisted: Vec<RetainedWrite> = image.writes.iter().zip(&image.persisted).filter(|(_, p)| **p).map(|(w, _)| w.clone()).collect();
    pool.apply_writes(&persisted);
    pool
}

fn last_root_write(writes: &[RetainedWrite]) -> usize {
    writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("流里有根槽写")
}

/// 一份盘面上的按槽归盘与 Q3 判官前提：(落在公式算不到的槽里的非第 0 代根, 第 0 代根副本个数, 最新那条根的实例表里有它自己实例的行)。
fn slot_scan(image: &MemoryPool) -> (Vec<String>, usize, bool) {
    let im = match choose_system_configuration(image) {
        Ok(sc) => sc.immutable,
        Err(_) => return (vec!["no-sc".into()], 0, false),
    };
    let mut off = Vec::new();
    let mut genesis = 0;
    for (slot, root) in ring_roots(image) {
        if root.checkpoint_txg.0 == 0 {
            genesis += 1;
        } else if target_for_publish(root.checkpoint_txg, im.sizes.root_ring_slots_per_region) != slot {
            off.push(format!("txg{}@{}/{}", root.checkpoint_txg.0, slot.region, slot.slot));
        }
    }
    let sc = choose_system_configuration(image).unwrap();
    let own_row = choose_root(image, &sc)
        .and_then(|newest| instance_table_of_root(image, &newest).map(|t| (newest, t)))
        .map_or(false, |(newest, t)| t.rows.iter().any(|r| r.instance >= newest.instance) || root_is_abandoned_by_the_instance_table(&newest, &t));
    (off, genesis, own_row)
}

/// l0：起点 + 计划（按 id 在固定计划与随机计划里找）→ 录下挂载与之后 OPUS_STEPS 次改动的写流 → 每一段都展开的层 0 枚举，
/// 每个崩溃态恢复后跑 checker 与 oracle（版本表带着起点与这一段写过的每条根的内容），再逐状态扫按槽归盘与 Q3 判官前提。
fn l0(name: &str, plan_id: &str) {
    let steps = env_usize("OPUS_STEPS", 4);
    let max_states = env_usize("OPUS_L0_MAX_STATES", 1_000_000) as u64;
    let plan_only = std::env::var_os("OPUS_L0_PLAN_ONLY").is_some();
    let sp = start_point_named(name);
    let k = rebuild_pass_read_number(&sp);
    let truth = truth_of(&sp.image);
    let seed = env_usize("OPUS_SEED", 1) as u64;
    let n = env_usize("OPUS_PLANS", 50) as u64;
    let mut plans = fixed_plans(&truth, k);
    plans.extend((0..n).map(|i| random_plan(&truth, k, seed, i)));
    let plan = plans.into_iter().find(|p| p.id == plan_id).unwrap_or_else(|| panic!("没有计划 {plan_id}"));
    let shared = shared_with(&sp, &plan);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<FaultDevice>)> = fault_devices_on(&sp.image, &shared)
        .into_iter()
        .map(|(i, d)| (i, RecordingBlockDevice::with_shared_stream(i, d, stream.clone())))
        .collect();
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &sp.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::RereadImmediately,
    );
    let outcome = outcome_of(&mounted);
    let mut published = 0usize;
    let versions: Vec<PublishedVersion>;
    match mounted {
        Ok(mounted) => {
            let (output, mut session) = MountedSession::of_the_mount(mounted);
            if sp.narrow {
                session.allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
            }
            let mut pool: PoolUnderTest<RecordingBlockDevice<FaultDevice>> = PoolUnderTest {
                parameters: sp.parameters.clone(),
                device_bytes: sp.device_bytes,
                devices,
                session: Some(session),
                write_time_seconds: sp.write_time_seconds + 1000,
                content_of_the_current_version: sp.contents.get(&(output.effective_root.instance, output.effective_root.checkpoint_txg)).cloned().unwrap_or_default(),
                content_of_each_root: sp.contents.clone(),
            };
            let roots: Vec<RootRecord> = std::iter::once(&output.row_publish).chain(&output.warm_up_publishes).map(|v| *v.root()).collect();
            for root in &roots {
                pool.note_root_of_the_current_content(root);
            }
            pool.note_floor_raises(output.space_admission.floor_raises());
            for _ in 0..steps {
                if pool.overwrite(OVERWRITE_BYTES).is_err() {
                    break;
                }
                published += 1;
            }
            versions = pool.published_versions();
        }
        Err(_) => versions = Vec::new(),
    }
    let operations = stream.retained_operations();
    let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(&operations, &[], &sp.width.fixed_geometry());
    let states = closed_form_state_count(&segments);
    println!(
        "OPUS3-L0-PLAN sp={} arm={:?} plan={} outcome={outcome} published={published} writes={} segments={} closed_form_state_count={states} versions={}",
        sp.name,
        arm(),
        plan.id,
        writes.len(),
        segments.len(),
        versions.len()
    );
    if plan_only || writes.is_empty() {
        return;
    }
    if states > max_states {
        println!("OPUS3-L0-SKIP sp={} plan={} states={states} > {max_states}", sp.name, plan.id);
        return;
    }
    let judged = last_root_write(&writes);
    let mut off_states = 0u64;
    let mut off_examples: BTreeSet<String> = BTreeSet::new();
    let mut genesis_hist: BTreeMap<usize, u64> = BTreeMap::new();
    let mut own_row_states = 0u64;
    let mut observed = 0u64;
    let mut observe = |image: &CrashImage<'_>, _report: &RecoveryReport| {
        observed += 1;
        let m = materialize(image);
        let (off, genesis, own_row) = slot_scan(&m);
        if !off.is_empty() {
            off_states += 1;
            for o in off.into_iter().take(3) {
                off_examples.insert(o);
            }
        }
        *genesis_hist.entry(genesis).or_insert(0) += 1;
        if own_row {
            own_row_states += 1;
        }
    };
    let t0 = std::time::Instant::now();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(&sp.image, &writes, &segments, judged, &versions, &|_, _| true, &mut observe);
    println!(
        "OPUS3-L0-TALLY sp={} arm={:?} plan={} states={} oracle_violations={} first_violation={:?} checker_violated={:?} checker_first={:?} failed_states={} secs={:.1}",
        sp.name,
        arm(),
        plan.id,
        tally.states,
        tally.violations,
        tally.first_violation.as_deref().map(|s| s.chars().take(300).collect::<String>()),
        tally.checker_violated_states,
        tally.checker_first_violation.iter().map(|(k, v)| (k.to_string(), v.chars().take(300).collect::<String>())).collect::<BTreeMap<_, _>>(),
        tally.failed_states,
        t0.elapsed().as_secs_f64()
    );
    println!(
        "OPUS3-L0-SCAN sp={} plan={} observed={observed} off_formula_states={off_states} off_examples={off_examples:?} genesis_copies_hist={genesis_hist:?} newest_root_with_own_row_or_abandoned={own_row_states}",
        sp.name, plan.id
    );
}

// ---------------------------------------------------------------- K2：mkfs 之后第一次可写挂载的每个崩溃态

fn k2young(width: HistoryDeviceWidth) {
    let parameters = width.parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|i| (i, SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let fresh = MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: width.device_bytes() };
    let stream = SharedStream::retaining_contents();
    let mut rec: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&fresh).into_iter().map(|(i, d)| (i, RecordingBlockDevice::with_shared_stream(i, d, stream.clone()))).collect();
    singlefs_core::mount::mount_writable(&parameters, &mut rec).expect("第一次可写挂载");
    let operations = stream.retained_operations();
    let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(&operations, &[], &width.fixed_geometry());
    let states = closed_form_state_count(&segments);
    println!("OPUS3-K2Y-PLAN width={} writes={} segments={} closed_form_state_count={states}", width_name(width), writes.len(), segments.len());
    let judged = last_root_write(&writes);
    let sp = StartPoint {
        name: format!("young-{}", width_name(width)),
        width,
        parameters: parameters.clone(),
        device_bytes: width.device_bytes(),
        image: fresh.clone(),
        narrow: false,
        write_time_seconds: 1_788_000_000,
        contents: BTreeMap::new(),
        abandoned: None,
    };
    let mut summary: BTreeMap<String, u64> = BTreeMap::new();
    let mut observe = |image: &CrashImage<'_>, _report: &RecoveryReport| {
        let m = materialize(image);
        let (off, genesis, own_row) = slot_scan(&m);
        // 每块盘读得出的自证根（按读出它的槽所在的区域归盘）；只剩第 0 代根的盘。
        let im = choose_system_configuration(&m).ok().map(|sc| sc.immutable);
        let mut by_slot: BTreeMap<u32, Vec<u64>> = BTreeMap::new();
        let mut by_formula: BTreeMap<u32, Vec<u64>> = BTreeMap::new();
        if let Some(im) = &im {
            for (slot, root) in ring_roots(&m) {
                by_slot.entry(im.region_devices[usize::try_from(slot.region).unwrap()].0).or_default().push(root.checkpoint_txg.0);
                let t = target_for_publish(root.checkpoint_txg, im.sizes.root_ring_slots_per_region);
                by_formula.entry(im.region_devices[usize::try_from(t.region).unwrap()].0).or_default().push(root.checkpoint_txg.0);
            }
        }
        let differs = (0..2u32).any(|d| by_slot.get(&d).map_or(0, |v| v.len()).min(1) != by_formula.get(&d).map_or(0, |v| v.len()).min(1));
        let image_sp = StartPoint { image: m.clone(), ..sp.clone() };
        let shared = Shared::on(&m);
        let run = mount_with(&image_sp, &shared);
        let outcome = outcome_of(&run.result);
        let checker = match &run.result {
            Ok(_) => {
                let after = MemoryPool { devices: run.devices.iter().map(|(i, d)| (*i, d.inner.image.clone())).collect(), device_size_in_bytes: m.device_size_in_bytes };
                let (classes, _) = violation_classes(&after);
                format!("{classes:?}")
            }
            Err(_) => "-".into(),
        };
        *summary
            .entry(format!("genesis_copies={genesis} formula_vs_slot_differs={differs} off={} own_row={own_row} outcome={outcome} hooks={} checker_after={checker}", off.len(), run.hooks.len()))
            .or_insert(0) += 1;
    };
    let tally = enumerate_layer0_selecting_versions_observing_each_state(&fresh, &writes, &segments, judged, &[], &|_, _| true, &mut observe);
    println!("OPUS3-K2Y-TALLY width={} states={} checker_violated={:?}", width_name(width), tally.states, tally.checker_violated_states);
    for (key, n) in &summary {
        println!("OPUS3-K2Y-SUM width={} {key} n={n}", width_name(width));
    }
}


// ---------------------------------------------------------------- s1grid（第 10i 批新类）

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ops {
    Overwrite,
    Inode,
    Mix,
    OverwriteThenUnmount,
}

fn ops_from_env() -> Ops {
    match std::env::var("OPUS_OPS").as_deref() {
        Ok("inode") => Ops::Inode,
        Ok("mix") => Ops::Mix,
        Ok("ow-unmount") => Ops::OverwriteThenUnmount,
        _ => Ops::Overwrite,
    }
}

fn read_counters() -> [u64; 4] {
    singlefs_core::mount::OPUS_I1_TL.with(|c| *c.borrow())
}

/// 造 S1 槽的计划：槽 s（住有效根 V 或被抛弃根 A），触发重读的盘 d（第一遍 d 上每个住有效根的槽读坏），s 的读法三种：
/// a = 第一遍读得出、重读起一直坏；b = 第一遍坏、重读读得出、之后一直坏；c = 第一遍读得出、重读坏、之后每读三次有一次读得出。
fn s1_plans(truth: &Truth, k: u64) -> Vec<Plan> {
    let mut plans = Vec::new();
    if std::env::var_os("OPUS_BASELINE").is_some() {
        let mut p = Plan { id: "s1-N@0/0(d0)-trig0-n-E".into(), ..Plan::default() };
        p.desc = describe(truth, &p, k);
        plans.push(p);
        let mut q = Plan { id: "s1-T@0/0(d0)-trig0-t-E".into(), ..Plan::default() };
        for (t, tdev, tclass) in &truth.slots {
            if *tdev == DeviceIdentity(0) && matches!(tclass, SlotTruth::Valid(_)) {
                q.rules.insert((*t, k), Fault::Error);
            }
        }
        q.desc = describe(truth, &q, k);
        plans.push(q);
        return plans;
    }
    let only: Option<String> = std::env::var("OPUS_S1_ONLY_CLASS").ok();
    for (s, sdev, class) in &truth.slots {
        let letter = class_letter(*class);
        if !matches!(class, SlotTruth::Valid(_) | SlotTruth::Abandoned(_)) {
            continue;
        }
        if let Some(o) = &only {
            if o != letter {
                continue;
            }
        }
        for d in [DeviceIdentity(0), DeviceIdentity(1)] {
            let mut base = Plan::default();
            for (t, tdev, tclass) in &truth.slots {
                if *tdev == d && matches!(tclass, SlotTruth::Valid(_)) {
                    base.rules.insert((*t, k), Fault::Error);
                }
            }
            let s_bad_first = base.rules.contains_key(&(*s, k));
            for f in [Fault::Error, Fault::Zeros, Fault::Garbage] {
                for pattern in ["a", "b", "c"] {
                    let mut p = base.clone();
                    match pattern {
                        "a" => {
                            if s_bad_first {
                                continue;
                            }
                            p.from_onward.insert(*s, (k + 1, f.clone()));
                        }
                        "b" => {
                            p.rules.insert((*s, k), f.clone());
                            p.rules.insert((*s, k + 1), Fault::Error);
                            p.rules.remove(&(*s, k + 1));
                            p.from_onward.insert(*s, (k + 2, f.clone()));
                        }
                        _ => {
                            if s_bad_first {
                                continue;
                            }
                            for n in (k + 1)..(k + 400) {
                                if (n - k) % 3 != 0 {
                                    p.rules.insert((*s, n), f.clone());
                                }
                            }
                        }
                    }
                    p.id = format!("s1-{letter}@{}/{}(d{})-trig{}-{}-{}", s.region, s.slot, sdev.0, d.0, pattern, f.short());
                    p.desc = describe(truth, &p, k);
                    mark_c554(truth, &mut p, k);
                    plans.push(p);
                }
            }
        }
    }
    plans
}

/// 挂上之后按 ops 发 steps 次（OverwriteThenUnmount：覆盖写 steps 次再正常卸载），每步核 checker 违例类与丢盘整读（有效根 ≥ F 与被抛弃根分开记）。
fn continue_i1(sp: &StartPoint, mounted: Mounted, devices: Vec<(DeviceIdentity, FaultDevice)>, baseline: &Baseline, abandoned: &BTreeSet<(u32, u64)>, steps: usize, ops: Ops) -> (Vec<String>, String) {
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    if sp.narrow {
        session.allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    }
    let mut pool: PoolUnderTest<FaultDevice> = PoolUnderTest {
        parameters: sp.parameters.clone(),
        device_bytes: sp.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: sp.write_time_seconds + 1000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let mut findings = Vec::new();
    let mut classes_seen: BTreeSet<String> = BTreeSet::new();
    let check = |image: &MemoryPool, floor: u64, txg: u64, step: usize, findings: &mut Vec<String>, classes_seen: &mut BTreeSet<String>| {
        let (classes, violations) = violation_classes(image);
        let broken = broken_roots(image, baseline, floor);
        let protected: Vec<_> = broken.iter().filter(|(t, _, p)| *p && !abandoned.iter().any(|(_, at)| at == t)).collect();
        let broken_abandoned: Vec<_> = broken.iter().filter(|(t, _, _)| abandoned.iter().any(|(_, at)| at == t)).collect();
        for c in &classes {
            classes_seen.insert(c.clone());
        }
        if !protected.is_empty() {
            classes_seen.insert("broken>=F".to_string());
        }
        if !broken_abandoned.is_empty() {
            classes_seen.insert("broken-abandoned".to_string());
        }
        let red = classes.iter().any(|c| c != "I-3.1+") || !protected.is_empty() || !broken_abandoned.is_empty();
        if red && !findings.iter().any(|f: &String| f.starts_with("CHECKER ")) {
            let short: Vec<String> = violations.iter().map(|v| v.chars().take(500).collect()).collect();
            findings.push(format!("CHECKER step={step} F={floor} txg={txg} classes={classes:?} broken>=F={protected:?} broken_abandoned={broken_abandoned:?} v={short:?}"));
        } else if red && findings.len() < 3 {
            let short: Vec<String> = violations.iter().map(|v| v.chars().take(260).collect()).collect();
            findings.push(format!("step={step} F={floor} txg={txg} classes={classes:?} broken>=F={protected:?} broken_abandoned={broken_abandoned:?} v={short:?}"));
        }
    };
    {
        let image = pool.image();
        check(&image, pool.session().current.root().rollback_floor.0, pool.session().current.root().checkpoint_txg.0, 0, &mut findings, &mut classes_seen);
    }
    let mut use_inodes = ops == Ops::Inode;
    let mut refused = String::new();
    let mut floors: Vec<u64> = Vec::new();
    let mut latent_steps: Vec<usize> = Vec::new();
    let mut txgs: Vec<u64> = Vec::new();
    for step in 1..=steps {
        let seq_units = env_usize("OPUS_SEQ_UNITS_AFTER_LATENT", 0);
        let seq_from = env_usize("OPUS_SEQ_FROM_STEP", usize::MAX);
        let outcome = if seq_units > 0 && (step >= seq_from || (seq_from == usize::MAX && !latent_steps.is_empty())) {
            pool.sequential_write(seq_units * singlefs_core::unit::data_unit_payload_capacity() + 100).map(|_| ())
        } else if use_inodes || (ops == Ops::Mix && step % 2 == 0) {
            pool.create_one_inode().map(|_| ())
        } else {
            match pool.overwrite(OVERWRITE_BYTES) {
                Err(singlefs_core::mounted_session::UserChangeRefused::NoFileVersionToChange { .. }) => {
                    use_inodes = true;
                    pool.create_one_inode().map(|_| ())
                }
                other => other.map(|_| ()),
            }
        };
        if let Err(refusal) = outcome {
            refused = format!("refused_at={step}:{}", format!("{refusal:?}").chars().take(600).collect::<String>());
            break;
        }
        let f = pool.session().current.root().rollback_floor.0;
        txgs.push(pool.session().current.root().checkpoint_txg.0);
        if floors.last() != Some(&f) {
            floors.push(f);
        }
        let image = pool.image();
        check(&image, f, pool.session().current.root().checkpoint_txg.0, step, &mut findings, &mut classes_seen);
        let (latent, sample) = latent_abandoned_free(&pool.session().allocator);
        if latent > 0 {
            latent_steps.push(step);
            classes_seen.insert("latent-abandoned-free".to_string());
            if findings.len() < 3 {
                findings.push(format!("step={step} F={f} latent_abandoned_free={latent} {sample:?}"));
            }
        }
    }
    let mut rollback_note = String::new();
    if std::env::var_os("OPUS_ROLLBACK_AT_END").is_some() && refused.is_empty() {
        // K2：挂着时管理员回退到环里一条有效根（按无故障盘面选：F 之上、比现行旧的那几条里按 OPUS_ROLLBACK_PICK 取，默认最新的那条）。
        let image = pool.image();
        let floor = pool.session().current.root().rollback_floor.0;
        let cur = *pool.session().current.root();
        let sc = choose_system_configuration(&image).expect("系统配置");
        let newest = choose_root(&image, &sc).expect("择根");
        let table = instance_table_of_root(&image, &newest).expect("表");
        // 无故障盘面上的 F_生效（正推腿加：判「回退目标与 F_生效 判错」的真值）。
        let true_floor = singlefs_core::recovery::effective_rollback_floor_rereading_the_newest_instance_table_once(
            &image,
            &sc.immutable.region_devices,
            &sc.immutable.sizes,
            &sc.immutable.filesystem_identifier,
        )
        .map(|f| f.0)
        .unwrap_or(0);
        let below = std::env::var_os("OPUS_ROLLBACK_BELOW").is_some();
        let mut targets: Vec<RootRecord> = ring_roots(&image)
            .into_iter()
            .map(|(_, r)| r)
            .filter(|r| {
                !root_is_abandoned_by_the_instance_table(r, &table)
                    && r.checkpoint_txg < cur.checkpoint_txg
                    && if below { r.checkpoint_txg.0 < true_floor } else { r.checkpoint_txg.0 >= floor }
            })
            .collect();
        targets.sort_by_key(|r| std::cmp::Reverse(r.checkpoint_txg));
        let pick = if std::env::var_os("SONNET_PICK_LAST").is_some() { targets.len().saturating_sub(1) } else { env_usize("OPUS_ROLLBACK_PICK", 0) };
        match targets.get(pick) {
            None => rollback_note = format!(" rollback=no-target(n={})", targets.len()),
            Some(target) => {
                let mut session = pool.session.take().expect("会话");
                let result = match &mut session.current {
                    singlefs_core::transaction::PoolVersion::WithFile(v) | singlefs_core::transaction::PoolVersion::WithTreesAndNoFile(v) => {
                        Some(singlefs_core::mount::roll_back_by_a_forward_publish(
                            &sp.parameters,
                            &mut pool.devices,
                            &mut session.allocator,
                            v,
                            singlefs_core::mount::RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg },
                        ))
                    }
                    _ => None,
                };
                let expect = if target.checkpoint_txg.0 >= true_floor { "cand" } else { "below" };
                rollback_note = match result {
                    None => " rollback=no-file-version".to_string(),
                    Some(Ok(_)) => format!(" rollback=Ok(txg{}) expect={expect} trueF={true_floor}", target.checkpoint_txg.0),
                    Some(Err(e)) => format!(" rollback=Err(txg{}:{}) expect={expect} trueF={true_floor}", target.checkpoint_txg.0, format!("{e:?}").chars().take(400).collect::<String>().replace(' ', "")),
                };
                pool.session = Some(session);
                let image = pool.image();
                check(&image, floor, 0, steps + 1, &mut findings, &mut classes_seen);
                // 正推腿加：回退之后接着覆盖写 SONNET_AFTER_ROLLBACK_STEPS 次（回退成功才有意义），每步核 checker（看回退留下的账之后再抬 F / 发布会不会红）。
                let after = env_usize("SONNET_AFTER_ROLLBACK_STEPS", 0);
                if after > 0 && rollback_note.contains("rollback=Ok") {
                    let mut done = 0usize;
                    let mut stop = String::new();
                    for k in 1..=after {
                        match pool.overwrite(OVERWRITE_BYTES) {
                            Ok(_) => done = k,
                            Err(refusal) => {
                                stop = format!(" after_rollback_refused_at={k}:{}", format!("{refusal:?}").chars().take(120).collect::<String>());
                                break;
                            }
                        }
                        let image = pool.image();
                        let f = pool.session().current.root().rollback_floor.0;
                        check(&image, f, pool.session().current.root().checkpoint_txg.0, steps + 1 + k, &mut findings, &mut classes_seen);
                    }
                    rollback_note.push_str(&format!(" after_rollback_ok={done}{stop}"));
                }
            }
        }
    }
    let mut unmount_note = String::new();
    if ops == Ops::OverwriteThenUnmount {
        let mut session = pool.session.take().expect("会话");
        let result = singlefs_core::mount::unmount(&sp.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
        unmount_note = format!(" unmount={}", match &result { Ok(_) => "Ok".to_string(), Err(e) => format!("Err({})", member(e)) });
        let (latent, sample) = latent_abandoned_free(&session.allocator);
        if latent > 0 {
            classes_seen.insert("latent-abandoned-free-after-unmount".to_string());
            if findings.len() < 3 {
                findings.push(format!("unmount latent_abandoned_free={latent} {sample:?}"));
            }
        }
        let image = MemoryPool { devices: pool.devices.iter().map(|(i, d)| (*i, d.inner.image.clone())).collect(), device_size_in_bytes: sp.image.device_size_in_bytes };
        let floor = match &session.current {
            singlefs_core::transaction::PoolVersion::WithFile(v) | singlefs_core::transaction::PoolVersion::WithTreesAndNoFile(v) => v.root.rollback_floor.0,
            singlefs_core::transaction::PoolVersion::WithoutFile(v) => v.root.rollback_floor.0,
        };
        check(&image, floor, 0, steps + 1, &mut findings, &mut classes_seen);
    }
    let latent_span = match (latent_steps.first(), latent_steps.last()) { (Some(a), Some(b)) => format!("{a}..{b}"), _ => "-".into() };
    let summary = format!("floors={floors:?} latent_steps={latent_span} txg_first_last={:?} classes={classes_seen:?} {refused}{unmount_note}{rollback_note}", (txgs.first(), txgs.last()));
    (findings, summary)
}


/// 潜在的影子账漏洞：根环表里记着的被抛弃根（两类）引用的落点，这会儿在分配器里可发（不分配、不隔离、不扣）。正确的实现恒 0。
fn latent_abandoned_free(allocator: &singlefs_core::allocator::PoolAllocator) -> (u64, Vec<String>) {
    use singlefs_core::allocator::{RootReadOnOneRebuildPass, RootRingOccupant};
    let Some(occupancy) = allocator.root_ring_occupancy() else { return (0, Vec::new()) };
    let mut n = 0;
    let mut sample = Vec::new();
    for (slot, occupant) in occupancy.opus_occupants() {
        let (kind, placements) = match occupant {
            RootRingOccupant::AbandonedRoot { referenced_placements } => ("A", referenced_placements),
            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { root: RootReadOnOneRebuildPass::AbandonedRoot { referenced_placements }, .. } => ("S1A", referenced_placements),
            _ => continue,
        };
        for p in placements {
            let Some(map) = allocator.devices.iter().find(|m| m.device == p.device) else { continue };
            for k in 0..p.span {
                if map.is_free(SlotNumber(p.slot.0 + k)) {
                    n += 1;
                    if sample.len() < 4 {
                        sample.push(format!("{kind}@{}/{}:d{}s{}", slot.region, slot.slot, p.device.0, p.slot.0 + k));
                    }
                }
            }
        }
    }
    (n, sample)
}

fn run_plan_i1(sp: &StartPoint, k: u64, plan: &Plan, baseline: &Baseline, abandoned: &BTreeSet<(u32, u64)>, steps: usize, ops: Ops) -> (String, String) {
    singlefs_core::mount::OPUS_I1_TL.with(|c| *c.borrow_mut() = [0; 4]);
    let shared = shared_with(sp, plan);
    let run = mount_with(sp, &shared);
    let outcome = outcome_of(&run.result);
    let hook_ok = run.hooks.first().map_or(false, |h| h.values().all(|n| *n == k));
    let k_note = if plan.rules.keys().any(|(_, n)| *n == k) && !hook_ok { " k_mismatch" } else { "" };
    let s1_slots = match &run.result {
        Ok(m) => singlefs_core::mount::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(&m.allocator).len(),
        Err(_) => 0,
    };
    let (findings, summary, red) = match run.result {
        Ok(mounted) => {
            let isolated = format!("{:?}", mounted.output.isolated_slots_per_device);
            let (findings, summary) = continue_i1(sp, mounted, run.devices, baseline, abandoned, steps, ops);
            let red = !findings.is_empty();
            (findings, format!("isolated={isolated} {summary}"), red)
        }
        Err(_) => {
            let after = MemoryPool { devices: run.devices.iter().map(|(i, d)| (*i, d.inner.image.clone())).collect(), device_size_in_bytes: sp.image.device_size_in_bytes };
            let unchanged = after == sp.image;
            if unchanged {
                (Vec::new(), "image_unchanged=true".to_string(), false)
            } else {
                let (classes, violations) = violation_classes(&after);
                let broken = broken_roots(&after, baseline, 0);
                let red = classes.iter().any(|c| c != "I-3.1+") || !broken.is_empty();
                let short: Vec<String> = violations.iter().map(|v| v.chars().take(220).collect()).collect();
                (if red { vec![format!("after_err classes={classes:?} broken={broken:?} v={short:?}")] } else { Vec::new() }, format!("image_unchanged=false after_err_classes={classes:?}"), red)
            }
        }
    };
    let c = read_counters();
    let reached = if c[1] > 0 { "s1A-bad-raise" } else if c[0] > 0 { "t2-substituted" } else if c[3] > 0 { "s1-bad-ceiling" } else if s1_slots > 0 { "s1-only" } else { "no-s1" };
    let line = format!(
        "OPUSI1-RUN sp={} arm={:?} ops={ops:?} plan={} c554={} outcome={outcome} s1_slots={s1_slots} reached={reached} counters={c:?}{k_note} red={red} {summary} findings={findings:?}",
        sp.name, arm(), plan.id, plan.c554
    );
    let class = plan.id.split('@').next().unwrap_or("").to_string();
    let key = format!("sp={} arm={:?} ops={ops:?} {class} outcome={outcome} reached={reached} red={red}{k_note}", sp.name, arm());
    (line, key)
}

fn s1grid(start_points: &[&str]) {
    let steps = env_usize("OPUS_STEPS", 30);
    let workers = env_usize("OPUS_WORKERS", 1);
    let limit = env_usize("OPUS_PLAN_LIMIT", usize::MAX);
    let ops = ops_from_env();
    for name in start_points {
        let t0 = std::time::Instant::now();
        let sp = start_point_named(name);
        let k = rebuild_pass_read_number(&sp);
        let truth = truth_of(&sp.image);
        let baseline = baseline_of(&sp.image);
        let abandoned: BTreeSet<(u32, u64)> = ring_roots(&sp.image)
            .into_iter()
            .filter(|(slot, _)| truth.slots.iter().any(|(s, _, t)| s == slot && matches!(t, SlotTruth::Abandoned(_))))
            .map(|(_, r)| (r.instance.0, r.checkpoint_txg.0))
            .collect();
        println!("OPUSI1-SP sp={} arm={:?} k={k} abandoned={abandoned:?} truth={} build_s={:.1}", sp.name, arm(), truth_summary(&truth), t0.elapsed().as_secs_f64());
        let mut plans = s1_plans(&truth, k);
        plans.truncate(limit);
        let t1 = std::time::Instant::now();
        let next = std::sync::atomic::AtomicUsize::new(0);
        let results: std::sync::Mutex<Vec<(usize, String, String)>> = std::sync::Mutex::new(Vec::new());
        std::thread::scope(|scope| {
            for _ in 0..workers.max(1) {
                scope.spawn(|| loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(plan) = plans.get(i) else { break };
                    let (line, key) = run_plan_i1(&sp, k, plan, &baseline, &abandoned, steps, ops);
                    results.lock().unwrap().push((i, line, key));
                });
            }
        });
        let mut results = results.into_inner().unwrap();
        results.sort_by_key(|(i, _, _)| *i);
        let mut summary: BTreeMap<String, u64> = BTreeMap::new();
        for (_, line, key) in &results {
            println!("{line}");
            *summary.entry(key.clone()).or_insert(0) += 1;
        }
        for (key, count) in &summary {
            println!("OPUSI1-SUM {key} n={count}");
        }
        println!("OPUSI1-TIME sp={} plans={} steps={steps} workers={workers} run_s={:.1}", sp.name, plans.len(), t1.elapsed().as_secs_f64());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let list = |i: usize| -> Vec<String> {
        match args.get(i).map(String::as_str) {
            None | Some("all") => ALL_START_POINTS.iter().map(|s| s.to_string()).collect(),
            Some(names) => names.split(',').map(str::to_string).collect(),
        }
    };
    match args.get(1).map(String::as_str) {
        Some("grid") => {
            let names = list(2);
            grid(&names.iter().map(String::as_str).collect::<Vec<_>>());
        }
        Some("cells") => {
            let names = list(2);
            cells(&names.iter().map(String::as_str).collect::<Vec<_>>());
        }
        Some("l0") => l0(&args[2], &args[3]),
        Some("s1grid") => {
            let names = list(2);
            s1grid(&names.iter().map(String::as_str).collect::<Vec<_>>());
        }
        Some("k2young") => {
            k2young(HistoryDeviceWidth::FourGibibytes);
            k2young(HistoryDeviceWidth::UnitAreaOf384Slots);
        }
        _ => eprintln!("用法：grid|cells [起点,…|all] / l0 <起点> <计划> / k2young"),
    }
}
