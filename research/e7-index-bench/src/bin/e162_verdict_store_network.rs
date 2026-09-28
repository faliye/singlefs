//! E162（崩溃放量判定块存储选型）第三段 S3：两机共用一个库时，经网络送块跟不跟得上、断线重连之后库里的块与送出的块一致不一致。
//!
// admission: always 每跑一次都是新的中断与计时观测，同一份代码两次跑的计数与时延本来就不同
// run-condition: command python3 stat df
//!
//! 跑前登记：`research/prompts/e162-preregistration.md`：6.3（Q3a–Q3j、帧与中断）、5.3 的 PC3-drop、PC3-torn、PC3-rate、
//! 5.5 的 S3-T、S3-C、S3-W1、S3-bs10，补 5.1（S3-W256、S3-raw、跨机格的接法）、补 6（Q3k、Q3r）、补 8、补 9 的 M27–M29，
//! 第十二节「修订」里第三段那几行。
//!
//! 独立手写模型，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）。候选的读写路径照登记 5.2
//! 逐字实现，从 `e162_crash_verdict_block_store.rs` 拷来（出处行号写在修订里）；帧、块值、块键与锚点用送块那个 bin 的同一份
//! （`#[path]` 引进来的 `sender_role` 模块）。
//!
//! 角色：父进程（`anchors`、`s3 <臂> [格…]`、`discrimination <Q3a> <Q3a>`）按格驱动；库进程（`library`）、回环转发进程
//! （`forward`）、核对进程（`verify`）是这个 bin 按角色参数起的子进程；送块进程是同一个目录下的 `e162-verdict-store-sender`。
//! 跨机格的数据通道（库进程自己起 `<对端命令> <中继命令>` 当数据通道、回出第 a_k 个确认之后杀它）写在库进程里；
//! 跨机格在另一台上放文件、起送块进程与收尾（补 5.1「B 上放什么」）这一版没写，见修订。
//!
//! 计时都在被测进程里用 `Instant` 计（送块进程计 Q3a、库进程的直写线程计 Q3b），父进程读子进程的行只用来排中断，
//! 不打时间戳（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）。

use e7_index_bench::Emitter;
use redb::ReadableDatabase;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

#[path = "e162_verdict_store_sender.rs"]
#[allow(dead_code, reason = "送块二进制的入口与只有它自己用的几样，在这个 bin 里只有单测自起送块角色时用")]
mod sender_role;

use sender_role::{
    anchor_checks, block_frame_checksum, block_value, compress_true_positions, encode_acknowledgement_frame, hexadecimal,
    input_fingerprint, interleaved_block_key, network_path_block_index, read_until_full_or_end, BlockKey, SenderEndpoint,
    SenderSettings, SplitMix64, ValuePattern, ACKNOWLEDGEMENTS_PER_WINDOW, BLOCK_FRAME_HEADER_BYTES, BLOCK_FRAME_MAGIC,
    BLOCK_KEY_BYTES, DEFAULT_ACKNOWLEDGEMENT_STALL_SECONDS, FRAME_CHECKSUM_BYTES, PATH_BLOCK_COUNT, PRIMARY_STATES_PER_BLOCK,
    SEED_NETWORK_BLOCKS, SMALL_STATES_PER_BLOCK,
};

// ============================== 登记写死的常量 ==============================

/// seed_NET（登记 5.1）：中断点 a_k、u_k 与 PC3-drop 挑帧各从它起一个 SplitMix64。
const SEED_NETWORK_INTERRUPTIONS: u64 = 0xE162_0000_0000_0009;
/// S3-C、PC3-torn 的中断次数（登记 5.3、6.3 末段）；V12、V14 打中时加到 200。
const REGISTERED_INTERRUPTIONS: usize = 50;
const ESCALATED_INTERRUPTIONS: usize = 200;
/// 中断之后停 u × 200 ms 再起（登记 6.3 末段）。
const INTERRUPTION_PAUSE_UNIT_MILLISECONDS: f64 = 200.0;
/// PC3-drop：每 1 000 帧挑一帧只回确认、不写（登记 5.3）。
const DROP_CONTROL_FRAMES_PER_PICK: u64 = 1_000;
/// PC3-rate：送 2 000 块两遍，慢的那遍每帧提交之前睡 5 ms；慢的 ≤ 200 块/秒、快的 > 200 块/秒（登记 5.3）。
const RATE_CONTROL_BLOCKS: u64 = 2_000;
const RATE_CONTROL_SLEEP: Duration = Duration::from_millis(5);
const RATE_CONTROL_THRESHOLD_BLOCKS_PER_SECOND: f64 = 200.0;
/// 窗口：主 64，敏感性 1 与 256（登记 6.3 末段、补 5.1）。
const PRIMARY_WINDOW_FRAMES: usize = 64;
const STOP_AND_WAIT_WINDOW_FRAMES: usize = 1;
const WIDE_WINDOW_FRAMES: usize = 256;
/// 核对子进程 3 600 秒没交回就杀（登记 5.5，同 S1）。
const VERIFICATION_TIMEOUT_SECONDS: u64 = 3_600;
/// V15 与修订五：每 30 秒看一次、最多等 20 分钟，最多重跑三次。
const INTERFERENCE_POLL_SECONDS: u64 = 30;
const INTERFERENCE_MAXIMUM_WAIT_SECONDS: u64 = 1_200;
const INTERFERENCE_MAXIMUM_RERUNS: usize = 3;
/// H4：开跑前库所在文件系统可用 < 40 GiB 不开跑。
const MINIMUM_AVAILABLE_BYTES: u64 = 40 << 30;
const FORWARDER_BIND_DEADLINE: Duration = Duration::from_secs(5);
const CHILD_START_DEADLINE: Duration = Duration::from_secs(600);
const SENDER_BINARY_NAME: &str = "e162-verdict-store-sender";

/// 候选 F（登记 5.2）：文件头 48 字节；FORMAT 16 字节。
const FILE_MAGIC: &[u8; 4] = b"SFCB";
const FILE_FORMAT_VERSION: u32 = 1;
const FILE_HEADER_BYTES: usize = 48;
const FILE_HEADER_CHECKSUMMED_BYTES: usize = 44;
const FORMAT_MAGIC: &[u8; 8] = b"SFE162FB";
const FORMAT_FILE_BYTES: usize = 16;
const FORMAT_FILE_NAME: &str = "FORMAT";

const REDB_TABLE: redb::TableDefinition<&[u8], &[u8]> = redb::TableDefinition::new("blocks");
const REDB_FILE_NAME: &str = "blocks.redb";

// ============================== 准入、入口、产物行 ==============================

/// 子进程角色的第一个参数；带着它起的是装置自己起的子进程，不再判准入。
const CHILD_ROLE_ARGUMENTS: [&str; 3] = ["library", "forward", "verify"];

fn preflight() -> Option<String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|role| CHILD_ROLE_ARGUMENTS.contains(&role.as_str()))
    {
        return None;
    }
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e162_verdict_store_network.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
    let first_line = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").to_string();
    first_line.strip_prefix("forced").map(|summary| summary.trim().replace(' ', "_"))
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    let mode = arguments.first().map(String::as_str).unwrap_or("");
    let remaining: Vec<String> = arguments.iter().skip(1).cloned().collect();
    let exit_code = match mode {
        "library" => run_library_role(&remaining),
        "forward" => run_forward_role(&remaining),
        "verify" => run_verify_role(&remaining),
        "anchors" | "s3" | "discrimination" => {
            let mut output = ProductLines::new();
            if let Some(summary) = forced_summary {
                output.line(format!("name=preflight forced={summary}"));
            }
            let code = match mode {
                "anchors" => run_anchors(&mut output),
                "s3" => run_loopback_question(&mut output, &remaining),
                _ => run_discrimination(&mut output, &remaining),
            };
            output.finish();
            code
        }
        _ => {
            eprintln!("  ✗ 用法：e162-verdict-store-network anchors | s3 <臂> [格…] | discrimination <Q3a 状态每秒> <Q3a 状态每秒>");
            eprintln!("  → 怎么办：臂取 R1 R0 K F；格取 PC3-drop PC3-rate PC3-torn S3-T S3-C S3-W1 S3-W256 S3-bs10 S3-raw（不写就按这个次序全跑）；");
            eprintln!("     library/forward/verify 是装置自己起的子进程角色；送块进程 e162-verdict-store-sender 要先编好、放在同一个目录");
            2
        }
    };
    std::process::exit(exit_code);
}

/// 产物行：每行 `E7RESULT ` 起头，收尾 `name=done emitted=N`（N 连 done 行自己都算）。
struct ProductLines {
    emitter: Emitter,
}

impl ProductLines {
    fn new() -> Self {
        Self { emitter: Emitter::new() }
    }

    fn line(&mut self, body: String) {
        let text = self.emitter.emit_raw(&body);
        let mut standard_output = std::io::stdout().lock();
        writeln!(standard_output, "{text}").expect("写得进标准输出");
        standard_output.flush().expect("刷得出标准输出");
    }

    fn finish(&mut self) {
        let text = self.emitter.finish();
        println!("{text}");
    }
}

// ============================== 锚点 ==============================

/// 送块那个 bin 的锚点（A1–A6、A11 的帧长、B10、B10b），加上只在这个 bin 里有的 A11 后半（F 的块文件长）与 A10。
fn network_anchor_lines() -> Vec<(String, String, String, bool)> {
    let mut lines: Vec<(String, String, String, bool)> = anchor_checks()
        .into_iter()
        .map(|check| {
            let matches = check.matches();
            (check.identifier, check.computed, check.registered.to_string(), matches)
        })
        .collect();
    let file_length = (FILE_HEADER_BYTES + 65_536).to_string();
    let file_matches = file_length == "65584";
    lines.push(("A11_file".to_string(), file_length, "65584".to_string(), file_matches));
    let path_blocks = 2 * PATH_BLOCK_COUNT;
    let value_bytes = format!("{}/{}", path_blocks * PRIMARY_STATES_PER_BLOCK, path_blocks * SMALL_STATES_PER_BLOCK);
    let value_matches = value_bytes == "6553600000/102400000";
    lines.push(("A10".to_string(), value_bytes, "6553600000/102400000".to_string(), value_matches));
    lines
}

fn run_anchors(output: &mut ProductLines) -> i32 {
    let lines = network_anchor_lines();
    let mismatches = lines.iter().filter(|(_, _, _, matches)| !matches).count();
    for (identifier, computed, _registered, matches) in &lines {
        output.line(format!("name=anchor id={identifier} value={computed} matches_registration={matches}"));
    }
    output.line(format!("name=verdict part=anchors anchors_checked={} anchor_mismatches={mismatches}", lines.len()));
    i32::from(mismatches > 0)
}

// ============================== 候选（登记 5.2，从 S1 装置拷来） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Candidate {
    /// R1：redb，每个写事务 Durability::Immediate + quick-repair。
    RedbQuickRepair,
    /// R0：redb，Durability::Immediate，quick-repair 关（默认）。
    RedbDefault,
    /// K：RocksDB，LZ4，同步写、WAL 开。
    RocksDatabase,
    /// F：加固的文件，一块一个文件，临时文件 → fsync → 改名 → 目录 fsync。
    HardenedFile,
}

impl Candidate {
    fn label(self) -> &'static str {
        match self {
            Candidate::RedbQuickRepair => "R1",
            Candidate::RedbDefault => "R0",
            Candidate::RocksDatabase => "K",
            Candidate::HardenedFile => "F",
        }
    }

    fn parse(text: &str) -> Option<Candidate> {
        match text {
            "R1" => Some(Candidate::RedbQuickRepair),
            "R0" => Some(Candidate::RedbDefault),
            "K" => Some(Candidate::RocksDatabase),
            "F" => Some(Candidate::HardenedFile),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum StoreError {
    Redb(String),
    RocksDatabase(String),
    File(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Redb(message) => write!(formatter, "redb: {message}"),
            StoreError::RocksDatabase(message) => write!(formatter, "rocksdb: {message}"),
            StoreError::File(message) => write!(formatter, "file: {message}"),
        }
    }
}

fn redb_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::Redb(error.to_string())
}

fn rocksdb_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::RocksDatabase(error.to_string())
}

fn file_error(context: &str, error: impl std::fmt::Display) -> StoreError {
    StoreError::File(format!("{context}: {error}"))
}

/// 读一块的三种结局（登记 5.1 的三列）。
#[derive(Debug, PartialEq, Eq)]
enum ReadOutcome {
    Found(Vec<u8>),
    NotFound,
    ReadError(String),
}

fn rocksdb_options() -> rocksdb::Options {
    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);
    options.set_compression_type(rocksdb::DBCompressionType::Lz4);
    options
}

/// K 的写选项：同步写、WAL 开（登记 5.2）。
fn rocksdb_write_options() -> rocksdb::WriteOptions {
    let mut write_options = rocksdb::WriteOptions::default();
    write_options.disable_wal(false);
    write_options.set_sync(true);
    write_options
}

/// 一个线程手里的库：redb 与 RocksDB 两个线程共用一个打开的库（redb 的写事务由它自己排队、
/// RocksDB 两个线程并发 `put_opt`），F 每个线程各开一份、各写各的文件（登记 6.3 末段）。
enum StoreHandle {
    Redb(Arc<redb::Database>),
    RocksDatabase(Arc<rocksdb::DB>),
    File(FileLibrary),
}

struct ThreadStore {
    candidate: Candidate,
    handle: StoreHandle,
}

/// 写入用的打开：库不在就建；返回 thread_count 个共用同一个库的句柄。
fn open_thread_stores(candidate: Candidate, library: &Path, thread_count: usize) -> Result<Vec<ThreadStore>, StoreError> {
    let mut stores = Vec::with_capacity(thread_count);
    match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            fs::create_dir_all(library).map_err(|error| file_error("建 redb 库目录", error))?;
            let database = Arc::new(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?);
            for _thread in 0..thread_count {
                stores.push(ThreadStore { candidate, handle: StoreHandle::Redb(Arc::clone(&database)) });
            }
        }
        Candidate::RocksDatabase => {
            let database = Arc::new(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?);
            for _thread in 0..thread_count {
                stores.push(ThreadStore { candidate, handle: StoreHandle::RocksDatabase(Arc::clone(&database)) });
            }
        }
        Candidate::HardenedFile => {
            for _thread in 0..thread_count {
                stores.push(ThreadStore { candidate, handle: StoreHandle::File(FileLibrary::open(library, true)?) });
            }
        }
    }
    Ok(stores)
}

/// 核对用的打开：同一个打开调用（redb 的 `Database::create`、RocksDB 的 `DB::open`），F 不建新库。
fn open_existing_store(candidate: Candidate, library: &Path) -> Result<ThreadStore, StoreError> {
    let handle = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            StoreHandle::Redb(Arc::new(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?))
        }
        Candidate::RocksDatabase => StoreHandle::RocksDatabase(Arc::new(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?)),
        Candidate::HardenedFile => StoreHandle::File(FileLibrary::open(library, false)?),
    };
    Ok(ThreadStore { candidate, handle })
}

impl ThreadStore {
    /// 一次持久提交一块；返回 Ok 就算这一块已确认。
    fn put_block(&mut self, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        match &mut self.handle {
            StoreHandle::Redb(database) => {
                let quick_repair = self.candidate == Candidate::RedbQuickRepair;
                let mut transaction = database.begin_write().map_err(redb_error)?;
                transaction.set_durability(redb::Durability::Immediate).map_err(redb_error)?;
                transaction.set_quick_repair(quick_repair);
                {
                    let mut table = transaction.open_table(REDB_TABLE).map_err(redb_error)?;
                    table.insert(&key[..], value).map_err(redb_error)?;
                }
                transaction.commit().map_err(redb_error)
            }
            StoreHandle::RocksDatabase(database) => database.put_opt(key, value, &rocksdb_write_options()).map_err(rocksdb_error),
            StoreHandle::File(library) => library.put_block(key, value),
        }
    }

    fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match &self.handle {
            StoreHandle::Redb(database) => {
                let transaction = match database.begin_read() {
                    Ok(transaction) => transaction,
                    Err(error) => return ReadOutcome::ReadError(error.to_string()),
                };
                let table = match transaction.open_table(REDB_TABLE) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => return ReadOutcome::NotFound,
                    Err(error) => return ReadOutcome::ReadError(error.to_string()),
                };
                let outcome = match redb::ReadableTable::get(&table, &key[..]) {
                    Ok(Some(guard)) => ReadOutcome::Found(guard.value().to_vec()),
                    Ok(None) => ReadOutcome::NotFound,
                    Err(error) => ReadOutcome::ReadError(error.to_string()),
                };
                outcome
            }
            StoreHandle::RocksDatabase(database) => match database.get(key) {
                Ok(Some(value)) => ReadOutcome::Found(value),
                Ok(None) => ReadOutcome::NotFound,
                Err(error) => ReadOutcome::ReadError(error.to_string()),
            },
            StoreHandle::File(library) => library.read_block(key),
        }
    }

    /// 枚举库里的全部键与各自块值的长度（F 按文件名与文件长度，不读内容）。
    fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        match &self.handle {
            StoreHandle::Redb(database) => {
                let transaction = database.begin_read().map_err(redb_error)?;
                let table = match transaction.open_table(REDB_TABLE) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
                    Err(error) => return Err(redb_error(error)),
                };
                let mut entries = Vec::new();
                for item in redb::ReadableTable::iter(&table).map_err(redb_error)? {
                    let (key, value) = item.map_err(redb_error)?;
                    entries.push((key.value().to_vec(), u64::try_from(value.value().len()).expect("块值长度装得进 u64")));
                }
                Ok(entries)
            }
            StoreHandle::RocksDatabase(database) => {
                let mut entries = Vec::new();
                for item in database.iterator(rocksdb::IteratorMode::Start) {
                    let (key, value) = item.map_err(rocksdb_error)?;
                    entries.push((key.to_vec(), u64::try_from(value.len()).expect("块值长度装得进 u64")));
                }
                Ok(entries)
            }
            StoreHandle::File(library) => library.enumerate_keys_with_value_lengths(),
        }
    }
}

