//! 装置自己的枚举计划与撕裂镜像表（停机条款 S3、锚点 A1）。

use super::*;

// ───────────────────────────── 自己的枚举计划与撕裂镜像表（S3、A1） ─────────────────────────────

/// 一段怎么展开（本文件自己的一份，与 crates 的 `CrashPointReplaySegmentExpansion` 在 S3 里逐段对拍状态数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LocalExpansion {
    /// 不展开：只以整段持久进入后面的状态。
    Skipped,
    /// 全量：段内每次写各取它的几态、任意组合，去掉整段全持久。
    EveryCombination,
    /// 甲二：原地写各取几态、任意组合 × 单元写全不落或全落，去掉整段全落。
    InPlaceCombinationsWithUnitWritesNoneOrAll,
}

impl LocalExpansion {
    pub(super) fn crates_expansion(self) -> CrashPointReplaySegmentExpansion {
        match self {
            Self::Skipped => CrashPointReplaySegmentExpansion::NotExpanded,
            Self::EveryCombination => CrashPointReplaySegmentExpansion::EveryProperSubset,
            Self::InPlaceCombinationsWithUnitWritesNoneOrAll => {
                CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
            }
        }
    }
}

/// 一次写在一个崩溃状态里怎么落。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Landing {
    NotPersisted,
    Torn,
    Persisted,
}

/// 原地覆写的三条判据（D13（验证路线） 已定项 4「原地覆写（不是单元写、长于一个扇区、罩住的范围里原来有东西的写）」），
/// 本文件自己判：基镜像那一段有没有非零字节按 [`MemoryPool`] 读出来看，不碰 crates 的私有字段。
pub(super) fn is_in_place_overwrite(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    write_index: usize,
) -> bool {
    let write = &writes[write_index];
    let is_unit_write = match write.kind {
        StepKind::UnitWrite => true,
        StepKind::ZeroFill
        | StepKind::Discard
        | StepKind::JournalRecord
        | StepKind::RootRecordFua
        | StepKind::SystemConfigurationSlot
        | StepKind::Barrier => false,
    };
    if is_unit_write || write.length_in_bytes() <= LOCAL_SECTOR_BYTES {
        return false;
    }
    let write_end = write.offset.0 + write.length_in_bytes();
    let base_bytes = base.devices.get(&write.device).map(|device| {
        device.read(
            write.offset,
            usize::try_from(write.length_in_bytes()).expect("写长装得进 usize"),
        )
    });
    let base_holds_nonzero = base_bytes.is_some_and(|bytes| bytes.iter().any(|byte| *byte != 0));
    let earlier_overlapping_write_with_bytes = writes[..write_index].iter().any(|earlier| {
        let has_bytes = match earlier.contents {
            WrittenContents::Bytes(_) => true,
            WrittenContents::Zeros { .. } => false,
        };
        has_bytes
            && earlier.device == write.device
            && earlier.offset.0 < write_end
            && write.offset.0 < earlier.offset.0 + earlier.length_in_bytes()
    });
    base_holds_nonzero || earlier_overlapping_write_with_bytes
}

/// 撕裂镜像的字节：新旧不同的那一截前一半新、后一半旧（D13（验证路线） 已定项 4「按字节撕」），本文件自己的一份。
fn torn_bytes(old: &[u8], new: &WrittenContents) -> Vec<u8> {
    let length = usize::try_from(new.length_in_bytes()).expect("写长装得进 usize");
    let mut new_bytes = vec![0u8; length];
    new.copy_range_into(0, &mut new_bytes);
    let differing: Vec<usize> = (0..length)
        .filter(|index| old[*index] != new_bytes[*index])
        .collect();
    let (Some(first_differing), Some(last_differing)) = (differing.first(), differing.last())
    else {
        return new_bytes;
    };
    let first_old_byte = first_differing + (last_differing - first_differing).div_ceil(2);
    let mut torn = new_bytes;
    torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);
    torn
}

/// 枚举用的写表：录制流的写在前，每个原地覆写接一条撕裂镜像、再接同段里之后与它重叠的写的重放。
pub(super) struct TornWriteTable {
    pub(super) writes: Vec<RetainedWrite>,
    pub(super) is_in_place_overwrite: Vec<bool>,
    /// 原地覆写（录制流下标）→（撕裂镜像的下标，[(重放的下标, 被重放的录制流下标)]）。
    pub(super) torn_image_by_write: BTreeMap<usize, (usize, Vec<(usize, usize)>)>,
    pub(super) replay_count: usize,
}

