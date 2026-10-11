//! E162（崩溃放量判定块存储选型）第三段 S3：送块进程，以及跨机格数据通道在另一台那一头的中继。
//!
// admission: always 每跑一次都是新的送块与计时观测；送块、中继两个角色由 e162-verdict-store-network 起，不判准入
// run-condition: none 只算锚点、收发帧，不碰盘、不要设备与权限；准入只在手跑 anchors 时判
//!
//! 跑前登记：`research/prompts/e162-preregistration.md` 第六节 6.3 末段（帧、窗口、重连与重发）、
//! 补 5 开头那张表（这个 bin 的角色）、补 5.1（跨机格的接法、PC3-rate-X 的压节奏）、第十二节「修订」（第三段的补写法）。
//! 不带任何存储依赖：跨机格要把它编成 `x86_64-unknown-linux-musl` 静态、送到另一台去跑（补 5）。
//! 独立手写模型，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）。
//!
//! `e162_verdict_store_network.rs` 用 `#[path]` 把这个文件当成模块引进去：帧的编解码、块值、块键与锚点
//! 两个 bin 共用这一份；网络那个 bin 的单测自起送块角色也走这里的入口。
//!
//! 角色（第一个参数）：
//! - `send`：送块。接法二选一：`--connect tcp:<地址>:<端口>`（回环格：连转发进程，断了每 10 ms 重连、最多 30 秒）；
//!   `--listen unix:<路径>`（跨机格：等中继连进来，断了等下一个连接、最多 30 秒）。
//! - `relay`：`--relay unix:<路径>`，把自己的标准输入输出接到那个套接字上（跨机格的中继）。
//! - `anchors`：锚点自检，逐项打一行。
//!
//! `send` 的标准输出每行一次 `write`、不经缓冲；计时都在这个进程里用 `Instant` 计，读它的父进程不打时间戳
//! （`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）。第一行 `P <pid>`；每收到一个块的第一个确认一行
//! `A <累计确认数>`；每次重连一行 `R …`；收尾 `W …`（每 1 000 个确认一窗的结束时刻）与 `S …`（汇总）。

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

// ============================== 登记写死的常量 ==============================

/// seed_S2（登记 5.1）：只给锚点 A3、A3b、A4 用。
pub(crate) const SEED_THROUGHPUT_BLOCKS: u64 = 0xE162_0000_0000_0002;
/// seed_S3（登记 5.1）：S3 各格的块值。
pub(crate) const SEED_NETWORK_BLOCKS: u64 = 0xE162_0000_0000_0004;
/// seed_FP（登记 5.1）。
pub(crate) const SEED_FINGERPRINT: u64 = 0xE162_0000_0000_0007;

const SPLITMIX_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;
const SPLITMIX_FIRST_MULTIPLIER: u64 = 0xBF58_476D_1CE4_E5B9;
const SPLITMIX_SECOND_MULTIPLIER: u64 = 0x94D0_49BB_1331_11EB;
/// 第 j 块的起始状态 = seed XOR (j × 这个数 mod 2⁶⁴)（登记 5.1）。
const BLOCK_START_MULTIPLIER: u64 = 0xD1B5_4A32_D192_ED03;

/// 主取样点每块状态数 2¹⁶（每状态 1 字节，块值 65 536 字节）。
pub(crate) const PRIMARY_STATES_PER_BLOCK: u64 = 65_536;
/// S3-bs10 每块状态数 2¹⁰。
pub(crate) const SMALL_STATES_PER_BLOCK: u64 = 1_024;
pub(crate) const BLOCK_KEY_BYTES: usize = 32;
const FINGERPRINT_BYTES: usize = 16;
const INTERLEAVED_STREAM_COUNT: u64 = 64;
const SEGMENTS_PER_NODE: u64 = 8;

/// 帧：魔数 `SFBK`（4）｜块键（32）｜块值长度 u32 小端（4）｜块值｜CRC-32C（4，覆盖键、长度与块值）。
pub(crate) const BLOCK_FRAME_MAGIC: &[u8; 4] = b"SFBK";
/// 帧头：魔数、块键、块值长度。
pub(crate) const BLOCK_FRAME_HEADER_BYTES: usize = 4 + BLOCK_KEY_BYTES + 4;
pub(crate) const FRAME_CHECKSUM_BYTES: usize = 4;
/// 确认帧：魔数 `SFAK`（4）｜块键（32）｜CRC-32C（4，覆盖块键）。
pub(crate) const ACKNOWLEDGEMENT_FRAME_MAGIC: &[u8; 4] = b"SFAK";
pub(crate) const ACKNOWLEDGEMENT_FRAME_BYTES: usize = 4 + BLOCK_KEY_BYTES + 4;

/// 断线之后每隔这么久重连一次（回环格）或看一次有没有新连接（跨机格）。
const RECONNECT_INTERVAL: Duration = Duration::from_millis(10);
/// 重连最多等这么久（登记 6.3 末段、补 5 表）。
const RECONNECT_DEADLINE: Duration = Duration::from_secs(30);
/// 连着的时候这么久没有新确认就停（悬挂保护，不是性能门槛；修订第三段第 3 条）。
pub(crate) const DEFAULT_ACKNOWLEDGEMENT_STALL_SECONDS: u64 = 120;
/// 分窗：每 1 000 个确认一窗（补 6 Q3k）。
pub(crate) const ACKNOWLEDGEMENTS_PER_WINDOW: u64 = 1_000;
/// 两路各写 S2 块数的一半（登记 5.5 S3-T 一行）。
pub(crate) const PATH_BLOCK_COUNT: u64 = 50_000;

/// PC3-rate-X：送 200 块，第 i 帧不早于第一帧之后 i × 100 ms（补 5.1）。
pub(crate) const CROSS_RATE_CONTROL_BLOCKS: u64 = 200;
pub(crate) const CROSS_RATE_CONTROL_PACE_MILLISECONDS: u64 = 100;
/// 下界容最后一个确认晚到 2.2 秒（补 5.1 PC3-rate-X 一行）；按毫秒整数加，免得 19.9 + 2.2 的浮点和不是 22.1。
const CROSS_RATE_CONTROL_LATE_ACKNOWLEDGEMENT_MILLISECONDS: u64 = 2_200;

// ============================== 随机源与校验和 ==============================

/// SplitMix64（登记 5.1）：全部 64 位回绕。
pub(crate) struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub(crate) fn new(state: u64) -> Self {
        Self { state }
    }

    pub(crate) fn next_output(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX_INCREMENT);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(SPLITMIX_FIRST_MULTIPLIER);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(SPLITMIX_SECOND_MULTIPLIER);
        mixed ^ (mixed >> 31)
    }
}

/// CRC-32C（Castagnoli，反射多项式 0x82F63B78），查表法。
struct Crc32cTable {
    entries: [u32; 256],
}

impl Crc32cTable {
    fn new() -> Self {
        let mut entries = [0u32; 256];
        for (byte_value, entry) in entries.iter_mut().enumerate() {
            let mut remainder = u32::try_from(byte_value).expect("表下标小于 256");
            for _bit in 0..8 {
                remainder = if remainder & 1 == 1 {
                    (remainder >> 1) ^ 0x82F6_3B78
                } else {
                    remainder >> 1
                };
            }
            *entry = remainder;
        }
        Self { entries }
    }

    fn checksum(&self, parts: &[&[u8]]) -> u32 {
        let mut running = 0xFFFF_FFFFu32;
        for part in parts {
            for &byte in part.iter() {
                let table_index = usize::from(running.to_le_bytes()[0] ^ byte);
                running = (running >> 8) ^ self.entries[table_index];
            }
        }
        running ^ 0xFFFF_FFFF
    }
}

pub(crate) fn crc32c(parts: &[&[u8]]) -> u32 {
    thread_local! {
        static TABLE: Crc32cTable = Crc32cTable::new();
    }
    TABLE.with(|table| table.checksum(parts))
}

