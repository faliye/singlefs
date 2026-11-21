//! Q3-1：P332 在 H3 × Φ3 上的违例比例；有效根、忽略 journal 的读与无故障读三种判法，严格判据与诊断计数。

use super::*;

// ============================================================================
// 八、Q3-1（岔路 3 候选 3 那一半，跑前登记第二段）：P332 在 H3 × Φ3 上的违例比例。
//    今天那一臂 = 候选 3（`crates/` 今天的代码），不建副本。Φ3：那一刻存着自证根的根槽
//    （不按实例筛，5.1 原文），取 |F| ≤ 2 的全部子集（含空集）。
//    5.1「H3 的省法」：一个故障集合的冷启动 recover 所选根与 F=∅ 时相同 ⇒ 不做后两处，
//    直接记它们与 F=∅ 那一对相同（非违例）——F=∅ 本身照样全算，不吃自己的省法。
// ============================================================================

/// P332 在某一处的 (V1, V2)：`chosen_root_abandoned` 是 V1（所选根落在被抛弃集合里），
/// `content_off_timeline` 是 V2（读回的内容号不属于当前时间线上任何一个根）。
#[derive(Clone, Copy, Default)]
struct ViolationAtOneCheckpoint {
    chosen_root_abandoned: bool,
    content_off_timeline: bool,
}

fn is_abandoned(node: &SimNode, root: TimelineRoot) -> bool {
    node.rollback_events
        .iter()
        .any(|event| event.abandoned.contains(&root))
}

/// V2：`signature`（`None` = 没有文件，`Some(字节)` = 文件内容）不等于当前时间线上任何一个根
/// 的内容号 ⇒ 违例。账本如实之后它只问「这份内容出现过没有」，读回的是不是该读回的那一份归严格判据
/// （`judge_read_against_its_effective_root`、`judge_no_fault_read`、`judge_read_that_ignores_the_journal`）。
fn content_is_off_the_current_timeline(node: &SimNode, signature: &Option<Vec<u8>>) -> bool {
    !node
        .timeline
        .iter()
        .any(|root| node.content_by_root.get(root) == Some(signature))
}

/// 交回报告点名的一处操作化（5.6 待认清单没有覆盖到它：「回退到无文件那一版」要 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 落地才可达）：`NoFile` 结局时 V2 恒记非违例，只判 V1——「读回的内容号」预设了读回了内容，
/// 没有文件时没有内容号可比。
fn violation_at_checkpoint(
    node: &SimNode,
    outcome: &RecoveryOutcome,
) -> Option<ViolationAtOneCheckpoint> {
    match outcome {
        RecoveryOutcome::Failed { .. } => None,
        RecoveryOutcome::NoFile { root } => Some(ViolationAtOneCheckpoint {
            chosen_root_abandoned: is_abandoned(node, (root.0 .0, root.1 .0)),
            content_off_timeline: false,
        }),
        RecoveryOutcome::FileRead { root, content } => Some(ViolationAtOneCheckpoint {
            chosen_root_abandoned: is_abandoned(node, (root.0 .0, root.1 .0)),
            content_off_timeline: content_is_off_the_current_timeline(node, &Some(content.clone())),
        }),
    }
}

pub(super) fn chosen_root_of(outcome: &RecoveryOutcome) -> Option<TimelineRoot> {
    match outcome {
        RecoveryOutcome::NoFile { root } | RecoveryOutcome::FileRead { root, .. } => {
            Some((root.0 .0, root.1 .0))
        }
        RecoveryOutcome::Failed { root, .. } => {
            root.map(|chosen_root| (chosen_root.0 .0, chosen_root.1 .0))
        }
    }
}

// ----------------------------------------------------------------------------
// Q3-1 的严格判据。V2 只问内容出现过没有；这里问读回的是不是该读回的那一份，按实际走的根判：
// 恢复施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录（D23（journal 的角色与格式） 已定项 14），
// 读回内容与所选根本来可以不同，所以「内容与所选根不同」只是读数，不是违例。
// `judge_*` 是纯函数，单测拿手写账本直接判；`diagnose_*` 把它们的结论记进计数、有违例才逐条打。
// ----------------------------------------------------------------------------

/// 读回的内容号：`None` = 这次恢复没读成。`Some(None)` = 没有文件，`Some(Some(字节))` = 文件内容。
fn read_signature_of(outcome: &RecoveryOutcome) -> Option<Option<Vec<u8>>> {
    match outcome {
        RecoveryOutcome::Failed { .. } => None,
        RecoveryOutcome::NoFile { .. } => Some(None),
        RecoveryOutcome::FileRead { content, .. } => Some(Some(content.clone())),
    }
}

fn tip_signature_of(node: &SimNode) -> Option<&Option<Vec<u8>>> {
    node.content_by_root.get(
        node.timeline
            .last()
            .expect("bootstrap 至少写下 mkfs 根，时间线非空"),
    )
}

/// 一次带故障冷读对着实际走的根判出来的结论。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EffectiveRootFindings {
    effective_root_is_not_the_tip: bool,
    effective_root_off_the_timeline: bool,
    content_differs_from_its_effective_root: bool,
    differs_from_chosen_root: bool,
    differs_with_nothing_applied: bool,
}

fn judge_read_against_its_effective_root(
    node: &SimNode,
    chosen: Option<TimelineRoot>,
    effective: Option<TimelineRoot>,
    read: &Option<Vec<u8>>,
    records_applied: usize,
) -> EffectiveRootFindings {
    let tip = *node
        .timeline
        .last()
        .expect("bootstrap 至少写下 mkfs 根，时间线非空");
    let effective_root_on_the_timeline = effective.filter(|root| node.timeline.contains(root));
    let differs_from_chosen_root = chosen.is_some_and(|root| {
        node.timeline.contains(&root) && node.content_by_root.get(&root) != Some(read)
    });
    EffectiveRootFindings {
        effective_root_is_not_the_tip: effective != Some(tip),
        effective_root_off_the_timeline: effective_root_on_the_timeline.is_none(),
        content_differs_from_its_effective_root: effective_root_on_the_timeline
            .is_some_and(|root| node.content_by_root.get(&root) != Some(read)),
        differs_from_chosen_root,
        differs_with_nothing_applied: differs_from_chosen_root && records_applied == 0,
    }
}

