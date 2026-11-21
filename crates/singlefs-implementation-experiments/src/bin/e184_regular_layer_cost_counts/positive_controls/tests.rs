use super::*;
use crate::helpers_shared_by_the_tests::{block_of, planned_blocks_of, tiny_dump};

#[test]
fn blocks_with_the_same_reuse_key_and_another_plan_hash_are_not_shared_but_count_for_the_reuse_key_share(
) {
    let plan_with_the_original_hash = vec![block_of(1, 1, 0, 10, 0), block_of(1, 1, 65_536, 10, 0)];
    let other_plan = vec![block_of(1, 2, 0, 10, 0)];
    let against = block_keys_of(&plan_with_the_original_hash);
    assert_eq!(
        shared_blocks_and_states(&other_plan, &against),
        (0, 0),
        "计划哈希不同的块键不同，哪怕复用键相同"
    );
    let against_reuse = reuse_keys_of(&plan_with_the_original_hash);
    assert!(
        (reuse_key_share_fraction(&other_plan, &against_reuse) - 1.0).abs() < f64::EPSILON,
        "复用键是块键共用的上界"
    );
    let same_start_other_reuse = vec![block_of(2, 1, 0, 10, 0)];
    assert_eq!(
        shared_blocks_and_states(&same_start_other_reuse, &against),
        (0, 0)
    );
}

#[test]
fn the_two_expansions_share_exactly_the_blocks_of_the_points_without_segments_and_every_reuse_key()
{
    let full = planned_blocks_of(
        vec![
            block_of(1, 10, 0, 7, 0),
            block_of(2, 20, 0, 7, 1),
            block_of(3, 30, 0, 1, 2),
        ],
        1,
    );
    let in_place_subsets = planned_blocks_of(
        vec![
            block_of(1, 11, 0, 3, 0),
            block_of(2, 21, 0, 3, 1),
            block_of(3, 30, 0, 1, 2),
        ],
        1,
    );
    let control = two_expansions_control("flow", &full, &in_place_subsets);
    assert!(control.passed, "{control:?}");
    let end_node_differs = planned_blocks_of(
        vec![
            block_of(1, 11, 0, 3, 0),
            block_of(2, 21, 0, 3, 1),
            block_of(3, 31, 0, 1, 2),
        ],
        1,
    );
    assert!(
        !two_expansions_control("flow", &full, &end_node_differs).passed,
        "末节点的块键也不同：共用的块数 0 ≠ 1"
    );
    let plan_hash_ignored = planned_blocks_of(
        vec![
            block_of(1, 10, 0, 7, 0),
            block_of(2, 20, 0, 7, 1),
            block_of(3, 30, 0, 1, 2),
        ],
        1,
    );
    assert!(
        !two_expansions_control("flow", &full, &plan_hash_ignored).passed,
        "两种展开的块键全部相同：计划哈希没有区分开它们"
    );
    let other_reuse_key = planned_blocks_of(
        vec![
            block_of(1, 11, 0, 3, 0),
            block_of(9, 21, 0, 3, 1),
            block_of(3, 30, 0, 1, 2),
        ],
        1,
    );
    assert!(
        !two_expansions_control("flow", &full, &other_reuse_key).passed,
        "复用键两边不是同一批"
    );
}

#[test]
fn the_dump_round_trip_control_passes_on_a_dump_and_the_name_of_a_flow_is_not_in_its_keys() {
    assert!(dump_round_trip_control(&tiny_dump("flow", 2)).passed);
    assert!(
        controls_apply_to(&tiny_dump("flow", 2)),
        "种子下标 0 的随机历史跑对照"
    );
    let mut other_seed = tiny_dump("flow", 2);
    other_seed.seed = Some(SEED_BASE_OF_THE_RANDOM_HISTORIES + 1);
    assert!(!controls_apply_to(&other_seed));
    other_seed.seed = None;
    assert!(
        controls_apply_to(&other_seed),
        "手写流（没有种子）每条都跑对照"
    );
}
