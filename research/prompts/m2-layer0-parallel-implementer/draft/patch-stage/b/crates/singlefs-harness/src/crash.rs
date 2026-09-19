//! 层 0 崩溃点重放（里程碑步 7，D13（验证路线） 已定项 4）：拿录制流在内存里重建镜像，屏障与 FUA 切段、段内任意整写子集、
//! 撕裂态并进「没持久」、没持久的位置放旧字节、两盘各算；每个状态跑一遍步 6 的恢复，拿 oracle（E77（发布的持久顺序） 判据 1）判。
//! 恢复只通过 [`PoolReader`] 读盘，所以崩溃镜像不落文件：基线是稀疏的扇区图，其上叠一层「这一状态里持久了的写」。

use std::collections::BTreeMap;

use std::collections::BTreeSet;
use std::num::{NonZeroU64, NonZeroUsize};
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES,
    UNIT_AREA_START_SLOT,
};

use crate::segments::{FixedGeometry, StepKind};
use crate::{RecordedOperationKind, RetainedOperation};

/// 稀疏设备的扇区宽：第一版测试镜像的 physical_block_size。
pub const SECTOR_BYTES: u64 = 512;

/// 一块盘：只存写过的扇区，没写过的读出全 0。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SparseDevice {
    sectors: BTreeMap<u64, Vec<u8>>,
}

impl SparseDevice {
    #[must_use]
    pub fn read(&self, offset: DeviceOffsetInBytes, length: usize) -> Vec<u8> {
        let mut out = vec![0u8; length];
        self.read_into(offset, &mut out);
        out
    }
    /// 读进调用方的缓冲：先整段写 0，再只拷区间里写过的扇区（按区间查一次，不逐扇区查）——可写挂载与冷启动要把 768 MiB 的
    /// journal 环逐条读一遍，几乎全是没写过的扇区（随机历史，增补 3 第 1 件）。
    pub fn read_into(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) {
        assert!(offset.0.is_multiple_of(SECTOR_BYTES), "读要按扇区对齐");
        let length_in_bytes = u64::try_from(buffer.len()).expect("长度");
        assert!(
            length_in_bytes.is_multiple_of(SECTOR_BYTES),
            "读长度要是整扇区"
        );
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        buffer.fill(0);
        let first_sector = offset.0 / SECTOR_BYTES;
        for (sector, bytes) in self
            .sectors
            .range(first_sector..first_sector + length_in_bytes / SECTOR_BYTES)
        {
            let start = usize::try_from(sector - first_sector).expect("扇区下标") * sector_bytes;
            buffer[start..start + sector_bytes].copy_from_slice(bytes);
        }
    }
    pub fn write(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        assert!(offset.0.is_multiple_of(SECTOR_BYTES), "写要按扇区对齐");
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        assert!(bytes.len().is_multiple_of(sector_bytes), "写长度要是整扇区");
        let first_sector = offset.0 / SECTOR_BYTES;
        for (sector_index, chunk) in bytes.chunks_exact(sector_bytes).enumerate() {
            self.sectors.insert(
                first_sector + u64::try_from(sector_index).expect("扇区下标"),
                chunk.to_vec(),
            );
        }
    }
    /// `[offset, offset + length)` 里写过的扇区号。
    #[must_use]
    pub fn written_sectors_in(&self, offset: DeviceOffsetInBytes, length: u64) -> Vec<u64> {
        let first_sector = offset.0 / SECTOR_BYTES;
        let end_sector = (offset.0 + length) / SECTOR_BYTES;
        self.sectors
            .range(first_sector..end_sector)
            .map(|(sector, _)| *sector)
            .collect()
    }
}

/// 稀疏设备当一块盘用：宿主上重跑整条路（与虚机里同参数同字节）时的被测设备。没写过的读出全 0；屏障什么都不做（内存里没有缓存）。
pub struct SparseBlockDevice {
    pub image: SparseDevice,
    size_in_bytes: u64,
    physical_block_size: singlefs_core::block_device::PhysicalBlockSizeInBytes,
}

impl SparseBlockDevice {
    #[must_use]
    pub fn new(
        size_in_bytes: u64,
        physical_block_size: singlefs_core::block_device::PhysicalBlockSizeInBytes,
    ) -> Self {
        Self {
            image: SparseDevice::default(),
            size_in_bytes,
            physical_block_size,
        }
    }
}

impl singlefs_core::block_device::BlockDevice for SparseBlockDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.image.read_into(offset, buffer);
        Ok(())
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        _durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.image.write(offset, bytes);
        Ok(())
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        Ok(())
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.physical_block_size
    }
    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }
}

/// 一个池在内存里的样子：从带内容的录制流重建。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryPool {
    pub devices: BTreeMap<DeviceIdentity, SparseDevice>,
    /// 每块盘的字节数（checker 算单元区容量用）。
    pub device_size_in_bytes: u64,
}

/// 录制流里的一次写，带内容（屏障不进这张表）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedWrite {
    pub device: DeviceIdentity,
    pub kind: StepKind,
    pub is_force_unit_access: bool,
    pub offset: DeviceOffsetInBytes,
    pub bytes: Vec<u8>,
}

impl MemoryPool {
    #[must_use]
    pub fn with_devices(identities: &[DeviceIdentity], device_size_in_bytes: u64) -> Self {
        Self {
            devices: identities
                .iter()
                .map(|identity| (*identity, SparseDevice::default()))
                .collect(),
            device_size_in_bytes,
        }
    }
    /// 把一段录制流按次序整个施加上去（屏障不改镜像）。
    pub fn apply(&mut self, operations: &[RetainedOperation]) {
        for retained in operations {
            if retained.operation.kind == RecordedOperationKind::Barrier {
                continue;
            }
            let contents = retained
                .contents
                .as_ref()
                .expect("崩溃点重放要开了内容保留的录制流");
            self.devices
                .get_mut(&retained.operation.device)
                .expect("录到的写都落在池里的盘上")
                .write(retained.operation.offset, contents);
        }
    }
    /// 翻一个字节（坏字节探针，里程碑步 6 验收）。
    pub fn flip_byte(
        &mut self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        byte_index: u64,
    ) {
        let sector_offset = DeviceOffsetInBytes(offset.0 + byte_index - byte_index % SECTOR_BYTES);
        let device_image = self.devices.get_mut(&device).expect("有这块盘");
        let mut sector =
            device_image.read(sector_offset, usize::try_from(SECTOR_BYTES).expect("512"));
        sector[usize::try_from(byte_index % SECTOR_BYTES).expect("扇区内偏移")] ^= 0xff;
        device_image.write(sector_offset, &sector);
    }
}

fn record_slot_offsets(
    sectors: impl Iterator<Item = u64>,
    ring_start: DeviceOffsetInBytes,
) -> Vec<DeviceOffsetInBytes> {
    let mut offsets: Vec<DeviceOffsetInBytes> = sectors
        .map(|sector| sector * SECTOR_BYTES)
        .filter(|offset| (offset - ring_start.0).is_multiple_of(JOURNAL_RECORD_BYTES))
        .map(DeviceOffsetInBytes)
        .collect();
    offsets.dedup();
    offsets
}

impl PoolReader for MemoryPool {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.devices.keys().copied().collect()
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.devices
            .get(&device)
            .map(|device_image| device_image.read(offset, length))
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let device_image = self.devices.get(&device)?;
        Some(record_slot_offsets(
            device_image
                .written_sectors_in(ring_start, ring_bytes)
                .into_iter(),
            ring_start,
        ))
    }
}

/// 层 0 枚举出的一个崩溃状态：基线 + 这一状态里持久了的写。
pub struct CrashImage<'base> {
    pub base: &'base MemoryPool,
    pub writes: &'base [RetainedWrite],
    pub persisted: Vec<bool>,
}

