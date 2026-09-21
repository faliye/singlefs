//! 把录制流切成段（D13（验证路线） 已定项 4：屏障与 FUA 写切段，段内任意整写子集），并按
//! `layout/01-first-txn.md` 八那张登记表的写法报出段序列与每段的步骤种类多重集（门禁 52 号比对的形态）。
//!
//! 步骤种类只有五种（D17（实现分层与第三方管道） 已定项 2）：写单元、写 journal 记录、根槽 FUA 写、系统配置槽原地覆写、屏障。
//! 一条写属于哪一种，由它的落点决定（第一版几何写死：系统配置槽 / 根环区域 / journal 环 / 单元区）。

use std::collections::BTreeMap;

use singlefs_format::{JOURNAL_RING_START_SLOT, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE};

use crate::{RecordedOperation, RecordedOperationKind};
use singlefs_core::root_ring::{region_start, ring_end};

/// 提交步骤的封闭枚举，`match` 不写通配臂；声明序就是种类串的规范序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StepKind {
    UnitWrite,
    JournalRecord,
    RootRecordFua,
    SystemConfigurationSlot,
    Barrier,
}

impl StepKind {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            StepKind::UnitWrite => "unit_write",
            StepKind::JournalRecord => "journal_record",
            StepKind::RootRecordFua => "root_record_fua",
            StepKind::SystemConfigurationSlot => "superblock_slot",
            StepKind::Barrier => "barrier",
        }
    }
}

/// 第一版的固定几何：系统配置两槽从偏移 0 起按槽距排；根环从 1 MiB 起到 `ring_end`；journal 环从槽 1024 起；之后是单元区。
#[derive(Clone, Copy, Debug)]
pub struct FixedGeometry {
    pub fixed_structure_slot_spacing: u32,
    pub journal_ring_bytes: u64,
}

impl FixedGeometry {
    #[must_use]
    pub fn classify(&self, operation: &RecordedOperation) -> StepKind {
        match operation.kind {
            RecordedOperationKind::Barrier => StepKind::Barrier,
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                let offset = operation.offset.0;
                let system_configuration_end = SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                    * u64::from(self.fixed_structure_slot_spacing);
                let journal_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
                let journal_end = journal_start + self.journal_ring_bytes;
                if offset < system_configuration_end {
                    StepKind::SystemConfigurationSlot
                } else if offset >= region_start(0).0
                    && offset < ring_end(self.fixed_structure_slot_spacing)
                {
                    StepKind::RootRecordFua
                } else if offset >= journal_start && offset < journal_end {
                    StepKind::JournalRecord
                } else {
                    StepKind::UnitWrite
                }
            }
        }
    }
}

/// 切段（与 E142（第一个事务的干跑） 的切法同一条规则，产物 `name=segments` 就是按它切的）：
/// 屏障关掉它之前那一段（屏障算在被关掉的那一段里）；段里还一个写都没有时（流首那道屏障）它并进即将开始的那一段；
/// FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）；
/// 流尾只有屏障没有写的那一串并进上一段——录到的每一步都恰好落在一个段里。
#[must_use]
pub fn split_into_segments(
    operations: &[RecordedOperation],
    geometry: &FixedGeometry,
) -> Vec<Vec<StepKind>> {
    let mut segments: Vec<Vec<StepKind>> = Vec::new();
    let mut current: Vec<StepKind> = Vec::new();
    let mut writes_in_current: usize = 0;
    for operation in operations {
        current.push(geometry.classify(operation));
        match operation.kind {
            RecordedOperationKind::Barrier => {
                if writes_in_current > 0 {
                    segments.push(std::mem::take(&mut current));
                    writes_in_current = 0;
                }
            }
            RecordedOperationKind::WriteForceUnitAccess => {
                segments.push(std::mem::take(&mut current));
                writes_in_current = 0;
            }
            RecordedOperationKind::Write => writes_in_current += 1,
        }
    }
    if !current.is_empty() {
        match (writes_in_current, segments.last_mut()) {
            (0, Some(last_segment)) => last_segment.append(&mut current),
            (_, _) => segments.push(current),
        }
    }
    segments
}

/// 登记表的「段序列」：每段的写数（屏障不算写），用 `+` 连起来，例如 `4+1+1+1+4`。
#[must_use]
pub fn segment_sizes_text(segments: &[Vec<StepKind>]) -> String {
    segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .filter(|kind| **kind != StepKind::Barrier)
                .count()
                .to_string()
        })
        .collect::<Vec<String>>()
        .join("+")
}

/// E77（发布的持久顺序） 的闭式：崩溃状态数 = 1 + Σ(2^|段| − 1)，|段| 按写数算。
#[must_use]
pub fn closed_form_state_count(segments: &[Vec<StepKind>]) -> u64 {
    1 + segments
        .iter()
        .map(|segment| {
            let writes = segment
                .iter()
                .filter(|kind| **kind != StepKind::Barrier)
                .count();
            (1u64 << writes) - 1
        })
        .sum::<u64>()
}

