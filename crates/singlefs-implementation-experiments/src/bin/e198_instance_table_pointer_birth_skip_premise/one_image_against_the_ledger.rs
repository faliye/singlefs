use super::*;

// ============================================================================
// 七、一张镜像：两条臂的分类对上账本的真值、停机条款 S1、S3、S5 与（nrb、rs1 两组）今天的池级 checker。
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ArmCounts {
    pub(super) today_e1: u64,
    pub(super) today_e2: u64,
    pub(super) with_pointer_birth_e1: u64,
    pub(super) with_pointer_birth_e2: u64,
}

impl ArmCounts {
    pub(super) fn add(&mut self, other: ArmCounts) {
        self.today_e1 += other.today_e1;
        self.today_e2 += other.today_e2;
        self.with_pointer_birth_e1 += other.with_pointer_birth_e1;
        self.with_pointer_birth_e2 += other.with_pointer_birth_e2;
    }
}

/// 一个洞对上两条臂各错了哪一样：E1 = 认作取号跳过而真值不是直接接上；E2 = 真值直接接上而不认作取号跳过（登记第一节）。
/// 真值是「R 不在账本里」的不计。
fn arm_counts_of_the_pair(pair: &HolePair, truth: Truth) -> ArmCounts {
    let directly_bridged = match truth {
        Truth::DirectlyBridged => true,
        Truth::ExistedVersionsInBetween | Truth::PreviousNotOnTheChain => false,
        Truth::VersionMissingFromTheLedger => return ArmCounts::default(),
    };
    ArmCounts {
        today_e1: u64::from(pair.bridged_today && !directly_bridged),
        today_e2: u64::from(directly_bridged && !pair.bridged_today),
        with_pointer_birth_e1: u64::from(pair.bridged_with_the_pointer_birth && !directly_bridged),
        with_pointer_birth_e2: u64::from(directly_bridged && !pair.bridged_with_the_pointer_birth),
    }
}

/// 停机条款 S5 的读数：装置按 ①–④ 认出的「由记录施加出来」的版本，对账本里两支（被某次挂载施加过、根槽没落；根槽落过、后来被环盖掉或读坏，登记修订 3），
/// 两支都要账本推的行与根环占位让 ① ② ④ 成立（登记修订 4）、记录此刻还读得出的那几版。装置多认的按账本里那一版的根槽落没落过分两格
/// （落过的那一格是账本两支都没收进的形，单列给主 agent 看）。
#[derive(Clone, Debug, Default)]
pub(super) struct RecordsAppliedComparison {
    pub(super) apparatus_extra_with_a_root_that_landed_once: Vec<VersionKey>,
    pub(super) apparatus_extra_otherwise: Vec<VersionKey>,
    pub(super) apparatus_missing: Vec<VersionKey>,
}

impl RecordsAppliedComparison {
    pub(super) fn mismatch_count(&self) -> usize {
        self.apparatus_extra_with_a_root_that_landed_once.len()
            + self.apparatus_extra_otherwise.len()
            + self.apparatus_missing.len()
    }
}

#[derive(Clone, Debug)]
pub(super) struct ImageResult {
    pub(super) label: String,
    pub(super) step: u64,
    pub(super) reading: Result<ImageReadingOfTheJudgements, String>,
    pub(super) truths: Vec<Truth>,
    pub(super) counts: ArmCounts,
    pub(super) pointer_unreadable_pairs: u64,
    pub(super) ring_slot_mismatches: Vec<String>,
    pub(super) row_mismatch: Option<String>,
    pub(super) records_applied: RecordsAppliedComparison,
    pub(super) checker_verdicts: Option<(String, String)>,
    pub(super) corrupted_skipped: Vec<String>,
}

fn verdict_name(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "holds",
        InvariantVerdict::Violated(_) => "violated",
        InvariantVerdict::NotApplicable(_) => "not_applicable",
        InvariantVerdict::PartiallyJudged(_) => "partially_judged",
    }
}