impl PoolReader for CrashImage<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.base.device_identities()
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let mut out = PoolReader::read(self.base, device, offset, length)?;
        let read_start = offset.0;
        let read_end = offset.0 + u64::try_from(length).expect("长度");
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            let write_start = write.offset.0;
            let write_end = write_start + u64::try_from(write.bytes.len()).expect("长度");
            let overlap_start = read_start.max(write_start);
            let overlap_end = read_end.min(write_end);
            if overlap_start >= overlap_end {
                continue;
            }
            let destination = usize::try_from(overlap_start - read_start).expect("偏移");
            let source = usize::try_from(overlap_start - write_start).expect("偏移");
            let overlap_length = usize::try_from(overlap_end - overlap_start).expect("长度");
            out[destination..destination + overlap_length]
                .copy_from_slice(&write.bytes[source..source + overlap_length]);
        }
        Some(out)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let mut offsets = self
            .base
            .journal_record_offsets_hint(device, ring_start, ring_bytes)?;
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if !is_persisted || write.device != device || write.kind != StepKind::JournalRecord {
                continue;
            }
            offsets.push(write.offset);
        }
        offsets.sort_unstable();
        offsets.dedup();
        Some(offsets)
    }
}

/// 把一段录制流拆成「写表」与「段（写表下标）」，切法与 [`crate::segments::split_into_segments`] 同一条规则。
#[must_use]
pub fn writes_and_segments(
    operations: &[RetainedOperation],
    geometry: &FixedGeometry,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let mut writes: Vec<RetainedWrite> = Vec::new();
    let mut segments: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for retained in operations {
        match retained.operation.kind {
            RecordedOperationKind::Barrier => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                writes.push(RetainedWrite {
                    device: retained.operation.device,
                    kind: geometry.classify(&retained.operation),
                    is_force_unit_access: retained.operation.kind
                        == RecordedOperationKind::WriteForceUnitAccess,
                    offset: retained.operation.offset,
                    bytes: retained
                        .contents
                        .clone()
                        .expect("崩溃点重放要开了内容保留的录制流"),
                });
                current.push(writes.len() - 1);
                if retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess {
                    segments.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    (writes, segments)
}

/// E77（发布的持久顺序） 的闭式：1 + Σ(2^|段| − 1)。
#[must_use]
pub fn closed_form_state_count(segments: &[Vec<usize>]) -> u64 {
    1 + segments
        .iter()
        .map(|segment| (1u64 << segment.len()) - 1)
        .sum::<u64>()
}

/// checker 读崩溃镜像的口子：与恢复读的是同一份字节，但走 checker 自己的 trait，不经实现的块设备抽象。
impl ImageReader for MemoryPool {
    fn devices(&self) -> Vec<u32> {
        self.devices.keys().map(|identity| identity.0).collect()
    }
    fn device_bytes(&self, _device: u32) -> Option<u64> {
        Some(self.device_size_in_bytes)
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        PoolReader::read(
            self,
            DeviceIdentity(device),
            DeviceOffsetInBytes(offset),
            length,
        )
    }
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        let image = self.devices.get(&DeviceIdentity(device))?;
        let slots: BTreeSet<u64> = image
            .sectors
            .keys()
            .map(|sector| sector * SECTOR_BYTES / SLOT_BYTES)
            .filter(|slot| *slot >= UNIT_AREA_START_SLOT)
            .collect();
        Some(slots.into_iter().collect())
    }
    /// journal 环里写过的扇区所在的记录槽（录制流只收真的记录写，环没有整段写 0 的那一次）。
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        let image = self.devices.get(&DeviceIdentity(device))?;
        let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        let slots: BTreeSet<u64> = image
            .written_sectors_in(DeviceOffsetInBytes(ring_start), JOURNAL_RING_DEFAULT_BYTES)
            .into_iter()
            .map(|sector| (sector * SECTOR_BYTES - ring_start) / JOURNAL_RECORD_BYTES)
            .collect();
        Some(slots.into_iter().collect())
    }
}

impl ImageReader for CrashImage<'_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.base)
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        ImageReader::device_bytes(self.base, device)
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        PoolReader::read(
            self,
            DeviceIdentity(device),
            DeviceOffsetInBytes(offset),
            length,
        )
    }
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        let mut slots: BTreeSet<u64> = ImageReader::candidate_unit_slots(self.base, device)?
            .into_iter()
            .collect();
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if *is_persisted && write.device.0 == device && write.kind == StepKind::UnitWrite {
                slots.insert(write.offset.0 / SLOT_BYTES);
            }
        }
        Some(slots.into_iter().collect())
    }
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        let mut slots: BTreeSet<u64> = ImageReader::candidate_journal_slots(self.base, device)?
            .into_iter()
            .collect();
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if *is_persisted
                && write.device.0 == device
                && (ring_start..ring_start + JOURNAL_RING_DEFAULT_BYTES).contains(&write.offset.0)
            {
                slots.insert((write.offset.0 - ring_start) / JOURNAL_RECORD_BYTES);
            }
        }
        Some(slots.into_iter().collect())
    }
}

/// 记录核对器两条判据在一个状态上的结论。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecordCheck {
    /// 某次发布的根槽写已在盘上，而那次发布的 journal 记录一份都不在（E77（发布的持久顺序） b_ur 臂的「记录流有洞」）。
    pub root_without_record: bool,
    /// 恢复实际走的根 txg ≥ 某次发布，而那次发布写出的某个单元两份都不在（E77（发布的持久顺序） 的独立审计）。
    pub claimed_state_missing_unit: bool,
}

/// 一次发布在写表里的下标：单元写、journal 记录写、根槽写，以及根里写的 checkpoint_txg。
struct PublishWrites {
    units: Vec<usize>,
    records: Vec<usize>,
    root: usize,
    instance: u32,
    checkpoint_txg: u64,
}

fn publishes_in(writes: &[RetainedWrite]) -> Vec<PublishWrites> {
    let mut publishes = Vec::new();
    let mut units = Vec::new();
    let mut records = Vec::new();
    for (index, write) in writes.iter().enumerate() {
        match write.kind {
            StepKind::UnitWrite => units.push(index),
            StepKind::JournalRecord => records.push(index),
            StepKind::RootRecordFua => {
                let (instance, checkpoint_txg) = root_identity_of_write(write);
                publishes.push(PublishWrites {
                    units: std::mem::take(&mut units),
                    records: std::mem::take(&mut records),
                    root: index,
                    instance: instance.0,
                    checkpoint_txg: checkpoint_txg.0,
                });
            }
            StepKind::SuperblockSlot | StepKind::Barrier => {}
        }
    }
    publishes
}

/// 记录核对器（D13（验证路线） 故意不给编号；入参 (崩溃前镜像, 记录流, 崩溃后镜像)）：崩溃前镜像是 `image.base`、
/// 记录流是 `image.writes`、崩溃后镜像是 `image` 本身。一处写「在盘上」= 崩溃后镜像那个位置上的字节与记录流里写下的逐字节相同；
/// 不经恢复代码、不解析树。`effective_root` 取恢复报出来的实际走的那条根。
#[must_use]
pub fn check_records(
    image: &CrashImage<'_>,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> RecordCheck {
    let in_place = |index: usize| {
        let write = &image.writes[index];
        PoolReader::read(image, write.device, write.offset, write.bytes.len())
            .is_some_and(|bytes| bytes == write.bytes)
    };
    let mut check = RecordCheck::default();
    for publish in publishes_in(image.writes) {
        if in_place(publish.root) && !publish.records.iter().any(|record| in_place(*record)) {
            check.root_without_record = true;
        }
        if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg) {
            let mut copies_by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
            for unit in &publish.units {
                copies_by_offset
                    .entry(image.writes[*unit].offset.0)
                    .or_default()
                    .push(*unit);
            }
            if copies_by_offset
                .values()
                .any(|copies| !copies.iter().any(|copy| in_place(*copy)))
            {
                check.claimed_state_missing_unit = true;
            }
        }
    }
    check
}

