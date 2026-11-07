//! 帧格式。整数一律小端。
//!
//! 请求帧：`[u32 帧体长度][u64 请求序号][u8 操作码][操作的内容]`，帧体长度不含它自己那 4 字节。
//! 请求序号是一条连接上的线路序号，只用来对回应；客户端重连之后照旧往上数，不复用。
//! - 报到（操作码 6，序号 0，服务不回应）：客户端连上之后发的第一帧：`u8 长`主机名、`u8 长`任务号、`u8 长`第几次、`u32 进程号`，
//!   任务号与第几次取环境里的 `TASK_SERVER_TASK`、`TASK_SERVER_ATTEMPT`（不在 task-server 里跑的写空）。服务记进这条连接的登记，
//!   列连着的客户端、写故障行都用它。不发报到的客户端照常可用，登记里身份那几段是空的。
//! - 列连着的客户端（操作码 7，只认 unix socket，控制命令用）：没有内容；回这条连接之外的每条连接。
//! - 宣布停服务（操作码 8，只认 unix socket，控制命令用）：没有内容；服务从此不再做新帧，给这条连接之外的每条连接发一帧
//!   「服务停了」（状态 6），回发通知之前的客户端清单。
//!
//! 表（`u8 表`，1 录入、2 事实、3 结论）只是分表码：库里三张表同在默认列族、键原样（`storage.rs` 文件头），用哪张表的名义读写同一把键都是同一行。
//! 第一个字节是 0xF0–0xFF 的键留给服务自己的记账（`is_service_key`）：读、写、认领这样的键，或按以它起头的前缀扫，回状态 1。
//! - 成批点读（操作码 1）：`u32 键数`，每键 `u8 表`、`u32 键长`、键；服务端调一次 `multi_get` 查完。
//! - 成批写（操作码 2）：`u8 客户端身份长`、身份（1 到 255 字节）、`u64 请求号`（同一个身份从 1 起往上数）、`u32 条数`，
//!   每条 `u8 表`、`u32 键长`、键、`u32 值长`、值。服务端按（身份，请求号）去重：早已生效的请求号不再写，照回写确认；
//!   没生效的合进组提交的 WriteBatch，同一批里记下「这个请求号已生效」、释放这些键上的认领，WAL 同步之后才回确认。
//! - 按前缀扫（操作码 3）：`u8 表`、`u32 前缀长`、前缀、`u32 条数上限`、`u32 续扫起点长`、续扫起点。续扫起点空是从前缀起；不空时必须以前缀起头
//!   （不然回状态 1），从排在它之后的第一把键起扫（上一帧回的最后一把键）。一帧被条数上限或帧长上限截断时，客户端拿这一帧的最后一把键接着扫，
//!   不往下拆前缀；发的帧数是条目数除以一帧装下的条数。
//! - 统计（操作码 4）：没有内容；回 RocksDB 自己的计数（同步了几次 WAL、调了几次 multi_get 这类）与服务自己数的
//!   （待处理帧数峰值、回了几次忙、重复的请求号、WAL 同步用时的分布），自证拿它核「一帧只同步一次」「待处理帧数不超上限」。
//! - 认领（操作码 5）：`u8 客户端身份长`、身份、`u32 租期毫秒`、`u32 键数`，每键 `u8 表`、`u32 键长`、键。
//!   逐键判：库里已有这个键（结论已写）回「已有结论」；别的身份持有、租期没到回「别人持有」；否则这个身份认领到
//!   （自己已持有、租期没到的照样续上：到期取已记下的与「现在加这一帧的租期」里晚的那一个，只往后不往前，`committer.rs` 文件头「认领」）。
//!   认领记录与写入走同一个组提交，WAL 同步之后才回。
//!   「别人持有」带持有者身份（至多 255 字节），一帧短键可以换回比帧长上限还长的回应：服务先逐键判、数回应的字节，
//!   超过帧长上限就整帧回状态 1、说明里写回应会有多少字节，一个键都不认领、不续租（不留部分结果）；调用方把键拆成几帧再认领。
//! - 删前缀（操作码 9）：`u32 前缀长`、前缀、`u32 续扫起点长`、续扫起点、`u32 本帧至多几把`、`u8 只数不删`（0 删、1 只数）。
//!   不认表码、不认键的种类：键以前缀起头的行都是目标，三张表同一个键空间。前缀不许空（空前缀等于清空整个库）、不许落在服务记账那一段；
//!   续扫起点空是从前缀起，不空时必须以前缀起头、从排在它之后的第一把键起数（上一帧回的最后一把键，已删的墓碑不用再跳过）；
//!   本帧至多几把在 1 到 `MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME` 之间。提交线程里成批做：同一组里排在前面的写先提交，
//!   删的是已落盘的库；删的键与回应同一组提交、WAL 同步之后才回，删掉的键上的认领同一组释放。回应（带最后一把键）记不进服务的待处理额度时
//!   整帧回忙（状态 5）、一把都不删（`committer.rs` 文件头「删前缀」）。只数不删时不改库、不动认领，数的口径与删一样。
//!   删本来就幂等，所以不按（身份，请求号）去重，也不进客户端的追加日志：重发只会让某一帧报的数少一批，库里的结果不变。
//!
//! 回应帧：`[u32 帧体长度][u64 请求序号][u8 状态][u8 操作码][内容]`。状态 0 是做完，内容按操作码：
//! - 成批点读：`u32 个数`，每个 `u8 在不在`（0 / 1），在的再跟 `u32 值长`、值，次序与请求里的键相同；
//! - 成批写：`u32 写了几条`、`u8 早已生效`（1：这个请求号以前生效过，这一次没再写）；
//! - 按前缀扫：`u8 有没有被条数上限或帧长上限截断`、`u32 条数`，每条 `u32 键长`、键、`u32 值长`、值，按键升序；
//! - 统计：`u32 个数`，每个 `u32 名字长`、名字、`u64 值`；
//! - 认领：`u32 个数`，每个 `u8 结局`：0 认领到；1 别人持有，再跟 `u8 持有者身份长`、身份、`u64 剩余租期毫秒`；2 已有结论。
//! - 删前缀：`u32 这一帧数到（只数不删时）或删掉（删时）几把`、`u8 前缀下是不是已经没有更多的键`（1 没有了；0 还有，用下面的最后一把键接着来）、
//!   `u32 最后一把键长`、最后一把键（这一帧一把都没碰到时是空）。
//! - 列连着的客户端、宣布停服务：`u32 个数`，每个 `u64 连接号`、`u8 走哪条`（1 unix、2 TCP）、`u32 长`对端地址、`u8 报到过没有`，
//!   报到过的再跟报到的四个字段，最后 `u64 连上的时刻`（自 1970 年起的毫秒）。
//!
//! 状态非 0 是拒绝，内容是 `u32 说明长`、说明（UTF-8）：1 内容解不开（键数与内容对不上、帧尾多字节、认不得的表、身份长 0、
//! 键或前缀落在服务记账那一段），
//! 2 认不得的操作码，3 帧长度不对（短于帧头或长于 `MAXIMUM_FRAME_BODY_BYTES`；这一种序号写 0、服务回完就关连接，
//! 因为帧边界已经找不回来），4 存储层出错。1、2、4 之后连接照常可用。
//! 6 服务停了：序号 0、操作码 0，服务在宣布停服务之后主动发给每条连接（之后来的帧也只回这一种）；客户端收到就不再重试，
//! 照 `reliable_client` 文件头的退出码表退出。
//! 状态 5 是忙：这条连接或整个服务的待处理帧数（或待处理字节数）到了上限，这一帧没有排队、没有做；内容是
//! `u32 多少毫秒后再来`、`u32 说明长`、说明。客户端照这个毫秒数加抖动退避之后重发同一帧（写帧带同一个请求号）。
//! 待处理字节数记着收着的帧体与没写出去的回应（`server.rs` 文件头「额度」）：收帧时记不进的，帧体照读完丢掉、回忙；
//! 读、扫、统计的回应编好了记不进的，这一帧做过、回应没留，回忙（不改库，重发即可）；认领的回应记不进的，一个键都没认领、回忙。
//!
//! 流水：客户端不等回应接着发；读、扫、统计由几个工作线程并行处理，写与认领由一个提交线程按到达次序组提交，
//! 回应按完成先后写回，可能乱序，客户端按回应里的序号对回请求。同一条连接上先写后读同一个键要等写的回应到了再发读；
//! 同一个客户端两帧写同一个键，回了忙的那一帧重发之后可能落在后发的那一帧之后，要定次序就等前一帧的确认再发后一帧。

