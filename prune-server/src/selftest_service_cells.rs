//! 自证里服务转正那一批的格（格表在 `selftest.rs` 的 `CELLS`）：常驻单元与控制命令、停服务列客户端、停服务时客户端快失败、
//! 服务没打招呼就断了时客户端有限时退出、故障行、监听三格、写超时、经 TCP 代理杀服务。
//!
//! 起的子进程（服务、控制命令、自证客户端）的弄坏开关都由这一格自己定：环境变量 `PRUNE_SERVER_BREAK` 只交这一格要证红的那几项
//! （`BreakSwitches::only_these`），别的格的开关不碰这一格。控制命令跑在 `PRUNE_SERVER_CONTROL_SELFTEST_ROOT` 指的临时目录里，
//! systemctl 换成那里的假脚本（`FAKE_SYSTEMCTL`），服务是真的、挂在那个目录里的 socket 上。

use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::administration::{CLIENT_LINE_PREFIX, SELFTEST_CLIENT_LINE_PREFIX};
use crate::bench::SERVER_READY_TIMEOUT;
use crate::break_switches::{BreakSwitch, BreakSwitches, BREAK_VARIABLE};
use crate::client::{RouteAddress, SimpleClient};
use crate::connections::{TASK_ATTEMPT_VARIABLE, TASK_NUMBER_VARIABLE};
use crate::fault_log::{timestamp_text, FAULT_LINE_PREFIX, FAULT_LOG_FILE_NAME};
use crate::fault_proxy::{FaultProxy, ProxyListen};
use crate::protocol::{ClientAnnouncement, Table};
use crate::reliable_client::{
    ClientFailure, ReconnectPolicy, ReliableClient, ReliableClientSettings,
    PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
};
use crate::selftest::{line_field, CellContext, CellDirectory};
use crate::selftest_fault_cells::within_deadline;
use crate::server::LISTENING_LINE_PREFIX;
use crate::spawned::{SpawnedProcess, StandardError};

/// 控制命令跑在哪个临时目录里（`prune-server/control` 文件头）。
const CONTROL_SELFTEST_ROOT_VARIABLE: &str = "PRUNE_SERVER_CONTROL_SELFTEST_ROOT";
/// 一条控制命令（watch 之外）跑完的上限：回读的限时在自证里是 3 秒，这里只防卡死。
const CONTROL_RUN_TIMEOUT: Duration = Duration::from_secs(120);
/// 等自证客户端连上、头一帧确认的上限。
const CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(60);
/// watch 起来之后先看它这么久不退（这期间什么都没出），再造情形。
const WATCH_QUIET_PERIOD: Duration = Duration::from_secs(2);
/// 造了情形之后 watch 退出的上限（它在自证里每 0.3 秒查一次）。
const WATCH_EXIT_TIMEOUT: Duration = Duration::from_secs(30);

/// 假 systemctl：记下每次调用（`fake/calls.log`），按 `fake/` 下几个文件回状态；单元的 cgroup 是 `cgroup/fake.slice`。
const FAKE_SYSTEMCTL: &str = r#"#!/usr/bin/env bash
# 自证用的假 systemctl（prune-server 的控制命令那几格）：记下每次调用，按 fake/ 下的文件回状态。
root="$(cd "$(dirname "$0")" && pwd)"
printf '%s\n' "$*" >> "$root/fake/calls.log"
[ "$1" = "--user" ] && shift
processes="$root/cgroup/fake.slice/cgroup.procs"
case "$1" in
  daemon-reload) exit 0 ;;
  enable) echo active > "$root/fake/state"; echo 4242 > "$processes"; exit 0 ;;
  start)
    if [ -f "$root/fake/start-result" ]; then cat "$root/fake/start-result" > "$root/fake/state"; else echo active > "$root/fake/state"; fi
    echo 4242 > "$processes"; exit 0 ;;
  stop)
    echo inactive > "$root/fake/state"
    if [ -f "$root/fake/stop-leaves-process" ]; then echo 4242 > "$processes"; else : > "$processes"; fi
    exit 0 ;;
  is-active)
    state="$(cat "$root/fake/state" 2>/dev/null || echo inactive)"
    echo "$state"
    [ "$state" = active ] && exit 0
    exit 3 ;;
  show) echo /fake.slice; exit 0 ;;
  *) echo "假 systemctl 认不得：$*" >&2; exit 1 ;;
esac
"#;

fn make_private_directory(path: &Path) -> Result<(), String> {
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(|error| format!("建不了目录 {}：{error}", path.display()))
}

/// 一格里起的真服务（路径由这一格定，弄坏开关只交这一格要的）。
struct SandboxServer {
    process: SpawnedProcess,
    unix_socket: PathBuf,
    network_address: String,
    fault_log: PathBuf,
}