/// 同一个故障集合不看 journal 再读一次（`JournalPolicy::Ignore`）判出来的结论。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IgnorePolicyFindings {
    chosen_root_changed: bool,
    content_differs_from_chosen_root: bool,
    content_is_not_the_tip: bool,
}

fn judge_read_that_ignores_the_journal(
    node: &SimNode,
    chosen_when_consulting: Option<TimelineRoot>,
    chosen_when_ignoring: Option<TimelineRoot>,
    read_when_ignoring: &Option<Vec<u8>>,
) -> IgnorePolicyFindings {
    IgnorePolicyFindings {
        chosen_root_changed: chosen_when_ignoring != chosen_when_consulting,
        content_differs_from_chosen_root: chosen_when_ignoring
            .and_then(|root| node.content_by_root.get(&root))
            != Some(read_when_ignoring),
        content_is_not_the_tip: tip_signature_of(node) != Some(read_when_ignoring),
    }
}

/// 不注入故障的那一次读判出来的结论：所选根与内容都必须是时间线尾的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NoFaultReadFindings {
    read_failed: bool,
    read_is_not_the_tip: bool,
}

fn judge_no_fault_read(
    node: &SimNode,
    chosen: Option<TimelineRoot>,
    signature: Option<&Option<Vec<u8>>>,
) -> NoFaultReadFindings {
    let tip = *node
        .timeline
        .last()
        .expect("bootstrap 至少写下 mkfs 根，时间线非空");
    match signature {
        None => NoFaultReadFindings {
            read_failed: true,
            read_is_not_the_tip: false,
        },
        Some(read) => NoFaultReadFindings {
            read_failed: false,
            read_is_not_the_tip: chosen != Some(tip) || tip_signature_of(node) != Some(read),
        },
    }
}

/// 判红的六个量，任一非 0 这一档判红（`stop_when_a_strict_criterion_is_red`）。
#[derive(Clone, Copy, Debug, Default)]
struct StrictCriterionReadings {
    content_differs_from_its_effective_root: u64,
    effective_root_off_the_timeline: u64,
    differs_rows_with_nothing_applied: u64,
    ignore_policy_content_differs_from_chosen_root: u64,
    no_fault_read_is_not_the_tip: u64,
    no_fault_read_failed: u64,
}

fn strict_criteria_that_are_red(readings: &StrictCriterionReadings) -> Vec<(&'static str, u64)> {
    [
        (
            "content_differs_from_its_effective_root",
            readings.content_differs_from_its_effective_root,
        ),
        (
            "effective_root_off_the_timeline",
            readings.effective_root_off_the_timeline,
        ),
        (
            "differs_rows_with_nothing_applied",
            readings.differs_rows_with_nothing_applied,
        ),
        (
            "ignore_policy_content_differs_from_chosen_root",
            readings.ignore_policy_content_differs_from_chosen_root,
        ),
        (
            "no_fault_read_is_not_the_tip",
            readings.no_fault_read_is_not_the_tip,
        ),
        ("no_fault_read_failed", readings.no_fault_read_failed),
    ]
    .into_iter()
    .filter(|(_criterion, count)| *count > 0)
    .collect()
}

/// (a)(b)(c) 三处对着时间线尾的读数，`faulted_cold_content_differs_from_its_root` 比的是所选根，只是读数。
#[derive(Default)]
struct LedgerDiagnostics {
    no_fault_reads_checked: std::sync::atomic::AtomicU64,
    no_fault_reads_just_after_a_rollback: std::sync::atomic::AtomicU64,
    no_fault_read_failed: std::sync::atomic::AtomicU64,
    no_fault_read_is_not_the_tip: std::sync::atomic::AtomicU64,
    faulted_cold_reads_checked: std::sync::atomic::AtomicU64,
    faulted_cold_read_failed: std::sync::atomic::AtomicU64,
    faulted_cold_chosen_root_is_not_the_tip: std::sync::atomic::AtomicU64,
    faulted_cold_chosen_root_off_the_timeline: std::sync::atomic::AtomicU64,
    faulted_cold_content_differs_from_its_root: std::sync::atomic::AtomicU64,
    faulted_cold_content_is_not_the_tip_with_rollback: std::sync::atomic::AtomicU64,
    faulted_cold_content_is_not_the_tip_without_rollback: std::sync::atomic::AtomicU64,
    mount_then_reads_checked: std::sync::atomic::AtomicU64,
    mount_then_content_is_not_the_tip: std::sync::atomic::AtomicU64,
    after_fault_removed_reads_checked: std::sync::atomic::AtomicU64,
    after_fault_removed_content_is_not_the_tip_with_rollback: std::sync::atomic::AtomicU64,
    after_fault_removed_content_is_not_the_tip_without_rollback: std::sync::atomic::AtomicU64,
}

static LEDGER_DIAGNOSTICS: std::sync::LazyLock<LedgerDiagnostics> =
    std::sync::LazyLock::new(LedgerDiagnostics::default);

