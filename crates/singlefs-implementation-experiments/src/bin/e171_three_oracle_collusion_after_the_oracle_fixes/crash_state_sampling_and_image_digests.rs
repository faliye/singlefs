//! 三批取样点、撕裂扫描的取样序号、状态掩码与崩溃镜像的摘要。

use super::*;

// ───────────────────────────── 取样（登记 5.4） ─────────────────────────────

/// 批 1 在一个非空区间 [a, b) 里取的点：a、b − 1 与 a + ⌊i·(b − 1 − a)/7⌋（i = 1..6）。
fn first_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    let mut points = vec![first, state_range.end - 1];
    for step in 1..FIRST_BATCH_DIVISOR {
        points.push(first + step * span / FIRST_BATCH_DIVISOR);
    }
    points
}

/// 批 2：a + ⌊(2i + 1)·(b − 1 − a)/14⌋（i = 0..6）。
fn second_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    (0..SECOND_BATCH_DIVISOR / 2)
        .map(|step| first + (2 * step + 1) * span / SECOND_BATCH_DIVISOR)
        .collect()
}

/// 批 3：a + ⌊i·(b − 1 − a)/23⌋（i = 0..23）。
fn third_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    (0..=THIRD_BATCH_DIVISOR)
        .map(|step| first + step * span / THIRD_BATCH_DIVISOR)
        .collect()
}

/// 带撕裂写的那一段扫了几个序号、取到几个带「1」的状态（登记 5.4「扫不到 4 个的照实报个数」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TornScan {
    pub(super) segment_index: usize,
    pub(super) scanned: u64,
    pub(super) found: usize,
}

/// 三批的状态序号（各自升序、互不重叠）与撕裂扫描的记录。
pub(super) struct SampledOrdinals {
    pub(super) batches: [BTreeSet<u64>; 3],
    pub(super) torn_scans: Vec<TornScan>,
}

pub(super) fn sample_ordinals(
    state_ranges_by_segment: &[std::ops::Range<u64>],
    state_count: u64,
    segment_has_a_torn_write: &[bool],
    landing_vector_of_ordinal: &dyn Fn(u64) -> String,
) -> SampledOrdinals {
    let non_empty = || {
        state_ranges_by_segment
            .iter()
            .filter(|range| !range.is_empty())
    };
    let mut first: BTreeSet<u64> = non_empty().flat_map(first_batch_points).collect();
    first.insert(state_count - 1);
    let mut torn_scans = Vec::new();
    for (segment_index, state_range) in state_ranges_by_segment.iter().enumerate() {
        if state_range.is_empty() || !segment_has_a_torn_write[segment_index] {
            continue;
        }
        let scan_end = state_range
            .end
            .min(state_range.start + TORN_SCAN_ORDINALS_PER_SEGMENT);
        let mut found = 0usize;
        let mut scanned = 0u64;
        for ordinal in state_range.start..scan_end {
            if found == TORN_STATES_PER_SEGMENT {
                break;
            }
            scanned += 1;
            if landing_vector_of_ordinal(ordinal).contains('1') && first.insert(ordinal) {
                found += 1;
            }
        }
        torn_scans.push(TornScan {
            segment_index,
            scanned,
            found,
        });
    }
    let second: BTreeSet<u64> = non_empty()
        .flat_map(second_batch_points)
        .filter(|ordinal| !first.contains(ordinal))
        .collect();
    let third: BTreeSet<u64> = non_empty()
        .flat_map(third_batch_points)
        .filter(|ordinal| !first.contains(ordinal) && !second.contains(ordinal))
        .collect();
    SampledOrdinals {
        batches: [first, second, third],
        torn_scans,
    }
}

// ───────────────────────────── 落地向量与镜像摘要（登记 5.3） ─────────────────────────────

/// 崩溃状态序号 → 持久掩码：由枚举表现造，掩码按 `crash_facts::persisted_writes_of_state`（事实表与 GPU 判器共用的那一份，与崩溃点重放
/// `CrashStatePlan` 同一套数字次序）算，不走 `extract_flow_facts`——那一个给 GPU 判器抽事实，枚举写表多于 2048 次写的流（流三）它当场 panic。
/// 回调里崩溃点重放给的持久集合与这一份逐态比（`crash_facts_mask` 那一行），两份是两段代码。
pub(super) struct StateMasks {
    pub(super) recorded_write_count: usize,
    enumerated_write_count: usize,
    is_tearable: Vec<bool>,
    pub(super) torn_images: Vec<TornImageFact>,
    pub(super) segments: Vec<SegmentFact>,
    pub(super) state_count: u64,
}