impl SandboxServer {
    fn spawn(
        context: &CellContext,
        socket: &Path,
        state_directory: &Path,
        switches: &BreakSwitches,
        extra: &[&str],
    ) -> Result<SandboxServer, String> {
        let mut command = Command::new(&context.executable);
        command
            .arg("serve")
            .arg("--selftest-instance")
            .arg("--socket")
            .arg(socket)
            .arg("--state-directory")
            .arg(state_directory)
            .arg("--library")
            .arg(state_directory.join("library"))
            .arg("--tcp")
            .arg("127.0.0.1:0")
            .arg("--worker-threads")
            .arg("2")
            .args(extra)
            .env(BREAK_VARIABLE, switches.environment_text());
        let mut process = SpawnedProcess::spawn(command, StandardError::Capture)?;
        let line = process
            .wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)
            .map_err(|message| {
                format!(
                    "服务没起来：{message}；标准错误：{}",
                    process.standard_error_text()
                )
            })?;
        let network_address =
            line_field(&line, "tcp").ok_or_else(|| format!("就绪行没有 tcp=：{line}"))?;
        Ok(SandboxServer {
            process,
            unix_socket: socket.to_path_buf(),
            network_address,
            fault_log: state_directory.join(FAULT_LOG_FILE_NAME),
        })
    }

    /// 起在一格自己的目录里（socket、状态目录都在 `directory` 下）。
    fn spawn_in(
        context: &CellContext,
        directory: &Path,
        switches: &BreakSwitches,
        extra: &[&str],
    ) -> Result<SandboxServer, String> {
        SandboxServer::spawn(
            context,
            &directory.join("server.sock"),
            directory,
            switches,
            extra,
        )
    }

    fn unix_route(&self) -> RouteAddress {
        RouteAddress::UnixSocket(self.unix_socket.clone())
    }

    fn network_route(&self) -> RouteAddress {
        RouteAddress::Network(self.network_address.clone())
    }

    /// `kill -9` 这一份服务，等它退出。
    fn kill(&mut self) -> Result<(), String> {
        self.process.send_signal(libc::SIGKILL)?;
        self.process
            .wait_for_exit(SERVER_READY_TIMEOUT)
            .ok_or("服务收了 SIGKILL 还没退出")?;
        Ok(())
    }

    fn fault_lines(&self) -> Vec<String> {
        std::fs::read_to_string(&self.fault_log)
            .unwrap_or_default()
            .lines()
            .filter(|line| line.starts_with(FAULT_LINE_PREFIX))
            .map(str::to_string)
            .collect()
    }
}

/// 一条控制命令跑完的样子。
struct ControlRun {
    exit_code: Option<i32>,
    standard_output: String,
    standard_error: String,
}

impl ControlRun {
    fn describe(&self) -> String {
        format!(
            "退出码 {:?}；标准输出：{}；标准错误：{}",
            self.exit_code,
            self.standard_output.trim(),
            self.standard_error.trim()
        )
    }
}

/// 控制命令的临时根目录：路径照 `prune-server/control` 的 `Paths` 自证那一支。
struct ControlSandbox {
    directory: CellDirectory,
}

impl ControlSandbox {
    fn create() -> Result<ControlSandbox, String> {
        let directory = CellDirectory::create()?;
        let root = directory.path.clone();
        for subdirectory in ["fake", "cgroup/fake.slice"] {
            make_private_directory(&root.join(subdirectory))?;
        }
        let systemctl = root.join("fake-systemctl");
        std::fs::write(&systemctl, FAKE_SYSTEMCTL)
            .and_then(|()| {
                std::fs::set_permissions(&systemctl, std::fs::Permissions::from_mode(0o755))
            })
            .map_err(|error| format!("写不了假 systemctl：{error}"))?;
        // 控制命令经共用取法（scripts/hosts/multi_host_addresses.py）判服务在哪台，配置要整份判得过：
        // 127.0.0.1 在每一台的 lo 上（判成这台），192.0.2.1 是文档段地址（哪一台都没有），NFS_SHARED_ROOTS 指自证根（这一台已在的目录）；
        // 按台的三个键照判法（scripts/crash/crash-point-replay-shard-configuration-check.sh 文件头「按台的三个键」）给 HOSTS 里每一台各写一项。
        std::fs::write(
            root.join("multi-host.env"),
            format!(
                "# 自证用的多机配置\nHOSTS=127.0.0.1 192.0.2.1\nPRUNE_SERVER_TCP_ADDRESS=127.0.0.1:0\n\
                 CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.1=/selftest/peer-cargo\n\
                 BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.1=/selftest/peer-builds\n\
                 SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.1=1G\n\
                 QUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\nQUIESCE_STARTED_CHECK_COMMAND=\n\
                 ENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\nNFS_SHARED_ROOTS={}\n",
                root.display()
            ),
        )
        .map_err(|error| format!("写不了自证的多机配置：{error}"))?;
        Ok(ControlSandbox { directory })
    }

    fn root(&self) -> &Path {
        &self.directory.path
    }

    fn socket(&self) -> PathBuf {
        self.root().join("run").join("server.sock")
    }

    fn state_directory(&self) -> PathBuf {
        self.root().join("state")
    }

    fn binary(&self) -> PathBuf {
        self.root().join("bin").join("prune-server")
    }

    fn unit_file(&self) -> PathBuf {
        self.root()
            .join("systemd")
            .join("singlefs-prune-server.service")
    }

    fn fake(&self, name: &str) -> PathBuf {
        self.root().join("fake").join(name)
    }

    fn set_fake(&self, name: &str, content: &str) -> Result<(), String> {
        std::fs::write(self.fake(name), content)
            .map_err(|error| format!("写不了假 systemctl 的 {name}：{error}"))
    }

