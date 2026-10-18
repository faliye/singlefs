//! 块设备抽象：四个动作（读、写——普通写或 FUA 写、写零、屏障），外加三问（探测 `physical_block_size`、设备字节数、
//! journal 环里可能有字节的记录槽）。
//!
//! 第一版两个后端（D17（实现分层与第三方管道） 已定项 5）：文件后端（测试镜像）与 `O_DIRECT` 后端（虚机里的真块设备）。
//! 两个后端的持久语义相同：屏障 = `sync_data`（块设备上是一次 FLUSH），FUA 写 = 写完立刻 `sync_data`。
//!
//! 「写零」这个动作是后加的（用户 2026-09-19 定案，`records/2026-09-19-里程碑二遗留收拢.md` 第五之二节第 4 行：
//! 问「块设备抽象加不加写零」答「加」）：一次调用覆盖 `[offset, offset + length)` 一整段，真设备上对应 WRITE ZEROES。
//! **一次调用就是一个动作**——底下拆成几次 I/O 由后端自己定（[`ZERO_FILL_CHUNK_BYTES`]），上层（录制流、段序列）看到的是一步。
//!
//! 文件后端的写零落成打洞（`fallocate(2)` 的 `FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE`），文件系统报不支持打洞
//! （`EOPNOTSUPP`、`ENOSYS`）才退回逐块写 0，别的错照常报成 [`BlockDeviceError::InputOutput`]（[`ZeroFillLanding`]，
//! 用户 2026-09-30 定「只在不支持时退」）：mkfs 每块盘清 768 MiB 的 journal 环，真写 0 时每池 1.5 GiB 进页缓存又写回盘，
//! 而镜像里非零的只有几 KiB（C603（测试与验证的资源浪费没有自检），用户 2026-09-30 定「稀疏打洞」）。
//! 读回全 0、文件长度不变与真写 0 相同；持久同样由后面那次屏障给：打洞是元数据改动，ext4 的 `ext4_punch_hole`
//! 把它记进 `fdatasync` 要等的那个日志事务（本机 6.17 内核源码 `fs/ext4/inode.c` 里调 `ext4_update_inode_fsync_trans(handle, inode, 1)`）。
//! `O_DIRECT` 后端照旧真写 0：真设备上的块层计数与设备日志比对（checker 档 `device_log.rs` 的折叠）靠的是那些写。

use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use crate::address::DeviceOffsetInBytes;

/// 一次写要不要在返回前做到持久（FUA）。写成枚举，不写成布尔参数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteDurability {
    /// 普通写：进设备队列即可，持久由后面的屏障保证。
    Plain,
    /// FUA 写：返回时已持久（根槽这样写，D16（发布语义） 已定项 7）。
    ForceUnitAccess,
}

/// 挂载时探测到的物理块大小；不许硬编码（D20（承重面：单元的原子性与自包含） 推论三）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalBlockSizeInBytes(pub u32);

/// 块设备能出的错，按调用方要做的决定分成员。
#[derive(Debug)]
pub enum BlockDeviceError {
    /// 请求越过设备末尾：调用方的落点算错了，不重试。
    OutOfRange {
        offset: DeviceOffsetInBytes,
        length: u64,
        device_size: u64,
    },
    /// 请求的偏移或长度不是物理块的整数倍：违反 D2（RAID 条带策略） 硬要求，不重试。
    Unaligned {
        offset: DeviceOffsetInBytes,
        length: u64,
        physical_block_size: u32,
    },
    /// 底层 I/O 错：调用方决定重试还是报坏盘。
    InputOutput(io::Error),
    /// 块设备的 queue 属性读不到（sysfs 没有这块盘，或内容不是数）：不许拿一个写死的值顶上（D20（承重面：单元的原子性与自包含） 推论三）。
    ProbeFailed { attribute: &'static str },
    /// 要建的镜像文件已经在那儿了：不许接着用旧镜像（进程号会被系统复用、上一轮被杀留下的文件不会被清，
    /// 接着用就是拿别人的盘面当自己的起点）。调用方自己挑一个没人用的路径，或先清掉。
    ImageFileAlreadyExists { path: PathBuf },
    /// 要重开的镜像文件不在：重开的语义是「接着用这块盘」，文件没了就不是重开，不许静默建一块空盘。
    ImageFileMissing { path: PathBuf },
}

impl std::fmt::Display for BlockDeviceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockDeviceError::OutOfRange {
                offset,
                length,
                device_size,
            } => {
                write!(
                    formatter,
                    "请求越过设备末尾：偏移 {} 长度 {} 设备 {} 字节",
                    offset.0, length, device_size
                )
            }
            BlockDeviceError::Unaligned {
                offset,
                length,
                physical_block_size,
            } => {
                write!(
                    formatter,
                    "请求没按物理块对齐：偏移 {} 长度 {} 物理块 {}",
                    offset.0, length, physical_block_size
                )
            }
            BlockDeviceError::InputOutput(error) => write!(formatter, "块设备 I/O 错：{error}"),
            BlockDeviceError::ProbeFailed { attribute } => {
                write!(formatter, "读不到块设备的 queue/{attribute}")
            }
            BlockDeviceError::ImageFileAlreadyExists { path } => {
                write!(formatter, "要建的镜像文件已经在那儿了：{}", path.display())
            }
            BlockDeviceError::ImageFileMissing { path } => {
                write!(formatter, "要重开的镜像文件不在：{}", path.display())
            }
        }
    }
}

