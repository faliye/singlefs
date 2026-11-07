//! RocksDB 那一层。服务只有自己那一份库（`--library`，缺省状态目录下的 `library`，头一次起时建），布局只有一种，
//! 与崩溃判定库（`crates/singlefs-verification/src/verdict_store.rs` 建的那种）相同：只有默认列族，全部行都在它里面、靠键的第一个字节分开。
//! - 三张表（录入、事实、结论）只是协议里的分表码，存法上都落在默认列族、键原样（不加前缀、不加表标签）：用哪张表的名义读写同一把键，
//!   读写的都是同一行；按前缀扫哪一张表扫到的都是这个前缀下的全部键。崩溃行的键以表标签字节（0x00–0x1B）起头，Python 三家的键以 ASCII 族名起头，
//!   互不相撞。
//! - 两张记账表也在默认列族，各一个键前缀（`APPLIED_REQUEST_KEY_TAG`、`CLAIM_KEY_TAG`），落在客户端不许用的那一段键里
//!   （第一个字节 0xF0–0xFF，`protocol::is_service_key`）：解帧时拒客户端读、写、认领这一段的键与以它起头的前缀，按前缀扫走到这一段就停，
//!   所以客户端既读不到也写不坏记账行。
//!   已生效的请求号：键 `F0` ‖ `u8 身份长` ‖ 身份 ‖ `u64 请求号`（大端），值 `u32 写了几条`（小端）。
//!   认领记录：键 `F1` ‖ 键（三张表同一个键空间，认领不分表），值 `u64 租期到点`（自 1970 年起的毫秒，小端）‖ 持有者身份。
//! - 开库选项照 `verdict_store.rs` 的 `library_options`（`library_options` 这里同名，逐项对着抄）：LZ4、比较器与表格式取 RocksDB 缺省、
//!   不建默认之外的列族；库目录不在就建（服务自己的库）；目录在而不是 RocksDB（非空、没有 CURRENT）、有默认之外的列族、
//!   RocksDB 因选项对不上拒开，一律退 6，不改库、不 repair；被别的进程锁着、读写出错退 3。
//! - 文件数上限：开库选项不设 `max_open_files`（`verdict_store.rs` 也不设，RocksDB 缺省不限），开库时 RocksDB 把库里的 SST 全打开、
//!   一直开着，库里有几个文件就占几个描述符；systemd 用户单元与登录会话给的软上限常是 1024，硬上限高得多，库一大就撞软上限、
//!   RocksDB 报「Too many open files」拒开。所以每次开库之前（`open_single_column_family_library`：服务、自证与压测起的临时服务都走它）
//!   先把本进程 RLIMIT_NOFILE 的软上限抬到硬上限、回读，再核够不够：要的数是库目录里的文件数、本进程此刻开着的描述符数
//!   与余量 `DESCRIPTOR_HEADROOM` 之和。抬完仍小于它就不开库、退 8
//!   （`server::DESCRIPTOR_LIMIT_TOO_LOW_EXIT_CODE`），标准错误写「文件数上限不够：要 N、硬上限 M」与出路（`DESCRIPTOR_LIMIT_NEXT_STEP`）：
//!   硬上限是起这个进程的那一层给的，进程自己抬不了；单元模板里不另写软上限，软上限由这里抬。
//!   弄坏开关 descriptor-soft-limit-not-raised 退回改前（不抬、不核），红在自证 library-opens-under-low-descriptor-soft-limit。
//! - 写走组提交：提交线程把几帧的写（数据与记账一起）合进一个 WriteBatch、不带同步写进去，再 `flush_wal(true)` 同步一次 WAL；
//!   库开着 `manual_wal_flush`，WAL 的内容在 `flush_wal` 之前只在进程内存里，所以服务被 `kill -9` 时没同步的写整批不在，
//!   「回确认之前已落盘」由自证 replay-after-server-kill 那一格拿 `kill -9` 判得出来。持久性与 `verdict_store.rs` 的 `synced_write_options` 相同：
//!   回确认时 WAL 已同步。没同步的写在 `flush_wal` 之前对别的连接的读已经可见；服务这时崩了，那一批没回确认，客户端会重放，读到的值随后又回来。
//! - 同步 WAL 失败之后服务退出、重起时重开库（只这一条路，不留在进程里接着服务、不在进程里重试同步）：写批已经进了内存，
//!   对读与去重可见，盘上没有。提交线程给这一组每帧回存储出错（写进套接字再走）、每条连接记一行故障，然后进程退出，
//!   退出码 7（`server::STORAGE_SYNC_FAILED_EXIT_CODE`）。留在进程里的话，去重查到只在内存里生效的请求号，给重发的同一请求回成功，
//!   确认就早于落盘（弄坏开关 sync-failure-kept-serving 退回这一种）；同步失败之后再同步成功也不证明先前那一批落了盘
//!   （内核可能已丢掉写失败的脏页），所以不重试。重起时 RocksDB 从 WAL 文件重开库，内存里只剩盘上真有的；服务停着时客户端连不上、
//!   退 75，没确认的写帧留在客户端的追加日志里，重起之后重放：盘上有的去重回早已生效，没有的重新写、同步之后才回确认。
//!   写批本身没写进去（写失败，`GroupCommitFailure::WriteFailed`）的库没变，只回这一组存储出错、照常服务。
//!   自证用 `--selftest-sync-failure-file <文件>`（只认 `--selftest-instance`）：那个文件在时组提交写完写批不同步、当成同步失败，
//!   写批留在进程内存里、不进 WAL 文件。真实的同步失败（写批进了 WAL 文件与页缓存、libc 的 fdatasync 回 EIO，`flush_wal` 回错）
//!   由同一格的另一条输入造（LD_PRELOAD 包装，`selftest_hardening_cells.rs` 的 `real_fdatasync_failure_input`，设了
//!   `PRUNE_SERVER_SELFTEST_WRAPPER_COMPILER` 才跑）。
//! - 只有一个进程打开它：RocksDB 自己的 LOCK 文件保证同一个库目录只有一个持有者，第二个打开的拿不到锁、打不开。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use rocksdb::statistics::{StatsLevel, Ticker};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBCompressionType, DBPinnableSlice, Direction, ErrorKind,
    IteratorMode, Options, WriteBatch, WriteOptions, DB, DEFAULT_COLUMN_FAMILY_NAME,
};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::command_line::DEFAULT_MAXIMUM_CONNECTIONS;
use crate::protocol::{is_service_key, EntryInTable, KeyInTable};

