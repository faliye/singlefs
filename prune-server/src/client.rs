//! 流水客户端：一条连接拆成发送端与接收端，可以放在两个线程里。发送端发帧不等回应，发之前把序号记进「在等」的集合；
//! 接收端按回应里的序号从集合里认领，认不到（服务回了没发过或已经回过的序号）是错。
//! 弄坏开关 `response-sequence-unchecked` 打开时接收端不核序号，把回应配给最早发出、还在等的请求。
//!
//! 连上之后头一帧发报到（主机名、任务号、第几次、进程号，`protocol` 文件头）。
//!
//! 建连有期限（`ConnectDeadline`）：地址解析、连接、报到每一步阻塞都只用到期限剩下的时间，每 `CONNECT_POLL_SLICE` 看一次调用方停不停。
//! 连接用非阻塞的 `connect` 加 `poll`：unix socket 的连接队列满了当场回「队列满」算这一次没连上（阻塞的写法会一直等到服务 accept），
//! TCP 握手迟迟不完成的等到期限为止；报到前设写期限。容错客户端的期限是这一轮重连的总时长上限，一问一答的客户端是 `SIMPLE_CLIENT_TIMEOUT`。
//! 弄坏开关 connect-without-deadline 退回阻塞的 `connect` 与不设写期限的报到。
//!
//! 单条超限当场报：`ConnectionSender::send_frame` 拼好帧先核帧体长度，超过 `MAXIMUM_FRAME_BODY_BYTES` 的不发、不记「在等」，
//! 当场回 `InvalidInput`（说帧体多大、上限多少）；一问一答的客户端照它回错，不重试。服务回「帧长度不对」（序号 0）的照
//! `ReceiveFailure::ConnectionRejected` 回，同样不重试。自证造坏帧长走 `write_raw_bytes`，不经这道核。

use std::collections::BTreeSet;
use std::io::{self, BufReader, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::connections::announcement_from_environment;
use crate::protocol::{
    begin_point_read_frame, begin_write_frame, decode_response_body, encode_announce_stop_frame,
    encode_claim_frame, encode_connected_clients_frame, encode_connection_announcement_frame,
    encode_delete_prefix_frame, encode_prefix_scan_frame, encode_statistics_frame,
    is_frame_length_acceptable, ClaimOutcome, ClientAnnouncement, ConnectedClient, Response,
    ResponseOutcome, ScanEntry, StatisticsCounter, Table, CONNECTION_LEVEL_SEQUENCE,
    FRAME_LENGTH_FIELD_BYTES, MAXIMUM_FRAME_BODY_BYTES, RESPONSE_HEADER_BYTES,
};

/// 连哪一处：本机走 unix socket，第二台走 TCP。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteAddress {
    UnixSocket(PathBuf),
    Network(String),
}

impl RouteAddress {
    pub fn label(&self) -> &'static str {
        match self {
            RouteAddress::UnixSocket(_) => "unix",
            RouteAddress::Network(_) => "tcp",
        }
    }
}

pub(crate) enum ClientStream {
    Unix(UnixStream),
    Network(TcpStream),
}

/// 建连等就绪、等地址解析时一次最多等这么久，再回头看调用方停不停、到没到期限。
const CONNECT_POLL_SLICE: Duration = Duration::from_millis(50);

/// 建连与报到的期限（文件头）：每一步阻塞只用到 `deadline` 剩下的时间；`stop_check` 回真就不再等。
pub(crate) struct ConnectDeadline<'stop> {
    pub(crate) deadline: Instant,
    pub(crate) stop_check: &'stop dyn Fn() -> bool,
    /// 弄坏开关 connect-without-deadline：阻塞的 `connect`、报到不设写期限。
    pub(crate) is_unbounded: bool,
}

impl ConnectDeadline<'_> {
    /// 这一步还能等多久；调用方要停或到了期限回错（说明里带 `step`）。
    fn remaining_for(&self, step: &str) -> Result<Duration, String> {
        if (self.stop_check)() {
            return Err(format!("{step}：调用方要停"));
        }
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(format!("{step}：到了建连期限"));
        }
        Ok(remaining)
    }
}

/// 一次 `poll` 等的毫秒数：不超过 `CONNECT_POLL_SLICE` 与剩下的时间，至少 1 毫秒。
fn poll_milliseconds(remaining: Duration) -> libc::c_int {
    let slice = remaining
        .min(CONNECT_POLL_SLICE)
        .max(Duration::from_millis(1));
    libc::c_int::try_from(slice.as_millis()).expect("至多 CONNECT_POLL_SLICE 那么多毫秒")
}

/// `connect` 用的套接字地址。
enum RawSocketAddress {
    InternetVersion4(libc::sockaddr_in),
    InternetVersion6(libc::sockaddr_in6),
    UnixPath(libc::sockaddr_un),
}

