//! 两个二进制共用的入口。两者认同一组子命令：`serve`（服务）、`bench`（压测）、`--selftest`、`--selftest-break-sweep`；
//! `prune-server-bench` 不带子命令时整串参数当压测参数。压测器与自证起服务都是起自己这个可执行文件的 `serve`，
//! 所以两个二进制谁起服务都是同一份服务代码（这个库的 `server` 模块）。
//! 内部起的子进程带 `--spawned-by-parent`（服务）或 `--selftest-instance`（自证起的服务、压测器与每格的 `selftest-cell` 进程），
//! 不再判准入：起它的一方判过了。

use std::path::PathBuf;
use std::time::Duration;

use crate::administration::{run_admin, run_selftest_client};
use crate::bench::{default_worker_threads, run_bench, BENCH_USAGE};
use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::client::{RouteAddress, SimpleClient};
use crate::committer::CommitterSettings;
use crate::configuration::{
    default_multi_host_configuration_path, default_socket_path, default_state_directory,
    library_directory_in, read_configuration_value, NETWORK_ADDRESS_KEY,
};
use crate::oversized_values::run_drop_oversized_values;
use crate::preflight::preflight;
use crate::protocol::{ClaimOutcome, Table};
use crate::selftest::{run_break_sweep, run_selftest, run_selftest_cell};
use crate::server::{serve, AdmissionLimits, SelftestDelay, ServerSettings};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProgramName {
    Server,
    Bench,
}

impl ProgramName {
    fn source_file(self) -> &'static str {
        match self {
            ProgramName::Server => "src/bin/prune_server.rs",
            ProgramName::Bench => "src/bin/prune_server_bench.rs",
        }
    }
}

pub const SERVE_USAGE: &str = "用法：prune-server serve [--socket <路径>] [--state-directory <目录>] [--library <目录>，只给自证与压测指临时库] [--tcp <地址:端口>] \
[--multi-host-configuration <文件>] [--worker-threads <数>] [--group-commit-milliseconds <数，缺省 2>] \
[--group-commit-frames <数，缺省 256>] [--maximum-pending-per-connection <数，缺省 512>] [--maximum-pending-total <数，缺省 8192>] \
[--maximum-pending-bytes <数，缺省 1073741824>] [--maximum-pending-bytes-per-connection <数，缺省 268435456>] \
[--maximum-connections <数，缺省 1024>] [--frame-read-timeout-milliseconds <数，缺省 60000>] [--write-timeout-milliseconds <数，缺省 60000>] \
[--busy-retry-milliseconds <数，缺省 20>]\n      prune-server admin health|connected-clients|announce-stop [--socket <路径>]\n      prune-server drop-oversized-values [--state-directory <目录>] [--dry-run]（服务停着时跑）\n      prune-server --selftest | --selftest-break-sweep\n      prune-server bench <压测参数，见 prune-server-bench>";

/// 退出码：TCP 不能用——多机配置里没有 TCP 地址（`ServeFailure::NetworkUnavailable` 同一个码）。
const NETWORK_UNAVAILABLE_EXIT_CODE: i32 = 5;
/// 组提交窗口与每组帧数的缺省：窗口短到一个人写时多等的不到一次同步的用时，帧数够把压测里一秒的小块帧合进几组。
pub const DEFAULT_GROUP_COMMIT_MILLISECONDS: u64 = 2;
pub const DEFAULT_GROUP_COMMIT_FRAMES: usize = 256;
/// 待处理上限的缺省：每条连接 512 帧（压测器窗口缺省 256 帧，平常跑不会撞上），整个服务 8192 帧、1 GiB。
pub const DEFAULT_MAXIMUM_PENDING_PER_CONNECTION: usize = 512;
pub const DEFAULT_MAXIMUM_PENDING_TOTAL: usize = 8192;
pub const DEFAULT_MAXIMUM_PENDING_BYTES: usize = 1 << 30;
/// 每条连接记着的字节（收着的帧体、待处理的帧、没写出去的回应）的缺省上限：整个服务那一份的四分之一，
/// 一个只发不收的客户端占不满服务的额度，别的客户端照常读写（`server.rs` 文件头「额度」）。
pub const DEFAULT_MAXIMUM_PENDING_BYTES_PER_CONNECTION: usize = 1 << 28;
/// 同时连着的连接数缺省上限：两台的任务各自一个进程一条连接，几百个并发进程留出余量。
pub const DEFAULT_MAXIMUM_CONNECTIONS: usize = 1024;
/// 一帧从头一个字节起收齐的缺省期限：满块一帧约 9 MB，直连段上一秒以内收完，留足余量。
pub const DEFAULT_FRAME_READ_TIMEOUT_MILLISECONDS: u64 = 60_000;
/// 写回应一次堵住的缺省期限：比容错客户端的写超时（30 秒）长，正常客户端收得过来。
pub const DEFAULT_WRITE_TIMEOUT_MILLISECONDS: u64 = 60_000;
pub const DEFAULT_BUSY_RETRY_MILLISECONDS: u64 = 20;
/// 认领持有者打的一行的开头（自证的认领那一格认它拿身份）。
pub const CLAIM_HOLDER_LINE_PREFIX: &str = "PRUNE_SERVER_CLAIM_HOLDER";

