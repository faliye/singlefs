//! 对端不读时回应受额度管的一格 `cell_unread_responses_bounded`，与没收完的帧受额度与期限管的一格 `cell_incomplete_frames_bounded`。

use super::*;

/// 对端不读那一格的数：值 1 MiB、每条连接额度 4 MiB、只发不收的点读 64 次、写期限 3 秒。
const UNREAD_VALUE_BYTES: usize = 1024 * 1024;
const UNREAD_PER_CONNECTION_BYTES: usize = 4 * 1024 * 1024;
const UNREAD_SERVICE_PENDING_BYTES: usize = 64 * 1024 * 1024;
const UNREAD_READS: u64 = 64;
const UNREAD_WRITE_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) fn connect_raw(socket: &Path) -> Result<UnixStream, String> {
    UnixStream::connect(socket).map_err(|error| format!("连 {}：{error}", socket.display()))
}

pub(crate) fn cell_unread_responses_bounded(context: &CellContext) -> Result<String, String> {
    let service_pending = UNREAD_SERVICE_PENDING_BYTES.to_string();
    let per_connection = UNREAD_PER_CONNECTION_BYTES.to_string();
    let write_timeout = UNREAD_WRITE_TIMEOUT.as_millis().to_string();
    let server = HardeningServer::start(
        context,
        context
            .break_switches
            .only(BreakSwitch::ResponseSlotReleasedOnEnqueue),
        &[
            "--maximum-pending-bytes",
            &service_pending,
            "--maximum-pending-bytes-per-connection",
            &per_connection,
            "--write-timeout-milliseconds",
            &write_timeout,
        ],
    )?;
    let mut writer = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    writer.write(&[(
        Table::Conclusions,
        bytes("unread/big"),
        vec![b'z'; UNREAD_VALUE_BYTES],
    )])?;
    let mut slow = connect_raw(&server.socket_path())?;
    for sequence in 1..=UNREAD_READS {
        let mut builder = begin_point_read_frame(sequence, 1, 16);
        builder.key(Table::Conclusions, b"unread/big");
        slow.write_all(&builder.finish())
            .map_err(|error| format!("只发不收的连接发不出点读：{error}"))?;
        thread::sleep(Duration::from_millis(15));
    }
    thread::sleep(Duration::from_millis(300));
    let mut other = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    other
        .write(&[entry("unread/other", "still-served")])
        .map_err(|message| format!("一条连接只发不收时别的客户端写不进：{message}"))?;
    let other_value = other
        .read(&[(Table::Conclusions, bytes("unread/other"))])?
        .into_iter()
        .next()
        .flatten();
    if other_value.as_deref() != Some(b"still-served".as_slice()) {
        return Err(format!(
            "一条连接只发不收时别的客户端读回 {}（该是 still-served）",
            shown_value(other_value.as_deref())
        ));
    }
    let queued_peak = counter(&other.statistics()?, OUTBOUND_QUEUED_BYTES_PEAK_COUNTER)?;
    let queued_bound = u64::try_from(
        UNREAD_PER_CONNECTION_BYTES + UNREAD_VALUE_BYTES + UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION,
    )
    .expect("几 MiB");
    if queued_peak > queued_bound {
        return Err(format!(
            "小请求读 1 MiB 的值、只发不收 {UNREAD_READS} 次：服务写出队峰值 {queued_peak} 字节，超过每条连接的额度 {UNREAD_PER_CONNECTION_BYTES} 加一个回应加不带名额的回应上限（{queued_bound}）：回应不受额度管、在队里无界地积"
        ));
    }
    let release_deadline = Instant::now() + UNREAD_WRITE_TIMEOUT + Duration::from_secs(10);
    loop {
        let statistics = other.statistics()?;
        let write_timeouts = counter(&statistics, WRITE_TIMEOUTS_COUNTER)?;
        let pending_bytes = counter(&statistics, PENDING_BYTES_COUNTER)?;
        if write_timeouts >= 1 && pending_bytes <= RELEASED_PENDING_BYTES_SLACK {
            drop(slow);
            return Ok(format!(
                "小请求读 1 MiB 的值、只发不收 {UNREAD_READS} 次：写出队峰值 {queued_peak} 字节（上界 {queued_bound}），别的客户端照常读写；\
                 写期限 {} 秒到了慢连接被关、额度放回到 {pending_bytes} 字节",
                UNREAD_WRITE_TIMEOUT.as_secs()
            ));
        }
        if Instant::now() >= release_deadline {
            return Err(format!(
                "写期限 {} 秒过了：因写堵住关掉的连接 {write_timeouts} 条（该至少 1）、额度里还记着 {pending_bytes} 字节（该至多 {RELEASED_PENDING_BYTES_SLACK}）",
                UNREAD_WRITE_TIMEOUT.as_secs()
            ));
        }
        thread::sleep(Duration::from_millis(200));
    }
}