/// 层 0 的计数：每个状态跑一遍看 journal 的恢复与一遍不看的，oracle 只判前者。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layer0Tally {
    pub states: u64,
    pub violations: u64,
    pub root_persisted_states: u64,
    pub no_file_states: u64,
    pub file_read_states: u64,
    pub failed_states: u64,
    pub journal_differing_states: u64,
    pub verification_ran_states: u64,
    pub verification_failed_states: u64,
    pub first_violation: Option<String>,
    /// 不看 journal 那一遍恢复（`JournalPolicy::Ignore`）过同一个 oracle 判违例的状态数，另计、不混进 `violations`
    /// （靶向对照里「根槽已持久而单元缺席」那一格两遍都违例）。发布 B 之后两遍恢复会读出两个不同的版本，这一遍此前没人判。
    pub ignored_violations: u64,
    pub first_ignored_violation: Option<String>,
    /// 记录核对器判「根在案而记录缺席」的状态数。
    pub record_root_without_record: u64,
    /// 记录核对器判「恢复自称新态而单元缺席」的状态数。
    pub record_claimed_state_missing_unit: u64,
    /// checker：每条不变量在几个状态上评估过（成立或违例）、在几个状态上判违例、第一处违例。
    pub checker_evaluated_states: BTreeMap<&'static str, u64>,
    pub checker_violated_states: BTreeMap<&'static str, u64>,
    pub checker_first_violation: BTreeMap<&'static str, String>,
    /// checker 报「不适用」（这条不变量判的代码在这个状态上没跑到）的状态数：与评估过的状态数分开报，阴性结果不与「没跑到」混在一起
    /// （里程碑「第二个事务」步 6 验收第 3 条）；每条不变量的评估过 + 不适用 = `states`。
    pub checker_not_applicable_states: BTreeMap<&'static str, u64>,
}

impl Layer0Tally {
    /// checker 那一半按不变量报成一段：`I-x.y=评估过/判违例/不适用`，次序照 checker 的清单。
    #[must_use]
    pub fn checker_counts_by_invariant(&self) -> String {
        let count_of = |counts: &BTreeMap<&'static str, u64>, invariant: &str| {
            counts.get(invariant).copied().unwrap_or(0)
        };
        singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .map(|invariant| {
                format!(
                    "{invariant}={}/{}/{}",
                    count_of(&self.checker_evaluated_states, invariant),
                    count_of(&self.checker_violated_states, invariant),
                    count_of(&self.checker_not_applicable_states, invariant)
                )
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// 把紧跟在后面的那一片的计数并进来：计数逐项相加；「第一处」只在前面各片都没有时取这一片的。各片按状态序号从小到大并，
    /// 「第一处」就是序号最小的那一处，与单线程逐个跑逐项相同。按字段拆开写全：新加一个字段而这里没并，编译不过。
    fn absorb_following_slice(&mut self, following_slice: Layer0Tally) {
        let Layer0Tally {
            states,
            violations,
            root_persisted_states,
            no_file_states,
            file_read_states,
            failed_states,
            journal_differing_states,
            verification_ran_states,
            verification_failed_states,
            first_violation,
            ignored_violations,
            first_ignored_violation,
            record_root_without_record,
            record_claimed_state_missing_unit,
            checker_evaluated_states,
            checker_violated_states,
            checker_first_violation,
            checker_not_applicable_states,
        } = following_slice;
        self.states += states;
        self.violations += violations;
        self.root_persisted_states += root_persisted_states;
        self.no_file_states += no_file_states;
        self.file_read_states += file_read_states;
        self.failed_states += failed_states;
        self.journal_differing_states += journal_differing_states;
        self.verification_ran_states += verification_ran_states;
        self.verification_failed_states += verification_failed_states;
        if self.first_violation.is_none() {
            self.first_violation = first_violation;
        }
        self.ignored_violations += ignored_violations;
        if self.first_ignored_violation.is_none() {
            self.first_ignored_violation = first_ignored_violation;
        }
        self.record_root_without_record += record_root_without_record;
        self.record_claimed_state_missing_unit += record_claimed_state_missing_unit;
        for (invariant, evaluated_states) in checker_evaluated_states {
            *self.checker_evaluated_states.entry(invariant).or_insert(0) += evaluated_states;
        }
        for (invariant, violated_states) in checker_violated_states {
            *self.checker_violated_states.entry(invariant).or_insert(0) += violated_states;
        }
        for (invariant, detail) in checker_first_violation {
            self.checker_first_violation
                .entry(invariant)
                .or_insert(detail);
        }
        for (invariant, not_applicable_states) in checker_not_applicable_states {
            *self
                .checker_not_applicable_states
                .entry(invariant)
                .or_insert(0) += not_applicable_states;
        }
    }
}

/// oracle（E77（发布的持久顺序） 判据 1）：读回的内容要对；根槽已持久就不许恢复到旧态；走读不许失败。
/// 单版本形态：整条流只有一次带文件的发布（第一个事务）。
#[must_use]
pub fn oracle_violation(
    outcome: &RecoveryOutcome,
    root_persisted: bool,
    expected_content: &[u8],
) -> Option<String> {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            (content != expected_content).then(|| "读回的内容不对".to_string())
        }
        RecoveryOutcome::NoFile { .. } => {
            root_persisted.then(|| "根槽已持久而恢复到旧态".to_string())
        }
        RecoveryOutcome::Failed { failure, .. } => Some(format!("走读失败：{failure:?}")),
    }
}

/// 文件的一个已发布版本：哪次发布（实例代号 + checkpoint_txg）写出了什么内容。多次发布的流上 oracle 按实际走的根判该读出哪一版。
/// 版本按 (txg, 实例) 认，不只按 txg：设备失而复得或回退会造出两条同 txg 不同实例的根（D22（单元原子性怎么合成） 已定项 7），
/// 只按 txg 认时 oracle 在那一格两个方向都错（三方代码第一轮攻方腿）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedVersion {
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
    pub content: Vec<u8>,
}

/// 多版本形态的 oracle：实际走的根是哪一代，读回的就得是那一代写出的内容；根下面没有文件的那几代（mkfs、暖机）只许报没有文件；
/// 盘上已持久的最新根槽是 (T, 实例 i)，恢复就不许落到按 (txg, 实例) 字典序比它旧的根上（D22（单元原子性怎么合成） 已定项 7：
/// 择新 txg 为主、平局按实例代号高者赢）；走读不许失败。
#[must_use]
pub fn oracle_violation_for_versions(
    outcome: &RecoveryOutcome,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
    versions: &[PublishedVersion],
) -> Option<String> {
    let Some((effective_instance, effective_txg)) = effective_root else {
        return Some("没择到根".to_string());
    };
    if let Some((newest_txg, newest_instance)) = newest_persisted_root {
        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {
            return Some(format!(
                "根槽已持久而恢复到旧态（盘上最新的根槽是实例 {} 第 {} 代，走的是实例 {} 第 {} 代）",
                newest_instance.0, newest_txg.0, effective_instance.0, effective_txg.0
            ));
        }
    }
    let version = versions.iter().find(|version| {
        version.checkpoint_txg == effective_txg && version.instance == effective_instance
    });
    match (outcome, version) {
        (RecoveryOutcome::Failed { failure, .. }, _) => Some(format!("走读失败：{failure:?}")),
        // 落在一个没发布过的更新的根上报「没有文件」：更旧的根下面有文件时是违例（第二轮攻方腿：此前一律放过）。
        (RecoveryOutcome::NoFile { .. }, None) => versions
            .iter()
            .find(|candidate_version| {
                (candidate_version.checkpoint_txg, candidate_version.instance)
                    < (effective_txg, effective_instance)
            })
            .map(|older_version| {
                format!(
                    "走到实例 {} 第 {} 代根报没有文件，而没有这一代的版本、实例 {} 第 {} 代下面有文件",
                    effective_instance.0,
                    effective_txg.0,
                    older_version.instance.0,
                    older_version.checkpoint_txg.0
                )
            }),
        (RecoveryOutcome::NoFile { .. }, Some(_)) => {
            Some(format!("第 {} 代根下面有文件却报没有", effective_txg.0))
        }
        (RecoveryOutcome::FileRead { .. }, None) => Some(format!(
            "第 {} 代根下面没有文件却读出了内容",
            effective_txg.0
        )),
        (RecoveryOutcome::FileRead { content, .. }, Some(version)) => (*content != version.content)
            .then(|| format!("读回的内容不对（走的是第 {} 代根）", effective_txg.0)),
    }
}

