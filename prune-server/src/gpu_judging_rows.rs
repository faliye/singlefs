//! GPU 判的三族键与行：待判行、认领、强制现判行。GPU 批处理服务（`gpu-verification-server/`）与 CPU 一侧的判段照这一份读写判定库，
//! 帧怎么走在 `protocol.rs` 文件头「不在才写」「写结论」，容错客户端的调用在 `reliable_client.rs` 文件头「GPU 判的三族」；
//! Python 一侧照同样的字节写（`scripts/prune-server/prune_server_client.py` 文件头「GPU 判的三族」），两边的键与值逐字节相同。
//!
//! 键（ASCII，用结论表的名义读写；与别的族同一个键空间、互不相撞：崩溃行以表标签 0x00–0x1B 起头，Python 五族以各自的族名起头）：
//! - 待判行 `gpu-pending/<流键>/<块号>`；
//! - 认领键 `gpu-claim/<流键>/<块号>`：只经认领帧认领、经写结论帧释放，不当数据行写（写了它，之后认领它就回「已有结论」）；
//! - 强制现判行 `gpu-rejudged/<入池序号>/<流键>/<块号>`。
//!
//! 流键是字节串（可能带 `/`），键里写成小写十六进制；块号与入池序号写成 16 位小写十六进制、补零，按键升序扫就是按号升序。
//!
//! 待判行的值（`GpuPendingRow`）：`u32 长`+录入表的键、`u32 长`+判表摘要、`u64 状态序号区间的起`、`u64 止`（不含）、
//! `u32 长`+期待的 GPU 判器组键、`u32 长`+期待的结论版本键、`u8 强制现判`（0 / 1）、`u64 入池序号`；整数小端，解完不许剩字节。
//! 行只指向录入表，不另抄旋钮与判表。CPU 一侧照不在才写写（同一块的行已在就不重写）；写结论那一帧里删掉；「判完」不存，照库上现算。
//!
//! 认领：持有者身份 `gpu-service/<随机号>/card-<卡序号>/thread-<判线程号>`（`GpuClaimHolder::identity`），随机号是服务进程起来时取的
//! 64 位数、16 位小写十六进制（`draw_service_start_random`）：每次进程另取一个，重起之后的进程不认上一个进程持着的认领。
//!
//! 写结论（`GpuConclusion::guarded_write`）：写结论帧的认领键是这一块的 `gpu-claim/`、删键是这一块的 `gpu-pending/`（期待是写的一方读到的那一行待判行，
//! 判段读完之后把它升成强制现判的不删），守卫键是结论块那一行，期待是写的一方读到的那一份；先落的旁表行在前、结论块那一行排最后一条。强制现判的块另带一条 `gpu-rejudged/` 行（排在结论块那一行之前），
//! 值（`GpuRejudgedRow`）是 `u8 长`+持有者身份、32 字节结论摘要：结论块那一行的值的 SHA-256，帧里写了结论块取写的值，
//! 没写（库里那一份与判出的相同、不重写）取帧里的期待。CPU 一侧核这一行在、结论摘要与库里结论块那一行的 SHA-256 相同。

use std::fmt::Write as _;
use std::fs::File;
use std::io::Read;

use crate::protocol::{
    ByteCursor, GuardRowState, GuardedConclusionWrite, GuardedDeletion, Table,
    GUARD_ROW_DIGEST_BYTES, MAXIMUM_CLIENT_IDENTITY_BYTES,
};
use crate::reliable_client::TableEntry;
use crate::sha256::sha256_digest;

/// 待判行的族前缀。
pub const GPU_PENDING_FAMILY_PREFIX: &[u8] = b"gpu-pending/";
/// 认领键的族前缀。
pub const GPU_CLAIM_FAMILY_PREFIX: &[u8] = b"gpu-claim/";
/// 强制现判行的族前缀。
pub const GPU_REJUDGED_FAMILY_PREFIX: &[u8] = b"gpu-rejudged/";
/// 键里各段之间的分隔字节（流键写成十六进制，不会带它）。
const KEY_FIELD_SEPARATOR: u8 = b'/';
/// 块号与入池序号在键里的十六进制位数。
const NUMBER_FIELD_HEXADECIMAL_DIGITS: usize = 16;
/// 持有者身份里服务进程随机号的前缀。
const HOLDER_IDENTITY_PREFIX: &str = "gpu-service/";
/// 取服务进程随机号读的系统随机源。
const SYSTEM_RANDOM_SOURCE: &str = "/dev/urandom";

