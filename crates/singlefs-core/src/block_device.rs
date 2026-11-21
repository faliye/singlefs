//! 块设备抽象：五个动作（读、写——普通写或 FUA 写、写零、discard、屏障），外加三问（探测 `physical_block_size`、设备字节数、
//! journal 环里可能有字节的记录槽）。
//!
//! 第一版两个后端（D17（实现分层与第三方管道） 已定项 5）：文件后端（测试镜像）与 `O_DIRECT` 后端（虚机里的真块设备）。
//! 两个后端的持久语义相同：屏障 = `sync_data`（块设备上是一次 FLUSH），FUA 写 = 写完立刻 `sync_data`。
//!
//! 「写零」这个动作是后加的（`records/2026-09-19-里程碑二遗留收拢.md` 第五之二节第 4 行：
//! 问「块设备抽象加不加写零」答「加」）：一次调用覆盖 `[offset, offset + length)` 一整段，真设备上对应 WRITE ZEROES。
//! **一次调用就是一个动作**——底下拆成几次 I/O 由后端自己定（[`ZERO_FILL_CHUNK_BYTES`]），上层（录制流、段序列）看到的是一步。
//!
//! 文件后端的写零落成打洞（`fallocate(2)` 的 `FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE`），文件系统报不支持打洞
//! （`EOPNOTSUPP`、`ENOSYS`）才退回逐块写 0，别的错照常报成 [`BlockDeviceError::InputOutput`]（[`ZeroFillLanding`]）：
//! mkfs 每块盘清 768 MiB 的 journal 环，真写 0 时每池 1.5 GiB 进页缓存又写回盘，
//! 而镜像里非零的只有几 KiB（C603（测试与验证的资源浪费没有自检））。
//! 读回全 0、文件长度不变与真写 0 相同；持久同样由后面那次屏障给：打洞是元数据改动，ext4 的 `ext4_punch_hole`
//! 把它记进 `fdatasync` 要等的那个日志事务（本机 6.17 内核源码 `fs/ext4/inode.c` 里调 `ext4_update_inode_fsync_trans(handle, inode, 1)`）。
//! `O_DIRECT` 后端照旧真写 0：真设备上的块层计数与设备日志比对（`device_log.rs` 的折叠）靠的是那些写。
//!
//! discard 是第七个动作（`research/prompts/m4-gc-r1-main-verification.md` 第三节 G5「trait 形态」；D26（后台整理与放置回收）
//! 未定项 9「还缺一个前提」：盘的 FTL 只有收到 discard 才知道哪些块不用了）。它与写零的差别在**结局由设备定**：
//! 持久之后那一段读回 0 还是旧字节由设备定，模型（崩溃点重放）按「范围写零」枚举两态（D13（验证路线） 已定项 4：持久 → 0、没持久 → 旧字节）。
//! 后端不认 discard 时交回 [`DiscardOutcome::Unsupported`]、一个字节不动；文件后端复用打洞，`O_DIRECT` 后端第一版不认（`BLKDISCARD` ioctl 留到真盘线）。
//! 谁发、什么时候发（分配器的可发谓词、发布返回之后发）不在这一层。
//!
//! 类型、[`BlockDevice`] 特质与两个后端的特质实现都在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」的块设备信任边界一簇，
//! 被验区域登记在 `.claude/gate.d/verus-pins.tsv`，必红世界在 `crates/singlefs-core/verus-must-red-worlds.tsv`）。
//! 特质带一份 ghost 操作日志 [`BlockDevice::issued_operations`]：写、写零、交给了底层的 discard 与屏障各记一条，按发出的次序，
//! 供发布装配证写序（复制、屏障、journal、屏障、根槽 FUA）。日志记的是 core 发出了什么，
//! 设备守不守序、写到一半断电落成什么不在这里（崩溃点重放与真设备那一侧管）。
//! 真碰盘的几步（`pread` / `pwrite`、`fdatasync`、打洞的 `fallocate`、对齐暂存区的指针对齐、按 errno 分路、造带 errno 的 I/O 错）与
//! `Display` 是 `external_body`，三个 std 类型（`io::Error`、`PathBuf`、`File`）是不透明的类型规格，逐条登记在信任清单
//! `.claude/gate.d/verus-trust-list.tsv`，每条配一条差分测试（本文件的 `tests` 模块）；请求怎么判、写零怎么拆块、打洞失败之后走哪条路、
//! 计数与日志怎么记由 Verus 验。
//! 两个句柄建出来的那几步（开文件、探 sysfs、只供测试的开关）不在 `verus!` 里：Verus 不验它们，句柄上的日志从它们交回时起记。
//! 计数一律 `saturating_add`：数到 `u64::MAX` 停住（一个句柄上发 2^64 次走不到），Verus 才证得出加一不溢出。

use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use vstd::prelude::*;

use crate::address::DeviceOffsetInBytes;

verus! {

// Verus 不认 std 里没给规格的类型。下面三条只说「有这个类型」、不说里面是什么（Verus 把它当不透明的），登记进信任清单；
// 差分测试是真用到这个类型的那几条后端测试。
#[verifier::external_type_specification]
#[verifier::external_body]
#[allow(dead_code, reason = "只给 Verus 看的类型规格：产品构建里没有人读它包着的字段")]
pub struct ExternalTypeSpecificationOfInputOutputError(io::Error);

#[verifier::external_type_specification]
#[verifier::external_body]
#[allow(dead_code, reason = "只给 Verus 看的类型规格：产品构建里没有人读它包着的字段")]
pub struct ExternalTypeSpecificationOfPathBuffer(PathBuf);

#[verifier::external_type_specification]
#[verifier::external_body]
#[allow(dead_code, reason = "只给 Verus 看的类型规格：产品构建里没有人读它包着的字段")]
pub struct ExternalTypeSpecificationOfFile(File);

/// 一次写要不要在返回前做到持久（FUA）。写成枚举，不写成布尔参数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteDurability {
    /// 普通写：进设备队列即可，持久由后面的屏障保证。
    Plain,
    /// FUA 写：返回时已持久（根槽这样写，D16（发布语义） 已定项 7）。
    ForceUnitAccess,
}

/// 一次 [`BlockDevice::discard_at`] 在这块盘上落成了什么（`.claude/rules/fs-design.md`「五条硬要求」第 4 条：分支可观测，调用方按它数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscardOutcome {
    /// 后端把这一段交给了底层：之后读这一段可能是 0、也可能还是旧字节，持久之后是哪一个由设备定。
    Applied,
    /// 后端不认 discard：这一段一个字节没动，读回仍是旧字节。不是错——discard 只是告诉盘「这一段不用了」。
    Unsupported,
}

/// 一个句柄上的 discard 各交回了哪一种结局：两项之和就是这个句柄上交回 `Ok` 的 discard 次数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiscardOutcomeCounts {
    pub applied_calls: u64,
    pub unsupported_calls: u64,
}

/// 计数加一、到 `u64::MAX` 停住（`saturating_add` 的值）。
pub open spec fn count_after_one_more(count: u64) -> u64 {
    if count == u64::MAX {
        u64::MAX
    } else {
        (count + 1) as u64
    }
}