pub fn run(program: ProgramName) -> i32 {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let is_internal = arguments
        .iter()
        .any(|argument| argument == "--spawned-by-parent" || argument == "--selftest-instance");
    if !is_internal {
        if let Err(exit_code) = preflight(program.source_file(), &arguments) {
            return exit_code;
        }
    }
    let arguments: Vec<String> = arguments
        .into_iter()
        .filter(|argument| argument != "--force")
        .collect();
    match arguments.first().map(String::as_str) {
        Some("serve") => run_server(&arguments[1..]),
        Some("bench") => run_bench(&arguments[1..]),
        Some("claim-holder") => run_claim_holder(&arguments[1..]),
        Some("admin") => run_admin(&arguments[1..]),
        Some("drop-oversized-values") => run_drop_oversized_values(&arguments[1..]),
        Some("selftest-client") => run_selftest_client(&arguments[1..]),
        Some("selftest-cell") => run_selftest_cell(&arguments[1..]),
        Some("--selftest") => run_selftest(),
        Some("--selftest-break-sweep") => run_break_sweep(),
        Some("--help" | "-h") => {
            println!("{SERVE_USAGE}");
            println!("{BENCH_USAGE}");
            0
        }
        Some(_) | None => match program {
            ProgramName::Bench => run_bench(&arguments),
            ProgramName::Server => {
                eprintln!("  ✗ 缺子命令或子命令认不得：{arguments:?}");
                eprintln!("  → 怎么办：{SERVE_USAGE}");
                2
            }
        },
    }
}

struct ServeArguments {
    socket_path: Option<PathBuf>,
    state_directory: Option<PathBuf>,
    library_directory: Option<PathBuf>,
    network_address: Option<String>,
    selftest_storage_failure_key: Option<Vec<u8>>,
    configuration_path: Option<PathBuf>,
    worker_threads: usize,
    is_selftest_instance: bool,
    selftest_delay: Option<SelftestDelay>,
    selftest_commit_delay: Option<Duration>,
    group_commit_milliseconds: u64,
    group_commit_frames: usize,
    maximum_pending_per_connection: usize,
    maximum_pending_total: usize,
    maximum_pending_bytes: usize,
    maximum_pending_bytes_per_connection: usize,
    maximum_connections: usize,
    frame_read_timeout_milliseconds: u64,
    write_timeout_milliseconds: u64,
    busy_retry_milliseconds: u64,
    selftest_sync_failure_file: Option<PathBuf>,
}

pub(crate) fn parse_positive_number<Number: std::str::FromStr + PartialOrd + Default>(
    flag: &str,
    value: &str,
) -> Result<Number, String> {
    value
        .parse::<Number>()
        .ok()
        .filter(|number| *number > Number::default())
        .ok_or_else(|| format!("{flag} 要正整数：{value}"))
}