/// 带故障冷读对着实际走的根的读数，与不看 journal 那一臂的读数。
#[derive(Default)]
struct EffectiveRootDiagnostics {
    reads_checked: std::sync::atomic::AtomicU64,
    effective_root_is_not_the_tip: std::sync::atomic::AtomicU64,
    effective_root_off_the_timeline: std::sync::atomic::AtomicU64,
    content_differs_from_its_effective_root: std::sync::atomic::AtomicU64,
    differs_from_chosen_root_rows: std::sync::atomic::AtomicU64,
    differs_rows_with_nothing_applied: std::sync::atomic::AtomicU64,
    ignore_policy_reads_checked: std::sync::atomic::AtomicU64,
    ignore_policy_read_failed: std::sync::atomic::AtomicU64,
    ignore_policy_chosen_root_changed: std::sync::atomic::AtomicU64,
    ignore_policy_content_differs_from_chosen_root: std::sync::atomic::AtomicU64,
    ignore_policy_content_is_not_the_tip: std::sync::atomic::AtomicU64,
}

static EFFECTIVE_ROOT_DIAGNOSTICS: std::sync::LazyLock<EffectiveRootDiagnostics> =
    std::sync::LazyLock::new(EffectiveRootDiagnostics::default);

/// 被拒的步进按「步、错误开头」计数：看得见枚举域里哪几步被产品拒在哪。
static REFUSED_STEPS: std::sync::Mutex<BTreeMap<String, u64>> =
    std::sync::Mutex::new(BTreeMap::new());
/// 逐条列 V2 违例时每个权重已经打了多少条。
static LISTED_VIOLATIONS: std::sync::Mutex<BTreeMap<u32, u64>> =
    std::sync::Mutex::new(BTreeMap::new());
/// 逐条列 V2 违例的上限（每个权重），总数看 `fault_set_violation_by_weight`。
const LISTED_VIOLATIONS_AT_ONE_WEIGHT: u64 = 12_000;
/// 严格判据违例已经打了多少条。
static LISTED_STRICT_CRITERION_VIOLATIONS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
/// 逐条列严格判据违例的上限，总数看 `e158_effective_root_summary` 与 `e158_ledger_diagnostics_summary`。
const LISTED_STRICT_CRITERION_VIOLATIONS_LIMIT: u64 = 12_000;

