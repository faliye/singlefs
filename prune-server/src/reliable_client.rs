//! 容错客户端：给 CPU 写入与 GPU 读写用的那一种。一条连接、一个线程用，发帧不等回应（有窗口），回应按序号对回。
//!
//! - 请求号一个不分第二次：追加日志记着分出去过的最大请求号，整理之后照留，重开之后接着往上分（`journal` 文件头）。
//! - 不丢：写帧带客户端身份与请求号，发出去之前记进本地追加日志（`journal`），确认到了才勾掉；连接断了重连之后，
//!   没确认的帧按原来的次序重发（序号换新的、请求号不变），服务按（身份，请求号）去重，重放是幂等的；
//!   客户端进程重起时从日志里读回没确认的帧，连上就重放。
//! - 网络波动：每帧有超时（`frame_timeout`），到点没回应当连接坏了，断开重连；发帧有写超时（`write_timeout`，对端
//!   这么久一个字节都不收就当连接坏了，不永远堵在写里）；TCP 开 keepalive 认半开的连接；
//!   重连用指数退避加抖动（起始间隔每次翻倍、封顶，再乘 0.5 到 1 之间的随机数），有次数与总时长上限（缺省 30 次、120 秒），
//!   到上限回 `ClientFailure::ServerUnreachable`（「连不上」），没确认的帧留在日志里，不悄悄丢。在途帧数有窗口，满了停发、先收回应。
//!   总时长上限管住这一轮里的每一步：每次建连（地址解析、连接、报到）只用这一轮剩下的时间、其间照 `stop_check` 停（`client.rs` 文件头）；
//!   一轮从要连的那一刻起、到收到头一个回应才算完，这一轮里发出的帧最晚等到总时长上限，连上了却一直没有回应（服务不读、不回）
//!   也在上限里回「连不上」，不等满一份帧超时。打开时连上了、没有要等回应的帧，这一轮当场算完（调用方隔很久才发头一帧不受它管）。
//! - 服务有意停了（`prune-server/control stop` 先宣布停服务，服务给每条连接发一帧「服务停了」）：收到就不再重试，
//!   回 `ClientFailure::ServerStopped`。服务没打招呼就断了（崩溃、网络断）照上一条重连，到上限回「连不上」。
//! - 防雪崩：服务回「忙、多少毫秒后再来」时，这一帧照那个毫秒数再加 0 到一半的随机抖动之后重发；同一帧忙到上限回
//!   `ClientFailure::BusyLimit`。读失败（连不上、忙到上限、被拒、服务停了）回错误给调用方，不当成「键不在」。
//! - 按前缀扫（`scan`）与点读走同一套：忙照上一条退避重发、忙到上限回 `BusyLimit`，连接断了照重连那一条重发、到上限回「连不上」，
//!   被拒照原样回；扫回来的行原样交（截没截断由调用方接着拆前缀），不当成「前缀下没有行」。
//! - 删前缀（`delete_prefix_frame`、`delete_prefix`）与按前缀扫同一套退避与重连；不进追加日志、不带请求号：删本来就幂等，重发只会让某一帧报的数少一批，
//!   库里的结果不变（`protocol` 文件头「删前缀」）。`delete_prefix` 一帧一帧接着来，直到服务说前缀下没有更多的键。
//! - 重复的回应（序号已经回过，例如网络把一帧送了两遍）不算错，记一笔、丢掉。
//! - 单条超限当场报：写、读、扫、认领的帧拼好之后先核帧体长度，超过 `MAXIMUM_FRAME_BODY_BYTES` 的当场回 `ClientFailure::Protocol`
//!   （说是哪一种帧、几条、帧体多大、上限多少），不记追加日志、不发、不重试（服务会拒整条连接，重发照样拒）。
//!   服务回「帧长度不对」（序号 0 的状态 3）同样回协议错、不重连；发帧出错时先读出还躺在接收缓冲里的回应，是「帧长度不对」就照这一条回。
//! - 回应越过帧长上限就拆键：成批点读的请求帧小、读回的值合计可以越过帧长上限，服务这时整帧拒（`MalformedPayload`，说明里写读回会有多少字节，
//!   协议不变）；`read` 把这一帧的键对半拆开各自再问、递归到单键，按原次序接起来（单键回应至多一段长加帧头，不会越过：长过一段的值分段存）。
//! - 大值分段：一行的值长过一段（一帧装不下）照 Python 客户端的分段格式存成清单加段（`segmented_values` 文件头；格式说明只在
//!   `scripts/prune-server/prune_server_client.py` 文件头「键空间」大值分段那一段）：写前调 `write_segments_of_large_values` 先写段、
//!   再写它交回的清单，读用 `read_assembling_segmented_values` 拼回整值、核 sha256，拼不回回 `ClientFailure::SegmentedValue`（退 3），不当成键不在。
//!
//! 调用方照 `ClientFailure::exit_code` 退出。退出码表（不与 task-server 已用的 77、78、231、233 这几个撞）：
//!   75  prune-server 不可用：服务有意停了（「服务停了」）、重连到上限（「连不上」）、同一帧忙到上限。
//!       任务很快结束；修好、重起服务之后由提交它的会话重交，没确认的写帧在追加日志里，重开时重放。
//!   3   别的错（服务拒了帧、协议错、追加日志读写出错、调用方要停）。
//!
//! 弄坏开关（只给自证用）逐项弄坏上面某一处，表在 `src/bin/prune_server.rs` 文件头。

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::client::{
    open_announced_stream, ClientStream, ConnectDeadline, PrefixDeletionFrame, RouteAddress,
};
use crate::connections::announcement_from_environment;
use crate::journal::{Journal, JournalOptions, JournalStatistics};
use crate::keepalive::{enable_keepalive, read_keepalive, KeepaliveSettings};
use crate::protocol::{
    begin_point_read_frame, begin_write_frame, decode_response_body, encode_claim_frame,
    encode_delete_prefix_frame, encode_prefix_scan_frame, encode_statistics_frame,
    is_frame_length_acceptable, replace_claim_lease, replace_request_sequence, ClaimOutcome,
    ClientAnnouncement, PointReadFrameBuilder, RejectionStatus, Response, ResponseOutcome,
    ScanEntry, StatisticsCounter, Table, WriteFrameBuilder, CONNECTION_LEVEL_SEQUENCE,
    FRAME_LENGTH_FIELD_BYTES, MAXIMUM_FRAME_BODY_BYTES, RESPONSE_HEADER_BYTES,
};
use crate::segmented_values::{
    assemble_rereading_the_original_key, frame_ranges, plan_segmented_write,
    segment_count_high_water, segment_count_keys_to_read_before_writing, segment_key,
    SegmentManifest, SegmentedValueBeingAssembled, SegmentedValueFailure, SegmentsReadByPoint,
    StoredSegmentsReader, SEGMENTED_FRAME_ENTRY_LIMIT, SEGMENT_READ_FRAME_BYTES,
    SEGMENT_READ_OVERHEAD_BYTES_PER_KEY,
};

/// 退出码：prune-server 不可用（服务停了、连不上、忙到上限）。表在文件头。
pub const PRUNE_SERVER_UNAVAILABLE_EXIT_CODE: i32 = 75;
/// 退出码：别的错。
pub const OTHER_CLIENT_FAILURE_EXIT_CODE: i32 = 3;
/// 追加日志的缺省文件名（放在调用方给的目录下）。
pub const DEFAULT_JOURNAL_FILE_NAME: &str = "prune-server-client.journal";
/// 读回应时一次最多读这么多字节进缓冲。
const READ_CHUNK_BYTES: usize = 1024 * 1024;
/// 没有到点的事要等时，一次收回应最多等这么久（再回头看停不停、超时没有）。
const POLL_SLICE: Duration = Duration::from_millis(50);
/// 连接坏了时，读出还躺在接收缓冲里的「服务停了」最多等多久。
const STOP_NOTICE_DRAIN_TIMEOUT: Duration = Duration::from_millis(200);
/// 成批写的帧里每条在键值之外另占的字节（表 1、键长 4、值长 4，`protocol` 文件头）。
const WRITE_BYTES_PER_ENTRY: usize = 9;
/// 成批点读的帧里每把键在键之外另占的字节（表 1、键长 4，`protocol` 文件头）。
const POINT_READ_BYTES_PER_KEY: usize = 5;

/// 拼好的一帧（含 4 字节长度字段）帧体超过帧长上限时回协议错（文件头「单条超限当场报」）；`what` 说是哪一种帧、几条。
fn oversize_frame_failure(frame: &[u8], what: &str) -> Option<ClientFailure> {
    let body_bytes = frame.len().saturating_sub(FRAME_LENGTH_FIELD_BYTES);
    let maximum = usize::try_from(MAXIMUM_FRAME_BODY_BYTES).expect("u32 放得进 usize");
    (body_bytes > maximum).then(|| {
        ClientFailure::Protocol(format!(
            "{what}：帧体 {body_bytes} 字节，超过帧长上限 {MAXIMUM_FRAME_BODY_BYTES} 字节；没记追加日志、没发，不重试（服务会拒整条连接，重发照样拒；把条目拆成几帧或缩短键）"
        ))
    })
}

/// 连接坏了的那一刻接收缓冲里还躺着的序号 0 的通知（`ReliableClient::connection_notice_left_unread`）。
enum ConnectionNotice {
    /// 服务停了（状态 6）。
    ServerStopped(String),
    /// 帧长度不对（状态 3）：服务拒了整条连接。
    FrameLengthRejected(String),
}

