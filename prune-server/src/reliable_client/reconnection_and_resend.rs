//! 断线重连与重发：退避、建连、重连周期的总时限、标断、日志里确认、把没发出去的帧发出去。

use super::*;

impl ReliableClient {
    pub(super) fn backoff_delay(&mut self, attempts_made: u32) -> Duration {
        let policy = self.settings.reconnect;
        let nominal = if self.is_enabled(BreakSwitch::ReconnectBackoffFlat) {
            policy.initial_delay
        } else {
            let doublings = attempts_made.saturating_sub(1).min(20);
            policy
                .initial_delay
                .saturating_mul(1_u32 << doublings)
                .min(policy.maximum_delay)
        };
        if self.is_enabled(BreakSwitch::ReconnectWithoutJitter) {
            return nominal;
        }
        nominal.mul_f64(0.5 + 0.5 * self.next_random_fraction())
    }

    fn sleep_unless_stopped(&self, duration: Duration) -> Result<(), ClientFailure> {
        let deadline = Instant::now() + duration;
        loop {
            if (self.settings.stop_check)() {
                return Err(ClientFailure::Stopped);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Ok(());
            }
            thread::sleep(remaining.min(POLL_SLICE));
        }
    }

    /// 没连着就照重连策略连。一轮重连从要连的那一刻起，到连上之后收到第一个回应才结束：连得上、一发帧就断的
    /// （服务一接就关、中间的代理接了却连不上服务）也算这一轮里没成的一次，照样退避、照样数进次数与总时长上限，不会空转。
    fn ensure_connected(&mut self) -> Result<(), ClientFailure> {
        if self.connection.is_some() {
            return Ok(());
        }
        let policy = self.settings.reconnect;
        let is_unbounded = self.is_enabled(BreakSwitch::ReconnectUnbounded);
        let mut cycle = self
            .reconnect_cycle
            .take()
            .unwrap_or_else(|| ReconnectCycle {
                started: Instant::now(),
                attempt_times: Vec::new(),
                last_error: String::new(),
            });
        loop {
            if (self.settings.stop_check)() {
                self.diagnostics
                    .connection_attempt_cycles
                    .push(cycle.attempt_times);
                return Err(ClientFailure::Stopped);
            }
            if !cycle.attempt_times.is_empty() {
                let attempts = u32::try_from(cycle.attempt_times.len()).unwrap_or(u32::MAX);
                let delay = self.backoff_delay(attempts);
                let elapsed = cycle.started.elapsed();
                if !is_unbounded
                    && (attempts >= policy.maximum_attempts
                        || elapsed + delay > policy.maximum_total)
                {
                    self.diagnostics
                        .connection_attempt_cycles
                        .push(cycle.attempt_times);
                    return Err(ClientFailure::ServerUnreachable {
                        attempts,
                        elapsed,
                        last_error: cycle.last_error,
                    });
                }
                self.activity.enter(ClientActivity::BackingOff);
                if let Err(stopped) = self.sleep_unless_stopped(delay) {
                    self.diagnostics
                        .connection_attempt_cycles
                        .push(cycle.attempt_times);
                    return Err(stopped);
                }
            }
            let attempt_started = Instant::now();
            cycle.attempt_times.push(attempt_started);
            // 每次尝试只用这一轮剩下的时间（不设总时长上限的弄坏开关下每次给满一份上限）。
            let attempt_deadline = if is_unbounded {
                attempt_started + policy.maximum_total
            } else {
                cycle.started + policy.maximum_total
            };
            self.activity.enter(ClientActivity::Connecting);
            let opened = open_connection(&self.settings, attempt_deadline);
            self.activity.enter(ClientActivity::Outside);
            match opened {
                Ok(connection) => {
                    self.connection = Some(connection);
                    break;
                }
                Err(message) => cycle.last_error = message,
            }
        }
        if self.has_connected_before {
            self.diagnostics.reconnections += 1;
        }
        self.has_connected_before = true;
        self.reconnect_cycle = Some(cycle);
        Ok(())
    }

