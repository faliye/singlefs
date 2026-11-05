//! 控制命令（`prune-server/control`）调的几样，与自证起的客户端子进程。都只走本机的 unix socket。
//!
//!   prune-server admin health [--socket <路径>]             服务通不通：回了统计就打 `PRUNE_SERVER_HEALTH ok frames_handled=…`、退 0
//!   prune-server admin connected-clients [--socket <路径>]  列连着的客户端：每个一行 `PRUNE_SERVER_CLIENT <字段>`，末行 `PRUNE_SERVER_CLIENTS count=… action=listed`
//!   prune-server admin announce-stop [--socket <路径>]      宣布停服务（服务给连着的客户端发「服务停了」），打发通知之前的清单，
//!                                                           末行 `PRUNE_SERVER_CLIENTS count=… action=stop-announced`
//!   连不上服务退 75（与容错客户端「prune-server 不可用」同一个码），服务回了错退 1，用法错退 2。
//!   `PRUNE_SERVER_CLIENT` 行的字段：connection、transport、peer、host、task、attempt、pid、connected_since_milliseconds（`connections::connected_client_fields`）。
//!
//!   prune-server selftest-client --selftest-instance --socket <路径> --journal <文件> --reconnect-total-seconds <数>
//!       只给自证：容错客户端照这个进程的环境报到（任务号取 `TASK_SERVER_TASK`），每 100 毫秒写一帧等确认；头一帧确认之后打
//!       `PRUNE_SERVER_SELFTEST_CLIENT connected …`；客户端出错时打 `PRUNE_SERVER_SELFTEST_CLIENT failed exit_code=… reason=…`，
//!       照 `ClientFailure::exit_code` 退出。弄坏开关读环境里的 `PRUNE_SERVER_BREAK`（起它的那一格只交它要证红的几项）。

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::break_switches::{BreakSwitches, BREAK_VARIABLE};
use crate::client::{RouteAddress, SimpleClient};
use crate::command_line::{parse_positive_number, report_refusal};
use crate::configuration::default_socket_path;
use crate::connections::{announcement_from_environment, connected_client_fields};
use crate::protocol::{ConnectedClient, Table};
use crate::reliable_client::{
    ReconnectPolicy, ReliableClient, ReliableClientSettings, PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
};

pub const HEALTH_LINE_PREFIX: &str = "PRUNE_SERVER_HEALTH";
pub const CLIENT_LINE_PREFIX: &str = "PRUNE_SERVER_CLIENT";
pub const CLIENTS_SUMMARY_LINE_PREFIX: &str = "PRUNE_SERVER_CLIENTS";
pub const SELFTEST_CLIENT_LINE_PREFIX: &str = "PRUNE_SERVER_SELFTEST_CLIENT";

const ADMIN_USAGE: &str =
    "用法：prune-server admin health|connected-clients|announce-stop [--socket <路径>] [--spawned-by-parent]";
const SELFTEST_CLIENT_USAGE: &str = "用法（只给自证）：prune-server selftest-client --selftest-instance --socket <路径> --journal <文件> --reconnect-total-seconds <数>";
/// 自证客户端两帧之间隔多久。
const SELFTEST_CLIENT_WRITE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AdminAction {
    Health,
    ConnectedClients,
    AnnounceStop,
}

fn print_clients(clients: &[ConnectedClient], action: &str) {
    for client in clients {
        println!("{CLIENT_LINE_PREFIX} {}", connected_client_fields(client));
    }
    println!(
        "{CLIENTS_SUMMARY_LINE_PREFIX} count={} action={action}",
        clients.len()
    );
}

pub fn run_admin(arguments: &[String]) -> i32 {
    let action = match arguments.first().map(String::as_str) {
        Some("health") => AdminAction::Health,
        Some("connected-clients") => AdminAction::ConnectedClients,
        Some("announce-stop") => AdminAction::AnnounceStop,
        Some(_) | None => {
            report_refusal(
                &format!("认不得的 admin 子命令：{arguments:?}"),
                ADMIN_USAGE,
            );
            return 2;
        }
    };
    let mut socket_path = None;
    let mut position = 1;
    while position < arguments.len() {
        match arguments[position].as_str() {
            "--spawned-by-parent" => position += 1,
            "--socket" => {
                let Some(value) = arguments.get(position + 1) else {
                    report_refusal("--socket 后面缺值", ADMIN_USAGE);
                    return 2;
                };
                socket_path = Some(PathBuf::from(value));
                position += 2;
            }
            other => {
                report_refusal(&format!("认不得的参数「{other}」"), ADMIN_USAGE);
                return 2;
            }
        }
    }
    let socket_path = match socket_path.map_or_else(default_socket_path, Ok) {
        Ok(path) => path,
        Err(message) => {
            report_refusal(&message, "设 XDG_RUNTIME_DIR，或给 --socket <路径>");
            return 2;
        }
    };
    let mut client = match SimpleClient::connect(
        &RouteAddress::UnixSocket(socket_path.clone()),
        BreakSwitches::none(),
    ) {
        Ok(client) => client,
        Err(message) => {
            report_refusal(
                &format!("prune-server 不通（{}）：{message}", socket_path.display()),
                "python3 prune-server/control check 看卡在哪；起不来看 journalctl --user -u singlefs-prune-server.service -n 100",
            );
            return PRUNE_SERVER_UNAVAILABLE_EXIT_CODE;
        }
    };
    let outcome = match action {
        AdminAction::Health => client.statistics().map(|counters| {
            let frames_handled = counters
                .iter()
                .find(|counter| counter.name == crate::server::FRAMES_HANDLED_COUNTER)
                .map_or(0, |counter| counter.value);
            println!("{HEALTH_LINE_PREFIX} ok frames_handled={frames_handled}");
        }),
        AdminAction::ConnectedClients => client
            .connected_clients()
            .map(|clients| print_clients(&clients, "listed")),
        AdminAction::AnnounceStop => client
            .announce_stop()
            .map(|clients| print_clients(&clients, "stop-announced")),
    };
    match outcome {
        Ok(()) => 0,
        Err(message) => {
            report_refusal(
                &format!("prune-server 回了错：{message}"),
                "看 journalctl --user -u singlefs-prune-server.service -n 100 与状态目录里的 faults.log",
            );
            1
        }
    }
}