// ============================== 块值与块键 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValuePattern {
    /// P-rand：从起始状态连取 V/8 个输出，逐个小端拼成 V 字节。
    Random,
    /// P-sparse：每状态取一个输出 r，r >> 56 == 0 时是 1 + (r & 0xFF) mod 255，否则 0（只给锚点 A4 用）。
    Sparse,
}

/// 第 block_index 块的块值；每状态 1 字节，所以块值字节数 = 每块状态数。
pub(crate) fn block_value(pattern: ValuePattern, seed: u64, block_index: u64, states_per_block: u64) -> Vec<u8> {
    let value_bytes = usize::try_from(states_per_block).expect("每块状态数装得进 usize");
    let mut generator = SplitMix64::new(seed ^ block_index.wrapping_mul(BLOCK_START_MULTIPLIER));
    match pattern {
        ValuePattern::Random => {
            let mut value = Vec::with_capacity(value_bytes);
            for _word in 0..value_bytes / 8 {
                value.extend_from_slice(&generator.next_output().to_le_bytes());
            }
            value
        }
        ValuePattern::Sparse => {
            let mut value = vec![0u8; value_bytes];
            for state_verdict in value.iter_mut() {
                let output = generator.next_output();
                if output >> 56 == 0 {
                    *state_verdict = 1 + u8::try_from((output & 0xFF) % 255).expect("小于 255");
                }
            }
            value
        }
    }
}

/// 输入指纹：SplitMix64(seed_FP) 连取两个输出、小端拼成 16 字节。
pub(crate) fn input_fingerprint() -> [u8; FINGERPRINT_BYTES] {
    let mut generator = SplitMix64::new(SEED_FINGERPRINT);
    let mut fingerprint = [0u8; FINGERPRINT_BYTES];
    fingerprint[..8].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint[8..].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint
}

pub(crate) type BlockKey = [u8; BLOCK_KEY_BYTES];

/// 交错次序下第 block_index 块的键：流号 s = j mod 64、流内块号 b = j div 64；
/// 节点 u32 大端（s div 8）｜段号 u32 大端（s mod 8）｜起点 u64 大端（b × 每块状态数）｜输入指纹。
pub(crate) fn interleaved_block_key(block_index: u64, states_per_block: u64, fingerprint: &[u8; FINGERPRINT_BYTES]) -> BlockKey {
    let stream = block_index % INTERLEAVED_STREAM_COUNT;
    let block_in_stream = block_index / INTERLEAVED_STREAM_COUNT;
    let node = u32::try_from(stream / SEGMENTS_PER_NODE).expect("节点号小于 8");
    let segment = u32::try_from(stream % SEGMENTS_PER_NODE).expect("段号小于 8");
    let mut key = [0u8; BLOCK_KEY_BYTES];
    key[0..4].copy_from_slice(&node.to_be_bytes());
    key[4..8].copy_from_slice(&segment.to_be_bytes());
    key[8..16].copy_from_slice(&(block_in_stream * states_per_block).to_be_bytes());
    key[16..32].copy_from_slice(fingerprint);
    key
}

/// 经网络那一路写交错次序里 j 为奇数的块（登记 5.5 S3-T 一行）：第 p 个网络块是 j = 2p + 1。
pub(crate) fn network_path_block_index(network_position: u64) -> u64 {
    2 * network_position + 1
}

pub(crate) fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ============================== 帧 ==============================

pub(crate) fn encode_block_frame(key: &BlockKey, value: &[u8]) -> Vec<u8> {
    let value_length = u32::try_from(value.len()).expect("块值装得进 u32");
    let mut frame = Vec::with_capacity(BLOCK_FRAME_HEADER_BYTES + value.len() + FRAME_CHECKSUM_BYTES);
    frame.extend_from_slice(BLOCK_FRAME_MAGIC);
    frame.extend_from_slice(key);
    frame.extend_from_slice(&value_length.to_le_bytes());
    frame.extend_from_slice(value);
    let checksum = block_frame_checksum(key, value_length, value);
    frame.extend_from_slice(&checksum.to_le_bytes());
    frame
}

/// 帧的 CRC-32C 覆盖键、长度与块值（登记 6.3 末段）。
pub(crate) fn block_frame_checksum(key: &BlockKey, value_length: u32, value: &[u8]) -> u32 {
    crc32c(&[key, &value_length.to_le_bytes(), value])
}

#[cfg_attr(not(test), allow(dead_code, reason = "送块进程只解确认帧；编确认帧的是库进程（e162_verdict_store_network.rs 经 #[path] 引这一份）与这里的单测"))]
pub(crate) fn encode_acknowledgement_frame(key: &BlockKey) -> [u8; ACKNOWLEDGEMENT_FRAME_BYTES] {
    let mut frame = [0u8; ACKNOWLEDGEMENT_FRAME_BYTES];
    frame[0..4].copy_from_slice(ACKNOWLEDGEMENT_FRAME_MAGIC);
    frame[4..4 + BLOCK_KEY_BYTES].copy_from_slice(key);
    frame[4 + BLOCK_KEY_BYTES..].copy_from_slice(&crc32c(&[key]).to_le_bytes());
    frame
}

/// 魔数或 CRC 不对返回 None。
pub(crate) fn decode_acknowledgement_frame(frame: &[u8; ACKNOWLEDGEMENT_FRAME_BYTES]) -> Option<BlockKey> {
    if &frame[0..4] != ACKNOWLEDGEMENT_FRAME_MAGIC {
        return None;
    }
    let key: BlockKey = frame[4..4 + BLOCK_KEY_BYTES].try_into().expect("32 字节");
    let stored_checksum = u32::from_le_bytes(frame[4 + BLOCK_KEY_BYTES..].try_into().expect("4 字节"));
    (crc32c(&[&key]) == stored_checksum).then_some(key)
}

/// 读满整个缓冲区，或者读到 EOF 为止；返回读到的字节数（小于缓冲区长度就是在中途断了）。
pub(crate) fn read_until_full_or_end(reader: &mut dyn Read, destination: &mut [u8]) -> usize {
    let mut filled = 0;
    while filled < destination.len() {
        match reader.read(&mut destination[filled..]) {
            Ok(0) => break,
            Ok(read_bytes) => filled += read_bytes,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_connection_error) => break,
        }
    }
    filled
}

// ============================== 锚点（登记 7.2 A1–A6、A11，补 7 B10、B10b） ==============================

pub(crate) struct AnchorCheck {
    pub(crate) identifier: String,
    pub(crate) computed: String,
    pub(crate) registered: &'static str,
}

impl AnchorCheck {
    pub(crate) fn matches(&self) -> bool {
        self.computed == self.registered
    }
}

/// PC3-rate-X 的上下界（补 7 B10）：上界 = 块数 ÷ ((块数 − 1) × 节拍)，下界再容最后一个确认晚到 2.2 秒。
pub(crate) fn cross_rate_control_bounds() -> (f64, f64) {
    let paced_milliseconds = (CROSS_RATE_CONTROL_BLOCKS - 1) * CROSS_RATE_CONTROL_PACE_MILLISECONDS;
    let upper = CROSS_RATE_CONTROL_BLOCKS as f64 / (paced_milliseconds as f64 / 1_000.0);
    let lower = CROSS_RATE_CONTROL_BLOCKS as f64 / ((paced_milliseconds + CROSS_RATE_CONTROL_LATE_ACKNOWLEDGEMENT_MILLISECONDS) as f64 / 1_000.0);
    (upper, lower)
}

