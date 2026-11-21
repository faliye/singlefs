use super::*;

/// 最新根的实例表里证明「R 的实例接在实例 1 的 `previous_txg` 那一版上」的那几行：(1, `previous_txg`)，
/// 加上实例 1 与 R 的实例之间每个实例 j 一行 (j, 0, 0)（`instances_that_published_nothing`）。
fn rows_bridging_instance_one_at(
    previous_txg: u64,
    instances_that_published_nothing: &[u32],
) -> Vec<InstanceTableRow> {
    std::iter::once(InstanceTableRow {
        instance: 1,
        published_checkpoint_txg: previous_txg,
        applied_transaction_high_water_mark: 0,
    })
    .chain(
        instances_that_published_nothing
            .iter()
            .map(|instance| InstanceTableRow {
                instance: *instance,
                published_checkpoint_txg: 0,
                applied_transaction_high_water_mark: 0,
            }),
    )
    .collect()
}

/// C699（取号跳过改认出生代号）的第四条件，取 E198（取号跳过认出生代号的前提） 第一段有效性闸 C1 那一张的数（nrb、last 10、k 3、bad 1）：
/// 洞对 (1@10, 2@12)、最新根的行 (1, 10)、R 的实例表指针出生 txg 11（写行那一版建的）。三条行条件都成立、第四条件不成立：接不上。
/// 同一份行，R 的指针出生 txg 改成 12（R 就是新建实例表的那一版）接得上——两个分类只差第四条件。
/// 第四条件比成 R.txg − 1（变异「C699 甲：取号跳过第四条件比成 R 的前一个 txg」）这一条红。
#[test]
fn hole_whose_rows_bridge_is_not_a_number_acquisition_skip_unless_the_later_version_created_its_instance_table(
) {
    let rows = rows_bridging_instance_one_at(10, &[]);
    assert_eq!(
        hole_in_the_candidate_set_between(10, &BTreeSet::from([1]), 12, 2, Some(11), &rows),
        HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows,
        "R 的实例表指针生在写行那一版 11：R 所在实例自己更早的版本存在过，不算跳过"
    );
    assert_eq!(
        hole_in_the_candidate_set_between(10, &BTreeSet::from([1]), 12, 2, Some(12), &rows),
        HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows,
        "R 的实例表指针生在 R 自己：四个条件都成立"
    );
}

/// 合法的取号跳过，取 E198 第二段锚 C2 那一张的数（skr、last 6、k 0）：洞对 (1@6, 3@8)、最新根的行 (1, 6) 与 (2, 0, 0)、
/// R 的实例表指针出生 txg 8（R 是实例 3 写行那一版）。四个条件都成立：接得上。第四条件比成 R⁻ 的 txg
/// （变异「C699 甲：取号跳过第四条件比成 R⁻ 的 txg」）这一条红。
#[test]
fn number_acquisition_skip_whose_later_version_created_its_instance_table_is_bridged() {
    let rows = rows_bridging_instance_one_at(6, &[2]);
    assert_eq!(
        hole_in_the_candidate_set_between(6, &BTreeSet::from([1]), 8, 3, Some(8), &rows),
        HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows,
        "实例 2 的写行崩在根槽之前、实例 3 写行那一版新建实例表：取号跳过"
    );
}

/// R 的实例表指针整条全零（`instance_table_pointer_birth_txg_of` 交 `None`）：行接得上也按「R 不是新建实例表的那一版」认，接不上
/// （E198 装置的甲同一个读法）。第四条件写成「指针出生 txg 不大于 R.txg」（变异「C699 甲：取号跳过第四条件写成不大于」，全零读成最小）这一条红。
#[test]
fn hole_whose_later_version_has_an_all_zero_instance_table_pointer_is_not_a_number_acquisition_skip(
) {
    let rows = rows_bridging_instance_one_at(6, &[2]);
    assert_eq!(
        hole_in_the_candidate_set_between(6, &BTreeSet::from([1]), 8, 3, None, &rows),
        HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows,
        "R 的实例表指针全零：不认作取号跳过"
    );
}