impl std::error::Error for BlockDeviceError {}

/// 后端把一段写零拆成多大的块交给底层：4 MiB。
///
/// 实测（2026-09-22 本机 NVMe，`/tmp` 的 ext4，对 4 GiB 稀疏文件的 768 MiB 段写零，各 3 轮）：
/// `O_DIRECT` 下 256 KiB 要 233–240 ms、1 MiB 要 146–152 ms、4 MiB 要 135–138 ms、16 MiB 要 133–137 ms、64 MiB 要 134–135 ms；
/// 走页缓存那一侧 1 MiB 到 16 MiB 都是 43–49 ms。**拐点在 4 MiB**：比它小明显变慢（256 KiB 慢 75%），
/// 比它大只再快 1–2 ms 而暂存区从 4 MiB 长到 64 MiB（`O_DIRECT` 那条路每次调用要一块对齐暂存区）。
pub const ZERO_FILL_CHUNK_BYTES: u64 = 4 * 1024 * 1024;

/// 七个方法：四个动作（[`BlockDevice::read_at`]、[`BlockDevice::write_at`]——FUA 是它的 [`WriteDurability`] 参数、不是另一个方法——、
/// [`BlockDevice::write_zeroes_at`]、[`BlockDevice::barrier`]），三个只答问题、不碰介质的（物理块大小、设备字节数、journal 记录槽的提示）。
/// 另有四个带默认实现、只给记着这次挂载读过什么的那一层（`mounted_session::DeviceReadOnceInThisMount`）改写的：读坏之后当场重读、
/// 换上自证过的重读、读成而自证不过的记账丢块、这一刻读盘；默认落回 [`BlockDevice::read_at`] 或什么都不做。
/// 介质是什么这里看不见。
pub trait BlockDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError>;
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError>;
    /// 写零：把 `[offset, offset + length)` 整段变成 0。偏移与长度同样要按物理块对齐、不越过设备末尾。
    ///
    /// 与「拿一个全 0 缓冲反复调 [`BlockDevice::write_at`]」的区别在**它是一个动作**：
    /// mkfs 把 768 MiB 的 journal 环整环清零时，上层只发一次调用、录制流只记一步，
    /// 底下拆成几次 I/O 是后端自己的事（[`ZERO_FILL_CHUNK_BYTES`]）。
    ///
    /// 没有 [`WriteDurability`] 参数，与 [`BlockDevice::write_at`] 不同：清零永远是普通写，
    /// 持久由后面的屏障保证。唯一的调用方是 mkfs 的整环清零，它末尾有屏障；而「返回时已持久」这条语义
    /// 第一版只给根槽（D16（发布语义） 已定项 7），给清零留一个走不到的 FUA 分支，等于留一条没人测的路。
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError>;
    /// 屏障：之前发出的写全部持久之后才返回。
    fn barrier(&mut self) -> Result<(), BlockDeviceError>;
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes;
    fn size_in_bytes(&self) -> u64;
    /// journal 环里可能有字节的记录槽偏移：恢复扫环时只读这些槽；`None` 是不知道，恢复按记录宽逐槽扫全环。
    /// 默认 `None`：文件后端与真设备不记哪些扇区写过。只记写过的扇区的内存盘答得出，答出的集合要罩住环里每个可能过得了
    /// journal magic 的槽——漏一个槽就少读一条记录，恢复落到的那一版与全环扫描不同
    /// （对拍在 `crates/singlefs-harness/tests/sparse_block_device_journal_hint_matches_the_full_ring_scan.rs`）。
    fn journal_record_offsets_hint(
        &self,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
    /// 读坏（读不出或读出来自证不过）之后当场重读的那一读（条款要它再读一次盘：D19（块指针的结构与宽度预算） 已定项 5、
    /// D16（发布语义） 已定项 1「根槽这一次读坏」那一行）。默认就是 [`BlockDevice::read_at`]：盘自己不记读过什么。
    /// 记着这次挂载读过什么的那一层（`mounted_session::DeviceReadOnceInThisMount`）在这里绕过记下的那一份、直接读底下那块盘。
    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_at(offset, buffer)
    }
    /// 当场重读那一读读到、调用方核过自证（根槽自证、单元校验和与位置项相符）的字节：记着这次挂载读过什么的那一层拿它换掉记下的那一份，
    /// 之后再读这几块取的是它。默认什么都不做：盘自己不记读过什么。`offset` 与 `bytes` 是那一次重读的请求，对齐与越界照 [`BlockDevice::read_at`]。
    fn keep_the_self_verified_reread(&self, _offset: DeviceOffsetInBytes, _bytes: &[u8]) {}
    /// 一次读读成了、调用方核出自证不过（系统配置槽的 magic 与整槽校验和、节点大小，单元校验和与位置项不符）：`offset` 起 `length_in_bytes`
    /// 字节是那一次读的请求，对齐与越界照 [`BlockDevice::read_at`]。默认什么都不做：盘自己不记读过什么。
    /// 记着这次挂载读过什么的那一层（`mounted_session::DeviceReadOnceInThisMount`）把它记进读坏账、丢掉记下的那几块，下一次读它们穿到盘上
    /// （岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路一「两样都做」那一行，用户定：也记读坏、坏字节不留）。
    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        _offset: DeviceOffsetInBytes,
        _length_in_bytes: u64,
    ) {
    }
    /// 条款要这一刻读盘的那一读：取号写、轮换写与抬 F 先写系统配置那一步算世代号与覆写哪一槽时读本盘两个系统配置槽
    /// （D22（单元原子性怎么合成） 已定项 16 两槽轮换「覆写旧槽」要的是这一刻盘上哪一槽旧；可写挂载里这一读算进「取号写之前那几次读」，
    /// 代码轮 `research/prompts/m3-b44-code-r3-main-verification.md` 第二节 L1 ②）。默认就是 [`BlockDevice::read_at`]：盘自己不记读过什么；
    /// 只进内存的预演那一层（`mounted_session::DeviceKeepingWritesInMemory`）也走默认，写过的块取内存里那一份、没写过的照它底下那一份的路读。
    /// 记着这次挂载读过什么的那一层（`mounted_session::DeviceReadOnceInThisMount`）在这里不取也不改记下的那一份、直接读底下那块盘。
    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_at(offset, buffer)
    }
}

