//! 建连受重连总时长上限管的一格 `cell_connect_bounded_by_reconnect_total`：不通的地址、满了的积压队列、连上之后的读写都在总时长里。

use super::*;

/// 建连那一格：重连总时长上限、限时、超出上限还许多等的余量、调用方要停的时刻。
const CONNECT_TOTAL: Duration = Duration::from_millis(600);
pub(super) const CONNECT_CELL_DEADLINE: Duration = Duration::from_secs(10);
const CONNECT_RETURN_MARGIN: Duration = Duration::from_secs(2);
const CONNECT_STOP_AFTER: Duration = Duration::from_millis(300);
/// 连接队列的长度：Linux 上队列里排满它加一条之后，再来的连接排不进去。
const FULL_BACKLOG_LENGTH: libc::c_int = 1;
const FULL_BACKLOG_FILLERS: usize = 2;

pub(super) fn bounded_connect_settings(
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
