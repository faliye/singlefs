//! `m2-safety-r3` 辩方（Sonnet）自建的最小搭建：小盘 mkfs → 取号 → 暖机 → 第一个事务，
//! 之后逐次覆盖写、正常卸载/崩了再挂、C283（自己实现）重试，都在这个模块里直接调
//! `singlefs_core::mount` / `transaction` 的公开函数，不经 `history.rs` 那一整套模型对拍机器
//! （候选与 C283 都是这一轮新加的分支，模型不认得，跑起来只会被判成「模型答不上」）。
#![allow(dead_code, reason = "报告与用例各自只用到其中一部分")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::S4Candidate;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{FileBackedBlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable_with_admission_candidate, mount_writable_with_admission_candidate_and_c283,
    raise_rollback_floor, roll_back_by_a_forward_publish, rollback_floor_ceiling, unmount,
    MountError, RollbackError, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{choose_system_configuration, instance_table_chain_of_root};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, PublishError, TransactionOutput,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);
pub const FILE_BYTES: usize = 3000;
pub const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

pub type Recorded = RecordingBlockDevice<FileBackedBlockDevice>;

fn image_path(tag: &str, device: u32) -> PathBuf {
    let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "singlefs-r3-defense-{tag}-{}-{sequence}-dev{device}.img",
        std::process::id()
    ))
}

/// 每次内容不同：defer 窗口按「4 个不同状态」去重，同一个字节反复写会被判成同一个状态。
pub fn content_of(seed: u64) -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + usize::try_from(seed % 251).expect("取余")) % 251).expect("小于 256"))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    /// 正常卸载再挂：`mount::unmount` 把 F 抬到现行那一版的 txg（B1），再重开镜像可写挂载。
    NormalUnmount,
    /// 崩了再挂：不调 `unmount`，直接丢弃进程内状态、重开镜像可写挂载（与 `CloseAndMountWritable` 同一个模拟）。
    Crash,
}

pub struct DefensePool {
    pub paths: Vec<PathBuf>,
    pub devices: Option<Vec<(DeviceIdentity, Recorded)>>,
    pub parameters: MakeFilesystemParameters,
    pub device_bytes: u64,
    pub allocator: PoolAllocator,
    pub output: TransactionOutput,
    pub instance: InstanceGeneration,
}

impl Drop for DefensePool {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = std::fs::remove_file(path);
        }
    }
}

pub fn build_pool(tag: &str, width: HistoryDeviceWidth, candidate: S4Candidate) -> DefensePool {
    let parameters = width.parameters();
    let device_bytes = width.device_bytes();
    let mut paths = Vec::new();
    let mut devices: Vec<(DeviceIdentity, Recorded)> = Vec::new();
    let stream = SharedStream::new();
    for device_number in 0..2u32 {
        let path = image_path(tag, device_number);
        let file = FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            device_bytes,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像");
        devices.push((
            DeviceIdentity(device_number),
            RecordingBlockDevice::with_shared_stream(
                DeviceIdentity(device_number),
                file,
                stream.clone(),
            ),
        ));
        paths.push(path);
    }
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
        DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator.set_s4_candidate(candidate);
    let content = content_of(0);
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        let warm_up = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up.last_record_bytes,
        )
        .expect("第一个事务")
    };
    DefensePool {
        paths,
        devices: Some(devices),
        parameters,
        device_bytes,
        allocator,
        output,
        instance: InstanceGeneration(1),
    }
}

