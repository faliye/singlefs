//! 容错客户端按前缀扫遇到服务回忙（`prune_server::reliable_client` 文件头「按前缀扫」与「防雪崩」两条）：测试替身是一份只会答两帧的假服务
//! （这条用例独占的临时目录里的 unix socket），头一帧扫回忙、重发的那一帧扫回行；客户端照忙给的毫秒数等过再重发，扫照样拿到行，不把忙当错误交回。
//! 不起真服务、不连真服务。

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use prune_server::client::RouteAddress;
use prune_server::protocol::{
    begin_response, encode_busy, peek_request_header, OperationCode, ResponseStatus, ScanEntry,
    Table, FRAME_LENGTH_FIELD_BYTES,
};
use prune_server::reliable_client::{ReliableClient, ReliableClientSettings};

/// 假服务回忙时叫客户端隔多久再来。
const RETRY_AFTER_MILLISECONDS: u32 = 20;
/// 假服务等下一帧的上限：客户端不重发时假服务到点放手，用例照断言判红，不挂住。
const TEST_DOUBLE_READ_TIMEOUT: Duration = Duration::from_secs(30);
/// 扫的那一张表与前缀（假服务不看内容，只照帧头的操作码答）。
const TABLE: Table = Table::Conclusions;
const PREFIX: &[u8] = b"scan-after-busy/";

static TEMPORARY_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 这条用例独占的临时目录：放手时删。名字短，socket 全路径不超 107 字节。
struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn create() -> TemporaryDirectory {
        let path = std::env::temp_dir().join(format!(
            "prune-scan-busy-{}-{}",
            std::process::id(),
            TEMPORARY_DIRECTORY_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir(&path)
            .unwrap_or_else(|error| panic!("建不了临时目录 {}：{error}", path.display()));
        TemporaryDirectory { path }
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.path) {
            eprintln!("删不了用例的临时目录 {}：{error}", self.path.display());
        }
    }
}

/// 读一帧（长度字段小端，`protocol` 文件头），交回帧体；对端关了或到了读超时交回 `None`。
fn read_frame_body(stream: &mut UnixStream) -> Option<Vec<u8>> {
    let mut length_field = [0_u8; FRAME_LENGTH_FIELD_BYTES];
    stream.read_exact(&mut length_field).ok()?;
    let body_bytes = usize::try_from(u32::from_le_bytes(length_field)).expect("u32 放得进 usize");
    let mut body = vec![0_u8; body_bytes];
    stream.read_exact(&mut body).ok()?;
    Some(body)
}

/// 扫回行的回应帧（截断标记、条数、逐条键值，`protocol` 文件头）。
fn encode_scan_entries(sequence: u64, entries: &[ScanEntry]) -> Vec<u8> {
    let mut encoder = begin_response(
        sequence,
        ResponseStatus::Completed,
        OperationCode::PrefixScan.code(),
        64,
    );
    encoder.put_u8(0);
    encoder.put_u32(u32::try_from(entries.len()).expect("用例里只有几行"));
    for entry in entries {
        encoder.put_length_prefixed(&entry.key);
        encoder.put_length_prefixed(&entry.value);
    }
    encoder.finish()
}

/// 假服务：收下报到帧，头一帧扫回忙，下一帧扫回 `entries`；交回收到的每一帧扫的（序号，操作码）。
fn answer_busy_then_the_scanned_rows(
    listener: UnixListener,
    entries: Vec<ScanEntry>,
) -> thread::JoinHandle<Vec<(u64, u8)>> {
    thread::spawn(move || {
        let (mut stream, _address) = listener.accept().expect("假服务收得下客户端的连接");
        stream
            .set_read_timeout(Some(TEST_DOUBLE_READ_TIMEOUT))
            .expect("unix socket 设得了读超时");
        let mut scan_headers = Vec::new();
        let announcement = read_frame_body(&mut stream).expect("客户端连上先发报到帧");
        assert_eq!(
            peek_request_header(&announcement).map(|(_sequence, code)| code),
            Some(OperationCode::ConnectionAnnouncement.code()),
            "客户端连上发的头一帧该是报到"
        );
        while scan_headers.len() < 2 {
            let Some(body) = read_frame_body(&mut stream) else {
                break;
            };
            let header = peek_request_header(&body).expect("请求帧至少有帧头");
            scan_headers.push(header);
            let (sequence, _operation_code) = header;
            let answer = if scan_headers.len() == 1 {
                encode_busy(
                    sequence,
                    OperationCode::PrefixScan.code(),
                    RETRY_AFTER_MILLISECONDS,
                    "忙：测试替身回的头一帧",
                )
            } else {
                encode_scan_entries(sequence, &entries)
            };
            stream.write_all(&answer).expect("假服务写得出回应");
        }
        scan_headers
    })
}

#[test]
fn prefix_scan_answered_busy_once_waits_the_retry_after_and_hands_back_the_rows_of_the_resent_frame(
) {
    let directory = TemporaryDirectory::create();
    let socket_path = directory.path.join("test-double.sock");
    let listener = UnixListener::bind(&socket_path)
        .unwrap_or_else(|error| panic!("绑不上 {}：{error}", socket_path.display()));
    let stored_rows = vec![
        ScanEntry {
            key: b"scan-after-busy/alpha".to_vec(),
            value: b"first".to_vec(),
        },
        ScanEntry {
            key: b"scan-after-busy/beta".to_vec(),
            value: b"second".to_vec(),
        },
    ];
    let test_double_thread = answer_busy_then_the_scanned_rows(listener, stored_rows.clone());
    let mut client = ReliableClient::open(ReliableClientSettings::new(
        RouteAddress::UnixSocket(socket_path.clone()),
        directory.path.join("client.journal"),
    ))
    .unwrap_or_else(|failure| panic!("连不上测试替身：{failure}"));

    let scanned = client.scan(TABLE, PREFIX, 16);

    assert_eq!(
        scanned,
        Ok((false, stored_rows)),
        "头一帧回忙、重发的那一帧回两行：扫该交回那两行、没截断，不把忙当错误交回"
    );
    let diagnostics = client.diagnostics();
    assert_eq!(diagnostics.busy_responses, 1, "客户端该只收到一次忙");
    assert_eq!(
        diagnostics.busy_waits.len(),
        1,
        "回了忙的那一帧该重发一次：{:?}",
        diagnostics.busy_waits
    );
    let (waited, requested) = diagnostics.busy_waits[0];
    let retry_after = Duration::from_millis(u64::from(RETRY_AFTER_MILLISECONDS));
    assert_eq!(requested, retry_after, "记下的该是服务叫等的毫秒数");
    assert!(
        waited >= retry_after,
        "重发之前至少等服务叫等的 {retry_after:?}，实际等了 {waited:?}"
    );
    drop(client);
    let scan_headers = test_double_thread.join().expect("假服务线程不 panic");
    let scan_code = OperationCode::PrefixScan.code();
    assert_eq!(
        scan_headers
            .iter()
            .map(|(_sequence, code)| *code)
            .collect::<Vec<_>>(),
        vec![scan_code, scan_code],
        "假服务该收到两帧扫：头一帧与回忙之后重发的那一帧"
    );
    assert_ne!(
        scan_headers[0].0, scan_headers[1].0,
        "回忙之后重发的那一帧用新的序号"
    );
}
