//! m3-unlink-code-r3 攻方原型（Opus 腿）：N1 丙1（卸载那一串回收的槽不扣）在崩溃加丢盘上的交集与池级 checker 判定。
//! 只在冻结副本的拷贝上编、跑；同一份源码在「改之前那一版」（mount.rs 那一行改回 HeldUntilFloorTakesEffect）的拷贝上再编一次对照。
//! 起点（用户决定的那几步放开扫）由环境变量给；崩溃状态取层 0 的枚举域（checker 档 crash.rs 的
//! enumerate_layer0_selecting_versions_observing_each_state，每段都展开），每个状态再按「两块盘都在 / 丢盘 0 / 丢盘 1」三种各判一遍。
#![allow(dead_code, unused_imports, clippy::all)]

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use common_admission::*;
use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_space_admission, mount_writable_with_test_only_switches_and_deletion_switches,
    unmount, MountError, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
    RootRingSlotReading,
};
use singlefs_core::root_ring::slot_offset;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes_and_entries, MemoryPool,
    PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordedPublishEntry, RecordingBlockDevice, SharedStream};

fn env_list(name: &str, default: &str) -> Vec<u64> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().parse::<u64>().expect("十进制"))
        .collect()
}
fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().map_or(default, |v| v.parse().expect("十进制"))
}
fn env_str(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

/// 删除批大小 1（抄自 `admission_raises_the_floor_before_refusing.rs` 的同名函数）。
fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1")),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 一个起点：一个带可写会话的池（卸载就在这个会话上做）与切段要的几何。
struct Start {
    label: String,
    pool: PoolUnderTest<SparseBlockDevice>,
    geometry: FixedGeometry,
}

/// 窄盘起点（抄自 `pool_crashed_between_deletion_batches` 与 `mount_after_the_crash_with_batches_of_one`，四样参数放开）：
/// 准入关着的不补足窄盘上覆盖写 k 次、顺序写长到 u 个单元、意图之后 b 批（批大小 1）就崩，重开可写（准入判着、批大小 1）。
fn narrow_start(unit_area_slots: u64, overwrites: u64, units: u64, batches: u64) -> Result<Start, String> {
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(width, unit_area_slots);
    for i in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("overwrite{i}:{e:?}"))?;
    }
    if units > 1 {
        pool.sequential_write((units as usize - 1) * data_unit_payload_capacity() + 100)
            .map_err(|e| format!("seqwrite:{e:?}"))?;
    }
    {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().expect("会话");
        let mut current = session.current.version_of_trees().expect("树").clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), session.instance)
            .map_err(|e| format!("intent:{e:?}"))?;
        for b in 0..batches {
            current = publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER),
                deletion_switches_with_batches_of_one().batch_size, session.instance)
                .map_err(|e| format!("batch{b}:{e:?}"))?;
        }
    }
    pool.session = None;
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters, &mut pool.devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    ).map_err(|e| format!("mount:{e:?}"))?;
    let (output, session) = MountedSession::of_the_mount(mounted);
    let continuation = format!("{:?}", output.deletion_queue_continuation_outcome);
    let continuation = continuation.split(|c: char| c == '(' || c == '{' || c == ' ').next().unwrap_or("").to_string();
    pool.session = Some(session);
    Ok(Start {
        label: format!("narrow U={unit_area_slots} k={overwrites} u={units} b={batches} cont={continuation}"),
        pool,
        geometry: width.fixed_geometry(),
    })
}

/// 普通起点：随机历史那几档盘宽之一，第一个文件之后覆盖写 k 次、崩了再挂 m 次（每次之后再覆盖写 k2 次）。
fn plain_start(width_code: u64, overwrites: u64, remounts: u64, overwrites_after_each: u64) -> Result<Start, String> {
    let width = width_of(width_code);
    let mut pool = start_plain(width);
    for i in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("overwrite{i}:{e:?}"))?;
    }
    for m in 0..remounts {
        pool.crash_and_mount_writable().map_err(|e| format!("remount{m}:{e:?}"))?;
        for i in 0..overwrites_after_each {
            pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("remount{m}-overwrite{i}:{e:?}"))?;
        }
    }
    Ok(Start {
        label: format!("plain W={width_code} k={overwrites} m={remounts} k2={overwrites_after_each}"),
        pool,
        geometry: width.fixed_geometry(),
    })
}

fn width_of(code: u64) -> HistoryDeviceWidth {
    match code {
        4 => HistoryDeviceWidth::FourGibibytes,
        384 => HistoryDeviceWidth::UnitAreaOf384Slots,
        256 => HistoryDeviceWidth::UnitAreaOf256Slots,
        240 => HistoryDeviceWidth::UnitAreaOf240Slots,
        other => panic!("盘宽代号 {other}"),
    }
}

/// 卸载—再挂循环：在起点上不注错地卸载一次（整串做完），再崩了再挂可写（准入判着），再覆盖写 k3 次；交回新的起点。
fn after_a_full_unmount_and_remount(mut start: Start, overwrites_after: u64) -> Result<Start, String> {
    {
        let PoolUnderTest { session, devices, parameters, .. } = &mut start.pool;
        let session = session.as_mut().expect("会话");
        let mut allocator = session.allocator.clone();
        let mut current = session.current.clone();
        unmount(parameters, devices, &mut allocator, &mut current, session.shadow_ledger)
            .map_err(|e| format!("full-unmount:{e:?}"))?;
    }
    start.pool.crash_and_mount_writable().map_err(|e| format!("remount-after-unmount:{e:?}"))?;
    for i in 0..overwrites_after {
        start.pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("after-unmount-overwrite{i}:{e:?}"))?;
    }
    start.label = format!("{} | unmount+remount k3={overwrites_after}", start.label);
    Ok(start)
}

/// 在起点那份盘面的拷贝上录一次卸载（不注错）：交回结局、带内容的录制流与入口段。
fn recorded_unmount(start: &Start) -> (Result<Unmounted, MountError>, Vec<singlefs_harness::RetainedOperation>, Vec<singlefs_harness::RecordedEntrySpan>) {
    let image = start.pool.image();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&image)
        .into_iter()
        .map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone())))
        .collect();
    let session = start.pool.session.as_ref().expect("会话");
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    let shadow = session.shadow_ledger;
    let parameters = start.pool.parameters.clone();
    let result = stream.record_entry(RecordedPublishEntry::Unmount, || {
        unmount(&parameters, &mut devices, &mut allocator, &mut current, shadow)
    });
    (result, stream.retained_operations(), stream.entry_spans())
}