/// 一块：哪条流（流键）、第几块（块号）。
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct GpuBlockAddress {
    pub stream_key: Vec<u8>,
    pub block_number: u64,
}

fn lowercase_hexadecimal(bytes: &[u8]) -> String {
    let mut hexadecimal = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(hexadecimal, "{byte:02x}").expect("往 String 里写不会失败");
    }
    hexadecimal
}

fn parse_lowercase_hexadecimal(text: &[u8]) -> Option<Vec<u8>> {
    let (pairs, remainder) = text.as_chunks::<2>();
    if !remainder.is_empty() || !text.iter().all(is_lowercase_hexadecimal_digit) {
        return None;
    }
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

fn is_lowercase_hexadecimal_digit(digit: &u8) -> bool {
    matches!(*digit, b'0'..=b'9' | b'a'..=b'f')
}

fn number_field(number: u64) -> String {
    let field = format!("{number:016x}");
    debug_assert_eq!(field.len(), NUMBER_FIELD_HEXADECIMAL_DIGITS);
    field
}

fn parse_number_field(text: &[u8]) -> Option<u64> {
    if text.len() != NUMBER_FIELD_HEXADECIMAL_DIGITS
        || !text.iter().all(is_lowercase_hexadecimal_digit)
    {
        return None;
    }
    u64::from_str_radix(std::str::from_utf8(text).ok()?, 16).ok()
}

/// `<族前缀><流键的十六进制>/<块号>`。
fn block_key(family_prefix: &[u8], stream_key: &[u8], block_number: u64) -> Vec<u8> {
    let mut key = family_prefix.to_vec();
    key.extend_from_slice(lowercase_hexadecimal(stream_key).as_bytes());
    key.push(KEY_FIELD_SEPARATOR);
    key.extend_from_slice(number_field(block_number).as_bytes());
    key
}

/// 解 `<流键的十六进制>/<块号>`。
fn parse_block_fields(fields: &[u8]) -> Option<GpuBlockAddress> {
    let mut parts = fields.split(|byte| *byte == KEY_FIELD_SEPARATOR);
    let stream_key = parse_lowercase_hexadecimal(parts.next()?)?;
    let block_number = parse_number_field(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some(GpuBlockAddress {
        stream_key,
        block_number,
    })
}

/// 待判行的键 `gpu-pending/<流键>/<块号>`。
#[must_use]
pub fn gpu_pending_key(stream_key: &[u8], block_number: u64) -> Vec<u8> {
    block_key(GPU_PENDING_FAMILY_PREFIX, stream_key, block_number)
}

/// 一条流的待判行都在这个前缀下（按块号升序）。
#[must_use]
pub fn gpu_pending_prefix_of_stream(stream_key: &[u8]) -> Vec<u8> {
    let mut prefix = GPU_PENDING_FAMILY_PREFIX.to_vec();
    prefix.extend_from_slice(lowercase_hexadecimal(stream_key).as_bytes());
    prefix.push(KEY_FIELD_SEPARATOR);
    prefix
}

/// 认领键 `gpu-claim/<流键>/<块号>`。
#[must_use]
pub fn gpu_claim_key(stream_key: &[u8], block_number: u64) -> Vec<u8> {
    block_key(GPU_CLAIM_FAMILY_PREFIX, stream_key, block_number)
}

/// 强制现判行的键 `gpu-rejudged/<入池序号>/<流键>/<块号>`。
#[must_use]
pub fn gpu_rejudged_key(pool_sequence: u64, stream_key: &[u8], block_number: u64) -> Vec<u8> {
    let mut prefix = gpu_rejudged_prefix_of_pool_sequence(pool_sequence);
    prefix.extend_from_slice(&block_key(b"", stream_key, block_number));
    prefix
}

/// 一个入池序号的强制现判行都在这个前缀下。
#[must_use]
pub fn gpu_rejudged_prefix_of_pool_sequence(pool_sequence: u64) -> Vec<u8> {
    let mut prefix = GPU_REJUDGED_FAMILY_PREFIX.to_vec();
    prefix.extend_from_slice(number_field(pool_sequence).as_bytes());
    prefix.push(KEY_FIELD_SEPARATOR);
    prefix
}

/// 待判行的键解出块；不是这一族的形状回 `None`。
#[must_use]
pub fn parse_gpu_pending_key(key: &[u8]) -> Option<GpuBlockAddress> {
    parse_block_fields(key.strip_prefix(GPU_PENDING_FAMILY_PREFIX)?)
}

/// 认领键解出块；不是这一族的形状回 `None`。
#[must_use]
pub fn parse_gpu_claim_key(key: &[u8]) -> Option<GpuBlockAddress> {
    parse_block_fields(key.strip_prefix(GPU_CLAIM_FAMILY_PREFIX)?)
}

/// 强制现判行的键解出（入池序号，块）；不是这一族的形状回 `None`。
#[must_use]
pub fn parse_gpu_rejudged_key(key: &[u8]) -> Option<(u64, GpuBlockAddress)> {
    let fields = key.strip_prefix(GPU_REJUDGED_FAMILY_PREFIX)?;
    let separator = fields
        .iter()
        .position(|byte| *byte == KEY_FIELD_SEPARATOR)?;
    let pool_sequence = parse_number_field(&fields[..separator])?;
    let block = parse_block_fields(&fields[separator + 1..])?;
    Some((pool_sequence, block))
}

/// 一块的待判行（文件头「待判行的值」）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GpuPendingRow {
    /// 录入表的键：服务照它读回录入、重算块键核对。
    pub recording_table_key: Vec<u8>,
    pub judge_table_digest: Vec<u8>,
    /// 块的状态序号区间的起。
    pub start_state_sequence: u64,
    /// 块的状态序号区间的止（不含）。
    pub end_state_sequence: u64,
    pub expected_gpu_judge_group_key: Vec<u8>,
    pub expected_conclusion_version_key: Vec<u8>,
    /// 强制现判这一块（哨兵与强制现判那几块）：服务不读库里已有的结论，写结论那一帧另写强制现判行。
    pub is_forced: bool,
    pub pool_sequence: u64,
}

fn put_length_prefixed(value: &mut Vec<u8>, field: &[u8]) {
    let length = u32::try_from(field.len()).expect("待判行的一段远短于 4 GiB（键与摘要）");
    value.extend_from_slice(&length.to_le_bytes());
    value.extend_from_slice(field);
}

impl GpuPendingRow {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut value = Vec::new();
        put_length_prefixed(&mut value, &self.recording_table_key);
        put_length_prefixed(&mut value, &self.judge_table_digest);
        value.extend_from_slice(&self.start_state_sequence.to_le_bytes());
        value.extend_from_slice(&self.end_state_sequence.to_le_bytes());
        put_length_prefixed(&mut value, &self.expected_gpu_judge_group_key);
        put_length_prefixed(&mut value, &self.expected_conclusion_version_key);
        value.push(u8::from(self.is_forced));
        value.extend_from_slice(&self.pool_sequence.to_le_bytes());
        value
    }

    /// 解一条待判行的值。
    ///
    /// # Errors
    /// 字节不够、强制现判那一位不是 0 或 1、解完还剩字节：回说明。
    pub fn decode(value: &[u8]) -> Result<GpuPendingRow, String> {
        GpuPendingRow::decode_fields(&mut ByteCursor::new(value))
            .map_err(|problem| format!("待判行的值解不开：{problem}"))
    }

    fn decode_fields(cursor: &mut ByteCursor<'_>) -> Result<GpuPendingRow, String> {
        let recording_table_key = cursor.length_prefixed("录入表的键")?.to_vec();
        let judge_table_digest = cursor.length_prefixed("判表摘要")?.to_vec();
        let start_state_sequence = cursor.u64("状态序号区间的起")?;
        let end_state_sequence = cursor.u64("状态序号区间的止")?;
        let expected_gpu_judge_group_key = cursor.length_prefixed("期待的 GPU 判器组键")?.to_vec();
        let expected_conclusion_version_key = cursor.length_prefixed("期待的结论版本键")?.to_vec();
        let is_forced = match cursor.u8("强制现判")? {
            0 => false,
            1 => true,
            other => return Err(format!("强制现判只能是 0 或 1，收到 {other}")),
        };
        let pool_sequence = cursor.u64("入池序号")?;
        if cursor.remaining() != 0 {
            return Err(format!("解完之后还剩 {} 字节", cursor.remaining()));
        }
        Ok(GpuPendingRow {
            recording_table_key,
            judge_table_digest,
            start_state_sequence,
            end_state_sequence,
            expected_gpu_judge_group_key,
            expected_conclusion_version_key,
            is_forced,
            pool_sequence,
        })
    }
}

