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
//!   本帧至多几把在 1 到 `MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME` 之间。提交线程里成批做：同一组里排在前面的写与真删先提交，
//!   删的是已落盘的库；删的键与回应同一组提交、WAL 同步之后才回，删掉的键上的认领同一组释放。回应（带最后一把键）记不进服务的待处理额度时
//!   整帧回忙（状态 5）、一把都不删（`committer.rs` 文件头「删前缀」）。只数不删时不改库、不动认领，数的口径与删一样。
//!   删本来就幂等，所以不按（身份，请求号）去重，也不进客户端的追加日志：重发只会让某一帧报的数少一批，库里的结果不变。
//! - 不在才写（操作码 10）：内容与成批写相同。去重与成批写相同；没生效过的逐条看这把键在不在已落盘的库里（同一帧里同一把键写几次只认头一次）：
//!   不在的写、在的不动；同一批里记下「这个请求号已生效」，记的条数是真写了的条数；WAL 同步之后回确认，回应与成批写相同（写了几条是真写了的）。
//!   不释放认领。提交线程做它之前先把这一组里改过数据行的写提交掉，读的是已落盘的库。GPU 待判行（`gpu_judging_rows` 文件头）走它。
//! - 写结论（操作码 11）：`u8 身份长`、身份、`u64 请求号`、`u8 持有者身份长`、持有者身份（1 到 255 字节）、`u32 认领键长`、认领键、
//!   `u32 守卫键长`、守卫键、`u8 守卫那一行的期待`（0 不在；1 在，再跟 32 字节值的 SHA-256）、`u32 条数`、每条与成批写相同、`u32 删键数`、
//!   每把 `u32 键长`、键、`u8 这一行的期待`（与守卫那一行的期待同一种写法：写的一方读到的那一份）。
//!   回状态 1 的写法：认领键或守卫键空；守卫键在条目里出现不止一次、或出现了而不是最后一条（一份结论有多行时守卫放在最后落的那一行）；
//!   认领键在条目或删键里（它只记认领，写了它之后认领它就回「已有结论」）；删键空、与某条条目的键相同、或就是守卫键；任一把键落在服务记账那一段。
//!   提交线程做它之前先把这一组里改过数据行的写提交掉（核的是已落盘的库），再照次序判：
//!   1. 按（身份，请求号）去重：早已生效的照回确认（带「早已生效」），不再核认领与守卫那一行；
//!   2. 核认领：认领键被别的持有者持着、租期没到，整帧拒（状态 7 种类 1）；租期过了、持有者就是帧里的持有者、没人持着，往下走；
//!   3. 核守卫那一行：库里那一行此刻的样子（不在，或值的 SHA-256）与帧里的期待不同，整帧拒（状态 7 种类 2，带此刻的样子），不覆盖、不悄悄丢；
//!   4. 核删键：逐把比库里那一行此刻的样子与帧里给它的期待，有一把不同就整帧拒（状态 7 种类 3，带那一把键与此刻的样子）：
//!      删的只是写的一方读到的那一份，读完之后被别人改过的（待判行被判段升成强制现判）不删；
//!   5. 都过了：条目写进、删键删掉、认领键与条目键、删键上的认领释放、记下请求号已生效，同一组提交、WAL 同步之后回确认（与成批写相同）。
//!
//!   GPU 判出的结论（`gpu_judging_rows` 文件头：认领键是 `gpu-claim/` 那一行，删键是 `gpu-pending/` 那一行，强制现判另带一条 `gpu-rejudged/`）走它。
//! - 放认领（操作码 12）：`u8 持有者身份长`、持有者身份（1 到 255 字节）、`u32 键数`，每键 `u8 表`、`u32 键长`、键。键一律是 GPU 认领键
//!   （`gpu_judging_rows` 文件头，`gpu-claim/` 起头），有一把不是就整帧回状态 1。逐键看认领表：这把键上记的持有者就是帧里的持有者（租期到没到都算）
//!   的放掉，回「放掉了」；没有认领记录回「没人持着」；记的是别的持有者回「别人持有」、不动它（只放点名的持有者名下的）。
//!   放掉的与回应同一组提交、WAL 同步之后才回。不按（身份，请求号）去重、不进客户端的追加日志：放本来就幂等，重发只会把「放掉了」报成「没人持着」，
//!   重发之前别的持有者认领走的照「别人持有」不动。GPU 批处理服务起来时以上一个进程的持有者放它留下的认领、停的时候放自己的、
//!   认领到之后重读待判行不在了放刚认领到的（`gpu-verification-server/src/library_pending_pool.rs` 文件头「认领」）。
//! - 列认领（操作码 13）：`u32 前缀长`、前缀、`u32 续扫起点长`、续扫起点、`u32 条数上限`（至少 1，0 回状态 1）。前缀要以 GPU 认领键的族前缀（`gpu-claim/`，
//!   `gpu_judging_rows` 文件头）起头，不然回状态 1；续扫起点空是从前缀起，不空时必须以前缀起头（不然回状态 1），从排在它之后的第一把键起列（上一帧回的最后一把键）。
//!   只看提交线程内存里的认领表：键以前缀起头、租期此刻没到的认领按键升序列出，过了租期还留在表里的不列；每条带键（客户端的键，不带记账那一段的标签）、
//!   持有者身份与剩余租期。不改库、不动认领、不进客户端的追加日志，所以不按（身份，请求号）去重，断线重发无妨。
//!   一帧被条数上限或帧长上限截断时，客户端拿这一帧的最后一把键接着列；连一条都装不下一帧（键长逼近帧长上限）回状态 1。
//!   回应的字节先记进待处理额度，记不进整帧回忙（什么都没改，重发即可）；回应排在这一组后面、WAL 同步之后才回，所以回应里的认领都已同步进 WAL。
//!
//! 回应帧：`[u32 帧体长度][u64 请求序号][u8 状态][u8 操作码][内容]`。状态 0 是做完，内容按操作码：
//! - 成批点读：`u32 个数`，每个 `u8 在不在`（0 / 1），在的再跟 `u32 值长`、值，次序与请求里的键相同；
//! - 成批写、不在才写、写结论：`u32 写了几条`、`u8 早已生效`（1：这个请求号以前生效过，这一次没再写）；
//! - 按前缀扫：`u8 有没有被条数上限或帧长上限截断`、`u32 条数`，每条 `u32 键长`、键、`u32 值长`、值，按键升序；
//! - 统计：`u32 个数`，每个 `u32 名字长`、名字、`u64 值`；
//! - 认领：`u32 个数`，每个 `u8 结局`：0 认领到；1 别人持有，再跟 `u8 持有者身份长`、身份、`u64 剩余租期毫秒`；2 已有结论。
//! - 放认领：`u32 个数`，每个 `u8 结局`：0 放掉了；1 没人持着；2 别人持有（没放）。一键一字节，回应不长过请求，不另核帧长上限。
//! - 列认领：`u8 有没有被条数上限或帧长上限截断`、`u32 条数`，每条 `u32 键长`、键、`u8 持有者身份长`、身份、`u64 剩余租期毫秒`，按键升序。
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
//! 状态 7 是守卫不过（只回写结论）：内容是 `u8 种类`，1 认领被别的持有者持着、租期没到，再跟 `u8 持有者身份长`、持有者身份、`u64 剩余租期毫秒`；
//! 2 守卫那一行此刻的样子与帧里的期待不同，再跟 `u8 在不在`（0 / 1）、在的再跟 32 字节值的 SHA-256；
//! 3 一把删键那一行此刻的样子与帧里给它的期待不同，再跟 `u32 键长`、那一把键、`u8 在不在`、在的再跟 32 字节值的 SHA-256；之后 `u32 说明长`、说明。
//! 这一帧一条都没写、连接照常可用。它不是协议错：容错客户端从追加日志里勾掉这一帧、把种类交给调用方（`reliable_client` 文件头「写结论」），
//! 调用方照种类定：认领被别人持着就不写；守卫那一行变了就重读那一行、照它自己的规则比（两份都绿而不同报红，有一份红整份换时带重读到的那一份再写）；
//! 删键那一行变了就重读那一行、照它自己的规则定删不删、带重读到的那一份再写。
//! 状态 5 是忙：这条连接或整个服务的待处理帧数（或待处理字节数）到了上限，这一帧没有排队、没有做；内容是
//! `u32 多少毫秒后再来`、`u32 说明长`、说明。客户端照这个毫秒数加抖动退避之后重发同一帧（写帧带同一个请求号）。
//! 待处理字节数记着收着的帧体与没写出去的回应（`server.rs` 文件头「额度」）：收帧时记不进的，帧体照读完丢掉、回忙；
//! 读、扫、统计的回应编好了记不进的，这一帧做过、回应没留，回忙（不改库，重发即可）；认领的回应记不进的，一个键都没认领、回忙。
//!
//! 流水：客户端不等回应接着发；读、扫、统计由几个工作线程并行处理，写与认领由一个提交线程按到达次序组提交，
//! 回应按完成先后写回，可能乱序，客户端按回应里的序号对回请求。同一条连接上先写后读同一个键要等写的回应到了再发读；
//! 同一个客户端两帧写同一个键，回了忙的那一帧重发之后可能落在后发的那一帧之后，要定次序就等前一帧的确认再发后一帧。

