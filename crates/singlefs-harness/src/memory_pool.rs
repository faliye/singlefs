//! 内存池与它上面的崩溃镜像（harness 档的脚手架，D13（验证路线） 已定项 15）：稀疏设备、`MemoryPool`、录制流切成的保留写与段、
//! `CrashImage`（基镜像 + 持久集合）、从根槽写里读出根身份与卸载标记，以及记录核对器与多版本 oracle 交回的两个结果类型
//! （`RecordCheck`、`PublishedVersion`）。枚举崩溃状态、判记录与 oracle 的代码在 checker 档的 `singlefs_checker_tier::crash`。

use std::collections::BTreeMap;

use std::collections::BTreeSet;

use singlefs_checker::image::ImageReader;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};
use singlefs_core::recovery::PoolReader;
use singlefs_core::system_configuration::unit_area_start_slot_recorded_in_the_slot;
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
};

use crate::segments::{FixedGeometry, SegmentAfterOperation, SegmentClosingRule, StepKind};
use crate::{RecordedEntrySpan, RecordedOperationKind, RecordedPublishEntry, RetainedOperation};
use singlefs_core::root_record::{UnmountMarker, ROOT_RECORD_FLAG_UNMOUNT_MARKER};
use singlefs_core::system_configuration::SystemConfiguration;

/// 稀疏设备的扇区宽：第一版测试镜像的 physical_block_size。
pub const SECTOR_BYTES: u64 = 512;

/// 一块盘：只存写过的扇区，没写过的读出全 0。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SparseDevice {
    sectors: BTreeMap<u64, Vec<u8>>,
}

impl SparseDevice {
    /// 写过的扇区（扇区号 → 512 字节），只读：层 0 枚举器按它判基镜像在某段有没有非零字节、按它算续跑的输入摘要。
    #[must_use]
    pub fn written_sectors(&self) -> &BTreeMap<u64, Vec<u8>> {
        &self.sectors
    }

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
    /// 整段清零：把 `[offset, offset + length)` 里记着的扇区**删掉**，不是插进 length / 512 个全 0 扇区。
    ///
    /// 稀疏设备里「没记着的扇区」读出来就是全 0，两种做法读回的字节逐位相同；差别在别处：
    /// ① 内存——mkfs 一次清 768 MiB，插进去就是每块盘 150 万条 512 字节的项；
    /// ② `written_sectors_in`——它是「盘上哪些地方有东西」的线索（journal 记录槽、单元槽的候选集都从它来），
    ///    把整环标成写过，候选集就从几条变成 150 万条。清零之后那一段本来就什么都没有，删掉才是它的意思。
    pub fn zero_fill(&mut self, offset: DeviceOffsetInBytes, length: u64) {
        assert!(offset.0.is_multiple_of(SECTOR_BYTES), "清零要按扇区对齐");
        assert!(length.is_multiple_of(SECTOR_BYTES), "清零长度要是整扇区");
        let first_sector = offset.0 / SECTOR_BYTES;
        let end_sector = (offset.0 + length) / SECTOR_BYTES;
        let inside: Vec<u64> = self
            .sectors
            .range(first_sector..end_sector)
            .map(|(sector, _)| *sector)
            .collect();
        for sector in inside {
            self.sectors.remove(&sector);
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

/// 一次块设备请求合不合设备的契约：偏移与长度按物理块对齐、`offset + length` 不越过设备末尾（加法溢出也算越过）；
/// 不合时交回的错误成员与字段和文件后端的一样（`OutOfRange` / `Unaligned`）。
///
/// 判据照 core `block_device.rs` 的 `check_aligned_and_in_range` 写（那一份是私有的，harness 拿不到）；两边对同一组请求判得一样，
/// 由用例 `tests/sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs` 拿文件后端逐个请求对拍钉住。
fn check_request_against_the_device_contract(
    offset: DeviceOffsetInBytes,
    length_in_bytes: u64,
    physical_block_size: PhysicalBlockSizeInBytes,
    device_size_in_bytes: u64,
) -> Result<(), BlockDeviceError> {
    let physical_block_bytes = u64::from(physical_block_size.0);
    if !offset.0.is_multiple_of(physical_block_bytes)
        || !length_in_bytes.is_multiple_of(physical_block_bytes)
    {
        return Err(BlockDeviceError::Unaligned {
            offset,
            length: length_in_bytes,
            physical_block_size: physical_block_size.0,
        });
    }
    match offset.0.checked_add(length_in_bytes) {
        Some(end) if end <= device_size_in_bytes => Ok(()),
        Some(_) | None => Err(BlockDeviceError::OutOfRange {
            offset,
            length: length_in_bytes,
            device_size: device_size_in_bytes,
        }),
    }
}

/// 内存镜像的物理块宽：稀疏设备按扇区存，扇区就是它的物理块。
fn sector_as_the_physical_block_size() -> PhysicalBlockSizeInBytes {
    PhysicalBlockSizeInBytes(u32::try_from(SECTOR_BYTES).expect("扇区宽 512 装得进 u32"))
}

/// 稀疏设备当一块盘用：宿主上重跑整条路（与虚机里同参数同字节）时的被测设备。没写过的读出全 0；屏障什么都不做（内存里没有缓存）。
/// 读、写、清零都先按与文件后端同一条判据核请求（[`check_request_against_the_device_contract`]）：越界、没对齐就交回与真盘同一种错误、
/// 盘上一个字节不动，不静默照收。
pub struct SparseBlockDevice {
    pub image: SparseDevice,
    size_in_bytes: u64,
    physical_block_size: PhysicalBlockSizeInBytes,
}

impl SparseBlockDevice {
    #[must_use]
    pub fn new(size_in_bytes: u64, physical_block_size: PhysicalBlockSizeInBytes) -> Self {
        Self {
            image: SparseDevice::default(),
            size_in_bytes,
            physical_block_size,
        }
    }

    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) -> Result<(), BlockDeviceError> {
        check_request_against_the_device_contract(
            offset,
            length_in_bytes,
            self.physical_block_size,
            self.size_in_bytes,
        )
    }
}

impl singlefs_core::block_device::BlockDevice for SparseBlockDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, u64::try_from(buffer.len()).expect("读长装得进 u64"))?;
        self.image.read_into(offset, buffer);
        Ok(())
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        _durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"))?;
        self.image.write(offset, bytes);
        Ok(())
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        self.image.zero_fill(offset, length);
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

/// 一次写落到盘上的内容。整段清零只带长度：mkfs 一次清 768 MiB，摊成字节就是每条写表多背 768 MiB 的 0。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WrittenContents {
    /// 普通写（含 FUA 写）：这些字节。
    Bytes(Vec<u8>),
    /// 整段清零：这么多个 0。
    Zeros { length: u64 },
}