impl DiscardOutcomeCounts {
    /// 一次也没交回过。
    pub const NONE: DiscardOutcomeCounts = DiscardOutcomeCounts {
        applied_calls: 0,
        unsupported_calls: 0,
    };

    /// 记下一次交回的结局。
    pub fn note(&mut self, outcome: DiscardOutcome)
        ensures
            final(self).applied_calls == if outcome == DiscardOutcome::Applied {
                count_after_one_more(old(self).applied_calls)
            } else {
                old(self).applied_calls
            },
            final(self).unsupported_calls == if outcome == DiscardOutcome::Unsupported {
                count_after_one_more(old(self).unsupported_calls)
            } else {
                old(self).unsupported_calls
            },
    {
        match outcome {
            DiscardOutcome::Applied => self.applied_calls = self.applied_calls.saturating_add(1),
            DiscardOutcome::Unsupported => {
                self.unsupported_calls = self.unsupported_calls.saturating_add(1)
            }
        }
    }
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
    // 格式化宏 Verus 不认：可信，登记进信任清单，差分测试逐个成员核写出来的字里带着它的字段。
    #[verifier::external_body]
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
/// 实测（本机 NVMe，`/tmp` 的 ext4，对 4 GiB 稀疏文件的 768 MiB 段写零，各 3 轮）：
/// `O_DIRECT` 下 256 KiB 要 233–240 ms、1 MiB 要 146–152 ms、4 MiB 要 135–138 ms、16 MiB 要 133–137 ms、64 MiB 要 134–135 ms；
/// 走页缓存那一侧 1 MiB 到 16 MiB 都是 43–49 ms。**拐点在 4 MiB**：比它小明显变慢（256 KiB 慢 75%），
/// 比它大只再快 1–2 ms 而暂存区从 4 MiB 长到 64 MiB（`O_DIRECT` 那条路每次调用要一块对齐暂存区）。
pub const ZERO_FILL_CHUNK_BYTES: u64 = 4 * 1024 * 1024;

/// 一个句柄上发出、交给了设备的一个改介质的动作（ghost：只在 Verus 里有，产品构建里没有）。
/// 写带写下去的字节与持久要求；写零与 discard 带范围；屏障什么都不带。读不改介质，不记。
pub ghost enum IssuedDeviceOperation {
    Write { offset: DeviceOffsetInBytes, bytes: Seq<u8>, durability: WriteDurability },
    WriteZeroes { offset: DeviceOffsetInBytes, length: u64 },
    Discard { offset: DeviceOffsetInBytes, length: u64 },
    Barrier,
}

/// `value` 是物理块的整数倍（物理块是 0 时只有 0 算：与 `u64::is_multiple_of` 对 0 的取法相同）。
pub open spec fn is_a_whole_number_of_physical_blocks(value: int, physical_block_size_in_bytes: u32) -> bool {
    if physical_block_size_in_bytes == 0 {
        value == 0
    } else {
        value % (physical_block_size_in_bytes as int) == 0
    }
}

/// 一个请求后端收不收：偏移与长度按物理块对齐、不越过设备末尾（[`check_aligned_and_in_range`] 判的就是它）。
pub open spec fn request_is_aligned_and_in_range(
    offset: DeviceOffsetInBytes,
    length: int,
    physical_block_size_in_bytes: u32,
    device_size_in_bytes: u64,
) -> bool {
    &&& 0 <= length
    &&& is_a_whole_number_of_physical_blocks(offset.0 as int, physical_block_size_in_bytes)
    &&& is_a_whole_number_of_physical_blocks(length, physical_block_size_in_bytes)
    &&& offset.0 + length <= device_size_in_bytes
}

/// 日志里的一条写（特质的声明里不写结构体字面量：门禁按声明里有没有 `{` 认它有没有函数体）。
pub open spec fn issued_write_operation(
    offset: DeviceOffsetInBytes,
    bytes: Seq<u8>,
    durability: WriteDurability,
) -> IssuedDeviceOperation {
    IssuedDeviceOperation::Write { offset, bytes, durability }
}

/// 日志里的一条写零（同上）。
pub open spec fn issued_write_zeroes_operation(offset: DeviceOffsetInBytes, length: u64) -> IssuedDeviceOperation {
    IssuedDeviceOperation::WriteZeroes { offset, length }
}

/// 一次改介质的调用之后日志的样子：交回 `Ok` 的，日志恰好多出 `operation` 这一条；交回 `Err` 的，没发出去（日志不变）或发出去了（多这一条）都可能。
pub open spec fn issued_operations_after_the_call(
    before: Seq<IssuedDeviceOperation>,
    after: Seq<IssuedDeviceOperation>,
    operation: IssuedDeviceOperation,
    call_succeeded: bool,
) -> bool {
    if call_succeeded {
        after == before.push(operation)
    } else {
        after == before || after == before.push(operation)
    }
}

/// 八个方法：五个动作（[`BlockDevice::read_at`]、[`BlockDevice::write_at`]——FUA 是它的 [`WriteDurability`] 参数、不是另一个方法——、
/// [`BlockDevice::write_zeroes_at`]、[`BlockDevice::discard_at`]、[`BlockDevice::barrier`]），三个只答问题、不碰介质的（物理块大小、设备字节数、journal 记录槽的提示）。
/// discard 持久之后那一段读回 0 还是旧字节由设备定（不像写零那样必定是 0），模型按「范围写零」枚举（D13（验证路线） 已定项 4 的两态：持久 → 0、没持久 → 旧字节）；
/// 它带默认实现，只给不认 discard 的后端（交回不支持）。
/// 另有四个带默认实现、只给记着这次挂载读过什么的那一层（`mounted_session::DeviceReadOnceInThisMount`）改写的：读坏之后当场重读、
/// 换上自证过的重读、读成而自证不过的记账丢块、这一刻读盘；默认落回 [`BlockDevice::read_at`] 或什么都不做。
/// 介质是什么这里看不见。
///
/// 三个 spec fn（日志、物理块大小、设备字节数）是 Verus 看的抽象状态：`verus!` 里的实现各自给出，契约照它们写。
/// 它们的默认体只给 `verus!` 外面的实现编得过（Verus 编的时候特质里有这三项，外面的实现不写它们）：外面的实现 Verus 不验，
/// 默认体说的不是它们的真样子；`verus!` 里的实现一律自己给出这三项。
pub trait BlockDevice {
    /// 这个句柄上发出过、交给了设备的改介质动作，按发出的次序。
    open spec fn issued_operations(&self) -> Seq<IssuedDeviceOperation> {
        Seq::empty()
    }

    /// [`BlockDevice::probe_physical_block_size`] 答的那个数。
    open spec fn physical_block_size_of_the_device(&self) -> u32 {
        0
    }