fn add_one_to_counter(counter: &std::sync::atomic::AtomicU64) {
    counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

fn count_refused_step(step: &str, error: &str) {
    let error_head: String = error
        .chars()
        .take_while(|character| !matches!(character, '{' | '(' | ' '))
        .collect();
    let error_word_after_the_head: String = error
        .split_whitespace()
        .nth(1)
        .unwrap_or("")
        .chars()
        .take_while(|character| !matches!(character, '{' | '('))
        .collect();
    *REFUSED_STEPS
        .lock()
        .expect("这把计数锁里没有会 panic 的代码，不会中毒")
        .entry(format!("{step} {error_head} {error_word_after_the_head}"))
        .or_insert(0) += 1;
}

/// 有严格判据违例才打一行；干净的一趟一行都不打。
fn list_strict_criterion_violation(
    criterion: &str,
    node: &SimNode,
    target: RootRingSlotTarget,
    readings_of_this_read: &str,
) {
    let listed_before_this_row =
        LISTED_STRICT_CRITERION_VIOLATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if listed_before_this_row >= LISTED_STRICT_CRITERION_VIOLATIONS_LIMIT {
        return;
    }
    let slots: Vec<(u64, u64)> = target
        .named_slots
        .named_slots()
        .iter()
        .map(|slot| (slot.region, slot.slot))
        .collect();
    emit_result(&format!(
        "name=e158_strict_criterion_violation criterion={criterion} fault_set={slots:?} path={:?} tip={:?} {readings_of_this_read}",
        node.path,
        node.timeline.last(),
    ));
}

/// 不看 journal 读一次：与所选根配不配、走没走到更新版本，证「内容走到更新版本」靠的是重放。
fn recover_ignoring_the_journal(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    target: RootRingSlotTarget,
) -> RecoveryReport {
    let reader = PoolReaderWithUnreadableRootRingSlots::new(devices, target);
    recover(&reader, JournalPolicy::Ignore)
}

/// 只判带故障的那些读。所选根不是尾的，再不看 journal 读一次同一个故障集合。
fn diagnose_effective_root(
    node: &SimNode,
    target: RootRingSlotTarget,
    report: &RecoveryReport,
    devices: &[(DeviceIdentity, SparseBlockDevice)],
) {
    if target.named_slots == NamedRootRingSlots::NONE {
        return;
    }
    let Some(read) = read_signature_of(&report.outcome) else {
        return;
    };
    let diagnostics = &*EFFECTIVE_ROOT_DIAGNOSTICS;
    add_one_to_counter(&diagnostics.reads_checked);
    let tip = *node
        .timeline
        .last()
        .expect("bootstrap 至少写下 mkfs 根，时间线非空");
    let chosen = chosen_root_of(&report.outcome);
    let effective = report
        .effective_root
        .map(|(instance, checkpoint_txg)| (instance.0, checkpoint_txg.0));
    let findings = judge_read_against_its_effective_root(
        node,
        chosen,
        effective,
        &read,
        report.journal.prefix_applied,
    );
    if findings.effective_root_is_not_the_tip {
        add_one_to_counter(&diagnostics.effective_root_is_not_the_tip);
    }
    if findings.differs_from_chosen_root {
        add_one_to_counter(&diagnostics.differs_from_chosen_root_rows);
    }
    let red_counters_of_this_read = [
        (
            "effective_root_off_the_timeline",
            findings.effective_root_off_the_timeline,
            &diagnostics.effective_root_off_the_timeline,
        ),
        (
            "content_differs_from_its_effective_root",
            findings.content_differs_from_its_effective_root,
            &diagnostics.content_differs_from_its_effective_root,
        ),
        (
            "differs_rows_with_nothing_applied",
            findings.differs_with_nothing_applied,
            &diagnostics.differs_rows_with_nothing_applied,
        ),
    ];
    for (criterion, is_red, counter) in red_counters_of_this_read {
        if is_red {
            add_one_to_counter(counter);
            list_strict_criterion_violation(
                criterion,
                node,
                target,
                &format!(
                    "chosen={chosen:?} effective={effective:?} prefix_applied={} above_water={} read_is_the_tip_content={}",
                    report.journal.prefix_applied,
                    report.journal.above_water,
                    tip_signature_of(node) == Some(&read),
                ),
            );
        }
    }
    if chosen == Some(tip) {
        return;
    }
    add_one_to_counter(&diagnostics.ignore_policy_reads_checked);
    let ignoring = recover_ignoring_the_journal(devices, target);
    let Some(read_when_ignoring) = read_signature_of(&ignoring.outcome) else {
        add_one_to_counter(&diagnostics.ignore_policy_read_failed);
        return;
    };
    let chosen_when_ignoring = chosen_root_of(&ignoring.outcome);
    let ignore_findings = judge_read_that_ignores_the_journal(
        node,
        chosen,
        chosen_when_ignoring,
        &read_when_ignoring,
    );
    if ignore_findings.chosen_root_changed {
        add_one_to_counter(&diagnostics.ignore_policy_chosen_root_changed);
    }
    if ignore_findings.content_is_not_the_tip {
        add_one_to_counter(&diagnostics.ignore_policy_content_is_not_the_tip);
    }
    if ignore_findings.content_differs_from_chosen_root {
        add_one_to_counter(&diagnostics.ignore_policy_content_differs_from_chosen_root);
        list_strict_criterion_violation(
            "ignore_policy_content_differs_from_chosen_root",
            node,
            target,
            &format!("chosen={chosen:?} chosen_when_ignoring={chosen_when_ignoring:?}"),
        );
    }
}

/// (a) 那一处的读数。无故障：所选根与内容都必须是时间线尾的。
/// 带读故障：所选根不是尾、内容不是尾、内容与所选根不同，各自计数，都不判红。
fn diagnose_cold_recover(node: &SimNode, target: RootRingSlotTarget, outcome: &RecoveryOutcome) {
    let diagnostics = &*LEDGER_DIAGNOSTICS;
    let chosen = chosen_root_of(outcome);
    let signature = read_signature_of(outcome);
    if target.named_slots == NamedRootRingSlots::NONE {
        add_one_to_counter(&diagnostics.no_fault_reads_checked);
        if matches!(node.path.last(), Some(HistoryStep::RollbackTo(..))) {
            add_one_to_counter(&diagnostics.no_fault_reads_just_after_a_rollback);
        }
        let findings = judge_no_fault_read(node, chosen, signature.as_ref());
        let red_counters_of_this_read = [
            (
                "no_fault_read_failed",
                findings.read_failed,
                &diagnostics.no_fault_read_failed,
            ),
            (
                "no_fault_read_is_not_the_tip",
                findings.read_is_not_the_tip,
                &diagnostics.no_fault_read_is_not_the_tip,
            ),
        ];
        for (criterion, is_red, counter) in red_counters_of_this_read {
            if is_red {
                add_one_to_counter(counter);
                list_strict_criterion_violation(
                    criterion,
                    node,
                    target,
                    &format!("chosen={chosen:?}"),
                );
            }
        }
        return;
    }
    add_one_to_counter(&diagnostics.faulted_cold_reads_checked);
    let Some(read) = &signature else {
        add_one_to_counter(&diagnostics.faulted_cold_read_failed);
        return;
    };
    let tip = *node
        .timeline
        .last()
        .expect("bootstrap 至少写下 mkfs 根，时间线非空");
    if chosen != Some(tip) {
        add_one_to_counter(&diagnostics.faulted_cold_chosen_root_is_not_the_tip);
    }
    if let Some(chosen_root) = chosen {
        if !node.timeline.contains(&chosen_root) {
            add_one_to_counter(&diagnostics.faulted_cold_chosen_root_off_the_timeline);
        } else if node.content_by_root.get(&chosen_root) != Some(read) {
            add_one_to_counter(&diagnostics.faulted_cold_content_differs_from_its_root);
        }
    }
    if tip_signature_of(node) != Some(read) {
        if node.rollback_events.is_empty() {
            add_one_to_counter(&diagnostics.faulted_cold_content_is_not_the_tip_without_rollback);
        } else {
            add_one_to_counter(&diagnostics.faulted_cold_content_is_not_the_tip_with_rollback);
        }
    }
}

/// (b) 带着故障可写挂载之后的读：内容不是尾的内容就计数。
fn diagnose_mount_then_recover(node: &SimNode, outcome: &RecoveryOutcome) {
    let diagnostics = &*LEDGER_DIAGNOSTICS;
    add_one_to_counter(&diagnostics.mount_then_reads_checked);
    if let Some(read) = read_signature_of(outcome) {
        if tip_signature_of(node) != Some(&read) {
            add_one_to_counter(&diagnostics.mount_then_content_is_not_the_tip);
        }
    }
}

/// (c) 撤掉故障之后的读：带着故障的那次可写挂载已经写了盘，内容不是尾的内容就是已确认的那一版被盖掉了。
fn diagnose_recover_after_fault_removed(node: &SimNode, outcome: &RecoveryOutcome) {
    let diagnostics = &*LEDGER_DIAGNOSTICS;
    add_one_to_counter(&diagnostics.after_fault_removed_reads_checked);
    if let Some(read) = read_signature_of(outcome) {
        if tip_signature_of(node) != Some(&read) {
            if node.rollback_events.is_empty() {
                add_one_to_counter(
                    &diagnostics.after_fault_removed_content_is_not_the_tip_without_rollback,
                );
            } else {
                add_one_to_counter(
                    &diagnostics.after_fault_removed_content_is_not_the_tip_with_rollback,
                );
            }
        }
    }
}

/// 逐条列 V2 违例：路径、所选根、三处各自红没红。没有违例不打。
fn list_violation(
    node: &SimNode,
    weight: u32,
    fault_set: &[RootRingSlot],
    checkpoints: &ViolationCheckpoints,
) {
    let point_is_off = |point: Option<ViolationAtOneCheckpoint>| {
        point.is_some_and(|violation| violation.content_off_timeline)
    };
    let cold = point_is_off(checkpoints.cold_recover_with_fault);
    let mounted = point_is_off(checkpoints.mount_then_recover_with_fault);
    let removed = point_is_off(checkpoints.recover_after_fault_removed);
    if !(cold || mounted || removed) {
        return;
    }
    let mut listed = LISTED_VIOLATIONS
        .lock()
        .expect("这把计数锁里没有会 panic 的代码，不会中毒");
    let listed_at_this_weight = listed.entry(weight).or_insert(0);
    if *listed_at_this_weight >= LISTED_VIOLATIONS_AT_ONE_WEIGHT {
        return;
    }
    *listed_at_this_weight += 1;
    let slots: Vec<(u64, u64)> = fault_set
        .iter()
        .map(|slot| (slot.region, slot.slot))
        .collect();
    emit_result(&format!(
        "name=e158_ledger_diagnostics_violation weight={weight} fault_set={slots:?} path={:?} tip={:?} cold_recover_chosen_root={:?} cold_off={cold} mount_off={mounted} removed_off={removed}",
        node.path,
        node.timeline.last(),
        checkpoints.cold_recover_chosen_root,
    ));
}

fn counter_reading(counter: &std::sync::atomic::AtomicU64) -> u64 {
    counter.load(std::sync::atomic::Ordering::Relaxed)
}

fn emit_effective_root_diagnostics() {
    let diagnostics = &*EFFECTIVE_ROOT_DIAGNOSTICS;
    emit_result(&format!(
        "name=e158_effective_root_summary reads_checked={} effective_root_is_not_the_tip={} effective_root_off_the_timeline={} content_differs_from_its_effective_root={} differs_from_chosen_root_rows={} differs_rows_with_nothing_applied={} ignore_policy_reads_checked={} ignore_policy_read_failed={} ignore_policy_chosen_root_changed={} ignore_policy_content_differs_from_chosen_root={} ignore_policy_content_is_not_the_tip={}",
        counter_reading(&diagnostics.reads_checked),
        counter_reading(&diagnostics.effective_root_is_not_the_tip),
        counter_reading(&diagnostics.effective_root_off_the_timeline),
        counter_reading(&diagnostics.content_differs_from_its_effective_root),
        counter_reading(&diagnostics.differs_from_chosen_root_rows),
        counter_reading(&diagnostics.differs_rows_with_nothing_applied),
        counter_reading(&diagnostics.ignore_policy_reads_checked),
        counter_reading(&diagnostics.ignore_policy_read_failed),
        counter_reading(&diagnostics.ignore_policy_chosen_root_changed),
        counter_reading(&diagnostics.ignore_policy_content_differs_from_chosen_root),
        counter_reading(&diagnostics.ignore_policy_content_is_not_the_tip),
    ));
}

fn emit_ledger_diagnostics() {
    emit_effective_root_diagnostics();
    let diagnostics = &*LEDGER_DIAGNOSTICS;
    emit_result(&format!(
        "name=e158_ledger_diagnostics_summary no_fault_reads_checked={} no_fault_reads_just_after_a_rollback={} no_fault_read_failed={} no_fault_read_is_not_the_tip={} faulted_cold_reads_checked={} faulted_cold_read_failed={} faulted_cold_chosen_root_is_not_the_tip={} faulted_cold_chosen_root_off_the_timeline={} faulted_cold_content_differs_from_its_root={} faulted_cold_content_is_not_the_tip_with_rollback={} faulted_cold_content_is_not_the_tip_without_rollback={} mount_then_reads_checked={} mount_then_content_is_not_the_tip={} after_fault_removed_reads_checked={} after_fault_removed_content_is_not_the_tip_with_rollback={} after_fault_removed_content_is_not_the_tip_without_rollback={}",
        counter_reading(&diagnostics.no_fault_reads_checked),
        counter_reading(&diagnostics.no_fault_reads_just_after_a_rollback),
        counter_reading(&diagnostics.no_fault_read_failed),
        counter_reading(&diagnostics.no_fault_read_is_not_the_tip),
        counter_reading(&diagnostics.faulted_cold_reads_checked),
        counter_reading(&diagnostics.faulted_cold_read_failed),
        counter_reading(&diagnostics.faulted_cold_chosen_root_is_not_the_tip),
        counter_reading(&diagnostics.faulted_cold_chosen_root_off_the_timeline),
        counter_reading(&diagnostics.faulted_cold_content_differs_from_its_root),
        counter_reading(&diagnostics.faulted_cold_content_is_not_the_tip_with_rollback),
        counter_reading(&diagnostics.faulted_cold_content_is_not_the_tip_without_rollback),
        counter_reading(&diagnostics.mount_then_reads_checked),
        counter_reading(&diagnostics.mount_then_content_is_not_the_tip),
        counter_reading(&diagnostics.after_fault_removed_reads_checked),
        counter_reading(&diagnostics.after_fault_removed_content_is_not_the_tip_with_rollback),
        counter_reading(&diagnostics.after_fault_removed_content_is_not_the_tip_without_rollback),
    ));
    for (step_and_error, count) in REFUSED_STEPS
        .lock()
        .expect("这把计数锁里没有会 panic 的代码，不会中毒")
        .iter()
    {
        emit_result(&format!(
            "name=e158_ledger_diagnostics_refused step_and_error={step_and_error:?} count={count}"
        ));
    }
}

fn strict_criterion_readings_of_this_process() -> StrictCriterionReadings {
    StrictCriterionReadings {
        content_differs_from_its_effective_root: counter_reading(
            &EFFECTIVE_ROOT_DIAGNOSTICS.content_differs_from_its_effective_root,
        ),
        effective_root_off_the_timeline: counter_reading(
            &EFFECTIVE_ROOT_DIAGNOSTICS.effective_root_off_the_timeline,
        ),
        differs_rows_with_nothing_applied: counter_reading(
            &EFFECTIVE_ROOT_DIAGNOSTICS.differs_rows_with_nothing_applied,
        ),
        ignore_policy_content_differs_from_chosen_root: counter_reading(
            &EFFECTIVE_ROOT_DIAGNOSTICS.ignore_policy_content_differs_from_chosen_root,
        ),
        no_fault_read_is_not_the_tip: counter_reading(
            &LEDGER_DIAGNOSTICS.no_fault_read_is_not_the_tip,
        ),
        no_fault_read_failed: counter_reading(&LEDGER_DIAGNOSTICS.no_fault_read_failed),
    }
}

/// 六个量任一非 0：打一行 `name=stop`、退 1，不打收尾的 `name=done`（与 V3、S5 那两道停机同一个写法）。
fn stop_when_a_strict_criterion_is_red(geometry: &Geometry) {
    let red_criteria = strict_criteria_that_are_red(&strict_criterion_readings_of_this_process());
    if red_criteria.is_empty() {
        return;
    }
    emit_result(&format!(
        "name=stop geometry={} reason=strict_criterion red_criteria={red_criteria:?} detail=\"读回的内容不是实际走的根的，或不注入故障读回的不是时间线尾\"",
        geometry.label
    ));
    std::process::exit(1);
}

/// P332 在 (a)(b)(c) 三处各自的判定：`cold_recover_with_fault` 是 (a)，
/// `mount_then_recover_with_fault` 是 (b)（`mount_writable` 之后再带着同一个故障冷启动读一次），
/// `recover_after_fault_removed` 是 (c)（撤掉故障之后的冷启动 recover）。
struct ViolationCheckpoints {
    cold_recover_chosen_root: Option<TimelineRoot>,
    cold_recover_with_fault: Option<ViolationAtOneCheckpoint>,
    mount_then_recover_with_fault: Option<ViolationAtOneCheckpoint>,
    recover_after_fault_removed: Option<ViolationAtOneCheckpoint>,
    cold_recover_with_fault_failed: bool,
    mount_writable_with_fault_failed: bool,
    skipped_by_the_reduction_rule: bool,
}

/// 在给定节点、给定故障集合下把 P332 的 (a)(b)(c) 各判一次。`baseline_chosen_root` 是这段历史
/// F=∅ 时 (a) 的所选根（省法要比的对象）。`devices_for_cold_recover` 由调用方对这一整个节点建一次、
/// 传进来给每一个故障集合的 (a) 复用——(a) 只读，不写这几份设备，一个节点要判几十个故障集合，
/// 每次都重新拷一遍 4 GiB 的稀疏镜像是这个函数最先量出来的挂钟大头（G0 层一里跑不完 100 秒）。
fn evaluate_violation_checkpoints(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    geometry: &Geometry,
    target: RootRingSlotTarget,
    baseline_chosen_root: Option<TimelineRoot>,
    devices_for_cold_recover: &[(DeviceIdentity, SparseBlockDevice)],
) -> ViolationCheckpoints {
    let cold_recover_reader =
        PoolReaderWithUnreadableRootRingSlots::new(devices_for_cold_recover, target);
    let cold_recover_report = recover(&cold_recover_reader, JournalPolicy::Consult);
    diagnose_effective_root(node, target, &cold_recover_report, devices_for_cold_recover);
    let cold_recover_outcome = cold_recover_report.outcome;
    let cold_recover_chosen_root = chosen_root_of(&cold_recover_outcome);
    diagnose_cold_recover(node, target, &cold_recover_outcome);
    let cold_recover_with_fault = violation_at_checkpoint(node, &cold_recover_outcome);
    let cold_recover_with_fault_failed = cold_recover_with_fault.is_none();

    if target.named_slots != NamedRootRingSlots::NONE
        && cold_recover_chosen_root == baseline_chosen_root
    {
        // 省法：这个故障集合没有改变 (a) 的所选根 ⇒ (b)(c) 与 F=∅ 那一对相同（非违例）。
        return ViolationCheckpoints {
            cold_recover_chosen_root,
            cold_recover_with_fault,
            mount_then_recover_with_fault: Some(ViolationAtOneCheckpoint::default()),
            recover_after_fault_removed: Some(ViolationAtOneCheckpoint::default()),
            cold_recover_with_fault_failed,
            mount_writable_with_fault_failed: false,
            skipped_by_the_reduction_rule: true,
        };
    }

    let plan = SharedFaultPlan::unarmed(fixed_geometry_for(geometry));
    plan.arm(FaultSchedule::every_read_of_named_root_ring_slots_fails(
        target,
    ));
    let mut faulted_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        devices_from_pool(&node.pool, IMAGE_BYTES)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, plan.clone()),
                )
            })
            .collect();
    let mount_result = mount_writable(parameters, &mut faulted_devices);
    let (mount_then_recover_with_fault, mount_writable_with_fault_failed) = match mount_result {
        Ok(_mounted) => {
            let mount_then_recover_outcome =
                recover(&faulted_devices, JournalPolicy::Consult).outcome;
            diagnose_mount_then_recover(node, &mount_then_recover_outcome);
            (
                violation_at_checkpoint(node, &mount_then_recover_outcome),
                false,
            )
        }
        Err(_error) => (None, true),
    };
    plan.disarm();
    let recover_after_fault_removed_outcome =
        recover(&faulted_devices, JournalPolicy::Consult).outcome;
    diagnose_recover_after_fault_removed(node, &recover_after_fault_removed_outcome);
    let recover_after_fault_removed =
        violation_at_checkpoint(node, &recover_after_fault_removed_outcome);

    ViolationCheckpoints {
        cold_recover_chosen_root,
        cold_recover_with_fault,
        mount_then_recover_with_fault,
        recover_after_fault_removed,
        cold_recover_with_fault_failed,
        mount_writable_with_fault_failed,
        skipped_by_the_reduction_rule: false,
    }
}