fn address_family(family: libc::c_int) -> libc::sa_family_t {
    libc::sa_family_t::try_from(family).expect("地址族常量放得进 sa_family_t")
}

fn socket_address_length<Address>() -> libc::socklen_t {
    libc::socklen_t::try_from(std::mem::size_of::<Address>()).expect("套接字地址结构几十字节")
}

impl RawSocketAddress {
    fn from_network(address: SocketAddr) -> RawSocketAddress {
        match address {
            SocketAddr::V4(address) => {
                // SAFETY: sockaddr_in 是只含整数与字节数组的 C 结构，全零是合法值；下面逐个填要用的字段。
                let mut raw: libc::sockaddr_in = unsafe { std::mem::zeroed() };
                raw.sin_family = address_family(libc::AF_INET);
                raw.sin_port = address.port().to_be();
                raw.sin_addr.s_addr = u32::from_ne_bytes(address.ip().octets());
                RawSocketAddress::InternetVersion4(raw)
            }
            SocketAddr::V6(address) => {
                // SAFETY: sockaddr_in6 同上，全零合法。
                let mut raw: libc::sockaddr_in6 = unsafe { std::mem::zeroed() };
                raw.sin6_family = address_family(libc::AF_INET6);
                raw.sin6_port = address.port().to_be();
                raw.sin6_flowinfo = address.flowinfo();
                raw.sin6_addr.s6_addr = address.ip().octets();
                raw.sin6_scope_id = address.scope_id();
                RawSocketAddress::InternetVersion6(raw)
            }
        }
    }

    fn from_unix_path(path: &Path) -> Result<RawSocketAddress, String> {
        // SAFETY: sockaddr_un 同上，全零合法；路径末尾的 0 由全零留着。
        let mut raw: libc::sockaddr_un = unsafe { std::mem::zeroed() };
        let path_bytes = path.as_os_str().as_bytes();
        if path_bytes.len() >= raw.sun_path.len() {
            return Err(format!(
                "socket 路径 {} 有 {} 字节，放不进 sun_path（至多 {} 字节）",
                path.display(),
                path_bytes.len(),
                raw.sun_path.len() - 1
            ));
        }
        raw.sun_family = address_family(libc::AF_UNIX);
        for (slot, byte) in raw.sun_path.iter_mut().zip(path_bytes) {
            *slot = libc::c_char::from_ne_bytes([*byte]);
        }
        Ok(RawSocketAddress::UnixPath(raw))
    }

    fn family(&self) -> libc::c_int {
        match self {
            RawSocketAddress::InternetVersion4(_) => libc::AF_INET,
            RawSocketAddress::InternetVersion6(_) => libc::AF_INET6,
            RawSocketAddress::UnixPath(_) => libc::AF_UNIX,
        }
    }

    /// 对 `socket` 发起连接；回 `connect` 的返回值（0 或 -1，错误在 errno 里）。
    fn connect(&self, socket: &OwnedFd) -> libc::c_int {
        let descriptor = socket.as_raw_fd();
        // SAFETY: 描述符来自活着的 OwnedFd；地址指针指向 self 里对应的那个结构，长度是它的字节数，connect 只读它。
        unsafe {
            match self {
                RawSocketAddress::InternetVersion4(raw) => libc::connect(
                    descriptor,
                    std::ptr::addr_of!(*raw).cast(),
                    socket_address_length::<libc::sockaddr_in>(),
                ),
                RawSocketAddress::InternetVersion6(raw) => libc::connect(
                    descriptor,
                    std::ptr::addr_of!(*raw).cast(),
                    socket_address_length::<libc::sockaddr_in6>(),
                ),
                RawSocketAddress::UnixPath(raw) => libc::connect(
                    descriptor,
                    std::ptr::addr_of!(*raw).cast(),
                    socket_address_length::<libc::sockaddr_un>(),
                ),
            }
        }
    }
}

