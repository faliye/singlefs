//! 服务这一侧的连接登记，与客户端这一侧的报到身份。
//!
//! - 登记：每条接进来的连接一条（连接号、走哪条、对端地址、连上的时刻、报到的身份、往这条连接写回应的通道）。
//!   列连着的客户端、宣布停服务时给每条连接发「服务停了」、写故障行时带上当时在处理的客户端，都从这里取。
//! - 报到身份：客户端连上之后头一帧报主机名、`TASK_SERVER_TASK`、`TASK_SERVER_ATTEMPT`、进程号（`protocol` 文件头「报到」）。
//! - 写出队的账：每条连接排着、还没写出去的帧按字节记账（这条连接一份、整个服务一份，统计里报整个服务的峰值）。
//!   回应带着它那一帧的待处理名额（`server::PendingSlot`，名额里记着请求与回应的字节）一起排进队，写出去或丢掉（连接没了）
//!   才放手，所以对端不读时回应占的内存受待处理额度管（`server.rs` 文件头「额度」）。不带名额的几种（忙、拒绝、列客户端、
//!   宣布停服务、服务停了）一条连接至多排 `UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION` 字节，再多就关这条连接：
//!   只发不收的客户端拿忙回应也堆不满服务的内存。弄坏开关 response-slot-released-on-enqueue 退回回应一进队就放名额的写法。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::server::PendingSlot;

/// 不带待处理名额的回应（忙、拒绝、管理回应、服务停了）一条连接的写出队里至多排这么多字节，再多就关这条连接（文件头）。
pub const UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION: usize = 1024 * 1024;

/// 交给一条连接的写线程的东西：一帧，或「前面排着的都写进套接字并 flush 了就回个话」的落盘回执。
pub enum Outgoing {
    Frame(QueuedFrame),
    FlushedReceipt(Sender<()>),
}

/// 排进写出队的一帧：字节与它的账；写出去或丢掉时账随之销掉、名额随之放手。
pub struct QueuedFrame {
    pub bytes: Vec<u8>,
    #[allow(
        dead_code,
        reason = "只为放手时销账：OutboundTicket 的 Drop 减写出队的字节，带着的待处理名额随之放手"
    )]
    ticket: OutboundTicket,
}

struct OutboundTicket {
    account: Arc<OutboundAccount>,
    frame_bytes: usize,
    is_budgeted: bool,
    #[allow(
        dead_code,
        reason = "只为持有：回应写出去或丢掉之前名额不放手（PendingSlot 的 Drop 放）"
    )]
    slot: Option<PendingSlot>,
}

impl Drop for OutboundTicket {
    fn drop(&mut self) {
        self.account
            .queued_bytes
            .fetch_sub(self.frame_bytes, Ordering::SeqCst);
        self.account
            .totals
            .queued_bytes
            .fetch_sub(self.frame_bytes, Ordering::SeqCst);
        if !self.is_budgeted {
            self.account
                .unbudgeted_bytes
                .fetch_sub(self.frame_bytes, Ordering::SeqCst);
        }
    }
}

/// 整个服务的写出队：此刻排着的字节、峰值、因不带名额的回应排超了上限关掉的连接数。
#[derive(Default)]
pub struct OutboundTotals {
    pub queued_bytes: AtomicUsize,
    pub queued_bytes_peak: AtomicUsize,
    pub overflow_closures: AtomicU64,
}

/// 一条连接的写出队的账。
struct OutboundAccount {
    queued_bytes: AtomicUsize,
    unbudgeted_bytes: AtomicUsize,
    totals: Arc<OutboundTotals>,
    /// 把这条连接两个方向都关掉（堵在写里的写线程随即出错、读线程读到流尾）。
    shut_down: Box<dyn Fn() + Send + Sync>,
    is_closed_for_overflow: AtomicBool,
    /// 弄坏开关 response-slot-released-on-enqueue：回应一进队就放名额。
    releases_slot_on_enqueue: bool,
}

/// 这条连接的写线程已经收尾（连接没了），或不带名额的回应排超了上限、连接关了，帧排不进去。
#[derive(Debug)]
pub struct WriterGone;

/// 往一条连接写回应的通道。`send_response` 排带名额的回应，`send` 排不带名额的几种（文件头）；
/// 要确认已经写进套接字的（宣布停服务时的「服务停了」）用 [`Responder::send_and_wait_flushed`]：排一帧再排一张回执，
/// 写线程写完、flush 完才回话。
#[derive(Clone)]
pub struct Responder {
    sender: Sender<Outgoing>,
    account: Arc<OutboundAccount>,
}

impl Responder {
    /// 新建一对：写线程拿 [`Receiver`]，处理帧的各方拿 [`Responder`]。`shut_down` 关这条连接（不带名额的回应排超了上限时调）。
    pub fn channel(
        totals: Arc<OutboundTotals>,
        shut_down: Box<dyn Fn() + Send + Sync>,
        releases_slot_on_enqueue: bool,
    ) -> (Responder, Receiver<Outgoing>) {
        let (sender, receiver) = mpsc::channel();
        let account = Arc::new(OutboundAccount {
            queued_bytes: AtomicUsize::new(0),
            unbudgeted_bytes: AtomicUsize::new(0),
            totals,
            shut_down,
            is_closed_for_overflow: AtomicBool::new(false),
            releases_slot_on_enqueue,
        });
        (Responder { sender, account }, receiver)
    }

