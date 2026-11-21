//! 持久标记的编码与写入循环、设备日志的解析与按写缓存模型的重放。

use super::*;

// ============================== 标记（补 5.2「标记」、补 7 B7） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MarkerKind {
    Start,
    Begin,
    Confirm,
    End,
}

impl MarkerKind {
    fn byte(self) -> u8 {
        match self {
            MarkerKind::Start => b'S',
            MarkerKind::Begin => b'B',
            MarkerKind::Confirm => b'C',
            MarkerKind::End => b'E',
        }
    }

    fn from_byte(byte: u8) -> Option<MarkerKind> {
        [MarkerKind::Start, MarkerKind::Begin, MarkerKind::Confirm, MarkerKind::End]
            .into_iter()
            .find(|kind| kind.byte() == byte)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Marker {
    pub(super) kind: MarkerKind,
    pub(super) sequence: u64,
    /// S、E 写全 1（None）。
    pub(super) block_index: Option<u64>,
    pub(super) arm_index: u32,
}

pub(super) fn encode_marker(marker: &Marker) -> [u8; SECTOR_BYTES] {
    let mut sector = [0u8; SECTOR_BYTES];
    sector[0..4].copy_from_slice(MARKER_MAGIC);
    sector[4] = marker.kind.byte();
    sector[8..16].copy_from_slice(&marker.sequence.to_le_bytes());
    sector[16..24].copy_from_slice(&marker.block_index.unwrap_or(MARKER_WITHOUT_BLOCK).to_le_bytes());
    sector[24..28].copy_from_slice(&marker.arm_index.to_le_bytes());
    let checksum = crc32c(&[&sector[..MARKER_CHECKSUMMED_BYTES]]);
    sector[28..32].copy_from_slice(&checksum.to_le_bytes());
    sector
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum MarkerDecoding {
    Valid(Marker),
    NotAMarker,
    ChecksumMismatch,
    Malformed(String),
}

pub(super) fn decode_marker(sector: &[u8]) -> MarkerDecoding {
    if sector.len() != SECTOR_BYTES || &sector[0..4] != MARKER_MAGIC {
        return MarkerDecoding::NotAMarker;
    }
    let stored_checksum = u32::from_le_bytes(sector[28..32].try_into().expect("4 字节"));
    if crc32c(&[&sector[..MARKER_CHECKSUMMED_BYTES]]) != stored_checksum {
        return MarkerDecoding::ChecksumMismatch;
    }
    let Some(kind) = MarkerKind::from_byte(sector[4]) else {
        return MarkerDecoding::Malformed(format!("种类字节 {:#04x}", sector[4]));
    };
    if sector[5..8].iter().any(|&byte| byte != 0) || sector[32..].iter().any(|&byte| byte != 0) {
        return MarkerDecoding::Malformed("该是 0 的字节不是 0".to_string());
    }
    let sequence = u64::from_le_bytes(sector[8..16].try_into().expect("8 字节"));
    let raw_block = u64::from_le_bytes(sector[16..24].try_into().expect("8 字节"));
    let arm_index = u32::from_le_bytes(sector[24..28].try_into().expect("4 字节"));
    let block_index = match (kind, raw_block == MARKER_WITHOUT_BLOCK) {
        (MarkerKind::Start | MarkerKind::End, true) => None,
        (MarkerKind::Begin | MarkerKind::Confirm, false) => Some(raw_block),
        (MarkerKind::Start | MarkerKind::End, false) | (MarkerKind::Begin | MarkerKind::Confirm, true) => {
            return MarkerDecoding::Malformed(format!("种类 {:?} 配块号 {raw_block}", kind));
        }
    };
    MarkerDecoding::Valid(Marker { kind, sequence, block_index, arm_index })
}

/// O_DIRECT 要按 4 KiB 对齐的缓冲；从一段稍大的 Vec 里切出对齐的 512 字节。
pub(super) struct AlignedSector {
    storage: Vec<u8>,
    aligned_offset: usize,
}

impl AlignedSector {
    pub(super) fn new() -> AlignedSector {
        let storage = vec![0u8; SECTOR_BYTES + DIRECT_WRITE_ALIGNMENT_BYTES];
        let aligned_offset = storage.as_ptr().align_offset(DIRECT_WRITE_ALIGNMENT_BYTES);
        assert!(aligned_offset + SECTOR_BYTES <= storage.len(), "对齐之后还放得下一个扇区");
        AlignedSector { storage, aligned_offset }
    }

    pub(super) fn fill(&mut self, sector: &[u8; SECTOR_BYTES]) -> &[u8] {
        let aligned = &mut self.storage[self.aligned_offset..self.aligned_offset + SECTOR_BYTES];
        aligned.copy_from_slice(sector);
        aligned
    }
}

/// 单测里记下的写入循环事件次序（补 9 M21）。
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WriteLoopEvent {
    Marker(Marker),
    WriteCallStarted { block_index: u64 },
    WriteCallReturned { block_index: u64 },
}

/// 标记发到哪里：来宾里写进 vda2 的第 c 个扇区（O_DIRECT）；单测里记下事件次序（补 9 M21，只在单测里有）。
pub(super) enum MarkerSink {
    MarkerPartition { device: File, aligned: AlignedSector, next_sequence: u64, arm_index: u32 },
    #[cfg(test)]
    Recording { events: Vec<WriteLoopEvent>, next_sequence: u64, arm_index: u32 },
}

impl MarkerSink {
    pub(super) fn open_marker_partition(path: &str, arm_index: u32) -> Result<MarkerSink, String> {
        let device = OpenOptions::new()
            .write(true)
            .custom_flags(OPEN_DIRECT_FLAG)
            .open(path)
            .map_err(|error| format!("以 O_DIRECT 打不开 {path}：{error}"))?;
        Ok(MarkerSink::MarkerPartition { device, aligned: AlignedSector::new(), next_sequence: 0, arm_index })
    }

    #[cfg(test)]
    pub(super) fn recording(arm_index: u32) -> MarkerSink {
        MarkerSink::Recording { events: Vec::new(), next_sequence: 0, arm_index }
    }

    fn emit(&mut self, kind: MarkerKind, block_index: Option<u64>) -> Result<(), String> {
        match self {
            MarkerSink::MarkerPartition { device, aligned, next_sequence, arm_index } => {
                let marker = Marker { kind, sequence: *next_sequence, block_index, arm_index: *arm_index };
                let sector = encode_marker(&marker);
                let offset = *next_sequence * sector_bytes();
                device
                    .write_all_at(aligned.fill(&sector), offset)
                    .map_err(|error| format!("第 {} 个标记写不进：{error}", *next_sequence))?;
                *next_sequence += 1;
                Ok(())
            }
            #[cfg(test)]
            MarkerSink::Recording { events, next_sequence, arm_index } => {
                events.push(WriteLoopEvent::Marker(Marker { kind, sequence: *next_sequence, block_index, arm_index: *arm_index }));
                *next_sequence += 1;
                Ok(())
            }
        }
    }

    fn note_write_call_started(&mut self, block_index: u64) {
        match self {
            MarkerSink::MarkerPartition { .. } => {
                let _marker_partition_does_not_record = block_index;
            }
            #[cfg(test)]
            MarkerSink::Recording { events, .. } => events.push(WriteLoopEvent::WriteCallStarted { block_index }),
        }
    }

    fn note_write_call_returned(&mut self, block_index: u64) {
        match self {
            MarkerSink::MarkerPartition { .. } => {
                let _marker_partition_does_not_record = block_index;
            }
            #[cfg(test)]
            MarkerSink::Recording { events, .. } => events.push(WriteLoopEvent::WriteCallReturned { block_index }),
        }
    }

    pub(super) fn emitted_marker_count(&self) -> u64 {
        match self {
            MarkerSink::MarkerPartition { next_sequence, .. } => *next_sequence,
            #[cfg(test)]
            MarkerSink::Recording { next_sequence, .. } => *next_sequence,
        }
    }
}

/// 写块的那一个线程：开始写之前一个 S；每块写入调用之前一个 B，返回 Ok 之后一个 C；全部写完一个 E。返回已确认的块数。
pub(super) fn write_blocks_with_markers(store: &mut BlockStore, sink: &mut MarkerSink, block_count: u64, states_per_block: u64) -> Result<u64, String> {
    let fingerprint = input_fingerprint();
    sink.emit(MarkerKind::Start, None)?;
    let mut confirmed_blocks = 0u64;
    for block_index in 0..block_count {
        let key = interleaved_block_key(block_index, states_per_block, &fingerprint);
        let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, block_index, states_per_block);
        sink.emit(MarkerKind::Begin, Some(block_index))?;
        sink.note_write_call_started(block_index);
        let write_outcome = store.put_block(&key, &value);
        sink.note_write_call_returned(block_index);
        write_outcome.map_err(|error| format!("第 {block_index} 块写入失败：{error}"))?;
        sink.emit(MarkerKind::Confirm, Some(block_index))?;
        confirmed_blocks += 1;
    }
    sink.emit(MarkerKind::End, None)?;
    Ok(confirmed_blocks)
}

// ============================== 设备侧日志（dm-log-writes 格式，补 5.2「设备侧日志的读法」） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LogEntry {
    Write { first_sector: u64, sector_count: u64, is_forced_unit_access: bool, data_offset_in_log: u64 },
    Flush,
    Discard { first_sector: u64, sector_count: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DeviceLog {
    /// 系统配置声明的条目数；系统配置全 0 时是 None。
    pub(super) declared_entry_count: Option<u64>,
    pub(super) entries: Vec<LogEntry>,
    /// 最后一个条目（连同数据）之后的字节偏移：「截到最后一个条目」截到这里。
    pub(super) end_offset_in_log: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DeviceLogError {
    BadMagic { found: u64 },
    UnknownVersion { found: u64 },
    UnsupportedSectorSize { found: u64 },
    TruncatedEntry { index: usize },
    UnknownFlags { index: usize, flags: u64 },
    UnsupportedFlagCombination { index: usize, flags: u64, sector_count: u64 },
    ReadFailed(String),
}

impl std::fmt::Display for DeviceLogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// 条目数怎么数：照系统配置声明的封顶（判定用），或读到全 0 的条目头为止（只给 H9 的诊断行用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeclaredCountHandling {
    StopAtDeclared,
    ReadUntilZeroHeader,
}

/// 日志字节在哪：单测与锚点在内存里，宿主与来宾在文件里（按偏移读，不整份读进内存）。
pub(super) enum LogBytes {
    InMemory(Vec<u8>),
    OnDisk { file: File, length: u64 },
}

impl LogBytes {
    pub(super) fn open(path: &Path) -> Result<LogBytes, String> {
        let file = File::open(path).map_err(|error| format!("打不开日志 {}：{error}", path.display()))?;
        let length = file.metadata().map_err(|error| format!("读不到日志长度：{error}"))?.len();
        Ok(LogBytes::OnDisk { file, length })
    }

    fn length(&self) -> u64 {
        match self {
            LogBytes::InMemory(bytes) => u64::try_from(bytes.len()).expect("装得进 u64"),
            LogBytes::OnDisk { length, .. } => *length,
        }
    }

    pub(super) fn read_exact_at(&self, buffer: &mut [u8], offset: u64) -> Result<(), String> {
        match self {
            LogBytes::InMemory(bytes) => {
                let start = usize::try_from(offset).expect("偏移装得进 usize");
                let end = start + buffer.len();
                if end > bytes.len() {
                    return Err(format!("读 [{start}, {end}) 越过日志末尾 {}", bytes.len()));
                }
                buffer.copy_from_slice(&bytes[start..end]);
                Ok(())
            }
            LogBytes::OnDisk { file, .. } => file.read_exact_at(buffer, offset).map_err(|error| format!("读日志偏移 {offset} 失败：{error}")),
        }
    }
}

pub(super) fn parse_device_log(bytes: &LogBytes, handling: DeclaredCountHandling) -> Result<DeviceLog, DeviceLogError> {
    let length = bytes.length();
    let mut super_bytes = [0u8; LOG_SUPER_BYTES];
    let declared_entry_count = if length >= u64::try_from(LOG_SUPER_BYTES).expect("28") {
        bytes.read_exact_at(&mut super_bytes, 0).map_err(DeviceLogError::ReadFailed)?;
        if super_bytes.iter().all(|&byte| byte == 0) {
            None
        } else {
            let magic = u64::from_le_bytes(super_bytes[0..8].try_into().expect("8 字节"));
            if magic != WRITE_LOG_MAGIC {
                return Err(DeviceLogError::BadMagic { found: magic });
            }
            let version = u64::from_le_bytes(super_bytes[8..16].try_into().expect("8 字节"));
            if version != WRITE_LOG_VERSION {
                return Err(DeviceLogError::UnknownVersion { found: version });
            }
            let declared = u64::from_le_bytes(super_bytes[16..24].try_into().expect("8 字节"));
            let sector_size = u64::from(u32::from_le_bytes(super_bytes[24..28].try_into().expect("4 字节")));
            if sector_size != sector_bytes() {
                return Err(DeviceLogError::UnsupportedSectorSize { found: sector_size });
            }
            Some(declared)
        }
    } else {
        None
    };
    let header_bytes = u64::try_from(LOG_ENTRY_HEADER_BYTES).expect("32");
    let known_flags = LOG_FLUSH_FLAG | LOG_FUA_FLAG | LOG_DISCARD_FLAG | LOG_MARK_FLAG | LOG_METADATA_FLAG;
    let mut entries = Vec::new();
    let mut offset = sector_bytes();
    loop {
        if offset + header_bytes > length {
            break;
        }
        let mut header = [0u8; LOG_ENTRY_HEADER_BYTES];
        bytes.read_exact_at(&mut header, offset).map_err(DeviceLogError::ReadFailed)?;
        if header.iter().all(|&byte| byte == 0) {
            break;
        }
        let parsed_count = u64::try_from(entries.len()).expect("条目数装得进 u64");
        if handling == DeclaredCountHandling::StopAtDeclared && declared_entry_count.is_some_and(|declared| parsed_count >= declared) {
            break;
        }
        let index = entries.len();
        let first_sector = u64::from_le_bytes(header[0..8].try_into().expect("8 字节"));
        let sector_count = u64::from_le_bytes(header[8..16].try_into().expect("8 字节"));
        let flags = u64::from_le_bytes(header[16..24].try_into().expect("8 字节"));
        if flags & !known_flags != 0 || flags & (LOG_MARK_FLAG | LOG_METADATA_FLAG) != 0 {
            return Err(DeviceLogError::UnknownFlags { index, flags });
        }
        offset += sector_bytes();
        if flags & LOG_DISCARD_FLAG != 0 {
            if flags != LOG_DISCARD_FLAG {
                return Err(DeviceLogError::UnsupportedFlagCombination { index, flags, sector_count });
            }
            entries.push(LogEntry::Discard { first_sector, sector_count });
            continue;
        }
        if flags & LOG_FLUSH_FLAG != 0 {
            if sector_count != 0 || flags != LOG_FLUSH_FLAG {
                return Err(DeviceLogError::UnsupportedFlagCombination { index, flags, sector_count });
            }
            entries.push(LogEntry::Flush);
            continue;
        }
        if sector_count == 0 {
            return Err(DeviceLogError::UnsupportedFlagCombination { index, flags, sector_count });
        }
        let data_bytes = sector_count * sector_bytes();
        if offset + data_bytes > length {
            return Err(DeviceLogError::TruncatedEntry { index });
        }
        entries.push(LogEntry::Write {
            first_sector,
            sector_count,
            is_forced_unit_access: flags & LOG_FUA_FLAG != 0,
            data_offset_in_log: offset,
        });
        offset += data_bytes;
    }
    Ok(DeviceLog { declared_entry_count, entries, end_offset_in_log: offset })
}

/// 合成一份日志（锚点 B8 与单测用）：系统配置声明 declared 个条目，条目依次是（起始扇区，扇区数，标志，数据）。
pub(super) fn synthetic_log_bytes(entries: &[(u64, u64, u64, Vec<u8>)], declared: u64) -> Vec<u8> {
    let mut log = Vec::new();
    log.extend_from_slice(&WRITE_LOG_MAGIC.to_le_bytes());
    log.extend_from_slice(&WRITE_LOG_VERSION.to_le_bytes());
    log.extend_from_slice(&declared.to_le_bytes());
    log.extend_from_slice(&u32::try_from(SECTOR_BYTES).expect("512").to_le_bytes());
    log.resize(SECTOR_BYTES, 0);
    for (first_sector, sector_count, flags, data) in entries {
        let header_start = log.len();
        log.extend_from_slice(&first_sector.to_le_bytes());
        log.extend_from_slice(&sector_count.to_le_bytes());
        log.extend_from_slice(&flags.to_le_bytes());
        log.extend_from_slice(&0u64.to_le_bytes());
        log.resize(header_start + SECTOR_BYTES, 0);
        log.extend_from_slice(data);
    }
    log
}

// ============================== 断电盘面的四种取法（补 5.2、补 7 B3、B8） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReplayVariant {
    /// U-drop（判定）：最后一个 FLUSH 之前的写全落，U(k) 全丢。
    DropUnflushed,
    /// U-keep：前 k 个条目的写全落。
    KeepUnflushed,
    /// U-rand：U(k) 里每个写按 B3 的生成器各自留或丢。
    RandomSubset,
    /// U-tear：同 U-rand，另把留下的某一个多扇区写截成前 t 个扇区。
    RandomSubsetWithTear,
}

pub(super) const ALL_REPLAY_VARIANTS: [ReplayVariant; 4] =
    [ReplayVariant::DropUnflushed, ReplayVariant::KeepUnflushed, ReplayVariant::RandomSubset, ReplayVariant::RandomSubsetWithTear];

impl ReplayVariant {
    pub(super) fn label(self) -> &'static str {
        match self {
            ReplayVariant::DropUnflushed => "U-drop",
            ReplayVariant::KeepUnflushed => "U-keep",
            ReplayVariant::RandomSubset => "U-rand",
            ReplayVariant::RandomSubsetWithTear => "U-tear",
        }
    }

    pub(super) fn parse(text: &str) -> Option<ReplayVariant> {
        ALL_REPLAY_VARIANTS.into_iter().find(|variant| variant.label() == text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TornWrite {
    pub(super) position_in_unflushed: usize,
    pub(super) sectors_kept: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UnflushedSubset {
    pub(super) kept: Vec<bool>,
    pub(super) torn: Option<TornWrite>,
}

/// 补 7 B3：状态 = seed_SUB XOR (((a << 32) | i) × 乘子)；U(k) 里每个写按日志次序取一个输出、最高位为 1 留；
/// 再在留下的多扇区写里取第 `输出 mod 个数` 个，截成前 `1 + 输出 mod (扇区数 − 1)` 个扇区。
pub(super) fn unflushed_subset(arm_index: u64, subset_index: u64, unflushed_sector_counts: &[u64]) -> UnflushedSubset {
    let mut generator = SplitMix64::new(SEED_SUBSET ^ ((arm_index << 32) | subset_index).wrapping_mul(MIXING_MULTIPLIER));
    let kept: Vec<bool> = unflushed_sector_counts.iter().map(|_sector_count| generator.next_output() >> 63 == 1).collect();
    let multi_sector_kept: Vec<usize> = kept
        .iter()
        .zip(unflushed_sector_counts)
        .enumerate()
        .filter(|(_position, (is_kept, sector_count))| **is_kept && **sector_count >= 2)
        .map(|(position, _pair)| position)
        .collect();
    let torn = if multi_sector_kept.is_empty() {
        None
    } else {
        let choices = u64::try_from(multi_sector_kept.len()).expect("装得进 u64");
        let chosen = multi_sector_kept[usize::try_from(generator.next_output() % choices).expect("小于个数")];
        let sectors_kept = 1 + generator.next_output() % (unflushed_sector_counts[chosen] - 1);
        Some(TornWrite { position_in_unflushed: chosen, sectors_kept })
    };
    UnflushedSubset { kept, torn }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ReplayedWrite {
    pub(super) first_sector: u64,
    sectors_applied: u64,
    data_offset_in_log: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReplaySelection {
    last_flush_entry: Option<usize>,
    /// U(k)：最后一个 FLUSH 之后、k 之前、不带 FUA 的写条目下标。
    pub(super) unflushed_entries: Vec<usize>,
    subset: UnflushedSubset,
    pub(super) writes: Vec<ReplayedWrite>,
}

/// 断电点 point（日志前 point 个条目已送到虚拟盘）上按 variant 取的写，按日志次序（后写的盖前写的）。
/// 带 FUA 的写只算它自己收到即持久，不进 U(k)。
pub(super) fn select_replayed_writes(entries: &[LogEntry], point: usize, variant: ReplayVariant, arm_index: u64, subset_index: u64) -> ReplaySelection {
    assert!(point <= entries.len(), "断电点 {point} 越过条目数 {}", entries.len());
    let last_flush_entry = entries[..point].iter().rposition(|entry| *entry == LogEntry::Flush);
    let mut unflushed_entries = Vec::new();
    let mut unflushed_sector_counts = Vec::new();
    for (entry_index, entry) in entries[..point].iter().enumerate() {
        if let LogEntry::Write { sector_count, is_forced_unit_access: false, .. } = *entry {
            if last_flush_entry.is_none_or(|flush| entry_index > flush) {
                unflushed_entries.push(entry_index);
                unflushed_sector_counts.push(sector_count);
            }
        }
    }
    let subset = unflushed_subset(arm_index, subset_index, &unflushed_sector_counts);
    let mut writes = Vec::new();
    let mut unflushed_position = 0usize;
    for (entry_index, entry) in entries[..point].iter().enumerate() {
        let LogEntry::Write { first_sector, sector_count, is_forced_unit_access, data_offset_in_log } = *entry else {
            continue;
        };
        let is_unflushed = last_flush_entry.is_none_or(|flush| entry_index > flush) && !is_forced_unit_access;
        let sectors_applied = if is_unflushed {
            let position = unflushed_position;
            unflushed_position += 1;
            let is_kept = match variant {
                ReplayVariant::DropUnflushed => false,
                ReplayVariant::KeepUnflushed => true,
                ReplayVariant::RandomSubset | ReplayVariant::RandomSubsetWithTear => subset.kept[position],
            };
            if !is_kept {
                continue;
            }
            let torn_here = variant == ReplayVariant::RandomSubsetWithTear
                && subset.torn.is_some_and(|torn| torn.position_in_unflushed == position);
            if torn_here {
                subset.torn.map_or(sector_count, |torn| torn.sectors_kept)
            } else {
                sector_count
            }
        } else {
            sector_count
        };
        writes.push(ReplayedWrite { first_sector, sectors_applied, data_offset_in_log });
    }
    ReplaySelection { last_flush_entry, unflushed_entries, subset, writes }
}

/// U(k) 里有几个写（Q4u 用）：从 point 往回数到最后一个 FLUSH。
pub(super) fn unflushed_write_count(entries: &[LogEntry], point: usize) -> usize {
    let mut count = 0usize;
    for entry in entries[..point].iter().rev() {
        match entry {
            LogEntry::Flush => break,
            LogEntry::Write { is_forced_unit_access: false, .. } => count += 1,
            LogEntry::Write { is_forced_unit_access: true, .. } | LogEntry::Discard { .. } => {}
        }
    }
    count
}

/// 盘面写到哪：锚点与单测在内存里，宿主与来宾在（稀疏）文件里。
pub(super) enum ReplayImage {
    InMemory(Vec<u8>),
    OnDisk(File),
}

pub(super) fn apply_replayed_writes(log: &LogBytes, writes: &[ReplayedWrite], image: &mut ReplayImage) -> Result<(), String> {
    let mut buffer = Vec::new();
    for write in writes {
        let byte_count = usize::try_from(write.sectors_applied * sector_bytes()).expect("装得进 usize");
        buffer.resize(byte_count, 0);
        log.read_exact_at(&mut buffer, write.data_offset_in_log)?;
        let image_offset = write.first_sector * sector_bytes();
        match image {
            ReplayImage::InMemory(bytes) => {
                let start = usize::try_from(image_offset).expect("装得进 usize");
                let end = start + byte_count;
                if end > bytes.len() {
                    return Err(format!("写 [{start}, {end}) 越过镜像末尾 {}", bytes.len()));
                }
                bytes[start..end].copy_from_slice(&buffer);
            }
            ReplayImage::OnDisk(file) => file
                .write_all_at(&buffer, image_offset)
                .map_err(|error| format!("镜像偏移 {image_offset} 写不进：{error}"))?,
        }
    }
    Ok(())
}

/// 只取镜像里 [first_sector, first_sector + sector_count) 这一截（PC4-C 与标记核对用，不造整盘）。
fn replayed_sector_range(log: &LogBytes, writes: &[ReplayedWrite], first_sector: u64, sector_count: u64) -> Result<Vec<u8>, String> {
    let range_end = first_sector + sector_count;
    let mut window = vec![0u8; usize::try_from(sector_count * sector_bytes()).expect("装得进 usize")];
    let mut sector = [0u8; SECTOR_BYTES];
    for write in writes {
        let write_end = write.first_sector + write.sectors_applied;
        let overlap_start = write.first_sector.max(first_sector);
        let overlap_end = write_end.min(range_end);
        for disk_sector in overlap_start..overlap_end {
            let offset_in_write = (disk_sector - write.first_sector) * sector_bytes();
            log.read_exact_at(&mut sector, write.data_offset_in_log + offset_in_write)?;
            let window_start = usize::try_from((disk_sector - first_sector) * sector_bytes()).expect("装得进 usize");
            window[window_start..window_start + SECTOR_BYTES].copy_from_slice(&sector);
        }
    }
    Ok(window)
}

// ============================== PC4-C（补 5.2、补 7 B4；V17） ==============================

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WriteCacheModelJudgement {
    pub(super) point: usize,
    pub(super) entries_between_batches: usize,
    pub(super) flushes_between_batches: usize,
    /// 每种取法：（前 100 个丢几个，后 100 个丢几个）。
    pub(super) lost_by_variant: Vec<(ReplayVariant, u64, u64)>,
    pub(super) registered_counts_hold: bool,
}

pub(super) fn judge_write_cache_model(log: &LogBytes, device_log: &DeviceLog) -> Result<WriteCacheModelJudgement, String> {
    let marker_count = 2 * WRITE_CACHE_MODEL_BATCH_WRITES;
    let range_end = WRITE_CACHE_MODEL_FIRST_SECTOR + marker_count;
    let mut marker_entries: Vec<Option<usize>> = vec![None; usize::try_from(marker_count).expect("200")];
    for (entry_index, entry) in device_log.entries.iter().enumerate() {
        let LogEntry::Write { first_sector, sector_count, .. } = *entry else { continue };
        for disk_sector in first_sector.max(WRITE_CACHE_MODEL_FIRST_SECTOR)..(first_sector + sector_count).min(range_end) {
            let slot = usize::try_from(disk_sector - WRITE_CACHE_MODEL_FIRST_SECTOR).expect("小于 200");
            if marker_entries[slot].is_none() {
                marker_entries[slot] = Some(entry_index);
            }
        }
    }
    let Some(entries_of_markers) = marker_entries.into_iter().collect::<Option<Vec<usize>>>() else {
        return Err("日志里找不齐 PC4-C 的 200 个扇区写".to_string());
    };
    let batch = usize::try_from(WRITE_CACHE_MODEL_BATCH_WRITES).expect("100");
    let last_of_first_batch = entries_of_markers[..batch].iter().copied().max().expect("100 个");
    let first_of_second_batch = entries_of_markers[batch..].iter().copied().min().expect("100 个");
    let point = entries_of_markers.iter().copied().max().expect("200 个") + 1;
    let (entries_between_batches, flushes_between_batches) = if first_of_second_batch > last_of_first_batch {
        let between = &device_log.entries[last_of_first_batch + 1..first_of_second_batch];
        (between.len(), between.iter().filter(|entry| **entry == LogEntry::Flush).count())
    } else {
        (0, 0)
    };
    let arm_index = u64::from(PowerCutArm::WriteCacheModel.index());
    let mut lost_by_variant = Vec::new();
    for variant in ALL_REPLAY_VARIANTS {
        let selection = select_replayed_writes(&device_log.entries, point, variant, arm_index, 0);
        let window = replayed_sector_range(log, &selection.writes, WRITE_CACHE_MODEL_FIRST_SECTOR, marker_count)?;
        let mut first_lost = 0u64;
        let mut second_lost = 0u64;
        for position in 0..marker_count {
            let start = usize::try_from(position * sector_bytes()).expect("装得进 usize");
            let survived = matches!(
                decode_marker(&window[start..start + SECTOR_BYTES]),
                MarkerDecoding::Valid(Marker { kind: MarkerKind::Confirm, sequence, .. }) if sequence == position
            );
            if !survived {
                if position < WRITE_CACHE_MODEL_BATCH_WRITES {
                    first_lost += 1;
                } else {
                    second_lost += 1;
                }
            }
        }
        lost_by_variant.push((variant, first_lost, second_lost));
    }
    let registered_counts_hold = entries_between_batches == 1
        && flushes_between_batches == 1
        && lost_by_variant.iter().all(|(variant, first_lost, second_lost)| {
            *first_lost == 0
                && match variant {
                    ReplayVariant::DropUnflushed => *second_lost == WRITE_CACHE_MODEL_BATCH_WRITES,
                    ReplayVariant::KeepUnflushed => *second_lost == 0,
                    ReplayVariant::RandomSubset => *second_lost == WRITE_CACHE_MODEL_RANDOM_SUBSET_LOST,
                    ReplayVariant::RandomSubsetWithTear => true,
                }
        });
    Ok(WriteCacheModelJudgement { point, entries_between_batches, flushes_between_batches, lost_by_variant, registered_counts_hold })
}

/// 照 PC4-C 的写流合成日志：前 100 个 C 标记、一个 FLUSH、后 100 个 C 标记（单测与补 9 M16、M17 用）。
pub(super) fn synthetic_write_cache_model_log() -> Vec<u8> {
    let mut entries = Vec::new();
    for position in 0..2 * WRITE_CACHE_MODEL_BATCH_WRITES {
        if position == WRITE_CACHE_MODEL_BATCH_WRITES {
            entries.push((0, 0, LOG_FLUSH_FLAG, Vec::new()));
        }
        let marker = Marker {
            kind: MarkerKind::Confirm,
            sequence: position,
            block_index: Some(position),
            arm_index: PowerCutArm::WriteCacheModel.index(),
        };
        entries.push((WRITE_CACHE_MODEL_FIRST_SECTOR + position, 1, 0, encode_marker(&marker).to_vec()));
    }
    let declared = u64::try_from(entries.len()).expect("装得进 u64");
    synthetic_log_bytes(&entries, declared)
}