// ============================== 候选 F：加固的文件 ==============================

/// 一块一个文件：`<根>/<节点 8 位十六进制>/<段号 8 位十六进制>/<起点 16 位十六进制>-<指纹 32 位十六进制>.blk`。
struct FileLibrary {
    root: PathBuf,
    known_directories: HashSet<PathBuf>,
}

fn sync_directory(directory: &Path) -> Result<(), StoreError> {
    File::open(directory)
        .and_then(|handle| handle.sync_all())
        .map_err(|error| file_error(&format!("目录 fsync {}", directory.display()), error))
}

fn format_file_contents() -> [u8; FORMAT_FILE_BYTES] {
    let mut contents = [0u8; FORMAT_FILE_BYTES];
    contents[0..8].copy_from_slice(FORMAT_MAGIC);
    contents[8..12].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    let checksum = sender_role::crc32c(&[&contents[0..12]]);
    contents[12..16].copy_from_slice(&checksum.to_le_bytes());
    contents
}

fn block_file_relative_path(key: &[u8]) -> Option<PathBuf> {
    if key.len() != BLOCK_KEY_BYTES {
        return None;
    }
    let node = u32::from_be_bytes(key[0..4].try_into().expect("4 字节"));
    let segment = u32::from_be_bytes(key[4..8].try_into().expect("4 字节"));
    let first_state = u64::from_be_bytes(key[8..16].try_into().expect("8 字节"));
    Some(
        PathBuf::from(format!("{node:08x}"))
            .join(format!("{segment:08x}"))
            .join(format!("{first_state:016x}-{}.blk", hexadecimal(&key[16..32]))),
    )
}

fn temporary_path_of(final_path: &Path) -> PathBuf {
    let mut name = final_path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

/// 文件头 48 字节：魔数｜版本 u32 小端｜块键｜块值长度 u32 小端｜CRC-32C（覆盖头的前 44 字节加整块值）。
fn block_file_header(key: &BlockKey, value: &[u8]) -> [u8; FILE_HEADER_BYTES] {
    let mut header = [0u8; FILE_HEADER_BYTES];
    header[0..4].copy_from_slice(FILE_MAGIC);
    header[4..8].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    header[8..40].copy_from_slice(key);
    let value_length = u32::try_from(value.len()).expect("块值装得进 u32");
    header[40..44].copy_from_slice(&value_length.to_le_bytes());
    let checksum = sender_role::crc32c(&[&header[..FILE_HEADER_CHECKSUMMED_BYTES], value]);
    header[44..48].copy_from_slice(&checksum.to_le_bytes());
    header
}

/// 读路径的校验：长度、魔数、版本、头里的键、长度字段、CRC 任一不对就是读报错。
fn validate_block_file(bytes: &[u8], key: &BlockKey) -> Result<Vec<u8>, String> {
    if bytes.len() < FILE_HEADER_BYTES {
        return Err(format!("文件只有 {} 字节，短于文件头", bytes.len()));
    }
    if &bytes[0..4] != FILE_MAGIC {
        return Err("魔数不对".to_string());
    }
    if bytes[4..8] != FILE_FORMAT_VERSION.to_le_bytes() {
        return Err("版本不对".to_string());
    }
    if &bytes[8..40] != key {
        return Err("头里的键与文件名不符".to_string());
    }
    let value_length = u32::from_le_bytes(bytes[40..44].try_into().expect("4 字节"));
    let value = &bytes[FILE_HEADER_BYTES..];
    if u64::from(value_length) != u64::try_from(value.len()).expect("装得进 u64") {
        return Err(format!("长度字段 {value_length} 与文件里的 {} 字节块值不符", value.len()));
    }
    let stored_checksum = u32::from_le_bytes(bytes[44..48].try_into().expect("4 字节"));
    if sender_role::crc32c(&[&bytes[..FILE_HEADER_CHECKSUMMED_BYTES], value]) != stored_checksum {
        return Err("CRC-32C 不对".to_string());
    }
    Ok(value.to_vec())
}

impl FileLibrary {
    /// 打开：根目录在、FORMAT 在且对，否则打开失败；打开时删掉残留的 `*.tmp`。
    /// allow_create：根目录不在时（写入用）先经 `<根>.creating` 建好 FORMAT、改名成根目录。
    fn open(root: &Path, allow_create: bool) -> Result<FileLibrary, StoreError> {
        if !root.exists() {
            if !allow_create {
                return Err(StoreError::File(format!("根目录 {} 不在", root.display())));
            }
            Self::create_root_atomically(root)?;
        }
        let format_bytes = fs::read(root.join(FORMAT_FILE_NAME)).map_err(|error| file_error("读 FORMAT", error))?;
        if format_bytes[..] != format_file_contents()[..] {
            return Err(StoreError::File("FORMAT 不对".to_string()));
        }
        let mut known_directories = HashSet::new();
        for node_entry in fs::read_dir(root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            if !node_path.is_dir() {
                continue;
            }
            known_directories.insert(node_path.clone());
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                if !segment_path.is_dir() {
                    continue;
                }
                known_directories.insert(segment_path.clone());
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_path = block_entry.map_err(|error| file_error("列段目录", error))?.path();
                    if block_path.extension().is_some_and(|extension| extension == "tmp") {
                        fs::remove_file(&block_path).map_err(|error| file_error("删残留的临时文件", error))?;
                    }
                }
            }
        }
        Ok(FileLibrary { root: root.to_path_buf(), known_directories })
    }

    fn create_root_atomically(root: &Path) -> Result<(), StoreError> {
        let mut creating_name = root.as_os_str().to_owned();
        creating_name.push(".creating");
        let creating = PathBuf::from(creating_name);
        if creating.exists() {
            fs::remove_dir_all(&creating).map_err(|error| file_error("删上一次没建完的根目录", error))?;
        }
        fs::create_dir_all(&creating).map_err(|error| file_error("建根目录", error))?;
        let mut format_file = File::create(creating.join(FORMAT_FILE_NAME)).map_err(|error| file_error("建 FORMAT", error))?;
        format_file.write_all(&format_file_contents()).map_err(|error| file_error("写 FORMAT", error))?;
        format_file.sync_all().map_err(|error| file_error("fsync FORMAT", error))?;
        sync_directory(&creating)?;
        fs::rename(&creating, root).map_err(|error| file_error("根目录改名", error))?;
        let parent = root.parent().expect("根目录有父目录");
        sync_directory(parent)
    }

    /// 节点与段目录这个句柄没见过就建（另一个线程刚建好、撞上 AlreadyExists 也算建好），
    /// 建完对新目录与它的父目录各 fsync：两个线程都 fsync，谁先返回都不靠另一个线程的 fsync 作保。
    fn ensure_directory(&mut self, directory: &Path) -> Result<(), StoreError> {
        if self.known_directories.contains(directory) {
            return Ok(());
        }
        let parent = directory.parent().expect("目录有父目录").to_path_buf();
        if parent != self.root {
            self.ensure_directory(&parent)?;
        }
        match fs::create_dir(directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(file_error("建目录", error)),
        }
        sync_directory(directory)?;
        sync_directory(&parent)?;
        self.known_directories.insert(directory.to_path_buf());
        Ok(())
    }

    fn final_path_of(&self, key: &BlockKey) -> PathBuf {
        self.root.join(block_file_relative_path(key).expect("块键 32 字节"))
    }

    /// 登记的写法：临时文件（一次 write_all）→ sync_all → 改名 → 目录 fsync。
    fn put_block(&mut self, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        let final_path = self.final_path_of(key);
        let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
        self.ensure_directory(&directory)?;
        let temporary_path = temporary_path_of(&final_path);
        let mut contents = Vec::with_capacity(FILE_HEADER_BYTES + value.len());
        contents.extend_from_slice(&block_file_header(key, value));
        contents.extend_from_slice(value);
        let mut handle = File::create(&temporary_path).map_err(|error| file_error("建临时文件", error))?;
        handle.write_all(&contents).map_err(|error| file_error("写临时文件", error))?;
        handle.sync_all().map_err(|error| file_error("fsync 临时文件", error))?;
        fs::rename(&temporary_path, &final_path).map_err(|error| file_error("改名", error))?;
        sync_directory(&directory)
    }

    fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match fs::read(self.final_path_of(key)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::NotFound,
            Err(error) => ReadOutcome::ReadError(error.to_string()),
            Ok(bytes) => match validate_block_file(&bytes, key) {
                Ok(value) => ReadOutcome::Found(value),
                Err(reason) => ReadOutcome::ReadError(reason),
            },
        }
    }

    /// 走目录收全部 `.blk` 文件名（`*.tmp` 与别的文件不算）；键从路径拆回，块值长度 = 文件长度 − 48。
    fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        let mut entries = Vec::new();
        for node_entry in fs::read_dir(&self.root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            let Some(node) = hexadecimal_name_as_u32(&node_path) else { continue };
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                let Some(segment) = hexadecimal_name_as_u32(&segment_path) else { continue };
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_entry = block_entry.map_err(|error| file_error("列段目录", error))?;
                    let name = block_entry.file_name().to_string_lossy().to_string();
                    let Some(stem) = name.strip_suffix(".blk") else { continue };
                    let Some((first_state_text, fingerprint_text)) = stem.split_once('-') else { continue };
                    let (Ok(first_state), Some(fingerprint)) =
                        (u64::from_str_radix(first_state_text, 16), parse_hexadecimal(fingerprint_text))
                    else {
                        continue;
                    };
                    let mut key = Vec::with_capacity(BLOCK_KEY_BYTES);
                    key.extend_from_slice(&node.to_be_bytes());
                    key.extend_from_slice(&segment.to_be_bytes());
                    key.extend_from_slice(&first_state.to_be_bytes());
                    key.extend_from_slice(&fingerprint);
                    let file_length = block_entry.metadata().map_err(|error| file_error("读文件长度", error))?.len();
                    entries.push((key, file_length.saturating_sub(u64::try_from(FILE_HEADER_BYTES).expect("48"))));
                }
            }
        }
        Ok(entries)
    }
}

fn hexadecimal_name_as_u32(path: &Path) -> Option<u32> {
    if !path.is_dir() {
        return None;
    }
    let name = path.file_name()?.to_str()?;
    if name.len() != 8 {
        return None;
    }
    u32::from_str_radix(name, 16).ok()
}

fn parse_hexadecimal(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).ok())
        .collect()
}

// ============================== 中断计划与 PC3-drop 挑帧 ==============================

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlannedInterruption {
    /// 确认数到 a_k 之后中断（回环格：送块进程报的确认数；跨机格：库进程回出的确认数）。
    acknowledgement_threshold: u64,
    /// u_k：停 u_k × 200 ms 再起。
    pause_fraction: f64,
}

/// a_k 在 [1, 网络块数) 里均匀取 count 个、排序，u_k 同源（登记 6.3 末段；网络块数 5 × 10⁴ 时就是登记的 [1, 5 × 10⁴)）：
/// SplitMix64(seed_NET) 每个中断先后取 r1、r2，a = 1 + r1 mod (网络块数 − 1)，u = (r2 >> 11) ÷ 2⁵³；按 a 稳定排序。
fn interruption_plan(count: usize, network_blocks: u64) -> Vec<PlannedInterruption> {
    assert!(network_blocks >= 2, "网络块数至少 2 才取得出 [1, 网络块数) 里的中断点");
    let mut generator = SplitMix64::new(SEED_NETWORK_INTERRUPTIONS);
    let mut plan: Vec<PlannedInterruption> = (0..count)
        .map(|_| {
            let threshold_output = generator.next_output();
            let pause_output = generator.next_output();
            PlannedInterruption {
                acknowledgement_threshold: 1 + threshold_output % (network_blocks - 1),
                pause_fraction: (pause_output >> 11) as f64 / (1u64 << 53) as f64,
            }
        })
        .collect();
    plan.sort_by_key(|interruption| interruption.acknowledgement_threshold);
    plan
}

fn pause_of(interruption: &PlannedInterruption) -> Duration {
    Duration::from_secs_f64(interruption.pause_fraction * INTERRUPTION_PAUSE_UNIT_MILLISECONDS / 1_000.0)
}

/// PC3-drop：第 w 个 1 000 帧里挑第 SplitMix64(seed_NET) 第 w 个输出 mod 1 000 帧（另起一个生成器，与中断计划不共用状态）。
fn drop_control_picks(window_count: usize) -> Vec<u64> {
    let mut generator = SplitMix64::new(SEED_NETWORK_INTERRUPTIONS);
    (0..window_count).map(|_| generator.next_output() % DROP_CONTROL_FRAMES_PER_PICK).collect()
}

// ============================== 库进程：收块线程 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReceiveMode {
    /// 臂的定义：CRC 不对或半帧就丢、不回确认；完整的帧按块持久提交，提交返回之后回确认（登记 6.3 末段）。
    Registered,
    /// PC3-drop：每 1 000 帧挑一帧只回确认、不写。
    DropOnePerThousand,
    /// PC3-torn：不校验帧 CRC、半帧用 0 补齐照写，且「键已在就跳过不写」。
    TornAccepting,
    /// PC3-rate 慢的那一遍：每帧提交之前睡 5 ms。
    SleepBeforeCommit,
    /// S3-raw：校验 CRC 之后不写库、直接回确认。
    AcknowledgeWithoutWriting,
}

impl ReceiveMode {
    fn label(self) -> &'static str {
        match self {
            ReceiveMode::Registered => "registered",
            ReceiveMode::DropOnePerThousand => "drop-one-per-thousand",
            ReceiveMode::TornAccepting => "torn-accepting",
            ReceiveMode::SleepBeforeCommit => "sleep-before-commit",
            ReceiveMode::AcknowledgeWithoutWriting => "acknowledge-without-writing",
        }
    }

    fn parse(text: &str) -> Option<ReceiveMode> {
        match text {
            "registered" => Some(ReceiveMode::Registered),
            "drop-one-per-thousand" => Some(ReceiveMode::DropOnePerThousand),
            "torn-accepting" => Some(ReceiveMode::TornAccepting),
            "sleep-before-commit" => Some(ReceiveMode::SleepBeforeCommit),
            "acknowledge-without-writing" => Some(ReceiveMode::AcknowledgeWithoutWriting),
            _ => None,
        }
    }
}

/// 一条连接为什么结束。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionEnd {
    /// 对面在帧边界上关了。
    ClosedAtFrameBoundary,
    /// 读到半帧对面就关了。
    HalfFrame,
    /// 魔数或块值长度不对：流已经对不上帧了，关掉这条连接。
    ProtocolError,
    AcknowledgementWriteFailed,
    /// 跨机格：回出第 a_k 个确认了，该杀数据通道了。
    InterruptionDue,
}

