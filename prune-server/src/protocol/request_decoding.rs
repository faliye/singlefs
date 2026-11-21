//! 请求帧的解码：字节游标、请求视图、带守卫的写结论帧的合法性、帧体与帧里的载荷解成请求，拒绝时给出为什么。

use super::*;

/// 按位置读帧体的游标；越界是「内容解不开」，不是 panic。GPU 判的几种行的值（`gpu_judging_rows`）照同一套小端写法，也拿它读。
pub(crate) struct ByteCursor<'frame> {
    pub(super) bytes: &'frame [u8],
    position: usize,
}

impl<'frame> ByteCursor<'frame> {
    pub(crate) fn new(bytes: &'frame [u8]) -> ByteCursor<'frame> {
        ByteCursor { bytes, position: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    pub(crate) fn take(&mut self, byte_count: usize, what: &str) -> Result<&'frame [u8], String> {
        if byte_count > self.remaining() {
            return Err(format!(
                "读{what}要 {byte_count} 字节，只剩 {} 字节",
                self.remaining()
            ));
        }
        let field = &self.bytes[self.position..self.position + byte_count];
        self.position += byte_count;
        Ok(field)
    }

    pub(crate) fn u8(&mut self, what: &str) -> Result<u8, String> {
        Ok(self.take(1, what)?[0])
    }

    pub(crate) fn u32(&mut self, what: &str) -> Result<u32, String> {
        let field = self.take(4, what)?;
        Ok(u32::from_le_bytes(
            field.try_into().expect("take 回的正好是 4 字节"),
        ))
    }

    pub(crate) fn u64(&mut self, what: &str) -> Result<u64, String> {
        let field = self.take(8, what)?;
        Ok(u64::from_le_bytes(
            field.try_into().expect("take 回的正好是 8 字节"),
        ))
    }

    pub(crate) fn length_prefixed(&mut self, what: &str) -> Result<&'frame [u8], String> {
        let length = self.u32(what)?;
        self.take(usize_from_u32(length), what)
    }

    fn table(&mut self) -> Result<Table, String> {
        let code = self.u8("表号")?;
        Table::from_code(code).ok_or_else(|| format!("认不得的表号 {code}"))
    }

    pub(super) fn client_identity(&mut self) -> Result<&'frame [u8], String> {
        let length = self.u8("客户端身份长")?;
        if length == 0 {
            return Err("客户端身份长是 0".to_string());
        }
        self.take(usize::from(length), "客户端身份")
    }

