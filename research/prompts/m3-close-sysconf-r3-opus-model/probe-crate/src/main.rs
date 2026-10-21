//! m3-close-sysconf-r3 攻方原型（由第二轮攻方原型改来：盘数可选 2 或 3、加 S3 判据、加 r3 与 alpha_prime 两种模式）。原第二轮说明：m3-close-sysconf-r2 攻方原型（只在攻方副本里编、直接执行编出来的二进制）。
//! 崩溃状态取崩溃点重放的同一个计划（`singlefs_checker_tier::crash::visit_crash_state_range_of_the_read_back_flow_with_slices`，
//! 副本里把它从 pub(crate) 改成 pub；展开取 EveryProperSubset，原地覆写取三态），每个状态自己判：后续故障、再可写挂载、池级 checker。
//! 取号一轮 / 两轮由环境变量 SINGLEFS_PROBE_ACQUISITION 选（副本 transaction.rs），回卷逐份 / 每盘一次由 SINGLEFS_PROBE_ROLLBACK 选。
#![allow(dead_code, clippy::too_many_lines)]

#[path = "../../singlefs-harness/tests/common_admission_pool/mod.rs"]
mod common_admission_pool;
mod probe_alpha;
mod probe_alpha_prime;
mod probe_body;
mod probe_r3;
pub use probe_alpha::probe_clone as probe_alpha_clone;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, PoolUnderTest,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use std::cell::Cell;
use std::rc::Rc;

/// 盘外包一层：点名的 4 KiB 槽读一律报错（持续读错）；`writes_fail_on_unreadable` 时写那几个槽也报错（写不进去）；
/// `fail_sysconf_write_from` 是整池第几次系统配置槽写起报错（Some(n)：第 n 次；`only_once` 时只那一次）。
pub struct ProbeDevice {
    pub identity: DeviceIdentity,
    pub inner: SparseBlockDevice,
    pub unreadable: Vec<u64>,
    pub writes_fail_on_unreadable: bool,
    pub sysconf_spacing: u64,
    pub sysconf_writes: Rc<Cell<u64>>,
    pub fail_sysconf_write_from: Option<u64>,
    pub only_once: bool,
}

impl ProbeDevice {
    fn hits_unreadable(&self, offset: u64, length: u64) -> bool {
        self.unreadable
            .iter()
            .any(|start| offset < start + 4096 && *start < offset + length)
    }
}

impl BlockDevice for ProbeDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if self.hits_unreadable(offset.0, buffer.len() as u64) {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if offset.0 == 0 || offset.0 == self.sysconf_spacing {
            let ordinal = self.sysconf_writes.get() + 1;
            self.sysconf_writes.set(ordinal);
            if let Some(first) = self.fail_sysconf_write_from {
                if (self.only_once && ordinal == first) || (!self.only_once && ordinal >= first) {
                    return Err(injected_block_device_error("写"));
                }
            }
        }
        if self.writes_fail_on_unreadable && self.hits_unreadable(offset.0, bytes.len() as u64) {
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
}

pub fn probe_devices_on(image: &MemoryPool, spacing: u64) -> Vec<(DeviceIdentity, ProbeDevice)> {
    let counter = Rc::new(Cell::new(0));
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                ProbeDevice {
                    identity,
                    inner,
                    unreadable: Vec::new(),
                    writes_fail_on_unreadable: false,
                    sysconf_spacing: spacing,
                    sysconf_writes: counter.clone(),
                    fail_sysconf_write_from: None,
                    only_once: true,
                },
            )
        })
        .collect()
}

pub fn image_of_probe(devices: &[(DeviceIdentity, ProbeDevice)], bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.inner.image.clone())).collect(),
        device_size_in_bytes: bytes,
    }
}

/// 起点（前缀，用户步骤放开扫的那几样）：`overwrites` 次会话覆盖写、`remount_first` 时先崩一次再可写挂载（实例 2）再写一次；都不卸载（进程退出）。
pub fn starting_point(overwrites: usize, remount_first: bool) -> (MemoryPool, singlefs_core::root_record::RootRecord) {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    for _ in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).expect("会话覆盖写");
    }
    if remount_first {
        pool.crash_and_mount_writable().expect("崩后再可写挂载");
        pool.overwrite(OVERWRITE_BYTES).expect("新实例覆盖写一次");
    }
    let root = *pool.session().current.root();
    (pool.image(), root)
}

/// 池的参数：PROBE_DEVICES=3 时三块盘（根环三个区域各在一块盘上），否则两块盘（384 槽小盘那一份）。
pub fn pool_parameters() -> singlefs_core::make_filesystem::MakeFilesystemParameters {
    let mut parameters = parameters_of_the_384_slot_small_devices();
    if device_count() == 3 {
        parameters.region_devices = [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(2)];
    }
    parameters
}

pub fn device_count() -> u32 {
    std::env::var("PROBE_DEVICES").ok().and_then(|value| value.parse().ok()).unwrap_or(2)
}

/// 起点：两块盘照第二轮的 `starting_point`；三块盘只有 k0（mkfs、取号、暖机、第一个文件之后直接崩），由 harness 库的
/// `run_new_pool_file_creation` 在三块 384 槽小盘上走一遍。
pub fn starting_point_any(overwrites: usize, remount_first: bool) -> (MemoryPool, singlefs_core::root_record::RootRecord) {
    if device_count() == 2 {
        return starting_point(overwrites, remount_first);
    }
    assert!(overwrites == 0 && !remount_first, "三块盘只做 k0 起点");
    let parameters = pool_parameters();
    let bytes = device_bytes_of_the_384_slot_small_devices();
    let stream = singlefs_harness::SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, singlefs_harness::RecordingBlockDevice<SparseBlockDevice>)> = (0..device_count())
        .map(|number| {
            let identity = DeviceIdentity(number);
            (identity, singlefs_harness::RecordingBlockDevice::with_shared_stream(identity, SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512)), stream.clone()))
        })
        .collect();
    let run = singlefs_harness::scenario::run_new_pool_file_creation(&parameters, &mut devices, &stream, |_, _| {}).expect("三块盘上新池新建文件");
    let image = MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
        device_size_in_bytes: bytes,
    };
    (image, run.output.root)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    probe_body::run(&args[1..]);
}