impl ConnectionEnd {
    fn label(self) -> &'static str {
        match self {
            ConnectionEnd::ClosedAtFrameBoundary => "closed_at_frame_boundary",
            ConnectionEnd::HalfFrame => "half_frame",
            ConnectionEnd::ProtocolError => "protocol_error",
            ConnectionEnd::AcknowledgementWriteFailed => "acknowledgement_write_failed",
            ConnectionEnd::InterruptionDue => "interruption_due",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ReceiveCounters {
    connections: u64,
    complete_frames: u64,
    checksum_failures: u64,
    half_frames_dropped: u64,
    /// PC3-torn：补 0 照写的半帧数。
    half_frames_written: u64,
    protocol_errors: u64,
    commits: u64,
    commit_failures: u64,
    /// 收到的完整帧的键这个收块线程早先已经写进库（重发的帧碰上库里已有的块，Q3h）。
    frames_already_stored: u64,
    /// PC3-torn：键已在、跳过不写的帧数。
    frames_skipped_as_already_stored: u64,
    /// PC3-drop：只回确认、没写的帧数。
    controlled_drops: u64,
    acknowledgements_sent: u64,
    acknowledgement_write_failures: u64,
}

struct FrameReceiver {
    receive_mode: ReceiveMode,
    expected_value_bytes: usize,
    store: ThreadStore,
    /// 这个收块线程写进库的键（库进程不重起，所以就是「库里已有」的网络块）。
    stored_keys: HashSet<BlockKey>,
    drop_picks: Vec<u64>,
    counters: ReceiveCounters,
    first_commit_error: String,
}

impl FrameReceiver {
    fn new(receive_mode: ReceiveMode, states_per_block: u64, network_blocks: u64, store: ThreadStore) -> FrameReceiver {
        let window_count = usize::try_from(network_blocks / DROP_CONTROL_FRAMES_PER_PICK + 2).expect("窗数装得进 usize");
        FrameReceiver {
            receive_mode,
            expected_value_bytes: usize::try_from(states_per_block).expect("块值字节装得进 usize"),
            store,
            stored_keys: HashSet::new(),
            drop_picks: drop_control_picks(window_count),
            counters: ReceiveCounters::default(),
            first_commit_error: String::new(),
        }
    }

    fn commit(&mut self, key: &BlockKey, value: &[u8]) -> bool {
        match self.store.put_block(key, value) {
            Ok(()) => {
                self.counters.commits += 1;
                self.stored_keys.insert(*key);
                true
            }
            Err(error) => {
                self.counters.commit_failures += 1;
                if self.first_commit_error.is_empty() {
                    self.first_commit_error = error.to_string();
                }
                false
            }
        }
    }

    /// PC3-torn 的半帧：键读全了才知道写哪一块；块值用 0 补齐到整块。
    fn write_zero_padded_half_frame(&mut self, header: &[u8], value_prefix: &[u8]) {
        if header.len() < 4 + BLOCK_KEY_BYTES || &header[0..4] != BLOCK_FRAME_MAGIC {
            self.counters.half_frames_dropped += 1;
            return;
        }
        let key: BlockKey = header[4..4 + BLOCK_KEY_BYTES].try_into().expect("32 字节");
        if self.stored_keys.contains(&key) {
            self.counters.half_frames_dropped += 1;
            return;
        }
        let mut padded = vec![0u8; self.expected_value_bytes];
        let copied = value_prefix.len().min(self.expected_value_bytes);
        padded[..copied].copy_from_slice(&value_prefix[..copied]);
        if self.commit(&key, &padded) {
            self.counters.half_frames_written += 1;
        }
    }

    /// 服务一条连接，直到它结束；stop_after_acknowledgements 给了就在累计回出这么多个确认之后返回 InterruptionDue。
    fn serve_connection(&mut self, reader: &mut dyn Read, writer: &mut dyn Write, stop_after_acknowledgements: Option<u64>) -> ConnectionEnd {
        self.counters.connections += 1;
        let accepts_torn_frames = self.receive_mode == ReceiveMode::TornAccepting;
        loop {
            let mut header = [0u8; BLOCK_FRAME_HEADER_BYTES];
            let header_bytes = read_until_full_or_end(reader, &mut header);
            if header_bytes == 0 {
                return ConnectionEnd::ClosedAtFrameBoundary;
            }
            if header_bytes < BLOCK_FRAME_HEADER_BYTES {
                if accepts_torn_frames {
                    self.write_zero_padded_half_frame(&header[..header_bytes], &[]);
                } else {
                    self.counters.half_frames_dropped += 1;
                }
                return ConnectionEnd::HalfFrame;
            }
            if &header[0..4] != BLOCK_FRAME_MAGIC {
                self.counters.protocol_errors += 1;
                return ConnectionEnd::ProtocolError;
            }
            let key: BlockKey = header[4..4 + BLOCK_KEY_BYTES].try_into().expect("32 字节");
            let value_length = u32::from_le_bytes(header[4 + BLOCK_KEY_BYTES..].try_into().expect("4 字节"));
            if usize::try_from(value_length).ok() != Some(self.expected_value_bytes) {
                self.counters.protocol_errors += 1;
                return ConnectionEnd::ProtocolError;
            }
            let mut rest = vec![0u8; self.expected_value_bytes + FRAME_CHECKSUM_BYTES];
            let rest_bytes = read_until_full_or_end(reader, &mut rest);
            if rest_bytes < rest.len() {
                if accepts_torn_frames {
                    let value_prefix = &rest[..rest_bytes.min(self.expected_value_bytes)];
                    self.write_zero_padded_half_frame(&header, value_prefix);
                } else {
                    self.counters.half_frames_dropped += 1;
                }
                return ConnectionEnd::HalfFrame;
            }
            self.counters.complete_frames += 1;
            let (value, checksum_bytes) = rest.split_at(self.expected_value_bytes);
            let stored_checksum = u32::from_le_bytes(checksum_bytes.try_into().expect("4 字节"));
            if !accepts_torn_frames && block_frame_checksum(&key, value_length, value) != stored_checksum {
                self.counters.checksum_failures += 1;
                continue;
            }
            let is_already_stored = self.stored_keys.contains(&key);
            if is_already_stored {
                self.counters.frames_already_stored += 1;
            }
            let is_confirmable = if accepts_torn_frames && is_already_stored {
                self.counters.frames_skipped_as_already_stored += 1;
                true
            } else {
                match self.receive_mode {
                    ReceiveMode::Registered | ReceiveMode::TornAccepting => self.commit(&key, value),
                    ReceiveMode::SleepBeforeCommit => {
                        std::thread::sleep(RATE_CONTROL_SLEEP);
                        self.commit(&key, value)
                    }
                    ReceiveMode::AcknowledgeWithoutWriting => true,
                    ReceiveMode::DropOnePerThousand => {
                        let valid_frame_index = self.counters.complete_frames - self.counters.checksum_failures - 1;
                        let window = usize::try_from(valid_frame_index / DROP_CONTROL_FRAMES_PER_PICK).expect("窗号装得进 usize");
                        let picked = self.drop_picks.get(window).copied();
                        if picked == Some(valid_frame_index % DROP_CONTROL_FRAMES_PER_PICK) {
                            self.counters.controlled_drops += 1;
                            true
                        } else {
                            self.commit(&key, value)
                        }
                    }
                }
            };
            if !is_confirmable {
                continue;
            }
            if writer.write_all(&encode_acknowledgement_frame(&key)).and_then(|()| writer.flush()).is_err() {
                self.counters.acknowledgement_write_failures += 1;
                return ConnectionEnd::AcknowledgementWriteFailed;
            }
            self.counters.acknowledgements_sent += 1;
            if stop_after_acknowledgements.is_some_and(|threshold| self.counters.acknowledgements_sent >= threshold) {
                return ConnectionEnd::InterruptionDue;
            }
        }
    }

    fn summary_line(&self, connection_ends: &BTreeSet<String>) -> String {
        let counters = &self.counters;
        format!(
            "N receive_mode={} connections={} complete_frames={} checksum_failures={} half_frames_dropped={} half_frames_written={} protocol_errors={} commits={} commit_failures={} frames_already_stored={} frames_skipped_as_already_stored={} controlled_drops={} acknowledgements_sent={} acknowledgement_write_failures={} connection_ends={} first_commit_error={}",
            self.receive_mode.label(),
            counters.connections,
            counters.complete_frames,
            counters.checksum_failures,
            counters.half_frames_dropped,
            counters.half_frames_written,
            counters.protocol_errors,
            counters.commits,
            counters.commit_failures,
            counters.frames_already_stored,
            counters.frames_skipped_as_already_stored,
            counters.controlled_drops,
            counters.acknowledgements_sent,
            counters.acknowledgement_write_failures,
            if connection_ends.is_empty() { "none".to_string() } else { connection_ends.iter().cloned().collect::<Vec<_>>().join(",") },
            text_field(&self.first_commit_error),
        )
    }
}

/// 产物行的值里不许有空白：错误文字里的空白换成下划线，空的写 none。
fn text_field(text: &str) -> String {
    if text.is_empty() {
        "none".to_string()
    } else {
        text.split_whitespace().collect::<Vec<_>>().join("_")
    }
}

// ============================== 库进程：直写线程 ==============================

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DirectPathReport {
    blocks: u64,
    confirmed: u64,
    started: u64,
    /// 第一次提交开始到最后一次提交返回（口径同 Q2a）。
    elapsed_microseconds: u64,
    commit_median_microseconds: u64,
    first_error: String,
}

/// 直写那一路：交错次序里 j 为偶数的块，块值由另一个线程预先生成、经深度 64 的有界通道交给写线程（口径同 Q2a）。
fn run_direct_path(mut store: ThreadStore, block_count: u64, states_per_block: u64) -> DirectPathReport {
    let (value_sender, prepared_blocks) = mpsc::sync_channel::<(BlockKey, Vec<u8>)>(64);
    let generator = std::thread::spawn(move || {
        let fingerprint = input_fingerprint();
        for direct_position in 0..block_count {
            let block_index = 2 * direct_position;
            let key = interleaved_block_key(block_index, states_per_block, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, states_per_block);
            if value_sender.send((key, value)).is_err() {
                return;
            }
        }
    });
    let mut report = DirectPathReport { blocks: block_count, ..DirectPathReport::default() };
    let mut first_commit_started: Option<Instant> = None;
    let mut last_commit_returned: Option<Instant> = None;
    let mut commit_latencies = Vec::with_capacity(usize::try_from(block_count).expect("块数装得进 usize"));
    for (key, value) in prepared_blocks.iter() {
        let commit_started = Instant::now();
        first_commit_started.get_or_insert(commit_started);
        report.started += 1;
        match store.put_block(&key, &value) {
            Ok(()) => {
                let commit_returned = Instant::now();
                last_commit_returned = Some(commit_returned);
                commit_latencies.push(u64::try_from((commit_returned - commit_started).as_micros()).expect("微秒装得进 u64"));
                report.confirmed += 1;
            }
            Err(error) => {
                report.first_error = error.to_string();
                break;
            }
        }
    }
    drop(prepared_blocks);
    generator.join().expect("生成块值的线程没 panic");
    if let (Some(first), Some(last)) = (first_commit_started, last_commit_returned) {
        report.elapsed_microseconds = u64::try_from((last - first).as_micros()).expect("微秒装得进 u64");
    }
    commit_latencies.sort_unstable();
    report.commit_median_microseconds = commit_latencies.get(commit_latencies.len() / 2).copied().unwrap_or(0);
    report
}

impl DirectPathReport {
    fn summary_line(&self) -> String {
        format!(
            "D blocks={} confirmed={} started={} elapsed_microseconds={} commit_median_microseconds={} first_error={}",
            self.blocks,
            self.confirmed,
            self.started,
            self.elapsed_microseconds,
            self.commit_median_microseconds,
            text_field(&self.first_error)
        )
    }
}

// ============================== 库进程 ==============================

/// 收块的数据通道：回环格在 127.0.0.1 上等转发进程连进来；跨机格库进程自己起 `<对端命令…> <中继命令>`，
/// 帧走它的标准输出、确认走它的标准输入，回出第 a_k 个确认之后杀掉它、停 u_k × 200 ms 再起（补 1 读法表「网络中断（跨机格）」）。
#[derive(Debug, Clone, PartialEq)]
enum ReceiveChannel {
    TcpListen,
    PeerCommand { command_words: Vec<String>, relay_command: String, interruptions: Vec<PlannedInterruption> },
}

struct LibrarySettings {
    candidate: Candidate,
    library: PathBuf,
    receive_mode: ReceiveMode,
    direct_blocks: u64,
    network_blocks: u64,
    states_per_block: u64,
    channel: ReceiveChannel,
}

fn parse_flag_values(arguments: &[String]) -> Result<HashMap<String, String>, String> {
    let mut values = HashMap::new();
    let mut position = 0;
    while position < arguments.len() {
        let flag = arguments[position].clone();
        let value = arguments.get(position + 1).ok_or(format!("{flag} 后面缺值"))?.clone();
        values.insert(flag, value);
        position += 2;
    }
    Ok(values)
}

fn required_number(values: &HashMap<String, String>, flag: &str) -> Result<u64, String> {
    values
        .get(flag)
        .ok_or(format!("缺 {flag}"))?
        .parse()
        .map_err(|_| format!("{flag} 不是数"))
}

impl LibrarySettings {
    fn to_arguments(&self) -> Vec<String> {
        let mut arguments = vec![
            "library".to_string(),
            "--arm".to_string(),
            self.candidate.label().to_string(),
            "--library".to_string(),
            self.library.display().to_string(),
            "--mode".to_string(),
            self.receive_mode.label().to_string(),
            "--direct-blocks".to_string(),
            self.direct_blocks.to_string(),
            "--network-blocks".to_string(),
            self.network_blocks.to_string(),
            "--states-per-block".to_string(),
            self.states_per_block.to_string(),
        ];
        if let ReceiveChannel::PeerCommand { command_words, relay_command, interruptions } = &self.channel {
            arguments.push("--peer-command".to_string());
            arguments.push(command_words.join(" "));
            arguments.push("--relay-command".to_string());
            arguments.push(relay_command.clone());
            arguments.push("--interruptions".to_string());
            arguments.push(interruptions.len().to_string());
        }
        arguments
    }

    fn parse(arguments: &[String]) -> Result<LibrarySettings, String> {
        let values = parse_flag_values(arguments)?;
        let candidate = Candidate::parse(values.get("--arm").ok_or("缺 --arm")?).ok_or("认不出的臂")?;
        let receive_mode = ReceiveMode::parse(values.get("--mode").ok_or("缺 --mode")?).ok_or("认不出的收块写法")?;
        let network_blocks = required_number(&values, "--network-blocks")?;
        let channel = match (values.get("--peer-command"), values.get("--relay-command")) {
            (Some(command), Some(relay_command)) => {
                let interruption_count = usize::try_from(required_number(&values, "--interruptions")?).map_err(|_| "中断数装不进 usize")?;
                ReceiveChannel::PeerCommand {
                    command_words: command.split_whitespace().map(str::to_string).collect(),
                    relay_command: relay_command.clone(),
                    interruptions: interruption_plan(interruption_count, network_blocks),
                }
            }
            (None, None) => ReceiveChannel::TcpListen,
            _ => return Err("--peer-command 与 --relay-command 要一起给".to_string()),
        };
        Ok(LibrarySettings {
            candidate,
            library: PathBuf::from(values.get("--library").ok_or("缺 --library")?),
            receive_mode,
            direct_blocks: required_number(&values, "--direct-blocks")?,
            network_blocks,
            states_per_block: required_number(&values, "--states-per-block")?,
            channel,
        })
    }
}

fn print_flushed(line: &str) {
    let mut standard_output = std::io::stdout().lock();
    writeln!(standard_output, "{line}").expect("写得进标准输出");
    standard_output.flush().expect("刷得出标准输出");
}

struct ReceiveOutcome {
    receiver: FrameReceiver,
    connection_ends: BTreeSet<String>,
    interruption_lines: Vec<String>,
}

fn receive_over_tcp(mut receiver: FrameReceiver, listener: TcpListener, stop_requested: Arc<AtomicBool>) -> ReceiveOutcome {
    let mut connection_ends = BTreeSet::new();
    for incoming in listener.incoming() {
        if stop_requested.load(Ordering::SeqCst) {
            break;
        }
        let Ok(stream) = incoming else { continue };
        stream.set_nodelay(true).expect("TCP 连接设得了 nodelay");
        let mut writer = stream.try_clone().expect("连接复制得出写的一端");
        let mut reader = stream;
        let end = receiver.serve_connection(&mut reader, &mut writer, None);
        connection_ends.insert(end.label().to_string());
    }
    ReceiveOutcome { receiver, connection_ends, interruption_lines: Vec::new() }
}

fn spawn_peer_channel(command_words: &[String], relay_command: &str) -> std::io::Result<Child> {
    let (program, fixed_arguments) = command_words.split_first().expect("对端命令至少一个词");
    Command::new(program)
        .args(fixed_arguments)
        .arg(relay_command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
}

fn receive_over_peer_channel(
    mut receiver: FrameReceiver,
    command_words: Vec<String>,
    relay_command: String,
    interruptions: Vec<PlannedInterruption>,
    stop_requested: Arc<AtomicBool>,
    current_channel: Arc<Mutex<Option<Child>>>,
) -> ReceiveOutcome {
    let mut connection_ends = BTreeSet::new();
    let mut interruption_lines = Vec::new();
    let mut next_interruption = 0usize;
    while !stop_requested.load(Ordering::SeqCst) {
        let mut child = match spawn_peer_channel(&command_words, &relay_command) {
            Ok(child) => child,
            Err(error) => {
                connection_ends.insert(format!("spawn_failed:{}", text_field(&error.to_string())));
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        };
        let mut channel_output = child.stdout.take().expect("数据通道的标准输出接在管道上");
        let mut channel_input = child.stdin.take().expect("数据通道的标准输入接在管道上");
        *current_channel.lock().expect("数据通道的锁没中毒") = Some(child);
        if stop_requested.load(Ordering::SeqCst) {
            break;
        }
        let threshold = interruptions.get(next_interruption).map(|interruption| interruption.acknowledgement_threshold);
        let frames_before = receiver.counters.complete_frames;
        let end = receiver.serve_connection(&mut channel_output, &mut channel_input, threshold);
        connection_ends.insert(end.label().to_string());
        if receiver.counters.complete_frames == frames_before && end != ConnectionEnd::InterruptionDue {
            // 数据通道一帧没送就断了（中继还没连上送块进程、或对端命令起不来）：隔 10 ms 再起，不空转。
            std::thread::sleep(Duration::from_millis(10));
        }
        if let Some(mut finished) = current_channel.lock().expect("数据通道的锁没中毒").take() {
            let _already_exited = finished.kill();
            finished.wait().expect("收得了数据通道子进程");
        }
        drop(channel_input);
        drop(channel_output);
        if end == ConnectionEnd::InterruptionDue {
            let interruption = interruptions[next_interruption];
            let pause = pause_of(&interruption);
            let paused_at = Instant::now();
            std::thread::sleep(pause);
            interruption_lines.push(format!(
                "I index={next_interruption} threshold={} acknowledgements_sent={} pause_microseconds={}",
                interruption.acknowledgement_threshold,
                receiver.counters.acknowledgements_sent,
                paused_at.elapsed().as_micros()
            ));
            next_interruption += 1;
        }
    }
    ReceiveOutcome { receiver, connection_ends, interruption_lines }
}

fn run_library_role(arguments: &[String]) -> i32 {
    let settings = match LibrarySettings::parse(arguments) {
        Ok(settings) => settings,
        Err(problem) => {
            eprintln!("  ✗ library 的参数不对：{problem}");
            return 2;
        }
    };
    if anchor_checks().iter().any(|check| !check.matches()) {
        print_flushed("E anchor_mismatch");
        return 5;
    }
    let mut stores = match open_thread_stores(settings.candidate, &settings.library, 2) {
        Ok(stores) => stores,
        Err(error) => {
            print_flushed(&format!("E open_failed={}", text_field(&error.to_string())));
            return 6;
        }
    };
    let receive_store = stores.pop().expect("开了两个句柄");
    let direct_store = stores.pop().expect("开了两个句柄");
    let receiver = FrameReceiver::new(settings.receive_mode, settings.states_per_block, settings.network_blocks, receive_store);
    let stop_requested = Arc::new(AtomicBool::new(false));
    let current_channel: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let (receive_thread, own_port) = match settings.channel.clone() {
        ReceiveChannel::TcpListen => {
            let listener = TcpListener::bind("127.0.0.1:0").expect("绑得上回环端口");
            let port = listener.local_addr().expect("有地址").port();
            let stop = Arc::clone(&stop_requested);
            let thread = std::thread::spawn(move || receive_over_tcp(receiver, listener, stop));
            print_flushed(&format!("L port={port}"));
            (thread, Some(port))
        }
        ReceiveChannel::PeerCommand { command_words, relay_command, interruptions } => {
            let stop = Arc::clone(&stop_requested);
            let channel_slot = Arc::clone(&current_channel);
            let thread = std::thread::spawn(move || {
                receive_over_peer_channel(receiver, command_words, relay_command, interruptions, stop, channel_slot)
            });
            print_flushed("L port=none");
            (thread, None)
        }
    };
    let direct_blocks = settings.direct_blocks;
    let states_per_block = settings.states_per_block;
    let direct_thread = std::thread::spawn(move || run_direct_path(direct_store, direct_blocks, states_per_block));
    let _consumed = std::io::copy(&mut std::io::stdin().lock(), &mut std::io::sink());
    stop_requested.store(true, Ordering::SeqCst);
    if let Some(port) = own_port {
        let _wake_accept = TcpStream::connect(("127.0.0.1", port));
    }
    if let Some(mut channel) = current_channel.lock().expect("数据通道的锁没中毒").take() {
        let _already_exited = channel.kill();
        let _reaped = channel.wait();
    }
    let outcome = receive_thread.join().expect("收块线程没 panic");
    let direct_report = direct_thread.join().expect("直写线程没 panic");
    print_flushed(&direct_report.summary_line());
    for line in &outcome.interruption_lines {
        print_flushed(line);
    }
    print_flushed(&outcome.receiver.summary_line(&outcome.connection_ends));
    0
}

// ============================== 回环转发进程 ==============================

fn copy_until_end(mut from: TcpStream, mut to: TcpStream) {
    let mut chunk = vec![0u8; 65_536];
    loop {
        match from.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(read_bytes) => {
                if to.write_all(&chunk[..read_bytes]).is_err() {
                    break;
                }
            }
        }
    }
    let _already_closed = to.shutdown(std::net::Shutdown::Write);
}

/// 双向逐字节转发（登记 6.3 末段）：在 --listen-port 上等（0 = 让内核挑，挑到的端口打一行 `F port=<端口>`），
/// 每个连进来的连接接到 127.0.0.1:<--target-port>；一直跑到被杀。
fn run_forward_role(arguments: &[String]) -> i32 {
    let values = match parse_flag_values(arguments) {
        Ok(values) => values,
        Err(problem) => {
            eprintln!("  ✗ forward 的参数不对：{problem}");
            return 2;
        }
    };
    let (Ok(listen_port), Ok(target_port)) = (required_number(&values, "--listen-port"), required_number(&values, "--target-port")) else {
        eprintln!("  ✗ forward 要 --listen-port 与 --target-port");
        return 2;
    };
    let listen_port = u16::try_from(listen_port).expect("端口装得进 u16");
    let target_port = u16::try_from(target_port).expect("端口装得进 u16");
    let started = Instant::now();
    let listener = loop {
        match TcpListener::bind(("127.0.0.1", listen_port)) {
            Ok(listener) => break listener,
            Err(error) if started.elapsed() < FORWARDER_BIND_DEADLINE => {
                let _retry_after = error;
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                eprintln!("  ✗ forward 绑不上 127.0.0.1:{listen_port}：{error}");
                return 3;
            }
        }
    };
    print_flushed(&format!("F port={}", listener.local_addr().expect("有地址").port()));
    for incoming in listener.incoming() {
        let Ok(client) = incoming else { continue };
        std::thread::spawn(move || {
            let Ok(library) = TcpStream::connect(("127.0.0.1", target_port)) else { return };
            client.set_nodelay(true).expect("nodelay");
            library.set_nodelay(true).expect("nodelay");
            let client_reader = client.try_clone().expect("复制");
            let library_reader = library.try_clone().expect("复制");
            let upstream = std::thread::spawn(move || copy_until_end(client_reader, library));
            copy_until_end(library_reader, client);
            upstream.join().expect("转发线程没 panic");
        });
    }
    0
}

// ============================== 逐块核（登记 5.1 的表，两路共 N 块） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockClass {
    Confirmed,
    InFlight,
    NotStarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadKind {
    MatchingBytes,
    DifferentBytes,
    NotFound,
    ReadError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockVerdict {
    Correct,
    /// Q3d：收到过确认（直写：提交返回过）而读不到。
    LostConfirmed,
    /// Q3e：读到而字节不同。
    SilentlyCorrupted,
    /// Q3f 的一半：已确认而读报错。
    ConfirmedReadError,
    /// Q3f 的另一半：在途或未开始而读报错。
    UnconfirmedReadError,
    /// Q3g：读到未开始的块。
    Phantom,
}

fn classify_block(class: BlockClass, read: ReadKind) -> BlockVerdict {
    match (class, read) {
        (BlockClass::Confirmed, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::Confirmed, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::Confirmed, ReadKind::NotFound) => BlockVerdict::LostConfirmed,
        (BlockClass::Confirmed, ReadKind::ReadError) => BlockVerdict::ConfirmedReadError,
        (BlockClass::InFlight, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::InFlight, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
        (BlockClass::NotStarted, ReadKind::MatchingBytes | ReadKind::DifferentBytes) => BlockVerdict::Phantom,
        (BlockClass::NotStarted, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::NotStarted, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
    }
}

/// 两路各自哪些块已确认、在途、未开始：直写是顺序的，已确认是前缀 [0, confirmed)、在途是 [confirmed, started)；
/// 网络那一路按送块进程报的确认区间与「发出过的块序号」[0, sent)。
#[derive(Debug, Clone, PartialEq, Eq)]
struct PathVerificationPlan {
    states_per_block: u64,
    direct_blocks: u64,
    direct_confirmed: u64,
    direct_started: u64,
    network_blocks: u64,
    network_acknowledged: Vec<bool>,
    network_sent: u64,
}

impl PathVerificationPlan {
    fn direct_class(&self, direct_position: u64) -> BlockClass {
        if direct_position < self.direct_confirmed {
            BlockClass::Confirmed
        } else if direct_position < self.direct_started {
            BlockClass::InFlight
        } else {
            BlockClass::NotStarted
        }
    }

    fn network_class(&self, network_position: u64) -> BlockClass {
        let position_as_index = usize::try_from(network_position).expect("块序号装得进 usize");
        if self.network_acknowledged.get(position_as_index).copied().unwrap_or(false) {
            BlockClass::Confirmed
        } else if network_position < self.network_sent {
            BlockClass::InFlight
        } else {
            BlockClass::NotStarted
        }
    }

    fn to_arguments(&self, candidate: Candidate, library: &Path) -> Vec<String> {
        vec![
            "verify".to_string(),
            "--arm".to_string(),
            candidate.label().to_string(),
            "--library".to_string(),
            library.display().to_string(),
            "--states-per-block".to_string(),
            self.states_per_block.to_string(),
            "--direct-blocks".to_string(),
            self.direct_blocks.to_string(),
            "--direct-confirmed".to_string(),
            self.direct_confirmed.to_string(),
            "--direct-started".to_string(),
            self.direct_started.to_string(),
            "--network-blocks".to_string(),
            self.network_blocks.to_string(),
            "--network-acknowledged".to_string(),
            compress_true_positions(&self.network_acknowledged),
            "--network-sent".to_string(),
            self.network_sent.to_string(),
        ]
    }

    fn parse(values: &HashMap<String, String>) -> Result<PathVerificationPlan, String> {
        let network_blocks = required_number(values, "--network-blocks")?;
        let acknowledged_text = values.get("--network-acknowledged").ok_or("缺 --network-acknowledged")?;
        Ok(PathVerificationPlan {
            states_per_block: required_number(values, "--states-per-block")?,
            direct_blocks: required_number(values, "--direct-blocks")?,
            direct_confirmed: required_number(values, "--direct-confirmed")?,
            direct_started: required_number(values, "--direct-started")?,
            network_blocks,
            network_acknowledged: expand_true_positions(acknowledged_text, network_blocks).ok_or("确认区间写坏了")?,
            network_sent: required_number(values, "--network-sent")?,
        })
    }
}

/// `compress_true_positions` 的反过来：给出总数，还原成布尔串；写坏了返回 None。
fn expand_true_positions(text: &str, total: u64) -> Option<Vec<bool>> {
    let mut flags = vec![false; usize::try_from(total).ok()?];
    if text == "none" {
        return Some(flags);
    }
    for range in text.split(',') {
        let (start_text, end_text) = range.split_once('-')?;
        let start: usize = start_text.parse().ok()?;
        let end: usize = end_text.parse().ok()?;
        if start > end || end >= flags.len() {
            return None;
        }
        for flag in &mut flags[start..=end] {
            *flag = true;
        }
    }
    Some(flags)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct VerificationReport {
    open_failed: bool,
    open_error: String,
    /// 只有 K：第一次打开失败之后调 DB::repair 再开，能开就是 Some(true)。
    opened_after_repair: Option<bool>,
    open_microseconds: u64,
    confirmed_checked: u64,
    in_flight_checked: u64,
    not_started_checked: u64,
    lost_confirmed: Vec<u64>,
    silently_corrupted: Vec<u64>,
    confirmed_read_errors: Vec<u64>,
    unconfirmed_read_errors: Vec<u64>,
    phantom_keys: BTreeSet<String>,
    enumerated_key_count: u64,
    enumerated_value_bytes: u64,
    enumeration_error: String,
    first_read_error: String,
    /// 核对子进程超时或没交回结果行。
    verification_missing: bool,
}

fn verify_library(candidate: Candidate, library: &Path, plan: &PathVerificationPlan) -> VerificationReport {
    let mut report = VerificationReport::default();
    let open_started = Instant::now();
    let opened = open_existing_store(candidate, library);
    report.open_microseconds = u64::try_from(open_started.elapsed().as_micros()).expect("微秒装得进 u64");
    let store = match opened {
        Ok(store) => store,
        Err(open_error) => {
            report.open_failed = true;
            report.open_error = open_error.to_string();
            match candidate {
                Candidate::RocksDatabase => {
                    let repaired = rocksdb::DB::repair(&rocksdb_options(), library)
                        .map_err(rocksdb_error)
                        .and_then(|()| open_existing_store(candidate, library));
                    report.opened_after_repair = Some(repaired.is_ok());
                    match repaired {
                        Ok(store) => store,
                        Err(_still_broken) => return report,
                    }
                }
                Candidate::RedbQuickRepair | Candidate::RedbDefault | Candidate::HardenedFile => return report,
            }
        }
    };
    let fingerprint = input_fingerprint();
    let mut checked: Vec<(u64, BlockClass)> = Vec::new();
    for direct_position in 0..plan.direct_blocks {
        checked.push((2 * direct_position, plan.direct_class(direct_position)));
    }
    for network_position in 0..plan.network_blocks {
        checked.push((network_path_block_index(network_position), plan.network_class(network_position)));
    }
    for (block_index, class) in checked {
        match class {
            BlockClass::Confirmed => report.confirmed_checked += 1,
            BlockClass::InFlight => report.in_flight_checked += 1,
            BlockClass::NotStarted => report.not_started_checked += 1,
        }
        let key = interleaved_block_key(block_index, plan.states_per_block, &fingerprint);
        let read = match store.read_block(&key) {
            ReadOutcome::Found(found) => {
                let expected = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, plan.states_per_block);
                if found.len() == expected.len() && found == expected {
                    ReadKind::MatchingBytes
                } else {
                    ReadKind::DifferentBytes
                }
            }
            ReadOutcome::NotFound => ReadKind::NotFound,
            ReadOutcome::ReadError(reason) => {
                if report.first_read_error.is_empty() {
                    report.first_read_error = format!("block={block_index}:{reason}");
                }
                ReadKind::ReadError
            }
        };
        match classify_block(class, read) {
            BlockVerdict::Correct => {}
            BlockVerdict::LostConfirmed => report.lost_confirmed.push(block_index),
            BlockVerdict::SilentlyCorrupted => report.silently_corrupted.push(block_index),
            BlockVerdict::ConfirmedReadError => report.confirmed_read_errors.push(block_index),
            BlockVerdict::UnconfirmedReadError => report.unconfirmed_read_errors.push(block_index),
            BlockVerdict::Phantom => {
                report.phantom_keys.insert(hexadecimal(&key));
            }
        }
    }
    let started_keys: HashSet<BlockKey> = (0..plan.direct_started)
        .map(|direct_position| interleaved_block_key(2 * direct_position, plan.states_per_block, &fingerprint))
        .chain((0..plan.network_sent).map(|network_position| {
            interleaved_block_key(network_path_block_index(network_position), plan.states_per_block, &fingerprint)
        }))
        .collect();
    match store.enumerate_keys_with_value_lengths() {
        Ok(entries) => {
            for (key, value_length) in entries {
                report.enumerated_key_count += 1;
                report.enumerated_value_bytes += value_length;
                let is_started = <[u8; BLOCK_KEY_BYTES]>::try_from(key.as_slice()).is_ok_and(|fixed_key| started_keys.contains(&fixed_key));
                if !is_started {
                    report.phantom_keys.insert(hexadecimal(&key));
                }
            }
        }
        Err(error) => report.enumeration_error = error.to_string(),
    }
    report
}

fn first_numbers(numbers: &[u64]) -> String {
    if numbers.is_empty() {
        "none".to_string()
    } else {
        numbers.iter().take(10).map(u64::to_string).collect::<Vec<_>>().join(",")
    }
}

impl VerificationReport {
    fn encode(&self) -> String {
        format!(
            "V open_failed={} open_error={} opened_after_repair={} open_microseconds={} confirmed_checked={} in_flight_checked={} not_started_checked={} lost_confirmed={} lost_confirmed_first={} silently_corrupted={} silently_corrupted_first={} confirmed_read_errors={} unconfirmed_read_errors={} read_errors_first={} phantom_keys={} enumerated_keys={} enumerated_value_bytes={} enumeration_error={} first_read_error={}",
            self.open_failed,
            text_field(&self.open_error),
            self.opened_after_repair.map_or("not_applicable".to_string(), |opened| opened.to_string()),
            self.open_microseconds,
            self.confirmed_checked,
            self.in_flight_checked,
            self.not_started_checked,
            self.lost_confirmed.len(),
            first_numbers(&self.lost_confirmed),
            self.silently_corrupted.len(),
            first_numbers(&self.silently_corrupted),
            self.confirmed_read_errors.len(),
            self.unconfirmed_read_errors.len(),
            first_numbers(&self.confirmed_read_errors.iter().chain(self.unconfirmed_read_errors.iter()).copied().collect::<Vec<_>>()),
            self.phantom_keys.len(),
            self.enumerated_key_count,
            self.enumerated_value_bytes,
            text_field(&self.enumeration_error),
            text_field(&self.first_read_error),
        )
    }
}

fn run_verify_role(arguments: &[String]) -> i32 {
    let values = match parse_flag_values(arguments) {
        Ok(values) => values,
        Err(problem) => {
            eprintln!("  ✗ verify 的参数不对：{problem}");
            return 2;
        }
    };
    let Some(candidate) = values.get("--arm").and_then(|text| Candidate::parse(text)) else {
        eprintln!("  ✗ verify 缺 --arm 或认不出");
        return 2;
    };
    let Some(library) = values.get("--library").map(PathBuf::from) else {
        eprintln!("  ✗ verify 缺 --library");
        return 2;
    };
    let plan = match PathVerificationPlan::parse(&values) {
        Ok(plan) => plan,
        Err(problem) => {
            eprintln!("  ✗ verify 的计划写坏了：{problem}");
            return 2;
        }
    };
    let report = verify_library(candidate, &library, &plan);
    print_flushed(&report.encode());
    0
}

// ============================== 父进程：起子进程、读它的行 ==============================

/// 起这个 bin 的一个子进程角色（library、forward、verify）；单测里走 libtest 自起（见 tests 模块）。
#[cfg(not(test))]
fn child_command(role_arguments: &[String]) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("找得到自己的可执行文件"));
    command.args(role_arguments);
    command
}

#[cfg(test)]
fn child_command(role_arguments: &[String]) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("找得到测试二进制"));
    command
        .args(["--exact", "tests::child_process_role_entry", "--nocapture", "--test-threads=1", "--quiet"])
        .env(tests::CHILD_ROLE_ENVIRONMENT, role_arguments.join("\u{1f}"));
    command
}

/// 送块进程：同一个目录下的 `e162-verdict-store-sender`；单测里是测试二进制自起送块角色。
#[cfg(not(test))]
fn sender_command(sender_arguments: &[String]) -> Command {
    let mut command = Command::new(sender_binary_path());
    command.args(sender_arguments);
    command
}

#[cfg(test)]
fn sender_command(sender_arguments: &[String]) -> Command {
    child_command(sender_arguments)
}

fn sender_binary_path() -> PathBuf {
    std::env::current_exe().expect("找得到自己的可执行文件").with_file_name(SENDER_BINARY_NAME)
}

/// 送块进程的参数；与 `SenderSettings::parse` 互为反过来（单测往返核）。
fn sender_arguments(settings: &SenderSettings) -> Vec<String> {
    let mut arguments = vec!["send".to_string()];
    match &settings.endpoint {
        SenderEndpoint::ConnectTcp(address) => {
            arguments.push("--connect".to_string());
            arguments.push(format!("tcp:{address}"));
        }
        SenderEndpoint::ListenUnix(path) => {
            arguments.push("--listen".to_string());
            arguments.push(format!("unix:{}", path.display()));
        }
    }
    let pace_milliseconds = settings
        .pace_between_first_transmissions
        .map_or(0, |pace| u64::try_from(pace.as_millis()).expect("节拍毫秒数装得进 u64"));
    for (flag, value) in [
        ("--blocks", settings.block_count.to_string()),
        ("--states-per-block", settings.states_per_block.to_string()),
        ("--window", settings.window_frames.to_string()),
        ("--pace-milliseconds", pace_milliseconds.to_string()),
        ("--stall-seconds", settings.acknowledgement_stall_limit.as_secs().to_string()),
        ("--spin-threads", settings.spinning_thread_count.to_string()),
    ] {
        arguments.push(flag.to_string());
        arguments.push(value);
    }
    arguments
}

struct RunningChild {
    child: Child,
    lines: mpsc::Receiver<String>,
    input: Option<ChildStdin>,
}

fn spawn_running_child(mut command: Command, keep_input_open: bool) -> RunningChild {
    let mut child = command
        .stdin(if keep_input_open { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("起得了子进程");
    let standard_output = child.stdout.take().expect("子进程的标准输出接在管道上");
    let input = child.stdin.take();
    let (line_sender, lines) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(standard_output).lines() {
            let Ok(line) = line else { break };
            if line_sender.send(line).is_err() {
                break;
            }
        }
    });
    RunningChild { child, lines, input }
}

/// 等子进程打出以 prefix 起头的一行；子进程先退了或超时返回 None，已读到的行放进 seen。
fn wait_for_line(running: &RunningChild, prefix: &str, seen: &mut Vec<String>) -> Option<String> {
    let deadline = Instant::now() + CHILD_START_DEADLINE;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match running.lines.recv_timeout(remaining) {
            Ok(line) if line.starts_with(prefix) => return Some(line),
            Ok(line) => seen.push(line),
            Err(_ended_or_timed_out) => return None,
        }
    }
}

/// 对子进程的 pid 发 SIGKILL（`Child::kill` 在 Unix 上发的就是 SIGKILL），再收掉它。
fn kill_and_reap(child: &mut Child) {
    let _already_exited = child.kill();
    child.wait().expect("收得了被杀的子进程");
}

fn field_value<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    line.split(' ').find_map(|field| field.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
}

fn fields_of(line: &str) -> Vec<(String, String)> {
    line.split(' ')
        .filter_map(|field| field.split_once('=').map(|(key, value)| (key.to_string(), value.to_string())))
        .collect()
}

fn number_field(line: &str, key: &str) -> u64 {
    field_value(line, key).and_then(|text| text.parse().ok()).unwrap_or(0)
}

// ============================== 盘上位置与环境核查（同 S1 装置） ==============================

/// `${TMPDIR:-/tmp}/e162-<pid>/`，Drop 时整目录删掉；E162_KEEP_SCRATCH=1 时留下并打印路径。
struct ScratchDirectory {
    path: PathBuf,
}

impl ScratchDirectory {
    fn create() -> ScratchDirectory {
        let base = std::env::var_os("TMPDIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
        let path = base.join(format!("e162-{}", std::process::id()));
        fs::create_dir_all(&path).expect("建得了盘上位置");
        ScratchDirectory { path }
    }
}

impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        if std::env::var("E162_KEEP_SCRATCH").is_ok_and(|value| value == "1") {
            eprintln!("E162_KEEP_SCRATCH=1：现场留在 {}", self.path.display());
        } else {
            let _cleanup_outcome = fs::remove_dir_all(&self.path);
        }
    }
}

