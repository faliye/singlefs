//! 自证里容错、组提交与压测环境判法的各格（格表在 `selftest.rs` 的 `CELLS`）。
//! 每格起自己的真服务；要注入网络故障的经本进程里的故障注入代理（`fault_proxy`）连服务。
//! 会卡住的格（弄坏开关打开时客户端可能一直等下去）都有限时：到点置停标记、停代理，客户端随即回「调用方要停」，这一格判红。
//! 限时只是上限，跑得快就不等；判的都是对错与次数，不判吞吐。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use rocksdb::statistics::Ticker;

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::client::{connect, random_client_identity, RouteAddress, SimpleClient};
use crate::command_line::CLAIM_HOLDER_LINE_PREFIX;
use crate::environment::{
    busy_percent, judge_environment, parse_input_output_ticks, parse_load_averages,
    EnvironmentJudgment,
};
use crate::fault_proxy::{FaultProxy, ProxyListen};
use crate::keepalive::KeepaliveSettings;
use crate::protocol::{begin_write_frame, ClaimOutcome, ResponseOutcome, Table};
use crate::reliable_client::{
    ClientDiagnostics, ClientFailure, ReconnectPolicy, ReliableClient, ReliableClientSettings,
};
use crate::selftest::{
    bytes, counter, counter_delta, line_field, loopback_arguments, server_command,
    single_frame_single_sync, start_test_server, statistics_after_sync, CellContext, CellDirectory,
    SERVER_READY_TIMEOUT,
};
use crate::server::{DUPLICATE_REQUESTS_COUNTER, LISTENING_LINE_PREFIX, PENDING_PEAK_COUNTER};
use crate::spawned::{SpawnedProcess, StandardError};

/// 自证客户端的重连策略：服务在几格里被 `kill -9` 再重起，次数给足、间隔封顶 500 毫秒，一轮至多 60 秒。
const PATIENT_RECONNECT: ReconnectPolicy = ReconnectPolicy {
    initial_delay: Duration::from_millis(20),
    maximum_delay: Duration::from_millis(500),
    maximum_attempts: 1000,
    maximum_total: Duration::from_secs(60),
};

fn client_settings(
    route: RouteAddress,
    journal_path: PathBuf,
    break_switches: &BreakSwitches,
    stop: &Arc<AtomicBool>,
) -> ReliableClientSettings {
    let mut settings = ReliableClientSettings::new(route, journal_path);
    settings.frame_timeout = Duration::from_secs(30);
    settings.maximum_in_flight_frames = 16;
    settings.reconnect = PATIENT_RECONNECT;
    settings.break_switches = break_switches.clone();
    let stop = Arc::clone(stop);
    settings.stop_check = Arc::new(move || stop.load(Ordering::SeqCst));
    settings
}

fn failure_text(failure: &ClientFailure) -> String {
    failure.to_string()
}

/// 在一个线程里做 `work`，至多等 `deadline`；到点调 `on_timeout`（置停标记、停代理）让它收手，再等它退出、判卡住。
pub(crate) fn within_deadline<Output: Send>(
    deadline: Duration,
    on_timeout: impl FnOnce(),
    work: impl FnOnce() -> Output + Send,
) -> Result<Output, String> {
    thread::scope(|scope| {
        let (done_sender, done_receiver) = mpsc::channel();
        let handle = scope.spawn(move || {
            let output = work();
            let _ = done_sender.send(());
            output
        });
        match done_receiver.recv_timeout(deadline) {
            Ok(()) => handle.join().map_err(|_| "做事的线程 panic 了".to_string()),
            Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => {
                on_timeout();
                let _ = handle.join();
                Err(format!(
                    "{} 秒没做完，卡住了（到点置了停标记）",
                    deadline.as_secs()
                ))
            }
        }
    })
}

/// 第 `frame` 帧写的 5 个键与值。
fn frame_entries(prefix: &str, frame: u64) -> Vec<(Table, Vec<u8>, Vec<u8>)> {
    (0..5)
        .map(|index| {
            (
                Table::Conclusions,
                bytes(&format!("{prefix}/{frame:03}/{index}")),
                bytes(&format!("value-{frame}-{index}")),
            )
        })
        .collect()
}

/// 直连服务读回 `frames` 帧写的全部键，逐个对上；回读了几个键。
fn verify_frames(route: &RouteAddress, prefix: &str, frames: u64) -> Result<usize, String> {
    let mut reader = SimpleClient::connect(route, BreakSwitches::none())?;
    let expected: Vec<(Table, Vec<u8>, Vec<u8>)> = (1..=frames)
        .flat_map(|frame| frame_entries(prefix, frame))
        .collect();
    let keys: Vec<(Table, Vec<u8>)> = expected
        .iter()
        .map(|(table, key, _)| (*table, key.clone()))
        .collect();
    let values = reader.read(&keys)?;
    for ((_, key, value), received) in expected.iter().zip(&values) {
        if received.as_deref() != Some(value.as_slice()) {
            return Err(format!(
                "键 {} 读回 {:?}，该是 {}（这一帧丢了，或被别的帧改了）",
                String::from_utf8_lossy(key),
                received.as_deref().map(String::from_utf8_lossy),
                String::from_utf8_lossy(value)
            ));
        }
    }
    Ok(keys.len())
}

fn read_register(route: &RouteAddress, key: &str) -> Result<Option<u64>, String> {
    let mut reader = SimpleClient::connect(route, BreakSwitches::none())?;
    let values = reader.read(&[(Table::Conclusions, bytes(key))])?;
    Ok(values
        .first()
        .cloned()
        .flatten()
        .and_then(|value| value.try_into().ok())
        .map(u64::from_be_bytes))
}

fn register_entry(key: &str, frame: u64) -> (Table, Vec<u8>, Vec<u8>) {
    (Table::Conclusions, bytes(key), frame.to_be_bytes().to_vec())
}

/// 「不丢」那一格：断线落在一帧中间（另加一次吞掉回应再断）与服务 `kill -9` 后重起两种情形，库里逐键等于每帧恰好生效一次的结果。
pub(crate) fn cell_replay_after_disconnect_and_server_kill(
    context: &CellContext,
) -> Result<String, String> {
    let disconnect = replay_after_mid_frame_disconnect(context)?;
    let server_kill = replay_after_server_kill(context)?;
    Ok(format!("{disconnect}；{server_kill}"))
}