/// 调用方要停（压测器收了 SIGTERM、自证到了限时）时回真；重连的等待里每一小段看一次。
pub type StopCheck = Arc<dyn Fn() -> bool + Send + Sync>;

/// 一条要写的条目：哪张表、键、值。
pub type TableEntry = (Table, Vec<u8>, Vec<u8>);

/// 容错客户端此刻停在哪一步；压测器的进度行与卡住诊断从别的线程读它（`ActivityProbe`）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClientActivity {
    /// 不在客户端的阻塞调用里（调用方在算、或客户端在两步之间）。
    Outside,
    Connecting,
    BackingOff,
    WritingFrame,
    WaitingForResponses,
    /// 没有在等的帧，照调用方给的时长睡。
    Sleeping,
    /// 往追加日志里追加或整理（整理时整份重写、同步）。
    WritingJournal,
}

impl ClientActivity {
    const ALL: [ClientActivity; 7] = [
        ClientActivity::Outside,
        ClientActivity::Connecting,
        ClientActivity::BackingOff,
        ClientActivity::WritingFrame,
        ClientActivity::WaitingForResponses,
        ClientActivity::Sleeping,
        ClientActivity::WritingJournal,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ClientActivity::Outside => "outside-client",
            ClientActivity::Connecting => "connecting",
            ClientActivity::BackingOff => "backing-off",
            ClientActivity::WritingFrame => "writing-frame",
            ClientActivity::WaitingForResponses => "waiting-for-responses",
            ClientActivity::Sleeping => "sleeping",
            ClientActivity::WritingJournal => "writing-journal",
        }
    }

    fn code(self) -> u8 {
        match self {
            ClientActivity::Outside => 0,
            ClientActivity::Connecting => 1,
            ClientActivity::BackingOff => 2,
            ClientActivity::WritingFrame => 3,
            ClientActivity::WaitingForResponses => 4,
            ClientActivity::Sleeping => 5,
            ClientActivity::WritingJournal => 6,
        }
    }
}

struct ActivityCell {
    code: AtomicU8,
    /// 进入这一步的时刻，从 `origin` 起算的微秒数。
    entered_at_micros: AtomicU64,
    origin: Instant,
}

/// 别的线程看这个客户端停在哪一步、停了多久。
#[derive(Clone)]
pub struct ActivityProbe {
    cell: Arc<ActivityCell>,
}

impl ActivityProbe {
    fn new() -> ActivityProbe {
        ActivityProbe {
            cell: Arc::new(ActivityCell {
                code: AtomicU8::new(ClientActivity::Outside.code()),
                entered_at_micros: AtomicU64::new(0),
                origin: Instant::now(),
            }),
        }
    }

    /// 换到 `activity`；与此刻相同就不动，停了多久从头一次进入算。
    fn enter(&self, activity: ClientActivity) {
        let code = activity.code();
        if self.cell.code.swap(code, Ordering::SeqCst) != code {
            let micros = u64::try_from(self.cell.origin.elapsed().as_micros()).unwrap_or(u64::MAX);
            self.cell.entered_at_micros.store(micros, Ordering::SeqCst);
        }
    }

    /// 此刻停在哪一步、进入这一步多久了。
    pub fn snapshot(&self) -> (ClientActivity, Duration) {
        let code = self.cell.code.load(Ordering::SeqCst);
        let entered = self.cell.entered_at_micros.load(Ordering::SeqCst);
        let activity = ClientActivity::ALL
            .into_iter()
            .find(|candidate| candidate.code() == code)
            .unwrap_or(ClientActivity::Outside);
        let now = u64::try_from(self.cell.origin.elapsed().as_micros()).unwrap_or(u64::MAX);
        (activity, Duration::from_micros(now.saturating_sub(entered)))
    }
}