    /// [`BlockDevice::size_in_bytes`] 答的那个数。
    open spec fn size_of_the_device_in_bytes(&self) -> u64 {
        0
    }

    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> (result: Result<(), BlockDeviceError>)
        ensures
            final(buffer)@.len() == old(buffer)@.len(),
            result is Ok ==> request_is_aligned_and_in_range(
                offset,
                old(buffer)@.len() as int,
                self.physical_block_size_of_the_device(),
                self.size_of_the_device_in_bytes(),
            ),
    ;

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> (result: Result<(), BlockDeviceError>)
        ensures
            final(self).physical_block_size_of_the_device() == old(self).physical_block_size_of_the_device(),
            final(self).size_of_the_device_in_bytes() == old(self).size_of_the_device_in_bytes(),
            result is Ok ==> request_is_aligned_and_in_range(
                offset,
                bytes@.len() as int,
                old(self).physical_block_size_of_the_device(),
                old(self).size_of_the_device_in_bytes(),
            ),
            issued_operations_after_the_call(
                old(self).issued_operations(),
                final(self).issued_operations(),
                issued_write_operation(offset, bytes@, durability),
                result is Ok,
            ),
    ;

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
    ) -> (result: Result<(), BlockDeviceError>)
        ensures
            final(self).physical_block_size_of_the_device() == old(self).physical_block_size_of_the_device(),
            final(self).size_of_the_device_in_bytes() == old(self).size_of_the_device_in_bytes(),
            result is Ok ==> request_is_aligned_and_in_range(
                offset,
                length as int,
                old(self).physical_block_size_of_the_device(),
                old(self).size_of_the_device_in_bytes(),
            ),
            issued_operations_after_the_call(
                old(self).issued_operations(),
                final(self).issued_operations(),
                issued_write_zeroes_operation(offset, length),
                result is Ok,
            ),
    ;

    /// discard：告诉盘 `[offset, offset + length)` 这一段不用了。偏移与长度同样要按物理块对齐、不越过设备末尾，不合就报
    /// [`BlockDeviceError::Unaligned`] / [`BlockDeviceError::OutOfRange`]、一个字节不动。
    ///
    /// 交回 [`DiscardOutcome::Applied`] 之后，那一段在持久之前读回旧字节或 0 都可能，持久之后读回 0 还是旧字节由设备定；
    /// 调用方不许指望读回什么（D13（验证路线） 已定项 4：模型按「范围写零」枚举持久 / 没持久两态）。
    /// 与写零一样没有 [`WriteDurability`] 参数：持久由后面的屏障给。
    ///
    /// **长度 0 没有动作**：核过请求（偏移照样要对齐、不越过末尾）就交回 [`DiscardOutcome::Applied`]、一个字节不动、
    /// 不记进 [`DiscardOutcomeCounts::applied_calls`]；默认实现、文件后端、`O_DIRECT` 后端与 harness 的内存盘结局相同
    /// 日志只在长度大于 0 而交回 [`DiscardOutcome::Applied`] 时多一条；交回不支持、长度 0、请求被拒的，日志不变。
    ///
    /// **默认实现只给不认 discard 的后端**：核过请求就交回 [`DiscardOutcome::Unsupported`]（长度 0 除外，见上），一个字节不动。
    /// 默认实现没有状态、记不了数：「交回了几次不支持」由调用方按返回值数，或由覆盖了它的后端自己数（[`DiscardOutcomeCounts`]）。
    /// 产品后端必须覆盖它（文件后端、`O_DIRECT` 后端都覆盖了）；包在别的设备外面的包装要转给里面那块盘，
    /// 不覆盖就把里面那块盘认得的 discard 吞成「不支持」。
    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> (result: Result<DiscardOutcome, BlockDeviceError>)
        ensures
            final(self).physical_block_size_of_the_device() == old(self).physical_block_size_of_the_device(),
            final(self).size_of_the_device_in_bytes() == old(self).size_of_the_device_in_bytes(),
            result is Ok ==> request_is_aligned_and_in_range(
                offset,
                length as int,
                old(self).physical_block_size_of_the_device(),
                old(self).size_of_the_device_in_bytes(),
            ),
            length == 0 && request_is_aligned_and_in_range(
                offset,
                length as int,
                old(self).physical_block_size_of_the_device(),
                old(self).size_of_the_device_in_bytes(),
            ) ==> result matches Ok(DiscardOutcome::Applied),
            final(self).issued_operations() == if length > 0 && result matches Ok(DiscardOutcome::Applied) {
                old(self).issued_operations().push(IssuedDeviceOperation::Discard { offset, length })
            } else {
                old(self).issued_operations()
            },
    {
        check_aligned_and_in_range(
            offset,
            length,
            self.probe_physical_block_size(),
            self.size_in_bytes(),
        )?;
        if length == 0 {
            // 默认实现：长度 0 没有动作，与认 discard 的后端结局相同。
            return Ok(DiscardOutcome::Applied);
        }
        Ok(DiscardOutcome::Unsupported)
    }
    /// 屏障：之前发出的写全部持久之后才返回。
    fn barrier(&mut self) -> (result: Result<(), BlockDeviceError>)
        ensures
            final(self).physical_block_size_of_the_device() == old(self).physical_block_size_of_the_device(),
            final(self).size_of_the_device_in_bytes() == old(self).size_of_the_device_in_bytes(),
            issued_operations_after_the_call(
                old(self).issued_operations(),
                final(self).issued_operations(),
                IssuedDeviceOperation::Barrier,
                result is Ok,
            ),
    ;

    fn probe_physical_block_size(&self) -> (physical_block_size: PhysicalBlockSizeInBytes)
        ensures
            physical_block_size.0 == self.physical_block_size_of_the_device(),
    ;

    fn size_in_bytes(&self) -> (device_size: u64)
        ensures
            device_size == self.size_of_the_device_in_bytes(),
    ;