impl WrittenContents {
    #[must_use]
    pub fn length_in_bytes(&self) -> u64 {
        match self {
            WrittenContents::Bytes(bytes) => u64::try_from(bytes.len()).expect("写长装得进 u64"),
            WrittenContents::Zeros { length } => *length,
        }
    }
    /// 普通写的字节；整段清零没有摆出来的字节，交回 `None`。
    #[must_use]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            WrittenContents::Bytes(bytes) => Some(bytes),
            WrittenContents::Zeros { .. } => None,
        }
    }
    /// 施加到一块稀疏盘上（两条路：普通写照写，清零走 [`SparseDevice::zero_fill`]）。
    pub fn apply_to(&self, device: &mut SparseDevice, offset: DeviceOffsetInBytes) {
        match self {
            WrittenContents::Bytes(bytes) => device.write(offset, bytes),
            WrittenContents::Zeros { length } => device.zero_fill(offset, *length),
        }
    }
    /// `[start, start + length)` 这一段写进 `destination`（`start` 是这次写内部的偏移）。
    pub fn copy_range_into(&self, start: usize, destination: &mut [u8]) {
        match self {
            WrittenContents::Bytes(bytes) => {
                destination.copy_from_slice(&bytes[start..start + destination.len()]);
            }
            WrittenContents::Zeros { .. } => destination.fill(0),
        }
    }
    /// 盘上这一段是不是还就是这次写写下去的内容。
    #[must_use]
    pub fn still_on_disk(&self, read_back: &[u8]) -> bool {
        match self {
            WrittenContents::Bytes(bytes) => read_back == bytes.as_slice(),
            WrittenContents::Zeros { .. } => read_back.iter().all(|byte| *byte == 0),
        }
    }
    /// 盘上 `read_back` 那一段是不是还是这次写在 `[start, start + read_back.len())` 上写下的内容（`start` 是这次写内部的偏移）。
    #[must_use]
    pub fn range_still_on_disk(&self, start: usize, read_back: &[u8]) -> bool {
        match self {
            WrittenContents::Bytes(bytes) => read_back == &bytes[start..start + read_back.len()],
            WrittenContents::Zeros { .. } => read_back.iter().all(|byte| *byte == 0),
        }
    }
}