/// 帧体长度上限：满块一帧 65536 态、每态一条约 130 字节，约 9 MB；留出几倍余量，又不让一个坏长度把服务的内存吃满。
pub const MAXIMUM_FRAME_BODY_BYTES: u32 = 64 * 1024 * 1024;
/// 请求帧头：序号 8 字节加操作码 1 字节。
pub const REQUEST_HEADER_BYTES: u32 = 9;
/// 回应帧头：序号 8 字节、状态 1 字节、操作码 1 字节。
pub const RESPONSE_HEADER_BYTES: u32 = 10;
/// 帧体长度字段的字节数。
pub const FRAME_LENGTH_FIELD_BYTES: usize = 4;
/// 服务对整条连接的拒绝（帧长度不对）、服务停了的通知、报到这几种连接层的帧用的序号：客户端的请求序号从 1 起，不会撞上。
pub const CONNECTION_LEVEL_SEQUENCE: u64 = 0;
/// 请求帧里序号字段的位置：长度字段之后的 8 字节。客户端重发同一帧时只换这 8 字节。
const SEQUENCE_FIELD_OFFSET: usize = FRAME_LENGTH_FIELD_BYTES;
/// 报到里各段文字的字节数上限（长度字段是 u8）。
pub const MAXIMUM_ANNOUNCEMENT_TEXT_BYTES: usize = 255;
/// 客户端身份的字节数上限（长度字段是 u8）。
pub const MAXIMUM_CLIENT_IDENTITY_BYTES: usize = 255;
/// 删前缀一帧至多数到或删掉的键数：提交线程一帧里扫这么多把键、把这么多个删除记进同一个写批，
/// 写批与提交线程占着的时间都有界，前缀下键多时客户端分几帧接着来（文件头「删前缀」）。
pub const MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME: u32 = 200_000;

/// 三类表「录入、事实、结论」：协议里的分表码，库里同在默认列族（`storage.rs` 文件头）。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Table {
    Recording,
    Facts,
    Conclusions,
}

impl Table {
    pub const ALL: [Table; 3] = [Table::Recording, Table::Facts, Table::Conclusions];

    pub fn code(self) -> u8 {
        match self {
            Table::Recording => 1,
            Table::Facts => 2,
            Table::Conclusions => 3,
        }
    }

    pub fn from_code(code: u8) -> Option<Table> {
        Table::ALL.into_iter().find(|table| table.code() == code)
    }
}

/// 第一个字节不小于它的键留给服务自己的记账（`storage.rs` 文件头）：崩溃行的表标签（0x00–0x1B）与 Python 三家的 ASCII 族名都小于它。
pub const SERVICE_KEY_LEADING_BYTE_MINIMUM: u8 = 0xF0;

/// 这把键落在服务记账那一段（客户端不许读写）。
pub fn is_service_key(key: &[u8]) -> bool {
    key.first()
        .is_some_and(|leading_byte| *leading_byte >= SERVICE_KEY_LEADING_BYTE_MINIMUM)
}

/// 客户端碰服务记账那一段键：平常拒；只有弄坏开关 `service-keys-open-to-clients` 放过。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServiceKeys {
    Refused,
    Accepted,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperationCode {
    BatchedPointRead,
    BatchedWrite,
    PrefixScan,
    Statistics,
    Claim,
    ConnectionAnnouncement,
    ConnectedClients,
    AnnounceStop,
    DeletePrefix,
}

impl OperationCode {
    pub const ALL: [OperationCode; 9] = [
        OperationCode::BatchedPointRead,
        OperationCode::BatchedWrite,
        OperationCode::PrefixScan,
        OperationCode::Statistics,
        OperationCode::Claim,
        OperationCode::ConnectionAnnouncement,
        OperationCode::ConnectedClients,
        OperationCode::AnnounceStop,
        OperationCode::DeletePrefix,
    ];

    pub fn code(self) -> u8 {
        match self {
            OperationCode::BatchedPointRead => 1,
            OperationCode::BatchedWrite => 2,
            OperationCode::PrefixScan => 3,
            OperationCode::Statistics => 4,
            OperationCode::Claim => 5,
            OperationCode::ConnectionAnnouncement => 6,
            OperationCode::ConnectedClients => 7,
            OperationCode::AnnounceStop => 8,
            OperationCode::DeletePrefix => 9,
        }
    }

    /// 写、认领与删前缀要改库（或要读已落盘的库），交提交线程按到达次序组提交；别的交工作线程并行做，或由读线程当场做（报到、列客户端、宣布停服务）。
    pub fn changes_library(self) -> bool {
        match self {
            OperationCode::BatchedWrite | OperationCode::Claim | OperationCode::DeletePrefix => {
                true
            }
            OperationCode::BatchedPointRead
            | OperationCode::PrefixScan
            | OperationCode::Statistics
            | OperationCode::ConnectionAnnouncement
            | OperationCode::ConnectedClients
            | OperationCode::AnnounceStop => false,
        }
    }

    /// 要这条连接的登记、由读线程当场做的几种。
    pub fn is_handled_by_connection_reader(self) -> bool {
        match self {
            OperationCode::ConnectionAnnouncement
            | OperationCode::ConnectedClients
            | OperationCode::AnnounceStop => true,
            OperationCode::BatchedPointRead
            | OperationCode::BatchedWrite
            | OperationCode::PrefixScan
            | OperationCode::Statistics
            | OperationCode::Claim
            | OperationCode::DeletePrefix => false,
        }
    }

    pub fn from_code(code: u8) -> Option<OperationCode> {
        OperationCode::ALL
            .into_iter()
            .find(|operation| operation.code() == code)
    }
}

/// 回应状态。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResponseStatus {
    Completed,
    MalformedPayload,
    UnknownOperation,
    FrameLengthRejected,
    StorageFailed,
    Busy,
    ServerStopping,
}

impl ResponseStatus {
    pub const ALL: [ResponseStatus; 7] = [
        ResponseStatus::Completed,
        ResponseStatus::MalformedPayload,
        ResponseStatus::UnknownOperation,
        ResponseStatus::FrameLengthRejected,
        ResponseStatus::StorageFailed,
        ResponseStatus::Busy,
        ResponseStatus::ServerStopping,
    ];

