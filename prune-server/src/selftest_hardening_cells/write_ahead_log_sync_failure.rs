//! 同步 WAL 失败之后不回确认的一格 `cell_write_ahead_log_sync_failure_never_acknowledged`：内建造错点那条输入，与装上 libc 包装让 fdatasync 真回 EIO 的那条输入。

use super::*;

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
pub(super) const FDATASYNC_ARMED_VARIABLE: &str = "PRUNE_SERVER_SELFTEST_FDATASYNC_ARMED";
pub(super) const FDATASYNC_LIBRARY_VARIABLE: &str = "PRUNE_SERVER_SELFTEST_FDATASYNC_LIBRARY";
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