/// 文件后端的写零怎么落到镜像文件上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZeroFillLanding {
    /// 先打洞；`fallocate` 报不支持打洞（`HolePunchFailure::PunchingUnsupported`）就退回逐块写 0，
    /// 被信号打断（`HolePunchFailure::InterruptedBySignal`）原样重打，
    /// 报别的错（`HolePunchFailure::OtherFailure`）照常交回 [`BlockDeviceError::InputOutput`]，不退、不写 0。
    PunchHoleOrWriteZeroBytesIfUnsupported,
    /// 只供测试的开关（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：不调 `fallocate`，当作它报了 `EOPNOTSUPP`。
    /// 这个 errno 与真失败走同一段按 errno 分路的判定（`classify_hole_punch_failure`），落进退回逐块写 0 那条路。
    ActAsIfPunchingWereUnsupported,
    /// 只供测试的开关：不调 `fallocate`，当作它报了 `EIO`；与真失败走同一段判定，落进照常报错那条路。
    ActAsIfPunchingFailedWithAnInputOutputError,
    /// 只供测试的开关：这一次写零的前 `punches_interrupted_before_the_real_one` 次打洞不调 `fallocate`，每次都当作它报了 `EINTR`；
    /// 与真失败走同一段判定，落进原样重打那条路，之后那一次照 [`ZeroFillLanding::PunchHoleOrWriteZeroBytesIfUnsupported`] 真调 `fallocate`。
    /// 连着被打断几次都照样重打（代码三方 m3-code-r2 J5：只能打断第一次的开关测不出「被打断第二次就报错」）。
    ActAsIfTheFirstPunchesWereInterruptedBySignal {
        punches_interrupted_before_the_real_one: u64,
    },
}

/// 打洞失败之后走哪条路，按 `fallocate` 交回的 errno 分（用户 2026-09-30 定「只在不支持时退」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HolePunchFailure {
    /// 文件系统不支持打洞（`EOPNOTSUPP`，Linux 上与 `ENOTSUP` 同值）或内核没有 `fallocate`（`ENOSYS`）：
    /// 退回逐块写 0，整段照样变成全 0。
    PunchingUnsupported,
    /// 打洞被信号打断（`EINTR`）：`fallocate(2)` 在动文件之前就返回了，原样再打一次洞；不算不支持、不退回逐块写 0，也不报错
    /// （代码三方 m3-code-r1 Z3-b 采）。
    InterruptedBySignal,
    /// 别的错（读写错、`EBADF`、`EINVAL`、`ENOSPC`，以及没有 errno 的）：照常报成块设备 I/O 错，不被退路吞掉。
    OtherFailure,
}

/// 按 errno 判一次打洞失败该退回逐块写 0，还是照常报错。
fn classify_hole_punch_failure(error: &io::Error) -> HolePunchFailure {
    match error.raw_os_error() {
        Some(libc::EOPNOTSUPP | libc::ENOSYS) => HolePunchFailure::PunchingUnsupported,
        Some(libc::EINTR) => HolePunchFailure::InterruptedBySignal,
        Some(_) | None => HolePunchFailure::OtherFailure,
    }
}

/// 文件后端这个句柄上的写零各落成了哪一种（`.claude/rules/fs-design.md`「五条硬要求」第 4 条：分支可观测）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroFillLandingCounts {
    /// 打洞成功、没经 write 系统调用的写零次数。
    pub punched_calls: u64,
    /// 退回逐块写 0 的写零次数。
    pub written_as_zero_bytes_calls: u64,
    /// 打洞被信号打断（`EINTR`）、原样重打的次数（按打洞的次数数，不按写零的次数数）。
    pub punch_attempts_interrupted_by_signal: u64,
}

