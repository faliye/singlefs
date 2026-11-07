//! 提交线程：写与认领都交给它，按到达次序组提交。
//!
//! 收到一帧之后在一个短窗口里（`--group-commit-milliseconds`）再收，至多收到 `--group-commit-frames` 帧，
//! 几帧的写合进一个 WriteBatch、只同步一次 WAL，同步完再一起回各自的确认；确认的意思不变：回确认时已落盘。
//! 去重：写帧按（客户端身份，请求号）查库里「已生效的请求号」那几行与这一组里已收的，早已生效的不再写，照回写确认（带「早已生效」）。
//! 认领：认领记录在内存里有一份（服务起来时从库里读，之后只由这个线程改），判「别人持有」不读库；判「已有结论」读库，
//! 所以认领帧之前已收进这一组的写先提交（同步、回确认），认领读的是已落盘的库。写结论时同一批里删掉这些键上的认领。
//! 同一身份再认领自己持有、租期没到的键：到期时刻取已记下的与「现在加帧里的租期」里晚的那一个，只往后、不往前挪
//! （弄坏开关 claim-expiry-moved-earlier 退回照帧里的租期覆盖）。容错客户端每次发出去都给整段租期（`reliable_client.rs` 文件头「认领」），
//! 所以重发落下的、忙了很久才落下的认领，回到调用方手里时到期离落下那一刻都还有整段租期，调用方从回来那一刻起按整段租期写，盖得住；
//! 重发落下时往后推的那一段至多是这一次认领调用自己用掉的时间（重连总时长上限管着）。
//! 认领先逐键判结局、只数回应的字节，再动库与认领表：回应会越过帧长上限的整帧回状态 1（说明里写回应会有多少字节），
//! 回应记不进待处理额度的回忙，两种都一个键都不认领、不续租（弄坏开关 claim-response-unbudgeted 退回不核字节、照编超长回应的写法）。
//! 删前缀：同样先把这一组里已收的写提交掉（删的是已落盘的库），再扫；回应（带这一帧最后一把键）先编好、照认领的办法先记进待处理额度，
//! 记不进整帧回忙、一把键都不删（只数不删的那一帧同样回忙，不留超限的回应），记得进才把删除记进新的一组，
//! 同时照写结论的办法把删掉的键上的认领从内存认领表与写批里去掉；回应和删除同一组提交、WAL 同步之后才回。
//! 只数不删时扫得一样、不记删除、不动认领。弄坏开关 delete-prefix-response-unbudgeted 退回强行记下回应，delete-prefix-claims-kept 退回不释放认领，
//! delete-prefix-count-releases-claims 让只数不删也释放认领。一帧至多碰 `MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME` 把键，占着提交线程的时间有界（`protocol.rs` 文件头「删前缀」）。
//! 只有这一个线程改库，去重与认领的「先判再写」不会被别的线程插进来。
//! 读写库出错各记一行故障（`fault_log`），带出错那一帧的连接身份；一组写失败时这一组里每条连接各记一行。
//! 组提交的写批没写进去：这一组回存储出错、认领表从库里重读，照常服务（自证用 `--selftest-write-failure-file <文件>` 造，只认
//! `--selftest-instance`：那个文件在时写批不为空的组不写、当成写失败；容错客户端照同一请求号再交，`reliable_client.rs` 文件头「存储出错」）。
//! 写批写进去了、同步 WAL 失败：这一组回存储出错
//! （写进套接字再走）、记故障行，服务退出（退出码 7），不留在进程里接着回确认——规则与为什么在 `storage.rs` 文件头「同步 WAL 失败之后」。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};