/// 已生效的请求号那张记账表的键前缀（文件头）。
pub const APPLIED_REQUEST_KEY_TAG: u8 = 0xF0;
/// 认领记录那张记账表的键前缀（文件头）。
pub const CLAIM_KEY_TAG: u8 = 0xF1;

/// 开库之后服务另要开的描述符里，压实与落盘新写的 SST、换上之前新旧同时开着的 WAL 与 MANIFEST、RocksDB 的运行日志、
/// 故障日志与两处监听合起来留的数（文件头「文件数上限」）：压实一次的输出文件至多几百个，留到两千有余。
const DESCRIPTOR_HEADROOM_FOR_LIBRARY_GROWTH: u64 = 2048;
/// 开库之前核文件数上限时，库目录里的文件数与本进程此刻开着的描述符数之外加的余量（文件头「文件数上限」）：
/// 每条连接占两个描述符（套接字与读写两个线程分开用的那一份复制，`server.rs` 的 `try_clone`），按缺省连接数上限算；
/// 另加 `DESCRIPTOR_HEADROOM_FOR_LIBRARY_GROWTH`。
pub(crate) const DESCRIPTOR_HEADROOM: u64 =
    2 * DEFAULT_MAXIMUM_CONNECTIONS as u64 + DESCRIPTOR_HEADROOM_FOR_LIBRARY_GROWTH;
/// 文件数上限不够时的出路。
pub const DESCRIPTOR_LIMIT_NEXT_STEP: &str = "RLIMIT_NOFILE 的硬上限是起这个进程的那一层给的，进程自己只能把软上限抬到硬上限（开库之前已经抬了）：\
请有权限的人把那一层的硬上限抬到至少上面「要」的那个数（不超过 /proc/sys/fs/nr_open）再重起——systemd 起的看用户管理器的 DefaultLimitNOFILE，\
登录 shell 起的看 /etc/security/limits.conf 的 hard nofile；不要在单元模板里另写软上限绕过去";

/// 开库失败的三种：服务按它们退不同的码（`server::ServeFailure`）。
#[derive(Debug)]
pub enum LibraryOpenFailure {
    /// 库打不开：被别的进程持有（RocksDB 的 LOCK）、目录不在或建不了、读写出错。
    Unavailable(String),
    /// 库对不上：不是 RocksDB、有默认之外的列族、RocksDB 因选项对不上拒开。
    Mismatched(String),
    /// 文件数上限不够：软上限抬到硬上限之后仍小于开这份库要的描述符数，或抬不上去（文件头「文件数上限」）。
    DescriptorLimitTooLow(String),
}

