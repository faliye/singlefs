//! E184（常规层与大负载层的代价计数）写流转储 bin（跑前登记 `research/prompts/e184-preregistration.md` 5.3 第一份 bin）。
//!
// admission: always 每跑一次都在今天这份 crates 上重录随机历史的写流、重核基镜像往返，转储里的挂钟每次都是新观测；已有写完的转储由装置自己整份跳过
// run-condition: command python3
//!
//! 只录写流、写转储：只引 `singlefs-harness`、`singlefs-core`，不引 checker 档包自己的模块，好让它在第二段的旧提交副本里也编得过。
//! 转储是二进制文件，装一条流从 mkfs 做完那一刻起的全部东西：基镜像 = 空盘叠上 mkfs 的全部写（连同清零写）、mkfs 之后的写表与段、
//! 版本表、历史怎么收的尾。块键不在这里算（算块键要引 checker 档包），由 `e184_regular_layer_cost_counts` 的 `keys` 读转储去算。
//! 另一份 bin 经 `#[path]` 把这个文件当模块引进去，转储的写法与读法因此只有这一份。
//!
//!   e184_regular_layer_cost_counts_write_dump dump-random-histories --steps <步数> --weights <BROAD|BROAD_WITH_COMPACTION_WHILE_MOUNTED>
//!       --width <FourGibibytes|UnitAreaOf240Slots> --seeds <条数> --out <目录> [--check-determinism]
//!
//! 种子 = 种子基 + 下标（下标从 0 起，共 `--seeds` 条）。每条历史一份转储，文件名 `<流名>.dump`，先写临时名再改名：重起同一条命令时已有的文件
//! （末尾有 `name=dump_done` 收尾）整条跳过。`--check-determinism` 另把种子下标 0 与 1 各再录一遍，两遍的写表摘要不同就退 4（登记 X2）。
//! 线程数取环境变量 `E184_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值。
//! 每条历史的 panic 原样记进 `name=dump_failed` 行（登记停机 T7），不换种子补数。
//! 每份新录的转储另核一遍（登记 5.2「转储往返」）：由转储里的写表重建的基镜像，与直接把录制流 mkfs 那几步叠到空盘上的基镜像逐扇区相同，
//! 结果打在 `name=dump` 行的 `base_matches_the_stream=` 上，收尾行报不同的条数 `base_mismatches=`。
//! 退出码：0 正常；2 命令行不对；3 有历史没录成（panic 或 mkfs 之后没有操作）；4 阳性对照没过（确定性核或基镜像往返核，登记 X1、X2）；
//! 78 准入与运行条件没满足（`preflight` 拒绝，带 `--force` 才照跑；强制跑的那一次，标准输出第一行多一行 `name=preflight forced=…`，`emitted=` 不数它）。

use std::collections::BTreeMap;
use std::io::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperationKind, HistoryRun, HistorySeed, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, PublishedVersion, PublishedVersionWithoutAFile, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES};
use singlefs_harness::{RetainedOperation, SharedStream};

/// 随机历史的种子基：值抄自 `crates/singlefs-harness/src/history.rs` 的 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`（注释写「种子基永久固定」）。
/// 主装置的 `selfcheck` 与 crates 那一份回比（登记停机 T1）。
pub const SEED_BASE_OF_THE_RANDOM_HISTORIES: u64 = 7_463_871_032_432_355_113;

/// 转储文件头的魔数与格式版本：读到别的就拒。
const DUMP_MAGIC: &[u8; 8] = b"E184DUMP";
const DUMP_FORMAT_VERSION: u64 = 1;
/// 转储文件的收尾：续跑时看文件末尾有它才算整份写完（写完才改名，改名之后一定有它）。
pub const DUMP_TRAILER: &[u8] = b"name=dump_done\n";

const THREADS_ENVIRONMENT_VARIABLE: &str = "E184_THREADS";
const THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";
/// 每台默认重活线程数的登记原文：构建时编进来，运行时不找仓根（与 `singlefs_harness::heavy_threads` 同一个读法；这个 bin 要在旧提交副本里编，
/// 旧提交里那个模块不一定有，所以这里自己读一遍）。相对的那一段单写成以 `../` 起头的字面量，拷贝范围漏了这份登记时门禁点名。
const HEAVY_THREAD_REGISTRATION_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/",
    "../../.claude/heavy-thread-default"
));

pub const EXIT_CODE_OK: i32 = 0;
pub const EXIT_CODE_USAGE: i32 = 2;
pub const EXIT_CODE_HISTORY_NOT_RECORDED: i32 = 3;
pub const EXIT_CODE_POSITIVE_CONTROL_FAILED: i32 = 4;

/// 线程数：`E184_THREADS` 设了照它，没设取 `SINGLEFS_THREAD_CAP`，再没设取登记值。设了而不是正整数就停下，不悄悄退回登记值。
///
/// # Panics
/// 两个变量里设了的那个不是正整数，或编进来的登记原文读不出数。
#[must_use]
pub fn worker_threads_from_the_environment() -> usize {
    worker_threads_given(
        std::env::var(THREADS_ENVIRONMENT_VARIABLE).ok().as_deref(),
        std::env::var(THREAD_CAP_ENVIRONMENT_VARIABLE)
            .ok()
            .as_deref(),
        HEAVY_THREAD_REGISTRATION_TEXT,
    )
}

/// [`worker_threads_from_the_environment`] 的判法，两个变量的值与登记原文由调用方给（`None` 是没设）。
///
/// # Panics
/// 设了的变量不是正整数；登记原文里没有「<正整数>」起头的非注释行。
#[must_use]
pub fn worker_threads_given(
    own_value: Option<&str>,
    thread_cap_value: Option<&str>,
    registration_text: &str,
) -> usize {
    let positive_integer = |text: &str| {
        text.parse::<usize>()
            .ok()
            .filter(|count| *count > 0 && !text.starts_with('0'))
    };
    if let Some(text) = own_value {
        return positive_integer(text)
            .unwrap_or_else(|| panic!("{THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}"));
    }
    if let Some(text) = thread_cap_value.filter(|text| !text.is_empty()) {
        return positive_integer(text).unwrap_or_else(|| {
            panic!("{THREAD_CAP_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}")
        });
    }
    let registered_line = registration_text
        .lines()
        .map(|line| line.trim_start_matches([' ', '\t']))
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .expect("登记文件里有一条非注释行");
    let digits: String = registered_line
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    positive_integer(&digits)
        .unwrap_or_else(|| panic!("登记文件第一条非注释行 {registered_line:?} 不是以正整数起头"))
}

// ============================================================================================
// 转储：一条流从 mkfs 做完那一刻起的全部东西
// ============================================================================================

/// 一条流的转储。基镜像 = 空盘叠上 `base_writes`（mkfs 的全部写，连同清零写）；`writes` 与 `segments` 只取 mkfs 之后的操作切出来的写表与段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowDump {
    /// 哪一族：`random_history`、`overwrite_chain`、`smoke`。
    pub family: String,
    /// 流名，也是转储与块键文件的文件名主干。
    pub label: String,
    pub tier: String,
    pub seed: Option<u64>,
    /// 历史怎么收的尾（`Completed`、`KnownRed:form=<号>`、`NewFinding`）；不是随机历史的流写 `NotAHistory`。
    pub ending: String,
    pub device_bytes: u64,
    pub device_identities: Vec<u32>,
    /// mkfs 占了录制流开头的几步。
    pub operations_of_the_make_filesystem: usize,
    pub base_writes: Vec<RetainedWrite>,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    /// 被判的那次根槽写在写表里的下标：写表里最后一条根槽 FUA 写；一条都没有是 `None`。
    pub judged_root_index: Option<usize>,
    pub versions_with_a_file: Vec<PublishedVersion>,
    pub versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    /// 生成出来的历史里挂着时整理这一类操作的次数：数的是历史的操作表，不是录出来的写流，是登记锚 A11 的独立一路；不是随机历史的流是 `None`。
    pub compaction_operations: Option<u64>,
}

