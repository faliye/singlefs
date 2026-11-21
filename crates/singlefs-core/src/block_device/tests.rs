use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 测试镜像一律放临时目录（`command-safety.md`），文件名带进程号与计数器免得并行测试互相盖。
/// 交回的是认领着这条路径的守卫：建镜像之前就认领，用例做完、断言失败、panic 都删掉它。
fn temporary_image_path(tag: &str) -> TemporaryPathRemovedOnDrop {
    let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    TemporaryPathRemovedOnDrop(std::env::temp_dir().join(format!(
        "singlefs-core-{tag}-{}-{sequence}.img",
        std::process::id()
    )))
}

/// 临时路径的删除守卫：用例做完、断言失败、panic 都在离开作用域时删掉它（文件删文件，目录连同里面的一起删，不在就算了；
/// `command-safety.md`「测试镜像一律放临时目录」）。核心层不依赖验证包，这里留一份与 `singlefs_verification::temporary_files`
/// 同名同删法的。删不掉只打一行，不在 Drop 里 panic（panic 途中再 panic 会直接中止进程）。
struct TemporaryPathRemovedOnDrop(PathBuf);

impl std::ops::Deref for TemporaryPathRemovedOnDrop {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for TemporaryPathRemovedOnDrop {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryPathRemovedOnDrop {
    fn drop(&mut self) {
        let removed = match std::fs::symlink_metadata(&self.0) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return,
            Err(error) => Err(error),
            Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(&self.0),
            Ok(_not_a_directory) => std::fs::remove_file(&self.0),
        };
        if let Err(error) = removed {
            if error.kind() != io::ErrorKind::NotFound {
                eprintln!("删不掉临时路径 {}：{error}", self.0.display());
            }
        }
    }
}

/// 场景：守卫认领一份镜像与一个带文件的目录，作用域照常走完。预期：两条路径都不在了。
/// 判别力（`crates/mutations.tsv`）：守卫的 Drop 不删，两条都还在，这条红。
#[test]
fn temporary_image_and_directory_are_gone_once_the_scope_ends_normally() {
    let image_path;
    let directory_path;
    {
        let image = temporary_image_path("guard-normal-end");
        FileBackedBlockDevice::create_image_file_exclusively(
            &image,
            4096,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像");
        let directory = TemporaryPathRemovedOnDrop(std::env::temp_dir().join(format!(
            "singlefs-core-guard-directory-{}-{}",
            std::process::id(),
            IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst)
        )));
        std::fs::create_dir_all(directory.join("nested")).expect("建目录");
        std::fs::write(directory.join("nested").join("attribute"), "1\n").expect("写文件");
        image_path = image.to_path_buf();
        directory_path = directory.to_path_buf();
        assert!(
            image_path.exists() && directory_path.exists(),
            "作用域里两条路径都在"
        );
    }
    assert!(
        !image_path.exists(),
        "作用域走完镜像 {} 不在了",
        image_path.display()
    );
    assert!(
        !directory_path.exists(),
        "作用域走完目录 {} 连同里面的不在了",
        directory_path.display()
    );
}

/// 场景：守卫认领的镜像建好之后作用域里 panic。预期：panic 接住之后镜像不在了。
/// 判别力（`crates/mutations.tsv`）：守卫的 Drop 不删，镜像还在，这条红。
#[test]
fn temporary_image_is_gone_after_a_panic_inside_the_scope() {
    let image = temporary_image_path("guard-panic");
    let image_path = image.to_path_buf();
    let panicked = std::panic::catch_unwind(move || {
        FileBackedBlockDevice::create_image_file_exclusively(
            &image,
            4096,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像");
        assert!(image.exists(), "panic 之前镜像在");
        panic!("守卫作用域里造的 panic");
    });
    assert!(panicked.is_err(), "作用域里真 panic 了");
    assert!(
        !image_path.exists(),
        "panic 之后镜像 {} 不在了",
        image_path.display()
    );
}

#[test]
fn write_then_read_returns_the_same_bytes_and_zero_fill_elsewhere() {
    let path = temporary_image_path("roundtrip");
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        16384,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像");
    let payload = vec![0xA5u8; 1024];
    device
        .write_at(
            DeviceOffsetInBytes(4096),
            &payload,
            WriteDurability::ForceUnitAccess,
        )
        .expect("写");
    let mut read_back = vec![0u8; 1024];
    device
        .read_at(DeviceOffsetInBytes(4096), &mut read_back)
        .expect("读");
    assert_eq!(read_back, payload);
    let mut untouched = vec![0xFFu8; 512];
    device
        .read_at(DeviceOffsetInBytes(0), &mut untouched)
        .expect("读没写过的地方");
    assert!(
        untouched.iter().all(|byte| *byte == 0),
        "稀疏镜像没写过的地方读回全 0"
    );
}

/// 写零把整段变成 0、段外一个字节不动；长度大于一个块时底下拆成几次写，拆的边界上不许留下没清的洞。
/// 段长取 `ZERO_FILL_CHUNK_BYTES` 的 2.5 倍：两个整块加一个半块，最后一块要按剩余长度截短。
#[test]
fn writing_zeroes_clears_the_whole_range_across_chunk_boundaries_and_nothing_outside_it() {
    let path = temporary_image_path("zeroes");
    let chunk = ZERO_FILL_CHUNK_BYTES;
    let zero_start = chunk;
    let zero_length = chunk * 2 + chunk / 2;
    let image_bytes = zero_start + zero_length + chunk;
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        image_bytes,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像");
    // 整个镜像先填成非 0：稀疏镜像本来就读回全 0，不先弄脏就分不出「清过」与「从来没写过」
    // （`test-discipline.md`「读不到 ≠ 读到 0」）。
    let dirt = vec![0xC3u8; usize::try_from(chunk).expect("块长")];
    let mut filled: u64 = 0;
    while filled < image_bytes {
        let this_piece = usize::try_from((image_bytes - filled).min(chunk)).expect("片长");
        device
            .write_at(
                DeviceOffsetInBytes(filled),
                &dirt[..this_piece],
                WriteDurability::Plain,
            )
            .expect("填脏");
        filled += u64::try_from(this_piece).expect("片长装得进 u64");
    }
    device
        .write_zeroes_at(DeviceOffsetInBytes(zero_start), zero_length)
        .expect("写零");
    // 抽样读：段内取每一块的首尾与两个块边界的两侧，段外取紧挨着的前后各一扇区。
    let mut sample = vec![0xFFu8; 512];
    for offset in [
        zero_start,
        zero_start + chunk - 512,
        zero_start + chunk,
        zero_start + 2 * chunk - 512,
        zero_start + 2 * chunk,
        zero_start + zero_length - 512,
    ] {
        device
            .read_at(DeviceOffsetInBytes(offset), &mut sample)
            .expect("读段内");
        assert!(
            sample.iter().all(|byte| *byte == 0),
            "偏移 {offset} 那一扇区没被清干净"
        );
    }
    for offset in [zero_start - 512, zero_start + zero_length] {
        device
            .read_at(DeviceOffsetInBytes(offset), &mut sample)
            .expect("读段外");
        assert!(
            sample.iter().all(|byte| *byte == 0xC3),
            "偏移 {offset} 在清零段外，不该被动"
        );
    }
}

/// 本线程经 write 一族系统调用交出去的字节数（`/proc/thread-self/io` 的 `wchar`）。
/// 取线程这一份、不取进程那一份：同一个测试二进制里别的用例在别的线程上并行写各自的镜像。
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

/// 打洞那条路：物理块 512、文件系统块 4096，段的头尾都不落在文件系统块边界上，读回整段全 0、
/// 同一个文件系统块里段外的扇区原样不动；第二段清到文件末尾，文件长度不变；两次写零一个字节都没经 write 系统调用。
#[test]
fn punching_ranges_off_the_filesystem_block_boundaries_reads_back_zero_keeps_the_neighbours_and_the_file_length(
) {
    const IMAGE_BYTES: u64 = 65536;
    const DIRT: u8 = 0xC3;
    let path = temporary_image_path("punch");
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        IMAGE_BYTES,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像");
    let image_length = usize::try_from(IMAGE_BYTES).expect("镜像长装得进 usize");
    // 先弄脏整个镜像：稀疏镜像本来就读回全 0，不弄脏分不出「清过」与「从来没写过」。
    device
        .write_at(
            DeviceOffsetInBytes(0),
            &vec![DIRT; image_length],
            WriteDurability::Plain,
        )
        .expect("填脏");
    // 段一 [4608, 16896)：头落在第 2 个文件系统块中间、尾落在第 5 个中间；段二是最后 1536 字节，清到文件末尾。
    let ranges: [(u64, u64); 2] = [(4608, 12288), (IMAGE_BYTES - 1536, 1536)];
    let handed_before = bytes_handed_to_write_calls_by_this_thread();
    for (range_start, range_length) in ranges {
        device
            .write_zeroes_at(DeviceOffsetInBytes(range_start), range_length)
            .expect("写零");
    }
    let handed_during_zero_fills = bytes_handed_to_write_calls_by_this_thread() - handed_before;
    assert_eq!(
        handed_during_zero_fills, 0,
        "打洞不经 write 系统调用；交出去 {handed_during_zero_fills} 字节说明退回了逐块写 0"
    );
    assert_eq!(
        device.zero_fill_landing_counts(),
        ZeroFillLandingCounts {
            punched_calls: 2,
            written_as_zero_bytes_calls: 0,
            punch_attempts_interrupted_by_signal: 0,
        },
        "两次写零都该落成打洞"
    );
    let mut whole_image = vec![0xFFu8; image_length];
    device
        .read_at(DeviceOffsetInBytes(0), &mut whole_image)
        .expect("读回整个镜像");
    for (index, byte) in whole_image.iter().enumerate() {
        let position = u64::try_from(index).expect("下标装得进 u64");
        let is_inside_a_range = ranges.iter().any(|(range_start, range_length)| {
            position >= *range_start && position < range_start + range_length
        });
        if is_inside_a_range {
            assert_eq!(*byte, 0, "偏移 {position} 在清零段内，没被清干净");
        } else {
            assert_eq!(*byte, DIRT, "偏移 {position} 在清零段外，不该被动");
        }
    }
    assert_eq!(
        std::fs::metadata(&path).expect("读镜像元数据").len(),
        IMAGE_BYTES,
        "打洞带 KEEP_SIZE，文件长度不变"
    );
}

/// 打洞报不支持的退路（测试开关当作 `fallocate` 报了 `EOPNOTSUPP`，与真失败走同一段按 errno 分路的判定）：
/// 逐块写 0 铺满整段。段长取 `ZERO_FILL_CHUNK_BYTES` 的 2.5 倍，
/// 拆块的边界上不留没清的洞、段外一个字节不动、文件长度不变，经 write 系统调用交出去的恰好是段长那么多字节。
#[test]
fn an_unsupported_hole_punch_falls_back_to_writing_zero_bytes_across_chunk_boundaries() {
    let path = temporary_image_path("unsupported-punch");
    let chunk = ZERO_FILL_CHUNK_BYTES;
    let zero_start = chunk;
    let zero_length = chunk * 2 + chunk / 2;
    let image_bytes = zero_start + zero_length + chunk;
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        image_bytes,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像")
    .with_test_only_zero_fill_landing(ZeroFillLanding::ActAsIfPunchingWereUnsupported);
    let dirt = vec![0xC3u8; usize::try_from(chunk).expect("块长")];
    let mut filled: u64 = 0;
    while filled < image_bytes {
        let this_piece = usize::try_from((image_bytes - filled).min(chunk)).expect("片长");
        device
            .write_at(
                DeviceOffsetInBytes(filled),
                &dirt[..this_piece],
                WriteDurability::Plain,
            )
            .expect("填脏");
        filled += u64::try_from(this_piece).expect("片长装得进 u64");
    }
    let handed_before = bytes_handed_to_write_calls_by_this_thread();
    device
        .write_zeroes_at(DeviceOffsetInBytes(zero_start), zero_length)
        .expect("写零");
    let handed_during_zero_fill = bytes_handed_to_write_calls_by_this_thread() - handed_before;
    assert_eq!(
        handed_during_zero_fill, zero_length,
        "退路逐块写 0：经 write 交出去的该恰好是段长"
    );
    assert_eq!(
        device.zero_fill_landing_counts(),
        ZeroFillLandingCounts {
            punched_calls: 0,
            written_as_zero_bytes_calls: 1,
            punch_attempts_interrupted_by_signal: 0,
        },
        "这一次写零该落进退路"
    );
    let mut sample = vec![0xFFu8; 512];
    for offset in [
        zero_start,
        zero_start + chunk - 512,
        zero_start + chunk,
        zero_start + 2 * chunk - 512,
        zero_start + 2 * chunk,
        zero_start + zero_length - 512,
    ] {
        device
            .read_at(DeviceOffsetInBytes(offset), &mut sample)
            .expect("读段内");
        assert!(
            sample.iter().all(|byte| *byte == 0),
            "偏移 {offset} 那一扇区没被清干净"
        );
    }
    for offset in [zero_start - 512, zero_start + zero_length] {
        device
            .read_at(DeviceOffsetInBytes(offset), &mut sample)
            .expect("读段外");
        assert!(
            sample.iter().all(|byte| *byte == 0xC3),
            "偏移 {offset} 在清零段外，不该被动"
        );
    }
    assert_eq!(
        std::fs::metadata(&path).expect("读镜像元数据").len(),
        image_bytes,
        "退路写的全在文件里面，文件长度不变"
    );
}

/// 打洞报别的错（测试开关当作 `fallocate` 报了 `EIO`）：照常交回 `InputOutput`、原 errno 带着，
/// 一个 0 都不写——段里还是旧字节，经 write 系统调用一个字节都没交出去，两个落地计数都不动。
#[test]
fn hole_punch_failing_with_an_input_output_error_is_reported_and_writes_no_zero_bytes() {
    const IMAGE_BYTES: u64 = 65536;
    const DIRT: u8 = 0xC3;
    let path = temporary_image_path("failing-punch");
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        IMAGE_BYTES,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像")
    .with_test_only_zero_fill_landing(ZeroFillLanding::ActAsIfPunchingFailedWithAnInputOutputError);
    let image_length = usize::try_from(IMAGE_BYTES).expect("镜像长装得进 usize");
    device
        .write_at(
            DeviceOffsetInBytes(0),
            &vec![DIRT; image_length],
            WriteDurability::Plain,
        )
        .expect("填脏");
    let handed_before = bytes_handed_to_write_calls_by_this_thread();
    let zero_fill_outcome = device.write_zeroes_at(DeviceOffsetInBytes(4096), 16384);
    let handed_during_zero_fill = bytes_handed_to_write_calls_by_this_thread() - handed_before;
    assert!(
        matches!(
            &zero_fill_outcome,
            Err(BlockDeviceError::InputOutput(error)) if error.raw_os_error() == Some(libc::EIO)
        ),
        "打洞报 EIO 该照常交回带着这个 errno 的 InputOutput，实际是 {zero_fill_outcome:?}"
    );
    assert_eq!(
        handed_during_zero_fill, 0,
        "打洞报别的错不退回逐块写 0；交出去 {handed_during_zero_fill} 字节说明退路吞掉了这个错"
    );
    assert_eq!(
        device.zero_fill_landing_counts(),
        ZeroFillLandingCounts {
            punched_calls: 0,
            written_as_zero_bytes_calls: 0,
            punch_attempts_interrupted_by_signal: 0,
        },
        "这一次写零既没打成洞，也没落进退路"
    );
    let mut whole_image = vec![0u8; image_length];
    device
        .read_at(DeviceOffsetInBytes(0), &mut whole_image)
        .expect("读回整个镜像");
    assert!(
        whole_image.iter().all(|byte| *byte == DIRT),
        "打洞报错之后镜像该原样不动，第一个被改的偏移是 {:?}",
        whole_image.iter().position(|byte| *byte != DIRT)
    );
}

/// 按 errno 分路：只有 `EOPNOTSUPP`（与 `ENOTSUP` 同值）与 `ENOSYS` 退回逐块写 0，`EINTR` 原样重打，
/// 读写错、`EBADF`、`EINVAL`、`ENOSPC`、`EPERM` 与没有 errno 的错都照常报。
#[test]
fn only_unsupported_errnos_send_a_failed_hole_punch_to_the_zero_byte_fallback() {
    for errno in [libc::EOPNOTSUPP, libc::ENOTSUP, libc::ENOSYS] {
        assert_eq!(
            classify_hole_punch_failure(&io::Error::from_raw_os_error(errno)),
            HolePunchFailure::PunchingUnsupported,
            "errno {errno} 是不支持打洞，该退回逐块写 0"
        );
    }
    for errno in [
        libc::EIO,
        libc::EBADF,
        libc::EINVAL,
        libc::ENOSPC,
        libc::EPERM,
    ] {
        assert_eq!(
            classify_hole_punch_failure(&io::Error::from_raw_os_error(errno)),
            HolePunchFailure::OtherFailure,
            "errno {errno} 不是不支持打洞，该照常报错"
        );
    }
    assert_eq!(
        classify_hole_punch_failure(&io::Error::from_raw_os_error(libc::EINTR)),
        HolePunchFailure::InterruptedBySignal,
        "EINTR 是被信号打断、什么都没做，该原样重打，不退回逐块写 0、不报错"
    );
    assert_eq!(
        classify_hole_punch_failure(&io::Error::from(io::ErrorKind::InvalidInput)),
        HolePunchFailure::OtherFailure,
        "没有 errno 的错（偏移装不进 off_t 那一类）该照常报错"
    );
}

/// 打洞被信号打断（测试开关当作前 `punches_interrupted` 次 `fallocate` 都报了 `EINTR`，与真失败走同一段按 errno 分路的判定）：
/// 每次都原样重打，最后真打成洞，交回 `Ok`——不退回逐块写 0（经 write 系统调用一个字节都没交出去）、不报错；整段读回 0、
/// 段外原样不动；落地计数记一次打洞、`punches_interrupted` 次被打断。
fn assert_punched_again_after_interruptions_without_falling_back_or_reporting_an_error(
    punches_interrupted: u64,
) {
    const IMAGE_BYTES: u64 = 65536;
    const DIRT: u8 = 0xC3;
    const ZERO_START: u64 = 4096;
    const ZERO_LENGTH: u64 = 16384;
    let path = temporary_image_path("interrupted-punch");
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        IMAGE_BYTES,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像")
    .with_test_only_zero_fill_landing(
        ZeroFillLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal {
            punches_interrupted_before_the_real_one: punches_interrupted,
        },
    );
    let image_length = usize::try_from(IMAGE_BYTES).expect("镜像长装得进 usize");
    device
        .write_at(
            DeviceOffsetInBytes(0),
            &vec![DIRT; image_length],
            WriteDurability::Plain,
        )
        .expect("填脏");
    let handed_before = bytes_handed_to_write_calls_by_this_thread();
    let zero_fill_outcome = device.write_zeroes_at(DeviceOffsetInBytes(ZERO_START), ZERO_LENGTH);
    let handed_during_zero_fill = bytes_handed_to_write_calls_by_this_thread() - handed_before;
    assert!(
            zero_fill_outcome.is_ok(),
            "被信号打断 {punches_interrupted} 次的打洞该每次原样重打、交回 Ok，实际是 {zero_fill_outcome:?}"
        );
    assert_eq!(
        device.zero_fill_landing_counts(),
        ZeroFillLandingCounts {
            punched_calls: 1,
            written_as_zero_bytes_calls: 0,
            punch_attempts_interrupted_by_signal: punches_interrupted,
        },
        "这一次写零被打断 {punches_interrupted} 次、重打成洞，没落进退路"
    );
    assert_eq!(
        handed_during_zero_fill, 0,
        "重打的是洞、不是逐块写 0；交出去 {handed_during_zero_fill} 字节说明 EINTR 落进了退路"
    );
    let mut whole_image = vec![0xFFu8; image_length];
    device
        .read_at(DeviceOffsetInBytes(0), &mut whole_image)
        .expect("读回整个镜像");
    for (index, byte) in whole_image.iter().enumerate() {
        let position = u64::try_from(index).expect("下标装得进 u64");
        if (ZERO_START..ZERO_START + ZERO_LENGTH).contains(&position) {
            assert_eq!(*byte, 0, "偏移 {position} 在清零段内，没被清干净");
        } else {
            assert_eq!(*byte, DIRT, "偏移 {position} 在清零段外，不该被动");
        }
    }
}

/// 打洞被信号打断一次：原样重打一次、真打成洞（[`assert_punched_again_after_interruptions_without_falling_back_or_reporting_an_error`]）。
#[test]
fn hole_punch_interrupted_by_a_signal_is_punched_again_without_falling_back_or_reporting_an_error()
{
    assert_punched_again_after_interruptions_without_falling_back_or_reporting_an_error(1);
}

/// 打洞连着被信号打断两次：第二次被打断也照样原样重打、第三次真打成洞，不报错、不退回逐块写 0。
/// 「被打断第二次就报错」（重打只给一次）时这里交回 `InputOutput`。
#[test]
fn hole_punch_interrupted_by_a_signal_twice_in_a_row_is_punched_a_third_time_without_falling_back_or_reporting_an_error(
) {
    assert_punched_again_after_interruptions_without_falling_back_or_reporting_an_error(2);
}

/// 一次长度 0 的写零在一个后端上的结局：交回什么、本线程经 write 系统调用交出去几个字节、镜像变没变。
#[derive(Debug, PartialEq, Eq)]
struct ZeroLengthZeroFillOutcome {
    returned: String,
    bytes_handed_to_write_calls: u64,
    is_the_image_unchanged: bool,
}

/// 在 `device` 上于偏移 0、中间、设备末尾、越过末尾一个物理块、没按物理块（512 字节）对齐的半块处各写零一次、长度都是 0，
/// 按这个次序交回每一次的结局。`device` 的镜像事先填满 `dirt`。
fn zero_length_zero_fills_on(
    device: &mut dyn BlockDevice,
    image_bytes: u64,
    dirt: u8,
) -> Vec<ZeroLengthZeroFillOutcome> {
    let image_length = usize::try_from(image_bytes).expect("镜像长装得进 usize");
    let offset_one_block_past_the_end = image_bytes + 512;
    let offset_half_a_block_in = 256;
    [
        0,
        image_bytes / 2,
        image_bytes,
        offset_one_block_past_the_end,
        offset_half_a_block_in,
    ]
    .into_iter()
    .map(|offset| {
        let handed_before = bytes_handed_to_write_calls_by_this_thread();
        let returned = format!(
            "{:?}",
            device.write_zeroes_at(DeviceOffsetInBytes(offset), 0)
        );
        let bytes_handed_to_write_calls =
            bytes_handed_to_write_calls_by_this_thread() - handed_before;
        let mut whole_image = vec![0u8; image_length];
        device
            .read_at(DeviceOffsetInBytes(0), &mut whole_image)
            .expect("读回整个镜像");
        ZeroLengthZeroFillOutcome {
            returned,
            bytes_handed_to_write_calls,
            is_the_image_unchanged: whole_image.iter().all(|byte| *byte == dirt),
        }
    })
    .collect()
}

/// 长度 0 的写零：文件后端与 `O_DIRECT` 后端（绕开页缓存、走页缓存两臂）在偏移 0、中间、
/// 设备末尾三处结局逐项相同——交回 `Ok`、经 write 系统调用一个字节都没交出去、镜像一个字节不动。文件后端另核落地计数一样都不记
/// （长度 0 不调 `fallocate`；调了内核报 `EINVAL`，交回的就是 `InputOutput`）。`O_DIRECT` 后端没有落地计数，比到「交回什么、
/// 交出去几个字节、镜像变没变」这三步为止。越过末尾一个物理块、没按物理块对齐的两处长度 0，
/// 三个后端照样先过请求判据、都拒成 `OutOfRange`、`Unaligned`，一个字节不动（长度 0 的早退挪到请求判据之前时文件后端交 `Ok`）。
/// 内存稀疏盘（`singlefs_verification::memory_pool::SparseBlockDevice`）在验证包，核心层的单测拿不到它，不在这里比
/// （它与文件后端对这两处长度 0 的请求逐个对拍在 `sparse_devices_refuse_requests_outside_the_device_like_the_file_backend`）。
#[test]
fn zero_length_zero_fill_ends_the_same_on_the_file_backend_and_both_arms_of_the_direct_backend() {
    const IMAGE_BYTES: u64 = 65536;
    const DIRT: u8 = 0x5A;
    let image_length = usize::try_from(IMAGE_BYTES).expect("镜像长装得进 usize");
    let accepted = || ZeroLengthZeroFillOutcome {
        returned: "Ok(())".to_string(),
        bytes_handed_to_write_calls: 0,
        is_the_image_unchanged: true,
    };
    let refused = |refusal: BlockDeviceError| ZeroLengthZeroFillOutcome {
        returned: format!("{:?}", Err::<(), BlockDeviceError>(refusal)),
        bytes_handed_to_write_calls: 0,
        is_the_image_unchanged: true,
    };
    let expected: Vec<ZeroLengthZeroFillOutcome> = vec![
        accepted(),
        accepted(),
        accepted(),
        refused(BlockDeviceError::OutOfRange {
            offset: DeviceOffsetInBytes(IMAGE_BYTES + 512),
            length: 0,
            device_size: IMAGE_BYTES,
        }),
        refused(BlockDeviceError::Unaligned {
            offset: DeviceOffsetInBytes(256),
            length: 0,
            physical_block_size: 512,
        }),
    ];

    let file_path = temporary_image_path("zero-length-file");
    let mut file_device = FileBackedBlockDevice::create_image_file_exclusively(
        &file_path,
        IMAGE_BYTES,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像");
    file_device
        .write_at(
            DeviceOffsetInBytes(0),
            &vec![DIRT; image_length],
            WriteDurability::Plain,
        )
        .expect("填脏");
    let file_outcomes = zero_length_zero_fills_on(&mut file_device, IMAGE_BYTES, DIRT);
    assert_eq!(
        file_device.zero_fill_landing_counts(),
        NO_ZERO_FILL_LANDED_YET,
        "长度 0 的写零既不打洞也不退回逐块写 0"
    );
    assert_eq!(
        file_outcomes, expected,
        "文件后端：设备里的长度 0 写零交回 Ok、什么都不做；越过末尾、没对齐的照样拒"
    );

    for policy in [
        PageCachePolicy::BypassWithDirectInputOutput,
        PageCachePolicy::GoThroughPageCache,
    ] {
        let direct_path = temporary_image_path("zero-length-direct");
        std::fs::write(&direct_path, vec![DIRT; image_length]).expect("建填脏的镜像");
        let mut direct_device = DirectInputOutputBlockDevice::open_existing(
            &direct_path,
            policy,
            PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(512)),
        )
        .expect("打开（临时目录在 ext4 上，O_DIRECT 打得开）");
        let direct_outcomes = zero_length_zero_fills_on(&mut direct_device, IMAGE_BYTES, DIRT);
        assert_eq!(
            direct_outcomes, file_outcomes,
            "{policy:?}：长度 0 的写零与文件后端结局逐项相同"
        );
    }
}

/// 写零与写走同一条前置判定：偏移或长度不按物理块对齐、或者越过设备末尾，都在动盘之前拒绝。
#[test]
fn writing_zeroes_rejects_unaligned_and_out_of_range_ranges_before_touching_the_file() {
    let path = temporary_image_path("zeroes-refused");
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        16384,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像");
    device
        .write_at(
            DeviceOffsetInBytes(0),
            &[0x7Eu8; 512],
            WriteDurability::Plain,
        )
        .expect("先写一扇区");
    assert!(matches!(
        device.write_zeroes_at(DeviceOffsetInBytes(1), 512),
        Err(BlockDeviceError::Unaligned { .. })
    ));
    assert!(matches!(
        device.write_zeroes_at(DeviceOffsetInBytes(0), 513),
        Err(BlockDeviceError::Unaligned { .. })
    ));
    assert!(matches!(
        device.write_zeroes_at(DeviceOffsetInBytes(16384), 512),
        Err(BlockDeviceError::OutOfRange { .. })
    ));
    let mut read_back = vec![0u8; 512];
    device
        .read_at(DeviceOffsetInBytes(0), &mut read_back)
        .expect("读");
    assert_eq!(read_back, vec![0x7Eu8; 512], "被拒的写零一个字节都没落盘");
}

/// 撞上同名文件一律报错、盘上那一份字节一个都不动（进程号会被系统复用、上一轮被杀留下的镜像不会被清：
/// 接着用旧镜像就是拿上一轮的盘面当这一轮的起点）。重开走另一条入口，文件不在时也报错、不静默建空盘。
#[test]
fn creating_an_image_that_already_exists_is_refused_and_leaves_the_old_bytes_alone() {
    let path = temporary_image_path("exclusive");
    let mut first = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        8192,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("第一次建得出来");
    first
        .write_at(
            DeviceOffsetInBytes(0),
            &[0x5Au8; 512],
            WriteDurability::ForceUnitAccess,
        )
        .expect("写一扇区");
    drop(first);
    let again = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        8192,
        PhysicalBlockSizeInBytes(512),
    );
    let refusal = again.err().map(|error| error.to_string());
    assert!(
        refusal
            .as_deref()
            .is_some_and(|text| text.contains("要建的镜像文件已经在那儿了")),
        "撞上同名文件要报 ImageFileAlreadyExists，报的是 {refusal:?}"
    );
    let reopened =
        FileBackedBlockDevice::open_existing_image_file(&path, 8192, PhysicalBlockSizeInBytes(512))
            .expect("重开得出来");
    let mut first_sector = [0u8; 512];
    reopened
        .read_at(DeviceOffsetInBytes(0), &mut first_sector)
        .expect("读回第一扇区");
    assert!(
        first_sector.iter().all(|byte| *byte == 0x5A),
        "被拒之后旧镜像逐字节不变"
    );
    drop(reopened);
    // 守卫丢掉时也会删；这里先删，是要看重开一份不在的镜像报什么
    std::fs::remove_file(&path).expect("清理镜像");
    let missing =
        FileBackedBlockDevice::open_existing_image_file(&path, 8192, PhysicalBlockSizeInBytes(512));
    let absence = missing.err().map(|error| error.to_string());
    assert!(
        absence
            .as_deref()
            .is_some_and(|text| text.contains("要重开的镜像文件不在")),
        "重开一个不在的镜像要报 ImageFileMissing，报的是 {absence:?}"
    );
    assert!(!path.exists(), "重开被拒之后没有建出文件来");
}

#[test]
fn unaligned_and_out_of_range_requests_are_rejected_before_touching_the_file() {
    let path = temporary_image_path("bounds");
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        8192,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像");
    let unaligned = device.write_at(
        DeviceOffsetInBytes(100),
        &[0u8; 512],
        WriteDurability::Plain,
    );
    assert!(matches!(unaligned, Err(BlockDeviceError::Unaligned { .. })));
    let short = device.write_at(DeviceOffsetInBytes(0), &[0u8; 100], WriteDurability::Plain);
    assert!(matches!(short, Err(BlockDeviceError::Unaligned { .. })));
    let beyond = device.write_at(
        DeviceOffsetInBytes(8192),
        &[0u8; 512],
        WriteDurability::Plain,
    );
    assert!(matches!(beyond, Err(BlockDeviceError::OutOfRange { .. })));
    let exactly_to_the_end = device.write_at(
        DeviceOffsetInBytes(7680),
        &[1u8; 512],
        WriteDurability::Plain,
    );
    assert!(exactly_to_the_end.is_ok(), "写到末尾正好一块是合法的");
    assert_eq!(
        device.probe_physical_block_size(),
        PhysicalBlockSizeInBytes(512)
    );
    assert_eq!(device.size_in_bytes(), 8192);
}

#[test]
fn direct_input_output_backend_round_trips_aligned_and_rejects_unaligned_requests() {
    let path = temporary_image_path("direct");
    std::fs::File::create(&path)
        .expect("建文件")
        .set_len(65536)
        .expect("撑大小");
    let mut device = DirectInputOutputBlockDevice::open_existing(
        &path,
        PageCachePolicy::BypassWithDirectInputOutput,
        PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(512)),
    )
    .expect("O_DIRECT 打开（临时目录在 ext4 上）");
    assert_eq!(
        device.size_in_bytes(),
        65536,
        "大小取设备末尾的偏移，不改大小"
    );
    let payload: Vec<u8> = (0..4096u32)
        .map(|index| u8::try_from(index % 253).expect("小于 256"))
        .collect();
    device
        .write_at(
            DeviceOffsetInBytes(8192),
            &payload,
            WriteDurability::ForceUnitAccess,
        )
        .expect("写");
    device.barrier().expect("屏障");
    let mut read_back = vec![0u8; 4096];
    device
        .read_at(DeviceOffsetInBytes(8192), &mut read_back)
        .expect("读");
    assert_eq!(read_back, payload);
    let unaligned = device.write_at(
        DeviceOffsetInBytes(100),
        &[0u8; 512],
        WriteDurability::Plain,
    );
    assert!(matches!(unaligned, Err(BlockDeviceError::Unaligned { .. })));
    let beyond = device.write_at(
        DeviceOffsetInBytes(65536),
        &[0u8; 512],
        WriteDurability::Plain,
    );
    assert!(matches!(beyond, Err(BlockDeviceError::OutOfRange { .. })));
    let probed = DirectInputOutputBlockDevice::open_existing(
        &path,
        PageCachePolicy::BypassWithDirectInputOutput,
        PhysicalBlockSizeSource::ProbeSysfs,
    );
    assert!(
        matches!(
            probed,
            Err(BlockDeviceError::ProbeFailed {
                attribute: "physical_block_size"
            })
        ),
        "普通文件在 sysfs 里没有 queue 属性：探不到就报错，不拿写死的值顶上"
    );
}