/// 本进程 RLIMIT_NOFILE 此刻的软上限与硬上限。
pub(crate) fn descriptor_limit() -> Result<libc::rlimit, String> {
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: getrlimit 只往这个本地结构体里写，结构体活到调用返回。
    if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) } != 0 {
        return Err(format!(
            "读不了本进程的 RLIMIT_NOFILE：{}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(limit)
}

/// 目录里有几项（读出错的那一项也回错，不当成没有）。
pub(crate) fn entries_in(directory: &Path) -> Result<u64, String> {
    let mut count = 0_u64;
    for entry in std::fs::read_dir(directory)
        .map_err(|error| format!("读不了目录 {}：{error}", directory.display()))?
    {
        entry.map_err(|error| format!("读不了目录 {} 里的一项：{error}", directory.display()))?;
        count += 1;
    }
    Ok(count)
}

/// 开 `directory` 那份库之前：软上限抬到硬上限、回读，再核够不够（文件头「文件数上限」）。抬不上去、抬完仍不够回
/// `DescriptorLimitTooLow`，说明以「文件数上限不够：」起头、写要几个与硬上限；读不了上限、库目录或 `/proc/self/fd` 回 `Unavailable`。
fn raise_descriptor_limit_for(
    directory: &Path,
    break_switches: &BreakSwitches,
) -> Result<(), LibraryOpenFailure> {
    if break_switches.is_enabled(BreakSwitch::DescriptorSoftLimitNotRaised) {
        return Ok(());
    }
    let before = descriptor_limit().map_err(LibraryOpenFailure::Unavailable)?;
    if before.rlim_cur < before.rlim_max {
        let raised = libc::rlimit {
            rlim_cur: before.rlim_max,
            rlim_max: before.rlim_max,
        };
        // SAFETY: setrlimit 只读这个本地结构体，结构体活到调用返回；改的是本进程自己的上限，软上限不超过硬上限，不要特权。
        if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &raised) } != 0 {
            return Err(LibraryOpenFailure::DescriptorLimitTooLow(format!(
                "文件数上限不够：软上限 {} 抬不到硬上限 {}：{}",
                before.rlim_cur,
                before.rlim_max,
                std::io::Error::last_os_error()
            )));
        }
    }
    let after = descriptor_limit().map_err(LibraryOpenFailure::Unavailable)?;
    let library_files = if directory.is_dir() {
        entries_in(directory).map_err(LibraryOpenFailure::Unavailable)?
    } else {
        0
    };
    let open_now =
        entries_in(Path::new("/proc/self/fd")).map_err(LibraryOpenFailure::Unavailable)?;
    let needed = library_files + open_now + DESCRIPTOR_HEADROOM;
    if after.rlim_cur < needed {
        return Err(LibraryOpenFailure::DescriptorLimitTooLow(format!(
            "文件数上限不够：要 {needed}、硬上限 {}（库 {} 里 {library_files} 个文件，本进程此刻开着 {open_now} 个描述符，余量 {DESCRIPTOR_HEADROOM}；\
             软上限抬到了 {}）",
            after.rlim_max,
            directory.display(),
            after.rlim_cur
        )));
    }
    Ok(())
}

/// 开库选项，照 `crates/singlefs-verification/src/verdict_store.rs` 的 `library_options`：落在库文件里、开错了会拒开或改写的几项逐项相同——
/// LZ4 压缩、比较器与表格式取 RocksDB 缺省（它没设，这里也不设）；`creates_library` 是库不在时建不建（服务自己的库建）。
/// 另加的几项只管这个进程怎么跑，不进 SST 与 MANIFEST：不建列族、后台线程数、统计、WAL 手动同步（回确认之前照样 `flush_wal(true)`）。
pub(crate) fn library_options(background_threads: usize, creates_library: bool) -> Options {
    let mut options = Options::default();
    options.create_if_missing(creates_library);
    options.create_missing_column_families(false);
    options.set_compression_type(DBCompressionType::Lz4);
    options.increase_parallelism(i32::try_from(background_threads.max(1)).unwrap_or(i32::MAX));
    options.enable_statistics();
    options.set_statistics_level(StatsLevel::ExceptHistogramOrTimers);
    options.set_manual_wal_flush(true);
    options
}

