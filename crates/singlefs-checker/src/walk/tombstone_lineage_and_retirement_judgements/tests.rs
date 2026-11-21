use super::*;
use singlefs_format::{TREE_IDENTIFIER_LINEAGE, TREE_IDENTIFIER_TOMBSTONE};

// ───── 容器容量的两个判法直接交 (容器号, 记录数) 表 ─────

/// 容量判法判的三棵容器树。
#[derive(Clone, Copy, Debug)]
enum ContainerTreeOfTheCapacityJudges {
    Tombstone,
    Lineage,
    Retirement,
}

impl ContainerTreeOfTheCapacityJudges {
    /// 一片容器的产品容量（墓碑 815、谱系 1359、退役 959）。
    fn product_records_per_container(self) -> u64 {
        match self {
            ContainerTreeOfTheCapacityJudges::Tombstone => TOMBSTONE_CONTAINER_RECORDS,
            ContainerTreeOfTheCapacityJudges::Lineage => {
                LineageOrRetirementTree::Lineage.product_records_per_container()
            }
            ContainerTreeOfTheCapacityJudges::Retirement => {
                LineageOrRetirementTree::Retirement.product_records_per_container()
            }
        }
    }

    /// 装没装旋钮时判法判的那一条（墓碑 I-9.21 / I-9.22，谱系与退役 I-9.36 / I-9.37）。
    fn capacity_invariant(self, knob_records_per_container: Option<u64>) -> &'static str {
        match self {
            ContainerTreeOfTheCapacityJudges::Tombstone => capacity_invariant_of(
                knob_records_per_container.map(tombstone_container_capacity_knob_of),
            ),
            ContainerTreeOfTheCapacityJudges::Lineage
            | ContainerTreeOfTheCapacityJudges::Retirement => {
                lineage_or_retirement_capacity_invariant_of(knob_records_per_container)
            }
        }
    }
}

fn tombstone_container_capacity_knob_of(
    records_per_container: u64,
) -> TombstoneContainerCapacityKnob {
    TombstoneContainerCapacityKnob::new(records_per_container).expect("旋钮值在 1..=815 里")
}

/// 一条整对象墓碑（容量判法只数条数，不读字段）。
fn whole_object_tombstone_for_the_capacity_judge() -> TombstoneRecordOfTheScan {
    TombstoneRecordOfTheScan {
        object: 2,
        object_birth: 3,
        range_start: TOMBSTONE_WHOLE_OBJECT_RANGE_START,
        range_end: TOMBSTONE_WHOLE_OBJECT_RANGE_END,
        death_txg: 4,
    }
}

/// 把沿叶序的 (容器号, 记录数) 表直接交这一棵的容量判法（墓碑树交 [`judge_tombstone_container_capacity_of_a_version`]，
/// 谱系树与退役树交 [`judge_lineage_or_retirement_container_capacity_of_a_version`]），旋钮值 `knob_records_per_container`（`None` 是没装）。
fn capacity_judgements_of_the_table(
    tree: ContainerTreeOfTheCapacityJudges,
    container_numbers_and_record_counts: &[(u64, u64)],
    knob_records_per_container: Option<u64>,
) -> Judgements {
    let mut judgements = Judgements::default();
    let record_count_as_usize =
        |record_count: u64| usize::try_from(record_count).expect("一片容器的记录数装得进 usize");
    let judge_the_lineage_or_retirement_table =
        |tree_kind: LineageOrRetirementTree,
         tree_identifier: u64,
         judgements_of_the_table: &mut Judgements| {
            let table: Vec<(u64, usize)> = container_numbers_and_record_counts
                .iter()
                .map(|(container, record_count)| (*container, record_count_as_usize(*record_count)))
                .collect();
            judge_lineage_or_retirement_container_capacity_of_a_version(
                "单测那一版",
                tree_kind,
                tree_identifier,
                &table,
                knob_records_per_container,
                judgements_of_the_table,
            );
        };
    match tree {
        ContainerTreeOfTheCapacityJudges::Tombstone => {
            let containers: Vec<TombstoneContainerOfTheScan> = container_numbers_and_record_counts
                .iter()
                .map(|(container, record_count)| TombstoneContainerOfTheScan {
                    container: *container,
                    records: vec![
                        whole_object_tombstone_for_the_capacity_judge();
                        record_count_as_usize(*record_count)
                    ],
                })
                .collect();
            judge_tombstone_container_capacity_of_a_version(
                "单测那一版",
                TREE_IDENTIFIER_TOMBSTONE,
                &containers,
                knob_records_per_container.map(tombstone_container_capacity_knob_of),
                &mut judgements,
            );
        }
        ContainerTreeOfTheCapacityJudges::Lineage => judge_the_lineage_or_retirement_table(
            LineageOrRetirementTree::Lineage,
            TREE_IDENTIFIER_LINEAGE,
            &mut judgements,
        ),
        ContainerTreeOfTheCapacityJudges::Retirement => judge_the_lineage_or_retirement_table(
            LineageOrRetirementTree::Retirement,
            singlefs_format::TREE_IDENTIFIER_RETIREMENT,
            &mut judgements,
        ),
    }
    judgements
}