    pub fn code(self) -> u8 {
        match self {
            ResponseStatus::Completed => 0,
            ResponseStatus::MalformedPayload => 1,
            ResponseStatus::UnknownOperation => 2,
            ResponseStatus::FrameLengthRejected => 3,
            ResponseStatus::StorageFailed => 4,
            ResponseStatus::Busy => 5,
            ResponseStatus::ServerStopping => 6,
        }
    }

    pub fn from_code(code: u8) -> Option<ResponseStatus> {
        ResponseStatus::ALL
            .into_iter()
            .find(|status| status.code() == code)
    }
}

/// 帧体长度在不在 `[帧头, 上限]` 之内。
pub fn is_frame_length_acceptable(declared_body_bytes: u32, header_bytes: u32) -> bool {
    declared_body_bytes >= header_bytes && declared_body_bytes <= MAXIMUM_FRAME_BODY_BYTES
}

/// 成批点读回应帧的内容有几个字节（不含回应帧头）：`u32 个数`，在的键 `u8 在不在`、`u32 值长`、值，不在的键只有 `u8 在不在`（文件头「回应帧」）。
/// `value_lengths_in_bytes` 与请求里的键同序，不在的键给 `None`。
pub fn point_read_response_payload_bytes(
    value_lengths_in_bytes: impl IntoIterator<Item = Option<usize>>,
) -> usize {
    4 + value_lengths_in_bytes
        .into_iter()
        .map(|value_length_in_bytes| value_length_in_bytes.map_or(1, |length| 5 + length))
        .sum::<usize>()
}

/// 成批点读的回应装不进一帧（内容加回应帧头超过帧长上限）：服务照它整帧拒，
/// `prune-server drop-oversized-values` 照它认一把键单独一帧也读不回的整值（`oversized_values.rs` 文件头）。
pub fn point_read_response_exceeds_frame_limit(payload_bytes: usize) -> bool {
    payload_bytes + usize_from_u32(RESPONSE_HEADER_BYTES) > usize_from_u32(MAXIMUM_FRAME_BODY_BYTES)
}

pub(crate) fn usize_from_u32(value: u32) -> usize {
    usize::try_from(value).expect("这个工程只编 64 位目标，u32 放得进 usize")
}

fn u32_from_usize(value: usize, what: &str) -> u32 {
    u32::try_from(value).unwrap_or_else(|_| {
        panic!(
            "{what} 超出 u32：{value}；帧长上限是 {MAXIMUM_FRAME_BODY_BYTES}，调用方先按上限切帧"
        )
    })
}

/// 往一帧里追加字段，`finish` 时回填帧体长度。
pub struct FrameEncoder {
    bytes: Vec<u8>,
}

impl FrameEncoder {
    pub fn with_capacity(capacity_in_bytes: usize) -> FrameEncoder {
        let mut bytes = Vec::with_capacity(capacity_in_bytes + FRAME_LENGTH_FIELD_BYTES);
        bytes.extend_from_slice(&[0_u8; FRAME_LENGTH_FIELD_BYTES]);
        FrameEncoder { bytes }
    }

    pub fn put_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn put_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn put_u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn put_length_prefixed(&mut self, field: &[u8]) {
        self.put_u32(u32_from_usize(field.len(), "字段长度"));
        self.bytes.extend_from_slice(field);
    }

    /// 一段短文字：`u8 长`、字节。超过 255 字节是调用方的 bug（报到的各段在 `ClientAnnouncement::new` 里截过）。
    pub fn put_short_text(&mut self, text: &str) {
        let length = u8::try_from(text.len()).unwrap_or_else(|_| {
            panic!(
                "短文字要至多 {MAXIMUM_ANNOUNCEMENT_TEXT_BYTES} 字节，收到 {} 字节",
                text.len()
            )
        });
        self.bytes.push(length);
        self.bytes.extend_from_slice(text.as_bytes());
    }

    /// 按前缀扫的回应：帧头之后先占位的截断标记与条数，扫完再填（`server.rs` 边扫边编，文件头「额度」）。
    ///
    /// # Panics
    /// 帧头之后还没放进截断标记与条数那 5 字节（调用方先 `put_u8`、`put_u32` 占位）。
    pub fn set_prefix_scan_header(&mut self, is_truncated: bool, entry_count: u32) {
        let start = FRAME_LENGTH_FIELD_BYTES + usize_from_u32(RESPONSE_HEADER_BYTES);
        self.bytes[start] = u8::from(is_truncated);
        self.bytes[start + 1..start + 5].copy_from_slice(&entry_count.to_le_bytes());
    }

    /// 再放 `additional` 字节要多分配的容量（`server.rs` 边编边记额度：先按它记，再 `grow_capacity`）：装得下回 0，装不下回把容量翻倍
    /// （至少到装得下）多出的字节。
    #[must_use]
    pub fn capacity_growth_for(&self, additional: usize) -> usize {
        let needed = self.bytes.len() + additional;
        let capacity = self.bytes.capacity();
        if needed <= capacity {
            0
        } else {
            needed.max(capacity.saturating_mul(2)) - capacity
        }
    }

    /// 容量加 `growth` 字节（`capacity_growth_for` 算的、已经记进额度的那么多）；0 不动。
    pub fn grow_capacity(&mut self, growth: usize) {
        if growth > 0 {
            let additional = self.bytes.capacity() + growth - self.bytes.len();
            self.bytes.reserve_exact(additional);
        }
    }

    /// 帧体此刻的字节数（不含长度字段）。
    pub fn body_bytes(&self) -> usize {
        self.bytes.len() - FRAME_LENGTH_FIELD_BYTES
    }

    pub fn finish(mut self) -> Vec<u8> {
        let body_bytes = u32_from_usize(self.body_bytes(), "帧体长度");
        self.bytes[..FRAME_LENGTH_FIELD_BYTES].copy_from_slice(&body_bytes.to_le_bytes());
        self.bytes
    }
}

fn begin_request(sequence: u64, operation_code: u8, capacity_in_bytes: usize) -> FrameEncoder {
    let mut encoder =
        FrameEncoder::with_capacity(capacity_in_bytes + usize_from_u32(REQUEST_HEADER_BYTES));
    encoder.put_u64(sequence);
    encoder.put_u8(operation_code);
    encoder
}

/// 成批点读帧：先声明键数，再逐个加键；加的个数与声明的不同是调用方的 bug。
pub struct PointReadFrameBuilder {
    encoder: FrameEncoder,
    declared_keys: u32,
    added_keys: u32,
}

pub fn begin_point_read_frame(
    sequence: u64,
    key_count: u32,
    capacity_in_bytes: usize,
) -> PointReadFrameBuilder {
    let mut encoder = begin_request(
        sequence,
        OperationCode::BatchedPointRead.code(),
        capacity_in_bytes + 4,
    );
    encoder.put_u32(key_count);
    PointReadFrameBuilder {
        encoder,
        declared_keys: key_count,
        added_keys: 0,
    }
}

impl PointReadFrameBuilder {
    pub fn key(&mut self, table: Table, key: &[u8]) {
        self.encoder.put_u8(table.code());
        self.encoder.put_length_prefixed(key);
        self.added_keys += 1;
    }

    pub fn finish(self) -> Vec<u8> {
        assert_eq!(
            self.added_keys, self.declared_keys,
            "成批点读帧声明的键数与加进去的键数不同"
        );
        self.encoder.finish()
    }
}

