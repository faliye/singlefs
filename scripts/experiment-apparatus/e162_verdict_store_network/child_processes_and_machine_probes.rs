//! 子进程的命令与起停、行的读取、临时目录、机器的安静度与干扰进程的探测。

use super::*;

// ============================== 父进程：起子进程、读它的行 ==============================

/// 起这个 bin 的一个子进程角色（library、forward、verify）；单测里走 libtest 自起（见 tests 模块）。
#[cfg(not(test))]
pub(super) fn child_command(role_arguments: &[String]) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("找得到自己的可执行文件"));
    command.args(role_arguments);
    command
}

#[cfg(test)]
pub(super) fn child_command(role_arguments: &[String]) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("找得到测试二进制"));
    command
        .args(["--exact", "tests::child_process_role_entry", "--nocapture", "--test-threads=1", "--quiet"])
        .env(tests::CHILD_ROLE_ENVIRONMENT, role_arguments.join("\u{1f}"));
    command
}

/// 送块进程：同一个目录下的 `e162-verdict-store-sender`；单测里是测试二进制自起送块角色。
#[cfg(not(test))]
pub(super) fn sender_command(sender_arguments: &[String]) -> Command {
    let mut command = Command::new(sender_binary_path());
    command.args(sender_arguments);
    command
}

#[cfg(test)]
pub(super) fn sender_command(sender_arguments: &[String]) -> Command {
    child_command(sender_arguments)
}

pub(super) fn sender_binary_path() -> PathBuf {
    std::env::current_exe().expect("找得到自己的可执行文件").with_file_name(SENDER_BINARY_NAME)
}

/// 送块进程的参数；与 `SenderSettings::parse` 互为反过来（单测往返核）。
pub(super) fn sender_arguments(settings: &SenderSettings) -> Vec<String> {
    let mut arguments = vec!["send".to_string()];
    match &settings.endpoint {
        SenderEndpoint::ConnectTcp(address) => {
            arguments.push("--connect".to_string());
            arguments.push(format!("tcp:{address}"));
        }
        SenderEndpoint::ListenUnix(path) => {
            arguments.push("--listen".to_string());
            arguments.push(format!("unix:{}", path.display()));
        }
    }
    let pace_milliseconds = settings
        .pace_between_first_transmissions
        .map_or(0, |pace| u64::try_from(pace.as_millis()).expect("节拍毫秒数装得进 u64"));
    for (flag, value) in [
        ("--blocks", settings.block_count.to_string()),
        ("--states-per-block", settings.states_per_block.to_string()),
        ("--window", settings.window_frames.to_string()),
        ("--pace-milliseconds", pace_milliseconds.to_string()),
        ("--stall-seconds", settings.acknowledgement_stall_limit.as_secs().to_string()),
        ("--spin-threads", settings.spinning_thread_count.to_string()),
    ] {
        arguments.push(flag.to_string());
        arguments.push(value);
    }
    arguments
}

pub(super) struct RunningChild {
    pub(super) child: Child,
    pub(super) lines: mpsc::Receiver<String>,
    pub(super) input: Option<ChildStdin>,
}

pub(super) fn spawn_running_child(mut command: Command, keep_input_open: bool) -> RunningChild {
    let mut child = command
        .stdin(if keep_input_open { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("起得了子进程");
    let standard_output = child.stdout.take().expect("子进程的标准输出接在管道上");
    let input = child.stdin.take();
    let (line_sender, lines) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(standard_output).lines() {
            let Ok(line) = line else { break };
            if line_sender.send(line).is_err() {
                break;
            }
        }
    });
    RunningChild { child, lines, input }
}

/// 等子进程打出以 prefix 起头的一行；子进程先退了或超时返回 None，已读到的行放进 seen。
pub(super) fn wait_for_line(running: &RunningChild, prefix: &str, seen: &mut Vec<String>) -> Option<String> {
    let deadline = Instant::now() + CHILD_START_DEADLINE;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match running.lines.recv_timeout(remaining) {
            Ok(line) if line.starts_with(prefix) => return Some(line),
            Ok(line) => seen.push(line),
            Err(_ended_or_timed_out) => return None,
        }
    }
}

/// 对子进程的 pid 发 SIGKILL（`Child::kill` 在 Unix 上发的就是 SIGKILL），再收掉它。
pub(super) fn kill_and_reap(child: &mut Child) {
    let _already_exited = child.kill();
    child.wait().expect("收得了被杀的子进程");
}

pub(super) fn field_value<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    line.split(' ').find_map(|field| field.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
}