/// RocksDB 的错按种类分：锁、读写出错算打不开，别的（选项对不上、不是这种库）算对不上；说明里写是哪一种。
pub(crate) fn classify_rocksdb_failure(
    error: &rocksdb::Error,
    what: &str,
    break_switches: &BreakSwitches,
) -> LibraryOpenFailure {
    let unavailable = || {
        LibraryOpenFailure::Unavailable(format!(
            "{what}：库被别的进程持有（RocksDB 的 LOCK）或读写出错：{error}"
        ))
    };
    if break_switches.is_enabled(BreakSwitch::LibraryOpenFailureReportedAsLocked) {
        return unavailable();
    }
    let mismatched = || {
        LibraryOpenFailure::Mismatched(format!(
            "{what}：选项与库对不上，或不是崩溃判定库那种 RocksDB：{error}"
        ))
    };
    match error.kind() {
        ErrorKind::IOError | ErrorKind::Busy | ErrorKind::TimedOut | ErrorKind::TryAgain => {
            unavailable()
        }
        ErrorKind::NotFound
        | ErrorKind::Corruption
        | ErrorKind::NotSupported
        | ErrorKind::InvalidArgument
        | ErrorKind::MergeInProgress
        | ErrorKind::Incomplete
        | ErrorKind::ShutdownInProgress
        | ErrorKind::Aborted
        | ErrorKind::Expired
        | ErrorKind::CompactionTooLarge
        | ErrorKind::ColumnFamilyDropped
        | ErrorKind::Unknown => mismatched(),
    }
}

/// 开一份只有默认列族的库（服务自己的那一份，或自证读的样本库）：目录在的，先看是 RocksDB（非空目录要有 CURRENT）、列族只有默认一个，
/// 再抬文件数上限、核够不够（文件头「文件数上限」），再用 `options` 开；不 repair、不建列族。
/// 目录不在时 `options` 不建库就退 3，建就交给 RocksDB 建（服务自己的库）。
pub(crate) fn open_single_column_family_library(
    directory: &Path,
    options: &Options,
    break_switches: &BreakSwitches,
) -> Result<DB, LibraryOpenFailure> {
    let shown = directory.display();
    let mut column_families = vec![DEFAULT_COLUMN_FAMILY_NAME.to_string()];
    if directory.is_dir() {
        let has_current = directory.join("CURRENT").is_file();
        let is_empty = std::fs::read_dir(directory)
            .map_err(|error| {
                LibraryOpenFailure::Unavailable(format!("读不了库目录 {shown}：{error}"))
            })?
            .next()
            .is_none();
        if !has_current
            && !is_empty
            && !break_switches.is_enabled(BreakSwitch::LibraryWithoutCurrentFileOpened)
        {
            return Err(LibraryOpenFailure::Mismatched(format!(
                "库目录 {shown} 里有文件却没有 RocksDB 的 CURRENT 文件，不是一份 RocksDB 库"
            )));
        }
        if has_current {
            let listed = DB::list_cf(options, directory).map_err(|error| {
                classify_rocksdb_failure(
                    &error,
                    &format!("读不出库 {shown} 的列族清单"),
                    break_switches,
                )
            })?;
            let others: Vec<String> = listed
                .into_iter()
                .filter(|name| name.as_str() != DEFAULT_COLUMN_FAMILY_NAME)
                .collect();
            if !others.is_empty() {
                if !break_switches.is_enabled(BreakSwitch::LibraryExtraColumnFamiliesAccepted) {
                    return Err(LibraryOpenFailure::Mismatched(format!(
                        "库 {shown} 除默认列族外还有列族：{}；库只有一种布局，全部行在默认列族里",
                        others.join("、")
                    )));
                }
                column_families.extend(others);
            }
        }
    }
    raise_descriptor_limit_for(directory, break_switches)?;
    // rust-rocksdb 的 `open_cf_descriptors` 不写 default 时补一个 `Options::default()` 的，那样默认列族就不是 LZ4；所以显式给。
    let descriptors: Vec<ColumnFamilyDescriptor> = column_families
        .iter()
        .map(|name| ColumnFamilyDescriptor::new(name, options.clone()))
        .collect();
    DB::open_cf_descriptors(options, directory, descriptors).map_err(|error| {
        classify_rocksdb_failure(&error, &format!("RocksDB 拒开库 {shown}"), break_switches)
    })
}

/// 一条认领：谁持有、租期到哪一刻（自 1970 年起的毫秒）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClaimRecord {
    pub holder_identity: Vec<u8>,
    pub lease_expiry_milliseconds: u64,
}

/// 认领记录的键：`CLAIM_KEY_TAG` ‖ 键（三张表同一个键空间，认领不分表）。
pub fn claim_key(key: &[u8]) -> Vec<u8> {
    let mut claim = Vec::with_capacity(1 + key.len());
    claim.push(CLAIM_KEY_TAG);
    claim.extend_from_slice(key);
    claim
}