/// S1：账本推的每个根环槽（写过的、没读坏的）住哪一版，对 `root_ring_the_ring_judgements_read` 逐槽交回的。
pub(super) fn ring_slot_mismatches(image: &MemoryPool, ledger: &Ledger) -> Vec<String> {
    let predicted: BTreeMap<(u64, u64), (u32, u64)> = ledger
        .ring_occupants
        .iter()
        .filter(|(ring_slot, _)| !ledger.corrupted_ring_slots.contains(*ring_slot))
        .map(|(ring_slot, key)| (*ring_slot, (key.instance, key.txg)))
        .collect();
    let Some(read) = root_ring_the_ring_judgements_read(image) else {
        return vec!["池级 checker 择不到系统配置，根环没读".to_string()];
    };
    let actual: BTreeMap<(u64, u64), (u32, u64)> = read
        .iter()
        .map(|slot| {
            (
                (slot.region, slot.slot_in_region),
                (slot.instance, slot.checkpoint_txg),
            )
        })
        .collect();
    let every_slot: BTreeSet<(u64, u64)> = predicted.keys().chain(actual.keys()).copied().collect();
    every_slot
        .into_iter()
        .filter(|ring_slot| predicted.get(ring_slot) != actual.get(ring_slot))
        .map(|ring_slot| {
            format!(
                "槽 {ring_slot:?}：账本 {:?}，池级 checker {:?}",
                predicted.get(&ring_slot),
                actual.get(&ring_slot)
            )
        })
        .collect()
}

/// S3：账本推的最新根那张实例表的行集，对镜像上读出的。
pub(super) fn row_mismatch(
    ledger: &Ledger,
    reading: &ImageReadingOfTheJudgements,
) -> Option<String> {
    let on_the_image: BTreeSet<RowTriple> = reading
        .rows
        .iter()
        .map(|row| {
            (
                row.instance,
                row.published_checkpoint_txg,
                row.applied_transaction_high_water_mark,
            )
        })
        .collect();
    let Some(version) = ledger.versions.get(&reading.newest) else {
        return Some(format!("最新根 {} 不在账本里", reading.newest.text()));
    };
    (*version.rows != on_the_image).then(|| {
        format!(
            "最新根 {}：账本推的行 {:?}，镜像上读出 {:?}",
            reading.newest.text(),
            version.rows,
            on_the_image
        )
    })
}

/// S5 账本这一边（登记修订 3、修订 4）：记录此刻还读得出、账本按自己推的行与根环占位认 ① ② ④ 成立的版本，分两支：
/// 被某次挂载施加过、根槽没落的；根槽落过、没被某次挂载施加过、后来被环盖掉或根槽读坏的。
/// ① 最新根的实例表（账本推的行集）里有这个实例的行 (i, Ti) 且 txg ≤ Ti；② 此刻读得出的根环槽（账本推的占位，减去读坏的槽）里没有这一版；
/// ④ 此刻读得出的根环槽里有一条没被最新根的实例表判抛弃的根，txg 比它小。③（不低于回退下界）在 ss、skr、sw 各组恒成立（这几组不抬 F），账本不判。
/// 两支都要过 ④：环转过去、最旧的有效根比这一版新之后，这一版换下的单元已可回收，装置按 ④ 不认，账本也不收（修订 4：sw-crash 的 k = 9 那一张，实例 3 崩掉的那一版）。
fn version_the_records_alone_show(
    ledger: &Ledger,
    rows_of_the_newest: &BTreeSet<RowTriple>,
    key: VersionKey,
    version: &LedgerVersion,
) -> bool {
    let applied_by_a_mount_without_a_root = version.applied_by_a_mount && !version.root_landed;
    let covered_after_its_root_landed = version.root_landed && !version.applied_by_a_mount;
    if !(applied_by_a_mount_without_a_root || covered_after_its_root_landed) {
        return false;
    }
    let readable_on_the_ring: Vec<VersionKey> = ledger
        .ring_occupants
        .iter()
        .filter(|(ring_slot, _)| !ledger.corrupted_ring_slots.contains(*ring_slot))
        .map(|(_, occupant)| *occupant)
        .collect();
    let applied_by_a_recovery =
        rows_of_the_newest
            .iter()
            .any(|(instance, published_checkpoint_txg, _)| {
                *instance == key.instance && key.txg <= *published_checkpoint_txg
            });
    let not_on_the_ring = !readable_on_the_ring.contains(&key);
    let abandoned_by_the_rows = |root: &VersionKey| {
        rows_of_the_newest
            .iter()
            .any(|(instance, published_checkpoint_txg, _)| {
                *instance == root.instance && root.txg > *published_checkpoint_txg
            })
    };
    let an_older_valid_root = readable_on_the_ring
        .iter()
        .any(|root| !abandoned_by_the_rows(root) && root.txg < key.txg);
    applied_by_a_recovery && not_on_the_ring && an_older_valid_root
}

