//! 只存写过的扇区的内存盘（没写过的读出全 0，屏障什么都不做）：探针自己的一份，不引验证包（验证包拖着判定库客户端与 RocksDB）。
//! 请求按物理块 512 对齐、不越过盘尾，不合就交回与真盘同一种错误、一个字节不动。

use std::collections::BTreeMap;

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_format::JOURNAL_RECORD_BYTES;

pub const SECTOR_BYTES: u64 = 512;

#[derive(Clone, Debug)]
pub struct MemoryDevice {
    sectors: BTreeMap<u64, Vec<u8>>,
    size_in_bytes: u64,
}

impl MemoryDevice {
    pub fn new(size_in_bytes: u64) -> Self {
        Self {
            sectors: BTreeMap::new(),
            size_in_bytes,
        }
    }

    fn check(&self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        if offset.0 % SECTOR_BYTES != 0 || length % SECTOR_BYTES != 0 {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length,
                physical_block_size: 512,
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

    fn remove(&mut self, offset: DeviceOffsetInBytes, length: u64) {
        let first = offset.0 / SECTOR_BYTES;
        let end = (offset.0 + length) / SECTOR_BYTES;
        let inside: Vec<u64> = self
            .sectors
            .range(first..end)
            .map(|(sector, _)| *sector)
            .collect();
        for sector in inside {
            self.sectors.remove(&sector);
        }
    }
}

impl BlockDevice for MemoryDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check(offset, length)?;
        buffer.fill(0);
        let first = offset.0 / SECTOR_BYTES;
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        for (sector, bytes) in self.sectors.range(first..first + length / SECTOR_BYTES) {
            let start = usize::try_from(sector - first).expect("扇区下标") * sector_bytes;
            buffer[start..start + sector_bytes].copy_from_slice(bytes);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        _durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"))?;
        let first = offset.0 / SECTOR_BYTES;
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        for (index, chunk) in bytes.chunks_exact(sector_bytes).enumerate() {
            self.sectors.insert(
                first + u64::try_from(index).expect("扇区下标"),
                chunk.to_vec(),
            );
        }
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check(offset, length)?;
        self.remove(offset, length);
        Ok(())
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.check(offset, length)?;
        self.remove(offset, length);
        Ok(DiscardOutcome::Applied)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(512)
    }

    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let first = ring_start.0 / SECTOR_BYTES;
        let end = (ring_start.0 + ring_bytes) / SECTOR_BYTES;
        let mut offsets: Vec<DeviceOffsetInBytes> = self
            .sectors
            .range(first..end)
            .map(|(sector, _)| sector * SECTOR_BYTES)
            .filter(|offset| (offset - ring_start.0) % JOURNAL_RECORD_BYTES == 0)
            .map(DeviceOffsetInBytes)
            .collect();
        offsets.dedup();
        Some(offsets)
    }
}