use crate::gpu_judging_rows::GPU_CLAIM_FAMILY_PREFIX;
use crate::sha256::{sha256_digest, DIGEST_BYTES};

mod connection_level_frames;
mod request_decoding;
mod request_encoding;
mod response_decoding;
mod response_encoding;

pub use self::connection_level_frames::*;
pub use self::request_decoding::*;
pub use self::request_encoding::*;
pub use self::response_decoding::*;
pub use self::response_encoding::*;

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
/// 写结论帧里守卫那一行的样子带的值摘要的字节数（SHA-256，文件头「写结论」）。
pub const GUARD_ROW_DIGEST_BYTES: usize = DIGEST_BYTES;
/// 守卫那一行的样子的编码：`u8` 0 不在。
const GUARD_ROW_ABSENT_CODE: u8 = 0;
/// 同上：`u8` 1 在，再跟值的 SHA-256。
const GUARD_ROW_PRESENT_CODE: u8 = 1;
/// 守卫不过（状态 7）的种类：认领被别的持有者持着、租期没到。
const GUARD_REFUSAL_CLAIM_HELD_BY_OTHER_CODE: u8 = 1;
/// 守卫不过（状态 7）的种类：守卫那一行此刻的样子与帧里的期待不同。
const GUARD_REFUSAL_GUARD_ROW_CHANGED_CODE: u8 = 2;
/// 守卫不过（状态 7）的种类：一把删键那一行此刻的样子与帧里给它的期待不同。
const GUARD_REFUSAL_DELETED_ROW_CHANGED_CODE: u8 = 3;

