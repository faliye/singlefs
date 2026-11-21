//! 十六进制名的解析、计划里的中断、接收模式与连接结局、帧接收器与它的计数。

use super::*;

pub(super) fn hexadecimal_name_as_u32(path: &Path) -> Option<u32> {
    if !path.is_dir() {
        return None;
    }
    let name = path.file_name()?.to_str()?;
    if name.len() != 8 {
        return None;
    }
    u32::from_str_radix(name, 16).ok()
}

pub(super) fn parse_hexadecimal(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).ok())
        .collect()
}

// ============================== 中断计划与 PC3-drop 挑帧 ==============================

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PlannedInterruption {
    /// 确认数到 a_k 之后中断（回环格：送块进程报的确认数；跨机格：库进程回出的确认数）。
    pub(super) acknowledgement_threshold: u64,
    /// u_k：停 u_k × 200 ms 再起。
    pause_fraction: f64,
}

/// a_k 在 [1, 网络块数) 里均匀取 count 个、排序，u_k 同源（登记 6.3 末段；网络块数 5 × 10⁴ 时就是登记的 [1, 5 × 10⁴)）：
/// SplitMix64(seed_NET) 每个中断先后取 r1、r2，a = 1 + r1 mod (网络块数 − 1)，u = (r2 >> 11) ÷ 2⁵³；按 a 稳定排序。
pub(super) fn interruption_plan(count: usize, network_blocks: u64) -> Vec<PlannedInterruption> {
    assert!(network_blocks >= 2, "网络块数至少 2 才取得出 [1, 网络块数) 里的中断点");
    let mut generator = SplitMix64::new(SEED_NETWORK_INTERRUPTIONS);
    let mut plan: Vec<PlannedInterruption> = (0..count)
        .map(|_| {
            let threshold_output = generator.next_output();
            let pause_output = generator.next_output();
            PlannedInterruption {
                acknowledgement_threshold: 1 + threshold_output % (network_blocks - 1),
                pause_fraction: (pause_output >> 11) as f64 / (1u64 << 53) as f64,
            }
        })
        .collect();
    plan.sort_by_key(|interruption| interruption.acknowledgement_threshold);
    plan
}

pub(super) fn pause_of(interruption: &PlannedInterruption) -> Duration {
    Duration::from_secs_f64(interruption.pause_fraction * INTERRUPTION_PAUSE_UNIT_MILLISECONDS / 1_000.0)
}

/// PC3-drop：第 w 个 1 000 帧里挑第 SplitMix64(seed_NET) 第 w 个输出 mod 1 000 帧（另起一个生成器，与中断计划不共用状态）。
fn drop_control_picks(window_count: usize) -> Vec<u64> {
    let mut generator = SplitMix64::new(SEED_NETWORK_INTERRUPTIONS);
    (0..window_count).map(|_| generator.next_output() % DROP_CONTROL_FRAMES_PER_PICK).collect()
}

// ============================== 库进程：收块线程 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReceiveMode {
    /// 臂的定义：CRC 不对或半帧就丢、不回确认；完整的帧按块持久提交，提交返回之后回确认（登记 6.3 末段）。
    Registered,
    /// PC3-drop：每 1 000 帧挑一帧只回确认、不写。
    DropOnePerThousand,
    /// PC3-torn：不校验帧 CRC、半帧用 0 补齐照写，且「键已在就跳过不写」。
    TornAccepting,
    /// PC3-rate 慢的那一遍：每帧提交之前睡 5 ms。
    SleepBeforeCommit,
    /// S3-raw：校验 CRC 之后不写库、直接回确认。
    AcknowledgeWithoutWriting,
}

impl ReceiveMode {
    pub(super) fn label(self) -> &'static str {
        match self {
            ReceiveMode::Registered => "registered",
            ReceiveMode::DropOnePerThousand => "drop-one-per-thousand",
            ReceiveMode::TornAccepting => "torn-accepting",
            ReceiveMode::SleepBeforeCommit => "sleep-before-commit",
            ReceiveMode::AcknowledgeWithoutWriting => "acknowledge-without-writing",
        }
    }

    pub(super) fn parse(text: &str) -> Option<ReceiveMode> {
        match text {
            "registered" => Some(ReceiveMode::Registered),
            "drop-one-per-thousand" => Some(ReceiveMode::DropOnePerThousand),
            "torn-accepting" => Some(ReceiveMode::TornAccepting),
            "sleep-before-commit" => Some(ReceiveMode::SleepBeforeCommit),
            "acknowledge-without-writing" => Some(ReceiveMode::AcknowledgeWithoutWriting),
            _ => None,
        }
    }
}