/// 没收完的帧那一格的数：额度 1024 字节、待处理 1 帧、四条连接各声明 16 MiB 只发 2 MiB、帧读取期限 1.5 秒、连接数上限 8。
const INCOMPLETE_PENDING_BYTES: u64 = 1024;
const INCOMPLETE_DECLARED_BODY_BYTES: u32 = 16 * 1024 * 1024;
const INCOMPLETE_SENT_BYTES: usize = 2 * 1024 * 1024;
const INCOMPLETE_FRAMES: u64 = 4;
const INCOMPLETE_FRAME_READ_TIMEOUT: Duration = Duration::from_millis(1500);
const INCOMPLETE_MAXIMUM_CONNECTIONS: usize = 8;
/// 逐字节慢发的那一帧：声明的帧体字节、每发一个字节隔多久。
const TRICKLED_BODY_BYTES: u32 = 50;
const TRICKLE_INTERVAL: Duration = Duration::from_millis(100);
/// 等服务关掉一条连接的上限。
const SERVER_CLOSE_TIMEOUT: Duration = Duration::from_secs(5);
/// 服务关掉连接之后到连接数减回去之间留的时间。
const CONNECTION_SETTLE_DELAY: Duration = Duration::from_millis(300);

/// 读到流尾（或连接被重置）算服务关了这条连接；读到点还连着回错。
fn expect_closed_by_server(stream: &mut UnixStream, what: &str) -> Result<(), String> {
    stream
        .set_read_timeout(Some(SERVER_CLOSE_TIMEOUT))
        .map_err(|error| format!("设不上读超时：{error}"))?;
    let mut chunk = [0_u8; 4096];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return Ok(()),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::ConnectionReset => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => {
                return Err(format!(
                    "{what}：{} 秒里服务没关这条连接（{error}）",
                    SERVER_CLOSE_TIMEOUT.as_secs()
                ))
            }
        }
    }
}

/// 连接数降下来之前新连接会被当场关掉：换新连接重试取统计，至多 10 秒。
fn statistics_when_admitted(
    route: &RouteAddress,
) -> Result<Vec<crate::protocol::StatisticsCounter>, String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let attempt = SimpleClient::connect(route, BreakSwitches::none())
            .and_then(|mut client| client.statistics());
        match attempt {
            Ok(statistics) => return Ok(statistics),
            Err(message) if Instant::now() >= deadline => {
                return Err(format!("10 秒里取不到统计：{message}"))
            }
            Err(_) => thread::sleep(Duration::from_millis(100)),
        }
    }
}