/// 镜像去掉一块盘（`lost` 为 None 时原样）。
fn without_device(image: &MemoryPool, lost: Option<u32>) -> MemoryPool {
    MemoryPool {
        devices: image.devices.iter().filter(|(id, _)| Some(id.0) != lost).map(|(id, d)| (*id, d.clone())).collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

/// 读的时候记下读了哪里的 checker 读者；候选单元槽交空（扫全单元区的那几条不读，只留沿树走读的读）。
struct Tracing<'a> {
    inner: &'a MemoryPool,
    reads: RefCell<Vec<(u32, u64, usize)>>,
}
impl ImageReader for Tracing<'_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.inner)
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        ImageReader::device_bytes(self.inner, device)
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        self.reads.borrow_mut().push((device, offset, length));
        ImageReader::read(self.inner, device, offset, length)
    }
    fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        self.inner.candidate_journal_slots(device)
    }
}

/// 把镜像里除 txg = `keep` 之外的每条自证过的根槽抹成 0（这份镜像上 checker 的候选集就只剩那一条根）。
fn with_only_root(image: &MemoryPool, keep: u64) -> MemoryPool {
    let mut copy = image.clone();
    let Ok(sc) = choose_system_configuration(&copy) else { return copy };
    let immutable = sc.immutable;
    let width = immutable.sizes.physical_block_size as usize;
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        let device = immutable.region_devices[ring_slot.region as usize];
        if !copy.devices.contains_key(&device) {
            continue;
        }
        if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(
            &copy, &immutable.region_devices, &immutable.sizes, &immutable.filesystem_identifier, ring_slot,
        ) {
            if root.checkpoint_txg.0 != keep {
                let offset = slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing);
                copy.devices.get_mut(&device).expect("盘").write(offset, &vec![0u8; width]);
            }
        }
    }
    copy
}

/// 根 txg = `root_txg` 引用的单元槽（盘, 槽号）：只留这一条根的镜像上，checker 沿树走读读过的单元区槽（读法一，不经实现的账）。
fn slots_read_by_the_checker_walking_only(image: &MemoryPool, root_txg: u64, unit_area_start_slot: u64) -> (BTreeSet<(u32, u64)>, Vec<String>) {
    let only = with_only_root(image, root_txg);
    let tracing = Tracing { inner: &only, reads: RefCell::new(Vec::new()) };
    let verdicts = check_pool_image(&tracing);
    let mut slots = BTreeSet::new();
    for (device, offset, length) in tracing.reads.borrow().iter() {
        let first = offset / SLOT_BYTES;
        let last = (offset + *length as u64 - 1) / SLOT_BYTES;
        for slot in first..=last {
            if slot >= unit_area_start_slot {
                slots.insert((*device, slot));
            }
        }
    }
    (slots, violation_names(verdicts))
}

fn violation_names(verdicts: Vec<(&'static str, InvariantVerdict)>) -> Vec<String> {
    verdicts.into_iter().filter_map(|(name, v)| matches!(v, InvariantVerdict::Violated(_)).then(|| name.to_string())).collect()
}

fn checker_names(image: &MemoryPool) -> Vec<String> {
    violation_names(check_pool_image(image))
}

/// 一个崩溃状态里持久了的单元写罩住的（盘, 槽号）。
fn unit_slots_written(writes: &[RetainedWrite], persisted: &[bool]) -> BTreeSet<(u32, u64)> {
    let mut slots = BTreeSet::new();
    for (write, is) in writes.iter().zip(persisted) {
        if *is && write.kind == StepKind::UnitWrite {
            let first = write.offset.0 / SLOT_BYTES;
            let last = (write.offset.0 + write.length_in_bytes() - 1) / SLOT_BYTES;
            for slot in first..=last {
                slots.insert((write.device.0, slot));
            }
        }
    }
    slots
}

fn materialize(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is) in writes.iter().zip(persisted) {
        if *is {
            write.contents.apply_to(pool.devices.get_mut(&write.device).expect("盘"), write.offset);
        }
    }
    pool
}


/// 实现那一侧的读者，读的时候记下读了哪里（读法二：恢复加只读挂载加把每个文件整读一遍，实际读到的单元区槽）。
struct PoolTracing<'a> {
    inner: &'a MemoryPool,
    reads: RefCell<Vec<(u32, u64, usize)>>,
}
impl singlefs_core::recovery::PoolReader for PoolTracing<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        singlefs_core::recovery::PoolReader::device_identities(self.inner)
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        singlefs_core::recovery::PoolReader::device_size_in_bytes(self.inner, device)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        self.reads.borrow_mut().push((device.0, offset.0, length));
        singlefs_core::recovery::PoolReader::read(self.inner, device, offset, length)
    }
    fn journal_record_offsets_hint(&self, device: DeviceIdentity, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        singlefs_core::recovery::PoolReader::journal_record_offsets_hint(self.inner, device, ring_start, ring_bytes)
    }
}

/// 只读挂载、再把挂载态里每个活着的文件从头到尾读一遍：交回（走到的根 txg, 读出的全部字节的 FNV 哈希）或错。
fn read_everything(reader: &dyn singlefs_core::recovery::PoolReader) -> Result<(u64, u64), String> {
    let mounted = mount_read_only(reader).map_err(|e| format!("ro:{}", format!("{e:?}").chars().take(80).collect::<String>()))?;
    let mut bytes = Vec::new();
    for record in mounted.mounted.inode_records().to_vec() {
        if record.nlink == singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = mounted.mounted.open_file(reader, InodeNumber(record.inode))
            .map_err(|e| format!("open:{}", format!("{e:?}").chars().take(80).collect::<String>()))?;
        let out = file.read_at(reader, singlefs_core::address::FileOffsetInBytes(0), record.size)
            .map_err(|e| format!("read:{}", format!("{e:?}").chars().take(80).collect::<String>()))?;
        bytes.extend_from_slice(&record.inode.to_le_bytes());
        bytes.extend_from_slice(&out.bytes);
    }
    Ok((mounted.effective_root.checkpoint_txg.0, singlefs_harness::fnv1a_64(&bytes)))
}