/// 一条连接为什么结束。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConnectionEnd {
    /// 对面在帧边界上关了。
    ClosedAtFrameBoundary,
    /// 读到半帧对面就关了。
    HalfFrame,
    /// 魔数或块值长度不对：流已经对不上帧了，关掉这条连接。
    ProtocolError,
    AcknowledgementWriteFailed,
    /// 跨机格：回出第 a_k 个确认了，该杀数据通道了。
    InterruptionDue,
}

impl ConnectionEnd {
    pub(super) fn label(self) -> &'static str {
        match self {
            ConnectionEnd::ClosedAtFrameBoundary => "closed_at_frame_boundary",
            ConnectionEnd::HalfFrame => "half_frame",
            ConnectionEnd::ProtocolError => "protocol_error",
            ConnectionEnd::AcknowledgementWriteFailed => "acknowledgement_write_failed",
            ConnectionEnd::InterruptionDue => "interruption_due",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ReceiveCounters {
    connections: u64,
    pub(super) complete_frames: u64,
    checksum_failures: u64,
    half_frames_dropped: u64,
    /// PC3-torn：补 0 照写的半帧数。
    half_frames_written: u64,
    protocol_errors: u64,
    commits: u64,
    commit_failures: u64,
    /// 收到的完整帧的键这个收块线程早先已经写进库（重发的帧碰上库里已有的块，Q3h）。
    frames_already_stored: u64,
    /// PC3-torn：键已在、跳过不写的帧数。
    frames_skipped_as_already_stored: u64,
    /// PC3-drop：只回确认、没写的帧数。
    controlled_drops: u64,
    pub(super) acknowledgements_sent: u64,
    acknowledgement_write_failures: u64,
}

pub(super) struct FrameReceiver {
    pub(super) receive_mode: ReceiveMode,
    expected_value_bytes: usize,
    store: ThreadStore,
    /// 这个收块线程写进库的键（库进程不重起，所以就是「库里已有」的网络块）。
    stored_keys: HashSet<BlockKey>,
    drop_picks: Vec<u64>,
    pub(super) counters: ReceiveCounters,
    first_commit_error: String,
}

impl FrameReceiver {
    pub(super) fn new(receive_mode: ReceiveMode, states_per_block: u64, network_blocks: u64, store: ThreadStore) -> FrameReceiver {
        let window_count = usize::try_from(network_blocks / DROP_CONTROL_FRAMES_PER_PICK + 2).expect("窗数装得进 usize");
        FrameReceiver {
            receive_mode,
            expected_value_bytes: usize::try_from(states_per_block).expect("块值字节装得进 usize"),
            store,
            stored_keys: HashSet::new(),
            drop_picks: drop_control_picks(window_count),
            counters: ReceiveCounters::default(),
            first_commit_error: String::new(),
        }
    }

    pub(super) fn commit(&mut self, key: &BlockKey, value: &[u8]) -> bool {
        match self.store.put_block(key, value) {
            Ok(()) => {
                self.counters.commits += 1;
                self.stored_keys.insert(*key);
                true
            }
            Err(error) => {
                self.counters.commit_failures += 1;
                if self.first_commit_error.is_empty() {
                    self.first_commit_error = error.to_string();
                }
                false
            }
        }
    }

    /// PC3-torn 的半帧：键读全了才知道写哪一块；块值用 0 补齐到整块。
    fn write_zero_padded_half_frame(&mut self, header: &[u8], value_prefix: &[u8]) {
        if header.len() < 4 + BLOCK_KEY_BYTES || &header[0..4] != BLOCK_FRAME_MAGIC {
            self.counters.half_frames_dropped += 1;
            return;
        }
        let key: BlockKey = header[4..4 + BLOCK_KEY_BYTES].try_into().expect("32 字节");
        if self.stored_keys.contains(&key) {
            self.counters.half_frames_dropped += 1;
            return;
        }
        let mut padded = vec![0u8; self.expected_value_bytes];
        let copied = value_prefix.len().min(self.expected_value_bytes);
        padded[..copied].copy_from_slice(&value_prefix[..copied]);
        if self.commit(&key, &padded) {
            self.counters.half_frames_written += 1;
        }
    }

    /// 服务一条连接，直到它结束；stop_after_acknowledgements 给了就在累计回出这么多个确认之后返回 InterruptionDue。
    pub(super) fn serve_connection(&mut self, reader: &mut dyn Read, writer: &mut dyn Write, stop_after_acknowledgements: Option<u64>) -> ConnectionEnd {
        self.counters.connections += 1;
        let accepts_torn_frames = self.receive_mode == ReceiveMode::TornAccepting;
        loop {
            let mut header = [0u8; BLOCK_FRAME_HEADER_BYTES];
            let header_bytes = read_until_full_or_end(reader, &mut header);
            if header_bytes == 0 {
                return ConnectionEnd::ClosedAtFrameBoundary;
            }
            if header_bytes < BLOCK_FRAME_HEADER_BYTES {
                if accepts_torn_frames {
                    self.write_zero_padded_half_frame(&header[..header_bytes], &[]);
                } else {
                    self.counters.half_frames_dropped += 1;
                }
                return ConnectionEnd::HalfFrame;
            }
            if &header[0..4] != BLOCK_FRAME_MAGIC {
                self.counters.protocol_errors += 1;
                return ConnectionEnd::ProtocolError;
            }
            let key: BlockKey = header[4..4 + BLOCK_KEY_BYTES].try_into().expect("32 字节");
            let value_length = u32::from_le_bytes(header[4 + BLOCK_KEY_BYTES..].try_into().expect("4 字节"));
            if usize::try_from(value_length).ok() != Some(self.expected_value_bytes) {
                self.counters.protocol_errors += 1;
                return ConnectionEnd::ProtocolError;
            }
            let mut rest = vec![0u8; self.expected_value_bytes + FRAME_CHECKSUM_BYTES];
            let rest_bytes = read_until_full_or_end(reader, &mut rest);
            if rest_bytes < rest.len() {
                if accepts_torn_frames {
                    let value_prefix = &rest[..rest_bytes.min(self.expected_value_bytes)];
                    self.write_zero_padded_half_frame(&header, value_prefix);
                } else {
                    self.counters.half_frames_dropped += 1;
                }
                return ConnectionEnd::HalfFrame;
            }
            self.counters.complete_frames += 1;
            let (value, checksum_bytes) = rest.split_at(self.expected_value_bytes);
            let stored_checksum = u32::from_le_bytes(checksum_bytes.try_into().expect("4 字节"));
            if !accepts_torn_frames && block_frame_checksum(&key, value_length, value) != stored_checksum {
                self.counters.checksum_failures += 1;
                continue;
            }
            let is_already_stored = self.stored_keys.contains(&key);
            if is_already_stored {
                self.counters.frames_already_stored += 1;
            }
            let is_confirmable = if accepts_torn_frames && is_already_stored {
                self.counters.frames_skipped_as_already_stored += 1;
                true
            } else {
                match self.receive_mode {
                    ReceiveMode::Registered | ReceiveMode::TornAccepting => self.commit(&key, value),
                    ReceiveMode::SleepBeforeCommit => {
                        std::thread::sleep(RATE_CONTROL_SLEEP);
                        self.commit(&key, value)
                    }
                    ReceiveMode::AcknowledgeWithoutWriting => true,
                    ReceiveMode::DropOnePerThousand => {
                        let valid_frame_index = self.counters.complete_frames - self.counters.checksum_failures - 1;
                        let window = usize::try_from(valid_frame_index / DROP_CONTROL_FRAMES_PER_PICK).expect("窗号装得进 usize");
                        let picked = self.drop_picks.get(window).copied();
                        if picked == Some(valid_frame_index % DROP_CONTROL_FRAMES_PER_PICK) {
                            self.counters.controlled_drops += 1;
                            true
                        } else {
                            self.commit(&key, value)
                        }
                    }
                }
            };
            if !is_confirmable {
                continue;
            }
            if writer.write_all(&encode_acknowledgement_frame(&key)).and_then(|()| writer.flush()).is_err() {
                self.counters.acknowledgement_write_failures += 1;
                return ConnectionEnd::AcknowledgementWriteFailed;
            }
            self.counters.acknowledgements_sent += 1;
            if stop_after_acknowledgements.is_some_and(|threshold| self.counters.acknowledgements_sent >= threshold) {
                return ConnectionEnd::InterruptionDue;
            }
        }
    }

    pub(super) fn summary_line(&self, connection_ends: &BTreeSet<String>) -> String {
        let counters = &self.counters;
        format!(
            "N receive_mode={} connections={} complete_frames={} checksum_failures={} half_frames_dropped={} half_frames_written={} protocol_errors={} commits={} commit_failures={} frames_already_stored={} frames_skipped_as_already_stored={} controlled_drops={} acknowledgements_sent={} acknowledgement_write_failures={} connection_ends={} first_commit_error={}",
            self.receive_mode.label(),
            counters.connections,
            counters.complete_frames,
            counters.checksum_failures,
            counters.half_frames_dropped,
            counters.half_frames_written,
            counters.protocol_errors,
            counters.commits,
            counters.commit_failures,
            counters.frames_already_stored,
            counters.frames_skipped_as_already_stored,
            counters.controlled_drops,
            counters.acknowledgements_sent,
            counters.acknowledgement_write_failures,
            if connection_ends.is_empty() { "none".to_string() } else { connection_ends.iter().cloned().collect::<Vec<_>>().join(",") },
            text_field(&self.first_commit_error),
        )
    }
}

#[cfg(test)]
#[path = "interruptions_and_frame_receiving_tests.rs"]
mod tests;