/// 录制流里的一次写，带内容（屏障不进这张表）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedWrite {
    pub device: DeviceIdentity,
    pub kind: StepKind,
    pub is_force_unit_access: bool,
    pub offset: DeviceOffsetInBytes,
    pub contents: WrittenContents,
}

impl RetainedWrite {
    #[must_use]
    pub fn length_in_bytes(&self) -> u64 {
        self.contents.length_in_bytes()
    }
    /// 普通写的字节；调用方只在「这一条按构造是普通写」的地方用，清零没有字节可给。
    #[must_use]
    pub fn bytes(&self) -> Option<&[u8]> {
        self.contents.as_bytes()
    }
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
    /// 把写表里的若干次写按次序施加上去（崩溃注入建「更早的段整段持久」那份基线用；与 [`Self::apply`] 同一条落盘路）。
    pub fn apply_writes(&mut self, writes: &[RetainedWrite]) {
        for write in writes {
            write.contents.apply_to(
                self.devices
                    .get_mut(&write.device)
                    .expect("写表里的写都落在池里的盘上"),
                write.offset,
            );
        }
    }
    /// 把一段录制流按次序整个施加上去（屏障不改镜像）。
    pub fn apply(&mut self, operations: &[RetainedOperation]) {
        for retained in operations {
            let device = match retained.operation.kind {
                RecordedOperationKind::Barrier => continue,
                RecordedOperationKind::WriteZeroes => {
                    self.devices
                        .get_mut(&retained.operation.device)
                        .expect("录到的写都落在池里的盘上")
                        .zero_fill(retained.operation.offset, retained.operation.length);
                    continue;
                }
                RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => self
                    .devices
                    .get_mut(&retained.operation.device)
                    .expect("录到的写都落在池里的盘上"),
            };
            let contents = retained
                .contents
                .as_ref()
                .expect("崩溃点重放要开了内容保留的录制流");
            device.write(retained.operation.offset, contents);
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
    /// 层 0 的内存池每块盘一样大，字节数住 `device_size_in_bytes` 那个字段；池里没有这块盘就 `None`。
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.devices
            .contains_key(&device)
            .then_some(self.device_size_in_bytes)
    }
    /// 读不到（池里没有这块盘、越界、没按扇区对齐）返回 None，与一池真盘那一份（`[(DeviceIdentity, 块设备)]` 的 `PoolReader`：
    /// 块设备按同一条判据报错、它交回 None）同一个契约；判据是 [`check_request_against_the_device_contract`]。
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let device_image = self.devices.get(&device)?;
        check_request_against_the_device_contract(
            offset,
            u64::try_from(length).expect("读长装得进 u64"),
            sector_as_the_physical_block_size(),
            self.device_size_in_bytes,
        )
        .ok()?;
        Some(device_image.read(offset, length))
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

/// 层 0 枚举出的一个崩溃状态：基线 + 这一状态里持久了的写（按 `writes` 的次序叠）。
/// 枚举器交给观察者的那一份，`writes` 是枚举用的写表：录制流里的写原样在前（下标不变），原地覆写的撕裂镜像与它之后的重放接在后面
/// （[`TearableInPlaceOverwrites`]），`persisted` 与它逐条对应——撕裂那一态是「原来那次写没持久、它的撕裂镜像持久」。
pub struct CrashImage<'base> {
    pub base: &'base MemoryPool,
    pub writes: &'base [RetainedWrite],
    pub persisted: Vec<bool>,
}

impl PoolReader for CrashImage<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.base.device_identities()
    }
    /// 截断出来的崩溃镜像与它的基线池同几何：盘的字节数不随截断变。
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.base.device_size_in_bytes(device)
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
            let write_end = write_start + write.length_in_bytes();
            let overlap_start = read_start.max(write_start);
            let overlap_end = read_end.min(write_end);
            if overlap_start >= overlap_end {
                continue;
            }
            let destination = usize::try_from(overlap_start - read_start).expect("偏移");
            let source = usize::try_from(overlap_start - write_start).expect("偏移");
            let overlap_length = usize::try_from(overlap_end - overlap_start).expect("长度");
            write
                .contents
                .copy_range_into(source, &mut out[destination..destination + overlap_length]);
        }
        Some(out)
    }
    /// 基镜像那一份，加上这一状态里持久了的每次写在环里罩住的扇区中按记录槽对齐的那几个——与基镜像同一套「写过的扇区 →
    /// 对齐的记录槽」算法（[`record_slot_offsets`]）：一次写罩几个记录槽就给几个，不对齐的偏移一个不给。提示之外的记录槽，开头那一扇区
    /// 要么没写过、要么最后罩住它的是整段清零，读出来是全 0、过不了 journal magic，所以按提示扫与不给提示的全环扫描读出的记录逐条相同
    /// （用例 `tests/crash_image_journal_hint_matches_the_full_ring_scan.rs` 在同一批镜像上逐项比）。
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let mut offsets = self
            .base
            .journal_record_offsets_hint(device, ring_start, ring_bytes)?;
        let ring_end = ring_start.0 + ring_bytes;
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            match write.contents {
                // 整段清零之后那一段全是 0，记录槽开头过不了 journal magic：不进提示。
                WrittenContents::Zeros { .. } => continue,
                WrittenContents::Bytes(_) => {}
            }
            let written_start_in_the_ring = write.offset.0.max(ring_start.0);
            let written_end_in_the_ring = (write.offset.0 + write.length_in_bytes()).min(ring_end);
            if written_start_in_the_ring >= written_end_in_the_ring {
                continue;
            }
            offsets.extend(record_slot_offsets(
                written_start_in_the_ring / SECTOR_BYTES
                    ..written_end_in_the_ring.div_ceil(SECTOR_BYTES),
                ring_start,
            ));
        }
        offsets.sort_unstable();
        offsets.dedup();
        Some(offsets)
    }
}