use crate::connections::Responder;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::break_switches::BreakSwitch;
use crate::connections::ConnectionRecord;
use crate::fault_log::FaultKind;
use crate::protocol::{
    decode_request_body, encode_busy, encode_claim_outcomes, encode_prefix_deleted_response,
    encode_rejection, encode_write_acknowledgement, held_by_other_outcome_bytes,
    maximum_response_payload_bytes, ClaimOutcome, EntryInTable, KeyInTable, OperationCode,
    RequestRejection, RequestView, ResponseStatus, ServiceKeys, TrailingBytes,
    CLAIM_OUTCOME_COUNT_BYTES, FRAME_LENGTH_FIELD_BYTES, MAXIMUM_CLIENT_IDENTITY_BYTES,
    MAXIMUM_FRAME_BODY_BYTES, RESPONSE_HEADER_BYTES, SHORT_CLAIM_OUTCOME_BYTES,
};
use crate::server::{PendingSlot, SharedState, STORAGE_SYNC_FAILED_EXIT_CODE};
use crate::storage::{claim_key, ClaimRecord, GroupBatch, GroupCommitFailure};

/// 弄坏开关 acknowledge-before-sync 打开时，回确认之后过这么久才同步 WAL。
const ACKNOWLEDGE_BEFORE_SYNC_DELAY: Duration = Duration::from_secs(2);
/// 同步 WAL 失败、退出之前等这一组的存储出错写进套接字的限时。
const SYNC_FAILURE_REPLY_FLUSH_TIMEOUT: Duration = Duration::from_secs(5);

/// 认领逐键判出的结局（还没动库与认领表）；「别人持有」的身份与剩余租期到真认领时再从认领表里取。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ClaimVerdict {
    Claimed,
    HeldByOther,
    AlreadyConcluded,
}

pub struct CommitJob {
    pub body: Vec<u8>,
    pub responder: Responder,
    pub slot: PendingSlot,
    pub connection: Arc<ConnectionRecord>,
}

/// 解开的一帧删前缀（`protocol::RequestView::DeletePrefix` 的四个字段）。
struct DeletePrefixRequest<'frame> {
    prefix: &'frame [u8],
    after_key: &'frame [u8],
    maximum_keys: u32,
    count_only: bool,
}

/// 一帧的回应往哪交、占着的待处理名额、哪条连接来的（帧体拆出去解码之后剩下的那一半）。
struct Reply {
    responder: Responder,
    slot: PendingSlot,
    connection: Arc<ConnectionRecord>,
}

#[derive(Clone, Debug)]
pub struct CommitterSettings {
    pub group_window: Duration,
    pub group_maximum_frames: usize,
    pub selftest_commit_delay: Option<Duration>,
    /// 只给自证：写到这个键的那一组不落库、当成存储写失败（故障行那一格造服务端错用）。
    pub selftest_storage_failure_key: Option<Vec<u8>>,
    /// 只给自证：这个文件在时写批不为空的组不写、当成存储写失败，文件删掉就照常写（存储出错重放那一格造一次性的写失败用）。
    pub selftest_write_failure_file: Option<PathBuf>,
}

/// 自 1970 年起的毫秒；认领的租期按它算，服务重起之后照样对得上。
pub fn now_milliseconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
}

/// 起提交线程；认领记录先从库里读一遍。
pub fn start_committer(
    shared: &Arc<SharedState>,
    settings: CommitterSettings,
) -> Result<Sender<CommitJob>, String> {
    let claims = shared.library.load_claims(now_milliseconds())?;
    let (job_sender, job_receiver) = mpsc::channel::<CommitJob>();
    let shared = Arc::clone(shared);
    thread::spawn(move || {
        let mut committer = Committer {
            shared,
            settings,
            claims,
        };
        committer.run(&job_receiver);
    });
    Ok(job_sender)
}

/// 一帧在这一组里排着等同步之后才回的回应。
struct QueuedResponse {
    responder: Responder,
    response: Vec<u8>,
    slot: PendingSlot,
    sequence: u64,
    operation_code: u8,
    connection: Arc<ConnectionRecord>,
}

/// 正在攒的这一组。
struct Group {
    batch: GroupBatch,
    responses: Vec<QueuedResponse>,
    applied_in_group: HashSet<(Vec<u8>, u64)>,
    /// 这一组里有写到自证造错的那个键（`CommitterSettings::selftest_storage_failure_key`）。
    has_selftest_storage_failure: bool,
}