    /// 这一轮重连还没收到回应时，在途的帧最晚等到哪一刻：这一轮的总时长上限。连上了却一直没有回应（服务不读、不回）的，
    /// 也在总时长上限里收场，不等满一份帧超时。没在重连里、或弄坏开关 connect-without-deadline、reconnect-unbounded 下回 `None`。
    pub(super) fn reconnect_cycle_deadline(&self) -> Option<Instant> {
        if self.is_enabled(BreakSwitch::ConnectWithoutDeadline)
            || self.is_enabled(BreakSwitch::ReconnectUnbounded)
        {
            return None;
        }
        self.reconnect_cycle
            .as_ref()
            .map(|cycle| cycle.started + self.settings.reconnect.maximum_total)
    }

    /// 收到一个回应：这一轮重连算成了。
    pub(super) fn close_reconnect_cycle(&mut self) {
        if let Some(cycle) = self.reconnect_cycle.take() {
            self.diagnostics
                .connection_attempt_cycles
                .push(cycle.attempt_times);
        }
    }

    /// 连接坏了：断开，在途的帧都标成要重发。弄坏开关 unacknowledged-frames-dropped-on-reconnect 在对端关了连接或读写出错时
    /// （帧超时那一种不算）把在途的写当成已确认丢掉。
    pub(super) fn mark_broken(
        &mut self,
        reason: String,
        cause: BreakCause,
    ) -> Result<(), ClientFailure> {
        // 服务宣布停了之后马上退出、或拒了帧长之后关了连接时，写先出错、「服务停了」「帧长度不对」那一帧还躺在接收缓冲里：
        // 重连之前先把它读出来（文件头「单条超限当场报」）。
        if cause == BreakCause::PeerClosedOrFailed {
            match self.connection_notice_left_unread() {
                Some(ConnectionNotice::ServerStopped(message))
                    if !self.is_enabled(BreakSwitch::StopNoticeRetried) =>
                {
                    self.connection = None;
                    return Err(ClientFailure::ServerStopped { message });
                }
                Some(ConnectionNotice::FrameLengthRejected(message))
                    if !self.is_enabled(BreakSwitch::OversizeFrameSentAndRetried) =>
                {
                    self.connection = None;
                    return Err(ClientFailure::Protocol(format!(
                        "服务回「帧长度不对」并关了连接（上限 {MAXIMUM_FRAME_BODY_BYTES} 字节），不重连：{message}"
                    )));
                }
                Some(
                    ConnectionNotice::ServerStopped(_) | ConnectionNotice::FrameLengthRejected(_),
                )
                | None => {}
            }
        }
        let drops_unacknowledged = cause == BreakCause::PeerClosedOrFailed
            && self
                .settings
                .break_switches
                .is_enabled(BreakSwitch::UnacknowledgedFramesDroppedOnReconnect);
        self.connection = None;
        self.wire.clear();
        if let Some(cycle) = self.reconnect_cycle.as_mut() {
            cycle.last_error.clone_from(&reason);
        }
        self.diagnostics.last_break_reason = Some(reason);
        let on_wire: Vec<OperationIdentifier> = self
            .operations
            .iter()
            .filter(|(_, operation)| matches!(operation.state, OperationState::OnWire { .. }))
            .map(|(identifier, _)| *identifier)
            .collect();
        for identifier in on_wire {
            let operation = self.operations.get_mut(&identifier).expect("上面刚列出来");
            let dropped_request = match operation.kind {
                OperationKind::Write { request_number } if drops_unacknowledged => {
                    Some(request_number)
                }
                OperationKind::Write { .. }
                | OperationKind::Read
                | OperationKind::Scan
                | OperationKind::Claim { .. }
                | OperationKind::Statistics
                | OperationKind::DeletePrefix
                | OperationKind::ReleaseClaims
                | OperationKind::ListClaims => {
                    operation.state = OperationState::Unsent;
                    self.diagnostics.replayed_frames += 1;
                    None
                }
            };
            if let Some(request_number) = dropped_request {
                self.operations.remove(&identifier);
                self.acknowledge_in_journal(request_number)?;
                self.completions.push_back(Completion::WriteAcknowledged {
                    operation: identifier,
                    request_number,
                    entries_written: 0,
                    is_already_applied: false,
                });
            }
        }
        Ok(())
    }