/// 把一段录制流拆成「写表」与「段（写表下标）」，切法与 [`crate::segments::split_into_segments`] 同一条规则
/// （[`SegmentClosingRule`]：当前段里每块有写的盘都被自己的屏障或 FUA 放行了才关段）。
#[must_use]
pub fn writes_and_segments(
    operations: &[RetainedOperation],
    geometry: &FixedGeometry,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let (writes, segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(operations, geometry);
    (writes, segments)
}

/// 同上，再交回写表每一项在录制流里的下标：崩溃注入（增补 3 第 3 件）按它把一个崩溃状态落回历史的哪一步。
/// 切段只有这一份实现，两个调用方不各切一遍。
///
/// 这条流没记入口（[`writes_and_segments_with_stream_indexes_and_entries`] 交空的入口段）：流里一条带卸载记号的根槽写都不许有。
///
/// # Panics
/// 流里有带卸载记号的根槽写（C557（卸载记号只由卸载入口打没有检查））；写不带内容（流没开内容保留）。
#[must_use]
pub fn writes_and_segments_with_stream_indexes(
    operations: &[RetainedOperation],
    geometry: &FixedGeometry,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, Vec<usize>) {
    writes_and_segments_with_stream_indexes_and_entries(operations, &[], geometry)
}

/// 同上，这条流记了入口（`entry_spans`，下标与 `operations` 同一套、从 0 数）：切段之前先核 C557（卸载记号只由卸载入口打没有检查）——
/// 带卸载记号的根槽写只许落在卸载入口发的那几段里（[`unmount_markers_outside_the_unmount_entry`]）。
///
/// # Panics
/// 有带卸载记号的根槽写落在卸载入口之外；写不带内容（流没开内容保留）。
#[must_use]
pub fn writes_and_segments_with_stream_indexes_and_entries(
    operations: &[RetainedOperation],
    entry_spans: &[RecordedEntrySpan],
    geometry: &FixedGeometry,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, Vec<usize>) {
    let stray_markers =
        unmount_markers_outside_the_unmount_entry(operations, entry_spans, geometry);
    assert!(
        stray_markers.is_empty(),
        "C557（卸载记号只由卸载入口打没有检查）：录制流里有带卸载记号的根槽写不在卸载入口发的那一段里：{stray_markers:?}"
    );
    let mut writes: Vec<RetainedWrite> = Vec::new();
    let mut stream_indexes: Vec<usize> = Vec::new();
    let mut segments: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut closing_rule = SegmentClosingRule::default();
    for (stream_index, retained) in operations.iter().enumerate() {
        match retained.operation.kind {
            RecordedOperationKind::Barrier => {}
            RecordedOperationKind::WriteZeroes => {
                writes.push(RetainedWrite {
                    device: retained.operation.device,
                    kind: geometry.classify(&retained.operation),
                    is_force_unit_access: false,
                    offset: retained.operation.offset,
                    contents: WrittenContents::Zeros {
                        length: retained.operation.length,
                    },
                });
                stream_indexes.push(stream_index);
                current.push(writes.len() - 1);
            }
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                writes.push(RetainedWrite {
                    device: retained.operation.device,
                    kind: geometry.classify(&retained.operation),
                    is_force_unit_access: retained.operation.kind
                        == RecordedOperationKind::WriteForceUnitAccess,
                    offset: retained.operation.offset,
                    contents: WrittenContents::Bytes(
                        retained
                            .contents
                            .clone()
                            .expect("崩溃点重放要开了内容保留的录制流"),
                    ),
                });
                stream_indexes.push(stream_index);
                current.push(writes.len() - 1);
            }
        }
        match closing_rule.after(&retained.operation) {
            SegmentAfterOperation::Closes => segments.push(std::mem::take(&mut current)),
            SegmentAfterOperation::StaysOpen => {}
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    (writes, segments, stream_indexes)
}

/// E77（发布的持久顺序） 的闭式：1 + Σ(2^|段| − 1)，每次写只取两态（没持久 / 持久）时的全量。
/// 写表里有原地覆写的写时层 0 全量枚举出来的比它多（第三态），数见 [`layer0_state_count_with_torn_in_place_overwrites`]。
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
    /// 池里没有这块盘就 `None`（与 [`PoolReader::device_size_in_bytes`] 同一个答案）。
    fn device_bytes(&self, device: u32) -> Option<u64> {
        PoolReader::device_size_in_bytes(self, DeviceIdentity(device))
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        PoolReader::read(
            self,
            DeviceIdentity(device),
            DeviceOffsetInBytes(offset),
            length,
        )
    }
    /// 单元区里写过的扇区所在的槽：单元区从这块盘系统配置槽 0 自述的单元区起始槽号起（[`MemoryPool::recorded_geometry_of`]）。
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        let image = self.devices.get(&DeviceIdentity(device))?;
        let lowest_candidate_slot = self
            .recorded_geometry_of(DeviceIdentity(device))
            .lowest_candidate_unit_slot();
        let slots: BTreeSet<u64> = image
            .sectors
            .keys()
            .map(|sector| sector * SECTOR_BYTES / SLOT_BYTES)
            .filter(|slot| *slot >= lowest_candidate_slot)
            .collect();
        Some(slots.into_iter().collect())
    }
    /// journal 环里写过的扇区所在的记录槽（录制流只收真的记录写，环没有整段写 0 的那一次）。环长取这块盘系统配置槽 0 自述的
    /// （[`MemoryPool::recorded_geometry_of`]）。
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        let image = self.devices.get(&DeviceIdentity(device))?;
        let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        let candidate_ring_bytes = self
            .recorded_geometry_of(DeviceIdentity(device))
            .candidate_journal_ring_bytes(self.device_size_in_bytes);
        let slots: BTreeSet<u64> = image
            .written_sectors_in(DeviceOffsetInBytes(ring_start), candidate_ring_bytes)
            .into_iter()
            .map(|sector| (sector * SECTOR_BYTES - ring_start) / JOURNAL_RECORD_BYTES)
            .collect();
        Some(slots.into_iter().collect())
    }
}