struct Committer {
    shared: Arc<SharedState>,
    settings: CommitterSettings,
    claims: HashMap<Vec<u8>, ClaimRecord>,
}

impl Committer {
    fn is_enabled(&self, switch: BreakSwitch) -> bool {
        self.shared.break_switches.is_enabled(switch)
    }

    fn run(&mut self, receiver: &Receiver<CommitJob>) {
        while let Ok(first) = receiver.recv() {
            let mut jobs = vec![first];
            if !self.is_enabled(BreakSwitch::GroupCommitDisabled) {
                let deadline = Instant::now() + self.settings.group_window;
                while jobs.len() < self.settings.group_maximum_frames {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    match receiver.recv_timeout(remaining) {
                        Ok(job) => jobs.push(job),
                        Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => break,
                    }
                }
            }
            self.process_group(jobs);
        }
    }

    fn new_group(&self) -> Group {
        Group {
            batch: self.shared.library.begin_group(),
            responses: Vec::new(),
            applied_in_group: HashSet::new(),
            has_selftest_storage_failure: false,
        }
    }

    /// 读库出错：记一行故障、当场回存储出错。
    fn reply_storage_read_failed(
        &self,
        job: Reply,
        sequence: u64,
        operation_code: u8,
        message: &str,
    ) {
        self.shared.fault_log.record(
            FaultKind::StorageReadFailed,
            Some(&*job.connection),
            message,
        );
        send_now(
            job.responder,
            encode_rejection(
                sequence,
                ResponseStatus::StorageFailed,
                operation_code,
                message,
            ),
            job.slot,
        );
    }

    fn process_group(&mut self, jobs: Vec<CommitJob>) {
        let mut group = self.new_group();
        for job in jobs {
            group = self.process_job(group, job);
        }
        self.commit(group);
    }

    /// 处理一帧；认领帧之前要先把这一组提交掉，所以回的是之后接着攒的那一组。
    fn process_job(&mut self, mut group: Group, commit_job: CommitJob) -> Group {
        let trailing_bytes = if self.is_enabled(BreakSwitch::TrailingBytesAccepted) {
            TrailingBytes::Accepted
        } else {
            TrailingBytes::Rejected
        };
        let CommitJob {
            body,
            responder,
            slot,
            connection,
        } = commit_job;
        let job = Reply {
            responder,
            slot,
            connection,
        };
        let service_keys = if self.is_enabled(BreakSwitch::ServiceKeysOpenToClients) {
            ServiceKeys::Accepted
        } else {
            ServiceKeys::Refused
        };
        let decoded = match decode_request_body(&body, trailing_bytes, service_keys) {
            Ok(decoded) => decoded,
            Err((
                sequence,
                RequestRejection::MalformedPayload {
                    operation_code,
                    reason,
                },
            )) => {
                send_now(
                    job.responder,
                    encode_rejection(
                        sequence,
                        ResponseStatus::MalformedPayload,
                        operation_code,
                        &reason,
                    ),
                    job.slot,
                );
                return group;
            }
            Err((sequence, RequestRejection::UnknownOperation { operation_code })) => {
                let message =
                    format!("提交线程收到认不得的操作码 {operation_code}；服务的分派有 bug");
                self.shared.fault_log.record(
                    FaultKind::DispatchBug,
                    Some(&*job.connection),
                    &message,
                );
                send_now(
                    job.responder,
                    encode_rejection(
                        sequence,
                        ResponseStatus::UnknownOperation,
                        operation_code,
                        &message,
                    ),
                    job.slot,
                );
                return group;
            }
        };
        let sequence = decoded.sequence;
        self.shared
            .counters
            .frames_handled
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match decoded.view {
            RequestView::BatchedWrite {
                client_identity,
                request_number,
                entries,
            } => {
                self.add_write(
                    &mut group,
                    job,
                    sequence,
                    client_identity,
                    request_number,
                    &entries,
                );
                group
            }
            RequestView::Claim {
                client_identity,
                lease_milliseconds,
                keys,
            } => {
                if !group.batch.is_empty() {
                    self.commit(group);
                    group = self.new_group();
                }
                self.add_claim(
                    &mut group,
                    job,
                    sequence,
                    client_identity,
                    lease_milliseconds,
                    &keys,
                );
                group
            }
            RequestView::DeletePrefix {
                prefix,
                after_key,
                maximum_keys,
                count_only,
            } => {
                if !group.batch.is_empty() {
                    self.commit(group);
                    group = self.new_group();
                }
                let request = DeletePrefixRequest {
                    prefix,
                    after_key,
                    maximum_keys,
                    count_only,
                };
                self.add_delete_prefix(&mut group, job, sequence, &request);
                group
            }
            RequestView::BatchedPointRead(_)
            | RequestView::PrefixScan { .. }
            | RequestView::Statistics
            | RequestView::ConnectionAnnouncement(_)
            | RequestView::ConnectedClients
            | RequestView::AnnounceStop => {
                let operation_code = body.get(8).copied().unwrap_or(0);
                let message = "提交线程只收写、认领与删前缀；服务的分派有 bug";
                self.shared.fault_log.record(
                    FaultKind::DispatchBug,
                    Some(&*job.connection),
                    message,
                );
                send_now(
                    job.responder,
                    encode_rejection(
                        sequence,
                        ResponseStatus::MalformedPayload,
                        operation_code,
                        message,
                    ),
                    job.slot,
                );
                group
            }
        }
    }