    fn remove_fake(&self, name: &str) -> Result<(), String> {
        match std::fs::remove_file(self.fake(name)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("删不了假 systemctl 的 {name}：{error}")),
        }
    }

    /// 控制命令要调的二进制：链到自证自己这个可执行文件（install 那一格另由 install --binary 拷）。
    fn link_binary(&self, context: &CellContext) -> Result<(), String> {
        std::fs::create_dir_all(self.root().join("bin"))
            .and_then(|()| std::os::unix::fs::symlink(&context.executable, self.binary()))
            .map_err(|error| format!("链不了二进制：{error}"))
    }

    fn spawn_server(
        &self,
        context: &CellContext,
        switches: &BreakSwitches,
    ) -> Result<SandboxServer, String> {
        make_private_directory(&self.root().join("run"))?;
        make_private_directory(&self.state_directory())?;
        SandboxServer::spawn(
            context,
            &self.socket(),
            &self.state_directory(),
            switches,
            &[],
        )
    }

    fn control_command(&self, arguments: &[&str], switches: &BreakSwitches) -> Command {
        let mut command = Command::new("python3");
        command
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("control"))
            .args(arguments)
            .env(CONTROL_SELFTEST_ROOT_VARIABLE, self.root());
        let switches_text = switches.environment_text();
        if switches_text.is_empty() {
            command.env_remove(BREAK_VARIABLE);
        } else {
            command.env(BREAK_VARIABLE, switches_text);
        }
        command
    }

    fn run_control(
        &self,
        arguments: &[&str],
        switches: &BreakSwitches,
    ) -> Result<ControlRun, String> {
        let mut process = SpawnedProcess::spawn(
            self.control_command(arguments, switches),
            StandardError::Capture,
        )?;
        let status = process.wait_for_exit(CONTROL_RUN_TIMEOUT).ok_or_else(|| {
            format!(
                "control {} {} 秒没跑完",
                arguments.join(" "),
                CONTROL_RUN_TIMEOUT.as_secs()
            )
        })?;
        let standard_output = process.remaining_lines(Duration::from_secs(5)).join("\n");
        Ok(ControlRun {
            exit_code: status.code(),
            standard_output,
            standard_error: process.standard_error_text(),
        })
    }
}

/// 起一个自证客户端子进程（`administration` 文件头），等它头一帧确认；`task` 是（任务号，第几次），`None` 是不在 task-server 里跑。
fn spawn_selftest_client(
    context: &CellContext,
    socket: &Path,
    journal: &Path,
    reconnect_total_seconds: u64,
    task: Option<(&str, &str)>,
    switches: &BreakSwitches,
) -> Result<SpawnedProcess, String> {
    let mut command = Command::new(&context.executable);
    command
        .arg("selftest-client")
        .arg("--selftest-instance")
        .arg("--socket")
        .arg(socket)
        .arg("--journal")
        .arg(journal)
        .arg("--reconnect-total-seconds")
        .arg(reconnect_total_seconds.to_string())
        .env(BREAK_VARIABLE, switches.environment_text());
    match task {
        Some((number, attempt)) => {
            command
                .env(TASK_NUMBER_VARIABLE, number)
                .env(TASK_ATTEMPT_VARIABLE, attempt);
        }
        None => {
            command
                .env_remove(TASK_NUMBER_VARIABLE)
                .env_remove(TASK_ATTEMPT_VARIABLE);
        }
    }
    let mut process = SpawnedProcess::spawn(command, StandardError::Capture)?;
    process
        .wait_for_line_starting_with(
            &format!("{SELFTEST_CLIENT_LINE_PREFIX} connected"),
            CLIENT_CONNECT_TIMEOUT,
        )
        .map_err(|message| {
            format!(
                "自证客户端没连上：{message}；标准错误：{}",
                process.standard_error_text()
            )
        })?;
    Ok(process)
}

/// 等自证客户端退出：回（退出码，它打的 failed 行，从调用起等了多久）。
fn wait_client_exit(
    client: &mut SpawnedProcess,
    limit: Duration,
) -> Result<(Option<i32>, String, Duration), String> {
    let started = Instant::now();
    let status = client
        .wait_for_exit(limit)
        .ok_or_else(|| format!("客户端 {} 秒还没退出（还在重试或挂着）", limit.as_secs()))?;
    let waited = started.elapsed();
    let failed_line = client
        .remaining_lines(Duration::from_secs(5))
        .into_iter()
        .find(|line| line.starts_with(&format!("{SELFTEST_CLIENT_LINE_PREFIX} failed")))
        .unwrap_or_default();
    Ok((status.code(), failed_line, waited))
}

fn expect_exit(run: &ControlRun, expected: i32, what: &str) -> Result<(), String> {
    if run.exit_code != Some(expected) {
        return Err(format!("{what}：该退 {expected}，{}", run.describe()));
    }
    Ok(())
}