#[derive(Default)]
pub(super) struct FaultSetViolationSummary {
    pub(super) nodes_visited: u64,
    pub(super) pairs: u64,
    pairs_by_weight: BTreeMap<u32, u64>,
    violations_by_weight: BTreeMap<u32, u64>,
    pairs_with_rollback: u64,
    violations_with_rollback: u64,
    pairs_without_rollback: u64,
    violations_without_rollback: u64,
    violations_cold_recover_chosen_root_abandoned: u64,
    violations_cold_recover_content_off_timeline: u64,
    violations_mount_then_recover_chosen_root_abandoned: u64,
    violations_mount_then_recover_content_off_timeline: u64,
    violations_recover_after_fault_removed_chosen_root_abandoned: u64,
    violations_recover_after_fault_removed_content_off_timeline: u64,
    pairs_with_any_violation: u64,
    cold_recover_with_fault_failed: u64,
    mount_writable_with_fault_failed: u64,
    skipped_by_the_reduction_rule: u64,
}

fn record_violation_checkpoints_for_one_fault_set(
    summary: &mut FaultSetViolationSummary,
    weight: u32,
    has_rollback: bool,
    checkpoints: &ViolationCheckpoints,
) {
    summary.pairs += 1;
    *summary.pairs_by_weight.entry(weight).or_insert(0) += 1;
    if has_rollback {
        summary.pairs_with_rollback += 1;
    } else {
        summary.pairs_without_rollback += 1;
    }
    if checkpoints.cold_recover_with_fault_failed {
        summary.cold_recover_with_fault_failed += 1;
    }
    if checkpoints.mount_writable_with_fault_failed {
        summary.mount_writable_with_fault_failed += 1;
    }
    if checkpoints.skipped_by_the_reduction_rule {
        summary.skipped_by_the_reduction_rule += 1;
    }
    let mut any_violation = false;
    if let Some(point) = checkpoints.cold_recover_with_fault {
        if point.chosen_root_abandoned {
            summary.violations_cold_recover_chosen_root_abandoned += 1;
            any_violation = true;
        }
        if point.content_off_timeline {
            summary.violations_cold_recover_content_off_timeline += 1;
            any_violation = true;
        }
    }
    if let Some(point) = checkpoints.mount_then_recover_with_fault {
        if point.chosen_root_abandoned {
            summary.violations_mount_then_recover_chosen_root_abandoned += 1;
            any_violation = true;
        }
        if point.content_off_timeline {
            summary.violations_mount_then_recover_content_off_timeline += 1;
            any_violation = true;
        }
    }
    if let Some(point) = checkpoints.recover_after_fault_removed {
        if point.chosen_root_abandoned {
            summary.violations_recover_after_fault_removed_chosen_root_abandoned += 1;
            any_violation = true;
        }
        if point.content_off_timeline {
            summary.violations_recover_after_fault_removed_content_off_timeline += 1;
            any_violation = true;
        }
    }
    if any_violation {
        summary.pairs_with_any_violation += 1;
        *summary.violations_by_weight.entry(weight).or_insert(0) += 1;
        if has_rollback {
            summary.violations_with_rollback += 1;
        } else {
            summary.violations_without_rollback += 1;
        }
    }
}