/// 这一状态里持久了的根槽写中最新的那一条，按 (txg, 实例) 字典序取（D22（单元原子性怎么合成） 已定项 7 的择新序）。
#[must_use]
pub fn newest_persisted_root(
    writes: &[RetainedWrite],
    persisted: &[bool],
) -> Option<(CheckpointTxg, InstanceGeneration)> {
    publishes_in(writes)
        .iter()
        .filter(|publish| persisted[publish.root])
        .map(|publish| {
            (
                CheckpointTxg(publish.checkpoint_txg),
                InstanceGeneration(publish.instance),
            )
        })
        .max()
}

/// 根槽 FUA 写里的根记录身份：实例代号在偏移 24（4 字节）、checkpoint_txg 在偏移 28（8 字节）。
fn root_identity_of_write(write: &RetainedWrite) -> (InstanceGeneration, CheckpointTxg) {
    assert_eq!(
        write.kind,
        StepKind::RootRecordFua,
        "被判的那条写要是根槽 FUA 写"
    );
    (
        InstanceGeneration(u32::from_le_bytes(
            write.bytes[24..28]
                .try_into()
                .expect("根记录的实例代号在偏移 24"),
        )),
        CheckpointTxg(u64::from_le_bytes(
            write.bytes[28..36]
                .try_into()
                .expect("根记录的 checkpoint_txg 在偏移 28"),
        )),
    )
}

/// 单版本形态：被判的那次根槽写出的那一代就是唯一带文件的版本。
pub fn evaluate_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    root_index: usize,
    expected_content: &[u8],
    tally: &mut Layer0Tally,
) -> RecoveryReport {
    let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
    let versions = [PublishedVersion {
        instance,
        checkpoint_txg,
        content: expected_content.to_vec(),
    }];
    evaluate_state_for_versions(base, writes, persisted, root_index, &versions, tally)
}

/// 评一个状态：跑两种 journal 政策的恢复，记进计数。`judged_root_index` 是被判的那次根槽 FUA 写（计「根槽已持久」的状态数用），
/// oracle 按 `versions` 判实际走的根该读出哪一版。
pub fn evaluate_state_for_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    tally: &mut Layer0Tally,
) -> RecoveryReport {
    let root_persisted = persisted[judged_root_index];
    let newest_persisted = newest_persisted_root(writes, &persisted);
    let image = CrashImage {
        base,
        writes,
        persisted,
    };
    let consulted = recover(&image, JournalPolicy::Consult);
    let ignored = recover(&image, JournalPolicy::Ignore);
    tally.states += 1;
    if root_persisted {
        tally.root_persisted_states += 1;
    }
    if consulted.outcome != ignored.outcome {
        tally.journal_differing_states += 1;
    }
    if consulted.journal.verification_passed + consulted.journal.verification_failed > 0 {
        tally.verification_ran_states += 1;
    }
    if consulted.journal.verification_failed > 0 {
        tally.verification_failed_states += 1;
    }
    match &consulted.outcome {
        RecoveryOutcome::NoFile { .. } => tally.no_file_states += 1,
        RecoveryOutcome::FileRead { .. } => tally.file_read_states += 1,
        RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
    }
    if let Some(reason) = oracle_violation_for_versions(
        &consulted.outcome,
        consulted.effective_root,
        newest_persisted,
        versions,
    ) {
        tally.violations += 1;
        if tally.first_violation.is_none() {
            let persisted_kinds: Vec<&str> = image
                .persisted
                .iter()
                .zip(writes)
                .filter(|(is_persisted, _)| **is_persisted)
                .map(|(_, write)| write.kind.name())
                .collect();
            tally.first_violation = Some(format!(
                "{reason}（持久的写：{}）",
                persisted_kinds.join("|")
            ));
        }
    }
    if let Some(reason) = oracle_violation_for_versions(
        &ignored.outcome,
        ignored.effective_root,
        newest_persisted,
        versions,
    ) {
        tally.ignored_violations += 1;
        if tally.first_ignored_violation.is_none() {
            tally.first_ignored_violation = Some(reason);
        }
    }
    for (invariant, verdict) in check_pool_image(&image) {
        match verdict {
            InvariantVerdict::Holds => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1
            }
            InvariantVerdict::Violated(detail) => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1;
                *tally.checker_violated_states.entry(invariant).or_insert(0) += 1;
                tally
                    .checker_first_violation
                    .entry(invariant)
                    .or_insert(detail);
            }
            InvariantVerdict::NotApplicable(_) => {
                *tally
                    .checker_not_applicable_states
                    .entry(invariant)
                    .or_insert(0) += 1;
            }
        }
    }
    let records = check_records(&image, consulted.effective_root);
    if records.root_without_record {
        tally.record_root_without_record += 1;
    }
    if records.claimed_state_missing_unit {
        tally.record_claimed_state_missing_unit += 1;
    }
    consulted
}

/// 单版本形态的枚举。
#[must_use]
pub fn enumerate_layer0_selecting(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    root_index: usize,
    expected_content: &[u8],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> Layer0Tally {
    let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
    let versions = [PublishedVersion {
        instance,
        checkpoint_txg,
        content: expected_content.to_vec(),
    }];
    enumerate_layer0_selecting_versions(base, writes, segments, root_index, &versions, expand)
}

/// 层 0 枚举的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。门禁 54 号显式传进来。
pub const LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_LAYER0_THREADS";

/// 默认切法的片数下限：线程数为 1 时全量用例也按片报进度。
const LAYER0_MINIMUM_SLICE_COUNT: u64 = 64;
/// 默认切法里每个工作线程摊到的片数：越往后的状态历史越长、越贵，片切得比线程多，先跑完的线程接着领下一片。
const LAYER0_SLICES_PER_WORKER_THREAD: u64 = 16;
/// 默认切法里每片的状态数下限：平时 `cargo test` 里几十个状态的枚举只切成几片，进度行不刷屏。
const LAYER0_MINIMUM_STATES_PER_SLICE: u64 = 16;

/// 工作线程数是从哪来的：与实际起的线程数一起打进 `LAYER0_PARALLEL_START` / `LAYER0_PARALLEL_FINISHED` 两行（门禁 54 号拿实际起的线程数判「没显式设成 1、机器多于 1 核却只用了 1 个线程」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer0WorkerThreadsSource {
    /// 环境变量 [`LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    EnvironmentVariable,
    /// 没设环境变量，取 `available_parallelism`。
    AvailableParallelism,
    /// 没设环境变量，`available_parallelism` 也报不出来：只用 1 个线程，照实报出来。
    AvailableParallelismUnknown,
    /// 调用方在代码里直接给的（用例拿不同切法对拍）。
    GivenByCaller,
}

impl Layer0WorkerThreadsSource {
    fn name(self) -> &'static str {
        match self {
            Self::EnvironmentVariable => "environment_variable",
            Self::AvailableParallelism => "available_parallelism",
            Self::AvailableParallelismUnknown => "available_parallelism_unknown",
            Self::GivenByCaller => "given_by_caller",
        }
    }
}

