//! 连接级的帧：连接宣告（客户端身份、传输、说明文字）、列连接的请求与回应、宣告停服。

use super::*;

/// 报到帧（序号 0，服务不回）。
pub fn encode_connection_announcement_frame(announcement: &ClientAnnouncement) -> Vec<u8> {
    let mut encoder = begin_request(
        CONNECTION_LEVEL_SEQUENCE,
        OperationCode::ConnectionAnnouncement.code(),
        announcement.host.len() + announcement.task.len() + announcement.attempt.len() + 7,
    );
    put_announcement(&mut encoder, announcement);
    encoder.finish()
}

fn put_announcement(encoder: &mut FrameEncoder, announcement: &ClientAnnouncement) {
    encoder.put_short_text(&announcement.host);
    encoder.put_short_text(&announcement.task);
    encoder.put_short_text(&announcement.attempt);
    encoder.put_u32(announcement.process_identifier);
}

pub fn encode_connected_clients_frame(sequence: u64) -> Vec<u8> {
    begin_request(sequence, OperationCode::ConnectedClients.code(), 0).finish()
}

pub fn encode_announce_stop_frame(sequence: u64) -> Vec<u8> {
    begin_request(sequence, OperationCode::AnnounceStop.code(), 0).finish()
}

/// 客户端报到的身份：哪台、哪个任务的第几次、哪个进程。不在 task-server 里跑的，任务号与第几次是空串。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClientAnnouncement {
    pub host: String,
    pub task: String,
    pub attempt: String,
    pub process_identifier: u32,
}

/// 截到至多 255 字节，落在字符边界上。
fn truncate_to_short_text(text: &str) -> String {
    let mut end = text.len().min(MAXIMUM_ANNOUNCEMENT_TEXT_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}

impl ClientAnnouncement {
    /// 各段超过 255 字节的截掉（主机名与任务号都远短于它；截是为了编帧时不撞长度上限）。
    pub fn new(
        host: &str,
        task: &str,
        attempt: &str,
        process_identifier: u32,
    ) -> ClientAnnouncement {
        ClientAnnouncement {
            host: truncate_to_short_text(host),
            task: truncate_to_short_text(task),
            attempt: truncate_to_short_text(attempt),
            process_identifier,
        }
    }
}

/// 一条连接走的哪一处监听。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Transport {
    UnixSocket,
    Network,
}

impl Transport {
    pub const ALL: [Transport; 2] = [Transport::UnixSocket, Transport::Network];

    pub fn code(self) -> u8 {
        match self {
            Transport::UnixSocket => 1,
            Transport::Network => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Transport::UnixSocket => "unix",
            Transport::Network => "tcp",
        }
    }

    pub fn from_code(code: u8) -> Option<Transport> {
        Transport::ALL
            .into_iter()
            .find(|transport| transport.code() == code)
    }
}

/// 列连着的客户端时的一条。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConnectedClient {
    pub connection: u64,
    pub transport: Transport,
    pub peer_address: String,
    pub announcement: Option<ClientAnnouncement>,
    pub connected_since_milliseconds: u64,
}

/// 列连着的客户端、宣布停服务的回应。
pub fn encode_connected_clients_response(
    sequence: u64,
    operation: OperationCode,
    clients: &[ConnectedClient],
) -> Vec<u8> {
    let mut encoder = begin_response(
        sequence,
        ResponseStatus::Completed,
        operation.code(),
        4 + clients.len() * 96,
    );
    encoder.put_u32(u32_from_usize(clients.len(), "客户端个数"));
    for client in clients {
        encoder.put_u64(client.connection);
        encoder.put_u8(client.transport.code());
        encoder.put_length_prefixed(client.peer_address.as_bytes());
        match &client.announcement {
            Some(announcement) => {
                encoder.put_u8(1);
                put_announcement(&mut encoder, announcement);
            }
            None => encoder.put_u8(0),
        }
        encoder.put_u64(client.connected_since_milliseconds);
    }
    encoder.finish()
}

/// 服务停了（序号 0、操作码 0，服务主动发）。
pub fn encode_server_stopping(message: &str) -> Vec<u8> {
    let mut encoder = begin_response(
        CONNECTION_LEVEL_SEQUENCE,
        ResponseStatus::ServerStopping,
        0,
        message.len() + 4,
    );
    encoder.put_length_prefixed(message.as_bytes());
    encoder.finish()
}