/// 读法二：只留 txg = `root_txg` 那一条根的镜像上，恢复（看 journal）加只读挂载加整读每个文件实际读到的单元区槽。
fn slots_read_by_recovery_and_whole_reads(image: &MemoryPool, root_txg: u64, unit_area_start_slot: u64) -> (BTreeSet<(u32, u64)>, String) {
    let only = with_only_root(image, root_txg);
    let tracing = PoolTracing { inner: &only, reads: RefCell::new(Vec::new()) };
    let _ = recover(&tracing, JournalPolicy::Consult);
    let outcome = match read_everything(&tracing) {
        Ok((txg, hash)) => format!("ok txg={txg} hash={hash:016x}"),
        Err(e) => e,
    };
    let mut slots = BTreeSet::new();
    for (device, offset, length) in tracing.reads.borrow().iter() {
        let first = offset / SLOT_BYTES;
        let last = (offset + *length as u64 - 1) / SLOT_BYTES;
        for slot in first..=last {
            if slot >= unit_area_start_slot {
                slots.insert((*device, slot));
            }
        }
    }
    (slots, outcome)
}

/// 一个槽上单元头自述的诞生代号（码 2 节点、码 1 数据单元、码 3 打包单元依次试）。
fn birth_of_the_unit_at(image: &MemoryPool, device: u32, slot: u64) -> Option<u64> {
    let node = singlefs_core::recovery::PoolReader::read(image, DeviceIdentity(device), DeviceOffsetInBytes(slot * SLOT_BYTES), 16384)?;
    if let Ok(header) = singlefs_core::unit::parse_index_node(&node) {
        return Some(header.birth_txg.0);
    }
    let unit = singlefs_core::recovery::PoolReader::read(image, DeviceIdentity(device), DeviceOffsetInBytes(slot * SLOT_BYTES), 32768)?;
    if let Ok(header) = singlefs_core::unit::parse_data_unit(&unit) {
        return Some(header.birth_txg.0);
    }
    if let Ok(header) = singlefs_core::unit::parse_packed_unit(&unit) {
        return Some(header.birth_txg.0);
    }
    None
}

/// 对照：R 那一版引用的全部槽（账加单元头）：仍分配、诞生 ≤ R，或已释放、释放代 > R 而诞生 ≤ R。
fn every_slot_referenced_by_records_and_headers(records: &[singlefs_core::allocator::AllocationRecord], pre: &MemoryPool, root_txg: u64, lost: Option<u32>) -> BTreeSet<(u32, u64)> {
    let mut out = BTreeSet::new();
    for record in records {
        if Some(record.device.0) == lost {
            continue;
        }
        let alive_at_r = !record.is_released || record.generation.0 > root_txg;
        if alive_at_r && birth_of_the_unit_at(pre, record.device.0, record.slot.0).is_some_and(|birth| birth <= root_txg) {
            for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
                out.insert((record.device.0, slot));
            }
        }
    }
    out
}

/// 一个槽上的单元是什么（按头解）：节点给树号与层级，数据单元给 inode 与单元号，打包单元给出生树。
fn unit_kind_at(image: &MemoryPool, device: u32, slot: u64) -> String {
    let read = |len| singlefs_core::recovery::PoolReader::read(image, DeviceIdentity(device), DeviceOffsetInBytes(slot * SLOT_BYTES), len);
    if let Some(node) = read(16384) {
        if let Ok(h) = singlefs_core::unit::parse_index_node(&node) {
            return format!("node(tree={:?},level={},birth={})", h.tree, h.level, h.birth_txg.0);
        }
    }
    if let Some(unit) = read(32768) {
        if let Ok(h) = singlefs_core::unit::parse_data_unit(&unit) {
            return format!("data({:?},birth={})", h.identity, h.birth_txg.0);
        }
        if let Ok(h) = singlefs_core::unit::parse_packed_unit(&unit) {
            return format!("packed({:?},birth={})", h.identity, h.birth_txg.0);
        }
    }
    "unparsed".to_string()
}

/// 读法三（实现的账加单元头）：卸载之前分配器的记录里，起点是这个槽、已释放、释放代 > R，而卸载之前盘上那个槽的单元头诞生代号 ≤ R ——
/// 这个槽上的单元在 R 那一版还活着，R 引用它。
fn referenced_by_records_and_headers(records: &[singlefs_core::allocator::AllocationRecord], pre: &MemoryPool, root_txg: u64, slot: (u32, u64)) -> bool {
    let Some(record) = records.iter().find(|r| r.device.0 == slot.0 && r.slot.0 <= slot.1 && slot.1 < r.slot.0 + u64::from(r.span_slots)) else {
        return false;
    };
    if !(record.is_released && record.generation.0 > root_txg) {
        return false;
    }
    birth_of_the_unit_at(pre, slot.0, record.slot.0).is_some_and(|birth| birth <= root_txg)
}

/// 一份崩溃镜像按一种丢盘判出来的东西。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SurvivorVerdict {
    lost: String,
    chosen_txg: Option<u64>,
    floor_in_system_configuration: Option<u64>,
    /// 这一状态里持久了的单元写 ∩ 读法三（账加单元头）认的 R 引用的槽。
    intersection_by_records: usize,
    /// 同上 ∩ 读法二（恢复加整读实际读到的槽）。
    intersection_by_reads: usize,
    /// 同上 ∩ 读法一（只留 R 的镜像上 checker 读过的槽；它还读已释放记录罩的槽，偏大，只作参考）。
    intersection_by_checker_reads: usize,
    /// 整读结局：same（与卸载之前同一块盘面整读出来的相同）/ differs / 错。
    whole_read: String,
    checker: String,
}

struct Judge<'a> {
    pre: &'a MemoryPool,
    records: Vec<singlefs_core::allocator::AllocationRecord>,
    unit_area_start_slot: u64,
    newest_txg_before_the_unmount: u64,
    cache: BTreeMap<(Option<u32>, u64), (BTreeSet<(u32, u64)>, BTreeSet<(u32, u64)>)>,
    whole_read_before: BTreeMap<Option<u32>, String>,
    union_by_records: BTreeMap<(String, u64), BTreeSet<(u32, u64)>>,
    union_by_reads: BTreeMap<(String, u64), BTreeSet<(u32, u64)>>,
    run_checker: bool,
}

fn whole_read_text(image: &MemoryPool) -> String {
    match read_everything(image) {
        Ok((txg, hash)) => format!("txg={txg},hash={hash:016x}"),
        Err(e) => e,
    }
}