fn command_output_text(program: &str, arguments: &[&str]) -> String {
    Command::new(program)
        .args(arguments)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default()
}

/// V9：`stat -f -c %T`；H4：`df -B1` 的可用字节。
fn filesystem_type_and_available_bytes(directory: &Path) -> (String, u64) {
    let directory_text = directory.to_string_lossy().to_string();
    let filesystem_type = command_output_text("stat", &["-f", "-c", "%T", &directory_text]);
    let available_text = command_output_text("df", &["-B1", "--output=avail", &directory_text]);
    let available = available_text.lines().last().and_then(|line| line.trim().parse().ok()).unwrap_or(0);
    (filesystem_type, available)
}

/// V15：本用户的进程里有没有会干扰计时的（按可执行文件名与参数逐个比，不用模式串比整条命令行）。
fn interfering_processes() -> Vec<String> {
    use std::os::unix::fs::MetadataExt;
    let own_user = fs::metadata("/proc/self").map(|metadata| metadata.uid()).unwrap_or(u32::MAX);
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else { return found };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        if fs::metadata(entry.path()).map(|metadata| metadata.uid()).ok() != Some(own_user) {
            continue;
        }
        let Ok(command_line) = fs::read(entry.path().join("cmdline")) else { continue };
        let words: Vec<String> = command_line
            .split(|&byte| byte == 0)
            .filter(|word| !word.is_empty())
            .map(|word| String::from_utf8_lossy(word).to_string())
            .collect();
        let Some(program) = words.first() else { continue };
        let program_name = Path::new(program).file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default();
        let interferes = program_name.starts_with("qemu-system")
            || program_name == "cargo"
            || program_name == "rustc"
            || program_name == "fio"
            || program_name == "e152-file-system-benchmark"
            || words.iter().any(|word| word.ends_with("vm-bench.sh") || word.contains("e161"));
        if interferes {
            found.push(format!("{name}:{program_name}"));
        }
    }
    found.sort();
    found
}