/// 成批写帧：先声明条数，再逐条加。
pub struct WriteFrameBuilder {
    encoder: FrameEncoder,
    declared_entries: u32,
    added_entries: u32,
}

/// 身份长度不在 1 到 255 字节之内是调用方的 bug（身份由客户端自己生成）。
fn put_client_identity(encoder: &mut FrameEncoder, client_identity: &[u8]) {
    assert!(
        !client_identity.is_empty() && client_identity.len() <= MAXIMUM_CLIENT_IDENTITY_BYTES,
        "客户端身份要 1 到 {MAXIMUM_CLIENT_IDENTITY_BYTES} 字节，收到 {} 字节",
        client_identity.len()
    );
    encoder.put_u8(u8::try_from(client_identity.len()).expect("上面核过不超 255"));
    encoder.bytes.extend_from_slice(client_identity);
}

pub fn begin_write_frame(
    sequence: u64,
    client_identity: &[u8],
    request_number: u64,
    entry_count: u32,
    capacity_in_bytes: usize,
) -> WriteFrameBuilder {
    let mut encoder = begin_request(
        sequence,
        OperationCode::BatchedWrite.code(),
        capacity_in_bytes + client_identity.len() + 13,
    );
    put_client_identity(&mut encoder, client_identity);
    encoder.put_u64(request_number);
    encoder.put_u32(entry_count);
    WriteFrameBuilder {
        encoder,
        declared_entries: entry_count,
        added_entries: 0,
    }
}

impl WriteFrameBuilder {
    pub fn entry(&mut self, table: Table, key: &[u8], value: &[u8]) {
        self.encoder.put_u8(table.code());
        self.encoder.put_length_prefixed(key);
        self.encoder.put_length_prefixed(value);
        self.added_entries += 1;
    }

    pub fn finish(self) -> Vec<u8> {
        assert_eq!(
            self.added_entries, self.declared_entries,
            "成批写帧声明的条数与加进去的条数不同"
        );
        self.encoder.finish()
    }
}

/// 按前缀扫帧、从前缀起（续扫起点空）。
pub fn encode_prefix_scan_frame(
    sequence: u64,
    table: Table,
    prefix: &[u8],
    maximum_entries: u32,
) -> Vec<u8> {
    encode_prefix_scan_frame_after(sequence, table, prefix, maximum_entries, &[])
}

/// 按前缀扫帧（文件头「按前缀扫」）：`after_key` 空是从前缀起，不空是从排在它之后的第一把键起。
pub fn encode_prefix_scan_frame_after(
    sequence: u64,
    table: Table,
    prefix: &[u8],
    maximum_entries: u32,
    after_key: &[u8],
) -> Vec<u8> {
    let mut encoder = begin_request(
        sequence,
        OperationCode::PrefixScan.code(),
        prefix.len() + after_key.len() + 13,
    );
    encoder.put_u8(table.code());
    encoder.put_length_prefixed(prefix);
    encoder.put_u32(maximum_entries);
    encoder.put_length_prefixed(after_key);
    encoder.finish()
}

pub fn encode_statistics_frame(sequence: u64) -> Vec<u8> {
    begin_request(sequence, OperationCode::Statistics.code(), 0).finish()
}

/// 删前缀帧（文件头「删前缀」）：`after_key` 空是从前缀起，不空是从排在它之后的第一把键起；`count_only` 为真只数不删。
pub fn encode_delete_prefix_frame(
    sequence: u64,
    prefix: &[u8],
    after_key: &[u8],
    maximum_keys: u32,
    count_only: bool,
) -> Vec<u8> {
    let mut encoder = begin_request(
        sequence,
        OperationCode::DeletePrefix.code(),
        prefix.len() + after_key.len() + 13,
    );
    encoder.put_length_prefixed(prefix);
    encoder.put_length_prefixed(after_key);
    encoder.put_u32(maximum_keys);
    encoder.put_u8(u8::from(count_only));
    encoder.finish()
}

/// 认领帧：一批键各自向服务认领，租期以毫秒计。
pub fn encode_claim_frame(
    sequence: u64,
    client_identity: &[u8],
    lease_milliseconds: u32,
    keys: &[(Table, &[u8])],
) -> Vec<u8> {
    let capacity =
        client_identity.len() + 9 + keys.iter().map(|(_, key)| 5 + key.len()).sum::<usize>();
    let mut encoder = begin_request(sequence, OperationCode::Claim.code(), capacity);
    put_client_identity(&mut encoder, client_identity);
    encoder.put_u32(lease_milliseconds);
    encoder.put_u32(u32_from_usize(keys.len(), "认领的键数"));
    for (table, key) in keys {
        encoder.put_u8(table.code());
        encoder.put_length_prefixed(key);
    }
    encoder.finish()
}

/// 换掉一帧编好的认领请求（[`encode_claim_frame`]，带长度字段）里的租约毫秒数：帧头之后是客户端身份（长度 1 字节加身份），再后面 4 字节是租约。
/// 弄坏开关 claim-lease-shrunk-on-resend 下客户端每次发认领时照剩下的租约换（`reliable_client` 的 `claim_lease_left`，改前的写法），帧的其余字节不动。
pub fn replace_claim_lease(frame: &mut [u8], lease_milliseconds: u32) {
    let identity_length_offset = FRAME_LENGTH_FIELD_BYTES + usize_from_u32(REQUEST_HEADER_BYTES);
    let lease_offset = identity_length_offset + 1 + usize::from(frame[identity_length_offset]);
    frame[lease_offset..lease_offset + 4].copy_from_slice(&lease_milliseconds.to_le_bytes());
}

/// 换掉一帧请求的序号（重连之后重发、回了忙之后重发都用新序号，帧的其余字节不动）。
pub fn replace_request_sequence(frame: &mut [u8], sequence: u64) {
    frame[SEQUENCE_FIELD_OFFSET..SEQUENCE_FIELD_OFFSET + 8]
        .copy_from_slice(&sequence.to_le_bytes());
}

/// 一帧编好的请求是不是序号 0 的连接层帧（报到）：故障注入代理不拿它们数帧、不往它们身上注入。
pub fn is_connection_level_request(frame: &[u8]) -> bool {
    frame
        .get(FRAME_LENGTH_FIELD_BYTES..)
        .and_then(peek_request_header)
        .is_some_and(|(sequence, _)| sequence == CONNECTION_LEVEL_SEQUENCE)
}

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

/// 帧体（不含长度字段）开头的序号与操作码；帧体短于请求帧头回 `None`（长度闸已经挡在前面，这里只防御）。
pub fn peek_request_header(body: &[u8]) -> Option<(u64, u8)> {
    let header = body.get(..usize_from_u32(REQUEST_HEADER_BYTES))?;
    let sequence = u64::from_le_bytes(header[..8].try_into().expect("切的正好是 8 字节"));
    Some((sequence, header[8]))
}

/// 照原样拼一帧：自证造认不得的操作码与解不开的内容用。
pub fn encode_raw_request_frame(sequence: u64, operation_code: u8, payload: &[u8]) -> Vec<u8> {
    let mut encoder = begin_request(sequence, operation_code, payload.len());
    encoder.bytes.extend_from_slice(payload);
    encoder.finish()
}

/// 按位置读帧体的游标；越界是「内容解不开」，不是 panic。
struct ByteCursor<'frame> {
    bytes: &'frame [u8],
    position: usize,
}

