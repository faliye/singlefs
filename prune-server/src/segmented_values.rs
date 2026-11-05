//! 大值分段：一把键的值长过一帧装的上限时分几把键存，读回拼成整值、核 sha256，按前缀扫时段键与计数键不当条目。
//! 格式只有一份说明：`scripts/prune-server/prune_server_client.py` 文件头「键空间」里「大值分段」那一段（清单、段键、计数键、写读扫的次序）；
//! 这里照它逐字节相同地写、读、扫，常量与它同名，不另立第二份说明。容错客户端经它写读（`reliable_client` 的
//! `write_segments_of_large_values`、`read_assembling_segmented_values`），按前缀扫的一方把扫回来的原样喂给 [`SegmentedScanAssembler`]。
//!
//! 与那一段只差认段键的办法：Python 三家的原键是 UTF-8 路径、不带 NUL，那一侧拒一切带 NUL 的原键、扫时凡带 NUL 的键都不当条目；
//! 崩溃判定库（`crates/singlefs-verification/src/verdict_store.rs`）的键是二进制、表标签开头，常带 NUL，照那一侧的认法一把都写不进、扫不出。
//! 这里改认段键与计数键的形状（键尾是 NUL 加 [`SEGMENT_INDEX_DIGITS`] 位十进制，或 NUL 加 [`SEGMENT_COUNT_SUFFIX`]）：写时拒这两种形状的原键
//! （[`SegmentedValueFailure::ReservedKeyShape`]），扫时只跳过这两种形状的键。不带 NUL 的键两种认法逐个相同，两边存下的字节、读回的值一样。

use std::collections::BTreeMap;
use std::time::Duration;

use crate::protocol::Table;
use crate::sha256::sha256_hexadecimal;

/// 一段至多这么多字节；值长过它才分段（Python `SEGMENT_BYTES`，等于那一侧一帧装的上限 `FRAME_BATCH_BYTES`）。
pub const SEGMENT_BYTES: usize = 16 * 1024 * 1024;
/// 段键、计数键里原键后面接的那个字节（Python `SEGMENT_KEY_SEPARATOR`）。
pub const SEGMENT_KEY_SEPARATOR: u8 = 0x00;
/// 段号补零到这么多位十进制（Python `SEGMENT_INDEX_DIGITS`）。
pub const SEGMENT_INDEX_DIGITS: usize = 6;
/// 计数键里接在分隔字节后面的那一段（Python `SEGMENT_COUNT_SUFFIX`）。
pub const SEGMENT_COUNT_SUFFIX: &[u8] = b"count";
/// 分段清单的开头（Python `SEGMENT_MANIFEST_MAGIC`）。
pub const SEGMENT_MANIFEST_MAGIC: &[u8] = b"\x00singlefs-prune-server-segments\n";
/// 一把键至多分这么多段：[`SEGMENT_INDEX_DIGITS`] 位十进制装得下的段号个数。Python 那一侧不设这一条，段号到七位时段键认不出来；
/// 这里写之前拒（[`SegmentedValueFailure::TooManySegmentsForTheIndexDigits`]），一条都不写。
const SEGMENT_INDEX_LIMIT: usize = {
    let mut limit = 1usize;
    let mut digit = 0;
    while digit < SEGMENT_INDEX_DIGITS {
        limit *= 10;
        digit += 1;
    }
    limit
};
/// 读段时一帧至多装这么多字节（Python `assembled` 照 `FRAME_BATCH_BYTES` 切）。
pub const SEGMENT_READ_FRAME_BYTES: usize = SEGMENT_BYTES;
/// 读段时每把键在段长之外另算的字节（Python `assembled` 的 `lengths[index] + len(key) + 16`）。
pub const SEGMENT_READ_OVERHEAD_BYTES_PER_KEY: usize = 16;
/// 读写段与计数键时一帧至多几条（Python `FRAME_BATCH_KEYS`）。
pub const SEGMENTED_FRAME_ENTRY_LIMIT: usize = 8192;
/// 报错里键至多显示这么多个字符（Python `SHOWN_KEY_CHARACTERS`）。
const SHOWN_KEY_CHARACTERS: usize = 200;
/// 报错里存着的字节至多显示这么多个（Python 报清单解不开时取 `value[:160]`）。
const SHOWN_STORED_BYTES: usize = 160;

/// 分段存的值写不成、读不回（调用方一律照别的错退 3：Python 那一侧的 reserved-key 与 segmented-read）；不当成「库里没有」。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SegmentedValueFailure {
    /// 原键的形状是段键或计数键：一条都没写。
    ReservedKeyShape { shown_key: String },
    /// 值要的段数超过段号装得下的（[`SEGMENT_INDEX_LIMIT`]）：一条都没写。
    TooManySegmentsForTheIndexDigits {
        shown_key: String,
        segment_count: usize,
    },
    /// 计数键里存的不是十进制段数。
    SegmentCountUnreadable {
        shown_key: String,
        shown_stored: String,
    },
    /// 分段清单解不开。
    ManifestUnreadable {
        shown_key: String,
        shown_stored: String,
    },
    /// 清单点名的段不在服务里。
    SegmentMissing {
        shown_key: String,
        segment_index: usize,
        segment_count: usize,
    },
    /// 段长与清单写的不同。
    SegmentLengthDiffers {
        shown_key: String,
        segment_index: usize,
        stored_length_in_bytes: usize,
        manifest_length_in_bytes: usize,
    },
    /// 拼回的整值的 sha256 与清单写的不同。
    DigestDiffers {
        shown_key: String,
        whole_length_in_bytes: usize,
        whole_sha256: String,
        manifest_sha256: String,
    },
}

impl std::fmt::Display for SegmentedValueFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReservedKeyShape { shown_key } => write!(
                formatter,
                "键 {shown_key} 的形状是段键或计数键（键尾是 NUL 加 {SEGMENT_INDEX_DIGITS} 位十进制，或 NUL 加 count），那两种形状留给大值分段；一条都没写"
            ),
            Self::TooManySegmentsForTheIndexDigits {
                shown_key,
                segment_count,
            } => write!(
                formatter,
                "键 {shown_key} 的值要分 {segment_count} 段，段号只有 {SEGMENT_INDEX_DIGITS} 位十进制（至多 {SEGMENT_INDEX_LIMIT} 段）；一条都没写"
            ),
            Self::SegmentCountUnreadable {
                shown_key,
                shown_stored,
            } => write!(
                formatter,
                "分段存的值拼不回：{shown_key} 的计数键不是十进制段数：{shown_stored}"
            ),
            Self::ManifestUnreadable {
                shown_key,
                shown_stored,
            } => write!(
                formatter,
                "分段存的值拼不回：{shown_key} 的分段清单解不开：{shown_stored}"
            ),
            Self::SegmentMissing {
                shown_key,
                segment_index,
                segment_count,
            } => write!(
                formatter,
                "分段存的值拼不回：{shown_key} 分 {segment_count} 段存，段号 {segment_index} 不在服务里（拼不全）"
            ),
            Self::SegmentLengthDiffers {
                shown_key,
                segment_index,
                stored_length_in_bytes,
                manifest_length_in_bytes,
            } => write!(
                formatter,
                "分段存的值拼不回：{shown_key} 的段号 {segment_index} 长 {stored_length_in_bytes} 字节，清单写的是 {manifest_length_in_bytes} 字节"
            ),
            Self::DigestDiffers {
                shown_key,
                whole_length_in_bytes,
                whole_sha256,
                manifest_sha256,
            } => write!(
                formatter,
                "分段存的值拼不回：{shown_key} 拼回的 {whole_length_in_bytes} 字节 sha256 是 {whole_sha256}，清单写的是 {manifest_sha256}"
            ),
        }
    }
}

