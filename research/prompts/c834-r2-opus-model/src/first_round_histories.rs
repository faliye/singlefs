//! 第一轮攻方甲至戊的 M1（同一造法）：`clean` M1 无读故障；`tree_table_and_record` M1 时 R 的树表与记录读回全零（映射现算得出、隔离 R 的单元）；
//! `root_units_and_record` 入库 `urJ` 那一形的 M1（R 的根槽、全部单元、记录读回全零，M1 看不见 R）；`acquisition_rolled_back` M1 取号两写之后
//! 那道屏障（整池第 3 次设备屏障调用）报错、取号回卷、M1 报失败。交回 M1 的写、段与一行摘要。
use crate::common_unwitnessed_publish::{self, fixed_geometry_of_the_384_slot_small_devices, recorded_devices_reading_back_all_zero_on, RecordedPublish};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::mount::mount_writable;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite};
use singlefs_harness::SharedStream;

pub fn first_mount(recorded: &RecordedPublish, crash_image: &MemoryPool, history: &str) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, String) {
    let stream = SharedStream::retaining_contents();
    let summary = if history == "acquisition_rolled_back" {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0u32));
        let mut devices: Vec<(DeviceIdentity, singlefs_harness::RecordingBlockDevice<FailingTheNthBarrier>)> =
            common_unwitnessed_publish::plain_devices_on(crash_image)
                .into_iter()
                .map(|(identity, inner)| {
                    (
                        identity,
                        singlefs_harness::RecordingBlockDevice::with_shared_stream(
                            identity,
                            FailingTheNthBarrier { inner, calls: calls.clone(), fail_at: 3 },
                            stream.clone(),
                        ),
                    )
                })
                .collect();
        let failure = mount_writable(&recorded.pool.parameters, &mut devices).expect_err("M1 的取号回卷");
        let text = format!("{failure:?}");
        format!("m1=failed:{}", text.chars().filter(|c| !c.is_whitespace()).take(120).collect::<String>())
    } else {
        let faults = match history {
            "clean" => crate::ranges(recorded, "none"),
            "tree_table_and_record" => crate::ranges(recorded, "tree_table_and_record"),
            "root_units_and_record" => crate::ranges(recorded, "root_units_and_record"),
            other => panic!("认不出历史 {other}"),
        };
        let mut devices = recorded_devices_reading_back_all_zero_on(crash_image, &faults, &stream);
        let first = mount_writable(&recorded.pool.parameters, &mut devices).expect("M1 做得成");
        format!(
            "m1_instance={} m1_effective=({},{})",
            first.output.instance.0,
            first.output.effective_root.instance.0,
            first.output.effective_root.checkpoint_txg.0
        )
    };
    let (writes, segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    (writes, segments, summary)
}

/// 整池第 `fail_at` 次设备屏障调用报错，别的照转给稀疏盘（几块盘共用一个计数）。
pub struct FailingTheNthBarrier {
    inner: singlefs_harness::memory_pool::SparseBlockDevice,
    calls: std::rc::Rc<std::cell::Cell<u32>>,
    fail_at: u32,
}

impl singlefs_core::block_device::BlockDevice for FailingTheNthBarrier {
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let call = self.calls.get() + 1;
        self.calls.set(call);
        if call == self.fail_at {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(
                std::io::Error::other("注入的屏障错"),
            ));
        }
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}
