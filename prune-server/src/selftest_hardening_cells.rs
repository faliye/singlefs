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

pub(crate) fn cell_journal_high_water_survives_compaction(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::JournalCompactionDropsHighWater);
    let server = HardeningServer::start(context, BreakSwitches::none(), &[])?;
    let full = high_water_after_full_acknowledgement(&server, &switches)?;
    let partial = high_water_with_lower_request_pending(&server, &switches)?;
    Ok(format!("{full}；{partial}"))
}

/// 全确认之后整理、重开再写新键：新写要分到没用过的请求号、落库。
fn high_water_after_full_acknowledgement(
    server: &HardeningServer,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let mut settings = plain_client_settings(
        server.route(),
        server.path().join("full-acknowledgement.journal"),
        switches,
    );
    settings.journal_compaction_bytes = 1;
    let request_before_reopen = {
        let mut client =
            ReliableClient::open(settings.clone()).map_err(|failure| failure.to_string())?;
        let (_, request_number) = client
            .submit_write_entries(&[entry("high-water/full/before", "before")])
            .map_err(|failure| failure.to_string())?;
        client
            .wait_all_acknowledged()
            .map_err(|failure| failure.to_string())?;
        if client.journal_statistics().compactions == 0 {
            return Err(
                "整理门槛 1 字节、确认之后该整理过，整理了 0 次：没走到「整理之后重开」的局面"
                    .to_string(),
            );
        }
        request_number
    };
    let mut reopened = ReliableClient::open(settings).map_err(|failure| failure.to_string())?;
    let (operation, request_number) = reopened
        .submit_write_entries(&[entry("high-water/full/after", "after")])
        .map_err(|failure| failure.to_string())?;
    let is_already_applied = wait_write_acknowledgement(&mut reopened, operation)?;
    let stored = read_value(&server.route(), "high-water/full/after")?;
    if request_number <= request_before_reopen
        || is_already_applied
        || stored.as_deref() != Some(b"after".as_slice())
    {
        return Err(format!(
            "全确认之后整理、重开：新写分到请求号 {request_number}（整理之前用到 {request_before_reopen}），服务回「早已生效」{is_already_applied}，\
             库里读回 {}（该是 after）：整理丢了请求号高水位，新写被当成早已生效的重发",
            shown_value(stored.as_deref())
        ));
    }
    Ok(format!(
        "全确认之后整理、重开：新写分到请求号 {request_number}（整理之前用到 {request_before_reopen}），服务照新写落库"
    ))
}

/// 低号没确认、高号确认了之后整理、重开：新写要分到比高号还大的请求号、落库。
fn high_water_with_lower_request_pending(
    server: &HardeningServer,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let journal_path = server.path().join("lower-pending.journal");
    let recovered = Journal::open(
        &journal_path,
        JournalOptions::from_break_switches(1, switches),
    )?;
    let mut journal = recovered.journal;
    let identity = recovered.client_identity;
    let pending_frame = write_frame(&identity, 1, "high-water/partial/pending", "pending");
    let acknowledged_frame = write_frame(
        &identity,
        2,
        "high-water/partial/acknowledged",
        "acknowledged",
    );
    journal.append_write(1, &pending_frame)?;
    journal.append_write(2, &acknowledged_frame)?;
    let mut direct = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    let first_write = send_frame(&mut direct, &acknowledged_frame)?;
    if !matches!(
        first_write,
        ResponseOutcome::WriteAcknowledged {
            is_already_applied: false,
            ..
        }
    ) {
        return Err(format!(
            "请求号 2 头一次写该回做完、没生效过，回的是 {first_write:?}"
        ));
    }
    journal.append_acknowledgement(2, std::iter::once((1, pending_frame.as_slice())))?;
    if journal.statistics().compactions != 1 {
        return Err(format!(
            "确认请求号 2 时该整理一次（门槛 1 字节），整理了 {} 次：没走到「低号没确认、高号确认了」之后整理的局面",
            journal.statistics().compactions
        ));
    }
    drop(journal);
    let mut settings = plain_client_settings(server.route(), journal_path, switches);
    settings.journal_compaction_bytes = 1;
    let mut reopened = ReliableClient::open(settings).map_err(|failure| failure.to_string())?;
    let (operation, request_number) = reopened
        .submit_write_entries(&[entry("high-water/partial/after", "after")])
        .map_err(|failure| failure.to_string())?;
    let is_already_applied = wait_write_acknowledgement(&mut reopened, operation)?;
    reopened
        .wait_all_acknowledged()
        .map_err(|failure| failure.to_string())?;
    let after = read_value(&server.route(), "high-water/partial/after")?;
    let pending = read_value(&server.route(), "high-water/partial/pending")?;
    if request_number <= 2
        || is_already_applied
        || after.as_deref() != Some(b"after".as_slice())
        || pending.as_deref() != Some(b"pending".as_slice())
    {
        return Err(format!(
            "请求号 1 没确认、2 确认了之后整理、重开：新写分到请求号 {request_number}（该大于 2），服务回「早已生效」{is_already_applied}，\
             新键读回 {}（该是 after），重放的请求号 1 读回 {}（该是 pending）",
            shown_value(after.as_deref()),
            shown_value(pending.as_deref())
        ));
    }
    Ok(format!(
        "低号没确认、高号确认了之后整理、重开：重放请求号 1，新写分到请求号 {request_number}，两条都落库"
    ))
}

fn expect_storage_failed(outcome: &ResponseOutcome, what: &str) -> Result<(), String> {
    if matches!(
        outcome,
        ResponseOutcome::Rejected {
            status: RejectionStatus::StorageFailed,
            ..
        }
    ) {
        return Ok(());
    }
    Err(format!("{what}该回存储出错（状态 4），回的是 {outcome:?}"))
}

/// 服务该自己退出、退 `STORAGE_SYNC_FAILED_EXIT_CODE`；没退的时候在同一条连接上重发同一请求，把回的什么写进说明。
fn expect_sync_failure_exit(
    server: &mut HardeningServer,
    client: &mut SimpleClient,
    frame: &[u8],
    what: &str,
) -> Result<(), String> {
    match server.wait_for_exit(SYNC_FAILURE_EXIT_TIMEOUT) {
        Some(status) if status.code() == Some(STORAGE_SYNC_FAILED_EXIT_CODE) => Ok(()),
        Some(status) => Err(format!(
            "{what}：服务该退 {STORAGE_SYNC_FAILED_EXIT_CODE}，退了 {status}；标准错误：{}",
            server.standard_error_text()
        )),
        None => {
            let retried = send_frame(client, frame);
            Err(format!(
                "{what}：服务 {} 秒没退出、照常服务；同步故障没解除时重发同一请求回 {retried:?}（确认不能早于落盘，该回存储出错、服务退出）",
                SYNC_FAILURE_EXIT_TIMEOUT.as_secs()
            ))
        }
    }
}

/// 编真实 fdatasync 失败的包装用的 C 编译器（命令名或路径）；不设就不跑那条输入，这一格的说明里写「本次未跑」。
const WRAPPER_COMPILER_VARIABLE: &str = "PRUNE_SERVER_SELFTEST_WRAPPER_COMPILER";
/// 包装读的两个环境变量：哪个文件在时造失败、库目录（真实路径，带结尾的斜杠）。
const FDATASYNC_ARMED_VARIABLE: &str = "PRUNE_SERVER_SELFTEST_FDATASYNC_ARMED";
const FDATASYNC_LIBRARY_VARIABLE: &str = "PRUNE_SERVER_SELFTEST_FDATASYNC_LIBRARY";
/// 包装每造一次失败往服务的标准错误打一行：这个开头、空格、失败的文件。
const FDATASYNC_FAILED_LINE_PREFIX: &str = "PRUNE_SERVER_SELFTEST_FDATASYNC_FAILED";

/// 真实 fdatasync 失败的包装（照 records/2026-11-01-prune-server深入审核.md 附录 `/tmp/singlefs-review-sync.c`，只换了
/// 判哪些文件的写法与打的行）：`@ARMED@`、`@LIBRARY@`、`@FAILED@` 写源码时换成上面三个常量。
/// 包住的是 libc 的 fsync 与 fdatasync，RocksDB 同步 WAL（`flush_wal(true)` 里 `PosixWritableFile::Sync`）调的就是它们：
/// 写批照常写进 WAL 文件（进了页缓存），同步回 EIO，失败从 RocksDB 里一路回到 `storage.rs` 的 `flush_and_synchronize`。
const FDATASYNC_FAILURE_WRAPPER_TEMPLATE: &str = r#"#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int should_fail(int descriptor) {
    const char *armed = getenv("@ARMED@");
    const char *library = getenv("@LIBRARY@");
    char link[64];
    char target[4096];
    ssize_t length;
    if (armed == NULL || library == NULL || access(armed, F_OK) != 0) {
        return 0;
    }
    snprintf(link, sizeof link, "/proc/self/fd/%d", descriptor);
    length = readlink(link, target, sizeof target - 1);
    if (length < 4) {
        return 0;
    }
    target[length] = '\0';
    if (strncmp(target, library, strlen(library)) != 0 || strcmp(target + length - 4, ".log") != 0) {
        return 0;
    }
    fprintf(stderr, "@FAILED@ %s\n", target);
    errno = EIO;
    return 1;
}

int fsync(int descriptor) {
    int (*real_fsync)(int);
    if (should_fail(descriptor)) {
        return -1;
    }
    real_fsync = (int (*)(int))dlsym(RTLD_NEXT, "fsync");
    return real_fsync(descriptor);
}

int fdatasync(int descriptor) {
    int (*real_fdatasync)(int);
    if (should_fail(descriptor)) {
        return -1;
    }
    real_fdatasync = (int (*)(int))dlsym(RTLD_NEXT, "fdatasync");
    return real_fdatasync(descriptor);
}
"#;

/// 编包装：源码与 `.so` 都写进 `directory`；回 `.so` 的真实路径。
fn compile_fdatasync_failure_wrapper(
    compiler: &std::ffi::OsStr,
    directory: &Path,
) -> Result<PathBuf, String> {
    let source = directory.join("fdatasync-failure.c");
    let wrapper = directory.join("fdatasync-failure.so");
    let text = FDATASYNC_FAILURE_WRAPPER_TEMPLATE
        .replace("@ARMED@", FDATASYNC_ARMED_VARIABLE)
        .replace("@LIBRARY@", FDATASYNC_LIBRARY_VARIABLE)
        .replace("@FAILED@", FDATASYNC_FAILED_LINE_PREFIX);
    std::fs::write(&source, text)
        .map_err(|error| format!("写不了 {}：{error}", source.display()))?;
    let compiled = Command::new(compiler)
        .args(["-shared", "-fPIC", "-o"])
        .arg(&wrapper)
        .arg(&source)
        .arg("-ldl")
        .output()
        .map_err(|error| {
            format!(
                "起不了 {WRAPPER_COMPILER_VARIABLE} 指的 C 编译器 {}：{error}",
                compiler.to_string_lossy()
            )
        })?;
    if !compiled.status.success() {
        return Err(format!(
            "{} 编不出真实 fdatasync 失败的包装（{}）：{}",
            compiler.to_string_lossy(),
            compiled.status,
            String::from_utf8_lossy(&compiled.stderr)
        ));
    }
    std::fs::canonicalize(&wrapper)
        .map_err(|error| format!("取不了 {} 的真实路径：{error}", wrapper.display()))
}

