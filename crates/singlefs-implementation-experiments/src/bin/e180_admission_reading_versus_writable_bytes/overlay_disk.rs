//! 叠加盘：底盘的扇区、叠在上面的写、按策略拒数据单元的写与读不了的范围。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 设备：只存非零扇区的稀疏盘，加一层写时复制的叠层
// ---------------------------------------------------------------------------------------------------------------------

type Sector = [u8; SECTOR_BYTES];

/// 按扇区存的盘面。基底（没有叠层）的盘不存全零扇区；叠层存全零扇区，用来盖住底下基底里的非零扇区。
#[derive(Clone, Default)]
pub(super) struct SectorStore {
    sectors: BTreeMap<u64, Box<Sector>>,
}

impl SectorStore {
    pub(super) fn write_sectors(
        &mut self,
        byte_offset: u64,
        bytes: &[u8],
        keeps_zero_sectors: bool,
    ) {
        let first_sector = byte_offset / u64::try_from(SECTOR_BYTES).expect("512");
        for (index_in_write, chunk) in bytes.as_chunks::<SECTOR_BYTES>().0.iter().enumerate() {
            let sector_number =
                first_sector + u64::try_from(index_in_write).expect("写里的扇区下标");
            let chunk_is_all_zero = chunk.iter().all(|byte| *byte == 0);
            if chunk_is_all_zero && !keeps_zero_sectors {
                self.sectors.remove(&sector_number);
            } else {
                let mut sector: Box<Sector> = Box::new([0u8; SECTOR_BYTES]);
                sector.copy_from_slice(chunk);
                self.sectors.insert(sector_number, sector);
            }
        }
    }

    fn copy_into(&self, byte_offset: u64, buffer: &mut [u8]) {
        let sector_bytes = u64::try_from(SECTOR_BYTES).expect("512");
        let first_sector = byte_offset / sector_bytes;
        let sector_count = u64::try_from(buffer.len()).expect("缓冲长度") / sector_bytes;
        for (sector_number, sector) in self
            .sectors
            .range(first_sector..first_sector + sector_count)
        {
            let start =
                usize::try_from((sector_number - first_sector) * sector_bytes).expect("缓冲内偏移");
            buffer[start..start + SECTOR_BYTES].copy_from_slice(&sector[..]);
        }
    }

    fn sector_numbers_in(&self, sector_range: Range<u64>) -> Vec<u64> {
        self.sectors
            .range(sector_range)
            .map(|(number, _)| *number)
            .collect()
    }
}

/// 数据单元的写到达时怎么办（登记 5.1 探测捷径）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DataWritePolicy {
    /// 照写。
    AllowDataWrites,
    /// 第一个数据单元的写到达时记「到达」并报块设备错（会话把它当作发布途中的写错交回）。
    FailTheFirstDataWriteAndNoteIt,
}

static NEXT_RECORDED_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// 设备上一次交给它的写，按到达次序（崩溃态按「写到第 k 次」截取用）。
#[derive(Clone, Debug)]
pub(super) struct RecordedWrite {
    /// 全局写序号：两块盘上的写按到达次序排得回来（崩溃态按「写到根槽的 FUA 写」截取用）。
    pub(super) sequence: u64,
    pub(super) byte_offset: u64,
    pub(super) bytes: Vec<u8>,
    pub(super) is_force_unit_access: bool,
}

/// 一块盘：自己的扇区，加可选的只读基底（叠层用）。
pub(super) struct Disk<'base> {
    size_in_bytes: u64,
    base: Option<&'base SectorStore>,
    pub(super) own: SectorStore,
    data_write_policy: DataWritePolicy,
    pub(super) data_write_reached: bool,
    /// 读这些字节区间一律报读错（登记 5「历史族」表 HC 一行，被抛弃根与净释放族的挂载乙：根槽与它的 journal 记录每次读都失败）。
    pub(super) unreadable_byte_ranges: Vec<Range<u64>>,
    pub(super) recording: Option<Vec<RecordedWrite>>,
}

impl Disk<'static> {
    pub(super) fn blank(size_in_bytes: u64) -> Self {
        Disk {
            size_in_bytes,
            base: None,
            own: SectorStore::default(),
            data_write_policy: DataWritePolicy::AllowDataWrites,
            data_write_reached: false,
            unreadable_byte_ranges: Vec::new(),
            recording: None,
        }
    }

    /// 从另一块盘的盘面拷一份独立的盘（崩溃态、被抛弃根与净释放族的挂载乙用）。
    pub(super) fn independent_copy_of(store: &SectorStore, size_in_bytes: u64) -> Self {
        let mut disk = Disk::blank(size_in_bytes);
        disk.own = store.clone();
        disk
    }
}