impl StateMasks {
    pub(super) fn of(tables: &CrashStateEnumerationTables, segments: &[Vec<usize>]) -> Self {
        let write_index = |index: usize| u32::try_from(index).expect("写表下标装得进 u32");
        Self {
            recorded_write_count: tables.is_tearable_by_write.len(),
            enumerated_write_count: tables.enumerated_writes.len(),
            is_tearable: tables.is_tearable_by_write.clone(),
            torn_images: tables
                .torn_images
                .iter()
                .map(|row| TornImageFact {
                    write_index: write_index(row.write_index),
                    torn_image_index: write_index(row.torn_image_index),
                    replays: row
                        .replays
                        .iter()
                        .map(|(replay, replayed)| (write_index(*replay), write_index(*replayed)))
                        .collect(),
                })
                .collect(),
            segments: segments
                .iter()
                .zip(&tables.state_ranges_by_segment)
                .map(|(segment, states)| SegmentFact {
                    writes: segment.iter().map(|index| write_index(*index)).collect(),
                    states: states.clone(),
                    // 这一装置的流都不分片（C790 的分片只在崩溃放量按段登记的段上）
                    slice_start: None,
                })
                .collect(),
            state_count: tables.state_count,
        }
    }

    pub(super) fn persisted_writes_of_state(&self, ordinal: u64) -> Vec<bool> {
        persisted_writes_of_state(
            self.recorded_write_count,
            self.enumerated_write_count,
            &self.is_tearable,
            &self.torn_images,
            &self.segments,
            self.state_count,
            ordinal,
        )
    }
}

/// `extract_flow_facts` 数出的状态总数（S1 那一条）：枚举写表多于 GPU 掩码宽（2048）的流抽不了，交 `None`。
pub(super) fn flow_facts_state_count(
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
) -> Option<u64> {
    (tables.enumerated_writes.len() <= MAXIMUM_ENUMERATED_WRITES).then(|| {
        extract_flow_facts(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        )
        .state_count
    })
}

/// 把枚举写表上的持久集合折回按写序号的三值向量：原写持久记 `2`；原写没持久而它的撕裂镜像持久记 `1`；都没有记 `0`。
pub(super) fn landing_vector_of(
    persisted: &[bool],
    recorded_write_count: usize,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
) -> String {
    (0..recorded_write_count)
        .map(|write_index| {
            if persisted[write_index] {
                '2'
            } else if torn_image_index_by_write
                .get(&write_index)
                .is_some_and(|torn_image_index| persisted[*torn_image_index])
            {
                '1'
            } else {
                '0'
            }
        })
        .collect()
}

/// 镜像摘要的一个扇区：全零的不进摘要；进的依次是盘身份 u32、扇区号 u64、那 512 字节（小端）。
pub(super) fn append_sector_to_digest_message(
    message: &mut Vec<u8>,
    device: u32,
    sector: u64,
    sector_bytes: &[u8],
) {
    if sector_bytes.iter().all(|byte| *byte == 0) {
        return;
    }
    message.extend_from_slice(&device.to_le_bytes());
    message.extend_from_slice(&sector.to_le_bytes());
    message.extend_from_slice(sector_bytes);
}

/// 从 `CrashImage` 直接算镜像摘要：每块盘（身份升序）上可能不全为零的扇区——基镜像写过的扇区与这一态里持久了的带字节的写罩住的扇区——
/// 合成连续的区间按升序读（读走 `CrashImage` 自己的 `PoolReader::read`），全零的扇区跳过。
pub(super) fn crash_image_digest(image: &CrashImage<'_>) -> [u8; DIGEST_BYTES] {
    let mut message = Vec::new();
    let mut device_identities = image.device_identities();
    device_identities.sort_unstable();
    for device in device_identities {
        let mut sector_ranges: Vec<(u64, u64)> = image
            .base
            .devices
            .get(&device)
            .map(|sparse| {
                sparse
                    .written_sectors()
                    .keys()
                    .map(|sector| (*sector, sector + 1))
                    .collect()
            })
            .unwrap_or_default();
        for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
            let carries_bytes = match write.contents {
                WrittenContents::Bytes(_) => true,
                WrittenContents::Zeros { .. } => false,
            };
            if *is_persisted && carries_bytes && write.device == device {
                sector_ranges.push((
                    write.offset.0 / LOCAL_SECTOR_BYTES,
                    (write.offset.0 + write.length_in_bytes()).div_ceil(LOCAL_SECTOR_BYTES),
                ));
            }
        }
        for (first_sector, end_sector) in merged_sector_ranges(sector_ranges) {
            let bytes = PoolReader::read(
                image,
                device,
                DeviceOffsetInBytes(first_sector * LOCAL_SECTOR_BYTES),
                usize::try_from((end_sector - first_sector) * LOCAL_SECTOR_BYTES)
                    .expect("区间长装得进 usize"),
            )
            .expect("崩溃镜像在盘内、按扇区对齐的区间读得出");
            for (sector_offset, sector_bytes) in bytes
                .chunks_exact(usize::try_from(LOCAL_SECTOR_BYTES).expect("512"))
                .enumerate()
            {
                let sector = first_sector + u64::try_from(sector_offset).expect("扇区下标");
                append_sector_to_digest_message(&mut message, device.0, sector, sector_bytes);
            }
        }
    }
    sha256_digest(&message)
}

/// 扇区区间按起点排好、重叠或相接的合并。
fn merged_sector_ranges(mut sector_ranges: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
    sector_ranges.sort_unstable();
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (first_sector, end_sector) in sector_ranges {
        match merged.last_mut() {
            Some(last) if first_sector <= last.1 => last.1 = last.1.max(end_sector),
            Some(_) | None => merged.push((first_sector, end_sector)),
        }
    }
    merged
}

#[cfg(test)]
mod tests;