/// 非阻塞地连 `address`，等到连上、出错或到期限；连上之后把描述符换回阻塞的交回。
fn connect_before_deadline(
    address: &RawSocketAddress,
    shown: &str,
    limit: &ConnectDeadline<'_>,
) -> Result<OwnedFd, String> {
    limit.remaining_for(&format!("连 {shown}"))?;
    // SAFETY: socket 只收三个整数，回新描述符或 -1。
    let descriptor = unsafe {
        libc::socket(
            address.family(),
            libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
            0,
        )
    };
    if descriptor < 0 {
        return Err(format!(
            "连 {shown}：建不了套接字：{}",
            io::Error::last_os_error()
        ));
    }
    // SAFETY: descriptor 是上面刚建、没有别人持有的描述符，交给 OwnedFd 管，放手时关掉。
    let socket = unsafe { OwnedFd::from_raw_fd(descriptor) };
    if address.connect(&socket) != 0 {
        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::EINPROGRESS) => wait_until_connected(&socket, shown, limit)?,
            Some(libc::EAGAIN) => {
                return Err(format!(
                    "连 {shown}：服务的连接队列满了（{error}），这一次算没连上"
                ))
            }
            Some(_) | None => return Err(format!("连 {shown}：{error}")),
        }
    }
    let raw = socket.as_raw_fd();
    // SAFETY: fcntl 对活着的描述符读、写文件状态标志，只收整数。
    let flags = unsafe { libc::fcntl(raw, libc::F_GETFL) };
    if flags < 0 {
        return Err(format!(
            "连 {shown}：读不出描述符的状态标志：{}",
            io::Error::last_os_error()
        ));
    }
    // SAFETY: 同上，只把非阻塞那一位清掉写回去。
    let changed = unsafe { libc::fcntl(raw, libc::F_SETFL, flags & !libc::O_NONBLOCK) };
    if changed != 0 {
        return Err(format!(
            "连 {shown}：换不回阻塞模式：{}",
            io::Error::last_os_error()
        ));
    }
    Ok(socket)
}

/// 服务起来之前探 socket 上有没有别的服务在听：非阻塞地连一次，连上、握手进行中、连接队列满了都算有（服务卡住不 accept、
/// 队列满了的也是有服务占着），别的错（没人听、文件不是 socket）算没有。不阻塞：阻塞的 `connect` 碰上队列满会一直等。
pub(crate) fn unix_socket_has_listener(path: &Path) -> bool {
    let Ok(address) = RawSocketAddress::from_unix_path(path) else {
        return false;
    };
    // SAFETY: socket 只收三个整数，回新描述符或 -1。
    let descriptor = unsafe {
        libc::socket(
            libc::AF_UNIX,
            libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
            0,
        )
    };
    if descriptor < 0 {
        return false;
    }
    // SAFETY: descriptor 是上面刚建、没有别人持有的描述符，交给 OwnedFd 管，放手时关掉。
    let socket = unsafe { OwnedFd::from_raw_fd(descriptor) };
    if address.connect(&socket) == 0 {
        return true;
    }
    matches!(
        io::Error::last_os_error().raw_os_error(),
        Some(libc::EINPROGRESS | libc::EAGAIN)
    )
}

/// 非阻塞连接发起之后等它可写（连上或出错），每次至多等 `CONNECT_POLL_SLICE`，其间看调用方停不停、到没到期限。
fn wait_until_connected(
    socket: &OwnedFd,
    shown: &str,
    limit: &ConnectDeadline<'_>,
) -> Result<(), String> {
    loop {
        let remaining = limit.remaining_for(&format!("连 {shown}，握手还没完成"))?;
        let mut waited = libc::pollfd {
            fd: socket.as_raw_fd(),
            events: libc::POLLOUT,
            revents: 0,
        };
        // SAFETY: 指针指向栈上这一个 pollfd，个数给 1，poll 只在里面写 revents。
        let ready = unsafe {
            libc::poll(
                std::ptr::addr_of_mut!(waited),
                1,
                poll_milliseconds(remaining),
            )
        };
        if ready < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(format!("连 {shown}：等握手出错：{error}"));
        }
        if ready == 0 {
            continue;
        }
        let mut socket_error: libc::c_int = 0;
        let mut length = socket_address_length::<libc::c_int>();
        // SAFETY: 两个指针指向栈上的整数，长度是它的字节数，getsockopt 只往里写。
        let result = unsafe {
            libc::getsockopt(
                socket.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                std::ptr::addr_of_mut!(socket_error).cast(),
                std::ptr::addr_of_mut!(length),
            )
        };
        if result != 0 {
            return Err(format!(
                "连 {shown}：读不出握手结果：{}",
                io::Error::last_os_error()
            ));
        }
        if socket_error != 0 {
            return Err(format!(
                "连 {shown}：{}",
                io::Error::from_raw_os_error(socket_error)
            ));
        }
        return Ok(());
    }
}

/// 解析 TCP 地址：写成「地址:端口」的直接用；要查名字的放在一个线程里查，等的时候照期限与停不停。
fn resolve_before_deadline(
    address: &str,
    limit: &ConnectDeadline<'_>,
) -> Result<Vec<SocketAddr>, String> {
    if let Ok(parsed) = address.parse::<SocketAddr>() {
        return Ok(vec![parsed]);
    }
    let (sender, receiver) = mpsc::channel();
    let owned = address.to_string();
    thread::spawn(move || {
        let resolved = owned
            .to_socket_addrs()
            .map(|addresses| addresses.collect::<Vec<SocketAddr>>());
        // 等的一方到点先走了的，结果没人收，丢掉。
        let _ = sender.send(resolved);
    });
    loop {
        let remaining = limit.remaining_for(&format!("解析 {address}"))?;
        match receiver.recv_timeout(remaining.min(CONNECT_POLL_SLICE)) {
            Ok(Ok(addresses)) if !addresses.is_empty() => return Ok(addresses),
            Ok(Ok(_)) => return Err(format!("解析 {address}：一个地址都没有")),
            Ok(Err(error)) => return Err(format!("解析 {address}：{error}")),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(format!("解析 {address}：查名字的线程没回话就退了"))
            }
        }
    }
}

