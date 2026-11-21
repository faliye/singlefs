//! 收回应的泵：读到的回应按请求号对到在飞的操作上，连接级回应与停止通知，存储出错重试与忙回应推迟。

use super::*;

impl ReliableClient {
    /// 收一轮回应、处理到点的重发与超时；至多等 `maximum_wait`。
    pub(super) fn pump_once(&mut self, maximum_wait: Duration) -> Result<(), ClientFailure> {
        if (self.settings.stop_check)() {
            return Err(ClientFailure::Stopped);
        }
        let now = Instant::now();
        let mut has_unsent = false;
        for operation in self.operations.values_mut() {
            match operation.state {
                OperationState::Deferred {
                    due,
                    busy_received_at,
                    retry_after,
                } if due <= now => {
                    self.diagnostics
                        .busy_waits
                        .push((now.duration_since(busy_received_at), retry_after));
                    operation.state = OperationState::Unsent;
                    has_unsent = true;
                }
                OperationState::AwaitingStorageRetry { due } if due <= now => {
                    operation.state = OperationState::Unsent;
                    has_unsent = true;
                }
                OperationState::Unsent => has_unsent = true,
                OperationState::Deferred { .. }
                | OperationState::AwaitingStorageRetry { .. }
                | OperationState::OnWire { .. } => {}
            }
        }
        if has_unsent || self.connection.is_none() {
            self.send_unsent()?;
        }
        let ignores_timeout = self.is_enabled(BreakSwitch::FrameTimeoutIgnored);
        let mut wait = maximum_wait;
        for operation in self.operations.values() {
            let wake_at = match operation.state {
                OperationState::OnWire { deadline } if !ignores_timeout => Some(deadline),
                OperationState::Deferred { due, .. }
                | OperationState::AwaitingStorageRetry { due } => Some(due),
                OperationState::OnWire { .. } | OperationState::Unsent => None,
            };
            if let Some(wake_at) = wake_at {
                wait = wait.min(wake_at.saturating_duration_since(now));
            }
        }
        let wait = wait.max(Duration::from_millis(1));
        let read_result = {
            let Some(connection) = self.connection.as_mut() else {
                return Ok(());
            };
            self.activity.enter(ClientActivity::WaitingForResponses);
            let read_now = match connection.reader.set_read_timeout(Some(wait)) {
                Ok(()) => connection.reader.read(&mut self.scratch),
                Err(error) => Err(error),
            };
            self.activity.enter(ClientActivity::Outside);
            read_now
        };
        match read_result {
            Ok(0) => {
                return self.mark_broken("服务关了连接".to_string(), BreakCause::PeerClosedOrFailed)
            }
            Ok(read_now) => {
                let connection = self.connection.as_mut().expect("上面刚读过");
                connection
                    .buffer
                    .extend_from_slice(&self.scratch[..read_now]);
                self.handle_buffered_responses()?;
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => {
                return self.mark_broken(
                    format!("读回应出错：{error}"),
                    BreakCause::PeerClosedOrFailed,
                )
            }
        }
        if !ignores_timeout {
            let checked_at = Instant::now();
            let expired = self.operations.values().any(|operation| {
                matches!(operation.state, OperationState::OnWire { deadline } if deadline <= checked_at)
            });
            if expired {
                self.diagnostics.frame_timeouts += 1;
                let reason = match self.reconnect_cycle_deadline() {
                    Some(cycle_end) if cycle_end <= checked_at => format!(
                        "这一轮重连 {} 秒的总时长上限到了：连上了却一个回应都没收到（服务不读、不回）",
                        self.settings.reconnect.maximum_total.as_secs_f64()
                    ),
                    Some(_) | None => format!(
                        "在途的帧 {} 秒没回应",
                        self.settings.frame_timeout.as_secs_f64()
                    ),
                };
                return self.mark_broken(reason, BreakCause::FrameTimeout);
            }
        }
        Ok(())
    }

    fn handle_buffered_responses(&mut self) -> Result<(), ClientFailure> {
        let mut consumed = 0;
        loop {
            let Some(connection) = self.connection.as_ref() else {
                return Ok(());
            };
            let buffer = &connection.buffer[consumed..];
            let Some(length_field) = buffer.get(..FRAME_LENGTH_FIELD_BYTES) else {
                break;
            };
            let declared = u32::from_le_bytes(length_field.try_into().expect("4 字节"));
            if !is_frame_length_acceptable(declared, RESPONSE_HEADER_BYTES) {
                return Err(ClientFailure::Protocol(format!(
                    "回应帧体长度 {declared} 不对"
                )));
            }
            let frame_end =
                FRAME_LENGTH_FIELD_BYTES + usize::try_from(declared).expect("u32 放得进 usize");
            let Some(body) = buffer.get(FRAME_LENGTH_FIELD_BYTES..frame_end) else {
                break;
            };
            let response = decode_response_body(body).map_err(ClientFailure::Protocol)?;
            consumed += frame_end;
            self.handle_response(response)?;
        }
        if let Some(connection) = self.connection.as_mut() {
            connection.buffer.drain(..consumed);
        }
        Ok(())
    }

    /// 连接坏了的那一刻，接收缓冲与套接字里还没读的回应里有没有序号 0 的「服务停了」或「帧长度不对」；只读不处理别的帧。
    pub(super) fn connection_notice_left_unread(&mut self) -> Option<ConnectionNotice> {
        let connection = self.connection.as_mut()?;
        if connection
            .reader
            .set_read_timeout(Some(STOP_NOTICE_DRAIN_TIMEOUT))
            .is_ok()
        {
            let mut chunk = [0_u8; 64 * 1024];
            while let Ok(read_now) = connection.reader.read(&mut chunk) {
                if read_now == 0 {
                    break;
                }
                connection.buffer.extend_from_slice(&chunk[..read_now]);
            }
        }
        let mut buffer = connection.buffer.as_slice();
        while let Some(length_field) = buffer.get(..FRAME_LENGTH_FIELD_BYTES) {
            let declared = u32::from_le_bytes(length_field.try_into().expect("4 字节"));
            if !is_frame_length_acceptable(declared, RESPONSE_HEADER_BYTES) {
                return None;
            }
            let frame_end =
                FRAME_LENGTH_FIELD_BYTES + usize::try_from(declared).expect("u32 放得进 usize");
            let body = buffer.get(FRAME_LENGTH_FIELD_BYTES..frame_end)?;
            match decode_response_body(body) {
                Ok(Response {
                    sequence: CONNECTION_LEVEL_SEQUENCE,
                    outcome: ResponseOutcome::ServerStopping { message },
                    ..
                }) => return Some(ConnectionNotice::ServerStopped(message)),
                Ok(Response {
                    sequence: CONNECTION_LEVEL_SEQUENCE,
                    outcome:
                        ResponseOutcome::Rejected {
                            status: RejectionStatus::FrameLengthRejected,
                            message,
                        },
                    ..
                }) => return Some(ConnectionNotice::FrameLengthRejected(message)),
                Ok(_) | Err(_) => {}
            }
            buffer = &buffer[frame_end..];
        }
        None
    }

    /// 序号 0 的回应：服务停了、帧长度不对。
    fn handle_connection_level_response(
        &mut self,
        outcome: ResponseOutcome,
    ) -> Result<(), ClientFailure> {
        match outcome {
            ResponseOutcome::ServerStopping { message }
                if self.is_enabled(BreakSwitch::StopNoticeRetried) =>
            {
                self.mark_broken(
                    format!("服务说它停了：{message}"),
                    BreakCause::PeerClosedOrFailed,
                )
            }
            ResponseOutcome::ServerStopping { message } => {
                Err(ClientFailure::ServerStopped { message })
            }
            ResponseOutcome::Rejected {
                status: RejectionStatus::FrameLengthRejected,
                message,
            } if self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried) => self.mark_broken(
                format!("服务拒了整条连接：{message}"),
                BreakCause::PeerClosedOrFailed,
            ),
            other @ (ResponseOutcome::PointValues(_)
            | ResponseOutcome::WriteAcknowledged { .. }
            | ResponseOutcome::ScanEntries { .. }
            | ResponseOutcome::Statistics(_)
            | ResponseOutcome::Claims(_)
            | ResponseOutcome::ClaimsReleased(_)
            | ResponseOutcome::ClaimListing { .. }
            | ResponseOutcome::PrefixDeleted { .. }
            | ResponseOutcome::ConnectedClients(_)
            | ResponseOutcome::Busy { .. }
            | ResponseOutcome::Rejected { .. }
            | ResponseOutcome::GuardRefused { .. }) => Err(ClientFailure::Protocol(format!(
                "服务拒了整条连接：{other:?}"
            ))),
        }
    }