fn applied_request_key(
    client_identity: &[u8],
    request_number: u64,
    break_switches: &BreakSwitches,
) -> Vec<u8> {
    let mut key = Vec::with_capacity(10 + client_identity.len());
    if !break_switches.is_enabled(BreakSwitch::BookkeepingKeyTagDropped) {
        key.push(APPLIED_REQUEST_KEY_TAG);
    }
    key.push(u8::try_from(client_identity.len()).expect("身份长在解帧时核过不超 255"));
    key.extend_from_slice(client_identity);
    key.extend_from_slice(&request_number.to_be_bytes());
    key
}

/// 认领记录的值（文件头「认领记录」）；自证造带认领行的样本库也用它。
pub(crate) fn encode_claim_record(record: &ClaimRecord) -> Vec<u8> {
    let mut value = Vec::with_capacity(8 + record.holder_identity.len());
    value.extend_from_slice(&record.lease_expiry_milliseconds.to_le_bytes());
    value.extend_from_slice(&record.holder_identity);
    value
}

fn decode_claim_record(value: &[u8]) -> Option<ClaimRecord> {
    let expiry = value.get(..8)?;
    let holder_identity = value.get(8..)?.to_vec();
    if holder_identity.is_empty() {
        return None;
    }
    Some(ClaimRecord {
        holder_identity,
        lease_expiry_milliseconds: u64::from_le_bytes(expiry.try_into().ok()?),
    })
}

/// 一次组提交要写的东西（数据与记账在同一个写批里）；提交线程往里加，`commit_group` 写一次、同步一次。
pub struct GroupBatch {
    batch: WriteBatch,
}

impl GroupBatch {
    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }
}

/// 统计回应里回的 RocksDB 计数：自证拿前三个核「一帧一次 multi_get」「一帧只同步一次」，压测器只报不判。
pub const REPORTED_TICKERS: [Ticker; 7] = [
    Ticker::WalFileSynced,
    Ticker::NumberMultigetCalls,
    Ticker::NumberMultigetKeysRead,
    Ticker::NumberKeysRead,
    Ticker::NumberKeysWritten,
    Ticker::NumberDbSeek,
    Ticker::WalFileBytes,
];

/// 组提交失败的两种（文件头「同步 WAL 失败之后」那一条）。
#[derive(Debug)]
pub enum GroupCommitFailure {
    /// 写批没写进去：库没变。
    WriteFailed(String),
    /// 写批写进去了、同步 WAL 失败：这一批在内存里生效、对读与去重可见，盘上没有。
    SyncFailed(String),
}

impl GroupCommitFailure {
    pub fn message(&self) -> &str {
        match self {
            GroupCommitFailure::WriteFailed(message) | GroupCommitFailure::SyncFailed(message) => {
                message
            }
        }
    }
}

pub struct VerdictLibrary {
    database: DB,
    /// RocksDB 的统计对象挂在打开时用的那份选项上；选项不是 Sync，读计数时加锁。
    options_holding_statistics: Mutex<Options>,
    break_switches: BreakSwitches,
    /// 只给自证：这个文件在时组提交不同步、当成同步失败（文件头）。
    selftest_sync_failure_file: Option<PathBuf>,
}

/// 按前缀扫一帧的结局（`VerdictLibrary::scan_prefix_into`）：交给调用方的条数、到了条数上限或字节上限还没扫完这个前缀（`is_truncated`）、
/// 调用方拒收了一条（`is_refused_by_caller`，服务的工作线程记不进额度，这一帧回忙）。键值原样逐条交给调用方，分段存的值在这一层不拼
/// （拼回在客户端一侧的 `segmented_values`），这一层不攒整帧的条目。
pub struct PrefixScanOutcome {
    pub entries_accepted: usize,
    pub is_truncated: bool,
    pub is_refused_by_caller: bool,
}

/// 删前缀一帧碰到的键（`VerdictLibrary::scan_prefix_for_deletion`）：碰到几把、前缀下是不是已经没有更多的键、最后一把键，
/// 要不要删、碰到的那几把。删除由提交线程在回应记进额度之后才逐把记进写批、同时释放这些键上的认领（`committer.rs` 文件头「删前缀」）。
pub struct PrefixDeletion {
    pub keys_in_frame: u32,
    pub is_exhausted: bool,
    pub last_key: Vec<u8>,
    /// 这一帧要删碰到的键（不是只数不删；弄坏开关 delete-prefix-dry-run-deletes 下只数不删也删）。
    pub deletes_touched_keys: bool,
    /// 碰到的每一把键：要删时、或弄坏开关 delete-prefix-count-releases-claims 打开时才收，别的时候空（只数不删不为它占内存）。
    pub touched_keys: Vec<Vec<u8>>,
}