/// 往字节序列尾部写：整数一律 8 字节小端，文本与字节串先写长度。
struct ByteSink {
    bytes: Vec<u8>,
}

impl ByteSink {
    fn put_unsigned(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn put_count(&mut self, count: usize) {
        self.put_unsigned(u64::try_from(count).expect("个数装得进 u64"));
    }
    fn put_byte(&mut self, value: u8) {
        self.bytes.push(value);
    }
    fn put_byte_string(&mut self, bytes: &[u8]) {
        self.put_count(bytes.len());
        self.bytes.extend_from_slice(bytes);
    }
    fn put_text(&mut self, text: &str) {
        self.put_byte_string(text.as_bytes());
    }
    fn put_optional(&mut self, value: Option<u64>) {
        match value {
            Some(present) => {
                self.put_byte(1);
                self.put_unsigned(present);
            }
            None => self.put_byte(0),
        }
    }
}

/// 从字节序列里按写入的次序读；读不下去交回说明，不 panic。
struct ByteSource<'bytes> {
    bytes: &'bytes [u8],
    position: usize,
}

impl<'bytes> ByteSource<'bytes> {
    fn take(&mut self, count: usize) -> Result<&'bytes [u8], String> {
        let end = self
            .position
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| format!("转储在偏移 {} 处不够读 {count} 字节", self.position))?;
        let taken = &self.bytes[self.position..end];
        self.position = end;
        Ok(taken)
    }
    fn unsigned(&mut self) -> Result<u64, String> {
        let taken = self.take(8)?;
        Ok(u64::from_le_bytes(taken.try_into().expect("取了 8 字节")))
    }
    fn count(&mut self) -> Result<usize, String> {
        let value = self.unsigned()?;
        usize::try_from(value).map_err(|_| format!("个数 {value} 装不进 usize"))
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn byte_string(&mut self) -> Result<&'bytes [u8], String> {
        let length = self.count()?;
        self.take(length)
    }
    fn text(&mut self) -> Result<String, String> {
        String::from_utf8(self.byte_string()?.to_vec())
            .map_err(|error| format!("文本不是 UTF-8：{error}"))
    }
    fn optional(&mut self) -> Result<Option<u64>, String> {
        match self.byte()? {
            0 => Ok(None),
            1 => Ok(Some(self.unsigned()?)),
            other => Err(format!("可选值的标记应是 0 或 1，读到 {other}")),
        }
    }
}

/// 步骤种类在转储里的编码；`Barrier` 不进写表，编码留着是为了穷举。
fn code_of_the_step_kind(kind: StepKind) -> u8 {
    match kind {
        StepKind::ZeroFill => 0,
        StepKind::Discard => 1,
        StepKind::UnitWrite => 2,
        StepKind::JournalRecord => 3,
        StepKind::RootRecordFua => 4,
        StepKind::SystemConfigurationSlot => 5,
        StepKind::Barrier => 6,
    }
}

fn step_kind_of_the_code(code: u8) -> Result<StepKind, String> {
    [
        StepKind::ZeroFill,
        StepKind::Discard,
        StepKind::UnitWrite,
        StepKind::JournalRecord,
        StepKind::RootRecordFua,
        StepKind::SystemConfigurationSlot,
        StepKind::Barrier,
    ]
    .into_iter()
    .find(|kind| code_of_the_step_kind(*kind) == code)
    .ok_or_else(|| format!("认不出的步骤种类编码 {code}"))
}

fn put_write(sink: &mut ByteSink, write: &RetainedWrite) {
    sink.put_unsigned(u64::from(write.device.0));
    sink.put_byte(code_of_the_step_kind(write.kind));
    sink.put_byte(u8::from(write.is_force_unit_access));
    sink.put_unsigned(write.offset.0);
    match &write.contents {
        WrittenContents::Bytes(bytes) => {
            sink.put_byte(0);
            sink.put_byte_string(bytes);
        }
        WrittenContents::Zeros { length } => {
            sink.put_byte(1);
            sink.put_unsigned(*length);
        }
    }
}

fn read_write(source: &mut ByteSource<'_>) -> Result<RetainedWrite, String> {
    let device = u32::try_from(source.unsigned()?).map_err(|_| "盘号装不进 u32".to_string())?;
    let kind = step_kind_of_the_code(source.byte()?)?;
    let is_force_unit_access = source.byte()? == 1;
    let offset = DeviceOffsetInBytes(source.unsigned()?);
    let contents = match source.byte()? {
        0 => WrittenContents::Bytes(source.byte_string()?.to_vec()),
        1 => WrittenContents::Zeros {
            length: source.unsigned()?,
        },
        other => return Err(format!("写的内容标记应是 0 或 1，读到 {other}")),
    };
    Ok(RetainedWrite {
        device: DeviceIdentity(device),
        kind,
        is_force_unit_access,
        offset,
        contents,
    })
}

fn put_writes(sink: &mut ByteSink, writes: &[RetainedWrite]) {
    sink.put_count(writes.len());
    for write in writes {
        put_write(sink, write);
    }
}

fn read_writes(source: &mut ByteSource<'_>) -> Result<Vec<RetainedWrite>, String> {
    let count = source.count()?;
    let mut writes = Vec::new();
    for _ in 0..count {
        writes.push(read_write(source)?);
    }
    Ok(writes)
}

fn put_segments(sink: &mut ByteSink, segments: &[Vec<usize>]) {
    sink.put_count(segments.len());
    for segment in segments {
        sink.put_count(segment.len());
        for write_index in segment {
            sink.put_count(*write_index);
        }
    }
}

fn read_segments(source: &mut ByteSource<'_>) -> Result<Vec<Vec<usize>>, String> {
    let segment_count = source.count()?;
    let mut segments = Vec::new();
    for _ in 0..segment_count {
        let write_count = source.count()?;
        let mut segment = Vec::new();
        for _ in 0..write_count {
            segment.push(source.count()?);
        }
        segments.push(segment);
    }
    Ok(segments)
}

/// 池里写过的每个扇区（盘号、扇区号、字节）逐个喂进去取的 SHA-256。
#[must_use]
pub fn pool_sectors_digest(pool: &MemoryPool) -> [u8; DIGEST_BYTES] {
    let mut message = Vec::new();
    for (device, image) in &pool.devices {
        for (sector, bytes) in image.written_sectors() {
            message.extend_from_slice(&device.0.to_le_bytes());
            message.extend_from_slice(&sector.to_le_bytes());
            message.extend_from_slice(bytes);
        }
    }
    sha256_digest(&message)
}

/// 一个池里写过的每个扇区各写成一次整扇区的写，次序按（盘号，扇区号）：叠到同样大小的空盘上得到同一个池。
/// 手写固定脚本与覆盖写链的基镜像从已建好的池取，不重放它的建池操作。
#[must_use]
pub fn base_writes_of_the_pool(pool: &MemoryPool) -> Vec<RetainedWrite> {
    let mut writes = Vec::new();
    for (device, image) in &pool.devices {
        for (sector, bytes) in image.written_sectors() {
            writes.push(RetainedWrite {
                device: *device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset: DeviceOffsetInBytes(sector * SECTOR_BYTES_OF_THE_BASE_IMAGE),
                contents: WrittenContents::Bytes(bytes.clone()),
            });
        }
    }
    writes
}

/// 基镜像按扇区存，扇区 512 字节（崩溃点重放的扇区取 512 字节，D13 已定项 4）。
pub const SECTOR_BYTES_OF_THE_BASE_IMAGE: u64 = 512;

