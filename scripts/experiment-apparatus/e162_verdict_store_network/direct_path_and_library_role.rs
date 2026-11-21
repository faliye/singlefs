//! 直写路径的报告与跑法、库进程的设置与参数、网络与对端通道的接收、库角色与转发角色。

use super::*;

/// 产物行的值里不许有空白：错误文字里的空白换成下划线，空的写 none。
pub(super) fn text_field(text: &str) -> String {
    if text.is_empty() {
        "none".to_string()
    } else {
        text.split_whitespace().collect::<Vec<_>>().join("_")
    }
}

// ============================== 库进程：直写线程 ==============================

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DirectPathReport {
    blocks: u64,
    confirmed: u64,
    started: u64,
    /// 第一次提交开始到最后一次提交返回（口径同 Q2a）。
    elapsed_microseconds: u64,
    commit_median_microseconds: u64,
    first_error: String,
}

/// 直写那一路：交错次序里 j 为偶数的块，块值由另一个线程预先生成、经深度 64 的有界通道交给写线程（口径同 Q2a）。
fn run_direct_path(mut store: ThreadStore, block_count: u64, states_per_block: u64) -> DirectPathReport {
    let (value_sender, prepared_blocks) = mpsc::sync_channel::<(BlockKey, Vec<u8>)>(64);
    let generator = std::thread::spawn(move || {
        let fingerprint = input_fingerprint();
        for direct_position in 0..block_count {
            let block_index = 2 * direct_position;
            let key = interleaved_block_key(block_index, states_per_block, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, states_per_block);
            if value_sender.send((key, value)).is_err() {
                return;
            }
        }
    });
    let mut report = DirectPathReport { blocks: block_count, ..DirectPathReport::default() };
    let mut first_commit_started: Option<Instant> = None;
    let mut last_commit_returned: Option<Instant> = None;
    let mut commit_latencies = Vec::with_capacity(usize::try_from(block_count).expect("块数装得进 usize"));
    for (key, value) in prepared_blocks.iter() {
        let commit_started = Instant::now();
        first_commit_started.get_or_insert(commit_started);
        report.started += 1;
        match store.put_block(&key, &value) {
            Ok(()) => {
                let commit_returned = Instant::now();
                last_commit_returned = Some(commit_returned);
                commit_latencies.push(u64::try_from((commit_returned - commit_started).as_micros()).expect("微秒装得进 u64"));
                report.confirmed += 1;
            }
            Err(error) => {
                report.first_error = error.to_string();
                break;
            }
        }
    }
    drop(prepared_blocks);
    generator.join().expect("生成块值的线程没 panic");
    if let (Some(first), Some(last)) = (first_commit_started, last_commit_returned) {
        report.elapsed_microseconds = u64::try_from((last - first).as_micros()).expect("微秒装得进 u64");
    }
    commit_latencies.sort_unstable();
    report.commit_median_microseconds = commit_latencies.get(commit_latencies.len() / 2).copied().unwrap_or(0);
    report
}

impl DirectPathReport {
    fn summary_line(&self) -> String {
        format!(
            "D blocks={} confirmed={} started={} elapsed_microseconds={} commit_median_microseconds={} first_error={}",
            self.blocks,
            self.confirmed,
            self.started,
            self.elapsed_microseconds,
            self.commit_median_microseconds,
            text_field(&self.first_error)
        )
    }
}

// ============================== 库进程 ==============================

/// 收块的数据通道：回环格在 127.0.0.1 上等转发进程连进来；跨机格库进程自己起 `<对端命令…> <中继命令>`，
/// 帧走它的标准输出、确认走它的标准输入，回出第 a_k 个确认之后杀掉它、停 u_k × 200 ms 再起（补 1 读法表「网络中断（跨机格）」）。
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ReceiveChannel {
    TcpListen,
    PeerCommand { command_words: Vec<String>, relay_command: String, interruptions: Vec<PlannedInterruption> },
}

pub(super) struct LibrarySettings {
    pub(super) candidate: Candidate,
    pub(super) library: PathBuf,
    pub(super) receive_mode: ReceiveMode,
    pub(super) direct_blocks: u64,
    pub(super) network_blocks: u64,
    pub(super) states_per_block: u64,
    pub(super) channel: ReceiveChannel,
}