/// 一块盘系统配置槽 0 自述的 journal 环长与单元区起始槽号（偏移 333、417；checker 划环与单元区读的也是这两处，
/// `singlefs_checker::image::geometry_of`）。扫描候选只用来缩小 checker 要读的槽：宁多不少，自证不过时退到能罩住任何合法几何的那一档。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordedGeometryForScanCandidates {
    RecordedInSlotZero {
        journal_ring_bytes: u64,
        unit_area_start_slot: u64,
    },
    /// 槽 0 读不出或自证不过：环长与单元区起点都不知道。
    SlotZeroNotSelfDescribing,
}

impl RecordedGeometryForScanCandidates {
    /// 单元扫描候选的下界：自述的单元区起始槽号；不知道时取 journal 环起点（任何合法环长下单元区起点都不低于它）。
    fn lowest_candidate_unit_slot(self) -> u64 {
        match self {
            RecordedGeometryForScanCandidates::RecordedInSlotZero {
                unit_area_start_slot,
                ..
            } => unit_area_start_slot,
            RecordedGeometryForScanCandidates::SlotZeroNotSelfDescribing => JOURNAL_RING_START_SLOT,
        }
    }

    /// journal 扫描候选罩住的环长：自述的环长；不知道时从环起点罩到盘尾。
    fn candidate_journal_ring_bytes(self, device_size_in_bytes: u64) -> u64 {
        match self {
            RecordedGeometryForScanCandidates::RecordedInSlotZero {
                journal_ring_bytes, ..
            } => journal_ring_bytes,
            RecordedGeometryForScanCandidates::SlotZeroNotSelfDescribing => {
                device_size_in_bytes.saturating_sub(JOURNAL_RING_START_SLOT * SLOT_BYTES)
            }
        }
    }
}

