//! 历史驱动：两块内存稀疏盘、开着内容保留的录制流，按一串操作调 core 今天的公开入口；崩溃 = 拿那一步之前的镜像叠上那一步的一段前缀。
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::{
    publish_first_file, FirstFile, PoolVersion, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

pub type Device = RecordingBlockDevice<SparseBlockDevice>;
pub const WRITE_TIME: u64 = 1_788_000_000;

/// 由用户决定的一步（崩溃不在这里：崩溃是「做这一步、只落它的一段前缀」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Mount,
    FirstFile(u64),
    Overwrite(u64),
    SequentialWrite(u64),
    Unlink,
    CreateInode,
    Rollback(u64),
    /// 回退到根环里 txg 等于它的那条根（造坏镜像用，不进随机抽）。
    RollbackToTxg(u64),
    RaiseFloor(u64),
    Unmount,
}

pub struct Driver {
    pub width: HistoryDeviceWidth,
    pub params: MakeFilesystemParameters,
    pub stream: SharedStream,
    pub devices: Vec<(DeviceIdentity, Device)>,
    pub session: Option<MountedSession>,
}

pub fn content(selector: u64, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index as u64 * 7 + selector * 13 + 1) % 251).expect("<256"))
        .collect()
}

impl Driver {
    pub fn mkfs(width: HistoryDeviceWidth) -> Self {
        let params = width.parameters();
        let stream = SharedStream::retaining_contents();
        let devices = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                let device = SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512));
                (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()))
            })
            .collect();
        let mut driver = Driver { width, params, stream, devices, session: None };
        make_filesystem(&driver.params, &mut driver.devices).expect("mkfs");
        driver
    }

    /// 从一张镜像起一个新进程（没有会话、录制流从空起）。
    pub fn from_image(width: HistoryDeviceWidth, image: &MemoryPool) -> Self {
        let params = width.parameters();
        let stream = SharedStream::retaining_contents();
        let devices = image
            .devices
            .iter()
            .map(|(identity, sectors)| {
                let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
                device.image = sectors.clone();
                (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
            })
            .collect();
        Driver { width, params, stream, devices, session: None }
    }

    pub fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                .collect(),
            device_size_in_bytes: self.width.device_bytes(),
        }
    }

    pub fn geometry(&self) -> FixedGeometry {
        FixedGeometry {
            fixed_structure_slot_spacing: self.params.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: self.params.geometry.journal_ring_bytes,
            root_ring_slots_per_region: self.params.geometry.root_ring_slots_per_region,
        }
    }

    pub fn ops_since(&self, start: usize) -> Vec<RetainedOperation> {
        self.stream.retained_operations()[start..].to_vec()
    }

    /// 现行那一版的种类：0 没有会话，1 树表 0 条，2 带文件，3 有树而没有文件。
    pub fn state(&self) -> u8 {
        match &self.session {
            None => 0,
            Some(session) => match session.current {
                PoolVersion::WithoutFile(_) => 1,
                PoolVersion::WithFile(_) => 2,
                PoolVersion::WithTreesAndNoFile(_) => 3,
            },
        }
    }
}

/// 根环里读得出、自证过的根，按 (txg, 实例) 从新到旧、去重（checker 的读法，不与实现共用代码）。
pub fn ring_roots_newest_first(image: &MemoryPool) -> Vec<(u64, u32)> {
    use singlefs_checker::image::{chosen_system_configurations, valid_roots};
    let Some(geometry) = chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
    else {
        return Vec::new();
    };
    let mut roots: Vec<(u64, u32)> = valid_roots(image, &geometry)
        .into_iter()
        .map(|(_, _, view)| (view.checkpoint_txg, view.instance))
        .collect();
    roots.sort_unstable();
    roots.dedup();
    roots.reverse();
    roots
}