fn connect_network_before_deadline(
    address: &str,
    limit: &ConnectDeadline<'_>,
) -> Result<TcpStream, String> {
    let mut last_error = String::new();
    for candidate in resolve_before_deadline(address, limit)? {
        match connect_before_deadline(
            &RawSocketAddress::from_network(candidate),
            &candidate.to_string(),
            limit,
        ) {
            Ok(socket) => return Ok(TcpStream::from(socket)),
            Err(message) => last_error = message,
        }
    }
    Err(last_error)
}

/// 连上、发报到；回这条连接。`tune_network` 给 TCP 连接设 keepalive 这类选项。
/// 每一步阻塞只用到 `limit` 剩下的时间（文件头）；报到写完之后写期限清掉，由调用方自己设。
pub(crate) fn open_announced_stream(
    route: &RouteAddress,
    announcement: &ClientAnnouncement,
    limit: &ConnectDeadline<'_>,
    tune_network: impl FnOnce(&TcpStream) -> Result<(), String>,
) -> Result<ClientStream, String> {
    let mut stream = match route {
        RouteAddress::UnixSocket(path) if limit.is_unbounded => ClientStream::Unix(
            UnixStream::connect(path)
                .map_err(|error| format!("连 {} ：{error}", path.display()))?,
        ),
        RouteAddress::UnixSocket(path) => {
            let shown = path.display().to_string();
            let socket =
                connect_before_deadline(&RawSocketAddress::from_unix_path(path)?, &shown, limit)?;
            ClientStream::Unix(UnixStream::from(socket))
        }
        RouteAddress::Network(address) => {
            let stream = if limit.is_unbounded {
                TcpStream::connect(address).map_err(|error| format!("连 {address}：{error}"))?
            } else {
                connect_network_before_deadline(address, limit)?
            };
            stream
                .set_nodelay(true)
                .map_err(|error| format!("设不上 TCP_NODELAY：{error}"))?;
            tune_network(&stream)?;
            ClientStream::Network(stream)
        }
    };
    if !limit.is_unbounded {
        let remaining = limit.remaining_for("发报到")?;
        stream
            .set_write_timeout(Some(remaining))
            .map_err(|error| format!("设不上报到的写期限：{error}"))?;
    }
    stream
        .write_all(&encode_connection_announcement_frame(announcement))
        .map_err(|error| format!("发报到出错：{error}"))?;
    stream
        .set_write_timeout(None)
        .map_err(|error| format!("清不掉报到的写期限：{error}"))?;
    Ok(stream)
}

impl ClientStream {
    pub(crate) fn try_clone(&self) -> io::Result<ClientStream> {
        match self {
            ClientStream::Unix(stream) => stream.try_clone().map(ClientStream::Unix),
            ClientStream::Network(stream) => stream.try_clone().map(ClientStream::Network),
        }
    }

    pub(crate) fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        match self {
            ClientStream::Unix(stream) => stream.set_read_timeout(timeout),
            ClientStream::Network(stream) => stream.set_read_timeout(timeout),
        }
    }

    pub(crate) fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        match self {
            ClientStream::Unix(stream) => stream.set_write_timeout(timeout),
            ClientStream::Network(stream) => stream.set_write_timeout(timeout),
        }
    }

    /// TCP 连接关的时候直接回 RST，不走 FIN（故障注入代理在服务死了时用：对端的写立刻出错）；unix socket 不用设。
    pub(crate) fn reset_on_close(&self) -> io::Result<()> {
        match self {
            ClientStream::Unix(_) => Ok(()),
            ClientStream::Network(stream) => {
                let linger = libc::linger {
                    l_onoff: 1,
                    l_linger: 0,
                };
                let linger_bytes = libc::socklen_t::try_from(std::mem::size_of::<libc::linger>())
                    .expect("linger 结构几个字节，放得进 socklen_t");
                // SAFETY: fd 是这条还开着的 TcpStream 的；指针指向栈上这个 linger，长度是它的字节数，setsockopt 只读它。
                let result = unsafe {
                    libc::setsockopt(
                        stream.as_raw_fd(),
                        libc::SOL_SOCKET,
                        libc::SO_LINGER,
                        std::ptr::addr_of!(linger).cast(),
                        linger_bytes,
                    )
                };
                if result != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            }
        }
    }

    pub(crate) fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match self {
            ClientStream::Unix(stream) => stream.shutdown(how),
            ClientStream::Network(stream) => stream.shutdown(how),
        }
    }
}