pub(crate) fn anchor_checks() -> Vec<AnchorCheck> {
    let mut checks = Vec::new();
    let mut record = |identifier: &str, computed: String, registered: &'static str| {
        checks.push(AnchorCheck { identifier: identifier.to_string(), computed, registered });
    };
    let mut zero_generator = SplitMix64::new(0);
    let first_three: Vec<String> = (0..3).map(|_| format!("{:#018x}", zero_generator.next_output())).collect();
    record("A1", first_three.join(","), "0xe220a8397b1dcdaf,0x6e789e6aa1b965f4,0x06c45d188009454f");
    record("A2", format!("{:#010x}", crc32c(&[b"123456789"])), "0xe3069283");
    for (block_index, registered) in [
        (0u64, "76020f1b59444d587a6607195c0f8fed/8351809/0x08eefd5c"),
        (12_345, "b3c1f0df0c77c9e98c9df5b21d0395bf/8321879/0x565d4801"),
    ] {
        let value = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        let byte_sum: u64 = value.iter().map(|&byte| u64::from(byte)).sum();
        let summary = format!("{}/{byte_sum}/{:#010x}", hexadecimal(&value[..16]), crc32c(&[&value]));
        record(&format!("A3_j{block_index}"), summary, registered);
    }
    let small = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 0, SMALL_STATES_PER_BLOCK);
    let small_sum: u64 = small.iter().map(|&byte| u64::from(byte)).sum();
    record("A3b", format!("{}/{small_sum}", hexadecimal(&small[..16])), "76020f1b59444d587a6607195c0f8fed/131533");
    for (block_index, registered) in [(0u64, "267/32629"), (12_345, "245/28761")] {
        let value = block_value(ValuePattern::Sparse, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        let nonzero = value.iter().filter(|&&byte| byte != 0).count();
        let byte_sum: u64 = value.iter().map(|&byte| u64::from(byte)).sum();
        record(&format!("A4_j{block_index}"), format!("{nonzero}/{byte_sum}"), registered);
    }
    let fingerprint = input_fingerprint();
    record("A5", hexadecimal(&fingerprint), "7299025377b7594e92ceeb8a1c3cbeac");
    for (block_index, registered) in [
        (0u64, "000000000000000000000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (1, "000000000000000100000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (63, "000000070000000700000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (64, "000000000000000000000000000100007299025377b7594e92ceeb8a1c3cbeac"),
        (12_345, "00000007000000010000000000c000007299025377b7594e92ceeb8a1c3cbeac"),
        (99_999, "000000030000000700000000061a00007299025377b7594e92ceeb8a1c3cbeac"),
    ] {
        let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        record(&format!("A6_j{block_index}"), hexadecimal(&key), registered);
    }
    let small_key = interleaved_block_key(12_345, SMALL_STATES_PER_BLOCK, &fingerprint);
    record("A6b", hexadecimal(&small_key), "000000070000000100000000000300007299025377b7594e92ceeb8a1c3cbeac");
    let primary_value_bytes = usize::try_from(PRIMARY_STATES_PER_BLOCK).expect("65 536 装得进 usize");
    let frame = encode_block_frame(&small_key, &vec![0u8; primary_value_bytes]);
    record("A11_frame", frame.len().to_string(), "65580");
    let (upper, lower) = cross_rate_control_bounds();
    record("B10", format!("{upper}/{lower}"), "10.050251256281408/9.049773755656108");
    let frame_bytes = u64::try_from(frame.len()).expect("帧长装得进 u64");
    record("B10b", format!("{}/{}/{}", PATH_BLOCK_COUNT * frame_bytes, 64 * frame_bytes, 256 * frame_bytes), "3279000000/4197120/16788480");
    checks
}

// ============================== 送块 ==============================

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SenderEndpoint {
    /// 回环格：主动连 `<地址>:<端口>`（转发进程）。
    ConnectTcp(String),
    /// 跨机格：在这个 Unix 套接字上等中继连进来。
    ListenUnix(PathBuf),
}

#[derive(Debug, Clone)]
pub(crate) struct SenderSettings {
    pub(crate) endpoint: SenderEndpoint,
    pub(crate) block_count: u64,
    pub(crate) states_per_block: u64,
    /// 最多留几帧没收到确认（主 W = 64，敏感性 1 与 256）。
    pub(crate) window_frames: usize,
    /// PC3-rate-X：第 i 帧不早于第一帧之后 i × 节拍；None 是照常尽快送。
    pub(crate) pace_between_first_transmissions: Option<Duration>,
    pub(crate) acknowledgement_stall_limit: Duration,
    /// S3X-busy：另起这么多个不停算 SplitMix64 的线程。
    pub(crate) spinning_thread_count: usize,
}

impl SenderSettings {
    /// 与 `e162_verdict_store_network.rs` 的 `sender_arguments` 互为反过来（那边单测往返核）。
    pub(crate) fn parse(arguments: &[String]) -> Result<SenderSettings, String> {
        let mut values: HashMap<&str, &str> = HashMap::new();
        let mut position = 0;
        while position < arguments.len() {
            let flag = arguments[position].as_str();
            let value = arguments.get(position + 1).ok_or(format!("{flag} 后面缺值"))?;
            values.insert(flag, value.as_str());
            position += 2;
        }
        let number = |flag: &str, default: Option<u64>| -> Result<u64, String> {
            match (values.get(flag), default) {
                (Some(text), _) => text.parse().map_err(|_| format!("{flag} 不是数：{text}")),
                (None, Some(default_value)) => Ok(default_value),
                (None, None) => Err(format!("缺 {flag}")),
            }
        };
        let endpoint = match (values.get("--connect"), values.get("--listen")) {
            (Some(connect), None) => SenderEndpoint::ConnectTcp(
                connect.strip_prefix("tcp:").ok_or(format!("--connect 要写成 tcp:<地址>:<端口>，不是 {connect}"))?.to_string(),
            ),
            (None, Some(listen)) => SenderEndpoint::ListenUnix(PathBuf::from(
                listen.strip_prefix("unix:").ok_or(format!("--listen 要写成 unix:<路径>，不是 {listen}"))?,
            )),
            _ => return Err("--connect 与 --listen 恰好给一个".to_string()),
        };
        let pace_milliseconds = number("--pace-milliseconds", Some(0))?;
        Ok(SenderSettings {
            endpoint,
            block_count: number("--blocks", None)?,
            states_per_block: number("--states-per-block", None)?,
            window_frames: usize::try_from(number("--window", None)?).map_err(|_| "窗口装不进 usize".to_string())?,
            pace_between_first_transmissions: (pace_milliseconds > 0).then(|| Duration::from_millis(pace_milliseconds)),
            acknowledgement_stall_limit: Duration::from_secs(number("--stall-seconds", Some(DEFAULT_ACKNOWLEDGEMENT_STALL_SECONDS))?),
            spinning_thread_count: usize::try_from(number("--spin-threads", Some(0))?).map_err(|_| "线程数装不进 usize".to_string())?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SenderEnding {
    Complete,
    /// 连着而 acknowledgement_stall_limit 内没有新确认。
    AcknowledgementsStalled,
    /// 断了之后 30 秒内没再连上。
    ReconnectDeadlinePassed,
    AnchorMismatch,
}

impl SenderEnding {
    pub(crate) fn label(self) -> &'static str {
        match self {
            SenderEnding::Complete => "complete",
            SenderEnding::AcknowledgementsStalled => "acknowledgements_stalled",
            SenderEnding::ReconnectDeadlinePassed => "reconnect_deadline_passed",
            SenderEnding::AnchorMismatch => "anchor_mismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReconnectionRecord {
    pub(crate) acknowledged_at_disconnect: u64,
    pub(crate) reconnect_microseconds: u64,
    pub(crate) resent_frames: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SenderReport {
    pub(crate) ending: SenderEnding,
    pub(crate) acknowledged_count: u64,
    /// 第一次发出的帧数（重发不算）。
    pub(crate) first_transmissions: u64,
    pub(crate) resent_frames: u64,
    pub(crate) reconnections: Vec<ReconnectionRecord>,
    /// 一个块已经确认过、又收到它的确认。
    pub(crate) duplicate_acknowledgements: u64,
    /// 收到的确认不是任何一个已发出块的键。
    pub(crate) unexpected_acknowledgements: u64,
    pub(crate) invalid_acknowledgement_frames: u64,
    /// 第一帧发出到最后一个确认到达；一个确认都没有时是 0。
    pub(crate) first_frame_to_last_acknowledgement_microseconds: u64,
    /// 第 1 000、2 000 … 个确认到达的时刻（距第一帧发出）。
    pub(crate) window_end_offsets_microseconds: Vec<u64>,
    /// 收到过确认的网络块序号，写成闭区间并起来的样子（`0-49999`）；一个都没有是 `none`。
    pub(crate) acknowledged_ranges: String,
    pub(crate) outstanding_at_exit: u64,
    /// 至少发出过一次的网络块数（从 0 起连续）。
    pub(crate) sent_positions: u64,
}

impl SenderReport {
    pub(crate) fn summary_line(&self, settings: &SenderSettings) -> String {
        let pace_microseconds = settings
            .pace_between_first_transmissions
            .map_or(0, |pace| u64::try_from(pace.as_micros()).expect("节拍微秒数装得进 u64"));
        format!(
            "S ending={} acknowledged={} blocks={} first_transmissions={} resent_frames={} reconnections={} duplicate_acknowledgements={} unexpected_acknowledgements={} invalid_acknowledgement_frames={} elapsed_microseconds={} acknowledged_ranges={} outstanding_at_exit={} sent_positions={} window_frames={} states_per_block={} pace_microseconds={}",
            self.ending.label(),
            self.acknowledged_count,
            settings.block_count,
            self.first_transmissions,
            self.resent_frames,
            self.reconnections.len(),
            self.duplicate_acknowledgements,
            self.unexpected_acknowledgements,
            self.invalid_acknowledgement_frames,
            self.first_frame_to_last_acknowledgement_microseconds,
            self.acknowledged_ranges,
            self.outstanding_at_exit,
            self.sent_positions,
            settings.window_frames,
            settings.states_per_block,
            pace_microseconds,
        )
    }
}

/// 把一串布尔写成闭区间并起来的样子：`[真, 真, 假, 真]` → `0-1,3-3`。
pub(crate) fn compress_true_positions(flags: &[bool]) -> String {
    let mut ranges = Vec::new();
    let mut range_start: Option<usize> = None;
    for (position, &flag) in flags.iter().enumerate() {
        match (flag, range_start) {
            (true, None) => range_start = Some(position),
            (false, Some(start)) => {
                ranges.push(format!("{start}-{}", position - 1));
                range_start = None;
            }
            (true, Some(_)) | (false, None) => {}
        }
    }
    if let Some(start) = range_start {
        ranges.push(format!("{start}-{}", flags.len() - 1));
    }
    if ranges.is_empty() {
        "none".to_string()
    } else {
        ranges.join(",")
    }
}

/// 一条数据通道：回环格是 TCP，跨机格是 Unix 套接字。
enum Link {
    Tcp(TcpStream),
    Unix(UnixStream),
}

impl Link {
    fn reader(&self) -> std::io::Result<Box<dyn Read + Send>> {
        match self {
            Link::Tcp(stream) => Ok(Box::new(stream.try_clone()?)),
            Link::Unix(stream) => Ok(Box::new(stream.try_clone()?)),
        }
    }

    fn write_frame(&mut self, frame: &[u8]) -> std::io::Result<()> {
        match self {
            Link::Tcp(stream) => stream.write_all(frame),
            Link::Unix(stream) => stream.write_all(frame),
        }
    }

    fn close(&self) {
        let _already_closed = match self {
            Link::Tcp(stream) => stream.shutdown(std::net::Shutdown::Both),
            Link::Unix(stream) => stream.shutdown(std::net::Shutdown::Both),
        };
    }
}

/// 怎么（重新）接上：回环格主动连，跨机格等连进来。
enum Rendezvous {
    Tcp { address: String },
    Unix { listener: UnixListener },
}

impl Rendezvous {
    fn prepare(endpoint: &SenderEndpoint) -> std::io::Result<Rendezvous> {
        match endpoint {
            SenderEndpoint::ConnectTcp(address) => Ok(Rendezvous::Tcp { address: address.clone() }),
            SenderEndpoint::ListenUnix(path) => {
                let listener = UnixListener::bind(path)?;
                listener.set_nonblocking(true)?;
                Ok(Rendezvous::Unix { listener })
            }
        }
    }

    /// 每 10 ms 试一次，最多 30 秒。
    fn establish(&self) -> Option<Link> {
        let started = Instant::now();
        loop {
            match self {
                Rendezvous::Tcp { address } => {
                    if let Ok(stream) = TcpStream::connect(address.as_str()) {
                        stream.set_nodelay(true).expect("TCP 连接设得了 nodelay");
                        return Some(Link::Tcp(stream));
                    }
                }
                Rendezvous::Unix { listener } => {
                    if let Ok((stream, _peer_address)) = listener.accept() {
                        stream.set_nonblocking(false).expect("接进来的连接改得回阻塞");
                        return Some(Link::Unix(stream));
                    }
                }
            }
            if started.elapsed() >= RECONNECT_DEADLINE {
                return None;
            }
            std::thread::sleep(RECONNECT_INTERVAL);
        }
    }
}

enum LinkEvent {
    Acknowledged { key: BlockKey, arrived_at: Instant },
    InvalidAcknowledgement,
    Closed { link_generation: u64 },
}

/// 每条连接一个读确认的线程：确认到达的时刻在这里取（读到就取），交给主线程。
fn spawn_acknowledgement_reader(mut reader: Box<dyn Read + Send>, link_generation: u64, events: mpsc::Sender<LinkEvent>) {
    std::thread::spawn(move || {
        let mut frame = [0u8; ACKNOWLEDGEMENT_FRAME_BYTES];
        loop {
            let filled = read_until_full_or_end(reader.as_mut(), &mut frame);
            if filled < ACKNOWLEDGEMENT_FRAME_BYTES {
                let _receiver_gone = events.send(LinkEvent::Closed { link_generation });
                return;
            }
            let event = match decode_acknowledgement_frame(&frame) {
                Some(key) => LinkEvent::Acknowledged { key, arrived_at: Instant::now() },
                None => LinkEvent::InvalidAcknowledgement,
            };
            if events.send(event).is_err() {
                return;
            }
        }
    });
}

struct OutstandingFrame {
    position: u64,
    frame: Vec<u8>,
}

fn emit_line(output: &mut dyn Write, line: &str) {
    output.write_all(format!("{line}\n").as_bytes()).expect("写得进送块进程的输出");
    output.flush().expect("刷得出送块进程的输出");
}

fn elapsed_microseconds_between(earlier: Instant, later: Instant) -> u64 {
    u64::try_from(later.saturating_duration_since(earlier).as_micros()).expect("微秒数装得进 u64")
}

fn start_spinning_threads(thread_count: usize) {
    for thread_number in 0..thread_count {
        std::thread::spawn(move || {
            let mut generator = SplitMix64::new(u64::try_from(thread_number).expect("线程号装得进 u64"));
            loop {
                std::hint::black_box(generator.next_output());
            }
        });
    }
}

/// 送块的主循环。帧按网络块序号 0、1、2… 第一次发出；断了就重连、按原次序重发全部未确认的帧（登记 6.3 末段）。
pub(crate) fn run_sender(settings: &SenderSettings, output: &mut dyn Write) -> SenderReport {
    emit_line(output, &format!("P {}", std::process::id()));
    let block_count_as_index = usize::try_from(settings.block_count).expect("块数装得进 usize");
    let mut report = SenderReport {
        ending: SenderEnding::Complete,
        acknowledged_count: 0,
        first_transmissions: 0,
        resent_frames: 0,
        reconnections: Vec::new(),
        duplicate_acknowledgements: 0,
        unexpected_acknowledgements: 0,
        invalid_acknowledgement_frames: 0,
        first_frame_to_last_acknowledgement_microseconds: 0,
        window_end_offsets_microseconds: Vec::new(),
        acknowledged_ranges: "none".to_string(),
        outstanding_at_exit: 0,
        sent_positions: 0,
    };
    if anchor_checks().iter().any(|check| !check.matches()) {
        report.ending = SenderEnding::AnchorMismatch;
        return report;
    }
    start_spinning_threads(settings.spinning_thread_count);
    let fingerprint = input_fingerprint();
    let position_of_key: HashMap<BlockKey, u64> = (0..settings.block_count)
        .map(|position| (interleaved_block_key(network_path_block_index(position), settings.states_per_block, &fingerprint), position))
        .collect();
    let rendezvous = Rendezvous::prepare(&settings.endpoint).expect("送块进程的接法准备得好（Unix 套接字路径不许已存在）");
    let (event_sender, events) = mpsc::channel();
    let Some(mut link) = rendezvous.establish() else {
        report.ending = SenderEnding::ReconnectDeadlinePassed;
        return report;
    };
    let mut link_generation = 0u64;
    spawn_acknowledgement_reader(link.reader().expect("连接复制得出读的一端"), link_generation, event_sender.clone());
    let mut acknowledged = vec![false; block_count_as_index];
    let mut outstanding: VecDeque<OutstandingFrame> = VecDeque::new();
    let mut next_position = 0u64;
    let mut first_frame_at: Option<Instant> = None;
    let mut last_acknowledgement_at: Option<Instant> = None;
    let mut is_connected = true;
    loop {
        if next_position == settings.block_count && outstanding.is_empty() {
            break;
        }
        while is_connected && outstanding.len() < settings.window_frames && next_position < settings.block_count {
            if let (Some(pace), Some(first)) = (settings.pace_between_first_transmissions, first_frame_at) {
                let paced_frames = u32::try_from(next_position).expect("压节奏的帧号装得进 u32");
                let not_before = first + pace * paced_frames;
                let now = Instant::now();
                if not_before > now {
                    std::thread::sleep(not_before - now);
                }
            }
            let block_index = network_path_block_index(next_position);
            let key = interleaved_block_key(block_index, settings.states_per_block, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, settings.states_per_block);
            let frame = encode_block_frame(&key, &value);
            if first_frame_at.is_none() {
                first_frame_at = Some(Instant::now());
            }
            let write_outcome = link.write_frame(&frame);
            outstanding.push_back(OutstandingFrame { position: next_position, frame });
            next_position += 1;
            report.first_transmissions += 1;
            if write_outcome.is_err() {
                is_connected = false;
            }
        }
        if !is_connected {
            let disconnected_at = Instant::now();
            let acknowledged_at_disconnect = report.acknowledged_count;
            link.close();
            let Some(new_link) = rendezvous.establish() else {
                report.ending = SenderEnding::ReconnectDeadlinePassed;
                break;
            };
            link = new_link;
            let reconnected_at = Instant::now();
            link_generation += 1;
            spawn_acknowledgement_reader(link.reader().expect("连接复制得出读的一端"), link_generation, event_sender.clone());
            is_connected = true;
            let frames_to_resend: Vec<Vec<u8>> = outstanding.iter().map(|pending| pending.frame.clone()).collect();
            let mut resent_this_time = 0u64;
            for frame in &frames_to_resend {
                if link.write_frame(frame).is_err() {
                    is_connected = false;
                    break;
                }
                resent_this_time += 1;
            }
            report.resent_frames += resent_this_time;
            let record = ReconnectionRecord {
                acknowledged_at_disconnect,
                reconnect_microseconds: elapsed_microseconds_between(disconnected_at, reconnected_at),
                resent_frames: resent_this_time,
            };
            emit_line(
                output,
                &format!(
                    "R index={} acknowledged_at_disconnect={} reconnect_microseconds={} resent_frames={}",
                    report.reconnections.len(),
                    record.acknowledged_at_disconnect,
                    record.reconnect_microseconds,
                    record.resent_frames
                ),
            );
            report.reconnections.push(record);
            continue;
        }
        let first_event = match events.recv_timeout(settings.acknowledgement_stall_limit) {
            Ok(event) => event,
            Err(_stalled_or_gone) => {
                report.ending = SenderEnding::AcknowledgementsStalled;
                break;
            }
        };
        let mut pending_events = vec![first_event];
        while let Ok(event) = events.try_recv() {
            pending_events.push(event);
        }
        for event in pending_events {
            match event {
                LinkEvent::Acknowledged { key, arrived_at } => {
                    let Some(&position) = position_of_key.get(&key) else {
                        report.unexpected_acknowledgements += 1;
                        continue;
                    };
                    let position_as_index = usize::try_from(position).expect("块序号装得进 usize");
                    if let Some(queue_index) = outstanding.iter().position(|pending| pending.position == position) {
                        outstanding.remove(queue_index);
                    }
                    if acknowledged[position_as_index] {
                        report.duplicate_acknowledgements += 1;
                        continue;
                    }
                    if position >= next_position {
                        report.unexpected_acknowledgements += 1;
                        continue;
                    }
                    acknowledged[position_as_index] = true;
                    report.acknowledged_count += 1;
                    last_acknowledgement_at = Some(arrived_at);
                    if report.acknowledged_count.is_multiple_of(ACKNOWLEDGEMENTS_PER_WINDOW) {
                        let first = first_frame_at.expect("收到确认之前已经发过第一帧");
                        report.window_end_offsets_microseconds.push(elapsed_microseconds_between(first, arrived_at));
                    }
                    emit_line(output, &format!("A {}", report.acknowledged_count));
                }
                LinkEvent::InvalidAcknowledgement => report.invalid_acknowledgement_frames += 1,
                LinkEvent::Closed { link_generation: closed_generation } => {
                    if closed_generation == link_generation {
                        is_connected = false;
                    }
                }
            }
        }
    }
    link.close();
    if let (Some(first), Some(last)) = (first_frame_at, last_acknowledgement_at) {
        report.first_frame_to_last_acknowledgement_microseconds = elapsed_microseconds_between(first, last);
    }
    report.acknowledged_ranges = compress_true_positions(&acknowledged);
    report.outstanding_at_exit = u64::try_from(outstanding.len()).expect("未确认帧数装得进 u64");
    report.sent_positions = next_position;
    let offsets: Vec<String> = report.window_end_offsets_microseconds.iter().map(u64::to_string).collect();
    emit_line(output, &format!("W end_offsets_microseconds={}", if offsets.is_empty() { "none".to_string() } else { offsets.join(",") }));
    emit_line(output, &report.summary_line(settings));
    if let SenderEndpoint::ListenUnix(path) = &settings.endpoint {
        let _already_removed = std::fs::remove_file(path);
    }
    report
}

fn sender_exit_code(ending: SenderEnding) -> i32 {
    match ending {
        SenderEnding::Complete => 0,
        SenderEnding::AcknowledgementsStalled => 3,
        SenderEnding::ReconnectDeadlinePassed => 4,
        SenderEnding::AnchorMismatch => 5,
    }
}

pub(crate) fn run_send_role(arguments: &[String]) -> i32 {
    let settings = match SenderSettings::parse(arguments) {
        Ok(settings) => settings,
        Err(problem) => {
            eprintln!("  ✗ send 的参数不对：{problem}");
            return 2;
        }
    };
    let mut standard_output = std::io::stdout().lock();
    let report = run_sender(&settings, &mut standard_output);
    sender_exit_code(report.ending)
}

/// 中继：连上 Unix 套接字（每 10 ms 试一次、最多 30 秒），标准输入 → 套接字、套接字 → 标准输出；任一头断了就退出。
pub(crate) fn run_relay_role(arguments: &[String]) -> i32 {
    let (Some(flag), Some(target)) = (arguments.first(), arguments.get(1)) else {
        eprintln!("  ✗ relay 要 --relay unix:<路径>");
        return 2;
    };
    let Some(path) = target.strip_prefix("unix:").filter(|_| flag == "--relay") else {
        eprintln!("  ✗ relay 要 --relay unix:<路径>，不是 {flag} {target}");
        return 2;
    };
    relay_between(path, Box::new(std::io::stdout()))
}

/// 中继的主体：套接字 → data_output（正式跑是标准输出），标准输入 → 套接字。
pub(crate) fn relay_between(path: &str, mut data_output: Box<dyn Write + Send>) -> i32 {
    let started = Instant::now();
    let stream = loop {
        if let Ok(stream) = UnixStream::connect(path) {
            break stream;
        }
        if started.elapsed() >= RECONNECT_DEADLINE {
            eprintln!("  ✗ relay 30 秒内连不上 {path}");
            return 4;
        }
        std::thread::sleep(RECONNECT_INTERVAL);
    };
    let mut socket_reader = stream.try_clone().expect("套接字复制得出读的一端");
    std::thread::spawn(move || {
        let mut chunk = vec![0u8; 65_536];
        loop {
            match socket_reader.read(&mut chunk) {
                Ok(0) | Err(_) => std::process::exit(0),
                Ok(read_bytes) => {
                    if data_output.write_all(&chunk[..read_bytes]).and_then(|()| data_output.flush()).is_err() {
                        std::process::exit(0);
                    }
                }
            }
        }
    });
    let mut socket_writer = stream;
    let mut chunk = vec![0u8; 65_536];
    let mut standard_input = std::io::stdin().lock();
    loop {
        match standard_input.read(&mut chunk) {
            Ok(0) | Err(_) => {
                let _already_closed = socket_writer.shutdown(std::net::Shutdown::Both);
                return 0;
            }
            Ok(read_bytes) => {
                if socket_writer.write_all(&chunk[..read_bytes]).is_err() {
                    return 0;
                }
            }
        }
    }
}

pub(crate) fn run_sender_anchor_role() -> i32 {
    let checks = anchor_checks();
    let mut mismatches = 0usize;
    for check in &checks {
        if !check.matches() {
            mismatches += 1;
        }
        println!("anchor id={} value={} matches_registration={}", check.identifier, check.computed, check.matches());
    }
    println!("anchors checked={} mismatches={mismatches}", checks.len());
    i32::from(mismatches > 0)
}

/// 送块、中继两个角色是 e162-verdict-store-network 起的子进程（跨机格里在另一台上跑，那里没有仓），不判准入；
/// 手跑 `anchors` 时照 `.claude/singlefs-ai-sop/rules/preflight-discipline.md` 判。
fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().is_some_and(|role| role == "send" || role == "relay") {
        return;
    }
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e162_verdict_store_sender.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn main() {
    preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    let remaining: Vec<String> = arguments.iter().skip(1).cloned().collect();
    let exit_code = match arguments.first().map(String::as_str) {
        Some("send") => run_send_role(&remaining),
        Some("relay") => run_relay_role(&remaining),
        Some("anchors") => run_sender_anchor_role(),
        _ => {
            eprintln!("  ✗ 用法：e162-verdict-store-sender send (--connect tcp:<地址>:<端口> | --listen unix:<路径>) --blocks N --states-per-block S --window W [--pace-milliseconds P] [--stall-seconds T] [--spin-threads K]");
            eprintln!("          e162-verdict-store-sender relay --relay unix:<路径>");
            eprintln!("          e162-verdict-store-sender anchors");
            eprintln!("  → 怎么办：这个 bin 由 e162-verdict-store-network 起，平常不手跑");
            2
        }
    };
    std::process::exit(exit_code);
}

// ============================== 单测 ==============================

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    #[test]
    fn every_registered_anchor_matches_and_there_are_eighteen_of_them() {
        let checks = anchor_checks();
        for check in &checks {
            assert!(check.matches(), "锚点 {} 算出 {}，登记是 {}", check.identifier, check.computed, check.registered);
        }
        assert_eq!(checks.len(), 18, "A1、A2、A3 两块、A3b、A4 两块、A5、A6 六块、A6b、A11_frame、B10、B10b");
    }

    #[test]
    fn block_frame_is_65580_bytes_and_its_header_carries_key_and_little_endian_length() {
        let fingerprint = input_fingerprint();
        let key = interleaved_block_key(network_path_block_index(0), PRIMARY_STATES_PER_BLOCK, &fingerprint);
        let value = block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, 1, PRIMARY_STATES_PER_BLOCK);
        let frame = encode_block_frame(&key, &value);
        assert_eq!(frame.len(), 65_580);
        assert_eq!(
            hexadecimal(&frame[..BLOCK_FRAME_HEADER_BYTES]),
            "5346424b000000000000000100000000000000007299025377b7594e92ceeb8a1c3cbeac00000100"
        );
        let stored = u32::from_le_bytes(frame[frame.len() - 4..].try_into().expect("4 字节"));
        assert_eq!(stored, block_frame_checksum(&key, 65_536, &value));
        let mut flipped = value.clone();
        flipped[40_000] ^= 0x5A;
        assert_ne!(stored, block_frame_checksum(&key, 65_536, &flipped), "块值翻一个字节，CRC 要变");
        let small = encode_block_frame(&key, &vec![0u8; 1_024]);
        assert_eq!(small.len(), 1_068);
    }

    #[test]
    fn acknowledgement_frame_round_trips_and_rejects_a_flipped_byte_or_wrong_magic() {
        let key = interleaved_block_key(12_345, PRIMARY_STATES_PER_BLOCK, &input_fingerprint());
        let frame = encode_acknowledgement_frame(&key);
        assert_eq!(frame.len(), 40);
        assert_eq!(decode_acknowledgement_frame(&frame), Some(key));
        let mut flipped = frame;
        flipped[10] ^= 1;
        assert_eq!(decode_acknowledgement_frame(&flipped), None);
        let mut wrong_magic = frame;
        wrong_magic[0] = b'X';
        assert_eq!(decode_acknowledgement_frame(&wrong_magic), None);
    }

    #[test]
    fn acknowledged_positions_compress_into_closed_ranges() {
        assert_eq!(compress_true_positions(&[true, true, false, true, false, false, true]), "0-1,3-3,6-6");
        assert_eq!(compress_true_positions(&[false, false]), "none");
        assert_eq!(compress_true_positions(&vec![true; 50_000]), "0-49999");
    }

    #[test]
    fn sender_settings_parse_reads_every_flag_and_defaults() {
        let arguments: Vec<String> = ["--listen", "unix:/tmp/x/s.sock", "--blocks", "200", "--states-per-block", "1024", "--window", "1"]
            .iter()
            .map(|text| text.to_string())
            .collect();
        let parsed = SenderSettings::parse(&arguments).expect("认得出");
        assert_eq!(parsed.endpoint, SenderEndpoint::ListenUnix(PathBuf::from("/tmp/x/s.sock")));
        assert_eq!(parsed.block_count, 200);
        assert_eq!(parsed.states_per_block, 1_024);
        assert_eq!(parsed.window_frames, 1);
        assert_eq!(parsed.pace_between_first_transmissions, None);
        assert_eq!(parsed.acknowledgement_stall_limit, Duration::from_secs(120));
        assert_eq!(parsed.spinning_thread_count, 0);
        assert!(SenderSettings::parse(&arguments[2..]).is_err(), "--connect 与 --listen 一个都没给要拒");
    }

    /// 替身库：收帧、校验 CRC、记下键与块值、回确认。断线计划照「杀中继」的样子：
    /// 回出第 a 个确认之后，再整收一帧（记下、不回确认：确认丢在路上），再收半帧（丢掉），然后断开。
    #[derive(Default)]
    struct StandInLibraryRecord {
        stored: HashMap<BlockKey, Vec<u8>>,
        acknowledgements_sent: u64,
        half_frames_dropped: u64,
        disconnects: u64,
        largest_unacknowledged: u64,
        checksum_failures: u64,
    }

    enum StandInLink {
        Tcp(TcpStream),
        Unix(UnixStream),
    }

    impl StandInLink {
        fn split(self) -> (Box<dyn Read>, Box<dyn Write>) {
            match self {
                StandInLink::Tcp(stream) => {
                    stream.set_nodelay(true).expect("nodelay");
                    (Box::new(stream.try_clone().expect("复制")), Box::new(stream))
                }
                StandInLink::Unix(stream) => (Box::new(stream.try_clone().expect("复制")), Box::new(stream)),
            }
        }
    }

    /// 读一整帧并按替身库自己的写法（不走 `encode_block_frame` 的反过来）核魔数与 CRC；对面在帧边界上关了返回 None。
    fn read_stand_in_frame(reader: &mut dyn Read) -> Option<(BlockKey, Vec<u8>, bool)> {
        let mut header = [0u8; BLOCK_FRAME_HEADER_BYTES];
        let header_bytes = read_until_full_or_end(reader, &mut header);
        if header_bytes == 0 {
            return None;
        }
        assert_eq!(header_bytes, BLOCK_FRAME_HEADER_BYTES, "替身库只在帧边界上被对面断开");
        assert_eq!(&header[0..4], b"SFBK");
        let key: BlockKey = header[4..36].try_into().expect("32 字节");
        let value_length = usize::try_from(u32::from_le_bytes(header[36..40].try_into().expect("4 字节"))).expect("长度");
        let mut rest = vec![0u8; value_length + 4];
        let rest_bytes = read_until_full_or_end(reader, &mut rest);
        assert_eq!(rest_bytes, rest.len(), "送块进程不在帧中间停");
        let (value, checksum_bytes) = rest.split_at(value_length);
        let stored_checksum = u32::from_le_bytes(checksum_bytes.try_into().expect("4 字节"));
        let is_checksum_correct = crc32c(&[&header[4..40], value]) == stored_checksum;
        Some((key, value.to_vec(), is_checksum_correct))
    }

    /// 服务一条连接；disconnect_after 给了就在回出这么多个（累计）确认之后照上面的样子断开。返回是不是按计划断的。
    fn serve_stand_in_connection(link: StandInLink, record: &Arc<Mutex<StandInLibraryRecord>>, disconnect_after: Option<u64>) -> bool {
        let (mut reader, mut writer) = link.split();
        let mut is_withholding = false;
        loop {
            if is_withholding {
                // 半帧：只读下一帧的帧头就断开，这一帧丢掉。
                let mut header = [0u8; BLOCK_FRAME_HEADER_BYTES];
                let _partial = read_until_full_or_end(reader.as_mut(), &mut header);
                let mut guard = record.lock().expect("锁");
                guard.half_frames_dropped += 1;
                guard.disconnects += 1;
                return true;
            }
            let Some((key, value, is_checksum_correct)) = read_stand_in_frame(reader.as_mut()) else { return false };
            {
                let mut guard = record.lock().expect("锁");
                if !is_checksum_correct {
                    guard.checksum_failures += 1;
                    continue;
                }
                guard.stored.insert(key, value);
            }
            let total_sent = record.lock().expect("锁").acknowledgements_sent;
            if disconnect_after.is_some_and(|threshold| total_sent >= threshold) {
                // 这一帧已经记下，确认丢在路上；再收半帧就断。
                is_withholding = true;
                continue;
            }
            if writer.write_all(&encode_acknowledgement_frame(&key)).is_err() {
                return false;
            }
            record.lock().expect("锁").acknowledgements_sent += 1;
        }
    }

    /// 窗口单测用：一口气收帧、不回确认，直到 100 ms 里没有新帧（送块进程被窗口挡住了），
    /// 记下这一口收了几帧，再把这几帧的确认一起回出去；最大的一口就是送块进程留着没确认的帧数。
    fn serve_stand_in_in_bursts(stream: TcpStream, record: &Arc<Mutex<StandInLibraryRecord>>) {
        stream.set_nodelay(true).expect("nodelay");
        let mut writer = stream.try_clone().expect("复制");
        let mut reader = stream;
        loop {
            reader.set_read_timeout(None).expect("设得了读超时");
            let Some(first) = read_stand_in_frame(&mut reader) else { return };
            let mut burst = vec![first];
            reader.set_read_timeout(Some(Duration::from_millis(100))).expect("设得了读超时");
            loop {
                let mut probe = [0u8; 1];
                match reader.peek(&mut probe) {
                    Ok(0) => break,
                    Ok(_available) => {
                        // 帧头已经到了就把整帧读完，不让 100 ms 的探测超时切在帧中间（机器忙时 64 KiB 的帧体会晚到）
                        reader.set_read_timeout(None).expect("设得了读超时");
                        let frame = read_stand_in_frame(&mut reader);
                        reader.set_read_timeout(Some(Duration::from_millis(100))).expect("设得了读超时");
                        match frame {
                            Some(frame) => burst.push(frame),
                            None => break,
                        }
                    }
                    Err(_timed_out) => break,
                }
            }
            let mut guard = record.lock().expect("锁");
            guard.largest_unacknowledged = guard.largest_unacknowledged.max(u64::try_from(burst.len()).expect("帧数"));
            for (key, value, is_checksum_correct) in burst {
                assert!(is_checksum_correct);
                writer.write_all(&encode_acknowledgement_frame(&key)).expect("回得出确认");
                guard.acknowledgements_sent += 1;
                guard.stored.insert(key, value);
            }
        }
    }

    /// 替身库等送块进程连进来，最多等 30 秒：送块进程提前退了（例如锚点对不上），替身库线程也不永远挂着。
    fn accept_within_deadline(listener: &TcpListener) -> Option<TcpStream> {
        listener.set_nonblocking(true).expect("设得了非阻塞");
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(30) {
            if let Ok((stream, _peer_address)) = listener.accept() {
                stream.set_nonblocking(false).expect("改得回阻塞");
                return Some(stream);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        None
    }

    fn stand_in_settings(endpoint: SenderEndpoint, block_count: u64, window_frames: usize) -> SenderSettings {
        SenderSettings {
            endpoint,
            block_count,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            window_frames,
            pace_between_first_transmissions: None,
            acknowledgement_stall_limit: Duration::from_secs(3),
            spinning_thread_count: 0,
        }
    }

    fn expected_network_blocks(block_count: u64) -> HashMap<BlockKey, Vec<u8>> {
        let fingerprint = input_fingerprint();
        (0..block_count)
            .map(|position| {
                let block_index = network_path_block_index(position);
                (
                    interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint),
                    block_value(ValuePattern::Random, SEED_NETWORK_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK),
                )
            })
            .collect()
    }

    fn output_lines(output: &[u8]) -> Vec<String> {
        String::from_utf8_lossy(output).lines().map(str::to_string).collect()
    }

    /// 回环格的接法：替身库在 TCP 上等，送块进程连过来；断线计划里每一项断一次。
    fn run_against_tcp_stand_in(
        block_count: u64,
        window_frames: usize,
        disconnect_plan: Vec<u64>,
    ) -> (SenderReport, Arc<Mutex<StandInLibraryRecord>>, Vec<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("绑得上回环端口");
        let address = listener.local_addr().expect("有地址").to_string();
        let record = Arc::new(Mutex::new(StandInLibraryRecord::default()));
        let library_record = Arc::clone(&record);
        let library = std::thread::spawn(move || {
            let mut plan = disconnect_plan.into_iter();
            let mut next_disconnect = plan.next();
            while let Some(stream) = accept_within_deadline(&listener) {
                let disconnected_on_plan = serve_stand_in_connection(StandInLink::Tcp(stream), &library_record, next_disconnect);
                if disconnected_on_plan {
                    next_disconnect = plan.next();
                } else {
                    return;
                }
            }
        });
        let settings = stand_in_settings(SenderEndpoint::ConnectTcp(address), block_count, window_frames);
        let mut output = Vec::new();
        let report = run_sender(&settings, &mut output);
        library.join().expect("替身库线程没 panic");
        (report, record, output_lines(&output))
    }

    /// 跨机格的接法：送块进程在 Unix 套接字上等，替身库像中继那样连进去；断线计划里每一项断一次（等于杀中继）。
    fn run_against_unix_stand_in(
        block_count: u64,
        window_frames: usize,
        disconnect_plan: Vec<u64>,
    ) -> (SenderReport, Arc<Mutex<StandInLibraryRecord>>, Vec<String>) {
        let socket_directory = std::env::temp_dir().join(format!("e162-sender-test-{}-{}", std::process::id(), disconnect_plan.len()));
        let _stale = std::fs::remove_dir_all(&socket_directory);
        std::fs::create_dir_all(&socket_directory).expect("建得了套接字目录");
        let socket_path = socket_directory.join("s.sock");
        let settings = stand_in_settings(SenderEndpoint::ListenUnix(socket_path.clone()), block_count, window_frames);
        let record = Arc::new(Mutex::new(StandInLibraryRecord::default()));
        let library_record = Arc::clone(&record);
        let library = std::thread::spawn(move || {
            let mut plan = disconnect_plan.into_iter();
            let mut next_disconnect = plan.next();
            let started = Instant::now();
            loop {
                let Ok(stream) = UnixStream::connect(&socket_path) else {
                    if started.elapsed() > Duration::from_secs(60) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                };
                let disconnected_on_plan = serve_stand_in_connection(StandInLink::Unix(stream), &library_record, next_disconnect);
                if disconnected_on_plan {
                    next_disconnect = plan.next();
                } else {
                    return;
                }
            }
        });
        let mut output = Vec::new();
        let report = run_sender(&settings, &mut output);
        library.join().expect("替身库线程没 panic");
        let _cleanup = std::fs::remove_dir_all(&socket_directory);
        (report, record, output_lines(&output))
    }

    fn assert_every_block_delivered_once_acknowledged(report: &SenderReport, record: &StandInLibraryRecord, block_count: u64) {
        assert_eq!(report.ending, SenderEnding::Complete, "送块进程要送完；卡住说明断线之后没重发未确认的帧");
        assert_eq!(report.acknowledged_count, block_count);
        assert_eq!(report.outstanding_at_exit, 0);
        assert_eq!(report.acknowledged_ranges, format!("0-{}", block_count - 1));
        let expected = expected_network_blocks(block_count);
        let missing = expected.keys().filter(|key| !record.stored.contains_key(*key)).count();
        let differing = expected.iter().filter(|(key, value)| record.stored.get(*key).is_some_and(|found| found != *value)).count();
        assert_eq!(missing, 0, "确认过的块替身库里都要有");
        assert_eq!(differing, 0);
        assert_eq!(record.stored.len(), expected.len());
        assert_eq!(record.checksum_failures, 0);
    }

    #[test]
    fn tcp_sender_resends_unacknowledged_frames_after_each_of_five_disconnects() {
        let (report, record, lines) = run_against_tcp_stand_in(400, 64, vec![40, 110, 170, 260, 330]);
        let record = record.lock().expect("锁");
        assert_every_block_delivered_once_acknowledged(&report, &record, 400);
        assert_eq!(report.reconnections.len(), 5);
        assert_eq!(record.disconnects, 5);
        assert_eq!(record.half_frames_dropped, 5);
        assert!(report.resent_frames >= 10, "每次断线至少重发确认丢掉的那帧与半帧，五次至少 10 帧，实际 {}", report.resent_frames);
        assert_eq!(report.first_transmissions, 400);
        assert_eq!(lines.first().map(|line| line.starts_with("P ")), Some(true));
        assert_eq!(lines.iter().filter(|line| line.starts_with("A ")).count(), 400);
        assert_eq!(lines.iter().filter(|line| line.starts_with("R ")).count(), 5);
        assert!(lines.last().is_some_and(|line| line.starts_with("S ending=complete acknowledged=400 ")));
    }

    #[test]
    fn unix_listening_sender_resends_after_its_relay_is_killed_five_times() {
        let (report, record, _lines) = run_against_unix_stand_in(300, 64, vec![30, 90, 150, 200, 250]);
        let record = record.lock().expect("锁");
        assert_every_block_delivered_once_acknowledged(&report, &record, 300);
        assert_eq!(report.reconnections.len(), 5);
        assert_eq!(record.half_frames_dropped, 5);
        assert!(report.resent_frames >= 10, "实际重发 {}", report.resent_frames);
    }

    #[test]
    fn window_bounds_unacknowledged_frames_at_one_and_at_four() {
        for window_frames in [1usize, 4] {
            let listener = TcpListener::bind("127.0.0.1:0").expect("绑得上回环端口");
            let address = listener.local_addr().expect("有地址").to_string();
            let record = Arc::new(Mutex::new(StandInLibraryRecord::default()));
            let library_record = Arc::clone(&record);
            let library = std::thread::spawn(move || {
                if let Some(stream) = accept_within_deadline(&listener) {
                    serve_stand_in_in_bursts(stream, &library_record);
                }
            });
            let settings = stand_in_settings(SenderEndpoint::ConnectTcp(address), 60, window_frames);
            let mut output = Vec::new();
            let report = run_sender(&settings, &mut output);
            library.join().expect("替身库线程没 panic");
            let record = record.lock().expect("锁");
            assert_every_block_delivered_once_acknowledged(&report, &record, 60);
            assert_eq!(
                record.largest_unacknowledged,
                u64::try_from(window_frames).expect("窗口"),
                "窗口 {window_frames}：替身库看到的未确认帧数最多正好是窗口"
            );
        }
    }

    #[test]
    fn sender_that_never_hears_an_acknowledgement_stops_after_the_stall_limit_with_the_window_outstanding() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("绑得上回环端口");
        let address = listener.local_addr().expect("有地址").to_string();
        let silent = std::thread::spawn(move || {
            let Some(mut stream) = accept_within_deadline(&listener) else { return };
            let mut sink = vec![0u8; 1 << 20];
            while matches!(stream.read(&mut sink), Ok(read_bytes) if read_bytes > 0) {}
        });
        let mut settings = stand_in_settings(SenderEndpoint::ConnectTcp(address), 100, 8);
        settings.acknowledgement_stall_limit = Duration::from_secs(1);
        let mut output = Vec::new();
        let started = Instant::now();
        let report = run_sender(&settings, &mut output);
        assert_eq!(report.ending, SenderEnding::AcknowledgementsStalled);
        assert_eq!(report.acknowledged_count, 0);
        assert_eq!(report.outstanding_at_exit, 8);
        assert_eq!(report.acknowledged_ranges, "none");
        assert!(started.elapsed() >= Duration::from_secs(1));
        silent.join().expect("沉默的替身库线程没 panic");
    }

    /// 登记写死的 PC3-rate-X 判定区间 [9.0, 10.051] 块/秒（补 5.1、补 11 V26）；跨机格的驱动这一段没写，只有这条单测用它。
    const CROSS_RATE_CONTROL_LOWEST_BLOCKS_PER_SECOND: f64 = 9.0;
    const CROSS_RATE_CONTROL_HIGHEST_BLOCKS_PER_SECOND: f64 = 10.051;

    fn paced_run_blocks_per_second(pace: Option<Duration>) -> f64 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("绑得上回环端口");
        let address = listener.local_addr().expect("有地址").to_string();
        let record = Arc::new(Mutex::new(StandInLibraryRecord::default()));
        let library_record = Arc::clone(&record);
        let library = std::thread::spawn(move || {
            if let Some(stream) = accept_within_deadline(&listener) {
                serve_stand_in_connection(StandInLink::Tcp(stream), &library_record, None);
            }
        });
        let mut settings = stand_in_settings(SenderEndpoint::ConnectTcp(address), CROSS_RATE_CONTROL_BLOCKS, 64);
        settings.pace_between_first_transmissions = pace;
        let mut output = Vec::new();
        let report = run_sender(&settings, &mut output);
        library.join().expect("替身库线程没 panic");
        assert_eq!(report.ending, SenderEnding::Complete);
        assert_eq!(report.acknowledged_count, CROSS_RATE_CONTROL_BLOCKS);
        report.acknowledged_count as f64 / (report.first_frame_to_last_acknowledgement_microseconds as f64 / 1_000_000.0)
    }

    #[test]
    fn paced_cross_rate_control_lands_inside_nine_to_ten_point_zero_five_one_blocks_per_second() {
        let paced = paced_run_blocks_per_second(Some(Duration::from_millis(CROSS_RATE_CONTROL_PACE_MILLISECONDS)));
        assert!(
            (CROSS_RATE_CONTROL_LOWEST_BLOCKS_PER_SECOND..=CROSS_RATE_CONTROL_HIGHEST_BLOCKS_PER_SECOND).contains(&paced),
            "压 100 ms 节拍送 200 块，吞吐要落在 [9.0, 10.051] 块/秒，实际 {paced}"
        );
        let unpaced = paced_run_blocks_per_second(None);
        assert!(unpaced > CROSS_RATE_CONTROL_HIGHEST_BLOCKS_PER_SECOND, "不压节奏同样 200 块要快过区间上界，实际 {unpaced}");
    }
}
