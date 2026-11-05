//! 故障注入代理：夹在客户端与服务之间（unix socket 或 TCP 回环），按帧转发，按控制项注入网络故障。
//! 自证的容错各格与压测器的 fault-injection 负载用它；只在本进程里起，跑完停掉。
//!
//! 控制项（`ProxyControl`，跑的过程中随时改）：
//! - 断在一帧中间：下一帧客户端到服务的请求只转一半就把两头都关掉（服务读到半帧、整帧丢掉）；
//! - 吞掉回应再断：下一帧请求整帧转过去，之后服务回的都不转，过一会两头关掉（服务做了、客户端收不到确认）；
//! - 半开：接下来的一条连接转满若干帧之后两个方向都不再转、也不关（对端看来连接还在，就是没有回应）；
//! - 帧重复：每第 N 帧请求留一份，等下一帧转过去之后再把它转一遍（迟到的重复，会落在后一帧之后）；
//! - 延迟与抖动：两个方向每转一帧之前睡一个 [下限, 上限] 毫秒之间的随机时长。
//!
//! 计数（接了几条连接、转了几帧、注入了几次重复、断了几次）给调用方核故障真的注入了，不是空过。
//!
//! - 停读：接下来的一条连接一个字节都不读客户端的、也不关（对端看来服务不收了，写满发送缓冲就堵住；给客户端的写超时那一格用）；
//!
//! 序号 0 的连接层帧（报到）照转，不数、不往它们身上注入。
//!
//! 服务那头没了（读到流尾、写出错，例如服务被 `kill -9`）时把客户端那头真关掉：TCP 先设成关时回 RST，两头 shutdown，
//! 再放掉代理留着的这条连接的复制（转发线程退出时放掉它们手里的），fd 一个不留。客户端照直连时服务死了那样看到对端关了——
//! 读到流尾、写出错——不会往一个没人读的连接里一直写。
//! 弄坏开关 fault-proxy-half-closes-on-server-death 退回批次 3 的写法：只半关两头的写、不再读客户端的请求，客户端正在写或接着写
//! 一帧大过发送缓冲的帧就一直堵着（自证 bench 那一格在机器忙时量到过：客户端停在 writing-frame 300 秒、`kill -9` 之后一次都没重连）。
//! 弄坏开关 fault-proxy-only-stops-reading-on-server-death：只关客户端那头的读、连接与 fd 留着；unix socket 上对端的写照样出错，
//! TCP 上对端接着写不出错，写满窗口就堵住（批次 4 交下来的那一条疑虑）。

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::client::{ClientStream, RouteAddress};
use crate::protocol::{
    is_connection_level_request, FRAME_LENGTH_FIELD_BYTES, MAXIMUM_FRAME_BODY_BYTES,
};

/// 「吞掉回应再断」：整帧转过去之后等这么久再关，让服务来得及做完它。
const SWALLOW_THEN_CUT_DELAY: Duration = Duration::from_millis(200);
/// 停代理时接连接的线程隔这么久看一次停标记。
const ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// 代理在哪儿接客户端。
#[derive(Clone, Debug)]
pub enum ProxyListen {
    UnixSocket(PathBuf),
    /// `127.0.0.1:0` 这类；端口 0 由系统分。
    Network(String),
}

#[derive(Default)]
pub struct ProxyControl {
    stop: AtomicBool,
    pub cut_mid_frame_requests: AtomicU32,
    pub swallow_responses_then_cut_requests: AtomicU32,
    pub half_open_connections_remaining: AtomicU32,
    pub half_open_after_frames: AtomicU64,
    pub duplicate_every_frames: AtomicU32,
    pub delay_minimum_milliseconds: AtomicU64,
    pub delay_maximum_milliseconds: AtomicU64,
    pub connections_accepted: AtomicU64,
    pub frames_forwarded: AtomicU64,
    pub duplicates_injected: AtomicU64,
    pub cuts_done: AtomicU64,
    pub half_open_done: AtomicU64,
    pub stall_reading_connections_remaining: AtomicU32,
    pub stalls_done: AtomicU64,
    pub server_deaths_seen: AtomicU64,
    half_closes_on_server_death: AtomicBool,
    only_stops_reading_on_server_death: AtomicBool,
    next_connection: AtomicU64,
    /// 代理留着的每条连接两头的复制，带代理给的连接号：服务那头没了时按连接号放掉，停代理时全放掉。
    open_streams: Mutex<Vec<(u64, ClientStream)>>,
}