impl FlowDump {
    /// 基镜像：空盘叠上 mkfs 的全部写。
    #[must_use]
    pub fn base_pool(&self) -> MemoryPool {
        let identities: Vec<DeviceIdentity> = self
            .device_identities
            .iter()
            .map(|identity| DeviceIdentity(*identity))
            .collect();
        let mut pool = MemoryPool::with_devices(&identities, self.device_bytes);
        pool.apply_writes(&self.base_writes);
        pool
    }

    /// 基镜像的扇区摘要（[`pool_sectors_digest`]）：转储里存一份，读回时重算核一遍。
    #[must_use]
    pub fn base_sectors_digest(&self) -> [u8; DIGEST_BYTES] {
        pool_sectors_digest(&self.base_pool())
    }

    /// mkfs 之后的写表与段的摘要：同一个种子两遍录出来的流要逐位相同（登记 X2）。
    #[must_use]
    pub fn write_table_digest(&self) -> String {
        let mut sink = ByteSink { bytes: Vec::new() };
        put_writes(&mut sink, &self.writes);
        put_segments(&mut sink, &self.segments);
        sha256_hexadecimal(&sink.bytes)
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut sink = ByteSink {
            bytes: DUMP_MAGIC.to_vec(),
        };
        sink.put_unsigned(DUMP_FORMAT_VERSION);
        sink.put_text(&self.family);
        sink.put_text(&self.label);
        sink.put_text(&self.tier);
        sink.put_optional(self.seed);
        sink.put_text(&self.ending);
        sink.put_unsigned(self.device_bytes);
        sink.put_count(self.device_identities.len());
        for identity in &self.device_identities {
            sink.put_unsigned(u64::from(*identity));
        }
        sink.put_count(self.operations_of_the_make_filesystem);
        sink.put_byte_string(&self.base_sectors_digest());
        put_writes(&mut sink, &self.base_writes);
        put_writes(&mut sink, &self.writes);
        put_segments(&mut sink, &self.segments);
        sink.put_optional(
            self.judged_root_index
                .map(|index| u64::try_from(index).expect("写表下标装得进 u64")),
        );
        sink.put_count(self.versions_with_a_file.len());
        for version in &self.versions_with_a_file {
            sink.put_unsigned(u64::from(version.instance.0));
            sink.put_unsigned(version.checkpoint_txg.0);
            sink.put_byte_string(&version.content);
        }
        sink.put_count(self.versions_without_a_file.len());
        for version in &self.versions_without_a_file {
            sink.put_unsigned(u64::from(version.instance.0));
            sink.put_unsigned(version.checkpoint_txg.0);
        }
        sink.put_optional(self.compaction_operations);
        sink.bytes.extend_from_slice(DUMP_TRAILER);
        sink.bytes
    }
}

impl FlowDump {
    /// 读回一份转储。魔数、版本、收尾、基镜像摘要、读完没有多余字节，任何一样不对都交回说明。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let body = bytes
            .strip_suffix(DUMP_TRAILER)
            .ok_or_else(|| "转储末尾没有 name=dump_done 收尾：没写完，或不是转储".to_string())?;
        let mut source = ByteSource {
            bytes: body,
            position: 0,
        };
        if source.take(DUMP_MAGIC.len())? != DUMP_MAGIC {
            return Err("转储魔数不对".to_string());
        }
        let version = source.unsigned()?;
        if version != DUMP_FORMAT_VERSION {
            return Err(format!(
                "转储格式版本 {version}，这个 bin 只认 {DUMP_FORMAT_VERSION}"
            ));
        }
        let family = source.text()?;
        let label = source.text()?;
        let tier = source.text()?;
        let seed = source.optional()?;
        let ending = source.text()?;
        let device_bytes = source.unsigned()?;
        let device_count = source.count()?;
        let mut device_identities = Vec::new();
        for _ in 0..device_count {
            device_identities
                .push(u32::try_from(source.unsigned()?).map_err(|_| "盘号装不进 u32".to_string())?);
        }
        let operations_of_the_make_filesystem = source.count()?;
        let stored_base_digest = source.byte_string()?.to_vec();
        let base_writes = read_writes(&mut source)?;
        let writes = read_writes(&mut source)?;
        let segments = read_segments(&mut source)?;
        let judged_root_index = source
            .optional()?
            .map(|index| usize::try_from(index).map_err(|_| "被判根的下标装不进 usize".to_string()))
            .transpose()?;
        let with_a_file_count = source.count()?;
        let mut versions_with_a_file = Vec::new();
        for _ in 0..with_a_file_count {
            let (instance, checkpoint_txg) = read_version_identity(&mut source)?;
            versions_with_a_file.push(PublishedVersion {
                instance,
                checkpoint_txg,
                content: source.byte_string()?.to_vec(),
            });
        }
        let without_a_file_count = source.count()?;
        let mut versions_without_a_file = Vec::new();
        for _ in 0..without_a_file_count {
            let (instance, checkpoint_txg) = read_version_identity(&mut source)?;
            versions_without_a_file.push(PublishedVersionWithoutAFile {
                instance,
                checkpoint_txg,
            });
        }
        let compaction_operations = source.optional()?;
        if source.position != body.len() {
            return Err(format!(
                "转储读完之后还剩 {} 字节",
                body.len() - source.position
            ));
        }
        let dump = FlowDump {
            family,
            label,
            tier,
            seed,
            ending,
            device_bytes,
            device_identities,
            operations_of_the_make_filesystem,
            base_writes,
            writes,
            segments,
            judged_root_index,
            versions_with_a_file,
            versions_without_a_file,
            compaction_operations,
        };
        if dump.base_sectors_digest().as_slice() != stored_base_digest.as_slice() {
            return Err("读回的基镜像摘要与转储里存的不同：基镜像写丢了或改了".to_string());
        }
        Ok(dump)
    }

    /// 写到 `path`：先写同目录的临时名、写完改名，被打断时目录里没有半截文件。
    pub fn write_atomically(&self, path: &Path) -> Result<(), String> {
        write_bytes_atomically(path, &self.to_bytes())
    }

    pub fn read_from(path: &Path) -> Result<Self, String> {
        let bytes =
            std::fs::read(path).map_err(|error| format!("读 {} 失败：{error}", path.display()))?;
        Self::from_bytes(&bytes).map_err(|error| format!("{}：{error}", path.display()))
    }
}

/// 先写同目录的临时名、同步、再改名。
pub fn write_bytes_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("partial");
    let mut file = std::fs::File::create(&temporary)
        .map_err(|error| format!("建 {} 失败：{error}", temporary.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("写 {} 失败：{error}", temporary.display()))?;
    file.sync_all()
        .map_err(|error| format!("同步 {} 失败：{error}", temporary.display()))?;
    std::fs::rename(&temporary, path).map_err(|error| {
        format!(
            "{} 改名为 {} 失败：{error}",
            temporary.display(),
            path.display()
        )
    })
}

fn read_version_identity(
    source: &mut ByteSource<'_>,
) -> Result<(InstanceGeneration, CheckpointTxg), String> {
    let instance =
        u32::try_from(source.unsigned()?).map_err(|_| "实例代号装不进 u32".to_string())?;
    Ok((
        InstanceGeneration(instance),
        CheckpointTxg(source.unsigned()?),
    ))
}

// ============================================================================================
// 录一段随机历史
// ============================================================================================

/// 比重组：命令行上写的名字，对到 `GenerationWeights` 的两个常量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeightsChoice {
    Broad,
    BroadWithCompactionWhileMounted,
}

