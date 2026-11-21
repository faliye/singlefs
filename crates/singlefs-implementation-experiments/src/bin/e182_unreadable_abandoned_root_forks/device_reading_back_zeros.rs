use super::*;

// ============================================================================
// 三、读回全零的设备包装（`singlefs_verification::fault_injection` 里只有读报错、读回翻位，没有读回全零；登记「读法写死」第一条）。
//    按字节区间做；这个区间上一旦有写落下，这个进程之后读它读到的就是真字节（与 E158 装置的 `written_over` 同一个口径）。
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReadsZeroRange {
    pub(super) device: DeviceIdentity,
    pub(super) offset: u64,
    pub(super) length: u64,
    pub(super) label: &'static str,
}

#[derive(Default)]
pub(super) struct ReadsZeroState {
    pub(super) ranges: Vec<ReadsZeroRange>,
    pub(super) written_over: Vec<bool>,
    pub(super) intercepted_reads: Vec<u64>,
}

impl ReadsZeroState {
    pub(super) fn with_ranges(ranges: Vec<ReadsZeroRange>) -> Self {
        let count = ranges.len();
        ReadsZeroState {
            ranges,
            written_over: vec![false; count],
            intercepted_reads: vec![0; count],
        }
    }

    fn mark_written_over(&mut self, device: DeviceIdentity, offset: u64, length: u64) {
        for (range, written_over) in self.ranges.iter().zip(self.written_over.iter_mut()) {
            if range.device == device
                && range.offset < offset + length
                && offset < range.offset + range.length
            {
                *written_over = true;
            }
        }
    }
}

pub(super) struct ReadsZeroDevice {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    pub(super) state: Rc<RefCell<ReadsZeroState>>,
}

impl BlockDevice for ReadsZeroDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        let mut state = self.state.borrow_mut();
        let state = &mut *state;
        for ((range, written_over), intercepted) in state
            .ranges
            .iter()
            .zip(state.written_over.iter())
            .zip(state.intercepted_reads.iter_mut())
        {
            if *written_over || range.device != self.identity {
                continue;
            }
            let start = range.offset.max(offset.0);
            let end = (range.offset + range.length).min(offset.0 + length);
            if start >= end {
                continue;
            }
            *intercepted += 1;
            let first = usize::try_from(start - offset.0).expect("缓冲区内偏移");
            let last = usize::try_from(end - offset.0).expect("缓冲区内偏移");
            buffer[first..last].fill(0);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)?;
        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        self.state
            .borrow_mut()
            .mark_written_over(self.identity, offset.0, length);
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)?;
        self.state
            .borrow_mut()
            .mark_written_over(self.identity, offset.0, length);
        Ok(())
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        let outcome = self.inner.discard_at(offset, length)?;
        self.state
            .borrow_mut()
            .mark_written_over(self.identity, offset.0, length);
        Ok(outcome)
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
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

pub(super) type RecordedDevices = Vec<(DeviceIdentity, RecordingBlockDevice<ReadsZeroDevice>)>;

/// 以 `base` 的镜像为起点的两块录制盘，共用一条录制流与一份读回全零的状态。
pub(super) fn recording_devices(
    base: &MemoryPool,
    stream: &SharedStream,
    state: &Rc<RefCell<ReadsZeroState>>,
) -> RecordedDevices {
    device_identities()
        .into_iter()
        .map(|identity| {
            let mut inner = SparseBlockDevice::new(
                base.device_size_in_bytes,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES_LOCAL),
            );
            if let Some(image) = base.devices.get(&identity) {
                inner.image = image.clone();
            }
            let faulting = ReadsZeroDevice {
                identity,
                inner,
                state: Rc::clone(state),
            };
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, faulting, stream.clone()),
            )
        })
        .collect()
}

/// 录制盘用完之后的镜像（盘上真有的字节，与读回全零无关）。
pub(super) fn pool_of_the_recorded_devices(
    devices: RecordedDevices,
    device_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| {
                let (faulting, _operations) = device.into_inner_and_operations();
                (identity, faulting.inner.image)
            })
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

#[cfg(test)]
mod tests;
