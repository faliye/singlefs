//! 探针的盘：验证包的内存稀疏盘（`singlefs_verification::memory_pool::SparseBlockDevice`）外面包一层「崩在第一条记录之后」的剪刀。
//! 剪刀上膛之后，第一笔落进 journal 环的写照常落盘，之后两块盘上的写、清零、discard 一律丢掉、照报做成
//! （等于进程在那一笔之后断电：之前的写都持久，之后的一笔都没落）。没上膛时原样转给稀疏盘。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_format::{JOURNAL_RING_START_SLOT, SLOT_BYTES};
use singlefs_verification::memory_pool::SparseBlockDevice;

/// 一个池两块盘共用的剪刀。
#[derive(Debug, Default)]
pub struct CutAfterTheFirstRecord {
    armed: AtomicBool,
    cut: AtomicBool,
    journal_ring_bytes: std::sync::atomic::AtomicU64,
}

impl CutAfterTheFirstRecord {
    pub fn new(journal_ring_bytes: u64) -> Arc<Self> {
        let cut = Self::default();
        cut.journal_ring_bytes.store(journal_ring_bytes, Ordering::SeqCst);
        Arc::new(cut)
    }
    pub fn arm(&self) {
        self.cut.store(false, Ordering::SeqCst);
        self.armed.store(true, Ordering::SeqCst);
    }
    /// 卸下剪刀，交回这一趟剪没剪到（有没有一笔落进 journal 环）。
    pub fn disarm(&self) -> bool {
        self.armed.store(false, Ordering::SeqCst);
        self.cut.swap(false, Ordering::SeqCst)
    }
    fn dropping(&self) -> bool {
        self.cut.load(Ordering::SeqCst)
    }
    fn note_write(&self, offset: DeviceOffsetInBytes, length: u64) {
        if !self.armed.load(Ordering::SeqCst) {
            return;
        }
        let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        let ring_end = ring_start + self.journal_ring_bytes.load(Ordering::SeqCst);
        if offset.0 < ring_end && offset.0 + length > ring_start {
            self.cut.store(true, Ordering::SeqCst);
        }
    }
}

pub struct ProbeDevice {
    pub inner: SparseBlockDevice,
    cut: Arc<CutAfterTheFirstRecord>,
}

impl ProbeDevice {
    pub fn new(size_in_bytes: u64, cut: Arc<CutAfterTheFirstRecord>) -> Self {
        Self {
            inner: SparseBlockDevice::new(size_in_bytes, PhysicalBlockSizeInBytes(512)),
            cut,
        }
    }
    /// 同样的字节、另一把剪刀（拷贝池用）。
    pub fn copy_with(&self, cut: Arc<CutAfterTheFirstRecord>) -> Self {
        let mut inner = SparseBlockDevice::new(
            self.inner.size_in_bytes(),
            PhysicalBlockSizeInBytes(512),
        );
        inner.image = self.inner.image.clone();
        Self { inner, cut }
    }
}

impl BlockDevice for ProbeDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if self.cut.dropping() {
            return Ok(());
        }
        self.inner.write_at(offset, bytes, durability)?;
        self.cut
            .note_write(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"));
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        if self.cut.dropping() {
            return Ok(());
        }
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        if self.cut.dropping() {
            return Ok(DiscardOutcome::Applied);
        }
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
        self.inner.journal_record_offsets_hint(ring_start, ring_bytes)
    }
}