impl WeightsChoice {
    pub fn parse(text: &str) -> Result<Self, String> {
        [Self::Broad, Self::BroadWithCompactionWhileMounted]
            .into_iter()
            .find(|choice| choice.command_line_name() == text)
            .ok_or_else(|| {
                format!(
                    "--weights 要写 BROAD 或 BROAD_WITH_COMPACTION_WHILE_MOUNTED，读到 {text:?}"
                )
            })
    }

    #[must_use]
    pub fn command_line_name(self) -> &'static str {
        match self {
            WeightsChoice::Broad => "BROAD",
            WeightsChoice::BroadWithCompactionWhileMounted => "BROAD_WITH_COMPACTION_WHILE_MOUNTED",
        }
    }

    #[must_use]
    pub fn weights(self) -> &'static GenerationWeights {
        match self {
            WeightsChoice::Broad => &GenerationWeights::BROAD,
            WeightsChoice::BroadWithCompactionWhileMounted => {
                &GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED
            }
        }
    }
}

/// 命令行上的盘宽：成员的 Debug 名字（`FourGibibytes`、`UnitAreaOf240Slots`……）。
pub fn parse_device_width(text: &str) -> Result<HistoryDeviceWidth, String> {
    HistoryDeviceWidth::ALL
        .into_iter()
        .find(|width| format!("{width:?}") == text)
        .ok_or_else(|| format!("--width 认不出 {text:?}"))
}

/// 录哪一段历史：步数、比重组、盘宽，加种子下标（种子 = 种子基 + 下标）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomHistoryRequest {
    pub steps: usize,
    pub weights: WeightsChoice,
    pub width: HistoryDeviceWidth,
    pub seed_index: u64,
}

impl RandomHistoryRequest {
    #[must_use]
    pub fn seed(&self) -> u64 {
        SEED_BASE_OF_THE_RANDOM_HISTORIES + self.seed_index
    }

    /// 流名：比重组、盘宽、步数、种子下标都在里面，文件名与块键文件名用它。
    #[must_use]
    pub fn label(&self) -> String {
        format!(
            "random_history-{}-{:?}-steps{:04}-seed{:02}",
            self.weights.command_line_name().to_lowercase(),
            self.width,
            self.steps,
            self.seed_index
        )
    }
}

fn ending_text(ending: &HistoryEnding) -> String {
    match ending {
        HistoryEnding::Completed => "Completed".to_string(),
        HistoryEnding::KnownRed { form, .. } => format!("KnownRed:form={form}"),
        HistoryEnding::NewFinding { .. } => "NewFinding".to_string(),
    }
}

/// 录一段随机历史：按种子生成、在两块 `width` 宽的内存盘上跑、录制流开内容保留，照 mkfs 占的步数把流切成基镜像那一截与写表那一截
/// （切段用 `writes_and_segments`，崩溃点重放同一条规则）。版本表取历史停下时模型根环里的每一版：有内容的进带文件的，没有的进没有文件的。
///
/// # Panics
/// 历史本身 panic（原样往上冒，由调用方记成 `dump_failed`）；录制流比 mkfs 占的步数还短。
#[must_use]
pub fn record_random_history(request: &RandomHistoryRequest) -> FlowDump {
    record_random_history_with_direct_base(request).0
}

/// 一段随机历史跑完之后：整条录制流、mkfs 占了其中几步、历史本身的收尾与版本表。
pub struct ExecutedHistory {
    pub operations: Vec<RetainedOperation>,
    pub make_filesystem_operations: usize,
    pub run: HistoryRun,
    /// 生成出来的历史里挂着时整理那一类操作的次数（数操作表，不数写流）。
    pub compaction_operations: usize,
}

/// 按种子生成历史、在两块 `width` 宽的内存盘上跑完、录制流开内容保留。
///
/// # Panics
/// 历史本身 panic（原样往上冒）；录制流比 mkfs 占的步数还短。
#[must_use]
pub fn execute_random_history(request: &RandomHistoryRequest) -> ExecutedHistory {
    let history = generate_history_with_weights(
        HistorySeed(request.seed()),
        request.steps,
        request.weights.weights(),
    );
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: request.width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_observation| {});
    let operations = stream.retained_operations();
    let make_filesystem_operations = run.operations_written_by_make_filesystem;
    assert!(
        make_filesystem_operations <= operations.len(),
        "录制流只有 {} 步，比 mkfs 占的 {make_filesystem_operations} 步还短",
        operations.len()
    );
    let compaction_operations = history
        .operations
        .iter()
        .filter(|operation| operation.kind() == HistoryOperationKind::CompactWhileMounted)
        .count();
    ExecutedHistory {
        operations,
        make_filesystem_operations,
        run,
        compaction_operations,
    }
}

/// 同 [`record_random_history`]，另交回「直接把录制流 mkfs 那几步叠到空盘上」得到的基镜像：转储里的基镜像是由切出来的写表重建的，
/// 两条路在同一条历史上的扇区摘要必须相同（登记 5.2「转储往返」，漏掉 mkfs 的清零写这一类错在这里现形）。
///
/// # Panics
/// 同 [`record_random_history`]。
#[must_use]
pub fn record_random_history_with_direct_base(
    request: &RandomHistoryRequest,
) -> (FlowDump, MemoryPool) {
    dump_of_the_executed_history(request, execute_random_history(request))
}

/// 跑完的历史切成转储，另交回直接叠出来的基镜像（见 [`record_random_history_with_direct_base`]）。
///
/// # Panics
/// 历史里模型的实例代号装不进 u32。
#[must_use]
pub fn dump_of_the_executed_history(
    request: &RandomHistoryRequest,
    executed: ExecutedHistory,
) -> (FlowDump, MemoryPool) {
    let ExecutedHistory {
        operations,
        make_filesystem_operations,
        run,
        compaction_operations,
    } = executed;
    let identities = [DeviceIdentity(0), DeviceIdentity(1)];
    let mut direct_base = MemoryPool::with_devices(&identities, request.width.device_bytes());
    direct_base.apply(&operations[..make_filesystem_operations]);
    let geometry = request.width.fixed_geometry();
    let (base_writes, _segments_of_the_make_filesystem) =
        writes_and_segments(&operations[..make_filesystem_operations], &geometry);
    let (writes, segments) =
        writes_and_segments(&operations[make_filesystem_operations..], &geometry);
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua);
    let mut versions_with_a_file = Vec::new();
    let mut versions_without_a_file = Vec::new();
    for (key, content) in &run.committed_versions_at_the_end {
        let instance =
            InstanceGeneration(u32::try_from(key.instance.0).expect("模型的实例代号装得进 u32"));
        let checkpoint_txg = CheckpointTxg(key.checkpoint_txg.0);
        match content {
            Some(bytes) => versions_with_a_file.push(PublishedVersion {
                instance,
                checkpoint_txg,
                content: bytes.clone(),
            }),
            None => versions_without_a_file.push(PublishedVersionWithoutAFile {
                instance,
                checkpoint_txg,
            }),
        }
    }
    let dump = FlowDump {
        family: "random_history".to_string(),
        label: request.label(),
        tier: "candidate".to_string(),
        seed: Some(request.seed()),
        ending: ending_text(&run.ending),
        device_bytes: request.width.device_bytes(),
        device_identities: vec![0, 1],
        operations_of_the_make_filesystem: make_filesystem_operations,
        base_writes,
        writes,
        segments,
        judged_root_index,
        versions_with_a_file,
        versions_without_a_file,
        compaction_operations: Some(
            u64::try_from(compaction_operations).expect("操作次数装得进 u64"),
        ),
    };
    (dump, direct_base)
}

// ============================================================================================
// 命令行：dump-random-histories
// ============================================================================================

/// 往标准输出打产物行：行首 `E7RESULT `，数着打了几行（收尾行报它）。
pub struct Emitter {
    emitted: u64,
}

