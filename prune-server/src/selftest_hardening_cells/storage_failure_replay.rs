//! 写帧回存储出错之后照同一请求号重放的一格 `cell_write_storage_failure_replayed`：同步失败之后与写失败之后各一条输入。

use super::*;

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