/// 这一格的真实 fdatasync 失败那条输入（设了 `WRAPPER_COMPILER_VARIABLE` 才跑）：服务装上包装，同步 WAL 时 libc 的 fdatasync 真回 EIO。
/// 判：写回存储出错、服务退 7、标准错误里有包装造失败的行（失败真是 fdatasync 回的，不是别处）；撤掉之后重起、重发回做完、
/// 读得到，再重发回「早已生效」。故障没解除时重起那一步只在内建造错点那条输入上做：真实失败下重起时 RocksDB 把从 WAL 文件
/// 读回的那一批落成 SST 并同步（不经 `.log` 的 fdatasync），重发回早已生效还是存储出错取决于内核留没留住写失败的那几页，没有固定的结局。
fn real_fdatasync_failure_input(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let Some(compiler) =
        std::env::var_os(WRAPPER_COMPILER_VARIABLE).filter(|value| !value.is_empty())
    else {
        return Ok(format!(
            "真实 fdatasync 失败那条输入本次未跑（没设 {WRAPPER_COMPILER_VARIABLE}）"
        ));
    };
    let wrapper_directory = CellDirectory::create()?;
    let wrapper = compile_fdatasync_failure_wrapper(&compiler, &wrapper_directory.path)?;
    let armed = wrapper_directory.path.join("fdatasync-armed");
    let mut server = HardeningServer::start_with_fdatasync_failure(
        context,
        switches.clone(),
        &[],
        Some(FdatasyncFailurePreload {
            wrapper,
            armed: armed.clone(),
        }),
    )?;
    let identity = random_client_identity()?;
    let frame = write_frame(&identity, 1, "sync-failure/real-fdatasync", "real");
    std::fs::write(&armed, b"").map_err(|error| format!("建不了 {}：{error}", armed.display()))?;
    let mut failing_client = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    expect_storage_failed(
        &send_frame(&mut failing_client, &frame)?,
        "真实 fdatasync 失败的那一次写",
    )?;
    expect_sync_failure_exit(
        &mut server,
        &mut failing_client,
        &frame,
        "真实 fdatasync 失败之后",
    )?;
    let standard_error = server.standard_error_text();
    let failed_files: Vec<&str> = standard_error
        .lines()
        .filter_map(|line| line.strip_prefix(FDATASYNC_FAILED_LINE_PREFIX))
        .map(str::trim)
        .collect();
    let Some(failed_file) = failed_files.first() else {
        return Err(format!(
            "写回了存储出错、服务退了 {STORAGE_SYNC_FAILED_EXIT_CODE}，服务的标准错误里却没有包装造失败的行（{FDATASYNC_FAILED_LINE_PREFIX}）：\
             失败不是真实 fdatasync 回的；标准错误：{standard_error}"
        ));
    };
    let failure_summary = format!("包装造了 {} 次失败（{failed_file}）", failed_files.len());
    std::fs::remove_file(&armed).map_err(|error| format!("删不了 {}：{error}", armed.display()))?;
    server.spawn(context)?;
    let mut client = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    let recovered = send_frame(&mut client, &frame)?;
    let ResponseOutcome::WriteAcknowledged {
        entries_written: 1,
        is_already_applied: recovered_from_write_ahead_log,
    } = &recovered
    else {
        return Err(format!(
            "真实 fdatasync 失败、撤掉包装造的失败、服务重起之后重发同一请求该回做完，回的是 {recovered:?}"
        ));
    };
    let stored = read_value(&server.route(), "sync-failure/real-fdatasync")?;
    if stored.as_deref() != Some(b"real".as_slice()) {
        return Err(format!(
            "真实 fdatasync 失败、恢复之后确认了的写库里读回 {}（该是 real）",
            shown_value(stored.as_deref())
        ));
    }
    let repeated = send_frame(&mut client, &frame)?;
    if !matches!(
        repeated,
        ResponseOutcome::WriteAcknowledged {
            is_already_applied: true,
            ..
        }
    ) {
        return Err(format!(
            "真实 fdatasync 失败、恢复且确认过之后再重发同一请求该回「早已生效」，回的是 {repeated:?}"
        ));
    }
    let recovery = if *recovered_from_write_ahead_log {
        "重起时 RocksDB 从 WAL 文件读回了没同步成的那一批，重发回早已生效"
    } else {
        "重起时 WAL 文件里没有那一批，重发重新写、同步之后回确认"
    };
    Ok(format!(
        "真实 fdatasync 失败（{WRAPPER_COMPILER_VARIABLE}={} 编的 LD_PRELOAD 包装）：RocksDB 同步 WAL 回错、写回存储出错、服务退 {STORAGE_SYNC_FAILED_EXIT_CODE}，\
         {failure_summary}；撤掉之后{recovery}、读得到，再重发回「早已生效」",
        compiler.to_string_lossy()
    ))
}

/// 两条输入：内建造错点（`--selftest-sync-failure-file`，写批写进去之后不调 `flush_wal`、当成同步失败，不要 C 编译器，每次都跑）
/// 判服务拿到同步失败之后怎么办；真实 fdatasync 失败（`real_fdatasync_failure_input`，设了 `WRAPPER_COMPILER_VARIABLE` 才跑）
/// 判 RocksDB 碰上真的 fdatasync EIO 时把失败回给服务、走的是同一条退 7 的路。
pub(crate) fn cell_write_ahead_log_sync_failure_never_acknowledged(
    context: &CellContext,
) -> Result<String, String> {
    // 真实那条先跑：弄坏开关 sync-failure-kept-serving 打开、又设了 C 编译器时，红在真实那条上，证它判得出；不设时红在内建那条上。
    let real_failure = real_fdatasync_failure_input(
        context,
        &context
            .break_switches
            .only(BreakSwitch::SyncFailureKeptServing),
    )?;
    let arm_directory = CellDirectory::create()?;
    let arm = arm_directory.path.join("sync-failure-armed");
    let arm_argument = arm.display().to_string();
    let mut server = HardeningServer::start(
        context,
        context
            .break_switches
            .only(BreakSwitch::SyncFailureKeptServing),
        &["--selftest-sync-failure-file", &arm_argument],
    )?;
    let identity = random_client_identity()?;
    let frame = write_frame(&identity, 1, "sync-failure/key", "value");
    std::fs::write(&arm, b"").map_err(|error| format!("建不了 {arm_argument}：{error}"))?;
    let mut failing_client = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    expect_storage_failed(
        &send_frame(&mut failing_client, &frame)?,
        "同步 WAL 失败的那一次写",
    )?;
    expect_sync_failure_exit(
        &mut server,
        &mut failing_client,
        &frame,
        "同步 WAL 失败之后",
    )?;
    server.spawn(context)?;
    let mut armed_client = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    expect_storage_failed(
        &send_frame(&mut armed_client, &frame)?,
        "同步故障没解除、服务重起之后重发同一请求",
    )?;
    expect_sync_failure_exit(&mut server, &mut armed_client, &frame, "重起之后同步又失败")?;
    std::fs::remove_file(&arm).map_err(|error| format!("删不了 {arm_argument}：{error}"))?;
    server.spawn(context)?;
    let mut client = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    let recovered = send_frame(&mut client, &frame)?;
    if !matches!(
        recovered,
        ResponseOutcome::WriteAcknowledged {
            entries_written: 1,
            ..
        }
    ) {
        return Err(format!(
            "同步恢复、服务重起之后重发同一请求该回做完，回的是 {recovered:?}"
        ));
    }
    let stored = read_value(&server.route(), "sync-failure/key")?;
    if stored.as_deref() != Some(b"value".as_slice()) {
        return Err(format!(
            "同步恢复之后确认了的写库里读回 {}（该是 value）",
            shown_value(stored.as_deref())
        ));
    }
    let repeated = send_frame(&mut client, &frame)?;
    if !matches!(
        repeated,
        ResponseOutcome::WriteAcknowledged {
            is_already_applied: true,
            ..
        }
    ) {
        return Err(format!(
            "确认过之后再重发同一请求该回「早已生效」（幂等），回的是 {repeated:?}"
        ));
    }
    Ok(format!(
        "内建造错点造的同步 WAL 失败：写回存储出错、服务退 {STORAGE_SYNC_FAILED_EXIT_CODE}；故障没解除时重起重发同一请求照回存储出错、再退；\
         同步恢复之后重发才回做完、读得到，再重发回「早已生效」。{real_failure}"
    ))
}

/// 对端不读那一格的数：值 1 MiB、每条连接额度 4 MiB、只发不收的点读 64 次、写期限 3 秒。
const UNREAD_VALUE_BYTES: usize = 1024 * 1024;
const UNREAD_PER_CONNECTION_BYTES: usize = 4 * 1024 * 1024;
const UNREAD_SERVICE_PENDING_BYTES: usize = 64 * 1024 * 1024;
const UNREAD_READS: u64 = 64;
const UNREAD_WRITE_TIMEOUT: Duration = Duration::from_secs(3);

fn connect_raw(socket: &Path) -> Result<UnixStream, String> {
    UnixStream::connect(socket).map_err(|error| format!("连 {}：{error}", socket.display()))
}

pub(crate) fn cell_unread_responses_bounded(context: &CellContext) -> Result<String, String> {
    let service_pending = UNREAD_SERVICE_PENDING_BYTES.to_string();
    let per_connection = UNREAD_PER_CONNECTION_BYTES.to_string();
    let write_timeout = UNREAD_WRITE_TIMEOUT.as_millis().to_string();
    let server = HardeningServer::start(
        context,
        context
            .break_switches
            .only(BreakSwitch::ResponseSlotReleasedOnEnqueue),
        &[
            "--maximum-pending-bytes",
            &service_pending,
            "--maximum-pending-bytes-per-connection",
            &per_connection,
            "--write-timeout-milliseconds",
            &write_timeout,
        ],
    )?;
    let mut writer = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    writer.write(&[(
        Table::Conclusions,
        bytes("unread/big"),
        vec![b'z'; UNREAD_VALUE_BYTES],
    )])?;
    let mut slow = connect_raw(&server.socket_path())?;
    for sequence in 1..=UNREAD_READS {
        let mut builder = begin_point_read_frame(sequence, 1, 16);
        builder.key(Table::Conclusions, b"unread/big");
        slow.write_all(&builder.finish())
            .map_err(|error| format!("只发不收的连接发不出点读：{error}"))?;
        thread::sleep(Duration::from_millis(15));
    }
    thread::sleep(Duration::from_millis(300));
    let mut other = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    other
        .write(&[entry("unread/other", "still-served")])
        .map_err(|message| format!("一条连接只发不收时别的客户端写不进：{message}"))?;
    let other_value = other
        .read(&[(Table::Conclusions, bytes("unread/other"))])?
        .into_iter()
        .next()
        .flatten();
    if other_value.as_deref() != Some(b"still-served".as_slice()) {
        return Err(format!(
            "一条连接只发不收时别的客户端读回 {}（该是 still-served）",
            shown_value(other_value.as_deref())
        ));
    }
    let queued_peak = counter(&other.statistics()?, OUTBOUND_QUEUED_BYTES_PEAK_COUNTER)?;
    let queued_bound = u64::try_from(
        UNREAD_PER_CONNECTION_BYTES + UNREAD_VALUE_BYTES + UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION,
    )
    .expect("几 MiB");
    if queued_peak > queued_bound {
        return Err(format!(
            "小请求读 1 MiB 的值、只发不收 {UNREAD_READS} 次：服务写出队峰值 {queued_peak} 字节，超过每条连接的额度 {UNREAD_PER_CONNECTION_BYTES} 加一个回应加不带名额的回应上限（{queued_bound}）：回应不受额度管、在队里无界地积"
        ));
    }
    let release_deadline = Instant::now() + UNREAD_WRITE_TIMEOUT + Duration::from_secs(10);
    loop {
        let statistics = other.statistics()?;
        let write_timeouts = counter(&statistics, WRITE_TIMEOUTS_COUNTER)?;
        let pending_bytes = counter(&statistics, PENDING_BYTES_COUNTER)?;
        if write_timeouts >= 1 && pending_bytes <= RELEASED_PENDING_BYTES_SLACK {
            drop(slow);
            return Ok(format!(
                "小请求读 1 MiB 的值、只发不收 {UNREAD_READS} 次：写出队峰值 {queued_peak} 字节（上界 {queued_bound}），别的客户端照常读写；\
                 写期限 {} 秒到了慢连接被关、额度放回到 {pending_bytes} 字节",
                UNREAD_WRITE_TIMEOUT.as_secs()
            ));
        }
        if Instant::now() >= release_deadline {
            return Err(format!(
                "写期限 {} 秒过了：因写堵住关掉的连接 {write_timeouts} 条（该至少 1）、额度里还记着 {pending_bytes} 字节（该至多 {RELEASED_PENDING_BYTES_SLACK}）",
                UNREAD_WRITE_TIMEOUT.as_secs()
            ));
        }
        thread::sleep(Duration::from_millis(200));
    }
}

