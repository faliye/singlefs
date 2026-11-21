//! 工作线程执行读、扫、统计这类请求，编回应。

use super::*;

fn u32_count(count: usize) -> u32 {
    u32::try_from(count).expect("个数来自请求里的 u32 字段，或受帧长上限所限，放得进 u32")
}

fn decoded_operation_code(view: &RequestView<'_>) -> u8 {
    match view {
        RequestView::BatchedPointRead(_) => OperationCode::BatchedPointRead.code(),
        RequestView::BatchedWrite { .. } => OperationCode::BatchedWrite.code(),
        RequestView::PrefixScan { .. } => OperationCode::PrefixScan.code(),
        RequestView::Statistics => OperationCode::Statistics.code(),
        RequestView::Claim { .. } => OperationCode::Claim.code(),
        RequestView::ConnectionAnnouncement(_) => OperationCode::ConnectionAnnouncement.code(),
        RequestView::ConnectedClients => OperationCode::ConnectedClients.code(),
        RequestView::AnnounceStop => OperationCode::AnnounceStop.code(),
        RequestView::DeletePrefix { .. } => OperationCode::DeletePrefix.code(),
        RequestView::WriteIfAbsent { .. } => OperationCode::WriteIfAbsent.code(),
        RequestView::GuardedConclusionWrite(_) => OperationCode::GuardedConclusionWrite.code(),
        RequestView::ReleaseClaims { .. } => OperationCode::ReleaseClaims.code(),
        RequestView::ListClaims { .. } => OperationCode::ListClaims.code(),
    }
}

/// 读库出错：记一行故障、回存储出错。
fn storage_read_failed(
    shared: &SharedState,
    connection: &ConnectionRecord,
    sequence: u64,
    operation_code: u8,
    message: &str,
) -> Vec<u8> {
    shared
        .fault_log
        .record(FaultKind::StorageReadFailed, Some(connection), message);
    encode_rejection(
        sequence,
        ResponseStatus::StorageFailed,
        operation_code,
        message,
    )
}

