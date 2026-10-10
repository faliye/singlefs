//! m3-batch16-code-r1 攻方原型（Opus 腿）：底子是 `research/prompts/m3-batch14-code-r1-opus-model/probe/main.rs` 原样，文件末尾加 MODE=r2。
//! 以下是底子原来的文件头。
//! m3-batch14-code-r1 攻方原型（Opus 腿）。底子是上一轮 `research/prompts/m3-unlink-code-r3-opus-model/probe/main.rs` 的 N1 那一半
//! （起点四族、录卸载、前缀刀、三种丢盘、读法三交集、只读挂载整读、层 0 枚举），移到这一轮的冻结副本上；N2/N5/N7 那几段删掉。
//! 新加：MODE=remount（形 R：卸载里崩、两盘再可写挂载、挂载里再崩、丢一块盘）；MODE=abandon（造被抛弃的根再卸载）。
//! 同一份源码在三份拷贝上编：GA（冻结副本原样）、IMM（丙1：卸载那一格整截当场放开）、HELD（卸载那一格整截扣到生效）。
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
        current = publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), QF_TIME, session.instance)
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
// 这一轮新加。

/// 窄盘族意图那次给的写入时间（Q-F 乙之后意图带调用方的时间；与 N1 无关，取一个与第一个文件不同的值）。
const QF_TIME: u64 = FIRST_WRITE_TIME_SECONDS + 100_000;

/// 一块盘丢掉之后剩下那块盘上用户看得到的结局：只读挂载、整读每个活文件。交回（择到的根 txg, 分类, 内容哈希）。
fn lone_outcome(image: &MemoryPool, lost: u32) -> (Option<u64>, String, Option<u64>) {
    let survivor = without_device(image, Some(lost));
    let chosen = recover(&survivor, JournalPolicy::Consult).effective_root.map(|(_, txg)| txg.0);
    match read_everything(&survivor) {
        Ok((_txg, hash)) => (chosen, "ok".to_string(), Some(hash)),
        Err(e) => (chosen, e, None),
    }
}

/// 一份崩溃镜像上两块盘都在、再可写挂载一次（录写流、带内容）：窄盘族与起点同一种开关，别的族照 `crash_and_mount_writable`。
fn remount_recorded(crashed: &MemoryPool, start: &Start) -> Result<(Vec<RetainedWrite>, Vec<Vec<usize>>, String), String> {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(crashed)
        .into_iter()
        .map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone())))
        .collect();
    let mounted = if start.label.starts_with("narrow") {
        mount_writable_with_test_only_switches_and_deletion_switches(
            &start.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On,
            deletion_switches_with_batches_of_one(),
        )
    } else {
        mount_writable_with_space_admission(&start.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
    };
    let summary = match &mounted {
        Ok(m) => format!(
            "ok row_txg={} warm_up={} cont={}",
            m.output.row_publish.root().checkpoint_txg.0,
            m.output.warm_up_publishes.len(),
            format!("{:?}", m.output.deletion_queue_continuation_outcome).split(|c: char| c == '(' || c == '{' || c == ' ').next().unwrap_or("")
        ),
        Err(e) => return Err(format!("{e:?}").chars().take(120).collect()),
    };
    drop(mounted);
    let (writes, segments, _) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&stream.retained_operations(), &start.geometry);
    Ok((writes, segments, summary))
}

/// 形 R 一个状态的判定：丢 `lost` 之后剩下那块盘的结局，与卸载之前同一块盘上的比；交集按读法三分卸载那一截与再挂那一截。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RVerdict {
    lost: u32,
    chosen: Option<u64>,
    inter_unmount: usize,
    inter_remount: usize,
    read: String,
}

struct RJudge<'a> {
    pre: &'a MemoryPool,
    records: Vec<singlefs_core::allocator::AllocationRecord>,
    current_txg: u64,
    baseline: BTreeMap<u32, (Option<u64>, String, Option<u64>)>,
    /// 卸载之前两块盘都在时整读出的内容哈希（现行那一版）：丢盘之后读出它算「更新」，不算不同。
    current_hash: Option<u64>,
}

impl RJudge<'_> {
    fn judge(&self, image: &MemoryPool, unmount_slots: &BTreeSet<(u32, u64)>, remount_slots: &BTreeSet<(u32, u64)>, lost: u32) -> RVerdict {
        let (chosen, class, hash) = lone_outcome(image, lost);
        let referenced = |slot: &(u32, u64)| {
            slot.0 != lost && chosen.is_some_and(|r| r <= self.current_txg && referenced_by_records_and_headers(&self.records, self.pre, r, *slot))
        };
        let inter_unmount = unmount_slots.iter().filter(|s| referenced(s)).count();
        let inter_remount = remount_slots.iter().filter(|s| referenced(s)).count();
        let (_, base_class, base_hash) = &self.baseline[&lost];
        let read = if base_class != "ok" {
            format!("baseline_err[{base_class}]")
        } else if class != "ok" {
            format!("differs[{class}]")
        } else if hash != *base_hash && hash == self.current_hash {
            "newer".to_string()
        } else if hash != *base_hash {
            "differs[content]".to_string()
        } else {
            "same".to_string()
        };
        RVerdict { lost, chosen, inter_unmount, inter_remount, read }
    }
}

