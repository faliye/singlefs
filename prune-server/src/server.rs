//! 服务进程：开库、开两处监听、收帧、分派、写回应。
//!
//! 每条连接一个读线程、一个写线程：读线程按长度字段切帧，每帧先过准入（这条连接与整个服务的待处理帧数、待处理字节数各有上限），
//! 到了上限当场回「忙、多少毫秒后再来」，不排队；进了的写、认领与删前缀交提交线程组提交（`committer`），读、扫、统计交共用的工作线程池。
//! 处理完把编好的回应连同那一帧的待处理名额交给那条连接的写线程，写线程按到达先后写回，所以回应可能乱序；
//! 回应写进套接字（或连接没了、丢掉）之后名额才放手、待处理计数才减。
//! 帧长度不对时回一个序号 0 的拒绝、写完就关这条连接（帧边界已经找不回来）；别的拒绝之后连接照常可用。
//! TCP 连接两头都开 keepalive，半开的连接由内核认出来关掉，读线程随即收尾。
//!
//! 额度（`Admission`）：待处理字节数一份额度管三样——收着的帧体（读完长度字段、分配帧体之前先记）、进了准入还没回的帧、
//! 正在编与编好了还没写出去的回应（工作线程边编边记：成批点读先数出回应多大、记进再分配，按前缀扫每读出一条先记它的字节再往回应里放，
//! 记不进就整帧回忙、已经记下的放回，不先分配整份回应；统计这类短回应编完再记；提交线程的认领回应与删前缀回应先算字节再记，
//! 记不进回忙、一条都不认领、一把键都不删）。弄坏开关 response-built-before-budget 退回改前：工作线程先编好整份回应、编完才记。不看上限强行记下的只剩写确认（几十字节，已经做了、撤不回）与当场回的拒绝和忙
//! （说明是服务自己写的短句）；删前缀回应带一把可以长到接近一帧的键，不强行记（弄坏开关 delete-prefix-response-unbudgeted 退回强行记下）。
//! 整个服务（`--maximum-pending-bytes`）与每条连接（`--maximum-pending-bytes-per-connection`）各一份上限；
//! 记之前额度里只有这一帧自己（没有别的在用）时照记不误，不然比上限大的一帧永远进不来，所以任何时刻记着的字节至多是上限加一帧
//! （再加强行记下的那几十字节一份的写确认、拒绝与忙）。
//! 收帧时额度不够：帧体照声明的长度读完、丢进一块 `DISCARD_CHUNK_BYTES` 的缓冲（帧边界保住），回忙；不先分配声明那么大的帧体。
//! 弄坏开关 frame-body-read-before-admission 退回先分配、读完帧体再过准入的写法（没收完的帧占的内存不进额度）。
//! 服务占的内存因此有上界：额度上限加一帧（`MAXIMUM_FRAME_BODY_BYTES`；工作线程正在编的回应已在额度里），加每条连接固定的读写缓冲（各 `STREAM_BUFFER_BYTES`）与
//! 不带名额的回应（`connections::UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION`），乘连接数上限（`--maximum-connections`），
//! 加每个工作线程手里正从库里读出、还没放进回应的那一行（成批点读的值钉在 RocksDB 的块缓存里，按前缀扫一次只读一行）。
//! 统计里 `WORKER_RESPONSE_BYTES_PEAK_COUNTER` 报工作线程手里编着的回应字节合计的峰值，自证 worker-responses-within-pending-budget 拿它核这个上界。
//! 期限：一帧从长度字段的头一个字节起，`--frame-read-timeout-milliseconds` 里没收齐就关这条连接（帧开头之前的空闲不限）；
//! 写回应一次堵住 `--write-timeout-milliseconds`（对端不收）就关这条连接，排着的回应丢掉、名额放手。
//! 连接数：同时连着的到了 `--maximum-connections`，新接进来的当场关掉（统计里数一次 `prune-server.connections.refused`）。
//!
//! 每条连接登记在 `connections`（连接号、走哪条、对端、报到的身份）。报到、列连着的客户端、宣布停服务由读线程当场做。
//! 宣布停服务之后：给别的每条连接发一帧「服务停了」，此后任何连接来的帧都只回这一帧、关连接；进程由 systemd 的 SIGTERM 结束，
//! 不等在跑的读写（写都是同步落盘之后才回确认的，没确认的留在客户端的追加日志里）。
//! 服务自身的错（读写库出错、分派错、线程 panic）各记一行故障（`fault_log`）。

use std::io::{self, BufReader, BufWriter, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::client::unix_socket_has_listener;
use crate::command_line::{
    DEFAULT_BUSY_RETRY_MILLISECONDS, DEFAULT_FRAME_READ_TIMEOUT_MILLISECONDS,
    DEFAULT_GROUP_COMMIT_FRAMES, DEFAULT_GROUP_COMMIT_MILLISECONDS, DEFAULT_MAXIMUM_CONNECTIONS,
    DEFAULT_MAXIMUM_PENDING_BYTES, DEFAULT_MAXIMUM_PENDING_BYTES_PER_CONNECTION,
    DEFAULT_MAXIMUM_PENDING_PER_CONNECTION, DEFAULT_MAXIMUM_PENDING_TOTAL,
    DEFAULT_WRITE_TIMEOUT_MILLISECONDS,
};
use crate::committer::{start_committer, CommitJob, CommitterSettings};
use crate::configuration::{library_directory_in, socket_path_problem};
use crate::connections::{
    ConnectionRecord, ConnectionRegistry, OutboundTotals, Outgoing, Responder,
};
use crate::fault_log::{install_panic_recorder, FaultKind, FaultLog};
use crate::keepalive::{enable_keepalive, KeepaliveSettings};
use crate::protocol::{
    begin_response, decode_request_body, encode_busy, encode_connected_clients_response,
    encode_rejection, encode_server_stopping, is_frame_length_acceptable, peek_request_header,
    point_read_response_exceeds_frame_limit, point_read_response_payload_bytes, usize_from_u32,
    DecodedRequest, OperationCode, RequestRejection, RequestView, ResponseStatus, ServiceKeys,
    TrailingBytes, Transport, CONNECTION_LEVEL_SEQUENCE, FRAME_LENGTH_FIELD_BYTES,
    MAXIMUM_FRAME_BODY_BYTES, REQUEST_HEADER_BYTES, RESPONSE_HEADER_BYTES,
};
use crate::storage::{LibraryOpenFailure, VerdictLibrary, DESCRIPTOR_LIMIT_NEXT_STEP};

/// 宣布停服务之后发给客户端的那一帧里的说明。
/// 宣布停服务时等每条连接的「服务停了」写进套接字的限时（写线程堵着的连接到点不再等，记一行故障）。
const STOP_NOTICE_FLUSH_TIMEOUT: Duration = Duration::from_secs(5);
const SERVER_STOPPING_MESSAGE: &str =
    "prune-server 有意停了（prune-server/control stop）；不要重试，服务修好重起之后由提交任务的会话重交";

/// 服务就绪时往标准输出打的一行的开头；起它的一方（自证、压测器、以后的 systemd 检查）认这一行。
pub const LISTENING_LINE_PREFIX: &str = "PRUNE_SERVER_LISTENING";
/// 服务自己数的「处理了几帧」在统计回应里的名字。
pub const FRAMES_HANDLED_COUNTER: &str = "prune-server.frames.handled";
/// 统计回应里服务自己数的几项（自证与压测器按名字取）。
pub const PENDING_PEAK_COUNTER: &str = "prune-server.pending.peak";
pub const PENDING_LIMIT_COUNTER: &str = "prune-server.pending.limit";
pub const BUSY_REJECTIONS_COUNTER: &str = "prune-server.busy.rejections";
pub const DUPLICATE_REQUESTS_COUNTER: &str = "prune-server.duplicate.requests";
pub const GROUP_COMMITS_COUNTER: &str = "prune-server.group.commits";
pub const WRITE_AHEAD_LOG_SYNC_COUNT_COUNTER: &str = "prune-server.wal.sync.count";
pub const WRITE_AHEAD_LOG_SYNC_MEDIAN_COUNTER: &str = "prune-server.wal.sync.p50.microseconds";
pub const WRITE_AHEAD_LOG_SYNC_NINETY_NINTH_PERCENTILE_COUNTER: &str =
    "prune-server.wal.sync.p99.microseconds";
pub const WRITE_AHEAD_LOG_SYNC_MAXIMUM_COUNTER: &str = "prune-server.wal.sync.maximum.microseconds";
/// 额度里此刻记着的字节（收着的帧体、待处理的帧、没写出去的回应）与它的峰值（文件头「额度」）。
pub const PENDING_BYTES_COUNTER: &str = "prune-server.pending.bytes";
pub const PENDING_BYTES_PEAK_COUNTER: &str = "prune-server.pending.bytes.peak";
/// 给还没收完的帧体分配着的字节的峰值。
pub const RECEIVING_BODY_BYTES_PEAK_COUNTER: &str = "prune-server.receive.allocated.bytes.peak";
/// 工作线程手里正在编、还没交给写线程的回应字节合计的峰值（文件头「额度」）。
pub const WORKER_RESPONSE_BYTES_PEAK_COUNTER: &str = "prune-server.worker.response.bytes.peak";
/// 各连接写出队里排着、还没写出去的字节合计的峰值（`connections` 文件头「写出队的账」）。
pub const OUTBOUND_QUEUED_BYTES_PEAK_COUNTER: &str = "prune-server.outbound.queued.bytes.peak";
pub const CONNECTIONS_OPEN_COUNTER: &str = "prune-server.connections.open";
pub const CONNECTIONS_REFUSED_COUNTER: &str = "prune-server.connections.refused";
/// 因帧没在期限里收齐、写回应堵住、不带名额的回应排超了上限而关掉的连接数。
pub const FRAME_READ_TIMEOUTS_COUNTER: &str = "prune-server.connections.closed.frame-read-timeout";
pub const WRITE_TIMEOUTS_COUNTER: &str = "prune-server.connections.closed.write-timeout";
pub const OUTBOUND_OVERFLOWS_COUNTER: &str = "prune-server.connections.closed.outbound-overflow";
/// 退出码：同步 WAL 失败，服务退出、重起时重开库（`storage.rs` 文件头「同步 WAL 失败之后」）。
pub const STORAGE_SYNC_FAILED_EXIT_CODE: i32 = 7;
/// 读写缓冲：满块一帧约 9 MB，缓冲给 1 MiB，大帧直接穿过缓冲读写。
const STREAM_BUFFER_BYTES: usize = 1024 * 1024;
/// 收帧额度不够时帧体读进这么大的一块缓冲丢掉（文件头「额度」）。
const DISCARD_CHUNK_BYTES: usize = 64 * 1024;
/// WAL 同步用时留最近这么多次（环形），算分布用；每个 4 字节，满了约 4 MB。
const WRITE_AHEAD_LOG_SYNC_SAMPLES_KEPT: usize = 1 << 20;

/// 只给自证用：处理到这个序号的帧时先等一会，造出确定的乱序回应。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelftestDelay {
    pub sequence: u64,
    pub delay: Duration,
}

