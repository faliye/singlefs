//! 层 0 崩溃点重放（里程碑步 7，D13（验证路线） 已定项 4）：拿录制流在内存里重建镜像，屏障与 FUA 切段、段内任意整写子集、
//! 撕裂态并进「没持久」、没持久的位置放旧字节、两盘各算；每个状态跑一遍步 6 的恢复，拿 oracle（E77（发布的持久顺序） 判据 1）判。
//! 恢复只通过 [`PoolReader`] 读盘，所以崩溃镜像不落文件：基线是稀疏的扇区图，其上叠一层「这一状态里持久了的写」。

use std::collections::BTreeMap;

use std::collections::BTreeSet;

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

/// 枚举：前面的段全持久 + 当前段任意真子集，最后再加全部持久那一个状态；`expand` 决定哪一段展开子集
/// （不展开的段只以整段持久进入后面的状态，平时 `cargo test` 里跳过 18 个写那一段就靠它）。`judged_root_index` 是被判的那次根槽 FUA 写。
#[must_use]
pub fn enumerate_layer0_selecting_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> Layer0Tally {
    enumerate_layer0_selecting_versions_observing_each_state(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expand,
        &mut |_crash_image, _consulted_report| {},
    )
}

/// 同上，每个状态评完之后把这个崩溃镜像与看 journal 那一遍恢复的报告交给 `observe_state`：
/// 用例按自己独立算的谓词逐状态核恢复（预置的残留记录该不该施加、改坏 tail 之后终态与没改坏的是否逐项相等）。
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
    let mut tally = Layer0Tally::default();
    let mut persisted_before = vec![false; writes.len()];
    for (segment_index, segment) in segments.iter().enumerate() {
        if expand(segment_index, segment) {
            let full_mask = (1u64 << segment.len()) - 1;
            for mask in 0..full_mask {
                let mut persisted = persisted_before.clone();
                for (bit, write_index) in segment.iter().enumerate() {
                    if mask & (1 << bit) != 0 {
                        persisted[*write_index] = true;
                    }
                }
                let consulted_report = evaluate_state_for_versions(
                    base,
                    writes,
                    persisted.clone(),
                    judged_root_index,
                    versions,
                    &mut tally,
                );
                observe_state(
                    &CrashImage {
                        base,
                        writes,
                        persisted,
                    },
                    &consulted_report,
                );
            }
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    let consulted_report = evaluate_state_for_versions(
        base,
        writes,
        persisted_before.clone(),
        judged_root_index,
        versions,
        &mut tally,
    );
    observe_state(
        &CrashImage {
            base,
            writes,
            persisted: persisted_before,
        },
        &consulted_report,
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