impl Judge<'_> {
    fn judge(&mut self, crashed: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool], lost: Option<u32>) -> SurvivorVerdict {
        let survivor = without_device(crashed, lost);
        let lost_name = lost.map_or("none".to_string(), |d| format!("dev{d}"));
        let floor = choose_system_configuration(&survivor).ok().map(|sc| sc.quantities.rollback_floor.0);
        let report = recover(&survivor, JournalPolicy::Consult);
        let chosen = report.effective_root.map(|(_, txg)| txg.0);
        let written: BTreeSet<(u32, u64)> = unit_slots_written(writes, persisted)
            .into_iter()
            .filter(|(device, _)| Some(*device) != lost)
            .collect();
        let (mut by_records, mut by_reads, mut by_checker) = (0, 0, 0);
        if let Some(r) = chosen {
            if r <= self.newest_txg_before_the_unmount {
                let key = (lost, r);
                if !self.cache.contains_key(&key) {
                    let pre_survivor = without_device(self.pre, lost);
                    let (checker_slots, _) = slots_read_by_the_checker_walking_only(&pre_survivor, r, self.unit_area_start_slot);
                    let (read_slots, _) = slots_read_by_recovery_and_whole_reads(&pre_survivor, r, self.unit_area_start_slot);
                    self.cache.insert(key, (checker_slots, read_slots));
                }
                let (checker_slots, read_slots) = &self.cache[&key];
                let hit_records: BTreeSet<(u32, u64)> = written
                    .iter()
                    .copied()
                    .filter(|slot| referenced_by_records_and_headers(&self.records, self.pre, r, *slot))
                    .collect();
                let hit_reads: BTreeSet<(u32, u64)> = written.intersection(read_slots).copied().collect();
                by_checker = written.intersection(checker_slots).count();
                by_records = hit_records.len();
                by_reads = hit_reads.len();
                if !hit_records.is_empty() {
                    self.union_by_records.entry((lost_name.clone(), r)).or_default().extend(hit_records);
                }
                if !hit_reads.is_empty() {
                    self.union_by_reads.entry((lost_name.clone(), r)).or_default().extend(hit_reads);
                }
            }
        }
        if !self.whole_read_before.contains_key(&lost) {
            let text = whole_read_text(&without_device(self.pre, lost));
            self.whole_read_before.insert(lost, text);
        }
        let now = whole_read_text(&survivor);
        let whole_read = if now == self.whole_read_before[&lost] { "same".to_string() } else { format!("differs[{now}]") };
        let checker = if self.run_checker || by_records > 0 || by_reads > 0 { checker_names(&survivor).join("+") } else { "-".to_string() };
        SurvivorVerdict {
            lost: lost_name,
            chosen_txg: chosen,
            floor_in_system_configuration: floor,
            intersection_by_records: by_records,
            intersection_by_reads: by_reads,
            intersection_by_checker_reads: by_checker,
            whole_read,
            checker,
        }
    }
}

/// 两块盘都在的崩溃镜像上再可写挂载一次（准入判着、批大小 1，与起点同一种开关），交回结局与挂载之后池级 checker 判红的那几条。
fn writable_mount_after(crashed: &MemoryPool, start: &Start) -> String {
    let mut devices = plain_devices_on(crashed);
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &start.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    );
    let outcome = match &mounted {
        Ok(m) => format!("ok:{}", format!("{:?}", m.output.deletion_queue_continuation_outcome).chars().take(40).collect::<String>()),
        Err(e) => format!("err:{}", format!("{e:?}").chars().take(100).collect::<String>()),
    };
    drop(mounted);
    let image = MemoryPool { devices: devices.iter().map(|(id, d)| (*id, d.image.clone())).collect(), device_size_in_bytes: crashed.device_size_in_bytes };
    format!("{outcome}|after_checker={}", checker_names(&image).join("+"))
}

fn verdict_text(v: &SurvivorVerdict) -> String {
    format!("lost={} chosen={:?} sc_floor={:?} inter_records={} inter_reads={} inter_checker_reads={} whole_read={} checker={}",
        v.lost, v.chosen_txg, v.floor_in_system_configuration, v.intersection_by_records, v.intersection_by_reads,
        v.intersection_by_checker_reads, v.whole_read, v.checker)
}