/// 报错里的键：非 ASCII 字节转义，截到 [`SHOWN_KEY_CHARACTERS`] 个字符。
fn shown_key(key: &[u8]) -> String {
    key.escape_ascii()
        .to_string()
        .chars()
        .take(SHOWN_KEY_CHARACTERS)
        .collect()
}

/// 报错里存着的字节：头 [`SHOWN_STORED_BYTES`] 个，非 ASCII 字节转义。
fn shown_stored(stored: &[u8]) -> String {
    stored[..stored.len().min(SHOWN_STORED_BYTES)]
        .escape_ascii()
        .to_string()
}

/// 原键的第 `segment_index` 段的键（Python `segment_key`）。
#[must_use]
pub fn segment_key(key: &[u8], segment_index: usize) -> Vec<u8> {
    let mut key_of_the_segment = Vec::with_capacity(key.len() + 1 + SEGMENT_INDEX_DIGITS);
    key_of_the_segment.extend_from_slice(key);
    key_of_the_segment.push(SEGMENT_KEY_SEPARATOR);
    key_of_the_segment.extend_from_slice(
        format!("{segment_index:0digits$}", digits = SEGMENT_INDEX_DIGITS).as_bytes(),
    );
    key_of_the_segment
}

/// 原键的计数键（Python `segment_count_key`）。
#[must_use]
pub fn segment_count_key(key: &[u8]) -> Vec<u8> {
    let mut count_key = Vec::with_capacity(key.len() + 1 + SEGMENT_COUNT_SUFFIX.len());
    count_key.extend_from_slice(key);
    count_key.push(SEGMENT_KEY_SEPARATOR);
    count_key.extend_from_slice(SEGMENT_COUNT_SUFFIX);
    count_key
}

/// 键是段键的形状（键尾 NUL 加 [`SEGMENT_INDEX_DIGITS`] 位十进制）就交它前面的原键与段号。
fn owner_and_index_of_a_segment_key(key: &[u8]) -> Option<(&[u8], usize)> {
    let suffix_start = key.len().checked_sub(1 + SEGMENT_INDEX_DIGITS)?;
    let (owner_key, suffix) = key.split_at(suffix_start);
    let (separator, digits) = suffix.split_first()?;
    if *separator != SEGMENT_KEY_SEPARATOR || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let segment_index = std::str::from_utf8(digits)
        .expect("上面核过每个字节都是 ASCII 数字")
        .parse::<usize>()
        .expect("六位十进制放得进 usize");
    Some((owner_key, segment_index))
}

/// 键是计数键的形状（键尾 NUL 加 [`SEGMENT_COUNT_SUFFIX`]）。
fn has_the_shape_of_a_count_key(key: &[u8]) -> bool {
    key.len() > SEGMENT_COUNT_SUFFIX.len()
        && key.ends_with(SEGMENT_COUNT_SUFFIX)
        && key[key.len() - SEGMENT_COUNT_SUFFIX.len() - 1] == SEGMENT_KEY_SEPARATOR
}

/// 键是段键或计数键的形状：写时不许当原键，扫时不当条目（文件头「与那一段只差认段键的办法」）。
#[must_use]
pub fn has_the_shape_of_a_segment_or_count_key(key: &[u8]) -> bool {
    owner_and_index_of_a_segment_key(key).is_some() || has_the_shape_of_a_count_key(key)
}

/// 这一行是分段存放的一部分（清单、段键或计数键），原样读回来的值不是调用方写下的整值：只看一行就要判「这个前缀下有没有条目」的一方
/// 见到这种行要按 [`SegmentedScanAssembler`] 扫全再判。
#[must_use]
pub fn is_part_of_segmented_storage(key: &[u8], stored: &[u8]) -> bool {
    stored.starts_with(SEGMENT_MANIFEST_MAGIC) || has_the_shape_of_a_segment_or_count_key(key)
}

/// 这个值要分段存（Python `needs_segments`）：长过一段，或本身以清单开头（不分段存会被读成清单）。
fn needs_segments(value: &[u8]) -> bool {
    value.len() > SEGMENT_BYTES || value.starts_with(SEGMENT_MANIFEST_MAGIC)
}

/// 这个值本身要几段（Python `segments_needed`）：至少一段。
fn segments_needed(value: &[u8]) -> usize {
    value.len().div_ceil(SEGMENT_BYTES).max(1)
}

/// 把 `entries` 按次序切成几帧，交各帧的下标区间（Python `batches_of`）：一帧里各条 `size_of` 之和不超过 `byte_budget`、条数不超过 `entry_limit`，
/// 单独一条就超的自成一帧。
#[must_use]
pub fn frame_ranges<FrameEntry>(
    entries: &[FrameEntry],
    size_of: impl Fn(&FrameEntry) -> usize,
    byte_budget: usize,
    entry_limit: usize,
) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut frame_start = 0;
    let mut frame_bytes = 0usize;
    for (entry_index, entry) in entries.iter().enumerate() {
        let entry_bytes = size_of(entry);
        let frame_entries = entry_index - frame_start;
        if frame_entries > 0
            && (frame_bytes.saturating_add(entry_bytes) > byte_budget
                || frame_entries >= entry_limit)
        {
            ranges.push(frame_start..entry_index);
            frame_start = entry_index;
            frame_bytes = 0;
        }
        frame_bytes = frame_bytes.saturating_add(entry_bytes);
    }
    if frame_start < entries.len() {
        ranges.push(frame_start..entries.len());
    }
    ranges
}

/// 值的第 `segment_index` 段（Python `value[index * SEGMENT_BYTES:(index + 1) * SEGMENT_BYTES]`）：越过值尾的段是空的。
fn piece_of_the_value(value: &[u8], segment_index: usize) -> &[u8] {
    let start = segment_index.saturating_mul(SEGMENT_BYTES).min(value.len());
    let end = segment_index
        .saturating_add(1)
        .saturating_mul(SEGMENT_BYTES)
        .min(value.len());
    &value[start..end]
}