#[test]
fn queue_attributes_are_read_from_the_sysfs_tree_and_missing_ones_are_none() {
    let root = TemporaryPathRemovedOnDrop(
        std::env::temp_dir().join(format!("singlefs-core-fake-sysfs-{}", std::process::id())),
    );
    let queue = root.join("vdz").join("queue");
    std::fs::create_dir_all(&queue).expect("建假 sysfs");
    std::fs::write(queue.join("physical_block_size"), "4096\n").expect("写属性");
    std::fs::write(queue.join("write_cache"), "write back\n").expect("写属性");
    std::fs::write(queue.join("minimum_io_size"), "not-a-number\n").expect("写属性");
    let device = Path::new("/dev/vdz");
    assert_eq!(
        probe_queue_number_under(&root, device, "physical_block_size"),
        Some(4096)
    );
    assert_eq!(
        probe_queue_text_under(&root, device, "write_cache").as_deref(),
        Some("write back")
    );
    assert_eq!(
        probe_queue_number_under(&root, device, "minimum_io_size"),
        None,
        "不是数就是读不到，不当 0"
    );
    assert_eq!(
        probe_queue_number_under(&root, device, "logical_block_size"),
        None,
        "文件不在就是读不到"
    );
    assert_eq!(
        probe_queue_number_under(&root, Path::new("/dev/vdy"), "physical_block_size"),
        None
    );
}