/// 文件后端：一个镜像文件当一块盘。物理块大小按参数给（本机 NVMe 是 512，D22（单元原子性怎么合成） 已定项 9 那一节现查）。
pub struct FileBackedBlockDevice {
    file: File,
    size_in_bytes: u64,
    physical_block_size: PhysicalBlockSizeInBytes,
    zero_fill_landing: ZeroFillLanding,
    zero_fill_landing_counts: ZeroFillLandingCounts,
}

/// 两个构造入口起步时一样：先打洞，计数从 0 起。
const INITIAL_ZERO_FILL_LANDING: ZeroFillLanding =
    ZeroFillLanding::PunchHoleOrWriteZeroBytesIfUnsupported;
const NO_ZERO_FILL_LANDED_YET: ZeroFillLandingCounts = ZeroFillLandingCounts {
    punched_calls: 0,
    written_as_zero_bytes_calls: 0,
    punch_attempts_interrupted_by_signal: 0,
};

impl FileBackedBlockDevice {
    /// 只供测试：换掉写零的落地方式（[`ZeroFillLanding::ActAsIfPunchingWereUnsupported`] 强制走退路，
    /// [`ZeroFillLanding::ActAsIfPunchingFailedWithAnInputOutputError`] 强制走照常报错）。
    #[must_use]
    pub fn with_test_only_zero_fill_landing(self, landing: ZeroFillLanding) -> Self {
        Self {
            file: self.file,
            size_in_bytes: self.size_in_bytes,
            physical_block_size: self.physical_block_size,
            zero_fill_landing: landing,
            zero_fill_landing_counts: self.zero_fill_landing_counts,
        }
    }

    #[must_use]
    pub fn zero_fill_landing_counts(&self) -> ZeroFillLandingCounts {
        self.zero_fill_landing_counts
    }

    /// 排他地建出一个镜像文件并把它撑到 `size_in_bytes`；稀疏文件，不真占盘。
    /// 同名文件已经在那儿了就报 [`BlockDeviceError::ImageFileAlreadyExists`]，不接着用旧镜像：
    /// 测试的镜像名带进程号，而进程号会被系统复用、上一轮被杀留下的文件不会被清——接着用就是拿上一轮的盘面当这一轮的起点，
    /// 而且是静默的（代码三方 m2-supp3-item3-code-r1 判决第四节第 9 题，用户定案）。
    pub fn create_image_file_exclusively(
        path: &Path,
        size_in_bytes: u64,
        physical_block_size: PhysicalBlockSizeInBytes,
    ) -> Result<Self, BlockDeviceError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| {
                if error.kind() == io::ErrorKind::AlreadyExists {
                    BlockDeviceError::ImageFileAlreadyExists {
                        path: path.to_path_buf(),
                    }
                } else {
                    BlockDeviceError::InputOutput(error)
                }
            })?;
        file.set_len(size_in_bytes)
            .map_err(BlockDeviceError::InputOutput)?;
        Ok(Self {
            file,
            size_in_bytes,
            physical_block_size,
            zero_fill_landing: INITIAL_ZERO_FILL_LANDING,
            zero_fill_landing_counts: NO_ZERO_FILL_LANDED_YET,
        })
    }

    /// 重开一个已经在的镜像文件（关掉会话再挂回同一块盘、冷启动读回）：文件不在就报
    /// [`BlockDeviceError::ImageFileMissing`]，不静默建一块空盘。
    pub fn open_existing_image_file(
        path: &Path,
        size_in_bytes: u64,
        physical_block_size: PhysicalBlockSizeInBytes,
    ) -> Result<Self, BlockDeviceError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .truncate(false)
            .open(path)
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    BlockDeviceError::ImageFileMissing {
                        path: path.to_path_buf(),
                    }
                } else {
                    BlockDeviceError::InputOutput(error)
                }
            })?;
        file.set_len(size_in_bytes)
            .map_err(BlockDeviceError::InputOutput)?;
        Ok(Self {
            file,
            size_in_bytes,
            physical_block_size,
            zero_fill_landing: INITIAL_ZERO_FILL_LANDING,
            zero_fill_landing_counts: NO_ZERO_FILL_LANDED_YET,
        })
    }

    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        check_aligned_and_in_range(offset, length, self.physical_block_size, self.size_in_bytes)
    }
}

/// 请求要按物理块对齐、不越过设备末尾；两个后端与预演用的只进内存的盘（`mounted_session` 的 `DeviceKeepingWritesInMemory`：
/// 写不交给底下那块盘，不判就收得下一个真盘会拒的写）共用这一份判据。
pub(crate) fn check_aligned_and_in_range(
    offset: DeviceOffsetInBytes,
    length: u64,
    physical_block_size: PhysicalBlockSizeInBytes,
    device_size: u64,
) -> Result<(), BlockDeviceError> {
    let block = u64::from(physical_block_size.0);
    if !offset.0.is_multiple_of(block) || !length.is_multiple_of(block) {
        return Err(BlockDeviceError::Unaligned {
            offset,
            length,
            physical_block_size: physical_block_size.0,
        });
    }
    match offset.0.checked_add(length) {
        Some(end) if end <= device_size => Ok(()),
        Some(_) | None => Err(BlockDeviceError::OutOfRange {
            offset,
            length,
            device_size,
        }),
    }
}

