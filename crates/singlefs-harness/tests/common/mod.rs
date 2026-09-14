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
use singlefs_core::superblock::FormatTimeGeometry;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
    WarmUpOutput,
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
        geometry: FormatTimeGeometry {
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

impl BuiltPool {
    /// 整条录制流带内容。
    pub fn retained_operations(&self) -> Vec<RetainedOperation> {
        self.stream.retained_operations()
    }
    /// 把整条流施加到内存镜像上：与文件镜像同一份字节。
    pub fn memory_pool(&self) -> MemoryPool {
        let mut pool =
            MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        pool.apply(&self.retained_operations());
        pool
    }
    /// 只施加 mkfs 那 13 步：层 0 枚举的基线。
    pub fn memory_pool_after_mkfs(&self) -> MemoryPool {
        let mut pool =
            MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        pool.apply(&self.retained_operations()[..self.mkfs_operation_count]);
        pool
    }
    /// 冷启动：丢掉进程内的设备句柄，按路径重新打开镜像。
    pub fn reopen_cold(&mut self) -> Vec<(DeviceIdentity, FileBackedBlockDevice)> {
        drop(self.devices.take());
        self.paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                let device = FileBackedBlockDevice::open_or_create(
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
}

pub fn build_pool(tag: &str) -> BuiltPool {
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let mut paths = Vec::new();
    let mut devices: Vec<(DeviceIdentity, Recorded)> = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path(tag, device_number);
        let file = FileBackedBlockDevice::open_or_create(
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
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(&[
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    ]);
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