#[test]
fn aligned_scratch_starts_on_a_4096_boundary_for_every_length() {
    for length in [512usize, 4096, 16384, 32768, 1 << 20] {
        let scratch = AlignedScratch::new(length);
        assert_eq!(
            scratch.as_slice().as_ptr().align_offset(4096),
            0,
            "长度 {length} 的暂存区没有按 4096 对齐"
        );
        assert_eq!(scratch.as_slice().len(), length);
    }
}

/// discard 用例的镜像：64 KiB、整个先填成 0xC3（稀疏镜像本来就读回全 0，不先弄脏就分不出「打过洞」与「从来没写过」），
/// 换上 `landing` 那一种 discard 落地方式。
const DISCARD_IMAGE_BYTES: u64 = 65536;
const DISCARD_DIRT_BYTE: u8 = 0xC3;

fn dirty_image_for_discards(
    tag: &str,
    landing: DiscardLanding,
) -> (FileBackedBlockDevice, TemporaryPathRemovedOnDrop) {
    let path = temporary_image_path(tag);
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        DISCARD_IMAGE_BYTES,
        PhysicalBlockSizeInBytes(512),
    )
    .expect("建镜像");
    let dirt = vec![DISCARD_DIRT_BYTE; usize::try_from(DISCARD_IMAGE_BYTES).expect("64 KiB")];
    device
        .write_at(DeviceOffsetInBytes(0), &dirt, WriteDurability::Plain)
        .expect("填脏");
    (device.with_test_only_discard_landing(landing), path)
}