pub(crate) fn cell_control_install_start_stop_check(
    context: &CellContext,
) -> Result<String, String> {
    let switches = context.break_switches.only_these(&[
        BreakSwitch::ControlDryRunWrites,
        BreakSwitch::ControlInstallReadbackSkipped,
        BreakSwitch::ControlStartReadbackSkipped,
        BreakSwitch::ControlStopReadbackSkipped,
        BreakSwitch::ControlCheckOmitsLibrary,
    ]);
    let sandbox = ControlSandbox::create()?;
    let executable = context.executable.display().to_string();
    let dry_run = sandbox.run_control(
        &["install", "--dry-run", "--binary", &executable],
        &switches,
    )?;
    expect_exit(&dry_run, 0, "install --dry-run")?;
    let written: Vec<String> = [
        sandbox.unit_file(),
        sandbox.state_directory(),
        sandbox.binary(),
        sandbox.fake("calls.log"),
    ]
    .iter()
    .filter(|path| path.exists())
    .map(|path| path.display().to_string())
    .collect();
    if !written.is_empty() || !dry_run.standard_output.contains("ExecStart=") {
        return Err(format!(
            "install --dry-run 该一个文件不写、systemctl 一条不起、打出单元全文；写出来的：{written:?}；{}",
            dry_run.describe()
        ));
    }
    // 服务还没起：装完回读不通，要退 1，不许说装好了。
    let unreachable_install =
        sandbox.run_control(&["install", "--binary", &executable], &switches)?;
    expect_exit(
        &unreachable_install,
        1,
        "服务不通时 install（回读判出不通）",
    )?;
    let calls = std::fs::read_to_string(sandbox.fake("calls.log")).unwrap_or_default();
    if !calls.contains("daemon-reload")
        || !calls.lines().any(|line| line.starts_with("--user start "))
        || calls.lines().any(|line| line.starts_with("--user enable "))
        || !sandbox.unit_file().exists()
    {
        return Err(format!(
            "install 该写单元、daemon-reload、start，不 enable（开机不自起）；单元在不在 {}，systemctl 的调用：{calls}",
            sandbox.unit_file().exists()
        ));
    }
    let unit_text = std::fs::read_to_string(sandbox.unit_file()).unwrap_or_default();
    if unit_text.lines().any(|line| line == "[Install]") {
        return Err(format!("单元里不该有 [Install]（开机不自起）：{unit_text}"));
    }
    if !unit_text.lines().any(|line| line == "Restart=always") {
        return Err(format!(
            "单元该带 Restart=always（不是 systemctl stop 停掉的由 systemd 拉起）：{unit_text}"
        ));
    }
    let _server = sandbox.spawn_server(context, &BreakSwitches::none())?;
    let reachable_install =
        sandbox.run_control(&["install", "--binary", &executable], &switches)?;
    expect_exit(&reachable_install, 0, "服务通时 install")?;
    let installed_unit = std::fs::read_to_string(sandbox.unit_file()).unwrap_or_default();
    if installed_unit.contains("--library") {
        return Err(format!(
            "单元的 ExecStart 不该带 --library（服务只开状态目录下自己那一份库）：{installed_unit}"
        ));
    }
    sandbox.set_fake("start-result", "failed\n")?;
    let failed_start = sandbox.run_control(&["start"], &switches)?;
    expect_exit(
        &failed_start,
        1,
        "systemctl start 退 0 而 is-active 打 failed 时 start（回读判出没起来）",
    )?;
    sandbox.remove_fake("start-result")?;
    let start = sandbox.run_control(&["start"], &switches)?;
    expect_exit(&start, 0, "start")?;
    let check = sandbox.run_control(&["check"], &switches)?;
    expect_exit(&check, 0, "check")?;
    let expected_library_line = format!(
        "库目录 {}",
        sandbox.state_directory().join("library").display()
    );
    if !check.standard_output.contains(&expected_library_line) {
        return Err(format!(
            "check 该打库目录「{expected_library_line}」；{}",
            check.describe()
        ));
    }
    sandbox.set_fake("stop-leaves-process", "")?;
    let leftover_stop = sandbox.run_control(&["stop"], &switches)?;
    expect_exit(
        &leftover_stop,
        1,
        "systemctl stop 退 0 而 cgroup.procs 里还有进程时 stop（回读判出没停干净）",
    )?;
    sandbox.remove_fake("stop-leaves-process")?;
    let stop = sandbox.run_control(&["stop"], &switches)?;
    expect_exit(&stop, 0, "stop")?;
    Ok("install --dry-run 不写不起；服务不通时 install 退 1、通时退 0、单元 ExecStart 不带 --library；\
        start 回读 is-active 打 failed 退 1、正常退 0；check 退 0、打出状态目录下的库目录；\
        stop 回读 cgroup.procs 有进程退 1、正常退 0"
        .to_string())
}

pub(crate) fn cell_stop_lists_connected_clients(context: &CellContext) -> Result<String, String> {
    let sandbox = ControlSandbox::create()?;
    sandbox.link_binary(context)?;
    let _server = sandbox.spawn_server(
        context,
        &context
            .break_switches
            .only(BreakSwitch::AnnouncementIgnored),
    )?;
    let client_switches = context
        .break_switches
        .only(BreakSwitch::ClientAnnouncementOmitsTask);
    let first = spawn_selftest_client(
        context,
        &sandbox.socket(),
        &sandbox.root().join("first.journal"),
        30,
        Some(("101", "1")),
        &client_switches,
    )?;
    let second = spawn_selftest_client(
        context,
        &sandbox.socket(),
        &sandbox.root().join("second.journal"),
        30,
        Some(("202", "2")),
        &client_switches,
    )?;
    let stop = sandbox.run_control(
        &["stop"],
        &context
            .break_switches
            .only(BreakSwitch::ControlStopRecordSkipped),
    )?;
    expect_exit(&stop, 0, "stop")?;
    let listed: Vec<String> = stop
        .standard_output
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with(CLIENT_LINE_PREFIX))
        .map(str::to_string)
        .collect();
    let expected = [
        ("101", "1", first.process_identifier()),
        ("202", "2", second.process_identifier()),
    ];
    if listed.len() != expected.len() {
        return Err(format!(
            "stop 该列出 2 个客户端，列出：{listed:?}；{}",
            stop.describe()
        ));
    }
    for (task, attempt, process_identifier) in expected {
        let found = listed.iter().any(|line| {
            line_field(line, "task").as_deref() == Some(task)
                && line_field(line, "attempt").as_deref() == Some(attempt)
                && line_field(line, "pid") == Some(process_identifier.to_string())
                && line_field(line, "host").is_some_and(|host| host != "-")
                && line_field(line, "transport").as_deref() == Some("unix")
        });
        if !found {
            return Err(format!(
                "清单里没有任务 {task} 第 {attempt} 次、进程 {process_identifier}、主机名不空的那一个：{listed:?}"
            ));
        }
    }
    let records: Vec<PathBuf> = std::fs::read_dir(sandbox.state_directory().join("stops"))
        .map_err(|error| format!("停服务记录的目录读不了：{error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    let [record] = records.as_slice() else {
        return Err(format!("停服务记录该是 1 份，有：{records:?}"));
    };
    let record_text =
        std::fs::read_to_string(record).map_err(|error| format!("读不了停服务记录：{error}"))?;
    if !listed
        .iter()
        .all(|line| record_text.contains(line.as_str()))
    {
        return Err(format!("停服务记录里没写全清单：{record_text}"));
    }
    Ok(format!(
        "两个带不同任务号的客户端连着时 stop：清单 2 行，任务号、第几次、进程号、主机名都对，停服务记录 {} 里同样两行",
        record
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
    ))
}