/// 形 R：在起点上录一次卸载；对卸载写流的每个前缀刀 n（默认只取「已有系统配置写、根还没落满」那一截，`ALL_CUTS=1` 取全部），
/// 两块盘都在再可写挂载一次、录它的写流；再对那条写流的每个前缀刀 m（`RENUM_N` 点名的 n 另跑层 0 枚举域），丢盘 0 / 丢盘 1 各判一遍。
fn run_remount_on(start: Start) {
    let t0 = Instant::now();
    let session = start.pool.session.as_ref().expect("会话");
    let current_txg = session.current.root().checkpoint_txg.0;
    let (result, operations, spans) = recorded_unmount(&start);
    let outcome = match &result {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => format!("Raised publishes={}", raised.publishes.len()),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "NothingWritten".to_string(),
        Err(error) => format!("Err {}", format!("{error:?}").chars().take(120).collect::<String>()),
    };
    let (writes, _segments, _) = writes_and_segments_with_stream_indexes_and_entries(&operations, &spans, &start.geometry);
    let pre = start.pool.image();
    let baseline: BTreeMap<u32, (Option<u64>, String, Option<u64>)> = [0u32, 1].into_iter().map(|d| (d, lone_outcome(&pre, d))).collect();
    let current_hash = read_everything(&pre).ok().map(|(_, hash)| hash);
    let judge = RJudge { pre: &pre, records: session.allocator.records().to_vec(), current_txg, baseline, current_hash };
    let kinds: Vec<&'static str> = writes.iter().map(|w| w.kind.name()).collect();
    println!("RSTART {} | current_txg={current_txg} unmount={outcome} writes={} kinds={kinds:?} baseline={:?}", start.label, writes.len(), judge.baseline);
    if result.is_err() {
        return;
    }
    let total_roots = writes.iter().filter(|w| w.kind == StepKind::RootRecordFua).count();
    let all_cuts = env_u64("ALL_CUTS", 0) == 1;
    let renum: Vec<u64> = env_list("RENUM_N", "");
    let max_states = env_u64("MAX_STATES", 1_000_000);
    let print_limit = env_u64("RCUT_PRINT", 30);
    let mut printed = 0u64;
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    for n in 0..=writes.len() {
        let sc = writes[..n].iter().filter(|w| w.kind == StepKind::SystemConfigurationSlot).count();
        let roots = writes[..n].iter().filter(|w| w.kind == StepKind::RootRecordFua).count();
        if !all_cuts && !(sc >= 1 && roots < total_roots) {
            continue;
        }
        let persisted: Vec<bool> = (0..writes.len()).map(|i| i < n).collect();
        let crashed = materialize(&pre, &writes, &persisted);
        let unmount_slots = unit_slots_written(&writes, &persisted);
        let (w2, seg2, summary) = match remount_recorded(&crashed, &start) {
            Ok(x) => x,
            Err(e) => {
                println!("RREMOUNT_ERR {} n={n} sc={sc} roots={roots} {e}", start.label);
                *tally.entry("remount_err".to_string()).or_default() += 1;
                continue;
            }
        };
        let kinds2: Vec<&'static str> = w2.iter().map(|w| w.kind.name()).collect();
        println!("RREMOUNT {} n={n} sc={sc} roots={roots} {summary} writes2={} kinds2={kinds2:?}", start.label, w2.len());
        for m in 0..=w2.len() {
            let p2: Vec<bool> = (0..w2.len()).map(|i| i < m).collect();
            let image = materialize(&crashed, &w2, &p2);
            let remount_slots = unit_slots_written(&w2, &p2);
            for lost in [0u32, 1] {
                let v = judge.judge(&image, &unmount_slots, &remount_slots, lost);
                let class = format!("prefix lost=dev{lost} inter_unmount={} inter_remount={} read={}",
                    (v.inter_unmount > 0) as u8, (v.inter_remount > 0) as u8, v.read.split(':').next().unwrap_or(""));
                *tally.entry(class).or_default() += 1;
                if (v.read != "same" || v.inter_unmount + v.inter_remount > 0) && printed < print_limit {
                    printed += 1;
                    let survivor_floor = choose_system_configuration(&without_device(&image, Some(lost))).ok().map(|sc| sc.quantities.rollback_floor.0);
                    println!("RCUT {} n={n} sc={sc} roots={roots} m={m} next2={} survivor_sc_floor={survivor_floor:?} {v:?}", start.label, w2.get(m).map_or("end", |w| w.kind.name()));
                }
            }
        }
        if renum.contains(&(n as u64)) {
            let states = closed_form_state_count(&seg2);
            let Some(judged_root_index) = w2.iter().position(|w| w.kind == StepKind::RootRecordFua) else { continue };
            if states > max_states {
                println!("RENUM_SKIPPED {} n={n} closed_form_states={states} > {max_states}", start.label);
                continue;
            }
            let versions = start.pool.published_versions();
            let mut histogram: BTreeMap<RVerdict, u64> = BTreeMap::new();
            let mut observed = 0u64;
            let tally_enum = enumerate_layer0_selecting_versions_observing_each_state(
                &crashed, &w2, &seg2, judged_root_index, &versions, &|_, _| true,
                &mut |image, _report| {
                    observed += 1;
                    let state = materialize(image.base, image.writes, &image.persisted);
                    let remount_slots = unit_slots_written(image.writes, &image.persisted);
                    for lost in [0u32, 1] {
                        *histogram.entry(judge.judge(&state, &unmount_slots, &remount_slots, lost)).or_default() += 1;
                    }
                },
            );
            println!("RENUM {} n={n} closed_form_states={states} states={} observed={observed} checker_violated={:?}", start.label, tally_enum.states, tally_enum.checker_violated_states);
            for (v, count) in &histogram {
                println!("RENUMV {} n={n} count={count} {v:?}", start.label);
            }
        }
    }
    for (k, c) in &tally {
        println!("RTALLY {} {k} count={c}", start.label);
    }
    println!("RDONE {} elapsed={:.1}s", start.label, t0.elapsed().as_secs_f64());
}