fn sector_at(device: &FileBackedBlockDevice, offset: u64) -> Vec<u8> {
    let mut sector = vec![0x5Au8; 512];
    device
        .read_at(DeviceOffsetInBytes(offset), &mut sector)
        .expect("读一扇区");
    sector
}

/// 文件后端的 discard 复用打洞：段内读回全 0、段外一个字节不动、文件长度不变，交回「交给了底层」并记一次。
#[test]
fn discard_on_the_file_backend_punches_the_range_back_to_zero_and_leaves_the_neighbours() {
    let (mut device, _removed) = dirty_image_for_discards(
        "discard-punched",
        DiscardLanding::PunchHoleOrAnswerUnsupported,
    );
    let outcome = device
        .discard_at(DeviceOffsetInBytes(8192), 16384)
        .expect("discard");
    assert_eq!(outcome, DiscardOutcome::Applied);
    for offset in [8192u64, 8192 + 16384 - 512] {
        assert!(
            sector_at(&device, offset).iter().all(|byte| *byte == 0),
            "偏移 {offset} 在 discard 段内，打过洞读回全 0"
        );
    }
    for offset in [8192u64 - 512, 8192 + 16384] {
        assert!(
            sector_at(&device, offset)
                .iter()
                .all(|byte| *byte == DISCARD_DIRT_BYTE),
            "偏移 {offset} 在 discard 段外，不该被动"
        );
    }
    assert_eq!(
        device.file.metadata().expect("读文件元数据").len(),
        DISCARD_IMAGE_BYTES,
        "打洞不改文件长度"
    );
    assert_eq!(
        device.discard_outcome_counts(),
        DiscardOutcomeCounts {
            applied_calls: 1,
            unsupported_calls: 0
        }
    );
    assert_eq!(
        device.zero_fill_landing_counts(),
        NO_ZERO_FILL_LANDED_YET,
        "discard 不记进写零的落地计数"
    );
}