fn parse_selftest_delay(text: &str) -> Result<SelftestDelay, String> {
    let (sequence, milliseconds) = text
        .split_once(':')
        .ok_or_else(|| format!("--selftest-delay 要 <序号>:<毫秒>，收到「{text}」"))?;
    let sequence = sequence
        .parse()
        .map_err(|_| format!("--selftest-delay 的序号不是整数：{sequence}"))?;
    let milliseconds = milliseconds
        .parse()
        .map_err(|_| format!("--selftest-delay 的毫秒不是整数：{milliseconds}"))?;
    Ok(SelftestDelay {
        sequence,
        delay: Duration::from_millis(milliseconds),
    })
}

fn parse_serve_arguments(arguments: &[String]) -> Result<ServeArguments, String> {
    let mut parsed = ServeArguments {
        socket_path: None,
        state_directory: None,
        library_directory: None,
        network_address: None,
        selftest_storage_failure_key: None,
        configuration_path: None,
        worker_threads: default_worker_threads(),
        is_selftest_instance: false,
        selftest_delay: None,
        selftest_commit_delay: None,
        group_commit_milliseconds: DEFAULT_GROUP_COMMIT_MILLISECONDS,
        group_commit_frames: DEFAULT_GROUP_COMMIT_FRAMES,
        maximum_pending_per_connection: DEFAULT_MAXIMUM_PENDING_PER_CONNECTION,
        maximum_pending_total: DEFAULT_MAXIMUM_PENDING_TOTAL,
        maximum_pending_bytes: DEFAULT_MAXIMUM_PENDING_BYTES,
        maximum_pending_bytes_per_connection: DEFAULT_MAXIMUM_PENDING_BYTES_PER_CONNECTION,
        maximum_connections: DEFAULT_MAXIMUM_CONNECTIONS,
        frame_read_timeout_milliseconds: DEFAULT_FRAME_READ_TIMEOUT_MILLISECONDS,
        write_timeout_milliseconds: DEFAULT_WRITE_TIMEOUT_MILLISECONDS,
        busy_retry_milliseconds: DEFAULT_BUSY_RETRY_MILLISECONDS,
        selftest_sync_failure_file: None,
    };
    let mut position = 0;
    while position < arguments.len() {
        let flag = arguments[position].as_str();
        match flag {
            "--spawned-by-parent" => {
                position += 1;
                continue;
            }
            "--selftest-instance" => {
                parsed.is_selftest_instance = true;
                position += 1;
                continue;
            }
            _ => {}
        }
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{flag} 后面缺值"))?;
        match flag {
            "--socket" => parsed.socket_path = Some(PathBuf::from(value)),
            "--state-directory" => parsed.state_directory = Some(PathBuf::from(value)),
            "--library" => parsed.library_directory = Some(PathBuf::from(value)),
            "--tcp" => parsed.network_address = Some(value.clone()),
            "--selftest-storage-failure-key" => {
                parsed.selftest_storage_failure_key = Some(value.as_bytes().to_vec());
            }
            "--multi-host-configuration" => parsed.configuration_path = Some(PathBuf::from(value)),
            "--worker-threads" => parsed.worker_threads = parse_positive_number(flag, value)?,
            "--group-commit-milliseconds" => {
                parsed.group_commit_milliseconds = parse_positive_number(flag, value)?;
            }
            "--group-commit-frames" => {
                parsed.group_commit_frames = parse_positive_number(flag, value)?
            }
            "--maximum-pending-per-connection" => {
                parsed.maximum_pending_per_connection = parse_positive_number(flag, value)?;
            }
            "--maximum-pending-total" => {
                parsed.maximum_pending_total = parse_positive_number(flag, value)?
            }
            "--maximum-pending-bytes" => {
                parsed.maximum_pending_bytes = parse_positive_number(flag, value)?
            }
            "--maximum-pending-bytes-per-connection" => {
                parsed.maximum_pending_bytes_per_connection = parse_positive_number(flag, value)?;
            }
            "--maximum-connections" => {
                parsed.maximum_connections = parse_positive_number(flag, value)?;
            }
            "--frame-read-timeout-milliseconds" => {
                parsed.frame_read_timeout_milliseconds = parse_positive_number(flag, value)?;
            }
            "--write-timeout-milliseconds" => {
                parsed.write_timeout_milliseconds = parse_positive_number(flag, value)?;
            }
            "--busy-retry-milliseconds" => {
                parsed.busy_retry_milliseconds = parse_positive_number(flag, value)?;
            }
            "--selftest-sync-failure-file" => {
                parsed.selftest_sync_failure_file = Some(PathBuf::from(value));
            }
            "--selftest-delay" => parsed.selftest_delay = Some(parse_selftest_delay(value)?),
            "--selftest-commit-delay-milliseconds" => {
                parsed.selftest_commit_delay =
                    Some(Duration::from_millis(parse_positive_number(flag, value)?));
            }
            other => return Err(format!("认不得的参数「{other}」")),
        }
        position += 2;
    }
    if (parsed.selftest_delay.is_some()
        || parsed.selftest_commit_delay.is_some()
        || parsed.selftest_storage_failure_key.is_some()
        || parsed.selftest_sync_failure_file.is_some())
        && !parsed.is_selftest_instance
    {
        return Err(
            "--selftest-delay、--selftest-commit-delay-milliseconds、--selftest-storage-failure-key、--selftest-sync-failure-file 只给自证起的服务用，要与 --selftest-instance 一起给"
                .to_string(),
        );
    }
    Ok(parsed)
}