    /// 写结论帧里的持有者身份（`u8 长`、身份，1 到 255 字节）。
    pub(super) fn holder_identity(&mut self) -> Result<&'frame [u8], String> {
        let length = self.u8("持有者身份长")?;
        if length == 0 {
            return Err("持有者身份长是 0".to_string());
        }
        self.take(usize::from(length), "持有者身份")
    }

    /// 守卫那一行的样子（`u8` 0 不在；1 在，再跟 32 字节值的 SHA-256）。
    pub(super) fn guard_row_state(&mut self, what: &str) -> Result<GuardRowState, String> {
        match self.u8(what)? {
            GUARD_ROW_ABSENT_CODE => Ok(GuardRowState::Absent),
            GUARD_ROW_PRESENT_CODE => {
                let digest = self.take(GUARD_ROW_DIGEST_BYTES, what)?;
                Ok(GuardRowState::Present {
                    value_digest: digest.try_into().expect("take 回的正好是摘要那么多字节"),
                })
            }
            other => Err(format!(
                "{what}只能是 {GUARD_ROW_ABSENT_CODE}（不在）或 {GUARD_ROW_PRESENT_CODE}（在），收到 {other}"
            )),
        }
    }

    /// 成批写、不在才写、写结论的条目：`u32 条数`，每条 `u8 表`、`u32 键长`、键、`u32 值长`、值。
    pub(super) fn entries(&mut self) -> Result<Vec<EntryInTable<'frame>>, String> {
        let entry_count = self.u32("条数")?;
        let mut entries = Vec::with_capacity(self.capacity_for(entry_count, 9));
        for _ in 0..entry_count {
            let table = self.table()?;
            let key = self.length_prefixed("键")?;
            let value = self.length_prefixed("值")?;
            entries.push(EntryInTable { table, key, value });
        }
        Ok(entries)
    }

    fn short_text(&mut self, what: &str) -> Result<String, String> {
        let length = self.u8(what)?;
        let field = self.take(usize::from(length), what)?;
        std::str::from_utf8(field)
            .map(str::to_string)
            .map_err(|_| format!("{what}不是 UTF-8"))
    }

    pub(super) fn announcement(&mut self) -> Result<ClientAnnouncement, String> {
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
    pub(super) fn capacity_for(&self, declared_count: u32, minimum_bytes_per_item: usize) -> usize {
        usize_from_u32(declared_count).min(self.remaining() / minimum_bytes_per_item)
    }
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
    WriteIfAbsent {
        client_identity: &'frame [u8],
        request_number: u64,
        entries: Vec<EntryInTable<'frame>>,
    },
    GuardedConclusionWrite(GuardedConclusionWriteView<'frame>),
    /// 放认领（文件头「放认领」）：键都已核过是 GPU 认领键。
    ReleaseClaims {
        holder_identity: &'frame [u8],
        keys: Vec<KeyInTable<'frame>>,
    },
    /// 列认领（文件头「列认领」）：解帧只核条数上限与续扫起点，前缀是不是 GPU 认领键的族前缀由提交线程核（弄坏开关 claim-listing-prefix-unchecked 在那里放过）。
    ListClaims {
        prefix: &'frame [u8],
        after_key: &'frame [u8],
        maximum_entries: u32,
    },
}

/// 解开的写结论帧（文件头「写结论」），键与值借用帧体；解帧时已核过那几条写法（`guarded_conclusion_write_problem`）。
#[derive(Debug, PartialEq, Eq)]
pub struct GuardedConclusionWriteView<'frame> {
    pub client_identity: &'frame [u8],
    pub request_number: u64,
    pub holder_identity: &'frame [u8],
    pub claim_key: &'frame [u8],
    pub guard_key: &'frame [u8],
    pub guard_expectation: GuardRowState,
    pub entries: Vec<EntryInTable<'frame>>,
    pub deleted_keys: Vec<GuardedDeletionView<'frame>>,
}

/// 解开的一把删键与帧里给它的期待（[`GuardedDeletion`] 借用帧体的那一份）。
#[derive(Debug, PartialEq, Eq)]
pub struct GuardedDeletionView<'frame> {
    pub key: &'frame [u8],
    pub expectation: GuardRowState,
}

