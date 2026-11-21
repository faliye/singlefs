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
//! - 按前缀扫（`scan`、`scan_after`）与点读走同一套：忙照上一条退避重发、忙到上限回 `BusyLimit`，连接断了照重连那一条重发、到上限回「连不上」，
//!   被拒照原样回；扫回来的行原样交，不当成「前缀下没有行」。截断了的由调用方拿这一帧的最后一把键作续扫起点调 `scan_after` 接着扫
//!   （`protocol` 文件头「按前缀扫」），不往下拆前缀。
//! - 删前缀（`delete_prefix_frame`、`delete_prefix`）与按前缀扫同一套退避与重连；不进追加日志、不带请求号：删本来就幂等，重发只会让某一帧报的数少一批，
//!   库里的结果不变（`protocol` 文件头「删前缀」）。`delete_prefix` 一帧一帧接着来，直到服务说前缀下没有更多的键。
//! - 认领（`claim`、`submit_claim`）：每次发出去（头一次、断线重连之后、回了忙之后）帧里都是调用方给的整段租期，不换成剩下的；
//!   服务对同一身份、租期没到的认领到期只往后挪（`committer.rs` 文件头「认领」）。所以忙了大半租期才落下的、丢了回应之后重发才落下的认领，
//!   回到调用方手里时到期离服务落下它的那一刻还有整段租期，调用方从回来那一刻起按整段租期写，盖得住（只差同步与回程）。
//!   弄坏开关 claim-lease-shrunk-on-resend 退回改前：每次发出去换成「原租期减去从交给客户端起过了的毫秒数，至少 1 毫秒」。
//! - 存储出错（写帧回状态 4）：不是「重发也不会好」的拒绝。帧留在操作表与追加日志里，照重连策略的退避（起始间隔翻倍、封顶、抖动）
//!   隔一会用同一个请求号再交：写失败时服务还在，再交就写；同步失败时服务随即退 7、连接断了，照重连那一条等服务被拉起来再重放，
//!   到重连上限回「连不上」（`committer.rs` 文件头、`storage.rs` 文件头「同步 WAL 失败之后」）。同一帧连着回存储出错到重连次数上限
//!   （`ReconnectPolicy::maximum_attempts`）回 `ClientFailure::StorageFailing`（退 75），帧留在日志里。同一次调用里排着的别的帧照常发、照常收。
//!   读、扫、认领、删前缀回状态 4 照旧是这一帧失败、连接继续。弄坏开关 write-storage-failure-abandoned 退回改前：写帧回状态 4 就从操作表拿掉、
//!   回「服务拒了」退 3、不再重放。
//! - 不在才写（`write_if_absent`、`submit_write_if_absent`）与写结论（`write_guarded_conclusion`、`submit_guarded_conclusion_write`）
//!   与成批写同一套：带请求号、记进追加日志、断线重放、回忙退避、存储出错照同一请求号再交（`protocol.rs` 文件头「不在才写」「写结论」）。
//! - 写结论：服务回状态 7（守卫不过：认领被别的持有者持着、租期没到，守卫那一行或一把删键那一行此刻的样子与写的一方读到的不同）是这一帧的结局，
//!   不是协议错：这一帧一条都没写，从操作表与追加日志里勾掉，交 `Completion::ConclusionRefused`（`write_guarded_conclusion` 交
//!   `GuardedConclusionWriteOutcome::Refused`），调用方照种类定。重开时从追加日志重放的写结论帧收到它照样勾掉、交进完成队。
//!   勾掉了的帧不再重放；收到状态 7 之后、勾掉之前客户端死了的，重开时重放那一帧，服务照它此刻的认领与那一行再判一遍。
//! - 认领可以替别的持有者认领（`claim_as`、`submit_claim_as`）：持有者身份进认领帧的身份字段，不必是这个客户端的身份（GPU 服务一个客户端
//!   替几张卡、几个判线程认领，`gpu_judging_rows` 文件头「认领」）；`claim` 照旧用这个客户端的身份。
//! - GPU 判的三族（`gpu_judging_rows` 文件头）：`write_gpu_pending_rows` 走不在才写，`claim_gpu_blocks` 走 `claim_as`，`write_gpu_conclusion` 走写结论；
//!   弄坏开关 rejudged-row-dropped 让 `write_gpu_conclusion` 不带强制现判的那一行。
//! - 放认领（`release_claims_as`、`release_gpu_block_claims`）：以点名的持有者放掉几把 GPU 认领键上的认领（`protocol` 文件头「放认领」），
//!   与删前缀同一套：不进追加日志、不带请求号，忙退避、断线重连之后照样重发（放幂等）。
//! - 列认领（`list_claims_after` 一帧、`list_claims` 列到不再截断）：按前缀列出此刻租期没到的认领与各自的持有者身份、剩余租期（`protocol` 文件头「列认领」，
//!   前缀要以 `gpu-claim/` 起头）；只读，不进追加日志、不带请求号，忙退避、断线重连之后照样重发。
//! - 重复的回应（序号已经回过，例如网络把一帧送了两遍）不算错，记一笔、丢掉。
//! - 单条超限当场报：写、读、扫、认领的帧拼好之后先核帧体长度，超过 `MAXIMUM_FRAME_BODY_BYTES` 的当场回 `ClientFailure::Protocol`
//!   （说是哪一种帧、几条、帧体多大、上限多少），不记追加日志、不发、不重试（服务会拒整条连接，重发照样拒）。
//!   服务回「帧长度不对」（序号 0 的状态 3）同样回协议错、不重连；发帧出错时先读出还躺在接收缓冲里的回应，是「帧长度不对」就照这一条回。
//! - 回应越过帧长上限就拆键：成批点读的请求帧小、读回的值合计可以越过帧长上限，服务这时整帧拒（`MalformedPayload`，说明里写读回会有多少字节，
//!   协议不变）；`read` 把这一帧的键对半拆开各自再问、递归到单键，按原次序接起来（单键回应至多一段长加帧头，不会越过：长过一段的值分段存）。
//! - 大值分段：一行的值长过一段（一帧装不下）照 Python 客户端的分段格式存成清单加这一版的段（`segmented_values` 文件头；格式说明只在
//!   `scripts/prune-server/prune_server_client.py` 文件头「键空间」大值分段那一段）：写前调 `write_segments_of_large_values`
//!   （读版本登记键、删过期的版本、写登记键与这一版的段，段键带整值的 sha256、不就地覆写），再写它交回的清单；写方死在写清单之前，
//!   原键里的清单与它点名的段都没动，读回的照旧是写之前的值。读用 `read_assembling_segmented_values` 拼回整值、核 sha256，
//!   拼不回回 `ClientFailure::SegmentedValue`（退 3），不当成键不在。弄坏开关 segments-overwritten-in-place 退回改前：段键不带版本、就地覆写。
//!
//! 调用方照 `ClientFailure::exit_code` 退出。退出码表（不与 task-server 已用的 77、78、231、233 这几个撞）：
//!   75  prune-server 不可用：服务有意停了（「服务停了」）、重连到上限（「连不上」）、同一帧忙到上限、同一写帧连着回存储出错到重连次数上限。
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
use crate::gpu_judging_rows::{
    gpu_claim_key, gpu_pending_key, GpuClaimHolder, GpuConclusion, GpuPendingRow,
    GPU_REJUDGED_FAMILY_PREFIX,
};
use crate::journal::{Journal, JournalOptions, JournalStatistics};
use crate::keepalive::{enable_keepalive, read_keepalive, KeepaliveSettings};
use crate::protocol::{
    begin_point_read_frame, begin_write_frame, begin_write_if_absent_frame, decode_response_body,
    encode_claim_frame, encode_delete_prefix_frame, encode_guarded_conclusion_write_frame,
    encode_list_claims_frame, encode_prefix_scan_frame_after, encode_release_claims_frame,
    encode_statistics_frame, is_frame_length_acceptable, replace_claim_lease,
    replace_request_sequence, ClaimOutcome, ClaimReleaseOutcome, ClientAnnouncement, GuardRefusal,
    GuardedConclusionWrite, ListedClaim, PointReadFrameBuilder, RejectionStatus, Response,
    ResponseOutcome, ScanEntry, StatisticsCounter, Table, WriteFrameBuilder,
    CONNECTION_LEVEL_SEQUENCE, FRAME_LENGTH_FIELD_BYTES, MAXIMUM_CLIENT_IDENTITY_BYTES,
    MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME, MAXIMUM_FRAME_BODY_BYTES, RESPONSE_HEADER_BYTES,
};
use crate::segmented_values::{
    assemble_rereading_the_original_key, current_version_of_stored, frame_ranges,
    parse_version_registry, plan_segmented_write, segment_key, segment_version_prefix,
    version_registry_keys_to_read_before_writing, SegmentManifest, SegmentPlacement,
    SegmentedValueBeingAssembled, SegmentedValueFailure, SegmentsReadByPoint, StoredSegmentsReader,
    StoredVersions, SEGMENTED_FRAME_ENTRY_LIMIT, SEGMENT_KEY_TAIL_BYTES, SEGMENT_READ_FRAME_BYTES,
    SEGMENT_READ_OVERHEAD_BYTES_PER_KEY,
};