/// 准入的上限、期限与忙时叫客户端隔多久再来（文件头「额度」「期限」「连接数」）。
#[derive(Clone, Copy, Debug)]
pub struct AdmissionLimits {
    pub maximum_pending_per_connection: usize,
    pub maximum_pending_total: usize,
    pub maximum_pending_bytes: usize,
    pub maximum_pending_bytes_per_connection: usize,
    pub maximum_connections: usize,
    pub frame_read_timeout: Duration,
    pub write_timeout: Duration,
    pub busy_retry: Duration,
}

pub struct ServerSettings {
    pub socket_path: PathBuf,
    pub network_address: String,
    pub state_directory: PathBuf,
    pub library_directory: PathBuf,
    pub worker_threads: usize,
    pub break_switches: BreakSwitches,
    pub selftest_delay: Option<SelftestDelay>,
    /// 只给自证：这个文件在时组提交不同步、当成同步失败（`storage.rs` 文件头）。
    pub selftest_sync_failure_file: Option<PathBuf>,
    pub admission: AdmissionLimits,
    pub committer: CommitterSettings,
}

/// 起不来的几种，各对应一个退出码（`exit_code`），说明里写卡在哪。
#[derive(Debug)]
pub enum ServeFailure {
    LibraryUnavailable(String),
    /// 库对不上：不是一份只有默认列族、选项对得上的 RocksDB（`storage.rs` 文件头）。
    LibraryMismatched(String),
    /// 文件数上限不够：软上限抬到硬上限之后仍不够开这份库（`storage.rs` 文件头「文件数上限」）。
    DescriptorLimitTooLow(String),
    SocketUnavailable(String),
    NetworkUnavailable(String),
    ListenerStopped(String),
}

/// 退出码：库对不上（`ServeFailure::LibraryMismatched`）。
pub const LIBRARY_MISMATCHED_EXIT_CODE: i32 = 6;
/// 退出码：文件数上限不够（`ServeFailure::DescriptorLimitTooLow`，`storage.rs` 文件头「文件数上限」）。
pub const DESCRIPTOR_LIMIT_TOO_LOW_EXIT_CODE: i32 = 8;

impl ServeFailure {
    pub fn exit_code(&self) -> i32 {
        match self {
            ServeFailure::ListenerStopped(_) => 1,
            ServeFailure::LibraryUnavailable(_) => 3,
            ServeFailure::SocketUnavailable(_) => 4,
            ServeFailure::NetworkUnavailable(_) => 5,
            ServeFailure::LibraryMismatched(_) => LIBRARY_MISMATCHED_EXIT_CODE,
            ServeFailure::DescriptorLimitTooLow(_) => DESCRIPTOR_LIMIT_TOO_LOW_EXIT_CODE,
        }
    }

    pub fn describe(&self) -> (String, &'static str) {
        match self {
            ServeFailure::LibraryUnavailable(message) => (
                message.clone(),
                "库目录被别的进程持有（RocksDB 的 LOCK）就先停掉那个进程（另一个服务）；同一个库只许一个进程开着",
            ),
            ServeFailure::LibraryMismatched(message) => (
                message.clone(),
                "服务只开一种库（只有默认列族、LZ4、缺省比较器，src/storage.rs 文件头），不建列族、不 repair、不改库；\
                 --library 指错了就改对；不是这一种的库不认，挪开那个目录让服务建一份新的，判定在新库里重判",
            ),
            ServeFailure::DescriptorLimitTooLow(message) => {
                (message.clone(), DESCRIPTOR_LIMIT_NEXT_STEP)
            }
            ServeFailure::SocketUnavailable(message) => {
                (message.clone(), "已有服务在回话就不再起第二个；路径太长就换短的 --socket；目录权限不是 700 就改回 700")
            }
            ServeFailure::NetworkUnavailable(message) => (
                message.clone(),
                "在仓根 multi-host.env 加一行 PRUNE_SERVER_TCP_ADDRESS=<本机在有线直连段上的地址>:<端口>，或命令行给 --tcp <地址:端口>；端口被占就换一个",
            ),
            ServeFailure::ListenerStopped(message) => (message.clone(), "看上面监听线程报的错；修好之后重起服务"),
        }
    }
}

struct Job {
    body: Vec<u8>,
    responder: Responder,
    slot: PendingSlot,
    connection: Arc<ConnectionRecord>,
}

/// 服务自己数的几项。
#[derive(Default)]
pub struct ServerCounters {
    pub frames_handled: AtomicU64,
    pub busy_rejections: AtomicU64,
    pub duplicate_requests: AtomicU64,
    pub group_commits: AtomicU64,
    pub connections_open: AtomicUsize,
    pub connections_refused: AtomicU64,
    pub frame_read_timeouts: AtomicU64,
    pub write_timeouts: AtomicU64,
    /// 工作线程手里正在编的回应字节合计与它的峰值（`ResponseBudget`）。
    pub worker_response_bytes: AtomicUsize,
    pub worker_response_bytes_peak: AtomicUsize,
}

/// WAL 同步用时：最近 `WRITE_AHEAD_LOG_SYNC_SAMPLES_KEPT` 次的样本（微秒）、总次数与最大值。
pub struct WriteAheadLogSyncTimings {
    samples: Mutex<Vec<u32>>,
    count: AtomicU64,
    maximum_microseconds: AtomicU64,
}

impl WriteAheadLogSyncTimings {
    fn new() -> WriteAheadLogSyncTimings {
        WriteAheadLogSyncTimings {
            samples: Mutex::new(Vec::new()),
            count: AtomicU64::new(0),
            maximum_microseconds: AtomicU64::new(0),
        }
    }

    pub fn record(&self, duration: Duration) {
        let microseconds = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);
        let position = self.count.fetch_add(1, Ordering::Relaxed);
        self.maximum_microseconds
            .fetch_max(microseconds, Ordering::Relaxed);
        let sample = u32::try_from(microseconds).unwrap_or(u32::MAX);
        let mut samples = self
            .samples
            .lock()
            .expect("同步用时样本的锁只在记一次与算分布时短暂持有");
        if samples.len() < WRITE_AHEAD_LOG_SYNC_SAMPLES_KEPT {
            samples.push(sample);
        } else {
            let slot = usize::try_from(position % WRITE_AHEAD_LOG_SYNC_SAMPLES_KEPT as u64)
                .expect("环形下标小于样本数");
            samples[slot] = sample;
        }
    }

    /// （次数，p50，p99，最大），单位微秒；没有样本时都是 0。
    fn summary(&self) -> (u64, u64, u64, u64) {
        let mut sorted = self
            .samples
            .lock()
            .expect("同步用时样本的锁只在记一次与算分布时短暂持有")
            .clone();
        sorted.sort_unstable();
        let percentile = |percent: usize| -> u64 {
            if sorted.is_empty() {
                return 0;
            }
            let rank = (sorted.len() * percent).div_ceil(100).max(1);
            u64::from(sorted[rank - 1])
        };
        (
            self.count.load(Ordering::Relaxed),
            percentile(50),
            percentile(99),
            self.maximum_microseconds.load(Ordering::Relaxed),
        )
    }
}

/// 准入：这条连接与整个服务的待处理帧数、记着的字节（文件头「额度」）。进了的帧拿一个 `PendingSlot`，
/// 回应写出去或丢掉时放手、计数减回去。
pub struct Admission {
    limits: AdmissionLimits,
    is_unbounded: bool,
    pending_total: AtomicUsize,
    pending_bytes: AtomicUsize,
    pending_peak: AtomicUsize,
    pending_bytes_peak: AtomicUsize,
    receiving_body_bytes: AtomicUsize,
    receiving_body_bytes_peak: AtomicUsize,
}

/// 一条连接在准入里的账：待处理帧数、记着的字节。
#[derive(Default)]
pub struct ConnectionBudget {
    pending_frames: AtomicUsize,
    held_bytes: AtomicUsize,
}

/// 记进额度的一笔字节（整个服务与这条连接各记一份），放手时销掉。
pub struct ByteReservation {
    admission: Arc<Admission>,
    budget: Arc<ConnectionBudget>,
    bytes: usize,
}

impl Drop for ByteReservation {
    fn drop(&mut self) {
        self.admission
            .pending_bytes
            .fetch_sub(self.bytes, Ordering::SeqCst);
        self.budget
            .held_bytes
            .fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

/// 在 `counter` 上再记 `amount`：记完不超 `limit`，或记之前 `counter` 里只有这一笔自己的 `already_held_here`
/// （没有别的在用额度；不放过的话比上限大的一帧永远进不来），就记下回真；否则不记、回假。
fn reserve_against(
    counter: &AtomicUsize,
    amount: usize,
    limit: usize,
    already_held_here: usize,
) -> bool {
    let before = counter.fetch_add(amount, Ordering::SeqCst);
    if before + amount <= limit || before == already_held_here {
        return true;
    }
    counter.fetch_sub(amount, Ordering::SeqCst);
    false
}

impl ByteReservation {
    /// 这一笔再多记 `extra`（回应编好之后记它的字节）；`is_forced` 的不看上限（已经做了、撤不回的写确认）。记不进回假、这一笔不变。
    fn grow(&mut self, extra: usize, is_forced: bool) -> bool {
        let admission = &self.admission;
        let limits = admission.limits;
        if is_forced || admission.is_unbounded {
            self.budget.held_bytes.fetch_add(extra, Ordering::SeqCst);
            admission.pending_bytes.fetch_add(extra, Ordering::SeqCst);
        } else {
            if !reserve_against(
                &self.budget.held_bytes,
                extra,
                limits.maximum_pending_bytes_per_connection,
                self.bytes,
            ) {
                return false;
            }
            if !reserve_against(
                &admission.pending_bytes,
                extra,
                limits.maximum_pending_bytes,
                self.bytes,
            ) {
                self.budget.held_bytes.fetch_sub(extra, Ordering::SeqCst);
                return false;
            }
        }
        self.bytes += extra;
        admission.note_pending_bytes_peak();
        true
    }

    /// 这一笔放回 `released` 字节（工作线程边编边记、编到一半记不进时把已经记下的放回）。
    fn shrink(&mut self, released: usize) {
        let released = released.min(self.bytes);
        self.budget.held_bytes.fetch_sub(released, Ordering::SeqCst);
        self.admission
            .pending_bytes
            .fetch_sub(released, Ordering::SeqCst);
        self.bytes -= released;
    }
}

/// 一帧占着的待处理名额：帧数一个，字节记在 `reservation` 里（请求，回应编好之后再加上回应）。
pub struct PendingSlot {
    admission: Arc<Admission>,
    budget: Arc<ConnectionBudget>,
    reservation: ByteReservation,
}

impl PendingSlot {
    /// 回应编好之后记它的字节；记不进回假（调用方把回应换成忙）。
    pub fn reserve_response(&mut self, response_bytes: usize) -> bool {
        self.reservation.grow(response_bytes, false)
    }

    /// 已经做了、撤不回的回应（写确认）照记，不看上限。
    pub fn force_response(&mut self, response_bytes: usize) {
        self.reservation.grow(response_bytes, true);
    }

    /// 放回先前 `reserve_response` 记下的 `response_bytes`（回应编到一半记不进、换成忙时）。
    pub fn release_response(&mut self, response_bytes: usize) {
        self.reservation.shrink(response_bytes);
    }
}

/// 工作线程编一帧回应时的账（文件头「额度」）：边编边记进这一帧的名额（`take`），记不进就由调用方整帧回忙、已经记下的放回（`release`），
/// 不先分配整份回应；手里编着的字节另记进 `ServerCounters::worker_response_bytes`（统计报峰值），放手时减回去。
/// 弄坏开关 response-built-before-budget 下 `take` 不记额度、只记手里的字节，编完整份由工作线程再记（改前的写法）。
struct ResponseBudget<'work> {
    slot: &'work mut PendingSlot,
    counters: &'work ServerCounters,
    reserves_while_building: bool,
    reserved_bytes: usize,
    held_bytes: usize,
}

impl<'work> ResponseBudget<'work> {
    fn new(
        slot: &'work mut PendingSlot,
        counters: &'work ServerCounters,
        reserves_while_building: bool,
    ) -> ResponseBudget<'work> {
        ResponseBudget {
            slot,
            counters,
            reserves_while_building,
            reserved_bytes: 0,
            held_bytes: 0,
        }
    }

