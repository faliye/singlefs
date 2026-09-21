//! 步 6 / 步 7 验收共用的搭建：两个文件镜像上 mkfs → 取号 → 暖机 → 第一个事务，录制流开内容保留。
#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{FileBackedBlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemOutput, MakeFilesystemParameters, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    PublishError, TransactionOutput, WarmUpOutput,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);
pub const IMAGE_BYTES: u64 = 4 << 30;
/// E142 的第一个文件 3000 字节（`name=config file_bytes=3000`）。
pub const FILE_BYTES: usize = 3000;
/// E142 装置里的固定 fsid（`FIXED_FSID`）。
pub const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// E142 装置里的固定写入时间（`FIXED_WRITE_TIME_SECONDS`）。
pub const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

pub type Recorded = RecordingBlockDevice<FileBackedBlockDevice>;

pub fn image_path(tag: &str, device: u32) -> PathBuf {
    let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "singlefs-step67-{tag}-{}-{sequence}-dev{device}.img",
        std::process::id()
    ))
}

pub fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
    }
}

pub fn geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
    }
}

pub fn file_content() -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

pub struct BuiltPool {
    pub paths: Vec<PathBuf>,
    /// `take` 出去就是「进程退出、镜像关掉」：冷启动要重新打开文件。
    pub devices: Option<Vec<(DeviceIdentity, Recorded)>>,
    pub stream: SharedStream,
    pub genesis: MakeFilesystemOutput,
    pub warm_up: WarmUpOutput,
    pub output: TransactionOutput,
    pub allocator: PoolAllocator,
    pub mkfs_operation_count: usize,
}

impl Drop for BuiltPool {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// 把一段录制流施加到两块空内存盘上：与文件镜像同一份字节。
fn memory_pool_of(operations: &[RetainedOperation]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    pool.apply(operations);
    pool
}

/// 按路径重开镜像，写照旧录进同一条流。
fn reopen_recorded_images(
    paths: &[PathBuf],
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, Recorded)> {
    paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let file = FileBackedBlockDevice::open_existing_image_file(
                path,
                IMAGE_BYTES,
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

/// 按路径重开镜像，不录。
fn reopen_cold_images(paths: &[PathBuf]) -> Vec<(DeviceIdentity, FileBackedBlockDevice)> {
    paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let device = FileBackedBlockDevice::open_existing_image_file(
                path,
                IMAGE_BYTES,
                PhysicalBlockSizeInBytes(512),
            )
            .expect("重开镜像");
            (
                DeviceIdentity(u32::try_from(index).expect("设备号")),
                device,
            )
        })
        .collect()
}

/// 两个新建的全零文件镜像上 mkfs，录制流开内容保留：返回镜像路径、录着的设备、流、mkfs 写出的东西与 mkfs 占了流里几步。
fn formatted_recorded_images(
    tag: &str,
) -> (
    Vec<PathBuf>,
    Vec<(DeviceIdentity, Recorded)>,
    SharedStream,
    MakeFilesystemOutput,
    usize,
) {
    let stream = SharedStream::retaining_contents();
    let mut paths = Vec::new();
    let mut devices: Vec<(DeviceIdentity, Recorded)> = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path(tag, device_number);
        let file = FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            IMAGE_BYTES,
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
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
    (paths, devices, stream, genesis, mkfs_operation_count)
}

impl BuiltPool {
    /// 整条录制流带内容。
    pub fn retained_operations(&self) -> Vec<RetainedOperation> {
        self.stream.retained_operations()
    }
    /// 把整条流施加到内存镜像上：与文件镜像同一份字节。
    pub fn memory_pool(&self) -> MemoryPool {
        memory_pool_of(&self.retained_operations())
    }
    /// 只施加 mkfs 那 13 步：层 0 枚举的基线。
    pub fn memory_pool_after_mkfs(&self) -> MemoryPool {
        memory_pool_of(&self.retained_operations()[..self.mkfs_operation_count])
    }
    /// 重开镜像并继续录进同一条流：进程重开之后的可写挂载与发布都要进层 0 的整条流（里程碑「第二个事务」步 0 / 步 3）。
    pub fn reopen_recorded(&mut self) -> Vec<(DeviceIdentity, Recorded)> {
        drop(self.devices.take());
        reopen_recorded_images(&self.paths, &self.stream)
    }