/// 强制现判行的值（文件头「写结论」）：判它的持有者、结论摘要。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GpuRejudgedRow {
    pub holder_identity: Vec<u8>,
    /// 结论块那一行的值的 SHA-256。
    pub conclusion_digest: [u8; GUARD_ROW_DIGEST_BYTES],
}

impl GpuRejudgedRow {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let length = u8::try_from(self.holder_identity.len())
            .expect("持有者身份至多 255 字节（GpuClaimHolder::identity 远短于它）");
        let mut value = Vec::with_capacity(1 + self.holder_identity.len() + GUARD_ROW_DIGEST_BYTES);
        value.push(length);
        value.extend_from_slice(&self.holder_identity);
        value.extend_from_slice(&self.conclusion_digest);
        value
    }

    /// 解一条强制现判行的值。
    ///
    /// # Errors
    /// 字节不够、持有者身份是空的、解完还剩字节：回说明。
    pub fn decode(value: &[u8]) -> Result<GpuRejudgedRow, String> {
        GpuRejudgedRow::decode_fields(&mut ByteCursor::new(value))
            .map_err(|problem| format!("强制现判行的值解不开：{problem}"))
    }

    fn decode_fields(cursor: &mut ByteCursor<'_>) -> Result<GpuRejudgedRow, String> {
        let length = cursor.u8("持有者身份长")?;
        if length == 0 {
            return Err("持有者身份长是 0".to_string());
        }
        let holder_identity = cursor.take(usize::from(length), "持有者身份")?.to_vec();
        let conclusion_digest: [u8; GUARD_ROW_DIGEST_BYTES] = cursor
            .take(GUARD_ROW_DIGEST_BYTES, "结论摘要")?
            .try_into()
            .expect("take 回的正好是摘要那么多字节");
        if cursor.remaining() != 0 {
            return Err(format!("解完之后还剩 {} 字节", cursor.remaining()));
        }
        Ok(GpuRejudgedRow {
            holder_identity,
            conclusion_digest,
        })
    }
}

