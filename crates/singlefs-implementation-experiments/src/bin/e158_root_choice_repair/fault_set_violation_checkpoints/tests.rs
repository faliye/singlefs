use super::*;
use crate::tests::bare_node;

/// V2：内容号匹配当前时间线上某个根 ⇒ 不违例，即便同一份内容也出现在被抛弃的根上。
#[test]
fn content_is_off_the_current_timeline_is_false_when_the_content_matches_a_root_still_on_the_timeline(
) {
    let mut content_by_root = BTreeMap::new();
    content_by_root.insert((1, 1), Some(b"a".to_vec()));
    content_by_root.insert((1, 2), Some(b"b".to_vec()));
    content_by_root.insert((2, 5), Some(b"a".to_vec())); // 被抛弃的根，内容号与 (1,1) 撞了
    let node = bare_node(vec![(1, 1), (1, 2)], content_by_root, Vec::new());
    assert!(
        !content_is_off_the_current_timeline(&node, &Some(b"a".to_vec())),
        "内容 a 匹配时间线上的 (1,1)，不该判违例"
    );
}

/// V2：内容号只匹配被抛弃的根、不匹配当前时间线上任何一个根 ⇒ 违例。
#[test]
fn content_is_off_the_current_timeline_is_true_when_the_content_only_belongs_to_an_abandoned_root()
{
    let mut content_by_root = BTreeMap::new();
    content_by_root.insert((1, 1), Some(b"a".to_vec()));
    content_by_root.insert((2, 5), Some(b"z".to_vec())); // 被抛弃、不在 timeline 上
    let node = bare_node(vec![(1, 1)], content_by_root, Vec::new());
    assert!(
        content_is_off_the_current_timeline(&node, &Some(b"z".to_vec())),
        "内容 z 只在被抛弃的 (2,5) 上出现过，应当判违例"
    );
}

/// V2 的 `None`（没有文件）签名同样按内容号处理：时间线上有一个根也是「没有文件」才不算违例。
#[test]
fn content_is_off_the_current_timeline_treats_no_file_as_a_content_signature_too() {
    let mut content_by_root = BTreeMap::new();
    content_by_root.insert((0, 0), None);
    content_by_root.insert((1, 1), Some(b"a".to_vec()));
    let node_with_no_file_on_timeline =
        bare_node(vec![(0, 0)], content_by_root.clone(), Vec::new());
    assert!(!content_is_off_the_current_timeline(
        &node_with_no_file_on_timeline,
        &None
    ));
    let node_without_no_file_on_timeline = bare_node(vec![(1, 1)], content_by_root, Vec::new());
    assert!(content_is_off_the_current_timeline(
        &node_without_no_file_on_timeline,
        &None
    ));
}

/// V1：`is_abandoned` 要在**全部**回退事件的抛弃集合里查，不是只查最近一次。
#[test]
fn is_abandoned_checks_every_recorded_rollback_event_not_only_the_latest() {
    let node = bare_node(
        vec![(3, 30)],
        BTreeMap::new(),
        vec![
            RollbackEvent {
                abandoned: BTreeSet::from([(1, 5), (1, 6)]),
            },
            RollbackEvent {
                abandoned: BTreeSet::from([(2, 9)]),
            },
        ],
    );
    assert!(is_abandoned(&node, (1, 5)), "第一次回退抛弃的根也要算");
    assert!(is_abandoned(&node, (2, 9)), "第二次回退抛弃的根要算");
    assert!(
        !is_abandoned(&node, (3, 30)),
        "当前时间线尾不该被判成被抛弃"
    );
}

