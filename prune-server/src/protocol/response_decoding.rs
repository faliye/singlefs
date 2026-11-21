//! 回应帧的解码：回应体解成完成的结局、被拒、扫到的条目与统计。

use super::*;

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
    /// 放认领的回应：逐键一个结局，次序与请求里的键相同。
    ClaimsReleased(Vec<ClaimReleaseOutcome>),
    /// 列认领的回应：按键升序的认领，被条数上限或帧长上限截断时 `is_truncated` 为真（文件头「列认领」）。
    ClaimListing {
        is_truncated: bool,
        claims: Vec<ListedClaim>,
    },
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
    /// 写结论守卫不过（状态 7）：这一帧一条都没写。
    GuardRefused {
        refusal: GuardRefusal,
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
        ResponseStatus::GuardRefused => {
            let refusal = match cursor.u8("守卫不过的种类")? {
                GUARD_REFUSAL_CLAIM_HELD_BY_OTHER_CODE => {
                    let holder_identity = cursor.client_identity()?.to_vec();
                    let remaining_lease_milliseconds = cursor.u64("剩余租期毫秒")?;
                    GuardRefusal::ClaimHeldByOther {
                        holder_identity,
                        remaining_lease_milliseconds,
                    }
                }
                GUARD_REFUSAL_GUARD_ROW_CHANGED_CODE => GuardRefusal::GuardRowChanged {
                    current: cursor.guard_row_state("守卫那一行此刻的样子")?,
                },
                GUARD_REFUSAL_DELETED_ROW_CHANGED_CODE => GuardRefusal::DeletedRowChanged {
                    key: cursor.length_prefixed("变了的删键")?.to_vec(),
                    current: cursor.guard_row_state("删键那一行此刻的样子")?,
                },
                other => {
                    return Err(format!(
                        "守卫不过的种类只能是 {GUARD_REFUSAL_CLAIM_HELD_BY_OTHER_CODE}、{GUARD_REFUSAL_GUARD_ROW_CHANGED_CODE} 或 {GUARD_REFUSAL_DELETED_ROW_CHANGED_CODE}，收到 {other}"
                    ))
                }
            };
            let message =
                String::from_utf8_lossy(cursor.length_prefixed("守卫不过的说明")?).into_owned();
            ResponseOutcome::GuardRefused { refusal, message }
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
        OperationCode::BatchedWrite
        | OperationCode::WriteIfAbsent
        | OperationCode::GuardedConclusionWrite => {
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
        OperationCode::ReleaseClaims => {
            let outcome_count = cursor.u32("放认领结局个数")?;
            let mut outcomes = Vec::with_capacity(cursor.capacity_for(outcome_count, 1));
            for _ in 0..outcome_count {
                let code = cursor.u8("放认领结局")?;
                let outcome = ClaimReleaseOutcome::from_code(code)
                    .ok_or_else(|| format!("放认领结局只能是 0、1、2，收到 {code}"))?;
                outcomes.push(outcome);
            }
            Ok(ResponseOutcome::ClaimsReleased(outcomes))
        }
        OperationCode::ListClaims => {
            let is_truncated = match cursor.u8("截断标记")? {
                0 => false,
                1 => true,
                other => return Err(format!("截断标记只能是 0 或 1，收到 {other}")),
            };
            let claim_count = cursor.u32("条数")?;
            let mut claims = Vec::with_capacity(cursor.capacity_for(claim_count, 14));
            for _ in 0..claim_count {
                let key = cursor.length_prefixed("键")?.to_vec();
                let holder_identity = cursor.client_identity()?.to_vec();
                let remaining_lease_milliseconds = cursor.u64("剩余租期毫秒")?;
                claims.push(ListedClaim {
                    key,
                    holder_identity,
                    remaining_lease_milliseconds,
                });
            }
            Ok(ResponseOutcome::ClaimListing {
                is_truncated,
                claims,
            })
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