/// 认领的持有者（文件头「认领」）：服务进程起来时取的随机号、卡序号、判线程号。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct GpuClaimHolder {
    pub service_start_random: u64,
    pub card_index: u32,
    pub judging_thread_index: u32,
}

impl GpuClaimHolder {
    /// 认领帧与写结论帧里的持有者身份（远短于 255 字节）。
    #[must_use]
    pub fn identity(&self) -> Vec<u8> {
        let identity = format!(
            "{HOLDER_IDENTITY_PREFIX}{}/card-{}/thread-{}",
            number_field(self.service_start_random),
            self.card_index,
            self.judging_thread_index
        )
        .into_bytes();
        debug_assert!(identity.len() <= MAXIMUM_CLIENT_IDENTITY_BYTES);
        identity
    }
}

/// 服务进程起来时取一次的随机号（文件头「认领」），从系统随机源读 8 字节。
///
/// # Errors
/// 系统随机源打不开、读不满：回说明（服务照它退出，不拿可重复的数顶替）。
pub fn draw_service_start_random() -> Result<u64, String> {
    let mut bytes = [0_u8; 8];
    File::open(SYSTEM_RANDOM_SOURCE)
        .and_then(|mut source| source.read_exact(&mut bytes))
        .map_err(|error| format!("从 {SYSTEM_RANDOM_SOURCE} 取服务进程的随机号失败：{error}"))?;
    Ok(u64::from_le_bytes(bytes))
}

/// 一块 GPU 判出的结论要写的东西（文件头「写结论」）；`guarded_write` 拼成写结论帧。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GpuConclusion {
    pub block: GpuBlockAddress,
    pub holder: GpuClaimHolder,
    /// 结论块那一行的键（守卫键）。
    pub conclusion_block_key: Vec<u8>,
    /// 写的一方读到的结论块那一行（比较对象，`protocol.rs` 文件头「写结论」核守卫那一行）。
    pub guard_expectation: GuardRowState,
    /// 先于结论块落的旁表行（违例、计数向量、块属性、恢复字这类），照这个次序排在前面。
    pub side_entries: Vec<TableEntry>,
    /// 结论块那一行这一次写的值；`None` 是库里那一份与判出的相同、不重写（只删待判行、释放认领，强制现判时另写强制现判行）。
    pub conclusion_block_value: Option<Vec<u8>>,
    /// 强制现判的块给入池序号（另写强制现判行）；不是的给 `None`。
    pub forced_pool_sequence: Option<u64>,
    /// 写的一方读到的这一块的待判行（删键的期待，`protocol.rs` 文件头「写结论」第 4 步）：此刻库里那一行与它不同就整帧拒、不删。
    pub pending_row_as_read: GuardRowState,
}