    /// 冷启动：丢掉进程内的设备句柄，按路径重新打开镜像。
    pub fn reopen_cold(&mut self) -> Vec<(DeviceIdentity, FileBackedBlockDevice)> {
        drop(self.devices.take());
        reopen_cold_images(&self.paths)
    }
}

/// 只做过 mkfs 的池：没取过号、一个文件都没发布（里程碑「第二个事务」步 3：只做过 mkfs 的池也允许可写挂载，2026-09-17 用户定）。
pub struct FormattedPool {
    pub paths: Vec<PathBuf>,
    /// `take` 出去就是「进程退出、镜像关掉」。
    pub devices: Option<Vec<(DeviceIdentity, Recorded)>>,
    pub stream: SharedStream,
    pub genesis: MakeFilesystemOutput,
    pub mkfs_operation_count: usize,
}

impl Drop for FormattedPool {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl FormattedPool {
    pub fn retained_operations(&self) -> Vec<RetainedOperation> {
        self.stream.retained_operations()
    }
    pub fn memory_pool(&self) -> MemoryPool {
        memory_pool_of(&self.retained_operations())
    }
    pub fn memory_pool_after_mkfs(&self) -> MemoryPool {
        memory_pool_of(&self.retained_operations()[..self.mkfs_operation_count])
    }
    pub fn reopen_recorded(&mut self) -> Vec<(DeviceIdentity, Recorded)> {
        drop(self.devices.take());
        reopen_recorded_images(&self.paths, &self.stream)
    }
    pub fn reopen_cold(&mut self) -> Vec<(DeviceIdentity, FileBackedBlockDevice)> {
        drop(self.devices.take());
        reopen_cold_images(&self.paths)
    }
}

/// 一个崩溃状态的两块内存盘：mkfs 之后的基线再施加持久了的那几条写，外面包录制器、录进一条新流（挂载之后流里多一步就是发了写或屏障）。
pub fn crash_state_devices(
    base: &MemoryPool,
    writes: &[singlefs_harness::crash::RetainedWrite],
    persisted: &[bool],
    stream: &SharedStream,
) -> Vec<(
    DeviceIdentity,
    RecordingBlockDevice<singlefs_harness::crash::SparseBlockDevice>,
)> {
    base.devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = singlefs_harness::crash::SparseBlockDevice::new(
                IMAGE_BYTES,
                PhysicalBlockSizeInBytes(512),
            );
            device.image = sparse.clone();
            for (write, is_persisted) in writes.iter().zip(persisted) {
                if *is_persisted && write.device == *identity {
                    device.image.write(write.offset, &write.bytes);
                }
            }
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect()
}

/// 这几块内存盘此刻的整份镜像。
pub fn memory_pool_of_sparse_devices(
    devices: &[(
        DeviceIdentity,
        RecordingBlockDevice<singlefs_harness::crash::SparseBlockDevice>,
    )],
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner().image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

/// 盘上可比的一份快照：两盘各两个系统配置槽的原样字节、根环里全部自证过的根、录制流里已有几步。挂载或抬 F 被拒之后与拒之前逐项相等，
/// 才算「在任何写之前拒绝」（录制流不多一步 = 一个写、一道屏障都没发）。
#[derive(Debug, PartialEq, Eq)]
pub struct DiskSnapshot {
    pub system_configuration_slots: Vec<Vec<u8>>,
    pub readable_roots: Vec<singlefs_core::root_record::RootRecord>,
    pub recorded_operations: usize,
}

pub fn disk_snapshot(image: &MemoryPool, stream: &SharedStream) -> DiskSnapshot {
    use singlefs_core::recovery::PoolReader;
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut system_configuration_slots = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in [0, spacing] {
            system_configuration_slots.push(
                PoolReader::read(
                    image,
                    device,
                    singlefs_core::address::DeviceOffsetInBytes(offset),
                    slot_bytes,
                )
                .expect("系统配置槽读得到"),
            );
        }
    }
    let system_configuration =
        singlefs_core::recovery::choose_system_configuration(image).expect("系统配置");
    DiskSnapshot {
        system_configuration_slots,
        readable_roots: singlefs_core::recovery::readable_roots(
            image,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
        ),
        recorded_operations: stream.operations().len(),
    }
}

/// 第一个事务之后，在同一个进程里对同一个文件覆盖写一次（发布 B 起的每一次覆盖写走这一条）：
/// 错误原样交回，要不要 `expect` 由调用方定。
pub fn publish_overwrite_in_process(
    pool: &mut BuiltPool,
    previous: &TransactionOutput,
    content: &[u8],
    write_time_seconds: u64,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        previous,
        FirstFile {
            content,
            write_time_seconds,
        },
        instance,
    )
}

pub fn format_pool(tag: &str) -> FormattedPool {
    let (paths, devices, stream, genesis, mkfs_operation_count) = formatted_recorded_images(tag);
    FormattedPool {
        paths,
        devices: Some(devices),
        stream,
        genesis,
        mkfs_operation_count,
    }
}

pub fn build_pool(tag: &str) -> BuiltPool {
    let parameters = parameters();
    let (paths, mut devices, stream, genesis, mkfs_operation_count) =
        formatted_recorded_images(tag);
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
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
    let content = file_content();
    let (warm_up, output) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        let warm_up = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let output = publish_first_file(
            &mut pool,
            &mut allocator,
            &genesis.root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up.last_record_bytes,
        )
        .expect("第一个事务");
        (warm_up, output)
    };
    BuiltPool {
        paths,
        devices: Some(devices),
        stream,
        genesis,
        warm_up,
        output,
        allocator,
        mkfs_operation_count,
    }
}