/// P332：`NoFile` 结局恒不判 V2（没有内容号可比），V1 照判。
#[test]
fn violation_at_checkpoint_never_flags_content_off_timeline_on_a_no_file_outcome() {
    let node = bare_node(
        vec![(1, 1)],
        BTreeMap::new(),
        vec![RollbackEvent {
            abandoned: BTreeSet::from([(1, 9)]),
        }],
    );
    let outcome = RecoveryOutcome::NoFile {
        root: (InstanceGeneration(1), CheckpointTxg(9)),
    };
    let point = violation_at_checkpoint(&node, &outcome).expect("NoFile 有所选根，判得出 P332");
    assert!(
        point.chosen_root_abandoned,
        "所选根 (1,9) 在被抛弃集合里，V1 该为真"
    );
    assert!(!point.content_off_timeline, "NoFile 没有内容号，V2 恒为假");
}

/// P332：`Failed` 结局没有所选根，交回 `None`（这一处不判 P332，另计入失败计数，不当违例）。
#[test]
fn violation_at_checkpoint_returns_none_on_a_failed_outcome() {
    let node = bare_node(vec![(1, 1)], BTreeMap::new(), Vec::new());
    let outcome = RecoveryOutcome::Failed {
        root: None,
        failure: singlefs_core::recovery::RecoveryFailure::NoValidRoot,
    };
    assert!(violation_at_checkpoint(&node, &outcome).is_none());
}

/// `combinations_of_size`：从 3 个里取 2 个应当恰好 3 种，取 0 个是空集合本身，取超过总数是空列表。
#[test]
fn combinations_of_size_enumerates_every_subset_of_the_requested_size() {
    let items = vec![1, 2, 3];
    let pairs = combinations_of_size(&items, 2);
    assert_eq!(pairs, vec![vec![1, 2], vec![1, 3], vec![2, 3]]);
    assert_eq!(combinations_of_size(&items, 0), vec![Vec::<i32>::new()]);
    assert_eq!(combinations_of_size(&items, 4), Vec::<Vec<i32>>::new());
}

/// 手写的如实账本：(1,1) 内容 older → (1,2) 内容 newer → 回退到 (1,1) 那次发布的根 (2,3)，内容 older。
fn ledger_rolled_back_to_the_older_content() -> SimNode {
    bare_node(
        vec![(1, 1), (1, 2), (2, 3)],
        BTreeMap::from([
            ((1, 1), Some(b"older".to_vec())),
            ((1, 2), Some(b"newer".to_vec())),
            ((2, 3), Some(b"older".to_vec())),
        ]),
        vec![RollbackEvent {
            abandoned: BTreeSet::new(),
        }],
    )
}

/// 内容与所选根不同、与实际走的根相同：重放走过了所选根，不是违例（D23（journal 的角色与格式） 已定项 14）。
#[test]
fn effective_root_judge_is_clean_when_the_read_matches_the_effective_root_and_differs_from_the_chosen_root(
) {
    let findings = judge_read_against_its_effective_root(
        &ledger_rolled_back_to_the_older_content(),
        Some((1, 2)),
        Some((2, 3)),
        &Some(b"older".to_vec()),
        1,
    );
    assert_eq!(
        findings,
        EffectiveRootFindings {
            effective_root_is_not_the_tip: false,
            effective_root_off_the_timeline: false,
            content_differs_from_its_effective_root: false,
            differs_from_chosen_root: true,
            differs_with_nothing_applied: false,
        }
    );
}

#[test]
fn effective_root_judge_reports_a_read_that_is_not_the_content_of_the_effective_root() {
    let findings = judge_read_against_its_effective_root(
        &ledger_rolled_back_to_the_older_content(),
        Some((2, 3)),
        Some((2, 3)),
        &Some(b"newer".to_vec()),
        0,
    );
    assert!(findings.content_differs_from_its_effective_root);
}

#[test]
fn effective_root_judge_reports_an_effective_root_that_is_not_on_the_timeline() {
    let findings = judge_read_against_its_effective_root(
        &ledger_rolled_back_to_the_older_content(),
        Some((2, 3)),
        Some((9, 9)),
        &Some(b"older".to_vec()),
        1,
    );
    assert!(findings.effective_root_off_the_timeline);
    assert!(!findings.content_differs_from_its_effective_root);
}

