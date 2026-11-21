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

mod admission_budgets;
mod connection_serving;
mod frame_reading;
mod request_execution;
mod service_lifecycle;

pub use self::admission_budgets::*;
use self::connection_serving::*;
use self::frame_reading::*;
use self::request_execution::*;
pub use self::service_lifecycle::*;

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
