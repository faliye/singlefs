//! 自证里加固的八格（格表在 `selftest.rs` 的 `CELLS`）：追加日志的请求号高水位与排他锁、同步 WAL 失败之后不回确认、
//! 对端不读时回应受额度管、没收完的帧受额度与期限管、认领回应不越过帧长上限、建连受重连总时长上限管、
//! 一帧大过帧长上限的当场报协议错。
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

use crate::break_switches::{BreakSwitch, BreakSwitches, BREAK_VARIABLE};
use crate::client::{random_client_identity, RouteAddress, SimpleClient};
use crate::connections::UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION;
use crate::journal::{Journal, JournalOptions};
use crate::protocol::{
    begin_point_read_frame, begin_write_frame, encode_claim_frame, held_by_other_outcome_bytes,
    maximum_response_payload_bytes, replace_request_sequence, ClaimOutcome, RejectionStatus,
    ResponseOutcome, Table, CLAIM_OUTCOME_COUNT_BYTES, MAXIMUM_CLIENT_IDENTITY_BYTES,
    MAXIMUM_FRAME_BODY_BYTES,
};
use crate::reliable_client::{
    ClientFailure, Completion, OperationIdentifier, ReconnectPolicy, ReliableClient,
    ReliableClientSettings,
};
use crate::selftest::{
    bytes, counter, loopback_arguments, server_command, CellContext, CellDirectory,
    SERVER_READY_TIMEOUT,
};
use crate::selftest_fault_cells::within_deadline;
use crate::server::{
    CONNECTIONS_REFUSED_COUNTER, FRAME_READ_TIMEOUTS_COUNTER, LISTENING_LINE_PREFIX,
    OUTBOUND_QUEUED_BYTES_PEAK_COUNTER, PENDING_BYTES_COUNTER, RECEIVING_BODY_BYTES_PEAK_COUNTER,
    STORAGE_SYNC_FAILED_EXIT_CODE, WRITE_TIMEOUTS_COUNTER,
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
const CLAIM_LEASE_MILLISECONDS: u32 = 60_000;

fn claim_with_identity(
    client: &mut SimpleClient,
    identity: &[u8],
    keys: &[&[u8]],
) -> Result<ResponseOutcome, String> {
    let borrowed: Vec<(Table, &[u8])> = keys.iter().map(|key| (Table::Conclusions, *key)).collect();
    client.call(|sequence| {
        encode_claim_frame(sequence, identity, CLAIM_LEASE_MILLISECONDS, &borrowed)
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