/// 计数器减一，已经是 0 回假（一次性的控制项用）。
fn take_one(counter: &AtomicU32) -> bool {
    counter
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
            remaining.checked_sub(1)
        })
        .is_ok()
}

impl ProxyControl {
    fn random_delay(&self, random_state: &mut u64) -> Duration {
        let minimum = self.delay_minimum_milliseconds.load(Ordering::SeqCst);
        let maximum = self.delay_maximum_milliseconds.load(Ordering::SeqCst);
        if maximum == 0 || maximum < minimum {
            return Duration::ZERO;
        }
        *random_state ^= *random_state << 13;
        *random_state ^= *random_state >> 7;
        *random_state ^= *random_state << 17;
        Duration::from_millis(minimum + *random_state % (maximum - minimum + 1))
    }

    /// 关掉此刻全部被代理的连接，代理照常接新连接（压测器判卡住之后用，让堵在读写里的客户端出来）。
    pub fn cut_all_connections(&self) {
        let streams = std::mem::take(
            &mut *self
                .open_streams
                .lock()
                .expect("连接登记表的锁只在登记与停代理时短暂持有"),
        );
        for (_, stream) in streams {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }

    fn remember(&self, connection: u64, stream: &ClientStream) {
        if let Ok(clone) = stream.try_clone() {
            self.open_streams
                .lock()
                .expect("连接登记表的锁只在登记与停代理时短暂持有")
                .push((connection, clone));
        }
    }

    /// 放掉代理留着的这条连接的复制（fd 随之关掉）。
    fn forget(&self, connection: u64) {
        self.open_streams
            .lock()
            .expect("连接登记表的锁只在登记与停代理时短暂持有")
            .retain(|(remembered, _)| *remembered != connection);
    }
}

/// 一条被代理的连接两个方向共用的状态。
#[derive(Default)]
struct ConnectionFaults {
    is_half_open: AtomicBool,
    is_swallowing_responses: AtomicBool,
}

pub struct FaultProxy {
    route: RouteAddress,
    control: Arc<ProxyControl>,
    accept_thread: Option<thread::JoinHandle<()>>,
}

fn time_seed() -> u64 {
    let nanoseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.subsec_nanos());
    u64::from(nanoseconds) | 1
}

fn connect_upstream(upstream: &RouteAddress) -> io::Result<ClientStream> {
    match upstream {
        RouteAddress::UnixSocket(path) => UnixStream::connect(path).map(ClientStream::Unix),
        RouteAddress::Network(address) => {
            let stream = TcpStream::connect(address)?;
            stream.set_nodelay(true)?;
            Ok(ClientStream::Network(stream))
        }
    }
}

/// 读一整帧（长度字段加帧体）；在帧边界上读到流尾回 `None`。
fn read_whole_frame(stream: &mut ClientStream) -> io::Result<Option<Vec<u8>>> {
    let mut length_field = [0_u8; FRAME_LENGTH_FIELD_BYTES];
    let mut filled = 0;
    while filled < FRAME_LENGTH_FIELD_BYTES {
        let read_now = stream.read(&mut length_field[filled..])?;
        if read_now == 0 {
            if filled == 0 {
                return Ok(None);
            }
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "长度字段读到一半",
            ));
        }
        filled += read_now;
    }
    let declared = u32::from_le_bytes(length_field);
    if declared > MAXIMUM_FRAME_BODY_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "帧体长度超上限"));
    }
    let body_bytes = usize::try_from(declared).expect("u32 放得进 usize");
    let mut frame = vec![0_u8; FRAME_LENGTH_FIELD_BYTES + body_bytes];
    frame[..FRAME_LENGTH_FIELD_BYTES].copy_from_slice(&length_field);
    stream.read_exact(&mut frame[FRAME_LENGTH_FIELD_BYTES..])?;
    Ok(Some(frame))
}

