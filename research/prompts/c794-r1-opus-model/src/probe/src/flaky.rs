//! 读坏 / 吞写的故障层（包在录制器外面：被吞的写不进录制流，读坏只坏交回的那一读，盘上字节不动）。
use std::cell::RefCell;
use std::rc::Rc;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub type Rec = RecordingBlockDevice<SparseBlockDevice>;

#[derive(Default)]
pub struct Faults {
    /// (盘, 偏移, 还要坏几次)：这个偏移上的读报块设备错。
    pub read_fail: Vec<(DeviceIdentity, u64, u64)>,
    /// (盘, 起点, 长度)：落在这一段里的写报成功、一个字节不落（设备说谎）。
    pub swallow: Vec<(DeviceIdentity, u64, u64)>,
    pub swallowed_writes: u64,
    pub failed_reads: u64,
}

pub struct Flaky {
    pub id: DeviceIdentity,
    pub inner: Rec,
    pub faults: Rc<RefCell<Faults>>,
}

impl BlockDevice for Flaky {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        {
            let mut f = self.faults.borrow_mut();
            let mut hit = false;
            for entry in f.read_fail.iter_mut() {
                if entry.0 == self.id && entry.1 == offset.0 && entry.2 > 0 {
                    entry.2 -= 1;
                    hit = true;
                    break;
                }
            }
            if hit {
                f.failed_reads += 1;
                return Err(injected_block_device_error("读"));
            }
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        {
            let mut f = self.faults.borrow_mut();
            let end = offset.0 + bytes.len() as u64;
            let hit = f.swallow.iter().any(|(d, s, l)| *d == self.id && offset.0 < s + l && *s < end);
            if hit {
                f.swallowed_writes += 1;
                return Ok(());
            }
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
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
    fn journal_record_offsets_hint(&self, start: DeviceOffsetInBytes, bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(start, bytes)
    }
}

impl crate::pool::DeviceOverASparseImage for Flaky {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.wrapped_device().image
    }
}

impl Flaky {
    pub fn image_mut(&mut self) -> &mut SparseDevice {
        &mut self.inner.wrapped_device_mut().image
    }
}

pub fn start_pool(stream: &SharedStream, faults: &Rc<RefCell<Faults>>) -> crate::pool::PoolUnderTest<Flaky> {
    let stream = stream.clone();
    let faults = faults.clone();
    crate::pool::PoolUnderTest::start_after_the_first_file_on_devices_of(
        crate::pool::parameters_of_the_384_slot_small_devices(),
        crate::pool::device_bytes_of_the_384_slot_small_devices(),
        move |id, dev| Flaky { id, inner: RecordingBlockDevice::with_shared_stream(id, dev, stream.clone()), faults: faults.clone() },
    )
}