/// 有意停：宣布停服务之后照 systemd 那样给服务 SIGTERM，客户端要在这个限时里退 75。
const INTENTIONAL_STOP_LIMIT: Duration = Duration::from_secs(15);

pub(crate) fn cell_intentional_stop_fails_clients_fast(
    context: &CellContext,
) -> Result<String, String> {
    let directory = CellDirectory::create()?;
    let server = SandboxServer::spawn_in(
        context,
        &directory.path,
        &context.break_switches.only(BreakSwitch::StopNoticeNotSent),
        &[],
    )?;
    // 重连总时长给到 60 秒：客户端要是照常重试，限时（15 秒）里退不出来。
    let mut client = spawn_selftest_client(
        context,
        &server.unix_socket,
        &directory.path.join("client.journal"),
        60,
        None,
        &context.break_switches.only(BreakSwitch::StopNoticeRetried),
    )?;
    let announced =
        SimpleClient::connect(&server.unix_route(), BreakSwitches::none())?.announce_stop()?;
    if announced.len() != 1 {
        return Err(format!("宣布停服务时该列出 1 个客户端，列出 {announced:?}"));
    }
    server.process.send_signal(libc::SIGTERM)?;
    let (exit_code, failed_line, waited) = wait_client_exit(&mut client, INTENTIONAL_STOP_LIMIT)?;
    if exit_code != Some(PRUNE_SERVER_UNAVAILABLE_EXIT_CODE) || !failed_line.contains("服务停了")
    {
        return Err(format!(
            "有意停之后客户端该退 {PRUNE_SERVER_UNAVAILABLE_EXIT_CODE} 并说「服务停了」，退了 {exit_code:?}：{failed_line}"
        ));
    }
    Ok(format!(
        "宣布停服务再 SIGTERM：客户端 {:.1} 秒内退 {PRUNE_SERVER_UNAVAILABLE_EXIT_CODE}（限时 {} 秒）、说「服务停了」、不重试",
        waited.as_secs_f64(),
        INTENTIONAL_STOP_LIMIT.as_secs()
    ))
}

/// 没打招呼就断：客户端重连总时长上限。
const UNANNOUNCED_RECONNECT_TOTAL_SECONDS: u64 = 6;
/// 上限之外再给的余量（退避的最后一次、起进程）。
const UNANNOUNCED_EXIT_MARGIN: Duration = Duration::from_secs(20);

pub(crate) fn cell_unannounced_server_death_bounded(
    context: &CellContext,
) -> Result<String, String> {
    let directory = CellDirectory::create()?;
    let mut server =
        SandboxServer::spawn_in(context, &directory.path, &BreakSwitches::none(), &[])?;
    let mut client = spawn_selftest_client(
        context,
        &server.unix_socket,
        &directory.path.join("client.journal"),
        UNANNOUNCED_RECONNECT_TOTAL_SECONDS,
        None,
        &context
            .break_switches
            .only(BreakSwitch::UnreachableExitCodeGeneric),
    )?;
    server.kill()?;
    let limit = Duration::from_secs(UNANNOUNCED_RECONNECT_TOTAL_SECONDS) + UNANNOUNCED_EXIT_MARGIN;
    let (exit_code, failed_line, waited) = wait_client_exit(&mut client, limit)?;
    if exit_code != Some(PRUNE_SERVER_UNAVAILABLE_EXIT_CODE) || !failed_line.contains("连不上") {
        return Err(format!(
            "服务被 kill -9 且不重起，客户端该在重连上限里退 {PRUNE_SERVER_UNAVAILABLE_EXIT_CODE} 并说「连不上」，退了 {exit_code:?}：{failed_line}"
        ));
    }
    Ok(format!(
        "服务 kill -9 不重起：客户端重连 {UNANNOUNCED_RECONNECT_TOTAL_SECONDS} 秒到上限，{:.1} 秒时退 {PRUNE_SERVER_UNAVAILABLE_EXIT_CODE}、说「连不上」",
        waited.as_secs_f64()
    ))
}

/// 故障行那一格造存储写失败用的键。
const STORAGE_FAILURE_KEY: &str = "selftest/storage-failure";

