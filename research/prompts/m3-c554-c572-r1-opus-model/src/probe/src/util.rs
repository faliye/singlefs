//! 几格共用的：环境参数、读回全零的故障层、起点池、按盘拿掉一块盘、各类落点的字节区间、恢复读回什么。
use std::cell::RefCell;
use std::rc::Rc;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::TransactionOutput;
use singlefs_format::{JOURNAL_RECORD_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

use crate::pool::{self, PoolUnderTest};

pub fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
pub fn env_list(name: &str, default: &str) -> Vec<String> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// 一段 (盘, 起点字节, 长度)。
pub type Range = (DeviceIdentity, u64, u64);

/// 读回全零的故障层：落在 `zeros` 里任一段的字节读回 0（报成功），`lifted` 之后照读。写、清零、屏障原样往下交。
pub struct Faulty<D: BlockDevice> {
    pub id: DeviceIdentity,
    pub inner: D,
    pub zeros: Rc<RefCell<(Vec<Range>, bool)>>,
}
impl<D: BlockDevice> BlockDevice for Faulty<D> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let state = self.zeros.borrow();
        if state.1 {
            return Ok(());
        }
        let len = buffer.len() as u64;
        for (device, start, length) in &state.0 {
            if *device != self.id || *start >= offset.0 + len || offset.0 >= start + length {
                continue;
            }
            let from = start.saturating_sub(offset.0) as usize;
            let to = ((start + length).min(offset.0 + len) - offset.0) as usize;
            buffer[from..to].fill(0);
        }
        Ok(())
    }
    fn write_at(&mut self, o: DeviceOffsetInBytes, b: &[u8], d: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(o, b, d)
    }
    fn write_zeroes_at(&mut self, o: DeviceOffsetInBytes, l: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(o, l)
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
    fn journal_record_offsets_hint(&self, s: DeviceOffsetInBytes, b: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(s, b)
    }
}

pub type Rec = RecordingBlockDevice<SparseBlockDevice>;

/// 起点池：384 槽小盘（`common_admission_pool` 那一档），mkfs、取号 1、暖机、第一个文件，录制流带内容。
pub fn start_pool(stream: &SharedStream) -> PoolUnderTest<Rec> {
    let stream = stream.clone();
    PoolUnderTest::start_after_the_first_file_on_devices_of(
        pool::parameters_of_the_384_slot_small_devices(),
        device_bytes(),
        move |id, dev| RecordingBlockDevice::with_shared_stream(id, dev, stream.clone()),
    )
}

/// 每块盘的字节数：384 槽那一档，单元区槽数换成 `UNIT_SLOTS`（没设 384）。
pub fn device_bytes() -> u64 {
    pool::device_bytes_of_the_384_slot_small_devices() - 384 * SLOT_BYTES + env_u64("UNIT_SLOTS", 384) * SLOT_BYTES
}

pub fn params() -> MakeFilesystemParameters {
    pool::parameters_of_the_384_slot_small_devices()
}

/// 一份镜像拿掉一块盘（`lost`）之后剩下的样子。
pub fn image_without(image: &MemoryPool, lost: Option<u32>) -> MemoryPool {
    let mut copy = image.clone();
    if let Some(lost) = lost {
        copy.devices.remove(&DeviceIdentity(lost));
    }
    copy
}

/// txg 那一版的根槽（一个物理块）。
pub fn root_slot_range(txg: CheckpointTxg) -> Range {
    let p = params();
    let target = target_for_publish(txg, p.geometry.root_ring_slots_per_region);
    let device = p.region_devices[target.region as usize];
    let offset = slot_offset(target, p.geometry.fixed_structure_slot_spacing).0;
    (device, offset, u64::from(p.geometry.physical_block_size))
}

/// 一次发布写的每条记录（两块盘各一份）。
pub fn record_ranges(output: &TransactionOutput) -> Vec<Range> {
    let ring = params().geometry.journal_ring_bytes;
    let counters = output
        .earlier_records_of_this_publish
        .iter()
        .map(|w| w.record.counter)
        .chain(std::iter::once(output.record.counter));
    counters
        .flat_map(|c| {
            let off = singlefs_core::journal::record_offset(c, ring).0;
            [DeviceIdentity(0), DeviceIdentity(1)].map(|d| (d, off, JOURNAL_RECORD_BYTES))
        })
        .collect()
}

/// 一次发布新分配的单元（账里分配代 = 这次 txg、仍分配的那几条，两块盘各一份）。
pub fn new_unit_ranges(output: &TransactionOutput) -> Vec<Range> {
    output
        .allocation_records
        .iter()
        .filter(|r| r.generation == output.root.checkpoint_txg && !r.is_released)
        .map(|r| (r.device, r.slot.0 * SLOT_BYTES, u64::from(r.span_slots) * SLOT_BYTES))
        .collect()
}

/// 一次发布的数据单元（两份）。
pub fn data_unit_ranges(output: &TransactionOutput) -> Vec<Range> {
    let bytes = singlefs_format::DATA_UNIT_BYTES;
    output
        .data_pointers
        .iter()
        .flat_map(|p| p.locations)
        .map(|l| (l.device, l.slot.0 * SLOT_BYTES, bytes))
        .collect()
}