/// 每片几个状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer0SliceLength {
    /// 片数取 max(64, 16 × 工作线程数)，每片至少 16 个状态。
    ScaledToWorkerThreads,
    /// 每片固定这么多个状态（用例把切法推到两头：每片 1 个，或整条流 1 片）。
    StatesPerSlice(NonZeroU64),
}

/// 层 0 按状态序号区间切片、多线程跑：几个工作线程、每片几个状态。切法与线程数只影响跑得多快，不影响计数与「第一处违例」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layer0Parallelism {
    pub worker_threads: NonZeroUsize,
    pub worker_threads_source: Layer0WorkerThreadsSource,
    pub slice_length: Layer0SliceLength,
}

impl Layer0Parallelism {
    /// 线程数取环境变量 [`LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`]，没设就取 `available_parallelism`；片长按线程数定。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数：配错了就停，不悄悄退回单线程。
    #[must_use]
    pub fn from_environment() -> Self {
        Self::from_environment_value(
            std::env::var(LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE),
            std::thread::available_parallelism,
        )
    }

    /// [`Self::from_environment`] 的判定本身：环境变量读到什么、`available_parallelism` 报什么都由调用方给（用例不改进程的环境变量）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）。
    #[must_use]
    fn from_environment_value(
        environment_value: Result<String, std::env::VarError>,
        available_parallelism: impl FnOnce() -> std::io::Result<NonZeroUsize>,
    ) -> Self {
        let (worker_threads, worker_threads_source) = match environment_value {
            Ok(text) => (
                text.parse::<NonZeroUsize>().unwrap_or_else(|error| {
                    panic!(
                        "{LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}：{error}"
                    )
                }),
                Layer0WorkerThreadsSource::EnvironmentVariable,
            ),
            Err(std::env::VarError::NotPresent) => match available_parallelism() {
                Ok(available) => (available, Layer0WorkerThreadsSource::AvailableParallelism),
                Err(_unavailable) => (
                    NonZeroUsize::MIN,
                    Layer0WorkerThreadsSource::AvailableParallelismUnknown,
                ),
            },
            Err(std::env::VarError::NotUnicode(raw)) => panic!(
                "{LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到的不是 UTF-8：{raw:?}"
            ),
        };
        Self {
            worker_threads,
            worker_threads_source,
            slice_length: Layer0SliceLength::ScaledToWorkerThreads,
        }
    }
}

/// 枚举次序里每个状态的序号怎么落到「第几段、段内哪个子集」：次序与单线程逐段逐个子集走相同——
/// 前面的段全持久 + 当前段任意真子集（子集掩码从 0 数到 2^|段| − 2），最后再加全部持久那一个状态。
struct Layer0StatePlan<'segments> {
    segments: &'segments [Vec<usize>],
    /// 每一段展开出来的状态的序号区间，首尾相接、随段号递增；不展开的段是空区间。
    state_ranges_by_segment: Vec<Range<u64>>,
    /// 状态总数：各段展开出来的，再加最后全部持久那一个。
    state_count: u64,
}

impl<'segments> Layer0StatePlan<'segments> {
    /// `expand` 只在调用线程上逐段问一次，所以它不必能跨线程。
    fn new(segments: &'segments [Vec<usize>], expand: &dyn Fn(usize, &[usize]) -> bool) -> Self {
        let mut next_ordinal = 0u64;
        let state_ranges_by_segment = segments
            .iter()
            .enumerate()
            .map(|(segment_index, segment)| {
                let first_ordinal = next_ordinal;
                if expand(segment_index, segment) {
                    next_ordinal += (1u64 << segment.len()) - 1;
                }
                first_ordinal..next_ordinal
            })
            .collect();
        Self {
            segments,
            state_ranges_by_segment,
            state_count: next_ordinal + 1,
        }
    }

    /// 序号落在哪一段；等于段数说明是最后全部持久那一个状态。
    fn segment_of_state(&self, ordinal: u64) -> usize {
        self.state_ranges_by_segment
            .partition_point(|state_range| state_range.end <= ordinal)
    }

    /// 这个状态里持久了的写：所在的段之前每一段整段持久（展不展开都一样），所在的段按段内子集掩码（第 k 位对应段里第 k 个写）。
    fn persisted_writes_of_state(&self, ordinal: u64, write_count: usize) -> Vec<bool> {
        let segment_of_state = self.segment_of_state(ordinal);
        let mut persisted = vec![false; write_count];
        for segment in &self.segments[..segment_of_state] {
            for write_index in segment {
                persisted[*write_index] = true;
            }
        }
        if let Some(segment) = self.segments.get(segment_of_state) {
            let subset_mask = ordinal - self.state_ranges_by_segment[segment_of_state].start;
            for (bit, write_index) in segment.iter().enumerate() {
                if subset_mask & (1 << bit) != 0 {
                    persisted[*write_index] = true;
                }
            }
        }
        persisted
    }

    /// 进度行里的段号：最后全部持久那一个状态不在任何一段里，报成 `all_persisted`。
    fn segment_label(&self, segment_index: usize) -> String {
        if segment_index < self.segments.len() {
            segment_index.to_string()
        } else {
            "all_persisted".to_string()
        }
    }
}

/// 把 [0, `state_count`) 按序号切成首尾相接的区间。
fn state_slices(state_count: u64, parallelism: &Layer0Parallelism) -> Vec<Range<u64>> {
    let states_per_slice = match parallelism.slice_length {
        Layer0SliceLength::ScaledToWorkerThreads => {
            let worker_threads =
                u64::try_from(parallelism.worker_threads.get()).expect("线程数装得进 u64");
            let slice_count = LAYER0_MINIMUM_SLICE_COUNT
                .max(worker_threads.saturating_mul(LAYER0_SLICES_PER_WORKER_THREAD));
            state_count
                .div_ceil(slice_count)
                .max(LAYER0_MINIMUM_STATES_PER_SLICE)
        }
        Layer0SliceLength::StatesPerSlice(states_per_slice) => states_per_slice.get(),
    };
    (0..state_count.div_ceil(states_per_slice))
        .map(|slice_index| {
            let first_ordinal = slice_index * states_per_slice;
            let end_ordinal = first_ordinal
                .saturating_add(states_per_slice)
                .min(state_count);
            first_ordinal..end_ordinal
        })
        .collect()
}

/// 逐状态的观察者：拿到崩溃镜像与看 journal 那一遍恢复的报告。
pub type Layer0StateObserver<'observer> =
    &'observer mut dyn FnMut(&CrashImage<'_>, &RecoveryReport);

/// 工作线程评状态时 panic，就把旗子立起来：别的工作线程看到旗子不再领新片，调用线程不必等整条流跑完才知道红了。
struct RaiseFlagWhenPanicking<'flag>(&'flag AtomicBool);

impl Drop for RaiseFlagWhenPanicking<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

/// 工作线程要不要把每个状态的持久集合与看 journal 那一遍恢复的报告带回调用线程。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StateReportRetention {
    /// 有观察者：带回去，由调用线程按序号交给它。
    HandEachStateToObserver,
    /// 没有观察者：只带计数回去（全量流上两百多万个报告不必留）。
    CountOnly,
}

/// 一片跑完交回调用线程的东西。
struct FinishedSlice {
    slice_index: usize,
    tally: Layer0Tally,
    /// 按序号排好的（持久集合，看 journal 那一遍恢复的报告）；`CountOnly` 时是空的。
    observed_states: Vec<(Vec<bool>, RecoveryReport)>,
}

