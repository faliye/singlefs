//! mkfs 对外入口取 fsid 的随机源读不出的两支（D22（单元原子性怎么合成） 已定项 8 第 3 条「fsid 由 mkfs 的对外入口生成，前 8 字节取自随机源」；
//! m3-close-w13-code-r1 判决第二节 K2「随机源打不开、读不满都在任何写之前拒，不退回别的来源」与第三节改法表「K2 失败分支没有用例」）：
//! 打不开、读不满 16 字节，各一条用例。对外入口经 `make_filesystem_drawing_the_filesystem_identifier_from_the_random_source_at` 换随机源，
//! 两支都要交回 `MakeFilesystemError::RandomSourceUnreadable`，两块盘上一个操作都没有（录制流 0 步、稀疏盘上没有写过的扇区）。
//! 判别力：`crates/mutations.tsv` 里以「根环一簇 R5」开头的两行（读不出时退回默认随机源、退回一个写死的 fsid），各让这里点名的用例红。

use std::path::PathBuf;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem_drawing_the_filesystem_identifier_from_the_random_source_at,
    MakeFilesystemError,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::temporary_files::TemporaryPathRemovedOnDrop;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 4 GiB 稀疏盘：环 768 MiB ≤ 容量 / 4（D23（journal 的角色与格式） 已定项 19）。
const IMAGE_BYTES: u64 = 4 << 30;
/// 读不满那一支的随机源文件有几个字节：比 fsid 的 16 字节少。
const RANDOM_SOURCE_BYTES_SHORT_OF_A_FILESYSTEM_IDENTIFIER: usize = 8;

type RecordedSparseDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 两块空的稀疏盘，写与屏障都记进同一条录制流。
fn recorded_blank_devices() -> (RecordedSparseDevices, SharedStream) {
    let stream = SharedStream::new();
    let devices = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device_number),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    (devices, stream)
}

/// mkfs 的几何：物理块 512、io_min 512、默认环长，根环区域 0 / 1 / 2 在盘 0 / 1 / 0 上。
fn geometry() -> SystemImmutableSizes {
    SystemImmutableSizes {
        physical_block_size: 512,
        minimum_input_output_bytes: 512,
        fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    }
}

/// 这一条用例自己的随机源文件路径（临时目录下，带进程号与场景名，几条用例并行不撞）。
fn random_source_path_of(scenario: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "singlefs-random-source-{scenario}-{}",
        std::process::id()
    ))
}

/// 对外入口照 `random_source_path` 取 fsid 做 mkfs：要被拒、拒的是随机源读不出，两块盘上一个操作都没有。交回拒的那个 I/O 错的种类。
fn refused_before_any_write(random_source_path: &std::path::Path) -> std::io::ErrorKind {
    let (mut devices, stream) = recorded_blank_devices();
    let refusal = match make_filesystem_drawing_the_filesystem_identifier_from_the_random_source_at(
        random_source_path,
        [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry(),
        &mut devices,
    ) {
        Ok(made) => panic!(
            "随机源读不出，对外入口却做成了池（fsid {:?}）：退回了别的来源",
            made.parameters.filesystem_identifier
        ),
        Err(refusal) => refusal,
    };
    let MakeFilesystemError::RandomSourceUnreadable(error) = &refusal else {
        panic!("随机源读不出应当报 RandomSourceUnreadable，报的是 {refusal:?}");
    };
    let error_kind = error.kind();
    assert_eq!(
        stream.operation_count(),
        0,
        "取 fsid 在任何写之前：录制流一步都没有"
    );
    error_kind
}

/// 场景：对外入口的随机源点名一个不存在的文件。
/// 预期：`RandomSourceUnreadable`（打不开，`NotFound`），两块盘上一个操作都没有，不退回默认随机源或别的来源。
#[test]
fn make_filesystem_whose_random_source_cannot_be_opened_is_refused_before_any_write() {
    let missing = random_source_path_of("missing");
    assert!(!missing.exists(), "前提：随机源文件不存在");
    assert_eq!(
        refused_before_any_write(&missing),
        std::io::ErrorKind::NotFound,
        "打不开的随机源报打不开"
    );
}

/// 场景：对外入口的随机源点名一个只有 8 个字节的文件，读不满 fsid 的 16 字节。
/// 预期：`RandomSourceUnreadable`（读不满，`UnexpectedEof`），两块盘上一个操作都没有，不拿那 8 个字节凑、也不退回别的来源。
#[test]
fn make_filesystem_whose_random_source_is_shorter_than_a_filesystem_identifier_is_refused_before_any_write(
) {
    // 守卫在写之前认领：用例做完、断言失败、panic 都删掉这份随机源文件
    let short = TemporaryPathRemovedOnDrop::claiming(random_source_path_of("short"));
    std::fs::write(
        &*short,
        [0xa5_u8; RANDOM_SOURCE_BYTES_SHORT_OF_A_FILESYSTEM_IDENTIFIER],
    )
    .expect("写一个 8 字节的随机源文件");
    let error_kind = refused_before_any_write(&short);
    assert_eq!(
        error_kind,
        std::io::ErrorKind::UnexpectedEof,
        "读不满的随机源报读不满"
    );
}