struct SelftestClientArguments {
    socket_path: PathBuf,
    journal_path: PathBuf,
    reconnect_total: Duration,
}

fn parse_selftest_client_arguments(
    arguments: &[String],
) -> Result<SelftestClientArguments, String> {
    let mut socket_path = None;
    let mut journal_path = None;
    let mut reconnect_total = None;
    let mut is_selftest_instance = false;
    let mut position = 0;
    while position < arguments.len() {
        let flag = arguments[position].as_str();
        if flag == "--selftest-instance" {
            is_selftest_instance = true;
            position += 1;
            continue;
        }
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{flag} 后面缺值"))?;
        match flag {
            "--socket" => socket_path = Some(PathBuf::from(value)),
            "--journal" => journal_path = Some(PathBuf::from(value)),
            "--reconnect-total-seconds" => {
                reconnect_total = Some(Duration::from_secs(parse_positive_number(flag, value)?));
            }
            other => return Err(format!("认不得的参数「{other}」")),
        }
        position += 2;
    }
    match (
        socket_path,
        journal_path,
        reconnect_total,
        is_selftest_instance,
    ) {
        (Some(socket_path), Some(journal_path), Some(reconnect_total), true) => {
            Ok(SelftestClientArguments {
                socket_path,
                journal_path,
                reconnect_total,
            })
        }
        (_, _, _, _) => Err(
            "缺 --socket、--journal、--reconnect-total-seconds 或 --selftest-instance".to_string(),
        ),
    }
}

/// 自证起的客户端子进程（文件头）。
pub fn run_selftest_client(arguments: &[String]) -> i32 {
    let parsed = match parse_selftest_client_arguments(arguments) {
        Ok(parsed) => parsed,
        Err(message) => {
            report_refusal(&message, SELFTEST_CLIENT_USAGE);
            return 2;
        }
    };
    let break_switches = match BreakSwitches::from_environment() {
        Ok(switches) => switches,
        Err(message) => {
            report_refusal(&message, &format!("改对 {BREAK_VARIABLE} 的值，或不设它"));
            return 2;
        }
    };
    let mut settings = ReliableClientSettings::new(
        RouteAddress::UnixSocket(parsed.socket_path),
        parsed.journal_path,
    );
    settings.announcement = announcement_from_environment(&break_switches);
    settings.frame_timeout = Duration::from_secs(30);
    settings.reconnect = ReconnectPolicy {
        initial_delay: Duration::from_millis(50),
        maximum_delay: Duration::from_secs(1),
        maximum_attempts: 100_000,
        maximum_total: parsed.reconnect_total,
    };
    settings.break_switches = break_switches.clone();
    let task = settings.announcement.task.clone();
    let started = Instant::now();
    let report_failure = |failure: &crate::reliable_client::ClientFailure| -> i32 {
        let exit_code = failure.exit_code_with_switches(&break_switches);
        println!(
            "{SELFTEST_CLIENT_LINE_PREFIX} failed exit_code={exit_code} elapsed_milliseconds={} reason={failure}",
            started.elapsed().as_millis()
        );
        exit_code
    };
    let mut client = match ReliableClient::open(settings) {
        Ok(client) => client,
        Err(failure) => return report_failure(&failure),
    };
    let mut frame_number: u64 = 0;
    loop {
        frame_number += 1;
        let entry = (
            Table::Facts,
            format!("selftest-client/{}/{frame_number}", std::process::id()).into_bytes(),
            frame_number.to_be_bytes().to_vec(),
        );
        if let Err(failure) = client
            .submit_write_entries(std::slice::from_ref(&entry))
            .and_then(|_| client.wait_all_acknowledged())
        {
            return report_failure(&failure);
        }
        if frame_number == 1 {
            println!(
                "{SELFTEST_CLIENT_LINE_PREFIX} connected pid={} task={}",
                std::process::id(),
                if task.is_empty() { "-" } else { task.as_str() }
            );
        }
        std::thread::sleep(SELFTEST_CLIENT_WRITE_INTERVAL);
    }
}