/// 从 `items`（已按 (区域, 槽) 升序）里取 `size` 个的全部组合。
pub(super) fn combinations_of_size<Item: Clone>(items: &[Item], size: usize) -> Vec<Vec<Item>> {
    if size == 0 {
        return vec![Vec::new()];
    }
    if items.len() < size {
        return Vec::new();
    }
    let mut result = Vec::new();
    for index in 0..=items.len() - size {
        let head = items[index].clone();
        for mut tail in combinations_of_size(&items[index + 1..], size - 1) {
            let mut combo = vec![head.clone()];
            combo.append(&mut tail);
            result.push(combo);
        }
    }
    result
}

#[allow(
    clippy::too_many_arguments,
    reason = "Q3-1 的 H3 穷举需要节点、剩余深度、mkfs 参数、几何、独立解出的几何视图与汇总器六项，各自有名字有类型"
)]
fn enumerate_fault_set_violations(
    node: &SimNode,
    depth_remaining: u64,
    parameters: &MakeFilesystemParameters,
    geometry: &Geometry,
    geometry_view: &PoolGeometry,
    summary: &mut FaultSetViolationSummary,
) {
    summary.nodes_visited += 1;
    let all_roots = readable_roots_with_slots(&node.pool, geometry_view);
    let slots: Vec<RootRingSlot> = all_roots
        .iter()
        .map(|(region, slot, _)| RootRingSlot {
            region: *region,
            slot: *slot,
        })
        .collect();
    let has_rollback = !node.rollback_events.is_empty();

    let baseline_chosen_root = Some(
        *node
            .timeline
            .last()
            .expect("H3 的每个节点时间线非空：bootstrap 至少写下 mkfs 根"),
    );
    // (a) 对整个节点只读一次原始镜像；下面每个故障集合的 (a) 都复用这同一份，不逐个重拷。
    let devices_for_cold_recover = devices_from_pool(&node.pool, IMAGE_BYTES);
    let baseline_target = RootRingSlotTarget {
        named_slots: NamedRootRingSlots::NONE,
        region_devices: region_devices(),
        fixed_structure_slot_spacing: 4096,
    };
    let baseline = evaluate_violation_checkpoints(
        node,
        parameters,
        geometry,
        baseline_target,
        baseline_chosen_root,
        &devices_for_cold_recover,
    );
    record_violation_checkpoints_for_one_fault_set(summary, 0, has_rollback, &baseline);
    list_violation(node, 0, &[], &baseline);

    for weight in 1..=2usize {
        for combo in combinations_of_size(&slots, weight) {
            let target = RootRingSlotTarget {
                named_slots: NamedRootRingSlots::naming(&combo),
                region_devices: region_devices(),
                fixed_structure_slot_spacing: 4096,
            };
            let checkpoints = evaluate_violation_checkpoints(
                node,
                parameters,
                geometry,
                target,
                baseline_chosen_root,
                &devices_for_cold_recover,
            );
            record_violation_checkpoints_for_one_fault_set(
                summary,
                u32::try_from(weight).expect("weight ∈ {1,2}"),
                has_rollback,
                &checkpoints,
            );
            list_violation(
                node,
                u32::try_from(weight).expect("weight ∈ {1,2}"),
                &combo,
                &checkpoints,
            );
        }
    }

    if depth_remaining == 0 {
        return;
    }
    if node.session.is_some() {
        match apply_overwrite(node, parameters) {
            Ok(next) => enumerate_fault_set_violations(
                &next,
                depth_remaining - 1,
                parameters,
                geometry,
                geometry_view,
                summary,
            ),
            Err(error) => count_refused_step("W", &error),
        }
    }
    match apply_mount_writable(node, parameters) {
        Ok(next) => enumerate_fault_set_violations(
            &next,
            depth_remaining - 1,
            parameters,
            geometry,
            geometry_view,
            summary,
        ),
        Err(error) => count_refused_step("M", &error),
    }
    for readable_root in &readable_roots_independent(&node.pool, geometry_view) {
        let target = RollbackTarget {
            instance: InstanceGeneration(readable_root.0),
            checkpoint_txg: CheckpointTxg(readable_root.1),
        };
        match apply_mount_rollback(node, target, parameters) {
            Ok(next) => enumerate_fault_set_violations(
                &next,
                depth_remaining - 1,
                parameters,
                geometry,
                geometry_view,
                summary,
            ),
            Err(error) => count_refused_step("R", &error),
        }
    }
}