/// 分段清单：各段字节数与整值的 sha256（整值字节数是各段之和）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentManifest {
    segment_lengths_in_bytes: Vec<usize>,
    whole_sha256: String,
}

impl SegmentManifest {
    /// 把 `value` 分 `segment_count` 段存时的清单。
    fn of_the_value(value: &[u8], segment_count: usize) -> SegmentManifest {
        SegmentManifest {
            segment_lengths_in_bytes: (0..segment_count)
                .map(|segment_index| piece_of_the_value(value, segment_index).len())
                .collect(),
            whole_sha256: sha256_hexadecimal(value),
        }
    }

    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segment_lengths_in_bytes.len()
    }

    fn whole_length_in_bytes(&self) -> usize {
        self.segment_lengths_in_bytes.iter().sum()
    }

    /// 存进原键的字节（Python `encode_manifest`：清单开头接 `json.dumps(sort_keys=True, separators=(",", ":"))`，键按字典序、不带空白）。
    #[must_use]
    pub fn to_stored_bytes(&self) -> Vec<u8> {
        let segment_lengths: Vec<String> = self
            .segment_lengths_in_bytes
            .iter()
            .map(usize::to_string)
            .collect();
        let document = format!(
            "{{\"length\":{},\"segments\":[{}],\"sha256\":\"{}\"}}",
            self.whole_length_in_bytes(),
            segment_lengths.join(","),
            self.whole_sha256
        );
        let mut stored = Vec::with_capacity(SEGMENT_MANIFEST_MAGIC.len() + document.len());
        stored.extend_from_slice(SEGMENT_MANIFEST_MAGIC);
        stored.extend_from_slice(document.as_bytes());
        stored
    }

    /// 原键里存着的是清单就交它（Python `manifest_of`），不以清单开头的交 `None`。
    ///
    /// 只认 Python `encode_manifest` 写出的那一种排法（与 [`Self::to_stored_bytes`] 同一种）：两边写的都是它，别的排法当解不开。
    /// 判的几样与 Python 相同：每段字节数在 0 到 [`SEGMENT_BYTES`] 之间、各段之和等于整值字节数、sha256 是 64 个字符（这里另要小写十六进制）。
    ///
    /// # Errors
    /// 以清单开头而解不开：[`SegmentedValueFailure::ManifestUnreadable`]。
    pub fn parse_stored_value(
        key: &[u8],
        stored: &[u8],
    ) -> Result<Option<SegmentManifest>, SegmentedValueFailure> {
        let Some(document) = stored.strip_prefix(SEGMENT_MANIFEST_MAGIC) else {
            return Ok(None);
        };
        let unreadable = || SegmentedValueFailure::ManifestUnreadable {
            shown_key: shown_key(key),
            shown_stored: shown_stored(stored),
        };
        let mut reader = ManifestDocumentReader {
            remaining: document,
        };
        reader.literal(b"{\"length\":").ok_or_else(unreadable)?;
        let whole_length_in_bytes = reader.unsigned_number().ok_or_else(unreadable)?;
        reader.literal(b",\"segments\":[").ok_or_else(unreadable)?;
        let mut segment_lengths_in_bytes = Vec::new();
        if reader.literal(b"]").is_none() {
            // 迭代上界：每轮至少吃掉一个数字字节，文档有尽
            loop {
                let segment_length = reader.unsigned_number().ok_or_else(unreadable)?;
                if segment_length > SEGMENT_BYTES {
                    return Err(unreadable());
                }
                segment_lengths_in_bytes.push(segment_length);
                if reader.literal(b"]").is_some() {
                    break;
                }
                reader.literal(b",").ok_or_else(unreadable)?;
            }
        }
        reader.literal(b",\"sha256\":\"").ok_or_else(unreadable)?;
        let whole_sha256 = reader
            .lowercase_hexadecimal_digest()
            .ok_or_else(unreadable)?;
        reader.literal(b"\"}").ok_or_else(unreadable)?;
        if !reader.remaining.is_empty() {
            return Err(unreadable());
        }
        let segment_length_sum = segment_lengths_in_bytes
            .iter()
            .try_fold(0usize, |sum, segment_length| {
                sum.checked_add(*segment_length)
            });
        if segment_length_sum != Some(whole_length_in_bytes) {
            return Err(unreadable());
        }
        Ok(Some(SegmentManifest {
            segment_lengths_in_bytes,
            whole_sha256,
        }))
    }
}

/// 清单里那一段 JSON 的读头：只认 [`SegmentManifest::to_stored_bytes`] 写出的那一种排法。
struct ManifestDocumentReader<'document> {
    remaining: &'document [u8],
}

/// 清单里 sha256 那一项的字符数。
const MANIFEST_DIGEST_CHARACTERS: usize = 64;

impl ManifestDocumentReader<'_> {
    fn literal(&mut self, expected: &[u8]) -> Option<()> {
        self.remaining = self.remaining.strip_prefix(expected)?;
        Some(())
    }

    /// JSON 的非负整数：至少一位，不带前导零（只有 0 本身以 0 起头），放得进 usize。
    fn unsigned_number(&mut self) -> Option<usize> {
        let digit_count = self
            .remaining
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        let (digits, rest) = self.remaining.split_at(digit_count);
        if digits.is_empty() || (digits.len() > 1 && digits[0] == b'0') {
            return None;
        }
        let number = std::str::from_utf8(digits).ok()?.parse::<usize>().ok()?;
        self.remaining = rest;
        Some(number)
    }

    fn lowercase_hexadecimal_digest(&mut self) -> Option<String> {
        if self.remaining.len() < MANIFEST_DIGEST_CHARACTERS {
            return None;
        }
        let (digest, rest) = self.remaining.split_at(MANIFEST_DIGEST_CHARACTERS);
        if !digest
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
        {
            return None;
        }
        self.remaining = rest;
        Some(String::from_utf8(digest.to_vec()).expect("上面核过每个字节都是 ASCII 十六进制数字"))
    }
}

/// 写之前要先点读的计数键，与 `entries` 同序（Python `write_many` 先读 `segment_high_water`）。
///
/// # Errors
/// 有一把原键的形状是段键或计数键（[`SegmentedValueFailure::ReservedKeyShape`]）：一条都不写。
pub fn segment_count_keys_to_read_before_writing(
    entries: &[(Table, Vec<u8>, Vec<u8>)],
) -> Result<Vec<(Table, Vec<u8>)>, SegmentedValueFailure> {
    entries
        .iter()
        .map(|(table, key, _value)| {
            if has_the_shape_of_a_segment_or_count_key(key) {
                return Err(SegmentedValueFailure::ReservedKeyShape {
                    shown_key: shown_key(key),
                });
            }
            Ok((*table, segment_count_key(key)))
        })
        .collect()
}