    /// journal 环里可能有字节的记录槽偏移：恢复扫环时只读这些槽；`None` 是不知道，恢复按记录宽逐槽扫全环。
    /// 默认 `None`：文件后端与真设备不记哪些扇区写过。只记写过的扇区的内存盘答得出，答出的集合要罩住环里每个可能过得了
    /// journal magic 的槽——漏一个槽就少读一条记录，恢复落到的那一版与全环扫描不同
    /// （对拍在 `crates/singlefs-verification/tests/sparse_block_device_journal_hint_matches_the_full_ring_scan.rs`）。
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
    ) -> (result: Result<(), BlockDeviceError>)
        ensures
            final(buffer)@.len() == old(buffer)@.len(),
            result is Ok ==> request_is_aligned_and_in_range(
                offset,
                old(buffer)@.len() as int,
                self.physical_block_size_of_the_device(),
                self.size_of_the_device_in_bytes(),
            ),
    {
        self.read_at(offset, buffer)
    }

    /// 当场重读那一读读到、调用方核过自证（根槽自证、单元校验和与位置项相符）的字节：记着这次挂载读过什么的那一层拿它换掉记下的那一份，
    /// 之后再读这几块取的是它。默认什么都不做：盘自己不记读过什么。`offset` 与 `bytes` 是那一次重读的请求，对齐与越界照 [`BlockDevice::read_at`]。
    fn keep_the_self_verified_reread(&self, _offset: DeviceOffsetInBytes, _bytes: &[u8]) {
    }

    /// 一次读读成了、调用方核出自证不过（系统配置槽的 magic 与整槽校验和、节点大小，单元校验和与位置项不符）：`offset` 起 `length_in_bytes`
    /// 字节是那一次读的请求，对齐与越界照 [`BlockDevice::read_at`]。默认什么都不做：盘自己不记读过什么。
    /// 记着这次挂载读过什么的那一层（`mounted_session::DeviceReadOnceInThisMount`）把它记进读坏账、丢掉记下的那几块，下一次读它们穿到盘上
    /// （也记读坏、坏字节不留）。
    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        _offset: DeviceOffsetInBytes,
        _length_in_bytes: u64,
    ) {
    }

    /// 条款要这一刻读盘的那一读：取号写、轮换写与抬 F 先写系统配置那一步算世代号与覆写哪一槽时读本盘两个系统配置槽
    /// （D22（单元原子性怎么合成） 已定项 16 两槽轮换「覆写旧槽」要的是这一刻盘上哪一槽旧；可写挂载里这一读算进「取号写之前那几次读」）。
    /// 默认就是 [`BlockDevice::read_at`]：盘自己不记读过什么；
    /// 只进内存的预演那一层（`mounted_session::DeviceKeepingWritesInMemory`）也走默认，写过的块取内存里那一份、没写过的照它底下那一份的路读。
    /// 记着这次挂载读过什么的那一层（`mounted_session::DeviceReadOnceInThisMount`）在这里不取也不改记下的那一份、直接读底下那块盘。
    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> (result: Result<(), BlockDeviceError>)
        ensures
            final(buffer)@.len() == old(buffer)@.len(),
            result is Ok ==> request_is_aligned_and_in_range(
                offset,
                old(buffer)@.len() as int,
                self.physical_block_size_of_the_device(),
                self.size_of_the_device_in_bytes(),
            ),
    {
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
    /// 连着被打断几次都照样重打（只能打断第一次的开关测不出「被打断第二次就报错」）。
    ActAsIfTheFirstPunchesWereInterruptedBySignal {
        punches_interrupted_before_the_real_one: u64,
    },
}

/// 文件后端的 discard 怎么落到镜像文件上：复用写零的打洞（`punch_hole_keeping_the_file_length`）与同一段按 errno 分路的判定
/// （`classify_hole_punch_failure`）；与写零不同的只在「不支持打洞」那一路：discard 不退回逐块写 0，交回 [`DiscardOutcome::Unsupported`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscardLanding {
    /// 打洞；报不支持打洞交回 [`DiscardOutcome::Unsupported`]、一个字节不动，被信号打断原样重打，报别的错照常交回
    /// [`BlockDeviceError::InputOutput`]。
    PunchHoleOrAnswerUnsupported,
    /// 只供测试的开关（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：不调 `fallocate`，当作它报了 `EOPNOTSUPP`，
    /// 与真失败走同一段判定，落进「不支持」那条路。
    ActAsIfPunchingWereUnsupported,
    /// 只供测试的开关：不调 `fallocate`，当作它报了 `EIO`，落进照常报错那条路。
    ActAsIfPunchingFailedWithAnInputOutputError,
    /// 只供测试的开关：这一次 discard 的前 `punches_interrupted_before_the_real_one` 次打洞不调 `fallocate`，每次都当作它报了 `EINTR`；
    /// 之后那一次照 [`DiscardLanding::PunchHoleOrAnswerUnsupported`] 真调 `fallocate`。
    ActAsIfTheFirstPunchesWereInterruptedBySignal {
        punches_interrupted_before_the_real_one: u64,
    },
}

/// 打洞失败之后走哪条路，按 `fallocate` 交回的 errno 分。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HolePunchFailure {
    /// 文件系统不支持打洞（`EOPNOTSUPP`，Linux 上与 `ENOTSUP` 同值）或内核没有 `fallocate`（`ENOSYS`）：
    /// 退回逐块写 0，整段照样变成全 0。
    PunchingUnsupported,
    /// 打洞被信号打断（`EINTR`）：`fallocate(2)` 在动文件之前就返回了，原样再打一次洞；不算不支持、不退回逐块写 0，也不报错。
    InterruptedBySignal,
    /// 别的错（读写错、`EBADF`、`EINVAL`、`ENOSPC`，以及没有 errno 的）：照常报成块设备 I/O 错，不被退路吞掉。
    OtherFailure,
}

/// 按 errno 判一次打洞失败该退回逐块写 0，还是照常报错。
// errno 是 `io::Error` 里的，Verus 看不见：可信，登记进信任清单，差分测试逐个 errno 核分到哪条路。
#[verifier::external_body]
fn classify_hole_punch_failure(error: &io::Error) -> HolePunchFailure {
    match error.raw_os_error() {
        Some(libc::EOPNOTSUPP | libc::ENOSYS) => HolePunchFailure::PunchingUnsupported,
        Some(libc::EINTR) => HolePunchFailure::InterruptedBySignal,
        Some(_) | None => HolePunchFailure::OtherFailure,
    }
}