pub(crate) fn report_refusal(message: &str, next_step: &str) {
    eprintln!("  ✗ {message}");
    eprintln!("  → 怎么办：{next_step}");
}

fn run_server(arguments: &[String]) -> i32 {
    let parsed = match parse_serve_arguments(arguments) {
        Ok(parsed) => parsed,
        Err(message) => {
            report_refusal(&message, SERVE_USAGE);
            return 2;
        }
    };
    let break_switches = if parsed.is_selftest_instance {
        match BreakSwitches::from_environment() {
            Ok(switches) => switches,
            Err(message) => {
                report_refusal(&message, "改对 PRUNE_SERVER_BREAK 的值，或不设它");
                return 2;
            }
        }
    } else {
        BreakSwitches::none()
    };
    let network_address = match parsed.network_address {
        Some(address) => address,
        None => {
            let configuration_path = parsed
                .configuration_path
                .unwrap_or_else(default_multi_host_configuration_path);
            match read_configuration_value(&configuration_path, NETWORK_ADDRESS_KEY) {
                Ok(Some(address)) if !address.is_empty() => address,
                Ok(Some(_) | None) | Err(_)
                    if break_switches
                        .is_enabled(BreakSwitch::MissingNetworkAddressDefaultsToLoopback) =>
                {
                    "127.0.0.1:0".to_string()
                }
                Ok(Some(_) | None) => {
                    report_refusal(
                        &format!("多机配置 {} 里没有 {NETWORK_ADDRESS_KEY}（或是空串）", configuration_path.display()),
                        &format!("在仓根 multi-host.env 加一行 {NETWORK_ADDRESS_KEY}=<本机在有线直连段上的地址>:<端口>，或命令行给 --tcp <地址:端口>"),
                    );
                    return NETWORK_UNAVAILABLE_EXIT_CODE;
                }
                Err(message) => {
                    report_refusal(&message, "照仓根 multi-host.env.example 建 multi-host.env，或设 SINGLEFS_MULTI_HOST_CONFIG 指到它，或命令行给 --tcp <地址:端口>");
                    return NETWORK_UNAVAILABLE_EXIT_CODE;
                }
            }
        }
    };
    let socket_path = match parsed.socket_path.map_or_else(default_socket_path, Ok) {
        Ok(path) => path,
        Err(message) => {
            report_refusal(&message, "设 XDG_RUNTIME_DIR，或命令行给 --socket <路径>");
            return 4;
        }
    };
    let state_directory = match parsed
        .state_directory
        .map_or_else(default_state_directory, Ok)
    {
        Ok(path) => path,
        Err(message) => {
            report_refusal(&message, "设 HOME，或命令行给 --state-directory <目录>");
            return 3;
        }
    };
    let library_directory = parsed
        .library_directory
        .unwrap_or_else(|| library_directory_in(&state_directory));
    let settings = ServerSettings {
        socket_path,
        network_address,
        state_directory,
        library_directory,
        worker_threads: parsed.worker_threads,
        break_switches,
        selftest_delay: parsed.selftest_delay,
        selftest_sync_failure_file: parsed.selftest_sync_failure_file,
        admission: AdmissionLimits {
            maximum_pending_per_connection: parsed.maximum_pending_per_connection,
            maximum_pending_total: parsed.maximum_pending_total,
            maximum_pending_bytes: parsed.maximum_pending_bytes,
            maximum_pending_bytes_per_connection: parsed.maximum_pending_bytes_per_connection,
            maximum_connections: parsed.maximum_connections,
            frame_read_timeout: Duration::from_millis(parsed.frame_read_timeout_milliseconds),
            write_timeout: Duration::from_millis(parsed.write_timeout_milliseconds),
            busy_retry: Duration::from_millis(parsed.busy_retry_milliseconds),
        },
        committer: CommitterSettings {
            group_window: Duration::from_millis(parsed.group_commit_milliseconds),
            group_maximum_frames: parsed.group_commit_frames,
            selftest_commit_delay: parsed.selftest_commit_delay,
            selftest_storage_failure_key: parsed.selftest_storage_failure_key,
        },
    };
    match serve(settings) {
        Ok(()) => 0,
        Err(failure) => {
            let (message, next_step) = failure.describe();
            report_refusal(&message, next_step);
            failure.exit_code()
        }
    }
}