/// 没收完的帧那一格的数：额度 1024 字节、待处理 1 帧、四条连接各声明 16 MiB 只发 2 MiB、帧读取期限 1.5 秒、连接数上限 8。
const INCOMPLETE_PENDING_BYTES: u64 = 1024;
const INCOMPLETE_DECLARED_BODY_BYTES: u32 = 16 * 1024 * 1024;
const INCOMPLETE_SENT_BYTES: usize = 2 * 1024 * 1024;
const INCOMPLETE_FRAMES: u64 = 4;
const INCOMPLETE_FRAME_READ_TIMEOUT: Duration = Duration::from_millis(1500);
const INCOMPLETE_MAXIMUM_CONNECTIONS: usize = 8;
/// 逐字节慢发的那一帧：声明的帧体字节、每发一个字节隔多久。
const TRICKLED_BODY_BYTES: u32 = 50;
const TRICKLE_INTERVAL: Duration = Duration::from_millis(100);
/// 等服务关掉一条连接的上限。
const SERVER_CLOSE_TIMEOUT: Duration = Duration::from_secs(5);
/// 服务关掉连接之后到连接数减回去之间留的时间。
const CONNECTION_SETTLE_DELAY: Duration = Duration::from_millis(300);

/// 读到流尾（或连接被重置）算服务关了这条连接；读到点还连着回错。
fn expect_closed_by_server(stream: &mut UnixStream, what: &str) -> Result<(), String> {
    stream
        .set_read_timeout(Some(SERVER_CLOSE_TIMEOUT))
        .map_err(|error| format!("设不上读超时：{error}"))?;
    let mut chunk = [0_u8; 4096];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return Ok(()),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::ConnectionReset => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => {
                return Err(format!(
                    "{what}：{} 秒里服务没关这条连接（{error}）",
                    SERVER_CLOSE_TIMEOUT.as_secs()
                ))
            }
        }
    }
}

/// 连接数降下来之前新连接会被当场关掉：换新连接重试取统计，至多 10 秒。
fn statistics_when_admitted(
    route: &RouteAddress,
) -> Result<Vec<crate::protocol::StatisticsCounter>, String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let attempt = SimpleClient::connect(route, BreakSwitches::none())
            .and_then(|mut client| client.statistics());
        match attempt {
            Ok(statistics) => return Ok(statistics),
            Err(message) if Instant::now() >= deadline => {
                return Err(format!("10 秒里取不到统计：{message}"))
            }
            Err(_) => thread::sleep(Duration::from_millis(100)),
        }
    }
}

pub(crate) fn cell_incomplete_frames_bounded(context: &CellContext) -> Result<String, String> {
    let pending_bytes = INCOMPLETE_PENDING_BYTES.to_string();
    let frame_read_timeout = INCOMPLETE_FRAME_READ_TIMEOUT.as_millis().to_string();
    let maximum_connections = INCOMPLETE_MAXIMUM_CONNECTIONS.to_string();
    let server = HardeningServer::start(
        context,
        context
            .break_switches
            .only(BreakSwitch::FrameBodyReadBeforeAdmission),
        &[
            "--maximum-pending-bytes",
            &pending_bytes,
            "--maximum-pending-total",
            "1",
            "--frame-read-timeout-milliseconds",
            &frame_read_timeout,
            "--maximum-connections",
            &maximum_connections,
        ],
    )?;
    let socket = server.socket_path();
    let mut half_frames = Vec::new();
    for _ in 0..INCOMPLETE_FRAMES {
        let mut stream = connect_raw(&socket)?;
        stream
            .write_all(&INCOMPLETE_DECLARED_BODY_BYTES.to_le_bytes())
            .and_then(|()| stream.write_all(&vec![b'x'; INCOMPLETE_SENT_BYTES]))
            .map_err(|error| format!("发半帧出错：{error}"))?;
        half_frames.push(stream);
    }
    let mut trickled = connect_raw(&socket)?;
    let mut trickle_writer = trickled
        .try_clone()
        .map_err(|error| format!("复制不了慢发的连接：{error}"))?;
    let trickle = thread::spawn(move || {
        let mut frame = TRICKLED_BODY_BYTES.to_le_bytes().to_vec();
        frame.extend(std::iter::repeat_n(
            0_u8,
            usize::try_from(TRICKLED_BODY_BYTES).expect("50"),
        ));
        for byte in frame {
            if trickle_writer.write_all(&[byte]).is_err() {
                return;
            }
            thread::sleep(TRICKLE_INTERVAL);
        }
    });
    thread::sleep(INCOMPLETE_FRAME_READ_TIMEOUT + Duration::from_secs(1));
    for stream in &mut half_frames {
        expect_closed_by_server(stream, "声明 16 MiB、只发了 2 MiB 的连接，帧读取期限过后")?;
    }
    expect_closed_by_server(&mut trickled, "逐字节慢发的连接，帧读取期限过后")?;
    // 慢发的线程在服务关了连接之后写出错就收尾。
    let _ = trickle.join();
    // 读线程关了套接字之后才放手连接数，给那几条收尾留一点时间，免得算进下面的上限。
    thread::sleep(CONNECTION_SETTLE_DELAY);
    let mut idle = Vec::new();
    for _ in 0..INCOMPLETE_MAXIMUM_CONNECTIONS {
        idle.push(connect_raw(&socket)?);
    }
    thread::sleep(Duration::from_millis(200));
    let mut refused = connect_raw(&socket)?;
    expect_closed_by_server(&mut refused, "连接数到了上限之后接进来的连接")?;
    drop(idle);
    let statistics = statistics_when_admitted(&server.route())?;
    let receiving_peak = counter(&statistics, RECEIVING_BODY_BYTES_PEAK_COUNTER)?;
    let frame_read_timeouts = counter(&statistics, FRAME_READ_TIMEOUTS_COUNTER)?;
    let connections_refused = counter(&statistics, CONNECTIONS_REFUSED_COUNTER)?;
    let receiving_bound = u64::from(INCOMPLETE_DECLARED_BODY_BYTES) + INCOMPLETE_PENDING_BYTES;
    if receiving_peak > receiving_bound {
        return Err(format!(
            "额度 {INCOMPLETE_PENDING_BYTES} 字节、{INCOMPLETE_FRAMES} 条连接各声明 16 MiB 只发 2 MiB：服务给没收完的帧体分配的字节峰值 {receiving_peak}，\
             超过额度加一帧（{receiving_bound}）：帧体在准入之前就照声明的长度分配了"
        ));
    }
    if frame_read_timeouts < INCOMPLETE_FRAMES + 1 || connections_refused == 0 {
        return Err(format!(
            "因帧没在期限里收齐关掉的连接 {frame_read_timeouts} 条（该至少 {}）、因连接数满了拒掉的 {connections_refused} 条（该至少 1）",
            INCOMPLETE_FRAMES + 1
        ));
    }
    let mut client = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    client
        .write(&[entry("incomplete/after", "served")])
        .map_err(|message| format!("没收完的帧都被关掉之后写不进：{message}"))?;
    let stored = read_value(&server.route(), "incomplete/after")?;
    if stored.as_deref() != Some(b"served".as_slice()) {
        return Err(format!(
            "没收完的帧都被关掉之后写的键读回 {}（该是 served）",
            shown_value(stored.as_deref())
        ));
    }
    Ok(format!(
        "额度 {INCOMPLETE_PENDING_BYTES} 字节、待处理 1 帧：{INCOMPLETE_FRAMES} 条半帧连接，帧体分配峰值 {receiving_peak} 字节（上界 {receiving_bound}）；\
         半帧与逐字节慢发的连接在 {} 毫秒的期限后被关（{frame_read_timeouts} 条）；连接数满了拒掉 {connections_refused} 条；之后照常读写",
        INCOMPLETE_FRAME_READ_TIMEOUT.as_millis()
    ))
}

pub(crate) fn cell_journal_lock_survives_compaction(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::JournalLocksRenamedFile);
    let directory = CellDirectory::create()?;
    let path = directory.path.join("locked.journal");
    let compacting_options = JournalOptions::from_break_switches(1, &switches);
    let keeping_options = JournalOptions::from_break_switches(u64::MAX, &switches);
    let mut holder = Journal::open(&path, compacting_options)?.journal;
    holder.append_write(1, b"holder-frame")?;
    let (opened_sender, opened) = mpsc::channel::<()>();
    let (resume_sender, resume) = mpsc::channel::<()>();
    let racer_path = path.clone();
    let racer = thread::spawn(move || -> Result<String, String> {
        let pause = move || {
            // 主线程收不到也照样往下走：它到点会判这一格红。
            let _ = opened_sender.send(());
            let _ = resume.recv_timeout(Duration::from_secs(30));
        };
        match Journal::open_pausing_before_lock(&racer_path, keeping_options, pause) {
            Err(refusal) => Ok(refusal),
            Ok(mut recovered) => {
                recovered
                    .journal
                    .append_write(2, b"racer-frame")
                    .map_err(|message| format!("第二个打开者拿到锁之后追加出错：{message}"))?;
                Err("第二个打开者卡在开文件与加锁之间、第一个整理之后放行，它拿到了锁、追加了请求号 2".to_string())
            }
        }
    });
    let was_paused = opened.recv_timeout(Duration::from_secs(30)).is_ok();
    let compaction = holder.append_acknowledgement(1, std::iter::empty());
    let _ = resume_sender.send(());
    let racer_outcome = racer
        .join()
        .map_err(|_| "第二个打开者的线程 panic 了".to_string())?;
    compaction?;
    let compactions = holder.statistics().compactions;
    drop(holder);
    if !was_paused || compactions != 1 {
        return Err(format!(
            "第二个打开者停在加锁之前 {was_paused}（该是 true）、第一个整理了 {compactions} 次（该是 1）：没走到要判的交错"
        ));
    }
    let recovered = Journal::open(&path, keeping_options)?;
    let pending: Vec<u64> = recovered
        .unacknowledged
        .iter()
        .map(|write| write.request_number)
        .collect();
    match racer_outcome {
        Ok(refusal) if pending.is_empty() => Ok(format!(
            "第二个打开者卡在开文件与加锁之间、第一个整理（改名换上新文件）之后放行：被同一把锁挡住（{refusal}）"
        )),
        Ok(refusal) => Err(format!(
            "第二个打开者被拒了（{refusal}），可从原路径重开读回没确认的写帧 {pending:?}（该是空的）"
        )),
        Err(why) => Err(format!(
            "{why}；从原路径重开读回没确认的写帧 {pending:?}：它追加的那一帧写进了改名之后没人再读的旧文件"
        )),
    }
}