    /// 回应的缓冲要再分配 `bytes` 字节：先记进额度（记不进回假，调用方回忙），再算进手里编着的字节；0 字节不用记。
    fn take(&mut self, bytes: usize) -> bool {
        if bytes == 0 {
            return true;
        }
        if self.reserves_while_building {
            if !self.slot.reserve_response(bytes) {
                return false;
            }
            self.reserved_bytes += bytes;
        }
        self.held_bytes += bytes;
        let now_held = self
            .counters
            .worker_response_bytes
            .fetch_add(bytes, Ordering::SeqCst)
            + bytes;
        self.counters
            .worker_response_bytes_peak
            .fetch_max(now_held, Ordering::SeqCst);
        true
    }

    /// 编到一半记不进：已经记下的放回。
    fn release(&mut self) {
        self.slot.release_response(self.reserved_bytes);
        self.reserved_bytes = 0;
    }

    /// 编到一半记不进时回的忙（这一帧做过、回应没留，读与扫不改库，重发即可）。
    fn busy_response(
        &mut self,
        shared: &SharedState,
        sequence: u64,
        operation_code: u8,
        wanted_bytes: usize,
    ) -> Vec<u8> {
        self.release();
        let reason = shared
            .admission
            .describe_bytes_refusal(&self.slot.budget, wanted_bytes);
        shared
            .counters
            .busy_rejections
            .fetch_add(1, Ordering::Relaxed);
        let retry_after = shared.retry_after_milliseconds();
        encode_busy(
            sequence,
            operation_code,
            retry_after,
            &format!("忙：回应编到一半{reason}；这一帧做过、回应没留，{retry_after} 毫秒后再来"),
        )
    }
}

impl Drop for ResponseBudget<'_> {
    fn drop(&mut self) {
        self.counters
            .worker_response_bytes
            .fetch_sub(self.held_bytes, Ordering::SeqCst);
    }
}

impl Drop for PendingSlot {
    fn drop(&mut self) {
        self.budget.pending_frames.fetch_sub(1, Ordering::SeqCst);
        self.admission.pending_total.fetch_sub(1, Ordering::SeqCst);
    }
}

/// 给一帧帧体分配着的字节，收齐或放弃时放手（统计里报峰值）。
struct ReceivingBody<'admission> {
    admission: &'admission Admission,
    bytes: usize,
}

impl<'admission> ReceivingBody<'admission> {
    fn start(admission: &'admission Admission, bytes: usize) -> ReceivingBody<'admission> {
        let now_receiving = admission
            .receiving_body_bytes
            .fetch_add(bytes, Ordering::SeqCst)
            + bytes;
        admission
            .receiving_body_bytes_peak
            .fetch_max(now_receiving, Ordering::SeqCst);
        ReceivingBody { admission, bytes }
    }
}

impl Drop for ReceivingBody<'_> {
    fn drop(&mut self) {
        self.admission
            .receiving_body_bytes
            .fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

/// 先加再核：加完超了上限就减回去拒掉，所以进了的帧数任何时刻都不超上限。
fn reserve(
    counter: &AtomicUsize,
    amount: usize,
    limit: usize,
    is_unbounded: bool,
) -> Result<usize, usize> {
    let reached = counter.fetch_add(amount, Ordering::SeqCst) + amount;
    if reached > limit && !is_unbounded {
        counter.fetch_sub(amount, Ordering::SeqCst);
        return Err(reached - amount);
    }
    Ok(reached)
}

impl Admission {
    fn new(limits: AdmissionLimits, is_unbounded: bool) -> Admission {
        Admission {
            limits,
            is_unbounded,
            pending_total: AtomicUsize::new(0),
            pending_bytes: AtomicUsize::new(0),
            pending_peak: AtomicUsize::new(0),
            pending_bytes_peak: AtomicUsize::new(0),
            receiving_body_bytes: AtomicUsize::new(0),
            receiving_body_bytes_peak: AtomicUsize::new(0),
        }
    }

    fn note_pending_bytes_peak(&self) {
        self.pending_bytes_peak
            .fetch_max(self.pending_bytes.load(Ordering::SeqCst), Ordering::SeqCst);
    }

    /// 收一帧帧体之前先记它的字节（文件头「额度」）；记不进回 `None`（调用方把帧体读完丢掉、回忙）。
    fn reserve_bytes(
        self: &Arc<Self>,
        budget: &Arc<ConnectionBudget>,
        bytes: usize,
    ) -> Option<ByteReservation> {
        let mut reservation = ByteReservation {
            admission: Arc::clone(self),
            budget: Arc::clone(budget),
            bytes: 0,
        };
        reservation.grow(bytes, false).then_some(reservation)
    }

    /// 帧数过准入；`reservation` 记着这一帧的字节，进了就交给名额，没进随错误放手。
    fn admit(
        self: &Arc<Self>,
        budget: &Arc<ConnectionBudget>,
        reservation: ByteReservation,
    ) -> Result<PendingSlot, String> {
        let limits = self.limits;
        reserve(
            &budget.pending_frames,
            1,
            limits.maximum_pending_per_connection,
            self.is_unbounded,
        )
        .map_err(|held| {
            format!(
                "这条连接已有 {held} 帧待处理，上限 {}",
                limits.maximum_pending_per_connection
            )
        })?;
        let total = match reserve(
            &self.pending_total,
            1,
            limits.maximum_pending_total,
            self.is_unbounded,
        ) {
            Ok(total) => total,
            Err(held) => {
                budget.pending_frames.fetch_sub(1, Ordering::SeqCst);
                return Err(format!(
                    "服务已有 {held} 帧待处理，上限 {}",
                    limits.maximum_pending_total
                ));
            }
        };
        self.pending_peak.fetch_max(total, Ordering::SeqCst);
        Ok(PendingSlot {
            admission: Arc::clone(self),
            budget: Arc::clone(budget),
            reservation,
        })
    }

    /// 记不进额度时说明里报的现状。
    fn describe_bytes_refusal(&self, budget: &ConnectionBudget, bytes: usize) -> String {
        format!(
            "{bytes} 字节记不进额度：服务记着 {} 字节（上限 {}），这条连接记着 {} 字节（上限 {}）",
            self.pending_bytes.load(Ordering::SeqCst),
            self.limits.maximum_pending_bytes,
            budget.held_bytes.load(Ordering::SeqCst),
            self.limits.maximum_pending_bytes_per_connection
        )
    }

    fn retry_after_milliseconds(&self) -> u32 {
        u32::try_from(self.limits.busy_retry.as_millis()).unwrap_or(u32::MAX)
    }
}

pub struct SharedState {
    pub library: VerdictLibrary,
    pub break_switches: BreakSwitches,
    selftest_delay: Option<SelftestDelay>,
    pub counters: ServerCounters,
    pub write_ahead_log_sync_timings: WriteAheadLogSyncTimings,
    admission: Arc<Admission>,
    pub connections: ConnectionRegistry,
    pub outbound_totals: Arc<OutboundTotals>,
    pub fault_log: Arc<FaultLog>,
    /// 宣布了停服务。
    is_stopping: AtomicBool,
    /// 放手时回话（库关完之后）：必须是最后一个字段，字段按声明的次序放手，排在它前面的库先关。
    #[expect(
        dead_code,
        reason = "只为放手次序存在：不读，随共享状态最后放手时由它自己的 Drop 回话"
    )]
    release_notice: ReleaseNotice,
}

impl SharedState {
    pub fn is_enabled(&self, switch: BreakSwitch) -> bool {
        self.break_switches.is_enabled(switch)
    }

    pub fn retry_after_milliseconds(&self) -> u32 {
        self.admission.retry_after_milliseconds()
    }

    /// 回一帧忙（不带名额的回应）；排不进去（连接没了或不带名额的回应排超了上限）回假。
    pub fn reply_busy(
        &self,
        responder: &Responder,
        sequence: u64,
        operation_code: u8,
        reason: &str,
    ) -> bool {
        self.counters
            .busy_rejections
            .fetch_add(1, Ordering::Relaxed);
        let retry_after = self.retry_after_milliseconds();
        let message = format!("忙：{reason}；{retry_after} 毫秒后再来");
        responder
            .send(encode_busy(sequence, operation_code, retry_after, &message))
            .is_ok()
    }

    /// 接进来一条连接：没到连接数上限就记一条、回守卫（放手时减回去）；到了回 `None`、数一次拒。
    fn open_connection(self: &Arc<Self>) -> Option<OpenConnection> {
        let reached = self
            .counters
            .connections_open
            .fetch_add(1, Ordering::SeqCst)
            + 1;
        if reached > self.admission.limits.maximum_connections {
            self.counters
                .connections_open
                .fetch_sub(1, Ordering::SeqCst);
            self.counters
                .connections_refused
                .fetch_add(1, Ordering::Relaxed);
            return None;
        }
        Some(OpenConnection {
            shared: Arc::clone(self),
        })
    }
}

/// 一条记进连接数的连接；放手时减回去。
struct OpenConnection {
    shared: Arc<SharedState>,
}

impl Drop for OpenConnection {
    fn drop(&mut self) {
        self.shared
            .counters
            .connections_open
            .fetch_sub(1, Ordering::SeqCst);
    }
}

/// 读线程把一帧交给谁。
struct Dispatch {
    workers: Sender<Job>,
    committer: Sender<CommitJob>,
}

impl Clone for Dispatch {
    fn clone(&self) -> Dispatch {
        Dispatch {
            workers: self.workers.clone(),
            committer: self.committer.clone(),
        }
    }
}

trait ConnectionStream: Read + Write + Send + Sync + Sized + 'static {
    fn clone_for_writing(&self) -> io::Result<Self>;
    fn shut_down(&self);
    fn set_read_deadline_timeout(&self, timeout: Option<Duration>) -> io::Result<()>;
    fn set_write_deadline_timeout(&self, timeout: Option<Duration>) -> io::Result<()>;
}

impl ConnectionStream for UnixStream {
    fn clone_for_writing(&self) -> io::Result<UnixStream> {
        self.try_clone()
    }
    fn shut_down(&self) {
        // 对端可能已经先关了；关不上不影响这条连接的收尾。
        let _ = self.shutdown(Shutdown::Both);
    }
    fn set_read_deadline_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.set_read_timeout(timeout)
    }
    fn set_write_deadline_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.set_write_timeout(timeout)
    }
}

