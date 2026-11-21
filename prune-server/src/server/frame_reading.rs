//! 一条连接上读请求帧：按长度字段切帧、帧开头之前的空闲不限期、一帧的期限、额度不够时丢弃帧体。

use super::*;

/// 读一帧请求的结局。
pub(super) enum RequestFrameRead {
    /// 收齐的帧体与记着它字节的那一笔额度（弄坏开关 frame-length-unchecked、frame-body-read-before-admission 下还没记，是 `None`）。
    Body {
        body: Vec<u8>,
        reservation: Option<ByteReservation>,
    },
    /// 收帧额度不够：帧体读完丢掉了（帧边界保住），调用方回忙。
    Discarded {
        sequence: u64,
        operation_code: u8,
        refusal: String,
    },
    /// 在帧边界上读到流尾，或读出错：连接收尾。
    Closed,
    /// 一帧开了头、到期限还没收齐（文件头「期限」）：连接收尾。
    TimedOut,
    /// 声称的帧体长度不在 [帧头, 上限] 之内。
    LengthRejected(u32),
}

/// 读帧停在哪：流尾或读出错、到了期限。
enum FrameStop {
    Closed,
    TimedOut,
}

impl From<FrameStop> for RequestFrameRead {
    fn from(stop: FrameStop) -> RequestFrameRead {
        match stop {
            FrameStop::Closed => RequestFrameRead::Closed,
            FrameStop::TimedOut => RequestFrameRead::TimedOut,
        }
    }
}

/// 一条连接的读端：按长度字段切帧，一帧开了头之后在期限里收齐（文件头「期限」）。
pub(super) struct FrameReader<Stream: ConnectionStream> {
    pub(super) reader: BufReader<Stream>,
    pub(super) frame_read_timeout: Duration,
    /// 套接字此刻设着读超时（帧开头之前的空闲不设）。
    has_read_timeout: bool,
    /// 收帧额度不够时帧体读进这里丢掉；头一次要用时才分配。
    discard_buffer: Vec<u8>,
}

impl<Stream: ConnectionStream> FrameReader<Stream> {
    pub(super) fn new(stream: Stream, frame_read_timeout: Duration) -> FrameReader<Stream> {
        FrameReader {
            reader: BufReader::with_capacity(STREAM_BUFFER_BYTES, stream),
            frame_read_timeout,
            has_read_timeout: false,
            discard_buffer: Vec::new(),
        }
    }