pub(super) fn run_fault_set_violation_family(
    geometry: &Geometry,
    initial_overwrite_counts: &[u64],
    sigma_length_limit: u64,
) -> FaultSetViolationSummary {
    let parameters = parameters_for(geometry);
    let mut summary = FaultSetViolationSummary::default();
    for &overwrites_before_sigma in initial_overwrite_counts {
        let root_node = match bootstrap(geometry, overwrites_before_sigma) {
            Ok(node) => node,
            Err(error) => {
                emit_result(&format!(
                    "name=fault_set_violation_bootstrap_failed geometry={} overwrites_before_sigma={overwrites_before_sigma} detail={error:?}",
                    geometry.label
                ));
                continue;
            }
        };
        let geometry_view = match independent_geometry(&root_node.pool) {
            Ok(view) => view,
            Err(error) => {
                emit_result(&format!(
                    "name=fault_set_violation_geometry_failed geometry={} detail={error:?}",
                    geometry.label
                ));
                continue;
            }
        };
        enumerate_fault_set_violations(
            &root_node,
            sigma_length_limit,
            &parameters,
            geometry,
            &geometry_view,
            &mut summary,
        );
        emit_result(&format!(
            "name=fault_set_violation_progress geometry={} overwrites_before_sigma={overwrites_before_sigma} nodes_visited_so_far={} pairs_so_far={}",
            geometry.label, summary.nodes_visited, summary.pairs
        ));
    }
    emit_result(&format!(
        "name=fault_set_violation_summary geometry={} overwrites_before_sigma={:?} sigma_length_limit={} nodes_visited={} pairs={} pairs_with_any_violation={} cold_recover_with_fault_failed={} mount_writable_with_fault_failed={} skipped_by_the_reduction_rule={}",
        geometry.label,
        initial_overwrite_counts,
        sigma_length_limit,
        summary.nodes_visited,
        summary.pairs,
        summary.pairs_with_any_violation,
        summary.cold_recover_with_fault_failed,
        summary.mount_writable_with_fault_failed,
        summary.skipped_by_the_reduction_rule
    ));
    for (weight, pairs) in &summary.pairs_by_weight {
        let violations = summary
            .violations_by_weight
            .get(weight)
            .copied()
            .unwrap_or(0);
        emit_result(&format!(
            "name=fault_set_violation_by_weight geometry={} weight={weight} pairs={pairs} violations={violations}",
            geometry.label
        ));
    }
    emit_result(&format!(
        "name=fault_set_violation_by_rollback_presence geometry={} pairs_with_rollback={} violations_with_rollback={} pairs_without_rollback={} violations_without_rollback={}",
        geometry.label,
        summary.pairs_with_rollback,
        summary.violations_with_rollback,
        summary.pairs_without_rollback,
        summary.violations_without_rollback
    ));
    emit_result(&format!(
        "name=fault_set_violation_by_checkpoint_and_predicate geometry={} cold_recover_chosen_root_abandoned={} cold_recover_content_off_timeline={} mount_then_recover_chosen_root_abandoned={} mount_then_recover_content_off_timeline={} recover_after_fault_removed_chosen_root_abandoned={} recover_after_fault_removed_content_off_timeline={}",
        geometry.label,
        summary.violations_cold_recover_chosen_root_abandoned,
        summary.violations_cold_recover_content_off_timeline,
        summary.violations_mount_then_recover_chosen_root_abandoned,
        summary.violations_mount_then_recover_content_off_timeline,
        summary.violations_recover_after_fault_removed_chosen_root_abandoned,
        summary.violations_recover_after_fault_removed_content_off_timeline
    ));
    emit_ledger_diagnostics();
    stop_when_a_strict_criterion_is_red(geometry);
    summary
}

#[cfg(test)]
mod tests;