impl ConnectionStream for TcpStream {
    fn clone_for_writing(&self) -> io::Result<TcpStream> {
        self.try_clone()
    }
    fn shut_down(&self) {
        // 同上。
        let _ = self.shutdown(Shutdown::Both);
    }
    fn set_read_deadline_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.set_read_timeout(timeout)
    }
    fn set_write_deadline_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.set_write_timeout(timeout)
    }
}

/// 一条连接的来路：走哪条、对端地址。
struct ConnectionOrigin {
    transport: Transport,
    peer_address: String,
}

fn prepare_socket_directory(settings: &ServerSettings) -> Result<(), ServeFailure> {
    if let Some(problem) = socket_path_problem(&settings.socket_path) {
        return Err(ServeFailure::SocketUnavailable(problem));
    }
    let Some(directory) = settings.socket_path.parent() else {
        return Err(ServeFailure::SocketUnavailable(format!(
            "socket 路径 {} 没有上一层目录",
            settings.socket_path.display()
        )));
    };
    if !directory.exists() {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)
            .map_err(|error| {
                ServeFailure::SocketUnavailable(format!(
                    "建不了 socket 目录 {}：{error}",
                    directory.display()
                ))
            })?;
    }
    let metadata = std::fs::metadata(directory).map_err(|error| {
        ServeFailure::SocketUnavailable(format!(
            "读不了 socket 目录 {}：{error}",
            directory.display()
        ))
    })?;
    if !metadata.is_dir() {
        return Err(ServeFailure::SocketUnavailable(format!(
            "socket 目录 {} 不是目录",
            directory.display()
        )));
    }
    // socket 目录归服务管：谁建的、umask 多宽都不管，服务起来时把它收成 700（第二台 umask 002 建出来的是 775）
    let mode = metadata.permissions().mode() & 0o777;
    if mode != 0o700 {
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700)).map_err(
            |error| {
                ServeFailure::SocketUnavailable(format!(
                    "socket 目录 {} 的权限是 {mode:o}，改成 700 没成：{error}",
                    directory.display()
                ))
            },
        )?;
    }
    if settings.socket_path.exists() {
        if unix_socket_has_listener(&settings.socket_path) {
            return Err(ServeFailure::SocketUnavailable(format!(
                "已有服务在 {} 上回话",
                settings.socket_path.display()
            )));
        }
        std::fs::remove_file(&settings.socket_path).map_err(|error| {
            ServeFailure::SocketUnavailable(format!(
                "删不了没人回话的旧 socket {}：{error}",
                settings.socket_path.display()
            ))
        })?;
    }
    Ok(())
}

/// 状态目录（故障日志在里面）：没有就建成 700。
fn prepare_state_directory(settings: &ServerSettings) -> Result<(), ServeFailure> {
    if settings.state_directory.exists() {
        return Ok(());
    }
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&settings.state_directory)
        .map_err(|error| {
            ServeFailure::LibraryUnavailable(format!(
                "建不了状态目录 {}：{error}",
                settings.state_directory.display()
            ))
        })
}

/// 服务开好了库、绑好了两处监听、起好了提交线程与工作线程池，还没开始接连接的样子（[`serve`] 与 [`start_in_this_process`] 共用这一段）。
struct OpenedService {
    shared: Arc<SharedState>,
    dispatch: Dispatch,
    unix_listener: UnixListener,
    network_listener: TcpListener,
    bound_network_address: SocketAddr,
}

/// 线程 panic 记不记进故障日志：服务进程记（装进程级的 panic 钩子）；在调用方进程里起的那一份不装，那个进程的 panic 钩子是它自己的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PanicRecording {
    IntoTheFaultLog,
    LeftToTheHostProcess,
}

/// 开库、绑监听、起提交线程与工作线程池（[`OpenedService`]）；`release_notice` 随共享状态最后放手（库关完之后）时回话。
fn open_the_service(
    settings: ServerSettings,
    panic_recording: PanicRecording,
    release_notice: ReleaseNotice,
) -> Result<OpenedService, ServeFailure> {
    prepare_socket_directory(&settings)?;
    prepare_state_directory(&settings)?;
    let fault_log = Arc::new(
        FaultLog::open(&settings.state_directory, &settings.break_switches)
            .map_err(ServeFailure::LibraryUnavailable)?,
    );
    match panic_recording {
        PanicRecording::IntoTheFaultLog => install_panic_recorder(Arc::clone(&fault_log)),
        PanicRecording::LeftToTheHostProcess => {
            // 宿主进程的 panic 钩子归它自己，这一份不动
        }
    }
    if let Some(parent) = settings.library_directory.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            ServeFailure::LibraryUnavailable(format!(
                "建不了库目录的上一层 {}：{error}",
                parent.display()
            ))
        })?;
    }
    let mut library = VerdictLibrary::open(
        &settings.library_directory,
        settings.worker_threads,
        settings.break_switches.clone(),
    )
    .map_err(|failure| match failure {
        LibraryOpenFailure::Unavailable(message) => ServeFailure::LibraryUnavailable(message),
        LibraryOpenFailure::Mismatched(message) => ServeFailure::LibraryMismatched(message),
        LibraryOpenFailure::DescriptorLimitTooLow(message) => {
            ServeFailure::DescriptorLimitTooLow(message)
        }
    })?;
    library.set_selftest_sync_failure_file(settings.selftest_sync_failure_file.clone());
    let unix_listener = UnixListener::bind(&settings.socket_path).map_err(|error| {
        ServeFailure::SocketUnavailable(format!(
            "绑不上 socket {}：{error}",
            settings.socket_path.display()
        ))
    })?;
    std::fs::set_permissions(
        &settings.socket_path,
        std::fs::Permissions::from_mode(0o600),
    )
    .map_err(|error| ServeFailure::SocketUnavailable(format!("改不了 socket 权限：{error}")))?;
    let network_listener = TcpListener::bind(&settings.network_address).map_err(|error| {
        ServeFailure::NetworkUnavailable(format!(
            "绑不上 TCP 地址 {}：{error}",
            settings.network_address
        ))
    })?;
    let bound_network_address = network_listener.local_addr().map_err(|error| {
        ServeFailure::NetworkUnavailable(format!("读不出 TCP 监听的地址：{error}"))
    })?;

    let shared = Arc::new(SharedState {
        library,
        break_switches: settings.break_switches.clone(),
        selftest_delay: settings.selftest_delay,
        counters: ServerCounters::default(),
        write_ahead_log_sync_timings: WriteAheadLogSyncTimings::new(),
        admission: Arc::new(Admission::new(
            settings.admission,
            settings
                .break_switches
                .is_enabled(BreakSwitch::BusyReplacedByUnboundedQueue),
        )),
        connections: ConnectionRegistry::default(),
        outbound_totals: Arc::new(OutboundTotals::default()),
        fault_log,
        is_stopping: AtomicBool::new(false),
        release_notice,
    });
    let committer =
        start_committer(&shared, settings.committer).map_err(ServeFailure::LibraryUnavailable)?;
    let dispatch = Dispatch {
        workers: start_worker_pool(&shared, settings.worker_threads),
        committer,
    };
    Ok(OpenedService {
        shared,
        dispatch,
        unix_listener,
        network_listener,
        bound_network_address,
    })
}

/// 两个接连接的线程（unix socket、TCP）；`stop_requested` 置上之后各自接到的下一条连接不收、线程收尾。
fn start_accepting(
    opened: OpenedService,
    stop_requested: &Arc<AtomicBool>,
) -> [thread::JoinHandle<AcceptLoopEnd>; 2] {
    let OpenedService {
        shared,
        dispatch,
        unix_listener,
        network_listener,
        bound_network_address: _,
    } = opened;
    let unix_accept = {
        let shared = Arc::clone(&shared);
        let dispatch = dispatch.clone();
        let stop_requested = Arc::clone(stop_requested);
        thread::spawn(move || {
            accept_unix_connections(unix_listener, &shared, &dispatch, &stop_requested)
        })
    };
    let network_accept = {
        let stop_requested = Arc::clone(stop_requested);
        thread::spawn(move || {
            accept_network_connections(network_listener, &shared, &dispatch, &stop_requested)
        })
    };
    [unix_accept, network_accept]
}

/// 起服务；正常情况下不返回（由 SIGTERM 结束：写都是同步的，被杀不丢已回应的写）。
pub fn serve(settings: ServerSettings) -> Result<(), ServeFailure> {
    let socket_path = settings.socket_path.clone();
    let worker_threads = settings.worker_threads;
    let library_directory = settings.library_directory.clone();
    let opened = open_the_service(
        settings,
        PanicRecording::IntoTheFaultLog,
        ReleaseNotice::Unwatched,
    )?;
    println!(
        "{LISTENING_LINE_PREFIX} unix={} tcp={} library={} faults={} pid={} worker_threads={worker_threads}",
        socket_path.display(),
        opened.bound_network_address,
        library_directory.display(),
        opened.shared.fault_log.path().display(),
        std::process::id(),
    );
    io::stdout()
        .flush()
        .map_err(|error| ServeFailure::ListenerStopped(format!("就绪行写不出去：{error}")))?;

    // 服务进程里没有停服句柄：这个标记一直不置，接连接的线程只在监听出错时收尾
    let never_stopped = Arc::new(AtomicBool::new(false));
    let [unix_accept, network_accept] = start_accepting(opened, &never_stopped);
    let unix_result = unix_accept.join();
    let network_result = network_accept.join();
    let describe = |name: &str, result: thread::Result<AcceptLoopEnd>| match result {
        Ok(AcceptLoopEnd::ListenerFailed(error)) => format!("{name} 监听停了：{error}"),
        Ok(AcceptLoopEnd::StopRequested) => format!("{name} 监听被停服句柄叫停了"),
        Err(_) => format!("{name} 监听线程 panic 了"),
    };
    Err(ServeFailure::ListenerStopped(format!(
        "{}；{}",
        describe("unix socket", unix_result),
        describe("TCP", network_result)
    )))
}

/// 在调用方进程里起的那一份用几个工作线程（也是交给 RocksDB 的后台线程数）：只给测试自己起的临时服务，一个进程里同时开着好几份，各开少几个。
pub const WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS: usize = 2;
/// 在调用方进程里起的那一份绑的 TCP 地址：本机回环、端口由内核挑，几份同时起互不撞端口。
pub const NETWORK_ADDRESS_OF_A_SERVICE_IN_THIS_PROCESS: &str = "127.0.0.1:0";
/// 停服时等库关完的上限：接连接的线程收尾、连着的连接都关掉之后，提交线程与工作线程放手共享状态、库关上，正常在毫秒级。
pub const STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT: Duration = Duration::from_secs(30);
/// 停服时等库关完期间，每过这么久再关一遍连着的连接（刚接进来、还没登记的那几条登记之后才关得到）。
const STOP_OF_A_SERVICE_IN_THIS_PROCESS_POLL: Duration = Duration::from_millis(10);

/// 共享状态放手（库关完）时回话的一端：放在 [`SharedState`] 的最后一个字段，字段按声明的次序放手，回话时库已经关上。
enum ReleaseNotice {
    /// 服务进程：没人等（进程由 SIGTERM 结束）。
    Unwatched,
    /// 在调用方进程里起的那一份：停服句柄等这一声才删目录。
    Watched(Mutex<Sender<()>>),
}