/// 起点四族（与上一轮同一套参数名）加 list（窄盘形状 `U:k:u:b;…`）。
fn starts_of_the_family() -> Vec<Result<Start, String>> {
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
                            if family == "cycle" {
                                for &k3 in &env_list("OVERWRITES_AFTER_UNMOUNT", "1") {
                                    starts.push(plain_start(w, k, m, k2).map_err(|e| format!("plain W={w} k={k} m={m} k2={k2}: {e}"))
                                        .and_then(|s| after_a_full_unmount_and_remount(s, k3)));
                                }
                            } else {
                                starts.push(plain_start(w, k, m, k2).map_err(|e| format!("plain W={w} k={k} m={m} k2={k2}: {e}")));
                            }
                        }
                    }
                }
            }
        }
        "ncycle" => {
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
            for shape in env_str("SHAPES", "").split(';').filter(|x| !x.is_empty()) {
                let v: Vec<u64> = shape.split(':').map(|x| x.parse().expect("十进制")).collect();
                starts.push(narrow_start(v[0], v[1], v[2], v[3]).map_err(|e| format!("narrow {shape}: {e}")));
            }
        }
        "plainlist" => {
            for shape in env_str("SHAPES", "").split(';').filter(|x| !x.is_empty()) {
                let v: Vec<u64> = shape.split(':').map(|x| x.parse().expect("十进制")).collect();
                starts.push(plain_start(v[0], v[1], v[2], v[3]).map_err(|e| format!("plain {shape}: {e}")));
            }
        }
        "abandon" => {
            for &w in &env_list("WIDTHS", "384") {
                for &k in &env_list("OVERWRITES", "3") {
                    for &k2 in &env_list("OVERWRITES_AFTER", "0") {
                        starts.push(abandon_start(w, k, k2).map_err(|e| format!("abandon W={w} k={k} k2={k2}: {e}")));
                    }
                }
            }
        }
        other => panic!("FAMILY {other}"),
    }
    starts
}

