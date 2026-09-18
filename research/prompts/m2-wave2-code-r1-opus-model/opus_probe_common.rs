//! m2-wave2-code-r1 云端攻方腿（Opus）探针的共用搭建：两块 4 GiB 稀疏内存盘，mkfs → 取号 → 暖机 → 第一个事务，
//! 之后按「在同一个进程里覆盖写 / 空发布」与「可写挂载」拼历史。只读 crates 的公开入口，外加副本里三处只读插桩
//! （`opus_probe_reclaimed_count`、`opus_probe_exact_record_counts`、`OPUS_PROBE_LOG`，见报告第零节的 diff）。
#![allow(dead_code)]

use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, MountError, Mounted, OPUS_PROBE_LOG, OpusProbePreAcquisition};
use singlefs_core::recovery::verified_superblock_slots;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_version, warm_up, FirstFile,
    InstanceTablePlan, PoolWriter, PublishError, PublishPlan, TransactionOutput,
};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};

pub const IMAGE_BYTES: u64 = 4 << 30;
pub const CAPACITY: usize = 812;

pub type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub fn parameters() -> MakeFilesystemParameters {
    e142_parameters(512, 512)
}

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

pub struct Pool {
    pub devices: Devices,
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
    pub instance: InstanceGeneration,
    pub publishes: usize,
}

pub fn fresh_pool_with_bytes(device_bytes: [u64; 2]) -> Pool {
    let parameters = parameters();
    let mut devices: Devices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    device_bytes[usize::try_from(number).expect("盘号")],
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), device_bytes[0]),
        DeviceFreeMap::new(DeviceIdentity(1), device_bytes[1]),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let current = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            &genesis.root,
            FirstFile { content: &first_file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个事务")
    };
    Pool { devices, allocator, current, instance: InstanceGeneration(1), publishes: 0 }
}

pub fn fresh_pool() -> Pool {
    fresh_pool_with_bytes([IMAGE_BYTES, IMAGE_BYTES])
}

/// 同一个进程里接着现行版本覆盖写一次。
pub fn overwrite(pool: &mut Pool, seed: usize) -> Result<TransactionOutput, PublishError> {
    let parameters = parameters();
    let content = content_of(3000 + seed % 97, seed);
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    let previous = pool.current.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
        pool.instance,
    )?;
    pool.current = output.clone();
    pool.publishes += 1;
    Ok(output)
}

/// 同一个进程里接着现行版本推一次空发布（四个固定点单元）。
pub fn empty_publish(pool: &mut Pool) -> Result<TransactionOutput, PublishError> {
    let parameters = parameters();
    let previous = pool.current.clone();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    let output = publish_version(
        &mut writer,
        &mut pool.allocator,
        PublishPlan {
            txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
            counter: previous.record.counter + 1,
            transaction: 0,
            instance: pool.instance,
            back_chain: back_chain_of(&previous.record_bytes),
            file: None,
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(&previous),
    )?;
    pool.current = output.clone();
    pool.publishes += 1;
    Ok(output)
}

pub enum MountOutcome {
    Mounted(Box<Mounted>),
    Refused(MountError),
    Panicked(String),
}

pub fn take_probe_log() -> Vec<OpusProbePreAcquisition> {
    OPUS_PROBE_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}

/// 可写挂载一次（panic 也接住），成功就把分配器、现行版本、实例代号换成挂载交回的。
pub fn remount(pool: &mut Pool) -> MountOutcome {
    let parameters = parameters();
    let devices = &mut pool.devices;
    let result = catch_unwind(AssertUnwindSafe(|| mount_writable(&parameters, devices)));
    match result {
        Ok(Ok(mounted)) => {
            pool.allocator = mounted.allocator.clone();
            pool.current = mounted.current.file_version().expect("带文件").clone();
            pool.instance = mounted.output.instance;
            MountOutcome::Mounted(Box::new(mounted))
        }
        Ok(Err(error)) => MountOutcome::Refused(error),
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
                .unwrap_or_else(|| "（非字符串 panic）".to_string());
            MountOutcome::Panicked(message)
        }
    }
}

/// 两块盘四个超级块槽里自证过的实例代号。
pub fn superblock_instances(pool: &Pool) -> Vec<(u32, Vec<u32>)> {
    let parameters = parameters();
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    (0..2u32)
        .map(|device| {
            let mut instances: Vec<u32> = verified_superblock_slots(
                &pool.devices,
                DeviceIdentity(device),
                spacing,
                &parameters.filesystem_identifier,
            )
            .iter()
            .map(|superblock| superblock.journal_instance.0)
            .collect();
            instances.sort_unstable();
            (device, instances)
        })
        .collect()
}
