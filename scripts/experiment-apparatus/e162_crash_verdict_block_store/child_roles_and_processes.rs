//! 子进程角色的参数与写、读、核对三个角色，起子进程、收输出与读本机状态。

use super::*;

// ============================== 子进程角色的参数 ==============================

/// 子进程参数一律写成 `键=值`。
pub(super) struct RoleArguments {
    fields: BTreeMap<String, String>,
}

impl RoleArguments {
    pub(super) fn parse(arguments: &[String]) -> RoleArguments {
        let fields = arguments
            .iter()
            .map(|argument| {
                let (key, value) = argument.split_once('=').unwrap_or_else(|| panic!("子进程参数 {argument} 不是 键=值"));
                (key.to_string(), value.to_string())
            })
            .collect();
        RoleArguments { fields }
    }

    fn text(&self, key: &str) -> &str {
        self.fields.get(key).unwrap_or_else(|| panic!("子进程少了参数 {key}"))
    }

    fn number(&self, key: &str) -> u64 {
        self.text(key).parse().unwrap_or_else(|_| panic!("子进程参数 {key} 不是整数"))
    }

    pub(super) fn candidate(&self) -> Candidate {
        Candidate::parse(self.text("candidate")).expect("认得的臂")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WriteOrder {
    Interleaved,
    RandomPermutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProgressLines {
    /// S1：每块写之前一行 `B <j>`、提交返回之后一行 `C <j>`。
    EveryBlock,
    /// S2：只在每写满 1 000 块时一行 `C <已写块数>`，给父进程排占用测量。
    EveryWindow,
}

/// 一次写一行、不经缓冲：锁住标准输出、整行 write_all、flush。
fn emit_progress_line(line: &str) {
    let mut standard_output = std::io::stdout().lock();
    standard_output.write_all(line.as_bytes()).expect("写得进标准输出");
    standard_output.flush().expect("刷得出标准输出");
}

pub(super) fn sorted_percentile(sorted: &[u64], numerator: usize, denominator: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (sorted.len() * numerator).div_ceil(denominator).max(1);
    sorted[rank - 1]
}

pub(super) fn median_of(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted_percentile(&sorted, 1, 2)
}

/// A-raw：在 directory 里 1 000 次「追加 64 KiB（P-rand）+ fdatasync」，返回每次的微秒数。
pub(super) fn raw_append_fdatasync_probe(directory: &Path) -> Vec<u64> {
    let probe_path = directory.join("e162-raw-probe.bin");
    let mut probe = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&probe_path)
        .expect("建得了 A-raw 探针文件");
    let mut latencies = Vec::with_capacity(RAW_PROBE_APPENDS);
    for append_index in 0..RAW_PROBE_APPENDS {
        let states = u64::try_from(RAW_PROBE_APPEND_BYTES).expect("64 KiB");
        let payload = block_value(ValuePattern::Random, SEED_READ, u64::try_from(append_index).expect("小于 1000"), states);
        let started = Instant::now();
        probe.write_all(&payload).expect("写得进 A-raw 探针文件");
        probe.sync_data().expect("fdatasync 得了 A-raw 探针文件");
        latencies.push(elapsed_microseconds(started));
    }
    drop(probe);
    fs::remove_file(&probe_path).expect("删得了 A-raw 探针文件");
    latencies
}

/// 写入子进程：`writer candidate= write_path= library= seed= pattern= states= order= batch= first= end= sleep= timed= probe= progress=`。
/// end=0 是一直写到被杀。
pub(super) fn run_writer_role(arguments: &[String]) -> i32 {
    let role = RoleArguments::parse(arguments);
    let candidate = role.candidate();
    let write_path = WritePath::parse(role.text("write_path"));
    let library = PathBuf::from(role.text("library"));
    let seed = role.number("seed");
    let pattern = ValuePattern::parse(role.text("pattern"));
    let states_per_block = role.number("states");
    let order = match role.text("order") {
        "interleaved" => WriteOrder::Interleaved,
        "random" => WriteOrder::RandomPermutation,
        other => panic!("认不出的次序 {other}"),
    };
    let batch = usize::try_from(role.number("batch")).expect("批大小装得进 usize");
    let first_position = role.number("first");
    let end_position = match role.number("end") {
        0 => u64::MAX,
        end => end,
    };
    let sleep_microseconds = role.number("sleep");
    let timed = role.number("timed") == 1;
    let progress = match role.text("progress") {
        "every-block" => ProgressLines::EveryBlock,
        "every-window" => ProgressLines::EveryWindow,
        other => panic!("认不出的进度行形态 {other}"),
    };
    if role.number("probe") == 1 {
        let parent = library.parent().expect("库有父目录");
        let latencies = raw_append_fdatasync_probe(parent);
        emit_progress_line(&format!("P raw_median_microseconds={}\n", median_of(&latencies)));
    }
    let open_started = Instant::now();
    let mut store = match open_store_for_writing(candidate, write_path, &library) {
        Ok(store) => store,
        Err(error) => {
            emit_progress_line(&format!("E open {}\n", error.to_string().replace(char::is_whitespace, "_")));
            return 3;
        }
    };
    emit_progress_line(&format!("O {}\n", elapsed_microseconds(open_started)));
    let (sender, receiver) = mpsc::sync_channel::<(u64, BlockKey, Vec<u8>)>(GENERATOR_CHANNEL_DEPTH);
    let generator = std::thread::spawn(move || {
        let fingerprint = input_fingerprint();
        let permutation = match order {
            WriteOrder::Interleaved => Vec::new(),
            WriteOrder::RandomPermutation => random_permutation(end_position, SEED_ORDER),
        };
        let mut position = first_position;
        while position < end_position {
            let block_index = match order {
                WriteOrder::Interleaved => position,
                WriteOrder::RandomPermutation => permutation[usize::try_from(position).expect("位置装得进 usize")],
            };
            let key = interleaved_block_key(block_index, states_per_block, &fingerprint);
            let value = block_value(pattern, seed, block_index, states_per_block);
            if sender.send((block_index, key, value)).is_err() {
                return;
            }
            position += 1;
        }
    });
    let mut commit_latencies: Vec<u64> = Vec::new();
    let mut block_return_offsets: Vec<u64> = Vec::new();
    let mut first_commit_started: Option<Instant> = None;
    let mut written_blocks = 0u64;
    loop {
        let mut batch_blocks: Vec<(u64, BlockKey, Vec<u8>)> = Vec::with_capacity(batch);
        while batch_blocks.len() < batch {
            match receiver.recv() {
                Ok(item) => batch_blocks.push(item),
                Err(_) => break,
            }
        }
        if batch_blocks.is_empty() {
            break;
        }
        if progress == ProgressLines::EveryBlock {
            for (block_index, _, _) in &batch_blocks {
                emit_progress_line(&format!("B {block_index}\n"));
            }
        }
        let pairs: Vec<(BlockKey, Vec<u8>)> = batch_blocks.iter().map(|(_, key, value)| (*key, value.clone())).collect();
        let commit_started = Instant::now();
        let origin = *first_commit_started.get_or_insert(commit_started);
        if sleep_microseconds > 0 {
            std::thread::sleep(Duration::from_micros(sleep_microseconds));
        }
        let committed = store.put_blocks(&pairs);
        let commit_returned = Instant::now();
        if let Err(error) = committed {
            emit_progress_line(&format!("E commit {}\n", error.to_string().replace(char::is_whitespace, "_")));
            return 3;
        }
        commit_latencies.push(u64::try_from((commit_returned - commit_started).as_micros()).expect("装得进 u64"));
        let return_offset = u64::try_from((commit_returned - origin).as_micros()).expect("装得进 u64");
        for (block_index, _, _) in &batch_blocks {
            written_blocks += 1;
            block_return_offsets.push(return_offset);
            match progress {
                ProgressLines::EveryBlock => emit_progress_line(&format!("C {block_index}\n")),
                ProgressLines::EveryWindow => {
                    if written_blocks % WINDOW_BLOCKS == 0 {
                        emit_progress_line(&format!("C {written_blocks}\n"));
                    }
                }
            }
        }
    }
    generator.join().expect("生成线程没有 panic");
    if timed {
        for line in timing_summary_lines(&commit_latencies, &block_return_offsets, states_per_block) {
            emit_progress_line(&format!("{line}\n"));
        }
    }
    drop(store);
    emit_progress_line("D\n");
    0
}

/// 计时汇总：`T` 一行（Q2a、提交时延分布、Q2c 在几个产出速率下的最大积压），`W` 每窗一行（Q2b）。
pub(super) fn timing_summary_lines(commit_latencies: &[u64], block_return_offsets: &[u64], states_per_block: u64) -> Vec<String> {
    let block_count = u64::try_from(block_return_offsets.len()).expect("装得进 u64");
    let elapsed = block_return_offsets.last().copied().unwrap_or(0).max(1);
    let blocks_per_second = block_count as f64 * 1e6 / elapsed as f64;
    let states_per_second = blocks_per_second * states_per_block as f64;
    let mut sorted = commit_latencies.to_vec();
    sorted.sort_unstable();
    let mut lines = vec![format!(
        "T blocks={block_count} commits={} elapsed_microseconds={elapsed} blocks_per_second={blocks_per_second:.3} states_per_second={states_per_second:.1} commit_median_microseconds={} commit_percentile_99_microseconds={} commit_maximum_microseconds={}",
        commit_latencies.len(),
        sorted_percentile(&sorted, 1, 2),
        sorted_percentile(&sorted, 99, 100),
        sorted.last().copied().unwrap_or(0),
    )];
    for fraction_percent in [50u64, 75, 90, 100] {
        let production_states_per_microsecond = states_per_second * fraction_percent as f64 / 100.0 / 1e6;
        let mut maximum_backlog_states = 0f64;
        let mut peak_block = 0u64;
        for (consumed_before, &return_offset) in block_return_offsets.iter().enumerate() {
            let produced = production_states_per_microsecond * return_offset as f64;
            let consumed = consumed_before as f64 * states_per_block as f64;
            if produced - consumed > maximum_backlog_states {
                maximum_backlog_states = produced - consumed;
                peak_block = u64::try_from(consumed_before).expect("装得进 u64") + 1;
            }
        }
        lines.push(format!(
            "Q backlog_rate_percent_of_measured={fraction_percent} maximum_backlog_states={maximum_backlog_states:.0} peak_at_block={peak_block}"
        ));
    }
    let mut window_start = 0u64;
    for window_index in 0..block_count / WINDOW_BLOCKS {
        let last_block = usize::try_from((window_index + 1) * WINDOW_BLOCKS - 1).expect("装得进 usize");
        let window_end = block_return_offsets[last_block];
        let window_elapsed = (window_end - window_start).max(1);
        let window_states_per_second = (WINDOW_BLOCKS * states_per_block) as f64 * 1e6 / window_elapsed as f64;
        lines.push(format!(
            "W window={window_index} end_microseconds={window_end} states_per_second={window_states_per_second:.1}"
        ));
        window_start = window_end;
    }
    lines
}

// ============================== 读子进程与核对子进程 ==============================

fn process_read_bytes() -> u64 {
    let text = fs::read_to_string("/proc/self/io").unwrap_or_default();
    text.lines()
        .find_map(|line| line.strip_prefix("read_bytes: "))
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0)
}

/// 读子进程：`read candidate= library= seed= pattern= states= blocks= count= sleep= passes=`。
/// 块键按 seed_READ 在 [0, blocks) 里均匀取 count 个；每次一次读调用连同整块值拷进自己的缓冲区，计时只包这一段
/// （sleep > 0 时计时区间里、读调用之前再睡这么久，给 PC-read）。passes=2 是冷一遍再在同一个打开的库上热一遍。
pub(super) fn run_read_role(arguments: &[String]) -> i32 {
    let role = RoleArguments::parse(arguments);
    let candidate = role.candidate();
    let library = PathBuf::from(role.text("library"));
    let seed = role.number("seed");
    let pattern = ValuePattern::parse(role.text("pattern"));
    let states_per_block = role.number("states");
    let block_count = role.number("blocks");
    let read_count = usize::try_from(role.number("count")).expect("装得进 usize");
    let sleep_microseconds = role.number("sleep");
    let passes = role.number("passes");
    let open_started = Instant::now();
    let store = match open_existing_store(candidate, &library) {
        Ok(store) => store,
        Err(error) => {
            emit_progress_line(&format!("E open {}\n", error.to_string().replace(char::is_whitespace, "_")));
            return 3;
        }
    };
    let open_microseconds = elapsed_microseconds(open_started);
    let fingerprint = input_fingerprint();
    let mut generator = SplitMix64::new(SEED_READ);
    let chosen: Vec<u64> = (0..read_count).map(|_| generator.next_output() % block_count).collect();
    let distinct = chosen.iter().collect::<HashSet<_>>().len();
    for pass_index in 0..passes {
        let read_bytes_before = process_read_bytes();
        let mut latencies = Vec::with_capacity(read_count);
        let mut own_buffer: Vec<u8> = Vec::new();
        let mut mismatches = 0u64;
        for &block_index in &chosen {
            let key = interleaved_block_key(block_index, states_per_block, &fingerprint);
            let started = Instant::now();
            if sleep_microseconds > 0 {
                std::thread::sleep(Duration::from_micros(sleep_microseconds));
            }
            let outcome = store.read_block(&key);
            if let ReadOutcome::Found(value) = &outcome {
                own_buffer.clear();
                own_buffer.extend_from_slice(value);
            }
            latencies.push(elapsed_microseconds(started));
            let expected = block_value(pattern, seed, block_index, states_per_block);
            if !matches!(outcome, ReadOutcome::Found(_)) || own_buffer != expected {
                mismatches += 1;
            }
        }
        let read_bytes_delta = process_read_bytes().saturating_sub(read_bytes_before);
        let mut sorted = latencies.clone();
        sorted.sort_unstable();
        let pass_label = if pass_index == 0 { "first" } else { "second" };
        emit_progress_line(&format!(
            "R pass={pass_label} reads={read_count} distinct_blocks={distinct} open_microseconds={open_microseconds} median_microseconds={} percentile_99_microseconds={} maximum_microseconds={} mismatches={mismatches} read_bytes_delta={read_bytes_delta}\n",
            sorted_percentile(&sorted, 1, 2),
            sorted_percentile(&sorted, 99, 100),
            sorted.last().copied().unwrap_or(0),
        ));
    }
    0
}

/// 核对子进程：`verify candidate= library= seed= pattern= states= confirmed_end= started_end= checked=`，
/// checked 是 `range:<起>:<止>` 或 `list:<逗号隔开的块号>`。
pub(super) fn run_verify_role(arguments: &[String]) -> i32 {
    let role = RoleArguments::parse(arguments);
    let checked_text = role.text("checked");
    let checked_blocks = if let Some(range) = checked_text.strip_prefix("range:") {
        let (start, end) = range.split_once(':').expect("range:<起>:<止>");
        CheckedBlocks::Range {
            start: start.parse().expect("起点是整数"),
            end: end.parse().expect("止点是整数"),
        }
    } else {
        CheckedBlocks::Listed(split_numbers(checked_text.strip_prefix("list:").expect("list:<块号>")))
    };
    let plan = VerificationPlan {
        seed: role.number("seed"),
        pattern: ValuePattern::parse(role.text("pattern")),
        states_per_block: role.number("states"),
        confirmed_end: role.number("confirmed_end"),
        started_end: role.number("started_end"),
        checked_blocks,
    };
    let report = verify_library(role.candidate(), Path::new(role.text("library")), &plan);
    emit_progress_line(&format!("{}\n", encode_verification_report(&report)));
    0
}

// ============================== 父进程：起子进程、读它的行 ==============================

/// 起这个 bin 的一个子进程角色。单测里 current_exe 是测试二进制，改走 libtest 自起一个测试函数（见 tests 模块）。
#[cfg(not(test))]
fn child_command(role_arguments: &[String]) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("找得到自己的可执行文件"));
    command.args(role_arguments);
    command
}