impl MemoryPool {
    /// 这块盘系统配置槽 0 自述的 journal 环长与单元区起始槽号（单元区起点随环长走，C475（非默认环长下单元区起点取编译期常量））。
    fn recorded_geometry_of(&self, device: DeviceIdentity) -> RecordedGeometryForScanCandidates {
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        PoolReader::read(self, device, DeviceOffsetInBytes(0), slot_bytes)
            .and_then(|slot| {
                SystemConfiguration::parse_slot(&slot)
                    .ok()
                    .map(|system_configuration| {
                        RecordedGeometryForScanCandidates::RecordedInSlotZero {
                            journal_ring_bytes: system_configuration
                                .immutable
                                .sizes
                                .journal_ring_bytes,
                            unit_area_start_slot: unit_area_start_slot_recorded_in_the_slot(&slot)
                                .0,
                        }
                    })
            })
            .unwrap_or(RecordedGeometryForScanCandidates::SlotZeroNotSelfDescribing)
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
        let candidate_ring_bytes = self
            .base
            .recorded_geometry_of(DeviceIdentity(device))
            .candidate_journal_ring_bytes(self.base.device_size_in_bytes);
        let mut slots: BTreeSet<u64> = ImageReader::candidate_journal_slots(self.base, device)?
            .into_iter()
            .collect();
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if *is_persisted
                && write.device.0 == device
                && (ring_start..ring_start + candidate_ring_bytes).contains(&write.offset.0)
            {
                slots.insert((write.offset.0 - ring_start) / JOURNAL_RECORD_BYTES);
            }
        }
        Some(slots.into_iter().collect())
    }
}

/// 记录核对器两条判据在一个状态上的结论。可比大小：层 0 发现表把它当签名的一段（[`Layer0RedPass::RecordChecker`]）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordCheck {
    /// 某次发布的根槽写已在盘上，而那次发布的 journal 记录一份都不在（E77（发布的持久顺序） b_ur 臂的「记录流有洞」）。
    pub root_without_record: bool,
    /// 恢复实际走的根 txg ≥ 某次发布，而那次发布写出的某个单元两份都不在（E77（发布的持久顺序） 的独立审计）。
    pub claimed_state_missing_unit: bool,
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

/// 这个持久集合把一次发布截在了它的根落盘之前：那次发布的单元写或记录写至少有一次已持久，而它的根槽写没持久。
#[must_use]
pub fn some_publish_persisted_without_its_root(
    writes: &[RetainedWrite],
    persisted: &[bool],
) -> bool {
    publishes_in(writes, persisted, RecordStreamContinuity::OneRecording)
        .iter()
        .any(|publish| {
            !persisted[publish.root]
                && publish
                    .units
                    .iter()
                    .chain(publish.records.iter())
                    .any(|write| persisted[*write])
        })
}