/// S5：装置认出的由记录施加出来的版本，对账本里 `version_the_records_alone_show` 收的版本（两支，各过 ① ② ④，登记修订 3、修订 4）减去此刻记录已读不出的
/// （`journal_records_as_the_recovery_reads_them` 读，按 (实例, txg) 认）。
fn records_applied_comparison(
    image: &MemoryPool,
    ledger: &Ledger,
    reading: &ImageReadingOfTheJudgements,
) -> RecordsAppliedComparison {
    let readable: BTreeSet<VersionKey> = journal_records_as_the_recovery_reads_them(image)
        .unwrap_or_default()
        .iter()
        .map(|record| VersionKey {
            instance: record.instance,
            txg: record.checkpoint_txg,
        })
        .collect();
    let rows_of_the_newest = ledger.rows_of(Some(reading.newest));
    let in_the_ledger: BTreeSet<VersionKey> = ledger
        .versions
        .iter()
        .filter(|(key, version)| {
            readable.contains(*key)
                && version_the_records_alone_show(ledger, &rows_of_the_newest, **key, version)
        })
        .map(|(key, _)| *key)
        .collect();
    let by_the_apparatus: BTreeSet<VersionKey> = reading
        .applied_only_by_records
        .iter()
        .map(|version| version.key)
        .collect();
    let mut comparison = RecordsAppliedComparison::default();
    for extra in by_the_apparatus.difference(&in_the_ledger) {
        if ledger
            .versions
            .get(extra)
            .is_some_and(|version| version.root_landed)
        {
            comparison
                .apparatus_extra_with_a_root_that_landed_once
                .push(*extra);
        } else {
            comparison.apparatus_extra_otherwise.push(*extra);
        }
    }
    comparison.apparatus_missing = in_the_ledger
        .difference(&by_the_apparatus)
        .copied()
        .collect();
    comparison
}

/// 判一张镜像：两条臂的洞对分类、真值、停机条款 S1、S3、S5；`run_the_pool_checker` 时另跑今天的 `check_pool_image` 取 I-3.9 与 I-3.5。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
pub(super) fn judge_the_image(
    image: &MemoryPool,
    ledger: &Ledger,
    label: String,
    step: u64,
    run_the_pool_checker: bool,
    corrupted_skipped: Vec<String>,
) -> ImageResult {
    let reading = read_the_image(image);
    let mut truths = Vec::new();
    let mut counts = ArmCounts::default();
    let mut pointer_unreadable_pairs = 0;
    let mut row_mismatch_text = None;
    let mut records_applied = RecordsAppliedComparison::default();
    if let Ok(reading) = &reading {
        for pair in &reading.hole_pairs {
            let truth = truth_of_the_hole_pair(ledger, &pair.previous_group, pair.version.key);
            truths.push(truth);
            counts.add(arm_counts_of_the_pair(pair, truth));
            if pair.version.pointer_birth == PointerBirth::AllZeroOrUnreadable {
                pointer_unreadable_pairs += 1;
            }
        }
        row_mismatch_text = row_mismatch(ledger, reading);
        records_applied = records_applied_comparison(image, ledger, reading);
    }
    let checker_verdicts = run_the_pool_checker.then(|| {
        let verdicts = check_pool_image(image);
        let name_of = |invariant: &str| {
            verdicts
                .iter()
                .find(|(name, _)| *name == invariant)
                .map_or("missing", |(_, verdict)| verdict_name(verdict))
                .to_string()
        };
        (name_of("I-3.9"), name_of("I-3.5"))
    });
    ImageResult {
        label,
        step,
        reading,
        truths,
        counts,
        pointer_unreadable_pairs,
        ring_slot_mismatches: ring_slot_mismatches(image, ledger),
        row_mismatch: row_mismatch_text,
        records_applied,
        checker_verdicts,
        corrupted_skipped,
    }
}

#[cfg(test)]
mod tests;