    fn enqueue(
        &self,
        frame: Vec<u8>,
        is_budgeted: bool,
        slot: Option<PendingSlot>,
    ) -> Result<(), WriterGone> {
        let frame_bytes = frame.len();
        self.account
            .queued_bytes
            .fetch_add(frame_bytes, Ordering::SeqCst);
        let total = self
            .account
            .totals
            .queued_bytes
            .fetch_add(frame_bytes, Ordering::SeqCst)
            + frame_bytes;
        self.account
            .totals
            .queued_bytes_peak
            .fetch_max(total, Ordering::SeqCst);
        let ticket = OutboundTicket {
            account: Arc::clone(&self.account),
            frame_bytes,
            is_budgeted,
            slot,
        };
        // 排不进去的那一帧随错误一起放手，账也随之销掉。
        self.sender
            .send(Outgoing::Frame(QueuedFrame {
                bytes: frame,
                ticket,
            }))
            .map_err(|_| WriterGone)
    }

    /// 排一帧回应，`slot`（记着请求与回应的字节）跟着它，写出去或丢掉才放手；写线程已经收尾回 [`WriterGone`]。
    pub fn send_response(&self, frame: Vec<u8>, slot: PendingSlot) -> Result<(), WriterGone> {
        if self.account.releases_slot_on_enqueue {
            drop(slot);
            return self.enqueue(frame, true, None);
        }
        self.enqueue(frame, true, Some(slot))
    }

    /// 排一帧不带名额的回应（忙、拒绝、管理回应、服务停了）。这一类在这条连接的写出队里排超了
    /// `UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION` 就关连接、回 [`WriterGone`]；写线程已经收尾同样回它。
    pub fn send(&self, frame: Vec<u8>) -> Result<(), WriterGone> {
        let frame_bytes = frame.len();
        let reached = self
            .account
            .unbudgeted_bytes
            .fetch_add(frame_bytes, Ordering::SeqCst)
            + frame_bytes;
        if reached > UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION {
            self.account
                .unbudgeted_bytes
                .fetch_sub(frame_bytes, Ordering::SeqCst);
            if !self
                .account
                .is_closed_for_overflow
                .swap(true, Ordering::SeqCst)
            {
                self.account
                    .totals
                    .overflow_closures
                    .fetch_add(1, Ordering::Relaxed);
                (self.account.shut_down)();
            }
            return Err(WriterGone);
        }
        self.enqueue(frame, false, None)
    }

    /// 关这条连接两个方向（读线程随即读到流尾、收尾）：在调用方进程里起的那一份服务停服时用（`server::ServiceInThisProcess::stop`）。
    pub fn shut_down_connection(&self) {
        (self.account.shut_down)();
    }

    /// 排一帧并等它（连同排在它前面的）写进套接字、flush 完；写线程在 `deadline` 之前没回话、或已经收尾，回 `false`。
    pub fn send_and_wait_flushed(&self, frame: Vec<u8>, deadline: Instant) -> bool {
        let (receipt_sender, receipt) = mpsc::channel();
        if self.send(frame).is_err()
            || self
                .sender
                .send(Outgoing::FlushedReceipt(receipt_sender))
                .is_err()
        {
            return false;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        receipt
            .recv_timeout(remaining.max(Duration::from_millis(1)))
            .is_ok()
    }
}

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::committer::now_milliseconds;
use crate::protocol::{ClientAnnouncement, ConnectedClient, Transport};

/// task-server 给每次尝试的环境里放任务号与第几次的两个变量（`task-server/README.md`）。
pub const TASK_NUMBER_VARIABLE: &str = "TASK_SERVER_TASK";
pub const TASK_ATTEMPT_VARIABLE: &str = "TASK_SERVER_ATTEMPT";

fn local_host_name() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|text| text.trim().to_string())
        .unwrap_or_default()
}

/// 这个进程的报到身份：主机名、环境里的任务号与第几次（不在 task-server 里跑的是空串）、进程号。
/// 弄坏开关 client-announcement-omits-task 下任务号报空。
pub fn announcement_from_environment(break_switches: &BreakSwitches) -> ClientAnnouncement {
    let task = if break_switches.is_enabled(BreakSwitch::ClientAnnouncementOmitsTask) {
        String::new()
    } else {
        std::env::var(TASK_NUMBER_VARIABLE).unwrap_or_default()
    };
    ClientAnnouncement::new(
        &local_host_name(),
        &task,
        &std::env::var(TASK_ATTEMPT_VARIABLE).unwrap_or_default(),
        std::process::id(),
    )
}