fn load_average_text() -> String {
    fs::read_to_string("/proc/loadavg").unwrap_or_default().split_whitespace().take(3).collect::<Vec<_>>().join("/")
}

/// /proc/diskstats 里 nvme0n1 的写扇区数（第 10 列）。
fn nvme_written_sectors() -> u64 {
    fs::read_to_string("/proc/diskstats")
        .unwrap_or_default()
        .lines()
        .find_map(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            (columns.get(2) == Some(&"nvme0n1")).then(|| columns.get(9).and_then(|value| value.parse().ok())).flatten()
        })
        .unwrap_or(0)
}

fn list_or_dash(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(",")
    }
}

/// 修订五：每遍开跑之前先等干扰进程散去，每 30 秒看一次、最多等 20 分钟；返回等了多少秒。
fn wait_for_quiet_machine(maximum_wait_seconds: u64) -> u64 {
    let started = Instant::now();
    while !interfering_processes().is_empty() && started.elapsed().as_secs() < maximum_wait_seconds {
        std::thread::sleep(Duration::from_secs(INTERFERENCE_POLL_SECONDS));
    }
    started.elapsed().as_secs()
}

// ============================== 回环格：一格怎么跑 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LoopbackCell {
    label: &'static str,
    direct_blocks: u64,
    network_blocks: u64,
    states_per_block: u64,
    window_frames: usize,
    receive_mode: ReceiveMode,
    interruption_count: usize,
    /// V15 管的计时格（开始与结束各看一次干扰进程）。
    is_timed: bool,
}

impl LoopbackCell {
    fn throughput_shape(label: &'static str, window_frames: usize, states_per_block: u64, receive_mode: ReceiveMode) -> LoopbackCell {
        LoopbackCell {
            label,
            direct_blocks: PATH_BLOCK_COUNT,
            network_blocks: PATH_BLOCK_COUNT,
            states_per_block,
            window_frames,
            receive_mode,
            interruption_count: 0,
            is_timed: true,
        }
    }

    fn consistency_shape(label: &'static str, receive_mode: ReceiveMode, interruption_count: usize) -> LoopbackCell {
        LoopbackCell {
            label,
            direct_blocks: PATH_BLOCK_COUNT,
            network_blocks: PATH_BLOCK_COUNT,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            window_frames: PRIMARY_WINDOW_FRAMES,
            receive_mode,
            interruption_count,
            is_timed: false,
        }
    }