/// 登记表的「种类」串：每段一个多重集，按枚举声明序排成规范形、`×N` 计数，段之间用 `|` 隔开，
/// 例如 `[unit_write×4,barrier]|[root_record_fua]`。规范序是为了两段只要多重集相同、文字就一模一样。
#[must_use]
pub fn segment_kinds_text(segments: &[Vec<StepKind>]) -> String {
    segments
        .iter()
        .map(|segment| {
            let mut counts: BTreeMap<StepKind, usize> = BTreeMap::new();
            for kind in segment {
                *counts.entry(*kind).or_insert(0) += 1;
            }
            let parts: Vec<String> = counts
                .into_iter()
                .map(|(kind, count)| {
                    if count == 1 {
                        kind.name().to_string()
                    } else {
                        format!("{}×{count}", kind.name())
                    }
                })
                .collect();
            format!("[{}]", parts.join(","))
        })
        .collect::<Vec<String>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};

    fn operation(kind: RecordedOperationKind, offset: u64) -> RecordedOperation {
        RecordedOperation {
            device: DeviceIdentity(0),
            kind,
            offset: DeviceOffsetInBytes(offset),
            length: 512,
            content_hash: 0,
        }
    }

    #[test]
    fn writes_classify_by_their_landing_zone() {
        let geometry = FixedGeometry {
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: 768 << 20,
        };
        assert_eq!(
            geometry.classify(&operation(RecordedOperationKind::Write, 4096)),
            StepKind::SystemConfigurationSlot
        );
        assert_eq!(
            geometry.classify(&operation(
                RecordedOperationKind::WriteForceUnitAccess,
                1 << 20
            )),
            StepKind::RootRecordFua
        );
        assert_eq!(
            geometry.classify(&operation(RecordedOperationKind::Write, 16 << 20)),
            StepKind::JournalRecord
        );
        assert_eq!(
            geometry.classify(&operation(RecordedOperationKind::Write, 50176 * 16384)),
            StepKind::UnitWrite
        );
        assert_eq!(
            geometry.classify(&operation(RecordedOperationKind::Barrier, 0)),
            StepKind::Barrier
        );
    }

    #[test]
    fn barriers_close_segments_a_leading_barrier_folds_forward_and_fua_closes_its_own_segment() {
        let geometry = FixedGeometry {
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: 768 << 20,
        };
        let unit = 50176 * 16384;
        let journal = 1024 * 16384;
        let stream = vec![
            operation(RecordedOperationKind::Write, unit),
            operation(RecordedOperationKind::Write, unit + 32768),
            operation(RecordedOperationKind::Barrier, 0),
            operation(RecordedOperationKind::WriteForceUnitAccess, 1 << 20),
            operation(RecordedOperationKind::Write, 0),
            operation(RecordedOperationKind::Write, 4096),
            operation(RecordedOperationKind::Barrier, 0),
        ];
        let segments = split_into_segments(&stream, &geometry);
        assert_eq!(segment_sizes_text(&segments), "2+1+2");
        assert_eq!(
            segment_kinds_text(&segments),
            "[unit_write×2,barrier]|[root_record_fua]|[superblock_slot×2,barrier]"
        );
        assert_eq!(closed_form_state_count(&segments), 1 + 3 + 1 + 3);

        // 暖机那一段的形：流首屏障并进后面那一段，两道屏障都算在里面（E142 `path=warm_up` 的第一段）。
        let warm_up = vec![
            operation(RecordedOperationKind::Barrier, 0),
            operation(RecordedOperationKind::Write, journal),
            operation(RecordedOperationKind::Write, journal),
            operation(RecordedOperationKind::Barrier, 0),
            operation(RecordedOperationKind::WriteForceUnitAccess, 4 << 20),
            operation(RecordedOperationKind::Write, 0),
            operation(RecordedOperationKind::Write, 0),
        ];
        let warm_up_segments = split_into_segments(&warm_up, &geometry);
        assert_eq!(segment_sizes_text(&warm_up_segments), "2+1+2");
        assert_eq!(
            segment_kinds_text(&warm_up_segments),
            "[journal_record×2,barrier×2]|[root_record_fua]|[superblock_slot×2]"
        );

        // 种类串按枚举声明序，不按出现序：系统配置槽写先于单元写发出，串里仍是 unit_write 在前。
        let mixed = vec![
            operation(RecordedOperationKind::Write, 0),
            operation(RecordedOperationKind::Write, 4096),
            operation(RecordedOperationKind::Write, unit),
            operation(RecordedOperationKind::Barrier, 0),
        ];
        assert_eq!(
            segment_kinds_text(&split_into_segments(&mixed, &geometry)),
            "[unit_write,superblock_slot×2,barrier]"
        );

        // FUA 不替它前面的普通写做持久：普通写与 FUA 同段。
        let fua_after_plain = vec![
            operation(RecordedOperationKind::Write, unit),
            operation(RecordedOperationKind::WriteForceUnitAccess, 1 << 20),
        ];
        assert_eq!(
            segment_sizes_text(&split_into_segments(&fua_after_plain, &geometry)),
            "2"
        );

        // 流尾只有屏障：并进上一段，每一步恰好落在一个段里。
        let trailing_barrier = vec![
            operation(RecordedOperationKind::Write, unit),
            operation(RecordedOperationKind::Barrier, 0),
            operation(RecordedOperationKind::Barrier, 0),
        ];
        let trailing_segments = split_into_segments(&trailing_barrier, &geometry);
        assert_eq!(trailing_segments.len(), 1);
        assert_eq!(trailing_segments[0].len(), 3);
    }
}