/// 一次发布的账：树表那一片（两份）；读不出它，`placements_referenced_by_root` 就读不出这条根的账。
pub fn account_ranges(output: &TransactionOutput) -> Vec<Range> {
    output
        .root
        .tree_table
        .locations
        .iter()
        .map(|l| (l.device, l.slot.0 * SLOT_BYTES, SLOT_BYTES))
        .collect()
}

/// 恢复（看 journal）落到哪条根、读回什么：`Ok(内容)`、`NoFile`、`Err(失败说明)`。
pub fn recovered(image: &MemoryPool) -> (Option<(InstanceGeneration, CheckpointTxg)>, Result<Vec<u8>, String>) {
    let report = recover(image, JournalPolicy::Consult);
    match report.outcome {
        RecoveryOutcome::FileRead { root, content } => (Some(root), Ok(content)),
        RecoveryOutcome::NoFile { root } => (Some(root), Err("no_file".into())),
        RecoveryOutcome::Failed { root, failure } => (root, Err(format!("{failure:?}"))),
    }
}

/// 一份镜像交给两块新的稀疏盘，外面包录制（录进 `stream`）与读回全零的故障层。
pub fn faulty_recorded_devices(
    image: &MemoryPool,
    stream: &SharedStream,
    zeros: &Rc<RefCell<(Vec<Range>, bool)>>,
) -> Vec<(DeviceIdentity, Faulty<Rec>)> {
    pool::plain_devices_on(image)
        .into_iter()
        .map(|(id, dev)| {
            (
                id,
                Faulty {
                    id,
                    inner: RecordingBlockDevice::with_shared_stream(id, dev, stream.clone()),
                    zeros: zeros.clone(),
                },
            )
        })
        .collect()
}

/// 几块盘此刻的整份镜像。
pub fn image_of<D: BlockDevice>(devices: &[(DeviceIdentity, Faulty<RecordingBlockDevice<D>>)], bytes: u64) -> MemoryPool
where
    D: AsSparse,
{
    MemoryPool {
        devices: devices
            .iter()
            .map(|(id, d)| (*id, d.inner.wrapped_device().sparse().clone()))
            .collect(),
        device_size_in_bytes: bytes,
    }
}

pub trait AsSparse {
    fn sparse(&self) -> &singlefs_harness::memory_pool::SparseDevice;
}
impl AsSparse for SparseBlockDevice {
    fn sparse(&self) -> &singlefs_harness::memory_pool::SparseDevice {
        &self.image
    }
}

/// 崩溃镜像落成一份内存池：基线再施加持久了的那几条写。
pub fn materialize(
    base: &MemoryPool,
    writes: &[singlefs_harness::memory_pool::RetainedWrite],
    persisted: &[bool],
) -> MemoryPool {
    let mut image = base.clone();
    for (write, is_persisted) in writes.iter().zip(persisted) {
        if *is_persisted {
            write
                .contents
                .apply_to(image.devices.get_mut(&write.device).expect("写落在池里的盘上"), write.offset);
        }
    }
    image
}

/// 恢复结局归成一个短名：落到哪一类根、读对没读对。`expected` 是每条根该读回的内容。
pub fn classify(
    root: Option<(InstanceGeneration, CheckpointTxg)>,
    result: &Result<Vec<u8>, String>,
    abandoned: (InstanceGeneration, CheckpointTxg),
    expected: &std::collections::BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
) -> &'static str {
    let on_abandoned = root == Some(abandoned);
    match (on_abandoned, result) {
        (true, Ok(content)) if expected.get(&abandoned) == Some(content) => "abandoned_read_ok",
        (true, Ok(_)) => "abandoned_read_wrong",
        (true, Err(_)) => "abandoned_failed",
        (false, Ok(content)) => match root.and_then(|r| expected.get(&r)) {
            Some(want) if want == content => "other_read_ok",
            _ => "other_read_wrong",
        },
        (false, Err(e)) if e == "no_file" => "other_no_file",
        (false, Err(_)) => "other_failed",
    }
}

/// 只读挂载落到哪条根（施加之后的那一条）、读回整个文件：`Ok(内容)`，失败带说明。
pub fn read_only(image: &MemoryPool) -> (Option<(InstanceGeneration, CheckpointTxg)>, Result<Vec<u8>, String>) {
    use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
    let mounted = match singlefs_core::mounted_read::mount_read_only(image) {
        Ok(m) => m,
        Err(e) => return (None, Err(format!("RoMount{e:?}"))),
    };
    let root = Some((mounted.effective_root.instance, mounted.effective_root.checkpoint_txg));
    let inode = InodeNumber(singlefs_core::transaction::FIRST_INODE_NUMBER);
    let Some(size) = mounted.mounted.inode_records().iter().find(|r| r.inode == inode.0).map(|r| r.size) else {
        return (root, Err("no_file".into()));
    };
    let file = match mounted.mounted.open_file(image, inode) {
        Ok(f) => f,
        Err(e) => return (root, Err(format!("RoOpen{e:?}"))),
    };
    match file.read_at(image, FileOffsetInBytes(0), size) {
        Ok(out) => (root, Ok(out.bytes)),
        Err(e) => (root, Err(format!("RoRead{e:?}"))),
    }
}
