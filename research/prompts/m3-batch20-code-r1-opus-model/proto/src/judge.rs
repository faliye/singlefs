//! 一份镜像上池级 checker 的判定（六条删除线相关的与全部违例），加两种 journal 政策的 CPU 恢复结局与 GPU 恢复内核在这一版上会不会走
//! `judge_extent_node` 那一支（extent 树根零条目）。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::records::{TREE_KIND_EXTENT, TREE_KIND_INODE};
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::MemoryPool;

use crate::common_pointer_head_corruption::{newest_root, node_bytes, the_slot_of, unit_bytes_at};

pub const WATCHED: [&str; 6] = ["I-9.5", "I-3.4", "I-4.7", "I-4.6", "I-4.4", "I-9.6"];

/// 删除态扫描「一次都没判到」时报的四句不适用理由（`walk.rs` 的 `judge_deletion_state_of_the_versions` 末尾）与 I-4.4 恒报的那一句：
/// 不适用而理由不是这几句的，是某一版判不了（`part_of_the_range_not_judged`）。
pub fn is_a_never_judged_reason(invariant: &str, reason: &str) -> bool {
    match invariant {
        "I-9.5" => reason.starts_with("走读到的每一版树表里都没有 deleted_inodes 条目"),
        "I-3.4" => reason.starts_with("走读到的每一版树表里都没有记账树"),
        "I-4.7" => reason.starts_with("走读到的每一版里都没有处于删除中的 inode"),
        "I-4.6" => reason.starts_with("走读到的每一版 inode 树里都没有 nlink = 0 的记录"),
        "I-4.4" => true,
        _ => false,
    }
}

#[derive(Clone, Debug, Default)]
pub struct Judged {
    /// 六条各自的判定种类：H 成立、N 不适用、G 不适用而理由是某一版判不了、P 部分没判、V 违例。
    pub watched: Vec<(&'static str, char, String)>,
    pub violated: Vec<(&'static str, String)>,
}

pub fn kind(invariant: &str, verdict: &InvariantVerdict) -> char {
    match verdict {
        InvariantVerdict::Holds => 'H',
        InvariantVerdict::NotApplicable(reason) => {
            if is_a_never_judged_reason(invariant, reason) || invariant == "I-9.6" && !reason.contains("读不全") {
                'N'
            } else {
                'G'
            }
        }
        InvariantVerdict::PartiallyJudged(_) => 'P',
        InvariantVerdict::Violated(_) => 'V',
    }
}

pub fn detail(verdict: &InvariantVerdict) -> String {
    match verdict {
        InvariantVerdict::Holds => String::new(),
        InvariantVerdict::NotApplicable(reason) => reason.to_string(),
        InvariantVerdict::PartiallyJudged(unjudged) => unjudged.clone(),
        InvariantVerdict::Violated(detail) => detail.clone(),
    }
}

pub fn judge(image: &MemoryPool) -> Judged {
    let report = check_pool_image(image);
    let mut judged = Judged::default();
    for (invariant, verdict) in &report {
        if WATCHED.contains(invariant) {
            judged.watched.push((invariant, kind(invariant, verdict), detail(verdict)));
        }
        if let InvariantVerdict::Violated(text) = verdict {
            judged.violated.push((invariant, text.clone()));
        }
    }
    judged
}

impl Judged {
    pub fn watched_string(&self) -> String {
        self.watched.iter().map(|(invariant, kind, _)| format!("{invariant}={kind}")).collect::<Vec<_>>().join(",")
    }
    pub fn violated_names(&self) -> Vec<&'static str> {
        self.violated.iter().map(|(invariant, _)| *invariant).collect()
    }
    pub fn verdict_of(&self, invariant: &str) -> char {
        self.watched.iter().find(|(name, _, _)| *name == invariant).map_or('?', |(_, kind, _)| *kind)
    }
    pub fn detail_of(&self, invariant: &str) -> String {
        self.watched.iter().find(|(name, _, _)| *name == invariant).map_or(String::new(), |(_, _, text)| text.clone())
    }
}

/// CPU 恢复的结局种类：F 读出文件、Z 没有文件、X 失败（带失败的 Debug 头一段）。
pub fn outcome_class(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::FileRead { .. } => "F".to_string(),
        RecoveryOutcome::NoFile { .. } => "Z".to_string(),
        RecoveryOutcome::Failed { failure, .. } => {
            let text = format!("{failure:?}");
            format!("X({})", text.chars().take(90).collect::<String>())
        }
    }
}

#[derive(Clone, Debug)]
pub struct Recovered {
    pub consulted: String,
    pub ignored: String,
    /// 两遍恢复走的根都是最新那条根（发布之后的镜像上应当如此）。
    pub effective_is_newest: bool,
    /// 最新那条根的树表里 extent 树那一条：根指针全零、根节点的层级与条目数。
    pub extent_root: String,
    pub extent_root_entries: Option<usize>,
    /// inode 树根的层级与条目数（叶容器片数）。
    pub inode_root: String,
}

pub fn recovered(image: &MemoryPool) -> Recovered {
    let consulted = recover(image, JournalPolicy::Consult);
    let ignored = recover(image, JournalPolicy::Ignore);
    let root = newest_root(image);
    let key = (root.instance, root.checkpoint_txg);
    let effective_is_newest = consulted.effective_root == Some(key) && ignored.effective_root == Some(key);
    let (extent_root, extent_root_entries) = describe_tree_root(image, &root, TREE_KIND_EXTENT);
    let (inode_root, _) = describe_tree_root(image, &root, TREE_KIND_INODE);
    Recovered {
        consulted: outcome_class(&consulted.outcome),
        ignored: outcome_class(&ignored.outcome),
        effective_is_newest,
        extent_root,
        extent_root_entries,
        inode_root,
    }
}

fn describe_tree_root(image: &MemoryPool, root: &singlefs_core::root_record::RootRecord, kind: u16) -> (String, Option<usize>) {
    let Ok(tree_table) = parse_index_node(&unit_bytes_at(image, the_slot_of(&root.tree_table), node_bytes())) else {
        return ("tree_table_unparsable".to_string(), None);
    };
    let Some(entry) = tree_table.entries.iter().find(|entry| u16::from_le_bytes([entry[10], entry[11]]) == kind) else {
        return ("no_entry".to_string(), None);
    };
    let pointer = singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(entry, 14));
    if pointer == singlefs_core::pointer::NodePointer::empty_root() {
        return ("empty_root_pointer".to_string(), None);
    }
    match parse_index_node(&unit_bytes_at(image, pointer.locations[0].slot, node_bytes())) {
        Ok(node) => (format!("level{}_entries{}", node.level, node.entries.len()), Some(node.entries.len())),
        Err(_) => ("root_unparsable".to_string(), None),
    }
}