impl Read for ClientStream {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        match self {
            ClientStream::Unix(stream) => stream.read(destination),
            ClientStream::Network(stream) => stream.read(destination),
        }
    }
}

impl Write for ClientStream {
    fn write(&mut self, source: &[u8]) -> io::Result<usize> {
        match self {
            ClientStream::Unix(stream) => stream.write(source),
            ClientStream::Network(stream) => stream.write(source),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            ClientStream::Unix(stream) => stream.flush(),
            ClientStream::Network(stream) => stream.flush(),
        }
    }
}

type PendingSequences = Arc<Mutex<BTreeSet<u64>>>;

pub struct ConnectionSender {
    stream: ClientStream,
    next_sequence: u64,
    pending: PendingSequences,
    /// 弄坏开关 oversize-frame-sent-and-retried：`send_frame` 不核帧长、照发。
    break_switches: BreakSwitches,
}

pub struct ConnectionReceiver {
    reader: BufReader<ClientStream>,
    pending: PendingSequences,
    break_switches: BreakSwitches,
}

/// 一个回应与它认领到的请求序号（平常两者相同）。
#[derive(Debug)]
pub struct MatchedResponse {
    pub request_sequence: u64,
    pub response: Response,
}

#[derive(Debug)]
pub enum ReceiveFailure {
    /// 在帧边界上读到流尾：服务关了连接。
    EndOfStream,
    /// 读超时，而且这一帧一个字节都还没到。
    TimedOut,
    ReadFailed(io::Error),
    Malformed(String),
    /// 服务回了一个不在「在等」集合里的序号。
    UnexpectedSequence(u64),
    /// 服务对整条连接的拒绝（序号 0）：帧长度不对。
    ConnectionRejected(String),
    /// 服务说它停了（序号 0 的状态 6）：不要再重试。
    ServerStopping(String),
}

impl std::fmt::Display for ReceiveFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiveFailure::EndOfStream => write!(formatter, "服务关了连接"),
            ReceiveFailure::TimedOut => write!(formatter, "等回应超时"),
            ReceiveFailure::ReadFailed(error) => write!(formatter, "读回应出错：{error}"),
            ReceiveFailure::Malformed(reason) => write!(formatter, "回应解不开：{reason}"),
            ReceiveFailure::UnexpectedSequence(sequence) => {
                write!(formatter, "回应的序号 {sequence} 不是在等的请求")
            }
            ReceiveFailure::ConnectionRejected(message) => {
                write!(formatter, "服务拒了整条连接：{message}")
            }
            ReceiveFailure::ServerStopping(message) => write!(formatter, "服务停了：{message}"),
        }
    }
}

/// 连上、报到（身份取这个进程的环境）。
pub fn connect(
    route: &RouteAddress,
    break_switches: BreakSwitches,
) -> Result<(ConnectionSender, ConnectionReceiver), String> {
    let announcement = announcement_from_environment(&break_switches);
    connect_announcing(route, break_switches, &announcement)
}

/// 连上、按给的身份报到。
pub fn connect_announcing(
    route: &RouteAddress,
    break_switches: BreakSwitches,
    announcement: &ClientAnnouncement,
) -> Result<(ConnectionSender, ConnectionReceiver), String> {
    let never_stop = || false;
    let limit = ConnectDeadline {
        deadline: Instant::now() + SIMPLE_CLIENT_TIMEOUT,
        stop_check: &never_stop,
        is_unbounded: break_switches.is_enabled(BreakSwitch::ConnectWithoutDeadline),
    };
    let stream = open_announced_stream(route, announcement, &limit, |_| Ok(()))?;
    let reading_stream = stream
        .try_clone()
        .map_err(|error| format!("复制不了连接：{error}"))?;
    let pending: PendingSequences = Arc::new(Mutex::new(BTreeSet::new()));
    let sender = ConnectionSender {
        stream,
        next_sequence: 1,
        pending: Arc::clone(&pending),
        break_switches: break_switches.clone(),
    };
    let receiver = ConnectionReceiver {
        reader: BufReader::with_capacity(1024 * 1024, reading_stream),
        pending,
        break_switches,
    };
    Ok((sender, receiver))
}

