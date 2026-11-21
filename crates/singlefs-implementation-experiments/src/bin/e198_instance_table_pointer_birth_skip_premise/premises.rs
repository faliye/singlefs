use super::*;

// ============================================================================
// 八、前提 P1–P3（只读账本，登记第一节「前提」、第六节 Q1–Q3）。
// ============================================================================

#[derive(Clone, Debug, Default)]
pub(super) struct PremiseCounts {
    /// P1：写过行的实例（实例代号 ≥ 2）第一版的指针出生 txg 不等于它自己的 txg。
    pub(super) first_version_violations: Vec<String>,
    /// PC-P1：实例 1 的第一版报不成立的次数（不计入 Q2）。
    pub(super) instance_one_first_version_mismatches: u64,
    /// P2：同实例相邻两个存在过的版本，后一版的出生 txg 既不等于前一版的、又不是「清扫发布、等于自己的 txg」。
    pub(super) adjacent_version_violations: Vec<String>,
    /// PC-P2：清扫换指针的次数。
    pub(super) sweep_pointer_changes: u64,
    /// P3：出生 txg 大于自己 txg 的版本（实例 1 也算）。
    pub(super) birth_after_the_txg_violations: Vec<String>,
}

pub(super) fn premise_counts(ledger: &Ledger) -> PremiseCounts {
    let mut counts = PremiseCounts::default();
    let mut by_instance: BTreeMap<u32, Vec<(VersionKey, &LedgerVersion)>> = BTreeMap::new();
    for (key, version) in &ledger.versions {
        by_instance
            .entry(key.instance)
            .or_default()
            .push((*key, version));
        if version.pointer_birth_txg > key.txg {
            counts.birth_after_the_txg_violations.push(format!(
                "{}（{}）出生 txg {}",
                key.text(),
                version.kind.name(),
                version.pointer_birth_txg
            ));
        }
    }
    for (instance, mut versions) in by_instance {
        versions.sort_by_key(|(_, version)| version.publish_order);
        if instance == 0 {
            continue;
        }
        if let Some((first_key, first)) = versions.first() {
            let mismatch = first.pointer_birth_txg != first_key.txg;
            if instance == 1 {
                counts.instance_one_first_version_mismatches += u64::from(mismatch);
            } else if mismatch {
                counts.first_version_violations.push(format!(
                    "{}（{}）出生 txg {}",
                    first_key.text(),
                    first.kind.name(),
                    first.pointer_birth_txg
                ));
            }
        }
        if instance == 1 {
            continue;
        }
        let existed: Vec<&(VersionKey, &LedgerVersion)> = versions
            .iter()
            .filter(|(_, version)| version.existed())
            .collect();
        for adjacent in existed.windows(2) {
            let (previous_key, previous) = adjacent[0];
            let (next_key, next) = adjacent[1];
            if next.kind == PublishKind::Sweep && next.pointer_birth_txg == next_key.txg {
                counts.sweep_pointer_changes += 1;
            } else if next.pointer_birth_txg != previous.pointer_birth_txg {
                counts.adjacent_version_violations.push(format!(
                    "{} → {}（{}）出生 txg {} → {}",
                    previous_key.text(),
                    next_key.text(),
                    next.kind.name(),
                    previous.pointer_birth_txg,
                    next.pointer_birth_txg
                ));
            }
        }
    }
    counts
}

#[cfg(test)]
mod tests;