/// 认领那一格：持有者身份 255 字节、大批里重复查同一把短键的次数。
const OVERSIZE_CLAIM_REPEATS: usize = 260_000;
/// 认领回应不越过帧长上限那一格，每一帧认领的租期。
const OVERSIZE_CLAIM_LEASE_MILLISECONDS: u32 = 60_000;

fn claim_with_identity(
    client: &mut SimpleClient,
    identity: &[u8],
    keys: &[&[u8]],
) -> Result<ResponseOutcome, String> {
    let borrowed: Vec<(Table, &[u8])> = keys.iter().map(|key| (Table::Conclusions, *key)).collect();
    client.call(|sequence| {
        encode_claim_frame(
            sequence,
            identity,
            OVERSIZE_CLAIM_LEASE_MILLISECONDS,
            &borrowed,
        )
    })
}

pub(crate) fn cell_claim_response_within_frame_limit(
    context: &CellContext,
) -> Result<String, String> {
    let fitting = maximum_response_payload_bytes().saturating_sub(CLAIM_OUTCOME_COUNT_BYTES)
        / held_by_other_outcome_bytes(MAXIMUM_CLIENT_IDENTITY_BYTES);
    if fitting != 253_240 {
        return Err(format!(
            "255 字节身份的「别人持有」一帧回应放得下 {fitting} 个，已知答案是 253240 个（(67108864 − 10 − 4) / 265）"
        ));
    }
    let server = HardeningServer::start(
        context,
        context
            .break_switches
            .only(BreakSwitch::ClaimResponseUnbudgeted),
        &[],
    )?;
    let mut client = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    let holder = vec![b'a'; MAXIMUM_CLIENT_IDENTITY_BYTES];
    let claimed = claim_with_identity(&mut client, &holder, &[b"claim/held".as_slice()])?;
    if claimed != ResponseOutcome::Claims(vec![ClaimOutcome::Claimed]) {
        return Err(format!(
            "255 字节身份认领一把空闲的键该认领到，回的是 {claimed:?}"
        ));
    }
    let mixed = claim_with_identity(
        &mut client,
        b"other",
        &[b"claim/held".as_slice(), b"claim/mixed-free".as_slice()],
    )?;
    let ResponseOutcome::Claims(mixed_outcomes) = &mixed else {
        return Err(format!("混合认领回的不是认领结局：{mixed:?}"));
    };
    let is_mixed_right = matches!(
        mixed_outcomes.as_slice(),
        [ClaimOutcome::HeldByOther { holder_identity, .. }, ClaimOutcome::Claimed] if *holder_identity == holder
    );
    if !is_mixed_right {
        return Err(format!(
            "别人持有的键加一把空闲的键：该回别人持有（带 255 字节身份）、认领到，回的是 {mixed_outcomes:?}"
        ));
    }
    let mut oversize_keys: Vec<&[u8]> = vec![b"claim/oversize-free".as_slice()];
    oversize_keys.extend(std::iter::repeat_n(
        b"claim/held".as_slice(),
        OVERSIZE_CLAIM_REPEATS,
    ));
    let oversize = claim_with_identity(&mut client, b"other", &oversize_keys).map_err(|message| {
        format!("一帧 {} 个键的认领（{OVERSIZE_CLAIM_REPEATS} 个别人持有、身份 255 字节）：{message}", oversize_keys.len())
    })?;
    let ResponseOutcome::Rejected {
        status: RejectionStatus::MalformedPayload,
        message,
    } = &oversize
    else {
        let received = if let ResponseOutcome::Claims(outcomes) = &oversize {
            format!("{} 个结局", outcomes.len())
        } else {
            format!("{oversize:?}")
        };
        return Err(format!(
            "回应会越过帧长上限的认领该整帧回状态 1，回的是 {received}"
        ));
    };
    if !message.contains("超过帧长上限") {
        return Err(format!("整帧拒的说明该写回应会超过帧长上限：{message}"));
    }
    let after = claim_with_identity(&mut client, b"third", &[b"claim/oversize-free".as_slice()])?;
    if after != ResponseOutcome::Claims(vec![ClaimOutcome::Claimed]) {
        return Err(format!(
            "被整帧拒的那一帧里的空闲键不该被认领（不留部分副作用），别的身份再认领该认领到，回的是 {after:?}"
        ));
    }
    Ok(format!(
        "255 字节身份的「别人持有」一帧至多 {fitting} 个；混合认领照常；{} 个键的认领（回应会越过帧长上限）整帧回状态 1、说明写了上限，里面的空闲键没被认领",
        oversize_keys.len()
    ))
}

/// 建连那一格：重连总时长上限、限时、超出上限还许多等的余量、调用方要停的时刻。
const CONNECT_TOTAL: Duration = Duration::from_millis(600);
const CONNECT_CELL_DEADLINE: Duration = Duration::from_secs(10);
const CONNECT_RETURN_MARGIN: Duration = Duration::from_secs(2);
const CONNECT_STOP_AFTER: Duration = Duration::from_millis(300);
/// 连接队列的长度：Linux 上队列里排满它加一条之后，再来的连接排不进去。
const FULL_BACKLOG_LENGTH: libc::c_int = 1;
const FULL_BACKLOG_FILLERS: usize = 2;

fn bounded_connect_settings(
    route: RouteAddress,
    journal_path: PathBuf,
    switches: &BreakSwitches,
    stop: &Arc<AtomicBool>,
    maximum_total: Duration,
) -> ReliableClientSettings {
    let mut settings = ReliableClientSettings::new(route, journal_path);
    settings.reconnect = ReconnectPolicy {
        initial_delay: Duration::from_millis(20),
        maximum_delay: Duration::from_millis(100),
        maximum_attempts: 1000,
        maximum_total,
    };
    settings.frame_timeout = Duration::from_secs(60);
    settings.write_timeout = Duration::from_millis(300);
    settings.break_switches = switches.clone();
    let stop = Arc::clone(stop);
    settings.stop_check = Arc::new(move || stop.load(Ordering::SeqCst));
    settings
}

/// 在限时里打开客户端、等全部写确认；回（结局，用了多久）。到点置停标记、调 `release` 放掉堵着的连接，判卡住。
fn open_and_drain_within_deadline(
    settings: ReliableClientSettings,
    stop: &Arc<AtomicBool>,
    release: impl FnOnce(),
    what: &str,
) -> Result<(Result<(), ClientFailure>, Duration), String> {
    let started = Instant::now();
    let outcome = within_deadline(
        CONNECT_CELL_DEADLINE,
        || {
            stop.store(true, Ordering::SeqCst);
            release();
        },
        move || -> Result<(), ClientFailure> {
            let mut client = ReliableClient::open(settings)?;
            client.wait_all_acknowledged()
        },
    )
    .map_err(|message| {
        format!("{what}：{message}——客户端堵在建连或等回应里，重连总时长上限没管住")
    })?;
    Ok((outcome, started.elapsed()))
}

/// 结局该是「连不上」、说明里带 `expected_reason`，用时不超过总时长上限加余量。
fn expect_unreachable_in_time(
    outcome: &Result<(), ClientFailure>,
    elapsed: Duration,
    expected_reason: &str,
    what: &str,
) -> Result<(), String> {
    let limit = CONNECT_TOTAL + CONNECT_RETURN_MARGIN;
    if let Err(ClientFailure::ServerUnreachable { last_error, .. }) = outcome {
        if last_error.contains(expected_reason) && elapsed <= limit {
            return Ok(());
        }
    }
    Err(format!(
        "{what}：重连总时长上限 {} 毫秒，客户端 {} 毫秒时回 {outcome:?}（该在 {} 毫秒内回「连不上」、最后一次的错带「{expected_reason}」）",
        CONNECT_TOTAL.as_millis(),
        elapsed.as_millis(),
        limit.as_millis()
    ))
}

/// 把监听的连接队列改成 `FULL_BACKLOG_LENGTH`（Linux 上对已在听的套接字再 listen 一次就改了长度）。
fn shrink_backlog(descriptor: libc::c_int) -> Result<(), String> {
    // SAFETY: listen 只收两个整数；描述符来自调用方手里活着的监听套接字。
    let result = unsafe { libc::listen(descriptor, FULL_BACKLOG_LENGTH) };
    if result != 0 {
        return Err(format!(
            "改不了连接队列的长度：{}",
            io::Error::last_os_error()
        ));
    }
    Ok(())
}

/// 收掉监听队列里排着的连接（限时到了放开堵在阻塞 `connect` 里的客户端用）。
fn drain_unix_backlog(listener: &UnixListener) {
    if listener.set_nonblocking(true).is_ok() {
        while listener.accept().is_ok() {}
    }
}

fn drain_network_backlog(listener: &TcpListener) {
    if listener.set_nonblocking(true).is_ok() {
        while listener.accept().is_ok() {}
    }
}

