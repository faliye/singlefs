//! 前缀判定的规则与步行、按段恢复与恢复的计数、指标、待定轨迹、格的结果与固定序列的闭式锚点。

use super::*;

impl StateView {
    pub(super) fn of(state: &CrashState) -> StateView {
        let durable_new: BTreeSet<WriteContent> = state
            .durable
            .iter()
            .filter(|(_, form)| *form == DurableForm::New)
            .map(|(content, _)| *content)
            .collect();
        let mut records_by_jsn: Vec<RecordContent> = durable_new
            .iter()
            .filter_map(|content| match content {
                WriteContent::JournalRecord(record) => Some(*record),
                WriteContent::TransactionUnit { .. } | WriteContent::SharedUnit { .. } | WriteContent::RootSlot { .. } | WriteContent::SystemConfiguration { .. } => None,
            })
            .collect();
        records_by_jsn.sort_by_key(|record| record.jsn);
        // 择根：持久的根槽里 txg 最大的那个（登记 5.3 ①）。
        let chosen_root = durable_new
            .iter()
            .filter_map(|content| match content {
                WriteContent::RootSlot { publish, txg } => Some((*publish, *txg)),
                WriteContent::TransactionUnit { .. } | WriteContent::SharedUnit { .. } | WriteContent::JournalRecord(_) | WriteContent::SystemConfiguration { .. } => None,
            })
            .max_by_key(|(_, txg)| *txg);
        StateView { records_by_jsn, durable_new, chosen_root, returned: state.returned.clone() }
    }
    pub(super) fn chosen_root_txg(&self) -> u64 {
        self.chosen_root.map_or(START_ROOT_TXG, |(_, txg)| txg)
    }
}

/// 点名单元验得过 = 那些写持久且为「新」态（登记 5.3：提交记录点名它的 U，末条另点名本发布的 S，占位记录不点名 U）。
pub(super) fn named_units_are_verified(record: &RecordContent, durable_new: &BTreeSet<WriteContent>) -> bool {
    let names_its_unit = record.transaction != 0 && record.is_commit;
    let unit_verified = !names_its_unit || durable_new.contains(&WriteContent::TransactionUnit { transaction: record.transaction });
    let shared_verified = !record.ends_its_publish || durable_new.contains(&WriteContent::SharedUnit { publish: record.publish });
    unit_verified && shared_verified
}