impl VerdictLibrary {
    /// 打开（没有就建）`library_directory` 那一份库（文件头）。`background_threads` 交给 RocksDB 的压实与落盘。
    pub fn open(
        library_directory: &Path,
        background_threads: usize,
        break_switches: BreakSwitches,
    ) -> Result<VerdictLibrary, LibraryOpenFailure> {
        let options = library_options(background_threads, true);
        let database =
            open_single_column_family_library(library_directory, &options, &break_switches)?;
        Ok(VerdictLibrary {
            database,
            options_holding_statistics: Mutex::new(options),
            break_switches,
            selftest_sync_failure_file: None,
        })
    }

    /// 只给自证起的服务：`file` 在时组提交写完写批不同步、当成同步失败（文件头）。
    pub fn set_selftest_sync_failure_file(&mut self, file: Option<PathBuf>) {
        self.selftest_sync_failure_file = file;
    }

    fn default_column_family(&self) -> &ColumnFamily {
        self.database
            .cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
            .expect("默认列族每份库都有")
    }

    pub fn begin_group(&self) -> GroupBatch {
        GroupBatch {
            batch: WriteBatch::default(),
        }
    }

    /// 这个（身份，请求号）以前生效过的，回当时写了几条。
    pub fn applied_request_entries(
        &self,
        client_identity: &[u8],
        request_number: u64,
    ) -> Result<Option<u32>, String> {
        let value = self
            .database
            .get_pinned(applied_request_key(
                client_identity,
                request_number,
                &self.break_switches,
            ))
            .map_err(|error| format!("查已生效的请求号失败：{error}"))?;
        Ok(value.and_then(|stored| {
            stored
                .get(..4)
                .and_then(|count| count.try_into().ok())
                .map(u32::from_le_bytes)
        }))
    }