#[cfg(test)]
fn child_command(role_arguments: &[String]) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("找得到测试二进制"));
    command
        .args(["--exact", "tests::child_process_role_entry", "--nocapture", "--test-threads=1", "--quiet"])
        .env(tests::CHILD_ROLE_ENVIRONMENT, role_arguments.join("\u{1f}"));
    command
}

pub(super) fn spawn_child(role_arguments: &[String]) -> (Child, mpsc::Receiver<String>) {
    let mut child = child_command(role_arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("起得了子进程");
    let standard_output = child.stdout.take().expect("子进程的标准输出接在管道上");
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(standard_output).lines() {
            let Ok(line) = line else { break };
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    (child, receiver)
}

/// 对子进程的 pid 发 SIGKILL（`Child::kill` 在 Unix 上发的就是 SIGKILL），再收掉它。
pub(super) fn kill_and_reap(child: &mut Child) {
    let _already_exited = child.kill();
    child.wait().expect("收得了被杀的子进程");
}

/// 子进程跑完（或被杀）之后把剩下的行读到 EOF。
pub(super) fn drain_lines(receiver: &mpsc::Receiver<String>) -> Vec<String> {
    receiver.iter().collect()
}

pub(super) fn run_child_to_completion(role_arguments: &[String]) -> (Vec<String>, i32) {
    let (mut child, receiver) = spawn_child(role_arguments);
    let lines = drain_lines(&receiver);
    let status = child.wait().expect("收得了子进程");
    (lines, status.code().unwrap_or(-1))
}

pub(super) fn field_value<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    line.split(' ').find_map(|field| field.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
}

// ============================== 盘上位置与环境核查 ==============================

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

fn command_output_text(program: &str, arguments: &[&str]) -> String {
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