/// 内容走到了比所选根更新的版本，却一条记录都没施加：解释不了内容从哪来。
#[test]
fn effective_root_judge_reports_a_differing_read_with_no_record_applied() {
    let findings = judge_read_against_its_effective_root(
        &ledger_rolled_back_to_the_older_content(),
        Some((1, 2)),
        Some((2, 3)),
        &Some(b"older".to_vec()),
        0,
    );
    assert!(findings.differs_with_nothing_applied);
}

#[test]
fn ignore_policy_judge_is_clean_when_the_read_is_the_content_of_the_chosen_root() {
    let findings = judge_read_that_ignores_the_journal(
        &ledger_rolled_back_to_the_older_content(),
        Some((1, 2)),
        Some((1, 2)),
        &Some(b"newer".to_vec()),
    );
    assert_eq!(
        findings,
        IgnorePolicyFindings {
            chosen_root_changed: false,
            content_differs_from_chosen_root: false,
            content_is_not_the_tip: true,
        }
    );
}

#[test]
fn ignore_policy_judge_reports_a_read_that_is_not_the_content_of_the_chosen_root() {
    let findings = judge_read_that_ignores_the_journal(
        &ledger_rolled_back_to_the_older_content(),
        Some((1, 2)),
        Some((1, 2)),
        &Some(b"older".to_vec()),
    );
    assert!(findings.content_differs_from_chosen_root);
}

#[test]
fn ignore_policy_judge_reports_a_chosen_root_changed_by_ignoring_the_journal() {
    let findings = judge_read_that_ignores_the_journal(
        &ledger_rolled_back_to_the_older_content(),
        Some((1, 2)),
        Some((1, 1)),
        &Some(b"older".to_vec()),
    );
    assert!(findings.chosen_root_changed);
}

#[test]
fn no_fault_judge_is_clean_when_the_read_is_the_tip() {
    let read = Some(b"older".to_vec());
    let findings = judge_no_fault_read(
        &ledger_rolled_back_to_the_older_content(),
        Some((2, 3)),
        Some(&read),
    );
    assert_eq!(
        findings,
        NoFaultReadFindings {
            read_failed: false,
            read_is_not_the_tip: false,
        }
    );
}

#[test]
fn no_fault_judge_reports_a_read_whose_content_is_not_the_tip() {
    let read = Some(b"newer".to_vec());
    let findings = judge_no_fault_read(
        &ledger_rolled_back_to_the_older_content(),
        Some((2, 3)),
        Some(&read),
    );
    assert!(findings.read_is_not_the_tip);
}

/// (1,1) 的内容与尾相同，所选根却不是尾：不注入故障不该落回去。
#[test]
fn no_fault_judge_reports_a_chosen_root_that_is_not_the_tip_even_with_the_tip_content() {
    let read = Some(b"older".to_vec());
    let findings = judge_no_fault_read(
        &ledger_rolled_back_to_the_older_content(),
        Some((1, 1)),
        Some(&read),
    );
    assert!(findings.read_is_not_the_tip);
}

#[test]
fn no_fault_judge_reports_a_failed_read() {
    let findings = judge_no_fault_read(&ledger_rolled_back_to_the_older_content(), None, None);
    assert!(findings.read_failed);
}

#[test]
fn strict_criteria_are_clean_when_all_six_readings_are_zero() {
    assert!(strict_criteria_that_are_red(&StrictCriterionReadings::default()).is_empty());
}

#[test]
fn strict_criteria_red_on_content_differs_from_its_effective_root() {
    let readings = StrictCriterionReadings {
        content_differs_from_its_effective_root: 7,
        ..StrictCriterionReadings::default()
    };
    assert_eq!(
        strict_criteria_that_are_red(&readings),
        vec![("content_differs_from_its_effective_root", 7)]
    );
}

#[test]
fn strict_criteria_red_on_effective_root_off_the_timeline() {
    let readings = StrictCriterionReadings {
        effective_root_off_the_timeline: 7,
        ..StrictCriterionReadings::default()
    };
    assert_eq!(
        strict_criteria_that_are_red(&readings),
        vec![("effective_root_off_the_timeline", 7)]
    );
}

