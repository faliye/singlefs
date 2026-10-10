//! 一份镜像（整池镜像或崩溃状态）上：池级 checker 的删除线几条判定，按种类归；GPU 恢复内核会走到的那几处形状（读冻结副本的 wgsl 推的判据，CPU 算期望）。
use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::bytes::ByteReader;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{readable_roots, recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_core::records::{TREE_KIND_EXTENT, TREE_KIND_INODE};
use singlefs_core::unit::parse_index_node;
use singlefs_format::{NODE_BYTES, SLOT_BYTES};

use crate::pool::parameters;

pub const WATCHED: [&str; 5] = ["I-9.5", "I-3.4", "I-4.7", "I-4.6", "I-9.6"];

/// 删除态扫描一次都没判到时那几句不适用理由（`walk.rs` 的 `judge_deletion_state_of_the_versions` 末尾）；I-9.6 的不适用只认「没有对象」那一类。
fn never_judged(invariant: &str, reason: &str) -> bool {
    match invariant {
        "I-9.5" => reason.starts_with("走读到的每一版树表里都没有 deleted_inodes 条目"),
        "I-3.4" => reason.starts_with("走读到的每一版树表里都没有记账树"),
        "I-4.7" => reason.starts_with("走读到的每一版里都没有处于删除中的 inode"),
        "I-4.6" => reason.starts_with("走读到的每一版 inode 树里都没有 nlink = 0 的记录"),
        "I-9.6" => !reason.contains("判不了") && !reason.contains("读不全"),
        _ => false,
    }
}

/// H 成立、N 不适用（一次都没判到）、G 不适用（某一版判不了）、P 部分没判、V 违例。
pub fn kind(invariant: &str, verdict: &InvariantVerdict) -> char {
    match verdict {
        InvariantVerdict::Holds => 'H',
        InvariantVerdict::NotApplicable(reason) => if never_judged(invariant, reason) { 'N' } else { 'G' },
        InvariantVerdict::PartiallyJudged(_) => 'P',
        InvariantVerdict::Violated(_) => 'V',
    }
}

pub struct Judged {
    pub watched: Vec<(&'static str, char, String)>,
    pub violated: Vec<(&'static str, String)>,
}

pub fn judge(reader: &dyn ImageReader) -> Judged {
    let report = check_pool_image(reader);
    let mut watched = Vec::new();
    let mut violated = Vec::new();
    for (invariant, verdict) in &report {
        let text = match verdict {
            InvariantVerdict::Holds => String::new(),
            InvariantVerdict::NotApplicable(reason) => reason.to_string(),
            InvariantVerdict::PartiallyJudged(text) | InvariantVerdict::Violated(text) => text.clone(),
        };
        if WATCHED.contains(invariant) {
            watched.push((*invariant, kind(invariant, verdict), text.clone()));
        }
        if let InvariantVerdict::Violated(_) = verdict {
            violated.push((*invariant, text));
        }
    }
    Judged { watched, violated }
}

pub fn outcome_class(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::FileRead { .. } => "F".to_string(),
        RecoveryOutcome::NoFile { .. } => "Z".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("X({})", format!("{failure:?}").chars().take(80).collect::<String>()),
    }
}

/// GPU 恢复内核（冻结副本 `crash_judge_common.wgsl` 的 `walk_to_file` 与 `read_extent_tree_of_the_first_file`）在 CPU 答 Z 的版上还会拒的形状，
/// 按 wgsl 那几句推：extent 树条目根指针全零（`have_extent` 不置、走读 37）；extent 上段根零条目而层级不是 0（`judge_extent_node`）；
/// 上段叶条目多于一条或不是 1 号（32）；inode 树根层级不是 1（44）。交回空串是 GPU 推得与 CPU 同答。
pub fn gpu_shape_divergence(reader: &dyn PoolReader, effective: Option<(u32, u64)>) -> String {
    let parameters = parameters();
    let Some((instance, txg)) = effective else { return String::new() };
    let roots = readable_roots(reader, &parameters.region_devices, &parameters.geometry, &parameters.filesystem_identifier);
    let Some(root) = roots.iter().find(|root| root.instance.0 == instance && root.checkpoint_txg.0 == txg) else {
        return "effective_root_is_applied_from_records".to_string();
    };
    let node_at = |pointer: &NodePointer| {
        let location = pointer.locations[0];
        reader
            .read(location.device, singlefs_core::address::DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES), usize::try_from(NODE_BYTES).expect("16384"))
            .and_then(|bytes| parse_index_node(&bytes).ok())
    };
    let Some(tree_table) = node_at(&root.tree_table) else { return "tree_table_unreadable".to_string() };
    if tree_table.entries.is_empty() {
        return String::new();
    }
    let mut out = Vec::new();
    for (kind, name) in [(TREE_KIND_EXTENT, "extent"), (TREE_KIND_INODE, "inode")] {
        let Some(entry) = tree_table.entries.iter().find(|entry| u16::from_le_bytes([entry[10], entry[11]]) == kind) else {
            out.push(format!("{name}:no_entry"));
            continue;
        };
        let pointer = NodePointer::read_from(&mut ByteReader::at(entry, 14));
        if pointer == NodePointer::empty_root() {
            out.push(format!("{name}:empty_root_pointer"));
            continue;
        }
        let Some(node) = node_at(&pointer) else {
            out.push(format!("{name}:root_unreadable"));
            continue;
        };
        if kind == TREE_KIND_EXTENT {
            if node.entries.is_empty() && node.level != 0 {
                out.push(format!("extent:zero_entries_at_level{}", node.level));
            }
            if node.level == 0 {
                let others: Vec<u64> = node
                    .entries
                    .iter()
                    .map(|entry| u64::from_le_bytes(entry[8..16].try_into().expect("8")))
                    .filter(|inode| *inode != 1)
                    .collect();
                if !others.is_empty() || node.entries.len() > 1 {
                    out.push(format!("extent:upper_leaf_entries{}_others{:?}", node.entries.len(), others));
                }
            }
        } else if node.level != 1 {
            out.push(format!("inode:root_level{}", node.level));
        }
    }
    out.join("|")
}

pub fn recovered(reader: &dyn PoolReader) -> (String, String, String) {
    let consulted = recover(reader, JournalPolicy::Consult);
    let ignored = recover(reader, JournalPolicy::Ignore);
    let classes = (outcome_class(&consulted.outcome), outcome_class(&ignored.outcome));
    let mut shapes = Vec::new();
    for report in [&consulted, &ignored] {
        if matches!(report.outcome, RecoveryOutcome::NoFile { .. } | RecoveryOutcome::FileRead { .. }) {
            let shape = gpu_shape_divergence(reader, report.effective_root.map(|(instance, txg)| (instance.0, txg.0)));
            if !shape.is_empty() {
                shapes.push(shape);
            }
        }
    }
    (classes.0, classes.1, shapes.join(";"))
}