fn run_n1_on(start: Start) {
    let t0 = Instant::now();
    let session = start.pool.session.as_ref().expect("会话");
    let current_txg = session.current.root().checkpoint_txg.0;
    let floor_before = session.current.root().rollback_floor.0;
    let unit_area_start_slot = session.allocator.devices[0].unit_area_start().slot().0;
    let (result, operations, spans) = recorded_unmount(&start);
    let outcome = match &result {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => format!(
            "Raised publishes={} reclaimed={} sysconf_writes={}",
            raised.publishes.len(), raised.reclaimed.len(),
            raised.system_configuration_writes_before_the_first_publish.total().write_calls
        ),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "NothingWritten".to_string(),
        Err(error) => {
            let text = format!("{error:?}");
            format!("Err {}", text.chars().take(160).collect::<String>())
        }
    };
    let (writes, segments, _indexes) = writes_and_segments_with_stream_indexes_and_entries(&operations, &spans, &start.geometry);
    let states = closed_form_state_count(&segments);
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!("START {} | current_txg={current_txg} floor_before={floor_before} unmount={outcome} writes={} segments={sizes:?} closed_form_states={states}", start.label, writes.len());
    let pre = start.pool.image();
    let mut judge = Judge {
        pre: &pre,
        records: session.allocator.records().to_vec(),
        unit_area_start_slot,
        newest_txg_before_the_unmount: current_txg,
        cache: BTreeMap::new(),
        whole_read_before: BTreeMap::new(),
        union_by_records: BTreeMap::new(),
        union_by_reads: BTreeMap::new(),
        run_checker: env_u64("CHECK_ALL", 1) == 1,
    };
    // 前缀刀：录制流里前 n 个写持久（层 0 枚举域里的一部分），逐个 n 报三种丢盘的判定。
    let mut roots_before = 0;
    for n in 0..=writes.len() {
        let persisted: Vec<bool> = (0..writes.len()).map(|i| i < n).collect();
        let crashed = materialize(&pre, &writes, &persisted);
        let next_kind = writes.get(n).map_or("end", |w| w.kind.name());
        for lost in [None, Some(0), Some(1)] {
            let v = judge.judge(&crashed, &writes, &persisted, lost);
            let remount = if lost.is_none() && env_u64("REMOUNT_CUTS", 1) == 1 { writable_mount_after(&crashed, &start) } else { "-".to_string() };
            println!("CUT n={n} roots_landed={roots_before} next={next_kind} {} remount={remount}", verdict_text(&v));
        }
        if env_list("DETAIL_CUTS", "").contains(&(n as u64)) {
            for lost in [Some(0u32), Some(1u32)] {
                let survivor = without_device(&crashed, lost);
                for (name, verdict) in check_pool_image(&survivor) {
                    if let InvariantVerdict::Violated(detail) = verdict {
                        println!("DETAIL n={n} lost={lost:?} {name}: {}", detail.chars().take(300).collect::<String>());
                    }
                }
            }
        }
        if n < writes.len() && writes[n].kind == StepKind::RootRecordFua {
            roots_before += 1;
        }
    }
    let max_states = env_u64("MAX_STATES", 1_000_000);
    if env_u64("ENUM", 0) == 1 && !writes.is_empty() {
        if states > max_states {
            println!("ENUM_SKIPPED closed_form_states={states} > {max_states}");
        } else if let Some(judged_root_index) = writes.iter().position(|w| w.kind == StepKind::RootRecordFua) {
            let versions = start.pool.published_versions();
            let mut histogram: BTreeMap<SurvivorVerdict, u64> = BTreeMap::new();
            let mut observed = 0u64;
            let tally = enumerate_layer0_selecting_versions_observing_each_state(
                &pre, &writes, &segments, judged_root_index, &versions, &|_, _| true,
                &mut |image, _report| {
                    observed += 1;
                    let crashed = materialize(image.base, image.writes, &image.persisted);
                    for lost in [None, Some(0), Some(1)] {
                        let v = judge.judge(&crashed, image.writes, &image.persisted, lost);
                        *histogram.entry(v).or_default() += 1;
                    }
                },
            );
            println!("ENUM states={} observed={observed} violations={} failed_states={} checker_violated={:?}", tally.states, tally.violations, tally.failed_states, tally.checker_violated_states);
            for (v, count) in &histogram {
                println!("ENUMV count={count} {}", verdict_text(v));
            }
        }
    }
    for ((lost, r), slots) in &judge.union_by_records {
        let kinds: Vec<String> = slots.iter().map(|(d, sl)| {
            let rec = judge.records.iter().find(|x| x.device.0 == *d && x.slot.0 <= *sl && *sl < x.slot.0 + u64::from(x.span_slots));
            format!("({d},{sl})={} released_gen={:?}", unit_kind_at(&pre, *d, *sl), rec.map(|x| (x.is_released, x.generation.0)))
        }).collect();
        println!("INTERSECTION_BY_RECORDS lost={lost} root_txg={r} count={} slots={:?}", slots.len(), kinds);
    }
    for ((lost, r), slots) in &judge.union_by_reads {
        let referenced = judge.cache.get(&(if lost == "none" { None } else if lost == "dev0" { Some(0) } else { Some(1) }, *r)).map_or(0, |c| c.1.len());
        println!("INTERSECTION_BY_READS lost={lost} root_txg={r} slots_read_for_root={referenced} count={} slots={:?}", slots.len(), slots);
    }
    for ((lost, r), (checker_slots, read_slots)) in &judge.cache {
        let by_records = every_slot_referenced_by_records_and_headers(&judge.records, &pre, *r, *lost);
        println!("CONTROL lost={lost:?} root_txg={r} by_records={} read_slots={} read_slots_within_by_records={} checker_slots={} checker_slots_within_by_records={}",
            by_records.len(), read_slots.len(), read_slots.intersection(&by_records).count(), checker_slots.len(), checker_slots.intersection(&by_records).count());
    }
    for (lost, text) in &judge.whole_read_before {
        println!("WHOLE_READ_BEFORE lost={lost:?} {text}");
    }
    println!("DONE {} elapsed={:.1}s", start.label, t0.elapsed().as_secs_f64());
}


// ---------------------------------------------------------------------------------------------
// N2：挂载里每一个写调用逐个注错，比「设备一层落下的写 − 取号两次」与错里交的写账合计、「落下的根槽写」与错里的已落盘份数。

use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement, FaultSchedule,
    InjectedFault, SharedFaultPlan,
};
use singlefs_core::write_accounting::WritesByStructureKind;

type FaultyDevice = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;

/// 抄自 `admission_raises_the_floor_before_refusing.rs` 的 `fault_injecting_recorded_devices_on`。
fn faulty_devices_on(image: &MemoryPool, schedule: Option<FaultSchedule>) -> (Vec<(DeviceIdentity, FaultyDevice)>, SharedStream) {
    let geometry = HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry();
    let plan = match schedule {
        Some(schedule) => SharedFaultPlan::armed(geometry, schedule),
        None => SharedFaultPlan::unarmed(geometry),
    };
    let stream = SharedStream::new();
    let devices = plain_devices_on(image)
        .into_iter()
        .map(|(identity, device)| {
            (identity, FaultInjectingBlockDevice::new(identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()), plan.clone()))
        })
        .collect();
    (devices, stream)
}

fn failing_writes(ordinal: u64, every_later: bool) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::AnyOffset,
        counting: FaultCounting::AcrossThePool,
        occurrence: if every_later { FaultOccurrence::EveryMatchingCallFromTheNthOnward(ordinal) } else { FaultOccurrence::TheNthMatchingCall(ordinal) },
    }
}

fn landed(stream: &SharedStream) -> (u64, u64) {
    let ops = stream.operations();
    let writes = ops.iter().filter(|o| matches!(o.kind, singlefs_harness::RecordedOperationKind::Write | singlefs_harness::RecordedOperationKind::WriteForceUnitAccess | singlefs_harness::RecordedOperationKind::WriteZeroes)).count() as u64;
    let roots = ops.iter().filter(|o| o.kind == singlefs_harness::RecordedOperationKind::WriteForceUnitAccess).count() as u64;
    (writes, roots)
}

fn sum(ledgers: &[WritesByStructureKind]) -> u64 {
    ledgers.iter().map(|l| l.total().write_calls).sum()
}

