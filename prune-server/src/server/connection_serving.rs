//! 一条连接的读写线程：读线程把帧交到准入与分派，写线程按到达先后把回应写回，处理一帧。

use super::*;

/// 读线程当场做的几种（报到、列客户端、宣布停服务）。
fn handle_reader_request(
    body: &[u8],
    record: &ConnectionRecord,
    responder: &Responder,
    shared: &SharedState,
) {
    let response = match decode_request_body(body, TrailingBytes::Rejected, ServiceKeys::Refused) {
        Err((
            sequence,
            RequestRejection::MalformedPayload {
                operation_code,
                reason,
            },
        )) => Some(encode_rejection(
            sequence,
            ResponseStatus::MalformedPayload,
            operation_code,
            &reason,
        )),
        Err((sequence, RequestRejection::UnknownOperation { operation_code })) => {
            Some(encode_rejection(
                sequence,
                ResponseStatus::UnknownOperation,
                operation_code,
                "读线程收到认不得的操作码；服务的分派有 bug",
            ))
        }
        Ok(DecodedRequest { sequence, view }) => match view {
            RequestView::ConnectionAnnouncement(announcement) => {
                if !shared
                    .break_switches
                    .is_enabled(BreakSwitch::AnnouncementIgnored)
                {
                    record.record_announcement(announcement);
                }
                None
            }
            RequestView::ConnectedClients | RequestView::AnnounceStop
                if record.transport != Transport::UnixSocket =>
            {
                Some(encode_rejection(
                    sequence,
                    ResponseStatus::MalformedPayload,
                    peek_request_header(body).map_or(0, |(_, operation_code)| operation_code),
                    "列连着的客户端、宣布停服务只认 unix socket",
                ))
            }
            RequestView::ConnectedClients => Some(encode_connected_clients_response(
                sequence,
                OperationCode::ConnectedClients,
                &shared.connections.clients_except(record.connection),
            )),
            RequestView::AnnounceStop => {
                shared.is_stopping.store(true, Ordering::SeqCst);
                let clients = shared.connections.clients_except(record.connection);
                if !shared
                    .break_switches
                    .is_enabled(BreakSwitch::StopNoticeNotSent)
                {
                    // 每条连接的「服务停了」都写进套接字才回宣布的一方：控制命令拿到回话就 SIGTERM，排在队里没写的会丢
                    let (_, undelivered) = shared.connections.send_to_all_except_and_wait_flushed(
                        record.connection,
                        &encode_server_stopping(SERVER_STOPPING_MESSAGE),
                        Instant::now() + STOP_NOTICE_FLUSH_TIMEOUT,
                    );
                    if undelivered > 0 {
                        shared.fault_log.record(
                            FaultKind::StopNoticeUndelivered,
                            Some(record),
                            &format!("宣布停服务：{undelivered} 条连接的「服务停了」限时 {} 秒没写进套接字", STOP_NOTICE_FLUSH_TIMEOUT.as_secs()),
                        );
                    }
                }
                Some(encode_connected_clients_response(
                    sequence,
                    OperationCode::AnnounceStop,
                    &clients,
                ))
            }
            RequestView::BatchedPointRead(_)
            | RequestView::BatchedWrite { .. }
            | RequestView::PrefixScan { .. }
            | RequestView::Statistics
            | RequestView::Claim { .. }
            | RequestView::DeletePrefix { .. }
            | RequestView::WriteIfAbsent { .. }
            | RequestView::GuardedConclusionWrite(_)
            | RequestView::ReleaseClaims { .. }
            | RequestView::ListClaims { .. } => {
                shared.fault_log.record(
                    FaultKind::DispatchBug,
                    Some(record),
                    "读线程收到了该交工作线程或提交线程的帧",
                );
                Some(encode_rejection(
                    sequence,
                    ResponseStatus::MalformedPayload,
                    peek_request_header(body).map_or(0, |(_, operation_code)| operation_code),
                    "读线程只做报到、列客户端、宣布停服务；服务的分派有 bug",
                ))
            }
        },
    };
    if let Some(response) = response {
        // 连接正在收尾的，回应没人收，丢掉。
        let _ = responder.send(response);
    }
}