impl GpuConclusion {
    /// 拼成写结论帧要带的东西（文件头「写结论」）。
    ///
    /// # Errors
    /// 强制现判而结论块那一行这一次不写、读到的那一份又是「不在」：结论摘要无从取，回说明。
    pub fn guarded_write(&self) -> Result<GuardedConclusionWrite, String> {
        let holder_identity = self.holder.identity();
        let mut entries = self.side_entries.clone();
        if let Some(pool_sequence) = self.forced_pool_sequence {
            let conclusion_digest = match (&self.conclusion_block_value, self.guard_expectation) {
                (Some(value), GuardRowState::Absent | GuardRowState::Present { .. }) => {
                    sha256_digest(value)
                }
                (None, GuardRowState::Present { value_digest }) => value_digest,
                (None, GuardRowState::Absent) => {
                    return Err(format!(
                        "流键 {} 块 {}：强制现判而结论块那一行这一次不写、读到的那一份又是「不在」，结论摘要无从取",
                        lowercase_hexadecimal(&self.block.stream_key),
                        self.block.block_number
                    ));
                }
            };
            entries.push((
                Table::Conclusions,
                gpu_rejudged_key(
                    pool_sequence,
                    &self.block.stream_key,
                    self.block.block_number,
                ),
                GpuRejudgedRow {
                    holder_identity: holder_identity.clone(),
                    conclusion_digest,
                }
                .encode(),
            ));
        }
        if let Some(value) = &self.conclusion_block_value {
            entries.push((
                Table::Conclusions,
                self.conclusion_block_key.clone(),
                value.clone(),
            ));
        }
        Ok(GuardedConclusionWrite {
            holder_identity,
            claim_key: gpu_claim_key(&self.block.stream_key, self.block.block_number),
            guard_key: self.conclusion_block_key.clone(),
            guard_expectation: self.guard_expectation,
            entries,
            deleted_keys: vec![GuardedDeletion {
                key: gpu_pending_key(&self.block.stream_key, self.block.block_number),
                expectation: self.pending_row_as_read,
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 键、持有者身份与待判行的值钉成字面字节：Python 客户端的自证（gpu-pending-rows-written-only-if-absent）核同一组字面值，两边逐字节相同。
    #[test]
    fn gpu_keys_holder_identity_and_pending_row_match_the_bytes_pinned_for_the_python_client() {
        assert_eq!(
            gpu_pending_key(b"\x00/", 42),
            b"gpu-pending/002f/000000000000002a".to_vec()
        );
        let holder = GpuClaimHolder {
            service_start_random: 0x1234,
            card_index: 1,
            judging_thread_index: 2,
        };
        assert_eq!(
            holder.identity(),
            b"gpu-service/0000000000001234/card-1/thread-2".to_vec()
        );
        let row = GpuPendingRow {
            recording_table_key: b"\x04r".to_vec(),
            judge_table_digest: vec![0xAB, 0xCD],
            start_state_sequence: 1,
            end_state_sequence: 2,
            expected_gpu_judge_group_key: b"g".to_vec(),
            expected_conclusion_version_key: b"v".to_vec(),
            is_forced: true,
            pool_sequence: 3,
        };
        let pinned = parse_lowercase_hexadecimal(
            b"02000000047202000000abcd0100000000000000020000000000000001000000670100000076010300000000000000",
        );
        assert_eq!(Some(row.encode()), pinned);
    }

    /// 三族的键解回原来的块与入池序号；流键带 `/` 与 0 字节也解得回；块号按十六进制补零排序。
    #[test]
    fn gpu_family_keys_parse_back_to_the_block_they_were_built_from() {
        let stream_key = b"\x00stream/with/separators\xff".to_vec();
        let pending = gpu_pending_key(&stream_key, 42);
        let claim = gpu_claim_key(&stream_key, 42);
        let rejudged = gpu_rejudged_key(7, &stream_key, 42);
        let block = GpuBlockAddress {
            stream_key: stream_key.clone(),
            block_number: 42,
        };
        assert_eq!(parse_gpu_pending_key(&pending), Some(block.clone()));
        assert_eq!(parse_gpu_claim_key(&claim), Some(block.clone()));
        assert_eq!(parse_gpu_rejudged_key(&rejudged), Some((7, block)));
        assert!(pending.starts_with(&gpu_pending_prefix_of_stream(&stream_key)));
        assert!(rejudged.starts_with(&gpu_rejudged_prefix_of_pool_sequence(7)));
        assert!(gpu_pending_key(&stream_key, 9) < gpu_pending_key(&stream_key, 10));
        assert_eq!(parse_gpu_pending_key(&claim), None, "别的族的键不认");
    }

    /// 待判行与强制现判行的值解回编进去的；多一个字节、强制现判那一位是 2 都拒。
    #[test]
    fn gpu_rows_decode_to_what_was_encoded_and_refuse_trailing_bytes() {
        let row = GpuPendingRow {
            recording_table_key: b"\x04recording".to_vec(),
            judge_table_digest: vec![0xAB; 32],
            start_state_sequence: 65_536,
            end_state_sequence: 131_072,
            expected_gpu_judge_group_key: b"judge-group".to_vec(),
            expected_conclusion_version_key: b"conclusion-version".to_vec(),
            is_forced: true,
            pool_sequence: 3,
        };
        let encoded = row.encode();
        assert_eq!(GpuPendingRow::decode(&encoded), Ok(row));
        let mut trailing = encoded.clone();
        trailing.push(0);
        assert!(GpuPendingRow::decode(&trailing).is_err());
        let mut forced_flag_two = encoded;
        let flag_position = forced_flag_two.len() - 9;
        forced_flag_two[flag_position] = 2;
        assert!(GpuPendingRow::decode(&forced_flag_two).is_err());
        let rejudged = GpuRejudgedRow {
            holder_identity: b"gpu-service/0000000000000001/card-0/thread-0".to_vec(),
            conclusion_digest: sha256_digest(b"conclusion"),
        };
        assert_eq!(GpuRejudgedRow::decode(&rejudged.encode()), Ok(rejudged));
    }

    /// 强制现判的结论：强制现判行排在结论块那一行之前、结论块排最后；没写结论块时摘要取期待；读到「不在」又不写拒。
    #[test]
    fn forced_conclusion_puts_the_rejudged_row_before_the_conclusion_block() {
        let holder = GpuClaimHolder {
            service_start_random: 0x1234,
            card_index: 1,
            judging_thread_index: 2,
        };
        let mut conclusion = GpuConclusion {
            block: GpuBlockAddress {
                stream_key: b"stream".to_vec(),
                block_number: 5,
            },
            holder,
            conclusion_block_key: b"\x10conclusion".to_vec(),
            guard_expectation: GuardRowState::Absent,
            side_entries: vec![(Table::Conclusions, b"\x11side".to_vec(), b"side".to_vec())],
            conclusion_block_value: Some(b"verdicts".to_vec()),
            forced_pool_sequence: Some(9),
            pending_row_as_read: GuardRowState::of_stored_value(Some(b"pending row")),
        };
        let written = conclusion.guarded_write().expect("拼得成");
        let keys: Vec<&[u8]> = written
            .entries
            .iter()
            .map(|(_, key, _)| key.as_slice())
            .collect();
        let rejudged_key = gpu_rejudged_key(9, b"stream", 5);
        assert_eq!(
            keys,
            vec![
                b"\x11side".as_slice(),
                rejudged_key.as_slice(),
                b"\x10conclusion".as_slice()
            ]
        );
        assert_eq!(
            written.deleted_keys,
            vec![GuardedDeletion {
                key: gpu_pending_key(b"stream", 5),
                expectation: GuardRowState::of_stored_value(Some(b"pending row")),
            }]
        );
        assert_eq!(written.claim_key, gpu_claim_key(b"stream", 5));
        let rejudged = GpuRejudgedRow::decode(&written.entries[1].2).expect("解得开");
        assert_eq!(rejudged.conclusion_digest, sha256_digest(b"verdicts"));
        assert_eq!(rejudged.holder_identity, holder.identity());
        conclusion.conclusion_block_value = None;
        conclusion.guard_expectation = GuardRowState::of_stored_value(Some(b"stored"));
        let unchanged = conclusion.guarded_write().expect("拼得成");
        let unchanged_row = GpuRejudgedRow::decode(&unchanged.entries[1].2).expect("解得开");
        assert_eq!(unchanged_row.conclusion_digest, sha256_digest(b"stored"));
        conclusion.guard_expectation = GuardRowState::Absent;
        assert!(conclusion.guarded_write().is_err());
    }
}