pub(crate) fn cell_fault_line_on_server_error(context: &CellContext) -> Result<String, String> {
    let directory = CellDirectory::create()?;
    let server = SandboxServer::spawn_in(
        context,
        &directory.path,
        &context
            .break_switches
            .only(BreakSwitch::FaultLineNotWritten),
        &["--selftest-storage-failure-key", STORAGE_FAILURE_KEY],
    )?;
    let announcement = ClientAnnouncement::new("selftest-host", "404", "3", 4040);
    let mut client = SimpleClient::connect_announcing(
        &server.unix_route(),
        BreakSwitches::none(),
        &announcement,
    )?;
    let before = server.fault_lines().len();
    let written = client.write(&[(
        Table::Facts,
        STORAGE_FAILURE_KEY.as_bytes().to_vec(),
        b"value".to_vec(),
    )]);
    if !written
        .as_ref()
        .is_err_and(|message| message.contains("StorageFailed"))
    {
        return Err(format!("写到造错的键该回存储出错，回的是 {written:?}"));
    }
    let new_lines: Vec<String> = server.fault_lines().into_iter().skip(before).collect();
    let [line] = new_lines.as_slice() else {
        return Err(format!(
            "存储写失败该在故障日志里新增 1 行，新增：{new_lines:?}"
        ));
    };
    let expected_fields = [
        ("kind", "storage-write-failed"),
        ("transport", "unix"),
        ("host", "selftest-host"),
        ("task", "404"),
        ("attempt", "3"),
        ("pid", "4040"),
    ];
    for (name, value) in expected_fields {
        if line_field(line, name).as_deref() != Some(value) {
            return Err(format!("故障行的 {name}= 该是 {value}：{line}"));
        }
    }
    for name in ["time", "server_pid", "connection", "detail"] {
        if line_field(line, name).is_none_or(|value| value.is_empty()) {
            return Err(format!("故障行缺 {name}=：{line}"));
        }
    }
    client
        .statistics()
        .map_err(|message| format!("存储写失败之后连接不能用了：{message}"))?;
    Ok("存储写失败（自证造的）：故障日志新增 1 行，种类、时刻、服务进程号、连接号、走哪条、主机、任务号、第几次、客户端进程号、说明都齐".to_string())
}

/// 起 watch，看它这一段什么都没出时不退；回它的进程。
fn start_quiet_watch(
    sandbox: &ControlSandbox,
    switches: &BreakSwitches,
) -> Result<SpawnedProcess, String> {
    let mut watch = SpawnedProcess::spawn(
        sandbox.control_command(&["watch"], switches),
        StandardError::Capture,
    )?;
    watch.wait_for_line_starting_with("  · 盯着 prune-server", Duration::from_secs(30))?;
    if let Some(status) = watch.wait_for_exit(WATCH_QUIET_PERIOD) {
        let lines = watch.remaining_lines(Duration::from_secs(2));
        return Err(format!(
            "什么都没出，watch 却在 {} 秒里退了（{:?}）：{}",
            WATCH_QUIET_PERIOD.as_secs(),
            status.code(),
            lines.join(" | ")
        ));
    }
    Ok(watch)
}

/// 等 watch 退出：回（退出码，标准输出全文）。
fn wait_watch_exit(watch: &mut SpawnedProcess) -> Result<(Option<i32>, String), String> {
    let status = watch.wait_for_exit(WATCH_EXIT_TIMEOUT).ok_or_else(|| {
        format!(
            "造了情形之后 watch {} 秒没退：{}",
            WATCH_EXIT_TIMEOUT.as_secs(),
            watch
                .remaining_lines(Duration::from_millis(200))
                .join(" | ")
        )
    })?;
    Ok((
        status.code(),
        watch.remaining_lines(Duration::from_secs(5)).join("\n"),
    ))
}

fn watch_sandbox(context: &CellContext) -> Result<(ControlSandbox, SandboxServer), String> {
    let sandbox = ControlSandbox::create()?;
    sandbox.link_binary(context)?;
    sandbox.set_fake("state", "active\n")?;
    let server = sandbox.spawn_server(context, &BreakSwitches::none())?;
    Ok((sandbox, server))
}

pub(crate) fn cell_watch_reports_unit_not_running(context: &CellContext) -> Result<String, String> {
    let (sandbox, _server) = watch_sandbox(context)?;
    let mut watch = start_quiet_watch(
        &sandbox,
        &context
            .break_switches
            .only(BreakSwitch::WatchIgnoresUnitState),
    )?;
    sandbox.set_fake("state", "inactive\n")?;
    let (exit_code, output) = wait_watch_exit(&mut watch)?;
    if exit_code != Some(1) || !output.contains("不在跑") || !output.contains("此刻连着的客户端")
    {
        return Err(format!(
            "单元不在跑时 watch 该退 1、说不在跑、打连着的客户端，退了 {exit_code:?}：{output}"
        ));
    }
    Ok("单元 is-active 打 inactive：watch 退 1，打出「不在跑」与此刻连着的客户端".to_string())
}

pub(crate) fn cell_watch_reports_server_unreachable(
    context: &CellContext,
) -> Result<String, String> {
    let (sandbox, mut server) = watch_sandbox(context)?;
    let mut watch = start_quiet_watch(
        &sandbox,
        &context.break_switches.only(BreakSwitch::WatchIgnoresHealth),
    )?;
    server.kill()?;
    let (exit_code, output) = wait_watch_exit(&mut watch)?;
    if exit_code != Some(1) || !output.contains("服务不通") || !output.contains("拿不到") {
        return Err(format!(
            "服务不通时 watch 该退 1、说服务不通、客户端清单写拿不到，退了 {exit_code:?}：{output}"
        ));
    }
    Ok(
        "服务被 kill -9（单元还说 active）：watch 退 1，打出「服务不通」，客户端清单写「拿不到」"
            .to_string(),
    )
}