pub(super) fn handle_connection<Stream: ConnectionStream>(
    stream: Stream,
    origin: &ConnectionOrigin,
    shared: &Arc<SharedState>,
    job_sender: &Dispatch,
) {
    let budget = Arc::new(ConnectionBudget::default());
    let (Ok(writer_stream), Ok(shutdown_stream)) =
        (stream.clone_for_writing(), stream.clone_for_writing())
    else {
        stream.shut_down();
        return;
    };
    let limits = shared.admission.limits;
    if writer_stream
        .set_write_deadline_timeout(Some(limits.write_timeout))
        .is_err()
    {
        stream.shut_down();
        return;
    }
    let (response_sender, response_receiver) = Responder::channel(
        Arc::clone(&shared.outbound_totals),
        Box::new(move || shutdown_stream.shut_down()),
        shared.is_enabled(BreakSwitch::ResponseSlotReleasedOnEnqueue),
    );
    let writer_shared = Arc::clone(shared);
    let writer =
        thread::spawn(move || write_responses(writer_stream, &response_receiver, &writer_shared));
    let record = shared.connections.register(
        origin.transport,
        origin.peer_address.clone(),
        response_sender.clone(),
        &shared.fault_log,
    );
    let mut frames = FrameReader::new(stream, limits.frame_read_timeout);
    loop {
        let (body, reservation) = match frames.read_request_frame(shared, &budget) {
            RequestFrameRead::Body { body, reservation } => (body, reservation),
            RequestFrameRead::Closed => break,
            RequestFrameRead::TimedOut => {
                shared
                    .counters
                    .frame_read_timeouts
                    .fetch_add(1, Ordering::Relaxed);
                break;
            }
            RequestFrameRead::Discarded {
                sequence,
                operation_code,
                refusal,
            } => {
                let reason = format!("收这一帧：{refusal}；帧体读完丢掉了、没做");
                if !shared.reply_busy(&response_sender, sequence, operation_code, &reason) {
                    break;
                }
                continue;
            }
            RequestFrameRead::LengthRejected(declared_body_bytes) => {
                let message = format!(
                    "帧体长度 {declared_body_bytes} 不在 [{REQUEST_HEADER_BYTES}, {MAXIMUM_FRAME_BODY_BYTES}] 之内；帧边界找不回来，这条连接关掉"
                );
                let _ = response_sender.send(encode_rejection(
                    CONNECTION_LEVEL_SEQUENCE,
                    ResponseStatus::FrameLengthRejected,
                    0,
                    &message,
                ));
                break;
            }
        };
        let Some((sequence, operation_code)) = peek_request_header(&body) else {
            break;
        };
        let operation = OperationCode::from_code(operation_code);
        // 宣布停服务之后只认报到（连上就发，记不记都不碍事）与再宣布一次（控制命令可以再停一次），别的帧只回「服务停了」。
        let is_allowed_while_stopping = matches!(
            operation,
            Some(OperationCode::AnnounceStop | OperationCode::ConnectionAnnouncement)
        );
        if !is_allowed_while_stopping && shared.is_stopping.load(Ordering::SeqCst) {
            if !shared
                .break_switches
                .is_enabled(BreakSwitch::StopNoticeNotSent)
            {
                let _ = response_sender.send(encode_server_stopping(SERVER_STOPPING_MESSAGE));
            }
            break;
        }
        if operation.is_some_and(OperationCode::is_handled_by_connection_reader) {
            handle_reader_request(&body, &record, &response_sender, shared);
            continue;
        }
        let reservation = match reservation {
            Some(reservation) => reservation,
            None => match shared.admission.reserve_bytes(&budget, body.len()) {
                Some(reserved_after_reading) => reserved_after_reading,
                None => {
                    let reason = shared.admission.describe_bytes_refusal(&budget, body.len());
                    if !shared.reply_busy(&response_sender, sequence, operation_code, &reason) {
                        break;
                    }
                    continue;
                }
            },
        };
        let slot = match shared.admission.admit(&budget, reservation) {
            Ok(slot) => slot,
            Err(reason) => {
                if !shared.reply_busy(&response_sender, sequence, operation_code, &reason) {
                    break;
                }
                continue;
            }
        };
        let is_handled_by_committer = OperationCode::from_code(operation_code)
            .is_some_and(OperationCode::is_handled_by_committer);
        let handed_over = if is_handled_by_committer {
            job_sender
                .committer
                .send(CommitJob {
                    body,
                    responder: response_sender.clone(),
                    slot,
                    connection: Arc::clone(&record),
                })
                .is_ok()
        } else {
            job_sender
                .workers
                .send(Job {
                    body,
                    responder: response_sender.clone(),
                    slot,
                    connection: Arc::clone(&record),
                })
                .is_ok()
        };
        if !handed_over {
            break;
        }
    }
    shared.connections.unregister(record.connection);
    drop(response_sender);
    // 写线程等这条连接上还在处理的帧都回完（它们各持一份 responder）才退出。
    let _ = writer.join();
    frames.reader.get_ref().shut_down();
}