/// 文件系统不支持打洞：交回「不支持」、一个 0 都不写（段里还是旧字节），记一次「不支持」。
#[test]
fn discard_where_hole_punching_is_unsupported_answers_unsupported_and_leaves_the_old_bytes() {
    let (mut device, _removed) = dirty_image_for_discards(
        "discard-unsupported",
        DiscardLanding::ActAsIfPunchingWereUnsupported,
    );
    let outcome = device
        .discard_at(DeviceOffsetInBytes(8192), 16384)
        .expect("discard");
    assert_eq!(outcome, DiscardOutcome::Unsupported);
    assert!(
        sector_at(&device, 8192)
            .iter()
            .all(|byte| *byte == DISCARD_DIRT_BYTE),
        "不支持时一个字节不动"
    );
    assert_eq!(
        device.discard_outcome_counts(),
        DiscardOutcomeCounts {
            applied_calls: 0,
            unsupported_calls: 1
        }
    );
}

/// 打洞报 `EIO`：照常交回块设备 I/O 错，不吞成「不支持」，段里还是旧字节，两种结局都不记。
#[test]
fn discard_whose_hole_punch_fails_with_an_input_output_error_is_reported_and_changes_nothing() {
    let (mut device, _removed) = dirty_image_for_discards(
        "discard-input-output-error",
        DiscardLanding::ActAsIfPunchingFailedWithAnInputOutputError,
    );
    let outcome = device.discard_at(DeviceOffsetInBytes(8192), 16384);
    assert!(
        matches!(&outcome, Err(BlockDeviceError::InputOutput(error)) if error.raw_os_error() == Some(libc::EIO)),
        "EIO 照常报错：{outcome:?}"
    );
    assert!(sector_at(&device, 8192)
        .iter()
        .all(|byte| *byte == DISCARD_DIRT_BYTE));
    assert_eq!(device.discard_outcome_counts(), DiscardOutcomeCounts::NONE);
}