impl<'frame> ByteCursor<'frame> {
    fn new(bytes: &'frame [u8]) -> ByteCursor<'frame> {
        ByteCursor { bytes, position: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    fn take(&mut self, byte_count: usize, what: &str) -> Result<&'frame [u8], String> {
        if byte_count > self.remaining() {
            return Err(format!(
                "读{what}要 {byte_count} 字节，帧里只剩 {} 字节",
                self.remaining()
            ));
        }
        let field = &self.bytes[self.position..self.position + byte_count];
        self.position += byte_count;
        Ok(field)
    }

    fn u8(&mut self, what: &str) -> Result<u8, String> {
        Ok(self.take(1, what)?[0])
    }

    fn u32(&mut self, what: &str) -> Result<u32, String> {
        let field = self.take(4, what)?;
        Ok(u32::from_le_bytes(
            field.try_into().expect("take 回的正好是 4 字节"),
        ))
    }

    fn u64(&mut self, what: &str) -> Result<u64, String> {
        let field = self.take(8, what)?;
        Ok(u64::from_le_bytes(
            field.try_into().expect("take 回的正好是 8 字节"),
        ))
    }

    fn length_prefixed(&mut self, what: &str) -> Result<&'frame [u8], String> {
        let length = self.u32(what)?;
        self.take(usize_from_u32(length), what)
    }

    fn table(&mut self) -> Result<Table, String> {
        let code = self.u8("表号")?;
        Table::from_code(code).ok_or_else(|| format!("认不得的表号 {code}"))
    }

    fn client_identity(&mut self) -> Result<&'frame [u8], String> {
        let length = self.u8("客户端身份长")?;
        if length == 0 {
            return Err("客户端身份长是 0".to_string());
        }
        self.take(usize::from(length), "客户端身份")
    }

    fn short_text(&mut self, what: &str) -> Result<String, String> {
        let length = self.u8(what)?;
        let field = self.take(usize::from(length), what)?;
        std::str::from_utf8(field)
            .map(str::to_string)
            .map_err(|_| format!("{what}不是 UTF-8"))
    }

    fn announcement(&mut self) -> Result<ClientAnnouncement, String> {
        let host = self.short_text("主机名")?;
        let task = self.short_text("任务号")?;
        let attempt = self.short_text("第几次")?;
        let process_identifier = self.u32("进程号")?;
        Ok(ClientAnnouncement {
            host,
            task,
            attempt,
            process_identifier,
        })
    }

    /// 按声称的个数预留容量之前先按剩下的字节数封顶，坏的个数字段不会让服务一次要几十 GB。
    fn capacity_for(&self, declared_count: u32, minimum_bytes_per_item: usize) -> usize {
        usize_from_u32(declared_count).min(self.remaining() / minimum_bytes_per_item)
    }
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeyInTable<'frame> {
    pub table: Table,
    pub key: &'frame [u8],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntryInTable<'frame> {
    pub table: Table,
    pub key: &'frame [u8],
    pub value: &'frame [u8],
}

/// 解开的请求，键与值借用帧体，不另拷。
#[derive(Debug, PartialEq, Eq)]
pub enum RequestView<'frame> {
    BatchedPointRead(Vec<KeyInTable<'frame>>),
    BatchedWrite {
        client_identity: &'frame [u8],
        request_number: u64,
        entries: Vec<EntryInTable<'frame>>,
    },
    PrefixScan {
        table: Table,
        prefix: &'frame [u8],
        maximum_entries: u32,
        after_key: &'frame [u8],
    },
    Statistics,
    Claim {
        client_identity: &'frame [u8],
        lease_milliseconds: u32,
        keys: Vec<KeyInTable<'frame>>,
    },
    ConnectionAnnouncement(ClientAnnouncement),
    ConnectedClients,
    AnnounceStop,
    DeletePrefix {
        prefix: &'frame [u8],
        after_key: &'frame [u8],
        maximum_keys: u32,
        count_only: bool,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct DecodedRequest<'frame> {
    pub sequence: u64,
    pub view: RequestView<'frame>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestRejection {
    UnknownOperation { operation_code: u8 },
    MalformedPayload { operation_code: u8, reason: String },
}

/// 帧尾多出来的字节：平常拒；只有弄坏开关 `trailing-bytes-accepted` 放过。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrailingBytes {
    Rejected,
    Accepted,
}

/// 解请求帧体（不含长度字段）。拒绝时带上读得到的序号（帧体短于 8 字节时是 0）。
pub fn decode_request_body(
    body: &[u8],
    trailing_bytes: TrailingBytes,
    service_keys: ServiceKeys,
) -> Result<DecodedRequest<'_>, (u64, RequestRejection)> {
    let mut cursor = ByteCursor::new(body);
    let sequence = cursor.u64("请求序号").map_err(|reason| {
        (
            CONNECTION_LEVEL_SEQUENCE,
            RequestRejection::MalformedPayload {
                operation_code: 0,
                reason,
            },
        )
    })?;
    let operation_code = cursor.u8("操作码").map_err(|reason| {
        (
            sequence,
            RequestRejection::MalformedPayload {
                operation_code: 0,
                reason,
            },
        )
    })?;
    let Some(operation) = OperationCode::from_code(operation_code) else {
        return Err((
            sequence,
            RequestRejection::UnknownOperation { operation_code },
        ));
    };
    let malformed = |reason: String| {
        (
            sequence,
            RequestRejection::MalformedPayload {
                operation_code,
                reason,
            },
        )
    };
    let view = decode_request_payload(operation, &mut cursor).map_err(malformed)?;
    if cursor.remaining() != 0 && trailing_bytes == TrailingBytes::Rejected {
        return Err(malformed(format!(
            "内容解完之后帧尾还多 {} 字节",
            cursor.remaining()
        )));
    }
    if service_keys == ServiceKeys::Refused {
        if let Some(reason) = service_key_refusal(&view) {
            return Err(malformed(reason));
        }
    }
    Ok(DecodedRequest { sequence, view })
}

/// 请求里有落在服务记账那一段的键（或以它起头的前缀）时回说明。
fn service_key_refusal(view: &RequestView<'_>) -> Option<String> {
    let refused = |what: &str, key: &[u8]| {
        Some(format!(
            "{what} {:02x?} 的第一个字节不小于 0x{SERVICE_KEY_LEADING_BYTE_MINIMUM:02X}：这一段键留给服务自己的记账（去重与认领），客户端不许读写",
            &key[..key.len().min(16)]
        ))
    };
    match view {
        RequestView::BatchedPointRead(keys) | RequestView::Claim { keys, .. } => keys
            .iter()
            .find(|key| is_service_key(key.key))
            .and_then(|key| refused("键", key.key)),
        RequestView::BatchedWrite { entries, .. } => entries
            .iter()
            .find(|entry| is_service_key(entry.key))
            .and_then(|entry| refused("键", entry.key)),
        RequestView::PrefixScan { prefix, .. } | RequestView::DeletePrefix { prefix, .. }
            if is_service_key(prefix) =>
        {
            refused("前缀", prefix)
        }
        RequestView::PrefixScan { .. }
        | RequestView::DeletePrefix { .. }
        | RequestView::Statistics
        | RequestView::ConnectionAnnouncement(_)
        | RequestView::ConnectedClients
        | RequestView::AnnounceStop => None,
    }
}

