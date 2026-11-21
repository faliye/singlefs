//! 提交线程：写与认领都交给它，按到达次序组提交。
//!
//! 收到一帧之后在一个短窗口里（`--group-commit-milliseconds`）再收，至多收到 `--group-commit-frames` 帧，
//! 几帧的写合进一个 WriteBatch、只同步一次 WAL，同步完再一起回各自的确认；确认的意思不变：回确认时已落盘。
//! 去重：写帧按（客户端身份，请求号）查库里「已生效的请求号」那几行与这一组里已收的，早已生效的不再写，照回写确认（带「早已生效」）。
//! 认领：认领记录在内存里有一份（服务起来时从库里读，之后只由这个线程改），判「别人持有」不读库；判「已有结论」读库，
//! 所以认领帧之前这一组里改过数据行的（写，或真删了键的删前缀）先提交（同步、回确认），认领读的是已落盘的库；
//! 这一组里只有认领行与认领释放行时不用先提交（认领行在记账那一段，数据键的结论不受它影响），连着几帧认领合进同一次提交。写结论时同一批里删掉这些键上的认领。
//! 同一身份再认领自己持有、租期没到的键：到期时刻取已记下的与「现在加帧里的租期」里晚的那一个，只往后、不往前挪
//! （弄坏开关 claim-expiry-moved-earlier 退回照帧里的租期覆盖）。容错客户端每次发出去都给整段租期（`reliable_client.rs` 文件头「认领」），
//! 所以重发落下的、忙了很久才落下的认领，回到调用方手里时到期离落下那一刻都还有整段租期，调用方从回来那一刻起按整段租期写，盖得住；
//! 重发落下时往后推的那一段至多是这一次认领调用自己用掉的时间（重连总时长上限管着）。
//! 认领先逐键判结局、只数回应的字节，再动库与认领表：回应会越过帧长上限的整帧回状态 1（说明里写回应会有多少字节），
//! 回应记不进待处理额度的回忙，两种都一个键都不认领、不续租（弄坏开关 claim-response-unbudgeted 退回不核字节、照编超长回应的写法）。
//! 放认领（`protocol.rs` 文件头「放认领」）：只看内存认领表、只放帧里点名的持有者名下的，放掉的从内存认领表与这一组的写批里一起去掉，
//! 不碰数据行、不用先提交这一组（弄坏开关 claim-release-ignores-the-holder 退回不看持有者、有记录就放）。
//! 列认领（`protocol.rs` 文件头「列认领」）：只读内存认领表，键以前缀起头、租期此刻没到的按键升序列出，不改库、不动认领、不用先提交这一组；
//! 前缀不是 GPU 认领键的族前缀回状态 1，回应先记进待处理额度（记不进回忙、什么都没改），排在这一组后面同步之后才回。
//! 弄坏开关 claim-listing-includes-expired 连过了租期的也列，claim-listing-prefix-unchecked 不核前缀是不是族前缀，
//! claim-listing-ignores-the-prefix 不按请求的前缀筛、列全部认领，claim-listing-resume-key-ignored 不看续扫起点、总从头列，
//! claim-listing-writes-the-library 列的时候把列到的认领照原样再写一遍进这一组。
//! 删前缀：同样先把这一组里改过数据行的写与真删提交掉（删的是已落盘的库），再扫；只有认领行的一组不用先提交
//! （扫走到记账那一段就停，看不到认领行，`storage.rs` 的 `scan_prefix_for_deletion`）。回应（带这一帧最后一把键）先编好、照认领的办法先记进待处理额度，
//! 记不进整帧回忙、一把键都不删（只数不删的那一帧同样回忙，不留超限的回应），记得进才把删除记进新的一组，
//! 同时照写结论的办法把删掉的键上的认领从内存认领表与写批里去掉；回应和删除同一组提交、WAL 同步之后才回。
//! 只数不删时扫得一样、不记删除、不动认领。弄坏开关 delete-prefix-response-unbudgeted 退回强行记下回应，delete-prefix-claims-kept 退回不释放认领，
//! delete-prefix-count-releases-claims 让只数不删也释放认领。一帧至多碰 `MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME` 把键，占着提交线程的时间有界（`protocol.rs` 文件头「删前缀」）。
//! 不在才写与写结论（`protocol.rs` 文件头那两条）：同样先把这一组里改过数据行的写与真删提交掉，判的是已落盘的库。
//! 不在才写去重之后逐条点读键在不在，只写不在的，记的「已生效条数」是真写了的条数（弄坏开关 write-if-absent-overwrites 退回照成批写全写）。
//! 写结论按次序：去重、核认领（认领键在内存认领表里、持有者不是帧里的持有者、租期没到才拒）、核守卫那一行（点读、比 SHA-256）、
//! 逐把核删键（点读、与帧里给它的期待比 SHA-256），都过了才把条目、删键与认领的释放记进这一组；守卫不过当场回状态 7，不进组、不记已生效。核放在去重之后：重发一帧早已落下的写结论
//! 回「早已生效」，不因认领已被释放、别人接着认领或那一行已是它自己写的值而被拒（弄坏开关 conclusion-guard-before-deduplication 把核挪到去重之前）。
//! 弄坏开关 conclusion-guard-claim-unchecked、conclusion-guard-expired-claim-refused、conclusion-guard-own-claim-refused、
//! conclusion-guard-row-unchecked、conclusion-guard-deleted-row-unchecked、conclusion-write-pending-rows-kept 各弄坏核认领、认租期、认持有者、
//! 核守卫那一行、核删键、删待判行那一处。
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
use crate::gpu_judging_rows::GPU_CLAIM_FAMILY_PREFIX;
use crate::protocol::{
    decode_request_body, encode_busy, encode_claim_listing, encode_claim_outcomes,
    encode_claim_release_outcomes, encode_entries_acknowledgement, encode_guard_refusal,
    encode_prefix_deleted_response, encode_rejection, encode_write_acknowledgement,
    held_by_other_outcome_bytes, listed_claim_bytes, maximum_response_payload_bytes, ClaimOutcome,
    ClaimReleaseOutcome, EntryInTable, GuardRefusal, GuardedConclusionWriteView, KeyInTable,
    ListedClaim, OperationCode, RequestRejection, RequestView, ResponseStatus, ServiceKeys,
    TrailingBytes, CLAIM_LISTING_HEADER_BYTES, CLAIM_OUTCOME_COUNT_BYTES, FRAME_LENGTH_FIELD_BYTES,
    MAXIMUM_CLIENT_IDENTITY_BYTES, MAXIMUM_FRAME_BODY_BYTES, RESPONSE_HEADER_BYTES,
    SHORT_CLAIM_OUTCOME_BYTES,
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

/// 列认领帧解出来的三样（`protocol.rs` 文件头「列认领」）。
struct ClaimListingRequest<'frame> {
    prefix: &'frame [u8],
    after_key: &'frame [u8],
    maximum_entries: u32,
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
    /// 这一组里已生效的（客户端身份，请求号）与它第一次生效时写了几条：同组里重发的同一请求照第一次的条数回「早已生效」。
    applied_in_group: HashMap<(Vec<u8>, u64), u32>,
    /// 这一组里有写进数据行的写，或真删了键的删前缀：后面的认领与删前缀要读到落盘的库，所以先提交这一组。
    has_data_row_changes: bool,
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
            applied_in_group: HashMap::new(),
            has_data_row_changes: false,
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

    /// 处理一帧；认领与删前缀帧之前这一组改过数据行的要先提交掉，所以回的是之后接着攒的那一组。
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
                if group.has_data_row_changes {
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
                if group.has_data_row_changes {
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
            RequestView::WriteIfAbsent {
                client_identity,
                request_number,
                entries,
            } => {
                if group.has_data_row_changes {
                    self.commit(group);
                    group = self.new_group();
                }
                self.add_write_if_absent(
                    &mut group,
                    job,
                    sequence,
                    client_identity,
                    request_number,
                    &entries,
                );
                group
            }
            RequestView::GuardedConclusionWrite(conclusion) => {
                if group.has_data_row_changes {
                    self.commit(group);
                    group = self.new_group();
                }
                self.add_guarded_conclusion_write(&mut group, job, sequence, &conclusion);
                group
            }
            RequestView::ReleaseClaims {
                holder_identity,
                keys,
            } => {
                self.add_claim_release(&mut group, job, sequence, holder_identity, &keys);
                group
            }
            RequestView::ListClaims {
                prefix,
                after_key,
                maximum_entries,
            } => {
                self.add_claim_listing(
                    &mut group,
                    job,
                    sequence,
                    &ClaimListingRequest {
                        prefix,
                        after_key,
                        maximum_entries,
                    },
                );
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
        let Some(job) = self.answer_if_already_applied(
            group,
            job,
            sequence,
            OperationCode::BatchedWrite,
            client_identity,
            request_number,
        ) else {
            return;
        };
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
            .insert((client_identity.to_vec(), request_number), entries_written);
        if !entries.is_empty() {
            group.has_data_row_changes = true;
        }
        queue(
            group,
            job,
            encode_write_acknowledgement(sequence, entries_written, false),
            sequence,
            operation_code,
        );
    }

    /// 去重（文件头「去重」）：这个（身份，请求号）早已生效的照回确认（带「早已生效」与当时写了几条），读库出错回存储出错，
    /// 这两种回 `None`（这一帧答完了）；没生效过把 `job` 交回调用方接着写。弄坏开关 duplicate-requests-reapplied 下一律当没生效过。
    fn answer_if_already_applied(
        &self,
        group: &mut Group,
        job: Reply,
        sequence: u64,
        operation: OperationCode,
        client_identity: &[u8],
        request_number: u64,
    ) -> Option<Reply> {
        if self.is_enabled(BreakSwitch::DuplicateRequestsReapplied) {
            return Some(job);
        }
        let request = (client_identity.to_vec(), request_number);
        let earlier = match group.applied_in_group.get(&request) {
            Some(entries_of_the_first_application) => Ok(Some(*entries_of_the_first_application)),
            None => self
                .shared
                .library
                .applied_request_entries(client_identity, request_number),
        };
        match earlier {
            Ok(None) => Some(job),
            Ok(Some(earlier_entries)) => {
                self.shared
                    .counters
                    .duplicate_requests
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                queue(
                    group,
                    job,
                    encode_entries_acknowledgement(sequence, operation, earlier_entries, true),
                    sequence,
                    operation.code(),
                );
                None
            }
            Err(message) => {
                self.reply_storage_read_failed(job, sequence, operation.code(), &message);
                None
            }
        }
    }

    /// 不在才写一帧（`protocol.rs` 文件头「不在才写」）：去重之后逐条看键在不在已落盘的库，不在的写、在的不动；
    /// 调用方已把这一组里改过数据行的写提交掉。弄坏开关 write-if-absent-overwrites 下照成批写全写。
    fn add_write_if_absent(
        &mut self,
        group: &mut Group,
        job: Reply,
        sequence: u64,
        client_identity: &[u8],
        request_number: u64,
        entries: &[EntryInTable<'_>],
    ) {
        let operation = OperationCode::WriteIfAbsent;
        let Some(job) = self.answer_if_already_applied(
            group,
            job,
            sequence,
            operation,
            client_identity,
            request_number,
        ) else {
            return;
        };
        let overwrites = self.is_enabled(BreakSwitch::WriteIfAbsentOverwrites);
        let mut keys_in_frame: HashSet<&[u8]> = HashSet::with_capacity(entries.len());
        let mut absent_entries = Vec::with_capacity(entries.len());
        for entry in entries {
            if !keys_in_frame.insert(entry.key) {
                continue;
            }
            if overwrites {
                absent_entries.push(*entry);
                continue;
            }
            match self.shared.library.contains_key(entry.key) {
                Ok(true) => {}
                Ok(false) => absent_entries.push(*entry),
                Err(message) => {
                    self.reply_storage_read_failed(job, sequence, operation.code(), &message);
                    return;
                }
            }
        }
        let entries_written =
            u32::try_from(absent_entries.len()).expect("不多于帧里 u32 字段的条数");
        self.shared.library.add_write(
            &mut group.batch,
            client_identity,
            request_number,
            &absent_entries,
        );
        group
            .applied_in_group
            .insert((client_identity.to_vec(), request_number), entries_written);
        if !absent_entries.is_empty() {
            group.has_data_row_changes = true;
        }
        queue(
            group,
            job,
            encode_entries_acknowledgement(sequence, operation, entries_written, false),
            sequence,
            operation.code(),
        );
    }

    /// 写结论一帧（`protocol.rs` 文件头「写结论」五步）：去重、核认领、核守卫那一行、核删键，都过了才把条目、删键与认领的释放记进这一组；
    /// 守卫不过当场回状态 7、一条都不写。调用方已把这一组里改过数据行的写提交掉，核的是已落盘的库。
    fn add_guarded_conclusion_write(
        &mut self,
        group: &mut Group,
        mut job: Reply,
        sequence: u64,
        conclusion: &GuardedConclusionWriteView<'_>,
    ) {
        let operation = OperationCode::GuardedConclusionWrite;
        let checks_before_deduplication =
            self.is_enabled(BreakSwitch::ConclusionGuardBeforeDeduplication);
        if !checks_before_deduplication {
            let Some(not_applied_before) = self.answer_if_already_applied(
                group,
                job,
                sequence,
                operation,
                conclusion.client_identity,
                conclusion.request_number,
            ) else {
                return;
            };
            job = not_applied_before;
        }
        let guard_outcome = match self.conclusion_guard_refusal(conclusion) {
            Ok(guard_outcome) => guard_outcome,
            Err(message) => {
                self.reply_storage_read_failed(job, sequence, operation.code(), &message);
                return;
            }
        };
        if let Some((refusal, message)) = guard_outcome {
            send_now(
                job.responder,
                encode_guard_refusal(sequence, &refusal, &message),
                job.slot,
            );
            return;
        }
        if checks_before_deduplication {
            let Some(not_applied_before) = self.answer_if_already_applied(
                group,
                job,
                sequence,
                operation,
                conclusion.client_identity,
                conclusion.request_number,
            ) else {
                return;
            };
            job = not_applied_before;
        }
        self.shared.library.add_write(
            &mut group.batch,
            conclusion.client_identity,
            conclusion.request_number,
            &conclusion.entries,
        );
        if !self.is_enabled(BreakSwitch::ConclusionWritePendingRowsKept) {
            for deletion in &conclusion.deleted_keys {
                self.shared
                    .library
                    .add_key_deletion(&mut group.batch, deletion.key);
                self.release_claim_on(&mut group.batch, deletion.key);
            }
        }
        for entry in &conclusion.entries {
            self.release_claim_on(&mut group.batch, entry.key);
        }
        self.release_claim_on(&mut group.batch, conclusion.claim_key);
        let entries_written =
            u32::try_from(conclusion.entries.len()).expect("条数来自帧里的 u32 字段");
        group.applied_in_group.insert(
            (
                conclusion.client_identity.to_vec(),
                conclusion.request_number,
            ),
            entries_written,
        );
        group.has_data_row_changes = true;
        queue(
            group,
            job,
            encode_entries_acknowledgement(sequence, operation, entries_written, false),
            sequence,
            operation.code(),
        );
    }

    /// 写结论里核认领、核守卫那一行、核删键三步（`protocol.rs` 文件头「写结论」）：认领键被别的持有者持着、租期没到，守卫那一行此刻的样子与帧里的期待不同，
    /// 或一把删键那一行此刻的样子与帧里给它的期待不同，回守卫不过的种类与说明；都过了回 `None`；点读守卫那一行或删键出错回 `Err`。
    /// 比的是认领键上记的持有者与帧里的持有者身份（不是发帧的客户端身份）；租期按服务的时钟算。
    fn conclusion_guard_refusal(
        &self,
        conclusion: &GuardedConclusionWriteView<'_>,
    ) -> Result<Option<(GuardRefusal, String)>, String> {
        let now = now_milliseconds();
        if !self.is_enabled(BreakSwitch::ConclusionGuardClaimUnchecked) {
            if let Some(record) = self.claims.get(&claim_key(conclusion.claim_key)) {
                let is_other_holder = record.holder_identity != conclusion.holder_identity
                    || self.is_enabled(BreakSwitch::ConclusionGuardOwnClaimRefused);
                let is_lease_running = record.lease_expiry_milliseconds > now
                    || self.is_enabled(BreakSwitch::ConclusionGuardExpiredClaimRefused);
                if is_other_holder && is_lease_running {
                    let remaining_lease_milliseconds =
                        record.lease_expiry_milliseconds.saturating_sub(now);
                    let message = format!(
                        "认领键被别的持有者持着、租期还剩 {remaining_lease_milliseconds} 毫秒：这一帧一条都没写；这一块归持有者判，不再写它"
                    );
                    return Ok(Some((
                        GuardRefusal::ClaimHeldByOther {
                            holder_identity: record.holder_identity.clone(),
                            remaining_lease_milliseconds,
                        },
                        message,
                    )));
                }
            }
        }
        if !self.is_enabled(BreakSwitch::ConclusionGuardRowUnchecked) {
            let current = self.shared.library.guard_row_state(conclusion.guard_key)?;
            if current != conclusion.guard_expectation {
                return Ok(Some((
                    GuardRefusal::GuardRowChanged { current },
                    "守卫那一行此刻的样子与写的一方读到的那一份不同：这一帧一条都没写；重读那一行、照写的一方的规则比过再写"
                        .to_string(),
                )));
            }
        }
        if !self.is_enabled(BreakSwitch::ConclusionGuardDeletedRowUnchecked) {
            for deletion in &conclusion.deleted_keys {
                let current = self.shared.library.guard_row_state(deletion.key)?;
                if current != deletion.expectation {
                    return Ok(Some((
                        GuardRefusal::DeletedRowChanged {
                            key: deletion.key.to_vec(),
                            current,
                        },
                        "删键那一行此刻的样子与写的一方读到的那一份不同（读完之后被改过）：这一帧一条都没写、那一行没删；\
                         重读那一行、照写的一方的规则定删不删再写"
                            .to_string(),
                    )));
                }
            }
        }
        Ok(None)
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
                group.has_data_row_changes = true;
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

    /// 放认领一帧（`protocol.rs` 文件头「放认领」）：逐键看内存认领表，记的持有者就是帧里的持有者的放掉（内存认领表与这一组的写批一起），
    /// 没有记录回「没人持着」，记的是别的持有者回「别人持有」、不动。只动认领记录、不碰数据行，所以不用先提交这一组里改过数据行的写；
    /// 回应一键一字节、不长过请求，照写确认那样排进这一组、同步之后回。弄坏开关 claim-release-ignores-the-holder 下不看持有者，有记录就放。
    fn add_claim_release(
        &mut self,
        group: &mut Group,
        job: Reply,
        sequence: u64,
        holder_identity: &[u8],
        keys: &[KeyInTable<'_>],
    ) {
        let operation_code = OperationCode::ReleaseClaims.code();
        let ignores_the_holder = self.is_enabled(BreakSwitch::ClaimReleaseIgnoresTheHolder);
        let mut outcomes = Vec::with_capacity(keys.len());
        for key in keys {
            let is_held_by_the_named_holder = self
                .claims
                .get(&claim_key(key.key))
                .map(|record| record.holder_identity == holder_identity || ignores_the_holder);
            let outcome = match is_held_by_the_named_holder {
                None => ClaimReleaseOutcome::NotClaimed,
                Some(true) => {
                    self.release_claim_on(&mut group.batch, key.key);
                    ClaimReleaseOutcome::Released
                }
                Some(false) => ClaimReleaseOutcome::HeldByOther,
            };
            outcomes.push(outcome);
        }
        queue(
            group,
            job,
            encode_claim_release_outcomes(sequence, &outcomes),
            sequence,
            operation_code,
        );
    }

    /// 列认领一帧（`protocol.rs` 文件头「列认领」）：照内存认领表列出键以前缀起头、排在续扫起点之后、租期此刻没到的认领，按键升序，至多条数上限条、
    /// 回应不越过帧长上限（越过的在那一条处截断、标截断）。前缀不是 GPU 认领键的族前缀回状态 1；一条都装不下一帧回状态 1。
    /// 只读认领表：不改库、不动认领；回应先记进待处理额度（记不进回忙，什么都没改），排进这一组、同步之后回。
    /// 弄坏开关见文件头：claim-listing-includes-expired、claim-listing-prefix-unchecked、claim-listing-ignores-the-prefix、
    /// claim-listing-resume-key-ignored、claim-listing-writes-the-library。
    fn add_claim_listing(
        &mut self,
        group: &mut Group,
        mut job: Reply,
        sequence: u64,
        request: &ClaimListingRequest<'_>,
    ) {
        let operation_code = OperationCode::ListClaims.code();
        let is_prefix_checked = !self.is_enabled(BreakSwitch::ClaimListingPrefixUnchecked);
        if is_prefix_checked && !request.prefix.starts_with(GPU_CLAIM_FAMILY_PREFIX) {
            let message = format!(
                "列认领只收 GPU 认领键的族前缀（以 {} 起头），收到 {:02x?}",
                String::from_utf8_lossy(GPU_CLAIM_FAMILY_PREFIX),
                &request.prefix[..request.prefix.len().min(16)]
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
        let now = now_milliseconds();
        let includes_expired = self.is_enabled(BreakSwitch::ClaimListingIncludesExpired);
        let ignores_the_prefix = self.is_enabled(BreakSwitch::ClaimListingIgnoresThePrefix);
        let ignores_the_resume_key = self.is_enabled(BreakSwitch::ClaimListingResumeKeyIgnored);
        let writes_the_library = self.is_enabled(BreakSwitch::ClaimListingWritesTheLibrary);
        let mut matching: Vec<(&[u8], &ClaimRecord)> = self
            .claims
            .iter()
            .filter_map(|(claim, record)| {
                let key = claim.get(1..)?;
                let is_live = record.lease_expiry_milliseconds > now || includes_expired;
                let is_under_the_prefix = ignores_the_prefix || key.starts_with(request.prefix);
                let is_after_the_resume_key = ignores_the_resume_key
                    || request.after_key.is_empty()
                    || key > request.after_key;
                (is_live && is_under_the_prefix && is_after_the_resume_key).then_some((key, record))
            })
            .collect();
        matching.sort_unstable_by(|left, right| left.0.cmp(right.0));
        let maximum_entries = usize::try_from(request.maximum_entries).unwrap_or(usize::MAX);
        let payload_limit = maximum_response_payload_bytes();
        let mut payload_bytes = CLAIM_LISTING_HEADER_BYTES;
        let mut listed = Vec::new();
        let mut is_truncated = false;
        for (key, record) in matching {
            let entry_bytes = listed_claim_bytes(key.len(), record.holder_identity.len());
            if listed.len() >= maximum_entries || payload_bytes + entry_bytes > payload_limit {
                is_truncated = true;
                break;
            }
            payload_bytes += entry_bytes;
            if writes_the_library {
                self.shared
                    .library
                    .add_claim(&mut group.batch, &claim_key(key), record);
            }
            listed.push(ListedClaim {
                key: key.to_vec(),
                holder_identity: record.holder_identity.clone(),
                remaining_lease_milliseconds: record.lease_expiry_milliseconds.saturating_sub(now),
            });
        }
        if listed.is_empty() && is_truncated {
            let message = format!(
                "列认领的下一条认领连同回应头装不下一帧（帧长上限 {MAXIMUM_FRAME_BODY_BYTES} 字节）；没有列出任何认领"
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
        let response = encode_claim_listing(sequence, is_truncated, &listed);
        if !job.slot.reserve_response(response.len()) {
            let retry_after = self.shared.retry_after_milliseconds();
            self.shared
                .counters
                .busy_rejections
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let message = format!(
                "忙：列认领回应 {} 字节记不进待处理额度；什么都没改，{retry_after} 毫秒后再来",
                response.len()
            );
            send_now(
                job.responder,
                encode_busy(sequence, operation_code, retry_after, &message),
                job.slot,
            );
            return;
        }
        queue_reserved(group, job, response, sequence, operation_code);
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

    /// 一组写失败：这一组里每条连接记一行故障（同一条连接几帧只记一行），每帧回存储出错（原回应记进名额的字节先放回，
    /// 名额里只剩请求与这一帧拒绝），认领表从库里重读。
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
            let QueuedResponse {
                responder,
                response,
                mut slot,
                sequence,
                operation_code,
                ..
            } = queued;
            // 原回应不发了：它记在名额里的字节先放回，名额里才只剩请求与下面这一帧拒绝
            slot.release_response(response.len());
            let rejection = encode_rejection(
                sequence,
                ResponseStatus::StorageFailed,
                operation_code,
                message,
            );
            send_now(responder, rejection, slot);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::SimpleClient;
    use crate::in_process_service_for_tests::{
        committer_settings_grouping, ServiceUnderTest, TemporaryDirectoryOfTheTest,
        LOOPBACK_NETWORK_ADDRESS,
    };
    use crate::protocol::{
        begin_write_frame, encode_claim_frame, encode_delete_prefix_frame, statistics_value,
        RejectionStatus, ResponseOutcome, Table,
    };
    use crate::server::{GROUP_COMMITS_COUNTER, PENDING_BYTES_PEAK_COUNTER};

    /// 组提交窗口：一组收满 `group_maximum_frames` 帧就立刻提交，不等窗口；窗口只在凑不满时才用上（落库的单帧要等满它）。
    const GROUP_WINDOW: Duration = Duration::from_millis(500);
    /// 认领的租期：比用例跑的时间长得多。
    const CLAIM_LEASE_MILLISECONDS: u32 = 60_000;
    /// 连发的写帧用的请求号：比 `SimpleClient::write` 从 1 数起的号大得多，不与落种子的写撞。
    const REQUEST_NUMBER_OF_BACK_TO_BACK_WRITES: u64 = 1000;

    /// 一帧要发给提交线程的请求。
    enum FrameToCommit {
        Write {
            request_number: u64,
            entries: Vec<(&'static str, &'static str)>,
        },
        Claim {
            keys: Vec<String>,
        },
        DeletePrefix {
            prefix: &'static str,
            maximum_keys: u32,
        },
    }

    impl FrameToCommit {
        fn encode(&self, sequence: u64, client_identity: &[u8]) -> Vec<u8> {
            match self {
                FrameToCommit::Write {
                    request_number,
                    entries,
                } => {
                    let capacity: usize = entries
                        .iter()
                        .map(|(key, value)| 9 + key.len() + value.len())
                        .sum();
                    let mut builder = begin_write_frame(
                        sequence,
                        client_identity,
                        *request_number,
                        u32::try_from(entries.len()).expect("几条放得进 u32"),
                        capacity,
                    );
                    for (key, value) in entries {
                        builder.entry(Table::Facts, key.as_bytes(), value.as_bytes());
                    }
                    builder.finish()
                }
                FrameToCommit::Claim { keys } => {
                    let keys_in_table: Vec<(Table, &[u8])> = keys
                        .iter()
                        .map(|key| (Table::Facts, key.as_bytes()))
                        .collect();
                    encode_claim_frame(
                        sequence,
                        client_identity,
                        CLAIM_LEASE_MILLISECONDS,
                        &keys_in_table,
                    )
                }
                FrameToCommit::DeletePrefix {
                    prefix,
                    maximum_keys,
                } => encode_delete_prefix_frame(
                    sequence,
                    prefix.as_bytes(),
                    b"",
                    *maximum_keys,
                    false,
                ),
            }
        }
    }

    fn claim_of(key: &str) -> FrameToCommit {
        FrameToCommit::Claim {
            keys: vec![key.to_string()],
        }
    }

    /// 起一份服务，一组收满 `frames_per_group` 帧就提交。
    fn service_committing_every(name_prefix: &str, frames_per_group: usize) -> ServiceUnderTest {
        ServiceUnderTest::start(
            name_prefix,
            committer_settings_grouping(GROUP_WINDOW, frames_per_group),
        )
    }

    /// 在一条连接上不等回应连发 `frames`，等齐回应；回应按发的次序排。服务一组收满的帧数等于 `frames.len()` 时，这几帧就在同一组里。
    fn send_back_to_back(
        client: &mut SimpleClient,
        frames: &[FrameToCommit],
    ) -> Vec<ResponseOutcome> {
        let client_identity = client.client_identity.clone();
        let sequences: Vec<u64> = frames
            .iter()
            .map(|frame| {
                client
                    .sender
                    .send_frame(|sequence| frame.encode(sequence, &client_identity))
                    .expect("发得出去")
            })
            .collect();
        let mut outcome_by_sequence = HashMap::new();
        for _ in frames {
            let matched = client
                .receiver
                .receive()
                .unwrap_or_else(|failure| panic!("等回应：{failure}"));
            outcome_by_sequence.insert(matched.request_sequence, matched.response.outcome);
        }
        sequences
            .iter()
            .map(|sequence| {
                outcome_by_sequence
                    .remove(sequence)
                    .expect("每一帧都回了应")
            })
            .collect()
    }

    fn group_commits_so_far(client: &mut SimpleClient) -> u64 {
        let counters = client.statistics().expect("统计回得来");
        statistics_value(&counters, GROUP_COMMITS_COUNTER).expect("统计里有组提交的计数")
    }

    /// 连发 `frames`（服务一组收满的帧数等于帧数），回各帧的回应与这几帧一共做了几次组提交。
    fn send_back_to_back_counting_group_commits(
        client: &mut SimpleClient,
        frames: &[FrameToCommit],
    ) -> (Vec<ResponseOutcome>, u64) {
        let commits_before = group_commits_so_far(client);
        let outcomes = send_back_to_back(client, frames);
        (outcomes, group_commits_so_far(client) - commits_before)
    }

    /// 单帧写进 `keys`（值都是 `seeded`），落库、同步之后才回；服务一组收满的帧数大于 1 时这一帧要等满窗口。
    fn seed(client: &mut SimpleClient, keys: &[&str]) {
        let entries: Vec<(Table, Vec<u8>, Vec<u8>)> = keys
            .iter()
            .map(|key| (Table::Facts, key.as_bytes().to_vec(), b"seeded".to_vec()))
            .collect();
        client.write(&entries).expect("落种子的写回得了确认");
    }

    /// 连着几帧认领（各认领一把不同的键）在同一组里：只做一次组提交，每帧都认领到。
    /// 判别力：认领到达时只要这一组的写批不空（认领行也算）就先提交，这几帧被拆成每帧一次提交，计数是 4，这条红。
    #[test]
    fn consecutive_claim_frames_in_one_group_share_one_group_commit() {
        let running = service_committing_every("claims-share-commit", 4);
        let mut client = running.connect();
        let frames = [
            claim_of("claimed/1"),
            claim_of("claimed/2"),
            claim_of("claimed/3"),
            claim_of("claimed/4"),
        ];
        let (outcomes, group_commits) =
            send_back_to_back_counting_group_commits(&mut client, &frames);
        assert_eq!(
            outcomes,
            vec![ResponseOutcome::Claims(vec![ClaimOutcome::Claimed]); 4],
            "四把不同的键都该认领到"
        );
        assert_eq!(group_commits, 1, "同一组里只有认领行，该合进一次组提交");
    }

    /// 写之后的认领：认领读的是已落盘的库，所以写先提交；这把键已有结论，认领回「已有结论」，一共两次组提交。
    /// 判别力：只看「改过数据行」而漏掉写（写不置标记），认领读不到刚写的键，回「认领到」，这条红。
    #[test]
    fn claim_after_a_write_in_the_same_group_is_decided_after_the_write_is_committed() {
        let running = service_committing_every("claim-after-write", 2);
        let mut client = running.connect();
        let frames = [
            FrameToCommit::Write {
                request_number: REQUEST_NUMBER_OF_BACK_TO_BACK_WRITES,
                entries: vec![("written/1", "value")],
            },
            claim_of("written/1"),
        ];
        let (outcomes, group_commits) =
            send_back_to_back_counting_group_commits(&mut client, &frames);
        assert_eq!(
            outcomes,
            vec![
                ResponseOutcome::WriteAcknowledged {
                    entries_written: 1,
                    is_already_applied: false
                },
                ResponseOutcome::Claims(vec![ClaimOutcome::AlreadyConcluded]),
            ],
            "写过的键，同一组里随后的认领该回已有结论"
        );
        assert_eq!(group_commits, 2, "写先提交，认领与它自己的认领行再提交");
    }

    /// 真删之后的认领：删先提交，这把键已经没了，认领回「认领到」，一共两次组提交。
    /// 判别力：真删不置标记，认领读到还没删掉的键，回「已有结论」，这条红。
    #[test]
    fn claim_after_a_prefix_deletion_that_removed_keys_sees_them_gone() {
        let running = service_committing_every("claim-after-deletion", 2);
        let mut client = running.connect();
        seed(&mut client, &["doomed/1", "doomed/2"]);
        let frames = [
            FrameToCommit::DeletePrefix {
                prefix: "doomed/",
                maximum_keys: 10,
            },
            claim_of("doomed/1"),
        ];
        let (outcomes, group_commits) =
            send_back_to_back_counting_group_commits(&mut client, &frames);
        assert_eq!(
            outcomes,
            vec![
                ResponseOutcome::PrefixDeleted {
                    keys_in_frame: 2,
                    is_exhausted: true,
                    last_key: b"doomed/2".to_vec()
                },
                ResponseOutcome::Claims(vec![ClaimOutcome::Claimed]),
            ],
            "删掉的键，同一组里随后的认领该回认领到"
        );
        assert_eq!(group_commits, 2, "删先提交，认领再提交");
    }

    /// 真删之后的删前缀：后一帧扫的是删完的库，接着删后面的键，不重复碰前一帧删掉的。
    /// 判别力：真删不置标记，后一帧扫到还没删掉的键，重复碰前两把，最后一把键还是 doomed/2，这条红。
    #[test]
    fn prefix_deletion_after_a_prefix_deletion_that_removed_keys_continues_past_them() {
        let running = service_committing_every("deletion-after-deletion", 2);
        let mut client = running.connect();
        seed(
            &mut client,
            &["doomed/1", "doomed/2", "doomed/3", "doomed/4"],
        );
        let frames = [
            FrameToCommit::DeletePrefix {
                prefix: "doomed/",
                maximum_keys: 2,
            },
            FrameToCommit::DeletePrefix {
                prefix: "doomed/",
                maximum_keys: 2,
            },
        ];
        let (outcomes, group_commits) =
            send_back_to_back_counting_group_commits(&mut client, &frames);
        assert_eq!(
            outcomes,
            vec![
                ResponseOutcome::PrefixDeleted {
                    keys_in_frame: 2,
                    is_exhausted: false,
                    last_key: b"doomed/2".to_vec()
                },
                ResponseOutcome::PrefixDeleted {
                    keys_in_frame: 2,
                    is_exhausted: true,
                    last_key: b"doomed/4".to_vec()
                },
            ],
            "后一帧该删前一帧之后的两把键"
        );
        assert_eq!(group_commits, 2, "前一帧的删除先提交，后一帧再提交");
    }

    /// 写之后的删前缀：删读的是已落盘的库，写先提交，所以删得到刚写的键。
    /// 判别力：写不置标记，删前缀扫不到刚写的键，数到 0 把，这条红。
    #[test]
    fn prefix_deletion_after_a_write_in_the_same_group_deletes_the_written_key() {
        let running = service_committing_every("deletion-after-write", 2);
        let mut client = running.connect();
        let frames = [
            FrameToCommit::Write {
                request_number: REQUEST_NUMBER_OF_BACK_TO_BACK_WRITES,
                entries: vec![("doomed/1", "value")],
            },
            FrameToCommit::DeletePrefix {
                prefix: "doomed/",
                maximum_keys: 10,
            },
        ];
        let (outcomes, group_commits) =
            send_back_to_back_counting_group_commits(&mut client, &frames);
        assert_eq!(
            outcomes[1],
            ResponseOutcome::PrefixDeleted {
                keys_in_frame: 1,
                is_exhausted: true,
                last_key: b"doomed/1".to_vec()
            },
            "同一组里先写的键，随后的删前缀该删到"
        );
        assert_eq!(group_commits, 2, "写先提交，删再提交");
    }

    /// 认领之后的删前缀：这一组里只有认领行，删前缀扫不到记账那一段，不用先提交，认领和删合进一次组提交。
    /// 判别力：删前缀到达时只要写批不空就先提交，认领行被先提交一次，计数是 2，这条红。
    #[test]
    fn prefix_deletion_after_claims_in_the_same_group_shares_their_group_commit() {
        let running = service_committing_every("deletion-after-claims", 2);
        let mut client = running.connect();
        seed(&mut client, &["doomed/1", "doomed/2"]);
        let frames = [
            claim_of("other/1"),
            FrameToCommit::DeletePrefix {
                prefix: "doomed/",
                maximum_keys: 10,
            },
        ];
        let (outcomes, group_commits) =
            send_back_to_back_counting_group_commits(&mut client, &frames);
        assert_eq!(
            outcomes,
            vec![
                ResponseOutcome::Claims(vec![ClaimOutcome::Claimed]),
                ResponseOutcome::PrefixDeleted {
                    keys_in_frame: 2,
                    is_exhausted: true,
                    last_key: b"doomed/2".to_vec()
                },
            ],
            "认领到、两把键删掉"
        );
        assert_eq!(group_commits, 1, "认领行与删除合进一次组提交");
    }

    /// 同一组里同一个请求号先后两帧、条数不同：后到的一帧回「早已生效」，回执里的条数是第一帧生效时的条数，后一帧的键不落库。
    /// 判别力：同组命中回的是后到那一帧自己的条数（1），这条红。
    #[test]
    fn repeated_request_number_in_the_same_group_is_acknowledged_with_the_first_frames_entry_count()
    {
        let running = service_committing_every("repeated-request-in-group", 2);
        let mut client = running.connect();
        let frames = [
            FrameToCommit::Write {
                request_number: REQUEST_NUMBER_OF_BACK_TO_BACK_WRITES,
                entries: vec![("dup/a", "1"), ("dup/b", "2"), ("dup/c", "3")],
            },
            FrameToCommit::Write {
                request_number: REQUEST_NUMBER_OF_BACK_TO_BACK_WRITES,
                entries: vec![("dup/d", "4")],
            },
        ];
        let outcomes = send_back_to_back(&mut client, &frames);
        assert_eq!(
            outcomes,
            vec![
                ResponseOutcome::WriteAcknowledged {
                    entries_written: 3,
                    is_already_applied: false
                },
                ResponseOutcome::WriteAcknowledged {
                    entries_written: 3,
                    is_already_applied: true
                },
            ],
            "重发的那一帧回第一次生效的条数 3，不是它自己带的 1"
        );
        assert_eq!(
            client
                .read(&[(Table::Facts, b"dup/d".to_vec())])
                .expect("读得了"),
            vec![None],
            "早已生效的请求不再写，后一帧的键不该落库"
        );
    }

    /// 一组写失败：认领回应的字节先放回名额，名额里只剩请求与一帧拒绝。认领 600 把键，回应 600 多字节、比拒绝帧大；
    /// 待处理字节的峰值就是请求加认领回应（排进这一组那一刻），拒绝帧换上之后不会超过它。
    /// 判别力：换成拒绝帧之前不放回认领回应的字节，名额里是请求加认领回应再加拒绝帧，峰值超过请求加认领回应，这条红。
    #[test]
    fn failed_group_commit_leaves_the_slot_holding_only_the_request_and_the_rejection() {
        const CLAIMED_KEY_COUNT: usize = 600;
        let directory = TemporaryDirectoryOfTheTest::create("failed-group-slot");
        let write_failure_file = directory.path.join("write-failure");
        std::fs::File::create(&write_failure_file).expect("造得出写失败的标记文件");
        let running = ServiceUnderTest::start_in(
            directory,
            LOOPBACK_NETWORK_ADDRESS,
            CommitterSettings {
                group_window: GROUP_WINDOW,
                group_maximum_frames: 1,
                selftest_commit_delay: None,
                selftest_storage_failure_key: None,
                selftest_write_failure_file: Some(write_failure_file),
            },
        );
        let mut client = running.connect();
        let frame_to_commit = FrameToCommit::Claim {
            keys: (0..CLAIMED_KEY_COUNT)
                .map(|key_number| format!("claimed/{key_number:04}"))
                .collect(),
        };
        let request_body_bytes =
            frame_to_commit.encode(1, &client.client_identity).len() - FRAME_LENGTH_FIELD_BYTES;
        let claim_response_bytes =
            encode_claim_outcomes(1, &vec![ClaimOutcome::Claimed; CLAIMED_KEY_COUNT]).len();

        let outcomes = send_back_to_back(&mut client, &[frame_to_commit]);
        let ResponseOutcome::Rejected { status, message } = &outcomes[0] else {
            panic!("写失败的一组，认领该回拒绝，实际：{:?}", outcomes[0]);
        };
        assert_eq!(*status, RejectionStatus::StorageFailed);
        let rejection_bytes = encode_rejection(1, ResponseStatus::StorageFailed, 0, message).len();
        assert!(
            rejection_bytes < claim_response_bytes,
            "用例前提：拒绝帧（{rejection_bytes} 字节）该比认领回应（{claim_response_bytes} 字节）小，不然峰值分不出放没放回"
        );

        let counters = client.statistics().expect("统计回得来");
        let pending_bytes_peak = statistics_value(&counters, PENDING_BYTES_PEAK_COUNTER)
            .expect("统计里有待处理字节的峰值");
        let held_before_the_failure =
            u64::try_from(request_body_bytes + claim_response_bytes).expect("几 KiB 放得进 u64");
        assert!(
            pending_bytes_peak <= held_before_the_failure,
            "待处理字节峰值 {pending_bytes_peak} 超过了请求加认领回应的 {held_before_the_failure}：换成拒绝帧时认领回应的字节没放回"
        );
    }
}