    fn add_write(
        &mut self,
        group: &mut Group,
        job: Reply,
        sequence: u64,
        client_identity: &[u8],
        request_number: u64,
        entries: &[EntryInTable<'_>],
    ) {
        let operation_code = OperationCode::BatchedWrite.code();
        let entries_written = u32::try_from(entries.len()).expect("条数来自帧里的 u32 字段");
        if !self.is_enabled(BreakSwitch::DuplicateRequestsReapplied) {
            let request = (client_identity.to_vec(), request_number);
            let earlier = if group.applied_in_group.contains(&request) {
                Ok(Some(entries_written))
            } else {
                self.shared
                    .library
                    .applied_request_entries(client_identity, request_number)
            };
            match earlier {
                Ok(Some(earlier_entries)) => {
                    self.shared
                        .counters
                        .duplicate_requests
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    queue(
                        group,
                        job,
                        encode_write_acknowledgement(sequence, earlier_entries, true),
                        sequence,
                        operation_code,
                    );
                    return;
                }
                Ok(None) => {}
                Err(message) => {
                    self.reply_storage_read_failed(job, sequence, operation_code, &message);
                    return;
                }
            }
        }
        if let Some(failure_key) = &self.settings.selftest_storage_failure_key {
            if entries
                .iter()
                .any(|entry| entry.key == failure_key.as_slice())
            {
                group.has_selftest_storage_failure = true;
            }
        }
        if self.is_enabled(BreakSwitch::WriteSyncPerEntry) {
            let response = match self.shared.library.write_each_entry_synced(
                client_identity,
                request_number,
                entries,
            ) {
                Ok(()) => encode_write_acknowledgement(sequence, entries_written, false),
                Err(message) => {
                    self.shared.fault_log.record(
                        FaultKind::StorageWriteFailed,
                        Some(&*job.connection),
                        &message,
                    );
                    encode_rejection(
                        sequence,
                        ResponseStatus::StorageFailed,
                        operation_code,
                        &message,
                    )
                }
            };
            send_now(job.responder, response, job.slot);
            return;
        }
        self.shared
            .library
            .add_write(&mut group.batch, client_identity, request_number, entries);
        if !self.is_enabled(BreakSwitch::ClaimKeptAfterConclusion) {
            for entry in entries {
                self.release_claim_on(&mut group.batch, entry.key);
            }
        }
        group
            .applied_in_group
            .insert((client_identity.to_vec(), request_number));
        queue(
            group,
            job,
            encode_write_acknowledgement(sequence, entries_written, false),
            sequence,
            operation_code,
        );
    }

    fn add_claim(
        &mut self,
        group: &mut Group,
        mut job: Reply,
        sequence: u64,
        client_identity: &[u8],
        lease_milliseconds: u32,
        keys: &[KeyInTable<'_>],
    ) {
        let operation_code = OperationCode::Claim.code();
        let response_header_bytes =
            usize::try_from(RESPONSE_HEADER_BYTES).expect("回应帧头 10 字节");
        let now = now_milliseconds();
        // 先逐键判、数回应的字节，不动库与认领表（文件头「认领」）。同一帧里重复的键判法相同：头一次认领到的，
        // 后面几次看到的是自己持有、照样认领到，所以不用边判边改。
        let mut verdicts = Vec::with_capacity(keys.len());
        let mut payload_bytes = CLAIM_OUTCOME_COUNT_BYTES;
        for key in keys {
            if !self.is_enabled(BreakSwitch::ClaimKeptAfterConclusion) {
                match self.shared.library.contains_key(key.key) {
                    Ok(true) => {
                        verdicts.push(ClaimVerdict::AlreadyConcluded);
                        payload_bytes += SHORT_CLAIM_OUTCOME_BYTES;
                        continue;
                    }
                    Ok(false) => {}
                    Err(message) => {
                        self.reply_storage_read_failed(job, sequence, operation_code, &message);
                        return;
                    }
                }
            }
            match self.held_by_other(key.key, client_identity, now) {
                Some(record) => {
                    verdicts.push(ClaimVerdict::HeldByOther);
                    payload_bytes += held_by_other_outcome_bytes(record.holder_identity.len());
                }
                None => {
                    verdicts.push(ClaimVerdict::Claimed);
                    payload_bytes += SHORT_CLAIM_OUTCOME_BYTES;
                }
            }
        }
        if !self.is_enabled(BreakSwitch::ClaimResponseUnbudgeted) {
            let response_frame_bytes =
                FRAME_LENGTH_FIELD_BYTES + response_header_bytes + payload_bytes;
            if payload_bytes > maximum_response_payload_bytes() {
                let message = format!(
                    "这一帧 {} 个键的认领回应帧体会有 {} 字节，超过帧长上限 {MAXIMUM_FRAME_BODY_BYTES}（「别人持有」每个带持有者身份，至多 {} 字节）；\
                     整帧拒了，一个键都没认领、没续租；把键拆成几帧再认领",
                    keys.len(),
                    payload_bytes + response_header_bytes,
                    held_by_other_outcome_bytes(MAXIMUM_CLIENT_IDENTITY_BYTES)
                );
                send_now(
                    job.responder,
                    encode_rejection(
                        sequence,
                        ResponseStatus::MalformedPayload,
                        operation_code,
                        &message,
                    ),
                    job.slot,
                );
                return;
            }
            if !job.slot.reserve_response(response_frame_bytes) {
                let retry_after = self.shared.retry_after_milliseconds();
                self.shared
                    .counters
                    .busy_rejections
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let message = format!(
                    "忙：认领回应 {response_frame_bytes} 字节记不进待处理额度；一个键都没认领、没续租，{retry_after} 毫秒后再来"
                );
                send_now(
                    job.responder,
                    encode_busy(sequence, operation_code, retry_after, &message),
                    job.slot,
                );
                return;
            }
        }
        let mut outcomes = Vec::with_capacity(keys.len());
        for (key, verdict) in keys.iter().zip(verdicts) {
            match verdict {
                ClaimVerdict::AlreadyConcluded => outcomes.push(ClaimOutcome::AlreadyConcluded),
                ClaimVerdict::HeldByOther => {
                    let record = self
                        .held_by_other(key.key, client_identity, now)
                        .expect("判的时候别人持有，这一帧里没认领过这把键，认领表没变");
                    outcomes.push(ClaimOutcome::HeldByOther {
                        holder_identity: record.holder_identity.clone(),
                        remaining_lease_milliseconds: record
                            .lease_expiry_milliseconds
                            .saturating_sub(now),
                    });
                }
                ClaimVerdict::Claimed => {
                    let claim = claim_key(key.key);
                    let requested_expiry = now.saturating_add(u64::from(lease_milliseconds));
                    // 自己持有、租期没到的：到期只往后挪，不照帧里更短的租期往前挪（文件头「认领」）
                    let lease_expiry_milliseconds = match self.claims.get(&claim) {
                        Some(held)
                            if held.holder_identity == client_identity
                                && held.lease_expiry_milliseconds > now
                                && !self.is_enabled(BreakSwitch::ClaimExpiryMovedEarlier) =>
                        {
                            held.lease_expiry_milliseconds.max(requested_expiry)
                        }
                        Some(_) | None => requested_expiry,
                    };
                    let record = ClaimRecord {
                        holder_identity: client_identity.to_vec(),
                        lease_expiry_milliseconds,
                    };
                    self.shared
                        .library
                        .add_claim(&mut group.batch, &claim, &record);
                    self.claims.insert(claim, record);
                    outcomes.push(ClaimOutcome::Claimed);
                }
            }
        }
        let response = encode_claim_outcomes(sequence, &outcomes);
        if self.is_enabled(BreakSwitch::ClaimResponseUnbudgeted) {
            queue(group, job, response, sequence, operation_code);
        } else {
            queue_reserved(group, job, response, sequence, operation_code);
        }
    }

    /// 删前缀一帧：扫、先把回应记进额度（记不进回忙、一把都不删），再把删除记进这一组、释放删掉的键上的认领，
    /// 回应排在这一组后面（文件头「删前缀」）。
    fn add_delete_prefix(
        &mut self,
        group: &mut Group,
        mut job: Reply,
        sequence: u64,
        request: &DeletePrefixRequest<'_>,
    ) {
        let operation_code = OperationCode::DeletePrefix.code();
        let deletion = match self.shared.library.scan_prefix_for_deletion(
            request.prefix,
            request.after_key,
            request.maximum_keys,
            request.count_only,
        ) {
            Ok(deletion) => deletion,
            Err(message) => {
                self.reply_storage_read_failed(job, sequence, operation_code, &message);
                return;
            }
        };
        let response = encode_prefix_deleted_response(
            sequence,
            deletion.keys_in_frame,
            deletion.is_exhausted,
            &deletion.last_key,
        );
        let is_response_budgeted = !self.is_enabled(BreakSwitch::DeletePrefixResponseUnbudgeted);
        if is_response_budgeted && !job.slot.reserve_response(response.len()) {
            let retry_after = self.shared.retry_after_milliseconds();
            self.shared
                .counters
                .busy_rejections
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let message = format!(
                "忙：删前缀回应 {} 字节（带最后一把键 {} 字节）记不进待处理额度；一把键都没删，{retry_after} 毫秒后再来",
                response.len(),
                deletion.last_key.len()
            );
            send_now(
                job.responder,
                encode_busy(sequence, operation_code, retry_after, &message),
                job.slot,
            );
            return;
        }
        let releases_claims = if deletion.deletes_touched_keys {
            !self.is_enabled(BreakSwitch::DeletePrefixClaimsKept)
        } else {
            self.is_enabled(BreakSwitch::DeletePrefixCountReleasesClaims)
        };
        for key in &deletion.touched_keys {
            if deletion.deletes_touched_keys {
                self.shared.library.add_key_deletion(&mut group.batch, key);
            }
            if releases_claims {
                self.release_claim_on(&mut group.batch, key);
            }
        }
        if is_response_budgeted {
            queue_reserved(group, job, response, sequence, operation_code);
        } else {
            queue(group, job, response, sequence, operation_code);
        }
    }

    /// 把 `key` 上的认领从内存认领表与 `batch` 里去掉（写结论、删前缀删掉这把键时；这把键上没有认领就不动）。
    fn release_claim_on(&mut self, batch: &mut GroupBatch, key: &[u8]) {
        let claim = claim_key(key);
        if self.claims.remove(&claim).is_some() {
            self.shared.library.add_claim_release(batch, &claim);
        }
    }

    /// 这把键此刻被别的身份持有、租期没到：回那条认领记录。
    fn held_by_other(&self, key: &[u8], client_identity: &[u8], now: u64) -> Option<&ClaimRecord> {
        self.claims.get(&claim_key(key)).filter(|record| {
            record.holder_identity != client_identity
                && (record.lease_expiry_milliseconds > now
                    || self.is_enabled(BreakSwitch::ClaimLeaseNeverExpires))
                && !self.is_enabled(BreakSwitch::ClaimNotExclusive)
        })
    }

    /// 写这一组、同步一次、再回这一组的全部回应。存储出错时这一组的回应都换成存储出错，认领表从库里重读。
    fn commit(&mut self, group: Group) {
        if group.responses.is_empty() && group.batch.is_empty() {
            return;
        }
        if let Some(delay) = self.settings.selftest_commit_delay {
            thread::sleep(delay);
        }
        let counters = &self.shared.counters;
        counters
            .group_commits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if group.has_selftest_storage_failure {
            self.fail_group(
                group.responses,
                "自证造的存储写失败（这一组写到了 --selftest-storage-failure-key 给的键），这一组没落库",
            );
            return;
        }
        let armed_write_failure = self
            .settings
            .selftest_write_failure_file
            .as_ref()
            .filter(|file| file.exists() && !group.batch.is_empty())
            .map(|file| {
                format!(
                    "写失败：自证造的（{} 在），这一组的写批没写进去",
                    file.display()
                )
            });
        if let Some(message) = armed_write_failure {
            self.fail_group(group.responses, &message);
            return;
        }
        if self.is_enabled(BreakSwitch::AcknowledgeBeforeSync) && !group.batch.is_empty() {
            if let Err(message) = self.shared.library.write_unsynced(group.batch) {
                self.fail_group(group.responses, &message);
                return;
            }
            for queued in group.responses {
                send_reserved(queued.responder, queued.response, queued.slot);
            }
            thread::sleep(ACKNOWLEDGE_BEFORE_SYNC_DELAY);
            match self.shared.library.synchronize_write_ahead_log() {
                Ok(duration) => self.shared.write_ahead_log_sync_timings.record(duration),
                Err(message) => self.shared.fault_log.record(
                    FaultKind::StorageWriteFailed,
                    None,
                    &format!(
                        "弄坏开关 acknowledge-before-sync 下回了确认之后同步 WAL 失败：{message}"
                    ),
                ),
            }
            return;
        }
        match self.shared.library.commit_group(group.batch) {
            Ok(synced) => {
                if let Some(duration) = synced {
                    self.shared.write_ahead_log_sync_timings.record(duration);
                }
                for queued in group.responses {
                    send_reserved(queued.responder, queued.response, queued.slot);
                }
            }
            Err(GroupCommitFailure::WriteFailed(message)) => {
                self.fail_group(group.responses, &message);
            }
            Err(GroupCommitFailure::SyncFailed(message)) => {
                if self.is_enabled(BreakSwitch::SyncFailureKeptServing) {
                    self.fail_group(group.responses, &message);
                } else {
                    self.exit_after_sync_failure(group.responses, &message);
                }
            }
        }
    }

    /// 同步 WAL 失败：这一组每帧回存储出错（写进套接字再走）、每条连接记一行故障，然后服务退出（`storage.rs` 文件头「同步 WAL 失败之后」）。
    fn exit_after_sync_failure(&self, responses: Vec<QueuedResponse>, message: &str) -> ! {
        let detail = format!(
            "{message}；同步 WAL 失败之后服务退出（退出码 {STORAGE_SYNC_FAILED_EXIT_CODE}），不再回确认，重起时从 WAL 重开库"
        );
        let mut recorded_connections = HashSet::new();
        for queued in &responses {
            if recorded_connections.insert(queued.connection.connection) {
                self.shared.fault_log.record(
                    FaultKind::StorageWriteFailed,
                    Some(&*queued.connection),
                    &detail,
                );
            }
        }
        let deadline = Instant::now() + SYNC_FAILURE_REPLY_FLUSH_TIMEOUT;
        for queued in responses {
            let QueuedResponse {
                responder,
                slot,
                sequence,
                operation_code,
                ..
            } = queued;
            let rejection = encode_rejection(
                sequence,
                ResponseStatus::StorageFailed,
                operation_code,
                &detail,
            );
            // 写不进套接字的（连接没了、写线程堵着到点）不再等：客户端没收到确认，照样从追加日志重放。
            let _ = responder.send_and_wait_flushed(rejection, deadline);
            drop(slot);
        }
        eprintln!("  ✗ prune-server：{detail}");
        eprintln!(
            "  → 怎么办：查库所在的盘与文件系统为什么同步失败（dmesg、df）；修好之后 python3 prune-server/control start 重起，\
             客户端没确认的写帧在各自的追加日志里，重交任务时重放"
        );
        std::process::exit(STORAGE_SYNC_FAILED_EXIT_CODE);
    }

    /// 一组写失败：这一组里每条连接记一行故障（同一条连接几帧只记一行），每帧回存储出错，认领表从库里重读。
    fn fail_group(&mut self, responses: Vec<QueuedResponse>, message: &str) {
        let mut recorded_connections = HashSet::new();
        for queued in &responses {
            if recorded_connections.insert(queued.connection.connection) {
                self.shared.fault_log.record(
                    FaultKind::StorageWriteFailed,
                    Some(&*queued.connection),
                    message,
                );
            }
        }
        for queued in responses {
            let rejection = encode_rejection(
                queued.sequence,
                ResponseStatus::StorageFailed,
                queued.operation_code,
                message,
            );
            send_now(queued.responder, rejection, queued.slot);
        }
        match self.shared.library.load_claims(now_milliseconds()) {
            Ok(claims) => self.claims = claims,
            Err(reload_failure) => eprintln!(
                "  ✗ 组提交失败之后重读认领表也失败：{reload_failure}；内存里的认领表可能比库里多"
            ),
        }
    }
}

/// 回应排进这一组、等同步之后回；回应的字节照记进名额，不看上限：只给写确认（几十字节，已经要做、撤不回），
/// 与弄坏开关 claim-response-unbudgeted、delete-prefix-response-unbudgeted 退回的写法。
fn queue(group: &mut Group, mut job: Reply, response: Vec<u8>, sequence: u64, operation_code: u8) {
    job.slot.force_response(response.len());
    queue_reserved(group, job, response, sequence, operation_code);
}

/// 同 `queue`，回应的字节调用方已经记进名额（认领与删前缀的回应先核过额度）。
fn queue_reserved(
    group: &mut Group,
    job: Reply,
    response: Vec<u8>,
    sequence: u64,
    operation_code: u8,
) {
    group.responses.push(QueuedResponse {
        responder: job.responder,
        response,
        slot: job.slot,
        sequence,
        operation_code,
        connection: job.connection,
    });
}

/// 当场回（拒绝、忙、存储出错）：回应的字节照记进名额，连同名额交给那条连接的写线程。
fn send_now(responder: Responder, response: Vec<u8>, mut slot: PendingSlot) {
    slot.force_response(response.len());
    send_reserved(responder, response, slot);
}

/// 回应连同名额交给那条连接的写线程，写出去或丢掉时名额才放手（`server.rs` 文件头「额度」）；连接已经断了的没人收，丢掉。
fn send_reserved(responder: Responder, response: Vec<u8>, slot: PendingSlot) {
    let _ = responder.send_response(response, slot);
}