impl Drop for ReleaseNotice {
    fn drop(&mut self) {
        match self {
            ReleaseNotice::Unwatched => {
                // 服务进程由 SIGTERM 结束，没人等这一声
            }
            ReleaseNotice::Watched(sender) => {
                // 停服句柄已经不等了（等过了上限）的，回话没人收，丢掉
                let _ = sender
                    .get_mut()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .send(());
            }
        }
    }
}

/// 在调用方这个进程里起的一份服务（测试自己起的临时服务：状态目录、库与 socket 都在调用方给的位置，协议与服务进程同一份）。
/// 停服（[`ServiceInThisProcess::stop`]，丢掉时同样停）：不再接连接、关掉连着的连接、删 socket，等提交线程与工作线程放手、库关上才回。
/// 与服务进程不同的只有三处：不装进程级的 panic 钩子（`PanicRecording::LeftToTheHostProcess`）、不打就绪行、有停服句柄。
/// 同步 WAL 失败时与服务进程一样退出整个进程（`storage.rs` 文件头「同步 WAL 失败之后」），这一份不另改。
pub struct ServiceInThisProcess {
    socket_path: PathBuf,
    bound_network_address: SocketAddr,
    stop_requested: Arc<AtomicBool>,
    accept_threads: Vec<thread::JoinHandle<AcceptLoopEnd>>,
    shared: Weak<SharedState>,
    released: Receiver<()>,
    is_stopped: bool,
}

/// 在这个进程里起一份服务：状态目录 `state_directory`（故障日志与库 `library_directory_in(state_directory)` 都在里面，没有就建）、
/// socket `socket_path`（全路径受 `configuration::MAXIMUM_SOCKET_PATH_BYTES` 限）、TCP 绑本机回环上内核挑的端口；不读环境、不读多机配置、
/// 不认弄坏开关。各项上限取命令行的缺省（`command_line` 的 `DEFAULT_*`），工作线程 [`WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS`] 个。
///
/// # Errors
/// 同 [`serve`] 起不来的几种（[`ServeFailure`]）：socket 路径太长、已有服务在那个 socket 上回话、库打不开、端口绑不上。
pub fn start_in_this_process(
    state_directory: &Path,
    socket_path: &Path,
) -> Result<ServiceInThisProcess, ServeFailure> {
    let settings = ServerSettings {
        socket_path: socket_path.to_path_buf(),
        network_address: NETWORK_ADDRESS_OF_A_SERVICE_IN_THIS_PROCESS.to_string(),
        state_directory: state_directory.to_path_buf(),
        library_directory: library_directory_in(state_directory),
        worker_threads: WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS,
        break_switches: BreakSwitches::none(),
        selftest_delay: None,
        selftest_sync_failure_file: None,
        admission: AdmissionLimits {
            maximum_pending_per_connection: DEFAULT_MAXIMUM_PENDING_PER_CONNECTION,
            maximum_pending_total: DEFAULT_MAXIMUM_PENDING_TOTAL,
            maximum_pending_bytes: DEFAULT_MAXIMUM_PENDING_BYTES,
            maximum_pending_bytes_per_connection: DEFAULT_MAXIMUM_PENDING_BYTES_PER_CONNECTION,
            maximum_connections: DEFAULT_MAXIMUM_CONNECTIONS,
            frame_read_timeout: Duration::from_millis(DEFAULT_FRAME_READ_TIMEOUT_MILLISECONDS),
            write_timeout: Duration::from_millis(DEFAULT_WRITE_TIMEOUT_MILLISECONDS),
            busy_retry: Duration::from_millis(DEFAULT_BUSY_RETRY_MILLISECONDS),
        },
        committer: CommitterSettings {
            group_window: Duration::from_millis(DEFAULT_GROUP_COMMIT_MILLISECONDS),
            group_maximum_frames: DEFAULT_GROUP_COMMIT_FRAMES,
            selftest_commit_delay: None,
            selftest_storage_failure_key: None,
            selftest_write_failure_file: None,
        },
    };
    let (release_sender, released) = mpsc::channel();
    let opened = open_the_service(
        settings,
        PanicRecording::LeftToTheHostProcess,
        ReleaseNotice::Watched(Mutex::new(release_sender)),
    )?;
    let bound_network_address = opened.bound_network_address;
    let shared = Arc::downgrade(&opened.shared);
    let stop_requested = Arc::new(AtomicBool::new(false));
    let accept_threads = start_accepting(opened, &stop_requested).into();
    Ok(ServiceInThisProcess {
        socket_path: socket_path.to_path_buf(),
        bound_network_address,
        stop_requested,
        accept_threads,
        shared,
        released,
        is_stopped: false,
    })
}

impl ServiceInThisProcess {
    /// 客户端连的 socket。
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// TCP 实际绑上的地址（本机回环与内核挑的端口）。
    #[must_use]
    pub fn bound_network_address(&self) -> SocketAddr {
        self.bound_network_address
    }

    /// 停服：不再接连接、删 socket、关掉连着的连接，等库关上才回。
    ///
    /// # Errors
    /// 接连接的线程 panic 了；等满 [`STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT`] 库还没关上（还有线程拿着共享状态）。
    pub fn stop(mut self) -> Result<(), String> {
        self.stop_and_wait_for_the_library_to_close()
    }

    fn stop_and_wait_for_the_library_to_close(&mut self) -> Result<(), String> {
        self.is_stopped = true;
        self.stop_requested.store(true, Ordering::SeqCst);
        // 接连接的线程堵在 accept 上：各连一下叫醒它，它看到停服标记就收尾（这一条连接不收）
        let _ = UnixStream::connect(&self.socket_path);
        let mut wake_address = self.bound_network_address;
        if wake_address.ip().is_unspecified() {
            wake_address.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
        }
        let _ = TcpStream::connect(wake_address);
        let mut problems = Vec::new();
        for accept_thread in self.accept_threads.drain(..) {
            match accept_thread.join() {
                Ok(AcceptLoopEnd::StopRequested) => {}
                Ok(AcceptLoopEnd::ListenerFailed(error)) => {
                    problems.push(format!("停服之前监听就停了：{error}"));
                }
                Err(_) => problems.push("接连接的线程 panic 了".to_string()),
            }
        }
        if let Err(error) = std::fs::remove_file(&self.socket_path) {
            if error.kind() != io::ErrorKind::NotFound {
                problems.push(format!(
                    "删不了 socket {}：{error}",
                    self.socket_path.display()
                ));
            }
        }
        let deadline = Instant::now() + STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT;
        // 迭代上界：STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT / STOP_OF_A_SERVICE_IN_THIS_PROCESS_POLL 轮；库关上（回话）或回话的一端已经放手就出
        loop {
            if let Some(shared) = self.shared.upgrade() {
                shared.connections.shut_down_every_connection();
            }
            match self
                .released
                .recv_timeout(STOP_OF_A_SERVICE_IN_THIS_PROCESS_POLL)
            {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if Instant::now() >= deadline {
                problems.push(format!(
                    "等了 {} 秒库还没关上：还有 {} 处拿着服务的共享状态（连接、提交线程或工作线程没收尾）",
                    STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT.as_secs(),
                    self.shared.strong_count()
                ));
                break;
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems.join("；"))
        }
    }
}

impl Drop for ServiceInThisProcess {
    fn drop(&mut self) {
        if self.is_stopped {
            return;
        }
        if let Err(problem) = self.stop_and_wait_for_the_library_to_close() {
            eprintln!(
                "  ✗ 停这个进程里起的 prune-server（{}）没停干净：{problem}",
                self.socket_path.display()
            );
        }
    }
}

/// 接连接的线程为什么收尾。
enum AcceptLoopEnd {
    /// 监听出错（`accept` 交回的错）。
    ListenerFailed(io::Error),
    /// 停服句柄叫停（[`ServiceInThisProcess::stop`]）。
    StopRequested,
}

/// 工作队列的通道本身不设容量：排进去的帧都过了准入，条数受待处理上限所限。
fn start_worker_pool(shared: &Arc<SharedState>, worker_threads: usize) -> Sender<Job> {
    let (job_sender, job_receiver) = mpsc::channel::<Job>();
    let job_receiver = Arc::new(Mutex::new(job_receiver));
    for _ in 0..worker_threads.max(1) {
        let shared = Arc::clone(shared);
        let job_receiver: Arc<Mutex<Receiver<Job>>> = Arc::clone(&job_receiver);
        thread::spawn(move || loop {
            let received = job_receiver
                .lock()
                .expect("工作队列锁只在 recv 时持有，持有时不会 panic")
                .recv();
            let Ok(job) = received else {
                return;
            };
            let Job {
                body,
                responder,
                mut slot,
                connection,
            } = job;
            let reserves_while_building =
                !shared.is_enabled(BreakSwitch::ResponseBuiltBeforeBudget);
            let (mut response, reserved_while_building) = {
                let mut budget =
                    ResponseBudget::new(&mut slot, &shared.counters, reserves_while_building);
                let response = process_frame(&shared, &body, &connection, &mut budget);
                (response, budget.reserved_bytes)
            };
            // 回应记进额度才交出去（文件头「额度」）：边编边记过的只再记余下的帧头那几个字节；记不进就换成忙、已经记下的放回，
            // 这一帧做过、回应没留，重发即可（读、扫、统计都不改库）。
            if !slot.reserve_response(response.len().saturating_sub(reserved_while_building)) {
                slot.release_response(reserved_while_building);
                let (sequence, operation_code) = peek_request_header(&body).unwrap_or_default();
                let reason = shared
                    .admission
                    .describe_bytes_refusal(&slot.budget, response.len());
                shared
                    .counters
                    .busy_rejections
                    .fetch_add(1, Ordering::Relaxed);
                let retry_after = shared.retry_after_milliseconds();
                response = encode_busy(
                    sequence,
                    operation_code,
                    retry_after,
                    &format!("忙：回应{reason}；这一帧做过、回应没留，{retry_after} 毫秒后再来"),
                );
            }
            drop(body);
            // 连接已经断了的，回应没人收，丢掉；名额随回应写出去或丢掉时放手。
            let _ = responder.send_response(response, slot);
        });
    }
    job_sender
}

fn accept_unix_connections(
    listener: UnixListener,
    shared: &Arc<SharedState>,
    job_sender: &Dispatch,
    stop_requested: &AtomicBool,
) -> AcceptLoopEnd {
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                // 停服句柄叫停之后接到的（多半是叫醒这个线程的那一条）不收：放手 stream、线程收尾
                if stop_requested.load(Ordering::SeqCst) {
                    return AcceptLoopEnd::StopRequested;
                }
                // 到了连接数上限的当场关掉（放手 stream）。
                let Some(open_connection) = shared.open_connection() else {
                    continue;
                };
                let shared = Arc::clone(shared);
                let job_sender = job_sender.clone();
                let origin = ConnectionOrigin {
                    transport: Transport::UnixSocket,
                    peer_address: String::new(),
                };
                thread::spawn(move || {
                    handle_connection(stream, &origin, &shared, &job_sender);
                    drop(open_connection);
                });
            }
            Err(error) => return AcceptLoopEnd::ListenerFailed(error),
        }
    }
}