    fn handle_response(&mut self, response: Response) -> Result<(), ClientFailure> {
        if response.sequence == CONNECTION_LEVEL_SEQUENCE {
            return self.handle_connection_level_response(response.outcome);
        }
        let Some(identifier) = self.wire.remove(&response.sequence) else {
            if response.sequence < self.next_sequence {
                self.diagnostics.duplicate_responses += 1;
                if self.is_enabled(BreakSwitch::DuplicateResponsesFatal) {
                    return Err(ClientFailure::Protocol(format!(
                        "序号 {} 的回应已经收过一次",
                        response.sequence
                    )));
                }
                return Ok(());
            }
            return Err(ClientFailure::Protocol(format!(
                "回应的序号 {} 没发过（下一个要发的是 {}）",
                response.sequence, self.next_sequence
            )));
        };
        self.close_reconnect_cycle();
        let kind = self
            .operations
            .get(&identifier)
            .expect("在线路表里的操作都在操作表里")
            .kind;
        match (kind, response.outcome) {
            (
                _,
                ResponseOutcome::Busy {
                    retry_after_milliseconds,
                    ..
                },
            ) => self.defer_after_busy(
                identifier,
                kind,
                Duration::from_millis(u64::from(retry_after_milliseconds)),
            ),
            (
                OperationKind::Write { request_number },
                ResponseOutcome::WriteAcknowledged {
                    entries_written,
                    is_already_applied,
                },
            ) => {
                self.operations.remove(&identifier);
                self.acknowledge_in_journal(request_number)?;
                self.completions.push_back(Completion::WriteAcknowledged {
                    operation: identifier,
                    request_number,
                    entries_written,
                    is_already_applied,
                });
                Ok(())
            }
            (OperationKind::Read, ResponseOutcome::PointValues(values)) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::Values {
                    operation: identifier,
                    values,
                });
                Ok(())
            }
            (
                OperationKind::Scan,
                ResponseOutcome::ScanEntries {
                    is_truncated,
                    entries,
                },
            ) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::ScanEntries {
                    operation: identifier,
                    is_truncated,
                    entries,
                });
                Ok(())
            }
            (
                OperationKind::DeletePrefix,
                ResponseOutcome::PrefixDeleted {
                    keys_in_frame,
                    is_exhausted,
                    last_key,
                },
            ) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::PrefixDeleted {
                    operation: identifier,
                    keys_in_frame,
                    is_exhausted,
                    last_key,
                });
                Ok(())
            }
            (OperationKind::Claim { .. }, ResponseOutcome::Claims(outcomes)) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::Claims {
                    operation: identifier,
                    outcomes,
                });
                Ok(())
            }
            (OperationKind::ReleaseClaims, ResponseOutcome::ClaimsReleased(outcomes)) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::ClaimsReleased {
                    operation: identifier,
                    outcomes,
                });
                Ok(())
            }
            (
                OperationKind::ListClaims,
                ResponseOutcome::ClaimListing {
                    is_truncated,
                    claims,
                },
            ) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::ClaimListing {
                    operation: identifier,
                    is_truncated,
                    claims,
                });
                Ok(())
            }
            (OperationKind::Statistics, ResponseOutcome::Statistics(counters)) => {
                self.operations.remove(&identifier);
                self.completions.push_back(Completion::Statistics {
                    operation: identifier,
                    counters,
                });
                Ok(())
            }
            (
                OperationKind::Write { request_number },
                ResponseOutcome::GuardRefused { refusal, message },
            ) => {
                // 守卫不过是这一帧的结局、不是协议错（文件头「写结论」）：一条都没写，从追加日志里勾掉，结局交调用方。
                self.operations.remove(&identifier);
                self.acknowledge_in_journal(request_number)?;
                self.completions.push_back(Completion::ConclusionRefused {
                    operation: identifier,
                    request_number,
                    refusal,
                    message,
                });
                Ok(())
            }
            (
                OperationKind::Write { request_number },
                ResponseOutcome::Rejected {
                    status: RejectionStatus::StorageFailed,
                    message,
                },
            ) if !self.is_enabled(BreakSwitch::WriteStorageFailureAbandoned) => {
                self.retry_after_storage_failure(identifier, request_number, message)
            }
            (_, ResponseOutcome::Rejected { status, message }) => {
                self.operations.remove(&identifier);
                let failure = ClientFailure::Rejected { status, message };
                if matches!(kind, OperationKind::Write { .. }) {
                    // 写被拒是协议的错（内容解不开、认不得的操作码），重发也不会好；那一帧留在追加日志里，交调用方定。
                    // 存储出错不走这一支（上一支，文件头「存储出错」）。
                    return Err(failure);
                }
                self.completions.push_back(Completion::Failed {
                    operation: identifier,
                    failure,
                });
                Ok(())
            }
            (kind, outcome) => Err(ClientFailure::Protocol(format!(
                "{kind:?} 帧收到的回应对不上：{outcome:?}"
            ))),
        }
    }

    /// 写帧回了存储出错（文件头「存储出错」）：帧留在操作表与追加日志里，照重连策略退避一段之后用同一个请求号再交；
    /// 同一帧连着回到重连次数上限回 `StorageFailing`（帧照旧留在操作表与日志里）。
    fn retry_after_storage_failure(
        &mut self,
        identifier: OperationIdentifier,
        request_number: u64,
        message: String,
    ) -> Result<(), ClientFailure> {
        self.diagnostics.write_storage_failures += 1;
        let failures = {
            let operation = self
                .operations
                .get_mut(&identifier)
                .expect("调用方刚从操作表里取到");
            operation.storage_failures += 1;
            operation.storage_failures
        };
        let is_unbounded = self.is_enabled(BreakSwitch::ReconnectUnbounded);
        if failures >= self.settings.reconnect.maximum_attempts && !is_unbounded {
            self.operations
                .get_mut(&identifier)
                .expect("调用方刚从操作表里取到")
                .state = OperationState::Unsent;
            return Err(ClientFailure::StorageFailing {
                request_number,
                attempts: failures,
                last_error: message,
            });
        }
        let delay = self.backoff_delay(failures);
        self.operations
            .get_mut(&identifier)
            .expect("调用方刚从操作表里取到")
            .state = OperationState::AwaitingStorageRetry {
            due: Instant::now() + delay,
        };
        Ok(())
    }

    fn defer_after_busy(
        &mut self,
        identifier: OperationIdentifier,
        kind: OperationKind,
        retry_after: Duration,
    ) -> Result<(), ClientFailure> {
        self.diagnostics.busy_responses += 1;
        let jitter = retry_after.mul_f64(0.5 * self.next_random_fraction());
        let delay = if self.is_enabled(BreakSwitch::BusyRetryWithoutDelay) {
            Duration::ZERO
        } else {
            retry_after + jitter
        };
        let maximum_retries = self.settings.maximum_busy_retries;
        let operation = self
            .operations
            .get_mut(&identifier)
            .expect("调用方刚从操作表里取到");
        operation.busy_retries += 1;
        if operation.busy_retries > maximum_retries {
            let retries = operation.busy_retries;
            if matches!(kind, OperationKind::Write { .. }) {
                operation.state = OperationState::Unsent;
                return Err(ClientFailure::BusyLimit { retries });
            }
            self.operations.remove(&identifier);
            self.completions.push_back(Completion::Failed {
                operation: identifier,
                failure: ClientFailure::BusyLimit { retries },
            });
            return Ok(());
        }
        let now = Instant::now();
        operation.state = OperationState::Deferred {
            due: now + delay,
            busy_received_at: now,
            retry_after,
        };
        Ok(())
    }
}