pub(super) fn execute_request(
    shared: &SharedState,
    decoded: DecodedRequest<'_>,
    connection: &ConnectionRecord,
    budget: &mut ResponseBudget<'_>,
) -> Vec<u8> {
    let sequence = decoded.sequence;
    match decoded.view {
        RequestView::BatchedPointRead(keys) => {
            let operation_code = OperationCode::BatchedPointRead.code();
            let values = match shared.library.read_points(&keys) {
                Ok(values) => values,
                Err(message) => {
                    return storage_read_failed(
                        shared,
                        connection,
                        sequence,
                        operation_code,
                        &message,
                    )
                }
            };
            let payload_bytes = point_read_response_payload_bytes(
                values
                    .iter()
                    .map(|value| value.as_ref().map(|slice| slice.len())),
            );
            if point_read_response_exceeds_frame_limit(payload_bytes) {
                let message = format!("这一帧读回的值合计 {payload_bytes} 字节，回应会超过帧长上限 {MAXIMUM_FRAME_BODY_BYTES}；把这一帧的键拆成几帧");
                return encode_rejection(
                    sequence,
                    ResponseStatus::MalformedPayload,
                    operation_code,
                    &message,
                );
            }
            // 回应多大先数出来、记进额度再分配（文件头「额度」）：读回的值钉在块缓存里，记不进就回忙、不分配
            if !budget.take(payload_bytes) {
                return budget.busy_response(shared, sequence, operation_code, payload_bytes);
            }
            let mut encoder = begin_response(
                sequence,
                ResponseStatus::Completed,
                operation_code,
                payload_bytes,
            );
            encoder.put_u32(u32_count(values.len()));
            for value in &values {
                match value {
                    Some(slice) => {
                        encoder.put_u8(1);
                        encoder.put_length_prefixed(slice);
                    }
                    None => encoder.put_u8(0),
                }
            }
            encoder.finish()
        }
        RequestView::BatchedWrite { .. }
        | RequestView::Claim { .. }
        | RequestView::DeletePrefix { .. }
        | RequestView::WriteIfAbsent { .. }
        | RequestView::GuardedConclusionWrite(_)
        | RequestView::ReleaseClaims { .. }
        | RequestView::ListClaims { .. }
        | RequestView::ConnectionAnnouncement(_)
        | RequestView::ConnectedClients
        | RequestView::AnnounceStop => {
            let operation_code = decoded_operation_code(&decoded.view);
            let message = "工作线程只收成批点读、按前缀扫、统计（写、认领与删前缀走提交线程，报到这类走读线程）；服务的分派有 bug";
            shared
                .fault_log
                .record(FaultKind::DispatchBug, Some(connection), message);
            encode_rejection(
                sequence,
                ResponseStatus::MalformedPayload,
                operation_code,
                message,
            )
        }
        RequestView::PrefixScan {
            table: _,
            prefix,
            maximum_entries,
            after_key,
        } => {
            let operation_code = OperationCode::PrefixScan.code();
            let maximum_bytes =
                usize_from_u32(MAXIMUM_FRAME_BODY_BYTES - RESPONSE_HEADER_BYTES) - 5;
            // 回应边扫边编：每读出一条，回应的缓冲要多分配的那一截先记进额度（文件头「额度」），记得进才分配、放进回应；
            // 截断标记与条数占位，扫完再填
            let mut encoder =
                begin_response(sequence, ResponseStatus::Completed, operation_code, 5);
            encoder.put_u8(0);
            encoder.put_u32(0);
            let mut refused_growth_bytes = 0_usize;
            let scanned = shared.library.scan_prefix_into(
                prefix,
                after_key,
                maximum_entries,
                maximum_bytes,
                &mut |key, value| {
                    let growth = encoder.capacity_growth_for(8 + key.len() + value.len());
                    if !budget.take(growth) {
                        refused_growth_bytes = growth;
                        return false;
                    }
                    encoder.grow_capacity(growth);
                    encoder.put_length_prefixed(key);
                    encoder.put_length_prefixed(value);
                    true
                },
            );
            match scanned {
                Err(message) => {
                    budget.release();
                    storage_read_failed(shared, connection, sequence, operation_code, &message)
                }
                Ok(outcome) if outcome.is_refused_by_caller => {
                    drop(encoder);
                    budget.busy_response(shared, sequence, operation_code, refused_growth_bytes)
                }
                Ok(outcome) => {
                    encoder.set_prefix_scan_header(
                        outcome.is_truncated,
                        u32_count(outcome.entries_accepted),
                    );
                    encoder.finish()
                }
            }
        }
        RequestView::Statistics => {
            let operation_code = OperationCode::Statistics.code();
            let mut counters = shared.library.tickers();
            let own = &shared.counters;
            let (sync_count, sync_median, sync_ninety_ninth_percentile, sync_maximum) =
                shared.write_ahead_log_sync_timings.summary();
            let as_u64 = |count: usize| u64::try_from(count).unwrap_or(u64::MAX);
            counters.extend([
                (
                    FRAMES_HANDLED_COUNTER,
                    own.frames_handled.load(Ordering::Relaxed),
                ),
                (
                    BUSY_REJECTIONS_COUNTER,
                    own.busy_rejections.load(Ordering::Relaxed),
                ),
                (
                    DUPLICATE_REQUESTS_COUNTER,
                    own.duplicate_requests.load(Ordering::Relaxed),
                ),
                (
                    GROUP_COMMITS_COUNTER,
                    own.group_commits.load(Ordering::Relaxed),
                ),
                (
                    PENDING_PEAK_COUNTER,
                    as_u64(shared.admission.pending_peak.load(Ordering::SeqCst)),
                ),
                (
                    PENDING_LIMIT_COUNTER,
                    as_u64(shared.admission.limits.maximum_pending_total),
                ),
                (WRITE_AHEAD_LOG_SYNC_COUNT_COUNTER, sync_count),
                (WRITE_AHEAD_LOG_SYNC_MEDIAN_COUNTER, sync_median),
                (
                    WRITE_AHEAD_LOG_SYNC_NINETY_NINTH_PERCENTILE_COUNTER,
                    sync_ninety_ninth_percentile,
                ),
                (WRITE_AHEAD_LOG_SYNC_MAXIMUM_COUNTER, sync_maximum),
                (
                    PENDING_BYTES_COUNTER,
                    as_u64(shared.admission.pending_bytes.load(Ordering::SeqCst)),
                ),
                (
                    PENDING_BYTES_PEAK_COUNTER,
                    as_u64(shared.admission.pending_bytes_peak.load(Ordering::SeqCst)),
                ),
                (
                    RECEIVING_BODY_BYTES_PEAK_COUNTER,
                    as_u64(
                        shared
                            .admission
                            .receiving_body_bytes_peak
                            .load(Ordering::SeqCst),
                    ),
                ),
                (
                    WORKER_RESPONSE_BYTES_PEAK_COUNTER,
                    as_u64(own.worker_response_bytes_peak.load(Ordering::SeqCst)),
                ),
                (
                    OUTBOUND_QUEUED_BYTES_PEAK_COUNTER,
                    as_u64(
                        shared
                            .outbound_totals
                            .queued_bytes_peak
                            .load(Ordering::SeqCst),
                    ),
                ),
                (
                    CONNECTIONS_OPEN_COUNTER,
                    as_u64(own.connections_open.load(Ordering::SeqCst)),
                ),
                (
                    CONNECTIONS_REFUSED_COUNTER,
                    own.connections_refused.load(Ordering::Relaxed),
                ),
                (
                    FRAME_READ_TIMEOUTS_COUNTER,
                    own.frame_read_timeouts.load(Ordering::Relaxed),
                ),
                (
                    WRITE_TIMEOUTS_COUNTER,
                    own.write_timeouts.load(Ordering::Relaxed),
                ),
                (
                    OUTBOUND_OVERFLOWS_COUNTER,
                    shared
                        .outbound_totals
                        .overflow_closures
                        .load(Ordering::Relaxed),
                ),
            ]);
            let mut encoder = begin_response(
                sequence,
                ResponseStatus::Completed,
                operation_code,
                64 * counters.len(),
            );
            encoder.put_u32(u32_count(counters.len()));
            for (name, value) in counters {
                encoder.put_length_prefixed(name.as_bytes());
                encoder.put_u64(value);
            }
            encoder.finish()
        }
    }
}