fn length_of(buffer: &[u8]) -> u64 {
    u64::try_from(buffer.len()).expect("缓冲区长度装得进 u64")
}

/// 一次写零要准备多大的全 0 暂存区：整段与 [`ZERO_FILL_CHUNK_BYTES`] 取小的那个。
/// 清一段短于一个块的范围时不白白分配 4 MiB。
fn zero_fill_chunk_length(length: u64) -> usize {
    usize::try_from(length.min(ZERO_FILL_CHUNK_BYTES)).expect("块长不超过 4 MiB，装得进 usize")
}

/// 两个后端共用的拆块循环：`zeros` 是后端备好的全 0 暂存区（它的长度就是块大小），
/// 从 `offset` 起按块顺序交给 `write_chunk`，最后一块按剩余长度截短。
///
/// 长度按物理块对齐由调用方先核过（[`check_aligned_and_in_range`]）；每一块的起点都还对齐，前提是物理块大小整除块大小 4 MiB
/// （512、4096 都整除）。这个前提这里不判，物理块大小的来处也都不判它：[`PhysicalBlockSizeSource::Declared`] 收调用方给的任何值，
/// sysfs 读到的值也不核；不整除时第二块起的起点不对齐。
fn write_zeroes_in_chunks<WriteChunk>(
    offset: DeviceOffsetInBytes,
    length: u64,
    zeros: &[u8],
    mut write_chunk: WriteChunk,
) -> Result<(), BlockDeviceError>
where
    WriteChunk: FnMut(DeviceOffsetInBytes, &[u8]) -> Result<(), BlockDeviceError>,
{
    let mut written: u64 = 0;
    while written < length {
        let this_chunk = (length - written).min(length_of(zeros));
        let end = usize::try_from(this_chunk).expect("块长装得进 usize");
        write_chunk(DeviceOffsetInBytes(offset.0 + written), &zeros[..end])?;
        written += this_chunk;
    }
    Ok(())
}

/// 在 `file` 的 `[offset, offset + length)` 上打洞、不改文件长度：之后读这一段是全 0，
/// 整块的文件系统块被释放，头尾不满一块的部分由内核就地写 0（`fallocate(2)`「Deallocating file space」）。
/// 返回非 0 时交回 `errno`，由调用方按 `classify_hole_punch_failure` 决定退回逐块写 0、原样重打还是照常报错。
#[allow(
    unsafe_code,
    reason = "std 没有 fallocate 的安全封装；全 crate 只在这一处放开 unsafe（C603（测试与验证的资源浪费没有自检））"
)]
fn punch_hole_keeping_the_file_length(
    file: &File,
    offset: DeviceOffsetInBytes,
    length: u64,
) -> io::Result<()> {
    let offset_for_the_system_call = libc::off_t::try_from(offset.0)
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    let length_for_the_system_call =
        libc::off_t::try_from(length).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    // SAFETY: `fallocate` 只读它的四个整数参数，不碰进程内存；描述符取自 `file`，
    // 而 `file` 借用到这次调用返回，调用期间不会被关掉或换成别的文件。偏移与长度由上面的 `try_from`
    // 转成非负的 `off_t`，放不下就在转换处拒；越界的请求到不了这里：唯一的调用方 `write_zeroes_at` 先过 `check_request`
    // （对齐、落在设备范围内），长度 0 也在 `write_zeroes_at` 里先交回、不到这里。
    // 跨过文件末尾的打洞，文件内那一半照常打成 0，文件外的部分内核不做，都不碰内存。返回值在下面查了。
    let return_code = unsafe {
        libc::fallocate(
            file.as_raw_fd(),
            libc::FALLOC_FL_PUNCH_HOLE | libc::FALLOC_FL_KEEP_SIZE,
            offset_for_the_system_call,
            length_for_the_system_call,
        )
    };
    if return_code == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

impl BlockDevice for FileBackedBlockDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length_of(buffer))?;
        self.file
            .read_exact_at(buffer, offset.0)
            .map_err(BlockDeviceError::InputOutput)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length_of(bytes))?;
        self.file
            .write_all_at(bytes, offset.0)
            .map_err(BlockDeviceError::InputOutput)?;
        match durability {
            WriteDurability::Plain => Ok(()),
            WriteDurability::ForceUnitAccess => {
                self.file.sync_data().map_err(BlockDeviceError::InputOutput)
            }
        }
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        // 长度 0 不调系统调用：`fallocate(2)` 对长度 0 报 `EINVAL`，而另两个后端（`O_DIRECT` 后端、内存稀疏盘）对它交回 `Ok`、
        // 一个字节不动（代码三方 m3-code-r1 Z3-a）。同一个请求三个后端结局相同，这里照它们交回 `Ok`，不记进落地计数。
        if length == 0 {
            return Ok(());
        }
        // 循环：每一轮打一次洞。跨轮的状态只有这一次写零里被信号打断的次数（`punch_attempts_interrupted`，只增）；
        // 出口三个：打成洞、报不支持（退回逐块写 0）、报别的错。被打断就原样再打，轮数的上界是这期间送到的信号数
        // （与 std 的 `write_all_at` 对 `ErrorKind::Interrupted` 的处理同一个口径）。
        let mut punch_attempts_interrupted: u64 = 0;
        loop {
            let punched = match self.zero_fill_landing {
                ZeroFillLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal {
                    punches_interrupted_before_the_real_one,
                } if punch_attempts_interrupted < punches_interrupted_before_the_real_one => {
                    Err(io::Error::from_raw_os_error(libc::EINTR))
                }
                ZeroFillLanding::PunchHoleOrWriteZeroBytesIfUnsupported
                | ZeroFillLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal { .. } => {
                    punch_hole_keeping_the_file_length(&self.file, offset, length)
                }
                ZeroFillLanding::ActAsIfPunchingWereUnsupported => {
                    Err(io::Error::from_raw_os_error(libc::EOPNOTSUPP))
                }
                ZeroFillLanding::ActAsIfPunchingFailedWithAnInputOutputError => {
                    Err(io::Error::from_raw_os_error(libc::EIO))
                }
            };
            let punch_error = match punched {
                Ok(()) => {
                    self.zero_fill_landing_counts.punched_calls += 1;
                    return Ok(());
                }
                Err(punch_error) => punch_error,
            };
            match classify_hole_punch_failure(&punch_error) {
                HolePunchFailure::PunchingUnsupported => {
                    self.zero_fill_landing_counts.written_as_zero_bytes_calls += 1;
                    let zeros = vec![0u8; zero_fill_chunk_length(length)];
                    return write_zeroes_in_chunks(
                        offset,
                        length,
                        &zeros,
                        |chunk_offset, chunk| {
                            self.file
                                .write_all_at(chunk, chunk_offset.0)
                                .map_err(BlockDeviceError::InputOutput)
                        },
                    );
                }
                HolePunchFailure::InterruptedBySignal => {
                    punch_attempts_interrupted += 1;
                    self.zero_fill_landing_counts
                        .punch_attempts_interrupted_by_signal += 1;
                }
                HolePunchFailure::OtherFailure => {
                    return Err(BlockDeviceError::InputOutput(punch_error));
                }
            }
        }
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.file.sync_data().map_err(BlockDeviceError::InputOutput)
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }
}

