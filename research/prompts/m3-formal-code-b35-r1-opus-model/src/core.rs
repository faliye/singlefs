//! core 这一侧：照恢复的公开入口（择系统配置、择根、扫环、取前缀并施加）算出下一次挂载施加之后那一版的根，
//! 把它的七段拼成 364 字节新根段（D23 已定项 15 的对照，`craft::SEGMENT_FROM_ROOT`），与 checker 认的那一版逐字节比。
use singlefs_core::recovery::{choose_root, choose_system_configuration, replay_journal, scan_journal, PoolReader};
use singlefs_core::root_record::RootRecord;

use crate::craft::SEGMENT_FROM_ROOT;

/// 一张镜像上 core 的读法：所选根，与施加之后那一版（施加了至少一次发布才有）；replay 报错时记原文。
#[derive(Clone, Debug)]
pub struct CoreView {
    pub chosen: (u32, u64),
    pub applied: Option<((u32, u64), Vec<u8>, usize)>,
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
            Some(CoreView {
                chosen,
                applied: (key != chosen).then(|| (key, segment_of_root(&effective), report.prefix_applied)),
                refused: None,
            })
        }
        Err(failure) => Some(CoreView { chosen, applied: None, refused: Some(format!("{failure:?}")) }),
    }
}

/// checker 认的一版（实例、txg、新根段）对 core 施加出来的那一版：分类。
pub fn classify(checker: Option<(u32, u64, &[u8])>, core: &Option<CoreView>) -> &'static str {
    let Some(core) = core else {
        return if checker.is_some() { "checker-only(core择不出根)" } else { "none" };
    };
    if core.refused.is_some() {
        return if checker.is_some() { "checker-only(core拒)" } else { "none(core拒)" };
    }
    match (checker, &core.applied) {
        (None, None) => "none",
        (Some(_), None) => "checker-only",
        (None, Some(_)) => "core-only",
        (Some((instance, txg, segment)), Some((key, core_segment, _))) => {
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