fn decode_request_payload<'frame>(
    operation: OperationCode,
    cursor: &mut ByteCursor<'frame>,
) -> Result<RequestView<'frame>, String> {
    match operation {
        OperationCode::BatchedPointRead => {
            let key_count = cursor.u32("键数")?;
            let mut keys = Vec::with_capacity(cursor.capacity_for(key_count, 5));
            for _ in 0..key_count {
                let table = cursor.table()?;
                let key = cursor.length_prefixed("键")?;
                keys.push(KeyInTable { table, key });
            }
            Ok(RequestView::BatchedPointRead(keys))
        }
        OperationCode::BatchedWrite => {
            let client_identity = cursor.client_identity()?;
            let request_number = cursor.u64("请求号")?;
            let entry_count = cursor.u32("条数")?;
            let mut entries = Vec::with_capacity(cursor.capacity_for(entry_count, 9));
            for _ in 0..entry_count {
                let table = cursor.table()?;
                let key = cursor.length_prefixed("键")?;
                let value = cursor.length_prefixed("值")?;
                entries.push(EntryInTable { table, key, value });
            }
            Ok(RequestView::BatchedWrite {
                client_identity,
                request_number,
                entries,
            })
        }
        OperationCode::Claim => {
            let client_identity = cursor.client_identity()?;
            let lease_milliseconds = cursor.u32("租期毫秒")?;
            let key_count = cursor.u32("键数")?;
            let mut keys = Vec::with_capacity(cursor.capacity_for(key_count, 5));
            for _ in 0..key_count {
                let table = cursor.table()?;
                let key = cursor.length_prefixed("键")?;
                keys.push(KeyInTable { table, key });
            }
            Ok(RequestView::Claim {
                client_identity,
                lease_milliseconds,
                keys,
            })
        }
        OperationCode::PrefixScan => {
            let table = cursor.table()?;
            let prefix = cursor.length_prefixed("前缀")?;
            let maximum_entries = cursor.u32("条数上限")?;
            let after_key = cursor.length_prefixed("续扫起点")?;
            if !after_key.is_empty() && !after_key.starts_with(prefix) {
                return Err("续扫起点不以前缀起头：续扫起点是上一帧回的最后一把键".to_string());
            }
            Ok(RequestView::PrefixScan {
                table,
                prefix,
                maximum_entries,
                after_key,
            })
        }
        OperationCode::DeletePrefix => {
            let prefix = cursor.length_prefixed("前缀")?;
            let after_key = cursor.length_prefixed("续扫起点")?;
            let maximum_keys = cursor.u32("本帧至多几把")?;
            let count_only = match cursor.u8("只数不删")? {
                0 => false,
                1 => true,
                other => return Err(format!("「只数不删」只能是 0 或 1，收到 {other}")),
            };
            if prefix.is_empty() {
                return Err("前缀是空的：删空前缀等于清空整个库，不收".to_string());
            }
            if !after_key.is_empty() && !after_key.starts_with(prefix) {
                return Err("续扫起点不以前缀起头：续扫起点是上一帧回的最后一把键".to_string());
            }
            if maximum_keys == 0 || maximum_keys > MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME {
                return Err(format!(
                    "本帧至多几把要在 1 到 {MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME} 之间，收到 {maximum_keys}"
                ));
            }
            Ok(RequestView::DeletePrefix {
                prefix,
                after_key,
                maximum_keys,
                count_only,
            })
        }
        OperationCode::Statistics => Ok(RequestView::Statistics),
        OperationCode::ConnectionAnnouncement => {
            Ok(RequestView::ConnectionAnnouncement(cursor.announcement()?))
        }
        OperationCode::ConnectedClients => Ok(RequestView::ConnectedClients),
        OperationCode::AnnounceStop => Ok(RequestView::AnnounceStop),
    }
}

/// 回应帧的开头：序号、状态、操作码；内容由调用方接着写。
pub fn begin_response(
    sequence: u64,
    status: ResponseStatus,
    operation_code: u8,
    capacity_in_bytes: usize,
) -> FrameEncoder {
    let mut encoder =
        FrameEncoder::with_capacity(capacity_in_bytes + usize_from_u32(RESPONSE_HEADER_BYTES));
    encoder.put_u64(sequence);
    encoder.put_u8(status.code());
    encoder.put_u8(operation_code);
    encoder
}

pub fn encode_rejection(
    sequence: u64,
    status: ResponseStatus,
    operation_code: u8,
    message: &str,
) -> Vec<u8> {
    assert!(
        status != ResponseStatus::Completed,
        "拒绝回应的状态不能是做完"
    );
    assert!(
        status != ResponseStatus::Busy,
        "忙的回应带「多少毫秒后再来」，走 encode_busy"
    );
    let mut encoder = begin_response(sequence, status, operation_code, message.len() + 4);
    encoder.put_length_prefixed(message.as_bytes());
    encoder.finish()
}

/// 忙：这一帧没有排队、没有做，`retry_after_milliseconds` 之后再来。
pub fn encode_busy(
    sequence: u64,
    operation_code: u8,
    retry_after_milliseconds: u32,
    message: &str,
) -> Vec<u8> {
    let mut encoder = begin_response(
        sequence,
        ResponseStatus::Busy,
        operation_code,
        message.len() + 8,
    );
    encoder.put_u32(retry_after_milliseconds);
    encoder.put_length_prefixed(message.as_bytes());
    encoder.finish()
}

/// 写确认：写了几条、这个请求号是不是早已生效（这一次没再写）。
pub fn encode_write_acknowledgement(
    sequence: u64,
    entries_written: u32,
    is_already_applied: bool,
) -> Vec<u8> {
    let mut encoder = begin_response(
        sequence,
        ResponseStatus::Completed,
        OperationCode::BatchedWrite.code(),
        5,
    );
    encoder.put_u32(entries_written);
    encoder.put_u8(u8::from(is_already_applied));
    encoder.finish()
}

/// 认领的结局，逐键一个。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ClaimOutcome {
    Claimed,
    HeldByOther {
        holder_identity: Vec<u8>,
        remaining_lease_milliseconds: u64,
    },
    AlreadyConcluded,
}

/// 回应内容（回应帧头之后）至多这么多字节，再多整帧就越过帧长上限 `MAXIMUM_FRAME_BODY_BYTES`。
pub fn maximum_response_payload_bytes() -> usize {
    usize_from_u32(MAXIMUM_FRAME_BODY_BYTES - RESPONSE_HEADER_BYTES)
}

/// 认领回应内容开头的个数字段。
pub const CLAIM_OUTCOME_COUNT_BYTES: usize = 4;
/// 「认领到」「已有结论」编进回应各占的字节（只有结局那 1 字节）。
pub const SHORT_CLAIM_OUTCOME_BYTES: usize = 1;

/// 「别人持有」编进回应占的字节：结局 1、身份长 1、持有者身份、剩余租期 8。
pub fn held_by_other_outcome_bytes(holder_identity_bytes: usize) -> usize {
    1 + 1 + holder_identity_bytes + 8
}

/// 一组认领结局编成回应之后内容的字节数（不含回应帧头），与 `encode_claim_outcomes` 写出的逐字节相同。
pub fn claim_outcomes_payload_bytes(outcomes: &[ClaimOutcome]) -> usize {
    CLAIM_OUTCOME_COUNT_BYTES
        + outcomes
            .iter()
            .map(|outcome| match outcome {
                ClaimOutcome::HeldByOther {
                    holder_identity, ..
                } => held_by_other_outcome_bytes(holder_identity.len()),
                ClaimOutcome::Claimed | ClaimOutcome::AlreadyConcluded => SHORT_CLAIM_OUTCOME_BYTES,
            })
            .sum::<usize>()
}