pub(crate) fn cell_connect_bounded_by_reconnect_total(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::ConnectWithoutDeadline);
    let directory = CellDirectory::create()?;
    let full_socket = directory.path.join("full.sock");
    let full_listener = UnixListener::bind(&full_socket)
        .map_err(|error| format!("绑不上 {}：{error}", full_socket.display()))?;
    shrink_backlog(full_listener.as_raw_fd())?;
    let mut unix_fillers = Vec::new();
    for _ in 0..FULL_BACKLOG_FILLERS {
        unix_fillers.push(connect_raw(&full_socket)?);
    }
    let backlog_stop = Arc::new(AtomicBool::new(false));
    let (backlog_outcome, unix_elapsed) = open_and_drain_within_deadline(
        bounded_connect_settings(
            RouteAddress::UnixSocket(full_socket.clone()),
            directory.path.join("unix-backlog.journal"),
            &switches,
            &backlog_stop,
            CONNECT_TOTAL,
        ),
        &backlog_stop,
        || drain_unix_backlog(&full_listener),
        "unix socket 的连接队列满了",
    )?;
    expect_unreachable_in_time(
        &backlog_outcome,
        unix_elapsed,
        "连接队列满",
        "unix socket 的连接队列满了",
    )?;

    let requested_stop = Arc::new(AtomicBool::new(false));
    let stopper_flag = Arc::clone(&requested_stop);
    let stopper = thread::spawn(move || {
        thread::sleep(CONNECT_STOP_AFTER);
        stopper_flag.store(true, Ordering::SeqCst);
    });
    let (stop_outcome, stop_elapsed) = open_and_drain_within_deadline(
        bounded_connect_settings(
            RouteAddress::UnixSocket(full_socket),
            directory.path.join("unix-stop.journal"),
            &switches,
            &requested_stop,
            Duration::from_secs(60),
        ),
        &requested_stop,
        || drain_unix_backlog(&full_listener),
        "连接队列满、调用方要停",
    )?;
    let _ = stopper.join();
    if !matches!(stop_outcome, Err(ClientFailure::Stopped))
        || stop_elapsed > CONNECT_STOP_AFTER + CONNECT_RETURN_MARGIN
    {
        return Err(format!(
            "连接队列满、调用方 {} 毫秒时要停：客户端 {} 毫秒时回 {stop_outcome:?}（该在 {} 毫秒内回「调用方要停」）",
            CONNECT_STOP_AFTER.as_millis(),
            stop_elapsed.as_millis(),
            (CONNECT_STOP_AFTER + CONNECT_RETURN_MARGIN).as_millis()
        ));
    }
    drop(unix_fillers);

    let network_listener =
        TcpListener::bind("127.0.0.1:0").map_err(|error| format!("绑不上回环 TCP：{error}"))?;
    shrink_backlog(network_listener.as_raw_fd())?;
    let network_address: SocketAddr = network_listener
        .local_addr()
        .map_err(|error| format!("读不出回环 TCP 的地址：{error}"))?;
    let mut network_fillers = Vec::new();
    for _ in 0..FULL_BACKLOG_FILLERS {
        network_fillers.push(
            TcpStream::connect_timeout(&network_address, Duration::from_secs(2))
                .map_err(|error| format!("塞连接队列的 TCP 连接连不上：{error}"))?,
        );
    }
    let handshake_stop = Arc::new(AtomicBool::new(false));
    let (handshake_outcome, network_elapsed) = open_and_drain_within_deadline(
        bounded_connect_settings(
            RouteAddress::Network(network_address.to_string()),
            directory.path.join("tcp-handshake.journal"),
            &switches,
            &handshake_stop,
            CONNECT_TOTAL,
        ),
        &handshake_stop,
        || drain_network_backlog(&network_listener),
        "TCP 握手迟迟不完成",
    )?;
    expect_unreachable_in_time(
        &handshake_outcome,
        network_elapsed,
        "握手还没完成",
        "TCP 握手迟迟不完成",
    )?;
    drop(network_fillers);

    let silent_socket = directory.path.join("silent.sock");
    let silent_listener = UnixListener::bind(&silent_socket)
        .map_err(|error| format!("绑不上 {}：{error}", silent_socket.display()))?;
    let silent_journal = directory.path.join("silent.journal");
    {
        let recovered = Journal::open(
            &silent_journal,
            JournalOptions::from_break_switches(u64::MAX, &BreakSwitches::none()),
        )?;
        let mut journal = recovered.journal;
        journal.append_write(
            1,
            &write_frame(&recovered.client_identity, 1, "connect/silent", "pending"),
        )?;
    }
    let silent_stop = Arc::new(AtomicBool::new(false));
    let (silent_outcome, silent_elapsed) = open_and_drain_within_deadline(
        bounded_connect_settings(
            RouteAddress::UnixSocket(silent_socket),
            silent_journal,
            &switches,
            &silent_stop,
            CONNECT_TOTAL,
        ),
        &silent_stop,
        || drain_unix_backlog(&silent_listener),
        "连上之后服务不读报到、不回",
    )?;
    expect_unreachable_in_time(
        &silent_outcome,
        silent_elapsed,
        "一个回应都没收到",
        "连上之后服务不读报到、不回",
    )?;
    Ok(format!(
        "重连总时长上限 {} 毫秒：unix 连接队列满 {} 毫秒回「连不上」、调用方要停 {} 毫秒回「要停」、TCP 握手不完成 {} 毫秒回「连不上」、连上不读不回 {} 毫秒回「连不上」",
        CONNECT_TOTAL.as_millis(),
        unix_elapsed.as_millis(),
        stop_elapsed.as_millis(),
        network_elapsed.as_millis(),
        silent_elapsed.as_millis()
    ))
}

/// oversize-frame-reported-at-once：容错客户端的重连总时长上限（弄坏开关下重连重发到这么久回「连不上」）。
const OVERSIZE_RECONNECT_TOTAL: Duration = Duration::from_secs(2);

/// 单条超限当场报（`reliable_client.rs`、`client.rs` 文件头）：键长超过帧长上限的一条，容错客户端在重连总时长之内回协议错、
/// 说帧长上限、追加日志里没有这一帧；一问一答的客户端当场回错、说没发，同一条连接随后照常读写。
pub(crate) fn cell_oversize_frame_reported_at_once(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::OversizeFrameSentAndRetried);
    let server = HardeningServer::start(context, BreakSwitches::none(), &[])?;
    let oversize_bytes = usize::try_from(MAXIMUM_FRAME_BODY_BYTES).expect("u32 放得进 usize") + 1;
    let journal_path = server.path().join("oversize.journal");
    let stop = Arc::new(AtomicBool::new(false));
    let settings = bounded_connect_settings(
        server.route(),
        journal_path.clone(),
        &switches,
        &stop,
        OVERSIZE_RECONNECT_TOTAL,
    );
    let started = Instant::now();
    let reliable_outcome = within_deadline(
        CONNECT_CELL_DEADLINE,
        || stop.store(true, Ordering::SeqCst),
        move || -> Result<(), ClientFailure> {
            let mut client = ReliableClient::open(settings)?;
            client.submit_write_entries(&[(
                Table::Conclusions,
                vec![b'k'; oversize_bytes],
                bytes("v"),
            )])?;
            client.wait_all_acknowledged()
        },
    )
    .map_err(|message| format!("容错客户端写键长 {oversize_bytes} 字节的一条：{message}"))?;
    let reliable_elapsed = started.elapsed();
    let is_reliable_right = matches!(
        &reliable_outcome,
        Err(ClientFailure::Protocol(message)) if message.contains("帧长上限")
    ) && reliable_elapsed < OVERSIZE_RECONNECT_TOTAL;
    if !is_reliable_right {
        return Err(format!(
            "容错客户端写键长 {oversize_bytes} 字节的一条：{} 毫秒时回 {reliable_outcome:?}（该在 {} 毫秒内回协议错、说帧长上限，不重连）",
            reliable_elapsed.as_millis(),
            OVERSIZE_RECONNECT_TOTAL.as_millis()
        ));
    }
    // 重开一次追加日志、数要重放的写帧；这一句一结束客户端放手，日志的锁跟着放。
    let replayed = ReliableClient::open(plain_client_settings(
        server.route(),
        journal_path,
        &BreakSwitches::none(),
    ))
    .map_err(|failure| format!("重开追加日志：{failure}"))?
    .outstanding_writes();
    if replayed != 0 {
        return Err(format!(
            "超限的那一帧不该记进追加日志，重开之后要重放的写帧有 {replayed} 帧"
        ));
    }
    let mut simple = SimpleClient::connect(&server.route(), switches)?;
    let simple_outcome =
        simple.write(&[(Table::Conclusions, vec![b'k'; oversize_bytes], bytes("v"))]);
    let is_simple_right = matches!(
        &simple_outcome,
        Err(message) if message.contains("帧长上限") && message.contains("没发")
    );
    if !is_simple_right {
        return Err(format!(
            "一问一答的客户端写键长 {oversize_bytes} 字节的一条：回 {simple_outcome:?}（该当场回错、说帧长上限与没发）"
        ));
    }
    let written_after = simple.write(&[entry("oversize/after", "1")])?;
    let read_after = read_value(&server.route(), "oversize/after")?;
    if written_after != 1 || read_after.as_deref() != Some(b"1".as_slice()) {
        return Err(format!(
            "超限那一条报错之后同一条连接再写一条：写了 {written_after} 条、读回 {}（该写 1 条、读回 1）",
            shown_value(read_after.as_deref())
        ));
    }
    Ok(format!(
        "键长 {oversize_bytes} 字节的一条：容错客户端 {} 毫秒回协议错、追加日志里没有这一帧；一问一答的客户端当场回错，同一条连接随后照常读写",
        reliable_elapsed.as_millis()
    ))
}

/// 认领租期那一格的数：租期 4 秒（缺省 30 秒按比例缩短）。丢了回应那一半让重发先撞一条不读的连接、等满帧超时 2.5 秒，
/// 认领落下之后至少过 2.5 秒才回到调用方手里；忙那一半让一帧点读在工作线程里压 2.8 秒、服务只许 1 帧待处理，认领一直回忙到压着的那一帧做完。
/// 回来之后过租期的 0.6（2.4 秒）别的身份再认领：持有者从回来那一刻起按整段租期写，这时它的写还没到服务。
const LEASE_COVER_LEASE_MILLISECONDS: u32 = 4_000;
const LEASE_COVER_RESEND_FRAME_TIMEOUT: Duration = Duration::from_millis(2_500);
const LEASE_COVER_HOLDING_READ_MILLISECONDS: u64 = 2_800;
/// 压着的那一帧点读的序号：别的帧（容错客户端与一问一答客户端的序号都从 1 起）撞不上。
const LEASE_COVER_HOLDING_READ_SEQUENCE: u64 = 777_777;
const LEASE_COVER_PROBE_AFTER_ANSWER: Duration = Duration::from_millis(2_400);
/// 同一身份再认领时给的长短两种租期（到期只许往后挪那一问）。
const LEASE_COVER_LONG_LEASE_MILLISECONDS: u32 = 600_000;
const LEASE_COVER_SHORT_LEASE_MILLISECONDS: u32 = 1;
/// 每一半做完的限时。
const LEASE_COVER_DEADLINE: Duration = Duration::from_secs(60);

/// 别的身份（新连的一问一答客户端）认领 `key`，回那一把键的结局。
fn claim_as_another_identity(route: &RouteAddress, key: &[u8]) -> Result<ClaimOutcome, String> {
    let mut other = SimpleClient::connect(route, BreakSwitches::none())?;
    let outcomes = other.claim(
        &[(Table::Conclusions, key.to_vec())],
        LEASE_COVER_LEASE_MILLISECONDS,
    )?;
    match outcomes.as_slice() {
        [outcome] => Ok(outcome.clone()),
        _ => Err(format!(
            "认领一把键回了 {} 个结局：{outcomes:?}",
            outcomes.len()
        )),
    }
}

/// 认领回到持有者手里之后：立刻（还没到头一次落下时算的到期）与过 `LEASE_COVER_PROBE_AFTER_ANSWER`（持有者按整段租期还在写）
/// 各让别的身份认领一次，两次都该是「别人持有」。
fn expect_held_after_the_answer(
    route: &RouteAddress,
    key: &[u8],
    answered: Instant,
    what: &str,
) -> Result<(), String> {
    let right_after = claim_as_another_identity(route, key)?;
    if !matches!(right_after, ClaimOutcome::HeldByOther { .. }) {
        return Err(format!(
            "{what}：认领回来之后立刻（头一次落下时算的到期之前）别的身份认领回 {right_after:?}，该是别人持有"
        ));
    }
    thread::sleep(
        (answered + LEASE_COVER_PROBE_AFTER_ANSWER).saturating_duration_since(Instant::now()),
    );
    let while_writing = claim_as_another_identity(route, key)?;
    if !matches!(while_writing, ClaimOutcome::HeldByOther { .. }) {
        return Err(format!(
            "{what}：认领回来之后 {} 毫秒（持有者从回来那一刻起按整段 {LEASE_COVER_LEASE_MILLISECONDS} 毫秒租期写，这时还没写到服务）\
             别的身份认领回 {while_writing:?}，该是别人持有：服务给的到期没盖住回来之后的整段租期",
            LEASE_COVER_PROBE_AFTER_ANSWER.as_millis()
        ));
    }
    Ok(())
}