/// 一条连接的写线程：帧攒一批写、队空了 flush；碰到落盘回执先 flush 再回话（宣布停服务等「服务停了」真写进套接字）。
/// 写一次堵住到写期限（对端不收）就收尾：关掉连接两个方向（读线程随即读到流尾），排着的帧随写出队放手、名额随之放手。
fn write_responses<Stream: ConnectionStream>(
    stream: Stream,
    responses: &Receiver<Outgoing>,
    shared: &SharedState,
) {
    let mut writer = BufWriter::with_capacity(STREAM_BUFFER_BYTES, stream);
    if let Err(error) = write_until_senders_gone(&mut writer, responses) {
        if matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        ) {
            shared
                .counters
                .write_timeouts
                .fetch_add(1, Ordering::Relaxed);
        }
        writer.get_ref().shut_down();
    }
}

fn write_until_senders_gone<Stream: Write>(
    writer: &mut BufWriter<Stream>,
    responses: &Receiver<Outgoing>,
) -> io::Result<()> {
    while let Ok(first) = responses.recv() {
        let mut next = Some(first);
        while let Some(outgoing) = next.take() {
            match outgoing {
                Outgoing::Frame(frame) => {
                    writer.write_all(&frame.bytes)?;
                    // 写进去了：这一帧的账与名额放手。
                    drop(frame);
                }
                Outgoing::FlushedReceipt(receipt) => {
                    writer.flush()?;
                    let _ = receipt.send(());
                }
            }
            next = responses.try_recv().ok();
        }
        writer.flush()?;
    }
    Ok(())
}

pub(super) fn process_frame(
    shared: &SharedState,
    body: &[u8],
    connection: &ConnectionRecord,
    budget: &mut ResponseBudget<'_>,
) -> Vec<u8> {
    let trailing_bytes = if shared
        .break_switches
        .is_enabled(BreakSwitch::TrailingBytesAccepted)
    {
        TrailingBytes::Accepted
    } else {
        TrailingBytes::Rejected
    };
    let service_keys = if shared
        .break_switches
        .is_enabled(BreakSwitch::ServiceKeysOpenToClients)
    {
        ServiceKeys::Accepted
    } else {
        ServiceKeys::Refused
    };
    match decode_request_body(body, trailing_bytes, service_keys) {
        Err((sequence, RequestRejection::UnknownOperation { operation_code })) => {
            if shared
                .break_switches
                .is_enabled(BreakSwitch::UnknownOperationAccepted)
            {
                return begin_response(sequence, ResponseStatus::Completed, operation_code, 0)
                    .finish();
            }
            let message = format!("认不得的操作码 {operation_code}；认得的是 1 成批点读、2 成批写、3 按前缀扫、4 统计、5 认领");
            encode_rejection(
                sequence,
                ResponseStatus::UnknownOperation,
                operation_code,
                &message,
            )
        }
        Err((
            sequence,
            RequestRejection::MalformedPayload {
                operation_code,
                reason,
            },
        )) => encode_rejection(
            sequence,
            ResponseStatus::MalformedPayload,
            operation_code,
            &reason,
        ),
        Ok(decoded) => {
            if let Some(delay) = shared.selftest_delay {
                if delay.sequence == decoded.sequence {
                    thread::sleep(delay.delay);
                }
            }
            shared
                .counters
                .frames_handled
                .fetch_add(1, Ordering::Relaxed);
            execute_request(shared, decoded, connection, budget)
        }
    }
}