/// I-3.5 (b) 谱系那一半的几条用例共用的那一对：回退之前那一版 R⁻（实例 2、txg 8）引用自己 extent 树里的落点 (0, 100)（诞生代号 8），
/// R（实例 2、txg 9）的 extent 树根指针出生 txg 3、引用 (0, 200) 与 (0, 201)（诞生代号都是 3）：R⁻ 不引用它们、(a) 不成立，
/// 那棵树根指针出生 txg 3 小于 8、子树里诞生代号都不大于 3，(b) 前两半成立，剩下谱系那一半。R 的谱系树取 `lineage_records_of_the_later_version`。
const EXTENT_TREE_OF_THE_APPEARANCE_RULE_PAIR: UserVisibleTreeOfTheWalk =
    UserVisibleTreeOfTheWalk {
        kind: TREE_KIND_EXTENT,
        tree: 7,
    };
const TXG_OF_THE_VERSION_BEFORE_THE_ROLLBACK: u64 = 8;
const TXG_OF_THE_ROLLBACK_VERSION: u64 = 9;
const ROOT_POINTER_BIRTH_TXG_OF_THE_INHERITED_EXTENT_TREE: u64 = 3;

fn version_of_the_appearance_rule_pair(
    checkpoint_txg: u64,
    root_pointer_birth_txg: u64,
    placements: &[(u32, u64)],
    lineage_records: Result<Vec<LineageRecordOfTheScan>, DeletionScanGap>,
) -> VersionReferencesForTheAppearanceRule {
    VersionReferencesForTheAppearanceRule {
        instance: 2,
        checkpoint_txg,
        placements: placements
            .iter()
            .map(|placement| {
                (
                    *placement,
                    PlacementOfTheAppearanceRule {
                        identity_compared_across_versions: *placement,
                        unit_whose_birth_txg_it_takes: *placement,
                        user_visible_tree: Some(EXTENT_TREE_OF_THE_APPEARANCE_RULE_PAIR),
                    },
                )
            })
            .collect(),
        user_visible_tree_roots: vec![UserVisibleTreeRootOfAVersion {
            tree: EXTENT_TREE_OF_THE_APPEARANCE_RULE_PAIR,
            root_pointer_birth_txg,
        }],
        references_are_complete: true,
        kind_of_a_tree_table_entry_without_a_clause: None,
        instance_table_pointer_birth_txg: None,
        lineage_records,
    }
}

/// 那一对的 I-3.5 判定（R 的谱系树取 `lineage_records_of_the_later_version`）。
fn reference_interval_verdict_of_the_appearance_rule_pair(
    lineage_records_of_the_later_version: Result<Vec<LineageRecordOfTheScan>, DeletionScanGap>,
) -> InvariantVerdict {
    reference_interval_verdict_of_the_appearance_rule_pair_given_the_tree_table_entries_without_a_clause(
            lineage_records_of_the_later_version,
            None,
            None,
        )
}

/// 同 [`reference_interval_verdict_of_the_appearance_rule_pair`]，两版各带树表里「走到没有条款的形状」的条目的种类。
fn reference_interval_verdict_of_the_appearance_rule_pair_given_the_tree_table_entries_without_a_clause(
    lineage_records_of_the_later_version: Result<Vec<LineageRecordOfTheScan>, DeletionScanGap>,
    kind_without_a_clause_in_the_previous_version: Option<u16>,
    kind_without_a_clause_in_the_later_version: Option<u16>,
) -> InvariantVerdict {
    let mut versions = [
        version_of_the_appearance_rule_pair(
            TXG_OF_THE_VERSION_BEFORE_THE_ROLLBACK,
            TXG_OF_THE_VERSION_BEFORE_THE_ROLLBACK,
            &[(0, 100)],
            Ok(Vec::new()),
        ),
        version_of_the_appearance_rule_pair(
            TXG_OF_THE_ROLLBACK_VERSION,
            ROOT_POINTER_BIRTH_TXG_OF_THE_INHERITED_EXTENT_TREE,
            &[(0, 200), (0, 201)],
            lineage_records_of_the_later_version,
        ),
    ];
    versions[0].kind_of_a_tree_table_entry_without_a_clause =
        kind_without_a_clause_in_the_previous_version;
    versions[1].kind_of_a_tree_table_entry_without_a_clause =
        kind_without_a_clause_in_the_later_version;
    let birth_txg_of_walked_units = BTreeMap::from([
        ((0u32, 100u64), TXG_OF_THE_VERSION_BEFORE_THE_ROLLBACK),
        (
            (0, 200),
            ROOT_POINTER_BIRTH_TXG_OF_THE_INHERITED_EXTENT_TREE,
        ),
        (
            (0, 201),
            ROOT_POINTER_BIRTH_TXG_OF_THE_INHERITED_EXTENT_TREE,
        ),
    ]);
    let mut judgements = Judgements::default();
    judge_the_appearance_rule_of_adjacent_candidate_versions(
        &versions,
        &birth_txg_of_walked_units,
        &BTreeMap::new(),
        &[],
        &mut judgements,
    );
    judgements
        .into_report()
        .into_iter()
        .find(|(invariant, _)| *invariant == REFERENCE_INTERVALS_ARE_EXACT)
        .expect("I-3.5 在清单里")
        .1
}