/// 头一次认领落下、回应被代理吞掉再断；重发先撞一条不读的连接、等满帧超时，再重连重发才回来。
fn lease_after_a_lost_answer(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let server = HardeningServer::start(context, switches.clone(), &[])?;
    let directory = CellDirectory::create()?;
    let mut proxy = FaultProxy::start(
        ProxyListen::UnixSocket(directory.path.join("proxy.sock")),
        server.route(),
        &BreakSwitches::none(),
    )?;
    let control = proxy.shared_control();
    let stop = Arc::new(AtomicBool::new(false));
    let mut settings = plain_client_settings(
        proxy.route(),
        directory.path.join("lost-answer.journal"),
        switches,
    );
    settings.frame_timeout = LEASE_COVER_RESEND_FRAME_TIMEOUT;
    settings.reconnect = ReconnectPolicy {
        initial_delay: Duration::from_millis(20),
        maximum_delay: Duration::from_millis(200),
        maximum_attempts: 100,
        maximum_total: Duration::from_secs(30),
    };
    let stop_for_client = Arc::clone(&stop);
    settings.stop_check = Arc::new(move || stop_for_client.load(Ordering::SeqCst));
    let key = bytes("lease-cover/lost-answer");
    let control_for_client = Arc::clone(&control);
    let claimed = within_deadline(
        LEASE_COVER_DEADLINE,
        || stop.store(true, Ordering::SeqCst),
        || -> Result<(Vec<ClaimOutcome>, Instant, Instant), ClientFailure> {
            let mut client = ReliableClient::open(settings)?;
            control_for_client
                .swallow_responses_then_cut_requests
                .store(1, Ordering::SeqCst);
            control_for_client
                .stall_reading_connections_remaining
                .store(1, Ordering::SeqCst);
            let submitted = Instant::now();
            let outcomes = client.claim(
                &[(Table::Conclusions, key.clone())],
                LEASE_COVER_LEASE_MILLISECONDS,
            )?;
            Ok((outcomes, submitted, Instant::now()))
        },
    )?;
    let cuts = control.cuts_done.load(Ordering::SeqCst);
    let stalls = control.stalls_done.load(Ordering::SeqCst);
    proxy.stop();
    let (outcomes, submitted, answered) =
        claimed.map_err(|failure| format!("丢了回应的认领：客户端出错：{failure}"))?;
    if outcomes != [ClaimOutcome::Claimed] {
        return Err(format!("丢了回应、重发之后认领回 {outcomes:?}，该是认领到"));
    }
    let answer_delay = answered.duration_since(submitted);
    if cuts != 1 || stalls != 1 || answer_delay < LEASE_COVER_RESEND_FRAME_TIMEOUT {
        return Err(format!(
            "代理吞回应再断 {cuts} 次、停读 {stalls} 条连接（都该是 1）、认领 {} 毫秒回来（该至少等满帧超时 {} 毫秒）：故障没注入上，这一格没测到重发",
            answer_delay.as_millis(),
            LEASE_COVER_RESEND_FRAME_TIMEOUT.as_millis()
        ));
    }
    expect_held_after_the_answer(
        &server.route(),
        &key,
        answered,
        "丢了回应、重发才回来的认领",
    )?;
    Ok(format!(
        "丢了回应的认领 {} 毫秒后重发才回来：回来时与回来之后 {} 毫秒别的身份都回别人持有",
        answer_delay.as_millis(),
        LEASE_COVER_PROBE_AFTER_ANSWER.as_millis()
    ))
}

/// 服务只许 1 帧待处理、一帧点读在工作线程里压着：认领一直回忙，压着的那一帧做完才落下。
fn lease_after_a_long_busy(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let holding_delay =
        format!("{LEASE_COVER_HOLDING_READ_SEQUENCE}:{LEASE_COVER_HOLDING_READ_MILLISECONDS}");
    let server = HardeningServer::start(
        context,
        switches.clone(),
        &[
            "--maximum-pending-total",
            "1",
            "--busy-retry-milliseconds",
            "100",
            "--selftest-delay",
            &holding_delay,
        ],
    )?;
    let directory = CellDirectory::create()?;
    let stop = Arc::new(AtomicBool::new(false));
    let mut settings = plain_client_settings(
        server.route(),
        directory.path.join("long-busy.journal"),
        switches,
    );
    let stop_for_client = Arc::clone(&stop);
    settings.stop_check = Arc::new(move || stop_for_client.load(Ordering::SeqCst));
    let mut holding = connect_raw(&server.socket_path())?;
    let key = bytes("lease-cover/long-busy");
    let claimed = within_deadline(
        LEASE_COVER_DEADLINE,
        || stop.store(true, Ordering::SeqCst),
        || -> Result<(Vec<ClaimOutcome>, u64, Instant, Instant), String> {
            let mut client =
                ReliableClient::open(settings).map_err(|failure| failure.to_string())?;
            let mut builder = begin_point_read_frame(LEASE_COVER_HOLDING_READ_SEQUENCE, 1, 32);
            builder.key(Table::Conclusions, b"lease-cover/holding");
            holding
                .write_all(&builder.finish())
                .map_err(|error| format!("压着的那一帧点读发不出去：{error}"))?;
            thread::sleep(Duration::from_millis(100));
            let submitted = Instant::now();
            let outcomes = client
                .claim(
                    &[(Table::Conclusions, key.clone())],
                    LEASE_COVER_LEASE_MILLISECONDS,
                )
                .map_err(|failure| failure.to_string())?;
            Ok((
                outcomes,
                client.diagnostics().busy_responses,
                submitted,
                Instant::now(),
            ))
        },
    )?;
    let (outcomes, busy_responses, submitted, answered) =
        claimed.map_err(|message| format!("忙了很久的认领：{message}"))?;
    drop(holding);
    if outcomes != [ClaimOutcome::Claimed] {
        return Err(format!("忙了很久之后认领回 {outcomes:?}，该是认领到"));
    }
    let answer_delay = answered.duration_since(submitted);
    let half_lease = Duration::from_millis(u64::from(LEASE_COVER_LEASE_MILLISECONDS / 2));
    if busy_responses == 0 || answer_delay < half_lease {
        return Err(format!(
            "认领回了 {busy_responses} 次忙（该至少 1 次）、{} 毫秒回来（该过了租期的一半 {} 毫秒）：忙没造出来，这一格没测到",
            answer_delay.as_millis(),
            half_lease.as_millis()
        ));
    }
    expect_held_after_the_answer(&server.route(), &key, answered, "忙了大半租期才落下的认领")?;
    Ok(format!(
        "认领回忙 {busy_responses} 次、{} 毫秒才落下：回来时与回来之后 {} 毫秒别的身份都回别人持有",
        answer_delay.as_millis(),
        LEASE_COVER_PROBE_AFTER_ANSWER.as_millis()
    ))
}

/// 同一身份先用长租期认领，再用 1 毫秒的租期认领同一把键：到期不往前挪，别的身份过一会照样回别人持有、剩余租期还是长的。
fn renewal_never_moves_the_expiry_earlier(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let server = HardeningServer::start(context, switches.clone(), &[])?;
    let key = bytes("lease-cover/renewed-shorter");
    let mut holder = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    for lease in [
        LEASE_COVER_LONG_LEASE_MILLISECONDS,
        LEASE_COVER_SHORT_LEASE_MILLISECONDS,
    ] {
        let outcomes = holder.claim(&[(Table::Conclusions, key.clone())], lease)?;
        if outcomes != [ClaimOutcome::Claimed] {
            return Err(format!(
                "同一身份用 {lease} 毫秒的租期认领自己的键回 {outcomes:?}，该是认领到"
            ));
        }
    }
    thread::sleep(Duration::from_millis(100));
    let probe = claim_as_another_identity(&server.route(), &key)?;
    let half_long_lease = u64::from(LEASE_COVER_LONG_LEASE_MILLISECONDS / 2);
    match probe {
        ClaimOutcome::HeldByOther {
            remaining_lease_milliseconds,
            ..
        } if remaining_lease_milliseconds > half_long_lease => Ok(format!(
            "同一身份 {LEASE_COVER_LONG_LEASE_MILLISECONDS} 毫秒认领之后再用 {LEASE_COVER_SHORT_LEASE_MILLISECONDS} 毫秒认领：\
             别的身份回别人持有、剩余 {remaining_lease_milliseconds} 毫秒"
        )),
        other @ (ClaimOutcome::Claimed
        | ClaimOutcome::HeldByOther { .. }
        | ClaimOutcome::AlreadyConcluded) => Err(format!(
            "同一身份 {LEASE_COVER_LONG_LEASE_MILLISECONDS} 毫秒认领之后再用 {LEASE_COVER_SHORT_LEASE_MILLISECONDS} 毫秒认领，\
             过 100 毫秒别的身份认领回 {other:?}，该是别人持有、剩余超过 {half_long_lease} 毫秒：到期被帧里更短的租期往前挪了"
        )),
    }
}

/// 认领回到调用方手里之后，服务给的到期盖住调用方从回来那一刻起按整段租期写的那一段（`committer.rs` 文件头「认领」、
/// `reliable_client.rs` 文件头「认领」）。三条输入：丢了回应重发才回来、忙了大半租期才落下、同一身份用更短的租期再认领。
pub(crate) fn cell_claim_lease_covers_the_holder_after_the_answer(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context.break_switches.only_these(&[
        BreakSwitch::ClaimLeaseShrunkOnResend,
        BreakSwitch::ClaimExpiryMovedEarlier,
    ]);
    let lost_answer = lease_after_a_lost_answer(context, &switches)?;
    let long_busy = lease_after_a_long_busy(context, &switches)?;
    let renewal = renewal_never_moves_the_expiry_earlier(context, &switches)?;
    Ok(format!(
        "租期 {LEASE_COVER_LEASE_MILLISECONDS} 毫秒：{lost_answer}；{long_busy}；{renewal}"
    ))
}

/// 存储出错重放那一格：客户端的重连策略（服务退 7 到被拉起来之间连不上，次数给足、间隔封顶 500 毫秒，一轮至多 60 秒）。
const STORAGE_RETRY_RECONNECT: ReconnectPolicy = ReconnectPolicy {
    initial_delay: Duration::from_millis(50),
    maximum_delay: Duration::from_millis(500),
    maximum_attempts: 1000,
    maximum_total: Duration::from_secs(60),
};
/// 等故障日志里出现写失败那一行的上限。
const STORAGE_FAILURE_LINE_TIMEOUT: Duration = Duration::from_secs(20);

fn storage_retry_settings(
    route: RouteAddress,
    journal_path: PathBuf,
    switches: &BreakSwitches,
    stop: &Arc<AtomicBool>,
) -> ReliableClientSettings {
    let mut settings = plain_client_settings(route, journal_path, switches);
    settings.reconnect = STORAGE_RETRY_RECONNECT;
    let stop = Arc::clone(stop);
    settings.stop_check = Arc::new(move || stop.load(Ordering::SeqCst));
    settings
}

/// 开容错客户端、写一帧 `key` = value、等全部确认；回写帧回存储出错、照同一请求号再交的次数。
fn write_one_frame_until_acknowledged(
    settings: ReliableClientSettings,
    key: &str,
) -> Result<u64, ClientFailure> {
    let mut client = ReliableClient::open(settings)?;
    client.submit_write_entries(&[entry(key, "value")])?;
    client.wait_all_acknowledged()?;
    Ok(client.diagnostics().write_storage_failures)
}

