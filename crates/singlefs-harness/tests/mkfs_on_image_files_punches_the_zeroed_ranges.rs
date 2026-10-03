//! 文件后端上 mkfs 一次：整段清零落成打洞，经 write 系统调用交出去的只有 mkfs 真写的那些字节，
//! 两块盘上根环三段与 journal 环（每块盘约 768 MiB）一个 0 都没真写。
//!
//! 来历：C603（测试与验证的资源浪费没有自检）与 C486（整环清零的 55 秒与冷重开的 17 秒都没人管） 里「写零真写」那一半。
//! 调查量到一池文件后端测试写出 1 610 991 104 字节，其中非零 6 560 字节，页缓存峰值 1.5 GiB；改法是「稀疏打洞」。
//! 量法：本线程的 `/proc/thread-self/io` 里 `wchar`（经 write 一族系统调用交出去的字节数）在 mkfs 前后之差；
//! 取线程这一份、不取进程那一份，因为同一个测试二进制里别的用例在别的线程上并行写各自的镜像。
//! 镜像放 `std::env::temp_dir()`，那个目录所在的文件系统要支持打洞（ext4、xfs、btrfs、tmpfs 都支持）；报不支持打洞时写零退回真写、这条用例判红，
//! 报别的错时 mkfs 照常交回块设备 I/O 错（只在不支持打洞时退回真写）。
//! 两份镜像（各 4 GiB 稀疏、清零之前已写的那部分实占）由删除守卫删：用例做完、断言失败、panic 都删
//! （`.claude/singlefs-ai-sop/rules/command-safety.md`「测试镜像一律放临时目录」；代码三方 m3-code-r1 Z3-d：只在末尾删时红一次留一份）。

mod common;

use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use common::{image_path, parameters, IMAGE_BYTES};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::{
    FileBackedBlockDevice, PhysicalBlockSizeInBytes, ZeroFillLandingCounts,
};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

/// 本线程经 write 一族系统调用交出去的字节数（`/proc/thread-self/io` 的 `wchar`）。
fn bytes_handed_to_write_calls_by_this_thread() -> u64 {
    let accounting = std::fs::read_to_string("/proc/thread-self/io")
        .expect("读 /proc/thread-self/io（Linux 3.17 起就有）");
    accounting
        .lines()
        .find_map(|line| line.strip_prefix("wchar: "))
        .expect("io 统计里有 wchar 一行")
        .trim()
        .parse()
        .expect("wchar 是十进制数")
}

/// 这条用例建出的镜像：守卫离开作用域时逐个删掉，不管用例是做完、断言失败还是 panic 着离开的。
/// 建成一份记一份（建镜像那一步失败的那一份不记：同名文件不是这一次建的）。删不掉只打一行，不在 Drop 里 panic
/// （panic 途中再 panic 会直接中止进程，后面那几份就删不到了）。
struct ImageFilesRemovedOnDrop {
    paths: Vec<PathBuf>,
}

impl Drop for ImageFilesRemovedOnDrop {
    fn drop(&mut self) {
        for path in &self.paths {
            if let Err(error) = std::fs::remove_file(path) {
                eprintln!("删不掉测试镜像 {}：{error}", path.display());
            }
        }
    }
}

#[test]
fn mkfs_on_two_image_files_hands_only_the_written_bytes_to_write_calls_and_punches_every_zeroed_range(
) {
    let stream = SharedStream::new();
    let mut images = ImageFilesRemovedOnDrop { paths: Vec::new() };
    let mut devices = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path("punch-zeroed-ranges", device_number);
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
        images.paths.push(path);
    }

    let handed_before = bytes_handed_to_write_calls_by_this_thread();
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let handed_during_mkfs = bytes_handed_to_write_calls_by_this_thread() - handed_before;

    let operations = stream.operations();
    let zeroed_bytes: u64 = operations
        .iter()
        .filter(|operation| operation.kind == RecordedOperationKind::WriteZeroes)
        .map(|operation| operation.length)
        .sum();
    let zero_fill_calls = operations
        .iter()
        .filter(|operation| operation.kind == RecordedOperationKind::WriteZeroes)
        .count();
    let written_bytes: u64 = operations
        .iter()
        .filter(|operation| {
            matches!(
                operation.kind,
                RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess
            )
        })
        .map(|operation| operation.length)
        .sum();

    assert!(
        zeroed_bytes >= 2 * JOURNAL_RING_DEFAULT_BYTES,
        "mkfs 该清两块盘的 journal 环，录制层只记到 {zeroed_bytes} 字节的写零"
    );
    assert_eq!(
        handed_during_mkfs, written_bytes,
        "经 write 交出去的该恰好是 mkfs 真写的 {written_bytes} 字节；多出来的是写零真写了 0"
    );
    assert!(
        handed_during_mkfs.saturating_mul(1000) < zeroed_bytes,
        "经 write 交出去 {handed_during_mkfs} 字节，该远小于清零的 {zeroed_bytes} 字节"
    );
    let punched_calls: u64 = devices
        .iter()
        .map(|(_, device)| {
            let counts: ZeroFillLandingCounts = device.wrapped_device().zero_fill_landing_counts();
            assert_eq!(
                counts.written_as_zero_bytes_calls, 0,
                "没有一次写零退回逐块写 0"
            );
            counts.punched_calls
        })
        .sum();
    assert_eq!(
        punched_calls,
        u64::try_from(zero_fill_calls).expect("步数装得进 u64"),
        "录制层记的每一步写零都落成了打洞"
    );
    for path in &images.paths {
        let metadata = std::fs::metadata(path).expect("读镜像元数据");
        assert_eq!(metadata.len(), IMAGE_BYTES, "打洞不改镜像长度");
        let allocated_bytes = metadata.blocks() * 512;
        assert!(
            allocated_bytes.saturating_mul(1000) < zeroed_bytes / 2,
            "镜像 {} 在盘上占了 {allocated_bytes} 字节，该远小于这块盘清零的长度",
            path.display()
        );
    }
}