pub(crate) fn cell_incomplete_frames_bounded(context: &CellContext) -> Result<String, String> {
    let pending_bytes = INCOMPLETE_PENDING_BYTES.to_string();
    let frame_read_timeout = INCOMPLETE_FRAME_READ_TIMEOUT.as_millis().to_string();
    let maximum_connections = INCOMPLETE_MAXIMUM_CONNECTIONS.to_string();
    let server = HardeningServer::start(
        context,
        context
            .break_switches
            .only(BreakSwitch::FrameBodyReadBeforeAdmission),
        &[
            "--maximum-pending-bytes",
            &pending_bytes,
            "--maximum-pending-total",
            "1",
            "--frame-read-timeout-milliseconds",
            &frame_read_timeout,
            "--maximum-connections",
            &maximum_connections,
        ],
    )?;
    let socket = server.socket_path();
    let mut half_frames = Vec::new();
    for _ in 0..INCOMPLETE_FRAMES {
        let mut stream = connect_raw(&socket)?;
        stream
            .write_all(&INCOMPLETE_DECLARED_BODY_BYTES.to_le_bytes())
            .and_then(|()| stream.write_all(&vec![b'x'; INCOMPLETE_SENT_BYTES]))
            .map_err(|error| format!("发半帧出错：{error}"))?;
        half_frames.push(stream);
    }
    let mut trickled = connect_raw(&socket)?;
    let mut trickle_writer = trickled
        .try_clone()
        .map_err(|error| format!("复制不了慢发的连接：{error}"))?;
    let trickle = thread::spawn(move || {
        let mut frame = TRICKLED_BODY_BYTES.to_le_bytes().to_vec();
        frame.extend(std::iter::repeat_n(
            0_u8,
            usize::try_from(TRICKLED_BODY_BYTES).expect("50"),
        ));
        for byte in frame {
            if trickle_writer.write_all(&[byte]).is_err() {
                return;
            }
            thread::sleep(TRICKLE_INTERVAL);
        }
    });
    thread::sleep(INCOMPLETE_FRAME_READ_TIMEOUT + Duration::from_secs(1));
    for stream in &mut half_frames {
        expect_closed_by_server(stream, "声明 16 MiB、只发了 2 MiB 的连接，帧读取期限过后")?;
    }
    expect_closed_by_server(&mut trickled, "逐字节慢发的连接，帧读取期限过后")?;
    // 慢发的线程在服务关了连接之后写出错就收尾。
    let _ = trickle.join();
    // 读线程关了套接字之后才放手连接数，给那几条收尾留一点时间，免得算进下面的上限。
    thread::sleep(CONNECTION_SETTLE_DELAY);
    let mut idle = Vec::new();
    for _ in 0..INCOMPLETE_MAXIMUM_CONNECTIONS {
        idle.push(connect_raw(&socket)?);
    }
    thread::sleep(Duration::from_millis(200));
    let mut refused = connect_raw(&socket)?;
    expect_closed_by_server(&mut refused, "连接数到了上限之后接进来的连接")?;
    drop(idle);
    let statistics = statistics_when_admitted(&server.route())?;
    let receiving_peak = counter(&statistics, RECEIVING_BODY_BYTES_PEAK_COUNTER)?;
    let frame_read_timeouts = counter(&statistics, FRAME_READ_TIMEOUTS_COUNTER)?;
    let connections_refused = counter(&statistics, CONNECTIONS_REFUSED_COUNTER)?;
    let receiving_bound = u64::from(INCOMPLETE_DECLARED_BODY_BYTES) + INCOMPLETE_PENDING_BYTES;
    if receiving_peak > receiving_bound {
        return Err(format!(
            "额度 {INCOMPLETE_PENDING_BYTES} 字节、{INCOMPLETE_FRAMES} 条连接各声明 16 MiB 只发 2 MiB：服务给没收完的帧体分配的字节峰值 {receiving_peak}，\
             超过额度加一帧（{receiving_bound}）：帧体在准入之前就照声明的长度分配了"
        ));
    }
    if frame_read_timeouts < INCOMPLETE_FRAMES + 1 || connections_refused == 0 {
        return Err(format!(
            "因帧没在期限里收齐关掉的连接 {frame_read_timeouts} 条（该至少 {}）、因连接数满了拒掉的 {connections_refused} 条（该至少 1）",
            INCOMPLETE_FRAMES + 1
        ));
    }
    let mut client = SimpleClient::connect(&server.route(), BreakSwitches::none())?;
    client
        .write(&[entry("incomplete/after", "served")])
        .map_err(|message| format!("没收完的帧都被关掉之后写不进：{message}"))?;
    let stored = read_value(&server.route(), "incomplete/after")?;
    if stored.as_deref() != Some(b"served".as_slice()) {
        return Err(format!(
            "没收完的帧都被关掉之后写的键读回 {}（该是 served）",
            shown_value(stored.as_deref())
        ));
    }
    Ok(format!(
        "额度 {INCOMPLETE_PENDING_BYTES} 字节、待处理 1 帧：{INCOMPLETE_FRAMES} 条半帧连接，帧体分配峰值 {receiving_peak} 字节（上界 {receiving_bound}）；\
         半帧与逐字节慢发的连接在 {} 毫秒的期限后被关（{frame_read_timeouts} 条）；连接数满了拒掉 {connections_refused} 条；之后照常读写",
        INCOMPLETE_FRAME_READ_TIMEOUT.as_millis()
    ))
}