impl TornWriteTable {
    pub(super) fn of(stream: &PreparedStream) -> Self {
        let recorded = &stream.writes;
        let is_in_place_overwrite: Vec<bool> = (0..recorded.len())
            .map(|write_index| is_in_place_overwrite(&stream.base, recorded, write_index))
            .collect();
        let mut segment_of_write = BTreeMap::new();
        for (segment_index, segment) in stream.segments.iter().enumerate() {
            for write_index in segment {
                segment_of_write.insert(*write_index, segment_index);
            }
        }
        let mut writes = recorded.clone();
        let mut torn_image_by_write = BTreeMap::new();
        let mut replay_count = 0;
        let mut image_before = stream.base.clone();
        let mut next_to_apply = 0usize;
        for write_index in (0..recorded.len()).filter(|index| is_in_place_overwrite[*index]) {
            image_before.apply_writes(&recorded[next_to_apply..write_index]);
            next_to_apply = write_index;
            let write = &recorded[write_index];
            let length = usize::try_from(write.length_in_bytes()).expect("写长");
            let old = image_before
                .devices
                .get(&write.device)
                .expect("写落在池里的盘上")
                .read(write.offset, length);
            let torn_index = writes.len();
            writes.push(RetainedWrite {
                device: write.device,
                kind: write.kind,
                is_force_unit_access: write.is_force_unit_access,
                offset: write.offset,
                contents: WrittenContents::Bytes(torn_bytes(&old, &write.contents)),
            });
            let write_end = write.offset.0 + write.length_in_bytes();
            let later_overlapping: Vec<usize> = segment_of_write
                .get(&write_index)
                .map(|segment_index| {
                    stream.segments[*segment_index]
                        .iter()
                        .copied()
                        .filter(|later_index| {
                            let later = &recorded[*later_index];
                            *later_index > write_index
                                && later.device == write.device
                                && later.offset.0 < write_end
                                && write.offset.0 < later.offset.0 + later.length_in_bytes()
                        })
                        .collect()
                })
                .unwrap_or_default();
            let replays: Vec<(usize, usize)> = later_overlapping
                .into_iter()
                .map(|replayed| {
                    writes.push(recorded[replayed].clone());
                    replay_count += 1;
                    (writes.len() - 1, replayed)
                })
                .collect();
            torn_image_by_write.insert(write_index, (torn_index, replays));
        }
        Self {
            writes,
            is_in_place_overwrite,
            torn_image_by_write,
            replay_count,
        }
    }

    /// 掩码宽 W：带撕裂镜像的写表长度（A5：录制流写数 + 取三态的写数 + 重放数）。
    pub(super) fn mask_width(&self) -> usize {
        self.writes.len()
    }

    fn choices_of(&self, write_index: usize) -> u64 {
        if self.is_in_place_overwrite[write_index] {
            3
        } else {
            2
        }
    }

    fn landing_of_digit(&self, write_index: usize, digit: u64) -> Landing {
        match (self.is_in_place_overwrite[write_index], digit) {
            (false, 0) | (true, 0) => Landing::NotPersisted,
            (false, 1) | (true, 2) => Landing::Persisted,
            (true, 1) => Landing::Torn,
            (_, out_of_range) => unreachable!("数字 {out_of_range} 超出这次写的态数"),
        }
    }

    fn combinations_of(&self, write_indexes: &[usize]) -> u64 {
        write_indexes
            .iter()
            .map(|write_index| self.choices_of(*write_index))
            .product()
    }
}

/// 一段里的原地写与单元写（段内次序）。
fn in_place_and_unit_writes(
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    segment
        .iter()
        .copied()
        .partition(|write_index| match writes[*write_index].kind {
            StepKind::UnitWrite => false,
            StepKind::ZeroFill
            | StepKind::Discard
            | StepKind::SystemConfigurationSlot
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::Barrier => true,
        })
}

/// 自己的枚举计划：序号 → 段 → 段内混合进制（段里第一次写是最低位），最后可选全部持久那一个。
pub(super) struct LocalPlan<'plan> {
    pub(super) stream: &'plan PreparedStream,
    pub(super) table: &'plan TornWriteTable,
    pub(super) expansion_by_segment: Vec<LocalExpansion>,
    pub(super) first_ordinal_by_segment: Vec<u64>,
    pub(super) count_by_segment: Vec<u64>,
    /// 各段展开出来的状态数之和（不含全部持久那一个）。
    pub(super) expanded_state_count: u64,
}