/// 写结论帧违反文件头「写结论」那几条写法时回说明（服务回状态 1），合规回 `None`。
fn guarded_conclusion_write_problem(view: &GuardedConclusionWriteView<'_>) -> Option<String> {
    if view.claim_key.is_empty() {
        return Some("认领键是空的".to_string());
    }
    if view.guard_key.is_empty() {
        return Some("守卫键是空的".to_string());
    }
    let guard_positions: Vec<usize> = view
        .entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.key == view.guard_key)
        .map(|(position, _)| position)
        .collect();
    match guard_positions.as_slice() {
        [] => {}
        [position] if position + 1 == view.entries.len() => {}
        [_] | [_, _, ..] => {
            return Some(format!(
                "守卫键在条目里的位置（从 0 数）是 {guard_positions:?}，共 {} 条：守卫那一行只许写一次、且排最后一条（一份结论有多行时守卫放在最后落的那一行）",
                view.entries.len()
            ));
        }
    }
    let claim_key_written = view.entries.iter().any(|entry| entry.key == view.claim_key);
    let claim_key_deleted = view
        .deleted_keys
        .iter()
        .any(|deletion| deletion.key == view.claim_key);
    if claim_key_written || claim_key_deleted {
        return Some(
            "认领键在条目或删键里：认领键只记认领，不当数据行写、也不删（写了它，之后认领它就回「已有结论」）"
                .to_string(),
        );
    }
    for GuardedDeletionView { key, .. } in &view.deleted_keys {
        if key.is_empty() {
            return Some("删键里有空键".to_string());
        }
        if *key == view.guard_key || view.entries.iter().any(|entry| entry.key == *key) {
            return Some(format!(
                "删键 {:02x?} 是守卫键或某条条目的键：同一帧里不又写又删同一把键",
                &key[..key.len().min(16)]
            ));
        }
    }
    None
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
        RequestView::BatchedPointRead(keys)
        | RequestView::Claim { keys, .. }
        | RequestView::ReleaseClaims { keys, .. } => keys
            .iter()
            .find(|key| is_service_key(key.key))
            .and_then(|key| refused("键", key.key)),
        RequestView::BatchedWrite { entries, .. } | RequestView::WriteIfAbsent { entries, .. } => {
            entries
                .iter()
                .find(|entry| is_service_key(entry.key))
                .and_then(|entry| refused("键", entry.key))
        }
        RequestView::GuardedConclusionWrite(conclusion) => conclusion
            .entries
            .iter()
            .map(|entry| entry.key)
            .chain(conclusion.deleted_keys.iter().map(|deletion| deletion.key))
            .chain([conclusion.claim_key, conclusion.guard_key])
            .find(|key| is_service_key(key))
            .and_then(|key| refused("键", key)),
        RequestView::PrefixScan { prefix, .. }
        | RequestView::DeletePrefix { prefix, .. }
        | RequestView::ListClaims { prefix, .. }
            if is_service_key(prefix) =>
        {
            refused("前缀", prefix)
        }
        RequestView::PrefixScan { .. }
        | RequestView::DeletePrefix { .. }
        | RequestView::ListClaims { .. }
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
            let entries = cursor.entries()?;
            Ok(RequestView::BatchedWrite {
                client_identity,
                request_number,
                entries,
            })
        }
        OperationCode::WriteIfAbsent => {
            let client_identity = cursor.client_identity()?;
            let request_number = cursor.u64("请求号")?;
            let entries = cursor.entries()?;
            Ok(RequestView::WriteIfAbsent {
                client_identity,
                request_number,
                entries,
            })
        }
        OperationCode::GuardedConclusionWrite => {
            let client_identity = cursor.client_identity()?;
            let request_number = cursor.u64("请求号")?;
            let holder_identity = cursor.holder_identity()?;
            let claim_key = cursor.length_prefixed("认领键")?;
            let guard_key = cursor.length_prefixed("守卫键")?;
            let guard_expectation = cursor.guard_row_state("守卫那一行的期待")?;
            let entries = cursor.entries()?;
            let deleted_key_count = cursor.u32("删键数")?;
            let mut deleted_keys = Vec::with_capacity(cursor.capacity_for(deleted_key_count, 5));
            for _ in 0..deleted_key_count {
                let key = cursor.length_prefixed("删键")?;
                let expectation = cursor.guard_row_state("删键那一行的期待")?;
                deleted_keys.push(GuardedDeletionView { key, expectation });
            }
            let conclusion = GuardedConclusionWriteView {
                client_identity,
                request_number,
                holder_identity,
                claim_key,
                guard_key,
                guard_expectation,
                entries,
                deleted_keys,
            };
            match guarded_conclusion_write_problem(&conclusion) {
                Some(problem) => Err(problem),
                None => Ok(RequestView::GuardedConclusionWrite(conclusion)),
            }
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
        OperationCode::ReleaseClaims => {
            let holder_identity = cursor.holder_identity()?;
            let key_count = cursor.u32("键数")?;
            let mut keys = Vec::with_capacity(cursor.capacity_for(key_count, 5));
            for _ in 0..key_count {
                let table = cursor.table()?;
                let key = cursor.length_prefixed("键")?;
                if !key.starts_with(GPU_CLAIM_FAMILY_PREFIX) {
                    return Err(format!(
                        "放认领只收 GPU 认领键（以 {} 起头），收到 {:02x?}",
                        String::from_utf8_lossy(GPU_CLAIM_FAMILY_PREFIX),
                        &key[..key.len().min(16)]
                    ));
                }
                keys.push(KeyInTable { table, key });
            }
            Ok(RequestView::ReleaseClaims {
                holder_identity,
                keys,
            })
        }
        OperationCode::ListClaims => {
            let prefix = cursor.length_prefixed("前缀")?;
            let after_key = cursor.length_prefixed("续扫起点")?;
            let maximum_entries = cursor.u32("条数上限")?;
            if maximum_entries == 0 {
                return Err("列认领的条数上限至少 1".to_string());
            }
            if !after_key.is_empty() && !after_key.starts_with(prefix) {
                return Err("续扫起点不以前缀起头：续扫起点是上一帧回的最后一把键".to_string());
            }
            Ok(RequestView::ListClaims {
                prefix,
                after_key,
                maximum_entries,
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