/// 客户端的结局该是做完（退出码 0）；回错的把退出码写进说明。
fn expect_client_finished(outcome: Result<u64, ClientFailure>, what: &str) -> Result<u64, String> {
    outcome.map_err(|failure| {
        format!(
            "{what}：容错客户端回错、退出码 {}（该照同一请求号重放、做完退 0；连不上到上限才退 75，不许退 3）：{failure}",
            failure.exit_code()
        )
    })
}

/// 同步 WAL 失败、服务退 7，删掉造错文件再起服务：同一个客户端照同一请求号重放，键在库里。
fn storage_failure_replayed_after_sync_failure(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let arm_directory = CellDirectory::create()?;
    let arm = arm_directory.path.join("sync-failure-armed");
    let arm_argument = arm.display().to_string();
    let mut server = HardeningServer::start(
        context,
        BreakSwitches::none(),
        &["--selftest-sync-failure-file", &arm_argument],
    )?;
    std::fs::write(&arm, b"").map_err(|error| format!("建不了 {arm_argument}：{error}"))?;
    let stop = Arc::new(AtomicBool::new(false));
    let settings = storage_retry_settings(
        server.route(),
        server.path().join("sync-failure.journal"),
        switches,
        &stop,
    );
    let outcome = thread::scope(|scope| -> Result<Result<u64, ClientFailure>, String> {
        let writer =
            scope.spawn(|| write_one_frame_until_acknowledged(settings, "storage-retry/sync"));
        let restarted = match server.wait_for_exit(SYNC_FAILURE_EXIT_TIMEOUT) {
            Some(status) if status.code() == Some(STORAGE_SYNC_FAILED_EXIT_CODE) => {
                std::fs::remove_file(&arm)
                    .map_err(|error| format!("删不了 {arm_argument}：{error}"))
                    .and_then(|()| server.spawn(context))
            }
            Some(status) => Err(format!(
                "同步 WAL 失败之后服务该退 {STORAGE_SYNC_FAILED_EXIT_CODE}，退了 {status}；标准错误：{}",
                server.standard_error_text()
            )),
            None => Err(format!(
                "同步 WAL 失败之后服务 {} 秒没退出",
                SYNC_FAILURE_EXIT_TIMEOUT.as_secs()
            )),
        };
        if let Err(message) = restarted {
            stop.store(true, Ordering::SeqCst);
            let _ = writer.join();
            return Err(message);
        }
        writer.join().map_err(|_| "写的线程 panic 了".to_string())
    })?;
    let storage_failures = expect_client_finished(outcome, "同步 WAL 失败、服务退 7 又起来之后")?;
    let stored = read_value(&server.route(), "storage-retry/sync")?;
    if stored.as_deref() != Some(b"value".as_slice()) {
        return Err(format!(
            "同步 WAL 失败、服务重起之后客户端说做完了，库里读回 {}（该是 value）",
            shown_value(stored.as_deref())
        ));
    }
    Ok(format!(
        "同步 WAL 失败、服务退 {STORAGE_SYNC_FAILED_EXIT_CODE} 再起：客户端（收到存储出错 {storage_failures} 次）重连后照同一请求号重放、做完，键在库里"
    ))
}

/// 写批写失败、服务不退：造错文件在时每组都写失败，故障日志出现写失败那一行之后删掉造错文件，同一个客户端照同一请求号再交，键写进库。
fn storage_failure_replayed_after_write_failure(
    context: &CellContext,
    switches: &BreakSwitches,
) -> Result<String, String> {
    let arm_directory = CellDirectory::create()?;
    let arm = arm_directory.path.join("write-failure-armed");
    let arm_argument = arm.display().to_string();
    let mut server = HardeningServer::start(
        context,
        BreakSwitches::none(),
        &["--selftest-write-failure-file", &arm_argument],
    )?;
    std::fs::write(&arm, b"").map_err(|error| format!("建不了 {arm_argument}：{error}"))?;
    let fault_log = server.path().join(FAULT_LOG_FILE_NAME);
    let stop = Arc::new(AtomicBool::new(false));
    let settings = storage_retry_settings(
        server.route(),
        server.path().join("write-failure.journal"),
        switches,
        &stop,
    );
    let outcome = thread::scope(|scope| -> Result<Result<u64, ClientFailure>, String> {
        let writer =
            scope.spawn(|| write_one_frame_until_acknowledged(settings, "storage-retry/write"));
        let deadline = Instant::now() + STORAGE_FAILURE_LINE_TIMEOUT;
        loop {
            let logged = std::fs::read_to_string(&fault_log)
                .is_ok_and(|text| text.contains("kind=storage-write-failed"));
            if logged || writer.is_finished() {
                break;
            }
            if Instant::now() >= deadline {
                stop.store(true, Ordering::SeqCst);
                let _ = writer.join();
                return Err(format!(
                    "{} 秒里故障日志 {} 没出现写失败那一行：造错文件没起作用，这一格没测到",
                    STORAGE_FAILURE_LINE_TIMEOUT.as_secs(),
                    fault_log.display()
                ));
            }
            thread::sleep(Duration::from_millis(50));
        }
        if let Err(error) = std::fs::remove_file(&arm) {
            stop.store(true, Ordering::SeqCst);
            let _ = writer.join();
            return Err(format!("删不了 {arm_argument}：{error}"));
        }
        writer.join().map_err(|_| "写的线程 panic 了".to_string())
    })?;
    let storage_failures = expect_client_finished(outcome, "写批写失败、服务不退")?;
    if let Some(status) = server.wait_for_exit(Duration::ZERO) {
        return Err(format!(
            "写批写失败之后服务退了（{status}），该照常服务；标准错误：{}",
            server.standard_error_text()
        ));
    }
    if storage_failures == 0 {
        return Err(
            "造错文件在时写帧一次存储出错都没收到：这一格没测到写失败之后的重交".to_string(),
        );
    }
    let stored = read_value(&server.route(), "storage-retry/write")?;
    if stored.as_deref() != Some(b"value".as_slice()) {
        return Err(format!(
            "写批写失败、照同一请求号再交之后客户端说做完了，库里读回 {}（该是 value）",
            shown_value(stored.as_deref())
        ));
    }
    Ok(format!(
        "写批写失败、服务不退：客户端收到存储出错 {storage_failures} 次、照同一请求号再交，造错文件删掉之后写进库"
    ))
}

/// 写帧回存储出错不是「重发也不会好」（`reliable_client.rs` 文件头「存储出错」）：同步失败、服务退出重起之后重放，
/// 写失败、服务还在时再交，两种都做完、键在库里，不退 3。
pub(crate) fn cell_write_storage_failure_replayed(context: &CellContext) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::WriteStorageFailureAbandoned);
    let sync_failure = storage_failure_replayed_after_sync_failure(context, &switches)?;
    let write_failure = storage_failure_replayed_after_write_failure(context, &switches)?;
    Ok(format!("{sync_failure}；{write_failure}"))
}

/// 删前缀回应那一格的数：每条连接额度 64 KiB、前缀下一把 1 MiB 的长键、一条连接上一次排 8 帧删前缀；
/// 组提交等满 8 帧（至多 2 秒）再做，8 帧都进了准入才轮到头一帧，头一帧做的时候额度里还记着别的 7 帧。
const BUDGETED_DELETE_PER_CONNECTION_BYTES: usize = 64 * 1024;
const BUDGETED_DELETE_LONG_KEY_BYTES: usize = 1024 * 1024;
const BUDGETED_DELETE_PIPELINED_FRAMES: u64 = 8;
const BUDGETED_DELETE_GROUP_WINDOW_MILLISECONDS: u64 = 2_000;
/// 每帧删前缀的请求与忙的回应（服务自己写的短句）在额度里留的余量。
const BUDGETED_DELETE_SLACK_BYTES_PER_FRAME: usize = 1024;
/// 删前缀那一格的前缀。
const BUDGETED_DELETE_PREFIX: &[u8] = b"budgeted-delete/";

/// 从一条裸连接上读一帧回应、解开。
fn read_raw_response(stream: &mut UnixStream) -> Result<crate::protocol::Response, String> {
    let mut length_field = [0_u8; FRAME_LENGTH_FIELD_BYTES];
    stream
        .read_exact(&mut length_field)
        .map_err(|error| format!("读不到回应的长度字段：{error}"))?;
    let body_bytes = usize::try_from(u32::from_le_bytes(length_field))
        .map_err(|_| "回应帧体长度放不进 usize".to_string())?;
    let mut body = vec![0_u8; body_bytes];
    stream
        .read_exact(&mut body)
        .map_err(|error| format!("读不全 {body_bytes} 字节的回应帧体：{error}"))?;
    decode_response_body(&body)
}

/// 一次写出一串删前缀帧（`count_only` 逐帧给，序号从 `first_sequence` 起），再按序号读回全部回应；回（序号，结局）按序号排好。
fn delete_prefix_frames(
    stream: &mut UnixStream,
    first_sequence: u64,
    count_only: &[bool],
) -> Result<Vec<(u64, ResponseOutcome)>, String> {
    let mut frames = Vec::new();
    for (sequence, &is_count_only) in (first_sequence..).zip(count_only) {
        frames.extend(encode_delete_prefix_frame(
            sequence,
            BUDGETED_DELETE_PREFIX,
            b"",
            1,
            is_count_only,
        ));
    }
    stream
        .write_all(&frames)
        .map_err(|error| format!("一次写不出 {} 帧删前缀：{error}", count_only.len()))?;
    let mut answers = Vec::with_capacity(count_only.len());
    for _ in count_only {
        let response = read_raw_response(stream)?;
        answers.push((response.sequence, response.outcome));
    }
    answers.sort_by_key(|(sequence, _)| *sequence);
    Ok(answers)
}