/// 只供测试的落地开关当作 `fallocate` 报了哪一个 errno：不支持打洞是 `EOPNOTSUPP`，被信号打断是 `EINTR`，别的错是 `EIO`。
/// 造出来的错与真失败走同一段按 errno 分路的判定（`classify_hole_punch_failure`），分回同一条路。
// 造 `io::Error` 与 libc 的 errno 常量 Verus 都不认：可信，登记进信任清单，差分测试核每一种造出来再分回原路。
#[verifier::external_body]
fn hole_punch_error_the_test_switch_stands_for(failure: HolePunchFailure) -> io::Error {
    match failure {
        HolePunchFailure::PunchingUnsupported => io::Error::from_raw_os_error(libc::EOPNOTSUPP),
        HolePunchFailure::InterruptedBySignal => io::Error::from_raw_os_error(libc::EINTR),
        HolePunchFailure::OtherFailure => io::Error::from_raw_os_error(libc::EIO),
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
    discard_landing: DiscardLanding,
    discard_outcome_counts: DiscardOutcomeCounts,
    /// discard 的打洞被信号打断、原样重打的次数（按打洞的次数数，不按 discard 的次数数）。
    discard_punch_attempts_interrupted_by_signal: u64,
    /// 这个句柄上发出过的改介质动作（[`BlockDevice::issued_operations`]；ghost，产品构建里是零字节的占位）。
    operations_issued_on_this_handle: Ghost<Seq<IssuedDeviceOperation>>,
}

impl FileBackedBlockDevice {
    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> (result: Result<(), BlockDeviceError>)
        ensures
            result is Ok <==> request_is_aligned_and_in_range(
                offset,
                length as int,
                self.physical_block_size.0,
                self.size_in_bytes,
            ),
    {
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
) -> (result: Result<(), BlockDeviceError>)
    ensures
        result is Ok <==> request_is_aligned_and_in_range(
            offset,
            length as int,
            physical_block_size.0,
            device_size,
        ),
        !is_a_whole_number_of_physical_blocks(offset.0 as int, physical_block_size.0)
            || !is_a_whole_number_of_physical_blocks(length as int, physical_block_size.0)
            ==> result matches Err(BlockDeviceError::Unaligned { .. }),
        is_a_whole_number_of_physical_blocks(offset.0 as int, physical_block_size.0)
            && is_a_whole_number_of_physical_blocks(length as int, physical_block_size.0)
            && offset.0 + length > device_size ==> result matches Err(BlockDeviceError::OutOfRange { .. }),
{
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

fn length_of(buffer: &[u8]) -> (length: u64)
    ensures
        length == buffer@.len(),
        length <= usize::MAX,
{
    u64::try_from(buffer.len()).expect("缓冲区长度装得进 u64")
}

/// 一次写零要准备多大的全 0 暂存区：整段与 [`ZERO_FILL_CHUNK_BYTES`] 取小的那个。
/// 清一段短于一个块的范围时不白白分配 4 MiB。
fn zero_fill_chunk_length(length: u64) -> (chunk_length: usize)
    ensures
        chunk_length == if length < ZERO_FILL_CHUNK_BYTES { length } else { ZERO_FILL_CHUNK_BYTES },
{
    usize::try_from(length.min(ZERO_FILL_CHUNK_BYTES)).expect("块长不超过 4 MiB，装得进 usize")
}

/// 两个后端共用的拆块循环：`zeros` 是后端备好的全 0 暂存区（它的长度就是块大小），
/// 从 `offset` 起按块顺序交给 `write_chunk`，最后一块按剩余长度截短。
///
/// 长度按物理块对齐由调用方先核过（[`check_aligned_and_in_range`]）；每一块的起点都还对齐，前提是物理块大小整除块大小 4 MiB
/// （512、4096 都整除）。这个前提这里不判，物理块大小的来处也都不判它：[`PhysicalBlockSizeSource::Declared`] 收调用方给的任何值，
/// sysfs 读到的值也不核；不整除时第二块起的起点不对齐。
/// `requires` 是循环走得完、偏移加不出界的前提：长度大于 0 时暂存区不空（空的每轮写 0 字节、永远转），整段不越过 `u64`。
fn write_zeroes_in_chunks<WriteChunk>(
    offset: DeviceOffsetInBytes,
    length: u64,
    zeros: &[u8],
    write_chunk: WriteChunk,
) -> (result: Result<(), BlockDeviceError>)
where
    WriteChunk: Fn(DeviceOffsetInBytes, &[u8]) -> Result<(), BlockDeviceError>,
    requires
        length == 0 || zeros@.len() > 0,
        offset.0 + length <= u64::MAX,
        forall|chunk_offset: DeviceOffsetInBytes, chunk: &[u8]| write_chunk.requires((chunk_offset, chunk)),
{
    // 跨轮的状态只有已交出去的字节数 `written`（只增、不越过 `length`）；出口两个：写完、某一块写坏。
    let mut written: u64 = 0;
    while written < length
        invariant
            length == 0 || zeros@.len() > 0,
            offset.0 + length <= u64::MAX,
            forall|chunk_offset: DeviceOffsetInBytes, chunk: &[u8]| write_chunk.requires((chunk_offset, chunk)),
        decreases length - written,
    {
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
// 系统调用 Verus 验不了：可信，登记进信任清单，差分测试核打过的段读回全 0、段外与文件长度不动。
#[verifier::external_body]
#[allow(
    unsafe_code,
    reason = "std 没有 fallocate 的安全封装；全 crate 放开 unsafe 的两处之一，另一处是 checksum::crc32_castagnoli_on_route 调硬件 CRC 指令（C603（测试与验证的资源浪费没有自检））"
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
    // 转成非负的 `off_t`，放不下就在转换处拒；越界的请求到不了这里：两个调用方 `write_zeroes_at` 与 `discard_at` 都先过 `check_request`
    // （对齐、落在设备范围内），长度 0 也在它们里面先交回、不到这里。
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

/// `pread` 一族：从 `file` 的 `offset` 处读满 `buffer`（std 的 `read_exact_at`，被信号打断它自己重试）。
// 系统调用 Verus 验不了：可信，登记进信任清单，差分测试核写进去的字节读回来相同、缓冲长度不变。
#[verifier::external_body]
fn read_exact_at_the_offset(file: &File, buffer: &mut [u8], offset: u64) -> (result: io::Result<()>)
    ensures
        final(buffer)@.len() == old(buffer)@.len(),
{
    file.read_exact_at(buffer, offset)
}

/// `pwrite` 一族：把 `bytes` 整段写到 `file` 的 `offset` 处（std 的 `write_all_at`，被信号打断它自己重试）。
// 系统调用 Verus 验不了：可信，登记进信任清单，差分测试核写下去的字节读回来相同、段外不动。
#[verifier::external_body]
fn write_all_at_the_offset(file: &File, bytes: &[u8], offset: u64) -> io::Result<()> {
    file.write_all_at(bytes, offset)
}

/// `fdatasync`：之前写进 `file` 的字节全部持久之后才返回（块设备上是一次 FLUSH）。
// 系统调用 Verus 验不了：可信，登记进信任清单，差分测试核它在打开着的镜像上交回 Ok、不改字节。
#[verifier::external_body]
fn sync_data_of_the_file(file: &File) -> io::Result<()> {
    file.sync_data()
}

impl BlockDevice for FileBackedBlockDevice {
    closed spec fn issued_operations(&self) -> Seq<IssuedDeviceOperation> {
        self.operations_issued_on_this_handle@
    }

    closed spec fn physical_block_size_of_the_device(&self) -> u32 {
        self.physical_block_size.0
    }

    closed spec fn size_of_the_device_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> (result: Result<(), BlockDeviceError>) {
        self.check_request(offset, length_of(buffer))?;
        match read_exact_at_the_offset(&self.file, buffer, offset.0) {
            Ok(()) => Ok(()),
            Err(error) => Err(BlockDeviceError::InputOutput(error)),
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> (result: Result<(), BlockDeviceError>) {
        self.check_request(offset, length_of(bytes))?;
        self.operations_issued_on_this_handle = Ghost(
            self.operations_issued_on_this_handle@.push(
                IssuedDeviceOperation::Write { offset, bytes: bytes@, durability },
            ),
        );
        if let Err(error) = write_all_at_the_offset(&self.file, bytes, offset.0) {
            return Err(BlockDeviceError::InputOutput(error));
        }
        match durability {
            WriteDurability::Plain => Ok(()),
            WriteDurability::ForceUnitAccess => match sync_data_of_the_file(&self.file) {
                Ok(()) => Ok(()),
                Err(error) => Err(BlockDeviceError::InputOutput(error)),
            },
        }
    }

    // 打洞被信号打断就原样重打，轮数的上界是这期间送到的信号数、不是一个能写成 decreases 的量：这个循环 Verus 不证它停。
    #[verifier::exec_allows_no_decreases_clause]
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> (result: Result<(), BlockDeviceError>) {
        self.check_request(offset, length)?;
        // 长度 0 不调系统调用：`fallocate(2)` 对长度 0 报 `EINVAL`，而另两个后端（`O_DIRECT` 后端、内存稀疏盘）对它交回 `Ok`、
        // 一个字节不动。同一个请求三个后端结局相同，这里照它们交回 `Ok`，不记进落地计数。
        // 日志照样记这一次写零（交回 `Ok` 的写零都记，长度 0 也是）。
        self.operations_issued_on_this_handle = Ghost(
            self.operations_issued_on_this_handle@.push(
                IssuedDeviceOperation::WriteZeroes { offset, length },
            ),
        );
        if length == 0 {
            return Ok(());
        }
        // 循环：每一轮打一次洞。跨轮的状态只有这一次写零里被信号打断的次数（`punch_attempts_interrupted`，只增）；
        // 出口三个：打成洞、报不支持（退回逐块写 0）、报别的错。被打断就原样再打，轮数的上界是这期间送到的信号数
        // （与 std 的 `write_all_at` 对 `ErrorKind::Interrupted` 的处理同一个口径），所以这个循环不带 `decreases`。
        let mut punch_attempts_interrupted: u64 = 0;
        loop
            invariant
                self.physical_block_size == old(self).physical_block_size,
                self.size_in_bytes == old(self).size_in_bytes,
                self.operations_issued_on_this_handle@ == old(self).operations_issued_on_this_handle@.push(
                    IssuedDeviceOperation::WriteZeroes { offset, length },
                ),
                request_is_aligned_and_in_range(
                    offset,
                    length as int,
                    self.physical_block_size.0,
                    self.size_in_bytes,
                ),
        {
            let punched = match self.zero_fill_landing {
                ZeroFillLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal {
                    punches_interrupted_before_the_real_one,
                } if punch_attempts_interrupted < punches_interrupted_before_the_real_one => {
                    Err(hole_punch_error_the_test_switch_stands_for(HolePunchFailure::InterruptedBySignal))
                }
                ZeroFillLanding::PunchHoleOrWriteZeroBytesIfUnsupported
                | ZeroFillLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal { .. } => {
                    punch_hole_keeping_the_file_length(&self.file, offset, length)
                }
                ZeroFillLanding::ActAsIfPunchingWereUnsupported => {
                    Err(hole_punch_error_the_test_switch_stands_for(HolePunchFailure::PunchingUnsupported))
                }
                ZeroFillLanding::ActAsIfPunchingFailedWithAnInputOutputError => {
                    Err(hole_punch_error_the_test_switch_stands_for(HolePunchFailure::OtherFailure))
                }
            };
            let punch_error = match punched {
                Ok(()) => {
                    self.zero_fill_landing_counts.punched_calls =
                        self.zero_fill_landing_counts.punched_calls.saturating_add(1);
                    return Ok(());
                }
                Err(punch_error) => punch_error,
            };
            match classify_hole_punch_failure(&punch_error) {
                HolePunchFailure::PunchingUnsupported => {
                    self.zero_fill_landing_counts.written_as_zero_bytes_calls =
                        self.zero_fill_landing_counts.written_as_zero_bytes_calls.saturating_add(1);
                    let zeros = vec![0u8; zero_fill_chunk_length(length)];
                    let file = &self.file;
                    return write_zeroes_in_chunks(
                        offset,
                        length,
                        &zeros,
                        |chunk_offset: DeviceOffsetInBytes, chunk: &[u8]| -> Result<(), BlockDeviceError> {
                            match write_all_at_the_offset(file, chunk, chunk_offset.0) {
                                Ok(()) => Ok(()),
                                Err(error) => Err(BlockDeviceError::InputOutput(error)),
                            }
                        },
                    );
                }
                HolePunchFailure::InterruptedBySignal => {
                    punch_attempts_interrupted = punch_attempts_interrupted.saturating_add(1);
                    self.zero_fill_landing_counts.punch_attempts_interrupted_by_signal =
                        self.zero_fill_landing_counts.punch_attempts_interrupted_by_signal.saturating_add(1);
                }
                HolePunchFailure::OtherFailure => {
                    return Err(BlockDeviceError::InputOutput(punch_error));
                }
            }
        }
    }

    /// 复用写零的打洞：打成洞之后读这一段是全 0，文件长度不变（D13（验证路线） 已定项 4 的「持久 → 0」那一态在镜像文件上就是它）。
    /// 文件系统不支持打洞时交回 [`DiscardOutcome::Unsupported`]、不退回逐块写 0（discard 不承诺读回 0，写 0 只是白写）；
    /// 被信号打断原样重打；别的错照常报 [`BlockDeviceError::InputOutput`]。
    // 同写零：被信号打断原样重打的轮数没有能写成 decreases 的上界，这个循环 Verus 不证它停。
    #[verifier::exec_allows_no_decreases_clause]
    fn discard_at(
        &mut self,
        discarded_offset: DeviceOffsetInBytes,
        discarded_length: u64,
    ) -> (result: Result<DiscardOutcome, BlockDeviceError>) {
        self.check_request(discarded_offset, discarded_length)?;
        // 长度 0 没有动作（[`BlockDevice::discard_at`] 的文档）：`fallocate(2)` 对它报 `EINVAL`，不调；照「交给了底层」交回、不记数。
        if discarded_length == 0 {
            return Ok(DiscardOutcome::Applied);
        }
        // 循环：每一轮打一次洞。跨轮的状态只有这一次 discard 里被信号打断的次数（只增）；
        // 出口三个：打成洞、报不支持、报别的错。被打断就原样再打，轮数的上界是这期间送到的信号数，所以不带 `decreases`。
        let mut interrupted_in_this_discard: u64 = 0;
        loop
            invariant
                self.physical_block_size == old(self).physical_block_size,
                self.size_in_bytes == old(self).size_in_bytes,
                self.operations_issued_on_this_handle@ == old(self).operations_issued_on_this_handle@,
                request_is_aligned_and_in_range(
                    discarded_offset,
                    discarded_length as int,
                    self.physical_block_size.0,
                    self.size_in_bytes,
                ),
                discarded_length > 0,
        {
            let punch_result = match self.discard_landing {
                DiscardLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal {
                    punches_interrupted_before_the_real_one,
                } if interrupted_in_this_discard < punches_interrupted_before_the_real_one => {
                    Err(hole_punch_error_the_test_switch_stands_for(HolePunchFailure::InterruptedBySignal))
                }
                DiscardLanding::PunchHoleOrAnswerUnsupported
                | DiscardLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal { .. } => {
                    punch_hole_keeping_the_file_length(
                        &self.file,
                        discarded_offset,
                        discarded_length,
                    )
                }
                DiscardLanding::ActAsIfPunchingWereUnsupported => {
                    Err(hole_punch_error_the_test_switch_stands_for(HolePunchFailure::PunchingUnsupported))
                }
                DiscardLanding::ActAsIfPunchingFailedWithAnInputOutputError => {
                    Err(hole_punch_error_the_test_switch_stands_for(HolePunchFailure::OtherFailure))
                }
            };
            let discard_punch_error = match punch_result {
                Ok(()) => {
                    self.discard_outcome_counts.note(DiscardOutcome::Applied);
                    self.operations_issued_on_this_handle = Ghost(
                        self.operations_issued_on_this_handle@.push(
                            IssuedDeviceOperation::Discard {
                                offset: discarded_offset,
                                length: discarded_length,
                            },
                        ),
                    );
                    return Ok(DiscardOutcome::Applied);
                }
                Err(discard_punch_error) => discard_punch_error,
            };
            match classify_hole_punch_failure(&discard_punch_error) {
                HolePunchFailure::PunchingUnsupported => {
                    self.discard_outcome_counts
                        .note(DiscardOutcome::Unsupported);
                    return Ok(DiscardOutcome::Unsupported);
                }
                HolePunchFailure::InterruptedBySignal => {
                    interrupted_in_this_discard = interrupted_in_this_discard.saturating_add(1);
                    self.discard_punch_attempts_interrupted_by_signal =
                        self.discard_punch_attempts_interrupted_by_signal.saturating_add(1);
                }
                HolePunchFailure::OtherFailure => {
                    return Err(BlockDeviceError::InputOutput(discard_punch_error));
                }
            }
        }
    }

    fn barrier(&mut self) -> (result: Result<(), BlockDeviceError>) {
        self.operations_issued_on_this_handle = Ghost(
            self.operations_issued_on_this_handle@.push(IssuedDeviceOperation::Barrier),
        );
        match sync_data_of_the_file(&self.file) {
            Ok(()) => Ok(()),
            Err(error) => Err(BlockDeviceError::InputOutput(error)),
        }
    }

    fn probe_physical_block_size(&self) -> (physical_block_size: PhysicalBlockSizeInBytes) {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> (device_size: u64) {
        self.size_in_bytes
    }
}

/// 读写走不走页缓存。真设备上一律绕开（块层计数才与程序数的对得上）；走页缓存的那一臂只给阳性对照用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageCachePolicy {
    BypassWithDirectInputOutput,
    GoThroughPageCache,
}

/// 一块按 4096 对齐的暂存区：在多分配 4096 字节的 Vec 里用 `align_offset` 找对齐起点，不写 unsafe。
struct AlignedScratch {
    storage: Vec<u8>,
    start: usize,
    length: usize,
}

impl AlignedScratch {
    /// 对齐起点之后还装得下 `length` 字节。
    spec fn holds_the_window(&self) -> bool {
        self.start + self.length <= self.storage@.len() <= usize::MAX
    }

    // 指针对齐 Verus 验不了：可信，登记进信任清单，差分测试核每一种长度的起点都落在 4096 的整数倍上、窗口装得下。
    #[verifier::external_body]
    fn new(length: usize) -> (scratch: Self)
        ensures
            scratch.holds_the_window(),
            scratch.length == length,
    {
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

    fn as_slice(&self) -> (window: &[u8])
        requires
            self.holds_the_window(),
        ensures
            window@ == self.storage@.subrange(self.start as int, self.start + self.length),
    {
        &self.storage[self.start..self.start + self.length]
    }

    fn as_mut_slice(&mut self) -> (window: &mut [u8])
        requires
            old(self).holds_the_window(),
        ensures
            window@.len() == old(self).length,
            final(self).start == old(self).start,
            final(self).length == old(self).length,
            final(self).storage@.len() == old(self).storage@.len(),
    {
        let end = self.start + self.length;
        &mut self.storage.as_mut_slice()[self.start..end]
    }
}

/// `O_DIRECT` 后端：打开一个已经存在的块设备（或普通文件），大小取设备末尾的偏移，不改大小。
pub struct DirectInputOutputBlockDevice {
    file: File,
    size_in_bytes: u64,
    physical_block_size: PhysicalBlockSizeInBytes,
    policy: PageCachePolicy,
    discard_outcome_counts: DiscardOutcomeCounts,
    /// 这个句柄上发出过的改介质动作（[`BlockDevice::issued_operations`]；ghost，产品构建里是零字节的占位）。
    operations_issued_on_this_handle: Ghost<Seq<IssuedDeviceOperation>>,
}

impl BlockDevice for DirectInputOutputBlockDevice {
    closed spec fn issued_operations(&self) -> Seq<IssuedDeviceOperation> {
        self.operations_issued_on_this_handle@
    }

    closed spec fn physical_block_size_of_the_device(&self) -> u32 {
        self.physical_block_size.0
    }

    closed spec fn size_of_the_device_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> (result: Result<(), BlockDeviceError>) {
        check_aligned_and_in_range(
            offset,
            length_of(buffer),
            self.physical_block_size,
            self.size_in_bytes,
        )?;
        match self.policy {
            PageCachePolicy::BypassWithDirectInputOutput => {
                let mut scratch = AlignedScratch::new(buffer.len());
                if let Err(error) = read_exact_at_the_offset(&self.file, scratch.as_mut_slice(), offset.0) {
                    return Err(BlockDeviceError::InputOutput(error));
                }
                buffer.copy_from_slice(scratch.as_slice());
                Ok(())
            }
            PageCachePolicy::GoThroughPageCache => match read_exact_at_the_offset(&self.file, buffer, offset.0) {
                Ok(()) => Ok(()),
                Err(error) => Err(BlockDeviceError::InputOutput(error)),
            },
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> (result: Result<(), BlockDeviceError>) {
        check_aligned_and_in_range(
            offset,
            length_of(bytes),
            self.physical_block_size,
            self.size_in_bytes,
        )?;
        self.operations_issued_on_this_handle = Ghost(
            self.operations_issued_on_this_handle@.push(
                IssuedDeviceOperation::Write { offset, bytes: bytes@, durability },
            ),
        );
        let written = match self.policy {
            PageCachePolicy::BypassWithDirectInputOutput => {
                let mut scratch = AlignedScratch::new(bytes.len());
                scratch.as_mut_slice().copy_from_slice(bytes);
                write_all_at_the_offset(&self.file, scratch.as_slice(), offset.0)
            }
            PageCachePolicy::GoThroughPageCache => write_all_at_the_offset(&self.file, bytes, offset.0),
        };
        if let Err(error) = written {
            return Err(BlockDeviceError::InputOutput(error));
        }
        match durability {
            WriteDurability::Plain => Ok(()),
            WriteDurability::ForceUnitAccess => match sync_data_of_the_file(&self.file) {
                Ok(()) => Ok(()),
                Err(error) => Err(BlockDeviceError::InputOutput(error)),
            },
        }
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> (result: Result<(), BlockDeviceError>) {
        check_aligned_and_in_range(offset, length, self.physical_block_size, self.size_in_bytes)?;
        self.operations_issued_on_this_handle = Ghost(
            self.operations_issued_on_this_handle@.push(
                IssuedDeviceOperation::WriteZeroes { offset, length },
            ),
        );
        // `O_DIRECT` 要求用户缓冲对齐，所以全 0 暂存区走 `AlignedScratch`（它自己就是全 0 的）；
        // 走页缓存那一臂不需要对齐，但用同一块暂存区，两臂发出的 pwrite 完全一样。
        let scratch = AlignedScratch::new(zero_fill_chunk_length(length));
        let file = &self.file;
        write_zeroes_in_chunks(
            offset,
            length,
            scratch.as_slice(),
            |chunk_offset: DeviceOffsetInBytes, chunk: &[u8]| -> Result<(), BlockDeviceError> {
                match write_all_at_the_offset(file, chunk, chunk_offset.0) {
                    Ok(()) => Ok(()),
                    Err(error) => Err(BlockDeviceError::InputOutput(error)),
                }
            },
        )
    }

    /// 第一版不认 discard：核过请求就交回 [`DiscardOutcome::Unsupported`]，一个字节不动、不发系统调用，两臂（绕开 / 走页缓存）一样；
    /// 长度 0 的照 trait 文档交回 [`DiscardOutcome::Applied`]、不记数。
    /// 真块设备上的 discard 是 `BLKDISCARD` ioctl，留到真盘线（`research/prompts/m4-gc-r1-main-verification.md` 第三节 G5「真盘 ioctl 留到真盘线」）；
    /// 在那之前虚机里的设备侧日志里不会有 Discard 条目，宿主重跑那一侧要照这个结局投（`device_log.rs`）。
    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> (result: Result<DiscardOutcome, BlockDeviceError>) {
        check_aligned_and_in_range(offset, length, self.physical_block_size, self.size_in_bytes)?;
        if length == 0 {
            // `O_DIRECT` 后端：长度 0 没有动作，交回「交给了底层」、不记数（[`BlockDevice::discard_at`] 的文档）。
            return Ok(DiscardOutcome::Applied);
        }
        self.discard_outcome_counts
            .note(DiscardOutcome::Unsupported);
        Ok(DiscardOutcome::Unsupported)
    }

    fn barrier(&mut self) -> (result: Result<(), BlockDeviceError>) {
        self.operations_issued_on_this_handle = Ghost(
            self.operations_issued_on_this_handle@.push(IssuedDeviceOperation::Barrier),
        );
        match sync_data_of_the_file(&self.file) {
            Ok(()) => Ok(()),
            Err(error) => Err(BlockDeviceError::InputOutput(error)),
        }
    }

    fn probe_physical_block_size(&self) -> (physical_block_size: PhysicalBlockSizeInBytes) {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> (device_size: u64) {
        self.size_in_bytes
    }
}

} // verus!

/// 两个构造入口起步时一样：discard 先打洞。
const INITIAL_DISCARD_LANDING: DiscardLanding = DiscardLanding::PunchHoleOrAnswerUnsupported;

/// 两个构造入口起步时一样：先打洞，计数从 0 起。
const INITIAL_ZERO_FILL_LANDING: ZeroFillLanding =
    ZeroFillLanding::PunchHoleOrWriteZeroBytesIfUnsupported;
const NO_ZERO_FILL_LANDED_YET: ZeroFillLandingCounts = ZeroFillLandingCounts {
    punched_calls: 0,
    written_as_zero_bytes_calls: 0,
    punch_attempts_interrupted_by_signal: 0,
};

// 建句柄与只供测试的开关不在 `verus!` 里（模块文档）：句柄上的日志从这里交回的那一刻起记，`Ghost::assume_new` 是它的起点。
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
            discard_landing: self.discard_landing,
            discard_outcome_counts: self.discard_outcome_counts,
            discard_punch_attempts_interrupted_by_signal: self
                .discard_punch_attempts_interrupted_by_signal,
            operations_issued_on_this_handle: self.operations_issued_on_this_handle,
        }
    }

    /// 只供测试：换掉 discard 的落地方式（[`DiscardLanding`] 的三个 `ActAsIf…` 开关各强制走一条路）。
    #[must_use]
    pub fn with_test_only_discard_landing(self, landing: DiscardLanding) -> Self {
        Self {
            file: self.file,
            size_in_bytes: self.size_in_bytes,
            physical_block_size: self.physical_block_size,
            zero_fill_landing: self.zero_fill_landing,
            zero_fill_landing_counts: self.zero_fill_landing_counts,
            discard_landing: landing,
            discard_outcome_counts: self.discard_outcome_counts,
            discard_punch_attempts_interrupted_by_signal: self
                .discard_punch_attempts_interrupted_by_signal,
            operations_issued_on_this_handle: self.operations_issued_on_this_handle,
        }
    }

    #[must_use]
    pub fn zero_fill_landing_counts(&self) -> ZeroFillLandingCounts {
        self.zero_fill_landing_counts
    }

    /// 这个句柄上的 discard 各交回了哪一种结局（`.claude/rules/fs-design.md`「五条硬要求」第 4 条：分支可观测）。
    #[must_use]
    pub fn discard_outcome_counts(&self) -> DiscardOutcomeCounts {
        self.discard_outcome_counts
    }

    /// 这个句柄上 discard 的打洞被信号打断、原样重打了几次。
    #[must_use]
    pub fn discard_punch_attempts_interrupted_by_signal(&self) -> u64 {
        self.discard_punch_attempts_interrupted_by_signal
    }

    /// 排他地建出一个镜像文件并把它撑到 `size_in_bytes`；稀疏文件，不真占盘。
    /// 同名文件已经在那儿了就报 [`BlockDeviceError::ImageFileAlreadyExists`]，不接着用旧镜像：
    /// 测试的镜像名带进程号，而进程号会被系统复用、上一轮被杀留下的文件不会被清——接着用就是拿上一轮的盘面当这一轮的起点，
    /// 而且是静默的。
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
            discard_landing: INITIAL_DISCARD_LANDING,
            discard_outcome_counts: DiscardOutcomeCounts::NONE,
            discard_punch_attempts_interrupted_by_signal: 0,
            operations_issued_on_this_handle: Ghost::assume_new(),
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
            discard_landing: INITIAL_DISCARD_LANDING,
            discard_outcome_counts: DiscardOutcomeCounts::NONE,
            discard_punch_attempts_interrupted_by_signal: 0,
            operations_issued_on_this_handle: Ghost::assume_new(),
        })
    }
}

/// Linux open(2) 的 `O_DIRECT` 位；取值随架构不同，只接本机与虚机的两种，别的架构编不过。
#[cfg(target_arch = "x86_64")]
const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名，取自 <fcntl.h>，名字不归我们定
#[cfg(target_arch = "aarch64")]
const O_DIRECT: i32 = 0o200000; // naming-lint:external Linux open(2) 标志名，取自 <fcntl.h>，名字不归我们定

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
            discard_outcome_counts: DiscardOutcomeCounts::NONE,
            operations_issued_on_this_handle: Ghost::assume_new(),
        })
    }

    /// 这个句柄上的 discard 各交回了哪一种结局；第一版每一次都记「不支持」，长度 0 的不记（[`BlockDevice::discard_at`] 在这个后端上的文档）。
    #[must_use]
    pub fn discard_outcome_counts(&self) -> DiscardOutcomeCounts {
        self.discard_outcome_counts
    }
}

#[cfg(test)]
mod tests;