/// 这一状态里持久了的根槽写中最新的那一条，按 (txg, 实例) 字典序取（D22（单元原子性怎么合成） 已定项 7 的择新序）。
#[must_use]
pub fn newest_persisted_root(
    writes: &[RetainedWrite],
    persisted: &[bool],
) -> Option<(CheckpointTxg, InstanceGeneration)> {
    publishes_in(writes, persisted, RecordStreamContinuity::OneRecording)
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
pub fn root_identity_of_write(write: &RetainedWrite) -> (InstanceGeneration, CheckpointTxg) {
    assert_eq!(
        write.kind,
        StepKind::RootRecordFua,
        "被判的那条写要是根槽 FUA 写"
    );
    root_identity_written_by(write.bytes().expect("根槽 FUA 写是普通写，带着字节"))
}

/// 一次落在根环里的写写下的根身份：实例代号在偏移 24（4 字节）、checkpoint_txg 在偏移 28（8 字节）。
/// 崩溃注入（增补 3 第 3 件）拿它从截断了的录制流里数「盘上已持久的最新根槽」，写表与层 0 那一份不共用
/// （那一份要整条流的 `RetainedWrite`，这里只有录制流本身）。
///
/// # Panics
/// 这次写短于 36 字节：落在根环里的写都是整条根记录，短了说明调用方分错了种类。
#[must_use]
pub fn root_identity_written_by(bytes: &[u8]) -> (InstanceGeneration, CheckpointTxg) {
    (
        InstanceGeneration(u32::from_le_bytes(
            bytes[24..28].try_into().expect("根记录的实例代号在偏移 24"),
        )),
        CheckpointTxg(u64::from_le_bytes(
            bytes[28..36]
                .try_into()
                .expect("根记录的 checkpoint_txg 在偏移 28"),
        )),
    )
}

/// 一次落在根环里的写写下的卸载记号：根记录 flags 在偏移 20（4 字节），位 0（D22（单元原子性怎么合成） 已定项 7）。
/// 与 [`root_identity_written_by`] 同一个读法：只按偏移取字节，不验自证校验和。
///
/// # Panics
/// 这次写短于 24 字节：落在根环里的写都是整条根记录，短了说明调用方分错了种类。
#[must_use]
pub fn unmount_marker_written_by(bytes: &[u8]) -> UnmountMarker {
    let flags = u32::from_le_bytes(bytes[20..24].try_into().expect("根记录的 flags 在偏移 20"));
    if flags & ROOT_RECORD_FLAG_UNMOUNT_MARKER == 0 {
        UnmountMarker::NotWrittenByTheUnmountSequence
    } else {
        UnmountMarker::WrittenByTheUnmountSequence
    }
}

/// 录制流里一条带卸载记号的根槽写落在卸载入口发的那几段之外（C557（卸载记号只由卸载入口打没有检查））。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnmountMarkerOutsideTheUnmountEntry {
    /// 这次根槽写在录制流里的下标（与传进来的 `operations` 同一套）。
    pub stream_index: usize,
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
}

/// C557（卸载记号只由卸载入口打没有检查）：录制流上逐条核「带卸载记号的根只出现在卸载入口发的那一串里」。
/// 记号盘上没有别的东西核它：准入入口误打了记号、F 又抬过头，池级 checker 按带记号那一支放过（D22（单元原子性怎么合成） 已定项 7
/// 与 I-7.9（回退下界 F 不高于抬 F 的上限） 按记号分两支），所以这一格归录制流——它知道每一段是哪个入口发的
/// （[`crate::SharedStream::record_entry`]）。逐条看落在根环里的写（`FixedGeometry::classify` 认成根槽写的），
/// 带记号的要落在某一段 [`RecordedPublishEntry::Unmount`] 里；不在的按录制流的次序全列出来。
///
/// # Panics
/// 根槽写不带内容（流没开内容保留）。
#[must_use]
pub fn unmount_markers_outside_the_unmount_entry(
    operations: &[RetainedOperation],
    entry_spans: &[RecordedEntrySpan],
    geometry: &FixedGeometry,
) -> Vec<UnmountMarkerOutsideTheUnmountEntry> {
    let inside_the_unmount_entry = |stream_index: usize| {
        entry_spans.iter().any(|span| match span.entry {
            RecordedPublishEntry::Unmount => span.operations.contains(&stream_index),
        })
    };
    let mut stray_markers = Vec::new();
    for (stream_index, retained) in operations.iter().enumerate() {
        if geometry.classify(&retained.operation) != StepKind::RootRecordFua {
            continue;
        }
        let bytes = retained
            .contents
            .as_deref()
            .expect("核卸载记号要开了内容保留的录制流：根槽写要带着字节");
        match unmount_marker_written_by(bytes) {
            UnmountMarker::NotWrittenByTheUnmountSequence => {}
            UnmountMarker::WrittenByTheUnmountSequence => {
                if !inside_the_unmount_entry(stream_index) {
                    let (instance, checkpoint_txg) = root_identity_written_by(bytes);
                    stray_markers.push(UnmountMarkerOutsideTheUnmountEntry {
                        stream_index,
                        instance,
                        checkpoint_txg,
                    });
                }
            }
        }
    }
    stray_markers
}