/// 计数键里记的段数高水位（Python `segment_high_water`）：计数键不在是 0。
///
/// # Errors
/// 计数键在而不是十进制段数：[`SegmentedValueFailure::SegmentCountUnreadable`]。
pub fn segment_count_high_water(
    key: &[u8],
    stored_count: Option<&[u8]>,
) -> Result<usize, SegmentedValueFailure> {
    let Some(stored_count) = stored_count else {
        return Ok(0);
    };
    let unreadable = || SegmentedValueFailure::SegmentCountUnreadable {
        shown_key: shown_key(key),
        shown_stored: shown_stored(stored_count),
    };
    if stored_count.is_empty() || !stored_count.iter().all(u8::is_ascii_digit) {
        return Err(unreadable());
    }
    std::str::from_utf8(stored_count)
        .expect("上面核过每个字节都是 ASCII 数字")
        .parse::<usize>()
        .map_err(|_overflow| unreadable())
}

/// 一批写拆成的两步（Python `write_many`）：各段与计数键先写、等确认，清单与不分段的条目最后写。
#[derive(Debug, PartialEq, Eq)]
pub struct SegmentedWritePlan {
    pub segment_and_count_entries: Vec<(Table, Vec<u8>, Vec<u8>)>,
    pub entries_to_write_last: Vec<(Table, Vec<u8>, Vec<u8>)>,
}

/// 照计数键里读到的高水位（与 `entries` 同序，[`segment_count_high_water`]）拆一批写（Python `write_many`）：分段存过的（高水位不为 0）
/// 与要分段的（[`needs_segments`]）存成清单加段，段数取「这个值要几段」与高水位两者大的那一个，多出来的写成空段、照进清单
/// （重写同一把键时以前写过的段号全被新清单点名，旧段不留孤儿）；别的照原样。同一批里同一把键写了几次的，各次都照写之前读到的高水位取（与 Python 相同）。
///
/// # Errors
/// 有一个值要的段数超过段号装得下的（[`SegmentedValueFailure::TooManySegmentsForTheIndexDigits`]）：一条都不写。
///
/// # Panics
/// `segment_count_high_water_of_each_entry` 与 `entries` 不一样长（调用方照 [`segment_count_keys_to_read_before_writing`] 读回来的同序一份）。
pub fn plan_segmented_write(
    entries: Vec<(Table, Vec<u8>, Vec<u8>)>,
    segment_count_high_water_of_each_entry: &[usize],
) -> Result<SegmentedWritePlan, SegmentedValueFailure> {
    assert_eq!(
        entries.len(),
        segment_count_high_water_of_each_entry.len(),
        "每个条目一个高水位：调用方照 segment_count_keys_to_read_before_writing 同序读回"
    );
    let mut plan = SegmentedWritePlan {
        segment_and_count_entries: Vec::new(),
        entries_to_write_last: Vec::new(),
    };
    for ((table, key, value), &previous_segment_count) in entries
        .into_iter()
        .zip(segment_count_high_water_of_each_entry)
    {
        if previous_segment_count == 0 && !needs_segments(&value) {
            plan.entries_to_write_last.push((table, key, value));
            continue;
        }
        let segment_count = segments_needed(&value).max(previous_segment_count);
        if segment_count > SEGMENT_INDEX_LIMIT {
            return Err(SegmentedValueFailure::TooManySegmentsForTheIndexDigits {
                shown_key: shown_key(&key),
                segment_count,
            });
        }
        for segment_index in 0..segment_count {
            plan.segment_and_count_entries.push((
                table,
                segment_key(&key, segment_index),
                piece_of_the_value(&value, segment_index).to_vec(),
            ));
        }
        plan.segment_and_count_entries.push((
            table,
            segment_count_key(&key),
            segment_count.to_string().into_bytes(),
        ));
        let manifest = SegmentManifest::of_the_value(&value, segment_count);
        plan.entries_to_write_last
            .push((table, key, manifest.to_stored_bytes()));
    }
    Ok(plan)
}

/// 一把分段存的键：清单已经读到，段还没收齐（按前缀扫时紧跟在原键之后扫到的段先收着，缺的由调用方点读补上）。
#[derive(Debug)]
pub struct SegmentedValueBeingAssembled {
    owner_key: Vec<u8>,
    manifest: SegmentManifest,
    segments_already_read: BTreeMap<usize, Vec<u8>>,
}

