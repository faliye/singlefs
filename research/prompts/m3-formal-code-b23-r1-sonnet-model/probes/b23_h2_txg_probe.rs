//! 正推腿探针（第 23 批代码轮第一轮 K3，H2）：只暖机挂载 k 次之后写的第一个文件版本的 txg，k = 1..=22；
//! 与 H2 那两条用例的常数注释「每次只暖机的挂载发 3 版，第 k 次挂载上的第一个文件版本 txg 3k − 1」对。
mod common;

use common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

#[test]
fn first_file_txg_after_k_warm_up_only_mounts() {
    let s = ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;
    let mut line = format!("S={s}: ");
    for k in [1u64, 2, 3, 5, 8, 9, 10, 11, 12] {
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<_> = (0..2u32)
            .map(|n| {
                (
                    DeviceIdentity(n),
                    RecordingBlockDevice::with_shared_stream(
                        DeviceIdentity(n),
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        make_filesystem(&parameters(), &mut devices).expect("mkfs");
        let mut last = mount_writable(&parameters(), &mut devices).expect("挂载");
        for _ in 1..k {
            last = mount_writable(&parameters(), &mut devices).expect("挂载");
        }
        let PoolVersion::WithoutFile(version) = &last.current else { panic!("树表 0 条") };
        let mut allocator = last.allocator.clone();
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile { content: &file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            last.output.instance,
            &version.record_bytes,
        )
        .expect("第一个文件");
        let txg = first_file.root.checkpoint_txg.0;
        println!("{}", format!("k={k}:txg={txg}(3k-1={}){} ", 3 * k - 1, if txg == 3 * k - 1 { "" } else { "!!" }));
    }
    println!("{line}");
}
