//! 代码审阅第 10 条：稀疏盘（`SparseBlockDevice`）与内存镜像（`MemoryPool`、`CrashImage`）对设备之外的请求，要与真盘答得一样。
//!
//! 真盘那一边是文件后端 `FileBackedBlockDevice`：越界报 `OutOfRange`、没按物理块对齐报 `Unaligned`，盘上一个字节不动；
//! 一池真盘的 `PoolReader`（`[(DeviceIdentity, 块设备)]`）把这两种都答成 None，池里没有的盘也是 None
//! （`PoolReader::read` 的契约「读不到（没有那块盘、越界）返回 None」，checker 的 `ImageReader::read` 同一句）。
//! 稀疏盘照收越界写、内存镜像越界读给全 0 的时候，实现把一个单元落到盘尾之外，随机历史与冷启动读回照样绿，真盘上却是 I/O 错。
//!
//! 两条用例都拿真盘逐个请求对拍，不各自写期望：同一组请求，真盘怎么答，内存这一边就得怎么答。

use std::path::PathBuf;

use singlefs_checker::image::ImageReader;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, FileBackedBlockDevice, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::recovery::PoolReader;
use singlefs_harness::crash::{CrashImage, MemoryPool, SparseBlockDevice, SECTOR_BYTES};

/// 两边的盘都是 1 MiB、物理块 512 字节。
const DEVICE_BYTES: u64 = 1 << 20;
const PHYSICAL_BLOCK_BYTES: u32 = 512;

/// 对拍的请求：两个在设备里的、三个越过末尾的（跨过末尾、从末尾起、远在末尾之后）、一个偏移加长度溢出 u64 的、两个没对齐的。
fn requests() -> Vec<(DeviceOffsetInBytes, u64)> {
    vec![
        (DeviceOffsetInBytes(0), 512),
        (DeviceOffsetInBytes(DEVICE_BYTES - 512), 512),
        (DeviceOffsetInBytes(DEVICE_BYTES - 512), 1024),
        (DeviceOffsetInBytes(DEVICE_BYTES), 512),
        (DeviceOffsetInBytes(4 * DEVICE_BYTES), 4096),
        (DeviceOffsetInBytes(u64::MAX - 511), 1024),
        (DeviceOffsetInBytes(256), 512),
        (DeviceOffsetInBytes(0), 700),
    ]
}

/// 一次请求的结局，按调用方要做的决定分（`BlockDeviceError` 里带着 `io::Error`，不能直接比）。
#[derive(Debug, PartialEq, Eq)]
enum RequestVerdict {
    Accepted,
    OutOfRange {
        offset: DeviceOffsetInBytes,
        length: u64,
        device_size: u64,
    },
    Unaligned {
        offset: DeviceOffsetInBytes,
        length: u64,
        physical_block_size: u32,
    },
    OtherError(String),
}

fn verdict_of(result: Result<(), BlockDeviceError>) -> RequestVerdict {
    match result {
        Ok(()) => RequestVerdict::Accepted,
        Err(BlockDeviceError::OutOfRange {
            offset,
            length,
            device_size,
        }) => RequestVerdict::OutOfRange {
            offset,
            length,
            device_size,
        },
        Err(BlockDeviceError::Unaligned {
            offset,
            length,
            physical_block_size,
        }) => RequestVerdict::Unaligned {
            offset,
            length,
            physical_block_size,
        },
        Err(
            error @ (BlockDeviceError::InputOutput(_)
            | BlockDeviceError::ProbeFailed { .. }
            | BlockDeviceError::ImageFileAlreadyExists { .. }
            | BlockDeviceError::ImageFileMissing { .. }),
        ) => RequestVerdict::OtherError(error.to_string()),
    }
}

/// 用例建的镜像文件：成功、失败、panic 都删掉。
struct ImageFileRemovedOnDrop(PathBuf);

impl Drop for ImageFileRemovedOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn file_backed_device(
    scenario: &str,
    device: u32,
) -> (ImageFileRemovedOnDrop, FileBackedBlockDevice) {
    let path = std::env::temp_dir().join(format!(
        "singlefs-sparse-contract-{scenario}-{}-dev{device}.img",
        std::process::id()
    ));
    let removed_on_drop = ImageFileRemovedOnDrop(path.clone());
    let file_device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        DEVICE_BYTES,
        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
    )
    .expect("建镜像文件");
    (removed_on_drop, file_device)
}

fn request_length(length: u64) -> usize {
    usize::try_from(length).expect("用例里的请求长度装得进 usize")
}