mod claims;
mod reads_and_scans;
mod reconnection_and_resend;
mod response_handling;
mod segmented_values;
mod writes_and_acknowledgements;

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
    /// 同一写帧连着回存储出错到了重连次数上限（文件头「存储出错」）；那一帧留在追加日志里。
    StorageFailing {
        request_number: u64,
        attempts: u32,
        last_error: String,
    },
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
    /// 大值分段写不成、读不回（`segmented_values`）：原键是段键或版本登记键的形状、段数超了段号，版本登记键或清单解不开，段缺了、长度或 sha256 对不上。
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
            | ClientFailure::StorageFailing { .. }
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
            ClientFailure::StorageFailing {
                request_number,
                attempts,
                last_error,
            } => write!(
                formatter,
                "服务的存储一直出错：写帧（请求号 {request_number}）连着 {attempts} 次回存储出错，到了重连次数上限，最后一次：{last_error}；\
                 那一帧留在追加日志里，服务修好之后重交任务时重放"
            ),
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
    /// 放认领的结局，逐键一个（`protocol` 文件头「放认领」）。
    ClaimsReleased {
        operation: OperationIdentifier,
        outcomes: Vec<ClaimReleaseOutcome>,
    },
    /// 列认领的一帧（`protocol` 文件头「列认领」）。
    ClaimListing {
        operation: OperationIdentifier,
        is_truncated: bool,
        claims: Vec<ListedClaim>,
    },
    /// 读、扫、认领、统计、删前缀没做成（被拒、忙到上限）；写没做成是整个客户端的错，从 `poll` 直接回。
    Failed {
        operation: OperationIdentifier,
        failure: ClientFailure,
    },
    /// 写结论守卫不过（服务回状态 7，文件头「写结论」）：这一帧一条都没写，已从追加日志里勾掉。
    ConclusionRefused {
        operation: OperationIdentifier,
        request_number: u64,
        refusal: GuardRefusal,
        message: String,
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
            | Completion::ClaimsReleased { operation, .. }
            | Completion::ClaimListing { operation, .. }
            | Completion::Failed { operation, .. }
            | Completion::ConclusionRefused { operation, .. } => *operation,
        }
    }
}