    pub(super) fn acknowledge_in_journal(
        &mut self,
        request_number: u64,
    ) -> Result<(), ClientFailure> {
        let still_unacknowledged = self
            .operations
            .values()
            .filter_map(|operation| match operation.kind {
                OperationKind::Write {
                    request_number: unacknowledged,
                } => Some((unacknowledged, operation.frame.as_slice())),
                OperationKind::Read
                | OperationKind::Scan
                | OperationKind::Claim { .. }
                | OperationKind::Statistics
                | OperationKind::DeletePrefix
                | OperationKind::ReleaseClaims
                | OperationKind::ListClaims => None,
            });
        self.activity.enter(ClientActivity::WritingJournal);
        let journaled = self
            .journal
            .append_acknowledgement(request_number, still_unacknowledged);
        self.activity.enter(ClientActivity::Outside);
        journaled.map_err(ClientFailure::Journal)
    }

    /// 追加日志的追加与整理计数。
    pub fn journal_statistics(&self) -> JournalStatistics {
        self.journal.statistics()
    }

    /// 连上（必要时重连），按操作次序发出全部要发的帧；发到一半连接断了就重连再发。
    pub(super) fn send_unsent(&mut self) -> Result<(), ClientFailure> {
        loop {
            self.ensure_connected()?;
            match self.write_unsent_frames() {
                Ok(()) => return Ok(()),
                Err((reason, cause)) => {
                    if cause == BreakCause::WriteTimeout {
                        self.diagnostics.write_timeouts += 1;
                    }
                    self.mark_broken(reason, cause)?;
                }
            }
        }
    }

    fn write_unsent_frames(&mut self) -> Result<(), (String, BreakCause)> {
        let frame_timeout = self.settings.frame_timeout;
        let cycle_deadline = self.reconnect_cycle_deadline();
        let shrinks_claim_lease = self.is_enabled(BreakSwitch::ClaimLeaseShrunkOnResend);
        let Some(connection) = self.connection.as_mut() else {
            return Err(("没连着".to_string(), BreakCause::PeerClosedOrFailed));
        };
        for (identifier, operation) in &mut self.operations {
            if !matches!(operation.state, OperationState::Unsent) {
                continue;
            }
            let sequence = self.next_sequence;
            self.next_sequence += 1;
            replace_request_sequence(&mut operation.frame, sequence);
            if let OperationKind::Claim {
                lease_milliseconds,
                submitted_at,
            } = operation.kind
            {
                if shrinks_claim_lease {
                    replace_claim_lease(
                        &mut operation.frame,
                        claim_lease_left(lease_milliseconds, submitted_at.elapsed()),
                    );
                }
            }
            self.activity.enter(ClientActivity::WritingFrame);
            let written = connection.writer.write_all(&operation.frame);
            self.activity.enter(ClientActivity::Outside);
            written.map_err(|error| {
                let cause = if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) {
                    BreakCause::WriteTimeout
                } else {
                    BreakCause::PeerClosedOrFailed
                };
                (format!("发帧失败：{error}"), cause)
            })?;
            let frame_deadline = Instant::now() + frame_timeout;
            operation.state = OperationState::OnWire {
                deadline: cycle_deadline
                    .map_or(frame_deadline, |cycle_end| cycle_end.min(frame_deadline)),
            };
            self.wire.insert(sequence, *identifier);
        }
        Ok(())
    }
}