    /// 一帧的写加进这一组，同时记下这个请求号已生效。
    pub fn add_write(
        &self,
        group: &mut GroupBatch,
        client_identity: &[u8],
        request_number: u64,
        entries: &[EntryInTable<'_>],
    ) {
        for entry in entries {
            group.batch.put(entry.key, entry.value);
        }
        let entries_written = u32::try_from(entries.len()).expect("条数来自帧里的 u32 字段");
        group.batch.put(
            applied_request_key(client_identity, request_number, &self.break_switches),
            entries_written.to_le_bytes(),
        );
    }

    /// `claim` 是 `claim_key` 算出的键。
    pub fn add_claim(&self, group: &mut GroupBatch, claim: &[u8], record: &ClaimRecord) {
        group.batch.put(claim, encode_claim_record(record));
    }

    pub fn add_claim_release(&self, group: &mut GroupBatch, claim: &[u8]) {
        group.batch.delete(claim);
    }

    /// 删前缀删的一把数据键记进这一组（`scan_prefix_for_deletion` 交回的 `touched_keys` 里的）。
    pub fn add_key_deletion(&self, group: &mut GroupBatch, key: &[u8]) {
        group.batch.delete(key);
    }

    /// 库里有没有这个键（认领判「已有结论」用）。
    pub fn contains_key(&self, key: &[u8]) -> Result<bool, String> {
        self.database
            .get_pinned(key)
            .map(|value| value.is_some())
            .map_err(|error| format!("点读失败：{error}"))
    }

    /// 读出全部认领记录；租期已过的一并删掉（服务起来时调一次，之后认领表只由提交线程改，内存里那一份就是准的）。
    /// 回的键就是库里的键（`claim_key` 的形状）。
    pub fn load_claims(
        &self,
        now_milliseconds: u64,
    ) -> Result<HashMap<Vec<u8>, ClaimRecord>, String> {
        let mut claims = HashMap::new();
        let mut expired = WriteBatch::default();
        let prefix = [CLAIM_KEY_TAG];
        for item in self
            .database
            .iterator(IteratorMode::From(&prefix, Direction::Forward))
        {
            let (key, value) = item.map_err(|error| format!("读认领记录失败：{error}"))?;
            if !key.starts_with(&prefix) {
                break;
            }
            match decode_claim_record(&value) {
                Some(record) if record.lease_expiry_milliseconds > now_milliseconds => {
                    claims.insert(key.into_vec(), record);
                }
                Some(_) | None => expired.delete(key),
            }
        }
        if !expired.is_empty() {
            Self::write_batch_unsynced(&self.database, expired)?;
            Self::flush_and_synchronize(&self.database)?;
        }
        Ok(claims)
    }

    /// 写这一组、同步 WAL；回同步用了多久。组是空的不写不同步。失败分写失败与同步失败两种（`GroupCommitFailure`）。
    pub fn commit_group(&self, group: GroupBatch) -> Result<Option<Duration>, GroupCommitFailure> {
        if group.is_empty() {
            return Ok(None);
        }
        Self::write_batch_unsynced(&self.database, group.batch)
            .map_err(GroupCommitFailure::WriteFailed)?;
        if let Some(file) = &self.selftest_sync_failure_file {
            if file.exists() {
                return Err(GroupCommitFailure::SyncFailed(format!(
                    "同步 WAL 失败：自证造的（{} 在），写批进了内存、没同步",
                    file.display()
                )));
            }
        }
        Self::flush_and_synchronize(&self.database)
            .map(Some)
            .map_err(GroupCommitFailure::SyncFailed)
    }

    fn write_batch_unsynced(database: &DB, batch: WriteBatch) -> Result<(), String> {
        let mut unsynced = WriteOptions::default();
        unsynced.set_sync(false);
        database
            .write_opt(batch, &unsynced)
            .map_err(|error| format!("写失败：{error}"))
    }

    fn flush_and_synchronize(database: &DB) -> Result<Duration, String> {
        let started = Instant::now();
        database
            .flush_wal(true)
            .map_err(|error| format!("同步 WAL 失败：{error}"))?;
        Ok(started.elapsed())
    }

    /// 只写不同步（弄坏开关 acknowledge-before-sync 先写、回确认、过一会才同步）。
    pub fn write_unsynced(&self, group: GroupBatch) -> Result<(), String> {
        Self::write_batch_unsynced(&self.database, group.batch)
    }

    /// 把进程内存里的 WAL 写进文件并同步；回用了多久。
    pub fn synchronize_write_ahead_log(&self) -> Result<Duration, String> {
        Self::flush_and_synchronize(&self.database)
    }

    /// 成批点读：一次 `batched_multi_get_cf` 查完（三张表同在默认列族），回的次序与请求的次序相同。
    pub fn read_points<'library>(
        &'library self,
        keys: &[KeyInTable<'_>],
    ) -> Result<Vec<Option<DBPinnableSlice<'library>>>, String> {
        if self.break_switches.is_enabled(BreakSwitch::MultiGetPerKey) {
            return keys
                .iter()
                .map(|key| {
                    self.database
                        .get_pinned(key.key)
                        .map_err(|error| format!("点读失败：{error}"))
                })
                .collect();
        }
        self.database
            .batched_multi_get_cf(
                self.default_column_family(),
                keys.iter().map(|key| key.key),
                false,
            )
            .into_iter()
            .map(|result| result.map_err(|error| format!("multi_get 失败：{error}")))
            .collect()
    }