/// 行里 `名=值` 的值：空的写 `-`，空白换成 `_`（一行按空白切字段）。
pub fn line_field_value(text: &str) -> String {
    if text.is_empty() {
        return "-".to_string();
    }
    text.chars()
        .map(|character| {
            if character.is_whitespace() {
                '_'
            } else {
                character
            }
        })
        .collect()
}

/// 报到身份写进一行的几个字段：`host=… task=… attempt=… pid=…`；没报到的四个都是 `-`。
pub fn announcement_fields(announcement: Option<&ClientAnnouncement>) -> String {
    match announcement {
        Some(announcement) => format!(
            "host={} task={} attempt={} pid={}",
            line_field_value(&announcement.host),
            line_field_value(&announcement.task),
            line_field_value(&announcement.attempt),
            announcement.process_identifier
        ),
        None => "host=- task=- attempt=- pid=-".to_string(),
    }
}

/// 一个连着的客户端写成一行的字段（控制命令、停服务记录、监听的输出都用这一种写法）。
pub fn connected_client_fields(client: &ConnectedClient) -> String {
    format!(
        "connection={} transport={} peer={} {} connected_since_milliseconds={}",
        client.connection,
        client.transport.name(),
        line_field_value(&client.peer_address),
        announcement_fields(client.announcement.as_ref()),
        client.connected_since_milliseconds
    )
}

/// 一条连接的登记。
pub struct ConnectionRecord {
    pub connection: u64,
    pub transport: Transport,
    pub peer_address: String,
    pub connected_since_milliseconds: u64,
    announcement: Mutex<Option<ClientAnnouncement>>,
}

impl ConnectionRecord {
    pub fn record_announcement(&self, announcement: ClientAnnouncement) {
        *self
            .announcement
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(announcement);
    }

    pub fn announcement(&self) -> Option<ClientAnnouncement> {
        self.announcement
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn snapshot(&self) -> ConnectedClient {
        ConnectedClient {
            connection: self.connection,
            transport: self.transport,
            peer_address: self.peer_address.clone(),
            announcement: self.announcement(),
            connected_since_milliseconds: self.connected_since_milliseconds,
        }
    }
}

struct RegisteredConnection {
    record: Arc<ConnectionRecord>,
    responder: Responder,
}

/// 全部连着的连接。
pub struct ConnectionRegistry {
    next_connection: AtomicU64,
    connections: Mutex<BTreeMap<u64, RegisteredConnection>>,
}

impl Default for ConnectionRegistry {
    fn default() -> ConnectionRegistry {
        ConnectionRegistry {
            next_connection: AtomicU64::new(1),
            connections: Mutex::new(BTreeMap::new()),
        }
    }
}

impl ConnectionRegistry {
    /// 登记一条新连接；`responder` 是往这条连接写回应的通道（宣布停服务时经它发通知）。
    pub fn register(
        &self,
        transport: Transport,
        peer_address: String,
        responder: Responder,
    ) -> Arc<ConnectionRecord> {
        let record = Arc::new(ConnectionRecord {
            connection: self.next_connection.fetch_add(1, Ordering::SeqCst),
            transport,
            peer_address,
            connected_since_milliseconds: now_milliseconds(),
            announcement: Mutex::new(None),
        });
        self.connections
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                record.connection,
                RegisteredConnection {
                    record: Arc::clone(&record),
                    responder,
                },
            );
        record
    }

    /// 撤掉登记：放手它的回应通道，这条连接的写线程才收得了尾。
    pub fn unregister(&self, connection: u64) {
        self.connections
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&connection);
    }

    /// `excluded` 之外的每条连接此刻的样子（按连接号升序）。
    pub fn clients_except(&self, excluded: u64) -> Vec<ConnectedClient> {
        self.connections
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .filter(|registered| registered.record.connection != excluded)
            .map(|registered| registered.record.snapshot())
            .collect()
    }

    /// 关掉此刻登记着的每一条连接（[`Responder::shut_down_connection`]）；先把各条的通道取出来再放锁，关的时候不拿着登记表的锁。
    pub fn shut_down_every_connection(&self) {
        let responders: Vec<Responder> = self
            .connections
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .map(|registered| registered.responder.clone())
            .collect();
        for responder in &responders {
            responder.shut_down_connection();
        }
    }

    /// 给 `excluded` 之外的每条连接发一帧，等每条都写进套接字、flush 完（至多到 `deadline`）才回；
    /// 回（写到了的条数，没写到的条数：连接正在收尾、写线程堵着到点没回话）。先把各条的通道取出来再放锁，
    /// 等回执时不拿着登记表的锁（写线程收尾时要撤登记）。
    pub fn send_to_all_except_and_wait_flushed(
        &self,
        excluded: u64,
        frame: &[u8],
        deadline: Instant,
    ) -> (usize, usize) {
        let responders: Vec<Responder> = self
            .connections
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .filter(|registered| registered.record.connection != excluded)
            .map(|registered| registered.responder.clone())
            .collect();
        let delivered = responders
            .iter()
            .filter(|responder| responder.send_and_wait_flushed(frame.to_vec(), deadline))
            .count();
        (delivered, responders.len() - delivered)
    }
}
