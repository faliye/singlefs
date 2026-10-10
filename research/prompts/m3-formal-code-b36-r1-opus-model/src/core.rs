//! core 这一侧：照恢复的公开入口（择系统配置、择根、扫环、取前缀并施加）算两样——
//! ① 施加之后那一版（最后一次施加的发布的末条，`replay_journal` 交回的根）；
//! ② 施加的**第一次**发布：`replay_journal` 报 `prefix_applied` 条，施加的是水位之上（同实例、txg 大于所选根）按 (实例, 计数器) 键序的前
//!    `prefix_applied` 条（`singlefs_replay_prefix::replay_core` 的不变式「admitted == candidates_before」：前缀从水位之上第一条起、遇第一条
//!    不收的就停），其中第一条带末条标志的就是第一次发布的末条；它的新根段照 `replay_journal` 拼新根那一段拼成根、再摊成 364 字节。
//! 七段对照与第 35 批攻方原型同一张（`craft::SEGMENT_FROM_ROOT`）。
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::recovery::{choose_root, choose_system_configuration, replay_journal, scan_journal, PoolReader};
use singlefs_core::root_record::RootRecord;

use crate::craft::SEGMENT_FROM_ROOT;

/// 一张镜像上 core 的读法：所选根、施加的第一次发布、施加之后那一版（施加了至少一次发布才有）、施加几条与点名验不过几条；replay 报错时记原文。
#[derive(Clone, Debug)]
pub struct CoreView {
    pub chosen: (u32, u64),
    pub first: Option<((u32, u64), Vec<u8>)>,
    pub last: Option<((u32, u64), Vec<u8>)>,
    pub prefix_applied: usize,
    pub verification_failed: usize,
    pub refused: Option<String>,
}

pub fn segment_of_root(root: &RootRecord) -> Vec<u8> {
    let bytes = root.to_slot(512);
    let mut segment = vec![0u8; 364];
    for (s, r) in SEGMENT_FROM_ROOT.iter().cloned() {
        segment[s].copy_from_slice(&bytes[r]);
    }
    segment
}

pub fn core_view(reader: &dyn PoolReader) -> Option<CoreView> {
    let system_configuration = choose_system_configuration(reader).ok()?;
    let root = choose_root(reader, &system_configuration)?;
    let chosen = (root.instance.0, root.checkpoint_txg.0);
    let records = scan_journal(reader, &system_configuration);
    match replay_journal(reader, &root, system_configuration.immutable.sizes.journal_ring_bytes, &records, true) {
        Ok((report, effective)) => {
            let key = (effective.instance.0, effective.checkpoint_txg.0);
            let first = records
                .values()
                .filter(|r| r.instance == root.instance && r.checkpoint_txg > root.checkpoint_txg)
                .take(report.prefix_applied)
                .find(|r| r.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish)
                .map(|record| {
                    let rebuilt = RootRecord {
                        filesystem_identifier: root.filesystem_identifier,
                        unmount_marker: record.new_unmount_marker,
                        instance: record.instance,
                        checkpoint_txg: record.checkpoint_txg,
                        tree_table: record.new_tree_table,
                        tree_identifier_watermark: record.new_tree_identifier_watermark,
                        rollback_floor: record.new_rollback_floor,
                        instance_table: record.new_instance_table,
                        mapping_root: record.new_mapping_root,
                        allocation_record_tree_root: record.new_allocation_record_tree_root,
                    };
                    ((record.instance.0, record.checkpoint_txg.0), segment_of_root(&rebuilt))
                });
            assert_eq!(first.is_some(), report.prefix_applied > 0, "施加了记录就有第一次发布的末条");
            Some(CoreView {
                chosen,
                first,
                last: (report.prefix_applied > 0).then(|| (key, segment_of_root(&effective))),
                prefix_applied: report.prefix_applied,
                verification_failed: report.verification_failed,
                refused: None,
            })
        }
        Err(failure) => Some(CoreView { chosen, first: None, last: None, prefix_applied: 0, verification_failed: 0, refused: Some(format!("{failure:?}")) }),
    }
}

/// checker 认的一版（实例、txg、新根段）对 core 的一版：分类。
pub fn classify(checker: Option<(u32, u64, &[u8])>, core_side: Option<&((u32, u64), Vec<u8>)>, core: &Option<CoreView>) -> &'static str {
    let Some(core) = core else {
        return if checker.is_some() { "checker-only(core择不出根)" } else { "none(core择不出根)" };
    };
    if core.refused.is_some() {
        return if checker.is_some() { "checker-only(core拒)" } else { "none(core拒)" };
    }
    match (checker, core_side) {
        (None, None) => "none",
        (Some(_), None) => "checker-only",
        (None, Some(_)) => "core-only",
        (Some((instance, txg, segment)), Some((key, core_segment))) => {
            if (instance, txg) != *key {
                "different-version"
            } else if segment == core_segment.as_slice() {
                "same-bytes"
            } else {
                "same-version-different-bytes"
            }
        }
    }
}