fn replay_after_mid_frame_disconnect(context: &CellContext) -> Result<String, String> {
    const FRAMES: u64 = 40;
    const REGISTER: &str = "replay/register";
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    let directory = CellDirectory::create()?;
    let mut proxy = FaultProxy::start(
        ProxyListen::UnixSocket(directory.path.join("proxy.sock")),
        server.unix_route(),
        &BreakSwitches::none(),
    )?;
    let control = proxy.shared_control();
    let stop = Arc::new(AtomicBool::new(false));
    let settings = client_settings(
        proxy.route(),
        directory.path.join("client.journal"),
        &context.break_switches,
        &stop,
    );
    let diagnostics = within_deadline(
        Duration::from_secs(120),
        || stop.store(true, Ordering::SeqCst),
        || -> Result<ClientDiagnostics, ClientFailure> {
            let mut client = ReliableClient::open(settings)?;
            for frame in 1..=FRAMES {
                if frame == 11 {
                    control.cut_mid_frame_requests.store(1, Ordering::SeqCst);
                }
                if frame == 26 {
                    control
                        .swallow_responses_then_cut_requests
                        .store(1, Ordering::SeqCst);
                }
                let mut entries = frame_entries("replay", frame);
                entries.push(register_entry(REGISTER, frame));
                client.submit_write_entries(&entries)?;
            }
            client.wait_all_acknowledged()?;
            Ok(client.diagnostics().clone())
        },
    )?
    .map_err(|failure| format!("客户端出错：{}", failure_text(&failure)))?;
    let cuts = control.cuts_done.load(Ordering::SeqCst);
    proxy.stop();
    let keys = verify_frames(&server.unix_route(), "replay", FRAMES)?;
    let register = read_register(&server.unix_route(), REGISTER)?;
    if register != Some(FRAMES) {
        return Err(format!(
            "每帧都改写的那个键读回 {register:?}，该是最后一帧 {FRAMES}"
        ));
    }
    if cuts != 2 || diagnostics.reconnections < 2 {
        return Err(format!(
            "代理断了 {cuts} 次（该是 2 次：断在帧中间、吞掉回应再断）、客户端重连 {} 次（该至少 2 次）：故障没注入上，这一格没测到重放",
            diagnostics.reconnections
        ));
    }
    Ok(format!(
        "断在帧中间与吞掉回应再断各一次，客户端重连 {} 次、重发 {} 帧，{FRAMES} 帧 {keys} 个键逐个对上、每帧改写的键是最后一帧",
        diagnostics.reconnections, diagnostics.replayed_frames
    ))
}

/// 能 `kill -9` 再在同一个库、同一个 socket 上重起的服务。
struct RestartableServer {
    directory: CellDirectory,
    library: PathBuf,
    extra: Vec<String>,
    process: Option<SpawnedProcess>,
    unix_socket: PathBuf,
}

impl RestartableServer {
    fn start(context: &CellContext, extra: &[&str]) -> Result<RestartableServer, String> {
        let directory = CellDirectory::create()?;
        let library = directory.path.join("library");
        let mut server = RestartableServer {
            unix_socket: directory.path.join("server.sock"),
            directory,
            library,
            extra: extra
                .iter()
                .map(|argument| (*argument).to_string())
                .collect(),
            process: None,
        };
        server.spawn(context)?;
        Ok(server)
    }

    fn spawn(&mut self, context: &CellContext) -> Result<(), String> {
        let mut command = server_command(context, &self.directory.path, &self.library)?;
        command.args(loopback_arguments()).args(&self.extra);
        let mut process = SpawnedProcess::spawn(command, StandardError::Capture)?;
        let line = process
            .wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)
            .map_err(|message| {
                format!(
                    "服务没起来：{message}；标准错误：{}",
                    process.standard_error_text()
                )
            })?;
        let unix_socket = PathBuf::from(
            line_field(&line, "unix").ok_or_else(|| format!("就绪行没有 unix=：{line}"))?,
        );
        if unix_socket != self.unix_socket {
            return Err(format!(
                "服务挂在 {}，该是 {}",
                unix_socket.display(),
                self.unix_socket.display()
            ));
        }
        self.process = Some(process);
        Ok(())
    }

    fn kill(&mut self) -> Result<(), String> {
        let Some(mut process) = self.process.take() else {
            return Ok(());
        };
        process.send_signal(libc::SIGKILL)?;
        process
            .wait_for_exit(SERVER_READY_TIMEOUT)
            .ok_or("服务收了 SIGKILL 还没退出")?;
        Ok(())
    }

    fn route(&self) -> RouteAddress {
        RouteAddress::UnixSocket(self.unix_socket.clone())
    }

    fn path(&self) -> &Path {
        &self.directory.path
    }
}