/// 一个挂载错交得出的写账：(变体名, 写账合计的一种或两种读法, 已落盘份数)。
fn ledger_of(error: &MountError) -> Option<(String, Vec<(&'static str, u64)>, u64)> {
    fn of_cause(cause: &MountError) -> Option<(String, u64, u64)> {
        match cause {
            MountError::RaiseFloorSequencePublishFailed(p) => Some(("RaiseFloorSequencePublishFailed".into(),
                p.writes_before_the_first_publish.total().write_calls + sum(&p.writes_of_persisted_publishes) + sum(&p.writes_of_failed_publishes),
                p.writes_of_persisted_publishes.len() as u64)),
            MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(x) => Some(("RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot".into(), x.writes.total().write_calls, 0)),
            _ => None,
        }
    }
    match error {
        MountError::Publish(f) => Some(("Publish".into(), vec![("contract", f.writes_before_the_first_publish.total().write_calls + sum(&f.writes_of_persisted_publishes) + sum(&f.writes_of_failed_publishes))], f.writes_of_persisted_publishes.len() as u64)),
        MountError::FloorRaiseFailedAfterTheMountsPublishes(f) => {
            let (cause_name, cause_ledger, cause_persisted) = of_cause(&f.cause).unwrap_or((format!("other:{:?}", f.cause).chars().take(40).collect(), 0, 0));
            let sysconf: u64 = f.floor_raises.iter().map(|r| r.system_configuration_writes_before_the_first_publish.total().write_calls).sum();
            let raise_publishes: u64 = f.floor_raises.iter().flat_map(|r| r.publishes.iter()).map(|p| p.writes.total().write_calls).sum();
            let raise_publish_count: u64 = f.floor_raises.iter().map(|r| r.publishes.len() as u64).sum();
            let base = sum(&f.writes_of_persisted_publishes) + sysconf + cause_ledger;
            // 份数也有两种读法：续做那一处推成的那几串的空发布已在 writes_of_persisted_publishes 里，写行之后那一处不在、要加上。
            let persisted_continuation = f.writes_of_persisted_publishes.len() as u64 + cause_persisted;
            Some((format!("FloorRaiseFailed[{cause_name}]"), vec![("continuation_reading", base), ("row_reading", base + raise_publishes)],
                persisted_continuation * 1000 + persisted_continuation + raise_publish_count))
        }
        MountError::RaiseFloorSequencePublishFailed(_) | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_) => {
            let (name, l, p) = of_cause(error).expect("上面两种");
            Some((format!("bare:{name}"), vec![("own", l)], p))
        }
        _ => None,
    }
}

fn run_n2_on(label: &str, pool: &MemoryPool, parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters) {
    let mount = |devices: &mut Vec<(DeviceIdentity, FaultyDevice)>| {
        mount_writable_with_test_only_switches_and_deletion_switches(parameters, devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, deletion_switches_with_batches_of_one())
    };
    let (mut clean_devices, clean_stream) = faulty_devices_on(pool, None);
    let clean = mount(&mut clean_devices);
    let (total, total_roots) = landed(&clean_stream);
    let summary = match &clean {
        Ok(m) => format!("ok admission={} cont={} cont_raises={} cont_steps={}",
            format!("{:?}", m.output.space_admission).split(|c: char| c == '{' || c == '(' || c == ' ').next().unwrap_or(""),
            format!("{:?}", m.output.deletion_queue_continuation_outcome).split(|c: char| c == '{' || c == '(' || c == ' ').next().unwrap_or(""),
            m.output.floor_raises_pushed_by_the_continuation.len(), m.output.deletion_queue_continuation.len()),
        Err(e) => format!("err {}", format!("{e:?}").chars().take(80).collect::<String>()),
    };
    println!("N2START {label} | clean={summary} writes={total} roots={total_roots}");
    let geometry = HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry();
    let clean_kinds: Vec<&'static str> = clean_stream.operations().iter()
        .filter(|o| o.kind != singlefs_harness::RecordedOperationKind::Barrier)
        .map(|o| geometry.classify(o).name())
        .collect();
    if clean.is_err() { return; }
    drop(clean);
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    for every_later in [true, false] {
        for ordinal in 1..=total {
            let (mut devices, stream) = faulty_devices_on(pool, Some(failing_writes(ordinal, every_later)));
            let result = mount(&mut devices);
            let (writes, roots) = landed(&stream);
            let mode = if every_later { "power" } else { "once" };
            let key = match &result {
                Ok(_) => "Ok".to_string(),
                Err(error) => match ledger_of(error) {
                    Some((name, readings, persisted)) => {
                        let expected = writes.saturating_sub(2);
                        let matching: Vec<&str> = readings.iter().filter(|(_, l)| *l == expected).map(|(r, _)| *r).collect();
                        // FloorRaiseFailed 的份数编成「续做读法 × 1000 + 写行读法」，两种读法有一种对上就算对上。
                        let persisted_ok = if name.starts_with("FloorRaiseFailed") { persisted / 1000 == roots || persisted % 1000 == roots } else { persisted == roots };
                        let failing_kind = clean_kinds.get(ordinal as usize - 1).copied().unwrap_or("?");
                        if matching.is_empty() || !persisted_ok {
                            println!("N2MISMATCH {label} mode={mode} n={ordinal} failing_write_kind={failing_kind} variant={name} landed={writes} landed_minus_acq={expected} readings={readings:?} persisted={persisted} landed_roots={roots}");
                        }
                        format!("{name} match={} persisted_ok={persisted_ok}{}", if matching.is_empty() { "none".to_string() } else { matching.join("|") },
                            if persisted_ok { String::new() } else { format!(" failing_write_kind={failing_kind} roots_minus_persisted={}", roots as i64 - (if name.starts_with("FloorRaiseFailed") { (persisted % 1000) as i64 } else { persisted as i64 })) })
                    }
                    None => {
                        let name = format!("{error:?}").split(|c: char| c == '{' || c == '(' || c == ' ').next().unwrap_or("").to_string();
                        if writes > 2 { println!("N2UNLEDGERED {label} mode={mode} n={ordinal} variant={name} landed={writes} landed_roots={roots}"); }
                        format!("noledger:{name} landed_gt2={}", writes > 2)
                    }
                },
            };
            *tally.entry(format!("{mode} {key}")).or_default() += 1;
        }
    }
    for (key, count) in &tally {
        println!("N2TALLY {label} {key} count={count}");
    }
}

/// 窄盘起点停在挂载之前（`narrow_start` 去掉最后那次挂载）。
fn narrow_crashed(unit_area_slots: u64, overwrites: u64, units: u64, batches: u64) -> Result<PoolUnderTest<SparseBlockDevice>, String> {
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let mut pool = start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(width, unit_area_slots);
    for i in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("overwrite{i}:{e:?}"))?;
    }
    if units > 1 {
        pool.sequential_write((units as usize - 1) * data_unit_payload_capacity() + 100).map_err(|e| format!("seqwrite:{e:?}"))?;
    }
    {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().expect("会话");
        let mut current = session.current.version_of_trees().expect("树").clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), session.instance).map_err(|e| format!("intent:{e:?}"))?;
        for b in 0..batches {
            current = publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), deletion_switches_with_batches_of_one().batch_size, session.instance).map_err(|e| format!("batch{b}:{e:?}"))?;
        }
    }
    pool.session = None;
    Ok(pool)
}