impl ConnectionSender {
    /// 取下一个序号、照它拼帧、记进「在等」、发出去；回这一帧的序号。帧体超过帧长上限的不发、回 `InvalidInput`（文件头「单条超限当场报」）。
    pub fn send_frame(&mut self, build_frame: impl FnOnce(u64) -> Vec<u8>) -> io::Result<u64> {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        let frame = build_frame(sequence);
        let body_bytes = frame.len().saturating_sub(FRAME_LENGTH_FIELD_BYTES);
        if body_bytes > usize::try_from(MAXIMUM_FRAME_BODY_BYTES).expect("u32 放得进 usize")
            && !self
                .break_switches
                .is_enabled(BreakSwitch::OversizeFrameSentAndRetried)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "帧体 {body_bytes} 字节，超过帧长上限 {MAXIMUM_FRAME_BODY_BYTES} 字节；没发（服务会拒整条连接，重发照样拒；把条目拆成几帧或缩短键）"
                ),
            ));
        }
        self.pending
            .lock()
            .expect("在等集合的锁只在插入与认领时短暂持有")
            .insert(sequence);
        self.stream.write_all(&frame)?;
        Ok(sequence)
    }

    /// 照原样写几个字节，不记序号：自证造坏的帧长度用。
    pub fn write_raw_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stream.write_all(bytes)
    }

    /// 关掉写的一半：服务读到流尾，处理完还在途的帧之后关连接。
    pub fn close_writing(&self) {
        // 连接已经断了的，关不上不要紧。
        let _ = self.stream.shutdown(Shutdown::Write);
    }

    /// 还在等回应的帧数。
    pub fn outstanding_frames(&self) -> usize {
        self.pending
            .lock()
            .expect("在等集合的锁只在插入与认领时短暂持有")
            .len()
    }
}

/// 读满 `destination`；`frame_started` 为假且一个字节都没读到时，读超时回 `TimedOut`，否则超时了接着等（帧已经开头，半截丢下会错位）。
fn read_exact_resuming(
    reader: &mut impl Read,
    destination: &mut [u8],
    frame_started: bool,
) -> Result<(), ReceiveFailure> {
    let mut filled = 0;
    while filled < destination.len() {
        match reader.read(&mut destination[filled..]) {
            Ok(0) => {
                if filled == 0 && !frame_started {
                    return Err(ReceiveFailure::EndOfStream);
                }
                return Err(ReceiveFailure::ReadFailed(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "回应读到一半流断了",
                )));
            }
            Ok(read_now) => filled += read_now,
            Err(error)
                if error.kind() == io::ErrorKind::WouldBlock
                    || error.kind() == io::ErrorKind::TimedOut =>
            {
                if filled == 0 && !frame_started {
                    return Err(ReceiveFailure::TimedOut);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(ReceiveFailure::ReadFailed(error)),
        }
    }
    Ok(())
}

impl ConnectionReceiver {
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.reader.get_ref().set_read_timeout(timeout)
    }

    /// 读一帧回应、解开，不认领序号。
    pub fn receive_unmatched(&mut self) -> Result<Response, ReceiveFailure> {
        let mut length_field = [0_u8; FRAME_LENGTH_FIELD_BYTES];
        read_exact_resuming(&mut self.reader, &mut length_field, false)?;
        let declared_body_bytes = u32::from_le_bytes(length_field);
        if !is_frame_length_acceptable(declared_body_bytes, RESPONSE_HEADER_BYTES) {
            return Err(ReceiveFailure::Malformed(format!(
                "回应帧体长度 {declared_body_bytes} 不对"
            )));
        }
        let mut body = vec![
            0_u8;
            usize::try_from(declared_body_bytes)
                .expect("这个工程只编 64 位目标，u32 放得进 usize")
        ];
        read_exact_resuming(&mut self.reader, &mut body, true)?;
        decode_response_body(&body).map_err(ReceiveFailure::Malformed)
    }

    /// 读一帧回应并认领它的请求序号。
    pub fn receive(&mut self) -> Result<MatchedResponse, ReceiveFailure> {
        let response = self.receive_unmatched()?;
        if response.sequence == CONNECTION_LEVEL_SEQUENCE {
            return Err(match response.outcome {
                ResponseOutcome::ServerStopping { message } => {
                    ReceiveFailure::ServerStopping(message)
                }
                ResponseOutcome::Rejected { message, .. } => {
                    ReceiveFailure::ConnectionRejected(message)
                }
                other @ (ResponseOutcome::PointValues(_)
                | ResponseOutcome::WriteAcknowledged { .. }
                | ResponseOutcome::ScanEntries { .. }
                | ResponseOutcome::Statistics(_)
                | ResponseOutcome::Claims(_)
                | ResponseOutcome::PrefixDeleted { .. }
                | ResponseOutcome::ConnectedClients(_)
                | ResponseOutcome::Busy { .. }) => {
                    ReceiveFailure::ConnectionRejected(format!("序号 0 的回应不是拒绝：{other:?}"))
                }
            });
        }
        let mut pending = self
            .pending
            .lock()
            .expect("在等集合的锁只在插入与认领时短暂持有");
        if self
            .break_switches
            .is_enabled(BreakSwitch::ResponseSequenceUnchecked)
        {
            let oldest = pending
                .pop_first()
                .ok_or(ReceiveFailure::UnexpectedSequence(response.sequence))?;
            return Ok(MatchedResponse {
                request_sequence: oldest,
                response,
            });
        }
        if !pending.remove(&response.sequence) {
            return Err(ReceiveFailure::UnexpectedSequence(response.sequence));
        }
        Ok(MatchedResponse {
            request_sequence: response.sequence,
            response,
        })
    }
}