fn main() {
    let mode = env_str("MODE", "n1");
    if mode == "qf" {
        main_qf();
        return;
    }
    for start in starts_of_the_family() {
        match (mode.as_str(), start) {
            ("n1", Ok(start)) => run_n1_on(start),
            ("remount", Ok(start)) => run_remount_on(start),
            ("r2", Ok(start)) => run_r2_on(start),
            ("unmounthide", Ok(start)) => run_unmount_hide_on(start),
            (_, Err(error)) => println!("START_FAILED {error}"),
            (other, Ok(_)) => panic!("MODE {other}"),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// MODE=n1 FAMILY=abandon：造一条被抛弃的根，再在它之后的会话上卸载（判 GA 不算被抛弃的根那一支）。

/// 一次挂载期间点名的几段读回全 0（同 history.rs 私有的 `DeviceReadingZerosOverHiddenRanges` 的做法，写到哪一段那一段就不再藏）。
struct Hiding {
    inner: SparseBlockDevice,
    hidden: Vec<(u64, u64)>,
}

impl singlefs_core::block_device::BlockDevice for Hiding {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let length = buffer.len() as u64;
        // 第 16 批加：HIDE_ERR=1 时读到藏着的那几段报读错（不是读回全 0）。
        static HIDE_ERR: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if *HIDE_ERR.get_or_init(|| std::env::var("HIDE_ERR").is_ok_and(|v| v == "1"))
            && self.hidden.iter().any(|&(start, len)| start < offset.0 + length && offset.0 < start + len)
        {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(std::io::Error::other("注入的读错")));
        }
        self.inner.read_at(offset, buffer)?;
        for &(start, len) in &self.hidden {
            if start < offset.0 + length && offset.0 < start + len {
                let a = start.saturating_sub(offset.0) as usize;
                let b = ((start + len - offset.0).min(length)) as usize;
                buffer[a..b].fill(0);
            }
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: singlefs_core::block_device::WriteDurability) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let length = bytes.len() as u64;
        self.hidden.retain(|&(start, len)| !(start < offset.0 + length && offset.0 < start + len));
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.hidden.retain(|&(start, len)| !(start < offset.0 + length && offset.0 < start + len));
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 普通盘起点覆盖写 k 次；再录一次覆盖写，崩在它的根槽写落下、系统配置轮换没落的那一刀（系统配置没见证它，C554 乙不拒）；
/// 那一刀上可写挂载时把那条根的根槽与那次发布的单元写读回全 0 —— 恢复落到上一版、取新实例，那条根留在环里成了被抛弃的根；
/// 挂载交回之后照常读得出。之后在这次挂载的会话上再覆盖写 k2 次，交回起点（卸载在它上面录）。
fn abandon_start(width_code: u64, overwrites: u64, overwrites_after: u64) -> Result<Start, String> {
    use singlefs_core::transaction::{publish_overwrite, FirstFile};
    let width = width_of(width_code);
    let mut pool = start_plain(width);
    for i in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("overwrite{i}:{e:?}"))?;
    }
    let image = pool.image();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&image)
        .into_iter()
        .map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone())))
        .collect();
    let abandoned_txg = {
        let session = pool.session.as_ref().expect("会话");
        let mut allocator = session.allocator.clone();
        let previous = session.current.file_version().expect("带文件").clone();
        let mut writer = PoolWriter::new(&pool.parameters, devices.as_mut_slice());
        let content = vec![0x5a_u8; OVERWRITE_BYTES];
        publish_overwrite(&mut writer, &mut allocator, &previous, FirstFile { content: &content, write_time_seconds: QF_TIME }, session.instance)
            .map_err(|e| format!("recorded-overwrite:{e:?}"))?
            .root
            .checkpoint_txg
            .0
    };
    let (writes, _segments, _) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&stream.retained_operations(), &width.fixed_geometry());
    let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).ok_or("没有根槽写")?;
    let persisted: Vec<bool> = (0..writes.len()).map(|i| i <= root_index).collect();
    let crashed = materialize(&image, &writes, &persisted);
    let hidden_of = |device: DeviceIdentity| -> Vec<(u64, u64)> {
        writes.iter().take(root_index + 1)
            .filter(|w| w.device == device && (w.kind == StepKind::UnitWrite || w.kind == StepKind::RootRecordFua))
            .map(|w| (w.offset.0, w.length_in_bytes()))
            .collect()
    };
    let mut hiding: Vec<(DeviceIdentity, Hiding)> = plain_devices_on(&crashed)
        .into_iter()
        .map(|(id, d)| (id, Hiding { inner: d, hidden: hidden_of(id) }))
        .collect();
    pool.session = None;
    let mounted = mount_writable_with_space_admission(&pool.parameters, &mut hiding, SpaceAdmission::JudgedByTheFormula)
        .map_err(|e| format!("abandoning-mount:{}", format!("{e:?}").chars().take(160).collect::<String>()))?;
    let (output, session) = MountedSession::of_the_mount(mounted);
    let chosen = output.row_publish.root().checkpoint_txg.0;
    pool.devices = hiding.into_iter().map(|(id, h)| (id, h.inner)).collect();
    pool.session = Some(session);
    for i in 0..overwrites_after {
        pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("after-abandon-overwrite{i}:{e:?}"))?;
    }
    Ok(Start {
        label: format!("abandon W={width_code} k={overwrites} k2={overwrites_after} abandoned_txg={abandoned_txg} row_txg={chosen}"),
        pool,
        geometry: width.fixed_geometry(),
    })
}

// ---------------------------------------------------------------------------------------------
// MODE=qf（N3）：同一份崩溃镜像各自续做两次，两份盘逐字节比；续做各批这条记录的写入时间是不是意图那次给的。

