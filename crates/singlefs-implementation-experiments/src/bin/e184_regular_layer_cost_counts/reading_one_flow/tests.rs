use super::*;
use crate::helpers_shared_by_the_tests::{block_of, planned_blocks_of};

#[test]
fn the_count_disagreements_name_each_pair_of_numbers_that_do_not_agree() {
    let consistent = planned_blocks_of(vec![block_of(1, 1, 0, 7, 0), block_of(2, 2, 0, 3, 1)], 0);
    assert_eq!(
        count_disagreements_of("flow", Expansion::Full, Some(10), &consistent),
        Vec::<String>::new()
    );
    let counted_otherwise = PlannedBlocks {
        state_count_by_the_counting_function: 11,
        ..planned_blocks_of(vec![block_of(1, 1, 0, 7, 0), block_of(2, 2, 0, 3, 1)], 0)
    };
    assert_eq!(
        count_disagreements_of("flow", Expansion::Full, Some(10), &counted_otherwise).len(),
        1
    );
    let blocks_do_not_add_up = PlannedBlocks {
        state_count: 12,
        state_count_by_the_counting_function: 12,
        ..planned_blocks_of(vec![block_of(1, 1, 0, 7, 0)], 0)
    };
    assert_eq!(
        count_disagreements_of(
            "flow",
            Expansion::InPlaceSubsets,
            Some(12),
            &blocks_do_not_add_up
        )
        .len(),
        1,
        "各块态数之和 7 ≠ 12"
    );
    assert_eq!(
        count_disagreements_of("flow", Expansion::Full, Some(9), &consistent).len(),
        1,
        "本地按段数的态数 9 ≠ 计划的 10"
    );
}