/// 在工作线程上跑一片：每个状态自己建崩溃镜像、自己记进这一片的计数；基线与写表只读、各线程共用。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：基线、写表、状态计划、这一片的区间、被判的根、版本表、要不要带回报告"
)]
fn evaluate_state_slice(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    plan: &Layer0StatePlan<'_>,
    slice_index: usize,
    slice: Range<u64>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    retention: StateReportRetention,
) -> FinishedSlice {
    let mut tally = Layer0Tally::default();
    let mut observed_states = Vec::new();
    for ordinal in slice {
        let persisted = plan.persisted_writes_of_state(ordinal, writes.len());
        match retention {
            StateReportRetention::HandEachStateToObserver => {
                let consulted_report = evaluate_state_for_versions(
                    base,
                    writes,
                    persisted.clone(),
                    judged_root_index,
                    versions,
                    &mut tally,
                );
                observed_states.push((persisted, consulted_report));
            }
            StateReportRetention::CountOnly => {
                evaluate_state_for_versions(
                    base,
                    writes,
                    persisted,
                    judged_root_index,
                    versions,
                    &mut tally,
                );
            }
        }
    }
    FinishedSlice {
        slice_index,
        tally,
        observed_states,
    }
}

/// 枚举：前面的段全持久 + 当前段任意真子集，最后再加全部持久那一个状态；`expand` 决定哪一段展开子集
/// （不展开的段只以整段持久进入后面的状态，平时 `cargo test` 里跳过 18 个写那一段就靠它）。`judged_root_index` 是被判的那次根槽 FUA 写。
/// 按状态序号区间切片、多线程跑，线程数见 [`Layer0Parallelism::from_environment`]。
#[must_use]
pub fn enumerate_layer0_selecting_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> Layer0Tally {
    enumerate_layer0_in_state_slices(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expand,
        Layer0Parallelism::from_environment(),
        None,
    )
}

/// 同上，每个状态评完之后把这个崩溃镜像与看 journal 那一遍恢复的报告交给 `observe_state`：
/// 用例按自己独立算的谓词逐状态核恢复（预置的残留记录该不该施加、改坏 tail 之后终态与没改坏的是否逐项相等）。
/// 观察者在调用线程上、按状态序号从小到大调用，次序与单线程逐个跑相同，所以它不必能跨线程。
#[must_use]
pub fn enumerate_layer0_selecting_versions_observing_each_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
    observe_state: &mut dyn FnMut(&CrashImage<'_>, &RecoveryReport),
) -> Layer0Tally {
    enumerate_layer0_in_state_slices(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expand,
        Layer0Parallelism::from_environment(),
        Some(observe_state),
    )
}

/// 层 0 枚举的本体：把 [0, 状态数) 按 `parallelism` 切成首尾相接的序号区间，工作线程按片号从小到大领片、各自跑完交回；
/// 调用线程收到一片就打一行 `LAYER0_PROGRESS`（片号、序号区间、段号、已跑完的片数与状态数），再按片号从小到大并计数、调观察者。
/// 计数按片的次序相加，「第一处违例」取序号最小的那一处（`Layer0Tally` 的并片），结果与线程数、切法无关。
/// 开跑与跑完各打一行 `LAYER0_PARALLEL_START` / `LAYER0_PARALLEL_FINISHED`（状态数、片数、实际起的工作线程数、线程数从哪来、耗时）。
///
/// # Panics
/// 某个工作线程在评状态时 panic（恢复或 checker 里的断言：别的线程不再领新片，手上那一片跑完就退）；观察者 panic；
/// 有一片领了却没交回（不变量被破坏）。
#[allow(
    clippy::too_many_arguments,
    reason = "前六个与单线程时的枚举相同，多出来的是切法与观察者"
)]
#[must_use]
pub fn enumerate_layer0_in_state_slices(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
    parallelism: Layer0Parallelism,
    mut observe_state: Option<Layer0StateObserver<'_>>,
) -> Layer0Tally {
    let plan = Layer0StatePlan::new(segments, expand);
    let slices = state_slices(plan.state_count, &parallelism);
    let spawned_worker_threads = parallelism.worker_threads.get().min(slices.len());
    let retention = match observe_state {
        Some(_) => StateReportRetention::HandEachStateToObserver,
        None => StateReportRetention::CountOnly,
    };
    let started = Instant::now();
    println!(
        "LAYER0_PARALLEL_START states={} slices={} states_per_slice={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={}",
        plan.state_count,
        slices.len(),
        slices.first().map_or(0, |slice| slice.end - slice.start),
        parallelism.worker_threads,
        parallelism.worker_threads_source.name()
    );
    let next_slice_index = AtomicUsize::new(0);
    let some_worker_thread_panicked = AtomicBool::new(false);
    let (tally, merged_slice_count) = std::thread::scope(|scope| {
        // 收发两端都建在这个闭包里：观察者 panic 时接收端随闭包一起丢掉，工作线程下一次交片就发不出去、随即退出。
        let (finished_slice_sender, finished_slice_receiver) = mpsc::channel::<FinishedSlice>();
        for _ in 0..spawned_worker_threads {
            let finished_slice_sender = finished_slice_sender.clone();
            let plan = &plan;
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            let some_worker_thread_panicked = &some_worker_thread_panicked;
            scope.spawn(move || loop {
                let _raise_the_flag_if_this_thread_panics =
                    RaiseFlagWhenPanicking(some_worker_thread_panicked);
                if some_worker_thread_panicked.load(Ordering::Relaxed) {
                    break;
                }
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let finished = evaluate_state_slice(
                    base,
                    writes,
                    plan,
                    slice_index,
                    slice.clone(),
                    judged_root_index,
                    versions,
                    retention,
                );
                // 发不出去说明调用线程已经不收了（观察者 panic）：不再领新的片。
                if finished_slice_sender.send(finished).is_err() {
                    break;
                }
            });
        }
        // 只留工作线程手里的发送端：它们都退出之后，下面的接收循环才结束。
        drop(finished_slice_sender);
        let mut merged = Layer0Tally::default();
        let mut waiting_for_earlier_slices: BTreeMap<usize, FinishedSlice> = BTreeMap::new();
        let mut next_slice_to_merge = 0usize;
        let mut finished_states = 0u64;
        for (finished_slice_count, finished) in finished_slice_receiver.iter().enumerate() {
            let slice = &slices[finished.slice_index];
            finished_states += slice.end - slice.start;
            println!(
                "LAYER0_PROGRESS slice={}/{} states=[{},{}) segments={}..={} finished_slices={}/{} finished_states={finished_states}/{} elapsed_seconds={:.1}",
                finished.slice_index + 1,
                slices.len(),
                slice.start,
                slice.end,
                plan.segment_label(plan.segment_of_state(slice.start)),
                plan.segment_label(plan.segment_of_state(slice.end - 1)),
                finished_slice_count + 1,
                slices.len(),
                plan.state_count,
                started.elapsed().as_secs_f64()
            );
            waiting_for_earlier_slices.insert(finished.slice_index, finished);
            while let Some(in_order) = waiting_for_earlier_slices.remove(&next_slice_to_merge) {
                if let Some(observe) = observe_state.as_mut() {
                    for (persisted, consulted_report) in in_order.observed_states {
                        observe(
                            &CrashImage {
                                base,
                                writes,
                                persisted,
                            },
                            &consulted_report,
                        );
                    }
                }
                merged.absorb_following_slice(in_order.tally);
                next_slice_to_merge += 1;
            }
        }
        (merged, next_slice_to_merge)
    });
    assert_eq!(
        merged_slice_count,
        slices.len(),
        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "LAYER0_PARALLEL_FINISHED states={} slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={} elapsed_seconds={:.1}",
        plan.state_count,
        slices.len(),
        parallelism.worker_threads,
        parallelism.worker_threads_source.name(),
        started.elapsed().as_secs_f64()
    );
    tally
}

/// 全量、多版本：每一段都展开。
#[must_use]
pub fn enumerate_layer0_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
) -> Layer0Tally {
    enumerate_layer0_selecting_versions(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &|_segment_index, _segment| true,
    )
}