#[test]
fn strict_criteria_red_on_differs_rows_with_nothing_applied() {
    let readings = StrictCriterionReadings {
        differs_rows_with_nothing_applied: 7,
        ..StrictCriterionReadings::default()
    };
    assert_eq!(
        strict_criteria_that_are_red(&readings),
        vec![("differs_rows_with_nothing_applied", 7)]
    );
}

#[test]
fn strict_criteria_red_on_ignore_policy_content_differs_from_chosen_root() {
    let readings = StrictCriterionReadings {
        ignore_policy_content_differs_from_chosen_root: 7,
        ..StrictCriterionReadings::default()
    };
    assert_eq!(
        strict_criteria_that_are_red(&readings),
        vec![("ignore_policy_content_differs_from_chosen_root", 7)]
    );
}

#[test]
fn strict_criteria_red_on_no_fault_read_is_not_the_tip() {
    let readings = StrictCriterionReadings {
        no_fault_read_is_not_the_tip: 7,
        ..StrictCriterionReadings::default()
    };
    assert_eq!(
        strict_criteria_that_are_red(&readings),
        vec![("no_fault_read_is_not_the_tip", 7)]
    );
}

#[test]
fn strict_criteria_red_on_no_fault_read_failed() {
    let readings = StrictCriterionReadings {
        no_fault_read_failed: 7,
        ..StrictCriterionReadings::default()
    };
    assert_eq!(
        strict_criteria_that_are_red(&readings),
        vec![("no_fault_read_failed", 7)]
    );
}

/// 真实产品上的搭建：`bootstrap` 里一次覆盖写，时间线尾就是覆盖写那条根。
fn node_with_one_real_overwrite() -> SimNode {
    bootstrap(&GEOMETRY_PRIMARY, 1).expect("bootstrap 应当成功")
}

/// 真实产品上回退到 bootstrap 写的文件那一版（带文件的根是回退的合法目标）。
fn node_rolled_back_to_the_bootstrap_file() -> SimNode {
    let node = node_with_one_real_overwrite();
    let root_holding_the_bootstrap_file = node
        .timeline
        .iter()
        .find(|root| matches!(node.content_by_root.get(root), Some(Some(_))))
        .copied()
        .expect("bootstrap 写了文件，时间线上有带文件的根");
    apply_mount_rollback(
        &node,
        RollbackTarget {
            instance: InstanceGeneration(root_holding_the_bootstrap_file.0),
            checkpoint_txg: CheckpointTxg(root_holding_the_bootstrap_file.1),
        },
        &parameters_for(&GEOMETRY_PRIMARY),
    )
    .expect("回退到带文件的根应当成功")
}

fn slots_holding_the_tip(node: &SimNode) -> NamedRootRingSlots {
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
    let tip = *node
        .timeline
        .last()
        .expect("bootstrap 至少写下 mkfs 根，时间线非空");
    let slots: Vec<RootRingSlot> = readable_roots_with_slots(&node.pool, &geometry_view)
        .into_iter()
        .filter(|(_region, _slot, root)| *root == tip)
        .map(|(region, slot, _root)| RootRingSlot { region, slot })
        .collect();
    assert!(!slots.is_empty(), "时间线尾 {tip:?} 该读得出");
    NamedRootRingSlots::naming(&slots)
}

fn target_naming(named_slots: NamedRootRingSlots) -> RootRingSlotTarget {
    RootRingSlotTarget {
        named_slots,
        region_devices: region_devices(),
        fixed_structure_slot_spacing: 4096,
    }
}

fn recover_consulting_the_journal(
    node: &SimNode,
    named_slots: NamedRootRingSlots,
) -> RecoveryReport {
    let devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let reader =
        PoolReaderWithUnreadableRootRingSlots::new(devices.as_slice(), target_naming(named_slots));
    recover(&reader, JournalPolicy::Consult)
}