fn main_n2() {
    if env_str("N2_FAMILY", "narrow") == "remount" {
        // 只崩了再挂的窄盘（`crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck` 的造法）：
        // 第一个文件之后连着崩了再挂 r 次，判第 r + 1 次挂载。
        for &w in &env_list("WIDTHS", "240") {
            for &r in &env_list("REMOUNTS", "8") {
                let mut pool = start_plain(width_of(w));
                let mut failed = None;
                for i in 0..r {
                    if let Err(e) = pool.crash_and_mount_writable() { failed = Some(format!("remount{i}:{e:?}")); break; }
                }
                pool.session = None;
                let label = format!("remount W={w} r={r}");
                match failed {
                    None => run_n2_on(&label, &pool.image(), &pool.parameters),
                    Some(e) => println!("N2START_FAILED {label}: {}", e.chars().take(120).collect::<String>()),
                }
            }
        }
        return;
    }
    for &u_slots in &env_list("SLOTS", "192") {
        for &k in &env_list("OVERWRITES", "0") {
            for &units in &env_list("UNITS", "14") {
                for &b in &env_list("BATCHES", "1") {
                    let label = format!("U={u_slots} k={k} u={units} b={b}");
                    match narrow_crashed(u_slots, k, units, b) {
                        Ok(pool) => run_n2_on(&label, &pool.image(), &pool.parameters),
                        Err(e) => println!("N2START_FAILED {label}: {e}"),
                    }
                }
            }
        }
    }
}

// N5 / N2 第二问：取界（harness 里那一份 `FloorRaisesInTheMountWrites::of_the_mount_output`）与真值（写表里第一个带的 F 高于取号那次的系统配置槽写）。
fn floor_in_system_configuration_write(write: &RetainedWrite) -> Option<u64> {
    let bytes = write.bytes()?;
    if bytes.len() < 489 { return None; }
    Some(u64::from_le_bytes(bytes[481..489].try_into().ok()?))
}

fn main_n5() {
    use singlefs_harness::mount_transparency::FloorRaisesInTheMountWrites;
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    for &u_slots in &env_list("SLOTS", "192") {
        for &k in &env_list("OVERWRITES", "0") {
            for &units in &env_list("UNITS", "14") {
                for &b in &env_list("BATCHES", "1") {
                    let Ok(pool) = narrow_crashed(u_slots, k, units, b) else { continue };
                    let image = pool.image();
                    let stream = SharedStream::retaining_contents();
                    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&image)
                        .into_iter().map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone()))).collect();
                    let Ok(mounted) = mount_writable_with_test_only_switches_and_deletion_switches(&pool.parameters, &mut devices,
                        SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, deletion_switches_with_batches_of_one()) else {
                        *tally.entry("mount_err".into()).or_default() += 1; continue };
                    let geometry = HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry();
                    let (writes, _segments, _indexes) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
                    let located = FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &writes);
                    let floor_at_acquisition = writes.iter().find(|w| w.kind == StepKind::SystemConfigurationSlot).and_then(floor_in_system_configuration_write);
                    let truth = floor_at_acquisition.and_then(|f0| writes.iter().position(|w| w.kind == StepKind::SystemConfigurationSlot && floor_in_system_configuration_write(w).is_some_and(|f| f > f0)));
                    let admission = mounted.output.space_admission.floor_raises().len();
                    let continuation = mounted.output.floor_raises_pushed_by_the_continuation.len();
                    let verdict = match (located, truth) {
                        (FloorRaisesInTheMountWrites::NotPushed, None) => "ok-not-pushed".to_string(),
                        (FloorRaisesInTheMountWrites::FirstWriteAt(a), Some(t)) if a == t => "ok-located".to_string(),
                        (other, t) => { println!("N5MISS U={u_slots} k={k} u={units} b={b} located={other:?} truth={t:?} admission_raises={admission} continuation_raises={continuation}"); "miss".to_string() }
                    };
                    *tally.entry(format!("{verdict} admission_raises={} continuation_raises={}", admission.min(1), continuation.min(1))).or_default() += 1;
                }
            }
        }
    }
    for (k, c) in &tally { println!("N5TALLY {k} count={c}"); }
}