/// 一次发布在写表里的下标：单元写、journal 记录写、根槽写，以及根里写的 checkpoint_txg。
pub struct PublishWrites {
    pub units: Vec<usize>,
    pub records: Vec<usize>,
    pub root: usize,
    pub instance: u32,
    pub checkpoint_txg: u64,
}

/// 交给记录核对器的写表是怎么来的：一条录制流从头到尾，还是一条被崩溃截断的录制流后面接上崩溃之后在那份崩溃后镜像上写出的流
/// （[`publishes_in`] 按它在接缝处断开）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordStreamContinuity {
    /// 一条录制流从头到尾（层 0 与崩溃注入第一截）。
    OneRecording,
    /// 写表前 `first_write_after_the_crash` 项是被一次崩溃截断的那条录制流（整条录下来的，截断处之后的写都记成没持久），
    /// 从这一项起是崩溃之后在那份崩溃后镜像上写出的流（崩溃注入第二、三截：可写挂载那一段）。
    ResumedAfterACrash {
        first_write_after_the_crash: usize,
        /// 那次崩溃之后只读恢复落到的那一版（恢复失败是 None）：截断处在飞的那次发布只在恢复落到它上（由它的记录重建）时算发生过。
        version_landed_on_after_the_crash: Option<(InstanceGeneration, CheckpointTxg)>,
    },
}

/// 写表里的每次发布。一条录制流里按根槽写分：一次根槽写之前、上一次根槽写之后的单元写与 journal 记录写都归它。
///
/// 写表是崩溃截断的流接上崩溃之后写出的流（[`RecordStreamContinuity::ResumedAfterACrash`]）时，在接缝处断开，两条流各分各的：
/// - 截断的那条流末尾没等到根槽写的单元写与记录写，不归接缝之后的第一次发布（那次发布是崩溃之后另起的实例写的）；
/// - 截断的那条流里只留崩溃之后那条时间线上的发布：根槽写落了盘的（`persisted`），与那次崩溃之后恢复落到的那一版
///   （由记录重建时它的根槽写在截断处之后、没落盘）。截断处之后才写根槽写、恢复又没落到的发布没发生过：崩溃之后的挂载另起实例、
///   txg 从盘上最大的往上接，与它们的 (实例, txg) 撞得上，挂载写的实例表也不带挂载自己实例的行，留着就会被当成恢复自称的那一版该有的发布。
///
/// 两条流接成一条来分（B3b 报告第二节第 4 条）会假红：第一次崩溃截在一次发布中间时，只接到截断处为止，那次发布没落根的单元写与记录写
/// 归进挂载写行那次发布；整条接上，截断处之后的历史发布与挂载的发布撞了身份。
pub fn publishes_in(
    writes: &[RetainedWrite],
    persisted: &[bool],
    continuity: RecordStreamContinuity,
) -> Vec<PublishWrites> {
    match continuity {
        RecordStreamContinuity::OneRecording => publishes_of_one_recording(writes, 0),
        RecordStreamContinuity::ResumedAfterACrash {
            first_write_after_the_crash,
            version_landed_on_after_the_crash,
        } => {
            let (cut_recording, resumed_recording) = writes.split_at(first_write_after_the_crash);
            publishes_of_one_recording(cut_recording, 0)
                .into_iter()
                .filter(|publish| {
                    persisted[publish.root]
                        || version_landed_on_after_the_crash
                            == Some((
                                InstanceGeneration(publish.instance),
                                CheckpointTxg(publish.checkpoint_txg),
                            ))
                })
                .chain(publishes_of_one_recording(
                    resumed_recording,
                    first_write_after_the_crash,
                ))
                .collect()
        }
    }
}

/// 一条录制流（`writes`，它的第一项在整张写表里的下标是 `first_write_index`）里的每次发布，下标按整张写表记。
pub fn publishes_of_one_recording(
    writes: &[RetainedWrite],
    first_write_index: usize,
) -> Vec<PublishWrites> {
    let mut publishes = Vec::new();
    let mut units = Vec::new();
    let mut records = Vec::new();
    for (index_in_the_recording, write) in writes.iter().enumerate() {
        let index = first_write_index + index_in_the_recording;
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
            // 整段清零不属于任何一次发布（今天唯一的清零是 mkfs 清 journal 环，在第一次发布之前）。
            StepKind::ZeroFill | StepKind::SystemConfigurationSlot | StepKind::Barrier => {}
        }
    }
    publishes
}