fn accept_network_connections(
    listener: TcpListener,
    shared: &Arc<SharedState>,
    job_sender: &Dispatch,
    stop_requested: &AtomicBool,
) -> AcceptLoopEnd {
    loop {
        match listener.accept() {
            Ok((stream, peer)) => {
                // 停服句柄叫停之后接到的（多半是叫醒这个线程的那一条）不收：放手 stream、线程收尾
                if stop_requested.load(Ordering::SeqCst) {
                    return AcceptLoopEnd::StopRequested;
                }
                // 到了连接数上限的当场关掉（放手 stream）。
                let Some(open_connection) = shared.open_connection() else {
                    continue;
                };
                // 小帧（小块一块一帧）不等 Nagle 攒包；设不上只影响延迟，不影响对错。
                let _ = stream.set_nodelay(true);
                // 半开的连接由内核认出来关掉；设不上时这条连接要等对端自己断，照样服务。
                if let Err(message) = enable_keepalive(&stream, KeepaliveSettings::default()) {
                    eprintln!("  ✗ 给接进来的 TCP 连接开 keepalive 失败：{message}");
                }
                let shared = Arc::clone(shared);
                let job_sender = job_sender.clone();
                let origin = ConnectionOrigin {
                    transport: Transport::Network,
                    peer_address: peer.to_string(),
                };
                thread::spawn(move || {
                    handle_connection(stream, &origin, &shared, &job_sender);
                    drop(open_connection);
                });
            }
            Err(error) => return AcceptLoopEnd::ListenerFailed(error),
        }
    }
}

/// 读一帧请求的结局。
enum RequestFrameRead {
    /// 收齐的帧体与记着它字节的那一笔额度（弄坏开关 frame-length-unchecked、frame-body-read-before-admission 下还没记，是 `None`）。
    Body {
        body: Vec<u8>,
        reservation: Option<ByteReservation>,
    },
    /// 收帧额度不够：帧体读完丢掉了（帧边界保住），调用方回忙。
    Discarded {
        sequence: u64,
        operation_code: u8,
        refusal: String,
    },
    /// 在帧边界上读到流尾，或读出错：连接收尾。
    Closed,
    /// 一帧开了头、到期限还没收齐（文件头「期限」）：连接收尾。
    TimedOut,
    /// 声称的帧体长度不在 [帧头, 上限] 之内。
    LengthRejected(u32),
}

/// 读帧停在哪：流尾或读出错、到了期限。
enum FrameStop {
    Closed,
    TimedOut,
}

impl From<FrameStop> for RequestFrameRead {
    fn from(stop: FrameStop) -> RequestFrameRead {
        match stop {
            FrameStop::Closed => RequestFrameRead::Closed,
            FrameStop::TimedOut => RequestFrameRead::TimedOut,
        }
    }
}

/// 一条连接的读端：按长度字段切帧，一帧开了头之后在期限里收齐（文件头「期限」）。
struct FrameReader<Stream: ConnectionStream> {
    reader: BufReader<Stream>,
    frame_read_timeout: Duration,
    /// 套接字此刻设着读超时（帧开头之前的空闲不设）。
    has_read_timeout: bool,
    /// 收帧额度不够时帧体读进这里丢掉；头一次要用时才分配。
    discard_buffer: Vec<u8>,
}

impl<Stream: ConnectionStream> FrameReader<Stream> {
    fn new(stream: Stream, frame_read_timeout: Duration) -> FrameReader<Stream> {
        FrameReader {
            reader: BufReader::with_capacity(STREAM_BUFFER_BYTES, stream),
            frame_read_timeout,
            has_read_timeout: false,
            discard_buffer: Vec::new(),
        }
    }

    /// 读一些进 `destination`（至少 1 字节）；`deadline` 为 `None` 时一直等（帧开头之前的空闲）。
    fn read_some(
        &mut self,
        destination: &mut [u8],
        deadline: Option<Instant>,
    ) -> Result<usize, FrameStop> {
        loop {
            let timeout = match deadline {
                None => None,
                Some(frame_deadline) => {
                    let remaining = frame_deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return Err(FrameStop::TimedOut);
                    }
                    Some(remaining.max(Duration::from_millis(1)))
                }
            };
            if timeout.is_some() || self.has_read_timeout {
                if self
                    .reader
                    .get_ref()
                    .set_read_deadline_timeout(timeout)
                    .is_err()
                {
                    return Err(FrameStop::Closed);
                }
                self.has_read_timeout = timeout.is_some();
            }
            match self.reader.read(destination) {
                Ok(0) => return Err(FrameStop::Closed),
                Ok(read_now) => return Ok(read_now),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err(FrameStop::Closed),
            }
        }
    }

    fn read_exact_before(
        &mut self,
        destination: &mut [u8],
        deadline: Instant,
    ) -> Result<(), FrameStop> {
        let mut filled = 0;
        while filled < destination.len() {
            filled += self.read_some(&mut destination[filled..], Some(deadline))?;
        }
        Ok(())
    }

    /// 读完 `byte_count` 字节丢掉（收帧额度不够时保住帧边界）。
    fn discard_before(&mut self, byte_count: usize, deadline: Instant) -> Result<(), FrameStop> {
        if self.discard_buffer.is_empty() {
            self.discard_buffer = vec![0_u8; DISCARD_CHUNK_BYTES];
        }
        let mut discard_buffer = std::mem::take(&mut self.discard_buffer);
        let mut left = byte_count;
        let mut outcome = Ok(());
        while left > 0 {
            let chunk = left.min(discard_buffer.len());
            match self.read_some(&mut discard_buffer[..chunk], Some(deadline)) {
                Ok(read_now) => left -= read_now,
                Err(stop) => {
                    outcome = Err(stop);
                    break;
                }
            }
        }
        self.discard_buffer = discard_buffer;
        outcome
    }

    /// 读一帧请求：长度字段的头一个字节之前不限时，之后整帧在 `frame_read_timeout` 里收齐；帧体分配之前先记进额度，
    /// 记不进就读完丢掉、回 `Discarded`（文件头「额度」）。
    fn read_request_frame(
        &mut self,
        shared: &SharedState,
        budget: &Arc<ConnectionBudget>,
    ) -> RequestFrameRead {
        let mut length_field = [0_u8; FRAME_LENGTH_FIELD_BYTES];
        if let Err(stop) = self.read_some(&mut length_field[..1], None) {
            return stop.into();
        }
        let deadline = Instant::now() + self.frame_read_timeout;
        if let Err(stop) = self.read_exact_before(&mut length_field[1..], deadline) {
            return stop.into();
        }
        let declared_body_bytes = u32::from_le_bytes(length_field);
        if shared.is_enabled(BreakSwitch::FrameLengthUnchecked) {
            if self
                .reader
                .get_ref()
                .set_read_deadline_timeout(None)
                .is_err()
            {
                return RequestFrameRead::Closed;
            }
            self.has_read_timeout = false;
            let mut body = Vec::new();
            return match (&mut self.reader)
                .take(u64::from(declared_body_bytes))
                .read_to_end(&mut body)
            {
                Ok(_) => RequestFrameRead::Body {
                    body,
                    reservation: None,
                },
                Err(_) => RequestFrameRead::Closed,
            };
        }
        if !is_frame_length_acceptable(declared_body_bytes, REQUEST_HEADER_BYTES) {
            return RequestFrameRead::LengthRejected(declared_body_bytes);
        }
        let body_bytes = usize_from_u32(declared_body_bytes);
        let reservation = if shared.is_enabled(BreakSwitch::FrameBodyReadBeforeAdmission) {
            None
        } else {
            match shared.admission.reserve_bytes(budget, body_bytes) {
                Some(reservation) => Some(reservation),
                None => {
                    let refusal = shared.admission.describe_bytes_refusal(budget, body_bytes);
                    return self.discard_refused_frame(body_bytes, deadline, refusal);
                }
            }
        };
        let receiving = ReceivingBody::start(&shared.admission, body_bytes);
        let mut body = vec![0_u8; body_bytes];
        let received = self.read_exact_before(&mut body, deadline);
        drop(receiving);
        match received {
            Ok(()) => RequestFrameRead::Body { body, reservation },
            Err(stop) => stop.into(),
        }
    }

    /// 额度记不进的帧：读出帧头拿序号与操作码，帧体余下的读完丢掉。
    fn discard_refused_frame(
        &mut self,
        body_bytes: usize,
        deadline: Instant,
        refusal: String,
    ) -> RequestFrameRead {
        let mut header = vec![0_u8; usize_from_u32(REQUEST_HEADER_BYTES)];
        if let Err(stop) = self.read_exact_before(&mut header, deadline) {
            return stop.into();
        }
        if let Err(stop) = self.discard_before(body_bytes - header.len(), deadline) {
            return stop.into();
        }
        let (sequence, operation_code) = peek_request_header(&header).unwrap_or_default();
        RequestFrameRead::Discarded {
            sequence,
            operation_code,
            refusal,
        }
    }
}

/// 读线程当场做的几种（报到、列客户端、宣布停服务）。
fn handle_reader_request(
    body: &[u8],
    record: &ConnectionRecord,
    responder: &Responder,
    shared: &SharedState,
) {
    let response = match decode_request_body(body, TrailingBytes::Rejected, ServiceKeys::Refused) {
        Err((
            sequence,
            RequestRejection::MalformedPayload {
                operation_code,
                reason,
            },
        )) => Some(encode_rejection(
            sequence,
            ResponseStatus::MalformedPayload,
            operation_code,
            &reason,
        )),
        Err((sequence, RequestRejection::UnknownOperation { operation_code })) => {
            Some(encode_rejection(
                sequence,
                ResponseStatus::UnknownOperation,
                operation_code,
                "读线程收到认不得的操作码；服务的分派有 bug",
            ))
        }
        Ok(DecodedRequest { sequence, view }) => match view {
            RequestView::ConnectionAnnouncement(announcement) => {
                if !shared
                    .break_switches
                    .is_enabled(BreakSwitch::AnnouncementIgnored)
                {
                    record.record_announcement(announcement);
                }
                None
            }
            RequestView::ConnectedClients | RequestView::AnnounceStop
                if record.transport != Transport::UnixSocket =>
            {
                Some(encode_rejection(
                    sequence,
                    ResponseStatus::MalformedPayload,
                    peek_request_header(body).map_or(0, |(_, operation_code)| operation_code),
                    "列连着的客户端、宣布停服务只认 unix socket",
                ))
            }
            RequestView::ConnectedClients => Some(encode_connected_clients_response(
                sequence,
                OperationCode::ConnectedClients,
                &shared.connections.clients_except(record.connection),
            )),
            RequestView::AnnounceStop => {
                shared.is_stopping.store(true, Ordering::SeqCst);
                let clients = shared.connections.clients_except(record.connection);
                if !shared
                    .break_switches
                    .is_enabled(BreakSwitch::StopNoticeNotSent)
                {
                    // 每条连接的「服务停了」都写进套接字才回宣布的一方：控制命令拿到回话就 SIGTERM，排在队里没写的会丢
                    let (_, undelivered) = shared.connections.send_to_all_except_and_wait_flushed(
                        record.connection,
                        &encode_server_stopping(SERVER_STOPPING_MESSAGE),
                        Instant::now() + STOP_NOTICE_FLUSH_TIMEOUT,
                    );
                    if undelivered > 0 {
                        shared.fault_log.record(
                            FaultKind::StopNoticeUndelivered,
                            Some(record),
                            &format!("宣布停服务：{undelivered} 条连接的「服务停了」限时 {} 秒没写进套接字", STOP_NOTICE_FLUSH_TIMEOUT.as_secs()),
                        );
                    }
                }
                Some(encode_connected_clients_response(
                    sequence,
                    OperationCode::AnnounceStop,
                    &clients,
                ))
            }
            RequestView::BatchedPointRead(_)
            | RequestView::BatchedWrite { .. }
            | RequestView::PrefixScan { .. }
            | RequestView::Statistics
            | RequestView::Claim { .. }
            | RequestView::DeletePrefix { .. } => {
                shared.fault_log.record(
                    FaultKind::DispatchBug,
                    Some(record),
                    "读线程收到了该交工作线程或提交线程的帧",
                );
                Some(encode_rejection(
                    sequence,
                    ResponseStatus::MalformedPayload,
                    peek_request_header(body).map_or(0, |(_, operation_code)| operation_code),
                    "读线程只做报到、列客户端、宣布停服务；服务的分派有 bug",
                ))
            }
        },
    };
    if let Some(response) = response {
        // 连接正在收尾的，回应没人收，丢掉。
        let _ = responder.send(response);
    }
}