fn run_qf(u_slots: u64, overwrites: u64, units: u64, batches: u64, intent_time: u64) -> Result<String, String> {
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let mut pool = start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(width, u_slots);
    for i in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("overwrite{i}:{e:?}"))?;
    }
    if units > 1 {
        pool.sequential_write((units as usize - 1) * data_unit_payload_capacity() + 100).map_err(|e| format!("seqwrite:{e:?}"))?;
    }
    let time_before = pool.session().current.file_version().and_then(|v| {
        v.inode_leaf_containers.iter().flat_map(|c| c.contents.records.iter()).find(|r| r.inode == FIRST_INODE_NUMBER).map(|r| r.write_time_seconds)
    });
    {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().expect("会话");
        let mut current = session.current.version_of_trees().expect("树").clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), intent_time, session.instance)
            .map_err(|e| format!("intent:{e:?}"))?;
        for b in 0..batches {
            current = publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER),
                deletion_switches_with_batches_of_one().batch_size, session.instance).map_err(|e| format!("batch{b}:{e:?}"))?;
        }
    }
    pool.session = None;
    let image = pool.image();
    let continue_once = || -> Result<(MemoryPool, Vec<Option<u64>>), String> {
        let mut devices = plain_devices_on(&image);
        let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
            &pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, deletion_switches_with_batches_of_one(),
        ).map_err(|e| format!("mount:{}", format!("{e:?}").chars().take(100).collect::<String>()))?;
        let times: Vec<Option<u64>> = mounted.output.deletion_queue_continuation.iter().map(|v| {
            v.inode_leaf_containers.iter().flat_map(|c| c.contents.records.iter()).find(|r| r.inode == FIRST_INODE_NUMBER).map(|r| r.write_time_seconds)
        }).collect();
        drop(mounted);
        Ok((MemoryPool { devices: devices.iter().map(|(id, d)| (*id, d.image.clone())).collect(), device_size_in_bytes: image.device_size_in_bytes }, times))
    };
    let (first, times) = continue_once()?;
    let (second, _) = continue_once()?;
    let identical = first.devices == second.devices;
    let steps = times.len();
    let wrong_times = times.iter().filter(|t| t.is_some_and(|t| t != intent_time)).count();
    let on_disk_after = mount_read_only(&first).ok().and_then(|m| {
        m.mounted.inode_records().iter().find(|r| r.inode == FIRST_INODE_NUMBER).map(|r| r.write_time_seconds)
    });
    Ok(format!("identical={identical} continuation_steps={steps} steps_with_record={} wrong_times={wrong_times} time_before={time_before:?} on_disk_after={on_disk_after:?}",
        times.iter().filter(|t| t.is_some()).count()))
}

fn main_qf() {
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    for &time in &env_list("QF_TIMES", "1788100000,0,18446744073709551615") {
        for &u_slots in &env_list("SLOTS", "240") {
            for &k in &env_list("OVERWRITES", "0") {
                for &units in &env_list("UNITS", "4") {
                    for &b in &env_list("BATCHES", "0") {
                        let label = format!("U={u_slots} k={k} u={units} b={b} t={time}");
                        match run_qf(u_slots, k, units, b, time) {
                            Ok(text) => {
                                let key = format!("identical={} wrong_times_nonzero={}", text.contains("identical=true"), !text.contains("wrong_times=0 "));
                                *tally.entry(key).or_default() += 1;
                                println!("QF {label} {text}");
                            }
                            Err(e) => println!("QF_FAILED {label} {e}"),
                        }
                    }
                }
            }
        }
    }
    for (k, c) in &tally {
        println!("QFTALLY {k} count={c}");
    }
}

// =============================================================================================
// m3-batch16-code-r1 攻方腿（Opus）新加：MODE=r2（形 R 在甲之下放开扫）。
// 一段历史 = 起点（四族之一）→ 录卸载 → 卸载写流的前缀刀 n（崩在卸载里）→ 两块盘都在再可写挂载（可选：挂载期间盘 HIDE_DEV 的根环
// 那几槽读回全 0，模拟这次挂载里一直读坏、挂载之后又读得出的瞬时读错）→ 挂载做成之后照用户动作再覆盖写 J 次。挂载与 J 次覆盖写录在同一条写流里；
// 这条写流的每个前缀刀（RCUT）都判，`RENUM_N` 点名的 n 另把这条写流的层 0 枚举域整个跑一遍（每个状态 checker 照跑）。
// 判法：丢盘 0 / 丢盘 1，剩下那块盘只读挂载、整读每个活文件；读不出、或读出的内容不是任何一版合法内容（卸载之前、再挂之后每一版的两盘整读哈希，
// 加卸载之前同一块盘单独整读的哈希）就记坏。TWICE=1：再挂写流前缀刀 m1 落在暖机落满之前的，再挂第二次（不注读错），第二次的写流每个前缀刀再判。

use singlefs_harness::memory_pool::SparseDevice;

impl DeviceOverASparseImage for RecordingBlockDevice<Hiding> {
    fn sparse_image(&self) -> &SparseDevice {
        &self.wrapped_device().inner.image
    }
}

/// 盘 `device` 上根环全部槽的 (偏移, 槽距)。
fn ring_ranges_on(image: &MemoryPool, device: u32) -> Vec<(u64, u64)> {
    let Ok(sc) = choose_system_configuration(image) else { return Vec::new() };
    let immutable = sc.immutable;
    let spacing = immutable.sizes.fixed_structure_slot_spacing;
    every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter(|ring_slot| immutable.region_devices[ring_slot.region as usize].0 == device)
        .map(|ring_slot| (slot_offset(ring_slot, spacing).0, u64::from(spacing)))
        .collect()
}

struct R2Run {
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    summary: String,
    acceptable: BTreeSet<u64>,
    versions: Vec<PublishedVersion>,
    mount_writes: usize,
}

