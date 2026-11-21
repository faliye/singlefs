use super::*;
use crate::helpers_shared_by_the_tests::{tiny_dump, tiny_write};

#[test]
fn the_two_state_closed_form_gives_the_pinned_numbers_of_the_snapshot_creation_and_new_pool_shapes()
{
    assert_eq!(
        two_state_closed_form(&segments_of_sizes(&SNAPSHOT_CREATION_SEGMENT_SIZES)),
        Some(16_842_785)
    );
    assert_eq!(
        two_state_closed_form(&segments_of_sizes(&NEW_POOL_SEGMENT_SIZES)),
        Some(16_777_243)
    );
    assert_eq!(NEW_POOL_SEGMENT_SIZES.iter().sum::<usize>(), 43);
    assert_eq!(
        two_state_closed_form(&segments_of_sizes(&[1, 2, 3])),
        Some(1 + 1 + 3 + 7),
        "手算：1 + (2 − 1) + (4 − 1) + (8 − 1)"
    );
    assert_eq!(
        two_state_closed_form(&segments_of_sizes(&[1, 2, 3, 24])),
        Some(closed_form_state_count_of_write_index_segments(
            &segments_of_sizes(&[1, 2, 3, 24])
        )),
        "本地复算与 crates 的闭式函数在小段形上相等"
    );
}

#[test]
fn sixty_four_writes_in_one_segment_have_no_closed_form_and_a_sum_past_u64_has_none_either() {
    assert_eq!(two_state_closed_form(&segments_of_sizes(&[64])), None);
    assert_eq!(
        two_state_closed_form(&segments_of_sizes(&[63])),
        Some(1u64 << 63)
    );
    assert_eq!(
        two_state_closed_form(&segments_of_sizes(&[63, 63])),
        Some(u64::MAX)
    );
    assert_eq!(
        two_state_closed_form(&segments_of_sizes(&[63, 63, 63])),
        None
    );
    assert_eq!(LONGEST_SEGMENT_WITH_A_CLOSED_FORM, 63);
}

#[test]
fn the_block_count_cuts_at_the_local_block_length() {
    let cases: [(u64, u64); 7] = [
        (0, 0),
        (1, 1),
        (65_535, 1),
        (65_536, 1),
        (65_537, 2),
        (131_072, 2),
        (131_073, 3),
    ];
    for (states, blocks) in cases {
        assert_eq!(blocks_needed_for(states), blocks, "{states} 个状态");
    }
    assert_eq!(
        LOCAL_STATES_PER_BLOCK, STATES_PER_BLOCK,
        "本地块长与 crates 的块长相等（停机 T1）"
    );
}

#[test]
fn the_longest_segment_is_the_largest_one_and_not_the_first() {
    assert_eq!(
        longest_segment_of(&segments_of_sizes(&SNAPSHOT_CREATION_SEGMENT_SIZES)),
        24
    );
    assert_eq!(longest_segment_of(&segments_of_sizes(&[3, 1, 2])), 3);
    assert_eq!(longest_segment_of(&segments_of_sizes(&[1, 5, 2])), 5);
    assert_eq!(longest_segment_of(&[]), 0);
}

#[test]
fn the_block_count_bound_adds_the_points_left_over_to_the_blocks_of_the_states() {
    assert_eq!(block_count_bound_of(0, 0), 2, "没有态也留末节点与一点余量");
    assert_eq!(block_count_bound_of(65_536, 3), 1 + 3 + 2);
    assert_eq!(block_count_bound_of(65_537, 3), 2 + 3 + 2);
    assert_eq!(
        block_count_bound_of((1u64 << 56) - 1, 100),
        (1u64 << 40) + 102,
        "一个 56 写的段：2^56 − 1 个状态正好 2^40 块"
    );
    assert_eq!(
        block_count_bound_of(u64::MAX, usize::MAX),
        u64::MAX,
        "越界饱和，不回绕成小数"
    );
    assert_eq!(MOST_BLOCKS_PLANNED_PER_FLOW, 1 << 26);
    assert_eq!(MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP, 1 << 24);
}

/// 一条段写数 `long_segment_writes` 的单元写段，后面接一条根槽写的段：两态闭式 2^n + 1，只拿来数态数、不排计划。
fn dump_with_one_long_segment(label: &str, long_segment_writes: u64) -> FlowDump {
    let root_index = usize::try_from(long_segment_writes).expect("装得进 usize");
    let mut dump = tiny_dump(label, 2);
    dump.writes = (0..long_segment_writes)
        .map(|index| tiny_write(0, StepKind::UnitWrite, 4096 + 512 * index, 1))
        .chain(std::iter::once(tiny_write(
            0,
            StepKind::RootRecordFua,
            1 << 19,
            3,
        )))
        .collect();
    dump.segments = vec![(0..root_index).collect(), vec![root_index]];
    dump.judged_root_index = Some(root_index);
    dump
}