/// 删前缀的回应先记进额度、记不进整帧回忙一把都不删（`committer.rs` 文件头「删前缀」、`server.rs` 文件头「额度」）。
pub(crate) fn cell_delete_prefix_response_within_pending_budget(
    context: &CellContext,
) -> Result<String, String> {
    let per_connection = BUDGETED_DELETE_PER_CONNECTION_BYTES.to_string();
    let group_frames = BUDGETED_DELETE_PIPELINED_FRAMES.to_string();
    let group_window = BUDGETED_DELETE_GROUP_WINDOW_MILLISECONDS.to_string();
    let server = HardeningServer::start(
        context,
        context
            .break_switches
            .only(BreakSwitch::DeletePrefixResponseUnbudgeted),
        &[
            "--maximum-pending-bytes-per-connection",
            &per_connection,
            "--group-commit-frames",
            &group_frames,
            "--group-commit-milliseconds",
            &group_window,
        ],
    )?;
    let mut long_key = BUDGETED_DELETE_PREFIX.to_vec();
    long_key.resize(BUDGETED_DELETE_LONG_KEY_BYTES, b'k');
    let mut other = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    other.write(&[(Table::Conclusions, long_key.clone(), bytes("kept"))])?;
    let mut pipelining = connect_raw(&server.socket_path())?;
    pipelining
        .set_read_timeout(Some(Duration::from_secs(60)))
        .map_err(|error| format!("设不上读超时：{error}"))?;
    let frame_count = usize::try_from(BUDGETED_DELETE_PIPELINED_FRAMES).expect("8 帧");
    // 只数不删 8 帧：至多一帧的回应记得进，别的回忙
    let counting = delete_prefix_frames(&mut pipelining, 1, &vec![true; frame_count])?;
    let mut counted_frames = 0_usize;
    let mut busy_frames = 0_usize;
    for (sequence, outcome) in &counting {
        match outcome {
            ResponseOutcome::Busy { .. } => busy_frames += 1,
            ResponseOutcome::PrefixDeleted {
                keys_in_frame: 1,
                last_key,
                ..
            } if *last_key == long_key => counted_frames += 1,
            other_outcome @ (ResponseOutcome::PointValues(_)
            | ResponseOutcome::WriteAcknowledged { .. }
            | ResponseOutcome::ScanEntries { .. }
            | ResponseOutcome::Statistics(_)
            | ResponseOutcome::Claims(_)
            | ResponseOutcome::PrefixDeleted { .. }
            | ResponseOutcome::ConnectedClients(_)
            | ResponseOutcome::ServerStopping { .. }
            | ResponseOutcome::Rejected { .. }) => {
                return Err(format!(
                    "只数不删的序号 {sequence} 回 {other_outcome:?}，该是忙或数到 1 把、最后一把是那把长键"
                ))
            }
        }
    }
    if busy_frames == 0 {
        return Err(format!(
            "一条连接上同时排 {frame_count} 帧只数不删、每帧回应带一把 {BUDGETED_DELETE_LONG_KEY_BYTES} 字节的长键、每条连接额度 \
             {BUDGETED_DELETE_PER_CONNECTION_BYTES} 字节：{counted_frames} 帧都回了长键、一帧忙都没有：回应不看额度、强行记下了"
        ));
    }
    // 删的那一帧排在头一个：做它的时候额度里还记着别的 7 帧，记不进、回忙、一把都不删
    let mut deleting_pattern = vec![true; frame_count];
    deleting_pattern[0] = false;
    let deleting = delete_prefix_frames(
        &mut pipelining,
        BUDGETED_DELETE_PIPELINED_FRAMES + 1,
        &deleting_pattern,
    )?;
    let first_answer = deleting.first().map(|(_, outcome)| outcome);
    if !matches!(first_answer, Some(ResponseOutcome::Busy { .. })) {
        return Err(format!(
            "排在头一个的删前缀（额度里还记着别的 7 帧）回 {first_answer:?}，该是忙：回应记不进额度时不许删"
        ));
    }
    let still_there = other.read(&[(Table::Conclusions, long_key.clone())])?;
    if still_there.first().cloned().flatten().as_deref() != Some(b"kept".as_slice()) {
        return Err("删前缀回了忙，那把长键却从库里没了：记不进额度的那一帧照样删了".to_string());
    }
    other
        .write(&[entry("budgeted-delete-other/after", "served")])
        .map_err(|message| format!("删前缀回忙之后别的连接写不进：{message}"))?;
    let served = read_value(&server.route(), "budgeted-delete-other/after")?;
    if served.as_deref() != Some(b"served".as_slice()) {
        return Err(format!(
            "删前缀回忙之后别的连接读回 {}（该是 served）",
            shown_value(served.as_deref())
        ));
    }
    let pending_peak = counter(&other.statistics()?, PENDING_BYTES_PEAK_COUNTER)?;
    let long_response_bytes = BUDGETED_DELETE_LONG_KEY_BYTES + 64;
    let pending_bound = u64::try_from(
        BUDGETED_DELETE_PER_CONNECTION_BYTES
            + long_response_bytes
            + frame_count * 2 * BUDGETED_DELETE_SLACK_BYTES_PER_FRAME,
    )
    .expect("一两 MiB");
    if pending_peak > pending_bound {
        return Err(format!(
            "额度里记着的字节峰值 {pending_peak}，超过每条连接额度加一个长键回应加每帧余量（{pending_bound}）：删前缀回应在额度之外叠了一层"
        ));
    }
    drop(pipelining);
    Ok(format!(
        "每条连接额度 {BUDGETED_DELETE_PER_CONNECTION_BYTES} 字节、前缀下一把 {BUDGETED_DELETE_LONG_KEY_BYTES} 字节的长键：同时排 {frame_count} 帧只数不删，\
         {busy_frames} 帧回忙、{counted_frames} 帧回了长键；排在头一个的删前缀回忙、长键还在，别的连接照常读写；额度峰值 {pending_peak} 字节（上界 {pending_bound}）"
    ))
}

/// 工作线程额度那一格：服务整份额度（每条连接的额度照缺省，比它大）。
const WORKER_BUDGET_PENDING_BYTES: usize = 1024 * 1024;
/// 同一格：每把值的字节数（不分段的整值，一帧装得下两把）。
const WORKER_BUDGET_VALUE_BYTES: usize = 20 * 1024 * 1024;
/// 同一格：每个前缀下几把值（一帧扫回它们全部，回应约 40 MiB）。
const WORKER_BUDGET_VALUES_PER_PREFIX: usize = 2;
/// 同一格：同时扫的连接数，与自证服务的工作线程数相同（`selftest::SELFTEST_WORKER_THREADS`）。
const WORKER_BUDGET_SCANNERS: usize = 4;
/// 同一格：每条连接头一帧（序号 1）在工作线程里压这么久再做，几个工作线程同时开始编回应。
const WORKER_BUDGET_HOLD_MILLISECONDS: u64 = 600;
/// 同一格：扫完之后额度里还许记着的字节（统计那一帧自己的请求与回应）。
const WORKER_BUDGET_SETTLED_SLACK_BYTES: u64 = 64 * 1024;

/// 一个前缀下的值：`prefix` 的序号与值的序号定每个字节。
fn worker_budget_value(prefix_index: usize, value_index: usize) -> Vec<u8> {
    let filler = u8::try_from((prefix_index * WORKER_BUDGET_VALUES_PER_PREFIX + value_index) % 251)
        .expect("取模 251 放得进 u8");
    vec![filler; WORKER_BUDGET_VALUE_BYTES]
}

fn worker_budget_prefix(prefix_index: usize) -> Vec<u8> {
    bytes(&format!("worker-budget/{prefix_index}/"))
}

/// 工作线程边编边记额度（`server.rs` 文件头「额度」）：整份额度 1 MiB，四条连接同时按前缀扫各约 40 MiB 的回应（工作线程四个，头一帧都压 600 毫秒再做，
/// 四个工作线程同时开始编）：工作线程手里编着的回应合计的峰值不超额度加一帧（`WORKER_RESPONSE_BYTES_PEAK_COUNTER`），记不进的帧回忙、
/// 容错客户端退避重发，四条连接扫回的值逐字节对；扫完之后别的连接照常写读、额度放回。
pub(crate) fn cell_worker_responses_within_pending_budget(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context
        .break_switches
        .only(BreakSwitch::ResponseBuiltBeforeBudget);
    let pending_bytes = WORKER_BUDGET_PENDING_BYTES.to_string();
    let hold = format!("1:{WORKER_BUDGET_HOLD_MILLISECONDS}");
    let server = HardeningServer::start(
        context,
        switches,
        &[
            "--maximum-pending-bytes",
            &pending_bytes,
            "--selftest-delay",
            &hold,
        ],
    )?;
    let route = server.route();
    let mut writer = SimpleClient::connect(&route, BreakSwitches::none())?;
    for prefix_index in 0..WORKER_BUDGET_SCANNERS {
        let entries: Vec<(Table, Vec<u8>, Vec<u8>)> = (0..WORKER_BUDGET_VALUES_PER_PREFIX)
            .map(|value_index| {
                let mut key = worker_budget_prefix(prefix_index);
                key.extend_from_slice(format!("value-{value_index}").as_bytes());
                (
                    Table::Conclusions,
                    key,
                    worker_budget_value(prefix_index, value_index),
                )
            })
            .collect();
        writer.write(&entries)?;
    }
    let scanners: Vec<thread::JoinHandle<Result<(), String>>> = (0..WORKER_BUDGET_SCANNERS)
        .map(|prefix_index| {
            let route = route.clone();
            let journal = server.path().join(format!("scanner-{prefix_index}.journal"));
            thread::spawn(move || {
                let mut client = ReliableClient::open(plain_client_settings(
                    route,
                    journal,
                    &BreakSwitches::none(),
                ))
                .map_err(|failure| format!("扫的客户端开不了：{failure}"))?;
                let prefix = worker_budget_prefix(prefix_index);
                let (is_truncated, entries) = client
                    .scan(Table::Conclusions, &prefix, 100)
                    .map_err(|failure| format!("扫前缀 {} 不成：{failure}", String::from_utf8_lossy(&prefix)))?;
                let matched = !is_truncated
                    && entries.len() == WORKER_BUDGET_VALUES_PER_PREFIX
                    && entries
                        .iter()
                        .enumerate()
                        .all(|(value_index, entry)| entry.value == worker_budget_value(prefix_index, value_index));
                if !matched {
                    return Err(format!(
                        "扫前缀 {} 回 {} 条（截断 {is_truncated}），该是 {WORKER_BUDGET_VALUES_PER_PREFIX} 条、逐字节与写进去的相同",
                        String::from_utf8_lossy(&prefix),
                        entries.len()
                    ));
                }
                Ok(())
            })
        })
        .collect();
    for scanner in scanners {
        scanner
            .join()
            .map_err(|_| "扫的线程 panic 了".to_string())??;
    }
    writer.write(&[entry("worker-budget/after", "still-serving")])?;
    let after = writer.read(&[(Table::Conclusions, bytes("worker-budget/after"))])?;
    if after != [Some(bytes("still-serving"))] {
        return Err(format!(
            "扫完之后别的连接写了又读回 {after:?}，该读回 still-serving"
        ));
    }
    let statistics = writer.statistics()?;
    let peak = counter(&statistics, WORKER_RESPONSE_BYTES_PEAK_COUNTER)?;
    let busy = counter(&statistics, BUSY_REJECTIONS_COUNTER)?;
    let settled = counter(&statistics, PENDING_BYTES_COUNTER)?;
    let bound = u64::try_from(WORKER_BUDGET_PENDING_BYTES).unwrap_or(u64::MAX)
        + u64::from(MAXIMUM_FRAME_BODY_BYTES);
    if peak > bound {
        return Err(format!(
            "工作线程手里编着的回应合计峰值 {peak} 字节，超过额度 {WORKER_BUDGET_PENDING_BYTES} 加一帧 {MAXIMUM_FRAME_BODY_BYTES} = {bound}：\
             回应先编好整份才记额度，几个工作线程同时编的那几份都在额度之外"
        ));
    }
    if busy == 0 || settled > WORKER_BUDGET_SETTLED_SLACK_BYTES {
        return Err(format!(
            "四条连接同时扫各约 40 MiB：回忙 {busy} 次（该至少 1 次，额度 1 MiB 装不下两份回应）、扫完之后额度里还记着 {settled} 字节（该至多 {WORKER_BUDGET_SETTLED_SLACK_BYTES}）"
        ));
    }
    Ok(format!(
        "额度 {WORKER_BUDGET_PENDING_BYTES} 字节、四条连接同时扫各 {} 字节：工作线程手里编着的回应合计峰值 {peak} 字节（上界 {bound}）、回忙 {busy} 次、四份都扫回且逐字节对、扫完别的连接照常写读、额度放回到 {settled} 字节",
        WORKER_BUDGET_VALUE_BYTES * WORKER_BUDGET_VALUES_PER_PREFIX
    ))
}
