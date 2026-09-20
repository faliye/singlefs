//! 取号（D23（journal 的角色与格式） 已定项 16、D18（块里携带什么信息） 已定项 11；C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案）：
//! 第二次取号看得见、世代号逐盘 +1、取号之后那道屏障、取号写或屏障报错时回卷成旧代号。每条都从写完第一个事务的镜像出发。

mod common;

use std::io;

use common::{build_pool, parameters, BuiltPool, Recorded};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::recovery::{choose_superblock, verified_superblock_slots};
use singlefs_core::transaction::{acquire_instance, AcquisitionRollback, CommitStep, PoolWriter};

/// 系统配置两槽住在偏移 0 与 4096，都在这个界之下。
const SUPERBLOCK_SLOTS_END_OFFSET: u64 = 8192;

/// 包在录制盘外面：按开关让屏障或系统配置槽写报错，并数真正交给设备的屏障。
struct FaultInjectingDevice {
    inner: Recorded,
    fail_barriers: bool,
    fail_superblock_writes: bool,
    barrier_calls: u64,
}

fn injected(what: &str) -> BlockDeviceError {
    BlockDeviceError::InputOutput(io::Error::other(format!("注入的{what}错")))
}

impl BlockDevice for FaultInjectingDevice {
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
        if self.fail_superblock_writes && offset.0 < SUPERBLOCK_SLOTS_END_OFFSET {
            return Err(injected("系统配置槽写"));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        if self.fail_barriers {
            return Err(injected("屏障"));
        }
        self.barrier_calls += 1;
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

fn wrap(built: &mut BuiltPool) -> Vec<(DeviceIdentity, FaultInjectingDevice)> {
    built
        .devices
        .take()
        .expect("第一个事务写完，盘还开着")
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                FaultInjectingDevice {
                    inner,
                    fail_barriers: false,
                    fail_superblock_writes: false,
                    barrier_calls: 0,
                },
            )
        })
        .collect()
}

/// 一块盘两槽里自证过的系统配置：(世代号, 实例代号)，按世代号排好。
fn slots_of(
    devices: &[(DeviceIdentity, FaultInjectingDevice)],
    device: DeviceIdentity,
) -> Vec<(u64, InstanceGeneration)> {
    let parameters = parameters();
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let mut slots: Vec<(u64, InstanceGeneration)> =
        verified_superblock_slots(devices, device, spacing, &parameters.filesystem_identifier)
            .iter()
            .map(|superblock| (superblock.slot_generation, superblock.journal_instance))
            .collect();
    slots.sort();
    slots
}

fn acquire(
    devices: &mut [(DeviceIdentity, FaultInjectingDevice)],
) -> Result<InstanceGeneration, singlefs_core::transaction::AcquisitionFailed> {
    let parameters = parameters();
    let mut pool = PoolWriter::new(&parameters, devices);
    acquire_instance(&mut pool)
}

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

#[test]
fn second_acquisition_writes_generation_six_on_both_disks_and_the_next_acquisition_gets_three() {
    let mut built = build_pool("acquire-second");
    let mut devices = wrap(&mut built);
    assert_eq!(
        acquire(&mut devices).expect("第二次取号"),
        InstanceGeneration(2)
    );
    for disk in DISKS {
        assert_eq!(
            slots_of(&devices, disk),
            vec![(5, InstanceGeneration(1)), (6, InstanceGeneration(2))],
            "{disk:?}：第一个事务留下世代 5，第二次取号按那块盘自证过的最大世代号 + 1 写世代 6（D22（单元原子性怎么合成） 已定项 16 逐盘计）"
        );
    }
    assert_eq!(
        choose_superblock(&devices)
            .expect("择系统配置")
            .journal_instance,
        InstanceGeneration(2),
        "择到的那一份带新号：取号的世代号若恒为 2，会被世代 5 的旧槽藏住、这里读回 1"
    );
    assert_eq!(
        acquire(&mut devices).expect("第三次取号"),
        InstanceGeneration(3)
    );
}

#[test]
fn failed_barrier_after_acquisition_rolls_both_disks_back_and_the_skipped_number_is_never_handed_out_again(
) {
    let mut built = build_pool("acquire-barrier-error");
    let mut devices = wrap(&mut built);
    for (_, device) in &mut devices {
        device.fail_barriers = true;
    }
    let failure =
        acquire(&mut devices).expect_err("取号之后那道屏障报错，取号必须失败、不交出新号");
    assert!(
        matches!(failure.rollback, AcquisitionRollback::RolledBack),
        "两份取号写都报过 Ok，都要回卷：{failure:?}"
    );
    for disk in DISKS {
        assert_eq!(
            slots_of(&devices, disk),
            vec![(6, InstanceGeneration(2)), (7, InstanceGeneration(1))],
            "{disk:?}：取号写世代 6 带新号 2，回卷写世代 7 带取号之前全部自证过的槽中最大的号 1"
        );
    }
    for (_, device) in &mut devices {
        device.fail_barriers = false;
    }
    assert_eq!(
        acquire(&mut devices).expect("屏障好了再取号"),
        InstanceGeneration(3),
        "全部自证过的槽里还躺着号 2：跳过它；只读择到的那一份（世代 7、号 1）就会再发一次 2"
    );
}

#[test]
fn failed_superblock_write_on_the_second_disk_rolls_the_first_disk_back_and_leaves_the_second_untouched(
) {
    let mut built = build_pool("acquire-write-error");
    let mut devices = wrap(&mut built);
    devices
        .iter_mut()
        .find(|(identity, _)| *identity == DeviceIdentity(1))
        .expect("盘 1")
        .1
        .fail_superblock_writes = true;
    let failure = acquire(&mut devices).expect_err("盘 1 的取号写报错，取号必须失败");
    assert!(
        matches!(failure.rollback, AcquisitionRollback::RolledBack),
        "盘 0 那份已写出，要回卷：{failure:?}"
    );
    assert_eq!(
        slots_of(&devices, DeviceIdentity(0)),
        vec![(6, InstanceGeneration(2)), (7, InstanceGeneration(1))]
    );
    assert_eq!(
        slots_of(&devices, DeviceIdentity(1)),
        vec![(4, InstanceGeneration(1)), (5, InstanceGeneration(1))],
        "盘 1 一个字节都没写成"
    );
}

#[test]
fn barrier_right_after_the_acquisition_barrier_is_not_sent_to_the_devices() {
    let mut built = build_pool("acquire-barrier-skip");
    let mut devices = wrap(&mut built);
    {
        let parameters = parameters();
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        acquire_instance(&mut pool).expect("取号");
        pool.perform(CommitStep::Barrier).expect("紧跟着的一道屏障");
    }
    for (identity, device) in &devices {
        assert_eq!(
            device.barrier_calls, 1,
            "{identity:?}：取号之后那道屏障发一次；紧跟着的那道前面没有写，不再发——首次挂载路径上暖机开场那道就这样并掉，设备收到的 FLUSH 数不变"
        );
    }
}