#[test]
fn flow_whose_blocks_would_not_fit_in_memory_is_counted_but_not_planned() {
    let dump = dump_with_one_long_segment("fifty-six", 56);
    let base = dump.base_pool();
    let counted = state_count_of_the_flow(&base, &dump, Expansion::Full);
    assert_eq!(
        counted,
        Some((1u64 << 56) + 1),
        "1 + (2^56 − 1) + (2^1 − 1)"
    );
    assert_eq!(
        state_count_to_plan(&base, &dump, Expansion::Full, MOST_BLOCKS_PLANNED_PER_FLOW),
        None,
        "2^40 块，覆盖写链 u = 16 的单元段就是这个形"
    );
    assert_eq!(
        state_count_to_plan(&base, &dump, Expansion::Full, u64::MAX),
        counted,
        "上限放开就有值：拦的只是块数"
    );
    let refusal = plan_blocks(&base, &dump, Expansion::Full, default_item())
        .expect_err("块数上界超过上限，排计划之前就拒");
    assert!(
        refusal.contains("超过上限") && refusal.contains(&((1u64 << 56) + 1).to_string()),
        "{refusal}"
    );
}

#[test]
fn the_two_limits_split_flows_by_the_blocks_of_their_states() {
    let (base_of_forty, forty) = {
        let dump = dump_with_one_long_segment("forty", 40);
        (dump.base_pool(), dump)
    };
    assert_eq!(
        state_count_to_plan(
            &base_of_forty,
            &forty,
            Expansion::Full,
            MOST_BLOCKS_PLANNED_PER_FLOW
        ),
        Some((1u64 << 40) + 1),
        "2^24 + 1 块排得了计划"
    );
    assert_eq!(
        state_count_to_plan(
            &base_of_forty,
            &forty,
            Expansion::Full,
            MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP
        ),
        None,
        "转储往返要摆两份计划，上限再小一档，2^24 + 1 块就不比块键"
    );
    let bound_of_forty = (1u64 << 24) + 1 + 2 + 2;
    assert_eq!(
        block_count_bound_of((1u64 << 40) + 1, forty.segments.len()),
        bound_of_forty,
        "2^24 + 1 块，两段加末节点各留一块的零头"
    );
    assert_eq!(
        state_count_to_plan(&base_of_forty, &forty, Expansion::Full, bound_of_forty),
        Some((1u64 << 40) + 1),
        "上界正好等于上限还排得了计划"
    );
    assert_eq!(
        state_count_to_plan(&base_of_forty, &forty, Expansion::Full, bound_of_forty - 1),
        None,
        "上界比上限多一块就不排"
    );
    let thirty_nine = dump_with_one_long_segment("thirty-nine", 39);
    assert_eq!(
        state_count_to_plan(
            &thirty_nine.base_pool(),
            &thirty_nine,
            Expansion::Full,
            MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP
        ),
        Some((1u64 << 39) + 1)
    );
}

#[test]
fn small_limit_refuses_the_full_plan_of_a_real_flow_and_leaves_the_in_place_subsets_plan() {
    let prepared = prepared_new_pool_stream();
    let dump = prepared.dump();
    let refusal = plan_blocks_under(&prepared.base, &dump, Expansion::Full, default_item(), 100)
        .expect_err("16777268 个状态切 257 块，上界 271 超过 100");
    assert!(refusal.contains("超过上限 100"), "{refusal}");
    let in_place_subsets = plan_blocks_under(
        &prepared.base,
        &dump,
        Expansion::InPlaceSubsets,
        default_item(),
        100,
    )
    .expect("54 个状态的甲二排得了");
    assert_eq!(in_place_subsets.state_count, 54);
    let reading = read_the_flow_under(&dump, &prepared.base, 100).expect("读得出");
    assert_eq!(
        (
            reading.row.full,
            reading.row.blocks_full,
            reading.row.blocks_in_place_subsets.is_some(),
            reading.full.is_none(),
            reading.in_place_subsets.is_some()
        ),
        (Some(16_777_268), None, true, true, true),
        "态数照报，全量不排计划写 not_planned，甲二照排"
    );
    assert!(
        reading.row.body().contains(" blocks_full=not_planned "),
        "{}",
        reading.row.body()
    );
    let unrestricted = read_the_flow(&dump, &prepared.base).expect("读得出");
    assert_eq!(unrestricted.row.full, Some(16_777_268));
    assert!(unrestricted.row.blocks_full.is_some() && unrestricted.full.is_some());
}