const CLAIM_HOLDER_USAGE: &str = "用法（只给自证）：prune-server claim-holder --selftest-instance --socket <路径> --lease-milliseconds <数> --keys <键,…>";

/// 自证的认领那一格起的子进程：向服务认领一批结论表的键，打一行身份与结局，然后一直睡，等自证 `kill -9` 它。
fn run_claim_holder(arguments: &[String]) -> i32 {
    let mut socket_path = None;
    let mut lease_milliseconds = None;
    let mut keys: Vec<(Table, Vec<u8>)> = Vec::new();
    let mut is_selftest_instance = false;
    let mut position = 0;
    while position < arguments.len() {
        let flag = arguments[position].as_str();
        if flag == "--selftest-instance" {
            is_selftest_instance = true;
            position += 1;
            continue;
        }
        let Some(value) = arguments.get(position + 1) else {
            report_refusal(&format!("{flag} 后面缺值"), CLAIM_HOLDER_USAGE);
            return 2;
        };
        match flag {
            "--socket" => socket_path = Some(PathBuf::from(value)),
            "--lease-milliseconds" => match parse_positive_number::<u32>(flag, value) {
                Ok(lease) => lease_milliseconds = Some(lease),
                Err(message) => {
                    report_refusal(&message, CLAIM_HOLDER_USAGE);
                    return 2;
                }
            },
            "--keys" => {
                keys = value
                    .split(',')
                    .filter(|key| !key.is_empty())
                    .map(|key| (Table::Conclusions, key.as_bytes().to_vec()))
                    .collect();
            }
            other => {
                report_refusal(&format!("认不得的参数「{other}」"), CLAIM_HOLDER_USAGE);
                return 2;
            }
        }
        position += 2;
    }
    let (Some(socket_path), Some(lease_milliseconds), true, false) = (
        socket_path,
        lease_milliseconds,
        is_selftest_instance,
        keys.is_empty(),
    ) else {
        report_refusal(
            "缺 --socket、--lease-milliseconds、--keys 或 --selftest-instance",
            CLAIM_HOLDER_USAGE,
        );
        return 2;
    };
    let outcome = SimpleClient::connect(
        &RouteAddress::UnixSocket(socket_path),
        BreakSwitches::none(),
    )
    .and_then(|mut client| {
        let outcomes = client.claim(&keys, lease_milliseconds)?;
        Ok((client, outcomes))
    });
    let (client, outcomes) = match outcome {
        Ok(pair) => pair,
        Err(message) => {
            report_refusal(&message, "看服务起来没有、socket 路径对不对");
            return 3;
        }
    };
    let claimed = outcomes
        .iter()
        .filter(|claim| matches!(claim, ClaimOutcome::Claimed))
        .count();
    println!(
        "{CLAIM_HOLDER_LINE_PREFIX} identity={} claimed={claimed} keys={}",
        String::from_utf8_lossy(&client.client_identity),
        keys.len()
    );
    if std::io::Write::flush(&mut std::io::stdout()).is_err() {
        return 3;
    }
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