fn assert_the_pair_reports_the_tree_table_entry_without_a_clause(verdict: InvariantVerdict) {
    match verdict {
            InvariantVerdict::PartiallyJudged(unjudged) => assert!(
                unjudged.contains(&format!(
                    "有一版树表里有种类 {TREE_KIND_NOT_REGISTERED} 的条目，走读走到没有条款的形状、引用没收全，这一对不判"
                )),
                "说明点名那一条条目的种类：{unjudged}"
            ),
            other @ (InvariantVerdict::Holds
            | InvariantVerdict::Violated(_)
            | InvariantVerdict::NotApplicable(_)) => {
                panic!("R 或 R⁻ 的树表里有没有条款的条目：这一对报部分没判，实际 {other:?}")
            }
        }
}

/// 后一版树表里有种类没登记的条目：这一对不判。谱系树里一条担保都没有，判了就红，所以这一条分得出判没判。
#[test]
fn later_version_with_a_tree_table_entry_of_an_unregistered_kind_leaves_the_pair_not_judged() {
    assert_the_pair_reports_the_tree_table_entry_without_a_clause(
            reference_interval_verdict_of_the_appearance_rule_pair_given_the_tree_table_entries_without_a_clause(
                Ok(Vec::new()),
                None,
                Some(TREE_KIND_NOT_REGISTERED),
            ),
        );
}

/// 前一版树表里有种类没登记的条目：同样不判（同一理由）。
#[test]
fn previous_version_with_a_tree_table_entry_of_an_unregistered_kind_leaves_the_pair_not_judged() {
    assert_the_pair_reports_the_tree_table_entry_without_a_clause(
            reference_interval_verdict_of_the_appearance_rule_pair_given_the_tree_table_entries_without_a_clause(
                Ok(Vec::new()),
                Some(TREE_KIND_NOT_REGISTERED),
                None,
            ),
        );
}

fn lineage_record(rollback_txg: u64, parent_txg: u64) -> LineageRecordOfTheScan {
    LineageRecordOfTheScan {
        rollback_txg,
        parent_txg,
        inode_number_watermark_of_the_version_rolled_back_from: 1,
    }
}

fn assert_the_pair_reddens_the_reference_interval_invariant(
    lineage_records_of_the_later_version: Vec<LineageRecordOfTheScan>,
    what: &str,
) {
    match reference_interval_verdict_of_the_appearance_rule_pair(Ok(
        lineage_records_of_the_later_version,
    )) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("槽 200") && detail.contains("整棵继承：true"),
            "{what}：违例说的是整棵继承下来、谱系不担保的那棵树里的落点：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("{what}：I-3.5 应当判违例，实际 {other:?}")
        }
    }
}