fn append_fault_line(path: &Path, kind: &str, task: &str, detail: &str) -> Result<(), String> {
    let line = format!(
        "{FAULT_LINE_PREFIX} time={} kind={kind} server_pid=1 connection=1 transport=unix peer=- \
         host=selftest-host task={task} attempt=1 pid=1 detail={detail}\n",
        timestamp_text(std::time::UNIX_EPOCH)
    );
    std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .mode(0o600)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()))
        .map_err(|error| format!("往故障日志追加不了：{error}"))
}

pub(crate) fn cell_watch_reports_new_fault_line(context: &CellContext) -> Result<String, String> {
    const OLD_DETAIL: &str = "起watch之前就有的一行";
    const NEW_DETAIL: &str = "自证在watch起之后追加的一行";
    let (sandbox, server) = watch_sandbox(context)?;
    append_fault_line(&server.fault_log, "storage-read-failed", "-", OLD_DETAIL)?;
    let _client = spawn_selftest_client(
        context,
        &sandbox.socket(),
        &sandbox.root().join("client.journal"),
        30,
        Some(("303", "1")),
        &BreakSwitches::none(),
    )?;
    let mut watch = start_quiet_watch(
        &sandbox,
        &context.break_switches.only_these(&[
            BreakSwitch::WatchIgnoresFaultLog,
            BreakSwitch::WatchCountsOldFaultLines,
        ]),
    )?;
    append_fault_line(&server.fault_log, "storage-write-failed", "303", NEW_DETAIL)?;
    let (exit_code, output) = wait_watch_exit(&mut watch)?;
    let expectations = [
        (exit_code == Some(1), "退 1"),
        (
            output.contains(&format!("故障行：{FAULT_LINE_PREFIX}")) && output.contains(NEW_DETAIL),
            "打出新增的故障行原文",
        ),
        (!output.contains(OLD_DETAIL), "起之前就有的那一行不算"),
        (
            output.contains("此刻连着的客户端") && output.contains("task=303"),
            "打出连着的客户端（任务 303 那一个）",
        ),
        (
            output.contains("task-server/task status 303"),
            "打出任务 303 的 task-server/task status 查法",
        ),
    ];
    let missed: Vec<&str> = expectations
        .iter()
        .filter(|(holds, _)| !holds)
        .map(|(_, what)| *what)
        .collect();
    if !missed.is_empty() {
        return Err(format!(
            "故障日志新增一行时 watch 没做到：{}；退出码 {exit_code:?}：{output}",
            missed.join("、")
        ));
    }
    Ok("起之前就有一行故障时 watch 不退；新增一行后退 1，打出故障行原文、连着的客户端与 task-server/task status 303".to_string())
}

/// 写超时那一格的写超时（缺省 30 秒，这里缩短好判）与等它的上限。
const SHORT_WRITE_TIMEOUT: Duration = Duration::from_secs(2);
const WRITE_TIMEOUT_CELL_DEADLINE: Duration = Duration::from_secs(60);

fn big_entry(key: &str, bytes: usize) -> (Table, Vec<u8>, Vec<u8>) {
    (
        Table::Conclusions,
        key.as_bytes().to_vec(),
        vec![0x5A; bytes],
    )
}

pub(crate) fn cell_client_write_timeout_when_peer_stops_reading(
    context: &CellContext,
) -> Result<String, String> {
    const BIG_VALUE_BYTES: usize = 8 * 1024 * 1024;
    let directory = CellDirectory::create()?;
    let server = SandboxServer::spawn_in(context, &directory.path, &BreakSwitches::none(), &[])?;
    let mut proxy = FaultProxy::start(
        ProxyListen::UnixSocket(directory.path.join("proxy.sock")),
        server.unix_route(),
        &BreakSwitches::none(),
    )?;
    let control = proxy.shared_control();
    control
        .stall_reading_connections_remaining
        .store(1, Ordering::SeqCst);
    let stop = Arc::new(AtomicBool::new(false));
    let mut settings =
        ReliableClientSettings::new(proxy.route(), directory.path.join("client.journal"));
    settings.write_timeout = SHORT_WRITE_TIMEOUT;
    settings.frame_timeout = Duration::from_secs(120);
    settings.break_switches = context
        .break_switches
        .only(BreakSwitch::ClientWriteTimeoutUnset);
    let stop_for_client = Arc::clone(&stop);
    settings.stop_check = Arc::new(move || stop_for_client.load(Ordering::SeqCst));
    let started = Instant::now();
    let control_for_timeout = Arc::clone(&control);
    let outcome = within_deadline(
        WRITE_TIMEOUT_CELL_DEADLINE,
        || {
            stop.store(true, Ordering::SeqCst);
            control_for_timeout.cut_all_connections();
        },
        || -> Result<(u64, Option<String>), ClientFailure> {
            let mut client = ReliableClient::open(settings)?;
            client.submit_write_entries(&[big_entry("write-timeout/big", BIG_VALUE_BYTES)])?;
            client.wait_all_acknowledged()?;
            Ok((
                client.diagnostics().reconnections,
                client.diagnostics().last_break_reason.clone(),
            ))
        },
    );
    let elapsed = started.elapsed();
    let stalls = control.stalls_done.load(Ordering::SeqCst);
    proxy.stop();
    let (reconnections, break_reason) = outcome
        .map_err(|message| format!("对端不读时客户端：{message}——堵在写里，写超时没起作用"))?
        .map_err(|failure| format!("客户端出错：{failure}"))?;
    if stalls != 1 || reconnections == 0 {
        return Err(format!(
            "代理停读 {stalls} 条连接、客户端重连 {reconnections} 次：这一格没走到对端不读的局面"
        ));
    }
    let mut reader = SimpleClient::connect(&server.unix_route(), BreakSwitches::none())?;
    let read_back = reader.read(&[(Table::Conclusions, b"write-timeout/big".to_vec())])?;
    if read_back
        .first()
        .cloned()
        .flatten()
        .map(|value| value.len())
        != Some(BIG_VALUE_BYTES)
    {
        return Err("写超时、重连重放之后那一帧没落库".to_string());
    }
    Ok(format!(
        "代理一个字节都不读时，8 MiB 的写帧在写超时 {} 秒处出错（{}）、重连 {reconnections} 次重放落库，共 {:.1} 秒",
        SHORT_WRITE_TIMEOUT.as_secs(),
        break_reason.unwrap_or_default(),
        elapsed.as_secs_f64()
    ))
}