/// 两块盘都在、再可写挂载（`hide` 点名的盘这次挂载期间根环读回全 0），做成之后覆盖写 `after_j` 次；挂载与覆盖写录在同一条写流里。
fn remount_and_overwrite(crashed: &MemoryPool, start: &Start, hide: Option<u32>, after_j: u64) -> Result<R2Run, String> {
    let stream = SharedStream::retaining_contents();
    let devices: Vec<(DeviceIdentity, RecordingBlockDevice<Hiding>)> = plain_devices_on(crashed)
        .into_iter()
        .map(|(id, d)| {
            let hidden = if Some(id.0) == hide { ring_ranges_on(crashed, id.0) } else { Vec::new() };
            (id, RecordingBlockDevice::with_shared_stream(id, Hiding { inner: d, hidden }, stream.clone()))
        })
        .collect();
    let mut pool = PoolUnderTest {
        parameters: start.pool.parameters.clone(),
        device_bytes: start.pool.device_bytes,
        devices,
        session: None,
        write_time_seconds: start.pool.write_time_seconds,
        content_of_the_current_version: start.pool.content_of_the_current_version.clone(),
        content_of_each_root: start.pool.content_of_each_root.clone(),
    };
    let mounted = if start.label.starts_with("narrow") {
        mount_writable_with_test_only_switches_and_deletion_switches(
            &pool.parameters, &mut pool.devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On,
            deletion_switches_with_batches_of_one(),
        )
    } else {
        mount_writable_with_space_admission(&pool.parameters, &mut pool.devices, SpaceAdmission::JudgedByTheFormula)
    };
    let mounted = match mounted {
        Ok(m) => m,
        Err(e) => {
            let written = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&stream.retained_operations(), &start.geometry).0.len();
            return Err(format!("writes_before_err={written} {}", format!("{e:?}").chars().take(140).collect::<String>()));
        }
    };
    let (output, session) = MountedSession::of_the_mount(mounted);
    if let Some(content) = pool.content_of_each_root.get(&root_key(&output.effective_root)) {
        pool.content_of_the_current_version = content.clone();
    }
    let roots: Vec<singlefs_core::root_record::RootRecord> = std::iter::once(&output.row_publish)
        .chain(&output.warm_up_publishes)
        .map(|version| *version.root())
        .collect();
    for root in &roots {
        pool.note_root_of_the_current_content(root);
    }
    pool.note_floor_raises(output.space_admission.floor_raises());
    let mut summary = format!(
        "ok chosen={} effective={} effective_floor_in_row={} row_txg={} warm_up={} floor_raises={} cont={}",
        output.chosen_root.checkpoint_txg.0,
        output.effective_root.checkpoint_txg.0,
        output.row_publish.root().rollback_floor.0,
        output.row_publish.root().checkpoint_txg.0,
        output.warm_up_publishes.len(),
        output.space_admission.floor_raises().len(),
        format!("{:?}", output.deletion_queue_continuation_outcome).split(|c: char| c == '(' || c == '{' || c == ' ').next().unwrap_or("")
    );
    pool.session = Some(session);
    for (_, device) in pool.devices.iter_mut() {
        device.wrapped_device_mut().hidden.clear();
    }
    let mount_writes = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&stream.retained_operations(), &start.geometry).0.len();
    let mut acceptable = BTreeSet::new();
    if let Ok((_, hash)) = read_everything(&pool.image()) {
        acceptable.insert(hash);
    }
    let mut done = 0;
    for i in 0..after_j {
        // 窄盘族第一个文件已删掉，覆盖写没有文件可改：改成建一个 inode（同样走发布、取落点）。
        let outcome = match pool.overwrite(OVERWRITE_BYTES) {
            Err(singlefs_core::mounted_session::UserChangeRefused::NoFileVersionToChange { .. }) => pool.create_one_inode().map(|_| ()),
            other => other.map(|_| ()),
        };
        match outcome {
            Ok(_) => {
                done += 1;
                if let Ok((_, hash)) = read_everything(&pool.image()) {
                    acceptable.insert(hash);
                }
            }
            Err(e) => {
                summary.push_str(&format!(" overwrite{i}_refused={}", format!("{e:?}").chars().take(60).collect::<String>()));
                break;
            }
        }
    }
    summary.push_str(&format!(" overwrites_done={done}"));
    let (writes, segments, _) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(&stream.retained_operations(), &start.geometry);
    Ok(R2Run { writes, segments, summary, acceptable, versions: pool.published_versions(), mount_writes })
}

/// 一个状态、丢一块盘之后的判定。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct R2Verdict {
    lost: u32,
    chosen: Option<u64>,
    read: String,
    inter_unmount: usize,
    inter_remount: usize,
}

struct R2Judge<'a> {
    pre: &'a MemoryPool,
    records: Vec<singlefs_core::allocator::AllocationRecord>,
    current_txg: u64,
    baseline: BTreeMap<u32, (Option<u64>, String, Option<u64>)>,
    acceptable: BTreeSet<u64>,
}

