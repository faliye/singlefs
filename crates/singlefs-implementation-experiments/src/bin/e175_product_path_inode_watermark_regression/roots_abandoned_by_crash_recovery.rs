//! S3：被崩溃恢复抛弃的根两条路（路 1 现行实例表、路 2 步账），与旧出生代还在不在候选集里。
use super::*;

/// 一条根的身份：(实例代号, checkpoint_txg)。
pub(super) type RootIdentity = (u32, u64);

/// S1 对一条根读到的那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum VersionFacts {
    /// 树表 0 条：没有 inode 树，也没有记账水位。
    WithoutTrees,
    WithTrees {
        watermark: u64,
        inode_births: Vec<(u64, u64)>,
    },
    /// 重建报错（登记第六节 S1：记「读不出」另数，不当成没有记录）。
    Unreadable(String),
}

impl RootFacts {
    pub(super) fn watermark(&self) -> Option<u64> {
        match &self.version {
            VersionFacts::WithTrees { watermark, .. } => Some(*watermark),
            VersionFacts::WithoutTrees | VersionFacts::Unreadable(_) => None,
        }
    }

    pub(super) fn inode_births(&self) -> &[(u64, u64)] {
        match &self.version {
            VersionFacts::WithTrees { inode_births, .. } => inode_births,
            VersionFacts::WithoutTrees | VersionFacts::Unreadable(_) => &[],
        }
    }
}

pub(super) fn root_identity(root: &RootRecord) -> RootIdentity {
    (root.instance.0, root.checkpoint_txg.0)
}

/// 重建用的那条记录：同实例、同 txg、带「本次发布末条」标志的；没有就取环里计数器最大的那条顶着
/// （与 `mount::record_standing_for_the_version` 同一个取法，重建出来的版本只用它的 jsn 与事务号，这里都不用）。
fn record_standing_for(
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    root: &RootRecord,
) -> Option<JournalRecord> {
    records
        .values()
        .filter(|record| {
            record.instance == root.instance
                && record.checkpoint_txg == root.checkpoint_txg
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
}

pub(super) fn version_facts_of(
    image: &MemoryPool,
    root: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> VersionFacts {
    match rebuild_version(image, root, record_standing_for(records, root)) {
        Ok(RebuiltVersion::WithoutFile) => VersionFacts::WithoutTrees,
        Ok(RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version)) => {
            let mut inode_births: Vec<(u64, u64)> = version
                .inode_leaf_containers
                .iter()
                .flat_map(|container| container.contents.records.iter())
                .map(|record| (record.inode, record.object_birth.0))
                .collect();
            inode_births.sort_unstable();
            VersionFacts::WithTrees {
                watermark: version.inode_number_watermark(),
                inode_births,
            }
        }
        Err(failure) => VersionFacts::Unreadable(format!("{failure:?}").replace(' ', "")),
    }
}

/// S3 路 1：一条根按现行实例表判是不是被崩溃恢复抛弃的。
pub(super) fn abandoned_by_the_table(identity: RootIdentity, table: &InstanceTableRecords) -> bool {
    publish_is_abandoned_by_the_instance_table(
        InstanceGeneration(identity.0),
        CheckpointTxg(identity.1),
        table,
    )
}

/// S3 路 2 的步账：每次挂载做成，新实例接的那一版（模型记的所选根 = 那一步之前模型根环里最新的那条）之后、
/// 同一个旧实例里 txg 更大的根记为被抛弃。键是旧实例，值是接的那一版的 txg。
pub(super) fn abandoned_by_the_step_ledger(
    identity: RootIdentity,
    rows: &BTreeMap<u32, u64>,
) -> bool {
    rows.get(&identity.0)
        .is_some_and(|selected_txg| identity.1 > *selected_txg)
}

/// Q9 的分类（登记第六节 Q9 行）：持有旧出生代的根里有一条这一步可读、不被抛弃、txg ≥ 现行 F ⇒ 还在。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OlderBirthStillInTheCandidateSet {
    Still,
    Gone,
}

pub(super) fn classify_older_birth_candidacy(
    holders: &[HolderAtThisStep],
    current_rollback_floor: u64,
    breakage: Breakage,
) -> OlderBirthStillInTheCandidateSet {
    if breakage.older_birth_always_still_present {
        return OlderBirthStillInTheCandidateSet::Still;
    }
    if holders.iter().any(|holder| {
        holder.readable_now && !holder.abandoned_now && holder.txg >= current_rollback_floor
    }) {
        OlderBirthStillInTheCandidateSet::Still
    } else {
        OlderBirthStillInTheCandidateSet::Gone
    }
}

#[cfg(test)]
mod tests;
