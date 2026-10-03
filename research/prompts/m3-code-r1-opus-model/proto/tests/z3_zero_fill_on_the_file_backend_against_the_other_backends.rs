//! m3-code-r1 攻方腿 Z3 ① ③：文件后端写零（打洞）与 O_DIRECT 后端、内存稀疏盘在同一组输入上的结局；真 errno（seccomp 让内核交回）下的分路。
//! 镜像放 M3R1_IMAGE_DIRECTORY（默认草稿目录，ext4）。

use std::path::PathBuf;

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DirectInputOutputBlockDevice, FileBackedBlockDevice, PageCachePolicy,
    PhysicalBlockSizeInBytes, PhysicalBlockSizeSource, WriteDurability,
};
use singlefs_harness::memory_pool::SparseBlockDevice;

const BYTES: u64 = 1 << 20;

fn directory() -> PathBuf {
    PathBuf::from(std::env::var("M3R1_IMAGE_DIRECTORY").unwrap_or_else(|_| "/tmp/claude-1000/m3-code-r1-opus/images".to_string()))
}

fn image(name: &str) -> PathBuf {
    let directory = directory();
    std::fs::create_dir_all(&directory).expect("建目录");
    let path = directory.join(format!("{name}-{}.img", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn describe(outcome: &Result<(), BlockDeviceError>) -> String {
    match outcome {
        Ok(()) => "Ok".to_string(),
        Err(BlockDeviceError::InputOutput(error)) => format!("InputOutput(raw_os_error={:?}, kind={:?})", error.raw_os_error(), error.kind()),
        Err(other) => format!("{other:?}"),
    }
}

/// 同一组 (偏移, 长度) 在三个后端上的结局。
#[test]
fn zero_fills_on_the_three_backends() {
    let requests: [(u64, u64); 6] = [(0, 0), (BYTES, 0), (4096, 0), (0, 512), (BYTES - 512, 512), (BYTES, 512)];
    for (offset, length) in requests {
        let path = image("file-backend");
        let mut file = FileBackedBlockDevice::create_image_file_exclusively(&path, BYTES, PhysicalBlockSizeInBytes(512)).expect("建镜像");
        let file_outcome = file.write_zeroes_at(DeviceOffsetInBytes(offset), length);
        let counts = file.zero_fill_landing_counts();
        drop(file);
        std::fs::remove_file(&path).expect("清理");
        let direct_path = image("direct-backend");
        std::fs::File::create(&direct_path).expect("建").set_len(BYTES).expect("撑长");
        let mut direct = DirectInputOutputBlockDevice::open_existing(&direct_path, PageCachePolicy::GoThroughPageCache, PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(512))).expect("开");
        let direct_outcome = direct.write_zeroes_at(DeviceOffsetInBytes(offset), length);
        drop(direct);
        std::fs::remove_file(&direct_path).expect("清理");
        let mut sparse = SparseBlockDevice::new(BYTES, PhysicalBlockSizeInBytes(512));
        let sparse_outcome = sparse.write_zeroes_at(DeviceOffsetInBytes(offset), length);
        println!(
            "ZERO_FILL offset={offset} length={length} file={} file_counts={counts:?} direct={} sparse={}",
            describe(&file_outcome),
            describe(&direct_outcome),
            describe(&sparse_outcome)
        );
    }
}

/// 内核真交回 errno（seccomp 让 fallocate 交回它）时文件后端走哪条路：每个 errno 一个新线程（过滤器装上卸不掉）。
#[test]
fn real_errnos_from_fallocate() {
    for name in ["EOPNOTSUPP", "ENOSYS", "ENOSPC", "EINTR", "EPERM", "EIO", "EINVAL"] {
        let path = image(&format!("errno-{name}"));
        let report = std::thread::spawn({
            let path = path.clone();
            move || {
                let mut device = FileBackedBlockDevice::create_image_file_exclusively(&path, BYTES, PhysicalBlockSizeInBytes(512)).expect("建镜像");
                device.write_at(DeviceOffsetInBytes(0), &vec![0xa5u8; 65536], WriteDurability::Plain).expect("填脏");
                m3r1_opus_proto::make_fallocate_fail_with(m3r1_opus_proto::errno_named(name));
                let outcome = device.write_zeroes_at(DeviceOffsetInBytes(4096), 16384);
                let mut back = vec![0u8; 16384];
                device.read_at(DeviceOffsetInBytes(4096), &mut back).expect("读回");
                let zeroed = back.iter().all(|byte| *byte == 0);
                let untouched = back.iter().all(|byte| *byte == 0xa5);
                format!("ERRNO {name} outcome={} counts={:?} range_zeroed={zeroed} range_untouched={untouched}", describe(&outcome), device.zero_fill_landing_counts())
            }
        })
        .join()
        .expect("线程");
        println!("{report}");
        std::fs::remove_file(&path).expect("清理");
    }
}