/// 经 TCP 代理杀服务那一格：重连总时长上限、写超时（故意比限时长，判的是代理真关了连接、客户端的写立刻出错）、限时。
const PROXY_KILL_RECONNECT_TOTAL: Duration = Duration::from_secs(8);
const PROXY_KILL_WRITE_TIMEOUT: Duration = Duration::from_secs(90);
const PROXY_KILL_DEADLINE: Duration = Duration::from_secs(40);

pub(crate) fn cell_tcp_proxy_server_kill_client_fails(
    context: &CellContext,
) -> Result<String, String> {
    const BIG_VALUE_BYTES: usize = 8 * 1024 * 1024;
    let directory = CellDirectory::create()?;
    let mut server =
        SandboxServer::spawn_in(context, &directory.path, &BreakSwitches::none(), &[])?;
    let mut proxy = FaultProxy::start(
        ProxyListen::Network("127.0.0.1:0".to_string()),
        server.network_route(),
        &context
            .break_switches
            .only(BreakSwitch::FaultProxyOnlyStopsReadingOnServerDeath),
    )?;
    let control = proxy.shared_control();
    let stop = Arc::new(AtomicBool::new(false));
    let mut settings =
        ReliableClientSettings::new(proxy.route(), directory.path.join("client.journal"));
    settings.write_timeout = PROXY_KILL_WRITE_TIMEOUT;
    settings.frame_timeout = Duration::from_secs(120);
    settings.reconnect = ReconnectPolicy {
        initial_delay: Duration::from_millis(50),
        maximum_delay: Duration::from_secs(1),
        maximum_attempts: 100_000,
        maximum_total: PROXY_KILL_RECONNECT_TOTAL,
    };
    let stop_for_client = Arc::clone(&stop);
    settings.stop_check = Arc::new(move || stop_for_client.load(Ordering::SeqCst));
    let mut client = ReliableClient::open(settings).map_err(|failure| failure.to_string())?;
    client
        .submit_write_entries(&[(
            Table::Conclusions,
            b"proxy-kill/before".to_vec(),
            b"v".to_vec(),
        )])
        .and_then(|_| client.wait_all_acknowledged())
        .map_err(|failure| format!("kill -9 之前那一帧没写成：{failure}"))?;
    server.kill()?;
    let started = Instant::now();
    let control_for_timeout = Arc::clone(&control);
    let outcome = within_deadline(
        PROXY_KILL_DEADLINE,
        || {
            stop.store(true, Ordering::SeqCst);
            control_for_timeout.cut_all_connections();
        },
        || -> Result<(), ClientFailure> {
            for index in 0..4 {
                client.submit_write_entries(&[big_entry(
                    &format!("proxy-kill/after-{index}"),
                    BIG_VALUE_BYTES,
                )])?;
            }
            client.wait_all_acknowledged()
        },
    );
    let elapsed = started.elapsed();
    proxy.stop();
    let failure = match outcome {
        Ok(Err(failure)) => failure,
        Ok(Ok(())) => return Err("服务被 kill -9 且没重起，四帧却都确认了".to_string()),
        Err(message) => {
            return Err(format!(
            "经 TCP 代理 kill -9 服务之后：{message}——客户端堵在写里（代理没把客户端那头真关掉）"
        ))
        }
    };
    let exit_code = failure.exit_code();
    if !matches!(failure, ClientFailure::ServerUnreachable { .. })
        || exit_code != PRUNE_SERVER_UNAVAILABLE_EXIT_CODE
    {
        return Err(format!(
            "经 TCP 代理 kill -9 服务之后该回「连不上」（退 {PRUNE_SERVER_UNAVAILABLE_EXIT_CODE}），回的是 {failure:?}"
        ));
    }
    Ok(format!(
        "经 TCP 代理 kill -9 服务：客户端的写出错、重连到 {} 秒上限，{:.1} 秒时回「连不上」（退 {exit_code}），没堵在写里（写超时设的是 {} 秒）",
        PROXY_KILL_RECONNECT_TOTAL.as_secs(),
        elapsed.as_secs_f64(),
        PROXY_KILL_WRITE_TIMEOUT.as_secs()
    ))
}