impl SegmentedValueBeingAssembled {
    /// 点读原键读回清单：一段都还没读。
    #[must_use]
    pub fn read_by_point(owner_key: Vec<u8>, manifest: SegmentManifest) -> Self {
        Self {
            owner_key,
            manifest,
            segments_already_read: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn owner_key(&self) -> &[u8] {
        &self.owner_key
    }

    /// 清单点名、还没读到的段号，升序。
    #[must_use]
    pub fn missing_segment_indexes(&self) -> Vec<usize> {
        (0..self.manifest.segment_count())
            .filter(|segment_index| !self.segments_already_read.contains_key(segment_index))
            .collect()
    }

    /// 清单写的第 `segment_index` 段的字节数（调用方照它切读段的帧）。
    ///
    /// # Panics
    /// 段号不在清单里（调用方只拿 [`Self::missing_segment_indexes`] 交的段号来问）。
    #[must_use]
    pub fn segment_length_in_bytes(&self, segment_index: usize) -> usize {
        self.manifest.segment_lengths_in_bytes[segment_index]
    }

    /// 补上点读来的段（`None` 是服务里没有），按清单拼回整值，核每段长度与整值 sha256（Python `assembled`）；交 (原键, 整值)。
    ///
    /// # Errors
    /// 段缺了、长度对不上、sha256 对不上。
    pub fn assemble(
        mut self,
        segments_read_by_point: Vec<(usize, Option<Vec<u8>>)>,
    ) -> Result<(Vec<u8>, Vec<u8>), SegmentedValueFailure> {
        for (segment_index, piece) in segments_read_by_point {
            if let Some(piece) = piece {
                self.segments_already_read.insert(segment_index, piece);
            }
        }
        let segment_count = self.manifest.segment_count();
        let mut whole = Vec::with_capacity(self.manifest.whole_length_in_bytes());
        for (segment_index, &manifest_length_in_bytes) in
            self.manifest.segment_lengths_in_bytes.iter().enumerate()
        {
            let Some(piece) = self.segments_already_read.remove(&segment_index) else {
                return Err(SegmentedValueFailure::SegmentMissing {
                    shown_key: shown_key(&self.owner_key),
                    segment_index,
                    segment_count,
                });
            };
            if piece.len() != manifest_length_in_bytes {
                return Err(SegmentedValueFailure::SegmentLengthDiffers {
                    shown_key: shown_key(&self.owner_key),
                    segment_index,
                    stored_length_in_bytes: piece.len(),
                    manifest_length_in_bytes,
                });
            }
            whole.extend_from_slice(&piece);
        }
        let whole_sha256 = sha256_hexadecimal(&whole);
        if whole_sha256 != self.manifest.whole_sha256 {
            return Err(SegmentedValueFailure::DigestDiffers {
                shown_key: shown_key(&self.owner_key),
                whole_length_in_bytes: whole.len(),
                whole_sha256,
                manifest_sha256: self.manifest.whole_sha256,
            });
        }
        Ok((self.owner_key, whole))
    }
}

/// 拼不回时重读原键之前依次等这么久（毫秒；Python `SEGMENTED_REREAD_DELAYS_SECONDS`）：同一把键正被重写时，读的一方可能读到旧清单配上新写的段，
/// 写的一方紧接着写新清单（写见 Python 文件头「大值分段」），等这几趟清单就换了。
pub const SEGMENTED_REREAD_DELAYS_MILLISECONDS: [u64; 6] = [50, 100, 200, 400, 800, 1600];

/// 点读来的段：(段号, 段)，段是 `None` 的是服务里没有。
pub type SegmentsReadByPoint = Vec<(usize, Option<Vec<u8>>)>;

/// 拼回的一把键：(原键, 整值)。
pub type AssembledValue = (Vec<u8>, Vec<u8>);

/// 拼一把分段存的键时向服务要的三样（[`assemble_rereading_the_original_key`] 用）：点读还缺的段、重读原键里存着的原样、等一会儿。
pub trait StoredSegmentsReader {
    type ReadFailure;

    /// 点读 `being_assembled` 清单点名、还缺的段（`None` 是服务里没有）。
    ///
    /// # Errors
    /// 读不成。
    fn read_missing_segments(
        &mut self,
        being_assembled: &SegmentedValueBeingAssembled,
    ) -> Result<SegmentsReadByPoint, Self::ReadFailure>;

    /// 重读原键在服务里存着的原样（`None` 是不在）。
    ///
    /// # Errors
    /// 读不成。
    fn reread_the_original_key(&mut self) -> Result<Option<Vec<u8>>, Self::ReadFailure>;

    fn wait(&mut self, delay: Duration);
}

/// 拼回一把分段存的键（Python `assembled`）：拼不回（段缺了、长度或 sha256 对不上）就照 [`SEGMENTED_REREAD_DELAYS_MILLISECONDS`] 逐次等、重读原键——
/// 清单换过（同一把键正被重写，读到的是旧清单配新段）就丢掉已经收下的段、照新清单重拼；原键不再是清单就交它；等完清单还是原来那一份，交头一次拼不回的原因
/// （写到一半断了或被改过）。外层 `Err` 是读不成，内层 `Err` 是拼不回。
///
/// # Errors
/// 读段或重读原键不成（`reader` 的错）。
pub fn assemble_rereading_the_original_key<Reader: StoredSegmentsReader>(
    mut being_assembled: SegmentedValueBeingAssembled,
    reader: &mut Reader,
) -> Result<Result<AssembledValue, SegmentedValueFailure>, Reader::ReadFailure> {
    let mut delays = SEGMENTED_REREAD_DELAYS_MILLISECONDS.iter();
    loop {
        let manifest = being_assembled.manifest.clone();
        let owner_key = being_assembled.owner_key.clone();
        let pieces = reader.read_missing_segments(&being_assembled)?;
        let failure = match being_assembled.assemble(pieces) {
            Ok(assembled) => return Ok(Ok(assembled)),
            Err(failure) => failure,
        };
        let replacement = loop {
            let Some(&delay) = delays.next() else {
                return Ok(Err(failure));
            };
            reader.wait(Duration::from_millis(delay));
            let Some(stored) = reader.reread_the_original_key()? else {
                return Ok(Err(failure));
            };
            match SegmentManifest::parse_stored_value(&owner_key, &stored) {
                Ok(Some(reread)) if reread == manifest => {}
                Ok(Some(reread)) => break reread,
                Ok(None) => return Ok(Ok((owner_key, stored))),
                Err(unreadable) => return Ok(Err(unreadable)),
            }
        };
        being_assembled = SegmentedValueBeingAssembled::read_by_point(owner_key, replacement);
    }
}

/// 按前缀扫回来的一条，交给调用方之前的样子。
#[derive(Debug)]
pub enum ScannedEntry {
    /// 不分段存的条目，原样交。
    Whole { key: Vec<u8>, value: Vec<u8> },
    /// 分段存的键：调用方点读补上缺的段、拼回（[`SegmentedValueBeingAssembled::assemble`]）再交。
    Segmented(SegmentedValueBeingAssembled),
}

/// 按前缀扫的拼接（Python `scan_stream`）：按键升序逐条喂进服务里存着的原样，段键与计数键不交，清单那一条连同紧跟在它后面扫到的段交出去拼回；
/// 清单点名之外的段不交。同一时刻只攒一把键的段。
#[derive(Debug)]
pub struct SegmentedScanAssembler {
    being_assembled: Option<SegmentedValueBeingAssembled>,
}

impl SegmentedScanAssembler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            being_assembled: None,
        }
    }

    /// 喂进扫回来的一条（按键升序）；交这一条喂进来之后可以交出去的（至多两条：攒着的那把分段键，与这一条本身）。
    ///
    /// # Errors
    /// 这一条以清单开头而清单解不开。
    pub fn accept_stored_entry(
        &mut self,
        key: Vec<u8>,
        stored: Vec<u8>,
    ) -> Result<Vec<ScannedEntry>, SegmentedValueFailure> {
        if let Some(being_assembled) = &mut self.being_assembled {
            if let Some((owner_key, segment_index)) = owner_and_index_of_a_segment_key(&key) {
                if owner_key == being_assembled.owner_key.as_slice() {
                    being_assembled
                        .segments_already_read
                        .insert(segment_index, stored);
                    return Ok(Vec::new());
                }
            }
        }
        if has_the_shape_of_a_segment_or_count_key(&key) {
            return Ok(Vec::new());
        }
        let mut ready = Vec::new();
        if let Some(being_assembled) = self.being_assembled.take() {
            ready.push(ScannedEntry::Segmented(being_assembled));
        }
        match SegmentManifest::parse_stored_value(&key, &stored)? {
            Some(manifest) => {
                self.being_assembled =
                    Some(SegmentedValueBeingAssembled::read_by_point(key, manifest));
            }
            None => ready.push(ScannedEntry::Whole { key, value: stored }),
        }
        Ok(ready)
    }

    /// 扫完了：还攒着的那把分段键。
    #[must_use]
    pub fn finish(self) -> Option<SegmentedValueBeingAssembled> {
        self.being_assembled
    }
}