impl Emitter {
    #[must_use]
    pub fn new() -> Self {
        Self { emitted: 0 }
    }

    pub fn emit(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }

    /// 收尾行：`emitted=` 是到这一行为止（含它）打了几行 `E7RESULT`。
    pub fn emit_closing_line(&mut self, name: &str, fields: &str) {
        let emitted_including_this_line = self.emitted + 1;
        self.emit(&format!(
            "name={name} {fields} emitted={emitted_including_this_line}"
        ));
    }
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DumpCommand {
    pub steps: usize,
    pub weights: WeightsChoice,
    pub width: HistoryDeviceWidth,
    pub seeds: u64,
    pub out_directory: PathBuf,
    pub check_determinism: bool,
}

/// 解析 `dump-random-histories` 后面的参数。
pub fn parse_dump_command(arguments: &[String]) -> Result<DumpCommand, String> {
    let mut steps = None;
    let mut weights = None;
    let mut width = None;
    let mut seeds = None;
    let mut out_directory = None;
    let mut check_determinism = false;
    let mut remaining = arguments.iter();
    while let Some(option) = remaining.next() {
        if option == "--check-determinism" {
            check_determinism = true;
            continue;
        }
        let value = remaining
            .next()
            .ok_or_else(|| format!("{option} 后面缺值"))?;
        match option.as_str() {
            "--steps" => steps = Some(parse_count(option, value)?),
            "--weights" => weights = Some(WeightsChoice::parse(value)?),
            "--width" => width = Some(parse_device_width(value)?),
            "--seeds" => {
                seeds = Some(u64::try_from(parse_count(option, value)?).expect("个数装得进 u64"));
            }
            "--out" => out_directory = Some(PathBuf::from(value)),
            other => return Err(format!("认不出的选项 {other}")),
        }
    }
    Ok(DumpCommand {
        steps: steps.ok_or("缺 --steps")?,
        weights: weights.ok_or("缺 --weights")?,
        width: width.ok_or("缺 --width")?,
        seeds: seeds.ok_or("缺 --seeds")?,
        out_directory: out_directory.ok_or("缺 --out")?,
        check_determinism,
    })
}

fn parse_count(option: &str, value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|error| format!("{option} 后面要一个十进制整数，读到 {value:?}：{error}"))
}

/// 一片（一条历史）做完的结局。
enum SliceOutcome {
    AlreadyDumped {
        file_digest: String,
        summary: String,
    },
    Recorded {
        file_digest: String,
        summary: String,
        /// 由转储里的写表重建的基镜像，与直接把录制流 mkfs 那几步叠到空盘上的基镜像逐扇区相同（登记 5.2「转储往返」）。
        base_matches_the_stream: bool,
    },
    Failed {
        message: String,
    },
}

fn panic_message_of(payload: &(dyn std::any::Any + Send)) -> String {
    let message = payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "panic 的内容不是文本".to_string());
    message.replace('\n', "\\n")
}

fn summary_of_the_dump(dump: &FlowDump, steps: usize) -> String {
    format!(
        "family={} label={} seed={} steps={steps} ending={} base_writes={} writes={} segments={} versions_with_a_file={} versions_without_a_file={} compaction_operations={}",
        dump.family,
        dump.label,
        dump.seed.map_or_else(|| "none".to_string(), |seed| seed.to_string()),
        dump.ending,
        dump.base_writes.len(),
        dump.writes.len(),
        dump.segments.len(),
        dump.versions_with_a_file.len(),
        dump.versions_without_a_file.len(),
        dump.compaction_operations
            .map_or_else(|| "none".to_string(), |count| count.to_string())
    )
}

fn dump_one_history(command: &DumpCommand, request: &RandomHistoryRequest) -> SliceOutcome {
    let path = command
        .out_directory
        .join(format!("{}.dump", request.label()));
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(existing) = FlowDump::from_bytes(&bytes) {
            return SliceOutcome::AlreadyDumped {
                file_digest: sha256_hexadecimal(&bytes),
                summary: summary_of_the_dump(&existing, request.steps),
            };
        }
    }
    let recorded = catch_unwind(AssertUnwindSafe(|| {
        record_random_history_with_direct_base(request)
    }));
    match recorded {
        Err(payload) => SliceOutcome::Failed {
            message: panic_message_of(payload.as_ref()),
        },
        Ok((dump, direct_base)) => {
            let bytes = dump.to_bytes();
            match write_bytes_atomically(&path, &bytes) {
                Err(message) => SliceOutcome::Failed { message },
                Ok(()) => SliceOutcome::Recorded {
                    file_digest: sha256_hexadecimal(&bytes),
                    summary: summary_of_the_dump(&dump, request.steps),
                    base_matches_the_stream: dump.base_pool() == direct_base,
                },
            }
        }
    }
}

/// 一条历史录两遍的结果：流名，加（第一遍的写表摘要，第二遍的写表摘要）或 panic 消息。
type DeterminismReading = (String, Result<(String, String), String>);

/// 种子下标 0 与 1 各再录两遍，两遍的写表摘要（[`FlowDump::write_table_digest`]）要逐位相同；交回（流名，第一遍，第二遍）。
fn determinism_readings(command: &DumpCommand) -> Vec<DeterminismReading> {
    (0..command.seeds.min(2))
        .map(|seed_index| {
            let request = RandomHistoryRequest {
                steps: command.steps,
                weights: command.weights,
                width: command.width,
                seed_index,
            };
            let recorded_twice = catch_unwind(AssertUnwindSafe(|| {
                (
                    record_random_history(&request).write_table_digest(),
                    record_random_history(&request).write_table_digest(),
                )
            }));
            (
                request.label(),
                recorded_twice.map_err(|payload| panic_message_of(payload.as_ref())),
            )
        })
        .collect()
}