/// 这张表交这一棵的容量判法：那一条只红一处（只一句不成立），违例说明里有 `detail_fragment`。
fn assert_the_capacity_judge_violates_only_one_sentence(
    tree: ContainerTreeOfTheCapacityJudges,
    container_numbers_and_record_counts: &[(u64, u64)],
    knob_records_per_container: Option<u64>,
    detail_fragment: &str,
) {
    let invariant = tree.capacity_invariant(knob_records_per_container);
    let judgements = capacity_judgements_of_the_table(
        tree,
        container_numbers_and_record_counts,
        knob_records_per_container,
    );
    assert_eq!(
            judgements.violation_count(invariant),
            1,
            "{tree:?} 表 {container_numbers_and_record_counts:?}（旋钮 {knob_records_per_container:?}）：{invariant} 只一句不成立（第一处：{:?}）",
            judgements.first_violation_of(invariant)
        );
    assert!(
            judgements
                .first_violation_of(invariant)
                .is_some_and(|detail| detail.contains(detail_fragment)),
            "{tree:?} 表 {container_numbers_and_record_counts:?}：{invariant} 的违例说明写出「{detail_fragment}」，实际 {:?}",
            judgements.first_violation_of(invariant)
        );
}

/// 这张表交这一棵的容量判法：那一条判过、一处都不红。
fn assert_the_capacity_judge_holds(
    tree: ContainerTreeOfTheCapacityJudges,
    container_numbers_and_record_counts: &[(u64, u64)],
    knob_records_per_container: Option<u64>,
) {
    let invariant = tree.capacity_invariant(knob_records_per_container);
    let judgements = capacity_judgements_of_the_table(
        tree,
        container_numbers_and_record_counts,
        knob_records_per_container,
    );
    assert_eq!(
            judgements.violation_count(invariant),
            0,
            "{tree:?} 表 {container_numbers_and_record_counts:?}（旋钮 {knob_records_per_container:?}）：{invariant} 一处都不红（第一处：{:?}）",
            judgements.first_violation_of(invariant)
        );
    assert!(
        matches!(
            judgements
                .into_report()
                .into_iter()
                .find(|(name, _)| *name == invariant)
                .map(|(_, verdict)| verdict),
            Some(InvariantVerdict::Holds)
        ),
        "{tree:?} 表 {container_numbers_and_record_counts:?}：{invariant} 判过而成立"
    );
}

/// H2 产品支（没装旋钮）的对照：第一片装满产品容量、号 1、2，最右那片 1 条：判过、一处不红（下面三形各只改一处）。
fn assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
    tree: ContainerTreeOfTheCapacityJudges,
) {
    let full = tree.product_records_per_container();
    assert_the_capacity_judge_holds(tree, &[(1, full), (2, 1)], None);
}

/// H2 产品支第一形：第一片装满、号 2、3——容量那句成立，只「第一片容器号是 1」那句不成立。
fn assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
    tree: ContainerTreeOfTheCapacityJudges,
) {
    let full = tree.product_records_per_container();
    assert_the_capacity_judge_violates_only_one_sentence(
        tree,
        &[(2, full), (3, 1)],
        None,
        "沿叶序第一片容器的容器号是 2，不是 1",
    );
}

/// H2 产品支第二形：第一片装满、号 1、3——只「相邻两片加一」那句不成立。
fn assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
    tree: ContainerTreeOfTheCapacityJudges,
) {
    let full = tree.product_records_per_container();
    assert_the_capacity_judge_violates_only_one_sentence(
        tree,
        &[(1, full), (3, 1)],
        None,
        "沿叶序容器号不是逐片加一：容器 1 之后是容器 3",
    );
}