/// Linux open(2) 的 `O_DIRECT` 位；取值随架构不同，只接本机与虚机的两种，别的架构编不过。
#[cfg(target_arch = "x86_64")]
const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名，取自 <fcntl.h>，名字不归我们定
#[cfg(target_arch = "aarch64")]
const O_DIRECT: i32 = 0o200000; // naming-lint:external Linux open(2) 标志名，取自 <fcntl.h>，名字不归我们定

/// 读写走不走页缓存。真设备上一律绕开（块层计数才与程序数的对得上）；走页缓存的那一臂只给阳性对照用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageCachePolicy {
    BypassWithDirectInputOutput,
    GoThroughPageCache,
}

/// 物理块大小从哪来：块设备探 sysfs（不许硬编码，D20（承重面：单元的原子性与自包含） 推论三）；普通文件没有 sysfs，由调用方声明。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicalBlockSizeSource {
    ProbeSysfs,
    Declared(PhysicalBlockSizeInBytes),
}

/// sysfs 里块设备的 queue 属性，原样文本（例如 `write_cache` 是 `write back` / `write through`）；读不到返回 None（读不到 ≠ 读到 0）。
#[must_use]
pub fn probe_queue_text_under(
    sysfs_class_block: &Path,
    device_path: &Path,
    attribute: &str,
) -> Option<String> {
    let device_name = device_path.file_name()?.to_str()?;
    let text = std::fs::read_to_string(
        sysfs_class_block
            .join(device_name)
            .join("queue")
            .join(attribute),
    )
    .ok()?;
    Some(text.trim().to_string())
}

/// 数值型的 queue 属性（`physical_block_size`、`logical_block_size`、`minimum_io_size`）。
#[must_use]
pub fn probe_queue_number_under(
    sysfs_class_block: &Path,
    device_path: &Path,
    attribute: &str,
) -> Option<u32> {
    probe_queue_text_under(sysfs_class_block, device_path, attribute)?
        .parse()
        .ok()
}

/// 本机的 sysfs 根。
pub const SYSFS_CLASS_BLOCK: &str = "/sys/class/block";

/// `O_DIRECT` 要求用户缓冲按逻辑块对齐；按 4096 对齐同时罩住 512 与 4096 两档。
const DIRECT_BUFFER_ALIGNMENT: usize = 4096;

/// 一块按 4096 对齐的暂存区：在多分配 4096 字节的 Vec 里用 `align_offset` 找对齐起点，不写 unsafe。
struct AlignedScratch {
    storage: Vec<u8>,
    start: usize,
    length: usize,
}

