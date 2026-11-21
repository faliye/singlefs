//! 回应帧的编码：拒绝、忙、写确认、条目确认、守卫拒绝、认领结局、放认领结局、列认领与按前缀删的回应。

use super::*;

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
    assert!(
        status != ResponseStatus::GuardRefused,
        "守卫不过的回应带种类，走 encode_guard_refusal"
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
    encode_entries_acknowledgement(
        sequence,
        OperationCode::BatchedWrite,
        entries_written,
        is_already_applied,
    )
}

/// 成批写、不在才写、写结论的确认（三者回应相同，操作码照请求）：写了几条、这个请求号是不是早已生效。
pub fn encode_entries_acknowledgement(
    sequence: u64,
    operation: OperationCode,
    entries_written: u32,
    is_already_applied: bool,
) -> Vec<u8> {
    let mut encoder = begin_response(sequence, ResponseStatus::Completed, operation.code(), 5);
    encoder.put_u32(entries_written);
    encoder.put_u8(u8::from(is_already_applied));
    encoder.finish()
}

/// 写结论守卫不过（状态 7，文件头「回应帧」）：种类与它带的东西、说明。
pub fn encode_guard_refusal(sequence: u64, refusal: &GuardRefusal, message: &str) -> Vec<u8> {
    let key_bytes = match refusal {
        GuardRefusal::DeletedRowChanged { key, .. } => key.len(),
        GuardRefusal::ClaimHeldByOther { .. } | GuardRefusal::GuardRowChanged { .. } => 0,
    };
    let mut encoder = begin_response(
        sequence,
        ResponseStatus::GuardRefused,
        OperationCode::GuardedConclusionWrite.code(),
        message.len() + key_bytes + MAXIMUM_CLIENT_IDENTITY_BYTES + GUARD_ROW_DIGEST_BYTES + 16,
    );
    match refusal {
        GuardRefusal::ClaimHeldByOther {
            holder_identity,
            remaining_lease_milliseconds,
        } => {
            encoder.put_u8(GUARD_REFUSAL_CLAIM_HELD_BY_OTHER_CODE);
            put_client_identity(&mut encoder, holder_identity);
            encoder.put_u64(*remaining_lease_milliseconds);
        }
        GuardRefusal::GuardRowChanged { current } => {
            encoder.put_u8(GUARD_REFUSAL_GUARD_ROW_CHANGED_CODE);
            current.put(&mut encoder);
        }
        GuardRefusal::DeletedRowChanged { key, current } => {
            encoder.put_u8(GUARD_REFUSAL_DELETED_ROW_CHANGED_CODE);
            encoder.put_length_prefixed(key);
            current.put(&mut encoder);
        }
    }
    encoder.put_length_prefixed(message.as_bytes());
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

/// 放认领的结局，逐键一个（文件头「放认领」）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClaimReleaseOutcome {
    /// 这把键上记的持有者就是帧里的持有者：放掉了。
    Released,
    /// 这把键上没有认领记录。
    NotClaimed,
    /// 记的是别的持有者：没放。
    HeldByOther,
}

impl ClaimReleaseOutcome {
    pub const ALL: [ClaimReleaseOutcome; 3] = [
        ClaimReleaseOutcome::Released,
        ClaimReleaseOutcome::NotClaimed,
        ClaimReleaseOutcome::HeldByOther,
    ];

    pub fn code(self) -> u8 {
        match self {
            ClaimReleaseOutcome::Released => 0,
            ClaimReleaseOutcome::NotClaimed => 1,
            ClaimReleaseOutcome::HeldByOther => 2,
        }
    }

    pub fn from_code(code: u8) -> Option<ClaimReleaseOutcome> {
        ClaimReleaseOutcome::ALL
            .into_iter()
            .find(|outcome| outcome.code() == code)
    }
}

/// 放认领回应：`u32 个数`，每键一字节结局。
pub fn encode_claim_release_outcomes(sequence: u64, outcomes: &[ClaimReleaseOutcome]) -> Vec<u8> {
    let mut encoder = begin_response(
        sequence,
        ResponseStatus::Completed,
        OperationCode::ReleaseClaims.code(),
        CLAIM_OUTCOME_COUNT_BYTES + outcomes.len(),
    );
    encoder.put_u32(u32_from_usize(outcomes.len(), "放认领结局个数"));
    for outcome in outcomes {
        encoder.put_u8(outcome.code());
    }
    encoder.finish()
}

/// 列认领回应内容开头的字节：`u8` 截断标记、`u32` 条数。
pub const CLAIM_LISTING_HEADER_BYTES: usize = 5;

/// 列认领回的一条（文件头「列认领」）：认领键（客户端的键）、持有者身份、剩余租期。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ListedClaim {
    pub key: Vec<u8>,
    pub holder_identity: Vec<u8>,
    pub remaining_lease_milliseconds: u64,
}

/// 一条认领编进列认领回应占的字节：键长 4、键、身份长 1、持有者身份、剩余租期 8。
pub fn listed_claim_bytes(key_bytes: usize, holder_identity_bytes: usize) -> usize {
    4 + key_bytes + 1 + holder_identity_bytes + 8
}

/// 列认领回应：调用方先核过装得下一帧（`committer.rs` 的 `add_claim_listing`）。
pub fn encode_claim_listing(sequence: u64, is_truncated: bool, claims: &[ListedClaim]) -> Vec<u8> {
    let capacity = CLAIM_LISTING_HEADER_BYTES
        + claims
            .iter()
            .map(|claim| listed_claim_bytes(claim.key.len(), claim.holder_identity.len()))
            .sum::<usize>();
    let mut encoder = begin_response(
        sequence,
        ResponseStatus::Completed,
        OperationCode::ListClaims.code(),
        capacity,
    );
    encoder.put_u8(u8::from(is_truncated));
    encoder.put_u32(u32_from_usize(claims.len(), "列认领条数"));
    for claim in claims {
        encoder.put_length_prefixed(&claim.key);
        put_client_identity(&mut encoder, &claim.holder_identity);
        encoder.put_u64(claim.remaining_lease_milliseconds);
    }
    encoder.finish()
}