/// 打洞连着被信号打断两次：原样重打到第三次，打成洞、交回「交给了底层」，被打断的次数记 2。
#[test]
fn discard_interrupted_by_a_signal_twice_in_a_row_is_punched_a_third_time() {
    let (mut device, _removed) = dirty_image_for_discards(
        "discard-interrupted",
        DiscardLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal {
            punches_interrupted_before_the_real_one: 2,
        },
    );
    let outcome = device
        .discard_at(DeviceOffsetInBytes(8192), 16384)
        .expect("被打断之后重打成功");
    assert_eq!(outcome, DiscardOutcome::Applied);
    assert!(sector_at(&device, 8192).iter().all(|byte| *byte == 0));
    assert_eq!(device.discard_punch_attempts_interrupted_by_signal(), 2);
    assert_eq!(device.discard_outcome_counts().applied_calls, 1);
}

/// 没对齐、越过末尾的 discard 在动文件之前就拒，与读写同一条判据；段里还是旧字节。
#[test]
fn discard_off_the_block_boundaries_or_past_the_end_is_refused_before_touching_the_file() {
    let (mut device, _removed) = dirty_image_for_discards(
        "discard-refused",
        DiscardLanding::PunchHoleOrAnswerUnsupported,
    );
    assert!(matches!(
        device.discard_at(DeviceOffsetInBytes(100), 512),
        Err(BlockDeviceError::Unaligned { .. })
    ));
    assert!(matches!(
        device.discard_at(DeviceOffsetInBytes(DISCARD_IMAGE_BYTES - 512), 1024),
        Err(BlockDeviceError::OutOfRange { .. })
    ));
    assert!(sector_at(&device, DISCARD_IMAGE_BYTES - 512)
        .iter()
        .all(|byte| *byte == DISCARD_DIRT_BYTE));
    assert_eq!(device.discard_outcome_counts(), DiscardOutcomeCounts::NONE);
}

/// 文件后端上长度 0 的 discard 没有动作：交回「交给了底层」、不调 `fallocate`、两种结局都不记，段里还是旧字节
#[test]
fn zero_length_discard_on_the_file_backend_is_applied_without_being_counted() {
    let (mut device, _removed) = dirty_image_for_discards(
        "discard-zero-length",
        DiscardLanding::PunchHoleOrAnswerUnsupported,
    );
    let outcome = device
        .discard_at(DeviceOffsetInBytes(8192), 0)
        .expect("长度 0 的 discard");
    assert_eq!(outcome, DiscardOutcome::Applied, "长度 0 交回交给了底层");
    assert_eq!(
        device.discard_outcome_counts(),
        DiscardOutcomeCounts::NONE,
        "长度 0 没有动作，不记数"
    );
    assert!(sector_at(&device, 8192)
        .iter()
        .all(|byte| *byte == DISCARD_DIRT_BYTE));
}