// N7：中央映射树真节点数与模型那条上界（`model::central_mapping_tree_nodes_upper_bound`）——条目数取实现这一版的真条目数，
// 上一版的节点数一种取沿来路带的上界（模型的读法甲），一种取上一版的真节点数（更紧，判上界那段推理本身）。
fn main_n7() {
    use singlefs_core::mounted_session::{UserChange, UserChangePublished};
    use singlefs_core::transaction::TransactionOutput;
    use singlefs_harness::model::central_mapping_tree_nodes_upper_bound;
    let width = width_of(env_u64("WIDTH", 4));
    let ops_per_history = env_u64("OPS", 12);
    let max_units = env_u64("MAX_UNITS", 1500);
    let mut worst_slack_ratio = 0f64;
    let mut violations = 0u64;
    let mut publishes = 0u64;
    let mut max_nodes = 0usize;
    let mut max_bound = 0u64;
    let unit_area_slots = env_u64("UNIT_AREA_SLOTS", 211_968);
    let mut histories_crossing = 0u64;
    let mut histories = 0u64;
    for seed in env_list("SEEDS", "1,2,3") {
        histories += 1;
        // 只算映射树根之下那几个节点按上界多占的槽（模型每次发布把「上界 − 1」个节点各一槽加进占槽上界）：carried 是模型的读法甲，actual 是真节点数。
        let mut slack_slots_carried = 0u64;
        let mut slack_slots_actual = 0u64;
        let mut crossed_at: Option<u64> = None;
        let mut publish_index = 0u64;
        let mut rng = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut next = || { rng ^= rng << 13; rng ^= rng >> 7; rng ^= rng << 17; rng };
        let mut pool = start_plain(width);
        let first = pool.session().current.file_version().expect("第一个文件").clone();
        let mut prev_bound = first.central_mapping_tree.node_count() as u64;
        let mut prev_actual = prev_bound;
        let mut check = |label: &str, outputs: Vec<TransactionOutput>, prev_bound: &mut u64, prev_actual: &mut u64| {
            let mut outs = outputs;
            outs.sort_by_key(|o| o.root.checkpoint_txg);
            for o in outs {
                let entries = o.central_mapping_tree.shape.keys_in_order().len() as u64;
                let nodes = o.central_mapping_tree.node_count();
                let bound_carried = central_mapping_tree_nodes_upper_bound((*prev_bound).max(1), entries);
                let bound_tight = central_mapping_tree_nodes_upper_bound((*prev_actual).max(1), entries);
                publishes += 1;
                max_nodes = max_nodes.max(nodes);
                if nodes as u64 > bound_tight || nodes as u64 > bound_carried {
                    violations += 1;
                    println!("N7VIOLATION seed={seed} {label} txg={} entries={entries} nodes={nodes} bound_tight={bound_tight} bound_carried={bound_carried} prev_actual={prev_actual} prev_bound={prev_bound}", o.root.checkpoint_txg.0);
                }
                let ratio = bound_carried as f64 / nodes as f64;
                if ratio > worst_slack_ratio { worst_slack_ratio = ratio; println!("N7SLACK seed={seed} {label} txg={} entries={entries} nodes={nodes} bound_carried={bound_carried} bound_tight={bound_tight}", o.root.checkpoint_txg.0); }
                *prev_bound = bound_carried;
                *prev_actual = nodes as u64;
                max_bound = max_bound.max(bound_carried);
                publish_index += 1;
                slack_slots_carried += bound_carried - 1;
                slack_slots_actual += nodes as u64 - 1;
                if crossed_at.is_none() && slack_slots_carried * 64 > unit_area_slots {
                    crossed_at = Some(publish_index);
                    println!("N7CROSS seed={seed} publish={publish_index} mapping_slots_carried={slack_slots_carried} mapping_slots_actual={slack_slots_actual} threshold_slots={}", unit_area_slots / 64);
                }
            }
        };
        let outputs_of = |published: &UserChangePublished| -> Vec<TransactionOutput> {
            published.versions_published.iter().cloned()
                .chain(published.floor_raises.iter().flat_map(|r| r.publishes.iter().cloned()))
                .collect()
        };
        for op in 0..ops_per_history {
            let choice = next() % 10;
            let result = if op + 1 == ops_per_history {
                let inode = InodeNumber(FIRST_INODE_NUMBER);
                pool.publish(UserChange::Unlink { inode }, None).map(|p| ("unlink", p))
            } else if choice < 7 {
                let units = 2 + next() % (max_units - 1);
                pool.sequential_write((units as usize - 1) * data_unit_payload_capacity() + 1 + (next() % 1000) as usize).map(|p| ("seq", p))
            } else {
                pool.overwrite(OVERWRITE_BYTES).map(|p| ("overwrite", p))
            };
            match result {
                Ok((label, published)) => check(label, outputs_of(&published), &mut prev_bound, &mut prev_actual),
                Err(e) => { println!("N7REFUSED seed={seed} op={op} {}", format!("{e:?}").chars().take(120).collect::<String>()); break; }
            }
        }
        if crossed_at.is_some() { histories_crossing += 1; }
        println!("N7HISTORY seed={seed} publishes={publish_index} mapping_slots_carried={slack_slots_carried} mapping_slots_actual={slack_slots_actual} crossed_at={crossed_at:?}");
    }
    println!("N7CROSSING histories={histories} crossing={histories_crossing} max_bound={max_bound}");
    println!("N7SUMMARY publishes={publishes} violations={violations} max_nodes={max_nodes} worst_ratio_bound_carried_over_nodes={worst_slack_ratio:.2}");
}

fn main() {
    if env_str("MODE", "n1") == "n7" {
        main_n7();
        return;
    }
    if env_str("MODE", "n1") == "n5" {
        main_n5();
        return;
    }
    if env_str("MODE", "n1") == "n2" {
        main_n2();
        return;
    }
    let family = env_str("FAMILY", "narrow");
    let mut starts: Vec<Result<Start, String>> = Vec::new();
    match family.as_str() {
        "narrow" => {
            for &u_slots in &env_list("SLOTS", "192") {
                for &k in &env_list("OVERWRITES", "0") {
                    for &units in &env_list("UNITS", "14") {
                        for &b in &env_list("BATCHES", "1") {
                            starts.push(narrow_start(u_slots, k, units, b).map_err(|e| format!("narrow U={u_slots} k={k} u={units} b={b}: {e}")));
                        }
                    }
                }
            }
        }
        "plain" | "cycle" => {
            for &w in &env_list("WIDTHS", "384") {
                for &k in &env_list("OVERWRITES", "3") {
                    for &m in &env_list("REMOUNTS", "0") {
                        for &k2 in &env_list("OVERWRITES_AFTER", "0") {
                            let s = plain_start(w, k, m, k2).map_err(|e| format!("plain W={w} k={k} m={m} k2={k2}: {e}"));
                            if family == "cycle" {
                                for &k3 in &env_list("OVERWRITES_AFTER_UNMOUNT", "1") {
                                    let s2 = plain_start(w, k, m, k2).map_err(|e| format!("plain: {e}"))
                                        .and_then(|s| after_a_full_unmount_and_remount(s, k3));
                                    starts.push(s2);
                                }
                                drop(s);
                            } else {
                                starts.push(s);
                            }
                        }
                    }
                }
            }
        }
        "ncycle" => {
            // 窄盘起点卸载做完、再挂可写、覆盖写 k3 次，再在它上面判下一次卸载（卸载—再挂循环）。
            for &u_slots in &env_list("SLOTS", "192") {
                for &k in &env_list("OVERWRITES", "0") {
                    for &units in &env_list("UNITS", "14") {
                        for &b in &env_list("BATCHES", "1") {
                            for &k3 in &env_list("OVERWRITES_AFTER_UNMOUNT", "1") {
                                starts.push(narrow_start(u_slots, k, units, b)
                                    .and_then(|s| after_a_full_unmount_and_remount(s, k3))
                                    .map_err(|e| format!("ncycle U={u_slots} k={k} u={units} b={b} k3={k3}: {e}")));
                            }
                        }
                    }
                }
            }
        }
        "list" => {
            // SHAPES="U:k:u:b;U:k:u:b;…"：窄盘起点逐个点名。
            for shape in env_str("SHAPES", "").split(';').filter(|x| !x.is_empty()) {
                let v: Vec<u64> = shape.split(':').map(|x| x.parse().expect("十进制")).collect();
                starts.push(narrow_start(v[0], v[1], v[2], v[3]).map_err(|e| format!("narrow {shape}: {e}")));
            }
        }
        other => panic!("FAMILY {other}"),
    }
    for start in starts {
        match start {
            Ok(start) => run_n1_on(start),
            Err(error) => println!("START_FAILED {error}"),
        }
    }
}