impl AlignedScratch {
    fn new(length: usize) -> Self {
        let storage = vec![0u8; length + DIRECT_BUFFER_ALIGNMENT];
        let start = storage.as_ptr().align_offset(DIRECT_BUFFER_ALIGNMENT);
        assert!(
            start < DIRECT_BUFFER_ALIGNMENT,
            "运行期 align_offset 对 u8 指针总给得出偏移；给不出说明这个前提被打破了"
        );
        Self {
            storage,
            start,
            length,
        }
    }
    fn as_slice(&self) -> &[u8] {
        &self.storage[self.start..self.start + self.length]
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.storage[self.start..self.start + self.length]
    }
}

/// `O_DIRECT` 后端：打开一个已经存在的块设备（或普通文件），大小取设备末尾的偏移，不改大小。
pub struct DirectInputOutputBlockDevice {
    file: File,
    size_in_bytes: u64,
    physical_block_size: PhysicalBlockSizeInBytes,
    policy: PageCachePolicy,
}

impl DirectInputOutputBlockDevice {
    pub fn open_existing(
        path: &Path,
        policy: PageCachePolicy,
        block_size_source: PhysicalBlockSizeSource,
    ) -> Result<Self, BlockDeviceError> {
        let mut options = OpenOptions::new();
        options.read(true).write(true);
        match policy {
            PageCachePolicy::BypassWithDirectInputOutput => {
                options.custom_flags(O_DIRECT);
            }
            PageCachePolicy::GoThroughPageCache => {}
        }
        let mut file = options.open(path).map_err(BlockDeviceError::InputOutput)?;
        let size_in_bytes = file
            .seek(SeekFrom::End(0))
            .map_err(BlockDeviceError::InputOutput)?;
        let physical_block_size = match block_size_source {
            PhysicalBlockSizeSource::ProbeSysfs => PhysicalBlockSizeInBytes(
                probe_queue_number_under(Path::new(SYSFS_CLASS_BLOCK), path, "physical_block_size")
                    .ok_or(BlockDeviceError::ProbeFailed {
                        attribute: "physical_block_size",
                    })?,
            ),
            PhysicalBlockSizeSource::Declared(declared) => declared,
        };
        Ok(Self {
            file,
            size_in_bytes,
            physical_block_size,
            policy,
        })
    }
}