/// `O_DIRECT` 后端上长度 0 的 discard 没有动作：两臂都交回「交给了底层」、不记「不支持」。
#[test]
fn zero_length_discard_on_the_direct_backend_is_applied_without_being_counted_on_both_arms() {
    for policy in [
        PageCachePolicy::BypassWithDirectInputOutput,
        PageCachePolicy::GoThroughPageCache,
    ] {
        let path = temporary_image_path("direct-discard-zero-length");
        let mut file = std::fs::File::create(&path).expect("建文件");
        std::io::Write::write_all(&mut file, &[DISCARD_DIRT_BYTE; 65536]).expect("填脏");
        drop(file);
        let mut device = DirectInputOutputBlockDevice::open_existing(
            &path,
            policy,
            PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(512)),
        )
        .expect("打开（临时目录在 ext4 上）");
        assert_eq!(
            device
                .discard_at(DeviceOffsetInBytes(8192), 0)
                .expect("长度 0 的 discard"),
            DiscardOutcome::Applied,
            "{policy:?}"
        );
        assert_eq!(
            device.discard_outcome_counts(),
            DiscardOutcomeCounts::NONE,
            "{policy:?}：长度 0 没有动作，不记数"
        );
    }
}

/// `O_DIRECT` 后端第一版不认 discard：两臂都交回「不支持」、一个字节不动、每次记一次；没对齐的照样拒。
#[test]
fn the_direct_backend_answers_every_discard_unsupported_on_both_arms_and_counts_it() {
    for policy in [
        PageCachePolicy::BypassWithDirectInputOutput,
        PageCachePolicy::GoThroughPageCache,
    ] {
        let path = temporary_image_path("direct-discard");
        let mut file = std::fs::File::create(&path).expect("建文件");
        std::io::Write::write_all(&mut file, &[DISCARD_DIRT_BYTE; 65536]).expect("填脏");
        drop(file);
        let mut device = DirectInputOutputBlockDevice::open_existing(
            &path,
            policy,
            PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(512)),
        )
        .expect("打开（临时目录在 ext4 上）");
        assert_eq!(
            device
                .discard_at(DeviceOffsetInBytes(8192), 16384)
                .expect("discard"),
            DiscardOutcome::Unsupported,
            "{policy:?}"
        );
        assert!(matches!(
            device.discard_at(DeviceOffsetInBytes(100), 512),
            Err(BlockDeviceError::Unaligned { .. })
        ));
        let mut sector = vec![0u8; 512];
        device
            .read_at(DeviceOffsetInBytes(8192), &mut sector)
            .expect("读");
        assert!(sector.iter().all(|byte| *byte == DISCARD_DIRT_BYTE));
        assert_eq!(
            device.discard_outcome_counts(),
            DiscardOutcomeCounts {
                applied_calls: 0,
                unsupported_calls: 1
            },
            "{policy:?}"
        );
    }
}

/// 差分测试用的镜像：64 KiB、物理块 512。
const CONTRACT_IMAGE_BYTES: u64 = 65536;
const CONTRACT_PHYSICAL_BLOCK_BYTES: u32 = 512;

/// 差分测试按次发的请求：对齐落在盘里的、正好到盘尾的、长度 0 的（盘首与盘尾）、偏移没对齐的、长度没对齐的、整段越过盘尾的、
/// 跨过盘尾的、偏移加长度越过 `u64` 的。
const CONTRACT_REQUESTS: [(u64, u64); 10] = [
    (0, 512),
    (4096, 4096),
    (CONTRACT_IMAGE_BYTES - 512, 512),
    (0, 0),
    (CONTRACT_IMAGE_BYTES, 0),
    (100, 512),
    (512, 100),
    (CONTRACT_IMAGE_BYTES, 512),
    (CONTRACT_IMAGE_BYTES - 512, 1024),
    (u64::MAX - 511, 512),
];

/// 一段写下去的字节：随偏移、长度与第几趟变，挪了位置、少写一截、写成旧的一趟都读得出来。
fn contract_pattern(offset: u64, length: u64, round: u8) -> Vec<u8> {
    (0..length)
        .map(|index| {
            let mixed = (offset + index).wrapping_mul(31) ^ length ^ u64::from(round);
            u8::try_from(mixed % 251).expect("小于 251")
        })
        .collect()
}

/// 读回整块盘，与模型逐字节比。
fn assert_device_reads_back_as_the_model<Device: BlockDevice>(
    device: &Device,
    model: &[u8],
    label: &str,
) {
    let mut whole_device = vec![0xEEu8; model.len()];
    device
        .read_at(DeviceOffsetInBytes(0), &mut whole_device)
        .expect("读回整块盘");
    assert!(
        whole_device == model,
        "{label}：读回的盘与按交回 Ok 的写、写零算出来的模型不同（第一处不同在偏移 {:?}）",
        whole_device
            .iter()
            .zip(model)
            .position(|(read, expected)| read != expected)
    );
}

/// 特质契约在一块真盘上逐请求核（被验的 `verus!` 那一侧只信任 `pread` / `pwrite` / `fdatasync` 三个外壳，这里拿真后端对它们的规格）：
/// 读、写（普通与 FUA）、写零各发一遍 [`CONTRACT_REQUESTS`]，交回 `Ok` 恰好是 [`check_aligned_and_in_range`]（它与规格里的
/// `request_is_aligned_and_in_range` 等价，Verus 验过）说收的那些；读不改缓冲长度；交回 `Ok` 的写与写零恰好落在请求那一段
/// （整块盘读回与一份按 `Ok` 的写算出来的字节模型逐字节相同），交回 `Err` 的一个字节不动；屏障交回 `Ok`、不改字节；
/// 物理块大小与设备字节数一直不变。
fn assert_the_block_device_contract_holds_on_every_request<Device: BlockDevice>(
    device: &mut Device,
    label: &str,
) {
    let block = PhysicalBlockSizeInBytes(CONTRACT_PHYSICAL_BLOCK_BYTES);
    let mut model =
        vec![0u8; usize::try_from(CONTRACT_IMAGE_BYTES).expect("镜像长装得进 usize")];
    for (round, durability) in [
        (1u8, WriteDurability::Plain),
        (2u8, WriteDurability::ForceUnitAccess),
    ] {
        for (offset, length) in CONTRACT_REQUESTS {
            let accepted = check_aligned_and_in_range(
                DeviceOffsetInBytes(offset),
                length,
                block,
                CONTRACT_IMAGE_BYTES,
            )
            .is_ok();
            let length_in_memory = usize::try_from(length).expect("请求长度装得进 usize");
            let pattern = contract_pattern(offset, length, round);
            let written = device.write_at(DeviceOffsetInBytes(offset), &pattern, durability);
            assert_eq!(
                    written.is_ok(),
                    accepted,
                    "{label}：写 ({offset}, {length}, {durability:?}) 交回 {written:?}，判据说收不收是 {accepted}"
                );
            if written.is_ok() {
                let start = usize::try_from(offset).expect("偏移装得进 usize");
                model[start..start + length_in_memory].copy_from_slice(&pattern);
            }
            assert_device_reads_back_as_the_model(device, &model, label);
            let mut buffer = vec![0x5Au8; length_in_memory];
            let read = device.read_at(DeviceOffsetInBytes(offset), &mut buffer);
            assert_eq!(
                read.is_ok(),
                accepted,
                "{label}：读 ({offset}, {length}) 交回 {read:?}，判据说收不收是 {accepted}"
            );
            assert_eq!(buffer.len(), length_in_memory, "{label}：读不改缓冲长度");
            if read.is_ok() {
                let start = usize::try_from(offset).expect("偏移装得进 usize");
                assert!(
                    buffer == model[start..start + length_in_memory],
                    "{label}：读 ({offset}, {length}) 读到的不是这一段的字节"
                );
            }
            let zeroed = device.write_zeroes_at(DeviceOffsetInBytes(offset), length);
            assert_eq!(
                zeroed.is_ok(),
                accepted,
                "{label}：写零 ({offset}, {length}) 交回 {zeroed:?}，判据说收不收是 {accepted}"
            );
            if zeroed.is_ok() {
                let start = usize::try_from(offset).expect("偏移装得进 usize");
                model[start..start + length_in_memory].fill(0);
            }
            assert_device_reads_back_as_the_model(device, &model, label);
            let rewritten = device.write_at(DeviceOffsetInBytes(offset), &pattern, durability);
            assert_eq!(
                rewritten.is_ok(),
                accepted,
                "{label}：写零之后重写 ({offset}, {length})"
            );
            if rewritten.is_ok() {
                let start = usize::try_from(offset).expect("偏移装得进 usize");
                model[start..start + length_in_memory].copy_from_slice(&pattern);
            }
            let fenced = device.barrier();
            assert!(fenced.is_ok(), "{label}：屏障交回 {fenced:?}");
            assert_device_reads_back_as_the_model(device, &model, label);
            assert_eq!(
                device.probe_physical_block_size(),
                block,
                "{label}：物理块大小不变"
            );
            assert_eq!(
                device.size_in_bytes(),
                CONTRACT_IMAGE_BYTES,
                "{label}：设备字节数不变"
            );
        }
    }
}