    fn rate_control_shape(label: &'static str, receive_mode: ReceiveMode) -> LoopbackCell {
        LoopbackCell {
            label,
            direct_blocks: 0,
            network_blocks: RATE_CONTROL_BLOCKS,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            window_frames: PRIMARY_WINDOW_FRAMES,
            receive_mode,
            interruption_count: 0,
            is_timed: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct InterruptionRecord {
    index: usize,
    threshold: u64,
    acknowledged_seen: u64,
    pause_microseconds: u64,
    forwarder_restarted: bool,
}

#[derive(Debug, Clone, Default)]
struct CellRun {
    sender_summary: String,
    sender_windows: String,
    sender_reconnections: Vec<String>,
    sender_exit_code: i32,
    library_direct: String,
    library_network: String,
    library_exit_code: i32,
    interruptions: Vec<InterruptionRecord>,
    verification: VerificationReport,
    setup_problem: String,
}

impl CellRun {
    fn sender_blocks_per_second(&self) -> f64 {
        let acknowledged = number_field(&self.sender_summary, "acknowledged") as f64;
        let elapsed_seconds = number_field(&self.sender_summary, "elapsed_microseconds") as f64 / 1_000_000.0;
        if elapsed_seconds > 0.0 {
            acknowledged / elapsed_seconds
        } else {
            0.0
        }
    }

    fn direct_blocks_per_second(&self) -> f64 {
        let confirmed = number_field(&self.library_direct, "confirmed") as f64;
        let elapsed_seconds = number_field(&self.library_direct, "elapsed_microseconds") as f64 / 1_000_000.0;
        if elapsed_seconds > 0.0 {
            confirmed / elapsed_seconds
        } else {
            0.0
        }
    }

    fn read_errors(&self) -> u64 {
        u64::try_from(self.verification.confirmed_read_errors.len() + self.verification.unconfirmed_read_errors.len()).expect("装得进 u64")
    }
}

fn spawn_forwarder(listen_port: u16, target_port: u16) -> Option<(RunningChild, u16)> {
    let arguments = vec![
        "forward".to_string(),
        "--listen-port".to_string(),
        listen_port.to_string(),
        "--target-port".to_string(),
        target_port.to_string(),
    ];
    let mut running = spawn_running_child(child_command(&arguments), false);
    let mut seen = Vec::new();
    match wait_for_line(&running, "F port=", &mut seen) {
        Some(line) => {
            let port = u16::try_from(number_field(&line, "port")).expect("端口装得进 u16");
            Some((running, port))
        }
        None => {
            kill_and_reap(&mut running.child);
            None
        }
    }
}

/// 在子进程里核，3 600 秒没交回就杀（登记 5.5）。
fn verify_in_child_with_timeout(candidate: Candidate, library: &Path, plan: &PathVerificationPlan) -> VerificationReport {
    let mut running = spawn_running_child(child_command(&plan.to_arguments(candidate, library)), false);
    let deadline = Instant::now() + Duration::from_secs(VERIFICATION_TIMEOUT_SECONDS);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match running.lines.recv_timeout(remaining) {
            Ok(line) if line.starts_with("V ") => {
                let _exit_status = running.child.wait();
                return decode_verification_line(&line);
            }
            Ok(_other_line) => continue,
            Err(_ended_or_timed_out) => {
                kill_and_reap(&mut running.child);
                return VerificationReport { verification_missing: true, ..VerificationReport::default() };
            }
        }
    }
}

/// 核对子进程的结果行只带计数与前 10 个块号；父进程按计数还原成同样长的列表（块号只在产物行里看前 10 个）。
fn decode_verification_line(line: &str) -> VerificationReport {
    let first_list = |key: &str| -> Vec<u64> {
        field_value(line, key)
            .filter(|text| *text != "none")
            .map(|text| text.split(',').filter_map(|number| number.parse().ok()).collect())
            .unwrap_or_default()
    };
    let padded = |count: u64, first: Vec<u64>| -> Vec<u64> {
        let mut numbers = first;
        numbers.resize(usize::try_from(count).expect("装得进 usize"), u64::MAX);
        numbers
    };
    let read_error_first = first_list("read_errors_first");
    let confirmed_read_errors = number_field(line, "confirmed_read_errors");
    let unconfirmed_read_errors = number_field(line, "unconfirmed_read_errors");
    let phantom_count = number_field(line, "phantom_keys");
    VerificationReport {
        open_failed: field_value(line, "open_failed") == Some("true"),
        open_error: field_value(line, "open_error").unwrap_or("none").to_string(),
        opened_after_repair: match field_value(line, "opened_after_repair") {
            Some("true") => Some(true),
            Some("false") => Some(false),
            _ => None,
        },
        open_microseconds: number_field(line, "open_microseconds"),
        confirmed_checked: number_field(line, "confirmed_checked"),
        in_flight_checked: number_field(line, "in_flight_checked"),
        not_started_checked: number_field(line, "not_started_checked"),
        lost_confirmed: padded(number_field(line, "lost_confirmed"), first_list("lost_confirmed_first")),
        silently_corrupted: padded(number_field(line, "silently_corrupted"), first_list("silently_corrupted_first")),
        confirmed_read_errors: padded(confirmed_read_errors, read_error_first.iter().take(usize::try_from(confirmed_read_errors).expect("usize")).copied().collect()),
        unconfirmed_read_errors: padded(unconfirmed_read_errors, Vec::new()),
        phantom_keys: (0..phantom_count).map(|phantom_index| format!("phantom-{phantom_index}")).collect(),
        enumerated_key_count: number_field(line, "enumerated_keys"),
        enumerated_value_bytes: number_field(line, "enumerated_value_bytes"),
        enumeration_error: field_value(line, "enumeration_error").unwrap_or("none").to_string(),
        first_read_error: field_value(line, "first_read_error").unwrap_or("none").to_string(),
        verification_missing: false,
    }
}

/// 一格回环：起库进程（直写线程 + 收块线程）、转发进程、送块进程；按送块进程报的确认数中断转发进程；
/// 送块进程退出之后停转发、关库进程的标准输入让它收尾；再在核对子进程里逐块核两路。
fn run_loopback_cell(candidate: Candidate, cell: &LoopbackCell, scratch: &Path) -> CellRun {
    let mut run = CellRun::default();
    let library = scratch.join(format!("library-{}-{}", candidate.label(), cell.label));
    if library.exists() {
        fs::remove_dir_all(&library).expect("删得了上一次的库");
    }
    let library_settings = LibrarySettings {
        candidate,
        library: library.clone(),
        receive_mode: cell.receive_mode,
        direct_blocks: cell.direct_blocks,
        network_blocks: cell.network_blocks,
        states_per_block: cell.states_per_block,
        channel: ReceiveChannel::TcpListen,
    };
    let mut library_process = spawn_running_child(child_command(&library_settings.to_arguments()), true);
    let mut library_lines = Vec::new();
    let Some(port_line) = wait_for_line(&library_process, "L port=", &mut library_lines) else {
        kill_and_reap(&mut library_process.child);
        run.setup_problem = format!("library_did_not_start:{}", text_field(&library_lines.join("|")));
        return run;
    };
    let library_port = u16::try_from(number_field(&port_line, "port")).expect("端口装得进 u16");
    let Some((mut forwarder, forward_port)) = spawn_forwarder(0, library_port) else {
        kill_and_reap(&mut library_process.child);
        run.setup_problem = "forwarder_did_not_start".to_string();
        return run;
    };
    let sender_settings = SenderSettings {
        endpoint: SenderEndpoint::ConnectTcp(format!("127.0.0.1:{forward_port}")),
        block_count: cell.network_blocks,
        states_per_block: cell.states_per_block,
        window_frames: cell.window_frames,
        pace_between_first_transmissions: None,
        acknowledgement_stall_limit: Duration::from_secs(DEFAULT_ACKNOWLEDGEMENT_STALL_SECONDS),
        spinning_thread_count: 0,
    };
    let mut sender = spawn_running_child(sender_command(&sender_arguments(&sender_settings)), false);
    let plan = interruption_plan(cell.interruption_count, cell.network_blocks.max(2));
    let mut next_interruption = 0usize;
    for line in sender.lines.iter() {
        if let Some(count_text) = line.strip_prefix("A ") {
            let acknowledged: u64 = count_text.trim().parse().unwrap_or(0);
            while next_interruption < plan.len() && acknowledged >= plan[next_interruption].acknowledgement_threshold {
                let interruption = plan[next_interruption];
                kill_and_reap(&mut forwarder.child);
                let paused_at = Instant::now();
                std::thread::sleep(pause_of(&interruption));
                let pause_microseconds = u64::try_from(paused_at.elapsed().as_micros()).expect("微秒装得进 u64");
                let restarted = spawn_forwarder(forward_port, library_port);
                let forwarder_restarted = restarted.is_some();
                if let Some((replacement, _same_port)) = restarted {
                    forwarder = replacement;
                }
                run.interruptions.push(InterruptionRecord {
                    index: next_interruption,
                    threshold: interruption.acknowledgement_threshold,
                    acknowledged_seen: acknowledged,
                    pause_microseconds,
                    forwarder_restarted,
                });
                next_interruption += 1;
            }
        } else if line.starts_with("R ") {
            run.sender_reconnections.push(line);
        } else if line.starts_with("W ") {
            run.sender_windows = line;
        } else if line.starts_with("S ") {
            run.sender_summary = line;
        }
    }
    run.sender_exit_code = sender.child.wait().map(|status| status.code().unwrap_or(-1)).unwrap_or(-1);
    kill_and_reap(&mut forwarder.child);
    drop(library_process.input.take());
    for line in library_process.lines.iter() {
        if line.starts_with("D ") {
            run.library_direct = line;
        } else if line.starts_with("N ") {
            run.library_network = line;
        }
    }
    run.library_exit_code = library_process.child.wait().map(|status| status.code().unwrap_or(-1)).unwrap_or(-1);
    let network_acknowledged = expand_true_positions(field_value(&run.sender_summary, "acknowledged_ranges").unwrap_or("none"), cell.network_blocks)
        .unwrap_or_else(|| vec![false; usize::try_from(cell.network_blocks).expect("块数装得进 usize")]);
    let verification_plan = PathVerificationPlan {
        states_per_block: cell.states_per_block,
        direct_blocks: cell.direct_blocks,
        direct_confirmed: number_field(&run.library_direct, "confirmed"),
        direct_started: number_field(&run.library_direct, "started"),
        network_blocks: cell.network_blocks,
        network_acknowledged,
        network_sent: number_field(&run.sender_summary, "sent_positions"),
    };
    run.verification = verify_in_child_with_timeout(candidate, &library, &verification_plan);
    let _library_removed = fs::remove_dir_all(&library);
    run
}

// ============================== 回环格：驱动、产物行与判定 ==============================

/// 一格的一次跑（计时格 V15 干扰就重跑，最多重跑三次）。
struct CellAttempt {
    run: CellRun,
    attempt: usize,
    is_interfered: bool,
}

fn run_cell_with_interference_reruns(output: &mut ProductLines, candidate: Candidate, cell: &LoopbackCell, scratch: &Path) -> CellAttempt {
    let mut attempt = 0usize;
    loop {
        attempt += 1;
        let waited_seconds = if cell.is_timed { wait_for_quiet_machine(INTERFERENCE_MAXIMUM_WAIT_SECONDS) } else { 0 };
        let interference_at_start = interfering_processes();
        let load_at_start = load_average_text();
        let sectors_at_start = nvme_written_sectors();
        let run = run_loopback_cell(candidate, cell, scratch);
        let interference_at_end = interfering_processes();
        let is_interfered = cell.is_timed && !(interference_at_start.is_empty() && interference_at_end.is_empty());
        output.line(format!(
            "name=cell arm={} cell={} attempt={attempt} direct_blocks={} network_blocks={} states_per_block={} window_frames={} receive_mode={} interruptions_planned={} is_timed={} waited_for_quiet_seconds={waited_seconds} interference_at_start={} interference_at_end={} load_at_start={load_at_start} load_at_end={} nvme_written_sectors_at_start={sectors_at_start} nvme_written_sectors_at_end={} setup_problem={}",
            candidate.label(),
            cell.label,
            cell.direct_blocks,
            cell.network_blocks,
            cell.states_per_block,
            cell.window_frames,
            cell.receive_mode.label(),
            cell.interruption_count,
            cell.is_timed,
            list_or_dash(&interference_at_start),
            list_or_dash(&interference_at_end),
            load_average_text(),
            nvme_written_sectors(),
            text_field(&run.setup_problem),
        ));
        emit_cell_lines(output, candidate, cell, attempt, &run);
        if !is_interfered || attempt > INTERFERENCE_MAXIMUM_RERUNS {
            return CellAttempt { run, attempt, is_interfered };
        }
    }
}

fn prefixed_fields(line: &str, skip_first_word: bool) -> String {
    let body = if skip_first_word { line.split_once(' ').map_or("", |(_, rest)| rest) } else { line };
    if body.is_empty() {
        "missing=true".to_string()
    } else {
        body.to_string()
    }
}

/// 分窗：第 k 窗 = 第 1 000(k−1) 到第 1 000k 个确认，状态/秒 = 1 000 × 每块状态数 ÷ 窗长（第 1 窗从第一帧发出算起）。
fn window_states_per_second(end_offsets_microseconds: &[u64], states_per_block: u64) -> Vec<f64> {
    let mut previous = 0u64;
    end_offsets_microseconds
        .iter()
        .map(|&offset| {
            let window_seconds = (offset - previous) as f64 / 1_000_000.0;
            previous = offset;
            (ACKNOWLEDGEMENTS_PER_WINDOW * states_per_block) as f64 / window_seconds
        })
        .collect()
}

fn parse_window_offsets(windows_line: &str) -> Vec<u64> {
    field_value(windows_line, "end_offsets_microseconds")
        .filter(|text| *text != "none")
        .map(|text| text.split(',').filter_map(|number| number.parse().ok()).collect())
        .unwrap_or_default()
}

fn emit_cell_lines(output: &mut ProductLines, candidate: Candidate, cell: &LoopbackCell, attempt: usize, run: &CellRun) {
    let arm = candidate.label();
    let label = cell.label;
    output.line(format!("name=sender arm={arm} cell={label} attempt={attempt} exit_code={} {}", run.sender_exit_code, prefixed_fields(&run.sender_summary, true)));
    output.line(format!("name=sender_windows arm={arm} cell={label} attempt={attempt} {}", prefixed_fields(&run.sender_windows, true)));
    for reconnection in &run.sender_reconnections {
        output.line(format!("name=sender_reconnection arm={arm} cell={label} attempt={attempt} {}", prefixed_fields(reconnection, true)));
    }
    for interruption in &run.interruptions {
        output.line(format!(
            "name=interruption arm={arm} cell={label} attempt={attempt} index={} threshold={} acknowledged_seen={} pause_microseconds={} forwarder_restarted={}",
            interruption.index, interruption.threshold, interruption.acknowledged_seen, interruption.pause_microseconds, interruption.forwarder_restarted
        ));
    }
    let direct_fields: Vec<String> = fields_of(&run.library_direct).into_iter().map(|(key, value)| format!("direct_{key}={value}")).collect();
    output.line(format!(
        "name=library arm={arm} cell={label} attempt={attempt} exit_code={} {} {}",
        run.library_exit_code,
        if direct_fields.is_empty() { "direct_missing=true".to_string() } else { direct_fields.join(" ") },
        prefixed_fields(&run.library_network, true)
    ));
    let expected_keys = cell.direct_blocks + cell.network_blocks;
    let expected_bytes = expected_keys * cell.states_per_block;
    let verification = &run.verification;
    output.line(format!(
        "name=verification arm={arm} cell={label} attempt={attempt} verification_missing={} {} q3d_lost_confirmed={} q3e_silently_corrupted={} q3f_read_errors={} q3g_phantom_keys={} a_count_expected_keys={expected_keys} a_count_expected_value_bytes={expected_bytes} a_count_matches={}",
        verification.verification_missing,
        prefixed_fields(&verification.encode(), true),
        verification.lost_confirmed.len(),
        verification.silently_corrupted.len(),
        run.read_errors(),
        verification.phantom_keys.len(),
        verification.enumerated_key_count == expected_keys && verification.enumerated_value_bytes == expected_bytes,
    ));
    let network_blocks_per_second = run.sender_blocks_per_second();
    let direct_blocks_per_second = run.direct_blocks_per_second();
    let states = cell.states_per_block as f64;
    let window_rates = window_states_per_second(&parse_window_offsets(&run.sender_windows), cell.states_per_block);
    let lowest_window = window_rates.iter().copied().fold(f64::INFINITY, f64::min);
    output.line(format!(
        "name=rates arm={arm} cell={label} attempt={attempt} q3a_blocks_per_second={network_blocks_per_second:.3} q3a_states_per_second={:.1} q3b_blocks_per_second={direct_blocks_per_second:.3} q3b_states_per_second={:.1} q3c_states_per_second={:.1} q3k_windows={} q3k_lowest_window_states_per_second={} q3k_last_window_states_per_second={} q3k_window_states_per_second={}",
        network_blocks_per_second * states,
        direct_blocks_per_second * states,
        (network_blocks_per_second + direct_blocks_per_second) * states,
        window_rates.len(),
        if window_rates.is_empty() { "none".to_string() } else { format!("{lowest_window:.1}") },
        window_rates.last().map_or("none".to_string(), |rate| format!("{rate:.1}")),
        if window_rates.is_empty() { "none".to_string() } else { window_rates.iter().map(|rate| format!("{rate:.0}")).collect::<Vec<_>>().join(",") },
    ));
}

/// Q3a 对 R_B 的判定（第八节末段的判别力自证用同一个函数）：低于门槛就翻。
fn network_throughput_flips(states_per_second: f64, threshold_states_per_second: f64) -> bool {
    states_per_second < threshold_states_per_second
}

/// PC3-rate：慢的那遍 ≤ 200 块/秒、快的那遍 > 200 块/秒（登记 5.3）。
fn rate_control_seen(fast_blocks_per_second: f64, slow_blocks_per_second: f64) -> bool {
    slow_blocks_per_second <= RATE_CONTROL_THRESHOLD_BLOCKS_PER_SECOND && fast_blocks_per_second > RATE_CONTROL_THRESHOLD_BLOCKS_PER_SECOND
}

const LOOPBACK_CELL_ORDER: [&str; 9] = ["PC3-drop", "PC3-rate", "PC3-torn", "S3-T", "S3-C", "S3-W1", "S3-W256", "S3-bs10", "S3-raw"];

fn run_loopback_question(output: &mut ProductLines, arguments: &[String]) -> i32 {
    let Some(candidate) = arguments.first().and_then(|text| Candidate::parse(text)) else {
        eprintln!("  ✗ s3 要一个臂：R1 R0 K F");
        return 2;
    };
    let requested: Vec<&str> = if arguments.len() > 1 { arguments[1..].iter().map(String::as_str).collect() } else { LOOPBACK_CELL_ORDER.to_vec() };
    if let Some(unknown) = requested.iter().find(|label| !LOOPBACK_CELL_ORDER.contains(label)) {
        eprintln!("  ✗ 认不出的格 {unknown}；格取 {}", LOOPBACK_CELL_ORDER.join(" "));
        return 2;
    }
    if !sender_binary_path().exists() {
        eprintln!("  ✗ 找不到送块进程 {}", sender_binary_path().display());
        eprintln!("  → 怎么办：先 cargo build --release -p e7-index-bench --bin {SENDER_BINARY_NAME}（与这个 bin 编进同一个目录）");
        return 2;
    }
    if run_anchors(output) != 0 {
        return 1;
    }
    let scratch = ScratchDirectory::create();
    let (filesystem_type, available_bytes) = filesystem_type_and_available_bytes(&scratch.path);
    output.line(format!(
        "name=environment arm={} scratch={} filesystem_type={filesystem_type} available_bytes={available_bytes} logical_cpus={} kernel={}",
        candidate.label(),
        scratch.path.display(),
        std::thread::available_parallelism().map(|count| count.get()).unwrap_or(0),
        command_output_text("uname", &["-r"]),
    ));
    output.line(format!(
        "name=threshold arm={} t_state=pending_e161_segment_one peer_logical_cpus=pending_not_queried_this_segment r_b=pending r_ab=pending q3i_peer_cores_break_even=pending q3j_cross_link_bytes_per_second_needed=pending formula_q3j=r_b_times_65580_over_65536",
        candidate.label()
    ));
    if matches!(filesystem_type.as_str(), "tmpfs" | "ramfs" | "overlayfs") {
        output.line(format!("name=verdict part=environment arm={} v9_filesystem_allowed=false filesystem_type={filesystem_type}", candidate.label()));
        return 1;
    }
    if available_bytes < MINIMUM_AVAILABLE_BYTES {
        output.line(format!("name=verdict part=environment arm={} h4_available_bytes_enough=false available_bytes={available_bytes}", candidate.label()));
        return 1;
    }
    let arm = candidate.label();
    for label in requested {
        match label {
            "PC3-drop" => {
                let cell = LoopbackCell::consistency_shape("PC3-drop", ReceiveMode::DropOnePerThousand, 0);
                let attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                let lost = u64::try_from(attempt.run.verification.lost_confirmed.len()).expect("装得进 u64");
                let recorded = number_field(&attempt.run.library_network, "controlled_drops");
                output.line(format!(
                    "name=verdict part=pc3_drop arm={arm} lost_confirmed={lost} library_controlled_drops={recorded} positive_control_passed={}",
                    lost == recorded && recorded >= 1
                ));
            }
            "PC3-rate" => {
                let fast = run_cell_with_interference_reruns(output, candidate, &LoopbackCell::rate_control_shape("PC3-rate-fast", ReceiveMode::Registered), &scratch.path);
                let slow = run_cell_with_interference_reruns(output, candidate, &LoopbackCell::rate_control_shape("PC3-rate-slow", ReceiveMode::SleepBeforeCommit), &scratch.path);
                let fast_rate = fast.run.sender_blocks_per_second();
                let slow_rate = slow.run.sender_blocks_per_second();
                output.line(format!(
                    "name=verdict part=pc3_rate arm={arm} fast_blocks_per_second={fast_rate:.3} slow_blocks_per_second={slow_rate:.3} positive_control_passed={} timing_undisturbed={}",
                    rate_control_seen(fast_rate, slow_rate),
                    !fast.is_interfered && !slow.is_interfered
                ));
            }
            "PC3-torn" => {
                let mut cell = LoopbackCell::consistency_shape("PC3-torn", ReceiveMode::TornAccepting, REGISTERED_INTERRUPTIONS);
                let mut attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                let mut escalated = false;
                if attempt.run.verification.silently_corrupted.is_empty() {
                    escalated = true;
                    cell = LoopbackCell::consistency_shape("PC3-torn-200", ReceiveMode::TornAccepting, ESCALATED_INTERRUPTIONS);
                    attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                }
                output.line(format!(
                    "name=verdict part=pc3_torn arm={arm} interruptions={} escalated_to_200={escalated} silently_corrupted={} positive_control_passed={}",
                    cell.interruption_count,
                    attempt.run.verification.silently_corrupted.len(),
                    !attempt.run.verification.silently_corrupted.is_empty()
                ));
            }
            "S3-C" => {
                let mut cell = LoopbackCell::consistency_shape("S3-C", ReceiveMode::Registered, REGISTERED_INTERRUPTIONS);
                let mut attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                let paths_reached = disconnect_paths_reached;
                let mut escalated = false;
                if !paths_reached(&attempt.run) {
                    escalated = true;
                    cell = LoopbackCell::consistency_shape("S3-C-200", ReceiveMode::Registered, ESCALATED_INTERRUPTIONS);
                    attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                }
                emit_consistency_verdict(output, arm, &cell, &attempt, escalated, paths_reached(&attempt.run));
            }
            "S3-T" | "S3-W1" | "S3-W256" | "S3-bs10" | "S3-raw" => {
                let cell = match label {
                    "S3-T" => LoopbackCell::throughput_shape("S3-T", PRIMARY_WINDOW_FRAMES, PRIMARY_STATES_PER_BLOCK, ReceiveMode::Registered),
                    "S3-W1" => LoopbackCell::throughput_shape("S3-W1", STOP_AND_WAIT_WINDOW_FRAMES, PRIMARY_STATES_PER_BLOCK, ReceiveMode::Registered),
                    "S3-W256" => LoopbackCell::throughput_shape("S3-W256", WIDE_WINDOW_FRAMES, PRIMARY_STATES_PER_BLOCK, ReceiveMode::Registered),
                    "S3-bs10" => LoopbackCell::throughput_shape("S3-bs10", PRIMARY_WINDOW_FRAMES, SMALL_STATES_PER_BLOCK, ReceiveMode::Registered),
                    _ => LoopbackCell::throughput_shape("S3-raw", PRIMARY_WINDOW_FRAMES, PRIMARY_STATES_PER_BLOCK, ReceiveMode::AcknowledgeWithoutWriting),
                };
                let attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                let states = cell.states_per_block as f64;
                let network_states_per_second = attempt.run.sender_blocks_per_second() * states;
                let direct_states_per_second = attempt.run.direct_blocks_per_second() * states;
                output.line(format!(
                    "name=verdict part=s3_throughput arm={arm} cell={} network_states_per_second={network_states_per_second:.1} direct_states_per_second={direct_states_per_second:.1} both_paths_states_per_second={:.1} judgement_against_r_b=pending_t_state_and_peer_cores timing_undisturbed={} attempts={} sender_complete={}",
                    cell.label,
                    network_states_per_second + direct_states_per_second,
                    !attempt.is_interfered,
                    attempt.attempt,
                    field_value(&attempt.run.sender_summary, "ending") == Some("complete"),
                ));
                if label != "S3-raw" {
                    emit_consistency_verdict(output, arm, &cell, &attempt, false, true);
                }
            }
            _ => unreachable!("格名在上面已经核过"),
        }
    }
    0
}

/// V14：S3-C 的中断走没走到半帧与重发两条路。原读法是半帧、重发都 > 0；加修订里收严的一条：库进程至少收过两条连接
/// （至少一次重连真把帧送进了库；连上正在退出的转发进程的那一次到不了库）。
fn disconnect_paths_reached(run: &CellRun) -> bool {
    number_field(&run.library_network, "half_frames_dropped") > 0
        && number_field(&run.sender_summary, "resent_frames") > 0
        && number_field(&run.library_network, "connections") >= 2
}

fn emit_consistency_verdict(output: &mut ProductLines, arm: &str, cell: &LoopbackCell, attempt: &CellAttempt, escalated: bool, paths_reached: bool) {
    let run = &attempt.run;
    let expected_keys = cell.direct_blocks + cell.network_blocks;
    let expected_bytes = expected_keys * cell.states_per_block;
    output.line(format!(
        "name=verdict part=s3_consistency arm={arm} cell={} q3d_lost_confirmed={} q3e_silently_corrupted={} q3f_read_errors={} q3g_phantom_keys={} open_failed={} verification_missing={} a_count_matches={} half_frames_dropped={} resent_frames={} library_connections={} disconnect_paths_reached={} escalated_to_200={escalated} sender_complete={}",
        cell.label,
        run.verification.lost_confirmed.len(),
        run.verification.silently_corrupted.len(),
        run.read_errors(),
        run.verification.phantom_keys.len(),
        run.verification.open_failed,
        run.verification.verification_missing,
        run.verification.enumerated_key_count == expected_keys && run.verification.enumerated_value_bytes == expected_bytes,
        number_field(&run.library_network, "half_frames_dropped"),
        number_field(&run.sender_summary, "resent_frames"),
        number_field(&run.library_network, "connections"),
        if cell.interruption_count == 0 { "not_applicable".to_string() } else { paths_reached.to_string() },
        field_value(&run.sender_summary, "ending") == Some("complete"),
    ));
}

/// 第八节末段的判别力自证：拿两个 Q3a（状态/秒），门槛设成中点时一翻一不翻，设在两者之外时两点判得相同。
fn run_discrimination(output: &mut ProductLines, arguments: &[String]) -> i32 {
    let values: Vec<f64> = arguments.iter().filter_map(|text| text.parse().ok()).collect();
    let [first, second] = values[..] else {
        eprintln!("  ✗ discrimination 要两个 Q3a（状态/秒）");
        return 2;
    };
    let midpoint = (first + second) / 2.0;
    let below_both = first.min(second) / 2.0;
    let above_both = first.max(second) * 2.0;
    let midpoint_separates = network_throughput_flips(first, midpoint) != network_throughput_flips(second, midpoint);
    let outside_agree = network_throughput_flips(first, below_both) == network_throughput_flips(second, below_both)
        && network_throughput_flips(first, above_both) == network_throughput_flips(second, above_both);
    output.line(format!(
        "name=verdict part=discrimination first={first} second={second} midpoint={midpoint} midpoint_separates={midpoint_separates} outside_thresholds_agree={outside_agree}"
    ));
    i32::from(!(midpoint_separates && outside_agree))
}

// ============================== 单测 ==============================

#[cfg(test)]
mod tests {
    use super::*;

    /// 单测里起子进程角色：测试二进制带着这个环境变量、只跑 `child_process_role_entry` 一个测试。
    pub(super) const CHILD_ROLE_ENVIRONMENT: &str = "E162_S3_TEST_CHILD_ROLE";

    /// 跨机格单测里的中继：库进程（本身是带着 CHILD_ROLE_ENVIRONMENT 起的子进程）经 `sh -c` 起的孙进程，
    /// 另用这个环境变量带中继的目标，免得与 CHILD_ROLE_ENVIRONMENT 的 `\u{1f}` 分隔嵌在一起。
    const RELAY_TARGET_ENVIRONMENT: &str = "E162_S3_TEST_RELAY_TARGET";

    #[test]
    fn child_process_role_entry() {
        if let Ok(target) = std::env::var(RELAY_TARGET_ENVIRONMENT) {
            use std::os::fd::FromRawFd;
            let path = target.strip_prefix("unix:").expect("中继目标写成 unix:<路径>").to_string();
            // SAFETY: 单测的对端命令用 `3>&1 1>/dev/null` 把数据通道接在 3 号描述符上（libtest 往 1 号打的字不进数据通道），
            // 这个进程里没有别的东西持有或关掉 3 号描述符。
            let data_channel = unsafe { File::from_raw_fd(3) };
            std::process::exit(sender_role::relay_between(&path, Box::new(data_channel)));
        }
        let Ok(joined) = std::env::var(CHILD_ROLE_ENVIRONMENT) else { return };
        let arguments: Vec<String> = joined.split('\u{1f}').map(str::to_string).collect();
        let remaining = &arguments[1..];
        let exit_code = match arguments[0].as_str() {
            "library" => run_library_role(remaining),
            "forward" => run_forward_role(remaining),
            "verify" => run_verify_role(remaining),
            "send" => sender_role::run_send_role(remaining),
            "relay" => sender_role::run_relay_role(remaining),
            other => panic!("认不出的子进程角色 {other}"),
        };
        std::process::exit(exit_code);
    }

    /// 每个单测一个自己的临时目录，测完删掉。
    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new(name: &str) -> TestDirectory {
            let path = std::env::temp_dir().join(format!("e162-s3-test-{}-{name}", std::process::id()));
            if path.exists() {
                fs::remove_dir_all(&path).expect("删得了上一次的临时目录");
            }
            fs::create_dir_all(&path).expect("建得了临时目录");
            TestDirectory { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _cleanup_outcome = fs::remove_dir_all(&self.path);
        }
    }

    const ALL_ARMS: [Candidate; 4] = [Candidate::RedbQuickRepair, Candidate::RedbDefault, Candidate::RocksDatabase, Candidate::HardenedFile];

    #[test]
    fn every_anchor_matches_including_the_block_file_length_and_the_value_byte_totals() {
        let lines = network_anchor_lines();
        for (identifier, computed, registered, matches) in &lines {
            assert!(matches, "锚点 {identifier} 算出 {computed}，登记是 {registered}");
        }
        assert_eq!(lines.len(), 20, "送块那边 18 项，加 A11_file、A10");
    }

    #[test]
    fn interruption_plan_pins_first_thresholds_and_pauses_and_is_sorted() {
        let plan = interruption_plan(REGISTERED_INTERRUPTIONS, PATH_BLOCK_COUNT);
        assert_eq!(plan.len(), 50);
        let thresholds: Vec<u64> = plan.iter().map(|interruption| interruption.acknowledgement_threshold).collect();
        assert!(thresholds.windows(2).all(|pair| pair[0] <= pair[1]), "a_k 排好序");
        assert!(thresholds.iter().all(|&threshold| (1..50_000).contains(&threshold)), "a_k 在 [1, 5 × 10⁴)");
        assert_eq!(thresholds[..5], PINNED_FIRST_THRESHOLDS);
        assert_eq!(thresholds[49], PINNED_LAST_THRESHOLD);
        assert_eq!(plan[0].pause_fraction, PINNED_FIRST_PAUSE_FRACTION);
        let escalated = interruption_plan(ESCALATED_INTERRUPTIONS, PATH_BLOCK_COUNT);
        assert_eq!(escalated.len(), 200);
        let small = interruption_plan(5, 400);
        assert!(small.iter().all(|interruption| (1..400).contains(&interruption.acknowledgement_threshold)));
    }

    #[test]
    fn drop_control_picks_one_offset_below_a_thousand_per_window_and_pins_the_first_three() {
        let picks = drop_control_picks(52);
        assert_eq!(picks.len(), 52);
        assert!(picks.iter().all(|&pick| pick < 1_000));
        assert_eq!(picks[..3], PINNED_FIRST_DROP_PICKS);
    }

    #[test]
    fn sender_arguments_round_trip_through_the_sender_parser() {
        let settings = SenderSettings {
            endpoint: SenderEndpoint::ConnectTcp("127.0.0.1:4242".to_string()),
            block_count: 50_000,
            states_per_block: 65_536,
            window_frames: 256,
            pace_between_first_transmissions: Some(Duration::from_millis(100)),
            acknowledgement_stall_limit: Duration::from_secs(120),
            spinning_thread_count: 3,
        };
        let arguments = sender_arguments(&settings);
        assert_eq!(arguments[0], "send");
        let parsed = SenderSettings::parse(&arguments[1..]).expect("自己写的参数认得出");
        assert_eq!(parsed.endpoint, settings.endpoint);
        assert_eq!(parsed.block_count, 50_000);
        assert_eq!(parsed.states_per_block, 65_536);
        assert_eq!(parsed.window_frames, 256);
        assert_eq!(parsed.pace_between_first_transmissions, Some(Duration::from_millis(100)));
        assert_eq!(parsed.acknowledgement_stall_limit, Duration::from_secs(120));
        assert_eq!(parsed.spinning_thread_count, 3);
    }

    #[test]
    fn acknowledged_ranges_expand_back_to_the_same_flags() {
        let flags = vec![true, true, false, true, false, false, true];
        assert_eq!(expand_true_positions(&compress_true_positions(&flags), 7), Some(flags));
        assert_eq!(expand_true_positions("none", 2), Some(vec![false, false]));
        assert_eq!(expand_true_positions("0-7", 7), None, "越界的区间认不出");
        assert_eq!(expand_true_positions("3-1", 7), None, "倒过来的区间认不出");
        let everything = expand_true_positions("0-49999", 50_000).expect("认得出");
        assert_eq!(everything.iter().filter(|&&flag| flag).count(), 50_000);
    }

    #[test]
    fn classification_table_matches_the_registered_table() {
        use BlockClass::{Confirmed, InFlight, NotStarted};
        use BlockVerdict::{ConfirmedReadError, Correct, LostConfirmed, Phantom, SilentlyCorrupted, UnconfirmedReadError};
        use ReadKind::{DifferentBytes, MatchingBytes, NotFound, ReadError};
        let expected = [
            (Confirmed, [Correct, SilentlyCorrupted, LostConfirmed, ConfirmedReadError]),
            (InFlight, [Correct, SilentlyCorrupted, Correct, UnconfirmedReadError]),
            (NotStarted, [Phantom, Phantom, Correct, UnconfirmedReadError]),
        ];
        for (class, verdicts) in expected {
            for (read, verdict) in [MatchingBytes, DifferentBytes, NotFound, ReadError].into_iter().zip(verdicts) {
                assert_eq!(classify_block(class, read), verdict, "{class:?} × {read:?}");
            }
        }
    }

    #[test]
    fn window_rates_turn_end_offsets_into_states_per_second() {
        let rates = window_states_per_second(&[2_000_000, 3_000_000, 7_000_000], 65_536);
        assert_eq!(rates, vec![32_768_000.0, 65_536_000.0, 16_384_000.0]);
        assert_eq!(parse_window_offsets("W end_offsets_microseconds=10,20"), vec![10, 20]);
        assert!(parse_window_offsets("W end_offsets_microseconds=none").is_empty());
    }

    #[test]
    fn throughput_judgement_separates_two_measurements_only_at_a_threshold_between_them() {
        assert!(network_throughput_flips(1_000.0, 1_500.0));
        assert!(!network_throughput_flips(2_000.0, 1_500.0));
        assert!(!network_throughput_flips(1_500.0, 1_500.0), "等于门槛不翻（登记：< R_B 才翻）");
        let mut output = ProductLines::new();
        assert_eq!(run_discrimination(&mut output, &["1000".to_string(), "3000".to_string()]), 0);
        assert_eq!(run_discrimination(&mut output, &["1000".to_string(), "1000".to_string()]), 1, "两个一样的数分不开");
    }

    #[test]
    fn rate_control_needs_the_slow_pass_at_most_two_hundred_and_the_fast_pass_above() {
        assert!(rate_control_seen(201.0, 200.0));
        assert!(!rate_control_seen(200.0, 150.0), "快的那遍不到 200 以上");
        assert!(!rate_control_seen(900.0, 200.5), "慢的那遍超过 200");
    }

    #[test]
    fn disconnect_paths_need_a_half_frame_a_resend_and_a_second_library_connection() {
        let run_with = |half_frames: u64, resent: u64, connections: u64| CellRun {
            sender_summary: format!("S ending=complete resent_frames={resent}"),
            library_network: format!("N receive_mode=registered connections={connections} half_frames_dropped={half_frames}"),
            ..CellRun::default()
        };
        assert!(disconnect_paths_reached(&run_with(3, 64, 51)));
        assert!(!disconnect_paths_reached(&run_with(0, 64, 51)), "没丢过半帧");
        assert!(!disconnect_paths_reached(&run_with(3, 0, 51)), "没重发过");
        assert!(!disconnect_paths_reached(&run_with(3, 64, 1)), "重发都没进库：库进程只收过一条连接");
        assert!(disconnect_paths_reached(&run_with(1, 1, 2)), "三样各自刚过下沿");
    }

    fn frame_for(network_position: u64, states_per_block: u64) -> (BlockKey, Vec<u8>, Vec<u8>) {
        let block_index = network_path_block_index(network_position);
        let key = interleaved_block_key(block_index, states_per_block, &input_fingerprint());
        let value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, states_per_block);
        let frame = sender_role::encode_block_frame(&key, &value);
        (key, value, frame)
    }

    fn receiver_on_file_library(directory: &Path, receive_mode: ReceiveMode, network_blocks: u64) -> FrameReceiver {
        let mut stores = open_thread_stores(Candidate::HardenedFile, &directory.join("library"), 1).expect("开得了 F 库");
        FrameReceiver::new(receive_mode, SMALL_STATES_PER_BLOCK, network_blocks, stores.pop().expect("一个句柄"))
    }

    fn acknowledged_keys(acknowledgements: &[u8]) -> Vec<BlockKey> {
        acknowledgements
            .chunks(sender_role::ACKNOWLEDGEMENT_FRAME_BYTES)
            .map(|chunk| sender_role::decode_acknowledgement_frame(chunk.try_into().expect("40 字节")).expect("确认帧对"))
            .collect()
    }

    #[test]
    fn registered_receiver_drops_a_bad_checksum_frame_and_a_half_frame_and_acknowledges_the_rest() {
        let directory = TestDirectory::new("registered-receiver");
        let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::Registered, 10);
        let (first_key, first_value, first_frame) = frame_for(0, SMALL_STATES_PER_BLOCK);
        let (second_key, _second_value, mut corrupted_frame) = frame_for(1, SMALL_STATES_PER_BLOCK);
        corrupted_frame[BLOCK_FRAME_HEADER_BYTES + 100] ^= 0x5A;
        let (third_key, third_value, third_frame) = frame_for(2, SMALL_STATES_PER_BLOCK);
        let (half_key, _half_value, half_frame) = frame_for(3, SMALL_STATES_PER_BLOCK);
        let mut stream = Vec::new();
        stream.extend_from_slice(&first_frame);
        stream.extend_from_slice(&corrupted_frame);
        stream.extend_from_slice(&third_frame);
        stream.extend_from_slice(&half_frame[..half_frame.len() / 2]);
        let mut acknowledgements = Vec::new();
        let end = receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
        assert_eq!(end, ConnectionEnd::HalfFrame);
        assert_eq!(acknowledged_keys(&acknowledgements), vec![first_key, third_key]);
        assert_eq!(receiver.counters.complete_frames, 3);
        assert_eq!(receiver.counters.checksum_failures, 1);
        assert_eq!(receiver.counters.half_frames_dropped, 1);
        assert_eq!(receiver.counters.half_frames_written, 0);
        assert_eq!(receiver.counters.commits, 2);
        assert_eq!(receiver.store.read_block(&first_key), ReadOutcome::Found(first_value));
        assert_eq!(receiver.store.read_block(&second_key), ReadOutcome::NotFound, "CRC 不对的帧不写");
        assert_eq!(receiver.store.read_block(&third_key), ReadOutcome::Found(third_value));
        assert_eq!(receiver.store.read_block(&half_key), ReadOutcome::NotFound, "半帧不写");
    }