/// 列认领一帧的结局（[`ReliableClient::list_claims_after`]）：按键升序的认领，被条数上限或帧长上限截断时 `is_truncated` 为真。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClaimListingFrame {
    pub is_truncated: bool,
    pub claims: Vec<ListedClaim>,
}

/// [`ReliableClient::list_claims`] 一帧要几条：几十份登记与几千块的认领都在几帧之内列完。
pub const CLAIM_LISTING_ENTRIES_PER_FRAME: u32 = 1_000;

/// 写一帧不在才写或写结论落下之后服务回的确认。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WriteAcknowledgement {
    /// 真写了几条（不在才写只数不在而写了的）。
    pub entries_written: u32,
    /// 这个请求号以前生效过，这一次没再写（重连重发落下的）。
    pub is_already_applied: bool,
}

/// 写结论一帧的结局（文件头「写结论」）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GuardedConclusionWriteOutcome {
    Written(WriteAcknowledgement),
    /// 守卫不过，一条都没写：认领被别的持有者持着，或守卫那一行、一把删键那一行此刻的样子与写的一方读到的不同（带服务读到的那一份）。
    Refused {
        refusal: GuardRefusal,
        message: String,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OperationKind {
    Write {
        request_number: u64,
    },
    Read,
    Scan,
    /// 认领：交给客户端那一刻与租约毫秒数；每次发出去都给整段租约（文件头「认领」），弄坏开关 claim-lease-shrunk-on-resend 下只给剩下的（[`claim_lease_left`]）。
    Claim {
        lease_milliseconds: u32,
        submitted_at: Instant,
    },
    Statistics,
    /// 删前缀：不进追加日志（删本来就幂等，不去重，重发只会让某一帧报的数少一批，`protocol.rs` 文件头「删前缀」），断线重连之后照样重发。
    DeletePrefix,
    /// 放认领：同删前缀，不进追加日志、断线重连之后照样重发（放本来就幂等，`protocol.rs` 文件头「放认领」）。
    ReleaseClaims,
    /// 列认领：只读，不进追加日志，断线重连之后照样重发（`protocol.rs` 文件头「列认领」）。
    ListClaims,
}

/// 弄坏开关 claim-lease-shrunk-on-resend 下认领每次发出去给的租约（改前的写法，文件头「认领」）：原租约减去从交给客户端起过了的毫秒数，至少 1 毫秒。
/// 忙了大半租期才落下、重发才落下的认领拿到的到期就只剩这一点，调用方却从回来那一刻起按整段租约写。
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
    /// 写帧回了存储出错，到 `due` 用同一个请求号再交（文件头「存储出错」）。
    AwaitingStorageRetry {
        due: Instant,
    },
}