pub(super) fn parse_flag_values(arguments: &[String]) -> Result<HashMap<String, String>, String> {
    let mut values = HashMap::new();
    let mut position = 0;
    while position < arguments.len() {
        let flag = arguments[position].clone();
        let value = arguments.get(position + 1).ok_or(format!("{flag} 后面缺值"))?.clone();
        values.insert(flag, value);
        position += 2;
    }
    Ok(values)
}

pub(super) fn required_number(values: &HashMap<String, String>, flag: &str) -> Result<u64, String> {
    values
        .get(flag)
        .ok_or(format!("缺 {flag}"))?
        .parse()
        .map_err(|_| format!("{flag} 不是数"))
}

impl LibrarySettings {
    pub(super) fn to_arguments(&self) -> Vec<String> {
        let mut arguments = vec![
            "library".to_string(),
            "--arm".to_string(),
            self.candidate.label().to_string(),
            "--library".to_string(),
            self.library.display().to_string(),
            "--mode".to_string(),
            self.receive_mode.label().to_string(),
            "--direct-blocks".to_string(),
            self.direct_blocks.to_string(),
            "--network-blocks".to_string(),
            self.network_blocks.to_string(),
            "--states-per-block".to_string(),
            self.states_per_block.to_string(),
        ];
        if let ReceiveChannel::PeerCommand { command_words, relay_command, interruptions } = &self.channel {
            arguments.push("--peer-command".to_string());
            arguments.push(command_words.join(" "));
            arguments.push("--relay-command".to_string());
            arguments.push(relay_command.clone());
            arguments.push("--interruptions".to_string());
            arguments.push(interruptions.len().to_string());
        }
        arguments
    }

    pub(super) fn parse(arguments: &[String]) -> Result<LibrarySettings, String> {
        let values = parse_flag_values(arguments)?;
        let candidate = Candidate::parse(values.get("--arm").ok_or("缺 --arm")?).ok_or("认不出的臂")?;
        let receive_mode = ReceiveMode::parse(values.get("--mode").ok_or("缺 --mode")?).ok_or("认不出的收块写法")?;
        let network_blocks = required_number(&values, "--network-blocks")?;
        let channel = match (values.get("--peer-command"), values.get("--relay-command")) {
            (Some(command), Some(relay_command)) => {
                let interruption_count = usize::try_from(required_number(&values, "--interruptions")?).map_err(|_| "中断数装不进 usize")?;
                ReceiveChannel::PeerCommand {
                    command_words: command.split_whitespace().map(str::to_string).collect(),
                    relay_command: relay_command.clone(),
                    interruptions: interruption_plan(interruption_count, network_blocks),
                }
            }
            (None, None) => ReceiveChannel::TcpListen,
            _ => return Err("--peer-command 与 --relay-command 要一起给".to_string()),
        };
        Ok(LibrarySettings {
            candidate,
            library: PathBuf::from(values.get("--library").ok_or("缺 --library")?),
            receive_mode,
            direct_blocks: required_number(&values, "--direct-blocks")?,
            network_blocks,
            states_per_block: required_number(&values, "--states-per-block")?,
            channel,
        })
    }
}

pub(super) fn print_flushed(line: &str) {
    let mut standard_output = std::io::stdout().lock();
    writeln!(standard_output, "{line}").expect("写得进标准输出");
    standard_output.flush().expect("刷得出标准输出");
}

struct ReceiveOutcome {
    receiver: FrameReceiver,
    connection_ends: BTreeSet<String>,
    interruption_lines: Vec<String>,
}

fn receive_over_tcp(mut receiver: FrameReceiver, listener: TcpListener, stop_requested: Arc<AtomicBool>) -> ReceiveOutcome {
    let mut connection_ends = BTreeSet::new();
    for incoming in listener.incoming() {
        if stop_requested.load(Ordering::SeqCst) {
            break;
        }
        let Ok(stream) = incoming else { continue };
        stream.set_nodelay(true).expect("TCP 连接设得了 nodelay");
        let mut writer = stream.try_clone().expect("连接复制得出写的一端");
        let mut reader = stream;
        let end = receiver.serve_connection(&mut reader, &mut writer, None);
        connection_ends.insert(end.label().to_string());
    }
    ReceiveOutcome { receiver, connection_ends, interruption_lines: Vec::new() }
}

