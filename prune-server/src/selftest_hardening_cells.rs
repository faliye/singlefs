//! 自证里加固的十一格（格表在 `selftest.rs` 的 `CELLS`）：追加日志的请求号高水位与排他锁、同步 WAL 失败之后不回确认、
//! 对端不读时回应受额度管、没收完的帧受额度与期限管、认领回应不越过帧长上限、建连受重连总时长上限管、
//! 一帧大过帧长上限的当场报协议错；认领回到调用方手里之后到期盖住整段租期、写帧回存储出错照同一请求号重放、
//! 删前缀的回应先记进额度（records/2026-11-06-prune-server全量代码审核.md 的四项里前三项）；工作线程边编回应边记额度
//! （records/2026-11-03-task-server与prune-server深入对攻审核.md 的 PS-04）。
//! 每格起的服务与客户端只带这一格要证红的那一项弄坏开关（`BreakSwitches::only`），别的格的开关不碰这一格。
//! 会卡住的格（弄坏开关打开时客户端堵在建连或等回应里）都有限时：到点置停标记、放掉堵着的连接，这一格判红。

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use crate::bench::SERVER_READY_TIMEOUT;
use crate::break_switches::{BreakSwitch, BreakSwitches, BREAK_VARIABLE};
use crate::client::{random_client_identity, RouteAddress, SimpleClient};
use crate::connections::UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION;
use crate::fault_log::FAULT_LOG_FILE_NAME;
use crate::fault_proxy::{FaultProxy, ProxyListen};
use crate::journal::{Journal, JournalOptions};
use crate::protocol::{
    begin_point_read_frame, begin_write_frame, decode_response_body, encode_claim_frame,
    encode_delete_prefix_frame, held_by_other_outcome_bytes, maximum_response_payload_bytes,
    replace_request_sequence, ClaimOutcome, RejectionStatus, ResponseOutcome, Table,
    CLAIM_OUTCOME_COUNT_BYTES, FRAME_LENGTH_FIELD_BYTES, MAXIMUM_CLIENT_IDENTITY_BYTES,
    MAXIMUM_FRAME_BODY_BYTES,
};
use crate::reliable_client::{
    ClientFailure, Completion, OperationIdentifier, ReconnectPolicy, ReliableClient,
    ReliableClientSettings,
};
use crate::selftest::{
    bytes, counter, loopback_arguments, server_command, CellContext, CellDirectory,
};
use crate::selftest_fault_cells::within_deadline;
use crate::server::{
    BUSY_REJECTIONS_COUNTER, CONNECTIONS_REFUSED_COUNTER, FRAME_READ_TIMEOUTS_COUNTER,
    LISTENING_LINE_PREFIX, OUTBOUND_QUEUED_BYTES_PEAK_COUNTER, PENDING_BYTES_COUNTER,
    PENDING_BYTES_PEAK_COUNTER, RECEIVING_BODY_BYTES_PEAK_COUNTER, STORAGE_SYNC_FAILED_EXIT_CODE,
    WORKER_RESPONSE_BYTES_PEAK_COUNTER, WRITE_TIMEOUTS_COUNTER,
};
use crate::spawned::{SpawnedProcess, StandardError};

mod claim_lease_after_the_answer;
mod claim_response_and_oversize_frame_limits;
mod connect_bounded_by_reconnect_total;
mod journal_high_water_and_lock;
mod pending_budgets_of_responses;
mod storage_failure_replay;
mod unread_responses_and_incomplete_frames;
mod write_ahead_log_sync_failure;

pub(crate) use self::claim_lease_after_the_answer::*;
pub(crate) use self::claim_response_and_oversize_frame_limits::*;
pub(crate) use self::connect_bounded_by_reconnect_total::*;
pub(crate) use self::journal_high_water_and_lock::*;
pub(crate) use self::pending_budgets_of_responses::*;
pub(crate) use self::storage_failure_replay::*;
pub(crate) use self::unread_responses_and_incomplete_frames::*;
pub(crate) use self::write_ahead_log_sync_failure::*;

/// 等一帧写确认的上限。
const WRITE_ACKNOWLEDGEMENT_TIMEOUT: Duration = Duration::from_secs(60);
/// 同步 WAL 失败之后等服务自己退出的上限。
const SYNC_FAILURE_EXIT_TIMEOUT: Duration = Duration::from_secs(20);
/// 慢连接被关之后额度里还许记着的字节（统计那一帧自己的请求与回应）。
const RELEASED_PENDING_BYTES_SLACK: u64 = 64 * 1024;

/// 一格起的服务：socket、状态目录、库都在这一格自己的目录里，可以退出了再起（同一份库）。
/// 字段次序就是放手次序：先停服务，再删目录。
struct HardeningServer {
    process: Option<SpawnedProcess>,
    directory: CellDirectory,
    switches: BreakSwitches,
    extra: Vec<String>,
    fdatasync_failure: Option<FdatasyncFailurePreload>,
}

/// 真实 fdatasync 失败那条输入给服务装的包装（`LD_PRELOAD`，源码 `FDATASYNC_FAILURE_WRAPPER_TEMPLATE`）：
/// `armed` 在时，这一格库目录里 `.log` 结尾的文件（RocksDB 的 WAL）的 fsync 与 fdatasync 回 EIO。
struct FdatasyncFailurePreload {
    wrapper: PathBuf,
    armed: PathBuf,
}