struct Operation {
    kind: OperationKind,
    frame: Vec<u8>,
    busy_retries: u32,
    /// 这一帧连着回存储出错的次数（只有写帧会有）。
    storage_failures: u32,
    state: OperationState,
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
    /// 写帧回存储出错、照同一请求号再交的次数（文件头「存储出错」）。
    pub write_storage_failures: u64,
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
                storage_failures: 0,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 弄坏开关 claim-lease-shrunk-on-resend 退回的改前写法给剩下的租约：过了 3 秒的 10 秒租约只给 7 秒，过完了的给 1 毫秒，刚交的照原租约。
    /// 平常每次发出去都给整段租约（文件头「认领」），这一份只在那个开关下用；开关证红在自证 claim-lease-covers-the-holder-after-the-answer。
    /// 判别力（`crates/mutations.tsv`）：不减去过了的毫秒数，这条红。
    #[test]
    fn resent_claim_gets_only_the_lease_left_since_it_was_submitted() {
        assert_eq!(claim_lease_left(10_000, Duration::from_secs(3)), 7_000);
        assert_eq!(
            claim_lease_left(10_000, Duration::from_secs(30)),
            1,
            "过完了的至少给 1 毫秒"
        );
        assert_eq!(claim_lease_left(10_000, Duration::ZERO), 10_000);
    }
}