/// `dump-random-histories`：按命令行录 `--seeds` 条历史、写转储、打产物行。交回退出码。
pub fn run_dump_command(arguments: &[String]) -> i32 {
    let command = match parse_dump_command(arguments) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("命令行不对：{message}");
            return EXIT_CODE_USAGE;
        }
    };
    if let Err(error) = std::fs::create_dir_all(&command.out_directory) {
        eprintln!("建 {} 失败：{error}", command.out_directory.display());
        return EXIT_CODE_USAGE;
    }
    let requests: Vec<RandomHistoryRequest> = (0..command.seeds)
        .map(|seed_index| RandomHistoryRequest {
            steps: command.steps,
            weights: command.weights,
            width: command.width,
            seed_index,
        })
        .collect();
    let outcomes: Mutex<BTreeMap<usize, SliceOutcome>> = Mutex::new(BTreeMap::new());
    let next_request = AtomicUsize::new(0);
    let emitter = Mutex::new(Emitter::new());
    let started = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..worker_threads_from_the_environment().min(requests.len().max(1)) {
            scope.spawn(|| loop {
                let request_index = next_request.fetch_add(1, Ordering::SeqCst);
                let Some(request) = requests.get(request_index) else {
                    break;
                };
                let outcome = dump_one_history(&command, request);
                let mut finished = outcomes.lock().expect("结局表的锁没被毒化");
                finished.insert(request_index, outcome);
                let done = finished.len();
                drop(finished);
                emitter.lock().expect("打印器的锁没被毒化").emit(&format!(
                    "name=progress flow={} done={done}/{} seconds={:.1}",
                    request.label(),
                    requests.len(),
                    started.elapsed().as_secs_f64()
                ));
            });
        }
    });
    let mut emitter = emitter.into_inner().expect("打印器的锁没被毒化");
    let outcomes = outcomes.into_inner().expect("结局表的锁没被毒化");
    let (mut recorded, mut skipped, mut failed, mut base_mismatches) = (0u64, 0u64, 0u64, 0u64);
    for (request_index, outcome) in &outcomes {
        let request = &requests[*request_index];
        match outcome {
            SliceOutcome::AlreadyDumped {
                file_digest,
                summary,
            } => {
                skipped += 1;
                emitter.emit(&format!(
                    "name=dump {summary} sha256={file_digest} reused=true"
                ));
            }
            SliceOutcome::Recorded {
                file_digest,
                summary,
                base_matches_the_stream,
            } => {
                recorded += 1;
                base_mismatches += u64::from(!base_matches_the_stream);
                emitter.emit(&format!(
                    "name=dump {summary} sha256={file_digest} reused=false base_matches_the_stream={base_matches_the_stream}"
                ));
            }
            SliceOutcome::Failed { message } => {
                failed += 1;
                emitter.emit(&format!(
                    "name=dump_failed label={} seed={} steps={} panic={message}",
                    request.label(),
                    request.seed(),
                    request.steps
                ));
            }
        }
    }
    let mut determinism_broken = false;
    if command.check_determinism {
        for (label, readings) in determinism_readings(&command) {
            match readings {
                Ok((recording, repeated_recording)) => {
                    determinism_broken |= recording != repeated_recording;
                    emitter.emit(&format!(
                        "name=determinism label={label} write_table={recording} write_table_of_the_repeated_recording={repeated_recording} equal={}",
                        recording == repeated_recording
                    ));
                }
                Err(message) => {
                    determinism_broken = true;
                    emitter.emit(&format!(
                        "name=determinism label={label} panic={message} equal=false"
                    ));
                }
            }
        }
    }
    emitter.emit_closing_line(
        "dump_done",
        &format!(
            "histories={} recorded={recorded} skipped={skipped} failed={failed} base_mismatches={base_mismatches}",
            recorded + skipped
        ),
    );
    if determinism_broken || base_mismatches > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else if failed > 0 {
        EXIT_CODE_HISTORY_NOT_RECORDED
    } else {
        EXIT_CODE_OK
    }
}

/// 强制跑的命令行开关：`preflight` 自己认，各子命令的参数解析看不到它。
const FORCE_FLAG: &str = "--force";

/// 摘掉命令行里的 `--force`，别的参数原样按原次序留下。
#[must_use]
pub fn arguments_without_the_force_flag(arguments: Vec<String>) -> Vec<String> {
    arguments
        .into_iter()
        .filter(|argument| argument != FORCE_FLAG)
        .collect()
}

/// 这个 bin 在 `src/bin/` 下的文件名（`preflight` 判它头上的准入与运行条件）。
const OWN_SOURCE_FILE_NAME: &str = "e184_regular_layer_cost_counts_write_dump.rs";

/// 退出码：准入与运行条件没满足（`preflight` 拒绝）是 78，别的非 0 值或没有退出码一律 1。
const EXIT_CODE_PREFLIGHT_REFUSED: i32 = 78;

/// `preflight.py check` 退出码不是 0 时本进程退几：78 原样，别的非 0 值（含被信号杀掉、没有退出码）退 1。
#[must_use]
pub fn exit_code_after_a_failed_check(check_exit_code: Option<i32>) -> i32 {
    if check_exit_code == Some(EXIT_CODE_PREFLIGHT_REFUSED) {
        EXIT_CODE_PREFLIGHT_REFUSED
    } else {
        1
    }
}

/// `preflight.py check` 标准输出的第一行：以 `forced` 起头（`forced<TAB><类><TAB><摘要>`）时交回最后一段摘要（空白换成下划线），
/// 以 `met`、`undeclared` 起头时交回 `None`。
#[must_use]
pub fn forced_summary_of_the_check_output(standard_output: &str) -> Option<String> {
    let check_line = standard_output.lines().next().unwrap_or("");
    if !check_line.starts_with("forced") {
        return None;
    }
    let summary = check_line.rsplit('\t').next().unwrap_or("");
    Some(summary.split_whitespace().collect::<Vec<_>>().join("_"))
}

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `python3 <规范副本>/scripts/preflight.py check <源文件> [--force] -- <参数…>`
/// （不经 `sh -c`），退出码不是 0 就按 [`exit_code_after_a_failed_check`] 退出；交回强制跑时的摘要，调用方写进产物。
/// `source_file_name` 是这个 bin 在 `src/bin/` 下的文件名；另一份 bin 引这个文件当模块时传它自己的文件名。
pub fn preflight(source_file_name: &str) -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/{source_file_name}");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == FORCE_FLAG) {
        command.arg(FORCE_FLAG);
    }
    command.arg("--");
    command.args(arguments_without_the_force_flag(arguments));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入与运行条件：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(EXIT_CODE_PREFLIGHT_REFUSED)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(exit_code_after_a_failed_check(output.status.code()));
    }
    forced_summary_of_the_check_output(&String::from_utf8_lossy(&output.stdout))
}