/// `lib.rs:452` 反向链：单实例、没有残留记录，恒不断（登记第十一节未建模）。
pub(super) fn back_chain_is_broken(_record: &RecordContent) -> bool {
    false
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AnchorLookup {
    Anchored(u64),
    NoAnchor,
    MoreThanOneLastRecordFlag,
}

/// `lib.rs:190` counter_of_the_last_record_the_root_covers：所选根那次发布里带末条标志的那一条。
fn last_record_the_root_covers(records_by_jsn: &[RecordContent], root_txg: u64) -> AnchorLookup {
    let mut found: Option<u64> = None;
    for record in records_by_jsn {
        if record.txg == root_txg && record.ends_its_publish {
            if found.is_some() {
                return AnchorLookup::MoreThanOneLastRecordFlag;
            }
            found = Some(record.jsn);
        }
    }
    found.map_or(AnchorLookup::NoAnchor, AnchorLookup::Anchored)
}

/// `lib.rs:150` readable_record_of_the_same_publish_follows：这一条之后同一 txg 里还有没有读得出的记录。
fn readable_record_of_the_same_publish_follows(records_by_jsn: &[RecordContent], index: usize) -> bool {
    let record = records_by_jsn[index];
    for later in &records_by_jsn[index + 1..] {
        // lib.rs:173
        if later.txg > record.txg {
            return false;
        }
        // lib.rs:177
        if later.txg == record.txg {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Application {
    /// 施加单位是一次发布（D16（发布语义） 已定项 4）。
    WholePublish,
    /// 段里的记录逐条施加。
    RecordByRecord,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct PrefixRules {
    check_commit_before_transaction_change: bool,
    check_commit_on_last_record: bool,
    application: Application,
    stop_before_jsn: Option<u64>,
}

impl PrefixRules {
    pub(super) const TODAY: PrefixRules = PrefixRules {
        check_commit_before_transaction_change: true,
        check_commit_on_last_record: true,
        application: Application::WholePublish,
        stop_before_jsn: None,
    };
}

#[derive(Clone, Debug, Default)]
pub(super) struct PrefixWalk {
    accepted: Vec<RecordContent>,
    pub(super) applied: Vec<RecordContent>,
    in_flight_limit_reached: bool,
    pub(super) core_failure: bool,
}

/// `crates/singlefs-replay-prefix/src/lib.rs` 第 422–534 行 `replay_core` 主循环逐判转写；
/// `rules` 只为丙1、丙2 与阳性对照关掉写明的那几判或换施加单位，默认（`PrefixRules::TODAY`）就是丙0。
pub(super) fn walk_prefix(records_by_jsn: &[RecordContent], root_txg: u64, named_units_verified: &dyn Fn(&RecordContent) -> bool, rules: PrefixRules) -> PrefixWalk {
    let mut walk = PrefixWalk::default();
    // lib.rs:335 链首锚点
    let anchor = match last_record_the_root_covers(records_by_jsn, root_txg) {
        AnchorLookup::Anchored(anchor_jsn) => Some(anchor_jsn),
        AnchorLookup::NoAnchor => None,
        AnchorLookup::MoreThanOneLastRecordFlag => {
            walk.core_failure = true;
            return walk;
        }
    };
    let mut expected_next_jsn: Option<u64> = anchor.map(|anchor_jsn| anchor_jsn + 1);
    let chain_start_txg_without_anchor = root_txg + 1;
    let mut open_publish: Vec<RecordContent> = Vec::new();
    let mut taken: usize = 0;
    for (index, record) in records_by_jsn.iter().enumerate() {
        // lib.rs:422 不在水位之上跳过
        if record.txg <= root_txg {
            continue;
        }
        // lib.rs:427 取满在飞上限停
        if taken >= IN_FLIGHT_RECORD_LIMIT {
            walk.in_flight_limit_reached = true;
            break;
        }
        taken += 1;
        // 丙1：段截到 X 的第一条之前
        if rules.stop_before_jsn.is_some_and(|cut_jsn| record.jsn >= cut_jsn) {
            break;
        }
        match expected_next_jsn {
            // lib.rs:434 jsn 不等于期望停
            Some(expected_jsn) => {
                if record.jsn != expected_jsn {
                    break;
                }
            }
            // lib.rs:445 无锚点时链首要 txg = 根 + 1 且序号 1
            None => {
                if record.txg != chain_start_txg_without_anchor || record.ordinal_within_publish != 1 {
                    break;
                }
            }
        }
        // lib.rs:452 反向链断停
        if back_chain_is_broken(record) {
            break;
        }
        // lib.rs:456 攒着的为空而序号不是 1 停
        if open_publish.is_empty() && record.ordinal_within_publish != 1 {
            break;
        }
        if let Some(previous_record) = open_publish.last() {
            // lib.rs:463 攒着的发布里换了 txg 停
            if record.txg != previous_record.txg {
                break;
            }
            // lib.rs:468 发布内序号跳号停
            if record.ordinal_within_publish != previous_record.ordinal_within_publish + 1 {
                break;
            }
            // lib.rs:472 上一条没带提交标记而这一条换了事务号停
            if rules.check_commit_before_transaction_change && !previous_record.is_commit && record.transaction != previous_record.transaction {
                break;
            }
        }
        // lib.rs:478
        expected_next_jsn = Some(record.jsn + 1);
        // lib.rs:480 末条不带提交标记停
        if rules.check_commit_on_last_record && !record.is_commit && record.ends_its_publish {
            break;
        }
        // lib.rs:484 末条之后同发布还有可读记录停
        if record.ends_its_publish && readable_record_of_the_same_publish_follows(records_by_jsn, index) {
            break;
        }
        // lib.rs:493 点名单元验不过停
        if !named_units_verified(record) {
            break;
        }
        open_publish.push(*record);
        walk.accepted.push(*record);
        match rules.application {
            Application::RecordByRecord => {
                walk.applied.push(*record);
                if record.ends_its_publish {
                    open_publish.clear();
                }
            }
            Application::WholePublish => {
                // lib.rs:529 末条没到就接着往下走
                if !record.ends_its_publish {
                    continue;
                }
                // lib.rs:534 走到末条才把整次发布计入已施加
                walk.applied.append(&mut open_publish);
            }
        }
    }
    walk
}

/// 登记第一节 1.2「不合法前缀」的严格连续段：从链首起、jsn 逐一连续、全部持久、在水位以上的最长一段。
/// 链首照 5.3 ③：有锚点时是锚点 jsn + 1；没有锚点时是水位之上第一条、它要 txg = 根 + 1 且序号 1（修订第 2 条）。
pub(super) fn strict_continuous_segment(records_by_jsn: &[RecordContent], root_txg: u64) -> Vec<RecordContent> {
    let anchor = match last_record_the_root_covers(records_by_jsn, root_txg) {
        AnchorLookup::Anchored(anchor_jsn) => Some(anchor_jsn),
        AnchorLookup::NoAnchor => None,
        AnchorLookup::MoreThanOneLastRecordFlag => return Vec::new(),
    };
    let mut segment: Vec<RecordContent> = Vec::new();
    for record in records_by_jsn.iter().filter(|candidate| candidate.txg > root_txg) {
        let continues = match segment.last() {
            None => match anchor {
                Some(anchor_jsn) => record.jsn == anchor_jsn + 1,
                None => record.txg == root_txg + 1 && record.ordinal_within_publish == 1,
            },
            Some(last_in_segment) => record.jsn == last_in_segment.jsn + 1,
        };
        if !continues {
            break;
        }
        segment.push(*record);
    }
    segment
}

#[derive(Clone, Debug)]
struct TransactionInSegment {
    first_jsn: u64,
    has_commit: bool,
    jsns: Vec<u64>,
}

/// 段里各事务（占位记录的事务号 0 不算，D23（journal 的角色与格式） 已定项 19 ①）。
fn transactions_in(segment: &[RecordContent]) -> BTreeMap<u32, TransactionInSegment> {
    let mut transactions: BTreeMap<u32, TransactionInSegment> = BTreeMap::new();
    for record in segment.iter().filter(|candidate| candidate.transaction != 0) {
        let entry = transactions
            .entry(record.transaction)
            .or_insert(TransactionInSegment { first_jsn: record.jsn, has_commit: false, jsns: Vec::new() });
        entry.first_jsn = entry.first_jsn.min(record.jsn);
        entry.has_commit |= record.is_commit;
        entry.jsns.push(record.jsn);
    }
    transactions
}

/// N2a 的谓词：段里有两个或更多事务各有记录、而各自的提交记录不在段里。
fn two_or_more_incomplete_transactions_in(segment: &[RecordContent]) -> bool {
    transactions_in(segment).values().filter(|transaction| !transaction.has_commit).count() >= 2
}

/// 模式 P：段里某个事务 X 的提交记录不在段里，而段里有另一个事务 Y 的提交记录、且 Y 有一条记录的 jsn 大于 X 在段里的第一条。
fn incomplete_transaction_followed_by_complete_transaction_in(segment: &[RecordContent]) -> bool {
    let transactions = transactions_in(segment);
    for (incomplete_number, incomplete) in transactions.iter().filter(|(_, candidate)| !candidate.has_commit) {
        for (complete_number, complete) in transactions.iter().filter(|(_, candidate)| candidate.has_commit) {
            if complete_number != incomplete_number && complete.jsns.iter().any(|jsn| *jsn > incomplete.first_jsn) {
                return true;
            }
        }
    }
    false
}

/// 丙1 的 X：严格连续段里 jsn 最小的「提交记录不在段里」的事务，返回它在段里第一条的 jsn。
fn first_incomplete_transaction_first_jsn(segment: &[RecordContent]) -> Option<u64> {
    transactions_in(segment).values().filter(|transaction| !transaction.has_commit).map(|transaction| transaction.first_jsn).min()
}

/// 丙2：拿掉「提交记录不在段里」的每个事务的全部记录（占位记录留着）。
fn drop_transactions_without_commit_in(accepted: &[RecordContent]) -> Vec<RecordContent> {
    let committed: BTreeSet<u32> = accepted.iter().filter(|record| record.transaction != 0 && record.is_commit).map(|record| record.transaction).collect();
    accepted.iter().filter(|record| record.transaction == 0 || committed.contains(&record.transaction)).copied().collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RecoveryOutcome {
    pub(super) mounted: bool,
    chosen_root_txg: u64,
    pub(super) applied: Vec<RecordContent>,
    /// 装上的根：所选根，或施加的记录里 jsn 最大那条带的整次发布新根段（`crates/singlefs-core/src/transaction.rs:6728`）。
    installed_root_publish: Option<u32>,
    installed_root_txg: u64,
    in_flight_limit_reached: bool,
    pub(super) core_failure: bool,
}

impl RecoveryOutcome {
    fn mounted_with(view: &StateView, applied: Vec<RecordContent>, walk_flags: (bool, bool)) -> RecoveryOutcome {
        let (in_flight_limit_reached, core_failure) = walk_flags;
        let (installed_root_publish, installed_root_txg) = match applied.last() {
            Some(last_applied) => (Some(last_applied.publish), last_applied.txg),
            None => (view.chosen_root.map(|(publish, _)| publish), view.chosen_root_txg()),
        };
        RecoveryOutcome {
            mounted: !core_failure,
            chosen_root_txg: view.chosen_root_txg(),
            applied,
            installed_root_publish,
            installed_root_txg,
            in_flight_limit_reached,
            core_failure,
        }
    }
    pub(super) fn refused(view: &StateView) -> RecoveryOutcome {
        RecoveryOutcome {
            mounted: false,
            chosen_root_txg: view.chosen_root_txg(),
            applied: Vec::new(),
            installed_root_publish: None,
            installed_root_txg: START_ROOT_TXG,
            in_flight_limit_reached: false,
            core_failure: false,
        }
    }
    fn version_key(&self) -> (bool, Option<u32>, Vec<RecordContent>) {
        (self.mounted, self.installed_root_publish, self.applied.clone())
    }
}

const RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS: PrefixRules = PrefixRules {
    check_commit_before_transaction_change: false,
    check_commit_on_last_record: false,
    application: Application::RecordByRecord,
    stop_before_jsn: None,
};

pub(super) fn recover(view: &StateView, arm: RecoveryArm, control: RecoveryControl) -> RecoveryOutcome {
    let root_txg = view.chosen_root_txg();
    let durable_new = &view.durable_new;
    let verify = |record: &RecordContent| named_units_are_verified(record, durable_new);
    let strict_segment = strict_continuous_segment(&view.records_by_jsn, root_txg);
    let mounted_after = |rules: PrefixRules| -> RecoveryOutcome {
        let walk = walk_prefix(&view.records_by_jsn, root_txg, &verify, rules);
        RecoveryOutcome::mounted_with(view, walk.applied, (walk.in_flight_limit_reached, walk.core_failure))
    };
    match arm {
        RecoveryArm::Bing0Today => match control {
            RecoveryControl::NoControl => mounted_after(PrefixRules::TODAY),
            RecoveryControl::KeepLaterRecordsRemoved | RecoveryControl::CommitMarkerHandlingRemoved => mounted_after(RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS),
        },
        RecoveryArm::Bing1TruncateBeforeTheIncompleteTransaction => match control {
            RecoveryControl::NoControl => {
                let cut_before_the_incomplete_transaction = first_incomplete_transaction_first_jsn(&strict_segment);
                mounted_after(PrefixRules {
                    check_commit_before_transaction_change: false,
                    check_commit_on_last_record: false,
                    application: Application::WholePublish,
                    stop_before_jsn: cut_before_the_incomplete_transaction,
                })
            }
            RecoveryControl::KeepLaterRecordsRemoved | RecoveryControl::CommitMarkerHandlingRemoved => mounted_after(RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS),
        },
        RecoveryArm::Bing2DropTheIncompleteKeepTheComplete => {
            let walk = walk_prefix(&view.records_by_jsn, root_txg, &verify, RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS);
            let applied = match control {
                RecoveryControl::NoControl | RecoveryControl::KeepLaterRecordsRemoved => drop_transactions_without_commit_in(&walk.accepted),
                RecoveryControl::CommitMarkerHandlingRemoved => walk.accepted.clone(),
            };
            RecoveryOutcome::mounted_with(view, applied, (walk.in_flight_limit_reached, walk.core_failure))
        }
        RecoveryArm::Bing3ForbidPattern => match control {
            RecoveryControl::NoControl => {
                let judged_corrupt = incomplete_transaction_followed_by_complete_transaction_in(&strict_segment);
                if judged_corrupt {
                    RecoveryOutcome::refused(view)
                } else {
                    mounted_after(PrefixRules::TODAY)
                }
            }
            RecoveryControl::KeepLaterRecordsRemoved => mounted_after(PrefixRules {
                check_commit_before_transaction_change: true,
                check_commit_on_last_record: true,
                application: Application::RecordByRecord,
                stop_before_jsn: None,
            }),
            RecoveryControl::CommitMarkerHandlingRemoved => mounted_after(RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS),
        },
    }
}

// ===== 每个状态上的量（登记第六节） =====

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct RecoveryCounts {
    pub(super) refused: u64,
    pub(super) fsync_violations: u64,
    pub(super) fsync_violations_mounted_lost: u64,
    pub(super) fsync_violations_refused_with_returned: u64,
    pub(super) fsync_violations_branch_every_earlier_appended: u64,
    pub(super) fsync_violations_branch_earlier_passed_as_voided: u64,
    pub(super) dependency_violations: u64,
    pub(super) dangling_violations_returned_empty: u64,
    pub(super) dangling_violations_returned_nonempty: u64,
    pub(super) partial_publish: u64,
    pub(super) atomicity_violations: u64,
    pub(super) core_failures: u64,
    pub(super) differs_from_bing0: u64,
}

impl RecoveryCounts {
    pub(super) fn absorb(&mut self, other: &RecoveryCounts) {
        self.refused += other.refused;
        self.fsync_violations += other.fsync_violations;
        self.fsync_violations_mounted_lost += other.fsync_violations_mounted_lost;
        self.fsync_violations_refused_with_returned += other.fsync_violations_refused_with_returned;
        self.fsync_violations_branch_every_earlier_appended += other.fsync_violations_branch_every_earlier_appended;
        self.fsync_violations_branch_earlier_passed_as_voided += other.fsync_violations_branch_earlier_passed_as_voided;
        self.dependency_violations += other.dependency_violations;
        self.dangling_violations_returned_empty += other.dangling_violations_returned_empty;
        self.dangling_violations_returned_nonempty += other.dangling_violations_returned_nonempty;
        self.partial_publish += other.partial_publish;
        self.atomicity_violations += other.atomicity_violations;
        self.core_failures += other.core_failures;
        self.differs_from_bing0 += other.differs_from_bing0;
    }
    pub(super) fn dangling_violations(&self) -> u64 {
        self.dangling_violations_returned_empty + self.dangling_violations_returned_nonempty
    }
}

fn metrics_of(view: &StateView, layout: &CellLayout, outcome: &RecoveryOutcome) -> RecoveryCounts {
    let mut counts = RecoveryCounts::default();
    let mut applied_per_transaction: BTreeMap<u32, u32> = BTreeMap::new();
    let mut applied_per_publish: BTreeMap<u32, usize> = BTreeMap::new();
    for record in &outcome.applied {
        if record.transaction != 0 {
            *applied_per_transaction.entry(record.transaction).or_insert(0) += 1;
        }
        *applied_per_publish.entry(record.publish).or_insert(0) += 1;
    }
    let applied_of = |transaction: u32| applied_per_transaction.get(&transaction).copied().unwrap_or(0);
    let transaction_layout = |transaction: u32| &layout.transactions[transaction_position(transaction)];
    // 登记第六节 Q3-1：T 在恢复出的版本里 ⇔ T 所在发布的 txg 不大于所选根的 txg，或 T 的全部记录被施加。
    let in_recovered_version = |transaction: u32| {
        let layout_of_transaction = transaction_layout(transaction);
        layout_of_transaction.txg.is_some_and(|txg| txg <= outcome.chosen_root_txg) || applied_of(transaction) == layout_of_transaction.planned_records
    };
    if outcome.core_failure {
        counts.core_failures = 1;
    }
    if outcome.mounted {
        let lost: Vec<&(u32, ReturnBranch)> = view.returned.iter().filter(|(transaction, _)| !in_recovered_version(*transaction)).collect();
        if !lost.is_empty() {
            counts.fsync_violations = 1;
            counts.fsync_violations_mounted_lost = 1;
            counts.fsync_violations_branch_every_earlier_appended = u64::from(lost.iter().any(|(_, branch)| *branch == ReturnBranch::EveryEarlierAppended));
            counts.fsync_violations_branch_earlier_passed_as_voided = u64::from(lost.iter().any(|(_, branch)| *branch == ReturnBranch::SomeEarlierPassedAsVoided));
        }
        // Q3-2：施加了某事务 Y 的记录，而某个没作废、号小于 Y、txg 大于所选根的事务 X 的记录没全部施加。
        let dependency_violated = applied_per_transaction.keys().any(|later| {
            layout.transactions.iter().any(|earlier| {
                earlier.voided.is_none()
                    && earlier.number < *later
                    && earlier.txg.is_some_and(|txg| txg > outcome.chosen_root_txg)
                    && applied_of(earlier.number) < earlier.planned_records
            })
        });
        counts.dependency_violations = u64::from(dependency_violated);
        // Q3-3：装上的根引用的写里有不持久或撕裂的。
        let unit_missing = layout.transactions.iter().any(|transaction| {
            transaction.voided.is_none()
                && transaction.txg.is_some_and(|txg| txg <= outcome.installed_root_txg)
                && !view.durable_new.contains(&WriteContent::TransactionUnit { transaction: transaction.number })
        });
        let shared_missing = outcome
            .installed_root_publish
            .is_some_and(|publish| !view.durable_new.contains(&WriteContent::SharedUnit { publish }));
        if unit_missing || shared_missing {
            if view.returned.is_empty() {
                counts.dangling_violations_returned_empty = 1;
            } else {
                counts.dangling_violations_returned_nonempty = 1;
            }
        }
        // Q3-4：一次发布只施加了一部分记录。
        counts.partial_publish = u64::from(
            applied_per_publish
                .iter()
                .any(|(publish, applied_count)| *applied_count > 0 && *applied_count < layout.written_record_count_of(*publish)),
        );
        // 第七节 A3（C37（事务与记录的对应无定义） 层 0 那一半）：某个事务的记录被施加了一部分。
        counts.atomicity_violations = u64::from(
            applied_per_transaction
                .iter()
                .any(|(transaction, applied_count)| *applied_count > 0 && *applied_count < transaction_layout(*transaction).planned_records),
        );
    } else {
        counts.refused = u64::from(!outcome.core_failure);
        if !view.returned.is_empty() {
            counts.fsync_violations = 1;
            counts.fsync_violations_refused_with_returned = 1;
            counts.fsync_violations_branch_every_earlier_appended = u64::from(view.returned.iter().any(|(_, branch)| *branch == ReturnBranch::EveryEarlierAppended));
            counts.fsync_violations_branch_earlier_passed_as_voided = u64::from(view.returned.iter().any(|(_, branch)| *branch == ReturnBranch::SomeEarlierPassedAsVoided));
        }
    }
    counts
}

// ===== 被谓词消费的量的轨迹（登记 8.1 第一行，按闭段点记） =====

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PendingTrajectory {
    pub(super) initial_peak: u64,
    pub(super) peak_after_drain: u64,
    pub(super) positive_closers_peak: u64,
    pub(super) final_peak: u64,
    pub(super) never_drained: u64,
}

impl PendingTrajectory {
    pub(super) fn absorb(&mut self, other: &PendingTrajectory) {
        self.initial_peak = self.initial_peak.max(other.initial_peak);
        self.peak_after_drain = self.peak_after_drain.max(other.peak_after_drain);
        self.positive_closers_peak = self.positive_closers_peak.max(other.positive_closers_peak);
        self.final_peak = self.final_peak.max(other.final_peak);
        self.never_drained += other.never_drained;
    }
}

pub(super) fn pending_trajectory_of(execution: &Execution, reading: AppendedReading) -> PendingTrajectory {
    let closers: Vec<usize> = execution
        .operations
        .iter()
        .enumerate()
        .filter(|(_, operation)| operation.closes_a_segment())
        .map(|(operation_index, _)| operation_index)
        .collect();
    let mut trajectory = PendingTrajectory::default();
    for timeline in execution.timelines.iter().filter(|candidate| candidate.voided_from.is_none()) {
        let pending_at = |completed: Option<usize>, issued: Option<usize>| -> u64 {
            let pending = execution
                .timelines
                .iter()
                .filter(|earlier| earlier.number < timeline.number && !is_appended(earlier, completed, issued, reading) && !is_voided_by(earlier, issued))
                .count();
            u64::try_from(pending).expect("k ≤ 3")
        };
        let initial = pending_at(None, None);
        let values: Vec<u64> = closers.iter().map(|closer| pending_at(Some(*closer), Some(*closer))).collect();
        let positive_closers = u64::try_from(values.iter().filter(|value| **value > 0).count()).expect("闭段点数有限");
        let final_value = values.last().copied().unwrap_or(initial);
        let mut this_transaction = PendingTrajectory {
            initial_peak: initial,
            peak_after_drain: 0,
            positive_closers_peak: positive_closers,
            final_peak: final_value,
            never_drained: 0,
        };
        match values.iter().position(|value| *value == 0) {
            Some(drained_at) => this_transaction.peak_after_drain = values[drained_at..].iter().copied().max().unwrap_or(0),
            None => this_transaction.never_drained = 1,
        }
        trajectory.absorb(&this_transaction);
    }
    trajectory
}

// ===== 一格 =====

#[derive(Clone, Debug)]
pub(super) struct CellRequest {
    pub(super) condition: PublishCondition,
    pub(super) append_arm: AppendArm,
    pub(super) history: History,
    pub(super) reading: AppendedReading,
    pub(super) return_rule: ReturnRule,
    pub(super) discipline: AppendDiscipline,
    pub(super) barriers: PublishBarriers,
    pub(super) recovery_control: RecoveryControl,
}

impl CellRequest {
    pub(super) fn main_grid(condition: PublishCondition, append_arm: AppendArm, history: History, reading: AppendedReading) -> CellRequest {
        CellRequest {
            condition,
            append_arm,
            history,
            reading,
            return_rule: ReturnRule::AfterOwnPublishLastBarrier,
            discipline: AppendDiscipline::Kept,
            barriers: PublishBarriers::AllThree,
            recovery_control: RecoveryControl::NoControl,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct AppendCellResult {
    pub(super) executions: u64,
    pub(super) states: u64,
    pub(super) incomplete_pair_states: u64,
    pub(super) out_of_order_commit_pattern_states: u64,
    pub(super) strict_segment_length_peak: usize,
    pub(super) in_flight_limit_states: u64,
    pub(super) returned_before_last_barrier_points: u64,
    pub(super) pending: PendingTrajectory,
    pub(super) whole_arms_differing_states: u64,
    pub(super) recovery: [RecoveryCounts; 4],
    /// 第二段：这一格怎么枚举的（第一段为 None）。
    pub(super) multi_flight_enumeration: Option<MultiFlightEnumeration>,
    /// 按配置去重时走到过的配置数（停机条款 T3）。
    pub(super) configurations: Option<u64>,
    pub(super) configuration_limit_exceeded: bool,
}

/// 甲整段下一个事务的记录连着发出，中间不能作废：H2 对它不适用（登记 5.4）。
pub(super) fn cell_is_applicable(append_arm: AppendArm, history: &History) -> bool {
    !(append_arm == AppendArm::JiaWhole && history.family() == HistoryFamily::WideVoid)
}

pub(super) fn evaluate_cell(request: &CellRequest) -> Option<AppendCellResult> {
    if !cell_is_applicable(request.append_arm, &request.history) {
        return None;
    }
    let layout = plan_cell(request.condition, request.append_arm, &request.history, request.discipline);
    if request.condition.publishes_fly_together() {
        let enumeration = MultiFlightEnumeration::registered_for(&request.history);
        return Some(multi_flight_cell_result(request, &layout, enumeration).1);
    }
    let executions = build_executions(&layout, request.append_arm, request.discipline, request.barriers);
    let mut states: HashSet<CrashState> = HashSet::new();
    let mut returned_before_last_barrier_points = 0u64;
    let mut pending = PendingTrajectory::default();
    for execution in &executions {
        returned_before_last_barrier_points += collect_crash_states(execution, request.reading, request.return_rule, &mut states);
        pending.absorb(&pending_trajectory_of(execution, request.reading));
    }
    let enumerated = EnumeratedCrashStates {
        states,
        executions: u64::try_from(executions.len()).expect("执行数有限"),
        configurations: None,
        returned_before_last_barrier_points,
        pending,
        configuration_limit_exceeded: false,
    };
    Some(cell_result_from_states(&layout, &enumerated, request.recovery_control, None))
}

/// 一格多飞：按给定的枚举法得到崩溃状态，连同算出的格结果一起交回（路径对拍要比两种枚举法的状态集合）。
pub(super) fn multi_flight_cell_result(request: &CellRequest, layout: &CellLayout, enumeration: MultiFlightEnumeration) -> (EnumeratedCrashStates, AppendCellResult) {
    assert!(request.history.family().is_legal(), "第二段 H2 够判后未跑");
    let enumerated = enumerate_multi_flight(request, layout, enumeration);
    let result = cell_result_from_states(layout, &enumerated, request.recovery_control, Some(enumeration));
    (enumerated, result)
}

/// 一格的全部崩溃状态逐个过四条 Q3 臂，数登记第六节的各量。
fn cell_result_from_states(layout: &CellLayout, enumerated: &EnumeratedCrashStates, recovery_control: RecoveryControl, multi_flight_enumeration: Option<MultiFlightEnumeration>) -> AppendCellResult {
    let states = &enumerated.states;
    let mut result = AppendCellResult {
        executions: enumerated.executions,
        states: u64::try_from(states.len()).expect("状态数有限"),
        incomplete_pair_states: 0,
        out_of_order_commit_pattern_states: 0,
        strict_segment_length_peak: 0,
        in_flight_limit_states: 0,
        returned_before_last_barrier_points: enumerated.returned_before_last_barrier_points,
        pending: enumerated.pending,
        whole_arms_differing_states: 0,
        recovery: Default::default(),
        multi_flight_enumeration,
        configurations: enumerated.configurations,
        configuration_limit_exceeded: enumerated.configuration_limit_exceeded,
    };
    for state in states {
        let view = StateView::of(state);
        let strict_segment = strict_continuous_segment(&view.records_by_jsn, view.chosen_root_txg());
        result.incomplete_pair_states += u64::from(two_or_more_incomplete_transactions_in(&strict_segment));
        result.out_of_order_commit_pattern_states += u64::from(incomplete_transaction_followed_by_complete_transaction_in(&strict_segment));
        result.strict_segment_length_peak = result.strict_segment_length_peak.max(strict_segment.len());
        let outcomes: [RecoveryOutcome; 4] = RecoveryArm::ALL.map(|arm| recover(&view, arm, recovery_control));
        result.in_flight_limit_states += u64::from(outcomes.iter().any(|outcome| outcome.in_flight_limit_reached));
        let bing0_version = outcomes[RecoveryArm::Bing0Today.position()].version_key();
        result.whole_arms_differing_states += u64::from(outcomes.iter().any(|outcome| outcome.version_key() != bing0_version));
        for arm in RecoveryArm::ALL {
            let outcome = &outcomes[arm.position()];
            let mut counts = metrics_of(&view, layout, outcome);
            counts.differs_from_bing0 = u64::from(outcome.version_key() != bing0_version);
            result.recovery[arm.position()].absorb(&counts);
        }
    }
    result
}

// ===== 停机条款 T1：十四条固定记录序列（登记第七节 A2） =====

pub(super) struct FixedSequence {
    pub(super) number: u32,
    root_txg: u64,
    /// (jsn, txg, 事务号, 提交标记, 末条标志, 本次发布内序号)
    pub(super) records: Vec<(u64, u64, u32, bool, bool, u32)>,
    unverified_jsn: Option<u64>,
    pub(super) expected_publishes_applied: u64,
    pub(super) expected_records_applied: u64,
    pub(super) deciding_line: &'static str,
}

pub(super) fn fixed_sequences() -> Vec<FixedSequence> {
    let sequence = |number: u32, root_txg: u64, records: Vec<(u64, u64, u32, bool, bool, u32)>, unverified_jsn: Option<u64>, expected_publishes_applied: u64, expected_records_applied: u64, deciding_line: &'static str| FixedSequence {
        number,
        root_txg,
        records,
        unverified_jsn,
        expected_publishes_applied,
        expected_records_applied,
        deciding_line,
    };
    vec![
        sequence(1, 0, vec![(1, 1, 1, true, true, 1)], None, 1, 1, "529-534"),
        sequence(2, 0, vec![(1, 1, 1, false, false, 1), (2, 1, 1, true, true, 2)], None, 1, 2, "529-534"),
        sequence(3, 0, vec![(1, 1, 1, false, false, 1)], None, 0, 0, "529"),
        sequence(4, 0, vec![(1, 1, 1, false, false, 1), (2, 1, 2, true, false, 2), (3, 1, 1, true, true, 3)], None, 0, 0, "472"),
        sequence(5, 0, vec![(1, 1, 1, true, false, 1), (2, 1, 2, true, true, 2)], None, 1, 2, "529-534"),
        sequence(6, 0, vec![(1, 1, 1, true, false, 1), (2, 2, 2, true, true, 2)], None, 0, 0, "463"),
        sequence(7, 0, vec![(1, 1, 1, true, true, 1), (2, 2, 2, true, true, 1)], None, 2, 2, "529-534"),
        sequence(8, 0, vec![(1, 1, 1, true, true, 1), (3, 2, 2, true, true, 1)], None, 1, 1, "434"),
        sequence(9, 0, vec![(1, 1, 1, true, true, 1), (2, 1, 1, true, false, 2)], None, 0, 0, "484"),
        sequence(10, 0, vec![(1, 1, 1, true, true, 1)], Some(1), 0, 0, "493"),
        sequence(11, 0, vec![(1, 1, 1, false, true, 1)], None, 0, 0, "480"),
        sequence(12, 0, vec![(1, 2, 1, true, true, 1)], None, 0, 0, "445"),
        sequence(13, 1, vec![(1, 1, 1, true, true, 1), (2, 2, 2, true, true, 2)], None, 0, 0, "456"),
        sequence(14, 0, vec![(1, 1, 1, true, false, 1), (2, 1, 2, true, true, 3)], None, 0, 0, "468"),
    ]
}

/// 丙0 在一条固定序列上施加的（发布数，记录数）。点名单元除 `unverified_jsn` 那一条之外都验得过。
pub(super) fn bing0_on_fixed_sequence(sequence: &FixedSequence) -> (u64, u64) {
    let mut records: Vec<RecordContent> = sequence
        .records
        .iter()
        .map(|(jsn, txg, transaction, is_commit, ends_its_publish, ordinal)| RecordContent {
            jsn: *jsn,
            txg: *txg,
            publish: u32::try_from(*txg).expect("txg 小"),
            transaction: *transaction,
            is_commit: *is_commit,
            ends_its_publish: *ends_its_publish,
            ordinal_within_publish: *ordinal,
            origin: RecordOrigin::OwnRecord { index_within_transaction: *ordinal },
        })
        .collect();
    records.sort_by_key(|record| record.jsn);
    let unverified_jsn = sequence.unverified_jsn;
    let verify = |record: &RecordContent| unverified_jsn != Some(record.jsn);
    let walk = walk_prefix(&records, sequence.root_txg, &verify, PrefixRules::TODAY);
    let publishes = walk.applied.iter().filter(|record| record.ends_its_publish).count();
    (u64::try_from(publishes).expect("至多 2"), u64::try_from(walk.applied.len()).expect("至多 2"))
}

// ===== 第七节 B 类锚点 =====

fn two_to_the(exponent: u32) -> u64 {
    1u64 << exponent
}

/// B1：串落 H0 = 1 + Σ_i (6 + 2^{r_i})。
pub(super) fn serial_publish_anchor_closed_form(record_counts: &[u32]) -> u64 {
    1 + record_counts.iter().map(|count| 6 + two_to_the(*count)).sum::<u64>()
}

/// B2：同发 H0 甲整段、乙整段 = 2^{k+1} + 2^{Σr} + 3。
pub(super) fn shared_publish_anchor_closed_form(record_counts: &[u32]) -> u64 {
    let transaction_count = u32::try_from(record_counts.len()).expect("k 小");
    two_to_the(transaction_count + 1) + two_to_the(record_counts.iter().sum()) + 3
}

/// B4：作废·窄的闭式（登记第七节 B4 原样）。
pub(super) fn narrow_void_anchor_closed_form(condition: PublishCondition, reserves: bool, record_counts: &[u32], voided: u32) -> u64 {
    let transaction_count = u32::try_from(record_counts.len()).expect("k 小");
    let count_of = |transaction: u32| record_counts[transaction_position(transaction)];
    let others: Vec<u32> = (1..=transaction_count).filter(|transaction| *transaction != voided).collect();
    match (condition, reserves) {
        (PublishCondition::SharedPublish, false) => two_to_the(transaction_count) + two_to_the(others.iter().map(|transaction| count_of(*transaction)).sum()) + 3,
        (PublishCondition::SharedPublish, true) => two_to_the(transaction_count) + two_to_the(record_counts.iter().sum()) + 3,
        (PublishCondition::SerialPublishes, false) => 1 + others.iter().map(|transaction| 6 + two_to_the(count_of(*transaction))).sum::<u64>(),
        (PublishCondition::SerialPublishes, true) => {
            if voided < transaction_count {
                let rest: u64 = (1..=transaction_count)
                    .filter(|transaction| *transaction != voided && *transaction != voided + 1)
                    .map(|transaction| 6 + two_to_the(count_of(transaction)))
                    .sum();
                1 + rest + (6 + two_to_the(count_of(voided) + count_of(voided + 1)))
            } else {
                1 + others.iter().map(|transaction| 6 + two_to_the(count_of(*transaction))).sum::<u64>() + (3 + two_to_the(count_of(voided)))
            }
        }
        (PublishCondition::MultiFlightRootsOrdered | PublishCondition::MultiFlightRootsUnordered, _) => panic!("B4 只登记了串落与同发的闭式"),
    }
}