/// 客户端身份：16 个随机字节写成 32 位十六进制。两台、多个进程各自生成，撞上的概率可以不计；
/// 带追加日志的客户端把它记进日志，进程重起之后照用同一个身份，服务的去重才对得上。
pub fn random_client_identity() -> Result<Vec<u8>, String> {
    let mut random_bytes = [0_u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut random_bytes))
        .map_err(|error| format!("读不了 /dev/urandom 生成客户端身份：{error}"))?;
    Ok(random_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
        .into_bytes())
}

/// 删前缀一帧的结局（`SimpleClient::delete_prefix_frame`）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PrefixDeletionFrame {
    pub keys_in_frame: u32,
    pub is_exhausted: bool,
    pub last_key: Vec<u8>,
}

/// 一问一答的客户端：发一帧、等它的回应。不重连、不重放、没有追加日志，给自证与只读的轻量调用方用；
/// 要容错的写走 `reliable_client::ReliableClient`。
pub struct SimpleClient {
    pub sender: ConnectionSender,
    pub receiver: ConnectionReceiver,
    pub client_identity: Vec<u8>,
    next_request_number: u64,
}

/// 一问一答等回应的上限，也是它建连与报到的期限。
pub const SIMPLE_CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

impl SimpleClient {
    pub fn connect(
        route: &RouteAddress,
        break_switches: BreakSwitches,
    ) -> Result<SimpleClient, String> {
        let announcement = announcement_from_environment(&break_switches);
        SimpleClient::connect_announcing(route, break_switches, &announcement)
    }

    /// 按给的身份报到（自证造带任务号的客户端用）。
    pub fn connect_announcing(
        route: &RouteAddress,
        break_switches: BreakSwitches,
        announcement: &ClientAnnouncement,
    ) -> Result<SimpleClient, String> {
        let (sender, receiver) = connect_announcing(route, break_switches, announcement)
            .map_err(|error| format!("连不上 {route:?}：{error}"))?;
        receiver
            .set_read_timeout(Some(SIMPLE_CLIENT_TIMEOUT))
            .map_err(|error| format!("设不上读超时：{error}"))?;
        Ok(SimpleClient {
            sender,
            receiver,
            client_identity: random_client_identity()?,
            next_request_number: 1,
        })
    }

    /// 一帧认领。
    pub fn claim(
        &mut self,
        keys: &[(Table, Vec<u8>)],
        lease_milliseconds: u32,
    ) -> Result<Vec<ClaimOutcome>, String> {
        let borrowed: Vec<(Table, &[u8])> = keys
            .iter()
            .map(|(table, key)| (*table, key.as_slice()))
            .collect();
        let identity = self.client_identity.clone();
        let outcome = self.call(|sequence| {
            encode_claim_frame(sequence, &identity, lease_milliseconds, &borrowed)
        })?;
        let ResponseOutcome::Claims(outcomes) = outcome else {
            return Err(format!("认领回的不是认领结局：{outcome:?}"));
        };
        Ok(outcomes)
    }

    pub fn call(
        &mut self,
        build_frame: impl FnOnce(u64) -> Vec<u8>,
    ) -> Result<ResponseOutcome, String> {
        let sequence = self
            .sender
            .send_frame(build_frame)
            .map_err(|error| format!("发不出去：{error}"))?;
        let matched = self
            .receiver
            .receive()
            .map_err(|failure| failure.to_string())?;
        if matched.request_sequence != sequence {
            return Err(format!(
                "发的是序号 {sequence}，认领到的是 {}",
                matched.request_sequence
            ));
        }
        Ok(matched.response.outcome)
    }

    pub fn write(&mut self, entries: &[(Table, Vec<u8>, Vec<u8>)]) -> Result<u32, String> {
        let entry_count = u32::try_from(entries.len()).map_err(|_| "条数超过 u32".to_string())?;
        let capacity = entries
            .iter()
            .map(|(_, key, value)| 9 + key.len() + value.len())
            .sum();
        let request_number = self.next_request_number;
        self.next_request_number += 1;
        let identity = self.client_identity.clone();
        let outcome = self.call(|sequence| {
            let mut builder =
                begin_write_frame(sequence, &identity, request_number, entry_count, capacity);
            for (table, key, value) in entries {
                builder.entry(*table, key, value);
            }
            builder.finish()
        })?;
        let ResponseOutcome::WriteAcknowledged {
            entries_written, ..
        } = outcome
        else {
            return Err(format!("成批写回的不是写确认：{outcome:?}"));
        };
        Ok(entries_written)
    }

