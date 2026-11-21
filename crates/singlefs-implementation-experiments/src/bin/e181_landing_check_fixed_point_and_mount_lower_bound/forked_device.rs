//! 分叉盘：底层与叠在上面的几层，落盘与读回。

use super::*;

/// 稀疏设备的一层：写过的扇区加被整段清零或 discard 过的扇区区间。清零先于同一层里的写生效（读时先抹区间、再盖写过的扇区）。
#[derive(Clone, Debug, Default)]
struct DeviceLayer {
    written: SparseDevice,
    zeroed_sector_ranges: Vec<(u64, u64)>,
}

/// 写时复制的内存盘：封存的层只读、可被几份拷贝共享，顶层只属于这一份。
/// 拷贝一个已填满的池只拷层的引用，不拷几 GiB 的扇区（跑前登记 5.2 节「64 个种子各在写时复制的包装上走 300 步」）。
pub(super) struct ForkedDevice {
    sealed_layers: Vec<Arc<DeviceLayer>>,
    top_layer: DeviceLayer,
    size_in_bytes: u64,
    pub(super) physical_block_size: PhysicalBlockSizeInBytes,
}

impl ForkedDevice {
    pub(super) fn empty(size_in_bytes: u64) -> Self {
        Self {
            sealed_layers: Vec::new(),
            top_layer: DeviceLayer::default(),
            size_in_bytes,
            physical_block_size: PhysicalBlockSizeInBytes(512),
        }
    }

    /// 把顶层封存，交回一份与它读到同样字节的拷贝（拷贝之后各写各的顶层）。
    pub(super) fn fork(&mut self) -> ForkedDevice {
        let top_layer_is_empty = self.top_layer.written.written_sectors().is_empty()
            && self.top_layer.zeroed_sector_ranges.is_empty();
        if !top_layer_is_empty {
            self.sealed_layers
                .push(Arc::new(std::mem::take(&mut self.top_layer)));
        }
        ForkedDevice {
            sealed_layers: self.sealed_layers.clone(),
            top_layer: DeviceLayer::default(),
            size_in_bytes: self.size_in_bytes,
            physical_block_size: self.physical_block_size,
        }
    }

    /// 同 [`ForkedDevice::fork`]，但不封存：顶层必须已经是空的（交给几个线程共用同一份底时用）。
    pub(super) fn fork_of_a_sealed_base(&self) -> ForkedDevice {
        assert!(
            self.top_layer.written.written_sectors().is_empty()
                && self.top_layer.zeroed_sector_ranges.is_empty(),
            "共用的底顶层是空的：先 fork 过一次再交给线程"
        );
        ForkedDevice {
            sealed_layers: self.sealed_layers.clone(),
            top_layer: DeviceLayer::default(),
            size_in_bytes: self.size_in_bytes,
            physical_block_size: self.physical_block_size,
        }
    }

    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) -> Result<(), BlockDeviceError> {
        let physical_block_bytes = u64::from(self.physical_block_size.0);
        if !offset.0.is_multiple_of(physical_block_bytes)
            || !length_in_bytes.is_multiple_of(physical_block_bytes)
        {
            return Err(BlockDeviceError::Unaligned {
                offset,
                length: length_in_bytes,
                physical_block_size: self.physical_block_size.0,
            });
        }
        match offset.0.checked_add(length_in_bytes) {
            Some(end) if end <= self.size_in_bytes => Ok(()),
            Some(_) | None => Err(BlockDeviceError::OutOfRange {
                offset,
                length: length_in_bytes,
                device_size: self.size_in_bytes,
            }),
        }
    }

    fn zero_range_in_the_top_layer(&mut self, offset: DeviceOffsetInBytes, length: u64) {
        self.top_layer.written.zero_fill(offset, length);
        let first_sector = offset.0 / SECTOR_BYTES;
        self.top_layer
            .zeroed_sector_ranges
            .push((first_sector, first_sector + length / SECTOR_BYTES));
    }
}

impl BlockDevice for ForkedDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length_in_bytes)?;
        buffer.fill(0);
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("扇区宽装得进 usize");
        let first_sector = offset.0 / SECTOR_BYTES;
        let end_sector = first_sector + length_in_bytes / SECTOR_BYTES;
        let layers_bottom_up = self
            .sealed_layers
            .iter()
            .map(|layer| &**layer)
            .chain(std::iter::once(&self.top_layer));
        for layer in layers_bottom_up {
            for (zero_start, zero_end) in &layer.zeroed_sector_ranges {
                let from = (*zero_start).max(first_sector);
                let to = (*zero_end).min(end_sector);
                if from < to {
                    let from_byte =
                        usize::try_from(from - first_sector).expect("下标") * sector_bytes;
                    let to_byte = usize::try_from(to - first_sector).expect("下标") * sector_bytes;
                    buffer[from_byte..to_byte].fill(0);
                }
            }
            for (sector, bytes) in layer
                .written
                .written_sectors()
                .range(first_sector..end_sector)
            {
                let start = usize::try_from(sector - first_sector).expect("下标") * sector_bytes;
                buffer[start..start + sector_bytes].copy_from_slice(bytes);
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        _durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"))?;
        self.top_layer.written.write(offset, bytes);
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_range_in_the_top_layer(offset, length);
        Ok(())
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.check_request(offset, length)?;
        self.zero_range_in_the_top_layer(offset, length);
        Ok(DiscardOutcome::Applied)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    /// 各层写过的扇区里落在记录槽边界上的偏移（多给不要紧，漏一个才错；清零过的扇区多给一个无害）。
    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let mut offsets: BTreeSet<u64> = BTreeSet::new();
        for layer in self
            .sealed_layers
            .iter()
            .map(|layer| &**layer)
            .chain(std::iter::once(&self.top_layer))
        {
            for sector in layer.written.written_sectors_in(ring_start, ring_bytes) {
                let offset = sector * SECTOR_BYTES;
                if (offset - ring_start.0).is_multiple_of(JOURNAL_RECORD_BYTES) {
                    offsets.insert(offset);
                }
            }
        }
        Some(offsets.into_iter().map(DeviceOffsetInBytes).collect())
    }
}