/// 守卫那一行的样子（文件头「写结论」）：不在，或值的 SHA-256。写结论帧里是写的一方读到的那一份，守卫不过的回应里是服务此刻读到的那一份。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum GuardRowState {
    Absent,
    Present {
        value_digest: [u8; GUARD_ROW_DIGEST_BYTES],
    },
}

impl GuardRowState {
    /// 读到的一行（`None` 是不在）的样子。
    #[must_use]
    pub fn of_stored_value(value: Option<&[u8]>) -> GuardRowState {
        match value {
            None => GuardRowState::Absent,
            Some(stored) => GuardRowState::Present {
                value_digest: sha256_digest(stored),
            },
        }
    }

    fn put(self, encoder: &mut FrameEncoder) {
        match self {
            GuardRowState::Absent => encoder.put_u8(GUARD_ROW_ABSENT_CODE),
            GuardRowState::Present { value_digest } => {
                encoder.put_u8(GUARD_ROW_PRESENT_CODE);
                encoder.bytes.extend_from_slice(&value_digest);
            }
        }
    }
}

/// 写结论帧守卫不过（状态 7）的两种（文件头「回应帧」状态 7）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GuardRefusal {
    /// 认领键被别的持有者持着、租期没到。
    ClaimHeldByOther {
        holder_identity: Vec<u8>,
        remaining_lease_milliseconds: u64,
    },
    /// 守卫那一行此刻的样子（`current`）与写的一方读到的那一份不同。
    GuardRowChanged { current: GuardRowState },
    /// 删键 `key` 那一行此刻的样子（`current`）与写的一方读到的那一份不同。
    DeletedRowChanged {
        key: Vec<u8>,
        current: GuardRowState,
    },
}

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
    WriteIfAbsent,
    GuardedConclusionWrite,
    ReleaseClaims,
    ListClaims,
}