    pub fn read(&mut self, keys: &[(Table, Vec<u8>)]) -> Result<Vec<Option<Vec<u8>>>, String> {
        let key_count = u32::try_from(keys.len()).map_err(|_| "键数超过 u32".to_string())?;
        let capacity = keys.iter().map(|(_, key)| 5 + key.len()).sum();
        let outcome = self.call(|sequence| {
            let mut builder = begin_point_read_frame(sequence, key_count, capacity);
            for (table, key) in keys {
                builder.key(*table, key);
            }
            builder.finish()
        })?;
        let ResponseOutcome::PointValues(values) = outcome else {
            return Err(format!("成批点读回的不是值：{outcome:?}"));
        };
        Ok(values)
    }

    pub fn scan(
        &mut self,
        table: Table,
        prefix: &[u8],
        maximum_entries: u32,
    ) -> Result<(bool, Vec<ScanEntry>), String> {
        let outcome = self
            .call(|sequence| encode_prefix_scan_frame(sequence, table, prefix, maximum_entries))?;
        let ResponseOutcome::ScanEntries {
            is_truncated,
            entries,
        } = outcome
        else {
            return Err(format!("按前缀扫回的不是键值：{outcome:?}"));
        };
        Ok((is_truncated, entries))
    }

    /// 删前缀一帧（`protocol.rs` 文件头「删前缀」）：`after_key` 空是从前缀起；`count_only` 为真只数不删。
    pub fn delete_prefix_frame(
        &mut self,
        prefix: &[u8],
        after_key: &[u8],
        maximum_keys: u32,
        count_only: bool,
    ) -> Result<PrefixDeletionFrame, String> {
        let outcome = self.call(|sequence| {
            encode_delete_prefix_frame(sequence, prefix, after_key, maximum_keys, count_only)
        })?;
        match outcome {
            ResponseOutcome::PrefixDeleted {
                keys_in_frame,
                is_exhausted,
                last_key,
            } => Ok(PrefixDeletionFrame {
                keys_in_frame,
                is_exhausted,
                last_key,
            }),
            other @ (ResponseOutcome::PointValues(_)
            | ResponseOutcome::WriteAcknowledged { .. }
            | ResponseOutcome::ScanEntries { .. }
            | ResponseOutcome::Statistics(_)
            | ResponseOutcome::Claims(_)
            | ResponseOutcome::ConnectedClients(_)
            | ResponseOutcome::ServerStopping { .. }
            | ResponseOutcome::Busy { .. }
            | ResponseOutcome::Rejected { .. }) => {
                Err(format!("删前缀回的不是删前缀的结局：{other:?}"))
            }
        }
    }

    /// 数或删 `prefix` 下的全部键：一帧一帧接着来（每帧续扫起点取上一帧回的最后一把键），直到前缀下没有更多的键；
    /// 回数到或删掉的总数。`count_only` 为真只数不删（干跑）。
    pub fn delete_prefix(
        &mut self,
        prefix: &[u8],
        maximum_keys_per_frame: u32,
        count_only: bool,
    ) -> Result<u64, String> {
        let mut total = 0_u64;
        let mut after_key = Vec::new();
        loop {
            let frame =
                self.delete_prefix_frame(prefix, &after_key, maximum_keys_per_frame, count_only)?;
            total += u64::from(frame.keys_in_frame);
            if frame.is_exhausted {
                return Ok(total);
            }
            if frame.last_key.is_empty() {
                return Err("删前缀说前缀下还有键，却没回最后一把键：续不下去".to_string());
            }
            after_key = frame.last_key;
        }
    }

    pub fn statistics(&mut self) -> Result<Vec<StatisticsCounter>, String> {
        let outcome = self.call(encode_statistics_frame)?;
        let ResponseOutcome::Statistics(counters) = outcome else {
            return Err(format!("统计回的不是计数：{outcome:?}"));
        };
        Ok(counters)
    }

    /// 这条连接之外连着的客户端（只认 unix socket）。
    pub fn connected_clients(&mut self) -> Result<Vec<ConnectedClient>, String> {
        let outcome = self.call(encode_connected_clients_frame)?;
        let ResponseOutcome::ConnectedClients(clients) = outcome else {
            return Err(format!("列连着的客户端回的不是清单：{outcome:?}"));
        };
        Ok(clients)
    }

    /// 宣布停服务：服务给别的连接发「服务停了」，回发通知之前连着的客户端（只认 unix socket）。
    pub fn announce_stop(&mut self) -> Result<Vec<ConnectedClient>, String> {
        let outcome = self.call(encode_announce_stop_frame)?;
        let ResponseOutcome::ConnectedClients(clients) = outcome else {
            return Err(format!("宣布停服务回的不是客户端清单：{outcome:?}"));
        };
        Ok(clients)
    }
}