fn handle_connection<Stream: ConnectionStream>(
    stream: Stream,
    origin: &ConnectionOrigin,
    shared: &Arc<SharedState>,
    job_sender: &Dispatch,
) {
    let budget = Arc::new(ConnectionBudget::default());
    let (Ok(writer_stream), Ok(shutdown_stream)) =
        (stream.clone_for_writing(), stream.clone_for_writing())
    else {
        stream.shut_down();
        return;
    };
    let limits = shared.admission.limits;
    if writer_stream
        .set_write_deadline_timeout(Some(limits.write_timeout))
        .is_err()
    {
        stream.shut_down();
        return;
    }
    let (response_sender, response_receiver) = Responder::channel(
        Arc::clone(&shared.outbound_totals),
        Box::new(move || shutdown_stream.shut_down()),
        shared.is_enabled(BreakSwitch::ResponseSlotReleasedOnEnqueue),
    );
    let writer_shared = Arc::clone(shared);
    let writer =
        thread::spawn(move || write_responses(writer_stream, &response_receiver, &writer_shared));
    let record = shared.connections.register(
        origin.transport,
        origin.peer_address.clone(),
        response_sender.clone(),
    );
    let mut frames = FrameReader::new(stream, limits.frame_read_timeout);
    loop {
        let (body, reservation) = match frames.read_request_frame(shared, &budget) {
            RequestFrameRead::Body { body, reservation } => (body, reservation),
            RequestFrameRead::Closed => break,
            RequestFrameRead::TimedOut => {
                shared
                    .counters
                    .frame_read_timeouts
                    .fetch_add(1, Ordering::Relaxed);
                break;
            }
            RequestFrameRead::Discarded {
                sequence,
                operation_code,
                refusal,
            } => {
                let reason = format!("收这一帧：{refusal}；帧体读完丢掉了、没做");
                if !shared.reply_busy(&response_sender, sequence, operation_code, &reason) {
                    break;
                }
                continue;
            }
            RequestFrameRead::LengthRejected(declared_body_bytes) => {
                let message = format!(
                    "帧体长度 {declared_body_bytes} 不在 [{REQUEST_HEADER_BYTES}, {MAXIMUM_FRAME_BODY_BYTES}] 之内；帧边界找不回来，这条连接关掉"
                );
                let _ = response_sender.send(encode_rejection(
                    CONNECTION_LEVEL_SEQUENCE,
                    ResponseStatus::FrameLengthRejected,
                    0,
                    &message,
                ));
                break;
            }
        };
        let Some((sequence, operation_code)) = peek_request_header(&body) else {
            break;
        };
        let operation = OperationCode::from_code(operation_code);
        // 宣布停服务之后只认报到（连上就发，记不记都不碍事）与再宣布一次（控制命令可以再停一次），别的帧只回「服务停了」。
        let is_allowed_while_stopping = matches!(
            operation,
            Some(OperationCode::AnnounceStop | OperationCode::ConnectionAnnouncement)
        );
        if !is_allowed_while_stopping && shared.is_stopping.load(Ordering::SeqCst) {
            if !shared
                .break_switches
                .is_enabled(BreakSwitch::StopNoticeNotSent)
            {
                let _ = response_sender.send(encode_server_stopping(SERVER_STOPPING_MESSAGE));
            }
            break;
        }
        if operation.is_some_and(OperationCode::is_handled_by_connection_reader) {
            handle_reader_request(&body, &record, &response_sender, shared);
            continue;
        }
        let reservation = match reservation {
            Some(reservation) => reservation,
            None => match shared.admission.reserve_bytes(&budget, body.len()) {
                Some(reserved_after_reading) => reserved_after_reading,
                None => {
                    let reason = shared.admission.describe_bytes_refusal(&budget, body.len());
                    if !shared.reply_busy(&response_sender, sequence, operation_code, &reason) {
                        break;
                    }
                    continue;
                }
            },
        };
        let slot = match shared.admission.admit(&budget, reservation) {
            Ok(slot) => slot,
            Err(reason) => {
                if !shared.reply_busy(&response_sender, sequence, operation_code, &reason) {
                    break;
                }
                continue;
            }
        };
        let changes_library =
            OperationCode::from_code(operation_code).is_some_and(OperationCode::changes_library);
        let handed_over = if changes_library {
            job_sender
                .committer
                .send(CommitJob {
                    body,
                    responder: response_sender.clone(),
                    slot,
                    connection: Arc::clone(&record),
                })
                .is_ok()
        } else {
            job_sender
                .workers
                .send(Job {
                    body,
                    responder: response_sender.clone(),
                    slot,
                    connection: Arc::clone(&record),
                })
                .is_ok()
        };
        if !handed_over {
            break;
        }
    }
    shared.connections.unregister(record.connection);
    drop(response_sender);
    // 写线程等这条连接上还在处理的帧都回完（它们各持一份 responder）才退出。
    let _ = writer.join();
    frames.reader.get_ref().shut_down();
}

/// 一条连接的写线程：帧攒一批写、队空了 flush；碰到落盘回执先 flush 再回话（宣布停服务等「服务停了」真写进套接字）。
/// 写一次堵住到写期限（对端不收）就收尾：关掉连接两个方向（读线程随即读到流尾），排着的帧随写出队放手、名额随之放手。
fn write_responses<Stream: ConnectionStream>(
    stream: Stream,
    responses: &Receiver<Outgoing>,
    shared: &SharedState,
) {
    let mut writer = BufWriter::with_capacity(STREAM_BUFFER_BYTES, stream);
    if let Err(error) = write_until_senders_gone(&mut writer, responses) {
        if matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        ) {
            shared
                .counters
                .write_timeouts
                .fetch_add(1, Ordering::Relaxed);
        }
        writer.get_ref().shut_down();
    }
}

fn write_until_senders_gone<Stream: Write>(
    writer: &mut BufWriter<Stream>,
    responses: &Receiver<Outgoing>,
) -> io::Result<()> {
    while let Ok(first) = responses.recv() {
        let mut next = Some(first);
        while let Some(outgoing) = next.take() {
            match outgoing {
                Outgoing::Frame(frame) => {
                    writer.write_all(&frame.bytes)?;
                    // 写进去了：这一帧的账与名额放手。
                    drop(frame);
                }
                Outgoing::FlushedReceipt(receipt) => {
                    writer.flush()?;
                    let _ = receipt.send(());
                }
            }
            next = responses.try_recv().ok();
        }
        writer.flush()?;
    }
    Ok(())
}

fn process_frame(
    shared: &SharedState,
    body: &[u8],
    connection: &ConnectionRecord,
    budget: &mut ResponseBudget<'_>,
) -> Vec<u8> {
    let trailing_bytes = if shared
        .break_switches
        .is_enabled(BreakSwitch::TrailingBytesAccepted)
    {
        TrailingBytes::Accepted
    } else {
        TrailingBytes::Rejected
    };
    let service_keys = if shared
        .break_switches
        .is_enabled(BreakSwitch::ServiceKeysOpenToClients)
    {
        ServiceKeys::Accepted
    } else {
        ServiceKeys::Refused
    };
    match decode_request_body(body, trailing_bytes, service_keys) {
        Err((sequence, RequestRejection::UnknownOperation { operation_code })) => {
            if shared
                .break_switches
                .is_enabled(BreakSwitch::UnknownOperationAccepted)
            {
                return begin_response(sequence, ResponseStatus::Completed, operation_code, 0)
                    .finish();
            }
            let message = format!("认不得的操作码 {operation_code}；认得的是 1 成批点读、2 成批写、3 按前缀扫、4 统计、5 认领");
            encode_rejection(
                sequence,
                ResponseStatus::UnknownOperation,
                operation_code,
                &message,
            )
        }
        Err((
            sequence,
            RequestRejection::MalformedPayload {
                operation_code,
                reason,
            },
        )) => encode_rejection(
            sequence,
            ResponseStatus::MalformedPayload,
            operation_code,
            &reason,
        ),
        Ok(decoded) => {
            if let Some(delay) = shared.selftest_delay {
                if delay.sequence == decoded.sequence {
                    thread::sleep(delay.delay);
                }
            }
            shared
                .counters
                .frames_handled
                .fetch_add(1, Ordering::Relaxed);
            execute_request(shared, decoded, connection, budget)
        }
    }
}

fn u32_count(count: usize) -> u32 {
    u32::try_from(count).expect("个数来自请求里的 u32 字段，或受帧长上限所限，放得进 u32")
}

fn decoded_operation_code(view: &RequestView<'_>) -> u8 {
    match view {
        RequestView::BatchedPointRead(_) => OperationCode::BatchedPointRead.code(),
        RequestView::BatchedWrite { .. } => OperationCode::BatchedWrite.code(),
        RequestView::PrefixScan { .. } => OperationCode::PrefixScan.code(),
        RequestView::Statistics => OperationCode::Statistics.code(),
        RequestView::Claim { .. } => OperationCode::Claim.code(),
        RequestView::ConnectionAnnouncement(_) => OperationCode::ConnectionAnnouncement.code(),
        RequestView::ConnectedClients => OperationCode::ConnectedClients.code(),
        RequestView::AnnounceStop => OperationCode::AnnounceStop.code(),
        RequestView::DeletePrefix { .. } => OperationCode::DeletePrefix.code(),
    }
}

/// 读库出错：记一行故障、回存储出错。
fn storage_read_failed(
    shared: &SharedState,
    connection: &ConnectionRecord,
    sequence: u64,
    operation_code: u8,
    message: &str,
) -> Vec<u8> {
    shared
        .fault_log
        .record(FaultKind::StorageReadFailed, Some(connection), message);
    encode_rejection(
        sequence,
        ResponseStatus::StorageFailed,
        operation_code,
        message,
    )
}