impl HardeningServer {
    fn start(
        context: &CellContext,
        switches: BreakSwitches,
        extra: &[&str],
    ) -> Result<HardeningServer, String> {
        HardeningServer::start_with_fdatasync_failure(context, switches, extra, None)
    }

    fn start_with_fdatasync_failure(
        context: &CellContext,
        switches: BreakSwitches,
        extra: &[&str],
        fdatasync_failure: Option<FdatasyncFailurePreload>,
    ) -> Result<HardeningServer, String> {
        let mut server = HardeningServer {
            process: None,
            directory: CellDirectory::create()?,
            switches,
            extra: extra
                .iter()
                .map(|argument| (*argument).to_string())
                .collect(),
            fdatasync_failure,
        };
        server.spawn(context)?;
        Ok(server)
    }

    fn spawn(&mut self, context: &CellContext) -> Result<(), String> {
        let library = self.directory.path.join("library");
        let mut command = server_command(context, &self.directory.path, &library)?;
        command
            .args(loopback_arguments())
            .args(&self.extra)
            .env(BREAK_VARIABLE, self.switches.environment_text());
        if let Some(preload) = &self.fdatasync_failure {
            // 包装按 /proc/self/fd 读回的真实路径比，库目录给真实路径、带结尾的斜杠（不把同名前缀的别的目录算进来）。
            let real_directory = std::fs::canonicalize(&self.directory.path).map_err(|error| {
                format!(
                    "取不了 {} 的真实路径：{error}",
                    self.directory.path.display()
                )
            })?;
            command
                .env("LD_PRELOAD", &preload.wrapper)
                .env(FDATASYNC_ARMED_VARIABLE, &preload.armed)
                .env(
                    FDATASYNC_LIBRARY_VARIABLE,
                    format!("{}/", real_directory.join("library").display()),
                );
        }
        let mut process = SpawnedProcess::spawn(command, StandardError::Capture)?;
        process
            .wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)
            .map_err(|message| {
                format!(
                    "服务没起来：{message}；标准错误：{}",
                    process.standard_error_text()
                )
            })?;
        self.process = Some(process);
        Ok(())
    }

    fn socket_path(&self) -> PathBuf {
        self.directory.path.join("server.sock")
    }

    fn route(&self) -> RouteAddress {
        RouteAddress::UnixSocket(self.socket_path())
    }

    fn path(&self) -> &Path {
        &self.directory.path
    }

    fn wait_for_exit(&mut self, timeout: Duration) -> Option<ExitStatus> {
        self.process
            .as_mut()
            .and_then(|process| process.wait_for_exit(timeout))
    }

    fn standard_error_text(&mut self) -> String {
        self.process
            .as_mut()
            .map(SpawnedProcess::standard_error_text)
            .unwrap_or_default()
    }
}

/// 一帧写一条结论表的键值（序号写 0，发的时候换）。
fn write_frame(identity: &[u8], request_number: u64, key: &str, value: &str) -> Vec<u8> {
    let mut builder =
        begin_write_frame(0, identity, request_number, 1, 9 + key.len() + value.len());
    builder.entry(Table::Conclusions, key.as_bytes(), value.as_bytes());
    builder.finish()
}

/// 照原样发一帧（序号换成这条连接的下一个），等它的回应。
fn send_frame(client: &mut SimpleClient, frame: &[u8]) -> Result<ResponseOutcome, String> {
    client.call(|sequence| {
        let mut copy = frame.to_vec();
        replace_request_sequence(&mut copy, sequence);
        copy
    })
}

fn read_value(route: &RouteAddress, key: &str) -> Result<Option<Vec<u8>>, String> {
    let mut reader = SimpleClient::connect(route, BreakSwitches::none())?;
    Ok(reader
        .read(&[(Table::Conclusions, bytes(key))])?
        .into_iter()
        .next()
        .flatten())
}

fn shown_value(value: Option<&[u8]>) -> String {
    value.map_or_else(
        || "（不在）".to_string(),
        |present| String::from_utf8_lossy(present).into_owned(),
    )
}

/// 等 `operation` 那一帧写的确认；回服务说的「早已生效」。
fn wait_write_acknowledgement(
    client: &mut ReliableClient,
    operation: OperationIdentifier,
) -> Result<bool, String> {
    let deadline = Instant::now() + WRITE_ACKNOWLEDGEMENT_TIMEOUT;
    loop {
        let completions = client
            .poll(Duration::from_millis(100))
            .map_err(|failure| format!("等写确认出错：{failure}"))?;
        for completion in completions {
            if let Completion::WriteAcknowledged {
                operation: acknowledged,
                is_already_applied,
                ..
            } = completion
            {
                if acknowledged == operation {
                    return Ok(is_already_applied);
                }
            }
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "{} 秒没等到写确认",
                WRITE_ACKNOWLEDGEMENT_TIMEOUT.as_secs()
            ));
        }
    }
}

fn plain_client_settings(
    route: RouteAddress,
    journal_path: PathBuf,
    switches: &BreakSwitches,
) -> ReliableClientSettings {
    let mut settings = ReliableClientSettings::new(route, journal_path);
    settings.frame_timeout = Duration::from_secs(30);
    settings.break_switches = switches.clone();
    settings
}

fn entry(key: &str, value: &str) -> (Table, Vec<u8>, Vec<u8>) {
    (Table::Conclusions, bytes(key), bytes(value))
}