fn replay_after_server_kill(context: &CellContext) -> Result<String, String> {
    const ACKNOWLEDGED_FRAMES: u64 = 30;
    const STREAMED_FRAMES: u64 = 60;
    const KILL_AFTER_FRAMES: u64 = 20;
    const REGISTER: &str = "kill/register";
    let mut server = RestartableServer::start(context, &[])?;
    let stop = Arc::new(AtomicBool::new(false));
    // 确认过的帧：全部确认之后立刻 kill -9，重起之后一个都不许少（确认之前已落盘）。
    let acknowledged_settings = client_settings(
        server.route(),
        server.path().join("acknowledged.journal"),
        &context.break_switches,
        &stop,
    );
    {
        let mut client = ReliableClient::open(acknowledged_settings)
            .map_err(|failure| failure_text(&failure))?;
        for frame in 1..=ACKNOWLEDGED_FRAMES {
            client
                .submit_write_entries(&frame_entries("kill/acknowledged", frame))
                .map_err(|failure| failure_text(&failure))?;
        }
        client
            .wait_all_acknowledged()
            .map_err(|failure| failure_text(&failure))?;
        server.kill()?;
    }
    server.spawn(context)?;
    verify_frames(&server.route(), "kill/acknowledged", ACKNOWLEDGED_FRAMES)
        .map_err(|message| format!("确认过的帧 kill -9 之后丢了（确认早于落盘）：{message}"))?;
    // 流着写的帧：写到一半 kill -9、300 毫秒后重起，客户端重连重放，最后逐个对上。
    let streamed_settings = client_settings(
        server.route(),
        server.path().join("streamed.journal"),
        &context.break_switches,
        &stop,
    );
    let submitted = AtomicU64::new(0);
    // 写到第 KILL_AFTER_FRAMES 帧就停下等服务被杀：机器忙时这一格的主线程可能一直轮不上，不等的话客户端会在 kill 之前
    // 把 60 帧都写完、都确认（量到过：开关复跑 2168 里这一格因此判成「客户端没重连过」，与那一遍的开关无关）。
    let killed = AtomicBool::new(false);
    let outcome = thread::scope(|scope| -> Result<ClientDiagnostics, String> {
        let writer = scope.spawn(|| -> Result<ClientDiagnostics, ClientFailure> {
            let mut client = ReliableClient::open(streamed_settings)?;
            for frame in 1..=STREAMED_FRAMES {
                let mut entries = frame_entries("kill/streamed", frame);
                entries.push(register_entry(REGISTER, frame));
                client.submit_write_entries(&entries)?;
                submitted.store(frame, Ordering::SeqCst);
                if frame == KILL_AFTER_FRAMES {
                    while !killed.load(Ordering::SeqCst) && !stop.load(Ordering::SeqCst) {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            }
            client.wait_all_acknowledged()?;
            Ok(client.diagnostics().clone())
        });
        let waited_from = Instant::now();
        while submitted.load(Ordering::SeqCst) < KILL_AFTER_FRAMES && !writer.is_finished() {
            if waited_from.elapsed() > Duration::from_secs(120) {
                stop.store(true, Ordering::SeqCst);
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        let restart = server.kill().and_then(|()| {
            killed.store(true, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(300));
            server.spawn(context)
        });
        if let Err(message) = restart {
            stop.store(true, Ordering::SeqCst);
            let _ = writer.join();
            return Err(format!("kill -9 之后重起服务失败：{message}"));
        }
        writer
            .join()
            .map_err(|_| "写的线程 panic 了".to_string())?
            .map_err(|failure| format!("客户端出错：{}", failure_text(&failure)))
    })?;
    let keys = verify_frames(&server.route(), "kill/streamed", STREAMED_FRAMES)?;
    let register = read_register(&server.route(), REGISTER)?;
    if register != Some(STREAMED_FRAMES) {
        return Err(format!(
            "每帧都改写的那个键读回 {register:?}，该是最后一帧 {STREAMED_FRAMES}"
        ));
    }
    if outcome.reconnections < 1 {
        return Err("写到一半 kill -9 了服务，客户端却没重连过：这一格没测到重放".to_string());
    }
    Ok(format!(
        "确认过的 {ACKNOWLEDGED_FRAMES} 帧 kill -9 之后一个不少；写到第 {KILL_AFTER_FRAMES} 帧时 kill -9、重起，客户端重连 {} 次、重发 {} 帧，{STREAMED_FRAMES} 帧 {keys} 个键逐个对上",
        outcome.reconnections, outcome.replayed_frames
    ))
}

pub(crate) fn cell_delay_jitter_duplicate_tolerated(
    context: &CellContext,
) -> Result<String, String> {
    const PIPELINED_FRAMES: u64 = 30;
    const ORDERED_FRAMES: u64 = 9;
    const REGISTER: &str = "jitter/register";
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    let directory = CellDirectory::create()?;
    let mut proxy = FaultProxy::start(
        ProxyListen::UnixSocket(directory.path.join("proxy.sock")),
        server.unix_route(),
        &BreakSwitches::none(),
    )?;
    let control = proxy.shared_control();
    control
        .delay_minimum_milliseconds
        .store(0, Ordering::SeqCst);
    control
        .delay_maximum_milliseconds
        .store(15, Ordering::SeqCst);
    control.duplicate_every_frames.store(3, Ordering::SeqCst);
    let stop = Arc::new(AtomicBool::new(false));
    let settings = client_settings(
        proxy.route(),
        directory.path.join("client.journal"),
        &context.break_switches,
        &stop,
    );
    let diagnostics = within_deadline(
        Duration::from_secs(120),
        || stop.store(true, Ordering::SeqCst),
        || -> Result<ClientDiagnostics, ClientFailure> {
            let mut client = ReliableClient::open(settings)?;
            for frame in 1..=PIPELINED_FRAMES {
                client.submit_write_entries(&frame_entries("jitter", frame))?;
            }
            client.wait_all_acknowledged()?;
            // 每一帧都留一份、在下一帧之后再送一遍：旧帧若再生效一次，会把这个键改回旧值。
            control.duplicate_every_frames.store(1, Ordering::SeqCst);
            for frame in 1..=ORDERED_FRAMES {
                client.submit_write_entries(&[register_entry(REGISTER, frame)])?;
                client.wait_all_acknowledged()?;
            }
            Ok(client.diagnostics().clone())
        },
    )?
    .map_err(|failure| format!("客户端出错：{}", failure_text(&failure)))?;
    let duplicates_injected = control.duplicates_injected.load(Ordering::SeqCst);
    proxy.stop();
    let keys = verify_frames(&server.unix_route(), "jitter", PIPELINED_FRAMES)?;
    let register = read_register(&server.unix_route(), REGISTER)?;
    if register != Some(ORDERED_FRAMES) {
        return Err(format!(
            "逐帧改写的键读回 {register:?}，该是最后一帧 {ORDERED_FRAMES}：迟到的重复帧又生效了一次"
        ));
    }
    let mut statistics_client = SimpleClient::connect(&server.unix_route(), BreakSwitches::none())?;
    let server_duplicates = counter(&statistics_client.statistics()?, DUPLICATE_REQUESTS_COUNTER)?;
    if duplicates_injected == 0 || diagnostics.duplicate_responses == 0 || server_duplicates == 0 {
        return Err(format!(
            "代理注入重复 {duplicates_injected} 次、客户端收到重复回应 {} 次、服务认出重复请求 {server_duplicates} 次：该都大于 0，这一格没测到重复",
            diagnostics.duplicate_responses
        ));
    }
    Ok(format!(
        "两个方向每帧延迟 0 到 15 毫秒、注入迟到的重复 {duplicates_injected} 次：{PIPELINED_FRAMES} 帧 {keys} 个键对上，逐帧改写的键是最后一帧，服务去重 {server_duplicates} 次，客户端丢掉重复回应 {} 次，没卡住",
        diagnostics.duplicate_responses
    ))
}

/// 经代理连着的服务被 `kill -9` 之后，客户端接着写两帧大过发送缓冲（各 4 MiB）的帧：客户端要看到连接断了、重连到重起的服务、
/// 两帧都落库，不许堵在写里。代理在服务那头没了时要两头都关；只半关客户端那头的写、不再读它的请求，头一帧被代理读走后撞上死掉的服务，
/// 第二帧就一直堵在客户端的写里（自证 fault-injection-bench-finishes 在机器忙时撞上过这一种，这一格不靠时机，每一趟都走到）。
pub(crate) fn cell_server_kill_while_client_writes(
    context: &CellContext,
) -> Result<String, String> {
    const BIG_VALUE_BYTES: usize = 4 * 1024 * 1024;
    const DEADLINE: Duration = Duration::from_secs(120);
    let mut server = RestartableServer::start(context, &[])?;
    let mut proxy = FaultProxy::start(
        ProxyListen::UnixSocket(server.path().join("proxy.sock")),
        server.route(),
        &context
            .break_switches
            .only(BreakSwitch::FaultProxyHalfClosesOnServerDeath),
    )?;
    let control = proxy.shared_control();
    let stop = Arc::new(AtomicBool::new(false));
    let settings = client_settings(
        proxy.route(),
        server.path().join("big-frames.journal"),
        &context.break_switches,
        &stop,
    );
    let mut client = ReliableClient::open(settings).map_err(|failure| failure.to_string())?;
    client
        .submit_write_entries(&[register_entry("big/before-kill", 1)])
        .and_then(|_| client.wait_all_acknowledged())
        .map_err(|failure| format!("kill -9 之前那一帧没写成：{failure}"))?;
    server.kill()?;
    // 先重起再写：代理那条旧连接通的还是死掉的那一份服务，客户端写的帧照样撞上它；新服务等着客户端重连过来。
    server.spawn(context)?;
    let big_entries: Vec<(Table, Vec<u8>, Vec<u8>)> =
        ["big/after-kill-first", "big/after-kill-second"]
            .iter()
            .map(|key| (Table::Conclusions, bytes(key), vec![0xA5; BIG_VALUE_BYTES]))
            .collect();
    let written = within_deadline(
        DEADLINE,
        || {
            stop.store(true, Ordering::SeqCst);
            control.cut_all_connections();
        },
        || -> Result<ClientDiagnostics, ClientFailure> {
            for entry in &big_entries {
                client.submit_write_entries(std::slice::from_ref(entry))?;
            }
            client.wait_all_acknowledged()?;
            Ok(client.diagnostics().clone())
        },
    );
    proxy.stop();
    let diagnostics = match written {
        Ok(Ok(diagnostics)) => diagnostics,
        Ok(Err(failure)) => {
            return Err(format!(
                "kill -9 之后写两帧大帧失败：{failure}（代理在服务那头没了时要两头都关，客户端才看得到连接断了）"
            ))
        }
        Err(message) => {
            return Err(format!(
                "kill -9 之后写两帧 4 MiB 的大帧：{message}——客户端堵在写里（代理只半关了客户端那头、不再读它的请求）"
            ))
        }
    };
    if diagnostics.write_timeouts != 0 {
        return Err(format!(
            "kill -9 之后客户端撞了 {} 次写超时才重连（最近一次断的原因：{}）：代理没把客户端那头真关掉，客户端该当场读到流尾或写出错",
            diagnostics.write_timeouts,
            diagnostics.last_break_reason.as_deref().unwrap_or("没记原因")
        ));
    }
    let keys: Vec<(Table, Vec<u8>)> = big_entries
        .iter()
        .map(|(table, key, _)| (*table, key.clone()))
        .collect();
    let mut reader = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    let values = reader.read(&keys)?;
    for ((_, key, value), read_back) in big_entries.iter().zip(&values) {
        if read_back.as_deref() != Some(value.as_slice()) {
            return Err(format!(
                "{} 读回 {} 字节，该是 {BIG_VALUE_BYTES} 字节的 0xA5",
                String::from_utf8_lossy(key),
                read_back.as_ref().map_or(0, Vec::len)
            ));
        }
    }
    if diagnostics.reconnections == 0 {
        return Err("kill -9 之后客户端一次都没重连：这一格没走到要判的局面".to_string());
    }
    Ok(format!(
        "kill -9 再重起之后接着写两帧 4 MiB：客户端看到连接断了（{}）、重连 {} 次、重发 {} 帧，两帧逐字节读回对上",
        diagnostics
            .last_break_reason
            .as_deref()
            .unwrap_or("没记原因"),
        diagnostics.reconnections,
        diagnostics.replayed_frames
    ))
}

pub(crate) fn cell_half_open_connection_recovered(context: &CellContext) -> Result<String, String> {
    const FRAMES: u64 = 15;
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    let mut proxy = FaultProxy::start(
        ProxyListen::Network("127.0.0.1:0".to_string()),
        server.network_route(),
        &BreakSwitches::none(),
    )?;
    let control = proxy.shared_control();
    control.half_open_after_frames.store(5, Ordering::SeqCst);
    control
        .half_open_connections_remaining
        .store(1, Ordering::SeqCst);
    let directory = CellDirectory::create()?;
    let stop = Arc::new(AtomicBool::new(false));
    let mut settings = client_settings(
        proxy.route(),
        directory.path.join("client.journal"),
        &context.break_switches,
        &stop,
    );
    // 5 秒：比别的弄坏开关拖慢提交线程的最长一下（acknowledge-before-sync 回确认之后睡 2 秒）长，平常的帧不会被当成超时。
    settings.frame_timeout = Duration::from_secs(5);
    settings.maximum_in_flight_frames = 4;
    let proxy_control_for_timeout = Arc::clone(&control);
    let outcome = within_deadline(
        Duration::from_secs(90),
        || {
            stop.store(true, Ordering::SeqCst);
            proxy_control_for_timeout
                .half_open_connections_remaining
                .store(0, Ordering::SeqCst);
        },
        || -> Result<(Option<KeepaliveSettings>, ClientDiagnostics), ClientFailure> {
            let mut client = ReliableClient::open(settings)?;
            let keepalive = client
                .current_keepalive()
                .map_err(ClientFailure::Protocol)?;
            for frame in 1..=FRAMES {
                client.submit_write_entries(&frame_entries("half-open", frame))?;
            }
            client.wait_all_acknowledged()?;
            Ok((keepalive, client.diagnostics().clone()))
        },
    );
    let half_open_done = control.half_open_done.load(Ordering::SeqCst);
    proxy.stop();
    let (keepalive, diagnostics) =
        outcome?.map_err(|failure| format!("客户端出错：{}", failure_text(&failure)))?;
    let expected_keepalive = KeepaliveSettings::default();
    if keepalive != Some(expected_keepalive) {
        return Err(format!(
            "客户端的 TCP 连接在内核里的 keepalive 是 {keepalive:?}，该是 {expected_keepalive:?}"
        ));
    }
    let keys = verify_frames(&server.unix_route(), "half-open", FRAMES)?;
    if half_open_done != 1 || diagnostics.frame_timeouts == 0 || diagnostics.reconnections == 0 {
        return Err(format!(
            "代理造半开 {half_open_done} 次、客户端帧超时 {} 次、重连 {} 次：该都大于 0，这一格没测到半开",
            diagnostics.frame_timeouts, diagnostics.reconnections
        ));
    }
    Ok(format!(
        "TCP keepalive 读回 {expected_keepalive:?}；连接转满 5 帧后半开，客户端帧超时 {} 次、重连 {} 次，{FRAMES} 帧 {keys} 个键对上",
        diagnostics.frame_timeouts, diagnostics.reconnections
    ))
}

/// 一轮重连里第 `gap_index` 个间隔（第 gap_index+1 次与第 gap_index+2 次尝试之间）的名义值：起始间隔翻倍、封顶。
fn nominal_gap(policy: ReconnectPolicy, gap_index: usize) -> Duration {
    let doublings = u32::try_from(gap_index.min(20)).expect("不超过 20");
    policy
        .initial_delay
        .saturating_mul(1_u32 << doublings)
        .min(policy.maximum_delay)
}

/// 核两轮重连的尝试次数、间隔翻倍与抖动；回间隔的说明。
fn verify_backoff(cycles: &[Vec<Instant>], policy: ReconnectPolicy) -> Result<String, String> {
    let attempts_expected = usize::try_from(policy.maximum_attempts).expect("u32 放得进 usize");
    let mut jittered_gaps = 0;
    let mut described = Vec::new();
    for attempts in cycles {
        if attempts.len() != attempts_expected {
            return Err(format!(
                "一轮重连试了 {} 次，上限是 {attempts_expected} 次",
                attempts.len()
            ));
        }
        let gaps: Vec<Duration> = attempts
            .windows(2)
            .map(|pair| pair[1].duration_since(pair[0]))
            .collect();
        let early_maximum = gaps.iter().take(2).max().copied().unwrap_or_default();
        let late_minimum = gaps.iter().skip(5).min().copied().unwrap_or_default();
        if late_minimum <= early_maximum * 2 {
            return Err(format!(
                "重连间隔没往上翻倍：头两个间隔最大 {early_maximum:?}，第六个起最小 {late_minimum:?}（间隔：{gaps:?}）"
            ));
        }
        for (gap_index, gap) in gaps.iter().enumerate() {
            let nominal = nominal_gap(policy, gap_index);
            if nominal >= Duration::from_millis(320) && *gap < nominal.mul_f64(0.9) {
                jittered_gaps += 1;
            }
        }
        described.push(format!(
            "{:?}",
            gaps.iter().map(|gap| gap.as_millis()).collect::<Vec<_>>()
        ));
    }
    if jittered_gaps == 0 {
        return Err(format!(
            "名义值 320 毫秒以上的间隔没有一个短于名义值的 0.9 倍：重连没加抖动（各轮间隔毫秒：{}）",
            described.join("；")
        ));
    }
    Ok(described.join("；"))
}

pub(crate) fn cell_reconnect_gives_up_at_limit(context: &CellContext) -> Result<String, String> {
    const GIVE_UP_POLICY: ReconnectPolicy = ReconnectPolicy {
        initial_delay: Duration::from_millis(40),
        maximum_delay: Duration::from_secs(2),
        maximum_attempts: 9,
        maximum_total: Duration::from_secs(60),
    };
    let mut server = RestartableServer::start(context, &[])?;
    let stop = Arc::new(AtomicBool::new(false));
    let journal = server.path().join("client.journal");
    let mut settings = client_settings(
        server.route(),
        journal.clone(),
        &context.break_switches,
        &stop,
    );
    settings.reconnect = GIVE_UP_POLICY;
    settings.frame_timeout = Duration::from_secs(10);
    let mut client = ReliableClient::open(settings).map_err(|failure| failure_text(&failure))?;
    client
        .submit_write_entries(&[(
            Table::Conclusions,
            bytes("give-up/acknowledged"),
            bytes("present"),
        )])
        .and_then(|_| client.wait_all_acknowledged())
        .map_err(|failure| failure_text(&failure))?;
    server.kill()?;
    let cycles_before = client.diagnostics().connection_attempt_cycles.len();
    let write_result = within_deadline(
        Duration::from_secs(40),
        || stop.store(true, Ordering::SeqCst),
        || {
            client
                .submit_write_entries(&[(
                    Table::Conclusions,
                    bytes("give-up/pending"),
                    bytes("replayed"),
                )])
                .and_then(|_| client.wait_all_acknowledged())
        },
    )?;
    match &write_result {
        Err(ClientFailure::ServerUnreachable { .. }) => {}
        other => return Err(format!("服务停了，写该回「连不上服务」，回的是 {other:?}")),
    }
    let read_result = within_deadline(
        Duration::from_secs(40),
        || stop.store(true, Ordering::SeqCst),
        || client.read(&[(Table::Conclusions, bytes("give-up/acknowledged"))]),
    )?;
    match &read_result {
        Err(ClientFailure::ServerUnreachable { .. }) => {}
        other => {
            return Err(format!(
                "服务停了，读该回「连不上服务」的错误、不许当成键不在，回的是 {other:?}"
            ))
        }
    }
    let cycles = client.diagnostics().connection_attempt_cycles[cycles_before..].to_vec();
    let gaps = verify_backoff(&cycles, GIVE_UP_POLICY)?;
    drop(client);
    server.spawn(context)?;
    let reopened_settings =
        client_settings(server.route(), journal, &context.break_switches, &stop);
    let recovered = within_deadline(
        Duration::from_secs(60),
        || stop.store(true, Ordering::SeqCst),
        || -> Result<u64, ClientFailure> {
            let mut reopened = ReliableClient::open(reopened_settings)?;
            reopened.wait_all_acknowledged()?;
            Ok(reopened.diagnostics().journal_recovered_writes)
        },
    )?
    .map_err(|failure| format!("重开客户端出错：{}", failure_text(&failure)))?;
    let replayed = read_register_bytes(&server.route(), "give-up/pending")?;
    if replayed.as_deref() != Some(b"replayed".as_slice()) || recovered != 1 {
        return Err(format!(
            "放弃重连时没确认的那一帧：重开客户端从追加日志读回 {recovered} 帧（该是 1），库里读回 {:?}（该是 replayed）",
            replayed.as_deref().map(String::from_utf8_lossy)
        ));
    }
    Ok(format!(
        "服务停了：写与读各重连 {} 次到上限、回「连不上服务」（读不当成键不在）；间隔翻倍、有抖动（毫秒：{gaps}）；重开客户端从追加日志重放那一帧、库里读得到",
        GIVE_UP_POLICY.maximum_attempts
    ))
}

fn read_register_bytes(route: &RouteAddress, key: &str) -> Result<Option<Vec<u8>>, String> {
    let mut reader = SimpleClient::connect(route, BreakSwitches::none())?;
    Ok(reader
        .read(&[(Table::Conclusions, bytes(key))])?
        .into_iter()
        .next()
        .flatten())
}

pub(crate) fn cell_busy_backpressure_bounded(context: &CellContext) -> Result<String, String> {
    const CLIENTS: u64 = 3;
    const FRAMES_PER_CLIENT: u64 = 20;
    const PENDING_LIMIT: u64 = 6;
    const BUSY_RETRY_MILLISECONDS: u64 = 30;
    let server = start_test_server(
        context,
        &loopback_arguments(),
        &[
            "--maximum-pending-per-connection",
            "4",
            "--maximum-pending-total",
            "6",
            "--busy-retry-milliseconds",
            "30",
            "--selftest-commit-delay-milliseconds",
            "40",
            "--group-commit-frames",
            "2",
        ],
    )?;
    let directory = CellDirectory::create()?;
    let stop = Arc::new(AtomicBool::new(false));
    // 线程里只拿路由，不借服务句柄（句柄里收标准输出的通道不能跨线程共享）。
    let route = server.unix_route();
    let results = within_deadline(
        Duration::from_secs(180),
        || stop.store(true, Ordering::SeqCst),
        || {
            thread::scope(|scope| {
                let handles: Vec<_> = (1..=CLIENTS)
                    .map(|client_number| {
                        let mut settings = client_settings(
                            route.clone(),
                            directory
                                .path
                                .join(format!("client-{client_number}.journal")),
                            &context.break_switches,
                            &stop,
                        );
                        settings.maximum_busy_retries = 100_000;
                        scope.spawn(move || -> Result<ClientDiagnostics, ClientFailure> {
                            let mut client = ReliableClient::open(settings)?;
                            for frame in 1..=FRAMES_PER_CLIENT {
                                client.submit_write_entries(&frame_entries(
                                    &format!("busy/{client_number}"),
                                    frame,
                                ))?;
                            }
                            client.wait_all_acknowledged()?;
                            Ok(client.diagnostics().clone())
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|handle| {
                        handle.join().unwrap_or_else(|_| {
                            Err(ClientFailure::Protocol("客户端线程 panic 了".to_string()))
                        })
                    })
                    .collect::<Vec<_>>()
            })
        },
    )?;
    let mut busy_responses = 0;
    let mut shortest_extra_wait: Option<Duration> = None;
    let mut busy_waits = 0;
    for result in results {
        let diagnostics =
            result.map_err(|failure| format!("客户端出错：{}", failure_text(&failure)))?;
        busy_responses += diagnostics.busy_responses;
        for (waited, requested) in &diagnostics.busy_waits {
            busy_waits += 1;
            let extra = waited.checked_sub(*requested).unwrap_or_default();
            if *waited < *requested {
                return Err(format!(
                    "回了忙之后只等了 {waited:?} 就重发，服务叫等 {requested:?}"
                ));
            }
            shortest_extra_wait =
                Some(shortest_extra_wait.map_or(extra, |shortest| shortest.min(extra)));
        }
    }
    for client_number in 1..=CLIENTS {
        verify_frames(
            &server.unix_route(),
            &format!("busy/{client_number}"),
            FRAMES_PER_CLIENT,
        )?;
    }
    let mut statistics_client = SimpleClient::connect(&server.unix_route(), BreakSwitches::none())?;
    let pending_peak = counter(&statistics_client.statistics()?, PENDING_PEAK_COUNTER)?;
    if busy_responses == 0 || pending_peak > PENDING_LIMIT {
        return Err(format!(
            "服务变慢时客户端收到忙 {busy_responses} 次（该大于 0）、服务待处理帧数峰值 {pending_peak}（上限 {PENDING_LIMIT}）：待处理没有界，帧排进了队列"
        ));
    }
    Ok(format!(
        "服务每组提交多等 40 毫秒、待处理上限 {PENDING_LIMIT} 帧：{CLIENTS} 个客户端收到忙 {busy_responses} 次、每次至少等服务给的 {BUSY_RETRY_MILLISECONDS} 毫秒再重发（{busy_waits} 次重发，最少多等 {:?}），待处理峰值 {pending_peak}，{} 帧全部落库",
        shortest_extra_wait.unwrap_or_default(),
        CLIENTS * FRAMES_PER_CLIENT
    ))
}

fn claimed_by(outcomes: &[ClaimOutcome], position: usize) -> bool {
    matches!(outcomes.get(position), Some(ClaimOutcome::Claimed))
}

fn held_by(outcomes: &[ClaimOutcome], position: usize, holder: &[u8]) -> bool {
    matches!(outcomes.get(position), Some(ClaimOutcome::HeldByOther { holder_identity, .. }) if holder_identity == holder)
}

fn claim_keys(prefix: &str, range: std::ops::RangeInclusive<u32>) -> Vec<(Table, Vec<u8>)> {
    range
        .map(|number| (Table::Conclusions, bytes(&format!("{prefix}/{number:02}"))))
        .collect()
}

pub(crate) fn cell_claim_exclusive_and_lease_expiry(
    context: &CellContext,
) -> Result<String, String> {
    const LEASE_MILLISECONDS: u32 = 60_000;
    // 10 秒：持有者起来、kill -9、收尸、别人再认领这一串在同机重活压着时要几秒，提交线程被别的开关拖慢时每组还多 2 秒；
    // 租期要比这一串长，「租期没到」那一问才问得到。
    const HOLDER_LEASE_MILLISECONDS: u32 = 10_000;
    let server = start_test_server(context, &loopback_arguments(), &[])?;
    let directory = CellDirectory::create()?;
    let stop = Arc::new(AtomicBool::new(false));
    let first_keys = claim_keys("claim", 1..=20);
    let second_keys = claim_keys("claim", 11..=30);
    let barrier = Barrier::new(2);
    // 线程里只拿路由，不借服务句柄（句柄里收标准输出的通道不能跨线程共享）。
    let route = server.unix_route();
    let claim_with = |journal: &str,
                      keys: &[(Table, Vec<u8>)]|
     -> Result<(ReliableClient, Vec<ClaimOutcome>), String> {
        let settings = client_settings(
            route.clone(),
            directory.path.join(journal),
            &context.break_switches,
            &stop,
        );
        let mut client =
            ReliableClient::open(settings).map_err(|failure| failure_text(&failure))?;
        barrier.wait();
        let outcomes = client
            .claim(keys, LEASE_MILLISECONDS)
            .map_err(|failure| failure_text(&failure))?;
        Ok((client, outcomes))
    };
    let (first, second) = thread::scope(|scope| {
        let first = scope.spawn(|| claim_with("first.journal", &first_keys));
        let second = scope.spawn(|| claim_with("second.journal", &second_keys));
        (
            first
                .join()
                .unwrap_or_else(|_| Err("认领线程 panic 了".to_string())),
            second
                .join()
                .unwrap_or_else(|_| Err("认领线程 panic 了".to_string())),
        )
    });
    let (first_client, first_outcomes) = first?;
    let (second_client, second_outcomes) = second?;
    let first_identity = first_client.client_identity().to_vec();
    let second_identity = second_client.client_identity().to_vec();
    for number in 1..=30_usize {
        let in_first = number <= 20;
        let in_second = number >= 11;
        let first_claimed = in_first && claimed_by(&first_outcomes, number - 1);
        let second_claimed = in_second && claimed_by(&second_outcomes, number - 11);
        let loser_sees_winner = match (in_first, in_second, first_claimed, second_claimed) {
            (true, true, true, false) => held_by(&second_outcomes, number - 11, &first_identity),
            (true, true, false, true) => held_by(&first_outcomes, number - 1, &second_identity),
            (true, false, true, false) | (false, true, false, true) => true,
            (_, _, _, _) => false,
        };
        if !loser_sees_winner {
            return Err(format!(
                "键 claim/{number:02}：甲认领到 {first_claimed}、乙认领到 {second_claimed}，该恰好一个认领到、另一个看到「别人持有」且持有者是认领到的那个（甲 {:?}，乙 {:?}）",
                first_outcomes.get(number.wrapping_sub(1)),
                second_outcomes.get(number.wrapping_sub(11))
            ));
        }
    }
    let winner_of_overlap = if claimed_by(&first_outcomes, 10) {
        &first_identity
    } else {
        &second_identity
    };
    let conclusions: Vec<(Table, Vec<u8>, Vec<u8>)> = claim_keys("claim", 11..=15)
        .into_iter()
        .map(|(table, key)| (table, key, bytes("concluded")))
        .collect();
    let mut concluder = if winner_of_overlap == &first_identity {
        first_client
    } else {
        drop(first_client);
        second_client
    };
    concluder
        .submit_write_entries(&conclusions)
        .and_then(|_| concluder.wait_all_acknowledged())
        .map_err(|failure| failure_text(&failure))?;
    let mut third = SimpleClient::connect(&server.unix_route(), BreakSwitches::none())?;
    let mut probe_keys = claim_keys("claim", 11..=15);
    probe_keys.extend(claim_keys("claim", 16..=16));
    let probe = third.claim(&probe_keys, LEASE_MILLISECONDS)?;
    let concluded_released = probe[..5]
        .iter()
        .all(|outcome| matches!(outcome, ClaimOutcome::AlreadyConcluded));
    if !concluded_released || !held_by(&probe, 5, winner_of_overlap) {
        return Err(format!(
            "写了结论的 claim/11 到 15 该回「已有结论」、没写的 claim/16 该还是认领到的那个持有，回的是 {probe:?}"
        ));
    }
    drop(concluder);
    // 持有者 kill -9：租期没到时别人认领回「别人持有」，租期过了别人认领到。
    let mut holder_command = Command::new(&context.executable);
    holder_command.args([
        "claim-holder",
        "--selftest-instance",
        "--socket",
        &unix_socket_path(&server.unix_route())?
            .display()
            .to_string(),
        "--lease-milliseconds",
        &HOLDER_LEASE_MILLISECONDS.to_string(),
        "--keys",
        "lease/1,lease/2,lease/3",
    ]);
    let mut holder = SpawnedProcess::spawn(holder_command, StandardError::Capture)?;
    let holder_line = holder
        .wait_for_line_starting_with(CLAIM_HOLDER_LINE_PREFIX, SERVER_READY_TIMEOUT)
        .map_err(|message| {
            format!(
                "认领持有者没起来：{message}；标准错误：{}",
                holder.standard_error_text()
            )
        })?;
    holder.send_signal(libc::SIGKILL)?;
    holder
        .wait_for_exit(SERVER_READY_TIMEOUT)
        .ok_or("认领持有者收了 SIGKILL 还没退出")?;
    let holder_identity = line_field(&holder_line, "identity")
        .ok_or_else(|| format!("持有者那一行没有 identity=：{holder_line}"))?;
    if line_field(&holder_line, "claimed").as_deref() != Some("3") {
        return Err(format!("持有者该认领到 3 把键：{holder_line}"));
    }
    let lease_keys = claim_keys_plain(&["lease/1", "lease/2", "lease/3"]);
    let while_held = third.claim(&lease_keys, LEASE_MILLISECONDS)?;
    let remaining_maximum = while_held
        .iter()
        .map(|outcome| match outcome {
            ClaimOutcome::HeldByOther { holder_identity: reported_holder, remaining_lease_milliseconds } if reported_holder.as_slice() == holder_identity.as_bytes() => Some(*remaining_lease_milliseconds),
            ClaimOutcome::Claimed | ClaimOutcome::HeldByOther { .. } | ClaimOutcome::AlreadyConcluded => None,
        })
        .collect::<Option<Vec<u64>>>()
        .and_then(|remaining| remaining.into_iter().max())
        .filter(|remaining| *remaining <= u64::from(HOLDER_LEASE_MILLISECONDS))
        .ok_or_else(|| format!("持有者 kill -9 之后、租期没到时别人认领该回「别人持有」（持有者 {holder_identity}、剩余不超过 {HOLDER_LEASE_MILLISECONDS} 毫秒），回的是 {while_held:?}"))?;
    thread::sleep(Duration::from_millis(remaining_maximum + 300));
    let after_lease = third.claim(&lease_keys, LEASE_MILLISECONDS)?;
    if !after_lease
        .iter()
        .all(|outcome| matches!(outcome, ClaimOutcome::Claimed))
    {
        return Err(format!(
            "持有者 kill -9、租期过了之后别人认领该都认领到，回的是 {after_lease:?}"
        ));
    }
    Ok(format!(
        "甲认领 claim/01 到 20、乙认领 11 到 30 同时发：重叠的 10 把键每把恰好一个认领到、另一个看到持有者；写了结论的回「已有结论」；持有者 kill -9 后剩余租期 {remaining_maximum} 毫秒内回「别人持有」，过了之后别人认领到"
    ))
}

fn claim_keys_plain(keys: &[&str]) -> Vec<(Table, Vec<u8>)> {
    keys.iter()
        .map(|key| (Table::Conclusions, bytes(key)))
        .collect()
}

pub(crate) fn cell_group_commit_coalesces_syncs(context: &CellContext) -> Result<String, String> {
    const CONNECTIONS: u64 = 8;
    const FRAMES_PER_CONNECTION: u64 = 16;
    const TOTAL_FRAMES: u64 = CONNECTIONS * FRAMES_PER_CONNECTION;
    let server = start_test_server(
        context,
        &loopback_arguments(),
        &[
            "--group-commit-milliseconds",
            "20",
            "--group-commit-frames",
            "512",
        ],
    )?;
    let mut statistics_client = SimpleClient::connect(&server.unix_route(), BreakSwitches::none())?;
    let before = statistics_client.statistics()?;
    let mut connections = Vec::new();
    for connection_number in 0..CONNECTIONS {
        let (sender, receiver) = connect(&server.unix_route(), BreakSwitches::none())
            .map_err(|error| format!("连不上服务：{error}"))?;
        receiver
            .set_read_timeout(Some(Duration::from_secs(120)))
            .map_err(|error| format!("设不上读超时：{error}"))?;
        connections.push((
            connection_number,
            random_client_identity()?,
            sender,
            receiver,
        ));
    }
    for (connection_number, identity, sender, _) in &mut connections {
        for frame in 1..=FRAMES_PER_CONNECTION {
            sender
                .send_frame(|sequence| {
                    let mut builder = begin_write_frame(sequence, identity, frame, 3, 128);
                    for index in 0..3 {
                        builder.entry(
                            Table::Facts,
                            format!("group/{connection_number}/{frame}/{index}").as_bytes(),
                            b"grouped",
                        );
                    }
                    builder.finish()
                })
                .map_err(|error| format!("发不出去：{error}"))?;
        }
    }
    let mut acknowledged = 0_u64;
    for (_, _, _, receiver) in &mut connections {
        for _ in 0..FRAMES_PER_CONNECTION {
            let matched = receiver
                .receive()
                .map_err(|failure| format!("收确认失败：{failure}"))?;
            match matched.response.outcome {
                ResponseOutcome::WriteAcknowledged {
                    entries_written: 3,
                    is_already_applied: false,
                } => acknowledged += 1,
                other @ (ResponseOutcome::WriteAcknowledged { .. }
                | ResponseOutcome::PointValues(_)
                | ResponseOutcome::ScanEntries { .. }
                | ResponseOutcome::Statistics(_)
                | ResponseOutcome::Claims(_)
                | ResponseOutcome::PrefixDeleted { .. }
                | ResponseOutcome::ConnectedClients(_)
                | ResponseOutcome::ServerStopping { .. }
                | ResponseOutcome::Busy { .. }
                | ResponseOutcome::Rejected { .. }) => {
                    return Err(format!("写确认不对：{other:?}"))
                }
            }
        }
    }
    let after = statistics_after_sync(&mut statistics_client, &before)?;
    let synced = counter_delta(&before, &after, Ticker::WalFileSynced)?;
    // 每帧 3 条数据加一条「这个请求号已生效」：RocksDB 数到的键数对上，才说明这些帧真写进去了，同步次数少不是因为没写。
    let keys_written = counter_delta(&before, &after, Ticker::NumberKeysWritten)?;
    if acknowledged != TOTAL_FRAMES || keys_written != TOTAL_FRAMES * 4 || synced > TOTAL_FRAMES / 4
    {
        return Err(format!(
            "{CONNECTIONS} 条连接并发写 {TOTAL_FRAMES} 帧、收到确认 {acknowledged} 帧，RocksDB 数到写 {keys_written} 个键（该是 {}）、同步 WAL {synced} 次（该至多 {} 次，跨帧合并同步）",
            TOTAL_FRAMES * 4,
            TOTAL_FRAMES / 4
        ));
    }
    let single_frame = single_frame_single_sync(context)?;
    let keys: Vec<(Table, Vec<u8>)> = (0..CONNECTIONS)
        .map(|connection_number| {
            (
                Table::Facts,
                bytes(&format!(
                    "group/{connection_number}/{FRAMES_PER_CONNECTION}/2"
                )),
            )
        })
        .collect();
    let values = statistics_client.read(&keys)?;
    if values
        .iter()
        .any(|value| value.as_deref() != Some(b"grouped".as_slice()))
    {
        return Err(format!("合并同步之后读回不对：{values:?}"));
    }
    Ok(format!(
        "{single_frame}；{CONNECTIONS} 条连接并发写 {TOTAL_FRAMES} 帧全部确认，RocksDB 数到写 {keys_written} 个键、同步 WAL {synced} 次（一帧一同步该是 {TOTAL_FRAMES} 次）"
    ))
}

pub(crate) fn cell_bench_environment_judgment(context: &CellContext) -> Result<String, String> {
    let switches = &context.break_switches;
    let judgment_cases = [
        (Some(1.0), Some(2.0), 16.0, EnvironmentJudgment::Clean),
        (Some(16.0), Some(16.0), 16.0, EnvironmentJudgment::Clean),
        (Some(17.0), Some(2.0), 16.0, EnvironmentJudgment::Dirty),
        (Some(1.0), Some(16.5), 16.0, EnvironmentJudgment::Dirty),
        (None, Some(1.0), 16.0, EnvironmentJudgment::Dirty),
        (Some(46.46), Some(59.98), 16.0, EnvironmentJudgment::Dirty),
    ];
    for (load_at_start, load_at_end, threshold, expected) in judgment_cases {
        let judged = judge_environment(load_at_start, load_at_end, threshold, switches);
        if judged != expected {
            return Err(format!(
                "起跑前 loadavg {load_at_start:?}、跑完 {load_at_end:?}、阈值 {threshold} 判成 {judged:?}，该是 {expected:?}"
            ));
        }
    }
    let load_averages = parse_load_averages("51.60 57.82 69.82 46/2006 4072408\n");
    if load_averages != Some([51.60, 57.82, 69.82])
        || parse_load_averages("not a load average").is_some()
    {
        return Err(format!(
            "/proc/loadavg 解成 {load_averages:?}，该是 [51.60, 57.82, 69.82]；乱写的该解不出"
        ));
    }
    let diskstats = " 259       0 nvme0n1 9 0 9 9 9 0 9 9 0 777 9 0 0 0 0 0 0\n 259       2 nvme0n1p2 100 0 200 30 400 0 500 60 0 123456 90 0 0 0 0 0 0\n";
    let ticks = parse_input_output_ticks(diskstats, 259, 2);
    if ticks != Some(("nvme0n1p2".to_string(), 123_456))
        || parse_input_output_ticks(diskstats, 8, 0).is_some()
    {
        return Err(format!("diskstats 里 259:2 的 io_ticks 解成 {ticks:?}，该是 nvme0n1p2 的 123456；不在的设备该解不出"));
    }
    let busy = busy_percent(1_000, 1_500, 1_000.0);
    if (busy - 50.0).abs() > f64::EPSILON {
        return Err(format!(
            "io_ticks 1 秒涨 500 毫秒算成忙度 {busy}%，该是 50%"
        ));
    }
    Ok(format!(
        "环境判法 {} 例、loadavg 解析、diskstats 的 io_ticks 解析与忙度换算都对（loadavg 46.46 / 59.98、阈值 16 判不干净）",
        judgment_cases.len()
    ))
}

/// 认领那一格要把服务的 socket 路径交给子进程。
fn unix_socket_path(route: &RouteAddress) -> Result<PathBuf, String> {
    match route {
        RouteAddress::UnixSocket(path) => Ok(path.clone()),
        RouteAddress::Network(address) => Err(format!(
            "要 unix socket 的路径，拿到的是 TCP 地址 {address}"
        )),
    }
}
