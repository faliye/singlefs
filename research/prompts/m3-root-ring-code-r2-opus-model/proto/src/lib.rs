//! m3-root-ring-code-r2 攻方原型共用件。参数、建池与读故障包装的写法取自
//! `research/prompts/m3-root-ring-code-r1-opus-model/rr-attack-proto/src/lib.rs`（只留用得到的几个，读故障包装加了「读出来改一个字节」一档）。
#![allow(clippy::missing_panics_doc, clippy::must_use_candidate)]

use std::cell::Cell;
use std::rc::Rc;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"M3RRC-R2-OPUS-01";
pub const IMAGE_BYTES: u64 = 256 << 20;
pub const WRITE_TIME: u64 = 1_788_000_000;

pub fn parameters(slots_per_region: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: 1 << 20,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
                .expect("S 在 4..16"),
        },
    }
}

pub fn geometry(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

pub fn content(salt: u64) -> Vec<u8> {
    let salt = usize::try_from(salt % 251).expect("小");
    let length: usize = std::env::var("PROTO_CONTENT_BYTES").ok().and_then(|v| v.parse().ok()).unwrap_or(3000);
    (0..length).map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256")).collect()
}

pub type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub fn pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

pub fn devices_of(pool: &MemoryPool) -> Devices {
    pool.devices
        .iter()
        .map(|(identity, image)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = image.clone();
            (*identity, device)
        })
        .collect()
}

pub fn recording_of(pool: &MemoryPool, stream: &SharedStream) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    devices_of(pool)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect()
}

/// mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写到 txg `last`（每次都落盘）。
pub fn build_plain(slots_per_region: u64, last: u64) -> MemoryPool {
    let parameters = parameters(slots_per_region);
    let mut devices: Devices = (0..2u32)
        .map(|number| (DeviceIdentity(number), SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted { output, mut allocator, current } = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let instance = output.instance;
    let mut current: TransactionOutput = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(&mut pool, &mut allocator, current.root(), FirstFile { content: &content(0), write_time_seconds: WRITE_TIME }, instance, current.record_bytes()).expect("第一个文件")
    };
    while current.root.checkpoint_txg.0 < last {
        let salt = current.root.checkpoint_txg.0 + 1;
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, instance).expect("覆盖写");
    }
    pool_of(&devices)
}

/// 读故障包装：读到 (盘, [起, 止)) 这一段时按 `mode` 处置——0 照读，1 报块设备错，2 照读之后把这一段里第 `flip_at` 个字节取反
/// （「读出来自证不过」，盘上字节不变）。写、屏障照传。
pub struct FaultyDevice<Inner: BlockDevice> {
    pub device: DeviceIdentity,
    pub inner: Inner,
    pub target: (DeviceIdentity, u64, u64),
    pub mode: Rc<Cell<u8>>,
    pub flip_at: u64,
    pub hits: Rc<Cell<u64>>,
}

impl<Inner: BlockDevice> BlockDevice for FaultyDevice<Inner> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let start = offset.0;
        let end = start + buffer.len() as u64;
        let (device, a, b) = self.target;
        let overlaps = device == self.device && start < b && a < end;
        if overlaps {
            self.hits.set(self.hits.get() + 1);
        }
        if overlaps && self.mode.get() == 1 {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("m3-root-ring-code-r2 原型注入的读错")));
        }
        self.inner.read_at(offset, buffer)?;
        let at = a + self.flip_at;
        if overlaps && self.mode.get() == 2 && start <= at && at < end {
            let index = usize::try_from(at - start).expect("小");
            buffer[index] = !buffer[index];
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
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
}

/// 池级 checker 判违例的不变量，各带原文前 160 字。
pub fn red(image: &MemoryPool) -> Vec<String> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(text) => Some(format!("{name}：{}", text.chars().take(160).collect::<String>())),
            _ => None,
        })
        .collect()
}

/// 带文件的一版上的空发布（不改文件，只换根与记账）：取自 r1 攻方原型 lib.rs 的 `publish_empty`（同一个做法）。
pub fn publish_empty<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut singlefs_core::allocator::PoolAllocator,
    previous: &TransactionOutput,
    instance: singlefs_core::address::InstanceGeneration,
) -> Result<TransactionOutput, singlefs_core::transaction::PublishError> {
    use singlefs_core::transaction::{publish_version, InstanceTablePlan, PublishPlan};
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg: singlefs_core::address::CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
            counter: previous.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: previous.highest_transaction_number_in_this_instance,
            instance,
            back_chain: singlefs_core::journal::back_chain_of(&previous.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 回退候选集（攻方自己按条款算一份，判法用不到实现）：有效（最新根的实例表）∧ txg ≥ F_生效 ∧ 一圈窗口里（锚取最新根）∧ 不是最新根。
/// 同 [`rollback_targets`]，环境变量 FUZZ_ANY_TARGET=1 时不按一圈窗口滤（窗口外的有效根也交回，K2 拿它看实现拒不拒）。
pub fn rollback_targets(image: &dyn singlefs_core::recovery::PoolReader) -> Vec<(u32, u64)> {
    let any = std::env::var("FUZZ_ANY_TARGET").is_ok_and(|v| v == "1");
    rollback_targets_filtered(image, !any)
}

/// 一条 (实例, txg) 落不落在最新根的一圈窗口里（攻方按条款算）。
pub fn inside_the_window_of_the_newest(image: &dyn singlefs_core::recovery::PoolReader, txg: u64) -> bool {
    rollback_targets_filtered(image, true).iter().any(|(_, t)| *t == txg)
}

fn rollback_targets_filtered(image: &dyn singlefs_core::recovery::PoolReader, window: bool) -> Vec<(u32, u64)> {
    use singlefs_core::recovery::{choose_system_configuration, effective_rollback_floor, instance_table_of_root, readable_roots_with_ring_slots, root_is_abandoned_by_the_instance_table};
    let Ok(configuration) = choose_system_configuration(image) else { return Vec::new() };
    let i = &configuration.immutable;
    let roots: Vec<_> = readable_roots_with_ring_slots(image, &i.region_devices, &i.sizes, &i.filesystem_identifier).into_iter().map(|(_, r)| r).collect();
    let Some(newest) = roots.iter().copied().max_by_key(|r| (r.checkpoint_txg, r.instance)) else { return Vec::new() };
    let Some(table) = instance_table_of_root(image, &newest) else { return Vec::new() };
    let floor = effective_rollback_floor(image, &i.region_devices, &i.sizes, &i.filesystem_identifier);
    let ring = 3 * i.sizes.root_ring_slots_per_region.count();
    roots
        .iter()
        .filter(|r| **r != newest && !root_is_abandoned_by_the_instance_table(r, &table) && r.checkpoint_txg >= floor && (!window || r.checkpoint_txg.0 + ring > newest.checkpoint_txg.0))
        .map(|r| (r.instance.0, r.checkpoint_txg.0))
        .collect()
}
