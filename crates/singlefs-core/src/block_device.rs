//! 块设备抽象：读、写、屏障、FUA 写、探测 `physical_block_size`、写零。
//!
//! 第一版两个后端（D17（实现分层与第三方管道） 已定项 5）：文件后端（测试镜像）与 `O_DIRECT` 后端（虚机里的真块设备）。
//! 两个后端的持久语义相同：屏障 = `sync_data`（块设备上是一次 FLUSH），FUA 写 = 写完立刻 `sync_data`。
//!
//! 「写零」是第六个动作（用户 2026-09-19 定案，`records/2026-09-19-里程碑二遗留收拢.md` 第五之二节第 4 行：
//! 问「块设备抽象加不加写零」答「加」）：一次调用覆盖 `[offset, offset + length)` 一整段，真设备上对应 WRITE ZEROES。
//! **一次调用就是一个动作**——底下拆成几次 I/O 由后端自己定（[`ZERO_FILL_CHUNK_BYTES`]），上层（录制流、段序列）看到的是一步。

use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
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

/// 六个动作，只有这六个。介质是什么这里看不见。
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
}

/// 文件后端：一个镜像文件当一块盘。物理块大小按参数给（本机 NVMe 是 512，D22（单元原子性怎么合成） 已定项 9 那一节现查）。
pub struct FileBackedBlockDevice {
    file: File,
    size_in_bytes: u64,
    physical_block_size: PhysicalBlockSizeInBytes,
}

impl FileBackedBlockDevice {
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

/// 请求要按物理块对齐、不越过设备末尾；两个后端共用这一份判据。
fn check_aligned_and_in_range(
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
/// 长度按物理块对齐由调用方先核过（[`check_aligned_and_in_range`]），而块大小 4 MiB 是 512 与 4096 的整数倍，
/// 所以每一块都还是对齐的。
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
        let zeros = vec![0u8; zero_fill_chunk_length(length)];
        write_zeroes_in_chunks(offset, length, &zeros, |chunk_offset, chunk| {
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