/// H2 产品支第三形：第一片装满、号 1、2，最右那片 0 条——只「每片至少一条」那句不成立。
fn assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
    tree: ContainerTreeOfTheCapacityJudges,
) {
    let full = tree.product_records_per_container();
    assert_the_capacity_judge_violates_only_one_sentence(
        tree,
        &[(1, full), (2, 0)],
        None,
        "容器 2 一条记录都没有",
    );
}

/// H3 产品支：第一片装满、号 0、1——相邻加一成立，只「第一片容器号是 1」那句不成立（「第一片 ≤ 1」放过它）。
fn assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
    tree: ContainerTreeOfTheCapacityJudges,
) {
    let full = tree.product_records_per_container();
    assert_the_capacity_judge_violates_only_one_sentence(
        tree,
        &[(0, full), (1, 1)],
        None,
        "沿叶序第一片容器的容器号是 0，不是 1",
    );
}

/// H1 旋钮支：旋钮 1 写出的两片各 1 条交旋钮 2 判——容器号与「≤ 旋钮值」都成立，
/// 只「不是最右的那片装满旋钮值」那句不成立；同一张表交旋钮 1 判过、一处不红。
fn assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
    tree: ContainerTreeOfTheCapacityJudges,
) {
    assert_the_capacity_judge_holds(tree, &[(1, 1), (2, 1)], Some(1));
    assert_the_capacity_judge_violates_only_one_sentence(
        tree,
        &[(1, 1), (2, 1)],
        Some(2),
        "不是最右的容器 1 装了 1 条，不是旋钮值 2",
    );
}

#[test]
fn full_first_tombstone_container_and_one_more_hold_on_the_product_branch() {
    assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
        ContainerTreeOfTheCapacityJudges::Tombstone,
    );
}

#[test]
fn full_first_lineage_container_and_one_more_hold_on_the_product_branch() {
    assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
        ContainerTreeOfTheCapacityJudges::Lineage,
    );
}

#[test]
fn full_first_retirement_container_and_one_more_hold_on_the_product_branch() {
    assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
        ContainerTreeOfTheCapacityJudges::Retirement,
    );
}

#[test]
fn product_tombstone_containers_numbered_two_and_three_violate_only_the_first_number_sentence() {
    assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
        ContainerTreeOfTheCapacityJudges::Tombstone,
    );
}

#[test]
fn product_lineage_containers_numbered_two_and_three_violate_only_the_first_number_sentence() {
    assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
        ContainerTreeOfTheCapacityJudges::Lineage,
    );
}

#[test]
fn product_retirement_containers_numbered_two_and_three_violate_only_the_first_number_sentence() {
    assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
        ContainerTreeOfTheCapacityJudges::Retirement,
    );
}

#[test]
fn product_tombstone_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence() {
    assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
        ContainerTreeOfTheCapacityJudges::Tombstone,
    );
}

#[test]
fn product_lineage_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence() {
    assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
        ContainerTreeOfTheCapacityJudges::Lineage,
    );
}

#[test]
fn product_retirement_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence()
{
    assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
        ContainerTreeOfTheCapacityJudges::Retirement,
    );
}

#[test]
fn product_tombstone_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence() {
    assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
        ContainerTreeOfTheCapacityJudges::Tombstone,
    );
}

#[test]
fn product_lineage_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence() {
    assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
        ContainerTreeOfTheCapacityJudges::Lineage,
    );
}

#[test]
fn product_retirement_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence() {
    assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
        ContainerTreeOfTheCapacityJudges::Retirement,
    );
}

#[test]
fn product_tombstone_containers_numbered_zero_and_one_violate_only_the_first_number_sentence() {
    assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
        ContainerTreeOfTheCapacityJudges::Tombstone,
    );
}

#[test]
fn product_lineage_containers_numbered_zero_and_one_violate_only_the_first_number_sentence() {
    assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
        ContainerTreeOfTheCapacityJudges::Lineage,
    );
}

#[test]
fn product_retirement_containers_numbered_zero_and_one_violate_only_the_first_number_sentence() {
    assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
        ContainerTreeOfTheCapacityJudges::Retirement,
    );
}

#[test]
fn two_tombstone_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence()
{
    assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
        ContainerTreeOfTheCapacityJudges::Tombstone,
    );
}

#[test]
fn two_lineage_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence() {
    assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
        ContainerTreeOfTheCapacityJudges::Lineage,
    );
}

#[test]
fn two_retirement_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence()
{
    assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
        ContainerTreeOfTheCapacityJudges::Retirement,
    );
}