impl<'plan> LocalPlan<'plan> {
    pub(super) fn new(
        stream: &'plan PreparedStream,
        table: &'plan TornWriteTable,
        expansion_by_segment: Vec<LocalExpansion>,
    ) -> Self {
        let count_by_segment: Vec<u64> = stream
            .segments
            .iter()
            .zip(&expansion_by_segment)
            .map(|(segment, expansion)| {
                Self::segment_state_count(stream, table, segment, *expansion)
            })
            .collect();
        let mut first_ordinal_by_segment = Vec::new();
        let mut next = 0u64;
        for count in &count_by_segment {
            first_ordinal_by_segment.push(next);
            next += count;
        }
        Self {
            stream,
            table,
            expansion_by_segment,
            first_ordinal_by_segment,
            count_by_segment,
            expanded_state_count: next,
        }
    }

    fn segment_state_count(
        stream: &PreparedStream,
        table: &TornWriteTable,
        segment: &[usize],
        expansion: LocalExpansion,
    ) -> u64 {
        match expansion {
            LocalExpansion::Skipped => 0,
            LocalExpansion::EveryCombination => table.combinations_of(segment) - 1,
            LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll => {
                let (in_place, unit_writes) = in_place_and_unit_writes(&stream.writes, segment);
                let unit_choices = if unit_writes.is_empty() { 1 } else { 2 };
                table.combinations_of(&in_place) * unit_choices - 1
            }
        }
    }

    /// 序号落在哪一段；等于段数说明是全部持久那一个。
    pub(super) fn segment_of(&self, ordinal: u64) -> usize {
        self.first_ordinal_by_segment
            .iter()
            .zip(&self.count_by_segment)
            .position(|(first, count)| ordinal >= *first && ordinal < first + count)
            .unwrap_or(self.stream.segments.len())
    }

    fn assign(&self, landings: &mut [Landing], ordinal_within: u64, write_indexes: &[usize]) {
        let mut remaining = ordinal_within;
        for write_index in write_indexes {
            let choices = self.table.choices_of(*write_index);
            landings[*write_index] = self
                .table
                .landing_of_digit(*write_index, remaining % choices);
            remaining /= choices;
        }
        assert_eq!(
            remaining, 0,
            "段内序号 {ordinal_within} 超出这几次写的组合数"
        );
    }

    /// 这个状态的持久集合，按枚举用的写表给。
    pub(super) fn persisted_of(&self, ordinal: u64) -> Vec<bool> {
        let recorded_count = self.stream.writes.len();
        let mut landings = vec![Landing::NotPersisted; recorded_count];
        let segment_index = self.segment_of(ordinal);
        for segment in &self.stream.segments[..segment_index] {
            for write_index in segment {
                landings[*write_index] = Landing::Persisted;
            }
        }
        if let Some(segment) = self.stream.segments.get(segment_index) {
            let within = ordinal - self.first_ordinal_by_segment[segment_index];
            match self.expansion_by_segment[segment_index] {
                LocalExpansion::Skipped => unreachable!("不展开的段没有状态"),
                LocalExpansion::EveryCombination => self.assign(&mut landings, within, segment),
                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll => {
                    let (in_place, unit_writes) =
                        in_place_and_unit_writes(&self.stream.writes, segment);
                    if unit_writes.is_empty() {
                        self.assign(&mut landings, within, &in_place);
                    } else {
                        self.assign(&mut landings, within / 2, &in_place);
                        if within % 2 == 1 {
                            for write_index in &unit_writes {
                                landings[*write_index] = Landing::Persisted;
                            }
                        }
                    }
                }
            }
        }
        let mut persisted = vec![false; self.table.mask_width()];
        for (write_index, landing) in landings.iter().enumerate() {
            persisted[write_index] = *landing == Landing::Persisted;
        }
        for (write_index, (torn_index, replays)) in &self.table.torn_image_by_write {
            if landings[*write_index] == Landing::Torn {
                persisted[*torn_index] = true;
                for (replay_index, replayed) in replays {
                    persisted[*replay_index] = landings[*replayed] == Landing::Persisted;
                }
            }
        }
        persisted
    }
}

/// A1 的公式：3^m · 2^(n−m) − 1。
pub(super) fn formula_segment_state_count(writes_in_segment: u64, in_place_overwrites: u64) -> u64 {
    3u64.pow(u32::try_from(in_place_overwrites).expect("m"))
        * 2u64.pow(u32::try_from(writes_in_segment - in_place_overwrites).expect("n − m"))
        - 1
}

#[cfg(test)]
mod tests;