impl BlockDevice for DirectInputOutputBlockDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        check_aligned_and_in_range(
            offset,
            length_of(buffer),
            self.physical_block_size,
            self.size_in_bytes,
        )?;
        match self.policy {
            PageCachePolicy::BypassWithDirectInputOutput => {
                let mut scratch = AlignedScratch::new(buffer.len());
                self.file
                    .read_exact_at(scratch.as_mut_slice(), offset.0)
                    .map_err(BlockDeviceError::InputOutput)?;
                buffer.copy_from_slice(scratch.as_slice());
                Ok(())
            }
            PageCachePolicy::GoThroughPageCache => self
                .file
                .read_exact_at(buffer, offset.0)
                .map_err(BlockDeviceError::InputOutput),
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        check_aligned_and_in_range(
            offset,
            length_of(bytes),
            self.physical_block_size,
            self.size_in_bytes,
        )?;
        match self.policy {
            PageCachePolicy::BypassWithDirectInputOutput => {
                let mut scratch = AlignedScratch::new(bytes.len());
                scratch.as_mut_slice().copy_from_slice(bytes);
                self.file
                    .write_all_at(scratch.as_slice(), offset.0)
                    .map_err(BlockDeviceError::InputOutput)?;
            }
            PageCachePolicy::GoThroughPageCache => {
                self.file
                    .write_all_at(bytes, offset.0)
                    .map_err(BlockDeviceError::InputOutput)?;
            }
        }
        match durability {
            WriteDurability::Plain => Ok(()),
            WriteDurability::ForceUnitAccess => {
                self.file.sync_data().map_err(BlockDeviceError::InputOutput)
            }
        }
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        check_aligned_and_in_range(offset, length, self.physical_block_size, self.size_in_bytes)?;
        // `O_DIRECT` 要求用户缓冲对齐，所以全 0 暂存区走 `AlignedScratch`（它自己就是全 0 的）；
        // 走页缓存那一臂不需要对齐，但用同一块暂存区，两臂发出的 pwrite 完全一样。
        let scratch = AlignedScratch::new(zero_fill_chunk_length(length));
        write_zeroes_in_chunks(offset, length, scratch.as_slice(), |chunk_offset, chunk| {
            self.file
                .write_all_at(chunk, chunk_offset.0)
                .map_err(BlockDeviceError::InputOutput)
        })
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.file.sync_data().map_err(BlockDeviceError::InputOutput)
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// 测试镜像一律放临时目录（`command-safety.md`），文件名带进程号与计数器免得并行测试互相盖。
    fn temporary_image_path(tag: &str) -> std::path::PathBuf {
        let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "singlefs-core-{tag}-{}-{sequence}.img",
            std::process::id()
        ))
    }

    /// 镜像的删除守卫：用例做完、断言失败、panic 都在离开作用域时删掉镜像（`command-safety.md`「测试镜像一律放临时目录」）。
    /// 删不掉只打一行，不在 Drop 里 panic（panic 途中再 panic 会直接中止进程）。
    struct ImageRemovedOnDrop(std::path::PathBuf);

    impl Drop for ImageRemovedOnDrop {
        fn drop(&mut self) {
            if let Err(error) = std::fs::remove_file(&self.0) {
                if error.kind() != io::ErrorKind::NotFound {
                    eprintln!("删不掉测试镜像 {}：{error}", self.0.display());
                }
            }
        }
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
        std::fs::remove_file(&path).expect("清理镜像");
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
        std::fs::remove_file(&path).expect("清理镜像");
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
        std::fs::remove_file(&path).expect("清理镜像");
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
        std::fs::remove_file(&path).expect("清理镜像");
    }

    /// 打洞报别的错（测试开关当作 `fallocate` 报了 `EIO`）：照常交回 `InputOutput`、原 errno 带着，
    /// 一个 0 都不写——段里还是旧字节，经 write 系统调用一个字节都没交出去，两个落地计数都不动。
    #[test]
    fn a_hole_punch_failing_with_an_input_output_error_is_reported_and_writes_no_zero_bytes() {
        const IMAGE_BYTES: u64 = 65536;
        const DIRT: u8 = 0xC3;
        let path = temporary_image_path("failing-punch");
        let mut device = FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            IMAGE_BYTES,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像")
        .with_test_only_zero_fill_landing(
            ZeroFillLanding::ActAsIfPunchingFailedWithAnInputOutputError,
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
        std::fs::remove_file(&path).expect("清理镜像");
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
        let _image_removed_on_drop = ImageRemovedOnDrop(path.clone());
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
        let zero_fill_outcome =
            device.write_zeroes_at(DeviceOffsetInBytes(ZERO_START), ZERO_LENGTH);
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
    fn a_hole_punch_interrupted_by_a_signal_is_punched_again_without_falling_back_or_reporting_an_error(
    ) {
        assert_punched_again_after_interruptions_without_falling_back_or_reporting_an_error(1);
    }

    /// 打洞连着被信号打断两次（代码三方 m3-code-r2 J5）：第二次被打断也照样原样重打、第三次真打成洞，不报错、不退回逐块写 0。
    /// 「被打断第二次就报错」（重打只给一次）时这里交回 `InputOutput`。
    #[test]
    fn a_hole_punch_interrupted_by_a_signal_twice_in_a_row_is_punched_a_third_time_without_falling_back_or_reporting_an_error(
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

    /// 长度 0 的写零（代码三方 m3-code-r1 Z3-a）：文件后端与 `O_DIRECT` 后端（绕开页缓存、走页缓存两臂）在偏移 0、中间、
    /// 设备末尾三处结局逐项相同——交回 `Ok`、经 write 系统调用一个字节都没交出去、镜像一个字节不动。文件后端另核落地计数一样都不记
    /// （长度 0 不调 `fallocate`；调了内核报 `EINVAL`，交回的就是 `InputOutput`）。`O_DIRECT` 后端没有落地计数，比到「交回什么、
    /// 交出去几个字节、镜像变没变」这三步为止。越过末尾一个物理块、没按物理块对齐的两处长度 0（代码三方 m3-code-r2 J4），
    /// 三个后端照样先过请求判据、都拒成 `OutOfRange`、`Unaligned`，一个字节不动（长度 0 的早退挪到请求判据之前时文件后端交 `Ok`）。
    /// 内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`）在 harness 档，核心层的单测拿不到它，不在这里比
    /// （它与文件后端对这两处长度 0 的请求逐个对拍在 `sparse_devices_refuse_requests_outside_the_device_like_the_file_backend`）。
    #[test]
    fn a_zero_length_zero_fill_ends_the_same_on_the_file_backend_and_both_arms_of_the_direct_backend(
    ) {
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
        let _file_image_removed_on_drop = ImageRemovedOnDrop(file_path.clone());
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
            let _direct_image_removed_on_drop = ImageRemovedOnDrop(direct_path.clone());
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
        std::fs::remove_file(&path).expect("清理镜像");
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
        let reopened = FileBackedBlockDevice::open_existing_image_file(
            &path,
            8192,
            PhysicalBlockSizeInBytes(512),
        )
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
        std::fs::remove_file(&path).expect("清理镜像");
        let missing = FileBackedBlockDevice::open_existing_image_file(
            &path,
            8192,
            PhysicalBlockSizeInBytes(512),
        );
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
        std::fs::remove_file(&path).expect("清理镜像");
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
        std::fs::remove_file(&path).expect("清理镜像");
    }

    #[test]
    fn queue_attributes_are_read_from_the_sysfs_tree_and_missing_ones_are_none() {
        let root =
            std::env::temp_dir().join(format!("singlefs-core-fake-sysfs-{}", std::process::id()));
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
        std::fs::remove_dir_all(&root).expect("清理假 sysfs");
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
}