fn close_both(client: &ClientStream, server: &ClientStream) {
    // 对端可能已经关了；关不上不影响注入的结果。
    let _ = client.shutdown(Shutdown::Both);
    let _ = server.shutdown(Shutdown::Both);
}

/// 服务那头没了：客户端那头真关掉（见文件头；两个弄坏开关各退回一种旧写法）。
fn server_side_gone(
    connection: u64,
    client: &ClientStream,
    server: &ClientStream,
    control: &ProxyControl,
) {
    control.server_deaths_seen.fetch_add(1, Ordering::SeqCst);
    if control.half_closes_on_server_death.load(Ordering::SeqCst) {
        let _ = client.shutdown(Shutdown::Write);
        let _ = server.shutdown(Shutdown::Write);
        return;
    }
    if control
        .only_stops_reading_on_server_death
        .load(Ordering::SeqCst)
    {
        let _ = client.shutdown(Shutdown::Read);
        return;
    }
    // 设不上 RST 也照关：对端读到流尾、接着写时由内核回 RST，只是慢一步。
    let _ = client.reset_on_close();
    close_both(client, server);
    control.forget(connection);
}

impl FaultProxy {
    pub fn start(
        listen: ProxyListen,
        upstream: RouteAddress,
        break_switches: &BreakSwitches,
    ) -> Result<FaultProxy, String> {
        let control = Arc::new(ProxyControl::default());
        control.half_closes_on_server_death.store(
            break_switches.is_enabled(BreakSwitch::FaultProxyHalfClosesOnServerDeath),
            Ordering::SeqCst,
        );
        control.only_stops_reading_on_server_death.store(
            break_switches.is_enabled(BreakSwitch::FaultProxyOnlyStopsReadingOnServerDeath),
            Ordering::SeqCst,
        );
        let (route, accept_thread) = match listen {
            ProxyListen::UnixSocket(path) => {
                let listener = UnixListener::bind(&path)
                    .map_err(|error| format!("代理绑不上 {}：{error}", path.display()))?;
                listener
                    .set_nonblocking(true)
                    .map_err(|error| format!("代理设不上非阻塞：{error}"))?;
                let control = Arc::clone(&control);
                let handle = thread::spawn(move || loop {
                    if control.stop.load(Ordering::SeqCst) {
                        return;
                    }
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let _ = stream.set_nonblocking(false);
                            spawn_connection(ClientStream::Unix(stream), &upstream, &control);
                        }
                        Err(_) => thread::sleep(ACCEPT_POLL_INTERVAL),
                    }
                });
                (RouteAddress::UnixSocket(path), handle)
            }
            ProxyListen::Network(address) => {
                let listener = TcpListener::bind(&address)
                    .map_err(|error| format!("代理绑不上 {address}：{error}"))?;
                let bound = listener
                    .local_addr()
                    .map_err(|error| format!("读不出代理的地址：{error}"))?;
                listener
                    .set_nonblocking(true)
                    .map_err(|error| format!("代理设不上非阻塞：{error}"))?;
                let control = Arc::clone(&control);
                let handle = thread::spawn(move || loop {
                    if control.stop.load(Ordering::SeqCst) {
                        return;
                    }
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let _ = stream.set_nonblocking(false);
                            let _ = stream.set_nodelay(true);
                            spawn_connection(ClientStream::Network(stream), &upstream, &control);
                        }
                        Err(_) => thread::sleep(ACCEPT_POLL_INTERVAL),
                    }
                });
                (RouteAddress::Network(bound.to_string()), handle)
            }
        };
        Ok(FaultProxy {
            route,
            control,
            accept_thread: Some(accept_thread),
        })
    }

    /// 客户端连这里。
    pub fn route(&self) -> RouteAddress {
        self.route.clone()
    }

    pub fn control(&self) -> &ProxyControl {
        &self.control
    }

    /// 给别的线程（压测器的故障调度）拿着改控制项。
    pub fn shared_control(&self) -> Arc<ProxyControl> {
        Arc::clone(&self.control)
    }

    /// 停接新连接，关掉全部被代理的连接。
    pub fn stop(&mut self) {
        self.control.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.accept_thread.take() {
            // 接连接的线程只轮询停标记与 accept，不会 panic；join 只为等它退出。
            let _ = handle.join();
        }
        let streams = std::mem::take(
            &mut *self
                .control
                .open_streams
                .lock()
                .expect("连接登记表的锁只在登记与停代理时短暂持有"),
        );
        for (_, stream) in streams {
            let _ = stream.shutdown(Shutdown::Both);
        }
        if let RouteAddress::UnixSocket(path) = &self.route {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl Drop for FaultProxy {
    fn drop(&mut self) {
        self.stop();
    }
}

fn spawn_connection(client: ClientStream, upstream: &RouteAddress, control: &Arc<ProxyControl>) {
    control.connections_accepted.fetch_add(1, Ordering::SeqCst);
    let Ok(server) = connect_upstream(upstream) else {
        let _ = client.shutdown(Shutdown::Both);
        return;
    };
    let connection = control.next_connection.fetch_add(1, Ordering::SeqCst);
    control.remember(connection, &client);
    control.remember(connection, &server);
    let half_open_after = if take_one(&control.half_open_connections_remaining) {
        control.half_open_after_frames.load(Ordering::SeqCst)
    } else {
        0
    };
    let stalls_reading = take_one(&control.stall_reading_connections_remaining);
    let faults = Arc::new(ConnectionFaults::default());
    let (Ok(client_reader), Ok(server_reader)) = (client.try_clone(), server.try_clone()) else {
        close_both(&client, &server);
        return;
    };
    let (Ok(client_for_cleanup), Ok(server_for_cleanup)) = (client.try_clone(), server.try_clone())
    else {
        close_both(&client, &server);
        return;
    };
    // 线程带名字：压测器判卡住时按线程打 /proc 里等在哪，靠名字认出是哪一个。
    let requests_started = {
        let control = Arc::clone(control);
        let faults = Arc::clone(&faults);
        thread::Builder::new()
            .name("proxy-requests".to_string())
            .spawn(move || {
                let plan = RequestForwarding {
                    connection,
                    half_open_after,
                    stalls_reading,
                };
                forward_requests(client_reader, server, &control, &faults, &plan);
            })
            .is_ok()
    };
    let control = Arc::clone(control);
    let responses_started = requests_started
        && thread::Builder::new()
            .name("proxy-responses".to_string())
            .spawn(move || forward_responses(connection, server_reader, client, &control, &faults))
            .is_ok();
    if !responses_started {
        close_both(&client_for_cleanup, &server_for_cleanup);
    }
}

/// 一条连接上转请求那一头的安排。
struct RequestForwarding {
    connection: u64,
    /// 转满这么多帧之后半开（0 是不半开）。
    half_open_after: u64,
    /// 这条连接一个字节都不读客户端的。
    stalls_reading: bool,
}

/// 停读的连接隔这么久看一次代理停没停。
const STALL_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// 客户端到服务：按帧转，注入断帧、吞回应、半开、迟到的重复、延迟与停读。
fn forward_requests(
    mut client: ClientStream,
    mut server: ClientStream,
    control: &ProxyControl,
    faults: &ConnectionFaults,
    plan: &RequestForwarding,
) {
    if plan.stalls_reading {
        control.stalls_done.fetch_add(1, Ordering::SeqCst);
        // 不读也不关：客户端的写满了发送缓冲就堵住，直到它自己的写超时；代理停的时候才放手。
        while !control.stop.load(Ordering::SeqCst) {
            thread::sleep(STALL_POLL_INTERVAL);
        }
        return;
    }
    let mut random_state = time_seed();
    let mut frames_on_connection = 0_u64;
    let mut held_duplicate: Option<Vec<u8>> = None;
    while let Ok(Some(frame)) = read_whole_frame(&mut client) {
        if faults.is_half_open.load(Ordering::SeqCst) {
            continue;
        }
        if is_connection_level_request(&frame) {
            if server.write_all(&frame).is_err() {
                server_side_gone(plan.connection, &client, &server, control);
                return;
            }
            continue;
        }
        let delay = control.random_delay(&mut random_state);
        if !delay.is_zero() {
            thread::sleep(delay);
        }
        if take_one(&control.cut_mid_frame_requests) {
            let half = FRAME_LENGTH_FIELD_BYTES + (frame.len() - FRAME_LENGTH_FIELD_BYTES) / 2;
            let _ = server.write_all(&frame[..half.max(FRAME_LENGTH_FIELD_BYTES + 1)]);
            control.cuts_done.fetch_add(1, Ordering::SeqCst);
            close_both(&client, &server);
            return;
        }
        if server.write_all(&frame).is_err() {
            server_side_gone(plan.connection, &client, &server, control);
            return;
        }
        frames_on_connection += 1;
        control.frames_forwarded.fetch_add(1, Ordering::SeqCst);
        if let Some(duplicate) = held_duplicate.take() {
            if server.write_all(&duplicate).is_err() {
                server_side_gone(plan.connection, &client, &server, control);
                return;
            }
            control.duplicates_injected.fetch_add(1, Ordering::SeqCst);
        }
        let every = u64::from(control.duplicate_every_frames.load(Ordering::SeqCst));
        if every > 0 && frames_on_connection.is_multiple_of(every) {
            held_duplicate = Some(frame);
        }
        if take_one(&control.swallow_responses_then_cut_requests) {
            faults.is_swallowing_responses.store(true, Ordering::SeqCst);
            thread::sleep(SWALLOW_THEN_CUT_DELAY);
            control.cuts_done.fetch_add(1, Ordering::SeqCst);
            close_both(&client, &server);
            return;
        }
        if plan.half_open_after > 0 && frames_on_connection == plan.half_open_after {
            faults.is_half_open.store(true, Ordering::SeqCst);
            control.half_open_done.fetch_add(1, Ordering::SeqCst);
        }
    }
    // 客户端那头关了：服务那头只关写的一半，让它回完已收的帧再关。
    let _ = server.shutdown(Shutdown::Write);
}

/// 服务到客户端：按帧转；半开或吞回应时丢掉。
fn forward_responses(
    connection: u64,
    mut server: ClientStream,
    mut client: ClientStream,
    control: &ProxyControl,
    faults: &ConnectionFaults,
) {
    let mut random_state = time_seed().rotate_left(17);
    loop {
        let Ok(Some(frame)) = read_whole_frame(&mut server) else {
            // 服务那头关了或读出错；半开的连接照旧不转也不关。
            if !faults.is_half_open.load(Ordering::SeqCst) {
                server_side_gone(connection, &client, &server, control);
            }
            return;
        };
        if faults.is_half_open.load(Ordering::SeqCst)
            || faults.is_swallowing_responses.load(Ordering::SeqCst)
        {
            continue;
        }
        let delay = control.random_delay(&mut random_state);
        if !delay.is_zero() {
            thread::sleep(delay);
        }
        if client.write_all(&frame).is_err() {
            // 客户端那头没了：两头都关。
            close_both(&client, &server);
            return;
        }
    }
}