/// 读、写、清零三种请求，稀疏盘逐个与文件后端答得一样：同一个成员、同样的字段；被拒的写与清零在稀疏盘上一个扇区都不落。
#[test]
fn a_sparse_block_device_accepts_and_refuses_every_request_exactly_like_the_file_backend() {
    let (_removed_on_drop, mut file_device) = file_backed_device("block-device", 0);
    let mut sparse_device =
        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES));
    let mut refused_writes = 0u32;
    for (offset, length) in requests() {
        let mut file_buffer = vec![0u8; request_length(length)];
        let mut sparse_buffer = vec![0u8; request_length(length)];
        assert_eq!(
            verdict_of(sparse_device.read_at(offset, &mut sparse_buffer)),
            verdict_of(file_device.read_at(offset, &mut file_buffer)),
            "读：偏移 {} 长度 {length}",
            offset.0
        );

        let bytes = vec![0xA5u8; request_length(length)];
        let image_before_the_write = sparse_device.image.clone();
        let sparse_write =
            verdict_of(sparse_device.write_at(offset, &bytes, WriteDurability::Plain));
        assert_eq!(
            sparse_write,
            verdict_of(file_device.write_at(offset, &bytes, WriteDurability::Plain)),
            "写：偏移 {} 长度 {length}",
            offset.0
        );
        if sparse_write != RequestVerdict::Accepted {
            refused_writes += 1;
            assert_eq!(
                sparse_device.image, image_before_the_write,
                "被拒的写在稀疏盘上一个扇区都不落：偏移 {} 长度 {length}",
                offset.0
            );
        }

        let image_before_the_zero_fill = sparse_device.image.clone();
        let sparse_zero_fill = verdict_of(sparse_device.write_zeroes_at(offset, length));
        assert_eq!(
            sparse_zero_fill,
            verdict_of(file_device.write_zeroes_at(offset, length)),
            "清零：偏移 {} 长度 {length}",
            offset.0
        );
        if sparse_zero_fill != RequestVerdict::Accepted {
            assert_eq!(
                sparse_device.image, image_before_the_zero_fill,
                "被拒的清零在稀疏盘上一个扇区都不动：偏移 {} 长度 {length}",
                offset.0
            );
        }
    }
    assert_eq!(
        refused_writes, 6,
        "8 个请求里 4 个越界、2 个没对齐：两边都拒（两边都照收的话这条对拍就空了）"
    );
}

/// 内存镜像（`MemoryPool`）与叠在它上面的崩溃镜像（`CrashImage`）两个读口子（恢复的 `PoolReader`、checker 的 `ImageReader`），
/// 对每块盘（池里的两块与池里没有的第 7 块）× 每个请求，读出来的与一池真盘的 `PoolReader` 逐个相同：越界、没对齐、没有这块盘都是 None；
/// 盘的字节数也一样（没有这块盘是 None）。
#[test]
fn memory_pool_and_crash_image_readers_answer_none_outside_a_device_like_a_pool_of_real_devices() {
    let (_removed_zero, file_device_zero) = file_backed_device("pool-reader", 0);
    let (_removed_one, file_device_one) = file_backed_device("pool-reader", 1);
    let mut real_pool = vec![
        (DeviceIdentity(0), file_device_zero),
        (DeviceIdentity(1), file_device_one),
    ];
    let mut memory_pool =
        MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], DEVICE_BYTES);
    // 两边在盘 1 的最后一个物理块上写同样的字节：设备里的读要比出真内容，不是两边都全 0。
    let last_block = DeviceOffsetInBytes(DEVICE_BYTES - SECTOR_BYTES);
    let last_block_bytes = vec![0x3Cu8; request_length(SECTOR_BYTES)];
    real_pool[1]
        .1
        .write_at(last_block, &last_block_bytes, WriteDurability::Plain)
        .expect("真盘写最后一个物理块");
    memory_pool
        .devices
        .get_mut(&DeviceIdentity(1))
        .expect("内存池里有盘 1")
        .write(last_block, &last_block_bytes);
    let crash_image = CrashImage {
        base: &memory_pool,
        writes: &[],
        persisted: Vec::new(),
    };

    let mut unreadable_requests = 0u32;
    let mut readable_requests = 0u32;
    for device in [0u32, 1, 7] {
        for (offset, length) in requests() {
            let expected = PoolReader::read(
                real_pool.as_slice(),
                DeviceIdentity(device),
                offset,
                request_length(length),
            );
            match &expected {
                Some(_) => readable_requests += 1,
                None => unreadable_requests += 1,
            }
            let request = format!("盘 {device} 偏移 {} 长度 {length}", offset.0);
            assert_eq!(
                PoolReader::read(
                    &memory_pool,
                    DeviceIdentity(device),
                    offset,
                    request_length(length)
                ),
                expected,
                "MemoryPool 的 PoolReader：{request}"
            );
            assert_eq!(
                PoolReader::read(
                    &crash_image,
                    DeviceIdentity(device),
                    offset,
                    request_length(length)
                ),
                expected,
                "CrashImage 的 PoolReader：{request}"
            );
            assert_eq!(
                ImageReader::read(&memory_pool, device, offset.0, request_length(length)),
                expected,
                "MemoryPool 的 ImageReader：{request}"
            );
            assert_eq!(
                ImageReader::read(&crash_image, device, offset.0, request_length(length)),
                expected,
                "CrashImage 的 ImageReader：{request}"
            );
        }
        let expected_bytes =
            PoolReader::device_size_in_bytes(real_pool.as_slice(), DeviceIdentity(device));
        assert_eq!(
            PoolReader::device_size_in_bytes(&memory_pool, DeviceIdentity(device)),
            expected_bytes,
            "MemoryPool 的 PoolReader 报盘 {device} 的字节数"
        );
        assert_eq!(
            ImageReader::device_bytes(&memory_pool, device),
            expected_bytes,
            "MemoryPool 的 ImageReader 报盘 {device} 的字节数"
        );
        assert_eq!(
            ImageReader::device_bytes(&crash_image, device),
            expected_bytes,
            "CrashImage 的 ImageReader 报盘 {device} 的字节数"
        );
    }
    assert_eq!(
        (readable_requests, unreadable_requests),
        (4, 20),
        "真盘那一边：两块盘各 2 个读得出；两块盘各 6 个越界或没对齐、第 7 块盘 8 个全读不出"
    );
}