/// 场景：文件后端的一块 64 KiB 镜像上逐请求发读、写、写零、屏障。预期：特质契约逐条成立（见 `assert_the_block_device_contract_holds_on_every_request`）。
/// 它是信任清单里 `read_exact_at_the_offset`、`write_all_at_the_offset`、`sync_data_of_the_file` 三个外壳的差分测试：
/// 判别力（`crates/mutations.tsv`）：写的外壳把字节写到偏移后面一块、读的外壳少读一截、`fdatasync` 的外壳交回错，这条都红。
#[test]
fn the_file_backend_keeps_the_block_device_contract_on_every_request() {
    let path = temporary_image_path("contract-file");
    let mut device = FileBackedBlockDevice::create_image_file_exclusively(
        &path,
        CONTRACT_IMAGE_BYTES,
        PhysicalBlockSizeInBytes(CONTRACT_PHYSICAL_BLOCK_BYTES),
    )
    .expect("建镜像");
    assert_the_block_device_contract_holds_on_every_request(&mut device, "文件后端");
}

/// 场景：`O_DIRECT` 后端两臂（绕开页缓存、走页缓存）各开同一块 64 KiB 普通文件，逐请求发读、写、写零、屏障。
/// 预期：两臂特质契约都成立。绕开那一臂经对齐暂存区读写（信任清单里 `AlignedScratch::new` 的外壳），走页缓存那一臂直接读写。
#[test]
fn the_direct_backend_keeps_the_block_device_contract_on_every_request_on_both_arms() {
    for policy in [
        PageCachePolicy::BypassWithDirectInputOutput,
        PageCachePolicy::GoThroughPageCache,
    ] {
        let path = temporary_image_path("contract-direct");
        std::fs::File::create(&path)
            .expect("建文件")
            .set_len(CONTRACT_IMAGE_BYTES)
            .expect("撑大小");
        let mut device = DirectInputOutputBlockDevice::open_existing(
            &path,
            policy,
            PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(
                CONTRACT_PHYSICAL_BLOCK_BYTES,
            )),
        )
        .expect("打开（临时目录在 ext4 上）");
        assert_the_block_device_contract_holds_on_every_request(
            &mut device,
            &format!("O_DIRECT 后端 {policy:?}"),
        );
    }
}

/// 场景：块设备错的每一个成员各造一个，写成文字。预期：写出来的字里带着这个成员的字段（偏移、长度、设备字节数、物理块、底层错、
/// 属性名、路径），两个路径成员分得开。它是信任清单里 `BlockDeviceError::fmt`（`external_body`）的差分测试。
/// 判别力（`crates/mutations.tsv`）：越界那一句把设备字节数写成长度，这条红。
#[test]
fn every_block_device_error_displays_the_fields_it_carries() {
    let input_output_text = io::Error::from_raw_os_error(libc::EIO).to_string();
    let cases: Vec<(BlockDeviceError, Vec<String>)> = vec![
        (
            BlockDeviceError::OutOfRange {
                offset: DeviceOffsetInBytes(4096),
                length: 1536,
                device_size: 8192,
            },
            vec![
                "越过设备末尾".to_string(),
                "4096".to_string(),
                "1536".to_string(),
                "8192".to_string(),
            ],
        ),
        (
            BlockDeviceError::Unaligned {
                offset: DeviceOffsetInBytes(100),
                length: 700,
                physical_block_size: 512,
            },
            vec![
                "没按物理块对齐".to_string(),
                "100".to_string(),
                "700".to_string(),
                "512".to_string(),
            ],
        ),
        (
            BlockDeviceError::InputOutput(io::Error::from_raw_os_error(libc::EIO)),
            vec!["I/O 错".to_string(), input_output_text],
        ),
        (
            BlockDeviceError::ProbeFailed {
                attribute: "physical_block_size",
            },
            vec!["queue/physical_block_size".to_string()],
        ),
        (
            BlockDeviceError::ImageFileAlreadyExists {
                path: PathBuf::from("/tmp/singlefs-display-already-there.img"),
            },
            vec![
                "已经在那儿了".to_string(),
                "/tmp/singlefs-display-already-there.img".to_string(),
            ],
        ),
        (
            BlockDeviceError::ImageFileMissing {
                path: PathBuf::from("/tmp/singlefs-display-missing.img"),
            },
            vec![
                "要重开的镜像文件不在".to_string(),
                "/tmp/singlefs-display-missing.img".to_string(),
            ],
        ),
    ];
    for (error, pieces) in cases {
        let text = error.to_string();
        for piece in pieces {
            assert!(
                text.contains(&piece),
                "{error:?} 写成「{text}」，里面没有「{piece}」"
            );
        }
    }
}

/// 场景：只供测试的落地开关当作 `fallocate` 报了哪一种失败，造出那个 errno 的 I/O 错，再交给按 errno 分路的判定。
/// 预期：三种各是条款写的那个 errno（不支持 `EOPNOTSUPP`、打断 `EINTR`、别的错 `EIO`），分回原来那条路。
/// 它是信任清单里 `hole_punch_error_the_test_switch_stands_for`（`external_body`）的差分测试。
/// 判别力（`crates/mutations.tsv`）：被信号打断造成 `EIO`，这条红。
#[test]
fn every_simulated_hole_punch_failure_carries_its_errno_and_is_sorted_back_onto_its_own_path() {
    for (failure, errno) in [
        (HolePunchFailure::PunchingUnsupported, libc::EOPNOTSUPP),
        (HolePunchFailure::InterruptedBySignal, libc::EINTR),
        (HolePunchFailure::OtherFailure, libc::EIO),
    ] {
        let error = hole_punch_error_the_test_switch_stands_for(failure);
        assert_eq!(
            error.raw_os_error(),
            Some(errno),
            "{failure:?} 该造 errno {errno}"
        );
        assert_eq!(
            classify_hole_punch_failure(&error),
            failure,
            "{failure:?} 造出来的错该分回原路"
        );
    }
}
