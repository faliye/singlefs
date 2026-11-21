use super::*;

// ============================================================================================
// E31：只记不改的块设备包装，按落点区间分别数读写的次数与字节。区间的分法照 `crates/singlefs-verification/src/segments.rs` 的分类读法另写一份。
// ============================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoredRegion {
    SystemConfiguration,
    RootRing,
    JournalRing,
    UnitAreaAndEverythingElse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CallsAndBytes {
    pub(super) calls: u64,
    pub(super) bytes: u64,
}

impl CallsAndBytes {
    pub(super) fn minus(self, earlier: CallsAndBytes) -> CallsAndBytes {
        CallsAndBytes {
            calls: self.calls - earlier.calls,
            bytes: self.bytes - earlier.bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ReadsAndWrites {
    pub(super) reads: CallsAndBytes,
    pub(super) writes: CallsAndBytes,
}

impl ReadsAndWrites {
    pub(super) fn minus(self, earlier: ReadsAndWrites) -> ReadsAndWrites {
        ReadsAndWrites {
            reads: self.reads.minus(earlier.reads),
            writes: self.writes.minus(earlier.writes),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DeviceTally {
    pub(super) system_configuration: ReadsAndWrites,
    pub(super) root_ring: ReadsAndWrites,
    pub(super) journal_ring: ReadsAndWrites,
    pub(super) unit_area: ReadsAndWrites,
}

impl DeviceTally {
    fn region_mut(&mut self, region: StoredRegion) -> &mut ReadsAndWrites {
        match region {
            StoredRegion::SystemConfiguration => &mut self.system_configuration,
            StoredRegion::RootRing => &mut self.root_ring,
            StoredRegion::JournalRing => &mut self.journal_ring,
            StoredRegion::UnitAreaAndEverythingElse => &mut self.unit_area,
        }
    }

    pub(super) fn minus(self, earlier: DeviceTally) -> DeviceTally {
        DeviceTally {
            system_configuration: self
                .system_configuration
                .minus(earlier.system_configuration),
            root_ring: self.root_ring.minus(earlier.root_ring),
            journal_ring: self.journal_ring.minus(earlier.journal_ring),
            unit_area: self.unit_area.minus(earlier.unit_area),
        }
    }
}

/// 固定结构在一块盘上的区间边界（字节偏移）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FixedRegions {
    system_configuration_end: u64,
    root_ring_start: u64,
    root_ring_end: u64,
    journal_ring_start: u64,
    journal_ring_end: u64,
}

impl FixedRegions {
    pub(super) fn of_geometry(
        slot_spacing_bytes: u64,
        slots_per_region: u64,
        journal_ring_bytes: u64,
    ) -> Self {
        let root_ring_start = ROOT_RING_BASE_SLOT * SLOT_BYTES;
        let journal_ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        FixedRegions {
            system_configuration_end: SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE * slot_spacing_bytes,
            root_ring_start,
            root_ring_end: root_ring_start
                + (ROOT_RING_REGIONS - 1) * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES
                + slots_per_region * slot_spacing_bytes,
            journal_ring_start,
            journal_ring_end: journal_ring_start + journal_ring_bytes,
        }
    }

    fn region_of(&self, offset: u64) -> StoredRegion {
        if offset < self.system_configuration_end {
            StoredRegion::SystemConfiguration
        } else if offset >= self.root_ring_start && offset < self.root_ring_end {
            StoredRegion::RootRing
        } else if offset >= self.journal_ring_start && offset < self.journal_ring_end {
            StoredRegion::JournalRing
        } else {
            StoredRegion::UnitAreaAndEverythingElse
        }
    }
}

/// 包在一块盘外面，读写原样转交，只数。读是 `&self`，所以走 `RefCell`（单线程用）。
pub(super) struct CountingBlockDevice<Inner: BlockDevice> {
    pub(super) inner: Inner,
    pub(super) regions: FixedRegions,
    pub(super) tally: RefCell<DeviceTally>,
}

impl<Inner: BlockDevice> CountingBlockDevice<Inner> {
    pub(super) fn new(inner: Inner, regions: FixedRegions) -> Self {
        Self {
            inner,
            regions,
            tally: RefCell::new(DeviceTally::default()),
        }
    }

    pub(super) fn tally_now(&self) -> DeviceTally {
        *self.tally.borrow()
    }

    fn count_read(&self, offset: DeviceOffsetInBytes, length: usize) {
        let region = self.regions.region_of(offset.0);
        let mut tally = self.tally.borrow_mut();
        let reads = &mut tally.region_mut(region).reads;
        reads.calls += 1;
        reads.bytes += u64::try_from(length).expect("读长装得进 u64");
    }

    fn count_write(&mut self, offset: DeviceOffsetInBytes, length: u64) {
        let region = self.regions.region_of(offset.0);
        let writes = &mut self.tally.get_mut().region_mut(region).writes;
        writes.calls += 1;
        writes.bytes += length;
    }
}

impl<Inner: BlockDevice> BlockDevice for CountingBlockDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.count_read(offset, buffer.len());
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.count_write(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"));
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.count_write(offset, length);
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.inner.discard_at(offset, length)
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

    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.count_read(offset, buffer.len());
        self.inner.reread_at_after_a_bad_read(offset, buffer)
    }

    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        self.inner.keep_the_self_verified_reread(offset, bytes);
    }

    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        self.inner
            .note_the_read_failing_self_verification_and_forget_its_bytes(offset, length_in_bytes);
    }

    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.count_read(offset, buffer.len());
        self.inner.read_at_straight_from_the_disk(offset, buffer)
    }
}

#[cfg(test)]
mod tests;