#[allow(
    dead_code,
    reason = "另一份 bin 经 #[path] 把这个文件当模块引进去时，入口在它那边，这里的 main 没人调"
)]
fn main() {
    let forced_summary = preflight(OWN_SOURCE_FILE_NAME);
    if let Some(summary) = forced_summary {
        println!("E7RESULT name=preflight forced={summary}");
    }
    let arguments = arguments_without_the_force_flag(std::env::args().skip(1).collect());
    match arguments.split_first() {
        Some((subcommand, rest)) if subcommand == "dump-random-histories" => {
            std::process::exit(run_dump_command(rest));
        }
        Some(_) | None => {
            eprintln!(
                "用法：e184_regular_layer_cost_counts_write_dump dump-random-histories --steps <步数> --weights <BROAD|BROAD_WITH_COMPACTION_WHILE_MOUNTED> --width <盘宽> --seeds <条数> --out <目录> [--check-determinism]"
            );
            std::process::exit(EXIT_CODE_USAGE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECTOR: usize = 512;

    fn bytes_of_one_sector(fill: u8) -> Vec<u8> {
        vec![fill; SECTOR]
    }

    fn write_of(
        device: u32,
        kind: StepKind,
        offset: u64,
        contents: WrittenContents,
    ) -> RetainedWrite {
        RetainedWrite {
            device: DeviceIdentity(device),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: DeviceOffsetInBytes(offset),
            contents,
        }
    }

    /// 手造的小转储：基镜像里盘 0 的第 0 扇区先写字节、再被清零（清零把扇区删掉），盘 1 有一个两扇区的写；
    /// 写表三条写分两段，最后一条是根槽写。
    fn small_dump() -> FlowDump {
        FlowDump {
            family: "smoke".to_string(),
            label: "smoke-small".to_string(),
            tier: "baseline".to_string(),
            seed: None,
            ending: "NotAHistory".to_string(),
            device_bytes: 1 << 20,
            device_identities: vec![0, 1],
            operations_of_the_make_filesystem: 3,
            base_writes: vec![
                write_of(
                    0,
                    StepKind::SystemConfigurationSlot,
                    0,
                    WrittenContents::Bytes(bytes_of_one_sector(0xAB)),
                ),
                write_of(
                    0,
                    StepKind::ZeroFill,
                    0,
                    WrittenContents::Zeros { length: 512 },
                ),
                write_of(
                    1,
                    StepKind::UnitWrite,
                    4096,
                    WrittenContents::Bytes(vec![0xCD; 2 * SECTOR]),
                ),
            ],
            writes: vec![
                write_of(
                    0,
                    StepKind::UnitWrite,
                    8192,
                    WrittenContents::Bytes(bytes_of_one_sector(0x01)),
                ),
                write_of(
                    1,
                    StepKind::JournalRecord,
                    8192,
                    WrittenContents::Bytes(bytes_of_one_sector(0x02)),
                ),
                write_of(
                    0,
                    StepKind::RootRecordFua,
                    12288,
                    WrittenContents::Bytes(bytes_of_one_sector(0x03)),
                ),
            ],
            segments: vec![vec![0, 1], vec![2]],
            judged_root_index: Some(2),
            versions_with_a_file: vec![PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: vec![7, 8, 9],
            }],
            versions_without_a_file: vec![PublishedVersionWithoutAFile {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            }],
            compaction_operations: Some(3),
        }
    }

    #[test]
    fn dump_round_trips_through_bytes_and_keeps_the_zero_fill_that_follows_bytes_in_the_base_image()
    {
        let dump = small_dump();
        let read_back = FlowDump::from_bytes(&dump.to_bytes()).expect("手造的转储读得回");
        assert_eq!(read_back, dump);
        let base = read_back.base_pool();
        assert!(
            base.devices[&DeviceIdentity(0)]
                .written_sectors()
                .is_empty(),
            "盘 0 第 0 扇区先写字节、后被清零：清零把它删掉，读回的基镜像上盘 0 没有写过的扇区"
        );
        assert_eq!(
            base.devices[&DeviceIdentity(1)].written_sectors().len(),
            2,
            "盘 1 的两扇区写读回之后还是两个扇区"
        );
        assert_eq!(read_back.base_sectors_digest(), dump.base_sectors_digest());
    }

    #[test]
    fn dump_without_the_done_trailer_is_refused() {
        let mut bytes = small_dump().to_bytes();
        bytes.truncate(bytes.len() - DUMP_TRAILER.len());
        let refusal = FlowDump::from_bytes(&bytes).expect_err("没有收尾的转储要拒");
        assert!(refusal.contains("name=dump_done"), "{refusal}");
    }

    #[test]
    fn dump_with_another_magic_or_extra_bytes_is_refused() {
        let mut wrong_magic = small_dump().to_bytes();
        wrong_magic[0] ^= 0xFF;
        let magic_refusal = FlowDump::from_bytes(&wrong_magic).expect_err("魔数不对要拒");
        assert!(magic_refusal.contains("魔数"), "{magic_refusal}");
        let mut extra = small_dump().to_bytes();
        let trailer_start = extra.len() - DUMP_TRAILER.len();
        extra.insert(trailer_start, 0);
        let extra_refusal = FlowDump::from_bytes(&extra).expect_err("收尾前多一个字节要拒");
        assert!(extra_refusal.contains("还剩 1 字节"), "{extra_refusal}");
    }

    #[test]
    fn dump_whose_base_image_bytes_changed_after_the_digest_was_stored_is_refused() {
        let mut bytes = small_dump().to_bytes();
        let marker = vec![0xCD; 2 * SECTOR];
        let position = bytes
            .windows(marker.len())
            .position(|window| window == marker.as_slice())
            .expect("基镜像里盘 1 的两扇区写在转储里原样存着");
        bytes[position] ^= 0x01;
        let refusal = FlowDump::from_bytes(&bytes).expect_err("基镜像的字节变了，存着的摘要对不上");
        assert!(refusal.contains("基镜像摘要"), "{refusal}");
    }

    #[test]
    fn the_write_table_digest_changes_when_a_write_or_a_segment_changes() {
        let dump = small_dump();
        let mut other_segments = dump.clone();
        other_segments.segments = vec![vec![0], vec![1, 2]];
        assert_ne!(
            dump.write_table_digest(),
            other_segments.write_table_digest()
        );
        let mut other_write = dump.clone();
        other_write.writes[0] = write_of(
            0,
            StepKind::UnitWrite,
            8192,
            WrittenContents::Bytes(bytes_of_one_sector(0x09)),
        );
        assert_ne!(dump.write_table_digest(), other_write.write_table_digest());
        let mut other_base = dump.clone();
        other_base.base_writes.pop();
        assert_eq!(
            dump.write_table_digest(),
            other_base.write_table_digest(),
            "写表摘要只看 mkfs 之后的写与段，不看基镜像"
        );
    }

    #[test]
    fn the_base_writes_of_a_pool_rebuild_the_same_pool() {
        let pool = small_dump().base_pool();
        let rebuilt = {
            let identities = [DeviceIdentity(0), DeviceIdentity(1)];
            let mut pool_again = MemoryPool::with_devices(&identities, pool.device_size_in_bytes);
            pool_again.apply_writes(&base_writes_of_the_pool(&pool));
            pool_again
        };
        assert_eq!(rebuilt, pool);
        assert_eq!(pool_sectors_digest(&rebuilt), pool_sectors_digest(&pool));
        assert_eq!(
            base_writes_of_the_pool(&pool).len(),
            2,
            "盘 1 的两个扇区各一条写，盘 0 被清零之后什么都没有"
        );
    }

    #[test]
    fn worker_threads_take_the_own_variable_first_then_the_thread_cap_then_the_registration() {
        let registration = "# 注释\n\n31  # 理由\n";
        assert_eq!(worker_threads_given(Some("4"), Some("8"), registration), 4);
        assert_eq!(worker_threads_given(None, Some("8"), registration), 8);
        assert_eq!(worker_threads_given(None, Some(""), registration), 31);
        assert_eq!(worker_threads_given(None, None, registration), 31);
    }

    #[test]
    #[should_panic(expected = "E184_THREADS 要是正整数")]
    fn worker_threads_refuse_zero_in_the_own_variable() {
        let _ = worker_threads_given(Some("0"), None, "31  # 理由\n");
    }

    #[test]
    #[should_panic(expected = "不是以正整数起头")]
    fn worker_threads_refuse_a_registration_line_that_does_not_start_with_a_number() {
        let _ = worker_threads_given(None, None, "# 注释\nabc  # 理由\n");
    }

    #[test]
    fn the_registered_thread_count_compiled_in_is_a_positive_number() {
        assert!(worker_threads_given(None, None, HEAVY_THREAD_REGISTRATION_TEXT) > 0);
    }

    #[test]
    fn the_dump_command_line_needs_every_option_and_refuses_unknown_ones() {
        let arguments = |words: &[&str]| -> Vec<String> {
            words.iter().map(|word| (*word).to_string()).collect()
        };
        let complete = arguments(&[
            "--steps",
            "20",
            "--weights",
            "BROAD",
            "--width",
            "FourGibibytes",
            "--seeds",
            "2",
            "--out",
            "/tmp/somewhere",
            "--check-determinism",
        ]);
        let command = parse_dump_command(&complete).expect("完整的命令行");
        assert_eq!(command.steps, 20);
        assert_eq!(command.weights, WeightsChoice::Broad);
        assert_eq!(command.width, HistoryDeviceWidth::FourGibibytes);
        assert_eq!(command.seeds, 2);
        assert!(command.check_determinism);
        assert!(
            parse_dump_command(&complete[..complete.len() - 3]).is_err(),
            "缺 --out"
        );
        let unknown = arguments(&["--steps", "20", "--nonsense", "1"]);
        assert!(parse_dump_command(&unknown)
            .expect_err("认不出的选项")
            .contains("--nonsense"));
        let bad_weights = arguments(&["--weights", "NARROW"]);
        assert!(parse_dump_command(&bad_weights)
            .expect_err("比重组认不出")
            .contains("NARROW"));
    }

    fn request_of(steps: usize, seed_index: u64) -> RandomHistoryRequest {
        RandomHistoryRequest {
            steps,
            weights: WeightsChoice::Broad,
            width: HistoryDeviceWidth::FourGibibytes,
            seed_index,
        }
    }

    #[test]
    fn the_seed_of_index_zero_is_the_seed_base_and_each_index_adds_exactly_its_own_number() {
        assert_eq!(request_of(20, 0).seed(), 7_463_871_032_432_355_113);
        assert_eq!(request_of(20, 3).seed(), 7_463_871_032_432_355_116);
        assert_eq!(
            SEED_BASE_OF_THE_RANDOM_HISTORIES,
            singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            "本地的种子基与 crates 的同名常量相等（登记停机 T1）"
        );
    }

    #[test]
    fn the_label_names_weights_width_steps_and_the_seed_index() {
        assert_eq!(
            request_of(100, 7).label(),
            "random_history-broad-FourGibibytes-steps0100-seed07"
        );
        let with_compaction = RandomHistoryRequest {
            weights: WeightsChoice::BroadWithCompactionWhileMounted,
            width: HistoryDeviceWidth::UnitAreaOf240Slots,
            ..request_of(50, 12)
        };
        assert_eq!(
            with_compaction.label(),
            "random_history-broad_with_compaction_while_mounted-UnitAreaOf240Slots-steps0050-seed12"
        );
    }

    #[test]
    fn recorded_history_dump_starts_after_the_make_filesystem_and_its_base_image_matches_the_stream_applied_directly(
    ) {
        let request = request_of(20, 0);
        let executed = execute_random_history(&request);
        let geometry = request.width.fixed_geometry();
        let (all_writes, _all_segments) = writes_and_segments(&executed.operations, &geometry);
        let (dump, direct_base) = dump_of_the_executed_history(&request, executed);
        assert_eq!(
            dump.writes.len() + dump.base_writes.len(),
            all_writes.len(),
            "mkfs 之后的写表加上基镜像的写，正好是整条录制流的写：切口不多不少落在 mkfs 做完那一刻"
        );
        assert!(!dump.base_writes.is_empty(), "mkfs 有写");
        assert!(!dump.writes.is_empty(), "20 步历史在 mkfs 之后有写");
        assert_eq!(
            dump.segments.iter().map(Vec::len).sum::<usize>(),
            dump.writes.len(),
            "每次写恰好在一段里"
        );
        assert_eq!(
            dump.base_pool(),
            direct_base,
            "由切出来的写表重建的基镜像与直接叠录制流的基镜像逐扇区相同"
        );
        assert!(
            dump.base_writes
                .iter()
                .any(|write| matches!(write.contents, WrittenContents::Zeros { .. })),
            "mkfs 的写里有清零写，基镜像的重建经过它们"
        );
        assert_eq!(dump.seed, Some(SEED_BASE_OF_THE_RANDOM_HISTORIES));
        assert_eq!(dump.family, "random_history");
        assert_eq!(
            dump.compaction_operations,
            Some(0),
            "BROAD 比重表里没有挂着时整理这一类操作（锚 A11 的独立一路）"
        );
        let read_back = FlowDump::from_bytes(&dump.to_bytes()).expect("读得回");
        assert_eq!(read_back, dump);
    }

    #[test]
    fn two_recordings_of_one_seed_have_the_same_write_table_and_two_seeds_do_not() {
        let recording = record_random_history(&request_of(20, 0)).write_table_digest();
        let repeated_recording = record_random_history(&request_of(20, 0)).write_table_digest();
        let other_seed = record_random_history(&request_of(20, 1)).write_table_digest();
        assert_eq!(
            recording, repeated_recording,
            "同一个种子录两遍，写表摘要逐位相同（登记 X2）"
        );
        assert_ne!(
            recording, other_seed,
            "两个种子的写表摘要分得开：摘要真在看写表"
        );
    }

    #[test]
    fn dump_run_writes_one_file_per_seed_and_a_second_run_skips_them() {
        let directory = std::env::temp_dir().join(format!(
            "e184-write-dump-test-{}-{}",
            std::process::id(),
            "dump-run"
        ));
        let _ = std::fs::remove_dir_all(&directory);
        let arguments: Vec<String> = [
            "--steps",
            "20",
            "--weights",
            "BROAD",
            "--width",
            "FourGibibytes",
            "--seeds",
            "2",
            "--out",
            directory.to_str().expect("临时目录的路径是 UTF-8"),
        ]
        .iter()
        .map(|word| (*word).to_string())
        .collect();
        assert_eq!(run_dump_command(&arguments), EXIT_CODE_OK);
        let mut names: Vec<String> = std::fs::read_dir(&directory)
            .expect("输出目录在")
            .map(|entry| {
                entry
                    .expect("目录项")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        assert_eq!(
            names,
            vec![
                "random_history-broad-FourGibibytes-steps0020-seed00.dump".to_string(),
                "random_history-broad-FourGibibytes-steps0020-seed01.dump".to_string(),
            ],
            "两条历史两份转储，没有留下临时名"
        );
        let file_before_the_second_run = std::fs::read(directory.join(&names[0])).expect("读得到");
        assert_eq!(run_dump_command(&arguments), EXIT_CODE_OK, "已有的整条跳过");
        assert_eq!(
            std::fs::read(directory.join(&names[0])).expect("读得到"),
            file_before_the_second_run
        );
        std::fs::remove_dir_all(&directory).expect("清掉临时目录");
    }

    #[test]
    fn a_forced_check_line_gives_its_last_segment_with_the_blanks_joined_by_underscores() {
        assert_eq!(
            forced_summary_of_the_check_output("forced\tadmission\t输入 没变 两条\n"),
            Some("输入_没变_两条".to_string())
        );
        assert_eq!(
            forced_summary_of_the_check_output("forced\trun-condition\tpython3\nmet\tabcd\n"),
            Some("python3".to_string()),
            "只看标准输出的第一行"
        );
    }

    #[test]
    fn a_check_line_that_is_not_forced_gives_no_summary() {
        assert_eq!(forced_summary_of_the_check_output("met\t0123abcd\n"), None);
        assert_eq!(forced_summary_of_the_check_output("undeclared\n"), None);
        assert_eq!(forced_summary_of_the_check_output(""), None);
        assert_eq!(
            forced_summary_of_the_check_output("met\t0123\nforced\tx\ty\n"),
            None,
            "forced 不在第一行不算"
        );
    }

    #[test]
    fn a_refused_check_exits_78_and_every_other_failure_exits_1() {
        assert_eq!(exit_code_after_a_failed_check(Some(78)), 78);
        assert_eq!(exit_code_after_a_failed_check(Some(1)), 1);
        assert_eq!(exit_code_after_a_failed_check(Some(2)), 1);
        assert_eq!(exit_code_after_a_failed_check(Some(77)), 1);
        assert_eq!(
            exit_code_after_a_failed_check(None),
            1,
            "被信号杀掉没有退出码"
        );
    }

    #[test]
    fn the_force_flag_is_dropped_wherever_it_stands_and_the_other_arguments_keep_their_order() {
        let words = |list: &[&str]| -> Vec<String> {
            list.iter().map(|word| (*word).to_string()).collect()
        };
        assert_eq!(
            arguments_without_the_force_flag(words(&[
                "dump-random-histories",
                "--force",
                "--steps",
                "100"
            ])),
            words(&["dump-random-histories", "--steps", "100"])
        );
        assert_eq!(
            arguments_without_the_force_flag(words(&["--force", "--force"])),
            Vec::<String>::new()
        );
        assert_eq!(
            arguments_without_the_force_flag(words(&["--forced", "force"])),
            words(&["--forced", "force"]),
            "只摘整词 --force"
        );
    }

    #[test]
    fn the_own_source_file_name_points_at_a_file_in_the_source_directory_of_the_binaries() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/bin")
            .join(OWN_SOURCE_FILE_NAME);
        let text = std::fs::read_to_string(&path).expect("preflight 要判的源文件读得到");
        assert!(
            text.contains("// admission:") && text.contains("// run-condition:"),
            "{} 的文件头要有准入与运行条件",
            path.display()
        );
    }
}