impl Driver {
    /// 做一步；交回结局的一句话（`ok …` / `refused …` / `na`）。
    pub fn execute(&mut self, op: Op) -> String {
        let payload = data_unit_payload_capacity();
        let Driver { width, params, devices, session, .. } = self;
        match op {
            Op::Mount => {
                if session.is_some() {
                    return "na".into();
                }
                match mount_writable(params, devices) {
                    Ok(mounted) => {
                        *session = Some(MountedSession::of_the_mount(mounted).1);
                        "ok".into()
                    }
                    Err(error) => format!("refused {error:?}"),
                }
            }
            Op::FirstFile(selector) => {
                let Some(session) = session.as_mut() else { return "na".into() };
                let PoolVersion::WithoutFile(current) = &session.current else { return "na".into() };
                let bytes = if selector == u64::MAX { Vec::new() } else { content(selector, 1 + (selector as usize % (payload - 2))) };
                let root = current.root;
                let record_bytes = current.record_bytes.clone();
                let mut writer = PoolWriter::new(params, devices.as_mut_slice());
                match publish_first_file(
                    &mut writer,
                    &mut session.allocator,
                    &root,
                    FirstFile { content: &bytes, write_time_seconds: WRITE_TIME },
                    session.instance,
                    &record_bytes,
                ) {
                    Ok(output) => {
                        session.current = PoolVersion::WithFile(output);
                        "ok".into()
                    }
                    Err(error) => format!("refused {error:?}"),
                }
            }
            Op::Overwrite(selector) | Op::SequentialWrite(selector) => {
                let Some(session) = session.as_mut() else { return "na".into() };
                let (bytes, sequential) = match op {
                    Op::Overwrite(_) => (content(selector, 1 + (selector as usize % (payload - 2))), false),
                    _ => (content(selector, payload * (1 + selector as usize % 3) + 17), true),
                };
                let file = FirstFile { content: &bytes, write_time_seconds: WRITE_TIME + 60 };
                let change = if sequential { UserChange::SequentialWrite(file) } else { UserChange::Overwrite(file) };
                match session.publish_user_change(devices, change) {
                    Ok(published) => format!("ok raises={}", published.floor_raises.len()),
                    Err(error) => format!("refused {error:?}"),
                }
            }
            Op::Unlink | Op::CreateInode => {
                let Some(session) = session.as_mut() else { return "na".into() };
                let change = match op {
                    Op::Unlink => UserChange::Unlink {
                        inode: InodeNumber(FIRST_INODE_NUMBER),
                        write_time_seconds: WRITE_TIME + 120,
                    },
                    _ => UserChange::NewInodes { count: 1, write_time_seconds: WRITE_TIME + 120 },
                };
                match session.publish_user_change(devices, change) {
                    Ok(published) => format!("ok raises={}", published.floor_raises.len()),
                    Err(error) => format!("refused {error:?}"),
                }
            }
            Op::Rollback(_) | Op::RollbackToTxg(_) => {
                let image = MemoryPool {
                    devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
                    device_size_in_bytes: width.device_bytes(),
                };
                let Some(session) = session.as_mut() else { return "na".into() };
                let PoolVersion::WithFile(current) = &mut session.current else { return "na".into() };
                let roots = ring_roots_newest_first(&image);
                if roots.is_empty() {
                    return "na".into();
                }
                let (txg, instance) = match op {
                    Op::Rollback(index) => roots[(index % roots.len() as u64) as usize],
                    Op::RollbackToTxg(wanted) => match roots.iter().find(|(txg, _)| *txg == wanted) {
                        Some(found) => *found,
                        None => return "na".into(),
                    },
                    _ => unreachable!(),
                };
                let target = RollbackTarget {
                    instance: InstanceGeneration(instance),
                    checkpoint_txg: CheckpointTxg(txg),
                };
                match roll_back_by_a_forward_publish(params, devices, &mut session.allocator, current, target) {
                    Ok(_) => format!("ok to txg {txg}"),
                    Err(error) => format!("refused {error:?}"),
                }
            }
            Op::RaiseFloor(steps) => {
                let Some(session) = session.as_mut() else { return "na".into() };
                let PoolVersion::WithFile(current) = &mut session.current else { return "na".into() };
                let floor = current.root.rollback_floor.0;
                let txg = current.root.checkpoint_txg.0;
                let new_floor = CheckpointTxg(floor + steps % (txg.saturating_sub(floor) + 3));
                match raise_rollback_floor(params, devices, &mut session.allocator, current, new_floor, ShadowLedger::On) {
                    Ok(raised) => format!("ok F={} publishes={}", new_floor.0, raised.publishes.len()),
                    Err(error) => format!("refused {error:?}"),
                }
            }
            Op::Unmount => {
                let Some(mut taken) = session.take() else { return "na".into() };
                match unmount(params, devices, &mut taken.allocator, &mut taken.current, ShadowLedger::On) {
                    Ok(_) => "ok".into(),
                    Err(error) => format!("refused {error:?}"),
                }
            }
        }
    }
}