/// 删前缀回应（文件头「删前缀」）：这一帧数到或删掉几把、前缀下是不是已经没有更多的键、最后一把键（一把都没碰到时空）。
pub fn encode_prefix_deleted_response(
    sequence: u64,
    keys_in_frame: u32,
    is_exhausted: bool,
    last_key: &[u8],
) -> Vec<u8> {
    let mut encoder = begin_response(
        sequence,
        ResponseStatus::Completed,
        OperationCode::DeletePrefix.code(),
        last_key.len() + 9,
    );
    encoder.put_u32(keys_in_frame);
    encoder.put_u8(u8::from(is_exhausted));
    encoder.put_length_prefixed(last_key);
    encoder.finish()
}

/// 认领回应：调用方先按 `claim_outcomes_payload_bytes` 核过不超 `maximum_response_payload_bytes`（服务在认领之前核，
/// 超了整帧拒、一个键都不认领，`committer.rs` 的 `add_claim`）。
pub fn encode_claim_outcomes(sequence: u64, outcomes: &[ClaimOutcome]) -> Vec<u8> {
    let capacity = claim_outcomes_payload_bytes(outcomes);
    let mut encoder = begin_response(
        sequence,
        ResponseStatus::Completed,
        OperationCode::Claim.code(),
        capacity,
    );
    encoder.put_u32(u32_from_usize(outcomes.len(), "认领结局个数"));
    for outcome in outcomes {
        match outcome {
            ClaimOutcome::Claimed => encoder.put_u8(0),
            ClaimOutcome::HeldByOther {
                holder_identity,
                remaining_lease_milliseconds,
            } => {
                encoder.put_u8(1);
                put_client_identity(&mut encoder, holder_identity);
                encoder.put_u64(*remaining_lease_milliseconds);
            }
            ClaimOutcome::AlreadyConcluded => encoder.put_u8(2),
        }
    }
    encoder.finish()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RejectionStatus {
    MalformedPayload,
    UnknownOperation,
    FrameLengthRejected,
    StorageFailed,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ScanEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StatisticsCounter {
    pub name: String,
    pub value: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ResponseOutcome {
    PointValues(Vec<Option<Vec<u8>>>),
    WriteAcknowledged {
        entries_written: u32,
        is_already_applied: bool,
    },
    ScanEntries {
        is_truncated: bool,
        entries: Vec<ScanEntry>,
    },
    Statistics(Vec<StatisticsCounter>),
    Claims(Vec<ClaimOutcome>),
    /// 删前缀的回应：`keys_in_frame` 是这一帧数到（只数不删时）或删掉的键数，`is_exhausted` 为真是前缀下已经没有更多的键，
    /// `last_key` 是这一帧碰到的最后一把键（下一帧的续扫起点；一把都没碰到时空）。
    PrefixDeleted {
        keys_in_frame: u32,
        is_exhausted: bool,
        last_key: Vec<u8>,
    },
    ConnectedClients(Vec<ConnectedClient>),
    ServerStopping {
        message: String,
    },
    Busy {
        retry_after_milliseconds: u32,
        message: String,
    },
    Rejected {
        status: RejectionStatus,
        message: String,
    },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Response {
    pub sequence: u64,
    pub operation_code: u8,
    pub outcome: ResponseOutcome,
}

/// 统计回应里按名字取一个计数。
pub fn statistics_value(counters: &[StatisticsCounter], name: &str) -> Option<u64> {
    counters
        .iter()
        .find(|counter| counter.name == name)
        .map(|counter| counter.value)
}

/// 解回应帧体（不含长度字段）。认不得的状态、做完却带认不得的操作码、内容与长度对不上，都是错。
pub fn decode_response_body(body: &[u8]) -> Result<Response, String> {
    let mut cursor = ByteCursor::new(body);
    let sequence = cursor.u64("回应序号")?;
    let status_code = cursor.u8("状态")?;
    let operation_code = cursor.u8("操作码")?;
    let status = ResponseStatus::from_code(status_code)
        .ok_or_else(|| format!("认不得的回应状态 {status_code}"))?;
    let outcome = match status {
        ResponseStatus::Completed => {
            let operation = OperationCode::from_code(operation_code)
                .ok_or_else(|| format!("回应说做完了，操作码却是认不得的 {operation_code}"))?;
            decode_completed_payload(operation, &mut cursor)?
        }
        ResponseStatus::MalformedPayload => {
            rejected(RejectionStatus::MalformedPayload, &mut cursor)?
        }
        ResponseStatus::UnknownOperation => {
            rejected(RejectionStatus::UnknownOperation, &mut cursor)?
        }
        ResponseStatus::FrameLengthRejected => {
            rejected(RejectionStatus::FrameLengthRejected, &mut cursor)?
        }
        ResponseStatus::StorageFailed => rejected(RejectionStatus::StorageFailed, &mut cursor)?,
        ResponseStatus::ServerStopping => {
            let message =
                String::from_utf8_lossy(cursor.length_prefixed("服务停了的说明")?).into_owned();
            ResponseOutcome::ServerStopping { message }
        }
        ResponseStatus::Busy => {
            let retry_after_milliseconds = cursor.u32("多少毫秒后再来")?;
            let message = String::from_utf8_lossy(cursor.length_prefixed("忙的说明")?).into_owned();
            ResponseOutcome::Busy {
                retry_after_milliseconds,
                message,
            }
        }
    };
    if cursor.remaining() != 0 {
        return Err(format!(
            "回应内容解完之后帧尾还多 {} 字节",
            cursor.remaining()
        ));
    }
    Ok(Response {
        sequence,
        operation_code,
        outcome,
    })
}

fn rejected(
    status: RejectionStatus,
    cursor: &mut ByteCursor<'_>,
) -> Result<ResponseOutcome, String> {
    let message = String::from_utf8_lossy(cursor.length_prefixed("拒绝说明")?).into_owned();
    Ok(ResponseOutcome::Rejected { status, message })
}

fn decode_completed_payload(
    operation: OperationCode,
    cursor: &mut ByteCursor<'_>,
) -> Result<ResponseOutcome, String> {
    match operation {
        OperationCode::BatchedPointRead => {
            let value_count = cursor.u32("值的个数")?;
            let mut values = Vec::with_capacity(cursor.capacity_for(value_count, 1));
            for _ in 0..value_count {
                match cursor.u8("在不在")? {
                    0 => values.push(None),
                    1 => values.push(Some(cursor.length_prefixed("值")?.to_vec())),
                    other => return Err(format!("「在不在」只能是 0 或 1，收到 {other}")),
                }
            }
            Ok(ResponseOutcome::PointValues(values))
        }
        OperationCode::BatchedWrite => {
            let entries_written = cursor.u32("写了几条")?;
            let is_already_applied = match cursor.u8("早已生效标记")? {
                0 => false,
                1 => true,
                other => return Err(format!("早已生效标记只能是 0 或 1，收到 {other}")),
            };
            Ok(ResponseOutcome::WriteAcknowledged {
                entries_written,
                is_already_applied,
            })
        }
        OperationCode::Claim => {
            let outcome_count = cursor.u32("认领结局个数")?;
            let mut outcomes = Vec::with_capacity(cursor.capacity_for(outcome_count, 1));
            for _ in 0..outcome_count {
                match cursor.u8("认领结局")? {
                    0 => outcomes.push(ClaimOutcome::Claimed),
                    1 => {
                        let holder_identity = cursor.client_identity()?.to_vec();
                        let remaining_lease_milliseconds = cursor.u64("剩余租期毫秒")?;
                        outcomes.push(ClaimOutcome::HeldByOther {
                            holder_identity,
                            remaining_lease_milliseconds,
                        });
                    }
                    2 => outcomes.push(ClaimOutcome::AlreadyConcluded),
                    other => return Err(format!("认领结局只能是 0、1、2，收到 {other}")),
                }
            }
            Ok(ResponseOutcome::Claims(outcomes))
        }
        OperationCode::PrefixScan => {
            let is_truncated = match cursor.u8("截断标记")? {
                0 => false,
                1 => true,
                other => return Err(format!("截断标记只能是 0 或 1，收到 {other}")),
            };
            let entry_count = cursor.u32("条数")?;
            let mut entries = Vec::with_capacity(cursor.capacity_for(entry_count, 8));
            for _ in 0..entry_count {
                let key = cursor.length_prefixed("键")?.to_vec();
                let value = cursor.length_prefixed("值")?.to_vec();
                entries.push(ScanEntry { key, value });
            }
            Ok(ResponseOutcome::ScanEntries {
                is_truncated,
                entries,
            })
        }
        OperationCode::DeletePrefix => {
            let keys_in_frame = cursor.u32("这一帧碰到几把键")?;
            let is_exhausted = match cursor.u8("前缀下没有更多标记")? {
                0 => false,
                1 => true,
                other => return Err(format!("前缀下没有更多标记只能是 0 或 1，收到 {other}")),
            };
            let last_key = cursor.length_prefixed("最后一把键")?.to_vec();
            Ok(ResponseOutcome::PrefixDeleted {
                keys_in_frame,
                is_exhausted,
                last_key,
            })
        }
        OperationCode::Statistics => {
            let counter_count = cursor.u32("计数的个数")?;
            let mut counters = Vec::with_capacity(cursor.capacity_for(counter_count, 12));
            for _ in 0..counter_count {
                let name = String::from_utf8_lossy(cursor.length_prefixed("计数名")?).into_owned();
                let value = cursor.u64("计数值")?;
                counters.push(StatisticsCounter { name, value });
            }
            Ok(ResponseOutcome::Statistics(counters))
        }
        OperationCode::ConnectedClients | OperationCode::AnnounceStop => {
            let client_count = cursor.u32("客户端个数")?;
            let mut clients = Vec::with_capacity(cursor.capacity_for(client_count, 22));
            for _ in 0..client_count {
                let connection = cursor.u64("连接号")?;
                let transport_code = cursor.u8("走哪条")?;
                let transport = Transport::from_code(transport_code)
                    .ok_or_else(|| format!("认不得的「走哪条」{transport_code}"))?;
                let peer_address =
                    String::from_utf8_lossy(cursor.length_prefixed("对端地址")?).into_owned();
                let announcement = match cursor.u8("报到过没有")? {
                    0 => None,
                    1 => Some(cursor.announcement()?),
                    other => return Err(format!("「报到过没有」只能是 0 或 1，收到 {other}")),
                };
                let connected_since_milliseconds = cursor.u64("连上的时刻")?;
                clients.push(ConnectedClient {
                    connection,
                    transport,
                    peer_address,
                    announcement,
                    connected_since_milliseconds,
                });
            }
            Ok(ResponseOutcome::ConnectedClients(clients))
        }
        OperationCode::ConnectionAnnouncement => Err("报到不回应，服务回了做完的报到".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 换了租约的认领帧，服务解出来的租约是换上的那个数，身份与键不变。
    /// 判别力（`crates/mutations.tsv`）：租约的位置少算客户端身份的长度字节，这条红。
    #[test]
    fn a_claim_frame_with_its_lease_replaced_decodes_to_the_new_lease_and_the_same_keys() {
        let keys: [(Table, &[u8]); 2] = [
            (Table::Conclusions, b"first"),
            (Table::Conclusions, b"second"),
        ];
        let mut frame = encode_claim_frame(7, b"client-identity", 10_000, &keys);
        replace_claim_lease(&mut frame, 1_234);
        let decoded = decode_request_body(
            &frame[FRAME_LENGTH_FIELD_BYTES..],
            TrailingBytes::Rejected,
            ServiceKeys::Refused,
        )
        .expect("换了租约的认领帧照样解得开");
        let RequestView::Claim {
            client_identity,
            lease_milliseconds,
            keys: decoded_keys,
        } = decoded.view
        else {
            panic!("解出来不是认领：{decoded:?}");
        };
        assert_eq!(lease_milliseconds, 1_234, "租约是换上的那个数");
        assert_eq!(client_identity, b"client-identity");
        assert_eq!(decoded_keys.len(), 2);
    }

    fn decode_delete_prefix(
        prefix: &[u8],
        after_key: &[u8],
        maximum_keys: u32,
    ) -> Result<DecodedRequest<'static>, RequestRejection> {
        let frame = encode_delete_prefix_frame(3, prefix, after_key, maximum_keys, true);
        // 解出来的视图借用帧体：测试里泄漏一份，省得把帧体的生命周期带进辅助函数的签名
        let body: &'static [u8] = Box::leak(
            frame[FRAME_LENGTH_FIELD_BYTES..]
                .to_vec()
                .into_boxed_slice(),
        );
        decode_request_body(body, TrailingBytes::Rejected, ServiceKeys::Refused)
            .map_err(|(_, rejection)| rejection)
    }

    /// 删前缀帧解出来的前缀、续扫起点、本帧上限与只数标记，与编进去的相同。
    #[test]
    fn a_delete_prefix_frame_decodes_to_what_was_encoded() {
        let decoded =
            decode_delete_prefix(b"\x1cabc", b"\x1cabc/last", 7).expect("合规的删前缀帧解得开");
        assert_eq!(
            decoded.view,
            RequestView::DeletePrefix {
                prefix: b"\x1cabc",
                after_key: b"\x1cabc/last",
                maximum_keys: 7,
                count_only: true,
            }
        );
    }

    /// 空前缀、不以前缀起头的续扫起点、0 与超上限的本帧上限、落在服务记账那一段的前缀，各被拒成内容解不开。
    #[test]
    fn a_delete_prefix_frame_with_a_prefix_that_would_clear_the_library_or_a_bad_bound_is_refused()
    {
        let over_limit = MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME + 1;
        for (description, prefix, after_key, maximum_keys) in [
            ("空前缀", b"".as_slice(), b"".as_slice(), 10),
            (
                "续扫起点不以前缀起头",
                b"abc".as_slice(),
                b"abd/x".as_slice(),
                10,
            ),
            ("本帧至多 0 把", b"abc".as_slice(), b"".as_slice(), 0),
            (
                "本帧至多超过上限",
                b"abc".as_slice(),
                b"".as_slice(),
                over_limit,
            ),
            (
                "前缀落在服务记账那一段",
                b"\xF1abc".as_slice(),
                b"".as_slice(),
                10,
            ),
        ] {
            let rejection =
                decode_delete_prefix(prefix, after_key, maximum_keys).expect_err(description);
            assert!(
                matches!(rejection, RequestRejection::MalformedPayload { .. }),
                "{description}：该被拒成内容解不开，实际 {rejection:?}"
            );
        }
    }
}