/// 全量：每一段都展开。
#[must_use]
pub fn enumerate_layer0(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    root_index: usize,
    expected_content: &[u8],
) -> Layer0Tally {
    enumerate_layer0_selecting(
        base,
        writes,
        segments,
        root_index,
        expected_content,
        &|_segment_index, _segment| true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_device_reads_zero_for_holes_and_flip_byte_only_touches_one_sector() {
        let mut device = SparseDevice::default();
        device.write(DeviceOffsetInBytes(1024), &[7u8; 1024]);
        assert_eq!(device.read(DeviceOffsetInBytes(512), 512), vec![0u8; 512]);
        assert_eq!(device.read(DeviceOffsetInBytes(1024), 512), vec![7u8; 512]);
        assert_eq!(
            device.written_sectors_in(DeviceOffsetInBytes(0), 4096),
            vec![2, 3]
        );
        let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 20);
        pool.devices
            .get_mut(&DeviceIdentity(0))
            .expect("盘")
            .write(DeviceOffsetInBytes(0), &[1u8; 1024]);
        pool.flip_byte(DeviceIdentity(0), DeviceOffsetInBytes(0), 600);
        let read_back =
            PoolReader::read(&pool, DeviceIdentity(0), DeviceOffsetInBytes(0), 1024).expect("读");
        assert_eq!(read_back[600], 0xfe);
        assert!(read_back
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 600 || *byte == 1));
        assert_eq!(closed_form_state_count(&[vec![0, 1], vec![2]]), 1 + 3 + 1);
    }

    /// 单线程逐段逐个子集走（并行之前 `enumerate_layer0_selecting_versions_observing_each_state` 的次序）给出的持久集合，按次序排好。
    fn persisted_sets_walking_segment_by_segment(
        segments: &[Vec<usize>],
        write_count: usize,
        expand: &dyn Fn(usize, &[usize]) -> bool,
    ) -> Vec<Vec<bool>> {
        let mut persisted_sets = Vec::new();
        let mut persisted_before = vec![false; write_count];
        for (segment_index, segment) in segments.iter().enumerate() {
            if expand(segment_index, segment) {
                for subset_mask in 0..(1u64 << segment.len()) - 1 {
                    let mut persisted = persisted_before.clone();
                    for (bit, write_index) in segment.iter().enumerate() {
                        if subset_mask & (1 << bit) != 0 {
                            persisted[*write_index] = true;
                        }
                    }
                    persisted_sets.push(persisted);
                }
            }
            for write_index in segment {
                persisted_before[*write_index] = true;
            }
        }
        persisted_sets.push(persisted_before);
        persisted_sets
    }

    /// 按序号取状态（并行切片靠它）与逐段逐个子集走，给出同一串持久集合：展开的段夹着不展开的段、不展开的段在头上和尾上都算。
    #[test]
    fn the_state_plan_hands_out_the_same_persisted_sets_in_the_same_order_as_walking_segment_by_segment(
    ) {
        let segments = vec![
            vec![0, 1],
            vec![2],
            vec![3, 4, 5],
            vec![6, 7],
            vec![8, 9, 10, 11],
            vec![12],
        ];
        let write_count = 13;
        let every_segment = |_segment_index: usize, _segment: &[usize]| true;
        let skip_three_write_segment_and_the_ends = |segment_index: usize, segment: &[usize]| {
            segment.len() != 3 && segment_index != 0 && segment_index != 5
        };
        let only_the_four_write_segment =
            |_segment_index: usize, segment: &[usize]| segment.len() == 4;
        let assert_plan_matches_the_walk = |expand: &dyn Fn(usize, &[usize]) -> bool| {
            let walked = persisted_sets_walking_segment_by_segment(&segments, write_count, expand);
            let plan = Layer0StatePlan::new(&segments, expand);
            assert_eq!(
                plan.state_count,
                u64::try_from(walked.len()).expect("状态数"),
                "状态数与逐段走的相同"
            );
            let by_ordinal: Vec<Vec<bool>> = (0..plan.state_count)
                .map(|ordinal| plan.persisted_writes_of_state(ordinal, write_count))
                .collect();
            assert_eq!(by_ordinal, walked, "第 k 个状态就是逐段走到的第 k 个");
        };
        assert_plan_matches_the_walk(&every_segment);
        assert_plan_matches_the_walk(&skip_three_write_segment_and_the_ends);
        assert_plan_matches_the_walk(&only_the_four_write_segment);
    }

    /// 切片首尾相接、从 0 起、到状态数止、一片都不空：丢一片或两片重叠，这里与用例里「状态数等于闭式」的断言都红。
    /// 默认切法下全量两条流切出来的片数不少于线程数（每个线程都领得到片）。
    #[test]
    fn state_slices_cover_every_state_exactly_once_in_ordinal_order() {
        let thread_counts = [1usize, 3, 32, 200];
        let fixed_lengths = [1u64, 7, 16, u64::MAX];
        for state_count in [1u64, 2, 15, 16, 17, 22, 108, 1000, 262_165, 2_104_413] {
            let mut parallelisms: Vec<Layer0Parallelism> = thread_counts
                .iter()
                .map(|worker_threads| Layer0Parallelism {
                    worker_threads: NonZeroUsize::new(*worker_threads).expect("非 0"),
                    worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
                    slice_length: Layer0SliceLength::ScaledToWorkerThreads,
                })
                .collect();
            parallelisms.extend(
                fixed_lengths
                    .iter()
                    .map(|states_per_slice| Layer0Parallelism {
                        worker_threads: NonZeroUsize::MIN,
                        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
                        slice_length: Layer0SliceLength::StatesPerSlice(
                            NonZeroU64::new(*states_per_slice).expect("非 0"),
                        ),
                    }),
            );
            for parallelism in parallelisms {
                let slices = state_slices(state_count, &parallelism);
                let mut next_ordinal = 0u64;
                for slice in &slices {
                    assert_eq!(
                        slice.start, next_ordinal,
                        "{state_count} 个状态、{parallelism:?}：片首接上一片的尾"
                    );
                    assert!(slice.end > slice.start, "{parallelism:?}：没有空片");
                    next_ordinal = slice.end;
                }
                assert_eq!(
                    next_ordinal, state_count,
                    "{parallelism:?}：最后一片止于状态数"
                );
                if state_count >= 262_165 {
                    assert!(
                        slices.len() >= parallelism.worker_threads.get()
                            || matches!(
                                parallelism.slice_length,
                                Layer0SliceLength::StatesPerSlice(_)
                            ),
                        "{state_count} 个状态、{parallelism:?}：片数 {} 不少于线程数",
                        slices.len()
                    );
                }
            }
        }
    }

    /// 线程数：环境变量设了就用它（不再问 `available_parallelism`）；没设取 `available_parallelism`；那个也报不出来只用 1 个线程、照实报来源。
    #[test]
    fn worker_threads_come_from_the_environment_variable_before_available_parallelism() {
        let thirty_two = || Ok(NonZeroUsize::new(32).expect("非 0"));
        let from_variable = Layer0Parallelism::from_environment_value(Ok("4".to_string()), || {
            panic!("环境变量设了就不该再问 available_parallelism")
        });
        assert_eq!(
            (
                from_variable.worker_threads.get(),
                from_variable.worker_threads_source
            ),
            (4, Layer0WorkerThreadsSource::EnvironmentVariable)
        );
        let explicit_single =
            Layer0Parallelism::from_environment_value(Ok("1".to_string()), thirty_two);
        assert_eq!(
            (
                explicit_single.worker_threads.get(),
                explicit_single.worker_threads_source
            ),
            (1, Layer0WorkerThreadsSource::EnvironmentVariable),
            "显式设成 1 就是 1"
        );
        let unset = Layer0Parallelism::from_environment_value(
            Err(std::env::VarError::NotPresent),
            thirty_two,
        );
        assert_eq!(
            (unset.worker_threads.get(), unset.worker_threads_source),
            (32, Layer0WorkerThreadsSource::AvailableParallelism)
        );
        let unknown =
            Layer0Parallelism::from_environment_value(Err(std::env::VarError::NotPresent), || {
                Err(std::io::Error::other("平台报不出核数"))
            });
        assert_eq!(
            (unknown.worker_threads.get(), unknown.worker_threads_source),
            (1, Layer0WorkerThreadsSource::AvailableParallelismUnknown)
        );
        assert_eq!(unset.slice_length, Layer0SliceLength::ScaledToWorkerThreads);
    }

    /// 环境变量设成 0：停下，不悄悄退回单线程。不写成 `#[should_panic]`：那样 libtest 的输出行带「- should panic」，门禁 59 号认不出这条测试红没红。
    #[test]
    fn zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back() {
        let outcome = std::panic::catch_unwind(|| {
            Layer0Parallelism::from_environment_value(Ok("0".to_string()), || {
                Ok(NonZeroUsize::new(32).expect("非 0"))
            })
        });
        let panic_payload = outcome.expect_err("设成 0 要停下，不许退回 1 个线程接着跑");
        let panic_message = panic_payload
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_default();
        assert!(
            panic_message.contains("SINGLEFS_LAYER0_THREADS 要是正整数"),
            "停下时说清是哪个环境变量配错了：{panic_message}"
        );
    }

    /// 并片：计数相加，「第一处」取前面那一片的；前面那一片没有才取后面的。
    #[test]
    fn absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation() {
        let mut earlier = Layer0Tally {
            states: 3,
            violations: 1,
            root_persisted_states: 0,
            no_file_states: 3,
            file_read_states: 0,
            failed_states: 0,
            journal_differing_states: 0,
            verification_ran_states: 0,
            verification_failed_states: 0,
            first_violation: Some("前面那一片的".to_string()),
            ignored_violations: 0,
            first_ignored_violation: None,
            record_root_without_record: 0,
            record_claimed_state_missing_unit: 0,
            checker_evaluated_states: BTreeMap::from([("I-1.1", 3)]),
            checker_violated_states: BTreeMap::from([("I-1.1", 1)]),
            checker_first_violation: BTreeMap::from([("I-1.1", "前面那一片的 I-1.1".to_string())]),
            checker_not_applicable_states: BTreeMap::from([("I-3.1", 3)]),
        };
        let following = Layer0Tally {
            states: 5,
            violations: 2,
            root_persisted_states: 5,
            no_file_states: 0,
            file_read_states: 5,
            failed_states: 0,
            journal_differing_states: 1,
            verification_ran_states: 1,
            verification_failed_states: 0,
            first_violation: Some("后面那一片的".to_string()),
            ignored_violations: 1,
            first_ignored_violation: Some("后面那一片的 Ignore".to_string()),
            record_root_without_record: 1,
            record_claimed_state_missing_unit: 1,
            checker_evaluated_states: BTreeMap::from([("I-1.1", 5), ("I-3.1", 5)]),
            checker_violated_states: BTreeMap::from([("I-1.1", 2), ("I-3.1", 1)]),
            checker_first_violation: BTreeMap::from([
                ("I-1.1", "后面那一片的 I-1.1".to_string()),
                ("I-3.1", "后面那一片的 I-3.1".to_string()),
            ]),
            checker_not_applicable_states: BTreeMap::new(),
        };
        earlier.absorb_following_slice(following);
        assert_eq!(
            (
                earlier.states,
                earlier.violations,
                earlier.ignored_violations,
                earlier.no_file_states,
                earlier.file_read_states,
                earlier.root_persisted_states,
                earlier.journal_differing_states,
                earlier.verification_ran_states,
                earlier.record_root_without_record,
                earlier.record_claimed_state_missing_unit,
            ),
            (8, 3, 1, 3, 5, 5, 1, 1, 1, 1),
            "计数逐项相加"
        );
        assert_eq!(earlier.first_violation.as_deref(), Some("前面那一片的"));
        assert_eq!(
            earlier.first_ignored_violation.as_deref(),
            Some("后面那一片的 Ignore"),
            "前面那一片没有，取后面的"
        );
        assert_eq!(
            earlier.checker_evaluated_states,
            BTreeMap::from([("I-1.1", 8), ("I-3.1", 5)])
        );
        assert_eq!(
            earlier.checker_violated_states,
            BTreeMap::from([("I-1.1", 3), ("I-3.1", 1)])
        );
        assert_eq!(
            earlier.checker_not_applicable_states,
            BTreeMap::from([("I-3.1", 3)])
        );
        assert_eq!(
            earlier.checker_first_violation,
            BTreeMap::from([
                ("I-1.1", "前面那一片的 I-1.1".to_string()),
                ("I-3.1", "后面那一片的 I-3.1".to_string()),
            ]),
            "每条不变量的第一处各取最早有的那一片"
        );
    }
}