fn execute_request(
    shared: &SharedState,
    decoded: DecodedRequest<'_>,
    connection: &ConnectionRecord,
    budget: &mut ResponseBudget<'_>,
) -> Vec<u8> {
    let sequence = decoded.sequence;
    match decoded.view {
        RequestView::BatchedPointRead(keys) => {
            let operation_code = OperationCode::BatchedPointRead.code();
            let values = match shared.library.read_points(&keys) {
                Ok(values) => values,
                Err(message) => {
                    return storage_read_failed(
                        shared,
                        connection,
                        sequence,
                        operation_code,
                        &message,
                    )
                }
            };
            let payload_bytes = point_read_response_payload_bytes(
                values
                    .iter()
                    .map(|value| value.as_ref().map(|slice| slice.len())),
            );
            if point_read_response_exceeds_frame_limit(payload_bytes) {
                let message = format!("这一帧读回的值合计 {payload_bytes} 字节，回应会超过帧长上限 {MAXIMUM_FRAME_BODY_BYTES}；把这一帧的键拆成几帧");
                return encode_rejection(
                    sequence,
                    ResponseStatus::MalformedPayload,
                    operation_code,
                    &message,
                );
            }
            // 回应多大先数出来、记进额度再分配（文件头「额度」）：读回的值钉在块缓存里，记不进就回忙、不分配
            if !budget.take(payload_bytes) {
                return budget.busy_response(shared, sequence, operation_code, payload_bytes);
            }
            let mut encoder = begin_response(
                sequence,
                ResponseStatus::Completed,
                operation_code,
                payload_bytes,
            );
            encoder.put_u32(u32_count(values.len()));
            for value in &values {
                match value {
                    Some(slice) => {
                        encoder.put_u8(1);
                        encoder.put_length_prefixed(slice);
                    }
                    None => encoder.put_u8(0),
                }
            }
            encoder.finish()
        }
        RequestView::BatchedWrite { .. }
        | RequestView::Claim { .. }
        | RequestView::DeletePrefix { .. }
        | RequestView::ConnectionAnnouncement(_)
        | RequestView::ConnectedClients
        | RequestView::AnnounceStop => {
            let operation_code = decoded_operation_code(&decoded.view);
            let message = "工作线程只收成批点读、按前缀扫、统计（写、认领与删前缀走提交线程，报到这类走读线程）；服务的分派有 bug";
            shared
                .fault_log
                .record(FaultKind::DispatchBug, Some(connection), message);
            encode_rejection(
                sequence,
                ResponseStatus::MalformedPayload,
                operation_code,
                message,
            )
        }
        RequestView::PrefixScan {
            table: _,
            prefix,
            maximum_entries,
            after_key,
        } => {
            let operation_code = OperationCode::PrefixScan.code();
            let maximum_bytes =
                usize_from_u32(MAXIMUM_FRAME_BODY_BYTES - RESPONSE_HEADER_BYTES) - 5;
            // 回应边扫边编：每读出一条，回应的缓冲要多分配的那一截先记进额度（文件头「额度」），记得进才分配、放进回应；
            // 截断标记与条数占位，扫完再填
            let mut encoder =
                begin_response(sequence, ResponseStatus::Completed, operation_code, 5);
            encoder.put_u8(0);
            encoder.put_u32(0);
            let mut refused_growth_bytes = 0_usize;
            let scanned = shared.library.scan_prefix_into(
                prefix,
                after_key,
                maximum_entries,
                maximum_bytes,
                &mut |key, value| {
                    let growth = encoder.capacity_growth_for(8 + key.len() + value.len());
                    if !budget.take(growth) {
                        refused_growth_bytes = growth;
                        return false;
                    }
                    encoder.grow_capacity(growth);
                    encoder.put_length_prefixed(key);
                    encoder.put_length_prefixed(value);
                    true
                },
            );
            match scanned {
                Err(message) => {
                    budget.release();
                    storage_read_failed(shared, connection, sequence, operation_code, &message)
                }
                Ok(outcome) if outcome.is_refused_by_caller => {
                    drop(encoder);
                    budget.busy_response(shared, sequence, operation_code, refused_growth_bytes)
                }
                Ok(outcome) => {
                    encoder.set_prefix_scan_header(
                        outcome.is_truncated,
                        u32_count(outcome.entries_accepted),
                    );
                    encoder.finish()
                }
            }
        }
        RequestView::Statistics => {
            let operation_code = OperationCode::Statistics.code();
            let mut counters = shared.library.tickers();
            let own = &shared.counters;
            let (sync_count, sync_median, sync_ninety_ninth_percentile, sync_maximum) =
                shared.write_ahead_log_sync_timings.summary();
            let as_u64 = |count: usize| u64::try_from(count).unwrap_or(u64::MAX);
            counters.extend([
                (
                    FRAMES_HANDLED_COUNTER,
                    own.frames_handled.load(Ordering::Relaxed),
                ),
                (
                    BUSY_REJECTIONS_COUNTER,
                    own.busy_rejections.load(Ordering::Relaxed),
                ),
                (
                    DUPLICATE_REQUESTS_COUNTER,
                    own.duplicate_requests.load(Ordering::Relaxed),
                ),
                (
                    GROUP_COMMITS_COUNTER,
                    own.group_commits.load(Ordering::Relaxed),
                ),
                (
                    PENDING_PEAK_COUNTER,
                    as_u64(shared.admission.pending_peak.load(Ordering::SeqCst)),
                ),
                (
                    PENDING_LIMIT_COUNTER,
                    as_u64(shared.admission.limits.maximum_pending_total),
                ),
                (WRITE_AHEAD_LOG_SYNC_COUNT_COUNTER, sync_count),
                (WRITE_AHEAD_LOG_SYNC_MEDIAN_COUNTER, sync_median),
                (
                    WRITE_AHEAD_LOG_SYNC_NINETY_NINTH_PERCENTILE_COUNTER,
                    sync_ninety_ninth_percentile,
                ),
                (WRITE_AHEAD_LOG_SYNC_MAXIMUM_COUNTER, sync_maximum),
                (
                    PENDING_BYTES_COUNTER,
                    as_u64(shared.admission.pending_bytes.load(Ordering::SeqCst)),
                ),
                (
                    PENDING_BYTES_PEAK_COUNTER,
                    as_u64(shared.admission.pending_bytes_peak.load(Ordering::SeqCst)),
                ),
                (
                    RECEIVING_BODY_BYTES_PEAK_COUNTER,
                    as_u64(
                        shared
                            .admission
                            .receiving_body_bytes_peak
                            .load(Ordering::SeqCst),
                    ),
                ),
                (
                    WORKER_RESPONSE_BYTES_PEAK_COUNTER,
                    as_u64(own.worker_response_bytes_peak.load(Ordering::SeqCst)),
                ),
                (
                    OUTBOUND_QUEUED_BYTES_PEAK_COUNTER,
                    as_u64(
                        shared
                            .outbound_totals
                            .queued_bytes_peak
                            .load(Ordering::SeqCst),
                    ),
                ),
                (
                    CONNECTIONS_OPEN_COUNTER,
                    as_u64(own.connections_open.load(Ordering::SeqCst)),
                ),
                (
                    CONNECTIONS_REFUSED_COUNTER,
                    own.connections_refused.load(Ordering::Relaxed),
                ),
                (
                    FRAME_READ_TIMEOUTS_COUNTER,
                    own.frame_read_timeouts.load(Ordering::Relaxed),
                ),
                (
                    WRITE_TIMEOUTS_COUNTER,
                    own.write_timeouts.load(Ordering::Relaxed),
                ),
                (
                    OUTBOUND_OVERFLOWS_COUNTER,
                    shared
                        .outbound_totals
                        .overflow_closures
                        .load(Ordering::Relaxed),
                ),
            ]);
            let mut encoder = begin_response(
                sequence,
                ResponseStatus::Completed,
                operation_code,
                64 * counters.len(),
            );
            encoder.put_u32(u32_count(counters.len()));
            for (name, value) in counters {
                encoder.put_length_prefixed(name.as_bytes());
                encoder.put_u64(value);
            }
            encoder.finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{RouteAddress, SimpleClient};
    use crate::protocol::Table;

    /// 这条用例独占的临时目录（`${TMPDIR:-/tmp}` 下按进程号与用例名起名），丢掉时删。
    struct TemporaryDirectoryOfTheTest {
        path: PathBuf,
    }

    impl TemporaryDirectoryOfTheTest {
        fn create(test_name: &str) -> TemporaryDirectoryOfTheTest {
            let path = std::env::temp_dir()
                .join(format!("prune-server-{test_name}-{}", std::process::id()));
            std::fs::create_dir(&path).unwrap_or_else(|error| {
                panic!("建不了这条用例的临时目录 {}：{error}", path.display())
            });
            TemporaryDirectoryOfTheTest { path }
        }
    }

    impl Drop for TemporaryDirectoryOfTheTest {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    /// 在这个进程里起的一份：客户端经 socket 写得进、读得回；停服之后 socket 不在、连不上，库已经关上——同一个状态目录马上再起一份
    /// 开得了库（RocksDB 的 LOCK 放了）、读得回停服之前写的那一行。
    /// 判别力：停服不关连着的连接（`shut_down_every_connection` 什么都不做），客户端还连着，库等不到关上，`stop` 交回错误，这条红。
    #[test]
    fn a_service_started_in_this_process_serves_a_client_and_after_the_stop_releases_its_library_and_socket(
    ) {
        let directory = TemporaryDirectoryOfTheTest::create("in-process-stop");
        let state_directory = directory.path.join("state");
        let socket_path = directory.path.join("socket");
        let service = start_in_this_process(&state_directory, &socket_path)
            .unwrap_or_else(|failure| panic!("起不来：{failure:?}"));
        assert!(
            service.bound_network_address().ip().is_loopback(),
            "TCP 只绑本机回环：{}",
            service.bound_network_address()
        );
        let route = RouteAddress::UnixSocket(service.socket_path().to_path_buf());
        let mut client =
            SimpleClient::connect(&route, BreakSwitches::none()).expect("连得上这一份");
        client
            .write(&[(Table::Facts, b"key".to_vec(), b"value".to_vec())])
            .expect("写得进");
        assert_eq!(
            client
                .read(&[(Table::Facts, b"key".to_vec())])
                .expect("读得了"),
            vec![Some(b"value".to_vec())]
        );
        // 客户端还连着就停服：停服关掉连着的连接，不等客户端自己走
        service.stop().expect("停得干净：库关上了");
        assert!(!socket_path.exists(), "停服之后 socket 不在");
        assert!(UnixStream::connect(&socket_path).is_err(), "停服之后连不上");
        drop(client);
        let restarted =
            start_in_this_process(&state_directory, &socket_path).unwrap_or_else(|failure| {
                panic!("停服之后同一个状态目录再起一份，库应当开得了：{failure:?}")
            });
        let mut client_of_the_restarted =
            SimpleClient::connect(&route, BreakSwitches::none()).expect("连得上重起的那一份");
        assert_eq!(
            client_of_the_restarted
                .read(&[(Table::Facts, b"key".to_vec())])
                .expect("读得了"),
            vec![Some(b"value".to_vec())],
            "停服之前写的那一行在库里"
        );
        drop(client_of_the_restarted);
        restarted.stop().expect("重起的那一份同样停得干净");
    }
}
