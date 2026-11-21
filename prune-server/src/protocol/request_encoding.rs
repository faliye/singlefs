//! 请求帧的编码（文件头「帧」与各操作码的小节）：帧编码器、点读与写帧的构造器、带守卫的写结论帧、扫、统计、删前缀、认领、放认领、列认领各帧的编法，换认领租期与请求号。

use super::*;

/// 往一帧里追加字段，`finish` 时回填帧体长度。
pub struct FrameEncoder {
    pub(super) bytes: Vec<u8>,
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

pub(super) fn begin_request(
    sequence: u64,
    operation_code: u8,
    capacity_in_bytes: usize,
) -> FrameEncoder {
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
pub(super) fn put_client_identity(encoder: &mut FrameEncoder, client_identity: &[u8]) {
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
    begin_entries_frame(
        OperationCode::BatchedWrite,
        sequence,
        client_identity,
        request_number,
        entry_count,
        capacity_in_bytes,
    )
}

/// 不在才写帧（文件头「不在才写」）：内容与成批写帧相同，操作码 10。
pub fn begin_write_if_absent_frame(
    sequence: u64,
    client_identity: &[u8],
    request_number: u64,
    entry_count: u32,
    capacity_in_bytes: usize,
) -> WriteFrameBuilder {
    begin_entries_frame(
        OperationCode::WriteIfAbsent,
        sequence,
        client_identity,
        request_number,
        entry_count,
        capacity_in_bytes,
    )
}

/// 成批写与不在才写共用的帧头：身份、请求号、条数。
fn begin_entries_frame(
    operation: OperationCode,
    sequence: u64,
    client_identity: &[u8],
    request_number: u64,
    entry_count: u32,
    capacity_in_bytes: usize,
) -> WriteFrameBuilder {
    let mut encoder = begin_request(
        sequence,
        operation.code(),
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

/// 一帧写结论要带的东西（文件头「写结论」逐个字段）；客户端身份与请求号由发它的客户端填。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GuardedConclusionWrite {
    /// 认领的持有者身份（1 到 255 字节）：核认领时与认领键上记的持有者比。
    pub holder_identity: Vec<u8>,
    pub claim_key: Vec<u8>,
    pub guard_key: Vec<u8>,
    /// 写的一方读到的守卫那一行的样子。
    pub guard_expectation: GuardRowState,
    /// 守卫键在里面时排最后一条。
    pub entries: Vec<(Table, Vec<u8>, Vec<u8>)>,
    pub deleted_keys: Vec<GuardedDeletion>,
}

/// 写结论帧里的一把删键与写的一方读到的那一行（文件头「写结论」第 4 步）：库里那一行此刻的样子与 `expectation` 不同，整帧拒。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GuardedDeletion {
    pub key: Vec<u8>,
    pub expectation: GuardRowState,
}

/// 一把删键在写结论帧里另占的字节（键长 4、期待 1 加摘要），不含键本身。
const GUARDED_DELETION_FIXED_BYTES: usize = 4 + 1 + GUARD_ROW_DIGEST_BYTES;

/// 写结论帧在条目与删键之外另占的字节（身份长 1、请求号 8、持有者身份长 1、认领键长 4、守卫键长 4、期待 1 加摘要、条数 4、删键数 4）。
const GUARDED_CONCLUSION_FIXED_BYTES: usize =
    1 + 8 + 1 + 4 + 4 + 1 + GUARD_ROW_DIGEST_BYTES + 4 + 4;

/// 写结论帧（文件头「写结论」）。持有者身份长度不在 1 到 255 字节之内是调用方的 bug（容错客户端编帧之前先核，`reliable_client.rs`）。
pub fn encode_guarded_conclusion_write_frame(
    sequence: u64,
    client_identity: &[u8],
    request_number: u64,
    request: &GuardedConclusionWrite,
) -> Vec<u8> {
    let capacity = GUARDED_CONCLUSION_FIXED_BYTES
        + client_identity.len()
        + request.holder_identity.len()
        + request.claim_key.len()
        + request.guard_key.len()
        + request
            .entries
            .iter()
            .map(|(_, key, value)| 9 + key.len() + value.len())
            .sum::<usize>()
        + request
            .deleted_keys
            .iter()
            .map(|deletion| GUARDED_DELETION_FIXED_BYTES + deletion.key.len())
            .sum::<usize>();
    let mut encoder = begin_request(
        sequence,
        OperationCode::GuardedConclusionWrite.code(),
        capacity,
    );
    put_client_identity(&mut encoder, client_identity);
    encoder.put_u64(request_number);
    put_client_identity(&mut encoder, &request.holder_identity);
    encoder.put_length_prefixed(&request.claim_key);
    encoder.put_length_prefixed(&request.guard_key);
    request.guard_expectation.put(&mut encoder);
    encoder.put_u32(u32_from_usize(request.entries.len(), "写结论的条数"));
    for (table, key, value) in &request.entries {
        encoder.put_u8(table.code());
        encoder.put_length_prefixed(key);
        encoder.put_length_prefixed(value);
    }
    encoder.put_u32(u32_from_usize(request.deleted_keys.len(), "写结论的删键数"));
    for deletion in &request.deleted_keys {
        encoder.put_length_prefixed(&deletion.key);
        deletion.expectation.put(&mut encoder);
    }
    encoder.finish()
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

/// 放认领帧（文件头「放认领」）：以 `holder_identity` 放掉这几把 GPU 认领键上的认领。
///
/// # Panics
/// 持有者身份不在 1 到 255 字节之内（调用方先核）。
pub fn encode_release_claims_frame(
    sequence: u64,
    holder_identity: &[u8],
    keys: &[(Table, &[u8])],
) -> Vec<u8> {
    let capacity =
        holder_identity.len() + 5 + keys.iter().map(|(_, key)| 5 + key.len()).sum::<usize>();
    let mut encoder = begin_request(sequence, OperationCode::ReleaseClaims.code(), capacity);
    put_client_identity(&mut encoder, holder_identity);
    encoder.put_u32(u32_from_usize(keys.len(), "放认领的键数"));
    for (table, key) in keys {
        encoder.put_u8(table.code());
        encoder.put_length_prefixed(key);
    }
    encoder.finish()
}

/// 列认领帧（文件头「列认领」）：键以 `prefix` 起头、租期没到的认领，从排在 `after_key` 之后的第一把键起（`after_key` 空是从前缀起），至多 `maximum_entries` 条。
pub fn encode_list_claims_frame(
    sequence: u64,
    prefix: &[u8],
    after_key: &[u8],
    maximum_entries: u32,
) -> Vec<u8> {
    let mut encoder = begin_request(
        sequence,
        OperationCode::ListClaims.code(),
        prefix.len() + after_key.len() + 12,
    );
    encoder.put_length_prefixed(prefix);
    encoder.put_length_prefixed(after_key);
    encoder.put_u32(maximum_entries);
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