/// 追加日志放哪：给了就用给的，没给放在调用方给的目录下。
pub fn journal_path_for(explicit: Option<PathBuf>, caller_directory: &Path) -> PathBuf {
    explicit.unwrap_or_else(|| caller_directory.join(DEFAULT_JOURNAL_FILE_NAME))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReconnectPolicy {
    pub initial_delay: Duration,
    pub maximum_delay: Duration,
    pub maximum_attempts: u32,
    pub maximum_total: Duration,
}

impl Default for ReconnectPolicy {
    /// 起始 50 毫秒、封顶 5 秒、至多 30 次、一轮至多 120 秒：服务重起（systemd 拉起来）等得过，服务真停了两分钟内报错。
    fn default() -> ReconnectPolicy {
        ReconnectPolicy {
            initial_delay: Duration::from_millis(50),
            maximum_delay: Duration::from_secs(5),
            maximum_attempts: 30,
            maximum_total: Duration::from_secs(120),
        }
    }
}

#[derive(Clone)]
pub struct ReliableClientSettings {
    pub route: RouteAddress,
    pub journal_path: PathBuf,
    pub frame_timeout: Duration,
    pub write_timeout: Duration,
    pub announcement: ClientAnnouncement,
    pub maximum_in_flight_frames: usize,
    pub reconnect: ReconnectPolicy,
    pub maximum_busy_retries: u32,
    pub keepalive: KeepaliveSettings,
    pub journal_compaction_bytes: u64,
    pub break_switches: BreakSwitches,
    pub stop_check: StopCheck,
}

impl ReliableClientSettings {
    /// 缺省：每帧 120 秒超时（满块一帧在盘忙时同步要十几秒，量的：压测 2064 满块 p99 12918 毫秒）、写超时 30 秒
    /// （服务的读线程一直在收帧，满了回忙而不是停读，正常时写不会停这么久）、窗口 64 帧、同一帧忙 1000 次报错、日志过 64 MiB 整理；
    /// 报到身份取这个进程的环境。
    pub fn new(route: RouteAddress, journal_path: PathBuf) -> ReliableClientSettings {
        ReliableClientSettings {
            route,
            journal_path,
            frame_timeout: Duration::from_secs(120),
            write_timeout: Duration::from_secs(30),
            announcement: announcement_from_environment(&BreakSwitches::none()),
            maximum_in_flight_frames: 64,
            reconnect: ReconnectPolicy::default(),
            maximum_busy_retries: 1000,
            keepalive: KeepaliveSettings::default(),
            journal_compaction_bytes: 64 * 1024 * 1024,
            break_switches: BreakSwitches::none(),
            stop_check: Arc::new(|| false),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientFailure {
    /// 重连到了次数或总时长上限；没确认的写帧留在追加日志里。
    ServerUnreachable {
        attempts: u32,
        elapsed: Duration,
        last_error: String,
    },
    /// 同一帧被回忙到了上限。
    BusyLimit { retries: u32 },
    /// 调用方要停。
    Stopped,
    /// 服务有意停了（收到「服务停了」）；没确认的写帧留在追加日志里。
    ServerStopped { message: String },
    /// 服务拒了这一帧（内容解不开、存储出错这类）。
    Rejected {
        status: RejectionStatus,
        message: String,
    },
    /// 回应解不开、序号不对这类协议层的错。
    Protocol(String),
    /// 追加日志读写出错。
    Journal(String),
    /// 大值分段写不成、读不回（`segmented_values`）：原键是段键或计数键的形状、段数超了段号，计数键或清单解不开，段缺了、长度或 sha256 对不上。
    SegmentedValue(SegmentedValueFailure),
}

impl ClientFailure {
    /// 照文件头的退出码表。
    pub fn exit_code(&self) -> i32 {
        self.exit_code_with_switches(&BreakSwitches::none())
    }

    /// 同 `exit_code`；弄坏开关 unreachable-exit-code-generic 下「连不上」给 3（自证起的客户端子进程用）。
    pub fn exit_code_with_switches(&self, break_switches: &BreakSwitches) -> i32 {
        match self {
            ClientFailure::ServerUnreachable { .. }
                if break_switches.is_enabled(BreakSwitch::UnreachableExitCodeGeneric) =>
            {
                OTHER_CLIENT_FAILURE_EXIT_CODE
            }
            ClientFailure::ServerUnreachable { .. }
            | ClientFailure::BusyLimit { .. }
            | ClientFailure::ServerStopped { .. } => PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
            ClientFailure::Stopped
            | ClientFailure::Rejected { .. }
            | ClientFailure::Protocol(_)
            | ClientFailure::Journal(_)
            | ClientFailure::SegmentedValue(_) => OTHER_CLIENT_FAILURE_EXIT_CODE,
        }
    }
}

impl std::fmt::Display for ClientFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientFailure::ServerUnreachable {
                attempts,
                elapsed,
                last_error,
            } => write!(
                formatter,
                "连不上服务：重连 {attempts} 次、{:.1} 秒到了上限，最后一次的错：{last_error}；没确认的写帧留在追加日志里，下次打开时重放",
                elapsed.as_secs_f64()
            ),
            ClientFailure::BusyLimit { retries } => {
                write!(formatter, "服务一直忙：同一帧被回忙 {retries} 次，到了上限")
            }
            ClientFailure::Stopped => write!(formatter, "调用方要停"),
            ClientFailure::ServerStopped { message } => write!(
                formatter,
                "服务停了（有意停的，不再重试）：{message}；没确认的写帧留在追加日志里，服务重起之后重交任务时重放"
            ),
            ClientFailure::Rejected { status, message } => {
                write!(formatter, "服务拒了这一帧（{status:?}）：{message}")
            }
            ClientFailure::Protocol(message) => write!(formatter, "协议出错：{message}"),
            ClientFailure::Journal(message) => write!(formatter, "追加日志出错：{message}"),
            ClientFailure::SegmentedValue(failure) => write!(formatter, "大值分段：{failure}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct OperationIdentifier(pub u64);

/// 做完了的一帧。
#[derive(Debug)]
pub enum Completion {
    WriteAcknowledged {
        operation: OperationIdentifier,
        request_number: u64,
        entries_written: u32,
        is_already_applied: bool,
    },
    Values {
        operation: OperationIdentifier,
        values: Vec<Option<Vec<u8>>>,
    },
    Claims {
        operation: OperationIdentifier,
        outcomes: Vec<ClaimOutcome>,
    },
    Statistics {
        operation: OperationIdentifier,
        counters: Vec<StatisticsCounter>,
    },
    ScanEntries {
        operation: OperationIdentifier,
        is_truncated: bool,
        entries: Vec<ScanEntry>,
    },
    PrefixDeleted {
        operation: OperationIdentifier,
        keys_in_frame: u32,
        is_exhausted: bool,
        last_key: Vec<u8>,
    },
    /// 读、扫、认领、统计、删前缀没做成（被拒、忙到上限）；写没做成是整个客户端的错，从 `poll` 直接回。
    Failed {
        operation: OperationIdentifier,
        failure: ClientFailure,
    },
}

impl Completion {
    pub fn operation(&self) -> OperationIdentifier {
        match self {
            Completion::WriteAcknowledged { operation, .. }
            | Completion::Values { operation, .. }
            | Completion::Claims { operation, .. }
            | Completion::Statistics { operation, .. }
            | Completion::ScanEntries { operation, .. }
            | Completion::PrefixDeleted { operation, .. }
            | Completion::Failed { operation, .. } => *operation,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OperationKind {
    Write {
        request_number: u64,
    },
    Read,
    Scan,
    /// 认领：交给客户端那一刻与租约毫秒数，重发时只给剩下的（[`claim_lease_left`]）。
    Claim {
        lease_milliseconds: u32,
        submitted_at: Instant,
    },
    Statistics,
    /// 删前缀：不进追加日志（删本来就幂等，不去重，重发只会让某一帧报的数少一批，`protocol.rs` 文件头「删前缀」），断线重连之后照样重发。
    DeletePrefix,
}

/// 认领每次发出去（头一次、重连之后、回了忙之后）给的租约：原租约减去从交给客户端起过了的毫秒数，至少 1 毫秒。
/// 服务按收到的那一刻起算到期，重发照原租约给会把到期往后推；照剩下的给，到期不晚于头一次交出去时算的那一刻（客户端的钟）。
fn claim_lease_left(lease_milliseconds: u32, elapsed: Duration) -> u32 {
    let elapsed_milliseconds = u32::try_from(elapsed.as_millis()).unwrap_or(u32::MAX);
    lease_milliseconds
        .saturating_sub(elapsed_milliseconds)
        .max(1)
}

#[derive(Clone, Copy, Debug)]
enum OperationState {
    Unsent,
    OnWire {
        deadline: Instant,
    },
    Deferred {
        due: Instant,
        busy_received_at: Instant,
        retry_after: Duration,
    },
}

struct Operation {
    kind: OperationKind,
    frame: Vec<u8>,
    busy_retries: u32,
    state: OperationState,
}

/// 拼分段存的键时经服务读段、重读原键（[`StoredSegmentsReader`]）：`key_in_the_service` 是原键在服务里的全名。
struct SegmentReadsOverTheService<'client> {
    client: &'client mut ReliableClient,
    table: Table,
    key_in_the_service: &'client [u8],
}

impl StoredSegmentsReader for SegmentReadsOverTheService<'_> {
    type ReadFailure = ClientFailure;

    fn read_missing_segments(
        &mut self,
        being_assembled: &SegmentedValueBeingAssembled,
    ) -> Result<SegmentsReadByPoint, ClientFailure> {
        self.client
            .read_missing_segments(self.table, self.key_in_the_service, being_assembled)
    }

    fn reread_the_original_key(&mut self) -> Result<Option<Vec<u8>>, ClientFailure> {
        Ok(self
            .client
            .read_as_many_values_as_keys(&[(self.table, self.key_in_the_service.to_vec())])?
            .pop()
            .flatten())
    }

    fn wait(&mut self, delay: Duration) {
        thread::sleep(delay);
    }
}

/// 客户端自己数的，给自证与压测器看。
#[derive(Clone, Debug, Default)]
pub struct ClientDiagnostics {
    /// 连接坏了之后重新连上的次数。
    pub reconnections: u64,
    /// 每一轮连接（打开时那一轮也算）里每次尝试开始的时刻。
    pub connection_attempt_cycles: Vec<Vec<Instant>>,
    pub duplicate_responses: u64,
    pub busy_responses: u64,
    /// 回了忙的帧重发时：（实际等了多久，服务叫等多久）。
    pub busy_waits: Vec<(Duration, Duration)>,
    pub frame_timeouts: u64,
    /// 发帧撞上写超时（对端这么久一个字节都不收）的次数：写超时只兜底，对端关了连接应当当场读到流尾或写出错。
    pub write_timeouts: u64,
    /// 连接坏了之后重发的帧数。
    pub replayed_frames: u64,
    /// 打开时从追加日志里读回、没确认的写帧数。
    pub journal_recovered_writes: u64,
    /// 最近一次连接坏掉的原因。
    pub last_break_reason: Option<String>,
}

struct Connection {
    writer: ClientStream,
    reader: ClientStream,
    buffer: Vec<u8>,
}

/// 连一次、报到：每一步阻塞只用到 `deadline`（这一轮重连的总时长上限）剩下的时间，其间照 `stop_check` 停（`client.rs` 文件头）。
fn open_connection(
    settings: &ReliableClientSettings,
    deadline: Instant,
) -> Result<Connection, String> {
    let break_switches = &settings.break_switches;
    let stop_check = Arc::clone(&settings.stop_check);
    let limit = ConnectDeadline {
        deadline,
        stop_check: &*stop_check,
        is_unbounded: break_switches.is_enabled(BreakSwitch::ConnectWithoutDeadline),
    };
    let writer =
        open_announced_stream(&settings.route, &settings.announcement, &limit, |stream| {
            if break_switches.is_enabled(BreakSwitch::KeepaliveNotSet) {
                return Ok(());
            }
            enable_keepalive(stream, settings.keepalive)
        })?;
    if !break_switches.is_enabled(BreakSwitch::ClientWriteTimeoutUnset) {
        writer
            .set_write_timeout(Some(settings.write_timeout))
            .map_err(|error| format!("设不上写超时：{error}"))?;
    }
    let reader = writer
        .try_clone()
        .map_err(|error| format!("复制不了连接：{error}"))?;
    Ok(Connection {
        writer,
        reader,
        buffer: Vec::new(),
    })
}

/// 连接是怎么坏的。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BreakCause {
    /// 对端关了连接，或读写出错。
    PeerClosedOrFailed,
    /// 在途的帧到点没回应（半开、服务卡住）。
    FrameTimeout,
    /// 发帧撞上写超时（对端不收）。
    WriteTimeout,
}

/// 一轮重连：什么时候开始、每次尝试的时刻、最近一次没成的原因。
struct ReconnectCycle {
    started: Instant,
    attempt_times: Vec<Instant>,
    last_error: String,
}

pub struct ReliableClient {
    settings: ReliableClientSettings,
    journal: Journal,
    client_identity: Vec<u8>,
    connection: Option<Connection>,
    has_connected_before: bool,
    /// 还没收到回应的这一轮重连。
    reconnect_cycle: Option<ReconnectCycle>,
    operations: BTreeMap<OperationIdentifier, Operation>,
    wire: HashMap<u64, OperationIdentifier>,
    next_sequence: u64,
    next_operation: u64,
    next_request_number: u64,
    completions: VecDeque<Completion>,
    diagnostics: ClientDiagnostics,
    random_state: u64,
    scratch: Vec<u8>,
    activity: ActivityProbe,
}

impl ReliableClient {
    /// 打开追加日志、连上服务、重放日志里没确认的写帧。连不上回 `ServerUnreachable`，日志原样留着。
    pub fn open(settings: ReliableClientSettings) -> Result<ReliableClient, ClientFailure> {
        let recovered = Journal::open(
            &settings.journal_path,
            JournalOptions::from_break_switches(
                settings.journal_compaction_bytes,
                &settings.break_switches,
            ),
        )
        .map_err(ClientFailure::Journal)?;
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos())
            .to_le_bytes();
        let random_state = u64::from_le_bytes(seed[..8].try_into().expect("切的正好是 8 字节"))
            ^ u64::from(std::process::id()).rotate_left(32);
        let mut client = ReliableClient {
            settings,
            journal: recovered.journal,
            client_identity: recovered.client_identity,
            connection: None,
            has_connected_before: false,
            reconnect_cycle: None,
            operations: BTreeMap::new(),
            wire: HashMap::new(),
            next_sequence: 1,
            next_operation: 1,
            next_request_number: recovered.next_request_number,
            completions: VecDeque::new(),
            diagnostics: ClientDiagnostics::default(),
            random_state: random_state | 1,
            scratch: vec![0_u8; READ_CHUNK_BYTES],
            activity: ActivityProbe::new(),
        };
        for write in recovered.unacknowledged {
            client.register(
                OperationKind::Write {
                    request_number: write.request_number,
                },
                write.frame,
            );
            client.diagnostics.journal_recovered_writes += 1;
        }
        client.send_unsent()?;
        if client.operations.is_empty() {
            // 连上了、没有要等回应的帧：这一轮重连到此为止。留着的话调用方隔很久才发头一帧，那一帧的期限
            // （`reconnect_cycle_deadline`）早过了，会被当成连上却没回应。
            client.close_reconnect_cycle();
        }
        Ok(client)
    }

    pub fn client_identity(&self) -> &[u8] {
        &self.client_identity
    }

    pub fn diagnostics(&self) -> &ClientDiagnostics {
        &self.diagnostics
    }

    /// 给别的线程看这个客户端停在哪一步。
    pub fn activity_probe(&self) -> ActivityProbe {
        self.activity.clone()
    }

    /// 还没做完的帧数（在途、等重发、等重连的都算）。
    pub fn outstanding_operations(&self) -> usize {
        self.operations.len()
    }

    pub fn outstanding_writes(&self) -> usize {
        self.operations
            .values()
            .filter(|operation| matches!(operation.kind, OperationKind::Write { .. }))
            .count()
    }

    /// 此刻这条 TCP 连接在内核里的 keepalive 设置（unix socket 或没连着回 `Ok(None)`）。
    pub fn current_keepalive(&self) -> Result<Option<KeepaliveSettings>, String> {
        match self
            .connection
            .as_ref()
            .map(|connection| &connection.writer)
        {
            Some(ClientStream::Network(stream)) => read_keepalive(stream),
            Some(ClientStream::Unix(_)) | None => Ok(None),
        }
    }

    fn next_random_fraction(&mut self) -> f64 {
        // xorshift64*：只给抖动用，不要求密码学强度。
        let mut state = self.random_state;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        self.random_state = state;
        let mixed = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (mixed >> 11) as f64 / (1_u64 << 53) as f64
    }

    fn is_enabled(&self, switch: BreakSwitch) -> bool {
        self.settings.break_switches.is_enabled(switch)
    }

    fn register(&mut self, kind: OperationKind, frame: Vec<u8>) -> OperationIdentifier {
        let identifier = OperationIdentifier(self.next_operation);
        self.next_operation += 1;
        self.operations.insert(
            identifier,
            Operation {
                kind,
                frame,
                busy_retries: 0,
                state: OperationState::Unsent,
            },
        );
        identifier
    }

    /// 在途帧数到了窗口就先收回应，等窗口空出来。
    fn wait_for_window(&mut self) -> Result<(), ClientFailure> {
        while self.operations.len() >= self.settings.maximum_in_flight_frames {
            self.pump_once(POLL_SLICE)?;
        }
        Ok(())
    }

    /// 写一帧：记进追加日志、发出去，不等确认；回操作号与请求号。
    pub fn submit_write(
        &mut self,
        entry_count: u32,
        capacity_in_bytes: usize,
        fill: impl FnOnce(&mut WriteFrameBuilder),
    ) -> Result<(OperationIdentifier, u64), ClientFailure> {
        self.wait_for_window()?;
        let request_number = self.next_request_number;
        self.next_request_number += 1;
        let mut builder = begin_write_frame(
            0,
            &self.client_identity,
            request_number,
            entry_count,
            capacity_in_bytes,
        );
        fill(&mut builder);
        let frame = builder.finish();
        if let Some(failure) = oversize_frame_failure(
            &frame,
            &format!("成批写 {entry_count} 条（请求号 {request_number}）"),
        )
        .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            // 没记日志、没发：这个请求号收回，下一帧照用。
            self.next_request_number = request_number;
            return Err(failure);
        }
        self.activity.enter(ClientActivity::WritingJournal);
        let journaled = self.journal.append_write(request_number, &frame);
        self.activity.enter(ClientActivity::Outside);
        journaled.map_err(ClientFailure::Journal)?;
        let operation = self.register(OperationKind::Write { request_number }, frame);
        self.send_unsent()?;
        Ok((operation, request_number))
    }

    pub fn submit_write_entries(
        &mut self,
        entries: &[(Table, Vec<u8>, Vec<u8>)],
    ) -> Result<(OperationIdentifier, u64), ClientFailure> {
        let entry_count = u32::try_from(entries.len())
            .map_err(|_| ClientFailure::Protocol("一帧的条数超过 u32".to_string()))?;
        let capacity = entries
            .iter()
            .map(|(_, key, value)| WRITE_BYTES_PER_ENTRY + key.len() + value.len())
            .sum();
        self.submit_write(entry_count, capacity, |builder| {
            for (table, key, value) in entries {
                builder.entry(*table, key, value);
            }
        })
    }

    pub fn submit_read(
        &mut self,
        key_count: u32,
        capacity_in_bytes: usize,
        fill: impl FnOnce(&mut PointReadFrameBuilder),
    ) -> Result<OperationIdentifier, ClientFailure> {
        self.wait_for_window()?;
        let mut builder = begin_point_read_frame(0, key_count, capacity_in_bytes);
        fill(&mut builder);
        let frame = builder.finish();
        if let Some(failure) = oversize_frame_failure(&frame, &format!("成批点读 {key_count} 个键"))
            .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(OperationKind::Read, frame);
        self.send_unsent()?;
        Ok(operation)
    }

    pub fn submit_claim(
        &mut self,
        keys: &[(Table, Vec<u8>)],
        lease_milliseconds: u32,
    ) -> Result<OperationIdentifier, ClientFailure> {
        self.wait_for_window()?;
        let borrowed: Vec<(Table, &[u8])> = keys
            .iter()
            .map(|(table, key)| (*table, key.as_slice()))
            .collect();
        let frame = encode_claim_frame(0, &self.client_identity, lease_milliseconds, &borrowed);
        if let Some(failure) = oversize_frame_failure(&frame, &format!("认领 {} 个键", keys.len()))
            .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(
            OperationKind::Claim {
                lease_milliseconds,
                submitted_at: Instant::now(),
            },
            frame,
        );
        self.send_unsent()?;
        Ok(operation)
    }

    /// 收回应，至多等 `maximum_wait`；回这期间做完的帧。写失败与连不上从这里回错。
    pub fn poll(&mut self, maximum_wait: Duration) -> Result<Vec<Completion>, ClientFailure> {
        if self.completions.is_empty() {
            if self.operations.is_empty() {
                // 没有在等的帧：照调用方给的时长睡，调用方拿它当节拍。
                self.activity.enter(ClientActivity::Sleeping);
                thread::sleep(maximum_wait);
                self.activity.enter(ClientActivity::Outside);
            } else {
                self.pump_once(maximum_wait)?;
            }
        }
        Ok(self.completions.drain(..).collect())
    }

    /// 等某一帧做完；别的帧做完的留在队里给 `poll`。
    fn wait_for(&mut self, operation: OperationIdentifier) -> Result<Completion, ClientFailure> {
        loop {
            if let Some(position) = self
                .completions
                .iter()
                .position(|completion| completion.operation() == operation)
            {
                return Ok(self
                    .completions
                    .remove(position)
                    .expect("position 是上面找到的"));
            }
            self.pump_once(POLL_SLICE)?;
        }
    }

    /// 等全部写帧都确认；写确认从完成队里拿掉，别的留着。
    pub fn wait_all_acknowledged(&mut self) -> Result<(), ClientFailure> {
        while self.outstanding_writes() > 0 {
            self.pump_once(POLL_SLICE)?;
            self.completions
                .retain(|completion| !matches!(completion, Completion::WriteAcknowledged { .. }));
        }
        self.completions
            .retain(|completion| !matches!(completion, Completion::WriteAcknowledged { .. }));
        Ok(())
    }

    /// 成批点读，等回应：一帧问全部键，回应越过帧长上限时拆成几帧（文件头「回应越过帧长上限就拆键」）。读失败回错误，不当成键不在。
    pub fn read(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        let result = self.read_halving_on_an_oversized_answer(keys);
        match result {
            Err(_) if self.is_enabled(BreakSwitch::ReadFailureReportedAsAbsent) => {
                Ok(vec![None; keys.len()])
            }
            Ok(_) | Err(_) => result,
        }
    }

    /// 一帧问 `keys`；服务答回应越过帧长上限（整帧拒成 `MalformedPayload`）而键不止一个，拆成前后两半各自照这样问（递归，深度不超过键数的
    /// 二进制位数），按原次序接起来。回应多大事先不知道（不分段的值至多一段长，按这个上界切一帧只装得下三四个键），所以先整帧问、被拒再拆。
    /// 别的帧内容不对的拒也是 `MalformedPayload`，同样拆到单键，单键还被拒就把那一个拒照原样交回。
    fn read_halving_on_an_oversized_answer(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        match self.read_one_frame(keys) {
            Err(ClientFailure::Rejected {
                status: RejectionStatus::MalformedPayload,
                message: _,
            }) if keys.len() > 1 => {
                let (front_half, back_half) = keys.split_at(keys.len() / 2);
                let mut values = self.read_halving_on_an_oversized_answer(front_half)?;
                values.extend(self.read_halving_on_an_oversized_answer(back_half)?);
                Ok(values)
            }
            answered @ (Ok(_) | Err(_)) => answered,
        }
    }

    fn read_one_frame(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        let key_count = u32::try_from(keys.len())
            .map_err(|_| ClientFailure::Protocol("一帧的键数超过 u32".to_string()))?;
        let capacity = keys
            .iter()
            .map(|(_, key)| POINT_READ_BYTES_PER_KEY + key.len())
            .sum();
        let operation = self.submit_read(key_count, capacity, |builder| {
            for (table, key) in keys {
                builder.key(*table, key);
            }
        })?;
        match self.wait_for(operation)? {
            Completion::Values { values, .. } => Ok(values),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }) => Err(ClientFailure::Protocol(format!(
                "成批点读收到的不是值：{other:?}"
            ))),
        }
    }

    pub fn claim(
        &mut self,
        keys: &[(Table, Vec<u8>)],
        lease_milliseconds: u32,
    ) -> Result<Vec<ClaimOutcome>, ClientFailure> {
        let operation = self.submit_claim(keys, lease_milliseconds)?;
        match self.wait_for(operation)? {
            Completion::Claims { outcomes, .. } => Ok(outcomes),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }) => Err(ClientFailure::Protocol(format!(
                "认领收到的不是认领结局：{other:?}"
            ))),
        }
    }

    pub fn statistics(&mut self) -> Result<Vec<StatisticsCounter>, ClientFailure> {
        self.wait_for_window()?;
        let operation = self.register(OperationKind::Statistics, encode_statistics_frame(0));
        self.send_unsent()?;
        match self.wait_for(operation)? {
            Completion::Statistics { counters, .. } => Ok(counters),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::ScanEntries { .. }
            | Completion::PrefixDeleted { .. }) => Err(ClientFailure::Protocol(format!(
                "统计收到的不是计数：{other:?}"
            ))),
        }
    }

    /// 按前缀扫一帧，等回应：`table` 里键以 `prefix` 开头的行按键升序、至多 `maximum_entries` 行，交回截没截断与回来的行（删除记号、
    /// 分段存的清单与段照服务里存着的原样交）。忙、断线照文件头「按前缀扫」那一条，与点读同一套。
    ///
    /// # Errors
    /// 帧体超过帧长上限（[`ClientFailure::Protocol`]，没发）；连不上、忙到上限、服务停了（退 75）；被拒、回的不是扫回的行。
    pub fn scan(
        &mut self,
        table: Table,
        prefix: &[u8],
        maximum_entries: u32,
    ) -> Result<(bool, Vec<ScanEntry>), ClientFailure> {
        self.wait_for_window()?;
        let frame = encode_prefix_scan_frame(0, table, prefix, maximum_entries);
        if let Some(failure) =
            oversize_frame_failure(&frame, &format!("按前缀扫（前缀 {} 字节）", prefix.len()))
                .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(OperationKind::Scan, frame);
        self.send_unsent()?;
        match self.wait_for(operation)? {
            Completion::ScanEntries {
                is_truncated,
                entries,
                ..
            } => Ok((is_truncated, entries)),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::PrefixDeleted { .. }) => Err(ClientFailure::Protocol(format!(
                "按前缀扫收到的不是扫回的行：{other:?}"
            ))),
        }
    }

    /// 删前缀一帧，等回应（`protocol.rs` 文件头「删前缀」）：`after_key` 空是从前缀起，不空是从排在它之后的第一把键起；`count_only` 为真只数不删。
    /// 忙、断线与按前缀扫同一套；删不进追加日志，断线重连之后这一帧照样重发（删幂等，重发只会让这一帧报的数少一批）。
    ///
    /// # Errors
    /// 帧体超过帧长上限（[`ClientFailure::Protocol`]，没发）；连不上、忙到上限、服务停了（退 75）；被拒（前缀空、落在记账那一段、续扫起点不以前缀起头）、回的不是删前缀的结局。
    pub fn delete_prefix_frame(
        &mut self,
        prefix: &[u8],
        after_key: &[u8],
        maximum_keys: u32,
        count_only: bool,
    ) -> Result<PrefixDeletionFrame, ClientFailure> {
        self.wait_for_window()?;
        let frame = encode_delete_prefix_frame(0, prefix, after_key, maximum_keys, count_only);
        if let Some(failure) =
            oversize_frame_failure(&frame, &format!("删前缀（前缀 {} 字节）", prefix.len()))
                .filter(|_| !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried))
        {
            return Err(failure);
        }
        let operation = self.register(OperationKind::DeletePrefix, frame);
        self.send_unsent()?;
        match self.wait_for(operation)? {
            Completion::PrefixDeleted {
                keys_in_frame,
                is_exhausted,
                last_key,
                ..
            } => Ok(PrefixDeletionFrame {
                keys_in_frame,
                is_exhausted,
                last_key,
            }),
            Completion::Failed { failure, .. } => Err(failure),
            other @ (Completion::WriteAcknowledged { .. }
            | Completion::Values { .. }
            | Completion::Claims { .. }
            | Completion::Statistics { .. }
            | Completion::ScanEntries { .. }) => Err(ClientFailure::Protocol(format!(
                "删前缀收到的不是删前缀的结局：{other:?}"
            ))),
        }
    }

    /// 数或删 `prefix` 下的全部键：一帧一帧接着来（每帧续扫起点取上一帧回的最后一把键），直到前缀下没有更多的键；回数到或删掉的总数。
    /// `count_only` 为真只数不删（干跑）。每帧至多碰 `maximum_keys_per_frame` 把键。
    ///
    /// # Errors
    /// 同 [`Self::delete_prefix_frame`]；服务说前缀下还有键却没回最后一把键（续不下去）回 [`ClientFailure::Protocol`]。
    pub fn delete_prefix(
        &mut self,
        prefix: &[u8],
        maximum_keys_per_frame: u32,
        count_only: bool,
    ) -> Result<u64, ClientFailure> {
        let mut total = 0_u64;
        let mut after_key = Vec::new();
        loop {
            let frame =
                self.delete_prefix_frame(prefix, &after_key, maximum_keys_per_frame, count_only)?;
            total += u64::from(frame.keys_in_frame);
            if frame.is_exhausted {
                return Ok(total);
            }
            if frame.last_key.is_empty() {
                return Err(ClientFailure::Protocol(
                    "删前缀说前缀下还有键，却没回最后一把键：续不下去".to_string(),
                ));
            }
            after_key = frame.last_key;
        }
    }

    /// 成批点读（照 [`Self::read`]），回的值个数要与问的键数相同。
    fn read_as_many_values_as_keys(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        let values = self.read(keys)?;
        if values.len() != keys.len() {
            return Err(ClientFailure::Protocol(format!(
                "成批点读问了 {} 个键，服务回了 {} 个值",
                keys.len(),
                values.len()
            )));
        }
        Ok(values)
    }

    /// 成批写之前把大值分段（文件头「大值分段」；Python 客户端 `write_many` 的头两步）：先点读这批键的计数键，照高水位把要分段的
    /// 拆成各段与计数键，按 `frame_byte_budget` 切帧写下、等全部确认，再把清单连同不分段的条目交回，由调用方照它自己的切法最后写
    /// （一块一批落在一帧里的原子由调用方的切法保住）。没有要分段的就一条不写、原样交回（计数键照读：分段存过的键之后写短值也照分段存）。
    ///
    /// # Errors
    /// 有一把原键的形状是段键或计数键、有一个值要的段数超过段号装得下的（[`ClientFailure::SegmentedValue`]，一条都没写）；计数键读不成
    /// （照 [`Self::read`]）或不是十进制（[`ClientFailure::SegmentedValue`]）；写不成（照 [`Self::submit_write_entries`] 与
    /// [`Self::wait_all_acknowledged`]：段写了一部分时清单还是旧的，读时核不过、报读错，重写这把键就好）。
    pub fn write_segments_of_large_values(
        &mut self,
        entries: Vec<TableEntry>,
        frame_byte_budget: usize,
    ) -> Result<Vec<TableEntry>, ClientFailure> {
        if entries.is_empty() {
            return Ok(entries);
        }
        let count_keys = segment_count_keys_to_read_before_writing(&entries)
            .map_err(ClientFailure::SegmentedValue)?;
        let mut segment_count_high_water_of_each_entry = Vec::with_capacity(entries.len());
        for frame in frame_ranges(
            &count_keys,
            |(_table, count_key)| POINT_READ_BYTES_PER_KEY + count_key.len(),
            frame_byte_budget,
            SEGMENTED_FRAME_ENTRY_LIMIT,
        ) {
            let stored_counts = self.read_as_many_values_as_keys(&count_keys[frame.clone()])?;
            for ((_table, key, _value), stored_count) in entries[frame].iter().zip(stored_counts) {
                segment_count_high_water_of_each_entry.push(
                    segment_count_high_water(key, stored_count.as_deref())
                        .map_err(ClientFailure::SegmentedValue)?,
                );
            }
        }
        let plan = plan_segmented_write(entries, &segment_count_high_water_of_each_entry)
            .map_err(ClientFailure::SegmentedValue)?;
        for frame in frame_ranges(
            &plan.segment_and_count_entries,
            |(_table, key, value)| WRITE_BYTES_PER_ENTRY + key.len() + value.len(),
            frame_byte_budget,
            SEGMENTED_FRAME_ENTRY_LIMIT,
        ) {
            self.submit_write_entries(&plan.segment_and_count_entries[frame])?;
        }
        self.wait_all_acknowledged()?;
        Ok(plan.entries_to_write_last)
    }

    /// 成批点读，分段存的键拼回整值（文件头「大值分段」；Python 客户端 `read_many`）：原键照 [`Self::read`] 点读（回应越过帧长上限就拆键），
    /// 读回的是清单就按清单点读各段（[`Self::assemble_segmented_value`]）。
    ///
    /// # Errors
    /// 同 [`Self::read`]；回的值个数与问的键数不同（[`ClientFailure::Protocol`]）；清单解不开、段缺了、长度或 sha256 对不上
    /// （[`ClientFailure::SegmentedValue`]），不当成键不在。
    pub fn read_assembling_segmented_values(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        let stored_values = self.read_as_many_values_as_keys(keys)?;
        let mut values = Vec::with_capacity(keys.len());
        for ((table, key), stored) in keys.iter().zip(stored_values) {
            let Some(stored) = stored else {
                values.push(None);
                continue;
            };
            match SegmentManifest::parse_stored_value(key, &stored)
                .map_err(ClientFailure::SegmentedValue)?
            {
                None => values.push(Some(stored)),
                Some(manifest) => {
                    let (_owner_key, whole) = self.assemble_segmented_value(
                        *table,
                        key,
                        SegmentedValueBeingAssembled::read_by_point(key.clone(), manifest),
                    )?;
                    values.push(Some(whole));
                }
            }
        }
        Ok(values)
    }

    /// 一把分段存的键拼回整值（Python 客户端 `assembled`）：按清单点读还缺的段（每帧至多 [`SEGMENT_READ_FRAME_BYTES`]）、拼回、核每段长度与
    /// 整值 sha256，交 (调用方的原键, 整值)；拼不回照 [`assemble_rereading_the_original_key`] 重读原键（清单换过照新清单重拼）。
    /// `key_in_the_service` 是这把原键在服务里的全名（按前缀扫的一方交过来的原键去掉过自己的键空间，读段与重读原键要接回去）。
    ///
    /// # Errors
    /// 读段不成（照 [`Self::read`]）；等完清单没换、段缺了、长度或 sha256 对不上（[`ClientFailure::SegmentedValue`]）。
    pub fn assemble_segmented_value(
        &mut self,
        table: Table,
        key_in_the_service: &[u8],
        being_assembled: SegmentedValueBeingAssembled,
    ) -> Result<(Vec<u8>, Vec<u8>), ClientFailure> {
        let mut reader = SegmentReadsOverTheService {
            client: self,
            table,
            key_in_the_service,
        };
        assemble_rereading_the_original_key(being_assembled, &mut reader)?
            .map_err(ClientFailure::SegmentedValue)
    }

    /// 点读一把分段存的键清单点名、还缺的段（每帧至多 [`SEGMENT_READ_FRAME_BYTES`]）。
    fn read_missing_segments(
        &mut self,
        table: Table,
        key_in_the_service: &[u8],
        being_assembled: &SegmentedValueBeingAssembled,
    ) -> Result<SegmentsReadByPoint, ClientFailure> {
        let missing_segment_indexes = being_assembled.missing_segment_indexes();
        let mut segments_read_by_point = Vec::with_capacity(missing_segment_indexes.len());
        for frame in frame_ranges(
            &missing_segment_indexes,
            |segment_index| {
                being_assembled.segment_length_in_bytes(*segment_index)
                    + key_in_the_service.len()
                    + SEGMENT_READ_OVERHEAD_BYTES_PER_KEY
            },
            SEGMENT_READ_FRAME_BYTES,
            SEGMENTED_FRAME_ENTRY_LIMIT,
        ) {
            let segment_indexes = &missing_segment_indexes[frame];
            let segment_keys: Vec<(Table, Vec<u8>)> = segment_indexes
                .iter()
                .map(|segment_index| (table, segment_key(key_in_the_service, *segment_index)))
                .collect();
            let pieces = self.read_as_many_values_as_keys(&segment_keys)?;
            segments_read_by_point.extend(segment_indexes.iter().copied().zip(pieces));
        }
        Ok(segments_read_by_point)
    }

    fn backoff_delay(&mut self, attempts_made: u32) -> Duration {
        let policy = self.settings.reconnect;
        let nominal = if self.is_enabled(BreakSwitch::ReconnectBackoffFlat) {
            policy.initial_delay
        } else {
            let doublings = attempts_made.saturating_sub(1).min(20);
            policy
                .initial_delay
                .saturating_mul(1_u32 << doublings)
                .min(policy.maximum_delay)
        };
        if self.is_enabled(BreakSwitch::ReconnectWithoutJitter) {
            return nominal;
        }
        nominal.mul_f64(0.5 + 0.5 * self.next_random_fraction())
    }

    fn sleep_unless_stopped(&self, duration: Duration) -> Result<(), ClientFailure> {
        let deadline = Instant::now() + duration;
        loop {
            if (self.settings.stop_check)() {
                return Err(ClientFailure::Stopped);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Ok(());
            }
            thread::sleep(remaining.min(POLL_SLICE));
        }
    }

    /// 没连着就照重连策略连。一轮重连从要连的那一刻起，到连上之后收到第一个回应才结束：连得上、一发帧就断的
    /// （服务一接就关、中间的代理接了却连不上服务）也算这一轮里没成的一次，照样退避、照样数进次数与总时长上限，不会空转。
    fn ensure_connected(&mut self) -> Result<(), ClientFailure> {
        if self.connection.is_some() {
            return Ok(());
        }
        let policy = self.settings.reconnect;
        let is_unbounded = self.is_enabled(BreakSwitch::ReconnectUnbounded);
        let mut cycle = self
            .reconnect_cycle
            .take()
            .unwrap_or_else(|| ReconnectCycle {
                started: Instant::now(),
                attempt_times: Vec::new(),
                last_error: String::new(),
            });
        loop {
            if (self.settings.stop_check)() {
                self.diagnostics
                    .connection_attempt_cycles
                    .push(cycle.attempt_times);
                return Err(ClientFailure::Stopped);
            }
            if !cycle.attempt_times.is_empty() {
                let attempts = u32::try_from(cycle.attempt_times.len()).unwrap_or(u32::MAX);
                let delay = self.backoff_delay(attempts);
                let elapsed = cycle.started.elapsed();
                if !is_unbounded
                    && (attempts >= policy.maximum_attempts
                        || elapsed + delay > policy.maximum_total)
                {
                    self.diagnostics
                        .connection_attempt_cycles
                        .push(cycle.attempt_times);
                    return Err(ClientFailure::ServerUnreachable {
                        attempts,
                        elapsed,
                        last_error: cycle.last_error,
                    });
                }
                self.activity.enter(ClientActivity::BackingOff);
                if let Err(stopped) = self.sleep_unless_stopped(delay) {
                    self.diagnostics
                        .connection_attempt_cycles
                        .push(cycle.attempt_times);
                    return Err(stopped);
                }
            }
            let attempt_started = Instant::now();
            cycle.attempt_times.push(attempt_started);
            // 每次尝试只用这一轮剩下的时间（不设总时长上限的弄坏开关下每次给满一份上限）。
            let attempt_deadline = if is_unbounded {
                attempt_started + policy.maximum_total
            } else {
                cycle.started + policy.maximum_total
            };
            self.activity.enter(ClientActivity::Connecting);
            let opened = open_connection(&self.settings, attempt_deadline);
            self.activity.enter(ClientActivity::Outside);
            match opened {
                Ok(connection) => {
                    self.connection = Some(connection);
                    break;
                }
                Err(message) => cycle.last_error = message,
            }
        }
        if self.has_connected_before {
            self.diagnostics.reconnections += 1;
        }
        self.has_connected_before = true;
        self.reconnect_cycle = Some(cycle);
        Ok(())
    }

    /// 这一轮重连还没收到回应时，在途的帧最晚等到哪一刻：这一轮的总时长上限。连上了却一直没有回应（服务不读、不回）的，
    /// 也在总时长上限里收场，不等满一份帧超时。没在重连里、或弄坏开关 connect-without-deadline、reconnect-unbounded 下回 `None`。
    fn reconnect_cycle_deadline(&self) -> Option<Instant> {
        if self.is_enabled(BreakSwitch::ConnectWithoutDeadline)
            || self.is_enabled(BreakSwitch::ReconnectUnbounded)
        {
            return None;
        }
        self.reconnect_cycle
            .as_ref()
            .map(|cycle| cycle.started + self.settings.reconnect.maximum_total)
    }

    /// 收到一个回应：这一轮重连算成了。
    fn close_reconnect_cycle(&mut self) {
        if let Some(cycle) = self.reconnect_cycle.take() {
            self.diagnostics
                .connection_attempt_cycles
                .push(cycle.attempt_times);
        }
    }

    /// 连接坏了：断开，在途的帧都标成要重发。弄坏开关 unacknowledged-frames-dropped-on-reconnect 在对端关了连接或读写出错时
    /// （帧超时那一种不算）把在途的写当成已确认丢掉。
    fn mark_broken(&mut self, reason: String, cause: BreakCause) -> Result<(), ClientFailure> {
        // 服务宣布停了之后马上退出、或拒了帧长之后关了连接时，写先出错、「服务停了」「帧长度不对」那一帧还躺在接收缓冲里：
        // 重连之前先把它读出来（文件头「单条超限当场报」）。
        if cause == BreakCause::PeerClosedOrFailed {
            match self.connection_notice_left_unread() {
                Some(ConnectionNotice::ServerStopped(message))
                    if !self.is_enabled(BreakSwitch::StopNoticeRetried) =>
                {
                    self.connection = None;
                    return Err(ClientFailure::ServerStopped { message });
                }
                Some(ConnectionNotice::FrameLengthRejected(message))
                    if !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried) =>
                {
                    self.connection = None;
                    return Err(ClientFailure::Protocol(format!(
                        "服务回「帧长度不对」并关了连接（上限 {MAXIMUM_FRAME_BODY_BYTES} 字节），不重连：{message}"
                    )));
                }
                Some(
                    ConnectionNotice::ServerStopped(_) | ConnectionNotice::FrameLengthRejected(_),
                )
                | None => {}
            }
        }
        let drops_unacknowledged = cause == BreakCause::PeerClosedOrFailed
            && self
                .settings
                .break_switches
                .is_enabled(BreakSwitch::UnacknowledgedFramesDroppedOnReconnect);
        self.connection = None;
        self.wire.clear();
        if let Some(cycle) = self.reconnect_cycle.as_mut() {
            cycle.last_error.clone_from(&reason);
        }
        self.diagnostics.last_break_reason = Some(reason);
        let on_wire: Vec<OperationIdentifier> = self
            .operations
            .iter()
            .filter(|(_, operation)| matches!(operation.state, OperationState::OnWire { .. }))
            .map(|(identifier, _)| *identifier)
            .collect();
        for identifier in on_wire {
            let operation = self.operations.get_mut(&identifier).expect("上面刚列出来");
            let dropped_request = match operation.kind {
                OperationKind::Write { request_number } if drops_unacknowledged => {
                    Some(request_number)
                }
                OperationKind::Write { .. }
                | OperationKind::Read
                | OperationKind::Scan
                | OperationKind::Claim { .. }
                | OperationKind::Statistics
                | OperationKind::DeletePrefix => {
                    operation.state = OperationState::Unsent;
                    self.diagnostics.replayed_frames += 1;
                    None
                }
            };
            if let Some(request_number) = dropped_request {
                self.operations.remove(&identifier);
                self.acknowledge_in_journal(request_number)?;
                self.completions.push_back(Completion::WriteAcknowledged {
                    operation: identifier,
                    request_number,
                    entries_written: 0,
                    is_already_applied: false,
                });
            }
        }
        Ok(())
    }

    fn acknowledge_in_journal(&mut self, request_number: u64) -> Result<(), ClientFailure> {
        let still_unacknowledged = self
            .operations
            .values()
            .filter_map(|operation| match operation.kind {
                OperationKind::Write {
                    request_number: unacknowledged,
                } => Some((unacknowledged, operation.frame.as_slice())),
                OperationKind::Read
                | OperationKind::Scan
                | OperationKind::Claim { .. }
                | OperationKind::Statistics
                | OperationKind::DeletePrefix => None,
            });
        self.activity.enter(ClientActivity::WritingJournal);
        let journaled = self
            .journal
            .append_acknowledgement(request_number, still_unacknowledged);
        self.activity.enter(ClientActivity::Outside);
        journaled.map_err(ClientFailure::Journal)
    }

    /// 追加日志的追加与整理计数。
    pub fn journal_statistics(&self) -> JournalStatistics {
        self.journal.statistics()
    }

    /// 连上（必要时重连），按操作次序发出全部要发的帧；发到一半连接断了就重连再发。
    fn send_unsent(&mut self) -> Result<(), ClientFailure> {
        loop {
            self.ensure_connected()?;
            match self.write_unsent_frames() {
                Ok(()) => return Ok(()),
                Err((reason, cause)) => {
                    if cause == BreakCause::WriteTimeout {
                        self.diagnostics.write_timeouts += 1;
                    }
                    self.mark_broken(reason, cause)?;
                }
            }
        }
    }

    fn write_unsent_frames(&mut self) -> Result<(), (String, BreakCause)> {
        let frame_timeout = self.settings.frame_timeout;
        let cycle_deadline = self.reconnect_cycle_deadline();
        let Some(connection) = self.connection.as_mut() else {
            return Err(("没连着".to_string(), BreakCause::PeerClosedOrFailed));
        };
        for (identifier, operation) in &mut self.operations {
            if !matches!(operation.state, OperationState::Unsent) {
                continue;
            }
            let sequence = self.next_sequence;
            self.next_sequence += 1;
            replace_request_sequence(&mut operation.frame, sequence);
            if let OperationKind::Claim {
                lease_milliseconds,
                submitted_at,
            } = operation.kind
            {
                replace_claim_lease(
                    &mut operation.frame,
                    claim_lease_left(lease_milliseconds, submitted_at.elapsed()),
                );
            }
            self.activity.enter(ClientActivity::WritingFrame);
            let written = connection.writer.write_all(&operation.frame);
            self.activity.enter(ClientActivity::Outside);
            written.map_err(|error| {
                let cause = if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) {
                    BreakCause::WriteTimeout
                } else {
                    BreakCause::PeerClosedOrFailed
                };
                (format!("发帧失败：{error}"), cause)
            })?;
            let frame_deadline = Instant::now() + frame_timeout;
            operation.state = OperationState::OnWire {
                deadline: cycle_deadline
                    .map_or(frame_deadline, |cycle_end| cycle_end.min(frame_deadline)),
            };
            self.wire.insert(sequence, *identifier);
        }
        Ok(())
    }

    /// 收一轮回应、处理到点的重发与超时；至多等 `maximum_wait`。
    fn pump_once(&mut self, maximum_wait: Duration) -> Result<(), ClientFailure> {
        if (self.settings.stop_check)() {
            return Err(ClientFailure::Stopped);
        }
        let now = Instant::now();
        let mut has_unsent = false;
        for operation in self.operations.values_mut() {
            match operation.state {
                OperationState::Deferred {
                    due,
                    busy_received_at,
                    retry_after,
                } if due <= now => {
                    self.diagnostics
                        .busy_waits
                        .push((now.duration_since(busy_received_at), retry_after));
                    operation.state = OperationState::Unsent;
                    has_unsent = true;
                }
                OperationState::Unsent => has_unsent = true,
                OperationState::Deferred { .. } | OperationState::OnWire { .. } => {}
            }
        }
        if has_unsent || self.connection.is_none() {
            self.send_unsent()?;
        }
        let ignores_timeout = self.is_enabled(BreakSwitch::FrameTimeoutIgnored);
        let mut wait = maximum_wait;
        for operation in self.operations.values() {
            let wake_at = match operation.state {
                OperationState::OnWire { deadline } if !ignores_timeout => Some(deadline),
                OperationState::Deferred { due, .. } => Some(due),
                OperationState::OnWire { .. } | OperationState::Unsent => None,
            };
            if let Some(wake_at) = wake_at {
                wait = wait.min(wake_at.saturating_duration_since(now));
            }
        }
        let wait = wait.max(Duration::from_millis(1));
        let read_result = {
            let Some(connection) = self.connection.as_mut() else {
                return Ok(());
            };
            self.activity.enter(ClientActivity::WaitingForResponses);
            let read_now = match connection.reader.set_read_timeout(Some(wait)) {
                Ok(()) => connection.reader.read(&mut self.scratch),
                Err(error) => Err(error),
            };
            self.activity.enter(ClientActivity::Outside);
            read_now
        };
        match read_result {
            Ok(0) => {
                return self.mark_broken("服务关了连接".to_string(), BreakCause::PeerClosedOrFailed)
            }
            Ok(read_now) => {
                let connection = self.connection.as_mut().expect("上面刚读过");
                connection
                    .buffer
                    .extend_from_slice(&self.scratch[..read_now]);
                self.handle_buffered_responses()?;
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => {
                return self.mark_broken(
                    format!("读回应出错：{error}"),
                    BreakCause::PeerClosedOrFailed,
                )
            }
        }
        if !ignores_timeout {
            let checked_at = Instant::now();
            let expired = self.operations.values().any(|operation| {
                matches!(operation.state, OperationState::OnWire { deadline } if deadline <= checked_at)
            });
            if expired {
                self.diagnostics.frame_timeouts += 1;
                let reason = match self.reconnect_cycle_deadline() {
                    Some(cycle_end) if cycle_end <= checked_at => format!(
                        "这一轮重连 {} 秒的总时长上限到了：连上了却一个回应都没收到（服务不读、不回）",
                        self.settings.reconnect.maximum_total.as_secs_f64()
                    ),
                    Some(_) | None => format!(
                        "在途的帧 {} 秒没回应",
                        self.settings.frame_timeout.as_secs_f64()
                    ),
                };
                return self.mark_broken(reason, BreakCause::FrameTimeout);
            }
        }
        Ok(())
    }

    fn handle_buffered_responses(&mut self) -> Result<(), ClientFailure> {
        let mut consumed = 0;
        loop {
            let Some(connection) = self.connection.as_ref() else {
                return Ok(());
            };
            let buffer = &connection.buffer[consumed..];
            let Some(length_field) = buffer.get(..FRAME_LENGTH_FIELD_BYTES) else {
                break;
            };
            let declared = u32::from_le_bytes(length_field.try_into().expect("4 字节"));
            if !is_frame_length_acceptable(declared, RESPONSE_HEADER_BYTES) {
                return Err(ClientFailure::Protocol(format!(
                    "回应帧体长度 {declared} 不对"
                )));
            }
            let frame_end =
                FRAME_LENGTH_FIELD_BYTES + usize::try_from(declared).expect("u32 放得进 usize");
            let Some(body) = buffer.get(FRAME_LENGTH_FIELD_BYTES..frame_end) else {
                break;
            };
            let response = decode_response_body(body).map_err(ClientFailure::Protocol)?;
            consumed += frame_end;
            self.handle_response(response)?;
        }
        if let Some(connection) = self.connection.as_mut() {
            connection.buffer.drain(..consumed);
        }
        Ok(())
    }

    /// 连接坏了的那一刻，接收缓冲与套接字里还没读的回应里有没有序号 0 的「服务停了」或「帧长度不对」；只读不处理别的帧。
    fn connection_notice_left_unread(&mut self) -> Option<ConnectionNotice> {
        let connection = self.connection.as_mut()?;
        if connection
            .reader
            .set_read_timeout(Some(STOP_NOTICE_DRAIN_TIMEOUT))
            .is_ok()
        {
            let mut chunk = [0_u8; 64 * 1024];
            while let Ok(read_now) = connection.reader.read(&mut chunk) {
                if read_now == 0 {
                    break;
                }
                connection.buffer.extend_from_slice(&chunk[..read_now]);
            }
        }
        let mut buffer = connection.buffer.as_slice();
        while let Some(length_field) = buffer.get(..FRAME_LENGTH_FIELD_BYTES) {
            let declared = u32::from_le_bytes(length_field.try_into().expect("4 字节"));
            if !is_frame_length_acceptable(declared, RESPONSE_HEADER_BYTES) {
                return None;
            }
            let frame_end =
                FRAME_LENGTH_FIELD_BYTES + usize::try_from(declared).expect("u32 放得进 usize");
            let body = buffer.get(FRAME_LENGTH_FIELD_BYTES..frame_end)?;
            match decode_response_body(body) {
                Ok(Response {
                    sequence: CONNECTION_LEVEL_SEQUENCE,
                    outcome: ResponseOutcome::ServerStopping { message },
                    ..
                }) => return Some(ConnectionNotice::ServerStopped(message)),
                Ok(Response {
                    sequence: CONNECTION_LEVEL_SEQUENCE,
                    outcome:
                        ResponseOutcome::Rejected {
                            status: RejectionStatus::FrameLengthRejected,
                            message,
                        },
                    ..
                }) => return Some(ConnectionNotice::FrameLengthRejected(message)),
                Ok(_) | Err(_) => {}
            }
            buffer = &buffer[frame_end..];
        }
        None
    }

    /// 序号 0 的回应：服务停了、帧长度不对。
    fn handle_connection_level_response(
        &mut self,
        outcome: ResponseOutcome,
    ) -> Result<(), ClientFailure> {
        match outcome {
            ResponseOutcome::ServerStopping { message }
                if self.is_enabled(BreakSwitch::StopNoticeRetried) =>
            {
                self.mark_broken(
                    format!("服务说它停了：{message}"),
                    BreakCause::PeerClosedOrFailed,
                )
            }
            ResponseOutcome::ServerStopping { message } => {
                Err(ClientFailure::ServerStopped { message })
            }
            ResponseOutcome::Rejected {
                status: RejectionStatus::FrameLengthRejected,
                message,
            } if self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried) => self.mark_broken(
                format!("服务拒了整条连接：{message}"),
                BreakCause::PeerClosedOrFailed,
            ),
            other @ (ResponseOutcome::PointValues(_)
            | ResponseOutcome::WriteAcknowledged { .. }
            | ResponseOutcome::ScanEntries { .. }
            | ResponseOutcome::Statistics(_)
            | ResponseOutcome::Claims(_)
            | ResponseOutcome::PrefixDeleted { .. }
            | ResponseOutcome::ConnectedClients(_)
            | ResponseOutcome::Busy { .. }
            | ResponseOutcome::Rejected { .. }) => Err(ClientFailure::Protocol(format!(
                "服务拒了整条连接：{other:?}"
            ))),
        }
    }

    fn handle_response(&mut self, response: Response) -> Result<(), ClientFailure> {
        if response.sequence == CONNECTION_LEVEL_SEQUENCE {
            return self.handle_connection_level_response(response.outcome);
        }
        let Some(identifier) = self.wire.remove(&response.sequence) else {
            if response.sequence < self.next_sequence {
                self.diagnostics.duplicate_responses += 1;
                if self.is_enabled(BreakSwitch::DuplicateResponsesFatal) {
                    return Err(ClientFailure::Protocol(format!(
                        "序号 {} 的回应已经收过一次",
                        response.sequence
                    )));
                }
                return Ok(());
            }
            return Err(ClientFailure::Protocol(format!(
                "回应的序号 {} 没发过（下一个要发的是 {}）",
                response.sequence, self.next_sequence
            )));
        };
        self.close_reconnect_cycle();
        let kind = self
            .operations
            .get(&identifier)
            .expect("在线路表里的操作都在操作表里")
            .kind;
        match (kind, response.outcome) {
            (
                _,
                ResponseOutcome::Busy {
                    retry_after_milliseconds,
                    ..
                },
            ) => self.defer_after_busy(
                identifier,
                kind,
                Duration::from_millis(u64::from(retry_after_milliseconds)),
            ),
            (
                OperationKind::Write { request_number },
                ResponseOutcome::WriteAcknowledged {
                    entries_written,
                    is_already_applied,
                },
            ) => {
                self.operations.remove(&identifier);
                self.acknowledge_in_journal(request_number)?;
                self.completions.push_back(Completion::WriteAcknowledged {
                    operation: identifier,
                    request_number,
                    entries_written,
                    is_already_applied,
                });
                Ok(())
            }
            (OperationKind::Read, ResponseOutcome::PointValues(values)) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::Values {
                    operation: identifier,
                    values,
                });
                Ok(())
            }
            (
                OperationKind::Scan,
                ResponseOutcome::ScanEntries {
                    is_truncated,
                    entries,
                },
            ) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::ScanEntries {
                    operation: identifier,
                    is_truncated,
                    entries,
                });
                Ok(())
            }
            (
                OperationKind::DeletePrefix,
                ResponseOutcome::PrefixDeleted {
                    keys_in_frame,
                    is_exhausted,
                    last_key,
                },
            ) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::PrefixDeleted {
                    operation: identifier,
                    keys_in_frame,
                    is_exhausted,
                    last_key,
                });
                Ok(())
            }
            (OperationKind::Claim { .. }, ResponseOutcome::Claims(outcomes)) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::Claims {
                    operation: identifier,
                    outcomes,
                });
                Ok(())
            }
            (OperationKind::Statistics, ResponseOutcome::Statistics(counters)) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::Statistics {
                    operation: identifier,
                    counters,
                });
                Ok(())
            }
            (_, ResponseOutcome::Rejected { status, message }) => {
                self.operations.remove(&identifier);
                let failure = ClientFailure::Rejected { status, message };
                if matches!(kind, OperationKind::Write { .. }) {
                    // 写被拒是协议或存储的错，重发也不会好；那一帧留在追加日志里，交调用方定。
                    return Err(failure);
                }
                self.completions.push_back(Completion::Failed {
                    operation: identifier,
                    failure,
                });
                Ok(())
            }
            (kind, outcome) => Err(ClientFailure::Protocol(format!(
                "{kind:?} 帧收到的回应对不上：{outcome:?}"
            ))),
        }
    }

    fn defer_after_busy(
        &mut self,
        identifier: OperationIdentifier,
        kind: OperationKind,
        retry_after: Duration,
    ) -> Result<(), ClientFailure> {
        self.diagnostics.busy_responses += 1;
        let jitter = retry_after.mul_f64(0.5 * self.next_random_fraction());
        let delay = if self.is_enabled(BreakSwitch::BusyRetryWithoutDelay) {
            Duration::ZERO
        } else {
            retry_after + jitter
        };
        let maximum_retries = self.settings.maximum_busy_retries;
        let operation = self
            .operations
            .get_mut(&identifier)
            .expect("调用方刚从操作表里取到");
        operation.busy_retries += 1;
        if operation.busy_retries > maximum_retries {
            let retries = operation.busy_retries;
            if matches!(kind, OperationKind::Write { .. }) {
                operation.state = OperationState::Unsent;
                return Err(ClientFailure::BusyLimit { retries });
            }
            self.operations.remove(&identifier);
            self.completions.push_back(Completion::Failed {
                operation: identifier,
                failure: ClientFailure::BusyLimit { retries },
            });
            return Ok(());
        }
        let now = Instant::now();
        operation.state = OperationState::Deferred {
            due: now + delay,
            busy_received_at: now,
            retry_after,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 重发认领时给剩下的租约：过了 3 秒的 10 秒租约只给 7 秒，过完了的给 1 毫秒，刚交的照原租约。
    /// 判别力（`crates/mutations.tsv`）：重发照原租约给（到期往后推），这条红。
    #[test]
    fn a_resent_claim_gets_only_the_lease_left_since_it_was_submitted() {
        assert_eq!(claim_lease_left(10_000, Duration::from_secs(3)), 7_000);
        assert_eq!(
            claim_lease_left(10_000, Duration::from_secs(30)),
            1,
            "过完了的至少给 1 毫秒"
        );
        assert_eq!(claim_lease_left(10_000, Duration::ZERO), 10_000);
    }
}