impl OperationCode {
    pub const ALL: [OperationCode; 13] = [
        OperationCode::BatchedPointRead,
        OperationCode::BatchedWrite,
        OperationCode::PrefixScan,
        OperationCode::Statistics,
        OperationCode::Claim,
        OperationCode::ConnectionAnnouncement,
        OperationCode::ConnectedClients,
        OperationCode::AnnounceStop,
        OperationCode::DeletePrefix,
        OperationCode::WriteIfAbsent,
        OperationCode::GuardedConclusionWrite,
        OperationCode::ReleaseClaims,
        OperationCode::ListClaims,
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
            OperationCode::WriteIfAbsent => 10,
            OperationCode::GuardedConclusionWrite => 11,
            OperationCode::ReleaseClaims => 12,
            OperationCode::ListClaims => 13,
        }
    }

    /// 写（成批写、不在才写、写结论）、认领、放认领与删前缀要改库（或要读已落盘的库），列认领要读认领表（只在提交线程的内存里），
    /// 这几种交提交线程按到达次序组提交；别的交工作线程并行做，或由读线程当场做（报到、列客户端、宣布停服务）。
    pub fn is_handled_by_committer(self) -> bool {
        match self {
            OperationCode::BatchedWrite
            | OperationCode::Claim
            | OperationCode::DeletePrefix
            | OperationCode::WriteIfAbsent
            | OperationCode::GuardedConclusionWrite
            | OperationCode::ReleaseClaims
            | OperationCode::ListClaims => true,
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
            | OperationCode::DeletePrefix
            | OperationCode::WriteIfAbsent
            | OperationCode::GuardedConclusionWrite
            | OperationCode::ReleaseClaims
            | OperationCode::ListClaims => false,
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
    GuardRefused,
}

impl ResponseStatus {
    pub const ALL: [ResponseStatus; 8] = [
        ResponseStatus::Completed,
        ResponseStatus::MalformedPayload,
        ResponseStatus::UnknownOperation,
        ResponseStatus::FrameLengthRejected,
        ResponseStatus::StorageFailed,
        ResponseStatus::Busy,
        ResponseStatus::ServerStopping,
        ResponseStatus::GuardRefused,
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
            ResponseStatus::GuardRefused => 7,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 换了租约的认领帧，服务解出来的租约是换上的那个数，身份与键不变。
    /// 判别力（`crates/mutations.tsv`）：租约的位置少算客户端身份的长度字节，这条红。
    #[test]
    fn claim_frame_with_its_lease_replaced_decodes_to_the_new_lease_and_the_same_keys() {
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
    fn delete_prefix_frame_decodes_to_what_was_encoded() {
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
    fn delete_prefix_frame_with_a_prefix_that_would_clear_the_library_or_a_bad_bound_is_refused() {
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

    fn sample_guarded_conclusion_write() -> GuardedConclusionWrite {
        GuardedConclusionWrite {
            holder_identity: b"holder".to_vec(),
            claim_key: b"gpu-claim/aa/0000000000000001".to_vec(),
            guard_key: b"\x10conclusion-block".to_vec(),
            guard_expectation: GuardRowState::of_stored_value(Some(b"what the writer read")),
            entries: vec![
                (
                    Table::Conclusions,
                    b"\x11side-row".to_vec(),
                    b"side".to_vec(),
                ),
                (
                    Table::Conclusions,
                    b"\x10conclusion-block".to_vec(),
                    b"block".to_vec(),
                ),
            ],
            deleted_keys: vec![GuardedDeletion {
                key: b"gpu-pending/aa/0000000000000001".to_vec(),
                expectation: GuardRowState::of_stored_value(Some(b"the pending row as read")),
            }],
        }
    }

    fn decode_guarded_conclusion_write(
        request: &GuardedConclusionWrite,
    ) -> Result<DecodedRequest<'static>, RequestRejection> {
        let frame = encode_guarded_conclusion_write_frame(5, b"client", 9, request);
        // 同 decode_delete_prefix：测试里泄漏一份帧体，解出来的视图借用它
        let body: &'static [u8] = Box::leak(
            frame[FRAME_LENGTH_FIELD_BYTES..]
                .to_vec()
                .into_boxed_slice(),
        );
        decode_request_body(body, TrailingBytes::Rejected, ServiceKeys::Refused)
            .map_err(|(_, rejection)| rejection)
    }

    /// 写结论帧解出来的持有者、认领键、守卫键、期待、条目与删键，与编进去的相同。
    #[test]
    fn guarded_conclusion_write_frame_decodes_to_what_was_encoded() {
        let request = sample_guarded_conclusion_write();
        let decoded = decode_guarded_conclusion_write(&request).expect("合规的写结论帧解得开");
        let RequestView::GuardedConclusionWrite(conclusion) = decoded.view else {
            panic!("解出来不是写结论：{decoded:?}");
        };
        assert_eq!(conclusion.client_identity, b"client");
        assert_eq!(conclusion.request_number, 9);
        assert_eq!(conclusion.holder_identity, b"holder");
        assert_eq!(conclusion.claim_key, request.claim_key.as_slice());
        assert_eq!(conclusion.guard_key, request.guard_key.as_slice());
        assert_eq!(conclusion.guard_expectation, request.guard_expectation);
        let decoded_entries: Vec<(&[u8], &[u8])> = conclusion
            .entries
            .iter()
            .map(|entry| (entry.key, entry.value))
            .collect();
        let encoded_entries: Vec<(&[u8], &[u8])> = request
            .entries
            .iter()
            .map(|(_, key, value)| (key.as_slice(), value.as_slice()))
            .collect();
        assert_eq!(decoded_entries, encoded_entries);
        assert_eq!(
            conclusion.deleted_keys,
            vec![GuardedDeletionView {
                key: request.deleted_keys[0].key.as_slice(),
                expectation: request.deleted_keys[0].expectation,
            }]
        );
    }

    /// 守卫键不是最后一条、写了两次、认领键当条目写、删键就是条目的键：各被拒成内容解不开（文件头「写结论」）。
    #[test]
    fn guarded_conclusion_write_with_the_guard_row_not_last_or_a_claim_key_written_is_refused() {
        let guard_row_in_front = {
            let mut request = sample_guarded_conclusion_write();
            request.entries.reverse();
            request
        };
        let guard_row_twice = {
            let mut request = sample_guarded_conclusion_write();
            let guard_row = request.entries[1].clone();
            request.entries.insert(0, guard_row);
            request
        };
        let claim_key_written = {
            let mut request = sample_guarded_conclusion_write();
            let claim_row = (Table::Conclusions, request.claim_key.clone(), b"x".to_vec());
            request.entries.insert(0, claim_row);
            request
        };
        let deleted_key_written = {
            let mut request = sample_guarded_conclusion_write();
            request.deleted_keys.push(GuardedDeletion {
                key: b"\x11side-row".to_vec(),
                expectation: GuardRowState::Absent,
            });
            request
        };
        for (description, request) in [
            ("守卫那一行排在最前", guard_row_in_front),
            ("守卫那一行写了两次", guard_row_twice),
            ("认领键当条目写", claim_key_written),
            ("删键是某条条目的键", deleted_key_written),
        ] {
            let rejection = decode_guarded_conclusion_write(&request).expect_err(description);
            assert!(
                matches!(rejection, RequestRejection::MalformedPayload { .. }),
                "{description}：该被拒成内容解不开，实际 {rejection:?}"
            );
        }
    }

    /// 守卫不过的三种回应解出来与编进去的相同。
    #[test]
    fn guard_refusal_response_decodes_to_what_was_encoded() {
        for refusal in [
            GuardRefusal::DeletedRowChanged {
                key: b"gpu-pending/aa/0000000000000001".to_vec(),
                current: GuardRowState::of_stored_value(Some(b"raised to forced")),
            },
            GuardRefusal::DeletedRowChanged {
                key: b"gpu-pending/aa/0000000000000002".to_vec(),
                current: GuardRowState::Absent,
            },
            GuardRefusal::ClaimHeldByOther {
                holder_identity: b"another-holder".to_vec(),
                remaining_lease_milliseconds: 1_234,
            },
            GuardRefusal::GuardRowChanged {
                current: GuardRowState::Absent,
            },
            GuardRefusal::GuardRowChanged {
                current: GuardRowState::of_stored_value(Some(b"stored")),
            },
        ] {
            let frame = encode_guard_refusal(4, &refusal, "说明");
            let response =
                decode_response_body(&frame[FRAME_LENGTH_FIELD_BYTES..]).expect("解得开");
            assert_eq!(
                response.outcome,
                ResponseOutcome::GuardRefused {
                    refusal,
                    message: "说明".to_string(),
                }
            );
        }
    }

    /// 放认领帧解出来的持有者与键与编进去的相同，三种结局的回应解出来与编进去的相同。
    #[test]
    fn claim_release_frame_and_its_outcomes_decode_to_what_was_encoded() {
        let keys: [(Table, &[u8]); 2] = [
            (Table::Conclusions, b"gpu-claim/aa/0000000000000001"),
            (Table::Conclusions, b"gpu-claim/aa/0000000000000002"),
        ];
        let frame = encode_release_claims_frame(5, b"previous-holder", &keys);
        let decoded = decode_request_body(
            &frame[FRAME_LENGTH_FIELD_BYTES..],
            TrailingBytes::Rejected,
            ServiceKeys::Refused,
        )
        .expect("放认领帧解得开");
        let RequestView::ReleaseClaims {
            holder_identity,
            keys: decoded_keys,
        } = decoded.view
        else {
            panic!("解出来不是放认领：{decoded:?}");
        };
        assert_eq!(holder_identity, b"previous-holder");
        let decoded_key_bytes: Vec<&[u8]> = decoded_keys.iter().map(|key| key.key).collect();
        assert_eq!(decoded_key_bytes, vec![keys[0].1, keys[1].1]);
        let outcomes = ClaimReleaseOutcome::ALL.to_vec();
        let response = encode_claim_release_outcomes(6, &outcomes);
        assert_eq!(
            decode_response_body(&response[FRAME_LENGTH_FIELD_BYTES..])
                .expect("解得开")
                .outcome,
            ResponseOutcome::ClaimsReleased(outcomes)
        );
    }

    /// 放认领帧里有一把不是 GPU 认领键（`gpu-claim/` 起头）：整帧被拒成内容解不开（文件头「放认领」只放 GPU 认领键）。
    #[test]
    fn claim_release_frame_with_a_key_outside_the_gpu_claims_is_refused() {
        let keys: [(Table, &[u8]); 2] = [
            (Table::Conclusions, b"gpu-claim/aa/0000000000000001"),
            (Table::Conclusions, b"gpu-pending/aa/0000000000000001"),
        ];
        let frame = encode_release_claims_frame(5, b"previous-holder", &keys);
        let rejection = decode_request_body(
            &frame[FRAME_LENGTH_FIELD_BYTES..],
            TrailingBytes::Rejected,
            ServiceKeys::Refused,
        )
        .map(|decoded| format!("{decoded:?}"))
        .expect_err("待判行的键不是认领键：该整帧拒");
        assert!(
            matches!(rejection.1, RequestRejection::MalformedPayload { .. }),
            "该被拒成内容解不开，实际 {rejection:?}"
        );
    }

    /// 列认领帧解出来的前缀、续扫起点与条数上限与编进去的相同；回应解出来的条目与截断标记与编进去的相同。
    #[test]
    fn claim_listing_frame_and_its_response_decode_to_what_was_encoded() {
        let frame = encode_list_claims_frame(
            7,
            b"gpu-claim/temporary-library/",
            b"gpu-claim/temporary-library/aa",
            25,
        );
        let decoded = decode_request_body(
            &frame[FRAME_LENGTH_FIELD_BYTES..],
            TrailingBytes::Rejected,
            ServiceKeys::Refused,
        )
        .expect("列认领帧解得开");
        assert_eq!(
            decoded.view,
            RequestView::ListClaims {
                prefix: b"gpu-claim/temporary-library/",
                after_key: b"gpu-claim/temporary-library/aa",
                maximum_entries: 25,
            }
        );
        let claims = vec![
            ListedClaim {
                key: b"gpu-claim/temporary-library/aa/bb".to_vec(),
                holder_identity: b"holder-one".to_vec(),
                remaining_lease_milliseconds: 1_500,
            },
            ListedClaim {
                key: b"gpu-claim/temporary-library/cc".to_vec(),
                holder_identity: b"holder-two".to_vec(),
                remaining_lease_milliseconds: 30_000,
            },
        ];
        for is_truncated in [false, true] {
            let response = encode_claim_listing(8, is_truncated, &claims);
            assert_eq!(
                decode_response_body(&response[FRAME_LENGTH_FIELD_BYTES..])
                    .expect("解得开")
                    .outcome,
                ResponseOutcome::ClaimListing {
                    is_truncated,
                    claims: claims.clone(),
                }
            );
        }
    }

    /// 列认领帧的条数上限是 0、续扫起点不以前缀起头、前缀落在服务记账那一段：都整帧被拒成内容解不开。
    #[test]
    fn claim_listing_frames_with_a_zero_limit_a_foreign_resume_key_or_a_service_prefix_are_refused()
    {
        for (description, frame) in [
            (
                "条数上限 0",
                encode_list_claims_frame(7, b"gpu-claim/", b"", 0),
            ),
            (
                "续扫起点不以前缀起头",
                encode_list_claims_frame(7, b"gpu-claim/a", b"gpu-claim/b", 10),
            ),
            (
                "前缀落在记账那一段",
                encode_list_claims_frame(7, &[0xF1, b'g'], b"", 10),
            ),
        ] {
            let rejection = decode_request_body(
                &frame[FRAME_LENGTH_FIELD_BYTES..],
                TrailingBytes::Rejected,
                ServiceKeys::Refused,
            )
            .map(|decoded| format!("{decoded:?}"))
            .expect_err(description);
            assert!(
                matches!(rejection.1, RequestRejection::MalformedPayload { .. }),
                "{description}：该被拒成内容解不开，实际 {rejection:?}"
            );
        }
    }
}