/// 合法回退的边界：R 的谱系树里有 (T_rb 9 = R.txg, T_old 3 = 根指针出生 txg)，两处取等都算担保，I-3.5 成立。
/// 「T_rb ≤ R.txg」写成「<」、「根指针出生 txg ≤ T_old」写成「<」（两条变异）这一条红。
#[test]
fn rollback_record_at_the_later_txg_back_to_the_birth_of_the_root_vouches_for_the_inherited_tree() {
    assert_eq!(
        reference_interval_verdict_of_the_appearance_rule_pair(Ok(vec![lineage_record(
            TXG_OF_THE_ROLLBACK_VERSION,
            ROOT_POINTER_BIRTH_TXG_OF_THE_INHERITED_EXTENT_TREE,
        )])),
        InvariantVerdict::Holds,
        "T_rb = R.txg、T_old = 根指针出生 txg：谱系担保这棵树是回退带回来的"
    );
}

/// R 的谱系树一条记录都没有（零根）：整棵继承下来的树没有谱系担保，I-3.5 红。谱系那一半恒成立（变异「I-3.5 谱系那一半恒成立」）这一条红。
#[test]
fn an_inherited_tree_without_any_lineage_record_reddens_the_reference_interval_invariant() {
    assert_the_pair_reddens_the_reference_interval_invariant(Vec::new(), "R 的谱系树是零根");
}

/// 记录的 T_rb 等于 R⁻.txg（8）：那次回退不在 (R⁻.txg, R.txg] 里，担保不了这一对，I-3.5 红。
/// 「R⁻.txg < T_rb」写成「≤」（变异）这一条红。
#[test]
fn rollback_record_at_the_txg_of_the_previous_version_does_not_vouch_for_the_inherited_tree() {
    assert_the_pair_reddens_the_reference_interval_invariant(
        vec![lineage_record(
            TXG_OF_THE_VERSION_BEFORE_THE_ROLLBACK,
            ROOT_POINTER_BIRTH_TXG_OF_THE_INHERITED_EXTENT_TREE,
        )],
        "T_rb = R⁻.txg",
    );
}

/// 记录的 T_rb 是 10，晚于 R.txg（9）：I-3.5 红。去掉「T_rb ≤ R.txg」（变异）这一条红。
#[test]
fn rollback_record_later_than_the_later_version_does_not_vouch_for_the_inherited_tree() {
    assert_the_pair_reddens_the_reference_interval_invariant(
        vec![lineage_record(
            TXG_OF_THE_ROLLBACK_VERSION + 1,
            ROOT_POINTER_BIRTH_TXG_OF_THE_INHERITED_EXTENT_TREE,
        )],
        "T_rb = R.txg + 1",
    );
}

/// 记录的 T_old 是 2，比那棵树的根指针出生 txg（3）早：回退目标那一版还没有这棵树，担保不了，I-3.5 红。
/// 去掉「根指针出生 txg ≤ T_old」（变异）这一条红。
#[test]
fn rollback_record_to_a_target_older_than_the_root_does_not_vouch_for_the_inherited_tree() {
    assert_the_pair_reddens_the_reference_interval_invariant(
        vec![lineage_record(
            TXG_OF_THE_ROLLBACK_VERSION,
            ROOT_POINTER_BIRTH_TXG_OF_THE_INHERITED_EXTENT_TREE - 1,
        )],
        "T_old = 根指针出生 txg − 1",
    );
}

/// R 的谱系树读不全：(a) 不成立、(b) 前两半成立的那两个落点判不了，I-3.5 报部分没判、说明里写谱系树读不全，不报成立也不报违例。
/// 读不全照「没有记录」判（变异「I-3.5 谱系树读不全照没有记录判」）这一条红。
#[test]
fn an_unreadable_lineage_tree_leaves_the_units_of_the_inherited_tree_unjudged() {
    match reference_interval_verdict_of_the_appearance_rule_pair(Err(
        DeletionScanGap::LineageTreeUnreadable,
    )) {
        InvariantVerdict::PartiallyJudged(detail) => assert!(
            detail.contains("谱系树读不全（LineageTreeUnreadable）"),
            "部分没判的说明写谱系树读不全：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => {
            panic!("R 的谱系树读不全：I-3.5 应当报部分没判，实际 {other:?}")
        }
    }
}