impl R2Judge<'_> {
    fn judge(&self, image: &MemoryPool, unmount_slots: &BTreeSet<(u32, u64)>, remount_slots: &BTreeSet<(u32, u64)>, lost: u32, run_acceptable: &BTreeSet<u64>) -> R2Verdict {
        let (chosen, class, hash) = lone_outcome(image, lost);
        let referenced = |slot: &(u32, u64)| {
            slot.0 != lost && chosen.is_some_and(|r| r <= self.current_txg && referenced_by_records_and_headers(&self.records, self.pre, r, *slot))
        };
        let inter_unmount = unmount_slots.iter().filter(|s| referenced(s)).count();
        let inter_remount = remount_slots.iter().filter(|s| referenced(s)).count();
        let (_, base_class, base_hash) = &self.baseline[&lost];
        let read = if class != "ok" {
            if base_class != "ok" { format!("baseline_err_and_err[{}]", class.chars().take(40).collect::<String>()) } else { format!("BAD_err[{}]", class.chars().take(60).collect::<String>()) }
        } else {
            let h = hash.expect("ok 带哈希");
            if Some(h) == *base_hash { "same".to_string() }
            else if self.acceptable.contains(&h) || run_acceptable.contains(&h) { "known".to_string() }
            else { "BAD_content".to_string() }
        };
        R2Verdict { lost, chosen, read, inter_unmount, inter_remount }
    }
}