impl Disk<'_> {
    /// 写时复制的叠层：底下共享这块盘的盘面，只存这一次探测写的字节。
    pub(super) fn overlay(&self, data_write_policy: DataWritePolicy) -> Disk<'_> {
        assert!(self.base.is_none(), "叠层不套叠层");
        Disk {
            size_in_bytes: self.size_in_bytes,
            base: Some(&self.own),
            own: SectorStore::default(),
            data_write_policy,
            data_write_reached: false,
            unreadable_byte_ranges: Vec::new(),
            recording: None,
        }
    }

    fn keeps_zero_sectors(&self) -> bool {
        self.base.is_some()
    }

    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let block = u64::from(PHYSICAL_BLOCK_BYTES);
        if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length,
                physical_block_size: PHYSICAL_BLOCK_BYTES,
            });
        }
        match offset.0.checked_add(length) {
            Some(end) if end <= self.size_in_bytes => Ok(()),
            Some(_) | None => Err(BlockDeviceError::OutOfRange {
                offset,
                length,
                device_size: self.size_in_bytes,
            }),
        }
    }

    fn zero_fill(&mut self, offset: u64, length: u64) {
        let sector_bytes = u64::try_from(SECTOR_BYTES).expect("512");
        let sector_range = offset / sector_bytes..(offset + length) / sector_bytes;
        let own_numbers = self.own.sector_numbers_in(sector_range.clone());
        let base_numbers = self
            .base
            .map(|base| base.sector_numbers_in(sector_range))
            .unwrap_or_default();
        for number in own_numbers {
            if self.base.is_some() {
                self.own
                    .sectors
                    .insert(number, Box::new([0u8; SECTOR_BYTES]));
            } else {
                self.own.sectors.remove(&number);
            }
        }
        for number in base_numbers {
            self.own
                .sectors
                .insert(number, Box::new([0u8; SECTOR_BYTES]));
        }
    }

    fn is_data_unit_write(bytes: &[u8]) -> bool {
        bytes.len() == usize::try_from(DATA_UNIT_BYTES).expect("32768")
            && bytes[..4] == UNIT_MAGIC_BYTES
            && bytes[UNIT_CLASS_BYTE_INDEX] == UNIT_CLASS_DATA_CODE
    }
}

impl BlockDevice for Disk<'_> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长");
        self.check_request(offset, length)?;
        let request = offset.0..offset.0 + length;
        if self
            .unreadable_byte_ranges
            .iter()
            .any(|range| range.start < request.end && request.start < range.end)
        {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "装置注入的读错：这一段字节区间每次读都失败",
            )));
        }
        buffer.fill(0);
        if let Some(base) = self.base {
            base.copy_into(offset.0, buffer);
        }
        self.own.copy_into(offset.0, buffer);
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, u64::try_from(bytes.len()).expect("写长"))?;
        if self.data_write_policy == DataWritePolicy::FailTheFirstDataWriteAndNoteIt
            && Self::is_data_unit_write(bytes)
        {
            self.data_write_reached = true;
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "装置注入的写错：第一个数据单元的写到达",
            )));
        }
        if let Some(recording) = self.recording.as_mut() {
            recording.push(RecordedWrite {
                sequence: NEXT_RECORDED_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed),
                byte_offset: offset.0,
                bytes: bytes.to_vec(),
                is_force_unit_access: durability == WriteDurability::ForceUnitAccess,
            });
        }
        let keeps_zero_sectors = self.keeps_zero_sectors();
        self.own.write_sectors(offset.0, bytes, keeps_zero_sectors);
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_fill(offset.0, length);
        Ok(())
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_fill(offset.0, length);
        Ok(DiscardOutcome::Applied)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)
    }

    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    /// 写过的扇区所在的记录槽（只列记录起点上的扇区；叠层里全零的扇区是盖住基底用的，不算写过）。
    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let sector_bytes = u64::try_from(SECTOR_BYTES).expect("512");
        let sector_range = ring_start.0 / sector_bytes..(ring_start.0 + ring_bytes) / sector_bytes;
        let mut numbers: BTreeSet<u64> = BTreeSet::new();
        if let Some(base) = self.base {
            numbers.extend(base.sector_numbers_in(sector_range.clone()));
        }
        for (number, sector) in self.own.sectors.range(sector_range) {
            if sector.iter().all(|byte| *byte == 0) {
                numbers.remove(number);
            } else {
                numbers.insert(*number);
            }
        }
        Some(
            numbers
                .into_iter()
                .map(|number| number * sector_bytes)
                .filter(|offset| (offset - ring_start.0).is_multiple_of(JOURNAL_RECORD_BYTES))
                .map(DeviceOffsetInBytes)
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests;