    #[test]
    fn registered_receiver_rewrites_a_resent_frame_with_the_same_value_and_counts_it_as_already_stored() {
        let directory = TestDirectory::new("resent-frame");
        let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::Registered, 10);
        let (key, value, frame) = frame_for(5, SMALL_STATES_PER_BLOCK);
        let mut stream = frame.clone();
        stream.extend_from_slice(&frame);
        let mut acknowledgements = Vec::new();
        let end = receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
        assert_eq!(end, ConnectionEnd::ClosedAtFrameBoundary);
        assert_eq!(acknowledged_keys(&acknowledgements), vec![key, key]);
        assert_eq!(receiver.counters.commits, 2, "同键再来一次就同值再写一遍");
        assert_eq!(receiver.counters.frames_already_stored, 1);
        assert_eq!(receiver.store.read_block(&key), ReadOutcome::Found(value));
    }

    #[test]
    fn torn_accepting_receiver_writes_a_zero_padded_half_frame_and_then_skips_the_resent_frame() {
        let directory = TestDirectory::new("torn-receiver");
        let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::TornAccepting, 10);
        let (key, value, frame) = frame_for(7, SMALL_STATES_PER_BLOCK);
        let cut = BLOCK_FRAME_HEADER_BYTES + 300;
        let mut acknowledgements = Vec::new();
        let end = receiver.serve_connection(&mut &frame[..cut], &mut acknowledgements, None);
        assert_eq!(end, ConnectionEnd::HalfFrame);
        assert_eq!(receiver.counters.half_frames_written, 1);
        let end = receiver.serve_connection(&mut frame.as_slice(), &mut acknowledgements, None);
        assert_eq!(end, ConnectionEnd::ClosedAtFrameBoundary);
        assert_eq!(acknowledged_keys(&acknowledgements), vec![key], "重发的帧键已在：跳过不写、照回确认");
        assert_eq!(receiver.counters.frames_skipped_as_already_stored, 1);
        let mut padded = value[..300].to_vec();
        padded.resize(value.len(), 0);
        assert_eq!(receiver.store.read_block(&key), ReadOutcome::Found(padded), "库里留着补 0 的半帧");
    }

    #[test]
    fn drop_control_receiver_acknowledges_without_writing_exactly_the_picked_frames() {
        let directory = TestDirectory::new("drop-receiver");
        let network_blocks = 2_000u64;
        let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::DropOnePerThousand, network_blocks);
        let mut stream = Vec::new();
        for network_position in 0..network_blocks {
            stream.extend_from_slice(&frame_for(network_position, SMALL_STATES_PER_BLOCK).2);
        }
        let mut acknowledgements = Vec::new();
        receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
        assert_eq!(acknowledged_keys(&acknowledgements).len(), 2_000);
        assert_eq!(receiver.counters.controlled_drops, 2);
        assert_eq!(receiver.counters.commits, 1_998);
        let picks = drop_control_picks(2);
        for (window, pick) in picks.iter().enumerate() {
            let dropped_position = u64::try_from(window).expect("窗号") * 1_000 + pick;
            let (key, _value, _frame) = frame_for(dropped_position, SMALL_STATES_PER_BLOCK);
            assert_eq!(receiver.store.read_block(&key), ReadOutcome::NotFound, "挑中的第 {dropped_position} 帧不写");
        }
    }

    #[test]
    fn acknowledge_without_writing_receiver_leaves_the_library_empty() {
        let directory = TestDirectory::new("raw-receiver");
        let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::AcknowledgeWithoutWriting, 10);
        let mut stream = Vec::new();
        for network_position in 0..5 {
            stream.extend_from_slice(&frame_for(network_position, SMALL_STATES_PER_BLOCK).2);
        }
        let mut acknowledgements = Vec::new();
        receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
        assert_eq!(acknowledged_keys(&acknowledgements).len(), 5);
        assert_eq!(receiver.counters.commits, 0);
        assert_eq!(receiver.store.enumerate_keys_with_value_lengths().expect("枚举得了").len(), 0);
    }

    #[test]
    fn sleep_before_commit_receiver_spends_at_least_five_milliseconds_per_frame() {
        let directory = TestDirectory::new("sleep-receiver");
        let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::SleepBeforeCommit, 20);
        let mut stream = Vec::new();
        for network_position in 0..20 {
            stream.extend_from_slice(&frame_for(network_position, SMALL_STATES_PER_BLOCK).2);
        }
        let mut acknowledgements = Vec::new();
        let started = Instant::now();
        receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, None);
        assert!(started.elapsed() >= Duration::from_millis(100), "20 帧每帧先睡 5 ms");
        assert_eq!(receiver.counters.commits, 20);
    }

    #[test]
    fn stop_after_acknowledgements_returns_interruption_due_right_after_that_acknowledgement() {
        let directory = TestDirectory::new("stop-after");
        let mut receiver = receiver_on_file_library(&directory.path, ReceiveMode::Registered, 10);
        let mut stream = Vec::new();
        for network_position in 0..6 {
            stream.extend_from_slice(&frame_for(network_position, SMALL_STATES_PER_BLOCK).2);
        }
        let mut acknowledgements = Vec::new();
        let end = receiver.serve_connection(&mut stream.as_slice(), &mut acknowledgements, Some(4));
        assert_eq!(end, ConnectionEnd::InterruptionDue);
        assert_eq!(acknowledged_keys(&acknowledgements).len(), 4);
    }

    #[test]
    fn verification_counts_a_flipped_byte_a_missing_confirmed_block_and_a_phantom_key() {
        let directory = TestDirectory::new("verification");
        let library = directory.path.join("library");
        let mut stores = open_thread_stores(Candidate::HardenedFile, &library, 1).expect("开得了 F 库");
        let mut store = stores.pop().expect("一个句柄");
        let fingerprint = input_fingerprint();
        let states = SMALL_STATES_PER_BLOCK;
        for direct_position in 0..4u64 {
            let block_index = 2 * direct_position;
            let mut value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, states);
            if direct_position == 2 {
                value[500] ^= 0x5A;
            }
            store.put_block(&interleaved_block_key(block_index, states, &fingerprint), &value).expect("写得进");
        }
        for network_position in [0u64, 2] {
            let block_index = network_path_block_index(network_position);
            let value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, states);
            store.put_block(&interleaved_block_key(block_index, states, &fingerprint), &value).expect("写得进");
        }
        let phantom = interleaved_block_key(9_999, states, &fingerprint);
        store.put_block(&phantom, &[1u8; 1_024]).expect("写得进");
        let plan = PathVerificationPlan {
            states_per_block: states,
            direct_blocks: 4,
            direct_confirmed: 4,
            direct_started: 4,
            network_blocks: 4,
            network_acknowledged: vec![true, true, true, false],
            network_sent: 4,
        };
        let report = verify_library(Candidate::HardenedFile, &library, &plan);
        assert!(!report.open_failed);
        assert_eq!(report.silently_corrupted, vec![4], "第 2 个直写块 j = 4 翻了一个字节");
        assert_eq!(report.lost_confirmed, vec![3], "网络第 1 块 j = 3 确认过而不在");
        assert_eq!(report.phantom_keys.len(), 1);
        assert_eq!(report.confirmed_checked, 7);
        assert_eq!(report.in_flight_checked, 1);
        assert_eq!(report.enumerated_key_count, 7);
        let decoded = decode_verification_line(&report.encode());
        assert_eq!(decoded.lost_confirmed, vec![3]);
        assert_eq!(decoded.silently_corrupted, vec![4]);
        assert_eq!(decoded.phantom_keys.len(), 1);
        assert_eq!(decoded.enumerated_key_count, 7);
    }

    fn small_loopback_cell(label: &'static str, direct_blocks: u64, network_blocks: u64, states_per_block: u64, receive_mode: ReceiveMode, interruption_count: usize) -> LoopbackCell {
        LoopbackCell { label, direct_blocks, network_blocks, states_per_block, window_frames: PRIMARY_WINDOW_FRAMES, receive_mode, interruption_count, is_timed: false }
    }

    fn assert_clean_run(run: &CellRun, cell: &LoopbackCell, arm: Candidate) {
        let context = format!("臂 {} 格 {}：{} | {} | {}", arm.label(), cell.label, run.sender_summary, run.library_network, run.setup_problem);
        assert_eq!(run.setup_problem, "", "{context}");
        assert_eq!(field_value(&run.sender_summary, "ending"), Some("complete"), "{context}");
        assert_eq!(number_field(&run.sender_summary, "acknowledged"), cell.network_blocks, "{context}");
        assert_eq!(number_field(&run.library_direct, "confirmed"), cell.direct_blocks, "{context}");
        assert!(!run.verification.verification_missing, "{context}");
        assert!(!run.verification.open_failed, "{context}");
        assert_eq!(run.verification.lost_confirmed.len(), 0, "{context}");
        assert_eq!(run.verification.silently_corrupted.len(), 0, "{context}");
        assert_eq!(run.read_errors(), 0, "{context}");
        assert_eq!(run.verification.phantom_keys.len(), 0, "{context}");
        assert_eq!(run.verification.enumerated_key_count, cell.direct_blocks + cell.network_blocks, "{context}");
        assert_eq!(run.verification.enumerated_value_bytes, (cell.direct_blocks + cell.network_blocks) * cell.states_per_block, "{context}");
    }

    #[test]
    fn every_arm_keeps_both_paths_through_a_small_loopback_run() {
        let directory = TestDirectory::new("every-arm");
        for arm in ALL_ARMS {
            let cell = small_loopback_cell("small", 96, 128, SMALL_STATES_PER_BLOCK, ReceiveMode::Registered, 0);
            let run = run_loopback_cell(arm, &cell, &directory.path);
            assert_clean_run(&run, &cell, arm);
            assert_eq!(run.sender_exit_code, 0);
            assert_eq!(run.library_exit_code, 0);
            assert_eq!(number_field(&run.library_network, "commits"), 128);
        }
    }

    #[test]
    fn loopback_run_with_five_forwarder_kills_resends_and_keeps_every_block() {
        let directory = TestDirectory::new("five-kills");
        let cell = small_loopback_cell("five-kills", 128, 600, PRIMARY_STATES_PER_BLOCK, ReceiveMode::Registered, 5);
        let run = run_loopback_cell(Candidate::RocksDatabase, &cell, &directory.path);
        assert_clean_run(&run, &cell, Candidate::RocksDatabase);
        assert_eq!(run.interruptions.len(), 5);
        assert!(run.interruptions.iter().all(|interruption| interruption.forwarder_restarted));
        // 每次杀转发进程至少逼出一次重连；被杀的进程退出时连接先断、监听口后关，送块进程可能在这几十微秒里
        // 又连上那个正在退出的监听口、重发一段之后再断一次（推的：单测读到过重连用时几十微秒、同一确认数紧接一次
        // 正常重连的 R 行），所以一次中断记一到两次重连。
        let reconnections = number_field(&run.sender_summary, "reconnections");
        assert!(
            (5..=10).contains(&reconnections),
            "{} | {:?} | {:?} | {}",
            run.sender_summary,
            run.sender_reconnections,
            run.interruptions,
            run.library_network
        );
        assert!(number_field(&run.sender_summary, "resent_frames") >= 5, "{}", run.sender_summary);
        assert!(number_field(&run.library_network, "connections") >= 6, "{}", run.library_network);
    }

    #[test]
    fn drop_control_run_loses_exactly_the_frames_the_library_recorded() {
        let directory = TestDirectory::new("drop-run");
        let cell = small_loopback_cell("drop", 0, 2_000, SMALL_STATES_PER_BLOCK, ReceiveMode::DropOnePerThousand, 0);
        let run = run_loopback_cell(Candidate::HardenedFile, &cell, &directory.path);
        assert_eq!(number_field(&run.library_network, "controlled_drops"), 2);
        assert_eq!(run.verification.lost_confirmed.len(), 2, "PC3-drop：丢块数恰好 = 库进程记的数");
        assert_eq!(run.verification.silently_corrupted.len(), 0);
        assert_eq!(run.verification.enumerated_key_count, 1_998);
    }

    #[test]
    fn peer_channel_run_with_five_relay_kills_keeps_every_block() {
        let directory = TestDirectory::new("peer-channel");
        let socket_path = directory.path.join("s.sock");
        let network_blocks = 300u64;
        let sender_settings = SenderSettings {
            endpoint: SenderEndpoint::ListenUnix(socket_path.clone()),
            block_count: network_blocks,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            window_frames: PRIMARY_WINDOW_FRAMES,
            pace_between_first_transmissions: None,
            acknowledgement_stall_limit: Duration::from_secs(20),
            spinning_thread_count: 0,
        };
        let sender = std::thread::spawn(move || {
            let mut output = Vec::new();
            sender_role::run_sender(&sender_settings, &mut output)
        });
        let test_binary = std::env::current_exe().expect("找得到测试二进制");
        let relay_command = format!(
            "{RELAY_TARGET_ENVIRONMENT}='unix:{}' exec '{}' --exact tests::child_process_role_entry --nocapture --test-threads=1 --quiet 3>&1 1>/dev/null",
            socket_path.display(),
            test_binary.display()
        );
        let library = directory.path.join("library");
        let library_settings = LibrarySettings {
            candidate: Candidate::HardenedFile,
            library: library.clone(),
            receive_mode: ReceiveMode::Registered,
            direct_blocks: 64,
            network_blocks,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            channel: ReceiveChannel::PeerCommand {
                command_words: vec!["sh".to_string(), "-c".to_string()],
                relay_command,
                interruptions: interruption_plan(5, network_blocks),
            },
        };
        let mut library_process = spawn_running_child(child_command(&library_settings.to_arguments()), true);
        let report = sender.join().expect("送块线程没 panic");
        drop(library_process.input.take());
        let lines: Vec<String> = library_process.lines.iter().collect();
        library_process.child.wait().expect("收得了库进程");
        let network_line = lines.iter().find(|line| line.starts_with("N ")).cloned().unwrap_or_default();
        let direct_line = lines.iter().find(|line| line.starts_with("D ")).cloned().unwrap_or_default();
        let interruption_lines = lines.iter().filter(|line| line.starts_with("I ")).count();
        assert_eq!(report.ending, sender_role::SenderEnding::Complete, "{network_line}");
        assert_eq!(report.acknowledged_count, network_blocks);
        assert_eq!(report.reconnections.len(), 5, "{network_line}");
        assert_eq!(interruption_lines, 5, "库进程回出第 a_k 个确认之后杀数据通道，五次");
        let plan = PathVerificationPlan {
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            direct_blocks: 64,
            direct_confirmed: number_field(&direct_line, "confirmed"),
            direct_started: number_field(&direct_line, "started"),
            network_blocks,
            network_acknowledged: expand_true_positions(&report.acknowledged_ranges, network_blocks).expect("区间认得出"),
            network_sent: report.sent_positions,
        };
        let verification = verify_library(Candidate::HardenedFile, &library, &plan);
        assert_eq!(verification.lost_confirmed.len(), 0);
        assert_eq!(verification.silently_corrupted.len(), 0);
        assert_eq!(verification.phantom_keys.len(), 0);
        assert_eq!(verification.enumerated_key_count, 364);
    }

    // 下面几个数由另写的 python（另起一份 SplitMix64，脚本与原样输出抄在登记第十二节「修订」第三段那几行里）独立算出，
    // 不取自这个文件的输出。
    const PINNED_FIRST_THRESHOLDS: [u64; 5] = [18, 114, 463, 950, 1132];
    const PINNED_LAST_THRESHOLD: u64 = 49_349;
    const PINNED_FIRST_PAUSE_FRACTION: f64 = 0.5665882453311935;
    const PINNED_FIRST_DROP_PICKS: [u64; 3] = [541, 117, 338];
}