fn spawn_peer_channel(command_words: &[String], relay_command: &str) -> std::io::Result<Child> {
    let (program, fixed_arguments) = command_words.split_first().expect("对端命令至少一个词");
    Command::new(program)
        .args(fixed_arguments)
        .arg(relay_command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
}

fn receive_over_peer_channel(
    mut receiver: FrameReceiver,
    command_words: Vec<String>,
    relay_command: String,
    interruptions: Vec<PlannedInterruption>,
    stop_requested: Arc<AtomicBool>,
    current_channel: Arc<Mutex<Option<Child>>>,
) -> ReceiveOutcome {
    let mut connection_ends = BTreeSet::new();
    let mut interruption_lines = Vec::new();
    let mut next_interruption = 0usize;
    while !stop_requested.load(Ordering::SeqCst) {
        let mut child = match spawn_peer_channel(&command_words, &relay_command) {
            Ok(child) => child,
            Err(error) => {
                connection_ends.insert(format!("spawn_failed:{}", text_field(&error.to_string())));
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        };
        let mut channel_output = child.stdout.take().expect("数据通道的标准输出接在管道上");
        let mut channel_input = child.stdin.take().expect("数据通道的标准输入接在管道上");
        *current_channel.lock().expect("数据通道的锁没中毒") = Some(child);
        if stop_requested.load(Ordering::SeqCst) {
            break;
        }
        let threshold = interruptions.get(next_interruption).map(|interruption| interruption.acknowledgement_threshold);
        let frames_before = receiver.counters.complete_frames;
        let end = receiver.serve_connection(&mut channel_output, &mut channel_input, threshold);
        connection_ends.insert(end.label().to_string());
        if receiver.counters.complete_frames == frames_before && end != ConnectionEnd::InterruptionDue {
            // 数据通道一帧没送就断了（中继还没连上送块进程、或对端命令起不来）：隔 10 ms 再起，不空转。
            std::thread::sleep(Duration::from_millis(10));
        }
        if let Some(mut finished) = current_channel.lock().expect("数据通道的锁没中毒").take() {
            let _already_exited = finished.kill();
            finished.wait().expect("收得了数据通道子进程");
        }
        drop(channel_input);
        drop(channel_output);
        if end == ConnectionEnd::InterruptionDue {
            let interruption = interruptions[next_interruption];
            let pause = pause_of(&interruption);
            let paused_at = Instant::now();
            std::thread::sleep(pause);
            interruption_lines.push(format!(
                "I index={next_interruption} threshold={} acknowledgements_sent={} pause_microseconds={}",
                interruption.acknowledgement_threshold,
                receiver.counters.acknowledgements_sent,
                paused_at.elapsed().as_micros()
            ));
            next_interruption += 1;
        }
    }
    ReceiveOutcome { receiver, connection_ends, interruption_lines }
}

pub(super) fn run_library_role(arguments: &[String]) -> i32 {
    let settings = match LibrarySettings::parse(arguments) {
        Ok(settings) => settings,
        Err(problem) => {
            eprintln!("  ✗ library 的参数不对：{problem}");
            return 2;
        }
    };
    if anchor_checks().iter().any(|check| !check.matches()) {
        print_flushed("E anchor_mismatch");
        return 5;
    }
    let mut stores = match open_thread_stores(settings.candidate, &settings.library, 2) {
        Ok(stores) => stores,
        Err(error) => {
            print_flushed(&format!("E open_failed={}", text_field(&error.to_string())));
            return 6;
        }
    };
    let receive_store = stores.pop().expect("开了两个句柄");
    let direct_store = stores.pop().expect("开了两个句柄");
    let receiver = FrameReceiver::new(settings.receive_mode, settings.states_per_block, settings.network_blocks, receive_store);
    let stop_requested = Arc::new(AtomicBool::new(false));
    let current_channel: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let (receive_thread, own_port) = match settings.channel.clone() {
        ReceiveChannel::TcpListen => {
            let listener = TcpListener::bind("127.0.0.1:0").expect("绑得上回环端口");
            let port = listener.local_addr().expect("有地址").port();
            let stop = Arc::clone(&stop_requested);
            let thread = std::thread::spawn(move || receive_over_tcp(receiver, listener, stop));
            print_flushed(&format!("L port={port}"));
            (thread, Some(port))
        }
        ReceiveChannel::PeerCommand { command_words, relay_command, interruptions } => {
            let stop = Arc::clone(&stop_requested);
            let channel_slot = Arc::clone(&current_channel);
            let thread = std::thread::spawn(move || {
                receive_over_peer_channel(receiver, command_words, relay_command, interruptions, stop, channel_slot)
            });
            print_flushed("L port=none");
            (thread, None)
        }
    };
    let direct_blocks = settings.direct_blocks;
    let states_per_block = settings.states_per_block;
    let direct_thread = std::thread::spawn(move || run_direct_path(direct_store, direct_blocks, states_per_block));
    let _consumed = std::io::copy(&mut std::io::stdin().lock(), &mut std::io::sink());
    stop_requested.store(true, Ordering::SeqCst);
    if let Some(port) = own_port {
        let _wake_accept = TcpStream::connect(("127.0.0.1", port));
    }
    if let Some(mut channel) = current_channel.lock().expect("数据通道的锁没中毒").take() {
        let _already_exited = channel.kill();
        let _reaped = channel.wait();
    }
    let outcome = receive_thread.join().expect("收块线程没 panic");
    let direct_report = direct_thread.join().expect("直写线程没 panic");
    print_flushed(&direct_report.summary_line());
    for line in &outcome.interruption_lines {
        print_flushed(line);
    }
    print_flushed(&outcome.receiver.summary_line(&outcome.connection_ends));
    0
}

// ============================== 回环转发进程 ==============================

fn copy_until_end(mut from: TcpStream, mut to: TcpStream) {
    let mut chunk = vec![0u8; 65_536];
    loop {
        match from.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(read_bytes) => {
                if to.write_all(&chunk[..read_bytes]).is_err() {
                    break;
                }
            }
        }
    }
    let _already_closed = to.shutdown(std::net::Shutdown::Write);
}

/// 双向逐字节转发（登记 6.3 末段）：在 --listen-port 上等（0 = 让内核挑，挑到的端口打一行 `F port=<端口>`），
/// 每个连进来的连接接到 127.0.0.1:<--target-port>；一直跑到被杀。
pub(super) fn run_forward_role(arguments: &[String]) -> i32 {
    let values = match parse_flag_values(arguments) {
        Ok(values) => values,
        Err(problem) => {
            eprintln!("  ✗ forward 的参数不对：{problem}");
            return 2;
        }
    };
    let (Ok(listen_port), Ok(target_port)) = (required_number(&values, "--listen-port"), required_number(&values, "--target-port")) else {
        eprintln!("  ✗ forward 要 --listen-port 与 --target-port");
        return 2;
    };
    let listen_port = u16::try_from(listen_port).expect("端口装得进 u16");
    let target_port = u16::try_from(target_port).expect("端口装得进 u16");
    let started = Instant::now();
    let listener = loop {
        match TcpListener::bind(("127.0.0.1", listen_port)) {
            Ok(listener) => break listener,
            Err(error) if started.elapsed() < FORWARDER_BIND_DEADLINE => {
                let _retry_after = error;
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                eprintln!("  ✗ forward 绑不上 127.0.0.1:{listen_port}：{error}");
                return 3;
            }
        }
    };
    print_flushed(&format!("F port={}", listener.local_addr().expect("有地址").port()));
    for incoming in listener.incoming() {
        let Ok(client) = incoming else { continue };
        std::thread::spawn(move || {
            let Ok(library) = TcpStream::connect(("127.0.0.1", target_port)) else { return };
            client.set_nodelay(true).expect("nodelay");
            library.set_nodelay(true).expect("nodelay");
            let client_reader = client.try_clone().expect("复制");
            let library_reader = library.try_clone().expect("复制");
            let upstream = std::thread::spawn(move || copy_until_end(client_reader, library));
            copy_until_end(library_reader, client);
            upstream.join().expect("转发线程没 panic");
        });
    }
    0
}

#[cfg(test)]
#[path = "direct_path_and_library_role_tests.rs"]
mod tests;