impl DefensePool {
    /// 丢掉进程内的设备句柄，按路径重开镜像（新的一批录制流，与本文件的度量无关）。
    pub fn reopen(&mut self) -> Vec<(DeviceIdentity, Recorded)> {
        drop(self.devices.take());
        let stream = SharedStream::new();
        self.paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                let file = FileBackedBlockDevice::open_existing_image_file(
                    path,
                    self.device_bytes,
                    PhysicalBlockSizeInBytes(512),
                )
                .expect("重开镜像");
                let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, file, stream.clone()),
                )
            })
            .collect()
    }

    /// 同一个会话里接着现行版本覆盖写一次；错误原样交回。
    pub fn overwrite(&mut self, seed: u64) -> Result<TransactionOutput, PublishError> {
        let devices = self.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&self.parameters, devices.as_mut_slice());
        let previous = self.output.clone();
        let content = content_of(seed);
        let output = publish_overwrite(
            &mut writer,
            &mut self.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + seed,
            },
            self.instance,
        )?;
        self.output = output.clone();
        Ok(output)
    }

    /// 本文件自己实现的 C283（辩方，`m2-safety-r3`，不用攻方的）：覆盖写被空间准入拒了（且这次拒绝不是写行那次发布——
    /// 这里恒不是，写行只在挂载入口）就先推一次空发布把 F 抬到上限，重判一次；抬不动就原样报第一次的拒绝。
    /// 交回 `(结局, 这一次是不是抬过 F)`。
    pub fn overwrite_with_c283(&mut self, seed: u64) -> (Result<TransactionOutput, PublishError>, bool) {
        match self.overwrite(seed) {
            Ok(output) => (Ok(output), false),
            Err(PublishError::SpaceAdmissionRefused(refusal)) => match self.raise_the_floor_once() {
                Ok(true) => (self.overwrite(seed), true),
                Ok(false) => (Err(PublishError::SpaceAdmissionRefused(refusal)), false),
                Err(_raise_error) => (Err(PublishError::SpaceAdmissionRefused(refusal)), false),
            },
            Err(other) => (Err(other), false),
        }
    }

    /// C283 的一步：算出抬 F 的上限（D16（发布语义） 已定项 1「抬 F 的上限」），够抬就抬一次。
    fn raise_the_floor_once(&mut self) -> Result<bool, MountError> {
        let devices = self.devices.as_mut().expect("镜像还开着");
        let system_configuration = choose_system_configuration(&*devices)?;
        let table = instance_table_chain_of_root(&*devices, &self.output.root)
            .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?
            .records;
        let ceiling = rollback_floor_ceiling(
            devices,
            &system_configuration,
            self.output.root.rollback_floor,
            &table,
        )?;
        if ceiling <= self.output.root.rollback_floor {
            return Ok(false);
        }
        raise_rollback_floor(
            &self.parameters,
            devices,
            &mut self.allocator,
            &mut self.output,
            ceiling,
            ShadowLedger::On,
        )?;
        Ok(true)
    }

    /// 两支之一收掉这个会话，再重开可写挂载；`c283` 选走不走候选自己的 `_and_c283` 入口。
    /// 做成就把 `self.output` / `self.allocator` / `self.instance` 换成挂载交回的那一份；不成也把设备句柄放回去
    /// （挂载被拒不动盘，句柄仍然可用），交回原始错误。
    pub fn remount(
        &mut self,
        branch: Branch,
        candidate: S4Candidate,
        c283: bool,
    ) -> Result<(), MountError> {
        if branch == Branch::NormalUnmount {
            let devices = self.devices.as_mut().expect("镜像还开着");
            let mut version = PoolVersion::WithFile(self.output.clone());
            unmount(
                &self.parameters,
                devices,
                &mut self.allocator,
                &mut version,
                ShadowLedger::On,
            )?;
        }
        let mut devices = self.reopen();
        let mounted = if c283 {
            mount_writable_with_admission_candidate_and_c283(&self.parameters, &mut devices, candidate)
        } else {
            mount_writable_with_admission_candidate(&self.parameters, &mut devices, candidate)
        };
        match mounted {
            Ok(mounted) => {
                self.devices = Some(devices);
                self.allocator = mounted.allocator;
                self.instance = mounted.output.instance;
                self.output = mounted
                    .current
                    .into_file_version()
                    .expect("待文件的一版上写行、暖机之后现行那一版仍带文件");
                Ok(())
            }
            Err(error) => {
                self.devices = Some(devices);
                Err(error)
            }
        }
    }

    /// 吸收态探针里的一次回退（挂着时的一次向前发布，`mount::roll_back_by_a_forward_publish`）：这里**不**另包 C283——
    /// 只测今天字面代码能不能回退出这个格；报告据实标「没有另包 C283」。
    pub fn rollback_once(&mut self, target: RollbackTarget) -> Result<(), RollbackError> {
        let devices = self.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &self.parameters,
            devices,
            &mut self.allocator,
            &mut self.output,
            target,
        )
        .map(|_rolled_back| ())
    }
}