fn assert_no_fault_read_is_the_tip(node: &SimNode) {
    let report = recover_consulting_the_journal(node, NamedRootRingSlots::NONE);
    let signature = read_signature_of(&report.outcome);
    assert_eq!(
        judge_no_fault_read(node, chosen_root_of(&report.outcome), signature.as_ref()),
        NoFaultReadFindings {
            read_failed: false,
            read_is_not_the_tip: false,
        },
        "路径 {:?}",
        node.path
    );
}

fn findings_with_the_tip_slot_unreadable(node: &SimNode) -> EffectiveRootFindings {
    let report = recover_consulting_the_journal(node, slots_holding_the_tip(node));
    let read = read_signature_of(&report.outcome).expect("藏掉一条根，恢复应当读得成");
    judge_read_against_its_effective_root(
        node,
        chosen_root_of(&report.outcome),
        report
            .effective_root
            .map(|(instance, checkpoint_txg)| (instance.0, checkpoint_txg.0)),
        &read,
        report.journal.prefix_applied,
    )
}

#[test]
fn no_fault_read_after_a_real_rollback_is_the_tip_on_the_ledger() {
    assert_no_fault_read_is_the_tip(&node_rolled_back_to_the_bootstrap_file());
}

#[test]
fn no_fault_read_after_a_real_overwrite_following_a_rollback_is_the_tip_on_the_ledger() {
    let node = apply_overwrite(
        &node_rolled_back_to_the_bootstrap_file(),
        &parameters_for(&GEOMETRY_PRIMARY),
    )
    .expect("回退之后的覆盖写应当成功");
    assert_no_fault_read_is_the_tip(&node);
}

#[test]
fn real_rollback_records_no_abandoned_root() {
    let node = node_rolled_back_to_the_bootstrap_file();
    let event = node.rollback_events.last().expect("刚回退过一次");
    assert!(event.abandoned.is_empty(), "回退是一次向前发布，不抛弃根");
}

/// 藏掉尾那一槽：所选根落回前一条，重放读到尾的内容，与所选根不同、与实际走的根相同。
#[test]
fn hiding_the_tip_after_a_real_overwrite_reads_the_content_of_the_effective_root() {
    assert_eq!(
        findings_with_the_tip_slot_unreadable(&node_with_one_real_overwrite()),
        EffectiveRootFindings {
            effective_root_is_not_the_tip: false,
            effective_root_off_the_timeline: false,
            content_differs_from_its_effective_root: false,
            differs_from_chosen_root: true,
            differs_with_nothing_applied: false,
        }
    );
}

#[test]
fn hiding_the_rollback_root_after_a_real_rollback_reads_the_content_of_the_effective_root() {
    let findings = findings_with_the_tip_slot_unreadable(&node_rolled_back_to_the_bootstrap_file());
    assert!(!findings.effective_root_off_the_timeline);
    assert!(!findings.content_differs_from_its_effective_root);
    assert!(!findings.differs_with_nothing_applied);
}

/// 同一个故障集合不看 journal：读回所选根自己的内容，不是尾的内容。
#[test]
fn hiding_the_tip_after_a_real_overwrite_and_ignoring_the_journal_reads_the_chosen_root() {
    let node = node_with_one_real_overwrite();
    let named_slots = slots_holding_the_tip(&node);
    let consulting = recover_consulting_the_journal(&node, named_slots);
    let devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let ignoring = recover_ignoring_the_journal(devices.as_slice(), target_naming(named_slots));
    let read_when_ignoring =
        read_signature_of(&ignoring.outcome).expect("所选根自己的单元还在，应当读得成");
    assert_eq!(
        judge_read_that_ignores_the_journal(
            &node,
            chosen_root_of(&consulting.outcome),
            chosen_root_of(&ignoring.outcome),
            &read_when_ignoring,
        ),
        IgnorePolicyFindings {
            chosen_root_changed: false,
            content_differs_from_chosen_root: false,
            content_is_not_the_tip: true,
        }
    );
}