    /// 弄坏开关 write-sync-per-entry 的写法：一帧的每一条各自一个写批、各同步一次，最后记下请求号已生效。
    pub fn write_each_entry_synced(
        &self,
        client_identity: &[u8],
        request_number: u64,
        entries: &[EntryInTable<'_>],
    ) -> Result<(), String> {
        debug_assert!(self
            .break_switches
            .is_enabled(BreakSwitch::WriteSyncPerEntry));
        for entry in entries {
            let mut single = self.begin_group();
            single.batch.put(entry.key, entry.value);
            self.commit_group(single)
                .map_err(|failure| failure.message().to_string())?;
        }
        let mut record = self.begin_group();
        self.add_write(&mut record, client_identity, request_number, &[]);
        self.commit_group(record)
            .map(|_| ())
            .map_err(|failure| failure.message().to_string())
    }

    /// 按前缀扫：从前缀起（`after_key` 不空时从排在它之后的第一把键起，`protocol.rs` 文件头「按前缀扫」）往后读，走出前缀、走进记账那一段键（文件头）、
    /// 到条数上限或累计字节到 `maximum_bytes` 就停；每一条读出来就交给 `accept_entry`（键、值），不攒整帧。`accept_entry` 回假（服务记不进额度）就停，
    /// 结局里 `is_refused_by_caller` 为真。弄坏开关 scan-resume-key-ignored 下不看 `after_key`、每帧都从前缀起。
    pub fn scan_prefix_into(
        &self,
        prefix: &[u8],
        after_key: &[u8],
        maximum_entries: u32,
        maximum_bytes: usize,
        accept_entry: &mut dyn FnMut(&[u8], &[u8]) -> bool,
    ) -> Result<PrefixScanOutcome, String> {
        let maximum_entry_count =
            usize::try_from(maximum_entries).expect("这个工程只编 64 位目标，u32 放得进 usize");
        let service_keys_open = self
            .break_switches
            .is_enabled(BreakSwitch::ServiceKeysOpenToClients);
        // 排在 after_key 之后的第一把键是 after_key 后面接一个 0 字节（与 scan_prefix_for_deletion 的续扫起点同一个写法）
        let mut seek_from = after_key.to_vec();
        if seek_from.is_empty()
            || self
                .break_switches
                .is_enabled(BreakSwitch::ScanResumeKeyIgnored)
        {
            seek_from = prefix.to_vec();
        } else {
            seek_from.push(0);
        }
        let mut outcome = PrefixScanOutcome {
            entries_accepted: 0,
            is_truncated: false,
            is_refused_by_caller: false,
        };
        let mut accumulated_bytes = 0_usize;
        for item in self
            .database
            .iterator(IteratorMode::From(&seek_from, Direction::Forward))
        {
            let (key, value) = item.map_err(|error| format!("前缀扫失败：{error}"))?;
            if is_service_key(&key) && !service_keys_open {
                break;
            }
            if !key.starts_with(prefix)
                && !self
                    .break_switches
                    .is_enabled(BreakSwitch::PrefixScanOverruns)
            {
                break;
            }
            if outcome.entries_accepted == maximum_entry_count
                || accumulated_bytes + key.len() + value.len() + 8 > maximum_bytes
            {
                outcome.is_truncated = true;
                break;
            }
            if !accept_entry(&key[..], &value[..]) {
                outcome.is_refused_by_caller = true;
                break;
            }
            accumulated_bytes += key.len() + value.len() + 8;
            outcome.entries_accepted += 1;
        }
        Ok(outcome)
    }

    /// 删前缀一帧要碰的键（`protocol.rs` 文件头「删前缀」）：从前缀起（`after_key` 不空时从排在它之后的第一把键起）按键升序走，
    /// 至多碰 `maximum_keys` 把以前缀起头的键；走出前缀、走进记账那一段、前缀下没有更多的键就停。只扫、不动库：
    /// 不是只数不删时，碰到的每一把键交回在 `touched_keys` 里，调用方先把回应记进额度，记得进才逐把 `add_key_deletion`；只数不删时交回空。
    pub fn scan_prefix_for_deletion(
        &self,
        prefix: &[u8],
        after_key: &[u8],
        maximum_keys: u32,
        count_only: bool,
    ) -> Result<PrefixDeletion, String> {
        let deletes = !count_only
            || self
                .break_switches
                .is_enabled(BreakSwitch::DeletePrefixDryRunDeletes);
        let collects_keys = deletes
            || self
                .break_switches
                .is_enabled(BreakSwitch::DeletePrefixCountReleasesClaims);
        let overruns = self
            .break_switches
            .is_enabled(BreakSwitch::DeletePrefixOverruns);
        // 排在 after_key 之后的第一把键是 after_key 后面接一个 0 字节
        let mut seek_from = after_key.to_vec();
        if seek_from.is_empty() {
            seek_from.extend_from_slice(prefix);
        } else {
            seek_from.push(0);
        }
        let mut keys_in_frame = 0_u32;
        let mut last_key = Vec::new();
        let mut touched_keys = Vec::new();
        let mut is_exhausted = true;
        for item in self
            .database
            .iterator(IteratorMode::From(&seek_from, Direction::Forward))
        {
            let (key, _value) = item.map_err(|error| format!("删前缀扫键失败：{error}"))?;
            if is_service_key(&key) {
                break;
            }
            if !key.starts_with(prefix) && !overruns {
                break;
            }
            if keys_in_frame == maximum_keys {
                is_exhausted = false;
                break;
            }
            keys_in_frame += 1;
            last_key = key.into_vec();
            if collects_keys {
                touched_keys.push(last_key.clone());
            }
        }
        Ok(PrefixDeletion {
            keys_in_frame,
            is_exhausted,
            last_key,
            deletes_touched_keys: deletes,
            touched_keys,
        })
    }

    /// RocksDB 自己数的计数（`REPORTED_TICKERS`）。
    pub fn tickers(&self) -> Vec<(&'static str, u64)> {
        let options = self
            .options_holding_statistics
            .lock()
            .expect("统计锁只在读计数时短暂持有，持有时不会 panic");
        REPORTED_TICKERS
            .iter()
            .map(|ticker| (ticker.name(), options.get_ticker_count(*ticker)))
            .collect()
    }
}