impl Default for SegmentedScanAssembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: Table = Table::Conclusions;

    /// 一个值照 Python `encode_manifest` 存成的字节：清单开头接 `{"length":…,"segments":[…],"sha256":"…"}`，键按字典序、不带空白。
    /// 期望值照那份 Python 的写法手拼（`hello` 的 sha256 取自 `sha256sum`）：Rust 写出的清单与 Python 逐字节相同，Python 才读得回。
    /// 判别力（`crates/mutations.tsv`）：清单里各段字节数之间多了空白（与 Python 的 separators 写法不同），这条红。
    #[test]
    fn manifest_bytes_match_what_the_python_client_writes_for_a_known_value() {
        let manifest = SegmentManifest::of_the_value(b"hello", 3);
        assert_eq!(
            manifest.to_stored_bytes(),
            b"\x00singlefs-prune-server-segments\n{\"length\":5,\"segments\":[5,0,0],\"sha256\":\"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\"}"
                .to_vec(),
            "清单要与 Python 的 json.dumps(sort_keys=True, separators=(',', ':')) 逐字节相同"
        );
    }

    /// 段键是原键接 NUL 与六位补零的段号，计数键是原键接 NUL 与 count（Python `segment_key`、`segment_count_key`）。
    /// 判别力（`crates/mutations.tsv`）：段键里原键与段号之间不接 NUL，这条红。
    #[test]
    fn segment_and_count_keys_have_the_python_layout() {
        assert_eq!(
            segment_key(b"family/kind/key", 7),
            b"family/kind/key\x00000007".to_vec()
        );
        assert_eq!(
            segment_count_key(b"family/kind/key"),
            b"family/kind/key\x00count".to_vec()
        );
    }

    /// 长过一段的值存成清单加段：各段与计数键排在前一步，清单排在最后一步，段长依次是一整段、一整段、余下的。
    /// 判别力（`crates/mutations.tsv`）：切段时每段取两段长（段与段重叠），这条红。
    #[test]
    fn a_value_longer_than_a_segment_is_planned_as_segments_then_a_manifest() {
        let value: Vec<u8> = (0..2 * SEGMENT_BYTES + 5)
            .map(|position| u8::try_from(position % 251).expect("取模 251 放得进 u8"))
            .collect();
        let plan = plan_segmented_write(vec![(TABLE, b"key".to_vec(), value.clone())], &[0])
            .expect("三段装得下六位段号");
        let keys_written_first: Vec<&[u8]> = plan
            .segment_and_count_entries
            .iter()
            .map(|(_table, key, _value)| key.as_slice())
            .collect();
        assert_eq!(
            keys_written_first,
            vec![
                b"key\x00000000".as_slice(),
                b"key\x00000001".as_slice(),
                b"key\x00000002".as_slice(),
                b"key\x00count".as_slice()
            ]
        );
        assert_eq!(
            plan.segment_and_count_entries[2].2,
            value[2 * SEGMENT_BYTES..].to_vec()
        );
        assert_eq!(plan.segment_and_count_entries[3].2, b"3".to_vec());
        assert_eq!(
            plan.entries_to_write_last,
            vec![(
                TABLE,
                b"key".to_vec(),
                SegmentManifest {
                    segment_lengths_in_bytes: vec![SEGMENT_BYTES, SEGMENT_BYTES, 5],
                    whole_sha256: sha256_hexadecimal(&value),
                }
                .to_stored_bytes()
            )]
        );
    }

    /// 分段存过三段的键改写成一个短值：照高水位仍写三段（后两段是空段）、计数键仍是 3，新清单点名全部三段，旧段不留孤儿。
    /// 判别力（`crates/mutations.tsv`）：段数只取这个值要几段、不看高水位（Python 弄坏开关 segment-count-shrinks-on-rewrite 那一种），这条红。
    #[test]
    fn a_rewrite_with_fewer_segments_still_names_every_segment_the_high_water_recorded() {
        let plan = plan_segmented_write(vec![(TABLE, b"key".to_vec(), b"short".to_vec())], &[3])
            .expect("三段装得下六位段号");
        assert_eq!(
            plan.segment_and_count_entries,
            vec![
                (TABLE, b"key\x00000000".to_vec(), b"short".to_vec()),
                (TABLE, b"key\x00000001".to_vec(), Vec::new()),
                (TABLE, b"key\x00000002".to_vec(), Vec::new()),
                (TABLE, b"key\x00count".to_vec(), b"3".to_vec()),
            ]
        );
        assert_eq!(
            plan.entries_to_write_last,
            vec![(
                TABLE,
                b"key".to_vec(),
                SegmentManifest::of_the_value(b"short", 3).to_stored_bytes()
            )]
        );
    }

    /// 以清单开头的短值也分段存（否则读回来会被当成清单），没分段存过的短值照原样写、不另写计数键。
    /// 判别力（`crates/mutations.tsv`）：以清单开头的短值不分段，这条红。
    #[test]
    fn a_short_value_starting_with_the_manifest_magic_is_segmented_and_a_plain_one_is_not() {
        let mut looks_like_a_manifest = SEGMENT_MANIFEST_MAGIC.to_vec();
        looks_like_a_manifest.extend_from_slice(b"not a manifest");
        let plan = plan_segmented_write(
            vec![
                (TABLE, b"magic".to_vec(), looks_like_a_manifest.clone()),
                (TABLE, b"plain".to_vec(), b"plain value".to_vec()),
            ],
            &[0, 0],
        )
        .expect("一段装得下六位段号");
        assert_eq!(
            plan.segment_and_count_entries,
            vec![
                (
                    TABLE,
                    b"magic\x00000000".to_vec(),
                    looks_like_a_manifest.clone()
                ),
                (TABLE, b"magic\x00count".to_vec(), b"1".to_vec()),
            ]
        );
        assert_eq!(
            plan.entries_to_write_last,
            vec![
                (
                    TABLE,
                    b"magic".to_vec(),
                    SegmentManifest::of_the_value(&looks_like_a_manifest, 1).to_stored_bytes()
                ),
                (TABLE, b"plain".to_vec(), b"plain value".to_vec()),
            ]
        );
    }

    /// 原键的形状是段键或计数键的，写之前就拒、一条都不写；表标签开头、中间带 NUL 的二进制键照收。
    /// 判别力（`crates/mutations.tsv`）：写之前不核原键的形状，这条红。
    #[test]
    fn a_key_shaped_like_a_segment_or_count_key_is_refused_and_other_binary_keys_are_accepted() {
        for reserved in [b"key\x00000003".to_vec(), b"key\x00count".to_vec()] {
            match segment_count_keys_to_read_before_writing(&[(
                TABLE,
                reserved.clone(),
                b"value".to_vec(),
            )]) {
                Err(SegmentedValueFailure::ReservedKeyShape { .. }) => {}
                other => panic!("{reserved:?} 是段键或计数键的形状，要拒；交回的是 {other:?}"),
            }
        }
        let binary_key = vec![0x00, 0x05, 0x00, 0xFF, b'7'];
        assert_eq!(
            segment_count_keys_to_read_before_writing(&[(
                TABLE,
                binary_key.clone(),
                b"value".to_vec()
            )]),
            Ok(vec![(TABLE, segment_count_key(&binary_key))]),
            "表标签开头、带 NUL 而形状不是段键或计数键的键照收"
        );
    }

    /// 切帧照 Python `batches_of`：加上这一条超过字节上限或条数到了上限就另起一帧，单独一条就超的自成一帧。
    /// 判别力（`crates/mutations.tsv`）：条数到了上限还往这一帧里放一条，这条红。
    #[test]
    fn frames_are_cut_where_the_next_entry_would_pass_the_byte_budget_or_the_entry_limit() {
        assert_eq!(
            frame_ranges(&[4usize, 4, 4, 20, 1], |size| *size, 10, 8),
            vec![0..2, 2..3, 3..4, 4..5]
        );
        assert_eq!(
            frame_ranges(&[1usize, 1, 1], |size| *size, 10, 2),
            vec![0..2, 2..3]
        );
        assert!(frame_ranges(&[] as &[usize], |size| *size, 10, 2).is_empty());
    }

    /// 计数键不是十进制段数的当读错，计数键不在是 0。
    /// 判别力（`crates/mutations.tsv`）：计数键不在当成分段存过一段，这条红。
    #[test]
    fn a_segment_count_that_is_not_decimal_is_unreadable_and_an_absent_one_is_zero() {
        assert_eq!(segment_count_high_water(b"key", None), Ok(0));
        assert_eq!(segment_count_high_water(b"key", Some(b"12")), Ok(12));
        for unreadable in [b"".as_slice(), b"1x".as_slice(), b"-1".as_slice()] {
            assert!(
                matches!(
                    segment_count_high_water(b"key", Some(unreadable)),
                    Err(SegmentedValueFailure::SegmentCountUnreadable { .. })
                ),
                "{unreadable:?} 不是十进制段数"
            );
        }
    }

    fn being_assembled_from(value: &[u8], segment_count: usize) -> SegmentedValueBeingAssembled {
        SegmentedValueBeingAssembled::read_by_point(
            b"key".to_vec(),
            SegmentManifest::of_the_value(value, segment_count),
        )
    }

    /// 一份假的服务：段与原键各存一份；`rewrite_after_waits` 次等待之后把原键换成 `rewritten`（演写的一方紧接着写新清单）。
    struct ServiceBeingRewritten {
        segments: BTreeMap<usize, Vec<u8>>,
        original: Vec<u8>,
        rewritten: Option<Vec<u8>>,
        rewrite_after_waits: usize,
        waits: Vec<Duration>,
    }

    impl StoredSegmentsReader for ServiceBeingRewritten {
        type ReadFailure = ();

        fn read_missing_segments(
            &mut self,
            being_assembled: &SegmentedValueBeingAssembled,
        ) -> Result<SegmentsReadByPoint, ()> {
            Ok(being_assembled
                .missing_segment_indexes()
                .into_iter()
                .map(|segment_index| (segment_index, self.segments.get(&segment_index).cloned()))
                .collect())
        }

        fn reread_the_original_key(&mut self) -> Result<Option<Vec<u8>>, ()> {
            if self.waits.len() >= self.rewrite_after_waits {
                if let Some(rewritten) = self.rewritten.take() {
                    self.original = rewritten;
                }
            }
            Ok(Some(self.original.clone()))
        }

        fn wait(&mut self, delay: Duration) {
            self.waits.push(delay);
        }
    }

    /// 旧值「hello」两段存（第二段空），段已换成新值「jello」的、新清单还没写：读的一方拿旧清单拼不回，
    /// `rewrite_after_waits` 次等待之后原键换成新清单。
    fn service_rewriting_hello_into_jello(rewrite_after_waits: usize) -> ServiceBeingRewritten {
        ServiceBeingRewritten {
            segments: BTreeMap::from([(0, b"jello".to_vec()), (1, Vec::new())]),
            original: SegmentManifest::of_the_value(b"hello", 2).to_stored_bytes(),
            rewritten: Some(SegmentManifest::of_the_value(b"jello", 2).to_stored_bytes()),
            rewrite_after_waits,
            waits: Vec::new(),
        }
    }

    /// 读到旧清单配新段：等到写的一方写了新清单，照新清单拼回新值（同一把键正被重写时读的一方不退 3）。
    /// 判别力（`crates/mutations.tsv`）：拼不回不重读原键、当场交拼不回，这条红。
    #[test]
    fn a_reader_that_met_a_rewrite_halfway_rereads_the_manifest_and_assembles_the_new_value() {
        let mut service = service_rewriting_hello_into_jello(3);
        let outcome =
            assemble_rereading_the_original_key(being_assembled_from(b"hello", 2), &mut service);
        assert_eq!(
            outcome,
            Ok(Ok((b"key".to_vec(), b"jello".to_vec()))),
            "第三次等完原键换成新清单，照它拼回新值"
        );
        assert_eq!(
            service.waits,
            SEGMENTED_REREAD_DELAYS_MILLISECONDS[..3]
                .iter()
                .map(|&milliseconds| Duration::from_millis(milliseconds))
                .collect::<Vec<_>>(),
            "清单没换之前照表逐次等"
        );
    }

    /// 清单一直没换（写到一半断了或被改过）：等完照旧交头一次拼不回的原因，不当成读到了值。
    /// 判别力（`crates/mutations.tsv`）：等完清单没换却交成拼回了，这条红。
    #[test]
    fn a_manifest_that_never_changes_after_every_wait_is_still_refused() {
        let mut service = service_rewriting_hello_into_jello(usize::MAX);
        let outcome =
            assemble_rereading_the_original_key(being_assembled_from(b"hello", 2), &mut service);
        assert!(
            matches!(
                outcome,
                Ok(Err(SegmentedValueFailure::DigestDiffers { .. }))
            ),
            "清单一直是旧的，段是新的：拼不回 {outcome:?}"
        );
        assert_eq!(
            service.waits.len(),
            SEGMENTED_REREAD_DELAYS_MILLISECONDS.len(),
            "表里每一档都等过"
        );
    }

    /// 段齐、核得过的头一次就拼回，不等、不重读。
    #[test]
    fn a_value_that_assembles_at_once_does_not_wait() {
        let mut service = service_rewriting_hello_into_jello(0);
        service.segments = BTreeMap::from([(0, b"hello".to_vec()), (1, Vec::new())]);
        let outcome =
            assemble_rereading_the_original_key(being_assembled_from(b"hello", 2), &mut service);
        assert_eq!(outcome, Ok(Ok((b"key".to_vec(), b"hello".to_vec()))));
        assert!(
            service.waits.is_empty(),
            "头一次拼得回就不等：{:?}",
            service.waits
        );
    }

    /// 清单点名的段缺了一段：拼不回，不交值。
    /// 判别力（`crates/mutations.tsv`）：拼段时缺的段当空段拼（报成长度不对），这条红。
    #[test]
    fn assembling_refuses_a_missing_segment() {
        let outcome = being_assembled_from(b"hello", 2)
            .assemble(vec![(0, Some(b"hello".to_vec())), (1, None)]);
        assert!(
            matches!(
                outcome,
                Err(SegmentedValueFailure::SegmentMissing {
                    segment_index: 1,
                    ..
                })
            ),
            "段号 1 不在服务里：{outcome:?}"
        );
    }

    /// 一段短了：拼不回。
    /// 判别力（`crates/mutations.tsv`）：拼段时不核每段长度（报成 sha256 对不上），这条红。
    #[test]
    fn assembling_refuses_a_segment_whose_length_differs_from_the_manifest() {
        let outcome = being_assembled_from(b"hello", 1).assemble(vec![(0, Some(b"hell".to_vec()))]);
        assert!(
            matches!(
                outcome,
                Err(SegmentedValueFailure::SegmentLengthDiffers {
                    segment_index: 0,
                    ..
                })
            ),
            "段号 0 短了一个字节：{outcome:?}"
        );
    }

    /// 一段同长换了字节：只有 sha256 认得出，拼不回。
    /// 判别力（`crates/mutations.tsv`）：拼回之后不核整值的 sha256（Python 弄坏开关 segments-assembled-without-sha256 那一种），这条红。
    #[test]
    fn assembling_refuses_a_whole_value_whose_digest_differs_from_the_manifest() {
        let outcome =
            being_assembled_from(b"hello", 1).assemble(vec![(0, Some(b"jello".to_vec()))]);
        assert!(
            matches!(outcome, Err(SegmentedValueFailure::DigestDiffers { .. })),
            "同长换了一个字节，sha256 对不上：{outcome:?}"
        );
        assert_eq!(
            being_assembled_from(b"hello", 2)
                .assemble(vec![(0, Some(b"hello".to_vec())), (1, Some(Vec::new()))]),
            Ok((b"key".to_vec(), b"hello".to_vec())),
            "段齐、长度与 sha256 都对得上的拼回整值"
        );
    }

    /// 清单解不开（各段之和不等于整值字节数、一段长过上限、排法不是 Python 写的那一种）当读错；不以清单开头的不是清单。
    /// 判别力（`crates/mutations.tsv`）：不核各段之和等于整值字节数，这条红。
    #[test]
    fn a_manifest_that_does_not_add_up_or_is_laid_out_otherwise_is_unreadable() {
        let digest = sha256_hexadecimal(b"hello");
        let sum_differs = format!("{{\"length\":6,\"segments\":[5],\"sha256\":\"{digest}\"}}");
        let segment_too_long = format!(
            "{{\"length\":{too_long},\"segments\":[{too_long}],\"sha256\":\"{digest}\"}}",
            too_long = SEGMENT_BYTES + 1
        );
        let spaced = format!("{{\"length\": 5, \"segments\": [5], \"sha256\": \"{digest}\"}}");
        for document in [sum_differs, segment_too_long, spaced] {
            let mut stored = SEGMENT_MANIFEST_MAGIC.to_vec();
            stored.extend_from_slice(document.as_bytes());
            assert!(
                matches!(
                    SegmentManifest::parse_stored_value(b"key", &stored),
                    Err(SegmentedValueFailure::ManifestUnreadable { .. })
                ),
                "{document} 解不开"
            );
        }
        assert_eq!(
            SegmentManifest::parse_stored_value(b"key", b"plain value"),
            Ok(None)
        );
        let manifest = SegmentManifest::of_the_value(b"hello", 3);
        assert_eq!(
            SegmentManifest::parse_stored_value(b"key", &manifest.to_stored_bytes()),
            Ok(Some(manifest)),
            "自己写出的清单读得回"
        );
    }

    /// 按键升序喂进一把分段键（清单、三段、计数键）与它前后两条：交出去的只有三把原键，分段那一把交拼回的整值。
    /// 判别力（`crates/mutations.tsv`）：扫时段键与计数键也当条目交出去，这条红。
    #[test]
    fn the_scan_assembler_hands_out_owner_keys_only_and_assembles_the_segmented_one() {
        let value = b"segmented value".to_vec();
        let plan = plan_segmented_write(
            vec![
                (TABLE, b"prefix/a".to_vec(), b"before".to_vec()),
                (TABLE, b"prefix/b".to_vec(), value.clone()),
                (TABLE, b"prefix/c".to_vec(), b"after".to_vec()),
            ],
            &[0, 3, 0],
        )
        .expect("三段装得下六位段号");
        let mut stored_in_key_order: Vec<(Vec<u8>, Vec<u8>)> = plan
            .segment_and_count_entries
            .into_iter()
            .chain(plan.entries_to_write_last)
            .map(|(_table, key, stored)| (key, stored))
            .collect();
        stored_in_key_order.sort();
        let mut assembler = SegmentedScanAssembler::new();
        let mut handed_out = Vec::new();
        for (key, stored) in stored_in_key_order {
            for scanned in assembler
                .accept_stored_entry(key, stored)
                .expect("清单解得开")
            {
                handed_out.push(scanned);
            }
        }
        assert!(
            assembler.finish().is_none(),
            "分段那一把后面还有一条，早已交出去"
        );
        let mut entries = Vec::new();
        for scanned in handed_out {
            match scanned {
                ScannedEntry::Whole {
                    key,
                    value: whole_value,
                } => entries.push((key, whole_value)),
                ScannedEntry::Segmented(being_assembled) => {
                    assert!(
                        being_assembled.missing_segment_indexes().is_empty(),
                        "三段紧跟在原键之后，都扫到了"
                    );
                    entries.push(being_assembled.assemble(Vec::new()).expect("段齐"));
                }
            }
        }
        assert_eq!(
            entries,
            vec![
                (b"prefix/a".to_vec(), b"before".to_vec()),
                (b"prefix/b".to_vec(), value),
                (b"prefix/c".to_vec(), b"after".to_vec()),
            ]
        );
    }

    /// 前面没有清单的段键与计数键（写到一半断了留下的、或原键落在前缀之外）不当条目，也不拿去拼别的键。
    /// 判别力（`crates/mutations.tsv`）：计数键的形状认不出，这条红。
    #[test]
    fn the_scan_assembler_skips_segment_and_count_keys_with_no_manifest_before_them() {
        let mut assembler = SegmentedScanAssembler::new();
        for (key, stored) in [
            (b"orphan\x00000000".to_vec(), b"piece".to_vec()),
            (b"orphan\x00count".to_vec(), b"1".to_vec()),
        ] {
            assert!(
                assembler
                    .accept_stored_entry(key, stored)
                    .expect("不是清单")
                    .is_empty(),
                "段键与计数键不当条目"
            );
        }
        assert!(assembler.finish().is_none());
    }
}