fn run_r2_on(start: Start) {
    let t0 = Instant::now();
    let session = start.pool.session.as_ref().expect("会话");
    let current_txg = session.current.root().checkpoint_txg.0;
    let (result, operations, spans) = recorded_unmount(&start);
    let outcome = match &result {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => format!("Raised publishes={}", raised.publishes.len()),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "NothingWritten".to_string(),
        Err(error) => format!("Err {}", format!("{error:?}").chars().take(120).collect::<String>()),
    };
    let (writes, _segments, _) = writes_and_segments_with_stream_indexes_and_entries(&operations, &spans, &start.geometry);
    let pre = start.pool.image();
    let baseline: BTreeMap<u32, (Option<u64>, String, Option<u64>)> = [0u32, 1].into_iter().map(|d| (d, lone_outcome(&pre, d))).collect();
    let mut acceptable = BTreeSet::new();
    if let Ok((_, h)) = read_everything(&pre) {
        acceptable.insert(h);
    }
    for (_, (_, _, h)) in &baseline {
        if let Some(h) = h { acceptable.insert(*h); }
    }
    let judge = R2Judge { pre: &pre, records: session.allocator.records().to_vec(), current_txg, baseline, acceptable };
    println!("R2START {} | current_txg={current_txg} unmount={outcome} writes={} baseline={:?}", start.label, writes.len(), judge.baseline);
    if result.is_err() {
        return;
    }
    let total_roots = writes.iter().filter(|w| w.kind == StepKind::RootRecordFua).count();
    let all_cuts = env_u64("ALL_CUTS", 0) == 1;
    let hide: Option<u32> = std::env::var("HIDE_DEV").ok().map(|v| v.parse().expect("盘号"));
    let after_js = env_list("AFTER_J", "0");
    let twice = env_u64("TWICE", 0) == 1;
    let renum: Vec<u64> = env_list("RENUM_N", "");
    let max_states = env_u64("MAX_STATES", 1_000_000);
    let print_limit = env_u64("RCUT_PRINT", 20);
    let mut printed = 0u64;
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    for n in 0..=writes.len() {
        let sc = writes[..n].iter().filter(|w| w.kind == StepKind::SystemConfigurationSlot).count();
        let roots = writes[..n].iter().filter(|w| w.kind == StepKind::RootRecordFua).count();
        if !all_cuts && !(sc >= 1 && roots < total_roots) {
            continue;
        }
        let persisted: Vec<bool> = (0..writes.len()).map(|i| i < n).collect();
        let crashed = materialize(&pre, &writes, &persisted);
        let unmount_slots = unit_slots_written(&writes, &persisted);
        for &j in &after_js {
            let run = match remount_and_overwrite(&crashed, &start, hide, j) {
                Ok(run) => run,
                Err(e) => {
                    println!("R2REMOUNT_ERR {} n={n} sc={sc} roots={roots} j={j} {e}", start.label);
                    *tally.entry(format!("remount_err j={j}")).or_default() += 1;
                    continue;
                }
            };
            println!("R2REMOUNT {} n={n} sc={sc} roots={roots} j={j} {} writes2={} mount_writes={}", start.label, run.summary, run.writes.len(), run.mount_writes);
            let w2 = &run.writes;
            for m in 0..=w2.len() {
                let p2: Vec<bool> = (0..w2.len()).map(|i| i < m).collect();
                let image = materialize(&crashed, w2, &p2);
                let remount_slots = unit_slots_written(w2, &p2);
                let phase = if m <= run.mount_writes { "in_mount" } else { "after_mount" };
                for lost in [0u32, 1] {
                    let v = judge.judge(&image, &unmount_slots, &remount_slots, lost, &run.acceptable);
                    let class = format!("prefix j={j} {phase} lost=dev{lost} read={}", v.read.split('[').next().unwrap_or(""));
                    *tally.entry(class).or_default() += 1;
                    if v.read.starts_with("BAD") && printed < print_limit {
                        printed += 1;
                        println!("R2BAD {} n={n} sc={sc} roots={roots} j={j} m={m} next2={} {v:?}", start.label, w2.get(m).map_or("end", |w| w.kind.name()));
                    }
                }
                if twice && j == 0 && m < run.mount_writes {
                    // 暖机落满之前（挂载那一段写流还没写完）崩了，再挂第二次；第二次只录挂载、不注读错。
                    match remount_and_overwrite(&image, &start, None, 0) {
                        Err(e) => {
                            *tally.entry(format!("twice_remount_err j={j}")).or_default() += 1;
                            if printed < print_limit { printed += 1; println!("R2TWICE_ERR {} n={n} m={m} {e}", start.label); }
                        }
                        Ok(run3) => {
                            let w3 = &run3.writes;
                            for m3 in 0..=w3.len() {
                                let p3: Vec<bool> = (0..w3.len()).map(|i| i < m3).collect();
                                let image3 = materialize(&image, w3, &p3);
                                let mut slots = remount_slots.clone();
                                slots.extend(unit_slots_written(w3, &p3));
                                let mut acc = run.acceptable.clone();
                                acc.extend(run3.acceptable.iter().copied());
                                for lost in [0u32, 1] {
                                    let v = judge.judge(&image3, &unmount_slots, &slots, lost, &acc);
                                    let class = format!("twice j={j} lost=dev{lost} read={}", v.read.split('[').next().unwrap_or(""));
                                    *tally.entry(class).or_default() += 1;
                                    if v.read.starts_with("BAD") && printed < print_limit {
                                        printed += 1;
                                        println!("R2TWICE_BAD {} n={n} m={m} m3={m3} {v:?}", start.label);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if renum.contains(&(n as u64)) {
                let states = closed_form_state_count(&run.segments);
                let Some(judged_root_index) = w2.iter().position(|w| w.kind == StepKind::RootRecordFua) else { continue };
                if states > max_states {
                    println!("R2ENUM_SKIPPED {} n={n} j={j} closed_form_states={states} > {max_states}", start.label);
                    continue;
                }
                let mut histogram: BTreeMap<R2Verdict, u64> = BTreeMap::new();
                let mut observed = 0u64;
                let te = enumerate_layer0_selecting_versions_observing_each_state(
                    &crashed, w2, &run.segments, judged_root_index, &run.versions, &|_, _| true,
                    &mut |image, _report| {
                        observed += 1;
                        let state = materialize(image.base, image.writes, &image.persisted);
                        let remount_slots = unit_slots_written(image.writes, &image.persisted);
                        for lost in [0u32, 1] {
                            let mut v = judge.judge(&state, &unmount_slots, &remount_slots, lost, &run.acceptable);
                            v.inter_unmount = (v.inter_unmount > 0) as usize;
                            v.inter_remount = (v.inter_remount > 0) as usize;
                            *histogram.entry(v).or_default() += 1;
                        }
                    },
                );
                println!("R2ENUM {} n={n} j={j} closed_form_states={states} states={} observed={observed} violations={} failed_states={} checker_violated={:?}",
                    start.label, te.states, te.violations, te.failed_states, te.checker_violated_states);
                for (v, count) in &histogram {
                    println!("R2ENUMV {} n={n} j={j} count={count} {v:?}", start.label);
                }
            }
        }
    }
    for (k, c) in &tally {
        println!("R2TALLY {} {k} count={c}", start.label);
    }
    println!("R2DONE {} elapsed={:.1}s", start.label, t0.elapsed().as_secs_f64());
}

/// MODE=unmounthide（N4）：在起点的会话上卸载，卸载期间盘 HIDE_DEV 的根环全部槽读坏（HIDE_ERR=1 报读错，否则读回全 0）；报卸载的结局。
fn run_unmount_hide_on(start: Start) {
    let hide: u32 = env_u64("HIDE_DEV", 1) as u32;
    let image = start.pool.image();
    let mut devices: Vec<(DeviceIdentity, Hiding)> = plain_devices_on(&image)
        .into_iter()
        .map(|(id, d)| {
            let hidden = if id.0 == hide { ring_ranges_on(&image, id.0) } else { Vec::new() };
            (id, Hiding { inner: d, hidden })
        })
        .collect();
    let session = start.pool.session.as_ref().expect("会话");
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    let result = unmount(&start.pool.parameters, &mut devices, &mut allocator, &mut current, session.shadow_ledger);
    let outcome = match &result {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => format!("Raised publishes={} reclaimed={}", raised.publishes.len(), raised.reclaimed.len()),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "NothingWritten".to_string(),
        Err(error) => format!("Err {}", format!("{error:?}").chars().take(200).collect::<String>()),
    };
    println!("UNMOUNT_HIDE {} hide_dev={hide} err_mode={} {outcome}", start.label, std::env::var("HIDE_ERR").unwrap_or_default());
}