#[cfg(test)]
mod oracle_instance_tests {
    use super::{oracle_violation_for_versions, PublishedVersion};
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_core::recovery::RecoveryOutcome;

    /// 两条同 txg 不同实例的版本（设备失而复得、或回退实例与被抛弃实例同时在环里会造出来）。
    fn two_instances_at_txg_seven() -> Vec<PublishedVersion> {
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(7),
                content: b"written by instance one".to_vec(),
            },
            PublishedVersion {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(7),
                content: b"written by instance two".to_vec(),
            },
        ]
    }

    /// D22（单元原子性怎么合成） 已定项 7：txg 平局按实例代号高者赢。实例 2 的第 7 代根已持久而恢复落在实例 1 的第 7 代根上是一次退代。
    #[test]
    fn landing_on_the_lower_instance_of_the_same_txg_is_a_violation() {
        let outcome = RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(7)),
            content: b"written by instance one".to_vec(),
        };
        let violation = oracle_violation_for_versions(
            &outcome,
            Some((InstanceGeneration(1), CheckpointTxg(7))),
            Some((CheckpointTxg(7), InstanceGeneration(2))),
            &two_instances_at_txg_seven(),
        );
        assert!(
            violation.is_some(),
            "只按 txg 比会把实例 1 的第 7 代当成最新的"
        );
    }

    /// 走到一个没有版本的更新的根上报「没有文件」，而更旧的根下面有文件：违例，不许因为查不到版本就放过。
    #[test]
    fn newer_root_without_any_version_reporting_no_file_is_violation() {
        let outcome = RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(9)),
        };
        assert!(oracle_violation_for_versions(
            &outcome,
            Some((InstanceGeneration(1), CheckpointTxg(9))),
            Some((CheckpointTxg(7), InstanceGeneration(2))),
            &two_instances_at_txg_seven(),
        )
        .is_some());
        // 比每条版本都旧的根（暖机）报没有文件照旧不是违例。
        let warm_up = RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2)),
        };
        assert_eq!(
            oracle_violation_for_versions(
                &warm_up,
                Some((InstanceGeneration(1), CheckpointTxg(2))),
                Some((CheckpointTxg(2), InstanceGeneration(1))),
                &two_instances_at_txg_seven(),
            ),
            None
        );
    }

    /// 同 txg 的两条版本是两条版本：落在实例 2 上读出实例 2 的内容不是违例。
    #[test]
    fn the_same_txg_from_two_instances_are_two_versions() {
        let outcome = RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(7)),
            content: b"written by instance two".to_vec(),
        };
        assert_eq!(
            oracle_violation_for_versions(
                &outcome,
                Some((InstanceGeneration(2), CheckpointTxg(7))),
                Some((CheckpointTxg(7), InstanceGeneration(2))),
                &two_instances_at_txg_seven(),
            ),
            None
        );
    }
}