    /// 读一些进 `destination`（至少 1 字节）；`deadline` 为 `None` 时一直等（帧开头之前的空闲）。
    fn read_some(
        &mut self,
        destination: &mut [u8],
        deadline: Option<Instant>,
    ) -> Result<usize, FrameStop> {
        loop {
            let timeout = match deadline {
                None => None,
                Some(frame_deadline) => {
                    let remaining = frame_deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return Err(FrameStop::TimedOut);
                    }
                    Some(remaining.max(Duration::from_millis(1)))
                }
            };
            if timeout.is_some() || self.has_read_timeout {
                if self
                    .reader
                    .get_ref()
                    .set_read_deadline_timeout(timeout)
                    .is_err()
                {
                    return Err(FrameStop::Closed);
                }
                self.has_read_timeout = timeout.is_some();
            }
            match self.reader.read(destination) {
                Ok(0) => return Err(FrameStop::Closed),
                Ok(read_now) => return Ok(read_now),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err(FrameStop::Closed),
            }
        }
    }

    fn read_exact_before(
        &mut self,
        destination: &mut [u8],
        deadline: Instant,
    ) -> Result<(), FrameStop> {
        let mut filled = 0;
        while filled < destination.len() {
            filled += self.read_some(&mut destination[filled..], Some(deadline))?;
        }
        Ok(())
    }

    /// 读完 `byte_count` 字节丢掉（收帧额度不够时保住帧边界）。
    fn discard_before(&mut self, byte_count: usize, deadline: Instant) -> Result<(), FrameStop> {
        if self.discard_buffer.is_empty() {
            self.discard_buffer = vec![0_u8; DISCARD_CHUNK_BYTES];
        }
        let mut discard_buffer = std::mem::take(&mut self.discard_buffer);
        let mut left = byte_count;
        let mut outcome = Ok(());
        while left > 0 {
            let chunk = left.min(discard_buffer.len());
            match self.read_some(&mut discard_buffer[..chunk], Some(deadline)) {
                Ok(read_now) => left -= read_now,
                Err(stop) => {
                    outcome = Err(stop);
                    break;
                }
            }
        }
        self.discard_buffer = discard_buffer;
        outcome
    }

    /// 读一帧请求：长度字段的头一个字节之前不限时，之后整帧在 `frame_read_timeout` 里收齐；帧体分配之前先记进额度，
    /// 记不进就读完丢掉、回 `Discarded`（文件头「额度」）。
    pub(super) fn read_request_frame(
        &mut self,
        shared: &SharedState,
        budget: &Arc<ConnectionBudget>,
    ) -> RequestFrameRead {
        let mut length_field = [0_u8; FRAME_LENGTH_FIELD_BYTES];
        if let Err(stop) = self.read_some(&mut length_field[..1], None) {
            return stop.into();
        }
        let deadline = Instant::now() + self.frame_read_timeout;
        if let Err(stop) = self.read_exact_before(&mut length_field[1..], deadline) {
            return stop.into();
        }
        let declared_body_bytes = u32::from_le_bytes(length_field);
        if shared.is_enabled(BreakSwitch::FrameLengthUnchecked) {
            if self
                .reader
                .get_ref()
                .set_read_deadline_timeout(None)
                .is_err()
            {
                return RequestFrameRead::Closed;
            }
            self.has_read_timeout = false;
            let mut body = Vec::new();
            return match (&mut self.reader)
                .take(u64::from(declared_body_bytes))
                .read_to_end(&mut body)
            {
                Ok(_) => RequestFrameRead::Body {
                    body,
                    reservation: None,
                },
                Err(_) => RequestFrameRead::Closed,
            };
        }
        if !is_frame_length_acceptable(declared_body_bytes, REQUEST_HEADER_BYTES) {
            return RequestFrameRead::LengthRejected(declared_body_bytes);
        }
        let body_bytes = usize_from_u32(declared_body_bytes);
        let reservation = if shared.is_enabled(BreakSwitch::FrameBodyReadBeforeAdmission) {
            None
        } else {
            match shared.admission.reserve_bytes(budget, body_bytes) {
                Some(reservation) => Some(reservation),
                None => {
                    let refusal = shared.admission.describe_bytes_refusal(budget, body_bytes);
                    return self.discard_refused_frame(body_bytes, deadline, refusal);
                }
            }
        };
        let receiving = ReceivingBody::start(&shared.admission, body_bytes);
        let mut body = vec![0_u8; body_bytes];
        let received = self.read_exact_before(&mut body, deadline);
        drop(receiving);
        match received {
            Ok(()) => RequestFrameRead::Body { body, reservation },
            Err(stop) => stop.into(),
        }
    }

    /// 额度记不进的帧：读出帧头拿序号与操作码，帧体余下的读完丢掉。
    fn discard_refused_frame(
        &mut self,
        body_bytes: usize,
        deadline: Instant,
        refusal: String,
    ) -> RequestFrameRead {
        let mut header = vec![0_u8; usize_from_u32(REQUEST_HEADER_BYTES)];
        if let Err(stop) = self.read_exact_before(&mut header, deadline) {
            return stop.into();
        }
        if let Err(stop) = self.discard_before(body_bytes - header.len(), deadline) {
            return stop.into();
        }
        let (sequence, operation_code) = peek_request_header(&header).unwrap_or_default();
        RequestFrameRead::Discarded {
            sequence,
            operation_code,
            refusal,
        }
    }
}