pub(super) fn fields_of(line: &str) -> Vec<(String, String)> {
    line.split(' ')
        .filter_map(|field| field.split_once('=').map(|(key, value)| (key.to_string(), value.to_string())))
        .collect()
}

pub(super) fn number_field(line: &str, key: &str) -> u64 {
    field_value(line, key).and_then(|text| text.parse().ok()).unwrap_or(0)
}

// ============================== 盘上位置与环境核查（同 S1 装置） ==============================

/// `${TMPDIR:-/tmp}/e162-<pid>/`，Drop 时整目录删掉；E162_KEEP_SCRATCH=1 时留下并打印路径。
pub(super) struct ScratchDirectory {
    pub(super) path: PathBuf,
}

impl ScratchDirectory {
    pub(super) fn create() -> ScratchDirectory {
        let base = std::env::var_os("TMPDIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
        let path = base.join(format!("e162-{}", std::process::id()));
        fs::create_dir_all(&path).expect("建得了盘上位置");
        ScratchDirectory { path }
    }
}

impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        if std::env::var("E162_KEEP_SCRATCH").is_ok_and(|value| value == "1") {
            eprintln!("E162_KEEP_SCRATCH=1：现场留在 {}", self.path.display());
        } else {
            let _cleanup_outcome = fs::remove_dir_all(&self.path);
        }
    }
}

pub(super) fn command_output_text(program: &str, arguments: &[&str]) -> String {
    Command::new(program)
        .args(arguments)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default()
}

/// V9：`stat -f -c %T`；H4：`df -B1` 的可用字节。
pub(super) fn filesystem_type_and_available_bytes(directory: &Path) -> (String, u64) {
    let directory_text = directory.to_string_lossy().to_string();
    let filesystem_type = command_output_text("stat", &["-f", "-c", "%T", &directory_text]);
    let available_text = command_output_text("df", &["-B1", "--output=avail", &directory_text]);
    let available = available_text.lines().last().and_then(|line| line.trim().parse().ok()).unwrap_or(0);
    (filesystem_type, available)
}

/// V15：本用户的进程里有没有会干扰计时的（按可执行文件名与参数逐个比，不用模式串比整条命令行）。
pub(super) fn interfering_processes() -> Vec<String> {
    use std::os::unix::fs::MetadataExt;
    let own_user = fs::metadata("/proc/self").map(|metadata| metadata.uid()).unwrap_or(u32::MAX);
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else { return found };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        if fs::metadata(entry.path()).map(|metadata| metadata.uid()).ok() != Some(own_user) {
            continue;
        }
        let Ok(command_line) = fs::read(entry.path().join("cmdline")) else { continue };
        let words: Vec<String> = command_line
            .split(|&byte| byte == 0)
            .filter(|word| !word.is_empty())
            .map(|word| String::from_utf8_lossy(word).to_string())
            .collect();
        let Some(program) = words.first() else { continue };
        let program_name = Path::new(program).file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default();
        let interferes = program_name.starts_with("qemu-system")
            || program_name == "cargo"
            || program_name == "rustc"
            || program_name == "fio"
            || program_name == "e152-file-system-benchmark"
            || words.iter().any(|word| word.ends_with("vm-bench.sh") || word.contains("e161"));
        if interferes {
            found.push(format!("{name}:{program_name}"));
        }
    }
    found.sort();
    found
}

pub(super) fn load_average_text() -> String {
    fs::read_to_string("/proc/loadavg").unwrap_or_default().split_whitespace().take(3).collect::<Vec<_>>().join("/")
}

/// /proc/diskstats 里 nvme0n1 的写扇区数（第 10 列）。
pub(super) fn nvme_written_sectors() -> u64 {
    fs::read_to_string("/proc/diskstats")
        .unwrap_or_default()
        .lines()
        .find_map(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            (columns.get(2) == Some(&"nvme0n1")).then(|| columns.get(9).and_then(|value| value.parse().ok())).flatten()
        })
        .unwrap_or(0)
}

pub(super) fn list_or_dash(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(",")
    }
}

/// 修订五：每遍开跑之前先等干扰进程散去，每 30 秒看一次、最多等 20 分钟；返回等了多少秒。
pub(super) fn wait_for_quiet_machine(maximum_wait_seconds: u64) -> u64 {
    let started = Instant::now();
    while !interfering_processes().is_empty() && started.elapsed().as_secs() < maximum_wait_seconds {
        std::thread::sleep(Duration::from_secs(INTERFERENCE_POLL_SECONDS));
    }
    started.elapsed().as_secs()
}
